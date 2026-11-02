// Anonymous usage counts from the apps (docs/METRICS.md): `POST /v1/metrics`.
//
// Unauthenticated, and nothing identifying is read: no session, cookie or device ID, and the
// caller's address only keys the rate limit (a keyed hash in `limits`). The batch is checked field
// by field against the page's closed lists and patterns, then rebuilt from the validated values and
// passed on to PostHog's batch API with the request's country and nothing else of the request:
// never its address or headers. Nothing of a batch is stored in D1. A 400 names the failing field's path and
// never echoes a submitted value, and not even a submitted key: an unexpected key is reported at
// its parent.

import { type Checked, exactly, fail, isText, utcDay } from "./crash_reports";
import type { PepperKeys } from "./crypto";
import type { Env } from "./env";
import { error, readJson } from "./http";
import { clientIp, count, ipGroup, ipKey } from "./limits";
import { DAY } from "./protocol";

/** The largest body, in bytes (the page: "at most 16 KiB"). */
export const MAX_BATCH_BYTES = 16 * 1024;
export const MAX_EVENTS = 32;
/** How far an event's day may be from the Worker's UTC day, either way. */
export const DAY_SLACK_DAYS = 2;
/** PostHog gets this long to answer; a lost count isn't worth waiting for. */
export const POSTHOG_TIMEOUT_MS = 5_000;

export const PLATFORMS = ["macos", "windows", "linux", "ios", "android"] as const;
export const FORMS = ["desktop", "phone", "tablet"] as const;
export const SYNCS = ["off", "kanso", "self_hosted"] as const;
export const NOTES = ["0", "1-9", "10-99", "100-999", "1000+"] as const;
export const FEATURES = [
	"note_created",
	"voice_input",
	"publish",
	"import",
	"lock",
	"ai",
	"quick_capture",
	"shared_note",
	"export",
	"note_edited",
	"note_deleted",
	"note_restored",
	"note_pinned",
	"note_moved",
	"notebook_created",
	"tag_added",
	"template_used",
	"daily_note",
	"search",
	"wikilink_followed",
	"backlinks_opened",
	"graph_opened",
	"task_toggled",
	"attachment_added",
	"table_inserted",
	"diagram",
	"sticky_opened",
	"new_window",
	"focus_mode",
	"plugin_installed",
	"plugin_command",
	"theme_changed",
	"sync_conflict_resolved",
	"review_prompt",
] as const;
export const COUNTS = ["1", "2-5", "6-20", "21+"] as const;
export const DURATIONS = ["<1m", "1-5m", "5-15m", "15-60m", "1-3h", "3h+"] as const;
export const THEME_MODES = ["light", "dark", "system"] as const;
export const EDITOR_MODES = ["edit", "live", "preview", "split"] as const;
export const PLUGINS = ["0", "1-2", "3-5", "6+"] as const;
export const NOTEBOOKS = ["0", "1-4", "5-19", "20+"] as const;
export const ATTACHMENTS_MB = ["0", "<10", "10-100", "100-1000", "1000+"] as const;
export const SYNC_ERRORS = ["0", "1-5", "6+"] as const;
export const CHANNELS = ["stable", "beta", "dev"] as const;
/** The props only `app_active` may add to its seven required ones, in the page's order. */
export const ACTIVE_OPTIONAL = ["locale_lang", "theme_mode", "theme", "editor_mode", "plugins", "notebooks", "attachments_mb", "sync_errors", "channel"] as const;
export const SOURCES = [
	"markdown",
	"obsidian",
	"logseq",
	"notion",
	"evernote",
	"bear",
	"joplin",
	"standard_notes",
	"simplenote",
	"upnote",
	"apple_notes",
	"pouch",
	"other",
] as const;
const DESKTOPS: readonly string[] = ["macos", "windows", "linux"];

type Platform = (typeof PLATFORMS)[number];
type Form = (typeof FORMS)[number];

interface Common {
	platform: Platform;
	form: Form;
	app_version: string;
}
export type MetricsEvent =
	| {
			event: "app_active";
			day: string;
			props: Common & { os_version: string; sync: (typeof SYNCS)[number]; ai: boolean; notes: (typeof NOTES)[number] } & Partial<Record<(typeof ACTIVE_OPTIONAL)[number], string>>;
	  }
	| { event: "app_opened"; day: string; props: Common }
	| { event: "session"; day: string; props: Common & { duration: (typeof DURATIONS)[number] } }
	| { event: "feature_used"; day: string; props: Common & { feature: (typeof FEATURES)[number]; source?: (typeof SOURCES)[number]; count?: (typeof COUNTS)[number] } };

export interface Batch {
	v: 1;
	id: string;
	events: MetricsEvent[];
}

// ------------------------------------------------------------------ patterns

const ID = /^(a|i)_[0-9a-f]{32}$/;
const VERSION = /^[0-9]{1,4}(\.[0-9]{1,4}){2}$/;
const OS_VERSION = /^[0-9A-Za-z._-]{1,32}$/;
const DAY_TEXT = /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/;
const LOCALE_LANG = /^([a-z]{2}|other)$/;
const THEME = /^(custom|[a-z0-9][a-z0-9-]{0,31})$/;
/** A two-letter country; Cloudflare's `XX` (unknown) and `T1` (Tor) are left out. */
const COUNTRY = /^[A-Z]{2}$/;

const oneOf = <T extends string>(list: readonly T[], value: unknown): value is T => typeof value === "string" && (list as readonly string[]).includes(value);

/** Whether `day` is a real calendar day within DAY_SLACK_DAYS of `now`'s UTC day. */
export function dayInRange(day: string, now: number): boolean {
	if (!DAY_TEXT.test(day)) return false;
	const ms = Date.parse(`${day}T00:00:00Z`);
	if (Number.isNaN(ms) || utcDay(ms) !== day) return false;
	return Math.abs(ms - Date.parse(`${utcDay(now)}T00:00:00Z`)) <= DAY_SLACK_DAYS * DAY;
}

// ---------------------------------------------------------------- validation

const COMMON = ["platform", "form", "app_version"] as const;

/** The common props, or the first failing one; desktop platforms are always `desktop`. */
function common(p: Record<string, unknown>, path: string): Checked<Common> {
	if (!oneOf(PLATFORMS, p.platform)) return fail(`${path}.platform`);
	if (!oneOf(FORMS, p.form) || (DESKTOPS.includes(p.platform) && p.form !== "desktop")) return fail(`${path}.form`);
	if (!isText(p.app_version, VERSION)) return fail(`${path}.app_version`);
	return { ok: true, value: { platform: p.platform, form: p.form, app_version: p.app_version } };
}

/**
 * The object at `path` with every key of `required` and no key outside `required` and `optional`:
 * a missing documented key is named, an extra one is reported at the object itself.
 */
function within(value: unknown, path: string, required: readonly string[], optional: readonly string[]): Checked<Record<string, unknown>> {
	if (typeof value !== "object" || value === null || Array.isArray(value)) return fail(path);
	for (const key of required) if (!Object.hasOwn(value, key)) return fail(`${path}.${key}`);
	const known = new Set<string>([...required, ...optional]);
	return Object.keys(value).every((key) => known.has(key)) ? { ok: true, value: value as Record<string, unknown> } : fail(path);
}

const EVENTS = ["app_active", "app_opened", "session", "feature_used"] as const;

function validateEvent(input: unknown, path: string, now: number): Checked<MetricsEvent> {
	const top = exactly(input, path, ["event", "day", "props"]);
	if (!top.ok) return top;
	const e = top.value;
	if (!oneOf(EVENTS, e.event)) return fail(`${path}.event`);
	if (typeof e.day !== "string" || !dayInRange(e.day, now)) return fail(`${path}.day`);
	const day = e.day;
	const at = `${path}.props`;
	if (e.event === "app_opened") {
		const props = exactly(e.props, at, COMMON);
		if (!props.ok) return props;
		const c = common(props.value, at);
		return c.ok ? { ok: true, value: { event: "app_opened", day, props: c.value } } : c;
	}
	if (e.event === "session") {
		const props = exactly(e.props, at, [...COMMON, "duration"]);
		if (!props.ok) return props;
		const c = common(props.value, at);
		if (!c.ok) return c;
		if (!oneOf(DURATIONS, props.value.duration)) return fail(`${at}.duration`);
		return { ok: true, value: { event: "session", day, props: { ...c.value, duration: props.value.duration } } };
	}
	if (e.event === "app_active") {
		const props = within(e.props, at, [...COMMON, "os_version", "sync", "ai", "notes"], ACTIVE_OPTIONAL);
		if (!props.ok) return props;
		const p = props.value;
		const c = common(p, at);
		if (!c.ok) return c;
		if (!isText(p.os_version, OS_VERSION)) return fail(`${at}.os_version`);
		if (!oneOf(SYNCS, p.sync)) return fail(`${at}.sync`);
		if (typeof p.ai !== "boolean") return fail(`${at}.ai`);
		if (!oneOf(NOTES, p.notes)) return fail(`${at}.notes`);
		const optional: Partial<Record<(typeof ACTIVE_OPTIONAL)[number], string>> = {};
		const checks: Record<(typeof ACTIVE_OPTIONAL)[number], (v: unknown) => v is string> = {
			locale_lang: (v): v is string => isText(v, LOCALE_LANG),
			theme_mode: (v): v is string => oneOf(THEME_MODES, v),
			theme: (v): v is string => isText(v, THEME),
			editor_mode: (v): v is string => oneOf(EDITOR_MODES, v),
			plugins: (v): v is string => oneOf(PLUGINS, v),
			notebooks: (v): v is string => oneOf(NOTEBOOKS, v),
			attachments_mb: (v): v is string => oneOf(ATTACHMENTS_MB, v),
			sync_errors: (v): v is string => oneOf(SYNC_ERRORS, v),
			channel: (v): v is string => oneOf(CHANNELS, v),
		};
		// In the page's order, whatever order they arrived in.
		for (const key of ACTIVE_OPTIONAL) {
			if (!Object.hasOwn(p, key)) continue;
			const value = p[key];
			if (!checks[key](value)) return fail(`${at}.${key}`);
			optional[key] = value;
		}
		return {
			ok: true,
			value: { event: "app_active", day, props: { ...c.value, os_version: p.os_version, sync: p.sync, ai: p.ai, notes: p.notes, ...optional } },
		};
	}
	// `source` comes only, and always, with `import`; anywhere else it is an extra key.
	const imported = typeof e.props === "object" && e.props !== null && (e.props as Record<string, unknown>).feature === "import";
	const props = within(e.props, at, [...COMMON, "feature", ...(imported ? ["source"] : [])], ["count"]);
	if (!props.ok) return props;
	const p = props.value;
	const c = common(p, at);
	if (!c.ok) return c;
	if (!oneOf(FEATURES, p.feature)) return fail(`${at}.feature`);
	let source: (typeof SOURCES)[number] | undefined;
	if (imported) {
		if (!oneOf(SOURCES, p.source)) return fail(`${at}.source`);
		source = p.source;
	}
	let count: (typeof COUNTS)[number] | undefined;
	if (Object.hasOwn(p, "count")) {
		if (!oneOf(COUNTS, p.count)) return fail(`${at}.count`);
		count = p.count;
	}
	return {
		ok: true,
		value: { event: "feature_used", day, props: { ...c.value, feature: p.feature, ...(source ? { source } : {}), ...(count ? { count } : {}) } },
	};
}

/** The batch the page defines, rebuilt in the page's key order, or the path of the first failing field. */
export function validateBatch(input: unknown, now: number): Checked<Batch> {
	const top = exactly(input, "", ["v", "id", "events"]);
	if (!top.ok) return top;
	const b = top.value;
	if (b.v !== 1) return fail("v");
	if (!isText(b.id, ID)) return fail("id");
	if (!Array.isArray(b.events) || b.events.length < 1 || b.events.length > MAX_EVENTS) return fail("events");
	const events: MetricsEvent[] = [];
	for (const [i, input] of b.events.entries()) {
		const event = validateEvent(input, `events[${i}]`, now);
		if (!event.ok) return event;
		events.push(event.value);
	}
	return { ok: true, value: { v: 1, id: b.id, events } };
}

// ------------------------------------------------------------------- handler

/** The two-letter country Cloudflare gives the request, or null. */
export function country(request: Request): string | null {
	const value = (request as Request & { cf?: { country?: unknown } }).cf?.country;
	return typeof value === "string" && COUNTRY.test(value) && value !== "XX" ? value : null;
}

/**
 * When an event is stamped in PostHog: the moment the Worker received the batch, so a day's
 * events are never dated later than they arrived; for an earlier UTC day, the last second of that
 * day so it stays on its day.
 */
export function eventTimestamp(day: string, now: number): string {
	return day < utcDay(now) ? `${day}T23:59:59Z` : new Date(now).toISOString();
}

/** PostHog's batch body (docs/METRICS.md › The Worker and PostHog). */
export function posthogBody(key: string, batch: Batch, country: string | null, now: number): unknown {
	return {
		api_key: key,
		batch: batch.events.map((e) => ({
			event: e.event,
			distinct_id: batch.id,
			timestamp: eventTimestamp(e.day, now),
			properties: {
				...e.props,
				...(country === null ? {} : { country }),
				$process_person_profile: false,
				$geoip_disable: true,
				$lib: "kanso-worker",
			},
		})),
	};
}

/**
 * PostHog's `/batch/` answers 200 even for a key it doesn't know, and drops the events, so a key
 * from another project or region fails without a sound. `/flags` does refuse an unknown key: ask it
 * once per isolate and host, and log when it refuses. Nothing but the outcome is logged.
 */
const keyChecks = new Map<string, Promise<void>>();

/** For tests: forget which keys were checked. */
export function resetKeyChecks(): void {
	keyChecks.clear();
}

export function checkKey(host: string, key: string): Promise<void> {
	const base = host.trim().replace(/\/+$/, "");
	const memo = `${base}\n${key}`;
	let check = keyChecks.get(memo);
	if (!check) {
		check = (async () => {
			let outcome: string | number;
			try {
				const response = await fetch(`${base}/flags/?v=2`, {
					method: "POST",
					headers: { "content-type": "application/json", "user-agent": "kanso-sync" },
					body: JSON.stringify({ api_key: key, distinct_id: "kanso-worker-key-check" }),
					redirect: "manual",
					signal: AbortSignal.timeout(POSTHOG_TIMEOUT_MS),
				});
				await response.body?.cancel();
				if (response.ok) return;
				outcome = response.status === 401 || response.status === 403 ? "key_rejected" : `key_check_${response.status}`;
			} catch (err) {
				// Unknown: check again next time.
				keyChecks.delete(memo);
				outcome = `key_check_${(err as Error)?.name ?? "error"}`;
			}
			console.error(JSON.stringify({ route: "posthog", outcome }));
		})();
		keyChecks.set(memo, check);
	}
	return check;
}

/** Sends the batch to PostHog; any failure is logged by status or error name only. Says whether PostHog took it. */
export async function forward(host: string, body: unknown): Promise<boolean> {
	let outcome: string | number;
	try {
		const response = await fetch(`${host.trim().replace(/\/+$/, "")}/batch/`, {
			method: "POST",
			headers: { "content-type": "application/json", "user-agent": "kanso-sync" },
			body: JSON.stringify(body),
			redirect: "manual",
			signal: AbortSignal.timeout(POSTHOG_TIMEOUT_MS),
		});
		await response.body?.cancel();
		if (response.ok) {
			// Counts only, so a quiet PostHog can be told from a quiet Kanso.
			const events = (body as { batch?: unknown[] }).batch?.length ?? 0;
			console.log(JSON.stringify({ route: "posthog", outcome: "ok", events }));
			return true;
		}
		outcome = response.status;
	} catch (err) {
		outcome = (err as Error)?.name ?? "error";
	}
	// Statuses only: never IDs, props or the key.
	console.error(JSON.stringify({ route: "posthog", outcome }));
	return false;
}

/**
 * POST /v1/metrics: 204 valid (whether or not PostHog took it), 400 invalid, 413 over 16 KiB,
 * 429 too many from this address. Without POSTHOG_KEY and POSTHOG_HOST it checks and sends nothing.
 */
export async function submit(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> {
	const limited = await count(env.DB, "metrics-ip", await ipKey(keys, "metrics-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readJson(request, MAX_BATCH_BYTES);
	if (!body.ok) return body.response;
	const batch = validateBatch(body.value, now);
	if (!batch.ok) return error(400, "bad_request", `Invalid batch: ${batch.field}.`, { field: batch.field });
	const key = env.POSTHOG_KEY?.trim();
	if (key && env.POSTHOG_HOST) {
		await Promise.all([checkKey(env.POSTHOG_HOST, key), forward(env.POSTHOG_HOST, posthogBody(key, batch.value, country(request), now))]);
	}
	return new Response(null, { status: 204 });
}
