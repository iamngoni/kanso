// Anonymous usage counts from the apps (docs/METRICS.md): `POST /v1/metrics`.
//
// Unauthenticated, and nothing identifying is read: no session, cookie or device ID, and the
// caller's address only keys the rate limit (a keyed hash in `limits`). The batch is checked field
// by field against the page's closed lists and patterns, then rebuilt from the validated values and
// passed on to PostHog's batch API with the request's country and nothing else of the request:
// never its address or headers. Nothing is stored in D1. A 400 names the failing field's path and
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
export const FEATURES = ["note_created", "voice_input", "publish", "import", "lock", "ai", "quick_capture", "shared_note", "export"] as const;
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
			props: Common & { os_version: string; sync: (typeof SYNCS)[number]; ai: boolean; notes: (typeof NOTES)[number] };
	  }
	| { event: "feature_used"; day: string; props: Common & { feature: (typeof FEATURES)[number]; source?: (typeof SOURCES)[number] } };

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

function validateEvent(input: unknown, path: string, now: number): Checked<MetricsEvent> {
	const top = exactly(input, path, ["event", "day", "props"]);
	if (!top.ok) return top;
	const e = top.value;
	if (e.event !== "app_active" && e.event !== "feature_used") return fail(`${path}.event`);
	if (typeof e.day !== "string" || !dayInRange(e.day, now)) return fail(`${path}.day`);
	const at = `${path}.props`;
	if (e.event === "app_active") {
		const props = exactly(e.props, at, [...COMMON, "os_version", "sync", "ai", "notes"]);
		if (!props.ok) return props;
		const p = props.value;
		const c = common(p, at);
		if (!c.ok) return c;
		if (!isText(p.os_version, OS_VERSION)) return fail(`${at}.os_version`);
		if (!oneOf(SYNCS, p.sync)) return fail(`${at}.sync`);
		if (typeof p.ai !== "boolean") return fail(`${at}.ai`);
		if (!oneOf(NOTES, p.notes)) return fail(`${at}.notes`);
		return { ok: true, value: { event: "app_active", day: e.day, props: { ...c.value, os_version: p.os_version, sync: p.sync, ai: p.ai, notes: p.notes } } };
	}
	// `source` comes only, and always, with `import`; anywhere else it is an extra key.
	const imported = typeof e.props === "object" && e.props !== null && (e.props as Record<string, unknown>).feature === "import";
	const props = exactly(e.props, at, [...COMMON, "feature", ...(imported ? ["source"] : [])]);
	if (!props.ok) return props;
	const p = props.value;
	const c = common(p, at);
	if (!c.ok) return c;
	if (!oneOf(FEATURES, p.feature)) return fail(`${at}.feature`);
	if (!imported) return { ok: true, value: { event: "feature_used", day: e.day, props: { ...c.value, feature: p.feature } } };
	if (!oneOf(SOURCES, p.source)) return fail(`${at}.source`);
	return { ok: true, value: { event: "feature_used", day: e.day, props: { ...c.value, feature: p.feature, source: p.source } } };
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

/** PostHog's batch body (docs/METRICS.md › The Worker and PostHog). */
export function posthogBody(key: string, batch: Batch, country: string | null): unknown {
	return {
		api_key: key,
		batch: batch.events.map((e) => ({
			event: e.event,
			distinct_id: batch.id,
			timestamp: `${e.day}T12:00:00Z`,
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

/** Sends the batch to PostHog; any failure is logged by status or error name only, and dropped. */
async function forward(host: string, body: unknown): Promise<void> {
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
		if (response.ok) return;
		outcome = response.status;
	} catch (err) {
		outcome = (err as Error)?.name ?? "error";
	}
	// Statuses only: never IDs, props or the key.
	console.error(JSON.stringify({ route: "posthog", outcome }));
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
	if (key && env.POSTHOG_HOST) await forward(env.POSTHOG_HOST, posthogBody(key, batch.value, country(request)));
	return new Response(null, { status: 204 });
}
