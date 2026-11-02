/**
 * Guesses which desktop system a visitor is on, from the User-Agent and the
 * `Sec-CH-UA-Platform` client hint, to offer the right download first.
 * A guess only: phones, tablets and anything unrecognised get no guess, and
 * the download page always lists every build.
 */

/** The systems Kanso ships builds for. */
export type Platform = "macos" | "windows" | "linux";

export const PLATFORMS: readonly Platform[] = ["macos", "windows", "linux"];

export const PLATFORM_NAMES: Record<Platform, string> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux",
};

const MOBILE = /iphone|ipad|ipod|android|mobile|cros|tizen|kaios|playstation|xbox|nintendo/i;

/**
 * The visitor's desktop system, or null for phones, tablets and unknowns.
 * The client hint wins when present: browsers freeze parts of the
 * User-Agent, but the hint names the platform plainly (quoted).
 */
export function detectPlatform(userAgent: string | null | undefined, hint?: string | null): Platform | null {
  const fromHint = hint ? fromHintValue(hint) : undefined;
  if (fromHint !== undefined) return fromHint;
  const ua = userAgent ?? "";
  if (ua === "" || MOBILE.test(ua)) return null;
  if (/windows nt|win64|win32/i.test(ua)) return "windows";
  if (/macintosh|mac os x/i.test(ua)) return "macos";
  if (/linux|x11|freebsd|openbsd/i.test(ua)) return "linux";
  return null;
}

/** `"macOS"` and friends; undefined when the hint says nothing useful, so the User-Agent decides. */
function fromHintValue(hint: string): Platform | null | undefined {
  const value = hint.trim().replace(/^"|"$/g, "").toLowerCase();
  switch (value) {
    case "macos":
      return "macos";
    case "windows":
      return "windows";
    case "linux":
      return "linux";
    case "android":
    case "ios":
    case "chrome os":
    case "chromeos":
      return null;
    default:
      return undefined;
  }
}

/** The phone systems Kanso Pocket is offered on. */
export type MobileOs = "ios" | "android";

/**
 * Whether the visitor is on an iPhone/iPad or an Android device, or null for
 * everything else (desktops included). iPadOS Safari reports itself as a Mac,
 * so those iPads can't be told apart here and see the desktop order.
 */
export function detectMobileOs(userAgent: string | null | undefined, hint?: string | null): MobileOs | null {
  const value = hint?.trim().replace(/^"|"$/g, "").toLowerCase();
  if (value === "ios") return "ios";
  if (value === "android") return "android";
  if (value) return null;
  const ua = userAgent ?? "";
  if (/iphone|ipad|ipod/i.test(ua)) return "ios";
  if (/android/i.test(ua)) return "android";
  return null;
}
