-- PDFs up to 50 MiB (#43): the stored file is a 50 MiB plaintext container
-- plus up to 4 KiB of framing and authentication overhead. The server sees
-- only sealed bytes, so this is its one per-file limit; clients keep every
-- other attachment at 10 MiB. Rebuild the table to widen the CHECK, as 0013
-- did, preserving every size, upload timestamp and quota trigger.
CREATE TABLE account_files_next (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (substr(id, 1, 5) = 'file:' AND length(id) = 27 AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*'),
    size INTEGER NOT NULL CHECK (size BETWEEN 41 AND 52432937),
    uploaded_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id)
);

INSERT INTO account_files_next (account_id, id, size, uploaded_at)
SELECT account_id, id, size, uploaded_at FROM account_files;

DROP TABLE account_files;
ALTER TABLE account_files_next RENAME TO account_files;

CREATE TRIGGER uploads_global BEFORE INSERT ON account_files
WHEN coalesce((SELECT CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'upload_bytes'), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day FROM defaults), 0) - NEW.size
 AND coalesce((SELECT CASE WHEN window_start = NEW.uploaded_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'uploads-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:uploads-global'); END;

CREATE TRIGGER uploads_count_insert AFTER INSERT ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

CREATE TRIGGER uploads_count_update AFTER UPDATE OF uploaded_at ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

-- Account file quota: 500 MiB -> 1 GiB, so a few large PDFs do not use up an
-- account, and the server-wide file cap 50 GB -> 60 GB so the default quotas
-- of a full server (50 accounts) are still not overcommitted. Only a database
-- still at the shipped defaults moves; an operator's own values are kept, and
-- accounts with their own quota_file_bytes are untouched.
UPDATE settings SET value = '1073741824' WHERE name = 'quota_file_bytes' AND value = '524288000';
UPDATE settings SET value = '60000000000' WHERE name = 'global_file_bytes' AND value = '50000000000';
