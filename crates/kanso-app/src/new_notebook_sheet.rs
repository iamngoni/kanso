//! New Notebook (issues #390 slice 29 and #514): a name, and what the
//! notebook starts as: a blank notebook, a starter (sub-notebooks, tags,
//! templates and a first note in each) or a Video studio. The rules that make
//! one are `kanso_engine::presets::create_notebook`'s; this is the sheet. It
//! lays out from `NotebookKind::ALL` and the engine's catalogue, so the
//! preview a card shows is what Create makes.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::presets::{self, NotebookKind, Starter};
use kanso_engine::theme::muted_tag_color;
use kanso_store::Origin;

use crate::book_icons;
use crate::editor::rgb_color;
use crate::library::{Filter, Library};
use crate::sidebar::Sidebar;
use crate::theme::Colors;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 560.;

/// The tallest the list of starters gets before it scrolls.
pub const LIST_MAX_HEIGHT: f32 = 420.;

/// What a card under a disabled starter says.
pub const LOCKED_HINT: &str =
    "Starters can't be made inside a locked notebook. Make it at the top level, then move it in.";

/// The icon on a starter's tile: its catalogue icon, drawn by Lucide.
fn kind_icon(kind: NotebookKind) -> IconName {
    book_icons::icon_of(kind.icon()).unwrap_or(book_icons::DEFAULT)
}

/// The body of the sheet: the name, then the choice.
pub struct NewNotebookSheet {
    library: Entity<Library>,
    sidebar: Entity<Sidebar>,
    parent: Option<String>,
    pub(crate) kind: NotebookKind,
    pub(crate) name: Entity<InputState>,
    pub(crate) error: Option<String>,
    /// Each choice with what it makes beside the notebook, as of the day the
    /// sheet opened.
    choices: Vec<(NotebookKind, Option<Starter>)>,
    list_focus: FocusHandle,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl NewNotebookSheet {
    fn new(
        library: Entity<Library>,
        sidebar: Entity<Sidebar>,
        parent: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Notebook name"));
        let subscriptions = vec![cx.subscribe_in(
            &name,
            window,
            |this: &mut NewNotebookSheet, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.create(window, cx),
                InputEvent::Change => {
                    this.error = None;
                    cx.notify();
                }
                _ => {}
            },
        )];
        let today = chrono::Local::now().date_naive();
        NewNotebookSheet {
            library,
            sidebar,
            parent,
            kind: NotebookKind::Blank,
            name,
            error: None,
            choices: NotebookKind::ALL
                .into_iter()
                .map(|kind| (kind, kind.starter(today)))
                .collect(),
            list_focus: cx.focus_handle().tab_stop(true),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// The name as it will be used: without the spaces around it.
    pub(crate) fn typed_name(&self, cx: &App) -> String {
        self.name.read(cx).value().trim().to_owned()
    }

    /// Whether the notebook goes inside a locked notebook.
    pub(crate) fn parent_locked(&self, cx: &App) -> bool {
        self.parent
            .as_deref()
            .is_some_and(|parent| self.library.read(cx).is_book_locked(parent))
    }

    /// Whether `kind` can't be chosen: it makes notes, and a locked notebook
    /// takes none that aren't sealed.
    pub(crate) fn disabled(&self, kind: NotebookKind, cx: &App) -> bool {
        kind.makes_notes() && self.parent_locked(cx)
    }

    /// Chooses what the notebook starts as, unless it can't go here.
    pub fn set_kind(&mut self, kind: NotebookKind, cx: &mut Context<Self>) {
        if self.disabled(kind, cx) {
            return;
        }
        self.kind = kind;
        cx.notify();
    }

    /// Moves the choice up or down one, past any that can't be chosen.
    fn step(&mut self, down: bool, cx: &mut Context<Self>) {
        let all = NotebookKind::ALL;
        let Some(at) = all.iter().position(|kind| *kind == self.kind) else {
            return;
        };
        let order: Vec<NotebookKind> = if down {
            all[at + 1..].to_vec()
        } else {
            all[..at].iter().rev().copied().collect()
        };
        if let Some(next) = order.into_iter().find(|kind| !self.disabled(*kind, cx)) {
            self.set_kind(next, cx);
            if let Some(ix) = all.iter().position(|kind| *kind == next) {
                self.scroll.scroll_to_item(ix);
            }
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let mods = &event.keystroke.modifiers;
        if mods.control || mods.alt || mods.platform || mods.shift {
            return;
        }
        match event.keystroke.key.as_str() {
            "down" => self.step(true, cx),
            "up" => self.step(false, cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    /// Types `value` as the name (tests and snapshots).
    #[cfg(test)]
    pub fn set_name(&mut self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = value.to_owned();
        self.name
            .update(cx, |input, cx| input.set_value(value, window, cx));
        cx.notify();
    }

    /// Presses Create: makes the notebook and goes to it, or says why not.
    pub fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.typed_name(cx);
        if name.is_empty() || self.disabled(self.kind, cx) {
            return;
        }
        let store = self.library.read(cx).store.clone();
        match presets::create_notebook(&store, &name, self.parent.clone(), self.kind, Origin::Ui) {
            Ok(made) => {
                crate::metrics::record(cx, crate::metrics::Feature::NotebookCreated);
                let first_note = made.note_ids.first().cloned();
                if !made.child_ids.is_empty() {
                    self.sidebar
                        .update(cx, |sidebar, cx| sidebar.open_book(&made.book_id, cx));
                }
                self.library.update(cx, |library, cx| {
                    // The first note stays the open one, not the list's first.
                    library.leave_page(Filter::Book(made.book_id), first_note.is_some(), cx);
                    if let Some(note) = first_note {
                        library.select(Some(note), cx);
                    }
                });
                window.close_dialog(cx);
            }
            Err(err) => {
                self.error = Some(err.to_string());
                cx.notify();
            }
        }
    }

    /// One starter's card.
    fn card(
        &self,
        ix: usize,
        kind: NotebookKind,
        starter: Option<&Starter>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let accent = Hsla::from(colors.accent);
        let selected = kind == self.kind;
        let disabled = self.disabled(kind, cx);
        let books: Vec<String> = starter
            .map(|s| s.children.iter().map(|b| b.name.clone()).collect())
            .unwrap_or_default();
        let tags: Vec<_> = starter.map(|s| s.tags.clone()).unwrap_or_default();
        h_flex()
            .id(("notebook-kind", ix))
            .debug_selector(move || format!("notebook-kind-{ix}"))
            .items_start()
            .gap(px(12.))
            .px(px(14.))
            .py(px(12.))
            .rounded(px(10.))
            .border_1()
            .when(selected, |el| {
                el.border_color(accent).bg(accent.opacity(0.08))
            })
            .when(!selected, |el| el.border_color(colors.divider))
            .when(disabled, |el| el.opacity(0.5))
            .when(!disabled, |el| {
                el.cursor_pointer()
                    .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
            })
            .child(
                div()
                    .flex_none()
                    .size(px(32.))
                    .rounded(px(8.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(accent.opacity(0.16))
                    .text_color(colors.accent)
                    .child(Icon::new(kind_icon(kind)).size(px(16.))),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text)
                            .child(kind.label()),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(17.))
                            .text_color(colors.text_secondary)
                            .child(kind.description()),
                    )
                    .when(disabled, |el| {
                        el.child(
                            div()
                                .debug_selector(move || format!("notebook-kind-hint-{ix}"))
                                .text_size(px(12.5))
                                .line_height(px(17.))
                                .text_color(colors.danger)
                                .child(LOCKED_HINT),
                        )
                    })
                    .when(!books.is_empty(), |el| {
                        el.child(
                            h_flex()
                                .items_start()
                                .gap(px(6.))
                                .mt(px(3.))
                                .child(
                                    div().flex_none().mt(px(2.)).child(
                                        Icon::new(IconName::Notebook)
                                            .size(px(12.))
                                            .text_color(colors.text_muted),
                                    ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(px(12.))
                                        .line_height(px(16.))
                                        .text_color(colors.text_muted)
                                        .child(books.join(" \u{b7} ")),
                                ),
                        )
                    })
                    .when(!tags.is_empty(), |el| {
                        el.child(
                            h_flex()
                                .items_start()
                                .gap(px(6.))
                                .child(
                                    div().flex_none().mt(px(3.)).child(
                                        Icon::new(IconName::Tag)
                                            .size(px(12.))
                                            .text_color(colors.text_muted),
                                    ),
                                )
                                .child(
                                    h_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .flex_wrap()
                                        .gap(px(4.))
                                        .children(tags.iter().map(|tag| {
                                            let tint =
                                                rgb_color(muted_tag_color(tag.color.rgb(), dark));
                                            div()
                                                .px(px(6.))
                                                .py(px(1.))
                                                .rounded(px(5.))
                                                .text_size(px(11.5))
                                                .text_color(tint)
                                                .bg(tint.opacity(0.14))
                                                .child(tag.name)
                                        })),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .mt(px(2.))
                    .size(px(16.))
                    .rounded_full()
                    .border_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(selected, |el| {
                        el.border_color(accent).bg(accent).child(
                            Icon::new(IconName::Check)
                                .size(px(11.))
                                .text_color(colors.button_text),
                        )
                    })
                    .when(!selected, |el| el.border_color(colors.divider)),
            )
            .when(!disabled, |el| {
                el.on_click(cx.listener(move |this, _, window, cx| {
                    this.set_kind(kind, cx);
                    window.focus(&this.list_focus, cx);
                }))
            })
    }
}

// no-prop-test: drawing; the UI tests click each choice, and the snapshots
// show the sheet in both themes.
impl Render for NewNotebookSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let choices = self.choices.clone();
        let cards: Vec<_> = choices
            .iter()
            .enumerate()
            .map(|(ix, (kind, starter))| self.card(ix, *kind, starter.as_ref(), cx))
            .collect();
        v_flex()
            .gap(px(14.))
            .pb(px(4.))
            .child(
                div()
                    .debug_selector(|| "notebook-name".into())
                    .child(Input::new(&self.name)),
            )
            .child(
                v_flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text_secondary)
                            .child("Start from"),
                    )
                    .child(
                        div()
                            .relative()
                            .child(
                                v_flex()
                                    .id("notebook-starters")
                                    .debug_selector(|| "notebook-starters".into())
                                    .track_focus(&self.list_focus)
                                    .on_key_down(
                                        cx.listener(|this, event, _, cx| {
                                            this.on_key_down(event, cx)
                                        }),
                                    )
                                    .track_scroll(&self.scroll)
                                    .max_h(px(LIST_MAX_HEIGHT))
                                    .overflow_y_scroll()
                                    .gap(px(8.))
                                    .children(cards),
                            )
                            .child(
                                Scrollbar::vertical(&self.scroll)
                                    .mode(crate::themes::list_scrollbar_mode(cx)),
                            ),
                    ),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .debug_selector(|| "notebook-error".into())
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// The sheet's buttons.
struct NewNotebookFooter {
    sheet: Entity<NewNotebookSheet>,
    _observing: Subscription,
}

// no-prop-test: drawing; the UI tests press Create with and without a name.
impl Render for NewNotebookFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ready = !self.sheet.read(cx).typed_name(cx).is_empty();
        let target = self.sheet.clone();
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(
                Button::new("notebook-cancel")
                    .label("Cancel")
                    .debug_selector(|| "notebook-cancel".into())
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(
                Button::new("notebook-create")
                    .primary()
                    .label("Create")
                    .disabled(!ready)
                    .debug_selector(|| "notebook-create".into())
                    .on_click(move |_, window, cx| {
                        target.update(cx, |sheet, cx| sheet.create(window, cx));
                    }),
            )
    }
}

/// Opens New Notebook, inside `parent` when given. Returns the sheet, for
/// tests to drive.
// no-prop-test: opens a dialog; the engine's preset has the properties.
pub fn open(
    library: Entity<Library>,
    sidebar: Entity<Sidebar>,
    parent: Option<String>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<NewNotebookSheet> {
    let sheet = cx.new(|cx| NewNotebookSheet::new(library, sidebar, parent, window, cx));
    let footer = cx.new(|cx| NewNotebookFooter {
        sheet: sheet.clone(),
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
    });
    let name = sheet.read(cx).name.clone();
    let first_build = std::cell::Cell::new(true);
    let body = sheet.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            name.update(cx, |input, cx| input.focus(window, cx));
        }
        alert
            .title("New Notebook")
            .width(px(SHEET_WIDTH))
            .child(body.clone())
            .footer(footer.clone())
    });
    sheet
}
