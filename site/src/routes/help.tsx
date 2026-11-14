/**
 * Help, in plain words: getting started, sync and adding a device, locked
 * notes, the library key, typed dates and repeating tasks, the Video studio
 * (board, Schedule, templates), API keys and pairing for agents and
 * Raycast, sharing notes and notebooks, importing (Apple Notes too), crash
 * reports, deleting an account and where to ask. It summarises README.md,
 * docs/ACCOUNTS.md, docs/LOCAL-API.md and docs/CRASH-REPORTS.md and must
 * change when they do. The mobile apps' Settings › About opens it.
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
      { name: "description", content: "Getting started with Kanso, syncing and adding a device, locked notes, your library key, dates and repeating tasks, running a video studio, API keys and Raycast, sharing notes and notebooks, importing, crash reports and support." },
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
        <p>
          Kanso and its sync are free, with no subscription. Settings › Account › <strong>What Kanso costs</strong> shows how much
          of your sync space you use, and <Link to="/price" className="ink-link">the price page</Link> gives the limits.
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
    id: "tasks",
    title: "Dates and repeating tasks",
    body: (
      <>
        <p>
          A task is a line that starts with <code>- [ ]</code>. Type when it's due at the end of the line, in plain words:{" "}
          <code>- [ ] film Pixel review fri 3pm every week</code>. When you pause, a quiet hint at the end of the line shows how
          Kanso reads it, such as "Fri 14 Nov, 15:00 · every week". Press <strong>Tab</strong> on a computer, or tap the hint
          above the keyboard (it ends in <strong>Set</strong>) on a phone, and the words become Kanso's own:{" "}
          <code>- [ ] film Pixel review @2026-11-14T15:00 every week</code>. Press Escape, or keep typing, and the text stays as
          you wrote it. Nothing is rewritten without that one key, so "fri" in a sentence stays a word.
        </p>
        <p>
          Typing <code>@</code> on a task offers Today, Tomorrow and the days of the week. After a date, a space offers{" "}
          <strong>Every day</strong>, <strong>Every week</strong>, <strong>Every weekday</strong> and{" "}
          <strong>Every month</strong>.
        </p>
        <p>Kanso reads English, and leaves anything else as text, without guessing:</p>
        <ul className="grid gap-1 pl-5 list-disc">
          <li>
            Days: today, tonight (20:00), tomorrow or tmr, mon to sun or monday to sunday, this fri, next fri, in 3 days, in 2
            weeks, 14 nov, nov 14, 14/11, 2026-11-14. A weekday alone means the next one after today, so "fri" typed on a Friday
            is a week later.
          </li>
          <li>
            Times: 3pm, 3:30pm, 15:00, 1500 h, noon, midnight, morning (09:00), afternoon (14:00), evening (18:00). A time
            alone is today, or tomorrow once it has passed.
          </li>
          <li>
            Repeats, after a date: every day or daily, every weekday, every week or weekly, every 2 weeks, every mon and thu,
            every month or monthly, every month on the 15th, on the last day or on the 2nd tue, every year or yearly. Add{" "}
            <strong>after done</strong> to count from the day you tick it rather than the day it was due.
          </li>
        </ul>
        <p>
          A repeating task's date ends in ↻, and its tooltip, or a long press on a phone, says how often it repeats. When you
          tick it, Kanso keeps the ticked line, without its repeat, and writes the next one just above it:
        </p>
        <pre className="overflow-x-auto rounded-md border border-line bg-code p-3 font-mono text-[0.8125rem] text-code-fg">
          <code>{"- [ ] film Pixel review @2026-11-21T15:00 every week\n- [x] film Pixel review @2026-11-14T15:00"}</code>
        </pre>
        <p>
          So the note keeps a record of each time. One undo takes both changes back, and unticking the old line leaves the new
          one. To stop a task repeating, open its date's menu (right-click the date on a computer, tap it on a phone) and choose{" "}
          <strong>Stop Repeating</strong>. Kanso removes the <code>every …</code> words and keeps the date. Deleting those
          words yourself does the same. As before, only a task with a time reminds you;
          a repeating one reminds you each time, and ticking it from the reminder writes the next one too.
        </p>
        <p>
          Quick Capture (<strong>⌃⌥Space</strong> on a computer, the capture sheet on a phone) has a <strong>Note | Task</strong>{" "}
          switch. In Task mode the whole text is one task, with the same hint under the field. Tasks go to today's daily note, or
          to the Inbox if you choose that under <strong>Captured tasks go to</strong> (Settings › General › Quick capture on a
          computer, Settings on a phone). Inbox tasks collect in one note in the Inbox, <strong>Inbox tasks</strong>, whether
          they come from Quick Capture, Raycast or an agent. Tasks an older version of Raycast or an agent put in a note called
          "Tasks" in the Inbox are moved into it the next time a task goes to the Inbox, and the old note goes to the trash.
        </p>
      </>
    ),
  },
  {
    id: "phone-shortcuts",
    title: "Capturing from anywhere on a phone",
    body: (
      <>
        <p>
          On an iPhone or iPad, Kanso adds four actions to Spotlight, the Shortcuts app, Siri and the Action button, alongside New
          Note, Open Today's Note, Add to Kanso and Capture Voice Note:
        </p>
        <ul className="grid gap-1 pl-5 list-disc">
          <li>
            <strong>Capture in Kanso</strong> saves text as a note in your Inbox; its first line becomes the title. Say "Quick
            capture in Kanso" to Siri.
          </li>
          <li>
            <strong>Add Kanso task</strong> adds a task, reading a date and repeat at the end the way the app does, and says what
            it added: "Added: film Pixel review · Fri 14 Nov 15:00 · every week". It goes where{" "}
            <strong>Captured tasks go to</strong> says: today's daily note or the Inbox. If the library can't be opened just then,
            the task waits and Kanso adds it the next time it opens.
          </li>
          <li>
            <strong>Search Kanso</strong> opens the app's search on the words you give it.
          </li>
          <li>
            <strong>Open Kanso note</strong> opens a note you pick from your notes. Locked notes, and notes in a locked notebook,
            are never offered.
          </li>
        </ul>
        <p>
          Each one asks you to unlock the phone first, and Kanso's own lock still applies when the app opens.
        </p>
        <p>
          On Android, touch and hold Kanso's icon for <strong>Capture</strong>, <strong>Add task</strong>,{" "}
          <strong>Search</strong>, <strong>Voice note</strong> and <strong>Today</strong>; drag one to the home screen to keep it
          there. <strong>Add task</strong> opens the capture sheet in Task mode over whatever you had open in Kanso. There is also
          an <strong>Add task</strong> tile for Quick Settings: edit your tiles and drag it in. Tapping it on the lock screen asks
          you to unlock first.
        </p>
      </>
    ),
  },
  {
    id: "studio",
    title: "Running a video studio",
    body: (
      <>
        <p>
          File › New Notebook (on a phone, Library › + › Notebook) asks what to start from: <strong>Blank notebook</strong> or{" "}
          <strong>Video studio</strong>. Video studio sets up a board (Idea → Scripting → Shooting → Editing → Published), a
          Schedule, and two templates, <strong>Video brief</strong> and <strong>Shot list</strong>. Its notes get the properties
          a pipeline needs: <code>stage</code>, <code>embargo</code> (a date and time with its time zone), <code>publish</code>,{" "}
          <code>sponsor</code> and <code>owner</code>. You can rename or change any of it later.
        </p>
        <p>
          <strong>The board.</strong> View › Board on a computer, or Board in the notes list's options on a phone, shows the
          notebook's notes as cards in a lane for each stage. Drag a card to another lane to change its stage; on a computer, ⇧←
          and ⇧→ move the selected card and ⌘Z undoes a move. On a phone, touch and hold a card and choose the stage. A card
          shows how many of the note's tasks are ticked, so a shot list reads as progress. <strong>Board settings</strong>{" "}
          chooses what the board is grouped by (Status, or any select property), and renames, reorders, adds and removes lanes.{" "}
          <strong>Show cards without a stage</strong> adds a lane for notes with none. Removing a lane asks what to do with its
          notes.
        </p>
        <p>
          <strong>Properties.</strong> In the note's inspector, or its properties on a phone, a select property such as{" "}
          <code>stage</code> is a list to choose from, a date opens a calendar, a date and time adds a clock and a time zone, and
          a person property lists people.
        </p>
        <p>
          <strong>Embargo.</strong> Give a note an <code>embargo</code> date, and until then a banner under its title says
          "Embargoed until Wed 18 Nov, 09:00 New York." Publishing it, making a share link to it or exporting it first asks, for
          example "Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). Publish anyway?", and goes ahead only if you choose{" "}
          <strong>Publish anyway</strong>, <strong>Make link anyway</strong> or <strong>Export anyway</strong>. When the embargo
          lifts, Kanso reminds you: "Embargo lifted: Pixel 11 review". An embargo with a time zone lifts at the same moment
          everywhere; one without lifts at that time wherever you are.
        </p>
        <p>
          <strong>The Schedule.</strong> View › Schedule, or Schedule in the sidebar, on a computer; Schedule in the library on a
          phone. It shows a month (or a week, on a computer, and an agenda, on a phone) with your dated tasks and the notes that
          have a date property, each on its day, and repeating tasks on every day they come round. Click or tap a task to tick
          it. Drag a task or a note to another day to change its date, or use <strong>Move to day…</strong> on a phone. Filter
          by notebook and by date property, and turn on <strong>Show Completed</strong> to see what's done.
        </p>
        <p>
          <strong>Templates.</strong> New from template lists the notebook's own templates first, under{" "}
          <strong>In this notebook</strong>. Video brief has a stage, the dates, an angle, key points, a shot list and review
          tasks; Shot list has a table of shots and a task for each.
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
          choose <strong>New key…</strong>. Pick what it's for, what it may do (read, suggest edits, read and write, or add and tick only) and which
          notebooks it sees. Kanso shows the key once, inside the setup to paste into the agent. <strong>Revoke</strong> stops a key
          at once. Locked notes are never shown to a key.
        </p>
        <p>
          An app on your computer can ask for a key itself. The Kanso extension for Raycast does: the first time you run one of its
          commands, Kanso comes to the front with <strong>Raycast wants to connect</strong> and a six-digit code. Choose{" "}
          <strong>Allow</strong> only if Raycast shows the same code and you just started it. The key it gets can only search and
          read notes, add notes and tasks, and tick tasks; it can't change or delete existing text. Paired apps have their own
          group in Settings › Local API, with when each was added and last used. <strong>Revoke</strong> disconnects one at once,
          and running a Kanso command in Raycast connects it again.
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
        <p>
          To write a note together with other people, share it, or a whole notebook, with their Kanso accounts. See{" "}
          <a href="#shared-notebooks" className="ink-link">Sharing a notebook</a>.
        </p>
      </>
    ),
  },
  {
    id: "shared-notebooks",
    title: "Sharing a notebook",
    body: (
      <>
        <p>
          A shared notebook is one that several people read and write together: every note in it, its board, its schedule and its
          templates. It is end-to-end encrypted. Your sync server passes it on without being able to read it, and each person's
          devices hold the notebook's key. Sharing needs a Kanso account, and so does everyone you share with.
        </p>
        <p>
          On a computer, right-click the notebook in the sidebar and choose <strong>Share Notebook…</strong>; on a phone, open the
          notebook's actions and tap <strong>Share notebook…</strong>. Type the person's Kanso username, pick their role and choose{" "}
          <strong>Invite</strong>. The first time you share with someone, Kanso shows their fingerprint. Compare it with the one
          they see in Settings › Account, in person or on a call, and choose <strong>Mark as verified and invite</strong> if it
          matches. If someone's keys change later, Kanso asks again before it trusts them (<strong>Trust the new keys</strong>).
        </p>
        <p>
          Sharing a notebook for the first time sends every note up, and the sheet counts them ("Sharing Studio… 120 of 340").
          Locked notes can't be in a shared notebook: they move to a private notebook of the same name. Notes you already share
          on their own, and canvases, stay out too, and the sheet lists them before you share.
        </p>
        <p>The roles are:</p>
        <ul className="grid gap-1 pl-5 list-disc">
          <li>
            <strong>Owner</strong>: the person who shared it. They can do everything, and only they can stop sharing it.
          </li>
          <li>
            <strong>Manager</strong>: edits, and invites, removes and changes the roles of people below Manager.
          </li>
          <li>
            <strong>Can edit</strong>: writes, moves cards on the board and changes properties.
          </li>
          <li>
            <strong>Can comment</strong>: reads and comments.
          </li>
          <li>
            <strong>Can view</strong>: reads only.
          </li>
        </ul>
        <p>A notebook can have up to 25 people, its owner included.</p>
        <p>
          An invitation shows under <strong>Shared</strong> in the sidebar, or in the phone's library, as "Sam invited you to a
          notebook · Can edit". <strong>Accept</strong> joins it and <strong>Decline</strong> says no. Once you've joined on one
          device, your other devices show it with <strong>Open here</strong>.
        </p>
        <p>
          Inside a shared notebook you see who else is in a note as they type. Each note says who changed it last ("Edited by
          Sam · 5 min ago"), and the notebook shows whether it is <strong>Up to date</strong>, <strong>Offline</strong>, or has
          changes waiting to go. You can keep working offline: when you're back, everyone's edits are merged. Two people
          changing different properties of a note, such as its stage and its publish date, both keep their change. A note
          someone else deletes goes to your trash, where you can restore it.
        </p>
        <p>
          <strong>Remove from notebook…</strong> takes someone out, and <strong>Leave</strong> takes yourself out. The owner's{" "}
          <strong>Stop sharing</strong> ends it for everyone. In each case the notebook gets a new key, so a person who has left
          can't read what is written after. They keep a private copy of the notes as they were, and so does everyone when sharing
          stops. To come back, someone in the notebook invites them again.
        </p>
        <p>
          Everyone in a shared notebook needs an up-to-date Kanso. When an update changes how shared notes are kept, Kanso shows
          a notice, "Update Kanso on every device for shared notes", and an older app may show shared notes without their
          properties until it is updated. Update Kanso on every computer and phone you use, and ask the people you share with to
          do the same.
        </p>
        <p>
          A Kanso that is too old for shared notebooks is told so by the sync server. Your notes keep syncing as before; only
          shared notebooks, and invitations to them, wait. The Shared section says "Update Kanso to open this shared notebook."
          where an invitation would be, and a notice, <strong>Update Kanso to use shared notebooks</strong>, lists the steps:
          install the latest version (Settings › Updates on a computer, the App Store or Google Play on a phone), restart Kanso
          and open the Shared section again, then update your other devices too.
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
