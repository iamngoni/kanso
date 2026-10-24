// Device approval (design: Kanso accounts spec, revision 3 › Unlocking a new
// device › The protocol, and › Sign-in routes › Approvals) and the account
// event socket that tells an approver about it (› GET /v2/account/events).
//
// A new device N that signed in without the data key asks another signed-in
// device A to send it, end-to-end encrypted. The Worker only relays: it stores
// N's commitment to its ephemeral X25519 key, A's ephemeral key, N's revealed
// key with its sealed name, and A's sealed data key, each once. It holds no
// private key, so it can compute neither the shared secret nor the six-digit
// code both screens show, and it can't open the name or the data key.
//
// The order is what makes the short code safe: N commits before it sees A's
// key, A posts its key before N reveals its own, and the Worker refuses a
// revealed key that doesn't match the commitment (A checks again, since it
// doesn't trust the Worker).
//
//   POST   /v2/account/sign-in/approvals                 N asks; {id, expires_at, other_devices}
//   GET    /v2/account/sign-in/approvals                 the pending requests of other devices
//   GET    /v2/account/sign-in/approvals/<id>            N's poll
//   POST   /v2/account/sign-in/approvals/<id>/challenge  A's ephemeral key
//   POST   /v2/account/sign-in/approvals/<id>/name       N's key and sealed device name
//   POST   /v2/account/sign-in/approvals/<id>/approve    A's sealed data key
//   DELETE /v2/account/sign-in/approvals/<id>            cancel or decline
//   GET    /v2/account/events                            the live hints (WebSocket)
//
// A request lives five minutes and is answered once. At most three are
// pending per account; a fourth cancels the oldest. The account's room
// (account_room.ts) hears `approval.created`, `approval.named` and
// `approval.cancelled` after each D1 write commits; a room that can't be
// reached never fails the route.

import { type AccountEvent, ACCOUNT_ROOM_HEADERS, accountRoom, publish } from "./account_room";
import { type Session, authenticate, withSession } from "./auth";
import { proofMatches, sha256 } from "./crypto";
import type { Env } from "./env";
import { badRequest, error, invalidCredentials, json, notFound, readJson } from "./http";
import { accountKey, count, sessionFilter } from "./limits";
import { MAX_AUTH_BODY, base64url, fromBase64url, isObject, isSealedName, isWrappedKey, proof32 } from "./protocol";
import { isFlowId } from "./sign_in";

type Handler = (request: Request, env: Env, now: number) => Promise<Response>;

/** How long a request lives. */
export const APPROVAL_TTL_MS = 5 * 60_000;
/** Pending requests per account; a new one past this cancels the oldest. */
export const MAX_PENDING_APPROVALS = 3;

const COMMIT_LABEL = new TextEncoder().encode("kanso/v1/approve-commit");

/** `SHA-256("kanso/v1/approve-commit" || 0x01 || epk)` as 43 base64url characters. */
export async function approvalCommit(epk: Uint8Array): Promise<string> {
	const input = new Uint8Array(COMMIT_LABEL.length + 1 + epk.length);
	input.set(COMMIT_LABEL, 0);
	input[COMMIT_LABEL.length] = 0x01;
	input.set(epk, COMMIT_LABEL.length + 1);
	return base64url(await sha256(input));
}

/** One `device_approvals` row. */
export interface ApprovalRow {
	id: string;
	account_id: string;
	device_id: string;
	commit_hash: string;
	epk: string | null;
	approver_device_id: string | null;
	approver_epk: string | null;
	sealed_name: string | null;
	wrapped_key: string | null;
	created_at: number;
	expires_at: number;
	completed_at: number | null;
	cancelled_at: number | null;
}

export type ApprovalStatus = "waiting" | "challenged" | "approved" | "cancelled" | "expired";

/** A row's status as of `now`: cancelled, then expired, then approved, challenged or waiting. */
export function approvalStatus(row: ApprovalRow, now: number): ApprovalStatus {
	if (row.cancelled_at !== null) return "cancelled";
	if (row.expires_at <= now) return "expired";
	if (row.completed_at !== null) return "approved";
	if (row.approver_epk !== null) return "challenged";
	return "waiting";
}

const approvalExpired = (): Response => error(409, "approval_expired", "This approval request has expired.");
const approvalUsed = (): Response => error(409, "approval_used", "This approval request was already answered.");
const selfApproval = (): Response => error(403, "self_approval", "A device can't approve its own request.");

/** A 32-byte value as exactly 43 canonical base64url characters (a commitment or an ephemeral key). */
const key32 = (value: unknown): value is string => proof32(value) !== null;

/** Reads an approval body (8 KiB) that must be a JSON object. */
async function readObject(request: Request): Promise<{ ok: true; value: Record<string, unknown> } | { ok: false; response: Response }> {
	const body = await readJson(request, MAX_AUTH_BODY);
	if (!body.ok) return body;
	if (!isObject(body.value)) return { ok: false, response: badRequest("The body must be a JSON object.") };
	return { ok: true, value: body.value };
}

const ROW_COLUMNS = `id, account_id, device_id, commit_hash, epk, approver_device_id, approver_epk, sealed_name, wrapped_key,
  created_at, expires_at, completed_at, cancelled_at`;

/** The account's request `id`, or null; another account's is never read. */
function rowOf(env: Env, accountId: string, id: string): Promise<ApprovalRow | null> {
	return env.DB.prepare(`SELECT ${ROW_COLUMNS} FROM device_approvals WHERE id = ?1 AND account_id = ?2`).bind(id, accountId).first<ApprovalRow>();
}

/** Counts `rule` for the session's account first (H1); the 429 response, or null. */
const counted = (env: Env, rule: "approval-create" | "approval-poll" | "approval-respond", session: Session, now: number) =>
	count(env.DB, rule, accountKey(rule, session.accountId), now);

// ------------------------------------------------------------------- create

/**
 * `POST /v2/account/sign-in/approvals` `{commit}`: 200 `{id, expires_at,
 * other_devices}`. `other_devices` counts the account's active sessions other
 * than the caller's (0 means the app offers the recovery key at once). A
 * fourth pending request cancels the oldest. Counts `approval-create`.
 */
export const create = withSession(async (request, env, now, session) => {
	const limited = await counted(env, "approval-create", session, now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const commit = body.value.commit;
	if (!key32(commit)) return badRequest("commit must be 32 bytes of canonical base64url.");
	const id = base64url(crypto.getRandomValues(new Uint8Array(32)));
	const expiresAt = now + APPROVAL_TTL_MS;
	// Insert, then cancel every active request outside the newest two others. The
	// insert and the cancellation run in one D1 batch, so two concurrent creates
	// cannot both observe the same count and leave four rows pending. The new row
	// is excluded from cancellation; ties on created_at are settled by ID.
	const results = await env.DB.batch([
		env.DB.prepare(
			`INSERT INTO device_approvals (id, account_id, device_id, commit_hash, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)`,
		).bind(id, session.accountId, session.deviceId, commit, now, expiresAt),
		env.DB.prepare(
			`UPDATE device_approvals SET cancelled_at = ?2
  WHERE account_id = ?1 AND cancelled_at IS NULL AND completed_at IS NULL AND expires_at > ?2 AND id != ?3
    AND id NOT IN (
      SELECT id FROM device_approvals
      WHERE account_id = ?1 AND cancelled_at IS NULL AND completed_at IS NULL AND expires_at > ?2 AND id != ?3
      ORDER BY created_at DESC, id DESC LIMIT ?4
    )
  RETURNING id`,
		).bind(session.accountId, now, id, MAX_PENDING_APPROVALS - 1),
		env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE account_id = ?1 AND revoked_at IS NULL AND device_id != ?2").bind(
			session.accountId,
			session.deviceId,
		),
	]);
	const others = (results[2].results[0] as { n: number } | undefined)?.n ?? 0;
	for (const row of results[1].results as { id: string }[]) {
		await publish(env, session.accountId, "approval.cancelled", row.id);
	}
	await publish(env, session.accountId, "approval.created", id);
	return json({ id, expires_at: expiresAt, other_devices: others });
});

// --------------------------------------------------------------------- read

/**
 * `GET /v2/account/sign-in/approvals`: 200 `{approvals: [...]}`, the
 * account's pending requests (not cancelled, answered or expired) from other
 * devices, oldest first. Not counted: an approver runs it after an event, a
 * launch, a Settings open or a reconnect, never on a timer.
 */
export const list = withSession(async (_request, env, now, session) => {
	const { results } = await env.DB.prepare(
		`SELECT ${ROW_COLUMNS} FROM device_approvals
  WHERE account_id = ?1 AND device_id != ?2 AND cancelled_at IS NULL AND completed_at IS NULL AND expires_at > ?3
  ORDER BY created_at, id`,
	)
		.bind(session.accountId, session.deviceId, now)
		.all<ApprovalRow>();
	return json({
		approvals: results.map((row) => ({
			id: row.id,
			device_id: row.device_id,
			commit: row.commit_hash,
			epk: row.epk,
			created_at: row.created_at,
			expires_at: row.expires_at,
			approver_epk: row.approver_epk,
			approver_device_id: row.approver_device_id,
			sealed_name: row.sealed_name,
			status: approvalStatus(row, now),
		})),
	});
});

/**
 * `GET /v2/account/sign-in/approvals/<id>`, the requester's poll: 200
 * `{status, expires_at, approver_epk?, approver_device_id?, sealed_name?,
 * wrapped_key?}`. Only the requesting device reads it (404 for any other, and
 * for another account's request). A cancelled or expired request carries
 * nothing else; an approved one can be read again until it expires or N
 * deletes it, so a lost answer is retried by polling. Counts `approval-poll`.
 */
export function poll(id: string): Handler {
	return withSession(async (_request, env, now, session) => {
		if (!isFlowId(id)) return badRequest("Not an approval ID.");
		const limited = await counted(env, "approval-poll", session, now);
		if (limited) return limited;
		const row = await rowOf(env, session.accountId, id);
		if (!row || row.device_id !== session.deviceId) return notFound();
		const status = approvalStatus(row, now);
		const answer: Record<string, unknown> = { status, expires_at: row.expires_at };
		if (status === "challenged" || status === "approved") {
			answer.approver_epk = row.approver_epk;
			answer.approver_device_id = row.approver_device_id;
			if (row.sealed_name !== null) answer.sealed_name = row.sealed_name;
			if (row.wrapped_key !== null) answer.wrapped_key = row.wrapped_key;
		}
		return json(answer);
	});
}

// ------------------------------------------------------------------ answers

/** The common start of the three answers: the ID, the count, the body and the row, or the response that ends the request. */
async function answerStart(
	id: string,
	request: Request,
	env: Env,
	now: number,
	session: Session,
): Promise<{ row: ApprovalRow; body: Record<string, unknown> } | Response> {
	if (!isFlowId(id)) return badRequest("Not an approval ID.");
	const limited = await counted(env, "approval-respond", session, now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const row = await rowOf(env, session.accountId, id);
	if (!row || row.cancelled_at !== null) return notFound();
	return { row, body: body.value };
}

/**
 * `POST …/<id>/challenge` `{epk}`: 200 `{ok: true}`. The approver's
 * ephemeral key, stored once with its device. The same device with the same
 * key again is 200; another device or key is 409 `approval_used`. The
 * requester can't challenge its own request (403 `self_approval`).
 */
export function challenge(id: string): Handler {
	return withSession(async (request, env, now, session) => {
		const start = await answerStart(id, request, env, now, session);
		if (start instanceof Response) return start;
		const { row, body } = start;
		if (!key32(body.epk)) return badRequest("epk must be 32 bytes of canonical base64url.");
		if (row.device_id === session.deviceId) return selfApproval();
		if (row.expires_at <= now) return approvalExpired();
		const stored = await env.DB.prepare(
			`UPDATE device_approvals SET approver_device_id = ?3, approver_epk = ?4
  WHERE id = ?1 AND account_id = ?2 AND approver_epk IS NULL AND cancelled_at IS NULL AND expires_at > ?5 RETURNING id`,
		)
			.bind(id, session.accountId, session.deviceId, body.epk, now)
			.first();
		if (stored) return json({ ok: true });
		return settled(env, session, id, now, (again) => again.approver_device_id === session.deviceId && again.approver_epk === body.epk);
	});
}

/**
 * `POST …/<id>/name` `{epk, sealed_name}`: 200 `{ok: true}`. Only the
 * requesting device (404 for others), only after a challenge (400 before),
 * and only a key matching its commitment (400 `bad_request`). Both are
 * stored once, then `approval.named` is published; the identical pair again
 * is 200, a different one 409 `approval_used`.
 */
export function name(id: string): Handler {
	return withSession(async (request, env, now, session) => {
		const start = await answerStart(id, request, env, now, session);
		if (start instanceof Response) return start;
		const { row, body } = start;
		if (row.device_id !== session.deviceId) return notFound();
		const epk = proof32(body.epk);
		if (!epk) return badRequest("epk must be 32 bytes of canonical base64url.");
		const sealedName = body.sealed_name;
		if (typeof sealedName !== "string" || !isSealedName(sealedName)) return badRequest("sealed_name must be 55–400 base64url characters.");
		if (row.expires_at <= now) return approvalExpired();
		if (row.approver_epk === null) return badRequest("The request hasn't been challenged yet.");
		if ((await approvalCommit(epk)) !== row.commit_hash) return badRequest("epk doesn't match the request's commitment.");
		const stored = await env.DB.prepare(
			`UPDATE device_approvals SET epk = ?3, sealed_name = ?4
  WHERE id = ?1 AND account_id = ?2 AND epk IS NULL AND approver_epk IS NOT NULL AND cancelled_at IS NULL AND expires_at > ?5 RETURNING id`,
		)
			.bind(id, session.accountId, body.epk, sealedName, now)
			.first();
		if (stored) {
			await publish(env, session.accountId, "approval.named", id);
			return json({ ok: true });
		}
		return settled(env, session, id, now, (again) => again.epk === body.epk && again.sealed_name === sealedName);
	});
}

/**
 * `POST …/<id>/approve` `{wrapped_key}`: 200 `{ok: true}`. Only the device
 * that challenged, after the requester named itself (400 before). The
 * requester gets 403 `self_approval`; another device 409 `approval_used`;
 * the same wrapped key again is 200 (a retry), a different one 409.
 */
export function approve(id: string): Handler {
	return withSession(async (request, env, now, session) => {
		const start = await answerStart(id, request, env, now, session);
		if (start instanceof Response) return start;
		const { row, body } = start;
		const wrapped = body.wrapped_key;
		if (typeof wrapped !== "string" || !isWrappedKey(wrapped) || !fromBase64url(wrapped)) {
			return badRequest("wrapped_key must be 98 base64url characters.");
		}
		if (row.device_id === session.deviceId) return selfApproval();
		if (row.expires_at <= now) return approvalExpired();
		if (row.approver_device_id !== session.deviceId) return approvalUsed();
		if (row.epk === null) return badRequest("The requesting device hasn't named itself yet.");
		const stored = await env.DB.prepare(
			`UPDATE device_approvals SET wrapped_key = ?4, completed_at = ?5
  WHERE id = ?1 AND account_id = ?2 AND approver_device_id = ?3 AND wrapped_key IS NULL AND epk IS NOT NULL
    AND cancelled_at IS NULL AND expires_at > ?5 RETURNING id`,
		)
			.bind(id, session.accountId, session.deviceId, wrapped, now)
			.first();
		if (stored) return json({ ok: true });
		return settled(env, session, id, now, (again) => again.approver_device_id === session.deviceId && again.wrapped_key === wrapped);
	});
}

/**
 * An answer that changed nothing: the row was answered meanwhile (or before).
 * 200 when `same` says this request is a retry of what is stored, else the
 * row's state decides: gone or cancelled 404, expired 409
 * `approval_expired`, otherwise 409 `approval_used`.
 */
async function settled(env: Env, session: Session, id: string, now: number, same: (row: ApprovalRow) => boolean): Promise<Response> {
	const again = await rowOf(env, session.accountId, id);
	if (!again || again.cancelled_at !== null) return notFound();
	if (again.expires_at <= now) return approvalExpired();
	return same(again) ? json({ ok: true }) : approvalUsed();
}

// ------------------------------------------------------------------- cancel

/**
 * `DELETE /v2/account/sign-in/approvals/<id>`: 204, by any device of the
 * account (the requester cancels, an approver declines). It publishes
 * `approval.cancelled`, and the requester's poll says `cancelled`. A
 * request already cancelled is 204 again. An approved one is deleted by
 * its requester once it holds the key; anyone else gets 409 `approval_used`.
 */
export function cancel(id: string): Handler {
	return withSession(async (_request, env, now, session) => {
		if (!isFlowId(id)) return badRequest("Not an approval ID.");
		const row = await rowOf(env, session.accountId, id);
		if (!row) return notFound();
		// The requester, a device that could still challenge, or the one that
		// challenged may cancel or decline; once answered, only its requester
		// may delete it (below).
		if (row.device_id !== session.deviceId && row.approver_device_id !== null && row.approver_device_id !== session.deviceId) {
			return notFound();
		}
		if (row.cancelled_at !== null) return new Response(null, { status: 204 });
		if (row.completed_at !== null) {
			if (row.device_id !== session.deviceId) return approvalUsed();
			await env.DB.prepare("DELETE FROM device_approvals WHERE id = ?1 AND account_id = ?2").bind(id, session.accountId).run();
			return new Response(null, { status: 204 });
		}
		const cancelled = await env.DB.prepare(
			"UPDATE device_approvals SET cancelled_at = ?3 WHERE id = ?1 AND account_id = ?2 AND cancelled_at IS NULL AND completed_at IS NULL RETURNING id",
		)
			.bind(id, session.accountId, now)
			.first();
		if (!cancelled) {
			// Approved between the read and the update: only its requester may delete it.
			const again = await rowOf(env, session.accountId, id);
			if (again?.completed_at != null && again.device_id !== session.deviceId) return approvalUsed();
			if (again?.completed_at != null) {
				await env.DB.prepare("DELETE FROM device_approvals WHERE id = ?1 AND account_id = ?2").bind(id, session.accountId).run();
			}
			return new Response(null, { status: 204 });
		}
		await publish(env, session.accountId, "approval.cancelled", id);
		return new Response(null, { status: 204 });
	});
}

// ------------------------------------------------------------------- events

/** The upgrade's device headers. */
export const DEVICE_HEADER = "x-kanso-device";
export const DEVICE_PROOF_HEADER = "x-kanso-device-proof";

/**
 * `GET /v2/account/events`: the account's live approval hints, as a
 * WebSocket. Authenticated once, at the upgrade: the session token (and
 * SESSION_LIMITER, one token per upgrade), then `X-Kanso-Device` must be the
 * session's device and SHA-256 of `X-Kanso-Device-Proof` must equal the
 * session row's `proof_hash` (constant time). A missing or wrong device or
 * proof is 401 `invalid_credentials` and no socket is accepted; a stolen
 * token alone can't listen. Frames never touch D1 or a rate limit.
 */
export const events: Handler = async (request, env, now) => {
	const session = await authenticate(request, env, now);
	if (session instanceof Response) return session;
	const limited = await sessionFilter(env.SESSION_LIMITER, session.accountId);
	if (limited) return limited;
	const proof = proof32(request.headers.get(DEVICE_PROOF_HEADER));
	const row = await env.DB.prepare("SELECT proof_hash FROM sessions WHERE token_hash = ?1 AND revoked_at IS NULL")
		.bind(session.tokenHash)
		.first<{ proof_hash: string | null }>();
	// The proof is hashed and compared whatever else is wrong, so every refusal costs the same.
	const proofOk = await proofMatches(proof ?? new Uint8Array(32), row?.proof_hash);
	if (!proof || !proofOk || request.headers.get(DEVICE_HEADER) !== session.deviceId) return invalidCredentials();
	if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
		return error(426, "upgrade_required", "Open this route as a WebSocket.", {}, { upgrade: "websocket" });
	}
	return accountRoom(env, session.accountId).fetch(
		new Request("https://account-room/events", {
			headers: {
				upgrade: "websocket",
				[ACCOUNT_ROOM_HEADERS.account]: session.accountId,
				[ACCOUNT_ROOM_HEADERS.device]: session.deviceId,
			},
		}),
	);
};

export type { AccountEvent };
