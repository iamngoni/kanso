// Helpers for the shared-note tests: identities, signed member lists (with
// random signatures: the server never checks them), wraps, envelopes sealed
// with a stand-in cipher, a client for every /v2/collab route, and live
// sockets with a message queue.
//
// The stand-in cipher is AES-256-GCM under a note key only the test holds, so
// what the server stores is real ciphertext: a plaintext marker the test seals
// can't appear in it, and the tests check that it never appears anywhere the
// server keeps anything.

import { env } from "cloudflare:workers";
import { type Account, base64url, call, callJson } from "./helpers";

/** Random bytes, in 64 KiB pieces (getRandomValues' limit). */
function random(n: number): Uint8Array {
	const out = new Uint8Array(n);
	for (let i = 0; i < n; i += 65_536) crypto.getRandomValues(out.subarray(i, Math.min(n, i + 65_536)));
	return out;
}
const encoder = new TextEncoder();

export type Role = "owner" | "manager" | "edit" | "comment" | "view";
export type Kind = "u" | "s" | "c" | "p";

export const newShareId = (): string => base64url(random(16));
export const b64 = (n: number): string => base64url(random(n));

export interface Identity {
	sign_pub: string;
	box_pub: string;
	binding: string;
	sealed: string;
}

export const newIdentity = (): Identity => ({ sign_pub: b64(32), box_pub: b64(32), binding: b64(64), sealed: b64(105) });

const identities = new Map<string, Identity>();

/** Sets `a`'s identity once (the server refuses a different one), and remembers it for lists. */
export async function ensureIdentity(a: Account): Promise<Identity> {
	const known = identities.get(a.creds.accountId);
	if (known) {
		const r = await callJson("/v2/account/identity", { token: a.token, method: "PUT", json: known });
		if (r.status === 200 || r.status === 201) return known;
	}
	const identity = newIdentity();
	const r = await callJson("/v2/account/identity", { token: a.token, method: "PUT", json: identity });
	if (r.status !== 201) throw new Error(`identity for ${a.creds.username}: ${r.status} ${JSON.stringify(r.body)}`);
	identities.set(a.creds.accountId, identity);
	return identity;
}

export interface Listed {
	account: Account;
	role: Role;
	/** What the list claims instead of the account's own keys or name (a forged entry). */
	claims?: { sign_pub?: string; box_pub?: string; username?: string };
}

/** The owner's signed list for `epoch` as base64url, members sorted by account ID. */
export function memberList(shareId: string, epoch: number, owner: Account, members: Listed[], fromSeq: number | null = epoch > 1 ? 0 : null): string {
	const list = {
		v: 1,
		share: shareId,
		epoch,
		owner: owner.creds.accountId,
		key_commit: [...random(32)].map((b) => b.toString(16).padStart(2, "0")).join(""),
		at: 1_800_057_600_000,
		// From epoch 2 on: the log's head when the epoch began, between `at` and `members`.
		...(fromSeq === null ? {} : { from_seq: fromSeq }),
		members: [...members]
			.sort((x, y) => (x.account.creds.accountId < y.account.creds.accountId ? -1 : 1))
			.map((m) => {
				const id = identities.get(m.account.creds.accountId) ?? newIdentity();
				return {
					account: m.account.creds.accountId,
					username: m.claims?.username ?? m.account.creds.username,
					role: m.role,
					sign_pub: m.claims?.sign_pub ?? id.sign_pub,
					box_pub: m.claims?.box_pub ?? id.box_pub,
				};
			}),
	};
	return base64url(encoder.encode(JSON.stringify(list)));
}

export const wrapsFor = (members: Listed[]) => members.map((m) => ({ account: m.account.creds.accountId, wrapped: b64(105) }));

/** The note key of each share and epoch, which only the test holds. */
const noteKeys = new Map<string, CryptoKey>();

async function noteKey(shareId: string, epoch: number): Promise<CryptoKey> {
	const name = `${shareId}/${epoch}`;
	let key = noteKeys.get(name);
	if (!key) {
		key = (await crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"])) as CryptoKey;
		noteKeys.set(name, key);
	}
	return key;
}

/** `0x01 || kind || u32(epoch) || lp(author) || lp(device) || u64(counter)`. */
export function header(kind: Kind, epoch: number, author: string, device: string, counter: bigint): Uint8Array {
	const a = encoder.encode(author);
	const d = encoder.encode(device);
	const out = new Uint8Array(1 + 1 + 4 + 2 + a.length + 2 + d.length + 8);
	const view = new DataView(out.buffer);
	out[0] = 0x01;
	out[1] = kind.charCodeAt(0);
	view.setUint32(2, epoch);
	view.setUint16(6, a.length);
	out.set(a, 8);
	view.setUint16(8 + a.length, d.length);
	out.set(d, 10 + a.length);
	view.setBigUint64(10 + a.length + d.length, counter);
	return out;
}

let counter = 0n;

export interface Seal {
	kind: Kind;
	epoch: number;
	shareId: string;
	/** Whose envelope: author and device. */
	by: Account;
	/** The plaintext; random bytes of `size` when absent. */
	plaintext?: Uint8Array | string;
	size?: number;
	author?: string;
	device?: string;
}

/** An envelope as a device writes it: the header, a nonce, AES-GCM of the plaintext under the note key, a (random) signature. */
export async function envelope(s: Seal): Promise<Uint8Array> {
	counter += 1n;
	const head = header(s.kind, s.epoch, s.author ?? s.by.creds.accountId, s.device ?? s.by.device.id, counter);
	const nonce = random(24);
	const plaintext = typeof s.plaintext === "string" ? encoder.encode(s.plaintext) : (s.plaintext ?? random(s.size ?? 32));
	const key = await noteKey(s.shareId, s.epoch);
	const aad = new Uint8Array([...encoder.encode("kanso-collab"), 1, ...encoder.encode(s.shareId), ...head]);
	const sealed = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce.slice(0, 12), additionalData: aad }, key, plaintext));
	const out = new Uint8Array(head.length + 24 + sealed.length + 64);
	out.set(head, 0);
	out.set(nonce, head.length);
	out.set(sealed, head.length + 24);
	out.set(random(64), head.length + 24 + sealed.length);
	return out;
}

/** An envelope of exactly `total` bytes. */
export async function envelopeOfSize(s: Omit<Seal, "plaintext" | "size">, total: number): Promise<Uint8Array> {
	return envelope({ ...s, size: total - 86 - 24 - 16 - 64 });
}

export async function shareBody(shareId: string, owner: Account, others: Listed[], snapshot?: Uint8Array) {
	const members = [{ account: owner, role: "owner" as Role }, ...others];
	return {
		share_id: shareId,
		list: memberList(shareId, 1, owner, members),
		list_sig: b64(64),
		wraps: wrapsFor(members),
		snapshot: base64url(snapshot ?? (await envelope({ kind: "s", epoch: 1, shareId, by: owner }))),
	};
}

/** A new epoch's body; `fromSeq` must be the room's head (0 until anything is appended). */
export function epochBody(shareId: string, epoch: number, owner: Account, others: Listed[], fromSeq = 0) {
	const members = [{ account: owner, role: "owner" as Role }, ...others];
	return { epoch, list: memberList(shareId, epoch, owner, members, fromSeq), list_sig: b64(64), wraps: wrapsFor(members) };
}

const bytesCall = async (path: string, token: string, body: Uint8Array, headers: Record<string, string> = {}) => {
	const response = await call(path, { token, method: "POST", body, headers: { "content-type": "application/octet-stream", ...headers } });
	const text = await response.text();
	return { status: response.status, body: text === "" ? null : JSON.parse(text) };
};

/** Every /v2/collab route as the holder of `token`. */
export function collab(token: string) {
	const as = (path: string, init: Parameters<typeof callJson>[1] = {}) => callJson(path, { ...init, token });
	const base = (id: string) => `/v2/collab/shares/${id}`;
	return {
		token,
		identity: () => as("/v2/account/identity"),
		putIdentity: (body: unknown) => as("/v2/account/identity", { method: "PUT", json: body }),
		user: (name: string) => as(`/v2/users/${encodeURIComponent(name)}`),
		share: (body: unknown) => as("/v2/collab/shares", { json: body }),
		shares: () => as("/v2/collab/shares"),
		accept: (id: string) => as(`${base(id)}/accept`, { method: "POST" }),
		leave: (id: string) => as(`${base(id)}/membership`, { method: "DELETE" }),
		epoch: (id: string, body: unknown) => as(`${base(id)}/epoch`, { json: body }),
		stop: (id: string) => as(base(id), { method: "DELETE" }),
		log: (id: string, since?: number, limit?: number) => {
			const q = new URLSearchParams();
			if (since !== undefined) q.set("since", String(since));
			if (limit !== undefined) q.set("limit", String(limit));
			return as(`${base(id)}/log?${q}`);
		},
		append: (id: string, bytes: Uint8Array) => bytesCall(`${base(id)}/log`, token, bytes),
		snapshot: (id: string, baseSeq: number, bytes: Uint8Array) => bytesCall(`${base(id)}/snapshot`, token, bytes, { "x-kanso-base-seq": String(baseSeq) }),
		link: (id: string, body: unknown) => as(`${base(id)}/links`, { json: body }),
		unlink: (id: string, linkId: string) => as(`${base(id)}/links/${linkId}`, { method: "DELETE" }),
		join: (linkId: string, proof: string) => as(`/v2/collab/join/${linkId}`, { json: { proof } }),
		decline: (id: string, account: string) => as(`${base(id)}/requests/${account}`, { method: "DELETE" }),
		notify: (body: unknown) => as("/v2/collab/notifications", { json: body }),
		notifications: () => as("/v2/collab/notifications"),
		dismiss: (id: string) => as(`/v2/collab/notifications/${id}`, { method: "DELETE" }),
	};
}

export type Collab = ReturnType<typeof collab>;

/** Replaces `a`'s identity with fresh keys (a deleted-and-recreated account's, as far as the server can tell). */
export async function rekeyIdentity(a: Account): Promise<Identity> {
	await env.DB.prepare("DELETE FROM identities WHERE account_id = ?1").bind(a.creds.accountId).run();
	identities.delete(a.creds.accountId);
	return ensureIdentity(a);
}

/** Shares a note from `owner` with `others` (identities set first) and has everyone accept; the share ID. */
export async function shareWith(owner: Account, others: Listed[], accept = true): Promise<string> {
	await ensureIdentity(owner);
	for (const m of others) await ensureIdentity(m.account);
	const id = newShareId();
	const r = await collab(owner.token).share(await shareBody(id, owner, others));
	if (r.status !== 201) throw new Error(`share: ${r.status} ${JSON.stringify(r.body)}`);
	if (accept) for (const m of others) await collab(m.account.token).accept(id);
	return id;
}

// -------------------------------------------------------------------- sockets

export type Message = { text: any } | { bytes: Uint8Array };

export interface Socket {
	ws: WebSocket;
	/** The next message, waiting up to `ms`; throws when none comes. */
	next(ms?: number): Promise<Message>;
	/** Every message that arrives within `ms`. */
	drain(ms?: number): Promise<Message[]>;
	closed: Promise<{ code: number; reason: string }>;
}

/** Opens `…/live` as `token`; the response when it isn't upgraded. */
export async function openSocket(token: string, shareId: string, kind: "shares" | "books" = "shares"): Promise<Socket | Response> {
	return toSocket(await call(`/v2/collab/${kind}/${shareId}/live`, { token, headers: { upgrade: "websocket" } }));
}

/** An upgraded response as a socket with a message queue; the response when it isn't upgraded. */
export function toSocket(response: Response): Socket | Response {
	const ws = response.webSocket;
	if (response.status !== 101 || !ws) return response;
	const queue: Message[] = [];
	const waiters: ((m: Message) => void)[] = [];
	let closedResolve: (v: { code: number; reason: string }) => void = () => {};
	const closed = new Promise<{ code: number; reason: string }>((resolve) => (closedResolve = resolve));
	ws.accept();
	// The runtime's default is Blob for this compatibility date.
	ws.binaryType = "arraybuffer";
	ws.addEventListener("message", (event) => {
		const m: Message = typeof event.data === "string" ? { text: JSON.parse(event.data) } : { bytes: new Uint8Array(event.data as ArrayBuffer) };
		const waiter = waiters.shift();
		if (waiter) waiter(m);
		else queue.push(m);
	});
	ws.addEventListener("close", (event) => closedResolve({ code: event.code, reason: event.reason }));
	const next = (ms = 2000): Promise<Message> => {
		const queued = queue.shift();
		if (queued) return Promise.resolve(queued);
		return new Promise((resolve, reject) => {
			const timer = setTimeout(() => {
				const i = waiters.indexOf(done);
				if (i >= 0) waiters.splice(i, 1);
				reject(new Error("no message"));
			}, ms);
			const done = (m: Message) => {
				clearTimeout(timer);
				resolve(m);
			};
			waiters.push(done);
		});
	};
	const drain = async (ms = 200): Promise<Message[]> => {
		await new Promise((r) => setTimeout(r, ms));
		return queue.splice(0, queue.length);
	};
	return { ws, next, drain, closed };
}

/** A stored frame's seq and envelope. */
export function unframe(bytes: Uint8Array): { seq: number; envelope: Uint8Array } {
	return { seq: Number(new DataView(bytes.buffer, bytes.byteOffset).getBigUint64(0)), envelope: bytes.slice(8) };
}
