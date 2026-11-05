//! Highlights in the reader (issue #43, step 6): select text on a page, mark
//! it in one of four colours, add a comment, and have the quote written into
//! the note.
//!
//! The reader keeps the PDF's highlights as a [`kanso_pdf::Annotations`], read
//! from the note's body. It never writes the body itself. A change is handed
//! to the editor as a [`Write`], and the editor applies it as an ordinary
//! edit of the note's text, the path typing takes, so it is one undo step, it
//! reaches a shared note's other editors, and it is saved with the note.
//!
//! * The highlights are saved after a short pause (a burst of highlights is
//!   one write, so one revision).
//! * The quote is written at once, as its own undo step.
//! * The note changing elsewhere is followed; a block that went away leaves
//!   the highlights on screen, read-only, with **Restore annotations**.

use std::collections::HashSet;
use std::time::Duration;

use gpui_kit::component::input::InputEvent;
use kanso_pdf::highlights::{Annotations, Row, Status};
use kanso_pdf::model::Color;
use kanso_pdf::{CharPos, PageText, Selection, TextDoc};

use super::layout::{Layout, MARGIN};
use super::*;

/// How long after the last change the highlights are written to the note.
pub(crate) const SAVE_DELAY: Duration = Duration::from_millis(500);
/// How long a highlight the reader jumped to stays marked.
const FLASH: Duration = Duration::from_millis(1600);
/// The width of the highlights panel, in logical pixels.
const PANEL_WIDTH: f32 = 280.;
/// The toolbar's height, and the gap above the text it belongs to.
const TOOLBAR_HEIGHT: f32 = 36.;
const TOOLBAR_GAP: f32 = 8.;

/// What the reader asks the editor to do with the note's text.
#[derive(Clone, Debug)]
pub enum Write {
    /// Save the highlights into the note's block.
    Save(Box<Annotations>),
    /// Put a highlight's quote under `## Highlights` (once).
    Quote {
        annotations: Box<Annotations>,
        id: String,
        /// The PDF's name, for the quote's `###` heading.
        name: String,
    },
    /// Write the last highlights back after the block left the note.
    Restore(Box<Annotations>),
}

/// The highlights part of the reader's state.
#[derive(Default)]
pub(super) struct Notes {
    /// The note's text as the editor last said.
    pub body: String,
    pub annots: Option<Annotations>,
    /// The attachment's md5 now.
    pub md5: String,
    /// The characters of the pages seen so far.
    pub texts: TextDoc,
    /// Pages (0-based) whose text was asked of the worker.
    pub asked: HashSet<u32>,
    pub selection: Option<Selection>,
    anchor: Option<CharPos>,
    /// Where the button went down (page, x, y), for a click on a highlight.
    down: Option<(u32, f64, f64)>,
    moved: bool,
    /// The highlight clicked, whose toolbar shows.
    pub focus: Option<String>,
    pub panel: bool,
    /// The highlight whose comment is being written.
    pub editing: Option<String>,
    /// The highlight marked after a jump.
    pub flash: Option<String>,
    /// Something to say about the last change, in words.
    pub notice: Option<String>,
    /// Changes not yet handed to the editor.
    pub dirty: bool,
    /// A highlight to reveal once the pages are laid out.
    pub reveal: Option<String>,
}

impl Notes {
    fn annots(&self) -> &Annotations {
        self.annots
            .as_ref()
            .expect("set_note ran before the reader was shown")
    }
}

/// Which page a point of the page column is on, and where in it
/// (normalised, top-left origin). With `clamp`, a point off every page is
/// taken to the nearest one and into it; without, it is `None`.
pub(super) fn page_point(
    layout: &Layout,
    content_w: f32,
    x: f32,
    y: f32,
    clamp: bool,
) -> Option<(usize, f64, f64)> {
    let pages = layout.pages();
    if pages == 0 {
        return None;
    }
    let mut best = (0usize, f32::MAX);
    for page in 0..pages {
        let (top, [w, h]) = (layout.top(page), layout.size(page));
        let left = (content_w - w) / 2.;
        let dx = (left - x).max(x - (left + w)).max(0.);
        let dy = (top - y).max(y - (top + h)).max(0.);
        let d = dx.max(dy);
        if d < best.1 {
            best = (page, d);
        }
    }
    let (page, d) = best;
    if d > 0. && !clamp {
        return None;
    }
    let (top, [w, h]) = (layout.top(page), layout.size(page));
    let left = (content_w - w) / 2.;
    let nx = (f64::from(x - left) / f64::from(w.max(1e-3))).clamp(0., 1.);
    let ny = (f64::from(y - top) / f64::from(h.max(1e-3))).clamp(0., 1.);
    Some((page, nx, ny))
}

/// A highlight colour as drawn over a page.
pub(super) fn tint(color: Color, strong: bool) -> Hsla {
    let rgb = match color {
        Color::Yellow => 0xffd60a,
        Color::Green => 0x34c759,
        Color::Blue => 0x3b9cff,
        Color::Pink => 0xff5c9a,
    };
    Hsla::from(rgba((rgb << 8) | 0xff)).opacity(if strong { 0.62 } else { 0.38 })
}

fn color_name(color: Color) -> &'static str {
    match color {
        Color::Yellow => "yellow",
        Color::Green => "green",
        Color::Blue => "blue",
        Color::Pink => "pink",
    }
}

pub(super) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

impl PdfReader {
    // ---- the note ----------------------------------------------------------------

    /// Gives the reader the note it belongs to: its text now, and a
    /// highlight to show first when a link named one.
    pub fn set_note(&mut self, body: String, reveal: Option<String>, cx: &mut Context<Self>) {
        let md5 = self
            .store
            .file_meta(&self.file_id)
            .ok()
            .flatten()
            .and_then(|meta| meta.md5digest)
            .unwrap_or_default();
        self.notes.annots = Some(Annotations::from_body(&body, &self.file_id, &md5));
        self.notes.md5 = md5;
        self.notes.body = body;
        self.notes.reveal = reveal;
        self.pencil.forget();
        self.ink_rev = None;
        self.load_ink_file();
        cx.notify();
    }

    /// The note's text changed (typing, another editor, a restored version).
    pub fn note_changed(&mut self, body: String, cx: &mut Context<Self>) {
        self.notes.body = body;
        // Changes of mine not yet written stay on screen; the save that
        // follows unites them with whatever the block says by then, so
        // highlights another editor added meanwhile are kept.
        if self.notes.dirty {
            cx.notify();
            return;
        }
        let Some(annots) = self.notes.annots.as_mut() else {
            return;
        };
        if annots.sync(&self.notes.body) {
            let n = &mut self.notes;
            let gone = |id: &Option<String>| {
                id.as_deref()
                    .is_some_and(|id| n.annots.as_ref().and_then(|a| a.get(id)).is_none())
            };
            if gone(&n.focus) {
                n.focus = None;
            }
            if gone(&n.editing) {
                n.editing = None;
            }
        }
        self.load_ink_file();
        self.pencil_follow_note();
        cx.notify();
    }

    /// Hands any change not yet written to the editor, now.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        if let Some(write) = self.take_pending() {
            cx.emit(PdfEvent::Write(write));
        }
    }

    /// The change not yet written, for a caller that writes it itself (the
    /// reader is about to go, and events to a gone reader are lost).
    pub fn take_pending(&mut self) -> Option<Write> {
        self.save_task = None;
        if !std::mem::take(&mut self.notes.dirty) {
            return None;
        }
        self.notes
            .annots
            .as_ref()
            .map(|a| Write::Save(Box::new(a.clone())))
    }

    /// Something in the highlights changed: write it after a pause.
    pub(super) fn changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes.dirty = true;
        self.save_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            this.update_in(cx, |this, _, cx| this.flush(cx)).ok();
        }));
        cx.notify();
    }

    pub(super) fn say(&mut self, words: impl Into<String>, cx: &mut Context<Self>) {
        self.notes.notice = Some(words.into());
        cx.notify();
    }

    // ---- text ----------------------------------------------------------------------

    /// The pages near the screen whose text is not asked for yet; marks them
    /// asked. Most wanted first.
    pub(super) fn text_wanted(&mut self) -> Vec<u32> {
        let visible = self.page_range();
        let pages = self.layout.pages();
        let around = visible.start.saturating_sub(1)..(visible.end + 1).min(pages);
        let mut out: Vec<u32> = visible
            .clone()
            .chain(around.filter(|p| !visible.contains(p)))
            .map(|p| p as u32)
            .collect();
        out.retain(|p| self.notes.asked.insert(*p));
        out
    }

    pub(super) fn on_text(&mut self, text: PageText, cx: &mut Context<Self>) {
        self.notes.texts.set_page(&text);
        cx.notify();
    }

    // ---- the pointer -----------------------------------------------------------------

    pub(super) fn content_width(&self) -> f32 {
        let (viewport_w, _) = self.viewport();
        (self.layout.max_width() + 2. * MARGIN).max(viewport_w)
    }

    /// Where a window position is on a page.
    pub(super) fn locate(
        &self,
        position: gpui_kit::Point<Pixels>,
        clamp: bool,
    ) -> Option<kanso_pdf::Point> {
        let bounds = self.scroll.bounds();
        let offset = self.scroll.offset();
        let x = f32::from(position.x - bounds.origin.x - offset.x);
        let y = f32::from(position.y - bounds.origin.y - offset.y);
        let (page, nx, ny) = page_point(&self.layout, self.content_width(), x, y, clamp)?;
        Some(kanso_pdf::Point {
            page: page as u32 + 1,
            x: nx,
            y: ny,
        })
    }

    pub(super) fn mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.pencil.on {
            self.ink_down(event, window, cx);
            return;
        }
        window.focus(&self.focus, cx);
        self.notes.notice = None;
        let Some(at) = self.locate(event.position, false) else {
            self.clear_selection(cx);
            return;
        };
        self.notes.down = Some((at.page, at.x, at.y));
        self.notes.moved = false;
        self.notes.focus = None;
        let hit = self.notes.texts.hit(at);
        match (event.click_count, hit) {
            (2, Some(pos)) => {
                self.notes.selection = self.notes.texts.word_at(pos);
                self.notes.anchor = None;
                self.notes.moved = true;
            }
            (n, Some(pos)) if n >= 3 => {
                self.notes.selection = self.notes.texts.line_at(pos);
                self.notes.anchor = None;
                self.notes.moved = true;
            }
            (_, Some(pos)) if event.modifiers.shift && self.notes.selection.is_some() => {
                self.notes.selection = self.notes.selection.map(|s| s.extend_to(pos));
                self.notes.anchor = None;
                self.notes.moved = true;
            }
            (_, pos) => {
                self.notes.selection = None;
                self.notes.anchor = pos;
            }
        }
        cx.notify();
    }

    pub(super) fn mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.pencil.on {
            self.ink_move(event, window, cx);
            return;
        }
        let Some(anchor) = self.notes.anchor else {
            return;
        };
        if event.pressed_button != Some(MouseButton::Left) {
            self.notes.anchor = None;
            return;
        }
        let Some(at) = self.locate(event.position, true) else {
            return;
        };
        let Some(pos) = self.notes.texts.hit(at) else {
            return;
        };
        let selection = Selection::between(anchor, pos);
        // A click that wobbles by a pixel is still a click: it takes a
        // different character to make a selection.
        if pos != anchor || self.notes.selection.is_some() {
            self.notes.moved = true;
            if self.notes.selection != Some(selection) {
                self.notes.selection = Some(selection);
                cx.notify();
            }
        }
    }

    pub(super) fn mouse_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pencil.on || self.pencil.drawing() {
            self.ink_up(window, cx);
            return;
        }
        self.notes.anchor = None;
        let down = self.notes.down.take();
        if self.notes.moved {
            return;
        }
        // A click: on a highlight it picks it, anywhere else it lets go.
        self.notes.selection = None;
        let Some((page, x, y)) = down else {
            return;
        };
        let id = self
            .notes
            .annots
            .as_ref()
            .and_then(|a| a.hit(page, x, y))
            .map(|h| h.id.clone());
        if let Some(id) = id {
            self.notes.panel = true;
            self.notes.focus = Some(id.clone());
            self.flash(&id, window, cx);
        }
        cx.notify();
    }

    pub(super) fn clear_selection(&mut self, cx: &mut Context<Self>) {
        let had = self.notes.selection.take().is_some()
            | self.notes.focus.take().is_some()
            | self.notes.editing.take().is_some();
        if had {
            cx.notify();
        }
    }

    /// Whether Escape has something of the highlights to put away first.
    pub(super) fn has_transient(&self) -> bool {
        self.notes.selection.is_some() || self.notes.focus.is_some() || self.notes.editing.is_some()
    }

    /// The selected text, copied.
    pub(super) fn copy_selection(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(selection) = self.notes.selection else {
            return false;
        };
        let text = self.notes.texts.text(&selection);
        if text.is_empty() {
            return false;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        true
    }

    /// ⌘A: every character of the pages seen.
    pub(super) fn select_all(&mut self, cx: &mut Context<Self>) {
        self.notes.selection = self.notes.texts.select_all();
        self.notes.focus = None;
        cx.notify();
    }

    // ---- making and changing highlights --------------------------------------------

    /// Highlights the selection. `quote` writes its quote into the note;
    /// `comment` opens the comment box. The new highlight's id.
    pub(super) fn highlight(
        &mut self,
        color: Color,
        quote: bool,
        comment: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let selection = self.notes.selection?;
        let parts = self.notes.texts.highlight_parts(&selection);
        let annots = self.notes.annots.as_mut()?;
        let id = match annots.add(parts, color, now_ms()) {
            Ok(id) => id,
            Err(error) => {
                self.say(error.to_string(), cx);
                return None;
            }
        };
        self.notes.selection = None;
        self.notes.focus = Some(id.clone());
        self.notes.panel = true;
        self.changed(window, cx);
        if quote {
            self.write_quote(&id, cx);
        }
        if comment {
            self.begin_comment(&id, window, cx);
        }
        Some(id)
    }

    /// The setting: whether highlighting also quotes into the note.
    fn quote_on_highlight(&self, cx: &App) -> bool {
        cx.try_global::<crate::workspace::AppState>()
            .is_none_or(|state| state.config.discovery.pdf_quote)
    }

    pub(super) fn pick_color(&mut self, color: Color, window: &mut Window, cx: &mut Context<Self>) {
        // A highlight clicked changes colour; a selection makes one.
        if let Some(id) = self.notes.focus.clone() {
            self.edit(window, cx, |a| a.set_color(&id, color));
            return;
        }
        let quote = self.quote_on_highlight(cx);
        self.highlight(color, quote, false, window, cx);
    }

    pub(super) fn add_note_to_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.notes.focus.clone() {
            self.begin_comment(&id, window, cx);
            return;
        }
        let quote = self.quote_on_highlight(cx);
        self.highlight(Color::Yellow, quote, true, window, cx);
    }

    /// "Add quote" on a selection: a highlight, and its quote whatever the
    /// setting says.
    pub(super) fn add_quote(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.notes.focus.clone() {
            self.write_quote(&id, cx);
            return;
        }
        self.highlight(Color::Yellow, true, false, window, cx);
    }

    /// Runs a change on the highlights; saves it, or says why it could not.
    fn edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        change: impl FnOnce(&mut Annotations) -> Result<(), kanso_pdf::PdfError>,
    ) -> bool {
        let Some(annots) = self.notes.annots.as_mut() else {
            return false;
        };
        match change(annots) {
            Ok(()) => {
                self.changed(window, cx);
                true
            }
            Err(error) => {
                self.say(error.to_string(), cx);
                false
            }
        }
    }

    pub(super) fn delete_highlight(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = id.to_owned();
        if self.edit(window, cx, |a| a.delete(&id).map(|_| ())) {
            if self.notes.focus.as_deref() == Some(&id) {
                self.notes.focus = None;
            }
            if self.notes.editing.as_deref() == Some(&id) {
                self.notes.editing = None;
            }
        }
    }

    /// Writes a highlight's quote into the note, once.
    pub(super) fn write_quote(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(annots) = &self.notes.annots else {
            return;
        };
        if !annots.is_editable() {
            return;
        }
        // The quote goes in now, its own undo step; the block that holds the
        // highlight its link names follows after the pause (and is written
        // at once when the reader or the note goes, see `take_pending`).
        let annotations = Box::new(annots.clone());
        cx.emit(PdfEvent::Write(Write::Quote {
            annotations,
            id: id.to_owned(),
            name: self.name.clone(),
        }));
    }

    pub(super) fn restore(&mut self, cx: &mut Context<Self>) {
        if let Some(annots) = &self.notes.annots {
            cx.emit(PdfEvent::Write(Write::Restore(Box::new(annots.clone()))));
        }
    }

    // ---- comments ----------------------------------------------------------------------

    pub(super) fn begin_comment(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(h) = self.notes.annots().get(id) else {
            return;
        };
        let text = h.comment.clone().unwrap_or_default();
        self.notes.editing = Some(id.to_owned());
        self.notes.panel = true;
        self.comment.update(cx, |field, cx| {
            field.set_value(text, window, cx);
        });
        let focus = self.comment.read(cx).focus_handle(cx);
        window.focus(&focus, cx);
        cx.notify();
    }

    pub(super) fn commit_comment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.notes.editing.take() else {
            return;
        };
        let text = self.comment.read(cx).value().to_string();
        self.edit(window, cx, |a| a.set_comment(&id, Some(&text)));
        window.focus(&self.focus, cx);
    }

    pub(super) fn cancel_comment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes.editing = None;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn on_comment_event(
        &mut self,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::PressEnter { .. }) {
            self.commit_comment(window, cx);
        }
    }

    // ---- jumping --------------------------------------------------------------------------

    /// Scrolls to a highlight and marks it for a moment.
    pub(super) fn reveal_highlight(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(h) = self.notes.annots().get(id) else {
            return;
        };
        let page = (h.page as usize).saturating_sub(1);
        if page >= self.layout.pages() {
            return;
        }
        let y0 = h
            .quads
            .get(&h.page)
            .and_then(|q| q.first())
            .map_or(0., |q| q.0[1]) as f32;
        let (_, height) = self.viewport();
        let y = self.layout.top(page) + y0 * self.layout.size(page)[1] - height / 3.;
        self.set_scroll_top(y);
        self.sync(window, cx);
        self.flash(id, window, cx);
    }

    fn flash(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.notes.flash = Some(id.to_owned());
        let id = id.to_owned();
        self.flash_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(FLASH).await;
            this.update_in(cx, |this, _, cx| {
                if this.notes.flash.as_deref() == Some(&id) {
                    this.notes.flash = None;
                    cx.notify();
                }
            })
            .ok();
        }));
        cx.notify();
    }

    /// After the pages are laid out: shows the highlight a link named.
    pub(super) fn reveal_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.notes.reveal.take() else {
            return;
        };
        if self
            .notes
            .annots
            .as_ref()
            .and_then(|a| a.get(&id))
            .is_some()
        {
            self.reveal_highlight(&id, window, cx);
        }
    }

    // ---- drawing ------------------------------------------------------------------------------

    /// The marks on one page: highlights, then the selection.
    pub(super) fn page_marks(
        &self,
        page: usize,
        w: f32,
        h: f32,
        colors: &Colors,
    ) -> Vec<AnyElement> {
        let number = page as u32 + 1;
        let mut out = Vec::new();
        let mark = |q: &kanso_pdf::model::Quad, fill: Hsla| {
            let [x0, y0, x1, y1] = q.0;
            div()
                .absolute()
                .left(gpui_kit::px(x0 as f32 * w))
                .top(gpui_kit::px(y0 as f32 * h))
                .w(gpui_kit::px(((x1 - x0) as f32 * w).max(1.)))
                .h(gpui_kit::px(((y1 - y0) as f32 * h).max(1.)))
                .bg(fill)
                .rounded(gpui_kit::px(2.))
        };
        if let Some(annots) = &self.notes.annots {
            for hl in &annots.block.highlights {
                let Some(quads) = hl.quads.get(&number) else {
                    continue;
                };
                let strong = self.notes.flash.as_deref() == Some(&hl.id)
                    || self.notes.focus.as_deref() == Some(&hl.id);
                for q in quads {
                    out.push(
                        mark(q, tint(hl.color, strong))
                            .debug_selector({
                                let id = hl.id.clone();
                                move || format!("pdf-mark-{id}")
                            })
                            .into_any_element(),
                    );
                }
            }
        }
        if let Some(selection) = &self.notes.selection
            && let Some(quads) = self.notes.texts.quads(selection).get(&number)
        {
            let fill = Hsla::from(colors.accent).opacity(0.32);
            for q in quads {
                out.push(mark(q, fill).into_any_element());
            }
        }
        out
    }

    /// The small toolbar by the selection (or the highlight clicked).
    pub(super) fn toolbar(
        &self,
        colors: &Colors,
        content_w: f32,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (quads, picked, in_note) = match &self.notes.selection {
            Some(selection) => (self.notes.texts.quads(selection), None, false),
            None => {
                let h = self
                    .notes
                    .focus
                    .as_deref()
                    .and_then(|id| self.notes.annots.as_ref()?.get(id))?;
                let present = kanso_pdf::highlights::quote_ids(&self.notes.body, &self.file_id);
                (h.quads.clone(), Some(h), present.contains(&h.id))
            }
        };
        let (page, first) = quads
            .iter()
            .next()
            .and_then(|(p, q)| Some((*p, q.first()?)))?;
        let index = (page as usize).checked_sub(1)?;
        if index >= self.layout.pages() {
            return None;
        }
        let [w, h] = self.layout.size(index);
        let left = (content_w - w) / 2. + first.0[0] as f32 * w;
        let above = self.layout.top(index) + first.0[1] as f32 * h - TOOLBAR_HEIGHT - TOOLBAR_GAP;
        let top = if above < self.layout.top(index) - MARGIN / 2. {
            // No room above the first line: below the last.
            let (lp, lq) = quads
                .iter()
                .next_back()
                .and_then(|(p, q)| Some((*p, q.last()?)))?;
            let li = (lp as usize).saturating_sub(1).min(self.layout.pages() - 1);
            self.layout.top(li) + lq.0[3] as f32 * self.layout.size(li)[1] + TOOLBAR_GAP
        } else {
            above
        };
        let current = picked.map(|h| h.color);
        let mut bar = h_flex()
            .id("pdf-toolbar")
            .debug_selector(|| "pdf-toolbar".into())
            .absolute()
            .left(gpui_kit::px(left.max(4.)))
            .top(gpui_kit::px(top))
            .h(gpui_kit::px(TOOLBAR_HEIGHT))
            .px(px(6.))
            .gap(px(4.))
            .items_center()
            .rounded(px(8.))
            .bg(colors.elevated)
            .border_1()
            .border_color(colors.divider)
            .shadow_md()
            // The bar is not part of the page: pressing in it must not start
            // a new selection or let go of this one.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        for color in Color::ALL {
            let name = color_name(color);
            bar = bar.child(
                div()
                    .id(("pdf-color", color as usize))
                    .debug_selector(move || format!("pdf-color-{name}"))
                    .size(px(18.))
                    .rounded_full()
                    .cursor_pointer()
                    .bg(tint(color, true))
                    .border_2()
                    .border_color(if current == Some(color) {
                        colors.text
                    } else {
                        colors.divider
                    })
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.pick_color(color, window, cx)),
                    ),
            );
        }
        bar = bar
            .child(div().w(px(1.)).h(px(16.)).bg(colors.divider))
            .child(
                Button::new("pdf-add-note")
                    .debug_selector(|| "pdf-add-note".into())
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::MessageSquareText).size(px(13.)))
                    .label(if picked.is_some_and(|h| h.comment.is_some()) {
                        "Edit note"
                    } else {
                        "Add note"
                    })
                    .on_click(
                        cx.listener(|this, _, window, cx| this.add_note_to_selection(window, cx)),
                    ),
            );
        if picked.is_none() {
            bar = bar.child(
                Button::new("pdf-copy")
                    .debug_selector(|| "pdf-copy".into())
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::Copy).size(px(13.)))
                    .label("Copy")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.copy_selection(cx);
                    })),
            );
        }
        if picked.is_none() || !in_note {
            bar = bar.child(
                Button::new("pdf-add-quote")
                    .debug_selector(|| "pdf-add-quote".into())
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::TextQuote).size(px(13.)))
                    .label("Add quote")
                    .on_click(cx.listener(|this, _, window, cx| this.add_quote(window, cx))),
            );
        }
        Some(bar.into_any_element())
    }

    /// The highlights panel.
    pub(super) fn panel(&mut self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let annots = self.notes.annots().clone();
        let rows = annots.rows(&self.notes.body);
        let changed = annots.block.pdf_changed(&self.notes.md5);
        let ink_rows = annots.ink_rows();
        let mut list = v_flex().gap(px(2.)).p(px(8.));
        if rows.is_empty() && ink_rows.is_empty() {
            list = list.child(
                div()
                    .debug_selector(|| "pdf-panel-empty".into())
                    .p(px(8.))
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child("Select text on a page to highlight it. Its quote goes into the note."),
            );
        }
        for (index, row) in rows.iter().enumerate() {
            list = list.child(self.panel_row(index, row, &annots, colors, cx));
        }
        if !ink_rows.is_empty() {
            list = list.child(
                div()
                    .px(px(8.))
                    .pt(px(10.))
                    .pb(px(2.))
                    .text_size(px(11.))
                    .text_color(colors.text_secondary)
                    .child("Pencil"),
            );
            for row in &ink_rows {
                list = list.child(self.ink_row(row, annots.is_editable(), colors, cx));
            }
        }
        let banner = |id: &'static str, words: String, action: Option<AnyElement>| {
            h_flex()
                .id(id)
                .debug_selector(move || id.into())
                .mx(px(8.))
                .mt(px(8.))
                .p(px(8.))
                .gap(px(8.))
                .items_center()
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(12.))
                .text_color(colors.text_secondary)
                .child(div().flex_1().min_w_0().child(words))
                .children(action)
        };
        v_flex()
            .id("pdf-panel")
            .debug_selector(|| "pdf-panel".into())
            .flex_none()
            .w(gpui_kit::px(PANEL_WIDTH))
            .h_full()
            .border_l_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .child(
                h_flex()
                    .flex_none()
                    .h(px(36.))
                    .px(px(12.))
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(colors.divider)
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text)
                            .child("Highlights"),
                    )
                    .child(
                        div()
                            .debug_selector(|| "pdf-panel-count".into())
                            .text_size(px(12.))
                            .text_color(colors.text_secondary)
                            .child(rows.len().to_string()),
                    ),
            )
            .when(changed, |panel| {
                panel.child(banner(
                    "pdf-changed",
                    "This PDF changed; highlights may be misplaced.".into(),
                    None,
                ))
            })
            .when(annots.status == Status::Restorable, |panel| {
                panel.child(banner(
                    "pdf-restore-banner",
                    "The highlights left this note. Showing the last ones, read-only.".into(),
                    Some(
                        Button::new("pdf-restore")
                            .debug_selector(|| "pdf-restore".into())
                            .small()
                            .label("Restore annotations")
                            .on_click(cx.listener(|this, _, _, cx| this.restore(cx)))
                            .into_any_element(),
                    ),
                ))
            })
            .when_some(
                match &annots.status {
                    Status::Unreadable(why) => Some(why.clone()),
                    _ => None,
                },
                |panel, why| panel.child(banner("pdf-unreadable", why, None)),
            )
            .when_some(self.notes.notice.clone(), |panel, words| {
                panel.child(banner("pdf-notice", words, None))
            })
            .child(
                div()
                    .id("pdf-panel-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(list),
            )
            .into_any_element()
    }

    fn panel_row(
        &self,
        index: usize,
        row: &Row,
        annots: &Annotations,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = row.id.clone();
        let editing = self.notes.editing.as_deref() == Some(&row.id);
        let selected = self.notes.focus.as_deref() == Some(&row.id);
        let writable = annots.is_editable();
        let mut card = v_flex()
            .id(("pdf-row", index))
            .debug_selector({
                let id = id.clone();
                move || format!("pdf-row-{id}")
            })
            .p(px(8.))
            .gap(px(4.))
            .rounded(px(6.))
            .cursor_pointer()
            .when(selected, |row| row.bg(colors.selected))
            .hover(|row| row.bg(colors.hover))
            .on_click(cx.listener({
                let id = id.clone();
                move |this, _, window, cx| {
                    this.notes.focus = Some(id.clone());
                    this.reveal_highlight(&id, window, cx);
                }
            }))
            .child(
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .text_size(px(11.))
                    .text_color(colors.text_secondary)
                    .child(div().size(px(10.)).rounded_full().bg(tint(row.color, true)))
                    .child(format!("p. {}", row.page))
                    .when(!row.in_note, |line| {
                        line.child(
                            div()
                                .debug_selector({
                                    let id = id.clone();
                                    move || format!("pdf-not-in-note-{id}")
                                })
                                .child("· not in note"),
                        )
                    }),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .line_clamp(4)
                    .child(row.text.clone()),
            );
        if editing {
            card = card.child(
                v_flex()
                    .gap(px(4.))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .debug_selector(|| "pdf-comment-field".into())
                            .child(Input::new(&self.comment).small()),
                    )
                    .child(
                        h_flex()
                            .gap(px(4.))
                            .child(
                                Button::new(("pdf-comment-save", index))
                                    .debug_selector(|| "pdf-comment-save".into())
                                    .small()
                                    .label("Save")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.commit_comment(window, cx)
                                    })),
                            )
                            .child(
                                Button::new(("pdf-comment-cancel", index))
                                    .ghost()
                                    .small()
                                    .label("Cancel")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.cancel_comment(window, cx)
                                    })),
                            ),
                    ),
            );
        } else if let Some(comment) = &row.comment {
            card = card.child(
                div()
                    .debug_selector({
                        let id = id.clone();
                        move || format!("pdf-comment-{id}")
                    })
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child(comment.clone()),
            );
        }
        if writable && !editing {
            card = card.child(
                h_flex()
                    .gap(px(2.))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Button::new(("pdf-row-note", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(Lucide::MessageSquareText).size(px(12.)))
                            .tooltip(if row.comment.is_some() {
                                "Edit note"
                            } else {
                                "Add note"
                            })
                            .on_click(cx.listener({
                                let id = id.clone();
                                move |this, _, window, cx| this.begin_comment(&id, window, cx)
                            })),
                    )
                    .when(!row.in_note, |actions| {
                        actions.child(
                            Button::new(("pdf-row-quote", index))
                                .debug_selector({
                                    let id = id.clone();
                                    move || format!("pdf-add-to-note-{id}")
                                })
                                .ghost()
                                .small()
                                .icon(Icon::new(Lucide::TextQuote).size(px(12.)))
                                .label("Add to note")
                                .on_click(cx.listener({
                                    let id = id.clone();
                                    move |this, _, _, cx| this.write_quote(&id, cx)
                                })),
                        )
                    })
                    .child(
                        Button::new(("pdf-row-delete", index))
                            .debug_selector({
                                let id = id.clone();
                                move || format!("pdf-delete-{id}")
                            })
                            .ghost()
                            .small()
                            .icon(Icon::new(Lucide::Trash).size(px(12.)))
                            .tooltip("Delete highlight")
                            .on_click(cx.listener({
                                let id = id.clone();
                                move |this, _, window, cx| this.delete_highlight(&id, window, cx)
                            })),
                    ),
            );
        }
        card.into_any_element()
    }

    /// Whether a highlight id belongs to this reader (for tests).
    #[cfg(test)]
    pub(crate) fn highlight_ids(&self) -> Vec<String> {
        self.notes
            .annots
            .as_ref()
            .map(|a| {
                a.block
                    .highlights
                    .iter()
                    .map(|h: &kanso_pdf::model::Highlight| h.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// What the tests (here and the editor's) do to a reader without a worker.
#[cfg(test)]
impl PdfReader {
    /// The document opened, with these page sizes.
    pub(crate) fn test_open(
        &mut self,
        sizes: Vec<[f32; 2]>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.on_event(Event::Opened { sizes }, window, cx);
    }

    /// The worker delivered a page's characters.
    pub(crate) fn test_text(&mut self, text: PageText, cx: &mut Context<Self>) {
        self.on_text(text, cx);
    }

    /// Selects everything seen and highlights it.
    pub(crate) fn test_highlight_all(
        &mut self,
        color: Color,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        self.select_all(cx);
        self.pick_color(color, window, cx);
        self.notes.focus.clone()
    }

    /// Whether the worker has delivered a page's characters.
    pub(crate) fn test_has_text(&self, page: u32) -> bool {
        self.notes
            .texts
            .page(page)
            .is_some_and(|p| !p.glyphs.is_empty())
    }

    /// Selects lines `lines` of page `page` (1-based), as a drag would.
    pub(crate) fn test_select_lines(
        &mut self,
        page: u32,
        lines: std::ops::Range<usize>,
        cx: &mut Context<Self>,
    ) {
        let layout = self.notes.texts.page(page).expect("the page has text");
        let first = layout.lines[lines.start].glyphs.start;
        let last = layout.lines[lines.end - 1].glyphs.end - 1;
        self.notes.selection = Some(Selection::between(
            CharPos { page, index: first },
            CharPos { page, index: last },
        ));
        self.notes.focus = None;
        cx.notify();
    }

    pub(crate) fn test_pick(&mut self, color: Color, window: &mut Window, cx: &mut Context<Self>) {
        self.pick_color(color, window, cx);
    }

    pub(crate) fn test_comment(
        &mut self,
        id: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.begin_comment(id, window, cx);
        self.comment
            .update(cx, |field, cx| field.set_value(text.to_owned(), window, cx));
        self.commit_comment(window, cx);
    }

    pub(crate) fn test_hide_rail(&mut self) {
        self.rail_open = false;
    }

    pub(crate) fn test_show_panel(&mut self) {
        self.notes.panel = true;
    }

    pub(crate) fn test_notes(&self) -> &Annotations {
        self.notes.annots()
    }
}

#[cfg(test)]
pub(super) mod tests;
