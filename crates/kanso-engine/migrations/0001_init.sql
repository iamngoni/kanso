-- Kanso engine schema, v1.
-- SQLite is canonical. Markdown is the note body. Timestamps are Unix millis
-- (INTEGER). IDs are TEXT ("<prefix>:<uuid-v7>"). Deletes are soft.

CREATE TABLE notebooks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    parent_id   TEXT REFERENCES notebooks (id),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER,
    metadata    TEXT
);

CREATE TABLE notes (
    id                  TEXT PRIMARY KEY,
    notebook_id         TEXT NOT NULL REFERENCES notebooks (id),
    title               TEXT NOT NULL,
    body_markdown       TEXT NOT NULL,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    deleted_at          INTEGER,
    pinned              INTEGER NOT NULL DEFAULT 0,
    favorite            INTEGER NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'active',
    current_revision_id TEXT,
    metadata            TEXT
);
CREATE INDEX idx_notes_notebook ON notes (notebook_id);

CREATE TABLE tags (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL REFERENCES notes (id),
    tag_id  TEXT NOT NULL REFERENCES tags (id),
    PRIMARY KEY (note_id, tag_id)
);

CREATE TABLE attachments (
    id            TEXT PRIMARY KEY,
    note_id       TEXT NOT NULL REFERENCES notes (id),
    filename      TEXT NOT NULL,
    mime_type     TEXT NOT NULL,
    size_bytes    INTEGER NOT NULL,
    content_hash  TEXT NOT NULL,
    local_path    TEXT,
    remote_key    TEXT,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    metadata      TEXT
);
CREATE INDEX idx_attachments_note ON attachments (note_id);
CREATE INDEX idx_attachments_hash ON attachments (content_hash);

CREATE TABLE sketches (
    id                    TEXT PRIMARY KEY,
    note_id               TEXT NOT NULL REFERENCES notes (id),
    title                 TEXT,
    format_version        INTEGER NOT NULL,
    data_blob             BLOB NOT NULL,
    preview_attachment_id TEXT REFERENCES attachments (id),
    created_at            INTEGER NOT NULL,
    updated_at            INTEGER NOT NULL
);
CREATE INDEX idx_sketches_note ON sketches (note_id);

CREATE TABLE note_links (
    source_note_id TEXT NOT NULL REFERENCES notes (id),
    target_ref     TEXT NOT NULL,
    link_kind      TEXT NOT NULL, -- note | sketch | attachment
    PRIMARY KEY (source_note_id, target_ref, link_kind)
);

CREATE TABLE note_tasks (
    id      TEXT PRIMARY KEY,
    note_id TEXT NOT NULL REFERENCES notes (id),
    text    TEXT NOT NULL,
    checked INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_note_tasks_note ON note_tasks (note_id);

CREATE TABLE revisions (
    id                TEXT PRIMARY KEY,
    note_id           TEXT NOT NULL REFERENCES notes (id),
    body_markdown     TEXT NOT NULL,
    metadata_snapshot TEXT,
    reason            TEXT,
    source            TEXT NOT NULL, -- user | sync | agent | import | conflict
    created_at        INTEGER NOT NULL
);
CREATE INDEX idx_revisions_note ON revisions (note_id);

CREATE TABLE sync_outbox (
    id              TEXT PRIMARY KEY, -- client UUID; idempotency key
    entity_type     TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    operation       TEXT NOT NULL,
    payload_json    TEXT NOT NULL,
    local_sequence  INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    sent_at         INTEGER,
    acknowledged_at INTEGER,
    server_sequence INTEGER,
    retry_count     INTEGER NOT NULL DEFAULT 0,
    last_error      TEXT
);
CREATE INDEX idx_outbox_pending ON sync_outbox (acknowledged_at, local_sequence);

CREATE TABLE sync_state (
    device_id                   TEXT PRIMARY KEY,
    last_pulled_server_sequence INTEGER NOT NULL DEFAULT 0,
    last_pushed_local_sequence  INTEGER NOT NULL DEFAULT 0,
    backend_id                  TEXT,
    updated_at                  INTEGER NOT NULL
);

CREATE TABLE tombstones (
    entity_type          TEXT NOT NULL,
    entity_id            TEXT NOT NULL,
    deleted_at           INTEGER NOT NULL,
    deleted_by_device_id TEXT,
    sync_version         INTEGER,
    PRIMARY KEY (entity_type, entity_id)
);

-- Single-row monotonic counter for per-device local sequence numbers.
CREATE TABLE local_sequence (
    id    INTEGER PRIMARY KEY CHECK (id = 1),
    value INTEGER NOT NULL
);
INSERT INTO local_sequence (id, value) VALUES (1, 0);

-- Full-text search index. Maintained explicitly by the engine inside the same
-- transaction as the note write (no triggers, so indexing stays in one place).
CREATE VIRTUAL TABLE notes_fts USING fts5 (
    note_id UNINDEXED,
    title,
    body,
    tokenize = 'unicode61'
);
