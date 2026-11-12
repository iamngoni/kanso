-- Shared notes: who left (or declined) while the signed list still names them. Leaving deletes the
-- member row and wraps as before, but without this record the next epoch's list, which still lists
-- the leaver until the owner cuts them, recreated them as invited with a fresh wrap. Additive only
-- (0005's status CHECK stays): a row here makes the epoch route skip the member row and the wraps
-- for that account, and it goes when a list stops naming them, or when they ask to join again.
CREATE TABLE collab_left (
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    left_at INTEGER NOT NULL,
    PRIMARY KEY (share_id, account_id)
);
CREATE INDEX collab_left_by_account ON collab_left (account_id);
