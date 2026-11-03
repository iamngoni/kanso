//! The live sections below a person's or project's note (issue #200): open
//! items, what was promised and by whom, what was discussed, a timeline and
//! related pages. Computed sections come straight from the notes; AI
//! sections are written by the model with every line citing the lines it
//! rests on (`kanso_engine::pages`, which checks each citation against what
//! was sent). A line that cites nothing real is never shown.
//!
//! The panel draws nothing for a note that is not a page, so an ordinary
//! note's layout does not move. Nothing is written to the note itself: the
//! sections live in the page's cache (`kansoPage`), except when "Copy into
//! the note" is chosen.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::now_ms;
use kanso_engine::ask::{Receipt, relocate};
use kanso_engine::entities::{EntityPage, actions};
use kanso_engine::pages::{self, Item, Section, SectionKind, SectionState};
use kanso_store::{CheckOff, Origin, Store};

use crate::ai_feature::{self, Maker};
use crate::library::{Library, LibraryEvent};
use crate::page_chrome::{ai_error_card, ai_trouble};
use crate::people_data::{self, Loaded};
use crate::preview::TEXT_INSET;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// How long changes settle before the panel reads its page again.
const RELOAD_DELAY: Duration = Duration::from_millis(400);
/// The tallest the panel gets before it scrolls inside itself.
const MAX_HEIGHT: f32 = 320.;

/// What the panel asks of the editor around it.
pub enum PageEvent {
    /// Open a note and mark these lines (0-based, inclusive).
    OpenLines { id: String, first: u32, last: u32 },
    /// Open a note.
    OpenNote(String),
    /// Something worth telling the person.
    Notice(String),
    /// This page was merged into another: the note it became.
    Merged(String),
}

/// What a section's menu does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionAction {
    Refresh,
    Keep,
    Copy,
    Hide,
}

/// The menu entries of a section, in order: Refresh only for an AI section
/// while AI is on; Keep only for an AI section; Copy only with lines.
/// Pure, so the choices can be tested.
pub fn section_actions(section: &Section, ai_on: bool) -> Vec<SectionAction> {
    let ai = section.kind == SectionKind::Ai;
    let mut out = Vec::new();
    if ai && ai_on {
        out.push(SectionAction::Refresh);
    }
    if ai {
        out.push(SectionAction::Keep);
    }
    if !section.items.is_empty() {
        out.push(SectionAction::Copy);
    }
    out.push(SectionAction::Hide);
    out
}

/// The words for a section action.
pub fn action_label(action: SectionAction, pinned: bool) -> &'static str {
    match action {
        SectionAction::Refresh => "Refresh",
        SectionAction::Keep if pinned => "Stop keeping this version",
        SectionAction::Keep => "Keep this version",
        SectionAction::Copy => "Copy into the note",
        SectionAction::Hide => "Hide section",
    }
}

pub struct PagePanel {
    store: Arc<Store>,
    note_id: Option<String>,
    loaded: Option<Loaded>,
    /// Sections being written now.
    working: HashSet<String>,
    /// Sections that failed to write, with the model's own words.
    trouble: HashMap<String, String>,
    /// Sections folded to their title.
    folded: HashSet<String>,
    /// Computed sections shown in full.
    shown_all: HashSet<String>,
    /// The whole panel folded to its header.
    collapsed: bool,
    stale: bool,
    reload_task: Option<Task<()>>,
    write_task: Option<Task<()>>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PageEvent> for PagePanel {}

impl PagePanel {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if let LibraryEvent::Changed(change) = event
                && change.id.starts_with("note:")
                && this.note_id.is_some()
            {
                this.stale = true;
                this.reload_soon(cx);
            }
        })];
        PagePanel {
            store,
            note_id: None,
            loaded: None,
            working: HashSet::new(),
            trouble: HashMap::new(),
            folded: HashSet::new(),
            shown_all: HashSet::new(),
            collapsed: false,
            stale: false,
            reload_task: None,
            write_task: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// The note the editor shows now. The panel is read again for it.
    pub fn set_note(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if self.note_id == id {
            return;
        }
        self.note_id = id;
        self.loaded = None;
        self.working.clear();
        self.trouble.clear();
        self.shown_all.clear();
        self.write_task = None;
        self.reload(true, cx);
        cx.notify();
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(false, cx)).ok();
        }));
    }

    /// Reads the page off the main thread. `opened` is a note just shown:
    /// sections its refresh setting wants written are written then.
    fn reload(&mut self, opened: bool, cx: &mut Context<Self>) {
        self.stale = false;
        let Some(id) = self.note_id.clone() else {
            self.loaded = None;
            return;
        };
        let store = self.store.clone();
        // A window without settings (a test's editor) shows the defaults.
        let config = cx
            .try_global::<AppState>()
            .map(|state| state.config.clone())
            .unwrap_or_default();
        let shown: Vec<String> = self.shown_all.iter().cloned().collect();
        let task = cx.background_executor().spawn({
            let id = id.clone();
            async move {
                let all: Vec<&str> = shown.iter().map(String::as_str).collect();
                people_data::load(&store, &id, &config, &all)
            }
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let loaded = task.await;
            this.update(cx, |this, cx| {
                if this.note_id.as_deref() != Some(id.as_str()) {
                    return;
                }
                match loaded {
                    Ok(loaded) => {
                        let due = loaded
                            .as_ref()
                            .filter(|_| opened)
                            .map(|l| l.due.clone())
                            .unwrap_or_default();
                        this.loaded = loaded;
                        if !due.is_empty() {
                            this.write(due, cx);
                        }
                    }
                    Err(err) => eprintln!("kanso: page: {err}"),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Writes `keys` (every section that is not kept when empty) with the
    /// model, one at a time, so a failure names its own section.
    fn write(&mut self, keys: Vec<String>, cx: &mut Context<Self>) {
        let Some(id) = self.note_id.clone() else {
            return;
        };
        let maker: Maker = ai_feature::maker(cx);
        let model = cx.global::<AppState>().config.ai.model.clone();
        let store = self.store.clone();
        for key in &keys {
            self.working.insert(key.clone());
            self.trouble.remove(key);
        }
        self.write_task = Some(cx.spawn(async move |this, cx| {
            for key in keys {
                let result = cx
                    .background_executor()
                    .spawn({
                        let (store, id, key, model, maker) = (
                            store.clone(),
                            id.clone(),
                            key.clone(),
                            model.clone(),
                            maker.clone(),
                        );
                        async move { people_data::refresh(&store, &id, Some(&key), &model, &maker) }
                    })
                    .await;
                let stop = this
                    .update(cx, |this, cx| {
                        if this.note_id.as_deref() != Some(id.as_str()) {
                            return true;
                        }
                        this.working.remove(&key);
                        if let Err(reason) = result {
                            this.trouble.insert(key.clone(), reason);
                        }
                        this.reload(false, cx);
                        cx.notify();
                        false
                    })
                    .unwrap_or(true);
                if stop {
                    return;
                }
            }
        }));
    }

    /// The AI sections that are not kept, for Refresh all.
    fn refreshable(&self) -> Vec<String> {
        self.loaded
            .iter()
            .flat_map(|l| &l.view.sections)
            .filter(|s| {
                s.kind == SectionKind::Ai && !matches!(s.state, SectionState::Pinned { .. })
            })
            .map(|s| s.key.clone())
            .collect()
    }

    fn page(&self) -> Option<EntityPage> {
        let id = self.note_id.as_deref()?;
        EntityPage::load(&self.store, id).ok().flatten()
    }

    fn run(&mut self, key: &str, action: SectionAction, cx: &mut Context<Self>) {
        let Some(page) = self.page() else { return };
        let outcome = match action {
            SectionAction::Refresh => {
                self.write(vec![key.to_owned()], cx);
                return;
            }
            SectionAction::Keep => {
                let pinned = self
                    .loaded
                    .iter()
                    .flat_map(|l| &l.view.sections)
                    .any(|s| s.key == key && matches!(s.state, SectionState::Pinned { .. }));
                pages::pin(&self.store, &page, key, !pinned).map(|_| ())
            }
            SectionAction::Hide => pages::hide(&self.store, &page, key, true, now_ms()).map(|_| ()),
            SectionAction::Copy => {
                let section = self
                    .loaded
                    .iter()
                    .flat_map(|l| &l.view.sections)
                    .find(|s| s.key == key)
                    .cloned();
                if let Some(section) = section {
                    match actions::copy_into_note(
                        &self.store,
                        &page.id,
                        &section.title,
                        &pages::section_markdown(&section),
                        now_ms(),
                    ) {
                        Ok(_) => cx.emit(PageEvent::Notice("Copied into the note.".into())),
                        Err(err) => cx.emit(PageEvent::Notice(err.to_string())),
                    }
                }
                return;
            }
        };
        if let Err(err) = outcome {
            cx.emit(PageEvent::Notice(format!(
                "Couldn't change the page: {err}"
            )));
        }
        self.reload(false, cx);
    }

    fn show_hidden(&mut self, cx: &mut Context<Self>) {
        if let Some(page) = self.page() {
            let _ = pages::show_hidden(&self.store, &page, now_ms());
            self.reload(false, cx);
        }
    }

    fn not_right(&mut self, key: &str, index: usize, cx: &mut Context<Self>) {
        if let Some(page) = self.page() {
            let _ = pages::not_right(&self.store, &page, key, index);
            self.reload(false, cx);
        }
    }

    fn show_all(&mut self, key: &str, cx: &mut Context<Self>) {
        self.shown_all.insert(key.to_owned());
        self.reload(false, cx);
    }

    fn add_alias(&mut self, alias: &str, cx: &mut Context<Self>) {
        let Some(id) = self.note_id.clone() else {
            return;
        };
        if let Err(err) = actions::add_alias(&self.store, &id, alias, now_ms()) {
            cx.emit(PageEvent::Notice(err.to_string()));
        }
        self.reload(false, cx);
    }

    fn merge_into(&mut self, into: &str, cx: &mut Context<Self>) {
        let Some(id) = self.note_id.clone() else {
            return;
        };
        match actions::merge(&self.store, into, &id, now_ms()) {
            Ok(done) => cx.emit(PageEvent::Merged(done.into)),
            Err(err) => cx.emit(PageEvent::Notice(err.to_string())),
        }
    }

    /// Ticks a task a line shows, in its own note.
    fn tick(&mut self, item: &Item, checked: bool, cx: &mut Context<Self>) {
        let Some(task) = &item.task else { return };
        if cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.writes_held)
        {
            return;
        }
        match self.store.set_task_checked(
            &task.note_id,
            task.line,
            &task.text,
            checked,
            crate::clock::Clock::now_ms(cx),
            Origin::Ui,
        ) {
            Ok(CheckOff::Missing) => cx.emit(PageEvent::Notice(
                "That task changed in its note. The page is up to date now.".into(),
            )),
            Ok(_) => {}
            Err(err) => cx.emit(PageEvent::Notice(format!(
                "Couldn't change the task: {err}"
            ))),
        }
        self.reload(false, cx);
    }

    /// Opens the lines a source points at. Lines that moved since the
    /// section was written are found again; lines that are gone open the
    /// note and say so.
    fn open_source(&mut self, receipt: &Receipt, cx: &mut Context<Self>) {
        let body = self
            .store
            .note(&receipt.note_id)
            .ok()
            .flatten()
            .map(|note| note.body);
        match body.and_then(|body| relocate(&body, receipt)) {
            Some((first, last)) => cx.emit(PageEvent::OpenLines {
                id: receipt.note_id.clone(),
                first,
                last,
            }),
            None => {
                cx.emit(PageEvent::Notice(
                    kanso_engine::ask::LINES_CHANGED.to_owned(),
                ));
                cx.emit(PageEvent::OpenNote(receipt.note_id.clone()));
            }
        }
    }
}

// MARK: Drawing

impl PagePanel {
    fn render_item(
        &self,
        section: &Section,
        index: usize,
        item: &Item,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let key = section.key.clone();
        let ai = section.kind == SectionKind::Ai;
        let meta: Vec<String> = [
            item.tag.clone(),
            item.meta.clone(),
            item.shared.then(|| "shared note".to_owned()),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect();
        let who = item.who.as_deref().map(|who| match who {
            "you" => "You promised",
            _ => "They promised",
        });
        let sources: Vec<Receipt> = item.sources.clone();
        let target = item.target.clone();
        let line = item.clone();
        h_flex()
            .id(("page-item", index))
            .items_start()
            .gap(px(8.))
            .py(px(3.))
            .when_some(item.task.clone(), |row, task| {
                let ticker = view.clone();
                let item = item.clone();
                row.child(
                    Checkbox::new(("page-task", index))
                        .checked(task.checked)
                        .on_click(move |checked, _, cx| {
                            let item = item.clone();
                            let checked = *checked;
                            ticker
                                .update(cx, |this, cx| this.tick(&item, checked, cx))
                                .ok();
                        }),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .when_some(who, |col, who| {
                        col.child(
                            div()
                                .text_size(px(11.))
                                .text_color(colors.text_muted)
                                .child(who),
                        )
                    })
                    .child({
                        let opener = view.clone();
                        let target = target.clone();
                        let first = sources.first().cloned();
                        div()
                            .id(("page-text", index))
                            .text_size(px(13.5))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .when(first.is_some() || target.is_some(), |el| {
                                el.cursor_pointer()
                            })
                            .on_click(move |_, _, cx| {
                                opener
                                    .update(cx, |this, cx| {
                                        if let Some(target) = &target {
                                            cx.emit(PageEvent::OpenNote(target.clone()));
                                        } else if let Some(receipt) = &first {
                                            this.open_source(receipt, cx);
                                        }
                                    })
                                    .ok();
                            })
                            .child(line.text.clone())
                    })
                    .when(!meta.is_empty(), |col| {
                        col.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(colors.text_muted)
                                .child(meta.join(" \u{b7} ")),
                        )
                    })
                    .when(ai, |col| {
                        col.children(sources.iter().enumerate().map(|(n, receipt)| {
                            let opener = view.clone();
                            let receipt_for_click = receipt.clone();
                            div()
                                .id(("page-source", index * 16 + n))
                                .text_size(px(11.5))
                                .text_color(colors.link)
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    opener
                                        .update(cx, |this, cx| {
                                            this.open_source(&receipt_for_click, cx)
                                        })
                                        .ok();
                                })
                                .child(receipt.source_line())
                        }))
                    }),
            )
            .when(ai, |row| {
                let marker = view.clone();
                let key = key.clone();
                row.child(
                    Button::new(("page-not-right", index))
                        .ghost()
                        .xsmall()
                        .icon(Lucide::ThumbsDown)
                        .tooltip("Not right")
                        .on_click(move |_, _, cx| {
                            let key = key.clone();
                            marker
                                .update(cx, |this, cx| this.not_right(&key, index, cx))
                                .ok();
                        }),
                )
            })
            .into_any_element()
    }

    fn render_section(
        &self,
        ix: usize,
        section: &Section,
        ai_on: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let key = section.key.clone();
        let folded = self.folded.contains(&key);
        let working = self.working.contains(&key);
        let pinned = matches!(section.state, SectionState::Pinned { .. });
        let ai = section.kind == SectionKind::Ai;
        let choices = section_actions(section, ai_on);
        let menu_view = view.clone();
        let menu_key = key.clone();
        let menu = Button::new(("page-section-menu", ix))
            .ghost()
            .xsmall()
            .icon(Lucide::Ellipsis)
            .dropdown_menu(move |mut menu, _, _| {
                for action in choices.iter().copied() {
                    let (view, key) = (menu_view.clone(), menu_key.clone());
                    menu = menu.item(PopupMenuItem::new(action_label(action, pinned)).on_click(
                        move |_, _, cx| {
                            let key = key.clone();
                            view.update(cx, |this, cx| this.run(&key, action, cx)).ok();
                        },
                    ));
                }
                menu
            });
        let toggler = view.clone();
        let toggle_key = key.clone();
        let header = h_flex()
            .gap(px(6.))
            .items_center()
            .child(
                h_flex()
                    .id(("page-section-title", ix))
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        let key = toggle_key.clone();
                        toggler
                            .update(cx, |this, cx| {
                                if !this.folded.remove(&key) {
                                    this.folded.insert(key);
                                }
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(
                        Icon::new(if folded {
                            Lucide::ChevronRight
                        } else {
                            Lucide::ChevronDown
                        })
                        .size(px(12.))
                        .text_color(colors.text_muted),
                    )
                    .when(ai, |row| {
                        row.child(
                            Icon::new(Lucide::Sparkles)
                                .size(px(12.))
                                .text_color(colors.text_muted),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(section.title.clone()),
                    )
                    .when(pinned, |row| {
                        row.child(
                            Icon::new(Lucide::Pin)
                                .size(px(11.))
                                .text_color(colors.text_muted),
                        )
                    }),
            )
            .when(working, |row| {
                row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child("Writing\u{2026}"),
                )
            })
            .child(menu);

        let body = (!folded).then(|| {
            let mut group: Option<String> = None;
            let mut col = v_flex().pl(px(18.)).gap(px(1.));
            if let Some(stale) = section.stale_line() {
                col = col.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.warning)
                        .child(stale),
                );
            }
            if let Some(reason) = self.trouble.get(&key) {
                let (retry, dismiss) = (view.clone(), view.clone());
                let (retry_key, dismiss_key) = (key.clone(), key.clone());
                col = col.child(ai_error_card(
                    &ai_trouble(reason),
                    move |_, cx| {
                        let key = retry_key.clone();
                        retry.update(cx, |this, cx| this.write(vec![key], cx)).ok();
                    },
                    move |_, cx| {
                        let key = dismiss_key.clone();
                        dismiss
                            .update(cx, |this, cx| {
                                this.trouble.remove(&key);
                                cx.notify();
                            })
                            .ok();
                    },
                    window,
                    cx,
                ));
            }
            if section.items.is_empty()
                && !self.trouble.contains_key(&key)
                && let Some(line) = pages::empty_line(section)
            {
                col = col.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(line),
                );
            }
            for (index, item) in section.items.iter().enumerate() {
                if let Some(heading) = item.group.as_ref().filter(|g| group.as_ref() != Some(*g)) {
                    group = Some(heading.clone());
                    col = col.child(
                        div()
                            .pt(px(4.))
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child(heading.clone()),
                    );
                }
                col = col.child(self.render_item(section, index, item, cx));
            }
            if section.total > section.items.len() {
                let all = view.clone();
                let all_key = key.clone();
                col = col.child(
                    Button::new(("page-show-all", ix))
                        .ghost()
                        .xsmall()
                        .label(format!("Show all {}", section.total))
                        .on_click(move |_, _, cx| {
                            let key = all_key.clone();
                            all.update(cx, |this, cx| this.show_all(&key, cx)).ok();
                        }),
                );
            }
            if let Some(note) = &section.note {
                col = col.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(note.clone()),
                );
            }
            col
        });
        v_flex()
            .debug_selector(move || format!("page-section-{ix}"))
            .gap(px(4.))
            .py(px(6.))
            .child(header)
            .children(body)
            .into_any_element()
    }
}

impl Render for PagePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(loaded) = self.loaded.clone() else {
            return div().into_any_element();
        };
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let ai_on = loaded.ai_on;
        let refreshable = self.refreshable();
        let hidden = loaded.view.hidden.len();
        let working = !self.working.is_empty();

        let page_menu = {
            let (refresh, hide, alias, merge) =
                (view.clone(), view.clone(), view.clone(), view.clone());
            let store = self.store.clone();
            let this_id = self.note_id.clone().unwrap_or_default();
            let kind = loaded.header.kind;
            Button::new("page-menu")
                .ghost()
                .xsmall()
                .icon(Lucide::Ellipsis)
                .dropdown_menu(move |mut menu, _, _| {
                    if ai_on && !refreshable.is_empty() {
                        let (view, keys) = (refresh.clone(), refreshable.clone());
                        menu = menu.item(PopupMenuItem::new("Refresh all").on_click(
                            move |_, _, cx| {
                                let keys = keys.clone();
                                view.update(cx, |this, cx| this.write(keys, cx)).ok();
                            },
                        ));
                    }
                    if hidden > 0 {
                        let view = hide.clone();
                        menu = menu.item(PopupMenuItem::new("Show hidden sections").on_click(
                            move |_, _, cx| {
                                view.update(cx, |this, cx| this.show_hidden(cx)).ok();
                            },
                        ));
                    }
                    let view = alias.clone();
                    menu = menu.item(PopupMenuItem::new("Add alias\u{2026}").on_click(
                        move |_, window, cx| crate::page_view::ask_alias(view.clone(), window, cx),
                    ));
                    // Merge: one entry for each other page of the same kind.
                    let others: Vec<_> = store
                        .entities()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|p| p.id != this_id && p.kind == kind)
                        .collect();
                    if !others.is_empty() {
                        let mut sub = Vec::new();
                        for other in others {
                            sub.push((other.id, other.title));
                        }
                        for (id, title) in sub {
                            let view = merge.clone();
                            menu = menu.item(
                                PopupMenuItem::new(format!("Merge into {title}")).on_click(
                                    move |_, window, cx| {
                                        crate::page_view::confirm_merge(
                                            view.clone(),
                                            id.clone(),
                                            title.clone(),
                                            window,
                                            cx,
                                        )
                                    },
                                ),
                            );
                        }
                    }
                    menu
                })
        };

        let collapser = view.clone();
        let header = h_flex()
            .gap(px(8.))
            .items_center()
            .child(
                h_flex()
                    .id("page-panel-title")
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        collapser
                            .update(cx, |this, cx| {
                                this.collapsed = !this.collapsed;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(
                        Icon::new(if self.collapsed {
                            Lucide::ChevronRight
                        } else {
                            Lucide::ChevronDown
                        })
                        .size(px(13.))
                        .text_color(colors.text_muted),
                    )
                    .child(
                        Icon::new(match loaded.header.kind {
                            kanso_core::entities::EntityKind::Person => Lucide::User,
                            kanso_core::entities::EntityKind::Project => Lucide::Folder,
                        })
                        .size(px(13.))
                        .text_color(colors.text_muted),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(people_data::header_line(&loaded.header)),
                    ),
            )
            .when(working, |row| {
                row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child("Writing\u{2026}"),
                )
            })
            .child(page_menu);

        let sections: Vec<AnyElement> = if self.collapsed {
            Vec::new()
        } else {
            loaded
                .view
                .sections
                .iter()
                .enumerate()
                .map(|(ix, section)| self.render_section(ix, section, ai_on, window, cx))
                .collect()
        };
        let hidden_row = (!self.collapsed && hidden > 0).then(|| {
            let shower = view.clone();
            Button::new("page-show-hidden")
                .ghost()
                .xsmall()
                .label(format!(
                    "Show {hidden} hidden {}",
                    if hidden == 1 { "section" } else { "sections" }
                ))
                .on_click(move |_, _, cx| {
                    shower.update(cx, |this, cx| this.show_hidden(cx)).ok();
                })
        });
        v_flex()
            .id("page-panel")
            .debug_selector(|| "page-panel".into())
            .flex_none()
            .w_full()
            .px(px(TEXT_INSET))
            .py(px(6.))
            .border_t_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .child(header)
            .when(!self.collapsed, |panel| {
                panel.child(
                    v_flex()
                        .id("page-sections")
                        .max_h(px(MAX_HEIGHT))
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll)
                        .children(sections)
                        .children(hidden_row),
                )
            })
            .into_any_element()
    }
}

/// "Add alias…": the palette-free way, a small prompt for one more name.
pub fn ask_alias(view: WeakEntity<PagePanel>, window: &mut Window, cx: &mut App) {
    crate::dialogs::prompt_text(
        "Another name",
        "",
        "Add alias",
        window,
        cx,
        move |name, _, cx| {
            view.update(cx, |this, cx| this.add_alias(&name, cx)).ok();
        },
    );
}

/// "Merge into X": asks once, then folds this page into X.
pub fn confirm_merge(
    view: WeakEntity<PagePanel>,
    into: String,
    title: String,
    window: &mut Window,
    cx: &mut App,
) {
    crate::dialogs::confirm(
        format!("Merge into {title}?"),
        format!("This page's names and text move to {title}, and this page goes to the Trash."),
        "Merge",
        window,
        cx,
        move |_, cx| {
            let into = into.clone();
            view.update(cx, |this, cx| this.merge_into(&into, cx)).ok();
        },
    );
}
