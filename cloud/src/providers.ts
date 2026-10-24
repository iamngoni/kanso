// Sign in with Apple, Google and GitHub; new-account tickets; linking and
// unlinking (design: Kanso accounts spec, revision 3 › Sign in with Apple,
// Google and GitHub, and › Sign-in routes).
//
// Every flow is an `auth_flows` row bound to one device: its ID, and SHA-256 of
// its proof. The Worker, never the app, exchanges the code or polls the device
// endpoint and verifies the answer; it keeps the provider's stable subject on
// the flow and nothing else. No email, name, profile, token, code or subject is
// written to a log or an error message: this file never calls `console`.
//
// A flow's completion depends on its intent:
//   signin  a linked identity gets a session; an unlinked one a new-account
//           ticket (the flow ID), which `POST /v2/auth/social/signup` consumes
//   link    the identity is added to the flow's account, or 409 identity_taken
//   proof   the identity must already be the account's; the flow is then a
//           provider proof, consumed by the sensitive change that uses it
// Finishes and polls are idempotent: a retry after a lost answer gets the same
// result (a fresh token for the same device row, the same ticket), never a
// second account, identity or device.

import {
	type Session,
	authenticate,
	newSession,
	readObject,
	respond,
	sessionOutcome,
	sessionStatements,
	username,
} from "./auth";
import { type PepperKeys, proofMatches, randomBytes, sha256, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { accountDisabled, badRequest, constraint, error, invalidCredentials, json, notFound, readBody, refusal } from "./http";
import { UNKNOWN_LOGIN_KEY, accountKey, clientIp, count, ipGroup, ipKey, sessionFilter } from "./limits";
import {
	APPLE_ID_TOKEN_ALGORITHMS,
	APPLE_ISSUER,
	APPLE_JWKS_URL,
	GITHUB_MIN_INTERVAL,
	GOOGLE_ID_TOKEN_ALGORITHMS,
	GOOGLE_ISSUERS,
	GOOGLE_JWKS_URL,
	type TokenFailure,
	exchangeAppleCode,
	exchangeGoogleCode,
	githubDeviceCode,
	githubDeviceToken,
	githubSubject,
	pkceHash,
	pkceMatches,
	verifyIdToken,
} from "./oauth";
import {
	KDF_DEFAULT,
	MAX_AUTH_BODY,
	type NewDevice,
	base64url,
	checkDevice,
	isAccountId,
	isKeyId,
	isObject,
	isWrappedKey,
	proof32,
} from "./protocol";
import { accountPrefix } from "./purge";
import {
	type AuthFlowRow,
	type Provider,
	SIGN_IN_ROUTES,
	consumeFlow,
	flowById,
	flowExpired,
	flowForDevice,
	isFlowId,
	isProvider,
	newFlow,
	openDeviceCode,
	providerConfigured,
} from "./sign_in";
import { isReserved, normalizeUsername } from "./usernames";

type PepperHandler = (request: Request, env: Env, now: number, keys: PepperKeys) => Promise<Response>;

export const APPLE_AUTHORIZE_URL = "https://appleid.apple.com/auth/authorize";
export const GOOGLE_AUTHORIZE_URL = "https://accounts.google.com/o/oauth2/v2/auth";
/** Where the Apple callback page sends the person back to. */
export const APP_CALLBACK = "kanso://auth/apple";
/** A provider flow lives ten minutes. */
export const PROVIDER_FLOW_MS = 10 * 60_000;
/** The origin of Apple's `form_post`: the one Origin a public route accepts besides none. */
export const APPLE_FORM_ORIGIN = "https://appleid.apple.com";

const LOOPBACK = /^http:\/\/127\.0\.0\.1:([0-9]{4,5})$/;
const PROVIDER_NAME = /^[a-z][a-z0-9_]{1,31}$/;
/** RFC 7636: 43–128 unreserved characters. */
const PKCE_VERIFIER = /^[A-Za-z0-9._~-]{43,128}$/;
/** An authorization code as Apple and Google issue them: printable ASCII, bounded. */
const AUTH_CODE = /^[\x21-\x7e]{1,2048}$/;

// ------------------------------------------------------------------ helpers

/** The hash a flow stores for a device: SHA-256 of its proof as sent (base64url), as `flowForDevice` compares it. */
const flowProofHash = (device: NewDevice): Promise<string> => sha256Hex(base64url(device.proof));
const flowDevice = (device: NewDevice) => ({ id: device.id, proof: base64url(device.proof) });

const providerUnavailable = (provider: string): Response =>
	error(503, "provider_unavailable", "The provider could not be reached.", { provider });
const providerDenied = (): Response => error(403, "provider_denied", "The sign-in was declined at the provider.");
/** 410: the provider itself expired the flow (GitHub's `expired_token`), not our ten-minute clock. */
const flowGone = (): Response => error(410, "flow_expired", "This sign-in flow has expired.");
const providerDisabled = (): Response => error(403, "provider_disabled", "That provider isn't configured on this server.");
const identityTaken = (provider: string): Response =>
	error(409, "identity_taken", "That identity belongs to another account.", { provider });
const reauthRequired = (): Response => error(401, "reauth_required", "This change needs a fresh proof.", { reason: "reauth" });

/** A provider this Worker serves and has configured, or the response that ends the request. */
function servedProvider(env: Env, name: string): Provider | Response {
	if (!isProvider(name) || !SIGN_IN_ROUTES[name]) return notFound();
	if (!providerConfigured(env, name)) return providerDisabled();
	return name;
}

function tokenFailure(failure: TokenFailure | "provider_denied", provider: string): Response {
	switch (failure) {
		case "invalid_credentials":
			return invalidCredentials();
		case "nonce_reused":
			return error(400, "nonce_reused", "This sign-in was already used.");
		case "provider_denied":
			return providerDenied();
		case "provider_unavailable":
			return providerUnavailable(provider);
	}
}

/** Stores the verified subject on the flow, once. False when the flow already holds another one. */
async function storeSubject(env: Env, flowId: string, subject: string): Promise<boolean> {
	const row = await env.DB.prepare(
		"UPDATE auth_flows SET subject = ?2 WHERE id = ?1 AND (subject IS NULL OR subject = ?2) RETURNING subject",
	)
		.bind(flowId, subject)
		.first();
	return row !== null;
}

// ---------------------------------------------------------- new-account names

const ADJECTIVES = [
	"amber", "bold", "brave", "bright", "calm", "clever", "cosy", "crisp", "daring", "eager", "fair", "gentle", "glad",
	"golden", "happy", "honest", "jolly", "keen", "kind", "lively", "lucky", "mellow", "merry", "misty", "noble",
	"patient", "proud", "quick", "quiet", "rapid", "silver", "steady", "sunny", "swift", "tidy", "warm", "wise", "witty",
];
const ANIMALS = [
	"badger", "beaver", "bison", "crane", "deer", "dolphin", "eagle", "falcon", "ferret", "finch", "fox", "gecko",
	"heron", "ibis", "koala", "lark", "lemur", "lynx", "marten", "moose", "newt", "otter", "owl", "panda", "puffin",
	"quail", "raven", "robin", "seal", "sparrow", "stoat", "swan", "tapir", "tern", "tiger", "turtle", "walrus", "wren",
];

/**
 * The `n`th candidate name for a ticket: a word, an animal and four digits
 * (`quiet-otter-4821`), derived from the ticket so a retried finish suggests the
 * same name. Always valid and never reserved.
 */
export async function candidateUsername(ticket: string, n: number): Promise<string> {
	const digest = await sha256(`kanso/v1/suggested-username\u0000${ticket}\u0000${n}`);
	const word = ADJECTIVES[((digest[0] << 8) | digest[1]) % ADJECTIVES.length];
	const animal = ANIMALS[((digest[2] << 8) | digest[3]) % ANIMALS.length];
	const digits = String((((digest[4] << 16) | (digest[5] << 8) | digest[6]) % 9000) + 1000);
	return `${word}-${animal}-${digits}`;
}

/** A suggestion for the new-account form that no account has yet (the user may type another). */
export async function suggestUsername(env: Env, ticket: string): Promise<string> {
	let first = "";
	for (let n = 0; n < 8; n++) {
		const name = await candidateUsername(ticket, n);
		if (n === 0) first = name;
		if (normalizeUsername(name) !== name || isReserved(name)) continue;
		const taken = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE username = ?1").bind(name).first();
		if (!taken) return name;
	}
	// Eight collisions in a row: the form still checks the name at sign-up.
	return first;
}

// ------------------------------------------------------------------- starts

interface StartContext {
	provider: Provider;
	intent: "signin" | "link" | "proof";
	accountId: string | null;
	device: NewDevice;
	body: Record<string, unknown>;
}

/** Creates the flow and answers the provider's start shape. */
async function startFlow(env: Env, keys: PepperKeys, now: number, c: StartContext): Promise<Response> {
	const flowId = base64url(randomBytes(32));
	const expiresAt = now + PROVIDER_FLOW_MS;
	const base = {
		id: flowId,
		kind: c.provider,
		provider: c.provider,
		intent: c.intent,
		account_id: c.accountId,
		device_id: c.device.id,
		device_proof_hash: await flowProofHash(c.device),
		challenge: null,
		rp_id: null,
		subject: null,
		device_code: null,
		poll_interval_seconds: null,
		next_poll_at: null,
		pkce_hash: null,
		nonce_hash: null,
		state_hash: null,
		redirect_uri: null,
		expires_at: expiresAt,
	};

	switch (c.provider) {
		case "apple": {
			// The flow ID leads the state so the callback finds its row by key; the
			// 32 random bytes after it are the secret, stored only as SHA-256.
			const state = `${flowId}.${base64url(randomBytes(32))}`;
			const nonce = base64url(randomBytes(32));
			const redirectUri = env.APPLE_REDIRECT_URI as string;
			await newFlow(
				env,
				keys,
				{ ...base, nonce_hash: await sha256Hex(nonce), state_hash: await sha256Hex(state), redirect_uri: redirectUri },
				now,
			);
			// No scope: no name and no email, so Apple's private relay never enters the picture.
			const url = new URL(APPLE_AUTHORIZE_URL);
			url.search = new URLSearchParams({
				client_id: env.APPLE_CLIENT_ID as string,
				redirect_uri: redirectUri,
				response_type: "code",
				response_mode: "form_post",
				state,
				nonce,
			}).toString();
			return json({ flow_id: flowId, auth_url: url.toString(), expires_at: expiresAt });
		}
		case "google": {
			const redirectUri = c.body.redirect_uri;
			const match = typeof redirectUri === "string" ? LOOPBACK.exec(redirectUri) : null;
			const port = match ? Number(match[1]) : 0;
			if (!match || port < 1024 || port > 65535 || String(port) !== match[1]) {
				return badRequest("redirect_uri must be http://127.0.0.1:<port>, port 1024–65535, nothing after it.");
			}
			const challenge = c.body.code_challenge;
			const pkce = typeof challenge === "string" ? pkceHash(challenge) : null;
			if (!pkce) return badRequest("code_challenge must be base64url(SHA-256(verifier)).");
			const state = base64url(randomBytes(32));
			const nonce = base64url(randomBytes(32));
			await newFlow(
				env,
				keys,
				{
					...base,
					nonce_hash: await sha256Hex(nonce),
					state_hash: await sha256Hex(state),
					pkce_hash: pkce,
					redirect_uri: redirectUri as string,
				},
				now,
			);
			const url = new URL(GOOGLE_AUTHORIZE_URL);
			url.search = new URLSearchParams({
				client_id: env.GOOGLE_CLIENT_ID as string,
				redirect_uri: redirectUri as string,
				response_type: "code",
				scope: "openid",
				state,
				nonce,
				code_challenge: challenge as string,
				code_challenge_method: "S256",
				access_type: "online",
			}).toString();
			return json({ flow_id: flowId, auth_url: url.toString(), nonce, state, expires_at: expiresAt });
		}
		case "github": {
			const started = await githubDeviceCode(env);
			if (!started.ok) return providerUnavailable("github");
			const { device_code, user_code, verification_uri, expires_in, interval } = started.value;
			const expires = Math.min(expiresAt, now + expires_in * 1000);
			if (expires <= now) return providerUnavailable("github");
			await newFlow(
				env,
				keys,
				{ ...base, device_code, poll_interval_seconds: interval, next_poll_at: now + interval * 1000, expires_at: expires },
				now,
			);
			return json({
				flow_id: flowId,
				user_code,
				verification_uri,
				interval,
				expires_in: Math.floor((expires - now) / 1000),
			});
		}
	}
}

/** `POST /v2/auth/<provider>/start`: a sign-in or sign-up flow for this device. */
export function start(name: Provider): PepperHandler {
	return async (request, env, now, keys) => {
		const provider = servedProvider(env, name);
		if (provider instanceof Response) return provider;
		const body = await readObject(request);
		if (!body.ok) return body.response;
		const device = checkDevice(body.value.device, false);
		if (!device.ok) return respond(device);
		return startFlow(env, keys, now, { provider, intent: "signin", accountId: null, device: device.value, body: body.value });
	};
}

// --------------------------------------------------------------- completion

/** The session answer every sign-in method shares. */
function sessionAnswer(
	account: { id: string; username: string; key_id: string; epoch: string; has_password: number },
	token: string,
	evicted: { id: string; sealed_name: string | null } | null,
	status = 200,
): Response {
	return json(
		{
			kind: "session",
			account_id: account.id,
			username: account.username,
			token,
			key_id: account.key_id,
			epoch: account.epoch,
			has_password: account.has_password === 1,
			passkey_wrapped_key: null,
			evicted_device: evicted,
		},
		status,
	);
}

interface IdentityAccount {
	id: string;
	username: string;
	key_id: string;
	epoch: string;
	has_password: number;
	disabled_at: number | null;
	device_proof_hash: string | null;
	auth_hash: string;
}

/** What a verified subject does, by the flow's intent. */
async function complete(request: Request, env: Env, keys: PepperKeys, now: number, flow: AuthFlowRow, device: NewDevice): Promise<Response> {
	const provider = flow.provider as string;
	const subject = flow.subject as string;
	switch (flow.intent) {
		case "signin":
			return signIn(request, env, keys, now, flow, device, provider, subject);
		case "link":
			return link(env, now, flow, provider, subject);
		case "proof":
			return proof(env, flow, provider, subject);
		default:
			return flowExpired();
	}
}

type CompletionKind = "signin" | "link" | "social_signup";

/** A globally unique marker makes each batch's writes depend on its own flow claim. */
function completionAttemptId(): string {
	return base64url(randomBytes(32));
}

function claimCompletion(env: Env, flowId: string, attemptId: string, kind: CompletionKind, accountId: string, now: number) {
	return env.DB.prepare(
		`UPDATE auth_flows SET consumed_at = ?2, completion_attempt_id = ?3, completion_kind = ?4,
  completion_account_id = ?5, completion_succeeded = 0
  WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2 RETURNING id`,
	).bind(flowId, now, attemptId, kind, accountId);
}

function markCompletionSucceeded(
	env: Env,
	flowId: string,
	attemptId: string,
	kind: CompletionKind,
	accountId: string,
	condition: string,
	conditionParams: unknown[],
) {
	return env.DB.prepare(
		`UPDATE auth_flows SET completion_succeeded = 1
  WHERE id = ?1 AND completion_attempt_id = ?2 AND completion_kind = ?3 AND completion_account_id = ?4
    AND completion_succeeded = 0 AND ${condition} RETURNING id`,
	).bind(flowId, attemptId, kind, accountId, ...conditionParams);
}

async function signIn(
	request: Request,
	env: Env,
	keys: PepperKeys,
	now: number,
	flow: AuthFlowRow,
	device: NewDevice,
	provider: string,
	subject: string,
): Promise<Response> {
	let limited = await count(env.DB, "login-ip", await ipKey(keys, "login-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const account = await env.DB.prepare(
		`SELECT a.id, a.username, a.key_id, a.epoch, a.has_password, a.disabled_at, a.auth_hash,
  (SELECT s.proof_hash FROM sessions s WHERE s.account_id = a.id AND s.device_id = ?3 AND s.revoked_at IS NULL) AS device_proof_hash
  FROM account_identities i JOIN accounts a ON a.id = i.account_id WHERE i.provider = ?1 AND i.subject = ?2`,
	)
		.bind(provider, subject, device.id)
		.first<IdentityAccount>();
	if (!account) {
		if (flow.consumed_at !== null) return flowExpired();
		// Never a lookup by email: an unlinked identity is offered a new account. Every
		// unknown identity shares the same login-user row, like a password sign-in for a
		// name with no account, so a pile of tickets cannot bypass the sign-in limit.
		limited = await count(env.DB, "login-user", UNKNOWN_LOGIN_KEY, now);
		if (limited) return limited;
		return json({ kind: "new_account", ticket: flow.id, suggested_username: await suggestUsername(env, flow.id) });
	}
	const known = account.device_proof_hash !== null && (await proofMatches(device.proof, account.device_proof_hash));
	if (!known) {
		limited = await count(env.DB, "login-user", accountKey("login-user", account.id), now);
		if (limited) return limited;
	}
	if (account.disabled_at !== null) return accountDisabled();
	if (flow.consumed_at !== null) {
		// Only a successful sign-in completion for this same account can be replayed.
		// flowForDevice has already checked the unexpired flow and this device proof.
		if (
			flow.completion_succeeded !== 1 ||
			flow.completion_kind !== "signin" ||
			flow.completion_account_id !== account.id ||
			flow.completion_attempt_id === null
		) {
			return flowExpired();
		}
		const replayNow = Math.max(now, Date.now());
		if (flow.expires_at <= replayNow) return flowExpired();
		const session = await newSession(device);
		const results = await env.DB.batch(
			sessionStatements(
				env.DB,
				{ accountId: account.id, deviceId: device.id, ...session, sealedName: null, now: replayNow },
				[["auth_hash", account.auth_hash]],
				{ flowId: flow.id, attemptId: flow.completion_attempt_id },
			),
		);
		const outcome = sessionOutcome(results, 0);
		if (!outcome.ok) return invalidCredentials();
		return sessionAnswer(account, session.token, outcome.evicted);
	}

	const completionNow = Math.max(now, Date.now());
	const attemptId = completionAttemptId();
	const session = await newSession(device);
	const results = await env.DB.batch([
		claimCompletion(env, flow.id, attemptId, "signin", account.id, completionNow),
		...sessionStatements(
			env.DB,
			{ accountId: account.id, deviceId: device.id, ...session, sealedName: null, now: completionNow },
			[["auth_hash", account.auth_hash]],
			{ flowId: flow.id, attemptId },
		),
		markCompletionSucceeded(
			env,
			flow.id,
			attemptId,
			"signin",
			account.id,
			"EXISTS (SELECT 1 FROM sessions WHERE account_id = ?4 AND device_id = ?5 AND token_hash = ?6 AND revoked_at IS NULL)",
			[device.id, session.tokenHash],
		),
	]);
	if (results[0].results.length !== 1) return flowExpired();
	const outcome = sessionOutcome(results, 1);
	if (!outcome.ok || results[4].results.length !== 1) return invalidCredentials();
	return sessionAnswer(account, session.token, outcome.evicted);
}

async function link(env: Env, now: number, flow: AuthFlowRow, provider: string, subject: string): Promise<Response> {
	const accountId = flow.account_id as string;
	if (flow.consumed_at !== null) {
		if (
			flow.completion_succeeded !== 1 ||
			flow.completion_kind !== "link" ||
			flow.completion_account_id !== accountId ||
			flow.completion_attempt_id === null
		) {
			return flowExpired();
		}
		const replayNow = Math.max(now, Date.now());
		if (flow.expires_at <= replayNow) return flowExpired();
		const owner = await env.DB.prepare(
			"SELECT account_id, created_at FROM account_identities WHERE provider = ?1 AND subject = ?2",
		)
			.bind(provider, subject)
			.first<{ account_id: string; created_at: number }>();
		return owner?.account_id === accountId
			? json({ kind: "linked", provider, created_at: owner.created_at })
			: flowExpired();
	}

	const completionNow = Math.max(now, Date.now());
	const attemptId = completionAttemptId();
	// One identity per provider per account, and one account per identity: the
	// primary key settles a race between two accounts, the NOT EXISTS a second
	// identity of the same provider on this one.
	const results = await env.DB.batch([
		claimCompletion(env, flow.id, attemptId, "link", accountId, completionNow),
		env.DB.prepare(
			`INSERT INTO account_identities (account_id, provider, subject, created_at)
  SELECT ?1, ?2, ?3, ?4 WHERE EXISTS (SELECT 1 FROM auth_flows WHERE id = ?5 AND completion_attempt_id = ?6)
    AND EXISTS (SELECT 1 FROM accounts WHERE id = ?1)
    AND NOT EXISTS (SELECT 1 FROM account_identities WHERE account_id = ?1 AND provider = ?2)
	  ON CONFLICT (provider, subject) DO NOTHING RETURNING created_at`,
		).bind(accountId, provider, subject, completionNow, flow.id, attemptId),
		markCompletionSucceeded(
			env,
			flow.id,
			attemptId,
			"link",
			accountId,
			"EXISTS (SELECT 1 FROM account_identities WHERE account_id = ?4 AND provider = ?5 AND subject = ?6)",
			[provider, subject],
		),
	]);
	if (results[0].results.length !== 1) return flowExpired();
	if (results[2].results.length === 1) {
		const linked = await env.DB.prepare(
			"SELECT created_at FROM account_identities WHERE account_id = ?1 AND provider = ?2 AND subject = ?3",
		)
			.bind(accountId, provider, subject)
			.first<{ created_at: number }>();
		if (linked) return json({ kind: "linked", provider, created_at: linked.created_at });
	}
	const owner = await env.DB.prepare("SELECT account_id, created_at FROM account_identities WHERE provider = ?1 AND subject = ?2")
		.bind(provider, subject)
		.first<{ account_id: string; created_at: number }>();
	// Already this account's: the retry of a link whose answer was lost.
	if (owner?.account_id === accountId) return json({ kind: "linked", provider, created_at: owner.created_at });
	if (!owner) {
		const exists = await env.DB.prepare("SELECT 1 AS found FROM accounts WHERE id = ?1").bind(accountId).first();
		if (!exists) return flowExpired();
	}
	// Another account owns it, or this account already has another identity of
	// this provider. Never says which account.
	return identityTaken(provider);
}

async function proof(env: Env, flow: AuthFlowRow, provider: string, subject: string): Promise<Response> {
	const linked = await env.DB.prepare("SELECT 1 AS linked FROM account_identities WHERE account_id = ?1 AND provider = ?2 AND subject = ?3")
		.bind(flow.account_id, provider, subject)
		.first();
	// The flow stays unconsumed: the sensitive change that carries it consumes it.
	return linked ? json({ kind: "proof", provider }) : invalidCredentials();
}

/** The body's `flow_id` and device, and the device's live flow of `kind`, or the response that ends the request. */
async function deviceFlow(
	request: Request,
	env: Env,
	now: number,
	kind: string,
	count_?: (flowId: string) => Promise<Response | null>,
): Promise<{ ok: true; flow: AuthFlowRow; device: NewDevice; body: Record<string, unknown> } | { ok: false; response: Response }> {
	const body = await readObject(request);
	if (!body.ok) return body;
	const flowId = body.value.flow_id;
	if (typeof flowId !== "string" || !isFlowId(flowId)) {
		return { ok: false, response: badRequest("flow_id must be 32 bytes of canonical base64url.") };
	}
	if (count_) {
		const limited = await count_(flowId);
		if (limited) return { ok: false, response: limited };
	}
	const device = checkDevice(body.value.device, false);
	if (!device.ok) return { ok: false, response: respond(device) };
	const flow = await flowForDevice(env, flowId, kind, flowDevice(device.value), now);
	if (flow instanceof Response) return { ok: false, response: flow };
	return { ok: true, flow, device: device.value, body: body.value };
}

const pollCounter = (env: Env, keys: PepperKeys, now: number) => async (flowId: string) =>
	count(env.DB, "provider-poll", await ipKey(keys, "provider-poll", flowId), now);

/** `POST /v2/auth/apple/poll`: pending until Apple's callback has stored the subject. */
export const applePoll: PepperHandler = async (request, env, now, keys) => {
	const found = await deviceFlow(request, env, now, "apple", pollCounter(env, keys, now));
	if (!found.ok) return found.response;
	const { flow, device } = found;
	if (flow.subject === null) {
		// The callback consumes a flow it refused (a cancel, a bad code, a failed
		// verification), so the poll can stop instead of waiting out the flow.
		return flow.consumed_at === null ? json({ status: "pending" }) : providerDenied();
	}
	return complete(request, env, keys, now, flow, device);
};

/** `POST /v2/auth/google/finish`: the loopback's code, exchanged with the app's PKCE verifier. */
export const googleFinish: PepperHandler = async (request, env, now, keys) => {
	const found = await deviceFlow(request, env, now, "google");
	if (!found.ok) return found.response;
	const { flow, device, body } = found;
	const { code, code_verifier } = body;
	if (typeof code !== "string" || !AUTH_CODE.test(code)) return badRequest("code must be the authorization code.");
	if (typeof code_verifier !== "string" || !PKCE_VERIFIER.test(code_verifier)) {
		return badRequest("code_verifier must be 43–128 unreserved characters.");
	}
	if (flow.pkce_hash === null || !(await pkceMatches(code_verifier, flow.pkce_hash))) return invalidCredentials();
	if (flow.subject !== null) return complete(request, env, keys, now, flow, device);
	if (flow.consumed_at !== null) return flowExpired();
	if (!providerConfigured(env, "google")) return providerDisabled();

	const exchanged = await exchangeGoogleCode(env, code, code_verifier, flow.redirect_uri ?? undefined);
	if (!exchanged.ok) return tokenFailure(exchanged.error, "google");
	const verified = await verifyIdToken(exchanged.id_token, {
		jwks_url: GOOGLE_JWKS_URL,
		algorithms: GOOGLE_ID_TOKEN_ALGORITHMS,
		issuers: GOOGLE_ISSUERS,
		audience: env.GOOGLE_CLIENT_ID as string,
		nonce_hash: flow.nonce_hash as string,
		now,
	});
	if (!verified.ok) return tokenFailure(verified.error, "google");
	// Only `sub` is read; `email`, `name` and `picture` are never looked at.
	const subject = verified.claims.sub as string;
	if (!(await storeSubject(env, flow.id, subject))) return tokenFailure("nonce_reused", "google");
	return complete(request, env, keys, now, { ...flow, subject }, device);
};

/** `POST /v2/auth/github/poll`: one device-flow poll, no faster than GitHub's interval. */
export const githubPoll: PepperHandler = async (request, env, now, keys) => {
	const found = await deviceFlow(request, env, now, "github", pollCounter(env, keys, now));
	if (!found.ok) return found.response;
	const { flow, device } = found;
	if (flow.subject !== null) return complete(request, env, keys, now, flow, device);
	if (flow.consumed_at !== null) return providerDenied();
	if (!providerConfigured(env, "github")) return providerDisabled();

	// Claim this poll slot: of two parallel polls one calls GitHub, the other is told to wait.
	const next = flow.next_poll_at ?? 0;
	const intervalSeconds = flow.poll_interval_seconds ?? GITHUB_MIN_INTERVAL;
	const nextPollAt = now + intervalSeconds * 1000;
	const claimed =
		now >= next &&
		(await env.DB.prepare(
			`UPDATE auth_flows SET next_poll_at = ?3, poll_interval_seconds = coalesce(poll_interval_seconds, ?6)
  WHERE id = ?1 AND next_poll_at IS ?2 AND consumed_at IS NULL AND expires_at > ?4
    AND poll_interval_seconds IS ?5 RETURNING id`,
		)
			.bind(flow.id, flow.next_poll_at, nextPollAt, now, flow.poll_interval_seconds, intervalSeconds)
			.first()) !== null;
	if (!claimed) {
		const latest = await flowById(env, flow.id);
		if (latest?.subject) return complete(request, env, keys, now, latest, device);
		return error(429, "rate_limited", "Poll no faster than the interval.", {
			limit: "provider-poll",
			retry_after: Math.max(1, Math.ceil(((latest?.next_poll_at ?? next) - now) / 1000)),
		}, { "retry-after": String(Math.max(1, Math.ceil(((latest?.next_poll_at ?? next) - now) / 1000))) });
	}

	const deviceCode = await openDeviceCode(keys, flow);
	if (deviceCode === null) return flowExpired();
	const polled = await githubDeviceToken(env, deviceCode);
	if (!polled.ok) {
		switch (polled.error) {
			case "pending":
				return json({ status: "pending" });
			case "slow_down": {
				// GitHub raised its interval by five seconds. Keep that increase for later polls;
				// another slow_down adds five more rather than resetting to a fixed ten seconds.
				const scheduleNow = Math.max(now, Date.now());
				const updated = await env.DB.prepare(
					`UPDATE auth_flows
  SET poll_interval_seconds = coalesce(poll_interval_seconds, ?3) + ?4,
      next_poll_at = max(next_poll_at, ?2 + (coalesce(poll_interval_seconds, ?3) + ?4) * 1000)
  WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2
  RETURNING id`,
				)
					.bind(flow.id, scheduleNow, GITHUB_MIN_INTERVAL, GITHUB_MIN_INTERVAL)
					.first();
				if (!updated) return flowGone();
				return json({ status: "pending", slow_down: true });
			}
			case "flow_expired":
				await consumeFlow(env, flow.id, now);
				return flowGone();
			case "provider_denied":
				await consumeFlow(env, flow.id, now);
				return providerDenied();
			default:
				return providerUnavailable("github");
		}
	}
	// The token reads the numeric ID once and is dropped here: no column, no log, no retry.
	const user = await githubSubject(polled.token);
	if (!user.ok) return providerUnavailable("github");
	if (!(await storeSubject(env, flow.id, user.subject))) return tokenFailure("nonce_reused", "github");
	return complete(request, env, keys, now, { ...flow, subject: user.subject }, device);
};

// ------------------------------------------------------------ Apple callback

const CALLBACK_STYLE =
	"body{font:16px -apple-system,BlinkMacSystemFont,sans-serif;max-width:28em;margin:20vh auto;padding:0 1em;color:#222}a{color:#0a58ca}";

/** The page Apple's form post lands on: a redirect back to the app, or a short error sentence and no flow. */
function callbackPage(status: number, flowId: string | null, sentence: string): Response {
	const target = flowId === null ? null : `${APP_CALLBACK}?flow=${flowId}`;
	const refresh = target === null ? "" : `<meta http-equiv="refresh" content="0;url=${target}">`;
	const link = target === null ? "" : `<p><a href="${target}">Return to Kanso.</a></p>`;
	const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">${refresh}<title>Kanso</title><style>${CALLBACK_STYLE}</style></head><body><p>${sentence}</p>${link}</body></html>`;
	return new Response(html, {
		status,
		headers: {
			"content-type": "text/html; charset=utf-8",
			"cache-control": "no-store",
			"content-security-policy": "default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
			"referrer-policy": "no-referrer",
			"x-content-type-options": "nosniff",
		},
	});
}

const FAILED = "Kanso couldn't finish signing in with Apple. Return to Kanso and try again.";

/** Reads the callback's fields: the form body of a POST, or the query of a GET. */
async function callbackFields(request: Request): Promise<URLSearchParams | Response> {
	if (request.method === "GET") return new URL(request.url).searchParams;
	const type = (request.headers.get("content-type") ?? "").split(";")[0].trim().toLowerCase();
	if (type !== "application/x-www-form-urlencoded") return callbackPage(415, null, FAILED);
	const bytes = await readBody(request, MAX_AUTH_BODY);
	if (!bytes) return callbackPage(413, null, FAILED);
	try {
		return new URLSearchParams(new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(bytes));
	} catch {
		return callbackPage(400, null, FAILED);
	}
}

/**
 * `GET` and `POST /v2/auth/apple/callback`: Apple's `form_post`. The one public
 * route that accepts an `Origin` (Apple's). It never answers JSON and never
 * creates a session: it stores the verified subject on the flow, and the app's
 * poll, which carries the device proof, is the only thing that signs in.
 */
export const appleCallback: PepperHandler = async (request, env, now, keys) => {
	const fields = await callbackFields(request);
	if (fields instanceof Response) return fields;
	const state = fields.get("state") ?? "";
	const dot = state.indexOf(".");
	const flowId = dot === 43 ? state.slice(0, 43) : "";
	if (!isFlowId(flowId) || state.length !== 87) return callbackPage(400, null, FAILED);
	const limited = await count(env.DB, "provider-poll", await ipKey(keys, "provider-poll", flowId), now);
	if (limited) return callbackPage(429, null, FAILED);

	const flow = await flowById(env, flowId);
	if (!flow || flow.kind !== "apple" || flow.expires_at <= now || flow.state_hash === null) return callbackPage(400, null, FAILED);
	// Constant time: both sides are SHA-256 hex digests.
	const stateHash = await sha256Hex(state);
	let difference = 0;
	for (let i = 0; i < 64; i++) difference |= stateHash.charCodeAt(i) ^ flow.state_hash.charCodeAt(i);
	if (difference !== 0) return callbackPage(400, null, FAILED);
	// State is single use: a second post (a replayed code or token) finds the subject or the refusal stored.
	if (flow.subject !== null || flow.consumed_at !== null) return callbackPage(400, null, FAILED);

	const code = fields.get("code");
	if (fields.get("error") !== null || code === null || !AUTH_CODE.test(code)) {
		await consumeFlow(env, flow.id, now);
		return callbackPage(400, null, "Signing in with Apple was cancelled. Return to Kanso to try again.");
	}
	if (!providerConfigured(env, "apple")) return callbackPage(503, null, FAILED);
	if (flow.redirect_uri === null) return callbackPage(400, null, FAILED);
	const exchanged = await exchangeAppleCode(env, code, now, flow.redirect_uri);
	if (!exchanged.ok) {
		if (exchanged.error !== "provider_unavailable") await consumeFlow(env, flow.id, now);
		return callbackPage(exchanged.error === "provider_unavailable" ? 503 : 400, null, FAILED);
	}
	const verified = await verifyIdToken(exchanged.id_token, {
		jwks_url: APPLE_JWKS_URL,
		algorithms: APPLE_ID_TOKEN_ALGORITHMS,
		issuers: [APPLE_ISSUER],
		audience: env.APPLE_CLIENT_ID as string,
		nonce_hash: flow.nonce_hash as string,
		now,
	});
	if (!verified.ok) {
		if (verified.error !== "provider_unavailable") await consumeFlow(env, flow.id, now);
		return callbackPage(verified.error === "provider_unavailable" ? 503 : 400, null, FAILED);
	}
	// `email`, `email_verified` and `is_private_email`, if Apple ever sends them, are ignored.
	const stored = await env.DB.prepare(
		"UPDATE auth_flows SET subject = ?2 WHERE id = ?1 AND subject IS NULL AND consumed_at IS NULL RETURNING id",
	)
		.bind(flow.id, verified.claims.sub as string)
		.first();
	if (!stored) return callbackPage(400, null, FAILED);
	return callbackPage(200, flow.id, "Signed in with Apple. Return to Kanso.");
};

// ------------------------------------------------------------ social sign-up

interface AccountForRetry {
	id: string;
	username: string;
	recovery_hash: string;
	recovery_wrapped_key: string;
	key_id: string;
	epoch: string;
	has_password: number;
	disabled_at: number | null;
}

/** `POST /v2/auth/social/signup`: the account a new-account ticket was issued for. */
export const socialSignup: PepperHandler = async (request, env, now, keys) => {
	const limited = await count(env.DB, "signup-ip", await ipKey(keys, "signup-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	if (typeof b.ticket !== "string" || !isFlowId(b.ticket)) return badRequest("ticket must be a flow ID.");
	const name = username(b);
	if (!name.ok) return respond(name);
	if (isReserved(name.value)) return error(400, "reserved_username", "That username is reserved.");
	if (typeof b.account_id !== "string" || !isAccountId(b.account_id)) return badRequest("account_id must be a_ and 32 hex digits.");
	const recoveryAuthKey = proof32(b.recovery_auth_key);
	if (!recoveryAuthKey) return badRequest("recovery_auth_key must be 32 bytes of canonical base64url.");
	if (typeof b.recovery_wrapped_key !== "string" || !isWrappedKey(b.recovery_wrapped_key)) {
		return badRequest("recovery_wrapped_key must be 98 base64url characters.");
	}
	if (typeof b.key_id !== "string" || !isKeyId(b.key_id)) return badRequest("key_id must be 16 hex digits.");
	const device = checkDevice(b.device, true);
	if (!device.ok) return respond(device);
	const accountId = b.account_id;
	const recoveryWrappedKey = b.recovery_wrapped_key;
	const keyId = b.key_id;

	// The ticket is a provider flow this device started, verified and not linked.
	const ticket = await flowById(env, b.ticket);
	if (!ticket || ticket.expires_at <= now || ticket.provider === null || !isProvider(ticket.kind)) return flowExpired();
	const flow = await flowForDevice(env, b.ticket, ticket.kind, flowDevice(device.value), now);
	if (flow instanceof Response) return flow;
	if (flow.intent !== "signin" || flow.subject === null) return flowExpired();
	const provider = flow.provider as string;
	const subject = flow.subject;
	const recoveryHash = await sha256Hex(recoveryAuthKey);
	const session = await newSession(device.value);

	// A retry of a sign-up whose answer was lost: the same account ID and
	// username, this identity, and the same recovery hash, wrap and key ID.
	const retried = async (): Promise<Response | null> => {
		const existing = await env.DB.prepare(
			`SELECT a.id, a.username, a.recovery_hash, a.recovery_wrapped_key, a.key_id, a.epoch, a.has_password, a.disabled_at
  FROM accounts a JOIN account_identities i ON i.account_id = a.id
  WHERE a.id = ?1 AND i.provider = ?2 AND i.subject = ?3`,
		)
			.bind(accountId, provider, subject)
			.first<AccountForRetry>();
		if (!existing) return null;
		const recoveryMatches = await proofMatches(recoveryAuthKey, existing.recovery_hash);
		if (
			!recoveryMatches ||
			existing.username !== name.value ||
			existing.recovery_wrapped_key !== recoveryWrappedKey ||
			existing.key_id !== keyId
		) {
			return null;
		}
		if (existing.disabled_at !== null) return accountDisabled();
		const results = await env.DB.batch(
			sessionStatements(
				env.DB,
				{
					accountId: existing.id,
					deviceId: device.value.id,
					...session,
					sealedName: device.value.sealedName,
					now: Math.max(now, Date.now()),
				},
				[["recovery_hash", existing.recovery_hash]],
				{ flowId: flow.id, attemptId: flow.completion_attempt_id as string },
			),
		);
		const outcome = sessionOutcome(results, 0);
		if (!outcome.ok) return null;
		return sessionAnswer(existing, session.token, outcome.evicted, 201);
	};
	if (flow.consumed_at !== null) {
		if (
			flow.completion_succeeded !== 1 ||
			flow.completion_kind !== "social_signup" ||
			flow.completion_account_id !== accountId ||
			flow.completion_attempt_id === null
		) {
			return flowExpired();
		}
		return (await retried()) ?? flowExpired();
	}
	const completionNow = Math.max(now, Date.now());
	const attemptId = completionAttemptId();

	// A passwordless account: random KDF salt, auth hash and password wrap that
	// satisfy every CHECK and that no client holds. The KEK of that wrap never
	// existed: the wrap is random bytes of the right length.
	const epoch = [...randomBytes(8)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
	const account = { id: accountId, username: name.value, key_id: keyId, epoch, has_password: 0 };
	try {
		const results = await env.DB.batch([
			claimCompletion(env, flow.id, attemptId, "social_signup", accountId, completionNow),
			env.DB.prepare(
				"DELETE FROM purges WHERE prefix = ?1 AND EXISTS (SELECT 1 FROM auth_flows WHERE id = ?2 AND completion_attempt_id = ?3)",
			).bind(accountPrefix(accountId), flow.id, attemptId),
			env.DB.prepare(
				`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
	  recovery_wrapped_key, key_id, epoch, created_at, has_password)
  SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0
  WHERE EXISTS (SELECT 1 FROM auth_flows WHERE id = ?14 AND completion_attempt_id = ?15)`,
			).bind(
				accountId,
				name.value,
				KDF_DEFAULT.m_kib,
				KDF_DEFAULT.t,
				KDF_DEFAULT.p,
				base64url(randomBytes(16)),
				await sha256Hex(randomBytes(32)),
				base64url(randomBytes(73)),
				recoveryHash,
				recoveryWrappedKey,
				keyId,
				epoch,
				completionNow,
				flow.id,
				attemptId,
			),
			env.DB.prepare(
				`INSERT INTO account_identities (account_id, provider, subject, created_at)
  SELECT ?1, ?2, ?3, ?4 WHERE EXISTS (SELECT 1 FROM auth_flows WHERE id = ?5 AND completion_attempt_id = ?6)
    AND EXISTS (SELECT 1 FROM accounts WHERE id = ?1)`,
			).bind(accountId, provider, subject, completionNow, flow.id, attemptId),
			...sessionStatements(
				env.DB,
				{
					accountId,
					deviceId: device.value.id,
					...session,
					sealedName: device.value.sealedName,
					now: completionNow,
				},
				[["recovery_hash", recoveryHash]],
				{ flowId: flow.id, attemptId },
			),
			markCompletionSucceeded(
				env,
				flow.id,
				attemptId,
				"social_signup",
				accountId,
				`EXISTS (SELECT 1 FROM accounts WHERE id = ?4)
  AND EXISTS (SELECT 1 FROM account_identities WHERE account_id = ?4 AND provider = ?5 AND subject = ?6)
  AND EXISTS (SELECT 1 FROM sessions WHERE account_id = ?4 AND device_id = ?7 AND token_hash = ?8 AND revoked_at IS NULL)`,
				[provider, subject, device.value.id, session.tokenHash],
			),
		]);
		if (results[0].results.length !== 1) return flowExpired();
		const outcome = sessionOutcome(results, 4);
		if (!outcome.ok || results[7].results.length !== 1) return flowExpired();
	} catch (err) {
		const refused = refusal(err);
		const unique = constraint(err, "UNIQUE");
		if (refused?.kind === "signup_closed") return error(403, "signup_closed", "Sign-ups are closed.", { reason: refused.detail });
		if (unique) {
			const taken = await env.DB.batch([
				env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE id = ?1").bind(accountId),
				env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE username = ?1").bind(name.value),
				env.DB.prepare("SELECT 1 AS taken FROM account_identities WHERE provider = ?1 AND subject = ?2").bind(provider, subject),
			]);
			if (taken[2].results.length > 0) return identityTaken(provider);
			if (taken[1].results.length > 0) return error(409, "username_taken", "That username is taken.");
			if (taken[0].results.length > 0) return error(409, "account_id_taken", "That account ID is taken; generate another.");
		}
		throw err;
	}
	return sessionAnswer(account, session.token, null, 201);
};

// ----------------------------------------------------- session routes: proofs

/** Authenticates, passes SESSION_LIMITER, then runs `handler` with the pepper's keys. */
function withSessionKeys(
	handler: (request: Request, env: Env, now: number, keys: PepperKeys, session: Session) => Promise<Response>,
): PepperHandler {
	return async (request, env, now, keys) => {
		const session = await authenticate(request, env, now);
		if (session instanceof Response) return session;
		return (await sessionFilter(env.SESSION_LIMITER, session.accountId)) ?? handler(request, env, now, keys, session);
	};
}

interface ProofAccount {
	id: string;
	auth_hash: string;
	recovery_hash: string;
	has_password: number;
}

/** A checked proof: the statement guard it holds under, and the provider proof flow to consume with the change. */
type CheckedProof =
	| { ok: true; kind: "password"; authHash: string }
	| { ok: true; kind: "provider"; flowId: string }
	| { ok: false; response: Response };

/**
 * Checks a `proof` for a sensitive change (docs/ACCOUNTS.md › Changing a way in
 * needs a fresh proof). A password proof counts `password-account`. A provider
 * proof is accepted only where `provider` names the provider it may act on
 * (unlinking that same provider), and only while unconsumed. Passkey proofs
 * need the passkey routes (step 2) and are refused with `reauth_required` here.
 */
async function checkProof(env: Env, now: number, session: Session, account: ProofAccount, value: unknown, provider: string | null): Promise<CheckedProof> {
	if (!isObject(value)) return { ok: false, response: reauthRequired() };
	if (value.kind === "password") {
		const limited = await count(env.DB, "password-account", accountKey("password-account", session.accountId), now);
		if (limited) return { ok: false, response: limited };
		const authKey = proof32(value.auth_key);
		if (!authKey) return { ok: false, response: badRequest("proof.auth_key must be 32 bytes of canonical base64url.") };
		// A passwordless account's auth hash is random; nobody holds a key for it.
		if (!(await proofMatches(authKey, account.auth_hash)) || account.has_password !== 1) return { ok: false, response: invalidCredentials() };
		return { ok: true, kind: "password", authHash: account.auth_hash };
	}
	if (value.kind === "provider" && provider !== null) {
		const flowId = value.flow;
		if (typeof flowId !== "string" || !isFlowId(flowId)) return { ok: false, response: reauthRequired() };
		const flow = await flowById(env, flowId);
		const usable =
			flow !== null &&
			flow.kind === provider &&
			flow.intent === "proof" &&
			flow.account_id === session.accountId &&
			flow.subject !== null &&
			flow.consumed_at === null &&
			flow.expires_at > now;
		if (!usable) return { ok: false, response: reauthRequired() };
		const linked = await env.DB.prepare("SELECT 1 AS linked FROM account_identities WHERE account_id = ?1 AND provider = ?2 AND subject = ?3")
			.bind(session.accountId, provider, flow.subject)
			.first();
		if (!linked) return { ok: false, response: reauthRequired() };
		return { ok: true, kind: "provider", flowId };
	}
	return { ok: false, response: reauthRequired() };
}

async function proofAccount(env: Env, accountId: string): Promise<ProofAccount | null> {
	return env.DB.prepare("SELECT id, auth_hash, recovery_hash, has_password FROM accounts WHERE id = ?1").bind(accountId).first<ProofAccount>();
}

/**
 * `POST /v2/account/sign-in/identities/<provider>/start`: a link flow (with a
 * password or passkey proof) or a proof flow (the session is enough: the
 * provider sign-in is the proof). Counts `proof-account`.
 */
export function identityStart(name: string): PepperHandler {
	return withSessionKeys(async (request, env, now, keys, session) => {
		const provider = servedProvider(env, name);
		if (provider instanceof Response) return provider;
		const limited = await count(env.DB, "proof-account", accountKey("proof-account", session.accountId), now);
		if (limited) return limited;
		const body = await readObject(request);
		if (!body.ok) return body.response;
		const b = body.value;
		if (b.intent !== "link" && b.intent !== "proof") return badRequest('intent must be "link" or "proof".');
		const device = checkDevice(b.device, false);
		if (!device.ok) return respond(device);
		// The flow is this session's device's: the same ID, proving itself as its session row records.
		const row = await env.DB.prepare("SELECT proof_hash FROM sessions WHERE token_hash = ?1").bind(session.tokenHash).first<{ proof_hash: string }>();
		if (device.value.id !== session.deviceId || !row || !(await proofMatches(device.value.proof, row.proof_hash))) {
			return badRequest("The device isn't this session's.");
		}
		if (b.intent === "link") {
			const account = await proofAccount(env, session.accountId);
			if (!account) return invalidCredentials();
			const checked = await checkProof(env, now, session, account, b.proof, null);
			if (!checked.ok) return checked.response;
		}
		return startFlow(env, keys, now, { provider, intent: b.intent, accountId: session.accountId, device: device.value, body: b });
	});
}

/**
 * `POST /v2/account/sign-in/identities/<provider>/unlink`: removes the identity.
 * Refused with 409 `last_method` when the account would have no password, no
 * identity and no passkey, unless the body proves the recovery key. Works for
 * a provider this server no longer configures, so an old link can always go.
 */
export function unlink(name: string): PepperHandler {
	return withSessionKeys(async (request, env, now, _keys, session) => {
		if (!PROVIDER_NAME.test(name)) return notFound();
		const body = await readObject(request);
		if (!body.ok) return body.response;
		const account = await proofAccount(env, session.accountId);
		if (!account) return invalidCredentials();
		const checked = await checkProof(env, now, session, account, body.value.proof, name);
		if (!checked.ok) return checked.response;

		let recovered = false;
		if (body.value.recovery_auth_key !== undefined) {
			const recovery = proof32(body.value.recovery_auth_key);
			if (!recovery) return badRequest("recovery_auth_key must be 32 bytes of canonical base64url.");
			if (!(await proofMatches(recovery, account.recovery_hash))) return invalidCredentials();
			recovered = true;
		}
		const linked = await env.DB.prepare("SELECT 1 AS linked FROM account_identities WHERE account_id = ?1 AND provider = ?2")
			.bind(session.accountId, name)
			.first();
		if (!linked) return notFound();

		// The rule, the proof's guard and the deletion are one statement, so two
		// unlinks of an account's last two methods can't both pass.
		const others = `(?3 = 1 OR (SELECT has_password FROM accounts WHERE id = ?1) = 1
    OR EXISTS (SELECT 1 FROM account_identities WHERE account_id = ?1 AND provider != ?2)
    OR EXISTS (SELECT 1 FROM account_passkeys WHERE account_id = ?1))`;
		const deletion = `DELETE FROM account_identities WHERE account_id = ?1 AND provider = ?2 AND ${others} AND `;
		const statements = [
			checked.kind === "password"
				? env.DB.prepare(`${deletion}EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND auth_hash = ?4) RETURNING provider`).bind(
						session.accountId,
						name,
						recovered ? 1 : 0,
						checked.authHash,
					)
				: env.DB.prepare(`${deletion}EXISTS (SELECT 1 FROM auth_flows WHERE id = ?4 AND consumed_at = ?5) RETURNING provider`).bind(
						session.accountId,
						name,
						recovered ? 1 : 0,
						checked.flowId,
						now,
					),
		];
		if (checked.kind === "provider") {
			// Consumed first, in the same batch: a provider proof is single use even when the change is refused.
			statements.unshift(
				env.DB.prepare("UPDATE auth_flows SET consumed_at = ?2 WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2").bind(checked.flowId, now),
			);
		}
		const results = await env.DB.batch(statements);
		if (results[results.length - 1].results.length === 1) return new Response(null, { status: 204 });
		if (checked.kind === "provider" && (results[0].meta?.changes ?? 0) === 0) return reauthRequired();
		const still = await env.DB.prepare("SELECT 1 AS linked FROM account_identities WHERE account_id = ?1 AND provider = ?2")
			.bind(session.accountId, name)
			.first();
		if (!still) return notFound();
		if (checked.kind === "password") {
			const current = await proofAccount(env, session.accountId);
			if (current?.auth_hash !== checked.authHash) return invalidCredentials();
		}
		return error(409, "last_method", "This is the account's last way to sign in; prove the recovery key to remove it.");
	});
}

/** `GET /v2/account/sign-in`: the account's ways in. Never a subject or a public key. */
export const methods = async (request: Request, env: Env, now: number): Promise<Response> => {
	const session = await authenticate(request, env, now);
	if (session instanceof Response) return session;
	const limited = await sessionFilter(env.SESSION_LIMITER, session.accountId);
	if (limited) return limited;
	const [account, identities, passkeys] = await env.DB.batch([
		env.DB.prepare("SELECT username, has_password FROM accounts WHERE id = ?1").bind(session.accountId),
		env.DB.prepare("SELECT provider, created_at FROM account_identities WHERE account_id = ?1 ORDER BY created_at, provider").bind(
			session.accountId,
		),
		env.DB.prepare(
			`SELECT credential_id, created_at, last_used_at, wrapped_key IS NOT NULL AS prf, transports
  FROM account_passkeys WHERE account_id = ?1 ORDER BY created_at, credential_id`,
		).bind(session.accountId),
	]);
	const row = account.results[0] as { username: string; has_password: number } | undefined;
	if (!row) return error(401, "unauthorized", "Not signed in.", { reason: "unknown" });
	const hour = 3_600_000;
	return json({
		username: row.username,
		has_password: row.has_password === 1,
		identities: (identities.results as { provider: string; created_at: number }[]).map((r) => ({ provider: r.provider, created_at: r.created_at })),
		passkeys: (
			passkeys.results as { credential_id: string; created_at: number; last_used_at: number | null; prf: number; transports: string }[]
		).map((r) => ({
			credential_id: r.credential_id,
			created_at: r.created_at,
			last_used_at: r.last_used_at === null ? null : Math.floor(r.last_used_at / hour) * hour,
			prf: r.prf === 1,
			transports: r.transports === "" ? [] : r.transports.split(" "),
		})),
		recovery: true,
	});
};
