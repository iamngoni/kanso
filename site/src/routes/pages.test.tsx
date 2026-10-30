/**
 * Tests for the release notes, help and privacy pages: what each shows, the
 * fallback when GitHub can't be read, that release text is rendered as text
 * and never as HTML, that the help topics are complete and linked, and that
 * privacy links the usage-count note and documents.
 */
import type { ReactNode } from "react";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

// Routes are built with the real router; only <Link> becomes a plain anchor.
vi.mock("@tanstack/react-router", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@tanstack/react-router")>()),
  Link: ({ to, hash, children, ...rest }: { to: string; hash?: string; children?: ReactNode; className?: string }) => (
    <a href={hash ? `${to}#${hash}` : to} {...rest}>
      {children}
    </a>
  ),
}));
vi.mock("../server/fns", () => ({ getSiteData: vi.fn(), getReleaseNotes: vi.fn() }));

import { RELEASES_URL, REPOSITORY_URL } from "../lib/release";
import { Help, ISSUES_URL, TOPICS } from "./help";
import { Privacy } from "./privacy";
import { ReleaseNotes } from "./releases";

const note = (version: string, body: string) => ({
  version,
  publishedAt: "2026-10-20T09:30:00Z",
  pageUrl: `${RELEASES_URL}/tag/v${version}`,
  body,
});

describe("ReleaseNotes", () => {
  it("lists each release with its date, link and notes", () => {
    render(<ReleaseNotes releases={[note("0.9.0", "### Features\n\n- Mobile apps ([abc](https://github.com/iamngoni/kanso/commit/abc))")]} />);
    expect(screen.getByRole("heading", { level: 2, name: "0.9.0" })).toBeInTheDocument();
    expect(screen.getByText("20 October 2026")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "On GitHub" })).toHaveAttribute("href", `${RELEASES_URL}/tag/v0.9.0`);
    expect(screen.getByRole("heading", { level: 4, name: "Features" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "abc" })).toHaveAttribute("href", "https://github.com/iamngoni/kanso/commit/abc");
  });

  it("renders notes as text, with no scripts and no unsafe links", () => {
    const { container } = render(<ReleaseNotes releases={[note("0.9.0", "<script>alert(1)</script>\n\n[x](javascript:alert(1))")]} />);
    expect(container.querySelector("script")).toBeNull();
    expect(screen.getByTestId("release-body")).toHaveTextContent("<script>alert(1)</script>");
    expect(screen.queryByRole("link", { name: "x" })).toBeNull();
  });

  it("falls back to GitHub when the list is missing or empty", () => {
    for (const releases of [null, []]) {
      const { unmount } = render(<ReleaseNotes releases={releases} />);
      expect(screen.getByText(/couldn't be loaded/)).toBeInTheDocument();
      expect(screen.getByRole("link", { name: /releases page on GitHub/ })).toHaveAttribute("href", RELEASES_URL);
      expect(screen.queryByTestId("release-body")).toBeNull();
      unmount();
    }
  });
});

describe("Help", () => {
  it("has every topic, each linked from the contents", () => {
    render(<Help />);
    const toc = within(screen.getByRole("navigation", { name: "Topics" }));
    for (const topic of TOPICS) {
      expect(toc.getByRole("link", { name: topic.title })).toHaveAttribute("href", `#${topic.id}`);
      expect(screen.getByRole("heading", { level: 2, name: topic.title })).toBeInTheDocument();
    }
    expect(new Set(TOPICS.map((t) => t.id)).size).toBe(TOPICS.length);
  });

  it("covers the topics the apps point people to", () => {
    render(<Help />);
    for (const title of ["Syncing, and adding a device", "Locked notes", "Sharing", "Crash reports", "Deleting your account", "Getting support"]) {
      expect(screen.getByRole("heading", { level: 2, name: title })).toBeInTheDocument();
    }
  });

  it("points to GitHub issues for support and says what crash reports leave out", () => {
    render(<Help />);
    expect(screen.getByRole("link", { name: /issues on GitHub/ })).toHaveAttribute("href", ISSUES_URL);
    expect(screen.getByText(/never holds note text, titles or file names/)).toBeInTheDocument();
  });
});

describe("Privacy", () => {
  it("says the website has no analytics and explains the apps' usage counts", () => {
    const { container } = render(<Privacy />);
    expect(screen.getByText(/No analytics, no cookies/)).toBeInTheDocument();
    const section = container.querySelector("section#metrics") as HTMLElement;
    expect(within(section).getByRole("heading", { level: 2, name: "Usage counts" })).toBeInTheDocument();
    expect(within(section).getByText("Your IP address")).toBeInTheDocument();
    expect(within(section).getByRole("link", { name: "What's sent, and what isn't" })).toHaveAttribute("href", "/metrics");
    expect(within(section).getByRole("link", { name: "the usage counts document" })).toHaveAttribute(
      "href",
      `${REPOSITORY_URL}/blob/master/docs/METRICS.md`,
    );
  });
});
