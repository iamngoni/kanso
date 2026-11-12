// Shared notebooks: the manifest and note logs and the live room (issue #390 › Shared notebooks:
// Worker; slice 34). Over HTTP: appending, reading and compacting each log, roles per entry kind,
// membership (stranger, invitee, someone who left), the caps, rate limits, append-only, a new epoch's
// from_seq checked against the real head, stopping. Over the room's WebSockets: two clients getting
// each other's envelopes per note, presence, a removed member kicked on a new epoch, stopping.

import { runCollabPurges } from "../src/collab";
import { runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { MANIFEST_ID } from "../src/book_room";
import { type Socket, envelope, envelopeOfSize, openSocket } from "./collab-helpers";
import { type Book, advance, books, newShareId, shareBook } from "./book-helpers";
import { type Account, base64url, call, clock, resetServer, signUp } from "./helpers";

let alice: Account;
let bob: Account;
let carol: Account;
let dave: Account;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	bob = await signUp("bob");
	carol = await signUp("carol");
	dave = await signUp("dave");
});

const NOTE = newShareId();
const NOTE2 = newShareId();
const ZERO = MANIFEST_ID;

const path = (id: string, stream: string | null, tail = "") => `/v2/collab/books/${id}/${stream === null ? "manifest" : `notes/${stream}`}${tail}`;

async function post(a: Account, url: string, body: Uint8Array, headers: Record<string, string> = {}) {
	const response = await call(url, { token: a.token, method: "POST", body, headers: { "content-type": "application/octet-stream", ...headers } });
	const text = await response.text();
	return { status: response.status, body: text === "" ? null : JSON.parse(text) };
}

const append = (a: Account, id: string, stream: string | null, bytes: Uint8Array) => post(a, path(id, stream), bytes);
const snapshot = (a: Account, id: string, stream: string | null, base: number, bytes: Uint8Array) => post(a, path(id, stream, "/snapshot"), bytes, { "x-kanso-base-seq": String(base) });
const read = async (a: Account, id: string, stream: string | null, query = "") => {
	const response = await call(`${path(id, stream)}${query}`, { token: a.token });
	return { status: response.status, body: (await response.json()) as any };
};

const seal = (book: Book, kind: "u" | "s" | "c" | "p", by: Account, epoch = book.epoch) => envelope({ kind, epoch, shareId: book.id, by });

/** Alice owns, bob edits, carol comments, dave views; all accepted. */
const team = () =>
	shareBook(alice, [
		{ account: bob, role: "edit" },
		{ account: carol, role: "comment" },
		{ account: dave, role: "view" },
	]);

describe("the logs over HTTP", () => {
	it("keeps the manifest and each note's log apart, with one seq count across them", async () => {
		const book = await team();
		const m = await seal(book, "u", alice);
		const n1 = await seal(book, "u", bob);
		const n2 = await seal(book, "u", bob);
		expect((await append(alice, book.id, null, m)).body).toEqual({ seq: 1 });
		expect((await append(bob, book.id, NOTE, n1)).body).toEqual({ seq: 2 });
		expect((await append(bob, book.id, NOTE2, n2)).body).toEqual({ seq: 3 });
		expect((await read(dave, book.id, null)).body).toMatchObject({ head: 3, stream_head: 1, entries: [{ seq: 1, envelope: base64url(m) }], more: false });
		expect((await read(dave, book.id, NOTE)).body).toMatchObject({ head: 3, stream_head: 2, entries: [{ seq: 2, envelope: base64url(n1) }] });
		expect((await read(dave, book.id, NOTE, "?since=2")).body.entries).toEqual([]);
		expect((await read(dave, book.id, NOTE2, "?since=1&limit=1")).body.entries).toEqual([{ seq: 3, envelope: base64url(n2) }]);
	});

	it("serves ciphertext only: a plaintext marker the writer sealed is nowhere in the answer", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await envelope({ kind: "u", epoch: 1, shareId: book.id, by: bob, plaintext: "SECRET-MARKER-xyz" }));
		const page = JSON.stringify((await read(alice, book.id, NOTE)).body);
		expect(page).not.toContain("SECRET-MARKER");
	});

	it("needs a session (401)", async () => {
		const book = await team();
		expect((await call(path(book.id, null))).status).toBe(401);
		expect((await call(path(book.id, null), { method: "POST", body: new Uint8Array(1), headers: { "content-type": "application/octet-stream" } })).status).toBe(401);
	});

	it("is for accepted members only: a stranger, an invitee and a bad ID are refused", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }], false);
		const e = await seal(book, "u", alice);
		expect((await append(carol, book.id, NOTE, e)).status).toBe(404);
		expect((await read(carol, book.id, NOTE)).status).toBe(404);
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(404);
		expect((await read(bob, book.id, null)).status).toBe(404);
		expect((await read(alice, "short", null)).status).toBe(400);
		expect((await append(alice, newShareId(), NOTE, e)).status).toBe(404);
		await books(bob.token).accept(book.id);
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
	});

	it("shuts out someone who left, though the signed list still names them", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		expect((await books(bob.token).leave(book.id)).status).toBe(204);
		expect((await read(bob, book.id, NOTE)).status).toBe(404);
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(404);
		expect(((await openSocket(bob.token, book.id, "books")) as Response).status).toBe(404);
	});

	it("gives each role what its kind allows: view only presence, comment comments, edit and up updates and snapshots", async () => {
		const book = await team();
		// dave views: nothing stored
		expect((await append(dave, book.id, NOTE, await seal(book, "u", dave))).status).toBe(403);
		expect((await append(dave, book.id, NOTE, await seal(book, "c", dave))).status).toBe(403);
		// carol comments: comments on notes, not updates, not on the manifest
		expect((await append(carol, book.id, NOTE, await seal(book, "u", carol))).status).toBe(403);
		expect((await append(carol, book.id, NOTE, await seal(book, "c", carol))).status).toBe(200);
		expect((await append(carol, book.id, null, await seal(book, "c", carol))).status).toBe(400);
		expect((await snapshot(carol, book.id, NOTE, 2, await seal(book, "s", carol))).status).toBe(403);
		// bob edits
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		expect((await append(bob, book.id, NOTE, await seal(book, "c", bob))).status).toBe(200);
		expect((await append(bob, book.id, null, await seal(book, "u", bob))).status).toBe(200);
		expect((await append(bob, book.id, null, await seal(book, "c", bob))).status).toBe(400);
		// the owner writes too; presence is never stored over HTTP
		expect((await append(alice, book.id, NOTE, await seal(book, "u", alice))).status).toBe(200);
		expect((await append(alice, book.id, NOTE, await seal(book, "p", alice))).status).toBe(400);
	});

	it("lets a manager write, as the role table says", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "manager" }]);
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		expect((await append(bob, book.id, null, await seal(book, "u", bob))).status).toBe(200);
	});

	it("checks the envelope: author and device are the session's, the epoch is the notebook's, the size is capped, the shape is real", async () => {
		const book = await team();
		const other = await envelope({ kind: "u", epoch: 1, shareId: book.id, by: bob, author: alice.creds.accountId });
		expect((await append(bob, book.id, NOTE, other)).body.error).toBe("forbidden_author");
		const device = await envelope({ kind: "u", epoch: 1, shareId: book.id, by: bob, device: `d_${"0".repeat(32)}` });
		expect((await append(bob, book.id, NOTE, device)).body.error).toBe("forbidden_author");
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 2))).body).toMatchObject({ error: "epoch_changed", epoch: 1 });
		expect((await append(bob, book.id, NOTE, await envelopeOfSize({ kind: "u", epoch: 1, shareId: book.id, by: bob }, 256 * 1024 + 1))).status).toBe(413);
		expect((await append(bob, book.id, NOTE, await envelopeOfSize({ kind: "u", epoch: 1, shareId: book.id, by: bob }, 256 * 1024))).status).toBe(200);
		expect((await append(bob, book.id, NOTE, new Uint8Array(200))).status).toBe(400);
		expect((await append(bob, book.id, NOTE, await seal(book, "s", bob))).status).toBe(400);
		expect((await read(bob, book.id, NOTE, "?limit=1001")).status).toBe(400);
	});

	it("refuses a note ID that isn't 16 bytes of base64url or is the manifest's all zeros", async () => {
		const book = await team();
		const e = await seal(book, "u", bob);
		expect((await append(bob, book.id, "not-a-note", e)).status).toBe(400);
		expect((await append(bob, book.id, ZERO, e)).status).toBe(400);
		expect((await read(bob, book.id, ZERO)).status).toBe(400);
	});
});

describe("snapshots", () => {
	it("replace a log's updates up to the base, only while the base is that log's last seq", async () => {
		const book = await team();
		for (let i = 0; i < 3; i++) await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await append(bob, book.id, NOTE2, await seal(book, "u", bob));
		const comment = await seal(book, "c", carol);
		await append(carol, book.id, NOTE, comment);
		expect((await snapshot(bob, book.id, NOTE, 3, await seal(book, "s", bob))).body).toMatchObject({ error: "head_moved", head: 5 });
		const s = await seal(book, "s", bob);
		expect((await snapshot(bob, book.id, NOTE, 5, s)).body).toEqual({ seq: 5 });
		const page = (await read(carol, book.id, NOTE)).body;
		expect(page.snapshot).toEqual({ seq: 5, base_seq: 5, envelope: base64url(s) });
		expect(page.entries).toEqual([{ seq: 5, envelope: base64url(comment) }]);
		// a reader past the base needs no snapshot; the other note is untouched
		expect((await read(carol, book.id, NOTE, "?since=5")).body.snapshot).toBeNull();
		expect((await read(carol, book.id, NOTE2)).body).toMatchObject({ snapshot: null, entries: [{ seq: 4 }] });
		expect((await snapshot(bob, book.id, NOTE2, 4, await seal(book, "s", bob))).status).toBe(200);
	});

	it("start a log: base 0 for a log with nothing, and the head doesn't move", async () => {
		const book = await team();
		expect((await snapshot(alice, book.id, null, 0, await seal(book, "s", alice))).body).toEqual({ seq: 0 });
		expect((await read(bob, book.id, null)).body).toMatchObject({ head: 0, stream_head: 0, entries: [], snapshot: { seq: 0, base_seq: 0 } });
		expect((await append(alice, book.id, null, await seal(book, "u", alice))).body).toEqual({ seq: 1 });
		expect((await snapshot(alice, book.id, null, 0, await seal(book, "s", alice))).body.error).toBe("head_moved");
		expect((await snapshot(alice, book.id, null, -1 as number, await seal(book, "s", alice))).status).toBe(400);
	});

	it("need the right kind, role, epoch and base header", async () => {
		const book = await team();
		expect((await snapshot(dave, book.id, NOTE, 0, await seal(book, "s", dave))).status).toBe(403);
		expect((await snapshot(bob, book.id, NOTE, 0, await seal(book, "u", bob))).status).toBe(400);
		expect((await snapshot(bob, book.id, NOTE, 0, await seal(book, "s", bob, 3))).body.error).toBe("epoch_changed");
		expect((await post(bob, path(book.id, NOTE, "/snapshot"), await seal(book, "s", bob))).status).toBe(400);
	});
});

describe("append-only", () => {
	it("has no way to change or delete an entry: stored seqs and bytes stay as written, and other methods are refused", async () => {
		const book = await team();
		const first = await seal(book, "u", bob);
		await append(bob, book.id, NOTE, first);
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		for (const method of ["PUT", "PATCH", "DELETE"]) {
			const r = await call(path(book.id, NOTE), { token: bob.token, method });
			expect(r.status).toBeGreaterThanOrEqual(400);
		}
		const page = (await read(alice, book.id, NOTE)).body;
		expect(page.entries.map((e: any) => e.seq)).toEqual([1, 2]);
		expect(page.entries[0].envelope).toBe(base64url(first));
		// a replay of the same bytes is a new, later entry, never an overwrite
		expect((await append(bob, book.id, NOTE, first)).body).toEqual({ seq: 3 });
		expect((await read(alice, book.id, NOTE)).body.entries[0].envelope).toBe(base64url(first));
	});

	it("comments survive compaction", async () => {
		const book = await team();
		await append(carol, book.id, NOTE, await seal(book, "c", carol));
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await snapshot(bob, book.id, NOTE, 2, await seal(book, "s", bob));
		expect((await read(alice, book.id, NOTE)).body.entries).toHaveLength(1);
	});
});

describe("caps and limits", () => {
	it("refuses a 2,001st note (413 notes) but not more entries to a note it holds, or the manifest", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), (_room, state) => {
			for (let i = 0; i < 1999; i++) {
				state.storage.sql.exec("INSERT INTO streams (stream, head, entries_since, bytes_since, comment_bytes) VALUES (?, 1, 0, 0, 0)", `filler-${i}`);
			}
		});
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		expect((await append(bob, book.id, null, await seal(book, "u", bob))).status).toBe(200);
		expect((await append(bob, book.id, NOTE2, await seal(book, "u", bob))).body).toMatchObject({ error: "quota_exceeded", quota: "notes", limit: 2000 });
		expect((await snapshot(bob, book.id, NOTE2, 0, await seal(book, "s", bob))).status).toBe(413);
	});

	it("caps a note's comments at 8 MiB (413 comment_bytes)", async () => {
		const book = await team();
		await append(carol, book.id, NOTE, await seal(book, "c", carol));
		await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), (_room, state) => {
			state.storage.sql.exec("UPDATE streams SET comment_bytes = ?", 8 * 1024 * 1024 - 10);
		});
		expect((await append(carol, book.id, NOTE, await seal(book, "c", carol))).body).toMatchObject({ error: "quota_exceeded", quota: "comment_bytes" });
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
	});

	it("asks for a snapshot first once a log has 10,000 entries since the last (409 compact_first)", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), (_room, state) => {
			state.storage.sql.exec("UPDATE streams SET entries_since = 10000");
		});
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).body.error).toBe("compact_first");
		expect((await append(bob, book.id, NOTE2, await seal(book, "u", bob))).status).toBe(200);
	});

	it("keeps the owner within their document bytes, counting every notebook they own (413 doc_bytes)", async () => {
		const book = await team();
		const big = await envelopeOfSize({ kind: "u", epoch: 1, shareId: book.id, by: bob }, 1000);
		await env.DB.prepare("UPDATE accounts SET doc_bytes = 0, quota_doc_bytes = 1500 WHERE id = ?1").bind(alice.creds.accountId).run();
		expect((await append(bob, book.id, NOTE, big)).status).toBe(200);
		expect((await append(bob, book.id, NOTE, big)).body).toMatchObject({ error: "quota_exceeded", quota: "doc_bytes", limit: 1500 });
		// the room's size reaches D1, so the owner's other notebook sees it
		const other = await shareBook(alice, []);
		await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), async (room) => {
			await (room as any).flush(0);
		});
		expect((await env.DB.prepare("SELECT bytes FROM book_shares WHERE id = ?1").bind(book.id).first<{ bytes: number }>())!.bytes).toBeGreaterThan(900);
		expect((await append(alice, other.id, NOTE, await envelopeOfSize({ kind: "u", epoch: 1, shareId: other.id, by: alice }, 700))).body.error).toBe("quota_exceeded");
	});

	it("counts every log read and write per account: the 30,001st is 429", async () => {
		const book = await team();
		await env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, 30000)")
			.bind(`book-log-account:${bob.creds.accountId}`, Math.floor(clock.now / 3_600_000) * 3_600_000)
			.run();
		expect(await read(bob, book.id, NOTE)).toMatchObject({ status: 429, body: { limit: "book-log-account" } });
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(429);
		expect(((await openSocket(bob.token, book.id, "books")) as Response).status).toBe(429);
		expect((await read(carol, book.id, NOTE)).status).toBe(200);
	});

	it("limits the bytes a notebook's room stores a day (429 room-bytes)", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), (_room, state) => {
			state.storage.sql.exec("INSERT INTO meta (name, value) VALUES ('day_bytes', 256 * 1024 * 1024) ON CONFLICT (name) DO UPDATE SET value = excluded.value");
		});
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).body).toMatchObject({ error: "rate_limited", limit: "room-bytes" });
	});
});

describe("a new epoch's from_seq", () => {
	it("must be the notebook's real log head (409 head_moved), and then the list is accepted", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await append(alice, book.id, null, await seal(book, "u", alice));
		const members = [
			{ account: alice, role: "owner" as const },
			{ account: bob, role: "edit" as const },
			{ account: carol, role: "comment" as const },
		];
		expect(await advance(book, alice, members, { newEpoch: true, fromSeq: 0 })).toMatchObject({ status: 409, body: { error: "head_moved", head: 2 } });
		expect(await advance(book, alice, members, { newEpoch: true, fromSeq: 3 })).toMatchObject({ status: 409, body: { head: 2 } });
		expect(await advance(book, alice, members, { newEpoch: true, fromSeq: 2 })).toMatchObject({ status: 200, body: { epoch: 2 } });
		// the room took the epoch: an old-epoch envelope is refused, a new one stored
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 1))).body.error).toBe("epoch_changed");
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 2))).body).toEqual({ seq: 3 });
	});

	it("is not checked for a list in the same epoch", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		const members = [...book.members, { account: dave, role: "view" as const }].filter((m, i, all) => all.findIndex((x) => x.account === m.account) === i);
		expect((await advance(book, alice, members, { wrapFor: [] })).status).toBe(200);
	});
});

describe("stopping", () => {
	it("purges the room: logs gone, the ID retired, the purge row dropped, reads 404", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		const room = env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id));
		expect(await room.state()).toMatchObject({ head: 1 });
		expect((await books(alice.token).stop(book.id)).status).toBe(204);
		expect(await room.state()).toEqual({ head: 0, epoch: 0 });
		expect((await room.read({ stream: base64url(new Uint8Array(16).fill(1)), since: 0 })).entries).toEqual([]);
		expect((await env.DB.prepare("SELECT count(*) AS n FROM book_purges").first<{ n: number }>())!.n).toBe(0);
		expect((await read(bob, book.id, NOTE)).status).toBe(404);
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(404);
	});

	it("is left to the cron when the room can't be reached right away", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await env.DB.prepare("INSERT INTO book_purges (id, requested_at) VALUES (?1, ?2)").bind(book.id, clock.now).run();
		expect(await runCollabPurges(env, clock.now)).toBe(1);
		expect(await env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)).state()).toEqual({ head: 0, epoch: 0 });
		expect((await env.DB.prepare("SELECT count(*) AS n FROM book_purges").first<{ n: number }>())!.n).toBe(0);
	});

	it("an account deletion queues its notebooks' rooms", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await env.DB.prepare("INSERT INTO book_purges (id, requested_at) SELECT id, 1 FROM book_shares WHERE owner_id = ?1").bind(alice.creds.accountId).run();
		await runCollabPurges(env, clock.now);
		expect(await env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)).state()).toEqual({ head: 0, epoch: 0 });
	});
});

// ------------------------------------------------------------------ the room

async function socket(a: Account, id: string): Promise<Socket> {
	const s = await openSocket(a.token, id, "books");
	if (s instanceof Response) throw new Error(`live: ${s.status} ${await s.text()}`);
	return s;
}

async function joined(a: Account, id: string): Promise<Socket> {
	const s = await socket(a, id);
	expect(((await s.next()) as any).text.type).toBe("hello");
	return s;
}

const text = (m: unknown) => (m as { text: any }).text;
const bytes = (m: unknown) => (m as { bytes: Uint8Array }).bytes;
const streamBytes = (id: string) => Uint8Array.from(atob(id.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));
const toRoom = (id: string, e: Uint8Array) => {
	const out = new Uint8Array(16 + e.length);
	out.set(id === ZERO ? new Uint8Array(16) : streamBytes(id), 0);
	out.set(e, 16);
	return out;
};
/** Splits `stream || u64(seq) || envelope`. */
const fromRoom = (b: Uint8Array) => ({ stream: base64url(b.slice(0, 16)), seq: Number(new DataView(b.buffer, b.byteOffset + 16).getBigUint64(0)), envelope: b.slice(24) });
const open = async (s: Socket, id: string) => {
	s.ws.send(JSON.stringify({ type: "open", stream: id }));
	expect(text(await s.next())).toMatchObject({ type: "opened", stream: id });
};

describe("the live room", () => {
	it("says hello with the notebook's head and epoch; only accepted members get in", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }], false);
		await append(alice, book.id, NOTE, await seal(book, "u", alice));
		const s = await socket(alice, book.id);
		expect(text(await s.next())).toEqual({ type: "hello", head: 1, epoch: 1 });
		expect(((await openSocket(bob.token, book.id, "books")) as Response).status).toBe(404);
		expect(((await openSocket(carol.token, book.id, "books")) as Response).status).toBe(404);
		expect((await call(`/v2/collab/books/${book.id}/live`, { token: alice.token })).status).toBe(426);
		s.ws.close();
	});

	it("two clients receive each other's envelopes per note, and the sender gets its own back as an acknowledgement", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		const b = await joined(bob, book.id);
		await open(a, NOTE);
		await open(b, NOTE);
		const sent = await seal(book, "u", bob);
		b.ws.send(toRoom(NOTE, sent));
		for (const s of [a, b]) {
			const f = fromRoom(bytes(await s.next()));
			expect(f).toMatchObject({ stream: NOTE, seq: 1 });
			expect(f.envelope).toEqual(sent);
		}
		const reply = await seal(book, "u", alice);
		a.ws.send(toRoom(NOTE, reply));
		for (const s of [a, b]) expect(fromRoom(bytes(await s.next()))).toMatchObject({ stream: NOTE, seq: 2 });
		expect((await read(carol, book.id, NOTE)).body.entries.map((e: any) => e.seq)).toEqual([1, 2]);
		a.ws.close();
		b.ws.close();
	});

	it("multiplexes: a socket hears only the notes it has open, the manifest always", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		const b = await joined(bob, book.id);
		await open(a, NOTE);
		await open(b, NOTE2);
		await open(b, NOTE);
		a.ws.send(toRoom(NOTE, await seal(book, "u", alice)));
		expect(fromRoom(bytes(await a.next())).stream).toBe(NOTE);
		expect(fromRoom(bytes(await b.next())).stream).toBe(NOTE);
		b.ws.send(toRoom(NOTE2, await seal(book, "u", bob)));
		expect(fromRoom(bytes(await b.next())).stream).toBe(NOTE2);
		expect(await a.drain()).toEqual([]);
		b.ws.send(toRoom(ZERO, await seal(book, "u", bob)));
		for (const s of [a, b]) expect(fromRoom(bytes(await s.next()))).toMatchObject({ stream: ZERO, seq: 3 });
		// closing a note stops its frames
		b.ws.send(JSON.stringify({ type: "close", stream: NOTE }));
		await new Promise((r) => setTimeout(r, 50));
		a.ws.send(toRoom(NOTE, await seal(book, "u", alice)));
		expect(fromRoom(bytes(await a.next())).seq).toBe(4);
		expect(await b.drain()).toEqual([]);
		a.ws.close();
		b.ws.close();
	});

	it("tells a client where a note's log is when it opens it, so it can catch up over HTTP", async () => {
		const book = await team();
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		const a = await joined(alice, book.id);
		a.ws.send(JSON.stringify({ type: "open", stream: NOTE }));
		expect(text(await a.next())).toEqual({ type: "opened", stream: NOTE, head: 1 });
		a.ws.close();
	});

	it("an HTTP append reaches the sockets that have the note open", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		await open(a, NOTE);
		const sent = await seal(book, "c", carol);
		expect((await append(carol, book.id, NOTE, sent)).body).toEqual({ seq: 1 });
		expect(fromRoom(bytes(await a.next())).envelope).toEqual(sent);
		a.ws.close();
	});

	it("relays presence to the others only and never stores it", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		const d = await joined(dave, book.id);
		await open(a, NOTE);
		await open(d, NOTE);
		d.ws.send(toRoom(NOTE, await seal(book, "p", dave)));
		expect(fromRoom(bytes(await a.next()))).toMatchObject({ stream: NOTE, seq: 0 });
		expect(await d.drain()).toEqual([]);
		expect((await read(alice, book.id, NOTE)).body).toMatchObject({ head: 0, entries: [] });
		a.ws.close();
		d.ws.close();
	});

	it("drops a refused frame with an error frame: role, author, epoch, size, a note not open, a snapshot, an unknown stream", async () => {
		const book = await team();
		const d = await joined(dave, book.id);
		const c = await joined(carol, book.id);
		await open(d, NOTE);
		await open(c, NOTE);
		d.ws.send(toRoom(NOTE, await seal(book, "u", dave)));
		expect(text(await d.next())).toMatchObject({ type: "error", error: "forbidden_role", stream: NOTE, kind: "u" });
		c.ws.send(toRoom(ZERO, await seal(book, "c", carol)));
		expect(text(await c.next())).toMatchObject({ error: "forbidden_role" });
		c.ws.send(toRoom(NOTE, await envelope({ kind: "c", epoch: 1, shareId: book.id, by: carol, author: alice.creds.accountId })));
		expect(text(await c.next())).toMatchObject({ error: "forbidden_author" });
		c.ws.send(toRoom(NOTE, await seal(book, "c", carol, 5)));
		expect(text(await c.next())).toMatchObject({ error: "epoch_changed", epoch: 1 });
		c.ws.send(toRoom(NOTE, await envelopeOfSize({ kind: "c", epoch: 1, shareId: book.id, by: carol }, 16 * 1024 + 1)));
		expect(text(await c.next())).toMatchObject({ error: "too_large" });
		c.ws.send(toRoom(NOTE2, await seal(book, "c", carol)));
		expect(text(await c.next())).toMatchObject({ error: "not_open", stream: NOTE2 });
		c.ws.send(toRoom(NOTE, await seal(book, "s", carol)));
		expect(text(await c.next())).toMatchObject({ error: "bad_request" });
		c.ws.send(new Uint8Array(10));
		expect(text(await c.next())).toMatchObject({ error: "bad_request" });
		c.ws.send(JSON.stringify({ type: "open", stream: ZERO }));
		expect(text(await c.next())).toMatchObject({ error: "bad_request" });
		c.ws.send("not json");
		expect(text(await c.next())).toMatchObject({ error: "bad_request" });
		expect((await read(alice, book.id, NOTE)).body.entries).toEqual([]);
		d.ws.close();
		c.ws.close();
	});

	it("limits a socket to 32 open notes and to 30 frames a second", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		for (let i = 0; i < 32; i++) await open(a, base64url(new Uint8Array(16).fill(i + 1)));
		a.ws.send(JSON.stringify({ type: "open", stream: NOTE }));
		expect(text(await a.next())).toMatchObject({ error: "too_many_open", limit: 32 });
		await open(a, base64url(new Uint8Array(16).fill(1)));
		// bursts until the bucket (60 frames, 30 a second) runs dry; a loaded machine may need more than one
		let limited = false;
		for (let round = 0; round < 10 && !limited; round++) {
			for (let i = 0; i < 300; i++) a.ws.send(JSON.stringify({ type: "close", stream: NOTE }));
			limited = (await a.drain(400)).map(text).some((m) => m.error === "rate_limited" && m.limit === "socket");
		}
		expect(limited).toBe(true);
		a.ws.close();
	});

	it("kicks a removed member on the new epoch, and tells the others", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		const b = await joined(bob, book.id);
		const c = await joined(carol, book.id);
		await open(b, NOTE);
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await b.next();
		const members = [
			{ account: alice, role: "owner" as const },
			{ account: carol, role: "comment" as const },
			{ account: dave, role: "view" as const },
		];
		expect(await advance(book, alice, members, { newEpoch: true, fromSeq: 1 })).toMatchObject({ status: 200 });
		expect(text(await b.next())).toEqual({ type: "closed", reason: "removed" });
		expect(await b.closed).toMatchObject({ reason: "removed" });
		for (const s of [a, c]) expect(text(await s.next())).toEqual({ type: "epoch", epoch: 2 });
		// and cannot get back in or read
		expect(((await openSocket(bob.token, book.id, "books")) as Response).status).toBe(404);
		expect((await read(bob, book.id, NOTE)).status).toBe(404);
		a.ws.close();
		c.ws.close();
	});

	it("applies a demotion at once: the socket keeps its place but not its write role", async () => {
		const book = await team();
		const b = await joined(bob, book.id);
		await open(b, NOTE);
		const members = [
			{ account: alice, role: "owner" as const },
			{ account: bob, role: "view" as const },
			{ account: carol, role: "comment" as const },
			{ account: dave, role: "view" as const },
		];
		expect((await advance(book, alice, members, { newEpoch: true, fromSeq: 0 })).status).toBe(200);
		expect(text(await b.next())).toEqual({ type: "epoch", epoch: 2 });
		// the room tells it twice: when the epoch begins and when D1 has committed
		for (const m of await b.drain(150)) expect(text(m)).toEqual({ type: "epoch", epoch: 2 });
		b.ws.send(toRoom(NOTE, await seal(book, "u", bob, 2)));
		expect(text(await b.next())).toMatchObject({ error: "forbidden_role" });
		b.ws.close();
	});

	it("closes a member's sockets when they leave", async () => {
		const book = await team();
		const c = await joined(carol, book.id);
		expect((await books(carol.token).leave(book.id)).status).toBe(204);
		expect(text(await c.next())).toEqual({ type: "closed", reason: "removed" });
		expect(await c.closed).toMatchObject({ code: 1000 });
	});

	it("limits an account to 8 sockets", async () => {
		const book = await team();
		const sockets: Socket[] = [];
		for (let i = 0; i < 8; i++) sockets.push(await joined(bob, book.id));
		expect(await openSocket(bob.token, book.id, "books")).toMatchObject({ status: 429 });
		for (const s of sockets) s.ws.close();
	});

	it("stopping the notebook tells every socket and deletes what the room held", async () => {
		const book = await team();
		const a = await joined(alice, book.id);
		const b = await joined(bob, book.id);
		await append(bob, book.id, NOTE, await seal(book, "u", bob));
		await open(b, NOTE);
		expect((await books(alice.token).stop(book.id)).status).toBe(204);
		for (const s of [a, b]) expect(text(await s.next())).toEqual({ type: "closed", reason: "stopped" });
		expect(await env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)).state()).toEqual({ head: 0, epoch: 0 });
		expect(((await openSocket(alice.token, book.id, "books")) as Response).status).toBe(404);
	});
});
