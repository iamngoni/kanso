//! Overview, Kanso's home (issue #240): one page with **Today** and
//! **This week**, drawn from `kanso_engine::overview`.
//!
//! It replaces the Morning Brief and the Weekly Review. The page is a child
//! module of the digest view so it can use the view's state: the digest view
//! owns the reloading, the snapshot and the store writes, and this file draws
//! the masthead, the agenda, the Lists, *Also*, the coda and the week, and
//! turns clicks into those writes.
//!
//! The first frame draws from the snapshot, so nothing waits on the store;
//! sections are drawn only when they have something real, so a quiet day is
//! a short page.

use std::sync::{Arc, Mutex};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::{MarkdownExtensions, TextView};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::{Date, DateTime, format};
use kanso_core::revisit::{Choice, Revisit};
use kanso_engine::overview::{
    self, AgendaKind, AgendaRow, AlsoKind, AlsoLine, ListKind, ListRow, PastNote, RowKind, State,
};
use kanso_store::{CheckOff, LoopClose, Origin};

use super::{AiJob, DigestEvent, DigestView, Perspective, now_of, writes_held};
use crate::clock::Clock;
use crate::library::Filter;
use crate::page_chrome;
use crate::preview_rich::Rich;
use crate::theme::Colors;
use crate::zoom::px;

/// The reading column's width.
const COLUMN: f32 = 640.;
/// How far the time gutter hangs left of the column.
const HANG: f32 = 92.;
/// The pane width from which the gutter hangs in the margin.
const WIDE: f32 = 840.;
/// How long a tick can be undone.
pub(crate) const UNDO_FOR: std::time::Duration = std::time::Duration::from_secs(5);

/// The AI paragraph kept for a day or a week.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AiPara {
    pub text: String,
    /// `(note id, title)` the model was given: what its citations may link.
    pub pairs: Vec<(String, String)>,
    pub at: i64,
    /// What the page was when it was written: how many changes since.
    pub fingerprint: String,
    pub warning: Option<String>,
}

/// A tick that can still be undone.
#[derive(Clone, Debug)]
pub(crate) struct Undo {
    pub note_id: String,
    pub line: u32,
    pub raw: String,
    pub title: String,
}

/// What the Overview page needs besides the digest view's own state.
pub(crate) struct OverviewParts {
    pub rich: Arc<Rich>,
    /// Kept so the link sender stays open.
    pub _dead_links: async_channel::Receiver<String>,
    pub links: async_channel::Sender<String>,
}

impl OverviewParts {
    pub fn new(store: Arc<kanso_store::Store>) -> OverviewParts {
        let (wake, _woken) = async_channel::unbounded();
        let (edits, _edited) = async_channel::unbounded();
        let (links, dead_links) = async_channel::unbounded();
        OverviewParts {
            rich: Arc::new(Rich {
                wake,
                store,
                edits,
                note: Arc::new(Mutex::new(None)),
                text_scale: Mutex::new(1.),
                // An overview shows notes; it never edits them.
                sketch_read_only: std::sync::atomic::AtomicBool::new(true),
            }),
            _dead_links: dead_links,
            links,
        }
    }
}

type AlsoAction = (
    &'static str,
    fn(&mut DigestView, &mut Window, &mut Context<DigestView>),
);

fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn untitled(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".into()
    } else {
        title.to_owned()
    }
}

/// The gutter's words for an agenda row.
fn gutter(row: &AgendaRow) -> (String, bool, bool) {
    // (text, late, accent)
    match row.kind {
        AgendaKind::Overdue => (overview::late_label(row.days_late), true, false),
        AgendaKind::Event if row.now => ("Now".into(), false, true),
        AgendaKind::Event | AgendaKind::Reminder => {
            (overview::clock(row.at_min.unwrap_or(0)), false, false)
        }
        AgendaKind::Due => match row.at_min {
            Some(at) => (overview::clock(at), false, false),
            None => ("Today".into(), false, false),
        },
    }
}

/// A due date (`YYYY-MM-DD`, perhaps with a time) as words: "today",
/// "tomorrow", "2d late", or the day, "Fri 29 May".
fn due_words(due: &str, today: Date) -> String {
    let Some(date) = due.get(..10).and_then(Date::parse_iso) else {
        return due.to_owned();
    };
    let days = date.to_days() - today.to_days();
    let time = due.get(11..16).map(|t| format!(" {t}")).unwrap_or_default();
    match days {
        0 => format!("today{time}"),
        1 => format!("tomorrow{time}"),
        d if d < 0 => overview::late_label((-d) as u32),
        _ => format!("{}{time}", format(&DateTime::at_midnight(date), "%a %e %b")),
    }
}

fn also_label(line: &AlsoLine) -> &'static str {
    match line.kind {
        AlsoKind::Folded(ListKind::Activity) => "Activity",
        AlsoKind::Folded(ListKind::Waiting) => "Waiting",
        AlsoKind::Folded(ListKind::PickUp) => "Pick up",
        AlsoKind::Revisit => "To revisit",
        AlsoKind::Captured => "Captured",
        AlsoKind::Promises => "Promises",
        AlsoKind::Themes => "Themes",
        AlsoKind::MeetingsWithoutNotes => "Meetings",
    }
}

impl DigestView {
    // MARK: Data

    /// The page worked out again, off the main thread: Today always, the
    /// week too while it shows. The snapshot is kept for the next launch.
    pub(crate) fn reload_overview(&mut self, cx: &mut Context<Self>) {
        let store = self.store.clone();
        let now = now_of(cx);
        let week_start = self.week_start;
        let want_week = self.perspective == Perspective::Week;
        let mut opts = overview::Options::new();
        let briefs = cx
            .try_global::<crate::workspace::AppState>()
            .map(|state| state.config.briefs.clone().sanitized())
            .unwrap_or_default();
        opts.quiet_days = briefs.quiet_days;
        opts.loops = briefs.loops;
        // The editor's note is not left out: Overview hides the editor, and
        // at launch the note loaded quietly behind it is usually the very
        // one being written, which Pick up exists to bring back.
        opts.open_note = None;
        opts.model = store.embedding_model().ok().flatten();
        opts.agent = agent_items(cx);
        let task = cx.background_executor().spawn(async move {
            let today = overview::today(&store, now, &opts)?;
            overview::save_snapshot(&store, &today).ok();
            let week = if want_week {
                Some(overview::week(&store, week_start, now, &opts)?)
            } else {
                None
            };
            Ok::<_, kanso_store::StoreError>((today, week))
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((today, week)) => {
                        this.overview = Some(today);
                        if week.is_some() {
                            this.week = week;
                        }
                        this.loaded = true;
                        this.load_ai(cx);
                    }
                    Err(err) => cx.emit(DigestEvent::Notice(format!(
                        "Couldn't read your notes: {err}"
                    ))),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Draws the snapshot, when there is one for today, before the page is
    /// worked out.
    pub(crate) fn load_snapshot(&mut self, cx: &mut Context<Self>) {
        if self.overview.is_some() {
            return;
        }
        let today = now_of(cx).today().iso();
        self.overview = overview::load_snapshot(&self.store, &today);
        if self.overview.is_some() {
            self.loaded = true;
        }
    }

    fn ai_key(&self, cx: &App) -> String {
        match self.perspective {
            Perspective::Today => format!("overview.ai.today.{}", now_of(cx).today().iso()),
            Perspective::Week => format!(
                "overview.ai.week.{}",
                kanso_engine::brief::iso_week_label(self.week_start)
            ),
        }
    }

    /// Reads the paragraph kept for what shows.
    pub(crate) fn load_ai(&mut self, cx: &App) {
        let key = self.ai_key(cx);
        self.ai_para = self
            .store
            .meta(&key)
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str(&raw).ok());
    }

    /// What the page is now, to tell how much changed since the paragraph.
    fn fingerprint(&self) -> String {
        match (&self.overview, &self.week, self.perspective) {
            (Some(o), _, Perspective::Today) => format!(
                "{}:{}:{}:{}",
                o.today.agenda.rows.len() + o.today.agenda.earlier.len(),
                o.today.lists.iter().map(|l| l.rows.len()).sum::<usize>(),
                o.today.also.len(),
                o.today.past.as_ref().map_or("", |p| p.note_id.as_str())
            ),
            (_, Some(w), Perspective::Week) => {
                format!("{}:{}:{}", w.wrote, w.changed, w.finished_count)
            }
            _ => String::new(),
        }
    }

    #[cfg(test)]
    pub fn agenda_titles_for_test(&self) -> Vec<String> {
        self.overview
            .as_ref()
            .map(|o| {
                o.today
                    .agenda
                    .rows
                    .iter()
                    .map(|r| r.title.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Shows `overview` as if it had been worked out (snapshots of states
    /// the demo library can't reach, such as a new library).
    #[cfg(test)]
    pub fn set_overview_for_test(
        &mut self,
        overview: kanso_engine::overview::Overview,
        cx: &mut Context<Self>,
    ) {
        self.overview = Some(overview);
        self.loaded = true;
        cx.notify();
    }

    #[cfg(test)]
    pub fn tick_first_for_test(&mut self, cx: &mut Context<Self>) {
        if let Some(row) = self
            .overview
            .as_ref()
            .and_then(|o| o.today.agenda.rows.first().cloned())
        {
            self.tick_agenda(&row, cx);
        }
    }

    #[cfg(test)]
    pub fn undo_for_test(&mut self, cx: &mut Context<Self>) {
        self.undo_tick(cx);
    }

    #[cfg(test)]
    pub fn can_undo_for_test(&self) -> bool {
        self.undo.is_some()
    }

    // MARK: Actions

    pub(crate) fn show_perspective(&mut self, perspective: Perspective, cx: &mut Context<Self>) {
        if self.perspective != perspective {
            self.perspective = perspective;
            self.ai_more = false;
            self.ai_hidden = false;
            self.load_ai(cx);
            self.reload(cx);
        }
    }

    /// Ticks an agenda row: a task is checked in its note, a promise is
    /// closed. A task can be undone for a few seconds.
    fn tick_agenda(&mut self, row: &AgendaRow, cx: &mut Context<Self>) {
        if writes_held(cx) || row.note_id.is_empty() {
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        if row.is_loop {
            let mark = Self::mark_in_note(cx);
            match self.store.close_loop(
                &row.note_id,
                &row.key,
                row.line,
                LoopClose::Done { mark_in_note: mark },
                Clock::now_ms(cx),
                Origin::Ui,
            ) {
                Ok(_) => {}
                Err(err) => cx.emit(DigestEvent::Notice(format!(
                    "Couldn't change the loop: {err}"
                ))),
            }
        } else {
            match self.store.set_task_checked(
                &row.note_id,
                row.line,
                &row.raw,
                true,
                Clock::now_ms(cx),
                Origin::Ui,
            ) {
                Ok(CheckOff::Missing) => cx.emit(DigestEvent::Notice(
                    "That task changed in its note. The page is up to date now.".into(),
                )),
                Ok(_) => {
                    crate::metrics::record(cx, crate::metrics::Feature::TaskToggled);
                    self.set_undo(
                        Undo {
                            note_id: row.note_id.clone(),
                            line: row.line,
                            raw: row.raw.clone(),
                            title: row.title.clone(),
                        },
                        cx,
                    );
                }
                Err(err) => cx.emit(DigestEvent::Notice(format!(
                    "Couldn't change the task: {err}"
                ))),
            }
        }
        self.reload(cx);
    }

    fn set_undo(&mut self, undo: Undo, cx: &mut Context<Self>) {
        self.undo = Some(undo);
        self.undo_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(UNDO_FOR).await;
            this.update(cx, |this, cx| {
                this.undo = None;
                cx.notify();
            })
            .ok();
        }));
    }

    fn undo_tick(&mut self, cx: &mut Context<Self>) {
        let Some(undo) = self.undo.take() else {
            return;
        };
        self.undo_task = None;
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        if let Err(err) = self.store.set_task_checked(
            &undo.note_id,
            undo.line,
            &undo.raw,
            false,
            Clock::now_ms(cx),
            Origin::Ui,
        ) {
            cx.emit(DigestEvent::Notice(format!("Couldn't undo: {err}")));
        }
        self.reload(cx);
    }

    /// *Done* on a waiting row: closes the loop or ticks the task.
    fn done_waiting(&mut self, row: &ListRow, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let now = Clock::now_ms(cx);
        let result = if row.kind == RowKind::WaitingLoop {
            let mark = Self::mark_in_note(cx);
            self.store.close_loop(
                &row.note_id,
                &row.key,
                row.line,
                LoopClose::Done { mark_in_note: mark },
                now,
                Origin::Ui,
            )
        } else {
            self.store
                .set_task_checked(&row.note_id, row.line, &row.raw, true, now, Origin::Ui)
        };
        if let Err(err) = result {
            cx.emit(DigestEvent::Notice(format!("Couldn't change that: {err}")));
        }
        self.reload(cx);
    }

    /// *Not yet*: puts the item off for three days.
    fn snooze_waiting(&mut self, row: &ListRow, cx: &mut Context<Self>) {
        let until = Clock::now_ms(cx) + 3 * 86_400_000;
        if let Err(err) = self.store.snooze_waiting(&row.key, until) {
            cx.emit(DigestEvent::Notice(format!("Couldn't put it off: {err}")));
        }
        self.reload(cx);
    }

    /// *Not now* on Pick up: hides the note until it is written in again.
    fn hide_place(&mut self, row: &ListRow, cx: &mut Context<Self>) {
        if let Err(err) = self
            .store
            .hide_writing_place(&row.note_id, Clock::now_ms(cx))
        {
            cx.emit(DigestEvent::Notice(format!("Couldn't hide it: {err}")));
        }
        self.reload(cx);
    }

    /// Sets a revisit date on a note, without counting as an edit.
    pub(crate) fn revisit_note(&mut self, note_id: &str, choice: Choice, cx: &mut Context<Self>) {
        let today = now_of(cx).today();
        let revisit = Revisit::new(choice.date(today), Clock::now_ms(cx), "");
        match self.store.set_revisit(note_id, Some(&revisit)) {
            Ok(_) => cx.emit(DigestEvent::Notice(format!(
                "You'll see this again on {}.",
                format(&DateTime::at_midnight(choice.date(today)), "%A %e %B")
            ))),
            Err(err) => cx.emit(DigestEvent::Notice(format!("Couldn't set that: {err}"))),
        }
        self.reload(cx);
    }

    /// Clears a note's revisit date.
    pub(crate) fn clear_revisit(&mut self, note_id: &str, cx: &mut Context<Self>) {
        match self.store.set_revisit(note_id, None) {
            Ok(_) => cx.emit(DigestEvent::Notice("Revisit date cleared.".into())),
            Err(err) => cx.emit(DigestEvent::Notice(format!("Couldn't clear that: {err}"))),
        }
        self.reload(cx);
    }

    /// *Not useful*: the note from the past goes, and no other comes today.
    fn dismiss_past(&mut self, past: &PastNote, cx: &mut Context<Self>) {
        let now = now_of(cx);
        if let Err(err) = overview::dismiss_past(&self.store, now, past) {
            cx.emit(DigestEvent::Notice(format!("Couldn't hide it: {err}")));
        }
        self.reload(cx);
    }

    /// Opens a note from a row; ⌘ opens it in a new tab and leaves the page.
    fn open_row(&mut self, note_id: &str, line: u32, cx: &mut Context<Self>) {
        if note_id.is_empty() {
            return;
        }
        self.store
            .set_meta(
                &format!("collab.seen.{note_id}"),
                &Clock::now_ms(cx).to_string(),
            )
            .ok();
        self.open(note_id, line, false, cx);
    }

    pub(crate) fn summarise_overview(&mut self, cx: &mut Context<Self>) {
        if self.working.is_some() {
            return;
        }
        let with_events = false;
        // What the prompt is built from; it reads each note (to leave out
        // locked and #ai-exclude ones), so it is built off the main thread.
        enum Page {
            Today(Box<overview::Today>),
            Week(Box<overview::Week>),
        }
        let page = match (self.perspective, &self.overview, &self.week) {
            (Perspective::Today, Some(o), _) => Page::Today(Box::new(o.today.clone())),
            (Perspective::Week, _, Some(w)) => Page::Week(Box::new(w.clone())),
            _ => return,
        };
        let job = AiJob::Summary {
            brief: self.perspective == Perspective::Today,
        };
        self.working = Some(job);
        self.ai_error = None;
        let fingerprint = self.fingerprint();
        let key = self.ai_key(cx);
        let store = self.store.clone();
        let now = Clock::now_ms(cx);
        let maker = crate::ai_feature::maker(cx);
        let task = cx.background_executor().spawn(async move {
            let (prompt, pairs) = match &page {
                Page::Today(today) => {
                    kanso_engine::loops_ai::overview_today_prompt(&store, today, with_events)
                }
                Page::Week(week) => kanso_engine::loops_ai::overview_week_prompt(&store, week),
            };
            let allowed: Vec<String> = pairs.iter().map(|(_, t)| t.clone()).collect();
            let shared = maker()?;
            let provider = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let summary = kanso_engine::loops_ai::summarise(provider.as_ref(), &prompt, &allowed)
                .map_err(|e| e.to_string())?;
            let warning = summary.warning("page");
            let para = AiPara {
                text: summary.text,
                pairs,
                at: now,
                fingerprint,
                warning,
            };
            if let Ok(json) = serde_json::to_string(&para) {
                store.set_meta(&key, &json).ok();
            }
            Ok::<_, String>(para)
        });
        self.ai_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.working = None;
                match result {
                    Ok(para) => {
                        this.ai_para = Some(para);
                        this.ai_hidden = false;
                        this.ai_more = false;
                    }
                    Err(message) => {
                        this.ai_error = Some((job, page_chrome::ai_trouble(&message)));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Save Week as Note, asking first when it is saved already.
    pub(crate) fn save_week(&mut self, replace: bool, window: &mut Window, cx: &mut Context<Self>) {
        let now = now_of(cx);
        let review = match kanso_engine::brief::weekly_review(&self.store, self.week_start, now) {
            Ok(review) => review,
            Err(err) => {
                cx.emit(DigestEvent::Notice(format!(
                    "Couldn't read the week: {err}"
                )));
                return;
            }
        };
        match kanso_engine::brief::save_review(&self.store, &review, replace, Origin::Ui) {
            Ok(kanso_engine::brief::Saved::Written { id }) => {
                cx.emit(DigestEvent::Open {
                    note_id: id,
                    line: 0,
                    focus: false,
                });
            }
            Ok(kanso_engine::brief::Saved::Exists { .. }) => {
                let label = kanso_engine::brief::iso_week_label(review.from);
                let view = cx.entity();
                crate::dialogs::confirm(
                    format!("Replace the saved review for {label}?"),
                    "The saved note is ordinary text. Replacing it writes this page over it.",
                    "Replace",
                    window,
                    cx,
                    move |window, cx| {
                        view.update(cx, |this, cx| this.save_week(true, window, cx));
                    },
                );
            }
            Err(err) => cx.emit(DigestEvent::Notice(format!(
                "Couldn't save the week: {err}"
            ))),
        }
    }

    // MARK: Drawing: the page

    pub(crate) fn render_overview(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = *Colors::get(cx);
        let wide = f32::from(self.pane_width) >= WIDE;
        let width = if wide {
            COLUMN
        } else {
            (f32::from(self.pane_width) - 48.).clamp(300., COLUMN)
        };
        let header = self.overview_header(window, cx);
        let mut col = v_flex().w(px(width)).pt(px(30.)).pb(px(72.));
        if self.overview.is_none() && self.perspective == Perspective::Today {
            // No snapshot yet (a first open): the date and the greeting need
            // nothing from the store, so they draw at once and the sections
            // follow when the page is worked out. No spinner.
            let now = now_of(cx);
            let day = format(&DateTime::at_midnight(now.today()), "%A %e %B").to_uppercase();
            let overline = self.overline(day, None, cx);
            let greeting = overview::lede::greeting(now.local().hour).to_owned();
            col = self.masthead(col, overline, greeting, String::new(), false, cx);
        } else {
            col = match self.perspective {
                Perspective::Today => self.today_page(col, wide, window, cx),
                Perspective::Week => self.week_page(col, wide, window, cx),
            };
        }
        let undo = self.undo.clone().map(|undo| {
            let view = cx.entity().downgrade();
            h_flex()
                .absolute()
                .bottom(px(20.))
                .left(px(0.))
                .right(px(0.))
                .justify_center()
                .child(
                    h_flex()
                        .gap(px(12.))
                        .h(px(36.))
                        .px(px(14.))
                        .rounded(px(10.))
                        .bg(colors.elevated)
                        .border_1()
                        .border_color(colors.divider)
                        .shadow_md()
                        .items_center()
                        .child(
                            div()
                                .max_w(px(280.))
                                .truncate()
                                .text_size(px(13.))
                                .text_color(colors.text_secondary)
                                .child(format!("Done: {}", undo.title)),
                        )
                        .child(
                            div()
                                .id("overview-undo")
                                .cursor_pointer()
                                .text_size(px(13.))
                                .font_medium()
                                .text_color(colors.link)
                                .child("Undo")
                                .on_click(move |_, _, cx| {
                                    view.update(cx, |this, cx| this.undo_tick(cx)).ok();
                                }),
                        ),
                )
        });
        div()
            .size_full()
            .relative()
            .bg(colors.editor)
            .child(
                v_flex().size_full().child(header).child(
                    div()
                        .id("overview-page")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .track_scroll(&self.page_scroll)
                        .child(page_chrome::centred(col)),
                ),
            )
            .children(undo)
            .into_any_element()
    }

    /// The header band: the title, the Today | This week switch, ✦ Summarise
    /// and the ⋯ menu.
    fn overview_header(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = *Colors::get(cx);
        let _ = window;
        let ai = Self::ai_on(cx);
        let running = self.working.is_some();
        let seg = |id: &'static str, label: &'static str, on: bool, p: Perspective| {
            let view = cx.entity().downgrade();
            div()
                .id(id)
                .h(px(24.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(6.))
                .text_size(px(12.5))
                .cursor_pointer()
                .when(on, |el| {
                    el.bg(colors.editor)
                        .text_color(colors.text)
                        .font_medium()
                        .shadow_sm()
                })
                .when(!on, |el| el.text_color(colors.text_secondary))
                .child(label)
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| this.show_perspective(p, cx))
                        .ok();
                })
        };
        let today_on = self.perspective == Perspective::Today;
        let switch = h_flex()
            .debug_selector(|| "overview-switch".into())
            .p(px(2.))
            .h(px(28.))
            .rounded(px(8.))
            .bg(colors.fill)
            .child(seg("overview-today", "Today", today_on, Perspective::Today))
            .child(seg(
                "overview-week",
                "This week",
                !today_on,
                Perspective::Week,
            ));
        let view = cx.entity().downgrade();
        let summarise = ai.then(|| {
            let view = view.clone();
            Button::new("overview-summarise")
                .ghost()
                .small()
                .icon(
                    Icon::new(Lucide::Sparkles)
                        .size(px(14.))
                        .text_color(colors.accent),
                )
                .label(if running {
                    "Summarising…"
                } else {
                    "Summarise"
                })
                .loading(running)
                .tooltip("Write one short paragraph over this page, with your model")
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| this.summarise_overview(cx)).ok();
                })
        });
        let menu_view = cx.entity().downgrade();
        let week = self.perspective == Perspective::Week;
        let more = div().debug_selector(|| "overview-more".into()).child(
            Button::new("overview-more")
                .ghost()
                .small()
                .icon(
                    Icon::new(Lucide::Ellipsis)
                        .size(px(15.))
                        .text_color(colors.text_secondary),
                )
                .tooltip("More")
                .dropdown_menu(move |menu, _, _| {
                    let open = menu_view.clone();
                    let save = menu_view.clone();
                    let settings = menu_view.clone();
                    let mut menu = menu.item(PopupMenuItem::new("Open Today's Note").on_click(
                        move |_, _, cx| {
                            open.update(cx, |_, cx| cx.emit(DigestEvent::OpenToday))
                                .ok();
                        },
                    ));
                    if week {
                        menu = menu.item(PopupMenuItem::new("Save Week as Note").on_click(
                            move |_, window, cx| {
                                save.update(cx, |this, cx| this.save_week(false, window, cx))
                                    .ok();
                            },
                        ));
                    }
                    menu.item(PopupMenuItem::new("Overview Settings…").on_click(
                        move |_, window, cx| {
                            settings
                                .update(cx, |_, cx| cx.emit(DigestEvent::OpenSettings))
                                .ok();
                            let _ = window;
                        },
                    ))
                }),
        );
        h_flex()
            .debug_selector(|| "overview-header".into())
            .h(px(crate::layout::TOP_BAND_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .pl(px(16.) + self.leading_inset)
            .pr(px(10.))
            .gap(px(6.))
            .items_center()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child("Overview"),
            )
            .child(switch)
            .children(summarise)
            .child(more)
            .into_any_element()
    }

    // MARK: Drawing: masthead

    fn overline(
        &self,
        text: String,
        week_nav: Option<(String, bool)>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = *Colors::get(cx);
        let line = h_flex().gap(px(10.)).h(px(20.)).items_center().child(
            div()
                .text_size(px(11.5))
                .font_semibold()
                .text_color(colors.text_muted)
                .child(text),
        );
        match week_nav {
            None => line.into_any_element(),
            Some((label, is_this_week)) => line
                .child(
                    h_flex()
                        .gap(px(2.))
                        .child(
                            Self::icon_button(
                                "overview-prev",
                                Lucide::ChevronLeft,
                                "Previous week",
                                cx,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.step_week(-1, cx))),
                        )
                        .child(
                            Self::icon_button(
                                "overview-next",
                                Lucide::ChevronRight,
                                "Next week",
                                cx,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.step_week(1, cx))),
                        )
                        .when(!is_this_week, |el| {
                            el.child(
                                Button::new("overview-this-week")
                                    .ghost()
                                    .small()
                                    .label("This week")
                                    .on_click(cx.listener(|this, _, _, cx| this.this_week(cx))),
                            )
                        }),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(label),
                )
                .into_any_element(),
        }
    }

    fn masthead(
        &mut self,
        col: Div,
        overline: AnyElement,
        title: String,
        lede: String,
        show_link: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = *Colors::get(cx);
        col.child(overline)
            .child(
                div()
                    .mt(px(8.))
                    .text_size(px(30.))
                    .line_height(px(36.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(title),
            )
            .child({
                // What's next or now stands out from the counts before it.
                let start = ["Next: ", "Now: "]
                    .iter()
                    .filter_map(|lead| lede.find(lead))
                    .min();
                let highlights: Vec<_> = start
                    .map(|start| {
                        (
                            start..lede.len(),
                            HighlightStyle {
                                color: Some(colors.text.into()),
                                font_weight: Some(FontWeight::MEDIUM),
                                ..Default::default()
                            },
                        )
                    })
                    .into_iter()
                    .collect();
                div()
                    .mt(px(10.))
                    .max_w(px(560.))
                    .text_size(px(16.))
                    .line_height(px(25.))
                    .text_color(colors.text_secondary)
                    .child(StyledText::new(lede).with_highlights(highlights))
            })
            .when(show_link, |el| {
                el.child(
                    h_flex()
                        .id("overview-open-today")
                        .mt(px(16.))
                        .gap(px(6.))
                        .cursor_pointer()
                        .text_size(px(13.5))
                        .text_color(colors.link)
                        .child(Icon::new(Lucide::CalendarDays).size(px(14.)))
                        .child("Open today's note")
                        .child(
                            div()
                                .px(px(4.))
                                .text_size(px(11.))
                                .text_color(colors.text_muted)
                                .border_1()
                                .border_color(colors.divider)
                                .rounded(px(4.))
                                .child("\u{2318}D"),
                        )
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(DigestEvent::OpenToday))),
                )
            })
    }

    /// The model's paragraph: a 2 pt accent rule, rendered as Markdown with
    /// real links, and a footnote.
    fn ai_block(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let colors = *Colors::get(cx);
        if let Some((_, trouble)) = self.ai_error.as_ref() {
            let _ = trouble;
            if let Some(card) = self.error_card(
                &[
                    AiJob::Summary { brief: true },
                    AiJob::Summary { brief: false },
                ],
                window,
                cx,
            ) {
                return Some(div().mt(px(20.)).child(card).into_any_element());
            }
        }
        let para = self.ai_para.clone().filter(|_| !self.ai_hidden)?;
        let linked = overview::summary::link_citations(&para.text, &para.pairs);
        let (shown, cut) = if self.ai_more {
            (linked.clone(), false)
        } else {
            overview::summary::shown(&linked)
        };
        let preview = crate::themes::preview_colors(cx);
        let dark = cx.theme().is_dark();
        let style = crate::preview::text_style(&colors, &preview, dark);
        let ty = crate::preview::preview_type(cx);
        let parts = &self.parts;
        let extensions = crate::preview_rich::register(
            MarkdownExtensions::default()
                .plugin(crate::wikilinks::UnresolvedLink::new(parts.links.clone())),
            parts.rich.clone(),
        );
        let view = cx.entity().downgrade();
        let at = Clock::local_ms(cx, para.at);
        let clock = DateTime::from_local_ms(at);
        let stamp = format!("{:02}:{:02}", clock.hour, clock.minute);
        let changed = para.fingerprint != self.fingerprint();
        let rewrite = cx.entity().downgrade();
        let hide = cx.entity().downgrade();
        let more = cx.entity().downgrade();
        Some(
            v_flex()
                .mt(px(24.))
                .pl(px(16.))
                .border_l(px(2.))
                .border_color(Hsla::from(colors.accent).opacity(0.45))
                .gap(px(8.))
                .child(
                    TextView::markdown(
                        ElementId::Name("overview-ai".into()),
                        SharedString::from(shown),
                    )
                    .style(style)
                    .font_family(ty.family)
                    .text_size(px(15.))
                    .line_height(px(24.))
                    .text_color(preview.text)
                    .selectable(true)
                    .markdown_extensions(extensions)
                    .on_link_click(move |url, _, _, cx| {
                        let url = url.to_string();
                        view.update(cx, |_, cx| cx.emit(DigestEvent::Link(url)))
                            .ok();
                    }),
                )
                .when(cut, |el| {
                    el.child(
                        div()
                            .id("overview-ai-more")
                            .cursor_pointer()
                            .text_size(px(12.5))
                            .text_color(colors.link)
                            .child("More")
                            .on_click(move |_, _, cx| {
                                more.update(cx, |this, cx| {
                                    this.ai_more = true;
                                    cx.notify();
                                })
                                .ok();
                            }),
                    )
                })
                .when_some(para.warning.clone(), |el, warning| {
                    el.child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.danger)
                            .child(warning),
                    )
                })
                .child(
                    h_flex()
                        .gap(px(8.))
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .items_center()
                        .child(
                            Icon::new(Lucide::Sparkles)
                                .size(px(12.))
                                .text_color(colors.accent),
                        )
                        .child(format!(
                            "Written by your model at {stamp}{}",
                            if changed {
                                " \u{b7} before some changes"
                            } else {
                                ""
                            }
                        ))
                        .child(
                            div()
                                .id("overview-ai-rewrite")
                                .cursor_pointer()
                                .text_color(colors.text_secondary)
                                .child("Rewrite")
                                .on_click(move |_, _, cx| {
                                    rewrite
                                        .update(cx, |this, cx| this.summarise_overview(cx))
                                        .ok();
                                }),
                        )
                        .child(
                            div()
                                .id("overview-ai-hide")
                                .cursor_pointer()
                                .text_color(colors.text_secondary)
                                .child("Hide")
                                .on_click(move |_, _, cx| {
                                    hide.update(cx, |this, cx| {
                                        this.ai_hidden = true;
                                        cx.notify();
                                    })
                                    .ok();
                                }),
                        ),
                )
                .into_any_element(),
        )
    }

    // MARK: Drawing: Today

    fn today_page(
        &mut self,
        col: Div,
        wide: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = *Colors::get(cx);
        let Some(page) = self.overview.clone() else {
            return col;
        };
        let t = page.today;
        let overline = self.overline(t.overline.clone(), None, cx);
        if t.state == State::New {
            return self.new_library(col, overline, t.greeting.clone(), cx);
        }
        let quiet = t.state == State::Quiet;
        let mut col = self.masthead(col, overline, t.greeting.clone(), t.lede.clone(), true, cx);
        if let Some(ai) = self.ai_block(window, cx) {
            col = col.child(ai);
        }
        if !t.agenda.is_empty() {
            col = col.child(self.agenda_block(&t.agenda, wide, cx));
        }
        for list in &t.lists {
            col = col.child(self.list_block(list, cx));
        }
        if !t.also.is_empty() {
            col = col.child(self.also_block(&t.also, cx));
        }
        if let Some(past) = &t.past {
            col = col.child(self.coda_block(past, cx));
        }
        if !t.footer.is_empty() {
            col = col.child(
                div()
                    .mt(px(if quiet { 32. } else { 40. }))
                    .pt(px(16.))
                    .border_t_1()
                    .border_color(colors.divider)
                    .text_size(px(12.5))
                    .line_height(px(20.))
                    .text_color(colors.text_secondary)
                    .child(t.footer.clone()),
            );
        }
        col
    }

    fn section_head(&self, label: &str, count: Option<u32>, cx: &App) -> Div {
        let colors = *Colors::get(cx);
        h_flex()
            .gap(px(8.))
            .mb(px(8.))
            .items_baseline()
            .child(
                div()
                    .text_size(px(13.5))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(label.to_owned()),
            )
            .when_some(count.filter(|n| *n > 0), |el, n| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_muted)
                        .child(n.to_string()),
                )
            })
    }

    fn section_shell(&self, cx: &App) -> Div {
        let colors = *Colors::get(cx);
        v_flex()
            .mt(px(38.))
            .pt(px(16.))
            .border_t_1()
            .border_color(colors.divider)
    }

    fn agenda_block(
        &mut self,
        agenda: &overview::Agenda,
        wide: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = *Colors::get(cx);
        let now = Clock::now_ms(cx);
        let _ = now;
        let shown = (agenda.rows.len() as u32) + agenda.more;
        let mut block = self
            .section_shell(cx)
            .child(self.section_head("Today", Some(shown), cx));
        if !agenda.earlier.is_empty() {
            let open = self.expanded.contains("earlier");
            let names: Vec<String> = agenda
                .earlier
                .iter()
                .map(|r| format!("{} {}", r.title, overview::clock(r.at_min.unwrap_or(0))))
                .collect();
            block = block.child(
                h_flex()
                    .id("overview-earlier")
                    .gap(px(8.))
                    .py(px(4.))
                    .mb(px(6.))
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child(
                        Icon::new(if open {
                            Lucide::ChevronDown
                        } else {
                            Lucide::ChevronRight
                        })
                        .size(px(12.))
                        .text_color(colors.text_muted),
                    )
                    .child("Earlier today")
                    .child(
                        div()
                            .text_color(colors.text_muted)
                            .truncate()
                            .child(format!("\u{b7} {}", names.join(" \u{b7} "))),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        if !this.expanded.remove("earlier") {
                            this.expanded.insert("earlier".into());
                        }
                        cx.notify();
                    })),
            );
            if open {
                for row in &agenda.earlier {
                    block = block.child(self.agenda_row(row, wide, true, cx));
                }
            }
        }
        for row in &agenda.rows {
            block = block.child(self.agenda_row(row, wide, false, cx));
        }
        if agenda.more > 0 {
            let label = format!("{} more today", agenda.more);
            block = block.child(
                // The count is words, not a link; Tasks is where the rest are.
                h_flex()
                    .mt(px(4.))
                    .gap(px(12.))
                    .text_size(px(13.))
                    .text_color(colors.text_secondary)
                    .child(label)
                    .when(agenda.more_tasks > 0, |el| {
                        el.child(
                            div()
                                .id("overview-all-tasks")
                                .text_color(colors.link)
                                .cursor_pointer()
                                .child(format!("All {} in Tasks \u{2192}", agenda.task_total))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.library.update(cx, |library, cx| {
                                        library.set_filter(Filter::Tasks, cx)
                                    });
                                })),
                        )
                    }),
            );
        }
        block
    }

    /// A row under a day's heading in *Ahead*: the heading says the day,
    /// so a task without a time has no gutter (not "Today").
    fn ahead_row(&mut self, row: &AgendaRow, wide: bool, cx: &mut Context<Self>) -> AnyElement {
        self.agenda_row_in(row, wide, false, true, cx)
    }

    fn agenda_row(
        &mut self,
        row: &AgendaRow,
        wide: bool,
        dimmed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.agenda_row_in(row, wide, dimmed, false, cx)
    }

    fn agenda_row_in(
        &mut self,
        row: &AgendaRow,
        wide: bool,
        dimmed: bool,
        day_shown: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = *Colors::get(cx);
        let (mut gut, late, accent) = gutter(row);
        if day_shown && row.kind == AgendaKind::Due && row.at_min.is_none() {
            gut = String::new();
        }
        let id = SharedString::from(format!("agenda-{}-{}", row.key, row.line));
        let group = id.clone();
        let is_event = row.kind == AgendaKind::Event;
        let gutter_color = if late {
            colors.warning
        } else if accent {
            colors.accent
        } else if row.free {
            colors.text_muted
        } else {
            colors.text_secondary
        };
        let glyph: AnyElement =
            if is_event {
                div()
                    .w(px(16.))
                    .flex_none()
                    .flex()
                    .justify_center()
                    .child(div().mt(px(5.)).size(px(9.)).rounded(px(3.)).bg(
                        if row.color.is_empty() {
                            Hsla::from(colors.accent)
                        } else {
                            parse_hex(&row.color).unwrap_or(Hsla::from(colors.accent))
                        },
                    ))
                    .into_any_element()
            } else {
                let tick = row.clone();
                div()
                    .id(SharedString::from(format!("tick-{}-{}", row.key, row.line)))
                    .flex_none()
                    .w(px(16.))
                    .pt(px(2.))
                    .cursor_pointer()
                    .child(
                        Icon::new(if row.kind == AgendaKind::Reminder {
                            Lucide::Bell
                        } else {
                            Lucide::Circle
                        })
                        .size(px(16.))
                        .text_color(colors.text_muted),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.tick_agenda(&tick, cx);
                    }))
                    .into_any_element()
            };
        let open = row.clone();
        let meeting_note = row.meeting.clone();
        let hang = if wide { -HANG } else { 0. };
        let gutter_w = if wide { 64. } else { 48. };
        div()
            .id(id)
            .group(group)
            .relative()
            .ml(px(hang))
            .when(dimmed, |el| el.opacity(0.6))
            .child(
                h_flex()
                    .w_full()
                    .gap(px(6.))
                    .items_start()
                    .py(px(8.))
                    .pr(px(10.))
                    .rounded(px(7.))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.row_hover))
                    .when(row.now, |el| el.border_l_2().border_color(colors.accent))
                    .child(
                        div()
                            .w(px(gutter_w))
                            .flex_none()
                            .text_right()
                            .pt(px(1.))
                            .text_size(px(12.))
                            .line_height(px(18.))
                            .text_color(gutter_color)
                            .when(late || accent, |el| el.font_semibold())
                            .child(gut),
                    )
                    .child(glyph)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .line_height(px(20.))
                                    .text_color(colors.text)
                                    .when(is_event, |el| el.font_medium())
                                    .child(row.title.clone()),
                            )
                            .when(!row.context.is_empty(), |el| {
                                el.child(
                                    div()
                                        .truncate()
                                        .text_size(px(12.5))
                                        .line_height(px(18.))
                                        .text_color(colors.text_secondary)
                                        .child(row.context.clone()),
                                )
                            }),
                    )
                    .when_some(meeting_note, |el, meeting| {
                        let note = meeting.note_id.clone();
                        el.child(
                            h_flex()
                                .id(SharedString::from(format!("meeting-{}", row.key)))
                                .flex_none()
                                .self_center()
                                .gap(px(4.))
                                .text_size(px(12.5))
                                .text_color(colors.link)
                                .child(if meeting.locked {
                                    "Meeting note (locked)".to_owned()
                                } else {
                                    "Meeting note".to_owned()
                                })
                                .child(Icon::new(Lucide::ChevronRight).size(px(12.)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.open_row(&note, 0, cx);
                                })),
                        )
                    }),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open_row(&open.note_id, open.line, cx);
            }))
            .into_any_element()
    }

    fn list_block(&mut self, list: &overview::List, cx: &mut Context<Self>) -> Div {
        let colors = *Colors::get(cx);
        let mut block =
            self.section_shell(cx)
                .child(self.section_head(list.kind.label(), None, cx));
        let key = format!("more-{:?}", list.kind);
        let expanded = self.expanded.contains(&key);
        // `N more` shows the rest in place.
        let shown = list
            .rows
            .iter()
            .chain(list.extra.iter().filter(|_| expanded));
        for (ix, row) in shown.enumerate() {
            block = block.child(match list.kind {
                ListKind::PickUp => self.pick_up_row(row, ix, cx),
                ListKind::Waiting => self.waiting_row(row, ix, cx),
                ListKind::Activity => self.activity_row(row, ix, cx),
            });
        }
        if list.more > 0 && !expanded {
            let k = key.clone();
            block = block.child(
                div()
                    .id(SharedString::from(format!("more-{:?}", list.kind)))
                    .mt(px(4.))
                    .cursor_pointer()
                    .text_size(px(13.))
                    .text_color(colors.link)
                    .child(format!("{} more", list.more))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.expanded.insert(k.clone());
                        cx.notify();
                    })),
            );
        }
        block
    }

    fn pick_up_row(&mut self, row: &ListRow, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let colors = *Colors::get(cx);
        let now = Clock::now_ms(cx);
        let offset = i64::from(Clock::utc_offset_secs(cx, now)) * 1000;
        let group = SharedString::from(format!("pick-{ix}"));
        let open = row.clone();
        let hide = row.clone();
        let revisit_note = row.note_id.clone();
        let view = cx.entity().downgrade();
        let when = overview::ago(row.at, now, offset);
        let meta = if row.place.is_empty() {
            when
        } else {
            format!("{} \u{b7} {when}", row.place)
        };
        div()
            .id(group.clone())
            .group(group.clone())
            .py(px(9.))
            .cursor_pointer()
            .rounded(px(7.))
            .hover(|el| el.bg(colors.row_hover))
            .child(
                h_flex()
                    .h(px(22.))
                    .gap(px(10.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(14.5))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(untitled(&row.title)),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .gap(px(4.))
                            .opacity(0.)
                            .group_hover(group.clone(), |s| s.opacity(1.))
                            .child(
                                Self::icon_button(
                                    SharedString::from(format!("pick-revisit-{ix}")),
                                    Lucide::Clock,
                                    "Revisit in\u{2026}",
                                    cx,
                                )
                                .dropdown_menu(
                                    move |menu, _, cx| revisit_menu(menu, &view, &revisit_note, cx),
                                ),
                            )
                            .child(
                                Self::icon_button(
                                    SharedString::from(format!("pick-hide-{ix}")),
                                    Lucide::X,
                                    "Not now",
                                    cx,
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.hide_place(&hide, cx);
                                    },
                                )),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(meta),
                    ),
            )
            .when(!row.detail.is_empty(), |el| {
                // The paragraph wraps in the column; the caret is a glyph
                // at its end, in the accent, so it sits where the words
                // stop, on whatever line that is.
                let mut text = row.detail.clone();
                let mut highlights = Vec::new();
                if row.caret {
                    let start = text.len();
                    text.push('\u{258f}');
                    highlights.push((
                        start..text.len(),
                        HighlightStyle {
                            color: Some(colors.accent.into()),
                            ..Default::default()
                        },
                    ));
                }
                el.child(
                    div()
                        .mt(px(2.))
                        .text_size(px(14.))
                        .line_height(px(22.))
                        .text_color(colors.text_secondary)
                        .child(StyledText::new(text).with_highlights(highlights)),
                )
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open_row(&open.note_id, open.line, cx);
            }))
            .into_any_element()
    }

    fn waiting_row(&mut self, row: &ListRow, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let colors = *Colors::get(cx);
        let group = SharedString::from(format!("wait-{ix}"));
        let open = row.clone();
        let done = row.clone();
        let snooze = row.clone();
        let person = row.person.clone();
        let quiet = format!("{} quiet", plural(row.quiet_days, "day", "days"));
        let warn = row.quiet_days >= 10;
        h_flex()
            .id(group.clone())
            .group(group.clone())
            .gap(px(14.))
            .py(px(8.))
            .px(px(2.))
            .items_start()
            .cursor_pointer()
            .rounded(px(7.))
            .hover(|el| el.bg(colors.row_hover))
            .child(
                div()
                    .id(SharedString::from(format!("wait-who-{ix}")))
                    .w(px(108.))
                    .flex_none()
                    .truncate()
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(row.title.clone())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.open_row(&person, 0, cx);
                    })),
            )
            // What for, then the note it is in.
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .truncate()
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(colors.text)
                            .child(row.detail.clone()),
                    )
                    .when(!row.place.is_empty(), |el| {
                        el.child(
                            div()
                                .truncate()
                                .text_size(px(12.5))
                                .line_height(px(18.))
                                .text_color(colors.text_secondary)
                                .child(row.place.clone()),
                        )
                    }),
            )
            .child(
                h_flex()
                    .flex_none()
                    .self_center()
                    .gap(px(2.))
                    .opacity(0.)
                    .group_hover(group, |s| s.opacity(1.))
                    .child(
                        Self::icon_button(
                            SharedString::from(format!("wait-done-{ix}")),
                            Lucide::Check,
                            "Done",
                            cx,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.done_waiting(&done, cx);
                        })),
                    )
                    .child(
                        Self::icon_button(
                            SharedString::from(format!("wait-later-{ix}")),
                            Lucide::AlarmClock,
                            "Not yet: ask again in 3 days",
                            cx,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.snooze_waiting(&snooze, cx);
                        })),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .text_size(px(12.5))
                    .text_color(if warn {
                        colors.warning
                    } else {
                        colors.text_secondary
                    })
                    .when(warn, |el| el.font_medium())
                    .child(quiet),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open_row(&open.note_id, open.line, cx);
            }))
            .into_any_element()
    }

    fn activity_row(&mut self, row: &ListRow, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let colors = *Colors::get(cx);
        let now = Clock::now_ms(cx);
        let offset = i64::from(Clock::utc_offset_secs(cx, now)) * 1000;
        let agent = matches!(row.kind, RowKind::AgentSuggestion | RowKind::AgentTask);
        let open = row.clone();
        h_flex()
            .id(SharedString::from(format!("activity-{ix}")))
            .gap(px(10.))
            .py(px(8.))
            .items_baseline()
            .cursor_pointer()
            .rounded(px(7.))
            .hover(|el| el.bg(colors.row_hover))
            .child(
                Icon::new(if agent {
                    Lucide::Sparkles
                } else {
                    Lucide::Users
                })
                .size(px(14.))
                .text_color(if agent {
                    colors.accent
                } else {
                    colors.text_muted
                }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .text_color(colors.text)
                    .child(row.title.clone()),
            )
            .child(
                div()
                    .flex_none()
                    .text_size(px(12.5))
                    .text_color(if agent {
                        colors.link
                    } else {
                        colors.text_secondary
                    })
                    .when(agent, |el| el.font_medium())
                    .child(if agent {
                        "Review".to_owned()
                    } else {
                        overview::ago(row.at, now, offset)
                    }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                if matches!(open.kind, RowKind::AgentSuggestion | RowKind::AgentTask) {
                    crate::review_queue::open_review(window, cx);
                } else {
                    this.open_row(&open.note_id, open.line, cx);
                }
            }))
            .into_any_element()
    }

    fn also_block(&mut self, lines: &[AlsoLine], cx: &mut Context<Self>) -> Div {
        let colors = *Colors::get(cx);
        let mut block = self
            .section_shell(cx)
            .child(self.section_head("Also", None, cx));
        let last = lines.len().saturating_sub(1);
        for (ix, line) in lines.iter().enumerate() {
            block = block.child(self.also_line(line, ix, ix == last, cx));
        }
        let _ = colors;
        block
    }

    fn also_line(
        &mut self,
        line: &AlsoLine,
        ix: usize,
        last: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = *Colors::get(cx);
        let label = also_label(line);
        let mut inline: Vec<AnyElement> = Vec::new();
        let count_text = match line.kind {
            AlsoKind::Captured => format!("Inbox {}", line.count),
            AlsoKind::Promises => format!(
                "{} {} you said you'd do",
                line.count,
                if line.count == 1 { "thing" } else { "things" }
            ),
            AlsoKind::Folded(_) => format!("{} waiting", line.count),
            AlsoKind::Revisit if line.count > 0 => String::new(),
            _ => String::new(),
        };
        if !count_text.is_empty() {
            inline.push(
                div()
                    .flex_none()
                    .text_color(colors.text_secondary)
                    .child(count_text)
                    .into_any_element(),
            );
        }
        // Items give way first, each ending in an ellipsis, so the action
        // at the end of the line is never pushed out or overlapped.
        for (i, item) in line.items.iter().enumerate() {
            let open = item.clone();
            let text = if item.voice_secs > 0 {
                format!(
                    "\u{201c}{}\u{201d} {}:{:02}",
                    item.text,
                    item.voice_secs / 60,
                    item.voice_secs % 60
                )
            } else if line.kind == AlsoKind::Revisit {
                item.text.clone()
            } else {
                format!("\u{201c}{}\u{201d}", item.text)
            };
            inline.push(
                h_flex()
                    .id(SharedString::from(format!("also-{ix}-{i}")))
                    .min_w_0()
                    .flex_shrink(1.)
                    .gap(px(4.))
                    .cursor_pointer()
                    .text_color(colors.text)
                    .when(item.voice_secs > 0, |el| {
                        el.child(
                            Icon::new(Lucide::Mic)
                                .size(px(12.))
                                .text_color(colors.text_secondary),
                        )
                    })
                    .child(div().min_w_0().truncate().child(text))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_row(&open.note_id, open.line, cx);
                    }))
                    .into_any_element(),
            );
        }
        if line.kind == AlsoKind::Revisit && line.cards > 0 {
            inline.push(
                div()
                    .text_color(colors.text_secondary)
                    .child(plural(line.cards, "flashcard", "flashcards"))
                    .into_any_element(),
            );
        }
        if line.kind == AlsoKind::Themes {
            inline.push(
                div()
                    .text_color(colors.link)
                    .child(line.detail.join(" \u{b7} "))
                    .into_any_element(),
            );
        }
        let action: Option<AlsoAction> = match line.kind {
            AlsoKind::Captured => Some(("Triage", |_, window, cx| {
                window.dispatch_action(Box::new(crate::triage_view::ShowTriage), cx)
            })),
            AlsoKind::Promises => Some(("See all", |this, _, cx| {
                this.library
                    .update(cx, |library, cx| library.set_filter(Filter::OpenLoops, cx));
            })),
            AlsoKind::Revisit if line.cards > 0 => Some(("Review", |_, window, cx| {
                window.dispatch_action(Box::new(crate::flashcards_view::ShowFlashcards), cx)
            })),
            AlsoKind::Folded(ListKind::Activity) => Some(("Review", |_, window, cx| {
                crate::review_queue::open_review(window, cx);
            })),
            _ => None,
        };
        let mut row = h_flex()
            .gap(px(14.))
            .py(px(7.))
            .items_baseline()
            .text_size(px(13.5))
            .line_height(px(20.))
            .when(!last, |el| {
                el.border_b_1()
                    .border_color(Hsla::from(colors.divider).opacity(0.55))
            })
            .child(
                div()
                    .w(px(112.))
                    .flex_none()
                    .text_size(px(12.5))
                    .font_semibold()
                    .text_color(colors.text_secondary)
                    .child(label),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .overflow_hidden()
                    .children(
                        inline
                            .into_iter()
                            .enumerate()
                            .flat_map(|(i, el)| {
                                let sep = (i > 0).then(|| {
                                    div()
                                        .flex_none()
                                        .text_color(colors.text_muted)
                                        .child("\u{b7}")
                                        .into_any_element()
                                });
                                sep.into_iter().chain(std::iter::once(el))
                            })
                            .collect::<Vec<_>>(),
                    ),
            );
        if let Some((text, run)) = action {
            row = row.child(
                div()
                    .id(SharedString::from(format!("also-act-{ix}")))
                    .flex_none()
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .font_medium()
                    .text_color(colors.link)
                    .child(text)
                    .on_click(cx.listener(move |this, _, window, cx| run(this, window, cx))),
            );
        }
        row.into_any_element()
    }

    fn coda_block(&mut self, past: &PastNote, cx: &mut Context<Self>) -> Div {
        let colors = *Colors::get(cx);
        let open = past.clone();
        let dismiss = past.clone();
        let revisit_note = past.note_id.clone();
        let view = cx.entity().downgrade();
        v_flex()
            .mt(px(44.))
            .pt(px(22.))
            .border_t_1()
            .border_color(colors.divider)
            .child(
                div()
                    .text_size(px(11.5))
                    .font_semibold()
                    .text_color(colors.text_muted)
                    .child(format!(
                        "A NOTE FROM THE PAST \u{b7} {}",
                        past.why.to_uppercase()
                    )),
            )
            .child(
                div()
                    .id("overview-coda-title")
                    .mt(px(8.))
                    .cursor_pointer()
                    .text_size(px(17.))
                    .line_height(px(24.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(untitled(&past.title))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_row(&open.note_id, 0, cx);
                    })),
            )
            .when(!past.excerpt.is_empty(), |el| {
                el.child(
                    div()
                        .mt(px(6.))
                        .pl(px(14.))
                        .border_l_2()
                        .border_color(colors.divider)
                        .text_size(px(15.))
                        .line_height(px(24.))
                        .text_color(colors.text_secondary)
                        .child(format!("\u{201c}{}\u{201d}", past.excerpt)),
                )
            })
            .child(
                h_flex()
                    .mt(px(10.))
                    .gap(px(14.))
                    .text_size(px(12.5))
                    .text_color(colors.link)
                    .child(
                        div()
                            .id("overview-coda-open")
                            .cursor_pointer()
                            .child("Open")
                            .on_click(cx.listener({
                                let id = past.note_id.clone();
                                move |this, _, _, cx| this.open_row(&id, 0, cx)
                            })),
                    )
                    .child(
                        Button::new("overview-coda-revisit")
                            .ghost()
                            .small()
                            .label("Revisit in\u{2026}")
                            .dropdown_menu(move |menu, _, cx| {
                                revisit_menu(menu, &view, &revisit_note, cx)
                            }),
                    )
                    .child(
                        div()
                            .id("overview-coda-no")
                            .cursor_pointer()
                            .text_color(colors.text_secondary)
                            .child("Not useful")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.dismiss_past(&dismiss, cx);
                            })),
                    ),
            )
    }

    /// A library with nothing in it: what the page is for, two buttons and
    /// three examples of how it fills in.
    fn new_library(
        &mut self,
        col: Div,
        overline: AnyElement,
        greeting: String,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = *Colors::get(cx);
        // What a note says, in its own words (code for Markdown, italics
        // for a description), and where on the page it shows.
        let example = |text: &'static str, to: &'static str, code: bool| {
            h_flex()
                .gap(px(14.))
                .py(px(11.))
                .items_center()
                .border_b_1()
                .border_color(Hsla::from(colors.divider).opacity(0.6))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .map(|el| {
                            if code {
                                el.font_family("Menlo").text_color(colors.text)
                            } else {
                                el.italic().text_color(colors.text_secondary)
                            }
                        })
                        .child(text),
                )
                .child(
                    Icon::new(Lucide::ArrowRight)
                        .size(px(14.))
                        .text_color(colors.text_muted),
                )
                .child(
                    div()
                        .w(px(190.))
                        .text_size(px(12.5))
                        .font_medium()
                        .text_color(colors.text_secondary)
                        .child(to),
                )
        };
        col.child(overline)
            .child(
                div()
                    .mt(px(8.))
                    .text_size(px(30.))
                    .line_height(px(36.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(greeting),
            )
            .child(
                div()
                    .mt(px(10.))
                    .max_w(px(560.))
                    .text_size(px(16.))
                    .line_height(px(25.))
                    .text_color(colors.text_secondary)
                    .child("This is your Overview. As you write, it gathers what is due today, the notes you're in the middle of, and the people you're waiting on."),
            )
            .child(
                h_flex()
                    .mt(px(22.))
                    .gap(px(8.))
                    .child(
                        Button::new("overview-write-today")
                            .primary()
                            .icon(Lucide::CalendarDays)
                            .label("Write today's note")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DigestEvent::OpenToday))),
                    )
                    .child(
                        Button::new("overview-import")
                            .outline()
                            .icon(Lucide::Download)
                            .label("Import notes\u{2026}")
                            .on_click(cx.listener(|_, _, window, cx| {
                                window.dispatch_action(Box::new(crate::workspace::ImportLibrary), cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .mt(px(38.))
                    .pt(px(16.))
                    .border_t_1()
                    .border_color(colors.divider)
                    .child(self.section_head("How this page fills in", None, cx))
                    .child(example(
                        "- [ ] Send the deck by Friday @2026-11-07",
                        "Today",
                        true,
                    ))
                    .child(example(
                        "Waiting on @Rudo for the quote",
                        "Waiting on others",
                        true,
                    ))
                    .child(example(
                        "The paragraph you stopped in",
                        "Pick up where you left off",
                        false,
                    )),
            )
    }

    // MARK: Drawing: This week

    fn week_page(
        &mut self,
        col: Div,
        wide: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = *Colors::get(cx);
        let Some(week) = self.week.clone() else {
            return col.child(div().h(px(120.)));
        };
        let this_week = kanso_engine::brief::week_start(now_of(cx).today()) == self.week_start;
        let overline = self.overline(
            week.range_label.clone(),
            Some((week.week_label.clone(), this_week)),
            cx,
        );
        let mut col = self.masthead(
            col,
            overline,
            if this_week {
                "This week".into()
            } else {
                "That week".into()
            },
            week.lede.clone(),
            false,
            cx,
        );
        if let Some(ai) = self.ai_block(window, cx) {
            col = col.child(ai);
        }
        if !week.projects.is_empty() {
            let mut block =
                self.section_shell(cx)
                    .child(self.section_head("Active projects", None, cx));
            for (ix, p) in week.projects.iter().enumerate() {
                let open = p.clone();
                let last = if p.last_title.is_empty() {
                    String::new()
                } else {
                    format!(" \u{b7} last: \u{201c}{}\u{201d}", p.last_title)
                };
                let next = if p.next_task.is_empty() {
                    None
                } else if p.next_due.is_empty() {
                    Some(format!("Next: {}", p.next_task))
                } else {
                    Some(format!(
                        "Next: {} \u{b7} {}",
                        p.next_task,
                        due_words(&p.next_due, now_of(cx).today())
                    ))
                };
                block = block.child(
                    v_flex()
                        .id(SharedString::from(format!("project-{ix}")))
                        .py(px(9.))
                        .cursor_pointer()
                        .rounded(px(7.))
                        .hover(|el| el.bg(colors.row_hover))
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_semibold()
                                .text_color(colors.text)
                                .child(p.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text_secondary)
                                .child(format!(
                                    "{} changed{last}",
                                    plural(p.notes_changed, "note", "notes")
                                )),
                        )
                        .when_some(next, |el, next| {
                            el.child(
                                div()
                                    .mt(px(2.))
                                    .text_size(px(13.))
                                    .text_color(colors.text)
                                    .child(next),
                            )
                        })
                        // The notebook's homepage, which links its notes.
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.library.update(cx, |library, cx| {
                                library.set_filter(Filter::Book(open.id.clone()), cx);
                                library.set_home(true, cx);
                            });
                        })),
                );
            }
            col = col.child(block);
        }
        if !week.finished.is_empty() {
            let mut block = self.section_shell(cx).child(self.section_head(
                "Finished",
                Some(week.finished.len() as u32 + week.finished_more),
                cx,
            ));
            for (ix, f) in week.finished.iter().enumerate() {
                let id = f.note_id.clone();
                block = block.child(
                    h_flex()
                        .id(SharedString::from(format!("finished-{ix}")))
                        .gap(px(10.))
                        .py(px(6.))
                        .cursor_pointer()
                        .items_center()
                        .child(
                            Icon::new(Lucide::Check)
                                .size(px(14.))
                                .text_color(colors.success),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(px(14.))
                                .text_color(colors.text_secondary)
                                .child(f.text.clone()),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| this.open_row(&id, 0, cx))),
                );
            }
            if week.finished_more > 0 {
                block = block.child(
                    div()
                        .mt(px(4.))
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child(format!("{} more", week.finished_more)),
                );
            }
            col = col.child(block);
        }
        if !week.waiting.is_empty() {
            let mut block =
                self.section_shell(cx)
                    .child(self.section_head("Waiting on others", None, cx));
            for (ix, row) in week.waiting.iter().enumerate() {
                block = block.child(self.waiting_row(row, ix + 100, cx));
            }
            col = col.child(block);
        }
        if !week.still_open.is_empty() {
            let mut block = self.section_shell(cx).child(self.section_head(
                "Still open",
                Some(week.still_open_total),
                cx,
            ));
            for row in &week.still_open {
                block = block.child(self.agenda_row(row, wide, false, cx));
            }
            if week.still_open_total as usize > week.still_open.len() {
                block = block.child(
                    div()
                        .id("overview-all-open")
                        .mt(px(4.))
                        .cursor_pointer()
                        .text_size(px(13.))
                        .text_color(colors.link)
                        .child(format!("All {} open tasks \u{2192}", week.still_open_total))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.library
                                .update(cx, |library, cx| library.set_filter(Filter::Tasks, cx));
                        })),
                );
            }
            col = col.child(block);
        }
        if !week.ahead.is_empty() {
            let mut block = self
                .section_shell(cx)
                .child(self.section_head("Ahead", None, cx));
            for day in &week.ahead {
                if day.rows.is_empty() {
                    block = block.child(
                        div()
                            .py(px(6.))
                            .text_size(px(13.))
                            .text_color(colors.text_secondary)
                            .child(day.summary.clone()),
                    );
                } else {
                    block = block.child(
                        div()
                            .mt(px(10.))
                            .mb(px(2.))
                            .text_size(px(12.5))
                            .font_semibold()
                            .text_color(colors.text_secondary)
                            .child(day.summary.clone()),
                    );
                    for row in &day.rows {
                        block = block.child(self.ahead_row(row, wide, cx));
                    }
                }
            }
            col = col.child(block);
        }
        if !week.also.is_empty() {
            col = col.child(self.also_block(&week.also, cx));
        }
        let saved = week.saved_as.clone();
        col.child(
            h_flex()
                .mt(px(40.))
                .pt(px(16.))
                .border_t_1()
                .border_color(colors.divider)
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(if saved.is_empty() {
                    div()
                        .id("overview-save-week")
                        .cursor_pointer()
                        .text_color(colors.link)
                        .child("Save this week as a note")
                        .on_click(
                            cx.listener(|this, _, window, cx| this.save_week(false, window, cx)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .child(format!("Saved as \u{201c}{saved}\u{201d}"))
                        .into_any_element()
                }),
        )
    }
}

/// The *Revisit in…* menu: the six dates, each with its day.
fn revisit_menu(
    mut menu: gpui_kit::component::menu::PopupMenu,
    view: &WeakEntity<DigestView>,
    note_id: &str,
    cx: &mut App,
) -> gpui_kit::component::menu::PopupMenu {
    let today = Clock::now_ms(cx);
    let offset = i64::from(Clock::utc_offset_secs(cx, today)) * 1000;
    let day = Date::from_local_ms(today + offset);
    for choice in Choice::ALL {
        let view = view.clone();
        let id = note_id.to_owned();
        let date = choice.date(day);
        menu = menu.item(
            PopupMenuItem::new(format!(
                "{}   {}",
                choice.label(),
                format(&DateTime::at_midnight(date), "%a %e %b")
            ))
            .on_click(move |_, _, cx| {
                let id = id.clone();
                view.update(cx, |this, cx| this.revisit_note(&id, choice, cx))
                    .ok();
            }),
        );
    }
    menu
}

/// `#rrggbb` as a colour.
fn parse_hex(text: &str) -> Option<Hsla> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(rgb(value).into())
}

/// The agent suggestions waiting for review, grouped by note and agent.
fn agent_items(cx: &App) -> Vec<overview::ActivityItem> {
    let Some(queue) = crate::review_queue::queue(cx) else {
        return Vec::new();
    };
    let mut out: Vec<overview::ActivityItem> = Vec::new();
    for change in queue.list() {
        let note_id = change.note_id.clone().unwrap_or_default();
        match out
            .iter_mut()
            .find(|i| i.note_id == note_id && i.who == change.agent)
        {
            Some(item) => {
                item.count += 1;
                item.at = item.at.max(change.created_at);
            }
            None => out.push(overview::ActivityItem {
                kind: RowKind::AgentSuggestion,
                note_id,
                title: if change.is_new_note() {
                    change.new_title.clone()
                } else {
                    change.old_title.clone()
                },
                who: change.agent.clone(),
                count: 1,
                at: change.created_at,
            }),
        }
    }
    out
}
