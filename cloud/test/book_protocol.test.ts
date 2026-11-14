// Old clients and shared notebooks (issue #390, slice 41): the book routes refuse a client whose
// `x-kanso-book-protocol` is missing or below the minimum with a 426 that says how to fix it, and
// leave every other route alone.

import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import * as entry from "../src/index";
import { BOOK_PROTOCOL, BOOK_PROTOCOL_HEADER, CLIENT_TOO_OLD, MIN_BOOK_PROTOCOL, bookProtocolOf } from "../src/protocol";
import { genesisBody, newShareId } from "./book-helpers";
import { type Account, call, callJson, resetServer, signUp } from "./helpers";

let alice: Account;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
});

/** A request with no protocol header at all: what a Kanso before notebook sharing sends. */
const bare = (path: string, init: { method?: string; json?: unknown; token?: string } = {}) =>
	callJson(path, { ...init, headers: { [BOOK_PROTOCOL_HEADER]: "" } });

describe("the shared-notebook protocol check", () => {
	it("answers a client with no header with the fix-it error", async () => {
		const r = await bare("/v2/collab/books", { token: alice.token });
		expect(r.status).toBe(426);
		expect(r.body).toMatchObject({ error: "client_too_old", message: CLIENT_TOO_OLD, min_protocol: MIN_BOOK_PROTOCOL, protocol: null });
		expect(r.body.message).toMatch(/^Update Kanso to use shared notebooks/);
	});

	it("answers an old protocol number with the fix-it error, and names it", async () => {
		const r = await callJson("/v2/collab/books", { token: alice.token, headers: { [BOOK_PROTOCOL_HEADER]: "0" } });
		expect(r.status).toBe(426);
		expect(r.body).toMatchObject({ error: "client_too_old", protocol: 0, min_protocol: MIN_BOOK_PROTOCOL });
		expect(r.body.message).toContain("Update Kanso");
	});

	it("treats a malformed header as too old", async () => {
		for (const value of ["two", "-1", "1.5", "1 2", "9999999"]) {
			const r = await callJson("/v2/collab/books", { token: alice.token, headers: { [BOOK_PROTOCOL_HEADER]: value } });
			expect(r.status, value).toBe(426);
		}
	});

	it("checks every book route, before the session and the body", async () => {
		const id = newShareId();
		const body = (await genesisBody(id, alice, [])).body;
		const routes: [string, string][] = [
			["POST", "/v2/collab/books"],
			["GET", `/v2/collab/books/${id}`],
			["DELETE", `/v2/collab/books/${id}`],
			["POST", `/v2/collab/books/${id}/accept`],
			["DELETE", `/v2/collab/books/${id}/membership`],
			["POST", `/v2/collab/books/${id}/epoch`],
			["GET", `/v2/collab/books/${id}/manifest`],
			["POST", `/v2/collab/books/${id}/notes/${newShareId()}`],
			["GET", `/v2/collab/books/${id}/live`],
			["POST", `/v2/collab/books/${id}/links`],
		];
		for (const [method, path] of routes) {
			const r = await bare(path, { method, ...(method === "POST" ? { json: body } : {}) });
			expect(r.status, `${method} ${path}`).toBe(426);
			expect(r.body.error).toBe("client_too_old");
		}
	});

	it("lets a current client through to the route's own answers", async () => {
		expect((await callJson("/v2/collab/books", { token: alice.token })).status).toBe(200);
		expect((await callJson("/v2/collab/books", {})).status).toBe(401);
		const current = await call("/v2/collab/books", { token: alice.token, headers: { [BOOK_PROTOCOL_HEADER]: String(BOOK_PROTOCOL + 1) } });
		expect(current.status).toBe(200);
	});

	it("leaves every other route alone, so notes sync on an old Kanso", async () => {
		expect((await bare("/v2/collab/shares", { token: alice.token })).status).toBe(200);
		expect((await bare("/v2/changes?since=0", { token: alice.token })).status).toBe(200);
	});

	it("reads the header as a plain integer", () => {
		expect(bookProtocolOf(null)).toBeNull();
		expect(bookProtocolOf("")).toBeNull();
		expect(bookProtocolOf(" 3 ")).toBe(3);
		expect(bookProtocolOf("03")).toBe(3);
		expect(bookProtocolOf("0x3")).toBeNull();
	});

	it("parses exactly the plain integers, and round-trips every level (property)", () => {
		fc.assert(
			fc.property(fc.integer({ min: 0, max: 999_999 }), fc.constantFrom("", " ", "  "), (level, pad) => {
				expect(bookProtocolOf(`${pad}${level}${pad}`)).toBe(level);
			}),
		);
		fc.assert(
			fc.property(fc.string({ maxLength: 12 }), (text) => {
				const level = bookProtocolOf(text);
				if (/^[0-9]{1,6}$/.test(text.trim())) expect(level).toBe(Number(text.trim()));
				else expect(level).toBeNull();
				// Whatever it reads is a level the check can compare: never NaN, never negative.
				if (level !== null) expect(Number.isInteger(level) && level >= 0).toBe(true);
			}),
		);
	});

	it("refuses exactly the levels below the minimum (property)", async () => {
		await fc.assert(
			fc.asyncProperty(fc.integer({ min: 0, max: MIN_BOOK_PROTOCOL + 5 }), async (level) => {
				const r = await call("/v2/collab/books", { token: alice.token, headers: { [BOOK_PROTOCOL_HEADER]: String(level) } });
				expect(r.status).toBe(level < MIN_BOOK_PROTOCOL ? 426 : 200);
			}),
			{ numRuns: 12 },
		);
	});

	it("keeps the Worker's entry module to handlers and classes, so workerd starts", () => {
		// A constant exported from src/index.ts passes these tests and then stops `wrangler dev`
		// and deploys with "Incorrect type for map entry".
		for (const [name, value] of Object.entries(entry)) {
			if (name === "default") continue;
			expect(typeof value, name).toBe("function");
		}
	});
});
