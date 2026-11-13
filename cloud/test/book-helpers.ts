// Helpers for the shared-notebook tests: signed member lists in the chain's shape (with random
// signatures: the server never checks them), wraps, history links and a client for every
// /v2/collab/books route.

import { linkHash } from "../src/book";
import { type Listed, type Role, b64, ensureIdentity, newShareId } from "./collab-helpers";
import { type Account, base64url, callJson } from "./helpers";

export { newShareId, b64 };
export type { Listed, Role };

const encoder = new TextEncoder();

export interface ListSpec {
	shareId: string;
	rev: number;
	epoch: number;
	owner: Account;
	signer: Account;
	prev?: string;
	keyCommit?: string;
	fromSeq?: number | null;
	members: Listed[];
	/** Overrides of the list's own fields (to forge one). */
	extra?: Record<string, unknown>;
}

/** The commitment a test epoch uses unless told otherwise: 64 hex digits that differ per epoch. */
export const commitOf = (epoch: number): string => epoch.toString(16).padStart(64, "0");

/** A signed list's bytes as base64url, with its link hash. Members are sorted by account ID. */
export async function bookList(s: ListSpec): Promise<{ list: string; hash: string }> {
	const members = [];
	for (const m of [...s.members].sort((x, y) => (x.account.creds.accountId < y.account.creds.accountId ? -1 : 1))) {
		const id = await ensureIdentity(m.account);
		members.push({
			account: m.account.creds.accountId,
			username: m.claims?.username ?? m.account.creds.username,
			role: m.role,
			sign_pub: m.claims?.sign_pub ?? id.sign_pub,
			box_pub: m.claims?.box_pub ?? id.box_pub,
		});
	}
	const body: Record<string, unknown> = {
		v: 1,
		share: s.shareId,
		rev: s.rev,
		epoch: s.epoch,
		owner: s.owner.creds.accountId,
		signer: s.signer.creds.accountId,
		...(s.rev > 1 ? { prev: s.prev } : {}),
		key_commit: s.keyCommit ?? commitOf(s.epoch),
		at: 1_800_057_600_000,
		...(s.epoch > 1 ? { from_seq: s.fromSeq ?? 0 } : {}),
		members,
		...s.extra,
	};
	const bytes = encoder.encode(JSON.stringify(body));
	return { list: base64url(bytes), hash: await linkHash(bytes) };
}

export const wraps = (accounts: Account[]) => accounts.map((a) => ({ account: a.creds.accountId, wrapped: b64(105) }));

/** What a test holds of a notebook's chain: its ID, head and members. */
export interface Book {
	id: string;
	owner: Account;
	rev: number;
	hash: string;
	epoch: number;
	members: Listed[];
}

/** The owner's genesis body for a notebook with `others`. */
export async function genesisBody(id: string, owner: Account, others: Listed[]) {
	const members: Listed[] = [{ account: owner, role: "owner" }, ...others];
	for (const m of members) await ensureIdentity(m.account);
	const { list, hash } = await bookList({ shareId: id, rev: 1, epoch: 1, owner, signer: owner, members });
	return { body: { share_id: id, list, list_sig: b64(64), wraps: wraps(members.map((m) => m.account)) }, hash, members };
}

export interface NextOptions {
	newEpoch?: boolean;
	wrapFor?: Account[];
	history?: boolean;
	extra?: Record<string, unknown>;
	/** The log head a new epoch names (0 by default). */
	fromSeq?: number;
}

/** The next list's body after `book`, signed by `signer`, with `members` listed. */
export async function nextBody(book: Book, signer: Account, members: Listed[], opts: NextOptions = {}) {
	const epoch = opts.newEpoch ? book.epoch + 1 : book.epoch;
	const next = await bookList({
		shareId: book.id,
		rev: book.rev + 1,
		epoch,
		owner: book.owner,
		signer,
		prev: book.hash,
		fromSeq: opts.newEpoch || book.epoch > 1 ? (opts.fromSeq ?? 0) : null,
		members,
		extra: opts.extra,
	});
	const wrapFor = opts.wrapFor ?? (opts.newEpoch ? members.map((m) => m.account) : []);
	return {
		hash: next.hash,
		epoch,
		body: {
			rev: book.rev + 1,
			list: next.list,
			list_sig: b64(64),
			wraps: wraps(wrapFor),
			...(opts.newEpoch || opts.history ? { history_link: b64(73) } : {}),
		},
	};
}

/** Every /v2/collab/books route as the holder of `token`. */
export function books(token: string) {
	const as = (path: string, init: Parameters<typeof callJson>[1] = {}) => callJson(path, { ...init, token });
	const base = (id: string) => `/v2/collab/books/${id}`;
	return {
		create: (body: unknown) => as("/v2/collab/books", { json: body }),
		mine: () => as("/v2/collab/books"),
		get: (id: string, query = "") => as(`${base(id)}${query}`),
		list: (id: string, body: unknown) => as(`${base(id)}/epoch`, { json: body }),
		link: (id: string, body: unknown) => as(`${base(id)}/links`, { json: body }),
		unlink: (id: string, linkId: string) => as(`${base(id)}/links/${linkId}`, { method: "DELETE" }),
		accept: (id: string) => as(`${base(id)}/accept`, { method: "POST" }),
		leave: (id: string) => as(`${base(id)}/membership`, { method: "DELETE" }),
		stop: (id: string) => as(base(id), { method: "DELETE" }),
	};
}

/** Shares a notebook from `owner` with `others`, everyone accepting; the book. */
export async function shareBook(owner: Account, others: Listed[], accept = true): Promise<Book> {
	const id = newShareId();
	const { body, hash, members } = await genesisBody(id, owner, others);
	const r = await books(owner.token).create(body);
	if (r.status !== 201) throw new Error(`create: ${r.status} ${JSON.stringify(r.body)}`);
	if (accept) for (const m of others) await books(m.account.token).accept(id);
	return { id, owner, rev: 1, hash, epoch: 1, members };
}

/** Appends a list to `book` as `signer`; the book's head moves on when it is accepted. */
export async function advance(book: Book, signer: Account, members: Listed[], opts: NextOptions = {}) {
	const next = await nextBody(book, signer, members, opts);
	const r = await books(signer.token).list(book.id, next.body);
	if (r.status === 200) Object.assign(book, { rev: book.rev + 1, hash: next.hash, epoch: next.epoch, members });
	return r;
}
