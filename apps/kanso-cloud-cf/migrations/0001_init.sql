CREATE TABLE users (
  user_id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE devices (
  device_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX idx_devices_user ON devices(user_id);

CREATE TABLE user_sequences (
  user_id TEXT PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
  value INTEGER NOT NULL
);

CREATE TABLE events (
  user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  server_sequence INTEGER NOT NULL,
  event_id TEXT NOT NULL,
  origin_device_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  operation TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  local_sequence INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(user_id, server_sequence),
  UNIQUE(user_id, event_id)
);
CREATE INDEX idx_events_user_sequence ON events(user_id, server_sequence);
CREATE INDEX idx_events_user_origin ON events(user_id, origin_device_id);

CREATE TABLE blobs (
  user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  hash TEXT NOT NULL,
  body_base64 TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(user_id, hash)
);
