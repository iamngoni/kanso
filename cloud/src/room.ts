// One shared note's live room (docs/ACCOUNTS.md › Collaboration › The server:
// storage and the live room): a Durable Object, keyed by share ID, that keeps
// the note's log in its own SQLite storage, so a keystroke never writes to D1,
// and relays envelopes between the members' open sockets.
//
// The room is reachable only through the Worker (a Durable Object has no
// public address). The Worker authenticates the session, reads the caller's
// role and the share's epoch from D1, checks the envelope's header, and only
// then calls the room with them. On a new epoch, a removal or a stop the
// Worker tells the room, which updates or closes the affected sockets at once.
//
// Storage: `log` holds updates and comments (seq, kind, epoch, author,
// envelope, at); `snapshot` the one current snapshot; `meta` the head, the
// epoch, the entries and bytes since the snapshot, and the bytes stored this
// UTC day. A snapshot whose `base_seq` is still the head replaces every update
// up to it; comments are never compacted. A snapshot takes no sequence number
// of its own: its `seq` is its `base_seq`, and the head doesn't move. Live
// sockets aren't sent snapshots (they hold the state already).
//
// Sockets use the hibernation API: each carries its member's account, device
// and role as its attachment, so the room can sleep between messages. At most
// 8 sockets per account. Each socket's rate limit, and each account's presence
// budget, live in memory and start full again after a wake.
//
// Storage caps: comments (never compacted) at most 8 MiB per room, and every
// stored envelope must keep the owner's documents plus all their shared
// notes within their document-byte quota. The room reads the owner's figures
// from D1 on each append and writes its own size back to `collab_shares.bytes`
// once it has grown by 64 KiB, at a snapshot, and 30 seconds after any change.

import { DurableObject } from "cloudflare:workers";
import {
	COMPACT_BYTES,
	COMPACT_ENTRIES,
	DEFAULT_LOG_LIMIT,
	FLUSH_BYTES,
	FLUSH_DELAY_MS,
	MAX_COMMENT_BYTES,
	MAX_SOCKETS_PER_ACCOUNT,
	PRESENCE_BURST,
	PRESENCE_RATE,
	SOCKETS_RETRY_MS,
	type Kind,
	LOG_PAGE_BYTES,
	MAX_ENVELOPE,
	type Role,
	ROOM_BYTES_PER_DAY,
	SOCKET_BURST,
	SOCKET_RATE,
	frame,
	mayWrite,
	parseEnvelope,
} from "./envelope";
import type { Env } from "./env";
import { rateLimited } from "./http";

const DAY = 86_400_000;

/** A refusal the Worker turns into an HTTP error, or the room into an error frame. */
export interface RoomRefusal {
	ok: false;
	status: number;
	error: string;
	extras?: Record<string, number | string>;
}

export type RoomResult<T> = ({ ok: true } & T) | RoomRefusal;

export interface LogPage {
	head: number;
	snapshot: { seq: number; base_seq: number; envelope: Uint8Array } | null;
	entries: { seq: number; envelope: Uint8Array }[];
	more: boolean;
}

/** What a socket carries through hibernation. */
interface Attachment {
	account: string;
	device: string;
	role: Role;
}

/** The internal headers the Worker sends with a WebSocket upgrade. */
export const ROOM_HEADERS = {
	account: "x-kanso-room-account",
	device: "x-kanso-room-device",
	role: "x-kanso-room-role",
	epoch: "x-kanso-room-epoch",
} as const;

type Meta = "ready" | "head" | "epoch" | "entries_since" | "bytes_since" | "day" | "day_bytes" | "log_bytes" | "comment_bytes" | "flushed_bytes";

const refuse = (status: number, error: string, extras?: Record<string, number | string>): RoomRefusal => ({ ok: false, status, error, extras });

/** A copy of the bytes as a standalone ArrayBuffer, which SQLite stores as a BLOB. */
const blob = (bytes: Uint8Array): ArrayBuffer => bytes.slice().buffer;

export class NoteRoom extends DurableObject<Env> {
	private readonly sql: SqlStorage;
	/** Each socket's token bucket: frames left, and when it was last topped up. */
	private readonly buckets = new Map<WebSocket, { tokens: number; at: number }>();
	/** Each account's presence budget in this room, across its sockets. */
	private readonly presence = new Map<string, { tokens: number; at: number }>();

	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env);
		this.sql = ctx.storage.sql;
		this.schema();
	}

	private schema(): void {
		this.sql.exec(`CREATE TABLE IF NOT EXISTS log (
  seq INTEGER PRIMARY KEY, kind TEXT NOT NULL, epoch INTEGER NOT NULL, author TEXT NOT NULL, envelope BLOB NOT NULL, at INTEGER NOT NULL)`);
		this.sql.exec(`CREATE TABLE IF NOT EXISTS snapshot (
  id INTEGER PRIMARY KEY CHECK (id = 1), seq INTEGER NOT NULL, base_seq INTEGER NOT NULL, epoch INTEGER NOT NULL,
  author TEXT NOT NULL, envelope BLOB NOT NULL, at INTEGER NOT NULL)`);
		this.sql.exec("CREATE TABLE IF NOT EXISTS meta (name TEXT PRIMARY KEY, value INTEGER NOT NULL)");
		this.sql.exec("CREATE TABLE IF NOT EXISTS info (id INTEGER PRIMARY KEY CHECK (id = 1), share_id TEXT NOT NULL)");
	}

	private shareId(): string | null {
		return this.sql.exec<{ share_id: string }>("SELECT share_id FROM info WHERE id = 1").toArray()[0]?.share_id ?? null;
	}

	private get(name: Meta): number {
		const rows = this.sql.exec<{ value: number }>("SELECT value FROM meta WHERE name = ?", name).toArray();
		return rows.length === 0 ? 0 : rows[0].value;
	}

	private set(name: Meta, value: number): void {
		this.sql.exec("INSERT INTO meta (name, value) VALUES (?, ?) ON CONFLICT (name) DO UPDATE SET value = excluded.value", name, value);
	}

	/** The room's epoch: the highest it has been told, by a push or by a request the Worker checked against D1. */
	private adoptEpoch(epoch: number): number {
		const current = this.get("epoch");
		if (epoch > current) {
			this.set("epoch", epoch);
			return epoch;
		}
		return current;
	}

	/** Bytes stored today, counted towards the latest day seen (a request stamped earlier counts towards it too). */
	private dayBytes(now: number): { day: number; used: number } {
		const day = Math.floor(now / DAY);
		const stored = this.get("day");
		return day > stored ? { day, used: 0 } : { day: stored, used: this.get("day_bytes") };
	}

	private overDay(now: number, size: number): RoomRefusal | null {
		const { used } = this.dayBytes(now);
		if (used + size <= ROOM_BYTES_PER_DAY) return null;
		const retryAfter = Math.max(1, Math.ceil((DAY - (now % DAY)) / 1000));
		return refuse(429, "rate_limited", { limit: "room-bytes", retry_after: retryAfter });
	}

	private countDay(now: number, size: number): void {
		const { day, used } = this.dayBytes(now);
		this.set("day", day);
		this.set("day_bytes", used + size);
	}

	/** What the room stores: the snapshot and the log. */
	private storedBytes(): number {
		const snapshot = this.sql.exec<{ n: number }>("SELECT coalesce(sum(length(envelope)), 0) AS n FROM snapshot").one().n;
		return snapshot + this.get("log_bytes");
	}

	private send(ws: WebSocket, data: string | Uint8Array): void {
		try {
			ws.send(data);
		} catch {
			// Closing or gone: its close event cleans up.
		}
	}

	private broadcast(data: Uint8Array, except?: WebSocket): void {
		for (const ws of this.ctx.getWebSockets()) if (ws !== except) this.send(ws, data);
	}

	// ------------------------------------------------------------- the Worker

	/** Stores the first snapshot (base 0, head 0) of a new share. A second call (a retried share) changes nothing. */
	async init(input: { shareId: string; epoch: number; author: string; envelope: Uint8Array; now: number }): Promise<{ head: number; bytes: number }> {
		this.schema();
		if (this.get("ready") === 1) return { head: this.get("head"), bytes: this.storedBytes() };
		this.sql.exec(
			"INSERT INTO snapshot (id, seq, base_seq, epoch, author, envelope, at) VALUES (1, 0, 0, ?, ?, ?, ?)",
			input.epoch,
			input.author,
			blob(input.envelope),
			input.now,
		);
		this.set("head", 0);
		this.set("epoch", input.epoch);
		this.set("entries_since", 0);
		this.set("bytes_since", 0);
		this.set("log_bytes", 0);
		this.set("comment_bytes", 0);
		this.set("flushed_bytes", input.envelope.length);
		this.sql.exec("INSERT INTO info (id, share_id) VALUES (1, ?) ON CONFLICT (id) DO UPDATE SET share_id = excluded.share_id", input.shareId);
		this.countDay(input.now, input.envelope.length);
		this.set("ready", 1);
		return { head: 0, bytes: input.envelope.length };
	}

	/** The head and the epoch; both 0 for a room that holds nothing. */
	async state(): Promise<{ head: number; epoch: number }> {
		this.schema();
		return { head: this.get("head"), epoch: this.get("epoch") };
	}

	/**
	 * Appends an update or comment the Worker has checked (format, kind, author,
	 * role, size, and the epoch against D1's `epoch`), after the room's own
	 * checks: the epoch it knows, a snapshot due, the day's bytes. Every socket
	 * gets `u64(seq) || envelope`.
	 */
	async append(input: { epoch: number; envelope: Uint8Array; now: number }): Promise<RoomResult<{ seq: number }>> {
		this.schema();
		if (this.get("ready") !== 1) return refuse(404, "not_found");
		const header = parseEnvelope(input.envelope);
		if (!header || (header.kind !== "u" && header.kind !== "c")) return refuse(400, "bad_request");
		const epoch = this.adoptEpoch(input.epoch);
		if (header.epoch !== epoch) return refuse(409, "epoch_changed", { epoch });
		const stored = await this.store(header.kind, header.epoch, header.author, input.envelope, input.now);
		if (!stored.ok) return stored;
		this.broadcast(frame(stored.seq, input.envelope));
		return stored;
	}

	/**
	 * The owner's quota: their documents, their other shared notes as D1 last
	 * heard, and this room as it is now, plus `size`, within `quota_doc_bytes`.
	 * 413 `quota_exceeded` `{quota: "doc_bytes", used, limit}` otherwise; 404
	 * when the share is gone from D1.
	 */
	private async overQuota(size: number): Promise<RoomRefusal | null> {
		const id = this.shareId();
		if (id === null) return refuse(404, "not_found");
		const row = await this.env.DB.prepare(
			`SELECT a.doc_bytes + coalesce((SELECT sum(o.bytes) FROM collab_shares o WHERE o.owner_id = s.owner_id AND o.id != s.id), 0)
  + coalesce((SELECT sum(b.bytes) FROM book_shares b WHERE b.owner_id = s.owner_id), 0) AS others,
  coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota
  FROM collab_shares s JOIN accounts a ON a.id = s.owner_id, defaults d WHERE s.id = ?1`,
		)
			.bind(id)
			.first<{ others: number; quota: number }>();
		if (!row) return refuse(404, "not_found");
		const used = row.others + this.storedBytes();
		return used + size > row.quota ? refuse(413, "quota_exceeded", { quota: "doc_bytes", used, limit: row.quota }) : null;
	}

	/** Writes the room's size to `collab_shares.bytes` now when it has moved by `threshold` since the last write, else within 30 s. */
	private async flush(threshold = FLUSH_BYTES): Promise<void> {
		const bytes = this.storedBytes();
		if (Math.abs(bytes - this.get("flushed_bytes")) < threshold) {
			if (bytes !== this.get("flushed_bytes") && (await this.ctx.storage.getAlarm()) === null) await this.ctx.storage.setAlarm(Date.now() + FLUSH_DELAY_MS);
			return;
		}
		const id = this.shareId();
		if (id === null) return;
		await this.env.DB.prepare("UPDATE collab_shares SET bytes = ?2 WHERE id = ?1").bind(id, bytes).run();
		this.set("flushed_bytes", bytes);
	}

	override async alarm(): Promise<void> {
		this.schema();
		if (this.get("ready") === 1) await this.flush(0);
	}

	private async store(kind: Kind, epoch: number, author: string, envelope: Uint8Array, now: number): Promise<RoomResult<{ seq: number }>> {
		if (this.get("entries_since") >= COMPACT_ENTRIES || this.get("bytes_since") >= COMPACT_BYTES) {
			return refuse(409, "compact_first");
		}
		const over = this.overDay(now, envelope.length);
		if (over) return over;
		if (kind === "c" && this.get("comment_bytes") + envelope.length > MAX_COMMENT_BYTES) {
			return refuse(413, "quota_exceeded", { quota: "comment_bytes", used: this.get("comment_bytes"), limit: MAX_COMMENT_BYTES });
		}
		const quota = await this.overQuota(envelope.length);
		if (quota) return quota;
		const seq = this.get("head") + 1;
		this.sql.exec("INSERT INTO log (seq, kind, epoch, author, envelope, at) VALUES (?, ?, ?, ?, ?, ?)", seq, kind, epoch, author, blob(envelope), now);
		this.set("head", seq);
		this.set("entries_since", this.get("entries_since") + 1);
		this.set("bytes_since", this.get("bytes_since") + envelope.length);
		this.set("log_bytes", this.get("log_bytes") + envelope.length);
		if (kind === "c") this.set("comment_bytes", this.get("comment_bytes") + envelope.length);
		this.countDay(now, envelope.length);
		await this.flush();
		return { ok: true, seq };
	}

	/**
	 * Takes a snapshot only while `baseSeq` is still the head (409 `head_moved`
	 * with the head otherwise). It replaces the old snapshot and every update up
	 * to `baseSeq`; comments stay. Returns its seq, which is `baseSeq`, and the
	 * bytes the room now stores, which the Worker records in D1.
	 */
	async snapshot(input: { epoch: number; baseSeq: number; envelope: Uint8Array; now: number }): Promise<RoomResult<{ seq: number; bytes: number }>> {
		this.schema();
		if (this.get("ready") !== 1) return refuse(404, "not_found");
		const header = parseEnvelope(input.envelope);
		if (!header || header.kind !== "s") return refuse(400, "bad_request");
		const epoch = this.adoptEpoch(input.epoch);
		if (header.epoch !== epoch) return refuse(409, "epoch_changed", { epoch });
		const head = this.get("head");
		if (input.baseSeq !== head) return refuse(409, "head_moved", { head });
		const over = this.overDay(input.now, input.envelope.length);
		if (over) return over;
		const seq = input.baseSeq;
		this.sql.exec(
			`INSERT INTO snapshot (id, seq, base_seq, epoch, author, envelope, at) VALUES (1, ?, ?, ?, ?, ?, ?)
  ON CONFLICT (id) DO UPDATE SET seq = excluded.seq, base_seq = excluded.base_seq, epoch = excluded.epoch,
    author = excluded.author, envelope = excluded.envelope, at = excluded.at`,
			seq,
			input.baseSeq,
			header.epoch,
			header.author,
			blob(input.envelope),
			input.now,
		);
		this.sql.exec("DELETE FROM log WHERE kind = 'u' AND seq <= ?", input.baseSeq);
		const logBytes = this.sql.exec<{ n: number }>("SELECT coalesce(sum(length(envelope)), 0) AS n FROM log").one().n;
		this.set("log_bytes", logBytes);
		this.set("entries_since", 0);
		this.set("bytes_since", 0);
		this.countDay(input.now, input.envelope.length);
		await this.flush(0);
		return { ok: true, seq, bytes: input.envelope.length + logBytes };
	}

	/**
	 * A page of the log after `since`: the snapshot when `since` is below its
	 * base, or 0 (a reader starting from nothing always needs it), then the stored updates and comments after `since` (updates before the
	 * snapshot are gone), at most `limit` and about LOG_PAGE_BYTES of envelopes
	 * with the snapshot, always at least one.
	 */
	async read(input: { since: number; limit?: number }): Promise<LogPage> {
		this.schema();
		const head = this.get("head");
		if (this.get("ready") !== 1) return { head: 0, snapshot: null, entries: [], more: false };
		const limit = input.limit ?? DEFAULT_LOG_LIMIT;
		let budget = LOG_PAGE_BYTES;
		let snapshot: LogPage["snapshot"] = null;
		const snap = this.sql.exec<{ seq: number; base_seq: number; envelope: ArrayBuffer }>("SELECT seq, base_seq, envelope FROM snapshot WHERE id = 1").toArray()[0];
		if (snap && (input.since === 0 || input.since < snap.base_seq)) {
			snapshot = { seq: snap.seq, base_seq: snap.base_seq, envelope: new Uint8Array(snap.envelope) };
			budget -= snap.envelope.byteLength;
		}
		const sizes = this.sql
			.exec<{ seq: number; size: number }>("SELECT seq, length(envelope) AS size FROM log WHERE seq > ? ORDER BY seq LIMIT ?", input.since, limit + 1)
			.toArray();
		let taken = 0;
		for (const row of sizes.slice(0, limit)) {
			if (taken > 0 && row.size > budget) break;
			budget -= row.size;
			taken += 1;
		}
		const entries =
			taken === 0
				? []
				: this.sql
						.exec<{ seq: number; envelope: ArrayBuffer }>("SELECT seq, envelope FROM log WHERE seq > ? AND seq <= ? ORDER BY seq", input.since, sizes[taken - 1].seq)
						.toArray()
						.map((r) => ({ seq: r.seq, envelope: new Uint8Array(r.envelope) }));
		return { head, snapshot, entries, more: taken < sizes.length };
	}

	/**
	 * A new epoch: the sockets of accounts no longer in `roles` get
	 * `{"type":"closed","reason":"removed"}` and are closed; the others take
	 * their new role and get `{"type":"epoch","epoch"}`.
	 */
	async setEpoch(input: { epoch: number; roles: Record<string, Role> }): Promise<void> {
		this.schema();
		const epoch = this.adoptEpoch(input.epoch);
		for (const ws of this.ctx.getWebSockets()) {
			const attachment = ws.deserializeAttachment() as Attachment;
			const role = Object.hasOwn(input.roles, attachment.account) ? input.roles[attachment.account] : undefined;
			if (role === undefined) {
				this.close(ws, "removed");
				continue;
			}
			ws.serializeAttachment({ ...attachment, role } satisfies Attachment);
			this.send(ws, JSON.stringify({ type: "epoch", epoch }));
		}
	}

	/** Closes the sockets of accounts that left or were deleted. */
	async removeMembers(accounts: string[]): Promise<void> {
		for (const account of accounts) {
			for (const ws of this.ctx.getWebSockets(account)) this.close(ws, "removed");
		}
	}

	/** Sharing stopped: every socket hears so and is closed, and everything the room stores is deleted. */
	async purge(): Promise<void> {
		for (const ws of this.ctx.getWebSockets()) this.close(ws, "stopped");
		this.buckets.clear();
		this.presence.clear();
		await this.ctx.storage.deleteAlarm();
		await this.ctx.storage.deleteAll();
	}

	private close(ws: WebSocket, reason: "removed" | "stopped"): void {
		this.send(ws, JSON.stringify({ type: "closed", reason }));
		this.buckets.delete(ws);
		try {
			ws.close(1000, reason);
		} catch {
			// Already closing.
		}
	}

	// ------------------------------------------------------------ the sockets

	/** The WebSocket upgrade, from the Worker, with the member's account, device, role and the epoch it read. */
	override async fetch(request: Request): Promise<Response> {
		this.schema();
		if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") return new Response(null, { status: 426 });
		const account = request.headers.get(ROOM_HEADERS.account) ?? "";
		const device = request.headers.get(ROOM_HEADERS.device) ?? "";
		const role = (request.headers.get(ROOM_HEADERS.role) ?? "") as Role;
		const known = Number(request.headers.get(ROOM_HEADERS.epoch) ?? "0");
		if (this.get("ready") !== 1) return new Response(null, { status: 404 });
		if (this.ctx.getWebSockets(account).length >= MAX_SOCKETS_PER_ACCOUNT) {
			return rateLimited("sockets", SOCKETS_RETRY_MS);
		}
		const epoch = this.adoptEpoch(Number.isSafeInteger(known) ? known : 0);
		const pair = new WebSocketPair();
		const [client, server] = Object.values(pair);
		// Tagged with the account, so a removal finds its sockets.
		this.ctx.acceptWebSocket(server, [account]);
		server.serializeAttachment({ account, device, role } satisfies Attachment);
		server.send(JSON.stringify({ type: "hello", head: this.get("head"), epoch }));
		return new Response(null, { status: 101, webSocket: client });
	}

	/** Takes one token from `key`'s bucket in `buckets` (`rate` a second, `burst` at most); false when it is empty. */
	private take<K>(buckets: Map<K, { tokens: number; at: number }>, key: K, now: number, rate: number, burst: number): boolean {
		const bucket = buckets.get(key) ?? { tokens: burst, at: now };
		bucket.tokens = Math.min(burst, bucket.tokens + ((now - bucket.at) / 1000) * rate);
		bucket.at = now;
		buckets.set(key, bucket);
		if (bucket.tokens < 1) return false;
		bucket.tokens -= 1;
		return true;
	}

	private refuseFrame(ws: WebSocket, error: string, extras: Record<string, unknown> = {}): void {
		this.send(ws, JSON.stringify({ type: "error", error, ...extras }));
	}

	/**
	 * One frame from a member: an update, a comment or presence. Checked as the
	 * HTTP routes check an envelope (format, kind, epoch, author, role, size),
	 * then stored and sent to every socket, or, for presence, sent to the others
	 * only and never stored. A refused frame is dropped with an error frame.
	 */
	override async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
		const now = Date.now();
		if (!this.take(this.buckets, ws, now, SOCKET_RATE, SOCKET_BURST)) return this.refuseFrame(ws, "rate_limited", { limit: "socket", retry_after: 1 });
		if (typeof message === "string") return this.refuseFrame(ws, "bad_request");
		const attachment = ws.deserializeAttachment() as Attachment;
		const bytes = new Uint8Array(message);
		const header = parseEnvelope(bytes);
		if (!header || header.kind === "s") return this.refuseFrame(ws, "bad_request");
		const which = { kind: header.kind, counter: header.counter.toString() };
		const epoch = this.get("epoch");
		if (header.epoch !== epoch) return this.refuseFrame(ws, "epoch_changed", { ...which, epoch });
		if (header.author !== attachment.account || header.device !== attachment.device) return this.refuseFrame(ws, "forbidden_author", which);
		if (!mayWrite(attachment.role, header.kind)) return this.refuseFrame(ws, "forbidden_role", which);
		if (bytes.length > MAX_ENVELOPE[header.kind]) return this.refuseFrame(ws, "too_large", which);
		if (header.kind === "p") {
			if (!this.take(this.presence, attachment.account, now, PRESENCE_RATE, PRESENCE_BURST)) {
				return this.refuseFrame(ws, "rate_limited", { ...which, limit: "presence", retry_after: 1 });
			}
			this.broadcast(frame(0, bytes), ws);
			return;
		}
		const stored = await this.store(header.kind, header.epoch, header.author, bytes, now);
		if (!stored.ok) return this.refuseFrame(ws, stored.error, { ...which, ...stored.extras });
		this.broadcast(frame(stored.seq, bytes));
	}

	override async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
		this.buckets.delete(ws);
		try {
			ws.close(code === 1005 || code === 1006 ? 1000 : code, reason);
		} catch {
			// Already closed.
		}
	}

	override async webSocketError(ws: WebSocket): Promise<void> {
		this.buckets.delete(ws);
	}
}
