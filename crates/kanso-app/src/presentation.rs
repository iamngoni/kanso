//! A note presented: its slides (`kanso_core::slides`) full screen, on the
//! display the note's window is on, in the preview's theme, so diagrams, math
//! and pictures draw as they do there.
//!
//! The window shows the note as it was when it opened, and changes nothing:
//! the preview's edits and note links go to channels nobody reads, so ticking a
//! box or dragging a picture on a slide leaves the note alone. Links to the web
//! still open. Slides close when the note can't be read any more: its keys were
//! forgotten, or it was deleted.

use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::{Note, slides};
use kanso_store::Store;

use crate::locked::LockKeys;
use crate::preview::Preview;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [PresentNote]);

/// The window size, in layout units, slide text is set for.
const DESIGN_SIZE: (f32, f32) = (1280., 720.);
/// How large slide text is in a window of [`DESIGN_SIZE`].
const DESIGN_TEXT: f32 = 28.;
/// How small and large slide text gets, so a small window and a very large
/// display both stay readable.
const TEXT_RANGE: (f32, f32) = (20., 72.);
/// The room around a slide, as a share of the window.
const MARGIN: f32 = 0.08;
/// How much of the slide one press scrolls, so a line stays in view.
const PAGE: f32 = 0.8;
/// The frames after a change the overflow is looked at again: its layout comes
/// a frame late, and a drawing's a little later.
const SETTLE_FRAMES: u8 = 3;

/// How much larger than `base`, the preview's text size, slide text is in a
/// window of `size`. Whichever of width and height is tighter sets it, so a
/// portrait display keeps a usable line length. Pure.
fn text_scale(size: (f32, f32), base: f32) -> f32 {
    let fit = (size.0 / DESIGN_SIZE.0).min(size.1 / DESIGN_SIZE.1);
    (DESIGN_TEXT * fit).clamp(TEXT_RANGE.0, TEXT_RANGE.1) / base.max(1.)
}

/// What a key does to the slideshow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    /// The next slide, whatever is left of this one.
    Next,
    /// The slide before, wherever this one is scrolled to.
    Previous,
    /// Down this slide, or the next once it is at its end.
    Forward,
    /// Up this slide, or the slide before once it is at its start.
    Back,
    First,
    Last,
    Fullscreen,
    Close,
}

/// What `key` does, if anything. Pure.
fn key_action(key: &str) -> Option<Key> {
    Some(match key {
        "right" | "enter" | "n" => Key::Next,
        "left" | "backspace" | "p" => Key::Previous,
        "down" | "space" | "pagedown" => Key::Forward,
        "up" | "pageup" => Key::Back,
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
        Key::Next | Key::Forward => (current + 1).min(last),
        Key::Previous | Key::Back => current.saturating_sub(1),
        Key::First => 0,
        Key::Last => last,
        Key::Fullscreen | Key::Close => current,
    }
}

/// Where a slide scrolled to `top` of `max` goes for `key`, by `page`, or
/// `None` when `key` isn't a scroll or the slide is at that end. Pure.
fn scrolled(top: f32, max: f32, page: f32, key: Key) -> Option<f32> {
    match key {
        Key::Forward if has_more(top, max) => Some((top + page).min(max)),
        Key::Back if top >= 0.5 => Some((top - page).max(0.)),
        _ => None,
    }
}

/// Whether a slide scrolled to `top` of `max` has more below. Pure.
fn has_more(top: f32, max: f32) -> bool {
    max - top >= 0.5
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

/// Presents note `id` on the display `window` is on. Typing waiting in any
/// window is saved first, so the slides have it; a locked note has to be open.
pub fn present(id: String, window: &Window, cx: &mut App) {
    let display = window.display(cx).map(|display| display.id());
    let asked_in = window.window_handle();
    // The windows can't be looked through from inside this one's update.
    cx.defer(move |cx| {
        flush_everywhere(cx);
        let store = cx.global::<AppState>().store.clone();
        let Ok(Some(mut note)) = store.note(&id) else {
            return;
        };
        if crate::locked::reveal(&mut note, cx).is_closed() {
            let message = Notification::error("Unlock the note to present it.");
            asked_in
                .update(cx, |_, window, cx| window.push_notification(message, cx))
                .ok();
            return;
        }
        open(note, display, cx);
    });
}

/// Saves the typing waiting in the main window and in every note's own.
fn flush_everywhere(cx: &mut App) {
    if let Some((_, workspace)) = crate::workspace::main_workspace(cx) {
        workspace.update(cx, |workspace, cx| workspace.flush(cx));
    }
    for (_, view) in crate::note_window::note_windows(cx) {
        let editor = view.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.flush(cx));
    }
}

/// Presents `note`, whose text is already readable, on `display`.
fn open(note: Note, display: Option<DisplayId>, cx: &mut App) {
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
    /// Frames left to look at the overflow again.
    settling: u8,
    _subscriptions: Vec<Subscription>,
}

impl Presentation {
    fn new(note: Note, store: Arc<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let slides = slides_of(&note.title, &note.body);
        // The receivers go at once: a slide asks nothing of the note.
        let (links, _) = async_channel::unbounded();
        let (edits, _) = async_channel::unbounded();
        let id = note.id.clone();
        let scale = text_scale(window_size(window), base_size(cx));
        let preview = cx.new(|cx| {
            let mut preview = Preview::new(store.clone(), links, edits, cx);
            preview.set_note(Some(id.clone()), cx);
            preview.show_at_once(slides[0].clone().into(), cx);
            preview.set_text_scale(scale, cx);
            preview
        });
        let focus = cx.focus_handle();
        let subscriptions = vec![
            cx.observe_window_bounds(window, |this, window, cx| {
                let scale = text_scale(window_size(window), base_size(cx));
                this.preview
                    .update(cx, |preview, cx| preview.set_text_scale(scale, cx));
                this.settle(cx);
            }),
            // A drawing that finishes changes how much the slide overflows.
            cx.observe(&preview, |this, _, cx| this.settle(cx)),
            // Whatever a click took the keyboard to, the keys stay here.
            cx.on_focus_lost(window, |this, window, cx| window.focus(&this.focus, cx)),
            // Any note can be locked after it starts, so this always watches.
            cx.observe_global_in::<LockKeys>(window, move |_, window, cx| {
                let closed = match store.note(&id) {
                    Ok(Some(mut note)) => crate::locked::reveal(&mut note, cx).is_closed(),
                    _ => true,
                };
                if closed {
                    window.remove_window();
                }
            }),
        ];
        window.focus(&focus, cx);
        Presentation {
            slides,
            current: 0,
            preview,
            focus,
            settling: SETTLE_FRAMES,
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
            Some(key) => {
                let page = f32::from(window.viewport_size().height) * (1. - 2. * MARGIN) * PAGE;
                let (top, max) = self.scroll(cx);
                match scrolled(top, max, page, key) {
                    Some(top) => {
                        self.preview
                            .update(cx, |preview, cx| preview.scroll_to(top, cx));
                        cx.notify();
                    }
                    None => self.go_to(moved(self.current, self.slides.len(), key), cx),
                }
            }
            None => {}
        }
    }

    /// Where the slide is scrolled to, and how far it can go.
    fn scroll(&self, cx: &App) -> (f32, f32) {
        let preview = self.preview.read(cx);
        (preview.scroll_top(), preview.scroll_max())
    }

    /// Looks at the overflow again over the next few frames.
    fn settle(&mut self, cx: &mut Context<Self>) {
        self.settling = SETTLE_FRAMES;
        cx.notify();
    }

    fn go_to(&mut self, slide: usize, cx: &mut Context<Self>) {
        if slide == self.current {
            return;
        }
        self.current = slide;
        let text: SharedString = self.slides[slide].clone().into();
        self.preview
            .update(cx, |preview, cx| preview.show_at_once(text, cx));
        self.settle(cx);
    }
}

#[cfg(test)]
impl Presentation {
    /// The slide on show, with the text the preview has.
    pub(crate) fn shown_for_test(&self, cx: &App) -> (usize, SharedString) {
        (self.current, self.preview.read(cx).shown().clone())
    }
}

/// The window's size, in layout units.
fn window_size(window: &Window) -> (f32, f32) {
    let size = window.viewport_size();
    (
        crate::zoom::unscaled(size.width),
        crate::zoom::unscaled(size.height),
    )
}

/// The preview's text size, from the settings.
fn base_size(cx: &mut App) -> f32 {
    crate::preview::preview_type(cx).size
}

impl Render for Presentation {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let viewport = window.viewport_size();
        let count = self.slides.len();
        let progress = (self.current + 1) as f32 / count as f32;
        let (top, max) = self.scroll(cx);
        if self.settling > 0 {
            self.settling -= 1;
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
        div()
            .id("presentation")
            .key_context("Presentation")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
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
                    .child(if has_more(top, max) {
                        format!("↓   {} / {count}", self.current + 1)
                    } else {
                        format!("{} / {count}", self.current + 1)
                    }),
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
    use super::{
        DESIGN_TEXT, Key, TEXT_RANGE, has_more, key_action, moved, scrolled, slides_of, text_scale,
    };
    use proptest::prelude::*;

    #[test]
    fn keys_move_through_the_slides() {
        assert_eq!(key_action("right"), Some(Key::Next));
        assert_eq!(key_action("left"), Some(Key::Previous));
        assert_eq!(key_action("escape"), Some(Key::Close));
        assert_eq!(key_action("x"), None);
        assert_eq!(moved(0, 3, Key::Previous), 0);
        assert_eq!(moved(2, 3, Key::Next), 2);
        assert_eq!(moved(1, 3, Key::Last), 2);
        assert_eq!(moved(1, 3, Key::First), 0);
    }

    #[test]
    fn a_long_slide_scrolls_before_the_next_one() {
        assert_eq!(key_action("pagedown"), Some(Key::Forward));
        assert_eq!(key_action("up"), Some(Key::Back));
        assert_eq!(scrolled(0., 500., 300., Key::Forward), Some(300.));
        assert_eq!(scrolled(300., 500., 300., Key::Forward), Some(500.));
        assert_eq!(scrolled(500., 500., 300., Key::Forward), None);
        assert_eq!(scrolled(100., 500., 300., Key::Back), Some(0.));
        assert_eq!(scrolled(0., 500., 300., Key::Back), None);
        assert_eq!(scrolled(0., 500., 300., Key::Next), None);
    }

    #[test]
    fn text_fits_the_tighter_side_of_the_window() {
        assert_eq!(text_scale((1280., 720.), 16.), DESIGN_TEXT / 16.);
        let portrait = text_scale((1080., 1920.), 16.);
        let landscape = text_scale((1920., 1080.), 16.);
        assert!(portrait < landscape, "{portrait} {landscape}");
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

        /// A larger window never makes text smaller, and text stays in
        /// range whatever size the preview's is set to.
        #[test]
        fn prop_text_grows_with_the_window(
            w in 1f32..5000.,
            h in 1f32..5000.,
            more in 0f32..2000.,
            base in 8f32..40.,
        ) {
            let size = text_scale((w, h), base) * base;
            prop_assert!(size <= text_scale((w + more, h + more), base) * base);
            prop_assert!((TEXT_RANGE.0 - 0.01..=TEXT_RANGE.1 + 0.01).contains(&size));
        }

        /// A scroll stays on the slide, and only happens when there is
        /// somewhere to go.
        #[test]
        fn prop_scrolls_stay_on_the_slide(
            top in 0f32..2000.,
            max in 0f32..2000.,
            page in 1f32..800.,
            forward in any::<bool>(),
        ) {
            let top = top.min(max);
            let key = if forward { Key::Forward } else { Key::Back };
            if let Some(to) = scrolled(top, max, page, key) {
                prop_assert!((0. ..=max).contains(&to));
            }
            prop_assert_eq!(
                forward && scrolled(top, max, page, key).is_some(),
                forward && has_more(top, max)
            );
        }
    }
}
