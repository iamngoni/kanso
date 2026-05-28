-- Skills: first-party, inspectable, Markdown-defined agent behaviors, plus a
-- run log. Skills are local engine config (not synced through the outbox yet).

CREATE TABLE skills (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    body_markdown TEXT NOT NULL,
    scope         TEXT NOT NULL DEFAULT 'global', -- global | notebook | note | project
    enabled       INTEGER NOT NULL DEFAULT 1,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    metadata      TEXT
);

CREATE TABLE skill_runs (
    id             TEXT PRIMARY KEY,
    skill_id       TEXT NOT NULL REFERENCES skills (id),
    target_type    TEXT,
    target_id      TEXT,
    mode           TEXT NOT NULL, -- dry_run | review_changes | apply_changes
    status         TEXT NOT NULL, -- running | completed | failed | rejected
    input_snapshot TEXT,
    output_summary TEXT,
    created_at     INTEGER NOT NULL,
    completed_at   INTEGER
);
CREATE INDEX idx_skill_runs_skill ON skill_runs (skill_id);
