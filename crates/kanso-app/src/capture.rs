//! Quick capture: a small window over every other app for jotting a note
//! down, opened by the global shortcut (`hotkey`) or the Quick Capture
//! command. It has a Markdown text area and a notebook picker that starts
//! at the Inbox. ⌘↩ saves the note and closes it; Escape closes it, asking
//! first when there is text.
//!
//! On macOS it is a panel that takes the keyboard without bringing Kanso
//! forward, so closing it leaves you in the app you were in; it works with
//! the main window closed. On Windows it is a topmost tool window, and on
//! Linux an ordinary small window (X11 gives pop-ups no keyboard).
//!
//! The note's title is its first line ([`split_capture`]).

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, InputEvent};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::roles::Role;
use kanso_store::{Origin, Store};

use crate::config::ViewMode;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

// The title and the save are the engine's, which the phones' share sheets
// use as well.
pub use kanso_engine::capture::{save_capture, split_capture};

actions!(kanso, [SaveCapture, CancelCapture]);

/// The window's key context.
pub const CONTEXT: &str = "QuickCapture";
/// The window's size in points.
pub const WIDTH: f32 = 560.;
pub const HEIGHT: f32 = 300.;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // Over the text area's own ⌘↩, which would add a line.
        KeyBinding::new("secondary-enter", SaveCapture, Some("QuickCapture > Input")),
        KeyBinding::new("secondary-enter", SaveCapture, Some(CONTEXT)),
        KeyBinding::new("secondary-s", SaveCapture, Some(CONTEXT)),
        KeyBinding::new("escape", CancelCapture, Some(CONTEXT)),
    ]);
    // From the palette and the File menu too, though the system-wide
    // shortcut is the usual way in.
    cx.on_action(|_: &crate::journal::QuickCapture, cx| open(cx));
}

pub struct CaptureView {
    store: Arc<Store>,
    body: Entity<EditorState>,
    /// The notebooks to choose from, `(id, name)`, the Inbox first.
    books: Vec<(String, String)>,
    /// `None`: the Inbox.
    book: Option<String>,
    /// Escape was pressed over text: asking whether to discard it.
    confirming: bool,
    error: Option<String>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl CaptureView {
    pub fn new(store: Arc<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(false)
                .folding(false)
                .indent_guides(false)
                .soft_wrap(true)
                .auto_close(false)
                .placeholder("Jot something down…")
        });
        let inbox = store.find_role_book(Role::Inbox).ok().flatten();
        let mut inbox_name = Role::Inbox.default_name().to_owned();
        let mut books: Vec<(String, String)> = Vec::new();
        for book in store.books().unwrap_or_default() {
            if Some(&book.id) == inbox.as_ref() {
                inbox_name = book.name;
            } else {
                books.push((book.id, book.name));
            }
        }
        books.sort_by_key(|(_, name)| name.to_lowercase());
        books.insert(0, (Role::Inbox.book_id().to_owned(), inbox_name));
        let subscriptions = vec![
            cx.subscribe_in(&body, window, |this, _, event, window, cx| match event {
                InputEvent::Change => {
                    if this.confirming || this.error.is_some() {
                        this.confirming = false;
                        this.error = None;
                        cx.notify();
                    }
                }
                InputEvent::PressEnter {
                    secondary: true, ..
                } => this.save(window, cx),
                _ => {}
            }),
        ];
        body.update(cx, |body, cx| body.focus(window, cx));
        CaptureView {
            store,
            body,
            books,
            book: None,
            confirming: false,
            error: None,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    pub fn text(&self, cx: &App) -> String {
        self.body.read(cx).value().to_string()
    }

    #[cfg(test)]
    pub fn set_text_for_test(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.body
            .update(cx, |body, cx| body.set_value(text.to_owned(), window, cx));
    }

    #[cfg(test)]
    pub fn is_confirming(&self) -> bool {
        self.confirming
    }

    fn book_name(&self) -> &str {
        let id = self.book.as_deref().unwrap_or(Role::Inbox.book_id());
        self.books
            .iter()
            .find(|(book, _)| book == id)
            .map_or(Role::Inbox.default_name(), |(_, name)| name.as_str())
    }

    pub fn set_book(&mut self, book: Option<String>, cx: &mut Context<Self>) {
        self.book = book.filter(|id| id != Role::Inbox.book_id());
        cx.notify();
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.text(cx);
        match save_capture(&self.store, &text, self.book.as_deref(), Origin::Ui) {
            Ok(_) => window.remove_window(),
            Err(err) => {
                self.error = Some(format!("Couldn't save the note: {err:#}"));
                cx.notify();
            }
        }
    }

    /// Escape: closes an empty capture; over text, asks first, and a second
    /// Escape discards it.
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.text(cx).trim().is_empty() || self.confirming {
            window.remove_window();
        } else {
            self.confirming = true;
            cx.notify();
        }
    }

    fn keep_writing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.confirming = false;
        self.body.update(cx, |body, cx| body.focus(window, cx));
        cx.notify();
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let books = self.books.clone();
        let current = self
            .book
            .clone()
            .unwrap_or_else(|| Role::Inbox.book_id().into());
        h_flex()
            .flex_none()
            .h(px(40.))
            .px(px(14.))
            .gap(px(8.))
            .child(
                Icon::new(Lucide::SquarePen)
                    .size(px(15.))
                    .text_color(colors.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Quick note"),
            )
            .child(
                Button::new("capture-book")
                    .ghost()
                    .small()
                    .debug_selector(|| "capture-book".into())
                    .icon(Icon::new(Lucide::Inbox).text_color(colors.text_secondary))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(self.book_name().to_owned()),
                    )
                    .dropdown_caret(true)
                    .tooltip("Save in notebook")
                    .dropdown_menu(move |mut menu, _, _| {
                        for (ix, (id, name)) in books.iter().enumerate() {
                            if ix == 1 {
                                menu = menu.separator();
                            }
                            let view = view.clone();
                            let id = id.clone();
                            menu = menu.item(
                                PopupMenuItem::new(name.clone())
                                    .checked(id == current)
                                    .on_click(move |_, _, cx| {
                                        let id = id.clone();
                                        view.update(cx, |this, cx| this.set_book(Some(id), cx));
                                    }),
                            );
                        }
                        menu
                    }),
            )
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let hint = |keys: &'static str, label: &'static str| {
            h_flex()
                .gap(px(5.))
                .child(
                    div()
                        .px(px(5.))
                        .py(px(1.))
                        .rounded(px(4.))
                        .bg(colors.fill)
                        .text_size(px(11.))
                        .text_color(colors.text_secondary)
                        .child(keys),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(label),
                )
        };
        let save_keys = if cfg!(target_os = "macos") {
            "⌘↩"
        } else {
            "Ctrl+Enter"
        };
        let bar = h_flex()
            .flex_none()
            .h(px(46.))
            .px(px(14.))
            .gap(px(14.))
            .border_t_1()
            .border_color(colors.divider);
        if self.confirming {
            return bar
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child("Discard this note?"),
                )
                .child(
                    Button::new("capture-keep")
                        .ghost()
                        .small()
                        .label("Keep writing")
                        .on_click(cx.listener(|this, _, window, cx| this.keep_writing(window, cx))),
                )
                .child(
                    Button::new("capture-discard")
                        .danger()
                        .small()
                        .label("Discard")
                        .on_click(cx.listener(|_, _, window, _| window.remove_window())),
                );
        }
        bar.child(hint(save_keys, "Save"))
            .child(hint("esc", "Cancel"))
            .child(div().flex_1())
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .child(
                Button::new("capture-save")
                    .primary()
                    .small()
                    .label("Save")
                    .debug_selector(|| "capture-save".into())
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
    }
}

impl Focusable for CaptureView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for CaptureView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // The editor's writing face and size, as Settings › Editor has them,
        // for a note being written on its own.
        let appearance = &cx.global::<AppState>().config.appearance;
        let size = appearance.editor_font_size;
        let family = if appearance.editor_font.face(ViewMode::Edit).mono {
            cx.theme().mono_font_family.clone()
        } else {
            cx.theme().font_family.clone()
        };
        v_flex()
            .size_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .bg(colors.editor)
            .text_color(colors.text)
            .on_action(cx.listener(|this, _: &SaveCapture, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|this, _: &CancelCapture, window, cx| this.cancel(window, cx)))
            .child(self.render_header(cx))
            .child(
                div().flex_1().min_h_0().px(px(6.)).child(
                    Editor::new(&self.body)
                        .bordered(false)
                        .h_full()
                        .font_family(family)
                        .text_size(px(size))
                        .line_height(relative(crate::editor::BODY_LINE_HEIGHT)),
                ),
            )
            .child(self.render_footer(cx))
    }
}

/// Opens the capture window, or brings the open one forward.
pub fn open(cx: &mut App) {
    if focus_existing(cx) {
        return;
    }
    let Some(store) = cx.try_global::<AppState>().map(|state| state.store.clone()) else {
        return;
    };
    let size = size(px(WIDTH), px(HEIGHT));
    let mut bounds = Bounds::centered(None, size, cx);
    // In the upper part of the screen, where the eye already is.
    bounds.origin.y = (bounds.origin.y - px(120.)).max(px(40.));
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let kind = WindowKind::PopUp;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let kind = WindowKind::Normal;
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: (!cfg!(target_os = "macos")).then(|| TitlebarOptions {
            title: Some("Quick Note".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        kind,
        is_resizable: false,
        is_minimizable: false,
        focus: true,
        show: true,
        #[cfg(target_os = "linux")]
        app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, move |window, cx| {
        crate::themes::apply(Some(window), cx);
        cx.new(|cx| CaptureView::new(store, window, cx))
    });
    if let Err(err) = opened {
        eprintln!("kanso: couldn't open the quick capture window: {err:#}");
    }
}

/// Brings an open capture window forward with the keyboard in it.
fn focus_existing(cx: &mut App) -> bool {
    for handle in cx.windows() {
        let found = handle
            .update(cx, |root, window, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return false;
                };
                let view = root.read(cx).view().clone();
                let Ok(capture) = view.downcast::<CaptureView>() else {
                    return false;
                };
                window.activate_window();
                capture.update(cx, |capture, cx| {
                    capture.body.update(cx, |body, cx| body.focus(window, cx))
                });
                true
            })
            .unwrap_or(false);
        if found {
            return true;
        }
    }
    false
}
