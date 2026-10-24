// Rate limits (docs/ACCOUNTS.md › Rate limits). Two layers:
//
// 1. Workers rate-limit bindings, which write nothing to D1: AUTH_LIMITER,
//    keyed by the caller's IP group, in front of every /v2/auth/* and
//    /v2/health request; SESSION_LIMITER, keyed by account ID, on every
//    session route after the session lookup. Without a binding (unit tests)
//    its layer is skipped.
// 2. Fixed-window counters in D1, the authority. The counting upsert runs
//    before credentials are checked or any work is done, and the request is
//    refused when the count it returns is over the maximum, so parallel
//    requests can't pass together.
//
// Keys never hold an IP or a username: IP subjects go through an HMAC keyed
// by the pepper's `limit` subkey; account subjects use the account ID.

import { type PepperKeys, hex, hmac16 } from "./crypto";
import { rateLimited } from "./http";
import { DAY, HOUR } from "./protocol";

export type Rule =
	| "signup-ip"
	| "login-user"
	| "login-ip"
	| "recover-ip"
	| "passkey-challenge"
	| "passkey-manage"
	| "proof-account"
	| "password-account"
	| "writes-account"
	| "uploads-account"
	| "lookup-account"
	| "join-account"
	| "notify-account";

/**
 * Window and maximum of each rule. The two per-account daily rules read their
 * maximum from D1 when they count, in the same batch: `writes-account` from the
 * account's `quota_writes_per_day`, else `settings.writes_per_day` (20,000 by
 * default; the owner raises one account with `admin quota <name>
 * --writes-per-day N`); `uploads-account` from `settings.upload_bytes_per_day`
 * (1 GiB by default). Without the upload rule one account could spend the whole
 * server-wide `global_upload_bytes_per_day` alone: replacing one 10 MiB file
 * about 480 times costs only 480 writes.
 *
 * The collaboration rules (docs/ACCOUNTS.md › Collaboration) count per account:
 * `lookup-account` every public-key lookup by username, `join-account` every
 * request to join through a link (each one is a guess at a link's proof), and
 * `notify-account` every notification sent.
 */
export const RULES: Record<Rule, { window: number; max: number | null }> = {
	"signup-ip": { window: HOUR, max: 10 },
	"login-user": { window: 15 * 60_000, max: 10 },
	"login-ip": { window: 15 * 60_000, max: 30 },
	"recover-ip": { window: HOUR, max: 10 },
	"passkey-challenge": { window: 15 * 60_000, max: 30 },
	"passkey-manage": { window: HOUR, max: 20 },
	"proof-account": { window: HOUR, max: 20 },
	"password-account": { window: 15 * 60_000, max: 5 },
	"writes-account": { window: DAY, max: null },
	"uploads-account": { window: DAY, max: null },
	"lookup-account": { window: HOUR, max: 120 },
	"join-account": { window: HOUR, max: 20 },
	"notify-account": { window: DAY, max: 500 },
};

/**
 * The maximum of a per-account daily rule for the account `?1`, as one row
 * `{max}`. A missing setting reads as NULL, which refuses, like the triggers.
 */
export const DAILY_MAX_SQL: Partial<Record<Rule, string>> = {
	"writes-account": `SELECT coalesce((SELECT quota_writes_per_day FROM accounts WHERE id = ?1),
  (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'writes_per_day')) AS max`,
	"uploads-account": "SELECT (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'upload_bytes_per_day') AS max",
};

/** The binding's window, which is also its `retry_after`. */
export const BINDING_PERIOD_MS = 60_000;

/**
 * Counts first, in fixed windows. A request stamped with an earlier window
 * than the row's (one begun just before the boundary, or a Worker whose clock
 * is a little behind) counts against the newer window instead of resetting it,
 * as the `daily` triggers do; only a later window starts the count again.
 */
export const COUNT_SQL = `INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, ?3)
ON CONFLICT (key) DO UPDATE SET
  count = CASE WHEN window_start >= excluded.window_start THEN count + excluded.count ELSE excluded.count END,
  window_start = max(window_start, excluded.window_start)
RETURNING count`;

export const windowStart = (now: number, window: number): number => Math.floor(now / window) * window;

/** `rule:` and 16 bytes of HMAC(limit_key, rule || 0x00 || subject), as hex. */
export async function ipKey(keys: PepperKeys, rule: Rule, subject: string): Promise<string> {
	return `${rule}:${hex(await hmac16(keys.limit, rule, subject))}`;
}

/** Account subjects aren't secret: `rule:<account id>`, which the admin can read. */
export const accountKey = (rule: Rule, accountId: string): string => `${rule}:${accountId}`;

/** `login-user` for a name with no account: every such name shares this row. */
export const UNKNOWN_LOGIN_KEY = "login-user:unknown";

/**
 * Counts `amount` against `key` under `rule`; the 429 response when the
 * window's count is now over the rule's maximum, else null. The per-account
 * daily rules (`DAILY_MAX_SQL`) read their maximum in the same batch; their
 * keys are `rule:<account id>`.
 */
export async function count(db: D1Database, rule: Rule, key: string, now: number, amount = 1): Promise<Response | null> {
	const { window } = RULES[rule];
	const start = windowStart(now, window);
	const maxSql = DAILY_MAX_SQL[rule];
	if (maxSql !== undefined) {
		const accountId = key.slice(rule.length + 1);
		const [counted, setting] = await db.batch<{ count?: number; max?: number | null }>([
			db.prepare(COUNT_SQL).bind(key, start, amount),
			// D1 refuses a binding the statement has no parameter for.
			maxSql.includes("?1") ? db.prepare(maxSql).bind(accountId) : db.prepare(maxSql),
		]);
		const total = counted.results[0]?.count ?? Number.MAX_SAFE_INTEGER;
		// A missing setting refuses, like the triggers.
		const limit = setting.results[0]?.max ?? 0;
		return total > limit ? rateLimited(rule, start + window - now) : null;
	}
	const row = await db.prepare(COUNT_SQL).bind(key, start, amount).first<{ count: number }>();
	const limit = RULES[rule].max ?? 0;
	return (row?.count ?? Number.MAX_SAFE_INTEGER) > limit ? rateLimited(rule, start + window - now) : null;
}

/** Deletes every row whose window has ended (the cron); rows of unknown rules go after two days. */
export function expiredLimitsSql(): string {
	const rules = Object.entries(RULES).map(([rule, { window }]) => `(key GLOB '${rule}:*' AND window_start + ${window} <= ?1)`);
	return `DELETE FROM limits WHERE ${rules.join(" OR ")} OR window_start + ${2 * DAY} <= ?1`;
}

// ----------------------------------------------------------------- IP groups

/** The eight 16-bit groups of an IPv6 address, or null. */
function parseIpv6(text: string): number[] | null {
	const address = text.split("%")[0].toLowerCase();
	const halves = address.split("::");
	if (halves.length > 2) return null;
	const parse = (part: string): number[] | null => {
		if (part === "") return [];
		const groups: number[] = [];
		const pieces = part.split(":");
		for (const [i, piece] of pieces.entries()) {
			if (i === pieces.length - 1 && piece.includes(".")) {
				const v4 = parseIpv4(piece);
				if (!v4) return null;
				groups.push((v4[0] << 8) | v4[1], (v4[2] << 8) | v4[3]);
			} else if (/^[0-9a-f]{1,4}$/.test(piece)) {
				groups.push(parseInt(piece, 16));
			} else {
				return null;
			}
		}
		return groups;
	};
	const head = parse(halves[0]);
	const tail = halves.length === 2 ? parse(halves[1]) : [];
	if (!head || !tail) return null;
	if (halves.length === 1) return head.length === 8 ? head : null;
	const missing = 8 - head.length - tail.length;
	if (missing < 1) return null;
	return [...head, ...new Array<number>(missing).fill(0), ...tail];
}

function parseIpv4(text: string): number[] | null {
	const parts = text.split(".");
	if (parts.length !== 4 || parts.some((p) => !/^[0-9]{1,3}$/.test(p) || Number(p) > 255)) return null;
	return parts.map(Number);
}

/** IPv6 addresses are grouped by their first 48 bits. */
const IPV6_PREFIX = 48;

/**
 * The subject a rule counts by: IPv4 per address, IPv6 per /48, in D1 and in
 * the binding alike. A /48 is the smallest block a home or small office is
 * commonly given, so a finer group would let one subscriber rotate through
 * 256 /56s (security review 2, finding 9). `CF-Connecting-IP` supplies the
 * address; "unknown" stands in when it's absent (local dev).
 */
export function ipGroup(ip: string | null): string {
	if (!ip) return "unknown";
	if (!ip.includes(":")) return parseIpv4(ip) ? ip : ip.toLowerCase();
	const groups = parseIpv6(ip);
	if (!groups) return ip.toLowerCase();
	// An IPv4-mapped address (::ffff:a.b.c.d) is that IPv4 address.
	if (groups.slice(0, 5).every((g) => g === 0) && groups[5] === 0xffff) {
		return [groups[6] >> 8, groups[6] & 255, groups[7] >> 8, groups[7] & 255].join(".");
	}
	const masked = groups.map((g, i) => {
		const bits = Math.max(0, Math.min(16, IPV6_PREFIX - 16 * i));
		return bits === 0 ? 0 : g & ((0xffff << (16 - bits)) & 0xffff);
	});
	return `${masked.map((g) => g.toString(16)).join(":")}/${IPV6_PREFIX}`;
}

export const clientIp = (request: Request): string | null => request.headers.get("cf-connecting-ip");

/** The binding layer: 429 `binding` when AUTH_LIMITER refuses this IP group, else null. */
export async function bindingFilter(limiter: RateLimit | undefined, request: Request): Promise<Response | null> {
	if (!limiter) return null;
	const { success } = await limiter.limit({ key: ipGroup(clientIp(request)) });
	return success ? null : rateLimited("binding", BINDING_PERIOD_MS);
}

/**
 * The session layer: 429 `session-account` when SESSION_LIMITER (about 120
 * requests a minute) refuses this account, else null. It runs after the
 * session lookup, keyed by account ID, so one account can't keep the shared
 * database busy from any number of IPs. Write-free, like AUTH_LIMITER, and
 * skipped when the binding is missing (unit tests).
 */
export async function sessionFilter(limiter: RateLimit | undefined, accountId: string): Promise<Response | null> {
	if (!limiter) return null;
	const { success } = await limiter.limit({ key: accountId });
	return success ? null : rateLimited("session-account", BINDING_PERIOD_MS);
}
