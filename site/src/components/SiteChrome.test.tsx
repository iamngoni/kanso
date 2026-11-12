/**
 * Tests for the shared header and footer: navigation links and their
 * targets, the wordmark home link, and the header's hairline appearing
 * once the page scrolls (and the scroll listener being removed on unmount).
 * Router links are mocked to plain anchors.
 */
import "../test/router";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { REPOSITORY_URL } from "../lib/release";
import { SiteFooter, SiteHeader, Wordmark } from "./SiteChrome";

const setScroll = (y: number) => {
  Object.defineProperty(window, "scrollY", { value: y, configurable: true });
  act(() => void window.dispatchEvent(new Event("scroll")));
};

afterEach(() => Object.defineProperty(window, "scrollY", { value: 0, configurable: true }));

describe("Wordmark", () => {
  it("links home", () => {
    render(<Wordmark className="x" />);
    const link = screen.getByRole("link", { name: "Kanso, home" });
    expect(link).toHaveAttribute("href", "/");
    expect(link).toHaveClass("x");
    expect(link.querySelector("svg")).not.toBeNull();
  });
});

describe("SiteHeader", () => {
  it("has the main navigation links", () => {
    render(<SiteHeader />);
    const nav = within(screen.getByRole("navigation", { name: "Main" }));
    expect(nav.getByRole("link", { name: "Help" })).toHaveAttribute("href", "/help");
    expect(nav.getByRole("link", { name: "Price" })).toHaveAttribute("href", "/price");
    expect(nav.getByRole("link", { name: "Privacy" })).toHaveAttribute("href", "/privacy");
    expect(nav.getByRole("link", { name: "GitHub" })).toHaveAttribute("href", REPOSITORY_URL);
    expect(nav.getByRole("link", { name: "Download" })).toHaveAttribute("href", "/download");
  });

  it("shows a hairline only once scrolled, and hides it again at the top", () => {
    render(<SiteHeader />);
    const header = screen.getByRole("banner");
    expect(header).toHaveAttribute("data-scrolled", "false");
    expect(header).toHaveClass("border-transparent");
    setScroll(200);
    expect(header).toHaveAttribute("data-scrolled", "true");
    expect(header).toHaveClass("border-line");
    setScroll(8);
    expect(header).toHaveAttribute("data-scrolled", "false");
    fireEvent.scroll(window);
  });

  it("starts scrolled when mounted mid-page, and stops listening on unmount", () => {
    Object.defineProperty(window, "scrollY", { value: 500, configurable: true });
    const { unmount } = render(<SiteHeader />);
    expect(screen.getByRole("banner")).toHaveAttribute("data-scrolled", "true");
    unmount();
    expect(() => setScroll(0)).not.toThrow();
  });
});

describe("SiteFooter", () => {
  it("has the footer links", () => {
    render(<SiteFooter />);
    const nav = within(screen.getByRole("navigation", { name: "Footer" }));
    expect(nav.getByRole("link", { name: "Download" })).toHaveAttribute("href", "/download");
    expect(nav.getByRole("link", { name: "Price" })).toHaveAttribute("href", "/price");
    expect(nav.getByRole("link", { name: "Privacy" })).toHaveAttribute("href", "/privacy");
    expect(nav.getByRole("link", { name: "Release notes" })).toHaveAttribute("href", "/releases");
    expect(nav.getByRole("link", { name: "Help" })).toHaveAttribute("href", "/help");
    expect(nav.getByRole("link", { name: "Source code" })).toHaveAttribute("href", REPOSITORY_URL);
    expect(screen.getByRole("contentinfo")).toHaveTextContent("AGPL");
  });
});
