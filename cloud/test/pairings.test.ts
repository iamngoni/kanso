// Screenless-device codes are owner-created, peppered, short-lived and
// single-use. The successful flow enters the existing approval protocol.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { formatPairingCode, normalizePairingCode, pairingState, PAIRING_TTL_MS } from "../src/pairings";
import { approvalsAs, ephemeral, first } from "./approval-helpers";
import { base64url, callJson, clock, newDevice, resetServer, signUp } from "./helpers";

const randomBytes = (length: number) => crypto.getRandomValues(new Uint8Array(length));

let account: Awaited<ReturnType<typeof signUp>>;

beforeEach(async () => {
	await resetServer();
	account = await signUp("pairing-owner");
});

const create = (now = clock.now) =>
	callJson("/v2/account/pairings", {
		token: account.token,
		now,
		headers: { "x-kanso-device": account.device.id, "x-kanso-device-proof": account.device.proof },
		json: {},
	});

describe("pairing code format", () => {
	it("normalizes separators and Crockford lookalikes", () => {
		expect(normalizePairingCode("o1il-23ab")).toBe("011123AB");
		expect(normalizePairingCode("0000-0000")).toBe("00000000");
		expect(normalizePairingCode("not a code")).toBeNull();
	});

	const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
	it("round-trips every 40-bit generated code", () => {
		fc.assert(fc.property(fc.uint8Array({ minLength: 5, maxLength: 5 }), (bytes) => {
			const code = formatPairingCode(bytes);
			return normalizePairingCode(code) === code.replace("-", "");
		}), runs(100));
	});

	it("keeps a consumed code used at every time and expires unused codes at their deadline", () => {
		fc.assert(fc.property(fc.integer(), fc.integer(), (expiry, now) => {
			const expected = expiry <= now ? "expired" : "waiting";
			return pairingState(expiry, null, now) === expected && pairingState(expiry, now, now) === "used";
		}), runs(100));
	});
});

describe("pairing", () => {
	it("creates a private code, consumes it once and publishes an approval request", async () => {
		const created = await create();
		expect(created.status).toBe(200);
		expect(created.body.code).toMatch(/^[0-9A-HJKMNP-TV-Z]{4}-[0-9A-HJKMNP-TV-Z]{4}$/);
		expect(created.body.expires_at).toBe(clock.now + PAIRING_TTL_MS);
		const stored = await env.DB.prepare("SELECT code_hash, expires_at FROM device_pairings WHERE id = ?1")
			.bind(created.body.id).first<{ code_hash: string; expires_at: number }>();
		expect(stored?.code_hash).not.toBe(created.body.code.replace("-", ""));
		expect(stored?.expires_at).toBe(created.body.expires_at);

		const newDeviceIdentity = newDevice(false);
		const requestId = base64url(randomBytes(32));
		const requester = ephemeral();
		const paired = await callJson("/v2/auth/pair", {
			json: {
				code: created.body.code.toLowerCase(),
				device: { id: newDeviceIdentity.id, proof: newDeviceIdentity.proof },
				request_id: requestId,
				commit: requester.commit,
			},
		});
		expect(paired.status).toBe(200);
		expect(paired.body).toMatchObject({ kind: "session", account_id: account.creds.accountId, approval_id: requestId });
		expect((await approvalsAs(first(account)).list()).body.approvals.map((row: { id: string }) => row.id)).toContain(requestId);
		expect((await callJson(`/v2/account/pairings/${created.body.id}`, { token: account.token })).body.status).toBe("used");
		const usedAgain = await callJson("/v2/auth/pair", {
			json: { code: created.body.code, device: { id: newDeviceIdentity.id, proof: newDeviceIdentity.proof }, request_id: base64url(randomBytes(32)), commit: requester.commit },
		});
		expect(usedAgain.status).toBe(400);
		expect(usedAgain.body.error).toBe("bad_code");
	});

	it("expires in ten minutes and refuses a creator without its device proof", async () => {
		const denied = await callJson("/v2/account/pairings", { token: account.token, json: {}, headers: { "x-kanso-device": account.device.id, "x-kanso-device-proof": base64url(randomBytes(32)) } });
		expect(denied.status).toBe(401);
		const created = await create();
		const device = newDevice(false);
		const response = await callJson("/v2/auth/pair", {
			now: clock.now + PAIRING_TTL_MS,
			json: { code: created.body.code, device: { id: device.id, proof: device.proof }, request_id: base64url(randomBytes(32)), commit: ephemeral().commit },
		});
		expect(response.status).toBe(400);
		expect(response.body.error).toBe("bad_code");
	});

	it("keeps at most three live codes", async () => {
		const firstCode = await create(clock.now);
		await create(clock.now + 1);
		await create(clock.now + 2);
		const fourth = await create(clock.now + 3);
		const rows = await env.DB.prepare("SELECT id FROM device_pairings WHERE account_id = ?1 AND used_at IS NULL AND expires_at > ?2 ORDER BY created_at")
			.bind(account.creds.accountId, clock.now + 3).all<{ id: string }>();
		expect(rows.results).toHaveLength(3);
		expect(rows.results.map((row) => row.id)).not.toContain(firstCode.body.id);
		expect(rows.results.map((row) => row.id)).toContain(fourth.body.id);
	});
});
