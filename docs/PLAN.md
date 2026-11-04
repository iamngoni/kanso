# Kanso plan

Kanso is a local-first markdown notes app. It has a GPUI desktop UI,
serves a widely used local-server REST API on `127.0.0.1:19840` so existing MCP setups keep
working, imports a PouchDB notes library losslessly, and syncs end-to-end encrypted through Cloudflare.

## Decisions

| Topic | Decision |
|---|---|
| UI | GPUI through `gpui-kit = "=0.7.3"` (gpui-component, `EditorState` for markdown source, base `TextView` for preview). Pinned exactly; upgrade on purpose. |
| API | Local-server compatible REST on `127.0.0.1:19840` with Basic auth. The reference MCP server (below) runs against it unchanged. |
| MCP | Native `/mcp` endpoint on the same port with the same 17 tool names and params, added after the REST API passes contract tests. |
| Credentials | Port, username and password are copied from the source app's `config.json` at import time and kept in Kanso's own config. Never logged. |
| Storage | rusqlite (bundled), WAL. Each document stored as verbatim JSON in the API's shape with derived index columns. |
| Search | FTS5 `trigram` over title and body, the same design the source app uses. |
| Sync | Accounts with end-to-end encryption on a Cloudflare Worker + D1 (accounts, sessions, sealed documents) + R2 (sealed attachment bytes). See "Sync" and [ACCOUNTS.md](ACCOUNTS.md). |
| Server placement | In the app process; the app stays alive with no window open. `kanso serve` runs headless. An `flock` on the data directory decides who owns the database and the port. |

## Workspace

```
crates/
  kanso-core     domain types, IDs, revs, search-query parser
  kanso-store    SQLite store, FTS, revisions, backups, sync state
  kanso-import   PouchDB library importer; Markdown folders and other apps' exports (library/)
  kanso-server   axum REST API (+ /mcp later)
  kanso-sync     accounts, encrypted sync client, `kanso-sync` command
  kanso-app      GPUI app, binary `kanso`
cloud/           Cloudflare Worker (D1 + R2) and admin script
```

## Data model

Documents keep the source library's IDs and shapes so note links and attachment embeds written
with the legacy note-link scheme inside note bodies keep working, and API clients see what they expect.

- `note:<8 chars>`: `title, body, bookId, tags[], status (none|active|onHold|completed|dropped),
  share (private|public), pinned, doctype=markdown, createdAt, updatedAt, timestamp,
  numOfTasks, numOfCheckedTasks`, plus any unknown fields (e.g. `sourceTemplateId`), kept as-is.
- `book:<id>`: `name, parentBookId, order, createdAt, updatedAt`. `count` is derived.
- `tag:<id>`: `name, color, createdAt, updatedAt`. `count` is derived.
- `file:<id>`: `name, contentType, contentLength, publicIn[], md5digest, createdAt` and the bytes.
- `trash` is a pseudo-book ID. `book:inbox` style non-random IDs exist.
- Derived fields (`count`, `numOfTasks`, `numOfCheckedTasks`) are stripped on write and recomputed.

Store tables:

- `docs(id PK, type, rev, seq UNIQUE, deleted, json, synced_rev)` with generated, indexed columns
  for `book_id, status, updated_at, created_at, title, pinned`.
- `note_tags(note_id, tag_id)`, filled in the same transaction as the note.
- `blobs(file_id PK, md5, bytes)`.
- `revisions(doc_id, rev, json, at)`: one snapshot per write, autosave capped at one per 5 minutes
  per note. This is the undo net when an agent or sync overwrites a note.
- `notes_fts`: FTS5 `tokenize='trigram'` over title and body, excluding trash.
- Daily `VACUUM INTO` backups with rotation.

### Revs and conflicts

- `_rev` is `"{n}-{md5 of canonical json}"`. Only the current rev is kept (plus snapshots).
- `POST /notes`: no `_id` creates; unknown `_id` creates with that ID; existing `_id` with a
  missing or stale `_rev` returns 409.
- `POST /book:<id>` and `/tag:<id>`: partial merge; 409 only when a given `_rev` is stale.
- `GET /<id>?rev=`: the current rev returns the doc, any other rev returns 404.
- Every write runs in `BEGIN IMMEDIATE`.
- GUI: an external change to a note with a clean buffer reloads and keeps the cursor; with a
  dirty buffer, the buffer wins, the other version is snapshotted, and a banner says so.

## HTTP API contract

What `@inkdropapp/mcp-server` 1.5.0 calls (must work exactly):

| Call | Notes |
|---|---|
| `GET /{note,book,tag,file}:<id>` | `?rev=`, `?attachments=true` (files return `_attachments.index.data` base64) |
| `GET /notes?keyword&limit&sort&descending` | `sort`: updatedAt, createdAt, title. Default limit 50. Full `body` in results. Trash excluded. |
| `GET /books`, `GET /tags` | Full arrays |
| `POST /notes` | Create or full-document update (`_id` + `_rev`) |
| `POST /books`, `POST /tags`, `POST /files` | Create. Files carry `_attachments.index.{content_type,data}` |
| `POST /{book,tag}:<id>` | Partial merge, `_rev` optional |

All POSTs answer `{ok, id, rev}`. Errors are non-2xx. Request bodies up to 16 MB.

Documented but unused in 9 months of logs (later): `skip`, `sort=tagCount|rank`, `DELETE`,
`GET /`, `GET /_changes`, `/note/<id>` aliases.

### Search syntax

Terms and quoted phrases match title or body as case-insensitive substrings (trigram phrase
queries; terms under 3 characters fall back to an `instr` scan). Qualifiers: `book:Name`,
`book:"Two Words"`, `bookId:<id without prefix>`, `tag:Name`, `tagId:tag:<id>`, `status:onHold`,
`title:"..."`, `body:word`, and `-` to negate any of them. Backlinks search for the note's ID
link. Priorities from 1,629 real queries: quoted phrases (49%), `title:` (39%),
`bookId:` (33%), `book:` (5%), everything else under 1%.

## Importer

- Reads a consistent snapshot of `<hash>.sqlite` (with its `-wal`), so the source app may keep
  running. Never writes to the source app's folder. Ignores the stale LevelDB folders, `mrview-*` and `fts.db`.
- Reads the winning, non-deleted revision of every `note:`, `book:`, `tag:` and `file:` doc
  (`document-store.winningseq` → `by-sequence`).
- Attachments live in `attach-store.body` as PouchDB binary strings (one char per byte).
  Decoding is verified against `_attachments.index.digest` (`md5-<base64>`) for every file.
- Skips `_design/*`, `config` and `_local` docs. Keeps `activity:` docs raw in an archive table.
- Idempotent (upsert by ID) and ends with a verification report against the source counts.
- Copies the server port and credentials from the source app's `config.json`.

## Other sources (`kanso-import/src/library`)

- Each format is a `Source` that streams `SourceNote`s (title, Markdown body, folder path, tags,
  aliases, dates, pinned/trashed, and attachments carried inside the note for Evernote). Folders and
  zips are read through `tree.rs` (a folder, a zip, Notion's zip of zips, or one file); nothing is
  ever written to the source.
- The engine (`library/mod.rs`) reads the notes twice: once for an index of keys, paths, titles and
  aliases (`resolve.rs`), once to write. Links are pointed at what the import makes: wiki-links to
  the note's Kanso title, Markdown links to notes as ID links, pictures and files as attachment
  links (both in the legacy note-link scheme). Attachments are read last, only those a note links to, each capped at
  10 MB; unlinked files are reported as skipped.
- IDs come from a SHA-1 of the source kind, the source's name and a stable key (a vault-relative
  path, or the app's own ID), so a second import finds the first one's documents. Every document
  carries `kansoImport: {source, key, digest, generation}`; it is left alone when the digest
  matches, updated through `Store::import_doc` with the next generation when its revision is still
  the generation the import wrote, and kept when its generation is higher (edited in Kanso) or it
  was deleted. Tags are matched by name, ignoring case.
- A preview runs the same code with nothing written, so the summary before an import is what the
  import does (attachments are compared by size, not read).

## Sync (Cloudflare)

The full design, protocol and rollout are in [ACCOUNTS.md](ACCOUNTS.md). In short:

- **Accounts.** A username and a password (at least 12 characters). Argon2id turns the password
  into an auth key, which the server stores only as a peppered hash, and a key that unwraps the
  account's data key. A recovery key, shown once, can set a new password; Kanso can't reset one.
  Each Mac signs in as a device with its own session; the Account page lists and signs out devices.
- **Encryption.** Every document and attachment is sealed (XChaCha20-Poly1305) under keys derived
  from the data key. IDs (apart from their kind) and revisions go over the wire as HMACs, so the
  server never sees a note, a title, an ID or a revision. It sees each document's kind, sizes,
  counts, times, and device IDs with when each was last seen.
- **Worker** (TypeScript, wrangler): `/v2/auth/*`, `/v2/account*`, `/v2/push`, `/v2/changes`,
  `/v2/files/<wire_id>`. D1 holds accounts, sessions, sealed documents, quotas and rate-limit
  counters; R2 holds sealed file bytes per account. Quotas, daily budgets and sign-up rules are
  enforced in D1 triggers and counting upserts, so parallel requests can't race past them.
- **Push and pull** are as before, over sealed values: every local doc whose `rev != synced_rev` is
  pushed with its `base_rev`; the server accepts when it matches, else returns its version; pulls
  page through the account's feed. Revision generations only grow, so a stale or broken server copy
  is rebased or repaired rather than lost.
- **Conflicts**: last writer by `updatedAt` wins; the losing version goes into `revisions` and the
  GUI flags the note.
- **Limits** (defaults, changed with `npm run -s admin`): 50 accounts, 50,000 documents and 100 MiB
  of documents, 10,000 files and 1 GiB of files per account; per account per UTC day 20,000
  writes and 1 GiB of uploads; server-wide per day 300,000 writes and 5 GB of uploads. Past a daily
  budget sync pauses until UTC midnight.
- The deploy token comes from the `IMNGONII_CLOUDFLARE_API_TOKEN` environment variable
  (exported in `~/.zshrc`), passed to wrangler as `CLOUDFLARE_API_TOKEN`. It is never written to the repo.

## Milestones

| | Scope | Done when |
|---|---|---|
| M0 | GPUI editor spike (timebox 2 days) | The go/no-go below passes |
| M1 | core, store, importer, revisions, backups | The real DB imports with matching counts and all attachments verified |
| M2 | REST API, search parser | The real reference MCP server passes all 17 tools against Kanso; parser property tests pass |
| M3 | GPUI app MVP | Sidebar (notebooks, tags, statuses, trash), note list, editor + preview, search, pickers, links, images, autosave, external-change banner |
| M4 | Cloudflare sync | Two machines converge; conflicts keep both versions |
| M5 | Native `/mcp` endpoint | Same 17 tools, outputs match the Node server |
| M6 | Polish | Backlinks panel, Markdown export, settings, launch at login, revisions UI |

### Status (2026-10-17)

All milestones are built and tested.

- M0: the automated criteria pass (load, edit latency, preview). IME, press-and-hold accents, emoji
  and undo grouping still need a hands-on check.
- M1: the real library imported (823 notes, 43 notebooks, 28 tags, 15 attachments, all verified);
  a second import changes nothing.
- M2 and M5: the real reference MCP server runs all 17 tools against Kanso, and `/mcp` gives the
  same results (`crates/kanso-server/tests/contract.rs`).
- M4: deployed to Cloudflare and checked end to end between two data folders.
- The source library's stored task counts were stale on 21 notes; Kanso recounts from the body like GFM does.
- The source app's API credentials are encrypted with Electron safeStorage, so copying them needs
  its "Safe Storage" Keychain item, which macOS asks the user to allow.
- A code review of the app found ways to lose typed text (switching notes or quitting while the
  "changed elsewhere" question was open, ⌘Q, a note deleted elsewhere). All are fixed, and a
  model-based test (`prop_typed_text_is_never_lost`) runs random sequences of typing, outside
  edits and deletions, switching, answers and quitting against the real views.

### UI redesign (2026-10-18)

The app now follows the design spec and mockups from May 2026 (macOS desktop spec; mockups 01 sync
settings, 05 command palette, 06 split editor, 07 empty notebook):

- Warm graphite (dark) and paper (light) themes from the spec's palette, SF Pro body text at 16 px,
  SF Mono in split mode, and restrained Markdown highlighting.
- A transparent title bar with shared 52 pt top and 36 pt bottom bands across the panes. The global
  status bar is gone: sync and the local API show in the sidebar footer, and cursor and word counts
  in the editor footer.
- Sidebar: All Notes, Pinned, Recent (last 7 days), notebooks, `#tag` chips, a Status group and
  Trash. ⌘\ folds it to an icon rail.
- Note list grouped by day, with status glyphs, two-line excerpts (`kanso_core::excerpt`), match
  highlighting, one filter/sort menu and the empty-notebook block.
- Editor header with breadcrumb, Edit/Preview/Split, status and tag chips, a formatting toolbar,
  and a backlinks menu. Split has a draggable divider and loose scroll sync.
- Command palette (⌘K) and Open Note (⌘P) drawn inside the workspace so commands reach its handlers.
- Focus mode (⇧⌘D) and a first-run welcome.
- Settings moved into the main window (General, Editor, Appearance, Sync, Local API); the separate
  settings window is gone.

Left out on purpose, because Kanso has no such features yet: sketch blocks, skills, MCP
permissions, templates and a Tasks view. (Accounts and devices came with encrypted sync, below.) The UI is checked headlessly by an
ignored snapshot test that renders PNGs (`KANSO_SNAPSHOT_DIR=<dir> cargo test -p kanso-app
snapshots -- --ignored`, optionally with `KANSO_SNAPSHOT_DB` pointing at a copy of a library).

### Accounts and encrypted sync (2026-10-18)

Sync moved from one shared token to accounts with end-to-end encryption (ACCOUNTS.md); the
plaintext v1 engine and its Worker routes are gone, and `kanso-sync unbind` is the rollback path.

- Settings › Account: create an account, sign in, recover with the recovery key, change the
  password, a new recovery key, devices, sign out and delete. A Mac with notes is asked whether to
  add them or replace them (after a backup).
- Tested by the shared fixtures (the Worker and the in-memory server pass the same cases), flow
  tests that run the real sync in headless windows, and `cloud/scripts/e2e-local.sh`: two Macs
  against the Worker under `wrangler dev` at production KDF parameters, ending with a scan of the
  stopped Worker's files for canaries (with a positive control).
- Not deployed yet: the owner's deploy and migration follow ACCOUNTS.md › Rollout. Until then the
  deployed Worker and Kanso.app still run v1.

### M0 go/no-go

- The 253 KB note opens in under 200 ms.
- p95 edit-to-frame under 16 ms at the start, middle and end of that note, with soft wrap on.
- Smooth scrolling.
- IME composition, press-and-hold accents and emoji work.
- Undo groups sensibly.
- Reloading a note can keep the cursor and scroll position.
- The preview renders that note, including attached images, in under 300 ms.

If IME or latency fails, fork only the input module.

## Risks

- gpui-kit is young and pins a weekly GPUI snapshot (`gpui-pre =0.3.9`). Exact pins and a
  committed `Cargo.lock`.
- Another app serving the same API holds port 19840. Kanso shows "API off: port in use" with a retry and never
  silently moves to another port.
- No live server of the source app to compare search results against. Parser tests are built from
  the request history in its `server.log`.
- Testing: property tests (proptest) for the query parser, rev/conflict rules and attachment
  decoding; integration tests on temp databases; contract tests with the real MCP server.
