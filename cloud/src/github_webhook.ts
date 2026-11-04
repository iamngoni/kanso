// `POST /v1/github/webhook` (docs/FEEDBACK.md › The webhook): GitHub's events move a report along its
// status chain. Checks, in order: Content-Type, the HMAC-SHA256 of the raw body (constant time), the
// event name, the repository, and the delivery ID (a redelivery is a no-op for 24 hours). Status
// only moves from the allow-listed events below, and never backwards past a merge.

import type { Env } from "./env";
import type { Status } from "./feedback";
import { commitsBetween, previousTag, signatureValid } from "./github";
import { error, isContentType, readBody } from "./http";
import { wakeAccount } from "./push";

export const MAX_WEBHOOK_BYTES = 2 * 1024 * 1024;
export const EVENTS = ["issues", "pull_request", "release", "ping"] as const;
const DEFAULT_OWNER = "iamngoni";

type Payload = Record<string, any>;

const ok = (body: Record<string, unknown> = {}): Response => new Response(JSON.stringify({ ok: true, ...body }), { status: 200, headers: { "content-type": "application/json; charset=utf-8" } });

/** Moves rows to `status` where `where` holds (the SQL after `WHERE`, `?1` the key), recording one event each. */
async function move(
	env: Env,
	now: number,
	status: Status,
	where: string,
	key: string | number,
	set = "",
	extra: (string | number | null)[] = [],
	detail: string | null = null,
): Promise<number> {
	// ?1 key, ?2 status, ?3.. extra (the parameters of `set`)
	const { results } = await env.DB.prepare(`UPDATE feedback SET status = ?2${set} WHERE ${where} RETURNING id`)
		.bind(key, status, ...extra)
		.all<{ id: string }>();
	if (results.length > 0) {
		await env.DB.batch(
			results.map((r) =>
				env.DB.prepare("INSERT OR IGNORE INTO feedback_events (feedback_id, at, status, detail) VALUES (?1, ?2, ?3, ?4)").bind(r.id, now, status, detail),
			),
		);
	}
	return results.length;
}

const closes = (body: string | null | undefined, n: number): boolean => new RegExp(`\\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\\s+#${n}\\b`, "i").test(body ?? "");

/** The status transitions of one event; the number of rows moved. */
export async function apply(env: Env, event: string, p: Payload, now: number): Promise<number> {
	const repo = env.FEEDBACK_REPO;
	const owner = env.FEEDBACK_OWNER || DEFAULT_OWNER;
	if (event === "issues") {
		const n = p.issue?.number;
		if (!Number.isInteger(n)) return 0;
		if (p.action === "labeled" && p.label?.name === "build" && p.sender?.login === owner) {
			return move(env, now, "building", "issue = ?1 AND status = 'filed'", n);
		}
		if (p.action === "closed" && p.issue?.state_reason === "not_planned") {
			return move(env, now, "closed", "issue = ?1 AND status IN ('received', 'filed', 'building', 'pr_open')", n);
		}
		// Closed for any other reason (completed, duplicate, none): finished, so retention can run. A
		// report with an open PR is left to the PR's own events, which say whether it merged.
		if (p.action === "closed") {
			return move(env, now, "closed", "issue = ?1 AND status IN ('received', 'filed', 'building')", n);
		}
		return 0;
	}
	if (event === "pull_request") {
		const pr = p.pull_request;
		const m = pr?.number;
		if (!Number.isInteger(m)) return 0;
		if (p.action === "opened") {
			const branch = /^feedback\/issue-([0-9]{1,9})$/.exec(String(pr.head?.ref ?? ""));
			if (!branch || pr.head?.repo?.full_name !== repo) return 0;
			const n = Number(branch[1]);
			if (!closes(pr.body, n)) return 0;
			return move(env, now, "pr_open", "issue = ?1 AND status IN ('filed', 'building')", n, ", pr = ?3", [m], `PR #${m}`);
		}
		if (p.action === "closed" && pr.merged === true) {
			const sha = typeof pr.merge_commit_sha === "string" && /^[0-9a-f]{40}$/.test(pr.merge_commit_sha) ? pr.merge_commit_sha : null;
			return move(env, now, "merged", "pr = ?1 AND status IN ('filed', 'building', 'pr_open')", m, ", merge_sha = ?3", [sha], `PR #${m}`);
		}
		if (p.action === "closed") {
			return move(env, now, "filed", "pr = ?1 AND status = 'pr_open'", m, ", pr = NULL");
		}
		return 0;
	}
	if (event === "release" && p.action === "published") return released(env, p, now);
	return 0;
}

/** `release.published`: every `merged` row whose commit is between the previous tag and this one is `released`. */
async function released(env: Env, p: Payload, now: number): Promise<number> {
	const tag = p.release?.tag_name;
	if (typeof tag !== "string" || !/^v[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}$/.test(tag) || p.release?.draft || p.release?.prerelease) return 0;
	const { results } = await env.DB.prepare("SELECT id, merge_sha, account_id FROM feedback WHERE status = 'merged'").all<{
		id: string;
		merge_sha: string | null;
		account_id: string | null;
	}>();
	if (results.length === 0) return 0;
	const base = await previousTag(env, tag, now);
	// With no earlier release, everything merged so far is in this one.
	const included = base === null ? null : await commitsBetween(env, base, tag, now);
	const mine = results.filter((r) => included === null || (r.merge_sha !== null && included.has(r.merge_sha)));
	if (mine.length === 0) return 0;
	const moved = new Set<string>();
	for (const r of mine) {
		moved.add(r.id);
		await move(env, now, "released", "id = ?1 AND status = 'merged'", r.id, ", release = ?3", [tag], tag);
	}
	// One silent wake-up per signed-in reporter: the app polls My feedback when it opens.
	for (const account of new Set(mine.map((r) => r.account_id).filter((a): a is string => a !== null))) await wakeAccount(env, account, "", now);
	return moved.size;
}

/** POST /v1/github/webhook: 200 handled or ignored, 400 unsupported event, 401 bad signature, 403 wrong repository, 413, 415, 503 not set up. */
export async function receive(request: Request, env: Env, now: number): Promise<Response> {
	if (!isContentType(request.headers.get("content-type"), "application/json", true)) {
		return error(415, "unsupported_media_type", "Send application/json.");
	}
	const secret = env.GITHUB_WEBHOOK_SECRET;
	if (!secret || !env.FEEDBACK_REPO) return error(503, "misconfigured", "The webhook is not set up.");
	const body = await readBody(request, MAX_WEBHOOK_BYTES);
	if (!body) return error(413, "too_large", "The body is too large.");
	if (!(await signatureValid(secret, body, request.headers.get("x-hub-signature-256")))) return error(401, "invalid_signature", "Wrong signature.");
	const event = request.headers.get("x-github-event") ?? "";
	if (!(EVENTS as readonly string[]).includes(event)) return error(400, "unsupported_event", "Unsupported event.");
	let payload: Payload;
	try {
		payload = JSON.parse(new TextDecoder().decode(body));
	} catch {
		return error(400, "bad_request", "The body is not JSON.");
	}
	if (event === "ping") return ok({ pong: true });
	if (payload?.repository?.full_name !== env.FEEDBACK_REPO) return error(403, "wrong_repository", "Wrong repository.");
	const delivery = request.headers.get("x-github-delivery") ?? "";
	if (!/^[0-9A-Za-z-]{1,64}$/.test(delivery)) return error(400, "bad_request", "Bad delivery ID.");
	const fresh = await env.DB.prepare("INSERT OR IGNORE INTO feedback_deliveries (id, at) VALUES (?1, ?2)").bind(delivery, now).run();
	if (fresh.meta.changes === 0) return ok({ duplicate: true });
	try {
		return ok({ moved: await apply(env, event, payload, now) });
	} catch (err) {
		// Forget the delivery so GitHub's retry is handled; it is logged by code only.
		await env.DB.prepare("DELETE FROM feedback_deliveries WHERE id = ?1").bind(delivery).run();
		console.error(JSON.stringify({ route: "github_webhook", error: "internal", kind: (err as Error)?.name ?? typeof err }));
		return error(500, "internal", "Internal error.");
	}
}
