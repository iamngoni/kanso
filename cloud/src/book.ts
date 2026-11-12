// Shared notebooks (issue #390 › Shared notebooks: Worker): the routes under
// /v2/collab/books (they extend the /v2/collab routes of single shared notes;
// no new prefix).
//
// The server stores and relays what clients sign and wrap, and never holds a
// key:
//   - the notebook's chain of signed member lists (rev, prev hash, epoch,
//     signer, key commitment), append-only: a list is accepted only as the
//     head's rev + 1 naming the head's hash, checked in the Worker and again
//     by a trigger, and no list is ever updated or deleted while the notebook
//     lives;
//   - every member's wrap of the notebook key per epoch (opaque, 105 bytes);
//   - each epoch's history link (opaque, 73 bytes).
// Clients verify the chain's signatures back to the owner's genesis list
// (crates/kanso-collab/src/book_members.rs). What the server checks is
// what it can without trusting anyone: shapes, the chain's position, that the
// caller is the list's signer and may sign (owner, or a manager the previous
// list names), the rank and epoch rules (a manager changes only people below
// Manager; a removal or demotion starts a new epoch), the 25-member cap, and
// that every listed account's keys and username are the account's own.
//
// Errors follow the single-note routes: 404 `not_found` for a notebook the
// caller isn't in, 403 `forbidden_role` for one they are in but whose role
// doesn't allow it, 409 `rev_conflict` `{rev, hash}` for a list that doesn't
// follow the head, 413 `quota_exceeded` `{quota, used, limit}`, 429
// `rate_limited` (`book-read-account`, `book-write-account`).

import type { Session } from "./auth";
import { BOOK_SHARE_HEADER, MANIFEST_ID, bookRoomOf, closeBookSockets, purgeBookRoom } from "./book_room";
import { type Handler, LINK_ROLES, MAX_LINK_MS, MAX_LIST_CHARS, bound, checkMemberKeys, forbiddenRole, identityRequired, noContent, route } from "./collab";
import { type Header, type Kind, MAX_ENVELOPE, MAX_LOG_LIMIT, type Role, isManifest, mayWriteStream, parseEnvelope } from "./envelope";
import type { Env } from "./env";
import { badRequest, constraint, error, json, notFound, readBody, readJson, refusal, tooLarge } from "./http";
import { identityOf, isB64 } from "./identity";
import { accountKey, count } from "./limits";
import { base64url, fromBase64url, isAccountId, isObject, isSha256Hex, natParam } from "./protocol";
import { ROOM_HEADERS, type RoomRefusal } from "./room";
import { isShareId } from "./shares";

export const MAX_BOOK_MEMBERS = 25;
export const MAX_OWNED_BOOKS = 50;
export const MAX_BOOK_LINKS = 10;
/** How far a new epoch's `from_seq` may jump past the previous epoch's (as the clients' rule). */
export const MAX_SEQ_JUMP = 1_000_000;
/** The most lists one GET returns; `more` says there are further ones. */
export const MAX_LISTS_PAGE = 200;
const MAX_BODY = 64 * 1024;

const BOOK_ROLES: readonly Role[] = ["owner", "manager", "edit", "comment", "view"];
const RANK: Record<Role, number> = { view: 0, comment: 1, edit: 2, manager: 3, owner: 4 };
/** A history link is a sealed 32-byte key: format byte, 24-byte nonce, 32 bytes, 16-byte tag. */
const HISTORY_LINK_BYTES = 73;

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false });

// ------------------------------------------------------------------ the list

export interface BookMember {
	account: string;
	username: string;
	role: Role;
	sign_pub: string;
	box_pub: string;
}

export interface BookList {
	rev: number;
	epoch: number;
	owner: string;
	signer: string;
	prev: string | null;
	keyCommit: string;
	fromSeq: number | null;
	members: BookMember[];
	/** The link hash a next list names as `prev`. */
	hash: string;
}

/** `hex(SHA-256("kanso-book-members-link" || 0x01 || list bytes))`. */
export async function linkHash(bytes: Uint8Array): Promise<string> {
	const label = new TextEncoder().encode("kanso-book-members-link");
	const input = new Uint8Array(label.length + 1 + bytes.length);
	input.set(label, 0);
	input[label.length] = 1;
	input.set(bytes, label.length + 1);
	const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", input));
	return [...digest].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/**
 * A signed list the caller sent, after the checks the server relies on:
 * canonical base64url of UTF-8 JSON, `v` 1, this notebook, a position and
 * epoch, owner and signer accounts, `prev` exactly from rev 2 on, a key
 * commitment, `from_seq` exactly from epoch 2 on, members sorted by account
 * ID without duplicates, each with a known role and key shapes, and exactly
 * one `owner`, the list's owner. The member cap and the signature are not
 * checked here (the cap is the caller's 413; the signature is the clients').
 */
export async function parseBookList(list: unknown, shareId: string): Promise<BookList | string> {
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
	if (value.v !== 1 || value.share !== shareId) return "list must have v 1 and this notebook's share ID.";
	const { rev, epoch, owner, signer, prev, key_commit, at, from_seq } = value;
	if (!Number.isSafeInteger(rev) || (rev as number) < 1) return "list.rev must be an integer ≥ 1.";
	if (!Number.isSafeInteger(epoch) || (epoch as number) < 1) return "list.epoch must be an integer ≥ 1.";
	if (typeof owner !== "string" || !isAccountId(owner) || typeof signer !== "string" || !isAccountId(signer)) return "list.owner and list.signer must be account IDs.";
	if (rev === 1) {
		if (Object.hasOwn(value, "prev")) return "list.prev is only for lists after the first.";
	} else if (typeof prev !== "string" || !isSha256Hex(prev)) return "list.prev must be 64 lowercase hex digits from rev 2 on.";
	if (typeof key_commit !== "string" || !isSha256Hex(key_commit)) return "list.key_commit must be 64 lowercase hex digits.";
	if (!Number.isSafeInteger(at)) return "list.at must be an integer.";
	if (epoch === 1) {
		if (Object.hasOwn(value, "from_seq")) return "list.from_seq is only for epochs after the first.";
	} else if (!Number.isSafeInteger(from_seq) || (from_seq as number) < 0) return "list.from_seq must be an integer ≥ 0 from epoch 2 on.";
	if (!Array.isArray(value.members) || value.members.length === 0) return "list.members must be a non-empty array.";
	if (value.members.length > 2 * MAX_BOOK_MEMBERS) return "list.members is far over the member limit.";
	const members: BookMember[] = [];
	for (const m of value.members) {
		if (!isObject(m)) return "Each member must be an object.";
		const { account, username, role, sign_pub, box_pub } = m;
		if (typeof account !== "string" || !isAccountId(account)) return "A member's account isn't an account ID.";
		if (typeof username !== "string" || username.length === 0 || username.length > 32) return "A member's username must be a string.";
		if (typeof role !== "string" || !(BOOK_ROLES as readonly string[]).includes(role)) return "A member's role must be owner, manager, edit, comment or view.";
		if (!isB64(sign_pub, 32) || !isB64(box_pub, 32)) return "A member's keys must be 43 base64url characters.";
		const previous = members[members.length - 1];
		if (previous && previous.account >= account) return "list.members must be sorted by account ID, each once.";
		members.push({ account, username, role: role as Role, sign_pub, box_pub });
	}
	const owners = members.filter((m) => m.role === "owner");
	if (owners.length !== 1 || owners[0].account !== owner) return "list.members must have exactly one owner: the list's owner.";
	return {
		rev: rev as number,
		epoch: epoch as number,
		owner,
		signer,
		prev: rev === 1 ? null : (prev as string),
		keyCommit: key_commit,
		fromSeq: epoch === 1 ? null : (from_seq as number),
		members,
		hash: await linkHash(bytes),
	};
}

const same = (a: BookMember | undefined, b: BookMember | undefined): boolean =>
	a === b || (!!a && !!b && a.username === b.username && a.role === b.role && a.sign_pub === b.sign_pub && a.box_pub === b.box_pub);

/**
 * What a signer of `signerRole` may change between `prev` and `next`, as the
 * clients check it: nobody changes the owner's entry; anyone who stays keeps
 * their username and keys (a new key is a removal and a fresh addition); a
 * manager touches nobody at Manager or above, on either side. Whether anyone
 * was removed or demoted (then the list must start a new epoch), or the reason.
 */
export function rankRules(prev: BookList, next: BookList, signerRole: Role): boolean | string {
	const before = new Map(prev.members.map((m) => [m.account, m]));
	const after = new Map(next.members.map((m) => [m.account, m]));
	if (!same(before.get(prev.owner), after.get(prev.owner))) return "Nobody changes the owner's entry.";
	let cuts = false;
	for (const account of new Set([...before.keys(), ...after.keys()])) {
		const b = before.get(account);
		const a = after.get(account);
		if (same(a, b)) continue;
		if (a && b && (a.sign_pub !== b.sign_pub || a.box_pub !== b.box_pub || a.username !== b.username)) {
			return "Someone who stays in the notebook keeps their username and keys; only a role changes.";
		}
		if (signerRole !== "owner" && ((b && RANK[b.role] >= RANK.manager) || (a && RANK[a.role] >= RANK.manager))) {
			return "Only the owner changes a manager's entry or makes someone a manager.";
		}
		cuts ||= !!b && (!a || RANK[a.role] < RANK[b.role]);
	}
	return cuts;
}

/** The epoch rules: a list in the same epoch keeps its commitment and log position and cuts nobody; a new epoch is the next, with a new commitment and a log position that doesn't go back or jump absurdly. */
export function epochRules(prev: BookList, next: BookList, cuts: boolean): string | null {
	if (next.epoch === prev.epoch) {
		if (cuts) return "A removal or demotion starts a new epoch.";
		if (next.keyCommit !== prev.keyCommit || next.fromSeq !== prev.fromSeq) return "A list in the same epoch keeps its key commitment and from_seq.";
		return null;
	}
	if (next.epoch !== prev.epoch + 1) return "A new epoch is the previous epoch + 1.";
	if (next.keyCommit === prev.keyCommit) return "A new epoch has a new key commitment.";
	const base = prev.fromSeq ?? 0;
	if (next.fromSeq === null || next.fromSeq < base || next.fromSeq - base > MAX_SEQ_JUMP) return "from_seq must not go back or jump past the previous epoch's by more than 1,000,000.";
	return null;
}

/**
 * `[{account, wrapped}]` with exactly one 105-byte wrap for each of `accounts`
 * (the wraps a list needs, at its epoch), none for anyone else. Null when it isn't so.
 */
function checkWraps(wraps: unknown, accounts: ReadonlySet<string>): { account: string; wrapped: string }[] | null {
	if (!Array.isArray(wraps) || wraps.length !== accounts.size) return null;
	const seen = new Set<string>();
	const out: { account: string; wrapped: string }[] = [];
	for (const w of wraps) {
		if (!isObject(w) || typeof w.account !== "string" || !accounts.has(w.account) || seen.has(w.account) || !isB64(w.wrapped, 105)) return null;
		seen.add(w.account);
		out.push({ account: w.account, wrapped: w.wrapped as string });
	}
	return out;
}

// ------------------------------------------------------------ who is asking

interface BookStanding {
	ownerId: string;
	epoch: number;
	headRev: number;
	headHash: string;
	role: Role | null;
	status: "invited" | "member" | null;
}

async function standing(env: Env, shareId: string, accountId: string): Promise<BookStanding | null> {
	const row = await env.DB.prepare(
		`SELECT s.owner_id, s.epoch, s.head_rev, s.head_hash, m.role, m.status FROM book_shares s
  LEFT JOIN book_members m ON m.share_id = s.id AND m.account_id = ?2 WHERE s.id = ?1 AND s.stopped_at IS NULL`,
	)
		.bind(shareId, accountId)
		.first<{ owner_id: string; epoch: number; head_rev: number; head_hash: string; role: Role | null; status: "invited" | "member" | null }>();
	return row ? { ownerId: row.owner_id, epoch: row.epoch, headRev: row.head_rev, headHash: row.head_hash, role: row.role, status: row.status } : null;
}

type Need = "read" | "member" | "manager" | "owner";

/**
 * The notebook and the caller's place in it, or the answer: 400 for a bad ID,
 * 404 for a notebook the caller isn't in (an invitation counts as in only to
 * read; anything else needs it accepted), 403 `forbidden_role` for a role
 * that doesn't allow the route. Counted per account first.
 */
async function asBook(env: Env, shareId: string, session: { accountId: string }, need: Need, now: number): Promise<BookStanding | Response> {
	const rule = need === "read" ? "book-read-account" : "book-write-account";
	const limited = await count(env.DB, rule, accountKey(rule, session.accountId), now);
	if (limited) return limited;
	if (!isShareId(shareId)) return badRequest("Not a share ID.");
	const s = await standing(env, shareId, session.accountId);
	if (!s || s.role === null) return notFound();
	if (need === "read") return s;
	if (s.status !== "member") return notFound();
	if (need === "owner" && s.ownerId !== session.accountId) return forbiddenRole();
	if (need === "manager" && s.role !== "owner" && s.role !== "manager") return forbiddenRole();
	return s;
}

const conflict = (s: { headRev: number; headHash: string }): Response =>
	error(409, "rev_conflict", "The member list moved on: build the next one on the head.", { rev: s.headRev, hash: s.headHash });

async function quotaExceeded(env: Env, quota: string, ownerId: string, shareId: string, now: number): Promise<Response> {
	const figure = async (sql: string, ...values: unknown[]): Promise<number> => (await bound(env.DB, sql, values).first<{ n: number }>())?.n ?? 0;
	switch (quota) {
		case "books":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM book_shares WHERE owner_id = ?1", ownerId),
				limit: MAX_OWNED_BOOKS,
			});
		case "members":
			return error(413, "quota_exceeded", "Over quota.", {
				quota,
				used: await figure("SELECT count(*) AS n FROM book_members WHERE share_id = ?1", shareId),
				limit: MAX_BOOK_MEMBERS,
			});
		default:
			return error(413, "quota_exceeded", "Over quota.", {
				quota: "links",
				used: await figure("SELECT count(*) AS n FROM book_links WHERE share_id = ?1 AND expires_at > ?2", shareId, now),
				limit: MAX_BOOK_LINKS,
			});
	}
}

/** Runs a batch; a trigger's `quota_exceeded:<kind>` becomes the 413 answer, `rev_conflict` the 409, `append_only` a 409. */
async function guarded(env: Env, statements: D1PreparedStatement[], ownerId: string, shareId: string, now: number): Promise<D1Result[] | Response> {
	try {
		return await env.DB.batch(statements);
	} catch (err) {
		const r = refusal(err);
		if (r?.kind === "quota_exceeded") return quotaExceeded(env, r.detail, ownerId, shareId, now);
		const text = `${(err as Error)?.message ?? ""} ${String((err as { cause?: unknown })?.cause ?? "")}`;
		if (text.includes("rev_conflict")) {
			const s = await standing(env, shareId, ownerId);
			return s ? conflict(s) : notFound();
		}
		if (text.includes("append_only")) return error(409, "append_only", "Member lists are never changed or removed.");
		if (constraint(err, "CHECK")) return badRequest("A value isn't storable.");
		throw err;
	}
}

// ------------------------------------------------------------------- create

/**
 * `POST /v2/collab/books` `{share_id, list, list_sig, wraps}`: shares a
 * notebook at epoch 1 with the owner's signed genesis list (rev 1, signed by
 * and for the caller) and one wrap per listed member. The owner is a member
 * at once; everyone else listed is invited. 201 `{share_id, epoch, rev}`;
 * sending the same notebook again (a lost answer) answers 201 again; an ID in
 * use is 409 `share_id_taken`, a retired one 409 `share_retired`.
 */
export const createBook = route(async (request, env, now, session) => {
	const limited = await count(env.DB, "book-write-account", accountKey("book-write-account", session.accountId), now);
	if (limited) return limited;
	const body = await readJson(request, MAX_BODY);
	if (!body.ok) return body.response;
	const b = body.value;
	if (!isObject(b)) return badRequest("The body must be a JSON object.");
	const shareId = b.share_id;
	if (typeof shareId !== "string" || !isShareId(shareId)) return badRequest("share_id must be 16 bytes of base64url (22 characters).");
	if (!(await identityOf(env, session.accountId))) return identityRequired();
	const parsed = await parseBookList(b.list, shareId);
	if (typeof parsed === "string") return badRequest(parsed);
	if (parsed.rev !== 1 || parsed.epoch !== 1) return badRequest("A new notebook starts with rev 1 at epoch 1.");
	if (parsed.owner !== session.accountId || parsed.signer !== session.accountId) return badRequest("The genesis list must name you as its owner and signer.");
	const members = parsed.members;
	if (members.length > MAX_BOOK_MEMBERS) return error(413, "quota_exceeded", "Over quota.", { quota: "members", used: members.length, limit: MAX_BOOK_MEMBERS });
	if (!isB64(b.list_sig, 64)) return badRequest("list_sig must be 86 base64url characters.");
	const wraps = checkWraps(b.wraps, new Set(members.map((m) => m.account)));
	if (!wraps) return badRequest("wraps must hold one 140-character wrap for each listed account.");
	const mismatch = await checkMemberKeys(env, members);
	if (mismatch) return mismatch;

	const values = [shareId, session.accountId, b.list_sig, b.list, now, parsed.hash, JSON.stringify(members), JSON.stringify(wraps)];
	const mine = "EXISTS (SELECT 1 FROM book_shares WHERE id = ?1 AND owner_id = ?2 AND created_at = ?5 AND NOT EXISTS (SELECT 1 FROM book_lists WHERE share_id = ?1))";
	const results = await guarded(
		env,
		[
			bound(
				env.DB,
				`INSERT INTO book_shares (id, owner_id, epoch, head_rev, head_hash, created_at, updated_at)
  SELECT ?1, ?2, 1, 1, ?6, ?5, ?5 WHERE NOT EXISTS (SELECT 1 FROM collab_retired WHERE id = ?1)
    AND NOT EXISTS (SELECT 1 FROM collab_shares WHERE id = ?1)
  ON CONFLICT (id) DO NOTHING RETURNING id`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO book_lists (share_id, rev, epoch, signer, prev, hash, list, list_sig) SELECT ?1, 1, 1, ?2, NULL, ?6, ?4, ?3 WHERE ${mine}`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO book_members (share_id, account_id, role, status, invited_at, joined_at)
  SELECT ?1, json_extract(value, '$.account'), json_extract(value, '$.role'),
    CASE WHEN json_extract(value, '$.account') = ?2 THEN 'member' ELSE 'invited' END, ?5,
    CASE WHEN json_extract(value, '$.account') = ?2 THEN ?5 END
  FROM json_each(?7) WHERE EXISTS (SELECT 1 FROM book_lists WHERE share_id = ?1 AND rev = 1 AND hash = ?6)
    AND EXISTS (SELECT 1 FROM book_shares WHERE id = ?1 AND owner_id = ?2 AND created_at = ?5)
  ON CONFLICT (share_id, account_id) DO NOTHING`,
				values,
			),
			bound(
				env.DB,
				`INSERT INTO book_wraps (share_id, account_id, epoch, wrapped)
  SELECT ?1, json_extract(value, '$.account'), 1, json_extract(value, '$.wrapped') FROM json_each(?8)
  WHERE EXISTS (SELECT 1 FROM book_lists WHERE share_id = ?1 AND rev = 1 AND hash = ?6)
    AND EXISTS (SELECT 1 FROM book_shares WHERE id = ?1 AND owner_id = ?2 AND created_at = ?5)
  ON CONFLICT (share_id, account_id, epoch) DO NOTHING`,
				values,
			),
		],
		session.accountId,
		shareId,
		now,
	);
	if (results instanceof Response) return results;
	if (results[0].results.length !== 1) {
		const first = await env.DB.prepare(
			"SELECT 1 AS ok FROM book_shares s JOIN book_lists l ON l.share_id = s.id AND l.rev = 1 WHERE s.id = ?1 AND s.owner_id = ?2 AND l.list_sig = ?3 AND l.hash = ?4",
		)
			.bind(shareId, session.accountId, b.list_sig, parsed.hash)
			.first();
		if (!first) {
			const retired = await env.DB.prepare("SELECT 1 AS retired FROM collab_retired WHERE id = ?1").bind(shareId).first();
			return retired
				? error(409, "share_retired", "That share ID stopped being shared and can't be used again; share under a new one.")
				: error(409, "share_id_taken", "That share ID can't be used; share under a new one.");
		}
	}
	return json({ share_id: shareId, epoch: 1, rev: 1 }, 201);
});

// ---------------------------------------------------------------------- get

/**
 * `GET /v2/collab/books/<id>?since_rev=&limit=`: for anyone in or invited to
 * the notebook, `{share_id, owner, role, status, epoch, rev, hash, lists,
 * more, wraps, epochs}`: the chain from `since_rev` (1 by default, at most
 * `limit` 1–200 lists, `more` when further ones follow), the caller's own
 * wraps, and every epoch's history link. The owner and managers also get
 * `members` (account, role, status) and the live `links`.
 */
export function getBook(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asBook(env, shareId, session, "read", now);
		if (s instanceof Response) return s;
		const params = new URL(request.url).searchParams;
		const since = natParam(params.get("since_rev"), 1);
		const limit = natParam(params.get("limit"), MAX_LISTS_PAGE);
		if (since === null || since < 1 || limit === null || limit < 1 || limit > MAX_LISTS_PAGE) {
			return badRequest(`since_rev must be ≥ 1 and limit 1–${MAX_LISTS_PAGE}.`);
		}
		const manages = s.role === "owner" || s.role === "manager";
		const [lists, wraps, epochs, members, links] = await env.DB.batch([
			env.DB.prepare("SELECT rev, epoch, signer, list, list_sig FROM book_lists WHERE share_id = ?1 AND rev >= ?2 ORDER BY rev LIMIT ?3").bind(shareId, since, limit + 1),
			env.DB.prepare("SELECT epoch, wrapped FROM book_wraps WHERE share_id = ?1 AND account_id = ?2 ORDER BY epoch").bind(shareId, session.accountId),
			env.DB.prepare("SELECT epoch, history_link FROM book_epochs WHERE share_id = ?1 ORDER BY epoch").bind(shareId),
			env.DB.prepare("SELECT account_id, role, status FROM book_members WHERE share_id = ?1 AND ?2 = 1 ORDER BY account_id").bind(shareId, manages ? 1 : 0),
			env.DB.prepare("SELECT id, role, created_by, created_at, expires_at FROM book_links WHERE share_id = ?1 AND expires_at > ?2 AND ?3 = 1 ORDER BY created_at, id").bind(
				shareId,
				now,
				manages ? 1 : 0,
			),
		]);
		const rows = lists.results as { rev: number; epoch: number; signer: string; list: string; list_sig: string }[];
		const out: Record<string, unknown> = {
			share_id: shareId,
			owner: s.ownerId,
			role: s.role,
			status: s.status,
			epoch: s.epoch,
			rev: s.headRev,
			hash: s.headHash,
			lists: rows.slice(0, limit).map((l) => ({ rev: l.rev, epoch: l.epoch, signer: l.signer, list: l.list, list_sig: l.list_sig })),
			more: rows.length > limit,
			wraps: wraps.results,
			epochs: epochs.results,
		};
		if (manages) {
			out.members = (members.results as { account_id: string; role: string; status: string }[]).map((m) => ({ account: m.account_id, role: m.role, status: m.status }));
			out.links = (links.results as { id: string; role: string; created_by: string; created_at: number; expires_at: number }[]).map((l) => ({
				link_id: l.id,
				role: l.role,
				created_by: l.created_by,
				created_at: l.created_at,
				expires_at: l.expires_at,
			}));
		}
		return json(out);
	});
}

// -------------------------------------------------------------------- epoch

/**
 * `POST /v2/collab/books/<id>/epoch` `{rev, list, list_sig, wraps,
 * history_link?}`: the next signed member list, from the owner or a manager
 * the head names. `rev` must be the head's + 1 and the list name the head's
 * hash (else 409 `rev_conflict` `{rev, hash}`); the caller must be its signer.
 * Within the same epoch (adding people, or promoting) the wraps are for
 * those added or whose role changed (a wrap binds the role); a removal or
 * demotion starts the next epoch, which carries a wrap for every listed
 * member and a history link. One compare-and-swap batch: accounts no longer
 * listed lose their rows and wraps, new ones are `invited`, roles follow the
 * list. Sending the head again (a lost answer) answers 200. 200 `{epoch,
 * rev, hash}`.
 */
export function newList(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asBook(env, shareId, session, "manager", now);
		if (s instanceof Response) return s;
		const body = await readJson(request, MAX_BODY);
		if (!body.ok) return body.response;
		const b = body.value;
		if (!isObject(b) || !Number.isSafeInteger(b.rev)) return badRequest('The body must be {"rev", "list", "list_sig", "wraps", "history_link"?}.');
		const rev = b.rev as number;
		const head = await env.DB.prepare("SELECT list, list_sig, hash, signer FROM book_lists WHERE share_id = ?1 AND rev = ?2")
			.bind(shareId, s.headRev)
			.first<{ list: string; list_sig: string; hash: string; signer: string }>();
		if (!head) return notFound();
		if (rev === s.headRev && b.list_sig === head.list_sig && b.list === head.list && head.signer === session.accountId) {
			const again = await parseBookList(head.list, shareId);
			if (typeof again !== "string") await tellRoom(env, shareId, s.epoch, again.members);
			return json({ epoch: s.epoch, rev: s.headRev, hash: s.headHash });
		}
		if (rev !== s.headRev + 1) return conflict(s);
		const next = await parseBookList(b.list, shareId);
		if (typeof next === "string") return badRequest(next);
		if (next.rev !== rev) return badRequest("list.rev must be the body's rev.");
		if (next.prev !== s.headHash) return conflict(s);
		if (next.owner !== s.ownerId) return badRequest("Ownership doesn't move: list.owner must stay the notebook's owner.");
		if (next.signer !== session.accountId) return error(403, "forbidden_author", "The list's signer must be this session's account.");
		const prev = await parseBookList(head.list, shareId);
		if (typeof prev === "string") return error(500, "internal", "The stored head list can't be read.");
		const signer = prev.members.find((m) => m.account === session.accountId);
		if (!signer || (signer.role !== "owner" && signer.role !== "manager")) return forbiddenRole();
		const cuts = rankRules(prev, next, signer.role);
		if (typeof cuts === "string") return error(403, "forbidden_role", cuts);
		// Only the owner re-adds a removed account with keys other than the ones it last had (as the clients' `seen_keys`).
		if (signer.role !== "owner") {
			const { results } = await env.DB.prepare("SELECT list FROM book_lists WHERE share_id = ?1 AND rev <= ?2 ORDER BY rev").bind(shareId, s.headRev).all<{ list: string }>();
			const seen = new Map<string, BookMember>();
			for (const row of results) {
				const earlier = await parseBookList(row.list, shareId);
				if (typeof earlier !== "string") for (const m of earlier.members) seen.set(m.account, m);
			}
			const before = new Set(prev.members.map((m) => m.account));
			const readded = next.members.find((m) => {
				const old = seen.get(m.account);
				return !before.has(m.account) && !!old && (old.sign_pub !== m.sign_pub || old.box_pub !== m.box_pub);
			});
			if (readded) return error(403, "forbidden_role", "Only the owner re-adds a removed account with keys other than the ones it last had.");
		}
		const wrong = epochRules(prev, next, cuts);
		if (wrong) return badRequest(wrong);
		// The list says from which seq on the old epoch's writers no longer count; it must be the
		// notebook's real log head now, or the signer signs the list again with from_seq at the head.
		if (next.epoch !== prev.epoch) {
			const { head } = await bookRoomOf(env, shareId).state();
			if (next.fromSeq !== head) return error(409, "head_moved", "The log moved on: sign the list again with from_seq at the head.", { head });
		}
		const members = next.members;
		if (members.length > MAX_BOOK_MEMBERS) return error(413, "quota_exceeded", "Over quota.", { quota: "members", used: members.length, limit: MAX_BOOK_MEMBERS });
		if (!isB64(b.list_sig, 64)) return badRequest("list_sig must be 86 base64url characters.");

		const newEpoch = next.epoch !== prev.epoch;
		const before = new Map(prev.members.map((m) => [m.account, m]));
		const needWrap = new Set(members.filter((m) => newEpoch || before.get(m.account)?.role !== m.role).map((m) => m.account));
		const wraps = checkWraps(b.wraps, needWrap);
		if (!wraps) {
			return badRequest(
				newEpoch
					? "wraps must hold one 140-character wrap for each listed account at the new epoch."
					: "wraps must hold one 140-character wrap for each account added or given another role, and no others.",
			);
		}
		const link = b.history_link;
		if (newEpoch ? !isB64(link, HISTORY_LINK_BYTES) : link !== undefined && link !== null) {
			return badRequest(newEpoch ? "history_link must be the previous epoch's key sealed under the new one (98 characters)." : "history_link is only for a new epoch.");
		}
		const mismatch = await checkMemberKeys(env, members);
		if (mismatch) return mismatch;

		const values = [
			shareId,
			rev,
			b.list_sig,
			b.list,
			now,
			session.accountId,
			JSON.stringify(members),
			JSON.stringify(wraps),
			next.epoch,
			next.hash,
			s.headHash,
			newEpoch ? link : null,
			JSON.stringify(members.map((m) => m.account)),
		];
		const mine = "EXISTS (SELECT 1 FROM book_shares WHERE id = ?1 AND head_rev = ?2 AND head_hash = ?10)";
		const listed = "(SELECT value FROM json_each(?13))";
		const sql = [
			`UPDATE book_shares SET epoch = ?9, head_rev = ?2, head_hash = ?10, updated_at = ?5
  WHERE id = ?1 AND stopped_at IS NULL AND head_rev = ?2 - 1 AND head_hash = ?11 RETURNING id`,
			`INSERT INTO book_lists (share_id, rev, epoch, signer, prev, hash, list, list_sig) SELECT ?1, ?2, ?9, ?6, ?11, ?10, ?4, ?3 WHERE ${mine}`,
			`DELETE FROM book_wraps WHERE share_id = ?1 AND account_id NOT IN ${listed} AND ${mine}`,
			`DELETE FROM book_members WHERE share_id = ?1 AND account_id NOT IN ${listed} AND ${mine}`,
			`INSERT INTO book_members (share_id, account_id, role, status, invited_at, joined_at)
  SELECT ?1, json_extract(value, '$.account'), json_extract(value, '$.role'), 'invited', ?5, NULL FROM json_each(?7) WHERE ${mine}
  ON CONFLICT (share_id, account_id) DO UPDATE SET role = excluded.role`,
			`INSERT INTO book_wraps (share_id, account_id, epoch, wrapped)
  SELECT ?1, json_extract(value, '$.account'), ?9, json_extract(value, '$.wrapped') FROM json_each(?8) WHERE ${mine}
  ON CONFLICT (share_id, account_id, epoch) DO UPDATE SET wrapped = excluded.wrapped`,
			`INSERT INTO book_epochs (share_id, epoch, history_link) SELECT ?1, ?9, ?12 WHERE ?12 IS NOT NULL AND ${mine}`,
		];
		const results = await guarded(
			env,
			sql.map((q) => bound(env.DB, q, values)),
			s.ownerId,
			shareId,
			now,
		);
		if (results instanceof Response) return results;
		if (results[0].results.length !== 1) {
			const current = await standing(env, shareId, session.accountId);
			return current ? conflict(current) : notFound();
		}
		await tellRoom(env, shareId, next.epoch, members);
		return json({ epoch: next.epoch, rev, hash: next.hash });
	});
}

// -------------------------------------------------------------------- links

/** `POST /v2/collab/books/<id>/links` `{link_id, proof_hash, role, expires_at}` (the owner or a manager): 201 `{link_id, role, expires_at}`. */
export function createLink(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asBook(env, shareId, session, "manager", now);
		if (s instanceof Response) return s;
		const body = await readJson(request, MAX_BODY);
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
					`INSERT INTO book_links (id, share_id, proof_hash, role, created_by, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
  ON CONFLICT (id) DO NOTHING RETURNING id`,
				).bind(link_id, shareId, proof_hash, role, session.accountId, now, expires_at),
			],
			s.ownerId,
			shareId,
			now,
		);
		if (results instanceof Response) return results;
		if (results[0].results.length !== 1) {
			const again = await env.DB.prepare("SELECT 1 AS same FROM book_links WHERE id = ?1 AND share_id = ?2 AND proof_hash = ?3 AND role = ?4 AND expires_at = ?5")
				.bind(link_id, shareId, proof_hash, role, expires_at)
				.first();
			if (!again) return error(409, "link_id_taken", "That link ID can't be used; make a new link.");
		}
		return json({ link_id, role, expires_at }, 201);
	});
}

/** `DELETE /v2/collab/books/<id>/links/<link_id>`: cancels a link (the owner or a manager). 204, or 404. */
export function deleteLink(shareId: string, linkId: string): Handler {
	return route(async (_request, env, now, session) => {
		const s = await asBook(env, shareId, session, "manager", now);
		if (s instanceof Response) return s;
		const row = await env.DB.prepare("DELETE FROM book_links WHERE id = ?1 AND share_id = ?2 RETURNING id").bind(linkId, shareId).first();
		return row ? noContent() : notFound();
	});
}

// ------------------------------------------------- accept, leave and stop

/** `POST /v2/collab/books/<id>/accept`: an invitation becomes membership. 204, also when already a member. */
export function accept(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		const s = await asBook(env, shareId, session, "read", now);
		if (s instanceof Response) return s;
		await env.DB.prepare("UPDATE book_members SET status = 'member', joined_at = coalesce(joined_at, ?3) WHERE share_id = ?1 AND account_id = ?2").bind(shareId, session.accountId, now).run();
		return noContent();
	});
}

/** `DELETE /v2/collab/books/<id>/membership`: leave, or decline an invitation, losing the wraps. The owner can't (403); they stop sharing. */
export function leave(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		const s = await asBook(env, shareId, session, "read", now);
		if (s instanceof Response) return s;
		if (s.ownerId === session.accountId) return forbiddenRole();
		const results = await env.DB.batch([
			env.DB.prepare("DELETE FROM book_wraps WHERE share_id = ?1 AND account_id = ?2").bind(shareId, session.accountId),
			env.DB.prepare("DELETE FROM book_members WHERE share_id = ?1 AND account_id = ?2 RETURNING account_id").bind(shareId, session.accountId),
		]);
		if (results[1].results.length !== 1) return notFound();
		await closeBookSockets(env, shareId, [session.accountId]);
		return noContent();
	});
}

/** `DELETE /v2/collab/books/<id>`: stop sharing (the owner). Every row goes, the ID is retired for good. 204. */
export function stopBook(shareId: string): Handler {
	return route(async (_request, env, now, session) => {
		const s = await asBook(env, shareId, session, "owner", now);
		if (s instanceof Response) return s;
		const mine = "EXISTS (SELECT 1 FROM book_shares WHERE id = ?1 AND owner_id = ?3)";
		const values = [shareId, now, session.accountId];
		const results = await env.DB.batch([
			bound(
				env.DB,
				`INSERT INTO book_purges (id, requested_at) SELECT ?1, ?2 WHERE ${mine} ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
				values,
			),
			bound(env.DB, "UPDATE book_shares SET stopped_at = ?2 WHERE id = ?1 AND owner_id = ?3", values),
			bound(env.DB, `DELETE FROM book_wraps WHERE share_id = ?1 AND ${mine}`, values),
			bound(env.DB, `DELETE FROM book_epochs WHERE share_id = ?1 AND ${mine}`, values),
			bound(env.DB, `DELETE FROM book_lists WHERE share_id = ?1 AND ${mine}`, values),
			bound(env.DB, `DELETE FROM book_links WHERE share_id = ?1 AND ${mine}`, values),
			bound(env.DB, `DELETE FROM book_members WHERE share_id = ?1 AND ${mine}`, values),
			bound(env.DB, "DELETE FROM book_shares WHERE id = ?1 AND owner_id = ?3 RETURNING id", values),
		]);
		if (results[results.length - 1].results.length !== 1) return notFound();
		await purgeBookRoom(env, shareId);
		return noContent();
	});
}

// ------------------------------------------------------ the logs and the room

/** Tells the room the notebook's epoch and everyone's role: sockets of accounts no longer listed close at once. */
async function tellRoom(env: Env, shareId: string, epoch: number, members: BookMember[]): Promise<void> {
	const roles = Object.fromEntries(members.map((m) => [m.account, m.role])) as Record<string, Role>;
	try {
		await bookRoomOf(env, shareId).setEpoch({ epoch, roles });
	} catch (err) {
		console.error(JSON.stringify({ route: "book", op: "set_epoch", outcome: (err as Error)?.name ?? typeof err }));
	}
}

const roomRefusal = (r: RoomRefusal): Response => {
	const extras = r.extras ?? {};
	const headers: Record<string, string> = typeof extras.retry_after === "number" ? { "retry-after": String(extras.retry_after) } : {};
	return error(r.status, r.error, `Refused by the room: ${r.error}.`, extras, headers);
};

/** A note's stream ID: its 16 bytes as 22 base64url characters; never the manifest's all-zero one. Null when it isn't. */
function noteStream(raw: string): string | null {
	const bytes = isShareId(raw) ? fromBase64url(raw) : null;
	return bytes && bytes.length === 16 && !isManifest(bytes) ? raw : null;
}

/**
 * The notebook for a log or socket route: counted per account (`book-log-account`), 400 for a bad ID,
 * 404 for anyone who isn't an accepted member (an invitation, someone who left or was removed, a stranger).
 */
async function asLogMember(env: Env, shareId: string, session: Session, now: number): Promise<BookStanding | Response> {
	const limited = await count(env.DB, "book-log-account", accountKey("book-log-account", session.accountId), now);
	if (limited) return limited;
	if (!isShareId(shareId)) return badRequest("Not a share ID.");
	const s = await standing(env, shareId, session.accountId);
	if (!s || s.role === null || s.status !== "member") return notFound();
	return s;
}

/** The header of an envelope sent to a stream: format, kind for the route, epoch (409), author and device (403), role (403), size (413). */
function checkEnvelope(bytes: Uint8Array, kinds: readonly Kind[], stream: string, s: BookStanding, session: Session): Header | Response {
	const header = parseEnvelope(bytes);
	if (!header) return badRequest("Not an envelope (0x01, kind, epoch, author, device, counter, nonce, sealed, signature).");
	if (!kinds.includes(header.kind)) return badRequest(`This route takes ${kinds.join(" or ")} envelopes.`);
	if (header.epoch !== s.epoch) return error(409, "epoch_changed", "The notebook has a newer epoch.", { epoch: s.epoch });
	if (header.author !== session.accountId || header.device !== session.deviceId) {
		return error(403, "forbidden_author", "The envelope's author and device must be this session's.");
	}
	if (!s.role || !mayWriteStream(s.role, header.kind, stream === MANIFEST_ID)) return forbiddenRole();
	if (bytes.length > MAX_ENVELOPE[header.kind]) return tooLarge(`A ${header.kind} envelope is at most ${MAX_ENVELOPE[header.kind]} bytes.`);
	return header;
}

/**
 * `GET /v2/collab/books/<id>/manifest` and `/notes/<note>?since=&limit=`: `{head, stream_head,
 * snapshot, entries, more}`, envelopes as base64url. `head` is the notebook's (every stream's seqs
 * share one count), `stream_head` this log's last seq; `since` is a seq, `limit` is 1–1000 (500 by default).
 */
export function readStream(shareId: string, note: string | null): Handler {
	return route(async (request, env, now, session) => {
		const s = await asLogMember(env, shareId, session, now);
		if (s instanceof Response) return s;
		const stream = note === null ? MANIFEST_ID : noteStream(note);
		if (stream === null) return badRequest("A note ID is 16 bytes of base64url (22 characters), and not all zeros.");
		const params = new URL(request.url).searchParams;
		const since = natParam(params.get("since"), 0);
		const limit = natParam(params.get("limit"), 500);
		if (since === null || limit === null || limit < 1 || limit > MAX_LOG_LIMIT) return badRequest(`since must be ≥ 0 and limit 1–${MAX_LOG_LIMIT}.`);
		const page = await bookRoomOf(env, shareId).read({ stream, since, limit });
		return json({
			head: page.head,
			stream_head: page.stream_head,
			snapshot: page.snapshot ? { seq: page.snapshot.seq, base_seq: page.snapshot.base_seq, envelope: base64url(page.snapshot.envelope) } : null,
			entries: page.entries.map((e) => ({ seq: e.seq, envelope: base64url(e.envelope) })),
			more: page.more,
		});
	});
}

/**
 * `POST /v2/collab/books/<id>/manifest` (an update) and `/notes/<note>` (an update or a comment),
 * as bytes: 200 `{seq}`. The room refuses with 413 `quota_exceeded` past the owner's document bytes
 * (`doc_bytes`), 2,000 notes (`notes`) or a note's 8 MiB of comments (`comment_bytes`).
 */
export function appendStream(shareId: string, note: string | null): Handler {
	return route(async (request, env, now, session) => {
		const s = await asLogMember(env, shareId, session, now);
		if (s instanceof Response) return s;
		const stream = note === null ? MANIFEST_ID : noteStream(note);
		if (stream === null) return badRequest("A note ID is 16 bytes of base64url (22 characters), and not all zeros.");
		const bytes = await readBody(request, MAX_ENVELOPE.u);
		if (!bytes) return tooLarge(`An envelope here is at most ${MAX_ENVELOPE.u} bytes.`);
		const header = checkEnvelope(bytes, note === null ? ["u"] : ["u", "c"], stream, s, session);
		if (header instanceof Response) return header;
		const result = await bookRoomOf(env, shareId).append({ shareId, stream, epoch: s.epoch, envelope: bytes, now });
		if (!result.ok) return roomRefusal(result);
		return json({ seq: result.seq });
	});
}

/**
 * `POST /v2/collab/books/<id>/manifest/snapshot` and `/notes/<note>/snapshot` with `x-kanso-base-seq`:
 * taken only while the base is that log's last seq (409 `head_moved` `{head}`; 0 for a log with
 * nothing). It replaces the log's updates up to the base; its seq is the base. 200 `{seq}`.
 */
export function postStreamSnapshot(shareId: string, note: string | null): Handler {
	return route(async (request, env, now, session) => {
		const s = await asLogMember(env, shareId, session, now);
		if (s instanceof Response) return s;
		const stream = note === null ? MANIFEST_ID : noteStream(note);
		if (stream === null) return badRequest("A note ID is 16 bytes of base64url (22 characters), and not all zeros.");
		const base = request.headers.get("x-kanso-base-seq");
		const baseSeq = base === null ? null : natParam(base, 0);
		if (baseSeq === null) return badRequest("x-kanso-base-seq must be the log's last seq the snapshot was made at.");
		const bytes = await readBody(request, MAX_ENVELOPE.s);
		if (!bytes) return tooLarge(`A snapshot is at most ${MAX_ENVELOPE.s} bytes.`);
		const header = checkEnvelope(bytes, ["s"], stream, s, session);
		if (header instanceof Response) return header;
		const result = await bookRoomOf(env, shareId).snapshot({ shareId, stream, epoch: s.epoch, baseSeq, envelope: bytes, now });
		if (!result.ok) return roomRefusal(result);
		return json({ seq: result.seq });
	});
}

/**
 * `GET /v2/collab/books/<id>/live`: the WebSocket, one per open notebook, accepted by the room with
 * the member's account, device, role and the epoch read here. A plain GET is 426 `upgrade_required`;
 * a ninth socket of one account is 429 `rate_limited` with `limit: "sockets"`.
 */
export function live(shareId: string): Handler {
	return route(async (request, env, now, session) => {
		const s = await asLogMember(env, shareId, session, now);
		if (s instanceof Response) return s;
		if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
			return error(426, "upgrade_required", "Open this route as a WebSocket.", {}, { upgrade: "websocket" });
		}
		return bookRoomOf(env, shareId).fetch(
			new Request("https://room/live", {
				headers: {
					upgrade: "websocket",
					[ROOM_HEADERS.account]: session.accountId,
					[ROOM_HEADERS.device]: session.deviceId,
					[ROOM_HEADERS.role]: s.role ?? "view",
					[ROOM_HEADERS.epoch]: String(s.epoch),
					[BOOK_SHARE_HEADER]: shareId,
				},
			}),
		);
	});
}
