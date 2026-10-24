// The Worker as a whole: the default export's fetch and scheduled handlers,
// health, body and parameter limits, the feed's page budget, file paths and
// headers, and what a 500 logs.

import { createExecutionContext, createScheduledController, waitOnExecutionContext } from "cloudflare:test";
import { env } from "cloudflare:workers";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import worker, { runCron } from "../src/index";
import { MAX_PUSH_BODY, MAX_PUSH_DOCS, MAX_SEALED, MAX_SEALED_FILE, MIN_SEALED_FILE } from "../src/protocol";
import { ORIGIN, type Account, type Client, START, call, callJson, client, r2Keys, resetServer, sealed, setSetting, signUp, testEnv, wireId, wireRev } from "./helpers";

const doc = (len = 64) => ({ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(len), base_rev: null });

let a: Account;
let alice: Client;

beforeEach(async () => {
	await resetServer();
	a = await signUp("alice");
	alice = client(a.token);
});

afterEach(() => vi.restoreAllMocks());

describe("the default export", () => {
	it("fetch answers with the real clock", async () => {
		const response = await worker.fetch(new Request(`${ORIGIN}/v2/health`), testEnv());
		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({
			ok: true,
			protocol: 2,
			signup: "open",
			// Nothing sign-in-related is configured in the test bindings.
			auth: { password: true, passkeys: null, providers: [] },
			// The live approval socket is always served.
			events: { account: "/v2/account/events" },
		});
		expect(response.headers.get("content-type")).toBe("application/json; charset=utf-8");
	});

	it("scheduled runs the cron: queued purges and expired limit rows go", async () => {
		await env.FILES.put("accounts/a_00000000000000000000000000000000/file:x", "orphan");
		await env.DB.batch([
			env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES ('accounts/a_00000000000000000000000000000000/', 0)"),
			env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES ('login-user:unknown', 0, 3)"),
		]);
		const ctx = createExecutionContext();
		worker.scheduled(createScheduledController({ scheduledTime: Date.now(), cron: "*/15 * * * *" }), testEnv(), ctx);
		await waitOnExecutionContext(ctx);
		expect(await r2Keys()).toEqual([]);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM purges").first("n")).toBe(0);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM limits WHERE window_start = 0").first("n")).toBe(0);
	});
});

describe("health", () => {
	it("says closed, open or full, and needs no session", async () => {
		const health = async () => (await callJson("/v2/health")).body.signup;
		expect(await health()).toBe("open");
		await setSetting("max_accounts", "1");
		expect(await health()).toBe("full");
		await setSetting("signup", "closed");
		expect(await health()).toBe("closed");
		await setSetting("signup", "only:bob");
		expect(await health()).toBe("closed");
	});
});

describe("push limits", () => {
	it("refuses more than 200 documents, bodies over 4 MiB and sealed text over 1,900,000 characters with 413", async () => {
		let r = await alice.push(Array.from({ length: MAX_PUSH_DOCS + 1 }, () => doc(55)));
		expect(r.status).toBe(413);
		expect(r.body.error).toBe("too_large");
		expect((await alice.push(Array.from({ length: MAX_PUSH_DOCS }, () => doc(55)))).status).toBe(200);
		r = await alice.push([doc(MAX_SEALED + 1)]);
		expect(r.status).toBe(413);
		r = await alice.push([doc(MAX_SEALED), doc(MAX_SEALED), doc(MAX_PUSH_BODY - 2 * MAX_SEALED)]);
		expect(r.status).toBe(413);
		expect((await alice.push([doc(MAX_SEALED)])).status).toBe(200);
	});

	it("refuses malformed documents with 400, counting one write for each refused body", async () => {
		const good = doc();
		const bad: unknown[] = [
			{ ...good, rev: "1-abc" },
			{ ...good, base_rev: "ABC" },
			{ ...good, id: "note:short" },
			{ ...good, id: `page:${"A".repeat(22)}` },
			{ ...good, deleted: 0 },
			{ ...good, sealed: sealed(54) },
			{ id: good.id, rev: good.rev, deleted: false, sealed: good.sealed },
			"nope",
		];
		for (const d of bad) {
			const r = await alice.push([d]);
			expect(r.status, JSON.stringify(d).slice(0, 80)).toBe(400);
		}
		expect((await alice.push({} as unknown as unknown[])).status).toBe(400);
		// A refused body isn't free: one write each (security review 2, finding 2).
		expect((await alice.account()).body.usage.writes_today).toBe(bad.length + 1);
		// Sealed text outside base64url passes validation, is counted by its documents, and is refused by the CHECK.
		const r = await alice.push([{ ...good, sealed: `${"A".repeat(60)}+/=` }]);
		expect(r.status).toBe(400);
		expect((await alice.account()).body.usage).toMatchObject({ docs: 0, writes_today: bad.length + 2 });
		expect((await alice.push([])).body).toEqual({ results: [] });
	});
});

describe("the feed", () => {
	it("validates since and limit", async () => {
		for (const query of ["since=-1", "since=1.5", "since=x", "limit=0", "limit=501", "limit=1e2", "since=", "limit= 5"]) {
			expect((await callJson(`/v2/changes?${query}`, { token: a.token })).status, query).toBe(400);
		}
		for (const query of ["", "since=0", "limit=500", "since=99&limit=1"]) {
			expect((await callJson(`/v2/changes?${query}`, { token: a.token })).status, query).toBe(200);
		}
	});

	it("fits pages in 8 MiB of sealed text, saying more", async () => {
		const docs = Array.from({ length: 5 }, () => doc(MAX_SEALED));
		for (let i = 0; i < docs.length; i += 2) expect((await alice.push(docs.slice(i, i + 2))).status).toBe(200);
		const first = (await alice.changes(0, 500)).body;
		expect(first.docs.map((d: { id: string }) => d.id)).toEqual(docs.slice(0, 4).map((d) => d.id));
		expect(first).toMatchObject({ last_seq: 4, more: true, head_seq: 5 });
		const second = (await alice.changes(4, 500)).body;
		expect(second.docs.map((d: { sealed: string }) => d.sealed)).toEqual([docs[4].sealed]);
		expect(second).toMatchObject({ last_seq: 5, more: false, head_seq: 5 });
	});
});

describe("files", () => {
	const id = `file:${"F".repeat(22)}`;

	it("checks the wire ID, the checksum header and the size", async () => {
		const put = (path: string, body: Uint8Array, sha?: string | null, type = "application/octet-stream") =>
			call(path, {
				method: "PUT",
				token: a.token,
				body,
				headers: { "content-type": type, ...(sha === null ? {} : { "x-kanso-sha256": sha ?? "0".repeat(64) }) },
			});
		for (const path of [`/v2/files/note:${"F".repeat(22)}`, "/v2/files/file:short", `/v2/files/file:${"F".repeat(21)}%2F`, "/v2/files/%E0%A4%A"]) {
			expect((await put(path, new Uint8Array(64))).status, path).toBe(400);
		}
		expect((await put(`/v2/files/${id}`, new Uint8Array(64), null)).status).toBe(400);
		expect((await put(`/v2/files/${id}`, new Uint8Array(64), "A".repeat(64))).status).toBe(400);
		const small = await alice.putFile(id, new Uint8Array(MIN_SEALED_FILE - 1));
		expect(small.status).toBe(400);
		const large = await alice.putFile(id, new Uint8Array(MAX_SEALED_FILE + 1), "0".repeat(64));
		expect(large.status).toBe(413);
		expect(large.body.error).toBe("too_large");
		expect((await alice.putFile(id, new Uint8Array(MAX_SEALED_FILE))).body).toEqual({ ok: true, size: MAX_SEALED_FILE });
		// A percent-encoded path names the same file.
		const got = await call(`/v2/files/file%3A${"F".repeat(22)}`, { token: a.token });
		expect(got.status).toBe(200);
		expect(got.headers.get("content-length")).toBe(String(MAX_SEALED_FILE));
		expect(await r2Keys()).toEqual([`accounts/${a.creds.accountId}/${id}`]);
	});

	it("serves bytes only with an account_files row, even when R2 holds an object", async () => {
		await env.FILES.put(`accounts/${a.creds.accountId}/${id}`, new Uint8Array(64));
		expect((await alice.getFile(id)).status).toBe(404);
		expect((await alice.getFile(wireId("file"))).status).toBe(404);
	});

	it("deletes a file's bytes and charges when its tombstone is accepted", async () => {
		const meta = { id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null };
		expect((await alice.push([meta])).status).toBe(200);
		expect((await alice.putFile(id, new Uint8Array(500))).status).toBe(200);
		// Deleted then revived in one push: the last accepted version is live, so the bytes stay.
		const gone = { ...meta, rev: wireRev(), deleted: true, base_rev: meta.rev };
		const back = { ...meta, rev: wireRev(), deleted: false, base_rev: gone.rev };
		expect((await alice.push([gone, back])).status).toBe(200);
		expect((await alice.getFile(id)).status).toBe(200);
		const tomb = { ...meta, rev: wireRev(), deleted: true, base_rev: back.rev };
		expect((await alice.push([tomb])).status).toBe(200);
		expect((await alice.getFile(id)).status).toBe(404);
		expect(await r2Keys()).toEqual([]);
		expect((await alice.account()).body.usage).toMatchObject({ files: 0, file_bytes: 0 });
	});
});

describe("R2 failures", () => {
	it("a tombstone still removes the file's row and charges when R2's delete fails; reconciliation removes the object", async () => {
		const id = wireId("file");
		const meta = { id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null };
		expect((await alice.push([meta])).status).toBe(200);
		expect((await alice.putFile(id, new Uint8Array(500))).status).toBe(200);
		const failing = {
			get: env.FILES.get.bind(env.FILES),
			head: env.FILES.head.bind(env.FILES),
			put: env.FILES.put.bind(env.FILES),
			list: env.FILES.list.bind(env.FILES),
			delete: async () => {
				throw new Error("R2 is down");
			},
		} as unknown as R2Bucket;
		const tomb = { ...meta, rev: wireRev(), deleted: true, base_rev: meta.rev };
		const r = await client(a.token, { env: testEnv({ FILES: failing }) }).push([tomb]);
		expect(r.status).toBe(200);
		expect(r.body.results).toEqual([{ id, status: "ok", seq: 2 }]);
		expect((await alice.account()).body.usage).toMatchObject({ files: 0, file_bytes: 0 });
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_files").first("n")).toBe(0);
		expect(await r2Keys()).toEqual([`accounts/${a.creds.accountId}/${id}`]);
		await runCron(testEnv(), START + 2 * 3_600_000);
		expect(await r2Keys()).toEqual([]);
	});
});

describe("errors", () => {
	it("a 500 logs the route and an error code, never the path, token or body", async () => {
		const logged: string[] = [];
		vi.spyOn(console, "error").mockImplementation((...args: unknown[]) => {
			logged.push(args.map(String).join(" "));
		});
		const broken = testEnv({
			DB: {
				prepare() {
					throw new Error(`boom ${a.token}`);
				},
				batch() {
					throw new Error("boom");
				},
			} as unknown as D1Database,
		});
		const r = await callJson(`/v2/files/${wireId("file")}`, { token: a.token, env: broken });
		expect(r.status).toBe(500);
		expect(r.body).toEqual({ error: "internal", message: expect.any(String) });
		expect(logged).toHaveLength(1);
		expect(JSON.parse(logged[0])).toEqual({ route: "files", error: "internal", kind: "Error" });
		expect(logged[0]).not.toContain(a.token);
		expect(logged[0]).not.toContain("file:");
	});

	it("answers JSON errors with a code and a message", async () => {
		const r = await callJson("/v2/nowhere");
		expect(r.body).toEqual({ error: "not_found", message: expect.any(String) });
		expect(r.headers.get("content-type")).toBe("application/json; charset=utf-8");
		expect((await callJson("/v2/account")).headers.get("www-authenticate")).toContain("Bearer");
	});

	it("the clock is the caller's: a session last seen in the future is still valid", async () => {
		expect((await client(a.token, { now: START - 60_000 }).account()).status).toBe(200);
	});
});
