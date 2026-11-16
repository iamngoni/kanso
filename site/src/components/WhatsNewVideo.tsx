/**
 * A release's What's new video: the animated WebP the release carries
 * (Kanso-<v>-whats-new.webp, docs/RELEASING.md › What's new video), loaded
 * straight from GitHub's release downloads, as the apps do.
 *
 * An animated WebP can't be paused, so under prefers-reduced-motion it isn't
 * shown playing: it sits behind a "Play" disclosure the visitor opens. That
 * is done in CSS (Tailwind's motion-safe and motion-reduce), so it is right
 * on the first paint, before any script runs. The image a visitor doesn't
 * see is display:none and lazy, so the browser doesn't fetch it.
 */
import { cn } from "../lib/cn";

/** The video's pixel size (scripts/whats-new-video.sh), so the browser reserves the space. */
export const VIDEO_WIDTH = 640;
export const VIDEO_HEIGHT = 360;

export interface WhatsNewVideoProps {
  /** The trusted release download URL (lib/release.ts › whatsNewUrl). */
  url: string;
  version: string;
  className?: string;
}

export function WhatsNewVideo({ url, version, className }: WhatsNewVideoProps) {
  const alt = `What's new in Kanso ${version}`;
  // Rounded and ringed like the site's screenshots (components/Shot.tsx).
  const image = (
    <img
      src={url}
      alt={alt}
      width={VIDEO_WIDTH}
      height={VIDEO_HEIGHT}
      loading="lazy"
      decoding="async"
      className="block aspect-video h-auto w-full rounded-[10px] bg-sidebar ring-1 ring-line"
    />
  );
  return (
    <figure className={cn("m-0 max-w-2xl", className)}>
      {/* Plays as soon as it's in view, for visitors who don't ask for less motion. */}
      <div className="motion-reduce:hidden">{image}</div>
      {/* For those who do: a still label until they choose to play it. */}
      <details className="hidden motion-reduce:block">
        <summary className="cursor-pointer text-[0.9375rem] text-fg-2">Play the video: {alt}</summary>
        <div className="mt-3">{image}</div>
      </details>
    </figure>
  );
}
