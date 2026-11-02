//! A note presented: its slides (`kanso_core::slides`) full screen, on the
//! display the note's window is on, in the preview's theme, so diagrams, math
//! and pictures draw as they do there.
//!
//! The window shows the note as it was when it opened, and changes nothing:
//! the preview's edits and note links go to channels nobody reads, so ticking a
//! box or dragging a picture on a slide leaves the note alone. Links to the web
//! still open. A locked note's slides close when its keys are forgotten.

use std::sync::Arc;

use gpui_kit::*;
use kanso_core::{Note, slides};
use kanso_store::Store;

use crate::locked::LockKeys;
use crate::preview::Preview;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [PresentNote]);

/// The window height, in layout units, slide text is set for.
const DESIGN_HEIGHT: f32 = 720.;
/// How much larger than the preview's size text is at [`DESIGN_HEIGHT`].
const DESIGN_SCALE: f32 = 1.6;
/// How far slide text scales, so a small window and a very tall display both
/// stay readable.
const SCALE_RANGE: (f32, f32) = (1.2, 4.);
/// The room around a slide, as a share of the window.
const MARGIN: f32 = 0.08;

/// How much larger than the preview a slide's text is in a window `height`
/// tall. Pure.
fn text_scale(height: f32) -> f32 {
    (DESIGN_SCALE * height / DESIGN_HEIGHT).clamp(SCALE_RANGE.0, SCALE_RANGE.1)
}

/// What a key does to the slideshow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Next,
    Previous,
    First,
    Last,
    Fullscreen,
    Close,
}

/// What `key` does, if anything. Pure.
fn key_action(key: &str) -> Option<Key> {
    Some(match key {
        "right" | "down" | "space" | "enter" | "pagedown" | "n" => Key::Next,
        "left" | "up" | "backspace" | "pageup" | "p" => Key::Previous,
        "home" => Key::First,
        "end" => Key::Last,
        "f" => Key::Fullscreen,
        "escape" => Key::Close,
        _ => return None,
    })
}

/// The slide `key` moves to from `current`, of `count`. Pure.
fn moved(current: usize, count: usize, key: Key) -> usize {
    let last = count.saturating_sub(1);
    match key {
        Key::Next => (current + 1).min(last),
        Key::Previous => current.saturating_sub(1),
        Key::First => 0,
        Key::Last => last,
        Key::Fullscreen | Key::Close => current,
    }
}

/// The slides a note presents: an empty one shows its title.
fn slides_of(title: &str, body: &str) -> Vec<String> {
    let slides = slides::split(body);
    if slides.is_empty() {
        vec![format!("# {title}")]
    } else {
        slides
    }
}

/// Presents `note`, whose text is already readable, on `display`.
pub fn open(note: Note, display: Option<DisplayId>, cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    let titlebar = TitlebarOptions {
        title: Some(note.title.clone().into()),
        appears_transparent: cfg!(target_os = "macos"),
        traffic_light_position: None,
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Fullscreen(Bounds::maximized(display, cx))),
        display_id: display,
        titlebar: Some(titlebar),
        #[cfg(target_os = "linux")]
        app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        crate::themes::apply(Some(window), cx);
        cx.new(|cx| Presentation::new(note, store, window, cx))
    });
    if let Err(err) = opened {
        eprintln!("kanso: couldn't open the presentation: {err:#}");
    }
}

pub struct Presentation {
    slides: Vec<String>,
    current: usize,
    preview: Entity<Preview>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Presentation {
    fn new(note: Note, store: Arc<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let slides = slides_of(&note.title, &note.body);
        // The receivers go at once: a slide asks nothing of the note.
        let (links, _) = async_channel::unbounded();
        let (edits, _) = async_channel::unbounded();
        let locked = store.note_is_locked(&note.id).unwrap_or(false);
        let id = note.id.clone();
        let preview = cx.new(|cx| {
            let mut preview = Preview::new(store.clone(), links, edits, cx);
            preview.set_note(Some(id.clone()), cx);
            preview.set_text(slides[0].clone().into(), cx);
            preview.set_text_scale(text_scale(window_height(window)), cx);
            preview
        });
        let fit = cx.observe_window_bounds(window, |this, window, cx| {
            let scale = text_scale(window_height(window));
            this.preview
                .update(cx, |preview, cx| preview.set_text_scale(scale, cx));
            cx.notify();
        });
        let mut subscriptions = vec![fit];
        if locked {
            subscriptions.push(
                cx.observe_global_in::<LockKeys>(window, move |_, window, cx| {
                    let closed = match store.note(&id) {
                        Ok(Some(mut note)) => crate::locked::reveal(&mut note, cx).is_closed(),
                        _ => true,
                    };
                    if closed {
                        window.remove_window();
                    }
                }),
            );
        }
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Presentation {
            slides,
            current: 0,
            preview,
            focus,
            _subscriptions: subscriptions,
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.platform || modifiers.control || modifiers.alt {
            return;
        }
        match key_action(&event.keystroke.key) {
            Some(Key::Close) => window.remove_window(),
            Some(Key::Fullscreen) => window.toggle_fullscreen(),
            Some(key) => self.go_to(moved(self.current, self.slides.len(), key), cx),
            None => {}
        }
    }

    fn go_to(&mut self, slide: usize, cx: &mut Context<Self>) {
        if slide == self.current {
            return;
        }
        self.current = slide;
        let text: SharedString = self.slides[slide].clone().into();
        self.preview.update(cx, |preview, cx| {
            preview.set_text(text, cx);
            preview.scroll_to(0., cx);
        });
        cx.notify();
    }
}

#[cfg(test)]
impl Presentation {
    /// The slide on show, with the text the preview has.
    pub(crate) fn shown_for_test(&self, cx: &App) -> (usize, SharedString) {
        (self.current, self.preview.read(cx).shown().clone())
    }
}

/// The window's height, in layout units.
fn window_height(window: &Window) -> f32 {
    crate::zoom::unscaled(window.viewport_size().height)
}

impl Render for Presentation {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let viewport = window.viewport_size();
        let count = self.slides.len();
        let progress = (self.current + 1) as f32 / count as f32;
        div()
            .id("presentation")
            .key_context("Presentation")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .relative()
            .size_full()
            .bg(colors.editor)
            .px(viewport.width * MARGIN)
            .py(viewport.height * MARGIN)
            .child(self.preview.clone())
            .child(
                div()
                    .absolute()
                    .bottom(px(16.))
                    .right(px(24.))
                    .text_size(px(14.))
                    .text_color(colors.text_muted)
                    .child(format!("{} / {count}", self.current + 1)),
            )
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .h(px(3.))
                    .w(relative(progress))
                    .bg(colors.accent),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, SCALE_RANGE, key_action, moved, slides_of, text_scale};
    use proptest::prelude::*;

    #[test]
    fn keys_move_through_the_slides() {
        assert_eq!(key_action("right"), Some(Key::Next));
        assert_eq!(key_action("pageup"), Some(Key::Previous));
        assert_eq!(key_action("escape"), Some(Key::Close));
        assert_eq!(key_action("x"), None);
        assert_eq!(moved(0, 3, Key::Previous), 0);
        assert_eq!(moved(2, 3, Key::Next), 2);
        assert_eq!(moved(1, 3, Key::Last), 2);
        assert_eq!(moved(1, 3, Key::First), 0);
    }

    #[test]
    fn a_note_with_no_text_presents_its_title() {
        assert_eq!(slides_of("Plans", "\n\n"), ["# Plans"]);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A move stays on a slide that exists.
        #[test]
        fn prop_moves_stay_in_range(
            current in 0usize..50,
            count in 1usize..50,
            key in prop::sample::select(vec![Key::Next, Key::Previous, Key::First, Key::Last]),
        ) {
            let current = current.min(count - 1);
            prop_assert!(moved(current, count, key) < count);
        }

        /// A taller window never makes text smaller, and text stays in range.
        #[test]
        fn prop_text_scale_grows_with_the_window(a in 1f32..5000., b in 1f32..5000.) {
            let (low, high) = (a.min(b), a.max(b));
            prop_assert!(text_scale(low) <= text_scale(high));
            prop_assert!((SCALE_RANGE.0..=SCALE_RANGE.1).contains(&text_scale(low)));
        }
    }
}
