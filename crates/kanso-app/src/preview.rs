//! The rendered note. It is its own view so the editor can cache it:
//! re-laying out a large note on every keystroke frame doubles edit latency.
//!
//! The note is shown in chunks (see [`crate::preview_chunks`]), each its own
//! text view in a list: an edit re-parses and re-lays out only the chunk it
//! falls in, and only the chunks on screen are laid out at all. Finding the
//! chunks of a long note parses it, which runs off the main thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::{
    CodeBlock, MarkdownExtensions, MarkdownNode, MarkdownParseContext, TextView, TextViewState,
    TextViewStyle, markdown_ast,
};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Selectable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::links::FILE_LINK_PREFIX;
use kanso_render::typography;
use kanso_store::Store;

use crate::preview_chunks::{self, End, Prepared, SPLIT_SYNC_BYTES, Splice, Start};
use crate::preview_rich::{self, Rich};
use crate::preview_tasks::{self, TaskTick};
use crate::theme::Colors;
use crate::theme_file::PreviewColors;
use crate::zoom::px;

/// The room above the first chunk and below the last.
const TEXT_TOP: f32 = 24.;
/// How far past the pane the list lays chunks out, so a scroll shows them
/// laid out already.
const OVERDRAW: f32 = 800.;
/// How far text sits from the pane's left edge: under the note's title, like
/// the source in Edit mode.
pub const TEXT_INSET: f32 = 28.;

/// The readable measure for text set at `size`, in the monospaced face or
/// the proportional one. The preview keeps to it, and so does the source in
/// Edit mode, at its own size.
pub fn readable_width(size: f32, mono: bool) -> f32 {
    typography::readable_width(size, mono)
}

/// Where clicks on note links ([`NOTE_LINK_PREFIX`]`<id>`) go.
pub type LinkSender = async_channel::Sender<String>;

/// A size picked for an attached image in the preview: the editor puts
/// `source` (the image's Markdown) back as `src` shown `width` wide, or at
/// its own size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageResize {
    pub source: String,
    pub src: String,
    pub alt: String,
    pub width: Option<u32>,
}

/// An edit the preview asks the editor for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewEdit {
    /// A size picked for an image.
    Resize(ImageResize),
    /// A block (a drawing brought back from Excalidraw) rewritten: the
    /// first `source` in the note becomes `replacement`.
    Replace { source: String, replacement: String },
    /// A task box ticked or cleared.
    Task(TaskTick),
}

/// Where the preview's edits go.
pub type ResizeSender = async_channel::Sender<PreviewEdit>;

pub use kanso_engine::edit::image::IMAGE_SIZES;

/// Shows the size buttons over every image, as a hover would.
#[cfg(test)]
pub(crate) static REVEAL_IMAGE_SIZES: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Shows every image mid-drag at this width, for the snapshots.
#[cfg(test)]
pub(crate) static PREVIEW_DRAG_WIDTH: std::sync::Mutex<Option<f32>> = std::sync::Mutex::new(None);

/// A chunk on show.
struct Shown {
    key: u64,
    /// The bytes of the note the chunk shows.
    len: usize,
    start: Start,
    end: End,
    state: Entity<TextViewState>,
    _watch: Subscription,
}

pub struct Preview {
    /// The text handed over last.
    text: SharedString,
    /// The text the chunks show: `text`, or an older one while it is split.
    shown: SharedString,
    chunks: Vec<Shown>,
    list: ListState,
    /// Splitting a long text, off the main thread.
    splitting: Option<Task<()>>,
    /// Counts the texts shown, so the editor can tell the preview changed.
    generation: u64,
    store: Arc<Store>,
    images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>>,
    links: LinkSender,
    /// Where a ticked task box goes.
    edits: ResizeSender,
    /// Whether text keeps to the readable measure (the setting), or fills
    /// the pane.
    readable: bool,
    /// Draws quotes in their own color (see [`quote_extensions`]).
    extensions: MarkdownExtensions,
    /// The text size and face the chunks were last measured at.
    shown_type: Option<PreviewType>,
    /// Measures the chunks again as math and diagrams finish drawing.
    _rich: Task<()>,
    /// The note shown: a query view leaves it out.
    note: Arc<Mutex<Option<String>>>,
}

impl Preview {
    pub fn new(
        store: Arc<Store>,
        links: LinkSender,
        resizes: ResizeSender,
        cx: &mut Context<Self>,
    ) -> Self {
        let images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>> = Arc::default();
        let resolver: ImageResolver = Arc::new(image_resolver(store.clone(), images.clone()));
        let (wake, woken) = async_channel::unbounded();
        let note = Arc::new(Mutex::new(None));
        let rich = Arc::new(Rich {
            wake,
            store: store.clone(),
            edits: resizes.clone(),
            note: note.clone(),
        });
        let edits = resizes.clone();
        let extensions = quote_extensions(Arc::new(Links(links.clone())), resolver, resizes, rich);
        let rich_task = cx.spawn(async move |this, cx| {
            while woken.recv().await.is_ok() {
                // Several drawings finishing together are one remeasure.
                while woken.try_recv().is_ok() {}
                if this.update(cx, |this, cx| this.drawings_ready(cx)).is_err() {
                    break;
                }
            }
        });
        Preview {
            text: SharedString::default(),
            shown: SharedString::default(),
            chunks: Vec::new(),
            list: ListState::new(0, ListAlignment::Top, px(OVERDRAW)).measure_all(),
            splitting: None,
            generation: 0,
            store,
            images,
            links,
            edits,
            readable: false,
            extensions,
            shown_type: None,
            _rich: rich_task,
            note,
        }
    }

    /// The note the preview shows, so a query view can leave it out.
    pub fn set_note(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        let changed = self.note.lock().map(|mut note| {
            if *note == id {
                false
            } else {
                *note = id;
                true
            }
        });
        if changed.unwrap_or(false) {
            cx.notify();
        }
    }

    /// Notes changed: query views show their notes as they are now. A note
    /// without a query has nothing to redraw.
    pub fn queries_changed(&mut self, cx: &mut Context<Self>) {
        if !kanso_core::query_view::contains_query(&self.shown) {
            return;
        }
        self.drawings_ready(cx);
    }

    /// Drawings finished: the chunks showing them change height.
    fn drawings_ready(&mut self, cx: &mut Context<Self>) {
        for chunk in &self.chunks {
            chunk
                .state
                .update(cx, |state, cx| state.invalidate_inline_layout(cx));
        }
        if !self.chunks.is_empty() {
            self.list.remeasure_items(0..self.chunks.len());
        }
        cx.notify();
    }

    /// Shows `text`. A short one shows at once; a long one once it has been
    /// split, off the main thread, with the text before it shown until then.
    /// While one is being split, a newer text waits for it and then follows.
    pub fn set_text(&mut self, text: SharedString, cx: &mut Context<Self>) {
        if self.text == text {
            return;
        }
        self.text = text;
        if self.splitting.is_some() {
            return;
        }
        if self.text.len() <= SPLIT_SYNC_BYTES {
            let prepared = preview_chunks::prepare(&self.text);
            self.apply(self.text.clone(), prepared, cx);
        } else {
            self.split_soon(cx);
        }
    }

    fn split_soon(&mut self, cx: &mut Context<Self>) {
        let text = self.text.clone();
        self.splitting = Some(cx.spawn(async move |this, cx| {
            let (text, prepared) = cx
                .background_spawn(async move {
                    let prepared = preview_chunks::prepare(&text);
                    (text, prepared)
                })
                .await;
            this.update(cx, |this, cx| {
                this.splitting = None;
                this.apply(text, prepared, cx);
                if this.shown != this.text {
                    if this.text.len() <= SPLIT_SYNC_BYTES {
                        let prepared = preview_chunks::prepare(&this.text);
                        this.apply(this.text.clone(), prepared, cx);
                    } else {
                        this.split_soon(cx);
                    }
                }
            })
            .ok();
        }));
    }

    /// Shows `text`, split into `prepared`: the chunks it shares with what
    /// is shown keep their parse and layout, the rest are parsed and
    /// measured again.
    fn apply(&mut self, text: SharedString, prepared: Vec<Prepared>, cx: &mut Context<Self>) {
        let old: Vec<u64> = self.chunks.iter().map(|c| c.key).collect();
        let new: Vec<u64> = prepared.iter().map(|c| c.key).collect();
        let Splice {
            old_range,
            new_range,
        } = preview_chunks::diff(&old, &new);
        let mut prepared = prepared;
        let mut changed = prepared.drain(new_range.clone());
        if old_range.len() == new_range.len() {
            // Typing changes a chunk in place: it keeps its view and its
            // measured height until it is measured again.
            for (shown, chunk) in self.chunks[old_range.clone()].iter_mut().zip(&mut changed) {
                shown.key = chunk.key;
                shown.len = chunk.text.len();
                shown.start = chunk.start;
                shown.end = chunk.end;
                shown
                    .state
                    .update(cx, |state, cx| state.set_text(&chunk.text, cx));
            }
            drop(changed);
            if !old_range.is_empty() {
                self.list.remeasure_items(old_range);
            }
        } else {
            let added: Vec<Shown> = changed.map(|chunk| self.show_chunk(chunk, cx)).collect();
            self.chunks.splice(old_range.clone(), added);
            self.list.splice(old_range, new_range.len());
            // Measures the new chunks, and only them, at the next layout.
            self.list.remeasure_items(new_range);
        }
        self.shown = text;
        self.generation += 1;
        cx.notify();
    }

    fn show_chunk(&mut self, chunk: Prepared, cx: &mut Context<Self>) -> Shown {
        let state = cx.new(|cx| TextViewState::markdown(&chunk.text, cx).selectable(true));
        // A long chunk parses in the background and changes height when it
        // is done, as does one whose images load: it is measured again.
        let watch = cx.observe(&state, |this, state, _| {
            if let Some(ix) = this.chunks.iter().position(|c| c.state == state) {
                this.list.remeasure_items(ix..ix + 1);
            }
        });
        Shown {
            key: chunk.key,
            len: chunk.text.len(),
            start: chunk.start,
            end: chunk.end,
            state,
            _watch: watch,
        }
    }

    /// A task box clicked in the chunk `key` at `ix`, whose item starts at
    /// byte `offset` of the chunk: asks the editor to toggle that task. A
    /// click the preview can't place with certainty asks for nothing.
    fn task_clicked(&mut self, ix: usize, key: u64, offset: usize) {
        // The chunks show the text handed over, whole: not an older text
        // while a newer one is split, and not the plain paragraphs that text
        // markdown-rs panics on is shown as.
        let shown: usize = self.chunks.iter().map(|c| c.len).sum();
        if self.shown != self.text
            || shown != self.shown.len()
            || self.chunks.get(ix).is_none_or(|c| c.key != key)
        {
            return;
        }
        let before: usize = self.chunks[..ix].iter().map(|c| c.len).sum();
        if let Some(tick) = preview_tasks::tick_at(&self.shown, before + offset) {
            self.edits.try_send(PreviewEdit::Task(tick)).ok();
        }
    }

    /// Counts the texts shown: it changes whenever the preview shows
    /// another.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The text the preview shows.
    #[cfg(test)]
    pub fn shown(&self) -> &SharedString {
        &self.shown
    }

    /// How many chunks the text is shown in.
    #[cfg(test)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// The chunks shown, by key, with their views.
    #[cfg(test)]
    fn chunk_views(&self) -> Vec<(u64, EntityId)> {
        self.chunks
            .iter()
            .map(|c| (c.key, c.state.entity_id()))
            .collect()
    }

    /// Keeps text to the readable measure, or lets it fill the pane.
    pub fn set_readable(&mut self, readable: bool, cx: &mut Context<Self>) {
        if self.readable != readable {
            self.readable = readable;
            cx.notify();
        }
    }

    /// How far the preview can scroll, as of its last layout.
    pub fn scroll_max(&self) -> f32 {
        f32::from(self.list.max_offset_for_scrollbar().y).max(0.)
    }

    /// Scrolls `top` points down from the start.
    pub fn scroll_to(&mut self, top: f32, cx: &mut Context<Self>) {
        let top = top.max(0.);
        if (self.scroll_top() - top).abs() >= 0.5 {
            // Scroll offsets are the window's own pixels, at any zoom.
            self.list
                .set_offset_from_scrollbar(point(gpui_kit::px(0.), gpui_kit::px(-top)));
            cx.notify();
        }
    }

    /// Where the preview is scrolled to, in points from the start.
    pub fn scroll_top(&self) -> f32 {
        -f32::from(self.list.scroll_px_offset_for_scrollbar().y)
    }

    /// Resolves attachment links ([`FILE_LINK_PREFIX`]`<id>`) from the store,
    /// caching decoded images.
    fn image_resolver(&self) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
        image_resolver(self.store.clone(), self.images.clone())
    }
}

/// Finds a note's images: attachment links from the store, decoded once
/// and cached; anything else as the URI says.
type ImageResolver = Arc<dyn Fn(&SharedUri) -> ImageSource + Send + Sync>;

fn image_resolver(
    store: Arc<Store>,
    images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>>,
) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
    move |uri| {
        let Some(suffix) = uri.strip_prefix(FILE_LINK_PREFIX) else {
            return ImageSource::from(uri.clone());
        };
        let id = format!("file:{suffix}");
        let mut cache = images.lock().expect("image cache poisoned");
        let image = cache.entry(id.clone()).or_insert_with(|| {
            let (content_type, attachment) = store.attachment(&id).ok()??;
            let format = ImageFormat::from_mime_type(&content_type)?;
            Some(Arc::new(Image::from_bytes(format, attachment.data)))
        });
        match image {
            Some(image) => ImageSource::Image(image.clone()),
            None => ImageSource::from(uri.clone()),
        }
    }
}

/// Where a clicked link goes: a note link to the editor, a web or mail link
/// to the system, anything else nowhere.
struct Links(LinkSender);

impl Links {
    fn open(&self, url: &str, cx: &mut App) {
        match linked_note(url) {
            Some(id) => {
                self.0.try_send(id).ok();
            }
            // An unresolved wiki-link: the workspace creates its note.
            None if url.starts_with(crate::wikilinks::NEW_NOTE_SCHEME) => {
                self.0.try_send(url.to_owned()).ok();
            }
            None if is_openable(url) => cx.open_url(url),
            None => {}
        }
    }
}

/// A quote's text without its `>` markers, to be shown as Markdown of its
/// own. Each line loses the whitespace before its `>`, the `>` and one space
/// or tab after it; a line without one (a lazy continuation) stays as it is.
/// Pure.
pub(crate) fn unquote(source: &str) -> String {
    source
        .split('\n')
        .map(
            |line| match line.trim_start_matches([' ', '\t']).strip_prefix('>') {
                Some(rest) => rest
                    .strip_prefix(' ')
                    .or_else(|| rest.strip_prefix('\t'))
                    .unwrap_or(rest),
                None => line,
            },
        )
        .collect::<Vec<_>>()
        .join("\n")
}

/// The custom node a quote becomes.
const QUOTE: &str = "kanso-quote";
/// The custom node an attached image on a line of its own becomes.
const PICTURE: &str = "kanso-picture";
/// The custom node front matter becomes: nothing. The inspector's
/// Properties section shows it instead.
const FRONT_MATTER: &str = "kanso-front-matter";
/// Changes when [`quote_extensions`] parses differently, so a text view
/// re-parses its note. 3: math, diagrams and drawings. 4: due-date chips.
/// 5: query views. 6: front matter hidden.
const QUOTE_PARSER_REVISION: u64 = 6;

/// Front matter, drawn as nothing: the Properties section shows it.
fn front_matter_node(node: &markdown_ast::Node) -> Option<MarkdownNode> {
    let markdown_ast::Node::Yaml(_) = node else {
        return None;
    };
    Some(MarkdownNode::new(FRONT_MATTER, ()).text("").markdown(""))
}

/// A quote's Markdown, without its markers.
struct Quote(SharedString);

/// An image on a line of its own, as `![alt](src)`, `![alt|300](src)` or
/// `<img src=… width=…>`.
struct Picture {
    src: String,
    /// Without any `|300`.
    alt: String,
    width: Option<u32>,
    source: String,
}

/// Whether the preview can show an image from `src`: an attachment or the
/// web.
fn showable_src(src: &str) -> bool {
    src.starts_with(FILE_LINK_PREFIX) || src.starts_with("https://") || src.starts_with("http://")
}

/// The image `node` shows on its own, if it is one: a paragraph of just an
/// attached or web image, or an `<img>` tag for one. Pure.
fn picture(node: &markdown_ast::Node, source: &str) -> Option<Picture> {
    let (src, alt, width) = match node {
        markdown_ast::Node::Paragraph(paragraph) => match paragraph.children.as_slice() {
            [markdown_ast::Node::Image(image)] => {
                let (alt, width) = kanso_render::html::split_image_width(&image.alt);
                (image.url.clone(), alt.to_owned(), width)
            }
            _ => return None,
        },
        markdown_ast::Node::Html(html) => {
            let tag = crate::smart_paste::img_tag(&html.value)?;
            (tag.src, tag.alt, tag.width)
        }
        _ => return None,
    };
    showable_src(&src).then(|| Picture {
        src,
        alt,
        width,
        source: source.trim_end().to_owned(),
    })
}

/// An image's right edge being dragged: the width it shows at, and where
/// its left edge and the room for it are (kept up to date as it's drawn).
#[derive(Clone, Copy, Default)]
struct PictureDrag {
    width: Option<f32>,
    left: f32,
    room: f32,
}

/// An image with its size buttons over its corner and a handle on its right
/// edge, both shown on hover. Dragging the handle shows the new width as it
/// goes and writes it once, when the button comes up: one undo step.
// no-prop-test: drawing, clicks and drags; `picture`, `sized_image` and
// `image_size::drag_width` have the tests, and the image snapshots show it.
fn render_picture(
    node: &MarkdownNode,
    resolver: &ImageResolver,
    resizes: &ResizeSender,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(picture) = node.data::<Picture>() else {
        return div().into_any_element();
    };
    let colors = Colors::get(cx);
    let at = node.source_range().map_or(0, |r| r.start);
    let group = SharedString::from(format!("{PICTURE}-{at}"));
    #[cfg(test)]
    let hidden = !REVEAL_IMAGE_SIZES.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
    let hidden = true;
    let drag = window.use_keyed_state(
        ElementId::NamedInteger("picture-drag".into(), at as u64),
        cx,
        |_, _| PictureDrag::default(),
    );
    #[cfg(test)]
    if let Some(width) = *PREVIEW_DRAG_WIDTH.lock().unwrap() {
        drag.update(cx, |drag, _| drag.width = Some(width));
    }
    let dragging = drag.read(cx).width;
    let shown = dragging.or(picture.width.map(|w| w as f32));
    let image = img(resolver(&SharedUri::from(picture.src.clone())))
        .object_fit(ObjectFit::Contain)
        .max_w(relative(1.))
        .when_some(shown, |el, width| el.w(px(width)));
    let resize = |width: Option<u32>| ImageResize {
        source: picture.source.clone(),
        src: picture.src.clone(),
        alt: picture.alt.clone(),
        width,
    };
    let buttons = IMAGE_SIZES.iter().enumerate().map(|(ix, (label, width))| {
        let resize = resize(*width);
        let resizes = resizes.clone();
        let selected = picture.width == *width;
        Button::new(ix)
            .ghost()
            .xsmall()
            .tab_stop(false)
            .selected(selected)
            .child(
                div()
                    .px_0p5()
                    .text_size(px(11.5))
                    .text_color(if selected {
                        colors.text
                    } else {
                        colors.text_secondary
                    })
                    .child(*label),
            )
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                resizes.try_send(PreviewEdit::Resize(resize.clone())).ok();
            })
    });
    let chrome = |el: Div| {
        el.absolute()
            .top(px(8.))
            .left(px(8.))
            .p(px(2.))
            .rounded(px(7.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.elevated)
            .shadow_sm()
    };
    let sizes = match dragging {
        // While dragging: the width it will be.
        Some(width) => chrome(div())
            .px(px(8.))
            .py(px(3.))
            .text_size(px(11.5))
            .text_color(colors.text)
            .debug_selector(|| "image-drag-width".into())
            .child(format!("{} px", width.round() as u32))
            .into_any_element(),
        None => chrome(h_flex().gap(px(2.)))
            .id("sizes")
            .debug_selector(|| "image-sizes".into())
            .when(hidden, |el| {
                el.opacity(0.)
                    .group_hover(group.clone(), |style| style.opacity(1.))
            })
            .children(buttons)
            .into_any_element(),
    };
    // The handle: a slim pill on the image's right edge.
    let handle = div()
        .id("resize-handle")
        .absolute()
        .top_0()
        .bottom_0()
        .right(px(-7.))
        .w(px(14.))
        .flex()
        .items_center()
        .justify_center()
        .cursor(CursorStyle::ResizeLeftRight)
        .debug_selector(|| "image-resize-handle".into())
        .when(hidden && dragging.is_none(), |el| {
            el.opacity(0.)
                .group_hover(group.clone(), |style| style.opacity(1.))
        })
        .on_mouse_down(MouseButton::Left, {
            let drag = drag.clone();
            move |event, window, cx| {
                cx.stop_propagation();
                drag.update(cx, |drag, cx| {
                    let width = crate::image_size::drag_width(
                        crate::zoom::unscaled(event.position.x),
                        drag.left,
                        drag.room,
                    );
                    drag.width = Some(width as f32);
                    cx.notify();
                });
                window.refresh();
            }
        })
        .child(
            div()
                .w(px(5.))
                .h(px(40.))
                .max_h(relative(0.6))
                .rounded(px(3.))
                .bg(colors.elevated)
                .border_1()
                .border_color(colors.divider)
                .shadow_sm(),
        );
    // Where the image may go, measured as it's drawn; while dragging, the
    // pointer anywhere in the window moves the edge, and letting go writes
    // the width.
    let tracker = canvas(
        {
            let drag = drag.clone();
            move |bounds, _, cx| {
                drag.update(cx, |drag, _| {
                    // In the layout's units, as the width is written.
                    drag.left = crate::zoom::unscaled(bounds.left());
                    drag.room = crate::zoom::unscaled(bounds.size.width);
                });
            }
        },
        {
            let (drag, resizes, resize) = (drag.clone(), resizes.clone(), resize(None));
            move |_, _, window, cx| {
                if drag.read(cx).width.is_none() {
                    return;
                }
                window.on_mouse_event({
                    let drag = drag.clone();
                    move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture {
                            return;
                        }
                        drag.update(cx, |drag, cx| {
                            if drag.width.is_some() {
                                let width = crate::image_size::drag_width(
                                    crate::zoom::unscaled(event.position.x),
                                    drag.left,
                                    drag.room,
                                );
                                drag.width = Some(width as f32);
                                cx.notify();
                            }
                        });
                        window.refresh();
                    }
                });
                window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture {
                        return;
                    }
                    let Some(width) = drag.update(cx, |drag, _| drag.width.take()) else {
                        return;
                    };
                    let resize = ImageResize {
                        width: Some(width.round() as u32),
                        ..resize.clone()
                    };
                    resizes.try_send(PreviewEdit::Resize(resize)).ok();
                    window.refresh();
                });
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full();
    v_flex()
        .id(ElementId::NamedInteger(PICTURE.into(), at as u64))
        .group(group)
        .relative()
        .w_full()
        .items_start()
        .child(tracker)
        .child(
            div()
                .relative()
                .max_w(relative(1.))
                .child(image)
                .child(handle),
        )
        .child(sizes)
        .into_any_element()
}

/// Quotes drawn by Kanso rather than the text view: the text view draws a
/// quote in its one muted color, the one it also gives finished tasks and
/// list markers, so a theme couldn't make quotes stand out without making
/// finished tasks shout too. Each quote becomes a text view of its own in
/// the theme's quote color, beside a bar.
fn quote_extensions(
    links: Arc<Links>,
    images: ImageResolver,
    resizes: ResizeSender,
    rich: Arc<Rich>,
) -> MarkdownExtensions {
    let parse = |node: &markdown_ast::Node, cx: &MarkdownParseContext<'_>| {
        if let Some(block) = front_matter_node(node) {
            return Some(block);
        }
        if let Some(block) = preview_rich::parse_block(node, cx) {
            return Some(block);
        }
        if let Some(block) = crate::query_view_ui::parse_block(node, cx) {
            return Some(block);
        }
        let source = cx.node_source(node)?;
        if let Some(picture) = picture(node, source) {
            let alt = picture.alt.clone();
            return Some(
                MarkdownNode::new(PICTURE, picture)
                    .text(alt)
                    .markdown(source),
            );
        }
        let markdown_ast::Node::Blockquote(_) = node else {
            return None;
        };
        let body = crate::preview_chunks::showable(&unquote(source)).into_owned();
        Some(
            MarkdownNode::new(QUOTE, Quote(body.clone().into()))
                .text(body)
                .markdown(source),
        )
    };
    let picture_images = images.clone();
    let picture_resizes = resizes.clone();
    let quote_rich = rich.clone();
    let query_links = links.0.clone();
    let extensions = MarkdownExtensions::default()
        .parser_revision(QUOTE_PARSER_REVISION)
        .frontmatter()
        // Unresolved wiki-links, drawn dashed (see `crate::wikilinks`).
        .plugin(crate::wikilinks::UnresolvedLink::new(links.0.clone()))
        // Tasks' due dates, as chips (see `crate::due`).
        .plugin(crate::due::DueChip)
        .block_parser(parse)
        .block_renderer(PICTURE, move |node, window, cx| {
            render_picture(node, &picture_images, &picture_resizes, window, cx)
        })
        .block_renderer(FRONT_MATTER, |_, _, _| div().into_any_element())
        .block_renderer(QUOTE, move |node, _window, cx| {
            let Some(Quote(body)) = node.data::<Quote>() else {
                return div().into_any_element();
            };
            let colors = Colors::get(cx);
            let preview = crate::themes::preview_colors(cx);
            let dark = cx.theme().is_dark();
            let style = text_style(colors, &preview, dark).with_foreground(preview.quote.into());
            let bar = Hsla::from(preview.quote).opacity(0.45);
            let ty = preview_type(cx);
            let id = ElementId::NamedInteger(
                QUOTE.into(),
                node.source_range().map_or(0, |r| r.start) as u64,
            );
            let links = links.clone();
            let resolver = images.clone();
            let rich = quote_rich.clone();
            div()
                .w_full()
                .min_w_0()
                .border_l(px(3.))
                .border_color(bar)
                .pl(px(14.))
                .child(
                    TextView::markdown(id, body.clone())
                        .style(style)
                        .font_family(ty.family)
                        .text_size(px(ty.size))
                        .line_height(relative(typography::LINE_HEIGHT))
                        .text_color(preview.quote)
                        .selectable(true)
                        .markdown_extensions(quote_extensions(
                            links.clone(),
                            resolver.clone(),
                            resizes.clone(),
                            rich,
                        ))
                        .code_block_actions(code_block_actions)
                        .image_source(move |uri| resolver(uri))
                        .on_link_click(move |url, _, _, cx| links.open(url, cx)),
                )
                .into_any_element()
        });
    let rich_for_queries = rich.clone();
    let extensions = crate::query_view_ui::register(extensions, rich_for_queries, query_links);
    preview_rich::register(extensions, rich)
}

/// A code block (or any Markdown) shown alone in the preview's style: the
/// source of a diagram that couldn't be drawn.
// no-prop-test: drawing; the fallback snapshot shows it.
pub(crate) fn code_view(
    id: ElementId,
    markdown: SharedString,
    _window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let preview = crate::themes::preview_colors(cx);
    let style = text_style(colors, &preview, cx.theme().is_dark());
    let markdown: SharedString = crate::preview_chunks::showable(&markdown)
        .into_owned()
        .into();
    let ty = preview_type(cx);
    TextView::markdown(id, markdown)
        .style(style)
        .font_family(ty.family)
        .text_size(px(ty.size))
        .line_height(relative(typography::LINE_HEIGHT))
        .text_color(preview.text)
        .selectable(true)
        .code_block_actions(code_block_actions)
        .into_any_element()
}

/// What sits in a code block's corner: its language, quietly, and a button
/// that copies the code.
fn code_block_actions(block: &CodeBlock, _: &mut Window, cx: &mut App) -> AnyElement {
    let colors = Colors::get(cx);
    let code = block.code();
    h_flex()
        .gap_1()
        .items_center()
        .children(block.lang().filter(|lang| !lang.is_empty()).map(|lang| {
            div()
                .pl_1()
                .font_family(cx.theme().font_family.clone())
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(lang)
        }))
        .child(
            Button::new("copy")
                .ghost()
                .xsmall()
                .icon(Icon::new(Lucide::Copy).text_color(colors.text_muted))
                .tooltip("Copy code")
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                }),
        )
        .into_any_element()
}

/// A heading's size relative to the body, and the gap above it relative to
/// its size, as the text view sets them.
fn heading_metrics(level: u8) -> (f32, f32) {
    let heading = typography::heading(level);
    (heading.scale, heading.gap)
}

/// The gap below a heading, relative to its size, as the text view sets it.
const HEADING_BOTTOM_GAP: f32 = typography::HEADING_BOTTOM_GAP;
/// The gap below ordinary blocks, as [`text_style`] sets it.
const PARAGRAPH_GAP: f32 = typography::PARAGRAPH_GAP;

/// The room above a chunk that starts like `start` (`first` for the note's
/// first): what the text view would leave above that block in the whole
/// note, less the gap the chunk before already leaves below itself.
fn room_above(start: Start, first: bool, size: f32, rem: Pixels) -> Pixels {
    if first {
        return px(TEXT_TOP);
    }
    match start {
        Start::Flush => px(0.),
        Start::Heading(level) => {
            let (scale, gap) = heading_metrics(level);
            (px(size * scale * gap) - rem * PARAGRAPH_GAP).max(px(0.))
        }
    }
}

/// The room below a chunk that ends like `end`: the gap the text view
/// leaves below that block when another follows it.
fn room_below(end: End, size: f32, rem: Pixels) -> Pixels {
    match end {
        End::Last => px(TEXT_TOP),
        End::Gap => rem * PARAGRAPH_GAP,
        End::Heading(level) => px(size * heading_metrics(level).0 * HEADING_BOTTOM_GAP),
    }
}

/// The note ID an in-app link points at.
pub use kanso_engine::links::linked_note;

/// Whether a clicked link may be handed to macOS to open. Only web and mail
/// links are: a note can come from sync or the API, and other schemes can
/// launch apps or open local files.
pub fn is_openable(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once(':') else {
        return false;
    };
    match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => rest.starts_with("//") && rest.len() > 2,
        "mailto" => !rest.is_empty(),
        _ => false,
    }
}

/// Body text in the preview theme's colors; code is set apart by a quiet
/// fill rather than by color, and so is a table's header row, which is also
/// set heavier. Table rows sit on the page itself; a table wider than the
/// column scrolls sideways rather than cutting off its last columns.
/// Finished tasks and list markers, which the text view draws in its muted
/// color, take the theme's `task_done` color (Kanso's secondary text), not
/// the quote's: quotes are drawn by [`quote_extensions`]. Code blocks leave
/// room at the top for their language and copy button. Headings, code and
/// a table's header take colors of their own only when the theme gives
/// them; a first-level heading is set bold. `dark` picks the check mark
/// that reads on the dark theme's task boxes.
// no-prop-test: maps theme colors onto a style; the snapshots show it.
pub(crate) fn text_style(colors: &Colors, p: &PreviewColors, dark: bool) -> TextViewStyle {
    let mut table = StyleRefinement::default().bg(colors.editor);
    table.overflow.x = Some(Overflow::Scroll);
    let selection = match p.selection {
        Some(color) => crate::theme::hsla(color.rgb).opacity(f32::from(color.alpha) / 255.),
        None => Hsla::from(colors.accent).opacity(0.3),
    };
    let mut code_block = StyleRefinement::default()
        .rounded(px(typography::CODE_RADIUS))
        .pt(px(CODE_TOP))
        .border_color(Hsla::from(p.border).opacity(0.6));
    if let Some(code) = p.code_text {
        code_block = code_block.text_color(code);
    }
    let mut table_head = StyleRefinement::default().font_weight(FontWeight::SEMIBOLD);
    if let Some(head) = p.table_header_background {
        table_head = table_head.bg(head);
    }
    let heading = p.heading;
    TextViewStyle::default()
        .with_dark(dark)
        .with_foreground(p.text.into())
        .with_muted_foreground(p.task_done.into())
        .with_link(p.link.into())
        .with_selection(selection)
        .with_code_background(p.code_background.into())
        .with_border(p.border.into())
        .with_paragraph_gap(rems(0.9))
        .with_code_block(code_block)
        .with_inline_code(HighlightStyle {
            background_color: Some(p.inline_code_background.into()),
            color: p.inline_code_text.map(Into::into),
            ..Default::default()
        })
        .with_table(table)
        .with_table_head(table_head)
        .with_heading(move |level| {
            let mut style = StyleRefinement::default();
            if level == 1 {
                style = style.font_weight(FontWeight::BOLD);
            }
            match heading {
                Some(color) => style.text_color(color),
                None => style,
            }
        })
}

/// The preview's text size and font, from Settings › Editor: the font
/// picked there while it is installed, else the theme's.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PreviewType {
    pub size: f32,
    pub family: SharedString,
    /// The app's zoom, which changes every chunk's height like the size.
    pub zoom: f32,
}

// no-prop-test: reads the settings; `fonts::family` has the properties.
pub(crate) fn preview_type(cx: &mut App) -> PreviewType {
    let (size, choice) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| {
            let appearance = &state.config.appearance;
            (
                appearance.preview_font_size,
                appearance.preview_font_family.clone(),
            )
        })
        .unwrap_or((crate::config::DEFAULT_FONT_SIZE, None));
    let installed = crate::fonts::installed(cx);
    PreviewType {
        size,
        family: crate::fonts::family(
            choice.as_deref(),
            &installed,
            cx.theme().font_family.clone(),
        ),
        zoom: crate::zoom::scale(),
    }
}

/// Room above a code block's first line, clear of its language and copy
/// button.
const CODE_TOP: f32 = typography::CODE_TOP;

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ty = preview_type(cx);
        if self.shown_type.as_ref() != Some(&ty) {
            // Another size or face: every chunk's height changes.
            if self.shown_type.is_some() && !self.chunks.is_empty() {
                self.list.remeasure_items(0..self.chunks.len());
            }
            self.shown_type = Some(ty.clone());
        }
        let colors = Colors::get(cx);
        let preview = crate::themes::preview_colors(cx);
        let style = text_style(colors, &preview, cx.theme().is_dark());
        let text_color = preview.text;
        let readable = self.readable;
        let chunks: Vec<(Entity<TextViewState>, Start, End, u64)> = self
            .chunks
            .iter()
            .map(|c| (c.state.clone(), c.start, c.end, c.key))
            .collect();
        let preview = cx.entity().downgrade();
        let resolver: ImageResolver = Arc::new(self.image_resolver());
        let links = Arc::new(Links(self.links.clone()));
        let extensions = self.extensions.clone();
        div().id("preview").size_full().bg(colors.editor).child(
            list(self.list.clone(), move |ix, window, _| {
                let Some((state, start, end, key)) = chunks.get(ix).cloned() else {
                    return div().into_any_element();
                };
                let preview = preview.clone();
                let rem = window.rem_size();
                let resolver = resolver.clone();
                let links = links.clone();
                let extensions = extensions.clone();
                let ty = ty.clone();
                div()
                    .w_full()
                    .pl(px(TEXT_INSET))
                    .pr_8()
                    .pt(room_above(start, ix == 0, ty.size, rem))
                    .pb(room_below(end, ty.size, rem))
                    .child(
                        div()
                            .w_full()
                            .when(readable, |el| el.max_w(px(readable_width(ty.size, false))))
                            .child(
                                TextView::new(&state)
                                    .style(style.clone())
                                    .font_family(ty.family)
                                    .text_size(px(ty.size))
                                    .line_height(relative(typography::LINE_HEIGHT))
                                    .text_color(text_color)
                                    .selectable(true)
                                    .markdown_extensions(extensions)
                                    .code_block_actions(code_block_actions)
                                    .image_source(move |uri| resolver(uri))
                                    .on_link_click(move |url, _, _, cx| links.open(url, cx))
                                    // Only here: a quote's or a query's own
                                    // text view has no map from its text back
                                    // to the note, so its boxes take no clicks.
                                    .on_task_click(move |offset, _, cx| {
                                        preview
                                            .update(cx, |this, _| {
                                                this.task_clicked(ix, key, offset)
                                            })
                                            .ok();
                                    }),
                            ),
                    )
                    .into_any_element()
            })
            .size_full(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{is_openable, linked_note, readable_width};
    use crate::theme::SYSTEM_MONO;
    use gpui_kit::{TextSystem, font, px};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// Ordinary note prose, to measure faces with.
    const PROSE: &str = "Kanso acts as an MCP host. Clients connect to Kanso, discover the \
        tools it offers, and ask before they act. Documents live in D1, attachments in R2. \
        Last writer wins by the update time; the loser stays in history. Rows, not cards. \
        The reason is not recorded in the inspected fields, so earlier QA cannot establish \
        current acceptance.";

    /// How many characters of [`PROSE`] fit in `width`, on average, set in
    /// `family` at `size`.
    fn characters_in(family: &str, size: f32, width: f32) -> f32 {
        let text_system = TextSystem::new(std::sync::Arc::new(gpui_apple::AppleTextSystem::new()));
        let id = text_system.resolve_font(&font(family));
        let advance = |c| f32::from(text_system.advance(id, px(size), c).unwrap().width);
        let total: f32 = PROSE.chars().map(advance).sum();
        width / (total / PROSE.chars().count() as f32)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: at every size the setting allows, the measure holds
        /// about 75 characters of prose in either face, so it grows with the
        /// text instead of crowding more words onto a line at small sizes.
        #[test]
        fn prop_readable_measure_holds_about_75_characters(size in 10f32..=28., mono in any::<bool>()) {
            let family = if mono { SYSTEM_MONO } else { ".SystemUIFont" };
            let characters = characters_in(family, size, readable_width(size, mono));
            prop_assert!((72.0..=82.0).contains(&characters), "{size} {mono}: {characters:.1}");
        }
    }

    /// The default body size gives the preview 560 points of text, and the
    /// wider monospaced face a wider measure.
    #[test]
    fn readable_width_examples() {
        assert_eq!(readable_width(16., false), 560.);
        assert!(readable_width(16., true) > readable_width(16., false));
        // The old fixed 720 points held far more than 75 characters.
        assert!(characters_in(".SystemUIFont", 16., 720.) > 95.);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: a note link built from an ID resolves to that ID.
        #[test]
        fn prop_note_links_resolve_to_their_id(suffix in "[A-Za-z0-9_-]{1,20}", tail in prop::option::of("[?#/][a-z=]{0,8}")) {
            let url = format!("inkdrop://note/{suffix}{}", tail.unwrap_or_default());
            prop_assert_eq!(linked_note(&url), Some(format!("note:{suffix}")));
        }

        /// Refinement: only web and mail links are opened, whatever the
        /// scheme's case.
        #[test]
        fn prop_only_web_and_mail_links_open(scheme in "[a-zA-Z][a-zA-Z0-9+.-]{0,10}", rest in "(//)?[a-z@./]{1,20}") {
            let url = format!("{scheme}:{rest}");
            let allowed = matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https" | "mailto");
            if is_openable(&url) {
                prop_assert!(allowed, "{}", url);
            }
            // Metamorphic: changing the scheme's case changes nothing.
            let flipped = format!("{}:{rest}", scheme.to_ascii_uppercase());
            prop_assert_eq!(is_openable(&flipped), is_openable(&url));
        }

        /// Invariant: other URLs never resolve to a note.
        #[test]
        fn prop_other_urls_are_not_note_links(url in "(https?|file|inkdrop)://[a-z:/.]{0,20}") {
            prop_assume!(!url.starts_with("inkdrop://note"));
            prop_assert_eq!(linked_note(&url), None);
        }
    }

    /// Incremental equals full: however a run of edits arrives (each shown,
    /// or several while a split is under way), the chunks shown once it
    /// settles are the chunks of the last text, and a chunk no edit touched
    /// keeps its view.
    // no-prop-test: drives the view through seeded random edits; the splice
    // itself is a property in preview_chunks.rs.
    #[gpui_kit::test]
    fn edits_settle_on_the_chunks_of_the_last_text(cx: &mut gpui_kit::TestAppContext) {
        use super::Preview;
        use crate::preview_chunks::prepare;
        use gpui_kit::AppContext as _;

        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
        let (links, _rx) = async_channel::unbounded();
        let (resizes, _resized) = async_channel::unbounded();
        let preview = cx.new(|cx| Preview::new(store, links, resizes, cx));
        let mut rng = fastrand::Rng::with_seed(20261017);
        let pieces = [
            "word ",
            "\n",
            "\n\n",
            "## Head\n\n",
            "- item\n",
            "```\n",
            "Para. ",
            "> q\n",
        ];
        let mut text: String = (0..200)
            .map(|n| format!("## Part {n}\n\nText {n}.\n\n"))
            .collect();
        let set = |cx: &mut gpui_kit::TestAppContext, text: &str| {
            let text = text.to_owned();
            preview.update(cx, |p, cx| p.set_text(text.into(), cx));
        };
        set(cx, &text);
        cx.run_until_parked();
        for round in 0..60 {
            let before = cx.read(|cx| preview.read(cx).chunk_views());
            // One to four edits before the preview gets to run.
            for _ in 0..rng.usize(1..=4) {
                let mut at = rng.usize(0..=text.len());
                while !text.is_char_boundary(at) {
                    at -= 1;
                }
                if rng.bool() || text.is_empty() {
                    text.insert_str(at, pieces[rng.usize(0..pieces.len())]);
                } else {
                    let end = (at + rng.usize(1..8)).min(text.len());
                    let end = (end..=text.len())
                        .find(|e| text.is_char_boundary(*e))
                        .unwrap();
                    text.replace_range(at..end, "");
                }
                set(cx, &text);
            }
            cx.run_until_parked();
            let (shown, views) = cx.read(|cx| {
                let p = preview.read(cx);
                (p.shown().to_string(), p.chunk_views())
            });
            assert_eq!(shown, text, "round {round}");
            let keys: Vec<u64> = views.iter().map(|(k, _)| *k).collect();
            let expected: Vec<u64> = prepare(&text).iter().map(|c| c.key).collect();
            assert_eq!(keys, expected, "round {round}");
            // The chunks before the first that changed keep their views.
            let kept = before
                .iter()
                .zip(&views)
                .take_while(|(a, b)| a.0 == b.0)
                .count();
            assert!(before[..kept].iter().zip(&views).all(|(a, b)| a.1 == b.1));
        }
    }

    /// Lines of a quote's body: anything but line breaks.
    fn quote_lines() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[^\r\n]{0,24}", 1..8)
    }

    fn quote(lines: &[String]) -> String {
        lines
            .iter()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Round trip: quoting lines with `> ` and unquoting gives them
        /// back, leading spaces, tabs and inner `>` included.
        #[test]
        fn prop_unquote_inverts_quote(lines in quote_lines()) {
            prop_assert_eq!(super::unquote(&quote(&lines)), lines.join("\n"));
        }

        /// Round trip for a nested quote: one level comes off at a time.
        #[test]
        fn prop_unquote_peels_one_level(lines in quote_lines()) {
            let inner = quote(&lines);
            let outer = quote(&inner.split('\n').map(str::to_owned).collect::<Vec<_>>());
            prop_assert_eq!(super::unquote(&outer), inner);
        }

        /// Invariants: the line count holds, and a line without a marker (a
        /// lazy continuation) comes through as it was.
        #[test]
        fn prop_unquote_keeps_lines(lines in quote_lines(), indent in "[ \t]{0,3}") {
            let text: Vec<String> = lines
                .iter()
                .enumerate()
                .map(|(ix, line)| if ix % 2 == 0 { format!("{indent}> {line}") } else { line.clone() })
                .collect();
            let out = super::unquote(&text.join("\n"));
            prop_assert_eq!(out.split('\n').count(), text.len());
            for ((ix, line), got) in text.iter().enumerate().zip(out.split('\n')) {
                if ix % 2 == 1 && !line.trim_start_matches([' ', '\t']).starts_with('>') {
                    prop_assert_eq!(got, line.as_str());
                }
            }
        }
    }

    #[test]
    fn unquote_examples() {
        use super::unquote;
        assert_eq!(unquote("> a\n> b"), "a\nb");
        assert_eq!(unquote(">a\n>b"), "a\nb");
        assert_eq!(unquote("> a\nlazy"), "a\nlazy");
        assert_eq!(unquote("  > - item\n  > - two"), "- item\n- two");
        assert_eq!(unquote("> > nested"), "> nested");
        assert_eq!(unquote(">     code"), "    code");
        assert_eq!(unquote(">"), "");
    }

    /// An attached or web image alone on its line gets size buttons and a
    /// handle; one in a sentence, or from a path the preview can't load, stays the text view's.
    #[test]
    fn attached_images_on_their_own_get_sizes() {
        let first = |text: &str| {
            let root = markdown::to_mdast(text, &markdown::ParseOptions::gfm()).unwrap();
            let node = root.children().unwrap()[0].clone();
            let at = node.position().unwrap();
            super::picture(&node, &text[at.start.offset..at.end.offset])
                .map(|p| (p.src, p.alt, p.width, p.source))
        };
        let src = "inkdrop://file:abc".to_owned();
        assert_eq!(
            first("![Cat](inkdrop://file:abc)\n"),
            Some((
                src.clone(),
                "Cat".into(),
                None,
                "![Cat](inkdrop://file:abc)".into()
            ))
        );
        let tag = "<img src=\"inkdrop://file:abc\" width=\"240\">";
        assert_eq!(
            first(tag),
            Some((src, String::new(), Some(240), tag.to_owned()))
        );
        assert_eq!(
            first("![Cat|300](inkdrop://file:abc)").map(|p| (p.1, p.2)),
            Some(("Cat".into(), Some(300)))
        );
        assert_eq!(
            first("![](https://x.io/a.png)").map(|p| p.0),
            Some("https://x.io/a.png".into())
        );
        assert_eq!(first("![](a.png)"), None);
        assert_eq!(first("Look: ![](inkdrop://file:abc)"), None);
    }

    #[test]
    fn note_links() {
        assert_eq!(
            linked_note("inkdrop://note/abc123"),
            Some("note:abc123".into())
        );
        assert_eq!(
            linked_note("inkdrop://note:abc123"),
            Some("note:abc123".into())
        );
        assert_eq!(linked_note("inkdrop://note/"), None);
        assert_eq!(linked_note("https://example.com"), None);
    }

    #[test]
    fn front_matter_is_drawn_as_nothing() {
        let mut options = markdown::ParseOptions::gfm();
        options.constructs.frontmatter = true;
        let root = markdown::to_mdast("---\ntitle: x\n---\n# Heading\n", &options).unwrap();
        let first = root.children().unwrap().first().unwrap();
        let block = super::front_matter_node(first).expect("front matter is a block");
        assert_eq!(block.name(), super::FRONT_MATTER);
        assert_eq!(block.as_text(), "");
        assert!(super::front_matter_node(&root.children().unwrap()[1]).is_none());
    }

    #[test]
    fn openable_links() {
        assert!(is_openable("https://example.com/a?b#c"));
        assert!(is_openable("HTTP://example.com"));
        assert!(is_openable("mailto:someone@example.com"));
        for url in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "x-apple.systempreferences:com.apple.preference",
            "smb://server/share",
            "https:",
            "https://",
            "example.com",
        ] {
            assert!(!is_openable(url), "{url}");
        }
    }
}
