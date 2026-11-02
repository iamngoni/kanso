/**
 * Tests for the download page's Kanso Pocket section: the official App Store
 * badge links to the store, Android shows "Coming soon" without a link while
 * ANDROID_URL is null, and the section leads (as "Your device") on a phone.
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

import { APP_STORE_URL } from "../lib/stores";
import { Download } from "./download";

describe("Download page, mobile section", () => {
  it("links the App Store badge and shows Android as coming soon", () => {
    render(<Download release={null} platform="macos" />);
    expect(screen.getByRole("link", { name: /App Store/ })).toHaveAttribute("href", APP_STORE_URL);
    expect(screen.getByAltText("Download on the App Store")).toBeInTheDocument();
    expect(screen.getByText("Coming soon to Google Play")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /Google Play/ })).toBeNull();
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
