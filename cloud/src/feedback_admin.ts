// The owner's "run feedback triage now" (docs/FEEDBACK.md › Running triage now): the same
// `runTriage` the cron runs, on demand, so a fix to triage or filing is checked in seconds rather
// than at the next quarter hour. Bearer FEEDBACK_ADMIN_TOKEN only; with the secret unset the route
// does not exist (404), so a server without it has no admin surface at all.
import type { Env } from "./env";
import { runTriage } from "./feedback_triage";

const encoder = new TextEncoder();

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

/** POST /v1/admin/feedback/run: 200 with the run's report (counts and each failure's code). */
export async function run(request: Request, env: Env, now: number): Promise<Response> {
	const token = env.FEEDBACK_ADMIN_TOKEN;
	if (!token || token.length < 32) return json(404, { error: "not_found", message: "Not found." });
	if (!(await authorized(request.headers.get("authorization"), token))) {
		return json(401, { error: "unauthorized", message: "Send the admin token." });
	}
	return json(200, await runTriage(env, now));
}
