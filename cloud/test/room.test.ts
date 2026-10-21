// The live room over WebSockets (docs/ACCOUNTS.md › Collaboration › The
// server: storage and the live room): the hello frame, stored frames sent to
// everyone (the sender's acknowledgement), presence relayed to the others and
// never stored, refused frames answered with error frames, epoch and closed
// frames when the owner changes the list or stops sharing, the per-socket
// rate limit, and a room that hibernates between messages.

import { evictDurableObject, runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { type Account, base64url, resetServer, signUp } from "./helpers";
import { type Message, type Socket, collab, envelope, envelopeOfSize, epochBody, openSocket, shareWith, unframe } from "./collab-helpers";

let alice: Account;
let bob: Account;
let carol: Account;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	bob = await signUp("bob");
	carol = await signUp("carol");
});

async function socket(a: Account, id: string): Promise<Socket> {
	const s = await openSocket(a.token, id);
	if (s instanceof Response) throw new Error(`live: ${s.status} ${await s.text()}`);
	return s;
}

/** Opens a socket and reads its hello. */
async function joined(a: Account, id: string): Promise<Socket> {
	const s = await socket(a, id);
	const hello = await s.next();
	expect("text" in hello && hello.text.type).toBe("hello");
	return s;
}

const text = (m: unknown) => (m as { text: any }).text;
const bytes = (m: unknown) => (m as { bytes: Uint8Array }).bytes;

describe("connecting", () => {
	it("says hello with the head and the epoch", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		await collab(alice.token).append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }));
		const s = await socket(bob, id);
		expect(text(await s.next())).toEqual({ type: "hello", head: 1, epoch: 1 });
		s.ws.close();
	});

	it("only members: 404 for a stranger and for an invitee alike", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }], false);
		expect(((await openSocket(carol.token, id)) as Response).status).toBe(404);
		expect(((await openSocket(bob.token, id)) as Response).status).toBe(404);
	});
});

describe("frames", () => {
	it("an update goes to everyone, the sender included, with its seq, and is in the log", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const a = await joined(alice, id);
		const b = await joined(bob, id);
		const sent = await envelope({ kind: "u", epoch: 1, shareId: id, by: bob });
		b.ws.send(sent);
		for (const s of [a, b]) {
			const f = unframe(bytes(await s.next()));
			expect(f.seq).toBe(1);
			expect(f.envelope).toEqual(sent);
		}
		const log = (await collab(alice.token).log(id)).body;
		expect(log.entries).toEqual([{ seq: 1, envelope: base64url(sent) }]);
		a.ws.close();
		b.ws.close();
	});

	it("an append over HTTP reaches the sockets too", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "comment" }]);
		const a = await joined(alice, id);
		const sent = await envelope({ kind: "c", epoch: 1, shareId: id, by: bob });
		expect((await collab(bob.token).append(id, sent)).body).toEqual({ seq: 1 });
		expect(unframe(bytes(await a.next()))).toEqual({ seq: 1, envelope: sent });
		a.ws.close();
	});

	it("presence goes to the others with seq 0, never back to the sender, and is never stored", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		const a = await joined(alice, id);
		const b = await joined(bob, id);
		const presence = await envelope({ kind: "p", epoch: 1, shareId: id, by: bob, plaintext: '{"cursor":"x"}' });
		b.ws.send(presence);
		expect(unframe(bytes(await a.next()))).toEqual({ seq: 0, envelope: presence });
		expect(await b.drain()).toEqual([]);
		expect((await collab(alice.token).log(id)).body).toMatchObject({ head: 0, entries: [] });
		a.ws.close();
		b.ws.close();
	});

	it("refuses what the role, epoch, author, kind or size don't allow, with error frames, storing nothing", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "view" },
			{ account: carol, role: "comment" },
		]);
		const b = await joined(bob, id);
		const c = await joined(carol, id);
		const expectError = async (s: Socket, error: string) => {
			const m = text(await s.next());
			expect(m).toMatchObject({ type: "error", error });
			return m;
		};
		const u = await envelope({ kind: "u", epoch: 1, shareId: id, by: bob });
		b.ws.send(u);
		const refused = await expectError(b, "forbidden_role");
		expect(refused.kind).toBe("u");
		expect(typeof refused.counter).toBe("string");
		b.ws.send(await envelope({ kind: "c", epoch: 1, shareId: id, by: bob }));
		await expectError(b, "forbidden_role");
		c.ws.send(await envelope({ kind: "u", epoch: 1, shareId: id, by: carol }));
		await expectError(c, "forbidden_role");
		c.ws.send(await envelope({ kind: "c", epoch: 2, shareId: id, by: carol }));
		expect(await expectError(c, "epoch_changed")).toMatchObject({ epoch: 1 });
		c.ws.send(await envelope({ kind: "c", epoch: 1, shareId: id, by: carol, author: alice.creds.accountId }));
		await expectError(c, "forbidden_author");
		c.ws.send(await envelope({ kind: "s", epoch: 1, shareId: id, by: carol }));
		await expectError(c, "bad_request");
		c.ws.send("hello");
		await expectError(c, "bad_request");
		c.ws.send(new Uint8Array([1, 2, 3]));
		await expectError(c, "bad_request");
		c.ws.send(await envelopeOfSize({ kind: "p", epoch: 1, shareId: id, by: carol }, 2 * 1024 + 1));
		await expectError(c, "too_large");
		expect((await collab(alice.token).log(id)).body).toMatchObject({ head: 0, entries: [] });
		// A viewer may still send presence.
		b.ws.send(await envelope({ kind: "p", epoch: 1, shareId: id, by: bob }));
		expect(unframe(bytes(await c.next())).seq).toBe(0);
		b.ws.close();
		c.ws.close();
	});

	it("rate-limits each socket to bursts of 60, dropping the rest with rate_limited", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		const b = await joined(bob, id);
		const sent = 200;
		const frames = await Promise.all(Array.from({ length: sent }, () => envelope({ kind: "p", epoch: 1, shareId: id, by: bob })));
		const started = Date.now();
		for (const f of frames) b.ws.send(f);
		// Accepted presence gets no answer, so a text frame (always refused,
		// after the frames before it) marks the end: bad_request once the
		// bucket has a token for it, rate_limited before that.
		const answers: Awaited<ReturnType<typeof b.drain>> = [];
		let markers = 0;
		const isEnd = (m: (typeof answers)[number]) => "text" in m && m.text.error === "bad_request";
		while (!answers.some(isEnd)) {
			b.ws.send("end");
			markers += 1;
			for (let i = 0; i < 20 && !answers.some(isEnd); i++) answers.push(...(await b.drain(100)));
		}
		const elapsed = Date.now() - started;
		// The bucket refills at 30 a second while the room works through the
		// frames, and a loaded machine works slowly: bound by the time taken.
		const refilled = Math.ceil((30 * elapsed) / 1000);
		// Presence past the account's own budget is refused too (limit "presence"); only the socket's count here.
		const limited = answers.filter((m) => "text" in m && m.text.error === "rate_limited" && m.text.limit === "socket").slice(markers - 1);
		expect(sent - limited.length).toBeLessThanOrEqual(60 + refilled);
		expect(limited.length).toBeLessThanOrEqual(sent - 60);
		if (60 + refilled < sent) expect(limited.length).toBeGreaterThan(0);
		// A machine slow enough refills the bucket for every frame: nothing to read then.
		for (const m of limited) expect(text(m)).toMatchObject({ type: "error", error: "rate_limited", limit: "socket" });
		b.ws.close();
	});
});

describe("presence and socket caps", () => {
	it("presence is budgeted per account across its sockets: bursts of 20, then rate_limited presence", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		const a = await joined(alice, id);
		const b1 = await joined(bob, id);
		const b2 = await joined(bob, id);
		const frames = await Promise.all(Array.from({ length: 30 }, () => envelope({ kind: "p", epoch: 1, shareId: id, by: bob })));
		const started = Date.now();
		frames.forEach((f, i) => (i % 2 === 0 ? b1 : b2).ws.send(f));
		// Every frame gets exactly one answer: relayed to alice, or refused to
		// its sender. A loaded machine works through them slowly, so wait for
		// all 30 (up to a deadline) rather than for a fixed time.
		const refused: Message[] = [];
		const relayed: Message[] = [];
		const collect = async (ms: number) => {
			refused.push(...[...(await b1.drain(ms)), ...(await b2.drain(0))].filter((m) => "text" in m));
			relayed.push(...(await a.drain(0)).filter((m) => "bytes" in m));
		};
		const deadline = Date.now() + 10_000;
		while (relayed.length + refused.length < 30 && Date.now() < deadline) await collect(50);
		const refill = Math.ceil((10 * (Date.now() - started)) / 1000);
		// Anything past the 30 would be an extra answer: give it time to show.
		await collect(100);
		for (const m of refused) expect(text(m)).toMatchObject({ type: "error", error: "rate_limited", limit: "presence", kind: "p" });
		expect(relayed.length + refused.length).toBe(30);
		expect(relayed.length).toBeGreaterThanOrEqual(20);
		expect(relayed.length).toBeLessThanOrEqual(20 + refill);
		// Another account's budget is its own.
		a.ws.send(await envelope({ kind: "p", epoch: 1, shareId: id, by: alice }));
		expect(unframe(bytes(await b1.next())).seq).toBe(0);
		for (const s of [a, b1, b2]) s.ws.close();
	});

	it("at most 8 sockets per account in a room: the ninth upgrade is 429 rate_limited sockets", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		const open: Socket[] = [];
		for (let i = 0; i < 8; i++) open.push(await joined(bob, id));
		const ninth = (await openSocket(bob.token, id)) as Response;
		expect(ninth.status).toBe(429);
		expect(await ninth.json()).toMatchObject({ error: "rate_limited", limit: "sockets" });
		const other = await joined(alice, id);
		open[0].ws.close();
		await open[0].closed;
		await new Promise((r) => setTimeout(r, 50));
		const again = await joined(bob, id);
		for (const s of [...open.slice(1), other, again]) s.ws.close();
	});

	it("a socket's comment past the owner's quota or the room's comment cap is refused with quota_exceeded", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "comment" }]);
		const b = await joined(bob, id);
		await runInDurableObject(env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)), (_room, state) => {
			state.storage.sql.exec("UPDATE meta SET value = ? WHERE name = 'comment_bytes'", 8 * 1024 * 1024 - 100);
		});
		b.ws.send(await envelope({ kind: "c", epoch: 1, shareId: id, by: bob }));
		expect(text(await b.next())).toMatchObject({ type: "error", error: "quota_exceeded", quota: "comment_bytes", limit: 8 * 1024 * 1024, kind: "c" });
		await runInDurableObject(env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)), (_room, state) => {
			state.storage.sql.exec("UPDATE meta SET value = 0 WHERE name = 'comment_bytes'");
		});
		await env.DB.prepare("UPDATE accounts SET quota_doc_bytes = 300 WHERE id = ?1").bind(alice.creds.accountId).run();
		b.ws.send(await envelope({ kind: "c", epoch: 1, shareId: id, by: bob }));
		expect(text(await b.next())).toMatchObject({ type: "error", error: "quota_exceeded", quota: "doc_bytes", limit: 300 });
		b.ws.close();
	});
});

describe("the owner's changes", () => {
	it("a new epoch: removed members hear closed/removed and are closed; the rest hear the epoch and their new role applies", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "edit" },
		]);
		const b = await joined(bob, id);
		const c = await joined(carol, id);
		expect((await collab(alice.token).epoch(id, epochBody(id, 2, alice, [{ account: carol, role: "view" }]))).status).toBe(200);
		expect(text(await b.next())).toEqual({ type: "closed", reason: "removed" });
		expect((await b.closed).reason).toBe("removed");
		expect(text(await c.next())).toEqual({ type: "epoch", epoch: 2 });
		c.ws.send(await envelope({ kind: "u", epoch: 2, shareId: id, by: carol }));
		expect(text(await c.next())).toMatchObject({ type: "error", error: "forbidden_role" });
		c.ws.close();
	});

	it("leaving closes the member's sockets; stopping closes everyone's with stopped", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "edit" },
		]);
		const a = await joined(alice, id);
		const b = await joined(bob, id);
		const c = await joined(carol, id);
		expect((await collab(bob.token).leave(id)).status).toBe(204);
		expect(text(await b.next())).toEqual({ type: "closed", reason: "removed" });
		expect((await collab(alice.token).stop(id)).status).toBe(204);
		for (const s of [a, c]) {
			expect(text(await s.next())).toEqual({ type: "closed", reason: "stopped" });
			expect((await s.closed).reason).toBe("stopped");
		}
	});
});

describe("hibernation", () => {
	it("sockets keep their member after the room is evicted", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "view" },
			{ account: carol, role: "edit" },
		]);
		const b = await joined(bob, id);
		const c = await joined(carol, id);
		await evictDurableObject(env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)));
		b.ws.send(await envelope({ kind: "u", epoch: 1, shareId: id, by: bob }));
		expect(text(await b.next())).toMatchObject({ type: "error", error: "forbidden_role" });
		const sent = await envelope({ kind: "u", epoch: 1, shareId: id, by: carol });
		c.ws.send(sent);
		expect(unframe(bytes(await b.next()))).toEqual({ seq: 1, envelope: sent });
		b.ws.close();
		c.ws.close();
	});
});
