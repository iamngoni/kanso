-- Sharing metadata. This is local product state for note/notebook members; the
-- cloud ACL layer can mirror these resources into per-stream memberships.

CREATE TABLE shares (
    id            TEXT PRIMARY KEY,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('note', 'notebook')),
    resource_id   TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    UNIQUE (resource_type, resource_id)
);
CREATE INDEX idx_shares_resource ON shares (resource_type, resource_id);

CREATE TABLE share_members (
    id         TEXT PRIMARY KEY,
    share_id   TEXT NOT NULL REFERENCES shares (id) ON DELETE CASCADE,
    email      TEXT NOT NULL,
    role       TEXT NOT NULL CHECK (role IN ('owner', 'editor', 'viewer')),
    status     TEXT NOT NULL DEFAULT 'invited',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (share_id, email)
);
CREATE INDEX idx_share_members_share ON share_members (share_id);
