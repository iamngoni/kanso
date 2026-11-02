# Plugins

A plugin adds palette commands, draws fenced blocks, or shows a panel in
the inspector. It is a JavaScript file and a manifest. Kanso runs it in a
sandbox of its own, and it reaches your notes only through calls Kanso
answers, each held to what you allowed when you installed it.

Plugins never see locked notes, the trash, or notes tagged `#ai-exclude`.
They can't change a note outside the notebooks you chose, and every change
they make to more than the open note waits for you to look at it first.

## Where a plugin runs

Each plugin runs in its own `kanso-plugin-host` process, installed beside
Kanso. The host is a QuickJS interpreter with nothing else in it: no files,
no network, no timers, no `eval`, and an empty environment. Its working
folder is the plugin's own folder. The only way out is `kanso.*`, which
sends a request to Kanso and waits for the answer.

No plugin runs at launch, on a timer or in the background. A host starts
when one of its commands, blocks or panel buttons is used, and stops after
30 seconds unused unless its panel is open.

| Budget | Limit |
|---|---|
| A command or a button | 2 seconds of the plugin's own time, 5 seconds in all |
| Drawing a block | 3 seconds |
| Starting (running the script once) | 1 second |
| Memory | 64 MB |
| Calls to Kanso | 60 a minute |

A plugin past a budget is stopped and the request fails; Kanso carries on.

Kanso turns a plugin off, and says so in Settings › Plugins, when it
crashes twice in ten minutes, fails to start three times, or asks three
times for something it wasn't given. **Try again** turns it back on.

### Safe mode

Start Kanso with `--safe-mode`, or hold Shift as it opens, and every plugin
stays off for the session. Settings › Plugins can turn them back on one at
a time.

## Installing

Settings › Plugins has three ways in.

- **Browse** lists the Kanso plugin directory: a signed index of plugins,
  each bundle signed by its author. Kanso checks both signatures before it
  shows the install sheet. The directory is the
  [`iamngoni/kanso-plugins`](https://github.com/iamngoni/kanso-plugins)
  repository, and Kanso fetches it only from that repository's path on
  `raw.githubusercontent.com`, over HTTPS.
- **Install from file…** takes a `.kansoplugin` file, but only one whose
  exact bytes the directory lists.
- **Developer mode** runs an unsigned folder on this Mac only. Kanso asks you
  to type the folder's path and tick **I trust this code** before it runs,
  and picks up changes to the folder the next time the plugin is used. `kanso plugin dev <folder>`
  offers a folder to Kanso from the command line.

The install sheet says what the plugin can and can't do, who signed it, and
whether its ID looks like another plugin's. You choose which notebooks it
may read and change; locked notebooks are never offered. **Ask before
changing notes** starts on.

The first key a plugin was signed with is the one every update must be
signed with. An update signed by another key says so in red, and installs
only with **Install anyway (key changed)**. An update that asks for more
lists the new permissions. Kanso never installs an older version over a
newer one.

The directory can withdraw a version, and Kanso turns it off at the next
check. With a plugin from the directory installed, Kanso checks it 30
seconds after it opens and every six hours, for withdrawn versions and
updates; with none, it never asks. **Block on this Mac** does the same for any plugin.

Removing a plugin deletes its folder, its tokens and its settings, and, if
you tick it, the data it kept on notes.

## What is kept where

| What | Where |
|---|---|
| The plugin's files | `<data>/plugins/<id>/code/` |
| Its settings (`kanso.settings`) | `<data>/plugins/<id>/settings.json` |
| Its network requests, last 100 | `<data>/plugins/<id>/egress.json` |
| What you allowed, and its state | The local database; it never syncs |
| Its tokens | The Keychain, as `Kanso Plugin <id> <name>` |
| Its data on a note | The note's `kansoPlugin.<id>` field, which syncs with the note |

## Writing a plugin

A plugin is a folder with `manifest.json` and the script it names. Two
samples are in the directory's repository, under
[`plugins/`](https://github.com/iamngoni/kanso-plugins/tree/main/plugins):
**Word count** (commands, the selection and a panel) and **Bar chart** (a
fenced block). To run one from a clone, in developer mode:

```
kanso plugin dev kanso-plugins/plugins/app.kanso.sample.wordcount/source
```

### manifest.json

```json
{
  "schema": 1,
  "apiVersion": 1,
  "id": "com.example.wordcount",
  "name": "Word count",
  "version": "1.0.0",
  "entry": "main.js",
  "publisher": { "name": "Sam Example", "key": "ed25519:<64 hex digits>" },
  "minAppVersion": "0.8.0",
  "description": "Counts the words in the open note.",
  "permissions": {
    "commands": true,
    "editor": ["note.read", "selection.read", "selection.replace"],
    "notes": {
      "read": { "books": [], "tags": [], "all": false },
      "write": [],
      "create": false
    },
    "fences": [{ "language": "barchart" }],
    "ui": ["panel"],
    "secrets": ["token"],
    "network": ["api.example.com"]
  }
}
```

Unknown keys anywhere are refused. `id` is reverse-DNS, `version` is
semver, and `entry` and `icon` are paths inside the folder. A bundle is at
most 5 MB and 200 files, its script at most 2 MB.

| Permission | What it allows |
|---|---|
| `commands` | Palette commands |
| `editor` | `note.read`: the open note. `selection.read`: the selected text. `selection.replace`: replacing it, as one edit Undo takes back |
| `notes.read` | Reading notes in the notebooks or tags you choose at install, or every notebook with `all` |
| `notes.write` | Changing notes in the notebooks you choose (which it must also read) |
| `notes.create` | Making notes in a notebook it may change |
| `fences` | Drawing fenced blocks of these languages. `mermaid`, `math`, `excalidraw` and the other built-in ones are Kanso's |
| `ui: ["panel"]` | A panel in the inspector |
| `secrets` | Reading these tokens, which you type in Settings › Plugins |
| `network` | HTTPS requests to exactly these hosts |

### The API

The script runs once when the host starts, and registers what it offers.
Every call is synchronous and returns JSON values. A failed call throws an
`Error` with a `code`: `not_found`, `denied`, `invalid`, `conflict`,
`limit`, `timeout` or `unavailable`. A note Kanso hides is `not_found`,
the same as one that doesn't exist.

```js
kanso.commands.register({ id, title, keywords, when, run })
```

`when` is `always`, `note-open` or `selection`. `run(args)` returns
nothing, or `{ text }` to show in a notification.

```js
kanso.notes.list({ bookId, tag, query, limit, cursor }) // { notes, cursor }
kanso.notes.get(id)                                     // the note, with its body
kanso.notes.update(id, { title, body, tags, status }, { baseRev })
kanso.notes.create({ bookId, title, body })
kanso.notes.pluginData.get(id)
kanso.notes.pluginData.set(id, value)                   // at most 32 KB a note
kanso.attachments.get(fileId)                           // { contentType, size, data }
```

`list` returns summaries without bodies, most recently changed first, at
most 100 at a time. A note is `{ id, rev, title, body, bookId, tags,
tagNames, status, createdAt, updatedAt }`. `update` needs the `rev` it
read as `baseRev`; if the note changed since, nothing is changed and the
call fails with `conflict`. Attachments come only from notes it may read,
up to 4 MB.

Changes are staged, not written: when the command ends, Kanso applies them
in one go, or shows them for review first. Review is skipped only for a
change to the open note alone with **Ask before changing notes** off. A
command changes at most 500 notes, a plugin writes at most 20 MB a day,
and **Undo** in the notification takes the whole change back. History
shows each version as "Changed by a plugin", with the plugin's name.

```js
kanso.editor.noteId()                      // or null
kanso.editor.selection()                   // { text, start, end } or null
kanso.editor.replaceSelection({ text, selection: { start, end } })
```

`start` and `end` are byte offsets in the note's text. Passing the
selection it read makes the call fail with `conflict` if the person moved
it.

```js
kanso.fences.register(language, ({ source, theme }) => svg)
```

A renderer gets the block's text and the theme (`dark`, `background`,
`text`, `muted`, `accent`, `border`, `surface`, as `#rrggbb`) and returns
SVG. It can call nothing else. Kanso keeps the SVG only after cleaning it:
no scripts, no links out, no external images, at most 512 KB and 8192 by
8192 pixels. A block that can't be drawn shows its text with a badge.

```js
kanso.ui.panel(tree)
kanso.ui.onPress(buttonId, (values) => { ... })
```

A panel is a tree of nodes Kanso draws itself, inside a frame that names
the plugin: `text` (`value`, `style`: `body`, `muted` or `caption`),
`heading`, `row` and `column` (`children`, `gap`), `list` (`items`),
`button` (`id`, `label`, `tone`), `input`, `toggle`, `select` (`id`,
`label`, `value`, `options`), `image` (`attachmentId`), `link` (`label`,
an `https://` `url`), `badge`, `progress` (0 to 1) and `divider`. At most
500 nodes, 8 deep, strings up to 8 KB. Pressing a button calls its handler
with the inputs' values; nothing typed reaches the plugin before that.

```js
kanso.settings.get()        // its own JSON, up to 64 KB
kanso.settings.set(value)
kanso.secrets.get(name)     // or null
kanso.fetch(url, { method, headers, body }) // { status, headers, body }
kanso.log(level, text)      // shown in Settings › Plugins
```

`fetch` goes through Kanso: HTTPS only, to the manifest's hosts only, never
to an IP address or a private network, with redirects checked at each hop.
A plugin may make 30 requests a minute and receive 20 MB a day, and
Settings › Plugins lists its recent requests. Renderers can't fetch.

## Decisions

The design's open questions are settled:

- Plugins may write, with review.
- The directory is a repository.
- Panels exist.
- Network access is a host allowlist.
- Attachments come only from readable notes.
- Plugins are JavaScript only.
- The budgets are the ones above.
- Reads leave out `#ai-exclude` notes.
- Settings stay on this Mac.
- Per-note data syncs with the note.

## Publishing

`kanso-plugin` packs, signs and lists bundles.

```
kanso-plugin keygen --secret-out author.key
kanso-plugin pack my-plugin -o my-plugin.kansoplugin
KANSO_PLUGIN_AUTHOR_KEY=… kanso-plugin sign my-plugin.kansoplugin
```

`keygen` prints the `ed25519:` public key for the manifest's
`publisher.key`. `pack` checks the folder as Kanso would and writes the
same bytes for the same files. `sign` signs the bundle's SHA-256 with your
secret key.

To publish, open a pull request on
[`iamngoni/kanso-plugins`](https://github.com/iamngoni/kanso-plugins) that
adds `plugins/<id>/source/` and `plugins/<id>/author-signature.txt`. Once
it is merged, the repository's CI runs `kanso-plugin build-directory` and
signs the index in a protected environment. A version, once published,
never changes: a fix is a new version, signed with the same key. Anyone
can check the published directory with `kanso-plugin verify-directory`.
