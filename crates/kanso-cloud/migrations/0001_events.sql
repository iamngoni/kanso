-- Event log: append-only, ordered by authoritative server sequence.
--
-- Single-tenant for now (per-service deployment = per-user).
-- TODO: add a `user_id` column (UUID FK → users table) when auth lands so
--       the table becomes multi-tenant and every query gains a user_id filter.

CREATE TABLE events (
    server_sequence  BIGSERIAL    PRIMARY KEY,
    event_id         UUID         NOT NULL UNIQUE,
    origin_device_id TEXT         NOT NULL,
    entity_type      TEXT         NOT NULL,
    entity_id        TEXT         NOT NULL,
    operation        TEXT         NOT NULL,
    payload          JSONB        NOT NULL,
    local_sequence   BIGINT       NOT NULL,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Fast forward-scan by sequence (the common pull query path).
CREATE INDEX idx_events_seq ON events (server_sequence);
