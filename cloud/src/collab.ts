// Shared notes (docs/ACCOUNTS.md › Collaboration › The server: routes): every
// session route under /v2/collab/, the public invite page `GET /j/<link_id>`,
// and the parts of an account deletion and of the cron that concern sharing.
//
// D1 holds who may do what: the share, its owner, epoch and signed member list
// (collab_shares), every epoch's signed list (collab_lists), each member's
// role and status (collab_members), their wrapped note keys per epoch
// (collab_wraps), invite links, requests to join
// and notifications. The note itself lives in its NoteRoom (room.ts), which
// the Worker calls only after it has authenticated the session, read the
// caller's role and the epoch from D1, and checked the envelope's header:
// format, kind for the route, epoch, author, role and size. The server never
// holds a note key, so it never opens an envelope.
//
// Errors follow the v2 conventions: 404 `not_found` for a share the caller
// isn't in (whether or not it exists; for reading and writing the note, an
// invitation not yet accepted counts as not in), 403 `forbidden_role` for one
// they are in but whose role doesn't allow the request, 409
// `epoch_changed` `{epoch}`, 413 `quota_exceeded` `{quota, used, limit}` for
// the caps (members, shares, links, requests, doc_bytes).
//
// A share ID that stops being shared is retired for good (collab_retired,
// filled by a trigger), and its room is purged: right away when possible,
// else by the cron from collab_purges.

import { type Session, withSession } from "./auth";
import { proofMatches, randomBytes } from "./crypto";
import { type Header, type Kind, MAX_ENVELOPE, MAX_LOG_LIMIT, type Role, mayWrite, parseEnvelope } from "./envelope";
import type { Env } from "./env";
import { badRequest, constraint, error, json, notFound, rateLimited, readBody, readJson, refusal, tooLarge } from "./http";
import { identityOf, isB64 } from "./identity";
import { BINDING_PERIOD_MS, accountKey, clientIp, count, ipGroup } from "./limits";
import { DAY, base64url, fromBase64url, isAccountId, isObject, isSha256Hex, natParam, proof32 } from "./protocol";
import { wakeShare } from "./push";
import { type NoteRoom, ROOM_HEADERS, type RoomRefusal } from "./room";
import { isShareId } from "./shares";
import { PUBLIC_HEADERS } from "./viewer";

export type Handler = (request: Request, env: Env, now: number) => Promise<Response>;
export type SessionHandler = (request: Request, env: Env, now: number, session: Session) => Promise<Response>;

/** withSession, looked up when a request arrives (auth.ts imports this module for account deletion). */
export const route =
	(handler: SessionHandler): Handler =>
	(request, env, now) =>
		withSession(handler)(request, env, now);

export const MAX_MEMBERS = 20;
export const MAX_OWNED_SHARES = 200;
export const MAX_LIVE_LINKS = 10;
export const MAX_PENDING_REQUESTS = 50;
export const MAX_NOTIFICATIONS = 500;
/** A link lives at most this long. */
export const MAX_LINK_MS = 30 * DAY;
/** The signed member list, as base64url. */
export const MAX_LIST_CHARS = 16_384;
/** A sealed notification: at most 4 KiB, at least the wrap's 0x01, ephemeral key, nonce and tag. */
export const MIN_NOTICE_BYTES = 1 + 32 + 24 + 16;
export const MAX_NOTICE_BYTES = 4096;
/** JSON bodies; sharing a note carries its first snapshot (up to 4 MiB, as base64url). */
const MAX_COLLAB_BODY = 64 * 1024;
const MAX_SHARE_BODY = 6 * 1024 * 1024;
/** Rooms one cron run (or one account deletion) purges. */
export const COLLAB_PURGE_BATCH = 100;

const ROLES: readonly Role[] = ["owner", "edit", "comment", "view"];
export const LINK_ROLES: readonly Role[] = ["edit", "comment", "view"];

export const roomOf = (env: Env, shareId: string): DurableObjectStub<NoteRoom> => env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(shareId));

/** Binds `values` up to the highest `?N` the statement uses (D1 refuses extra bindings). */
export function bound(db: D1Database, sql: string, values: unknown[]): D1PreparedStatement {
	const highest = Math.max(0, ...[...sql.matchAll(/\?(\d+)/g)].map((m) => Number(m[1])));
	return db.prepare(sql).bind(...values.slice(0, highest));
}

export const noContent = (): Response => new Response(null, { status: 204 });
export const forbiddenRole = (): Response => error(403, "forbidden_role", "Your role in this note doesn't allow that.");
export const identityRequired = (): Response => error(403, "identity_required", "Set up this account's identity keys first.");

// ------------------------------------------------------------------ the list

export interface ListMember {
	account: string;
	username: string;
	role: Role;
	sign_pub: string;
	box_pub: string;
}

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false });

export interface CheckedList {
	members: ListMember[];
	/** The log's head when the epoch began: clients drop entries of the epoch before it written after this. Null for epoch 1. */
	fromSeq: number | null;
}

/**
 * The members of a signed list the owner sent, after checking what the server
 * relies on: canonical base64url of UTF-8 JSON, `v` 1, this share, this epoch,
 * this owner, a key commitment, `from_seq` (an integer ≥ 0 from epoch 2 on,
 * absent at epoch 1), members sorted by account ID without duplicates, each
 * with a known role and key shapes, and exactly one `owner`, the owner
 * themself. The signature is the clients' to check.
 */
export function checkList(list: unknown, shareId: string, epoch: number, owner: string): CheckedList | string {
	if (typeof list !== "string" || list.length === 0 || list.length > MAX_LIST_CHARS) return `list must be 1–${MAX_LIST_CHARS} base64url characters.`;
	const bytes = fromBase64url(list);
	if (!bytes) return "list must be canonical base64url.";
	let value: unknown;
	try {
		value = JSON.parse(utf8.decode(bytes));
	} catch {
		return "list must be UTF-8 JSON.";
	}
	if (!isObject(value)) return "list must be a JSON object.";
	if (value.v !== 1 || value.share !== shareId || value.epoch !== epoch || value.owner !== owner) {
		return "list must have v 1, this share, this epoch and you as its owner.";
	}
	if (typeof value.key_commit !== "string" || !isSha256Hex(value.key_commit)) return "list.key_commit must be 64 lowercase hex digits.";
	if (!Number.isSafeInteger(value.at)) return "list.at must be an integer.";
	let fromSeq: number | null = null;
	if (epoch === 1) {
		if (Object.hasOwn(value, "from_seq")) return "list.from_seq is only for epochs after the first.";
	} else {
		if (!Number.isSafeInteger(value.from_seq) || (value.from_seq as number) < 0) return "list.from_seq must be the log's head, an integer ≥ 0, from epoch 2 on.";
		fromSeq = value.from_seq as number;
	}
	if (!Array.isArray(value.members) || value.members.length === 0) return "list.members must be a non-empty array.";
	const members: ListMember[] = [];
	for (const m of value.members) {
		if (!isObject(m)) return "Each member must be an object.";
		const { account, username, role, sign_pub, box_pub } = m;
		if (typeof account !== "string" || !isAccountId(account)) return "A member's account isn't an account ID.";
		if (typeof username !== "string" || username.length === 0 || username.length > 32) return "A member's username must be a string.";
		if (typeof role !== "string" || !(ROLES as readonly string[]).includes(role)) return "A member's role must be owner, edit, comment or view.";
		if (!isB64(sign_pub, 32) || !isB64(box_pub, 32)) return "A member's keys must be 43 base64url characters.";
		const previous = members[members.length - 1];
		if (previous && previous.account >= account) return "list.members must be sorted by account ID, each once.";
		members.push({ account, username, role: role as Role, sign_pub, box_pub });
	}
	const owners = members.filter((m) => m.role === "owner");
	if (owners.length !== 1 || owners[0].account !== owner) return "list.members must have exactly one owner: you.";
	return { members, fromSeq };
}

/** The most wraps one new epoch may carry: 20 per epoch so far, and never more than 400. */
export const maxWraps = (epoch: number): number => Math.min(MAX_MEMBERS * epoch, 400);

/**
 * `[{account, wrapped, epoch?}]`: exactly one 105-byte wrap per listed member
 * for `epoch` (an absent `epoch` means it), and, only on a new epoch, wraps of
 * earlier epochs (1 ≤ n < `epoch`) for listed accounts, so a new member can
 * read the comments written before they joined. Each (account, epoch) once, at
 * most `maxWraps(epoch)` in all. Null when it isn't so.
 */
function checkWraps(wraps: unknown, members: ListMember[], epoch: number, earlier: boolean): { account: string; wrapped: string; epoch: number }[] | null {
	if (!Array.isArray(wraps) || wraps.length > maxWraps(epoch)) return null;
	const listed = new Set(members.map((m) => m.account));
	const seen = new Set<string>();
	const out: { account: string; wrapped: string; epoch: number }[] = [];
	for (const w of wraps) {
		if (!isObject(w) || typeof w.account !== "string" || !listed.has(w.account) || !isB64(w.wrapped, 105)) return null;
		const at = w.epoch === undefined ? epoch : w.epoch;
		if (!Number.isSafeInteger(at) || (at as number) < 1 || (at as number) > epoch || (!earlier && at !== epoch)) return null;
		const key = `${w.account}/${at}`;
		if (seen.has(key)) return null;
		seen.add(key);
		out.push({ account: w.account, wrapped: w.wrapped as string, epoch: at as number });
	}
	const current = out.filter((w) => w.epoch === epoch).length;
	return current === members.length ? out : null;
}

/**
 * Checks every listed member against what the server knows of the account:
 * 409 `unknown_member` `{account}` for the first one without an identity
 * (unknown, deleted, or never set up), then 409 `member_keys_mismatch`
 * `{account}` for the first whose `sign_pub`, `box_pub` or `username` isn't
 * the account's own. Otherwise anyone could list someone else's account under
 * their own keys and have members pin them. Null when all match.
 */
export async function checkMemberKeys(env: Env, members: ListMember[], missingCode: (account: string) => string = () => "unknown_member"): Promise<Response | null> {
	const { results } = await env.DB.prepare(
		`SELECT i.account_id, i.sign_pub, i.box_pub, a.username FROM identities i JOIN accounts a ON a.id = i.account_id
  WHERE i.account_id IN (SELECT value FROM json_each(?1))`,
	)
		.bind(JSON.stringify(members.map((m) => m.account)))
		.all<{ account_id: string; sign_pub: string; box_pub: string; username: string }>();
	const known = new Map(results.map((r) => [r.account_id, r]));
	const missing = members.find((m) => !known.has(m.account));
	if (missing) {
		return error(409, missingCode(missing.account), "A listed account has no identity keys (unknown, deleted, or not set up).", { account: missing.account });
	}
	const wrong = members.find((m) => {
		const k = known.get(m.account)!;
		return k.sign_pub !== m.sign_pub || k.box_pub !== m.box_pub || k.username !== m.username;
	});
	return wrong
		? error(409, "member_keys_mismatch", "A listed member's keys or username aren't that account's.", { account: wrong.account })
		: null;
}

/** 413 `quota_exceeded` with the cap's usage, after a trigger (or a check) refused. */
async function quotaExceeded(env: Env, quota: string, ownerId: string, shareId: string, now: number): Promise<Response> {
	const figure = async (sql: string, ...values: unknown[]): Promise<number> =>
		(await bound(env.DB, sql, values).first<{ n: number }>())?.n ?? 0;
	switch (quota) {
		case "shares":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM collab_shares WHERE owner_id = ?1", ownerId),
				limit: MAX_OWNED_SHARES,
			});
		case "members":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM collab_members WHERE share_id = ?1", shareId),
				limit: MAX_MEMBERS,
			});
		case "links":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM collab_links WHERE share_id = ?1 AND expires_at > ?2", shareId, now),
				limit: MAX_LIVE_LINKS,
			});
		case "requests":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM collab_requests WHERE share_id = ?1", shareId),
				limit: MAX_PENDING_REQUESTS,
			});
		default: {
			const row = await env.DB.prepare(
				`SELECT a.doc_bytes + coalesce((SELECT sum(bytes) FROM collab_shares WHERE owner_id = a.id), 0) AS used,
  coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota FROM accounts a, defaults d WHERE a.id = ?1`,
			)
				.bind(ownerId)
				.first<{ used: number; quota: number }>();
			return error(413, "quota_exceeded", "Over quota.", { quota: "doc_bytes", used: row?.used ?? 0, limit: row?.quota ?? 0 });
		}
	}
}

/** Runs a batch; a trigger's `quota_exceeded:<kind>` becomes the 413 answer. */
async function guarded(env: Env, statements: D1PreparedStatement[], ownerId: string, shareId: string, now: number): Promise<D1Result[] | Response> {
	try {
		return await env.DB.batch(statements);
	} catch (err) {
		const r = refusal(err);
		if (r?.kind === "quota_exceeded") return quotaExceeded(env, r.detail, ownerId, shareId, now);
		if (constraint(err, "CHECK")) return badRequest("A value isn't storable.");
		throw err;
	}
}

// ------------------------------------------------------------- who is asking

interface Standing {
	ownerId: string;
	epoch: number;
	listSig: string;
	role: Role | null;
	status: "invited" | "member" | null;
}

/** The share and the caller's place in it; null when the share doesn't exist. */
async function standing(env: Env, shareId: string, accountId: string): Promise<Standing | null> {
	const row = await env.DB.prepare(
		`SELECT s.owner_id, s.epoch, s.list_sig, m.role, m.status FROM collab_shares s
  LEFT JOIN collab_members m ON m.share_id = s.id AND m.account_id = ?2 WHERE s.id = ?1`,
	)
		.bind(shareId, accountId)
		.first<{ owner_id: string; epoch: number; list_sig: string; role: Role | null; status: "invited" | "member" | null }>();
	return row ? { ownerId: row.owner_id, epoch: row.epoch, listSig: row.list_sig, role: row.role, status: row.status } : null;
}

/** The share for an owner's route, or the answer: 400 for a bad ID, 404 for a non-member, 403 for a member who isn't the owner. */
async function asOwner(env: Env, shareId: string, session: Session): Promise<Standing | Response> {
	if (!isShareId(shareId)) return badRequest("Not a share ID.");
	const s = await standing(env, shareId, session.accountId);
	if (!s || s.role === null) return notFound();
	if (s.ownerId !== session.accountId) return forbiddenRole();
	return s;
}

/** The share for a member's route: 404 for a non-member and for an invitation not yet accepted alike. */
async function asMember(env: Env, shareId: string, session: Session): Promise<Standing | Response> {
	if (!isShareId(shareId)) return badRequest("Not a share ID.");
	const s = await standing(env, shareId, session.accountId);
	if (!s || s.role === null || s.status !== "member") return notFound();
	return s;
}

const roomRefusal = (r: RoomRefusal): Response => {
	const extras = r.extras ?? {};
	const headers: Record<string, string> = typeof extras.retry_after === "number" ? { "retry-after": String(extras.retry_after) } : {};
	return error(r.status, r.error, `Refused by the room: ${r.error}.`, extras, headers);
};

/**
 * The header of an envelope sent to `kinds`, checked as the spec lists:
 * format, kind for the route, epoch (409 `epoch_changed`), author and device
 * (403 `forbidden_author`), role (403 `forbidden_role`), size (413).
 */
function checkEnvelope(bytes: Uint8Array, kinds: readonly Kind[], s: Standing, session: Session): Header | Response {
	const header = parseEnvelope(bytes);
	if (!header) return badRequest("Not an envelope (0x01, kind, epoch, author, device, counter, nonce, sealed, signature).");
	if (!kinds.includes(header.kind)) return badRequest(`This route takes ${kinds.join(" or ")} envelopes.`);
	if (header.epoch !== s.epoch) return error(409, "epoch_changed", "The note has a newer epoch.", { epoch: s.epoch });
	if (header.author !== session.accountId || header.device !== session.deviceId) {
		return error(403, "forbidden_author", "The envelope's author and device must be this session's.");
	}
	if (!s.role || !mayWrite(s.role, header.kind)) return forbiddenRole();
	if (bytes.length > MAX_ENVELOPE[header.kind]) return tooLarge(`A ${header.kind} envelope is at most ${MAX_ENVELOPE[header.kind]} bytes.`);
	return header;
}

// -------------------------------------------------------------------- shares

/**
 * `POST /v2/collab/shares`: shares a note at epoch 1 with its first snapshot
 * (base_seq 0; the log is empty, head 0). The owner is a member at once; everyone else listed is
 * invited. 201 `{share_id, epoch, head}`. Sending the same share again (a lost
 * answer) answers 201 again; an ID in use is 409 `share_id_taken`, a retired
 * one 409 `share_retired`.
 */
export const createShare = route(async (request, env, now, session) => {
	const body = await readJson(request, MAX_SHARE_BODY);
	if (!body.ok) return body.response;
	const b = body.value;
	if (!isObject(b)) return badRequest("The body must be a JSON object.");
	const shareId = b.share_id;
	if (typeof shareId !== "string" || !isShareId(shareId)) return badRequest("share_id must be 16 bytes of base64url (22 characters).");
	if (!(await identityOf(env, session.accountId))) return identityRequired();
	const checked = checkList(b.list, shareId, 1, session.accountId);
	if (typeof checked === "string") return badRequest(checked);
	const members = checked.members;
	if (members.length > MAX_MEMBERS) return error(413, "quota_exceeded", "Over quota.", { quota: "members", used: members.length, limit: MAX_MEMBERS });
	if (!isB64(b.list_sig, 64)) return badRequest("list_sig must be 86 base64url characters.");
	const wraps = checkWraps(b.wraps, members, 1, false);
	if (!wraps) return badRequest("wraps must hold one 140-character wrap for each listed account.");
	const snapshot = typeof b.snapshot === "string" ? fromBase64url(b.snapshot) : null;
	if (!snapshot) return badRequest("snapshot must be an envelope in base64url.");
	const s: Standing = { ownerId: session.accountId, epoch: 1, listSig: b.list_sig as string, role: "owner", status: "member" };
	const header = checkEnvelope(snapshot, ["s"], s, session);
	if (header instanceof Response) return header;
	const mismatch = await checkMemberKeys(env, members);
	if (mismatch) return mismatch;

	const values = [shareId, session.accountId, b.list_sig, b.list, now, snapshot.length, JSON.stringify(members), JSON.stringify(wraps)];
	const mine = "EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1 AND owner_id = ?2 AND epoch = 1 AND list_sig = ?3)";
	const results = await guarded(
		env,
		[
			bound(
				env.DB,
				`INSERT INTO collab_shares (id, owner_id, epoch, list, list_sig, bytes, created_at, updated_at)
  SELECT ?1, ?2, 1, ?4, ?3, ?6, ?5, ?5 WHERE NOT EXISTS (SELECT 1 FROM collab_retired WHERE id = ?1)
  ON CONFLICT (id) DO NOTHING RETURNING id`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO collab_members (share_id, account_id, role, status, invited_at, joined_at)
  SELECT ?1, json_extract(value, '$.account'), json_extract(value, '$.role'),
    CASE WHEN json_extract(value, '$.account') = ?2 THEN 'member' ELSE 'invited' END, ?5,
    CASE WHEN json_extract(value, '$.account') = ?2 THEN ?5 END
  FROM json_each(?7) WHERE ${mine}
  ON CONFLICT (share_id, account_id) DO NOTHING`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO collab_wraps (share_id, account_id, epoch, wrapped)
  SELECT ?1, json_extract(value, '$.account'), 1, json_extract(value, '$.wrapped') FROM json_each(?8) WHERE ${mine}
  ON CONFLICT (share_id, account_id, epoch) DO NOTHING`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO collab_lists (share_id, epoch, list, list_sig) SELECT ?1, 1, ?4, ?3 WHERE ${mine}
  ON CONFLICT (share_id, epoch) DO NOTHING`,
				values,
			),
		],
		session.accountId,
		shareId,
		now,
	);
	if (results instanceof Response) return results;
	if (results[0].results.length !== 1) {
		const existing = await env.DB.prepare("SELECT owner_id, epoch, list_sig FROM collab_shares WHERE id = ?1")
			.bind(shareId)
			.first<{ owner_id: string; epoch: number; list_sig: string }>();
		const retry = existing && existing.owner_id === session.accountId && existing.epoch === 1 && existing.list_sig === b.list_sig;
		if (!retry) {
			const retired = await env.DB.prepare("SELECT 1 AS retired FROM collab_retired WHERE id = ?1").bind(shareId).first();
			return retired
				? error(409, "share_retired", "That share ID stopped being shared and can't be used again; share under a new one.")
				: error(409, "share_id_taken", "That share ID can't be used; share under a new one.");
		}
	}
	const room = await roomOf(env, shareId).init({ shareId, epoch: 1, author: session.accountId, envelope: snapshot, now });
	return json({ share_id: shareId, epoch: 1, head: room.head }, 201);
});

interface ShareRow {
	id: string;
	owner_id: string;
	owner_username: string;
	epoch: number;
	list: string;
	list_sig: string;
	role: Role;
	status: string;
}

/**
 * `GET /v2/collab/shares`: every share the account is in or invited to, with
 * its own wraps (every epoch it has one for) and the room's head; for the
 * account's own shares also every member's status, the live links and the
 * pending requests (with the requester's public keys, so the owner can check
 * them before approving).
 */
export const listShares = route(async (_request, env, now, session) => {
	const owned = "share_id IN (SELECT id FROM collab_shares WHERE owner_id = ?1)";
	const [shares, wraps, lists, members, links, requests] = await env.DB.batch([
		env.DB.prepare(
			`SELECT s.id, s.owner_id, o.username AS owner_username, s.epoch, s.list, s.list_sig, m.role, m.status
  FROM collab_members m JOIN collab_shares s ON s.id = m.share_id JOIN accounts o ON o.id = s.owner_id
  WHERE m.account_id = ?1 ORDER BY s.created_at, s.id`,
		).bind(session.accountId),
		env.DB.prepare("SELECT share_id, epoch, wrapped FROM collab_wraps WHERE account_id = ?1 ORDER BY share_id, epoch").bind(session.accountId),
		// Each epoch's signed list the caller may need: those it holds a key for, and the current one.
		env.DB.prepare(
			`SELECT l.share_id, l.epoch, l.list, l.list_sig FROM collab_lists l
  JOIN collab_members m ON m.share_id = l.share_id AND m.account_id = ?1
  WHERE l.epoch = (SELECT s.epoch FROM collab_shares s WHERE s.id = l.share_id)
     OR EXISTS (SELECT 1 FROM collab_wraps w WHERE w.share_id = l.share_id AND w.account_id = ?1 AND w.epoch = l.epoch)
  ORDER BY l.share_id, l.epoch`,
		).bind(session.accountId),
		env.DB.prepare(
			`SELECT share_id, account_id, status FROM collab_members WHERE ${owned}
  UNION ALL SELECT share_id, account_id, 'left' FROM collab_left WHERE ${owned} ORDER BY share_id, account_id`,
		).bind(session.accountId),
		env.DB.prepare(`SELECT id, share_id, role, created_at, expires_at FROM collab_links WHERE ${owned} AND expires_at > ?2 ORDER BY share_id, created_at, id`).bind(
			session.accountId,
			now,
		),
		env.DB.prepare(
			`SELECT r.share_id, r.account_id, a.username, r.link_id, r.role, r.requested_at, i.sign_pub, i.box_pub, i.binding
  FROM collab_requests r JOIN accounts a ON a.id = r.account_id JOIN identities i ON i.account_id = r.account_id
  WHERE r.${owned} ORDER BY r.share_id, r.requested_at, r.account_id`,
		).bind(session.accountId),
	]);
	const group = <T extends { share_id: string }>(rows: T[]): Map<string, T[]> => {
		const map = new Map<string, T[]>();
		for (const row of rows) map.set(row.share_id, [...(map.get(row.share_id) ?? []), row]);
		return map;
	};
	const wrapsBy = group(wraps.results as { share_id: string; epoch: number; wrapped: string }[]);
	const listsBy = group(lists.results as { share_id: string; epoch: number; list: string; list_sig: string }[]);
	const membersBy = group(members.results as { share_id: string; account_id: string; status: string }[]);
	const linksBy = group(links.results as { id: string; share_id: string; role: string; created_at: number; expires_at: number }[]);
	const requestsBy = group(
		requests.results as {
			share_id: string;
			account_id: string;
			username: string;
			link_id: string;
			role: string;
			requested_at: number;
			sign_pub: string;
			box_pub: string;
			binding: string;
		}[],
	);
	const rows = shares.results as unknown as ShareRow[];
	const heads = await Promise.all(rows.map((r) => roomOf(env, r.id).state()));
	return json({
		shares: rows.map((r, i) => {
			const share: Record<string, unknown> = {
				share_id: r.id,
				owner: r.owner_id,
				owner_username: r.owner_username,
				role: r.role,
				status: r.status,
				epoch: r.epoch,
				list: r.list,
				list_sig: r.list_sig,
				wraps: (wrapsBy.get(r.id) ?? []).map((w) => ({ epoch: w.epoch, wrapped: w.wrapped })),
				lists: (listsBy.get(r.id) ?? []).map((l) => ({ epoch: l.epoch, list: l.list, list_sig: l.list_sig })),
				head: heads[i].head,
			};
			if (r.owner_id === session.accountId) {
				share.members = (membersBy.get(r.id) ?? []).map((m) => ({ account: m.account_id, status: m.status }));
				share.links = (linksBy.get(r.id) ?? []).map((l) => ({ link_id: l.id, role: l.role, created_at: l.created_at, expires_at: l.expires_at }));
				share.requests = (requestsBy.get(r.id) ?? []).map((q) => ({
					account: q.account_id,
					username: q.username,
					link_id: q.link_id,
					role: q.role,
					requested_at: q.requested_at,
					sign_pub: q.sign_pub,
					box_pub: q.box_pub,
					binding: q.binding,
				}));
			}
			return share;
		}),
	});
});

/** `POST /v2/collab/shares/<id>/accept`: an invitation becomes membership. 204, also when already a member. */
export function accept(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		if (!isShareId(shareId)) return badRequest("Not a share ID.");
		const row = await env.DB.prepare(
			`UPDATE collab_members SET status = 'member', joined_at = coalesce(joined_at, ?3)
  WHERE share_id = ?1 AND account_id = ?2 RETURNING account_id`,
		)
			.bind(shareId, session.accountId, now)
			.first();
		return row ? noContent() : notFound();
	});
}

/**
 * The statements that take `accountId` out of a share it doesn't own: its
 * member row, its wraps and the notifications it holds for the share.
 */
function leaveStatements(db: D1Database, shareId: string, accountId: string, now: number): D1PreparedStatement[] {
	const notOwner = "NOT EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1 AND owner_id = ?2)";
	return [
		// Recorded first: the signed list keeps naming them until the owner cuts them, and the next epoch must not bring them back.
		db.prepare(`INSERT OR IGNORE INTO collab_left (share_id, account_id, left_at) SELECT ?1, ?2, ?3 WHERE ${notOwner}
  AND EXISTS (SELECT 1 FROM collab_members WHERE share_id = ?1 AND account_id = ?2)`).bind(shareId, accountId, now),
		db.prepare(`DELETE FROM collab_wraps WHERE share_id = ?1 AND account_id = ?2 AND ${notOwner}`).bind(shareId, accountId),
		db.prepare(`DELETE FROM collab_notifications WHERE share_id = ?1 AND recipient_id = ?2 AND ${notOwner}`).bind(shareId, accountId),
		db.prepare(`DELETE FROM collab_members WHERE share_id = ?1 AND account_id = ?2 AND ${notOwner} RETURNING account_id`).bind(shareId, accountId),
	];
}

/** Tells a room to close some accounts' sockets; a room that can't be reached closes them when they next fail. */
async function closeSockets(env: Env, shareId: string, accounts: string[]): Promise<void> {
	if (accounts.length === 0) return;
	try {
		await roomOf(env, shareId).removeMembers(accounts);
	} catch (err) {
		console.error(JSON.stringify({ route: "collab", op: "remove_members", outcome: (err as Error)?.name ?? typeof err }));
	}
}

/** `DELETE /v2/collab/shares/<id>/membership`: leave, or decline an invitation. The owner can't (403); they stop sharing. */
export function leave(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		if (!isShareId(shareId)) return badRequest("Not a share ID.");
		const s = await standing(env, shareId, session.accountId);
		if (!s || s.role === null) return notFound();
		if (s.ownerId === session.accountId) return forbiddenRole();
		const results = await env.DB.batch(leaveStatements(env.DB, shareId, session.accountId, now));
		if (results[3].results.length !== 1) return notFound();
		await closeSockets(env, shareId, [session.accountId]);
		return noContent();
	});
}

/**
 * `POST /v2/collab/shares/<id>/epoch`: the owner's new member list and a wrap
 * for every member, at the current epoch + 1 (else 409 `epoch_changed`). In
 * one compare-and-swap batch: accounts no longer listed lose their rows, wraps
 * and notifications; new ones are `invited`, or `member` at once when they
 * asked through a link (their request goes); roles follow the list. Then the
 * room hears the new epoch and roles. Sending the same epoch again (a lost
 * answer) answers 200 and tells the room again.
 */
export function newEpoch(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asOwner(env, shareId, session);
		if (s instanceof Response) return s;
		const body = await readJson(request, MAX_COLLAB_BODY);
		if (!body.ok) return body.response;
		const b = body.value;
		if (!isObject(b) || !Number.isSafeInteger(b.epoch)) return badRequest('The body must be {"epoch", "list", "list_sig", "wraps"}.');
		const epoch = b.epoch as number;
		const retry = epoch === s.epoch && b.list_sig === s.listSig;
		if (epoch !== s.epoch + 1 && !retry) return error(409, "epoch_changed", "The note has another epoch.", { epoch: s.epoch });
		const checked = checkList(b.list, shareId, epoch, session.accountId);
		if (typeof checked === "string") return badRequest(checked);
		const members = checked.members;
		if (members.length > MAX_MEMBERS) return error(413, "quota_exceeded", "Over quota.", { quota: "members", used: members.length, limit: MAX_MEMBERS });
		if (!isB64(b.list_sig, 64)) return badRequest("list_sig must be 86 base64url characters.");
		const wraps = checkWraps(b.wraps, members, epoch, true);
		if (!wraps) {
			return badRequest(
				`wraps must hold one 140-character wrap for each listed account at the new epoch, earlier epochs only for listed accounts, each once, at most ${maxWraps(epoch)}.`,
			);
		}
		const mismatch = await checkMemberKeys(env, members);
		if (mismatch) return mismatch;
		// The list says from which seq on the old epoch's writers no longer count; it must be the
		// head now, or the owner signs a new list (an identical retry of an applied epoch passes).
		if (!retry) {
			const { head } = await roomOf(env, shareId).state();
			if (checked.fromSeq !== head) return error(409, "head_moved", "The log moved on: sign the list again with from_seq at the head.", { head });
		}

		const accounts = JSON.stringify(members.map((m) => m.account));
		const values = [shareId, epoch, b.list_sig, b.list, now, session.accountId, accounts, JSON.stringify(members), JSON.stringify(wraps)];
		const mine = "EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1 AND epoch = ?2 AND list_sig = ?3)";
		const listed = "(SELECT value FROM json_each(?7))";
		// Someone who left stays out (no member row, no wrap) until a list stops naming them.
		const notLeft = "json_extract(value, '$.account') NOT IN (SELECT account_id FROM collab_left WHERE share_id = ?1)";
		const asked = "EXISTS (SELECT 1 FROM collab_requests r WHERE r.share_id = ?1 AND r.account_id = json_extract(value, '$.account'))";
		const sql = [
			`UPDATE collab_shares SET epoch = ?2, list = ?4, list_sig = ?3, updated_at = ?5
  WHERE id = ?1 AND owner_id = ?6 AND epoch = ?2 - 1 RETURNING id`,
			`DELETE FROM collab_wraps WHERE share_id = ?1 AND account_id NOT IN ${listed} AND ${mine}`,
			`DELETE FROM collab_notifications WHERE share_id = ?1 AND recipient_id NOT IN ${listed} AND ${mine}`,
			`DELETE FROM collab_members WHERE share_id = ?1 AND account_id NOT IN ${listed} AND ${mine}`,
			`DELETE FROM collab_left WHERE share_id = ?1 AND account_id NOT IN ${listed} AND ${mine}`,
			`INSERT INTO collab_members (share_id, account_id, role, status, invited_at, joined_at)
  SELECT ?1, json_extract(value, '$.account'), json_extract(value, '$.role'),
    CASE WHEN ${asked} THEN 'member' ELSE 'invited' END, ?5, CASE WHEN ${asked} THEN ?5 END
  FROM json_each(?8) WHERE ${mine} AND ${notLeft}
  ON CONFLICT (share_id, account_id) DO UPDATE SET role = excluded.role`,
			`DELETE FROM collab_requests WHERE share_id = ?1 AND account_id IN ${listed} AND ${mine}`,
			`INSERT INTO collab_wraps (share_id, account_id, epoch, wrapped)
  SELECT ?1, json_extract(value, '$.account'), json_extract(value, '$.epoch'), json_extract(value, '$.wrapped') FROM json_each(?9) WHERE ${mine} AND ${notLeft}
  ON CONFLICT (share_id, account_id, epoch) DO UPDATE SET wrapped = excluded.wrapped`,
			`INSERT INTO collab_lists (share_id, epoch, list, list_sig) SELECT ?1, ?2, ?4, ?3 WHERE ${mine}
  ON CONFLICT (share_id, epoch) DO NOTHING`,
		];
		const results = await guarded(
			env,
			sql.map((q) => bound(env.DB, q, values)),
			session.accountId,
			shareId,
			now,
		);
		if (results instanceof Response) return results;
		if (results[0].results.length !== 1 && !retry) {
			const current = await env.DB.prepare("SELECT epoch, list_sig FROM collab_shares WHERE id = ?1").bind(shareId).first<{ epoch: number; list_sig: string }>();
			if (!current) return notFound();
			if (current.epoch !== epoch || current.list_sig !== b.list_sig) {
				return error(409, "epoch_changed", "The note has another epoch.", { epoch: current.epoch });
			}
		}
		const roles = Object.fromEntries(members.map((m) => [m.account, m.role])) as Record<string, Role>;
		await roomOf(env, shareId).setEpoch({ epoch, roles });
		return json({ epoch });
	});
}

/**
 * The statements that stop sharing `shareId` when `ownerSql` holds (bound as
 * ?1 share, ?2 now, ?3 guard value): the room is queued for purging, and every
 * row of the share goes; deleting the share's row retires its ID.
 */
function stopStatements(db: D1Database, shareId: string, now: number, ownerSql: string, guard: string): D1PreparedStatement[] {
	const where = `share_id = ?1 AND EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1 AND ${ownerSql})`;
	const values = [shareId, now, guard];
	return [
		bound(
			db,
			`INSERT INTO collab_purges (id, requested_at) SELECT ?1, ?2 WHERE EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1 AND ${ownerSql})
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
			values,
		),
		bound(db, `DELETE FROM collab_wraps WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_left WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_lists WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_notifications WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_requests WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_links WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_members WHERE ${where}`, values),
		bound(db, `DELETE FROM collab_shares WHERE id = ?1 AND ${ownerSql} RETURNING id`, values),
	];
}

/** Purges one room and drops its purge row; false when the room couldn't be reached (the cron tries again). */
async function purgeRoom(env: Env, shareId: string): Promise<boolean> {
	try {
		await roomOf(env, shareId).purge();
	} catch (err) {
		console.error(JSON.stringify({ route: "collab", op: "purge", outcome: (err as Error)?.name ?? typeof err }));
		return false;
	}
	await env.DB.prepare("DELETE FROM collab_purges WHERE id = ?1").bind(shareId).run();
	return true;
}

/** `DELETE /v2/collab/shares/<id>`: stop sharing. Every row, wrap, link and request goes, the ID is retired, and the room is purged. 204. */
export function stopShare(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		const s = await asOwner(env, shareId, session);
		if (s instanceof Response) return s;
		const results = await env.DB.batch(stopStatements(env.DB, shareId, now, "owner_id = ?3", session.accountId));
		if (results[results.length - 1].results.length !== 1) return notFound();
		await purgeRoom(env, shareId);
		return noContent();
	});
}

// ----------------------------------------------------------------------- log

/**
 * `GET /v2/collab/shares/<id>/log?since=&limit=`: `{head, snapshot, entries,
 * more}` with envelopes as base64url. `limit` is 1–1000 (500 by default).
 */
export function readLog(shareId: string): Handler {
	return route(async (request, env, _now, session) => {
		const s = await asMember(env, shareId, session);
		if (s instanceof Response) return s;
		const params = new URL(request.url).searchParams;
		const since = natParam(params.get("since"), 0);
		const limit = natParam(params.get("limit"), 500);
		if (since === null || limit === null || limit < 1 || limit > MAX_LOG_LIMIT) return badRequest(`since must be ≥ 0 and limit 1–${MAX_LOG_LIMIT}.`);
		const page = await roomOf(env, shareId).read({ since, limit });
		return json({
			head: page.head,
			snapshot: page.snapshot ? { seq: page.snapshot.seq, base_seq: page.snapshot.base_seq, envelope: base64url(page.snapshot.envelope) } : null,
			entries: page.entries.map((e) => ({ seq: e.seq, envelope: base64url(e.envelope) })),
			more: page.more,
		});
	});
}

/**
 * `POST /v2/collab/shares/<id>/log`: one update (edit, owner) or comment
 * (comment, edit, owner), as bytes. 200 `{seq}`. The room refuses it with 413
 * `quota_exceeded` when the owner's document bytes would pass their quota
 * (`quota: "doc_bytes"`) or the room's comments 8 MiB (`quota: "comment_bytes"`).
 */
export function appendLog(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asMember(env, shareId, session);
		if (s instanceof Response) return s;
		const bytes = await readBody(request, MAX_ENVELOPE.u);
		if (!bytes) return tooLarge(`An envelope here is at most ${MAX_ENVELOPE.u} bytes.`);
		const header = checkEnvelope(bytes, ["u", "c"], s, session);
		if (header instanceof Response) return header;
		const result = await roomOf(env, shareId).append({ epoch: s.epoch, envelope: bytes, now });
		if (!result.ok) return roomRefusal(result);
		await wakeShare(env, shareId, session, now);
		return json({ seq: result.seq });
	});
}

/**
 * `POST /v2/collab/shares/<id>/snapshot` with `x-kanso-base-seq`: taken only
 * while the base is the log's head (409 `head_moved` `{head}`). A snapshot
 * takes no sequence number of its own: the answer's `seq` is its base. The
 * room's new size goes to `collab_shares.bytes`. 200 `{seq}`.
 */
export function postSnapshot(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asMember(env, shareId, session);
		if (s instanceof Response) return s;
		const base = request.headers.get("x-kanso-base-seq");
		const baseSeq = base === null ? null : natParam(base, 0);
		if (baseSeq === null) return badRequest("x-kanso-base-seq must be the log's head the snapshot was made at.");
		const bytes = await readBody(request, MAX_ENVELOPE.s);
		if (!bytes) return tooLarge(`A snapshot is at most ${MAX_ENVELOPE.s} bytes.`);
		const header = checkEnvelope(bytes, ["s"], s, session);
		if (header instanceof Response) return header;
		const result = await roomOf(env, shareId).snapshot({ epoch: s.epoch, baseSeq, envelope: bytes, now });
		if (!result.ok) return roomRefusal(result);
		return json({ seq: result.seq });
	});
}

/**
 * `GET /v2/collab/shares/<id>/live`: the WebSocket, accepted by the room with
 * the member's account, device, role and the epoch read here. A plain GET is
 * 426 `upgrade_required`; a ninth socket of one account in one room is 429
 * `rate_limited` with `limit: "sockets"`.
 */
export function live(shareId: string): Handler {
	return route(async (request, env, _now, session) => {
		const s = await asMember(env, shareId, session);
		if (s instanceof Response) return s;
		if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
			return error(426, "upgrade_required", "Open this route as a WebSocket.", {}, { upgrade: "websocket" });
		}
		return roomOf(env, shareId).fetch(
			new Request("https://room/live", {
				headers: {
					upgrade: "websocket",
					[ROOM_HEADERS.account]: session.accountId,
					[ROOM_HEADERS.device]: session.deviceId,
					[ROOM_HEADERS.role]: s.role ?? "view",
					[ROOM_HEADERS.epoch]: String(s.epoch),
				},
			}),
		);
	});
}

// --------------------------------------------------------- links and requests

/** `POST /v2/collab/shares/<id>/links` `{link_id, proof_hash, role, expires_at}`: 201 `{link_id, role, expires_at}`. */
export function createLink(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asOwner(env, shareId, session);
		if (s instanceof Response) return s;
		const body = await readJson(request, MAX_COLLAB_BODY);
		if (!body.ok) return body.response;
		const b = body.value;
		if (!isObject(b)) return badRequest("The body must be a JSON object.");
		const { link_id, proof_hash, role, expires_at } = b;
		if (typeof link_id !== "string" || !isShareId(link_id)) return badRequest("link_id must be 16 bytes of base64url (22 characters).");
		if (typeof proof_hash !== "string" || !isSha256Hex(proof_hash)) return badRequest("proof_hash must be 64 lowercase hex digits.");
		if (typeof role !== "string" || !(LINK_ROLES as readonly string[]).includes(role)) return badRequest("role must be edit, comment or view.");
		if (!Number.isSafeInteger(expires_at) || (expires_at as number) <= now || (expires_at as number) > now + MAX_LINK_MS) {
			return badRequest("expires_at must be a Unix time in ms, later than now and at most 30 days ahead.");
		}
		const results = await guarded(
			env,
			[
				env.DB.prepare(
					`INSERT INTO collab_links (id, share_id, proof_hash, role, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
  ON CONFLICT (id) DO NOTHING RETURNING id`,
				).bind(link_id, shareId, proof_hash, role, now, expires_at),
			],
			session.accountId,
			shareId,
			now,
		);
		if (results instanceof Response) return results;
		if (results[0].results.length !== 1) {
			const same = await env.DB.prepare(
				"SELECT 1 AS same FROM collab_links WHERE id = ?1 AND share_id = ?2 AND proof_hash = ?3 AND role = ?4 AND expires_at = ?5",
			)
				.bind(link_id, shareId, proof_hash, role, expires_at)
				.first();
			if (!same) return error(409, "link_id_taken", "That link ID can't be used; make a new link.");
		}
		return json({ link_id, role, expires_at }, 201);
	});
}

/** `DELETE /v2/collab/shares/<id>/links/<link_id>`: cancels a link. 204, or 404. */
export function deleteLink(shareId: string, linkId: string): Handler {
	return route(async (_request, env, _now, session) => {
		const s = await asOwner(env, shareId, session);
		if (s instanceof Response) return s;
		const row = await env.DB.prepare("DELETE FROM collab_links WHERE id = ?1 AND share_id = ?2 RETURNING id").bind(linkId, shareId).first();
		return row ? noContent() : notFound();
	});
}

/**
 * `POST /v2/collab/join/<link_id>` `{proof}`: asks to join through a link,
 * counted by `join-account` before anything is read. 202 `{share_id,
 * owner_username, role}`; 404 for an unknown, expired or cancelled link and a
 * wrong proof alike. Someone already in the note gets the same 202 and no
 * request.
 */
export function join(linkId: string): Handler {
	return route(async (request, env, now, session) => {
		const limited = await count(env.DB, "join-account", accountKey("join-account", session.accountId), now);
		if (limited) return limited;
		const body = await readJson(request, MAX_COLLAB_BODY);
		if (!body.ok) return body.response;
		const proof = isObject(body.value) ? proof32(body.value.proof) : null;
		if (!proof) return badRequest("proof must be 32 bytes of canonical base64url.");
		if (!isShareId(linkId)) return notFound();
		if (!(await identityOf(env, session.accountId))) return identityRequired();
		const link = await env.DB.prepare(
			`SELECT l.share_id, l.proof_hash, l.role, l.expires_at, o.username AS owner_username
  FROM collab_links l JOIN collab_shares s ON s.id = l.share_id JOIN accounts o ON o.id = s.owner_id WHERE l.id = ?1`,
		)
			.bind(linkId)
			.first<{ share_id: string; proof_hash: string; role: Role; expires_at: number; owner_username: string }>();
		// The proof is always hashed and compared, so an unknown link costs what a wrong proof does.
		const matches = await proofMatches(proof, link?.proof_hash);
		if (!link || !matches || link.expires_at <= now) return notFound();
		const answer = json({ share_id: link.share_id, owner_username: link.owner_username, role: link.role }, 202);
		const member = await env.DB.prepare("SELECT 1 AS member FROM collab_members WHERE share_id = ?1 AND account_id = ?2").bind(link.share_id, session.accountId).first();
		if (member) return answer;
		const s = await standing(env, link.share_id, session.accountId);
		const results = await guarded(
			env,
			[
				// Asking again through a link is a new, deliberate request: it ends an earlier leaving.
				env.DB.prepare("DELETE FROM collab_left WHERE share_id = ?1 AND account_id = ?2").bind(link.share_id, session.accountId),
				env.DB.prepare(
					`INSERT INTO collab_requests (share_id, account_id, link_id, role, requested_at) VALUES (?1, ?2, ?3, ?4, ?5)
  ON CONFLICT (share_id, account_id) DO UPDATE SET link_id = excluded.link_id, role = excluded.role, requested_at = excluded.requested_at`,
				).bind(link.share_id, session.accountId, linkId, link.role, now),
			],
			s?.ownerId ?? "",
			link.share_id,
			now,
		);
		return results instanceof Response ? results : answer;
	});
}

/** `DELETE /v2/collab/shares/<id>/requests/<account>`: declines a request. 204, or 404. */
export function declineRequest(shareId: string, accountId: string): Handler {
	return route(async (_request, env, _now, session) => {
		const s = await asOwner(env, shareId, session);
		if (s instanceof Response) return s;
		if (!isAccountId(accountId)) return badRequest("Not an account ID.");
		const row = await env.DB.prepare("DELETE FROM collab_requests WHERE share_id = ?1 AND account_id = ?2 RETURNING account_id").bind(shareId, accountId).first();
		return row ? noContent() : notFound();
	});
}

// ------------------------------------------------------------- notifications

/**
 * `POST /v2/collab/notifications` `{share_id, recipient, sealed}`: from a
 * member who may comment, to a member or invitee of the same note, counted by
 * `notify-account`. 201 `{id}`. The recipient keeps the newest 500.
 */
export const notify = route(async (request, env, now, session) => {
	const limited = await count(env.DB, "notify-account", accountKey("notify-account", session.accountId), now);
	if (limited) return limited;
	const body = await readJson(request, MAX_COLLAB_BODY);
	if (!body.ok) return body.response;
	const b = body.value;
	if (!isObject(b)) return badRequest("The body must be a JSON object.");
	const { share_id, recipient, sealed } = b;
	if (typeof share_id !== "string" || !isShareId(share_id)) return badRequest("share_id must be a share ID.");
	if (typeof recipient !== "string" || !isAccountId(recipient)) return badRequest("recipient must be an account ID.");
	const bytes = typeof sealed === "string" ? fromBase64url(sealed) : null;
	if (!bytes || bytes.length < MIN_NOTICE_BYTES || bytes.length > MAX_NOTICE_BYTES) {
		return badRequest(`sealed must be ${MIN_NOTICE_BYTES}–${MAX_NOTICE_BYTES} bytes of canonical base64url.`);
	}
	const s = await standing(env, share_id, session.accountId);
	if (!s || s.role === null || s.status !== "member") return notFound();
	if (!mayWrite(s.role, "c")) return forbiddenRole();
	const id = base64url(randomBytes(16));
	const row = await env.DB.prepare(
		`INSERT INTO collab_notifications (id, recipient_id, share_id, sender_id, sealed, at)
  SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE EXISTS (SELECT 1 FROM collab_members WHERE share_id = ?3 AND account_id = ?2) RETURNING id`,
	)
		.bind(id, recipient, share_id, session.accountId, sealed, now)
		.first();
	return row ? json({ id }, 201) : notFound();
});

/** `GET /v2/collab/notifications`: `{notifications: [{id, share_id, from, sealed, at}]}`, oldest first. */
export const listNotifications = route(async (_request, env, _now, session) => {
	const { results } = await env.DB.prepare(
		`SELECT id, share_id, sender_id AS "from", sealed, at FROM collab_notifications WHERE recipient_id = ?1
  ORDER BY at, id LIMIT ${MAX_NOTIFICATIONS}`,
	)
		.bind(session.accountId)
		.all();
	return json({ notifications: results });
});

/** `DELETE /v2/collab/notifications/<id>`: the recipient's. 204, or 404. */
export function deleteNotification(id: string): Handler {
	return route(async (_request, env, _now, session) => {
		if (!isShareId(id)) return badRequest("Not a notification ID.");
		const row = await env.DB.prepare("DELETE FROM collab_notifications WHERE id = ?1 AND recipient_id = ?2 RETURNING id").bind(id, session.accountId).first();
		return row ? noContent() : notFound();
	});
}

// ----------------------------------------------------------- the invite page

export const INVITE_STYLE = `:root{color-scheme:light dark;--paper:#fbfaf7;--ink:#2b2a27;--muted:#76736c;--line:#e6e2da;--card:#ffffff}
@media (prefers-color-scheme:dark){:root{--paper:#1c1b19;--ink:#e9e6df;--muted:#9a968d;--line:#34322e;--card:#232220}}
*{box-sizing:border-box}
html,body{margin:0;min-height:100%;background:var(--paper);color:var(--ink);font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}
main{min-height:100vh;display:flex;align-items:center;justify-content:center;padding:24px 16px}
.card{max-width:440px;padding:28px 30px;border:1px solid var(--line);border-radius:12px;background:var(--card)}
h1{margin:0 0 8px;font-size:17px;font-weight:600;letter-spacing:-.01em}
p{margin:0 0 8px;color:var(--muted)}
strong{color:var(--ink);font-weight:600}`;

export const INVITE_HTML = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow">
<meta name="referrer" content="no-referrer">
<title>Join a shared note</title>
<style>${INVITE_STYLE}</style>
</head>
<body>
<main><div class="card">
<h1>You're invited to a shared note</h1>
<p>Open Kanso, choose <strong>Join a shared note…</strong>, and paste this link.</p>
<p>The note opens only in Kanso: this page can't read it, and neither can the server.</p>
</div></main>
</body>
</html>
`;

let invitePolicy: Promise<string> | null = null;

/** Its own style by hash, and nothing else at all: the page has no script and loads nothing. */
function inviteCsp(): Promise<string> {
	invitePolicy ??= (async () => {
		const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(INVITE_STYLE)));
		let binary = "";
		for (const byte of digest) binary += String.fromCharCode(byte);
		return [
			"default-src 'none'",
			`style-src 'sha256-${btoa(binary)}'`,
			"base-uri 'none'",
			"form-action 'none'",
			"frame-ancestors 'none'",
		].join("; ");
	})();
	return invitePolicy;
}

/**
 * `GET /j/<link_id>`: the page an invite link opens in a browser. The same for
 * every link and it never reads D1; behind the PUBLIC_LIMITER binding only.
 * The link's secret stays in the #fragment, which browsers never send.
 */
export function invitePage(linkId: string): Handler {
	return async (request, env) => {
		const headers = { ...PUBLIC_HEADERS, "cache-control": "no-store" };
		if (!isShareId(linkId)) {
			const r = notFound();
			for (const [k, v] of Object.entries(headers)) r.headers.set(k, v);
			return r;
		}
		if (env.PUBLIC_LIMITER) {
			const { success } = await env.PUBLIC_LIMITER.limit({ key: ipGroup(clientIp(request)) });
			if (!success) {
				const r = rateLimited("binding", BINDING_PERIOD_MS);
				for (const [k, v] of Object.entries(headers)) r.headers.set(k, v);
				return r;
			}
		}
		return new Response(INVITE_HTML, {
			headers: {
				"content-type": "text/html; charset=utf-8",
				"content-security-policy": await inviteCsp(),
				"cache-control": "public, max-age=300",
				"permissions-policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
				...PUBLIC_HEADERS,
			},
		});
	};
}

// ------------------------------------------------------ deletion and the cron

/**
 * The statements that take an account out of shared notebooks (an account
 * deletion), bound as the share route binds them: ?1 account, ?2 guard value,
 * ?3 now. Its own notebooks are stopped (`stopped_at` is what lets their lists
 * go; deleting the row retires the ID), and its place in others' goes.
 */
function bookAccountStatements(bind: (sql: string) => D1PreparedStatement, guardSql: string): D1PreparedStatement[] {
	const owned = "(SELECT id FROM book_shares WHERE owner_id = ?1)";
	return [
		bind(`UPDATE book_shares SET stopped_at = ?3 WHERE owner_id = ?1 AND ${guardSql}`),
		bind(`DELETE FROM book_wraps WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM book_epochs WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM book_seen_keys WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM book_lists WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM book_links WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM book_members WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM book_shares WHERE owner_id = ?1 AND ${guardSql}`),
	];
}

/**
 * The statements that take an account out of sharing (an account deletion),
 * guarded by `guardSql` (bound as ?1 account, ?2 guard value, ?3 now): its
 * own shares are stopped (rows gone, IDs retired, rooms queued for purging),
 * and its identity, memberships, wraps, requests and notifications go.
 */
export function accountCollabStatements(db: D1Database, accountId: string, guardSql: string, guardValue: string, now: number): D1PreparedStatement[] {
	const owned = "(SELECT id FROM collab_shares WHERE owner_id = ?1)";
	const bind = (sql: string) => bound(db, sql, [accountId, guardValue, now]);
	return [
		bind(
			`INSERT INTO collab_purges (id, requested_at) SELECT id, ?3 FROM collab_shares WHERE owner_id = ?1 AND ${guardSql}
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
		),
		bind(`DELETE FROM collab_notifications WHERE (recipient_id = ?1 OR sender_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM collab_wraps WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM collab_left WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM collab_lists WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM collab_requests WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM collab_links WHERE share_id IN ${owned} AND ${guardSql}`),
		bind(`DELETE FROM collab_members WHERE (account_id = ?1 OR share_id IN ${owned}) AND ${guardSql}`),
		bind(`DELETE FROM collab_shares WHERE owner_id = ?1 AND ${guardSql}`),
		...bookAccountStatements(bind, guardSql),
		bind(`DELETE FROM identities WHERE account_id = ?1 AND ${guardSql}`),
	];
}

/** The shares an account is in without owning them: their rooms close its sockets once it is deleted. */
export async function memberships(env: Env, accountId: string): Promise<string[]> {
	const { results } = await env.DB.prepare("SELECT share_id FROM collab_members WHERE account_id = ?1 AND role != 'owner'")
		.bind(accountId)
		.all<{ share_id: string }>();
	return results.map((r) => r.share_id);
}

/** After an account deletion: its sockets in others' rooms close, and its own rooms are purged. */
export async function afterAccountDeleted(env: Env, accountId: string, shares: string[], now: number): Promise<void> {
	await Promise.all(shares.map((id) => closeSockets(env, id, [accountId])));
	await runCollabPurges(env, now);
}

/**
 * Purges the rooms in collab_purges, oldest first, and drops each row once its
 * room has deleted everything; expired links go too. Returns the rooms purged.
 */
export async function runCollabPurges(env: Env, now: number, limit = COLLAB_PURGE_BATCH): Promise<number> {
	await env.DB.prepare("DELETE FROM collab_links WHERE expires_at <= ?1").bind(now).run();
	await env.DB.prepare("DELETE FROM book_links WHERE expires_at <= ?1").bind(now).run();
	const { results } = await env.DB.prepare("SELECT id FROM collab_purges ORDER BY requested_at, id LIMIT ?1").bind(limit).all<{ id: string }>();
	let purged = 0;
	for (const { id } of results) if (await purgeRoom(env, id)) purged += 1;
	return purged;
}
