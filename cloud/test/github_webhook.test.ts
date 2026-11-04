// POST /v1/github/webhook (docs/FEEDBACK.md › The webhook): the HMAC check, the order of the other
// checks, redelivery as a no-op, and every status transition, including the release compare.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { sign, signatureValid } from "../src/github";
import { START, call, resetServer, signUp } from "./helpers";
import { REPO, WEBHOOK_SECRET, configured, fakeNetwork, newId } from "./feedback-helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const PATH = "/v1/github/webhook";
const SHA = "a".repeat(40);

let net: ReturnType<typeof fakeNetwork>;
let delivery = 0;
beforeEach(async () => {
	await resetServer();
	await env.DB.batch([env.DB.prepare("DELETE FROM feedback_events"), env.DB.prepare("DELETE FROM feedback"), env.DB.prepare("DELETE FROM feedback_deliveries")]);
	net = fakeNetwork();
});
afterEach(() => net.restore());

interface Hook {
	event?: string;
	payload?: Record<string, unknown>;
	secret?: string;
	signature?: string | null;
	type?: string;
	id?: string | null;
	now?: number;
	raw?: string;
}

async function hook(h: Hook = {}) {
	const body = new TextEncoder().encode(h.raw ?? JSON.stringify({ repository: { full_name: REPO }, ...h.payload }));
	const headers: Record<string, string> = { "content-type": h.type ?? "application/json", "x-github-event": h.event ?? "issues" };
	const signature = h.signature === undefined ? `sha256=${await sign(h.secret ?? WEBHOOK_SECRET, body)}` : h.signature;
	if (signature !== null) headers["x-hub-signature-256"] = signature;
	if (h.id !== null) headers["x-github-delivery"] = h.id ?? `d-${++delivery}`;
	const r = await call(PATH, { method: "POST", body, headers, env: await configured(), now: h.now ?? START });
	const text = await r.text();
	return { status: r.status, body: text ? JSON.parse(text) : null };
}

async function seed(status: string, over: Record<string, unknown> = {}, account: string | null = null): Promise<string> {
	const id = newId();
	const columns = { issue: null, pr: null, merge_sha: null, release: null, ...over };
	await env.DB.batch([
		env.DB.prepare(
			`INSERT INTO feedback (id, account_id, reporter, created_at, kind, report, status, issue, pr, merge_sha, release)
  VALUES (?1, ?2, ?3, ?4, 'bug', '{}', ?5, ?6, ?7, ?8, ?9)`,
		).bind(id, account, account ? null : "r_" + "2".repeat(32), START, status, columns.issue, columns.pr, columns.merge_sha, columns.release),
		env.DB.prepare("INSERT INTO feedback_events (feedback_id, at, status) VALUES (?1, ?2, 'received')").bind(id, START),
	]);
	return id;
}
const row = async (id: string) => (await env.DB.prepare("SELECT * FROM feedback WHERE id = ?1").bind(id).first<any>())!;
const statuses = async (id: string) => (await env.DB.prepare("SELECT status FROM feedback_events WHERE feedback_id = ?1 ORDER BY at, status").bind(id).all<any>()).results.map((e) => e.status);

describe("checks", () => {
	it("415 for another content type, before anything else", async () => {
		expect((await hook({ type: "application/x-www-form-urlencoded" })).status).toBe(415);
	});

	it("401 for a missing, malformed or wrong signature, and for a body signed with another secret", async () => {
		expect((await hook({ signature: null })).status).toBe(401);
		expect((await hook({ signature: "sha256=zz" })).status).toBe(401);
		expect((await hook({ signature: `sha1=${"0".repeat(40)}` })).status).toBe(401);
		expect((await hook({ signature: `sha256=${"0".repeat(64)}` })).status).toBe(401);
		expect((await hook({ secret: "another-secret" })).status).toBe(401);
		expect((await hook({})).status).toBe(200);
	});

	it("401 when one byte of the body changed after signing", async () => {
		const body = JSON.stringify({ repository: { full_name: REPO }, action: "labeled" });
		const signature = `sha256=${await sign(WEBHOOK_SECRET, new TextEncoder().encode(body))}`;
		expect((await hook({ raw: body.replace("labeled", "Labeled"), signature })).status).toBe(401);
	});

	it("503 when the secret or the repository isn't configured", async () => {
		const body = new TextEncoder().encode("{}");
		const r = await call(PATH, { method: "POST", body, headers: { "content-type": "application/json", "x-hub-signature-256": `sha256=${await sign("x", body)}`, "x-github-event": "issues" } });
		expect(r.status).toBe(503);
	});

	it("400 for an event outside the allow-list, and for a body that isn't JSON; ping answers", async () => {
		expect((await hook({ event: "push" })).status).toBe(400);
		expect((await hook({ event: "workflow_run" })).status).toBe(400);
		expect((await hook({ raw: "not json" })).status).toBe(400);
		expect((await hook({ event: "ping", payload: {} })).status).toBe(200);
	});

	it("403 for another repository, or none", async () => {
		expect((await hook({ raw: JSON.stringify({ repository: { full_name: "someone/else" } }) })).status).toBe(403);
		expect((await hook({ raw: JSON.stringify({}) })).status).toBe(403);
	});

	it("400 for a missing delivery ID, and a redelivery is a no-op", async () => {
		expect((await hook({ id: null })).status).toBe(400);
		const id = await seed("filed", { issue: 5 });
		const labeled = { action: "labeled", label: { name: "build" }, issue: { number: 5 }, sender: { login: "iamngoni" } };
		expect((await hook({ payload: labeled, id: "same" })).body).toMatchObject({ moved: 1 });
		await env.DB.prepare("UPDATE feedback SET status = 'filed' WHERE id = ?1").bind(id).run();
		expect((await hook({ payload: labeled, id: "same" })).body).toEqual({ ok: true, duplicate: true });
		expect((await row(id)).status).toBe("filed");
	});

	it("413 for a body over 2 MiB", async () => {
		expect((await hook({ raw: JSON.stringify({ repository: { full_name: REPO }, pad: "x".repeat(2 * 1024 * 1024) }) })).status).toBe(413);
	});

	it("refuses browsers (an Origin header)", async () => {
		const body = new TextEncoder().encode("{}");
		const r = await call(PATH, { method: "POST", body, headers: { origin: "https://github.com", "content-type": "application/json", "x-hub-signature-256": `sha256=${await sign(WEBHOOK_SECRET, body)}`, "x-github-event": "ping" } });
		expect(r.status).toBe(403);
	});

	it("only POST is allowed", async () => {
		expect((await call(PATH, { method: "GET" })).status).toBe(405);
	});
});

describe("issues", () => {
	const labeled = (over: Record<string, unknown> = {}) => ({ action: "labeled", label: { name: "build" }, issue: { number: 7 }, sender: { login: "iamngoni" }, ...over });

	it("the build label added by the owner moves a filed report to building", async () => {
		const id = await seed("filed", { issue: 7 });
		const other = await seed("filed", { issue: 8 });
		expect((await hook({ payload: labeled() })).body).toEqual({ ok: true, moved: 1 });
		expect((await row(id)).status).toBe("building");
		expect((await row(other)).status).toBe("filed");
		expect((await statuses(id)).sort()).toEqual(["building", "received"]);
	});

	it("moves every report filed under that issue", async () => {
		const a = await seed("filed", { issue: 7 });
		const b = await seed("filed", { issue: 7 });
		expect((await hook({ payload: labeled() })).body).toMatchObject({ moved: 2 });
		expect([(await row(a)).status, (await row(b)).status]).toEqual(["building", "building"]);
	});

	it("ignores the build label from anyone but the owner, and any other label", async () => {
		const id = await seed("filed", { issue: 7 });
		await hook({ payload: labeled({ sender: { login: "stranger" } }) });
		await hook({ payload: labeled({ label: { name: "bug" } }) });
		await hook({ payload: labeled({ sender: { login: "iamngoni2" } }) });
		await hook({ payload: labeled({ sender: {} }) });
		expect((await row(id)).status).toBe("filed");
	});

	it("does not move a report that is already further along", async () => {
		const id = await seed("pr_open", { issue: 7, pr: 9 });
		await hook({ payload: labeled() });
		expect((await row(id)).status).toBe("pr_open");
	});

	it("closing as not planned closes the report; closing as completed does not", async () => {
		const id = await seed("filed", { issue: 7 });
		await hook({ payload: { action: "closed", issue: { number: 7, state_reason: "completed" } } });
		expect((await row(id)).status).toBe("filed");
		await hook({ payload: { action: "closed", issue: { number: 7, state_reason: "not_planned" } } });
		expect((await row(id)).status).toBe("closed");
	});

	it("a merged or released report stays when its issue is closed as not planned", async () => {
		const merged = await seed("merged", { issue: 7, pr: 9 });
		const released = await seed("released", { issue: 7, pr: 9 });
		await hook({ payload: { action: "closed", issue: { number: 7, state_reason: "not_planned" } } });
		expect((await row(merged)).status).toBe("merged");
		expect((await row(released)).status).toBe("released");
	});
});

describe("pull requests", () => {
	const opened = (over: Record<string, any> = {}) => ({
		action: "opened",
		pull_request: { number: 52, body: "Closes #7\n\nDone.", head: { ref: "feedback/issue-7", repo: { full_name: REPO } }, ...over },
	});

	it("a PR from feedback/issue-N that closes #N moves the report to pr_open and records the PR", async () => {
		const id = await seed("building", { issue: 7 });
		expect((await hook({ event: "pull_request", payload: opened() })).body).toMatchObject({ moved: 1 });
		expect(await row(id)).toMatchObject({ status: "pr_open", pr: 52 });
		expect((await env.DB.prepare("SELECT detail FROM feedback_events WHERE feedback_id = ?1 AND status = 'pr_open'").bind(id).first<any>()).detail).toBe("PR #52");
	});

	it("takes 'Fixes #N' and a filed report that was never labelled", async () => {
		const id = await seed("filed", { issue: 7 });
		await hook({ event: "pull_request", payload: opened({ body: "fixes #7" }) });
		expect((await row(id)).status).toBe("pr_open");
	});

	for (const [name, over] of [
		["the wrong branch", { head: { ref: "feature/x", repo: { full_name: REPO } } }],
		["another issue's branch", { head: { ref: "feedback/issue-8", repo: { full_name: REPO } } }],
		["a fork", { head: { ref: "feedback/issue-7", repo: { full_name: "someone/kanso" } } }],
		["no Closes line", { body: "Mentions #7 only" }],
		["a Closes line for another issue", { body: "Closes #70" }],
	] as [string, Record<string, unknown>][]) {
		it(`ignores a PR with ${name}`, async () => {
			const id = await seed("building", { issue: 7 });
			await hook({ event: "pull_request", payload: opened(over) });
			expect((await row(id)).status).toBe("building");
		});
	}

	it("a merged PR moves the report to merged with the merge commit", async () => {
		const id = await seed("pr_open", { issue: 7, pr: 52 });
		await hook({ event: "pull_request", payload: { action: "closed", pull_request: { number: 52, merged: true, merge_commit_sha: SHA } } });
		expect(await row(id)).toMatchObject({ status: "merged", merge_sha: SHA });
	});

	it("a PR closed without a merge sends the report back to filed", async () => {
		const id = await seed("pr_open", { issue: 7, pr: 52 });
		await hook({ event: "pull_request", payload: { action: "closed", pull_request: { number: 52, merged: false } } });
		expect(await row(id)).toMatchObject({ status: "filed", pr: null });
		expect(await statuses(id)).toEqual(["filed", "received"].sort());
	});

	it("a PR that isn't the report's changes nothing", async () => {
		const id = await seed("pr_open", { issue: 7, pr: 52 });
		await hook({ event: "pull_request", payload: { action: "closed", pull_request: { number: 53, merged: true, merge_commit_sha: SHA } } });
		expect((await row(id)).status).toBe("pr_open");
	});
});

describe("releases", () => {
	const published = (tag: string, over: Record<string, unknown> = {}) => ({ action: "published", release: { tag_name: tag, draft: false, prerelease: false, ...over } });
	const A = "1".repeat(40);
	const B = "2".repeat(40);
	const C = "3".repeat(40);

	it("marks the merged reports whose commit is between the previous tag and this one as released", async () => {
		net.releases = [
			{ tag_name: "v0.9.2", draft: false, prerelease: false },
			{ tag_name: "v0.9.1", draft: false, prerelease: false },
		];
		net.commits["v0.9.2"] = [A, B];
		const a = await seed("merged", { issue: 1, pr: 11, merge_sha: A });
		const b = await seed("merged", { issue: 2, pr: 12, merge_sha: B });
		const later = await seed("merged", { issue: 3, pr: 13, merge_sha: C });
		const open = await seed("pr_open", { issue: 4, pr: 14 });
		expect((await hook({ event: "release", payload: published("v0.9.2") })).body).toMatchObject({ moved: 2 });
		expect(await row(a)).toMatchObject({ status: "released", release: "v0.9.2" });
		expect((await row(b)).status).toBe("released");
		expect((await row(later)).status).toBe("merged");
		expect((await row(open)).status).toBe("pr_open");
		const compare = net.sent.filter((s) => s.url.includes("/compare/"));
		expect(compare).toHaveLength(1);
		expect(new URL(compare[0].url).pathname).toBe(`/repos/${REPO}/compare/v0.9.1...v0.9.2`);
		expect((await env.DB.prepare("SELECT detail FROM feedback_events WHERE feedback_id = ?1 AND status = 'released'").bind(a).first<any>()).detail).toBe("v0.9.2");
	});

	it("with no earlier release, everything merged so far is in this one", async () => {
		net.releases = [{ tag_name: "v0.1.0", draft: false, prerelease: false }];
		const a = await seed("merged", { issue: 1, pr: 11, merge_sha: A });
		await hook({ event: "release", payload: published("v0.1.0") });
		expect((await row(a)).status).toBe("released");
		expect(net.sent.some((s) => s.url.includes("/compare/"))).toBe(false);
	});

	it("skips drafts, pre-releases and tags that aren't vX.Y.Z, without calling GitHub", async () => {
		const a = await seed("merged", { issue: 1, pr: 11, merge_sha: A });
		await hook({ event: "release", payload: published("v0.9.2", { draft: true }) });
		await hook({ event: "release", payload: published("v0.9.2", { prerelease: true }) });
		await hook({ event: "release", payload: published("nightly") });
		await hook({ event: "release", payload: { action: "created", release: { tag_name: "v0.9.2" } } });
		expect((await row(a)).status).toBe("merged");
		expect(net.sent).toEqual([]);
	});

	it("wakes each signed-in reporter once", async () => {
		net.releases = [{ tag_name: "v0.9.2", draft: false, prerelease: false }, { tag_name: "v0.9.1", draft: false, prerelease: false }];
		net.commits["v0.9.2"] = [A, B];
		const alice = await signUp("alice");
		await env.DB.prepare("UPDATE sessions SET push_platform = 'apns', push_token = ?2, push_updated_at = 1, push_environment = 'production' WHERE account_id = ?1")
			.bind(alice.creds.accountId, "ab".repeat(32))
			.run();
		await seed("merged", { issue: 1, pr: 11, merge_sha: A }, alice.creds.accountId);
		await seed("merged", { issue: 2, pr: 12, merge_sha: B }, alice.creds.accountId);
		await hook({ event: "release", payload: published("v0.9.2") });
		const due = await env.DB.prepare("SELECT push_due_at FROM sessions WHERE account_id = ?1").bind(alice.creds.accountId).first<any>();
		expect(due.push_due_at).not.toBeNull();
	});

	it("a failing compare is a 500 and the delivery is forgotten, so GitHub's retry is handled", async () => {
		net.releases = [{ tag_name: "v0.9.2", draft: false, prerelease: false }, { tag_name: "v0.9.1", draft: false, prerelease: false }];
		net.failing.push("/compare/");
		const a = await seed("merged", { issue: 1, pr: 11, merge_sha: A });
		expect((await hook({ event: "release", payload: published("v0.9.2"), id: "retry-me" })).status).toBe(500);
		expect((await row(a)).status).toBe("merged");
		net.failing.length = 0;
		net.commits["v0.9.2"] = [A];
		expect((await hook({ event: "release", payload: published("v0.9.2"), id: "retry-me" })).body).toMatchObject({ moved: 1 });
		expect((await row(a)).status).toBe("released");
	});
});

describe("properties", () => {
	it("a body signed with the secret verifies; any other secret or any changed byte does not", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 1, maxLength: 200 }), fc.string({ minLength: 1, maxLength: 30 }), fc.nat(), async (body, other, at) => {
				const signature = `sha256=${await sign(WEBHOOK_SECRET, body)}`;
				expect(await signatureValid(WEBHOOK_SECRET, body, signature)).toBe(true);
				if (other !== WEBHOOK_SECRET) expect(await signatureValid(other, body, signature)).toBe(false);
				const changed = Uint8Array.from(body);
				changed[at % changed.length] ^= 1;
				expect(await signatureValid(WEBHOOK_SECRET, changed, signature)).toBe(false);
			}),
			runs(60),
		);
	});

	it("whatever the payload, an unsigned or misigned request changes no report", async () => {
		const id = await seed("filed", { issue: 7 });
		await fc.assert(
			fc.asyncProperty(fc.jsonValue(), fc.constantFrom(null, "sha256=" + "0".repeat(64), "nope"), async (payload, signature) => {
				const r = await hook({ raw: JSON.stringify(payload), signature });
				expect(r.status).toBe(401);
			}),
			runs(40),
		);
		expect((await row(id)).status).toBe("filed");
	});
});
