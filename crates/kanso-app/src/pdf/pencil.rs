//! Pencil ink on the pages (issue #43, step 10).
//!
//! The pencil is the canvas's: the same pens, the same pen input (a stylus's
//! pressure and tilt), the same two erasers, the same outline and the same
//! stroke type, so a mark looks the same here and on a canvas. A page being
//! drawn on is a one-layer canvas ([`CanvasState`]); the canvas does the
//! drawing and erasing, and what it made is handed to the reader's
//! [`Annotations`](kanso_pdf::Annotations) after every stroke, saved with the
//! highlights (one write after a short pause).
//!
//! Points are in page units (`kanso_pdf::ink`): a page is 1,000 across, so
//! ink stays where it was drawn at every zoom. Ink that outgrows the note
//! lives in an attachment ([`super::ink_file`]).

use std::collections::HashMap;
use std::sync::Arc;

use gpui_kit::component::Disableable as _;
use kanso_core::canvas::Refused;
use kanso_pdf::ink::{ERASER_PX, PAGE_UNITS, WIDTHS, to_units};
use kanso_render::canvas::pen;

use super::annotate::now_ms;
use super::*;
use crate::canvas_state::{COLORS, CanvasState, Outcome, PenSample, Source, Tool};

/// The tools the pencil offers, in the order of its bar.
pub(super) const TOOLS: [(Tool, &str, Lucide); 6] = [
    (Tool::Pen, "pdf-ink-pen", Lucide::PenTool),
    (Tool::Pencil, "pdf-ink-pencil", Lucide::Pencil),
    (Tool::Marker, "pdf-ink-marker", Lucide::Brush),
    (Tool::Highlighter, "pdf-ink-highlighter", Lucide::Droplet),
    (Tool::Eraser, "pdf-ink-eraser", Lucide::Trash),
    (Tool::Partial, "pdf-ink-partial", Lucide::Split),
];

/// The pencil's state: the tool, and the page canvases being drawn on.
pub(super) struct Pencil {
    /// The pencil is picked up: the pointer draws instead of selecting.
    pub on: bool,
    pub tool: Tool,
    pub color: String,
    pub width: f64,
    pages: HashMap<u32, CanvasState>,
    /// The page (1-based) and the source of the stroke or erase under way.
    gesture: Option<(u32, Source)>,
    /// The pages edited, newest last: undo goes back through them.
    trail: Vec<u32>,
    redone: Vec<u32>,
    /// Where an eraser hovers: the page (1-based) and the point in units.
    pub hover: Option<(u32, f64, f64)>,
}

impl Default for Pencil {
    fn default() -> Self {
        Pencil {
            on: false,
            tool: Tool::Pen,
            color: COLORS[0].into(),
            width: WIDTHS[1],
            pages: HashMap::new(),
            gesture: None,
            trail: Vec::new(),
            redone: Vec::new(),
            hover: None,
        }
    }
}

impl Pencil {
    pub fn drawing(&self) -> bool {
        self.gesture.is_some()
    }

    pub fn can_undo(&self) -> bool {
        self.trail
            .last()
            .is_some_and(|p| self.pages.get(p).is_some_and(CanvasState::can_undo))
    }

    pub fn can_redo(&self) -> bool {
        self.redone
            .last()
            .is_some_and(|p| self.pages.get(p).is_some_and(CanvasState::can_redo))
    }

    fn erasing(&self) -> bool {
        matches!(self.tool, Tool::Eraser | Tool::Partial)
    }

    /// Forgets the drawing states (the PDF changed under them).
    pub fn forget(&mut self) {
        self.pages.clear();
        self.gesture = None;
        self.trail.clear();
        self.redone.clear();
    }
}

/// A stroke ready to fill.
struct Paint {
    polygon: Arc<Vec<[f64; 2]>>,
    color: Hsla,
    opacity: f32,
}

impl PdfReader {
    // ---- picking the pencil up ------------------------------------------------------

    pub(super) fn toggle_pencil(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pencil.on = !self.pencil.on;
        self.pencil.hover = None;
        if self.pencil.on {
            self.clear_selection(cx);
            self.notes.panel = true;
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn set_pencil_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        self.pencil.tool = tool;
        self.pencil.hover = None;
        cx.notify();
    }

    // ---- the canvases ---------------------------------------------------------------

    /// The canvas for a page (1-based), made from what the note holds.
    fn page_state(&mut self, page: u32) -> Option<&mut CanvasState> {
        if !self.pencil.pages.contains_key(&page) {
            let ink = self.notes.annots.as_ref()?.page_ink(page);
            self.pencil
                .pages
                .insert(page, CanvasState::new(ink.to_canvas()));
        }
        self.pencil.pages.get_mut(&page)
    }

    /// After the note was read again (another editor, another device, an
    /// attachment that arrived): a page whose ink is not what its canvas has
    /// is drawn again from the note. A page mid-stroke is left to finish.
    pub(super) fn pencil_follow_note(&mut self) {
        let Some(annots) = &self.notes.annots else {
            return;
        };
        let gesture = self.pencil.gesture.map(|(p, _)| p);
        let stale: Vec<u32> = self
            .pencil
            .pages
            .iter()
            .filter(|(page, state)| {
                Some(**page) != gesture && {
                    let mine = annots.page_ink(**page);
                    mine.from_canvas(&state.canvas) != mine
                }
            })
            .map(|(page, _)| *page)
            .collect();
        for page in stale {
            self.pencil.pages.remove(&page);
            self.pencil.trail.retain(|p| *p != page);
            self.pencil.redone.retain(|p| *p != page);
        }
    }

    /// Hands what a page's canvas holds to the reader's annotations and
    /// schedules the save. A change the annotations refuse (more ink than a
    /// PDF can hold) is taken back, and said.
    fn commit_page(&mut self, page: u32, window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.pencil.pages.get(&page) else {
            return;
        };
        let Some(annots) = self.notes.annots.as_mut() else {
            return;
        };
        let next = annots.page_ink(page).from_canvas(&state.canvas);
        match annots.set_page_ink(next) {
            Ok(()) => self.changed(window, cx),
            Err(error) => {
                self.pencil.pages.remove(&page);
                self.pencil.trail.retain(|p| *p != page);
                self.say(error.to_string(), cx);
            }
        }
    }

    // ---- the pointer ------------------------------------------------------------------

    /// The point of a page (1-based) under a window position, in units.
    fn ink_point(&self, page: u32, position: gpui_kit::Point<Pixels>) -> (f64, f64) {
        let index = page as usize - 1;
        let bounds = self.scroll.bounds();
        let offset = self.scroll.offset();
        let x = f32::from(position.x - bounds.origin.x - offset.x);
        let y = f32::from(position.y - bounds.origin.y - offset.y);
        let [w, h] = self.layout.size(index);
        let left = (self.content_width() - w) / 2.;
        to_units(
            f64::from(x - left),
            f64::from(y - self.layout.top(index)),
            f64::from(w),
            f64::from(h),
        )
    }

    /// A pen sample for the canvas: the stylus's pressure and tilt when one
    /// is near the tablet, a mouse's plain stroke otherwise.
    fn ink_sample(
        &self,
        page: u32,
        position: gpui_kit::Point<Pixels>,
        source: Source,
    ) -> PenSample {
        let (x, y) = self.ink_point(page, position);
        let mut sample = PenSample::mouse(x, y);
        sample.source = source;
        if let Some(pen) = crate::pen_input::current() {
            sample.pressure = pen.pressure;
            sample.tilt = pen.tilt;
            if pen.eraser {
                sample.source = Source::PenEraser;
            } else if pen.in_proximity {
                sample.source = Source::Pen;
            }
        }
        sample
    }

    /// The eraser's radius in units on a page (1-based), from its size on
    /// the screen.
    fn eraser_units(&self, page: u32) -> f64 {
        let w = f64::from(self.layout.size(page as usize - 1)[0]).max(1.);
        ERASER_PX * PAGE_UNITS / w
    }

    fn prepare_tools(&mut self, page: u32) {
        let (tool, color, width) = (
            self.pencil.tool,
            self.pencil.color.clone(),
            self.pencil.width,
        );
        if let Some(state) = self.page_state(page) {
            state.tool = tool;
            state.color = color;
            state.width = width;
        }
    }

    pub(super) fn ink_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        self.notes.notice = None;
        let Some(at) = self.locate(event.position, false) else {
            return;
        };
        let Some((editable, pending)) = self
            .notes
            .annots
            .as_ref()
            .map(|a| (a.is_editable(), a.ink_pending))
        else {
            return;
        };
        if !editable {
            self.say(kanso_pdf::highlights::READ_ONLY, cx);
            return;
        }
        if pending {
            self.say(kanso_pdf::highlights::INK_ARRIVING, cx);
            return;
        }
        let page = at.page;
        self.prepare_tools(page);
        let sample = self.ink_sample(page, event.position, Source::Mouse);
        self.pencil.gesture = Some((page, sample.source));
        let radius = self.eraser_units(page);
        let now = now_ms();
        if let Some(state) = self.page_state(page) {
            let outcome = state.begin(sample, radius, now);
            self.after_ink(page, outcome, false, window, cx);
        }
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn ink_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((page, source)) = self.pencil.gesture {
            if event.pressed_button != Some(MouseButton::Left) {
                // The button went up where no mouse-up reached the reader.
                self.ink_up(window, cx);
                return;
            }
            let sample = self.ink_sample(page, event.position, source);
            let radius = self.eraser_units(page);
            let now = now_ms();
            self.pencil.hover = Some((page, sample.x, sample.y));
            if let Some(state) = self.page_state(page) {
                let outcome = state.extend(sample, radius, now);
                self.after_ink(page, outcome, false, window, cx);
            }
            cx.notify();
            return;
        }
        let hover = (self.pencil.erasing())
            .then(|| {
                let at = self.locate(event.position, false)?;
                let (x, y) = self.ink_point(at.page, event.position);
                Some((at.page, x, y))
            })
            .flatten();
        if hover != self.pencil.hover {
            self.pencil.hover = hover;
            cx.notify();
        }
    }

    pub(super) fn ink_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((page, _)) = self.pencil.gesture.take() else {
            return;
        };
        let now = now_ms();
        if let Some(state) = self.page_state(page) {
            let outcome = state.end(now);
            self.after_ink(page, outcome, true, window, cx);
        }
        cx.notify();
    }

    /// What a step of drawing or erasing did. A finished stroke, and every
    /// step of an erase, goes to the annotations.
    fn after_ink(
        &mut self,
        page: u32,
        outcome: Outcome,
        _finished: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match outcome {
            Outcome::Changed => {
                self.pencil.redone.clear();
                if self.pencil.trail.last() != Some(&page) {
                    self.pencil.trail.push(page);
                }
                self.commit_page(page, window, cx);
            }
            Outcome::Refused(why) => {
                let words = match why {
                    Refused::Full => kanso_pdf::ink::FULL,
                    Refused::TooLong => "That stroke is too long. Lift the pencil sooner.",
                    Refused::Empty => return,
                };
                self.say(words, cx);
            }
            Outcome::Nothing => {}
        }
    }

    // ---- undo ---------------------------------------------------------------------------

    pub(super) fn ink_undo(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(&page) = self.pencil.trail.last() else {
            return false;
        };
        let now = now_ms();
        let Some(state) = self.pencil.pages.get_mut(&page) else {
            self.pencil.trail.pop();
            return false;
        };
        let outcome = state.undo(now);
        if !state.can_undo() {
            self.pencil.trail.pop();
        }
        self.pencil.redone.push(page);
        if outcome == Outcome::Changed {
            self.commit_page(page, window, cx);
        }
        cx.notify();
        true
    }

    pub(super) fn ink_redo(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(page) = self.pencil.redone.pop() else {
            return false;
        };
        let now = now_ms();
        let Some(state) = self.pencil.pages.get_mut(&page) else {
            return false;
        };
        let outcome = state.redo(now);
        if self.pencil.trail.last() != Some(&page) {
            self.pencil.trail.push(page);
        }
        if outcome == Outcome::Changed {
            self.commit_page(page, window, cx);
        }
        cx.notify();
        true
    }

    /// Rubs out everything drawn on a page: the panel's "Clear".
    pub(super) fn clear_ink_page(
        &mut self,
        page: u32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(annots) = self.notes.annots.as_mut() else {
            return;
        };
        match annots.clear_page_ink(page, now_ms()) {
            Ok(true) => {
                self.pencil.pages.remove(&page);
                self.pencil.trail.retain(|p| *p != page);
                self.pencil.redone.retain(|p| *p != page);
                self.changed(window, cx);
            }
            Ok(false) => {}
            Err(error) => self.say(error.to_string(), cx),
        }
    }

    // ---- drawing --------------------------------------------------------------------------

    /// The ink of one page (0-based), painted over it: the strokes, the one
    /// being drawn, and an eraser's ring.
    pub(super) fn ink_layer(&mut self, page: usize, w: f32, colors: &Colors) -> Option<AnyElement> {
        let number = page as u32 + 1;
        let has_ink = self
            .notes
            .annots
            .as_ref()
            .is_some_and(|a| !a.page_ink(number).is_blank());
        let drawing_here = self.pencil.gesture.is_some_and(|(p, _)| p == number);
        if !has_ink && !drawing_here && !self.pencil.pages.contains_key(&number) {
            return None;
        }
        let dark = self.invert();
        let hover = self.pencil.hover.filter(|(p, _, _)| *p == number);
        let ring = self.eraser_units(number);
        let state = self.page_state(number)?;
        let seen = ([-1.0e6, -1.0e6], [1.0e6, 1.0e6]);
        let mut strokes: Vec<Paint> = state
            .visible_ink(seen)
            .into_iter()
            .map(|ink| Paint {
                polygon: ink.polygon,
                color: crate::canvas_view::ink_hsla(&ink.color, dark),
                opacity: pen::opacity(&ink.tool) as f32,
            })
            .collect();
        if let Some((stroke, polygon)) = state.live_outline(&[]) {
            strokes.push(Paint {
                polygon: Arc::new(polygon),
                color: crate::canvas_view::ink_hsla(&stroke.color, dark),
                opacity: pen::opacity(&stroke.tool) as f32,
            });
        }
        if strokes.is_empty() && hover.is_none() {
            return None;
        }
        let paint = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let scale = f64::from(f32::from(bounds.size.width)) / PAGE_UNITS;
                let origin = bounds.origin;
                let to_px = |x: f64, y: f64| {
                    point(
                        origin.x + gpui_kit::px((x * scale) as f32),
                        origin.y + gpui_kit::px((y * scale) as f32),
                    )
                };
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    for ink in &strokes {
                        if let Some(path) =
                            crate::canvas_view::fill_path(&ink.polygon, (0., 0.), &to_px)
                        {
                            window.paint_path(path, ink.color.opacity(ink.opacity));
                        }
                    }
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let eraser = hover.map(|(_, x, y)| {
            let scale = f64::from(w) / PAGE_UNITS;
            let r = (ring * scale) as f32;
            div()
                .absolute()
                .left(gpui_kit::px((x * scale) as f32 - r))
                .top(gpui_kit::px((y * scale) as f32 - r))
                .size(gpui_kit::px(r * 2.))
                .rounded_full()
                .border_1()
                .border_color(colors.text_secondary)
        });
        Some(
            div()
                .debug_selector(move || format!("pdf-ink-{page}"))
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .child(paint)
                .children(eraser)
                .into_any_element(),
        )
    }

    /// One page with pencil marks, in the panel: click to go there; Clear
    /// rubs the page's marks out.
    pub(super) fn ink_row(
        &self,
        row: &kanso_pdf::InkRow,
        writable: bool,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let page = row.page;
        h_flex()
            .id(("pdf-ink-row", page as usize))
            .debug_selector(move || format!("pdf-ink-row-{page}"))
            .p(px(8.))
            .gap(px(6.))
            .items_center()
            .rounded(px(6.))
            .cursor_pointer()
            .hover(|row| row.bg(colors.hover))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.go_to_page(page as usize - 1, window, cx)
            }))
            .child(
                div()
                    .flex_1()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(format!(
                        "p. {page} · {} {}",
                        row.strokes,
                        if row.strokes == 1 {
                            "stroke"
                        } else {
                            "strokes"
                        }
                    )),
            )
            .when(writable, |line| {
                line.child(
                    Button::new(("pdf-ink-clear", page as usize))
                        .debug_selector(move || format!("pdf-ink-clear-{page}"))
                        .ghost()
                        .small()
                        .label("Clear")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.clear_ink_page(page, window, cx)
                        })),
                )
            })
            .into_any_element()
    }

    // ---- the bar -------------------------------------------------------------------------------

    /// The pencil's bar, under the header while the pencil is up.
    pub(super) fn pencil_bar(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let tool_button = |(tool, id, icon): (Tool, &'static str, Lucide),
                           cx: &mut Context<Self>| {
            self.button(id, icon, tool.label(), self.pencil.tool == tool)
                .on_click(cx.listener(move |this, _, _, cx| this.set_pencil_tool(tool, cx)))
        };
        let swatch = |color: &'static str, cx: &mut Context<Self>| {
            let chosen = self.pencil.color == color;
            let shade = pen::parse_hex(color).map_or(0, |(r, g, b)| {
                (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
            });
            div()
                .id(SharedString::from(format!("pdf-ink-colour-{color}")))
                .debug_selector(move || format!("pdf-ink-colour-{color}"))
                .size(px(18.))
                .rounded_full()
                .bg(Hsla::from(rgb(shade)))
                .border_2()
                .border_color(if chosen {
                    colors.accent
                } else {
                    colors.divider
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.pencil.color = color.into();
                    cx.notify();
                }))
        };
        let width_dot = |w: f64, cx: &mut Context<Self>| {
            let chosen = (self.pencil.width - w).abs() < 1e-9;
            div()
                .id(SharedString::from(format!("pdf-ink-width-{w}")))
                .debug_selector(move || format!("pdf-ink-width-{w}"))
                .size(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(5.))
                .when(chosen, |el| el.bg(colors.selected))
                .cursor_pointer()
                .child(
                    div()
                        .size(px((w as f32 * 1.2 + 2.).min(14.)))
                        .rounded_full()
                        .bg(colors.text),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.pencil.width = w;
                    cx.notify();
                }))
        };
        let divider = || div().w(px(1.)).h(px(18.)).mx(px(4.)).bg(colors.divider);
        h_flex()
            .id("pdf-ink-bar")
            .debug_selector(|| "pdf-ink-bar".into())
            .flex_none()
            .h(px(38.))
            .px(px(12.))
            .gap(px(2.))
            .items_center()
            .border_b_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .children(
                TOOLS
                    .into_iter()
                    .map(|t| tool_button(t, cx).into_any_element()),
            )
            .child(divider())
            .children(COLORS.iter().map(|&c| swatch(c, cx).into_any_element()))
            .child(divider())
            .children(WIDTHS.iter().map(|&w| width_dot(w, cx).into_any_element()))
            .child(divider())
            .child(
                self.button("pdf-ink-undo", Lucide::Undo2, "Undo", false)
                    .disabled(!self.pencil.can_undo())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.ink_undo(window, cx);
                    })),
            )
            .child(
                self.button("pdf-ink-redo", Lucide::Redo2, "Redo", false)
                    .disabled(!self.pencil.can_redo())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.ink_redo(window, cx);
                    })),
            )
            .child(div().flex_1())
            .child(
                Button::new("pdf-ink-done")
                    .debug_selector(|| "pdf-ink-done".into())
                    .small()
                    .label("Done")
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_pencil(window, cx))),
            )
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
impl PdfReader {
    pub(crate) fn test_pencil(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_pencil(window, cx);
    }

    pub(crate) fn test_pencil_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        self.set_pencil_tool(tool, cx);
    }

    /// Draws a stroke on a page (1-based) from points given in units, the
    /// way the pointer does: a press, moves, a release.
    pub(crate) fn test_stroke(
        &mut self,
        page: u32,
        points: &[(f64, f64, f64)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.prepare_tools(page);
        let now = now_ms();
        let radius = self.eraser_units(page);
        let sample = |(x, y, p): (f64, f64, f64)| PenSample {
            x,
            y,
            pressure: p,
            tilt: None,
            source: Source::Pen,
        };
        self.pencil.gesture = Some((page, Source::Pen));
        if let Some(state) = self.page_state(page) {
            let outcome = state.begin(sample(points[0]), radius, now);
            self.after_ink(page, outcome, false, window, cx);
        }
        for p in &points[1..] {
            if let Some(state) = self.page_state(page) {
                let outcome = state.extend(sample(*p), radius, now);
                self.after_ink(page, outcome, false, window, cx);
            }
        }
        self.ink_up(window, cx);
    }

    pub(crate) fn test_undo(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.ink_undo(window, cx)
    }

    pub(crate) fn test_redo(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.ink_redo(window, cx)
    }

    pub(crate) fn test_ink(&self, page: u32) -> kanso_pdf::PageInk {
        self.notes
            .annots
            .as_ref()
            .map(|a| a.page_ink(page))
            .unwrap_or_else(|| kanso_pdf::PageInk::new(page))
    }

    pub(crate) fn test_pencil_on(&self) -> bool {
        self.pencil.on
    }
}
