// The admin commands (docs/ACCOUNTS.md › Admin script), planned here and run
// by scripts/admin.ts through `wrangler d1 execute` (plus `wrangler d1
// export`, `wrangler r2 object get`, `wrangler secret delete` and `wrangler d1
// time-travel restore` where a command needs them). There is no admin HTTP
// route.
//
// `wrangler d1 execute` takes SQL text but no bound parameters, so every
// value is validated before it reaches SQL and quoted when it does. Commands
// talk to the outside world only through `AdminIO`, which the tests implement
// over the test database. Node runs this file as it is (type stripping), so
// it imports nothing but ./usernames.ts and uses only erasable syntax.

import { normalizeUsername } from "./usernames.ts";

export const USAGE = `Usage: npm run -s admin -- [--local [--persist-to DIR]] <command>

  accounts                          Every account: usage against quotas, writes and uploads today,
                                    devices, last seen and status; the settings, global sums and
                                    purges.
  settings                          The settings.
  disable <username>                Refuse the account's sign-ins and sessions, and stop serving
                                    its published notes.
  enable <username>                 Undo disable.
  shares <username>                 The account's published notes: IDs, sizes and expiries.
  unpublish <share-id>              Take a published note down (the ID in its link, after /p/):
                                    the link stops working and the ID can't be used again.
  delete <username> --confirm <username>
                                    Delete the account's rows and queue its R2 purge.
  signups open | close | only <username>
                                    Open sign-ups, close them, or reserve them for one name.
                                    Run only immediately before that name signs up: anyone
                                    who guesses the name can take it first. If someone did,
                                    delete <username> --confirm <username>, run only again,
                                    and sign up at once.
  max-accounts <n> [--force]        At most n accounts.
  quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--writes-per-day N]
        [--force]
  quota <username> --default        Per-account quotas, or back to the defaults. --writes-per-day
                                    raises (or lowers) one account's daily write budget.
  default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--force]
                                    The default quotas.
  writes-per-day <n> [--force]      Documents pushed plus files uploaded, per account per UTC day
                                    (the default; quota --writes-per-day overrides it).
  upload-bytes-per-day <n> [--force]
                                    Uploaded file bytes, per account per UTC day.
  global-writes-per-day <n> [--force]
                                    Stored document writes per UTC day, all accounts together.
  global-upload-bytes-per-day <n> [--force]
                                    Uploaded file bytes per UTC day, all accounts together.
  recount [<username>]              Recompute usage counters from the tables.
  restore <time | bookmark> [--confirm <time | bookmark>]
                                    The only way to restore D1: Time Travel back to a Unix time
                                    in seconds, an RFC 3339 time or a bookmark, then at once a
                                    new epoch for every account (restored --all). Without
                                    --confirm, prints what it would run. Deployed database only.
  restored [<username> | --all]     A new epoch, so Macs re-merge; restore runs it for you.
  legacy-status                     The single-user tables and the files/ purge.
  legacy-export --dir DIR           manifest.json, legacy.sql and every live legacy file.
  purge-legacy --account <username> --verified R.json
                                    Drop the single-user tables, queue the files/ purge and
                                    delete SYNC_TOKEN, once R.json proves everything arrived.

Runs against the deployed database unless --local is given. max-accounts, quota and
default-quota refuse a change that would overcommit a global cap unless --force is given, and
the daily commands (and quota --writes-per-day) refuse one that would let a single account's
daily budget exceed the server-wide one. The server-wide daily caps are shared fairly: an
account within its fair share (the cap ÷ max_accounts) is never refused by them, so a day's
total stays under about twice each cap; the daily commands and max-accounts print the shares.`;

export class UsageError extends Error {}

/** One statement's rows, as `wrangler d1 execute --json` and D1's batch() both report them. */
export interface StatementResult {
	results: Record<string, unknown>[];
}

/** Everything a command does outside this file. */
export interface AdminIO {
	/** Whether the commands run against the local database (wrangler's --local). */
	local: boolean;
	/** Runs the statements in order, in one transaction. */
	sql(statements: string[]): Promise<StatementResult[]>;
	/** `wrangler d1 export` of these tables (schema and rows) into `file`. */
	exportTables(tables: string[], file: string): Promise<void>;
	/** Copies an R2 object of the files bucket into `file`; false when there is no such object. */
	r2Get(key: string, file: string): Promise<boolean>;
	makeDir(dir: string): Promise<void>;
	writeFile(file: string, content: string): Promise<void>;
	readFile(file: string): Promise<string>;
	/** `wrangler secret delete <name>`, with stdin attached so wrangler can ask. */
	deleteSecret(name: string): Promise<void>;
	/**
	 * `wrangler <restoreArgs(target)>`: D1 Time Travel on the deployed database.
	 * Answers the bookmark restored and the one before it (to undo).
	 */
	timeTravelRestore(target: RestoreTarget): Promise<{ bookmark: string; previousBookmark: string }>;
}

/** The deployed D1 database's name, as wrangler.jsonc gives it. */
export const D1_DATABASE = "kanso-sync";

/** A point to restore D1 to: a Unix time in seconds or an RFC 3339 time (`timestamp`), or a bookmark. */
export interface RestoreTarget {
	kind: "timestamp" | "bookmark";
	value: string;
}

/** wrangler's arguments for the restore; `--json` skips its prompt and answers the bookmarks. */
export const restoreArgs = (target: RestoreTarget): string[] => ["d1", "time-travel", "restore", D1_DATABASE, `--${target.kind}=${target.value}`, "--json"];

/** New epochs, so each Mac re-merges: every account's (`restored --all`, and `restore`'s second step), or those `where` picks. */
const newEpochs = (where = ""): string => `UPDATE accounts SET epoch = lower(hex(randomblob(8)))${where} RETURNING username`;

export interface Report {
	ok: boolean;
	/** For the terminal; never holds a key or a token. */
	message: string;
}

// -------------------------------------------------------------------- values

/** A SQL string literal. Callers validate the value first; this only doubles quotes. */
export function sqlText(value: string): string {
	return `'${value.replaceAll("'", "''")}'`;
}

const MIB = 1_048_576;
const DAY = 86_400_000;
const ROW_COST = 512;
const FILE_COST = 1024;
/** Settings hold at most 15 digits (the migration's CHECK). */
const MAX_SETTING = 999_999_999_999_999;

function username(value: string | undefined): string {
	const normalized = value === undefined ? null : normalizeUsername(value);
	if (normalized === null) {
		throw new UsageError(`"${value ?? ""}" is not a username: 3–32 letters, digits, - or _, starting and ending with a letter or digit.`);
	}
	return normalized;
}

function positive(value: string | undefined, what: string, scale = 1): number {
	if (value === undefined || !/^[0-9]{1,15}$/.test(value) || Number(value) === 0 || Number(value) * scale > MAX_SETTING) {
		throw new UsageError(`${what} must be a whole number from 1 to ${Math.floor(MAX_SETTING / scale)}.`);
	}
	return Number(value) * scale;
}

/** Positional arguments and flags; a flag in `valued` takes the next argument. */
function parse(args: string[], booleans: string[], valued: string[]): { positional: string[]; flags: Map<string, string | true> } {
	const positional: string[] = [];
	const flags = new Map<string, string | true>();
	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (arg.startsWith("--")) {
			if (flags.has(arg)) throw new UsageError(`${arg} is given twice.`);
			if (booleans.includes(arg)) flags.set(arg, true);
			else if (valued.includes(arg)) {
				const value = args[++i];
				if (value === undefined || value.startsWith("--")) throw new UsageError(`${arg} needs a value.`);
				flags.set(arg, value);
			} else throw new UsageError(`Unknown option ${arg}.\n\n${USAGE}`);
		} else {
			positional.push(arg);
		}
	}
	return { positional, flags };
}

function arity(positional: string[], min: number, max: number): void {
	if (positional.length < min || positional.length > max) throw new UsageError(USAGE);
}

const flag = (flags: Map<string, string | true>, name: string): string | undefined => {
	const value = flags.get(name);
	return typeof value === "string" ? value : undefined;
};

// ------------------------------------------------------------------- reports

const date = (ms: unknown): string => (typeof ms === "number" ? new Date(ms).toISOString().slice(0, 10) : "");
const time = (ms: unknown): string => (typeof ms === "number" ? new Date(ms).toISOString().slice(0, 16).replace("T", " ") : "never");

function size(bytes: unknown): string {
	const n = Number(bytes);
	if (n >= 10 * MIB) return `${Math.round(n / MIB)} MiB`;
	if (n >= 1024) return `${(n / 1024).toFixed(1)} KiB`;
	return `${n} B`;
}

export function table(rows: string[][]): string {
	const widths = rows[0].map((_, c) => Math.max(...rows.map((r) => [...r[c]].length)));
	return rows.map((r) => r.map((cell, c) => cell.padEnd(widths[c] + (cell.length - [...cell].length))).join("  ").trimEnd()).join("\n");
}

const ok = (message: string): Report => ({ ok: true, message });
const refused = (message: string): Report => ({ ok: false, message });

// ----------------------------------------------------------------- capacity

export interface Capacity {
	maxAccounts: number;
	defaultDocBytes: number;
	defaultFileBytes: number;
	globalDocBytes: number;
	globalFileBytes: number;
	accounts: { username: string; quotaDocBytes: number | null; quotaFileBytes: number | null }[];
}

/**
 * What the quotas promise: Σ effective quotas of the existing accounts plus
 * (max_accounts − accounts) × the default, for document and file bytes.
 */
export function committed(c: Capacity): { docBytes: number; fileBytes: number } {
	const open = Math.max(0, c.maxAccounts - c.accounts.length);
	const sum = (pick: (a: Capacity["accounts"][number]) => number) => c.accounts.reduce((total, a) => total + pick(a), 0);
	return {
		docBytes: sum((a) => a.quotaDocBytes ?? c.defaultDocBytes) + open * c.defaultDocBytes,
		fileBytes: sum((a) => a.quotaFileBytes ?? c.defaultFileBytes) + open * c.defaultFileBytes,
	};
}

/** The caps a capacity overcommits, in words; empty when it fits. */
export function overcommitted(c: Capacity): string[] {
	const { docBytes, fileBytes } = committed(c);
	const problems: string[] = [];
	if (docBytes > c.globalDocBytes) problems.push(`document quotas would promise ${docBytes} bytes, over global_doc_bytes ${c.globalDocBytes}`);
	if (fileBytes > c.globalFileBytes) problems.push(`file quotas would promise ${fileBytes} bytes, over global_file_bytes ${c.globalFileBytes}`);
	return problems;
}

const CAPACITY_SQL = [
	"SELECT name, value FROM settings",
	"SELECT username, quota_doc_bytes, quota_file_bytes FROM accounts ORDER BY username",
];

async function capacity(io: AdminIO): Promise<{ capacity: Capacity; settings: Map<string, string> }> {
	const [settingsRows, accountRows] = await io.sql(CAPACITY_SQL);
	const settings = new Map(settingsRows.results.map((r) => [String(r.name), String(r.value)]));
	// A missing setting reads as 0, as the triggers read it.
	const n = (name: string) => Number(settings.get(name) ?? 0);
	return {
		settings,
		capacity: {
			maxAccounts: n("max_accounts"),
			defaultDocBytes: n("quota_doc_bytes"),
			defaultFileBytes: n("quota_file_bytes"),
			globalDocBytes: n("global_doc_bytes"),
			globalFileBytes: n("global_file_bytes"),
			accounts: accountRows.results.map((r) => ({
				username: String(r.username),
				quotaDocBytes: r.quota_doc_bytes === null ? null : Number(r.quota_doc_bytes),
				quotaFileBytes: r.quota_file_bytes === null ? null : Number(r.quota_file_bytes),
			})),
		},
	};
}

/** Refuses (unless forced) a change after which the quotas would promise more than a global cap. */
function overcommitRefusal(after: Capacity, force: boolean): Report | null {
	const problems = overcommitted(after);
	if (problems.length === 0 || force) return null;
	return refused(`Refused: ${problems.join("; ")}. Raise the global cap first, or repeat with --force.`);
}

// -------------------------------------------------------------- daily caps

/** The per-account and server-wide daily budgets, as the settings and overrides hold them. */
export interface DailyCaps {
	/** The fair share of a server-wide cap is the cap ÷ max_accounts. */
	maxAccounts: number;
	writesPerDay: number;
	uploadBytesPerDay: number;
	globalWritesPerDay: number;
	globalUploadBytesPerDay: number;
	/** Accounts with their own `quota_writes_per_day`. */
	writeOverrides: { username: string; writesPerDay: number }[];
}

/**
 * The daily budgets overcommit when one account alone may spend more than
 * the whole server's day: its write budget (the default or its own) over
 * `global_writes_per_day`, or `upload_bytes_per_day` over
 * `global_upload_bytes_per_day`. In words; empty when it fits.
 */
export function dailyOvercommitted(c: DailyCaps): string[] {
	const problems: string[] = [];
	if (c.writesPerDay > c.globalWritesPerDay) {
		problems.push(`writes_per_day ${c.writesPerDay} is over global_writes_per_day ${c.globalWritesPerDay}`);
	}
	for (const o of c.writeOverrides.filter((o) => o.writesPerDay > c.globalWritesPerDay)) {
		problems.push(`${o.username}'s writes per day ${o.writesPerDay} is over global_writes_per_day ${c.globalWritesPerDay}`);
	}
	if (c.uploadBytesPerDay > c.globalUploadBytesPerDay) {
		problems.push(`upload_bytes_per_day ${c.uploadBytesPerDay} is over global_upload_bytes_per_day ${c.globalUploadBytesPerDay}`);
	}
	return problems;
}

/** A server-wide cap's fair share per account: cap ÷ max_accounts, rounded down, as the triggers compute it; 0 without accounts. */
export function fairShare(global: number, maxAccounts: number): number {
	return maxAccounts > 0 ? Math.floor(global / maxAccounts) : 0;
}

/**
 * The daily arithmetic, in words: each server-wide cap's fair share per
 * account (an account at or below it is never refused by that cap), how many
 * accounts at their whole default budget reach the cap (after which only
 * accounts above their fair share wait for UTC midnight), and the day's
 * bound of about twice the cap.
 */
export function dailyShares(c: DailyCaps): string {
	const share = (perAccount: number, global: number, what: string, globalName: string, unit: string) => {
		const fair = fairShare(global, c.maxAccounts);
		let reach = "";
		if (perAccount > 0) {
			const n = Math.ceil(global / perAccount);
			reach = ` ${n === 1 ? "1 account using its" : `${n} accounts using their`} whole ${what} (${perAccount}) reach${n === 1 ? "es" : ""} it, after which only accounts above their fair share wait for UTC midnight.`;
		}
		return ` ${globalName} (${global}) ÷ max_accounts (${c.maxAccounts}): a fair share of ${fair} ${unit} per account, which it never refuses.${reach} A day's total stays under about ${2 * global}.`;
	};
	return (
		share(c.writesPerDay, c.globalWritesPerDay, "writes_per_day", "global_writes_per_day", "writes") +
		share(c.uploadBytesPerDay, c.globalUploadBytesPerDay, "upload_bytes_per_day", "global_upload_bytes_per_day", "bytes")
	);
}

const DAILY_SQL = [
	"SELECT name, value FROM settings WHERE name IN ('max_accounts', 'writes_per_day', 'upload_bytes_per_day', 'global_writes_per_day', 'global_upload_bytes_per_day')",
	"SELECT username, quota_writes_per_day FROM accounts WHERE quota_writes_per_day IS NOT NULL ORDER BY username",
];

async function dailyCaps(io: AdminIO): Promise<DailyCaps> {
	const [settingsRows, overrideRows] = await io.sql(DAILY_SQL);
	const settings = new Map(settingsRows.results.map((r) => [String(r.name), Number(r.value)]));
	// A missing setting reads as 0, as the Worker reads it (it then refuses).
	const n = (name: string) => settings.get(name) ?? 0;
	return {
		maxAccounts: n("max_accounts"),
		writesPerDay: n("writes_per_day"),
		uploadBytesPerDay: n("upload_bytes_per_day"),
		globalWritesPerDay: n("global_writes_per_day"),
		globalUploadBytesPerDay: n("global_upload_bytes_per_day"),
		writeOverrides: overrideRows.results.map((r) => ({ username: String(r.username), writesPerDay: Number(r.quota_writes_per_day) })),
	};
}

/** Refuses (unless forced) a change after which one account's daily budget exceeds the server's. */
function dailyRefusal(after: DailyCaps, force: boolean): Report | null {
	const problems = dailyOvercommitted(after);
	if (problems.length === 0 || force) return null;
	return refused(`Refused: ${problems.join("; ")}: one account could use up the server's day. Raise the server-wide cap first, or repeat with --force.`);
}

// ------------------------------------------------------------ legacy rows

/** One manifest row: [id, rev, deleted, md5digest]. */
export type ManifestRow = [string, string, boolean, string | null];

/** The legacy rows as `legacy-export` writes them and `purge-legacy` re-reads them. */
export const LEGACY_ROWS_SQL = `SELECT id, rev, deleted,
  CASE WHEN id GLOB 'file:*' AND json_valid(fields) AND json_type(fields, '$.md5digest') = 'text'
       THEN json_extract(fields, '$.md5digest') END AS md5digest
  FROM docs ORDER BY id`;

const toManifestRows = (result: StatementResult): ManifestRow[] =>
	result.results.map((r) => [String(r.id), String(r.rev), Number(r.deleted) !== 0, r.md5digest === null ? null : String(r.md5digest)]);

const encoder = new TextEncoder();

function compareUtf8(a: string, b: string): number {
	const x = encoder.encode(a);
	const y = encoder.encode(b);
	for (let i = 0; i < Math.min(x.length, y.length); i++) if (x[i] !== y[i]) return x[i] - y[i];
	return x.length - y.length;
}

/**
 * The manifest hash (docs/ACCOUNTS.md › Command line): rows sorted by id
 * (bytewise UTF-8), each `id TAB rev TAB 1|0 TAB md5digest-or-nothing LF`,
 * SHA-256 as 64 lowercase hex digits. `exported_at` is not covered.
 */
export async function manifestSha256(rows: ManifestRow[]): Promise<string> {
	const sorted = [...rows].sort((a, b) => compareUtf8(a[0], b[0]));
	const text = sorted.map(([id, rev, deleted, md5]) => `${id}\t${rev}\t${deleted ? "1" : "0"}\t${md5 ?? ""}\n`).join("");
	const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", encoder.encode(text)));
	return [...digest].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function legacyTables(io: AdminIO): Promise<string[]> {
	const [tables] = await io.sql(["SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('docs', 'counter') ORDER BY name"]);
	return tables.results.map((r) => String(r.name));
}

const join = (dir: string, name: string): string => `${dir.replace(/\/+$/, "")}/${name}`;

// ----------------------------------------------------------------- commands

/** The daily commands, their settings, and how each changes the daily caps. */
const SETTING_COMMANDS: Record<string, { setting: string; apply: (c: DailyCaps, n: number) => DailyCaps }> = {
	"writes-per-day": { setting: "writes_per_day", apply: (c, n) => ({ ...c, writesPerDay: n }) },
	"upload-bytes-per-day": { setting: "upload_bytes_per_day", apply: (c, n) => ({ ...c, uploadBytesPerDay: n }) },
	"global-writes-per-day": { setting: "global_writes_per_day", apply: (c, n) => ({ ...c, globalWritesPerDay: n }) },
	"global-upload-bytes-per-day": { setting: "global_upload_bytes_per_day", apply: (c, n) => ({ ...c, globalUploadBytesPerDay: n }) },
};

const QUOTA_FLAGS = ["--docs", "--doc-mib", "--files", "--file-mib"];
/** `quota <username>` also takes the account's own daily write budget. */
const ACCOUNT_QUOTA_FLAGS = [...QUOTA_FLAGS, "--writes-per-day"];

interface QuotaChange {
	docs?: number;
	docBytes?: number;
	files?: number;
	fileBytes?: number;
	writesPerDay?: number;
}

function quotaChange(flags: Map<string, string | true>): QuotaChange {
	const change: QuotaChange = {};
	if (flags.has("--docs")) change.docs = positive(flag(flags, "--docs"), "--docs");
	if (flags.has("--doc-mib")) change.docBytes = positive(flag(flags, "--doc-mib"), "--doc-mib", MIB);
	if (flags.has("--files")) change.files = positive(flag(flags, "--files"), "--files");
	if (flags.has("--file-mib")) change.fileBytes = positive(flag(flags, "--file-mib"), "--file-mib", MIB);
	if (flags.has("--writes-per-day")) change.writesPerDay = positive(flag(flags, "--writes-per-day"), "--writes-per-day");
	return change;
}

/**
 * Sets an account's quota overrides in raw units (null: back to the
 * default), refusing to overcommit a global cap, or to give it a daily write
 * budget over the server's, unless forced. The fixtures' `{admin: "quota"}`
 * steps run this.
 */
export async function setQuota(
	io: AdminIO,
	name: string,
	change: { docs?: number | null; docBytes?: number | null; files?: number | null; fileBytes?: number | null; writesPerDay?: number | null },
	force: boolean,
): Promise<Report> {
	const columns: [keyof typeof change, string][] = [
		["docs", "quota_docs"],
		["docBytes", "quota_doc_bytes"],
		["files", "quota_files"],
		["fileBytes", "quota_file_bytes"],
		["writesPerDay", "quota_writes_per_day"],
	];
	const sets = columns
		.filter(([key]) => change[key] !== undefined)
		.map(([key, column]) => {
			const value = change[key];
			if (value !== null && (!Number.isSafeInteger(value) || (value as number) <= 0)) throw new UsageError(`${column} must be positive.`);
			return `${column} = ${value === null ? "NULL" : String(value)}`;
		});
	if (sets.length === 0) throw new UsageError("Name at least one of --docs, --doc-mib, --files, --file-mib, --writes-per-day, or --default.");
	const { capacity: before } = await capacity(io);
	const target = before.accounts.find((a) => a.username === name);
	if (!target) return refused(`No account is named ${name}.`);
	const after: Capacity = {
		...before,
		accounts: before.accounts.map((a) =>
			a.username !== name
				? a
				: {
						...a,
						quotaDocBytes: change.docBytes !== undefined ? change.docBytes : a.quotaDocBytes,
						quotaFileBytes: change.fileBytes !== undefined ? change.fileBytes : a.quotaFileBytes,
					},
		),
	};
	const refusal = overcommitRefusal(after, force);
	if (refusal) return refusal;
	if (typeof change.writesPerDay === "number") {
		const daily = await dailyCaps(io);
		const others = daily.writeOverrides.filter((o) => o.username !== name);
		const dailyRefused = dailyRefusal({ ...daily, writeOverrides: [...others, { username: name, writesPerDay: change.writesPerDay }] }, force);
		if (dailyRefused) return dailyRefused;
	}
	const [updated] = await io.sql([`UPDATE accounts SET ${sets.join(", ")} WHERE username = ${sqlText(name)} RETURNING id`]);
	return updated.results.length === 1 ? ok(`Set ${sets.join(", ")} for ${name}.`) : refused(`No account is named ${name}.`);
}

/** Runs one admin command. Throws `UsageError` for anything malformed, before any SQL runs. */
export async function admin(args: string[], io: AdminIO, now: number = Date.now()): Promise<Report> {
	const [name, ...rest] = args;
	switch (name) {
		case "accounts":
			return accounts(rest, io, now);
		case "settings": {
			arity(parse(rest, [], []).positional, 0, 0);
			const [rows] = await io.sql(["SELECT name, value FROM settings ORDER BY name"]);
			return ok(table([["SETTING", "VALUE"], ...rows.results.map((r) => [String(r.name), String(r.value)])]));
		}
		case "disable":
		case "enable": {
			const { positional } = parse(rest, [], []);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			const disable = name === "disable";
			const [result] = await io.sql([
				disable
					? `UPDATE accounts SET disabled_at = ${now} WHERE username = ${sqlText(user)} AND disabled_at IS NULL RETURNING id`
					: `UPDATE accounts SET disabled_at = NULL WHERE username = ${sqlText(user)} AND disabled_at IS NOT NULL RETURNING id`,
			]);
			return result.results.length === 1
				? ok(`${disable ? "Disabled" : "Enabled"} ${user}.`)
				: refused(`No ${disable ? "enabled" : "disabled"} account is named ${user}.`);
		}
		case "shares": {
			const { positional } = parse(rest, [], []);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			const [found, rows] = await io.sql([
				`SELECT id, disabled_at FROM accounts WHERE username = ${sqlText(user)}`,
				`SELECT s.id, s.size, s.created_at, s.updated_at, s.expires_at FROM shares s JOIN accounts a ON a.id = s.account_id
  WHERE a.username = ${sqlText(user)} ORDER BY s.created_at, s.id`,
			]);
			if (found.results.length !== 1) return refused(`No account is named ${user}.`);
			if (rows.results.length === 0) return ok(`${user} has no published notes.`);
			const disabled = found.results[0].disabled_at !== null ? ` ${user} is disabled, so none of them is served.` : "";
			return ok(
				`${table([
					["SHARE ID", "SIZE", "PUBLISHED", "UPDATED", "EXPIRES"],
					...rows.results.map((r) => [String(r.id), size(r.size), time(r.created_at), time(r.updated_at), time(r.expires_at)]),
				])}\n\nThe server holds only ciphertext; the links' keys never reach it.${disabled}`,
			);
		}
		case "unpublish": {
			const { positional } = parse(rest, [], []);
			arity(positional, 1, 1);
			const shareId = positional[0];
			if (!/^[A-Za-z0-9_-]{22}$/.test(shareId)) {
				throw new UsageError(`"${shareId}" is not a share ID: the 22 characters after /p/ in the link.`);
			}
			const id = sqlText(shareId);
			// The same as the owner's unpublish: the quota goes back, the object is queued for the
			// cron, and deleting the row retires the ID for good (the shares_retire trigger).
			const results = await io.sql([
				`UPDATE accounts SET files = files - 1, file_bytes = file_bytes - (SELECT size + ${FILE_COST} FROM shares WHERE id = ${id})
  WHERE id = (SELECT account_id FROM shares WHERE id = ${id})`,
				`INSERT INTO share_purges (id, requested_at) SELECT id, ${now} FROM shares WHERE id = ${id}
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
				`DELETE FROM shares WHERE id = ${id} RETURNING (SELECT username FROM accounts WHERE accounts.id = shares.account_id) AS username`,
			]);
			const deleted = results[results.length - 1].results[0];
			return deleted
				? ok(
						`Unpublished ${shareId} (${String(deleted.username)}). The link answers 404 now, the ID can never be published again, and the cron deletes its R2 object (and its short link, if it has one) within 15 minutes.`,
					)
				: refused(`No published note has the ID ${shareId}.`);
		}
		case "delete": {
			const { positional, flags } = parse(rest, [], ["--confirm"]);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			const confirm = flag(flags, "--confirm");
			if (confirm === undefined || normalizeUsername(confirm) !== user) {
				throw new UsageError(`Repeat the username to confirm: delete ${user} --confirm ${user}`);
			}
			const id = `(SELECT id FROM accounts WHERE username = ${sqlText(user)})`;
			const results = await io.sql([
				`DELETE FROM account_docs WHERE account_id = ${id}`,
				`DELETE FROM account_files WHERE account_id = ${id}`,
				`INSERT INTO share_purges (id, requested_at) SELECT id, ${now} FROM shares WHERE account_id = ${id}
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
				`DELETE FROM shares WHERE account_id = ${id}`,
				// Shared notes, as the Worker's deletion: its own are stopped (their rooms queued for the
				// cron, their IDs retired by a trigger); its identity and its place in others' go.
				`INSERT INTO collab_purges (id, requested_at) SELECT id, ${now} FROM collab_shares WHERE owner_id = ${id}
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
				`DELETE FROM collab_notifications WHERE recipient_id = ${id} OR sender_id = ${id}
  OR share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_wraps WHERE account_id = ${id} OR share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_lists WHERE share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_requests WHERE account_id = ${id} OR share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_links WHERE share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_members WHERE account_id = ${id} OR share_id IN (SELECT id FROM collab_shares WHERE owner_id = ${id})`,
				`DELETE FROM collab_shares WHERE owner_id = ${id}`,
				`DELETE FROM identities WHERE account_id = ${id}`,
				// The sign-in rows reference the account, so they go before it, as in the Worker's deletion.
				`DELETE FROM device_approvals WHERE account_id = ${id}`,
				`DELETE FROM account_passkeys WHERE account_id = ${id}`,
				`DELETE FROM account_identities WHERE account_id = ${id}`,
				`DELETE FROM auth_flows WHERE account_id = ${id}`,
				`DELETE FROM sessions WHERE account_id = ${id}`,
				`DELETE FROM limits WHERE key IN ('login-user:' || ${id}, 'password-account:' || ${id}, 'writes-account:' || ${id},
  'uploads-account:' || ${id}, 'lookup-account:' || ${id}, 'join-account:' || ${id}, 'notify-account:' || ${id},
  'approval-create:' || ${id}, 'approval-poll:' || ${id}, 'approval-respond:' || ${id})`,
				`INSERT INTO purges (prefix, requested_at) SELECT 'accounts/' || id || '/', ${now} FROM accounts WHERE username = ${sqlText(user)}
  ON CONFLICT (prefix) DO UPDATE SET requested_at = excluded.requested_at`,
				`DELETE FROM accounts WHERE username = ${sqlText(user)} RETURNING id`,
			]);
			const deleted = results[results.length - 1].results[0];
			return deleted
				? ok(
						`Deleted ${user} (${String(deleted.id)}). The cron empties accounts/${String(deleted.id)}/, deletes its published notes in R2 and purges the notes it shared within 15 minutes.`,
					)
				: refused(`No account is named ${user}.`);
		}
		case "signups": {
			const { positional } = parse(rest, [], []);
			const mode = positional[0];
			if (mode === "open" || mode === "close" || mode === "closed") {
				arity(positional, 1, 1);
				const value = mode === "open" ? "open" : "closed";
				await io.sql([`UPDATE settings SET value = ${sqlText(value)} WHERE name = 'signup'`]);
				return ok(`Sign-ups are ${value}.`);
			}
			if (mode === "only") {
				arity(positional, 2, 2);
				const user = username(positional[1]);
				const [, existing] = await io.sql([
					`UPDATE settings SET value = ${sqlText(`only:${user}`)} WHERE name = 'signup'`,
					`SELECT id FROM accounts WHERE username = ${sqlText(user)}`,
				]);
				return existing.results.length > 0
					? ok(
							`Sign-ups are reserved for ${user}, but ${user} already has an account: the reservation is spent. Run \`signups close\` or \`signups open\`. If that account isn't the one you meant to create, someone took the name first: run \`delete ${user} --confirm ${user}\`, then \`signups only ${user}\` again, and sign up at once.`,
						)
					: ok(`Sign-ups are reserved for ${user} until that account exists. Sign up now: anyone who guesses the name can take it first.`);
			}
			throw new UsageError(USAGE);
		}
		case "max-accounts": {
			const { positional, flags } = parse(rest, ["--force"], []);
			arity(positional, 1, 1);
			const n = positive(positional[0], "max-accounts");
			const { capacity: before } = await capacity(io);
			const refusal = overcommitRefusal({ ...before, maxAccounts: n }, flags.has("--force"));
			if (refusal) return refusal;
			// The fair shares of the server-wide daily caps follow max_accounts.
			const daily = { ...(await dailyCaps(io)), maxAccounts: n };
			await io.sql([`UPDATE settings SET value = ${sqlText(String(n))} WHERE name = 'max_accounts'`]);
			return ok(`max_accounts = ${n}.${dailyShares(daily)}`);
		}
		case "quota": {
			const { positional, flags } = parse(rest, ["--default", "--force"], ACCOUNT_QUOTA_FLAGS);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			if (flags.has("--default")) {
				if (ACCOUNT_QUOTA_FLAGS.some((f) => flags.has(f))) throw new UsageError("--default goes alone.");
				return setQuota(io, user, { docs: null, docBytes: null, files: null, fileBytes: null, writesPerDay: null }, flags.has("--force"));
			}
			return setQuota(io, user, quotaChange(flags), flags.has("--force"));
		}
		case "default-quota": {
			const { positional, flags } = parse(rest, ["--force"], QUOTA_FLAGS);
			arity(positional, 0, 0);
			const change = quotaChange(flags);
			const settings: [string, number | undefined][] = [
				["quota_docs", change.docs],
				["quota_doc_bytes", change.docBytes],
				["quota_files", change.files],
				["quota_file_bytes", change.fileBytes],
			];
			const chosen = settings.filter((s): s is [string, number] => s[1] !== undefined);
			if (chosen.length === 0) throw new UsageError("Name at least one of --docs, --doc-mib, --files, --file-mib.");
			const { capacity: before } = await capacity(io);
			const after = {
				...before,
				defaultDocBytes: change.docBytes ?? before.defaultDocBytes,
				defaultFileBytes: change.fileBytes ?? before.defaultFileBytes,
			};
			const refusal = overcommitRefusal(after, flags.has("--force"));
			if (refusal) return refusal;
			await io.sql(chosen.map(([setting, value]) => `UPDATE settings SET value = ${sqlText(String(value))} WHERE name = ${sqlText(setting)}`));
			return ok(`Defaults: ${chosen.map(([setting, value]) => `${setting} = ${value}`).join(", ")}.`);
		}
		case "writes-per-day":
		case "upload-bytes-per-day":
		case "global-writes-per-day":
		case "global-upload-bytes-per-day": {
			const { positional, flags } = parse(rest, ["--force"], []);
			arity(positional, 1, 1);
			const { setting, apply } = SETTING_COMMANDS[name];
			const n = positive(positional[0], name);
			const after = apply(await dailyCaps(io), n);
			const refusal = dailyRefusal(after, flags.has("--force"));
			if (refusal) return refusal;
			await io.sql([`UPDATE settings SET value = ${sqlText(String(n))} WHERE name = ${sqlText(setting)}`]);
			return ok(`${setting} = ${n}.${dailyShares(after)}`);
		}
		case "recount":
			return recount(rest, io, now);
		case "restore":
			return restore(rest, io, now);
		case "restored": {
			const { positional, flags } = parse(rest, ["--all"], []);
			arity(positional, 0, 1);
			const all = flags.has("--all");
			if (all === (positional.length === 1)) throw new UsageError("Name one account, or give --all.");
			const where = all ? "" : ` WHERE username = ${sqlText(username(positional[0]))}`;
			const [result] = await io.sql([newEpochs(where)]);
			if (result.results.length === 0) return refused(all ? "There are no accounts." : `No account is named ${positional[0]}.`);
			return ok(`New epoch for ${result.results.map((r) => String(r.username)).join(", ")}: each Mac re-merges on its next sync.`);
		}
		case "legacy-status":
			arity(parse(rest, [], []).positional, 0, 0);
			return legacyStatus(io);
		case "legacy-export": {
			const { positional, flags } = parse(rest, [], ["--dir"]);
			arity(positional, 0, 0);
			const dir = flag(flags, "--dir");
			if (!dir) throw new UsageError("legacy-export needs --dir DIR.");
			return legacyExport(io, dir, now);
		}
		case "purge-legacy": {
			const { positional, flags } = parse(rest, [], ["--account", "--verified"]);
			arity(positional, 0, 0);
			const user = username(flag(flags, "--account"));
			const verified = flag(flags, "--verified");
			if (!verified) throw new UsageError("purge-legacy needs --verified R.json (from kanso-sync verify --legacy).");
			return purgeLegacy(io, user, verified, now);
		}
		default:
			throw new UsageError(USAGE);
	}
}

// ------------------------------------------------------------------ restore

/** RFC 3339 with a zone, to the minute, second or fraction (what wrangler accepts). */
const RFC3339 = /^\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d(?::[0-5]\d(?:\.\d{1,9})?)?(?:Z|[+-][0-2]\d:[0-5]\d)$/;
/** A D1 bookmark as `wrangler d1 time-travel info` prints it: 8-8-8-32 lowercase hex digits. */
const BOOKMARK = /^[0-9a-f]{8}-[0-9a-f]{8}-[0-9a-f]{8}-[0-9a-f]{32}$/;

function restoreTarget(value: string, now: number): RestoreTarget {
	if (BOOKMARK.test(value)) return { kind: "bookmark", value };
	const ms = /^[0-9]{10}$/.test(value) ? Number(value) * 1000 : RFC3339.test(value) ? Date.parse(value) : Number.NaN;
	if (!Number.isFinite(ms)) {
		throw new UsageError(
			`"${value}" is not a restore point: give a Unix time in seconds, an RFC 3339 time such as 2026-10-17T08:00:00Z, or a bookmark from \`npx wrangler d1 time-travel info ${D1_DATABASE}\`.`,
		);
	}
	if (ms > now) throw new UsageError(`${value} is in the future.`);
	if (ms < now - 30 * DAY) throw new UsageError(`${value} is more than 30 days ago, past what D1 Time Travel keeps.`);
	return { kind: "timestamp", value };
}

const RESTORE_EFFECTS = `Everything in D1 goes back to that point, sessions, passwords and recovery keys included:
accounts and devices added since are gone (their Macs must create the account or sign in
again), and every sign-out, deletion and password or recovery key change since is undone. So
revoke again any device revoked since, and anyone who changed a password or recovery key since
must change it again. R2 isn't restored: the cron drops rows whose file is gone and deletes
files that have no row.`;

/**
 * `restore <time | bookmark>`: the only way to restore D1 (docs/ACCOUNTS.md ›
 * Rollback). A restore alone would leave each Mac trusting its cursor against
 * an older feed, so the epoch rotation of `restored --all` follows at once.
 * Without `--confirm` it only prints what it would run.
 */
async function restore(rest: string[], io: AdminIO, now: number): Promise<Report> {
	const { positional, flags } = parse(rest, [], ["--confirm"]);
	arity(positional, 1, 1);
	if (io.local) throw new UsageError("restore works on the deployed database only: D1 Time Travel keeps no local history.");
	const target = restoreTarget(positional[0], now);
	const confirm = flag(flags, "--confirm");
	if (confirm === undefined) {
		return ok(
			[
				`Dry run: nothing changed. With --confirm ${target.value}, this runs against the deployed database:`,
				`  1. npx wrangler ${restoreArgs(target).join(" ")}`,
				`  2. at once, as restored --all: ${newEpochs()}`,
				RESTORE_EFFECTS,
			].join("\n"),
		);
	}
	if (confirm !== target.value) throw new UsageError(`Repeat the restore point to confirm: restore ${target.value} --confirm ${target.value}`);
	const restored = await io.timeTravelRestore(target);
	const undo = `To undo the restore: npm run -s admin -- restore ${restored.previousBookmark} --confirm ${restored.previousBookmark}`;
	let rotated: StatementResult;
	try {
		[rotated] = await io.sql([newEpochs()]);
	} catch (err) {
		return refused(
			[
				`Restored ${D1_DATABASE} to bookmark ${restored.bookmark}, but the new epochs failed (${(err as Error).message}).`,
				"Run `npm run -s admin -- restored --all` now, before any Mac syncs.",
				undo,
			].join("\n"),
		);
	}
	const names = rotated.results.map((r) => String(r.username));
	return ok(
		[
			`Restored ${D1_DATABASE} to bookmark ${restored.bookmark}.`,
			names.length > 0 ? `New epoch for ${names.join(", ")}: each Mac re-merges on its next sync.` : "There are no accounts at that point.",
			undo,
			RESTORE_EFFECTS,
		].join("\n"),
	);
}

async function accounts(rest: string[], io: AdminIO, now: number): Promise<Report> {
	arity(parse(rest, [], []).positional, 0, 0);
	const today = Math.floor(now / DAY) * DAY;
	const [rows, settings, sums, daily, purges] = await io.sql([
		`SELECT a.username, a.id, a.created_at, a.disabled_at, a.docs, a.doc_bytes, a.files, a.file_bytes,
  coalesce(a.quota_docs, d.docs, 0) AS quota_docs, coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota_doc_bytes,
  coalesce(a.quota_files, d.files, 0) AS quota_files, coalesce(a.quota_file_bytes, d.file_bytes, 0) AS quota_file_bytes,
  coalesce((SELECT CASE WHEN window_start = ${today} THEN count ELSE 0 END FROM limits WHERE key = 'writes-account:' || a.id), 0) AS writes_today,
  coalesce(a.quota_writes_per_day, (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'writes_per_day'), 0) AS writes_per_day,
  coalesce((SELECT CASE WHEN window_start = ${today} THEN count ELSE 0 END FROM limits WHERE key = 'uploads-account:' || a.id), 0) AS uploads_today,
  (SELECT count(*) FROM sessions s WHERE s.account_id = a.id AND s.revoked_at IS NULL) AS devices,
  (SELECT max(last_seen_at) FROM sessions s WHERE s.account_id = a.id) AS last_seen
  FROM accounts a, defaults d ORDER BY a.created_at, a.username`,
		"SELECT name, value FROM settings ORDER BY name",
		"SELECT count(*) AS accounts, coalesce(sum(doc_bytes), 0) AS doc_bytes, coalesce(sum(file_bytes), 0) AS file_bytes FROM accounts",
		"SELECT name, day, count FROM daily ORDER BY name",
		"SELECT prefix, requested_at FROM purges ORDER BY requested_at, prefix",
	]);
	const accountRows = rows.results.map((a) => [
		String(a.username),
		String(a.id),
		date(a.created_at),
		`${a.docs}/${a.quota_docs}`,
		`${size(a.doc_bytes)}/${size(a.quota_doc_bytes)}`,
		`${a.files}/${a.quota_files}`,
		`${size(a.file_bytes)}/${size(a.quota_file_bytes)}`,
		`${a.writes_today}/${a.writes_per_day}`,
		size(a.uploads_today),
		String(a.devices),
		time(a.last_seen),
		a.disabled_at === null ? "active" : `disabled ${date(a.disabled_at)}`,
	]);
	const s = sums.results[0] ?? {};
	const setting = new Map(settings.results.map((r) => [String(r.name), String(r.value)]));
	const day = Math.floor(now / DAY);
	const dailyText = daily.results.map((r) => `${r.name} ${Number(r.day) >= day ? r.count : 0}`).join(", ");
	const max = Number(setting.get("max_accounts") ?? 0);
	const fairWrites = fairShare(Number(setting.get("global_writes_per_day") ?? 0), max);
	const fairUploads = fairShare(Number(setting.get("global_upload_bytes_per_day") ?? 0), max);
	return ok(
		[
			accountRows.length > 0
				? table([
						["USERNAME", "ID", "CREATED", "DOCS", "DOC BYTES", "FILES", "FILE BYTES", "WRITES TODAY", "UPLOADED TODAY", "DEVICES", "LAST SEEN", "STATUS"],
						...accountRows,
					])
				: "No accounts.",
			"",
			table([["SETTING", "VALUE"], ...settings.results.map((r) => [String(r.name), String(r.value)])]),
			"",
			`Accounts: ${s.accounts} of ${setting.get("max_accounts")}. Documents: ${size(s.doc_bytes)} of ${size(Number(setting.get("global_doc_bytes")))}. Files: ${size(s.file_bytes)} of ${size(Number(setting.get("global_file_bytes")))}.`,
			`Today (UTC): ${dailyText}. Past a server-wide cap, accounts within their fair share (${fairWrites} writes, ${fairUploads} upload bytes) carry on.`,
			purges.results.length > 0
				? `Pending purges: ${purges.results.map((p) => `${p.prefix} (since ${time(p.requested_at)})`).join(", ")}.`
				: "No pending purges.",
		].join("\n"),
	);
}

async function recount(rest: string[], io: AdminIO, now: number): Promise<Report> {
	const { positional } = parse(rest, [], []);
	arity(positional, 0, 1);
	const where = positional.length === 1 ? ` WHERE username = ${sqlText(username(positional[0]))}` : "";
	const columns = "username, docs, doc_bytes, files, file_bytes";
	// A recount sets recounted_at, which the quota triggers exempt: a
	// correction must never be refused. It always changes, even within a millisecond.
	const [before, after] = await io.sql([
		`SELECT ${columns} FROM accounts${where} ORDER BY username`,
		`UPDATE accounts SET
  docs = (SELECT count(*) FROM account_docs d WHERE d.account_id = accounts.id AND d.deleted = 0),
  doc_bytes = (SELECT coalesce(sum(octet_length(d.sealed) + ${ROW_COST}), 0) FROM account_docs d WHERE d.account_id = accounts.id),
  files = (SELECT count(*) FROM account_files f WHERE f.account_id = accounts.id)
    + (SELECT count(*) FROM shares s WHERE s.account_id = accounts.id),
  file_bytes = (SELECT coalesce(sum(f.size + ${FILE_COST}), 0) FROM account_files f WHERE f.account_id = accounts.id)
    + (SELECT coalesce(sum(s.size + ${FILE_COST}), 0) FROM shares s WHERE s.account_id = accounts.id),
  recounted_at = max(${now}, coalesce(recounted_at, 0) + 1)${where}
  RETURNING ${columns}`,
	]);
	if (after.results.length === 0) return refused(positional.length === 1 ? `No account is named ${positional[0]}.` : "There are no accounts.");
	const old = new Map(before.results.map((r) => [String(r.username), r]));
	const changed = after.results.filter((r) => {
		const was = old.get(String(r.username));
		return !was || ["docs", "doc_bytes", "files", "file_bytes"].some((k) => Number(was[k]) !== Number(r[k]));
	});
	const lines = changed.map((r) => {
		const was = old.get(String(r.username)) ?? {};
		return `${r.username}: docs ${was.docs} → ${r.docs}, doc_bytes ${was.doc_bytes} → ${r.doc_bytes}, files ${was.files} → ${r.files}, file_bytes ${was.file_bytes} → ${r.file_bytes}`;
	});
	return ok(
		`Recounted ${after.results.length} account${after.results.length === 1 ? "" : "s"}; ${changed.length} changed.${lines.length > 0 ? `\n${lines.join("\n")}` : ""}`,
	);
}

async function legacyStatus(io: AdminIO): Promise<Report> {
	const tables = await legacyTables(io);
	const [purge] = await io.sql(["SELECT requested_at FROM purges WHERE prefix = 'files/'"]);
	const purgeText =
		purge.results.length > 0
			? `The files/ purge is pending (queued ${time(purge.results[0].requested_at)}).`
			: "No files/ purge is pending.";
	if (tables.length === 0) return ok(`The single-user tables are gone. ${purgeText}`);
	const statements: string[] = [];
	if (tables.includes("docs")) {
		statements.push(`SELECT count(*) AS rows, coalesce(sum(deleted = 0), 0) AS live,
  coalesce(sum(deleted = 0 AND id GLOB 'file:*'), 0) AS live_files,
  coalesce(sum(length(id) + length(rev) + length(fields) + 64), 0) AS bytes FROM docs`);
	}
	if (tables.includes("counter")) statements.push("SELECT value FROM counter WHERE name = 'seq'");
	const results = await io.sql(statements);
	const lines = [`Single-user tables: ${tables.join(", ")}.`];
	if (tables.includes("docs")) {
		const d = results[0].results[0];
		lines.push(`docs: ${d.rows} rows, ${d.live} live documents (${d.live_files} files), about ${size(d.bytes)}.`);
	}
	if (tables.includes("counter")) lines.push(`counter: seq ${results[results.length - 1].results[0]?.value ?? "missing"}.`);
	lines.push(purgeText);
	return ok(lines.join("\n"));
}

async function legacyExport(io: AdminIO, dir: string, now: number): Promise<Report> {
	const tables = await legacyTables(io);
	if (!tables.includes("docs") || !tables.includes("counter")) return refused("The single-user tables are gone: nothing to export.");
	const [result] = await io.sql([LEGACY_ROWS_SQL]);
	const rows = toManifestRows(result);
	await io.makeDir(join(dir, "files"));
	// The SQL dump first: if wrangler can't make it, no manifest suggests the export is complete.
	await io.exportTables(["docs", "counter"], join(dir, "legacy.sql"));
	await io.writeFile(join(dir, "manifest.json"), `${JSON.stringify({ exported_at: now, rows }, null, 1)}\n`);
	const files = rows.filter(([id, , deleted]) => id.startsWith("file:") && !deleted);
	const missing: string[] = [];
	for (const [id] of files) {
		if (!(await io.r2Get(`files/${id}`, join(dir, `files/${encodeURIComponent(id)}`)))) missing.push(id);
	}
	const hash = await manifestSha256(rows);
	return {
		ok: missing.length === 0,
		message: [
			`Exported ${rows.length} legacy rows (${rows.filter((r) => !r[2]).length} live) to ${join(dir, "manifest.json")} and ${join(dir, "legacy.sql")}.`,
			`Saved ${files.length - missing.length} of ${files.length} live files in ${join(dir, "files")}.`,
			missing.length > 0 ? `Missing from R2: ${missing.join(", ")}.` : "",
			`Manifest SHA-256: ${hash}`,
		]
			.filter(Boolean)
			.join("\n"),
	};
}

async function purgeLegacy(io: AdminIO, user: string, verifiedFile: string, now: number): Promise<Report> {
	let verified: Record<string, unknown>;
	try {
		verified = JSON.parse(await io.readFile(verifiedFile));
	} catch (err) {
		return refused(`Refused: ${verifiedFile} isn't a readable JSON report (${(err as Error).message}).`);
	}
	if (typeof verified !== "object" || verified === null || Array.isArray(verified)) return refused(`Refused: ${verifiedFile} isn't a report.`);
	const { manifest_sha256, account_id } = verified;
	if (typeof manifest_sha256 !== "string" || !/^[0-9a-f]{64}$/.test(manifest_sha256) || typeof account_id !== "string") {
		return refused(`Refused: ${verifiedFile} has no manifest_sha256 or account_id.`);
	}
	if (verified.ok !== true) return refused(`Refused: ${verifiedFile} isn't ok; run kanso-sync verify --legacy again until it is.`);
	const [account] = await io.sql([`SELECT id FROM accounts WHERE username = ${sqlText(user)}`]);
	const id = account.results[0]?.id;
	if (id === undefined) return refused(`Refused: no account is named ${user}.`);
	if (id !== account_id) return refused(`Refused: ${verifiedFile} verifies account ${account_id}, not ${user} (${String(id)}).`);
	const tables = await legacyTables(io);
	if (!tables.includes("docs") || !tables.includes("counter")) return refused("Refused: the single-user tables are already gone.");
	const [rows] = await io.sql([LEGACY_ROWS_SQL]);
	const fresh = await manifestSha256(toManifestRows(rows));
	if (fresh !== manifest_sha256) {
		return refused(`Refused: the legacy rows changed since the verified export (now ${fresh}, verified ${manifest_sha256}). Export and verify again.`);
	}
	await io.sql([
		"DROP TABLE docs",
		"DROP TABLE counter",
		`INSERT INTO purges (prefix, requested_at) VALUES ('files/', ${now}) ON CONFLICT (prefix) DO NOTHING`,
	]);
	const lines = ["Dropped docs and counter, and queued the files/ purge (the cron runs it within 15 minutes)."];
	if (io.local) {
		lines.push("Local database: no SYNC_TOKEN secret to delete.");
	} else {
		await io.deleteSecret("SYNC_TOKEN");
		lines.push("Deleted the SYNC_TOKEN secret.");
	}
	lines.push("D1 Time Travel keeps the dropped rows for 30 days; the export directory keeps them for good.");
	return ok(lines.join("\n"));
}
