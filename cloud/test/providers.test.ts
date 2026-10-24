// Sign in with Apple, Google and GitHub, new-account tickets, linking and
// unlinking, end to end through handle() with every provider endpoint mocked
// (design: Kanso accounts spec, revision 3 › Sign in with Apple, Google and
// GitHub; › Sign-in routes; › Tests, the "Provider flows" and "Linking and
// method changes" rows). Property tests use the pinned fast-check seed.
/// <reference types="node" />

import { Buffer } from "node:buffer";
import { createHash, generateKeyPairSync, sign as nodeSign } from "node:crypto";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
	APPLE_ISSUER,
	APPLE_JWKS_URL,
	APPLE_TOKEN_URL,
	GITHUB_DEVICE_CODE_URL,
	GITHUB_TOKEN_URL,
	GITHUB_USER_URL,
	GOOGLE_JWKS_URL,
	GOOGLE_TOKEN_URL,
	resetProviderCache,
} from "../src/oauth";
import { APPLE_AUTHORIZE_URL, GOOGLE_AUTHORIZE_URL, candidateUsername } from "../src/providers";
import { isReserved, normalizeUsername } from "../src/usernames";
import {
	type Account,
	type Device,
	START,
	call,
	callJson,
	newCreds,
	newDevice,
	resetServer,
	setSetting,
	signUp,
	testEnv,
} from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const T0 = START + 12 * 3_600_000;

// ------------------------------------------------------------ the providers

const rsa = generateKeyPairSync("rsa", { modulusLength: 2048 });
const rsaJwk = { ...rsa.publicKey.export({ format: "jwk" }), kid: "provider-key", alg: "RS256", use: "sig" };
const appleKey = generateKeyPairSync("ec", { namedCurve: "prime256v1" });
const applePem = String(appleKey.privateKey.export({ type: "pkcs8", format: "pem" }));

const CONFIG = {
	APPLE_CLIENT_ID: "app.kanso.Kanso.web",
	APPLE_TEAM_ID: "4973Y692JU",
	APPLE_KEY_ID: "MS6Y3S5ZL7",
	APPLE_REDIRECT_URI: "https://sync.kansoapp.app/v2/auth/apple/callback",
	APPLE_PRIVATE_KEY: applePem,
	GOOGLE_CLIENT_ID: "google-client.apps.googleusercontent.com",
	GOOGLE_CLIENT_SECRET: "google-desktop-secret",
	GITHUB_CLIENT_ID: "Iv1.github-client",
};
const penv = () => testEnv(CONFIG);

/** Profile data each provider offers and the Worker must never keep or log. */
const PII = {
	email: "person.private@example.com",
	name: "Private Person",
	login: "private-login-name",
	avatar: "https://avatars.example.com/u/1",
	accessToken: "gho_SECRETACCESSTOKEN0000000000000000",
	deviceCode: "3584d83530557fdd1f46af8289938c8ef79f9dc5",
	appleCode: "c-apple-authorization-code-0001",
	googleCode: "4/0-google-authorization-code",
	googleRefresh: "1//google-refresh-token",
};

const b64 = (value: string | Uint8Array) => Buffer.from(value).toString("base64url");

function idToken(claims: Record<string, unknown>): string {
	const header = b64(JSON.stringify({ alg: "RS256", kid: "provider-key", typ: "JWT" }));
	const body = b64(JSON.stringify(claims));
	const signature = nodeSign("RSA-SHA256", Buffer.from(`${header}.${body}`), rsa.privateKey);
	return `${header}.${body}.${b64(signature)}`;
}

/** The mocked provider side: what each endpoint answers next. */
interface Providers {
	apple: { sub: string; nonce: string | null; tokenStatus: number };
	google: { sub: string; nonce: string | null; tokenStatus: number };
	github: { id: number; answers: Record<string, unknown>[]; deviceStatus: number; interval: number };
	calls: string[];
}

let providers: Providers;
const jsonResponse = (value: unknown, status = 200) =>
	new Response(JSON.stringify(value), { status, headers: { "content-type": "application/json" } });

function installProviders(): void {
	providers = {
		apple: { sub: "001234.apple-subject.0001", nonce: null, tokenStatus: 200 },
		google: { sub: "109876543210987654321", nonce: null, tokenStatus: 200 },
		github: { id: 583231, answers: [], deviceStatus: 200, interval: 5 },
		calls: [],
	};
	const now = () => Math.floor(T0 / 1000);
	vi.stubGlobal(
		"fetch",
		vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
			const url = String(input instanceof Request ? input.url : input);
			providers.calls.push(url);
			const form = new URLSearchParams(typeof init?.body === "string" ? init.body : "");
			if (url === APPLE_JWKS_URL || url === GOOGLE_JWKS_URL) return jsonResponse({ keys: [rsaJwk] });
			if (url === APPLE_TOKEN_URL) {
				if (providers.apple.tokenStatus !== 200) return jsonResponse({ error: "server_error" }, providers.apple.tokenStatus);
				if (form.get("code") !== PII.appleCode) return jsonResponse({ error: "invalid_grant" }, 400);
				expect(form.get("client_id")).toBe(CONFIG.APPLE_CLIENT_ID);
				expect(form.get("redirect_uri")).toBe(CONFIG.APPLE_REDIRECT_URI);
				return jsonResponse({
					access_token: "apple-access",
					refresh_token: "apple-refresh",
					id_token: idToken({
						iss: APPLE_ISSUER,
						aud: CONFIG.APPLE_CLIENT_ID,
						iat: now(),
						exp: now() + 600,
						sub: providers.apple.sub,
						nonce: providers.apple.nonce,
						email: PII.email,
						email_verified: true,
						is_private_email: true,
					}),
				});
			}
			if (url === GOOGLE_TOKEN_URL) {
				if (providers.google.tokenStatus !== 200) return jsonResponse({ error: "backend" }, providers.google.tokenStatus);
				if (form.get("code") !== PII.googleCode) return jsonResponse({ error: "invalid_grant" }, 400);
				expect(form.get("client_secret")).toBe(CONFIG.GOOGLE_CLIENT_SECRET);
				expect(form.get("redirect_uri")).toMatch(/^http:\/\/127\.0\.0\.1:\d+$/);
				return jsonResponse({
					access_token: "ya29.google-access",
					refresh_token: PII.googleRefresh,
					id_token: idToken({
						iss: "https://accounts.google.com",
						aud: CONFIG.GOOGLE_CLIENT_ID,
						iat: now(),
						exp: now() + 600,
						sub: providers.google.sub,
						nonce: providers.google.nonce,
						email: PII.email,
						name: PII.name,
						picture: PII.avatar,
					}),
				});
			}
			if (url === GITHUB_DEVICE_CODE_URL) {
				expect(form.get("client_id")).toBe(CONFIG.GITHUB_CLIENT_ID);
				expect(form.has("scope")).toBe(false);
				if (providers.github.deviceStatus !== 200) return jsonResponse({ error: "unavailable" }, providers.github.deviceStatus);
				return jsonResponse({
					device_code: PII.deviceCode,
					user_code: "WDJB-MJHT",
					verification_uri: "https://github.com/login/device",
					expires_in: 900,
					interval: providers.github.interval,
				});
			}
			if (url === GITHUB_TOKEN_URL) {
				expect(form.get("device_code")).toBe(PII.deviceCode);
				return jsonResponse(providers.github.answers.shift() ?? { error: "authorization_pending" });
			}
			if (url === GITHUB_USER_URL) {
				const headers = new Headers(init?.headers);
				expect(headers.get("authorization")).toBe(`Bearer ${PII.accessToken}`);
				expect(headers.get("user-agent")).toMatch(/^kanso-sync\//);
				return jsonResponse({ id: providers.github.id, login: PII.login, email: PII.email, name: PII.name, avatar_url: PII.avatar });
			}
			throw new Error(`unexpected fetch ${url}`);
		}),
	);
}

const consoleCalls: unknown[][] = [];

beforeEach(async () => {
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
	resetProviderCache();
	installProviders();
	consoleCalls.length = 0;
	for (const level of ["log", "info", "warn", "error", "debug"] as const) {
		vi.spyOn(console, level).mockImplementation((...args: unknown[]) => {
			consoleCalls.push(args);
		});
	}
	await resetServer({ signup: "open", signups_per_day: "1000" });
});

afterEach(() => {
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
});

// ------------------------------------------------------------ the app's side

const wireDevice = (d: Device) => ({ id: d.id, proof: d.proof });
const namedDevice = (d: Device) => ({ id: d.id, proof: d.proof, sealed_name: d.sealedName });
const as = (path: string, json: unknown, more: { now?: number; token?: string; headers?: Record<string, string>; ip?: string } = {}) =>
	callJson(path, { env: penv(), now: more.now ?? T0, json, token: more.token, headers: more.headers, ip: more.ip });

type Answer = { status: number; body: any };

/** Apple: start, Apple's form post to the callback (from Apple's origin), then the app's poll. */
async function apple(d: Device, sub: string, now = T0): Promise<Answer & { flowId: string }> {
	const started = await as("/v2/auth/apple/start", { device: wireDevice(d) }, { now });
	expect(started.status).toBe(200);
	const flowId = await appleCallback(started.body.auth_url, sub, now);
	const polled = await as("/v2/auth/apple/poll", { flow_id: flowId, device: wireDevice(d) }, { now: now + 4000 });
	return { ...polled, flowId };
}

async function appleCallback(authUrl: string, sub: string, now = T0): Promise<string> {
	const url = new URL(authUrl);
	providers.apple.sub = sub;
	providers.apple.nonce = url.searchParams.get("nonce");
	const response = await postCallback({ state: url.searchParams.get("state") ?? "", code: PII.appleCode }, now);
	expect(response.status).toBe(200);
	const page = await response.text();
	const match = /kanso:\/\/auth\/apple\?flow=([A-Za-z0-9_-]{43})/.exec(page);
	expect(match).not.toBeNull();
	return match![1];
}

function postCallback(fields: Record<string, string>, now = T0, headers: Record<string, string> = { origin: "https://appleid.apple.com" }) {
	return call("/v2/auth/apple/callback", {
		env: penv(),
		now,
		method: "POST",
		body: new URLSearchParams(fields).toString(),
		headers: { "content-type": "application/x-www-form-urlencoded", ...headers },
	});
}

function pkce(): { verifier: string; challenge: string } {
	const verifier = b64(crypto.getRandomValues(new Uint8Array(48)));
	return { verifier, challenge: createHash("sha256").update(verifier).digest("base64url") };
}

/** Google: start with a loopback URI and a PKCE challenge, then the finish with the code and verifier. */
async function google(d: Device, sub: string, now = T0): Promise<Answer & { flowId: string }> {
	const { verifier, challenge } = pkce();
	const started = await as(
		"/v2/auth/google/start",
		{ device: wireDevice(d), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge },
		{ now },
	);
	expect(started.status).toBe(200);
	providers.google.sub = sub;
	providers.google.nonce = started.body.nonce;
	const finished = await as(
		"/v2/auth/google/finish",
		{ flow_id: started.body.flow_id, code: PII.googleCode, code_verifier: verifier, device: wireDevice(d) },
		{ now: now + 1000 },
	);
	return { ...finished, flowId: started.body.flow_id };
}

/** GitHub: start, then one poll after the interval with GitHub's token ready. */
async function github(d: Device, id: number, now = T0): Promise<Answer & { flowId: string }> {
	const started = await as("/v2/auth/github/start", { device: wireDevice(d) }, { now });
	expect(started.status).toBe(200);
	providers.github.id = id;
	providers.github.answers.push({ access_token: PII.accessToken, token_type: "bearer", scope: "" });
	const polled = await as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device: wireDevice(d) }, { now: now + 5000 });
	return { ...polled, flowId: started.body.flow_id };
}

const SIGN_IN = {
	apple: (d: Device, n: number, now?: number) => apple(d, `00${n}.apple.subject`, now),
	google: (d: Device, n: number, now?: number) => google(d, `1${String(n).padStart(20, "0")}`, now),
	github: (d: Device, n: number, now?: number) => github(d, 1000 + n, now),
} as const;
type ProviderName = keyof typeof SIGN_IN;
const PROVIDERS = Object.keys(SIGN_IN) as ProviderName[];

function socialSignupBody(ticket: string, username: string, d: Device) {
	const creds = newCreds(username);
	return {
		ticket,
		username,
		account_id: creds.accountId,
		recovery_auth_key: creds.recoveryAuthKey,
		recovery_wrapped_key: creds.recoveryWrappedKey,
		key_id: creds.keyId,
		device: namedDevice(d),
	};
}

/** A new account made through `provider`: the ticket, then social sign-up. */
async function socialAccount(provider: ProviderName, n: number, username: string) {
	const d = newDevice();
	const flow = await SIGN_IN[provider](d, n);
	expect(flow.status).toBe(200);
	expect(flow.body.kind).toBe("new_account");
	const body = socialSignupBody(flow.body.ticket, username, d);
	const made = await as("/v2/auth/social/signup", body);
	expect(made.status).toBe(201);
	return { device: d, body, session: made.body };
}

const passwordProof = (a: Account) => ({ kind: "password", auth_key: a.creds.authKey });

async function linkStart(a: Account, provider: ProviderName, proof: unknown, intent: "link" | "proof" = "link") {
	return as(`/v2/account/sign-in/identities/${provider}/start`, { intent, proof, device: wireDevice(a.device) }, { token: a.token });
}

/** Completes a started link or proof flow the way the app would. */
async function linkFinish(a: Account, provider: ProviderName, started: Answer, n: number): Promise<Answer> {
	const device = wireDevice(a.device);
	if (provider === "apple") {
		const flowId = await appleCallback(started.body.auth_url, `00${n}.apple.subject`);
		return as("/v2/auth/apple/poll", { flow_id: flowId, device });
	}
	if (provider === "google") throw new Error("use googleLink");
	providers.github.id = 1000 + n;
	providers.github.answers.push({ access_token: PII.accessToken });
	return as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device }, { now: T0 + 5000 });
}

async function dump(): Promise<string> {
	const tables = ["accounts", "account_identities", "auth_flows", "sessions", "limits", "account_passkeys"];
	const rows = await env.DB.batch(tables.map((t) => env.DB.prepare(`SELECT * FROM ${t}`)));
	return JSON.stringify(rows.map((r) => r.results));
}

// ------------------------------------------------------------------- tests

describe("each provider creates an account and signs in", () => {
	for (const provider of PROVIDERS) {
		it(`${provider}: a ticket, a sign-up, then a session on another device`, async () => {
			const first = newDevice();
			const flow = await SIGN_IN[provider](first, 1);
			expect(flow.status).toBe(200);
			expect(flow.body).toEqual({ kind: "new_account", ticket: flow.flowId, suggested_username: expect.any(String) });
			const suggested = flow.body.suggested_username as string;
			expect(normalizeUsername(suggested)).toBe(suggested);
			expect(isReserved(suggested)).toBe(false);

			// The same finish or poll again: the same ticket, never an account.
			const again =
				provider === "apple"
					? await as("/v2/auth/apple/poll", { flow_id: flow.flowId, device: wireDevice(first) })
					: provider === "github"
						? await as("/v2/auth/github/poll", { flow_id: flow.flowId, device: wireDevice(first) }, { now: T0 + 20_000 })
						: null;
			if (again) expect(again.body).toEqual(flow.body);

			const body = socialSignupBody(flow.body.ticket, "river", first);
			const made = await as("/v2/auth/social/signup", body);
			expect(made.status).toBe(201);
			expect(made.body).toMatchObject({
				kind: "session",
				account_id: body.account_id,
				username: "river",
				key_id: body.key_id,
				has_password: false,
				passkey_wrapped_key: null,
				evicted_device: null,
			});
			expect(made.body.token).toMatch(/^kanso_[A-Za-z0-9_-]{43}$/);
			const row = await env.DB.prepare("SELECT has_password FROM accounts WHERE id = ?1").bind(body.account_id).first();
			expect(row).toEqual({ has_password: 0 });

			// The session works, and the method list names the provider.
			const methods = await callJson("/v2/account/sign-in", { env: penv(), now: T0, token: made.body.token });
			expect(methods.body).toEqual({
				username: "river",
				has_password: false,
				identities: [{ provider, created_at: T0 }],
				passkeys: [],
				recovery: true,
			});

			// A retried sign-up (its answer lost): 201, the same account, a fresh token.
			const retried = await as("/v2/auth/social/signup", body);
			expect(retried.status).toBe(201);
			expect(retried.body.account_id).toBe(body.account_id);
			expect(retried.body.token).not.toBe(made.body.token);
			const devices = await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE account_id = ?1").bind(body.account_id).first();
			expect(devices).toEqual({ n: 1 });

			// The same identity on a new device signs in to that account, locked: no wrap of any kind.
			const second = newDevice();
			const signedIn = await SIGN_IN[provider](second, 1, T0 + 60_000);
			expect(signedIn.status).toBe(200);
			expect(signedIn.body).toMatchObject({ kind: "session", account_id: body.account_id, username: "river", has_password: false });
			expect(signedIn.body).not.toHaveProperty("wrapped_key");
			expect(signedIn.body.passkey_wrapped_key).toBeNull();
		});
	}

	it("a provider sign-in of a password account says it has a password, and never sends the password wrap", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const started = await linkStart(alice, "github", passwordProof(alice));
		expect((await linkFinish(alice, "github", started, 7)).body.kind).toBe("linked");
		const other = newDevice();
		const signedIn = await github(other, 1007, T0 + 60_000);
		expect(signedIn.body).toMatchObject({ kind: "session", account_id: alice.creds.accountId, has_password: true });
		expect(JSON.stringify(signedIn.body)).not.toContain(alice.creds.wrappedKey);
	});
});

describe("Apple's web flow", () => {
	it("starts with the Services ID, form_post, no scope, and a state and nonce", async () => {
		const started = await as("/v2/auth/apple/start", { device: wireDevice(newDevice()) });
		expect(started.body).toEqual({ flow_id: expect.any(String), auth_url: expect.any(String), expires_at: T0 + 600_000 });
		const url = new URL(started.body.auth_url);
		expect(`${url.origin}${url.pathname}`).toBe(APPLE_AUTHORIZE_URL);
		expect(Object.fromEntries(url.searchParams)).toEqual({
			client_id: CONFIG.APPLE_CLIENT_ID,
			redirect_uri: CONFIG.APPLE_REDIRECT_URI,
			response_type: "code",
			response_mode: "form_post",
			state: expect.stringMatching(/^[A-Za-z0-9_-]{43}\.[A-Za-z0-9_-]{43}$/),
			nonce: expect.stringMatching(/^[A-Za-z0-9_-]{43}$/),
		});
		// Only hashes of the state and nonce are stored.
		const flow = await env.DB.prepare("SELECT * FROM auth_flows WHERE id = ?1").bind(started.body.flow_id).first<Record<string, unknown>>();
		expect(JSON.stringify(flow)).not.toContain(url.searchParams.get("nonce"));
		expect(JSON.stringify(flow)).not.toContain(url.searchParams.get("state")!.slice(44));
	});

	it("exchanges the code with the redirect URI stored on the flow", async () => {
		const d = newDevice();
		const started = await callJson("/v2/auth/apple/start", {
			env: testEnv(CONFIG),
			now: T0,
			json: { device: wireDevice(d) },
		});
		const url = new URL(started.body.auth_url);
		providers.apple.nonce = url.searchParams.get("nonce");
		const changedEnv = testEnv({ ...CONFIG, APPLE_REDIRECT_URI: "https://changed.example/auth/apple/callback" });
		const response = await call("/v2/auth/apple/callback", {
			env: changedEnv,
			now: T0,
			method: "POST",
			body: new URLSearchParams({ state: url.searchParams.get("state")!, code: PII.appleCode }).toString(),
			headers: { origin: "https://appleid.apple.com", "content-type": "application/x-www-form-urlencoded" },
		});
		expect(response.status).toBe(200);
		expect(providers.calls.filter((endpoint) => endpoint === APPLE_TOKEN_URL)).toHaveLength(1);
	});

	it("the callback takes Apple's origin, or none, and refuses any other", async () => {
		for (const headers of [{ origin: "https://appleid.apple.com" }, {}]) {
			const d = newDevice();
			const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
			const url = new URL(started.body.auth_url);
			providers.apple.nonce = url.searchParams.get("nonce");
			const response = await postCallback({ state: url.searchParams.get("state")!, code: PII.appleCode }, T0, headers as Record<string, string>);
			expect(response.status).toBe(200);
			expect(response.headers.get("content-type")).toMatch(/^text\/html/);
		}
		const evil = await postCallback({ state: "x", code: "y" }, T0, { origin: "https://evil.example" });
		expect(evil.status).toBe(403);
		expect((await evil.json<{ error: string }>()).error).toBe("forbidden_origin");
	});

	it("the callback answers a page, never JSON: wrong type, bad state, a cancel, a replay", async () => {
		const d = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const url = new URL(started.body.auth_url);
		const state = url.searchParams.get("state")!;
		providers.apple.nonce = url.searchParams.get("nonce");

		const json = await call("/v2/auth/apple/callback", {
			env: penv(),
			now: T0,
			json: { state, code: PII.appleCode },
			headers: { origin: "https://appleid.apple.com" },
		});
		expect(json.status).toBe(415);
		expect(json.headers.get("content-type")).toMatch(/^text\/html/);

		const forged = await postCallback({ state: `${state.slice(0, 44)}${"A".repeat(43)}`, code: PII.appleCode });
		expect(forged.status).toBe(400);
		const page = await forged.text();
		expect(page).not.toContain("kanso://");
		expect(page).not.toContain(started.body.flow_id);
		expect((await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(d) })).body).toEqual({ status: "pending" });

		const ok = await postCallback({ state, code: PII.appleCode });
		expect(ok.status).toBe(200);
		// A replayed post (the same state and code): refused, and the flow keeps its subject.
		const replay = await postCallback({ state, code: PII.appleCode });
		expect(replay.status).toBe(400);
		expect(await replay.text()).not.toContain("kanso://");
		expect((await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(d) })).body.kind).toBe("new_account");
	});

	it("a cancel at Apple ends the poll with provider_denied", async () => {
		const d = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const state = new URL(started.body.auth_url).searchParams.get("state")!;
		const page = await postCallback({ state, error: "user_cancelled_authorize" });
		expect(page.status).toBe(400);
		const polled = await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(d) });
		expect(polled).toMatchObject({ status: 403, body: { error: "provider_denied" } });
	});

	it("a token with another flow's nonce is refused", async () => {
		const d = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const state = new URL(started.body.auth_url).searchParams.get("state")!;
		providers.apple.nonce = "another-flows-nonce";
		expect((await postCallback({ state, code: PII.appleCode })).status).toBe(400);
		expect((await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(d) })).body.error).toBe("provider_denied");
	});

	it("a GET callback reads the query", async () => {
		const d = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const url = new URL(started.body.auth_url);
		providers.apple.nonce = url.searchParams.get("nonce");
		const query = new URLSearchParams({ state: url.searchParams.get("state")!, code: PII.appleCode });
		const response = await call(`/v2/auth/apple/callback?${query}`, { env: penv(), now: T0 });
		expect(response.status).toBe(200);
	});

	it("Apple down: the callback says so and the poll stays pending", async () => {
		const d = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const url = new URL(started.body.auth_url);
		providers.apple.tokenStatus = 502;
		expect((await postCallback({ state: url.searchParams.get("state")!, code: PII.appleCode })).status).toBe(503);
		expect((await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(d) })).body).toEqual({ status: "pending" });
	});
});

describe("Google's loopback and PKCE", () => {
	it("starts with scope openid, S256, online access and the loopback URI", async () => {
		const { challenge } = pkce();
		const started = await as("/v2/auth/google/start", {
			device: wireDevice(newDevice()),
			redirect_uri: "http://127.0.0.1:49152",
			code_challenge: challenge,
		});
		expect(started.status).toBe(200);
		const url = new URL(started.body.auth_url);
		expect(`${url.origin}${url.pathname}`).toBe(GOOGLE_AUTHORIZE_URL);
		expect(Object.fromEntries(url.searchParams)).toEqual({
			client_id: CONFIG.GOOGLE_CLIENT_ID,
			redirect_uri: "http://127.0.0.1:49152",
			response_type: "code",
			scope: "openid",
			state: started.body.state,
			nonce: started.body.nonce,
			code_challenge: challenge,
			code_challenge_method: "S256",
			access_type: "online",
		});
	});

	it("accepts a redirect URI exactly when it is http://127.0.0.1:<1024–65535>", async () => {
		const { challenge } = pkce();
		const shaped = fc.oneof(
			fc.integer({ min: 0, max: 99999 }).map((port) => `http://127.0.0.1:${port}`),
			fc.constantFrom(
				"http://localhost:53682",
				"https://127.0.0.1:53682",
				"http://127.0.0.1:53682/",
				"http://127.0.0.1:53682/cb",
				"http://127.0.0.1:53682?x=1",
				"http://127.0.0.1:053682",
				"http://127.0.0.2:53682",
				"http://[::1]:53682",
				"http://127.0.0.1",
				"kanso://auth",
			),
		);
		await fc.assert(
			fc.asyncProperty(shaped, async (redirect) => {
				const r = await as("/v2/auth/google/start", { device: wireDevice(newDevice()), redirect_uri: redirect, code_challenge: challenge });
				const port = /^http:\/\/127\.0\.0\.1:([1-9][0-9]{3,4})$/.exec(redirect);
				const valid = port !== null && Number(port[1]) >= 1024 && Number(port[1]) <= 65535;
				expect(r.status).toBe(valid ? 200 : 400);
			}),
			runs(60),
		);
	});

	it("refuses a verifier that doesn't match, and a missing challenge", async () => {
		const d = newDevice();
		const { challenge } = pkce();
		const started = await as("/v2/auth/google/start", { device: wireDevice(d), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge });
		providers.google.nonce = started.body.nonce;
		const wrong = await as("/v2/auth/google/finish", {
			flow_id: started.body.flow_id,
			code: PII.googleCode,
			code_verifier: pkce().verifier,
			device: wireDevice(d),
		});
		expect(wrong).toMatchObject({ status: 401, body: { error: "invalid_credentials" } });
		expect(providers.calls).not.toContain(GOOGLE_TOKEN_URL);
		const missing = await as("/v2/auth/google/start", { device: wireDevice(d), redirect_uri: "http://127.0.0.1:53682" });
		expect(missing.status).toBe(400);
	});

	it("is idempotent: a finish repeated after a lost answer doesn't exchange the code again", async () => {
		const d = newDevice();
		const { verifier, challenge } = pkce();
		const started = await as("/v2/auth/google/start", { device: wireDevice(d), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge });
		providers.google.nonce = started.body.nonce;
		const body = { flow_id: started.body.flow_id, code: PII.googleCode, code_verifier: verifier, device: wireDevice(d) };
		const first = await as("/v2/auth/google/finish", body);
		const second = await as("/v2/auth/google/finish", body);
		expect(second.body).toEqual(first.body);
		expect(providers.calls.filter((u) => u === GOOGLE_TOKEN_URL)).toHaveLength(1);
	});

	it("an exchange that crosses flow expiry cannot create a session", async () => {
		const existing = await socialAccount("google", 61, "google-owner");
		const device = newDevice();
		const { verifier, challenge } = pkce();
		const startNow = T0 + 60_000;
		const started = await as(
			"/v2/auth/google/start",
			{ device: wireDevice(device), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge },
			{ now: startNow },
		);
		providers.google.nonce = started.body.nonce;
		let wallNow = startNow + 1000;
		vi.spyOn(Date, "now").mockImplementation(() => wallNow);
		const originalFetch = vi.mocked(fetch).getMockImplementation();
		if (!originalFetch) throw new Error("the provider fetch mock must be installed");
		vi.stubGlobal(
			"fetch",
			vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
				const response = await originalFetch(input, init);
				const url = String(input instanceof Request ? input.url : input);
				if (url === GOOGLE_TOKEN_URL) wallNow = started.body.expires_at + 1;
				return response;
			}),
		);
		const result = await as(
			"/v2/auth/google/finish",
			{ flow_id: started.body.flow_id, code: PII.googleCode, code_verifier: verifier, device: wireDevice(device) },
			{ now: startNow + 1000 },
		);
		expect(result).toMatchObject({ status: 400, body: { error: "flow_expired" } });
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE account_id = ?1").bind(existing.body.account_id).first()).toEqual({
			n: 1,
		});
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities WHERE account_id = ?1").bind(existing.body.account_id).first()).toEqual({
			n: 1,
		});
		const flow = await env.DB.prepare("SELECT consumed_at FROM auth_flows WHERE id = ?1").bind(started.body.flow_id).first();
		expect(flow).toEqual({ consumed_at: null });
	});

	it("parallel finishes create one session for the device and no extra identity", async () => {
		const existing = await socialAccount("google", 62, "google-owner");
		const device = newDevice();
		const { verifier, challenge } = pkce();
		const startNow = T0 + 60_000;
		const started = await as(
			"/v2/auth/google/start",
			{ device: wireDevice(device), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge },
			{ now: startNow },
		);
		providers.google.nonce = started.body.nonce;
		const body = { flow_id: started.body.flow_id, code: PII.googleCode, code_verifier: verifier, device: wireDevice(device) };
		const answers = await Promise.all([
			as("/v2/auth/google/finish", body, { now: startNow + 1000 }),
			as("/v2/auth/google/finish", body, { now: startNow + 1000 }),
		]);
		expect(answers.filter((answer) => answer.body.kind === "session")).toHaveLength(1);
		expect(answers.filter((answer) => answer.body.error === "flow_expired")).toHaveLength(1);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE account_id = ?1").bind(existing.body.account_id).first()).toEqual({
			n: 2,
		});
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities WHERE account_id = ?1").bind(existing.body.account_id).first()).toEqual({
			n: 1,
		});
	});

	it("an expired code is invalid_credentials and Google down is provider_unavailable", async () => {
		const d = newDevice();
		const { verifier, challenge } = pkce();
		const started = await as("/v2/auth/google/start", { device: wireDevice(d), redirect_uri: "http://127.0.0.1:53682", code_challenge: challenge });
		providers.google.nonce = started.body.nonce;
		const expired = await as("/v2/auth/google/finish", {
			flow_id: started.body.flow_id,
			code: "4/0-expired-code",
			code_verifier: verifier,
			device: wireDevice(d),
		});
		expect(expired).toMatchObject({ status: 401, body: { error: "invalid_credentials" } });
		providers.google.tokenStatus = 503;
		const down = await as("/v2/auth/google/finish", {
			flow_id: started.body.flow_id,
			code: PII.googleCode,
			code_verifier: verifier,
			device: wireDevice(d),
		});
		expect(down).toMatchObject({ status: 503, body: { error: "provider_unavailable", provider: "google" } });
	});
});

describe("GitHub's device flow", () => {
	async function started(d: Device) {
		const r = await as("/v2/auth/github/start", { device: wireDevice(d) });
		expect(r.body).toEqual({
			flow_id: expect.any(String),
			user_code: "WDJB-MJHT",
			verification_uri: "https://github.com/login/device",
			interval: providers.github.interval,
			expires_in: 600,
		});
		return r.body.flow_id as string;
	}
	const poll = (flowId: string, d: Device, now: number, ip?: string) =>
		as("/v2/auth/github/poll", { flow_id: flowId, device: wireDevice(d) }, { now, ...(ip ? { ip } : {}) });

	it("stores GitHub's interval in seconds and cumulatively adds five for each slow_down", async () => {
		providers.github.interval = 11;
		const d = newDevice();
		const flowId = await started(d);
		const stored = await env.DB.prepare("SELECT poll_interval_seconds, next_poll_at FROM auth_flows WHERE id = ?1")
			.bind(flowId)
			.first();
		expect(stored).toEqual({ poll_interval_seconds: 11, next_poll_at: T0 + 11_000 });
		const early = await poll(flowId, d, T0 + 10_000);
		expect(early).toMatchObject({ status: 429, body: { error: "rate_limited", limit: "provider-poll", retry_after: 1 } });
		expect(providers.calls).not.toContain(GITHUB_TOKEN_URL);
		providers.github.answers.push({ error: "slow_down" });
		expect((await poll(flowId, d, T0 + 11_000)).body).toEqual({ status: "pending", slow_down: true });
		let schedule = await env.DB.prepare("SELECT poll_interval_seconds, next_poll_at FROM auth_flows WHERE id = ?1")
			.bind(flowId)
			.first();
		expect(schedule).toEqual({ poll_interval_seconds: 16, next_poll_at: T0 + 27_000 });
		expect((await poll(flowId, d, T0 + 26_000)).status).toBe(429);
		providers.github.answers.push({ error: "slow_down" });
		expect((await poll(flowId, d, T0 + 27_000)).body).toEqual({ status: "pending", slow_down: true });
		schedule = await env.DB.prepare("SELECT poll_interval_seconds, next_poll_at FROM auth_flows WHERE id = ?1")
			.bind(flowId)
			.first();
		expect(schedule).toEqual({ poll_interval_seconds: 21, next_poll_at: T0 + 48_000 });
		expect((await poll(flowId, d, T0 + 47_000)).status).toBe(429);
		providers.github.answers.push({ access_token: PII.accessToken });
		const done = await poll(flowId, d, T0 + 48_000);
		expect(done.body.kind).toBe("new_account");
		// Repeated: the same answer, and GitHub isn't asked again.
		const before = providers.calls.length;
		expect((await poll(flowId, d, T0 + 49_000)).body).toEqual(done.body);
		expect(providers.calls.length).toBe(before);
	});

	it("parallel polls call GitHub once", async () => {
		const d = newDevice();
		const flowId = await started(d);
		const answers = await Promise.all([poll(flowId, d, T0 + 5000), poll(flowId, d, T0 + 5000), poll(flowId, d, T0 + 5000)]);
		expect(answers.map((a) => a.status).sort()).toEqual([200, 429, 429]);
		expect(providers.calls.filter((u) => u === GITHUB_TOKEN_URL)).toHaveLength(1);
	});

	it("limits the eleventh unlinked identity across IP groups", async () => {
		for (let index = 0; index < 11; index++) {
			const d = newDevice();
			const flowId = await started(d);
			providers.github.answers.push({ access_token: PII.accessToken });
			const answer = await poll(flowId, d, T0 + 5000, `198.51.${index + 1}.1`);
			if (index < 10) {
				expect(answer).toMatchObject({ status: 200, body: { kind: "new_account" } });
			} else {
				expect(answer).toMatchObject({ status: 429, body: { error: "rate_limited", limit: "login-user" } });
			}
		}
	});

	it("expired_token is flow_expired and access_denied is provider_denied", async () => {
		const d = newDevice();
		const a = await started(d);
		providers.github.answers.push({ error: "expired_token" });
		expect(await poll(a, d, T0 + 5000)).toMatchObject({ status: 410, body: { error: "flow_expired" } });
		const b = await started(d);
		providers.github.answers.push({ error: "access_denied" });
		expect((await poll(b, d, T0 + 5000))).toMatchObject({ status: 403, body: { error: "provider_denied" } });
		expect((await poll(b, d, T0 + 11_000)).body.error).toBe("provider_denied");
	});

	it("GitHub down is provider_unavailable", async () => {
		providers.github.deviceStatus = 502;
		expect(await as("/v2/auth/github/start", { device: wireDevice(newDevice()) })).toMatchObject({
			status: 503,
			body: { error: "provider_unavailable", provider: "github" },
		});
	});

	it("caps a flow's polls with provider-poll", async () => {
		const d = newDevice();
		const flowId = await started(d);
		let last: Answer = { status: 0, body: null };
		for (let i = 0; i < 301; i++) last = await poll(flowId, d, T0 + 1000);
		expect(last.body).toMatchObject({ error: "rate_limited", limit: "provider-poll" });
	});
});

describe("flows belong to one device", () => {
	it("a poll or finish from another device, or after ten minutes, is refused", async () => {
		const d = newDevice();
		const intruder = newDevice();
		const started = await as("/v2/auth/apple/start", { device: wireDevice(d) });
		const flowId = started.body.flow_id;
		expect((await as("/v2/auth/apple/poll", { flow_id: flowId, device: wireDevice(intruder) })).body.error).toBe("bad_request");
		expect((await as("/v2/auth/apple/poll", { flow_id: flowId, device: { id: d.id, proof: intruder.proof } })).body.error).toBe(
			"bad_request",
		);
		expect((await as("/v2/auth/github/poll", { flow_id: flowId, device: wireDevice(d) })).body.error).toBe("bad_request");
		expect((await as("/v2/auth/apple/poll", { flow_id: flowId, device: wireDevice(d) }, { now: T0 + 600_000 })).body.error).toBe(
			"flow_expired",
		);
	});

	it("a ticket is the device's, single use, and expires with the flow", async () => {
		const d = newDevice();
		const flow = await github(d, 42);
		const stolen = await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "thief", newDevice()));
		expect(stolen.body.error).toBe("bad_request");
		const late = await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "late", d), { now: T0 + 600_000 });
		expect(late.body.error).toBe("flow_expired");
		expect((await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "keeper", d))).status).toBe(201);
		// Used: another sign-up with it (a different account) is refused.
		expect((await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "second", d))).body.error).toBe("flow_expired");
	});

	it("parallel sign-ups with one ticket make one account", async () => {
		const d = newDevice();
		const flow = await google(d, "1234567");
		const answers = await Promise.all(
			["one-name", "two-name", "three-name"].map((name) => as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, name, d))),
		);
		expect(answers.filter((a) => a.status === 201)).toHaveLength(1);
		expect(answers.filter((a) => a.status !== 201).every((a) => ["identity_taken", "flow_expired"].includes(a.body.error))).toBe(true);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM accounts").first()).toEqual({ n: 1 });
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities").first()).toEqual({ n: 1 });
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions").first()).toEqual({ n: 1 });
	});

	it("sign-up follows the usual gates: names and closed sign-ups", async () => {
		await signUp("taken", { env: penv(), now: T0 });
		const d = newDevice();
		const flow = await github(d, 77);
		expect((await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "taken", d))).body.error).toBe("username_taken");
		expect((await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "admin", d))).body.error).toBe("reserved_username");
		expect((await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "no", d))).body.error).toBe("invalid_username");
		await setSetting("signup", "closed");
		expect(await as("/v2/auth/social/signup", socialSignupBody(flow.body.ticket, "free-name", d))).toMatchObject({
			status: 403,
			body: { error: "signup_closed", reason: "closed" },
		});
	});
});

describe("linking", () => {
	it("needs a password proof; a session alone, a provider proof or a wrong password can't link", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		expect((await linkStart(alice, "github", undefined)).body).toMatchObject({ error: "reauth_required", reason: "reauth" });
		expect((await linkStart(alice, "github", { kind: "provider", flow: "A".repeat(43) })).body.error).toBe("reauth_required");
		expect((await linkStart(alice, "github", { kind: "password", auth_key: newCreds("x").authKey })).body.error).toBe(
			"invalid_credentials",
		);
		const started = await linkStart(alice, "github", passwordProof(alice));
		expect(started.status).toBe(200);
		const linked = await linkFinish(alice, "github", started, 5);
		expect(linked.body).toEqual({ kind: "linked", provider: "github", created_at: T0 + 5000 });
		const methods = await callJson("/v2/account/sign-in", { env: penv(), now: T0, token: alice.token });
		expect(methods.body.identities).toEqual([{ provider: "github", created_at: T0 + 5000 }]);
		expect(methods.body.has_password).toBe(true);
	});

	it("the link flow is the session's own device's", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const other = newDevice();
		const r = await as(
			"/v2/account/sign-in/identities/github/start",
			{ intent: "link", proof: passwordProof(alice), device: wireDevice(other) },
			{ token: alice.token },
		);
		expect(r.body.error).toBe("bad_request");
	});

	it("a second account linking the same identity gets 409 identity_taken", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const bob = await signUp("bob", { env: penv(), now: T0 });
		expect((await linkFinish(alice, "apple", await linkStart(alice, "apple", passwordProof(alice)), 9)).body.kind).toBe("linked");
		const taken = await linkFinish(bob, "apple", await linkStart(bob, "apple", passwordProof(bob)), 9);
		expect(taken).toMatchObject({ status: 409, body: { error: "identity_taken", provider: "apple" } });
		expect(JSON.stringify(taken.body)).not.toContain("alice");
		// And a social sign-up with it signs in to alice instead of making an account.
		const signedIn = await apple(newDevice(), "009.apple.subject");
		expect(signedIn.body).toMatchObject({ kind: "session", account_id: alice.creds.accountId });
	});

	it("linking races: parallel links of one identity by several accounts give one link and the rest 409", async () => {
		await fc.assert(
			fc.asyncProperty(fc.integer({ min: 2, max: 4 }), fc.integer({ min: 1, max: 9_999 }), async (accounts, n) => {
				await resetServer({ signup: "open", signups_per_day: "1000" });
				const people = await Promise.all(Array.from({ length: accounts }, (_, i) => signUp(`racer${i}`, { env: penv(), now: T0 })));
				const flows = await Promise.all(people.map((p) => linkStart(p, "github", passwordProof(p))));
				providers.github.id = 50_000 + n;
				for (const _ of people) providers.github.answers.push({ access_token: PII.accessToken });
				const answers = await Promise.all(
					people.map((p, i) => as("/v2/auth/github/poll", { flow_id: flows[i].body.flow_id, device: wireDevice(p.device) }, { now: T0 + 5000 })),
				);
				expect(answers.filter((a) => a.status === 200 && a.body.kind === "linked")).toHaveLength(1);
				expect(answers.filter((a) => a.status === 409 && a.body.error === "identity_taken")).toHaveLength(accounts - 1);
				const rows = await env.DB.prepare("SELECT count(*) AS n FROM account_identities").first();
				expect(rows).toEqual({ n: 1 });
			}),
			runs(8),
		);
	});

	it("a link and a social sign-up racing for one identity: exactly one wins", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const d = newDevice();
		const ticket = await github(d, 31337);
		const started = await linkStart(alice, "github", passwordProof(alice));
		providers.github.id = 31337;
		providers.github.answers.push({ access_token: PII.accessToken });
		const [linked, made] = await Promise.all([
			as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device: wireDevice(alice.device) }, { now: T0 + 5000 }),
			as("/v2/auth/social/signup", socialSignupBody(ticket.body.ticket, "newcomer", d)),
		]);
		const winners = [linked.status === 200 && linked.body.kind === "linked", made.status === 201].filter(Boolean);
		expect(winners).toHaveLength(1);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities").first()).toEqual({ n: 1 });
	});

	it("an account links one identity per provider", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		expect((await linkFinish(alice, "github", await linkStart(alice, "github", passwordProof(alice)), 1)).body.kind).toBe("linked");
		const second = await linkFinish(alice, "github", await linkStart(alice, "github", passwordProof(alice)), 2);
		expect(second.body.error).toBe("identity_taken");
	});

	it("a provider not configured is provider_disabled, an unknown one 404", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const bare = await callJson("/v2/account/sign-in/identities/google/start", {
			env: testEnv(),
			now: T0,
			token: alice.token,
			json: { intent: "link", proof: passwordProof(alice), device: wireDevice(alice.device) },
		});
		expect(bare).toMatchObject({ status: 403, body: { error: "provider_disabled" } });
		expect((await callJson("/v2/auth/google/start", { env: testEnv(), now: T0, json: { device: wireDevice(alice.device) } })).body.error).toBe(
			"provider_disabled",
		);
		expect((await linkStart(alice, "myspace" as ProviderName, passwordProof(alice))).status).toBe(404);
	});
});

describe("unlinking", () => {
	it("with a password proof; then 404", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		await linkFinish(alice, "github", await linkStart(alice, "github", passwordProof(alice)), 3);
		const unlink = () => as("/v2/account/sign-in/identities/github/unlink", { proof: passwordProof(alice) }, { token: alice.token });
		expect((await unlink()).status).toBe(204);
		expect((await unlink()).status).toBe(404);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities").first()).toEqual({ n: 0 });
		// The account itself is untouched: its password still signs in.
		const account = await env.DB.prepare("SELECT has_password FROM accounts WHERE id = ?1").bind(alice.creds.accountId).first();
		expect(account).toEqual({ has_password: 1 });
	});

	it("needs a proof", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		await linkFinish(alice, "github", await linkStart(alice, "github", passwordProof(alice)), 3);
		expect((await as("/v2/account/sign-in/identities/github/unlink", {}, { token: alice.token })).body.error).toBe("reauth_required");
	});

	it("the last method needs the recovery key; a provider proof is single use and only for its provider", async () => {
		const made = await socialAccount("github", 11, "solo");
		const token = made.session.token;
		const proofFlow = async () => {
			const started = await as(
				"/v2/account/sign-in/identities/github/start",
				{ intent: "proof", device: wireDevice(made.device) },
				{ token },
			);
			expect(started.status).toBe(200);
			providers.github.id = 1011;
			providers.github.answers.push({ access_token: PII.accessToken });
			const done = await as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device: wireDevice(made.device) }, { now: T0 + 5000 });
			expect(done.body).toEqual({ kind: "proof", provider: "github" });
			return started.body.flow_id as string;
		};
		const flow = await proofFlow();
		const last = await as("/v2/account/sign-in/identities/github/unlink", { proof: { kind: "provider", flow } }, { token });
		expect(last).toMatchObject({ status: 409, body: { error: "last_method" } });
		// Used once already: the same proof again is refused.
		const reused = await as(
			"/v2/account/sign-in/identities/github/unlink",
			{ proof: { kind: "provider", flow }, recovery_auth_key: made.body.recovery_auth_key },
			{ token },
		);
		expect(reused.body.error).toBe("reauth_required");
		// A provider proof can't act on another provider.
		const other = await as("/v2/account/sign-in/identities/apple/unlink", { proof: { kind: "provider", flow: await proofFlow() } }, { token });
		expect(other.body.error).toBe("reauth_required");
		// A wrong recovery key is refused; the right one removes the last method.
		const wrong = await as(
			"/v2/account/sign-in/identities/github/unlink",
			{ proof: { kind: "provider", flow: await proofFlow() }, recovery_auth_key: newCreds("x").recoveryAuthKey },
			{ token },
		);
		expect(wrong.body.error).toBe("invalid_credentials");
		const done = await as(
			"/v2/account/sign-in/identities/github/unlink",
			{ proof: { kind: "provider", flow: await proofFlow() }, recovery_auth_key: made.body.recovery_auth_key },
			{ token },
		);
		expect(done.status).toBe(204);
	});

	it("a proof flow for an identity that isn't the account's proves nothing", async () => {
		const made = await socialAccount("github", 12, "solo");
		const started = await as(
			"/v2/account/sign-in/identities/github/start",
			{ intent: "proof", device: wireDevice(made.device) },
			{ token: made.session.token },
		);
		providers.github.id = 99_999;
		providers.github.answers.push({ access_token: PII.accessToken });
		const done = await as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device: wireDevice(made.device) }, { now: T0 + 5000 });
		expect(done.body.error).toBe("invalid_credentials");
	});

	it("parallel unlinks of an account's last two identities leave one", async () => {
		const made = await socialAccount("github", 13, "pair");
		const token = made.session.token;
		const account = { creds: { ...newCreds("pair"), accountId: made.body.account_id }, device: made.device, token, epoch: "" };
		const linkApple = await as(
			"/v2/account/sign-in/identities/apple/start",
			{ intent: "proof", device: wireDevice(made.device) },
			{ token },
		);
		expect(linkApple.status).toBe(200);
		// Link apple directly (the link proof needs a password this account lacks; step 6 sets one).
		await env.DB.prepare("INSERT INTO account_identities VALUES (?1, 'apple', '0013.apple.subject', ?2)").bind(account.creds.accountId, T0).run();
		const proofFor = async (provider: "github" | "apple") => {
			const started = await as(`/v2/account/sign-in/identities/${provider}/start`, { intent: "proof", device: wireDevice(made.device) }, { token });
			if (provider === "github") {
				providers.github.id = 1013;
				providers.github.answers.push({ access_token: PII.accessToken });
				await as("/v2/auth/github/poll", { flow_id: started.body.flow_id, device: wireDevice(made.device) }, { now: T0 + 5000 });
			} else {
				await appleCallback(started.body.auth_url, "0013.apple.subject");
				await as("/v2/auth/apple/poll", { flow_id: started.body.flow_id, device: wireDevice(made.device) });
			}
			return started.body.flow_id as string;
		};
		const [githubFlow, appleFlow] = [await proofFor("github"), await proofFor("apple")];
		const answers = await Promise.all([
			as("/v2/account/sign-in/identities/github/unlink", { proof: { kind: "provider", flow: githubFlow } }, { token }),
			as("/v2/account/sign-in/identities/apple/unlink", { proof: { kind: "provider", flow: appleFlow } }, { token }),
		]);
		expect(answers.map((a) => a.status).sort()).toEqual([204, 409]);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_identities").first()).toEqual({ n: 1 });
	});
});

describe("browser requests", () => {
	it("every provider route but Apple's callback refuses an Origin", async () => {
		const alice = await signUp("alice", { env: penv(), now: T0 });
		const paths = [
			"/v2/auth/apple/start",
			"/v2/auth/apple/poll",
			"/v2/auth/google/start",
			"/v2/auth/google/finish",
			"/v2/auth/github/start",
			"/v2/auth/github/poll",
			"/v2/auth/social/signup",
			"/v2/account/sign-in/identities/github/start",
			"/v2/account/sign-in/identities/github/unlink",
		];
		for (const origin of ["https://appleid.apple.com", "https://evil.example", "null"]) {
			for (const path of paths) {
				const r = await callJson(path, { env: penv(), now: T0, token: alice.token, json: {}, headers: { origin } });
				expect(r, `${path} ${origin}`).toMatchObject({ status: 403, body: { error: "forbidden_origin" } });
			}
			const methods = await callJson("/v2/account/sign-in", { env: penv(), now: T0, token: alice.token, headers: { origin } });
			expect(methods.status).toBe(403);
		}
	});
});

describe("nothing private is stored or logged", () => {
	it("D1 holds subjects only, and the console is never given a token, code, email or name", async () => {
		for (const provider of PROVIDERS) await socialAccount(provider, 21, `quiet-${provider}`);
		const alice = await signUp("alice", { env: penv(), now: T0 });
		await linkFinish(alice, "github", await linkStart(alice, "github", passwordProof(alice)), 22);
		await github(newDevice(), 1022, T0 + 60_000);
		// Failures too: a refused token, a provider outage, a forged callback.
		providers.google.tokenStatus = 500;
		await google(newDevice(), "555");
		await postCallback({ state: `${"A".repeat(43)}.${"B".repeat(43)}`, code: PII.appleCode });

		const stored = await dump();
		for (const secret of Object.values(PII)) expect(stored).not.toContain(secret);
		expect(stored).not.toContain("apple-access");
		expect(stored).not.toContain("ya29.");
		expect(stored).not.toContain("eyJ"); // no JWT, ID token or client secret
		const logged = JSON.stringify(consoleCalls);
		for (const secret of [...Object.values(PII), "001021", "1021", "apple.subject", "eyJ"]) expect(logged).not.toContain(secret);
		expect(consoleCalls).toEqual([]);
	});
});

describe("suggested usernames", () => {
	it("are always valid, never reserved, and stable for a ticket", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 32, maxLength: 32 }), fc.nat(7), async (bytes, n) => {
				const ticket = b64(bytes);
				const name = await candidateUsername(ticket, n);
				expect(normalizeUsername(name)).toBe(name);
				expect(isReserved(name)).toBe(false);
				expect(name).toMatch(/^[a-z]+-[a-z]+-[1-9][0-9]{3}$/);
				expect(await candidateUsername(ticket, n)).toBe(name);
			}),
			runs(200),
		);
	});
});
