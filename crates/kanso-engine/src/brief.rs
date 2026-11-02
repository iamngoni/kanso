//! The morning brief and the weekly review, computed from the library when
//! they are opened: nothing is stored, nothing runs in the background, and
//! no clock or window is involved (the caller passes the time).
//!
//! Locked notes and notes in locked notebooks never appear: the derived
//! tables hold nothing of them, and the notes this suggests or counts for
//! themes pass the store's `discoverable` rule (also not `#ai-exclude`,
//! not in the trash).

mod markdown;
mod schedule;
mod themes;

pub use markdown::{iso_week_label, review_markdown, review_title};
pub use schedule::{
    BriefSettings, GRACE_MS, NoticeKind, PLAN_DAYS, Planned, due_now, fire_at, next_wake, plan,
    shown_key,
};
pub use themes::{MAX_TAGS, MAX_TERMS, MIN_NOTES, Theme, ThemeKind, themes};

use kanso_core::dates::{Date, DateTime};
use kanso_core::loops::age_days;
use kanso_core::reminders::task_key;
use kanso_core::roles::daily_id;
use kanso_store::{
    FinishedRow, LoopQuery, LoopRow, ReminderTask, Result, Store, TaskQuery, TaskRow, ThemeNote,
};

const DAY_MS: i64 = 86_400_000;

/// At most this many loops are on the brief.
pub const BRIEF_LOOPS: usize = 7;
/// A loop in a note not changed for this long is "still open from last
/// week".
pub const STALE_DAYS: i64 = 7;
/// How many past notes the brief suggests.
pub const RELEVANT_NOTES: usize = 2;
/// The most notes read for themes.
pub const THEME_NOTE_CAP: usize = 20_000;

/// The time as a phone or desktop knows it: UTC milliseconds and the
/// zone's offset from UTC at that moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Now {
    pub utc_ms: i64,
    pub offset_ms: i64,
}

impl Now {
    pub fn local_ms(self) -> i64 {
        self.utc_ms.saturating_add(self.offset_ms)
    }

    pub fn local(self) -> DateTime {
        DateTime::from_local_ms(self.local_ms())
    }

    pub fn today(self) -> Date {
        self.local().date
    }

    /// The UTC time of local midnight starting `date`.
    pub fn start_of(self, date: Date) -> i64 {
        date.to_days()
            .saturating_mul(DAY_MS)
            .saturating_sub(self.offset_ms)
    }
}

/// Why a past note is suggested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// Today's daily note links to it.
    Linked,
    /// The daily note from a week ago.
    WeekAgo,
    /// The daily note from a month ago.
    MonthAgo,
    /// The daily note from a year ago ("A year ago today").
    YearAgo,
    /// Similar to today's daily note or to a task due today.
    Similar,
}

impl Why {
    pub fn label(self) -> &'static str {
        match self {
            Why::Linked => "Linked from today",
            Why::WeekAgo => "A week ago today",
            Why::MonthAgo => "A month ago today",
            Why::YearAgo => "A year ago today",
            Why::Similar => "Related to today",
        }
    }
}

/// A past note the brief suggests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelevantNote {
    pub id: String,
    pub title: String,
    pub why: Why,
    /// When it was last changed (UTC ms).
    pub updated_at: i64,
    /// Two lines of its text.
    pub excerpt: String,
}

/// What yesterday held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DayCounts {
    pub wrote: u32,
    pub finished: u32,
}

/// The morning brief.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Brief {
    /// Open tasks due today or earlier, overdue first.
    pub due: Vec<TaskRow>,
    /// The loops shown (at most [`BRIEF_LOOPS`]).
    pub loops: Vec<LoopRow>,
    /// How many loops are open in all.
    pub loops_total: u32,
    /// Timed tasks due later today.
    pub later_today: Vec<ReminderTask>,
    pub relevant: Vec<RelevantNote>,
    pub yesterday: DayCounts,
}

impl Brief {
    /// Nothing due and no open loops: the brief says so instead of showing
    /// empty sections.
    pub fn is_empty(&self) -> bool {
        self.due.is_empty() && self.loops.is_empty() && self.later_today.is_empty()
    }

    /// The counts a notification says: "3 due, 5 open loops".
    pub fn notification_text(&self) -> String {
        notice_text(self.due.len() as u32, self.loops_total)
    }
}

/// What the brief notification says, from counts alone (never note text).
pub fn notice_text(due: u32, loops: u32) -> String {
    let mut parts = Vec::new();
    if due > 0 {
        parts.push(format!("{due} due"));
    }
    if loops > 0 {
        parts.push(format!(
            "{loops} open {}",
            if loops == 1 { "loop" } else { "loops" }
        ));
    }
    if parts.is_empty() {
        "Nothing due today".to_owned()
    } else {
        parts.join(", ")
    }
}

/// The loops the brief shows: the newest first, then any older than
/// [`STALE_DAYS`] ("still open from last week"), at most [`BRIEF_LOOPS`].
fn pick_loops(all: Vec<LoopRow>, now_ms: i64) -> Vec<LoopRow> {
    // The store lists the newest notes first.
    let (mut stale, mut recent): (Vec<LoopRow>, Vec<LoopRow>) = all
        .into_iter()
        .partition(|l| age_days(l.note_updated_at, now_ms) >= STALE_DAYS);
    stale.sort_by_key(|l| l.note_updated_at);
    stale.truncate(BRIEF_LOOPS / 2);
    recent.truncate(BRIEF_LOOPS - stale.len());
    recent.extend(stale);
    recent
}

/// Today's brief at `now`. `model` names the embedding model in use, when
/// there is one: notes nearest to today's daily note and to the notes of
/// the tasks due today are then suggested too.
pub fn morning_brief(store: &Store, now: Now, model: Option<&str>) -> Result<Brief> {
    let today = now.today();
    let tomorrow = today.add_days(1);
    let due = store.tasks(&TaskQuery {
        due_before: Some(tomorrow.iso()),
        ..TaskQuery::default()
    })?;

    let loops_total = store.open_loop_count(&LoopQuery::default())?;
    let all = store.open_loops(&LoopQuery::default())?;
    let loops = pick_loops(all, now.utc_ms);

    let local = now.local();
    let today_iso = today.iso();
    let clock = format!("{:02}:{:02}", local.hour, local.minute);
    let later_today: Vec<ReminderTask> = store
        .reminder_tasks()?
        .into_iter()
        .filter(|t| {
            t.due.get(..10) == Some(today_iso.as_str())
                && t.due.get(11..16).is_some_and(|time| time > clock.as_str())
                && !t.acks.contains_key(&task_key(&t.note_id, &t.text, &t.due))
        })
        .collect();

    let relevant = relevant_notes(store, today, model, &due)?;

    let from = now.start_of(today.add_days(-1));
    let to = now.start_of(today);
    let wrote = store
        .notes_changed_between(from, to)?
        .iter()
        .filter(|n| n.created_at >= from && n.created_at < to)
        .count() as u32;
    let finished = store.finished_between(from, to)?.len() as u32;

    Ok(Brief {
        due,
        loops,
        loops_total,
        later_today,
        relevant,
        yesterday: DayCounts { wrote, finished },
    })
}

fn relevant_notes(
    store: &Store,
    today: Date,
    model: Option<&str>,
    due: &[TaskRow],
) -> Result<Vec<RelevantNote>> {
    let daily = daily_id(today);
    let mut candidates: Vec<(String, Why)> = Vec::new();
    if let Some(model) = model {
        let mut seeds = vec![daily.clone()];
        seeds.extend(due.iter().map(|t| t.note_id.clone()).take(2));
        for seed in seeds {
            if let Some(near) = store.related_notes(&seed, model, 3)? {
                candidates.extend(near.into_iter().map(|n| (n.id, Why::Similar)));
            }
        }
    }
    for id in store.linked_note_ids(&daily)? {
        candidates.push((id, Why::Linked));
    }
    for (days, why) in [(365, Why::YearAgo), (30, Why::MonthAgo), (7, Why::WeekAgo)] {
        candidates.push((daily_id(today.add_days(-days)), why));
    }
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|(id, _)| id != &daily && seen.insert(id.clone()));
    let ids: Vec<String> = candidates.iter().map(|(id, _)| id.clone()).collect();
    let notes = store.brief_notes(&ids)?;
    Ok(notes
        .into_iter()
        .take(RELEVANT_NOTES)
        .map(|n| RelevantNote {
            why: candidates
                .iter()
                .find(|(id, _)| id == &n.id)
                .map_or(Why::Linked, |(_, why)| *why),
            id: n.id,
            title: n.title,
            updated_at: n.updated_at,
            excerpt: n.excerpt,
        })
        .collect())
}

/// A note written or changed in the reviewed week.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenNote {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub book_name: String,
    /// Created in the week (otherwise only changed).
    pub created: bool,
    pub updated_at: i64,
}

/// The weekly review of seven days from `from`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Review {
    /// The first day (a Monday by default) and the last.
    pub from: Date,
    pub to: Date,
    pub written: Vec<WrittenNote>,
    pub finished: Vec<FinishedRow>,
    /// Open loops, oldest first.
    pub still_open: Vec<LoopRow>,
    /// Open tasks due before today, oldest first.
    pub overdue: Vec<TaskRow>,
    pub themes: Vec<Theme>,
    /// Tasks due in the seven days after the week.
    pub next: Vec<TaskRow>,
}

impl Review {
    pub fn new_count(&self) -> usize {
        self.written.iter().filter(|n| n.created).count()
    }

    pub fn changed_count(&self) -> usize {
        self.written.len() - self.new_count()
    }

    /// "12 new, 31 changed".
    pub fn counts_text(&self) -> String {
        format!("{} new, {} changed", self.new_count(), self.changed_count())
    }

    /// The text the weekly notification says: counts only.
    pub fn notification_text(&self) -> String {
        format!(
            "{} {} written, {} finished",
            self.written.len(),
            if self.written.len() == 1 { "note" } else { "notes" },
            self.finished.len()
        )
    }
}

/// The Monday of the week holding `date`.
pub fn week_start(date: Date) -> Date {
    date.add_days(-(date.weekday() as i64))
}

/// The review of the week starting `week_start` (seven days), seen at
/// `now`.
pub fn weekly_review(store: &Store, week_start: Date, now: Now) -> Result<Review> {
    let to_day = week_start.add_days(6);
    let from = now.start_of(week_start);
    let to = now.start_of(week_start.add_days(7));

    let mut written = Vec::new();
    for note in store.notes_changed_between(from, to)? {
        let book_name = store
            .get(&note.book_id)?
            .and_then(|d| d.fields.get("name").and_then(|n| n.as_str().map(str::to_owned)))
            .unwrap_or_default();
        written.push(WrittenNote {
            created: note.created_at >= from && note.created_at < to,
            id: note.id,
            title: note.title,
            book_id: note.book_id,
            book_name,
            updated_at: note.updated_at,
        });
    }

    let finished = store.finished_between(from, to)?;

    let mut still_open = store.open_loops(&LoopQuery::default())?;
    still_open.sort_by(|a, b| {
        a.note_updated_at
            .cmp(&b.note_updated_at)
            .then_with(|| a.note_id.cmp(&b.note_id))
            .then_with(|| a.line.cmp(&b.line))
    });
    let today = now.today();
    let overdue = store.tasks(&TaskQuery {
        due_before: Some(today.iso()),
        ..TaskQuery::default()
    })?;

    let theme_notes: Vec<ThemeNote> = store.theme_notes(from, to, THEME_NOTE_CAP)?;
    let themes = themes(&theme_notes);

    let next = store.tasks(&TaskQuery {
        due_after: Some(to_day.iso()),
        due_before: Some(to_day.add_days(8).iso()),
        ..TaskQuery::default()
    })?;

    Ok(Review {
        from: week_start,
        to: to_day,
        written,
        finished,
        still_open,
        overdue,
        themes,
        next,
    })
}
