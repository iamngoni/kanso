/**
 * The home page is the app: a short introduction, then Kanso itself running
 * in the browser on a library that tells its own tour. Below it, what Kanso
 * stands for in one paragraph, the real app in screenshots, and the
 * download. Data comes from `getSiteData`; when GitHub can't be reached the
 * buttons point to its releases page instead.
 */
import { createFileRoute } from "@tanstack/react-router";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { useRef } from "react";
import { AppStoreBadge } from "../components/AppStoreBadge";
import { DownloadCta } from "../components/DownloadCta";
import { Logo } from "../components/Logo";
import { Shot, type ShotSource } from "../components/Shot";
import { KansoWindow } from "../components/app/KansoWindow";
import { APP_STORE_URL } from "../lib/stores";
import { getSiteData } from "../server/fns";

export const Route = createFileRoute("/")({
  loader: () => getSiteData(),
  component: Home,
});

const WINDOW = { width: 2000, height: 1250 } as const;

export const GALLERY: readonly { shot: ShotSource; caption: string }[] = [
  {
    shot: { name: "split", alt: "Kanso's main window: notebooks, a list of notes, and a note in split view.", ...WINDOW },
    caption: "Notebooks, notes, and a note in split view.",
  },
  {
    shot: { name: "tasks", alt: "Kanso's Tasks view: overdue, today and upcoming tasks gathered from every note.", ...WINDOW },
    caption: "Tasks from every note, beside the note they came from.",
  },
  {
    shot: { name: "graph", alt: "Kanso's graph view: notes as dots joined by their links.", ...WINDOW },
    caption: "The graph of how your notes link.",
  },
  {
    shot: { name: "history", dark: false, alt: "A note's version history with the changes since an earlier version highlighted.", ...WINDOW },
    caption: "Every version of a note, and what changed.",
  },
  {
    shot: { name: "palette-headings", dark: false, alt: "Kanso's quick open listing notes and the headings inside them.", ...WINDOW },
    caption: "Quick open finds headings as well as titles.",
  },
  {
    shot: { name: "publish-published", dark: false, alt: "Kanso's publish sheet showing a note's link.", ...WINDOW },
    caption: "Publishing a note, with the key kept in the link.",
  },
];

function Home() {
  const { release, platform } = Route.useLoaderData();
  return (
    <>
      <section className="page pt-[clamp(2rem,5vw,4rem)]">
        <div className="grid items-end gap-8 lg:grid-cols-[minmax(0,1fr)_auto]">
          <div>
            <h1 className="fade-up max-w-[18ch] text-[clamp(2.3rem,1.6rem+2.8vw,3.9rem)] leading-[1.02] font-semibold tracking-[-0.035em]">
              A quiet place for everything you write down.
            </h1>
            <p className="fade-up mt-5 max-w-[36rem] text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal text-fg-2 [--step:1]">
              Kanso is a Markdown notebook for Mac, Windows and Linux. Your notes live on your computer, and sync
              between your devices end-to-end encrypted.
            </p>
            <DownloadCta release={release} platform={platform} className="fade-up mt-8 [--step:2]" />
            <p className="fade-up mt-4 text-[0.875rem] text-fg-3 [--step:2]">
              Also on iPhone and iPad:{" "}
              <a href={APP_STORE_URL} className="ink-link text-fg-2 hover:text-fg">
                Kanso Pocket on the App Store
              </a>
            </p>
          </div>
          <p className="fade-up hidden max-w-[15rem] pb-2 text-[0.875rem] leading-snug text-fg-3 lg:block [--step:3]">
            Below is Kanso itself, sketched in your browser. Type, tick a box, follow a link, or press{" "}
            <span className="kbd">⌘K</span>.
          </p>
        </div>
      </section>

      <section aria-label="Try Kanso" className="page mt-[clamp(2rem,4vw,3rem)]">
        <KansoWindow className="window-in" />
        <p className="mt-4 text-center text-[0.8125rem] text-fg-3">
          A sketch of the app in your browser. The real one is native, and nothing you type here leaves this tab.
        </p>
      </section>

      <Plainly />
      <Gallery />

      <section aria-labelledby="get" className="page pt-[clamp(4rem,10vw,8rem)] pb-[clamp(4rem,10vw,7rem)]">
        <Logo className="size-14 text-fg" />
        <h2 id="get" className="mt-6 text-[clamp(1.9rem,1.4rem+2vw,3rem)] leading-tight font-semibold tracking-[-0.03em]">
          Start with a blank page.
        </h2>
        <p className="mt-3 max-w-[32rem] text-fg-2">Kanso works offline, updates itself, and costs nothing.</p>
        <DownloadCta release={release} platform={platform} className="mt-7" />
        <div className="mt-6 flex flex-wrap items-center gap-x-5 gap-y-3">
          <AppStoreBadge />
          <p className="text-[0.875rem] text-fg-3">Kanso Pocket, for iPhone and iPad</p>
        </div>
      </section>
    </>
  );
}

const PLAINLY: readonly (readonly [string, string])[] = [
  ["Kanso keeps your notes on your computer,", " as Markdown on your own disk, and all of it works offline."],
  [" When you sync, each note is sealed before it leaves,", " so the server holds ciphertext and a few counts, and nothing it could read."],
  [" It's a native app,", " written in Rust and drawn by your graphics card, so it's quick and keeps out of the way."],
  [" And it's free software:", " if we ever stop, the code doesn't."],
];

function Plainly() {
  return (
    <section aria-label="What Kanso stands for" className="page pt-[clamp(5rem,12vw,10rem)]">
      <p className="max-w-[52rem] text-[clamp(1.45rem,1.1rem+1.5vw,2.35rem)] leading-[1.28] font-medium tracking-[-0.02em]">
        {PLAINLY.map(([lead, rest]) => (
          <span key={lead}>
            <span className="text-fg">{lead}</span>
            <span className="text-fg-3">{rest}</span>
          </span>
        ))}
      </p>
    </section>
  );
}

function Gallery() {
  const strip = useRef<HTMLUListElement>(null);
  const move = (direction: 1 | -1) => {
    const el = strip.current;
    if (el) el.scrollBy({ left: direction * el.clientWidth * 0.8, behavior: "smooth" });
  };
  const arrow = "grid size-10 cursor-pointer place-items-center rounded-full border border-line text-fg-2 hover:border-fg-3 hover:text-fg";
  return (
    <section aria-labelledby="real" className="pt-[clamp(5rem,12vw,9rem)]">
      <div className="page flex items-end justify-between gap-6">
        <div>
          <h2 id="real" className="text-[clamp(1.5rem,1.2rem+1vw,2rem)] font-semibold tracking-[-0.02em]">
            The real thing.
          </h2>
          <p className="mt-2 max-w-[32rem] text-fg-2">The window above is a sketch. This is Kanso on a Mac, as the app draws itself.</p>
        </div>
        <div className="hidden gap-2 sm:flex">
          <button type="button" onClick={() => move(-1)} aria-label="Previous screenshots" className={arrow}>
            <ArrowLeft className="size-4" />
          </button>
          <button type="button" onClick={() => move(1)} aria-label="Next screenshots" className={arrow}>
            <ArrowRight className="size-4" />
          </button>
        </div>
      </div>
      <ul
        ref={strip}
        className="mt-8 flex snap-x snap-mandatory gap-5 overflow-x-auto px-[var(--edge)] pb-6 [--edge:max(clamp(1rem,4vw,3rem),calc((100%_-_82rem)/2_+_clamp(1rem,4vw,3rem)))] [scroll-padding-inline:var(--edge)] [scrollbar-width:none]"
      >
        {GALLERY.map((item, i) => (
          <li key={item.shot.name} className="w-[min(46rem,86vw)] shrink-0 snap-start">
            <Shot shot={item.shot} figure={`${i + 1}`} caption={item.caption} />
          </li>
        ))}
      </ul>
    </section>
  );
}
