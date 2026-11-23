// The feedback credit (docs/FEEDBACK.md › Credit): the tagged payload and the old one, GitHub's login rule,
// the footer's "Reported by" line (a name never mentions anyone; a login does), and the user search the
// "credit me by GitHub username" picker calls. Properties use fast-check, seed pinned in vitest.config.ts.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { type Report, validateReport } from "../src/feedback";
import { type Credit, GITHUB_LOGIN, creditText, parseCredit } from "../src/feedback_credit";
import { bareIssue, creditOf, renderIssue } from "../src/feedback_triage";
import { cleanQuery, resetSearchCache } from "../src/feedback_users";
import { stripCredit } from "../src/github";
import { START, call, callJson, resetServer, signUp } from "./helpers";
import { REPO, TRIAGE_OK, configured, fakeNetwork, newKey, report } from "./feedback-helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

/** A second statement of GitHub's rule, by characters rather than by pattern (the oracle). */
const loginOracle = (s: string): boolean => s.length >= 1 && s.length <= 39 && /^[A-Za-z0-9-]+$/.test(s) && !s.startsWith("-") && !s.endsWith("-") && !s.includes("--");

const alnum = fc.constantFrom(..."abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
/** Logins GitHub accepts: alphanumeric runs joined by single hyphens, 1–39 characters. */
const genLogin = fc
	.array(fc.string({ unit: alnum, minLength: 1, maxLength: 10 }), { minLength: 1, maxLength: 4 })
	.map((parts) => parts.join("-"))
	.filter((s) => s.length <= 39);
/** Strings near the rule's edges: its alphabet plus the characters that must be refused. */
const genLoginish = fc.string({ unit: fc.constantFrom(..."aZ09-_@. é"), minLength: 0, maxLength: 45 });
const genName = fc.string({ unit: fc.constantFrom(..."abcXYZ019 ._-éñ名@"), minLength: 0, maxLength: 45 });
const genCredit: fc.Arbitrary<Credit> = fc.oneof(
	genLogin.map((login): Credit => ({ kind: "github", login })),
	fc
		.string({ unit: fc.constantFrom(..."abcXYZ019._-éñ名"), minLength: 1, maxLength: 39 })
		.filter((n) => n.trim() !== "")
		.map((name): Credit => ({ kind: "name", name: name.normalize("NFC") })),
);

describe("GitHub's login rule", () => {
	it.each([
		["octocat", true],
		["a", true],
		["a-b", true],
		["A1-b2-C3", true],
		["a".repeat(39), true],
		["a".repeat(40), false],
		["", false],
		["-a", false],
		["a-", false],
		["a--b", false],
		["a_b", false],
		["a b", false],
		["a.b", false],
		["@octocat", false],
		["octocat\n", false],
		["名前", false],
	])("%j is %s", (login, ok) => expect(GITHUB_LOGIN.test(login)).toBe(ok));

	it("agrees with the character-by-character oracle on near-miss strings", () =>
		fc.assert(fc.property(genLoginish, (s) => GITHUB_LOGIN.test(s) === loginOracle(s)), runs(500)));

	it("accepts every generated login, and writes it as @login", () =>
		fc.assert(
			fc.property(genLogin, (login) => {
				const parsed = parseCredit({ kind: "github", login });
				expect(parsed).toEqual({ ok: true, value: { kind: "github", login } });
				expect(creditText({ kind: "github", login })).toBe(`@${login}`);
			}),
			runs(300),
		));
});

describe("the credit payload", () => {
	it("takes the tagged name, the tagged login, null, and the old forms of a name", () => {
		expect(parseCredit(null)).toEqual({ ok: true, value: null });
		expect(parseCredit({ kind: "name", name: "Tendai M" })).toEqual({ ok: true, value: { kind: "name", name: "Tendai M" } });
		expect(parseCredit({ kind: "github", login: "octocat" })).toEqual({ ok: true, value: { kind: "github", login: "octocat" } });
		// Older apps: `{name}`, and a bare string.
		expect(parseCredit({ name: "tendai" })).toEqual({ ok: true, value: { kind: "name", name: "tendai" } });
		expect(parseCredit("tendai")).toEqual({ ok: true, value: { kind: "name", name: "tendai" } });
	});

	it.each<[string, unknown, string]>([
		["a mention as a name", { kind: "name", name: "@tendai" }, "credit.name"],
		["a mention as an old name", { name: "@tendai" }, "credit.name"],
		["a mention as a bare string", "@tendai", "credit"],
		["a login with an @", { kind: "github", login: "@octocat" }, "credit.login"],
		["a login with a double hyphen", { kind: "github", login: "a--b" }, "credit.login"],
		["a login over 39 characters", { kind: "github", login: "a".repeat(40) }, "credit.login"],
		["a login that is not text", { kind: "github", login: 7 }, "credit.login"],
		["a name that is not text", { kind: "name", name: ["x"] }, "credit.name"],
		["an unknown kind", { kind: "email", name: "x" }, "credit.kind"],
		["kind github with a name", { kind: "github", name: "octocat" }, "credit.kind"],
		["a login with a name beside it", { kind: "github", login: "octocat", name: "x" }, "credit"],
		["an extra key", { kind: "name", name: "tendai", url: "x" }, "credit"],
		["an empty object", {}, "credit"],
		["a number", 5, "credit"],
		["an array", ["tendai"], "credit"],
	])("refuses %s at %s", (_what, input, field) => expect(parseCredit(input)).toEqual({ ok: false, field }));

	it("round-trips every credit through JSON and the validator", () =>
		fc.assert(
			fc.property(genCredit, (credit) => {
				const wire = JSON.parse(JSON.stringify(credit));
				expect(parseCredit(wire)).toEqual({ ok: true, value: credit });
				const checked = validateReport({ ...report(), credit: wire });
				expect(checked.ok && checked.value.credit).toEqual(credit);
			}),
			runs(300),
		));

	it("reads the old forms of a name as the tagged one (metamorphic)", () =>
		fc.assert(
			fc.property(genName, (name) => {
				const tagged = parseCredit({ kind: "name", name });
				expect(parseCredit({ name })).toEqual(tagged);
				expect(parseCredit(name)).toEqual(tagged.ok ? tagged : { ok: false, field: "credit" });
			}),
			runs(300),
		));

	it("never accepts a name with an @, and never writes one for a name", () =>
		fc.assert(
			fc.property(genName, (name) => {
				const parsed = parseCredit({ kind: "name", name });
				if (name.includes("@")) expect(parsed.ok).toBe(false);
				if (parsed.ok && parsed.value) expect(creditText(parsed.value) ?? "").not.toContain("@");
			}),
			runs(300),
		));

	it("strips an @ from a stored name even though intake would have refused it", () => {
		expect(creditText({ kind: "name", name: "@everyone" })).toBe("everyone");
		expect(creditText({ kind: "name", name: "@@" })).toBeNull();
		expect(creditText({ kind: "github", login: "-bad" })).toBeNull();
	});
});

describe("the filed issue's footer", () => {
	const withCredit = (credit: unknown): Report => ({ ...report(), credit }) as Report;

	it("mentions a GitHub login with @login", () => {
		const r = withCredit({ kind: "github", login: "octocat" });
		expect(creditOf(r)).toBe("@octocat");
		expect(bareIssue(r).body).toMatch(/ · triage by Haiku · Reported by @octocat<\/sub>$/);
		expect(renderIssue({ ...TRIAGE_OK, category: "bug" } as never, r).body).toContain(" · Reported by @octocat</sub>");
	});

	it("writes a plain name as itself, old and tagged", () => {
		for (const credit of [{ kind: "name", name: "tendai" }, { name: "tendai" }]) {
			expect(bareIssue(withCredit(credit)).body).toContain(" · Reported by tendai</sub>");
		}
	});

	it("leaves the credit out when there is none, or it is unsafe", () => {
		expect(bareIssue(withCredit(null)).body).not.toContain("Reported by");
		expect(bareIssue(withCredit({ kind: "name", name: "www.evil.example" })).body).not.toContain("Reported by");
		expect(bareIssue(withCredit({ kind: "github", login: "a--b" })).body).not.toContain("Reported by");
	});

	it("never lets a stored name mention anyone, and mentions only a valid login", () =>
		fc.assert(
			fc.property(fc.oneof(genName.map((name) => ({ kind: "name", name })), genName.map((name) => ({ name })), genLoginish.map((login) => ({ kind: "github", login }))), (credit) => {
				const text = creditOf(withCredit(credit));
				const isLogin = "login" in credit;
				if (!isLogin) expect(text ?? "").not.toContain("@");
				if (text?.includes("@")) {
					expect(isLogin).toBe(true);
					expect(GITHUB_LOGIN.test((credit as { login: string }).login)).toBe(true);
					expect(text).toBe(`@${(credit as { login: string }).login}`);
				}
			}),
			runs(400),
		));

	it("strips the login's line when the report is deleted", () => {
		const body = "x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku · Reported by @octocat</sub>";
		expect(stripCredit(body, "@octocat")).toBe("x\n\n<sub>Filed from the app · report `fb_x…` · triage by Haiku</sub>");
	});
});

describe("filing a report with a GitHub credit", () => {
	beforeEach(async () => {
		await resetServer();
		await env.DB.batch([env.DB.prepare("DELETE FROM feedback_events"), env.DB.prepare("DELETE FROM feedback")]);
	});

	const KEY = newKey();
	const send = (r: unknown) =>
		callJson("/v1/feedback", { method: "POST", json: r, headers: { "x-kanso-feedback-key": KEY }, ip: "203.0.113.9", now: START });

	it("stores the tagged credit, and keeps @login for the deletion queue", async () => {
		const r = report({ credit: { kind: "github", login: "octocat" } });
		expect((await send(r)).status).toBe(201);
		const row = await env.DB.prepare("SELECT report, credit_name FROM feedback WHERE id = ?1").bind(r.id).first<{ report: string; credit_name: string }>();
		expect(JSON.parse(row!.report).credit).toEqual({ kind: "github", login: "octocat" });
		expect(row!.credit_name).toBe("@octocat");
	});

	it("stores an older app's name as the tagged one", async () => {
		const r = report({ credit: { name: "tendai" } });
		expect((await send(r)).status).toBe(201);
		const row = await env.DB.prepare("SELECT report, credit_name FROM feedback WHERE id = ?1").bind(r.id).first<{ report: string; credit_name: string }>();
		expect(JSON.parse(row!.report).credit).toEqual({ kind: "name", name: "tendai" });
		expect(row!.credit_name).toBe("tendai");
	});

	it("answers 400 at the credit's path and never echoes the value", async () => {
		const res = await send(report({ credit: { kind: "github", login: "bad--login" } }));
		expect(res.status).toBe(400);
		expect(res.body).toMatchObject({ error: "invalid", field: "credit.login" });
		expect(JSON.stringify(res.body)).not.toContain("bad--login");
		expect((await send(report({ credit: { kind: "name", name: "@octocat" } }))).body.field).toBe("credit.name");
	});
});

describe("GET /v1/feedback/users", () => {
	let net: ReturnType<typeof fakeNetwork>;
	const KEY = newKey();
	const PATH = "/v1/feedback/users";
	const avatar = (id: number) => `https://avatars.githubusercontent.com/u/${id}?v=4`;
	const user = (login: string, id = 1) => ({ login, avatar_url: avatar(id), id, type: "User", html_url: `https://github.com/${login}`, score: 1 });
	const search = async (q: string, init: { now?: number; ip?: string; key?: string } = {}) =>
		callJson(`${PATH}?q=${encodeURIComponent(q)}`, {
			headers: { "x-kanso-feedback-key": init.key ?? KEY },
			ip: init.ip ?? "203.0.113.9",
			now: init.now ?? START,
			env: await configured(),
		});
	const searches = () => net.sent.filter((s) => s.url.includes("/search/users"));

	beforeEach(async () => {
		await resetServer();
		await env.DB.exec("DELETE FROM limits");
		resetSearchCache();
		net = fakeNetwork();
	});
	// The fake network is restored by the next beforeEach replacing it; restore here so other files see the real one.
	const done = () => net.restore();

	it("returns only login and avatar_url, from the App's token", async () => {
		net.users = [user("octocat", 583231), user("octokit", 3430433)];
		const res = await search("octo");
		expect(res.status).toBe(200);
		expect(res.body).toEqual({
			users: [
				{ login: "octocat", avatar_url: avatar(583231) },
				{ login: "octokit", avatar_url: avatar(3430433) },
			],
		});
		const [sent] = searches();
		expect(sent.headers.get("authorization")).toBe("Bearer ghs_test");
		const q = new URL(sent.url);
		expect(q.searchParams.get("q")).toBe("octo in:login type:user");
		expect(q.searchParams.get("per_page")).toBe("8");
		done();
	});

	it("limits results to 8 and drops any with an off-pattern login or avatar", async () => {
		net.users = [
			...Array.from({ length: 12 }, (_, i) => user(`user${i}`, i)),
			{ login: "bad--login", avatar_url: avatar(1) },
			{ login: "evil", avatar_url: "https://evil.example/a.png" },
			{ login: "http-avatar", avatar_url: "http://avatars.githubusercontent.com/u/1" },
			{ login: 5, avatar_url: avatar(1) },
		];
		const { body } = await search("user");
		expect(body.users).toHaveLength(8);
		expect(body.users.map((u: { login: string }) => u.login)).toEqual(Array.from({ length: 8 }, (_, i) => `user${i}`));
		net.users = [{ login: "bad--login", avatar_url: avatar(1) }, { login: "evil", avatar_url: "https://evil.example/a.png" }];
		resetSearchCache();
		expect((await search("user")).body).toEqual({ users: [] });
		done();
	});

	it.each([["a"], ["a b"], ["octo in:login"], ["octo+org:x"], ["octo\nfoo"], ["a".repeat(40)], [""], ["名前"], ["a:b"]])("refuses %j with 400 q and never calls GitHub", async (q) => {
		const res = await search(q);
		expect(res.status).toBe(400);
		expect(res.body).toMatchObject({ error: "invalid", field: "q" });
		expect(searches()).toHaveLength(0);
		done();
	});

	it("takes a leading @ and any case, and caches an answer for a minute", async () => {
		net.users = [user("octocat")];
		expect((await search("@Octo")).status).toBe(200);
		expect((await search("octo", { now: START + 30_000 })).body.users).toHaveLength(1);
		expect(searches()).toHaveLength(1);
		await search("octo", { now: START + 60_001 });
		expect(searches()).toHaveLength(2);
		done();
	});

	it("needs the same credentials as the other feedback routes", async () => {
		const none = await callJson(`${PATH}?q=octo`, { ip: "203.0.113.9", now: START, env: await configured() });
		expect(none.status).toBe(401);
		const bad = await callJson(`${PATH}?q=octo`, { headers: { "x-kanso-feedback-key": "short" }, ip: "203.0.113.9", now: START, env: await configured() });
		expect(bad.status).toBe(401);
		expect(searches()).toHaveLength(0);
		done();
	});

	it("works for a signed-in device", async () => {
		const a = await signUp("alice");
		net.users = [user("octocat")];
		const res = await callJson(`${PATH}?q=octo`, {
			token: a.token,
			headers: { "x-kanso-device": a.device.id, "x-kanso-device-proof": a.device.proof },
			now: START,
			env: await configured(),
		});
		expect(res.status).toBe(200);
		done();
	});

	it("is not held to the 5-a-minute report binding: many searches pass until the search rule says 429", async () => {
		net.users = [user("octocat")];
		for (let i = 0; i < 60; i++) expect((await search("octo", { now: START + i })).status).toBe(200);
		const over = await search("octo", { now: START + 61 });
		expect(over.status).toBe(429);
		expect(over.body).toMatchObject({ error: "rate_limited", limit: "feedback-search" });
		// Another address has its own count, and the window passes.
		expect((await search("octo", { now: START + 62, ip: "198.51.100.4" })).status).toBe(200);
		expect((await search("octo", { now: START + 10 * 60_000 + 1 })).status).toBe(200);
		done();
	});

	it("answers 503 search_unavailable when GitHub fails, and does not cache the failure", async () => {
		net.failing.push("/search/users");
		const res = await search("octo");
		expect(res.status).toBe(503);
		expect(res.body).toMatchObject({ error: "search_unavailable" });
		net.failing.length = 0;
		net.users = [user("octocat")];
		expect((await search("octo", { now: START + 1 })).body.users).toHaveLength(1);
		done();
	});

	it("answers 503 search_unavailable when the GitHub App is not set up", async () => {
		const res = await callJson(`${PATH}?q=octo`, { headers: { "x-kanso-feedback-key": KEY }, ip: "203.0.113.9", now: START });
		expect(res.status).toBe(503);
		expect(res.body.error).toBe("search_unavailable");
		expect(searches()).toHaveLength(0);
		done();
	});

	it("refuses other methods", async () => {
		const res = await call(`${PATH}?q=octo`, { method: "POST", json: {}, headers: { "x-kanso-feedback-key": KEY }, now: START });
		expect(res.status).toBe(405);
		done();
	});

	it("cleans a query to the login characters or nothing (property)", () =>
		fc.assert(
			fc.property(fc.string({ maxLength: 50 }), (raw) => {
				const q = cleanQuery(raw);
				if (q !== null) {
					expect(q).toMatch(/^[A-Za-z0-9-]{2,39}$/);
					expect(raw.trim().replace(/^@/, "")).toBe(q);
				}
			}),
			runs(400),
		));
});

void REPO;
