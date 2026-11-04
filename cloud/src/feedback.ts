// In-app feedback intake (docs/FEEDBACK.md): `POST /v1/feedback`, `PUT /v1/feedback/<id>/screenshot`,
// `GET /v1/feedback`, `DELETE /v1/feedback/<id>`.
//
// A report is private. The validated body goes to D1 (re-serialized in a fixed key order, never as it
// arrived), the screenshot to R2 under `feedback/<id>/`, and nothing leaves the Worker here: triage
// (feedback_triage.ts) and filing (github.ts) run later, from the cron. Like crash reports, a 400
// names the failing field's path and never echoes a submitted value. The row holds the account ID
// (signed in) or a keyed hash of the anonymous key, and never an IP address or a device ID.
//
// Signed in: `Authorization: Bearer`, plus `X-Kanso-Device` and `X-Kanso-Device-Proof`, so a stolen
// token alone can't file reports. Anonymous: `X-Kanso-Feedback-Key`, 32 random bytes the install keeps.

import { authenticate } from "./auth";
import { type PepperKeys, hex, hmac16, proofMatches, sha256Hex } from "./crypto";
import type { Env } from "./env";
import * as github from "./github";
import { error, isContentType, json, notFound, readBody, readJson } from "./http";
import { accountKey, bindingFilter, clientIp, count, ipGroup, ipKey, sessionFilter } from "./limits";
import { DAY, HOUR, proof32 } from "./protocol";

export const MAX_BODY_BYTES = 48 * 1024;
export const MAX_SCREENSHOT_BYTES = 5 * 1024 * 1024;
export const MIN_TEXT = 10;
export const MAX_TEXT = 4000;
export const MAX_ERRORS = 20;
export const MAX_ERROR_CHARS = 200;
/** How long the screenshot slot of a report stays open. */
export const UPLOAD_WINDOW_MS = HOUR;
/** Raw reports are kept this long after the issue closes (after receipt, for noise). */
export const KEEP_DAYS = 180;
/** Webhook deliveries are remembered this long. */
export const DELIVERY_KEEP_MS = DAY;

export const KINDS = ["bug", "idea", "question"] as const;
export type Kind = (typeof KINDS)[number];
export const PLATFORMS = ["macos", "windows", "linux", "ios", "android"] as const;
export const SCREENS = ["editor", "note-list", "settings", "capture", "ask", "board", "graph", "calendar", "sync", "account", "other"] as const;
export const SYNCS = ["none", "kanso", "self-hosted"] as const;
export const IMAGE_TYPES = ["image/png", "image/jpeg"] as const;

export type Status = "received" | "noise" | "duplicate" | "filed" | "building" | "pr_open" | "merged" | "released" | "closed";

export interface AppInfo {
	platform: (typeof PLATFORMS)[number];
	version: string;
	build: string;
	patch: number | null;
	os: string;
	device: string;
	locale: string;
	screen: (typeof SCREENS)[number];
	sync: (typeof SYNCS)[number];
	errors: string[];
}

export interface Report {
	v: 1;
	id: string;
	kind: Kind;
	text: string;
	credit: { name: string } | null;
	app: AppInfo | null;
	screenshot: { type: (typeof IMAGE_TYPES)[number]; bytes: number; sha256: string } | null;
}

// ------------------------------------------------------------------ patterns

export const ID = /^fb_[0-9a-z]{26}$/;
const CREDIT_NAME = /^[\p{L}\p{N} ._-]{1,39}$/u;
const VERSION = /^[0-9]{1,4}(\.[0-9]{1,4}){2}$/;
const BUILD = /^[0-9]{1,10}$/;
const OS_VERSION = /^[0-9A-Za-z._-]{1,32}$/;
const DEVICE = /^[A-Za-z0-9 ,._()-]{1,64}$/;
const LOCALE = /^[A-Za-z]{2,3}([-_][A-Za-z0-9]{2,8}){0,2}$/;
const SHA256 = /^[0-9a-f]{64}$/;
/** What `scrub_log_line` (kanso-engine) lets through; the Worker holds the same grammar. */
const LOG_LINE = /^(?:(?:update|theme|sync|ai|voice|export|import): [a-z][a-z ,:_-]{0,180}|crash: [A-Za-z_$][A-Za-z0-9_$.]{0,127})$/;
/** Control characters (the line feed is removed before this is tested). */
const CONTROL = /\p{Cc}/u;

export type Checked<T> = { ok: true; value: T } | { ok: false; field: string };
const fail = (field: string): { ok: false; field: string } => ({ ok: false, field });
const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const isText = (value: unknown, pattern: RegExp): value is string => typeof value === "string" && pattern.test(value);
const oneOf = <T extends string>(value: unknown, list: readonly T[]): value is T => typeof value === "string" && (list as readonly string[]).includes(value);

/** The object at `path` if it has exactly `keys`; else the failing path (an extra key is reported at the object). */
function exactly(value: unknown, path: string, keys: readonly string[]): Checked<Record<string, unknown>> {
	if (!isObject(value)) return fail(path === "" ? "(body)" : path);
	const join = (key: string) => (path === "" ? key : `${path}.${key}`);
	for (const key of keys) if (!Object.hasOwn(value, key)) return fail(join(key));
	return Object.keys(value).length === keys.length ? { ok: true, value } : fail(path === "" ? "(body)" : path);
}

/** Code points in `text`. */
export const length = (text: string): number => [...text].length;

/** The report text the Worker keeps: NFC, 10–4,000 characters, no control characters but `\n`; or null. */
export function cleanText(input: unknown): string | null {
	if (typeof input !== "string") return null;
	const text = input.normalize("NFC");
	// A lone surrogate is not text.
	if (/\p{Surrogate}/u.test(text)) return null;
	const n = length(text);
	if (n < MIN_TEXT || n > MAX_TEXT) return null;
	if (CONTROL.test(text.replaceAll("\n", ""))) return null;
	return text;
}

function validApp(input: unknown): Checked<AppInfo> {
	const o = exactly(input, "app", ["platform", "version", "build", "patch", "os", "device", "locale", "screen", "sync", "errors"]);
	if (!o.ok) return o;
	const a = o.value;
	if (!oneOf(a.platform, PLATFORMS)) return fail("app.platform");
	if (!isText(a.version, VERSION)) return fail("app.version");
	if (!isText(a.build, BUILD)) return fail("app.build");
	if (a.patch !== null && !(typeof a.patch === "number" && Number.isInteger(a.patch) && a.patch >= 0 && a.patch <= 100_000)) return fail("app.patch");
	if (!isText(a.os, OS_VERSION)) return fail("app.os");
	if (!isText(a.device, DEVICE)) return fail("app.device");
	if (!isText(a.locale, LOCALE)) return fail("app.locale");
	if (!oneOf(a.screen, SCREENS)) return fail("app.screen");
	if (!oneOf(a.sync, SYNCS)) return fail("app.sync");
	if (!Array.isArray(a.errors) || a.errors.length > MAX_ERRORS) return fail("app.errors");
	for (const [i, line] of a.errors.entries()) {
		if (typeof line !== "string" || line.length > MAX_ERROR_CHARS || !LOG_LINE.test(line)) return fail(`app.errors[${i}]`);
	}
	return {
		ok: true,
		value: {
			platform: a.platform,
			version: a.version,
			build: a.build,
			patch: a.patch as number | null,
			os: a.os,
			device: a.device,
			locale: a.locale,
			screen: a.screen,
			sync: a.sync,
			errors: a.errors as string[],
		},
	};
}

/** The report in the page's key order, or the path of the first failing field. */
export function validateReport(input: unknown): Checked<Report> {
	const top = exactly(input, "", ["v", "id", "kind", "text", "credit", "app", "screenshot"]);
	if (!top.ok) return top;
	const r = top.value;
	if (r.v !== 1) return fail("v");
	if (!isText(r.id, ID)) return fail("id");
	if (!oneOf(r.kind, KINDS)) return fail("kind");
	const text = cleanText(r.text);
	if (text === null) return fail("text");
	let credit: Report["credit"] = null;
	if (r.credit !== null) {
		const c = exactly(r.credit, "credit", ["name"]);
		if (!c.ok) return c;
		if (typeof c.value.name !== "string") return fail("credit.name");
		const name = c.value.name.normalize("NFC");
		if (!CREDIT_NAME.test(name) || name.trim() === "") return fail("credit.name");
		credit = { name };
	}
	let app: AppInfo | null = null;
	if (r.app !== null) {
		const a = validApp(r.app);
		if (!a.ok) return a;
		app = a.value;
	}
	let screenshot: Report["screenshot"] = null;
	if (r.screenshot !== null) {
		const s = exactly(r.screenshot, "screenshot", ["type", "bytes", "sha256"]);
		if (!s.ok) return s;
		if (!oneOf(s.value.type, IMAGE_TYPES)) return fail("screenshot.type");
		const bytes = s.value.bytes;
		if (typeof bytes !== "number" || !Number.isInteger(bytes) || bytes < 1 || bytes > MAX_SCREENSHOT_BYTES) return fail("screenshot.bytes");
		if (!isText(s.value.sha256, SHA256)) return fail("screenshot.sha256");
		screenshot = { type: s.value.type, bytes, sha256: s.value.sha256 };
	}
	return { ok: true, value: { v: 1, id: r.id, kind: r.kind, text, credit, app, screenshot } };
}

/** The first line of `text`, trimmed and cut to 80 characters (My feedback's row). */
export function preview(text: string): string {
	const line = (text.split("\n").find((l) => l.trim() !== "") ?? "").trim();
	const chars = [...line];
	return chars.length > 80 ? `${chars.slice(0, 79).join("")}…` : line;
}

/** Whether `bytes` start like a PNG or a JPEG, as the declared type says. */
export function imageMatches(type: string, bytes: Uint8Array): boolean {
	if (type === "image/png") return bytes.length >= 8 && [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a].every((b, i) => bytes[i] === b);
	if (type === "image/jpeg") return bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff;
	return false;
}

export const screenshotKey = (id: string): string => `feedback/${id}/screenshot`;
export const feedbackPrefix = (id: string): string => `feedback/${id}/`;

// ------------------------------------------------------------------ reporter

export interface Reporter {
	accountId: string | null;
	reporter: string | null;
}

const FEEDBACK_KEY_HEADER = "x-kanso-feedback-key";
const DEVICE_HEADER = "x-kanso-device";
const DEVICE_PROOF_HEADER = "x-kanso-device-proof";

/** `r_` and 16 bytes of HMAC(limit key, anonymous key) as hex: 34 characters. */
export async function reporterHash(keys: PepperKeys, anonymousKey: string): Promise<string> {
	return `r_${hex(await hmac16(keys.limit, "feedback-reporter", anonymousKey))}`;
}

/** Who is calling, or the response that ends the request. A bearer token, when sent, is the only way in. */
export async function whoIs(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Reporter | Response> {
	if (request.headers.has("authorization")) {
		const session = await authenticate(request, env, now);
		if (session instanceof Response) return session;
		const limited = await sessionFilter(env.SESSION_LIMITER, session.accountId);
		if (limited) return limited;
		const proof = proof32(request.headers.get(DEVICE_PROOF_HEADER));
		// Hashed whatever else is wrong, so every refusal costs the same.
		const proofOk = await proofMatches(proof ?? new Uint8Array(32), session.proofHash);
		if (!proof || !proofOk || request.headers.get(DEVICE_HEADER) !== session.deviceId) {
			return error(401, "invalid_credentials", "Wrong credentials.");
		}
		return { accountId: session.accountId, reporter: null };
	}
	const key = request.headers.get(FEEDBACK_KEY_HEADER);
	if (key === null || proof32(key) === null) return error(401, "unauthorized", "Not signed in.", { reason: "unknown" });
	const limited = await bindingFilter(env.FEEDBACK_LIMITER, request);
	if (limited) return limited;
	return { accountId: null, reporter: await reporterHash(keys, key) };
}

const ownerWhere = (who: Reporter): { sql: string; value: string } =>
	who.accountId !== null ? { sql: "account_id = ?1", value: who.accountId } : { sql: "reporter = ?1", value: who.reporter as string };

async function feedbackOpen(env: Env): Promise<boolean> {
	const row = await env.DB.prepare("SELECT CAST(value AS INTEGER) AS open FROM settings WHERE name = 'feedback_open'").first<{ open: number }>();
	return row?.open === 1;
}

// ------------------------------------------------------------------ handlers

interface Row {
	id: string;
	account_id: string | null;
	reporter: string | null;
	created_at: number;
	kind: Kind;
	report: string;
	has_screenshot: number;
	screenshot_state: "none" | "awaited" | "stored" | "expired";
	status: Status;
	issue: number | null;
	duplicate_of: number | null;
	pr: number | null;
	release: string | null;
	credit_name: string | null;
}

const upload = (id: string, createdAt: number) => ({ url: `/v1/feedback/${id}/screenshot`, expires_ms: createdAt + UPLOAD_WINDOW_MS });

const sameOwner = (row: Row, who: Reporter): boolean => (who.accountId !== null ? row.account_id === who.accountId : row.reporter === who.reporter);

/** POST /v1/feedback: 201 stored, 200 a repeat, 400, 401/403, 409 the ID is another reporter's, 413, 429, 503 closed. */
export async function submit(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> {
	const who = await whoIs(request, env, now, keys);
	if (who instanceof Response) return who;
	if (!(await feedbackOpen(env))) return error(503, "feedback_closed", "Feedback is closed.");
	const body = await readJson(request, MAX_BODY_BYTES);
	if (!body.ok) return body.response;
	const checked = validateReport(body.value);
	if (!checked.ok) return error(400, "invalid", `Invalid report: ${checked.field}.`, { field: checked.field });
	const r = checked.value;

	const existing = await env.DB.prepare("SELECT * FROM feedback WHERE id = ?1").bind(r.id).first<Row>();
	if (existing) return repeat(existing, who);

	const own =
		who.accountId !== null
			? await count(env.DB, "feedback-account", accountKey("feedback-account", who.accountId), now)
			: await count(env.DB, "feedback-ip", await ipKey(keys, "feedback-ip", ipGroup(clientIp(request))), now);
	if (own) return own;
	const global = await count(env.DB, "feedback-global", "feedback-global:all", now);
	if (global) return global;

	try {
		await env.DB.batch([
			env.DB.prepare(
				`INSERT INTO feedback (id, account_id, reporter, created_at, kind, report, has_screenshot, screenshot_state, credit_name)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)`,
			).bind(r.id, who.accountId, who.reporter, now, r.kind, JSON.stringify(r), r.screenshot ? 1 : 0, r.screenshot ? "awaited" : "none", r.credit?.name ?? null),
			env.DB.prepare("INSERT INTO feedback_events (feedback_id, at, status, detail) VALUES (?1, ?2, 'received', NULL)").bind(r.id, now),
		]);
	} catch (err) {
		// A parallel request with the same ID won the insert.
		const winner = await env.DB.prepare("SELECT * FROM feedback WHERE id = ?1").bind(r.id).first<Row>();
		if (winner) return repeat(winner, who);
		throw err;
	}
	return json({ id: r.id, status: "received", ...(r.screenshot ? { upload: upload(r.id, now) } : {}) }, 201);
}

function repeat(row: Row, who: Reporter): Response {
	if (!sameOwner(row, who)) return error(409, "conflict", "That report ID is taken.");
	const slot = row.screenshot_state === "awaited" ? { upload: upload(row.id, row.created_at) } : {};
	return json({ id: row.id, status: row.status, ...slot });
}

/** PUT /v1/feedback/<id>/screenshot: 204 stored, 400 wrong bytes, 404, 409 already stored, 410 slot closed, 413, 415. */
export function putScreenshot(id: string) {
	return async (request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> => {
		const who = await whoIs(request, env, now, keys);
		if (who instanceof Response) return who;
		if (!ID.test(id)) return notFound();
		const row = await env.DB.prepare("SELECT * FROM feedback WHERE id = ?1").bind(id).first<Row>();
		if (!row || !sameOwner(row, who) || row.has_screenshot !== 1) return notFound();
		if (row.screenshot_state === "stored") return error(409, "conflict", "The screenshot is already stored.");
		if (row.screenshot_state !== "awaited" || now > row.created_at + UPLOAD_WINDOW_MS) return error(410, "upload_expired", "The upload slot has closed.");
		const declared = (JSON.parse(row.report) as Report).screenshot;
		if (!declared) return notFound();
		const type = request.headers.get("content-type");
		if (!isContentType(type, declared.type, false)) return error(415, "unsupported_media_type", `Send ${declared.type}.`);
		const bytes = await readBody(request, MAX_SCREENSHOT_BYTES);
		if (!bytes) return error(413, "too_large", "The image is larger than 5 MiB.");
		if (bytes.length !== declared.bytes || !imageMatches(declared.type, bytes) || (await sha256Hex(bytes)) !== declared.sha256) {
			return error(400, "invalid", "Invalid screenshot.", { field: "screenshot" });
		}
		// Claim the slot first, so two uploads can't both write.
		const claimed = await env.DB.prepare("UPDATE feedback SET screenshot_state = 'stored' WHERE id = ?1 AND screenshot_state = 'awaited' RETURNING id")
			.bind(id)
			.first();
		if (!claimed) return error(409, "conflict", "The screenshot is already stored.");
		try {
			await env.FILES.put(screenshotKey(id), bytes, { customMetadata: { type: declared.type } });
		} catch (err) {
			await env.DB.prepare("UPDATE feedback SET screenshot_state = 'awaited' WHERE id = ?1 AND screenshot_state = 'stored'").bind(id).run();
			throw err;
		}
		return new Response(null, { status: 204 });
	};
}

/** GET /v1/feedback[?released=X.Y.Z]: this reporter's reports, newest first, at most 100. */
export async function list(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> {
	const who = await whoIs(request, env, now, keys);
	if (who instanceof Response) return who;
	const released = new URL(request.url).searchParams.get("released");
	if (released !== null && !VERSION.test(released)) return error(400, "invalid", "Invalid released.", { field: "released" });
	const owner = ownerWhere(who);
	const { results } = await env.DB.prepare(
		`SELECT id, created_at, kind, report, status, issue, duplicate_of, pr, release FROM feedback
  WHERE ${owner.sql} AND (?2 IS NULL OR release IN (?2, 'v' || ?2)) ORDER BY created_at DESC, id LIMIT 100`,
	)
		.bind(owner.value, released)
		.all<Row>();
	const events = await env.DB.prepare(
		`SELECT e.feedback_id, e.at, e.status FROM feedback_events e JOIN feedback f ON f.id = e.feedback_id
  WHERE f.${owner.sql} ORDER BY e.at, e.status`,
	)
		.bind(owner.value)
		.all<{ feedback_id: string; at: number; status: Status }>();
	const chains = new Map<string, { status: Status; at_ms: number }[]>();
	for (const e of events.results) chains.set(e.feedback_id, [...(chains.get(e.feedback_id) ?? []), { status: e.status, at_ms: e.at }]);
	return json({
		reports: results.map((row) => ({
			id: row.id,
			created_ms: row.created_at,
			kind: row.kind,
			preview: preview((JSON.parse(row.report) as { text?: string }).text ?? ""),
			status: row.status,
			issue: row.issue,
			duplicate_of: row.duplicate_of,
			pr: row.pr,
			release: row.release,
			events: chains.get(row.id) ?? [],
		})),
	});
}

/** DELETE /v1/feedback/<id>: 204. The row, its events and its screenshot go; the issue stays, without the credit line. */
export function remove(id: string) {
	return async (request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> => {
		const who = await whoIs(request, env, now, keys);
		if (who instanceof Response) return who;
		if (!ID.test(id)) return notFound();
		const row = await env.DB.prepare("SELECT * FROM feedback WHERE id = ?1").bind(id).first<Row>();
		if (!row || !sameOwner(row, who)) return notFound();
		const credit = row.issue !== null && row.credit_name !== null;
		await env.DB.batch([
			env.DB.prepare("INSERT OR REPLACE INTO feedback_purges (feedback_id, requested_at) VALUES (?1, ?2)").bind(id, now),
			...(credit ? [env.DB.prepare(CREDIT_QUEUE_SQL).bind(row.issue, row.credit_name, now)] : []),
			env.DB.prepare("DELETE FROM feedback_events WHERE feedback_id = ?1").bind(id),
			env.DB.prepare("DELETE FROM feedback WHERE id = ?1").bind(id),
		]);
		try {
			await purgeOne(env, id);
			if (credit) await removeCreditOne(env, row.issue as number, row.credit_name as string, now);
		} catch (err) {
			// The cron finishes the R2 purge and retries the credit edit from their queues.
			console.error(JSON.stringify({ route: "feedback", error: "delete_cleanup_failed", kind: (err as Error)?.name ?? typeof err }));
		}
		return new Response(null, { status: 204 });
	};
}

// ----------------------------------------------------------------- the cron

/** Empties a deleted report's R2 prefix and drops its queue row. */
export async function purgeOne(env: Env, id: string): Promise<void> {
	const listed = await env.FILES.list({ prefix: feedbackPrefix(id), limit: 100 });
	if (listed.objects.length > 0) await env.FILES.delete(listed.objects.map((o) => o.key));
	await env.DB.prepare("DELETE FROM feedback_purges WHERE feedback_id = ?1").bind(id).run();
}

const CREDIT_QUEUE_SQL = "INSERT OR IGNORE INTO feedback_credit_removals (issue, name, requested_at) VALUES (?1, ?2, ?3)";

/** Edits one queued credit line out of its issue and drops the queue row; throws (row kept) when GitHub fails. */
async function removeCreditOne(env: Env, issue: number, name: string, now: number): Promise<void> {
	if (!github.configOf(env)) return; // Kept queued until the App is set up.
	await github.removeCredit(env, issue, name, now);
	await env.DB.prepare("DELETE FROM feedback_credit_removals WHERE issue = ?1 AND name = ?2").bind(issue, name).run();
}

export interface Housekeeping {
	slotsExpired: number;
	purged: number;
	retained: number;
	credits: number;
}


/** Closes upload slots, empties queued R2 prefixes, applies the retention, forgets old webhook deliveries. */
export async function housekeeping(env: Env, now: number, limit = 50): Promise<Housekeeping> {
	const expired = await env.DB.prepare(
		"UPDATE feedback SET screenshot_state = 'expired' WHERE screenshot_state = 'awaited' AND created_at <= ?1 RETURNING id",
	)
		.bind(now - UPLOAD_WINDOW_MS)
		.all();
	const queue = await env.DB.prepare("SELECT feedback_id FROM feedback_purges ORDER BY requested_at LIMIT ?1").bind(limit).all<{ feedback_id: string }>();
	for (const { feedback_id } of queue.results) await purgeOne(env, feedback_id);
	let credits = 0;
	const removals = await env.DB.prepare("SELECT issue, name FROM feedback_credit_removals ORDER BY requested_at LIMIT ?1").bind(limit).all<{ issue: number; name: string }>();
	for (const { issue, name } of removals.results) {
		try {
			await removeCreditOne(env, issue, name, now);
			credits += 1;
		} catch (err) {
			// Stays queued for the next run.
			console.error(JSON.stringify({ route: "feedback", error: "credit_removal_failed", kind: (err as Error)?.name ?? typeof err }));
		}
	}
	// Retention: 180 days after the last status change, whatever the status, so nothing is kept forever.
	// The status chain stays, the raw words and the picture go.
	const cutoff = now - KEEP_DAYS * DAY;
	const due = await env.DB.prepare(
		`SELECT f.id FROM feedback f WHERE f.purged_at IS NULL
  AND coalesce((SELECT max(e.at) FROM feedback_events e WHERE e.feedback_id = f.id), f.created_at) <= ?1 ORDER BY f.created_at LIMIT ?2`,
	)
		.bind(cutoff, limit)
		.all<{ id: string }>();
	for (const { id } of due.results) {
		await env.DB.prepare(
			`UPDATE feedback SET report = json_object('v', 1, 'id', id, 'kind', kind, 'text', '', 'credit', NULL, 'app', NULL, 'screenshot', NULL),
  triage = NULL, credit_name = NULL, screenshot_state = CASE WHEN has_screenshot = 1 THEN 'expired' ELSE 'none' END, purged_at = ?2
  WHERE id = ?1 AND purged_at IS NULL`,
		)
			.bind(id, now)
			.run();
		await env.FILES.delete(screenshotKey(id));
	}
	await env.DB.prepare("DELETE FROM feedback_deliveries WHERE at <= ?1").bind(now - DELIVERY_KEEP_MS).run();
	return { slotsExpired: expired.results.length, purged: queue.results.length, retained: due.results.length, credits };
}

/** The statements that delete an account's feedback with the account (docs/FEEDBACK.md › GDPR); `where` guards each. */
export function deletionStatements(db: D1Database, accountId: string, where: string, guard: string, now: number): D1PreparedStatement[] {
	return [
		// Credit lines on filed issues are edited out by the cron (retried until GitHub accepts).
		db
			.prepare(
				`INSERT OR IGNORE INTO feedback_credit_removals (issue, name, requested_at) SELECT issue, credit_name, ?3 FROM feedback
  WHERE account_id = ?1 AND issue IS NOT NULL AND credit_name IS NOT NULL AND ${where}`,
			)
			.bind(accountId, guard, now),
		db
			.prepare(
				`INSERT OR REPLACE INTO feedback_purges (feedback_id, requested_at) SELECT id, ?3 FROM feedback WHERE account_id = ?1 AND has_screenshot = 1 AND ${where}`,
			)
			.bind(accountId, guard, now),
		db.prepare(`DELETE FROM feedback_events WHERE feedback_id IN (SELECT id FROM feedback WHERE account_id = ?1) AND ${where}`).bind(accountId, guard),
		db.prepare(`DELETE FROM feedback WHERE account_id = ?1 AND ${where}`).bind(accountId, guard),
	];
}
