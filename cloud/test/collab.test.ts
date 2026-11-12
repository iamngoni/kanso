// Shared notes over HTTP (docs/ACCOUNTS.md › Collaboration): identities and
// lookups, sharing, invitations, roles for each kind of envelope, epochs and
// removals, snapshots and the log, links and requests, notifications, the
// caps and rate limits, stopping, account deletion and the cron, the /j/ page,
// and the Origin and Content-Type rules.

import { runDurableObjectAlarm, runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { sha256Hex } from "../src/crypto";
import { MAX_ENVELOPE } from "../src/envelope";
import { runCron } from "../src/index";
import { type Account, START, base64url, call, callJson, fromBase64url, resetServer, runAdmin, setSetting, signUp, testEnv } from "./helpers";
import {
	b64,
	collab,
	ensureIdentity,
	envelope,
	envelopeOfSize,
	epochBody,
	memberList,
	newIdentity,
	newShareId,
	shareBody,
	shareWith,
} from "./collab-helpers";

const DAY = 86_400_000;

let alice: Account;
let bob: Account;
let carol: Account;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	bob = await signUp("bob");
	carol = await signUp("carol");
});

const roomState = (id: string) => env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)).state();

/** The proof a link's secret gives, and its hash as the owner sends it. */
async function linkProof(): Promise<{ proof: string; hash: string }> {
	const proof = crypto.getRandomValues(new Uint8Array(32));
	return { proof: base64url(proof), hash: await sha256Hex(proof) };
}

describe("identity", () => {
	it("is set once: 201, the same again 200, another 409 identity_exists; GET returns it", async () => {
		const c = collab(alice.token);
		expect((await c.identity()).status).toBe(404);
		const id = newIdentity();
		expect(await c.putIdentity(id)).toMatchObject({ status: 201, body: { ok: true } });
		expect(await c.putIdentity(id)).toMatchObject({ status: 200, body: { ok: true } });
		expect(await c.putIdentity(newIdentity())).toMatchObject({ status: 409, body: { error: "identity_exists" } });
		expect(await c.identity()).toMatchObject({ status: 200, body: id });
	});

	it("refuses keys of the wrong shape", async () => {
		const c = collab(alice.token);
		for (const bad of [
			{ ...newIdentity(), sign_pub: b64(31) },
			{ ...newIdentity(), binding: b64(63) },
			{ ...newIdentity(), sealed: `${b64(105).slice(0, 139)}*` },
			{ ...newIdentity(), box_pub: undefined },
		]) {
			expect((await c.putIdentity(bad)).body).toMatchObject({ error: "bad_request" });
		}
		expect((await c.identity()).status).toBe(404);
	});

	it("GET /v2/users/<name> gives public keys by normalized name; 404 unknown, without identity or disabled; 400 invalid", async () => {
		const id = await ensureIdentity(bob);
		const c = collab(alice.token);
		const r = await c.user("  BOB ");
		expect(r).toEqual({
			status: 200,
			headers: expect.anything(),
			body: { account_id: bob.creds.accountId, username: "bob", sign_pub: id.sign_pub, box_pub: id.box_pub, binding: id.binding },
		});
		expect(r.body.sealed).toBeUndefined();
		expect((await c.user("nobody")).status).toBe(404);
		expect((await c.user("carol")).status).toBe(404);
		expect((await c.user("a")).body.error).toBe("invalid_username");
		expect((await runAdmin(["disable", "bob"])).ok).toBe(true);
		expect((await c.user("bob")).status).toBe(404);
	});

	it("lookups are counted by lookup-account, 120 an hour", async () => {
		await env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, 120)").bind(`lookup-account:${alice.creds.accountId}`, START).run();
		const r = await collab(alice.token).user("bob");
		expect(r.status).toBe(429);
		expect(r.body).toMatchObject({ error: "rate_limited", limit: "lookup-account" });
	});
});

describe("sharing", () => {
	it("201 {share_id, epoch, head}; the owner is a member, others invited, with wraps and the list as sent", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const id = newShareId();
		const body = await shareBody(id, alice, [{ account: bob, role: "edit" }]);
		const r = await collab(alice.token).share(body);
		expect(r).toMatchObject({ status: 201, body: { share_id: id, epoch: 1, head: 0 } });

		const mine = (await collab(alice.token).shares()).body.shares;
		expect(mine).toHaveLength(1);
		expect(mine[0]).toMatchObject({
			share_id: id,
			owner: alice.creds.accountId,
			owner_username: "alice",
			role: "owner",
			status: "member",
			epoch: 1,
			list: body.list,
			list_sig: body.list_sig,
			head: 0,
			links: [],
			requests: [],
		});
		expect(mine[0].wraps).toEqual([{ epoch: 1, wrapped: body.wraps.find((w) => w.account === alice.creds.accountId)!.wrapped }]);
		expect(mine[0].members).toEqual(
			[
				{ account: alice.creds.accountId, status: "member" },
				{ account: bob.creds.accountId, status: "invited" },
			].sort((x, y) => (x.account < y.account ? -1 : 1)),
		);
		const theirs = (await collab(bob.token).shares()).body.shares;
		expect(theirs[0]).toMatchObject({ share_id: id, role: "edit", status: "invited", head: 0 });
		expect(theirs[0].members).toBeUndefined();
		expect(theirs[0].wraps).toEqual([{ epoch: 1, wrapped: body.wraps.find((w) => w.account === bob.creds.accountId)!.wrapped }]);
		expect((await collab(carol.token).shares()).body).toEqual({ shares: [] });
	});

	it("the same share again answers 201 (a lost answer); another owner's ID 409 share_id_taken", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(carol);
		const id = newShareId();
		const body = await shareBody(id, alice, []);
		expect((await collab(alice.token).share(body)).status).toBe(201);
		expect(await collab(alice.token).share(body)).toMatchObject({ status: 201, body: { share_id: id, epoch: 1, head: 0 } });
		expect((await collab(carol.token).share(await shareBody(id, carol, []))).body.error).toBe("share_id_taken");
	});

	it("needs identities: 403 identity_required for the owner, 409 unknown_member for a listed account", async () => {
		const id = newShareId();
		expect((await collab(alice.token).share(await shareBody(id, alice, []))).body.error).toBe("identity_required");
		await ensureIdentity(alice);
		const r = await collab(alice.token).share(await shareBody(id, alice, [{ account: bob, role: "view" }]));
		expect(r).toMatchObject({ status: 409, body: { error: "unknown_member", account: bob.creds.accountId } });
	});

	it("checks the list, the wraps and the snapshot", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const id = newShareId();
		const c = collab(alice.token);
		const good = await shareBody(id, alice, [{ account: bob, role: "view" }]);
		const cases: [unknown, number, string][] = [
			[{ ...good, list: memberList(id, 2, alice, [{ account: alice, role: "owner" }]) }, 400, "bad_request"],
			[{ ...good, list: memberList(newShareId(), 1, alice, [{ account: alice, role: "owner" }]) }, 400, "bad_request"],
			[{ ...good, list: memberList(id, 1, alice, [{ account: alice, role: "edit" }, { account: bob, role: "owner" }]) }, 400, "bad_request"],
			[{ ...good, list: "!!" }, 400, "bad_request"],
			[{ ...good, list_sig: b64(63) }, 400, "bad_request"],
			[{ ...good, wraps: good.wraps.slice(1) }, 400, "bad_request"],
			[{ ...good, wraps: [...good.wraps.slice(1), { account: carol.creds.accountId, wrapped: b64(105) }] }, 400, "bad_request"],
			[{ ...good, snapshot: base64url(await envelope({ kind: "u", epoch: 1, shareId: id, by: alice })) }, 400, "bad_request"],
			[{ ...good, snapshot: base64url(await envelope({ kind: "s", epoch: 2, shareId: id, by: alice })) }, 409, "epoch_changed"],
			[{ ...good, snapshot: base64url(await envelope({ kind: "s", epoch: 1, shareId: id, by: bob })) }, 403, "forbidden_author"],
			[{ ...good, share_id: "short" }, 400, "bad_request"],
		];
		for (const [body, status, code] of cases) {
			expect(await c.share(body), code).toMatchObject({ status, body: { error: code } });
		}
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_shares").first("n")).toBe(0);
	});

	it("at most 20 members (413), 200 owned shares (413), and nothing new past the document-byte quota (413)", async () => {
		await ensureIdentity(alice);
		const id = newShareId();
		const tooMany = await shareBody(id, alice, []);
		const list = JSON.parse(new TextDecoder().decode(fromBase64url(tooMany.list)!));
		list.members = [
			...list.members,
			...Array.from({ length: 20 }, (_, i) => ({
				account: `a_${"f".repeat(30)}${String(i).padStart(2, "0")}`,
				username: `u${i}x`,
				role: "view",
				sign_pub: b64(32),
				box_pub: b64(32),
			})),
		].sort((x: { account: string }, y: { account: string }) => (x.account < y.account ? -1 : 1));
		const r = await collab(alice.token).share({ ...tooMany, list: base64url(new TextEncoder().encode(JSON.stringify(list))) });
		expect(r).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "members", used: 21, limit: 20 } });

		// 200 shares already (rows written directly; their rooms don't matter here).
		await env.DB.batch(
			Array.from({ length: 200 }, () =>
				env.DB.prepare("INSERT INTO collab_shares (id, owner_id, epoch, list, list_sig, created_at, updated_at) VALUES (?1, ?2, 1, 'AA', ?3, 1, 1)").bind(
					newShareId(),
					alice.creds.accountId,
					b64(64),
				),
			),
		);
		const capped = await collab(alice.token).share(await shareBody(newShareId(), alice, []));
		expect(capped).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "shares", used: 200, limit: 200 } });
		await env.DB.prepare("DELETE FROM collab_shares").run();

		await env.DB.prepare("UPDATE accounts SET quota_doc_bytes = 200 WHERE id = ?1").bind(alice.creds.accountId).run();
		const full = await collab(alice.token).share(await shareBody(newShareId(), alice, []));
		expect(full).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "doc_bytes", limit: 200 } });
	});
});

describe("membership", () => {
	it("an invitee can't read until they accept (404, as for a stranger)", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }], false);
		expect((await collab(bob.token).log(id)).status).toBe(404);
		expect((await collab(carol.token).log(id)).status).toBe(404);
		expect((await collab(carol.token).accept(id)).status).toBe(404);
		expect((await collab(bob.token).accept(id)).status).toBe(204);
		expect((await collab(bob.token).accept(id)).status).toBe(204);
		const log = await collab(bob.token).log(id);
		// since=0 returns the epoch-1 snapshot, whose base is 0.
		expect(log).toMatchObject({ status: 200, body: { head: 0, snapshot: { seq: 0, base_seq: 0 }, entries: [], more: false } });
	});

	it("leaving or declining takes the rows and wraps; the owner can't leave", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		await shareWith(alice, [{ account: carol, role: "view" }], false);
		expect((await collab(alice.token).leave(id)).body.error).toBe("forbidden_role");
		expect((await collab(bob.token).leave(id)).status).toBe(204);
		expect((await collab(bob.token).leave(id)).status).toBe(404);
		expect((await collab(bob.token).log(id)).status).toBe(404);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_wraps WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);
		const invited = (await collab(carol.token).shares()).body.shares[0].share_id;
		expect((await collab(carol.token).leave(invited)).status).toBe(204);
		expect((await collab(carol.token).shares()).body.shares).toEqual([]);
	});

	it("someone who left stays out: the next epoch, which still lists them, gives no row or wrap; accept and reads are 404; the owner sees them as left", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "view" },
		]);
		expect((await collab(bob.token).leave(id)).status).toBe(204);
		// The owner's next list still names bob (they don't know yet): bob is not brought back.
		const both = [
			{ account: bob, role: "edit" as const },
			{ account: carol, role: "view" as const },
		];
		expect((await collab(alice.token).epoch(id, epochBody(id, 2, alice, both))).status).toBe(200);
		expect((await collab(bob.token).accept(id)).status).toBe(404);
		expect((await collab(bob.token).log(id)).status).toBe(404);
		expect((await collab(bob.token).shares()).body.shares).toEqual([]);
		const left = (a: Account) => env.DB.prepare("SELECT count(*) AS n FROM collab_wraps WHERE account_id = ?1").bind(a.creds.accountId).first("n");
		expect(await left(bob)).toBe(0);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_members WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);
		expect(await left(carol)).toBe(2);
		const owned = (await collab(alice.token).shares()).body.shares[0];
		expect(owned.members).toContainEqual({ account: bob.creds.accountId, status: "left" });
		// Epochs that keep naming them keep them out; one that drops them clears the record, and a later list may invite them afresh.
		expect((await collab(alice.token).epoch(id, epochBody(id, 3, alice, both))).status).toBe(200);
		expect(await left(bob)).toBe(0);
		expect((await collab(alice.token).epoch(id, epochBody(id, 4, alice, [{ account: carol, role: "view" }]))).status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_left").first("n")).toBe(0);
		expect((await collab(alice.token).epoch(id, epochBody(id, 5, alice, both))).status).toBe(200);
		expect((await collab(bob.token).shares()).body.shares[0]).toMatchObject({ status: "invited" });
	});

	it("leaving in the same epoch, a role change for the leaver gives them nothing either", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		await collab(bob.token).leave(id);
		const raised = [{ account: bob, role: "edit" as const }];
		expect((await collab(alice.token).epoch(id, epochBody(id, 2, alice, raised))).status).toBe(200);
		expect((await collab(bob.token).log(id)).status).toBe(404);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_wraps WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);
	});
});

describe("envelopes by role", () => {
	it("view writes nothing; comment writes comments; edit and owner write updates and snapshots", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "view" },
			{ account: carol, role: "comment" },
		]);
		const u = (by: Account) => envelope({ kind: "u", epoch: 1, shareId: id, by });
		const c = (by: Account) => envelope({ kind: "c", epoch: 1, shareId: id, by });
		expect((await collab(bob.token).append(id, await u(bob))).body.error).toBe("forbidden_role");
		expect((await collab(bob.token).append(id, await c(bob))).body.error).toBe("forbidden_role");
		expect((await collab(carol.token).append(id, await u(carol))).body.error).toBe("forbidden_role");
		expect(await collab(carol.token).append(id, await c(carol))).toEqual({ status: 200, body: { seq: 1 } });
		expect(await collab(alice.token).append(id, await u(alice))).toEqual({ status: 200, body: { seq: 2 } });
		const s = await envelope({ kind: "s", epoch: 1, shareId: id, by: carol });
		expect((await collab(carol.token).snapshot(id, 2, s)).body.error).toBe("forbidden_role");
	});

	it("checks format, kind, epoch, author, device and size", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const c = collab(bob.token);
		const good = await envelope({ kind: "u", epoch: 1, shareId: id, by: bob });
		const wrongFormat = good.slice();
		wrongFormat[0] = 2;
		expect((await c.append(id, wrongFormat)).body.error).toBe("bad_request");
		expect((await c.append(id, good.slice(0, 120))).body.error).toBe("bad_request");
		expect((await c.append(id, await envelope({ kind: "s", epoch: 1, shareId: id, by: bob }))).body.error).toBe("bad_request");
		expect((await c.append(id, await envelope({ kind: "p", epoch: 1, shareId: id, by: bob }))).body.error).toBe("bad_request");
		expect(await c.append(id, await envelope({ kind: "u", epoch: 2, shareId: id, by: bob }))).toEqual({
			status: 409,
			body: expect.objectContaining({ error: "epoch_changed", epoch: 1 }),
		});
		expect((await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: bob, author: alice.creds.accountId }))).body.error).toBe(
			"forbidden_author",
		);
		expect((await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: bob, device: alice.device.id }))).body.error).toBe(
			"forbidden_author",
		);
		const big = await envelopeOfSize({ kind: "c", epoch: 1, shareId: id, by: bob }, MAX_ENVELOPE.c + 1);
		expect((await c.append(id, big)).status).toBe(413);
		const fits = await envelopeOfSize({ kind: "c", epoch: 1, shareId: id, by: bob }, MAX_ENVELOPE.c);
		expect((await c.append(id, fits)).status).toBe(200);
		const huge = await envelopeOfSize({ kind: "u", epoch: 1, shareId: id, by: bob }, MAX_ENVELOPE.u + 1);
		expect((await c.append(id, huge)).body.error).toBe("too_large");
		expect((await c.append(id, await envelopeOfSize({ kind: "u", epoch: 1, shareId: id, by: bob }, MAX_ENVELOPE.u))).status).toBe(200);
		const json = await callJson(`/v2/collab/shares/${id}/log`, { token: bob.token, json: { envelope: "x" } });
		expect(json.status).toBe(415);
	});
});

describe("snapshots and the log", () => {
	it("compare-and-swap on the head; a snapshot takes its base as seq, replaces earlier updates but keeps comments", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const c = collab(bob.token);
		const sent: Record<number, string> = {};
		for (const kind of ["u", "c", "u"] as const) {
			const bytes = await envelope({ kind, epoch: 1, shareId: id, by: bob });
			const r = await c.append(id, bytes);
			sent[r.body.seq] = base64url(bytes);
		}
		const snap = await envelope({ kind: "s", epoch: 1, shareId: id, by: bob });
		expect(await c.snapshot(id, 2, snap)).toEqual({ status: 409, body: expect.objectContaining({ error: "head_moved", head: 3 }) });
		expect(await c.snapshot(id, 3, snap)).toEqual({ status: 200, body: { seq: 3 } });
		const log = (await c.log(id)).body;
		expect(log).toEqual({ head: 3, snapshot: { seq: 3, base_seq: 3, envelope: base64url(snap) }, entries: [{ seq: 2, envelope: sent[2] }], more: false });
		expect((await c.log(id, 2)).body.snapshot).toMatchObject({ seq: 3 });
		expect((await c.log(id, 3)).body).toEqual({ head: 3, snapshot: null, entries: [], more: false });
		expect((await c.snapshot(id, 2, snap)).body.error).toBe("head_moved");
		const row = await env.DB.prepare("SELECT bytes FROM collab_shares WHERE id = ?1").bind(id).first<{ bytes: number }>();
		// The room's size: the snapshot and the comment that stays.
		expect(row!.bytes).toBe(snap.length + fromBase64url(sent[2])!.length);
		expect((await callJson(`/v2/collab/shares/${id}/snapshot`, { token: bob.token, method: "POST", body: snap, headers: { "content-type": "application/octet-stream" } })).status).toBe(400);
	});

	it("pages by limit, with more; limit outside 1–1000 is 400", async () => {
		const id = await shareWith(alice, []);
		const c = collab(alice.token);
		for (let i = 0; i < 5; i++) await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }));
		const first = (await c.log(id, 0, 2)).body;
		expect(first.snapshot.seq).toBe(0);
		expect(first.entries.map((e: { seq: number }) => e.seq)).toEqual([1, 2]);
		expect(first.more).toBe(true);
		const last = (await c.log(id, 4, 2)).body;
		expect(last).toMatchObject({ head: 5, snapshot: null, more: false });
		expect(last.entries.map((e: { seq: number }) => e.seq)).toEqual([5]);
		for (const q of ["limit=0", "limit=1001", "since=-1", "since=x"]) {
			expect((await callJson(`/v2/collab/shares/${id}/log?${q}`, { token: alice.token })).status).toBe(400);
		}
	});

	it("409 compact_first past 10,000 entries since the snapshot, until a snapshot", async () => {
		const id = await shareWith(alice, []);
		const stub = env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id));
		await runInDurableObject(stub, (_room, state) => {
			state.storage.sql.exec("UPDATE meta SET value = 10000 WHERE name = 'entries_since'");
		});
		const c = collab(alice.token);
		expect((await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }))).body.error).toBe("compact_first");
		expect((await c.snapshot(id, 0, await envelope({ kind: "s", epoch: 1, shareId: id, by: alice }))).status).toBe(200);
		expect((await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }))).status).toBe(200);
	});

	it("429 rate_limited room-bytes past 64 MiB a UTC day", async () => {
		const id = await shareWith(alice, []);
		const stub = env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id));
		await runInDurableObject(stub, (_room, state) => {
			state.storage.sql.exec("UPDATE meta SET value = ? WHERE name = 'day_bytes'", 64 * 1024 * 1024 - 100);
		});
		const r = await collab(alice.token).append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }));
		expect(r).toMatchObject({ status: 429, body: { error: "rate_limited", limit: "room-bytes" } });
		expect(r.body.retry_after).toBeGreaterThan(0);
		// The next UTC day starts over.
		const later = await callJson(`/v2/collab/shares/${id}/log`, {
			token: alice.token,
			now: START + DAY,
			body: await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }),
			headers: { "content-type": "application/octet-stream" },
		});
		expect(later.status).toBe(200);
	});
});

describe("epochs", () => {
	it("only the owner, only current + 1 (409 epoch_changed with the epoch); a retry answers 200", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const next = epochBody(id, 2, alice, [{ account: bob, role: "comment" }]);
		expect((await collab(bob.token).epoch(id, next)).body.error).toBe("forbidden_role");
		expect((await collab(carol.token).epoch(id, next)).status).toBe(404);
		expect(await collab(alice.token).epoch(id, epochBody(id, 3, alice, []))).toMatchObject({ status: 409, body: { error: "epoch_changed", epoch: 1 } });
		expect(await collab(alice.token).epoch(id, next)).toMatchObject({ status: 200, body: { epoch: 2 } });
		expect(await collab(alice.token).epoch(id, next)).toMatchObject({ status: 200, body: { epoch: 2 } });
		expect(await collab(alice.token).epoch(id, epochBody(id, 2, alice, []))).toMatchObject({ status: 409, body: { epoch: 2 } });
		const theirs = (await collab(bob.token).shares()).body.shares[0];
		expect(theirs).toMatchObject({ role: "comment", status: "member", epoch: 2, list: next.list });
		expect(theirs.wraps.map((w: { epoch: number }) => w.epoch)).toEqual([1, 2]);
		// Envelopes of the old epoch are refused now.
		expect((await collab(alice.token).append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }))).body).toMatchObject({
			error: "epoch_changed",
			epoch: 2,
		});
		expect((await collab(bob.token).append(id, await envelope({ kind: "u", epoch: 2, shareId: id, by: bob }))).body.error).toBe("forbidden_role");
		expect((await collab(bob.token).append(id, await envelope({ kind: "c", epoch: 2, shareId: id, by: bob }))).status).toBe(200);
	});

	it("a removed member loses rows, wraps, notifications and the log; a new one is invited", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		await ensureIdentity(carol);
		await collab(alice.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(200) });
		expect(await collab(alice.token).epoch(id, epochBody(id, 2, alice, [{ account: carol, role: "view" }]))).toMatchObject({ status: 200 });
		expect((await collab(bob.token).log(id)).status).toBe(404);
		expect((await collab(bob.token).shares()).body.shares).toEqual([]);
		expect((await collab(bob.token).notifications()).body.notifications).toEqual([]);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_wraps WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);
		expect((await collab(carol.token).shares()).body.shares[0]).toMatchObject({ status: "invited", role: "view", epoch: 2 });
		expect((await collab(carol.token).shares()).body.shares[0].wraps.map((w: { epoch: number }) => w.epoch)).toEqual([2]);
	});
});

describe("from_seq", () => {
	it("is required from epoch 2, refused at epoch 1, and must be the head (409 head_moved {head}); an applied retry still answers 200", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const early = await shareBody(newShareId(), alice, []);
		const withFrom = { ...early, list: memberList(early.share_id, 1, alice, [{ account: alice, role: "owner" }], 0) };
		expect((await collab(alice.token).share(withFrom)).body.error).toBe("bad_request");

		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const c = collab(alice.token);
		const missing = epochBody(id, 2, alice, [], null as unknown as number);
		expect((await c.epoch(id, missing)).body.error).toBe("bad_request");
		expect((await c.epoch(id, epochBody(id, 2, alice, [], -1))).body.error).toBe("bad_request");
		for (let i = 0; i < 3; i++) await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice }));
		expect(await c.epoch(id, epochBody(id, 2, alice, [], 2))).toMatchObject({ status: 409, body: { error: "head_moved", head: 3 } });
		expect(await c.epoch(id, epochBody(id, 2, alice, [], 4))).toMatchObject({ status: 409, body: { error: "head_moved", head: 3 } });
		const good = epochBody(id, 2, alice, [{ account: bob, role: "edit" }], 3);
		expect(await c.epoch(id, good)).toMatchObject({ status: 200, body: { epoch: 2 } });
		await collab(bob.token).append(id, await envelope({ kind: "u", epoch: 2, shareId: id, by: bob }));
		expect(await c.epoch(id, good)).toMatchObject({ status: 200, body: { epoch: 2 } });
		const list = JSON.parse(new TextDecoder().decode(fromBase64url(good.list)!));
		expect(Object.keys(list)).toEqual(["v", "share", "epoch", "owner", "key_commit", "at", "from_seq", "members"]);
	});
});

describe("room storage caps", () => {
	it("an append past the owner's document quota is 413 quota_exceeded doc_bytes, counting their documents and every shared note", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const other = await shareWith(alice, []);
		const bytesOf = async (share: string) => (await env.DB.prepare("SELECT bytes FROM collab_shares WHERE id = ?1").bind(share).first<number>("bytes"))!;
		const u = await envelope({ kind: "u", epoch: 1, shareId: id, by: bob, size: 100 });
		const used = 1000 + (await bytesOf(id)) + (await bytesOf(other));
		await env.DB.prepare("UPDATE accounts SET doc_bytes = 1000, quota_doc_bytes = ?2 WHERE id = ?1").bind(alice.creds.accountId, used + u.length - 1).run();
		expect(await collab(bob.token).append(id, u)).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "doc_bytes", used, limit: used + u.length - 1 } });
		await env.DB.prepare("UPDATE accounts SET quota_doc_bytes = ?2 WHERE id = ?1").bind(alice.creds.accountId, used + u.length).run();
		expect((await collab(bob.token).append(id, u)).status).toBe(200);
	});

	it("comments stop at 8 MiB per room: 413 quota_exceeded comment_bytes", async () => {
		const id = await shareWith(alice, []);
		await runInDurableObject(env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)), (_room, state) => {
			state.storage.sql.exec("UPDATE meta SET value = ? WHERE name = 'comment_bytes'", 8 * 1024 * 1024 - 300);
		});
		const c = collab(alice.token);
		const big = await envelope({ kind: "c", epoch: 1, shareId: id, by: alice, size: 200 });
		expect(await c.append(id, big)).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "comment_bytes", used: 8 * 1024 * 1024 - 300, limit: 8 * 1024 * 1024 } });
		// Updates aren't comments.
		expect((await c.append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, size: 200 }))).status).toBe(200);
		expect((await c.append(id, await envelope({ kind: "c", epoch: 1, shareId: id, by: alice, size: 10 }))).status).toBe(200);
	});

	it("collab_shares.bytes follows the room: at once past 64 KiB of growth, else on the alarm", async () => {
		const id = await shareWith(alice, []);
		const stub = env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id));
		const bytesOf = async () => (await env.DB.prepare("SELECT bytes FROM collab_shares WHERE id = ?1").bind(id).first<number>("bytes"))!;
		const start = await bytesOf();
		const small = await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, size: 1000 });
		await collab(alice.token).append(id, small);
		expect(await bytesOf()).toBe(start);
		expect(await runDurableObjectAlarm(stub)).toBe(true);
		expect(await bytesOf()).toBe(start + small.length);
		const big = await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, size: 70 * 1024 });
		await collab(alice.token).append(id, big);
		expect(await bytesOf()).toBe(start + small.length + big.length);
	});
});

describe("wraps of earlier epochs", () => {
	it("a new epoch may carry earlier epochs' wraps for listed accounts, replacing any held, within 20 × epoch (at most 400)", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		await ensureIdentity(carol);
		const body = epochBody(id, 2, alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "comment" },
		]);
		const old = b64(105);
		const replaced = b64(105);
		const withHistory = {
			...body,
			wraps: [
				...body.wraps,
				{ account: carol.creds.accountId, wrapped: old, epoch: 1 },
				{ account: bob.creds.accountId, wrapped: replaced, epoch: 1 },
			],
		};
		const c = collab(alice.token);
		const bad = [
			{ ...body, wraps: [...body.wraps, { account: carol.creds.accountId, wrapped: old, epoch: 3 }] },
			{ ...body, wraps: [...body.wraps, { account: carol.creds.accountId, wrapped: old, epoch: 0 }] },
			{ ...body, wraps: [...body.wraps, { account: carol.creds.accountId, wrapped: old, epoch: 2 }] },
			{ ...body, wraps: [...body.wraps.slice(1), { account: body.wraps[0].account, wrapped: old, epoch: 1 }] },
			{ ...body, wraps: [...body.wraps, { account: "a_" + "0".repeat(32), wrapped: old, epoch: 1 }] },
			{ ...body, wraps: [...body.wraps, ...Array.from({ length: 38 }, () => ({ account: carol.creds.accountId, wrapped: old, epoch: 1 }))] },
		];
		for (const b of bad) expect((await c.epoch(id, b)).status).toBe(400);
		expect((await c.epoch(id, withHistory)).status).toBe(200);
		const carolWraps = (await collab(carol.token).shares()).body.shares[0].wraps;
		expect(carolWraps).toEqual([
			{ epoch: 1, wrapped: old },
			{ epoch: 2, wrapped: body.wraps.find((w) => w.account === carol.creds.accountId)!.wrapped },
		]);
		expect((await collab(bob.token).shares()).body.shares[0].wraps[0]).toEqual({ epoch: 1, wrapped: replaced });
	});
});

describe("every epoch's list", () => {
	const listsOf = async (a: Account) => {
		const share = (await collab(a.token).shares()).body.shares[0];
		return share ? share.lists : null;
	};

	it("is kept; a member gets the lists of the epochs it holds keys for and the current one", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		await ensureIdentity(carol);
		const id = newShareId();
		const first = await shareBody(id, alice, [{ account: bob, role: "edit" }]);
		expect((await collab(alice.token).share(first)).status).toBe(201);
		await collab(bob.token).accept(id);
		const one = { epoch: 1, list: first.list, list_sig: first.list_sig };
		expect(await listsOf(alice)).toEqual([one]);
		expect(await listsOf(bob)).toEqual([one]);

		const second = epochBody(id, 2, alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "view" },
		]);
		expect((await collab(alice.token).epoch(id, second)).status).toBe(200);
		const two = { epoch: 2, list: second.list, list_sig: second.list_sig };
		expect(await listsOf(alice)).toEqual([one, two]);
		expect(await listsOf(bob)).toEqual([one, two]);
		// Carol holds no key of epoch 1, so she gets only the current list.
		expect(await listsOf(carol)).toEqual([two]);
		const top = (await collab(carol.token).shares()).body.shares[0];
		expect({ list: top.list, list_sig: top.list_sig }).toEqual({ list: second.list, list_sig: second.list_sig });

		// Handed epoch 1's key later, she gets epoch 1's list too.
		const third = epochBody(id, 3, alice, [{ account: carol, role: "view" }]);
		const withHistory = { ...third, wraps: [...third.wraps, { account: carol.creds.accountId, wrapped: b64(105), epoch: 1 }] };
		expect((await collab(alice.token).epoch(id, withHistory)).status).toBe(200);
		const three = { epoch: 3, list: third.list, list_sig: third.list_sig };
		expect(await listsOf(carol)).toEqual([one, two, three]);
		// Bob, removed, gets nothing.
		expect(await listsOf(bob)).toBeNull();
		// A retried epoch keeps the list it stored.
		expect((await collab(alice.token).epoch(id, withHistory)).status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_lists WHERE share_id = ?1").bind(id).first("n")).toBe(3);

		expect((await collab(alice.token).stop(id)).status).toBe(204);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_lists").first("n")).toBe(0);
	});

	it("goes with an owner's account", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const theirs = await shareWith(carol, [{ account: alice, role: "view" }]);
		const deleted = await callJson("/v2/account/delete", { token: alice.token, json: { auth_key: alice.creds.authKey, username: "alice" } });
		expect(deleted.status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_lists WHERE share_id = ?1").bind(id).first("n")).toBe(0);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_lists WHERE share_id = ?1").bind(theirs).first("n")).toBe(1);
	});
});

describe("listed members' keys", () => {
	it("must be the account's own keys and current username: 409 member_keys_mismatch {account}, on sharing and on a new epoch", async () => {
		const aliceId = await ensureIdentity(alice);
		await ensureIdentity(bob);
		await ensureIdentity(carol);
		const forged = [
			{ sign_pub: aliceId.sign_pub, box_pub: aliceId.box_pub },
			{ sign_pub: b64(32) },
			{ box_pub: b64(32) },
			{ username: "carol" },
			{ username: "Bob" },
		];
		for (const claims of forged) {
			const id = newShareId();
			const r = await collab(alice.token).share(await shareBody(id, alice, [{ account: bob, role: "edit", claims }]));
			expect(r, JSON.stringify(claims)).toMatchObject({ status: 409, body: { error: "member_keys_mismatch", account: bob.creds.accountId } });
		}
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_shares").first("n")).toBe(0);
		// The owner's own entry too.
		const own = await collab(alice.token).share(
			await shareBody(newShareId(), alice, []).then((b) => ({
				...b,
				list: memberList(b.share_id, 1, alice, [{ account: alice, role: "owner", claims: { box_pub: b64(32) } }]),
			})),
		);
		expect(own.body).toMatchObject({ error: "member_keys_mismatch", account: alice.creds.accountId });

		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		for (const claims of forged) {
			const r = await collab(alice.token).epoch(id, epochBody(id, 2, alice, [{ account: carol, role: "view", claims: { ...claims, username: claims.username === "carol" ? "bob" : claims.username } }]));
			expect(r, JSON.stringify(claims)).toMatchObject({ status: 409, body: { error: "member_keys_mismatch", account: carol.creds.accountId } });
		}
		expect((await collab(carol.token).shares()).body.shares).toEqual([]);
		expect((await collab(alice.token).epoch(id, epochBody(id, 2, alice, [{ account: carol, role: "view" }]))).status).toBe(200);
	});
});

describe("links and requests", () => {
	it("make, join with the proof (202), approve through an epoch (member at once), decline, cancel", async () => {
		const id = await shareWith(alice, []);
		const { proof, hash } = await linkProof();
		const linkId = newShareId();
		const expires = START + 7 * DAY;
		expect(await collab(alice.token).link(id, { link_id: linkId, proof_hash: hash, role: "comment", expires_at: expires })).toMatchObject({
			status: 201,
			body: { link_id: linkId, role: "comment", expires_at: expires },
		});
		const bobId = await ensureIdentity(bob);
		expect((await collab(bob.token).join(linkId, b64(32))).status).toBe(404);
		expect((await collab(bob.token).join(newShareId(), proof)).status).toBe(404);
		expect(await collab(bob.token).join(linkId, proof)).toMatchObject({ status: 202, body: { share_id: id, owner_username: "alice", role: "comment" } });
		const owner = (await collab(alice.token).shares()).body.shares[0];
		expect(owner.links).toEqual([{ link_id: linkId, role: "comment", created_at: START, expires_at: expires }]);
		expect(owner.requests).toEqual([
			{
				account: bob.creds.accountId,
				username: "bob",
				link_id: linkId,
				role: "comment",
				requested_at: START,
				sign_pub: bobId.sign_pub,
				box_pub: bobId.box_pub,
				binding: bobId.binding,
			},
		]);
		expect(await collab(alice.token).epoch(id, epochBody(id, 2, alice, [{ account: bob, role: "comment" }]))).toMatchObject({ status: 200 });
		expect((await collab(bob.token).shares()).body.shares[0]).toMatchObject({ status: "member", role: "comment" });
		expect((await collab(bob.token).log(id)).status).toBe(200);
		expect((await collab(alice.token).shares()).body.shares[0].requests).toEqual([]);
		// Already in: 202 again, and no request.
		expect((await collab(bob.token).join(linkId, proof)).status).toBe(202);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_requests").first("n")).toBe(0);

		await ensureIdentity(carol);
		expect((await collab(carol.token).join(linkId, proof)).status).toBe(202);
		expect((await collab(carol.token).decline(id, carol.creds.accountId)).status).toBe(404);
		expect((await collab(alice.token).decline(id, carol.creds.accountId)).status).toBe(204);
		expect((await collab(alice.token).decline(id, carol.creds.accountId)).status).toBe(404);
		expect((await collab(alice.token).unlink(id, linkId)).status).toBe(204);
		expect((await collab(carol.token).join(linkId, proof)).status).toBe(404);
	});

	it("links expire within 30 days; an expired one answers 404; a joiner needs an identity", async () => {
		const id = await shareWith(alice, []);
		const { proof, hash } = await linkProof();
		const c = collab(alice.token);
		expect((await c.link(id, { link_id: newShareId(), proof_hash: hash, role: "edit", expires_at: START + 31 * DAY })).status).toBe(400);
		expect((await c.link(id, { link_id: newShareId(), proof_hash: hash, role: "owner", expires_at: START + DAY })).status).toBe(400);
		expect((await c.link(id, { link_id: newShareId(), proof_hash: hash, role: "edit", expires_at: START })).status).toBe(400);
		const linkId = newShareId();
		expect((await c.link(id, { link_id: linkId, proof_hash: hash, role: "view", expires_at: START + DAY })).status).toBe(201);
		expect((await collab(bob.token).join(linkId, proof)).body.error).toBe("identity_required");
		await ensureIdentity(bob);
		expect((await callJson(`/v2/collab/join/${linkId}`, { token: bob.token, json: { proof }, now: START + DAY })).status).toBe(404);
		expect((await collab(bob.token).link(id, { link_id: newShareId(), proof_hash: hash, role: "view", expires_at: START + DAY })).status).toBe(404);
	});

	it("at most 10 live links and 50 pending requests per note (413); joins counted by join-account", async () => {
		const id = await shareWith(alice, []);
		const { proof, hash } = await linkProof();
		const links: string[] = [];
		for (let i = 0; i < 10; i++) {
			const linkId = newShareId();
			links.push(linkId);
			expect((await collab(alice.token).link(id, { link_id: linkId, proof_hash: hash, role: "view", expires_at: START + DAY })).status).toBe(201);
		}
		const over = await collab(alice.token).link(id, { link_id: newShareId(), proof_hash: hash, role: "view", expires_at: START + DAY });
		expect(over).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "links", used: 10, limit: 10 } });

		await setSetting("max_accounts", "100");
		await setSetting("signups_per_day", "100");
		for (let i = 0; i < 50; i++) {
			const someone = await signUp(`asker${i}`, { ip: `198.51.100.${i}` });
			await ensureIdentity(someone);
			expect((await collab(someone.token).join(links[0], proof)).status).toBe(202);
		}
		await ensureIdentity(bob);
		expect(await collab(bob.token).join(links[0], proof)).toMatchObject({ status: 413, body: { quota: "requests", used: 50, limit: 50 } });

		await env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, 20)").bind(`join-account:${carol.creds.accountId}`, START).run();
		expect((await collab(carol.token).join(links[0], proof)).body).toMatchObject({ error: "rate_limited", limit: "join-account" });
	});
});

describe("notifications", () => {
	it("from a member who may comment to a member or invitee; the recipient lists and dismisses them", async () => {
		const id = await shareWith(alice, [
			{ account: bob, role: "view" },
			{ account: carol, role: "comment" },
		]);
		const sealed = b64(300);
		expect((await collab(bob.token).notify({ share_id: id, recipient: carol.creds.accountId, sealed })).body.error).toBe("forbidden_role");
		const sent = await collab(carol.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed });
		expect(sent.status).toBe(201);
		expect(sent.body.id).toMatch(/^[A-Za-z0-9_-]{22}$/);
		const stranger = await signUp("dave");
		expect((await collab(carol.token).notify({ share_id: id, recipient: stranger.creds.accountId, sealed })).status).toBe(404);
		expect((await collab(stranger.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed })).status).toBe(404);
		expect((await collab(carol.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(4097) })).status).toBe(400);
		expect((await collab(carol.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(72) })).status).toBe(400);
		expect((await collab(bob.token).notifications()).body).toEqual({
			notifications: [{ id: sent.body.id, share_id: id, from: carol.creds.accountId, sealed, at: START }],
		});
		expect((await collab(carol.token).dismiss(sent.body.id)).status).toBe(404);
		expect((await collab(bob.token).dismiss(sent.body.id)).status).toBe(204);
		expect((await collab(bob.token).notifications()).body.notifications).toEqual([]);
	});

	it("keeps the newest 500 per recipient, and counts notify-account (500 a day)", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }]);
		await env.DB.batch(
			Array.from({ length: 500 }, (_, i) =>
				env.DB.prepare("INSERT INTO collab_notifications (id, recipient_id, share_id, sender_id, sealed, at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)").bind(
					newShareId(),
					bob.creds.accountId,
					id,
					alice.creds.accountId,
					b64(100),
					START - 1000 + i,
				),
			),
		);
		const oldest = (await collab(bob.token).notifications()).body.notifications[0];
		expect((await collab(alice.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(100) })).status).toBe(201);
		const kept = (await collab(bob.token).notifications()).body.notifications;
		expect(kept).toHaveLength(500);
		expect(kept.some((n: { id: string }) => n.id === oldest.id)).toBe(false);
		await env.DB.prepare("UPDATE limits SET count = 500 WHERE key = ?1").bind(`notify-account:${alice.creds.accountId}`).run();
		expect((await collab(alice.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(100) })).body).toMatchObject({
			error: "rate_limited",
			limit: "notify-account",
		});
	});
});

describe("stopping", () => {
	it("every row goes, the ID is retired, the room is purged", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const { hash } = await linkProof();
		await collab(alice.token).link(id, { link_id: newShareId(), proof_hash: hash, role: "view", expires_at: START + DAY });
		await collab(bob.token).append(id, await envelope({ kind: "u", epoch: 1, shareId: id, by: bob }));
		expect((await collab(bob.token).stop(id)).body.error).toBe("forbidden_role");
		expect((await collab(alice.token).stop(id)).status).toBe(204);
		expect((await collab(alice.token).stop(id)).status).toBe(404);
		for (const table of ["collab_shares", "collab_members", "collab_wraps", "collab_links", "collab_purges"]) {
			expect(await env.DB.prepare(`SELECT count(*) AS n FROM ${table}`).first("n"), table).toBe(0);
		}
		expect(await env.DB.prepare("SELECT id FROM collab_retired").first("id")).toBe(id);
		expect(await roomState(id)).toEqual({ head: 0, epoch: 0 });
		expect((await collab(alice.token).share(await shareBody(id, alice, []))).body.error).toBe("share_retired");
	});
});

describe("account deletion and the cron", () => {
	const deleteAccount = (a: Account) => callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: a.creds.username } });

	it("a member's deletion takes its identity and place; an owner's stops its shares and purges their rooms", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const theirs = await shareWith(carol, [{ account: alice, role: "view" }]);
		await collab(carol.token).notify({ share_id: theirs, recipient: alice.creds.accountId, sealed: b64(100) });
		expect((await deleteAccount(bob)).status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM identities WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_members WHERE account_id = ?1").bind(bob.creds.accountId).first("n")).toBe(0);

		expect((await deleteAccount(alice)).status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_shares WHERE id = ?1").bind(id).first("n")).toBe(0);
		expect(await env.DB.prepare("SELECT id FROM collab_retired").first("id")).toBe(id);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_purges").first("n")).toBe(0);
		expect(await roomState(id)).toEqual({ head: 0, epoch: 0 });
		// Carol's note stays, without Alice.
		expect((await collab(carol.token).shares()).body.shares[0].members).toEqual([{ account: carol.creds.accountId, status: "member" }]);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_notifications").first("n")).toBe(0);
		for (const rule of ["lookup-account", "join-account", "notify-account"]) {
			expect(await env.DB.prepare("SELECT count(*) AS n FROM limits WHERE key = ?1").bind(`${rule}:${alice.creds.accountId}`).first("n")).toBe(0);
		}
	});

	it("admin delete queues the rooms; the cron purges them and reports collabPurged", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		expect((await runAdmin(["delete", "alice", "--confirm", "alice"])).ok).toBe(true);
		expect(await env.DB.prepare("SELECT id FROM collab_purges").first("id")).toBe(id);
		expect(await roomState(id)).toEqual({ head: 0, epoch: 1 });
		const report = await runCron(testEnv(), START + 1);
		expect(report.collabPurged).toBe(1);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_purges").first("n")).toBe(0);
		expect(await roomState(id)).toEqual({ head: 0, epoch: 0 });
		expect((await collab(bob.token).shares()).body.shares).toEqual([]);
		expect((await runCron(testEnv(), START + 2)).collabPurged).toBe(0);
	});

	it("the cron drops expired links", async () => {
		const id = await shareWith(alice, []);
		const { hash } = await linkProof();
		await collab(alice.token).link(id, { link_id: newShareId(), proof_hash: hash, role: "view", expires_at: START + DAY });
		await runCron(testEnv(), START + DAY);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_links").first("n")).toBe(0);
	});
});

describe("the Worker's rules", () => {
	it("refuses an Origin header on every collab route", async () => {
		const id = await shareWith(alice, []);
		for (const path of ["/v2/collab/shares", `/v2/collab/shares/${id}/log`, "/v2/account/identity", "/v2/users/bob", "/v2/collab/notifications"]) {
			const r = await callJson(path, { token: alice.token, headers: { origin: "https://evil.example" } });
			expect(r, path).toMatchObject({ status: 403, body: { error: "forbidden_origin" } });
		}
	});

	it("the live route needs an upgrade (426) and a member", async () => {
		const id = await shareWith(alice, [{ account: bob, role: "view" }], false);
		expect((await callJson(`/v2/collab/shares/${id}/live`, { token: alice.token })).status).toBe(426);
		expect((await call(`/v2/collab/shares/${id}/live`, { token: bob.token, headers: { upgrade: "websocket" } })).status).toBe(404);
		expect((await call(`/v2/collab/shares/${id}/live`, { token: carol.token, headers: { upgrade: "websocket" } })).status).toBe(404);
		expect((await call(`/v2/collab/shares/${id}/live`, { headers: { upgrade: "websocket" } })).status).toBe(401);
	});

	it("unknown sub-routes are 404 and wrong methods 405", async () => {
		const id = newShareId();
		expect((await callJson(`/v2/collab/shares/${id}/nope`, { token: alice.token })).status).toBe(404);
		expect((await callJson(`/v2/collab/shares/${id}/epoch`, { token: alice.token })).status).toBe(405);
		expect((await callJson("/v2/collab/shares", { token: alice.token, method: "DELETE" })).status).toBe(405);
	});
});

describe("GET /j/<link_id>", () => {
	it("is a static page that tells the reader to open Kanso, with no script, from a browser too, never reading D1", async () => {
		const noDb = testEnv({ DB: new Proxy(env.DB, { get: () => () => { throw new Error("D1 read"); } }) as D1Database });
		const response = await call(`/j/${newShareId()}`, { env: noDb, headers: { origin: "https://example.com" } });
		expect(response.status).toBe(200);
		const html = await response.text();
		expect(html).toContain("Open Kanso, choose <strong>Join a shared note…</strong>, and paste this link.");
		expect(html).not.toContain("<script");
		const csp = response.headers.get("content-security-policy")!;
		expect(csp).toContain("default-src 'none'");
		expect(csp).not.toContain("script-src");
		expect(response.headers.get("x-frame-options")).toBe("DENY");
		expect(response.headers.get("referrer-policy")).toBe("no-referrer");
		expect((await call("/j/not-a-link", { env: noDb })).status).toBe(404);
		expect((await call(`/j/${newShareId()}`, { env: noDb, method: "POST" })).status).toBe(405);
	});

	it("is behind PUBLIC_LIMITER", async () => {
		const limiter = { limit: async () => ({ success: false }) } as unknown as RateLimit;
		const r = await call(`/j/${newShareId()}`, { env: testEnv({ PUBLIC_LIMITER: limiter }) });
		expect(r.status).toBe(429);
	});
});
