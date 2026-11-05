//! A canvas embedded in a note: a `canvas` fence names a canvas note (and
//! maybe one of its groups) and the preview shows it in place.
//!
//! The canvas note is read off the main thread and kept for a moment; the
//! drawing is a `kanso_render` job like any other. A locked canvas is never
//! read: the preview shows a lock and "Locked canvas", and nothing of the
//! canvas is drawn or kept. Hovering shows **Open canvas**, **Open at this
//! group** and **Copy as image**; clicking the picture opens the canvas.
//!
//! No view here reads its own entity while it renders: the hover state is
//! a keyed state of its own, as the drawing block's is.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Icon, Sizable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::canvas::Embed;
use kanso_render::Job;
use kanso_render::canvas::embed::{EmbedNote, Notice, Spec};

use crate::preview_rich::{
    BLOCK, Block, CANVAS_GROUP_ANCHOR, Rich, drawing, image, palette, placeholder,
};
use crate::theme::Colors;
use crate::zoom::px;

/// How long a canvas note read is trusted before it is read again in the
/// background; the preview keeps showing the old one meanwhile.
const FRESH: Duration = Duration::from_secs(1);

/// What was read of each canvas note, and when.
static NOTES: LazyLock<Mutex<HashMap<String, Read>>> = LazyLock::new(Default::default);

/// When a canvas note was read, and what it was (`None`: no such note).
type Read = (Instant, Option<EmbedNote>);
static READING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

/// What the canvas note `id` is: `Some(None)` for no such note, `None`
/// until the first read finishes. A note older than [`FRESH`] is read again
/// in the background, and the preview woken if it changed.
fn note(id: &str, rich: &Rich, cx: &mut App) -> Option<Option<EmbedNote>> {
    let known = NOTES.lock().ok()?.get(id).cloned();
    let stale = known.as_ref().is_none_or(|(at, _)| at.elapsed() > FRESH);
    if stale && READING.lock().ok()?.insert(id.to_owned()) {
        let (store, id, wake) = (rich.store.clone(), id.to_owned(), rich.waker());
        cx.background_executor()
            .spawn(async move {
                let found = kanso_engine::canvas_embed::embed_note(&store, &id);
                let changed = NOTES.lock().is_ok_and(|mut notes| {
                    let changed = notes.get(&id).is_none_or(|(_, old)| *old != found);
                    notes.insert(id.clone(), (Instant::now(), found));
                    changed
                });
                if let Ok(mut reading) = READING.lock() {
                    reading.remove(&id);
                }
                if changed {
                    wake();
                }
            })
            .detach();
    }
    known.map(|(_, note)| note)
}

/// Fixes what the canvas note `id` is, for an hour, so a snapshot or test
/// needs no store.
#[cfg(test)]
pub(crate) fn seed(id: &str, found: EmbedNote) {
    if let Ok(mut notes) = NOTES.lock() {
        let later = Instant::now() + Duration::from_secs(3600);
        notes.insert(id.to_owned(), (later, Some(found)));
    }
}

/// What the preview of a canvas block shows beyond the picture.
#[derive(Clone, Copy, Default)]
struct Hover {
    over: bool,
    shown: u64,
}

/// A quiet card saying why there is no picture; a lock glyph on request.
fn card(words: &'static str, lock: bool, at: u64, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    h_flex()
        .id(ElementId::NamedInteger("canvas-card".into(), at))
        .w(px(380.))
        .max_w(relative(1.))
        .h(px(96.))
        .items_center()
        .justify_center()
        .gap(px(6.))
        .rounded(px(8.))
        .border_1()
        .border_dashed()
        .border_color(colors.divider)
        .bg(Hsla::from(colors.fill).opacity(0.35))
        .text_size(px(13.))
        .text_color(colors.text_secondary)
        .when(lock, |el| {
            el.child(
                Icon::new(Lucide::Lock)
                    .size(px(14.))
                    .text_color(colors.text_secondary),
            )
        })
        .child(words)
        .into_any_element()
}

/// The words for a quiet banner under the picture.
fn banner(notice: Notice, at: u64, ix: usize, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    div()
        .id(ElementId::NamedInteger(
            format!("canvas-notice-{ix}").into(),
            at,
        ))
        .pt(px(4.))
        .text_size(px(12.))
        .text_color(colors.text_muted)
        .child(notice.text())
        .into_any_element()
}

/// Draws the canvas block: the picture and its notices with the actions
/// under it, or a placeholder.
pub(crate) fn render(
    block: &Block,
    rich: &Arc<Rich>,
    at: u64,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(embed) = kanso_core::canvas::embed::parse(&block.source) else {
        return placeholder(block.kind, cx);
    };
    let (title, body) = match note(&embed.note, rich, cx) {
        None => return placeholder(block.kind, cx),
        Some(None) => return card("Canvas not found", false, at, cx),
        Some(Some(EmbedNote::Locked)) => return card("Locked canvas", true, at, cx),
        Some(Some(EmbedNote::NotCanvas)) => return card("This note isn't a canvas", false, at, cx),
        Some(Some(EmbedNote::Open { title, body })) => (title, body),
    };
    let spec = Spec {
        title,
        group: embed.group.clone(),
        width: embed.width,
    };
    let job = Job::canvas(body, spec, palette(cx));
    let Some(outcome) = drawing(&job, rich) else {
        return placeholder(block.kind, cx);
    };
    let svg = match outcome.as_ref() {
        Ok(svg) => svg.clone(),
        Err(_) => return card("Couldn't draw this canvas", false, at, cx),
    };
    let scale = rich.text_scale();
    let width = svg.width * scale;
    let picture = img(image(&job, &svg))
        .object_fit(ObjectFit::Contain)
        .max_w(relative(1.))
        .w(px(width));
    let links = rich.links.clone();
    let open = {
        let (links, note) = (links.clone(), embed.note.clone());
        move || {
            links.try_send(note.clone()).ok();
        }
    };
    let clickable = div()
        .id(ElementId::NamedInteger("canvas-picture".into(), at))
        .cursor_pointer()
        .on_click({
            let open = open.clone();
            move |_, _, _| open()
        })
        .child(picture);
    let picture_with_notices = v_flex()
        .child(clickable)
        .children(
            svg.notices
                .iter()
                .enumerate()
                .map(|(ix, notice)| banner(*notice, at, ix, cx)),
        )
        .into_any_element();
    actions(
        picture_with_notices,
        width,
        &embed,
        svg.markup.clone(),
        links,
        at,
        window,
        cx,
    )
}

/// The picture with its actions under it, shown on hover; the row keeps its
/// room while hidden, so showing it never moves the text below.
#[allow(clippy::too_many_arguments)]
fn actions(
    picture: AnyElement,
    width: f32,
    embed: &Embed,
    markup: String,
    links: crate::preview::LinkSender,
    at: u64,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let hover = window.use_keyed_state(
        ElementId::NamedInteger("canvas-hover".into(), at),
        cx,
        |_, _| Hover::default(),
    );
    let Hover { over, shown } = *hover.read(cx);
    #[cfg(test)]
    let hidden = !crate::preview::REVEAL_IMAGE_SIZES.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
    let hidden = true;
    let action = |ix: usize, label: &'static str, tooltip: &'static str| {
        Button::new(ix)
            .ghost()
            .xsmall()
            .tab_stop(false)
            .tooltip(tooltip)
            .child(
                div()
                    .px_0p5()
                    .text_size(px(11.5))
                    .text_color(colors.text_secondary)
                    .child(label),
            )
    };
    let open = {
        let (links, note) = (links.clone(), embed.note.clone());
        action(0, "Open canvas", "Opens the canvas").on_click(move |_, _, cx| {
            cx.stop_propagation();
            links.try_send(note.clone()).ok();
        })
    };
    let at_group = embed.group.clone().map(|group| {
        let (links, note) = (links.clone(), embed.note.clone());
        action(
            1,
            "Open at this group",
            "Opens the canvas with this group filling the view",
        )
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            links
                .try_send(format!("{note}#{CANVAS_GROUP_ANCHOR}{group}"))
                .ok();
        })
    });
    let copy = action(
        2,
        "Copy as image",
        "Copies the picture of this canvas to the clipboard",
    )
    .on_click(move |_, window, cx| {
        cx.stop_propagation();
        copy_as_image(markup.clone(), window, cx);
    });
    let row = h_flex()
        .id(ElementId::NamedInteger("canvas-actions".into(), at))
        .w_full()
        .justify_end()
        .pt(px(2.))
        .gap(px(2.))
        .debug_selector(|| "canvas-actions".into())
        .child(open)
        .children(at_group)
        .child(copy);
    let row = if over || !hidden {
        row.with_animation(
            ElementId::NamedInteger("canvas-fade".into(), shown),
            Animation::new(Duration::from_millis(160)).with_easing(ease_out_quint()),
            |row, t| row.opacity(t),
        )
        .into_any_element()
    } else {
        row.invisible().into_any_element()
    };
    v_flex()
        .id(ElementId::NamedInteger(BLOCK.into(), at))
        .on_hover(move |over, _, cx| {
            hover.update(cx, |hover, cx| {
                if hover.over != *over {
                    hover.over = *over;
                    hover.shown += u64::from(*over);
                    cx.notify();
                }
            });
        })
        .w(px(width.max(380.)))
        .max_w(relative(1.))
        .items_start()
        .child(picture)
        .child(row)
        .into_any_element()
}

/// Puts the picture on the clipboard as a PNG at twice its size. The
/// drawing is rasterized off the main thread.
fn copy_as_image(markup: String, window: &mut Window, cx: &mut App) {
    let handle = window.window_handle();
    let work = cx
        .background_executor()
        .spawn(async move { kanso_render::raster::svg_to_png(&markup, 2.0) });
    cx.spawn(async move |cx| {
        let outcome = work.await;
        cx.update(|cx| match outcome {
            Ok(png) => {
                cx.write_to_clipboard(ClipboardItem::new_image(&Image::from_bytes(
                    ImageFormat::Png,
                    png,
                )));
                handle
                    .update(cx, |_, window, cx| {
                        window
                            .push_notification(Notification::info("Canvas copied as an image."), cx)
                    })
                    .ok();
            }
            Err(why) => {
                handle
                    .update(cx, |_, window, cx| {
                        window.push_notification(Notification::error(why), cx)
                    })
                    .ok();
            }
        });
    })
    .detach();
}
