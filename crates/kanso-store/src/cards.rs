//! Flashcards: the cards written in note bodies (`note_cards`, derived),
//! their schedule (`card_state`, a local cache replayed from the synced
//! `review` documents), sessions, answers and stats.
//!
//! The words of a card live only in the note. What syncs is the log of
//! answers, one document per note and month, merged by union
//! ([`kanso_core::review_doc`]); the schedule is always recomputed from
//! it, so two devices that reviewed offline agree.

use std::collections::{BTreeMap, BTreeSet};

use kanso_core::DocType;
use kanso_core::review_doc::{self, ReviewDoc};
use kanso_core::srs::{self, CardState, Grade, Mark, MarkKind, Phase, Review};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Map, Value};

use crate::cards_model::{
    CardCounts, CardScope, CardSettings, CardStats, DueCard, HardCard, NoteCard,
};
use crate::validate::{Refs, normalize};
use crate::write::{Row, Synced, delete_row, load_row, next_rev, store_row};
use crate::{Change, Origin, Result, Store, StoreError};

const DAY_MS: i64 = srs::DAY_MS;
const SETTINGS_KEY: &str = "cards.settings";

/// Notes whose cards may be shown: live, not in the trash, not locked and
/// not in a locked notebook. A locked note has no `note_cards` rows to
/// begin with; this is the second lock on the door.
const ELIGIBLE: &str = "d.type = 'note' AND d.deleted = 0 \
    AND IFNULL(d.book_id, '') != 'trash' \
    AND json_extract(d.json, '$.kansoLock') IS NULL \
    AND IFNULL(d.book_id, '') NOT IN (SELECT id FROM locked_books)";

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::New => "new",
        Phase::Learning => "learning",
        Phase::Review => "review",
        Phase::Relearning => "relearning",
    }
}

fn phase_of(name: &str) -> Phase {
    match name {
        "learning" => Phase::Learning,
        "review" => Phase::Review,
        "relearning" => Phase::Relearning,
        _ => Phase::New,
    }
}

/// The SQL that narrows eligible cards to `scope`, and its parameter.
fn scope_sql(scope: &CardScope) -> (String, Option<String>) {
    match scope {
        CardScope::All => (String::new(), None),
        CardScope::Book(id) => (
            "AND d.book_id IN (WITH RECURSIVE t(id) AS (SELECT ?1 UNION \
             SELECT b.id FROM docs b JOIN t ON b.parent_book_id = t.id \
             WHERE b.type = 'book' AND b.deleted = 0) SELECT id FROM t)"
                .into(),
            Some(id.clone()),
        ),
        CardScope::Tag(id) => (
            "AND d.id IN (SELECT note_id FROM note_tags WHERE tag_id = ?1)".into(),
            Some(id.clone()),
        ),
        CardScope::Note(id) => ("AND nc.note_id = ?1".into(), Some(id.clone())),
    }
}

fn bind(extra: &Option<String>) -> Vec<&dyn rusqlite::ToSql> {
    match extra {
        Some(value) => vec![value as &dyn rusqlite::ToSql],
        None => Vec::new(),
    }
}

// --- the log of answers ------------------------------------------------

/// Recomputes the schedule of every card of the note a review document is
/// about, after that document was written (`fields`) or deleted.
pub(crate) fn refresh_review(
    tx: &Transaction,
    _id: &str,
    previous: Option<&Row>,
    fields: Option<&Map<String, Value>>,
) -> Result<()> {
    let note_of =
        |f: &Map<String, Value>| f.get("noteId").and_then(Value::as_str).map(str::to_owned);
    let mut notes = BTreeSet::new();
    notes.extend(fields.and_then(note_of));
    notes.extend(previous.and_then(|p| note_of(&p.fields)));
    let retention = settings_in(tx)?.retention;
    for note in notes {
        recompute(tx, &note, retention)?;
    }
    Ok(())
}

fn live_review_docs(conn: &Connection, note_id: &str) -> Result<Vec<ReviewDoc>> {
    let mut stmt = conn.prepare_cached(
        "SELECT json FROM docs WHERE type = 'review' AND deleted = 0 \
         AND json_extract(json, '$.noteId') = ?",
    )?;
    let docs = stmt
        .query_map([note_id], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    docs.iter()
        .map(|json| match serde_json::from_str::<Value>(json)? {
            Value::Object(fields) => Ok(ReviewDoc::from_fields(&fields)),
            _ => Ok(ReviewDoc::default()),
        })
        .collect()
}

/// Rebuilds `card_state` and `card_log` for one note from its documents.
fn recompute(tx: &Transaction, note_id: &str, retention: f64) -> Result<()> {
    let docs = live_review_docs(tx, note_id)?;
    tx.execute("DELETE FROM card_state WHERE note_id = ?", [note_id])?;
    tx.execute("DELETE FROM card_log WHERE note_id = ?", [note_id])?;
    let mut logs: BTreeMap<String, Review> = BTreeMap::new();
    let mut marks: BTreeMap<String, Mark> = BTreeMap::new();
    for doc in docs {
        for r in doc.log {
            logs.entry(r.id.clone()).or_insert(r);
        }
        for m in doc.marks {
            marks.entry(m.id.clone()).or_insert(m);
        }
    }
    let logs: Vec<Review> = logs.into_values().collect();
    let marks: Vec<Mark> = marks.into_values().collect();
    let cards: BTreeSet<&str> = logs
        .iter()
        .map(|r| r.card.as_str())
        .chain(marks.iter().map(|m| m.card.as_str()))
        .collect();
    let mut insert_state = tx.prepare_cached(
        "INSERT INTO card_state (card_key, note_id, due_ms, stability, difficulty, reps, \
         lapses, phase, step, last_ms, suspended) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;
    let mut insert_log = tx.prepare_cached(
        "INSERT OR IGNORE INTO card_log (id, card_key, note_id, at, grade) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for card in cards {
        let counted = srs::since_reset(&logs, &marks, card);
        let owned: Vec<Review> = counted.iter().map(|r| (*r).clone()).collect();
        let state = srs::replay(&owned, retention);
        let suspended = srs::is_suspended(&marks, card);
        insert_state.execute(params![
            card,
            note_id,
            state.due_ms,
            state.stability,
            state.difficulty,
            state.reps,
            state.lapses,
            phase_name(state.phase),
            state.step,
            state.last_ms,
            suspended
        ])?;
        for r in &owned {
            insert_log.execute(params![r.id, card, note_id, r.at_ms, r.grade.number()])?;
        }
    }
    Ok(())
}

fn settings_in(conn: &Connection) -> Result<CardSettings> {
    let json = crate::schema::get_meta(conn, SETTINGS_KEY)?;
    let mut settings: CardSettings = json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();
    settings.retention = srs::clamp_retention(settings.retention);
    Ok(settings)
}

/// Appends to the note's current review document (a new one when the
/// month's is full or missing).
fn write_entry(
    tx: &Transaction,
    changes: &mut Vec<Change>,
    note_id: &str,
    at_ms: i64,
    add: impl FnOnce(&mut ReviewDoc),
) -> Result<()> {
    let month = review_doc::month_of(at_ms);
    let mut index = 1;
    let (id, prev, mut doc) = loop {
        let id = review_doc::doc_id(note_id, &month, index);
        let prev = load_row(tx, &id)?;
        match prev.as_ref().filter(|p| !p.deleted) {
            None => break (id, prev, ReviewDoc::new(note_id, &month)),
            Some(p) => {
                let doc = ReviewDoc::from_fields(&p.fields);
                if doc.len() < review_doc::MAX_ENTRIES {
                    break (id, prev, doc);
                }
            }
        }
        index += 1;
    };
    add(&mut doc);
    let mut fields = doc.to_fields();
    fields.insert("updatedAt".into(), at_ms.into());
    let fields = normalize(
        DocType::Review,
        &id,
        fields,
        prev.as_ref().filter(|p| !p.deleted).map(|p| &p.fields),
        at_ms,
        Refs::Skip,
    )?;
    let rev = next_rev(prev.as_ref(), &Value::Object(fields.clone()))?;
    store_row(
        tx,
        &id,
        DocType::Review,
        &rev,
        false,
        &fields,
        prev.as_ref(),
        Synced::Keep,
        Origin::Ui,
        changes,
    )
}

/// Deletes every review document of `note_id` (a tombstone each).
pub(crate) fn forget_note_reviews(
    tx: &Transaction,
    changes: &mut Vec<Change>,
    note_id: &str,
    origin: Origin,
) -> Result<()> {
    let ids: Vec<String> = tx
        .prepare_cached(
            "SELECT id FROM docs WHERE type = 'review' AND deleted = 0 \
             AND json_extract(json, '$.noteId') = ?",
        )?
        .query_map([note_id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in ids {
        if let Some(row) = load_row(tx, &id)?.filter(|r| !r.deleted) {
            delete_row(tx, &id, &row, origin, changes)?;
        }
    }
    Ok(())
}

fn new_entry_id(prefix: &str, at_ms: i64) -> String {
    format!("{prefix}_{at_ms:x}{:08x}", fastrand::u32(..))
}

fn state_from_row(row: &rusqlite::Row<'_>, from: usize) -> rusqlite::Result<CardState> {
    Ok(CardState {
        due_ms: row.get(from)?,
        stability: row.get(from + 1)?,
        difficulty: row.get(from + 2)?,
        reps: row.get(from + 3)?,
        lapses: row.get(from + 4)?,
        phase: phase_of(&row.get::<_, String>(from + 5)?),
        step: row.get(from + 6)?,
        last_ms: row.get(from + 7)?,
    })
}

impl Store {
    pub fn card_settings(&self) -> Result<CardSettings> {
        self.read(|conn| settings_in(conn))
    }

    /// Saves the settings; a new target recall reschedules every card.
    pub fn set_card_settings(&self, settings: &CardSettings) -> Result<()> {
        let mut settings = settings.clone();
        settings.retention = srs::clamp_retention(settings.retention);
        self.write(|tx, _| {
            let before = settings_in(tx)?;
            crate::schema::set_meta(tx, SETTINGS_KEY, &serde_json::to_string(&settings)?)?;
            if (before.retention - settings.retention).abs() > f64::EPSILON {
                let notes: Vec<String> = tx
                    .prepare(
                        "SELECT DISTINCT json_extract(json, '$.noteId') FROM docs \
                         WHERE type = 'review' AND deleted = 0",
                    )?
                    .query_map([], |row| row.get::<_, Option<String>>(0))?
                    .filter_map(|r| r.transpose())
                    .collect::<rusqlite::Result<_>>()?;
                for note in notes {
                    recompute(tx, &note, settings.retention)?;
                }
            }
            Ok(())
        })
    }

    /// The cards of one note with their schedule, for the editor.
    pub fn note_cards(&self, note_id: &str) -> Result<Vec<NoteCard>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT nc.card_key, nc.line, nc.question, nc.answer, cs.due_ms, cs.reps, \
                 IFNULL(cs.suspended, 0) \
                 FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
                 LEFT JOIN card_state cs ON cs.card_key = nc.card_key \
                 WHERE nc.note_id = ?1 AND {ELIGIBLE} ORDER BY nc.ord"
            ))?;
            let rows = stmt
                .query_map([note_id], |row| {
                    let reps: Option<u32> = row.get(5)?;
                    Ok(NoteCard {
                        card_key: row.get(0)?,
                        line: row.get(1)?,
                        question: row.get(2)?,
                        answer: row.get(3)?,
                        due_ms: if reps.unwrap_or(0) > 0 {
                            row.get(4)?
                        } else {
                            None
                        },
                        suspended: row.get(6)?,
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
    }

    /// How many answers and fresh cards today already used up.
    fn used_today(conn: &Connection, day_start_ms: i64) -> Result<(u32, u32)> {
        let reviews: u32 = conn.query_row(
            "SELECT COUNT(*) FROM card_log WHERE at >= ?",
            [day_start_ms],
            |row| row.get(0),
        )?;
        let fresh: u32 = conn.query_row(
            "SELECT COUNT(*) FROM (SELECT MIN(at) AS first FROM card_log GROUP BY card_key) \
             WHERE first >= ?",
            [day_start_ms],
            |row| row.get(0),
        )?;
        Ok((reviews, fresh))
    }

    /// The cards to show now in `scope`: due ones by due time, then new
    /// ones in note order, within the day's limits (counted from today's
    /// answers on every device). `day_start_ms` is the start of the
    /// person's day.
    pub fn due_cards(
        &self,
        scope: &CardScope,
        now_ms: i64,
        day_start_ms: i64,
    ) -> Result<Vec<DueCard>> {
        self.read(|conn| {
            let settings = settings_in(conn)?;
            let (used_reviews, used_new) = Self::used_today(conn, day_start_ms)?;
            let (clause, extra) = scope_sql(scope);
            let mut stmt = conn.prepare(&format!(
                "SELECT nc.card_key, nc.note_id, IFNULL(d.title, ''), nc.line, nc.question, nc.answer, \
                 cs.due_ms, cs.stability, cs.difficulty, cs.reps, cs.lapses, cs.phase, cs.step, cs.last_ms, \
                 IFNULL(cs.suspended, 0), cs.card_key IS NULL \
                 FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
                 LEFT JOIN card_state cs ON cs.card_key = nc.card_key \
                 WHERE {ELIGIBLE} {clause} \
                 ORDER BY d.created_at, nc.note_id, nc.ord"
            ))?;
            let mut due = Vec::new();
            let mut fresh = Vec::new();
            let mut rows = stmt.query(bind(&extra).as_slice())?;
            while let Some(row) = rows.next()? {
                if row.get::<_, bool>(14)? {
                    continue;
                }
                let missing: bool = row.get(15)?;
                let state = if missing {
                    CardState::new()
                } else {
                    state_from_row(row, 6)?
                };
                let card = DueCard {
                    card_key: row.get(0)?,
                    note_id: row.get(1)?,
                    note_title: row.get(2)?,
                    line: row.get(3)?,
                    question: row.get(4)?,
                    answer: row.get(5)?,
                    is_new: state.reps == 0,
                    state,
                };
                if card.is_new {
                    fresh.push(card);
                } else if card.state.due_ms <= now_ms {
                    due.push(card);
                }
            }
            due.sort_by_key(|c| c.state.due_ms);
            due.truncate(settings.reviews_per_day.saturating_sub(used_reviews) as usize);
            fresh.truncate(settings.new_per_day.saturating_sub(used_new) as usize);
            due.extend(fresh);
            Ok(due)
        })
    }

    /// How many cards are waiting (the sidebar badge): due plus new,
    /// within the day's limits.
    pub fn cards_waiting(
        &self,
        scope: &CardScope,
        now_ms: i64,
        day_start_ms: i64,
    ) -> Result<usize> {
        self.read(|conn| {
            let settings = settings_in(conn)?;
            let (used_reviews, used_new) = Self::used_today(conn, day_start_ms)?;
            let counts = counts_in(conn, scope, now_ms)?;
            let reviews = counts
                .due
                .min(settings.reviews_per_day.saturating_sub(used_reviews) as usize);
            let fresh = counts
                .new
                .min(settings.new_per_day.saturating_sub(used_new) as usize);
            Ok(reviews + fresh)
        })
    }

    pub fn card_counts(&self, scope: &CardScope, now_ms: i64) -> Result<CardCounts> {
        self.read(|conn| counts_in(conn, scope, now_ms))
    }

    /// Records an answer to a card of `note_id`. The note must be one the
    /// card may be shown from. Returns the card's new schedule.
    pub fn record_review(
        &self,
        note_id: &str,
        card_key: &str,
        grade: Grade,
        device: &str,
        at_ms: i64,
    ) -> Result<CardState> {
        self.write(|tx, changes| {
            check_card(tx, note_id, card_key)?;
            let entry = Review {
                id: new_entry_id("r", at_ms),
                card: card_key.to_owned(),
                at_ms,
                grade,
                device: device.to_owned(),
            };
            write_entry(tx, changes, note_id, at_ms, |doc| doc.log.push(entry))?;
            let state = tx
                .query_row(
                    "SELECT due_ms, stability, difficulty, reps, lapses, phase, step, last_ms \
                     FROM card_state WHERE card_key = ?",
                    [card_key],
                    |row| state_from_row(row, 0),
                )
                .optional()?;
            Ok(state.unwrap_or_default())
        })
    }

    /// Suspends or resumes a card.
    pub fn suspend_card(
        &self,
        note_id: &str,
        card_key: &str,
        suspended: bool,
        at_ms: i64,
    ) -> Result<()> {
        self.write(|tx, changes| {
            check_card(tx, note_id, card_key)?;
            let mark = Mark {
                id: new_entry_id("m", at_ms),
                card: card_key.to_owned(),
                at_ms,
                kind: if suspended {
                    MarkKind::Suspend
                } else {
                    MarkKind::Resume
                },
            };
            write_entry(tx, changes, note_id, at_ms, |doc| doc.marks.push(mark))
        })
    }

    /// The next due time of a card for each grade, as spans from `now_ms`.
    pub fn card_intervals(&self, card_key: &str, now_ms: i64) -> Result<[i64; 4]> {
        self.read(|conn| {
            let settings = settings_in(conn)?;
            let state = conn
                .query_row(
                    "SELECT due_ms, stability, difficulty, reps, lapses, phase, step, last_ms \
                     FROM card_state WHERE card_key = ?",
                    [card_key],
                    |row| state_from_row(row, 0),
                )
                .optional()?
                .unwrap_or_default();
            Ok(srs::preview(&state, now_ms, settings.retention))
        })
    }

    /// Forgets every answer and mark on this account (Settings > Flashcards
    /// > Reset Review History). The review documents become tombstones,
    /// which sync.
    pub fn reset_review_history(&self) -> Result<usize> {
        self.write(|tx, changes| {
            let ids: Vec<String> = tx
                .prepare("SELECT id FROM docs WHERE type = 'review' AND deleted = 0")?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            for id in &ids {
                if let Some(row) = load_row(tx, id)?.filter(|r| !r.deleted) {
                    delete_row(tx, id, &row, Origin::Ui, changes)?;
                }
            }
            tx.execute("DELETE FROM card_state", [])?;
            tx.execute("DELETE FROM card_log", [])?;
            Ok(ids.len())
        })
    }

    /// Everything the stats page shows for `scope`.
    pub fn card_stats(
        &self,
        scope: &CardScope,
        now_ms: i64,
        day_start_ms: i64,
    ) -> Result<CardStats> {
        self.read(|conn| {
            let counts = counts_in(conn, scope, now_ms)?;
            let (clause, extra) = scope_sql(scope);
            let params_vec = bind(&extra);
            // Answers per day over 30 days, restricted to the scope's cards.
            let first_day = day_start_ms - 29 * DAY_MS;
            let mut per_day = vec![0u32; 30];
            let mut again = 0u32;
            let mut total = 0u32;
            {
                let mut stmt = conn.prepare(&format!(
                    "SELECT cl.at, cl.grade FROM card_log cl \
                     JOIN note_cards nc ON nc.card_key = cl.card_key \
                     JOIN docs d ON d.id = nc.note_id \
                     WHERE {ELIGIBLE} {clause} AND cl.at >= {first_day}"
                ))?;
                let mut rows = stmt.query(params_vec.as_slice())?;
                while let Some(row) = rows.next()? {
                    let at: i64 = row.get(0)?;
                    let grade: i64 = row.get(1)?;
                    let day = ((at - first_day) / DAY_MS).clamp(0, 29) as usize;
                    per_day[day] += 1;
                    total += 1;
                    again += u32::from(grade == 1);
                }
            }
            let retention = (total > 0).then(|| f64::from(total - again) / f64::from(total));
            let mut forecast = vec![0u32; 7];
            {
                let mut stmt = conn.prepare(&format!(
                    "SELECT cs.due_ms FROM card_state cs \
                     JOIN note_cards nc ON nc.card_key = cs.card_key \
                     JOIN docs d ON d.id = nc.note_id \
                     WHERE {ELIGIBLE} {clause} AND cs.suspended = 0 AND cs.reps > 0 \
                     AND cs.due_ms < {}",
                    day_start_ms + 7 * DAY_MS
                ))?;
                let mut rows = stmt.query(params_vec.as_slice())?;
                while let Some(row) = rows.next()? {
                    let due: i64 = row.get(0)?;
                    let day = ((due - day_start_ms).max(0) / DAY_MS).min(6) as usize;
                    forecast[day] += 1;
                }
            }
            let mut stmt = conn.prepare(&format!(
                "SELECT nc.card_key, nc.note_id, IFNULL(d.title, ''), nc.line, nc.question, \
                 SUM(cl.grade = 1) AS agains \
                 FROM card_log cl JOIN note_cards nc ON nc.card_key = cl.card_key \
                 JOIN docs d ON d.id = nc.note_id \
                 WHERE {ELIGIBLE} {clause} GROUP BY nc.card_key HAVING agains > 0 \
                 ORDER BY agains DESC, nc.card_key LIMIT 10"
            ))?;
            let hardest = stmt
                .query_map(params_vec.as_slice(), |row| {
                    Ok(HardCard {
                        card_key: row.get(0)?,
                        note_id: row.get(1)?,
                        note_title: row.get(2)?,
                        line: row.get(3)?,
                        question: row.get(4)?,
                        agains: row.get(5)?,
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
            Ok(CardStats {
                counts,
                per_day,
                retention,
                forecast,
                hardest,
            })
        })
    }

    /// A note's cards as the sidebar groups them: how many each notebook
    /// holds, for the picker.
    pub fn card_books(&self) -> Result<Vec<(String, usize)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT d.book_id, COUNT(*) FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
                 WHERE {ELIGIBLE} GROUP BY d.book_id"
            ))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
                })?
                .collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
    }

    /// Whether any card exists in an eligible note (the sidebar shows
    /// Review only then).
    pub fn has_cards(&self) -> Result<bool> {
        self.read(|conn| {
            Ok(conn.query_row(
                &format!(
                    "SELECT EXISTS (SELECT 1 FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
                     WHERE {ELIGIBLE})"
                ),
                [],
                |row| row.get(0),
            )?)
        })
    }
}

fn counts_in(conn: &Connection, scope: &CardScope, now_ms: i64) -> Result<CardCounts> {
    let (clause, extra) = scope_sql(scope);
    let mut stmt = conn.prepare(&format!(
        "SELECT \
           COUNT(*), \
           IFNULL(SUM(IFNULL(cs.suspended, 0) = 0 AND cs.reps > 0 AND cs.due_ms <= {now_ms}), 0), \
           IFNULL(SUM(IFNULL(cs.suspended, 0) = 0 AND IFNULL(cs.reps, 0) = 0), 0), \
           IFNULL(SUM(IFNULL(cs.suspended, 0) = 1), 0) \
         FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
         LEFT JOIN card_state cs ON cs.card_key = nc.card_key \
         WHERE {ELIGIBLE} {clause}"
    ))?;
    let counts = stmt.query_row(bind(&extra).as_slice(), |row| {
        Ok(CardCounts {
            total: row.get::<_, i64>(0)? as usize,
            due: row.get::<_, i64>(1)? as usize,
            new: row.get::<_, i64>(2)? as usize,
            suspended: row.get::<_, i64>(3)? as usize,
        })
    })?;
    Ok(counts)
}

/// The card must exist in a note that may show cards.
fn check_card(tx: &Transaction, note_id: &str, card_key: &str) -> Result<()> {
    let found: bool = tx.query_row(
        &format!(
            "SELECT EXISTS (SELECT 1 FROM note_cards nc JOIN docs d ON d.id = nc.note_id \
             WHERE nc.note_id = ?1 AND nc.card_key = ?2 AND {ELIGIBLE})"
        ),
        params![note_id, card_key],
        |row| row.get(0),
    )?;
    if found {
        Ok(())
    } else {
        Err(StoreError::NotFound)
    }
}
