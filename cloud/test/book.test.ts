// Shared notebooks over HTTP (issue #390 › Shared notebooks: Worker; slice 33): sharing, the chain
// of signed member lists (append-only, rank and epoch rules), wraps and history links, roles
// (owner, manager, member, invited, stranger), the caps, links, rate limits, stopping and account
// deletion, and that a shared note still refuses the Manager role.

import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { sha256Hex } from "../src/crypto";
import { collab, ensureIdentity, newIdentity, rekeyIdentity, shareBody } from "./collab-helpers";
import { type Account, base64url, callJson, clock, resetServer, setSetting, signUp } from "./helpers";
import { advance, b64, bookList, books, commitOf, genesisBody, newShareId, nextBody, shareBook } from "./book-helpers";

const DAY = 86_400_000;

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

const count = async (table: string, where = "1 = 1", ...values: unknown[]): Promise<number> =>
	(await env.DB.prepare(`SELECT count(*) AS n FROM ${table} WHERE ${where}`).bind(...values).first<{ n: number }>())!.n;

async function linkBody(hours = 1) {
	const proof = crypto.getRandomValues(new Uint8Array(32));
	return { link_id: newShareId(), proof_hash: await sha256Hex(proof), role: "view", expires_at: clock.now + hours * 3_600_000 };
}

describe("creating a notebook", () => {
	it("stores the genesis list and the wraps; 201, and the same again 201", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const id = newShareId();
		const { body } = await genesisBody(id, alice, [{ account: bob, role: "edit" }]);
		expect(await books(alice.token).create(body)).toMatchObject({ status: 201, body: { share_id: id, epoch: 1, rev: 1 } });
		expect(await books(alice.token).create(body)).toMatchObject({ status: 201, body: { share_id: id, epoch: 1, rev: 1 } });
		expect(await count("book_lists", "share_id = ?1", id)).toBe(1);
		expect(await count("book_wraps", "share_id = ?1", id)).toBe(2);
		const rows = await env.DB.prepare("SELECT account_id, status, role FROM book_members WHERE share_id = ?1 ORDER BY role").bind(id).all<{ account_id: string; status: string; role: string }>();
		expect(rows.results.map((r) => [r.role, r.status])).toEqual([
			["edit", "invited"],
			["owner", "member"],
		]);
	});

	it("takes a manager in the list (note shares refuse the role)", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const { body } = await genesisBody(newShareId(), alice, [{ account: bob, role: "manager" }]);
		expect((await books(alice.token).create(body)).status).toBe(201);
		const note = await shareBody(newShareId(), alice, []);
		const list = JSON.parse(atob(note.list.replace(/-/g, "+").replace(/_/g, "/")));
		list.members.push({ ...list.members[0], account: bob.creds.accountId, role: "manager" });
		list.members.sort((x: { account: string }, y: { account: string }) => (x.account < y.account ? -1 : 1));
		const forged = base64url(new TextEncoder().encode(JSON.stringify(list)));
		expect((await collab(alice.token).share({ ...note, list: forged })).status).toBe(400);
	});

	it("refuses bodies and lists of the wrong shape (400)", async () => {
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const id = newShareId();
		const { body } = await genesisBody(id, alice, [{ account: bob, role: "view" }]);
		const c = books(alice.token);
		for (const bad of [
			{ ...body, share_id: "short" },
			{ ...body, list: "not base64url!" },
			{ ...body, list_sig: b64(10) },
			{ ...body, wraps: body.wraps.slice(1) },
			{ ...body, wraps: [...body.wraps, body.wraps[0]] },
			{ ...body, wraps: body.wraps.map((w) => ({ ...w, wrapped: b64(104) })) },
			{ ...body, wraps: [...body.wraps.slice(1), { account: carol.creds.accountId, wrapped: b64(105) }] },
		]) {
			expect(await c.create(bad)).toMatchObject({ status: 400 });
		}
		const owner = { account: alice, role: "owner" as const };
		const forge = async (spec: Parameters<typeof bookList>[0]) => ({ ...body, list: (await bookList(spec)).list });
		const base = { shareId: id, rev: 1, epoch: 1, owner: alice, signer: alice, members: [owner] };
		// A list that isn't a genesis, or isn't the caller's, or isn't this notebook's.
		expect((await c.create(await forge({ ...base, rev: 2, prev: "0".repeat(64) }))).status).toBe(400);
		expect((await c.create(await forge({ ...base, epoch: 2 }))).status).toBe(400);
		expect((await c.create(await forge({ ...base, signer: bob }))).status).toBe(400);
		expect((await c.create(await forge({ ...base, shareId: newShareId() }))).status).toBe(400);
		expect((await c.create(await forge({ ...base, members: [{ account: alice, role: "edit" }] }))).status).toBe(400);
		expect((await c.create(await forge({ ...base, extra: { role_extra: 1, key_commit: "xyz" } }))).status).toBe(400);
		expect((await books(bob.token).create(await forge({ ...base })) as { status: number }).status).toBe(400);
		expect(await count("book_shares")).toBe(0);
	});

	it("needs identity keys, and every listed member's keys must be the account's own", async () => {
		const id = newShareId();
		const { body } = await genesisBody(id, alice, []);
		await env.DB.prepare("DELETE FROM identities WHERE account_id = ?1").bind(alice.creds.accountId).run();
		expect(await books(alice.token).create(body)).toMatchObject({ status: 403, body: { error: "identity_required" } });
		await ensureIdentity(alice);
		await ensureIdentity(bob);
		const lie = await genesisBody(newShareId(), alice, [{ account: bob, role: "view", claims: { box_pub: newIdentity().box_pub } }]);
		expect(await books(alice.token).create(lie.body)).toMatchObject({ status: 409, body: { error: "member_keys_mismatch", account: bob.creds.accountId } });
		const ghost = await genesisBody(newShareId(), alice, [{ account: carol, role: "view" }]);
		await env.DB.prepare("DELETE FROM identities WHERE account_id = ?1").bind(carol.creds.accountId).run();
		expect(await books(alice.token).create(ghost.body)).toMatchObject({ status: 409, body: { error: "unknown_member" } });
	});

	it("needs a session", async () => {
		const { body } = await genesisBody(newShareId(), alice, []);
		expect((await callJson("/v2/collab/books", { json: body })).status).toBe(401);
		expect((await callJson("/v2/collab/books", { json: body, token: "kanso_nope" })).status).toBe(401);
	});

	it("answers 409 for an ID another notebook, a shared note or a stopped one holds", async () => {
		const book = await shareBook(alice, []);
		await ensureIdentity(bob);
		const other = await genesisBody(book.id, bob, []);
		expect(await books(bob.token).create(other.body)).toMatchObject({ status: 409, body: { error: "share_id_taken" } });
		// Different list for the same owner and ID is not a retry either.
		const again = await genesisBody(book.id, alice, []);
		expect(await books(alice.token).create(again.body)).toMatchObject({ status: 409, body: { error: "share_id_taken" } });
		const noteId = newShareId();
		expect((await collab(alice.token).share(await shareBody(noteId, alice, []))).status).toBe(201);
		const clash = await genesisBody(noteId, alice, []);
		expect(await books(alice.token).create(clash.body)).toMatchObject({ status: 409, body: { error: "share_id_taken" } });
		expect((await books(alice.token).stop(book.id)).status).toBe(204);
		const reuse = await genesisBody(book.id, alice, []);
		expect(await books(alice.token).create(reuse.body)).toMatchObject({ status: 409, body: { error: "share_retired" } });
	});
});

describe("reading a notebook", () => {
	it("gives members the chain, their own wraps and the history links; managers also the members and links", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "manager" },
			{ account: carol, role: "view" },
		]);
		const asAlice = await books(alice.token).get(book.id);
		expect(asAlice).toMatchObject({
			status: 200,
			body: { share_id: book.id, owner: alice.creds.accountId, role: "owner", status: "member", epoch: 1, rev: 1, hash: book.hash, more: false },
		});
		expect(asAlice.body.lists).toHaveLength(1);
		expect(asAlice.body.lists[0]).toMatchObject({ rev: 1, epoch: 1, signer: alice.creds.accountId });
		expect(asAlice.body.wraps).toHaveLength(1);
		expect(asAlice.body.members).toHaveLength(3);
		expect(asAlice.body.links).toEqual([]);
		const asBob = await books(bob.token).get(book.id);
		expect(asBob.body).toMatchObject({ role: "manager", status: "member" });
		expect(asBob.body.members).toHaveLength(3);
		const asCarol = await books(carol.token).get(book.id);
		expect(asCarol.body).toMatchObject({ role: "view", status: "member" });
		expect(asCarol.body.members).toBeUndefined();
		expect(asCarol.body.links).toBeUndefined();
		expect(asCarol.body.wraps).toHaveLength(1);
	});

	it("shows an invitation before it is accepted, and hides the notebook from anyone else (404)", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }], false);
		expect(await books(bob.token).get(book.id)).toMatchObject({ status: 200, body: { role: "edit", status: "invited" } });
		expect((await books(carol.token).get(book.id)).status).toBe(404);
		expect((await books(carol.token).get(newShareId())).status).toBe(404);
		expect((await books(carol.token).get("short")).status).toBe(400);
		expect((await callJson(`/v2/collab/books/${book.id}`)).status).toBe(401);
		// Declining ends it.
		expect((await books(bob.token).leave(book.id)).status).toBe(204);
		expect((await books(bob.token).get(book.id)).status).toBe(404);
		expect(await count("book_wraps", "account_id = ?1", bob.creds.accountId)).toBe(0);
	});

	it("pages the chain with since_rev and limit", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "view" }]);
		const withCarol = [...book.members, { account: carol, role: "view" as const }];
		expect((await advance(book, alice, withCarol, { wrapFor: [carol] })).status).toBe(200);
		const withDave = [...withCarol, { account: dave, role: "view" as const }];
		expect((await advance(book, alice, withDave, { wrapFor: [dave] })).status).toBe(200);
		const page = await books(alice.token).get(book.id, "?since_rev=1&limit=2");
		expect(page.body.lists.map((l: { rev: number }) => l.rev)).toEqual([1, 2]);
		expect(page.body.more).toBe(true);
		const rest = await books(alice.token).get(book.id, "?since_rev=3");
		expect(rest.body.lists.map((l: { rev: number }) => l.rev)).toEqual([3]);
		expect(rest.body.more).toBe(false);
		for (const q of ["?since_rev=0", "?limit=0", "?limit=201", "?since_rev=x"]) expect((await books(alice.token).get(book.id, q)).status).toBe(400);
	});
});

describe("the chain of member lists", () => {
	it("appends the next list in the same epoch: new members are invited and get a wrap; the chain reads back in order", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		const members = [...book.members, { account: carol, role: "comment" as const }];
		const r = await advance(book, alice, members, { wrapFor: [carol] });
		expect(r).toMatchObject({ status: 200, body: { epoch: 1, rev: 2, hash: book.hash } });
		expect((await books(carol.token).get(book.id)).body).toMatchObject({ role: "comment", status: "invited", rev: 2, epoch: 1 });
		const chain = (await books(carol.token).get(book.id)).body.lists;
		expect(chain.map((l: { rev: number; signer: string }) => [l.rev, l.signer])).toEqual([
			[1, alice.creds.accountId],
			[2, alice.creds.accountId],
		]);
		expect(await count("book_wraps", "share_id = ?1", book.id)).toBe(3);
	});

	it("a role change in the same epoch needs a new wrap (the wrap binds the role) and updates the role", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		const promoted = [{ account: alice, role: "owner" as const }, { account: bob, role: "manager" as const }];
		const missing = await nextBody(book, alice, promoted, { wrapFor: [] });
		expect((await books(alice.token).list(book.id, missing.body)).status).toBe(400);
		expect((await advance(book, alice, promoted, { wrapFor: [bob] })).status).toBe(200);
		expect((await books(bob.token).get(book.id)).body.role).toBe("manager");
	});

	it("refuses a rev that isn't head + 1 (409 rev_conflict with the head), and one that doesn't name the head", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		const members = [...book.members, { account: carol, role: "view" as const }];
		const next = await nextBody(book, alice, members, { wrapFor: [carol] });
		const conflict = { status: 409, body: { error: "rev_conflict", rev: 1, hash: book.hash } };
		// Skipping a rev.
		expect(await books(alice.token).list(book.id, { ...next.body, rev: 3 })).toMatchObject(conflict);
		// Going back over the genesis list.
		expect(await books(alice.token).list(book.id, { ...next.body, rev: 1 })).toMatchObject(conflict);
		// A list built on another parent.
		const wrongPrev = await nextBody({ ...book, hash: "ab".repeat(32) }, alice, members, { wrapFor: [carol] });
		expect(await books(alice.token).list(book.id, wrongPrev.body)).toMatchObject(conflict);
		// A body whose list says another rev than the body.
		expect((await books(alice.token).list(book.id, { ...next.body, rev: 2, list: (await nextBody({ ...book, rev: 5 }, alice, members, { wrapFor: [carol] })).body.list })).status).toBe(400);
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(1);
		// The right one still goes in, once: a different list for the same rev is refused, and the head repeated is a retry.
		expect((await books(alice.token).list(book.id, next.body)).status).toBe(200);
		expect((await books(alice.token).list(book.id, next.body)).status).toBe(200);
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(2);
		const fork = await nextBody(book, alice, [...book.members, { account: dave, role: "view" as const }], { wrapFor: [dave] });
		expect(await books(alice.token).list(book.id, fork.body)).toMatchObject({ status: 409, body: { error: "rev_conflict", rev: 2, hash: next.hash } });
		expect(await count("book_members", "share_id = ?1", book.id)).toBe(3);
	});

	it("never overwrites or deletes a stored list: the database refuses (append-only)", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		const attempt = async (sql: string, ...values: unknown[]) => {
			try {
				await env.DB.prepare(sql).bind(...values).run();
			} catch (err) {
				return String((err as Error).message) + String((err as { cause?: unknown }).cause);
			}
			return "accepted";
		};
		expect(await attempt("UPDATE book_lists SET list_sig = ?2 WHERE share_id = ?1", book.id, b64(64))).toContain("append_only");
		expect(await attempt("DELETE FROM book_lists WHERE share_id = ?1", book.id)).toContain("append_only");
		expect(await attempt("UPDATE book_epochs SET history_link = ?2 WHERE share_id = ?1", book.id, b64(73))).toBe("accepted");
		// A list at an existing rev, at a gap, or off another parent is refused whoever writes it.
		const row = await env.DB.prepare("SELECT * FROM book_lists WHERE share_id = ?1").bind(book.id).first<Record<string, string | number>>();
		const insert = (rev: number, prev: string | null, hash: string) =>
			attempt(
				"INSERT INTO book_lists (share_id, rev, epoch, signer, prev, hash, list, list_sig) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7)",
				book.id,
				rev,
				row!.signer,
				prev,
				hash,
				row!.list,
				row!.list_sig,
			);
		expect(await insert(1, null, "cd".repeat(32))).toMatch(/rev_conflict|UNIQUE/);
		expect(await insert(3, "ab".repeat(32), "cd".repeat(32))).toContain("rev_conflict");
		expect(await insert(2, "ab".repeat(32), "cd".repeat(32))).toContain("rev_conflict");
		expect(await insert(2, String(row!.hash), "cd".repeat(32))).toBe("accepted");
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(2);
		expect(await count("book_lists", "share_id = ?1 AND rev = 1 AND hash = ?2", book.id, row!.hash)).toBe(1);
	});

	it("races: two lists for the same rev, one wins, the other gets 409 and leaves no rows", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "manager" }]);
		const a = await nextBody(book, alice, [...book.members, { account: carol, role: "view" }], { wrapFor: [carol] });
		const b = await nextBody(book, bob, [...book.members, { account: dave, role: "view" }], { wrapFor: [dave] });
		const [ra, rb] = await Promise.all([books(alice.token).list(book.id, a.body), books(bob.token).list(book.id, b.body)]);
		expect([ra.status, rb.status].sort()).toEqual([200, 409]);
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(2);
		expect(await count("book_members", "share_id = ?1", book.id)).toBe(3);
		expect(await count("book_wraps", "share_id = ?1", book.id)).toBe(3);
	});

	it("checks the list against the chain: owner, signer, shape and the caller", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "manager" }]);
		const members = [...book.members, { account: carol, role: "view" as const }];
		const ok = await nextBody(book, alice, members, { wrapFor: [carol] });
		const c = books(alice.token);
		const refused = async (status: number, body: unknown) => expect((await c.list(book.id, body)).status).toBe(status);
		await refused(400, { ...ok.body, list: "?" });
		await refused(400, { ...ok.body, list_sig: "x" });
		await refused(400, { ...ok.body, rev: "2" });
		await refused(400, { ...ok.body, wraps: [] });
		await refused(400, { ...ok.body, history_link: b64(73) });
		const spec = { shareId: book.id, rev: 2, epoch: 1, owner: alice, signer: alice, prev: book.hash, members };
		await refused(400, { ...ok.body, list: (await bookList({ ...spec, shareId: newShareId() })).list });
		// Another owner: ownership doesn't move.
		await refused(400, { ...ok.body, list: (await bookList({ ...spec, owner: bob })).list });
		// A signer who isn't the caller (bob's list sent by alice).
		await refused(403, { ...ok.body, list: (await bookList({ ...spec, signer: bob })).list });
		// A list with no owner entry, or two.
		await refused(400, { ...ok.body, list: (await bookList({ ...spec, members: [{ account: bob, role: "manager" }, { account: carol, role: "view" }] })).list });
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(1);
		expect((await c.list(book.id, ok.body)).status).toBe(200);
	});

	it("applies the rank rules: nobody changes the owner or a stayer's keys; a manager changes nobody at Manager or above", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "manager" },
			{ account: carol, role: "edit" },
		]);
		const stranger = newIdentity();
		const owner = { account: alice, role: "owner" as const };
		const asOwner = (members: Parameters<typeof advance>[2], opts = {}) => advance(book, alice, members, opts);
		// A stayer with other keys or another name.
		const rekeyed = [owner, { account: bob, role: "manager" as const }, { account: carol, role: "edit" as const, claims: { sign_pub: stranger.sign_pub } }];
		expect((await asOwner(rekeyed, { wrapFor: [] })).status).toBe(403);
		// A manager makes a manager, or touches one.
		const bobAsManager = (members: Parameters<typeof advance>[2], opts = {}) => advance(book, bob, members, opts);
		const promote = [owner, { account: bob, role: "manager" as const }, { account: carol, role: "manager" as const }];
		expect(await bobAsManager(promote, { wrapFor: [carol] })).toMatchObject({ status: 403, body: { error: "forbidden_role" } });
		const demoteSelf = [owner, { account: bob, role: "edit" as const }, { account: carol, role: "edit" as const }];
		expect((await bobAsManager(demoteSelf, { newEpoch: true })).status).toBe(403);
		// A manager adds someone below Manager.
		const addDave = [...book.members, { account: dave, role: "comment" as const }];
		expect((await bobAsManager(addDave, { wrapFor: [dave] })).status).toBe(200);
		expect(book.rev).toBe(2);
		// The owner demotes the manager (a new epoch with wraps for all and a history link).
		const demoted = [owner, { account: bob, role: "edit" as const }, { account: carol, role: "edit" as const }, { account: dave, role: "comment" as const }];
		expect((await asOwner(demoted, { newEpoch: true })).status).toBe(200);
		expect((await books(bob.token).get(book.id)).body).toMatchObject({ role: "edit", epoch: 2 });
		// A demoted manager can't sign any more (the head no longer names them).
		expect((await advance(book, bob, demoted, { wrapFor: [] })).status).toBe(403);
	});

	it("lets only the owner re-add a removed account with keys other than its last ones", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "manager" },
			{ account: carol, role: "edit" },
		]);
		const owner = { account: alice, role: "owner" as const };
		const bobM = { account: bob, role: "manager" as const };
		expect((await advance(book, alice, [owner, bobM], { newEpoch: true })).status).toBe(200);
		const fresh = await rekeyIdentity(carol);
		const swapped = { account: carol, role: "edit" as const, claims: { sign_pub: fresh.sign_pub, box_pub: fresh.box_pub } };
		expect((await advance(book, bob, [owner, bobM, swapped], { wrapFor: [carol] })).status).toBe(403);
		expect((await advance(book, alice, [owner, bobM, swapped], { wrapFor: [carol] })).status).toBe(200);
	});

	it("applies the epoch rules: a removal or demotion starts a new epoch with every wrap and a history link", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "edit" },
		]);
		const owner = { account: alice, role: "owner" as const };
		const without = [owner, { account: carol, role: "edit" as const }];
		// Same epoch can't cut anyone; a new epoch needs every wrap and a history link.
		expect((await advance(book, alice, without, { wrapFor: [] })).status).toBe(400);
		expect((await advance(book, alice, without, { newEpoch: true, wrapFor: [carol] })).status).toBe(400);
		const noLink = await nextBody(book, alice, without, { newEpoch: true });
		expect((await books(alice.token).list(book.id, { ...noLink.body, history_link: undefined })).status).toBe(400);
		expect((await books(alice.token).list(book.id, { ...noLink.body, history_link: b64(72) })).status).toBe(400);
		// An epoch that skips one, keeps the commitment, or moves the log position back.
		const skip = await bookList({ shareId: book.id, rev: 2, epoch: 3, owner: alice, signer: alice, prev: book.hash, members: without });
		expect((await books(alice.token).list(book.id, { ...noLink.body, list: skip.list })).status).toBe(400);
		const same = await bookList({ shareId: book.id, rev: 2, epoch: 2, owner: alice, signer: alice, prev: book.hash, members: without, keyCommit: commitOf(1) });
		expect((await books(alice.token).list(book.id, { ...noLink.body, list: same.list })).status).toBe(400);
		const far = await bookList({ shareId: book.id, rev: 2, epoch: 2, owner: alice, signer: alice, prev: book.hash, members: without, fromSeq: 1_000_001 });
		expect((await books(alice.token).list(book.id, { ...noLink.body, list: far.list })).status).toBe(400);
		expect(await count("book_epochs", "share_id = ?1", book.id)).toBe(0);

		expect((await advance(book, alice, without, { newEpoch: true })).status).toBe(200);
		const bobsView = await books(bob.token).get(book.id);
		expect(bobsView.status).toBe(404);
		expect(await count("book_wraps", "share_id = ?1 AND account_id = ?2", book.id, bob.creds.accountId)).toBe(0);
		expect(await count("book_members", "share_id = ?1 AND account_id = ?2", book.id, bob.creds.accountId)).toBe(0);
		const carols = await books(carol.token).get(book.id);
		expect(carols.body.wraps.map((w: { epoch: number }) => w.epoch)).toEqual([1, 2]);
		expect(carols.body.epochs.map((e: { epoch: number }) => e.epoch)).toEqual([2]);
		// A removed member can come back, at the new epoch only.
		const back = [...without, { account: bob, role: "view" as const }];
		expect((await advance(book, alice, back, { wrapFor: [bob] })).status).toBe(200);
		expect((await books(bob.token).get(book.id)).body.wraps.map((w: { epoch: number }) => w.epoch)).toEqual([2]);
	});

	it("refuses a list from anyone but the owner or a manager (403), and from a stranger or an invitee (404)", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "edit" },
			{ account: carol, role: "manager" },
		]);
		await ensureIdentity(dave);
		const members = [...book.members, { account: dave, role: "view" as const }];
		const asBob = await nextBody(book, bob, members, { wrapFor: [dave] });
		expect(await books(bob.token).list(book.id, asBob.body)).toMatchObject({ status: 403, body: { error: "forbidden_role" } });
		const asDave = await nextBody(book, dave, members, { wrapFor: [dave] });
		expect((await books(dave.token).list(book.id, asDave.body)).status).toBe(404);
		expect((await callJson(`/v2/collab/books/${book.id}/epoch`, { json: asDave.body })).status).toBe(401);
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(1);
		// A manager who hasn't accepted yet is not in.
		const fresh = await shareBook(alice, [{ account: bob, role: "manager" }], false);
		const next = await nextBody(fresh, bob, [...fresh.members, { account: dave, role: "view" }], { wrapFor: [dave] });
		expect((await books(bob.token).list(fresh.id, next.body)).status).toBe(404);
		await books(bob.token).accept(fresh.id);
		expect((await books(bob.token).list(fresh.id, next.body)).status).toBe(200);
	});

	it("caps a notebook at 25 people (413), in the Worker and in the database", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "view" }]);
		const owner = { account: alice, role: "owner" as const };
		const fake = (n: number) =>
			Array.from({ length: n }, (_, i) => ({
				account: `a_${"e".repeat(30)}${String(i).padStart(2, "0")}`,
				username: `u${i}x`,
				role: "view" as const,
				sign_pub: b64(32),
				box_pub: b64(32),
			}));
		const withFake = async (n: number) => {
			const spec = await nextBody(book, alice, [owner, { account: bob, role: "view" }], { wrapFor: [] });
			const list = JSON.parse(new TextDecoder().decode(Uint8Array.from(atob(spec.body.list.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0))));
			list.members = [...list.members, ...fake(n)].sort((x: { account: string }, y: { account: string }) => (x.account < y.account ? -1 : 1));
			return { ...spec.body, list: base64url(new TextEncoder().encode(JSON.stringify(list))) };
		};
		expect(await books(alice.token).list(book.id, await withFake(24))).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "members", used: 26, limit: 25 } });
		// At the create route too.
		await ensureIdentity(alice);
		const gen = await genesisBody(newShareId(), alice, []);
		const list = JSON.parse(new TextDecoder().decode(Uint8Array.from(atob(gen.body.list.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0))));
		list.members = [...list.members, ...fake(25)].sort((x: { account: string }, y: { account: string }) => (x.account < y.account ? -1 : 1));
		expect(await books(alice.token).create({ ...gen.body, list: base64url(new TextEncoder().encode(JSON.stringify(list))) })).toMatchObject({
			status: 413,
			body: { quota: "members", used: 26, limit: 25 },
		});
		expect(await count("book_lists", "share_id = ?1", book.id)).toBe(1);

		// The trigger: with 24 rows in a notebook a 25th is stored, a 26th refused, a repeat of one already in is not counted twice.
		const others = [carol, dave];
		for (const account of others) await ensureIdentity(account);
		const insert = (account: string) =>
			env.DB.prepare("INSERT INTO book_members (share_id, account_id, role, status, invited_at) VALUES (?1, ?2, 'view', 'invited', 1) ON CONFLICT (share_id, account_id) DO UPDATE SET role = 'view'").bind(book.id, account).run();
		await setSetting("max_accounts", "100");
		await setSetting("signups_per_day", "100");
		const extra: Account[] = [];
		for (let i = 0; i < 22; i++) extra.push(await signUp(`filler${i}`, { ip: `198.51.100.${i + 10}` }));
		for (const a of extra) {
			await env.DB.prepare("INSERT INTO book_members (share_id, account_id, role, status, invited_at) VALUES (?1, ?2, 'view', 'invited', 1)").bind(book.id, a.creds.accountId).run();
		}
		await env.DB.prepare("INSERT INTO book_members (share_id, account_id, role, status, invited_at) VALUES (?1, ?2, 'view', 'invited', 1)").bind(book.id, carol.creds.accountId).run();
		expect(await count("book_members", "share_id = ?1", book.id)).toBe(25);
		await env.DB.prepare("INSERT INTO book_members (share_id, account_id, role, status, invited_at) VALUES (?1, ?2, 'view', 'invited', 1) ON CONFLICT (share_id, account_id) DO UPDATE SET role = 'view'").bind(book.id, carol.creds.accountId).run();
		await expect(
			env.DB.prepare("INSERT INTO book_members (share_id, account_id, role, status, invited_at) VALUES (?1, ?2, 'view', 'invited', 1)").bind(book.id, dave.creds.accountId).run(),
		).rejects.toThrow(/quota_exceeded:members/);
	});

	it("caps an owner at 50 notebooks (413)", async () => {
		await ensureIdentity(alice);
		await env.DB.batch(
			Array.from({ length: 50 }, () =>
				env.DB.prepare("INSERT INTO book_shares (id, owner_id, epoch, head_rev, head_hash, created_at, updated_at) VALUES (?1, ?2, 1, 1, ?3, 1, 1)").bind(
					newShareId(),
					alice.creds.accountId,
					"a".repeat(64),
				),
			),
		);
		const { body } = await genesisBody(newShareId(), alice, []);
		expect(await books(alice.token).create(body)).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "books", used: 50, limit: 50 } });
	});
});

describe("invite links", () => {
	it("are made by the owner or a manager (201, again 201), refused to others (403), and cancelled (204)", async () => {
		const book = await shareBook(alice, [
			{ account: bob, role: "manager" },
			{ account: carol, role: "edit" },
		]);
		const link = await linkBody();
		expect(await books(alice.token).link(book.id, link)).toMatchObject({ status: 201, body: { link_id: link.link_id, role: "view" } });
		expect((await books(alice.token).link(book.id, link)).status).toBe(201);
		expect(await count("book_links")).toBe(1);
		expect(await books(alice.token).link(book.id, { ...link, role: "edit" })).toMatchObject({ status: 409, body: { error: "link_id_taken" } });
		const byBob = await linkBody();
		expect((await books(bob.token).link(book.id, byBob)).status).toBe(201);
		expect(await books(carol.token).link(book.id, await linkBody())).toMatchObject({ status: 403, body: { error: "forbidden_role" } });
		expect((await books(dave.token).link(book.id, await linkBody())).status).toBe(404);
		expect((await callJson(`/v2/collab/books/${book.id}/links`, { json: link })).status).toBe(401);
		expect((await books(alice.token).get(book.id)).body.links).toHaveLength(2);
		expect((await books(carol.token).unlink(book.id, link.link_id)).status).toBe(403);
		expect((await books(bob.token).unlink(book.id, link.link_id)).status).toBe(204);
		expect((await books(bob.token).unlink(book.id, link.link_id)).status).toBe(404);
	});

	it("refuse bad bodies (400) and an 11th live link (413)", async () => {
		const book = await shareBook(alice, []);
		const c = books(alice.token);
		const link = await linkBody();
		for (const bad of [
			{ ...link, link_id: "x" },
			{ ...link, proof_hash: "ABC" },
			{ ...link, role: "owner" },
			{ ...link, role: "manager" },
			{ ...link, expires_at: 1 },
			{ ...link, expires_at: clock.now + 40 * DAY },
		]) {
			expect((await c.link(book.id, bad)).status).toBe(400);
		}
		for (let i = 0; i < 10; i++) expect((await c.link(book.id, await linkBody())).status).toBe(201);
		expect(await c.link(book.id, await linkBody())).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "links", used: 10, limit: 10 } });
	});
});

describe("stopping", () => {
	it("is the owner's alone: a manager gets 403, a stranger 404; everything goes and the ID is retired", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "manager" }]);
		await books(alice.token).link(book.id, await linkBody());
		await advance(book, alice, [...book.members, { account: carol, role: "view" }], { wrapFor: [carol] });
		await advance(book, alice, [book.members[0], book.members[1]], { newEpoch: true });
		expect(await count("book_epochs")).toBe(1);
		expect((await books(bob.token).stop(book.id)).status).toBe(403);
		expect((await books(dave.token).stop(book.id)).status).toBe(404);
		expect((await callJson(`/v2/collab/books/${book.id}`, { method: "DELETE" })).status).toBe(401);
		expect((await books(alice.token).stop(book.id)).status).toBe(204);
		for (const table of ["book_shares", "book_lists", "book_members", "book_wraps", "book_epochs", "book_links"]) expect(await count(table)).toBe(0);
		expect(await env.DB.prepare("SELECT id FROM collab_retired").first("id")).toBe(book.id);
		expect((await books(alice.token).get(book.id)).status).toBe(404);
		expect((await books(alice.token).stop(book.id)).status).toBe(404);
	});

	it("an owner leaves by stopping (403 on leaving); a member leaves and loses their wraps", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		expect((await books(alice.token).leave(book.id)).status).toBe(403);
		expect((await books(bob.token).leave(book.id)).status).toBe(204);
		expect((await books(bob.token).leave(book.id)).status).toBe(404);
		expect((await books(bob.token).accept(book.id)).status).toBe(404);
		expect(await count("book_wraps", "share_id = ?1", book.id)).toBe(1);
	});

	it("an account deletion stops its notebooks and removes its place in others'", async () => {
		const mine = await shareBook(alice, [{ account: bob, role: "edit" }]);
		const theirs = await shareBook(bob, [{ account: carol, role: "view" }]);
		await advance(theirs, bob, [theirs.members[0], theirs.members[1], { account: alice, role: "view" }], { wrapFor: [alice] });
		const r = await callJson("/v2/account/delete", { token: alice.token, json: { auth_key: alice.creds.authKey, username: alice.creds.username } });
		expect(r.status).toBe(200);
		expect(await count("book_shares", "id = ?1", mine.id)).toBe(0);
		expect(await count("book_lists", "share_id = ?1", mine.id)).toBe(0);
		expect(await count("book_members", "account_id = ?1", alice.creds.accountId)).toBe(0);
		expect(await count("book_wraps", "account_id = ?1", alice.creds.accountId)).toBe(0);
		expect(await env.DB.prepare("SELECT id FROM collab_retired").first("id")).toBe(mine.id);
		expect(await count("book_shares", "id = ?1", theirs.id)).toBe(1);
	});
});

/** Puts a rule's count for the account at `n` in the current window. */
const setCount = (rule: string, account: Account, n: number) =>
	env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, ?3) ON CONFLICT (key) DO UPDATE SET count = excluded.count, window_start = excluded.window_start")
		.bind(`${rule}:${account.creds.accountId}`, Math.floor(clock.now / 3_600_000) * 3_600_000, n)
		.run();

describe("rate limits", () => {
	it("count reads and writes per account: the 601st read and the 121st write are 429", async () => {
		const book = await shareBook(alice, []);
		await setCount("book-read-account", alice, 600);
		expect(await books(alice.token).get(book.id)).toMatchObject({ status: 429, body: { error: "rate_limited", limit: "book-read-account" } });
		expect((await books(bob.token).get(book.id)).status).toBe(404);
		await setCount("book-write-account", alice, 120);
		expect(await books(alice.token).link(book.id, await linkBody())).toMatchObject({ status: 429, body: { limit: "book-write-account" } });
		const { body } = await genesisBody(newShareId(), alice, []);
		expect((await books(alice.token).create(body)).status).toBe(429);
		expect((await books(alice.token).stop(book.id)).status).toBe(429);
		expect(await count("book_shares")).toBe(1);
	});
});

describe("what the server keeps", () => {
	it("only opaque wraps, history links and signed lists: no key material", async () => {
		const book = await shareBook(alice, [{ account: bob, role: "edit" }]);
		await advance(book, alice, [book.members[0]], { newEpoch: true });
		for (const table of ["book_wraps", "book_epochs", "book_lists"]) {
			const rows = await env.DB.prepare(`SELECT * FROM ${table}`).all();
			expect(rows.results.length).toBeGreaterThan(0);
		}
		const columns = await env.DB.prepare("SELECT m.name AS tbl, p.name AS col FROM sqlite_master m, pragma_table_info(m.name) p WHERE m.name LIKE 'book_%' AND m.type = 'table'").all<{ tbl: string; col: string }>();
		expect(columns.results.map((c) => c.col).filter((c) => /key(?!_commit)|secret|plain/i.test(c))).toEqual([]);
	});
});
