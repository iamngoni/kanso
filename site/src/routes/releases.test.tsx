/**
 * Tests for the release notes page's What's new video: a release with a
 * video asset leads with it, sized and lazy, and a release without one shows
 * only its notes.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../server/fns", () => ({ getReleaseNotes: vi.fn() }));

import type { ReleaseNote } from "../lib/releases";
import { ReleaseNotes } from "./releases";

const VIDEO = "https://github.com/iamngoni/kanso/releases/download/v0.10.9/Kanso-0.10.9-whats-new.webp";

const note = (version: string, videoUrl: string | null): ReleaseNote => ({
  version,
  publishedAt: "2026-11-16T12:00:00Z",
  pageUrl: `https://github.com/iamngoni/kanso/releases/tag/v${version}`,
  body: "### New\n\n- Something",
  videoUrl,
});

describe("Release notes page, What's new video", () => {
  it("shows a release's video, sized and lazy, from its release download", () => {
    render(<ReleaseNotes releases={[note("0.10.9", VIDEO), note("0.10.8", null)]} />);
    // The playing image and the reduced-motion one behind "Play"; CSS shows one.
    const images = screen.getAllByAltText("What's new in Kanso 0.10.9");
    expect(images.length).toBeGreaterThan(0);
    for (const img of images) {
      expect(img).toHaveAttribute("src", VIDEO);
      expect(img).toHaveAttribute("width", "640");
      expect(img).toHaveAttribute("height", "360");
      expect(img).toHaveAttribute("loading", "lazy");
    }
    expect(screen.getByText(/Play the video/)).toBeInTheDocument();
  });

  it("shows no video for a release without one", () => {
    render(<ReleaseNotes releases={[note("0.10.8", null)]} />);
    expect(screen.queryByAltText(/What's new in Kanso/)).toBeNull();
    expect(screen.queryByText(/Play the video/)).toBeNull();
  });
});
