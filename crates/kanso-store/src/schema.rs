//! Connection setup and schema migrations.

use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{LibraryKey, Result};

pub(crate) const MIGRATIONS: &[&str] = &[
    r#"
CREATE TABLE docs (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    rev TEXT NOT NULL,
    seq INTEGER NOT NULL UNIQUE,
    deleted INTEGER NOT NULL DEFAULT 0,
    -- The document without `_id`, `_rev` and derived fields.
    json TEXT NOT NULL,
    -- The revision last known to match the sync server; NULL if never synced.
    synced_rev TEXT,
    book_id TEXT GENERATED ALWAYS AS (json_extract(json, '$.bookId')) VIRTUAL,
    parent_book_id TEXT GENERATED ALWAYS AS (json_extract(json, '$.parentBookId')) VIRTUAL,
    status TEXT GENERATED ALWAYS AS (json_extract(json, '$.status')) VIRTUAL,
    title TEXT GENERATED ALWAYS AS (json_extract(json, '$.title')) VIRTUAL,
    name TEXT GENERATED ALWAYS AS (json_extract(json, '$.name')) VIRTUAL,
    updated_at INTEGER GENERATED ALWAYS AS (json_extract(json, '$.updatedAt')) VIRTUAL,
    created_at INTEGER GENERATED ALWAYS AS (json_extract(json, '$.createdAt')) VIRTUAL,
    pinned INTEGER GENERATED ALWAYS AS (json_extract(json, '$.pinned')) VIRTUAL
);
CREATE INDEX docs_type_updated ON docs (type, deleted, updated_at);
CREATE INDEX docs_book ON docs (type, book_id);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (note_id, tag_id)
) WITHOUT ROWID;
CREATE INDEX note_tags_tag ON note_tags (tag_id);

CREATE TABLE blobs (
    file_id TEXT PRIMARY KEY NOT NULL,
    md5 TEXT NOT NULL,
    data BLOB NOT NULL
);

CREATE TABLE revisions (
    id INTEGER PRIMARY KEY,
    doc_id TEXT NOT NULL,
    rev TEXT NOT NULL,
    json TEXT NOT NULL,
    saved_at INTEGER NOT NULL,
    origin TEXT NOT NULL
);
CREATE INDEX revisions_doc ON revisions (doc_id, saved_at);

CREATE TABLE meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Imported documents Kanso doesn't use (an imported library's activity log), kept verbatim.
CREATE TABLE archive (
    id TEXT PRIMARY KEY NOT NULL,
    json TEXT NOT NULL
);

CREATE VIRTUAL TABLE notes_fts USING fts5(note_id UNINDEXED, title, body, tokenize = 'trigram');
"#,
    // The replay floor (docs/ACCOUNTS.md › Sync with encryption › What a Mac
    // remembers per document): the highest generation this Mac has seen on
    // its account for the document. A server-reported loss and a re-merge
    // clear `synced_rev` but never this. Existing rows start at the
    // generation of their `synced_rev`.
    r#"
ALTER TABLE docs ADD COLUMN synced_gen INTEGER NOT NULL DEFAULT 0;
UPDATE docs SET synced_gen = CAST(substr(synced_rev, 1, instr(synced_rev, '-') - 1) AS INTEGER)
    WHERE synced_rev IS NOT NULL AND instr(synced_rev, '-') > 1;
"#,
    // The revision in a push whose answer hasn't been processed: its echo
    // is our own, never a conflict.
    r#"
ALTER TABLE docs ADD COLUMN sent_rev TEXT;
"#,
    // Tables derived from note bodies (`derived.rs`): local to this device,
    // filled from Rust when the store opens and kept current by every write.
    r#"
CREATE TABLE note_links (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    -- `note:…` for an ID link to a note.
    target_id TEXT,
    -- The normalized title for a wiki-link.
    target_title TEXT,
    heading TEXT,
    line INTEGER NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX note_links_target_id ON note_links (target_id) WHERE target_id IS NOT NULL;
CREATE INDEX note_links_target_title ON note_links (target_title) WHERE target_title IS NOT NULL;

CREATE TABLE note_headings (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    level INTEGER NOT NULL,
    text TEXT NOT NULL,
    line INTEGER NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;

CREATE TABLE note_tasks (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    line INTEGER NOT NULL,
    checked INTEGER NOT NULL,
    text TEXT NOT NULL,
    -- `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, from an `@date` in the task.
    due TEXT,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX note_tasks_due ON note_tasks (due) WHERE due IS NOT NULL;
"#,
    // Find and replace across notes (`replace.rs`): the version each note
    // had before a replace is kept under the replace's batch ID, with the
    // revision the replace wrote, so "Undo replace" can tell a note edited
    // again since.
    r#"
ALTER TABLE revisions ADD COLUMN batch TEXT;
ALTER TABLE revisions ADD COLUMN batch_rev TEXT;
CREATE INDEX revisions_batch ON revisions (batch) WHERE batch IS NOT NULL;
"#,
    // Discovery (`embed.rs`, `ocr.rs`): local to this device like the
    // tables above, never synced, and emptied with the library. A note's
    // embedding for related notes, from one model, with the hash of the
    // text it was made from; and the text read from an image attachment
    // (mirrored from the file's synced `kansoOcr`), for the app's search.
    r#"
CREATE TABLE note_embeddings (
    note_id TEXT NOT NULL,
    model TEXT NOT NULL,
    dim INTEGER NOT NULL,
    -- `dim` little-endian f32s.
    vector BLOB NOT NULL,
    content_hash TEXT NOT NULL,
    PRIMARY KEY (note_id, model)
) WITHOUT ROWID;

CREATE TABLE file_text (
    file_id TEXT PRIMARY KEY NOT NULL,
    -- The MD5 of the image the text was read from.
    md5 TEXT NOT NULL,
    text TEXT NOT NULL
);
"#,
    // Locked notes (`Store::lock_note`): the search index forgets a row's
    // words when it is deleted, rather than leaving them in its segments
    // until they merge.
    r#"
INSERT INTO notes_fts (notes_fts, rank) VALUES ('secure-delete', 1);
"#,
    // Locked notebooks: a notebook whose `kansoLock` is set, and every
    // notebook inside one. Their notes are read as locked notes are (with
    // an empty body) by everything but the app, sealed or not.
    r#"
CREATE INDEX docs_parent ON docs (type, parent_book_id);
CREATE VIEW locked_books (id) AS
WITH RECURSIVE locked (id) AS (
    SELECT id FROM docs WHERE type = 'book' AND deleted = 0
        AND json_extract(json, '$.kansoLock') IS NOT NULL
    UNION
    SELECT d.id FROM docs d JOIN locked l ON d.parent_book_id = l.id
        WHERE d.type = 'book' AND d.deleted = 0
)
SELECT id FROM locked;
"#,
    // Shared notes (`collab/mod.rs`, docs/ACCOUNTS.md › Collaboration):
    // local to this device, never synced; each device rebuilds them from
    // the server. Saved versions say who made them.
    crate::collab::MIGRATION,
    // Suggested edits (issue #46, the review queue): a version replaced
    // by an edit an agent suggested and the person applied says which
    // agent suggested it.
    r#"
ALTER TABLE revisions ADD COLUMN agent TEXT;
"#,
    // Attachment ownership, locally derived from note bodies or sealed
    // notes' app-only `kansoFiles` lists.
    r#"
CREATE TABLE note_files (
    note_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    PRIMARY KEY (note_id, file_id)
) WITHOUT ROWID;
CREATE INDEX note_files_file_id ON note_files (file_id);
"#,
    // Resurfacing related notes while writing (`embed.rs`, `passages.rs`,
    // `meaning/resurface.rs`): local to this device like the embeddings,
    // never synced, emptied with the library. A note's passages with an
    // int8 vector each (a little-endian f32 scale, then `dim` numbers),
    // keyed like the note's own embedding; and the notes a person said
    // are not related to another, which the twin banner uses too.
    r#"
CREATE TABLE note_passages (
    note_id TEXT NOT NULL,
    model TEXT NOT NULL,
    ord INTEGER NOT NULL,
    -- The byte range in the note's body.
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    dim INTEGER NOT NULL,
    vector BLOB NOT NULL,
    PRIMARY KEY (note_id, model, ord)
) WITHOUT ROWID;

CREATE TABLE resurface_dismissed (
    note_id TEXT NOT NULL,
    other_id TEXT NOT NULL,
    at INTEGER NOT NULL,
    PRIMARY KEY (note_id, other_id)
) WITHOUT ROWID;
"#,
    // Inbox triage (`triage.rs`): suggestions a person rejected, so they
    // do not come back. Local to this device, never synced, and emptied
    // with the library.
    r#"
CREATE TABLE triage_dismissed (
    note_id TEXT NOT NULL,
    field TEXT NOT NULL,
    value TEXT NOT NULL,
    at INTEGER NOT NULL,
    PRIMARY KEY (note_id, field, value)
) WITHOUT ROWID;
"#,
    // Open loops (issue #194): the loops found in note bodies (derived,
    // rebuilt like `note_tasks`), and the local history of what was
    // finished, which a rebuild must not lose, so it is not derived.
    r#"
CREATE TABLE note_loops (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    line INTEGER NOT NULL,
    kind TEXT NOT NULL,
    text TEXT NOT NULL,
    key TEXT NOT NULL,
    due TEXT,
    source TEXT NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX note_loops_key ON note_loops (key);
CREATE TABLE finished_items (
    note_id TEXT NOT NULL,
    key TEXT NOT NULL,
    kind TEXT NOT NULL,
    text TEXT NOT NULL,
    at INTEGER NOT NULL,
    PRIMARY KEY (note_id, key)
) WITHOUT ROWID;
CREATE INDEX finished_items_at ON finished_items (at);
"#,
    // Overview's open-loop scan orders by note activity. Keep that timestamp
    // in the derived rows so SQLite can satisfy the order from this index.
    r#"
ALTER TABLE note_loops ADD COLUMN note_updated_at INTEGER NOT NULL DEFAULT 0;
CREATE INDEX note_loops_updated ON note_loops (note_updated_at DESC, note_id, ord);
"#,
    // Flashcards (issue #198): the `question :: answer` lines in note
    // bodies (derived, rebuilt like `note_tasks`); the schedule of each
    // card and the log of answers, both local caches rebuilt from the
    // synced `review` documents and never synced themselves.
    r#"
CREATE TABLE IF NOT EXISTS note_cards (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    card_key TEXT NOT NULL,
    line INTEGER NOT NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS note_cards_key ON note_cards (card_key);
CREATE TABLE IF NOT EXISTS card_state (
    card_key TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL,
    due_ms INTEGER NOT NULL,
    stability REAL NOT NULL,
    difficulty REAL NOT NULL,
    reps INTEGER NOT NULL,
    lapses INTEGER NOT NULL,
    phase TEXT NOT NULL,
    step INTEGER NOT NULL,
    last_ms INTEGER NOT NULL,
    suspended INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS card_state_due ON card_state (due_ms) WHERE suspended = 0;
CREATE INDEX IF NOT EXISTS card_state_note ON card_state (note_id);
-- A note's review documents are found by this, not by reading every one.
CREATE INDEX IF NOT EXISTS docs_review_note ON docs (json_extract(json, '$.noteId')) WHERE type = 'review';
CREATE TABLE IF NOT EXISTS card_log (
    id TEXT PRIMARY KEY NOT NULL,
    card_key TEXT NOT NULL,
    note_id TEXT NOT NULL,
    at INTEGER NOT NULL,
    grade INTEGER NOT NULL
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS card_log_at ON card_log (at);
CREATE INDEX IF NOT EXISTS card_log_card ON card_log (card_key);
CREATE INDEX IF NOT EXISTS card_log_note ON card_log (note_id);
"#,
    // Ask with receipts (issue #199, `passage_embed.rs`): a vector for each
    // passage of a note longer than the text its own vector is made from,
    // so the tail of a long note can be found by meaning. Local to this
    // device like `note_embeddings`, never synced, emptied with the library
    // and when related notes are turned off. `start_line` and `end_line`
    // are 0-based lines of the body, as `kanso_core::passages` counts them.
    r#"
CREATE TABLE IF NOT EXISTS passage_embeddings (
    note_id TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    model TEXT NOT NULL,
    dim INTEGER NOT NULL,
    vector BLOB NOT NULL,
    content_hash TEXT NOT NULL,
    PRIMARY KEY (note_id, start_line, model)
) WITHOUT ROWID;
"#,
    // People and projects (issue #200, `entity_index.rs`, `entities.rs`):
    // the notes marked `kanso: person` / `kanso: project`, the names each
    // answers to, and every line of every note that mentions one. Derived
    // from note bodies like `note_links`, local to this device, never
    // synced, rebuilt when `DERIVED_VERSION` moves. `entity_candidates`
    // holds what detection suggests, and `entity_dismissed` the names the
    // person said no to ("never") or "not now" to ("later", with the
    // number of notes it had then); neither is synced from here (the
    // "never" names are also kept in the notebook the pages live in).
    r#"
CREATE TABLE IF NOT EXISTS entities (
    note_id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS entity_names (
    name TEXT NOT NULL,
    note_id TEXT NOT NULL,
    PRIMARY KEY (name, note_id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS entity_names_note ON entity_names (note_id);
CREATE TABLE IF NOT EXISTS note_mentions (
    note_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    line INTEGER NOT NULL,
    how TEXT NOT NULL,
    PRIMARY KEY (note_id, entity_id, line)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS note_mentions_entity ON note_mentions (entity_id);
CREATE TABLE IF NOT EXISTS entity_candidates (
    name TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    notes INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,
    forms TEXT NOT NULL,
    score REAL NOT NULL,
    alias_of TEXT
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS entity_dismissed (
    name TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    notes INTEGER NOT NULL DEFAULT 0,
    at INTEGER NOT NULL
) WITHOUT ROWID;
"#,
    // In-app feedback's outbox (issue #232): local, never synced, no note content.
    r#"
CREATE TABLE IF NOT EXISTS feedback_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    payload TEXT NOT NULL,
    kind TEXT NOT NULL,
    preview TEXT NOT NULL,
    screenshot_path TEXT,
    screenshot_sent INTEGER NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'queued' CHECK (state IN ('queued', 'sent', 'deleting')),
    server_status TEXT,
    issue INTEGER,
    duplicate_of INTEGER,
    pr INTEGER,
    release TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_try_ms INTEGER NOT NULL DEFAULT 0,
    created_ms INTEGER NOT NULL,
    -- Who sent it: `anonymous` or `account:<id>`. Only that reporter can act on it.
    sender TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;
"#,
    // Overview (issue #240, `overview.rs`, `revisit.rs`): three local tables
    // and one derived one. `writing_places` holds where the person last
    // stopped writing in a note (a line and a column, never text), kept per
    // note as one session at a time; `loop_feedback` the dismissals and
    // ticks that teach Overview which phrase families to stop guessing;
    // `waiting_snoozed` the items "Not yet" put off. None of them is synced,
    // backed up or sent anywhere, and all are emptied with the library.
    // `note_revisit` is derived from the `kansoRevisit` field like
    // `note_tasks` and rebuilt when `DERIVED_VERSION` moves.
    r#"
CREATE TABLE IF NOT EXISTS writing_places (
    note_id TEXT PRIMARY KEY NOT NULL,
    line INTEGER NOT NULL,
    col INTEGER NOT NULL,
    anchor_hash TEXT NOT NULL,
    session_start INTEGER NOT NULL,
    session_end INTEGER NOT NULL,
    session_chars INTEGER NOT NULL,
    source TEXT NOT NULL,
    hidden_at INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS writing_places_end ON writing_places (session_end);
CREATE TABLE IF NOT EXISTS loop_feedback (
    family TEXT NOT NULL,
    at INTEGER NOT NULL,
    what TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS loop_feedback_family ON loop_feedback (family, at);
CREATE TABLE IF NOT EXISTS waiting_snoozed (
    key TEXT PRIMARY KEY NOT NULL,
    until INTEGER NOT NULL
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS note_revisit (
    note_id TEXT PRIMARY KEY NOT NULL,
    on_day TEXT NOT NULL,
    set_at INTEGER NOT NULL,
    why TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS note_revisit_day ON note_revisit (on_day);
"#,
    // Text in PDFs (issue #43 step 7): the words on each page of a PDF,
    // extracted on this device and never synced, and what each PDF's
    // extraction was made from. `file_text` mirrors the joined pages.
    r#"
CREATE TABLE IF NOT EXISTS pdf_state (
    file_id TEXT PRIMARY KEY NOT NULL,
    -- The MD5 of the bytes the pages were read from.
    md5 TEXT NOT NULL,
    pages INTEGER NOT NULL,
    engine TEXT NOT NULL,
    at INTEGER NOT NULL
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS pdf_page_text (
    file_id TEXT NOT NULL,
    -- From 1.
    page INTEGER NOT NULL,
    text TEXT NOT NULL,
    PRIMARY KEY (file_id, page)
) WITHOUT ROWID;
"#,
    // API keys (issue #243): the keys that open the local API. Local, never
    // synced, so a key works on one computer only. Only the SHA-256 of a key
    // is stored; the key itself is shown once, when it is made.
    r#"
CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    client TEXT NOT NULL,
    token_hash BLOB NOT NULL UNIQUE,
    access TEXT NOT NULL CHECK (access IN ('read', 'suggest', 'write')),
    books TEXT,
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    revoked_at INTEGER
) WITHOUT ROWID;
"#,
    // Shared canvases (issue #45): what a shared note is, text or canvas,
    // kept in its row when the share is made so a body that later fails to
    // parse cannot change the kind of document the engine merges.
    r#"
ALTER TABLE collab_notes ADD COLUMN kind TEXT NOT NULL DEFAULT 'text';
"#,
    // Scanned PDF pages (issue #43 step 8): the words read from the pages
    // of a PDF so far, kept on this device until every scanned page is
    // read, so reading can be paused or interrupted and go on where it
    // stopped. The finished set goes into the file's `kansoPdfOcr` once.
    r#"
CREATE TABLE IF NOT EXISTS pdf_ocr_pending (
    file_id TEXT NOT NULL,
    -- The MD5 of the PDF the page was read from.
    md5 TEXT NOT NULL,
    -- From 1.
    page INTEGER NOT NULL,
    -- The page's `OcrPage` as JSON.
    json TEXT NOT NULL,
    PRIMARY KEY (file_id, page)
) WITHOUT ROWID;
"#,
    // PDFs a note linked when it was locked (issue #43 step 8), on this
    // device: the words read from their scans are dropped once no note
    // outside a lock links them, and never taken again while that holds.
    // File ids only; a locked note's own list of links is sealed with it.
    r#"
CREATE TABLE IF NOT EXISTS pdf_locked_files (
    file_id TEXT PRIMARY KEY
) WITHOUT ROWID;
"#,
    r#"
CREATE TABLE IF NOT EXISTS note_events (
    note_id TEXT PRIMARY KEY NOT NULL,
    uid TEXT NOT NULL,
    instance_start INTEGER
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS note_events_uid_start ON note_events (uid, instance_start);
"#,
    // Shared notebooks (`collab/books.rs`, issue #390): local to this device,
    // never synced, like shared notes'.
    crate::collab::books::MIGRATION,
];

/// A note's body as everything but the app reads it: empty for a locked
/// note, and for any note in a locked notebook. The argument is the `docs`
/// table's alias with its dot (`"d."`), or `""`.
macro_rules! readable_body {
    ($d:literal) => {
        concat!(
            "(CASE WHEN json_extract(",
            $d,
            "json, '$.kansoLock') IS NULL AND IFNULL(",
            $d,
            "book_id, '') NOT IN (SELECT id FROM locked_books) THEN json_extract(",
            $d,
            "json, '$.body') ELSE '' END)"
        )
    };
}
pub(crate) use readable_body;

/// Whether a note is read as locked: sealed itself, or in a locked notebook.
macro_rules! note_locked {
    ($d:literal) => {
        concat!(
            "(json_extract(",
            $d,
            "json, '$.kansoLock') IS NOT NULL OR IFNULL(",
            $d,
            "book_id, '') IN (SELECT id FROM locked_books))"
        )
    };
}
pub(crate) use note_locked;

/// Whether a note is tagged `#ai-exclude` (by the tag's name, however it is
/// written): the same rule retrieval and the agents' notes server apply.
/// The argument is the `docs` table's alias with its dot (`"d."`).
macro_rules! ai_excluded {
    ($d:literal) => {
        concat!(
            "EXISTS (SELECT 1 FROM note_tags nt JOIN docs t ON t.id = nt.tag_id
                WHERE nt.note_id = ",
            $d,
            "id AND t.deleted = 0 AND kanso_ai_exclude(json_extract(t.json, '$.name')))"
        )
    };
}
pub(crate) use ai_excluded;

/// Whether a live note outside the trash may be embedded, read for its
/// words by the discovery features and shown to them: it isn't locked
/// (sealed, or in a locked notebook) and isn't tagged `#ai-exclude`.
macro_rules! discoverable {
    ($d:literal) => {
        concat!(
            "(",
            $d,
            "type = 'note' AND ",
            $d,
            "deleted = 0 AND IFNULL(",
            $d,
            "book_id, '') != 'trash' AND NOT ",
            $crate::schema::note_locked!($d),
            " AND NOT ",
            $crate::schema::ai_excluded!($d),
            ")"
        )
    };
}
pub(crate) use discoverable;

pub fn connect(path: &Path, key: Option<&LibraryKey>) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // The key comes first: before WAL is chosen or anything is read.
    crate::library_key::unlock(&conn, key)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    // Deleted rows are overwritten, not just unlinked: a note's text gone
    // when it is locked (or deleted) doesn't stay in free pages.
    conn.pragma_update(None, "secure_delete", "ON")?;
    register_functions(&conn)?;
    Ok(conn)
}

/// The SQL functions the schema's indexes and queries call. A connection
/// that copies a library (encryption, issue #243) needs them too.
pub(crate) fn register_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "kanso_contains",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let haystack = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            let needle = ctx.get::<Option<String>>(1)?.unwrap_or_default();
            Ok(kanso_core::query::contains_ci(&haystack, &needle))
        },
    )?;
    // The part of a body around a search term's first match, when its
    // excerpt can't reach it from the start.
    conn.create_scalar_function(
        "kanso_window",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let body = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            let term = ctx.get::<Option<String>>(1)?.unwrap_or_default();
            Ok(kanso_core::excerpt::match_window(&body, &term).map(str::to_owned))
        },
    )?;
    // The front matter of a note body, without its `---` lines: what a
    // query view reads when it doesn't need the whole body.
    conn.create_scalar_function(
        "kanso_front_matter",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let body = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            Ok(kanso_core::properties::front_matter(&body)
                .unwrap_or_default()
                .to_owned())
        },
    )?;
    // Whether a tag's name is `#ai-exclude`.
    conn.create_scalar_function(
        "kanso_ai_exclude",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let name = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            Ok(kanso_core::tags::is_ai_exclude(&name))
        },
    )?;
    // A title as wiki-links match it.
    conn.create_scalar_function(
        "kanso_norm_title",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let title = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            Ok(kanso_core::links::normalize_title(&title))
        },
    )?;
    Ok(())
}

pub fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        conn.execute_batch(&format!(
            "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
            index + 1
        ))?;
    }
    repair_note_loops_updated_at(conn)?;
    Ok(())
}

/// 0.10.0 put the `note_loops.note_updated_at` migration in the middle of
/// [`MIGRATIONS`] rather than at its end, so a library made by 0.9.x was
/// already past its index and skipped it, and then failed to open
/// ("table note_loops has no column named note_updated_at"). Adds the
/// column and its index wherever they are missing; the derived loop rows
/// get the same default of 0 the original migration gave them.
fn repair_note_loops_updated_at(conn: &Connection) -> Result<()> {
    let has_table: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'note_loops')",
        [],
        |row| row.get(0),
    )?;
    if !has_table {
        return Ok(());
    }
    let has_column: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM pragma_table_info('note_loops') WHERE name = 'note_updated_at')",
        [],
        |row| row.get(0),
    )?;
    if has_column {
        return Ok(());
    }
    conn.execute_batch(
        "BEGIN;
         ALTER TABLE note_loops ADD COLUMN note_updated_at INTEGER NOT NULL DEFAULT 0;
         CREATE INDEX IF NOT EXISTS note_loops_updated ON note_loops (note_updated_at DESC, note_id, ord);
         COMMIT;",
    )?;
    Ok(())
}

pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// The `meta` key holding the highest change sequence handed out before
/// [`crate::Store::replace_library`] emptied `docs`, so sequences never
/// repeat and API change feeds keep moving forward.
pub const SEQ_FLOOR_KEY: &str = "store.seq_floor";

/// The highest change sequence handed out so far.
pub fn last_seq(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT max(COALESCE((SELECT MAX(seq) FROM docs), 0),
                    COALESCE((SELECT CAST(value AS INTEGER) FROM meta WHERE key = ?1), 0))",
        [SEQ_FLOOR_KEY],
        |row| row.get(0),
    )?)
}

/// The next value of the store-wide change sequence.
pub fn next_seq(conn: &Connection) -> Result<i64> {
    Ok(last_seq(conn)? + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(conn: &Connection) -> i64 {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
        conn.prepare(&format!(
            "SELECT 1 FROM pragma_table_info('{table}') WHERE name = ?"
        ))
        .unwrap()
        .exists([column])
        .unwrap()
    }

    /// A new database ends on the last version, one per migration, with
    /// every phase's tables.
    #[test]
    fn a_new_database_takes_every_migration() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_passages", "end_byte"));
        assert!(has_column(&conn, "resurface_dismissed", "other_id"));
        assert!(has_column(&conn, "note_tasks", "due"));
        assert!(has_column(&conn, "revisions", "batch_rev"));
        assert!(has_column(&conn, "note_embeddings", "content_hash"));
        assert!(has_column(&conn, "file_text", "md5"));
        // Running it again changes nothing.
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_files", "file_id"));
        assert!(has_column(&conn, "revisions", "author"));
        assert!(has_column(&conn, "revisions", "agent"));
        assert!(has_column(&conn, "triage_dismissed", "value"));
    }

    /// A 0.9.x library already past the index 0.10.0 gave the
    /// `note_loops.note_updated_at` migration skipped it; opening it adds
    /// the column and its index.
    #[test]
    fn library_missing_note_loops_updated_at_is_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        migrate(&conn).unwrap();
        conn.execute_batch(
            "DROP INDEX note_loops_updated;
             ALTER TABLE note_loops DROP COLUMN note_updated_at;",
        )
        .unwrap();
        assert!(!has_column(&conn, "note_loops", "note_updated_at"));
        migrate(&conn).unwrap();
        assert!(has_column(&conn, "note_loops", "note_updated_at"));
        conn.execute(
            "INSERT INTO note_loops (note_id, ord, line, kind, text, key, due, source, note_updated_at)
             VALUES ('n', 0, 1, 'k', 't', 'key', NULL, 's', 5)",
            [],
        )
        .unwrap();
        // Running again changes nothing.
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
    }

    /// A database on version 5 (find and replace) moves to 6 (discovery)
    /// keeping what it had.
    #[test]
    fn version_five_moves_to_six() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(5).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 5);
        assert!(!has_column(&conn, "note_embeddings", "vector"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "revisions", "batch"));
        assert!(has_column(&conn, "note_embeddings", "vector"));
        assert!(has_column(&conn, "file_text", "text"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
    }

    /// A database pinned at version 12 (resurfacing) moves to 13 (Inbox
    /// triage) keeping what it had.
    #[test]
    fn version_twelve_moves_to_thirteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(12).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 12);
        assert!(!has_column(&conn, "triage_dismissed", "value"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "triage_dismissed", "value"));
        assert!(has_column(&conn, "note_loops", "key"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
    }

    /// A database pinned at version 13 (Inbox triage) moves to 14 (open
    /// loops) keeping what it had, and the loops tables work.
    #[test]
    fn version_thirteen_moves_to_fourteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(13).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 13);
        assert!(!has_column(&conn, "note_loops", "key"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_loops", "key"));
        assert!(has_column(&conn, "finished_items", "at"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        conn.execute(
            "INSERT INTO finished_items (note_id, key, kind, text, at) VALUES ('n', 'k', 'ask', 't', 1)",
            [],
        )
        .unwrap();
    }

    /// A database pinned at version 14 (open loops) moves to 15
    /// (flashcards) keeping what it had, and the card tables work.
    #[test]
    fn version_fourteen_moves_to_fifteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(14).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 14);
        assert!(!has_column(&conn, "note_cards", "card_key"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_cards", "card_key"));
        assert!(has_column(&conn, "card_state", "due_ms"));
        assert!(has_column(&conn, "card_log", "grade"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        // Running it again changes nothing.
        migrate(&conn).unwrap();
    }

    /// A database pinned at version 15 (flashcards) moves to head (Ask with
    /// receipts) keeping what it had, and the passage vectors table works.
    #[test]
    fn version_fifteen_moves_to_sixteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(15).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 15);
        assert!(!has_column(&conn, "passage_embeddings", "vector"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "passage_embeddings", "content_hash"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        conn.execute(
            "INSERT INTO passage_embeddings VALUES ('n', 0, 3, 'm', 1, x'00000000', 'h')",
            [],
        )
        .unwrap();
        // Running a migration that was already applied changes nothing.
        conn.execute_batch(MIGRATIONS[15]).unwrap();
    }

    /// A database pinned at version 16 (Ask with receipts) moves to head
    /// (people and projects) keeping what it had, and the entity tables
    /// work. Running the migration again changes nothing.
    #[test]
    fn version_sixteen_moves_to_seventeen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(16).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), 16);
        assert!(!has_column(&conn, "note_mentions", "entity_id"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_mentions", "entity_id"));
        assert!(has_column(&conn, "entity_names", "name"));
        assert!(has_column(&conn, "entity_candidates", "alias_of"));
        assert!(has_column(&conn, "entity_dismissed", "kind"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        conn.execute(
            "INSERT INTO entities VALUES ('note:a', 'person', 'sam')",
            [],
        )
        .unwrap();
        conn.execute_batch(MIGRATIONS[16]).unwrap();
    }

    /// A database one version before the feedback outbox moves to head
    /// keeping what it had, and the outbox works. Running it again changes nothing.
    #[test]
    fn the_feedback_outbox_migration_moves_from_the_version_before() {
        let at = MIGRATIONS
            .iter()
            .position(|sql| sql.contains("feedback_outbox"))
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(at).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        assert_eq!(version(&conn), at as i64);
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "feedback_outbox", "next_try_ms"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        conn.execute(
            "INSERT INTO feedback_outbox (id, payload, kind, preview, created_ms) VALUES ('fb_a', '{}', 'bug', 'p', 1)",
            [],
        )
        .unwrap();
        conn.execute_batch(MIGRATIONS[at]).unwrap();
    }
}

#[cfg(test)]
mod overview_tests {
    use super::*;

    fn has_table(conn: &Connection, table: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
            [table],
            |_| Ok(()),
        )
        .optional()
        .unwrap()
        .is_some()
    }

    /// A database pinned at version 17 (people and projects) moves to head
    /// (Overview and meeting-note index) keeping what it had, and the new tables work. Running the
    /// migration again changes nothing.
    #[test]
    fn version_seventeen_moves_to_nineteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db"), None).unwrap();
        for (index, sql) in MIGRATIONS.iter().take(17).enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                index + 1
            ))
            .unwrap();
        }
        let before: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(before, 17);
        assert!(!has_table(&conn, "writing_places"));
        set_meta(&conn, "kept", "yes").unwrap();
        migrate(&conn).unwrap();
        let after: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(after, MIGRATIONS.len() as i64);
        for table in [
            "writing_places",
            "loop_feedback",
            "waiting_snoozed",
            "note_revisit",
            "note_events",
        ] {
            assert!(has_table(&conn, table), "{table}");
        }
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
        conn.execute(
            "INSERT INTO writing_places VALUES ('note:a', 3, 4, 'h', 1, 2, 50, 'ui', 0)",
            [],
        )
        .unwrap();
        conn.execute_batch(MIGRATIONS[18]).unwrap();
    }
}

/// Released migrations are frozen. A library's `user_version` counts how
/// many of [`MIGRATIONS`] it has run, so changing, removing or inserting
/// one before the end makes upgraded libraries skip or repeat work (0.10.0
/// inserted one mid-list and 0.9.x libraries stopped opening). New
/// migrations go at the end; once released, add their hash here.
#[cfg(test)]
mod frozen_migrations {
    use sha2::{Digest, Sha256};

    const RELEASED: &[&str] = &[
        "4db9515b7311029a",
        "64dbe43a1c31b17d",
        "08ca39e00848cb0f",
        "2735a48858a0afa3",
        "6c3494ceca4a8c54",
        "8b9fb28849310a94",
        "3d0ae7f12fd98856",
        "62bd7c9d3203b580",
        "1b489df3cabba75a",
        "9d1c30ad36455bd0",
        "2c15bc2d82305d54",
        "bee33aff5944b716",
        "47dc3d20ccd190f1",
        "9daf465a993a376d",
        "f3b7f978fd2a5d2c",
        "eb4ba3b4f6e6ea1f",
        "6183fc6cae6b40f4",
        "dfeed430aa590885",
        "0498b583344c3d7f",
        "98703742c0a98525",
        "e7cb7a40428812c0",
        "6498d51de657003d",
        "e32cd11b1d19ed18",
        "867d921b798c5865",
        "4ebda7fee54b7e70",
        "d59e69a5b2acdf78",
    ];

    fn hash(sql: &str) -> String {
        let h: String = Sha256::digest(sql.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        h[..16].to_string()
    }

    #[test]
    fn released_migrations_are_unchanged_and_in_order() {
        assert!(
            super::MIGRATIONS.len() >= RELEASED.len(),
            "a released migration was removed"
        );
        for (index, want) in RELEASED.iter().enumerate() {
            assert_eq!(
                hash(super::MIGRATIONS[index]),
                *want,
                "migration {index} changed or a new one was inserted before it; \
                 released migrations are frozen, add new ones at the end"
            );
        }
    }
}
