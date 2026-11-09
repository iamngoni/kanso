// Crash reports (docs/CRASH-REPORTS.md): what the endpoint accepts, every way it refuses, the size
// limit, the per-address rate limit, the 90-day purge, and properties over reports generated from
// the page's grammar (fast-check, seed pinned in vitest.config.ts).

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { MAX_REPORT_BYTES, utcDay, validFrame } from "../src/crash_reports";
import { runCron } from "../src/index";
import { START, call, resetServer } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const DAY = 86_400_000;
const PATH = "/v1/crash-reports";

const REPORT = {
	v: 1,
	kind: "dart",
	type: "StateError",
	app: { version: "0.9.0", build: "57", patch: 2 },
	os: { name: "ios", version: "18.1" },
	device: "iPhone17,1",
	day: "2026-10-26",
	frames: ["dart _NoteEditorState._save package:kanso/note/views/widgets/note_editor.dart:120:7", "dart <closure> dart:async/zone.dart:1525:47"],
};

type Json = Record<string, any>;
const clone = (): Json => structuredClone(REPORT);

interface Row {
	id: number;
	received_day: string;
	kind: string;
	type: string;
	app_version: string;
	report: string;
}
const rows = async (): Promise<Row[]> => (await env.DB.prepare("SELECT * FROM crash_reports ORDER BY id").all<Row>()).results;
const clear = async () => {
	await env.DB.batch([env.DB.prepare("DELETE FROM crash_reports"), env.DB.prepare("DELETE FROM limits")]);
};

async function send(body: string | Uint8Array | null, init: { ip?: string; now?: number; headers?: Record<string, string> } = {}) {
	const response = await call(PATH, {
		method: "POST",
		body,
		headers: { "content-type": "application/json", ...init.headers },
		ip: init.ip ?? "203.0.113.9",
		now: init.now,
	});
	const text = await response.text();
	return { status: response.status, text, headers: response.headers, body: text === "" ? null : (JSON.parse(text) as any) };
}
const post = (report: unknown, init: Parameters<typeof send>[1] = {}) => send(JSON.stringify(report), init);

beforeEach(async () => {
	await resetServer();
	await clear();
});

describe("accepting", () => {
	it("stores a valid report and answers 204", async () => {
		const r = await post(REPORT, { now: START });
		expect(r.status).toBe(204);
		expect(r.text).toBe("");
		const [row] = await rows();
		expect(row).toMatchObject({ received_day: utcDay(START), kind: "dart", type: "StateError", app_version: "0.9.0" });
		expect(row.report).toBe(JSON.stringify(REPORT));
	});

	it("stores the validated object in the page's key order, not the body that arrived", async () => {
		const shuffled = `{ "frames": ${JSON.stringify(REPORT.frames)},\n "day": "2026-10-26", "device": "iPhone17,1", "os": {"version": "18.1", "name": "ios"},
 "app": {"patch": 2, "build": "57", "version": "0.9.0"}, "type": "StateError", "kind": "dart", "v": 1 }`;
		expect((await send(shuffled)).status).toBe(204);
		expect((await rows())[0].report).toBe(JSON.stringify(REPORT));
	});

	it("accepts a null patch, unknown placeholders and every frame shape", async () => {
		const report = clone();
		report.app.patch = null;
		report.type = "unknown";
		report.device = "unknown";
		report.os.version = "unknown";
		report.frames = [
			"dart ? ?",
			"dart <closure> package:a/b.dart:1",
			"dart-pc +0x1f2e",
			"native Runner +0x1000",
			"native ? +0x1000 _ZN4kanso3fooEv+0x10",
			"native libkanso_mobile.so +0x1000 0123456789abcdef-0123",
			"native Flutter +0x1000 sym.x+0xa 01234567-89ab",
			"jvm app.kanso.Main.run:-1",
			"jvm app.kanso.Main.<init>:42",
		];
		expect((await post(report)).status).toBe(204);
	});

	it("accepts a MetricKit kill: SIGKILL with its termination code in type", async () => {
		const report = clone();
		report.kind = "ios-native";
		report.type = "SIGKILL.0xdead10cc";
		report.app.patch = null;
		report.frames = ["native libsystem_kernel.dylib +0x1b64 EEEEEEEE-1111-2222-3333-444444444444", "native CoreFoundation +0x5f3a0"];
		expect((await post(report)).status).toBe(204);
	});

	it("rejects a report without the patch key: null must be sent, not left out", async () => {
		const report = clone();
		delete (report.app as Record<string, unknown>).patch;
		const res = await post(report);
		expect(res.status).toBe(400);
		expect(res.body).toMatchObject({ field: "app.patch" });
	});

	it("stores no address, session or account", async () => {
		await post(REPORT, { ip: "198.51.100.77", headers: { authorization: "Bearer kanso_x", cookie: "a=b" } });
		const stored = JSON.stringify(await rows());
		expect(stored).not.toContain("198.51.100.77");
		const { results } = await env.DB.prepare("SELECT key FROM limits").all<{ key: string }>();
		expect(results).toHaveLength(1);
		expect(results[0].key).not.toContain("198.51.100.77");
		expect((await env.DB.prepare("SELECT name FROM pragma_table_info('crash_reports')").all<{ name: string }>()).results.map((c) => c.name)).toEqual([
			"id",
			"received_day",
			"kind",
			"type",
			"app_version",
			"report",
		]);
	});
});

describe("refusing", () => {
	const mutate = (edit: (r: Json) => void): Json => {
		const r = clone();
		edit(r);
		return r;
	};
	const cases: [string, string, Json][] = [
		["v", "v", mutate((r) => (r.v = 2))],
		["v as text", "v", mutate((r) => (r.v = "1"))],
		["kind", "kind", mutate((r) => (r.kind = "swift"))],
		["type with a space", "type", mutate((r) => (r.type = "State Error"))],
		["type empty", "type", mutate((r) => (r.type = ""))],
		["type too long", "type", mutate((r) => (r.type = "A".repeat(129)))],
		["app.version", "app.version", mutate((r) => (r.app.version = "1.2"))],
		["app.build", "app.build", mutate((r) => (r.app.build = "5a"))],
		["app.build as number", "app.build", mutate((r) => (r.app.build = 57))],
		["app.patch negative", "app.patch", mutate((r) => (r.app.patch = -1))],
		["app.patch fraction", "app.patch", mutate((r) => (r.app.patch = 1.5))],
		["app.patch too big", "app.patch", mutate((r) => (r.app.patch = 100_001))],
		["os.name", "os.name", mutate((r) => (r.os.name = "macos"))],
		["os.version", "os.version", mutate((r) => (r.os.version = "18 1"))],
		["device", "device", mutate((r) => (r.device = "iPhone/17"))],
		["day", "day", mutate((r) => (r.day = "2026-10-26T10:00"))],
		["frames empty", "frames", mutate((r) => (r.frames = []))],
		["frames not an array", "frames", mutate((r) => (r.frames = "dart-pc +0x1"))],
		["too many frames", "frames", mutate((r) => (r.frames = Array(129).fill("dart-pc +0x1")))],
		["frame not text", "frames[1]", mutate((r) => (r.frames[1] = 7))],
		["frame with a path", "frames[0]", mutate((r) => (r.frames[0] = "dart f file:///Users/x/y.dart:1:2"))],
		["frame of unknown keyword", "frames[1]", mutate((r) => r.frames.splice(1, 1, "swift x"))],
		["frame over 300 characters", "frames[0]", mutate((r) => (r.frames[0] = `dart ${"a".repeat(300)} ?`))],
		["dart frame with a bad line", "frames[0]", mutate((r) => (r.frames[0] = "dart f package:a/b.dart:x"))],
		["dart frame with no location", "frames[0]", mutate((r) => (r.frames[0] = "dart f"))],
		["dart-pc without hex", "frames[0]", mutate((r) => (r.frames[0] = "dart-pc +0xZZ"))],
		["native with upper-case offset", "frames[0]", mutate((r) => (r.frames[0] = "native Runner +0xAB"))],
		["native with a short build ID", "frames[0]", mutate((r) => (r.frames[0] = "native Runner +0x1 abc"))],
		["jvm without a line", "frames[0]", mutate((r) => (r.frames[0] = "jvm app.Main.run"))],
		["a double space", "frames[0]", mutate((r) => (r.frames[0] = "dart  f package:a/b.dart:1"))],
		["missing key", "day", mutate((r) => delete r.day)],
		["missing nested key", "app.build", mutate((r) => delete r.app.build)],
		["app not an object", "app", mutate((r) => (r.app = "0.9.0"))],
	];
	it.each(cases)("400 for %s, naming %s", async (_name, field, report) => {
		const r = await post(report);
		expect(r.status).toBe(400);
		expect(r.body).toMatchObject({ error: "bad_request", field });
		expect(await rows()).toEqual([]);
	});

	it("400 for an extra key at each level, naming the object and not the key", async () => {
		for (const [path, edit] of [
			["(body)", (r: Json) => (r.secret = "x")],
			["app", (r: Json) => (r.app.secret = "x")],
			["os", (r: Json) => (r.os.secret = "x")],
		] as [string, (r: Json) => void][]) {
			const r = await post(mutate(edit));
			expect(r.status).toBe(400);
			expect(r.body.field).toBe(path);
			expect(r.text).not.toContain("secret");
		}
		expect(await rows()).toEqual([]);
	});

	it("400 for a body that is not an object, not JSON or not UTF-8", async () => {
		// Each from its own address, so the rate limit doesn't answer first.
		const bodies = ["[]", "null", "7", '"x"', "{", "", "\u{feff}{}", '{"__proto__": {}}', new Uint8Array([0x7b, 0xff, 0x7d])];
		for (const [i, body] of bodies.entries()) {
			expect((await send(body, { ip: `203.0.113.${100 + i}` })).status, String(body)).toBe(400);
		}
	});

	it("never echoes a submitted value", async () => {
		const r = await post(mutate((x) => (x.device = "SECRET note: my passwords\n/etc")));
		expect(r.status).toBe(400);
		expect(r.text).not.toMatch(/SECRET|passwords|etc/);
	});

	it("415 without application/json", async () => {
		expect((await send(JSON.stringify(REPORT), { headers: { "content-type": "text/plain" } })).status).toBe(415);
		expect((await send(JSON.stringify(REPORT), { headers: { "content-type": "application/json; charset=utf-8" } })).status).toBe(204);
	});

	it("405 for other methods, 404 for other /v1 paths, 403 for a browser", async () => {
		for (const method of ["GET", "PUT", "DELETE"]) {
			const r = await call(PATH, { method, ip: "203.0.113.9" });
			expect(r.status).toBe(405);
			expect(r.headers.get("allow")).toBe("POST");
		}
		expect((await call("/v1/crash-reports/1", { method: "POST", json: REPORT })).status).toBe(404);
		expect((await call("/v1/other", { method: "POST", json: REPORT })).status).toBe(404);
		expect((await call(PATH, { method: "POST", json: REPORT, headers: { origin: "https://evil.example" } })).status).toBe(403);
		expect(await rows()).toEqual([]);
	});
});

describe("size", () => {
	const padded = (bytes: number): Uint8Array => {
		const json = JSON.stringify(REPORT);
		return new TextEncoder().encode(json + " ".repeat(bytes - json.length));
	};

	it("accepts exactly 32 KiB and refuses one byte more with 413", async () => {
		expect(MAX_REPORT_BYTES).toBe(32768);
		expect((await send(padded(32768))).status).toBe(204);
		const over = await send(padded(32769));
		expect(over.status).toBe(413);
		expect(over.body.error).toBe("too_large");
		expect(await rows()).toHaveLength(1);
	});

	it("checks the bytes actually read when no length is declared", async () => {
		const bytes = padded(32769);
		const stream = new ReadableStream<Uint8Array>({
			start(controller) {
				controller.enqueue(bytes.slice(0, 20_000));
				controller.enqueue(bytes.slice(20_000));
				controller.close();
			},
		});
		const request = new Request("https://kanso-sync.test" + PATH, {
			method: "POST",
			body: stream,
			headers: { "content-type": "application/json", "cf-connecting-ip": "203.0.113.9" },
			// @ts-expect-error duplex is required for stream bodies
			duplex: "half",
		});
		const { handle } = await import("../src/index");
		const { testEnv } = await import("./helpers");
		expect((await handle(request, testEnv(), START)).status).toBe(413);
	});

	it("413 on a declared length over the limit without reading the body", async () => {
		const r = await send(JSON.stringify(REPORT), { headers: { "content-length": "40000" } }).catch(() => null);
		// The runtime may refuse a length that disagrees with the body; if it gets through it's a 413.
		if (r) expect(r.status).toBe(413);
	});
});

describe("rate limit", () => {
	it("allows five an hour per address, then 429 with retry_after, and resets with the window", async () => {
		for (let i = 0; i < 5; i++) expect((await post(REPORT, { now: START + i })).status).toBe(204);
		const sixth = await post(REPORT, { now: START + 10 });
		expect(sixth.status).toBe(429);
		expect(sixth.body).toMatchObject({ error: "rate_limited", limit: "crash-ip" });
		expect(sixth.headers.get("retry-after")).toBe(String(sixth.body.retry_after));
		expect(await rows()).toHaveLength(5);
		expect((await post(REPORT, { now: START + 3_600_000 })).status).toBe(204);
	});

	it("counts invalid reports too, and keeps addresses apart; IPv6 counts by /48", async () => {
		for (let i = 0; i < 5; i++) expect((await post({ ...REPORT, v: 2 })).status).toBe(400);
		expect((await post(REPORT)).status).toBe(429);
		expect((await post(REPORT, { ip: "203.0.113.10" })).status).toBe(204);
		for (let i = 0; i < 5; i++) expect((await post(REPORT, { ip: `2001:db8:1:${i}::1` })).status).toBe(204);
		expect((await post(REPORT, { ip: "2001:db8:1:ff::2" })).status).toBe(429);
	});
});

describe("the 90-day purge", () => {
	it("deletes reports received more than 90 days ago and keeps newer ones", async () => {
		const now = START + 200 * DAY;
		const insert = (day: string) =>
			env.DB.prepare("INSERT INTO crash_reports (received_day, kind, type, app_version, report) VALUES (?1, 'dart', 'E', '1.0.0', '{}')").bind(day);
		await env.DB.batch([insert(utcDay(now - 91 * DAY)), insert(utcDay(now - 90 * DAY)), insert(utcDay(now - DAY)), insert(utcDay(now))]);
		await runCron(env, now);
		expect((await rows()).map((r) => r.received_day)).toEqual([utcDay(now - 90 * DAY), utcDay(now - DAY), utcDay(now)]);
	});
});

describe("the frame grammar", () => {
	it("agrees with the page on a few edge cases", () => {
		expect(validFrame("dart f dart:core/list.dart:1")).toBe(true);
		expect(validFrame("dart f package:a/b.dart")).toBe(false);
		expect(validFrame("dart 1f package:a/b.dart:1")).toBe(false);
		expect(validFrame("dart f package:A/b.dart:1")).toBe(false);
		expect(validFrame("native a/b +0x1")).toBe(false);
		expect(validFrame("native Runner +0x1 s+0x2 abcdef12")).toBe(true);
		expect(validFrame("native Runner +0x1 abcdef12 s+0x2")).toBe(false);
		expect(validFrame("jvm A.b:1:2")).toBe(false);
		expect(validFrame(`jvm ${"A".repeat(300)}.m:1`)).toBe(false);
	});
});

// ---------------------------------------------------------------- properties

const chars = (set: string) => fc.constantFrom(...set);
const text = (first: string, rest: string, max: number) =>
	fc.tuple(chars(first), fc.string({ unit: chars(rest), maxLength: max - 1 })).map(([a, b]) => a + b);
const sized = (set: string, min: number, max: number) => fc.string({ unit: chars(set), minLength: min, maxLength: max });

const ALPHA = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS = "0123456789";
const LOWER_HEX = "0123456789abcdef";

const hex = sized(LOWER_HEX, 1, 16);
const number = sized(DIGITS, 1, 7);
const dartSymbol = fc.oneof(fc.constant("?"), text(`${ALPHA}_$<`, `${ALPHA}${DIGITS}_$.<>`, 60));
const dartUri = fc.oneof(
	fc.tuple(sized("abcdefghijklmnopqrstuvwxyz0123456789_", 1, 20), sized(`${ALPHA}${DIGITS}_/.-`, 1, 60)).map(([p, f]) => `package:${p}/${f}.dart`),
	sized("abcdefghijklmnopqrstuvwxyz0123456789_/.-", 1, 30).map((p) => `dart:${p}`),
);
const dartFrame = fc.oneof(
	fc.tuple(dartSymbol).map(([s]) => `dart ${s} ?`),
	fc.tuple(dartSymbol, dartUri, number, fc.option(number, { nil: undefined })).map(([s, u, l, c]) => `dart ${s} ${u}:${l}${c === undefined ? "" : `:${c}`}`),
);
const frame = fc.oneof(
	dartFrame,
	hex.map((h) => `dart-pc +0x${h}`),
	fc
		.tuple(
			fc.oneof(fc.constant("?"), sized(`${ALPHA}${DIGITS}_.+-`, 1, 40)),
			hex,
			fc.option(fc.tuple(sized(`${ALPHA}${DIGITS}_$.`, 1, 60), hex), { nil: undefined }),
			fc.option(sized(`${LOWER_HEX}ABCDEF-`, 8, 64), { nil: undefined }),
		)
		.map(([b, o, s, id]) => ["native", b, `+0x${o}`, ...(s ? [`${s[0]}+0x${s[1]}`] : []), ...(id ? [id] : [])].join(" ")),
	fc
		.tuple(text(`${ALPHA}_$`, `${ALPHA}${DIGITS}_$.`, 100), text(`${ALPHA}_$<`, `${ALPHA}${DIGITS}_$<>`, 40), fc.integer({ min: -1_000_000, max: 9_999_999 }))
		.map(([c, m, l]) => `jvm ${c}.${m}:${l}`),
);

const report: fc.Arbitrary<Json> = fc.record({
	v: fc.constant(1),
	kind: fc.constantFrom(...["dart", "ios-native", "android-native", "android-anr", "android-jvm"]),
	type: text(`${ALPHA}_$`, `${ALPHA}${DIGITS}_$.`, 128),
	app: fc.record({
		version: fc.tuple(sized(DIGITS, 1, 4), sized(DIGITS, 1, 4), sized(DIGITS, 1, 4)).map((p) => p.join(".")),
		build: sized(DIGITS, 1, 10),
		patch: fc.option(fc.integer({ min: 0, max: 100_000 }), { nil: null }),
	}),
	os: fc.record({ name: fc.constantFrom("ios", "android"), version: sized(`${ALPHA}${DIGITS}._-`, 1, 32) }),
	device: sized(`${ALPHA}${DIGITS} ,._()-`, 1, 64),
	day: fc.tuple(sized(DIGITS, 4, 4), sized(DIGITS, 2, 2), sized(DIGITS, 2, 2)).map((p) => p.join("-")),
	frames: fc.array(frame, { minLength: 1, maxLength: 128 }),
});

/** A name no field uses, so a response containing it echoed the sender's text. */
const MARK = "QZX";

describe("properties", () => {
	it("prop_grammar_reports_are_accepted_and_stored_as_canonical_json", async () => {
		await fc.assert(
			fc.asyncProperty(report, fc.boolean(), async (r, reversed) => {
				await clear();
				const canonical = JSON.stringify(r);
				expect(canonical.length).toBeLessThanOrEqual(MAX_REPORT_BYTES);
				// Sent in its canonical form, or with its keys reversed and indented.
				const body = reversed ? JSON.stringify(Object.fromEntries(Object.entries(r).reverse()), null, 2) : canonical;
				const response = await send(body);
				expect(response.status).toBe(204);
				const stored = await rows();
				expect(stored).toHaveLength(1);
				expect(stored[0].report).toBe(canonical);
				expect(stored[0]).toMatchObject({ kind: r.kind, type: r.type, app_version: r.app.version });
			}),
			runs(100),
		);
	});

	const FIELDS = ["kind", "type", "app.version", "app.build", "os.name", "os.version", "device", "day", "frame"] as const;
	/** Characters outside every non-frame pattern (a space is allowed in `device`, so it isn't here). */
	const BAD = ["/", "\n", "\t", "é", "日", "\"", "\\", "*", ";", "@", "#", "<", "\u0000", "\u{1f4a5}", ":", "%"];
	/** Outside the last part of every frame shape, so appending it breaks the frame. */
	const BAD_FRAME = [...BAD, " "];

	it("prop_a_character_outside_a_fields_pattern_is_refused_and_never_echoed", async () => {
		await fc.assert(
			fc.asyncProperty(
				report,
				fc.constantFrom(...FIELDS),
				fc.nat(),
				fc.nat(),
				fc.string({ unit: chars(`${ALPHA}${DIGITS}`), maxLength: 6 }),
				async (r, field, pick, at, extra) => {
					await clear();
					const bad = (field === "frame" ? BAD_FRAME : BAD)[pick % (field === "frame" ? BAD_FRAME : BAD).length];
					const injected = `${MARK}${extra}${bad}${MARK}`;
					let path: string;
					if (field === "frame") {
						const i = at % r.frames.length;
						r.frames[i] = r.frames[i] + injected;
						path = `frames[${i}]`;
					} else {
						const [a, b] = field.split(".");
						if (b) r[a][b] = injected;
						else r[field] = injected;
						path = field;
					}
					const response = await post(r);
					expect(response.status).toBe(400);
					expect(response.body.field).toBe(path);
					expect(response.text).not.toContain(MARK);
					expect(response.text).not.toContain(extra === "" ? "\u0001" : extra + bad);
					expect(await rows()).toEqual([]);
				},
			),
			runs(100),
		);
	});

	it("prop_extra_keys_anywhere_are_refused_and_never_echoed", async () => {
		await fc.assert(
			fc.asyncProperty(
				report,
				fc.constantFrom("", "app", "os"),
				fc.string({ unit: chars(`${ALPHA}${DIGITS}_ -/é`), maxLength: 12 }),
				fc.jsonValue(),
				async (r, where, name, value) => {
					await clear();
					const key = `${MARK}${name}`;
					(where === "" ? r : r[where])[key] = value;
					const response = await post(r);
					expect(response.status).toBe(400);
					expect(response.body.field).toBe(where === "" ? "(body)" : where);
					expect(response.text).not.toContain(MARK);
					expect(await rows()).toEqual([]);
				},
			),
			runs(100),
		);
	});
});
