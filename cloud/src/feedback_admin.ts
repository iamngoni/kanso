// The owner's feedback admin routes (docs/FEEDBACK.md › Admin commands): read a raw report,
// its triage and its screenshot. The same `FEEDBACK_ADMIN_TOKEN` as `/run` guards every route;
// with the secret unset none of them exists (404), so a server without it has no admin surface.
import type { Env } from "./env";
import { screenshotKey } from "./feedback";
import { parseCredit } from "./feedback_credit";
import { runTriage } from "./feedback_triage";

const encoder = new TextEncoder();
const FEEDBACK_ID = /^fb_[0-9a-z]{26}$/;

async function digest(text: string): Promise<Uint8Array> {
	return new Uint8Array(await crypto.subtle.digest("SHA-256", encoder.encode(text)));
}

/** Whether `header` is `Bearer <token>`, compared in constant time. */
export async function authorized(header: string | null, token: string): Promise<boolean> {
	if (header === null || !header.startsWith("Bearer ")) return false;
	const given = await digest(header.slice("Bearer ".length));
	const expected = await digest(token);
	let difference = 0;
	for (let index = 0; index < expected.length; index++) difference |= expected[index] ^ given[index];
	return difference === 0;
}

const json = (status: number, body: unknown): Response =>
	new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

const fail = (status: number, error: string, message: string): Response => json(status, { error, message });

/** 404 without the secret, 401 without the token; otherwise null, so the handler proceeds. */
async function guard(request: Request, env: Env): Promise<Response | null> {
	const token = env.FEEDBACK_ADMIN_TOKEN;
	if (!token || token.length < 32) return fail(404, "not_found", "Not found.");
	if (!(await authorized(request.headers.get("authorization"), token))) return fail(401, "unauthorized", "Send the admin token.");
	return null;
}

const parsed = (text: unknown): unknown => {
	if (typeof text !== "string") return null;
	try {
		return JSON.parse(text);
	} catch {
		return null;
	}
};

/** POST /v1/admin/feedback/run: 200 with the run's report (counts and each failure's code). */
export async function run(request: Request, env: Env, now: number): Promise<Response> {
	const refused = await guard(request, env);
	if (refused) return refused;
	return json(200, await runTriage(env, now));
}

/** GET /v1/admin/feedback?limit=N: newest reports (1–100, default 20), newest first. */
export async function list(request: Request, env: Env): Promise<Response> {
	const refused = await guard(request, env);
	if (refused) return refused;
	const raw = new URL(request.url).searchParams.get("limit") ?? "20";
	if (!/^[0-9]{1,3}$/.test(raw) || Number(raw) < 1 || Number(raw) > 100) return fail(400, "invalid_limit", "limit must be 1 to 100.");
	const rows = await env.DB.prepare(
		`SELECT id, created_at, kind, status, issue, has_screenshot, screenshot_state,
		        substr(json_extract(report, '$.text'), 1, 60) AS text
		 FROM feedback ORDER BY created_at DESC LIMIT ?1`,
	)
		.bind(Number(raw))
		.all<Record<string, unknown>>();
	return json(200, {
		reports: rows.results.map((r) => ({
			id: String(r.id),
			created_at: Number(r.created_at),
			kind: String(r.kind),
			status: String(r.status),
			issue: r.issue === null ? null : Number(r.issue),
			has_screenshot: r.has_screenshot === 1,
			screenshot_state: String(r.screenshot_state),
			text: String(r.text ?? ""),
		})),
	});
}

/** The tagged credit of a stored report (`{kind, ...}`), or null; reads rows stored before the tagged form. */
const creditOfReport = (report: unknown): unknown => {
	const raw = typeof report === "object" && report !== null ? (report as { credit?: unknown }).credit : null;
	const credit = parseCredit(raw ?? null);
	return credit.ok ? credit.value : null;
};

/** GET /v1/admin/feedback/<id>: the validated raw report, triage and status chain. */
export function show(id: string): (request: Request, env: Env, now: number) => Promise<Response> {
	return async (request, env) => {
		const refused = await guard(request, env);
		if (refused) return refused;
		if (!FEEDBACK_ID.test(id)) return fail(400, "invalid_id", "Not a feedback ID.");
		const row = await env.DB.prepare(
			`SELECT id, created_at, kind, status, issue, duplicate_of, pr, release, credit_name,
			        screenshot_state, triage_attempts, purged_at, report, triage
			 FROM feedback WHERE id = ?1`,
		)
			.bind(id)
			.first<Record<string, unknown>>();
		if (!row) return fail(404, "not_found", "Not found.");
		const events = await env.DB.prepare("SELECT at, status, detail FROM feedback_events WHERE feedback_id = ?1 ORDER BY at, status")
			.bind(id)
			.all<Record<string, unknown>>();
		const report = parsed(row.report);
		return json(200, {
			id: row.id,
			created_at: Number(row.created_at),
			kind: String(row.kind),
			status: String(row.status),
			issue: row.issue === null ? null : Number(row.issue),
			duplicate_of: row.duplicate_of === null ? null : Number(row.duplicate_of),
			pr: row.pr === null ? null : Number(row.pr),
			release: row.release ?? null,
			credit_name: row.credit_name ?? null,
			credit: creditOfReport(report),
			screenshot_state: String(row.screenshot_state),
			triage_attempts: Number(row.triage_attempts),
			purged_at: row.purged_at === null ? null : Number(row.purged_at),
			report,
			triage: parsed(row.triage),
			history: events.results.map((e) => ({ at: Number(e.at), status: String(e.status), detail: e.detail ?? null })),
		});
	};
}

/** GET /v1/admin/feedback/<id>/screenshot: the stored PNG or JPEG, never a public URL. */
export function screenshot(id: string): (request: Request, env: Env, now: number) => Promise<Response> {
	return async (request, env) => {
		const refused = await guard(request, env);
		if (refused) return refused;
		if (!FEEDBACK_ID.test(id)) return fail(400, "invalid_id", "Not a feedback ID.");
		const row = await env.DB.prepare("SELECT screenshot_state FROM feedback WHERE id = ?1").bind(id).first<{ screenshot_state: string }>();
		if (!row || row.screenshot_state !== "stored") return fail(404, "not_found", "Not found.");
		const object = await env.FILES.get(screenshotKey(id));
		if (!object) return fail(404, "not_found", "Not found.");
		const jpeg = object.customMetadata?.type === "image/jpeg";
		return new Response(object.body, {
			headers: {
				"content-type": jpeg ? "image/jpeg" : "image/png",
				"content-length": String(object.size),
				"content-disposition": `attachment; filename="${id}.${jpeg ? "jpg" : "png"}"`,
			},
		});
	};
}
