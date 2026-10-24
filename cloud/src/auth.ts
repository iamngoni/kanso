// Accounts and sessions (docs/ACCOUNTS.md › Routes): prelogin, sign-up,
// sign-in, recovery, logout, the account routes, devices and the session
// lookup every session route starts with.
//
// The server holds SHA-256 of each 32-byte proof (auth key, recovery auth
// key, device proof) and of each session token, never the values. Credential
// updates are compare-and-swap: every statement of a batch that changes them
// carries the hash the Worker just checked, so of two concurrent changes one
// wins and the other finds nothing to change.

import { accountCollabStatements, afterAccountDeleted, memberships } from "./collab";
import { type PepperKeys, hashToken, hex, hmac16, newToken, proofMatches, randomHex, sha256Hex } from "./crypto";
import type { Env } from "./env";
import {
	accountDisabled,
	badRequest,
	constraint,
	error,
	invalidCredentials,
	json,
	notFound,
	readJson,
	refusal,
	type SignOutReason,
	unauthorized,
} from "./http";
import { UNKNOWN_LOGIN_KEY, accountKey, clientIp, count, ipGroup, ipKey, sessionFilter, windowStart } from "./limits";
import {
	type Checked,
	DAY,
	HOUR,
	KDF_DEFAULT,
	type Kdf,
	MAX_ACTIVE_DEVICES,
	MAX_AUTH_BODY,
	MAX_REVOKED_ROWS,
	type NewDevice,
	PROTOCOL_VERSION,
	SESSION_IDLE_MS,
	base64url,
	checkDevice,
	checkKdf,
	isAccountId,
	isDeviceId,
	isKeyId,
	isObject,
	isSalt,
	isSealedName,
	isTokenShape,
	isWrappedKey,
	proof32,
} from "./protocol";
import { DELETE_PURGE_ROUNDS, accountPrefix, purgePrefix } from "./purge";
import { accountShareStatements, runSharePurges } from "./shares";
import { healthAuth, healthEvents } from "./sign_in";
import { isReserved, normalizeUsername } from "./usernames";

type Handler = (request: Request, env: Env, now: number) => Promise<Response>;
type PepperHandler = (request: Request, env: Env, now: number, keys: PepperKeys) => Promise<Response>;

/** A session route's caller. */
export interface Session {
	tokenHash: string;
	accountId: string;
	deviceId: string;
	username: string;
}

const respond = (checked: Checked<unknown> & { ok: false }): Response => error(checked.status, checked.code, checked.message);

/** Reads an auth or account body (8 KiB) that must be a JSON object. */
async function readObject(request: Request): Promise<{ ok: true; value: Record<string, unknown> } | { ok: false; response: Response }> {
	const body = await readJson(request, MAX_AUTH_BODY);
	if (!body.ok) return body;
	if (!isObject(body.value)) return { ok: false, response: badRequest("The body must be a JSON object.") };
	return { ok: true, value: body.value };
}

/** The `username` field, normalized; a 400 when it isn't a string or isn't valid. */
function username(body: Record<string, unknown>): Checked<string> {
	if (typeof body.username !== "string") return { ok: false, status: 400, code: "bad_request", message: "username must be a string." };
	const normalized = normalizeUsername(body.username);
	if (normalized === null) return { ok: false, status: 400, code: "invalid_username", message: "Not a valid username." };
	return { ok: true, value: normalized };
}

const kdfJson = (m_kib: number, t: number, p: number) => ({ alg: KDF_DEFAULT.alg, version: KDF_DEFAULT.version, m_kib, t, p });

// ------------------------------------------------------------------ sessions

const SESSION_SQL = `SELECT s.device_id, s.last_seen_at, s.revoked_reason, a.id AS account_id, a.username, a.disabled_at
  FROM sessions s JOIN accounts a ON a.id = s.account_id WHERE s.token_hash = ?1`;

/**
 * The caller's session, or the response that ends the request: 401 with the
 * reason (a malformed header never touches D1), 403 for a disabled account
 * unless `allowDisabled`. Expires a session unseen for 180 days, and records
 * `last_seen_at` when it is more than an hour old.
 */
export async function authenticate(request: Request, env: Env, now: number, allowDisabled = false): Promise<Session | Response> {
	const match = /^Bearer (\S+)$/.exec(request.headers.get("authorization") ?? "");
	if (!match || !isTokenShape(match[1])) return unauthorized("unknown");
	const tokenHash = await hashToken(match[1]);
	const row = await env.DB.prepare(SESSION_SQL).bind(tokenHash).first<{
		device_id: string;
		last_seen_at: number;
		revoked_reason: SignOutReason | null;
		account_id: string;
		username: string;
		disabled_at: number | null;
	}>();
	if (!row) return unauthorized("unknown");
	if (row.revoked_reason !== null) return unauthorized(row.revoked_reason);
	if (now - row.last_seen_at > SESSION_IDLE_MS) {
		await env.DB.prepare("UPDATE sessions SET revoked_at = ?2, revoked_reason = 'expired' WHERE token_hash = ?1 AND revoked_at IS NULL")
			.bind(tokenHash, now)
			.run();
		return unauthorized("expired");
	}
	if (row.disabled_at !== null && !allowDisabled) return accountDisabled();
	if (now - row.last_seen_at > HOUR) {
		await env.DB.prepare("UPDATE sessions SET last_seen_at = ?2 WHERE token_hash = ?1 AND revoked_at IS NULL AND last_seen_at < ?2")
			.bind(tokenHash, now)
			.run();
	}
	return { tokenHash, accountId: row.account_id, deviceId: row.device_id, username: row.username };
}

/**
 * Wraps a session route: authenticates, passes SESSION_LIMITER (keyed by the
 * account), then runs `handler`.
 */
export function withSession(handler: (request: Request, env: Env, now: number, session: Session) => Promise<Response>): Handler {
	return async (request, env, now) => {
		const session = await authenticate(request, env, now);
		if (session instanceof Response) return session;
		return (await sessionFilter(env.SESSION_LIMITER, session.accountId)) ?? handler(request, env, now, session);
	};
}

type GuardColumn = "auth_hash" | "recovery_hash";

/**
 * The statements that make `device` this account's active session, in one
 * batch with the caller's own: revoke the least recently seen other session
 * when ten are active (`evicted`), upsert this one, and keep at most 90
 * revoked rows. Every statement holds only while the account is enabled and
 * each guard column still has the hash the caller checked.
 * Parameters: ?1 account, ?2 device, ?3 token hash, ?4 proof hash, ?5 now,
 * ?6 sealed name (null keeps the old one), ?7… guard values.
 */
function sessionStatements(
	db: D1Database,
	s: { accountId: string; deviceId: string; tokenHash: string; proofHash: string; sealedName: string | null; now: number },
	guards: [GuardColumn, string][],
): D1PreparedStatement[] {
	const guard = `EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND disabled_at IS NULL${guards
		.map(([column], i) => ` AND ${column} = ?${7 + i}`)
		.join("")})`;
	const params = [s.accountId, s.deviceId, s.tokenHash, s.proofHash, s.now, s.sealedName, ...guards.map(([, value]) => value)];
	const others = "account_id = ?1 AND revoked_at IS NULL AND device_id != ?2";
	return [
		db
			.prepare(
				`UPDATE sessions SET revoked_at = ?5, revoked_reason = 'evicted'
  WHERE account_id = ?1 AND revoked_at IS NULL
    AND device_id = (SELECT device_id FROM sessions WHERE ${others} ORDER BY last_seen_at, created_at, device_id LIMIT 1)
    AND (SELECT count(*) FROM sessions WHERE ${others}) >= ${MAX_ACTIVE_DEVICES}
    AND ${guard}
  RETURNING device_id, sealed_name`,
			)
			.bind(...params),
		db
			.prepare(
				`INSERT INTO sessions (account_id, device_id, token_hash, proof_hash, sealed_name, created_at, last_seen_at)
  SELECT ?1, ?2, ?3, ?4, ?6, ?5, ?5 WHERE ${guard}
  ON CONFLICT (account_id, device_id) DO UPDATE SET token_hash = excluded.token_hash, proof_hash = excluded.proof_hash,
    sealed_name = coalesce(excluded.sealed_name, sessions.sealed_name), created_at = excluded.created_at,
    last_seen_at = excluded.last_seen_at, revoked_at = NULL, revoked_reason = NULL
  RETURNING device_id`,
			)
			.bind(...params),
		db
			.prepare(
				`DELETE FROM sessions WHERE account_id = ?1 AND revoked_at IS NOT NULL AND device_id NOT IN (
  SELECT device_id FROM sessions WHERE account_id = ?1 AND revoked_at IS NOT NULL
  ORDER BY revoked_at DESC, device_id LIMIT ${MAX_REVOKED_ROWS})`,
			)
			.bind(s.accountId),
	];
}

interface SessionOutcome {
	ok: boolean;
	evicted: { id: string; sealed_name: string | null } | null;
}

/** Reads `sessionStatements`' results, starting at `offset` in the batch. */
function sessionOutcome(results: D1Result[], offset: number): SessionOutcome {
	const evicted = (results[offset].results as { device_id: string; sealed_name: string | null }[])[0];
	return {
		ok: results[offset + 1].results.length === 1,
		evicted: evicted ? { id: evicted.device_id, sealed_name: evicted.sealed_name } : null,
	};
}

interface NewSession {
	token: string;
	tokenHash: string;
	proofHash: string;
}

async function newSession(device: NewDevice): Promise<NewSession> {
	const token = newToken();
	return { token, tokenHash: await hashToken(token), proofHash: await sha256Hex(device.proof) };
}

// -------------------------------------------------------------------- health

export const health: Handler = async (_request, env) => {
	const row = await env.DB.prepare(
		`SELECT (SELECT value FROM settings WHERE name = 'signup') AS signup,
  (SELECT max_accounts FROM defaults) AS max_accounts, (SELECT count(*) FROM accounts) AS accounts`,
	).first<{ signup: string | null; max_accounts: number | null; accounts: number }>();
	let signup = "closed";
	if (row?.signup === "open") signup = row.accounts >= (row.max_accounts ?? 0) ? "full" : "open";
	return json({ ok: true, protocol: PROTOCOL_VERSION, signup, auth: healthAuth(env), events: healthEvents() });
};

// ------------------------------------------------------------------ prelogin

/** Unknown names get a deterministic fake salt and account ID from the pepper, and the default parameters. */
export const prelogin: PepperHandler = async (request, env, _now, keys) => {
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const name = username(body.value);
	if (!name.ok) return respond(name);
	const row = await env.DB.prepare("SELECT id, salt, kdf_m_kib, kdf_t, kdf_p FROM accounts WHERE username = ?1")
		.bind(name.value)
		.first<{ id: string; salt: string; kdf_m_kib: number; kdf_t: number; kdf_p: number }>();
	if (row) return json({ kdf: kdfJson(row.kdf_m_kib, row.kdf_t, row.kdf_p), salt: row.salt, account_id: row.id });
	const [salt, account] = await Promise.all([hmac16(keys.fake, "salt", name.value), hmac16(keys.fake, "account", name.value)]);
	return json({
		kdf: kdfJson(KDF_DEFAULT.m_kib, KDF_DEFAULT.t, KDF_DEFAULT.p),
		salt: base64url(salt),
		account_id: `a_${hex(account)}`,
	});
};

// ------------------------------------------------------------------- sign-up

interface SignupBody {
	username: string;
	accountId: string;
	kdf: Kdf;
	salt: string;
	authKey: Uint8Array;
	wrappedKey: string;
	recoveryAuthKey: Uint8Array;
	recoveryWrappedKey: string;
	keyId: string;
	device: NewDevice;
}

function checkSignup(body: Record<string, unknown>): Checked<SignupBody> {
	const name = username(body);
	if (!name.ok) return name;
	if (isReserved(name.value)) return { ok: false, status: 400, code: "reserved_username", message: "That username is reserved." };
	const bad = (message: string): Checked<never> => ({ ok: false, status: 400, code: "bad_request", message });
	const { account_id, salt, wrapped_key, recovery_wrapped_key, key_id } = body;
	if (typeof account_id !== "string" || !isAccountId(account_id)) return bad("account_id must be a_ and 32 hex digits.");
	const kdf = checkKdf(body.kdf);
	if (!kdf.ok) return kdf;
	if (typeof salt !== "string" || !isSalt(salt)) return bad("salt must be 22 base64url characters.");
	const authKey = proof32(body.auth_key);
	if (!authKey) return bad("auth_key must be 32 bytes of canonical base64url.");
	if (typeof wrapped_key !== "string" || !isWrappedKey(wrapped_key)) return bad("wrapped_key must be 98 base64url characters.");
	const recoveryAuthKey = proof32(body.recovery_auth_key);
	if (!recoveryAuthKey) return bad("recovery_auth_key must be 32 bytes of canonical base64url.");
	if (typeof recovery_wrapped_key !== "string" || !isWrappedKey(recovery_wrapped_key)) {
		return bad("recovery_wrapped_key must be 98 base64url characters.");
	}
	if (typeof key_id !== "string" || !isKeyId(key_id)) return bad("key_id must be 16 hex digits.");
	const device = checkDevice(body.device, true);
	if (!device.ok) return device;
	return {
		ok: true,
		value: {
			username: name.value,
			accountId: account_id,
			kdf: kdf.value,
			salt,
			authKey,
			wrappedKey: wrapped_key,
			recoveryAuthKey,
			recoveryWrappedKey: recovery_wrapped_key,
			keyId: key_id,
			device: device.value,
		},
	};
}

export const signup: PepperHandler = async (request, env, now, keys) => {
	const limited = await count(env.DB, "signup-ip", await ipKey(keys, "signup-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const checked = checkSignup(body.value);
	if (!checked.ok) return respond(checked);
	const b = checked.value;
	const session = await newSession(b.device);

	// A retry of a sign-up whose answer was lost: the same request again, so
	// the same account ID, KDF parameters, salt, auth key, wrapped keys,
	// recovery auth key and key ID. Matching the auth key alone would make
	// sign-up a password oracle outside the sign-in limits; a guesser knows
	// neither the wraps nor the recovery auth key. Null when it isn't a retry.
	const retried = async (): Promise<Response | null> => {
		const existing = await env.DB.prepare(
			`SELECT id, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash, recovery_wrapped_key, key_id,
  epoch, disabled_at FROM accounts WHERE username = ?1`,
		)
			.bind(b.username)
			.first<{
				id: string;
				kdf_m_kib: number;
				kdf_t: number;
				kdf_p: number;
				salt: string;
				auth_hash: string;
				wrapped_key: string;
				recovery_hash: string;
				recovery_wrapped_key: string;
				key_id: string;
				epoch: string;
				disabled_at: number | null;
			}>();
		if (!existing) return null;
		// Both hashes are always compared, so the answer's timing doesn't say which part differed.
		const authMatches = await proofMatches(b.authKey, existing.auth_hash);
		const recoveryMatches = await proofMatches(b.recoveryAuthKey, existing.recovery_hash);
		const same =
			authMatches &&
			recoveryMatches &&
			existing.id === b.accountId &&
			existing.kdf_m_kib === b.kdf.m_kib &&
			existing.kdf_t === b.kdf.t &&
			existing.kdf_p === b.kdf.p &&
			existing.salt === b.salt &&
			existing.wrapped_key === b.wrappedKey &&
			existing.recovery_wrapped_key === b.recoveryWrappedKey &&
			existing.key_id === b.keyId;
		if (!same) return null;
		if (existing.disabled_at !== null) return accountDisabled();
		const results = await env.DB.batch(
			sessionStatements(
				env.DB,
				{ accountId: existing.id, deviceId: b.device.id, ...session, sealedName: b.device.sealedName, now },
				[["auth_hash", existing.auth_hash]],
			),
		);
		if (!sessionOutcome(results, 0).ok) return error(409, "username_taken", "That username is taken.");
		return json({ account_id: existing.id, username: b.username, token: session.token, epoch: existing.epoch }, 201);
	};
	const retry = await retried();
	if (retry) return retry;

	const epoch = randomHex(8);
	try {
		await env.DB.batch([
			env.DB.prepare("DELETE FROM purges WHERE prefix = ?1").bind(accountPrefix(b.accountId)),
			env.DB.prepare(
				`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
  recovery_wrapped_key, key_id, epoch, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)`,
			).bind(
				b.accountId,
				b.username,
				b.kdf.m_kib,
				b.kdf.t,
				b.kdf.p,
				b.salt,
				await sha256Hex(b.authKey),
				b.wrappedKey,
				await sha256Hex(b.recoveryAuthKey),
				b.recoveryWrappedKey,
				b.keyId,
				epoch,
				now,
			),
			env.DB.prepare(
				`INSERT INTO sessions (account_id, device_id, token_hash, proof_hash, sealed_name, created_at, last_seen_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)`,
			).bind(b.accountId, b.device.id, session.tokenHash, session.proofHash, b.device.sealedName, now),
		]);
	} catch (err) {
		const refused = refusal(err);
		const unique = constraint(err, "UNIQUE");
		if (refused?.kind === "signup_closed" || unique) {
			// The same sign-up, sent again while the first was still running, lost the race to it
			// (the account now exists, or took the last place): answer as a retry.
			const retry = await retried();
			if (retry) return retry;
		}
		if (refused?.kind === "signup_closed") return error(403, "signup_closed", "Sign-ups are closed.", { reason: refused.detail });
		if (unique) {
			const taken = await env.DB.prepare("SELECT 1 AS taken FROM accounts WHERE id = ?1").bind(b.accountId).first();
			return taken
				? error(409, "account_id_taken", "That account ID is taken; generate another.")
				: error(409, "username_taken", "That username is taken.");
		}
		throw err;
	}
	return json({ account_id: b.accountId, username: b.username, token: session.token, epoch }, 201);
};

// ------------------------------------------------------------------- sign-in

interface AccountRow {
	id: string;
	username: string;
	auth_hash: string;
	wrapped_key: string;
	recovery_hash: string;
	recovery_wrapped_key: string;
	key_id: string;
	epoch: string;
	disabled_at: number | null;
}

const ACCOUNT_COLUMNS = "id, username, auth_hash, wrapped_key, recovery_hash, recovery_wrapped_key, key_id, epoch, disabled_at";
const ACCOUNT_BY_NAME = `SELECT ${ACCOUNT_COLUMNS} FROM accounts WHERE username = ?1`;
const ACCOUNT_BY_ID = `SELECT ${ACCOUNT_COLUMNS} FROM accounts WHERE id = ?1`;

export const login: PepperHandler = async (request, env, now, keys) => {
	let limited = await count(env.DB, "login-ip", await ipKey(keys, "login-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const name = username(body.value);
	if (!name.ok) return respond(name);
	const authKey = proof32(body.value.auth_key);
	if (!authKey) return badRequest("auth_key must be 32 bytes of canonical base64url.");
	const device = checkDevice(body.value.device, false);
	if (!device.ok) return respond(device);

	const account = await env.DB.prepare(
		`SELECT a.*, (SELECT s.proof_hash FROM sessions s WHERE s.account_id = a.id AND s.device_id = ?2
     AND s.revoked_at IS NULL) AS device_proof_hash
  FROM (${ACCOUNT_BY_NAME}) a`,
	)
		.bind(name.value, device.value.id)
		.first<AccountRow & { device_proof_hash: string | null }>();
	// A signed-in device, proving itself, doesn't count against the name. Once signed out, for
	// any reason (a password change, recovery, eviction, expiry, its own sign-out or another
	// Mac's), it is a stranger again: an old copy of its library holds the device secret, and the
	// owner can't remove a row the device list doesn't show.
	const known = account !== null && account.device_proof_hash !== null && (await proofMatches(device.value.proof, account.device_proof_hash));
	if (!known) {
		limited = await count(env.DB, "login-user", account ? accountKey("login-user", account.id) : UNKNOWN_LOGIN_KEY, now);
		if (limited) return limited;
	}
	if (!(await proofMatches(authKey, account?.auth_hash)) || !account) return invalidCredentials();
	if (account.disabled_at !== null) return accountDisabled();

	const session = await newSession(device.value);
	const results = await env.DB.batch(
		sessionStatements(env.DB, { accountId: account.id, deviceId: device.value.id, ...session, sealedName: null, now }, [
			["auth_hash", account.auth_hash],
		]),
	);
	const outcome = sessionOutcome(results, 0);
	// The password changed, or the account was disabled or deleted, since the check.
	if (!outcome.ok) return invalidCredentials();
	return json({
		account_id: account.id,
		username: account.username,
		token: session.token,
		wrapped_key: account.wrapped_key,
		key_id: account.key_id,
		epoch: account.epoch,
		evicted_device: outcome.evicted,
	});
};

// ------------------------------------------------------------------ recovery

export const recoverStart: PepperHandler = async (request, env, now, keys) => {
	const limited = await count(env.DB, "recover-ip", await ipKey(keys, "recover-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const name = username(body.value);
	if (!name.ok) return respond(name);
	const recoveryAuthKey = proof32(body.value.recovery_auth_key);
	if (!recoveryAuthKey) return badRequest("recovery_auth_key must be 32 bytes of canonical base64url.");
	const account = await env.DB.prepare(ACCOUNT_BY_NAME).bind(name.value).first<AccountRow>();
	if (!(await proofMatches(recoveryAuthKey, account?.recovery_hash)) || !account) return invalidCredentials();
	if (account.disabled_at !== null) return accountDisabled();
	return json({ account_id: account.id, recovery_wrapped_key: account.recovery_wrapped_key, key_id: account.key_id });
};

export const recoverFinish: PepperHandler = async (request, env, now, keys) => {
	const limited = await count(env.DB, "recover-ip", await ipKey(keys, "recover-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const name = username(b);
	if (!name.ok) return respond(name);
	const recoveryAuthKey = proof32(b.recovery_auth_key);
	if (!recoveryAuthKey) return badRequest("recovery_auth_key must be 32 bytes of canonical base64url.");
	const kdf = checkKdf(b.kdf);
	if (!kdf.ok) return respond(kdf);
	if (typeof b.salt !== "string" || !isSalt(b.salt)) return badRequest("salt must be 22 base64url characters.");
	const authKey = proof32(b.auth_key);
	if (!authKey) return badRequest("auth_key must be 32 bytes of canonical base64url.");
	if (typeof b.wrapped_key !== "string" || !isWrappedKey(b.wrapped_key)) return badRequest("wrapped_key must be 98 base64url characters.");
	const recoveryAuthKeyNew = proof32(b.recovery_auth_key_new);
	if (!recoveryAuthKeyNew) return badRequest("recovery_auth_key_new must be 32 bytes of canonical base64url.");
	if (typeof b.recovery_wrapped_key_new !== "string" || !isWrappedKey(b.recovery_wrapped_key_new)) {
		return badRequest("recovery_wrapped_key_new must be 98 base64url characters.");
	}
	const device = checkDevice(b.device, true);
	if (!device.ok) return respond(device);

	const authHash = await sha256Hex(authKey);
	const recoveryHashNew = await sha256Hex(recoveryAuthKeyNew);
	const session = await newSession(device.value);
	const s = { deviceId: device.value.id, ...session, sealedName: device.value.sealedName, now };

	let account = await env.DB.prepare(ACCOUNT_BY_NAME).bind(name.value).first<AccountRow>();
	if ((await proofMatches(recoveryAuthKey, account?.recovery_hash)) && account) {
		if (account.disabled_at !== null) return accountDisabled();
		const old = account.recovery_hash;
		// Every statement holds only while the old recovery hash is stored; the
		// account update, which replaces it, goes last.
		const results = await env.DB.batch([
			env.DB.prepare(
				`UPDATE sessions SET revoked_at = ?2, revoked_reason = 'recovered' WHERE account_id = ?1 AND revoked_at IS NULL
  AND EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND recovery_hash = ?3 AND disabled_at IS NULL)`,
			).bind(account.id, now, old),
			...sessionStatements(env.DB, { accountId: account.id, ...s }, [["recovery_hash", old]]),
			env.DB.prepare(
				`UPDATE accounts SET kdf_m_kib = ?3, kdf_t = ?4, kdf_p = ?5, salt = ?6, auth_hash = ?7, wrapped_key = ?8,
  recovery_hash = ?9, recovery_wrapped_key = ?10
  WHERE id = ?1 AND recovery_hash = ?2 AND disabled_at IS NULL RETURNING id`,
			).bind(
				account.id,
				old,
				kdf.value.m_kib,
				kdf.value.t,
				kdf.value.p,
				b.salt,
				authHash,
				b.wrapped_key,
				recoveryHashNew,
				b.recovery_wrapped_key_new,
			),
		]);
		if (results[4].results.length === 1) {
			return json({ account_id: account.id, username: account.username, token: session.token, epoch: account.epoch });
		}
		account = await env.DB.prepare(ACCOUNT_BY_NAME).bind(name.value).first<AccountRow>();
	}
	// A retry after a lost answer: the new recovery key and auth key are already in place.
	if (account && account.recovery_hash === recoveryHashNew && (await proofMatches(authKey, account.auth_hash))) {
		if (account.disabled_at !== null) return accountDisabled();
		const results = await env.DB.batch(
			sessionStatements(env.DB, { accountId: account.id, ...s }, [
				["recovery_hash", recoveryHashNew],
				["auth_hash", authHash],
			]),
		);
		if (sessionOutcome(results, 0).ok) {
			return json({ account_id: account.id, username: account.username, token: session.token, epoch: account.epoch });
		}
	}
	return invalidCredentials();
};

// -------------------------------------------------------------------- logout

/** Revokes this session (`signed_out`); 204 for disabled accounts too. */
export const logout: Handler = async (request, env, now) => {
	const session = await authenticate(request, env, now, true);
	if (session instanceof Response) return session;
	await revokeSelf(env, session, now);
	return new Response(null, { status: 204 });
};

function revokeSelf(env: Env, session: Session, now: number): Promise<D1Result> {
	return env.DB.prepare("UPDATE sessions SET revoked_at = ?2, revoked_reason = 'signed_out' WHERE token_hash = ?1 AND revoked_at IS NULL")
		.bind(session.tokenHash, now)
		.run();
}

// ------------------------------------------------------------------- account

export const account = withSession(async (_request, env, now, session) => {
	const row = await env.DB.prepare(
		`SELECT a.id, a.username, a.created_at, a.docs, a.doc_bytes, a.files, a.file_bytes,
  coalesce(a.quota_docs, d.docs, 0) AS quota_docs, coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota_doc_bytes,
  coalesce(a.quota_files, d.files, 0) AS quota_files, coalesce(a.quota_file_bytes, d.file_bytes, 0) AS quota_file_bytes,
  coalesce((SELECT CASE WHEN window_start = ?2 THEN count ELSE 0 END FROM limits WHERE key = 'writes-account:' || a.id), 0) AS writes_today,
  coalesce(a.quota_writes_per_day, (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'writes_per_day'), 0) AS writes_per_day
  FROM accounts a, defaults d WHERE a.id = ?1`,
	)
		.bind(session.accountId, windowStart(now, DAY))
		.first<Record<string, number | string>>();
	if (!row) return unauthorized("unknown");
	return json({
		account_id: row.id,
		username: row.username,
		created_at: row.created_at,
		usage: { docs: row.docs, doc_bytes: row.doc_bytes, files: row.files, file_bytes: row.file_bytes, writes_today: row.writes_today },
		quota: {
			docs: row.quota_docs,
			doc_bytes: row.quota_doc_bytes,
			files: row.quota_files,
			file_bytes: row.quota_file_bytes,
			writes_per_day: row.writes_per_day,
		},
	});
});

/**
 * Counts `password-account`, reads the body, then checks `field` (an auth
 * key) against the account's auth hash. The account row and the body, or
 * the response that ends the request.
 */
async function checkCurrentPassword(
	request: Request,
	env: Env,
	now: number,
	session: Session,
	field: string,
): Promise<{ ok: true; body: Record<string, unknown>; account: AccountRow; current: Uint8Array } | { ok: false; response: Response }> {
	const limited = await count(env.DB, "password-account", accountKey("password-account", session.accountId), now);
	if (limited) return { ok: false, response: limited };
	const body = await readObject(request);
	if (!body.ok) return body;
	const current = proof32(body.value[field]);
	if (!current) return { ok: false, response: badRequest(`${field} must be 32 bytes of canonical base64url.`) };
	const account = await env.DB.prepare(ACCOUNT_BY_ID).bind(session.accountId).first<AccountRow>();
	if (!account) return { ok: false, response: unauthorized("unknown") };
	return { ok: true, body: body.value, account, current };
}

/**
 * The `recovery_auth_key` and `recovery_wrapped_key` fields, which a new
 * recovery key and a password change both carry: the new recovery hash and
 * wrap, or the 400 that ends the request.
 */
async function newRecoveryKey(b: Record<string, unknown>): Promise<{ ok: true; hash: string; wrap: string } | { ok: false; response: Response }> {
	const recoveryAuthKey = proof32(b.recovery_auth_key);
	if (!recoveryAuthKey) return { ok: false, response: badRequest("recovery_auth_key must be 32 bytes of canonical base64url.") };
	if (typeof b.recovery_wrapped_key !== "string" || !isWrappedKey(b.recovery_wrapped_key)) {
		return { ok: false, response: badRequest("recovery_wrapped_key must be 98 base64url characters.") };
	}
	return { ok: true, hash: await sha256Hex(recoveryAuthKey), wrap: b.recovery_wrapped_key };
}

/**
 * Changes the password and replaces the recovery key in one compare-and-swap
 * update, so a password change always ends every older recovery key,
 * including one planted by someone who once knew the password.
 */
export const password = withSession(async (request, env, now, session) => {
	const checked = await checkCurrentPassword(request, env, now, session, "current_auth_key");
	if (!checked.ok) return checked.response;
	const { body: b, current } = checked;
	let account = checked.account;
	const kdf = checkKdf(b.kdf);
	if (!kdf.ok) return respond(kdf);
	if (typeof b.salt !== "string" || !isSalt(b.salt)) return badRequest("salt must be 22 base64url characters.");
	const authKey = proof32(b.auth_key);
	if (!authKey) return badRequest("auth_key must be 32 bytes of canonical base64url.");
	if (typeof b.wrapped_key !== "string" || !isWrappedKey(b.wrapped_key)) return badRequest("wrapped_key must be 98 base64url characters.");
	const recovery = await newRecoveryKey(b);
	if (!recovery.ok) return recovery.response;
	const authHash = await sha256Hex(authKey);

	if (await proofMatches(current, account.auth_hash)) {
		const results = await env.DB.batch([
			env.DB.prepare(
				`UPDATE sessions SET revoked_at = ?2, revoked_reason = 'password_changed'
  WHERE account_id = ?1 AND revoked_at IS NULL AND token_hash != ?3
    AND EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND auth_hash = ?4)
  RETURNING device_id`,
			).bind(account.id, now, session.tokenHash, account.auth_hash),
			env.DB.prepare(
				`UPDATE accounts SET kdf_m_kib = ?3, kdf_t = ?4, kdf_p = ?5, salt = ?6, auth_hash = ?7, wrapped_key = ?8,
  recovery_hash = ?9, recovery_wrapped_key = ?10
  WHERE id = ?1 AND auth_hash = ?2 RETURNING id`,
			).bind(
				account.id,
				account.auth_hash,
				kdf.value.m_kib,
				kdf.value.t,
				kdf.value.p,
				b.salt,
				authHash,
				b.wrapped_key,
				recovery.hash,
				recovery.wrap,
			),
		]);
		if (results[1].results.length === 1) return json({ signed_out_devices: results[0].results.length });
		account = (await env.DB.prepare(ACCOUNT_BY_ID).bind(session.accountId).first<AccountRow>()) ?? account;
	}
	// A retry after a lost answer: the new password material and recovery key are already stored.
	if (
		account.auth_hash === authHash &&
		account.wrapped_key === b.wrapped_key &&
		account.recovery_hash === recovery.hash &&
		account.recovery_wrapped_key === recovery.wrap
	) {
		return json({ signed_out_devices: 0 });
	}
	return invalidCredentials();
});

export const recoveryKey = withSession(async (request, env, now, session) => {
	const checked = await checkCurrentPassword(request, env, now, session, "current_auth_key");
	if (!checked.ok) return checked.response;
	const { body: b, account, current } = checked;
	const recovery = await newRecoveryKey(b);
	if (!recovery.ok) return recovery.response;
	if (!(await proofMatches(current, account.auth_hash))) return invalidCredentials();
	// Setting the same values again (a retry) changes the row just the same.
	const updated = await env.DB.prepare(
		"UPDATE accounts SET recovery_hash = ?3, recovery_wrapped_key = ?4 WHERE id = ?1 AND auth_hash = ?2 RETURNING id",
	)
		.bind(account.id, account.auth_hash, recovery.hash, recovery.wrap)
		.first();
	return updated ? json({}) : invalidCredentials();
});

/**
 * The deletion batch: documents, files, published notes (queued for their R2
 * objects' deletion), shared notes (its own stopped and their rooms queued for
 * purging; its identity, memberships, wraps, requests and notifications gone),
 * sessions, account-keyed limits, the purge row, then the account (children
 * first: D1 enforces foreign keys).
 * Every statement holds only while the checked auth hash is stored.
 */
export function deletionStatements(db: D1Database, accountId: string, now: number, authHash: string): D1PreparedStatement[] {
	const guard = "EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND auth_hash = ?2)";
	const bind = (sql: string) => db.prepare(sql).bind(accountId, authHash);
	return [
		bind(`DELETE FROM account_docs WHERE account_id = ?1 AND ${guard}`),
		bind(`DELETE FROM account_files WHERE account_id = ?1 AND ${guard}`),
		...accountShareStatements(db, accountId, guard, authHash, now),
		...accountCollabStatements(db, accountId, guard, authHash, now),
		bind(`DELETE FROM device_approvals WHERE account_id = ?1 AND ${guard}`),
		bind(`DELETE FROM account_passkeys WHERE account_id = ?1 AND ${guard}`),
		bind(`DELETE FROM account_identities WHERE account_id = ?1 AND ${guard}`),
		bind(`DELETE FROM auth_flows WHERE account_id = ?1 AND ${guard}`),
		bind(`DELETE FROM sessions WHERE account_id = ?1 AND ${guard}`),
		bind(
			`DELETE FROM limits WHERE key IN ('login-user:' || ?1, 'password-account:' || ?1, 'writes-account:' || ?1, 'uploads-account:' || ?1,
  'lookup-account:' || ?1, 'join-account:' || ?1, 'notify-account:' || ?1,
  'approval-create:' || ?1, 'approval-poll:' || ?1, 'approval-respond:' || ?1)
  AND ${guard}`,
		),
		db
			.prepare(
				`INSERT INTO purges (prefix, requested_at) SELECT 'accounts/' || ?1 || '/', ?3 WHERE ${guard}
  ON CONFLICT (prefix) DO UPDATE SET requested_at = excluded.requested_at`,
			)
			.bind(accountId, authHash, now),
		bind(`DELETE FROM accounts WHERE id = ?1 AND ${guard} RETURNING id`),
	];
}

export const deleteAccount = withSession(async (request, env, now, session) => {
	const checked = await checkCurrentPassword(request, env, now, session, "auth_key");
	if (!checked.ok) return checked.response;
	const { body: b, account, current } = checked;
	if (typeof b.username !== "string") return badRequest("username must be a string.");
	if (!(await proofMatches(current, account.auth_hash))) return invalidCredentials();
	if (normalizeUsername(b.username) !== account.username) {
		return error(400, "username_mismatch", "The username doesn't match this account.");
	}
	const shared = await memberships(env, account.id);
	const results = await env.DB.batch(deletionStatements(env.DB, account.id, now, account.auth_hash));
	if (results[results.length - 1].results.length !== 1) return invalidCredentials();
	await purgePrefix(env, accountPrefix(account.id), now, now, DELETE_PURGE_ROUNDS);
	await runSharePurges(env, now);
	await afterAccountDeleted(env, account.id, shared, now);
	return json({ deleted: true });
});

// ------------------------------------------------------------------- devices

export const setDevice = withSession(async (request, env, _now, session) => {
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const name = body.value.sealed_name;
	if (typeof name !== "string" || !isSealedName(name)) return badRequest("sealed_name must be 55–400 base64url characters.");
	await env.DB.prepare("UPDATE sessions SET sealed_name = ?2 WHERE token_hash = ?1 AND revoked_at IS NULL").bind(session.tokenHash, name).run();
	return new Response(null, { status: 204 });
});

export const devices = withSession(async (_request, env, _now, session) => {
	const { results } = await env.DB.prepare(
		`SELECT device_id, sealed_name, created_at, last_seen_at FROM sessions WHERE account_id = ?1 AND revoked_at IS NULL
  ORDER BY last_seen_at DESC, created_at DESC, device_id`,
	)
		.bind(session.accountId)
		.all<{ device_id: string; sealed_name: string | null; created_at: number; last_seen_at: number }>();
	return json({
		devices: results.map((r) => ({
			id: r.device_id,
			sealed_name: r.sealed_name,
			created_at: r.created_at,
			last_seen_at: r.last_seen_at,
			current: r.device_id === session.deviceId,
		})),
	});
});

/**
 * Signs out one of the account's devices; the current one is a logout.
 * Scoped to the caller's account. A row already signed out, for any reason,
 * is marked `revoked` too (keeping its time), so the owner can always say
 * "that Mac is gone"; 404 only for a device the account has no row for.
 */
export function revokeDevice(deviceId: string): Handler {
	return withSession(async (_request, env, now, session) => {
		if (!isDeviceId(deviceId)) return badRequest("Not a device ID.");
		if (deviceId === session.deviceId) {
			await revokeSelf(env, session, now);
			return new Response(null, { status: 204 });
		}
		const revoked = await env.DB.prepare(
			`UPDATE sessions SET revoked_at = coalesce(revoked_at, ?3), revoked_reason = 'revoked'
  WHERE account_id = ?1 AND device_id = ?2 RETURNING device_id`,
		)
			.bind(session.accountId, deviceId, now)
			.first();
		return revoked ? new Response(null, { status: 204 }) : notFound();
	});
}
