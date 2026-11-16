/**
 * Turns GitHub's list of releases into the entries on the release notes page.
 * Like `parseRelease`, parsing happens once here: an entry's link is built
 * from its tag rather than taken from the JSON, and its body is cleaned of
 * links that aren't web links before anything renders it. Drafts,
 * prereleases and tags that aren't versions are left out.
 */
import { type Block, type Inline, parse } from "./markdown";
import { RELEASES_URL, whatsNewUrl } from "./release";

/** How many releases the page lists. */
export const RELEASE_NOTES_COUNT = 20;

export interface ReleaseNote {
  /** Without the leading v, e.g. "0.9.0". */
  readonly version: string;
  /** ISO 8601, UTC. */
  readonly publishedAt: string;
  readonly pageUrl: string;
  /** The release's Markdown, as written; empty when it has none. */
  readonly body: string;
  /** The release's What's new video, or null when it has none. */
  readonly videoUrl: string | null;
}

const VERSION = /^v?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$/;

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** The releases a page can show, newest first, at most `RELEASE_NOTES_COUNT`; [] when the JSON isn't a list. */
export function parseReleaseNotes(json: unknown): ReleaseNote[] {
  if (!Array.isArray(json)) return [];
  const notes: ReleaseNote[] = [];
  for (const item of json) {
    if (!isRecord(item) || item.draft === true || item.prerelease === true) continue;
    const tag = item.tag_name;
    const version = typeof tag === "string" ? VERSION.exec(tag)?.[1] : undefined;
    const published = item.published_at;
    if (version === undefined || typeof published !== "string" || Number.isNaN(Date.parse(published))) continue;
    notes.push({
      version,
      publishedAt: published,
      pageUrl: `${RELEASES_URL}/tag/${tag as string}`,
      body: typeof item.body === "string" ? item.body : "",
      videoUrl: whatsNewUrl(Array.isArray(item.assets) ? item.assets.filter(isRecord) : [], tag as string, version),
    });
  }
  return notes.sort((a, b) => Date.parse(b.publishedAt) - Date.parse(a.publishedAt)).slice(0, RELEASE_NOTES_COUNT);
}

/** The inlines with every link that isn't to http(s) replaced by its text, so nothing like `javascript:` is ever an href. */
export function safeInlines(inlines: readonly Inline[]): Inline[] {
  return inlines.flatMap((i): Inline[] => {
    switch (i.t) {
      case "link":
        return /^https?:\/\//i.test(i.href) ? [{ ...i, c: safeInlines(i.c) }] : safeInlines(i.c);
      case "strong":
      case "em":
      case "del":
        return [{ ...i, c: safeInlines(i.c) }];
      default:
        return [i];
    }
  });
}

/** A release body as blocks that are safe to render. */
export function bodyBlocks(body: string): Block[] {
  return parse(body).map((block): Block => {
    switch (block.t) {
      case "heading":
      case "paragraph":
      case "quote":
        return { ...block, c: safeInlines(block.c) };
      case "list":
        return { ...block, items: block.items.map((item) => ({ ...item, c: safeInlines(item.c) })) };
      case "table":
        return { ...block, head: block.head.map(safeInlines), rows: block.rows.map((row) => row.map(safeInlines)) };
      default:
        return block;
    }
  });
}
