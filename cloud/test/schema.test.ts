// The schema check (src/schema.ts, #366): the Worker's list of migrations matches
// cloud/migrations, a fully migrated database reports "current", and a database
// missing a migration reports "behind" on /v2/health and files one issue from the cron.

import { env } from "cloudflare:workers";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EXPECTED_MIGRATIONS, SCHEMA_ISSUE_TITLE, checkSchema, pendingMigrations, schemaHint } from "../src/schema";
import { START, callJson, testEnv } from "./helpers";
import { REPO, configured, fakeNetwork } from "./feedback-helpers";

// no-prop-test: set difference over a fixed list; the cases below cover empty, one and all pending.

const LAST = EXPECTED_MIGRATIONS[EXPECTED_MIGRATIONS.length - 1];

async function forget(name: string): Promise<void> {
	await env.DB.prepare("DELETE FROM d1_migrations WHERE name = ?1").bind(name).run();
}

async function restore(name: string): Promise<void> {
	await env.DB.prepare("INSERT OR IGNORE INTO d1_migrations (name) VALUES (?1)").bind(name).run();
}

describe("schema check", () => {
	afterEach(async () => {
		await restore(LAST);
	});

	it("lists exactly the files in cloud/migrations, in order", () => {
		// TEST_MIGRATIONS is read from the folder by vitest.config.ts.
		expect([...EXPECTED_MIGRATIONS]).toEqual(env.TEST_MIGRATIONS.map((m) => m.name));
	});

	it("a fully migrated database is current", async () => {
		expect(await pendingMigrations(env)).toEqual([]);
		expect((await callJson("/v2/health")).body.schema).toBe("current");
	});

	it("a missing migration shows as behind on /v2/health", async () => {
		await forget(LAST);
		expect(await pendingMigrations(env)).toEqual([LAST]);
		expect((await callJson("/v2/health")).body.schema).toBe("behind");
	});

	it("the cron files one issue naming the missing migration, and not again while it is open", async () => {
		await forget(LAST);
		const net = fakeNetwork();
		try {
			const cfg = await configured();
			expect(await checkSchema(cfg, START)).toEqual([LAST]);
			const posts = net.sent.filter((s) => s.method === "POST" && s.url.endsWith(`/repos/${REPO}/issues`));
			expect(posts).toHaveLength(1);
			expect(posts[0].body.title).toBe(SCHEMA_ISSUE_TITLE);
			expect(posts[0].body.body).toContain(LAST);
			expect(posts[0].body.labels).toEqual(["bug"]);
			net.issues.push({ number: 301, title: SCHEMA_ISSUE_TITLE, labels: ["bug"] });
			await checkSchema(cfg, START);
			expect(net.sent.filter((s) => s.method === "POST" && s.url.endsWith(`/repos/${REPO}/issues`))).toHaveLength(1);
		} finally {
			net.restore();
		}
	});

	it("the cron files nothing when the database is current", async () => {
		const net = fakeNetwork();
		try {
			expect(await checkSchema(await configured(), START)).toEqual([]);
			expect(net.sent).toHaveLength(0);
		} finally {
			net.restore();
		}
	});
});

describe("schemaHint", () => {
	it("names a missing table or column and nothing else", () => {
		expect(schemaHint(new Error("D1_ERROR: no such table: device_pairings: SQLITE_ERROR"))).toBe("no such table: device_pairings");
		expect(schemaHint(new Error("table note_loops has no column named note_updated_at"))).toBe("has no column named: note_updated_at");
		expect(schemaHint(new Error("no such column: secret_tok-en.x"))).toBe("no such column: secret_tok");
		expect(schemaHint(new Error("boom tok_abc"))).toBeNull();
		expect(schemaHint("not an error")).toBeNull();
	});

	it("a 500 from a missing table logs which table, and not the token", async () => {
		const logged: string[] = [];
		const spy = vi.spyOn(console, "error").mockImplementation((...args: unknown[]) => {
			logged.push(args.map(String).join(" "));
		});
		const broken = testEnv({
			DB: {
				prepare() {
					throw new Error("D1_ERROR: no such table: device_pairings: SQLITE_ERROR tok_secret");
				},
				batch() {
					throw new Error("boom");
				},
			} as unknown as D1Database,
		});
		try {
			const r = await callJson("/v2/health", { env: broken });
			expect(r.status).toBe(500);
		} finally {
			spy.mockRestore();
		}
		expect(logged).toHaveLength(1);
		expect(JSON.parse(logged[0])).toMatchObject({ error: "internal", schema: "no such table: device_pairings" });
		expect(logged[0]).not.toContain("tok_secret");
	});
});
