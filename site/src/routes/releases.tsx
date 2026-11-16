/**
 * Release notes: the last twenty releases from GitHub, newest first, each
 * with the notes it was published with. The desktop apps and the iPhone,
 * iPad and Android apps share version numbers, so one list covers them all.
 * When GitHub can't be read, the page says so and links to its releases.
 */
import { createFileRoute } from "@tanstack/react-router";
import { ReleaseBody } from "../components/ReleaseBody";
import { WhatsNewVideo } from "../components/WhatsNewVideo";
import { RELEASES_URL, formatDate } from "../lib/release";
import type { ReleaseNote } from "../lib/releases";
import { getReleaseNotes } from "../server/fns";

export const Route = createFileRoute("/releases")({
  loader: () => getReleaseNotes(),
  head: () => ({
    meta: [
      { title: "Release notes — Kanso" },
      { name: "description", content: "What changed in each version of Kanso, on the desktop, iPhone, iPad and Android." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/releases" }],
  }),
  component: ReleasesPage,
});

function ReleasesPage() {
  const releases = Route.useLoaderData();
  return <ReleaseNotes releases={releases} />;
}

export function ReleaseNotes({ releases }: { releases: readonly ReleaseNote[] | null }) {
  const available = releases !== null && releases.length > 0;
  return (
    <section className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="text-[0.875rem] font-medium text-fg-3">Release notes</p>
      <h1 className="mt-3 max-w-3xl text-[clamp(2.2rem,1.6rem+2.6vw,3.6rem)] leading-[1.05] font-semibold tracking-[-0.035em]">What's new in Kanso.</h1>
      <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
        {available ? (
          <>
            Kanso on your computer, iPhone, iPad and Android shares one version number, so these notes cover them all.
            Older versions are on{" "}
            <a href={RELEASES_URL} className="ink-link">
              GitHub
            </a>
            .
          </>
        ) : (
          <>
            The release notes couldn't be loaded just now. Every release is on{" "}
            <a href={RELEASES_URL} className="ink-link">
              Kanso's releases page on GitHub
            </a>
            .
          </>
        )}
      </p>

      {available && (
        <div className="mt-[clamp(3rem,7vw,5rem)] grid max-w-3xl gap-14">
          {releases.map((release) => (
            <article key={release.version} aria-labelledby={`v${release.version}`} className="border-t border-line pt-6">
              <h2 id={`v${release.version}`} className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
                {release.version}
              </h2>
              <p className="mt-1 text-[0.9375rem] text-fg-3">
                <time dateTime={release.publishedAt}>{formatDate(release.publishedAt)}</time>
                {" · "}
                <a href={release.pageUrl} className="ink-link">
                  On GitHub
                </a>
              </p>
              {/* A release that carries a video leads with it, as the apps' What's new sheet does. */}
              {release.videoUrl && <WhatsNewVideo url={release.videoUrl} version={release.version} className="mt-5" />}
              <div className="mt-5">
                <ReleaseBody body={release.body} />
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
