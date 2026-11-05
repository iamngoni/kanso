//! The PDF reader (issue #43, steps 3 and 4): the editor's right pane while a
//! PDF is open, showing the document as one continuous column of pages.
//!
//! The view never touches PDF bytes or PDFium. A [`session::Session`] reads
//! the attachment on a background thread, hands it to `kanso-pdf-worker` (a
//! separate, contained process, see `kanso_pdf::client`) and draws the pages
//! the reader asks for. The reader knows every page's size from the start, so
//! the column is laid out at once ([`layout`]); it asks for the pages on
//! screen first and the ones it is heading for next ([`prefetch`]), keeps what
//! it is sent within a pixel budget ([`budget`]), and lets go of the farthest
//! pages first. Zoom changes the layout at once and draws again after a short
//! pause: the old picture stays, scaled, so a page never blanks.
//!
//! Dark mode asks the worker for smart invert (see `kanso_pdf::invert`): text
//! light on a dark page, pictures in their own colours. A toggle shows the
//! page as printed instead, remembered per PDF.
//!
//! A password-protected PDF asks for its password once per open. The password
//! goes to the worker for that open and nowhere else: it is not kept in the
//! reader, the store or the session.
//!
//! Page textures live in memory only, never on disk: a page image of a
//! locked PDF would be a plaintext leak.

mod budget;
mod layout;
mod model;
mod prefetch;
mod session;
mod texture;

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_pdf::client::WorkerPaths;
use kanso_store::Store;

use budget::{PAGE_BUDGET_BYTES, PixelBudget, THUMB_BUDGET_BYTES};
use layout::{Layout, MARGIN, PAGE_GAP, Zoom};
pub use model::ReaderState;
use model::{THUMB_WIDTH, original_key, render_width, thumb_render_width};
use prefetch::Heading;
use session::{Event, Job, Session};

use crate::theme::Colors;
use crate::zoom::px;

/// How wide the thumbnail rail is, in logical pixels.
const RAIL_WIDTH: f32 = 148.;
/// The space between thumbnails (room for the page number) and above the first.
const THUMB_GAP: f32 = 26.;
const THUMB_PAD: f32 = 12.;
/// How long after the last zoom change the pages are drawn again.
const ZOOM_SETTLE: Duration = Duration::from_millis(150);
/// The window size assumed before the first frame has measured it.
const FALLBACK_VIEWPORT: (f32, f32) = (800., 600.);
/// What a page looks like before its picture arrives in a dark reader: the
/// colour a white page is turned into.
const DARK_PAGE: u32 = 0x1b1b1b;

/// What the reader tells the editor.
pub enum PdfEvent {
    /// Back to the note: the button, or Escape.
    Close,
}

/// A page picture, and what it was drawn for.
struct Tex {
    image: Arc<RenderImage>,
    /// The width asked of the worker (not the width it drew, which it caps).
    asked_px: u32,
    invert: bool,
}

/// What the last frame saw, to tell when the reader must plan again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct View {
    width: i32,
    height: i32,
    top: i32,
    scale_percent: i32,
    invert: bool,
}

/// The reader for one PDF attachment.
pub struct PdfReader {
    store: Arc<Store>,
    /// The attachment's document id (`file:…`).
    file_id: String,
    name: String,
    state: ReaderState,
    session: Option<Session>,
    events: Option<Task<()>>,
    /// Where to find the worker; `None` when this build has none.
    paths: Option<WorkerPaths>,

    /// Each page's size in points, once the document is open.
    sizes: Vec<[f32; 2]>,
    zoom: Zoom,
    /// Logical pixels per point now.
    scale: f32,
    layout: Layout,
    thumb_layout: Layout,
    /// The page to scroll to when the document opens, 0-based.
    jump: Option<usize>,
    current: usize,
    heading: Heading,
    last_top: f32,
    seen: View,
    /// A zoom was just changed and the pages are not yet drawn again.
    zooming: bool,
    settle: Option<Task<()>>,
    /// The first frame has not measured the window yet.
    unmeasured: bool,

    pages: PixelBudget<Tex>,
    thumbs: PixelBudget<Tex>,
    /// Jobs the worker could not draw, so they are not asked again forever.
    failed: std::collections::HashSet<Job>,
    /// Textures let go of, to free on the GPU at the next frame.
    trash: Vec<Arc<RenderImage>>,

    display_scale: f32,
    dark: bool,
    /// Show the page as printed, even in a dark reader.
    original: bool,
    rail_open: bool,

    scroll: ScrollHandle,
    rail_scroll: ScrollHandle,
    password: Entity<InputState>,
    page_field: Entity<InputState>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PdfEvent> for PdfReader {}

impl PdfReader {
    /// Opens `file_id` (its document id) and shows page `page` (1-based;
    /// clamped to the document).
    pub fn new(
        store: Arc<Store>,
        file_id: String,
        name: String,
        page: u32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::with_paths(
            store,
            file_id,
            name,
            page,
            kanso_pdf::client::locate(),
            window,
            cx,
        )
    }

    /// As [`PdfReader::new`], with the worker found by the caller (tests).
    pub fn with_paths(
        store: Arc<Store>,
        file_id: String,
        name: String,
        page: u32,
        paths: Option<WorkerPaths>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Password")
        });
        let page_field = cx.new(|cx| InputState::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(
                &password,
                window,
                |this: &mut PdfReader, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.submit_password(window, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &page_field,
                window,
                |this: &mut PdfReader, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.go_to_typed_page(window, cx);
                    }
                },
            ),
        ];
        let original = store
            .meta(&original_key(&file_id))
            .ok()
            .flatten()
            .is_some_and(|value| value == "1");
        let mut reader = PdfReader {
            store,
            file_id,
            name,
            state: ReaderState::Loading,
            session: None,
            events: None,
            paths,
            sizes: Vec::new(),
            zoom: Zoom::FitWidth,
            scale: 1.,
            layout: Layout::new(&[], 1., PAGE_GAP, MARGIN),
            thumb_layout: Layout::new(&[], 1., PAGE_GAP, MARGIN),
            jump: Some(page.max(1) as usize - 1),
            current: 0,
            heading: Heading::Down,
            last_top: 0.,
            seen: View::default(),
            zooming: false,
            settle: None,
            unmeasured: true,
            pages: PixelBudget::new(PAGE_BUDGET_BYTES),
            thumbs: PixelBudget::new(THUMB_BUDGET_BYTES),
            failed: Default::default(),
            trash: Vec::new(),
            display_scale: window.scale_factor(),
            dark: false,
            original,
            rail_open: true,
            scroll: ScrollHandle::new(),
            rail_scroll: ScrollHandle::new(),
            password,
            page_field,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        // The textures leave the window's atlas with the reader: they are the
        // document in the clear, and up to 64 MB a page.
        cx.on_release(|reader: &mut PdfReader, cx| reader.release_textures(cx))
            .detach();
        reader.start(window, cx);
        reader
    }

    /// Lets go of every texture, in every window.
    fn release_textures(&mut self, cx: &mut App) {
        self.forget_pages();
        let images: Vec<Arc<RenderImage>> = self.trash.drain(..).collect();
        // Deferred: the window this runs in may be mid-update, and out of the
        // app's reach until it ends.
        cx.defer(move |cx| {
            for image in images {
                cx.drop_image(image, None);
            }
        });
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// Starts a worker for the document and listens to what it says.
    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state = ReaderState::Loading;
        self.forget_pages();
        let (session, events) = Session::start(
            self.store.clone(),
            self.file_id.clone(),
            self.paths.clone(),
        );
        self.session = Some(session);
        self.events = Some(cx.spawn_in(window, async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update_in(cx, |this, window, cx| this.on_event(event, window, cx))
                    .is_err()
                {
                    break;
                }
            }
        }));
        cx.notify();
    }

    fn forget_pages(&mut self) {
        for tex in self.pages.clear().into_iter().chain(self.thumbs.clear()) {
            self.trash.push(tex.image);
        }
        self.failed.clear();
    }

    fn on_event(&mut self, event: Event, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            Event::Opened { sizes } => {
                self.thumb_layout =
                    Layout::uniform_width(&sizes, THUMB_WIDTH, THUMB_GAP, THUMB_PAD);
                self.sizes = sizes;
                self.layout = Layout::new(&[], 1., PAGE_GAP, MARGIN);
                self.state = ReaderState::Showing {
                    page: 1,
                    pages: self.sizes.len() as u32,
                };
                self.relayout();
                if let Some(page) = self.jump.take() {
                    self.go_to_page(page, window, cx);
                }
                self.sync(window, cx);
            }
            Event::NeedsPassword { wrong } => {
                self.state = ReaderState::NeedsPassword { wrong };
                let field = self.password.read(cx).focus_handle(cx);
                window.focus(&field, cx);
            }
            Event::Rendered { job, image, bytes } => {
                let tex = Tex {
                    image,
                    asked_px: job.width_px,
                    invert: job.invert,
                };
                let (cache, keep, focus) = if job.thumb {
                    let keep = self.thumb_layout_visible();
                    (&mut self.thumbs, keep, self.current)
                } else {
                    (&mut self.pages, self.page_range(), self.current)
                };
                if let Some(old) = cache.insert(job.page, tex, bytes) {
                    self.trash.push(old.image);
                }
                let gone = cache.evict(keep, focus);
                self.trash.extend(gone.into_iter().map(|tex| tex.image));
            }
            Event::RenderFailed { job } => {
                self.failed.insert(job);
            }
            Event::Failed(words) => self.fail(words),
        }
        cx.notify();
    }

    fn fail(&mut self, words: String) {
        self.forget_pages();
        self.session = None;
        self.state = ReaderState::Failed(words);
    }

    /// Gives the worker the password that was typed. It is read from the
    /// field, the field is emptied, and the word goes to the session and
    /// nowhere else.
    fn submit_password(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let typed = self.password.read(cx).value().to_string();
        if typed.is_empty() {
            return;
        }
        self.password
            .update(cx, |field, cx| field.set_value("", window, cx));
        if let Some(session) = &self.session {
            session.submit_password(typed);
            self.state = ReaderState::Loading;
        }
        cx.notify();
    }

    fn retry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.jump = Some(self.current);
        self.start(window, cx);
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        cx.emit(PdfEvent::Close);
    }

    // ---- what the window shows -------------------------------------------------

    /// The scroll area's size in pixels (a guess before the first frame).
    fn viewport(&self) -> (f32, f32) {
        let bounds = self.scroll.bounds();
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        if w > 1. && h > 1. {
            (w, h)
        } else {
            FALLBACK_VIEWPORT
        }
    }

    fn scroll_top(&self) -> f32 {
        -f32::from(self.scroll.offset().y)
    }

    fn set_scroll_top(&self, y: f32) {
        let (_, height) = self.viewport();
        let y = y.clamp(0., self.layout.max_scroll(height));
        let offset = self.scroll.offset();
        self.scroll.set_offset(point(offset.x, gpui_kit::px(-y)));
    }

    fn page_range(&self) -> std::ops::Range<usize> {
        let (_, height) = self.viewport();
        self.layout.visible(self.scroll_top(), height)
    }

    fn thumb_layout_visible(&self) -> std::ops::Range<usize> {
        if !self.rail_open {
            return 0..0;
        }
        let bounds = self.rail_scroll.bounds();
        let height = f32::from(bounds.size.height);
        let height = if height > 1. { height } else { FALLBACK_VIEWPORT.1 };
        self.thumb_layout
            .visible(-f32::from(self.rail_scroll.offset().y), height)
    }

    /// Whether pages are drawn for a dark reader.
    fn invert(&self) -> bool {
        self.dark && !self.original
    }

    /// The width a page should be drawn at now.
    fn wanted_width(&self, page: usize) -> u32 {
        render_width(self.layout.size(page)[0], self.display_scale)
    }

    fn page_is_current(&self, page: usize) -> bool {
        self.pages.get(page).is_some_and(|tex| {
            tex.asked_px == self.wanted_width(page) && tex.invert == self.invert()
        })
    }

    fn thumb_is_current(&self, page: usize) -> bool {
        self.thumbs.get(page).is_some_and(|tex| {
            tex.asked_px == thumb_render_width(self.display_scale) && tex.invert == self.invert()
        })
    }

    /// The scale a Fit width or a percentage comes to at this window size.
    fn relayout(&mut self) -> bool {
        if self.sizes.is_empty() {
            return false;
        }
        let (width, _) = self.viewport();
        let widest = self.sizes.iter().map(|s| s[0]).fold(0., f32::max);
        let scale = self.zoom.scale(widest, width - 2. * MARGIN);
        if self.layout.pages() == self.sizes.len() && (scale - self.scale).abs() < 1e-4 {
            return false;
        }
        let first = self.layout.pages() == 0;
        let anchor = self.layout.anchor_at(self.scroll_top());
        self.scale = scale;
        self.layout = Layout::new(&self.sizes, scale, PAGE_GAP, MARGIN);
        if first {
            self.set_scroll_top(0.);
        } else {
            self.set_scroll_top(self.layout.offset_of(anchor));
        }
        true
    }

    /// The zoom as a whole percent, for the toolbar and for stepping.
    fn shown_percent(&self) -> u32 {
        (self.scale * 100.).round().max(1.) as u32
    }

    fn set_zoom(&mut self, zoom: Zoom, window: &mut Window, cx: &mut Context<Self>) {
        if zoom == self.zoom {
            return;
        }
        self.zoom = zoom;
        self.relayout();
        // A page wider than the window starts centred.
        let (width, _) = self.viewport();
        let content = (self.layout.max_width() + 2. * MARGIN).max(width);
        let offset = self.scroll.offset();
        self.scroll
            .set_offset(point(gpui_kit::px(-((content - width) / 2.)), offset.y));
        // The pages are drawn again once the zoom stops moving; until then
        // the pictures on screen are stretched to the new size.
        self.zooming = true;
        self.settle = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(ZOOM_SETTLE).await;
            this.update_in(cx, |this, window, cx| {
                this.zooming = false;
                this.sync(window, cx);
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    fn zoom_by(&mut self, direction: i32, window: &mut Window, cx: &mut Context<Self>) {
        let zoom = Zoom::step(self.shown_percent(), direction);
        self.set_zoom(zoom, window, cx);
    }

    /// Scrolls so `page` (0-based) is at the top.
    fn go_to_page(&mut self, page: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.layout.pages() == 0 {
            self.jump = Some(page);
            return;
        }
        let page = page.min(self.layout.pages() - 1);
        self.set_scroll_top(self.layout.top(page) - MARGIN / 2.);
        self.sync(window, cx);
        cx.notify();
    }

    fn go_to_typed_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let typed = self.page_field.read(cx).value().trim().to_owned();
        if let Ok(number) = typed.parse::<usize>() {
            self.go_to_page(number.max(1) - 1, window, cx);
        }
        window.focus(&self.focus, cx);
    }

    fn scroll_by(&mut self, delta: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.set_scroll_top(self.scroll_top() + delta);
        self.sync(window, cx);
        cx.notify();
    }

    fn toggle_original(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.original = !self.original;
        let key = original_key(&self.file_id);
        if self.original {
            self.store.set_meta(&key, "1").ok();
        } else {
            self.store.remove_meta(&key).ok();
        }
        self.sync(window, cx);
        cx.notify();
    }

    fn toggle_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rail_open = !self.rail_open;
        self.sync(window, cx);
        cx.notify();
    }

    /// Works out what to draw and asks the worker for it, most wanted first,
    /// and lets go of the pages that are far away.
    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ReaderState::Showing { pages, .. } = self.state else {
            return;
        };
        let (_, height) = self.viewport();
        let top = self.scroll_top();
        self.heading = Heading::after(self.heading, self.last_top, top);
        self.last_top = top;
        let visible = self.layout.visible(top, height);
        let current = self.layout.current(top, height);
        if current != self.current || self.state != (ReaderState::Showing { page: current as u32 + 1, pages }) {
            self.current = current;
            self.state = ReaderState::Showing {
                page: current as u32 + 1,
                pages,
            };
            self.show_page_number(window, cx);
            self.reveal_thumb(current);
        }

        let invert = self.invert();
        let mut jobs: Vec<Job> = prefetch::plan(
            visible.clone(),
            self.layout.pages(),
            self.heading,
            |page| self.page_is_current(page),
        )
        .into_iter()
        .map(|page| Job {
            thumb: false,
            page,
            width_px: self.wanted_width(page),
            invert,
        })
        .filter(|job| !self.failed.contains(job))
        .collect();

        let rail = self.thumb_layout_visible();
        // The rail's pages come after the column's: they are small and quick.
        let around = rail.start.saturating_sub(2)..(rail.end + 2).min(self.thumb_layout.pages());
        if self.rail_open && !rail.is_empty() {
            jobs.extend(
                around
                    .filter(|page| !self.thumb_is_current(*page))
                    .map(|page| Job {
                        thumb: true,
                        page,
                        width_px: thumb_render_width(self.display_scale),
                        invert,
                    })
                    .filter(|job| !self.failed.contains(job)),
            );
        }
        if let Some(session) = &self.session {
            session.set_jobs(jobs);
        }

        let gone = self.pages.evict(visible, current);
        self.trash.extend(gone.into_iter().map(|tex| tex.image));
        let gone = self.thumbs.evict(rail, current);
        self.trash.extend(gone.into_iter().map(|tex| tex.image));
    }

    /// Keeps the page field showing the page being read.
    fn show_page_number(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focused = self.page_field.read(cx).focus_handle(cx).is_focused(window);
        if !focused {
            let text = (self.current + 1).to_string();
            self.page_field
                .update(cx, |field, cx| field.set_value(text, window, cx));
        }
    }

    /// Scrolls the rail so the thumbnail of `page` is in it.
    fn reveal_thumb(&self, page: usize) {
        if !self.rail_open {
            return;
        }
        let bounds = self.rail_scroll.bounds();
        let height = f32::from(bounds.size.height);
        if height <= 1. {
            return;
        }
        let top = -f32::from(self.rail_scroll.offset().y);
        let (start, size) = (self.thumb_layout.top(page), self.thumb_layout.size(page)[1]);
        if start < top || start + size > top + height {
            let to = (start - THUMB_PAD).clamp(0., self.thumb_layout.max_scroll(height));
            let offset = self.rail_scroll.offset();
            self.rail_scroll
                .set_offset(point(offset.x, gpui_kit::px(-to)));
        }
    }

    // ---- test hooks ---------------------------------------------------------------

    #[cfg(test)]
    pub(crate) fn state(&self) -> &ReaderState {
        &self.state
    }

    /// The worker's process id, for the test that kills it.
    #[cfg(test)]
    pub(crate) fn worker_pid(&self) -> Option<u32> {
        self.session.as_ref().and_then(Session::pid)
    }

    #[cfg(test)]
    pub(crate) fn page_texture(&self, page: usize) -> Option<Arc<RenderImage>> {
        self.pages.get(page).map(|tex| tex.image.clone())
    }

    #[cfg(test)]
    pub(crate) fn thumb_texture(&self, page: usize) -> Option<Arc<RenderImage>> {
        self.thumbs.get(page).map(|tex| tex.image.clone())
    }

    /// Bytes of page bitmap held, and the limit.
    #[cfg(test)]
    pub(crate) fn page_bytes(&self) -> (usize, usize) {
        (self.pages.used(), self.pages.limit())
    }

    #[cfg(test)]
    pub(crate) fn limit_pages_to(&mut self, bytes: usize) {
        self.pages = PixelBudget::new(bytes);
    }

    #[cfg(test)]
    pub(crate) fn visible_pages(&self) -> std::ops::Range<usize> {
        self.page_range()
    }

    #[cfg(test)]
    pub(crate) fn shown_zoom_percent(&self) -> u32 {
        self.shown_percent()
    }

    #[cfg(test)]
    pub(crate) fn scroll_to(&mut self, y: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.set_scroll_top(y);
        self.sync(window, cx);
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn column_height(&self) -> f32 {
        self.layout.total_height()
    }

    #[cfg(test)]
    pub(crate) fn zoom_to(&mut self, zoom: Zoom, window: &mut Window, cx: &mut Context<Self>) {
        self.set_zoom(zoom, window, cx);
    }

    #[cfg(test)]
    pub(crate) fn pick_original(&mut self, original: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.original != original {
            self.toggle_original(window, cx);
        }
    }

    #[cfg(test)]
    pub(crate) fn type_password(
        &mut self,
        password: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.password
            .update(cx, |field, cx| field.set_value(password, window, cx));
        self.submit_password(window, cx);
    }

    /// Everything the reader holds as text, for the test that the password is
    /// nowhere in it.
    #[cfg(test)]
    pub(crate) fn debug_dump(&self, cx: &App) -> String {
        format!(
            "{:?} {} {} {:?} {:?} {}",
            self.state,
            self.name,
            self.file_id,
            self.session,
            self.zoom,
            self.password.read(cx).value()
        )
    }

    // ---- drawing ----------------------------------------------------------------------

    fn button(
        &self,
        id: &'static str,
        icon: Lucide,
        tip: &'static str,
        selected: bool,
    ) -> Button {
        Button::new(id)
            .debug_selector(|| id.into())
            .ghost()
            .small()
            .icon(Icon::new(icon).size(px(14.)))
            .tooltip(tip)
            .selected(selected)
    }

    fn header(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let showing = matches!(self.state, ReaderState::Showing { .. });
        let pages = match self.state {
            ReaderState::Showing { pages, .. } => pages,
            _ => 0,
        };
        h_flex()
            .flex_none()
            .h(px(44.))
            .px(px(12.))
            .gap(px(6.))
            .items_center()
            .border_b_1()
            .border_color(colors.divider)
            .child(
                Button::new("pdf-back")
                    .debug_selector(|| "pdf-back".into())
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::ArrowLeft).size(px(14.)))
                    .label("Back to Note")
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .px(px(6.))
                    .text_size(px(13.))
                    .text_color(colors.text)
                    .debug_selector(|| "pdf-name".into())
                    .child(self.name.clone()),
            )
            .when(showing, |bar| {
                bar.child(
                    self.button("pdf-rail", Lucide::PanelLeft, "Thumbnails", self.rail_open)
                        .on_click(cx.listener(|this, _, window, cx| this.toggle_rail(window, cx))),
                )
                .child(
                    h_flex()
                        .gap(px(4.))
                        .items_center()
                        .text_size(px(12.))
                        .text_color(colors.text_secondary)
                        .child(
                            div()
                                .w(px(46.))
                                .debug_selector(|| "pdf-page-field".into())
                                .child(Input::new(&self.page_field).small()),
                        )
                        .child(
                            div()
                                .debug_selector(|| "pdf-page-label".into())
                                .child(format!("of {pages}")),
                        ),
                )
                .child(
                    self.button("pdf-zoom-out", Lucide::ZoomOut, "Zoom Out", false)
                        .on_click(cx.listener(|this, _, window, cx| this.zoom_by(-1, window, cx))),
                )
                .child(
                    div()
                        .w(px(42.))
                        .flex()
                        .justify_center()
                        .text_size(px(12.))
                        .text_color(colors.text_secondary)
                        .debug_selector(|| "pdf-zoom-label".into())
                        .child(format!("{}%", self.shown_percent())),
                )
                .child(
                    self.button("pdf-zoom-in", Lucide::ZoomIn, "Zoom In", false)
                        .on_click(cx.listener(|this, _, window, cx| this.zoom_by(1, window, cx))),
                )
                .child(
                    self.button(
                        "pdf-fit",
                        Lucide::Maximize,
                        "Fit width",
                        self.zoom == Zoom::FitWidth,
                    )
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.set_zoom(Zoom::FitWidth, window, cx)
                    })),
                )
                .child(
                    self.button(
                        "pdf-actual",
                        Lucide::Scan,
                        "Actual size",
                        self.zoom == Zoom::ACTUAL,
                    )
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.set_zoom(Zoom::ACTUAL, window, cx)
                    })),
                )
                .when(self.dark, |bar| {
                    bar.child(
                        self.button(
                            "pdf-original",
                            Lucide::SunMoon,
                            if self.original {
                                "Invert for dark mode"
                            } else {
                                "Show the page as printed"
                            },
                            self.original,
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.toggle_original(window, cx)
                        })),
                    )
                })
            })
    }

    fn body(&mut self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        match self.state.clone() {
            ReaderState::Showing { .. } => self.pages_view(colors, cx),
            ReaderState::NeedsPassword { wrong } => self.password_prompt(wrong, colors, cx),
            ReaderState::Failed(words) => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap(px(12.))
                .child(
                    Icon::new(Lucide::TriangleAlert)
                        .size(px(20.))
                        .text_color(colors.text_secondary),
                )
                .child(
                    div()
                        .debug_selector(|| "pdf-error".into())
                        .text_size(px(14.))
                        .text_color(colors.text_secondary)
                        .child(words),
                )
                .child(
                    Button::new("pdf-retry")
                        .debug_selector(|| "pdf-retry".into())
                        .small()
                        .label("Try Again")
                        .on_click(cx.listener(|this, _, window, cx| this.retry(window, cx))),
                )
                .into_any_element(),
            ReaderState::Loading => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .debug_selector(|| "pdf-loading".into())
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child("Opening…"),
                )
                .into_any_element(),
        }
    }

    /// The password prompt: what the PDF needs, a masked field, and a plain
    /// promise about what happens to the word.
    fn password_prompt(&self, wrong: bool, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .debug_selector(|| "pdf-password".into())
                    .w(px(320.))
                    .gap(px(12.))
                    .p(px(20.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.elevated)
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .items_center()
                            .child(
                                Icon::new(Lucide::Lock)
                                    .size(px(16.))
                                    .text_color(colors.text_secondary),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(colors.text)
                                    .child("This PDF is locked"),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_secondary)
                            .child("Enter its password to read it. Kanso doesn't keep the password."),
                    )
                    .child(
                        div()
                            .debug_selector(|| "pdf-password-field".into())
                            .child(Input::new(&self.password)),
                    )
                    .when(wrong, |card| {
                        card.child(
                            div()
                                .debug_selector(|| "pdf-password-wrong".into())
                                .text_size(px(12.))
                                .text_color(colors.text_secondary)
                                .child("That password isn't right."),
                        )
                    })
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .justify_end()
                            .child(
                                Button::new("pdf-password-cancel")
                                    .debug_selector(|| "pdf-password-cancel".into())
                                    .small()
                                    .label("Cancel")
                                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
                            )
                            .child(
                                Button::new("pdf-password-open")
                                    .debug_selector(|| "pdf-password-open".into())
                                    .small()
                                    .primary()
                                    .label("Open")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.submit_password(window, cx)
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn pages_view(&mut self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let (viewport_w, viewport_h) = self.viewport();
        let top = self.scroll_top();
        let visible = self.layout.visible(top, viewport_h);
        let content_w = (self.layout.max_width() + 2. * MARGIN).max(viewport_w);
        let blank = if self.invert() {
            rgb(DARK_PAGE)
        } else {
            rgb(0xffffff)
        };

        let mut column = div()
            .relative()
            .w(gpui_kit::px(content_w))
            .h(gpui_kit::px(self.layout.total_height()));
        for page in visible {
            self.pages.touch(page);
            let [w, h] = self.layout.size(page);
            let image = self.pages.get(page).map(|tex| tex.image.clone());
            column = column.child(
                div()
                    .absolute()
                    .left(gpui_kit::px((content_w - w) / 2.))
                    .top(gpui_kit::px(self.layout.top(page)))
                    .w(gpui_kit::px(w))
                    .h(gpui_kit::px(h))
                    .bg(blank)
                    .border_1()
                    .border_color(colors.divider)
                    .shadow_md()
                    .debug_selector(move || format!("pdf-page-{page}"))
                    .when_some(image, |frame, image| {
                        frame.child(
                            img(ImageSource::Render(image))
                                .object_fit(ObjectFit::Fill)
                                .size_full(),
                        )
                    }),
            );
        }

        let pages = div()
            .id("pdf-pages")
            .flex_1()
            .min_w_0()
            .h_full()
            .relative()
            .child(
                div()
                    .id("pdf-scroll")
                    .debug_selector(|| "pdf-pages".into())
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&self.scroll)
                    .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| window.focus(&this.focus, cx)),
                    )
                    .child(column),
            )
            .child(Scrollbar::vertical(&self.scroll));

        h_flex()
            .flex_1()
            .min_h_0()
            .w_full()
            .when(self.rail_open, |row| row.child(self.rail(colors, cx)))
            .child(pages)
            .into_any_element()
    }

    /// The thumbnail rail.
    fn rail(&mut self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let range = self.thumb_layout_visible();
        let margin = range.start.saturating_sub(2)..(range.end + 2).min(self.thumb_layout.pages());
        let mut column = div()
            .relative()
            .w(gpui_kit::px(RAIL_WIDTH))
            .h(gpui_kit::px(self.thumb_layout.total_height()));
        for page in margin {
            self.thumbs.touch(page);
            let [w, h] = self.thumb_layout.size(page);
            let top = self.thumb_layout.top(page);
            let image = self.thumbs.get(page).map(|tex| tex.image.clone());
            let here = page == self.current;
            let blank = if self.invert() {
                rgb(DARK_PAGE)
            } else {
                rgb(0xffffff)
            };
            column = column
                .child(
                    div()
                        .id(("pdf-thumb", page))
                        .debug_selector(move || format!("pdf-thumb-{page}"))
                        .absolute()
                        .left(gpui_kit::px((RAIL_WIDTH - w) / 2.))
                        .top(gpui_kit::px(top))
                        .w(gpui_kit::px(w))
                        .h(gpui_kit::px(h))
                        .bg(blank)
                        .border_2()
                        .border_color(if here { colors.accent } else { colors.divider })
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.go_to_page(page, window, cx)
                        }))
                        .when_some(image, |frame, image| {
                            frame.child(
                                img(ImageSource::Render(image))
                                    .object_fit(ObjectFit::Fill)
                                    .size_full(),
                            )
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .w(gpui_kit::px(RAIL_WIDTH))
                        .top(gpui_kit::px(top + h + 4.))
                        .flex()
                        .justify_center()
                        .text_size(px(10.))
                        .text_color(if here {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child((page + 1).to_string()),
                );
        }
        div()
            .id("pdf-rail-box")
            .debug_selector(|| "pdf-rail".into())
            .flex_none()
            .w(gpui_kit::px(RAIL_WIDTH))
            .h_full()
            .relative()
            .border_r_1()
            .border_color(colors.divider)
            .bg(colors.sidebar)
            .child(
                div()
                    .id("pdf-rail-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.rail_scroll)
                    .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
                    .child(column),
            )
    }
}

impl Focusable for PdfReader {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for PdfReader {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // Textures let go of since the last frame are freed on the GPU.
        for image in self.trash.drain(..) {
            window.drop_image(image).ok();
        }
        self.dark = cx.theme().is_dark();
        self.display_scale = window.scale_factor();

        // Plan again when the window, the scroll, the zoom or the theme moved.
        let (width, height) = self.viewport();
        let view = View {
            width: width.round() as i32,
            height: height.round() as i32,
            top: self.scroll_top().round() as i32,
            scale_percent: (self.display_scale * 100.).round() as i32,
            invert: self.invert(),
        };
        if matches!(self.state, ReaderState::Showing { .. }) {
            if view != self.seen {
                let resized = view.width != self.seen.width;
                self.seen = view;
                if resized {
                    self.relayout();
                }
                if !self.zooming {
                    cx.spawn_in(window, async move |this, cx| {
                        this.update_in(cx, |this, window, cx| {
                            this.sync(window, cx);
                            cx.notify();
                        })
                        .ok();
                    })
                    .detach();
                }
            }
            // The first frame has no measure of the window yet; another comes.
            if self.unmeasured && self.scroll.bounds().size.height > px(1.) {
                self.unmeasured = false;
            } else if self.unmeasured {
                window.request_animation_frame();
            }
        }

        let body = self.body(colors, cx);
        v_flex()
            .id("pdf-reader")
            .key_context("PdfReader")
            .debug_selector(|| "pdf-reader".into())
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.on_key(event, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .size_full()
            .bg(colors.editor)
            .child(self.header(colors, cx))
            .child(body)
    }
}

impl PdfReader {
    /// Keyboard: Escape closes; Page Up and Down scroll a screen; Home and
    /// End go to the ends; ⌘+ and ⌘− zoom; ⌘0 is actual size; ⌥⌘← and ⌥⌘→
    /// move a page.
    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let keys = &event.keystroke;
        let command = keys.modifiers.secondary();
        let showing = matches!(self.state, ReaderState::Showing { .. });
        match keys.key.as_str() {
            "escape" => self.close(cx),
            "pagedown" if showing => {
                let (_, height) = self.viewport();
                self.scroll_by(height * 0.9, window, cx);
            }
            "pageup" if showing => {
                let (_, height) = self.viewport();
                self.scroll_by(-height * 0.9, window, cx);
            }
            "home" if showing => self.go_to_page(0, window, cx),
            "end" if showing => self.go_to_page(self.layout.pages().saturating_sub(1), window, cx),
            "=" | "+" if showing && command => self.zoom_by(1, window, cx),
            "-" if showing && command => self.zoom_by(-1, window, cx),
            "0" if showing && command => self.set_zoom(Zoom::ACTUAL, window, cx),
            "right" if showing && command && keys.modifiers.alt => {
                self.go_to_page(self.current + 1, window, cx)
            }
            "left" if showing && command && keys.modifiers.alt => {
                self.go_to_page(self.current.saturating_sub(1), window, cx)
            }
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
pub(crate) mod tests;
