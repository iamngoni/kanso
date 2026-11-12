// The D1 migrations this Worker was built against, and a check that the live
// database has them all. A Worker deployed ahead of its schema fails only on the
// routes that touch the missing tables, so the check runs on /v2/health and in
// the cron, which files an issue (#366: 0018 never reached production).

import * as github from "./github";
import type { Env } from "./env";

/** Every file in cloud/migrations, in order. test/schema.test.ts keeps this in step with the folder. */
export const EXPECTED_MIGRATIONS: readonly string[] = [
	"0001_init.sql",
	"0002_accounts.sql",
	"0003_shares.sql",
	"0004_short_links.sql",
	"0005_collab.sql",
	"0006_sign_in.sql",
	"0007_provider_completion.sql",
	"0008_apple_app_secret.sql",
	"0009_socket_closes.sql",
	"0010_end_flows_with_sessions.sql",
	"0011_push.sql",
	"0012_crash_reports.sql",
	"0013_sealed_file_size.sql",
	"0014_server_stats.sql",
	"0015_sessions_by_last_seen.sql",
	"0016_feedback.sql",
	"0017_pdf_file_size.sql",
	"0018_device_pairings.sql",
	"0019_book_shares.sql",
];

export const SCHEMA_ISSUE_TITLE = "Server database is missing migrations";

/**
 * "no such table: x" and the like, from a D1 error, or null. Only the schema
 * name is kept, so nothing from the request can reach a log through it.
 */
export function schemaHint(err: unknown): string | null {
	const message = String((err as Error)?.message ?? "");
	const match = /(no such (?:table|column)): ([A-Za-z_]\w{0,63})|(has no column named) ([A-Za-z_]\w{0,63})/.exec(message);
	if (!match) return null;
	return match[1] ? `${match[1]}: ${match[2]}` : `${match[3]}: ${match[4]}`;
}

/** Expected migrations the database has not recorded in `d1_migrations`, in order. */
export async function pendingMigrations(env: Env): Promise<string[]> {
	let applied: Set<string>;
	try {
		const rows = await env.DB.prepare("SELECT name FROM d1_migrations").all<{ name: string }>();
		applied = new Set(rows.results.map((r) => r.name));
	} catch {
		// No migrations table at all: nothing was ever applied with wrangler.
		applied = new Set();
	}
	return EXPECTED_MIGRATIONS.filter((name) => !applied.has(name));
}

/** Cron: log and file one issue (deduplicated by title) while migrations are pending. */
export async function checkSchema(env: Env, nowMs: number): Promise<string[]> {
	const pending = await pendingMigrations(env);
	if (pending.length === 0) return pending;
	console.error(JSON.stringify({ route: "schema", outcome: "behind", pending }));
	if (!github.configOf(env)) return pending;
	const open = await github.openIssues(env, nowMs);
	if (open.some((i) => i.title === SCHEMA_ISSUE_TITLE)) return pending;
	await github.createIssue(
		env,
		{
			title: SCHEMA_ISSUE_TITLE,
			body: [
				"The production D1 database has not applied these migrations, so routes that use them fail:",
				"",
				...pending.map((name) => `- \`${name}\``),
				"",
				"Apply them from `cloud/`: `npm run migrate:remote`. Close this issue afterwards; the cron opens a new one if the database falls behind again.",
			].join("\n"),
			labels: ["bug"],
		},
		nowMs,
	);
	return pending;
}
