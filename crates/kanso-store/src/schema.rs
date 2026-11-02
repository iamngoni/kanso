//! Connection setup and schema migrations.

use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

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

pub fn connect(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    // Deleted rows are overwritten, not just unlinked: a note's text gone
    // when it is locked (or deleted) doesn't stay in free pages.
    conn.pragma_update(None, "secure_delete", "ON")?;
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
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        conn.execute_batch(&format!(
            "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
            index + 1
        ))?;
    }
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
        let conn = connect(&dir.path().join("k.db")).unwrap();
        migrate(&conn).unwrap();
        assert_eq!(MIGRATIONS.len(), 15);
        assert_eq!(version(&conn), MIGRATIONS.len() as i64);
        assert!(has_column(&conn, "note_passages", "end_byte"));
        assert!(has_column(&conn, "resurface_dismissed", "other_id"));
        assert!(has_column(&conn, "note_tasks", "due"));
        assert!(has_column(&conn, "revisions", "batch_rev"));
        assert!(has_column(&conn, "note_embeddings", "content_hash"));
        assert!(has_column(&conn, "file_text", "md5"));
        // Running it again changes nothing.
        migrate(&conn).unwrap();
        assert_eq!(version(&conn), 15);
        assert!(has_column(&conn, "note_files", "file_id"));
        assert!(has_column(&conn, "revisions", "author"));
        assert!(has_column(&conn, "revisions", "agent"));
        assert!(has_column(&conn, "triage_dismissed", "value"));
    }

    /// A database on version 5 (find and replace) moves to 6 (discovery)
    /// keeping what it had.
    #[test]
    fn version_five_moves_to_six() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db")).unwrap();
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
        assert_eq!(version(&conn), 15);
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
        let conn = connect(&dir.path().join("k.db")).unwrap();
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
        assert_eq!(version(&conn), 15);
        assert!(has_column(&conn, "triage_dismissed", "value"));
        assert!(has_column(&conn, "note_loops", "key"));
        assert_eq!(get_meta(&conn, "kept").unwrap().as_deref(), Some("yes"));
    }

    /// A database pinned at version 13 (Inbox triage) moves to 14 (open
    /// loops) keeping what it had, and the loops tables work.
    #[test]
    fn version_thirteen_moves_to_fourteen() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(&dir.path().join("k.db")).unwrap();
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
        assert_eq!(MIGRATIONS.len(), 15);
        assert_eq!(version(&conn), 15);
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
        let conn = connect(&dir.path().join("k.db")).unwrap();
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
}
