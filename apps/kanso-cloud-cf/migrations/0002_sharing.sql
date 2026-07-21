CREATE TABLE shares (
  user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  id TEXT NOT NULL,
  resource_type TEXT NOT NULL CHECK(resource_type IN ('note', 'notebook')),
  resource_id TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(user_id, id),
  UNIQUE(user_id, resource_type, resource_id)
);
CREATE INDEX idx_shares_user_resource ON shares(user_id, resource_type, resource_id);

CREATE TABLE share_members (
  user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  id TEXT NOT NULL,
  share_id TEXT NOT NULL,
  email TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('owner', 'editor', 'viewer')),
  status TEXT NOT NULL DEFAULT 'invited',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(user_id, id),
  UNIQUE(user_id, share_id, email),
  FOREIGN KEY(user_id, share_id) REFERENCES shares(user_id, id) ON DELETE CASCADE
);
CREATE INDEX idx_share_members_share ON share_members(user_id, share_id);
