/**
 * Fetches Kanso's recent releases from GitHub's public API for the release
 * notes page. Answered like `latestRelease`: kept in the edge cache for ten
 * minutes, and any failure (network, timeout, rate limit, JSON that isn't a
 * list) is null, and the page then links to the releases page on GitHub.
 */
import { REPOSITORY } from "../lib/release";
import { RELEASE_NOTES_COUNT, type ReleaseNote, parseReleaseNotes } from "../lib/releases";
import { cachedGithubJson } from "./cachedGithub";
import type { ReleaseDeps } from "./release";

// Drafts and prereleases come back in the list and are dropped by the parser, so ask for a few extra.
export const RELEASE_LIST_API = `https://api.github.com/repos/${REPOSITORY}/releases?per_page=${RELEASE_NOTES_COUNT + 10}`;

export async function recentReleases(deps: ReleaseDeps): Promise<ReleaseNote[] | null> {
  const json = await cachedGithubJson(deps, RELEASE_LIST_API, "releases", Array.isArray);
  return json === null ? null : parseReleaseNotes(json);
}
