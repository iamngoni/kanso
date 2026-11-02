// no-prop-test: framework glue
/**
 * The server function pages load their data with: the latest release and a
 * guess at the visitor's system from their request headers. It runs on the
 * Worker; the headers it reads are not stored or logged.
 */
import { createServerFn } from "@tanstack/react-start";
import { getRequestHeader } from "@tanstack/react-start/server";
import { type MobileOs, type Platform, detectMobileOs, detectPlatform } from "../lib/platform";
import type { Release } from "../lib/release";
import type { ReleaseNote } from "../lib/releases";
import { latestRelease, runtimeDeps } from "./release";
import { recentReleases } from "./releases";

export interface SiteData {
  readonly release: Release | null;
  readonly platform: Platform | null;
  readonly mobile: MobileOs | null;
}

export const getSiteData = createServerFn({ method: "GET" }).handler(
  async (): Promise<SiteData> => ({
    release: await latestRelease(runtimeDeps()),
    platform: detectPlatform(getRequestHeader("user-agent"), getRequestHeader("sec-ch-ua-platform")),
    mobile: detectMobileOs(getRequestHeader("user-agent"), getRequestHeader("sec-ch-ua-platform")),
  }),
);

/** The recent releases for the release notes page; null when GitHub can't be reached. */
export const getReleaseNotes = createServerFn({ method: "GET" }).handler(
  async (): Promise<ReleaseNote[] | null> => recentReleases(runtimeDeps()),
);
