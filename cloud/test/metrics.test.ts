// Usage counts (docs/METRICS.md): what the endpoint accepts and passes on to PostHog, every way it
// refuses, the size limit, the per-address rate limit, PostHog's failures, and properties over
// batches generated from the page's lists (fast-check, seed pinned in vitest.config.ts).

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { utcDay } from "../src/crash_reports";
import { type Env, handle } from "../src/index";
import {
	ACTIVE_OPTIONAL,
	ATTACHMENTS_MB,
	CHANNELS,
	COUNTS,
	DURATIONS,
	EDITOR_MODES,
	FEATURES,
	FORMS,
	MAX_BATCH_BYTES,
	NOTEBOOKS,
	NOTES,
	PLATFORMS,
	PLUGINS,
	SOURCES,
	SYNCS,
	SYNC_ERRORS,
	THEME_MODES,
	activeUuid,
	eventTimestamp,
	resetKeyChecks,
} from "../src/metrics";
import { ORIGIN, START, call, resetServer, testEnv } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const DAY = 86_400_000;
const PATH = "/v1/metrics";
const KEY = "phc_test_key";
const HOST = "https://eu.i.posthog.com";
const TODAY = utcDay(START);
const ID = "a_3f9c0d4e5b6a79881726354433221100";

const BATCH = {
	v: 1,
	id: ID,
	events: [
		{
			event: "app_active",
			day: TODAY,
			props: { platform: "macos", form: "desktop", app_version: "0.9.0", os_version: "15.1", sync: "kanso", ai: false, notes: "100-999" },
		},
		{
			event: "feature_used",
			day: TODAY,
			props: { platform: "macos", form: "desktop", app_version: "0.9.0", feature: "import", source: "obsidian" },
		},
	],
};

const COMMON = { platform: "ios", form: "phone", app_version: "0.10.0" };
const OPENED = { event: "app_opened", day: TODAY, props: COMMON };
const SESSION = { event: "session", day: TODAY, props: { ...COMMON, duration: "5-15m" } };
const EXTRAS = {
	locale_lang: "pt",
	theme_mode: "dark",
	theme: "kanso-dark",
	editor_mode: "live",
	plugins: "1-2",
	notebooks: "5-19",
	attachments_mb: "100-1000",
	sync_errors: "0",
	channel: "stable",
};

type Json = Record<string, any>;
const clone = (): Json => structuredClone(BATCH);
const posthogEnv = (overrides: Partial<Env> = {}): Env => testEnv({ POSTHOG_KEY: KEY, POSTHOG_HOST: HOST, ...overrides });

interface Sent {
	url: string;
	init: RequestInit | undefined;
	body: any;
}
let sent: Sent[];
let answer: () => Promise<Response>;
let checks: Sent[];
let flags: () => Promise<Response>;

beforeEach(async () => {
	await resetServer();
	await clear();
	sent = [];
	answer = async () => Response.json({ status: "Ok" });
	checks = [];
	flags = async () => Response.json({ flags: {} });
	resetKeyChecks();
	vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
		if (String(input).includes("/flags/")) {
			checks.push({ url: String(input), init, body: JSON.parse(String(init?.body)) });
			return flags();
		}
		sent.push({ url: String(input), init, body: JSON.parse(String(init?.body)) });
		return answer();
	});
});
afterEach(() => vi.restoreAllMocks());

const clear = async () => {
	await env.DB.prepare("DELETE FROM limits").run();
};

interface Send {
	ip?: string;
	now?: number;
	headers?: Record<string, string>;
	env?: Env;
	country?: string | null;
}

async function send(body: string | Uint8Array | null, init: Send = {}) {
	const headers = new Headers({ "content-type": "application/json", "cf-connecting-ip": init.ip ?? "203.0.113.9", ...init.headers });
	const cf = init.country === null ? undefined : { country: init.country ?? "DE" };
	const request = new Request(`${ORIGIN}${PATH}`, { method: "POST", body, headers, cf } as RequestInit);
	const response = await handle(request, init.env ?? posthogEnv(), init.now ?? START);
	const text = await response.text();
	return { status: response.status, text, headers: response.headers, body: text === "" ? null : (JSON.parse(text) as any) };
}
const post = (batch: unknown, init: Send = {}) => send(JSON.stringify(batch), init);

describe("accepting", () => {
	it("answers 204 and passes the batch on to PostHog exactly as the page says", async () => {
		const r = await post(BATCH);
		expect(r.status).toBe(204);
		expect(r.text).toBe("");
		expect(sent).toHaveLength(1);
		expect(sent[0].url).toBe(`${HOST}/batch/`);
		expect(sent[0].init?.method).toBe("POST");
		expect(sent[0].body).toEqual({
			api_key: KEY,
			batch: [
				{
					event: "app_active",
					uuid: await activeUuid(ID, TODAY, "macos", "desktop"),
					distinct_id: ID,
					timestamp: new Date(START).toISOString(),
					properties: {
						platform: "macos",
						form: "desktop",
						app_version: "0.9.0",
						os_version: "15.1",
						sync: "kanso",
						ai: false,
						notes: "100-999",
						country: "DE",
						$process_person_profile: false,
						$geoip_disable: true,
						$lib: "kanso-worker",
					},
				},
				{
					event: "feature_used",
					distinct_id: ID,
					timestamp: new Date(START).toISOString(),
					properties: {
						platform: "macos",
						form: "desktop",
						app_version: "0.9.0",
						feature: "import",
						source: "obsidian",
						country: "DE",
						$process_person_profile: false,
						$geoip_disable: true,
						$lib: "kanso-worker",
					},
				},
			],
		});
	});

	it("passes on no address, header, session or cookie, and stores nothing but the limit's hash", async () => {
		const tables = async () =>
			(await env.DB.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name").all<{ name: string }>()).results.map((t) => t.name);
		const before = await tables();
		await post(BATCH, { ip: "198.51.100.77", headers: { authorization: "Bearer kanso_x", cookie: "a=b", "user-agent": "Kanso/0.9.0", "x-forwarded-for": "198.51.100.77" } });
		expect(sent).toHaveLength(1);
		const forwarded = JSON.stringify(sent[0]);
		for (const leak of ["198.51.100.77", "kanso_x", "a=b", "Kanso/0.9.0", "$ip"]) expect(forwarded).not.toContain(leak);
		expect(new Headers(sent[0].init?.headers).get("x-forwarded-for")).toBeNull();
		expect([...new Headers(sent[0].init?.headers).keys()].sort()).toEqual(["content-type", "user-agent"]);
		const { results } = await env.DB.prepare("SELECT key FROM limits").all<{ key: string }>();
		expect(results).toHaveLength(1);
		expect(results[0].key).toMatch(/^metrics-ip:[0-9a-f]{32}$/);
		expect(await tables()).toEqual(before);
	});

	it("adds the country only when Cloudflare gives a two-letter code", async () => {
		const countryOf = async (country: string | null) => {
			sent = [];
			expect((await post(BATCH, { country })).status).toBe(204);
			return sent[0].body.batch.map((e: Json) => e.properties.country);
		};
		expect(await countryOf("ZW")).toEqual(["ZW", "ZW"]);
		for (const none of [null, "XX", "T1", "", "zw", "ZWE"]) {
			expect(await countryOf(none), String(none)).toEqual([undefined, undefined]);
			expect(Object.keys(sent[0].body.batch[0].properties)).not.toContain("country");
		}
	});

	it("rebuilds props in the page's order from the validated values", async () => {
		const shuffled = `{"events": [{"props": {"notes": "0", "ai": true, "sync": "off", "os_version": "unknown", "app_version": "1.2.3", "form": "tablet", "platform": "android"},
 "day": "${TODAY}", "event": "app_active"}], "id": "i_${"0".repeat(32)}", "v": 1}`;
		expect((await send(shuffled)).status).toBe(204);
		expect(Object.keys(sent[0].body.batch[0].properties)).toEqual([
			"platform",
			"form",
			"app_version",
			"os_version",
			"sync",
			"ai",
			"notes",
			"country",
			"$process_person_profile",
			"$geoip_disable",
			"$lib",
		]);
		expect(sent[0].body.batch[0].distinct_id).toBe(`i_${"0".repeat(32)}`);
	});

	it("accepts a feature without a source, and days up to two either side of the Worker's", async () => {
		const batch = clone();
		batch.events = [-2, -1, 0, 1, 2].map((d) => ({
			event: "feature_used",
			day: utcDay(START + d * DAY),
			props: { platform: "ios", form: "phone", app_version: "0.9.0", feature: "note_created" },
		}));
		expect((await post(batch)).status).toBe(204);
		// Stamped with the receipt time; an earlier day stays on its day; never later than receipt.
		expect(sent[0].body.batch.map((e: Json) => e.timestamp)).toEqual([
			`${utcDay(START - 2 * DAY)}T23:59:59Z`,
			`${utcDay(START - DAY)}T23:59:59Z`,
			new Date(START).toISOString(),
			new Date(START).toISOString(),
			new Date(START).toISOString(),
		]);
		expect(sent[0].body.batch[0].properties).not.toHaveProperty("source");
		// The Worker's day is its UTC day, whatever the hour.
		expect((await post(batch, { now: START + DAY - 1 })).status).toBe(204);
		expect((await post(batch, { now: START + DAY })).body.field).toBe("events[0].day");
	});

	it("accepts the 0.9.0 batch unchanged: no count, none of the new props", async () => {
		expect((await post(BATCH)).status).toBe(204);
		const [active, used] = sent[0].body.batch;
		expect(Object.keys(active.properties).slice(0, 7)).toEqual(["platform", "form", "app_version", "os_version", "sync", "ai", "notes"]);
		expect(used.properties).not.toHaveProperty("count");
	});

	it("accepts every new event and prop exactly as the page defines them, in the page's order", async () => {
		const batch = clone();
		batch.events = [
			{ ...batch.events[0], props: { ...Object.fromEntries(Object.entries(EXTRAS).reverse()), ...batch.events[0].props } },
			OPENED,
			SESSION,
			{ event: "feature_used", day: TODAY, props: { ...COMMON, feature: "note_edited", count: "21+" } },
			{ event: "feature_used", day: TODAY, props: { ...COMMON, feature: "import", source: "bear", count: "2-5" } },
		];
		expect((await post(batch)).status).toBe(204);
		const forwarded = sent[0].body.batch as Json[];
		const own = (e: Json) => Object.keys(e.properties).filter((k) => !k.startsWith("$") && k !== "country");
		expect(own(forwarded[0])).toEqual(["platform", "form", "app_version", "os_version", "sync", "ai", "notes", ...ACTIVE_OPTIONAL]);
		expect(own(forwarded[1])).toEqual(["platform", "form", "app_version"]);
		expect(own(forwarded[2])).toEqual(["platform", "form", "app_version", "duration"]);
		expect(own(forwarded[3])).toEqual(["platform", "form", "app_version", "feature", "count"]);
		expect(own(forwarded[4])).toEqual(["platform", "form", "app_version", "feature", "source", "count"]);
		// Any subset of the optional props is fine.
		sent = [];
		const some = clone();
		some.events[0].props.theme = "custom";
		some.events[0].props.locale_lang = "other";
		expect((await post(some)).status).toBe(204);
	});

	it("checks and sends nothing without POSTHOG_KEY", async () => {
		expect((await post(BATCH, { env: testEnv({ POSTHOG_HOST: HOST }) })).status).toBe(204);
		expect((await post(BATCH, { env: testEnv({ POSTHOG_HOST: HOST, POSTHOG_KEY: " " }) })).status).toBe(204);
		expect((await post({ ...BATCH, v: 2 }, { env: testEnv({ POSTHOG_HOST: HOST }) })).status).toBe(400);
		expect(sent).toEqual([]);
	});

	it("answers 204 whether or not PostHog takes it", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		answer = async () => new Response("down", { status: 503 });
		expect((await post(BATCH)).status).toBe(204);
		answer = async () => {
			throw new DOMException("timed out", "TimeoutError");
		};
		expect((await post(BATCH)).status).toBe(204);
		expect(sent).toHaveLength(2);
		expect(sent[0].init?.signal).toBeInstanceOf(AbortSignal);
		const lines = log.mock.calls.map((c) => String(c[0]));
		expect(lines).toEqual([JSON.stringify({ route: "posthog", outcome: 503 }), JSON.stringify({ route: "posthog", outcome: "TimeoutError" })]);
		for (const line of lines) expect(line).not.toContain(ID);
	});
});

describe("resends", () => {
	it("answer before PostHog does when the Worker can send after the response", async () => {
		let release: (r: Response) => void = () => {};
		answer = () => new Promise((resolve) => (release = resolve));
		const deferred: Promise<unknown>[] = [];
		const headers = { "content-type": "application/json", "cf-connecting-ip": "203.0.113.9" };
		const request = new Request(`${ORIGIN}${PATH}`, { method: "POST", body: JSON.stringify(BATCH), headers });
		const response = await handle(request, posthogEnv(), START, undefined, (work) => deferred.push(work));
		expect(response.status).toBe(204);
		expect(deferred).toHaveLength(1);
		// The forward starts after the answer; PostHog answers only once it has the batch.
		await vi.waitFor(() => expect(sent).toHaveLength(1));
		release(Response.json({ status: "Ok" }));
		await Promise.all(deferred);
	});

	it("give a resent app_active the same uuid, and leave other events to PostHog", async () => {
		await post(BATCH);
		await post(BATCH);
		const [first, second] = sent.map((s) => s.body.batch as Json[]);
		expect(first[0].uuid).toBe(second[0].uuid);
		expect(first[1]).not.toHaveProperty("uuid");
	});
});

describe("the PostHog key", () => {
	it("is checked once against /flags, which refuses an unknown key that /batch/ would swallow", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		flags = async () => Response.json({ type: "authentication_error" }, { status: 401 });
		expect((await post(BATCH)).status).toBe(204);
		expect((await post(BATCH)).status).toBe(204);
		expect(checks).toHaveLength(1);
		expect(checks[0].url).toBe(`${HOST}/flags/?v=2`);
		expect(checks[0].body).toEqual({ api_key: KEY, distinct_id: "kanso-worker-key-check" });
		expect(sent).toHaveLength(2);
		const lines = log.mock.calls.map((c) => String(c[0]));
		expect(lines).toEqual([JSON.stringify({ route: "posthog", outcome: "key_rejected" })]);
		for (const line of lines) expect(line).not.toContain(KEY);
	});

	it("logs nothing for a key /flags accepts, and a count for each batch PostHog takes", async () => {
		const error = vi.spyOn(console, "error").mockImplementation(() => {});
		const info = vi.spyOn(console, "log").mockImplementation(() => {});
		expect((await post(BATCH)).status).toBe(204);
		expect(error).not.toHaveBeenCalled();
		const lines = info.mock.calls.map((c) => String(c[0]));
		expect(lines).toEqual([JSON.stringify({ route: "posthog", outcome: "ok", events: (BATCH.events as unknown[]).length })]);
		for (const line of lines) expect(line).not.toContain(ID);
	});

	it("checks again after a check that couldn't reach PostHog", async () => {
		vi.spyOn(console, "error").mockImplementation(() => {});
		flags = async () => {
			throw new DOMException("timed out", "TimeoutError");
		};
		await post(BATCH);
		flags = async () => Response.json({ flags: {} });
		await post(BATCH);
		expect(checks).toHaveLength(2);
	});
});

describe("refusing", () => {
	const mutate = (edit: (b: Json) => void): Json => {
		const b = clone();
		edit(b);
		return b;
	};
	const active = (edit: (p: Json) => void) => mutate((b) => edit(b.events[0].props));
	const feature = (edit: (p: Json) => void) => mutate((b) => edit(b.events[1].props));
	const P0 = "events[0].props";
	const P1 = "events[1].props";
	const opened = (edit: (p: Json) => void) => mutate((b) => ((b.events[0] = structuredClone(OPENED)), edit(b.events[0].props)));
	const session = (edit: (p: Json) => void) => mutate((b) => ((b.events[0] = structuredClone(SESSION)), edit(b.events[0].props)));
	const NEW_CASES: [string, string, Json][] = [
		...(
			[
				["locale_lang", "english"],
				["locale_lang", "EN"],
				["locale_lang", 5],
				["theme_mode", "auto"],
				["theme", "My Theme"],
				["theme", "-dark"],
				["theme", "a".repeat(33)],
				["editor_mode", "wysiwyg"],
				["plugins", "2"],
				["notebooks", "1-5"],
				["attachments_mb", "10"],
				["sync_errors", "7"],
				["channel", "nightly"],
			] as [string, unknown][]
		).map(([key, value]): [string, string, Json] => [`optional ${key} out of its list`, `${P0}.${key}`, active((p) => (p[key] = value))]),
		["app_opened with an extra prop", P0, opened((p) => (p.duration = "<1m"))],
		["app_opened without a form", `${P0}.form`, opened((p) => delete p.form)],
		["session without a duration", `${P0}.duration`, session((p) => delete p.duration)],
		["session duration out of its list", `${P0}.duration`, session((p) => (p.duration = "2m"))],
		["session with an extra prop", P0, session((p) => (p.notes = "0"))],
		["count out of its list", `${P1}.count`, feature((p) => (p.count = "5"))],
		["count as a number", `${P1}.count`, feature((p) => (p.count = 3))],
		["an unknown feature", `${P1}.feature`, feature((p) => ((p.feature = "keystrokes"), delete p.source))],
	];
	const cases: [string, string, Json][] = [
		["v", "v", mutate((b) => (b.v = 2))],
		["v as text", "v", mutate((b) => (b.v = "1"))],
		["id without a prefix", "id", mutate((b) => (b.id = "3f9c0d4e5b6a79881726354433221100"))],
		["id with upper-case hex", "id", mutate((b) => (b.id = `a_${"A".repeat(32)}`))],
		["id too short", "id", mutate((b) => (b.id = `i_${"0".repeat(31)}`))],
		["id of another kind", "id", mutate((b) => (b.id = `u_${"0".repeat(32)}`))],
		["events empty", "events", mutate((b) => (b.events = []))],
		["events not an array", "events", mutate((b) => (b.events = b.events[0]))],
		["too many events", "events", mutate((b) => (b.events = Array(33).fill(b.events[1])))],
		["event not an object", "events[1]", mutate((b) => (b.events[1] = "app_active"))],
		["event name", "events[0].event", mutate((b) => (b.events[0].event = "server_stats"))],
		["day with a time", "events[0].day", mutate((b) => (b.events[0].day = `${TODAY}T10:00`))],
		["day not a calendar day", "events[0].day", mutate((b) => (b.events[0].day = "2027-02-30"))],
		["day three days back", "events[0].day", mutate((b) => (b.events[0].day = utcDay(START - 3 * DAY)))],
		["day three days ahead", "events[1].day", mutate((b) => (b.events[1].day = utcDay(START + 3 * DAY)))],
		["day as a number", "events[0].day", mutate((b) => (b.events[0].day = 20270116))],
		["props not an object", P0, mutate((b) => (b.events[0].props = []))],
		["platform", `${P0}.platform`, active((p) => (p.platform = "freebsd"))],
		["form", `${P0}.form`, active((p) => (p.form = "watch"))],
		["a desktop platform on a phone", `${P0}.form`, active((p) => (p.form = "phone"))],
		["app_version", `${P0}.app_version`, active((p) => (p.app_version = "0.9"))],
		["os_version with a space", `${P0}.os_version`, active((p) => (p.os_version = "15 1"))],
		["os_version too long", `${P0}.os_version`, active((p) => (p.os_version = "1".repeat(33)))],
		["sync", `${P0}.sync`, active((p) => (p.sync = "icloud"))],
		["ai as text", `${P0}.ai`, active((p) => (p.ai = "false"))],
		["notes as a count", `${P0}.notes`, active((p) => (p.notes = 120))],
		["notes out of the list", `${P0}.notes`, active((p) => (p.notes = "100-1000"))],
		["missing prop", `${P0}.notes`, active((p) => delete p.notes)],
		["feature", `${P1}.feature`, feature((p) => ((p.feature = "telemetry"), delete p.source))],
		["source", `${P1}.source`, feature((p) => (p.source = "roam"))],
		["import without a source", `${P1}.source`, feature((p) => delete p.source)],
		["a source without import", P1, feature((p) => (p.feature = "export"))],
		["app_active props on feature_used", `${P1}.feature`, mutate((b) => (b.events[1].props = structuredClone(b.events[0].props)))],
		...NEW_CASES,
		["missing day", "events[0].day", mutate((b) => delete b.events[0].day)],
		["missing id", "id", mutate((b) => delete b.id)],
	];
	it.each(cases)("400 for %s, naming %s", async (_name, field, batch) => {
		const r = await post(batch);
		expect(r.status).toBe(400);
		expect(r.body).toMatchObject({ error: "bad_request", field });
		expect(sent).toEqual([]);
	});

	it("refuses the whole batch when one event is bad", async () => {
		const batch = clone();
		batch.events.push({ ...batch.events[1], day: "2020-01-01" });
		expect((await post(batch)).body.field).toBe("events[2].day");
		expect(sent).toEqual([]);
	});

	it("400 for an extra key at each level, naming the object and not the key", async () => {
		for (const [path, edit] of [
			["(body)", (b: Json) => (b.secret = "x")],
			["events[0]", (b: Json) => (b.events[0].secret = "x")],
			["events[0].props", (b: Json) => (b.events[0].props.secret = "x")],
			["events[1].props", (b: Json) => (b.events[1].props.secret = "x")],
		] as [string, (b: Json) => void][]) {
			const r = await post(mutate(edit));
			expect(r.status).toBe(400);
			expect(r.body.field).toBe(path);
			expect(r.text).not.toContain("secret");
		}
		expect(sent).toEqual([]);
	});

	it("400 for a body that is not an object, not JSON or not UTF-8", async () => {
		const bodies = ["[]", "null", "7", '"x"', "{", "", "\u{feff}{}", '{"__proto__": {}}', new Uint8Array([0x7b, 0xff, 0x7d])];
		for (const body of bodies) expect((await send(body)).status, String(body)).toBe(400);
		expect(sent).toEqual([]);
	});

	it("never echoes a submitted value", async () => {
		const r = await post(active((p) => (p.os_version = "SECRET note: my passwords\n/etc")));
		expect(r.status).toBe(400);
		expect(r.text).not.toMatch(/SECRET|passwords|etc/);
	});

	it("415 without application/json", async () => {
		expect((await send(JSON.stringify(BATCH), { headers: { "content-type": "text/plain" } })).status).toBe(415);
		expect((await send(JSON.stringify(BATCH), { headers: { "content-type": "application/json; charset=utf-8" } })).status).toBe(204);
	});

	it("405 for other methods, 404 for other paths under it, 403 for a browser", async () => {
		for (const method of ["GET", "PUT", "DELETE"]) {
			const r = await call(PATH, { method, ip: "203.0.113.9" });
			expect(r.status).toBe(405);
			expect(r.headers.get("allow")).toBe("POST");
		}
		expect((await call("/v1/metrics/1", { method: "POST", json: BATCH })).status).toBe(404);
		expect((await call(PATH, { method: "POST", json: BATCH, headers: { origin: "https://evil.example" } })).status).toBe(403);
		expect(sent).toEqual([]);
	});

	it("503 without a valid pepper", async () => {
		expect((await post(BATCH, { env: posthogEnv({ AUTH_PEPPER: undefined }) })).status).toBe(503);
		expect(sent).toEqual([]);
	});
});

describe("size", () => {
	const padded = (bytes: number): Uint8Array => {
		const json = JSON.stringify(BATCH);
		return new TextEncoder().encode(json + " ".repeat(bytes - json.length));
	};

	it("accepts exactly 16 KiB and refuses one byte more with 413", async () => {
		expect(MAX_BATCH_BYTES).toBe(16384);
		expect((await send(padded(16384))).status).toBe(204);
		const over = await send(padded(16385));
		expect(over.status).toBe(413);
		expect(over.body.error).toBe("too_large");
		expect(sent).toHaveLength(1);
	});

	it("fits 32 of the largest events", async () => {
		const batch = clone();
		const big = {
			event: "app_active",
			day: TODAY,
			props: { platform: "android", form: "tablet", app_version: "9999.9999.9999", os_version: "x".repeat(32), sync: "self_hosted", ai: false, notes: "100-999" },
		};
		batch.events = Array(32).fill(big);
		expect(JSON.stringify(batch).length).toBeLessThan(MAX_BATCH_BYTES);
		expect((await post(batch)).status).toBe(204);
	});
});

describe("rate limit", () => {
	it("allows 600 an hour per address, then 429 with retry_after, and resets with the window", async () => {
		for (let i = 0; i < 600; i++) expect((await post(BATCH, { now: START + i })).status).toBe(204);
		const over = await post(BATCH, { now: START + 100 });
		expect(over.status).toBe(429);
		expect(over.body).toMatchObject({ error: "rate_limited", limit: "metrics-ip" });
		expect(over.headers.get("retry-after")).toBe(String(over.body.retry_after));
		expect(sent).toHaveLength(600);
		expect((await post(BATCH, { now: START + 3_600_000 })).status).toBe(204);
	});

	it("counts invalid batches too, keeps addresses apart, and is separate from crash reports", async () => {
		for (let i = 0; i < 600; i++) expect((await post({ ...BATCH, v: 2 })).status).toBe(400);
		expect((await post(BATCH)).status).toBe(429);
		expect((await post(BATCH, { ip: "203.0.113.10" })).status).toBe(204);
		const crash = await call("/v1/crash-reports", { method: "POST", json: {}, ip: "203.0.113.9" });
		expect(crash.status).toBe(400);
	});
});

// ---------------------------------------------------------------- properties

const version = fc.tuple(fc.nat(9999), fc.nat(9999), fc.nat(9999)).map((p) => p.join("."));
const osVersion = fc.string({ unit: fc.constantFrom(..."0123456789abcXYZ._-"), minLength: 1, maxLength: 32 });
const platformForm = fc.constantFrom(...PLATFORMS).chain((platform) =>
	fc.record({ platform: fc.constant(platform), form: ["macos", "windows", "linux"].includes(platform) ? fc.constant("desktop") : fc.constantFrom(...FORMS) }),
);
const day = fc.integer({ min: -2, max: 2 }).map((d) => utcDay(START + d * DAY));
const common = fc.tuple(platformForm, version).map(([pf, app_version]) => ({ ...pf, app_version }));
const optionalProps = fc
	.record(
		{
			locale_lang: fc.oneof(fc.constant("other"), fc.stringMatching(/^[a-z]{2}$/)),
			theme_mode: fc.constantFrom(...THEME_MODES),
			theme: fc.oneof(fc.constant("custom"), fc.stringMatching(/^[a-z0-9][a-z0-9-]{0,31}$/)),
			editor_mode: fc.constantFrom(...EDITOR_MODES),
			plugins: fc.constantFrom(...PLUGINS),
			notebooks: fc.constantFrom(...NOTEBOOKS),
			attachments_mb: fc.constantFrom(...ATTACHMENTS_MB),
			sync_errors: fc.constantFrom(...SYNC_ERRORS),
			channel: fc.constantFrom(...CHANNELS),
		},
		{ requiredKeys: [] },
	)
	.map((o) => o as Json);
const event: fc.Arbitrary<Json> = fc.oneof(
	fc.record({
		event: fc.constant("app_active"),
		day,
		props: fc
			.tuple(common, osVersion, fc.constantFrom(...SYNCS), fc.boolean(), fc.constantFrom(...NOTES), optionalProps)
			.map(([c, os_version, sync, ai, notes, extra]) => ({ ...c, os_version, sync, ai, notes, ...extra })),
	}),
	fc.record({ event: fc.constant("app_opened"), day, props: common }),
	fc.record({ event: fc.constant("session"), day, props: fc.tuple(common, fc.constantFrom(...DURATIONS)).map(([c, duration]) => ({ ...c, duration })) }),
	fc.record({
		event: fc.constant("feature_used"),
		day,
		props: fc
			.tuple(common, fc.constantFrom(...FEATURES), fc.constantFrom(...SOURCES), fc.option(fc.constantFrom(...COUNTS), { nil: undefined }))
			.map(([c, feature, source, count]) => ({ ...c, feature, ...(feature === "import" ? { source } : {}), ...(count ? { count } : {}) })),
	}),
);
const batch: fc.Arbitrary<Json> = fc.record({
	v: fc.constant(1),
	id: fc.tuple(fc.constantFrom("a", "i"), fc.string({ unit: fc.constantFrom(..."0123456789abcdef"), minLength: 32, maxLength: 32 })).map(([k, h]) => `${k}_${h}`),
	events: fc.array(event, { minLength: 1, maxLength: 32 }),
});

/** A name no field uses, so a response containing it echoed the sender's text. */
const MARK = "QZX";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-8[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
const hexId = fc
	.tuple(fc.constantFrom("a", "i"), fc.string({ unit: fc.constantFrom(..."0123456789abcdef"), minLength: 32, maxLength: 32 }))
	.map(([k, h]) => `${k}_${h}`);
const activeKey = fc.record({ id: hexId, day, pf: platformForm });

describe("properties", () => {
	it("prop_app_active_uuid_is_a_v8_uuid_fixed_by_id_day_platform_and_form", async () => {
		await fc.assert(
			fc.asyncProperty(activeKey, activeKey, async (a, b) => {
				const ua = await activeUuid(a.id, a.day, a.pf.platform, a.pf.form);
				expect(ua).toMatch(UUID);
				expect(await activeUuid(a.id, a.day, a.pf.platform, a.pf.form)).toBe(ua);
				const same = a.id === b.id && a.day === b.day && a.pf.platform === b.pf.platform && a.pf.form === b.pf.form;
				const ub = await activeUuid(b.id, b.day, b.pf.platform, b.pf.form);
				expect(ub === ua).toBe(same);
			}),
			runs(200),
		);
	});


	it("prop_valid_batches_are_accepted_and_forwarded_prop_for_prop", async () => {
		await fc.assert(
			fc.asyncProperty(batch, async (b) => {
				await clear();
				sent = [];
				expect((await post(b)).status).toBe(204);
				expect(sent).toHaveLength(1);
				const forwarded = sent[0].body.batch as Json[];
				expect(forwarded).toHaveLength(b.events.length);
				for (const [i, e] of b.events.entries()) {
					const { country, $process_person_profile, $geoip_disable, $lib, ...props } = forwarded[i].properties;
					expect(props).toEqual(e.props);
					expect({ country, $process_person_profile, $geoip_disable, $lib }).toEqual({ country: "DE", $process_person_profile: false, $geoip_disable: true, $lib: "kanso-worker" });
					expect(forwarded[i]).toMatchObject({ event: e.event, distinct_id: b.id, timestamp: eventTimestamp(e.day, START) });
					expect(Date.parse(forwarded[i].timestamp)).toBeLessThanOrEqual(START);
				}
			}),
			runs(100),
		);
	});

	it("prop_any_value_outside_a_list_or_pattern_is_refused_and_never_echoed", async () => {
		await fc.assert(
			fc.asyncProperty(batch, fc.nat(), fc.nat(), fc.string({ maxLength: 8 }), async (generated, which, pick, extra) => {
				const b = structuredClone(generated);
				await clear();
				sent = [];
				const i = which % b.events.length;
				const keys = Object.keys(b.events[i].props);
				const key = keys[pick % keys.length];
				// `/` is outside every list and pattern.
				b.events[i].props[key] = `${MARK}${extra}/`;
				const r = await post(b);
				expect(r.status).toBe(400);
				// Changing an import's feature leaves its source as an extra key, reported at the props.
				const imported = key === "feature" && keys.includes("source");
				expect(r.body.field).toBe(imported ? `events[${i}].props` : `events[${i}].props.${key}`);
				expect(r.text).not.toContain(MARK);
				expect(sent).toEqual([]);
			}),
			runs(100),
		);
	});

	it("prop_extra_keys_anywhere_are_refused_and_never_echoed", async () => {
		await fc.assert(
			fc.asyncProperty(batch, fc.constantFrom("body", "event", "props"), fc.string({ maxLength: 12 }), fc.jsonValue(), async (generated, where, name, value) => {
				const b = structuredClone(generated);
				await clear();
				sent = [];
				const key = `${MARK}${name}`;
				const target = where === "body" ? b : where === "event" ? b.events[0] : b.events[0].props;
				target[key] = value;
				const r = await post(b);
				expect(r.status).toBe(400);
				expect(r.body.field).toBe(where === "body" ? "(body)" : where === "event" ? "events[0]" : "events[0].props");
				expect(r.text).not.toContain(MARK);
				expect(sent).toEqual([]);
			}),
			runs(100),
		);
	});
});
