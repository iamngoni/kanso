# Changelog

Every release of Kanso, newest first. Each entry links its commit, and each
heading compares the release with the one before.

## Unreleased

### New

- **mobile:** The Tasks place's library: open tasks across the notes grouped Overdue, Today, Upcoming and No date (and Completed when shown) as the desktop's Tasks view groups them, with a notebook filter, the overdue count for the place's badge, ticking one as one undo step, and locked notes left out whether their lock is open or not; the grouping now lives in the shared engine (#48 step 7)
- **mobile:** The Tasks place on the phone and tablet: a Tasks row in the library with the overdue count, and a list of the tasks in the desktop's groups with due chips, a notebook filter and Show Completed, ticking with a tap (struck through in place, with Undo), the note a tap opens, pull to sync, and the keyboard's arrows, space, return and undo on a tablet (#48 step 7)
- **editor:** Formatting shortcuts in the note's text, shown in the toolbar's tooltips and More menu and in Note › Format: ⌘B bold, ⌘I italic, ⇧⌘X strikethrough, ⌘E code, ⌥⌘C code block, ⇧⌘K link, ⌥⌘1 to ⌥⌘6 headings, ⌥⌘9 task, ⌥⌘8 bulleted and ⌥⌘7 numbered list, ⇧⌘. quote (Ctrl on Windows and Linux). Each acts on the selection (every line it touches for line formats), takes the format off again when pressed twice, and is one undo step. They don't fire in the title or search, or in Vim's Normal mode
- **plugins:** The plugin directory is the `iamngoni/kanso-plugins` repository, fetched only from its path on raw.githubusercontent.com and signed by its CI with the directory key now compiled in; Kanso checks it 30 seconds after opening and every six hours while a plugin from it is installed, for withdrawn versions and updates; `kanso-plugin build-directory` and `verify-directory` build and check it; the Windows and Linux updaters replace `kanso-plugin-host` with `kanso`, all or nothing (#37)
- **themes:** The theme gallery's signed catalog, path rules, install plan, key list and SVG sanitizer, with imported and on-disk icon themes held to it (#49 steps 1–2)
- **themes:** Install, update and remove gallery color and icon themes under `<data>/gallery/`, with atomic swaps, crash-safe state, edited-copy detection, and the gallery-aware theme loaders (#49 step 3)
- **themes:** Browse the gallery from Settings › Themes and Icon Themes, with search, filters, paired cards, live color preview and install/use/remove actions (#49 step 4)
- **themes:** Check the gallery on the app's update timer, update unedited installed themes quietly, honour signed revocations (an edited revoked copy is disabled, not deleted), fall back when a selected theme is withdrawn, and add the "Keep gallery themes up to date" switch and `themes.log` (#49 step 5)
- **themes:** Document the gallery (Browse, badges, offline, edited copies, revocation, built-ins), add the loopback end-to-end test and Browse snapshots, and wire the `KANSO_GALLERY_INDEX` debug override (#49 step 7)
- **ai:** Answer with an agent on this computer: Settings › AI › Answers come from adds Claude Agent, Codex, Gemini CLI, OpenCode or a custom ACP command, used through the person's own subscription. Kanso starts the agent in an empty private folder with a scrubbed environment, offers it no file or terminal access, rejects its permission requests, and picks its most restrictive mode (#46 steps 0–4)
- **ai:** An agent may also search and read the notes itself through a read-only notes server: a short-lived token per session (Bearer on `/mcp` only, or a private loopback listener when the local API is off), never locked, trashed, `#ai-exclude` or out-of-scope notes, every note it reads under Used notes, and a notebook scope sheet; follow-ups keep one agent session per panel conversation; more agents from the ACP registry, marked "Not tested with Kanso", with a confirmed one-click `npm install -g` (#46 step 5)
- **ai:** Let an agent suggest edits to the text or title of the notes it may read, or a new note, from Settings › AI (off by default): each suggestion waits in a review queue with a diff and writes nothing until you apply it, once, if the note is unchanged and still not locked, trashed or `#ai-exclude`; a note's history names the agent (#46 step 7)
- **ai:** An inline assistant in the editor (Improve, Make Shorter, Fix Spelling and Grammar, Continue, Summarize the selection, from the right-click menu or the palette) and a note summary in the Ask panel, on the chosen backend with no tools and no notes access; a suggestion is applied only when you accept it, in one undo step, and a locked, trashed or `#ai-exclude` note is never sent (#46 step 6)
- **ai:** When an agent asks to use one of its own tools, a sheet names the agent, the tool and its kind, with Reject as the default (Enter and Escape reject) and Allow once for just that request; Kanso never answers "always", a request waits a minute at most and is cancelled with the turn, and without a window it is rejected (#46 step 4)
- **ai:** Copy diagnostics in Settings › AI, beside Test, copies a report to paste into an issue: Kanso's version and system, whether the agent was found and where, its name and version, whether it runs, and the names, timings and error codes of its last messages. It holds no note text, titles, answers, tokens or the agent's own output, and a crash can still be diagnosed (#46 step 8)
- **plugins:** Plugins, with notes kept safe: palette commands, fenced blocks and inspector panels from JavaScript plugins, each run in its own sandboxed `kanso-plugin-host` process with time and memory budgets. A plugin reaches only the notebooks you chose at install, never locked, trashed or `#ai-exclude` notes; its changes are staged, reviewed when they reach beyond the open note, and undone in one step. Network only to its listed HTTPS hosts through Kanso's proxy, tokens in the Keychain, a signed directory with author keys pinned on first install, developer mode for unsigned folders, safe mode (`--safe-mode` or Shift at launch), `kanso plugin dev` and the `kanso-plugin` packing and signing tool (docs/PLUGINS.md) (#37)
- **engine:** Lock sessions, account flows and sync scheduling move into the new UI-free `kanso-engine` crate, shared with the coming mobile app; the desktop behaves as before (#48 step 1)
- **engine:** Wiki-link and tag autocomplete, link copying, daily notes and templates, due dates and reminders, the Markdown edits and the theme format move into `kanso-engine`, which also gets a Markdown and code highlighter of its own, checked against the desktop editor's, with Enter continuing a list item and UTF-16 offsets for the mobile editor; the desktop behaves as before, except that Tab no longer fails on lines selected down to the start of the line below a list (#48 step 2)
- **engine:** The store, renderer, shared notes, importers, sync client and AI client build for iOS and Android and pass their tests on the iOS Simulator and an Android emulator, with a `mobile` release profile that is small and unwinds on a panic; phones check servers' certificates with the system's store (the new `kanso-tls` crate), and the desktop behaves as before (#48 step 3)
- **engine:** A note's version history (why a version was kept, who made it, the changed sections and words), the Replace options, words and batch IDs, and the backup schedule (folder, daily and manual backups, the list, deleting one made on request) move into `kanso-engine`, shared with the mobile app; the desktop behaves as before (#48 parity D, H)
- **mobile:** The library gives the phones a note's version history, what changed since a version and restoring one, Replace in all notes with its undo, and daily and on-request backups inside the library's folder; a locked note's versions are opened only while its lock is open, and Replace never touches a locked note (#48 parity D, H)
- **sync:** The sync server can wake a signed-in device with a silent push when another device changes something: devices register an APNs or FCM token (`PUT /v2/account/devices/<id>/push`), a push or a shared-note update wakes the account's other devices and the note's other members' devices, at most one push per device every 30 seconds with a trailing one never lost, tokens are cleared on sign-out, revoke, expiry or a new sign-in and when the provider reports them gone, and no push carries a note, title, count or name (#48 step 8b)
- **mobile:** Release engineering for the iPhone, iPad and Android apps: one version from `Cargo.toml`, a signed IPA to TestFlight and a signed bundle to Play internal testing from the `v*` tag (every step waits for its secret), Shorebird patches made only from CI and signed, the export compliance answer, the apps' privacy manifest, and the store listing text (docs/RELEASING-MOBILE.md) (#48 step 13)
- **mobile:** A swipe or a change to several notes shows a short notice at the foot of the screen, worded as the desktop's is, with Undo that puts back what each note had (a field changed again since keeps its newer value), over the list and the note on a phone and a tablet (#48 step 7)
- **mobile:** The desktop's bundled icon themes, chosen under Settings › Appearance › Icons: each theme listed with a row of its icons and its author and licence, the choice applied to every icon in the app at once and kept on the phone, and an icon a theme leaves out drawn as Kanso's own; the icon theme format is read by `kanso-engine` (as the desktop reads it) and the library carries only the SVGs the phone draws (#48 step 7)
- **mobile:** Crash reports without an SDK and without note content: the iPhone, iPad and Android apps keep a report when they crash (MetricKit, Android's exit reasons and ANRs, uncaught JVM and Dart errors), built from a whitelist of fields (stack frames in a strict grammar, versions, the kind of device and the day), shown in full, and sent only when you tap Send; the sync server keeps it for 90 days with no IP address or account (docs/CRASH-REPORTS.md) (#48 step 13)
- **mobile:** Settings › About links to the release notes and help on kansoapp.app, shows the Shorebird patch number, and the store pictures for every required size are rendered from the golden tests on each build (#48 step 13)
- **site:** Release notes and Help pages, and a Crash reports section on the privacy page (#48 step 13)
- **mobile:** The note's preview for the phones' app (#48 step 7): the published page with tappable notes, wiki-links to headings, notes to make, tags, attachments, task boxes, due-date chips, query views and a copy link on every code block, headings anchored, code colored by the theme, failed diagrams shown with the reason and empty drawings as the desktop shows them; it carries no script, handler or fetch, and every address not on the app's short list is dropped. Ticking a box writes only that box and refuses a note changed since the page was drawn; a locked note's text is read only while it is open
- **mobile:** A note in Preview is its rendered page, under its title and facts: a tapped note link opens the note, a wiki-link to a note not made yet makes it and opens it, a tag opens its list, a heading link scrolls to it, a web or mail link goes to the system, an image attachment opens full screen and any other file opens a sheet to share it, a task box is ticked in one write with Undo, and a code block's button copies it; the page follows the theme and is drawn again when the library changes (#48 step 7)
- **mobile:** Locked notes on the phones' app (#48 step 9): lock a note or a notebook, unlock with the lock password or Face ID, Touch ID or fingerprint, change the lock password and remove a lock, the desktop's auto-lock and a lock when the app goes to the background, and a cover over a locked note in the app switcher (`FLAG_SECURE` on Android) so its text never shows there
- **mobile:** Accounts on the phones' app (#48 step 8): sign up, sign in with a password, passkey or Apple, Google or GitHub, join an account from another device by approval or with the recovery key, approve a new device from a phone, see and revoke devices, and sign out, all in the desktop's words; signing in publishes the account's sharing identity so others can share with a phone-only account at once
- **mobile:** Sync on the phones (#48 step 8): Settings › Sync says when the library last synced or why it couldn't ("Synced 5 min ago", paused until 14:05, signed out), with Sync now, Sync automatically and Sync every (1 minute to 4 hours) as the desktop has them; the app syncs when it comes to the front, soon after an edit and on the chosen interval, and pulling down the note list or Tasks syncs
- **mobile:** Shared notes on the phones (#48 step 8): share a note with people by username, with the first-share fingerprint to compare, roles, links that expire, requests to approve, stopping and leaving (each keeps a private copy), and Shared with you for invitations and joining by link; an open shared note is live, so what others type appears within a moment and what you type reaches them, and someone's typing never undoes yours
- **mobile:** Comments on shared notes (#48 step 8): the note's bar has Comments with the number open; the sheet lists them in text order, then those whose words were deleted, with Resolve and Reopen, Edit and Delete for your own, and @mentions; choosing one opens the editor on its words, the commented words are tinted, and Comment in the text menu comments on the selection
- **mobile:** Others in a shared note (#48 step 8): their carets with their names and their selections in their colours, as the desktop draws them; a status line under the bar ("Shared · Can view · Read only · Live", a private copy once sharing ends, and the size limit when it is near) and a banner for what couldn't be done, with Dismiss; a note shared to view or comment on opens read only
- **mobile:** The theme gallery on the phones: browse and search the gallery's color and icon themes from Settings › Appearance, with their badges, and install one from its card (#48 step 8)

### Fixed

- **export:** Published notes and exported pages keep their code readable when Kanso uses a dark theme: a light page takes the code colors of your light theme (Kanso Light's if it can't be found), where it had pale colors meant for a dark background
- **accounts:** Approving a new device from another one works without restarting Kanso first, and cancelling the sign-in cancels that request
- **app:** Kanso is listed in Finder's and Explorer's Open With for Markdown and text files, and opens a file from there, from a double-click once chosen, or from `kanso file.md`, as a note that is shown and selected
- **sharing:** A comment or someone's caret in a shared note stays on its words after an accented letter or an emoji earlier in the text, where it could drift by a character or two
- **themes:** Code is easier to read in Kanso Dark and Kanso Light: functions, types, properties, attributes, keywords, strings and numbers each get their own colour, and operators and punctuation a quiet one, all at 4.5:1 or better on the editor and the code block background
- **replace:** Replace in all notes never writes into a locked note (sealed, or in a locked notebook), whatever the preview named, and Undo replace leaves a note that was locked since instead of failing or writing plain text into it (#48 parity H)

## [0.8.4](https://github.com/iamngoni/kanso/compare/v0.8.3...v0.8.4) - 2026-10-22

### New

- Import Apple Notes from its default library location ([184aa3e](https://github.com/iamngoni/kanso/commit/184aa3e4bada92bcbf7bb20245a11da9645aaabb))

### Fixed

- Prevent overlapping link scan matches ([b43bb79](https://github.com/iamngoni/kanso/commit/b43bb790ef90259e0cce570cce2c0bc513e074c9))

## [0.8.3](https://github.com/iamngoni/kanso/compare/v0.8.2...v0.8.3) - 2026-10-22

### Fixed

- **import:** Apple Notes builds on Linux and Windows again, and imports long first lines, links and Math Notes results intact ([f41ffb7](https://github.com/iamngoni/kanso/commit/f41ffb7f0dd22daed53ffcce904caad50d5b6a9e))
- **import:** A TextBundle or TextPack opened on its own imports, and any text.* is its body ([a22ac25](https://github.com/iamngoni/kanso/commit/a22ac252f7d8ad4c3f3f9ac1a8a5646cae669bf6))
- **import:** Roam pages keep their dates, block references, aliases and multi-line blocks ([90084d4](https://github.com/iamngoni/kanso/commit/90084d4d647d14caf165ae18c4cd84100311c192))
- **import:** Tomboy's deleted notes go to the trash, templates are left out, and notebooks are notebooks ([f472aac](https://github.com/iamngoni/kanso/commit/f472aac91e535f316ed3cf17e968d79c1f433eee))
- **import:** Deeply nested HTML no longer overflows the stack, and HTML links with spaces or parentheses resolve ([4c2c2df](https://github.com/iamngoni/kanso/commit/4c2c2df2889b93e2c8ade0176468e05b2e39b1af))
- **import:** Apple Journal entries keep their order, media, dates and places; Keep notes without an edit time import; big HTML pages strip scripts quickly ([0f87530](https://github.com/iamngoni/kanso/commit/0f87530425fe068c477d65fd9a75a3f33da1012c))

### Under the hood

- **import:** The link-finding property test only builds real code spans ([dd8db50](https://github.com/iamngoni/kanso/commit/dd8db50306c7ce4ca95ecda94f5bfe86bbf3b440))

## [0.8.2](https://github.com/iamngoni/kanso/compare/v0.8.1...v0.8.2) - 2026-10-22

### New

- Add support for seven new library import formats ([78d0792](https://github.com/iamngoni/kanso/commit/78d0792cbb972faf1576c283b668562652282826))

## [0.8.1](https://github.com/iamngoni/kanso/compare/v0.8.0...v0.8.1) - 2026-10-22

### New

- **voice:** The words you dictate show in the note as you say them ([63d9d0b](https://github.com/iamngoni/kanso/commit/63d9d0bb83c95d61ca0af6394558cf68d543284a))
- **icons:** Every icon theme from the themes repository comes with Kanso, on its own Settings page ([9b32f34](https://github.com/iamngoni/kanso/commit/9b32f34e23a3f161fa2908162ff255645ee5aaff))
- **editor:** Pick any installed font for writing, monospace and the preview, and size the preview on its own ([b073ad0](https://github.com/iamngoni/kanso/commit/b073ad0731f748fbf192ca477863480ddab96388))
- **app:** Zoom the whole app with ⌘= / ⌘− / ⌘0 ([c2a4eb6](https://github.com/iamngoni/kanso/commit/c2a4eb6b8a7f9bc1d15d6fb7dc623fbc0e54b083))
- **voice:** Spoken punctuation and commands for on-device dictation ([27f7cd4](https://github.com/iamngoni/kanso/commit/27f7cd4a5aa9c97ab12589ac60b2c727b84c98ec))
- **notes:** Go back and forward between notes, and copy a note's link ([f4b3c39](https://github.com/iamngoni/kanso/commit/f4b3c3983b64ebdc18173eee960152c3aa4a3f02))
- **editor:** Misspelled words in a note get a red wavy underline, with suggestions on right-click ([c12c129](https://github.com/iamngoni/kanso/commit/c12c1293bae88fa9d54d8b1973bb7b5eb4172533))
- **notes:** Open a note in its own window ([eedecab](https://github.com/iamngoni/kanso/commit/eedecabb886437a37fabc4f4c980e7c6f84e4132))
- **notes:** Select several notes and drag them onto a notebook or tag ([a41ef94](https://github.com/iamngoni/kanso/commit/a41ef944519b4233a44d945cc2ce4d256a2113a8))

### Fixed

- **voice:** Pressing the mic with System Dictation chosen starts dictation instead of closing Kanso ([7533747](https://github.com/iamngoni/kanso/commit/75337476b91e1c1bb80df7dd24fb7a6d53a4a40a))
- **history:** Versions stay newest first when the computer's clock goes back ([61a2570](https://github.com/iamngoni/kanso/commit/61a2570f2ffaa77eeb5273f9571aac33947c03be))
- **notes:** Copy link and open in window from a note window and from several selected notes ([a16afa1](https://github.com/iamngoni/kanso/commit/a16afa143c640587c8892ce0195cc589a5f6555b))
- **settings:** The spelling row uses an icon every built-in icon theme draws ([2cc3a54](https://github.com/iamngoni/kanso/commit/2cc3a5411f8c30dfd95e98f5474c177bf3057a71))
- Review fixes across the merged QOL features ([0466acb](https://github.com/iamngoni/kanso/commit/0466acb456349c7b6c3fe696f62f3eaa9bffa34b))
- **notes:** The back/forward buttons ask the store once per visit; Copy Note Link works in a note's own window ([b3da519](https://github.com/iamngoni/kanso/commit/b3da519752fd8d48b6f54f8f18bd8cf0d1e19f1b))

### Under the hood

- **themes:** The built-in themes come from the kanso-themes repository, as a submodule ([a05ee2a](https://github.com/iamngoni/kanso/commit/a05ee2a210ca223941966e3866f41da58221d790))
- Compile Speller::new only on macOS and in tests, fixing Linux clippy ([920fb4c](https://github.com/iamngoni/kanso/commit/920fb4c62e1a454973fa53d69e1b4661a9ca9a45)) by @gridopsbot[bot]

## [0.8.0](https://github.com/iamngoni/kanso/compare/v0.7.0...v0.8.0) - 2026-10-22

### New

- **icons:** Icon themes restyle every icon in Kanso and switch live, in place of icon packs ([02703c9](https://github.com/iamngoni/kanso/commit/02703c926d5e60df306eb963e419929b742e602c))

### Fixed

- **icons:** Imported icon themes keep their attachment icons, credits and licenses, and large Zed themes import ([a27f6c3](https://github.com/iamngoni/kanso/commit/a27f6c3288b07352792026b55202e61b81e73ba4))
- **collab:** Someone who joins a shared note sees the others' carets within seconds, not ten ([4b21503](https://github.com/iamngoni/kanso/commit/4b215030f2ea010df56f73d0a11158131676c114))

### Under the hood

- **icons:** The component library's own icons can follow an icon theme ([b6774a4](https://github.com/iamngoni/kanso/commit/b6774a4160c82aca64d121a6aa4df566ed730abb))
- **icons:** The README and the icon themes guide explain how to make, import and choose an icon theme ([b4c2186](https://github.com/iamngoni/kanso/commit/b4c2186c3a0923bd49c61def048244543de6741f))
- **cloud:** Record the 0.7.0 Worker deployment, its collaboration migration and the bookmark before it ([dba36ff](https://github.com/iamngoni/kanso/commit/dba36ff9c702d45339df1b0222b6892bd51fdd13))
- **site:** Parse each note once for quick open's destinations ([8bbd15a](https://github.com/iamngoni/kanso/commit/8bbd15a94fd82b68c706d75b4dd801b9282e7db2)) by @gridopsbot[bot]

## [0.7.0](https://github.com/iamngoni/kanso/compare/v0.6.1...v0.7.0) - 2026-10-21

### New

- **voice:** Kanso turns speech into text on the device with Parakeet ([b300d96](https://github.com/iamngoni/kanso/commit/b300d967354ed458145da8bad4b53dcbfa0d2f17))
- **editor:** Voice input writes what you say into the note ([8125099](https://github.com/iamngoni/kanso/commit/8125099925324487119dda878bdb650609b0b97b))
- **search:** The palette finds notes about what you type, not only notes with its words ([351d3b5](https://github.com/iamngoni/kanso/commit/351d3b5ec05f01fc8783348989c7d6b88c1dae75))
- **tags:** Kanso suggests the tags that notes about the same things have ([7816245](https://github.com/iamngoni/kanso/commit/78162452db0c71630ddf3b16a5dbae0887294fea))
- **editor:** A note that says nearly the same as another says so, and the two can be merged ([db1e581](https://github.com/iamngoni/kanso/commit/db1e581812dbabaa376dae9f53d89d09906f0f21))
- **voice:** A recording dropped into a note is transcribed on the device, under its link ([cd22f45](https://github.com/iamngoni/kanso/commit/cd22f45ffa48810a8be67f89fe4325dedc55b29e))
- **locked:** Windows Hello unlocks locked notes on Windows, with the lock password as the fallback ([6942a92](https://github.com/iamngoni/kanso/commit/6942a926bc0827e46bde246f647a3a27fe0c1d39))
- **locked:** Lock a whole notebook, and every note in it and in notebooks inside it is sealed, now and when notes go in later ([f3d3785](https://github.com/iamngoni/kanso/commit/f3d378548e6dbf4235099355938b1855e546c300))
- **board:** A board lays the list's notes out by status, and their tasks as To do and Done, moved by drag or keys ([c3e17e2](https://github.com/iamngoni/kanso/commit/c3e17e2d0eae0aec24f501dd63cdd7c6abf20822))
- **journal:** A month calendar marks the days with a daily note and opens or starts any day's note ([d4f9057](https://github.com/iamngoni/kanso/commit/d4f9057519791c623edb7887677d77a00e6921b2))
- **publish:** A notebook publishes as a small site at one link, sealed end to end, with its notes linked and its locked notes left out ([fb6db09](https://github.com/iamngoni/kanso/commit/fb6db0918d60e98bcfc545cd7c19a70ceb27c2ee))
- **collab:** Share a note with other Kanso users and edit it together, end-to-end encrypted ([ca8f47b](https://github.com/iamngoni/kanso/commit/ca8f47b9961135df853ad01c25b9d4ec8aa90ba6))
- **sidebar:** Notebooks fold like an accordion, and each can have its own icon ([d5f8aab](https://github.com/iamngoni/kanso/commit/d5f8aab4b75b0b698c441612b55da14a8073e85d)) by @gtchakama
- **sidebar:** The notebook icon picker has 144 icons and a search ([55c893a](https://github.com/iamngoni/kanso/commit/55c893af7c2fae1b3e69096c1c2e8e25aeb77770)) by @gtchakama
- **icons:** Icon packs give notebooks more icons and can draw Kanso's own ([3e9b295](https://github.com/iamngoni/kanso/commit/3e9b295a3a93fee9a9eeec5802427a500e01d143)) by @gtchakama
- **icons:** Browse and install icon packs from inside Kanso ([b012b07](https://github.com/iamngoni/kanso/commit/b012b074a48dec20d3ddca4763441137525cca6b)) by @gtchakama
- **icons:** An icon pack chosen for the app shows at once, without a restart ([483e991](https://github.com/iamngoni/kanso/commit/483e99198c48d9d88defb4ac73c82fedc3c5915e)) by @gtchakama

### Fixed

- **locked:** A locked notebook's text stays in Kanso through templates, moved notebooks, meaning results and late embeddings ([795bf8a](https://github.com/iamngoni/kanso/commit/795bf8ae13cfa78281ddcb155d5d341da122c53c))
- **journal:** The calendar builds on Windows and Linux, where the first weekday comes from no system setting ([283d34c](https://github.com/iamngoni/kanso/commit/283d34c3ae1bc06e264a31d529f422341e0b08f5))
- **collab:** Shared notes survive leaving members, deleted accounts, long text, late updates, large edits and locked copies ([8545620](https://github.com/iamngoni/kanso/commit/854562024a86bb9824816b26c2ca79ac41b560d5))
- **icons:** A notebook's pack icon shows from launch, and no icon pack, however it is made, can slow or stop Kanso ([2fa94e9](https://github.com/iamngoni/kanso/commit/2fa94e9beaedd605793249849c92d02e81b7cea5))
- **icons:** Uninstalling a pack keeps the files you added to it, and an install finishes and says so with the gallery closed ([c7ae156](https://github.com/iamngoni/kanso/commit/c7ae15684b66232670d3095cd959a1df7d0f6af6))
- **sidebar:** The arrow on a notebook around the one showing folds it, and the palette shows a locked notebook's lock ([f6b9ed8](https://github.com/iamngoni/kanso/commit/f6b9ed8a0ec2bb7e2b478c41d50d6f8c06b7dc5a))
- **icons:** Choosing an app icon pack lets go of the old window as closing it would, and voice input carries over ([2f9c4c0](https://github.com/iamngoni/kanso/commit/2f9c4c0a0eba1823a1ca221c5c15ce6e9d677dc4))

### Under the hood

- **app:** Voice input's rules, apart from the microphone ([da66f3b](https://github.com/iamngoni/kanso/commit/da66f3bbba7b7186795c4534d5b82868811ca758))
- **app:** The config remembers how voice input listens ([7545a4b](https://github.com/iamngoni/kanso/commit/7545a4b676a8abb1ea0021b319f7bbcd4357b080))
- **app:** Model downloads carry on where they stopped ([ead993f](https://github.com/iamngoni/kanso/commit/ead993fb0f3ae7d0498116c1efc265743cd2ac4b))
- **macos:** Kanso may ask for the microphone ([6903679](https://github.com/iamngoni/kanso/commit/690367966122fe788dcaacaee982b2cc4a6636de))
- **cloud:** The socket rate-limit test passes on a runner slow enough to refill every frame ([e5c76ca](https://github.com/iamngoni/kanso/commit/e5c76ca2b51b7bcf055fd753733d4c1c96e6bf27))
- **cloud:** The presence budget test waits for every frame's answer on a slow runner ([7f6d2d5](https://github.com/iamngoni/kanso/commit/7f6d2d5c05b35c52c0489f9bda7c077b8aa7d072))

## [0.6.1](https://github.com/iamngoni/kanso/compare/v0.6.0...v0.6.1) - 2026-10-21

### New

- **app:** Windows and Linux get a menu bar ([25c1c19](https://github.com/iamngoni/kanso/commit/25c1c198d64c327954248dedffbf99894f137ff9))
- **app:** A note's menu is the same in the list and the editor ([7912ce4](https://github.com/iamngoni/kanso/commit/7912ce4c27c60ee8fc164f0153bc73e20c7a58ed))
- **app:** Settings copies the MCP setup for your AI agent ([8a6e575](https://github.com/iamngoni/kanso/commit/8a6e5754a5771e03f1bc109cc94951c42a08c65c))
- **editor:** A Dictate button starts the system's dictation in the note ([fc99d25](https://github.com/iamngoni/kanso/commit/fc99d253de47b5be1071bac724176d2b36498530))
- **sync:** Kanso's server is sync.kansoapp.app, and published notes open on share.kansoapp.app ([48a79f8](https://github.com/iamngoni/kanso/commit/48a79f869b475265d157dd1d3e274978abb6094d))
- **app:** Kanso tells people they can sync, without getting in the way ([7b61dfe](https://github.com/iamngoni/kanso/commit/7b61dfe3c3735ba04b370df1811833c3455c4297))
- **locked:** Lock a note with a password, sealed on this device and in sync ([aff0a7c](https://github.com/iamngoni/kanso/commit/aff0a7c711c4ab57198c9b2c4cf30b861ffee2a6)) by @gtchakama
- **app:** Password fields have an eye to show what was typed ([2d2ca64](https://github.com/iamngoni/kanso/commit/2d2ca64b402b81823a8caef0fedb67740a6915ac)) by @gtchakama
- **locked:** Touch ID unlocks locked notes, with the lock password as the fallback ([9051847](https://github.com/iamngoni/kanso/commit/90518471036c4a7fcf1d15ced46dec92be0e3588)) by @gtchakama

### Fixed

- **locked:** A locked note's old text is gone on every device, and the API never serves its seal ([e94888e](https://github.com/iamngoni/kanso/commit/e94888e00006b5d01207b168926abcc4b5f132f7))

### Under the hood

- **excerpt:** The sample roadmap lists wiki-links instead of naming another app ([4646f32](https://github.com/iamngoni/kanso/commit/4646f328e9aade7b35967f762b6be04b48af2c8b))
- Kanso names other apps only where it imports from them ([07761e6](https://github.com/iamngoni/kanso/commit/07761e6cde665538e8b801d92a0b1fc027dc7565))
- **editor:** Dictation lands in a note as spoken ([a1e0a72](https://github.com/iamngoni/kanso/commit/a1e0a7267eb66220cdab2a493ba5882f8ac52f24))

## [0.6.0](https://github.com/iamngoni/kanso/compare/v0.5.2...v0.6.0) - 2026-10-20

### New

- **cloud:** Short links are made on go.kansoapp.app ([2c13feb](https://github.com/iamngoni/kanso/commit/2c13febb560e61646d9276540646960054a91556))
- **site:** Kanso's website, kansoapp.app ([e339fd3](https://github.com/iamngoni/kanso/commit/e339fd36f17a7dc5a871e3161907a022603e1e27))
- **import:** Markdown folders, Obsidian, Logseq, Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote and UpNote come in, with links, tags and attachments ([ccb4a03](https://github.com/iamngoni/kanso/commit/ccb4a034d1faf482ec2a4872fdcc0de88c053f39))
- **app:** Import… previews and imports other apps' notes, File › Open… brings in a Markdown file, and kanso import does both from a terminal ([d56f379](https://github.com/iamngoni/kanso/commit/d56f3799f4488399d70813b4b760a13529183869))
- **site:** The home page is Kanso, running in the browser ([4d03793](https://github.com/iamngoni/kanso/commit/4d037936d98b2a2f8383019880ff4d5bc9dc0dbb))

### Fixed

- **import:** A Joplin export's name comes from its file on Windows too ([df9b0aa](https://github.com/iamngoni/kanso/commit/df9b0aa56fbc4b413c51f36d8191ae2d2393a8b4))

### Under the hood

- **licence:** Kanso is free software under the AGPL-3.0-or-later ([9ff280e](https://github.com/iamngoni/kanso/commit/9ff280e1fea6c26cb3f969d959b00ec6a8c3ac22))
- **snapshots:** The demo roadmap lists wiki-links instead of naming another app ([40e8b93](https://github.com/iamngoni/kanso/commit/40e8b93deb8465296374e07787f96065fa20e771))
- **import:** The Notion export is written by its test, so Windows can check out the repository ([5c4da38](https://github.com/iamngoni/kanso/commit/5c4da38332106d8d864f587e79c8021d6f1e0ecb))

## [0.5.2](https://github.com/iamngoni/kanso/compare/v0.5.0...v0.5.2) - 2026-10-20

### New

- **cloud:** Published notes get a Rerout short link that previews the note's title ([46eb294](https://github.com/iamngoni/kanso/commit/46eb29486dc32741899efb2be47da632ef932923))
- **sync:** Publishing sends the note's title and shows the short link with the key ([d8dfc5f](https://github.com/iamngoni/kanso/commit/d8dfc5fa8b25ed5b3ab74d974eb99eb96d1a73b5))

### Fixed

- **import:** Importing a library on Windows builds a SQLite file URI it accepts ([515dcf8](https://github.com/iamngoni/kanso/commit/515dcf83f70ed27740ab84c071c2386f6bbf2ef7))
- **updates:** Restart quits and opens the new Kanso, instead of waiting for the next quit ([2cd9879](https://github.com/iamngoni/kanso/commit/2cd98798e2ca6d38f611f4d971180310e9e966a6))

### Under the hood

- Run every crate's tests but the app's on Windows and Linux too ([e4b2dfd](https://github.com/iamngoni/kanso/commit/e4b2dfd19183f4ed684bacc7d8afd386d0a83533))
- **e2e:** The wrong-password check holds across a rate-limit window boundary ([dedb0e3](https://github.com/iamngoni/kanso/commit/dedb0e355c433b4cf2bfb27d450e01037ec43db8))
- **changelog:** The 0.5.1 notes describe the Windows import fix without naming another app ([62594db](https://github.com/iamngoni/kanso/commit/62594dbefa32a50ff2bc30a28da8d4d6b77d6c20))
- Bind the e2e Worker's port in the same step that picks it ([2bd8f69](https://github.com/iamngoni/kanso/commit/2bd8f699e1f21ee7cf2b0fd14621ce82c5ffada2))

## [0.5.0](https://github.com/iamngoni/kanso/compare/v0.2.0...v0.5.0) - 2026-10-20

### New

- **release:** A per-user Windows installer beside the zip ([f66153a](https://github.com/iamngoni/kanso/commit/f66153a377dbab5b5aa139f56146421134c71a3a))
- **render:** Draw math, Mermaid diagrams and Excalidraw sketches in the theme's colours ([8594dfd](https://github.com/iamngoni/kanso/commit/8594dfd222cf8a03e8514d746db3e1ae7fbdaba2))
- **preview:** Show math, diagrams and Excalidraw drawings in the preview, and export notes as web pages or PDFs ([653584c](https://github.com/iamngoni/kanso/commit/653584c35a0a9b1dfda5e3eb734015af22a966b1))
- **mcp:** Add kanso-export-html to export a note as a standalone web page ([f39daf2](https://github.com/iamngoni/kanso/commit/f39daf284abae746c92179e2b19bed2f71635c67))
- **math:** Show $$…$$ on a line of its own as display math, centred, in the preview and exports ([4694608](https://github.com/iamngoni/kanso/commit/46946083be7502def1adbeb8a3e3f2b0399b57a2))
- **store:** Index each note's links, headings and tasks on this device ([14a021a](https://github.com/iamngoni/kanso/commit/14a021a1f20907eb93a9ae1c020421e2a815ba82))
- **mcp:** Add kanso-get-outline and kanso-resolve-wikilink tools ([489acc7](https://github.com/iamngoni/kanso/commit/489acc74adc4a8a20b72f6a8d7b54b8d3324a96b))
- **app:** An outline and backlinks beside the note, and wiki-links between notes ([aa87983](https://github.com/iamngoni/kanso/commit/aa87983703be72d9850c5cc38367265044ff93c6))
- **list:** Wiki-links read as their alias or title in note excerpts ([dd31e6f](https://github.com/iamngoni/kanso/commit/dd31e6f9f59a7afad74e8422db27024744d3e399))
- **list:** Math reads as its TeX in note excerpts ([7ffe1f5](https://github.com/iamngoni/kanso/commit/7ffe1f53a56f628077698d51e6bbf330ef895ae4))
- **cloud:** Publish a note as an encrypted page anyone with the link can read ([5f504da](https://github.com/iamngoni/kanso/commit/5f504da43a5069de48771180e5c938ca8b8bf2bc))
- **sync:** Seal and publish a note with its own key that never reaches the server ([99ef980](https://github.com/iamngoni/kanso/commit/99ef9805ee7937628839a484b42f39c0846eac84))
- **publish:** Publish a note to the web from its ⋯ menu, update it, or take it down ([28db263](https://github.com/iamngoni/kanso/commit/28db263732c6d77cbe482955bac2091ba21efec9))
- **store:** Daily notes, a Journal, Inbox and Templates, and tags nested with / ([8be2538](https://github.com/iamngoni/kanso/commit/8be2538476477481a700b18e4da74dadd5051a6b))
- **mcp:** Kanso-create-daily-note and kanso-create-from-template ([1e36f92](https://github.com/iamngoni/kanso/commit/1e36f9251fcc1170d954405e20236cde0df2fbee))
- **app:** Open at login on Windows, and as a macOS login item on macOS 13 and later ([4d93b00](https://github.com/iamngoni/kanso/commit/4d93b005aa815affde09893dcd0cfd6b46315f9e))
- **app:** Today's note, notes from templates, quick capture from any app and a tag tree ([028eb3f](https://github.com/iamngoni/kanso/commit/028eb3f7d33c6cba91ba84f3982071fde17b69e8))
- **cli:** Kanso capture --book saves to a notebook by its name or ID ([4b1ccc9](https://github.com/iamngoni/kanso/commit/4b1ccc962df0c81ea359b06fe525824b75078038))
- **editor:** Vim keys, with Normal, Insert and Visual modes in a note's text ([d19fc09](https://github.com/iamngoni/kanso/commit/d19fc096e6c2954969712807842a8a8aa0c8d6b5))
- **editor:** Vim keys gain H M L, ge, > and <, and . repeats Visual mode changes ([27e5964](https://github.com/iamngoni/kanso/commit/27e59645c93aa558f6c4aec47984b466ae8cf36c))
- **tasks:** Due dates show as chips in the preview and are tinted in the editor, with @ date completion ([6c91085](https://github.com/iamngoni/kanso/commit/6c9108560bca2fa856e22c0d873ebf74e0a95591))
- **tasks:** A Tasks view lists every task by when it's due, ticks them off in place and opens them at their line ([e436364](https://github.com/iamngoni/kanso/commit/e43636409cd846f1e2d2fc66cc0b9635c3b46e88))
- **reminders:** Tasks with a time send a notification when due, with Mark Done and Dismiss ([0e17f71](https://github.com/iamngoni/kanso/commit/0e17f71a1a1f2065e35062e7e553fe8d3a99cf7d))
- **replace:** Replace in all notes, with a live preview, options, and Undo replace ([ff34ddb](https://github.com/iamngoni/kanso/commit/ff34ddb4362f63af7587305af86543574f423b88))
- **history:** Note history shows what changed since each version, with each section to copy ([6e9bfc1](https://github.com/iamngoni/kanso/commit/6e9bfc161b9cbb4799be6e1ce011cd00f7cd7e0e))
- **images:** Drag an image's edge to resize it, size it with ![alt|300], or use Resize image… from the palette ([d8a3b0b](https://github.com/iamngoni/kanso/commit/d8a3b0b0ae7b04eaef89b4419611b813c618f3fa))
- **related:** Keep each note's embedding and the words read in its pictures on this device ([80b92c2](https://github.com/iamngoni/kanso/commit/80b92c2469a0a48252a410998e3aa8944d8b0fdd))
- **mcp:** Kanso-related-notes finds the notes closest in meaning to a note ([6a95ada](https://github.com/iamngoni/kanso/commit/6a95adaa7e83dcd6c80c46b89ad198ee5dcd9924))
- **graph:** See how notes link, around the open note in the inspector and across the whole library ([ef2cac7](https://github.com/iamngoni/kanso/commit/ef2cac740ee39b55f3213a572d76259a763387a4))
- **ocr:** Search finds notes by the words in their pictures, read with Vision on macOS ([1aaf62e](https://github.com/iamngoni/kanso/commit/1aaf62e3bb7abb4af28e06718c280a37aebd093e))

### Improved

- **app:** Typing never waits on the outline ([d95c0a0](https://github.com/iamngoni/kanso/commit/d95c0a06e73e743ade9627326f6de6458a880111))

### Fixed

- **app:** Plainer Local API and import wording ([76d2e0c](https://github.com/iamngoni/kanso/commit/76d2e0c6eba450fefca6ee37ddd30de5a8b63dc4))
- **release:** Build the Windows installer for dev versions too ([7b0fe29](https://github.com/iamngoni/kanso/commit/7b0fe29cb4e5e2f193f52fe0447fa6188ca59131))
- **render:** Draw Mermaid arrowheads smaller so they sit lightly on thin lines and clear the nodes ([58924a5](https://github.com/iamngoni/kanso/commit/58924a5e7f13b7b9d47dad967e008d80076c33ca))
- **preview:** Keep a drawing's actions in a quiet row under it that fades in on hover, never over the drawing ([f78cf44](https://github.com/iamngoni/kanso/commit/f78cf449c6f4aa8bee9dd2bc7dcbff74fb51cb9d))
- **export:** Export as PDF prints real pages on your default paper size, with margins ([c1de524](https://github.com/iamngoni/kanso/commit/c1de524adc81c3cc89246314e64e7ae44103cac2))
- **preview:** Text the Markdown parser can't handle shows as plain text instead of crashing ([77de02e](https://github.com/iamngoni/kanso/commit/77de02e393a94a29165553879a9546547ee43192))
- **cloud:** Retire unpublished links for good, serve only the checked upload, and let the owner take a note down ([ef1ef50](https://github.com/iamngoni/kanso/commit/ef1ef50d89bb1000d4154221816b3bc70a1be070))
- **sync:** Publish under a new link when the old one has expired or was retired, never reviving it ([fc31799](https://github.com/iamngoni/kanso/commit/fc317992986b9bf40027a718c7e6647ea6eb84dd))
- **api:** Keep a note's published link when the local API or MCP writes the note ([03563d8](https://github.com/iamngoni/kanso/commit/03563d8889ea63f3c84ed4c5663f0026424b712e))
- **publish:** Say your devices, not your Macs, in the Publish sheet ([225d09e](https://github.com/iamngoni/kanso/commit/225d09eba7c1439705e18c45ac18e27988cca1ee))
- **sync:** A Journal, Inbox or Templates notebook made on two devices offline is one notebook, not a conflict ([52d0a21](https://github.com/iamngoni/kanso/commit/52d0a21e35901a367d5e16540d0f8275275e86e4))
- **capture:** Quick notes are written in the editor's font and size from Settings ([ade79a3](https://github.com/iamngoni/kanso/commit/ade79a3dbdbce67252296674580bbd27f945710d))
- **app:** A new note made under a nested tag gets that tag ([b7803c0](https://github.com/iamngoni/kanso/commit/b7803c0af3f429c8800d48aaff1472bf175f8715))
- **reminders:** Reminders only reach the notification centre from an app bundle, and say when notifications are off ([185cb86](https://github.com/iamngoni/kanso/commit/185cb86e0bb9e086e30159d14910e859104476e5))
- **reminders:** Build cleanly on Windows and Linux, where the bundle check isn't needed ([f2f21f4](https://github.com/iamngoni/kanso/commit/f2f21f450fac76b56d88d195512e0d6320797209))
- **export:** Wiki-links in exported and published pages show as their label, and ![alt|300] images keep their width ([85f185e](https://github.com/iamngoni/kanso/commit/85f185e03dbf219002d00e2a6c325d7d706e1649))
- Say device instead of Mac on Windows and Linux, name each platform's secret store, and show Ctrl shortcuts off macOS ([13f3310](https://github.com/iamngoni/kanso/commit/13f33100387c6ccb6afd79839f87a7c459c08ee9))
- **related:** Say the same thing about the model download in the inspector and Settings ([29e496a](https://github.com/iamngoni/kanso/commit/29e496a64841f1345adfe057fd114ba7e833324f))

### Under the hood

- **release:** Publish each release automatically once every check passes ([9a01c36](https://github.com/iamngoni/kanso/commit/9a01c362e65336c3943af8baf09a32e74f05cce8))
- Describe Kanso on its own terms in the README, changelog and themes guide ([f54d468](https://github.com/iamngoni/kanso/commit/f54d46805699469586f30cef1d7f158b1eac2016))
- **readme:** Releases publish themselves once every check passes ([9a7f813](https://github.com/iamngoni/kanso/commit/9a7f813ec63c3330155e160a96302953940b1050))
- **readme:** The Mac download is notarized and updates itself ([45d8b8e](https://github.com/iamngoni/kanso/commit/45d8b8efa1a289e503d6f93377ae88b1f6125c5d))
- **app:** Measure typing latency with the outline open ([cb1bace](https://github.com/iamngoni/kanso/commit/cb1bacefb99d8b0a5b4c06c50ef4543f3916be36))
- **core:** Excerpt properties allow dots and brackets a note itself has ([618072b](https://github.com/iamngoni/kanso/commit/618072b9d757bd3be74962a8897cf61736ba4ed8))
- **app:** Snapshots date the notes they make by the demo clock ([437fb72](https://github.com/iamngoni/kanso/commit/437fb72cd09f6dbae676dc564d6e01167b1ededa))
- **e2e:** Give each end-to-end run its own Worker port so parallel runs never share one ([67c70c1](https://github.com/iamngoni/kanso/commit/67c70c1b2545f7cb6fb4515b73dc59b97336f774))
- **editor:** Snapshots of each Vim mode, and the typing benchmark with Vim keys on ([5771afc](https://github.com/iamngoni/kanso/commit/5771afc0d78cb634b6f48b7040c75291cafcadfc))
- **editor:** Vim snapshots show the caret as it is half a blink later ([30b016b](https://github.com/iamngoni/kanso/commit/30b016b19f14861025324408c6030471575d1992))
- **app:** Snapshots of the Tasks view, due chips, the replace sheet and the history changes ([436eeb2](https://github.com/iamngoni/kanso/commit/436eeb2499f2957b33f642c82de0ac40331c82f9))
- **editor:** Due dates stay tinted and @ completes dates with Vim keys on ([49b5243](https://github.com/iamngoni/kanso/commit/49b524353a1b046609663df6b77fcc29d07dbafd))
- Describe Kanso as it is now, with its newer shortcuts, kanso capture and device-neutral sync wording ([bff32fe](https://github.com/iamngoni/kanso/commit/bff32fe9adff9157a68681baa6127f0dd5fd8525))
- **changelog:** Release notes describe Kanso on its own terms ([c7be1b2](https://github.com/iamngoni/kanso/commit/c7be1b27da5fb98315d6756824dd95082611e174))

## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.1...v0.2.0) - 2026-10-19

### New

- Close brackets, quotes and Markdown markers as you type ([4f383a9](https://github.com/iamngoni/kanso/commit/4f383a991bd116220cfe1848cb9345da67a87631))
- Tab and Shift-Tab indent list items, with what hangs under them ([6ea7aac](https://github.com/iamngoni/kanso/commit/6ea7aacf8eaa3624829e83373c36e91833525bf7))
- Format Table lines up a Markdown table ([3945fee](https://github.com/iamngoni/kanso/commit/3945fee951b59353366cbc4081e81f4e7cf18c6b))
- Smart paste: links over selected words, formatted text as Markdown, images as attachments ([0b023f6](https://github.com/iamngoni/kanso/commit/0b023f689f86e5dc15dc93744f054be2603723cc))
- Size attached images in the preview ([a1a7dfd](https://github.com/iamngoni/kanso/commit/a1a7dfdab76e0980cb2717da7b68edbd463ade5d))
- Word goal with a progress ring in focus mode ([8ee7204](https://github.com/iamngoni/kanso/commit/8ee7204287b572a1eafcd3b1b34db203217886e5))
- Typewriter scrolling, and the writing aids in Settings › Editor ([18e0836](https://github.com/iamngoni/kanso/commit/18e08363e197903d385588f2e0acc4b902173eef))

### Fixed

- Kanso comes to the front after Restart now installs an update, instead of opening behind other windows ([7982fed](https://github.com/iamngoni/kanso/commit/7982fed))

### Under the hood

- Snapshots and latency runs for the writing aids ([d3c7467](https://github.com/iamngoni/kanso/commit/d3c7467ec807eb709fd7a867e51f984a00f79a27))

## [0.1.1](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.1.1) - 2026-10-19

### Fixed

- **release:** Brace variables before an ellipsis in release.sh ([6f819da](https://github.com/iamngoni/kanso/commit/6f819da6a4ae2585478dad4c27d0eceb4c308ab8))

## [0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0) - 2026-10-19

The first release: a calm, local-first Markdown notes app built on GPUI
for macOS, Windows and Linux. What follows sums up
the history so far; every commit is in [the history up to
v0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0).

### New

- Kanso updates itself from signed releases, says clearly when an update is ready, and shows what's new after it installs ([6bb7570](https://github.com/iamngoni/kanso/commit/6bb7570))
- Notes, notebooks within notebooks, tags and statuses in a SQLite store with trigram search, revisions and daily backups ([169d0e4](https://github.com/iamngoni/kanso/commit/169d0e4))
- Import a PouchDB notes library, inside the app or from the command line, keeping Kanso's own edits on a re-import ([e34c7bc](https://github.com/iamngoni/kanso/commit/e34c7bc), [e5479c4](https://github.com/iamngoni/kanso/commit/e5479c4))
- The editor with a live preview, side by side or alone, a draggable split and focus mode ([a531b38](https://github.com/iamngoni/kanso/commit/a531b38), [d0e68c2](https://github.com/iamngoni/kanso/commit/d0e68c2))
- The note list grouped by day, with search matches shown in the excerpts ([d7744b9](https://github.com/iamngoni/kanso/commit/d7744b9))
- The command palette and Open Note ([d268159](https://github.com/iamngoni/kanso/commit/d268159))
- Backlinks and each note's history, with a way back to any earlier version ([4b45548](https://github.com/iamngoni/kanso/commit/4b45548))
- Export a note, or every note, as Markdown ([0e35966](https://github.com/iamngoni/kanso/commit/0e35966))
- A local API and a native MCP endpoint for agents and other clients, compatible with popular local-server clients ([88ba474](https://github.com/iamngoni/kanso/commit/88ba474), [d5fd4d5](https://github.com/iamngoni/kanso/commit/d5fd4d5))
- Accounts with end-to-end encrypted sync between your computers, recovery keys, and a Settings page for the account and its devices ([f2d5a5a](https://github.com/iamngoni/kanso/commit/f2d5a5a), [666c77f](https://github.com/iamngoni/kanso/commit/666c77f), [b5fbac9](https://github.com/iamngoni/kanso/commit/b5fbac9))
- Settings inside the main window, and opening at login ([157cd61](https://github.com/iamngoni/kanso/commit/157cd61))
- Twenty themes, separate UI, syntax and preview themes, and importing theme packages and Zed themes ([1f9d3ff](https://github.com/iamngoni/kanso/commit/1f9d3ff), [8f050bc](https://github.com/iamngoni/kanso/commit/8f050bc))
- Code blocks colored in 23 languages, with a copy button ([3709232](https://github.com/iamngoni/kanso/commit/3709232), [27f54a0](https://github.com/iamngoni/kanso/commit/27f54a0))
- An app icon: an ink ensō with a vermilion seal ([c403439](https://github.com/iamngoni/kanso/commit/c403439))
- Builds for Windows and Linux as well as macOS ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))

### Improved

- A warm graphite and paper look and a calmer writing surface ([2e1b76b](https://github.com/iamngoni/kanso/commit/2e1b76b))
- Typing stays fast in long notes ([e0d7ad0](https://github.com/iamngoni/kanso/commit/e0d7ad0))

### Fixed

- Typed text is never lost, through saves, syncs and a note's revision moving underneath it ([f6a6583](https://github.com/iamngoni/kanso/commit/f6a6583), [9442143](https://github.com/iamngoni/kanso/commit/9442143))
- Sync keeps local edits through its own echoes, lost answers and restores ([7b0d30d](https://github.com/iamngoni/kanso/commit/7b0d30d), [652b439](https://github.com/iamngoni/kanso/commit/652b439), [3ab3f29](https://github.com/iamngoni/kanso/commit/3ab3f29))
- Recovery keys are kept until they're saved, and replaced with every password change ([645c5cd](https://github.com/iamngoni/kanso/commit/645c5cd), [ff8f89f](https://github.com/iamngoni/kanso/commit/ff8f89f))
- Auth keys, proofs and tokens stay out of logs and freed memory ([3bda3bd](https://github.com/iamngoni/kanso/commit/3bda3bd))
- Three rounds of review findings in the main window ([61f005e](https://github.com/iamngoni/kanso/commit/61f005e), [f325bfc](https://github.com/iamngoni/kanso/commit/f325bfc), [18b218d](https://github.com/iamngoni/kanso/commit/18b218d))

### Under the hood

- Property tests with a pinned seed across the workspace, and PNG snapshots of the real views ([ff9515f](https://github.com/iamngoni/kanso/commit/ff9515f))
- A release workflow that builds every platform into a draft release ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))
