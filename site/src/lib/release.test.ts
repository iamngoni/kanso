/**
 * Tests for GitHub release parsing and formatting: a realistic release
 * fixture, rejection of drafts, prereleases, foreign URLs and garbage,
 * build ordering, size/date formatting, and totality properties.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { type Release, buildsFor, formatDate, formatSize, parseRelease } from "./release";

const BASE = "https://github.com/iamngoni/kanso/releases/download/v0.5.2";
const asset = (name: string, size = 48_000_000, url = `${BASE}/${name}`) => ({ name, size, browser_download_url: url });

function fixture(overrides: Record<string, unknown> = {}) {
  return {
    tag_name: "v0.5.2",
    draft: false,
    prerelease: false,
    published_at: "2026-10-20T09:30:00Z",
    assets: [
      asset("Kanso-0.5.2-linux-arm64.tar.gz"),
      asset("Kanso-0.5.2-windows-x64.zip", 40_000_000),
      asset("Kanso-0.5.2-macos-arm64.dmg", 52_400_000),
      asset("Kanso-0.5.2-macos-arm64.app.tar.gz", 50_000_000),
      asset("Kanso-0.5.2-windows-x64-setup.exe", 41_000_000),
      asset("Kanso-0.5.2-linux-x64.tar.gz", 45_000_000),
      asset("SHA256SUMS", 400),
    ],
    ...overrides,
  };
}

function parsed(overrides: Record<string, unknown> = {}): Release {
  const release = parseRelease(fixture(overrides));
  if (release === null) throw new Error("fixture did not parse");
  return release;
}

describe("parseRelease", () => {
  it("parses a realistic release", () => {
    const r = parsed();
    expect(r.version).toBe("0.5.2");
    expect(r.publishedAt).toBe("2026-10-20T09:30:00Z");
    expect(r.pageUrl).toBe("https://github.com/iamngoni/kanso/releases/tag/v0.5.2");
    expect(r.checksumsUrl).toBe(`${BASE}/SHA256SUMS`);
    expect(r.builds).toHaveLength(5);
    expect(r.builds.map((b) => b.file)).not.toContain("Kanso-0.5.2-macos-arm64.app.tar.gz");
    expect(r.videoUrl).toBeNull();
  });

  it("finds the What's new video, only from this repository's release downloads", () => {
    const video = "Kanso-0.5.2-whats-new.webp";
    const withVideo = (url: string) => parsed({ assets: [...fixture().assets, asset(video, 400_000, url)] });
    expect(withVideo(`${BASE}/${video}`).videoUrl).toBe(`${BASE}/${video}`);
    expect(withVideo(`https://evil.example/${video}`).videoUrl).toBeNull();
    // Another release's video isn't this one's.
    expect(parsed({ assets: [...fixture().assets, asset("Kanso-0.5.1-whats-new.webp")] }).videoUrl).toBeNull();
  });

  it("orders builds as the download page lists them, whatever the asset order", () => {
    expect(parsed().builds.map((b) => b.kind)).toEqual(["macos-dmg", "windows-setup", "windows-zip", "linux-x64", "linux-arm64"]);
  });

  it("accepts a tag without the v prefix and prerelease-style versions", () => {
    const base = "https://github.com/iamngoni/kanso/releases/download/1.0.0-rc.1";
    const r = parseRelease(
      fixture({ tag_name: "1.0.0-rc.1", assets: [asset("Kanso-1.0.0-rc.1-macos-arm64.dmg", 10, `${base}/Kanso-1.0.0-rc.1-macos-arm64.dmg`)] }),
    );
    expect(r?.version).toBe("1.0.0-rc.1");
    expect(r?.checksumsUrl).toBeNull();
  });

  it("rejects drafts and prereleases", () => {
    expect(parseRelease(fixture({ draft: true }))).toBeNull();
    expect(parseRelease(fixture({ prerelease: true }))).toBeNull();
  });

  it("rejects bad tags and dates", () => {
    expect(parseRelease(fixture({ tag_name: 5 }))).toBeNull();
    expect(parseRelease(fixture({ tag_name: "latest" }))).toBeNull();
    expect(parseRelease(fixture({ published_at: "not a date" }))).toBeNull();
    expect(parseRelease(fixture({ published_at: null }))).toBeNull();
  });

  it("rejects non-objects and releases without usable assets", () => {
    for (const junk of [null, undefined, 3, "x", [], [fixture()], {}]) expect(parseRelease(junk)).toBeNull();
    expect(parseRelease(fixture({ assets: "nope" }))).toBeNull();
    expect(parseRelease(fixture({ assets: [] }))).toBeNull();
    expect(parseRelease(fixture({ assets: [null, 4, asset("README.md")] }))).toBeNull();
  });

  it("drops assets served from outside the repository's release downloads", () => {
    const name = "Kanso-0.5.2-macos-arm64.dmg";
    const bad = [
      `https://evil.example/iamngoni/kanso/releases/download/v0.5.2/${name}`,
      `https://github.com/other/kanso/releases/download/v0.5.2/${name}`,
      `https://github.com/iamngoni/kanso/releases/download/v0.5.1/${name}`,
      `http://github.com/iamngoni/kanso/releases/download/v0.5.2/${name}`,
      `${BASE}/${name}?x=1`,
      `${BASE}/${name}#frag`,
      `https://github.com.evil.example/iamngoni/kanso/releases/download/v0.5.2/${name}`,
      "https://github.com:bad/iamngoni/kanso/releases/download/v0.5.2/x",
      42,
    ];
    for (const url of bad) expect(parseRelease(fixture({ assets: [asset(name, 5, url as string)] }))).toBeNull();
  });

  it("drops builds with unusable sizes and ignores a foreign checksums URL", () => {
    const name = "Kanso-0.5.2-macos-arm64.dmg";
    for (const size of [0, -1, 1.5, "9", null, Number.MAX_VALUE]) {
      expect(parseRelease(fixture({ assets: [{ name, size, browser_download_url: `${BASE}/${name}` }] }))).toBeNull();
    }
    const r = parseRelease(fixture({ assets: [asset(name), asset("SHA256SUMS", 1, "https://evil.example/SHA256SUMS")] }));
    expect(r?.checksumsUrl).toBeNull();
  });

  it("never throws on arbitrary JSON", () => {
    fc.assert(
      fc.property(fc.anything(), (json) => {
        const r = parseRelease(json);
        expect(r === null || r.builds.length > 0).toBe(true);
      }),
    );
    fc.assert(
      fc.property(fc.jsonValue(), (json) => {
        expect(() => parseRelease(json)).not.toThrow();
      }),
    );
  });
});

describe("buildsFor", () => {
  it("returns a platform's builds, primary first", () => {
    const r = parsed();
    expect(buildsFor(r, "windows").map((b) => b.kind)).toEqual(["windows-setup", "windows-zip"]);
    expect(buildsFor(r, "linux").map((b) => b.kind)).toEqual(["linux-x64", "linux-arm64"]);
    expect(buildsFor(r, "macos")).toHaveLength(1);
  });

  it("is empty when the release lacks the platform", () => {
    const r = parseRelease(fixture({ assets: [asset("Kanso-0.5.2-macos-arm64.dmg")] }));
    expect(r && buildsFor(r, "linux")).toEqual([]);
  });
});

describe("formatSize", () => {
  it("uses decimal KB below a megabyte and MB above", () => {
    expect(formatSize(1)).toBe("1 KB");
    expect(formatSize(0)).toBe("1 KB");
    expect(formatSize(400_000)).toBe("400 KB");
    expect(formatSize(999_999)).toBe("1000 KB");
    expect(formatSize(1_000_000)).toBe("1 MB");
    expect(formatSize(48_400_000)).toBe("48 MB");
  });

  it("is never empty and never decreases as size grows", () => {
    const num = (s: string) => Number.parseInt(s, 10);
    const mb = (s: string) => (s.endsWith("MB") ? 1000 : 1) * num(s);
    fc.assert(
      fc.property(fc.nat(10 ** 10), fc.nat(10 ** 10), (a, b) => {
        const [lo, hi] = a <= b ? [a, b] : [b, a];
        expect(formatSize(lo)).toMatch(/^\d+ (KB|MB)$/);
        expect(mb(formatSize(lo))).toBeLessThanOrEqual(mb(formatSize(hi)));
      }),
    );
  });
});

describe("formatDate", () => {
  it("formats in UTC, British style", () => {
    expect(formatDate("2026-10-20T09:30:00Z")).toBe("20 October 2026");
    expect(formatDate("2026-12-31T23:59:59Z")).toBe("31 December 2026");
    expect(formatDate("2026-01-01T00:00:00+02:00")).toBe("31 December 2025");
  });
});
