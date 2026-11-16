/**
 * Tests for the release notes parser: which releases are listed and in what
 * order, the link built from the tag, and the clean-up that keeps any
 * non-web link out of a release body.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { RELEASE_NOTES_COUNT, bodyBlocks, parseReleaseNotes, safeInlines } from "./releases";
import { inline } from "./markdown";

const item = (tag: string, published = "2026-10-20T09:30:00Z", extra: Record<string, unknown> = {}) => ({
  tag_name: tag,
  published_at: published,
  body: `Notes for ${tag}`,
  ...extra,
});

describe("parseReleaseNotes", () => {
  it("lists releases newest first, with a link built from the tag", () => {
    const notes = parseReleaseNotes([item("v0.8.0", "2026-09-01T00:00:00Z"), item("v0.9.0")]);
    expect(notes.map((n) => n.version)).toEqual(["0.9.0", "0.8.0"]);
    expect(notes[0]).toEqual({
      version: "0.9.0",
      publishedAt: "2026-10-20T09:30:00Z",
      pageUrl: "https://github.com/iamngoni/kanso/releases/tag/v0.9.0",
      body: "Notes for v0.9.0",
      videoUrl: null,
    });
  });

  it("carries a release's What's new video when it has one", () => {
    const url = "https://github.com/iamngoni/kanso/releases/download/v0.10.9/Kanso-0.10.9-whats-new.webp";
    const [note] = parseReleaseNotes([item("v0.10.9", undefined, { assets: [{ name: "Kanso-0.10.9-whats-new.webp", browser_download_url: url }] })]);
    expect(note?.videoUrl).toBe(url);
  });

  it("leaves out drafts, prereleases, odd tags and bad dates", () => {
    const notes = parseReleaseNotes([
      item("v0.9.0", undefined, { draft: true }),
      item("v0.9.1", undefined, { prerelease: true }),
      item("nightly"),
      item("v0.9.2", "yesterday"),
      { tag_name: 5, published_at: "2026-10-20T09:30:00Z" },
      "string",
      null,
      item("v0.9.3"),
    ]);
    expect(notes.map((n) => n.version)).toEqual(["0.9.3"]);
  });

  it("uses an empty body when there is none", () => {
    expect(parseReleaseNotes([{ ...item("v1.0.0"), body: null }])[0]?.body).toBe("");
  });

  it("is empty for JSON that isn't a list", () => {
    expect(parseReleaseNotes({})).toEqual([]);
    expect(parseReleaseNotes(null)).toEqual([]);
  });

  it("never lists more than the limit, whatever it is given", () => {
    fc.assert(
      fc.property(fc.integer({ min: 0, max: 60 }), (n) => {
        const json = Array.from({ length: n }, (_, i) => item(`v1.0.${i}`, new Date(Date.UTC(2026, 0, 1 + i)).toISOString()));
        const notes = parseReleaseNotes(json);
        expect(notes.length).toBe(Math.min(n, RELEASE_NOTES_COUNT));
        const times = notes.map((x) => Date.parse(x.publishedAt));
        expect([...times].sort((a, b) => b - a)).toEqual(times);
      }),
    );
  });
});

describe("safeInlines", () => {
  it("keeps web links, also inside emphasis", () => {
    const out = safeInlines(inline("**[a](https://x.test/a)** and [b](http://x.test/b)"));
    expect(JSON.stringify(out)).toContain("https://x.test/a");
    expect(JSON.stringify(out)).toContain("http://x.test/b");
  });

  it("turns other links into their text", () => {
    const out = safeInlines(inline("[click](javascript:alert(1)) [mail](mailto:a@b.c) ~~[x](data:text/html,hi)~~"));
    expect(JSON.stringify(out)).not.toMatch(/javascript|mailto|data:/);
    expect(JSON.stringify(out)).toContain("click");
  });

  it("never leaves a link that is not http(s), for any text", () => {
    fc.assert(
      fc.property(fc.string({ maxLength: 40 }), fc.string({ maxLength: 20 }), (label, href) => {
        const walk = (nodes: ReturnType<typeof inline>): boolean =>
          nodes.every((n) => (n.t === "link" ? /^https?:\/\//i.test(n.href) && walk([...n.c]) : "c" in n ? walk([...n.c]) : true));
        expect(walk(safeInlines(inline(`[${label.replace(/[\]\n]/g, "")}](${href.replace(/[)\s]/g, "")})`)))).toBe(true);
      }),
    );
  });
});

describe("bodyBlocks", () => {
  it("cleans links in headings, paragraphs, quotes, lists and tables", () => {
    const body = [
      "## [h](javascript:1)",
      "[p](javascript:1)",
      "> [q](javascript:1)",
      "- [l](javascript:1)",
      "",
      "| [t](javascript:1) |",
      "|---|",
      "| [c](javascript:1) |",
      "",
      "---",
      "```",
      "[code](javascript:1)",
      "```",
    ].join("\n");
    const blocks = bodyBlocks(body);
    expect(blocks.map((b) => b.t)).toEqual(expect.arrayContaining(["heading", "paragraph", "quote", "list", "table", "hr", "code"]));
    const text = JSON.stringify(blocks.filter((b) => b.t !== "code"));
    expect(text).not.toContain("javascript");
  });
});
