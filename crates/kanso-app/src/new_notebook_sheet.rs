//! New Notebook (issue #390, slice 29): a name, and what the notebook starts
//! as: a blank notebook, or a Video studio with its stage board, dates and
//! two templates. The rules that make one are
//! `kanso_engine::presets::create_notebook`'s; this is the sheet.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Icon, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::presets::{self, NotebookKind};
use kanso_store::Origin;

use crate::library::{Filter, Library};
use crate::theme::Colors;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 480.;

/// The icon beside each choice.
fn kind_icon(kind: NotebookKind) -> IconName {
    match kind {
        NotebookKind::Blank => IconName::Notebook,
        NotebookKind::VideoStudio => IconName::Clapperboard,
    }
}

/// The body of the sheet: the choice, then the name.
pub struct NewNotebookSheet {
    library: Entity<Library>,
    parent: Option<String>,
    pub(crate) kind: NotebookKind,
    pub(crate) name: Entity<InputState>,
    pub(crate) error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl NewNotebookSheet {
    fn new(
        library: Entity<Library>,
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
        NewNotebookSheet {
            library,
            parent,
            kind: NotebookKind::Blank,
            name,
            error: None,
            _subscriptions: subscriptions,
        }
    }

    /// The name as it will be used: without the spaces around it.
    pub(crate) fn typed_name(&self, cx: &App) -> String {
        self.name.read(cx).value().trim().to_owned()
    }

    /// Chooses what the notebook starts as.
    pub fn set_kind(&mut self, kind: NotebookKind, cx: &mut Context<Self>) {
        self.kind = kind;
        cx.notify();
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
        if name.is_empty() {
            return;
        }
        let store = self.library.read(cx).store.clone();
        match presets::create_notebook(&store, &name, self.parent.clone(), self.kind, Origin::Ui) {
            Ok(made) => {
                crate::metrics::record(cx, crate::metrics::Feature::NotebookCreated);
                self.library.update(cx, |library, cx| {
                    library.set_filter(Filter::Book(made.book_id), cx)
                });
                window.close_dialog(cx);
            }
            Err(err) => {
                self.error = Some(err.to_string());
                cx.notify();
            }
        }
    }
}

// no-prop-test: drawing; the UI tests click each choice, and the snapshots
// show the sheet in both themes.
impl Render for NewNotebookSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let accent = Hsla::from(colors.accent);
        let cards = NotebookKind::ALL.into_iter().enumerate().map(|(ix, kind)| {
            let selected = kind == self.kind;
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
                .when(!selected, |el| {
                    el.border_color(colors.divider)
                        .hover(|el| el.bg(colors.row_hover))
                })
                .cursor_pointer()
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
                        ),
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
                .on_click(cx.listener(move |this, _, _, cx| this.set_kind(kind, cx)))
        });
        v_flex()
            .gap(px(14.))
            .pb(px(4.))
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
                    .children(cards),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text_secondary)
                            .child("Name"),
                    )
                    .child(
                        div()
                            .debug_selector(|| "notebook-name".into())
                            .child(Input::new(&self.name)),
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
    parent: Option<String>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<NewNotebookSheet> {
    let sheet = cx.new(|cx| NewNotebookSheet::new(library, parent, window, cx));
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
