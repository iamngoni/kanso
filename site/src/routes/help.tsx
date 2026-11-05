/**
 * Help, in plain words: getting started, sync and adding a device, locked
 * notes, the library key, API keys for agents, sharing, importing (Apple Notes too), crash reports, deleting an account and where to
 * ask. It summarises README.md, docs/ACCOUNTS.md and docs/CRASH-REPORTS.md
 * and must change when they do. The mobile apps' Settings › About opens it.
 */
import { Link, createFileRoute } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { REPOSITORY_URL } from "../lib/release";
import { ANDROID_URL, APP_STORE_URL } from "../lib/stores";

export const ISSUES_URL = `${REPOSITORY_URL}/issues`;

export const Route = createFileRoute("/help")({
  head: () => ({
    meta: [
      { title: "Help — Kanso" },
      { name: "description", content: "Getting started with Kanso, syncing and adding a device, locked notes, your library key, API keys for agents, sharing, importing, crash reports and support." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/help" }],
  }),
  component: Help,
});

interface Topic {
  readonly id: string;
  readonly title: string;
  readonly body: ReactNode;
}

export const TOPICS: readonly Topic[] = [
  {
    id: "start",
    title: "Getting started",
    body: (
      <>
        <p>
          On a computer, <Link to="/download" className="ink-link">download Kanso</Link> for macOS, Windows or Linux and open it. Your
          notes live on that computer, in plain Markdown, and Kanso works without an account or a connection.
        </p>
        <p>
          On an iPhone or iPad, get{" "}
          <a href={APP_STORE_URL} className="ink-link">Kanso Pocket from the App Store</a>
          {ANDROID_URL ? (
            <>
              ; on Android, <a href={ANDROID_URL} className="ink-link">get it from Google Play</a>
            </>
          ) : (
            "; the Android version is coming soon to Google Play"
          )}
          . It is the same app: write, search and lock notes there as you do on
          your computer, and sign in with your Kanso account (see below) to keep them in step. The mobile apps share version
          numbers with the computer apps, so{" "}
          <Link to="/releases" className="ink-link">the release notes</Link> cover them all.
        </p>
        <p>You don't need an account to use Kanso on one computer. You need one to keep several devices in step.</p>
      </>
    ),
  },
  {
    id: "sync",
    title: "Syncing, and adding a device",
    body: (
      <>
        <p>
          On a computer, open Settings › Account and choose <strong>Create account</strong>. Pick a username and a password of at
          least 12 characters. Kanso encrypts your notes on the device before they are uploaded, with a key the server never sees.
        </p>
        <p>
          Kanso then shows a <strong>recovery key</strong>. Save it somewhere safe, such as a password manager. Kanso can't reset a
          forgotten password: the recovery key is how you set a new one.
        </p>
        <p>
          To add a device, sign in to the same account on it. Your other devices hold the key to your notes, so the new one asks
          one of them to approve it. It shows a six-digit code; open Kanso on a device that is already signed in and approve the
          request only if it shows the same code. If no other device is at hand, use your recovery key instead.
        </p>
        <p>
          Signing out keeps every note on the device. If a device already has notes when you sign in, Kanso asks whether to add
          them to the account or replace them with the account's notes, and saves a backup first.
        </p>
      </>
    ),
  },
  {
    id: "locked",
    title: "Locked notes",
    body: (
      <>
        <p>
          <strong>Lock note…</strong> in the command palette or the Note menu seals a note's text with a lock password. You can lock
          a whole notebook too. A locked note's text stays sealed in sync and opens on your other devices with the same password.
        </p>
        <p>
          <strong>Kanso can't recover a forgotten lock password</strong>, and the notes locked with it can't be read without it.
          Titles, notebooks, tags and dates of locked notes stay visible, and pictures and files attached to them aren't sealed.
        </p>
      </>
    ),
  },
  {
    id: "library-key",
    title: "Your library key",
    body: (
      <>
        <p>
          On a computer, Kanso encrypts your library and its backups on the disk, with a key your system keeps for you. You don't
          type a password to open Kanso. A backup opens on another computer only with the <strong>library key</strong>, which
          Kanso shows as 24 words.
        </p>
        <p>
          If you don't sync, Kanso asks you to save the key right after it encrypts your library, and asks again at each launch
          until you choose <strong>I've saved it</strong>. Write the words down and keep them away from the computer. To see them
          again, open Settings › General › Data › Backups and choose <strong>Show key…</strong>. To open a backup from another
          computer, choose <strong>Restore…</strong> there and type its 24 words. Signed in to sync, you don't need them: signing in
          on a new computer brings your notes back.
        </p>
        <p>
          If Kanso can't encrypt your library, it tells you why and how to fix it, and tries again each time it opens. Nothing is
          lost.
        </p>
      </>
    ),
  },
  {
    id: "api-keys",
    title: "Connecting an agent or a script",
    body: (
      <>
        <p>
          Every AI agent, script or app that reaches Kanso's local API needs its own key. Open Settings › Local API › API keys and
          choose <strong>New key…</strong>. Pick what it's for, what it may do (read, suggest edits, or read and write) and which
          notebooks it sees. Kanso shows the key once, inside the setup to paste into the agent. <strong>Revoke</strong> stops a key
          at once. Locked notes are never shown to a key.
        </p>
        <p>
          The local API's old username and password are gone. If you had set them up, a banner explains it and{" "}
          <strong>How to fix</strong> lists the steps: make a key for each agent or script, paste the new setup in place of the old
          one, and restart the agent. A script sends <code>Authorization: Bearer</code> and the key. The details are in{" "}
          <a href={`${REPOSITORY_URL}/blob/master/docs/LOCAL-API.md`} className="ink-link">the local API guide</a>.
        </p>
      </>
    ),
  },
  {
    id: "sharing",
    title: "Sharing",
    body: (
      <>
        <p>
          You can export a note as a web page or a PDF, or publish it as an encrypted link. The key to a published note is in the
          link itself, so only people you give the link to can read it. Locked notes can't be published.
        </p>
      </>
    ),
  },
  {
    id: "importing",
    title: "Importing your notes",
    body: (
      <>
        <p>
          On a computer, <strong>File › Import…</strong> reads a folder or an export from another notes app and shows what it found
          before it writes anything. The source is only ever read, and importing the same source again updates the notes it made
          instead of duplicating them. It reads Markdown folders and many other apps' exports; the list is in{" "}
          <a href={`${REPOSITORY_URL}#importing`} className="ink-link">the README</a>.
        </p>
      </>
    ),
  },
  {
    id: "apple-notes",
    title: "Bringing Apple Notes across",
    body: (
      <>
        <p>
          On a Mac, <strong>File › Import… › Apple Notes</strong> reads the Notes library itself: folders, checklists, tables,
          scans, drawings, attachments and links between notes. It is the best way across, and sync then brings everything to
          your iPhone and iPad.
        </p>
        <p>
          An iPhone or iPad can't read the Notes library, so there are two more ways. In Notes, <strong>Share › Kanso</strong>{" "}
          saves a note to the Inbox as a Markdown note, its checklist as tasks and its pictures attached. Or export notes as files
          (PDF, RTF, HTML, text or Markdown) and choose the folder, a <code>.zip</code> or a single file in Settings › Import ›
          Apple Notes. They go under an <strong>Apple Notes</strong> notebook, the export's folders inside it, and importing the
          same files again updates those notes instead of duplicating them.
        </p>
        <p>
          Locked Apple notes can't be read by any of these ways: unlock them in Notes first. If you have locked the Apple Notes
          notebook in Kanso, an import leaves its notes as they are. Windows and Linux have no Apple Notes: import on a Mac and
          sync.
        </p>
      </>
    ),
  },
  {
    id: "crashes",
    title: "Crash reports",
    body: (
      <>
        <p>
          If Kanso on your iPhone, iPad or Android device crashes, it keeps a report on the device. It is sent only when you tap{" "}
          <strong>Send</strong>, and the app shows you the exact text first.
        </p>
        <p>
          A report says where in Kanso's code the failure happened, the app and system versions, the kind of device and the day. It
          never holds note text, titles or file names. The{" "}
          <Link to="/privacy" hash="crash-reports" className="ink-link">privacy page</Link> has the details, and{" "}
          <a href={`${REPOSITORY_URL}/blob/master/docs/CRASH-REPORTS.md`} className="ink-link">the definition</a> lists every field.
        </p>
      </>
    ),
  },
  {
    id: "delete",
    title: "Deleting your account",
    body: (
      <>
        <p>
          On a computer, Settings › Account has <strong>Delete account…</strong>. It asks you to prove it's you, and then removes
          the account and everything the sync server held for it. The notes on your devices stay. Backups kept by the server's
          database host can hold the encrypted data for up to 30 days.
        </p>
      </>
    ),
  },
  {
    id: "support",
    title: "Getting support",
    body: (
      <>
        <p>
          Kanso is open source and supported in public. Search{" "}
          <a href={ISSUES_URL} className="ink-link">the issues on GitHub</a> to see whether someone has met the same problem, or{" "}
          <a href={`${ISSUES_URL}/new`} className="ink-link">open a new one</a>. Say which device and Kanso version you use, and what
          you did. Please leave note text out of it.
        </p>
      </>
    ),
  },
];

export function Help() {
  return (
    <article className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="text-[0.875rem] font-medium text-fg-3">Help</p>
      <h1 className="mt-3 max-w-3xl text-[clamp(2.2rem,1.6rem+2.6vw,3.6rem)] leading-[1.05] font-semibold tracking-[-0.035em]">How can we help?</h1>
      <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
        The short answers to what people ask most, for Kanso on your computer, iPhone, iPad and Android.
      </p>

      <nav aria-label="Topics" className="mt-10 flex max-w-3xl flex-wrap gap-x-6 gap-y-2 text-[0.9375rem]">
        {TOPICS.map((t) => (
          <a key={t.id} href={`#${t.id}`} className="ink-link">
            {t.title}
          </a>
        ))}
      </nav>

      <div className="mt-[clamp(3rem,7vw,5rem)] grid max-w-3xl gap-14">
        {TOPICS.map((t) => (
          <section key={t.id} id={t.id} aria-labelledby={`${t.id}-title`} className="scroll-mt-20 border-t border-line pt-6">
            <h2 id={`${t.id}-title`} className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
              {t.title}
            </h2>
            <div className="mt-4 grid gap-4 text-fg-2">{t.body}</div>
          </section>
        ))}
      </div>
    </article>
  );
}
