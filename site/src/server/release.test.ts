/**
 * Tests for latestRelease: fake fetch and cache objects cover success and
 * caching, cache hits, non-OK, thrown and timed-out requests, invalid JSON,
 * unparsable releases and logging, plus runtimeDeps' wiring.
 */
import { afterEach, describe, expect, it, vi } from "vitest";
import { LATEST_RELEASE_API, RELEASE_CACHE_SECONDS, RELEASE_TIMEOUT_MS, type ReleaseDeps, latestRelease, runtimeDeps } from "./release";

const BASE = "https://github.com/iamngoni/kanso/releases/download/v0.5.2";
const json = {
  tag_name: "v0.5.2",
  published_at: "2026-10-20T09:30:00Z",
  assets: [{ name: "Kanso-0.5.2-macos-arm64.dmg", size: 52_000_000, browser_download_url: `${BASE}/Kanso-0.5.2-macos-arm64.dmg` }],
};

const ok = (body: unknown = json) => new Response(JSON.stringify(body), { status: 200 });

function setup(response: () => Promise<Response>, cached?: Response) {
  const fetch = vi.fn(response) as unknown as ReleaseDeps["fetch"] & ReturnType<typeof vi.fn>;
  const match = vi.fn(async () => cached);
  const put = vi.fn(async () => {});
  const log = vi.fn();
  return { fetch, match, put, log, deps: { fetch, cache: { match, put }, log } as ReleaseDeps };
}

afterEach(() => vi.unstubAllGlobals());

describe("latestRelease", () => {
  it("fetches, parses and caches a release", async () => {
    const t = setup(async () => ok());
    const release = await latestRelease(t.deps);
    expect(release?.version).toBe("0.5.2");
    expect(t.fetch).toHaveBeenCalledTimes(1);
    const [request, init] = t.fetch.mock.calls[0] as [Request, RequestInit];
    expect(request.url).toBe(LATEST_RELEASE_API);
    expect(request.headers.get("accept")).toBe("application/vnd.github+json");
    expect(init.signal).toBeInstanceOf(AbortSignal);
    expect(t.put).toHaveBeenCalledTimes(2);
    const stored = (t.put.mock.calls[0] as unknown as [Request, Response])[1];
    expect(stored.headers.get("cache-control")).toBe(`public, max-age=${RELEASE_CACHE_SECONDS}`);
    expect(await stored.json()).toEqual(json);
    expect(t.log).not.toHaveBeenCalled();
  });

  it("serves a cache hit without fetching", async () => {
    const t = setup(async () => ok(), ok());
    expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
    expect(t.fetch).not.toHaveBeenCalled();
    expect(t.put).not.toHaveBeenCalled();
  });

  it("works without a cache or a logger", async () => {
    expect((await latestRelease({ fetch: (async () => ok()) as unknown as typeof fetch }))?.version).toBe("0.5.2");
    expect(await latestRelease({ fetch: (async () => new Response("", { status: 500 })) as unknown as typeof fetch })).toBeNull();
  });

  it("returns null and logs the status for a non-OK response", async () => {
    const t = setup(async () => new Response("rate limited", { status: 403 }));
    expect(await latestRelease(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("release: github 403");
    expect(t.put).not.toHaveBeenCalled();
  });

  it("returns null when fetch throws or times out", async () => {
    const boom = setup(async () => {
      throw new TypeError("network down");
    });
    expect(await latestRelease(boom.deps)).toBeNull();
    expect(boom.log).toHaveBeenCalledWith("release: TypeError");

    const timeout = setup(async () => {
      throw Object.assign(new Error("timed out"), { name: "TimeoutError" });
    });
    expect(await latestRelease(timeout.deps)).toBeNull();
    expect(timeout.log).toHaveBeenCalledWith("release: TimeoutError");

    const odd = setup(() => Promise.reject("string"));
    expect(await latestRelease(odd.deps)).toBeNull();
    expect(odd.log).toHaveBeenCalledWith("release: error");
    expect(RELEASE_TIMEOUT_MS).toBeGreaterThan(0);
  });

  it("returns null for invalid JSON, without caching", async () => {
    const t = setup(async () => new Response("<html>", { status: 200 }));
    expect(await latestRelease(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("release: SyntaxError");
    expect(t.put).not.toHaveBeenCalled();
  });

  it("returns null for JSON that is not an offerable release", async () => {
    const t = setup(async () => ok({ ...json, draft: true }));
    expect(await latestRelease(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("release: unparsable");
    expect(t.put).not.toHaveBeenCalled();
  });

  it("returns null when the cache itself fails", async () => {
    const t = setup(async () => ok());
    t.match.mockRejectedValueOnce(new Error("cache down"));
    expect(await latestRelease(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("release: Error");
  });
});

describe("runtimeDeps", () => {
  it("uses global fetch, no cache outside Workers, and logs with console.warn", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const deps = runtimeDeps();
    expect(deps.cache).toBeUndefined();
    deps.log?.("hello");
    expect(warn).toHaveBeenCalledWith("hello");
    warn.mockRestore();
  });

  it("picks up the edge cache when the runtime has one", () => {
    const edge = { match: vi.fn(), put: vi.fn() };
    vi.stubGlobal("caches", { default: edge });
    expect(runtimeDeps().cache).toBe(edge);
  });
});
