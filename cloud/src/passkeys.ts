// Passkey registration and assertion (docs/ACCOUNTS.md › Sign-in methods ›
// Passkeys). The Worker is the relying party: it issues a single-use
// five-minute challenge, checks the ceremony with `webauthn.ts`, and stores
// only the public key, algorithm, AAGUID, transports, sign count and the
// optional PRF wrap. It never sees a private key or a PRF output.
//
// The same routes serve a platform passkey (macOS AuthenticationServices,
// Windows `webauthn.h`) and a Linux security key: the app turns CTAP2 into the
// same WebAuthn registration and assertion messages, so there is no
// Linux-specific route, attestation format or column.

import {
	type PepperKeys,
	hashToken,
	hex,
	newToken,
	proofMatches,
	randomBytes,
	sha256Hex,
} from "./crypto";
import type { Env } from "./env";
import { accountDisabled, badRequest, constraint, error, invalidCredentials, json, refusal } from "./http";
import { accountKey, clientIp, count, ipGroup, ipKey } from "./limits";
import {
	type NewDevice,
	base64url,
	checkDevice,
	fromBase64url,
	isAccountId,
	isKeyId,
	isWrappedKey,
	proof32,
} from "./protocol";
import { flowForDevice, flowExpired, isFlowId, newFlow, passkeyAdvertisement } from "./sign_in";
import { isReserved, normalizeUsername } from "./usernames";
import {
	type Expected,
	COSE_ALGORITHMS,
	bytesField,
	signCountAccepted,
	userHandle,
	verifyAssertion,
	verifyRegistration,
} from "./webauthn";
import { readObject, respond, sessionOutcome, sessionStatements } from "./auth";

/** The session shape every sign-in route answers (docs/ACCOUNTS.md › Sign-in routes). */
interface SessionAnswer {
	kind: "session";
	account_id: string;
	username: string;
	token: string;
	key_id: string;
	epoch: string;
	has_password: boolean;
	passkey_wrapped_key: string | null;
	evicted_device: { id: string; sealed_name: string | null } | null;
}

interface PasskeyAccount {
	id: string;
	username: string;
	key_id: string;
	epoch: string;
	has_password: number;
	disabled_at: number | null;
	auth_hash: string;
}

interface PasskeyRow {
	credential_id: string;
	account_id: string;
	public_key: string;
	alg: -8 | -7 | -257;
	sign_count: number;
	transports: string;
	wrapped_key: string | null;
}

const RP_NOT_CONFIGURED = (): Response => error(400, "unsupported_method", "This server doesn't offer passkeys.");
const INVALID = (): Response => invalidCredentials();

/** The relying party and accepted origins, or the 400 for a deployment without them. */
function expected(env: Env, challenge: string): Expected | Response {
	// The same validated advertisement health uses, so a malformed or
	// out-of-scope origin can't be accepted here but hidden there.
	const advertisement = passkeyAdvertisement(env);
	if (!advertisement) return RP_NOT_CONFIGURED();
	return { rpId: advertisement.rp_id, origins: advertisement.origins, challenge };
}

/** The transports a start lists, from the space-separated hints the server stored. */
const transports = (value: string): string[] => value.split(" ").filter((t) => t !== "");

/**
 * Reads a `transports` array from a registration body: at most 32 lowercase
 * hints, de-duplicated, as the space-separated string the column holds.
 */
function transportsField(value: unknown): string | null {
	if (value === undefined || value === null) return "";
	if (!Array.isArray(value) || value.length > 32) return null;
	const seen: string[] = [];
	for (const item of value) {
		if (typeof item !== "string" || !/^[a-z][a-z0-9-]{0,31}$/.test(item)) return null;
		if (!seen.includes(item)) seen.push(item);
	}
	const joined = seen.join(" ");
	return joined.length <= 64 ? joined : null;
}

function optionalWrap(value: unknown): string | null | undefined {
	if (value === undefined || value === null) return null;
	return typeof value === "string" && isWrappedKey(value) ? value : undefined;
}

// ----------------------------------------------------------- sign-in: start

/**
 * `POST /v2/auth/passkey/start` — username-less by default, so the platform
 * shows its passkey picker. A typed username lists that account's credentials
 * to help security keys without discoverable storage; an unknown name lists
 * nothing, which says no more than sign-up already does.
 */
export const passkeyStart = async (request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> => {
	const advertisement = passkeyAdvertisement(env);
	if (!advertisement) return RP_NOT_CONFIGURED();
	const limited = await count(env.DB, "passkey-challenge", await ipKey(keys, "passkey-challenge", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const device = checkDevice(body.value.device, false);
	if (!device.ok) return respond(device);
	let accountId: string | null = null;
	if (body.value.username !== undefined) {
		if (typeof body.value.username !== "string") return badRequest("username must be a string.");
		const name = normalizeUsername(body.value.username);
		if (name === null) return error(400, "invalid_username", "Not a valid username.");
		const account = await env.DB.prepare("SELECT id FROM accounts WHERE username = ?1").bind(name).first<{ id: string }>();
		accountId = account?.id ?? null;
	}
	const challenge = base64url(randomBytes(32));
	const flowId = await newFlow(
		env,
		keys,
		{
			kind: "passkey_signin",
			intent: "signin",
			account_id: accountId,
			device_id: device.value.id,
			device_proof_hash: await sha256Hex(device.value.proof),
			challenge,
			rp_id: advertisement.rp_id,
			nonce_hash: null,
			state_hash: null,
			pkce_hash: null,
			redirect_uri: null,
			provider: null,
			subject: null,
			device_code: null,
			next_poll_at: null,
			expires_at: now + 5 * 60_000,
		},
		now,
	);
	let allow: { id: string; transports: string[] }[] = [];
	if (accountId !== null) {
		const { results } = await env.DB.prepare(
			"SELECT credential_id, transports FROM account_passkeys WHERE account_id = ?1 ORDER BY created_at, credential_id",
		)
			.bind(accountId)
			.all<{ credential_id: string; transports: string }>();
		allow = results.map((row) => ({ id: row.credential_id, transports: transports(row.transports) }));
	}
	return json({
		flow_id: flowId,
		challenge,
		rp_id: advertisement.rp_id,
		allow_credentials: allow,
		user_verification: "required",
		timeout: 300_000,
	});
};

// ---------------------------------------------------------- sign-in: finish

/** The credential and its account, or null. */
async function credential(env: Env, credentialId: string): Promise<(PasskeyRow & { account: PasskeyAccount }) | null> {
	const row = await env.DB.prepare(
		`SELECT p.credential_id, p.account_id, p.public_key, p.alg, p.sign_count, p.transports, p.wrapped_key,
  a.username, a.key_id, a.epoch, a.has_password, a.disabled_at, a.auth_hash
  FROM account_passkeys p JOIN accounts a ON a.id = p.account_id WHERE p.credential_id = ?1`,
	)
		.bind(credentialId)
		.first<
			PasskeyRow & {
				username: string;
				key_id: string;
				epoch: string;
				has_password: number;
				disabled_at: number | null;
				auth_hash: string;
			}
		>();
	if (!row) return null;
	return {
		credential_id: row.credential_id,
		account_id: row.account_id,
		public_key: row.public_key,
		alg: row.alg,
		sign_count: row.sign_count,
		transports: row.transports,
		wrapped_key: row.wrapped_key,
		account: {
			id: row.account_id,
			username: row.username,
			key_id: row.key_id,
			epoch: row.epoch,
			has_password: row.has_password,
			disabled_at: row.disabled_at,
			auth_hash: row.auth_hash,
		},
	};
}

/** Whether this device already holds an active session for the account, proving itself. */
async function knownDevice(env: Env, accountId: string, device: NewDevice): Promise<boolean> {
	const row = await env.DB.prepare(
		"SELECT proof_hash FROM sessions WHERE account_id = ?1 AND device_id = ?2 AND revoked_at IS NULL",
	)
		.bind(accountId, device.id)
		.first<{ proof_hash: string }>();
	return row !== null && (await proofMatches(device.proof, row.proof_hash));
}

function sessionAnswer(
	account: PasskeyAccount,
	token: string,
	evicted: SessionAnswer["evicted_device"],
	wrapped: string | null,
	status = 201,
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
			passkey_wrapped_key: wrapped,
			evicted_device: evicted,
		} satisfies SessionAnswer,
		status,
	);
}

/**
 * `POST /v2/auth/passkey/finish` — checks the assertion, the account's enabled
 * flag and the sign count, then issues the same session shape as every other
 * sign-in. A stored sign count of 0 is never enforced; when both are above 0 a
 * non-growing count is a possible clone and is refused.
 */
export const passkeyFinish = async (request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> => {
	let limited = await count(env.DB, "login-ip", await ipKey(keys, "login-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const device = checkDevice(b.device, false);
	if (!device.ok) return respond(device);
	const credentialId = bytesField(b.credential_id, 16, 1023);
	if (!credentialId) return badRequest("credential_id must be 16–1023 bytes of canonical base64url.");
	const authenticatorData = bytesField(b.authenticator_data, 37, 8192);
	const clientDataJson = bytesField(b.client_data_json, 1, 8192);
	const signature = bytesField(b.signature, 1, 1024);
	if (!authenticatorData || !clientDataJson || !signature) return badRequest("The assertion is malformed.");
	const flowId = typeof b.flow_id === "string" ? b.flow_id : "";
	if (!isFlowId(flowId)) return badRequest("flow_id must be 32 bytes of canonical base64url.");
	const flow = await flowForDevice(env, flowId, "passkey_signin", device.value, now);
	if (flow instanceof Response) return flow;
	const credentialBase64 = base64url(credentialId);
	const found = await credential(env, credentialBase64);
	// An assertion for a credential this server doesn't hold, or one on another
	// account than the flow names, is indistinguishable from a bad signature.
	if (!found) return INVALID();
	if (flow.account_id !== null && flow.account_id !== found.account_id) return INVALID();
	const known = await knownDevice(env, found.account_id, device.value);
	if (!known) {
		limited = await count(env.DB, "login-user", accountKey("login-user", found.account_id), now);
		if (limited) return limited;
	}

	// A consumed sign-in flow is never replayed: unlike a sign-up, another
	// answer can be obtained without creating anything, and a replay would skip
	// the assertion check. The client starts a fresh challenge.
	if (flow.consumed_at !== null) return flowExpired();

	const expectedValue = expected(env, flow.challenge ?? "");
	if (expectedValue instanceof Response) return expectedValue;
	const publicKey = fromBase64url(found.public_key);
	if (!publicKey) return INVALID();
	const verified = await verifyAssertion(
		{ authenticatorData, clientDataJson, signature },
		{ publicKey, alg: found.alg },
		expectedValue,
	);
	if (!verified) return INVALID();
	if (!signCountAccepted(found.sign_count, verified.signCount)) return INVALID();
	// The account is checked after the signature, so a disabled flag never
	// short-circuits a credential failure.
	if (found.account.disabled_at !== null) return accountDisabled();

	const token = newToken();
	// The count and the flow are claimed first, both atomically. Only then is
	// a session made, so a clone that raced this request (or a flow another
	// request already used) never leaves a token behind.
	const claimed = await env.DB.batch([
		env.DB.prepare(
			`UPDATE account_passkeys SET sign_count = ?3, last_used_at = ?4
  WHERE credential_id = ?1 AND account_id = ?2 AND (sign_count = 0 OR sign_count < ?3)
  RETURNING credential_id`,
		).bind(credentialBase64, found.account_id, verified.signCount, now),
		env.DB.prepare(
			`UPDATE auth_flows SET consumed_at = ?2, account_id = ?3
  WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2 RETURNING id`,
		).bind(flowId, now, found.account_id),
	]);
	// Another assertion advanced the count first: refuse this one rather than
	// accept a clone that raced it.
	if (claimed[0].results.length !== 1 || claimed[1].results.length !== 1) return INVALID();
	const results = await env.DB.batch(
		sessionStatements(
			env.DB,
			{
				accountId: found.account_id,
				deviceId: device.value.id,
				tokenHash: await hashToken(token),
				proofHash: await sha256Hex(device.value.proof),
				sealedName: null,
				now,
			},
			[["auth_hash", found.account.auth_hash]],
		),
	);
	const outcome = sessionOutcome(results, 0);
	if (!outcome.ok) return INVALID();
	return sessionAnswer(found.account, token, outcome.evicted, found.wrapped_key, 200);
};

// ---------------------------------------------------------- sign-up: start

interface SignupStateRow {
	signup: string | null;
	max_accounts: number | null;
	accounts: number;
	signups_today: number | null;
	signups_per_day: number | null;
}

/** Closed, full or busy, as the sign-up triggers would answer; null when a new account may be made. */
async function signupRefusal(env: Env, now: number, username: string): Promise<Response | null> {
	const row = await env.DB.prepare(
		`SELECT (SELECT value FROM settings WHERE name = 'signup') AS signup,
  (SELECT max_accounts FROM defaults) AS max_accounts, (SELECT count(*) FROM accounts) AS accounts,
  (SELECT CASE WHEN day >= ?1 THEN count ELSE 0 END FROM daily WHERE name = 'signups') AS signups_today,
  (SELECT signups_per_day FROM defaults) AS signups_per_day`,
	)
		.bind(Math.floor(now / 86_400_000))
		.first<SignupStateRow>();
	const state = row ?? { signup: "closed", max_accounts: 0, accounts: 0, signups_today: 0, signups_per_day: 0 };
	const setting = state.signup ?? "closed";
	const open = setting === "open" || setting === `only:${username}`;
	if (!open) return error(403, "signup_closed", "Sign-ups are closed.", { reason: "closed" });
	if (state.accounts >= (state.max_accounts ?? 0)) return error(403, "signup_closed", "The server is full.", { reason: "full" });
	if ((state.signups_today ?? 0) >= (state.signups_per_day ?? 0)) {
		return error(403, "signup_closed", "The server has taken enough new accounts today.", { reason: "busy" });
	}
	return null;
}

/** The registration options the app hands to the authenticator. */
function registrationOptions(_env: Env, flowId: string, challenge: string, username: string, handle: string, rpId: string) {
	return {
		flow_id: flowId,
		challenge,
		rp_id: rpId,
		user: { id: handle, name: username, displayName: username },
		pub_key_cred_params: [...COSE_ALGORITHMS],
		authenticator_selection: { residentKey: "required", userVerification: "required" },
		exclude_credentials: [],
	};
}

/**
 * `POST /v2/auth/passkey/signup/start` — validates the proposed name before a
 * username exists, counts `signup-ip`, and returns the registration options.
 * The username travels in the flow's `subject` column (the schema has no
 * username column) so the finish checks it. The finish repeats every check.
 */
export const passkeySignupStart = async (request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> => {
	const advertisement = passkeyAdvertisement(env);
	if (!advertisement) return RP_NOT_CONFIGURED();
	const limited = await count(env.DB, "signup-ip", await ipKey(keys, "signup-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	if (typeof b.username !== "string") return badRequest("username must be a string.");
	const username = normalizeUsername(b.username);
	if (username === null) return error(400, "invalid_username", "Not a valid username.");
	if (isReserved(username)) return error(400, "reserved_username", "That username is reserved.");
	if (typeof b.account_id !== "string" || !isAccountId(b.account_id)) return badRequest("account_id must be a_ and 32 hex digits.");
	const device = checkDevice(b.device, false);
	if (!device.ok) return respond(device);
	const takenName = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE username = ?1").bind(username).first();
	if (takenName) return error(409, "username_taken", "That username is taken.");
	const takenId = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE id = ?1").bind(b.account_id).first();
	if (takenId) return error(409, "account_id_taken", "That account ID is taken; generate another.");
	const refusal = await signupRefusal(env, now, username);
	if (refusal) return refusal;

	const challenge = base64url(randomBytes(32));
	const flowId = await newFlow(
		env,
		keys,
		{
			kind: "passkey_signup",
			intent: "signup",
			account_id: b.account_id,
			device_id: device.value.id,
			device_proof_hash: await sha256Hex(device.value.proof),
			challenge,
			rp_id: advertisement.rp_id,
			nonce_hash: null,
			state_hash: null,
			pkce_hash: null,
			redirect_uri: null,
			provider: null,
			// The proposed username, checked by the finish.
			subject: username,
			device_code: null,
			next_poll_at: null,
			expires_at: now + 5 * 60_000,
		},
		now,
	);
	return json(registrationOptions(env, flowId, challenge, username, await userHandle(b.account_id), advertisement.rp_id));
};

// --------------------------------------------------------- sign-up: finish

/** A random 32-byte value in base64url, for the unopenable password material a passkey account stores. */
const randomProof = (): string => base64url(randomBytes(32));

/**
 * `POST /v2/auth/passkey/signup/finish` — verifies the attestation, inserts the
 * account, passkey, recovery material and session in one batch (the same
 * `accounts` insert as every other method, so every gate applies), and answers
 * the session shape. `has_password = 0`; the KDF parameters and password wrap
 * are random values no client holds, so the row looks like any other to D1.
 */
export const passkeySignupFinish = async (request: Request, env: Env, now: number): Promise<Response> => {
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const device = checkDevice(b.device, true);
	if (!device.ok) return respond(device);
	const flowId = typeof b.flow_id === "string" ? b.flow_id : "";
	if (!isFlowId(flowId)) return badRequest("flow_id must be 32 bytes of canonical base64url.");
	const flow = await flowForDevice(env, flowId, "passkey_signup", device.value, now);
		if (flow instanceof Response) return flow;
	if (typeof b.username !== "string") return badRequest("username must be a string.");
	const username = normalizeUsername(b.username);
	if (username === null) return error(400, "invalid_username", "Not a valid username.");
	if (isReserved(username)) return error(400, "reserved_username", "That username is reserved.");
	if (typeof b.account_id !== "string" || !isAccountId(b.account_id)) return badRequest("account_id must be a_ and 32 hex digits.");
	// The flow is what the start checked; a changed name or account ID is a
	// different ceremony, not a retry.
	if (flow.subject !== null && flow.subject !== username) return badRequest("This flow belongs to another username.");
	if (flow.account_id !== null && flow.account_id !== b.account_id) return badRequest("This flow belongs to another account.");

	const credentialId = bytesField(b.credential_id, 16, 1023);
	const attestationObject = bytesField(b.attestation_object, 1, 16_384);
	const clientDataJson = bytesField(b.client_data_json, 1, 8192);
	if (!credentialId || !attestationObject || !clientDataJson) return badRequest("The registration is malformed.");
	const transportValue = transportsField(b.transports);
	if (transportValue === null) return badRequest("transports must be lowercase hints.");
	const wrapped = optionalWrap(b.passkey_wrapped_key);
	if (wrapped === undefined) return badRequest("passkey_wrapped_key must be 98 base64url characters.");
	const recoveryAuthKey = proof32(b.recovery_auth_key);
	if (!recoveryAuthKey) return badRequest("recovery_auth_key must be 32 bytes of canonical base64url.");
	if (typeof b.recovery_wrapped_key !== "string" || !isWrappedKey(b.recovery_wrapped_key)) {
		return badRequest("recovery_wrapped_key must be 98 base64url characters.");
	}
	if (typeof b.key_id !== "string" || !isKeyId(b.key_id)) return badRequest("key_id must be 16 hex digits.");

	// An idempotent retry: the same request after a lost answer. The account,
	// credential, recovery hash and key ID must all be the ones it created.
	if (flow.consumed_at !== null) {
		const existing = await env.DB.prepare(
			`SELECT a.id, a.username, a.epoch, a.key_id, a.recovery_hash, a.recovery_wrapped_key, a.has_password, a.auth_hash,
   (SELECT count(*) FROM account_passkeys p WHERE p.account_id = a.id AND p.credential_id = ?2) AS credential
   FROM accounts a WHERE a.id = ?1 AND a.username = ?3`,
		)
			.bind(b.account_id, base64url(credentialId), username)
			.first<{
				id: string;
				username: string;
				epoch: string;
				key_id: string;
				recovery_hash: string;
				recovery_wrapped_key: string;
				has_password: number;
				auth_hash: string;
				credential: number;
			}>();
		if (!existing || existing.credential !== 1 || existing.key_id !== b.key_id) return flowExpired();
		if (existing.recovery_wrapped_key !== b.recovery_wrapped_key) return flowExpired();
		if (!(await proofMatches(recoveryAuthKey, existing.recovery_hash))) return flowExpired();
		const token = newToken();
		const results = await env.DB.batch(
			sessionStatements(
				env.DB,
				{
					accountId: existing.id,
					deviceId: device.value.id,
					tokenHash: await hashToken(token),
					proofHash: await sha256Hex(device.value.proof),
					sealedName: device.value.sealedName,
					now,
				},
				[["auth_hash", existing.auth_hash]],
			),
		);
		const outcome = sessionOutcome(results, 0);
		if (!outcome.ok) return flowExpired();
		return sessionAnswer(
			{
				id: existing.id,
				username: existing.username,
				key_id: existing.key_id,
				epoch: existing.epoch,
				has_password: existing.has_password,
				disabled_at: null,
				auth_hash: existing.auth_hash,
			},
			token,
			outcome.evicted,
			null,
		);
	}

	const expectedValue = expected(env, flow.challenge ?? "");
	if (expectedValue instanceof Response) return expectedValue;
	const registration = await verifyRegistration({ credentialId, attestationObject, clientDataJson }, expectedValue);
	if (!registration) return INVALID();
	if (username !== flow.subject) return badRequest("This flow belongs to another username.");

	const recoveryHash = await sha256Hex(recoveryAuthKey);
	const epoch = hex(randomBytes(8));
	const sessionToken = newToken();
	const tokenHash = await hashToken(sessionToken);
	const proofHash = await sha256Hex(device.value.proof);
	const accountPassword = randomProof();
	const authHash = await sha256Hex(accountPassword);
	const salt = base64url(randomBytes(16));
	const wrappedPassword = base64url(randomBytes(73));
	try {
		const results = await env.DB.batch([
			env.DB.prepare(
				`UPDATE auth_flows SET consumed_at = ?2, account_id = ?3 WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2 RETURNING id`,
			).bind(flowId, now, b.account_id),
			// Each insert holds only when the flow's one consume just happened, so a
			// second finish can't create an account, credential or session.
			env.DB.prepare(
				`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
  recovery_wrapped_key, key_id, epoch, created_at, has_password)
  SELECT ?1, ?2, 1048576, 3, 4, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0
  WHERE EXISTS (SELECT 1 FROM auth_flows WHERE id = ?11 AND consumed_at = ?10)
  RETURNING id`,
			).bind(
				b.account_id,
				username,
				salt,
				authHash,
				wrappedPassword,
				recoveryHash,
				b.recovery_wrapped_key,
				b.key_id,
				epoch,
				now,
				flowId,
			),
			env.DB.prepare(
				`INSERT INTO account_passkeys (credential_id, account_id, public_key, alg, sign_count, transports, wrapped_key,
  aaguid, created_at, last_used_at)
  SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9
  WHERE EXISTS (SELECT 1 FROM accounts WHERE id = ?2 AND created_at = ?9)
  RETURNING credential_id`,
			).bind(
				base64url(credentialId),
				b.account_id,
				base64url(registration.publicKey),
				registration.alg,
				registration.signCount,
				transportValue,
				wrapped,
				hex(registration.aaguid),
				now,
			),
			env.DB.prepare(
				`INSERT INTO sessions (account_id, device_id, token_hash, proof_hash, sealed_name, created_at, last_seen_at)
  SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?6
  WHERE EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND created_at = ?6)
  RETURNING device_id`,
			).bind(b.account_id, device.value.id, tokenHash, proofHash, device.value.sealedName, now),
		]);
		if (
			results[0].results.length !== 1 ||
			results[1].results.length !== 1 ||
			results[2].results.length !== 1 ||
			results[3].results.length !== 1
		) {
			return flowExpired();
		}
	} catch (err) {
		const refused = refusal(err);
		if (refused?.kind === "signup_closed") {
			return error(403, "signup_closed", "Sign-ups are closed.", { reason: refused.detail });
		}
		if (constraint(err, "UNIQUE")) {
			const taken = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE id = ?1").bind(b.account_id).first();
			if (taken) return error(409, "account_id_taken", "That account ID is taken; generate another.");
			const name = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE username = ?1").bind(username).first();
			if (name) return error(409, "username_taken", "That username is taken.");
			return error(409, "credential_exists", "That passkey is already added.");
		}
		throw err;
	}
	return sessionAnswer(
		{
			id: b.account_id,
			username,
			key_id: b.key_id,
			epoch,
			has_password: 0,
			disabled_at: null,
			auth_hash: "",
		},
		sessionToken,
		null,
		null,
	);
};

