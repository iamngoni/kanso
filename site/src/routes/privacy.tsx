/**
 * Privacy, in plain words: what this website collects (nothing), what
 * Kanso's sync server can and can't see, and what the apps send. It summarises
 * docs/ACCOUNTS.md ("What the server can and can't see"), docs/CRASH-REPORTS.md
 * docs/FEEDBACK.md and docs/METRICS.md (with its note, docs/METRICS-NOTE.md), and must change
 * when they do.
 */
import { createFileRoute } from "@tanstack/react-router";
import { REPOSITORY_URL } from "../lib/release";

export const Route = createFileRoute("/privacy")({
  head: () => ({
    meta: [
      { title: "Privacy — Kanso" },
      { name: "description", content: "What Kanso's website and sync server can and can't see." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/privacy" }],
  }),
  component: Privacy,
});

const CAN_SEE = [
  ["Your account", "Your username, when the account was made, and whether it's active."],
  ["Sizes and times", "How many notes, notebooks, tags and files you have, roughly how big each is, and when each was written."],
  ["Your devices", "How many devices are signed in and when each was last seen, to the hour. Not their names: those are encrypted."],
  ["The network", "Your IP address, as any server does while handling a request. It is kept only as a keyed hash in rate-limit records, for at most two days."],
] as const;

const CANNOT_SEE = [
  "Note titles or text",
  "Notebook and tag names",
  "Which notes are in which notebook or carry which tag",
  "Links between notes",
  "File names or contents",
  "Your password, your recovery key or the key your notes are encrypted with",
] as const;

const COUNTS_SEND = [
  "The kind of device and its system version, and Kanso's version and release channel",
  "Whether sync is on and whose server it uses, how many sync errors there were, and whether AI is turned on",
  "Your language, your theme and editor mode as a plain label (a theme you made or downloaded is just \"custom\")",
  "How many notes, notebooks and plugins you have, and how much room attachments take, each as a range",
  "That Kanso was opened, and how long it was open, as a range",
  "Which features were used since the last count (creating, editing or deleting a note, search, links, tasks, sync and the like), and roughly how often, as a range, never what with",
] as const;

const COUNTS_NEVER = [
  "Notes, titles, notebooks, tags, files or links",
  "Anything you type, search for or ask AI",
  "The name of a theme or a plugin",
  "Your name, username or email address",
  "Your IP address",
] as const;

const FEEDBACK_SEND = [
  "What you typed in the box, and whether it is a bug, an idea or a question",
  "If you tick \"Include app info\": Kanso's version, your system and device model, your language, which screen you were on (a label such as \"editor\", never a note), whether sync is on, and up to 20 short lines about recent errors",
  "If you tick \"Credit me by name\": the name you chose, as plain text",
  "If you attach one: a screenshot you picked or captured, shown to you before it goes",
] as const;

const FEEDBACK_NEVER = [
  "Note titles or text, notebook or tag names, file names or your library",
  "Your email address, your IP address or a device ID",
  "Anything at all until you press Send, after the app shows you the exact report",
] as const;

export function Privacy() {
  return (
    <article className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="text-[0.875rem] font-medium text-fg-3">Privacy</p>
      <h1 className="mt-3 max-w-3xl text-[clamp(2.2rem,1.6rem+2.6vw,3.6rem)] leading-[1.05] font-semibold tracking-[-0.035em]">We'd rather not know.</h1>
      <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
        Kanso keeps your notes on your computer. If you sync, they are encrypted on your device first, so the server holds
        only what it needs to pass them between your devices.
      </p>

      <div className="mt-[clamp(3.5rem,8vw,6rem)] grid gap-16 lg:grid-cols-12">
        <section aria-labelledby="site" className="lg:col-span-4">
          <h2 id="site" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
            This website
          </h2>
          <p className="mt-4 text-fg-2">
            No analytics, no cookies, no trackers, no forms. The server reads your browser's User-Agent to suggest the right
            download and then forgets it.
          </p>
        </section>

        <section aria-labelledby="server" className="lg:col-span-7 lg:col-start-6">
          <h2 id="server" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
            The sync server
          </h2>
          <h3 className="mt-8 text-[0.8125rem] font-semibold text-fg-3">It can't see</h3>
          <ul className="mt-4 grid gap-x-8 gap-y-2.5 sm:grid-cols-2">
            {CANNOT_SEE.map((item) => (
              <li key={item} className="flex gap-3 text-fg">
                <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-fg-3" />
                {item}
              </li>
            ))}
          </ul>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">It can see</h3>
          <dl className="mt-4 grid gap-5">
            {CAN_SEE.map(([term, detail]) => (
              <div key={term} className="grid gap-1 border-t border-line pt-4 sm:grid-cols-[11rem_1fr] sm:gap-6">
                <dt className="font-medium text-fg">{term}</dt>
                <dd className="text-fg-2">{detail}</dd>
              </div>
            ))}
          </dl>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">Published notes</h3>
          <p className="mt-4 text-fg-2">
            A published note is encrypted too, and its key lives only in the link you share. To give the link a short address
            and a preview, Kanso sends the note's title, and never its key or text, to the link service.
          </p>
          <p className="mt-10 text-[0.9375rem] text-fg-3">
            Every detail, including how the encryption works, is in{" "}
            <a href={`${REPOSITORY_URL}/blob/master/docs/ACCOUNTS.md`} className="ink-link">
              the accounts design document
            </a>
            . You can also run the sync server on your own Cloudflare account.
          </p>
        </section>

        <section id="crash-reports" aria-labelledby="crashes" className="lg:col-span-7 lg:col-start-6">
          <h2 id="crashes" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
            Crash reports
          </h2>
          <p className="mt-4 text-fg-2">
            When Kanso on an iPhone, iPad or Android device crashes, it writes a report on the device and nothing leaves it. A
            report is sent only if you tap Send, and the app shows you the exact text first.
          </p>
          <p className="mt-4 text-fg-2">
            It holds where in Kanso's code the failure happened, the app version, the system version, the kind of device and the
            day, and never note text, titles or file names. The sync server checks each report against that list, refuses
            anything else, keeps the rest for 90 days and stores no IP address and no account with it.
          </p>
          <p className="mt-4 text-[0.9375rem] text-fg-3">
            Every field is in{" "}
            <a href={`${REPOSITORY_URL}/blob/master/docs/CRASH-REPORTS.md`} className="ink-link">
              the crash reports document
            </a>
            .
          </p>
        </section>

        <section id="metrics" aria-labelledby="counts" className="lg:col-span-7 lg:col-start-6">
          <h2 id="counts" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
            Usage counts
          </h2>
          <p className="mt-4 text-fg-2">
            So we know how many people use Kanso and on which devices, the apps send a small anonymous count every hour
            while they are open, when you open them and when you close them, after showing you a notice the first time. It goes to the sync server, which checks every field against a fixed list,
            refuses anything else, adds the country the connection comes from and passes the rest on to PostHog (EU). It can't
            be turned off; debug and development builds send nothing.
          </p>
          <div className="mt-8 grid gap-8 sm:grid-cols-2">
            <div>
              <h3 className="text-[0.8125rem] font-semibold text-fg-3">It sends</h3>
              <ul className="mt-4 grid gap-2.5">
                {COUNTS_SEND.map((item) => (
                  <li key={item} className="flex gap-3 text-fg">
                    <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-fg-3" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h3 className="text-[0.8125rem] font-semibold text-fg-3">It never sends</h3>
              <ul className="mt-4 grid gap-2.5">
                {COUNTS_NEVER.map((item) => (
                  <li key={item} className="flex gap-3 text-fg">
                    <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-fg-3" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
          <p className="mt-8 text-fg-2">
            Devices signed in to the same account share one ID made from the account's encryption key, which the server never
            has, so they count as one person and nobody can tell which account it is. Otherwise each device has a random ID of
            its own.
          </p>
          <p className="mt-4 text-[0.9375rem] text-fg-3">
            <a href="/metrics" className="ink-link">
              What's sent, and what isn't
            </a>
            , and every field in{" "}
            <a href={`${REPOSITORY_URL}/blob/master/docs/METRICS.md`} className="ink-link">
              the usage counts document
            </a>
            .
          </p>
        </section>

        <section id="feedback" aria-labelledby="feedback-heading" className="lg:col-span-7 lg:col-start-6">
          <h2 id="feedback-heading" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
            Feedback
          </h2>
          <p className="mt-4 text-fg-2">
            You can send feedback from inside Kanso. It is sent only if you press Send, and the app shows you the exact report
            first. Every report goes to Kanso's sync server, even if your own notes sync somewhere else.
          </p>
          <div className="mt-8 grid gap-8 sm:grid-cols-2">
            <div>
              <h3 className="text-[0.8125rem] font-semibold text-fg-3">It sends</h3>
              <ul className="mt-4 grid gap-2.5">
                {FEEDBACK_SEND.map((item) => (
                  <li key={item} className="flex gap-3 text-fg">
                    <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-fg-3" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h3 className="text-[0.8125rem] font-semibold text-fg-3">It never sends</h3>
              <ul className="mt-4 grid gap-2.5">
                {FEEDBACK_NEVER.map((item) => (
                  <li key={item} className="flex gap-3 text-fg">
                    <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-fg-3" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">Your report stays private</h3>
          <p className="mt-4 text-fg-2">
            The text and the screenshot are stored privately on Cloudflare, where only the developer can read them. They are never
            put on GitHub. A screenshot can show your notes, so check it before you send it.
          </p>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">How it is sorted</h3>
          <p className="mt-4 text-fg-2">
            To sort reports, an AI model (Claude Haiku, from Anthropic) reads each one through Cloudflare AI Gateway. The gateway
            doesn't keep the report's text in its logs. The model can only answer with a short, fixed form: it can't act on
            anything, and the server checks every field of the answer.
          </p>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">What becomes public</h3>
          <p className="mt-4 text-fg-2">
            Unless the report is noise or a repeat, the server files a short summary as an issue on Kanso's GitHub, written by the
            model and never your own words or your screenshot. If you chose to be credited, the issue and the release notes say
            "Reported by" and your name as plain text, never as a GitHub account.
          </p>
          <h3 className="mt-12 text-[0.8125rem] font-semibold text-fg-3">How long it is kept, and deleting it</h3>
          <p className="mt-4 text-fg-2">
            The text, app info, name and screenshot are deleted 180 days after the report's last change. What remains is its
            status, so you can still see that it was fixed. You can delete a report yourself at any time from My feedback in the
            app, and deleting your account deletes your reports and takes your name off the issues. The summary on GitHub stays,
            because it holds none of your words. A report sent without an account can be deleted from the device that sent it; if
            you lost that device, email the developer with the report's ID.
          </p>
          <p className="mt-4 text-[0.9375rem] text-fg-3">
            Cloudflare stores and processes reports, Anthropic reads them for sorting, and GitHub holds the public summaries. Every
            detail is in{" "}
            <a href={`${REPOSITORY_URL}/blob/master/docs/FEEDBACK.md`} className="ink-link">
              the feedback document
            </a>
            .
          </p>
        </section>
      </div>
    </article>
  );
}
