// One-time pairing codes (issue #245): a signed-in device starts the request;
// a headless device uses the code and then follows the existing end-to-end
// approval protocol. The Worker stores no pairing code or data key in clear.

import { type Session, authenticate, closeEvicted, withSession } from "./auth";
import { publish } from "./account_room";
import { type PepperKeys, hmac16, hex, pepperKeys, proofMatches, randomBytes, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { badRequest, error, invalidCredentials, json, notFound, readJson } from "./http";
import { accountKey, clientIp, count, ipGroup, ipKey, sessionFilter } from "./limits";
import { type NewDevice, MAX_AUTH_BODY, base64url, checkDevice, isObject, proof32 } from "./protocol";

type Handler = (request: Request, env: Env, now: number) => Promise<Response>;
type PepperHandler = (request: Request, env: Env, now: number, keys: PepperKeys) => Promise<Response>;

export const PAIRING_TTL_MS = 10 * 60_000;
const MAX_LIVE_CODES = 3;
const MAX_PENDING_APPROVALS = 3;
const APPROVAL_TTL_MS = 5 * 60_000;
const ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const BAD_CODE = () => error(400, "bad_code", "That pairing code has expired or was already used.");

export type PairingState = "waiting" | "used" | "expired";

/** Current state with consumption taking precedence over expiry. */
export function pairingState(expiresAt: number, usedAt: number | null, now: number): PairingState {
	if (usedAt !== null) return "used";
	return expiresAt <= now ? "expired" : "waiting";
}

/** Canonical Crockford form; visually ambiguous characters map to their usual digits. */
export function normalizePairingCode(input: unknown): string | null {
	if (typeof input !== "string") return null;
	const normalized = input.toUpperCase().replaceAll("-", "").replaceAll("O", "0").replace(/[IL]/g, "1");
	return /^[0-9A-HJKMNP-TV-Z]{8}$/.test(normalized) ? normalized : null;
}

/** Eight Crockford symbols with a separator for copying between devices. */
export function formatPairingCode(bytes: Uint8Array): string {
	if (bytes.length < 5) throw new RangeError("pairing code needs 40 random bits");
	let value = "";
	let buffer = 0;
	let bits = 0;
	for (let i = 0; i < 5; i++) {
		buffer = (buffer << 8) | bytes[i];
		bits += 8;
		while (bits >= 5) {
			bits -= 5;
			value += ALPHABET[(buffer >> bits) & 31];
		}
		buffer &= (1 << bits) - 1;
	}
	return `${value.slice(0, 4)}-${value.slice(4)}`;
}

async function digestCode(keys: PepperKeys, code: string): Promise<string> {
	return hex(await hmac16(keys.limit, "pair-code", code));
}

async function readBody(request: Request): Promise<{ ok: true; value: Record<string, unknown> } | { ok: false; response: Response }> {
	const body = await readJson(request, MAX_AUTH_BODY);
	if (!body.ok) return body;
	if (!isObject(body.value)) return { ok: false, response: badRequest("The body must be a JSON object.") };
	return { ok: true, value: body.value };
}

async function sessionWithDevice(request: Request, env: Env, now: number): Promise<Session | Response> {
	const session = await authenticate(request, env, now);
	if (session instanceof Response) return session;
	const limited = await sessionFilter(env.SESSION_LIMITER, session.accountId);
	if (limited) return limited;
	const proof = proof32(request.headers.get("x-kanso-device-proof"));
	const proofOk = await proofMatches(proof ?? new Uint8Array(32), session.proofHash);
	if (!proof || !proofOk || request.headers.get("x-kanso-device") !== session.deviceId) return invalidCredentials();
	return session;
}

/** `POST /v2/account/pairings`: issue a code only to a proof-bearing session. */
export const create: Handler = async (request, env, now) => {
	const session = await sessionWithDevice(request, env, now);
	if (session instanceof Response) return session;
	const body = await readBody(request);
	if (!body.ok) return body.response;
	if (Object.keys(body.value).length !== 0) return badRequest("The body must be an empty object.");
	const keysPromise = pepperKeys(env.AUTH_PEPPER);
	const keys = keysPromise ? await keysPromise : null;
	if (!keys) return error(503, "misconfigured", "The server's authorization pepper isn't configured.");
	const code = formatPairingCode(randomBytes(5));
	const id = base64url(randomBytes(32));
	const createdAt = now;
	const expiresAt = now + PAIRING_TTL_MS;
	const hash = await digestCode(keys, code.replace("-", ""));
	await env.DB.batch([
		env.DB.prepare(
			`INSERT INTO device_pairings (id, account_id, code_hash, created_by_device, created_at, expires_at)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6)`,
		).bind(id, session.accountId, hash, session.deviceId, createdAt, expiresAt),
		env.DB.prepare(
			`DELETE FROM device_pairings WHERE account_id = ?1 AND id NOT IN (
			 SELECT id FROM device_pairings WHERE account_id = ?1 AND used_at IS NULL AND expires_at > ?2
			 ORDER BY created_at DESC, id DESC LIMIT ?3)`,
		).bind(session.accountId, now, MAX_LIVE_CODES),
	]);
	return json({ id, code, expires_at: expiresAt });
};

interface PairingRow {
	id: string;
	account_id: string;
	created_by_device: string;
	expires_at: number;
	used_at: number | null;
	username: string;
	key_id: string;
	epoch: string;
	has_password: number;
	disabled_at: number | null;
}

/** `GET /v2/account/pairings/<id>` only reports the requesting device's code. */
export function status(id: string): Handler {
	return withSession(async (_request, env, now, session) => {
		if (!/^[A-Za-z0-9_-]{43}$/.test(id)) return notFound();
		const row = await env.DB.prepare(
			"SELECT id, expires_at, used_at FROM device_pairings WHERE id = ?1 AND account_id = ?2 AND created_by_device = ?3",
		).bind(id, session.accountId, session.deviceId).first<{ id: string; expires_at: number; used_at: number | null }>();
		if (!row) return notFound();
		return json({ status: pairingState(row.expires_at, row.used_at, now), expires_at: row.expires_at });
	});
}

/** `DELETE /v2/account/pairings/<id>` cancels an unused code. */
export function cancel(id: string): Handler {
	return withSession(async (_request, env, _now, session) => {
		if (!/^[A-Za-z0-9_-]{43}$/.test(id)) return notFound();
		await env.DB.prepare(
			"DELETE FROM device_pairings WHERE id = ?1 AND account_id = ?2 AND created_by_device = ?3 AND used_at IS NULL",
		).bind(id, session.accountId, session.deviceId).run();
		return new Response(null, { status: 204 });
	});
}

/** `POST /v2/auth/pair`: consume a code and make a locked session for approval. */
export const pair: PepperHandler = async (request, env, now, keys) => {
	const group = ipGroup(clientIp(request));
	let limited = await count(env.DB, "pair-ip", await ipKey(keys, "pair-ip", group), now);
	if (limited) return limited;
	limited = await count(env.DB, "login-ip", await ipKey(keys, "login-ip", group), now);
	if (limited) return limited;
	const body = await readBody(request);
	if (!body.ok) return body.response;
	const normalized = normalizePairingCode(body.value.code);
	const checked = checkDevice(body.value.device, false);
	const requestId = body.value.request_id;
	const commit = body.value.commit;
	if (!checked.ok || typeof requestId !== "string" || !/^[A-Za-z0-9_-]{43}$/.test(requestId) || typeof commit !== "string" || !/^[A-Za-z0-9_-]{43}$/.test(commit)) {
		return BAD_CODE();
	}
	const hash = await digestCode(keys, normalized ?? "00000000");
	const pairing = await env.DB.prepare(
		`SELECT p.id, p.account_id, p.created_by_device, p.expires_at, p.used_at,
		 a.username, a.key_id, a.epoch, a.has_password, a.disabled_at
		 FROM device_pairings p JOIN accounts a ON a.id = p.account_id WHERE p.code_hash = ?1`,
	).bind(hash).first<PairingRow>();
	if (!normalized || !pairing || pairing.used_at !== null || pairing.expires_at <= now || pairing.disabled_at !== null) return BAD_CODE();
	const device: NewDevice = checked.value;
	const token = `kanso_${base64url(randomBytes(32))}`;
	const tokenHash = await sha256Hex(token);
	const proofHash = await sha256Hex(device.proof);
	const approvalExpires = now + APPROVAL_TTL_MS;
	const limitKey = accountKey("approval-create", pairing.account_id);
	const limitedApproval = await count(env.DB, "approval-create", limitKey, now);
	if (limitedApproval) return limitedApproval;
	const results = await env.DB.batch([
		env.DB.prepare(
			`UPDATE device_pairings SET used_at = ?2, used_by_device = ?3
			 WHERE code_hash = ?1 AND used_at IS NULL AND expires_at > ?2 RETURNING id`,
		).bind(hash, now, device.id),
		env.DB.prepare(
			`INSERT INTO sessions (account_id, device_id, token_hash, proof_hash, sealed_name, created_at, last_seen_at)
			 SELECT ?1, ?2, ?3, ?4, NULL, ?5, ?5 WHERE changes() > 0 AND EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND disabled_at IS NULL)
			 ON CONFLICT (account_id, device_id) DO UPDATE SET token_hash = excluded.token_hash, proof_hash = excluded.proof_hash,
			 sealed_name = coalesce(excluded.sealed_name, sessions.sealed_name), created_at = excluded.created_at,
			 last_seen_at = excluded.last_seen_at, revoked_at = NULL, revoked_reason = NULL RETURNING device_id`,
		).bind(pairing.account_id, device.id, tokenHash, proofHash, now),
		env.DB.prepare(
			`UPDATE sessions SET revoked_at = ?3, revoked_reason = 'evicted'
			 WHERE changes() > 0 AND account_id = ?1 AND revoked_at IS NULL AND device_id = (
			   SELECT device_id FROM sessions WHERE account_id = ?1 AND revoked_at IS NULL AND device_id != ?2
			   ORDER BY last_seen_at, created_at, device_id LIMIT 1)
			 AND (SELECT count(*) FROM sessions WHERE account_id = ?1 AND revoked_at IS NULL AND device_id != ?2) >= 10
			 RETURNING device_id, sealed_name`,
		).bind(pairing.account_id, device.id, now),
		env.DB.prepare(
			`INSERT INTO device_approvals (id, account_id, device_id, commit_hash, created_at, expires_at)
			 SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE EXISTS (SELECT 1 FROM sessions WHERE account_id = ?2 AND device_id = ?3 AND token_hash = ?7 AND revoked_at IS NULL)
			 ON CONFLICT (id) DO NOTHING RETURNING id`,
		).bind(requestId, pairing.account_id, device.id, commit, now, approvalExpires, tokenHash),
		env.DB.prepare(
			`UPDATE device_approvals SET cancelled_at = ?2 WHERE changes() > 0 AND account_id = ?1 AND cancelled_at IS NULL AND completed_at IS NULL AND expires_at > ?2 AND id != ?3
			 AND id NOT IN (SELECT id FROM device_approvals WHERE account_id = ?1 AND cancelled_at IS NULL AND completed_at IS NULL AND expires_at > ?2 AND id != ?3 ORDER BY created_at DESC, id DESC LIMIT ?4)
			 RETURNING id`,
		).bind(pairing.account_id, now, requestId, MAX_PENDING_APPROVALS - 1),
	]);
	if (results[0].results.length !== 1 || results[1].results.length !== 1 || results[3].results.length !== 1) return BAD_CODE();
	const evicted = (results[2].results[0] as { device_id: string; sealed_name: string | null } | undefined);
	if (evicted) await closeEvicted(env, pairing.account_id, { ok: true, evicted: { id: evicted.device_id, sealed_name: evicted.sealed_name } });
	await publish(env, pairing.account_id, "approval.created", requestId);
	const otherDevices = await env.DB.prepare(
		"SELECT count(*) AS n FROM sessions WHERE account_id = ?1 AND revoked_at IS NULL AND device_id != ?2",
	).bind(pairing.account_id, device.id).first<{ n: number }>();
	return json({
		kind: "session",
		account_id: pairing.account_id,
		username: pairing.username,
		token,
		key_id: pairing.key_id,
		epoch: pairing.epoch,
		has_password: pairing.has_password === 1,
		passkey_wrapped_key: null,
		evicted_device: evicted ? { id: evicted.device_id, sealed_name: evicted.sealed_name } : null,
		approval_id: requestId,
		expires_at: approvalExpires,
		other_devices: otherDevices?.n ?? 0,
	});
};
