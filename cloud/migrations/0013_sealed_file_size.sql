-- Sealed attachments (#44): the stored file is a 10 MiB plaintext container
-- plus up to 4 KiB of framing and authentication overhead. Rebuild the table
-- so existing D1 databases accept the same maximum as the Worker, while
-- preserving every size, upload timestamp and quota trigger.
--
-- SQLite cannot widen a CHECK constraint in place. account_files has no child
-- tables, so this copy, swap and trigger recreation is safe. The migration is
-- applied once by wrangler; existing rows are copied byte for byte.
CREATE TABLE account_files_next (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (substr(id, 1, 5) = 'file:' AND length(id) = 27 AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*'),
    size INTEGER NOT NULL CHECK (size BETWEEN 41 AND 10489897),
    uploaded_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id)
);

INSERT INTO account_files_next (account_id, id, size, uploaded_at)
SELECT account_id, id, size, uploaded_at FROM account_files;

DROP TABLE account_files;
ALTER TABLE account_files_next RENAME TO account_files;

-- Server-wide upload bytes per UTC day, weighted by size and counted on every
-- upload, replacements included. These are the triggers from 0002_accounts.sql,
-- recreated after the table swap.
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
