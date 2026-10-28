// Migrations 0002_accounts.sql and 0011_push.sql (docs/ACCOUNTS.md › Storage): 0002 applies through
// applyD1Migrations on top of a v1 database, leaves the legacy rows alone, seeds
// sign-ups closed, and its triggers fail closed. The step-1 minimum; the Worker
// builder extends it.

import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { SEEDED_SETTINGS } from "./helpers";

const db = env.MIGRATION_DB;
const v1 = () => env.TEST_MIGRATIONS.filter((m) => m.name.startsWith("0001"));

/** Drops every table and view, d1_migrations included, so 0001 can apply again. */
async function emptyDatabase(): Promise<void> {
	const { results } = await db
		.prepare("SELECT type, name FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%'")
		.all<{ type: string; name: string }>();
	for (const { type, name } of results.filter((r) => r.type === "view")) await db.exec(`DROP VIEW "${name}"`);
	// Children before parents, so foreign keys never point at a dropped table.
	const order = [
		"collab_notifications",
		"collab_wraps",
		"collab_lists",
		"collab_requests",
		"collab_links",
		"collab_members",
		"collab_shares",
		"identities",
		"sessions",
		"account_docs",
		"account_files",
		"shares",
	];
	const tables = results.filter((r) => r.type === "table").map((r) => r.name);
	const rank = (name: string) => (order.includes(name) ? order.indexOf(name) : order.length);
	tables.sort((a, b) => rank(a) - rank(b));
	for (const name of tables) await db.exec(`DROP TABLE "${name}"`);
}

async function legacyState(): Promise<unknown> {
	const docs = await db.prepare("SELECT id, rev, seq, deleted, fields, updated_at FROM docs ORDER BY id").all();
	const counter = await db.prepare("SELECT name, value FROM counter ORDER BY name").all();
	return { docs: docs.results, counter: counter.results };
}

async function refusal(sql: string, ...params: unknown[]): Promise<string> {
	try {
		await db.prepare(sql).bind(...params).run();
	} catch (e) {
		return String((e as Error).message);
	}
	return "accepted";
}

const genLegacyDoc = fc.record({
	id: fc.stringMatching(/^(note|book|tag|file):[A-Za-z0-9]{1,10}$/),
	rev: fc.stringMatching(/^[1-9]-[0-9a-f]{32}$/),
	deleted: fc.boolean(),
	fields: fc.json(),
	updated_at: fc.option(fc.integer({ min: 0, max: 2 ** 41 }), { nil: null }),
});

describe("0002_accounts.sql", () => {
	it("applies after 0001 and seeds sign-ups closed", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		const settings = await db.prepare("SELECT name, value FROM settings").all<{ name: string; value: string }>();
		expect(Object.fromEntries(settings.results.map((r) => [r.name, r.value]))).toMatchObject({ signup: "closed", max_accounts: "50" });
		expect(Object.fromEntries(settings.results.map((r) => [r.name, r.value]))).toEqual(SEEDED_SETTINGS);
		const daily = await db.prepare("SELECT name, day, count FROM daily ORDER BY name").all();
		expect(daily.results).toEqual([
			{ name: "signups", day: 0, count: 0 },
			{ name: "upload_bytes", day: 0, count: 0 },
			{ name: "writes", day: 0, count: 0 },
		]);
	});

	it("fails closed: sign-ups refused while closed, settings rows can't be deleted, a missing setting refuses", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		const account = [
			`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
			   recovery_wrapped_key, key_id, epoch, created_at)
			 VALUES (?1, ?2, 1048576, 3, 4, ?3, ?4, ?5, ?4, ?5, '0123456789abcdef', '0123456789abcdef', 1800057600000)`,
			`a_${"0".repeat(32)}`,
			"alice",
			"A".repeat(22),
			"0".repeat(64),
			"A".repeat(98),
		] as const;
		expect(await refusal(...account)).toContain("signup_closed:closed");
		expect(await refusal("DELETE FROM settings WHERE name = 'max_accounts'")).toContain("settings_required");
		expect(await refusal("DELETE FROM daily WHERE name = 'signups'")).toContain("daily_required");
		await db.exec("UPDATE settings SET value = 'open' WHERE name = 'signup'");
		// Drop the guard to show what a lost row would do: max_accounts reads as 0.
		await db.exec("DROP TRIGGER settings_keep");
		await db.exec("DELETE FROM settings WHERE name = 'max_accounts'");
		expect(await refusal(...account)).toContain("signup_closed:full");
	});

	// Invariant: whatever a v1 database holds, applying 0002 leaves its docs and
	// counter rows byte for byte as they were.
	it("prop_0002_leaves_legacy_rows_untouched", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.uniqueArray(genLegacyDoc, { selector: (d) => d.id, maxLength: 12 }),
				fc.nat({ max: 10_000 }),
				async (docs, extra) => {
					await emptyDatabase();
					await applyD1Migrations(db, v1());
					const insert = db.prepare("INSERT INTO docs (id, rev, seq, deleted, fields, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)");
					const statements = docs.map((d, i) => insert.bind(d.id, d.rev, i + 1, d.deleted ? 1 : 0, JSON.stringify(d.fields), d.updated_at));
					statements.push(db.prepare("UPDATE counter SET value = ?1 WHERE name = 'seq'").bind(docs.length + extra));
					await db.batch(statements);
					const before = await legacyState();
					await applyD1Migrations(db, env.TEST_MIGRATIONS);
					expect(await legacyState()).toEqual(before);
				},
			),
			{ numRuns: 25 },
		);
	});
});

// ---------------------------------------------------------------------- 0011

const ACCOUNT_ID = `a_${"0".repeat(32)}`;
const device = (n: number) => `d_${n.toString(16).padStart(32, "0")}`;
const hash = (n: number) => n.toString(16).padStart(64, "0");
const apnsToken = (n: number) => n.toString(16).padStart(64, "0");
const fcmToken = (n: number) => `${n}`.padEnd(120, "A");

async function accountWithSession(): Promise<void> {
	await db.exec("UPDATE settings SET value = 'open' WHERE name = 'signup'");
	await db
		.prepare(
			`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
			   recovery_wrapped_key, key_id, epoch, created_at)
			 VALUES (?1, 'alice', 1048576, 3, 4, ?2, ?3, ?4, ?3, ?4, '0123456789abcdef', '0123456789abcdef', 1800057600000)`,
		)
		.bind(ACCOUNT_ID, "A".repeat(22), "0".repeat(64), "A".repeat(98))
		.run();
}

const addSession = (n: number) =>
	db
		.prepare("INSERT INTO sessions (account_id, device_id, token_hash, proof_hash, created_at, last_seen_at) VALUES (?1, ?2, ?3, ?3, 1, 1)")
		.bind(ACCOUNT_ID, device(n), hash(n))
		.run();

const setPush = (n: number, platform: string | null, token: string | null, environment: string | null, updated: number | null = token === null ? null : 5) =>
	refusal(
		"UPDATE sessions SET push_platform = ?2, push_token = ?3, push_environment = ?4, push_updated_at = ?5 WHERE device_id = ?1",
		device(n),
		platform,
		token,
		environment,
		updated,
	);

const pushRow = (n: number) =>
	db
		.prepare("SELECT push_platform, push_token, push_environment, push_updated_at, push_last_sent_at, push_due_at FROM sessions WHERE device_id = ?1")
		.bind(device(n))
		.first<Record<string, unknown>>();

describe("0011_push.sql", () => {
	it("adds nullable columns and leaves existing sessions as they were", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS.filter((m) => m.name < "0011"));
		await accountWithSession();
		await addSession(1);
		const before = await db.prepare("SELECT * FROM sessions").all();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		const after = await db.prepare("SELECT * FROM sessions").all<Record<string, unknown>>();
		expect(after.results).toEqual([
			{ ...before.results[0], push_platform: null, push_token: null, push_environment: null, push_updated_at: null, push_last_sent_at: null, push_due_at: null },
		]);
	});

	it("checks the shape of what is stored: platform, token, environment, and that they come together", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await accountWithSession();
		for (const n of [1, 2]) await addSession(n);
		const bad = async (platform: string | null, token: string | null, environment: string | null, updated?: number | null) =>
			expect(await setPush(1, platform, token, environment, updated), `${platform} ${token?.length} ${environment}`).toContain("CHECK constraint failed");
		await bad("web", apnsToken(1), "production");
		await bad("apns", apnsToken(1), "staging");
		await bad("apns", apnsToken(1).slice(1), "production");
		await bad("apns", apnsToken(1).slice(0, 30), "production");
		await bad("apns", `${apnsToken(1)}${apnsToken(1)}${apnsToken(1)}${apnsToken(1).slice(0, 10)}`, "production");
		await bad("apns", apnsToken(1).toUpperCase().replace(/^0+/, "AB"), "production");
		await bad("apns", `${apnsToken(1).slice(2)}zz`, "production");
		await bad("fcm", fcmToken(1).slice(0, 99), "production");
		await bad("fcm", `${fcmToken(1)}!`, "production");
		await bad("fcm", "A".repeat(4097), "production");
		await bad("fcm", fcmToken(1), "development");
		await bad("fcm", apnsToken(1), null);
		await bad("apns", null, "production");
		await bad(null, apnsToken(1), "production");
		await bad("apns", apnsToken(1), null);
		await bad(null, null, "production");
		await bad("apns", apnsToken(1), "production", null);
		await bad(null, null, null, 5);
		expect(await refusal("UPDATE sessions SET push_due_at = 5 WHERE device_id = ?1", device(1))).toContain("CHECK constraint failed");
		expect(await pushRow(1)).toMatchObject({ push_platform: null, push_token: null, push_due_at: null });
		for (const [platform, token, environment] of [
			["apns", apnsToken(1), "production"],
			["apns", apnsToken(1).slice(0, 32), "development"],
			["apns", `${apnsToken(1)}${apnsToken(2)}${apnsToken(3)}`.slice(0, 200), "production"],
			["fcm", fcmToken(1), "production"],
			["fcm", "A".repeat(4096), "production"],
		] as const) {
			expect(await setPush(1, platform, token, environment)).toBe("accepted");
		}
		expect(await refusal("UPDATE sessions SET push_due_at = 5 WHERE device_id = ?1", device(1))).toBe("accepted");
	});

	it("a token is on one session at most", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await accountWithSession();
		for (const n of [1, 2, 3]) await addSession(n);
		expect(await setPush(1, "apns", apnsToken(7), "production")).toBe("accepted");
		expect(await setPush(2, "apns", apnsToken(7), "production")).toContain("UNIQUE constraint failed");
		expect(await setPush(2, "apns", apnsToken(8), "production")).toBe("accepted");
		// Rows without a token don't collide.
		expect(await setPush(3, null, null, null)).toBe("accepted");
		expect(await refusal("UPDATE sessions SET push_platform = NULL, push_token = NULL, push_environment = NULL, push_updated_at = NULL WHERE device_id = ?1", device(1))).toBe("accepted");
		expect(await setPush(2, "apns", apnsToken(7), "production")).toBe("accepted");
	});

	it("the token is cleared when the session is revoked or its token hash changes, and not otherwise; the last send stays", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await accountWithSession();
		for (const n of [1, 2, 3, 4]) {
			await addSession(n);
			expect(await setPush(n, "apns", apnsToken(n), "production")).toBe("accepted");
			await db.prepare("UPDATE sessions SET push_last_sent_at = 99, push_due_at = 100 WHERE device_id = ?1").bind(device(n)).run();
		}
		const cleared = { push_platform: null, push_token: null, push_environment: null, push_updated_at: null, push_last_sent_at: 99, push_due_at: null };
		// Unrelated updates keep it.
		await db.exec("UPDATE sessions SET sealed_name = '" + "A".repeat(60) + "', last_seen_at = 9");
		expect((await pushRow(1))?.push_token).toBe(apnsToken(1));
		// Revoked.
		await db.prepare("UPDATE sessions SET revoked_at = 5, revoked_reason = 'signed_out' WHERE device_id = ?1").bind(device(1)).run();
		expect(await pushRow(1)).toEqual(cleared);
		// A new token hash (the device signed in again).
		await db.prepare("UPDATE sessions SET token_hash = ?2 WHERE device_id = ?1").bind(device(2), hash(200)).run();
		expect(await pushRow(2)).toEqual(cleared);
		// The sign-in upsert: new hash and the revoked marks removed at once.
		await db.prepare("UPDATE sessions SET revoked_at = 5, revoked_reason = 'revoked' WHERE device_id = ?1").bind(device(3)).run();
		await setPush(3, "apns", apnsToken(3), "production").catch(() => {});
		await db.prepare("UPDATE sessions SET token_hash = ?2, revoked_at = NULL, revoked_reason = NULL WHERE device_id = ?1").bind(device(3), hash(300)).run();
		expect((await pushRow(3))?.push_token).toBeNull();
		// A bulk revoke clears every row it touches, and only those.
		await db.prepare("UPDATE sessions SET revoked_at = 6, revoked_reason = 'password_changed' WHERE device_id = ?1").bind(device(4)).run();
		expect(await pushRow(4)).toEqual(cleared);
	});

	// Invariant: after any sequence of registrations, revocations and re-sign-ins, a revoked
	// session never holds a token and no token is held twice.
	it("prop_0011_revoked_sessions_hold_no_token", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await accountWithSession();
		for (const n of [1, 2, 3]) await addSession(n);
		let hashes = 1000;
		const op = fc.record({ who: fc.integer({ min: 1, max: 3 }), kind: fc.constantFrom("set", "revoke", "signin", "clear"), token: fc.integer({ min: 1, max: 3 }) });
		await fc.assert(
			fc.asyncProperty(fc.array(op, { minLength: 1, maxLength: 12 }), async (ops) => {
				for (const o of ops) {
					const d = device(o.who);
					if (o.kind === "set") {
						await db.batch([
							db.prepare("UPDATE sessions SET push_platform = NULL, push_token = NULL, push_environment = NULL, push_updated_at = NULL, push_due_at = NULL WHERE push_token = ?1").bind(apnsToken(o.token)),
							db.prepare("UPDATE sessions SET push_platform = 'apns', push_token = ?2, push_environment = 'production', push_updated_at = 1 WHERE device_id = ?1 AND revoked_at IS NULL").bind(d, apnsToken(o.token)),
						]);
					} else if (o.kind === "revoke") {
						await db.prepare("UPDATE sessions SET revoked_at = coalesce(revoked_at, 7), revoked_reason = coalesce(revoked_reason, 'revoked') WHERE device_id = ?1").bind(d).run();
					} else if (o.kind === "signin") {
						hashes += 1;
						await db.prepare("UPDATE sessions SET token_hash = ?2, revoked_at = NULL, revoked_reason = NULL WHERE device_id = ?1").bind(d, hash(hashes)).run();
					} else {
						await db.prepare("UPDATE sessions SET push_platform = NULL, push_token = NULL, push_environment = NULL, push_updated_at = NULL, push_due_at = NULL WHERE device_id = ?1").bind(d).run();
					}
				}
				const rows = await db.prepare("SELECT push_token, revoked_at FROM sessions").all<{ push_token: string | null; revoked_at: number | null }>();
				for (const r of rows.results) if (r.revoked_at !== null) expect(r.push_token).toBeNull();
				const held = rows.results.filter((r) => r.push_token !== null).map((r) => r.push_token);
				expect(new Set(held).size).toBe(held.length);
			}),
			{ numRuns: 25 },
		);
	});
});

// ---------------------------------------------------------------------- 0013

describe("0013_sealed_file_size.sql", () => {
	it("widens the stored-file limit without losing existing rows or quota triggers", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS.filter((m) => m.name < "0013"));
		await accountWithSession();
		const fileId = `file:${"A".repeat(22)}`;
		await db
			.prepare("INSERT INTO account_files (account_id, id, size, uploaded_at) VALUES (?1, ?2, 41, 1)")
			.bind(ACCOUNT_ID, fileId)
			.run();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		expect(await db.prepare("SELECT size FROM account_files WHERE id = ?1").bind(fileId).first()).toEqual({ size: 41 });
		expect(
			(
				await db
					.prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name IN ('uploads_global', 'uploads_count_insert', 'uploads_count_update') ORDER BY name")
					.all()
			).results,
		).toEqual([{ name: "uploads_count_insert" }, { name: "uploads_count_update" }, { name: "uploads_global" }]);
		const add = (letter: string, size: number) =>
			refusal(
				"INSERT INTO account_files (account_id, id, size, uploaded_at) VALUES (?1, ?2, ?3, 2)",
				ACCOUNT_ID,
				`file:${letter.repeat(22)}`,
				size,
			);
		expect(await add("B", 10_489_897)).toBe("accepted");
		expect(await add("C", 10_489_898)).toContain("CHECK constraint failed");
	});
});
