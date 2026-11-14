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
import { Price } from "./price";
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
    for (const title of ["Syncing, and adding a device", "Locked notes", "Your library key", "Dates and repeating tasks", "Running a video studio", "Capturing from anywhere on a phone", "Connecting an agent or a script", "Sharing", "Sharing a notebook", "Crash reports", "Deleting your account", "Getting support"]) {
      expect(screen.getByRole("heading", { level: 2, name: title })).toBeInTheDocument();
    }
  });

  it("explains pairing, repeats, the studio and shared notebooks in the apps' own words", () => {
    const { container } = render(<Help />);
    const topic = (id: string) => container.querySelector(`section#${id}`) as HTMLElement;
    expect(within(topic("api-keys")).getByText("Raycast wants to connect")).toBeInTheDocument();
    expect(within(topic("tasks")).getByText("Captured tasks go to")).toBeInTheDocument();
    expect(within(topic("tasks")).getByText(/@2026-11-21T15:00 every week/)).toBeInTheDocument();
    expect(within(topic("studio")).getByText("Video studio")).toBeInTheDocument();
    expect(within(topic("studio")).getByText("Board settings")).toBeInTheDocument();
    for (const role of ["Owner", "Manager", "Can edit", "Can comment", "Can view"]) {
      expect(within(topic("shared-notebooks")).getByText(role)).toBeInTheDocument();
    }
    expect(within(topic("shared-notebooks")).getByText("Stop sharing")).toBeInTheDocument();
    expect(within(topic("sharing")).getByRole("link", { name: "Sharing a notebook" })).toHaveAttribute("href", "#shared-notebooks");
  });

  it("explains the embargo, the old-app warning and the phone shortcuts", () => {
    const { container } = render(<Help />);
    const topic = (id: string) => container.querySelector(`section#${id}`) as HTMLElement;
    expect(within(topic("studio")).getByText("Publish anyway")).toBeInTheDocument();
    expect(within(topic("studio")).getByText(/Embargoed until Wed 18 Nov, 09:00 New York/)).toBeInTheDocument();
    expect(within(topic("shared-notebooks")).getByText("Update Kanso to use shared notebooks")).toBeInTheDocument();
    expect(within(topic("shared-notebooks")).getByText(/Update Kanso to open this shared notebook\./)).toBeInTheDocument();
    for (const action of ["Capture in Kanso", "Add Kanso task", "Search Kanso", "Open Kanso note"]) {
      expect(within(topic("phone-shortcuts")).getByText(action)).toBeInTheDocument();
    }
    expect(within(topic("phone-shortcuts")).getAllByText("Add task").length).toBeGreaterThan(0);
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

  it("explains feedback: what's sent, what's never sent, who reads it, how long it's kept", () => {
    const { container } = render(<Privacy />);
    const section = container.querySelector("section#feedback") as HTMLElement;
    expect(within(section).getByRole("heading", { level: 2, name: "Feedback" })).toBeInTheDocument();
    expect(within(section).getByRole("heading", { level: 3, name: "It sends" })).toBeInTheDocument();
    expect(within(section).getByRole("heading", { level: 3, name: "It never sends" })).toBeInTheDocument();
    expect(within(section).getByText(/Note titles or text, notebook or tag names, file names or your library/)).toBeInTheDocument();
    expect(within(section).getByText(/Your email address, your IP address or a device ID/)).toBeInTheDocument();
    expect(within(section).getByRole("heading", { level: 3, name: "Your report stays private" })).toBeInTheDocument();
    expect(within(section).getByText(/never\s+put on GitHub/)).toBeInTheDocument();
    expect(within(section).getByText(/Claude Haiku, from Anthropic.*Cloudflare AI Gateway/s)).toBeInTheDocument();
    expect(within(section).getByText(/short summary as an issue on Kanso's GitHub/)).toBeInTheDocument();
    expect(within(section).getByText(/deleted 180 days after/)).toBeInTheDocument();
    expect(within(section).getByText(/delete a report yourself at any time/)).toBeInTheDocument();
    // A merged "Reported-by" trailer can't be taken back; the page must not promise it can.
    expect(within(section).getByText(/stays in Kanso's code history and in that release's notes for good/)).toBeInTheDocument();
    expect(within(section).getByText(/not out of fixes that\s+already shipped/)).toBeInTheDocument();
    expect(within(section).getByRole("link", { name: "the feedback document" })).toHaveAttribute(
      "href",
      `${REPOSITORY_URL}/blob/master/docs/FEEDBACK.md`,
    );
  });
});

describe("Price", () => {
  it("says Kanso is free, states the sync limits and the self-hosting and change promises", () => {
    render(<Price />);
    expect(screen.getByRole("heading", { level: 1, name: "What Kanso costs" })).toBeInTheDocument();
    expect(screen.getByText(/no subscription, no trial and no paid\s+tier/)).toBeInTheDocument();
    expect(screen.getByText(/up to 100 MiB of notes and\s+1 GiB of files, and upload up to 1 GiB a day/)).toBeInTheDocument();
    expect(screen.getByText(/Shared notebooks count against their\s+owner's space/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 2, name: "You can run your own sync server" })).toBeInTheDocument();
    expect(screen.getByText(/It will say so here first, and nothing you already have will stop working/)).toBeInTheDocument();
  });
});
