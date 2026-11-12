// Properties of shared notes (fast-check, seed pinned in vitest.config.ts):
// the routes against a model; removed members never read again; viewers
// never write and commenters write only comments, over HTTP and WebSocket;
// the server stores exactly the bytes sent and never anything a member
// sealed, nor a key; and the envelope parser, against the shared fixtures and
// random bytes.

import { runDurableObjectAlarm, runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { MAX_ENVELOPE, MIN_ENVELOPE, parseEnvelope } from "../src/envelope";
import fixture from "./fixtures/collab.json";
import { type Account, base64url, fromBase64url, resetServer, signUp } from "./helpers";
import {
	type Listed,
	type Role,
	type Socket,
	b64,
	collab,
	ensureIdentity,
	envelope,
	epochBody,
	header,
	newShareId,
	openSocket,
	shareBody,
	shareWith,
	unframe,
} from "./collab-helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

const roles: Role[] = ["edit", "comment", "view"];
const genRole = fc.constantFrom(...roles);

async function three(): Promise<[Account, Account, Account]> {
	await resetServer();
	const accounts = [await signUp("alice"), await signUp("bob"), await signUp("carol")] as [Account, Account, Account];
	for (const a of accounts) await ensureIdentity(a);
	return accounts;
}

const hexOf = (hex: string): Uint8Array => Uint8Array.from(hex.match(/../g) ?? [], (h) => parseInt(h, 16));

// ------------------------------------------------------------------- (e) parser

describe("the envelope header", () => {
	it("parses every shared fixture case as it says", () => {
		for (const c of fixture.cases) {
			const parsed = parseEnvelope(hexOf(c.hex));
			if (!c.ok) {
				expect(parsed, c.name).toBeNull();
				continue;
			}
			expect(parsed, c.name).toEqual({ kind: c.kind, epoch: c.epoch, author: c.author, device: c.device, counter: BigInt(c.counter!) });
		}
	});

	// Random bytes never crash the parser, and whatever it accepts re-encodes to
	// its own first bytes with room left for a nonce, a tag and a signature.
	it("prop_random_bytes_never_crash_and_accepted_ones_are_well_formed", () => {
		const almost = fc
			.record({
				head: fc.uint8Array({ minLength: 0, maxLength: 120 }),
				prefix: fc.boolean(),
			})
			.map(({ head, prefix }) => {
				if (!prefix || head.length < 2) return head;
				// Often start like an envelope, so the deeper checks are reached.
				const out = head.slice();
				out[0] = 1;
				out[1] = [0x75, 0x73, 0x63, 0x70][head[1] % 4];
				return out;
			});
		fc.assert(
			fc.property(fc.oneof(fc.uint8Array({ maxLength: 400 }), almost), (bytes) => {
				const parsed = parseEnvelope(bytes);
				if (parsed === null) return;
				const head = header(parsed.kind, parsed.epoch, parsed.author, parsed.device, parsed.counter);
				expect(bytes.slice(0, head.length)).toEqual(head);
				expect(bytes.length).toBeGreaterThanOrEqual(MIN_ENVELOPE);
				expect(parsed.epoch).toBeGreaterThan(0);
			}),
		);
	});

	const hex32 = fc.uint8Array({ minLength: 16, maxLength: 16 }).map((b) => [...b].map((x) => x.toString(16).padStart(2, "0")).join(""));
	const genParts = fc.record({
		kind: fc.constantFrom("u", "s", "c", "p") as fc.Arbitrary<"u" | "s" | "c" | "p">,
		epoch: fc.integer({ min: 1, max: 0xffffffff }),
		author: hex32.map((h) => `a_${h}`),
		device: hex32.map((h) => `d_${h}`),
		counter: fc.bigInt({ min: 0n, max: (1n << 64n) - 1n }),
		sealed: fc.integer({ min: 16, max: 200 }),
	});

	// Well-formed envelopes always parse to their parts; cutting any byte off
	// the end of the minimum, or changing the format byte, never parses.
	it("prop_well_formed_envelopes_parse_to_their_parts", () => {
		fc.assert(
			fc.property(genParts, fc.integer({ min: 2, max: 255 }), (p, format) => {
				const head = header(p.kind, p.epoch, p.author, p.device, p.counter);
				const bytes = new Uint8Array(head.length + 24 + p.sealed + 64);
				bytes.set(head);
				expect(parseEnvelope(bytes)).toEqual({ kind: p.kind, epoch: p.epoch, author: p.author, device: p.device, counter: p.counter });
				const short = bytes.slice(0, head.length + 24 + 16 + 63);
				expect(parseEnvelope(short)).toBeNull();
				const other = bytes.slice();
				other[0] = format;
				expect(parseEnvelope(other)).toBeNull();
			}),
		);
	});
});

// ------------------------------------------------------------- (a) the model

interface ModelMember {
	role: Role | "owner";
	status: "invited" | "member";
}

interface Model {
	stopped: boolean;
	epoch: number;
	members: Map<string, ModelMember>;
	/** Left while the list still names them: kept out until a list stops naming them. */
	left: Set<string>;
	head: number;
	/** Stored updates and comments: seq and kind. */
	log: { seq: number; kind: "u" | "c" }[];
	snapshot: { seq: number; base: number };
}

type Op =
	| { op: "epoch"; bob: Role | null; carol: Role | null; staleHead: boolean; again: boolean }
	| { op: "accept"; who: 1 | 2 }
	| { op: "append"; who: 0 | 1 | 2; kind: "u" | "c"; stale: boolean }
	| { op: "snapshot"; who: 0 | 1 | 2; behind: boolean }
	| { op: "leave"; who: 0 | 1 | 2 }
	| { op: "read"; who: 0 | 1 | 2 }
	| { op: "stop" };

const who = fc.constantFrom(0, 1, 2) as fc.Arbitrary<0 | 1 | 2>;
const genOp: fc.Arbitrary<Op> = fc.oneof(
	{ arbitrary: fc.record({
			op: fc.constant("epoch" as const),
			bob: fc.option(genRole, { nil: null }),
			carol: fc.option(genRole, { nil: null }),
			staleHead: fc.integer({ min: 0, max: 3 }).map((n) => n === 0),
			again: fc.boolean(),
		}), weight: 2 },
	{ arbitrary: fc.record({ op: fc.constant("accept" as const), who: fc.constantFrom(1, 2) as fc.Arbitrary<1 | 2> }), weight: 3 },
	{ arbitrary: fc.record({ op: fc.constant("append" as const), who, kind: fc.constantFrom("u", "c") as fc.Arbitrary<"u" | "c">, stale: fc.integer({ min: 0, max: 3 }).map((n) => n === 0) }), weight: 12 },
	{ arbitrary: fc.record({ op: fc.constant("snapshot" as const), who, behind: fc.boolean() }), weight: 4 },
	{ arbitrary: fc.record({ op: fc.constant("leave" as const), who }), weight: 1 },
	{ arbitrary: fc.record({ op: fc.constant("read" as const), who }), weight: 3 },
	{ arbitrary: fc.constant({ op: "stop" as const }), weight: 1 },
);

const mayWrite = (role: string, kind: "u" | "c" | "s") => (kind === "c" ? role !== "view" : role === "edit" || role === "owner");

describe("the routes follow the model", () => {
	// Model-based: a share's members (role, status), epoch, head, log and
	// snapshot. Every answer's status matches the model's, reads return exactly
	// the model's log, and every account's share list matches it.
	it("prop_collab_routes_match_the_model", async () => {
		await fc.assert(
			fc.asyncProperty(genRole, fc.option(genRole, { nil: null }), fc.boolean(), fc.array(genOp, { minLength: 8, maxLength: 40, size: "medium" }), async (bobRole, carolRole, accepted, ops) => {
				const people = await three();
				const [alice] = people;
				const ids = people.map((p) => p.creds.accountId);
				const initial: Listed[] = [{ account: people[1], role: bobRole }];
				if (carolRole) initial.push({ account: people[2], role: carolRole });
				const id = await shareWith(alice, initial, accepted);
				const m: Model = {
					stopped: false,
					epoch: 1,
					members: new Map<string, ModelMember>([
						[ids[0], { role: "owner", status: "member" }],
						[ids[1], { role: bobRole, status: accepted ? "member" : "invited" }],
					]),
					left: new Set<string>(),
					head: 0,
					log: [],
					snapshot: { seq: 0, base: 0 },
				};
				if (carolRole) m.members.set(ids[2], { role: carolRole, status: accepted ? "member" : "invited" });

				/** 404 for a non-member and an invitee alike, else null. */
				const gate = (i: number): number | null => {
					const member = m.members.get(ids[i]);
					return m.stopped || !member || member.status !== "member" ? 404 : null;
				};

				for (const op of ops) {
					const label = JSON.stringify(op);
					switch (op.op) {
						case "epoch": {
							const listed: Listed[] = [];
							if (op.bob) listed.push({ account: people[1], role: op.bob });
							if (op.carol) listed.push({ account: people[2], role: op.carol });
							// from_seq must be the head; one past it is refused, and the owner signs again.
							const body = epochBody(id, m.epoch + 1, alice, listed, op.staleHead ? m.head + 1 : m.head);
							const r = await collab(alice.token).epoch(id, body);
							if (m.stopped) {
								expect(r.status, label).toBe(404);
								break;
							}
							if (op.staleHead) {
								expect(r.body, label).toMatchObject({ error: "head_moved", head: m.head });
								break;
							}
							expect(r.status, label).toBe(200);
							// The identical request again (a lost answer) still succeeds.
							if (op.again) expect((await collab(alice.token).epoch(id, body)).status, label).toBe(200);
							m.epoch += 1;
							const next = new Map<string, ModelMember>([[ids[0], { role: "owner", status: "member" }]]);
							for (const l of listed) {
								if (m.left.has(l.account.creds.accountId)) continue;
								const old = m.members.get(l.account.creds.accountId);
								next.set(l.account.creds.accountId, { role: l.role, status: old?.status ?? "invited" });
							}
							m.members = next;
							m.left = new Set([...m.left].filter((a) => listed.some((l) => l.account.creds.accountId === a)));
							break;
						}
						case "accept": {
							const r = await collab(people[op.who].token).accept(id);
							const member = m.members.get(ids[op.who]);
							if (m.stopped || !member) expect(r.status, label).toBe(404);
							else {
								expect(r.status, label).toBe(204);
								member.status = "member";
							}
							break;
						}
						case "append": {
							const stale = op.stale && m.epoch > 1;
							const bytes = await envelope({ kind: op.kind, epoch: stale ? m.epoch - 1 : m.epoch, shareId: id, by: people[op.who] });
							const r = await collab(people[op.who].token).append(id, bytes);
							const gated = gate(op.who);
							if (gated) expect(r.status, label).toBe(gated);
							else if (stale) expect(r.body, label).toMatchObject({ error: "epoch_changed", epoch: m.epoch });
							else if (!mayWrite(m.members.get(ids[op.who])!.role, op.kind)) expect(r.body.error, label).toBe("forbidden_role");
							else {
								m.head += 1;
								expect(r, label).toEqual({ status: 200, body: { seq: m.head } });
								m.log.push({ seq: m.head, kind: op.kind });
							}
							break;
						}
						case "snapshot": {
							const base = op.behind && m.head > 0 ? m.head - 1 : m.head;
							const bytes = await envelope({ kind: "s", epoch: m.epoch, shareId: id, by: people[op.who] });
							const r = await collab(people[op.who].token).snapshot(id, base, bytes);
							const gated = gate(op.who);
							if (gated) expect(r.status, label).toBe(gated);
							else if (!mayWrite(m.members.get(ids[op.who])!.role, "s")) expect(r.body.error, label).toBe("forbidden_role");
							else if (base !== m.head) expect(r.body, label).toMatchObject({ error: "head_moved", head: m.head });
							else {
								// A snapshot takes its base as its seq; the head stays.
								expect(r, label).toEqual({ status: 200, body: { seq: base } });
								m.snapshot = { seq: base, base };
								m.log = m.log.filter((e) => e.kind === "c" || e.seq > base);
							}
							break;
						}
						case "leave": {
							const r = await collab(people[op.who].token).leave(id);
							const member = m.members.get(ids[op.who]);
							if (m.stopped || !member) expect(r.status, label).toBe(404);
							else if (op.who === 0) expect(r.status, label).toBe(403);
							else {
								expect(r.status, label).toBe(204);
								m.members.delete(ids[op.who]);
								m.left.add(ids[op.who]);
							}
							break;
						}
						case "read": {
							const r = await collab(people[op.who].token).log(id, 0, 1000);
							const gated = gate(op.who);
							if (gated) expect(r.status, label).toBe(gated);
							else {
								expect(r.status, label).toBe(200);
								expect(r.body.head, label).toBe(m.head);
								expect(r.body.snapshot, label).toMatchObject({ seq: m.snapshot.seq, base_seq: m.snapshot.base });
								expect(r.body.entries.map((e: { seq: number }) => e.seq), label).toEqual(m.log.map((e) => e.seq));
								expect(r.body.more, label).toBe(false);
							}
							break;
						}
						case "stop": {
							const r = await collab(alice.token).stop(id);
							expect(r.status, label).toBe(m.stopped ? 404 : 204);
							m.stopped = true;
							break;
						}
					}
					for (const [i, p] of people.entries()) {
						const listed = (await collab(p.token).shares()).body.shares;
						const member = m.members.get(ids[i]);
						if (m.stopped || !member) expect(listed, `${label}: ${p.creds.username}'s shares`).toEqual([]);
						else {
							expect(listed, `${label}: ${p.creds.username}'s shares`).toHaveLength(1);
							expect(listed[0], label).toMatchObject({ share_id: id, role: member.role, status: member.status, epoch: m.epoch, head: m.head });
						}
					}
				}
			}),
			runs(25),
		);
	});
});

// --------------------------------------------------------- (b) removed members

describe("removal", () => {
	// Whatever the sequence of lists, an account that isn't listed reads no log
	// and holds no wrap or list, and a listed one holds wraps for exactly the
	// epochs of its current stretch of membership (never one from before a
	// removal), and is given exactly those epochs' signed lists, as sent.
	it("prop_removed_members_never_read_or_get_later_wraps", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.array(fc.record({ bob: fc.option(genRole, { nil: null }), carol: fc.option(genRole, { nil: null }) }), { minLength: 1, maxLength: 8 }),
				async (lists) => {
					const [alice, bob, carol] = await three();
					const id = newShareId();
					const body = await shareBody(id, alice, [
						{ account: bob, role: "edit" },
						{ account: carol, role: "view" },
					]);
					expect((await collab(alice.token).share(body)).status).toBe(201);
					for (const a of [bob, carol]) await collab(a.token).accept(id);
					const sent = new Map<number, { epoch: number; list: string; list_sig: string }>([[1, { epoch: 1, list: body.list, list_sig: body.list_sig }]]);
					const held = new Map<Account, number[]>([
						[bob, [1]],
						[carol, [1]],
					]);
					let epoch = 1;
					for (const list of lists) {
						epoch += 1;
						const listed: Listed[] = [];
						if (list.bob) listed.push({ account: bob, role: list.bob });
						if (list.carol) listed.push({ account: carol, role: list.carol });
						const next = epochBody(id, epoch, alice, listed);
						expect((await collab(alice.token).epoch(id, next)).status).toBe(200);
						sent.set(epoch, { epoch, list: next.list, list_sig: next.list_sig });
						for (const a of [bob, carol]) {
							const inList = listed.some((l) => l.account === a);
							held.set(a, inList ? [...(held.get(a) ?? []), epoch] : []);
							if (inList) await collab(a.token).accept(id);
							const log = await collab(a.token).log(id);
							const shares = (await collab(a.token).shares()).body.shares;
							const wraps = await env.DB.prepare("SELECT epoch FROM collab_wraps WHERE share_id = ?1 AND account_id = ?2 ORDER BY epoch")
								.bind(id, a.creds.accountId)
								.all<{ epoch: number }>();
							expect(wraps.results.map((w) => w.epoch)).toEqual(held.get(a));
							if (inList) {
								expect(log.status).toBe(200);
								expect(shares[0].wraps.map((w: { epoch: number }) => w.epoch)).toEqual(held.get(a));
								expect(shares[0].lists).toEqual(held.get(a)!.map((e) => sent.get(e)));
							} else {
								expect(log.status).toBe(404);
								expect(shares).toEqual([]);
								const ws = await openSocket(a.token, id);
								expect(ws instanceof Response && ws.status).toBe(404);
							}
						}
					}
				},
			),
			runs(20),
		);
	});
});

// ------------------------------------------------------------ forged lists

describe("forged list entries", () => {
	// A list is refused with 409 member_keys_mismatch naming the account exactly
	// when some entry claims a sign key, box key or username that isn't that
	// account's own, on sharing and on a new epoch alike; nothing is stored then.
	it("prop_lists_must_carry_each_members_own_keys_and_name", async () => {
		const claim = fc.option(
			fc.oneof(
				fc.constant("sign_pub" as const),
				fc.constant("box_pub" as const),
				fc.constant("username" as const),
				fc.constant("other" as const),
			),
			{ nil: null },
		);
		await fc.assert(
			fc.asyncProperty(claim, claim, fc.boolean(), async (bobClaim, carolClaim, onEpoch) => {
				const [alice, bob, carol] = await three();
				const ids = new Map([
					[bob, await ensureIdentity(bob)],
					[carol, await ensureIdentity(carol)],
				]);
				const forge = (a: Account, other: Account, c: typeof bobClaim): Listed["claims"] => {
					if (c === null) return undefined;
					if (c === "sign_pub") return { sign_pub: b64(32) };
					if (c === "box_pub") return { box_pub: b64(32) };
					if (c === "username") return { username: other.creds.username };
					// Someone else's keys under this account.
					return { sign_pub: ids.get(other)!.sign_pub, box_pub: ids.get(other)!.box_pub };
				};
				const listed: Listed[] = [
					{ account: bob, role: "edit", claims: forge(bob, carol, bobClaim) },
					{ account: carol, role: "view", claims: forge(carol, bob, carolClaim) },
				];
				// Entries are checked in account order; the first forged one is named.
				const sorted = [...listed].sort((x, y) => (x.account.creds.accountId < y.account.creds.accountId ? -1 : 1));
				const first = sorted.find((l) => l.claims !== undefined)?.account.creds.accountId;
				const id = newShareId();
				let r;
				if (onEpoch) {
					expect((await collab(alice.token).share(await shareBody(id, alice, []))).status).toBe(201);
					r = await collab(alice.token).epoch(id, epochBody(id, 2, alice, listed));
				} else {
					r = await collab(alice.token).share(await shareBody(id, alice, listed));
				}
				if (first === undefined) {
					expect(r.status).toBe(onEpoch ? 200 : 201);
				} else {
					expect(r).toMatchObject({ status: 409, body: { error: "member_keys_mismatch", account: first } });
					expect(await env.DB.prepare("SELECT count(*) AS n FROM collab_members WHERE account_id != ?1").bind(alice.creds.accountId).first("n")).toBe(0);
					expect(await env.DB.prepare("SELECT max(epoch) AS e FROM collab_lists").first("e")).toBe(onEpoch ? 1 : null);
				}
			}),
			runs(40),
		);
	});
});

// ------------------------------------------------------------ storage quota

describe("the owner's quota", () => {
	// Every append, over HTTP or the socket, is stored exactly when the owner's
	// documents plus the room (snapshot and log) plus the new envelope stay
	// within quota_doc_bytes; refused ones answer quota_exceeded doc_bytes and
	// store nothing; and the room's size reaches collab_shares.bytes.
	it("prop_appends_stay_within_the_owners_document_quota", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.integer({ min: 0, max: 3000 }),
				fc.array(
					fc.record({
						kind: fc.constantFrom("u", "c") as fc.Arbitrary<"u" | "c">,
						size: fc.integer({ min: 0, max: 600 }),
						live: fc.boolean(),
						// Sized to fill the quota exactly (or one byte over), so the boundary is tried.
						fill: fc.constantFrom(null, null, 0, 1),
					}),
					{ minLength: 1, maxLength: 12 },
				),
				async (room, writes) => {
					const [alice] = await three();
					const id = newShareId();
					const body = await shareBody(id, alice, []);
					const snapshotBytes = fromBase64url(body.snapshot)!.length;
					const docBytes = 5000;
					const quota = docBytes + snapshotBytes + room;
					await env.DB.prepare("UPDATE accounts SET doc_bytes = ?2, quota_doc_bytes = ?3 WHERE id = ?1").bind(alice.creds.accountId, docBytes, quota).run();
					expect((await collab(alice.token).share(body)).status).toBe(201);
					const s = await openSocket(alice.token, id);
					if (s instanceof Response) throw new Error(`live ${s.status}`);
					await s.next();
					let stored = snapshotBytes;
					let head = 0;
					for (const w of writes) {
						const left = quota - docBytes - stored - 190 + (w.fill ?? 0);
						const size = w.fill !== null && left >= 0 && left <= MAX_ENVELOPE.c - 190 ? left : w.size;
						const bytes = await envelope({ kind: w.kind, epoch: 1, shareId: id, by: alice, size });
						const fits = docBytes + stored + bytes.length <= quota;
						if (w.live) {
							s.ws.send(bytes);
							const m = await s.next();
							if (fits) expect("bytes" in m && unframe(m.bytes).seq).toBe(head + 1);
							else expect("text" in m && m.text).toMatchObject({ type: "error", error: "quota_exceeded", quota: "doc_bytes", limit: quota });
						} else {
							const r = await collab(alice.token).append(id, bytes);
							if (fits) expect(r.body).toEqual({ seq: head + 1 });
							else expect(r).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "doc_bytes", used: docBytes + stored, limit: quota } });
							if (fits) await s.next();
						}
						if (fits) {
							stored += bytes.length;
							head += 1;
						}
					}
					expect((await collab(alice.token).log(id)).body.head).toBe(head);
					await runDurableObjectAlarm(env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id)));
					expect(await env.DB.prepare("SELECT bytes FROM collab_shares WHERE id = ?1").bind(id).first("bytes")).toBe(stored);
					s.ws.close();
				},
			),
			runs(25),
		);
	});
});

// ------------------------------------------------------------ (c) roles, both ways

describe("viewers and commenters", () => {
	// Over HTTP and the socket alike: a viewer's updates and comments are always
	// refused, a commenter's updates refused and comments stored.
	it("prop_viewers_never_write_and_commenters_only_comment", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.array(
					fc.record({
						who: fc.constantFrom("view", "comment") as fc.Arbitrary<"view" | "comment">,
						kind: fc.constantFrom("u", "c") as fc.Arbitrary<"u" | "c">,
						live: fc.boolean(),
					}),
					{ minLength: 1, maxLength: 10 },
				),
				async (writes) => {
					const [alice, viewer, commenter] = await three();
					const id = await shareWith(alice, [
						{ account: viewer, role: "view" },
						{ account: commenter, role: "comment" },
					]);
					const sockets = new Map<string, Socket>();
					for (const [role, a] of [
						["view", viewer],
						["comment", commenter],
					] as const) {
						const s = await openSocket(a.token, id);
						if (s instanceof Response) throw new Error(`live ${s.status}`);
						await s.next();
						sockets.set(role, s);
					}
					let head = 0;
					for (const w of writes) {
						const by = w.who === "view" ? viewer : commenter;
						const bytes = await envelope({ kind: w.kind, epoch: 1, shareId: id, by });
						const allowed = w.who === "comment" && w.kind === "c";
						if (w.live) {
							const s = sockets.get(w.who)!;
							s.ws.send(bytes);
							for (;;) {
								const m = await s.next();
								if ("text" in m) {
									expect(allowed).toBe(false);
									expect(m.text).toMatchObject({ type: "error", error: "forbidden_role" });
									break;
								}
								const f = unframe(m.bytes);
								if (f.seq === 0) continue;
								if (f.envelope.length === bytes.length && f.envelope.every((b, i) => b === bytes[i])) {
									expect(allowed).toBe(true);
									head += 1;
									expect(f.seq).toBe(head);
									break;
								}
							}
						} else {
							const r = await collab(by.token).append(id, bytes);
							if (allowed) {
								head += 1;
								expect(r).toEqual({ status: 200, body: { seq: head } });
							} else {
								expect(r).toMatchObject({ status: 403, body: { error: "forbidden_role" } });
							}
						}
					}
					const log = (await collab(alice.token).log(id)).body;
					expect(log.head).toBe(head);
					expect(log.entries.length).toBe(head);
					for (const s of sockets.values()) s.ws.close();
				},
			),
			runs(25),
		);
	});
});

// --------------------------------------------------------- (d) no plaintext, no key

/** Every value the server holds about sharing: D1's collab tables and identities, and the room's storage, as bytes. */
async function everythingStored(id: string): Promise<Uint8Array[]> {
	const values: Uint8Array[] = [];
	const encoder = new TextEncoder();
	for (const table of ["identities", "collab_shares", "collab_members", "collab_wraps", "collab_links", "collab_requests", "collab_notifications", "limits"]) {
		const { results } = await env.DB.prepare(`SELECT * FROM ${table}`).all<Record<string, unknown>>();
		for (const row of results) for (const v of Object.values(row)) values.push(encoder.encode(String(v)));
	}
	const stub = env.NOTE_ROOM.get(env.NOTE_ROOM.idFromName(id));
	await runInDurableObject(stub, (_room, state) => {
		for (const table of ["log", "snapshot", "meta"]) {
			for (const row of state.storage.sql.exec(`SELECT * FROM ${table}`).toArray()) {
				for (const v of Object.values(row)) values.push(v instanceof ArrayBuffer ? new Uint8Array(v) : encoder.encode(String(v)));
			}
		}
	});
	return values;
}

function contains(haystack: Uint8Array, needle: Uint8Array): boolean {
	outer: for (let i = 0; i + needle.length <= haystack.length; i++) {
		for (let j = 0; j < needle.length; j++) if (haystack[i + j] !== needle[j]) continue outer;
		return true;
	}
	return false;
}

describe("what the server holds", () => {
	// A member seals text with a marker only the test knows, under a note key
	// only the test holds. The room stores exactly the bytes sent, and no value
	// D1 or the room keeps contains the marker or the key, raw or as base64url.
	it("prop_the_server_stores_exactly_the_sent_bytes_and_never_plaintext_or_keys", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.uint8Array({ minLength: 12, maxLength: 12 }),
				fc.array(fc.record({ kind: fc.constantFrom("u", "c") as fc.Arbitrary<"u" | "c">, pad: fc.integer({ min: 0, max: 300 }) }), {
					minLength: 1,
					maxLength: 8,
				}),
				async (markerBytes, writes) => {
					const [alice, bob] = await three();
					const marker = `secret-${base64url(markerBytes)}`;
					const rawKey = crypto.getRandomValues(new Uint8Array(32));
					const key = await crypto.subtle.importKey("raw", rawKey, { name: "AES-GCM" }, false, ["encrypt"]);
					const id = newShareId();
					const seal = async (kind: "u" | "c" | "s", by: Account, text: string): Promise<Uint8Array> => {
						const head = header(kind, 1, by.creds.accountId, by.device.id, BigInt(Math.floor(Math.random() * 1e9)));
						const nonce = crypto.getRandomValues(new Uint8Array(24));
						const ct = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce.slice(0, 12), additionalData: head }, key, new TextEncoder().encode(text)));
						const out = new Uint8Array(head.length + 24 + ct.length + 64);
						out.set(head);
						out.set(nonce, head.length);
						out.set(ct, head.length + 24);
						out.set(crypto.getRandomValues(new Uint8Array(64)), head.length + 24 + ct.length);
						return out;
					};
					const snapshot = await seal("s", alice, `${marker} the whole note`);
					expect((await collab(alice.token).share(await shareBody(id, alice, [{ account: bob, role: "edit" }], snapshot))).status).toBe(201);
					await collab(bob.token).accept(id);
					await collab(alice.token).notify({ share_id: id, recipient: bob.creds.accountId, sealed: b64(120) });
					const sent: Uint8Array[] = [];
					for (const w of writes) {
						const bytes = await seal(w.kind, bob, `${marker} ${"x".repeat(w.pad)}`);
						expect((await collab(bob.token).append(id, bytes)).status).toBe(200);
						sent.push(bytes);
					}
					const log = (await collab(bob.token).log(id)).body;
					expect(fromBase64url(log.snapshot.envelope)).toEqual(snapshot);
					expect(log.entries.map((e: { envelope: string }) => fromBase64url(e.envelope))).toEqual(sent);
					const stored = await everythingStored(id);
					const needles = [
						new TextEncoder().encode(marker),
						new TextEncoder().encode(base64url(new TextEncoder().encode(marker)).slice(0, 16)),
						rawKey,
						new TextEncoder().encode(base64url(rawKey)),
					];
					for (const value of stored) for (const needle of needles) expect(contains(value, needle)).toBe(false);
				},
			),
			runs(20),
		);
	});
});

