// Purges of R2 prefixes and the cron's other jobs (docs/ACCOUNTS.md › R2
// layout, purges and the cron), shared notes' rooms among them.
//
// A purge round is one list of up to 1,000 keys and one delete. A purge row
// goes only when a list comes back empty and the row is more than an hour
// old, so an upload still in flight can't leave an orphan behind.
//
// Account IDs are chosen by the client, so a deleted account's ID can sign up
// again. Purging `accounts/<id>/` while an account owns it would delete that
// account's files, so sign-up clears the purge row for its prefix, and a
// round finding the account alive drops the row instead of listing; objects
// the old account left behind are then orphans, which reconciliation removes.

import { signOutSockets } from "./account_room";
import { runCollabPurges } from "./collab";
import { purgeCrashReports } from "./crash_reports";
import type { Env } from "./env";
import { housekeeping as feedbackHousekeeping } from "./feedback";
import { runTriage } from "./feedback_triage";
import { refusal } from "./http";
import { expiredLimitsSql } from "./limits";
import { DAY, FILE_ROW_COST, HOUR, MAX_SEALED_FILE, MIN_SEALED_FILE, REVOKED_KEEP_MS, SESSION_IDLE_MS } from "./protocol";
import { runShortLinkPurges } from "./rerout";
import { runServerStats } from "./server_stats";
import { MAX_SEALED_SHARE, MIN_SEALED_SHARE, checksumHex, expireShares, runSharePurges, unpublishStatements } from "./shares";

export const PURGE_LIST_LIMIT = 1000;
/** Rounds the cron spends on purges each run. */
export const CRON_PURGE_ROUNDS = 50;
/** Rounds an account deletion runs inline. */
export const DELETE_PURGE_ROUNDS = 20;
export const CRON_PERIOD_MS = 15 * 60_000;
/** Accounts whose prefixes one cron run reconciles, taken in turn. */
export const RECONCILE_ACCOUNTS_PER_RUN = 5;
/** R2 list calls one cron run may spend reconciling. */
export const RECONCILE_LIST_BUDGET = 50;

export const accountPrefix = (accountId: string): string => `accounts/${accountId}/`;
export const fileKey = (accountId: string, wireId: string): string => `${accountPrefix(accountId)}${wireId}`;

type RoundResult = "more" | "empty" | "done";

/** One purge round for `prefix`. */
export async function purgeRound(env: Env, prefix: string, requestedAt: number, now: number): Promise<RoundResult> {
	const owner = /^accounts\/(a_[0-9a-f]{32})\/$/.exec(prefix)?.[1];
	if (owner) {
		const alive = await env.DB.prepare("SELECT 1 AS alive FROM accounts WHERE id = ?1").bind(owner).first();
		if (alive) {
			await env.DB.prepare("DELETE FROM purges WHERE prefix = ?1").bind(prefix).run();
			return "done";
		}
	}
	const listed = await env.FILES.list({ prefix, limit: PURGE_LIST_LIMIT });
	if (listed.objects.length > 0) {
		await env.FILES.delete(listed.objects.map((o) => o.key));
		return "more";
	}
	if (requestedAt <= now - HOUR) {
		await env.DB.prepare("DELETE FROM purges WHERE prefix = ?1 AND requested_at <= ?2").bind(prefix, now - HOUR).run();
		return "done";
	}
	return "empty";
}

/** Up to `rounds` rounds on one prefix; how many it used. */
export async function purgePrefix(env: Env, prefix: string, requestedAt: number, now: number, rounds: number): Promise<number> {
	let used = 0;
	while (used < rounds) {
		used += 1;
		if ((await purgeRound(env, prefix, requestedAt, now)) !== "more") break;
	}
	return used;
}

/** Up to `budget` rounds over every pending purge, oldest first; the rounds used. */
export async function runPurges(env: Env, now: number, budget = CRON_PURGE_ROUNDS): Promise<number> {
	const { results } = await env.DB.prepare("SELECT prefix, requested_at FROM purges ORDER BY requested_at, prefix").all<{
		prefix: string;
		requested_at: number;
	}>();
	let used = 0;
	for (const row of results) {
		if (used >= budget) break;
		used += await purgePrefix(env, row.prefix, row.requested_at, now, budget - used);
	}
	return used;
}

// ------------------------------------------------------------ reconciliation

export interface ReconcileReport {
	accounts: string[];
	orphansDeleted: number;
	sizesFixed: number;
	rowsDropped: number;
	/** Objects larger than their row whose real size the quotas refused: object and row deleted. */
	oversizeDeleted: number;
	/** Shares whose row was moved to the object R2 holds after racing uploads. */
	sharesAdopted: number;
	/** Shares unpublished because the quotas refused the object R2 holds. */
	sharesDropped: number;
}

const ROWS_FOR_IDS = `SELECT id, size, uploaded_at FROM account_files
  WHERE account_id = ?1 AND id IN (SELECT value FROM json_each(?2))`;

// Sets a row's size to its object's. No recounted_at: a correction that grows
// file_bytes goes through the quota triggers like an upload, because two
// racing uploads of one ID can leave a small row over a large object, and an
// exempt correction would let that object past the account's quota and the
// server's cap. A shrinking correction always passes.
const FIX_SIZE = [
	`UPDATE accounts SET file_bytes = file_bytes - ?3 + ?4
  WHERE id = ?1 AND EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3)`,
	"UPDATE account_files SET size = ?4 WHERE account_id = ?1 AND id = ?2 AND size = ?3",
];

const DROP_ROW = [
	`UPDATE accounts SET files = files - 1, file_bytes = file_bytes - (?3 + ${FILE_ROW_COST})
  WHERE id = ?1 AND EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3 AND uploaded_at <= ?4)`,
	"DELETE FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3 AND uploaded_at <= ?4",
];

/**
 * Reconciles one account's R2 prefix with its `account_files` rows: deletes
 * objects that have no row and are more than an hour old, sets a row's size
 * to its object's where they differ, and (when the whole prefix was listed)
 * drops rows over an hour old whose object is gone. Returns the list budget left.
 */
async function reconcileAccount(env: Env, accountId: string, now: number, budget: number, report: ReconcileReport): Promise<number> {
	const prefix = accountPrefix(accountId);
	const seen = new Set<string>();
	let cursor: string | undefined;
	let complete = false;
	while (budget > 0) {
		budget -= 1;
		const listed = await env.FILES.list({ prefix, limit: PURGE_LIST_LIMIT, cursor });
		const ids = listed.objects.map((o) => o.key.slice(prefix.length));
		const { results: rows } = await env.DB.prepare(ROWS_FOR_IDS)
			.bind(accountId, JSON.stringify(ids))
			.all<{ id: string; size: number; uploaded_at: number }>();
		const byId = new Map(rows.map((r) => [r.id, r]));
		const orphans: string[] = [];
		const shrinks: D1PreparedStatement[] = [];
		const growths: { id: string; key: string; size: number; objectSize: number }[] = [];
		for (const object of listed.objects) {
			const id = object.key.slice(prefix.length);
			seen.add(id);
			const row = byId.get(id);
			if (!row) {
				if (object.uploaded.getTime() <= now - HOUR) orphans.push(object.key);
			} else if (
				row.size !== object.size &&
				row.uploaded_at <= now - HOUR &&
				object.size >= MIN_SEALED_FILE &&
				object.size <= MAX_SEALED_FILE
			) {
				if (object.size < row.size) {
					shrinks.push(
						env.DB.prepare(FIX_SIZE[0]).bind(accountId, id, row.size, object.size),
						env.DB.prepare(FIX_SIZE[1]).bind(accountId, id, row.size, object.size),
					);
					report.sizesFixed += 1;
				} else {
					growths.push({ id, key: object.key, size: row.size, objectSize: object.size });
				}
			}
		}
		if (orphans.length > 0) {
			await env.FILES.delete(orphans);
			report.orphansDeleted += orphans.length;
		}
		if (shrinks.length > 0) await env.DB.batch(shrinks);
		// One batch per growth, so one refusal doesn't roll back the others.
		for (const g of growths) {
			try {
				await env.DB.batch([
					env.DB.prepare(FIX_SIZE[0]).bind(accountId, g.id, g.size, g.objectSize),
					env.DB.prepare(FIX_SIZE[1]).bind(accountId, g.id, g.size, g.objectSize),
				]);
				report.sizesFixed += 1;
			} catch (err) {
				if (refusal(err)?.kind !== "quota_exceeded") throw err;
				// The quotas refuse the object's real size: the row goes (refunded), if it still
				// holds the size listed here, and then the object.
				const [, dropped] = await env.DB.batch(DROP_ROW.map((sql) => env.DB.prepare(sql).bind(accountId, g.id, g.size, now - HOUR)));
				if ((dropped.meta.changes ?? 0) > 0) {
					seen.delete(g.id);
					report.oversizeDeleted += 1;
					try {
						await env.FILES.delete(g.key);
					} catch {
						// Without its row the object is an orphan: a later run deletes it.
					}
				}
			}
		}
		if (!listed.truncated) {
			complete = true;
			break;
		}
		cursor = listed.cursor;
	}
	if (complete) {
		const { results } = await env.DB.prepare("SELECT id, size FROM account_files WHERE account_id = ?1 AND uploaded_at <= ?2")
			.bind(accountId, now - HOUR)
			.all<{ id: string; size: number }>();
		const drops = results
			.filter((r) => !seen.has(r.id))
			.flatMap((r) => DROP_ROW.map((sql) => env.DB.prepare(sql).bind(accountId, r.id, r.size, now - HOUR)));
		if (drops.length > 0) {
			await env.DB.batch(drops);
			report.rowsDropped += drops.length / 2;
		}
	}
	return budget;
}

/** R2 list calls one cron run may spend on `shares/`. */
export const RECONCILE_SHARE_PAGES = 10;
/** The base64url alphabet in R2's (byte) order: each run starts at the next first character. */
const SHARE_ID_ORDER = "-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

/** Moves a share's row to the object R2 holds, when it still describes the object listed. */
const ADOPT_SHARE = [
	`UPDATE accounts SET file_bytes = file_bytes - ?3 + ?4
  WHERE id = ?1 AND ?6 IS NOT NULL AND EXISTS (SELECT 1 FROM shares WHERE id = ?2 AND account_id = ?1 AND size = ?3 AND sha256 = ?5)`,
	"UPDATE shares SET size = ?4, sha256 = ?6 WHERE id = ?2 AND account_id = ?1 AND size = ?3 AND sha256 = ?5",
];

/**
 * Reconciles `shares/` with the shares table, a slice per run (starting at a
 * different first character each run, within RECONCILE_SHARE_PAGES lists):
 * deletes objects more than an hour old that have no row (an upload that
 * raced an unpublish), and where a row over an hour old doesn't describe its
 * object (two racing uploads of one share left the other one in R2), moves
 * the row to the object: its size counts against the quotas as an upload
 * would, and a share the quotas refuse is unpublished. Until then the public
 * route serves nothing for that share.
 */
async function reconcileShares(env: Env, now: number, report: ReconcileReport): Promise<void> {
	const first = SHARE_ID_ORDER[Math.floor(now / CRON_PERIOD_MS) % SHARE_ID_ORDER.length];
	const prefix = "shares/";
	let cursor: string | undefined;
	for (let page = 0; page < RECONCILE_SHARE_PAGES; page++) {
		const listed = await env.FILES.list(cursor ? { prefix, cursor, limit: PURGE_LIST_LIMIT } : { prefix, startAfter: `${prefix}${first}`, limit: PURGE_LIST_LIMIT });
		const ids = listed.objects.map((o) => o.key.slice(prefix.length));
		const { results: rows } = await env.DB.prepare(
			"SELECT id, account_id, size, sha256, updated_at FROM shares WHERE id IN (SELECT value FROM json_each(?1))",
		)
			.bind(JSON.stringify(ids))
			.all<{ id: string; account_id: string; size: number; sha256: string; updated_at: number }>();
		const byId = new Map(rows.map((r) => [r.id, r]));
		const orphans: string[] = [];
		for (const object of listed.objects) {
			const row = byId.get(object.key.slice(prefix.length));
			if (!row) {
				if (object.uploaded.getTime() <= now - HOUR) orphans.push(object.key);
				continue;
			}
			if (row.updated_at > now - HOUR) continue;
			const listedSha = checksumHex(object);
			if (object.size === row.size && (listedSha === null || listedSha === row.sha256)) continue;
			const head = await env.FILES.head(object.key);
			const sha = head ? checksumHex(head) : null;
			if (!head || (head.size === row.size && sha === row.sha256)) continue;
			if (sha !== null && head.size >= MIN_SEALED_SHARE && head.size <= MAX_SEALED_SHARE) {
				try {
					await env.DB.batch(ADOPT_SHARE.map((sql) => env.DB.prepare(sql).bind(row.account_id, row.id, row.size, head.size, row.sha256, sha)));
					report.sharesAdopted += 1;
					continue;
				} catch (err) {
					if (refusal(err)?.kind !== "quota_exceeded") throw err;
				}
			}
			// The quotas refuse it, or it isn't a share R2 checked: unpublished (and retired).
			const done = await env.DB.batch(unpublishStatements(env.DB, row.account_id, row.id, now));
			if (done[2].results.length === 1) {
				report.sharesDropped += 1;
				orphans.push(object.key);
			}
		}
		if (orphans.length > 0) {
			await env.FILES.delete(orphans);
			report.orphansDeleted += orphans.length;
		}
		if (!listed.truncated) break;
		cursor = listed.cursor;
	}
}

/** Reconciles a bounded number of accounts' prefixes, taking accounts in turn from one run to the next, and a slice of `shares/`. */
export async function reconcile(env: Env, now: number): Promise<ReconcileReport> {
	const report: ReconcileReport = { accounts: [], orphansDeleted: 0, sizesFixed: 0, rowsDropped: 0, oversizeDeleted: 0, sharesAdopted: 0, sharesDropped: 0 };
	await reconcileShares(env, now, report);
	const { results } = await env.DB.prepare("SELECT id FROM accounts ORDER BY id").all<{ id: string }>();
	if (results.length === 0) return report;
	const count = Math.min(RECONCILE_ACCOUNTS_PER_RUN, results.length);
	const start = (Math.floor(now / CRON_PERIOD_MS) * RECONCILE_ACCOUNTS_PER_RUN) % results.length;
	let budget = RECONCILE_LIST_BUDGET;
	for (let i = 0; i < count && budget > 0; i++) {
		const { id } = results[(start + i) % results.length];
		report.accounts.push(id);
		budget = await reconcileAccount(env, id, now, budget, report);
	}
	return report;
}

// ---------------------------------------------------------------------- cron

export interface CronReport {
	purgeRounds: number;
	reconcile: ReconcileReport;
	/** Published notes whose expiry passed, unpublished by this run. */
	sharesExpired: number;
	/** Published notes' R2 objects deleted by this run. */
	sharesPurged: number;
	/** Short links of published notes deleted from Rerout by this run. */
	shortLinksPurged: number;
	/** Rooms of shared notes that stopped (or whose owner was deleted) purged by this run. */
	collabPurged: number;
	/** Accounts whose event sockets this run closed: admin disables and deletions, and sessions it expired. */
	socketsClosed: number;
}

/** The scheduled jobs, every 15 minutes. */
export async function runCron(env: Env, now: number): Promise<CronReport> {
	const purgeRounds = await runPurges(env, now);
	const sharesExpired = await expireShares(env, now);
	const sharesPurged = await runSharePurges(env, now);
	const shortLinksPurged = await runShortLinkPurges(env);
	const collabPurged = await runCollabPurges(env, now);
	await env.DB.prepare(expiredLimitsSql()).bind(now).run();
	await purgeCrashReports(env, now);
	try {
		await feedbackHousekeeping(env, now);
		await runTriage(env, now);
	} catch (err) {
		// Feedback is best effort: a failure here never stops the other jobs.
		console.error(JSON.stringify({ route: "feedback", outcome: "failed", kind: (err as Error)?.name ?? "error" }));
	}
	try {
		await runServerStats(env, now);
	} catch (err) {
		// The numbers are best effort: a failure here never stops the other jobs.
		console.error(JSON.stringify({ route: "server_stats", outcome: "failed", kind: (err as Error)?.name ?? "error" }));
	}
	const reconciled = await reconcile(env, now);
	const housekeeping = await env.DB.batch([
		// Sign-in ceremonies and approvals are dead a day after they expire; the
		// approval row itself is authoritative until then, so a late poll still
		// sees `expired` rather than a missing request.
		env.DB.prepare("DELETE FROM auth_flows WHERE expires_at < ?1 - ?2").bind(now, DAY),
		env.DB.prepare("DELETE FROM device_approvals WHERE expires_at < ?1 - ?2").bind(now, DAY),
		env.DB.prepare(
			"UPDATE sessions SET revoked_at = ?1, revoked_reason = 'expired' WHERE revoked_at IS NULL AND last_seen_at < ?1 - ?2 RETURNING account_id, device_id",
		).bind(now, SESSION_IDLE_MS),
		env.DB.prepare("DELETE FROM sessions WHERE revoked_at IS NOT NULL AND revoked_at < ?1 - ?2").bind(now, REVOKED_KEEP_MS),
	]);
	const expired = new Map<string, string[]>();
	for (const row of housekeeping[2].results as { account_id: string; device_id: string }[]) {
		expired.set(row.account_id, [...(expired.get(row.account_id) ?? []), row.device_id]);
	}
	for (const [accountId, devices] of expired) await signOutSockets(env, accountId, devices);
	const socketsClosed = expired.size + (await runSocketCloses(env));
	return { purgeRounds, reconcile: reconciled, sharesExpired, sharesPurged, shortLinksPurged, collabPurged, socketsClosed };
}

/** Accounts closed per cron run from `socket_closes`; the rest wait for the next. */
export const SOCKET_CLOSE_BATCH = 100;

/**
 * Drains `socket_closes` (migration 0009), the admin script's requests: closes
 * every event socket of each account, then deletes its row unless a newer
 * request replaced it meanwhile. A room that can't be reached is logged and
 * the row goes anyway: the account's sessions already fail on their own.
 */
export async function runSocketCloses(env: Env, limit = SOCKET_CLOSE_BATCH): Promise<number> {
	const { results } = await env.DB.prepare("SELECT account_id, requested_at FROM socket_closes ORDER BY requested_at, account_id LIMIT ?1")
		.bind(limit)
		.all<{ account_id: string; requested_at: number }>();
	for (const row of results) {
		await signOutSockets(env, row.account_id, null);
		await env.DB.prepare("DELETE FROM socket_closes WHERE account_id = ?1 AND requested_at <= ?2").bind(row.account_id, row.requested_at).run();
	}
	return results.length;
}
