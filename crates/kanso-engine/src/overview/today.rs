//! Assembling Today from the library: the agenda, the three Lists, the
//! Lines of *Also*, the note from the past and the footer.

use std::collections::{HashMap, HashSet};

use kanso_core::DocType;
use kanso_core::capture_mark::read_duration_ms;
use kanso_core::dates::{Date, DateTime, format};
use kanso_core::loop_score::{self, Context};
use kanso_core::loops::LoopKind;
use kanso_core::reminders::task_key;
use kanso_core::roles::Role;
use kanso_core::writing;
use kanso_store::{CardScope, LoopQuery, Result, Store, TaskQuery, TaskRow, WaitingRow};

use super::density::{self, ALSO_CAP};
use super::lede::{self, Next, TodayFacts};
use super::model::*;
use super::{agenda, past};
use crate::brief::Now;
use crate::tasks::display_text;

const DAY_MS: i64 = 86_400_000;
/// At most this many loops are scored for the Promises line.
const LOOP_SCAN: usize = 2_000;
/// How many revisits the Revisit line looks at.
const REVISIT_SCAN: usize = 12;

/// Something an agent or a person you share with did, handed in by the
/// app: agent suggestions are queued in memory (`review_queue`), so the
/// store doesn't know them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityItem {
    pub kind: RowKind,
    pub note_id: String,
    pub title: String,
    pub who: String,
    pub count: u32,
    pub at: i64,
}

/// What the caller adds to the page.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Days of quiet before an item counts as waiting (3 to 14).
    pub quiet_days: u32,
    /// The note open in the window, which *Pick up* leaves out.
    pub open_note: Option<String>,
    /// The embedding model in use, for the note from the past.
    pub model: Option<String>,
    /// Today's calendar events, read from the device's calendars.
    pub events: Vec<Event>,
    /// Agent suggestions waiting for review.
    pub agent: Vec<ActivityItem>,
    /// Whether open loops are detected at all.
    pub loops: bool,
    /// Meeting notes for events, keyed `uid@start`.
    pub meetings: HashMap<String, MeetingLink>,
}

impl Options {
    pub fn new() -> Options {
        Options {
            quiet_days: 5,
            loops: true,
            ..Options::default()
        }
    }
}

fn minutes_into_day(ms: i64, day_start: i64) -> u32 {
    ((ms - day_start).clamp(0, DAY_MS) / 60_000) as u32
}

fn date_of_due(due: &str) -> Option<Date> {
    Date::parse_iso(due.get(..10)?)
}

fn time_of_due(due: &str) -> Option<u32> {
    let time = due.get(11..16)?;
    let (h, m) = time.split_once(':')?;
    Some(h.parse::<u32>().ok()? * 60 + m.parse::<u32>().ok()?)
}

pub(super) fn task_row(task: &TaskRow, today: Date) -> Option<AgendaRow> {
    let due = task.due.clone()?;
    let date = date_of_due(&due)?;
    let late = (today.to_days() - date.to_days()).max(0) as u32;
    let at_min = (late == 0).then(|| time_of_due(&due)).flatten();
    let kind = if late > 0 {
        AgendaKind::Overdue
    } else if at_min.is_some() {
        AgendaKind::Reminder
    } else {
        AgendaKind::Due
    };
    Some(AgendaRow {
        kind,
        key: task_key(&task.note_id, &task.text, ""),
        title: display_text(&task.text),
        context: task.note_title.clone(),
        at_min,
        end_min: None,
        days_late: late,
        now: false,
        free: false,
        note_id: task.note_id.clone(),
        line: task.line,
        is_loop: false,
        due,
        meeting: None,
        color: String::new(),
        event: None,
    })
}

/// The line under an event's title: its time range, who is in it, and where.
fn event_context(event: &Event, day_start: i64) -> String {
    let (s, e) = (
        minutes_into_day(event.start, day_start),
        minutes_into_day(event.end, day_start),
    );
    let mut parts = vec![format!("{}\u{2013}{}", clock(s), clock(e))];
    let names: Vec<&str> = event
        .attendees
        .iter()
        .map(|a| a.name.as_str())
        .filter(|n| !n.is_empty())
        .collect();
    match names.len() {
        0 => {}
        1..=8 => parts.push(names.join(", ")),
        n => parts.push(format!("{} and {} others", names[..2].join(", "), n - 2)),
    }
    let place = if !event.location.is_empty() {
        event.location.clone()
    } else if event.conference_url.contains("meet.google.com") {
        "Google Meet".into()
    } else if event.conference_url.contains("zoom.us") {
        "Zoom".into()
    } else if event.conference_url.contains("teams.microsoft.com") {
        "Microsoft Teams".into()
    } else {
        String::new()
    };
    if !place.is_empty() {
        parts.push(place);
    }
    parts.join(" \u{b7} ")
}

fn event_rows(
    events: &[Event],
    now: Now,
    day_start: i64,
    meetings: &HashMap<String, MeetingLink>,
) -> (Vec<AgendaRow>, Vec<String>) {
    let day_end = day_start + DAY_MS;
    let mut rows = Vec::new();
    let mut all_day = Vec::new();
    for event in events {
        if event.declined || event.end <= day_start || event.start >= day_end {
            continue;
        }
        if event.all_day {
            all_day.push(event.title.clone());
            continue;
        }
        let key = format!("{}@{}", event.uid, event.start);
        rows.push(AgendaRow {
            kind: AgendaKind::Event,
            key: key.clone(),
            title: event.title.clone(),
            context: event_context(event, day_start),
            at_min: Some(minutes_into_day(event.start, day_start)),
            end_min: Some(minutes_into_day(event.end, day_start)),
            days_late: 0,
            now: event.start <= now.utc_ms && now.utc_ms < event.end,
            free: event.free,
            note_id: String::new(),
            line: 0,
            is_loop: false,
            due: String::new(),
            meeting: meetings.get(&key).cloned(),
            color: event.color.clone(),
            event: Some((event.uid.clone(), event.start)),
        });
    }
    (rows, all_day)
}

fn day_label(date: Date) -> String {
    format(&DateTime::at_midnight(date), "%A %e %B").to_uppercase()
}

/// A person's day: the weekday and date, the all-day events after a dot.
fn overline(date: Date, all_day: &[String]) -> String {
    let mut text = day_label(date);
    for title in all_day {
        text.push_str(" \u{b7} ");
        text.push_str(title);
    }
    text
}

/// The first sentence a loop holds and its notebook-free context: whether it
/// names something (a date, an @ mention or a name).
fn names_something(text: &str, due: &Option<String>) -> bool {
    if due.is_some() || text.contains('@') {
        return true;
    }
    text.split_whitespace()
        .skip(1)
        .any(|w| w.chars().next().is_some_and(char::is_uppercase) && w.chars().count() > 2)
}

fn cut(text: &str, chars: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= chars {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(chars).collect();
    out.push('\u{2026}');
    out
}

pub(super) fn book_name(store: &Store, cache: &mut HashMap<String, String>, id: &str) -> Result<String> {
    if let Some(name) = cache.get(id) {
        return Ok(name.clone());
    }
    let name = store
        .get(id)?
        .and_then(|d| {
            d.fields
                .get("name")
                .and_then(|n| n.as_str().map(str::to_owned))
        })
        .unwrap_or_default();
    cache.insert(id.to_owned(), name.clone());
    Ok(name)
}

fn pick_up_list(
    store: &Store,
    now: Now,
    opts: &Options,
    names: &mut HashMap<String, String>,
) -> Result<List> {
    let exclude: Vec<String> = opts.open_note.iter().cloned().collect();
    let places = store.pick_up_places(now.utc_ms, &exclude, density::LIST_CAP + 8)?;
    let mut rows = Vec::new();
    for place in places {
        let found = writing::find(&place.body, place.line, place.col, &place.anchor_hash);
        let (detail, caret) = match found {
            Some(found) => (
                writing::excerpt(&found, writing::EXCERPT_CHARS),
                found.caret.is_some(),
            ),
            None => (String::new(), false),
        };
        rows.push(ListRow {
            kind: RowKind::PickUp,
            note_id: place.note_id,
            line: place.line,
            title: if place.title.is_empty() {
                "Untitled".into()
            } else {
                place.title
            },
            detail,
            place: book_name(store, names, &place.book_id)?,
            at: place.session_end,
            quiet_days: 0,
            key: String::new(),
            person: String::new(),
            caret,
            count: 0,
        });
    }
    let (rows, more) = density::cap_rows(rows, usize::MAX);
    Ok(List {
        kind: ListKind::PickUp,
        rows,
        more,
    })
}

pub(super) fn waiting_row(row: WaitingRow, now: i64) -> ListRow {
    let text = display_text(&row.text);
    ListRow {
        kind: if row.kind == "loop" {
            RowKind::WaitingLoop
        } else {
            RowKind::WaitingTask
        },
        note_id: row.note_id,
        line: row.line,
        title: row.person,
        detail: cut(&text, 90),
        place: row.note_title,
        at: row.quiet_since,
        quiet_days: ((now - row.quiet_since).max(0) / DAY_MS) as u32,
        key: row.key,
        person: row.person_id,
        caret: false,
        count: 0,
    }
}

/// Words for what an agent or a person did, for a row's detail.
fn activity_rows(store: &Store, opts: &Options) -> Result<Vec<ListRow>> {
    let mut rows: Vec<ListRow> = Vec::new();
    for item in &opts.agent {
        let what = match item.kind {
            RowKind::AgentTask => format!("{} has changes to review in \u{201c}{}\u{201d}", item.who, item.title),
            _ => format!(
                "{} suggests {} {} in \u{201c}{}\u{201d}",
                item.who,
                item.count,
                if item.count == 1 { "edit" } else { "edits" },
                item.title
            ),
        };
        rows.push(ListRow {
            kind: item.kind,
            note_id: item.note_id.clone(),
            line: 0,
            title: what,
            detail: String::new(),
            place: item.who.clone(),
            at: item.at,
            quiet_days: 0,
            key: String::new(),
            person: String::new(),
            caret: false,
            count: item.count,
        });
    }
    // Edits by people you share with, since the note was last opened.
    for shared in store.shared_notes()? {
        if shared.role == "owner" && shared.owner_name.is_empty() {
            continue;
        }
        let authors = store.shared_authors(&shared.note_id)?;
        if authors.is_empty() {
            continue;
        }
        let Some(doc) = store.get(&shared.note_id)? else {
            continue;
        };
        let updated = doc
            .fields
            .get("updatedAt")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let seen = store
            .meta(&format!("collab.seen.{}", shared.note_id))?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        if updated <= seen {
            continue;
        }
        let title = doc
            .fields
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Untitled");
        rows.push(ListRow {
            kind: RowKind::SharedEdit,
            note_id: shared.note_id.clone(),
            line: 0,
            title: format!("{} edited \u{201c}{title}\u{201d}", authors.join(", ")),
            detail: String::new(),
            place: authors.first().cloned().unwrap_or_default(),
            at: updated,
            quiet_days: 0,
            key: String::new(),
            person: String::new(),
            caret: false,
            count: authors.len() as u32,
        });
    }
    for notice in store.notices()?.into_iter().filter(|n| !n.read) {
        rows.push(ListRow {
            kind: RowKind::Mention,
            note_id: String::new(),
            line: 0,
            title: format!(
                "{} mentioned you in \u{201c}{}\u{201d}",
                notice.from_name, notice.title
            ),
            detail: cut(&notice.excerpt, 90),
            place: notice.from_name,
            at: notice.at,
            quiet_days: 0,
            key: notice.share_id,
            person: String::new(),
            caret: false,
            count: 1,
        });
    }
    // Anything waiting for approval first, then the newest.
    rows.sort_by(|a, b| {
        let waiting = |r: &ListRow| !matches!(r.kind, RowKind::AgentSuggestion | RowKind::AgentTask);
        waiting(a).cmp(&waiting(b)).then(b.at.cmp(&a.at))
    });
    Ok(rows)
}

fn first_line(text: &str, chars: usize) -> String {
    let line = text
        .lines()
        .map(|l| l.trim().trim_start_matches('#').trim())
        .find(|l| !l.is_empty())
        .unwrap_or("");
    cut(line, chars)
}

fn revisit_line(store: &Store, now: Now, today: Date) -> Result<Option<AlsoLine>> {
    let due = store.revisits_due(&today.iso(), REVISIT_SCAN)?;
    let cards = store
        .due_cards(&CardScope::All, now.utc_ms, now.start_of(today))?
        .len() as u32;
    if due.is_empty() && cards == 0 {
        return Ok(None);
    }
    Ok(Some(AlsoLine {
        kind: AlsoKind::Revisit,
        count: due.len() as u32,
        cards,
        items: due
            .iter()
            .take(2)
            .map(|r| Item {
                note_id: r.note_id.clone(),
                line: 0,
                text: if r.title.is_empty() {
                    "Untitled".into()
                } else {
                    r.title.clone()
                },
                voice_secs: 0,
            })
            .collect(),
        detail: due
            .iter()
            .map(|r| {
                let set = Date::from_days(
                    (r.set_at + now.offset_ms).div_euclid(DAY_MS),
                );
                let mut text = format!("Set on {}", format(&DateTime::at_midnight(set), "%e %b"));
                if !r.why.is_empty() {
                    text.push_str(&format!(" \u{b7} \u{201c}{}\u{201d}", r.why));
                }
                text
            })
            .collect(),
    }))
}

fn captured_line(store: &Store) -> Result<Option<AlsoLine>> {
    let waiting = store.inbox_waiting(Role::Inbox.book_id())?;
    if waiting.is_empty() {
        return Ok(None);
    }
    let mut items = Vec::new();
    for note in waiting.iter().take(2) {
        let Some(doc) = store.get(&note.id)? else {
            continue;
        };
        let body = doc
            .fields
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let title = doc
            .fields
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let text = if body.trim().is_empty() { title } else { body };
        let voice = read_duration_ms(&doc.fields).map_or(0, |ms| (ms / 1000) as u32);
        items.push(Item {
            note_id: note.id.clone(),
            line: 0,
            text: first_line(text, 40),
            voice_secs: voice,
        });
    }
    Ok(Some(AlsoLine {
        kind: AlsoKind::Captured,
        count: waiting.len() as u32,
        cards: 0,
        items,
        detail: Vec::new(),
    }))
}

/// The promises Overview is sure of, and the families it stopped guessing.
fn promises(
    store: &Store,
    now: Now,
    suppressed: &[loop_score::Family],
) -> Result<(Option<AlsoLine>, Vec<kanso_store::LoopRow>)> {
    let rows = store.open_loops(&LoopQuery {
        limit: Some(LOOP_SCAN),
        ..LoopQuery::default()
    })?;
    let mut confident: Vec<(f32, &kanso_store::LoopRow)> = Vec::new();
    for row in &rows {
        if row.kind == LoopKind::Waiting {
            continue;
        }
        let family = loop_score::family(&row.text, row.kind);
        let ctx = Context {
            names_something: names_something(&row.text, &row.due),
            note_recent: now.utc_ms - row.note_updated_at <= 14 * DAY_MS,
            quoted: row.text.trim_start().starts_with('>'),
            suppressed: suppressed.contains(&family),
        };
        let score = loop_score::score(&row.text, row.kind, ctx);
        if loop_score::is_confident(score) {
            confident.push((score, row));
        }
    }
    confident.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.1.note_updated_at.cmp(&a.1.note_updated_at))
    });
    let line = (!confident.is_empty()).then(|| AlsoLine {
        kind: AlsoKind::Promises,
        count: confident.len() as u32,
        cards: 0,
        items: confident
            .iter()
            .take(1)
            .map(|(_, r)| Item {
                note_id: r.note_id.clone(),
                line: r.line,
                text: cut(&r.text, 60),
                voice_secs: 0,
            })
            .collect(),
        detail: Vec::new(),
    });
    Ok((line, rows))
}

/// Today at `now`.
pub fn today(store: &Store, now: Now, opts: &Options) -> Result<Overview> {
    let today = now.today();
    let day_start = now.start_of(today);
    let local = now.local();
    let now_min = local.hour * 60 + local.minute;
    let mut names: HashMap<String, String> = HashMap::new();

    // The agenda.
    let tasks = store.tasks(&TaskQuery {
        due_before: Some(today.add_days(1).iso()),
        ..TaskQuery::default()
    })?;
    let mut rows: Vec<AgendaRow> = tasks.iter().filter_map(|t| task_row(t, today)).collect();
    let suppressed = store.suppressed_loop_families(now.utc_ms)?;
    let (promise_line, loop_rows) = if opts.loops {
        promises(store, now, &suppressed)?
    } else {
        (None, Vec::new())
    };
    // A promise whose date has passed is overdue, marked as promised.
    let task_keys: HashSet<String> = rows.iter().map(|r| r.key.clone()).collect();
    for row in &loop_rows {
        let Some(due) = row.due.as_deref().and_then(date_of_due) else {
            continue;
        };
        let late = today.to_days() - due.to_days();
        if late <= 0 || row.kind == LoopKind::Waiting || task_keys.contains(&row.key) {
            continue;
        }
        rows.push(AgendaRow {
            kind: AgendaKind::Overdue,
            key: row.key.clone(),
            title: display_text(&row.text),
            context: format!(
                "promised {} \u{b7} {}",
                format(&DateTime::at_midnight(due), "%e %b"),
                row.note_title
            ),
            at_min: None,
            end_min: None,
            days_late: late as u32,
            now: false,
            free: false,
            note_id: row.note_id.clone(),
            line: row.line,
            is_loop: true,
            due: row.due.clone().unwrap_or_default(),
            meeting: None,
            color: String::new(),
            event: None,
        });
    }
    let (event_rows, all_day) = event_rows(&opts.events, now, day_start, &opts.meetings);
    let meetings = event_rows.iter().filter(|r| !r.free).count() as u32;
    rows.extend(event_rows);

    // The lede reads the rows before they are folded and capped.
    let task_rows: Vec<&AgendaRow> = rows.iter().filter(|r| r.is_task()).collect();
    let late_rows: Vec<&&AgendaRow> = task_rows
        .iter()
        .filter(|r| r.kind == AgendaKind::Overdue)
        .collect();
    let next = next_up(&rows, now_min);

    let agenda = agenda::build(rows.clone(), now_min);

    // The Lists.
    let pick_up = pick_up_list(store, now, opts, &mut names)?;
    let waiting = List {
        kind: ListKind::Waiting,
        rows: store
            .waiting_items(now.utc_ms, opts.quiet_days.clamp(3, 14), 12)?
            .into_iter()
            .map(|r| waiting_row(r, now.utc_ms))
            .collect(),
        more: 0,
    };
    let activity = List {
        kind: ListKind::Activity,
        rows: activity_rows(store, opts)?,
        more: 0,
    };
    let (pick_has, wait_has) = (!pick_up.rows.is_empty(), !waiting.rows.is_empty());
    let (lists, folded) = density::lay_out(vec![pick_up, waiting, activity]);

    // Also.
    let folded_line = folded.as_ref().map(density::fold_line);
    let mut lines = Vec::new();
    lines.extend(revisit_line(store, now, today)?);
    lines.extend(captured_line(store)?);
    lines.extend(promise_line);
    let also = density::also_block(folded_line, lines);
    debug_assert!(also.len() <= ALSO_CAP);

    // The coda.
    let seeds = past_seeds(&lists, &agenda);
    let past = past::past_note(store, now, opts.model.as_deref(), &seeds)?;

    // The footer.
    let from = now.start_of(today.add_days(-1));
    let to = now.start_of(today);
    let wrote = store
        .notes_changed_between(from, to)?
        .iter()
        .filter(|n| n.created_at >= from && n.created_at < to)
        .count() as u32;
    let finished = store.finished_between(from, to)?.len() as u32;
    let footer = if wrote == 0 && finished == 0 {
        String::new()
    } else {
        footer_text(wrote, finished)
    };

    let facts = TodayFacts {
        meetings,
        tasks: task_rows.len() as u32,
        late: late_rows.len() as u32,
        oldest_late_days: late_rows.iter().map(|r| r.days_late).max().unwrap_or(0),
        next,
        pick_up: pick_has,
    };
    let _ = wait_has;
    let lede_text = lede::today_lede(&facts);

    let notes = store
        .counts()?
        .get(&DocType::Note)
        .copied()
        .unwrap_or_default();
    let busy = !agenda.is_empty()
        || !lists.is_empty()
        || !also.is_empty()
        || past.is_some();
    let state = if notes == 0 {
        State::New
    } else if busy {
        State::Busy
    } else {
        State::Quiet
    };
    let quiet = !busy;
    let state = if quiet && state != State::New {
        State::Quiet
    } else {
        state
    };

    Ok(Overview {
        version: VERSION,
        at: now.utc_ms,
        day: today.iso(),
        today: Today {
            greeting: lede::greeting(local.hour).to_owned(),
            overline: overline(today, &all_day),
            all_day,
            lede: lede_text,
            state,
            agenda,
            lists,
            also,
            past,
            footer,
            suppressed: suppressed.iter().map(|f| f.as_str().to_owned()).collect(),
        },
    })
}

fn footer_text(wrote: u32, finished: u32) -> String {
    let mut parts = Vec::new();
    if wrote > 0 {
        parts.push(format!(
            "wrote {wrote} {}",
            if wrote == 1 { "note" } else { "notes" }
        ));
    }
    if finished > 0 {
        parts.push(format!(
            "finished {finished} {}",
            if finished == 1 { "task" } else { "tasks" }
        ));
    }
    format!("Yesterday you {}.", parts.join(" and "))
}

fn next_up(rows: &[AgendaRow], now_min: u32) -> Option<Next> {
    if let Some(row) = rows
        .iter()
        .filter(|r| r.kind == AgendaKind::Event && r.now)
        .min_by_key(|r| r.end_min)
    {
        return Some(Next::Now {
            title: row.title.clone(),
            until_min: row.end_min.unwrap_or(now_min),
        });
    }
    let row = rows
        .iter()
        .filter(|r| matches!(r.kind, AgendaKind::Event | AgendaKind::Reminder))
        .filter(|r| r.at_min.is_some_and(|a| a >= now_min))
        .min_by_key(|r| (r.at_min, r.kind != AgendaKind::Event))?;
    let at = row.at_min?;
    Some(if at - now_min <= 60 {
        Next::In {
            title: row.title.clone(),
            minutes: at - now_min,
        }
    } else {
        Next::At {
            title: row.title.clone(),
            at_min: at,
        }
    })
}

/// What the note from the past is chosen for: the notes in *Pick up*, the
/// notes holding today's tasks, and today's linked events' notes.
fn past_seeds(lists: &[List], agenda: &Agenda) -> Vec<past::Seed> {
    let mut seeds: Vec<past::Seed> = Vec::new();
    for event in agenda
        .rows
        .iter()
        .chain(agenda.earlier.iter())
        .filter(|r| r.kind == AgendaKind::Event)
    {
        if let Some(meeting) = &event.meeting {
            seeds.push(past::Seed {
                note_id: meeting.note_id.clone(),
                why: format!(
                    "Related to your {} {}",
                    clock(event.at_min.unwrap_or(0)),
                    event.title
                ),
            });
        }
    }
    for list in lists.iter().filter(|l| l.kind == ListKind::PickUp) {
        for row in &list.rows {
            seeds.push(past::Seed {
                note_id: row.note_id.clone(),
                why: format!("Related to what you were writing in \u{201c}{}\u{201d}", row.title),
            });
        }
    }
    for row in agenda.rows.iter().filter(|r| r.is_task() && !r.note_id.is_empty()) {
        seeds.push(past::Seed {
            note_id: row.note_id.clone(),
            why: format!("Related to your task \u{201c}{}\u{201d}", cut(&row.title, 40)),
        });
    }
    let mut seen = HashSet::new();
    seeds.retain(|s| seen.insert(s.note_id.clone()));
    seeds.truncate(5);
    seeds
}
