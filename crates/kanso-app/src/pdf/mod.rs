//! The PDF reader shell (issue #43, step 3): the editor's right pane while a
//! PDF is open, showing one page the worker drew.
//!
//! The view never touches PDF bytes or PDFium. It reads the attachment from
//! the library on a background thread, hands it to `kanso-pdf-worker` (a
//! separate, contained process, see `kanso_pdf::client`), asks for a page and
//! turns the BGRA pixels that come back into a texture. A worker that hangs,
//! crashes or is killed shows "This PDF couldn't be opened." with Try Again;
//! the note and the app carry on. The continuous reader (scrolling, zoom,
//! thumbnails) is step 4.
//!
//! Page textures live in memory only, never on disk: a page image of a
//! locked PDF would be a plaintext leak.

mod model;
mod texture;

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_pdf::client::{RenderedPage, Worker, WorkerError, WorkerPaths};
use kanso_store::Store;

pub use model::ReaderState;
use model::{NOT_READABLE, UNAVAILABLE, render_width};

use crate::theme::Colors;
use crate::zoom::px;

/// The widest a page is shown, in points: the reader's page column.
const COLUMN_WIDTH: f32 = 880.;

/// What the reader tells the editor.
pub enum PdfEvent {
    /// Back to the note: the button, or Escape.
    Close,
}

/// The reader for one PDF attachment.
pub struct PdfReader {
    store: Arc<Store>,
    /// The attachment's document id (`file:…`).
    file_id: String,
    name: String,
    state: ReaderState,
    /// The page on show.
    image: Option<Arc<RenderImage>>,
    /// The worker and the page count, while there is one to ask again.
    worker: Option<(Worker, u32)>,
    /// Where to find the worker; `None` when this build has none.
    paths: Option<WorkerPaths>,
    /// The width asked of the worker, in pixels.
    width_px: u32,
    focus: FocusHandle,
    task: Option<Task<()>>,
}

impl EventEmitter<PdfEvent> for PdfReader {}

/// What one trip to the worker came back with.
struct Outcome {
    worker: Option<(Worker, u32)>,
    result: Result<(u32, RenderedPage), String>,
}

impl PdfReader {
    /// Opens `file_id` (its document id) and shows page `page` (1-based;
    /// clamped to the document). `scale` is the window's display scale.
    pub fn new(
        store: Arc<Store>,
        file_id: String,
        name: String,
        page: u32,
        scale: f32,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::with_paths(
            store,
            file_id,
            name,
            page,
            scale,
            kanso_pdf::client::locate(),
            cx,
        )
    }

    /// As [`PdfReader::new`], with the worker found by the caller (tests).
    pub fn with_paths(
        store: Arc<Store>,
        file_id: String,
        name: String,
        page: u32,
        scale: f32,
        paths: Option<WorkerPaths>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut reader = PdfReader {
            store,
            file_id,
            name,
            state: ReaderState::Loading,
            image: None,
            worker: None,
            paths,
            width_px: render_width(COLUMN_WIDTH, scale),
            focus: cx.focus_handle(),
            task: None,
        };
        reader.show_page(page.max(1) - 1, cx);
        reader
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    pub fn state(&self) -> &ReaderState {
        &self.state
    }

    /// The worker's process id, for the test that kills it.
    #[cfg(test)]
    pub(crate) fn worker_pid(&self) -> Option<u32> {
        self.worker.as_ref().and_then(|(worker, _)| worker.pid())
    }

    /// Draws page `index` (0-based): with the worker already open if there is
    /// one, else after starting one and opening the document.
    pub fn show_page(&mut self, index: u32, cx: &mut Context<Self>) {
        let prior = self.worker.take();
        if prior.is_none() {
            self.state = ReaderState::Loading;
            self.image = None;
        }
        let (store, id, paths, width) = (
            self.store.clone(),
            self.file_id.clone(),
            self.paths.clone(),
            self.width_px,
        );
        let work = cx
            .background_executor()
            .spawn(async move { run(prior, &store, &id, paths.as_ref(), index, width) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let outcome = work.await;
            this.update(cx, |this, cx| this.finish(index, outcome, cx))
                .ok();
        }));
        cx.notify();
    }

    fn finish(&mut self, index: u32, outcome: Outcome, cx: &mut Context<Self>) {
        self.worker = outcome.worker;
        match outcome.result {
            Ok((pages, page)) => {
                let (width, height) = (page.width, page.height);
                match texture::texture(page) {
                    Some(image) => {
                        self.image = Some(image);
                        self.state = ReaderState::Showing {
                            page: index.min(pages.saturating_sub(1)) + 1,
                            pages,
                            width,
                            height,
                        };
                    }
                    None => self.fail(NOT_READABLE.to_owned()),
                }
            }
            Err(words) => self.fail(words),
        }
        cx.notify();
    }

    fn fail(&mut self, words: String) {
        self.image = None;
        self.worker = None;
        self.state = ReaderState::Failed(words);
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        cx.emit(PdfEvent::Close);
    }

    fn header(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_none()
            .h(px(44.))
            .px(px(12.))
            .gap(px(10.))
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
                    .text_size(px(13.))
                    .text_color(colors.text)
                    .debug_selector(|| "pdf-name".into())
                    .child(self.name.clone()),
            )
            .when_some(self.state.page_label(), |el, label| {
                el.child(
                    div()
                        .flex_none()
                        .debug_selector(|| "pdf-page-label".into())
                        .text_size(px(12.))
                        .text_color(colors.text_secondary)
                        .child(label),
                )
            })
    }

    fn body(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        match (&self.state, &self.image) {
            (ReaderState::Showing { .. }, Some(image)) => div()
                .id("pdf-pages")
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_y_scroll()
                .child(
                    v_flex().w_full().items_center().p(px(24.)).child(
                        img(ImageSource::Render(image.clone()))
                            .debug_selector(|| "pdf-page".into())
                            .object_fit(ObjectFit::Contain)
                            .w_full()
                            .max_w(px(COLUMN_WIDTH))
                            .border_1()
                            .border_color(colors.divider)
                            .shadow_md(),
                    ),
                )
                .into_any_element(),
            (ReaderState::Failed(words), _) => v_flex()
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
                        .child(words.clone()),
                )
                .child(
                    Button::new("pdf-retry")
                        .debug_selector(|| "pdf-retry".into())
                        .small()
                        .label("Try Again")
                        .on_click(cx.listener(|this, _, _, cx| this.show_page(0, cx))),
                )
                .into_any_element(),
            _ => v_flex()
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
}

impl Focusable for PdfReader {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for PdfReader {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        v_flex()
            .id("pdf-reader")
            .key_context("PdfReader")
            .debug_selector(|| "pdf-reader".into())
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    this.close(cx);
                    cx.stop_propagation();
                }
            }))
            .size_full()
            .bg(colors.editor)
            .child(self.header(colors, cx))
            .child(self.body(colors, cx))
    }
}

/// One trip to the worker, on a background thread: opens the document if
/// there is no worker yet, then draws the page.
fn run(
    prior: Option<(Worker, u32)>,
    store: &Store,
    file_id: &str,
    paths: Option<&WorkerPaths>,
    index: u32,
    width_px: u32,
) -> Outcome {
    let failed = |words: String| Outcome {
        worker: None,
        result: Err(words),
    };
    let (mut worker, pages) = match prior {
        Some(ready) => ready,
        None => {
            let Some(paths) = paths else {
                return failed(UNAVAILABLE.to_owned());
            };
            // Only a PDF the library holds in the clear; a sealed one is not
            // returned at all.
            let bytes = match store.attachment(file_id) {
                Ok(Some((content_type, attachment))) if content_type == "application/pdf" => {
                    attachment.data
                }
                _ => return failed(NOT_READABLE.to_owned()),
            };
            let mut worker = match Worker::start(paths) {
                Ok(worker) => worker,
                Err(error) => return failed(error.message()),
            };
            match worker.open(&bytes, None) {
                Ok(opened) => (worker, opened.pages),
                Err(error) => return failed(reader_words(&error)),
            }
        }
    };
    let index = index.min(pages.saturating_sub(1));
    match worker.render(index, width_px) {
        Ok(page) => Outcome {
            worker: Some((worker, pages)),
            result: Ok((pages, page)),
        },
        Err(error) => failed(reader_words(&error)),
    }
}

/// What the reader says for a worker error. A password-protected PDF waits
/// for the password prompt (step 4), so for now it says why it can't open.
fn reader_words(error: &WorkerError) -> String {
    match error {
        WorkerError::Refused {
            kind: kanso_pdf::protocol::ErrorKind::PasswordRequired,
            ..
        } => "This PDF needs a password, and PDFs with passwords can't be opened yet.".to_owned(),
        other => other.message(),
    }
}

#[cfg(test)]
mod tests;
