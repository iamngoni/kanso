//! The feedback outbox (issue #232): reports are saved here first and only
//! then sent, so a report can't be lost, and a retry can't make a duplicate
//! (the id is the idempotency key).
//!
//! The table is local and never synced. It is not a note, holds no note
//! content, and nothing in it is derived from the library.

use rusqlite::{OptionalExtension, Row, params};

use crate::feedback_outbox_model::{OutboxNew, OutboxRow, OutboxState, ServerStatus};
use crate::{Result, Store};

const COLUMNS: &str = "id, payload, kind, preview, screenshot_path, screenshot_sent, state, \
     server_status, issue, duplicate_of, pr, release, attempts, next_try_ms, created_ms";

fn read(row: &Row) -> rusqlite::Result<OutboxRow> {
    let state: String = row.get(6)?;
    Ok(OutboxRow {
        id: row.get(0)?,
        payload: row.get(1)?,
        kind: row.get(2)?,
        preview: row.get(3)?,
        screenshot_path: row.get(4)?,
        screenshot_sent: row.get::<_, i64>(5)? != 0,
        state: OutboxState::parse(&state).unwrap_or(OutboxState::Queued),
        server_status: row.get(7)?,
        issue: row.get(8)?,
        duplicate_of: row.get(9)?,
        pr: row.get(10)?,
        release: row.get(11)?,
        attempts: row.get(12)?,
        next_try_ms: row.get(13)?,
        created_ms: row.get(14)?,
    })
}

impl Store {
    /// Saves a report. Saving the same id twice keeps the first.
    pub fn feedback_enqueue(&self, new: &OutboxNew) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR IGNORE INTO feedback_outbox \
                 (id, payload, kind, preview, screenshot_path, screenshot_sent, state, attempts, next_try_ms, created_ms) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, 'queued', 0, 0, ?6)",
                params![
                    new.id,
                    new.payload,
                    new.kind,
                    new.preview,
                    new.screenshot_path,
                    new.created_ms
                ],
            )?;
            Ok(())
        })
    }

    /// Every report on this device, newest first.
    pub fn feedback_all(&self) -> Result<Vec<OutboxRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {COLUMNS} FROM feedback_outbox ORDER BY created_ms DESC, id"
            ))?;
            let rows = stmt.query_map([], read)?.collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
    }

    pub fn feedback_get(&self, id: &str) -> Result<Option<OutboxRow>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!("SELECT {COLUMNS} FROM feedback_outbox WHERE id = ?1"),
                    [id],
                    read,
                )
                .optional()?)
        })
    }

    /// Reports to send or delete now: queued ones whose wait is over, and
    /// every delete not yet through, oldest first.
    pub fn feedback_due(&self, now_ms: i64) -> Result<Vec<OutboxRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {COLUMNS} FROM feedback_outbox \
                 WHERE (state = 'queued' AND next_try_ms <= ?1) \
                    OR (state = 'deleting' AND next_try_ms <= ?1) \
                 ORDER BY created_ms, id"
            ))?;
            let rows = stmt
                .query_map([now_ms], read)?
                .collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
    }

    /// The server took the report.
    pub fn feedback_mark_sent(&self, id: &str, status: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE feedback_outbox SET state = 'sent', attempts = 0, next_try_ms = 0, \
                 server_status = ?2 WHERE id = ?1 AND state = 'queued'",
                params![id, status],
            )?;
            Ok(())
        })
    }

    /// The screenshot's bytes are up: forget the local copy's path.
    pub fn feedback_mark_screenshot_sent(&self, id: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE feedback_outbox SET screenshot_sent = 1, screenshot_path = NULL WHERE id = ?1",
                [id],
            )?;
            Ok(())
        })
    }

    /// A send or delete failed: count it and wait until `next_try_ms`.
    pub fn feedback_failed(&self, id: &str, next_try_ms: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE feedback_outbox SET attempts = attempts + 1, next_try_ms = ?2 WHERE id = ?1",
                params![id, next_try_ms],
            )?;
            Ok(())
        })
    }

    /// Makes every waiting report due again (launch, or back online).
    pub fn feedback_retry_now(&self) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE feedback_outbox SET attempts = 0, next_try_ms = 0 WHERE state != 'sent'",
                [],
            )?;
            Ok(())
        })
    }

    /// Mirrors what the server's list says about a report it holds.
    pub fn feedback_record_status(&self, id: &str, status: &ServerStatus) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE feedback_outbox SET state = CASE WHEN state = 'queued' THEN 'sent' ELSE state END, \
                 server_status = ?2, issue = ?3, duplicate_of = ?4, pr = ?5, release = ?6 WHERE id = ?1",
                params![
                    id,
                    status.status,
                    status.issue,
                    status.duplicate_of,
                    status.pr,
                    status.release
                ],
            )?;
            Ok(())
        })
    }

    /// The person deleted the report. A report the server never saw goes at
    /// once; one it holds stays as `deleting` until the server confirms.
    /// Returns whether a server delete is still owed.
    pub fn feedback_mark_deleting(&self, id: &str) -> Result<bool> {
        self.write(|tx, _| {
            let found: Option<(String, i64)> = tx
                .query_row(
                    "SELECT state, attempts FROM feedback_outbox WHERE id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            match found {
                None => Ok(false),
                // Never tried: the server can't have it.
                Some((state, 0)) if state == "queued" => {
                    tx.execute("DELETE FROM feedback_outbox WHERE id = ?1", [id])?;
                    Ok(false)
                }
                // Sent, or tried and unsure: ask the server (a 404 is done too).
                Some(_) => {
                    tx.execute(
                        "UPDATE feedback_outbox SET state = 'deleting', attempts = 0, next_try_ms = 0 WHERE id = ?1",
                        [id],
                    )?;
                    Ok(true)
                }
            }
        })
    }

    /// Drops a report from this device (the server confirmed the delete).
    pub fn feedback_remove(&self, id: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute("DELETE FROM feedback_outbox WHERE id = ?1", [id])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    fn new(id: &str, at: i64) -> OutboxNew {
        OutboxNew {
            id: id.into(),
            payload: "{}".into(),
            kind: "bug".into(),
            preview: "Export cuts off".into(),
            screenshot_path: Some("/data/feedback/x.png".into()),
            created_ms: at,
        }
    }

    #[test]
    fn a_saved_report_is_due_once_and_sent_once() {
        let (_d, s) = store();
        s.feedback_enqueue(&new("fb_a", 1)).unwrap();
        // The same id twice keeps one row.
        s.feedback_enqueue(&new("fb_a", 2)).unwrap();
        assert_eq!(s.feedback_all().unwrap().len(), 1);
        assert_eq!(s.feedback_due(0).unwrap().len(), 1);
        s.feedback_mark_sent("fb_a", "received").unwrap();
        assert!(s.feedback_due(i64::MAX).unwrap().is_empty());
        let row = s.feedback_get("fb_a").unwrap().unwrap();
        assert_eq!(row.state, OutboxState::Sent);
        assert_eq!(row.server_status.as_deref(), Some("received"));
    }

    #[test]
    fn a_failure_waits_and_launch_retries_now() {
        let (_d, s) = store();
        s.feedback_enqueue(&new("fb_a", 1)).unwrap();
        s.feedback_failed("fb_a", 60_000).unwrap();
        assert!(s.feedback_due(59_999).unwrap().is_empty());
        assert_eq!(s.feedback_due(60_000).unwrap().len(), 1);
        assert_eq!(s.feedback_get("fb_a").unwrap().unwrap().attempts, 1);
        s.feedback_retry_now().unwrap();
        assert_eq!(s.feedback_due(0).unwrap().len(), 1);
        assert_eq!(s.feedback_get("fb_a").unwrap().unwrap().attempts, 0);
    }

    #[test]
    fn deleting_a_report_the_server_never_saw_is_local() {
        let (_d, s) = store();
        s.feedback_enqueue(&new("fb_a", 1)).unwrap();
        assert!(!s.feedback_mark_deleting("fb_a").unwrap());
        assert!(s.feedback_all().unwrap().is_empty());
    }

    #[test]
    fn deleting_a_sent_report_waits_for_the_server() {
        let (_d, s) = store();
        s.feedback_enqueue(&new("fb_a", 1)).unwrap();
        s.feedback_mark_sent("fb_a", "filed").unwrap();
        assert!(s.feedback_mark_deleting("fb_a").unwrap());
        assert_eq!(s.feedback_due(0).unwrap()[0].state, OutboxState::Deleting);
        // A status refresh doesn't resurrect it.
        s.feedback_record_status(
            "fb_a",
            &ServerStatus {
                status: "filed".into(),
                issue: Some(241),
                duplicate_of: None,
                pr: None,
                release: None,
            },
        )
        .unwrap();
        assert_eq!(
            s.feedback_get("fb_a").unwrap().unwrap().state,
            OutboxState::Deleting
        );
        s.feedback_remove("fb_a").unwrap();
        assert!(s.feedback_all().unwrap().is_empty());
    }

    #[test]
    fn the_server_status_is_mirrored() {
        let (_d, s) = store();
        s.feedback_enqueue(&new("fb_a", 1)).unwrap();
        s.feedback_mark_screenshot_sent("fb_a").unwrap();
        s.feedback_record_status(
            "fb_a",
            &ServerStatus {
                status: "released".into(),
                issue: Some(241),
                duplicate_of: None,
                pr: Some(252),
                release: Some("v0.9.2".into()),
            },
        )
        .unwrap();
        let row = s.feedback_get("fb_a").unwrap().unwrap();
        assert_eq!(
            (row.issue, row.pr, row.release.as_deref(), row.screenshot_path),
            (Some(241), Some(252), Some("v0.9.2"), None)
        );
        assert!(row.screenshot_sent);
    }
}
