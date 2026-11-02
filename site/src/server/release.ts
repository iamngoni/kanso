/**
 * Fetches Kanso's latest release from GitHub's public API for the pages that
 * offer downloads. Answers are kept in the edge cache for ten minutes (plus a thirty-day last good copy, served when GitHub fails) so
 * visitors don't spend the Worker's unauthenticated GitHub quota. Any failure
 * (network, timeout, rate limit, a release that doesn't parse) is null, and
 * the pages then link to the releases page on GitHub instead of a build.
 */
import { cachedGithubJson } from "./cachedGithub";
import { REPOSITORY, type Release, parseRelease } from "../lib/release";

export const LATEST_RELEASE_API = `https://api.github.com/repos/${REPOSITORY}/releases/latest`;
export { RELEASE_CACHE_SECONDS, RELEASE_TIMEOUT_MS } from "./cachedGithub";

/** What `latestRelease` needs from its runtime; tests pass fakes. */
export interface ReleaseDeps {
  readonly fetch: typeof fetch;
  /** Cloudflare's `caches.default`; absent outside Workers. */
  readonly cache?: Pick<Cache, "match" | "put"> | undefined;
  /** Logs one line per failure, without request details. */
  /** Optional GitHub token (the GITHUB_TOKEN secret) for a higher rate limit. */
  readonly token?: string | undefined;
  readonly log?: ((line: string) => void) | undefined;
}

export async function latestRelease(deps: ReleaseDeps): Promise<Release | null> {
  const json = await cachedGithubJson(deps, LATEST_RELEASE_API, "release", (j) => parseRelease(j) !== null);
  return json === null ? null : parseRelease(json);
}

/** The runtime's dependencies: global fetch, and the edge cache where there is one. */
export function runtimeDeps(): ReleaseDeps {
  const edge = (globalThis as { caches?: { default?: Cache } }).caches?.default;
  return { fetch: globalThis.fetch.bind(globalThis), cache: edge, token: process.env.GITHUB_TOKEN || undefined, log: (line) => console.warn(line) };
}
