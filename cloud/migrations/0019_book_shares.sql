-- Shared notebooks (issue #390 › Shared notebooks: Worker). Additive only: new tables, indexes and
-- triggers; no applied migration changes (0005's CHECKs stay, so a shared note still refuses the
-- Manager role, which only notebooks have). The server stores what clients sign and wrap and never
-- sees a key: each notebook's chain of signed member lists, every member's wrapped notebook key per
-- epoch, the history links that let a member read earlier epochs, and invite links.
--
-- The member lists are append-only: a trigger refuses any list whose `rev` isn't the head's + 1,
-- any update, and any delete except while the notebook is being stopped. The caps (50 notebooks per
-- owner, 25 people per notebook, 10 live links) are triggers too, so concurrent requests can't
-- race past them; they refuse with `quota_exceeded:<kind>`, which the Worker answers with 413.

CREATE TABLE book_shares (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    owner_id TEXT NOT NULL REFERENCES accounts (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    -- The newest list's position in the chain and its link hash (hex SHA-256), the compare-and-swap key.
    head_rev INTEGER NOT NULL CHECK (head_rev >= 1),
    head_hash TEXT NOT NULL CHECK (length(head_hash) = 64 AND head_hash NOT GLOB '*[^0-9a-f]*'),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- Set in the batch that deletes the notebook, which is what lets its lists be deleted.
    stopped_at INTEGER
);
CREATE INDEX book_shares_by_owner ON book_shares (owner_id);

CREATE TRIGGER book_shares_cap BEFORE INSERT ON book_shares
WHEN (SELECT count(*) FROM book_shares WHERE owner_id = NEW.owner_id) >= 50
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:books'); END;

-- A stopped notebook's ID is retired for good, in the same table shared notes use.
CREATE TRIGGER book_shares_retire AFTER DELETE ON book_shares
BEGIN
    INSERT OR IGNORE INTO collab_retired (id, retired_at) VALUES (OLD.id, CAST(strftime('%s', 'now') AS INTEGER) * 1000);
END;

CREATE TABLE book_members (
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'manager', 'edit', 'comment', 'view')),
    -- 'left': the member left or declined, but the signed list still names them. The row stays so a later list
    -- that still names them neither re-invites them nor wraps a key to them; it goes when a list stops naming them.
    status TEXT NOT NULL CHECK (status IN ('invited', 'member', 'left')),
    invited_at INTEGER NOT NULL,
    joined_at INTEGER,
    PRIMARY KEY (share_id, account_id)
);
CREATE INDEX book_members_by_account ON book_members (account_id);

CREATE TRIGGER book_members_cap BEFORE INSERT ON book_members
WHEN (SELECT count(*) FROM book_members WHERE share_id = NEW.share_id AND account_id != NEW.account_id) >= 25
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:members'); END;

-- Each member's notebook key for each epoch they hold it, wrapped to their box key (105 bytes).
CREATE TABLE book_wraps (
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    wrapped TEXT NOT NULL CHECK (length(wrapped) = 140 AND wrapped NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, account_id, epoch)
);
CREATE INDEX book_wraps_by_account ON book_wraps (account_id);

-- The chain of signed member lists, genesis first. `hash` names the list for the next one's `prev`.
CREATE TABLE book_lists (
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    rev INTEGER NOT NULL CHECK (rev >= 1),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    signer TEXT NOT NULL CHECK (length(signer) = 34 AND signer GLOB 'a_*'),
    prev TEXT CHECK (prev IS NULL OR (length(prev) = 64 AND prev NOT GLOB '*[^0-9a-f]*')),
    hash TEXT NOT NULL CHECK (length(hash) = 64 AND hash NOT GLOB '*[^0-9a-f]*'),
    list TEXT NOT NULL CHECK (length(list) BETWEEN 1 AND 16384 AND list NOT GLOB '*[^A-Za-z0-9_-]*'),
    list_sig TEXT NOT NULL CHECK (length(list_sig) = 86 AND list_sig NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, rev),
    CHECK ((rev = 1) = (prev IS NULL))
);

CREATE TRIGGER book_lists_next BEFORE INSERT ON book_lists
WHEN NEW.rev != coalesce((SELECT max(rev) FROM book_lists WHERE share_id = NEW.share_id), 0) + 1
  OR (NEW.rev > 1 AND NEW.prev IS NOT (SELECT hash FROM book_lists WHERE share_id = NEW.share_id AND rev = NEW.rev - 1))
BEGIN SELECT RAISE(ABORT, 'rev_conflict'); END;

CREATE TRIGGER book_lists_no_update BEFORE UPDATE ON book_lists
BEGIN SELECT RAISE(ABORT, 'append_only'); END;

CREATE TRIGGER book_lists_no_delete BEFORE DELETE ON book_lists
WHEN NOT EXISTS (SELECT 1 FROM book_shares WHERE id = OLD.share_id AND stopped_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT, 'append_only'); END;

-- History links: epoch `epoch`'s link seals the previous epoch's key under a key derived from this
-- one (73 bytes), so members read earlier work. Written once with its epoch.
CREATE TABLE book_epochs (
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 2),
    history_link TEXT NOT NULL CHECK (length(history_link) = 98 AND history_link NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, epoch)
);

CREATE TRIGGER book_epochs_no_update BEFORE UPDATE ON book_epochs
BEGIN SELECT RAISE(ABORT, 'append_only'); END;

CREATE TABLE book_links (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    proof_hash TEXT NOT NULL CHECK (length(proof_hash) = 64 AND proof_hash NOT GLOB '*[^0-9a-f]*'),
    role TEXT NOT NULL CHECK (role IN ('edit', 'comment', 'view')),
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL CHECK (expires_at > created_at)
);
CREATE INDEX book_links_by_share ON book_links (share_id, expires_at);

CREATE TRIGGER book_links_cap BEFORE INSERT ON book_links
WHEN (SELECT count(*) FROM book_links WHERE share_id = NEW.share_id AND expires_at > NEW.created_at) >= 10
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:links'); END;

-- The keys each account was last listed with in this notebook's chain, kept after it is removed, so
-- a manager can't re-add a removed account with other keys (only the owner can). One lookup, not a
-- scan of the chain. No foreign key on the account: it outlives a deleted account's rows.
CREATE TABLE book_seen_keys (
    share_id TEXT NOT NULL REFERENCES book_shares (id),
    account_id TEXT NOT NULL CHECK (length(account_id) = 34 AND account_id GLOB 'a_*'),
    sign_pub TEXT NOT NULL CHECK (length(sign_pub) = 43 AND sign_pub NOT GLOB '*[^A-Za-z0-9_-]*'),
    box_pub TEXT NOT NULL CHECK (length(box_pub) = 43 AND box_pub NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, account_id)
);
