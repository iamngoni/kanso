// Usage counts (docs/METRICS.md): what the endpoint accepts and passes on to PostHog, every way it
// refuses, the size limit, the per-address rate limit, PostHog's failures, and properties over
// batches generated from the page's lists (fast-check, seed pinned in vitest.config.ts).

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { utcDay } from "../src/crash_reports";
import { type Env, handle } from "../src/index";
import { FEATURES, FORMS, MAX_BATCH_BYTES, NOTES, PLATFORMS, SOURCES, SYNCS } from "../src/metrics";
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

beforeEach(async () => {
	await resetServer();
	await clear();
	sent = [];
	answer = async () => Response.json({ status: "Ok" });
	vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
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
					distinct_id: ID,
					timestamp: `${TODAY}T12:00:00Z`,
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
					timestamp: `${TODAY}T12:00:00Z`,
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
		expect(sent[0].body.batch.map((e: Json) => e.timestamp)).toEqual([-2, -1, 0, 1, 2].map((d) => `${utcDay(START + d * DAY)}T12:00:00Z`));
		expect(sent[0].body.batch[0].properties).not.toHaveProperty("source");
		// The Worker's day is its UTC day, whatever the hour.
		expect((await post(batch, { now: START + DAY - 1 })).status).toBe(204);
		expect((await post(batch, { now: START + DAY })).body.field).toBe("events[0].day");
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
		["event name", "events[0].event", mutate((b) => (b.events[0].event = "app_opened"))],
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
		["feature", `${P1}.feature`, feature((p) => ((p.feature = "search"), delete p.source))],
		["source", `${P1}.source`, feature((p) => (p.source = "roam"))],
		["import without a source", `${P1}.source`, feature((p) => delete p.source)],
		["a source without import", P1, feature((p) => (p.feature = "export"))],
		["app_active props on feature_used", `${P1}.feature`, mutate((b) => (b.events[1].props = structuredClone(b.events[0].props)))],
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
	it("allows 60 an hour per address, then 429 with retry_after, and resets with the window", async () => {
		for (let i = 0; i < 60; i++) expect((await post(BATCH, { now: START + i })).status).toBe(204);
		const over = await post(BATCH, { now: START + 100 });
		expect(over.status).toBe(429);
		expect(over.body).toMatchObject({ error: "rate_limited", limit: "metrics-ip" });
		expect(over.headers.get("retry-after")).toBe(String(over.body.retry_after));
		expect(sent).toHaveLength(60);
		expect((await post(BATCH, { now: START + 3_600_000 })).status).toBe(204);
	});

	it("counts invalid batches too, keeps addresses apart, and is separate from crash reports", async () => {
		for (let i = 0; i < 60; i++) expect((await post({ ...BATCH, v: 2 })).status).toBe(400);
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
const event: fc.Arbitrary<Json> = fc.oneof(
	fc.record({
		event: fc.constant("app_active"),
		day,
		props: fc
			.tuple(platformForm, version, osVersion, fc.constantFrom(...SYNCS), fc.boolean(), fc.constantFrom(...NOTES))
			.map(([pf, app_version, os_version, sync, ai, notes]) => ({ ...pf, app_version, os_version, sync, ai, notes })),
	}),
	fc.record({
		event: fc.constant("feature_used"),
		day,
		props: fc
			.tuple(platformForm, version, fc.constantFrom(...FEATURES), fc.constantFrom(...SOURCES))
			.map(([pf, app_version, feature, source]) => ({ ...pf, app_version, feature, ...(feature === "import" ? { source } : {}) })),
	}),
);
const batch: fc.Arbitrary<Json> = fc.record({
	v: fc.constant(1),
	id: fc.tuple(fc.constantFrom("a", "i"), fc.string({ unit: fc.constantFrom(..."0123456789abcdef"), minLength: 32, maxLength: 32 })).map(([k, h]) => `${k}_${h}`),
	events: fc.array(event, { minLength: 1, maxLength: 32 }),
});

/** A name no field uses, so a response containing it echoed the sender's text. */
const MARK = "QZX";

describe("properties", () => {
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
					expect(forwarded[i]).toMatchObject({ event: e.event, distinct_id: b.id, timestamp: `${e.day}T12:00:00Z` });
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
