//! Overview on the phone (issue #240): Today and This week.
//!
//! The rules are the desktop's (`kanso_engine::overview`): the agenda's
//! order, the caps, the lede, Pick up, Waiting on others, the note from the
//! past. This file reads the library, says the page's words, and keeps the
//! person's small decisions (Not now, Not yet, Revisit in…) in the store.
//!
//! The first frame is instant: [`Library::overview_snapshot`] returns the
//! last Today kept in the library's local meta, and [`Library::overview_today`]
//! works the page out again. Locked notes and notes in locked notebooks never
//! appear, and `#ai-exclude` notes never reach a model.

use chrono::{Local, Offset as _, TimeZone as _};
use kanso_core::dates::{Date, DateTime, format};
use kanso_core::now_ms;
use kanso_core::revisit::{Choice, Revisit};
use kanso_engine::brief::{self, BriefSettings, Now};
use kanso_engine::loops_ai;
use kanso_engine::overview::{
    self, AgendaKind, AgendaRow, AlsoKind, AlsoLine, List, ListKind, ListRow, Options, PastNote,
    RowKind, State,
};

use crate::ai::{AiHandle, AiRun, ai_error};
use crate::boundary::guard;
use crate::brief_model::AiSummary;
use crate::errors::KansoError;
use crate::library::Library;
use crate::overview_model::{
    OverviewAgendaRow, OverviewAheadDay, OverviewAi, OverviewFinished, OverviewItem, OverviewLine,
    OverviewLineKind, OverviewList, OverviewListKind, OverviewListRow, OverviewPast,
    OverviewProject, OverviewRowKind, OverviewState, OverviewToday, OverviewWeek, RevisitChoice,
    RevisitOption,
};

/// Where the model's paragraph is kept, per day or week.
const AI_TODAY: &str = "overview.ai.today.";
const AI_WEEK: &str = "overview.ai.week.";
const CALENDAR_EVENTS: &str = "overview.calendar.events";

/// The kept paragraph: the same shape the desktop keeps.
#[derive(serde::Serialize, serde::Deserialize)]
struct KeptAi {
    text: String,
    #[serde(default)]
    pairs: Vec<(String, String)>,
    at: i64,
    #[serde(default)]
    fingerprint: String,
    #[serde(default)]
    warning: Option<String>,
}

impl Library {
    /// Replaces the local, read-only events supplied by the system calendar
    /// plugin. The value is local metadata and is never added to sync.
    pub fn overview_put_calendar_events(&self, events_json: &str) -> Result<(), KansoError> {
        guard(|| {
            let events = calendar_events(events_json)?;
            let value = serde_json::to_string(&events)
                .map_err(|_| KansoError::invalid("Those calendar events are invalid."))?;
            self.inner.store.set_meta(CALENDAR_EVENTS, &value)?;
            Ok(())
        })
    }

    /// Creates a Journal note linked to the single event in `event_json`.
    pub fn overview_create_meeting_note(
        &self,
        event_json: &str,
        context: &str,
    ) -> Result<String, KansoError> {
        guard(|| {
            let events = calendar_events(event_json)?;
            let [event] = events.as_slice() else {
                return Err(KansoError::invalid("Choose one calendar event."));
            };
            overview::create_meeting_note(&self.inner.store, event, now_here().offset_ms, context)
                .map_err(Into::into)
        })
    }

    /// Links an existing note to a system-calendar event occurrence.
    pub fn overview_link_meeting_note(
        &self,
        note_id: &str,
        event_json: &str,
    ) -> Result<(), KansoError> {
        guard(|| {
            let events = calendar_events(event_json)?;
            let [event] = events.as_slice() else {
                return Err(KansoError::invalid("Choose one calendar event."));
            };
            overview::link_meeting_note(&self.inner.store, note_id, event, now_here().offset_ms)
                .map_err(Into::into)
        })
    }

    /// Today, worked out at the phone's time now. Kept as the next launch's
    /// first frame.
    pub fn overview_today(&self) -> Result<OverviewToday, KansoError> {
        guard(|| {
            let now = now_here();
            let page = overview::today(&self.inner.store, now, &self.overview_options())?;
            overview::save_snapshot(&self.inner.store, &page).ok();
            Ok(self.today_view(&page, now))
        })
    }

    /// The last Today kept, when it is today's: the first frame, drawn
    /// before anything is worked out. Nothing waits on a slow disk.
    pub fn overview_snapshot(&self) -> Option<OverviewToday> {
        let now = now_here();
        let page = overview::load_snapshot(&self.inner.store, &now.today().iso())?;
        Some(self.today_view(&page, now))
    }

    /// The week of `from` (`YYYY-MM-DD`, moved back to its Monday), or this
    /// week.
    pub fn overview_week(&self, from: Option<String>) -> Result<OverviewWeek, KansoError> {
        guard(|| {
            let now = now_here();
            let this_week = brief::week_start(now.today());
            let start = week_of(from.as_deref(), now)?;
            let week = overview::week(&self.inner.store, start, now, &self.overview_options())?;
            Ok(OverviewWeek {
                from: week.from.clone(),
                range_label: week.range_label.clone(),
                week_label: week.week_label.clone(),
                is_this_week: start == this_week,
                lede: week.lede.clone(),
                projects: week
                    .projects
                    .iter()
                    .map(|p| OverviewProject {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        notes_changed: p.notes_changed,
                        last_title: p.last_title.clone(),
                        last_note_id: p.last_note.clone(),
                        next_task: p.next_task.clone(),
                        next_due: p.next_due.clone(),
                    })
                    .collect(),
                finished: week
                    .finished
                    .iter()
                    .map(|f| OverviewFinished {
                        note_id: f.note_id.clone(),
                        text: f.text.clone(),
                    })
                    .collect(),
                finished_more: week.finished_more,
                waiting: week.waiting.iter().map(|r| list_row(r, now)).collect(),
                still_open: week.still_open.iter().map(agenda_row).collect(),
                still_open_total: week.still_open_total,
                ahead: week
                    .ahead
                    .iter()
                    .map(|d| OverviewAheadDay {
                        summary: d.summary.clone(),
                        rows: d.rows.iter().map(agenda_row).collect(),
                    })
                    .collect(),
                also: week.also.iter().map(line).collect(),
                saved_as: week.saved_as.clone(),
                ai: self.kept_ai(&format!("{AI_WEEK}{}", brief::iso_week_label(start))),
            })
        })
    }

    /// *Not now* on Pick up: the note is hidden until it is written in again.
    pub fn overview_hide_place(&self, note_id: &str) -> Result<(), KansoError> {
        guard(|| Ok(self.inner.store.hide_writing_place(note_id, now_ms())?))
    }

    /// *Not yet* on Waiting on others: asked again in three days.
    pub fn overview_snooze_waiting(&self, key: &str) -> Result<(), KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .snooze_waiting(key, now_ms() + 3 * 86_400_000)?)
        })
    }

    /// *Not useful* on the note from the past: it goes, and no other comes
    /// today.
    pub fn overview_dismiss_past(&self, note_id: &str) -> Result<(), KansoError> {
        guard(|| {
            let now = now_here();
            let Some(page) = overview::load_snapshot(&self.inner.store, &now.today().iso()) else {
                return Ok(());
            };
            if let Some(past) = page.today.past.filter(|p| p.note_id == note_id) {
                overview::dismiss_past(&self.inner.store, now, &past)?;
            }
            Ok(())
        })
    }

    /// The choices of *Revisit in…* with the day each means today.
    pub fn revisit_options(&self) -> Vec<RevisitOption> {
        let today = now_here().today();
        Choice::ALL
            .iter()
            .map(|choice| RevisitOption {
                choice: choice_of(*choice),
                label: choice.label().to_owned(),
                day_label: format(&DateTime::at_midnight(choice.date(today)), "%a %e %b"),
            })
            .collect()
    }

    /// Sets a note's revisit date. Setting it is not an edit: the note's
    /// modified time stays. Returns the day, `YYYY-MM-DD`.
    pub fn set_revisit(&self, note_id: &str, choice: RevisitChoice) -> Result<String, KansoError> {
        guard(|| {
            let now = now_here();
            let date = revisit_choice(choice).date(now.today());
            let revisit = Revisit::new(date, now.utc_ms, "");
            self.inner.store.set_revisit(note_id, Some(&revisit))?;
            Ok(date.iso())
        })
    }

    /// A note's revisit date (`YYYY-MM-DD`), if it has one.
    pub fn revisit_of(&self, note_id: &str) -> Result<Option<String>, KansoError> {
        guard(|| Ok(self.inner.store.revisit_of(note_id)?.map(|r| r.on)))
    }

    /// Takes a note off *To revisit*.
    pub fn clear_revisit(&self, note_id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.inner.store.set_revisit(note_id, None)?;
            Ok(())
        })
    }

    /// Typing in a note, for Pick up: where the caret is (`line`, `column`),
    /// the characters typed since the last call and the hash of the
    /// paragraph's first words. The editor calls this at most every ten
    /// seconds.
    pub fn note_writing(
        &self,
        note_id: &str,
        line: u32,
        column: u32,
        typed_chars: u32,
        body: &str,
    ) -> Result<(), KansoError> {
        guard(|| {
            let anchor = kanso_core::writing::anchor_at(body, line);
            Ok(self.inner.store.note_writing(
                note_id,
                line,
                column,
                typed_chars,
                &anchor,
                now_ms(),
            )?)
        })
    }

    fn overview_options(&self) -> Options {
        let settings = self.overview_settings();
        let mut opts = Options::new();
        opts.quiet_days = settings.quiet_days;
        opts.loops = settings.loops;
        opts.model = self.inner.store.embedding_model().ok().flatten();
        opts.events = self
            .inner
            .store
            .meta(CALENDAR_EVENTS)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        opts
    }

    fn overview_settings(&self) -> BriefSettings {
        self.inner
            .store
            .meta("brief.settings")
            .ok()
            .flatten()
            .and_then(|text| serde_json::from_str::<BriefSettings>(&text).ok())
            .unwrap_or_default()
            .sanitized()
    }

    fn kept_ai(&self, key: &str) -> Option<OverviewAi> {
        let kept: KeptAi = serde_json::from_str(&self.inner.store.meta(key).ok()??).ok()?;
        let local = Local
            .timestamp_millis_opt(kept.at)
            .single()
            .map(|t| {
                format!(
                    "{:02}:{:02}",
                    chrono::Timelike::hour(&t),
                    chrono::Timelike::minute(&t)
                )
            })
            .unwrap_or_default();
        Some(OverviewAi {
            text: overview::summary::link_citations(&kept.text, &kept.pairs),
            warning: kept.warning,
            time_label: local,
        })
    }

    fn today_view(&self, page: &overview::Overview, now: Now) -> OverviewToday {
        let t = &page.today;
        OverviewToday {
            day: page.day.clone(),
            greeting: t.greeting.clone(),
            overline: t.overline.clone(),
            lede: t.lede.clone(),
            state: match t.state {
                State::New => OverviewState::New,
                State::Quiet => OverviewState::Quiet,
                State::Busy => OverviewState::Busy,
            },
            earlier: t.agenda.earlier.iter().map(agenda_row).collect(),
            agenda: t.agenda.rows.iter().map(agenda_row).collect(),
            agenda_more: t.agenda.more,
            task_total: t.agenda.task_total,
            lists: t.lists.iter().map(|l| list(l, now)).collect(),
            also: t.also.iter().map(line).collect(),
            past: t.past.as_ref().map(past),
            footer: t.footer.clone(),
            ai: self.kept_ai(&format!("{AI_TODAY}{}", page.day)),
        }
    }
}

fn calendar_events(json: &str) -> Result<Vec<overview::Event>, KansoError> {
    const MAX_EVENTS: usize = 20_000;
    const MAX_TEXT_CHARS: usize = 2_000;
    let events: Vec<overview::Event> = serde_json::from_str(json)
        .map_err(|_| KansoError::invalid("Those calendar events are invalid."))?;
    if events.len() > MAX_EVENTS
        || events.iter().any(|event| {
            event.uid.is_empty()
                || event.title.chars().count() > MAX_TEXT_CHARS
                || event.location.chars().count() > MAX_TEXT_CHARS
                || event.calendar.chars().count() > MAX_TEXT_CHARS
                || event.start >= event.end
                || event.attendees.len() > 500
        })
    {
        return Err(KansoError::invalid("Those calendar events are invalid."));
    }
    Ok(events)
}

fn week_of(from: Option<&str>, now: Now) -> Result<Date, KansoError> {
    Ok(match from {
        Some(text) => brief::week_start(
            Date::parse_iso(text).ok_or_else(|| KansoError::invalid("That isn't a date."))?,
        ),
        None => brief::week_start(now.today()),
    })
}

fn choice_of(choice: Choice) -> RevisitChoice {
    match choice {
        Choice::Tomorrow => RevisitChoice::Tomorrow,
        Choice::InThreeDays => RevisitChoice::InThreeDays,
        Choice::NextWeek => RevisitChoice::NextWeek,
        Choice::InTwoWeeks => RevisitChoice::InTwoWeeks,
        Choice::InAMonth => RevisitChoice::InAMonth,
        Choice::InThreeMonths => RevisitChoice::InThreeMonths,
    }
}

fn revisit_choice(choice: RevisitChoice) -> Choice {
    match choice {
        RevisitChoice::Tomorrow => Choice::Tomorrow,
        RevisitChoice::InThreeDays => Choice::InThreeDays,
        RevisitChoice::NextWeek => Choice::NextWeek,
        RevisitChoice::InTwoWeeks => Choice::InTwoWeeks,
        RevisitChoice::InAMonth => Choice::InAMonth,
        RevisitChoice::InThreeMonths => Choice::InThreeMonths,
    }
}

/// The words of an agenda row's gutter.
fn gutter(row: &AgendaRow) -> String {
    match row.kind {
        AgendaKind::Overdue => overview::late_label(row.days_late),
        AgendaKind::Event if row.now => "Now".to_owned(),
        AgendaKind::Event | AgendaKind::Reminder => overview::clock(row.at_min.unwrap_or(0)),
        AgendaKind::Due => row
            .at_min
            .map_or_else(|| "Today".to_owned(), overview::clock),
    }
}

fn agenda_row(row: &AgendaRow) -> OverviewAgendaRow {
    OverviewAgendaRow {
        kind: match row.kind {
            AgendaKind::Overdue => OverviewRowKind::Overdue,
            AgendaKind::Event => OverviewRowKind::Event,
            AgendaKind::Reminder => OverviewRowKind::Reminder,
            AgendaKind::Due => OverviewRowKind::Due,
        },
        key: row.key.clone(),
        title: row.title.clone(),
        context: row.context.clone(),
        gutter: gutter(row),
        now: row.now,
        free: row.free,
        note_id: row.note_id.clone(),
        line: row.line,
        is_loop: row.is_loop,
        raw: row.raw.clone(),
        color: row.color.clone(),
        meeting_note_id: row
            .meeting
            .as_ref()
            .map(|m| m.note_id.clone())
            .unwrap_or_default(),
        meeting_which: row
            .meeting
            .as_ref()
            .map(|m| m.which.clone())
            .unwrap_or_default(),
        meeting_day: row
            .meeting
            .as_ref()
            .map(|m| m.day.clone())
            .unwrap_or_default(),
        event_json: row
            .event_details
            .as_ref()
            .and_then(|event| serde_json::to_string(event).ok())
            .unwrap_or_default(),
    }
}

fn list_kind(kind: ListKind) -> OverviewListKind {
    match kind {
        ListKind::PickUp => OverviewListKind::PickUp,
        ListKind::Waiting => OverviewListKind::Waiting,
        ListKind::Activity => OverviewListKind::Activity,
    }
}

fn list_row(row: &ListRow, now: Now) -> OverviewListRow {
    OverviewListRow {
        kind: match row.kind {
            RowKind::PickUp => OverviewListKind::PickUp,
            RowKind::WaitingLoop | RowKind::WaitingTask => OverviewListKind::Waiting,
            _ => OverviewListKind::Activity,
        },
        note_id: row.note_id.clone(),
        line: row.line,
        title: row.title.clone(),
        detail: row.detail.clone(),
        place: row.place.clone(),
        when_label: overview::ago(row.at, now.utc_ms, now.offset_ms),
        quiet_days: row.quiet_days,
        warn: row.quiet_days >= 10,
        key: row.key.clone(),
        person: row.person.clone(),
        raw: row.raw.clone(),
        is_loop: row.kind == RowKind::WaitingLoop,
        caret: row.caret,
        count: row.count,
        agent: matches!(row.kind, RowKind::AgentSuggestion | RowKind::AgentTask),
    }
}

fn list(list: &List, now: Now) -> OverviewList {
    OverviewList {
        kind: list_kind(list.kind),
        label: list.kind.label().to_owned(),
        // Every row, the ones past the cap last: `more` of them are shown
        // only when the person asks for them.
        rows: list
            .rows
            .iter()
            .chain(list.extra.iter())
            .map(|r| list_row(r, now))
            .collect(),
        more: list.extra.len() as u32,
    }
}

fn line(line: &AlsoLine) -> OverviewLine {
    let (kind, label) = match line.kind {
        AlsoKind::Folded(ListKind::Activity) => (OverviewLineKind::Activity, "Activity"),
        AlsoKind::Folded(ListKind::Waiting) => (OverviewLineKind::Waiting, "Waiting"),
        AlsoKind::Folded(ListKind::PickUp) => (OverviewLineKind::PickUp, "Pick up"),
        AlsoKind::Revisit => (OverviewLineKind::Revisit, "To revisit"),
        AlsoKind::Captured => (OverviewLineKind::Captured, "Captured"),
        AlsoKind::Promises => (OverviewLineKind::Promises, "Promises"),
        AlsoKind::Themes => (OverviewLineKind::Themes, "Themes"),
        AlsoKind::MeetingsWithoutNotes => (OverviewLineKind::Meetings, "Meetings"),
    };
    OverviewLine {
        kind,
        label: label.to_owned(),
        count: line.count,
        cards: line.cards,
        items: line
            .items
            .iter()
            .map(|i| OverviewItem {
                note_id: i.note_id.clone(),
                line: i.line,
                text: i.text.clone(),
                voice_secs: i.voice_secs,
            })
            .collect(),
        detail: line.detail.clone(),
    }
}

fn past(p: &PastNote) -> OverviewPast {
    OverviewPast {
        note_id: p.note_id.clone(),
        title: p.title.clone(),
        excerpt: p.excerpt.clone(),
        why: p.why.clone(),
    }
}

/// The AI paragraph over Overview, run here on the calling thread (the
/// bridge calls it off the UI isolate), as the other summaries are. Locked
/// notes and notes tagged `#ai-exclude` are never sent.
impl AiHandle {
    /// Writes one short paragraph over Today (`from` `None`) or over the
    /// week of `from`, keeps it for the day or the week and returns it with
    /// its citations linked to the notes they name.
    pub fn summarise_overview(
        &self,
        from: Option<String>,
        week: bool,
        run: &AiRun,
    ) -> Result<AiSummary, KansoError> {
        guard(|| {
            let now = now_here();
            let store = &self.inner.store;
            let settings = self.enabled()?;
            let provider = self.provider_for(&settings)?;
            let opts = Options::new();
            let (prompt, pairs, key, what) = if week {
                let start = week_of(from.as_deref(), now)?;
                let page = overview::week(store, start, now, &opts)?;
                let (prompt, pairs) = loops_ai::overview_week_prompt(store, &page);
                (
                    prompt,
                    pairs,
                    format!("{AI_WEEK}{}", brief::iso_week_label(start)),
                    "page",
                )
            } else {
                let page = overview::today(store, now, &opts)?;
                let (prompt, pairs) = loops_ai::overview_today_prompt(store, &page.today, false);
                (prompt, pairs, format!("{AI_TODAY}{}", page.day), "page")
            };
            if run.stopped() {
                return Err(KansoError::invalid("Stopped."));
            }
            self.inner.metrics().count(crate::metrics::Feature::Ai);
            let provider = crate::boundary::held(&provider);
            let allowed: Vec<String> = pairs.iter().map(|(_, t)| t.clone()).collect();
            let summary = loops_ai::summarise(provider.as_ref(), &prompt, &allowed)
                .map_err(|err| ai_error(err, settings.api_key.as_deref()))?;
            let warning = summary.warning(what);
            let kept = KeptAi {
                text: summary.text.clone(),
                pairs: pairs.clone(),
                at: now.utc_ms,
                fingerprint: String::new(),
                warning: warning.clone(),
            };
            if let Ok(json) = serde_json::to_string(&kept) {
                store.set_meta(&key, &json).ok();
            }
            Ok(AiSummary {
                text: overview::summary::link_citations(&summary.text, &pairs),
                warning,
            })
        })
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
mod calendar_events_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_valid_system_events_round_trip(
            uid in "[a-zA-Z0-9-]{1,40}",
            start in -2_000_000_000_000i64..2_000_000_000_000,
            duration in 1i64..604_800_000,
            title in ".{0,100}",
            calendar in ".{1,100}",
            free in any::<bool>(),
            all_day in any::<bool>(),
        ) {
            let events = vec![overview::Event {
                uid,
                start,
                end: start.saturating_add(duration),
                title,
                calendar,
                free,
                all_day,
                ..overview::Event::default()
            }];
            let json = serde_json::to_string(&events).unwrap();
            prop_assert_eq!(calendar_events(&json).unwrap(), events);
        }
    }

    #[test]
    fn malformed_or_reversed_system_events_are_rejected() {
        assert!(calendar_events("not json").is_err());
        assert!(
            calendar_events(r#"[{"uid":"event","start":5,"end":5,"title":"Broken"}]"#,).is_err()
        );
    }
}
