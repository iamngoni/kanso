# Kanso

A local-first Markdown notes app for macOS, with Windows and Linux builds, built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit).

[kansoapp.app](https://kansoapp.app) has downloads and a tour; the website's source is in [`site/`](site).

**Writing**
- Brackets, quotes and Markdown marks pair as you type; lists and tasks continue on Return and
  indent with Tab; tables line up as you edit them.
- Smart paste turns a pasted link into Markdown and keeps code as code; ⇧⌘V pastes plain text.
- Typewriter scrolling, a word goal per note, focus mode, and optional Vim keys.
- Voice input: press the mic, speak, and the words go in at the cursor as you say them. It runs on your device
  with NVIDIA's Parakeet model (25 languages, downloaded once on first use), or with your
  system's own dictation.

**Linking**
- `[[Wiki-links]]` with autocomplete, backlinks with their context, unlinked mentions, and an
  outline of the note's headings beside the editor.
- ⌘P finds notes by title and by heading.

**Capture**
- Today's note (⌘D) in a Journal notebook, and templates with `{{date}}`, `{{time}}` and `{{title}}`.
- Quick capture from anywhere with ⌃⌥Space, or `kanso capture` from a terminal, into an Inbox.
- Nested tags (`work/clients`) shown as a tree, with `#` completion in the editor.

**Preview**
- Live preview (⌘4): headings drawn larger and the Markdown marks hidden, except on the line
  you're on.
- Mermaid diagrams, math, and Excalidraw drawings, drawn in the note's theme.
- Images resize from the preview or with `![alt|300](image.png)`.
- **Present** a note full screen as slides (Note › Present, or the palette). A `---` line after a blank
  line starts a slide, or each `#` and `##` heading does if the note has none. Arrow keys, Space,
  Page Down and Page Up move through a long slide and on to the next, F leaves full screen and Esc
  ends it. Phones and tablets have Present in the note's More menu: swipe or tap a side to move.

**Tasks**
- Due dates (`- [ ] Send the draft @2026-10-25`), a Tasks view of what's overdue, due today and
  coming up, and reminders as notifications.

**Finding and changing**
- Search with filters, find and replace across every note with one undo, and each note's
  version history with what changed since each version.

**Sharing**
- Export a note as a web page or PDF, or publish it as an end-to-end encrypted link: the key
  stays in the link, so the server only holds ciphertext.

**Locked notes**
- Lock a note with a password (**Lock note…** in the palette or the Note menu). Its text is sealed
  on this device, stays sealed in sync, and opens on your other devices with the same password.
- Lock a whole notebook, and every note in it and in the notebooks inside it is locked, now and
  when notes go in later.
- Unlock them with Touch ID on a Mac that has it, or Windows Hello on Windows, with the password as
  the fallback.

**Sync, agents and the rest**
- **End-to-end encrypted sync.** An account on a Cloudflare Worker keeps your devices in step.
  Each device encrypts notes, titles, IDs and attachments before they leave it, with a key the
  server never sees. Conflicts keep both versions: the losing one stays in the note's history.
- **A local API for agents.** `http://127.0.0.1:19840` is a REST API over your notes, and `/mcp`
  on the same port is a native MCP endpoint. Every agent, script and app needs its own **API key**,
  made in Settings › Local API › **API keys**, with the access you choose (read, suggest edits, or
  read and write) and the notebooks it may see. **New key…** gives the setup for Claude Code,
  Claude Desktop, Codex, Cursor, VS Code, Gemini CLI, Windsurf, Zed or OpenCode, or for a script.
  Its REST API is compatible with popular local-server clients. [The local API](#the-local-api)
  has the details.
- **An encrypted library.** Your library and its backups are encrypted on this computer, with a key
  the system keeps. [Encryption on this computer](#encryption-on-this-computer) says what that
  does and doesn't stop, and how to save the key.
- **Ask your notes, with a key or an agent.** Off until you turn it on in Settings › AI. Answers cite
  the notes they used, and an inline assistant and note summaries work the same way. They come
  from a provider you have a key for, or from an agent on this computer (Claude Agent, Codex, Gemini
  CLI, OpenCode or any ACP agent) on your own subscription. Either way the notes it is sent leave your
  computer; [AI](#ai) says what is sent and what Kanso refuses.
- **History and backups.** Earlier versions of each note, including every version replaced by the
  API, sync or an import, and a daily database backup.
- **Imports.** Bring in a folder of Markdown files, an Obsidian vault or a Logseq graph, or an
  export from Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote or UpNote, or a PouchDB
  notes library found on this computer, with their notebooks, tags, links and attachments. The source is only read, you see what will come
  in before anything is written, and importing again picks up newer edits without reverting notes
  you changed in Kanso.
- Notebooks nest, and fold away in the sidebar like an accordion (the arrow beside one opens it);
  right-click one › **Change Icon…** to search 144 icons and give it one of its own, or › **Change Color…**
  to wash its row in a color (a swatch, the picker, or hex like `#f8f8f8`) so it's easy to find; both sync with it.
- Themes, light and dark, icon themes that draw every icon, and updates that install themselves.
- **An anonymous daily count.** Once a day Kanso sends which kind of device, its version and which
  features were used, never anything you wrote; [METRICS.md](docs/METRICS.md) lists every value.

## Download

Builds for macOS (Apple silicon), Windows (x64) and Linux (x64 and arm64) are on the
[Releases page](https://github.com/iamngoni/kanso/releases). `SHA256SUMS` lists each download's
checksum (`shasum -a 256 -c SHA256SUMS --ignore-missing` checks the ones you have).

**macOS** (`Kanso-<version>-macos-arm64.dmg`): open the disk image and drag Kanso to Applications.
It is signed with a Developer ID and notarized by Apple, so it opens like any other Mac app.
Once installed, Kanso updates itself: it checks for new releases and installs them when you choose
Restart now.

**Windows** (`Kanso-<version>-windows-x64-setup.exe`): run the installer. It installs for you only,
into `%LOCALAPPDATA%\Programs\Kanso`, without asking for an administrator, and adds Kanso to Start
(and, if you tick it, the desktop); uninstall it from Settings › Apps. It isn't code-signed yet, so
SmartScreen may say "Windows protected your PC": choose More info › **Run anyway**. Prefer no
installer? `Kanso-<version>-windows-x64.zip` holds the same `kanso.exe` to put anywhere you can write
to. Either way Kanso updates itself. Notes live in `%APPDATA%\Kanso` and stay when Kanso is
uninstalled unless you ask for them to go; sign-ins go to the Windows Credential Manager.

**Linux** (`Kanso-<version>-linux-x64.tar.gz` or `-linux-arm64.tar.gz`, built on Ubuntu 22.04 so
glibc 2.35 or newer): unpack it and run `./install.sh`, which puts `kanso` in `~/.local/bin` and Kanso
in your applications menu (`./install.sh --uninstall` removes them). Kanso runs on Wayland or X11 and
needs Vulkan graphics drivers (Mesa's are fine). Notes live in `~/.local/share/kanso`
(`$XDG_DATA_HOME/kanso`); sign-ins go to the Secret Service (GNOME Keyring or KWallet), so one must be
running to stay signed in.

**Linux with apt** (Debian 12+, Ubuntu 22.04+, x64 and arm64): add Kanso's repository once, and
`sudo apt upgrade` updates Kanso with the rest of your system.

```bash
curl -fsSL https://kansoapp.app/apt/kanso.gpg | sudo tee /usr/share/keyrings/kanso.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/kanso.gpg] https://kansoapp.app/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/kanso.list
sudo apt update && sudo apt install kanso
```

It installs `kanso` and `kanso-plugin-host` in `/usr/bin` and Kanso in your applications menu. It needs
glibc 2.35 or newer, Vulkan drivers and a Secret Service, as above. If you used `install.sh` before, run
`./install.sh --uninstall` first: `~/.local/bin` comes before `/usr/bin` in `PATH`. Kanso still tells
you when a release is out, and leaves installing it to apt. The repository holds the latest release
only ([docs/APT.md](docs/APT.md)).

To build the macOS app yourself:

```bash
scripts/bundle-macos.sh
```

It builds an ad-hoc signed `target/release/Kanso.app`; copy it to `/Applications`. On Windows and
Linux, `cargo build --release -p kanso-app` builds `target/release/kanso`. On first run with an
empty library, Kanso offers to import a PouchDB notes library if it finds one on this computer.

## Using it

### Importing

**File › Import…** (also in the Kanso menu and the command palette) asks where your notes are,
then shows what it found before writing anything: how many notes, notebooks, tags and attachments,
what is new or changed since a last import, and what it will skip and why. **Import** brings them
in with a progress bar and a report. The source is only ever read. Each import is keyed by where
a note came from (a vault-relative path, or the other app's own ID), so importing the same source
again updates its notes instead of duplicating them, and leaves alone any note you changed or
deleted in Kanso since. **File › Open…** brings in a single `.md` or `.txt` file (and the pictures
it links to) as a note in the Inbox and opens it; opening it again updates that note. Kanso also opens Markdown files from Finder's or Explorer's Open With, or with `kanso file.md`.

| Source | Pick | What comes in |
|---|---|---|
| Markdown folder | the folder | Every `.md`, `.markdown` and `.txt` file. Subfolders become nested notebooks. YAML front matter gives the title, tags, aliases and created/updated dates; otherwise the first `# heading`, then the file name, is the title and the file's dates are used. Other front-matter keys stay at the top of the note. Linked pictures and files become attachments; links between the files become links between notes. |
| Obsidian vault | the vault's folder | All of the above, with the file name as the title, `[[wiki-links]]` (headings and aliases kept), `![[embeds]]` of pictures and files (`\|300` widths kept; embedded notes become links), `#tags` and `#nested/tags` in the text and front matter, and the attachment folder from Settings › Files and links. Callouts stay as Markdown. `.obsidian/`, `.trash/` and `.canvas` files are skipped and listed. |
| Logseq graph | the graph's folder | Pages and journals, `title::`, `tags::` and `alias::` properties, `[[page refs]]`, and `a___b` namespaced pages as `a/b`. The `logseq/` folder is skipped. |
| Notion | the `.zip` from Settings › Export (Markdown & CSV), or its folder | Pages with the 32-character IDs taken off names and titles, sub-pages in a notebook named after their page, links between pages (including `notion.so` links to pages in the export) and files. A database becomes a note with its table, the first column linking its rows; its rows are notes with their properties. Large exports split into several zips work as they come. |
| Evernote | an `.enex` file, or a folder of them | Each `.enex` is a notebook. Notes become Markdown (headings, lists, checklists, tables, code blocks, links), with their tags, dates and attachments. |
| Bear | a `.bear2bk` backup, or an Export Notes folder (TextBundle or Markdown) | Notes with their pictures and files, `#tags`, `#nested/tags` and `#multi word tags#`, pinned, archived (in a notebook of their own) and trashed notes. |
| Joplin | a `.jex` file, or a RAW export folder | Notebooks and sub-notebooks, tags, attachments and links between notes. |
| Standard Notes | a decrypted backup (`.txt`/`.json`, or the `.zip`) | Notes, nested tags, pinned, archived and trashed notes. Encrypted backups can't be read; export a decrypted one. |
| Simplenote | the `.zip` from Export Notes, or its `notes.json` | Notes (the first line is the title), tags, pinned and trashed notes. |
| UpNote | a folder from Export › Markdown | Read as a Markdown folder. |
| Apple Notes | found on this computer | Accounts, folders, notes, tables, scans and drawings with a downloaded preview, pictures, PDFs and other attachments, internal links, tags and pinned notes. Locked notes and content not downloaded from iCloud are skipped and listed. Kanso finds the standard Notes library itself and macOS may ask once to allow read-only access. |
| Apple Notes export | a folder or `.zip` of notes exported as PDF, RTF, HTML, text or Markdown | Under an "Apple Notes" notebook, the export's folders as notebooks inside it. Checklists become tasks, a PDF becomes a note with the PDF attached, a note of only a picture says "(drawing)". What a phone can import. |
| Apple Journal | an entry `.html` file, the export's folder, or a `.zip` | Each entry as a dated note, with its reflection prompt, body text and useful activity/location/media metadata. `index.html` is skipped. |
| Google Keep | a Keep `.json` file, a Google Takeout `.zip`, or its folder | Notes, lists and checklists, labels as tags, pinned/archived/trashed state, annotations, dates and local attachments. |
| HTML | an `.html`/`.htm` file, a folder of pages, or a `.zip` | Pages as Markdown with their folder tree, links between pages, and local pictures/files as attachments. Remote pages are not fetched. |
| Roam | a Roam graph export in JSON | Pages and nested outlines, block and page references that resolve inside the graph, tags, checklists and tables. Remote attachments are left as links. |
| TextBundle/TextPack | a `.textbundle` folder, a `.textpack`, or a `.zip` of bundles | Markdown notes and their assets. |
| Tomboy/Gnote | a `.note` file or a Tomboy/Gnote notes folder | Titles, text, nested lists, internal links, supported formatting, tags and dates. |
| PouchDB notes library | found on this computer | Notes, notebooks, tags, statuses and attachments, keeping their IDs (below). |

Attachments larger than 10 MB, the most a Kanso attachment can be, are skipped and listed, and
links to them stay as they were. Apple Notes is read from its local database, so use the macOS app
to make sure the notes you need have downloaded before importing. `kanso import apple-notes` finds
the standard Notes library; `kanso import apple-notes <path>` can read a copy somewhere else.

**Apple Notes on iPhone and iPad.** Apple does not let apps read the Notes library on a phone or
tablet, so there are three ways across (Settings › Import › Apple Notes explains them): import on
your Mac and let sync bring everything (recommended), share a note from Notes to Kanso (it arrives
in the Inbox as a Markdown note, checklists as tasks), or choose notes exported as files with
**Apple Notes export**. Locked notes cannot be read by any route. Windows and Linux have no Apple
Notes; import on a Mac and sync.

From a terminal, with Kanso closed, `kanso import <kind> [path]` does the same (`kanso import
help` lists the kinds); Apple Notes finds its own path, and `--dry-run` prints the summary and
writes nothing. A terminal can't show Kanso's Notes access panel, so the terminal itself needs
Full Disk Access once, or give Apple Notes a readable copy instead.

**Importing a PouchDB notes library.** Choosing it in **Import…** (or `kanso import pouchdb`)
copies the library found on this computer in, keeping its IDs and revisions; its files are only
read. The local API listens on port 19840; if another app already uses that port, the plug icon at
the bottom of the sidebar turns amber, and Kanso keeps trying until it's free or you choose another
port in Settings › Local API. To keep clients you had set up against that app's local server,
**Copy local API settings…** on that page takes its port and address (`--credentials` does the same
for `kanso import pouchdb`). It takes no username or password: those clients need a Kanso API key,
made in Settings › Local API › API keys.

**Sync.** Settings › Account › **Create account** picks a username and a password of at least 12
characters; this device's notes are encrypted and uploaded. Kanso then shows a **recovery key**: save
it somewhere safe (a password manager, or printed). Kanso can't reset a forgotten password. The
recovery key sets a new one (**Forgot your password?**), and a new key replaces it afterwards. On
your other devices, **Sign in** with the same username and password. A device that already has notes asks
whether to add them to the account or replace them with the account's notes (after saving a backup
of its library). Signing out keeps every note on the device. The Account page also lists your devices,
signs one out, changes the password and deletes the account.

The default server is the owner's Worker, `sync.kansoapp.app`, which takes new
accounts only when its owner opens sign-ups. Published notes open on `share.kansoapp.app`. To run your own, deploy the Worker in
[cloud/](cloud/README.md) to your Cloudflare account and choose **Use a different server…** before
creating the account. An account belongs to its server's address, so pick the final URL first.

### Locked notes

**Lock note…** (palette, or the Note menu) seals the open note's text with your lock password. The
first time, Kanso asks you to set one, at least 8 characters; on another device, or once notes are
locked, it asks for that password instead. **Kanso can't recover a forgotten lock password**, and the
notes locked with it can't be read without it.

Once you type the password, the notes locked with it open until **Lock notes now**, quitting, or ten
minutes without opening or saving one. **Remove lock…** keeps a note's text as plain text again, and
**Change lock password…** seals every locked note again with a new password.

**Touch ID** (macOS): once you've typed the lock password, Kanso offers to unlock with Touch ID
instead; **Unlock locked notes with Touch ID** in the palette or the Note menu turns it on at any time,
and **Stop using Touch ID for locked notes** turns it off. With it on, Unlock asks for a fingerprint
and falls back to the password if you cancel. The lock password still exists and still works: it's
what other devices and Macs without Touch ID use. Touch ID unlocks the lock *keys*, which Kanso keeps
in this Mac's login Keychain (never the password itself) and reads only after a fingerprint; macOS
doesn't enforce the fingerprint on the Keychain item, so this guards against someone using your
unlocked Mac, not against software running as you. Changing the lock password replaces the kept key.

**Windows Hello** (Windows) works the same way, with the same commands named for it: **Unlock locked
notes with Windows Hello** and **Stop using Windows Hello for locked notes**. Windows Hello accepts
your face, a fingerprint or your Windows Hello PIN, never the Windows password, and Kanso's lock
password stays the fallback. The kept lock keys go into the Windows Credential Manager, which only
your signed-in Windows account can read; as on a Mac, that guards against someone using your
unlocked PC, not against software running as you. Linux has no such unlock: locked notes open with
the lock password there.

**Locked notebooks.** Right-click a notebook and choose **Lock Notebook…**: every note in it, and in
the notebooks inside it, is sealed with your lock password, and so is a note made there or moved in
later. Opening a locked notebook asks for the password (or Touch ID, or Windows Hello) once, and its
notes stay open until they lock again with the others. A locked notebook shows a lock in the sidebar.

- A note locked on its own before keeps its own seal, and opens with the password it was locked with.
- A note moved out of a locked notebook stays locked; **Remove lock…** makes it plain again. Inside a
  locked notebook, a note can't have its lock removed.
- **Remove Notebook Lock…** (after opening the notebook) stops locking the notes put there later. The
  notes in it now stay locked, so no text turns plain without you asking for that note.
- The notebook's lock syncs inside its own encrypted document, like a note's. A note another device
  puts there without sealing it (one running an older Kanso, say) is kept from search, the local API
  and related notes here, and is sealed the next time you open the notebook.
- The local API and MCP don't see a locked notebook or the notes in it: to an API key they are "not
  found", and can't be read, changed, moved or deleted.
- **Change lock password…** moves locked notebooks to the new password too.

What locking does and doesn't do:

- The text is sealed with XChaCha20-Poly1305 under a key Argon2id derives from the password (256 MiB,
  three passes), bound to the note, and kept in the note itself (`kansoLock`). The note's body in the
  database is empty, so search, backlinks, tasks, reminders, open loops, the morning brief, the weekly review, related
  notes and the local API (REST and MCP) never see it. Sync carries the sealed text inside the note's own encrypted document.
- Locking clears the note's `kansoLoops` field (which loops were closed or hidden) and the loops found
  in it, and the finished-loop history kept on this device.
- Past notes about the paragraph you are writing use a vector for each passage of a note and your
  "Not related" answers. Both stay on this device: they are not synced, they are emptied with the
  library (or when the embedding model changes), and they are in the daily `VACUUM INTO` backups.
- Locking deletes the note's earlier versions, its embedding and the search index's words, and SQLite
  overwrites the space they took. Daily backups made before you locked it still hold the text until
  they are replaced, after 14 days.
- The title, notebook, tags, status and dates stay visible, and pictures and files attached to a
  locked note aren't sealed.
- The local API doesn't see a locked note at all: to an API key it is "not found", so it can't read,
  retitle, tag, move or delete it.
- **Empty Trash** deletes locked notes too. Its question counts them ("Delete 12 notes permanently,
  including 2 locked notes and their sealed attachments"), and a locked note's sealed attachments are
  deleted with it. An attachment a note outside the trash still uses is kept.
- Export All Notes leaves locked notes, and the notes in locked notebooks, out, and none of them can
  be published.
- Every device needs this version of Kanso: an older one shows a locked note as empty, and doesn't
  know a notebook is locked, so a note it makes there stays plain until this version seals it.

### Encryption on this computer

Kanso encrypts your library (`kanso.db`, its write-ahead log and your daily backups) with SQLCipher
on macOS, Windows and Linux. The key is 256 random bits that the system keeps for you: the Keychain
on macOS, the Credential Manager on Windows, the Secret Service on Linux (a headless server may use
`secrets.json`, see [Headless / VPS](#headless--vps)). It is unlocked with your login session, so
Kanso doesn't ask for a password or Touch ID to open. The phones keep the stock database: each app's
sandbox, and iOS Data Protection or Android file encryption, already cover their files.

- **New libraries** are encrypted from the start.
- **A library from before** is encrypted once, at the first launch of a version that can: a window
  reads "Encrypting your library… 62%" and the library window opens when it is done. Kanso copies
  the library into an encrypted file, checks every table row by row, and only then swaps it in and
  overwrites the plain file. Your existing backups are encrypted too. If anything fails the plain
  library is left as it was, and a crash at any step leaves the library either plain or finished,
  never half of each.
- **If Kanso can't encrypt it** (the system keychain refuses the key, or the disk has less free
  space than the library takes), the library stays a plain file that any program running as you can
  read, nothing is lost, and Kanso tries again at every launch. An update notice, **Your library
  isn't encrypted yet**, lists the fixes: unlock the keychain (on Linux, run a Secret Service such as
  GNOME Keyring or KeePassXC), free some disk space, and restart Kanso. On a server with no keychain,
  see [Headless / VPS](#headless--vps).

**Save your library key.** A backup opens on another computer only with the library key, shown as
**24 words**. If the library isn't signed in to sync, Kanso shows **Save your library key** right
after it encrypts, with the words, **Copy** (the clipboard is cleared after a minute) and **I've
saved it**. **Later** closes it, and it comes back at each launch until you choose **I've saved
it**. A library signed in to sync gets no sheet: signing in on a new computer brings the notes back.
Until you save it, an update notice, **Your library is now encrypted**, says the same, and it goes
when you do.

In Settings › General › Data › **Backups**:

- **Library key** › **Show key…** confirms it's you (Touch ID on a Mac that has it, Windows Hello on
  Windows, otherwise a confirmation) and shows the 24 words again. The row says "Library key not
  saved" until you've chosen I've saved it.
- **Restore from backup** › **Restore…** takes a backup file. One this computer's key opens, or a
  plain one, goes straight to the question; a backup made on another computer asks for its 24
  words first. Kanso stages the backup, re-encrypted with this library's key, and asks you to quit
  and open it again. At that launch the app swaps the backup in and keeps the library you had as
  a backup; `kanso capture`, `serve`, `sticky`, `plugin`, `import`, `account` and `kanso-sync`
  leave a waiting restore alone and say "A restore is waiting; open Kanso to apply it." The API
  keys stay the library's own (see [The local API](#the-local-api)).
  A library signed in to sync can't be restored: sign out (Settings › Account) first, then
  restore. If you sign in again before reopening Kanso, the restore waits until you sign out. The
  restored library comes back signed out and not linked to any account, so when you sign in
  again Kanso asks, as it does on any device with notes, whether to add the restored notes to the
  account (a note deleted from the account since the backup comes back, and a note changed on
  both sides keeps both versions) or replace them with the account's notes (after saving a
  backup of them).

Both rows show only for an encrypted library. Without the key (and without sync) a backup can't be
opened: that is the cost of encryption.

What it does and doesn't do:

- It stops a copy of `kanso.db` or a backup from being read without the key, and a stolen disk. Keep
  FileVault, BitLocker or LUKS on as well: SSDs can keep old blocks of the plain file.
- **On macOS** the Keychain item trusts only the app that made it, so another program (the
  `security` command included) gets a system prompt. **On Windows and Linux**, any program running as
  you can read the key, so there encryption protects against a stolen disk or copied files, not
  against software running as you.
- Plugin settings and plugins' network logs are kept in the library, and `config.json` has no
  secrets, so no note text sits in the files beside it.
- It does not hide your notes from the local API: that is what API keys are for.

### The local API

`http://127.0.0.1:19840` is a REST API, and `/mcp` on the same port a native MCP endpoint. **Every
caller needs its own API key**, even a program that can start Kanso. Make one in Settings › Local API
› **API keys** › **New key…**: pick what it's for, a name, what it can do (**Read notes**, **Read
and suggest edits**, or **Read and write**) and which notebooks. Kanso shows the key once, inside the
setup to paste into the agent, and stores only a hash. **Revoke** stops a key at once; Settings shows
each key's last use.

Send a key as `Authorization: Bearer kcl_…`, or as the password of HTTP Basic with any username for a
tool that takes only a login. Locked notes, the trash and `#ai-exclude` notes are never visible to a
key. A request without a valid key gets `401` and a message that says how to make one; so does
`kanso mcp` without `KANSO_MCP_TOKEN`. Making a key, what each access level allows, every error and
its fix, and the network cautions are in [docs/LOCAL-API.md](docs/LOCAL-API.md).

**Upgrading.** The local API's username and password are gone, and agents set up with them stopped
working at the update. If you had a login, a banner reads **Your AI agents and scripts need new
keys**: **How to fix** gives the steps (make a key for each agent or script, paste the new setup in
place of the old Kanso entry, restart the agent; a script sends `Authorization: Bearer <key>`).
**Dismiss** hides the banner; **Mark as fixed** also drops it from the palette's **Update notices**.
The notice stays under **Needs your attention** in What's new.

### Open loops, the morning brief and the weekly review

**Open loops** are the promises your notes make: sentences like "I'll send the deck", "waiting on Rudo
for the invoice" or "need to find out whether...". Kanso finds them with phrase rules on this device
(English only; the rules are data, in `crates/kanso-core/src/loops`), with no AI and nothing sent
anywhere. **Open Loops** in the sidebar (and the palette) lists them by kind (I will, Waiting on, To
find out) and by how long ago the note changed. For each one you can mark it **Done** (remembered; with
**Mark done loops in the note** on in Settings, Kanso also adds a tick after the sentence), **Not a
loop** (hidden for good) or **Make Task** (the sentence becomes a `- [ ]` task in its note). Sentences
inside code, quotes and links are skipped, and locked notes have no loops.

**Morning Brief** shows what is due today or earlier, the open loops, timed tasks still to come today,
and past notes that matter today (linked from today's daily note, or written a week, a month or a
year ago today, or similar to what's due). It is worked out when you open it. **Weekly Review** shows
the last seven days: what you wrote and finished, what is still open, overdue tasks, the themes that
came up (tags and repeated words, counted on this device) and what is due next week; step back through
earlier weeks. **Save as Note** writes it to the Journal as "Weekly review - 2026-W45"; saving the same
week again asks before replacing it.

Both can notify at a time you choose in **Settings > General > Brief and review** (the morning brief
every day or on weekdays, the review on the day you pick). The notification says only counts, like
"3 due, 2 open loops", never a note's words, and shows once a day, at most three hours late, through the
same path as task reminders. On phones the next week of notifications is handed to the system ahead of
time with fixed text. Clicking one opens the brief or the review.

**Find More Open Loops...** (the palette, or the button in Open Loops) is the only AI part, and only
while AI is on in Settings > AI: it reads the notes changed in the last 14 days with your model and
proposes sentences the rules missed. Each proposal is checked against the note's real text, and none
is saved until you approve it. Locked notes and `#ai-exclude` notes are never sent. **Summarise**
on the brief and the review (AI on only) writes a short paragraph from the same view, with its sources.

In the local API, `kanso-list-open-loops` (MCP) and `GET /loops` (REST) list the open loops, with the
same exclusions as everywhere else (locked, trashed and `#ai-exclude` notes have none there). Which loops you have closed or hidden syncs with the note (the
`kansoLoops` field); if two devices close different loops of the same note at once, the later write
wins, as with `kansoReminderAck`.

### Icon themes

An icon theme draws every icon in Kanso, the controls' own included, and switches at once:
Settings › **Icon Themes**, or **Change icon theme…** in the palette. Kanso's own Lucide icons,
Phosphor and 13 themes converted from Zed's (Catppuccin, Material, JetBrains and more) are built in; a theme can also restyle the library's own icons (a notebook folded or
open, the Inbox, a pinned note) and add icons to a notebook's **Change Icon…** picker. Add your own
Settings › Icon Themes › **Browse** installs and updates icon themes from the signed
[theme gallery](docs/THEMES.md), apart from your own files. A theme can also be
a folder of SVGs and one JSON file in the `icon-themes` folder of Kanso's data folder
(Settings › Icon Themes › Icon theme files › **Open Folder**), or import a Zed icon theme with
**Import…** there or `kanso icon-theme import <path>`. An icon a theme leaves out stays Kanso's own.
[themes/icon-themes/README.md](https://github.com/iamngoni/kanso-themes/blob/main/icon-themes/README.md) has the format.

### AI

AI is off until you turn it on in Settings › AI. Then **Ask Your Notes…** (⇧⌘A) answers a question
from your own notes, with links to the notes the answer used. **Summarize this note**, in the Ask
panel (also **Summarize Note** in the command palette), sums up the open note without changing it.
Select some text and right-click for **Improve Writing**, **Make Shorter**, **Fix Spelling and
Grammar**, **Continue Writing** and **Summarize Selection**: each suggests new text, which you
**Accept** or **Discard**. Locked notes, notes in the trash and notes tagged `#ai-exclude` are never
sent, and a feature on such a note sends nothing.

**Agent tasks.** **Agent Task…** (File menu and the command palette, or **Ask an Agent About N Notes…**
on a selection) gives the AI a goal such as "merge my Lisbon trip notes into one plan". It reads
the notes you allow and proposes new notes, edits, moves and merges, and writes nothing. When it
finishes, the Agents tab lists the changes as cards you tick and **Apply** in one step; **Undo
Agent Task** (also 30 seconds in the status line) takes them all back, and each note's history
names the agent. Changes are kept in memory only, never synced, and a note that changed since the
agent read it is flagged. A task stops after 20 minutes or 200 tool calls, and at most two run at
once. Locked notes, the trash and `#ai-exclude` notes are never readable, and a task may only touch
notes it was allowed to read. On the phone, tasks run with an API key (Ask page › Agent tasks).
Settings › AI › Agent tasks turns them off or limits what an agent may propose.

**Answers come from** one of two places, chosen in Settings › AI:

- **A provider you have a key for**, or a model on this computer or your network (Ollama, LM Studio
  or any OpenAI-compatible server). The key is kept in the system keychain (macOS Keychain, Windows
  Credential Manager, the Secret Service on Linux), never in the settings file. Questions and the
  excerpts they use, with their titles, dates and notebook names, go straight from this computer
  to that provider.
- **An agent on this computer** that speaks the [Agent Client Protocol](https://agentclientprotocol.com):
  Claude Agent, Codex, Gemini CLI or OpenCode, signed in with your own subscription, or another
  agent from the ACP registry, or your own command. The agent keeps its own sign-in, model and
  billing; Kanso doesn't proxy or keep what it sends.

**What an agent is sent.** Ask sends the question and up to eight excerpts from notes chosen on this
computer. With **Note access** on (it is on unless you turn it off), the agent may also search and
read notes through Kanso's read-only notes server, with a token that ends with the conversation and
only the notebooks you pick under **Notebooks**; every note it reads is counted and listed under
the answer as **Used notes**. Summaries and the inline items send only the note or selection they
work on, and get no search. **These notes leave your computer**: the agent sends what it is sent to
its provider (Anthropic, OpenAI, Google, or whichever one it uses), and it may keep its own history
of the conversation. Kanso can't stop that, so leave AI off, or tag a note `#ai-exclude`, for notes
that must not go anywhere.

**What Kanso refuses.** Kanso gives an agent no file system and no terminal, and refuses its
requests for them. When it asks to use one of its own tools, a sheet names the agent and the tool,
and **Reject** is the default (Enter and Escape reject); **Allow once** allows that one request and
is never remembered. Kanso starts the agent in an empty private folder inside its data folder, with
an environment that carries none of Kanso's settings or secrets, and asks for the agent's most
restrictive mode where it offers one. That is not a sandbox: the agent runs as you, and once you
allow a tool Kanso can't see or limit what it does. Notes are sent as data, not instructions, but a
note can still steer an answer, so check the citations.

**Suggested edits.** With **Let the agent suggest edits** on (under Note access, off by default), an
agent can propose changes to a note's title and text, or a new note. Nothing is written until you
press **Apply**: **Review** in the Ask panel opens the suggested edits, each as a diff, and
**Discard** drops one. If the note changed since the agent read it, Kanso refuses the edit and shows
both versions. Suggestions are kept in memory for an hour and are gone when Kanso quits. An applied
edit keeps the version it replaced in the note's history, naming the agent.

**Signing in.** Install an agent with the command shown in Settings › AI (for an npm package,
**Install…** runs `npm install -g` after you confirm), then **Test**. If the agent needs a sign-in,
**Sign in** hands it to the agent's own sign-in, usually in your browser; Kanso never sees the
password or token. After installing, **Find again** looks for the agent once more.

**When something goes wrong**, Settings › AI › **Copy diagnostics**, beside Test, copies a report
you can paste into an issue: Kanso's version and this computer's system, whether the agent was found
and where, its name and version, whether it is running, and the names, timings and error codes of
the last protocol messages. It carries no note text or titles, questions, answers, tokens or the
agent's own output, and your home folder is written `~`. The agent's own log may say more.

**Removing an agent.** Uninstall it the way you installed it (for example `npm uninstall -g
@agentclientprotocol/claude-agent-acp`). If **Answers come from** is a provider, nothing changes and
nothing shows an error. If it is that agent, Ask and the other AI features say it isn't installed
and point you to Settings › AI; choose another agent or a provider there. Switching to a provider
stops every agent at once.

### Shortcuts

On Windows and Linux, Ctrl takes the place of ⌘ (Ctrl+K, Ctrl+Shift+F and so on), and Alt the place
of ⌥.

| | |
|---|---|
| ⌘K | Command palette: every command, notebook, tag and status, plus note search |
| ⌘P | Open a note by title |
| ⌘N / ⇧⌘N | New note / notebook |
| ⌘D | Today's note |
| ⌃⌥Space | Quick capture from any app (change it in Settings › General) |
| ⇧⌘F | Search all notes (`book:`, `tag:`, `status:`, `title:`, `body:`, `"phrases"`, `-exclude`) |
| ⇧⌘H | Find and replace in all notes |
| ⌘L | Search the list that is showing |
| ↑ ↓ or J K in the list, Return | Move through notes, edit the selected one |
| ⇧⌘L | Back to the note list |
| ⌥⌘↓ / ⌥⌘↑ | Next / previous note from anywhere |
| ⌘1 / ⌘2 / ⌘3 / ⌘4 | Edit / split (drag the divider) / preview / live preview |
| ⇧⌘D | Focus mode: only the note; Escape leaves it |
| ⌘\ | Fold the sidebar to icons, or open it again |
| ⌥⌘I | Outline and backlinks beside the note |
| ⇧⌘V | Paste as plain text |
| ⇧⌘P | Pin or unpin |
| ⌘S | Save now (notes also save on their own) |
| ⌘B / ⌘I / ⇧⌘X | Bold / italic / strikethrough the selection; again takes it off. With nothing selected the markers go in and the caret between them |
| ⌘E / ⌥⌘C | Inline code / code block |
| ⇧⌘K | Link: the selection becomes the text and the address is selected |
| ⌥⌘1 to ⌥⌘6 | Heading 1 to 6 on the line (again takes it off) |
| ⌥⌘9 / ⌥⌘8 / ⌥⌘7 | Task / bulleted / numbered list on every selected line (again takes it off) |
| ⇧⌘. | Quote the selected lines |
| ⌘, | Settings, inside the window; Escape goes back to the notes |

The formatting shortcuts work in the note's text, not in the title or a search box, and in Vim mode only
in Insert mode. They are in Note › Format and in the toolbar's tooltips.

### Command line

```bash
kanso                       # open the app
kanso serve                 # only the API (REST and /mcp), no window; syncs when signed in
kanso account sign-in --username ngoni   # sign a headless library in; then kanso serve syncs it
kanso account status        # account, this device, last sync (works while serve runs)
kanso mcp                   # MCP over stdin/stdout for agents that start a program; needs KANSO_MCP_TOKEN=<API key>
kanso api-key create --name "Backup script"   # make an API key with no window (list, revoke; works while serve runs); prints only the key
kanso import obsidian ~/Vault --dry-run   # what an import would bring in
kanso import notion ~/Downloads/Export.zip  # markdown, obsidian, notion, evernote, …
kanso import apple-notes                  # the Notes library on this Mac
kanso import pouchdb        # the PouchDB library on this computer; --credentials copies its API port and address
kanso capture "Call Sam"    # save a note to the Inbox (--book <name> picks the notebook); needs no API key
pbpaste | kanso capture     # with no text, it reads standard input
```

Every command takes `--data-dir <dir>`. The default is `$KANSO_DATA_DIR` when set, otherwise
`~/Library/Application Support/Kanso` on macOS, `%APPDATA%\Kanso` on Windows and
`~/.local/share/kanso` (`$XDG_DATA_HOME/kanso`) on Linux.

### Headless / VPS

`kanso serve` runs the API and `/mcp` without a window, so an agent on a server (Codex, say) can use
your notes, with an API key. Keys are stored in the library, which never syncs them. On the server,
`kanso api-key create --name <name>` prints a new key and nothing else (`list` and `revoke` do the
rest). They work while `kanso serve` or the app runs, so a leaked key is revoked in under a second
with nothing stopped, and a request it already started does not finish. Until the library has a
key, `kanso serve` and `kanso mcp` refuse every request. Sign the server's library in to your account once, with the service stopped, and
`kanso serve` syncs it while it runs: at start, every `sync.intervalSecs` (5 minutes), and 20
seconds after any API or MCP write.

```bash
sudo useradd --system --home-dir /var/lib/kanso --shell /usr/sbin/nologin kanso
sudo install -d -o kanso -g kanso -m 700 /var/lib/kanso
sudo cp assets/linux/kanso.service /etc/systemd/system/ && sudo systemctl daemon-reload
sudo -u kanso KANSO_DATA_DIR=/var/lib/kanso kanso account sign-in --username ngoni
sudo systemctl enable --now kanso
```

Sign in again later with the service stopped (`sudo systemctl stop kanso` first).

Sign in with your password (typed, or on the first line of standard input with
`--password-stdin`), or with `--provider github`, approved from a device that's already signed in
with a code you compare on both. Google, Apple, passkeys and security keys need a browser on the
server and are refused; the command never creates or recovers an account. `kanso account devices`,
`revoke <id>` and `sign-out` do the rest. If another device signs the server out, `kanso serve`
says so once and keeps serving the library.

A server has no keyring, so the session goes in `secrets.json` in the data folder (owner-only,
`0600`; `--secrets file`, the default when no Secret Service answers). **That file holds the key
that decrypts your notes, and the key that decrypts the library itself.** Kanso's sync server still
never sees it, but anyone with root on the server can read it, and without `KANSO_SECRETS_KEY_FILE`
the library's encryption stops only a copy of `kanso.db` taken on its own, not anyone who can read the
data folder. To keep it out of backups and snapshots, seal it under a key held elsewhere:
point `KANSO_SECRETS_KEY_FILE` at a file of 32 random bytes (for example a systemd credential,
`LoadCredentialEncrypted=kanso-secrets-key:…` with
`Environment=KANSO_SECRETS_KEY_FILE=%d/kanso-secrets-key`). A file written before the key was set is
sealed the next time `kanso serve` starts with it; from then on, every command that reads it needs
the same `KANSO_SECRETS_KEY_FILE`. A systemd credential exists only inside the service, so with one,
`kanso account` can't read the file from a shell: run it in a transient unit given the service's
`User=`, `LoadCredentialEncrypted=` and `Environment=` lines (`systemd-run --pty -p …`), or keep the
key in a file only the `kanso` user can read instead. Before you delete the server, run
`kanso account sign-out` (or sign it out from another device).

An example unit is in [`assets/linux/kanso.service`](assets/linux/kanso.service). Keep `bindAddress`
at `127.0.0.1` and reach the API through an SSH tunnel (`ssh -L 19840:127.0.0.1:19840 vps`) or a TLS
proxy; agents on the server itself use `kanso mcp`, with `KANSO_MCP_TOKEN` set to a key. The binary still links the desktop's shared
libraries, so install them even without a display:

```bash
sudo apt install libwayland-client0 libxkbcommon0 libxkbcommon-x11-0 libvulkan1 libegl1 libx11-6 \
  libx11-xcb1 libxcb1 libxcb-randr0 libxcb-shape0 libxcb-shm0 libxcb-xfixes0 libfontconfig1 \
  libfreetype6 libssl3 libasound2 libdbus-1-3 libzstd1 libfido2-1
```

## Development

The built-in color and icon themes are the
[kanso-themes](https://github.com/iamngoni/kanso-themes) repository, checked out
at `themes/` as a submodule: clone with `--recursive`, or after cloning run

```bash
git submodule update --init
```

A theme change goes to kanso-themes first; moving the submodule to that commit
puts it in the next build.

```bash
PROPTEST_RNG_SEED=20261017 cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
(cd cloud && npm ci && npm test)
cloud/scripts/e2e-local.sh   # the Worker under wrangler dev and the Rust client against it
```

| Crate | |
|---|---|
| `kanso-core` | Document types, revisions, the search query language, task and link parsing |
| `kanso-store` | SQLite store: writes, search, history, change feed, backups, sync bookkeeping |
| `kanso-import` | Importers for folders of Markdown, other apps' exports and PouchDB-on-SQLite libraries, and Electron safeStorage decryption |
| `kanso-server` | The local REST API (compatible with popular local-server clients) and the native MCP endpoint |
| `kanso-sync` | Accounts and end-to-end encrypted sync, and the `kanso-sync` command (`verify`, `estimate`, `unbind`) |
| `kanso-app` | The GPUI app and the `kanso` command |
| `cloud/` | The sync Worker (TypeScript, D1, R2) and its admin script |

### Cutting a release

On a clean, up-to-date `master`:

```bash
scripts/release.sh 0.2.0 --push
```

It bumps the version, writes the release's section of `CHANGELOG.md`, commits, tags `v0.2.0` and
pushes. The tag starts [.github/workflows/release.yml](.github/workflows/release.yml), which builds
every platform, signs and notarizes the Mac app, signs the update feed, checks every file and
publishes the release; installed copies then update themselves. If any check fails, nothing is
published. Running the workflow by hand (Actions › Release › Run workflow) builds the same
downloads, versioned `<version>-dev.<commit>`, as workflow artifacts without making a release.
[docs/RELEASING.md](docs/RELEASING.md) has the details, including `scripts/release.sh publish` for
a release stuck as a draft.

Design notes and decisions are in [docs/PLAN.md](docs/PLAN.md); accounts and encrypted sync in
[docs/ACCOUNTS.md](docs/ACCOUNTS.md); the local API in [docs/LOCAL-API.md](docs/LOCAL-API.md).

## Licence

Kanso is free software under the [GNU Affero General Public License, version 3 or any later
version](LICENSE) (`AGPL-3.0-or-later`): the app, its crates and the sync Worker in [cloud/](cloud/).
You may use, study, change and share it. If you share a changed Kanso, or run a changed Worker as a
service other people use, you must offer them its source under the same licence.

Contributions are welcome. By opening a pull request you agree that your contribution is licensed
under the same terms.

Kanso's dependencies keep their own licences (MIT, Apache-2.0, BSD and similar; a few files under
MPL-2.0). The models Kanso downloads on request keep theirs: voice input's
[Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) by NVIDIA is
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), in istupakov's
[ONNX conversion](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx); related notes'
[potion-base-8M](https://huggingface.co/minishlab/potion-base-8M) is MIT.
