-- In-app feedback (docs/FEEDBACK.md). Raw text and app info are private: never sent to GitHub.
-- A row holds the account ID (signed in) or a keyed reporter hash (anonymous), never an IP or a
-- device ID. `report` is the validated body, re-serialized, never the body that arrived. Screenshots
-- live in R2 under `feedback/<id>/screenshot`, a prefix outside `accounts/`, so account quotas and
-- reconciliation never count them.
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY CHECK (id GLOB 'fb_*' AND length(id) = 29),
    account_id TEXT NULL,
    reporter TEXT NULL CHECK (reporter IS NULL OR (reporter GLOB 'r_*' AND length(reporter) = 34)),
    created_at INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('bug', 'idea', 'question')),
    report TEXT NOT NULL CHECK (json_valid(report) AND length(report) <= 49152),
    has_screenshot INTEGER NOT NULL DEFAULT 0 CHECK (has_screenshot IN (0, 1)),
    screenshot_state TEXT NOT NULL DEFAULT 'none' CHECK (screenshot_state IN ('none', 'awaited', 'stored', 'expired')),
    status TEXT NOT NULL DEFAULT 'received'
        CHECK (status IN ('received', 'noise', 'duplicate', 'filed', 'building', 'pr_open', 'merged', 'released', 'closed')),
    triage TEXT NULL CHECK (triage IS NULL OR json_valid(triage)),
    triage_attempts INTEGER NOT NULL DEFAULT 0,
    issue INTEGER NULL,
    duplicate_of INTEGER NULL,
    pr INTEGER NULL,
    merge_sha TEXT NULL,
    release TEXT NULL,
    credit_name TEXT NULL,
    -- Set when the retention job blanked the raw text and removed the screenshot.
    purged_at INTEGER NULL,
    CHECK ((account_id IS NULL) <> (reporter IS NULL))
);
CREATE INDEX IF NOT EXISTS feedback_account ON feedback (account_id, created_at);
CREATE INDEX IF NOT EXISTS feedback_reporter ON feedback (reporter, created_at);
CREATE INDEX IF NOT EXISTS feedback_issue ON feedback (issue);
CREATE INDEX IF NOT EXISTS feedback_pr ON feedback (pr);
CREATE INDEX IF NOT EXISTS feedback_pending ON feedback (status, triage_attempts) WHERE status = 'received';

-- The status chain My feedback shows. It outlives the raw text.
CREATE TABLE IF NOT EXISTS feedback_events (
    feedback_id TEXT NOT NULL REFERENCES feedback (id) ON DELETE CASCADE,
    at INTEGER NOT NULL,
    status TEXT NOT NULL,
    detail TEXT NULL,
    PRIMARY KEY (feedback_id, at, status)
);

-- Reports whose screenshot is still in R2 after the row went (a deletion, an account deletion):
-- the cron empties `feedback/<id>/` and drops the row. `purges` only accepts account prefixes.
CREATE TABLE IF NOT EXISTS feedback_purges (
    feedback_id TEXT PRIMARY KEY NOT NULL,
    requested_at INTEGER NOT NULL
);

-- "Reported by" credit lines to edit out of filed issues (a deleted report, a deleted account).
-- The cron retries each until GitHub accepts the edit, so an outage delays it and never drops it.
CREATE TABLE IF NOT EXISTS feedback_credit_removals (
    issue INTEGER NOT NULL,
    name TEXT NOT NULL,
    requested_at INTEGER NOT NULL,
    PRIMARY KEY (issue, name)
);

-- GitHub webhook deliveries seen in the last 24 hours, so a redelivered event is a no-op.
CREATE TABLE IF NOT EXISTS feedback_deliveries (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 64),
    at INTEGER NOT NULL
);

-- Kill switch and budgets (numeric like every other setting; the admin script sets them).
-- feedback_open: 1 takes reports, 0 answers 503. A missing row refuses, like the other settings.
INSERT OR IGNORE INTO settings (name, value) VALUES
    ('feedback_open', '1'),
    ('feedback_per_day', '200'),
    ('feedback_issues_per_day', '50');
