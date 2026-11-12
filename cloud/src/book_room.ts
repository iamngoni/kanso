// One shared notebook's live room (issue #390 › Shared notebooks: Worker): a
// Durable Object, keyed by the notebook's share ID, that keeps the notebook's
// logs in its own SQLite storage and relays envelopes between its members'
// open sockets. It is NoteRoom (room.ts) for many logs at once: the manifest
// and each note are "streams" (envelope.ts names them), each with its own
// snapshot, compaction and comment budget, all sharing one sequence number
// (`seq` counts every stored entry of the notebook, so it orders them and is
// what a new epoch's `from_seq` names) and one epoch (the notebook's).
//
// The room is reachable only through the Worker, which authenticates the
// session, reads the caller's role and the epoch from D1 and checks the
// envelope's header, then calls the room. The server never holds a key, so it
// never opens an envelope: entries are opaque ciphertext signed by clients.
// Entries are append-only: the only deletions are a snapshot replacing the
// updates it covers, and a purge when the notebook stops.
//
// Sockets (hibernation API) carry account, device, role and a socket ID.
// Binary frames in: `stream || envelope`; out: `stream || u64(seq) || envelope`.
// The manifest is always open. A socket opens a note with the text frame
// `{"type":"open","stream":"<22 base64url>"}` (answered `{"type":"opened",
// "stream","head"}`, `head` that stream's last seq so the client can catch up
// over HTTP), and closes it with `{"type":"close","stream"}`; at most 32 at a
// time. Stored entries go to the sockets that have the stream open (the
// sender must have it open); presence goes to the others only, never stored.
//
// Caps: 2,000 notes, comments 8 MiB per note, the day's bytes, and the owner's
// document-byte quota (their documents, shared notes and notebooks together).
// The room writes its size to `book_shares.bytes` as NoteRoom does.

import { DurableObject } from "cloudflare:workers";
import {
	BOOK_BYTES_PER_DAY,
	COMPACT_BYTES,
	COMPACT_ENTRIES,
	DEFAULT_LOG_LIMIT,
	FLUSH_BYTES,
	FLUSH_DELAY_MS,
	LOG_PAGE_BYTES,
	MANIFEST_STREAM,
	MAX_BOOK_NOTES,
	MAX_COMMENT_BYTES,
	MAX_ENVELOPE,
	MAX_OPEN_STREAMS,
	MAX_SOCKETS_PER_ACCOUNT,
	PRESENCE_BURST,
	PRESENCE_RATE,
	type Kind,
	type Role,
	SOCKET_BURST,
	SOCKET_RATE,
	SOCKETS_RETRY_MS,
	STREAM_BYTES,
	bookFrame,
	isManifest,
	mayWriteStream,
	parseEnvelope,
} from "./envelope";
import type { Env } from "./env";
import { rateLimited } from "./http";
import { base64url, fromBase64url } from "./protocol";
import { ROOM_HEADERS, type RoomRefusal, type RoomResult } from "./room";

const DAY = 86_400_000;
export const MANIFEST_ID = base64url(MANIFEST_STREAM);
/** The internal header carrying the notebook's share ID with a WebSocket upgrade. */
export const BOOK_SHARE_HEADER = "x-kanso-room-share";

export interface BookLogPage {
	/** The notebook's head: the seq of its last stored entry, across every stream. */
	head: number;
	/** The stream's last seq (0 for a stream with nothing). */
	stream_head: number;
	snapshot: { seq: number; base_seq: number; envelope: Uint8Array } | null;
	entries: { seq: number; envelope: Uint8Array }[];
	more: boolean;
}

interface Attachment {
	account: string;
	device: string;
	role: Role;
	sock: string;
}

type Meta = "head" | "epoch" | "day" | "day_bytes" | "log_bytes" | "flushed_bytes";

const refuse = (status: number, error: string, extras?: Record<string, number | string>): RoomRefusal => ({ ok: false, status, error, extras });
const blob = (bytes: Uint8Array): ArrayBuffer => bytes.slice().buffer;

export class BookRoom extends DurableObject<Env> {
	private readonly sql: SqlStorage;
	private readonly buckets = new Map<WebSocket, { tokens: number; at: number }>();
	private readonly presence = new Map<string, { tokens: number; at: number }>();

	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env);
		this.sql = ctx.storage.sql;
		this.schema();
	}

	private schema(): void {
		this.sql.exec(`CREATE TABLE IF NOT EXISTS log (
  seq INTEGER PRIMARY KEY, stream TEXT NOT NULL, kind TEXT NOT NULL, epoch INTEGER NOT NULL, author TEXT NOT NULL, envelope BLOB NOT NULL, at INTEGER NOT NULL)`);
		this.sql.exec("CREATE INDEX IF NOT EXISTS log_by_stream ON log (stream, seq)");
		this.sql.exec(`CREATE TABLE IF NOT EXISTS snapshot (
  stream TEXT PRIMARY KEY, seq INTEGER NOT NULL, base_seq INTEGER NOT NULL, epoch INTEGER NOT NULL,
  author TEXT NOT NULL, envelope BLOB NOT NULL, at INTEGER NOT NULL)`);
		this.sql.exec(`CREATE TABLE IF NOT EXISTS streams (
  stream TEXT PRIMARY KEY, head INTEGER NOT NULL, entries_since INTEGER NOT NULL, bytes_since INTEGER NOT NULL, comment_bytes INTEGER NOT NULL)`);
		this.sql.exec("CREATE TABLE IF NOT EXISTS subs (sock TEXT NOT NULL, stream TEXT NOT NULL, PRIMARY KEY (sock, stream))");
		this.sql.exec("CREATE TABLE IF NOT EXISTS meta (name TEXT PRIMARY KEY, value INTEGER NOT NULL)");
		this.sql.exec("CREATE TABLE IF NOT EXISTS info (id INTEGER PRIMARY KEY CHECK (id = 1), share_id TEXT NOT NULL)");
	}

	private get(name: Meta): number {
		const rows = this.sql.exec<{ value: number }>("SELECT value FROM meta WHERE name = ?", name).toArray();
		return rows.length === 0 ? 0 : rows[0].value;
	}

	private set(name: Meta, value: number): void {
		this.sql.exec("INSERT INTO meta (name, value) VALUES (?, ?) ON CONFLICT (name) DO UPDATE SET value = excluded.value", name, value);
	}

	private shareId(): string | null {
		return this.sql.exec<{ share_id: string }>("SELECT share_id FROM info WHERE id = 1").toArray()[0]?.share_id ?? null;
	}

	private adoptEpoch(epoch: number): number {
		const current = this.get("epoch");
		if (epoch > current) {
			this.set("epoch", epoch);
			return epoch;
		}
		return current;
	}

	private dayBytes(now: number): { day: number; used: number } {
		const day = Math.floor(now / DAY);
		const stored = this.get("day");
		return day > stored ? { day, used: 0 } : { day: stored, used: this.get("day_bytes") };
	}

	private overDay(now: number, size: number): RoomRefusal | null {
		const { used } = this.dayBytes(now);
		if (used + size <= BOOK_BYTES_PER_DAY) return null;
		return refuse(429, "rate_limited", { limit: "room-bytes", retry_after: Math.max(1, Math.ceil((DAY - (now % DAY)) / 1000)) });
	}

	private countDay(now: number, size: number): void {
		const { day, used } = this.dayBytes(now);
		this.set("day", day);
		this.set("day_bytes", used + size);
	}

	private storedBytes(): number {
		const snapshots = this.sql.exec<{ n: number }>("SELECT coalesce(sum(length(envelope)), 0) AS n FROM snapshot").one().n;
		return snapshots + this.get("log_bytes");
	}

	private streamRow(stream: string): { head: number; entries_since: number; bytes_since: number; comment_bytes: number } | null {
		return this.sql.exec<{ head: number; entries_since: number; bytes_since: number; comment_bytes: number }>("SELECT head, entries_since, bytes_since, comment_bytes FROM streams WHERE stream = ?", stream).toArray()[0] ?? null;
	}

	private send(ws: WebSocket, data: string | Uint8Array): void {
		try {
			ws.send(data);
		} catch {
			// Closing or gone: its close event cleans up.
		}
	}

	/** Sends to every socket that has `stream` open (the manifest is open on all). */
	private broadcast(stream: string, data: Uint8Array, except?: WebSocket): void {
		const open = new Set(this.sql.exec<{ sock: string }>("SELECT sock FROM subs WHERE stream = ?", stream).toArray().map((r) => r.sock));
		for (const ws of this.ctx.getWebSockets()) {
			if (ws === except) continue;
			if (stream === MANIFEST_ID || open.has((ws.deserializeAttachment() as Attachment).sock)) this.send(ws, data);
		}
	}

	// ------------------------------------------------------------- the Worker

	/** The notebook's head and epoch; both 0 for a room that holds nothing. */
	async state(): Promise<{ head: number; epoch: number }> {
		this.schema();
		return { head: this.get("head"), epoch: this.get("epoch") };
	}

	/**
	 * The owner's quota: their documents, shared notes, other notebooks (as D1
	 * last heard) and this room as it is now, plus `size`, within
	 * `quota_doc_bytes`. 404 when the notebook is gone from D1 (a stopped one).
	 */
	private async overQuota(shareId: string, size: number): Promise<RoomRefusal | null> {
		const row = await this.env.DB.prepare(
			`SELECT a.doc_bytes + coalesce((SELECT sum(c.bytes) FROM collab_shares c WHERE c.owner_id = s.owner_id), 0)
  + coalesce((SELECT sum(o.bytes) FROM book_shares o WHERE o.owner_id = s.owner_id AND o.id != s.id), 0) AS others,
  coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota
  FROM book_shares s JOIN accounts a ON a.id = s.owner_id, defaults d WHERE s.id = ?1 AND s.stopped_at IS NULL`,
		)
			.bind(shareId)
			.first<{ others: number; quota: number }>();
		if (!row) return refuse(404, "not_found");
		const used = row.others + this.storedBytes();
		return used + size > row.quota ? refuse(413, "quota_exceeded", { quota: "doc_bytes", used, limit: row.quota }) : null;
	}

	private async flush(threshold = FLUSH_BYTES): Promise<void> {
		const bytes = this.storedBytes();
		if (Math.abs(bytes - this.get("flushed_bytes")) < threshold) {
			if (bytes !== this.get("flushed_bytes") && (await this.ctx.storage.getAlarm()) === null) await this.ctx.storage.setAlarm(Date.now() + FLUSH_DELAY_MS);
			return;
		}
		const id = this.shareId();
		if (id === null) return;
		await this.env.DB.prepare("UPDATE book_shares SET bytes = ?2 WHERE id = ?1").bind(id, bytes).run();
		this.set("flushed_bytes", bytes);
	}

	override async alarm(): Promise<void> {
		this.schema();
		await this.flush(0);
	}

	/** Appends an update or comment the Worker has checked, then sends it to the sockets that have the stream open. */
	async append(input: { shareId: string; stream: string; epoch: number; envelope: Uint8Array; now: number }): Promise<RoomResult<{ seq: number }>> {
		this.schema();
		const header = parseEnvelope(input.envelope);
		if (!header || (header.kind !== "u" && header.kind !== "c")) return refuse(400, "bad_request");
		const epoch = this.adoptEpoch(input.epoch);
		if (header.epoch !== epoch) return refuse(409, "epoch_changed", { epoch });
		const stored = await this.store(input.shareId, input.stream, header.kind, header.epoch, header.author, input.envelope, input.now);
		if (!stored.ok) return stored;
		this.broadcast(input.stream, bookFrame(fromBase64url(input.stream) as Uint8Array, stored.seq, input.envelope));
		return stored;
	}

	private async store(shareId: string, stream: string, kind: Kind, epoch: number, author: string, envelope: Uint8Array, now: number): Promise<RoomResult<{ seq: number }>> {
		const row = this.streamRow(stream);
		if (row && (row.entries_since >= COMPACT_ENTRIES || row.bytes_since >= COMPACT_BYTES)) return refuse(409, "compact_first");
		const over = this.overDay(now, envelope.length);
		if (over) return over;
		if (kind === "c" && (row?.comment_bytes ?? 0) + envelope.length > MAX_COMMENT_BYTES) {
			return refuse(413, "quota_exceeded", { quota: "comment_bytes", used: row?.comment_bytes ?? 0, limit: MAX_COMMENT_BYTES });
		}
		const notes = this.noteCapRefusal(stream, row !== null);
		if (notes) return notes;
		const quota = await this.overQuota(shareId, envelope.length);
		if (quota) return quota;
		this.sql.exec("INSERT INTO info (id, share_id) VALUES (1, ?) ON CONFLICT (id) DO UPDATE SET share_id = excluded.share_id", shareId);
		const seq = this.get("head") + 1;
		this.sql.exec("INSERT INTO log (seq, stream, kind, epoch, author, envelope, at) VALUES (?, ?, ?, ?, ?, ?, ?)", seq, stream, kind, epoch, author, blob(envelope), now);
		this.sql.exec(
			`INSERT INTO streams (stream, head, entries_since, bytes_since, comment_bytes) VALUES (?1, ?2, 1, ?3, ?4)
  ON CONFLICT (stream) DO UPDATE SET head = ?2, entries_since = entries_since + 1, bytes_since = bytes_since + ?3, comment_bytes = comment_bytes + ?4`,
			stream,
			seq,
			envelope.length,
			kind === "c" ? envelope.length : 0,
		);
		this.set("head", seq);
		this.set("log_bytes", this.get("log_bytes") + envelope.length);
		this.countDay(now, envelope.length);
		await this.flush();
		return { ok: true, seq };
	}

	/** A new note stream past the 2,000-note cap is refused; the manifest and existing streams never are. */
	private noteCapRefusal(stream: string, exists: boolean): RoomRefusal | null {
		if (exists || stream === MANIFEST_ID) return null;
		const used = this.sql.exec<{ n: number }>("SELECT count(*) AS n FROM streams WHERE stream != ?", MANIFEST_ID).one().n;
		return used >= MAX_BOOK_NOTES ? refuse(413, "quota_exceeded", { quota: "notes", used, limit: MAX_BOOK_NOTES }) : null;
	}

	/**
	 * Takes a snapshot of a stream only while `baseSeq` is still that stream's
	 * last seq (409 `head_moved` with it otherwise; 0 for a stream with
	 * nothing). It replaces the stream's old snapshot and its updates up to
	 * `baseSeq`; comments stay. Its seq is `baseSeq`, and nothing's head moves.
	 */
	async snapshot(input: { shareId: string; stream: string; epoch: number; baseSeq: number; envelope: Uint8Array; now: number }): Promise<RoomResult<{ seq: number; bytes: number }>> {
		this.schema();
		const header = parseEnvelope(input.envelope);
		if (!header || header.kind !== "s") return refuse(400, "bad_request");
		const epoch = this.adoptEpoch(input.epoch);
		if (header.epoch !== epoch) return refuse(409, "epoch_changed", { epoch });
		const row = this.streamRow(input.stream);
		const head = row?.head ?? 0;
		if (input.baseSeq !== head) return refuse(409, "head_moved", { head });
		const over = this.overDay(input.now, input.envelope.length);
		if (over) return over;
		const notes = this.noteCapRefusal(input.stream, row !== null);
		if (notes) return notes;
		const quota = await this.overQuota(input.shareId, input.envelope.length - this.snapshotBytes(input.stream));
		if (quota) return quota;
		this.sql.exec("INSERT INTO info (id, share_id) VALUES (1, ?) ON CONFLICT (id) DO UPDATE SET share_id = excluded.share_id", input.shareId);
		this.sql.exec(
			`INSERT INTO snapshot (stream, seq, base_seq, epoch, author, envelope, at) VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)
  ON CONFLICT (stream) DO UPDATE SET seq = excluded.seq, base_seq = excluded.base_seq, epoch = excluded.epoch,
    author = excluded.author, envelope = excluded.envelope, at = excluded.at`,
			input.stream,
			input.baseSeq,
			header.epoch,
			header.author,
			blob(input.envelope),
			input.now,
		);
		this.sql.exec("DELETE FROM log WHERE stream = ? AND kind = 'u' AND seq <= ?", input.stream, input.baseSeq);
		this.sql.exec(
			`INSERT INTO streams (stream, head, entries_since, bytes_since, comment_bytes) VALUES (?1, ?2, 0, 0, 0)
  ON CONFLICT (stream) DO UPDATE SET entries_since = 0, bytes_since = 0`,
			input.stream,
			input.baseSeq,
		);
		this.set("log_bytes", this.sql.exec<{ n: number }>("SELECT coalesce(sum(length(envelope)), 0) AS n FROM log").one().n);
		this.countDay(input.now, input.envelope.length);
		await this.flush(0);
		return { ok: true, seq: input.baseSeq, bytes: this.storedBytes() };
	}

	private snapshotBytes(stream: string): number {
		return this.sql.exec<{ n: number }>("SELECT coalesce(sum(length(envelope)), 0) AS n FROM snapshot WHERE stream = ?", stream).one().n;
	}

	/**
	 * A page of one stream after `since`: its snapshot when `since` is below its
	 * base (or 0), then its stored updates and comments after `since`, at most
	 * `limit` and about LOG_PAGE_BYTES, always at least one.
	 */
	async read(input: { stream: string; since: number; limit?: number }): Promise<BookLogPage> {
		this.schema();
		const head = this.get("head");
		const streamHead = this.streamRow(input.stream)?.head ?? 0;
		const limit = input.limit ?? DEFAULT_LOG_LIMIT;
		let budget = LOG_PAGE_BYTES;
		let snapshot: BookLogPage["snapshot"] = null;
		const snap = this.sql.exec<{ seq: number; base_seq: number; envelope: ArrayBuffer }>("SELECT seq, base_seq, envelope FROM snapshot WHERE stream = ?", input.stream).toArray()[0];
		if (snap && (input.since === 0 || input.since < snap.base_seq)) {
			snapshot = { seq: snap.seq, base_seq: snap.base_seq, envelope: new Uint8Array(snap.envelope) };
			budget -= snap.envelope.byteLength;
		}
		const sizes = this.sql
			.exec<{ seq: number; size: number }>("SELECT seq, length(envelope) AS size FROM log WHERE stream = ? AND seq > ? ORDER BY seq LIMIT ?", input.stream, input.since, limit + 1)
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
						.exec<{ seq: number; envelope: ArrayBuffer }>("SELECT seq, envelope FROM log WHERE stream = ? AND seq > ? AND seq <= ? ORDER BY seq", input.stream, input.since, sizes[taken - 1].seq)
						.toArray()
						.map((r) => ({ seq: r.seq, envelope: new Uint8Array(r.envelope) }));
		return { head, stream_head: streamHead, snapshot, entries, more: taken < sizes.length };
	}

	/**
	 * A new epoch or a role change: the sockets of accounts no longer in
	 * `roles` (removed, or having left) get `{"type":"closed","reason":
	 * "removed"}` and are closed; the others take their role and get
	 * `{"type":"epoch","epoch"}`.
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
		this.schema();
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
			this.sql.exec("DELETE FROM subs WHERE sock = ?", (ws.deserializeAttachment() as Attachment).sock);
		} catch {
			// The room is being purged.
		}
		try {
			ws.close(1000, reason);
		} catch {
			// Already closing.
		}
	}

	// ------------------------------------------------------------ the sockets

	override async fetch(request: Request): Promise<Response> {
		this.schema();
		if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") return new Response(null, { status: 426 });
		const account = request.headers.get(ROOM_HEADERS.account) ?? "";
		const device = request.headers.get(ROOM_HEADERS.device) ?? "";
		const role = (request.headers.get(ROOM_HEADERS.role) ?? "") as Role;
		const known = Number(request.headers.get(ROOM_HEADERS.epoch) ?? "0");
		if (this.ctx.getWebSockets(account).length >= MAX_SOCKETS_PER_ACCOUNT) return rateLimited("sockets", SOCKETS_RETRY_MS);
		const share = request.headers.get(BOOK_SHARE_HEADER) ?? "";
		if (share === "") return new Response(null, { status: 400 });
		this.sql.exec("INSERT INTO info (id, share_id) VALUES (1, ?) ON CONFLICT (id) DO UPDATE SET share_id = excluded.share_id", share);
		const epoch = this.adoptEpoch(Number.isSafeInteger(known) ? known : 0);
		const pair = new WebSocketPair();
		const [client, server] = Object.values(pair);
		this.ctx.acceptWebSocket(server, [account]);
		const sock = base64url(crypto.getRandomValues(new Uint8Array(12)));
		server.serializeAttachment({ account, device, role, sock } satisfies Attachment);
		server.send(JSON.stringify({ type: "hello", head: this.get("head"), epoch }));
		return new Response(null, { status: 101, webSocket: client });
	}

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

	/** A text frame: open or close a note's stream on this socket. */
	private control(ws: WebSocket, attachment: Attachment, text: string): void {
		let m: unknown;
		try {
			m = JSON.parse(text);
		} catch {
			return this.refuseFrame(ws, "bad_request");
		}
		const { type, stream } = (m ?? {}) as { type?: unknown; stream?: unknown };
		const bytes = typeof stream === "string" ? fromBase64url(stream) : null;
		if ((type !== "open" && type !== "close") || !bytes || bytes.length !== STREAM_BYTES || isManifest(bytes)) return this.refuseFrame(ws, "bad_request");
		const id = stream as string;
		if (type === "close") {
			this.sql.exec("DELETE FROM subs WHERE sock = ? AND stream = ?", attachment.sock, id);
			return;
		}
		const open = this.sql.exec<{ n: number }>("SELECT count(*) AS n FROM subs WHERE sock = ?", attachment.sock).one().n;
		const already = this.sql.exec<{ n: number }>("SELECT count(*) AS n FROM subs WHERE sock = ? AND stream = ?", attachment.sock, id).one().n;
		if (!already && open >= MAX_OPEN_STREAMS) return this.refuseFrame(ws, "too_many_open", { stream: id, limit: MAX_OPEN_STREAMS });
		this.sql.exec("INSERT OR IGNORE INTO subs (sock, stream) VALUES (?, ?)", attachment.sock, id);
		this.send(ws, JSON.stringify({ type: "opened", stream: id, head: this.streamRow(id)?.head ?? 0 }));
	}

	/**
	 * One frame from a member: a text control frame, or `stream || envelope`
	 * (an update, a comment or presence), checked as the HTTP routes check an
	 * envelope (format, kind, epoch, author, role, size), then stored and sent
	 * to the stream's sockets, or, for presence, sent to the others only. A
	 * refused frame is dropped with an error frame.
	 */
	override async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
		const now = Date.now();
		this.schema();
		if (!this.take(this.buckets, ws, now, SOCKET_RATE, SOCKET_BURST)) return this.refuseFrame(ws, "rate_limited", { limit: "socket", retry_after: 1 });
		const attachment = ws.deserializeAttachment() as Attachment;
		if (typeof message === "string") return this.control(ws, attachment, message);
		const all = new Uint8Array(message);
		if (all.length <= STREAM_BYTES) return this.refuseFrame(ws, "bad_request");
		const streamBytes = all.subarray(0, STREAM_BYTES);
		const bytes = all.subarray(STREAM_BYTES);
		const stream = base64url(streamBytes);
		const header = parseEnvelope(bytes);
		if (!header || header.kind === "s") return this.refuseFrame(ws, "bad_request");
		const which = { stream, kind: header.kind, counter: header.counter.toString() };
		if (stream !== MANIFEST_ID && !this.sql.exec("SELECT 1 FROM subs WHERE sock = ? AND stream = ?", attachment.sock, stream).toArray().length) {
			return this.refuseFrame(ws, "not_open", which);
		}
		const epoch = this.get("epoch");
		if (header.epoch !== epoch) return this.refuseFrame(ws, "epoch_changed", { ...which, epoch });
		if (header.author !== attachment.account || header.device !== attachment.device) return this.refuseFrame(ws, "forbidden_author", which);
		if (!mayWriteStream(attachment.role, header.kind, isManifest(streamBytes))) return this.refuseFrame(ws, "forbidden_role", which);
		if (bytes.length > MAX_ENVELOPE[header.kind]) return this.refuseFrame(ws, "too_large", which);
		if (header.kind === "p") {
			if (!this.take(this.presence, attachment.account, now, PRESENCE_RATE, PRESENCE_BURST)) {
				return this.refuseFrame(ws, "rate_limited", { ...which, limit: "presence", retry_after: 1 });
			}
			this.broadcast(stream, bookFrame(streamBytes, 0, bytes), ws);
			return;
		}
		const shareId = this.shareId();
		if (shareId === null) return this.refuseFrame(ws, "not_found", which);
		const stored = await this.store(shareId, stream, header.kind, header.epoch, header.author, bytes, now);
		if (!stored.ok) return this.refuseFrame(ws, stored.error, { ...which, ...stored.extras });
		this.broadcast(stream, bookFrame(streamBytes, stored.seq, bytes));
	}

	override async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
		this.schema();
		this.buckets.delete(ws);
		this.sql.exec("DELETE FROM subs WHERE sock = ?", (ws.deserializeAttachment() as Attachment).sock);
		try {
			ws.close(code === 1005 || code === 1006 ? 1000 : code, reason);
		} catch {
			// Already closed.
		}
	}

	override async webSocketError(ws: WebSocket): Promise<void> {
		this.schema();
		this.buckets.delete(ws);
		this.sql.exec("DELETE FROM subs WHERE sock = ?", (ws.deserializeAttachment() as Attachment).sock);
	}
}

// ------------------------------------------------------ the Worker's helpers

export const bookRoomOf = (env: Env, shareId: string): DurableObjectStub<BookRoom> => env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(shareId));

/** Tells a notebook's room to close some accounts' sockets; a room that can't be reached closes them when they next fail. */
export async function closeBookSockets(env: Env, shareId: string, accounts: string[]): Promise<void> {
	if (accounts.length === 0) return;
	try {
		await bookRoomOf(env, shareId).removeMembers(accounts);
	} catch (err) {
		console.error(JSON.stringify({ route: "book", op: "remove_members", outcome: (err as Error)?.name ?? typeof err }));
	}
}

/** Purges one notebook's room and drops its purge row; false when the room couldn't be reached (the cron tries again). */
export async function purgeBookRoom(env: Env, shareId: string): Promise<boolean> {
	try {
		await bookRoomOf(env, shareId).purge();
	} catch (err) {
		console.error(JSON.stringify({ route: "book", op: "purge", outcome: (err as Error)?.name ?? typeof err }));
		return false;
	}
	await env.DB.prepare("DELETE FROM book_purges WHERE id = ?1").bind(shareId).run();
	return true;
}
