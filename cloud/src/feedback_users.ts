// `GET /v1/feedback/users?q=<text>`: the GitHub user search behind the feedback sheet's "credit me by
// GitHub username" picker (docs/FEEDBACK.md › Credit). The apps ask the Worker rather than GitHub so the
// search runs on the App's installation token (the authenticated search rate, not an anonymous one shared
// by every phone behind a carrier) and so GitHub sees the Worker, not the person.
//
// Same callers as the other feedback routes (a signed-in device, or an anonymous key), but counted per
// client in D1 (`feedback-search`) rather than by the 5-a-minute binding, because the picker searches as
// the person types. Answers are cached in the isolate for a minute: the same prefix typed by many people,
// or retyped, costs GitHub one call. Only `login` and `avatar_url` leave; the query is limited to the
// characters of a login, so it can't carry a GitHub search qualifier.

import type { PepperKeys } from "./crypto";
import type { Env } from "./env";
import { whoIs } from "./feedback";
import * as github from "./github";
import { error, json } from "./http";
import { accountKey, clientIp, count, ipGroup, ipKey } from "./limits";

/** A search text: the characters of a login, 2–39 of them (a hyphen anywhere is fine for a prefix). */
export const QUERY = /^[A-Za-z0-9-]{2,39}$/;
/** How long an answer is reused. */
export const CACHE_MS = 60_000;
/** The most distinct queries held; the oldest goes first. */
export const CACHE_ENTRIES = 200;

const cache = new Map<string, { at: number; users: github.GithubUser[] }>();

/** Forgets every cached answer (tests). */
export function resetSearchCache(): void {
	cache.clear();
}

/** The search text from `q` with one leading `@` dropped, or null when it is not 2–39 login characters. */
export const cleanQuery = (raw: string | null): string | null => {
	const q = (raw ?? "").trim().replace(/^@/, "");
	return QUERY.test(q) ? q : null;
};

/** 200 `{users: [{login, avatar_url}]}`; 400 `q`; 401/403; 429 `feedback-search`; 503 `search_unavailable`. */
export async function search(request: Request, env: Env, now: number, keys: PepperKeys): Promise<Response> {
	const who = await whoIs(request, env, now, keys, { binding: false });
	if (who instanceof Response) return who;
	const q = cleanQuery(new URL(request.url).searchParams.get("q"));
	if (q === null) return error(400, "invalid", "Invalid q.", { field: "q" });
	const limited = await count(
		env.DB,
		"feedback-search",
		who.accountId !== null ? accountKey("feedback-search", who.accountId) : await ipKey(keys, "feedback-search", ipGroup(clientIp(request))),
		now,
	);
	if (limited) return limited;

	const key = q.toLowerCase();
	const hit = cache.get(key);
	if (hit && now - hit.at < CACHE_MS) return answer(hit.users);
	if (!github.configOf(env)) return unavailable();
	let users: github.GithubUser[];
	try {
		users = await github.searchUsers(env, q, now);
	} catch (err) {
		// GitHub's own refusal (rate limit, outage) is the person's "search unavailable": they can type a name instead.
		console.error(JSON.stringify({ route: "feedback_users", error: "search_failed", kind: (err as Error)?.name ?? typeof err }));
		return unavailable();
	}
	cache.delete(key);
	cache.set(key, { at: now, users });
	for (const oldest of cache.keys()) {
		if (cache.size <= CACHE_ENTRIES) break;
		cache.delete(oldest);
	}
	return answer(users);
}

const answer = (users: github.GithubUser[]): Response => json({ users }, 200, { "cache-control": "private, max-age=30" });

const unavailable = (): Response => error(503, "search_unavailable", "User search is unavailable.");
