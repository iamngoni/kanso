// The account event socket (design: Kanso accounts spec, revision 3 › GET
// /v2/account/events): the upgrade's authentication (token, device, device
// proof, no Origin), the hello, the approval hints, one socket per device,
// a room that hibernates and still delivers, ping/pong without a wake, the
// 1008 close for anything else, frames that carry only a type and an ID, and
// the catch-up GET after a reconnect.

import { evictDurableObject, runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import type { AccountRoom } from "../src/account_room";
import { type Signed, another, approvalsAs, ephemeral, first, listening, openEvents, sealedName, wrappedKey } from "./approval-helpers";
import { type Account, base64url, call, resetServer, signUp } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

let alice: Account;
let a: Signed;
let n: Signed;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	a = first(alice);
	n = await another(alice);
});

const room = () => env.ACCOUNT_ROOM.get(env.ACCOUNT_ROOM.idFromName(alice.creds.accountId));
const text = (f: unknown) => (f as { text: any }).text;

async function refused(s: Signed, headers: Record<string, string | null>, status: number, code: string): Promise<void> {
	const r = await openEvents(s, headers);
	expect(r instanceof Response).toBe(true);
	const response = r as Response;
	expect(response.status).toBe(status);
	expect(response.webSocket).toBeNull();
	expect(((await response.json()) as { error: string }).error).toBe(code);
}

describe("the upgrade", () => {
	it("needs the session's own device and its proof; anything else is 401 invalid_credentials and no socket", async () => {
		await refused(a, { "x-kanso-device-proof": null }, 401, "invalid_credentials");
		await refused(a, { "x-kanso-device-proof": n.device.proof }, 401, "invalid_credentials");
		await refused(a, { "x-kanso-device-proof": "not-a-proof" }, 401, "invalid_credentials");
		await refused(a, { "x-kanso-device": null }, 401, "invalid_credentials");
		await refused(a, { "x-kanso-device": n.device.id }, 401, "invalid_credentials");
		// n's device and proof with a's token: the token decides the device.
		await refused(a, { "x-kanso-device": n.device.id, "x-kanso-device-proof": n.device.proof }, 401, "invalid_credentials");
		await refused({ token: `kanso_${"A".repeat(43)}`, device: a.device }, {}, 401, "unauthorized");
		await refused(a, { origin: "https://evil.example" }, 403, "forbidden_origin");
		expect(await runInDurableObject(room(), (instance: AccountRoom) => instance.sockets())).toEqual([]);
	});

	it("property: any proof but the device's own is refused", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 32, maxLength: 32 }), async (bytes) => {
				const proof = base64url(bytes);
				fc.pre(proof !== a.device.proof);
				const r = await openEvents(a, { "x-kanso-device-proof": proof });
				expect((r as Response).status).toBe(401);
			}),
			runs(30),
		);
	});

	it("a plain GET with the right proof is 426", async () => {
		const r = await call("/v2/account/events", { token: a.token, headers: { "x-kanso-device": a.device.id, "x-kanso-device-proof": a.device.proof } });
		expect(r.status).toBe(426);
	});

	it("a signed-out session can't listen", async () => {
		await call("/v2/auth/logout", { token: n.token, method: "POST" });
		await refused(n, {}, 401, "unauthorized");
	});
});

describe("events", () => {
	it("says hello, then created, named and cancelled, each with only a type and an ID", async () => {
		const socket = await listening(a);
		const nKey = ephemeral();
		const id = (await approvalsAs(n).create(nKey.commit)).body.id;
		expect(text(await socket.next())).toEqual({ type: "approval.created", id });
		await approvalsAs(a).challenge(id, ephemeral().text);
		await approvalsAs(n).name(id, nKey.text, sealedName());
		expect(text(await socket.next())).toEqual({ type: "approval.named", id });
		await approvalsAs(n).cancel(id);
		expect(text(await socket.next())).toEqual({ type: "approval.cancelled", id });
		socket.ws.close();
	});

	it("every device of the account hears it, the requester too; another account hears nothing", async () => {
		const bob = await signUp("bob");
		const sockets = [await listening(a), await listening(n)];
		const other = await listening(first(bob));
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		for (const s of sockets) expect(text(await s.next())).toEqual({ type: "approval.created", id });
		expect(await other.drain()).toEqual([]);
		for (const s of [...sockets, other]) s.ws.close();
	});

	it("property: no frame carries a key, a name, a wrap or anything beyond its type and ID", async () => {
		await fc.assert(
			fc.asyncProperty(fc.integer({ min: 55, max: 400 }), async (length) => {
				const socket = await listening(a);
				const nKey = ephemeral();
				const aKey = ephemeral();
				const name = sealedName(length);
				const wrapped = wrappedKey();
				const id = (await approvalsAs(n).create(nKey.commit)).body.id;
				await approvalsAs(a).challenge(id, aKey.text);
				await approvalsAs(n).name(id, nKey.text, name);
				await approvalsAs(a).approve(id, wrapped);
				await approvalsAs(n).cancel(id);
				const frames = await socket.drain(150);
				expect(frames.length).toBeGreaterThan(0);
				for (const frame of frames) {
					const raw = JSON.stringify(frame);
					expect(Object.keys(text(frame)).sort()).toEqual(["id", "type"]);
					for (const secret of [nKey.text, nKey.commit, aKey.text, name, wrapped]) expect(raw.includes(secret)).toBe(false);
				}
				socket.ws.close();
			}),
			runs(10),
		);
	});
});

describe("one socket per device", () => {
	it("a second socket from the same device replaces the first (4000) and keeps receiving", async () => {
		const old = await listening(a);
		const fresh = await listening(a);
		expect(await old.closed).toEqual({ code: 4000, reason: "replaced" });
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		expect(text(await fresh.next())).toEqual({ type: "approval.created", id });
		expect(await runInDurableObject(room(), (instance: AccountRoom) => instance.sockets())).toEqual([a.device.id]);
		fresh.ws.close();
	});
});

describe("hibernation", () => {
	it("an evicted room still delivers to the sockets it accepted", async () => {
		const socket = await listening(a);
		await evictDurableObject(room());
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		expect(text(await socket.next())).toEqual({ type: "approval.created", id });
		socket.ws.close();
	});

	it("ping is answered with pong by the auto-response; any other frame closes with 1008", async () => {
		const socket = await listening(a);
		socket.ws.send("ping");
		expect(await socket.next()).toEqual({ raw: "pong" });
		socket.ws.send('{"type":"approve"}');
		expect((await socket.closed).code).toBe(1008);
	});
});

describe("reconnecting", () => {
	it("a request made while the socket was down is found by the one GET after reconnecting", async () => {
		const socket = await listening(a);
		socket.ws.close();
		const id = (await approvalsAs(n).create(ephemeral().commit)).body.id;
		const again = await listening(a);
		expect((await approvalsAs(a).list()).body.approvals.map((r: { id: string }) => r.id)).toEqual([id]);
		expect(await again.drain()).toEqual([]);
		again.ws.close();
	});

	it("a room that can't be reached never fails the route", async () => {
		const { testEnv, callJson } = await import("./helpers");
		const broken = testEnv({
			ACCOUNT_ROOM: {
				idFromName: () => {
					throw new Error("unreachable");
				},
			} as unknown as typeof env.ACCOUNT_ROOM,
		});
		const r = await callJson("/v2/account/sign-in/approvals", { token: n.token, json: { commit: ephemeral().commit }, env: broken });
		expect(r.status).toBe(200);
	});
});
