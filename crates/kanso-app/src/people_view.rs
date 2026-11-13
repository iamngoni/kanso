//! People & Projects over the window (issue #200): the names Kanso suggests
//! pages for, and the pages there are. A suggestion becomes a page only when
//! "Create page" is pressed; "Not a person" and "Not now" turn it away; and
//! a name that may be another name of a page asks whether it is.
//!
//! What is suggested, and why, is the engine's
//! (`kanso_engine::entities`), which the phones use too. This is the
//! window's side: drawing, the model call and the notices. A page shows its
//! live sections below its note (`page_view.rs`).

use gpui_component::scroll::ScrollableElement as _;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::entities::EntityKind;
use kanso_core::now_ms;
use kanso_engine::entities::{
    EntitySettings, Suggestion, actions, ai_detect,
    suggest::{self, MIN_GAP_MS},
};
use kanso_store::Store;

use crate::ai_feature::{self, Maker};
use crate::layout::BOTTOM_BAND_HEIGHT;
use crate::library::{Library, LibraryEvent};
use crate::page_chrome::{self, ActionStyle, HeaderAction, ai_error_card, ai_trouble};
use crate::theme::Colors;
use crate::workspace::{AppState, Workspace};
use crate::zoom::px;

/// How long the library stays still before a shown view is read again.
const RELOAD_DELAY: Duration = Duration::from_millis(300);
/// How soon after launch the first look for names is made, so the start
/// is not slowed, and how often it is made again.
const FIRST_LOOK: Duration = Duration::from_secs(20);

actions!(
    kanso,
    [
        ShowPeople,
        ClosePeopleView,
        NewPerson,
        NewProject,
        FindPeopleWithAi
    ]
);

/// What the view asks of the workspace.
pub enum PeopleEvent {
    /// Open the note in the editor, closing the view.
    Open(String),
    /// Something worth telling the person.
    Notice(String),
}

/// A page in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRow {
    pub id: String,
    pub title: String,
    pub kind: EntityKind,
    pub notes: u32,
    pub detail: String,
}

/// "14 notes · last 28 Oct".
pub fn page_detail(notes: u32, last: Option<&str>) -> String {
    let mut detail = format!("{notes} {}", if notes == 1 { "note" } else { "notes" });
    if notes > 0
        && let Some(day) = last
    {
        detail.push_str(&format!(" \u{b7} last {day}"));
    }
    detail
}

/// The pages, most mentioned first.
pub fn read_pages(store: &Store) -> kanso_store::Result<Vec<PageRow>> {
    let mut out = Vec::new();
    for page in store.entities()? {
        let (notes, last) = store.entity_mention_stats(&page.id)?;
        out.push(PageRow {
            detail: page_detail(
                notes,
                kanso_engine::entities::page::date_label(last).as_deref(),
            ),
            id: page.id,
            title: page.title,
            kind: page.kind,
            notes,
        });
    }
    out.sort_by(|a, b| b.notes.cmp(&a.notes).then(a.title.cmp(&b.title)));
    Ok(out)
}

/// The words on the "this is not one" button.
pub fn not_label(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Person => "Not a person",
        EntityKind::Project => "Not a project",
    }
}

pub struct PeopleView {
    store: Arc<Store>,
    suggestions: Vec<Suggestion>,
    pages: Vec<PageRow>,
    /// Drawn on screen: only then is it read again on changes.
    pub visible: bool,
    stale: bool,
    labelling: bool,
    /// What the model said when it failed, in its own words.
    trouble: Option<String>,
    focus: FocusHandle,
    scroll: ScrollHandle,
    reload_task: Option<Task<()>>,
    ai_task: Option<Task<()>>,
    pane_width: Pixels,
    leading_inset: Pixels,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PeopleEvent> for PeopleView {}

impl Focusable for PeopleView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl PeopleView {
    pub fn new(store: Arc<Store>, library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if matches!(event, LibraryEvent::Changed(_)) {
                this.stale = true;
                if this.visible {
                    this.reload_soon(cx);
                }
            }
        })];
        PeopleView {
            store,
            suggestions: Vec::new(),
            pages: Vec::new(),
            visible: false,
            stale: true,
            labelling: false,
            trouble: None,
            focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            reload_task: None,
            ai_task: None,
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

    /// Opens the view: the lists are read again.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        self.trouble = None;
        self.reload(cx);
    }

    #[cfg(test)]
    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    fn settings(cx: &App) -> EntitySettings {
        cx.global::<AppState>().config.people_projects.clone()
    }

    /// Reads the lists again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let store = self.store.clone();
        let settings = Self::settings(cx);
        let task = cx.background_executor().spawn(async move {
            let suggestions = suggest::suggestions(&store, &settings)?;
            let pages = read_pages(&store)?;
            Ok::<_, kanso_store::StoreError>((suggestions, pages))
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let read = task.await;
            this.update(cx, |this, cx| {
                match read {
                    Ok((suggestions, pages)) => {
                        this.suggestions = suggestions;
                        this.pages = pages;
                    }
                    Err(err) => eprintln!("kanso: people: {err}"),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the lists now, on this thread: for tests.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let settings = Self::settings(cx);
        self.suggestions = suggest::suggestions(&self.store, &settings).unwrap_or_default();
        self.pages = read_pages(&self.store).unwrap_or_default();
        cx.notify();
    }

    fn create(&mut self, name: &str, kind: EntityKind, aliases: &[String], cx: &mut Context<Self>) {
        let settings = Self::settings(cx);
        match actions::create_page(&self.store, &settings, kind, name, aliases, now_ms()) {
            Ok(id) => {
                self.reload(cx);
                cx.emit(PeopleEvent::Open(id));
            }
            Err(err) => cx.emit(PeopleEvent::Notice(err.to_string())),
        }
    }

    /// Turns a suggestion away, for tests.
    #[cfg(test)]
    pub fn dismiss_for_test(
        &mut self,
        suggestion: &Suggestion,
        never: bool,
        cx: &mut Context<Self>,
    ) {
        self.dismiss(suggestion, never, cx);
    }

    fn create_from(&mut self, suggestion: &Suggestion, cx: &mut Context<Self>) {
        self.create(&suggestion.name, suggestion.kind, &[], cx);
    }

    fn dismiss(&mut self, suggestion: &Suggestion, never: bool, cx: &mut Context<Self>) {
        let settings = Self::settings(cx);
        if let Err(err) = suggest::dismiss(&self.store, &settings, suggestion, never, now_ms()) {
            cx.emit(PeopleEvent::Notice(err.to_string()));
        }
        self.reload(cx);
    }

    /// "Yes, add alias": the name becomes another name of the page.
    fn confirm_alias(&mut self, suggestion: &Suggestion, cx: &mut Context<Self>) {
        let Some((page, _)) = &suggestion.alias_of else {
            return;
        };
        if let Err(err) = actions::add_alias(&self.store, page, &suggestion.name, now_ms()) {
            cx.emit(PeopleEvent::Notice(err.to_string()));
        }
        self.reload(cx);
    }

    /// Asks the model which suggestions are people or projects. It never
    /// makes a page.
    pub fn label_with_ai(&mut self, cx: &mut Context<Self>) {
        let config = cx.global::<AppState>().config.clone();
        if !config.ai.enabled || !config.people_projects.detect_with_ai || self.labelling {
            return;
        }
        let maker: Maker = ai_feature::maker(cx);
        let store = self.store.clone();
        self.labelling = true;
        self.trouble = None;
        let task = cx.background_executor().spawn(async move {
            let shared = maker()?;
            let provider = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            ai_detect::run(&store, provider.as_ref()).map_err(|err| err.to_string())
        });
        self.ai_task = Some(cx.spawn(async move |this, cx| {
            let done = task.await;
            this.update(cx, |this, cx| {
                this.labelling = false;
                if let Err(reason) = done {
                    this.trouble = Some(reason);
                } else {
                    AppState::update_config(cx, |config| {
                        config.people_projects.last_ai_detect = now_ms()
                    });
                }
                this.reload(cx);
            })
            .ok();
        }));
    }

    /// Whether the model may be asked to label again today.
    pub fn can_label(&self, cx: &App) -> bool {
        let config = &cx.global::<AppState>().config;
        config.ai.enabled
            && config.people_projects.detect_with_ai
            && !self.labelling
            && !self.suggestions.is_empty()
            && (config.people_projects.last_ai_detect == 0
                || now_ms() - config.people_projects.last_ai_detect >= ai_detect::MIN_GAP_MS)
    }
}

// MARK: Drawing

impl PeopleView {
    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = if self.pages.is_empty() {
            "People & Projects".to_owned()
        } else {
            format!("People & Projects \u{b7} {}", self.pages.len())
        };
        let label_ai = cx.entity().downgrade();
        let can_label = self.can_label(cx);
        let actions = vec![
            HeaderAction::new(
                "people-new-person",
                Lucide::UserPlus,
                "New person",
                |window, cx| window.dispatch_action(Box::new(NewPerson), cx),
            )
            .tooltip("Make a page for a person"),
            HeaderAction::new(
                "people-new-project",
                Lucide::Briefcase,
                "New project",
                |window, cx| window.dispatch_action(Box::new(NewProject), cx),
            )
            .tooltip("Make a page for a project"),
            HeaderAction::new(
                "people-label-ai",
                Lucide::Sparkles,
                "Use AI to find",
                move |_, cx| {
                    label_ai.update(cx, |this, cx| this.label_with_ai(cx)).ok();
                },
            )
            .tooltip("Your model labels the suggestions as people, projects or neither. Turn it on in Settings.")
            .disabled(!can_label)
            .loading(self.labelling)
            .style(ActionStyle::Outline),
        ];
        let close = HeaderAction::new("people-close", Lucide::Close, "Close", |window, cx| {
            window.dispatch_action(Box::new(ClosePeopleView), cx)
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

    fn render_suggestion(&self, ix: usize, s: &Suggestion, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let asking_alias = s.alias_of.clone();
        let buttons: Vec<AnyElement> = if let Some((_, title)) = asking_alias {
            let (yes, no) = (view.clone(), view.clone());
            let (yes_s, no_s) = (s.clone(), s.clone());
            vec![
                div()
                    .text_size(px(13.))
                    .text_color(colors.text_secondary)
                    .child(format!(
                        "Is \u{201c}{}\u{201d} the same as {title}?",
                        s.name
                    ))
                    .into_any_element(),
                Button::new(("people-alias-yes", ix))
                    .primary()
                    .small()
                    .label("Yes, add alias")
                    .on_click(move |_, _, cx| {
                        let s = yes_s.clone();
                        yes.update(cx, |this, cx| this.confirm_alias(&s, cx)).ok();
                    })
                    .into_any_element(),
                Button::new(("people-alias-no", ix))
                    .outline()
                    .small()
                    .label("No, different")
                    .on_click(move |_, _, cx| {
                        let s = no_s.clone();
                        no.update(cx, |this, cx| this.dismiss(&s, false, cx)).ok();
                    })
                    .into_any_element(),
            ]
        } else {
            let (create, never, later) = (view.clone(), view.clone(), view.clone());
            let (create_s, never_s, later_s) = (s.clone(), s.clone(), s.clone());
            vec![
                Button::new(("people-create", ix))
                    .primary()
                    .small()
                    .label("Create page")
                    .on_click(move |_, _, cx| {
                        let s = create_s.clone();
                        create.update(cx, |this, cx| this.create_from(&s, cx)).ok();
                    })
                    .into_any_element(),
                Button::new(("people-never", ix))
                    .outline()
                    .small()
                    .label(not_label(s.kind))
                    .on_click(move |_, _, cx| {
                        let s = never_s.clone();
                        never.update(cx, |this, cx| this.dismiss(&s, true, cx)).ok();
                    })
                    .into_any_element(),
                Button::new(("people-later", ix))
                    .ghost()
                    .small()
                    .label("Not now")
                    .on_click(move |_, _, cx| {
                        let s = later_s.clone();
                        later
                            .update(cx, |this, cx| this.dismiss(&s, false, cx))
                            .ok();
                    })
                    .into_any_element(),
            ]
        };
        v_flex()
            .debug_selector(move || format!("people-suggestion-{ix}"))
            .w_full()
            .gap(px(8.))
            .p(px(14.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(s.name.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_muted)
                            .child(s.detail()),
                    )
                    .children(s.also_line().map(|line| {
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_muted)
                            .child(line)
                    })),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .flex_wrap()
                    .items_center()
                    .children(buttons),
            )
            .into_any_element()
    }

    fn render_page(&self, ix: usize, page: &PageRow, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let id = page.id.clone();
        h_flex()
            .id(("people-page", ix))
            .gap(px(10.))
            .px(px(14.))
            .py(px(9.))
            .cursor_pointer()
            .hover(|el| el.bg(colors.hover))
            .child(
                Icon::new(match page.kind {
                    EntityKind::Person => Lucide::User,
                    EntityKind::Project => Lucide::Folder,
                })
                .size(px(15.))
                .text_color(colors.text_secondary),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(13.5))
                    .text_color(colors.text)
                    .child(page.title.clone()),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(page.detail.clone()),
            )
            .on_click(cx.listener(move |_, _, _, cx| cx.emit(PeopleEvent::Open(id.clone()))))
            .into_any_element()
    }
}

impl Render for PeopleView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.visible && self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let settings = Self::settings(cx);
        let view = cx.entity().downgrade();
        let body = if self.suggestions.is_empty() && self.pages.is_empty() {
            let message = if settings.suggest {
                "Kanso suggests a page when a name keeps coming up in your notes, and nothing becomes a page until you say so. You can also make one yourself."
            } else {
                "Suggestions are off. Turn them on in Settings, or make a page yourself."
            };
            div()
                .id("people-body")
                .size_full()
                .overflow_y_scrollbar()
                .child(page_chrome::centred(page_chrome::column().child(
                    page_chrome::page_empty(
                        Lucide::Users,
                        "No people or projects yet",
                        message,
                        vec![
                            Button::new("people-empty-person")
                                .primary()
                                .label("New person")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(NewPerson), cx)
                                })
                                .into_any_element(),
                            Button::new("people-empty-project")
                                .outline()
                                .label("New project")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(NewProject), cx)
                                })
                                .into_any_element(),
                        ],
                        cx,
                    ),
                )))
                .into_any_element()
        } else {
            let suggestions: Vec<AnyElement> = self
                .suggestions
                .iter()
                .enumerate()
                .map(|(ix, s)| self.render_suggestion(ix, s, cx))
                .collect();
            let pages: Vec<AnyElement> = self
                .pages
                .iter()
                .enumerate()
                .map(|(ix, p)| self.render_page(ix, p, cx))
                .collect();
            let new_label = suggest::count_label(self.suggestions.len());
            let trouble = self.trouble.clone().map(|raw| {
                let (retry, dismiss) = (view.clone(), view.clone());
                ai_error_card(
                    &ai_trouble(&raw),
                    move |_, cx| {
                        retry.update(cx, |this, cx| this.label_with_ai(cx)).ok();
                    },
                    move |_, cx| {
                        dismiss
                            .update(cx, |this, cx| {
                                this.trouble = None;
                                cx.notify();
                            })
                            .ok();
                    },
                    window,
                    cx,
                )
            });
            div()
                .id("people-body")
                .size_full()
                .overflow_y_scrollbar()
                .track_scroll(&self.scroll)
                .child(page_chrome::centred(
                    page_chrome::column()
                        .gap(px(12.))
                        .pt(px(16.))
                        .children(trouble)
                        .when(!suggestions.is_empty(), |col| {
                            col.child(
                                h_flex()
                                    .gap(px(8.))
                                    .child(
                                        Icon::new(Lucide::Sparkles)
                                            .size(px(14.))
                                            .text_color(colors.text_muted),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_semibold()
                                            .text_color(colors.text)
                                            .child("Suggestions"),
                                    )
                                    .children(new_label.map(|label| {
                                        div()
                                            .px(px(6.))
                                            .rounded(px(8.))
                                            .bg(colors.fill)
                                            .text_size(px(11.5))
                                            .text_color(colors.text_secondary)
                                            .child(label)
                                    })),
                            )
                            .children(suggestions)
                        })
                        .when(!pages.is_empty(), |col| {
                            col.child(page_chrome::section(
                                Lucide::Users,
                                "Pages",
                                Some(pages.len()),
                                pages,
                                None,
                                cx,
                            ))
                        }),
                ))
                .into_any_element()
        };
        v_flex()
            .id("people-view")
            .key_context("PeopleView")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified() {
                    cx.stop_propagation();
                    window.dispatch_action(Box::new(ClosePeopleView), cx);
                }
            }))
            // Presses here are the view's, not the window's below.
            .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
            .child(self.render_toolbar(window, cx))
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
                        "Nothing becomes a page, and nothing is merged, until you say so. Esc closes.",
                    )),
            )
    }
}

// MARK: Detection

/// Looks for names worth a page, now and every ten minutes, on a worker.
/// It runs only while suggestions are on, reads only the library, and makes
/// nothing. The sidebar's count follows from the library reloading when it
/// finishes.
pub fn start_detection(library: Entity<Library>, store: Arc<Store>, cx: &mut App) {
    cx.spawn(async move |cx| {
        cx.background_executor().timer(FIRST_LOOK).await;
        let mut last_run = 0_i64;
        loop {
            let settings = cx.update(|cx| cx.global::<AppState>().config.people_projects.clone());
            if settings.suggest {
                let task = cx.background_executor().spawn({
                    let store = store.clone();
                    async move {
                        let mut last = last_run;
                        let ran = suggest::refresh(&store, &settings, now_ms(), &mut last, false);
                        (last, ran)
                    }
                });
                let (last, ran) = task.await;
                last_run = last;
                if matches!(ran, Ok(Some(_))) {
                    cx.update(|cx| library.update(cx, |library, cx| library.reload(cx)));
                }
            }
            cx.background_executor()
                .timer(Duration::from_millis(MIN_GAP_MS as u64))
                .await;
        }
    })
    .detach();
}

// MARK: Workspace

/// The workspace's part: opening and closing the view, and making pages.
impl Workspace {
    pub(crate) fn show_people(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.close_board(window, cx);
        self.close_schedule(window, cx);
        self.close_triage(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let people = match &self.people {
            Some(people) => people.clone(),
            None => {
                let store = self.library.read(cx).store.clone();
                let library = self.library.clone();
                let people = cx.new(|cx| PeopleView::new(store, library, cx));
                self._people_events = Some(cx.subscribe_in(
                    &people,
                    window,
                    |this, _, event, window, cx| this.on_people_event(event, window, cx),
                ));
                self.people = Some(people.clone());
                people
            }
        };
        people.update(cx, |people, cx| people.show(cx));
        self.people_open = true;
        window.focus(&people.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_people(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.people_open {
            return;
        }
        self.people_open = false;
        if let Some(people) = &self.people {
            people.update(cx, |people, _| people.visible = false);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_people_event(
        &mut self,
        event: &PeopleEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            PeopleEvent::Open(id) => {
                self.close_people(window, cx);
                let id = id.clone();
                self.library
                    .update(cx, |library, cx| library.select(Some(id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }
            PeopleEvent::Notice(message) => {
                window.push_notification(
                    gpui_kit::component::notification::Notification::info(message.clone()),
                    cx,
                );
            }
        }
    }

    /// "New Person…" / "New Project…": asks for a name, then makes the page
    /// and opens it.
    pub(crate) fn new_entity_page(
        &mut self,
        kind: EntityKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = match kind {
            EntityKind::Person => "New person",
            EntityKind::Project => "New project",
        };
        let workspace = cx.entity().downgrade();
        crate::dialogs::prompt_text(title, "", "Create", window, cx, move |name, window, cx| {
            let settings = cx.global::<AppState>().config.people_projects.clone();
            let store = cx.global::<AppState>().store.clone();
            // "Sam Moyo, Sam" makes the page for the first name and lists
            // the others as its other names.
            let mut names = name.split(',').map(str::trim).filter(|n| !n.is_empty());
            let Some(first) = names.next() else { return };
            let aliases: Vec<String> = names.map(str::to_owned).collect();
            match actions::create_page(&store, &settings, kind, first, &aliases, now_ms()) {
                Ok(id) => {
                    workspace
                        .update(cx, |this, cx| {
                            this.close_people(window, cx);
                            this.library
                                .update(cx, |library, cx| library.select(Some(id), cx));
                            this.editor
                                .update(cx, |editor, cx| editor.focus_body(window, cx));
                        })
                        .ok();
                }
                Err(err) => window.push_notification(
                    gpui_kit::component::notification::Notification::error(err.to_string()),
                    cx,
                ),
            }
        });
    }
}

/// The workspace's actions for People & Projects.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowPeople, window, cx| this.show_people(window, cx)))
        .on_action(
            cx.listener(|this, _: &ClosePeopleView, window, cx| this.close_people(window, cx)),
        )
        .on_action(cx.listener(|this, _: &NewPerson, window, cx| {
            this.new_entity_page(EntityKind::Person, window, cx)
        }))
        .on_action(cx.listener(|this, _: &NewProject, window, cx| {
            this.new_entity_page(EntityKind::Project, window, cx)
        }))
        .on_action(cx.listener(|this, _: &FindPeopleWithAi, _, cx| {
            if let Some(people) = this.people.clone() {
                people.update(cx, |people, cx| people.label_with_ai(cx));
            }
        }))
}

/// People & Projects over the window, while open.
pub fn overlay(workspace: &Workspace, leading: f32) -> Option<impl IntoElement> {
    let people = workspace.people.clone().filter(|_| workspace.people_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left(px(leading))
            .right_0()
            .bottom_0()
            .debug_selector(|| "people-overlay".into())
            .child(people),
    )
}

#[cfg(test)]
mod tests {
    use super::{not_label, page_detail};
    use kanso_core::entities::EntityKind;

    // no-prop-test: three words over plain numbers.
    #[test]
    fn a_page_says_how_many_notes_and_when() {
        assert_eq!(page_detail(1, Some("28 Oct")), "1 note \u{b7} last 28 Oct");
        assert_eq!(page_detail(14, None), "14 notes");
        assert_eq!(page_detail(0, Some("28 Oct")), "0 notes");
    }

    // no-prop-test: a fixed list of the icons these views draw.
    #[test]
    fn every_icon_the_people_views_draw_is_bundled() {
        use gpui_kit::AssetSource as _;
        use gpui_kit::assets::IconName as Lucide;
        let icons = [
            Lucide::ChevronDown,
            Lucide::ChevronRight,
            Lucide::Close,
            Lucide::Ellipsis,
            Lucide::EyeOff,
            Lucide::Folder,
            Lucide::FolderOpen,
            Lucide::Briefcase,
            Lucide::Pin,
            Lucide::RefreshCw,
            Lucide::Sparkles,
            Lucide::ThumbsDown,
            Lucide::User,
            Lucide::UserPlus,
            Lucide::Users,
        ];
        for icon in icons {
            let path = icon.path();
            assert!(
                matches!(crate::assets::KansoAssets.load(&path), Ok(Some(_))),
                "{path}"
            );
        }
    }

    // no-prop-test: two labels.
    #[test]
    fn the_not_button_names_the_kind() {
        assert_eq!(not_label(EntityKind::Person), "Not a person");
        assert_eq!(not_label(EntityKind::Project), "Not a project");
    }
}
