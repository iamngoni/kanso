// Device approval on the Worker (design: Kanso accounts spec, revision 3 ›
// Unlocking a new device): the request/challenge/name/approve sequence, the
// commitment check, expiry, single use, cancellation, self-approval, the
// three-request cap, account isolation, the zero-device count, the limits,
// and that the Worker's answers and the room's frames carry nothing the
// approval protocol keeps from it. Commitments are computed with node's own
// SHA-256, not the Worker's.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { approvalCommit, approvalStatus } from "../src/approvals";
import { HOUR } from "../src/protocol";
import {
	type Signed,
	another,
	approvalsAs,
	commitOf,
	ephemeral,
	first,
	listening,
	random32,
	sealedName,
	wrappedKey,
} from "./approval-helpers";
import { type Account, base64url, clock, resetServer, signUp } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const FIVE_MINUTES = 5 * 60_000;

let alice: Account;
let a: Signed;
let n: Signed;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	a = first(alice);
	n = await another(alice);
});

/** N asks, A challenges, N names itself: the request ready to approve. */
async function named(): Promise<{ id: string; nKey: ReturnType<typeof ephemeral>; aKey: ReturnType<typeof ephemeral>; name: string }> {
	const nKey = ephemeral();
	const aKey = ephemeral();
	const created = await approvalsAs(n).create(nKey.commit);
	expect(created.status).toBe(200);
	const id = created.body.id;
	expect((await approvalsAs(a).challenge(id, aKey.text)).status).toBe(200);
	const name = sealedName();
	expect((await approvalsAs(n).name(id, nKey.text, name)).status).toBe(200);
	return { id, nKey, aKey, name };
}

describe("the sequence", () => {
	it("request, list, challenge, poll, name, approve, poll, delete", async () => {
		const nKey = ephemeral();
		const aKey = ephemeral();
		const created = await approvalsAs(n).create(nKey.commit);
		expect(created.status).toBe(200);
		expect(Object.keys(created.body).sort()).toEqual(["expires_at", "id", "other_devices"]);
		expect(created.body.id).toMatch(/^[A-Za-z0-9_-]{43}$/);
		expect(created.body.expires_at).toBe(clock.now + FIVE_MINUTES);
		expect(created.body.other_devices).toBe(1);
		const id = created.body.id;

		expect((await approvalsAs(n).poll(id)).body).toEqual({ status: "waiting", expires_at: clock.now + FIVE_MINUTES });
		const listed = await approvalsAs(a).list();
		expect(listed.body.approvals).toEqual([
			{
				id,
				device_id: n.device.id,
				commit: nKey.commit,
				epk: null,
				created_at: clock.now,
				expires_at: clock.now + FIVE_MINUTES,
				approver_epk: null,
				approver_device_id: null,
				sealed_name: null,
				status: "waiting",
			},
		]);

		expect((await approvalsAs(a).challenge(id, aKey.text)).body).toEqual({ ok: true });
		expect((await approvalsAs(n).poll(id)).body).toEqual({
			status: "challenged",
			expires_at: clock.now + FIVE_MINUTES,
			approver_epk: aKey.text,
			approver_device_id: a.device.id,
		});

		const name = sealedName(80);
		expect((await approvalsAs(n).name(id, nKey.text, name)).body).toEqual({ ok: true });
		const row = (await approvalsAs(a).list()).body.approvals[0];
		expect(row).toMatchObject({ epk: nKey.text, sealed_name: name, approver_epk: aKey.text, status: "challenged" });

		const wrapped = wrappedKey();
		expect((await approvalsAs(a).approve(id, wrapped)).body).toEqual({ ok: true });
		expect((await approvalsAs(n).poll(id)).body).toEqual({
			status: "approved",
			expires_at: clock.now + FIVE_MINUTES,
			approver_epk: aKey.text,
			approver_device_id: a.device.id,
			sealed_name: name,
			wrapped_key: wrapped,
		});
		// Answered: no longer pending.
		expect((await approvalsAs(a).list()).body.approvals).toEqual([]);
		// A lost answer is read again.
		expect((await approvalsAs(n).poll(id)).body.wrapped_key).toBe(wrapped);
		expect((await approvalsAs(n).cancel(id)).status).toBe(204);
		expect((await approvalsAs(n).poll(id)).status).toBe(404);
	});

	it("the Worker's commitment is node's SHA-256 of the label, 0x01 and the key", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 32, maxLength: 32 }), async (epk) => {
				expect(await approvalCommit(epk)).toBe(commitOf(epk));
			}),
			runs(100),
		);
	});

	it("each step waits for the one before: name before a challenge, approve before a name", async () => {
		const nKey = ephemeral();
		const id = (await approvalsAs(n).create(nKey.commit)).body.id;
		expect((await approvalsAs(n).name(id, nKey.text, sealedName())).body.error).toBe("bad_request");
		expect((await approvalsAs(a).challenge(id, ephemeral().text)).status).toBe(200);
		expect((await approvalsAs(a).approve(id, wrappedKey())).body.error).toBe("bad_request");
	});

	it("refuses malformed bodies and IDs", async () => {
		const api = approvalsAs(n);
		expect((await api.create("short")).body.error).toBe("bad_request");
		expect((await api.create(`${ephemeral().commit.slice(0, 42)}B`)).status).toBe(400);
		expect((await api.poll("nope")).body.error).toBe("bad_request");
		const id = (await api.create(ephemeral().commit)).body.id;
		expect((await approvalsAs(a).challenge(id, "x".repeat(43))).status).toBe(400);
		expect((await approvalsAs(a).approve(id, "x".repeat(97))).status).toBe(400);
	});
});

describe("the commitment", () => {
	it("a revealed key that doesn't match the commitment is refused, and the request stays open for the right one", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 32, maxLength: 32 }), fc.uint8Array({ minLength: 32, maxLength: 32 }), async (committed, revealed) => {
				await resetServer();
				alice = await signUp("alice");
				a = first(alice);
				n = await another(alice);
				const id = (await approvalsAs(n).create(commitOf(committed))).body.id;
				await approvalsAs(a).challenge(id, base64url(random32()));
				const name = sealedName();
				const r = await approvalsAs(n).name(id, base64url(revealed), name);
				const same = base64url(committed) === base64url(revealed);
				expect(r.status).toBe(same ? 200 : 400);
				expect((await approvalsAs(a).list()).body.approvals[0].epk).toBe(same ? base64url(revealed) : null);
				expect((await approvalsAs(n).name(id, base64url(committed), name)).status).toBe(200);
			}),
			runs(25),
		);
	});
});

describe("single use", () => {
	it("a second device's challenge, a second key, another name and another wrap are 409; the same again is 200", async () => {
		const third = await another(alice);
		const { id, nKey, aKey, name } = await named();
		expect((await approvalsAs(a).challenge(id, aKey.text)).status).toBe(200);
		expect((await approvalsAs(a).challenge(id, ephemeral().text)).body.error).toBe("approval_used");
		expect((await approvalsAs(third).challenge(id, aKey.text)).body.error).toBe("approval_used");
		expect((await approvalsAs(n).name(id, nKey.text, name)).status).toBe(200);
		expect((await approvalsAs(n).name(id, nKey.text, sealedName())).body.error).toBe("approval_used");
		// The approver must be the device that challenged.
		expect((await approvalsAs(third).approve(id, wrappedKey())).body.error).toBe("approval_used");
		const wrapped = wrappedKey();
		expect((await approvalsAs(a).approve(id, wrapped)).status).toBe(200);
		expect((await approvalsAs(a).approve(id, wrapped)).status).toBe(200);
		const second = await approvalsAs(a).approve(id, wrappedKey());
		expect(second.status).toBe(409);
		expect(second.body.error).toBe("approval_used");
		expect((await approvalsAs(n).poll(id)).body.wrapped_key).toBe(wrapped);
	});

	it("property: of any sequence of approvals by any devices, only the challenger's first wrap is stored", async () => {
		await fc.assert(
			fc.asyncProperty(fc.array(fc.tuple(fc.integer({ min: 0, max: 2 }), fc.integer({ min: 0, max: 2 })), { minLength: 1, maxLength: 6 }), async (tries) => {
				await resetServer();
				alice = await signUp("alice");
				a = first(alice);
				n = await another(alice);
				const third = await another(alice);
				const devices = [a, n, third];
				const wraps = [wrappedKey(), wrappedKey(), wrappedKey()];
				const { id } = await named();
				let stored: string | null = null;
				for (const [who, which] of tries) {
					const r = await approvalsAs(devices[who]).approve(id, wraps[which]);
					if (who === 1) expect(r.body.error).toBe("self_approval");
					else if (who === 2) expect(r.body.error).toBe("approval_used");
					else if (stored === null || stored === wraps[which]) {
						expect(r.status).toBe(200);
						stored = wraps[which];
					} else expect(r.body.error).toBe("approval_used");
				}
				const poll = (await approvalsAs(n).poll(id)).body;
				expect(poll.status).toBe(stored === null ? "challenged" : "approved");
				expect(poll.wrapped_key).toBe(stored ?? undefined);
			}),
			runs(20),
		);
	});
});

describe("refusals", () => {
	it("a device can't challenge or approve its own request", async () => {
		const nKey = ephemeral();
		const id = (await approvalsAs(n).create(nKey.commit)).body.id;
		const own = await approvalsAs(n).challenge(id, ephemeral().text);
		expect(own.status).toBe(403);
		expect(own.body.error).toBe("self_approval");
		await approvalsAs(a).challenge(id, ephemeral().text);
		await approvalsAs(n).name(id, nKey.text, sealedName());
		const approve = await approvalsAs(n).approve(id, wrappedKey());
		expect(approve.status).toBe(403);
		expect(approve.body.error).toBe("self_approval");
	});

	it("only the requester names or polls its request", async () => {
		const nKey = ephemeral();
		const id = (await approvalsAs(n).create(nKey.commit)).body.id;
		await approvalsAs(a).challenge(id, ephemeral().text);
		expect((await approvalsAs(a).name(id, nKey.text, sealedName())).status).toBe(404);
		expect((await approvalsAs(a).poll(id)).status).toBe(404);
	});

	it("another account's request is 404 everywhere, and its list is its own", async () => {
		const bob = await signUp("bob");
		const b = first(bob);
		const nKey = ephemeral();
		const id = (await approvalsAs(n).create(nKey.commit)).body.id;
		expect((await approvalsAs(b).list()).body.approvals).toEqual([]);
		expect((await approvalsAs(b).poll(id)).status).toBe(404);
		expect((await approvalsAs(b).challenge(id, ephemeral().text)).status).toBe(404);
		expect((await approvalsAs(b).approve(id, wrappedKey())).status).toBe(404);
		expect((await approvalsAs(b).cancel(id)).status).toBe(404);
		expect((await approvalsAs(n).poll(id)).body.status).toBe("waiting");
	});

	it("the list never shows the caller's own requests", async () => {
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		expect((await approvalsAs(n).list()).body.approvals).toEqual([]);
		expect((await approvalsAs(a).list()).body.approvals.map((r: { id: string }) => r.id)).toEqual([id]);
	});
});

describe("expiry", () => {
	it("five minutes: the poll says expired, answers are 409 approval_expired, and the list drops it", async () => {
		const { id, nKey } = await named();
		const later = { now: clock.now + FIVE_MINUTES };
		expect((await approvalsAs(n, later).poll(id)).body).toEqual({ status: "expired", expires_at: clock.now + FIVE_MINUTES });
		expect((await approvalsAs(a, later).list()).body.approvals).toEqual([]);
		const r = await approvalsAs(a, later).approve(id, wrappedKey());
		expect(r.status).toBe(409);
		expect(r.body.error).toBe("approval_expired");
		expect((await approvalsAs(a, later).challenge(id, ephemeral().text)).body.error).toBe("approval_expired");
		expect((await approvalsAs(n, later).name(id, nKey.text, sealedName())).body.error).toBe("approval_expired");
		// Just before, it was still answerable.
		expect((await approvalsAs(a, { now: clock.now + FIVE_MINUTES - 1 }).approve(id, wrappedKey())).status).toBe(200);
	});

	it("an approved request can't be read after it expires", async () => {
		const { id } = await named();
		await approvalsAs(a).approve(id, wrappedKey());
		const poll = (await approvalsAs(n, { now: clock.now + FIVE_MINUTES }).poll(id)).body;
		expect(poll).toEqual({ status: "expired", expires_at: clock.now + FIVE_MINUTES });
	});

	it("property: the status is a function of the row and the time, in the spec's order", () => {
		const row = fc.record({
			cancelled: fc.boolean(),
			completed: fc.boolean(),
			challenged: fc.boolean(),
			offset: fc.integer({ min: -FIVE_MINUTES, max: 2 * FIVE_MINUTES }),
		});
		fc.assert(
			fc.property(row, ({ cancelled, completed, challenged, offset }) => {
				const now = 1_000_000;
				const status = approvalStatus(
					{
						id: "x",
						account_id: "a",
						device_id: "d",
						commit_hash: "c",
						epk: null,
						approver_device_id: null,
						approver_epk: challenged || completed ? "k" : null,
						sealed_name: null,
						wrapped_key: null,
						created_at: 0,
						expires_at: now + offset,
						completed_at: completed && !cancelled ? 1 : null,
						cancelled_at: cancelled ? 1 : null,
					},
					now,
				);
				const expected = cancelled ? "cancelled" : offset <= 0 ? "expired" : completed ? "approved" : challenged ? "challenged" : "waiting";
				expect(status).toBe(expected);
			}),
		);
	});
});

describe("cancelling", () => {
	it("either side cancels; the poll says cancelled, the list drops it, and answers are 404", async () => {
		for (const canceller of ["requester", "approver"] as const) {
			const { id } = await named();
			const who = canceller === "requester" ? n : a;
			expect((await approvalsAs(who).cancel(id)).status).toBe(204);
			expect((await approvalsAs(who).cancel(id)).status).toBe(204);
			expect((await approvalsAs(n).poll(id)).body).toEqual({ status: "cancelled", expires_at: clock.now + FIVE_MINUTES });
			expect((await approvalsAs(a).list()).body.approvals).toEqual([]);
			expect((await approvalsAs(a).approve(id, wrappedKey())).status).toBe(404);
		}
	});

	it("an approved request is deleted by its requester only", async () => {
		const { id } = await named();
		await approvalsAs(a).approve(id, wrappedKey());
		expect((await approvalsAs(a).cancel(id)).body.error).toBe("approval_used");
		expect((await approvalsAs(n).cancel(id)).status).toBe(204);
		expect((await approvalsAs(n).cancel(id)).status).toBe(404);
	});

	it("a fourth pending request cancels the oldest, and the room hears it", async () => {
		const socket = await listening(a);
		const ids: string[] = [];
		for (let i = 0; i < 4; i++) {
			ids.push((await approvalsAs(n, { now: clock.now + i }).create(ephemeral().commit)).body.id);
		}
		const pending = (await approvalsAs(a, { now: clock.now + 4 }).list()).body.approvals.map((r: { id: string }) => r.id);
		expect(pending).toEqual(ids.slice(1));
		expect((await approvalsAs(n, { now: clock.now + 4 }).poll(ids[0])).body.status).toBe("cancelled");
		const frames = (await socket.drain(300)).map((f) => ("text" in f ? f.text : f));
		expect(frames).toEqual([
			{ type: "approval.created", id: ids[0] },
			{ type: "approval.created", id: ids[1] },
			{ type: "approval.created", id: ids[2] },
			{ type: "approval.cancelled", id: ids[0] },
			{ type: "approval.created", id: ids[3] },
		]);
		socket.ws.close();
	});

	it("concurrent creates still leave at most three pending", async () => {
		await Promise.all(
			Array.from({ length: 6 }, () => approvalsAs(n, { now: clock.now }).create(ephemeral().commit)),
		);
		const counted = await env.DB.prepare(
			`SELECT count(*) AS n FROM device_approvals d JOIN accounts a ON a.id = d.account_id
  WHERE a.username = ?1 AND d.cancelled_at IS NULL AND d.completed_at IS NULL AND d.expires_at > ?2`,
		)
			.bind("alice", clock.now)
			.first<{ n: number }>();
		expect(counted?.n).toBeLessThanOrEqual(3);
		const pending = (await approvalsAs(a, { now: clock.now }).list()).body.approvals;
		expect(pending.length).toBeLessThanOrEqual(3);
	});
});

describe("other devices", () => {
	it("counts the account's active sessions other than the requester's, 0 when it is alone", async () => {
		const solo = await signUp("solo");
		expect((await approvalsAs(first(solo)).create(ephemeral().commit)).body.other_devices).toBe(0);
		const third = await another(alice);
		expect((await approvalsAs(n).create(ephemeral().commit)).body.other_devices).toBe(2);
		await approvalsAs(a).cancel((await approvalsAs(a).list()).body.approvals[0].id);
		expect((await approvalsAs(third).create(ephemeral().commit)).body.other_devices).toBe(2);
	});
});

describe("limits", () => {
	it("approval-create allows 10 an hour per account", async () => {
		for (let i = 0; i < 10; i++) expect((await approvalsAs(n).create(ephemeral().commit)).status).toBe(200);
		const r = await approvalsAs(n).create(ephemeral().commit);
		expect(r.status).toBe(429);
		expect(r.body.limit).toBe("approval-create");
		expect((await approvalsAs(n, { now: clock.now + HOUR }).create(ephemeral().commit)).status).toBe(200);
	});

	it("approval-respond allows 30 answers an hour; approval-poll counts only the requester's poll", async () => {
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		const key = ephemeral().text;
		for (let i = 0; i < 30; i++) expect((await approvalsAs(a).challenge(id, key)).status).toBe(200);
		const r = await approvalsAs(a).challenge(id, key);
		expect(r.status).toBe(429);
		expect(r.body.limit).toBe("approval-respond");
		for (let i = 0; i < 5; i++) await approvalsAs(a).list();
		await approvalsAs(n).poll(id);
		const rows = await env.DB.prepare("SELECT key, count FROM limits WHERE key GLOB 'approval-*' ORDER BY key").all<{ key: string; count: number }>();
		expect(rows.results).toEqual([
			{ key: `approval-create:${alice.creds.accountId}`, count: 1 },
			{ key: `approval-poll:${alice.creds.accountId}`, count: 1 },
			{ key: `approval-respond:${alice.creds.accountId}`, count: 31 },
		]);
	});

	it("account deletion takes the approval rows and counters with it", async () => {
		await approvalsAs(n).create(ephemeral().commit);
		const r = await approvalsAs(a).list();
		expect(r.body.approvals.length).toBe(1);
		const { callJson } = await import("./helpers");
		const deleted = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: alice.creds.authKey, username: "alice" } });
		expect(deleted.status).toBe(200);
		expect((await env.DB.prepare("SELECT count(*) AS n FROM device_approvals").first<{ n: number }>())?.n).toBe(0);
		expect((await env.DB.prepare("SELECT count(*) AS n FROM limits WHERE key GLOB 'approval-*'").first<{ n: number }>())?.n).toBe(0);
	});
});

describe("what the Worker answers", () => {
	it("no answer about a request carries more than the protocol's fields", async () => {
		const { id } = await named();
		const allowedPoll = ["status", "expires_at", "approver_epk", "approver_device_id", "sealed_name", "wrapped_key"];
		for (const key of Object.keys((await approvalsAs(n).poll(id)).body)) expect(allowedPoll).toContain(key);
		const listed = (await approvalsAs(a).list()).body.approvals[0];
		expect(Object.keys(listed).sort()).toEqual(
			["approver_device_id", "approver_epk", "commit", "created_at", "device_id", "epk", "expires_at", "id", "sealed_name", "status"].sort(),
		);
	});
});
