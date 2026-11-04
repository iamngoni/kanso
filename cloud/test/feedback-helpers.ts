// Shared by the feedback tests: a Worker environment with the gateway and the GitHub App set up
// (with a throwaway RSA key), a fake network for both, and report builders.

import { env } from "cloudflare:workers";
import { type Env } from "../src/index";
import { outbound, resetTokenCache } from "../src/github";
import { base64url } from "../src/protocol";
import { START, testEnv } from "./helpers";

export const ACCOUNT = "0123456789abcdef0123456789abcdef";
export const REPO = "iamngoni/kanso";
export const WEBHOOK_SECRET = "test-webhook-secret";

const chars = "0123456789abcdefghijklmnopqrstuvwxyz";
export const newId = (): string => `fb_${Array.from(crypto.getRandomValues(new Uint8Array(26)), (b) => chars[b % 36]).join("")}`;
export const newKey = (): string => base64url(crypto.getRandomValues(new Uint8Array(32)));

export const APP = {
	platform: "macos",
	version: "0.9.1",
	build: "57",
	patch: null,
	os: "15.1",
	device: "Mac14,9",
	locale: "en-GB",
	screen: "editor",
	sync: "kanso",
	errors: ["update: check failed: timeout", "theme: install failed: signature"],
};

export function report(over: Record<string, unknown> = {}): Record<string, any> {
	return {
		v: 1,
		id: newId(),
		kind: "bug",
		text: "Export to PDF cuts off wide tables on the right. Expected it to wrap or shrink.",
		credit: null,
		app: structuredClone(APP),
		screenshot: null,
		...over,
	};
}

export const PNG = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3, 4, 5, 6, 7, 8]);
export const JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 0xe0, 1, 2, 3, 4]);

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
	return [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export async function withScreenshot(bytes = PNG, type = "image/png", over: Record<string, unknown> = {}) {
	return report({ screenshot: { type, bytes: bytes.length, sha256: await sha256Hex(bytes), ...over } });
}

let pem: Promise<{ pem: string; publicKey: CryptoKey }> | null = null;

/** A throwaway RSA key in the PKCS#8 PEM form the App's key comes in. */
export function appKey(): Promise<{ pem: string; publicKey: CryptoKey }> {
	pem ??= crypto.subtle.generateKey({ name: "RSASSA-PKCS1-v1_5", modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" }, true, ["sign", "verify"]).then(async (generated) => {
		const pair = generated as CryptoKeyPair;
		const der = new Uint8Array((await crypto.subtle.exportKey("pkcs8", pair.privateKey)) as ArrayBuffer);
		const body = btoa(String.fromCharCode(...der)).replace(/(.{64})/g, "$1\n");
		return { pem: `-----BEGIN PRIVATE KEY-----\n${body}\n-----END PRIVATE KEY-----\n`, publicKey: pair.publicKey };
	});
	return pem;
}

/** The test environment with triage and the App configured. */
export async function configured(over: Partial<Env> = {}): Promise<Env> {
	return testEnv({
		CF_ACCOUNT_ID: ACCOUNT,
		AIG_GATEWAY: "kanso-feedback",
		AIG_TOKEN: "aig-test-token",
		TRIAGE_MODEL: "claude-haiku-5-5",
		FEEDBACK_REPO: REPO,
		FEEDBACK_OWNER: "iamngoni",
		GITHUB_APP_ID: "12345",
		GITHUB_APP_PRIVATE_KEY: (await appKey()).pem,
		GITHUB_WEBHOOK_SECRET: WEBHOOK_SECRET,
		...over,
	});
}

export interface Sent {
	method: string;
	url: string;
	headers: Headers;
	body: any;
}

export interface Network {
	sent: Sent[];
	/** Answers for the gateway: called per request, in order of the queue; the last repeats. */
	gateway: (Response | ((sent: Sent) => Response))[];
	issues: { number: number; title: string; labels: string[]; pull_request?: unknown }[];
	nextIssue: number;
	issueBodies: Map<number, string>;
	releases: { tag_name: string; draft: boolean; prerelease: boolean }[];
	commits: Record<string, string[]>;
	/** Make these GitHub paths (substrings) answer 500. */
	failing: string[];
}

const jsonResponse = (value: unknown, status = 200) => new Response(JSON.stringify(value), { status, headers: { "content-type": "application/json" } });

/** Installs a fake for the gateway and api.github.com; call `restore` after. */
export function fakeNetwork(): Network & { restore: () => void } {
	const real = outbound.fetch;
	resetTokenCache();
	const net: Network & { restore: () => void } = { sent: [], gateway: [], issues: [], nextIssue: 301, issueBodies: new Map(), releases: [], commits: {}, failing: [], restore: () => undefined };
	let gatewayCalls = 0;
	outbound.fetch = async (url, init) => {
		const headers = new Headers(init.headers);
		const sent: Sent = { method: init.method ?? "GET", url, headers, body: typeof init.body === "string" ? JSON.parse(init.body) : init.body };
		net.sent.push(sent);
		if (url.startsWith("https://gateway.ai.cloudflare.com/")) {
			const next = net.gateway[Math.min(gatewayCalls++, net.gateway.length - 1)];
			if (!next) return jsonResponse({ error: "none" }, 500);
			return typeof next === "function" ? next(sent) : next.clone();
		}
		const path = new URL(url).pathname + new URL(url).search;
		if (net.failing.some((f) => `${sent.method} ${path}`.includes(f))) return jsonResponse({ message: "boom" }, 500);
		if (path === `/repos/${REPO}/installation`) return jsonResponse({ id: 77 });
		if (path === "/app/installations/77/access_tokens") return jsonResponse({ token: "ghs_test", expires_at: new Date(START + 3 * 3_600_000).toISOString() });
		if (path.startsWith(`/repos/${REPO}/issues?`)) return jsonResponse(net.issues.map((i) => ({ ...i, labels: i.labels.map((name) => ({ name })) })));
		if (path === `/repos/${REPO}/issues` && sent.method === "POST") {
			const number = net.nextIssue++;
			net.issueBodies.set(number, sent.body.body);
			return jsonResponse({ number }, 201);
		}
		const one = /^\/repos\/[^/]+\/[^/]+\/issues\/(\d+)$/.exec(path);
		if (one && sent.method === "GET") return jsonResponse({ body: net.issueBodies.get(Number(one[1])) ?? "" });
		if (one && sent.method === "PATCH") {
			net.issueBodies.set(Number(one[1]), sent.body.body);
			return jsonResponse({});
		}
		if (/\/issues\/\d+\/(comments|reactions)$/.test(path)) return jsonResponse({}, 201);
		if (path.startsWith(`/repos/${REPO}/releases`)) return jsonResponse(net.releases);
		const compare = /\/compare\/(.+)\.\.\.(.+)$/.exec(path);
		if (compare) return jsonResponse({ commits: (net.commits[decodeURIComponent(compare[2])] ?? []).map((sha) => ({ sha })) });
		return jsonResponse({ message: "unexpected" }, 404);
	};
	net.restore = () => {
		outbound.fetch = real;
	};
	return net;
}

/** The gateway's answer for a tool call. */
export function toolAnswer(input: unknown): Response {
	return jsonResponse({ content: [{ type: "tool_use", id: "tu_1", name: "file_triage", input }], stop_reason: "tool_use" });
}

export const TRIAGE_OK = {
	category: "bug",
	title: "PDF export cuts off wide tables",
	summary: "Exporting a note with a wide table to PDF clips the right edge of the table.",
	steps: ["Open a note with a wide table", "Export it to PDF"],
	expected: "The table wraps or shrinks to fit the page",
	actual: "The right side of the table is cut off",
	area: "export",
	platforms: ["macos"],
	severity: "broken",
	size: "m",
	duplicate_of: null,
	injection_suspected: false,
	contains_personal_data: false,
};

export const rawEnv = env;
