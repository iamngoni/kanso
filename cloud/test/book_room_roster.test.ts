// The notebook room enforces its own roster and epoch (issue #390, slice 34, security fixes): it takes
// the role from what it was told, never from an upgrade's headers; a lost roster call is retried from
// D1 by the cron; a write that raced a new epoch during the quota check is refused; a new epoch's
// head check and epoch raise are one step; a purged room never comes back.

import { runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { ROOM_HEADERS } from "../src/room";
import { BOOK_SHARE_HEADER } from "../src/book_room";
import { accountCollabStatements, runCollabPurges } from "../src/collab";
import { type Book, advance, books, newShareId, shareBook } from "./book-helpers";
import { type Socket, envelope, openSocket, toSocket } from "./collab-helpers";
import { type Account, call, clock, resetServer, signUp } from "./helpers";

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

let restore: (() => void) | null = null;
afterEach(() => {
	restore?.();
	restore = null;
});

/** Makes the room's roster push fail (`times` calls, or all), as an unreachable room would; undone after the test. */
async function failSetEpoch(book: { id: string }, times = Number.POSITIVE_INFINITY): Promise<void> {
	await runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(book.id)), (room: unknown) => {
		const proto = Object.getPrototypeOf(room) as { setEpoch: (...a: unknown[]) => Promise<unknown> };
		const real = proto.setEpoch;
		let left = times;
		proto.setEpoch = async function (this: unknown, ...args: unknown[]) {
			if (left > 0) {
				left -= 1;
				throw new Error("unreachable");
			}
			return real.apply(this, args);
		};
		restore = () => {
			proto.setEpoch = real;
		};
	});
}

/** Runs `fn` inside a notebook's room, with the instance untyped so a test can reach its internals. */
const inRoom = (id: string, fn: (room: any) => Promise<void> | void) => runInDurableObject(env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(id)), (r: unknown) => fn(r));

const NOTE = newShareId();
const roomOf = (id: string) => env.BOOK_ROOM.get(env.BOOK_ROOM.idFromName(id));
const seal = (book: Book, kind: "u" | "s" | "c" | "p", by: Account, epoch = book.epoch) => envelope({ kind, epoch, shareId: book.id, by });
const text = (m: unknown) => (m as { text: any }).text;

async function append(a: Account, id: string, stream: string, bytes: Uint8Array) {
	const response = await call(`/v2/collab/books/${id}/notes/${stream}`, {
		token: a.token,
		method: "POST",
		body: bytes,
		headers: { "content-type": "application/octet-stream" },
	});
	return { status: response.status, body: (await response.json()) as any };
}

const team = () =>
	shareBook(alice, [
		{ account: bob, role: "edit" },
		{ account: carol, role: "comment" },
		{ account: dave, role: "view" },
	]);

const members = (...spec: [Account, "owner" | "manager" | "edit" | "comment" | "view"][]) => spec.map(([account, role]) => ({ account, role }));

/** An upgrade as the Worker would send it, with whatever headers the test wants (stale or invented ones). */
const upgrade = (book: Book, a: Account, headers: Record<string, string>) =>
	roomOf(book.id).fetch(
		new Request("https://room/live", {
			headers: { upgrade: "websocket", [ROOM_HEADERS.account]: a.creds.accountId, [ROOM_HEADERS.device]: a.device.id, [BOOK_SHARE_HEADER]: book.id, ...headers },
		}),
	);

const toRoom = (stream: string, e: Uint8Array) => {
	const bytes = Uint8Array.from(atob(stream.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));
	const out = new Uint8Array(16 + e.length);
	out.set(bytes, 0);
	out.set(e, 16);
	return out;
};

async function joinedSocket(a: Account, id: string): Promise<Socket> {
	const s = await openSocket(a.token, id, "books");
	if (s instanceof Response) throw new Error(`live: ${s.status}`);
	expect(text(await s.next()).type).toBe("hello");
	return s;
}

describe("fix 1: the room trusts its own roster, not the upgrade", () => {
	it("an upgrade with a stale epoch or an invented role is refused, or gets the stored role", async () => {
		const book = await team();
		expect((await append(alice, book.id, NOTE, await seal(book, "u", alice))).status).toBe(200);
		// bob and dave are cut or demoted by a new epoch (head is 1)
		expect(await advance(book, alice, members([alice, "owner"], [bob, "view"], [carol, "comment"]), { newEpoch: true, fromSeq: 1 })).toMatchObject({ status: 200 });
		// a stale epoch, whatever role the headers claim
		const stale = await upgrade(book, bob, { [ROOM_HEADERS.epoch]: "1", "x-kanso-room-role": "owner" });
		expect(stale.status).toBe(409);
		expect(await stale.json()).toMatchObject({ error: "epoch_changed", epoch: 2 });
		// a removed account is not on the roster, however current its epoch
		const removed = await upgrade(book, dave, { [ROOM_HEADERS.epoch]: "2", "x-kanso-room-role": "owner" });
		expect(removed.status).toBe(409);
		expect(await removed.json()).toMatchObject({ error: "roster_stale" });
		// a current upgrade claiming a write role gets the stored one: bob is a viewer
		const ok = await upgrade(book, bob, { [ROOM_HEADERS.epoch]: "2", "x-kanso-room-role": "owner" });
		expect(ok.status).toBe(101);
		const s = toSocket(ok) as Socket;
		expect(text(await s.next())).toMatchObject({ type: "hello", epoch: 2 });
		s.ws.send(JSON.stringify({ type: "open", stream: NOTE }));
		expect(text(await s.next())).toMatchObject({ type: "opened" });
		s.ws.send(toRoom(NOTE, await seal(book, "u", bob, 2)));
		expect(text(await s.next())).toMatchObject({ error: "forbidden_role" });
		s.ws.close();
	});

	it("a demoted editor's open socket can't write after the demotion, even if the room never hears the sync", async () => {
		const book = await team();
		const b = await joinedSocket(bob, book.id);
		b.ws.send(JSON.stringify({ type: "open", stream: NOTE }));
		expect(text(await b.next())).toMatchObject({ type: "opened" });
		b.ws.send(toRoom(NOTE, await seal(book, "u", bob)));
		await b.next();
		// every roster push after the epoch is lost
		await failSetEpoch(book);
		expect(await advance(book, alice, members([alice, "owner"], [bob, "view"], [carol, "comment"], [dave, "view"]), { newEpoch: true, fromSeq: 1 })).toMatchObject({ status: 200 });
		b.ws.send(toRoom(NOTE, await seal(book, "u", bob, 2)));
		const replies = (await b.drain(300)).map(text);
		expect(replies.find((m) => m.type === "error")).toMatchObject({ error: "forbidden_role" });
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 2))).status).toBe(403);
		b.ws.close();
	});

	it("a removed member's socket is closed in the step that begins the epoch", async () => {
		const book = await team();
		const d = await joinedSocket(dave, book.id);
		await failSetEpoch(book);
		expect(await advance(book, alice, members([alice, "owner"], [bob, "edit"], [carol, "comment"]), { newEpoch: true, fromSeq: 0 })).toMatchObject({ status: 200 });
		expect(text(await d.next())).toEqual({ type: "closed", reason: "removed" });
	});
});

describe("fix 2: a roster change that doesn't reach the room is queued and retried", () => {
	it("leaving with the room unreachable once is applied by the cron", async () => {
		const book = await team();
		const c = await joinedSocket(carol, book.id);
		await failSetEpoch(book, 1);
		expect((await books(carol.token).leave(book.id)).status).toBe(204);
		const queued = await env.DB.prepare("SELECT id FROM book_room_sync WHERE id = ?1").bind(book.id).first();
		expect(queued).not.toBeNull();
		// D1 already refuses her; the room hasn't been told, so the socket is still open
		expect((await call(`/v2/collab/books/${book.id}/manifest`, { token: carol.token })).status).toBe(404);
		await runCollabPurges(env, clock.now);
		expect(await env.DB.prepare("SELECT id FROM book_room_sync WHERE id = ?1").bind(book.id).first()).toBeNull();
		expect(text(await c.next())).toEqual({ type: "closed", reason: "removed" });
	});

	it("a roster the room never got (a lost create) is synced on first use", async () => {
		const book = await team();
		await runInDurableObject(roomOf(book.id), (_room, state) => {
			state.storage.sql.exec("DELETE FROM roster");
			state.storage.sql.exec("DELETE FROM meta WHERE name = 'synced'");
		});
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		const s = await joinedSocket(alice, book.id);
		s.ws.close();
	});

	it("an account deletion queues the rosters of notebooks it was a member of, and the cron applies them", async () => {
		const book = await team();
		const b = await joinedSocket(bob, book.id);
		await env.DB.batch(
			accountCollabStatements(env.DB, bob.creds.accountId, "1 = 1", "", clock.now).map((s) => s),
		);
		expect((await env.DB.prepare("SELECT id FROM book_room_sync WHERE id = ?1").bind(book.id).first())).not.toBeNull();
		await runCollabPurges(env, clock.now);
		expect(text(await b.next())).toEqual({ type: "closed", reason: "removed" });
		expect(await env.DB.prepare("SELECT id FROM book_room_sync WHERE id = ?1").bind(book.id).first()).toBeNull();
	});
});

describe("fix 3: a write that raced a new epoch during the quota check is refused", () => {
	it("an update whose epoch ended during the quota await is refused", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		const e = await seal(book, "u", bob, 1);
		await inRoom(book.id, async (room) => {
			const real = room.overQuota.bind(room);
			room.overQuota = async () => {
				room.overQuota = real;
				const r = await room.beginEpoch({ expectedHead: 1, epoch: 2, roles: { [alice.creds.accountId]: "owner", [bob.creds.accountId]: "edit" } });
				expect(r.ok).toBe(true);
				return null;
			};
			const result = await room.append({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, envelope: e, now: clock.now });
			expect(result).toMatchObject({ ok: false, error: "epoch_changed", extras: { epoch: 2 } });
			expect(await room.state()).toEqual({ head: 1, epoch: 2 });
		});
	});

	it("a snapshot whose epoch ended, or whose log moved, during the quota await is refused", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		const snap = await seal(book, "s", bob, 1);
		const another = await seal(book, "u", alice, 1);
		await inRoom(book.id, async (room) => {
			const real = room.overQuota.bind(room);
			room.overQuota = async () => {
				room.overQuota = real;
				expect((await room.append({ shareId: book.id, stream: NOTE, account: alice.creds.accountId, epoch: 1, envelope: another, now: clock.now })).ok).toBe(true);
				return null;
			};
			expect(await room.snapshot({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, baseSeq: 1, envelope: snap, now: clock.now })).toMatchObject({
				ok: false,
				error: "head_moved",
				extras: { head: 2 },
			});
			room.overQuota = async () => {
				room.overQuota = real;
				await room.beginEpoch({ expectedHead: 2, epoch: 2, roles: { [alice.creds.accountId]: "owner", [bob.creds.accountId]: "edit" } });
				return null;
			};
			expect(await room.snapshot({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, baseSeq: 2, envelope: snap, now: clock.now })).toMatchObject({
				ok: false,
				error: "epoch_changed",
			});
		});
	});

	it("a role taken away during the quota await refuses the write too", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		const e = await seal(book, "u", bob, 1);
		await inRoom(book.id, async (room) => {
			const real = room.overQuota.bind(room);
			room.overQuota = async () => {
				room.overQuota = real;
				await room.setEpoch({ epoch: 1, roles: { [alice.creds.accountId]: "owner", [bob.creds.accountId]: "view" } });
				return null;
			};
			expect(await room.append({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, envelope: e, now: clock.now })).toMatchObject({ ok: false, error: "forbidden_role" });
		});
	});
});

describe("fix 4: the head check and the epoch raise are one step", () => {
	it("old-epoch writes in the gap before D1 commits are refused, not accepted", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		const room = roomOf(book.id);
		const roles = { [alice.creds.accountId]: "owner" as const, [bob.creds.accountId]: "edit" as const };
		expect(await room.beginEpoch({ expectedHead: 0, epoch: 2, roles })).toMatchObject({ ok: false, error: "head_moved", extras: { head: 1 } });
		expect(await room.state()).toEqual({ head: 1, epoch: 1 });
		expect(await room.beginEpoch({ expectedHead: 1, epoch: 2, roles })).toMatchObject({ ok: true, previous: 1 });
		// D1 still says epoch 1; the Worker passes it, and the room refuses
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 1))).body).toMatchObject({ error: "epoch_changed", epoch: 2 });
		expect(await room.state()).toEqual({ head: 1, epoch: 2 });
		// the same list retried is fine; D1 refusing the list puts the epoch back
		expect(await room.beginEpoch({ expectedHead: 1, epoch: 2, roles })).toMatchObject({ ok: true });
		await room.abortEpoch({ epoch: 2, previous: 1 });
		expect(await room.state()).toEqual({ head: 1, epoch: 1 });
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob, 1))).status).toBe(200);
	});

	it("two lists racing for the same epoch: one wins, and the room's epoch and roster are the winner's", async () => {
		const book = await team();
		const withoutDave = members([alice, "owner"], [bob, "edit"], [carol, "comment"]);
		const withoutCarol = members([alice, "owner"], [bob, "edit"], [dave, "view"]);
		const results = await Promise.all([advance({ ...book }, alice, withoutDave, { newEpoch: true }), advance({ ...book }, alice, withoutCarol, { newEpoch: true })]);
		expect(results.map((r) => r.status).sort()).toEqual([200, 409]);
		const winner = results[0].status === 200 ? withoutDave : withoutCarol;
		expect(await roomOf(book.id).state()).toEqual({ head: 0, epoch: 2 });
		await inRoom(book.id, (room) => {
			const rows = room.ctx.storage.sql.exec("SELECT account FROM roster ORDER BY account").toArray().map((r: { account: string }) => r.account);
			expect(rows).toEqual(winner.map((m) => m.account.creds.accountId).sort());
		});
	});
});

describe("fix 5: a purged room stays purged", () => {
	it("refuses an upgrade, an append, a snapshot and a roster after the notebook stopped", async () => {
		const book = await team();
		expect((await append(bob, book.id, NOTE, await seal(book, "u", bob))).status).toBe(200);
		expect((await books(alice.token).stop(book.id)).status).toBe(204);
		const room = roomOf(book.id);
		const roles = { [alice.creds.accountId]: "owner" as const, [bob.creds.accountId]: "edit" as const };
		expect((await upgrade(book, bob, { [ROOM_HEADERS.epoch]: "1" })).status).toBe(404);
		expect(await room.append({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, envelope: await seal(book, "u", bob), now: clock.now })).toMatchObject({
			ok: false,
			error: "not_found",
		});
		expect(await room.snapshot({ shareId: book.id, stream: NOTE, account: bob.creds.accountId, epoch: 1, baseSeq: 0, envelope: await seal(book, "s", bob), now: clock.now })).toMatchObject({
			ok: false,
			error: "not_found",
		});
		await room.setEpoch({ epoch: 1, roles });
		expect(await room.beginEpoch({ expectedHead: 0, epoch: 2, roles })).toMatchObject({ ok: false, error: "not_found" });
		expect((await upgrade(book, bob, { [ROOM_HEADERS.epoch]: "1" })).status).toBe(404);
		expect(await room.state()).toEqual({ head: 0, epoch: 0 });
	});
});
