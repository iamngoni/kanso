//! Open loops, the morning brief and the weekly review, in the note list's
//! place (`Filter::OpenLoops`, `Filter::Brief`, `Filter::Review`).
//!
//! Everything shown is computed when it opens and again when the library
//! changes (`kanso_engine::brief`): nothing is stored, and nothing is sent
//! to a model unless one of the explicit AI buttons is pressed. The parts
//! that decide (which loops, what is due, the themes) are the engine's;
//! this draws them and turns clicks into store writes.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{
    Disableable as _, Icon, Sizable as _, StyledExt as _, VirtualListScrollHandle, h_flex, v_flex, v_virtual_list,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::{Date, DateTime};
use kanso_core::loops::{AiLoop, LoopKind, age_label};
use kanso_engine::brief::{
    self, Brief, Now, Review, ThemeKind, Why, morning_brief, weekly_review,
};
use kanso_engine::loops_ai::{self, Proposal, Scan, Summary};
use kanso_store::{CheckOff, LoopClose, LoopQuery, LoopRow, Origin, Store, TaskRow};

use crate::clock::Clock;
use crate::layout::TOP_BAND_HEIGHT;
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_list::quiet_empty;
use crate::theme::{Colors, section_label};
use crate::workspace::AppState;
use crate::zoom::px;

const ROW_HEIGHT: f32 = 54.;
const HEADER_HEIGHT: f32 = 32.;
const ROW_INSET_X: f32 = 8.;
const ROW_PAD_X: f32 = 10.;
/// How long changes to notes settle before the page is worked out again.
const RELOAD_DELAY: Duration = Duration::from_millis(500);
/// How often the page is worked out again (a task falls due as time passes).
const TICK: Duration = Duration::from_secs(60);

actions!(
    digest,
    [OpenMorningBrief, OpenWeeklyReview, OpenOpenLoops, FindMoreOpenLoops]
);

/// The workspace's actions for the three pages.
pub fn on_actions(el: Div, cx: &mut Context<crate::workspace::Workspace>) -> Div {
    use crate::workspace::Workspace;
    let show = |filter: Filter| {
        move |this: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>| {
            this.leave_settings(window, cx);
            this.set_focus_mode(false, window, cx);
            this.library
                .update(cx, |library, cx| library.set_filter(filter.clone(), cx));
        }
    };
    let brief = show(Filter::Brief);
    let review = show(Filter::Review);
    let loops = show(Filter::OpenLoops);
    let loops_again = show(Filter::OpenLoops);
    el.on_action(cx.listener(move |this, _: &OpenMorningBrief, window, cx| brief(this, window, cx)))
        .on_action(
            cx.listener(move |this, _: &OpenWeeklyReview, window, cx| review(this, window, cx)),
        )
        .on_action(cx.listener(move |this, _: &OpenOpenLoops, window, cx| loops(this, window, cx)))
        .on_action(cx.listener(move |this, _: &FindMoreOpenLoops, window, cx| {
            loops_again(this, window, cx);
            this.digest
                .update(cx, |digest, cx| digest.ask_find_more(window, cx));
        }))
}

/// What the view tells the workspace.
pub enum DigestEvent {
    /// Open the note at a line; `focus` puts the keyboard in it.
    Open {
        note_id: String,
        line: u32,
        focus: bool,
    },
    /// Open today's daily note.
    OpenToday,
    /// Something to tell the user.
    Notice(String),
}

/// Which of the three pages shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Loops,
    Brief,
    Review,
}

/// A line of the loops list.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    Header { kind: LoopKind, count: usize },
    Loop(usize),
}

/// The AI sheet: the loops a run proposed, for the person to choose from.
struct Found {
    scan: Scan,
    checked: Vec<bool>,
}

pub struct DigestView {
    library: Entity<Library>,
    editor: Entity<crate::editor::NoteEditor>,
    store: Arc<Store>,
    loops: Vec<LoopRow>,
    loaded: bool,
    brief: Option<Brief>,
    review: Option<Review>,
    week_start: Date,
    /// An AI paragraph over the brief or the review, until the page changes.
    summary: Option<Summary>,
    found: Option<Found>,
    /// What the AI is doing now, if anything.
    working: Option<&'static str>,
    cancel: Arc<AtomicBool>,
    ai_task: Option<Task<()>>,
    focus: FocusHandle,
    list_scroll: VirtualListScrollHandle,
    page_scroll: ScrollHandle,
    leading_inset: Pixels,
    stale: bool,
    reload_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
    _tick: Task<()>,
}

impl EventEmitter<DigestEvent> for DigestView {}

impl Focusable for DigestView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// The loops grouped as the list shows them: waiting on others, mine, then
/// questions; each group in the store's order.
fn entries(loops: &[LoopRow]) -> Vec<Entry> {
    let mut out = Vec::new();
    for kind in [LoopKind::Waiting, LoopKind::Mine, LoopKind::Question] {
        let rows: Vec<usize> = loops
            .iter()
            .enumerate()
            .filter(|(_, l)| l.kind == kind)
            .map(|(ix, _)| ix)
            .collect();
        if rows.is_empty() {
            continue;
        }
        out.push(Entry::Header {
            kind,
            count: rows.len(),
        });
        out.extend(rows.into_iter().map(Entry::Loop));
    }
    out
}

fn group_label(kind: LoopKind) -> &'static str {
    match kind {
        LoopKind::Waiting => "Waiting on others",
        LoopKind::Mine => "Mine",
        LoopKind::Question => "Questions",
    }
}

fn now_of(cx: &App) -> Now {
    let utc_ms = Clock::now_ms(cx);
    Now {
        utc_ms,
        offset_ms: i64::from(Clock::utc_offset_secs(cx, utc_ms)) * 1000,
    }
}

fn writes_held(cx: &App) -> bool {
    cx.try_global::<crate::services::Services>()
        .is_some_and(|services| services.writes_held)
}

impl DigestView {
    pub fn new(
        library: Entity<Library>,
        editor: Entity<crate::editor::NoteEditor>,
        cx: &mut Context<Self>,
    ) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![cx.subscribe(&library, |this, library, event, cx| {
            let shown = this.mode(cx).is_some();
            match event {
                LibraryEvent::Changed(change) if change.id.starts_with("note:") => {
                    this.stale = true;
                    if shown {
                        this.reload_soon(cx);
                    }
                }
                LibraryEvent::Shown if shown => {
                    this.summary = None;
                    this.found = None;
                    this.reload(cx);
                }
                _ => {}
            }
            let _ = library;
        })];
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let alive = this.update(cx, |this, cx| {
                    if this.mode(cx).is_some() {
                        this.reload(cx);
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let today = crate::journal::now(cx).date;
        DigestView {
            library,
            editor,
            store,
            loops: Vec::new(),
            loaded: false,
            brief: None,
            review: None,
            week_start: brief::week_start(today),
            summary: None,
            found: None,
            working: None,
            cancel: Arc::new(AtomicBool::new(false)),
            ai_task: None,
            focus: cx.focus_handle(),
            list_scroll: VirtualListScrollHandle::new(),
            page_scroll: ScrollHandle::new(),
            leading_inset: px(0.),
            stale: true,
            reload_task: None,
            _subscriptions: subscriptions,
            _tick: tick,
        }
    }

    fn mode(&self, cx: &App) -> Option<Mode> {
        match self.library.read(cx).filter {
            Filter::OpenLoops => Some(Mode::Loops),
            Filter::Brief => Some(Mode::Brief),
            Filter::Review => Some(Mode::Review),
            _ => None,
        }
    }

    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    /// Works the page out again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let Some(mode) = self.mode(cx) else {
            return;
        };
        let store = self.store.clone();
        let now = now_of(cx);
        let week = self.week_start;
        let model = store.embedding_model().ok().flatten();
        let task = cx.background_executor().spawn(async move {
            Ok::<_, kanso_store::StoreError>(match mode {
                Mode::Loops => (
                    Some(store.open_loops(&LoopQuery::default())?),
                    None,
                    None,
                ),
                Mode::Brief => (None, Some(morning_brief(&store, now, model.as_deref())?), None),
                Mode::Review => (None, None, Some(weekly_review(&store, week, now)?)),
            })
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((loops, brief, review)) => {
                        if let Some(loops) = loops {
                            this.loops = loops;
                        }
                        if brief.is_some() {
                            this.brief = brief;
                        }
                        if review.is_some() {
                            this.review = review;
                        }
                        this.loaded = true;
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

    // MARK: Actions

    fn open(&mut self, note_id: &str, line: u32, focus: bool, cx: &mut Context<Self>) {
        cx.emit(DigestEvent::Open {
            note_id: note_id.to_owned(),
            line,
            focus,
        });
    }

    /// Closes loop `row` as `how`: the note's unsaved typing is saved first,
    /// so the write goes on top of it.
    fn close(&mut self, row: &LoopRow, how: LoopClose, cx: &mut Context<Self>) {
        if writes_held(cx) {
            cx.emit(DigestEvent::Notice(
                "Loops can't be changed while the library is being replaced.".into(),
            ));
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let outcome = self.store.close_loop(
            &row.note_id,
            &row.key,
            row.line,
            how,
            Clock::now_ms(cx),
            Origin::Ui,
        );
        match outcome {
            Ok(CheckOff::Missing) => {
                cx.emit(DigestEvent::Notice(
                    "That sentence changed in its note. The list is up to date now.".into(),
                ));
            }
            Ok(_) => {}
            Err(err) => cx.emit(DigestEvent::Notice(format!(
                "Couldn't change the loop: {err}"
            ))),
        }
        // Gone at once; the reload confirms it.
        self.loops.retain(|l| !(l.note_id == row.note_id && l.key == row.key));
        self.reload(cx);
        cx.notify();
    }

    fn mark_in_note(cx: &App) -> bool {
        cx.try_global::<AppState>()
            .is_some_and(|state| state.config.briefs.mark_done_in_note)
    }

    /// Ticks task `row` off from the brief.
    fn tick_task(&mut self, row: &TaskRow, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        match self.store.set_task_checked(
            &row.note_id,
            row.line,
            &row.text,
            true,
            Clock::now_ms(cx),
            Origin::Ui,
        ) {
            Ok(CheckOff::Missing) => cx.emit(DigestEvent::Notice(
                "That task changed in its note. The page is up to date now.".into(),
            )),
            Ok(_) => crate::metrics::record(cx, crate::metrics::Feature::TaskToggled),
            Err(err) => cx.emit(DigestEvent::Notice(format!(
                "Couldn't change the task: {err}"
            ))),
        }
        self.reload(cx);
    }

    fn step_week(&mut self, weeks: i64, cx: &mut Context<Self>) {
        self.week_start = self.week_start.add_days(7 * weeks);
        self.summary = None;
        self.reload(cx);
    }

    fn this_week(&mut self, cx: &mut Context<Self>) {
        self.week_start = brief::week_start(crate::journal::now(cx).date);
        self.summary = None;
        self.reload(cx);
    }

    /// Saves the reviewed week as a note in the Journal, asking first when
    /// that week is saved already.
    fn save_review(&mut self, replace: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(review) = self.review.clone() else {
            return;
        };
        match brief::save_review(&self.store, &review, replace, Origin::Ui) {
            Ok(brief::Saved::Written { id }) => {
                cx.emit(DigestEvent::Open {
                    note_id: id,
                    line: 0,
                    focus: false,
                });
            }
            Ok(brief::Saved::Exists { .. }) => {
                let label = brief::iso_week_label(review.from);
                let view = cx.entity();
                crate::dialogs::confirm(
                    format!("Replace the saved review for {label}?"),
                    "The saved note is ordinary text. Replacing it writes this page over it.",
                    "Replace",
                    window,
                    cx,
                    move |window, cx| {
                        view.update(cx, |this, cx| this.save_review(true, window, cx));
                    },
                );
            }
            Err(err) => cx.emit(DigestEvent::Notice(format!(
                "Couldn't save the review: {err}"
            ))),
        }
    }

    // MARK: AI

    fn ai_on(cx: &App) -> bool {
        cx.try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled)
    }

    /// Asks before reading the notes of the last 14 days with the model.
    fn ask_find_more(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ai = cx.global::<AppState>().config.ai.clone();
        let via = match ai.backend {
            crate::config::AiBackend::Api => ai.model.clone(),
            crate::config::AiBackend::Agent => "your agent".to_owned(),
        };
        let view = cx.entity();
        crate::dialogs::confirm(
            "Find more open loops?",
            format!(
                "Read your notes from the last {} days with {via} to find promises the rules missed? Locked notes and notes tagged #ai-exclude are left out.",
                loops_ai::DEFAULT_DAYS
            ),
            "Find Loops",
            window,
            cx,
            move |_, cx| view.update(cx, |this, cx| this.find_more(cx)),
        );
    }

    fn find_more(&mut self, cx: &mut Context<Self>) {
        if self.working.is_some() {
            return;
        }
        self.working = Some("Reading your notes…");
        self.cancel.store(false, Ordering::SeqCst);
        let cancel = self.cancel.clone();
        let maker = crate::ai_feature::maker(cx);
        let store = self.store.clone();
        let now = Clock::now_ms(cx);
        let task = cx.background_executor().spawn(async move {
            let shared = maker().map_err(|e| e.to_string())?;
            let provider = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            loops_ai::find_more(&store, provider.as_ref(), loops_ai::DEFAULT_DAYS, now, &|| {
                cancel.load(Ordering::SeqCst)
            })
            .map_err(|e| e.to_string())
        });
        self.ai_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.working = None;
                match result {
                    Ok(scan) if scan.proposals.is_empty() => {
                        let mut message = "No more open loops found.".to_owned();
                        if let Some(extra) = scan.dropped_text() {
                            message = format!("{message} {extra}.");
                        }
                        cx.emit(DigestEvent::Notice(message));
                    }
                    Ok(scan) => {
                        let checked = vec![false; scan.proposals.len()];
                        this.found = Some(Found { scan, checked });
                    }
                    Err(message) => cx.emit(DigestEvent::Notice(message)),
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Stores the loops ticked in the sheet.
    fn keep_selected(&mut self, cx: &mut Context<Self>) {
        let Some(found) = self.found.take() else {
            return;
        };
        let now = Clock::now_ms(cx);
        let mut kept = 0;
        for (proposal, _) in found
            .scan
            .proposals
            .iter()
            .zip(&found.checked)
            .filter(|(_, on)| **on)
        {
            let ai = AiLoop {
                key: String::new(),
                text: proposal.text.clone(),
                kind: proposal.kind,
                approved_at: now,
            };
            match self
                .store
                .approve_ai_loops(&proposal.note_id, &[ai], Origin::Ui)
            {
                Ok(n) => kept += n,
                Err(err) => cx.emit(DigestEvent::Notice(format!("Couldn't keep a loop: {err}"))),
            }
        }
        if kept > 0 {
            cx.emit(DigestEvent::Notice(format!(
                "Kept {kept} open {}.",
                if kept == 1 { "loop" } else { "loops" }
            )));
        }
        self.reload(cx);
        cx.notify();
    }

    /// Writes the paragraph over the brief (`brief` true) or the review's
    /// themes, from what is on the page, when asked.
    fn summarise(&mut self, for_brief: bool, cx: &mut Context<Self>) {
        if self.working.is_some() {
            return;
        }
        let (prompt, allowed) = if for_brief {
            match &self.brief {
                Some(brief) => loops_ai::brief_prompt(&self.store, brief),
                None => return,
            }
        } else {
            match &self.review {
                Some(review) => loops_ai::themes_prompt(&self.store, review),
                None => return,
            }
        };
        self.working = Some("Writing…");
        let maker = crate::ai_feature::maker(cx);
        let task = cx.background_executor().spawn(async move {
            let shared = maker()?;
            let provider = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            loops_ai::summarise(provider.as_ref(), &prompt, &allowed).map_err(|e| e.to_string())
        });
        self.ai_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.working = None;
                match result {
                    Ok(summary) => this.summary = Some(summary),
                    Err(message) => cx.emit(DigestEvent::Notice(message)),
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    // MARK: Drawing: shared

    fn top_band(
        &self,
        title: &str,
        detail: Option<String>,
        actions: Vec<AnyElement>,
        cx: &App,
    ) -> impl IntoElement {
        let colors = Colors::get(cx);
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_shrink_0()
            .pl(px(16.) + self.leading_inset)
            .pr(px(10.))
            .gap_1p5()
            .child(
                div()
                    .text_size(px(15.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(title.to_owned()),
            )
            .when_some(detail, |el, detail| {
                el.child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(detail),
                )
            })
            .child(div().flex_1())
            .children(actions)
    }

    fn heading(&self, text: impl Into<SharedString>, cx: &App) -> Div {
        let colors = Colors::get(cx);
        section_label(div(), colors)
            .px(px(ROW_INSET_X + ROW_PAD_X))
            .pt(px(14.))
            .pb(px(4.))
            .child(text.into())
    }

    /// A line of the brief or review: an icon, words, a muted second line.
    fn page_row(
        &self,
        id: impl Into<ElementId>,
        icon: Icon,
        title: String,
        detail: Option<String>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = Colors::get(cx);
        div()
            .id(id)
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(1.))
            .cursor_pointer()
            .child(
                h_flex()
                    .w_full()
                    .px(px(ROW_PAD_X))
                    .py(px(6.))
                    .gap(px(10.))
                    .items_start()
                    .rounded(px(7.))
                    .hover(|el| el.bg(colors.row_hover))
                    .child(icon.size(px(15.)).text_color(colors.text_muted).mt(px(2.)))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .line_height(px(18.))
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .when_some(detail, |el, detail| {
                                el.child(
                                    div()
                                        .text_size(px(12.))
                                        .line_height(px(16.))
                                        .text_color(colors.text_muted)
                                        .child(detail),
                                )
                            }),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| on_click(this, window, cx)))
    }

    fn icon_button(
        id: impl Into<ElementId>,
        icon: Lucide,
        tooltip: &'static str,
        cx: &App,
    ) -> Button {
        let colors = Colors::get(cx);
        Button::new(id)
            .ghost()
            .small()
            .icon(Icon::new(icon).size(px(15.)).text_color(colors.text_secondary))
            .tooltip(tooltip)
    }

    fn ai_button(&self, id: &'static str, label: &'static str, cx: &mut Context<Self>) -> Button {
        let colors = Colors::get(cx);
        Button::new(id)
            .ghost()
            .small()
            .icon(
                Icon::new(Lucide::Sparkles)
                    .size(px(14.))
                    .text_color(colors.text_secondary),
            )
            .label(label)
            .disabled(self.working.is_some())
    }

    fn working_line(&self, cx: &App) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        self.working.map(|what| {
            div()
                .px(px(ROW_INSET_X + ROW_PAD_X))
                .py(px(6.))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(what)
                .into_any_element()
        })
    }

    fn summary_block(&self, what: &'static str, cx: &App) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let summary = self.summary.as_ref()?;
        Some(
            v_flex()
                .mx(px(ROW_INSET_X))
                .my(px(6.))
                .p(px(10.))
                .gap(px(6.))
                .rounded(px(8.))
                .bg(colors.row_hover)
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text)
                        .child(summary.text.clone()),
                )
                .when_some(summary.warning(what), |el, warning| {
                    el.child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.danger)
                            .child(warning),
                    )
                })
                .into_any_element(),
        )
    }

    // MARK: Drawing: open loops

    fn loop_row(&self, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let row = self.loops[ix].clone();
        let library = self.library.read(cx);
        let book = library
            .book(&row.book_id)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        let title = if row.note_title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            row.note_title.clone()
        };
        let age = age_label(row.note_updated_at, Clock::now_ms(cx));
        let mut second = title;
        if !book.is_empty() {
            second = format!("{second} · {book}");
        }
        second = format!("{second} · {age}");
        // The matched phrase in bold.
        let phrase = phrase_end(&row);
        let (bold, rest) = row.text.split_at(phrase);
        let group = SharedString::from(format!("loop-{}-{}", row.note_id, row.key));
        let open_row = row.clone();
        let menu_row = row.clone();
        let view = cx.entity();
        let action = |id: String, icon: Lucide, tip: &'static str| {
            Self::icon_button(SharedString::from(id), icon, tip, cx)
        };
        let done = {
            let row = row.clone();
            action(format!("loop-done-{}", row.key), Lucide::Check, "Done").on_click(
                cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    let mark = Self::mark_in_note(cx);
                    this.close(&row, LoopClose::Done { mark_in_note: mark }, cx);
                }),
            )
        };
        let make_task = {
            let row = row.clone();
            action(format!("loop-task-{}", row.key), Lucide::ListPlus, "Make Task").on_click(
                cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.close(&row, LoopClose::MakeTask, cx);
                }),
            )
        };
        let dismiss = {
            let row = row.clone();
            action(format!("loop-dismiss-{}", row.key), Lucide::X, "Not a loop").on_click(
                cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.close(&row, LoopClose::Dismissed, cx);
                }),
            )
        };
        div()
            .id(SharedString::from(format!("loop-{}-{}", row.note_id, row.key)))
            .group(group.clone())
            .h(px(ROW_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(1.))
            .child(
                h_flex()
                    .size_full()
                    .px(px(ROW_PAD_X))
                    .gap(px(8.))
                    .items_start()
                    .pt(px(8.))
                    .rounded(px(7.))
                    .hover(|el| el.bg(colors.row_hover))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                h_flex()
                                    .truncate()
                                    .text_size(px(13.5))
                                    .line_height(px(18.))
                                    .text_color(colors.text)
                                    .child(div().font_semibold().child(bold.to_owned()))
                                    .child(div().child(rest.to_owned())),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.))
                                    .line_height(px(16.))
                                    .text_color(colors.text_muted)
                                    .child(second),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .opacity(0.)
                            .group_hover(group, |style| style.opacity(1.))
                            .child(done)
                            .child(make_task)
                            .child(dismiss),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                this.open(&open_row.note_id, open_row.line, false, cx);
            }))
            .context_menu(move |menu, _, cx| {
                let mark = Self::mark_in_note(cx);
                let item = |label: &'static str, how: LoopClose| {
                    let view = view.clone();
                    let row = menu_row.clone();
                    PopupMenuItem::new(label).on_click(move |_, _, cx| {
                        let row = row.clone();
                        view.update(cx, |this, cx| this.close(&row, how, cx));
                    })
                };
                menu.item(item("Done", LoopClose::Done { mark_in_note: mark }))
                    .item(item("Make Task", LoopClose::MakeTask))
                    .item(item("Not a Loop", LoopClose::Dismissed))
            })
            .into_any_element()
    }

    fn loop_header(&self, kind: LoopKind, count: usize, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        h_flex()
            .h(px(HEADER_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X + ROW_PAD_X))
            .pt(px(8.))
            .gap(px(6.))
            .child(section_label(div(), colors).child(group_label(kind)))
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(count.to_string()),
            )
            .into_any_element()
    }

    fn render_loops(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let list = entries(&self.loops);
        let ai = Self::ai_on(cx);
        let mut actions: Vec<AnyElement> = Vec::new();
        if ai {
            actions.push(
                self.ai_button("find-loops", "Find more…", cx)
                    .tooltip("Read recent notes with your model to find promises the rules missed")
                    .on_click(cx.listener(|this, _, window, cx| this.ask_find_more(window, cx)))
                    .into_any_element(),
            );
        }
        let off = !cx
            .global::<AppState>()
            .config
            .briefs
            .loops;
        let body = if let Some(found) = &self.found {
            self.found_sheet(found, cx)
        } else if off {
            quiet_empty(
                Lucide::CircleDashed,
                "Open loops are off".into(),
                Some("Turn them on in Settings › General.".into()),
                colors,
            )
            .into_any_element()
        } else if list.is_empty() {
            if self.loaded {
                quiet_empty(
                    Lucide::CircleDashed,
                    "No open loops".into(),
                    Some(
                        "When you write things like “I'll send the deck” or “need to check the lease”, they show up here."
                            .into(),
                    ),
                    colors,
                )
                .into_any_element()
            } else {
                div().into_any_element()
            }
        } else {
            let sizes = Rc::new(
                list.iter()
                    .map(|entry| {
                        size(
                            px(0.),
                            px(match entry {
                                Entry::Header { .. } => HEADER_HEIGHT,
                                Entry::Loop(_) => ROW_HEIGHT,
                            }),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let shown = Rc::new(list);
            v_virtual_list(cx.entity(), "loops", sizes, move |this, range, _, cx| {
                range
                    .filter_map(|at| {
                        Some(match shown.get(at)? {
                            Entry::Header { kind, count } => this.loop_header(*kind, *count, cx),
                            Entry::Loop(ix) => this.loop_row(*ix, cx),
                        })
                    })
                    .collect()
            })
            .size_full()
            .track_scroll(&self.list_scroll)
            .into_any_element()
        };
        let _ = window;
        v_flex()
            .size_full()
            .bg(colors.list)
            .child(self.top_band(
                "Open Loops",
                Some(self.loops.len().to_string()),
                actions,
                cx,
            ))
            .children(self.working_line(cx))
            .child(div().flex_1().min_h_0().child(body))
            .into_any_element()
    }

    /// The sheet of loops the model proposed: each with its source line and
    /// a box, none ticked.
    fn found_sheet(&self, found: &Found, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let mut rows = v_flex().w_full().gap(px(2.));
        for (ix, proposal) in found.scan.proposals.iter().enumerate() {
            let on = found.checked[ix];
            let Proposal {
                note_title, text, ..
            } = proposal;
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("found-{ix}")))
                    .w_full()
                    .px(px(ROW_INSET_X))
                    .cursor_pointer()
                    .child(
                        h_flex()
                            .w_full()
                            .px(px(ROW_PAD_X))
                            .py(px(6.))
                            .gap(px(10.))
                            .items_start()
                            .rounded(px(7.))
                            .hover(|el| el.bg(colors.row_hover))
                            .child(
                                Icon::new(if on {
                                    Lucide::SquareCheck
                                } else {
                                    Lucide::Square
                                })
                                .size(px(16.))
                                .text_color(if on { colors.accent } else { colors.text_muted }),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(13.5))
                                            .text_color(colors.text)
                                            .child(text.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(colors.text_muted)
                                            .child(format!(
                                                "{note_title} · line {}",
                                                proposal.line + 1
                                            )),
                                    ),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(found) = &mut this.found {
                            found.checked[ix] = !found.checked[ix];
                        }
                        cx.notify();
                    })),
            );
        }
        let notes = [found.scan.coverage(), found.scan.dropped_text()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        let any = found.checked.iter().any(|c| *c);
        v_flex()
            .size_full()
            .child(
                div()
                    .px(px(ROW_INSET_X + ROW_PAD_X))
                    .py(px(8.))
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child("Choose the ones that are real. Nothing is kept until you do."),
            )
            .child(
                div()
                    .id("found-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(rows),
            )
            .child(
                h_flex()
                    .flex_shrink_0()
                    .px(px(12.))
                    .py(px(8.))
                    .gap_2()
                    .border_t_1()
                    .border_color(colors.divider)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(notes),
                    )
                    .child(
                        Button::new("found-cancel")
                            .ghost()
                            .small()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.found = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("found-keep")
                            .primary()
                            .small()
                            .label("Keep Selected")
                            .disabled(!any)
                            .on_click(cx.listener(|this, _, _, cx| this.keep_selected(cx))),
                    ),
            )
            .into_any_element()
    }

    // MARK: Drawing: morning brief

    fn task_line(&self, row: &TaskRow, today: Date, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let title = crate::tasks_view::display_text(&row.text);
        let due = row
            .due
            .as_deref()
            .and_then(kanso_core::tasks::Due::parse)
            .map(|due| kanso_core::tasks::due_label(due, today));
        let sub = match due {
            Some(due) => format!("{due} · {}", row.note_title),
            None => row.note_title.clone(),
        };
        let tick_row = row.clone();
        let open = row.clone();
        h_flex()
            .id(SharedString::from(format!("due-{}-{}", row.note_id, row.line)))
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(1.))
            .child(
                h_flex()
                    .w_full()
                    .px(px(ROW_PAD_X))
                    .py(px(6.))
                    .gap(px(10.))
                    .items_start()
                    .rounded(px(7.))
                    .hover(|el| el.bg(colors.row_hover))
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "due-tick-{}-{}",
                                row.note_id, row.line
                            )))
                            .flex_none()
                            .pt(px(1.))
                            .cursor_pointer()
                            .child(
                                Icon::new(Lucide::Circle)
                                    .size(px(16.))
                                    .text_color(colors.text_muted),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.tick_task(&tick_row, cx);
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .line_height(px(18.))
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .line_height(px(16.))
                                    .text_color(colors.text_muted)
                                    .child(sub),
                            ),
                    ),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open(&open.note_id, open.line, false, cx)
            }))
            .into_any_element()
    }

    fn render_brief(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let now = now_of(cx);
        let today = now.today();
        let ai = Self::ai_on(cx);
        let mut actions: Vec<AnyElement> = Vec::new();
        if ai && self.brief.as_ref().is_some_and(|b| !b.is_empty()) {
            actions.push(
                self.ai_button("brief-summarise", "Summarise", cx)
                    .tooltip("Write a short paragraph from the items below, with your model")
                    .on_click(cx.listener(|this, _, _, cx| this.summarise(true, cx)))
                    .into_any_element(),
            );
        }
        let mut page = v_flex().w_full().pb(px(16.));
        if let Some(brief) = self.brief.clone() {
            page = page.children(self.working_line(cx));
            page = page.children(self.summary_block("brief", cx));
            if brief.is_empty() && brief.relevant.is_empty() {
                page = page.child(
                    v_flex()
                        .items_center()
                        .gap(px(12.))
                        .pt(px(60.))
                        .child(
                            Icon::new(Lucide::Sunrise)
                                .size(px(22.))
                                .text_color(colors.text_muted),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .text_color(colors.text_secondary)
                                .child("Nothing due and no open loops. Have a good day."),
                        )
                        .child(
                            Button::new("brief-open-today")
                                .outline()
                                .small()
                                .label("Open Today's Note")
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(DigestEvent::OpenToday))),
                        ),
                );
            } else {
                if !brief.due.is_empty() {
                    page = page.child(self.heading("Due today and overdue", cx));
                    for row in &brief.due {
                        page = page.child(self.task_line(row, today, cx));
                    }
                }
                if !brief.loops.is_empty() {
                    page = page.child(self.heading("Open loops", cx));
                    for l in &brief.loops {
                        let stale = kanso_core::loops::age_days(l.note_updated_at, now.utc_ms)
                            >= brief::STALE_DAYS;
                        let sub = if stale {
                            format!("Still open from last week · {}", l.note_title)
                        } else {
                            l.note_title.clone()
                        };
                        let open = l.clone();
                        page = page.child(self.page_row(
                            SharedString::from(format!("brief-loop-{}", l.key)),
                            Icon::new(Lucide::CircleDashed),
                            l.text.clone(),
                            Some(sub),
                            move |this, _, cx| this.open(&open.note_id, open.line, false, cx),
                            cx,
                        ));
                    }
                    if brief.loops_total as usize > brief.loops.len() {
                        let total = brief.loops_total;
                        page = page.child(self.page_row(
                            "brief-see-all",
                            Icon::new(Lucide::ArrowRight),
                            format!("See all ({total})"),
                            None,
                            |this, _, cx| {
                                this.library
                                    .update(cx, |library, cx| library.set_filter(Filter::OpenLoops, cx));
                            },
                            cx,
                        ));
                    }
                }
                if !brief.later_today.is_empty() {
                    page = page.child(self.heading("Today's reminders", cx));
                    for t in &brief.later_today {
                        let time = t.due.get(11..16).unwrap_or("").to_owned();
                        let open = t.clone();
                        page = page.child(self.page_row(
                            SharedString::from(format!("brief-reminder-{}", t.key)),
                            Icon::new(Lucide::Bell),
                            crate::tasks_view::display_text(&t.text),
                            Some(format!("{time} · {}", t.note_title)),
                            move |this, _, cx| this.open(&open.note_id, open.line, false, cx),
                            cx,
                        ));
                    }
                }
                if !brief.relevant.is_empty() {
                    page = page.child(self.heading("From your notes", cx));
                    for r in &brief.relevant {
                        let id = r.id.clone();
                        let when = Clock::local_ms(cx, r.updated_at);
                        let date = DateTime::from_local_ms(when).date.long_label();
                        let sub = format!("{} · {date}", r.why.label());
                        page = page.child(
                            self.page_row(
                                SharedString::from(format!("brief-note-{}", r.id)),
                                Icon::new(match r.why {
                                    Why::Similar => Lucide::Sparkles,
                                    _ => Lucide::FileText,
                                }),
                                r.title.clone(),
                                Some(if r.excerpt.is_empty() {
                                    sub
                                } else {
                                    format!("{sub}\n{}", r.excerpt)
                                }),
                                move |this, _, cx| this.open(&id, 0, false, cx),
                                cx,
                            ),
                        );
                    }
                }
            }
            let y = brief.yesterday;
            page = page.child(
                div()
                    .px(px(ROW_INSET_X + ROW_PAD_X))
                    .pt(px(18.))
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(format!(
                        "Yesterday you wrote {} and finished {}.",
                        plural(y.wrote, "note", "notes"),
                        plural(y.finished, "task", "tasks")
                    )),
            );
        }
        v_flex()
            .size_full()
            .bg(colors.list)
            .child(self.top_band("Morning Brief", Some(today.long_label()), actions, cx))
            .child(
                div()
                    .id("brief-page")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.page_scroll)
                    .child(page),
            )
            .into_any_element()
    }

    // MARK: Drawing: weekly review

    fn render_review(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let now = now_of(cx);
        let today = now.today();
        let ai = Self::ai_on(cx);
        let from = self.week_start;
        let label = format!(
            "{} · {}",
            brief::iso_week_label(from),
            DateTime::at_midnight(from).date.long_label()
        );
        let mut actions: Vec<AnyElement> = vec![
            Self::icon_button("review-prev", Lucide::ChevronLeft, "Previous week", cx)
                .on_click(cx.listener(|this, _, _, cx| this.step_week(-1, cx)))
                .into_any_element(),
            Self::icon_button("review-next", Lucide::ChevronRight, "Next week", cx)
                .on_click(cx.listener(|this, _, _, cx| this.step_week(1, cx)))
                .into_any_element(),
            Button::new("review-today")
                .ghost()
                .small()
                .label("This Week")
                .on_click(cx.listener(|this, _, _, cx| this.this_week(cx)))
                .into_any_element(),
        ];
        if ai {
            actions.push(
                self.ai_button("review-themes", "Describe themes", cx)
                    .tooltip("Describe the week's themes in a few sentences, with your model")
                    .on_click(cx.listener(|this, _, _, cx| this.summarise(false, cx)))
                    .into_any_element(),
            );
        }
        actions.push(
            Button::new("review-save")
                .outline()
                .small()
                .label("Save as Note")
                .tooltip("Keep this page as a note in the Journal, with links to every note on it")
                .on_click(cx.listener(|this, _, window, cx| this.save_review(false, window, cx)))
                .into_any_element(),
        );
        let mut page = v_flex().w_full().pb(px(16.));
        if let Some(review) = self.review.clone() {
            page = page.children(self.working_line(cx));
            page = page.children(self.summary_block("review", cx));
            page = page.child(self.heading(
                format!("What you wrote · {}", review.counts_text()),
                cx,
            ));
            let mut by_book: Vec<(String, Vec<&brief::WrittenNote>)> = Vec::new();
            for note in &review.written {
                match by_book.iter_mut().find(|(b, _)| *b == note.book_name) {
                    Some((_, notes)) => notes.push(note),
                    None => by_book.push((note.book_name.clone(), vec![note])),
                }
            }
            if review.written.is_empty() {
                page = page.child(self.muted_line("Nothing this week.", cx));
            }
            for (book, notes) in by_book {
                let name = if book.is_empty() { "No notebook".to_owned() } else { book };
                page = page.child(self.muted_line(&name, cx));
                for note in notes {
                    let id = note.id.clone();
                    page = page.child(self.page_row(
                        SharedString::from(format!("review-wrote-{}", note.id)),
                        Icon::new(Lucide::FileText),
                        if note.title.trim().is_empty() {
                            "Untitled".to_owned()
                        } else {
                            note.title.clone()
                        },
                        Some(if note.created { "New".to_owned() } else { "Changed".to_owned() }),
                        move |this, _, cx| this.open(&id, 0, false, cx),
                        cx,
                    ));
                }
            }
            page = page.child(self.heading("What you finished", cx));
            if review.finished.is_empty() {
                page = page.child(self.muted_line("Nothing ticked off.", cx));
            }
            for f in &review.finished {
                let id = f.note_id.clone();
                page = page.child(self.page_row(
                    SharedString::from(format!("review-done-{}-{}", f.note_id, f.key)),
                    Icon::new(Lucide::CircleCheck),
                    crate::tasks_view::display_text(&f.text),
                    Some(f.note_title.clone()),
                    move |this, _, cx| this.open(&id, 0, false, cx),
                    cx,
                ));
            }
            page = page.child(self.heading("Still open", cx));
            if review.still_open.is_empty() && review.overdue.is_empty() {
                page = page.child(self.muted_line("Nothing open.", cx));
            }
            for t in &review.overdue {
                page = page.child(self.task_line(t, today, cx));
            }
            for l in review.still_open.iter().take(40) {
                let open = l.clone();
                page = page.child(self.page_row(
                    SharedString::from(format!("review-open-{}", l.key)),
                    Icon::new(Lucide::CircleDashed),
                    l.text.clone(),
                    Some(format!(
                        "{} · {}",
                        l.note_title,
                        age_label(l.note_updated_at, now.utc_ms)
                    )),
                    move |this, _, cx| this.open(&open.note_id, open.line, false, cx),
                    cx,
                ));
            }
            if review.still_open.len() > 40 {
                page = page.child(self.muted_line(
                    &format!("and {} more in Open Loops", review.still_open.len() - 40),
                    cx,
                ));
            }
            if !review.themes.is_empty() {
                page = page.child(self.heading("Recurring themes", cx));
                for theme in &review.themes {
                    let (icon, text, query) = match theme.kind {
                        ThemeKind::Tag => (
                            Lucide::Hash,
                            format!("#{}", theme.term),
                            format!("tag:{}", theme.term),
                        ),
                        _ => (Lucide::Search, theme.term.clone(), theme.term.clone()),
                    };
                    page = page.child(self.page_row(
                        SharedString::from(format!("review-theme-{}", theme.term)),
                        Icon::new(icon),
                        text,
                        Some(format!("{} notes", theme.notes)),
                        move |this, _, cx| {
                            let query = query.clone();
                            this.library.update(cx, |library, cx| {
                                library.set_filter(Filter::All, cx);
                                library.set_search(query, cx);
                            });
                        },
                        cx,
                    ));
                }
            }
            page = page.child(self.heading("Next week", cx));
            if review.next.is_empty() {
                page = page.child(self.muted_line("Nothing due.", cx));
            }
            for t in &review.next {
                page = page.child(self.task_line(t, today, cx));
            }
        }
        v_flex()
            .size_full()
            .bg(colors.list)
            .child(self.top_band("Weekly Review", None, actions, cx))
            .child(
                div()
                    .px(px(16.) + self.leading_inset)
                    .pb(px(4.))
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(label),
            )
            .child(
                div()
                    .id("review-page")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.page_scroll)
                    .child(page),
            )
            .into_any_element()
    }

    fn muted_line(&self, text: &str, cx: &App) -> Div {
        let colors = Colors::get(cx);
        div()
            .px(px(ROW_INSET_X + ROW_PAD_X))
            .py(px(3.))
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .child(text.to_owned())
    }
}

fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Where the matched phrase of a loop ends in its sentence: its first
/// words, up to the end of the "I'll", "need to", "waiting on" that made
/// it one.
fn phrase_end(row: &LoopRow) -> usize {
    let items = kanso_core::loops::loop_items(&row.text);
    let end = items
        .first()
        .filter(|item| item.text == row.text)
        .map_or(0, |item| item.span.1);
    let mut end = end.min(row.text.len());
    while !row.text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

impl Render for DigestView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let page = match self.mode(cx) {
            Some(Mode::Loops) => self.render_loops(window, cx),
            Some(Mode::Brief) => self.render_brief(cx),
            Some(Mode::Review) => self.render_review(cx),
            None => div().into_any_element(),
        };
        div()
            .size_full()
            .bg(colors.list)
            .key_context("DigestView")
            .track_focus(&self.focus)
            .child(page)
    }
}
