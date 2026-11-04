// In-app feedback intake (docs/FEEDBACK.md): both ways in, validation that never echoes a value, the
// idempotent ID, the screenshot slot, the rate limits (also in parallel), list and delete, the account
// deletion cascade, the retention job, the migration, and properties over generated reports
// (fast-check, seed pinned in vitest.config.ts).

import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { KEEP_DAYS, UPLOAD_WINDOW_MS, cleanText, housekeeping, imageMatches, preview, validateReport } from "../src/feedback";
import { START, type Account, call, callJson, clock, resetServer, setSetting, signUp } from "./helpers";
import { APP, JPEG, PNG, configured, fakeNetwork, newId, newKey, report, sha256Hex, withScreenshot } from "./feedback-helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const DAY = 86_400_000;
const PATH = "/v1/feedback";

type Init = { key?: string; ip?: string; now?: number; as?: Account; headers?: Record<string, string> };

function headersOf(init: Init): Record<string, string> {
	if (init.as) return { "x-kanso-device": init.as.device.id, "x-kanso-device-proof": init.as.device.proof, ...init.headers };
	return { "x-kanso-feedback-key": init.key ?? KEY, ...init.headers };
}
const KEY = newKey();

async function post(body: unknown, init: Init = {}, raw = false) {
	const r = await call(PATH, {
		method: "POST",
		body: raw ? (body as string) : JSON.stringify(body),
		token: init.as?.token,
		headers: { "content-type": "application/json", ...headersOf(init) },
		ip: init.ip ?? "203.0.113.9",
		now: init.now,
	});
	const text = await r.text();
	return { status: r.status, text, body: text === "" ? null : (JSON.parse(text) as any) };
}

async function put(id: string, bytes: Uint8Array, init: Init & { type?: string } = {}) {
	return call(`${PATH}/${id}/screenshot`, {
		method: "PUT",
		body: bytes,
		token: init.as?.token,
		headers: { "content-type": init.type ?? "image/png", ...headersOf(init) },
		ip: init.ip ?? "203.0.113.9",
		now: init.now,
	});
}

const get = (init: Init = {}, query = "") =>
	callJson(`${PATH}${query}`, { token: init.as?.token, headers: headersOf(init), ip: init.ip ?? "203.0.113.9", now: init.now });
const del = (id: string, init: Init = {}) =>
	call(`${PATH}/${id}`, { method: "DELETE", token: init.as?.token, headers: headersOf(init), ip: init.ip ?? "203.0.113.9", now: init.now });

const rows = async () => (await env.DB.prepare("SELECT * FROM feedback ORDER BY created_at, id").all<any>()).results;
const events = async (id: string) => (await env.DB.prepare("SELECT status, at FROM feedback_events WHERE feedback_id = ?1 ORDER BY at").bind(id).all<any>()).results;

beforeEach(async () => {
	await resetServer();
	await env.DB.batch([env.DB.prepare("DELETE FROM feedback_events"), env.DB.prepare("DELETE FROM feedback"), env.DB.prepare("DELETE FROM feedback_purges"), env.DB.prepare("DELETE FROM feedback_deliveries"), env.DB.prepare("DELETE FROM feedback_credit_removals")]);
});

describe("accepting", () => {
	it("stores an anonymous report privately and answers 201", async () => {
		const r = report();
		const res = await post(r, { now: START });
		expect(res.status).toBe(201);
		expect(res.body).toEqual({ id: r.id, status: "received" });
		const [row] = await rows();
		expect(row).toMatchObject({ id: r.id, account_id: null, created_at: START, kind: "bug", status: "received", has_screenshot: 0, screenshot_state: "none", issue: null });
		expect(row.reporter).toMatch(/^r_[0-9a-f]{32}$/);
		expect(row.report).toBe(JSON.stringify(r));
		// Neither the key, the address nor a device ID is in the row.
		const all = JSON.stringify(row);
		expect(all).not.toContain(KEY);
		expect(all).not.toContain("203.0.113.9");
		expect(await events(r.id)).toEqual([{ status: "received", at: START }]);
	});

	it("stores a signed-in report under the account, with the device proof", async () => {
		const a = await signUp("alice");
		const r = report({ credit: { name: "tendai" } });
		const res = await post(r, { as: a });
		expect(res.status).toBe(201);
		const [row] = await rows();
		expect(row).toMatchObject({ account_id: a.creds.accountId, reporter: null, credit_name: "tendai" });
		expect(JSON.stringify(row)).not.toContain(a.device.id);
	});

	it("takes the report with no app info and re-serializes in the page's key order", async () => {
		const r = report({ app: null });
		const shuffled = { screenshot: null, app: null, credit: null, text: r.text, kind: "bug", id: r.id, v: 1 };
		expect((await post(shuffled)).status).toBe(201);
		expect((await rows())[0].report).toBe(JSON.stringify(r));
	});

	it("normalizes the text to NFC before it counts and stores it", async () => {
		const r = report({ text: "Café menu is cut off on the sidebar when I resize" });
		expect((await post(r)).status).toBe(201);
		expect(JSON.parse((await rows())[0].report).text).toBe("Café menu is cut off on the sidebar when I resize");
	});

	it("a repeat ID from the same reporter is a 200 with the current status; another reporter's is a 409", async () => {
		const r = report();
		expect((await post(r)).status).toBe(201);
		const again = await post(r);
		expect(again.status).toBe(200);
		expect(again.body).toEqual({ id: r.id, status: "received" });
		expect(await rows()).toHaveLength(1);
		expect((await post(r, { key: newKey() })).status).toBe(409);
		const a = await signUp("alice");
		expect((await post(r, { as: a })).status).toBe(409);
	});

	it("a repeat while the screenshot slot is open answers the slot again", async () => {
		const r = await withScreenshot();
		const first = await post(r, { now: START });
		expect(first.body.upload).toEqual({ url: `/v1/feedback/${r.id}/screenshot`, expires_ms: START + UPLOAD_WINDOW_MS });
		expect((await post(r, { now: START + 5 })).body.upload).toEqual(first.body.upload);
	});
});

describe("refusing", () => {
	it("401 without a key or a session, and for a malformed key", async () => {
		expect((await call(PATH, { method: "POST", json: report() })).status).toBe(401);
		expect((await post(report(), { key: "short" })).status).toBe(401);
		expect((await get({ key: "nope" })).status).toBe(401);
		expect(await rows()).toEqual([]);
	});

	it("a session token alone is not enough: the device and its proof are checked", async () => {
		const a = await signUp("alice");
		const bare = (headers: Record<string, string>) => call(PATH, { method: "POST", json: report(), token: a.token, headers });
		expect((await bare({})).status).toBe(401);
		expect((await bare({ "x-kanso-device": a.device.id })).status).toBe(401);
		expect((await bare({ "x-kanso-device-proof": a.device.proof })).status).toBe(401);
		const other = await signUp("bob");
		// Another device's proof, or another device's ID, with this token.
		expect((await bare({ "x-kanso-device": a.device.id, "x-kanso-device-proof": other.device.proof })).status).toBe(401);
		expect((await bare({ "x-kanso-device": other.device.id, "x-kanso-device-proof": a.device.proof })).status).toBe(401);
		expect((await bare({ "x-kanso-device": a.device.id, "x-kanso-device-proof": a.device.proof })).status).toBe(201);
		// A bad token is a 401 even with a feedback key: the bearer is the only way in once sent.
		expect((await call(PATH, { method: "POST", json: report(), token: "kanso_bad", headers: { "x-kanso-feedback-key": KEY } })).status).toBe(401);
	});

	it("415 for another content type, 413 over 48 KiB, 400 for non-JSON", async () => {
		expect((await call(PATH, { method: "POST", body: "x", headers: { "content-type": "text/plain", "x-kanso-feedback-key": KEY } })).status).toBe(415);
		expect((await post(report({ text: "x".repeat(60_000) }))).status).toBe(413);
		expect((await post("not json", {}, true)).status).toBe(400);
	});

	it("503 feedback_closed when the switch is off or the setting is missing", async () => {
		await setSetting("feedback_open", "0");
		const closed = await post(report());
		expect(closed.status).toBe(503);
		expect(closed.body.error).toBe("feedback_closed");
		await setSetting("feedback_open", "1");
		expect((await post(report())).status).toBe(201);
	});

	const bad: [string, (r: Record<string, any>) => void, string][] = [
		["a missing key", (r) => delete r.kind, "kind"],
		["an extra key", (r) => (r.extra = 1), "(body)"],
		["version 2", (r) => (r.v = 2), "v"],
		["an ID of the wrong shape", (r) => (r.id = "fb_UPPERCASE00000000000000000"), "id"],
		["another kind", (r) => (r.kind = "rant"), "kind"],
		["text under 10 characters", (r) => (r.text = "too short"), "text"],
		["text over 4,000 characters", (r) => (r.text = "a".repeat(4001)), "text"],
		["a control character", (r) => (r.text = "line one\u0007 and a bell in the middle"), "text"],
		["a carriage return", (r) => (r.text = "line one\r\nline two with more words"), "text"],
		["a lone surrogate", (r) => (r.text = "a broken emoji \ud83d in the middle of text"), "text"],
		["a credit with a mention", (r) => (r.credit = { name: "@tendai" }), "credit.name"],
		["a credit name over 39 characters", (r) => (r.credit = { name: "a".repeat(40) }), "credit.name"],
		["a blank credit name", (r) => (r.credit = { name: "   " }), "credit.name"],
		["a credit with an extra key", (r) => (r.credit = { name: "tendai", url: "x" }), "credit"],
		["a platform that is not one", (r) => (r.app.platform = "beos"), "app.platform"],
		["a version that is not x.y.z", (r) => (r.app.version = "0.9"), "app.version"],
		["a device with a quote", (r) => (r.app.device = 'Mac"14'), "app.device"],
		["a screen outside the enum", (r) => (r.app.screen = "note-7f3a"), "app.screen"],
		["a sync outside the enum", (r) => (r.app.sync = "cloud"), "app.sync"],
		["an error line with a path", (r) => (r.app.errors = ["update: failed at /Users/me/notes"]), "app.errors[0]"],
		["an error line from another source", (r) => (r.app.errors = ["editor: crashed"]), "app.errors[0]"],
		["21 error lines", (r) => (r.app.errors = Array(21).fill("sync: offline")), "app.errors"],
		["an extra app field", (r) => (r.app.notes = 3), "app"],
		["a screenshot of another type", (r) => (r.screenshot = { type: "image/gif", bytes: 10, sha256: "0".repeat(64) }), "screenshot.type"],
		["a screenshot over 5 MiB", (r) => (r.screenshot = { type: "image/png", bytes: 5 * 1024 * 1024 + 1, sha256: "0".repeat(64) }), "screenshot.bytes"],
		["a screenshot hash in capitals", (r) => (r.screenshot = { type: "image/png", bytes: 10, sha256: "A".repeat(64) }), "screenshot.sha256"],
	];
	for (const [name, mutate, field] of bad) {
		it(`400 for ${name}, naming ${field} and echoing no value`, async () => {
			const r = report();
			mutate(r);
			const secret = JSON.stringify(r);
			const res = await post(r);
			expect(res.status).toBe(400);
			expect(res.body).toMatchObject({ error: "invalid", field });
			for (const word of ["rant", "@tendai", "beos", "notes", "gif", "UPPERCASE", "Users/me"]) {
				if (secret.includes(word)) expect(res.text).not.toContain(word);
			}
			expect(res.text).not.toContain(r.id);
			expect(await rows()).toEqual([]);
		});
	}
});

describe("rate limits", () => {
	it("an anonymous reporter gets 5 reports a day from one address, then 429 feedback-ip", async () => {
		for (let i = 0; i < 5; i++) expect((await post(report(), { now: START + i, key: newKey() })).status).toBe(201);
		const sixth = await post(report(), { now: START + 6, key: newKey() });
		expect(sixth.status).toBe(429);
		expect(sixth.body).toMatchObject({ error: "rate_limited", limit: "feedback-ip" });
		// Another address is not affected; the next day resets.
		expect((await post(report(), { now: START + 7, ip: "198.51.100.4" })).status).toBe(201);
		expect((await post(report(), { now: START + DAY + 1 })).status).toBe(201);
	});

	it("a repeat of a stored report does not count", async () => {
		const r = report();
		await post(r);
		for (let i = 0; i < 8; i++) expect((await post(r)).status).toBe(200);
		expect((await rows()).length).toBe(1);
	});

	it("a signed-in account gets 10 a day, 429 feedback-account", async () => {
		const a = await signUp("alice");
		for (let i = 0; i < 10; i++) expect((await post(report(), { as: a })).status).toBe(201);
		const eleventh = await post(report(), { as: a });
		expect(eleventh.status).toBe(429);
		expect(eleventh.body.limit).toBe("feedback-account");
	});

	it("the global day cap fails closed: 429 feedback-global past the setting, and when it is missing", async () => {
		await setSetting("feedback_per_day", "2");
		expect((await post(report(), { key: newKey(), ip: "192.0.2.1" })).status).toBe(201);
		expect((await post(report(), { key: newKey(), ip: "192.0.2.2" })).status).toBe(201);
		const third = await post(report(), { key: newKey(), ip: "192.0.2.3" });
		expect(third.status).toBe(429);
		expect(third.body.limit).toBe("feedback-global");
	});

	it("parallel requests cannot pass the limit together", async () => {
		const results = await Promise.all(Array.from({ length: 12 }, () => post(report(), { key: newKey(), now: START })));
		expect(results.filter((r) => r.status === 201)).toHaveLength(5);
		expect(results.filter((r) => r.status === 429)).toHaveLength(7);
		expect(await rows()).toHaveLength(5);
	});

	it("parallel requests with the same ID store one report", async () => {
		const r = report();
		const results = await Promise.all(Array.from({ length: 4 }, () => post(r, { now: START })));
		expect(results.filter((x) => x.status === 201).length).toBeGreaterThanOrEqual(1);
		expect(results.every((x) => x.status === 201 || x.status === 200)).toBe(true);
		expect(await rows()).toHaveLength(1);
	});
});

describe("the screenshot", () => {
	it("stores a PNG in R2 under feedback/<id>/ with its type, answers 204, then 409", async () => {
		const r = await withScreenshot();
		await post(r, { now: START });
		const res = await put(r.id, PNG, { now: START + 1000 });
		expect(res.status).toBe(204);
		const object = await env.FILES.get(`feedback/${r.id}/screenshot`);
		expect(new Uint8Array(await object!.arrayBuffer())).toEqual(PNG);
		expect(object!.customMetadata).toEqual({ type: "image/png" });
		expect((await rows())[0].screenshot_state).toBe("stored");
		expect((await put(r.id, PNG)).status).toBe(409);
		// It sits outside accounts/, so no account's quota or reconcile sees it.
		expect((await env.FILES.list({ prefix: "accounts/" })).objects).toEqual([]);
	});

	it("takes a JPEG when that is what was declared", async () => {
		const r = await withScreenshot(JPEG, "image/jpeg");
		await post(r);
		expect((await put(r.id, JPEG, { type: "image/jpeg" })).status).toBe(204);
	});

	it("refuses bytes that don't match the declaration, and the slot stays open", async () => {
		const r = await withScreenshot();
		await post(r);
		const other = Uint8Array.from(PNG);
		other[10] ^= 1;
		expect((await put(r.id, other)).status).toBe(400);
		expect((await put(r.id, PNG.slice(0, 12))).status).toBe(400);
		expect((await put(r.id, JPEG, { type: "image/jpeg" })).status).toBe(415);
		expect((await put(r.id, PNG, { type: "text/plain" })).status).toBe(415);
		expect((await rows())[0].screenshot_state).toBe("awaited");
		expect((await put(r.id, PNG)).status).toBe(204);
	});

	it("refuses a file whose magic bytes are not an image, even with a matching hash", async () => {
		const text = new TextEncoder().encode("<html>not an image</html>");
		const r = report({ screenshot: { type: "image/png", bytes: text.length, sha256: await sha256Hex(text) } });
		await post(r);
		expect((await put(r.id, text)).status).toBe(400);
		expect(await env.FILES.get(`feedback/${r.id}/screenshot`)).toBeNull();
	});

	it("404 for a report without a declared screenshot, an unknown ID, and another reporter's", async () => {
		const none = report();
		await post(none);
		expect((await put(none.id, PNG)).status).toBe(404);
		expect((await put(newId(), PNG)).status).toBe(404);
		const r = await withScreenshot();
		await post(r);
		expect((await put(r.id, PNG, { key: newKey() })).status).toBe(404);
	});

	it("410 once the slot's hour has passed, and the cron marks it expired", async () => {
		const r = await withScreenshot();
		await post(r, { now: START });
		expect((await put(r.id, PNG, { now: START + UPLOAD_WINDOW_MS + 1 })).status).toBe(410);
		const done = await housekeeping(await configured(), START + UPLOAD_WINDOW_MS + 1);
		expect(done.slotsExpired).toBe(1);
		expect((await rows())[0].screenshot_state).toBe("expired");
	});

	it("two uploads at once store one", async () => {
		const r = await withScreenshot();
		await post(r);
		const statuses = (await Promise.all([put(r.id, PNG), put(r.id, PNG), put(r.id, PNG)])).map((x) => x.status).sort();
		expect(statuses).toEqual([204, 409, 409]);
	});
});

describe("listing", () => {
	it("returns only this reporter's reports, newest first, with the status chain", async () => {
		const a = report({ text: "First line of the older report\nsecond line stays out" });
		const b = report();
		await post(a, { now: START });
		await post(b, { now: START + 1000 });
		await post(report(), { key: newKey(), ip: "198.51.100.1", now: START + 2000 });
		const res = await get();
		expect(res.status).toBe(200);
		expect(res.body.reports.map((x: any) => x.id)).toEqual([b.id, a.id]);
		expect(res.body.reports[1]).toEqual({
			id: a.id,
			created_ms: START,
			kind: "bug",
			preview: "First line of the older report",
			status: "received",
			issue: null,
			duplicate_of: null,
			pr: null,
			release: null,
			events: [{ status: "received", at_ms: START }],
		});
	});

	it("a signed-in reporter sees theirs, not the anonymous ones", async () => {
		const a = await signUp("alice");
		const mine = report();
		await post(mine, { as: a });
		await post(report());
		expect((await get({ as: a })).body.reports.map((x: any) => x.id)).toEqual([mine.id]);
	});

	it("filters by the release that fixed a report", async () => {
		const a = report();
		const b = report();
		await post(a, { now: START });
		await post(b, { now: START + 1 });
		await env.DB.prepare("UPDATE feedback SET status = 'released', release = 'v0.9.2' WHERE id = ?1").bind(a.id).run();
		expect((await get({}, "?released=0.9.2")).body.reports.map((x: any) => x.id)).toEqual([a.id]);
		expect((await get({}, "?released=0.9.3")).body.reports).toEqual([]);
		expect((await get({}, "?released=latest")).status).toBe(400);
	});

	it("caps the list at 100", async () => {
		const stmts = Array.from({ length: 105 }, (_, i) =>
			env.DB.prepare("INSERT INTO feedback (id, reporter, created_at, kind, report) VALUES (?1, ?2, ?3, 'bug', '{}')").bind(`fb_${String(i).padStart(26, "0")}`, "r_" + "0".repeat(32), START + i),
		);
		await env.DB.batch(stmts);
		await post(report(), { key: newKey() });
		const { reporterHash } = await import("../src/feedback");
		const { pepperKeys } = await import("../src/crypto");
		const hash = await reporterHash((await pepperKeys(env.AUTH_PEPPER))!, KEY);
		await env.DB.prepare("UPDATE feedback SET reporter = ?1 WHERE reporter = ?2").bind(hash, "r_" + "0".repeat(32)).run();
		expect((await get()).body.reports).toHaveLength(100);
	});
});

describe("deleting", () => {
	it("removes the row, its events and the screenshot, and answers 204", async () => {
		const r = await withScreenshot();
		await post(r);
		await put(r.id, PNG);
		expect((await del(r.id)).status).toBe(204);
		expect(await rows()).toEqual([]);
		expect(await events(r.id)).toEqual([]);
		expect(await env.FILES.get(`feedback/${r.id}/screenshot`)).toBeNull();
		expect((await env.DB.prepare("SELECT * FROM feedback_purges").all()).results).toEqual([]);
		expect((await del(r.id)).status).toBe(404);
	});

	it("404 for another reporter's report, which stays", async () => {
		const r = report();
		await post(r);
		expect((await del(r.id, { key: newKey() })).status).toBe(404);
		expect(await rows()).toHaveLength(1);
		const a = await signUp("alice");
		expect((await del(r.id, { as: a })).status).toBe(404);
	});

	it("leaves a queued purge for the cron when the R2 clean-up cannot run, and the cron finishes it", async () => {
		const r = await withScreenshot();
		await post(r);
		await put(r.id, PNG);
		await env.DB.prepare("INSERT OR REPLACE INTO feedback_purges (feedback_id, requested_at) VALUES (?1, ?2)").bind(r.id, START).run();
		await housekeeping(await configured(), START + 1);
		expect(await env.FILES.get(`feedback/${r.id}/screenshot`)).toBeNull();
		expect((await env.DB.prepare("SELECT * FROM feedback_purges").all()).results).toEqual([]);
	});

	it("takes the credit line off the public issue, keeping the issue", async () => {
		const net = fakeNetwork();
		try {
			const e = await configured();
			const r = report({ credit: { name: "tendai" } });
			await post(r);
			net.issueBodies.set(301, "**Summary.** x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku · Reported by tendai</sub>");
			await env.DB.prepare("UPDATE feedback SET status = 'filed', issue = 301 WHERE id = ?1").bind(r.id).run();
			const res = await call(`${PATH}/${r.id}`, { method: "DELETE", headers: headersOf({}), env: e });
			expect(res.status).toBe(204);
			expect(net.issueBodies.get(301)).toBe("**Summary.** x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku</sub>");
			expect(await rows()).toEqual([]);
		} finally {
			net.restore();
		}
	});

	it("still deletes when GitHub is down", async () => {
		const net = fakeNetwork();
		net.failing.push("/issues/301");
		try {
			const r = report({ credit: { name: "tendai" } });
			await post(r);
			await env.DB.prepare("UPDATE feedback SET status = 'filed', issue = 301 WHERE id = ?1").bind(r.id).run();
			expect((await call(`${PATH}/${r.id}`, { method: "DELETE", headers: headersOf({}), env: await configured() })).status).toBe(204);
			expect(await rows()).toEqual([]);
		} finally {
			net.restore();
		}
	});
});

describe("account deletion", () => {
	it("deletes the account's reports, events and screenshots with it, and leaves everyone else's", async () => {
		const a = await signUp("alice");
		const mine = await withScreenshot();
		await post(mine, { as: a });
		await put(mine.id, PNG, { as: a });
		const others = report();
		await post(others);
		const gone = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
		expect(gone.status).toBe(200);
		expect((await rows()).map((x: any) => x.id)).toEqual([others.id]);
		expect(await events(mine.id)).toEqual([]);
		await housekeeping(await configured(), clock.now + 1);
		expect(await env.FILES.get(`feedback/${mine.id}/screenshot`)).toBeNull();
		expect((await env.DB.prepare("SELECT key FROM limits WHERE key LIKE 'feedback-account:%'").all()).results).toEqual([]);
	});
});

describe("account deletion credit lines", () => {
	const body = "**Summary.** x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku · Reported by tendai</sub>";
	const stripped = "**Summary.** x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku</sub>";

	it("queues each filed issue's credit removal with the account, and the cron retries through a GitHub outage", async () => {
		const net = fakeNetwork();
		try {
			const a = await signUp("alice");
			const r = report({ credit: { name: "tendai" } });
			await post(r, { as: a });
			await env.DB.prepare("UPDATE feedback SET status = 'filed', issue = 301 WHERE id = ?1").bind(r.id).run();
			net.issueBodies.set(301, body);
			const gone = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
			expect(gone.status).toBe(200);
			expect((await env.DB.prepare("SELECT issue, name FROM feedback_credit_removals").all()).results).toEqual([{ issue: 301, name: "tendai" }]);
			net.failing.push("/issues/301");
			expect((await housekeeping(await configured(), clock.now + 1)).credits).toBe(0);
			expect(net.issueBodies.get(301)).toBe(body);
			expect((await env.DB.prepare("SELECT issue FROM feedback_credit_removals").all()).results).toHaveLength(1);
			net.failing.length = 0;
			expect((await housekeeping(await configured(), clock.now + 2)).credits).toBe(1);
			expect(net.issueBodies.get(301)).toBe(stripped);
			expect((await env.DB.prepare("SELECT issue FROM feedback_credit_removals").all()).results).toEqual([]);
		} finally {
			net.restore();
		}
	});

	it("a single deletion during an outage is retried by the cron too", async () => {
		const net = fakeNetwork();
		try {
			const r = report({ credit: { name: "tendai" } });
			await post(r);
			await env.DB.prepare("UPDATE feedback SET status = 'filed', issue = 301 WHERE id = ?1").bind(r.id).run();
			net.issueBodies.set(301, body);
			net.failing.push("/issues/301");
			expect((await call(`${PATH}/${r.id}`, { method: "DELETE", headers: headersOf({}), env: await configured() })).status).toBe(204);
			net.failing.length = 0;
			await housekeeping(await configured(), START + 1);
			expect(net.issueBodies.get(301)).toBe(stripped);
		} finally {
			net.restore();
		}
	});
});

describe("retention", () => {
	const seed = async (id: string, status: string, createdAt: number, lastEvent: number, screenshot = false) => {
		await env.DB.batch([
			env.DB.prepare(
				`INSERT INTO feedback (id, reporter, created_at, kind, report, status, has_screenshot, screenshot_state, credit_name, triage)
  VALUES (?1, ?2, ?3, 'bug', ?4, ?5, ?6, ?7, 'tendai', '{"x":1}')`,
			).bind(id, "r_" + "1".repeat(32), createdAt, JSON.stringify(report({ id, credit: { name: "tendai" } })), status, screenshot ? 1 : 0, screenshot ? "stored" : "none"),
			env.DB.prepare("INSERT OR IGNORE INTO feedback_events (feedback_id, at, status) VALUES (?1, ?2, 'received'), (?1, ?3, ?4)").bind(id, createdAt, lastEvent, status),
		]);
		if (screenshot) await env.FILES.put(`feedback/${id}/screenshot`, PNG);
	};

	it("blanks the text, the triage and the screenshot 180 days after the issue closes, and keeps the chain", async () => {
		const id = newId();
		const closed = START;
		await seed(id, "released", closed - 400 * DAY, closed, true);
		await housekeeping(await configured(), closed + KEEP_DAYS * DAY - 1);
		expect((await rows())[0].purged_at).toBeNull();
		const done = await housekeeping(await configured(), closed + KEEP_DAYS * DAY);
		expect(done.retained).toBe(1);
		const [row] = await rows();
		expect(JSON.parse(row.report)).toEqual({ v: 1, id, kind: "bug", text: "", credit: null, app: null, screenshot: null });
		expect(row).toMatchObject({ status: "released", triage: null, credit_name: null, screenshot_state: "expired" });
		expect(await env.FILES.get(`feedback/${id}/screenshot`)).toBeNull();
	});

	it("purges every status 180 days after its last change, so nothing is kept forever", async () => {
		const ids: Record<string, string> = {};
		for (const status of ["noise", "received", "filed", "building", "pr_open", "closed", "duplicate"]) {
			ids[status] = newId();
			await seed(ids[status], status, START, START);
		}
		const recent = newId();
		await seed(recent, "filed", START, START + 10 * DAY);
		await housekeeping(await configured(), START + KEEP_DAYS * DAY + 1);
		const byId = new Map((await rows()).map((x: any) => [x.id, x.purged_at]));
		for (const id of Object.values(ids)) expect(byId.get(id)).not.toBeNull();
		expect(byId.get(recent)).toBeNull();
	});

	it("forgets webhook deliveries after a day", async () => {
		await env.DB.prepare("INSERT INTO feedback_deliveries (id, at) VALUES ('old', ?1), ('new', ?2)").bind(START, START + DAY).run();
		await housekeeping(await configured(), START + DAY + 1);
		expect((await env.DB.prepare("SELECT id FROM feedback_deliveries").all<any>()).results.map((x) => x.id)).toEqual(["new"]);
	});
});

describe("route table", () => {
	it("refuses browsers and other methods", async () => {
		expect((await call(PATH, { method: "POST", json: report(), headers: { origin: "https://evil.example", "x-kanso-feedback-key": KEY } })).status).toBe(403);
		expect((await call(PATH, { method: "PUT", headers: headersOf({}) })).status).toBe(405);
		expect((await call(`${PATH}/${newId()}`, { method: "GET", headers: headersOf({}) })).status).toBe(405);
		expect((await call(`${PATH}/${newId()}/screenshot`, { method: "POST", headers: headersOf({}) })).status).toBe(405);
		expect((await call(`${PATH}/${newId()}/other`, { headers: headersOf({}) })).status).toBe(404);
	});
});

describe("0016_feedback.sql", () => {
	it("migrates a database pinned at 0015 to head, keeping its rows and seeding the settings", async () => {
		const db = env.MIGRATION_DB;
		const before = env.TEST_MIGRATIONS.filter((m) => m.name < "0016");
		await applyD1Migrations(db, before);
		await db.prepare("INSERT INTO crash_reports (received_day, kind, type, app_version, report) VALUES ('2026-10-26', 'dart', 'E', '1.0.0', '{}')").run();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		expect((await db.prepare("SELECT count(*) AS n FROM crash_reports").first<any>()).n).toBe(1);
		const tables = (await db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE 'feedback%' ORDER BY name").all<any>()).results.map((t) => t.name);
		expect(tables).toEqual(["feedback", "feedback_credit_removals", "feedback_deliveries", "feedback_events", "feedback_purges"]);
		const settings = (await db.prepare("SELECT name, value FROM settings WHERE name LIKE 'feedback%' ORDER BY name").all<any>()).results;
		expect(settings).toEqual([
			{ name: "feedback_issues_per_day", value: "50" },
			{ name: "feedback_open", value: "1" },
			{ name: "feedback_per_day", value: "200" },
		]);
		// Applying again changes nothing.
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
	});

	it("holds its constraints: exactly one of account and reporter, and the ID shape", async () => {
		const insert = (id: string, account: string | null, reporter: string | null) =>
			env.DB.prepare("INSERT INTO feedback (id, account_id, reporter, created_at, kind, report) VALUES (?1, ?2, ?3, 0, 'bug', '{}')").bind(id, account, reporter).run();
		const id = newId();
		await expect(insert(id, null, null)).rejects.toThrow(/CHECK/);
		await expect(insert(id, "a_" + "0".repeat(32), "r_" + "0".repeat(32))).rejects.toThrow(/CHECK/);
		await expect(insert("fb_short", null, "r_" + "0".repeat(32))).rejects.toThrow(/CHECK/);
		await expect(env.DB.prepare("INSERT INTO feedback (id, reporter, created_at, kind, report) VALUES (?1, ?2, 0, 'rant', '{}')").bind(id, "r_" + "0".repeat(32)).run()).rejects.toThrow(/CHECK/);
		await insert(id, null, "r_" + "0".repeat(32));
	});
});

// ---------------------------------------------------------------- properties

const word = fc.string({ unit: fc.constantFrom(..."abcdefghij klmnop,.-éü漢"), minLength: 1, maxLength: 12 });
const genText = fc.array(word, { minLength: 3, maxLength: 80 }).map((w) => w.join(" ")).filter((t) => [...t].length >= 10 && [...t].length <= 4000);
const genApp = fc.option(
	fc.record({
		platform: fc.constantFrom("macos", "windows", "linux", "ios", "android"),
		version: fc.tuple(fc.nat(9999), fc.nat(9999), fc.nat(9999)).map((v) => v.join(".")),
		build: fc.nat().map(String),
		patch: fc.option(fc.nat(100_000), { nil: null }),
		os: fc.constantFrom("15.1", "11", "6.8.0-1", "14"),
		device: fc.constantFrom("Mac14,9", "iPhone17,1", "Pixel 8"),
		locale: fc.constantFrom("en-GB", "en", "pt_BR", "zh-Hans-CN"),
		screen: fc.constantFrom("editor", "note-list", "settings", "capture", "ask", "board", "graph", "calendar", "sync", "account", "other"),
		sync: fc.constantFrom("none", "kanso", "self-hosted"),
		errors: fc.array(fc.constantFrom("update: check failed: timeout", "sync: offline", "crash: StateError"), { maxLength: 20 }),
	}),
	{ nil: null },
);
const genReport = fc.record({
	v: fc.constant(1),
	id: fc.string({ unit: fc.constantFrom(...chars26()), minLength: 26, maxLength: 26 }).map((s) => `fb_${s}`),
	kind: fc.constantFrom("bug", "idea", "question"),
	text: genText,
	credit: fc.option(fc.record({ name: fc.constantFrom("tendai", "Ana María", "j.doe-1", "名前") }), { nil: null }),
	app: genApp,
	screenshot: fc.constant(null),
});
function chars26(): string[] {
	return [..."0123456789abcdefghijklmnopqrstuvwxyz"];
}

describe("properties", () => {
	it("a valid report validates, and validating the result again changes nothing", () => {
		fc.assert(
			fc.property(genReport, (r) => {
				const once = validateReport(r);
				expect(once.ok).toBe(true);
				if (!once.ok) return;
				expect(Object.keys(once.value)).toEqual(["v", "id", "kind", "text", "credit", "app", "screenshot"]);
				const twice = validateReport(JSON.parse(JSON.stringify(once.value)));
				expect(twice).toEqual(once);
			}),
			runs(200),
		);
	});

	it("validateReport never throws on any JSON, and an accepted text is NFC, 10-4,000 characters, with no control characters but a newline", () => {
		fc.assert(
			fc.property(fc.jsonValue(), fc.string({ maxLength: 5000 }), (anything, text) => {
				expect(() => validateReport(anything)).not.toThrow();
				const cleaned = cleanText(text);
				if (cleaned !== null) {
					expect(cleaned).toBe(cleaned.normalize("NFC"));
					expect([...cleaned].length).toBeGreaterThanOrEqual(10);
					expect([...cleaned].length).toBeLessThanOrEqual(4000);
					expect(/\p{Cc}/u.test(cleaned.replaceAll("\n", ""))).toBe(false);
				}
			}),
			runs(300),
		);
	});

	it("a rejected report names a field and nothing of its value", () => {
		fc.assert(
			fc.property(genReport, fc.constantFrom("kind", "text", "id", "v", "credit", "app", "screenshot"), fc.string({ minLength: 3, maxLength: 20 }), (r, key, junk) => {
				const mutated: Record<string, unknown> = { ...r, [key]: { junk: `ZZ${junk}ZZ` } };
				const out = validateReport(mutated);
				if (!out.ok) expect(out.field).not.toContain("ZZ");
			}),
			runs(100),
		);
	});

	it("preview is a single line of at most 80 characters", () => {
		fc.assert(
			fc.property(fc.string({ maxLength: 400 }), (t) => {
				const p = preview(t);
				expect(p.includes("\n")).toBe(false);
				expect([...p].length).toBeLessThanOrEqual(80);
			}),
			runs(200),
		);
	});

	it("imageMatches accepts only the declared type's magic bytes", () => {
		fc.assert(
			fc.property(fc.uint8Array({ maxLength: 40 }), (bytes) => {
				if (imageMatches("image/png", bytes)) expect(bytes[0]).toBe(0x89);
				if (imageMatches("image/jpeg", bytes)) expect([bytes[0], bytes[1], bytes[2]]).toEqual([0xff, 0xd8, 0xff]);
				expect(imageMatches("image/gif", bytes)).toBe(false);
			}),
			runs(200),
		);
	});

	it("reports from random anonymous keys land under distinct reporter hashes, each stable", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uniqueArray(fc.constant(0).map(() => newKey()), { minLength: 2, maxLength: 3 }), async (keys) => {
				await env.DB.batch([env.DB.prepare("DELETE FROM feedback_events"), env.DB.prepare("DELETE FROM feedback"), env.DB.prepare("DELETE FROM limits")]);
				for (const key of keys) await post(report(), { key, ip: `203.0.113.${Math.floor(Math.random() * 200)}` });
				const hashes = (await rows()).map((x: any) => x.reporter);
				expect(new Set(hashes).size).toBe(keys.length);
			}),
			{ numRuns: 8 },
		);
	});
});

afterEach(() => {
	clock.now = START;
});

void APP;
