# The local API

Kanso runs a REST API and a native MCP endpoint on your computer, for AI agents,
scripts and other apps. Both listen on `http://127.0.0.1:19840` (REST at the root,
MCP at `/mcp`). Turn them on or off, and change the address or port, in
Settings › Local API › Server.

**Every caller needs its own API key.** Kanso refuses anything without one, even a
program that can start Kanso. There is no username or password for the API any more
(see [Upgrading from the old login](#upgrading-from-the-old-login)).

Keys are made and kept in the app, in Settings › Local API › **API keys**. A server
library (`kanso serve`) has no Settings; see [Headless](#headless).

## Making a key

**New key…** opens a sheet:

| Field | Choices | Default |
|---|---|---|
| For | Claude Code, Claude Desktop, Codex, Cursor, VS Code, Gemini CLI, Windsurf, Zed, OpenCode, or **Script or other app** | |
| Name | free text (a key needs a name) | the choice's name and "on this Mac", for example "Codex on this Mac" |
| Can | **Read notes**, **Read and suggest edits** ("You review each edit before it's saved."), or **Read and write** ("Changes are saved right away.") | Read notes |
| Notebooks | **All notebooks**, or **Choose…** (at least one) | All notebooks |

**Create key** replaces the sheet with the setup for that agent, with the key already
in it. **Copy** puts it on the clipboard as a concealed secret, and the toast says
"Setup for Codex copied." (or "Setup copied." for a script). **Kanso shows this key
only once**, so copy it now. Kanso keeps only a SHA-256 of the key, which can't be
turned back into it. If you lose a key, revoke it and make another.

A key looks like `kcl_` followed by 43 characters (47 in all). The prefix tells it
apart from the short-lived tokens Kanso gives its own agents, and makes a leaked key
easy to search for. Keys live in your library, never sync, and work on this computer
only. A backup holds the keys as they were when it was made, so restoring one brings
those keys back, including any you revoked since. Check Settings › API keys after a
restore.

Terminal commands for Claude Code, Gemini CLI and Codex start with a space, so a
shell set to skip such lines (fish, or bash and zsh with `ignorespace`) keeps the
key out of its history. The key is in the command's arguments, because those
agents' tools take it no other way.

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

## What a key may do

| Access | MCP tools | REST |
|---|---|---|
| Read notes | the read tools | `GET` and `HEAD` only |
| Read and suggest edits | the read tools, and `patch-note`, `update-note` and `create-note`, which only stage a change for you to review (**Review** in the Ask panel; nothing is written until you press **Apply**) | `GET` and `HEAD` only |
| Read and write | every tool | every route |

Read and suggest keys have no access to attachments. A key that can write sees an
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

## Canvases

A canvas is a note of type `canvas`. The general note routes and tools treat it as one,
with these differences, and four tools and routes are only for canvases.

- **Lists and searches** (`GET /notes`, `search-notes`, `list-notes`) show a short Markdown
  summary of the canvas (its text cards, card titles, links and recognised handwriting, up
  to about 4,000 characters) in place of its JSON, and a `size` in bytes. `GET /notes?doctype=canvas`
  lists canvases only.
- **`GET /note:<id>`** and `read-note` return the stored note, whose `body` is the `canvas`
  fence, and add `canvasJson`: the fence's inner JSON. `canvasJson` is read-only; a client that
  reads a note and writes it back whole does not store it.
- **Writes** are validated as canvases: a body that is not one canvas, or one over the 1 MB
  budget, is refused. A shared canvas's body can't be written at all; use `edit-canvas`.
- **Locked canvases** are not found: not listed, not read, not edited.

| MCP tool | REST | What it does |
|---|---|---|
| `list-canvases` | `GET /notes?doctype=canvas` | Canvases only, with title, notebook, tags, size and summary; the same `bookId`, `tagIds`, `keyword`, `sort`, `limit` as `list-notes` |
| `read-canvas` | `GET /note:<id>/canvas[?group=<id>]` | The canvas as JSON (live items only, each with its `kansoVersion`) and the note's `_rev`; with a group, only what lies inside that group |
| `edit-canvas` | `PATCH /note:<id>/canvas` | A batch of operations, applied all or none. Answers the new `rev` and the IDs it created |
| `create-canvas` | `POST /notes` with `doctype: "canvas"` | A new canvas, empty or from a canvas document |

`edit-canvas` takes `{"ifRev": "…", "ops": [ … ]}`. The operations are `addNode`, `updateNode`,
`moveNode`, `resizeNode`, `removeNode`, `addEdge`, `updateEdge`, `removeEdge`, `addStrokes`,
`removeStrokes`, `moveStrokes`, `setGroup`, `renameGroup` and `rename`. Stroke points are
`[x, y, pressure]`, relative to their ink layer. An operation on an existing item may carry
`ifVersion` (its `kansoVersion` from `read-canvas`), and the batch may carry `ifRev`. If either is
out of date the batch is refused with `409` and the versions now, so a caller never overwrites
drawing it has not seen. Each change is stamped as an edit in the app is, and a batch is one
revision in the note's history, from the API, so one undo in Kanso reverts it.

A **shared** canvas is changed through Kanso's live room, as one change the people on it see
arrive, so Kanso must be signed in. Pictures (`file:` cards) can't be added to a shared canvas
yet.

## Connecting a caller

### An MCP client

Paste the setup Kanso gives you. It has one of these shapes:

```sh
# Claude Code (Gemini CLI is the same idea)
claude mcp add --transport http --scope user kanso http://127.0.0.1:19840/mcp --header "Authorization: Bearer kcl_…"

# Codex: starts `kanso mcp`, with the key in its environment
codex mcp add kanso --env KANSO_MCP_TOKEN=kcl_… -- "/Applications/Kanso.app/Contents/MacOS/kanso" mcp
```

```json
{ "mcpServers": { "kanso": { "command": "/Applications/Kanso.app/Contents/MacOS/kanso", "args": ["mcp"], "env": { "KANSO_MCP_TOKEN": "kcl_…" } } } }
```

Cursor, VS Code, Windsurf, Zed and OpenCode get an entry in their config file with the
URL and an `Authorization: Bearer …` header; Claude Desktop starts `kanso mcp`. Kanso
tells you which file to edit. Restart the agent after you paste.

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
| A key that can't write tries to | `403` `forbidden` (REST), or a tool error (MCP): "This key can read your notes but not change them. In Kanso, open Settings › Local API › API keys and create a key that can read and write." (a suggest key says it "can suggest edits through MCP but not change notes directly") | make a key that can read and write |
| A key without attachment access asks for one | "This key can read but has no access to attachments. In Kanso, open Settings › Local API › API keys and create a key that can read and write." | make a key that can read and write |
| A key limited to some notebooks writes elsewhere | "This key is limited to some notebooks, and … is not one of them. In Kanso, open Settings › Local API › API keys and create a key for the notebooks you need." | make a key for those notebooks |
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
library. There is no `kanso` command to make a key yet: keys are made in the app's
Settings. Until a library has one, `kanso serve` and `kanso mcp` refuse every
request. See [the README](../README.md#headless--vps) for running a server.
