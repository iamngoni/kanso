//! Open loops, the morning brief and the weekly review on the phone.
//!
//! The rules are the desktop's (`kanso_core::loops`, `kanso_engine::brief`):
//! which sentences are loops, what the brief holds, what the review counts
//! and which notifications to plan. This file reads the library, turns the
//! rows into what the screens show, and keeps the settings (as JSON in the
//! library's local meta, so they stay on this phone).
//!
//! Locked notes and notes in locked notebooks have no loops and are in no
//! brief or review: the store writes nothing for them.

use std::collections::HashMap;

use chrono::{Local, Offset as _, TimeZone as _};
use kanso_core::dates::{Date, DateTime, format};
use kanso_core::loops::{LoopKind, age_label};
use kanso_core::now_ms;
use kanso_core::tasks::Due;
use kanso_engine::brief::{self, BriefSettings, Now, ThemeKind};
use kanso_engine::tasks::chip_label;
use kanso_store::{LoopClose, LoopQuery, LoopRow, Origin, TaskRow};

use crate::boundary::guard;
use crate::brief_model::{
    BriefNote, BriefPrefs, BriefReminder, BriefView, LoopAction, OpenLoopItem, OpenLoopKind,
    OpenLoopsBoard, PlannedNotice, ReviewFinished, ReviewSaved, ReviewTheme, ReviewView,
    ReviewWritten,
};
use crate::errors::KansoError;
use crate::library::Library;

/// Where the settings are kept.
const SETTINGS_META: &str = "brief.settings";
/// The most loops one list holds.
const LIST_LIMIT: usize = 500;

impl Library {
    /// The open loops across the notes, newest notes first. Empty (and
    /// `enabled` false) when the detector is off in Settings.
    pub fn open_loops(&self) -> Result<OpenLoopsBoard, KansoError> {
        guard(|| {
            let enabled = self.brief_prefs_now().loops;
            if !enabled {
                return Ok(OpenLoopsBoard {
                    items: Vec::new(),
                    total: 0,
                    enabled,
                });
            }
            let store = &self.inner.store;
            let now = now_here();
            let query = LoopQuery {
                limit: Some(LIST_LIMIT),
                ..LoopQuery::default()
            };
            let rows = store.open_loops(&query)?;
            let total = store.open_loop_count(&LoopQuery::default())?;
            Ok(OpenLoopsBoard {
                items: rows.iter().map(|row| loop_item(row, now)).collect(),
                total,
                enabled,
            })
        })
    }

    /// Closes loop `key` of note `note_id` as `action`. `line` is where the
    /// person saw it; a loop that moved is found by its sentence. Returns
    /// whether anything changed (false when the loop is gone from the note).
    pub fn close_loop(
        &self,
        note_id: &str,
        key: &str,
        line: u32,
        action: LoopAction,
    ) -> Result<bool, KansoError> {
        guard(|| {
            let how = match action {
                LoopAction::Done => LoopClose::Done {
                    mark_in_note: self.brief_prefs_now().mark_done_in_note,
                },
                LoopAction::NotALoop => LoopClose::Dismissed,
                LoopAction::MakeTask => LoopClose::MakeTask,
            };
            let outcome =
                self.inner
                    .store
                    .close_loop(note_id, key, line, how, now_ms(), Origin::Ui)?;
            Ok(!matches!(outcome, kanso_store::CheckOff::Missing))
        })
    }

    /// The morning brief at the phone's local time now.
    pub fn brief(&self) -> Result<BriefView, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let now = now_here();
            let brief = brief::morning_brief(store, now, None)?;
            let names = self.notebook_names()?;
            let local = now.local();
            Ok(BriefView {
                date_label: format(&local, "%A %-d %B"),
                due: brief
                    .due
                    .iter()
                    .map(|row| crate::tasks::item(row, names.get(&row.book_id), local))
                    .collect(),
                loops: brief.loops.iter().map(|row| loop_item(row, now)).collect(),
                loops_total: brief.loops_total,
                later_today: brief
                    .later_today
                    .iter()
                    .map(|task| BriefReminder {
                        note_id: task.note_id.clone(),
                        note_title: task.note_title.clone(),
                        line: task.line,
                        text: kanso_engine::tasks::display_text(&task.text),
                        time_label: Due::parse(&task.due)
                            .map(|due| chip_label(due, false, local))
                            .unwrap_or_default(),
                    })
                    .collect(),
                relevant: brief
                    .relevant
                    .iter()
                    .map(|note| BriefNote {
                        note_id: note.id.clone(),
                        title: note.title.clone(),
                        why: note.why.label().to_owned(),
                        excerpt: note.excerpt.clone(),
                    })
                    .collect(),
                wrote_yesterday: brief.yesterday.wrote,
                finished_yesterday: brief.yesterday.finished,
                is_empty: brief.is_empty(),
            })
        })
    }

    /// The weekly review of the seven days from `from` (`YYYY-MM-DD`, moved
    /// back to its Monday), or of this week when `from` is `None`.
    pub fn review(&self, from: Option<String>) -> Result<ReviewView, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let now = now_here();
            let this_week = brief::week_start(now.today());
            let start = match from.as_deref() {
                Some(text) => brief::week_start(
                    Date::parse_iso(text)
                        .ok_or_else(|| KansoError::invalid("That isn't a date."))?,
                ),
                None => this_week,
            };
            let review = brief::weekly_review(store, start, now)?;
            let names = self.notebook_names()?;
            let local = now.local();
            let end = start.add_days(6);
            Ok(ReviewView {
                from: start.iso(),
                week_label: brief::iso_week_label(start),
                range_label: format!("{} to {}", short_day(start), short_day(end)),
                counts_text: review.counts_text(),
                written: review
                    .written
                    .iter()
                    .map(|note| ReviewWritten {
                        note_id: note.id.clone(),
                        title: note.title.clone(),
                        notebook_name: note.book_name.clone(),
                        created: note.created,
                    })
                    .collect(),
                finished: review
                    .finished
                    .iter()
                    .map(|row| ReviewFinished {
                        note_id: row.note_id.clone(),
                        note_title: row.note_title.clone(),
                        text: row.text.clone(),
                        was_loop: row.kind == "loop",
                    })
                    .collect(),
                still_open: review
                    .still_open
                    .iter()
                    .map(|row| loop_item(row, now))
                    .collect(),
                overdue: tasks_of(&review.overdue, &names, local),
                themes: review
                    .themes
                    .iter()
                    .map(|theme| ReviewTheme {
                        term: theme.term.clone(),
                        notes: theme.notes,
                        is_tag: theme.kind == ThemeKind::Tag,
                    })
                    .collect(),
                next: tasks_of(&review.next, &names, local),
                saved_note_id: brief::saved_review(store, start)?,
                is_this_week: start == this_week,
            })
        })
    }

    /// Saves the review of the week from `from` as a note in the Journal.
    /// With `replace` false, a week already saved is left alone and
    /// `exists` is true.
    pub fn save_review(&self, from: String, replace: bool) -> Result<ReviewSaved, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let start = brief::week_start(
                Date::parse_iso(&from).ok_or_else(|| KansoError::invalid("That isn't a date."))?,
            );
            let review = brief::weekly_review(store, start, now_here())?;
            Ok(match brief::save_review(store, &review, replace, Origin::Ui)? {
                brief::Saved::Written { id } => ReviewSaved {
                    note_id: id,
                    exists: false,
                },
                brief::Saved::Exists { id } => ReviewSaved {
                    note_id: id,
                    exists: true,
                },
            })
        })
    }

    /// The brief and review settings.
    pub fn brief_prefs(&self) -> Result<BriefPrefs, KansoError> {
        guard(|| Ok(prefs_of(&self.brief_prefs_now())))
    }

    /// Saves the brief and review settings (times and days are clamped to
    /// ones that exist).
    pub fn set_brief_prefs(&self, prefs: BriefPrefs) -> Result<BriefPrefs, KansoError> {
        guard(|| {
            let settings = settings_of(&prefs).sanitized();
            let text = serde_json::to_string(&settings)
                .map_err(|err| KansoError::storage(err.to_string()))?;
            self.inner.store.set_meta(SETTINGS_META, &text)?;
            Ok(prefs_of(&settings))
        })
    }

    /// The notifications to have the system show over the next week, in
    /// place of any it was given before. Their text is fixed. Empty when
    /// `can_notify` is false (the system said no).
    pub fn brief_plan(&self, can_notify: bool) -> Result<Vec<PlannedNotice>, KansoError> {
        guard(|| {
            if !can_notify {
                return Ok(Vec::new());
            }
            let settings = self.brief_prefs_now();
            Ok(brief::plan(&settings, now_here())
                .into_iter()
                .map(|planned| PlannedNotice {
                    id: format!("{}-{}", planned.kind.as_str(), planned.date.iso()),
                    is_review: planned.kind == brief::NoticeKind::Review,
                    title: planned.title.to_owned(),
                    body: planned.body.to_owned(),
                    year: planned.date.year,
                    month: planned.date.month,
                    day: planned.date.day,
                    hour: planned.hour,
                    minute: planned.minute,
                })
                .collect())
        })
    }

    /// The saved settings, or the defaults.
    fn brief_prefs_now(&self) -> BriefSettings {
        self.inner
            .store
            .meta(SETTINGS_META)
            .ok()
            .flatten()
            .and_then(|text| serde_json::from_str::<BriefSettings>(&text).ok())
            .unwrap_or_default()
            .sanitized()
    }

    fn notebook_names(&self) -> Result<HashMap<String, String>, KansoError> {
        Ok(self
            .inner
            .store
            .books()?
            .into_iter()
            .map(|book| (book.id, book.name))
            .collect())
    }
}

fn tasks_of(
    rows: &[TaskRow],
    names: &HashMap<String, String>,
    now: DateTime,
) -> Vec<crate::model::TaskItem> {
    rows.iter()
        .map(|row| crate::tasks::item(row, names.get(&row.book_id), now))
        .collect()
}

/// "2 Nov".
fn short_day(date: Date) -> String {
    format(
        &DateTime {
            date,
            hour: 0,
            minute: 0,
            second: 0,
        },
        "%-d %b",
    )
}

fn loop_item(row: &LoopRow, now: Now) -> OpenLoopItem {
    let due = row.due.as_deref().and_then(Due::parse);
    OpenLoopItem {
        key: row.key.clone(),
        note_id: row.note_id.clone(),
        note_title: row.note_title.clone(),
        notebook_id: row.book_id.clone(),
        line: row.line,
        kind: match row.kind {
            LoopKind::Mine => OpenLoopKind::Mine,
            LoopKind::Waiting => OpenLoopKind::Waiting,
            LoopKind::Question => OpenLoopKind::Question,
        },
        text: row.text.clone(),
        due_label: due.map(|due| chip_label(due, false, now.local())),
        age_label: age_label(row.note_updated_at, now.utc_ms),
        from_ai: row.source == kanso_store::LoopSource::Ai,
    }
}

fn prefs_of(s: &BriefSettings) -> BriefPrefs {
    BriefPrefs {
        loops: s.loops,
        brief: s.brief,
        brief_hour: s.brief_hour,
        brief_minute: s.brief_minute,
        brief_weekdays_only: s.brief_weekdays_only,
        review: s.review,
        review_day: s.review_day,
        review_hour: s.review_hour,
        review_minute: s.review_minute,
        notify_brief: s.notify_brief,
        notify_review: s.notify_review,
        mark_done_in_note: s.mark_done_in_note,
    }
}

fn settings_of(p: &BriefPrefs) -> BriefSettings {
    BriefSettings {
        loops: p.loops,
        brief: p.brief,
        brief_hour: p.brief_hour,
        brief_minute: p.brief_minute,
        brief_weekdays_only: p.brief_weekdays_only,
        review: p.review,
        review_day: p.review_day,
        review_hour: p.review_hour,
        review_minute: p.review_minute,
        notify_brief: p.notify_brief,
        notify_review: p.notify_review,
        mark_done_in_note: p.mark_done_in_note,
    }
}

/// The phone's time now and its zone's offset from UTC.
fn now_here() -> Now {
    let utc_ms = now_ms();
    let offset = Local
        .timestamp_millis_opt(utc_ms)
        .single()
        .map_or(0, |at| i64::from(at.offset().fix().local_minus_utc()));
    Now {
        utc_ms,
        offset_ms: offset * 1000,
    }
}

#[cfg(test)]
mod tests {
    use super::{prefs_of, settings_of};
    use kanso_engine::brief::BriefSettings;

    #[test]
    fn settings_round_trip_through_the_phone_model() {
        let s = BriefSettings::default();
        assert_eq!(settings_of(&prefs_of(&s)), s);
    }
}
