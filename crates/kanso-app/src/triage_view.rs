//! Inbox triage on the desktop (issue #195): the bar above the Inbox's notes
//! ("7 notes to file · Triage") and the Triage view over the window, one
//! card per note with a chip for each suggested notebook, tag, title and
//! link. A chip is accepted (✓), rejected (✕) or edited; File applies the
//! accepted chips of a card and File all accepted those of every card,
//! asking once, with one Undo. Nothing moves, retitles, retags or relinks
//! without that tap.
//!
//! The suggestions, the chips' state and filing are the engine's
//! (`kanso_engine::triage`), which the phones use too. This is the window's
//! side: drawing, keys, the model call and the notices.
//!
//! Keys (with the view focused): J or ↓ next card, K or ↑ previous; → or L
//! next chip, ← or H previous; Y accepts the chip, N rejects it, E edits
//! it; ⌘↩ accepts every chip of the card; F files the card, S skips it, O
//! opens it; Esc closes. The keys are handled here, not bound globally.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::roles::Role;
use kanso_engine::triage::file::{self, undo};
use kanso_engine::triage::session::{CardState, Chip, Mark, Session};
use kanso_engine::triage::{self, Field, Filed, Reason, Source, TriageOptions};
use kanso_store::Store;

use crate::clock::Clock;
use crate::config::{AppConfig, MAX_TAGS_LIMIT};
use crate::layout::BOTTOM_BAND_HEIGHT;
use crate::library::{Library, LibraryEvent};
use crate::page_chrome::{self, ActionStyle, HeaderAction};
use crate::theme::Colors;
use crate::workspace::{AppState, Workspace};
use crate::zoom::px;

/// How many cards the view computes at once.
const CARDS: usize = 200;
/// How long the library stays still before a shown view is read again.
const RELOAD_DELAY: Duration = Duration::from_millis(250);

actions!(kanso, [ShowTriage, CloseTriageView]);

/// What the view asks of the workspace.
pub enum TriageEvent {
    /// Open the note in the editor, closing Triage.
    Open(String),
    /// Open the note in a new tab, leaving Triage as it is.
    OpenInTab(String),
}

/// What a key does in the view. Pure, so the keys can be tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NextCard,
    PreviousCard,
    NextChip,
    PreviousChip,
    Accept,
    Reject,
    Edit,
    AcceptAll,
    File,
    Skip,
    Open,
    Close,
}

/// The command for a key, if it is one. `cmd` is ⌘ (Ctrl elsewhere).
pub fn command_for(key: &str, cmd: bool, shift: bool) -> Option<Command> {
    if cmd {
        return (key == "enter" && !shift).then_some(Command::AcceptAll);
    }
    Some(match key {
        "j" | "down" => Command::NextCard,
        "k" | "up" => Command::PreviousCard,
        "l" | "right" | "tab" if !shift => Command::NextChip,
        "h" | "left" => Command::PreviousChip,
        "tab" => Command::PreviousChip,
        "y" => Command::Accept,
        "n" => Command::Reject,
        "e" => Command::Edit,
        "f" => Command::File,
        "s" => Command::Skip,
        "o" | "enter" => Command::Open,
        "escape" => Command::Close,
        _ => return None,
    })
}

/// What a card shows of its note.
#[derive(Clone, Debug, Default)]
struct Info {
    title: String,
    snippet: String,
    updated_at: i64,
    /// Titles of the notes the chips cite, by ID.
    names: HashMap<String, String>,
}

/// What a background read found.
struct Loaded {
    /// The notes waiting, newest first.
    order: Vec<String>,
    /// The cards made now: not those of notes in `skip`.
    cards: Vec<triage::Card>,
    infos: HashMap<String, Info>,
}

pub struct TriageView {
    store: Arc<Store>,
    session: Session,
    infos: HashMap<String, Info>,
    /// Drawn on screen: only then is it read again on changes.
    pub visible: bool,
    loading: bool,
    stale: bool,
    /// The chip with the keys, on the focused card.
    chip: usize,
    focus: FocusHandle,
    scroll: ScrollHandle,
    reload_task: Option<Task<()>>,
    ai_task: Option<Task<()>>,
    ai_cancel: Arc<AtomicBool>,
    /// Cards the model is working on, or has finished.
    ai_done: HashSet<String>,
    /// Notes that changed since their card was made: only these are made
    /// again on the next read.
    dirty: HashSet<String>,
    /// The page's width and the room the traffic lights need at its left.
    pane_width: Pixels,
    leading_inset: Pixels,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<TriageEvent> for TriageView {}

impl Focusable for TriageView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// The options for a card, from Settings › General › Inbox.
pub fn options(config: &AppConfig) -> TriageOptions {
    TriageOptions {
        inbox_id: Role::Inbox.book_id().to_owned(),
        suggest_titles: config.inbox.suggest_titles,
        suggest_links: config.inbox.suggest_links,
        max_tags: usize::from(config.inbox.max_tags.clamp(1, MAX_TAGS_LIMIT)),
    }
}

/// Whether the model refines the cards: AI and its Inbox switch are on.
fn ai_on(config: &AppConfig) -> bool {
    config.ai.enabled && config.ai.inbox_suggestions
}

/// Reads which notes wait, and the cards of those not in `skip` (cards
/// already on screen and unchanged), sharing one read of the library.
fn read(store: &Store, opts: &TriageOptions, skip: &HashSet<String>) -> Result<Loaded, String> {
    let order: Vec<String> = store
        .inbox_waiting(&opts.inbox_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .take(CARDS)
        .map(|n| n.id)
        .collect();
    let fresh: Vec<String> = order
        .iter()
        .filter(|id| !skip.contains(*id))
        .cloned()
        .collect();
    let cards = triage::cards_for(store, &fresh, opts).map_err(|e| e.to_string())?;
    let mut infos = HashMap::new();
    for card in &cards {
        let Some(note) = store.note(&card.note_id).map_err(|e| e.to_string())? else {
            continue;
        };
        let mut names = HashMap::new();
        for s in card.candidates.iter().chain(&card.suggestions) {
            for id in &s.because {
                if !names.contains_key(id)
                    && let Ok(Some(n)) = store.note(id)
                {
                    names.insert(id.clone(), n.title);
                }
            }
        }
        infos.insert(
            card.note_id.clone(),
            Info {
                snippet: kanso_core::links::plain_line(note.body.trim_start())
                    .chars()
                    .take(280)
                    .collect(),
                title: note.title,
                updated_at: note.updated_at,
                names,
            },
        );
    }
    Ok(Loaded {
        order,
        cards,
        infos,
    })
}

impl TriageView {
    pub fn new(store: Arc<Store>, library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if let LibraryEvent::Changed(change) = event
                && change.id.starts_with("note:")
            {
                // A note open on a card changed elsewhere: its card says so
                // and File waits until the card is made again.
                if let Some(card) = this
                    .session
                    .cards
                    .iter_mut()
                    .find(|c| c.note_id == change.id && c.rev != change.rev)
                {
                    card.changed = true;
                }
                this.dirty.insert(change.id.clone());
                this.stale = true;
                if this.visible {
                    this.reload_soon(cx);
                }
                cx.notify();
            }
        })];
        TriageView {
            store,
            session: Session::default(),
            infos: HashMap::new(),
            visible: false,
            loading: false,
            stale: true,
            chip: 0,
            focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            reload_task: None,
            ai_task: None,
            ai_cancel: Arc::new(AtomicBool::new(false)),
            ai_done: Default::default(),
            dirty: HashSet::new(),
            pane_width: px(1000.),
            leading_inset: px(0.),
            _subscriptions: subscriptions,
        }
    }

    /// The page's width and leading inset, for fitting its header.
    pub fn set_frame(&mut self, width: Pixels, inset: Pixels, cx: &mut Context<Self>) {
        if self.pane_width != width || self.leading_inset != inset {
            self.pane_width = width;
            self.leading_inset = inset;
            cx.notify();
        }
    }

    /// Whether the model is still going over cards.
    fn refining(&self) -> bool {
        self.ai_task.is_some()
            && self
                .session
                .cards
                .iter()
                .any(|c| !self.ai_done.contains(&c.note_id))
    }

    /// Opens the view: the cards are made again.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        self.session = Session::default();
        self.ai_done.clear();
        self.dirty.clear();
        self.reload(cx);
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    /// Makes the cards again, off the main thread, keeping what the person
    /// did to the cards that are still there.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        self.loading = true;
        let store = self.store.clone();
        let opts = options(&cx.global::<AppState>().config);
        let skip = self.unchanged();
        let task = cx
            .background_executor()
            .spawn(async move { read(&store, &opts, &skip) });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let loaded = task.await;
            this.update(cx, |this, cx| {
                this.loading = false;
                match loaded {
                    Ok(loaded) => this.apply(loaded, cx),
                    Err(err) => eprintln!("kanso: triage: {err}"),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// The notes whose cards are on screen and whose note has not changed:
    /// they are not made again. Takes the changed set.
    fn unchanged(&mut self) -> HashSet<String> {
        let dirty = std::mem::take(&mut self.dirty);
        self.session
            .cards
            .iter()
            .map(|c| c.note_id.clone())
            .filter(|id| !dirty.contains(id))
            .chain(self.session.done.iter().cloned())
            .collect()
    }

    /// Reads the cards now, on this thread: for tests.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let opts = options(&cx.global::<AppState>().config);
        let skip = self.unchanged();
        if let Ok(loaded) = read(&self.store, &opts, &skip) {
            self.apply(loaded, cx);
        }
        cx.notify();
    }

    fn apply(&mut self, loaded: Loaded, cx: &mut Context<Self>) {
        let focused = self
            .session
            .focused()
            .map(|ix| self.session.cards[ix].note_id.clone());
        let old = std::mem::take(&mut self.session.cards);
        let done = std::mem::take(&mut self.session.done);
        let mut made: HashMap<String, triage::Card> = loaded
            .cards
            .into_iter()
            .map(|c| (c.note_id.clone(), c))
            .collect();
        let mut cards = Vec::new();
        for id in &loaded.order {
            if done.contains(id) {
                continue;
            }
            let previous = old.iter().find(|c| &c.note_id == id);
            let state = match (made.remove(id), previous) {
                // Not made again: exactly as the person left it.
                (None, Some(prev)) => prev.clone(),
                (None, None) => continue,
                // Unchanged: exactly as the person left it.
                (Some(card), Some(prev)) if prev.rev == card.rev => prev.clone(),
                // Changed elsewhere: what the person accepted or typed
                // stays, the rest is what the library finds now; a skip
                // stays too.
                (Some(card), Some(prev)) => {
                    let mut state = prev.clone();
                    state.rev = card.rev;
                    state.changed = false;
                    state.candidates = card.candidates;
                    state.missing = card.missing;
                    state.replace_suggestions(card.suggestions);
                    state
                }
                (Some(card), None) => CardState::new(card),
            };
            cards.push(state);
        }
        self.session = Session {
            cards,
            focus: 0,
            done,
        };
        if let Some(id) = focused
            && let Some(ix) = self.session.cards.iter().position(|c| c.note_id == id)
        {
            self.session.focus = ix;
        }
        if let Some(ix) = self.session.focused() {
            let len = self.session.cards[ix].chips.len();
            self.chip = self.chip.min(len.saturating_sub(1));
        }
        let shown: HashSet<&String> = self.session.cards.iter().map(|c| &c.note_id).collect();
        self.infos.retain(|id, _| shown.contains(id));
        self.infos.extend(loaded.infos);
        let config = cx.global::<AppState>().config.clone();
        if ai_on(&config) {
            self.refine_with_ai(cx);
        }
    }

    // MARK: AI

    /// Asks the model about each card not yet asked, one at a time.
    fn refine_with_ai(&mut self, cx: &mut Context<Self>) {
        let waiting: Vec<String> = self
            .session
            .cards
            .iter()
            .filter(|c| !self.ai_done.contains(&c.note_id))
            .map(|c| c.note_id.clone())
            .collect();
        if waiting.is_empty() {
            return;
        }
        self.ai_cancel.store(true, Ordering::Relaxed);
        let cancel = Arc::new(AtomicBool::new(false));
        self.ai_cancel = cancel.clone();
        let maker = crate::ai_feature::maker(cx);
        let store = self.store.clone();
        let opts = options(&cx.global::<AppState>().config);
        self.ai_task = Some(cx.spawn(async move |this, cx| {
            for id in waiting {
                if cancel.load(Ordering::Relaxed) {
                    return;
                }
                let card = this
                    .update(cx, |this, _| {
                        this.session
                            .cards
                            .iter()
                            .find(|c| c.note_id == id)
                            .map(|c| triage::Card {
                                note_id: c.note_id.clone(),
                                rev: c.rev.clone(),
                                suggestions: c.chips.iter().map(|c| c.suggestion.clone()).collect(),
                                candidates: c.candidates.clone(),
                                missing: c.missing,
                            })
                    })
                    .ok()
                    .flatten();
                let Some(card) = card else { continue };
                let (maker, store, opts, cancel2) =
                    (maker.clone(), store.clone(), opts.clone(), cancel.clone());
                let rev = card.rev.clone();
                let cancel3 = cancel.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let note = store.note(&card.note_id).ok().flatten();
                        let (title, text) = note.map(|n| (n.title, n.body)).unwrap_or_default();
                        crate::ai_feature::triage(
                            &maker, &store, &card, &title, &text, &opts, &cancel2,
                        )
                    })
                    .await;
                this.update(cx, |this, cx| {
                    // A newer pass took over, or the view closed: this
                    // answer is dropped and the card stays to be asked.
                    if cancel3.load(Ordering::Relaxed) {
                        return;
                    }
                    let Some(card) = this
                        .session
                        .cards
                        .iter_mut()
                        .find(|c| c.note_id == id && c.rev == rev)
                    else {
                        return;
                    };
                    // Only a card the answer was applied to is done.
                    this.ai_done.insert(id.clone());
                    match result {
                        Ok(suggestions) => {
                            card.ai_error = None;
                            card.replace_suggestions(suggestions);
                        }
                        Err(reason) => card.ai_error = Some(reason),
                    }
                    // Chips keep their places; only a shorter list moves
                    // the keys, to the last chip.
                    if let Some(ix) = this.session.focused() {
                        let len = this.session.cards[ix].chips.len();
                        this.chip = this.chip.min(len.saturating_sub(1));
                    }
                    cx.notify();
                })
                .ok();
            }
        }));
    }

    // MARK: Chips

    fn focused_card(&self) -> Option<usize> {
        self.session.focused()
    }

    fn scroll_to_focus(&self) {
        if let Some(ix) = self.focused_card() {
            let shown = self.session.shown();
            if let Some(at) = shown.iter().position(|s| *s == ix) {
                self.scroll.scroll_to_item(at);
            }
        }
    }

    fn step_card(&mut self, by: isize, cx: &mut Context<Self>) {
        self.session.step(by);
        self.chip = 0;
        self.scroll_to_focus();
        cx.notify();
    }

    fn step_chip(&mut self, by: isize, cx: &mut Context<Self>) {
        let Some(ix) = self.focused_card() else {
            return;
        };
        let count = self.session.cards[ix].chips.len();
        if count > 0 {
            self.chip = (self.chip as isize + by).clamp(0, count as isize - 1) as usize;
        }
        cx.notify();
    }

    fn accept(&mut self, card: usize, chip: usize, cx: &mut Context<Self>) {
        let Some(state) = self.session.cards.get_mut(card) else {
            return;
        };
        let lifting = state
            .chips
            .get(chip)
            .is_some_and(|c| c.mark == Mark::Rejected);
        if lifting {
            self.unreject(card, chip);
        }
        if let Some(state) = self.session.cards.get_mut(card) {
            state.toggle_accept(chip);
        }
        self.focus_chip(card, chip);
        cx.notify();
    }

    fn reject(&mut self, card: usize, chip: usize, cx: &mut Context<Self>) {
        let now = Clock::now_ms(cx);
        let Some(state) = self.session.cards.get_mut(card) else {
            return;
        };
        let note_id = state.note_id.clone();
        if let Some((suggestion, rejected)) = state.toggle_reject(chip) {
            let done = if rejected {
                triage::reject(&self.store, &note_id, &suggestion, now)
            } else {
                triage::unreject(&self.store, &note_id, &suggestion)
            };
            if let Err(err) = done {
                eprintln!("kanso: triage: couldn't remember a rejection: {err}");
            }
        }
        self.focus_chip(card, chip);
        cx.notify();
    }

    /// Takes back the memory of a rejection the person is now accepting.
    fn unreject(&mut self, card: usize, chip: usize) {
        if let Some(state) = self.session.cards.get(card)
            && let Some(c) = state.chips.get(chip)
            && let Err(err) = triage::unreject(&self.store, &state.note_id, &c.suggestion)
        {
            eprintln!("kanso: triage: {err}");
        }
    }

    fn focus_chip(&mut self, card: usize, chip: usize) {
        self.session.focus = card;
        self.chip = chip;
    }

    fn accept_all_on_card(&mut self, card: usize, cx: &mut Context<Self>) {
        if let Some(state) = self.session.cards.get_mut(card) {
            state.accept_all();
        }
        cx.notify();
    }

    fn accept_everywhere(&mut self, field: Field, cx: &mut Context<Self>) {
        self.session.accept_everywhere(field);
        cx.notify();
    }

    // MARK: Editing

    /// Edits chip `chip` of `card`: a dialog takes the new value, which
    /// must be one the library has (a tag may be new).
    fn edit(&mut self, card: usize, chip: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.session.cards.get(card) else {
            return;
        };
        let Some(current) = state.chips.get(chip).map(|c| c.suggestion.clone()) else {
            return;
        };
        let note_id = state.note_id.clone();
        let view = cx.entity().downgrade();
        let store = self.store.clone();
        let (title, initial) = match current.field {
            Field::Notebook => ("Notebook", current.label.clone()),
            Field::Tag => ("Tag", current.label.clone()),
            Field::Title => ("Title", current.value.clone()),
            Field::Link => ("Link to note", current.label.clone()),
        };
        let field = current.field;
        crate::dialogs::prompt_checked(title, &initial, "Use", window, cx, move |text, _, cx| {
            let made = triage::resolve(&store, field, &text)?;
            let (view, note_id) = (view.clone(), note_id.clone());
            view.update(cx, |this, cx| {
                if let Some(state) = this.session.cards.iter_mut().find(|c| c.note_id == note_id)
                    && chip < state.chips.len()
                {
                    state.edit(chip, made);
                }
                cx.notify();
            })
            .ok();
            Ok(())
        });
    }

    // MARK: Filing

    fn file_card(&mut self, card: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(filing) = self.session.cards.get(card).and_then(CardState::filing) else {
            return;
        };
        self.file(vec![filing], window, cx);
    }

    /// "File all accepted": asks once, then files every card with something
    /// accepted in one write.
    fn file_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let filings = self.session.filings();
        if filings.is_empty() {
            return;
        }
        crate::editor::peers::flush_everywhere(cx);
        let now = Clock::now_ms(cx);
        let plan = match file::plan(&self.store, &filings, now) {
            Ok(plan) => plan,
            Err(err) => {
                window.push_notification(
                    Notification::error(format!("Couldn't plan the filing: {err}")),
                    cx,
                );
                return;
            }
        };
        let count = plan.edits.len();
        let view = cx.entity().downgrade();
        let title = format!(
            "File {}?",
            crate::text_stats::plural(count, "note", "notes")
        );
        crate::dialogs::confirm(
            title,
            plan.sentence(),
            "File",
            window,
            cx,
            move |window, cx| {
                view.update(cx, |this, cx| this.file(filings.clone(), window, cx))
                    .ok();
            },
        );
    }

    /// Files `filings` in one write and says so with Undo.
    fn file(&mut self, filings: Vec<triage::Filing>, window: &mut Window, cx: &mut Context<Self>) {
        let held = cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.writes_held);
        if held {
            window.push_notification(
                Notification::error("Notes can't be changed while the library is being replaced."),
                cx,
            );
            return;
        }
        crate::editor::peers::flush_everywhere(cx);
        let now = Clock::now_ms(cx);
        let filed = match file::file(&self.store, &filings, now) {
            Ok(filed) => filed,
            Err(err) => {
                window.push_notification(
                    Notification::error(format!("Couldn't file the notes: {err}")),
                    cx,
                );
                return;
            }
        };
        let ids: Vec<String> = filed.edits.iter().map(|e| e.id.clone()).collect();
        self.session.remove(&ids);
        for id in &filed.stale {
            if let Some(card) = self.session.cards.iter_mut().find(|c| &c.note_id == id) {
                card.changed = true;
            }
        }
        if !filed.stale.is_empty() {
            self.reload(cx);
        }
        self.chip = 0;
        if filed.edits.is_empty() {
            window.push_notification(
                Notification::info("Changed on another device: those cards are made again."),
                cx,
            );
            cx.notify();
            return;
        }
        let words = crate::text_stats::plural(filed.edits.len(), "note", "notes");
        let store = self.store.clone();
        let edits = filed.edits.clone();
        let new_tags = filed.new_tags.clone();
        let view = cx.entity().downgrade();
        let notification = Notification::info(format!("Filed {words}"))
            .id::<Filed>()
            .action(move |_, _, _| {
                let (store, edits, new_tags, view) =
                    (store.clone(), edits.clone(), new_tags.clone(), view.clone());
                Button::new("undo-triage")
                    .label("Undo")
                    .debug_selector(|| "undo-triage".into())
                    .on_click(move |_, window, cx| {
                        crate::editor::peers::flush_everywhere(cx);
                        let now = Clock::now_ms(cx);
                        if let Err(err) = undo(&store, &edits, &new_tags, now) {
                            window.push_notification(
                                Notification::error(format!("Couldn't undo: {err}")),
                                cx,
                            );
                            return;
                        }
                        // The notes are no longer done: the view reads
                        // again and their cards come back.
                        let ids: Vec<String> = edits.iter().map(|e| e.id.clone()).collect();
                        view.update(cx, |this, cx| this.undone(&ids, cx)).ok();
                    })
            });
        window.push_notification(notification, cx);
        // The undone notes are back in the Inbox: the view reads again.
        self.reload_soon(cx);
        cx.notify();
    }

    /// Filed notes were put back by Undo: their cards come back.
    fn undone(&mut self, ids: &[String], cx: &mut Context<Self>) {
        self.session.restore(ids);
        for id in ids {
            self.ai_done.remove(id);
        }
        self.reload_task = None;
        self.reload(cx);
    }

    fn skip(&mut self, cx: &mut Context<Self>) {
        self.session.skip_focused();
        self.chip = 0;
        self.scroll_to_focus();
        cx.notify();
    }

    fn open_focused(&mut self, cx: &mut Context<Self>) {
        if let Some(ix) = self.focused_card() {
            cx.emit(TriageEvent::Open(self.session.cards[ix].note_id.clone()));
        }
    }

    /// Runs `command` for the card and chip with the keys.
    pub fn run(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        let Some(card) = self.focused_card() else {
            if command == Command::Close {
                window.dispatch_action(Box::new(CloseTriageView), cx);
            }
            return;
        };
        let chip = self.chip;
        match command {
            Command::NextCard => self.step_card(1, cx),
            Command::PreviousCard => self.step_card(-1, cx),
            Command::NextChip => self.step_chip(1, cx),
            Command::PreviousChip => self.step_chip(-1, cx),
            Command::Accept => self.accept(card, chip, cx),
            Command::Reject => self.reject(card, chip, cx),
            Command::Edit => self.edit(card, chip, window, cx),
            Command::AcceptAll => self.accept_all_on_card(card, cx),
            Command::File => self.file_card(card, window, cx),
            Command::Skip => self.skip(cx),
            Command::Open => self.open_focused(cx),
            Command::Close => window.dispatch_action(Box::new(CloseTriageView), cx),
        }
    }

    // MARK: Test access

    #[cfg(test)]
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }

    // MARK: Drawing

    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shown = self.session.shown().len();
        let accepted = self.session.filings().len();
        let title = if shown == 0 {
            "Triage".to_owned()
        } else {
            format!("Triage · {shown} to file")
        };
        let view = cx.entity().downgrade();
        let books = view.clone();
        let tags = view.clone();
        let file = view.clone();
        let actions = vec![
            HeaderAction::new(
                "triage-all-notebooks",
                Lucide::Notebook,
                "Accept all notebooks",
                move |_, cx| {
                    books
                        .update(cx, |this, cx| this.accept_everywhere(Field::Notebook, cx))
                        .ok();
                },
            )
            .tooltip("Tick every suggested notebook (nothing moves until File)")
            .disabled(shown == 0),
            HeaderAction::new(
                "triage-all-tags",
                Lucide::Tag,
                "Accept all tags",
                move |_, cx| {
                    tags.update(cx, |this, cx| this.accept_everywhere(Field::Tag, cx))
                        .ok();
                },
            )
            .tooltip("Tick every suggested tag (nothing is added until File)")
            .disabled(shown == 0),
            HeaderAction::new(
                "triage-file-all",
                Lucide::Inbox,
                format!("File all accepted ({accepted})"),
                move |window, cx| {
                    file.update(cx, |this, cx| this.file_all(window, cx)).ok();
                },
            )
            .tooltip("File every accepted suggestion, with one Undo")
            .style(ActionStyle::Primary)
            .disabled(accepted == 0),
        ];
        let close = HeaderAction::new("triage-close", Lucide::Close, "Close", |window, cx| {
            window.dispatch_action(Box::new(CloseTriageView), cx)
        })
        .tooltip("Close (Esc)");
        let colors = Colors::get(cx);
        page_chrome::page_header(
            title,
            self.leading_inset,
            self.pane_width,
            actions,
            Some(close),
            window,
            cx,
        )
        .border_b_1()
        .border_color(colors.divider)
        .bg(colors.list)
    }

    /// The lines above the cards: why notebooks and tags are missing, and
    /// where suggestions come from.
    fn render_notices(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let config = &cx.global::<AppState>().config;
        let related_off = self.store.embedding_model().ok().flatten().is_none();
        let mut lines: Vec<AnyElement> = Vec::new();
        if related_off {
            lines.push(
                h_flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child("Turn on Related notes to get notebook and tag suggestions."),
                    )
                    .child(
                        Button::new("triage-turn-on")
                            .outline()
                            .small()
                            .label("Turn on")
                            .debug_selector(|| "triage-turn-on".into())
                            .on_click(cx.listener(|this, _, _, cx| {
                                crate::related::set_enabled(true, cx);
                                this.reload_soon(cx);
                            })),
                    )
                    .into_any_element(),
            );
        }
        if !ai_on(config) {
            lines.push(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Suggestions come from your notes. Set up AI in Settings for better titles.")
                    .into_any_element(),
            );
        }
        (!lines.is_empty()).then(|| {
            v_flex()
                .flex_none()
                .px(px(18.))
                .py(px(8.))
                .gap(px(4.))
                .border_b_1()
                .border_color(colors.divider)
                .children(lines)
                .into_any_element()
        })
    }

    fn render_chip(
        &self,
        card_ix: usize,
        chip_ix: usize,
        chip: &Chip,
        focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let accent = Hsla::from(colors.accent);
        let s = &chip.suggestion;
        let text = match s.field {
            Field::Tag => format!("#{}", s.label),
            Field::Link => format!("[[{}]]", s.label),
            _ => s.label.clone(),
        };
        let (fill, border, text_color) = match chip.mark {
            Mark::Accepted => (accent.opacity(0.22), accent.opacity(0.7), colors.text),
            Mark::Rejected => (
                Hsla::transparent_black(),
                Hsla::from(colors.divider),
                colors.text_muted,
            ),
            Mark::Untouched => (
                Hsla::transparent_black(),
                Hsla::from(colors.text_muted).opacity(0.5),
                colors.text_secondary,
            ),
        };
        let rejected = chip.mark == Mark::Rejected;
        let id = format!("triage-chip-{card_ix}-{chip_ix}");
        h_flex()
            .id(SharedString::from(id.clone()))
            .debug_selector(move || id.clone())
            .h(px(24.))
            .pl(px(8.))
            .gap(px(4.))
            .items_center()
            .rounded(px(12.))
            .border_1()
            .border_color(if focused { accent } else { border })
            .bg(fill)
            .when(rejected, |el| el.opacity(0.6))
            .when(s.source == Source::Ai, |el| {
                el.child(
                    Icon::new(Lucide::Sparkles)
                        .size(px(11.))
                        .text_color(colors.accent),
                )
            })
            .when(s.is_new, |el| {
                el.child(
                    div()
                        .text_size(px(10.5))
                        .text_color(colors.text_muted)
                        .child("new"),
                )
            })
            .child(
                div()
                    .id(SharedString::from(format!(
                        "triage-edit-{card_ix}-{chip_ix}"
                    )))
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .text_color(text_color)
                    .when(rejected, |el| el.line_through())
                    .child(text)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.focus_chip(card_ix, chip_ix);
                        this.edit(card_ix, chip_ix, window, cx);
                    })),
            )
            .child(
                div()
                    .id(SharedString::from(format!(
                        "triage-accept-{card_ix}-{chip_ix}"
                    )))
                    .size(px(20.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(10.))
                    .hover(|el| el.bg(Hsla::from(colors.hover)))
                    .child(Icon::new(Lucide::Check).size(px(12.)).text_color(
                        if chip.mark == Mark::Accepted {
                            colors.accent
                        } else {
                            colors.text_muted
                        },
                    ))
                    .on_click(cx.listener(move |this, _, _, cx| this.accept(card_ix, chip_ix, cx))),
            )
            .child(
                div()
                    .id(SharedString::from(format!(
                        "triage-reject-{card_ix}-{chip_ix}"
                    )))
                    .size(px(20.))
                    .mr(px(2.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(10.))
                    .hover(|el| el.bg(Hsla::from(colors.hover)))
                    .child(
                        Icon::new(Lucide::Close)
                            .size(px(12.))
                            .text_color(if rejected {
                                colors.danger
                            } else {
                                colors.text_muted
                            }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.reject(card_ix, chip_ix, cx))),
            )
            .into_any_element()
    }

    /// One labelled row of a card's chips of `field`, if it has any.
    fn render_row(
        &self,
        ix: usize,
        label: &'static str,
        field: Field,
        focused_card: bool,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let card = &self.session.cards[ix];
        let chips: Vec<AnyElement> = card
            .chips
            .iter()
            .enumerate()
            .filter(|(_, c)| c.suggestion.field == field)
            .map(|(ci, chip)| self.render_chip(ix, ci, chip, focused_card && self.chip == ci, cx))
            .collect();
        (!chips.is_empty()).then(|| {
            h_flex()
                .gap(px(8.))
                .items_start()
                .child(
                    div()
                        .w(px(64.))
                        .flex_none()
                        .pt(px(4.))
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(label),
                )
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .flex_wrap()
                        .gap(px(6.))
                        .children(chips),
                )
                .into_any_element()
        })
    }

    fn render_card(&self, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let card = &self.session.cards[ix];
        let info = self.infos.get(&card.note_id).cloned().unwrap_or_default();
        let focused_card = self.focused_card() == Some(ix);
        let accent = Hsla::from(colors.accent);
        let title = if info.title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            info.title.clone()
        };
        let when = {
            let local = Clock::local_ms(cx, info.updated_at);
            kanso_engine::publish::when_label(local).replace(" at ", ", ")
        };
        let row = |label: &'static str, field: Field, cx: &mut Context<Self>| {
            self.render_row(ix, label, field, focused_card, cx)
        };
        let (r_book, r_tags, r_title, r_links) = (
            row("Notebook", Field::Notebook, cx),
            row("Tags", Field::Tag, cx),
            row("Title", Field::Title, cx),
            row("Links", Field::Link, cx),
        );
        // The notes that drove the suggestions, up to three.
        let mut because: Vec<String> = Vec::new();
        for chip in &card.chips {
            for id in &chip.suggestion.because {
                if !because.contains(id) && *id != card.note_id && because.len() < 3 {
                    because.push(id.clone());
                }
            }
        }
        let because_row = (!because.is_empty()).then(|| {
            h_flex()
                .gap(px(4.))
                .flex_wrap()
                .text_size(px(11.5))
                .text_color(colors.text_muted)
                .child("because: near")
                .children(
                    because
                        .iter()
                        .enumerate()
                        .map(|(i, id)| {
                            let name = info
                                .names
                                .get(id)
                                .filter(|n| !n.trim().is_empty())
                                .cloned()
                                .unwrap_or_else(|| "Untitled".to_owned());
                            let target = id.clone();
                            div()
                                .id(SharedString::from(format!("triage-because-{ix}-{i}")))
                                .cursor_pointer()
                                .text_color(colors.link)
                                .child(format!("“{name}”"))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.emit(TriageEvent::OpenInTab(target.clone()))
                                }))
                        })
                        .collect::<Vec<_>>(),
                )
        });
        let missing = match card.missing {
            Some(Reason::RelatedOff) => None,
            Some(Reason::NotEmbedded) => {
                Some("Notebook and tag suggestions come once this note is read.")
            }
            None => None,
        };
        let id = card.note_id.clone();
        let changed = card.changed;
        v_flex()
            .id(SharedString::from(format!("triage-card-{ix}")))
            .debug_selector(move || format!("triage-card-{ix}"))
            .flex_none()
            .w_full()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(10.))
            .border_1()
            .border_color(if focused_card {
                accent.opacity(0.6)
            } else {
                Hsla::from(colors.divider)
            })
            .bg(colors.list)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.session.focus = ix;
                    this.chip = 0;
                    cx.notify();
                }),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(14.))
                            .font_medium()
                            .text_color(colors.text)
                            .child(title),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child(when),
                    ),
            )
            .when(!info.snippet.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(17.))
                        .text_color(colors.text_secondary)
                        .line_clamp(2)
                        .child(info.snippet.clone()),
                )
            })
            .when(changed, |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.warning)
                        .child("Changed on another device"),
                )
            })
            .children(r_book)
            .children(r_tags)
            .children(r_title)
            .children(r_links)
            .when(card.chips.is_empty() && !changed, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child("Nothing to suggest for this note."),
                )
            })
            .children(because_row)
            .when_some(missing, |el, text| {
                el.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(text),
                )
            })
            .when_some(card.ai_error.clone(), |el, reason| {
                let trouble = page_chrome::ai_trouble(&reason);
                el.child(
                    h_flex()
                        .gap(px(6.))
                        .flex_wrap()
                        .text_size(px(12.))
                        .child(
                            Icon::new(Lucide::TriangleAlert)
                                .size(px(13.))
                                .text_color(colors.warning),
                        )
                        .child(div().text_color(colors.text_secondary).child(format!(
                            "{}. Showing suggestions from your notes. {}",
                            trouble.title, trouble.detail
                        )))
                        .child(
                            Button::new(SharedString::from(format!("triage-ai-settings-{ix}")))
                                .ghost()
                                .xsmall()
                                .label("AI Settings")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(
                                        Box::new(crate::settings::OpenAiSettings),
                                        cx,
                                    )
                                }),
                        ),
                )
            })
            .child(
                h_flex()
                    .gap(px(6.))
                    .justify_end()
                    .child(
                        Button::new(SharedString::from(format!("triage-accept-all-{ix}")))
                            .ghost()
                            .small()
                            .label("Accept all")
                            .tooltip("Accept every chip on this card (⌘↩)")
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.accept_all_on_card(ix, cx)),
                            ),
                    )
                    .child(
                        Button::new(SharedString::from(format!("triage-open-{ix}")))
                            .ghost()
                            .small()
                            .label("Open")
                            .on_click(cx.listener({
                                let id = id.clone();
                                move |_, _, _, cx| cx.emit(TriageEvent::Open(id.clone()))
                            })),
                    )
                    .child(
                        Button::new(SharedString::from(format!("triage-skip-{ix}")))
                            .ghost()
                            .small()
                            .label("Skip")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.session.focus = ix;
                                this.skip(cx)
                            })),
                    )
                    .child(
                        Button::new(SharedString::from(format!("triage-file-{ix}")))
                            .primary()
                            .small()
                            .label("File")
                            .disabled(changed)
                            .debug_selector(move || format!("triage-file-{ix}"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.file_card(ix, window, cx)
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for TriageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.visible && self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let shown = self.session.shown();
        let body = if shown.is_empty() && !self.loading {
            div()
                .id("triage-cards")
                .size_full()
                .overflow_y_scroll()
                .child(page_chrome::centred(page_chrome::column().child(
                    page_chrome::page_empty(
                        Lucide::Inbox,
                        "Your Inbox is clear",
                        "Triage files the notes that land in your Inbox. For each one it suggests a notebook, tags, a title and links, and nothing changes until you accept. New captures will wait here.",
                        vec![
                            Button::new("triage-empty-close")
                                .primary()
                                .label("Back to Notes")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(CloseTriageView), cx)
                                })
                                .into_any_element(),
                        ],
                        cx,
                    ),
                )))
                .into_any_element()
        } else {
            let refining = self.refining();
            div()
                .id("triage-cards")
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(page_chrome::centred(
                    page_chrome::column()
                        .gap(px(12.))
                        .pt(px(16.))
                        .when(refining, |el| {
                            el.child(
                                h_flex()
                                    .gap(px(6.))
                                    .text_size(px(12.5))
                                    .text_color(colors.text_muted)
                                    .child(
                                        Icon::new(Lucide::Sparkles)
                                            .size(px(13.))
                                            .text_color(colors.accent),
                                    )
                                    .child("Your model is improving the suggestions…"),
                            )
                        })
                        .children(shown.iter().map(|ix| self.render_card(*ix, cx))),
                ))
                .into_any_element()
        };
        v_flex()
            .id("triage-view")
            .key_context("TriageView")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let k = &event.keystroke;
                if let Some(command) = command_for(&k.key, k.modifiers.platform || k.modifiers.control, k.modifiers.shift)
                {
                    cx.stop_propagation();
                    this.run(command, window, cx);
                }
            }))
            // Presses here are the view's, not the window's below.
            .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
            .child(self.render_toolbar(window, cx))
            .children(self.render_notices(cx))
            .child(div().flex_1().min_h_0().child(body))
            .child(
                h_flex()
                    .h(px(BOTTOM_BAND_HEIGHT))
                    .flex_none()
                    .px(px(18.))
                    .border_t_1()
                    .border_color(colors.divider)
                    .bg(colors.list)
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(div().truncate().child(
                        "J K cards · ← → chips · Y accept · N reject · E edit · ⌘↩ accept all · F file · S skip · O open",
                    )),
            )
    }
}

// MARK: Workspace

/// The workspace's part: opening and closing Triage.
impl Workspace {
    pub(crate) fn show_triage(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.close_board(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let triage = match &self.triage {
            Some(triage) => triage.clone(),
            None => {
                let store = self.library.read(cx).store.clone();
                let library = self.library.clone();
                let triage = cx.new(|cx| TriageView::new(store, library, cx));
                self._triage_events = Some(cx.subscribe_in(
                    &triage,
                    window,
                    |this, _, event, window, cx| this.on_triage_event(event, window, cx),
                ));
                self.triage = Some(triage.clone());
                triage
            }
        };
        triage.update(cx, |triage, cx| triage.show(cx));
        self.triage_open = true;
        window.focus(&triage.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_triage(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.triage_open {
            return;
        }
        self.triage_open = false;
        if let Some(triage) = &self.triage {
            triage.update(cx, |triage, _| {
                triage.visible = false;
                triage.ai_cancel.store(true, Ordering::Relaxed);
            });
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_triage_event(
        &mut self,
        event: &TriageEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TriageEvent::Open(id) => {
                self.close_triage(window, cx);
                let id = id.clone();
                self.library
                    .update(cx, |library, cx| library.select(Some(id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }
            TriageEvent::OpenInTab(id) => self.open_note_in_new_tab(id, window, cx),
        }
    }
}

/// The workspace's actions for Triage.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowTriage, window, cx| this.show_triage(window, cx)))
        .on_action(
            cx.listener(|this, _: &CloseTriageView, window, cx| this.close_triage(window, cx)),
        )
}

/// Triage over the window, while open.
pub fn overlay(workspace: &Workspace, leading: f32) -> Option<impl IntoElement> {
    let triage = workspace.triage.clone().filter(|_| workspace.triage_open)?;
    // Over the list and the editor, beside the sidebar, as the other pages.
    Some(
        div()
            .absolute()
            .top_0()
            .left(px(leading))
            .right_0()
            .bottom_0()
            .debug_selector(|| "triage-overlay".into())
            .child(triage),
    )
}

// MARK: The bar

/// The bar above the Inbox's notes: how many wait, and Triage.
// no-prop-test: layout and one click; the count is the store's
// (`inbox_waiting`) and `command_for` has its own tests.
pub fn bar(count: usize, colors: &Colors) -> impl IntoElement {
    h_flex()
        .id("triage-bar")
        .debug_selector(|| "triage-bar".into())
        .flex_none()
        .h(px(36.))
        .mx(px(10.))
        .mb(px(6.))
        .px(px(10.))
        .gap(px(8.))
        .rounded(px(8.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.elevated)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(format!(
                    "{} to file",
                    crate::text_stats::plural(count, "note", "notes")
                )),
        )
        .child(
            Button::new("triage-open")
                .primary()
                .small()
                .label("Triage")
                .debug_selector(|| "triage-open".into())
                .on_click(|_, window, cx| window.dispatch_action(Box::new(ShowTriage), cx)),
        )
}

#[cfg(test)]
mod tests {
    use super::{Command, command_for};

    #[test]
    fn keys_map_to_commands() {
        assert_eq!(command_for("j", false, false), Some(Command::NextCard));
        assert_eq!(command_for("down", false, false), Some(Command::NextCard));
        assert_eq!(command_for("k", false, false), Some(Command::PreviousCard));
        assert_eq!(command_for("y", false, false), Some(Command::Accept));
        assert_eq!(command_for("n", false, false), Some(Command::Reject));
        assert_eq!(command_for("e", false, false), Some(Command::Edit));
        assert_eq!(command_for("enter", true, false), Some(Command::AcceptAll));
        assert_eq!(command_for("tab", false, true), Some(Command::PreviousChip));
        assert_eq!(command_for("tab", false, false), Some(Command::NextChip));
        assert_eq!(command_for("escape", false, false), Some(Command::Close));
        assert_eq!(command_for("j", true, false), None);
        assert_eq!(command_for("q", false, false), None);
    }
}
