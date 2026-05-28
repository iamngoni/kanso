-- Kanso Cloud event log. Per-user, append-only, ordered by a global sequence.
-- Single Postgres instance; shard/partition by user_id when scale requires it.

CREATE TABLE events (
    server_sequence  BIGSERIAL    PRIMARY KEY,
    user_id          TEXT         NOT NULL,
    event_id         UUID         NOT NULL UNIQUE,
    origin_device_id TEXT         NOT NULL,
    entity_type      TEXT         NOT NULL,
    entity_id        TEXT         NOT NULL,
    operation        TEXT         NOT NULL,
    payload          JSONB        NOT NULL,
    local_sequence   BIGINT       NOT NULL,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Pull is "the user's events after sequence N", so scope the index by user.
CREATE INDEX idx_events_user_seq ON events (user_id, server_sequence);
