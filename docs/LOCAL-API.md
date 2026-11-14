# The local API

Kanso runs a REST API and a native MCP endpoint on your computer, for AI agents,
scripts and other apps. Both listen on `http://127.0.0.1:19840` (REST at the root,
MCP at `/mcp`). Turn them on or off, and change the address or port, in
Settings › Local API › Server.

**Every caller needs its own API key.** Kanso refuses anything without one, even a
program that can start Kanso. There is no username or password for the API any more
(see [Upgrading from the old login](#upgrading-from-the-old-login)).

Keys are made and kept in the app, in Settings › Local API › **API keys**. An app on
this computer, such as the Raycast extension, can also ask for a key itself, and you
allow it with one click; see [Pairing an app](#pairing-an-app). A server library
(`kanso serve`) has no Settings; see [Headless](#headless).

## Making a key

**New key…** opens a sheet:

| Field | Choices | Default |
|---|---|---|
| For | Claude Code, Claude Desktop, Codex, Cursor, VS Code, Gemini CLI, Windsurf, Zed, OpenCode, or **Script or other app** | |
| Name | free text (a key needs a name) | the choice's name and "on this Mac", for example "Codex on this Mac" |
| Can | **Read notes**, **Read and suggest edits** ("You review each edit before it's saved."), or **Read and write** ("Changes are saved right away."), or **Add and tick only** ("Can read, add notes and tasks, and tick tasks. It can't change or delete anything.") | Read notes |
| Notebooks | **All notebooks**, or **Choose…** (at least one) | All notebooks |

**Create key** replaces the sheet with the setup for that agent, with the key already
in it. **Copy** puts it on the clipboard as a concealed secret, and the toast says
"Setup for Codex copied." (or "Setup copied." for a script). **Kanso shows this key
only once**, so copy it now. Kanso keeps only a SHA-256 of the key, which can't be
turned back into it. If you lose a key, revoke it and make another.

A key looks like `kcl_` followed by 43 characters (47 in all). The prefix tells it
apart from the short-lived tokens Kanso gives its own agents, and makes a leaked key
easy to search for. Keys live in your library, never sync, and work on this computer
only. Restoring a backup never brings back the keys it held: the library keeps the keys
it has now, so a key you revoked stays revoked and one made after the backup keeps
working.

No setup is a terminal command: a key in a command's arguments would be in the process
list and the shell's history. Every agent's setup is an entry for its config file.

### The list

Each row shows the key's name, what it can do and which notebooks, and when it was
last used ("Used 2 min ago", "Never used"). **Revoke** asks "Revoke the key for
“Codex on this Mac”?" and "It stops working right away." A revoked key is refused on
its next request.

To change what a key may do, revoke it and make a new one. A key's access is never
widened in place.

When a request is refused, the group shows for 24 hours: "Something tried to reach
Kanso without a valid key at 14:05." It says nothing about the program, because that
could be faked.

## Pairing an app

An app on this computer can ask Kanso for a key, so nobody has to copy one. The
Raycast extension is the first to do it. Pairing only ever makes an **Add and tick
only** key (see [Add and tick only](#add-and-tick-only)), whatever the app asks for.

1. The app asks. Kanso comes to the front with a sheet:

   > **Raycast wants to connect**
   >
   > An app on this Mac calling itself “Raycast” asks for a key.
   >
   > Code **482 913**. Check that Raycast shows the same code.
   >
   > It will be able to: search and read notes · add notes and tasks · tick tasks.
   > It will not be able to change or delete existing text.
   >
   > Notebooks: All notebooks. Locked notes are never shown to a key.
   >
   > **Don't Allow** · **Allow**

2. The app shows the same six-digit code while it waits. Allow only when the codes
   match and you started the request yourself. The name on the sheet is what the app
   calls itself, and Kanso can't check it.
3. **Allow** makes the key and hands it to the app, once. **Don't Allow** tells the
   app no. A request nobody answers expires after 2 minutes. Escape closes the sheet
   without answering, and the request then expires.

If a second app asks while a request waits, the sheet warns "Two apps asked for a key.
Deny unless you just started this from Raycast." and **Don't Allow** becomes the main
button. One of the two may be a program hoping you allow it in a hurry.

### Paired apps

Settings › Local API lists paired apps in a group of their own, **Paired apps**, apart
from the keys you made. Each row reads "Raycast · Added 12 Nov · Last used 2 min ago",
or "Never used" until the app first calls Kanso. **Revoke** asks "Disconnect Raycast?",
with "It stops working right away. Run a Kanso command in Raycast to connect again."
A paired key is refused on its next request after that, like any revoked key.

### The pairing routes

These are the only routes that take no key, so they are narrow:

```
POST /pair            {"client": "raycast", "name": "Raycast", "access": "capture", "books": null, "secret": "<32 characters>"}
  -> 202 {"request": "<id>", "code": "482913", "expires": 120}

GET  /pair/<request>  with the header X-Kanso-Pair-Secret: <the same secret>
  -> 202 {"state": "waiting"}
   | 200 {"state": "allowed", "key": "kcl_…", "id": "<key id>"}
   | 403 {"state": "denied"}
   | 410 {"state": "expired"} or {"state": "gone"}
```

- `client` is 1 to 32 lowercase letters, digits or dashes. `name` is what the sheet
  shows: 1 to 60 characters, with no control or invisible characters. `secret` is 32
  base64url characters that the app makes and keeps; Kanso keeps only its SHA-256.
  `books` is a list of notebook IDs, or `null` for every notebook.
- `access` may be left out or be `"capture"`. Asking for more is refused with `403`:
  "Pairing can only make a key that adds notes and ticks tasks. Create a key with more
  access in Settings › Local API."
- The app polls `GET /pair/<request>` with its secret, about once a second. A wrong
  request ID or secret is `404`, the same as a request that doesn't exist.
- The key comes out once, to the first poll after Allow, with `Cache-Control:
  no-store`. A later poll gets `410` `gone`. A key nobody fetches within 30 seconds of
  Allow is revoked, and the poll gets `410` `expired`.
- **Only from this computer.** A caller that isn't `127.0.0.1` or `::1` is refused,
  even when **Address** is set to more: "Pairing works only from this computer. Create
  a key in Settings › Local API instead." The `Host` header must be `localhost`,
  `127.0.0.1` or `[::1]`. A request with an `Origin` header is refused, "Pairing is for
  apps, not web pages.", so a web page can't pair.
- **One request at a time.** While one waits, another is `429`, "A request is already
  waiting in Kanso." At most 5 requests are taken in a minute; the next is `429`, "Too
  many pairing requests. Wait a minute and try again."

Nothing about pairing is logged: not the key, the secret or the request ID.

## What a key may do

| Access | MCP tools | REST |
|---|---|---|
| Read notes | the read tools | `GET` and `HEAD` only |
| Read and suggest edits | the read tools, and `patch-note`, `update-note` and `create-note`, which only stage a change for you to review (**Review** in the Ask panel; nothing is written until you press **Apply**) | `GET` and `HEAD` only |
| Add and tick only | the read tools, and three that add, saved at once: `create-note` (a new note only: no `_id`, no `_rev`, not the trash or Templates), `kanso-add-task` (appends one task) and `kanso-set-task-done` with `done: true`. A tool added later is refused until it is decided | `GET` and `HEAD`, and `POST /notes` with a new note. Every other route is `403` |
| Read and write | every tool | every route |

Add-and-tick keys can't edit, move, untick or delete anything that is already written.
Read, suggest and add-and-tick keys have no access to attachments. A key that can write sees an
attachment only when it sees every note that holds it. A key is never given local
file paths: `create-file` with `filePath` and `read-file` with `outputPath` are
refused.

What no key sees, whatever its access, because Kanso answers "not found":

- locked notes, and every note in a locked notebook
- notes in the trash
- notes tagged `#ai-exclude`
- anything outside the key's notebooks

Notebooks and tags show a count of the notes that key can see, not the library's.
Publish, share and lock fields are left out of every document. A key limited to some
notebooks writes only into them, and no key renames or deletes `#ai-exclude`.

### Add and tick only

This access is called `capture` in code and on the command line
(`kanso api-key create --access capture`), and it is the only one pairing makes. It
reads like a Read key, with fewer tools, and it adds:

| MCP tool | REST | What it does |
|---|---|---|
| `create-note` | `POST /notes` | A new note, saved at once. It may set only `bookId`, `title`, `body`, `status` and `tags`. Any other field, `_id` and `_rev` included, is `403`: "This key can only add a new note, with a notebook, title, text, status and tags. It can't set `…`, or change a note that is there." The trash and the Templates notebook are refused |
| `kanso-add-task` | none | Adds one task line at the end of a note. Takes `text` (one line), and `noteId` or `target`, not both: `daily` is today's daily note, made when missing, and the default; `inbox` is the Inbox's task note, titled Inbox tasks (`note:inbox-tasks`, the note Quick Capture's Task mode uses), made when missing. A note titled Tasks in the Inbox, where older versions put these, has its lines moved to the end of it and goes to the trash the next time a task goes there. `dateOrder` is `dmy` (the default) or `mdy`, for dates written with a slash. Returns `noteId`, `line` (0-based), `task` (the line written), `due`, `repeat`, `label` and `uri` |
| `kanso-parse-when` | none | Reads the date, time and repeat at the end of a line of task text, as `kanso-add-task` would, and writes nothing. Returns `text` (the words before the date), `due`, `repeat` and `label`; with no date, the text and three `null`s |
| `kanso-set-task-done` | none | Ticks a task, given `noteId`, `line` and its exact `text`. Only with `done: true`: clearing a checkbox is `403`, "This key can tick a task but not clear its checkbox. Pass `done: true`." Ticking a repeating task also writes the next one above it |

`kanso-add-task` turns the date and repeat typed at the end of the text into Kanso's
own tokens, with the same reader as the app (the words it knows are on the
[help page](https://kansoapp.app/help#tasks)). This call:

```json
{"text": "film Pixel review fri 3pm every week", "target": "daily"}
```

writes `- [ ] film Pixel review @2026-11-14T15:00 every week` and returns, with the
rest, `"label": "Fri 14 Nov, 15:00 · every week"`. Text that doesn't end in a date is
written as it is. A repeat needs a date before it, so `water plants every week` is
plain text. These are refused:

- a text that is only a date: "Say what the task is, as well as when: the text is only
  a date."
- a text with a line break: "A task is one line: send each task in its own call."
- a locked note, or one outside the key's notebooks: not found.

A Capture key reads with `read-note`, `search-notes`, `list-notes`, `list-notebooks`,
`list-templates` and the `kanso-` read tools: tasks, outlines, properties, backlinks,
links, mentions, open loops, tags, versions, related notes, query views, canvases, sync
status and the Ask tools. It doesn't have `read-backlinks`, `list-tags`, `read-tag` or
`read-book`, or anything to do with attachments, files or PDFs. On REST it has `GET`
and `HEAD` on the note, notebook and tag routes, and `POST /notes`; every other write is
`403`. A tool or route added to Kanso later is refused to Capture keys until someone
decides what Capture may do with it.

## Canvases

A canvas is a note of type `canvas`. The general note routes and tools treat it as a note,
with the differences below. Four tools and routes are only for canvases.

- **Lists and searches** (`GET /notes`, `search-notes`, `list-notes`) show a short Markdown
  summary of the canvas in place of its JSON: its text cards, card titles, links and
  recognised handwriting, up to about 4,000 characters. They also show `size`, the body's
  length in bytes. `GET /notes?doctype=canvas` lists only canvases, and `?doctype=markdown`
  only Markdown notes.
- **`GET /note:<id>`** and `read-note` return the stored note, whose `body` is the `canvas`
  fence, and add `canvasJson`, the JSON inside the fence. `canvasJson` is read-only: a client
  that reads a note and writes it back whole doesn't store it.
- **Writes** are validated as canvases. A body that isn't one canvas, or is over the 1 MB
  budget, is refused. A shared canvas's body can't be written at all: use `kanso-edit-canvas`.
- **Locked canvases**, and canvases in a locked notebook, in the trash, tagged
  `#ai-exclude` or outside the key's notebooks, are not found. They aren't listed, read or
  edited, and they add nothing to a list or a search.

| MCP tool | REST | Access | What it does |
|---|---|---|---|
| `kanso-list-canvases` | `GET /notes?doctype=canvas` | every key | Canvases only, each with its title, notebook, tags, size and summary. Takes the same `bookId`, `tagIds`, `keyword`, `sort`, `descending` and `limit` as `list-notes` |
| `kanso-read-canvas` | `GET /note:<id>/canvas[?group=<id>]` | every key | The canvas as JSON (live items only, each with its `kansoVersion`) and the note's `_rev`. With a group, only what lies inside that group's rectangle |
| `kanso-edit-canvas` | `PATCH /note:<id>/canvas` | Read and write | A batch of operations, applied all or none. Answers the new `rev` and the IDs the batch created |
| `kanso-create-canvas` | `POST /notes` with `doctype: "canvas"` and the fence as `body` | Read and write | A new canvas, empty or from a canvas document (`canvas`) |

A Read or Suggest key can list and read canvases but not change them: a suggestion can't
edit a canvas. A key limited to some notebooks must pass `bookId` to `kanso-create-canvas`.

## Embargo

A note can carry an `embargo` property: a date and time with a time zone
(`embargo: 2026-11-18T09:00 America/New_York`), a date and time, or a day. Until it lifts, `kanso-export-html` refuses
the note with a tool error that says when it lifts and how to go ahead: "This note is under
embargo until Wed 18 Nov, 09:00 New York (in 5 days). Exporting it would hand it out early.
To export it anyway, call kanso-export-html again with "ignoreEmbargo": true." Pass
`"ignoreEmbargo": true` only when the person asked for the note before its embargo date.
An embargo with a time zone lifts at the same moment everywhere; one without lifts at that
time on the computer's clock. Once it has lifted, the property changes nothing. Reading,
searching and editing the note are never affected.

## PDFs

PDF text tools read the saved per-page extraction, including OCR text. They do not read a
locked note, a note in a locked notebook, or a PDF also owned by a note the caller cannot
see. PDF writes update the note's `kanso-pdf` annotation block; the PDF bytes stay unchanged.

| MCP tool | REST | Access | What it does |
|---|---|---|---|
| `kanso-search-pdf` | `GET /pdf/search?query=<text>&limit=<1..100>` | agent token or Read and write key | Finds matching indexed pages in visible notes, with a short excerpt |
| `kanso-read-pdf-page` | `GET /notes/<noteId>/pdf/<fileId>/pages/<page>` | agent token or Read and write key | Reads one indexed page as text; page numbers start at 1 |
| `kanso-list-pdf-highlights` | `GET /notes/<noteId>/pdf/<fileId>/highlights` | agent token or Read and write key | Lists the highlights, comments and whether each quote is in the note |
| `kanso-add-pdf-highlight` | `POST /notes/<noteId>/pdf/<fileId>/highlights` | Read and write key | Takes `{ "page": 4, "quote": "…", "color": "yellow", "comment": "…", "includeQuote": true }`; resolves the quote through PDFium, adds its page link to the note by default, and saves one note revision. Set `includeQuote` to `false` to leave the note text alone. |
| `kanso-edit-pdf-comment` | `PATCH /notes/<noteId>/pdf/<fileId>/highlights/<highlightId>` | Read and write key | Takes `{ "comment": "…" }`; an empty string clears it |
| `kanso-edit-pdf-highlight` | same `PATCH` route | Read and write key | Takes `color`, `comment` or both to edit a highlight |
| `kanso-delete-pdf-highlight` | `DELETE /notes/<noteId>/pdf/<fileId>/highlights/<highlightId>` | Read and write key | Removes the annotation; a quote already copied into the note stays |

Read and Suggest API keys cannot see attachment-derived PDF text, matching their existing
attachment scope. Agent tokens can use the PDF read tools included in their scope; PDF
mutations require a Read and write key. When a shared note is edited, the running Kanso app
sends the body change through the live collaboration engine as one edit.

`kanso-edit-canvas` takes `{"ifRev": "…", "ops": [ … ]}`. The operations are `addNode`,
`updateNode`, `moveNode`, `resizeNode`, `removeNode`, `addEdge`, `updateEdge`, `removeEdge`,
`addStrokes`, `removeStrokes`, `moveStrokes`, `setGroup`, `renameGroup` and `rename`. Stroke
points are `[x, y, pressure]`, relative to their ink layer, and Kanso gives new strokes their
IDs. Each change is stamped as an edit in the app is.

- **Versions.** An operation on an existing item may carry `ifVersion`, its `kansoVersion`
  from `kanso-read-canvas`. The batch may carry `ifRev`, the note's `_rev`. If either is out of
  date, the batch is refused with `409` and the current `rev` and `versions`, so a caller never
  overwrites drawing it hasn't seen.
- **Refusals.** A refused batch changes nothing. An item that isn't there is `404`. Anything
  else the canvas can't hold is `400`, with the reason.
- **Limits.** A batch holds at most 500 operations and 4 MB of JSON. It adds at most 10,000
  strokes and 200,000 points, with at most 20,000 points in a stroke. A text card's text is at
  most 100,000 characters; a title, label or URL at most 4 KB. Coordinates and sizes are real
  numbers up to 10,000,000. New IDs are 1 to 64 letters, digits, dashes or underscores. The
  canvas after the batch must fit its 1 MB budget.
- **Cards.** A `file` card points at a note the key can see (`note:…`), or at a picture that
  is already on this canvas (`file:…`) and that the key can read, never one from another
  note or one sealed with a locked note. `kanso-create-canvas` refuses `file:` cards, since a
  new canvas has no pictures of its own.
- On a private canvas, a batch is one revision in the note's history, from the API.

A **shared** canvas is changed through Kanso's live room, so the Kanso app must be running and
signed in. `kanso serve` refuses edits to shared canvases. A batch reaches the people on the
canvas as one change, its new title included. A batch too big to send as one change (about
240 KB of changes) is refused: split it into smaller batches. A member who may only view the
canvas can't change it. Pictures (`file:` cards) can't be added to a shared canvas yet.

## Agent skill

`skills/kanso/` is an [Agent Skill](https://agentskills.io) that teaches a coding agent (Claude Code,
Codex, Cursor and others) to use Kanso's MCP tools well: how to connect, which tool does what, and how to
read before it writes. It does not hold a key; connect the agent first, as in
[An MCP client](#an-mcp-client). Install it with either:

```sh
npx skills add iamngoni/kanso
```

or copy the folder:

```sh
mkdir -p ~/.claude/skills && cp -R skills/kanso ~/.claude/skills/kanso
```

`skills/kanso/references/tools.md` lists every tool and its parameters; a test in `kanso-server`
fails when a tool is missing from it.

## Connecting a caller

### An MCP client

Paste the setup Kanso gives you into the agent's config file. It has one of these shapes:

```json
// Claude Code, in ~/.claude.json (quit Claude Code first; it rewrites that file)
{ "mcpServers": { "kanso": { "type": "http", "url": "http://127.0.0.1:19840/mcp", "headers": { "Authorization": "Bearer kcl_…" } } } }

// Gemini CLI, in ~/.gemini/settings.json
{ "mcpServers": { "kanso": { "httpUrl": "http://127.0.0.1:19840/mcp", "headers": { "Authorization": "Bearer kcl_…" } } } }

// Claude Desktop: starts `kanso mcp`, with the key in its environment
{ "mcpServers": { "kanso": { "command": "/Applications/Kanso.app/Contents/MacOS/kanso", "args": ["mcp"], "env": { "KANSO_MCP_TOKEN": "kcl_…" } } } }
```

```toml
# Codex, in ~/.codex/config.toml: starts `kanso mcp`, with the key in its environment
[mcp_servers.kanso]
command = "/Applications/Kanso.app/Contents/MacOS/kanso"
args = ["mcp"]

[mcp_servers.kanso.env]
KANSO_MCP_TOKEN = "kcl_…"
```

Cursor, VS Code, Windsurf, Zed and OpenCode get an entry in their config file with the
URL and an `Authorization: Bearer …` header. Kanso tells you which file to edit, and
never edits it for you. Restart the agent after you paste.

`kanso mcp` is a relay: it sends each message to the running app's `/mcp` with the
key from `KANSO_MCP_TOKEN` (the key is read from the environment, never from
arguments). It never opens the library or reads a secret, so Kanso must be open (or
`kanso serve` running) with the local API on. `KANSO_MCP_URL` may name the address,
but only plain `http://` to this computer (`127.0.0.1`, `[::1]` or `localhost`), so a
key can't be sent elsewhere. Without it the relay reads the address from
`config.json`, which holds no secrets.

### A script or another app

Choose **Script or other app**. The sheet shows the endpoint, `MCP: …/mcp`, and:

```
Authorization: Bearer kcl_…
```

```sh
curl -H "Authorization: Bearer kcl_…" "http://127.0.0.1:19840/notes?limit=5"
```

A tool that only accepts a username and password can send HTTP Basic with **any
username and the key as the password**. A key opens every route (REST and `/mcp`).

### Raycast

The Kanso extension for Raycast (`integrations/raycast`, for macOS and Windows) has four
commands, and a fifth to connect:

| Command | What it does |
|---|---|
| **Quick Capture** | Saves a note. The first line is the title and the rest is the note. **Notebook** starts at the Inbox. The toast, "Saved to Inbox", has **Open in Kanso** |
| **Add Task** | Adds one task. The line under the field shows how Kanso reads the date, such as "Fri 14 Nov, 15:00 · every week", or "No date or repeat. It will be saved as written." **Put it in** offers Today's daily note (the default), the Inbox task list (the Inbox tasks note, shared with Quick Capture) or a recent note. The toast says "Task added · Fri 14 Nov 15:00 · every week" |
| **Search Notes** | Searches your notes, with a preview. Enter opens the note in Kanso. **Copy Link** copies its `kanso://open/note/<id>` link and **Copy Title** its title. **Add Task Here** adds a task to a note that already has tasks |
| **Open Note** | Lists recently updated notes. Enter opens one in Kanso |
| **Connect to Kanso** | Pairs again, or **Paste a Key** to use a key made by hand |

The first time any command runs, it asks to connect: "Kanso must be open on this
computer. Raycast will ask Kanso for a key that can add notes and tasks, search, and
open notes." **Connect** starts [pairing](#pairing-an-app) and shows "Waiting for
Kanso…" with the code. Choose **Allow** in Kanso, and Raycast says "Connected to Kanso".
Raycast keeps the key in its own encrypted storage.

The extension's preferences are **Local API Port** (19840 unless you changed it) and
**Key (Optional)**: a `kcl_…` key made by hand under Settings › Local API › API keys,
which wins over a paired key. Make that key **Add and tick only**; Raycast needs no more.

| Raycast says | Fix |
|---|---|
| "Kanso isn't open. Open Kanso, then try again." | open Kanso (the **Open Kanso** action does it) |
| "Kanso's local API is off. Turn it on in Kanso › Settings › Local API." | turn the server on |
| "Something else answers on port …, and it isn't Kanso. Set the port shown in Kanso › Settings › Local API." | set **Local API Port** |
| "Kanso said no. Run any Kanso command to try again." | run a command and choose **Allow** |
| "Kanso didn't answer in time." | run a command and answer within 2 minutes |
| "A request is already waiting in Kanso" | if you didn't start it, choose **Don't Allow** in Kanso |
| "Kanso already handed over the key" | revoke the Raycast key in Settings › Local API, then connect again |
| "Kanso no longer knows this key. Connect again." | the key was revoked: run any command to pair again |

### Opening Kanso from a link

Kanso opens `kanso://open` links from Raycast, a script (`open "kanso://open/new"` on
macOS) or another app. They take no key, and they only open or show something: none
reads, changes or saves a note. The phone app reads the same links.

| Link | Opens |
|---|---|
| `kanso://open/note/<id>` | the note |
| `kanso://open/notebook/<id>` | the notebook's list |
| `kanso://open/search?q=<words>` | a search of every note |
| `kanso://open/new` | the Quick Capture window, empty |

A link starts Kanso when it isn't running. A link Kanso doesn't know does nothing.

### Kanso's own agents

The agents Kanso starts itself (Ask your notes with an agent, Agent tasks) don't use
keys. Kanso gives each one a short-lived token that ends with the question and opens
`/mcp` only.

### `kanso capture` and `kanso sticky`

These need no key. While Kanso is open they send the note over a private local link,
an endpoint only you can read, which can only add notes: it has no message that reads,
searches or lists. With Kanso closed, they write to the library directly.

## When a caller is refused

Each message says how to fix it, because a refused agent never shows Kanso's window.

| What happened | What the caller sees | Fix |
|---|---|---|
| `kanso mcp` started with no `KANSO_MCP_TOKEN` | error `-32001` on the first request, then exit status 2: "Kanso needs a key for this agent. In Kanso, open Settings › Local API › API keys, choose New key…, and paste the setup it gives you into this agent's settings." | make a key and paste the setup |
| Kanso answers 401 to the relay's key | error `-32001`: "Kanso doesn't recognise this key. It may have been revoked. Create a new one in Kanso › Settings › Local API › API keys." | make a new key |
| Any HTTP request without a valid key | `401`, with `WWW-Authenticate: Basic realm="Kanso"` and `{"error":"unauthorized","reason":"Kanso needs an API key. Create one in Settings › Local API › API keys."}` | send `Authorization: Bearer <key>` |
| A key that can't write tries to | `403` `forbidden` (REST), or a tool error (MCP): "This key can read your notes but not change them. In Kanso, open Settings › Local API › API keys and create a key that can read and write." (a suggest key says it "can suggest edits through MCP but not change notes directly"; an add-and-tick key says it "can add notes and tasks and tick tasks, but it can't change or delete what is already there") | make a key that can read and write |
| A key without attachment access asks for one | "This key can read but has no access to attachments. In Kanso, open Settings › Local API › API keys and create a key that can read and write." | make a key that can read and write |
| A key limited to some notebooks writes elsewhere | "This key is limited to some notebooks, and … is not one of them. In Kanso, open Settings › Local API › API keys and create a key for the notebooks you need." | make a key for those notebooks |
| `kanso-export-html` on a note whose embargo hasn't lifted | a tool error: "This note is under embargo until … (in …). Exporting it would hand it out early. To export it anyway, call kanso-export-html again with "ignoreEmbargo": true." | wait for the embargo, or pass `"ignoreEmbargo": true` if the person asked for it early |
| `kanso mcp` can't read `config.json` | error `-32000`: "Kanso's settings at … can't be read. Open Kanso once to repair them, then restart this agent." | open Kanso once |

A note, notebook or tag a key can't see is "not found" (404), the same as one that
doesn't exist.

## Upgrading from the old login

Kanso used to protect the local API with a username and password kept in
`config.json`. **That login is gone, and the old setups stopped working at the
update** (there is no grace period). On the first launch of the new version, Kanso
removes the username and password from `config.json` and remembers that you had one.

If you had one, Kanso shows an update notice, **Your AI agents and scripts need new
keys**, as a banner across the top of the window. To fix it:

1. Open Settings › Local API › API keys (the banner's **Open Settings** goes there).
2. Choose **New key…** for each agent or script, and pick what it may do.
3. In an MCP client, paste the setup Kanso gives you in place of the old Kanso entry,
   then restart the client.
4. In a REST tool or script, send the key as `Authorization: Bearer <key>` instead of
   the username and password. A tool that only takes a username and password can use
   any username, with the key as the password.

**Settings from another app** (Settings › Local API › Other apps, or
`kanso import pouchdb --credentials`) now takes only that app's port and address.
Clients you had set up against that app's server still need a Kanso key.

### Update notices

Any update that breaks something people rely on ships a notice you can dismiss, with
what changed, why, and numbered steps to fix it. Notices are compiled into the app,
never fetched. Kanso shows a notice only to people the change affects.

- A **banner** across the top of the window, until you dismiss it. **How to fix**
  opens the full notice, with **Open Settings** and **Mark as fixed**. The banner
  shows "1 of 2" when more than one is waiting.
- **Dismiss** hides the banner on this computer for good. The notice stays in
  What's new and in the palette.
- **Mark as fixed** hides the banner and drops the notice from the palette list. It
  stays in What's new.
- What's new opens with a **Needs your attention** section listing every notice for
  that release that applies to you, dismissed or fixed ones included.
- The command palette's **Update notices** lists every notice that applies and isn't
  marked fixed.
- On the phones, a notice that applies there is a card at the top of the notes list.

How a release is held to this is in [RELEASING.md](RELEASING.md#breaking-changes).

## Network and security

- Kanso listens on `127.0.0.1`, so only programs on this computer can reach it. If you
  set **Address** to anything else, the API is plain HTTP and **keys are sent
  unencrypted** on that network. Use an SSH tunnel or a TLS proxy instead.
- A key limited to some notebooks and to Read is the safest default. Each key's last
  use shows in Settings, and revoking takes effect on the next request.
- An agent's own config file holds its key in plain text: protect those files.
- A program that runs as you can still do what you can do. Keys stop an agent from
  reaching your notes by accident or without you knowing; they don't stop an attacker
  who already runs code as you. On the library file itself, see
  [Encryption on this computer](../README.md#encryption-on-this-computer).

## Headless

`kanso serve` runs the API without a window and takes keys like the app does. Keys are
stored in the library and never sync, so a server's keys must live in the server's own
library. Until a library has one, `kanso serve` and `kanso mcp` refuse every
request, and `kanso serve` says so when it starts.

```sh
kanso api-key create --name "Backup script" [--access read|suggest|write|capture] [--scope <notebook>]…
kanso api-key list
kanso api-key revoke <key id or name>
```

`create` prints the key, and only the key, on standard output, so
`KEY=$(kanso api-key create --name N)` holds exactly the key; it is shown this once.
`--access` is `read` by default, and each `--scope` (a notebook's name or id) limits the
key to that notebook. `list` shows the working keys, never a key or its hash. `revoke`
takes an id or a name, and asks for the id when names are shared.

These commands work while Kanso or `kanso serve` runs. A revoke refuses new requests
immediately and cuts short the requests already waiting. It waits for store calls
already under way for up to 10 seconds; if one does not finish, the command reports
an error and keeps the key blocked. A successful revoke returns only after those
calls finish, so no request for that key completes after `revoke` returns. Nothing
stops. `create` and `list` work the same way, and a key made this way works on the
next request.

How: the data folder's lock also guards encryption and migrations, so a second
program never writes the library while another holds it. The command is instead
sent to the running program, which does it itself. On Unix it goes over a
socket, `control.sock` in the data folder, with mode 0600 (`srw-------`). Both ends check
that the peer runs as the same user. On Windows the channel is a named pipe with
a DACL for the current user's SID, and each end checks the connected process's
token SID. Neither channel is on the API's port: no API key, however trusted, can
reach it, so a key cannot revoke other keys. Anyone who can already run programs
as you can revoke your keys, as they could read the library itself.

If a command says Kanso is running but can't take it from here, that Kanso is an
older version or its control channel could not start securely. Update and restart
it, or stop `kanso serve` (quit the app), run the command, and start it again.
See [the README](../README.md#headless--vps) for running a server.
