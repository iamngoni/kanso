/**
 * Apple's official "Download on the App Store" badge, unaltered: the black
 * one on light pages and the white one on dark ones, following the theme.
 */
import { cn } from "../lib/cn";
import { APP_STORE_URL } from "../lib/stores";

export function AppStoreBadge({ className }: { className?: string }) {
  return (
    <a
      href={APP_STORE_URL}
      aria-label="Download Kanso Pocket on the App Store"
      className={cn("inline-block rounded-[0.5rem] transition-opacity hover:opacity-85", className)}
    >
      <picture>
        <source srcSet="/badges/app-store-white.svg" media="(prefers-color-scheme: dark)" />
        <img src="/badges/app-store-black.svg" alt="Download on the App Store" width="150" height="50" className="h-[50px] w-auto" />
      </picture>
    </a>
  );
}

/** The Android half: a link to Google Play once there is one, a quiet note until then. */
export function AndroidStore({ url, className }: { url: string | null; className?: string }) {
  if (url) {
    return (
      <a
        href={url}
        aria-label="Get Kanso Pocket on Google Play"
        className={cn("-mx-[10px] -my-[11px] inline-block transition-opacity hover:opacity-85", className)}
      >
        {/* Google's badge carries its own clear space; the negative margins line it up with Apple's. */}
        <img src="/badges/google-play.png" alt="Get it on Google Play" width="186" height="72" className="h-[72px] w-auto" />
      </a>
    );
  }
  return (
    <span className={cn("inline-flex h-[50px] items-center rounded-[0.5rem] border border-line px-4 text-[0.9375rem] text-fg-3", className)}>
      Coming soon to Google Play
    </span>
  );
}
