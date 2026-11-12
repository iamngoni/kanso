/**
 * Price: what Kanso costs (nothing), what sync includes and its limits, and a
 * promise to say so here first if that ever changes. The sync limits come from
 * src/lib/quota.ts, which mirrors docs/ACCOUNTS.md ("Quotas and capacity").
 */
import { createFileRoute } from "@tanstack/react-router";
import { QUOTA_FILES_BYTES, QUOTA_NOTES_BYTES, QUOTA_UPLOAD_PER_DAY_BYTES, formatSize } from "../lib/quota";
import { REPOSITORY_URL } from "../lib/release";

export const Route = createFileRoute("/price")({
  head: () => ({
    meta: [
      { title: "What Kanso costs — Kanso" },
      { name: "description", content: "Kanso is free software under the AGPL-3.0. No subscription, no trial and no paid tier. Sync is included, with stated limits." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/price" }],
  }),
  component: Price,
});

const h2 = "text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]";

export function Price() {
  const limits = [
    ["Notes", `${formatSize(QUOTA_NOTES_BYTES)} per account`],
    ["Files", `${formatSize(QUOTA_FILES_BYTES)} per account`],
    ["Uploads", `${formatSize(QUOTA_UPLOAD_PER_DAY_BYTES)} a day`],
  ];
  return (
    <article className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="text-[0.875rem] font-medium text-fg-3">Price</p>
      <h1 className="mt-3 max-w-3xl text-[clamp(2.2rem,1.6rem+2.6vw,3.6rem)] leading-[1.05] font-semibold tracking-[-0.035em]">What Kanso costs</h1>
      <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
        <strong className="font-semibold text-fg">Nothing.</strong> Kanso is free software under the AGPL-3.0. There's no subscription, no trial and no paid
        tier, on the desktop or the phone.
      </p>

      <div className="mt-[clamp(3.5rem,8vw,6rem)] grid gap-14 lg:grid-cols-12">
        <section aria-labelledby="sync" className="lg:col-span-7">
          <h2 id="sync" className={h2}>
            Sync is included
          </h2>
          <p className="mt-4 text-fg-2">
            Notes and files are end-to-end encrypted before they leave your device. Each account can hold up to {formatSize(QUOTA_NOTES_BYTES)} of notes and{" "}
            {formatSize(QUOTA_FILES_BYTES)} of files, and upload up to {formatSize(QUOTA_UPLOAD_PER_DAY_BYTES)} a day. Shared notebooks count against their
            owner's space.
          </p>
          <dl className="mt-8 grid gap-5">
            {limits.map(([term, detail]) => (
              <div key={term} className="grid gap-1 border-t border-line pt-4 sm:grid-cols-[11rem_1fr] sm:gap-6">
                <dt className="font-medium text-fg">{term}</dt>
                <dd className="text-fg-2">{detail}</dd>
              </div>
            ))}
          </dl>
        </section>

        <div className="grid content-start gap-14 lg:col-span-4 lg:col-start-9">
          <section aria-labelledby="self-host">
            <h2 id="self-host" className={h2}>
              You can run your own sync server
            </h2>
            <p className="mt-4 text-fg-2">
              The server is in the same repository, under the same licence.{" "}
              <a href={REPOSITORY_URL} className="ink-link">
                Source code
              </a>
            </p>
          </section>
          <section aria-labelledby="change">
            <h2 id="change" className={h2}>
              If this ever changes
            </h2>
            <p className="mt-4 text-fg-2">It will say so here first, and nothing you already have will stop working.</p>
          </section>
        </div>
      </div>
    </article>
  );
}
