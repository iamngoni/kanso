/**
 * Tests for the download page's Kanso Pocket section: the official App Store
 * badge links to the store, the Google Play badge links to the listing, and the
 * section leads (as "Your device") on a phone.
 */
import type { ReactNode } from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-router", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@tanstack/react-router")>()),
  Link: ({ to, children, ...rest }: { to: string; children?: ReactNode; className?: string }) => (
    <a href={to} {...rest}>
      {children}
    </a>
  ),
}));
vi.mock("../server/fns", () => ({ getSiteData: vi.fn() }));

import { ANDROID_URL, APP_STORE_URL } from "../lib/stores";
import type { Release } from "../lib/release";
import { Download } from "./download";

describe("Download page, mobile section", () => {
  it("links both store badges", () => {
    render(<Download release={null} platform="macos" />);
    expect(screen.getByRole("link", { name: /App Store/ })).toHaveAttribute("href", APP_STORE_URL);
    expect(screen.getByAltText("Download on the App Store")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Google Play/ })).toHaveAttribute("href", ANDROID_URL);
    expect(screen.queryByText("Coming soon to Google Play")).toBeNull();
    expect(screen.queryByText("Your device")).toBeNull();
  });

  it("highlights the section as the visitor's device on iOS and puts it first", () => {
    render(<Download release={null} platform={null} mobile="ios" />);
    expect(screen.getByText("Your device")).toBeInTheDocument();
    const heading = screen.getByRole("heading", { name: /Kanso Pocket, for your phone/ });
    const title = screen.getByRole("heading", { level: 1 });
    expect(title.compareDocumentPosition(heading) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});

describe("Download page, What's new video", () => {
  const BASE = "https://github.com/iamngoni/kanso/releases/download/v0.10.9";
  const release = (videoUrl: string | null): Release => ({
    version: "0.10.9",
    publishedAt: "2026-11-16T12:00:00Z",
    pageUrl: "https://github.com/iamngoni/kanso/releases/tag/v0.10.9",
    builds: [
      { kind: "macos-dmg", platform: "macos", label: "Disk image (.dmg)", detail: "", file: "Kanso-0.10.9-macos-arm64.dmg", url: `${BASE}/Kanso-0.10.9-macos-arm64.dmg`, size: 50_000_000 },
    ],
    checksumsUrl: null,
    videoUrl,
  });

  it("shows the latest release's video when it has one", () => {
    render(<Download release={release(`${BASE}/Kanso-0.10.9-whats-new.webp`)} platform="macos" />);
    expect(screen.getAllByAltText("What's new in Kanso 0.10.9")[0]).toHaveAttribute("src", `${BASE}/Kanso-0.10.9-whats-new.webp`);
  });

  it("shows none when the release has no video", () => {
    render(<Download release={release(null)} platform="macos" />);
    expect(screen.queryByAltText(/What's new in Kanso/)).toBeNull();
  });
});
