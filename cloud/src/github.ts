// The GitHub App "Kanso Feedback" (docs/FEEDBACK.md › GitHub App): installed on one repository with
// Issues: read and write and Metadata: read, nothing else. The Worker signs a short-lived JWT with the
// App's private key (RS256, WebCrypto), trades it for an installation token (one hour, cached in the
// isolate) and files issues as `kanso-feedback[bot]`. The `build` label is never set here: every label
// passes `checkLabels`, whose allow-list does not hold it.

import { fromHex, hex } from "./crypto";
import type { Env } from "./env";
import { base64url } from "./protocol";

const encoder = new TextEncoder();
const API = "https://api.github.com";

/** The outside world, replaceable in tests. */
export const outbound: { fetch: (url: string, init: RequestInit) => Promise<Response> } = {
	fetch: (url, init) => fetch(url, init),
};

/** The labels the Worker may put on an issue. `build` is not here, on purpose. */
export const LABELS = [
	"from-app",
	"needs-triage",
	"bug",
	"enhancement",
	"question",
	"size:s",
	"size:m",
	"size:l",
] as const;

export function checkLabels(labels: readonly string[]): string[] {
	for (const label of labels) if (!(LABELS as readonly string[]).includes(label)) throw new Error("label_not_allowed");
	return [...labels];
}

export class GithubError extends Error {
	status: number;
	constructor(status: number) {
		super(`github ${status}`);
		this.name = "GithubError";
		this.status = status;
	}
}

export interface GithubConfig {
	appId: string;
	privateKey: string;
	repo: string;
}

/** The App's settings, or null when any is missing (nothing is filed then). */
export function configOf(env: Env): GithubConfig | null {
	const repo = env.FEEDBACK_REPO;
	if (!env.GITHUB_APP_ID || !env.GITHUB_APP_PRIVATE_KEY || !repo || !/^[\w.-]+\/[\w.-]+$/.test(repo)) return null;
	return { appId: env.GITHUB_APP_ID, privateKey: env.GITHUB_APP_PRIVATE_KEY, repo };
}

// ----------------------------------------------------------------------- JWT

const keys = new Map<string, Promise<CryptoKey>>();

function importKey(pem: string): Promise<CryptoKey> {
	let key = keys.get(pem);
	if (!key) {
		const der = pem.replace(/-----(BEGIN|END) PRIVATE KEY-----/g, "").replace(/\s+/g, "");
		const bytes = Uint8Array.from(atob(der), (c) => c.charCodeAt(0));
		key = crypto.subtle.importKey("pkcs8", bytes, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["sign"]);
		keys.set(pem, key);
	}
	return key;
}

/** The App's JWT: RS256, issued a minute ago (clock skew), valid nine minutes. */
export async function appJwt(config: Pick<GithubConfig, "appId" | "privateKey">, nowMs: number): Promise<string> {
	const seconds = Math.floor(nowMs / 1000);
	const part = (value: unknown) => base64url(encoder.encode(JSON.stringify(value)));
	const head = `${part({ alg: "RS256", typ: "JWT" })}.${part({ iat: seconds - 60, exp: seconds + 540, iss: config.appId })}`;
	const signature = await crypto.subtle.sign("RSASSA-PKCS1-v1_5", await importKey(config.privateKey), encoder.encode(head));
	return `${head}.${base64url(new Uint8Array(signature))}`;
}

// ------------------------------------------------------------ installation token

let cached: { key: string; token: string; expiresAt: number } | null = null;

/** Forgets the cached installation token (tests). */
export function resetTokenCache(): void {
	cached = null;
}

async function raw(token: string, method: string, path: string, body?: unknown): Promise<Response> {
	return outbound.fetch(`${API}${path}`, {
		method,
		headers: {
			authorization: `Bearer ${token}`,
			accept: "application/vnd.github+json",
			"x-github-api-version": "2022-11-28",
			"user-agent": "kanso-feedback",
			...(body === undefined ? {} : { "content-type": "application/json" }),
		},
		body: body === undefined ? undefined : JSON.stringify(body),
		signal: AbortSignal.timeout(20_000),
	});
}

/** An installation token for the repository, cached until five minutes before it ends. */
export async function installationToken(config: GithubConfig, nowMs: number): Promise<string> {
	const key = `${config.appId}/${config.repo}`;
	if (cached && cached.key === key && cached.expiresAt - 5 * 60_000 > nowMs) return cached.token;
	const jwt = await appJwt(config, nowMs);
	const found = await raw(jwt, "GET", `/repos/${config.repo}/installation`);
	if (!found.ok) throw new GithubError(found.status);
	const { id } = await found.json<{ id: number }>();
	const made = await raw(jwt, "POST", `/app/installations/${id}/access_tokens`, { repositories: [config.repo.split("/")[1]], permissions: { issues: "write" } });
	if (!made.ok) throw new GithubError(made.status);
	const { token, expires_at } = await made.json<{ token: string; expires_at: string }>();
	cached = { key, token, expiresAt: Date.parse(expires_at) };
	return token;
}

async function call<T>(env: Env, method: string, path: string, body?: unknown, nowMs = Date.now()): Promise<T> {
	const config = configOf(env);
	if (!config) throw new Error("github_unconfigured");
	const response = await raw(await installationToken(config, nowMs), method, path.replace("{repo}", config.repo), body);
	if (!response.ok) throw new GithubError(response.status);
	return (response.status === 204 ? null : await response.json()) as T;
}

// ------------------------------------------------------------------- operations

export interface OpenIssue {
	number: number;
	title: string;
}

/** The 50 most recently updated open issues labelled `from-app`, `bug` or `enhancement` (pull requests excluded). */
export async function openIssues(env: Env, nowMs = Date.now()): Promise<OpenIssue[]> {
	const issues = await call<{ number: number; title: string; pull_request?: unknown; labels: ({ name: string } | string)[] }[]>(
		env,
		"GET",
		"/repos/{repo}/issues?state=open&sort=updated&direction=desc&per_page=100",
		undefined,
		nowMs,
	);
	const wanted = new Set(["from-app", "bug", "enhancement"]);
	return issues
		.filter((i) => !i.pull_request && i.labels.some((l) => wanted.has(typeof l === "string" ? l : l.name)))
		.slice(0, 50)
		.map((i) => ({ number: i.number, title: i.title }));
}

export async function createIssue(env: Env, issue: { title: string; body: string; labels: readonly string[] }, nowMs = Date.now()): Promise<number> {
	const created = await call<{ number: number }>(env, "POST", "/repos/{repo}/issues", { title: issue.title, body: issue.body, labels: checkLabels(issue.labels) }, nowMs);
	return created.number;
}

export async function commentOnIssue(env: Env, issue: number, body: string, nowMs = Date.now()): Promise<void> {
	await call(env, "POST", `/repos/{repo}/issues/${issue}/comments`, { body }, nowMs);
}

export async function thumbsUp(env: Env, issue: number, nowMs = Date.now()): Promise<void> {
	await call(env, "POST", `/repos/{repo}/issues/${issue}/reactions`, { content: "+1" }, nowMs);
}

/** Drops "Reported by <name>" from the issue's footer (a reporter deleted their report). A no-op without the App. */
export async function removeCredit(env: Env, issue: number, name: string, nowMs = Date.now()): Promise<void> {
	if (!configOf(env)) return;
	const current = await call<{ body: string | null }>(env, "GET", `/repos/{repo}/issues/${issue}`, undefined, nowMs);
	const body = current.body ?? "";
	const next = stripCredit(body, name);
	if (next !== body) await call(env, "PATCH", `/repos/{repo}/issues/${issue}`, { body: next }, nowMs);
}

/** The footer without its credit line: ` · Reported by <name>`. */
export function stripCredit(body: string, name: string): string {
	return body.replace(` · Reported by ${name}`, "");
}

/** The tag of the release before `tag` among the published, non-prerelease ones, or null. */
export async function previousTag(env: Env, tag: string, nowMs = Date.now()): Promise<string | null> {
	const releases = await call<{ tag_name: string; draft: boolean; prerelease: boolean }[]>(env, "GET", "/repos/{repo}/releases?per_page=30", undefined, nowMs);
	const published = releases.filter((r) => !r.draft && !r.prerelease).map((r) => r.tag_name);
	const at = published.indexOf(tag);
	if (at >= 0) return published[at + 1] ?? null;
	return published.find((t) => t !== tag) ?? null;
}

/**
 * The commit SHAs between two tags. Unpaginated, compare lists at most 250 commits, so a release with
 * more would leave its merged reports `merged` forever; pages of 100 are read until `total_commits`.
 */
export async function commitsBetween(env: Env, base: string, head: string, nowMs = Date.now()): Promise<Set<string>> {
	const out = new Set<string>();
	for (let page = 1; page <= 30; page++) {
		const compared = await call<{ total_commits?: number; commits: { sha: string }[] }>(
			env,
			"GET",
			`/repos/{repo}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head)}?per_page=100&page=${page}`,
			undefined,
			nowMs,
		);
		for (const c of compared.commits) out.add(c.sha);
		if (compared.commits.length < 100 || out.size >= (compared.total_commits ?? 0)) break;
	}
	return out;
}

// ------------------------------------------------------------------- signature

/** Whether `header` is `sha256=` and the HMAC-SHA256 of the raw body under `secret`, compared in constant time. */
export async function signatureValid(secret: string, body: Uint8Array, header: string | null): Promise<boolean> {
	const match = /^sha256=([0-9a-f]{64})$/.exec(header ?? "");
	const key = await crypto.subtle.importKey("raw", encoder.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
	const expected = new Uint8Array(await crypto.subtle.sign("HMAC", key, body));
	// Compared whatever the header held, so every refusal costs the same.
	const given = match ? fromHex(match[1]) : new Uint8Array(32);
	return crypto.subtle.timingSafeEqual(expected, given) && match !== null;
}

/** HMAC-SHA256 hex of a body (tests sign their payloads with it). */
export async function sign(secret: string, body: Uint8Array): Promise<string> {
	const key = await crypto.subtle.importKey("raw", encoder.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
	return hex(new Uint8Array(await crypto.subtle.sign("HMAC", key, body)));
}
