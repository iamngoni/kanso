/**
 * Shared GitHub fetch for release.ts and releases.ts. Fresh answers live in
 * the edge cache for ten minutes; every good answer is also stored under a
 * second "last good" key for thirty days, and served when the fresh fetch
 * fails (rate limit, timeout, 5xx, bad JSON), so a GitHub hiccup doesn't
 * blank the download page. Returns the raw JSON, or null when neither exists.
 */
import type { ReleaseDeps } from "./release";

export const RELEASE_CACHE_SECONDS = 600;
export const RELEASE_STALE_SECONDS = 30 * 24 * 60 * 60;
export const RELEASE_TIMEOUT_MS = 4_000;

const staleKey = (url: string) => new Request(`${url}${url.includes("?") ? "&" : "?"}kanso-last-good=1`);

function githubRequest(url: string, token: string | undefined): Request {
  const headers: Record<string, string> = {
    accept: "application/vnd.github+json",
    "user-agent": "kansoapp.app",
    "x-github-api-version": "2022-11-28",
  };
  if (token) headers.authorization = `Bearer ${token}`;
  return new Request(url, { headers });
}

const stored = (json: unknown, seconds: number) =>
  new Response(JSON.stringify(json), {
    headers: { "content-type": "application/json", "cache-control": `public, max-age=${seconds}` },
  });

/** `accept` says whether the JSON is usable; anything else counts as a failed fetch. */
export async function cachedGithubJson(
  deps: ReleaseDeps,
  url: string,
  label: string,
  accept: (json: unknown) => boolean,
): Promise<unknown> {
  const log = deps.log ?? (() => {});
  const lastGood = staleKey(url);
  const fallback = async (): Promise<unknown> => {
    try {
      const old = await deps.cache?.match(lastGood);
      if (!old) return null;
      const json: unknown = await old.json();
      if (!accept(json)) return null;
      log(`${label}: served last good copy`);
      return json;
    } catch {
      return null;
    }
  };
  try {
    const cached = await deps.cache?.match(new Request(url));
    if (cached) return await cached.json();
    const response = await deps.fetch(githubRequest(url, deps.token), { signal: AbortSignal.timeout(RELEASE_TIMEOUT_MS) });
    if (!response.ok) {
      log(`${label}: github ${response.status}`);
      return await fallback();
    }
    const json: unknown = await response.json();
    if (!accept(json)) {
      log(`${label}: unparsable`);
      return await fallback();
    }
    await deps.cache?.put(new Request(url), stored(json, RELEASE_CACHE_SECONDS));
    await deps.cache?.put(lastGood, stored(json, RELEASE_STALE_SECONDS));
    return json;
  } catch (err) {
    log(`${label}: ${err instanceof Error ? err.name : "error"}`);
    return await fallback();
  }
}
