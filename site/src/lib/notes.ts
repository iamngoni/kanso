/**
 * The library the in-browser Kanso opens with. The tour of the app is told
 * as notes in it, written in the app's own Markdown, so reading about a
 * feature is also trying it. Claims here must stay true of the shipped app;
 * change the copy in the same change as the feature.
 *
 * Due dates are written as {{+n}} (n days from today) and filled in when the
 * library is made, so the Tasks view always has something overdue, due today
 * and coming up.
 */

export interface Note {
  readonly id: string;
  readonly notebook: string;
  /** Minutes before now the note was last edited, for the list's times. */
  readonly edited: number;
  readonly pinned?: boolean;
  readonly body: string;
}

export const NOTEBOOKS = ["Start here", "Inside Kanso", "Privacy", "For developers", "Personal"] as const;

const SOURCE: readonly Note[] = [
  {
    id: "welcome",
    notebook: "Start here",
    edited: 2,
    pinned: true,
    body: `# Welcome to Kanso

A calm, fast notebook for Markdown. Your notes live on your computer, work offline, and sync between your devices **end-to-end encrypted**.

This window is a sketch of the app, running in your browser. Everything in it works: type in this note, tick the boxes, open another note on the left.

- [ ] Write something in [[Writing]]
- [ ] Turn on **Server view** above and see what our server stores
- [ ] Press <kbd>⌘P</kbd> and jump to any note
- [ ] Find a task with a date in [[Tasks]]

Kanso is free and open source, for macOS, Windows and Linux. Nothing you type here leaves this tab.`,
  },
  {
    id: "writing",
    notebook: "Inside Kanso",
    edited: 14,
    body: `# Writing

Markdown that keeps out of your way. Try it in this note:

- Type a bracket or a quote at the end of this line: it pairs
- Press Return at the end of this line and the list carries on
  - Tab indents an item, Shift-Tab brings it back

Paste a link and it becomes Markdown; paste code and it stays code. Tables line themselves up while you edit them.

| Mode | What it shows |
| --- | --- |
| Edit | The Markdown, coloured |
| Preview | The page it makes |
| Split | Both, side by side |

In the app there's also focus mode, typewriter scrolling, a word goal for each note, and Vim keys if you want them. #writing`,
  },
  {
    id: "links",
    notebook: "Inside Kanso",
    edited: 51,
    body: `# Links

Type \`[[\` and choose a note, like [[Tasks]] or [[Encrypted sync]]. Click either one in the preview to go there.

Every note shows what links to it, with the sentence around each link, and the mentions that could become links. The graph draws the whole web of your notes.

> Related notes are found by meaning, and worked out on your computer.

#writing`,
  },
  {
    id: "tasks",
    notebook: "Inside Kanso",
    edited: 95,
    body: `# Tasks

Give any checkbox a date and it shows up in **Tasks**, under overdue, today or coming up.

- [ ] Send the draft to Sam @{{-1}}
- [ ] Book the plumber @{{0}}
- [x] Back up the photos @{{-3}}
- [ ] Plan the garden @{{4}}

Open Tasks in the sidebar: ticking one off there ticks it here, in the note. In the app, reminders arrive as notifications.`,
  },
  {
    id: "capture",
    notebook: "Inside Kanso",
    edited: 180,
    body: `# Capture

Press <kbd>⌃⌥Space</kbd> anywhere and a quick note opens over whatever you're doing. <kbd>⌘↩</kbd> files it in your Inbox and you're back where you were.

<kbd>⌘D</kbd> opens today's page in your journal. Templates fill in \`{{date}}\`, \`{{time}}\` and \`{{title}}\` for you, and tags nest: #home/garden.`,
  },
  {
    id: "diagrams",
    notebook: "Inside Kanso",
    edited: 1440,
    body: `# Diagrams

Mermaid diagrams render right in the preview, in your note's theme. The diagram is still text you can read anywhere:

\`\`\`mermaid
flowchart LR
  Draft --> Review --> Publish
\`\`\`

Images resize with a drag, or with \`![alt|300]\` in the Markdown. Any note exports to a web page or a PDF.`,
  },
  {
    id: "search",
    notebook: "Inside Kanso",
    edited: 2880,
    body: `# Search

<kbd>⌘P</kbd> jumps to any note by its title or by a heading inside it. Search narrows by notebook, tag and status, like \`tag:writing\`.

Find and replace works across every note and undoes in one step. On a Mac, even the words in your screenshots and photos are searchable.`,
  },
  {
    id: "history",
    notebook: "Inside Kanso",
    edited: 4320,
    body: `# History

Each note keeps its earlier versions and shows exactly what changed since each one. Restore any of them with a click.

The whole library is backed up every day, on your computer.`,
  },
  {
    id: "sync",
    notebook: "Privacy",
    edited: 33,
    pinned: true,
    body: `# Encrypted sync

Notes, titles, tags and files are sealed on your device, with a key only your devices hold, before anything is sent. The server stores ciphertext and can't read a word of it.

Turn on **Server view** above to see this library the way our server does. The sealing happens here, in your browser, with a key made in this tab.

**The server can't see:** note titles or text, notebook and tag names, links between notes, file names or contents, your password or your key.

**It can see:** your username, how many notes you have and roughly how big they are, when they changed, and how many devices you've signed in.

The app seals with XChaCha20-Poly1305; this page uses your browser's AES-256-GCM. You can run the sync server on your own Cloudflare account. See [[Publishing]] for sharing a note.`,
  },
  {
    id: "publishing",
    notebook: "Privacy",
    edited: 720,
    body: `# Publishing

Turn a note into a web page that anyone with the link can read. It's encrypted before it leaves your computer, and the key lives only in the link, so the server holding the page can't read it.

Links are short and preview with the note's title. Update the page or take it down any time.`,
  },
  {
    id: "agents",
    notebook: "For developers",
    edited: 240,
    body: `# Agents

Kanso runs a local API and an MCP server on your own computer. Point Claude, Codex, Cursor or any MCP client at it, and they can search, read and, if you allow it, write your notes, on 127.0.0.1, with a key of their own, never through a cloud.

In Settings › Local API › API keys, **New key…** makes one for each agent. You choose what it can do and which notebooks it sees, and Kanso gives you the setup for yours, key included:

\`\`\`sh
claude mcp add --transport http --scope user kanso http://127.0.0.1:19840/mcp \\
  --header "Authorization: Bearer kcl_…"
\`\`\`

Claude Code, Claude Desktop, Codex, Cursor, VS Code, Gemini CLI, Windsurf, Zed and OpenCode are covered. Twenty-four tools cover notes, notebooks, tags and files, plus outlines, tasks, templates, daily notes and related notes. #agents`,
  },
  {
    id: "source",
    notebook: "For developers",
    edited: 10080,
    body: `# Open source

Kanso is free software under the AGPL-3.0. It's written in Rust and drawn by your graphics card, with no web browser hiding inside.

Notes stay plain Markdown files, so reading one back is this small:

\`\`\`rust
// A note is just text on disk.
fn read_note(path: &Path) -> io::Result<String> {
    let text = fs::read_to_string(path)?;
    println!("{} words", text.split_whitespace().count());
    Ok(text)
}
\`\`\`

Read the code, open an issue, send a pull request: [github.com/iamngoni/kanso](https://github.com/iamngoni/kanso).`,
  },
  {
    id: "sunday",
    notebook: "Personal",
    edited: 300,
    body: `# Sunday

- [ ] Call Mum about the trip @{{2}}
- [ ] Ideas for the [[Garden plan]]: tomatoes along the south wall

The spare key is under the blue pot. #home`,
  },
  {
    id: "garden",
    notebook: "Personal",
    edited: 8000,
    body: `# Garden plan

- Tomatoes along the south wall
- Herbs by the kitchen door
- [ ] Order seeds @{{6}} #home/garden`,
  },
];

function isoDay(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

/** The day `offset` days from `today`, as `YYYY-MM-DD`. */
export function dayFrom(today: Date, offset: number): string {
  const d = new Date(today.getFullYear(), today.getMonth(), today.getDate() + offset);
  return isoDay(d);
}

/** The library, with its due dates set relative to `today`. */
export function library(today: Date): Note[] {
  return SOURCE.map((note) => ({
    ...note,
    body: note.body.replace(/\{\{([+-]?\d+)\}\}/g, (_, n: string) => dayFrom(today, Number(n))),
  }));
}

/** When a note was edited, the way the note list says it. */
export function editedLabel(minutes: number): string {
  if (minutes < 1) return "Just now";
  if (minutes < 60) return `${Math.round(minutes)} min ago`;
  if (minutes < 60 * 24) return `${Math.round(minutes / 60)} h ago`;
  const days = Math.round(minutes / (60 * 24));
  return days === 1 ? "Yesterday" : `${days} days ago`;
}
