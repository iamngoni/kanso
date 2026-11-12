-- Shared notebooks: the manifest and note logs (issue #390, slice 34). Additive only; no applied
-- migration changes. The logs themselves never reach D1: each notebook's manifest log and its
-- notes' logs live in the notebook's own Durable Object (BookRoom, src/book_room.ts), as a shared
-- note's does in NoteRoom. D1 learns three things:
--   - `bytes`, what the room stores, as it last reported it. Together with the owner's documents and
--     shared notes it counts against `quota_doc_bytes` (the room reads it on each append);
--   - the rooms still to purge: stopping a notebook queues its room here in the same batch that
--     deletes its rows, and the cron (or the stop itself) tells the room to delete its logs and
--     close its sockets, then drops the row;
--   - the rooms whose roster still has to be re-applied. The room enforces its own copy of who is in
--     the notebook and at what role, so a change to the members (a new list, leaving, an account's
--     deletion) queues a sync in the same batch; the row is dropped when a sync that began after the
--     change's `version` succeeded, and the cron retries until then.

ALTER TABLE book_shares ADD COLUMN bytes INTEGER NOT NULL DEFAULT 0 CHECK (bytes >= 0);

CREATE TABLE book_purges (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    requested_at INTEGER NOT NULL
);

CREATE TABLE book_room_sync (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    requested_at INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
);
