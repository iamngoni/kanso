-- A signed-in device creates a short-lived, single-use code for a computer
-- without a screen. Codes are stored only as peppered hashes.
CREATE TABLE device_pairings (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL UNIQUE,
    created_by_device TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    used_at INTEGER,
    used_by_device TEXT,
    CHECK ((used_at IS NULL) = (used_by_device IS NULL))
);
CREATE INDEX device_pairings_account ON device_pairings (account_id, created_at);
