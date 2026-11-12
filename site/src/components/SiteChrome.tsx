/**
 * The header and footer every page shares: the mark and name on the left,
 * a few plain links on the right, and a hairline under the header once the
 * page scrolls. Nothing collapses into a menu; there's too little to hide.
 */
import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { cn } from "../lib/cn";
import { REPOSITORY_URL } from "../lib/release";
import { Logo } from "./Logo";

export function Wordmark({ className }: { className?: string }) {
  return (
    <Link to="/" className={cn("inline-flex items-center gap-2 text-fg", className)} aria-label="Kanso, home">
      <Logo className="size-[22px]" />
      <span className="text-[1.0625rem] font-semibold tracking-[-0.01em]">Kanso</span>
    </Link>
  );
}

export function SiteHeader() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const update = () => setScrolled(window.scrollY > 8);
    update();
    window.addEventListener("scroll", update, { passive: true });
    return () => window.removeEventListener("scroll", update);
  }, []);

  const link = "rounded-md px-2.5 py-1.5 text-fg-2 transition-colors hover:text-fg";
  return (
    <header
      data-scrolled={scrolled}
      className={cn(
        "sticky top-0 z-40 border-b bg-app/90 backdrop-blur-md transition-colors duration-300",
        scrolled ? "border-line" : "border-transparent",
      )}
    >
      <div className="page flex h-14 items-center justify-between gap-4">
        <Wordmark />
        <nav aria-label="Main" className="flex items-center gap-0.5 text-[0.9375rem] sm:gap-1">
          <Link to="/help" className={cn(link, "hidden sm:inline-block")}>
            Help
          </Link>
          <Link to="/price" className={cn(link, "hidden sm:inline-block")}>
            Price
          </Link>
          <Link to="/privacy" className={cn(link, "hidden sm:inline-block")}>
            Privacy
          </Link>
          <a href={REPOSITORY_URL} className={link}>
            GitHub
          </a>
          <Link to="/download" className={cn(link, "font-medium text-fg")}>
            Download
          </Link>
        </nav>
      </div>
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t border-line">
      <div className="page flex flex-col gap-6 py-10 text-[0.875rem] text-fg-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3">
          <Logo className="size-5 text-fg-2" />
          <span>Kanso is free software under the AGPL-3.0.</span>
        </div>
        <nav aria-label="Footer" className="flex flex-wrap gap-x-6 gap-y-2 text-fg-2">
          <Link to="/download" className="ink-link">
            Download
          </Link>
          <Link to="/price" className="ink-link">
            Price
          </Link>
          <Link to="/privacy" className="ink-link">
            Privacy
          </Link>
          <Link to="/releases" className="ink-link">
            Release notes
          </Link>
          <Link to="/help" className="ink-link">
            Help
          </Link>
          <a href={REPOSITORY_URL} className="ink-link">
            Source code
          </a>
        </nav>
      </div>
    </footer>
  );
}
