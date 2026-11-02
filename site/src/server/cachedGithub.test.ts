/**
 * Tests for the last-good fallback shared by latestRelease and recentReleases:
 * a keyed in-memory cache and fake fetch cover fresh, cached, stale-on-failure,
 * nothing-to-serve and the optional token.
 */
import { describe, expect, it, vi } from "vitest";
import { RELEASE_STALE_SECONDS } from "./cachedGithub";
import { LATEST_RELEASE_API, type ReleaseDeps, latestRelease } from "./release";
import { RELEASE_LIST_API, recentReleases } from "./releases";

const BASE = "https://github.com/iamngoni/kanso/releases/download/v0.5.2";
const latestJson = {
  tag_name: "v0.5.2",
  published_at: "2026-10-20T09:30:00Z",
  assets: [{ name: "Kanso-0.5.2-macos-arm64.dmg", size: 52_000_000, browser_download_url: `${BASE}/Kanso-0.5.2-macos-arm64.dmg` }],
};
const listJson = [{ tag_name: "v0.9.0", published_at: "2026-10-20T09:30:00Z", body: "- Mobile apps" }];

/** A cache keyed by URL that keeps each entry's max-age, with fresh entries removable to mimic expiry. */
function fakeCache() {
  const store = new Map<string, { body: string; maxAge: string | null }>();
  const cache = {
    match: async (req: Request) => {
      const hit = store.get(req.url);
      return hit ? new Response(hit.body) : undefined;
    },
    put: async (req: Request, res: Response) => {
      store.set(req.url, { body: await res.text(), maxAge: res.headers.get("cache-control") });
    },
  };
  return { store, cache };
}

const ok = (body: unknown) => new Response(JSON.stringify(body), { status: 200 });

function setup(responses: Array<() => Promise<Response>>) {
  const queue = [...responses];
  const fetch = vi.fn(async () => (queue.shift() ?? (() => Promise.reject(new Error("unexpected"))))()) as unknown as ReleaseDeps["fetch"] &
    ReturnType<typeof vi.fn>;
  const { store, cache } = fakeCache();
  const log = vi.fn();
  return { fetch, store, log, deps: { fetch, cache, log } as ReleaseDeps };
}

const expire = (store: Map<string, unknown>, url: string) => store.delete(url);

describe("last good fallback", () => {
  it("fresh ok: fetches and stores a fresh and a long-lived copy", async () => {
    const t = setup([async () => ok(latestJson)]);
    expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
    expect(t.store.get(LATEST_RELEASE_API)?.maxAge).toBe("public, max-age=600");
    const keys = [...t.store.keys()].filter((k) => k !== LATEST_RELEASE_API);
    expect(keys).toHaveLength(1);
    expect(t.store.get(keys[0] as string)?.maxAge).toBe(`public, max-age=${RELEASE_STALE_SECONDS}`);
  });

  it("cached fresh: serves without fetching", async () => {
    const t = setup([async () => ok(latestJson)]);
    await latestRelease(t.deps);
    expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
    expect(t.fetch).toHaveBeenCalledTimes(1);
  });

  it("fetch fails: serves the stale copy once the fresh one has expired", async () => {
    for (const failure of [
      async () => new Response("rate limited", { status: 403 }),
      async () => new Response("bad gateway", { status: 502 }),
      async () => {
        throw Object.assign(new Error("timed out"), { name: "TimeoutError" });
      },
      async () => new Response("<html>", { status: 200 }),
    ]) {
      const t = setup([async () => ok(latestJson), failure]);
      await latestRelease(t.deps);
      expire(t.store, LATEST_RELEASE_API);
      expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
      expect(t.log).toHaveBeenCalledWith("release: served last good copy");
    }
  });

  it("fetch fails and no stale copy: null", async () => {
    const t = setup([async () => new Response("rate limited", { status: 403 })]);
    expect(await latestRelease(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("release: github 403");
  });

  it("applies to the release list too", async () => {
    const t = setup([async () => ok(listJson), async () => new Response("", { status: 403 })]);
    expect((await recentReleases(t.deps))?.map((n) => n.version)).toEqual(["0.9.0"]);
    expire(t.store, RELEASE_LIST_API);
    expect((await recentReleases(t.deps))?.map((n) => n.version)).toEqual(["0.9.0"]);
    const empty = setup([async () => new Response("", { status: 500 })]);
    expect(await recentReleases(empty.deps)).toBeNull();
  });

  it("does not replace the stale copy with an unusable answer", async () => {
    const t = setup([async () => ok(latestJson), async () => ok({ ...latestJson, draft: true }), async () => new Response("", { status: 500 })]);
    await latestRelease(t.deps);
    expire(t.store, LATEST_RELEASE_API);
    expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
    expect((await latestRelease(t.deps))?.version).toBe("0.5.2");
  });

  it("sends an Authorization header only when a token is set", async () => {
    const withToken = setup([async () => ok(latestJson)]);
    await latestRelease({ ...withToken.deps, token: "ghp_x" });
    expect(((withToken.fetch.mock.calls[0] as unknown as [Request])[0]).headers.get("authorization")).toBe("Bearer ghp_x");
    const without = setup([async () => ok(latestJson)]);
    await latestRelease(without.deps);
    expect(((without.fetch.mock.calls[0] as unknown as [Request])[0]).headers.get("authorization")).toBeNull();
  });
});
