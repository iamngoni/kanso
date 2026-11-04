//! Open loops and Overview, in the note list's place (`Filter::OpenLoops`,
//! `Filter::Overview`).
//!
//! Overview (issue #240) is Kanso's home, with Today and This week: see
//! [`overview_view`]. Everything shown is computed when it opens and again
//! when the library changes (`kanso_engine::overview`), and nothing is sent
//! to a model unless one of the explicit AI buttons is pressed. The parts
//! that decide are the engine's; this draws them and turns clicks into store
//! writes.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{
    Disableable as _, Icon, Sizable as _, StyledExt as _, VirtualListScrollHandle, h_flex, v_flex,
    v_virtual_list,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::Date;
use kanso_core::loops::{AiLoop, LoopKind, age_label};
use kanso_engine::brief::{self, Now};
use kanso_engine::loops_ai::{self, Proposal, Scan};
use kanso_engine::overview::Overview;
use kanso_store::{CheckOff, LoopClose, LoopQuery, LoopRow, Origin, Store};

use crate::clock::Clock;
use crate::library::{Filter, Library, LibraryEvent};
use crate::page_chrome::{self, HeaderAction};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

#[path = "overview_view.rs"]
mod overview_view;
use overview_view::{AiPara, OverviewParts, Undo};

const ROW_HEIGHT: f32 = 54.;
const HEADER_HEIGHT: f32 = 32.;
const ROW_PAD_X: f32 = 10.;
/// How long changes to notes settle before the page is worked out again.
const RELOAD_DELAY: Duration = Duration::from_millis(500);
/// How often the page is worked out again (a task falls due as time passes).
const TICK: Duration = Duration::from_secs(60);

actions!(
    digest,
    [
        OpenOverview,
        OpenOverviewToday,
        OpenOverviewWeek,
        SummariseOverview,
        SaveWeekAsNote,
        OpenMorningBrief,
        OpenWeeklyReview,
        OpenOpenLoops,
        FindMoreOpenLoops
    ]
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
    let overview = show(Filter::Overview);
    let today = show(Filter::Overview);
    let week = show(Filter::Overview);
    let brief = show(Filter::Overview);
    let review = show(Filter::Overview);
    let loops = show(Filter::OpenLoops);
    let loops_again = show(Filter::OpenLoops);
    el.on_action(cx.listener(move |this, _: &OpenOverview, window, cx| overview(this, window, cx)))
        .on_action(cx.listener(move |this, _: &OpenOverviewToday, window, cx| {
            today(this, window, cx);
            this.digest.update(cx, |digest, cx| {
                digest.show_perspective(Perspective::Today, cx)
            });
        }))
        .on_action(cx.listener(move |this, _: &OpenOverviewWeek, window, cx| {
            week(this, window, cx);
            this.digest.update(cx, |digest, cx| {
                digest.show_perspective(Perspective::Week, cx)
            });
        }))
        .on_action(cx.listener(move |this, _: &OpenMorningBrief, window, cx| {
            brief(this, window, cx);
            this.digest.update(cx, |digest, cx| {
                digest.show_perspective(Perspective::Today, cx)
            });
        }))
        .on_action(cx.listener(move |this, _: &OpenWeeklyReview, window, cx| {
            review(this, window, cx);
            this.digest.update(cx, |digest, cx| {
                digest.show_perspective(Perspective::Week, cx)
            });
        }))
        .on_action(cx.listener(|this, _: &SummariseOverview, _, cx| {
            this.digest
                .update(cx, |digest, cx| digest.summarise_overview(cx));
        }))
        .on_action(cx.listener(|this, _: &SaveWeekAsNote, window, cx| {
            this.digest.update(cx, |digest, cx| {
                digest.show_perspective(Perspective::Week, cx);
                digest.save_week(false, window, cx);
            });
        }))
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
    /// Escape: back to the place before the page.
    Close,
    /// A link in the page's text was clicked.
    Link(String),
    /// Open Settings at Overview's section.
    OpenSettings,
    /// Something to tell the user.
    Notice(String),
}

/// Which page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Loops,
    Overview,
}

/// Which perspective Overview shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Perspective {
    #[default]
    Today,
    Week,
}

/// An AI action a page can run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AiJob {
    FindMore,
    Summary { brief: bool },
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
    overview: Option<Overview>,
    week: Option<kanso_engine::overview::Week>,
    perspective: Perspective,
    week_start: Date,
    /// The model's paragraph over what shows, kept for the day or the week.
    ai_para: Option<AiPara>,
    ai_hidden: bool,
    ai_more: bool,
    /// Sections opened past their cap, by key.
    expanded: std::collections::HashSet<String>,
    undo: Option<Undo>,
    undo_task: Option<Task<()>>,
    parts: OverviewParts,
    found: Option<Found>,
    /// What the AI is doing now, if anything.
    working: Option<AiJob>,
    /// The last AI action that failed, shown where it was asked for.
    ai_error: Option<(AiJob, page_chrome::AiTrouble)>,
    /// The page's width, for fitting the header.
    pane_width: Pixels,
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
                    this.load_snapshot(cx);
                    this.ai_hidden = false;
                    this.ai_more = false;
                    this.ai_error = None;
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
            store: store.clone(),
            loops: Vec::new(),
            loaded: false,
            overview: None,
            week: None,
            perspective: Perspective::Today,
            week_start: brief::week_start(today),
            ai_para: None,
            ai_hidden: false,
            ai_more: false,
            expanded: Default::default(),
            undo: None,
            undo_task: None,
            parts: OverviewParts::new(store.clone()),
            found: None,
            working: None,
            ai_error: None,
            pane_width: px(1000.),
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
            Filter::Overview => Some(Mode::Overview),
            _ => None,
        }
    }

    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    /// The width the page has, for fitting its header.
    pub fn set_pane_width(&mut self, width: Pixels, cx: &mut Context<Self>) {
        if self.pane_width != width {
            self.pane_width = width;
            cx.notify();
        }
    }

    #[cfg(test)]
    pub fn show_error_for_test(&mut self, raw: &str) {
        self.ai_error = Some((
            AiJob::Summary { brief: false },
            page_chrome::ai_trouble(raw),
        ));
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
        if mode == Mode::Overview {
            self.reload_overview(cx);
            return;
        }
        let store = self.store.clone();
        let task = cx
            .background_executor()
            .spawn(async move { store.open_loops(&LoopQuery::default()) });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(loops) => {
                        this.loops = loops;
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
        self.loops
            .retain(|l| !(l.note_id == row.note_id && l.key == row.key));
        self.reload(cx);
        cx.notify();
    }

    fn mark_in_note(cx: &App) -> bool {
        cx.try_global::<AppState>()
            .is_some_and(|state| state.config.briefs.mark_done_in_note)
    }

    fn step_week(&mut self, weeks: i64, cx: &mut Context<Self>) {
        self.week_start = self.week_start.add_days(7 * weeks);
        self.ai_para = None;
        self.load_ai(cx);
        self.reload(cx);
    }

    fn this_week(&mut self, cx: &mut Context<Self>) {
        self.week_start = brief::week_start(crate::journal::now(cx).date);
        self.ai_para = None;
        self.load_ai(cx);
        self.reload(cx);
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
        self.working = Some(AiJob::FindMore);
        self.ai_error = None;
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
            loops_ai::find_more(
                &store,
                provider.as_ref(),
                loops_ai::DEFAULT_DAYS,
                now,
                &|| cancel.load(Ordering::SeqCst),
            )
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
                    Err(message) => {
                        this.ai_error = Some((AiJob::FindMore, page_chrome::ai_trouble(&message)));
                    }
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

    // MARK: Drawing: shared

    /// The page around `body`: the fitted header, then the centred column
    /// scrolling under it.
    fn shell(
        &self,
        title: &'static str,
        actions: Vec<HeaderAction>,
        body: AnyElement,
        scrolls: bool,
        window: &Window,
        cx: &App,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let header = page_chrome::page_header(
            title,
            self.leading_inset,
            self.pane_width,
            actions,
            None,
            window,
            cx,
        );
        let body = if scrolls {
            div()
                .id("digest-page")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(&self.page_scroll)
                .child(page_chrome::centred(body))
                .into_any_element()
        } else {
            div().flex_1().min_h_0().child(body).into_any_element()
        };
        v_flex()
            .size_full()
            .bg(colors.editor)
            .child(header)
            .child(body)
            .into_any_element()
    }

    /// The header action for an AI job, spinning while it runs.
    fn ai_action(
        &self,
        id: &'static str,
        job: AiJob,
        label: &'static str,
        busy: &'static str,
        tooltip: &'static str,
        cx: &mut Context<Self>,
    ) -> HeaderAction {
        let view = cx.entity().downgrade();
        let running = self.working == Some(job);
        HeaderAction::new(
            id,
            Lucide::Sparkles,
            if running { busy } else { label },
            move |window, cx| {
                view.update(cx, |this, cx| this.run_job(job, window, cx))
                    .ok();
            },
        )
        .tooltip(tooltip)
        .loading(running)
        .disabled(self.working.is_some())
    }

    fn run_job(&mut self, job: AiJob, window: &mut Window, cx: &mut Context<Self>) {
        match job {
            AiJob::FindMore => self.ask_find_more(window, cx),
            AiJob::Summary { .. } => self.summarise_overview(cx),
        }
    }

    /// The inline card for an AI action of this page that failed.
    fn error_card(
        &self,
        jobs: &[AiJob],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (job, trouble) = self.ai_error.as_ref().filter(|(j, _)| jobs.contains(j))?;
        let job = *job;
        let retry = cx.entity().downgrade();
        let dismiss = cx.entity().downgrade();
        Some(
            page_chrome::ai_error_card(
                trouble,
                move |window, cx| {
                    retry
                        .update(cx, |this, cx| {
                            this.ai_error = None;
                            match job {
                                // Asked once already: straight to work.
                                AiJob::FindMore => this.find_more(cx),
                                _ => this.run_job(job, window, cx),
                            }
                        })
                        .ok();
                },
                move |_, cx| {
                    dismiss
                        .update(cx, |this, cx| {
                            this.ai_error = None;
                            cx.notify();
                        })
                        .ok();
                },
                window,
                cx,
            )
            .into_any_element(),
        )
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
            .icon(
                Icon::new(icon)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .tooltip(tooltip)
    }

    fn today_button(&self, primary: bool, cx: &mut Context<Self>) -> AnyElement {
        let b = Button::new("digest-open-today")
            .icon(Lucide::CalendarDays)
            .label("Open Today's Note")
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DigestEvent::OpenToday)));
        if primary { b.primary() } else { b.outline() }.into_any_element()
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
            action(format!("loop-done-{}", row.key), Lucide::Check, "Done").on_click(cx.listener(
                move |this, _, _, cx| {
                    cx.stop_propagation();
                    let mark = Self::mark_in_note(cx);
                    this.close(&row, LoopClose::Done { mark_in_note: mark }, cx);
                },
            ))
        };
        let make_task = {
            let row = row.clone();
            action(
                format!("loop-task-{}", row.key),
                Lucide::ListPlus,
                "Make Task",
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.close(&row, LoopClose::MakeTask, cx);
            }))
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
        let inner = div()
            .id(SharedString::from(format!(
                "loop-{}-{}",
                row.note_id, row.key
            )))
            .group(group.clone())
            .h(px(ROW_HEIGHT))
            .w_full()
            .max_w(px(page_chrome::COLUMN_WIDTH))
            .px(px(24.))
            .py(px(1.))
            .cursor_pointer()
            .child(
                h_flex()
                    .size_full()
                    .px(px(ROW_PAD_X))
                    .gap(px(10.))
                    .items_start()
                    .pt(px(8.))
                    .rounded(px(7.))
                    .hover(|el| el.bg(colors.row_hover))
                    .child(
                        Icon::new(Lucide::CircleDot)
                            .size(px(15.))
                            .mt(px(2.))
                            .text_color(colors.text_muted),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                h_flex()
                                    .truncate()
                                    .text_size(px(14.))
                                    .line_height(px(19.))
                                    .text_color(colors.text)
                                    .child(div().font_semibold().child(bold.to_owned()))
                                    .child(div().child(rest.to_owned())),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.5))
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
            });
        page_chrome::centred(inner).into_any_element()
    }

    fn loop_header(&self, kind: LoopKind, count: usize, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        page_chrome::centred(
            h_flex()
                .h(px(HEADER_HEIGHT))
                .w_full()
                .max_w(px(page_chrome::COLUMN_WIDTH))
                .px(px(24. + ROW_PAD_X))
                .pt(px(12.))
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child(group_label(kind)),
                )
                .child(
                    div()
                        .px(px(6.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(11.5))
                        .text_color(colors.text_secondary)
                        .child(count.to_string()),
                ),
        )
        .into_any_element()
    }

    fn render_loops(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = entries(&self.loops);
        let ai = Self::ai_on(cx);
        let off = !cx.global::<AppState>().config.briefs.loops;
        let mut actions = Vec::new();
        // With none yet, the empty page offers it instead.
        if ai && !off && !list.is_empty() {
            actions.push(self.ai_action(
                "find-loops",
                AiJob::FindMore,
                "Find More with AI",
                "Finding…",
                "Read recent notes with your model to find promises the rules missed",
                cx,
            ));
        }
        // What the list shows, so the line and the list never disagree.
        let shown_loops = list
            .iter()
            .filter(|entry| matches!(entry, Entry::Loop(_)))
            .count();
        let hero = page_chrome::hero(
            "Open Loops",
            Some(if shown_loops == 0 {
                "The promises and questions in your notes, in one place.".into()
            } else {
                format!(
                    "{} in your notes: things you'll do, wait on or need to check.",
                    plural(shown_loops as u32, "open loop", "open loops")
                )
                .into()
            }),
            None,
            cx,
        );
        let top = page_chrome::column()
            .pb(px(0.))
            .child(hero)
            .children(self.error_card(&[AiJob::FindMore], window, cx));
        let body = if let Some(found) = &self.found {
            v_flex()
                .size_full()
                .child(page_chrome::centred(top))
                .child(self.found_sheet(found, cx))
                .into_any_element()
        } else if off || list.is_empty() {
            let empty = if off {
                page_chrome::page_empty(
                    Lucide::CircleDot,
                    "Open loops are off",
                    "Turn them on and Kanso keeps a list of the promises in your notes, like “I'll send the deck” or “waiting on Sam”, so nothing slips.",
                    vec![
                        Button::new("loops-turn-on")
                            .primary()
                            .label("Turn On Open Loops")
                            .on_click(cx.listener(|this, _, _, cx| {
                                AppState::update_config(cx, |config| config.briefs.loops = true);
                                this.reload(cx);
                            }))
                            .into_any_element(),
                    ],
                    cx,
                )
            } else {
                let mut buttons = vec![self.today_button(!ai, cx)];
                if ai {
                    buttons.insert(
                        0,
                        Button::new("loops-empty-find")
                            .primary()
                            .icon(Lucide::Sparkles)
                            .label("Find More with AI")
                            .loading(self.working == Some(AiJob::FindMore))
                            .disabled(self.working.is_some())
                            .on_click(
                                cx.listener(|this, _, window, cx| this.ask_find_more(window, cx)),
                            )
                            .into_any_element(),
                    );
                }
                page_chrome::page_empty(
                    Lucide::CircleCheck,
                    "No open loops",
                    "When you write things like “I'll send the deck”, “waiting on Sam” or “need to check the lease”, Kanso notices and lists them here, so nothing slips.",
                    buttons,
                    cx,
                )
            };
            let page = top.child(if self.loaded || off {
                empty.into_any_element()
            } else {
                div().into_any_element()
            });
            return self.shell(
                "Open Loops",
                actions,
                page.into_any_element(),
                true,
                window,
                cx,
            );
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
            v_flex()
                .size_full()
                .child(page_chrome::centred(top))
                .child(
                    div().flex_1().min_h_0().child(
                        v_virtual_list(cx.entity(), "loops", sizes, move |this, range, _, cx| {
                            range
                                .filter_map(|at| {
                                    Some(match shown.get(at)? {
                                        Entry::Header { kind, count } => {
                                            this.loop_header(*kind, *count, cx)
                                        }
                                        Entry::Loop(ix) => this.loop_row(*ix, cx),
                                    })
                                })
                                .collect()
                        })
                        .size_full()
                        .track_scroll(&self.list_scroll),
                    ),
                )
                .into_any_element()
        };
        self.shell("Open Loops", actions, body, false, window, cx)
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
                    .px(px(4.))
                    .cursor_pointer()
                    .child(
                        h_flex()
                            .w_full()
                            .px(px(ROW_PAD_X))
                            .py(px(8.))
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
                                .text_color(if on {
                                    colors.accent
                                } else {
                                    colors.text_muted
                                }),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .text_color(colors.text)
                                            .child(text.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.5))
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
        let column = page_chrome::column()
            .pt(px(0.))
            .child(
                div()
                    .py(px(8.))
                    .text_size(px(13.5))
                    .text_color(colors.text_secondary)
                    .child("Your model found these. Choose the ones that are real: nothing is kept until you do."),
            )
            .child(
                div()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .py(px(4.))
                    .child(rows),
            );
        v_flex()
            .size_full()
            .child(
                div()
                    .id("found-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(page_chrome::centred(column)),
            )
            .child(
                h_flex()
                    .flex_shrink_0()
                    .px(px(16.))
                    .py(px(10.))
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
            Some(Mode::Overview) => self.render_overview(window, cx),
            None => div().into_any_element(),
        };
        div()
            .size_full()
            .bg(colors.editor)
            .key_context("DigestView")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified() {
                    cx.emit(DigestEvent::Close);
                    cx.stop_propagation();
                }
            }))
            .child(page)
    }
}
