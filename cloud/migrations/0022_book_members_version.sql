-- Shared notebooks: a membership version, so the room can tell a stale roster from a current one.
-- The room keeps its own copy of who is in a notebook (0021) and is sent it by syncs that read D1
-- and can arrive out of order. `members_version` counts every change to a notebook's members (a new
-- notebook's members, a new list, accepting, leaving, an account's deletion), bumped by triggers so
-- it moves in the same D1 batch as the change, whatever statement makes it. A sync carries the
-- version it read, and the room ignores a roster older than the one it holds.

ALTER TABLE book_shares ADD COLUMN members_version INTEGER NOT NULL DEFAULT 0 CHECK (members_version >= 0);

CREATE TRIGGER book_members_version_insert AFTER INSERT ON book_members
BEGIN UPDATE book_shares SET members_version = members_version + 1 WHERE id = NEW.share_id; END;

CREATE TRIGGER book_members_version_update AFTER UPDATE ON book_members
BEGIN UPDATE book_shares SET members_version = members_version + 1 WHERE id = NEW.share_id; END;

CREATE TRIGGER book_members_version_delete AFTER DELETE ON book_members
BEGIN UPDATE book_shares SET members_version = members_version + 1 WHERE id = OLD.share_id; END;
