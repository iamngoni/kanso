// Crash reports from the mobile apps (docs/CRASH-REPORTS.md): `POST /v1/crash-reports`.
//
// Unauthenticated, and nothing identifying is read: no session, cookie or device ID, and the
// caller's address only keys the rate limit (a keyed hash in `limits`, never in the report table).
// The report is checked field by field against the page's patterns, then stored as the validated
// object re-serialized in the page's key order, never as the body that arrived. A 400 names the
// failing field's path and never echoes a submitted value, and not even a submitted key: an
// unexpected key is reported at its parent.

import type { PepperKeys } from "./crypto";
import type { Env } from "./env";
import { error, readJson } from "./http";
import { clientIp, count, ipGroup, ipKey } from "./limits";
import { DAY } from "./protocol";

/** The largest body, in bytes (the page: "at most 32 KiB"). */
export const MAX_REPORT_BYTES = 32 * 1024;
/** Reports are kept this long. */
export const KEEP_DAYS = 90;
export const MAX_FRAMES = 128;
export const MAX_FRAME_CHARS = 300;

export const KINDS = ["dart", "ios-native", "android-native", "android-anr", "android-jvm"] as const;
export type Kind = (typeof KINDS)[number];

export interface CrashReport {
	v: 1;
	kind: Kind;
	type: string;
	app: { version: string; build: string; patch: number | null };
	os: { name: "ios" | "android"; version: string };
	device: string;
	day: string;
	frames: string[];
}

// ------------------------------------------------------------------ patterns

const TYPE = /^[A-Za-z_$][A-Za-z0-9_$.]{0,127}$/;
const VERSION = /^[0-9]{1,4}(\.[0-9]{1,4}){2}$/;
const BUILD = /^[0-9]{1,10}$/;
const OS_VERSION = /^[0-9A-Za-z._-]{1,32}$/;
const DEVICE = /^[A-Za-z0-9 ,._()-]{1,64}$/;
const DAY_TEXT = /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/;

const DART_SYMBOL = /^[A-Za-z_$<][A-Za-z0-9_$.<>]{0,199}$/;
const DART_URI = /^(?:package:[a-z0-9_]{1,64}\/[A-Za-z0-9_/.-]{1,200}\.dart|dart:[a-z0-9_/.-]{1,100})$/;
const NUMBER = /^[0-9]{1,7}$/;
const HEX = /^[0-9a-f]{1,16}$/;
const OFFSET = /^\+0x[0-9a-f]{1,16}$/;
const BINARY = /^[A-Za-z0-9_.+-]{1,64}$/;
const NATIVE_SYMBOL = /^[A-Za-z0-9_$.]{1,256}$/;
const BUILD_ID = /^[0-9a-fA-F-]{8,64}$/;
const JVM_CLASS = /^[A-Za-z_$][A-Za-z0-9_$.]{0,199}$/;
const JVM_METHOD = /^[A-Za-z_$<][A-Za-z0-9_$<>]{0,99}$/;
const JVM_LINE = /^-?[0-9]{1,7}$/;

/** `<uri>:<line>[:<column>]`, or `?` for a URI the scrubber dropped. */
function dartLocation(text: string): boolean {
	if (text === "?") return true;
	const parts = text.split(":");
	// A URI has one colon of its own (`package:` or `dart:`).
	if (parts.length < 3 || parts.length > 4) return false;
	const uri = `${parts[0]}:${parts[1]}`;
	return DART_URI.test(uri) && parts.slice(2).every((p) => NUMBER.test(p));
}

/** `<symbol>+0x<hex>` */
function nativeSymbol(text: string): boolean {
	const plus = text.lastIndexOf("+0x");
	return plus > 0 && NATIVE_SYMBOL.test(text.slice(0, plus)) && HEX.test(text.slice(plus + 3));
}

/** `<class>.<method>:<line>` */
function jvmLocation(text: string): boolean {
	const colon = text.lastIndexOf(":");
	if (colon < 0 || !JVM_LINE.test(text.slice(colon + 1))) return false;
	const dot = text.lastIndexOf(".", colon);
	return dot > 0 && JVM_CLASS.test(text.slice(0, dot)) && JVM_METHOD.test(text.slice(dot + 1, colon));
}

/** Whether `frame` is one of the page's frame patterns, part by part. */
export function validFrame(frame: string): boolean {
	if (frame.length > MAX_FRAME_CHARS) return false;
	const parts = frame.split(" ");
	switch (parts[0]) {
		case "dart":
			return parts.length === 3 && (parts[1] === "?" || DART_SYMBOL.test(parts[1])) && dartLocation(parts[2]);
		case "dart-pc":
			return parts.length === 2 && OFFSET.test(parts[1]);
		case "native": {
			if (parts.length < 3 || parts.length > 5 || !(parts[1] === "?" || BINARY.test(parts[1])) || !OFFSET.test(parts[2])) return false;
			const rest = parts.slice(3);
			// Optional `<symbol>+0x<hex>` first, then an optional build ID; a build ID has no `+`.
			if (rest.length > 0 && rest[0].includes("+")) return nativeSymbol(rest[0]) && (rest.length === 1 || (rest.length === 2 && BUILD_ID.test(rest[1])));
			return rest.length === 0 || (rest.length === 1 && BUILD_ID.test(rest[0]));
		}
		case "jvm":
			return parts.length === 2 && jvmLocation(parts[1]);
		default:
			return false;
	}
}

// ---------------------------------------------------------------- validation

export type Checked<T> = { ok: true; value: T } | { ok: false; field: string };

export const fail = (field: string): { ok: false; field: string } => ({ ok: false, field });
const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
export const isText = (value: unknown, pattern: RegExp): value is string => typeof value === "string" && pattern.test(value);

/**
 * The object at `path` if it has exactly `keys`; else the failing path: a missing documented key
 * is named, an extra one is reported at the object itself (its name is the sender's).
 */
export function exactly(value: unknown, path: string, keys: readonly string[]): Checked<Record<string, unknown>> {
	if (!isObject(value)) return fail(path === "" ? "(body)" : path);
	const join = (key: string) => (path === "" ? key : `${path}.${key}`);
	for (const key of keys) if (!Object.hasOwn(value, key)) return fail(join(key));
	return Object.keys(value).length === keys.length ? { ok: true, value } : fail(path === "" ? "(body)" : path);
}

/** The report the page defines, rebuilt in the page's key order, or the path of the first failing field. */
export function validateReport(input: unknown): Checked<CrashReport> {
	const top = exactly(input, "", ["v", "kind", "type", "app", "os", "device", "day", "frames"]);
	if (!top.ok) return top;
	const r = top.value;
	if (r.v !== 1) return fail("v");
	if (typeof r.kind !== "string" || !(KINDS as readonly string[]).includes(r.kind)) return fail("kind");
	if (!isText(r.type, TYPE)) return fail("type");
	const app = exactly(r.app, "app", ["version", "build", "patch"]);
	if (!app.ok) return app;
	if (!isText(app.value.version, VERSION)) return fail("app.version");
	if (!isText(app.value.build, BUILD)) return fail("app.build");
	const patch = app.value.patch;
	if (patch !== null && !(typeof patch === "number" && Number.isInteger(patch) && patch >= 0 && patch <= 100_000)) return fail("app.patch");
	const os = exactly(r.os, "os", ["name", "version"]);
	if (!os.ok) return os;
	if (os.value.name !== "ios" && os.value.name !== "android") return fail("os.name");
	if (!isText(os.value.version, OS_VERSION)) return fail("os.version");
	if (!isText(r.device, DEVICE)) return fail("device");
	if (!isText(r.day, DAY_TEXT)) return fail("day");
	if (!Array.isArray(r.frames) || r.frames.length < 1 || r.frames.length > MAX_FRAMES) return fail("frames");
	for (const [i, frame] of r.frames.entries()) {
		if (typeof frame !== "string" || !validFrame(frame)) return fail(`frames[${i}]`);
	}
	return {
		ok: true,
		value: {
			v: 1,
			kind: r.kind as Kind,
			type: r.type,
			app: { version: app.value.version, build: app.value.build, patch },
			os: { name: os.value.name, version: os.value.version },
			device: r.device,
			day: r.day,
			frames: r.frames as string[],
		},
	};
}

// ------------------------------------------------------------------- handler

/** `YYYY-MM-DD` of a Unix-ms time, in UTC. */
export const utcDay = (ms: number): string => new Date(ms).toISOString().slice(0, 10);

export const INSERT_REPORT = "INSERT INTO crash_reports (received_day, kind, type, app_version, report) VALUES (?1, ?2, ?3, ?4, ?5)";

/** Deletes reports received more than KEEP_DAYS ago (the cron); the number deleted. */
export async function purgeCrashReports(env: Env, now: number): Promise<number> {
	const done = await env.DB.prepare("DELETE FROM crash_reports WHERE received_day < ?1").bind(utcDay(now - KEEP_DAYS * DAY)).run();
	return done.meta.changes ?? 0;
}

/** POST /v1/crash-reports: 204 stored, 400 invalid, 413 over 32 KiB, 429 too many from this address. */
export async function submit(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> {
	const limited = await count(env.DB, "crash-ip", await ipKey(keys, "crash-ip", ipGroup(clientIp(request))), now);
	if (limited) return limited;
	const body = await readJson(request, MAX_REPORT_BYTES);
	if (!body.ok) return body.response;
	const report = validateReport(body.value);
	if (!report.ok) return error(400, "bad_request", `Invalid report: ${report.field}.`, { field: report.field });
	const r = report.value;
	await env.DB.prepare(INSERT_REPORT).bind(utcDay(now), r.kind, r.type, r.app.version, JSON.stringify(r)).run();
	return new Response(null, { status: 204 });
}
