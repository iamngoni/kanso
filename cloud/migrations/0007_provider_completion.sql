-- The current GitHub device-flow interval is stored in seconds and grows by
-- five seconds for every slow_down response. Other providers leave it NULL.
ALTER TABLE auth_flows ADD COLUMN poll_interval_seconds INTEGER
    CHECK (poll_interval_seconds IS NULL OR poll_interval_seconds >= 5);

-- A provider completion claims its flow and records one unique attempt before
-- any session, identity or account write. The attempt marker gates those writes
-- in the same D1 batch; success is set only after the corresponding result is
-- present in that transaction.
ALTER TABLE auth_flows ADD COLUMN completion_attempt_id TEXT
    CHECK (completion_attempt_id IS NULL OR (length(completion_attempt_id) = 43
        AND completion_attempt_id NOT GLOB '*[^A-Za-z0-9_-]*'));
ALTER TABLE auth_flows ADD COLUMN completion_kind TEXT
    CHECK (completion_kind IS NULL OR completion_kind IN ('signin', 'link', 'social_signup'));
ALTER TABLE auth_flows ADD COLUMN completion_account_id TEXT
    CHECK (completion_account_id IS NULL OR (length(completion_account_id) = 34
        AND substr(completion_account_id, 1, 2) = 'a_'
        AND substr(completion_account_id, 3) NOT GLOB '*[^0-9a-f]*'));
ALTER TABLE auth_flows ADD COLUMN completion_succeeded INTEGER NOT NULL DEFAULT 0
    CHECK (completion_succeeded IN (0, 1));

CREATE UNIQUE INDEX auth_flows_completion_attempt ON auth_flows (completion_attempt_id)
    WHERE completion_attempt_id IS NOT NULL;
