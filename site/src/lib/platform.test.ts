/**
 * Tests for platform detection: example UAs for each desktop system, the
 * client hint taking precedence, mobile and unknown agents giving no guess,
 * and a property that the result is always null or a known platform.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { PLATFORMS, detectMobileOs, detectPlatform } from "./platform";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Safari/605.1.15";
const WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36";
const LINUX = "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0";

describe("detectPlatform", () => {
  it("recognises desktop user agents", () => {
    expect(detectPlatform(MAC)).toBe("macos");
    expect(detectPlatform(WIN)).toBe("windows");
    expect(detectPlatform(LINUX)).toBe("linux");
    expect(detectPlatform("Mozilla/5.0 (X11; FreeBSD amd64)")).toBe("linux");
  });

  it("gives no guess for mobile, consoles and ChromeOS", () => {
    expect(detectPlatform("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)")).toBeNull();
    expect(detectPlatform("Mozilla/5.0 (Linux; Android 14; Pixel 8) Mobile Safari")).toBeNull();
    expect(detectPlatform("Mozilla/5.0 (X11; CrOS x86_64 14541.0.0)")).toBeNull();
  });

  it("gives no guess for empty, missing or unknown agents", () => {
    expect(detectPlatform("")).toBeNull();
    expect(detectPlatform(null)).toBeNull();
    expect(detectPlatform(undefined)).toBeNull();
    expect(detectPlatform("curl/8.0")).toBeNull();
  });

  it("lets the client hint win over the user agent", () => {
    expect(detectPlatform(MAC, '"Windows"')).toBe("windows");
    expect(detectPlatform(WIN, '"macOS"')).toBe("macos");
    expect(detectPlatform(WIN, ' "Linux" ')).toBe("linux");
    expect(detectPlatform(WIN, '"Android"')).toBeNull();
    expect(detectPlatform(WIN, '"iOS"')).toBeNull();
    expect(detectPlatform(WIN, '"Chrome OS"')).toBeNull();
    expect(detectPlatform(WIN, '"ChromeOS"')).toBeNull();
  });

  it("falls back to the user agent when the hint is empty or unrecognised", () => {
    expect(detectPlatform(MAC, "")).toBe("macos");
    expect(detectPlatform(MAC, null)).toBe("macos");
    expect(detectPlatform(MAC, '"Haiku"')).toBe("macos");
  });

  it("always returns null or a known platform", () => {
    fc.assert(
      fc.property(fc.string(), fc.option(fc.string(), { nil: undefined }), (ua, hint) => {
        const result = detectPlatform(ua, hint);
        expect(result === null || PLATFORMS.includes(result)).toBe(true);
      }),
    );
  });
});

describe("detectMobileOs", () => {
  const IPHONE = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148";
  const IPAD = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148";
  const ANDROID = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/120.0 Mobile Safari/537.36";

  it("recognises iPhone, iPad and Android", () => {
    expect(detectMobileOs(IPHONE)).toBe("ios");
    expect(detectMobileOs(IPAD)).toBe("ios");
    expect(detectMobileOs(ANDROID)).toBe("android");
  });

  it("gives null for desktops, unknowns and desktop client hints", () => {
    for (const ua of [MAC, WIN, LINUX, "", null, undefined]) expect(detectMobileOs(ua)).toBeNull();
    expect(detectMobileOs(ANDROID, '"Windows"')).toBeNull();
    expect(detectMobileOs(MAC, '"iOS"')).toBe("ios");
    expect(detectMobileOs(WIN, '"Android"')).toBe("android");
  });

  it("never claims a desktop platform as mobile", () => {
    fc.assert(
      fc.property(fc.string(), (ua) => {
        if (detectPlatform(ua) !== null) expect(detectMobileOs(ua)).toBeNull();
      }),
    );
  });
});
