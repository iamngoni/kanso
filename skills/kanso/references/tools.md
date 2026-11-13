# Kanso MCP tool reference

Generated from `crates/kanso-server/src/mcp/tools.json`, `kanso_tools.json` and `task_tools.json`. A test (`crates/kanso-server/tests/skill_reference.rs`) fails when a tool is missing here. Which tools a key sees depends on its access: see SKILL.md.

## Notes, notebooks, tags, templates, attachments

Inkdrop-compatible tools. Which of them a client sees depends on its key's access.

### `read-note`

Retrieve the complete contents of the note by its ID from the database. A note created from a template has a `sourceTemplateId`; when present, the response includes a resource link to that source template, whose body holds the fill-out instructions and examples. Read the template (or pass `includeTemplate: true`) before completing the note.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note to retrieve. It can be found as `_id` in the note docs. It always starts with `note:`. |
| `includeTemplate` | `boolean` | no | When true and the note has a `sourceTemplateId`, also fetch that template and embed its body (the fill-out instructions and examples) as an extra text block, so you get the note and its template in one call. (default `false`) |

### `read-backlinks`

Retrieve backlinks for a note — the notes that link to it. Each result is returned as a resource link (`inkdrop://note/<id>`) pointing to the referring note. Backlinks are found by searching all note bodies for the note's link URI.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note to find backlinks for. It can be found as `_id` in the note docs. It always starts with `note:`. |

### `search-notes`

List all notes that contain a given keyword. The result does not include entire note bodies as they are truncated in 200 characters. You have to retrieve the full note content by calling `read-note`. Here are tips to specify keywords effectively:

| Parameter | Type | Required | Description |
|---|---|---|---|
| `keyword` | `string` | yes | Keyword to search for. |

### `list-notes`

List all notes with specified conditions. The result does not include entire note bodies as they are truncated in 200 characters. You have to retrieve the full note content by calling `read-note`.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `bookId` | `string` | no | ID of the notebook. It always starts with 'book:'. You can retrieve a list of notebooks with `list-notebooks`. The pseudo-notebooks 'trash' and 'template' are not supported here; use `list-templates` for templates |
| `tagIds` | `array of string` | no | An array of tag IDs to filter. It always starts with 'tag:'. You can retrieve a list of available tags from `list-tags`. (default `[]`) |
| `keyword` | `string` | no | Keyword to filter notes |
| `sort` | `updatedAt` \| `createdAt` \| `title` | no | Sort the documents by the specified field (default `"updatedAt"`) |
| `descending` | `boolean` | no | Reverse the order of the output documents (default `true`) |
| `limit` | `number` | no | Limit the number of results returned (default `100`) |

### `list-templates`

List note templates: your custom templates, followed by the official ones bundled with Inkdrop. Templates are notes whose `bookId` is `template`. Official templates are read-only. The result does not include entire template bodies as they are truncated in 200 characters. You have to retrieve the full template content by calling `read-note`.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `keyword` | `string` | no | Filter templates by fuzzy-matching their titles. Search qualifiers such as `book:` or `tag:` are not supported. |

### `create-note`

Create a new note in the database

| Parameter | Type | Required | Description |
|---|---|---|---|
| `bookId` | `string` | yes | The notebook ID. Specify `template` to create a custom note template |
| `title` | `string` | yes | The note title |
| `body` | `string` | yes | The content of the note represented with Markdown |
| `status` | `none` \| `active` \| `onHold` \| `completed` \| `dropped` | no | The status of the note |
| `tags` | `array of string` | no | An array of tag IDs to assign to the note. Call `list-tags` or `read-tag` to retrieve available tags. You can create a new tag with `create-tag` tool if necessary. |

### `update-note`

Update the existing note in the database. Only the fields you provide will be updated; omitted fields remain unchanged.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `_id` | `string` | yes | The unique document ID which should start with `note:` and the remains are randomly generated string |
| `_rev` | `string` | yes | This is a CouchDB specific field. The current MVCC-token/revision of this document (mandatory and immutable). |
| `bookId` | `string` | no | The notebook ID. Specify `template` to turn the note into a custom note template |
| `title` | `string` | no | The note title |
| `body` | `string` | no | The content of the note in Markdown. NOTE: Do not escape special characters like `\n`. |
| `status` | `none` \| `active` \| `onHold` \| `completed` \| `dropped` | no | The status of the note |
| `tags` | `array of string` | no | An array of tag IDs to assign to the note. Call `list-tags` or `read-tag` to retrieve available tags. You can create a new tag with `create-tag` tool if necessary. |

### `patch-note`

Update the body of the existing note by performing an exact string replacement. Use this tool to make partial updates to the note body without replacing the entire content. You must first read the note with `read-note` to get the current body, then specify the exact substring to replace.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `_id` | `string` | yes | The unique document ID which should start with `note:` and the remains are randomly generated string |
| `_rev` | `string` | yes | This is a CouchDB specific field. The current MVCC-token/revision of this document (mandatory and immutable). |
| `old_string` | `string` | yes | The exact text to find in the note body. Must match exactly one occurrence. Include enough surrounding context to ensure a unique match. |
| `new_string` | `string` | yes | The text to replace `old_string` with. Use an empty string to delete the matched text. |

### `list-tags`

Retrieve a list of all tags

No parameters.

### `read-tag`

Retrieve a single tag

| Parameter | Type | Required | Description |
|---|---|---|---|
| `tagId` | `string` | yes | ID of the tag to retrieve. It can be found as `_id` in the tag docs. It always starts with `tag:`. |

### `create-tag`

Create a new tag in the database

| Parameter | Type | Required | Description |
|---|---|---|---|
| `color` | `default` \| `red` \| `orange` \| `yellow` \| `olive` \| `green` \| `teal` \| `blue` \| `violet` \| `purple` \| `pink` \| `brown` \| `grey` \| `black` | no | The color type of the tag (default `"default"`) |
| `name` | `string` | yes | The name of the tag |

### `update-tag`

Update the existing tag in the database. Only the fields you provide will be updated; omitted fields remain unchanged, so you do not need to read the tag first. Call `list-tags` or `read-tag` to find the tag ID.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `_id` | `string` | yes | The unique document ID which should start with `tag:` and the remains are randomly generated string |
| `_rev` | `string` | no | This is a CouchDB specific field. The current MVCC-token/revision of this document. Optional: the server merges your fields into the stored tag, so it is only needed as an optimistic-concurrency guard. Pass it when you have read the tag first and want the update to fail on a conflicting concurrent edit. |
| `color` | `default` \| `red` \| `orange` \| `yellow` \| `olive` \| `green` \| `teal` \| `blue` \| `violet` \| `purple` \| `pink` \| `brown` \| `grey` \| `black` | no | The color type of the tag |
| `name` | `string` | no | The name of the tag |

### `list-notebooks`

Retrieve a list of all notebooks

No parameters.

### `read-book`

Retrieve a single book

| Parameter | Type | Required | Description |
|---|---|---|---|
| `bookId` | `string` | yes | ID of the book to retrieve. It can be found as `_id` in the book docs. It always starts with `book:`. |

### `create-book`

Create a new notebook in the database. To create a nested notebook, pass the parent notebook ID as `parentBookId`. Call `list-notebooks` first to find it.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `name` | `string` | yes | The notebook name |
| `parentBookId` | `string` | no | The ID of the parent notebook. Omit it to create the notebook at the root level. |

### `update-book`

Update the existing notebook in the database. Only the fields you provide will be updated; omitted fields remain unchanged, so you do not need to read the notebook first. Call `list-notebooks` or `read-book` to find the notebook ID.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `_id` | `string` | yes | The unique document ID which should start with `book:` and the remains are randomly generated string |
| `_rev` | `string` | no | This is a CouchDB specific field. The current MVCC-token/revision of this document. Optional: the server merges your fields into the stored notebook, so it is only needed as an optimistic-concurrency guard. Pass it when you have read the notebook first and want the update to fail on a conflicting concurrent edit. |
| `name` | `string` | no | The notebook name |
| `parentBookId` | `any` | no | The ID of the parent notebook. Pass `null` to move the notebook to the root level. |

### `create-file`

Create a new attachment file in the database from a local image file or from base64 image data. Returns the Markdown to embed the attachment in a note body. Prefer `filePath` over `data`: it keeps the image bytes out of the conversation.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `filePath` | `string` | no | Absolute path to a local image file to attach. Either this or `data` must be given. |
| `data` | `string` | no | Base64-encoded image data, without a `data:` URI prefix. Use `filePath` instead whenever the image is already on disk. |
| `name` | `string` | no | The file name. Defaults to the file name of `filePath`, or `image.<ext>` when `data` is given. |
| `contentType` | `image/png` \| `image/jpeg` \| `image/jpg` \| `image/svg+xml` \| `image/gif` \| `image/heic` \| `image/heif` | no | The MIME type of the image. Inferred from the `filePath` extension when omitted, so it is required with `data`. |

### `read-file`

Retrieve an attachment file by its ID. Returns the file metadata, and the image itself when it is a PNG, JPEG or GIF. Pass `outputPath` to write the attachment to disk instead — required to get at SVG, HEIC and HEIF attachments, which cannot be displayed inline.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `fileId` | `string` | yes | ID of the attachment to retrieve. It can be found as `_id` in the file docs. It always starts with `file:`. |
| `outputPath` | `string` | no | Absolute path to write the attachment to. When given, the file is saved there and its bytes are not returned inline. |
| `includeImage` | `boolean` | no | Whether to return the image itself alongside the metadata. Set to false to fetch only the metadata of a large attachment. (default `true`) |

## Kanso tools

Tools native to Kanso (names start with `kanso-`).

### `kanso-add-pdf-highlight`

Add a PDF highlight by page and exact text quote. The quote is resolved to page quads and saved in one note revision. Its page quote is also added to the note by default; set includeQuote to false to leave it out.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |
| `page` | `integer` | yes | One-based PDF page number. |
| `quote` | `string` | yes | Exact text on the page to highlight. |
| `color` | `yellow` \| `green` \| `blue` \| `pink` | no | Highlight color. (default `"yellow"`) |
| `comment` | `string` | no | Comment text for the highlight; empty text clears it. |
| `includeQuote` | `boolean` | no | Whether to add the page quote to the note body. (default `true`) |

### `kanso-add-task`

Add one task to a note, as a `- [ ] …` line after the note's last line. Nothing else in the note changes. A date and repeat at the end of the text are written as Kanso's own tokens: `film Pixel review fri 3pm every week` becomes `- [ ] film Pixel review @2026-11-13T15:00 every week`. Text that ends in no date is added as it is. Say where it goes with `noteId` (that note), or `target` (`daily`: today's daily note, made when missing, which is the default; `inbox`: the Inbox's task note, Inbox tasks, which Quick Capture also adds to, made when missing), not both. Returns the note's ID, the task's 0-based line, the line written, and its due date, repeat and a short label when it has them. A locked note is refused.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `text` | `string` | yes | The task, on one line, with its date and repeat at the end if it has any, such as `call Sam tomorrow 9am` or `water plants every 3 days after done`. |
| `noteId` | `string` | no | The note to add the task to. Leave it out to use `target`. |
| `target` | `daily` \| `inbox` | no | Where the task goes when there is no noteId: today's daily note (the default), or the Inbox's task list. |
| `dateOrder` | `dmy` \| `mdy` | no | How a date written with a slash reads: `14/11` is the 14th of November in dmy (the default), and `11/14` is in mdy. |

### `kanso-ask-read-lines`

Read lines of one note for Ask. Returns the lines written `L12\| text`, under the note's number, so you can cite them as `[number:L12-14]`. At most 200 lines. A note that cannot be read is reported as not found.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note. It always starts with `note:`. |
| `start` | `integer` | yes | The first line to read, counting from 1. |
| `end` | `integer` | no | The last line to read, counting from 1. At most 200 lines after start. |

### `kanso-ask-search`

Search the user's notes for Ask. Returns the best matching notes, each with a number, its title, the day it last changed and the lines that match, written `L12\| text`. Cite what you use as `[number:L12]` or `[number:L12-14]`, with the note's number and the line numbers shown. Only notes this session may read are searched.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `query` | `string` | yes | Words to search for. |
| `limit` | `integer` | no | How many notes, from 1 to 10. (default `5`) |

### `kanso-ask-when`

Find notes by when they were written or changed, or tasks by when they are due, for Ask. Dates are `YYYY-MM-DD`. Returns numbered notes with their opening lines, written `L1\| text`, and tasks due in the range with their lines.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `from` | `string` | yes | The first day, `YYYY-MM-DD`. |
| `to` | `string` | no | The last day, `YYYY-MM-DD`. Defaults to the first day. |
| `kind` | `written` \| `about` | no | `written` finds notes changed in the range; `about` finds tasks due in it. (default `"written"`) |
| `limit` | `integer` | no | How many notes, from 1 to 15. (default `8`) |

### `kanso-attach-file`

Create an image attachment from base64 bytes and explicitly append its Markdown image link to a note. The file must fit Kanso’s 10 MiB attachment limit and use a supported image content type.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note to update. |
| `name` | `string` | yes | The filename to store and the image alt text. |
| `contentType` | `image/png` \| `image/jpeg` \| `image/jpg` \| `image/svg+xml` \| `image/gif` \| `image/heic` \| `image/heif` | yes | Supported type: image/png, image/jpeg, image/jpg, image/svg+xml, image/gif, image/heic, or image/heif. |
| `data` | `string` | yes | Base64 encoded file bytes, no data URI prefix. |

### `kanso-create-canvas`

Create a new canvas note, empty or from a canvas document (JSON Canvas with Kanso's ink nodes), validated like any canvas. Picture (file:) cards are refused: a new canvas has no pictures of its own. A key limited to some notebooks must pass bookId.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `title` | `string` | yes | The canvas title |
| `bookId` | `string` | no | The notebook ID. Without one the canvas goes to the Inbox. |
| `canvas` | `object` | no | A canvas document: {nodes: [...], edges: [...]}. Leave out for an empty canvas. |
| `tags` | `array of string` | no | An array of tag IDs to assign. |

### `kanso-create-daily-note`

Open the daily note for a day, creating it when there is none, as Kanso's Today command does. Daily notes have fixed IDs (`note:daily-YYYY-MM-DD`) and titles (`YYYY-MM-DD`) and live in the Journal notebook (`book:journal`), which is created on first use. A new daily note is filled in from the Journal's daily template, if one is set. Returns the note's ID, title, notebook and link, and whether it was created now.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `date` | `string` | no | The day, as YYYY-MM-DD. Defaults to today in this computer's time zone. |

### `kanso-create-from-template`

Create a note from a template: a note in the Templates notebook (`book:templates`). The template's body is copied with its variables filled in: `{{date}}` (YYYY-MM-DD), `{{time}}` (HH:MM), `{{title}}` (the new note's title), `{{date:FORMAT}}` and `{{time:FORMAT}}` (strftime-like: %Y %m %d %e %B %b %A %a %H %M and more) and `{{cursor}}` (removed). Other text in double braces stays as it is. The note gets the template's tags and records it as `sourceTemplateId`. Returns the new note's ID, title, notebook and link.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `templateId` | `string` | yes | ID of the template note. It can be found as `_id` in the note docs. It always starts with `note:`. |
| `title` | `string` | yes | The new note's title; `{{title}}` in the template becomes it. |
| `bookId` | `string` | no | The notebook to create the note in. It can be found as `_id` in the book docs. It always starts with `book:`. Defaults to the Inbox (`book:inbox`), created on first use. |

### `kanso-delete-pdf-highlight`

Delete a PDF highlight and its comment. The quote already copied into the note stays there.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |
| `highlightId` | `string` | yes | ID of the highlight to change or remove. |

### `kanso-edit-canvas`

Change a canvas with a batch of operations, applied all or none (a refused batch changes nothing). Each operation has an `op`: addNode {type: text\|file\|link\|group\|ink, x, y, width?, height?, text?, file? (a 'note:' ID you can read, or a 'file:' picture already on this canvas), title?, url?, label?, color?, id?}, updateNode {id, text?, title?, color?, label?}, moveNode {id, x, y}, resizeNode {id, width, height}, removeNode {id} (arrows to it go too), addEdge {fromNode, toNode, fromSide?, toSide?, fromEnd?, toEnd?, label?, color?, id?}, updateEdge {id, ...}, removeEdge {id}, addStrokes {node, strokes: [{tool?, color?, width?, points: [[x, y, pressure], ...]}]} (points relative to the ink n

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The canvas note's ID, starting with 'note:'. |
| `ifRev` | `string` | no | The `_rev` you read; the batch is refused if the canvas has changed since. |
| `ops` | `array of object` | yes | The operations, in order. |

### `kanso-edit-pdf-comment`

Change or clear a highlight comment; an empty string clears it.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |
| `highlightId` | `string` | yes | ID of the highlight to change or remove. |
| `comment` | `string` | yes | Comment text for the highlight; empty text clears it. |

### `kanso-edit-pdf-highlight`

Change a PDF highlight color and/or comment. An empty comment clears it.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |
| `highlightId` | `string` | yes | ID of the highlight to change or remove. |
| `color` | `yellow` \| `green` \| `blue` \| `pink` | no | Highlight color. |
| `comment` | `string` | no | Comment text for the highlight; empty text clears it. |

### `kanso-export-html`

Export a note as one self-contained HTML page, as Kanso's "Export as web page" does. TeX math ($…$ and $$…$$), Mermaid diagrams and Excalidraw drawings are drawn as inline SVG, attached images are embedded as data URIs, and the page has no scripts and loads nothing from the network. Returns the HTML.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note to export. It can be found as `_id` in the note docs. It always starts with `note:`. |
| `theme` | `light` \| `dark` | no | The colors of the page: Kanso's light (paper) or dark (graphite) theme. (default `"light"`) |

### `kanso-get-outline`

Retrieve a note's outline: its headings in order, each with its level (1 to 6), its text without Markdown markers, and the 0-based line it is on in the note's body. Headings inside code blocks are not part of the outline.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note. It can be found as `_id` in the note docs. It always starts with `note:`. |

### `kanso-get-properties`

Read the Kanso properties parsed from a note body. The note must be visible to this caller.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note ID, with or without the `note:` prefix. |

### `kanso-list-attachments`

List attachment metadata for files embedded in a note. The note must be available through the local API.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note ID, with or without the `note:` prefix. |

### `kanso-list-backlinks`

List notes that link to this note by ID or wiki title. The target and every returned note must be visible to this caller.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The target note ID, with or without the `note:` prefix. |

### `kanso-list-canvases`

List canvases (notes of type canvas) only, never Markdown notes. Each has its title, notebook, tags, size in bytes and a short Markdown summary of the text cards, card titles, links and recognized handwriting on it. Read the whole document with `kanso-read-canvas`. Locked canvases are not listed.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `bookId` | `string` | no | ID of the notebook. It always starts with 'book:'. |
| `tagIds` | `array of string` | no | An array of tag IDs to filter. It always starts with 'tag:'. (default `[]`) |
| `keyword` | `string` | no | Keyword to filter canvases |
| `sort` | `updatedAt` \| `createdAt` \| `title` | no | Sort the canvases by the specified field (default `"updatedAt"`) |
| `descending` | `boolean` | no | Reverse the order of the output (default `true`) |
| `limit` | `number` | no | Limit the number of results returned (default `100`) |

### `kanso-list-mentions`

List the lines of notes that mention a person or project page (by link, @name or plain words). Pass the page's note ID or its name or alias. The page and every returned note must be visible to this caller.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `entity` | `string` | yes | The person or project page: its note ID (with the `note:` prefix), or its name or an alias. |

### `kanso-list-note-links`

List outgoing ID and wiki links in a note, with their source line and resolved visible target when available. The note must be visible to this caller.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note ID, with or without the `note:` prefix. |

### `kanso-list-open-loops`

List open loops: things notes say the writer will do ("I'll send the deck", "TODO: renew passport"), is waiting on ("waiting on Rudo"), or has to check, that are not checkboxes. Kanso finds them with phrase rules and, when the person approved them, with AI (`source` is `rule` or `ai`). Each has its sentence, kind (`mine`, `waiting` or `question`), any due date, its 0-based line in the note's body, and the note's ID, title, notebook and link. Loops the person closed or dismissed are not listed. Locked notes and notes tagged #ai-exclude have none.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `kind` | `mine` \| `waiting` \| `question` | no | Only loops of this kind. |
| `book` | `string` | no | Only loops in this notebook's notes. It can be found as `_id` in the book docs. It always starts with `book:`. |
| `limit` | `number` | no | Limit the number of loops returned (default `100`) |

### `kanso-list-pdf-highlights`

List the highlights and comments for a PDF attached to a note you may read.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |

### `kanso-list-tags`

List tags used by notes available to this caller. Counts include only notes the caller may see.

No parameters.

### `kanso-list-tasks`

List the task-list items (`- [ ] …` and `- [x] …`) in notes outside the trash, with the note each is in. A task's due date is an `@YYYY-MM-DD` (or `@YYYY-MM-DDTHH:MM`, which also sets a reminder) in its text. Dated tasks come first, soonest first, then tasks without a date. Each task has its text, whether it is done, its due date, its repeat (the `every …` words after the date, or null), its 0-based line in the note's body, and the note's ID, title, notebook and link. To tick a task off, change its `[ ]` to `[x]` on that line with patch-note.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `status` | `open` \| `done` \| `all` | no | Which tasks: open ones (not done), done ones, or all. (default `"open"`) |
| `dueBefore` | `string` | no | Only tasks due on a day before this one, as YYYY-MM-DD. |
| `dueAfter` | `string` | no | Only tasks due on a day after this one, as YYYY-MM-DD. |
| `bookId` | `string` | no | Only tasks in this notebook's notes. It can be found as `_id` in the book docs. It always starts with `book:`. |
| `limit` | `number` | no | Limit the number of tasks returned (default `100`) |

### `kanso-list-versions`

List saved versions of a visible note. This returns revision metadata only, not historical text.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note ID, with or without the `note:` prefix. |

### `kanso-move-note`

Move one note to an explicitly named notebook. The store enforces notebook and lock rules.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note to move. |
| `bookId` | `string` | yes | The destination notebook ID. |

### `kanso-parse-when`

Read the date, time and repeat at the end of a line of task text, the way kanso-add-task does, without writing anything. `film Pixel review fri 3pm every week` gives the words `film Pixel review`, the due date `2026-11-13T15:00`, the repeat `every week` and a label for a person to read. A line that does not end in a date gives its text back with due, repeat and label null. A repeat with no date before it is not read.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `text` | `string` | yes | A line of task text. |
| `dateOrder` | `dmy` \| `mdy` | no | How a date written with a slash reads: `14/11` is the 14th of November in dmy (the default), and `11/14` is in mdy. |

### `kanso-read-attachment`

Read one attachment document embedded in a visible note, including its stored base64 bytes and metadata. Only available to the full local API caller.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `fileId` | `string` | yes | The attachment file ID, with or without the `file:` prefix. |
| `noteId` | `string` | yes | The note ID containing this attachment, with or without the `note:` prefix. |

### `kanso-read-canvas`

Read a canvas as JSON: its cards (nodes), arrows (edges) and ink strokes, each with its `kansoVersion`, and the note's `_rev`. Pass `groupId` to get only what lies inside one group. Stroke points are [x, y, pressure] relative to their ink node. Use the versions as `ifVersion` and `_rev` as `ifRev` in `kanso-edit-canvas` so you never overwrite drawing you have not seen.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The canvas note's ID, starting with 'note:'. |
| `groupId` | `string` | no | The ID of a group node on the canvas: return only the cards, strokes and arrows inside its rectangle. |

### `kanso-read-pdf-page`

Read one PDF page as text from a PDF attached to a note you may read. Page numbers start at 1.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the visible note that owns this PDF. |
| `fileId` | `string` | yes | ID of the PDF attached to that note. |
| `page` | `integer` | yes | One-based PDF page number. |

### `kanso-read-version`

Read one saved version of a currently visible note. This does not restore or change the note.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note ID, with or without the `note:` prefix. |
| `rev` | `string` | yes | The exact revision identifier returned by kanso-list-versions. |

### `kanso-related-notes`

Find the notes most similar in meaning to a note, by Kanso's related notes: embeddings made on the user's device from each note's words, never sent anywhere. Notes the note already links to, or is linked from, are left out. Returns each note's ID, title, `inkdrop://note/<id>` link and similarity (0 to 1, higher is closer), most similar first. Related notes are off until the user turns them on in Kanso's Settings; while they are off, or the note isn't indexed yet, it returns an empty list and says why.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | ID of the note. It can be found as `_id` in the note docs. It always starts with `note:`. |
| `limit` | `integer` | no | How many notes to return, from 1 to 20. (default `5`) |

### `kanso-resolve-wikilink`

Find the note a wiki-link points at. Accepts `[[Title]]`, `[[Title#Heading]]` or `[[Title\|alias]]`, with or without the brackets. Titles match without regard to case or extra spaces, among notes outside the trash; when several notes share a title, the one created first is the match. Returns the note's ID, title and `inkdrop://note/<id>` link, which is the form to write when linking to it, plus the heading's line when the link names a heading the note has.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `link` | `string` | yes | The wiki-link, such as `[[Project Plan#Budget]]`, or just the title. |

### `kanso-run-query-view`

Run the first saved `query` fenced view in a note using Kanso’s query parser and evaluator. Locked, trashed, excluded, and out-of-scope notes are omitted.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note containing the saved query view, with or without the `note:` prefix. |

### `kanso-search-by-meaning`

Find the notes about something, by meaning rather than by words: the search is turned into numbers by the same language model as Kanso's related notes, on the user's device, and compared with every note's. A note can match without containing any of the words; for notes containing the words, use search-notes. Returns each note's ID, title, `inkdrop://note/<id>` link and similarity (0 to 1, higher is closer), most similar first, leaving out notes less than 0.35 similar. It needs related notes on in Kanso's Settings and the Kanso app open with its model loaded (a server started with `kanso serve` has no model); otherwise it returns an empty list and says why.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `query` | `string` | yes | What the notes should be about, in a few words or a sentence. |
| `limit` | `integer` | no | How many notes to return, from 1 to 20. (default `5`) |

### `kanso-search-pdf`

Search extracted PDF text in notes you may read. Locked notes and PDFs in locked notebooks are never returned.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `query` | `string` | yes | Text to find in indexed PDF pages. |
| `limit` | `integer` | no | Maximum number of matching pages to return, from 1 to 100. (default `20`) |

### `kanso-set-task-done`

Set one task’s checked state by its note, current line, and exact task text. This is an explicit note write; it fails without changing the note if the task moved ambiguously.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `noteId` | `string` | yes | The note containing the task. |
| `line` | `integer` | yes | The task’s current zero-based line in the note body. |
| `text` | `string` | yes | The exact task text returned by kanso-list-tasks, including any due token. |
| `done` | `boolean` | yes | True to tick the task, false to clear its checkbox. |

### `kanso-sync-now`

Ask the library's sync driver to sync now.

No parameters.

### `kanso-sync-status`

Read the library's safe sync status without revealing account or device identifiers.

No parameters.

## Agent-task tools

Only the token of an agent task that Kanso starts itself gets these. An API key never sees them. They stage proposals for the person to review and write nothing.

### `kanso-list-proposed`

List everything you have proposed so far in this task, so you can revise it.

No parameters.

### `kanso-propose-merge`

Propose merging several notes into one note. Nothing is written: the person reviews it. When they apply it, the source notes move to the Trash. Give the merged note's title and full text, and either `into` (an existing note to replace) or `bookId` (a notebook for a new note).

| Parameter | Type | Required | Description |
|---|---|---|---|
| `sources` | `array of object` | yes | 2 to 50 notes to combine, each with its _id and the _rev you read it at. |
| `into` | `object` | no | An existing note to replace with the merged text. |
| `bookId` | `string` | no | The notebook for the merged note, when it is a new note. |
| `title` | `string` | yes | The merged note's title. |
| `body` | `string` | yes | The merged note's full Markdown text. Cite each source as [[Title]]. |

### `kanso-propose-move`

Propose moving a note to another notebook. Nothing is written: the person reviews it.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `_id` | `string` | yes | The note to move. |
| `_rev` | `string` | yes | The revision you read the note at. |
| `bookId` | `string` | yes | The notebook to move it to. |

### `kanso-task-done`

Say you are finished. Give a short summary of what you proposed and anything you were unsure about.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `summary` | `string` | yes | A short summary for the person. |
