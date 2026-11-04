//! Saved versions the write path would not snapshot on its own.

use kanso_core::model::LOCK_FIELD;
use kanso_core::now_ms;
use rusqlite::{OptionalExtension, params};
use serde_json::{Map, Value};

use crate::write::snapshot_limit;
use crate::{Origin, Result, Store, StoreError};

/// When a version saved now goes in `id`'s history: now, or just after the
/// newest version kept if the clock has since gone back (set by hand, or
/// stepped by the network time), so the history stays newest first.
pub(crate) fn version_time(tx: &rusqlite::Transaction, id: &str) -> Result<i64> {
    let newest: Option<i64> = tx.query_row(
        "SELECT MAX(saved_at) FROM revisions WHERE doc_id = ?",
        [id],
        |row| row.get(0),
    )?;
    Ok(after(now_ms(), newest))
}

/// `now`, or one past `newest` when that is later. Pure.
pub(crate) fn after(now: i64, newest: Option<i64>) -> i64 {
    newest.map_or(now, |newest| now.max(newest.saturating_add(1)))
}

impl Store {
    /// Keeps `fields` as a saved version of `id`, for example unsaved edits
    /// the user set aside for a newer version made elsewhere.
    pub fn record_revision(
        &self,
        id: &str,
        rev: &str,
        fields: &Map<String, Value>,
        origin: Origin,
    ) -> Result<()> {
        self.record(id, rev, fields, origin, None)
    }

    /// [`Store::record_revision`], saved as of `saved_at`: a version kept
    /// from another clock, or a history drawn at a fixed time.
    pub fn record_revision_at(
        &self,
        id: &str,
        rev: &str,
        fields: &Map<String, Value>,
        origin: Origin,
        saved_at: i64,
    ) -> Result<()> {
        self.record(id, rev, fields, origin, Some(saved_at))
    }

    /// Keeps `fields` in `id`'s history as of `saved_at`, or as of now
    /// ([`version_time`]) without one.
    fn record(
        &self,
        id: &str,
        rev: &str,
        fields: &Map<String, Value>,
        origin: Origin,
        saved_at: Option<i64>,
    ) -> Result<()> {
        let json = serde_json::to_string(fields)?;
        let has_text = fields
            .get("body")
            .and_then(Value::as_str)
            .is_some_and(|body| !body.is_empty());
        self.write(|tx, _| {
            // A locked note's history holds only sealed versions, and so
            // does a note's in a locked notebook.
            if has_text && !fields.contains_key(LOCK_FIELD) {
                let locked: bool = tx
                    .query_row(
                        concat!(
                            "SELECT ",
                            crate::schema::note_locked!(""),
                            " FROM docs WHERE id = ?"
                        ),
                        [id],
                        |row| row.get(0),
                    )
                    .optional()?
                    .unwrap_or(false);
                if locked {
                    return Err(StoreError::Invalid(
                        "The note is locked: its versions are kept sealed.".into(),
                    ));
                }
            }
            let saved_at = match saved_at {
                Some(at) => at,
                None => version_time(tx, id)?,
            };
            tx.execute(
                "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, rev, json, saved_at, origin.as_str()],
            )?;
            tx.execute(
                "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
                 (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
                params![id, snapshot_limit(fields)],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::after;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    #[test]
    fn a_clock_gone_back_saves_just_after_the_newest() {
        assert_eq!(after(1_000, None), 1_000);
        assert_eq!(after(1_000, Some(400)), 1_000);
        assert_eq!(after(1_000, Some(5_000)), 5_001);
        assert_eq!(after(1_000, Some(i64::MAX)), i64::MAX);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A version saved now is never before now, and always after the
        /// newest kept (unless that is the end of time); with the clock
        /// ahead of the history, it is just now.
        #[test]
        fn prop_now_or_just_after_the_newest(now in any::<i64>(), newest in prop::option::of(any::<i64>())) {
            let at = after(now, newest);
            prop_assert!(at >= now);
            if let Some(newest) = newest {
                prop_assert!(at > newest || newest == i64::MAX);
                if now > newest {
                    prop_assert_eq!(at, now);
                }
            } else {
                prop_assert_eq!(at, now);
            }
        }
    }
}
