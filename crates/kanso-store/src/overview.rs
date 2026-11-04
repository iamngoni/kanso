//! What Overview (issue #240) reads that the rest of the store doesn't
//! hold: where the person last stopped writing, what they taught the loop
//! rules by dismissing, the items "Not yet" put off, the people someone is
//! being waited on, and the notes to revisit.
//!
//! Locked notes and notes in locked notebooks never come back from here,
//! nor trashed notes. `#ai-exclude` notes may, because nothing here goes to
//! a model. Every table but `note_revisit` is local to this device, never
//! synced or backed up, and emptied with the library.

use kanso_core::loop_score::{self, Family, Feedback};
use kanso_core::loops::LoopKind;
use kanso_core::reminders::task_key;
use kanso_core::revisit::{self, REVISIT_FIELD, Revisit};
use kanso_core::writing::{self, GAP_MS};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::schema::{note_locked, readable_body};
use crate::write::{load_row, rewrite_doc};
use crate::{Origin, Result, Store};

/// The tables emptied with the library.
pub(crate) const TABLES: [&str; 3] = ["writing_places", "loop_feedback", "waiting_snoozed"];

/// Where a person last stopped writing in a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingPlace {
    pub note_id: String,
    pub title: String,
    pub book_id: String,
    pub line: u32,
    pub col: u32,
    pub anchor_hash: String,
    pub session_end: i64,
    /// `ui` for typing here, `sync` for a session that arrived from another
    /// device.
    pub source: String,
    /// The note's text, for reading the paragraph; empty when it can't be.
    pub body: String,
}

/// A waiting item: something open that names a person with a page and has
/// gone quiet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitingRow {
    pub person_id: String,
    pub person: String,
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    /// The loop's key, or the task's key; what "Not yet" snoozes.
    pub key: String,
    /// `loop` or `task`.
    pub kind: String,
    /// The later of the note's and the person page's last change (ms).
    pub quiet_since: i64,
}

/// A note to revisit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisitRow {
    pub note_id: String,
    pub title: String,
    pub book_id: String,
    pub on: String,
    pub set_at: i64,
    pub why: String,
}

/// The session in progress or last finished, as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Session {
    start: i64,
    end: i64,
    chars: i64,
}

fn read_session(conn: &Connection, id: &str) -> rusqlite::Result<Option<Session>> {
    conn.query_row(
        "SELECT session_start, session_end, session_chars FROM writing_places WHERE note_id = ?",
        [id],
        |row| {
            Ok(Session {
                start: row.get(0)?,
                end: row.get(1)?,
                chars: row.get(2)?,
            })
        },
    )
    .optional()
}

/// Records typing in a note: extends the session when the last keystroke
/// was within [`GAP_MS`], otherwise starts a new one. Does nothing for a
/// note that doesn't exist or can't be shown.
pub(crate) fn record_typing(
    conn: &Connection,
    id: &str,
    line: u32,
    col: u32,
    typed_chars: u32,
    anchor: &str,
    now: i64,
    source: &str,
) -> Result<()> {
    let session = read_session(conn, id)?;
    let (start, chars) = match session {
        Some(s) if now >= s.end && now - s.end <= GAP_MS => {
            (s.start, s.chars + i64::from(typed_chars))
        }
        _ => (now, i64::from(typed_chars)),
    };
    conn.execute(
        "INSERT INTO writing_places
            (note_id, line, col, anchor_hash, session_start, session_end, session_chars, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (note_id) DO UPDATE SET line = excluded.line, col = excluded.col,
             anchor_hash = excluded.anchor_hash, session_start = excluded.session_start,
             session_end = excluded.session_end, session_chars = excluded.session_chars,
             source = excluded.source",
        params![id, line, col, anchor, start, now, chars, source],
    )?;
    Ok(())
}

/// A change that arrived by sync: finds where it was made and records it as
/// a session, `source: 'sync'`. Only changes of a real size, recent ones,
/// and ones to a note this device already had count.
pub(crate) fn record_synced_change(
    conn: &Connection,
    id: &str,
    old_body: &str,
    new_body: &str,
    updated_at: i64,
    now: i64,
) -> Result<()> {
    if now - updated_at > writing::STALE_DAYS * 86_400_000 {
        return Ok(());
    }
    let Some((line, col)) = writing::last_changed(old_body, new_body) else {
        return Ok(());
    };
    let grown = new_body.chars().count().abs_diff(old_body.chars().count()) as u32;
    if grown < writing::MIN_CHARS {
        return Ok(());
    }
    let lines: Vec<&str> = new_body.lines().collect();
    let anchor = lines
        .get(line as usize)
        .and_then(|_| {
            let found = writing::find(new_body, line, col, "")?;
            Some(writing::anchor_hash(&found.text))
        })
        .unwrap_or_default();
    record_typing(
        conn,
        id,
        line,
        col,
        grown,
        &anchor,
        updated_at.min(now),
        "sync",
    )
}

/// Forgets the place recorded for a note (it was deleted or locked).
pub(crate) fn forget_note(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM writing_places WHERE note_id = ?", [id])?;
    conn.execute("DELETE FROM note_revisit WHERE note_id = ?", [id])?;
    Ok(())
}

/// Drops the snapshot and the AI cache, which hold titles and sentences.
pub(crate) fn drop_snapshot(conn: &Connection) -> Result<()> {
    conn.execute(
        "DELETE FROM meta WHERE key LIKE 'overview.snapshot%' OR key LIKE 'overview.ai.%'",
        [],
    )?;
    Ok(())
}

impl Store {
    /// Records typing in note `id`: where the caret is (`line`, `col`, in
    /// characters), how many characters were typed since the last call, and
    /// the hash of the paragraph's first words. Callers throttle to once
    /// every ten seconds per note.
    pub fn note_writing(
        &self,
        id: &str,
        line: u32,
        col: u32,
        typed_chars: u32,
        anchor: &str,
        now: i64,
    ) -> Result<()> {
        self.write(|tx, _| {
            let live: bool = tx
                .query_row(
                    concat!(
                        "SELECT d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT ",
                        note_locked!("d."),
                        " FROM docs d WHERE d.id = ? AND d.type = 'note'"
                    ),
                    [id],
                    |row| row.get(0),
                )
                .optional()?
                .unwrap_or(false);
            if !live {
                return Ok(());
            }
            record_typing(tx, id, line, col, typed_chars, anchor, now, "ui")
        })
    }

    /// The notes the person was last writing in, newest first: the
    /// sessions that count, within the last 14 days, not in `exclude`, not
    /// daily notes, not notes marked completed or dropped, not hidden by
    /// "Not now" since.
    pub fn pick_up_places(
        &self,
        now: i64,
        exclude: &[String],
        limit: usize,
    ) -> Result<Vec<WritingPlace>> {
        let from = now - writing::STALE_DAYS * 86_400_000;
        let sql = format!(
            "SELECT w.note_id, d.title, d.book_id, w.line, w.col, w.anchor_hash, w.session_end,
                    w.source, IFNULL({body}, '')
             FROM writing_places w JOIN docs d ON d.id = w.note_id
             WHERE d.deleted = 0 AND d.type = 'note' AND IFNULL(d.book_id, '') != 'trash'
               AND NOT {locked}
               AND w.note_id NOT LIKE 'note:daily-%'
               AND IFNULL(d.status, '') NOT IN ('completed', 'dropped')
               AND w.session_end >= ?1 AND w.session_end > w.hidden_at
               AND (w.session_chars >= ?2 OR w.session_end - w.session_start >= ?3)
             ORDER BY w.session_end DESC LIMIT ?4",
            body = readable_body!("d."),
            locked = note_locked!("d."),
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(
                params![
                    from,
                    writing::MIN_CHARS,
                    writing::MIN_MS,
                    (limit + exclude.len()) as i64
                ],
                |row| {
                    Ok(WritingPlace {
                        note_id: row.get(0)?,
                        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        line: row.get(3)?,
                        col: row.get(4)?,
                        anchor_hash: row.get(5)?,
                        session_end: row.get(6)?,
                        source: row.get(7)?,
                        body: row.get(8)?,
                    })
                },
            )?;
            let mut out = Vec::new();
            for row in rows {
                let row = row?;
                if exclude.contains(&row.note_id) {
                    continue;
                }
                out.push(row);
                if out.len() == limit {
                    break;
                }
            }
            Ok(out)
        })
    }

    /// "Not now": hides the place until the person writes in the note again.
    pub fn hide_writing_place(&self, id: &str, now: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE writing_places SET hidden_at = ?2 WHERE note_id = ?1",
                params![id, now],
            )?;
            Ok(())
        })
    }

    /// How many writing sessions were recorded per notebook from `from` (ms)
    /// to `to`, for the week's active projects: `(book_id, sessions)`.
    pub fn writing_sessions_by_book(&self, from: i64, to: i64) -> Result<Vec<(String, u32)>> {
        let sql = format!(
            "SELECT IFNULL(d.book_id, ''), COUNT(*) FROM writing_places w JOIN docs d ON d.id = w.note_id
             WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT {locked}
               AND w.session_end >= ?1 AND w.session_end < ?2
               AND (w.session_chars >= ?3 OR w.session_end - w.session_start >= ?4)
             GROUP BY d.book_id",
            locked = note_locked!("d.")
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(
                params![from, to, writing::MIN_CHARS, writing::MIN_MS],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)),
            )?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    // ---- loop feedback ----

    /// Notes that the person dismissed (`Dismissed`) or ticked (`Done`) a
    /// loop of `family`.
    pub fn record_loop_feedback(&self, family: Family, what: Feedback, at: i64) -> Result<()> {
        self.write(|tx, _| {
            record_feedback(tx, family, what, at)?;
            Ok(())
        })
    }

    /// The families Overview stops guessing (two dismissals in 60 days).
    pub fn suppressed_loop_families(&self, now: i64) -> Result<Vec<Family>> {
        self.read(|conn| {
            let from = now - loop_score::WINDOW_DAYS * 86_400_000;
            let mut stmt =
                conn.prepare_cached("SELECT family, at, what FROM loop_feedback WHERE at >= ?")?;
            let rows: Vec<(String, i64, String)> = stmt
                .query_map([from], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let mut out = Vec::new();
            for family in Family::ALL {
                let events: Vec<(i64, Feedback)> = rows
                    .iter()
                    .filter(|(f, ..)| f == family.as_str())
                    .filter_map(|(_, at, what)| Some((*at, Feedback::parse(what)?)))
                    .collect();
                if loop_score::suppressed(&events, now) {
                    out.push(family);
                }
            }
            Ok(out)
        })
    }

    /// Forgets the dismissals of `family`: "Show them again".
    pub fn clear_loop_feedback(&self, family: Family) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "DELETE FROM loop_feedback WHERE family = ?",
                [family.as_str()],
            )?;
            Ok(())
        })
    }

    // ---- waiting on others ----

    /// Open items that name a person with a People page and have gone
    /// quiet: a waiting loop mentioning them, or an unchecked task on a
    /// line that mentions them. Quiet means neither the note nor the
    /// person's page changed in the last `quiet_days`. Snoozed items are
    /// left out. The quietest first, at most `limit`.
    pub fn waiting_items(
        &self,
        now: i64,
        quiet_days: u32,
        limit: usize,
    ) -> Result<Vec<WaitingRow>> {
        let cutoff = now - i64::from(quiet_days) * 86_400_000;
        let loops_sql = format!(
            "SELECT m.entity_id, IFNULL(pd.title, e.name), l.note_id, d.title, l.line, l.text, l.key, 'loop',
                    MAX(d.updated_at, pd.updated_at)
             FROM note_loops l
             JOIN docs d ON d.id = l.note_id
             JOIN note_mentions m ON m.note_id = l.note_id AND m.line = l.line
             JOIN entities e ON e.note_id = m.entity_id AND e.kind = 'person'
             JOIN docs pd ON pd.id = e.note_id
             WHERE l.kind = 'waiting'
               AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT {note_locked}
               AND pd.deleted = 0 AND NOT {person_locked}
               AND m.entity_id != l.note_id
               AND json_type(d.json, '$.kansoLoops.closed.\"' || l.key || '\"') IS NULL
               AND json_type(d.json, '$.kansoLoops.dismissed.\"' || l.key || '\"') IS NULL
               AND d.updated_at < ?1 AND pd.updated_at < ?1",
            note_locked = note_locked!("d."),
            person_locked = note_locked!("pd."),
        );
        let tasks_sql = format!(
            "SELECT m.entity_id, IFNULL(pd.title, e.name), t.note_id, d.title, t.line, t.text, '', 'task',
                    MAX(d.updated_at, pd.updated_at)
             FROM note_tasks t
             JOIN docs d ON d.id = t.note_id
             JOIN note_mentions m ON m.note_id = t.note_id AND m.line = t.line
             JOIN entities e ON e.note_id = m.entity_id AND e.kind = 'person'
             JOIN docs pd ON pd.id = e.note_id
             WHERE t.checked = 0
               AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT {note_locked}
               AND pd.deleted = 0 AND NOT {person_locked}
               AND m.entity_id != t.note_id
               AND d.updated_at < ?1 AND pd.updated_at < ?1",
            note_locked = note_locked!("d."),
            person_locked = note_locked!("pd."),
        );
        self.read(|conn| {
            let mut out: Vec<WaitingRow> = Vec::new();
            for sql in [loops_sql, tasks_sql] {
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map([cutoff], |row| {
                    Ok(WaitingRow {
                        person_id: row.get(0)?,
                        person: row.get(1)?,
                        note_id: row.get(2)?,
                        note_title: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        line: row.get(4)?,
                        text: row.get(5)?,
                        key: row.get(6)?,
                        kind: row.get(7)?,
                        quiet_since: row.get(8)?,
                    })
                })?;
                for row in rows {
                    let mut row = row?;
                    if row.kind == "task" {
                        row.key = task_key(&row.note_id, &row.text, "");
                    }
                    out.push(row);
                }
            }
            let snoozed: Vec<(String, i64)> = conn
                .prepare_cached("SELECT key, until FROM waiting_snoozed WHERE until > ?")?
                .query_map([now], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            out.retain(|r| !snoozed.iter().any(|(k, _)| *k == r.key));
            // One row per sentence, whoever else the line names.
            out.sort_by(|a, b| {
                a.quiet_since
                    .cmp(&b.quiet_since)
                    .then_with(|| a.note_id.cmp(&b.note_id))
                    .then_with(|| a.line.cmp(&b.line))
                    .then_with(|| a.person_id.cmp(&b.person_id))
            });
            out.dedup_by(|a, b| a.note_id == b.note_id && a.line == b.line);
            out.truncate(limit);
            Ok(out)
        })
    }

    /// "Not yet": puts the waiting item `key` off until `until` (ms).
    pub fn snooze_waiting(&self, key: &str, until: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR REPLACE INTO waiting_snoozed (key, until) VALUES (?1, ?2)",
                params![key, until],
            )?;
            tx.execute(
                "DELETE FROM waiting_snoozed WHERE until < ?",
                [until - 30 * 86_400_000],
            )?;
            Ok(())
        })
    }

    // ---- revisit ----

    /// The revisit set on note `id`, if any.
    pub fn revisit_of(&self, id: &str) -> Result<Option<Revisit>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT on_day, set_at, why FROM note_revisit WHERE note_id = ?",
                    [id],
                    |row| {
                        Ok(Revisit {
                            on: row.get(0)?,
                            set_at: row.get(1)?,
                            why: row.get(2)?,
                        })
                    },
                )
                .optional()?)
        })
    }

    /// Sets (`Some`) or clears (`None`) the revisit of note `id`. It is not
    /// an edit by the person: the note's text and edit time stay, and no
    /// version is added to its history. Does nothing for a locked note.
    /// Returns whether anything changed.
    pub fn set_revisit(&self, id: &str, revisit: Option<&Revisit>) -> Result<bool> {
        let value = revisit.map(revisit::to_value);
        self.write(|tx, changes| {
            let Some(prev) = load_row(tx, id)?.filter(|p| !p.deleted) else {
                return Ok(false);
            };
            if prev.fields.contains_key(kanso_core::model::LOCK_FIELD)
                || prev.fields.get(REVISIT_FIELD) == value.as_ref()
            {
                return Ok(false);
            }
            let known: i64 = tx.query_row(
                "SELECT COUNT(*) FROM revisions WHERE doc_id = ?1 AND rev = ?2",
                params![id, prev.rev],
                |row| row.get(0),
            )?;
            rewrite_doc(
                tx,
                id,
                prev.doc_type,
                Origin::Api,
                changes,
                |fields| match &value {
                    Some(value) => {
                        fields.insert(REVISIT_FIELD.into(), value.clone());
                    }
                    None => {
                        fields.remove(REVISIT_FIELD);
                    }
                },
            )?;
            if known == 0 {
                tx.execute(
                    "DELETE FROM revisions WHERE doc_id = ?1 AND rev = ?2",
                    params![id, prev.rev],
                )?;
            }
            Ok(true)
        })
    }

    /// Notes whose revisit day is `today` or earlier (`YYYY-MM-DD`), the
    /// oldest first.
    pub fn revisits_due(&self, today: &str, limit: usize) -> Result<Vec<RevisitRow>> {
        self.revisits_between(None, today, limit)
    }

    /// Notes with a revisit day after `from` and up to `to`, soonest first.
    pub fn revisits_between(
        &self,
        after: Option<&str>,
        to: &str,
        limit: usize,
    ) -> Result<Vec<RevisitRow>> {
        let sql = format!(
            "SELECT r.note_id, d.title, d.book_id, r.on_day, r.set_at, r.why
             FROM note_revisit r JOIN docs d ON d.id = r.note_id
             WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT {locked}
               AND r.on_day <= ?1 AND r.on_day > ?2
             ORDER BY r.on_day, r.note_id LIMIT ?3",
            locked = note_locked!("d.")
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![to, after.unwrap_or(""), limit as i64], |row| {
                Ok(RevisitRow {
                    note_id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    on: row.get(3)?,
                    set_at: row.get(4)?,
                    why: row.get(5)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many notes are due for a revisit by `today`.
    pub fn revisit_due_count(&self, today: &str) -> Result<u32> {
        let sql = format!(
            "SELECT COUNT(*) FROM note_revisit r JOIN docs d ON d.id = r.note_id
             WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT {locked}
               AND r.on_day <= ?",
            locked = note_locked!("d.")
        );
        self.read(|conn| Ok(conn.query_row(&sql, [today], |row| row.get::<_, u32>(0))?))
    }
}

/// Appends a feedback row and trims what is older than the window.
pub(crate) fn record_feedback(
    conn: &Connection,
    family: Family,
    what: Feedback,
    at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO loop_feedback (family, at, what) VALUES (?1, ?2, ?3)",
        params![family.as_str(), at, what.as_str()],
    )?;
    conn.execute(
        "DELETE FROM loop_feedback WHERE at < ?",
        [at - (loop_score::WINDOW_DAYS + 30) * 86_400_000],
    )?;
    Ok(())
}

/// What a loop's close teaches: the family and what was done.
pub(crate) fn feedback_for(kind: LoopKind, text: &str, dismissed: bool) -> (Family, Feedback) {
    (
        loop_score::family(text, kind),
        if dismissed {
            Feedback::Dismissed
        } else {
            Feedback::Done
        },
    )
}

/// Reads the revisit out of a note's fields and indexes it.
pub(crate) fn index_revisit(
    conn: &Connection,
    id: &str,
    fields: &serde_json::Map<String, Value>,
) -> Result<()> {
    conn.execute("DELETE FROM note_revisit WHERE note_id = ?", [id])?;
    if let Some(r) = revisit::read(fields) {
        conn.execute(
            "INSERT INTO note_revisit (note_id, on_day, set_at, why) VALUES (?1, ?2, ?3, ?4)",
            params![id, r.on, r.set_at, r.why],
        )?;
    }
    Ok(())
}
