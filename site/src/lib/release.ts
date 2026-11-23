/**
 * Turns GitHub's "latest release" JSON into the downloads the site offers.
 * Parsing happens once here: a `Release` holds only builds whose files are
 * served from Kanso's own repository, so the pages render links without
 * checking anything again. The macOS update archive is left out on purpose;
 * installed copies fetch it themselves.
 */
import type { Platform } from "./platform";

export const REPOSITORY = "iamngoni/kanso";
export const REPOSITORY_URL = `https://github.com/${REPOSITORY}`;
export const RELEASES_URL = `${REPOSITORY_URL}/releases`;
const DOWNLOAD_PREFIX = `${REPOSITORY_URL}/releases/download/`;

/** The installable builds a release carries. */
export type BuildKind = "macos-dmg" | "windows-setup" | "windows-zip" | "linux-x64" | "linux-arm64";

export interface Build {
  readonly kind: BuildKind;
  readonly platform: Platform;
  /** What the button says, e.g. "Installer (.exe)". */
  readonly label: string;
  /** Who it is for, e.g. "Apple silicon". */
  readonly detail: string;
  readonly file: string;
  readonly url: string;
  /** Bytes. */
  readonly size: number;
}

export interface Release {
  /** Without the leading v, e.g. "0.5.2". */
  readonly version: string;
  /** ISO 8601, UTC. */
  readonly publishedAt: string;
  readonly pageUrl: string;
  /** Never empty: a release with no recognisable build doesn't parse. */
  readonly builds: readonly [Build, ...Build[]];
  readonly checksumsUrl: string | null;
  /** The release's What's new video (an animated WebP), or null when it has none. */
  readonly videoUrl: string | null;
}

interface KindRule {
  readonly kind: BuildKind;
  readonly platform: Platform;
  readonly suffix: string;
  readonly label: string;
  readonly detail: string;
}

/** In the order the download page lists them; the first per platform is that platform's primary build. */
const RULES: readonly KindRule[] = [
  { kind: "macos-dmg", platform: "macos", suffix: "-macos-arm64.dmg", label: "Disk image (.dmg)", detail: "Apple silicon · macOS 13 or later" },
  { kind: "windows-setup", platform: "windows", suffix: "-windows-x64-setup.exe", label: "Installer (.exe)", detail: "64-bit · installs for you only" },
  { kind: "windows-zip", platform: "windows", suffix: "-windows-x64.zip", label: "Portable (.zip)", detail: "64-bit · no installer" },
  { kind: "linux-x64", platform: "linux", suffix: "-linux-x64.tar.gz", label: "x86-64 (.tar.gz)", detail: "glibc 2.35+ · Wayland or X11" },
  { kind: "linux-arm64", platform: "linux", suffix: "-linux-arm64.tar.gz", label: "ARM64 (.tar.gz)", detail: "glibc 2.35+ · Wayland or X11" },
];

const VERSION = /^v?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$/;

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** A download URL of this repository's release `tag`, or null. */
function trustedUrl(value: unknown, tag: string): string | null {
  if (typeof value !== "string" || !value.startsWith(`${DOWNLOAD_PREFIX}${tag}/`)) return null;
  try {
    const url = new URL(value);
    return url.protocol === "https:" && url.hostname === "github.com" && url.search === "" && url.hash === "" ? value : null;
  } catch {
    return null;
  }
}

/**
 * The URL of the release's What's new video, `Kanso-<version>-whats-new.webp`
 * (docs/RELEASING.md › What's new video), or null. Held to the same rule as a
 * build: only a file in this repository's own release downloads.
 */
export function whatsNewUrl(assets: readonly Record<string, unknown>[], tag: string, version: string): string | null {
  const asset = assets.find((a) => a.name === `Kanso-${version}-whats-new.webp`);
  return asset ? trustedUrl(asset.browser_download_url, tag) : null;
}

/** The release, or null when the JSON isn't one this site can offer (a draft, a prerelease, no builds). */
export function parseRelease(json: unknown): Release | null {
  if (!isRecord(json) || json.draft === true || json.prerelease === true) return null;
  const tag = json.tag_name;
  if (typeof tag !== "string") return null;
  const version = VERSION.exec(tag)?.[1];
  if (version === undefined) return null;
  return parseAssets(json, tag, version);
}

export const NIGHTLY_TAG = "nightly";
export const NIGHTLY_URL = `${RELEASES_URL}/tag/${NIGHTLY_TAG}`;
const NIGHTLY_VERSION = /^Kanso-(\d+\.\d+\.\d+-nightly\.\d+)-/;

/**
 * The rolling `nightly` prerelease. Its tag is not a version, so the version
 * comes from the asset names, which carry it
 * (`Kanso-0.10.11-nightly.202611230841-macos-arm64.dmg`). Only builds of that
 * one version count; feed files and the update archive are never builds.
 */
export function parseNightly(json: unknown): Release | null {
  if (!isRecord(json) || json.draft === true || json.tag_name !== NIGHTLY_TAG) return null;
  const assets = Array.isArray(json.assets) ? json.assets.filter(isRecord) : [];
  const version = assets.map((a) => (typeof a.name === "string" ? NIGHTLY_VERSION.exec(a.name)?.[1] : undefined)).find((v) => v !== undefined);
  if (version === undefined) return null;
  return parseAssets(json, NIGHTLY_TAG, version);
}

function parseAssets(json: Record<string, unknown>, tag: string, version: string): Release | null {
  const publishedAt = typeof json.published_at === "string" && !Number.isNaN(Date.parse(json.published_at)) ? json.published_at : null;
  if (publishedAt === null) return null;
  const assets = Array.isArray(json.assets) ? json.assets.filter(isRecord) : [];

  const builds: Build[] = [];
  let checksumsUrl: string | null = null;
  for (const rule of RULES) {
    const file = `Kanso-${version}${rule.suffix}`;
    const asset = assets.find((a) => a.name === file);
    if (!asset) continue;
    const url = trustedUrl(asset.browser_download_url, tag);
    const size = asset.size;
    if (url === null || typeof size !== "number" || !Number.isSafeInteger(size) || size <= 0) continue;
    builds.push({ kind: rule.kind, platform: rule.platform, label: rule.label, detail: rule.detail, file, url, size });
  }
  const sums = assets.find((a) => a.name === "SHA256SUMS");
  if (sums) checksumsUrl = trustedUrl(sums.browser_download_url, tag);

  const [first, ...rest] = builds;
  if (first === undefined) return null;
  return { version, publishedAt, pageUrl: `${RELEASES_URL}/tag/${tag}`, builds: [first, ...rest], checksumsUrl, videoUrl: whatsNewUrl(assets, tag, version) };
}

/** The builds for `platform`, primary first. */
export function buildsFor(release: Release, platform: Platform): Build[] {
  return release.builds.filter((b) => b.platform === platform);
}

/** "48 MB": decimal megabytes, as file managers show them. */
export function formatSize(bytes: number): string {
  if (bytes < 1_000_000) return `${Math.max(1, Math.round(bytes / 1_000))} KB`;
  return `${Math.round(bytes / 1_000_000)} MB`;
}

/** "20 October 2026", in UTC so the server and the browser agree. */
export function formatDate(iso: string): string {
  return new Intl.DateTimeFormat("en-GB", { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" }).format(new Date(iso));
}
