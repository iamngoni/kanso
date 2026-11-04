//! Assembling the week: where it went (*Active projects*), what was
//! finished, who is being waited on, what is still open, what is ahead and
//! the one-line extras.

use std::collections::{BTreeMap, HashMap};

use kanso_core::dates::{Date, DateTime, format};
use kanso_store::{CardScope, Result, Store, TaskQuery};

use super::lede::{self, WeekFacts};
use super::model::*;
use super::today::{Options, book_name, task_row, waiting_row};
use crate::brief::{self, Now, ThemeKind};
use crate::tasks::display_text;

const DAY_MS: i64 = 86_400_000;
/// *Active projects* shows this many.
const PROJECTS: usize = 4;
/// *Finished* shows this many before `N more`.
const FINISHED_SHOWN: usize = 5;
/// *Waiting on others* and *Still open* show this many.
const LIST_SHOWN: usize = 5;
/// *Ahead* shows this many rows in all.
const AHEAD_ROWS: usize = 8;

fn range_label(from: Date, to: Date) -> String {
    let month = |d: Date| format(&DateTime::at_midnight(d), "%B").to_uppercase();
    if from.month == to.month {
        format!("{} \u{2013} {} {}", from.day, to.day, month(to))
    } else {
        format!("{} {} \u{2013} {} {}", from.day, month(from), to.day, month(to))
    }
}

fn weekday(date: Date) -> String {
    format(&DateTime::at_midnight(date), "%A")
}

fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The week starting `week_start`, seen at `now`.
pub fn week(store: &Store, week_start: Date, now: Now, opts: &Options) -> Result<Week> {
    let review = brief::weekly_review(store, week_start, now)?;
    let today = now.today();
    let from = now.start_of(week_start);
    let to = now.start_of(week_start.add_days(7));
    let mut names: HashMap<String, String> = HashMap::new();

    // Active projects: notebooks, by writing sessions and then notes changed.
    let sessions: HashMap<String, u32> = store
        .writing_sessions_by_book(from, to)?
        .into_iter()
        .collect();
    let mut by_book: BTreeMap<String, Vec<&brief::WrittenNote>> = BTreeMap::new();
    for note in &review.written {
        by_book.entry(note.book_id.clone()).or_default().push(note);
    }
    let mut projects: Vec<ProjectRow> = Vec::new();
    for (book, notes) in &by_book {
        if book.is_empty() || book == "trash" {
            continue;
        }
        let last = notes.iter().max_by_key(|n| n.updated_at);
        let next = store
            .tasks(&TaskQuery {
                book_id: Some(book.clone()),
                limit: Some(1),
                ..TaskQuery::default()
            })?
            .into_iter()
            .next();
        projects.push(ProjectRow {
            id: book.clone(),
            name: book_name(store, &mut names, book)?,
            kind: "book".into(),
            notes_changed: notes.len() as u32,
            sessions: sessions.get(book).copied().unwrap_or(0),
            last_title: last.map(|n| n.title.clone()).unwrap_or_default(),
            last_note: last.map(|n| n.id.clone()).unwrap_or_default(),
            last_at: last.map_or(0, |n| n.updated_at),
            next_task: next.as_ref().map(|t| display_text(&t.text)).unwrap_or_default(),
            next_due: next.and_then(|t| t.due).unwrap_or_default(),
        });
    }
    projects.sort_by(|a, b| {
        b.sessions
            .cmp(&a.sessions)
            .then(b.notes_changed.cmp(&a.notes_changed))
            .then(a.name.cmp(&b.name))
    });
    projects.truncate(PROJECTS);

    // Finished.
    let mut finished: Vec<&kanso_store::FinishedRow> = review.finished.iter().collect();
    finished.sort_by(|a, b| b.at.cmp(&a.at));
    let finished_more = finished.len().saturating_sub(FINISHED_SHOWN) as u32;
    let finished_items: Vec<FinishedItem> = finished
        .iter()
        .take(FINISHED_SHOWN)
        .map(|f| FinishedItem {
            note_id: f.note_id.clone(),
            text: display_text(&f.text),
            at: f.at,
            kind: f.kind.clone(),
        })
        .collect();

    // Waiting on others.
    let waiting: Vec<ListRow> = store
        .waiting_items(now.utc_ms, opts.quiet_days.clamp(3, 14), LIST_SHOWN)?
        .into_iter()
        .map(|r| waiting_row(r, now.utc_ms))
        .collect();

    // Still open: overdue tasks, the oldest first.
    let mut open_rows: Vec<AgendaRow> = review
        .overdue
        .iter()
        .filter_map(|t| task_row(t, today))
        .collect();
    open_rows.sort_by(|a, b| b.days_late.cmp(&a.days_late).then(a.title.cmp(&b.title)));
    let still_open_total = open_rows.len() as u32;
    open_rows.truncate(LIST_SHOWN);

    // Ahead: past days of this week as one line each, then what is due.
    let mut ahead: Vec<AheadDay> = Vec::new();
    let mut finished_by_day: HashMap<i64, u32> = HashMap::new();
    for f in &review.finished {
        *finished_by_day
            .entry((f.at + now.offset_ms).div_euclid(DAY_MS))
            .or_default() += 1;
    }
    let mut wrote_by_day: HashMap<i64, u32> = HashMap::new();
    for n in review.written.iter().filter(|n| n.created) {
        *wrote_by_day
            .entry((n.updated_at + now.offset_ms).div_euclid(DAY_MS))
            .or_default() += 1;
    }
    for offset in 0..7 {
        let day = week_start.add_days(offset);
        if day >= today {
            break;
        }
        let key = day.to_days();
        let (wrote, done) = (
            wrote_by_day.get(&key).copied().unwrap_or(0),
            finished_by_day.get(&key).copied().unwrap_or(0),
        );
        let mut parts = Vec::new();
        if wrote > 0 {
            parts.push(format!("{} written", plural(wrote, "note", "notes")));
        }
        if done > 0 {
            parts.push(format!("{} finished", plural(done, "task", "tasks")));
        }
        if !parts.is_empty() {
            ahead.push(AheadDay {
                day: day.iso(),
                summary: format!("{} \u{b7} {}", weekday(day), parts.join(" \u{b7} ")),
                rows: Vec::new(),
            });
        }
    }
    let first_ahead = today.max(week_start);
    let last_ahead = week_start.add_days(13);
    if last_ahead >= first_ahead {
        let due = store.tasks(&TaskQuery {
            due_after: Some(first_ahead.add_days(-1).iso()),
            due_before: Some(last_ahead.add_days(1).iso()),
            ..TaskQuery::default()
        })?;
        let mut by_day: BTreeMap<String, Vec<AgendaRow>> = BTreeMap::new();
        for task in &due {
            if let Some(row) = task_row(task, today)
                && let Some(day) = task.due.as_deref().and_then(|d| d.get(..10))
            {
                by_day.entry(day.to_owned()).or_default().push(row);
            }
        }
        let mut room = AHEAD_ROWS;
        for (day, mut rows) in by_day {
            if room == 0 {
                break;
            }
            rows.sort_by(|a, b| a.at_min.cmp(&b.at_min).then(a.title.cmp(&b.title)));
            rows.truncate(room);
            room -= rows.len();
            let date = Date::parse_iso(&day).unwrap_or(first_ahead);
            ahead.push(AheadDay {
                summary: format(&DateTime::at_midnight(date), "%A %-d"),
                day,
                rows,
            });
        }
    }

    // Also: themes, revisits this week, the Inbox.
    let mut also = Vec::new();
    if !review.themes.is_empty() {
        also.push(AlsoLine {
            kind: AlsoKind::Themes,
            count: review.themes.len() as u32,
            cards: 0,
            items: Vec::new(),
            detail: review
                .themes
                .iter()
                .take(5)
                .map(|t| match t.kind {
                    ThemeKind::Tag => format!("#{}", t.term),
                    _ => t.term.clone(),
                })
                .collect(),
        });
    }
    let week_end = week_start.add_days(6);
    let revisits = store.revisits_between(
        Some(&week_start.add_days(-1).iso()),
        &week_end.iso(),
        50,
    )?;
    let cards = store
        .due_cards(&CardScope::All, now.utc_ms, now.start_of(today))?
        .len() as u32;
    if !revisits.is_empty() {
        also.push(AlsoLine {
            kind: AlsoKind::Revisit,
            count: revisits.len() as u32,
            cards,
            items: revisits
                .iter()
                .take(2)
                .map(|r| Item {
                    note_id: r.note_id.clone(),
                    line: 0,
                    text: r.title.clone(),
                    voice_secs: 0,
                })
                .collect(),
            detail: Vec::new(),
        });
    }
    let inbox = store.inbox_waiting(kanso_core::roles::Role::Inbox.book_id())?;
    if !inbox.is_empty() {
        also.push(AlsoLine {
            kind: AlsoKind::Captured,
            count: inbox.len() as u32,
            cards: 0,
            items: Vec::new(),
            detail: Vec::new(),
        });
    }

    let wrote = review.new_count() as u32;
    let changed = review.changed_count() as u32;
    let finished_count = review.finished.len() as u32;
    let top: Vec<String> = projects.iter().take(2).map(|p| p.name.clone()).collect();
    let saved_as = brief::saved_review(store, week_start)?
        .map(|_| brief::review_title(week_start))
        .unwrap_or_default();

    Ok(Week {
        from: week_start.iso(),
        to: week_end.iso(),
        range_label: range_label(week_start, week_end),
        week_label: format!("Week {}", week_start.add_days(3).iso_week()),
        lede: lede::week_lede(&WeekFacts {
            wrote,
            changed,
            finished: finished_count,
            top,
            meetings: None,
        }),
        projects,
        finished: finished_items,
        finished_more,
        waiting,
        still_open: open_rows,
        still_open_total,
        ahead,
        also,
        wrote,
        changed,
        finished_count,
        saved_as,
    })
}
