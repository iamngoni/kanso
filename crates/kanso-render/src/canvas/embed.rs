//! A canvas shown inside another note, where a `canvas` fence names it (see
//! [`kanso_core::canvas::embed`]). The preview, HTML, PDF, published pages,
//! **Copy as image** and the phone all draw an embed here, so they agree on
//! what it shows:
//!
//! 1. the group the fence names, if the canvas has it;
//! 2. else the bounds of all content, padded;
//! 3. else, for a canvas with nothing on it, a small card with its title.
//!
//! An embed draws the canvas document only. A note card on the canvas is a
//! title, never the note's own drawing, so a canvas that points at another
//! canvas, or at itself, cannot make an embed draw forever. A locked
//! canvas is never read: the embed is a placeholder.

use std::fmt::Write as _;
use std::sync::Arc;

use kanso_core::canvas::{self, Canvas, Embed};

use super::{Scene, drawn_area, wrap};
use crate::{Failure, Job, Kind, Outcome, Palette, Svg, escape, render_blocking, svg};

pub use super::embed_model::{EmbedNote, Notes, Notice, Shown, Spec};

/// A whole-canvas embed over this many points on a side is large.
pub const HUGE_SIDE: f64 = 8000.0;

/// A whole-canvas embed whose SVG is over this many bytes is large.
pub const HUGE_BYTES: usize = 2 << 20;

/// The size of the card an empty canvas draws, in points.
pub const EMPTY_SIZE: (f64, f64) = (320.0, 200.0);

impl Job {
    /// The canvas in `body` (a canvas note's fenced JSON), as `spec` says.
    pub fn canvas(body: impl Into<Arc<str>>, spec: Spec, palette: Palette) -> Job {
        let mut job = Job::new(Kind::Canvas, body, palette);
        job.canvas = Some(Arc::new(spec));
        job
    }
}

/// Draws a [`Kind::Canvas`] job.
pub(crate) fn render(job: &Job) -> Outcome {
    let Some(spec) = job.canvas.as_deref() else {
        return Err(Failure::Invalid("The embed names no canvas.".into()));
    };
    let canvas = canvas::parse(&job.source).map_err(Failure::Invalid)?;
    draw(&canvas, spec, &job.palette)
}

/// `canvas` as `spec` says: the group or the content, at the width asked
/// for, with the notices the reader needs.
pub fn draw(canvas: &Canvas, spec: &Spec, palette: &Palette) -> Outcome {
    let group = spec
        .group
        .as_deref()
        .filter(|g| !g.is_empty())
        .map(|g| (g, canvas.group_bounds(g)));
    let area = group.and_then(|(_, bounds)| bounds);
    let mut notices = Vec::new();
    if matches!(group, Some((_, None))) {
        notices.push(Notice::GroupMissing);
    }
    if area.is_none() && drawn_area(canvas, None).is_none() {
        return Ok(empty_card(&spec.title, spec.width, palette, notices));
    }
    let scene = Scene {
        area,
        width: spec.width.map(f64::from),
        ..Scene::default()
    };
    let mut drawn = super::render(canvas, &scene, palette)?;
    if area.is_none() {
        let big = drawn_area(canvas, None)
            .is_some_and(|a| a.width > HUGE_SIDE || a.height > HUGE_SIDE)
            || drawn.markup.len() > HUGE_BYTES;
        if big {
            notices.push(Notice::WholeCanvas);
        }
    }
    drawn.notices = notices;
    Ok(drawn)
}

/// The card an empty canvas shows: its title, and that it is empty.
fn empty_card(title: &str, width: Option<u32>, palette: &Palette, notices: Vec<Notice>) -> Svg {
    let (w, h) = EMPTY_SIZE;
    let scale = width.map_or(1.0, |width| f64::from(width) / w);
    let name = wrap(title.trim(), 28, 1).into_iter().next();
    let mut body = format!(
        r#"<rect x="0.75" y="0.75" width="{}" height="{}" rx="8" fill="{}" stroke="{}" stroke-width="1.5"/>"#,
        svg::number((w - 1.5) as f32),
        svg::number((h - 1.5) as f32),
        palette.surface.hex(),
        palette.border.hex()
    );
    if let Some(name) = name.filter(|n| !n.is_empty()) {
        let _ = write!(
            body,
            r#"<text x="{}" y="94" font-size="16" font-weight="600" text-anchor="middle" fill="{}">{}</text>"#,
            svg::number((w / 2.0) as f32),
            palette.text.hex(),
            escape(&name)
        );
    }
    let _ = write!(
        body,
        r#"<text x="{}" y="120" font-size="13" text-anchor="middle" fill="{}">Empty canvas</text>"#,
        svg::number((w / 2.0) as f32),
        palette.muted.hex()
    );
    let (out_w, out_h) = ((w * scale) as f32, (h * scale) as f32);
    Svg {
        markup: format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">{body}</svg>"#,
            svg::number(w as f32),
            svg::number(h as f32),
            svg::number(out_w),
            svg::number(out_h)
        ),
        width: out_w,
        height: out_h,
        baseline: None,
        notices,
    }
}

/// What `embed` shows, finding its note with `notes`. A locked note's body
/// is never asked for, and so never drawn or cached. Blocks while the
/// drawing is made (or taken from the cache).
pub fn show(embed: &Embed, notes: Notes<'_>, palette: &Palette) -> Shown {
    match notes(&embed.note) {
        None => Shown::Missing,
        Some(EmbedNote::Locked) => Shown::Locked,
        Some(EmbedNote::NotCanvas) => Shown::NotCanvas,
        Some(EmbedNote::Open { title, body }) => {
            let spec = Spec {
                title,
                group: embed.group.clone(),
                width: embed.width,
            };
            match render_blocking(&Job::canvas(body, spec, *palette)).as_ref() {
                Ok(svg) => Shown::Drawing(svg.clone()),
                Err(Failure::Invalid(message)) => Shown::Invalid(message.clone()),
                Err(other) => Shown::Invalid(other.badge()),
            }
        }
    }
}

/// The note the reader is told about a failed or placeholder embed, in
/// words: what the export and the phone's page print.
pub fn placeholder_text(shown: &Shown) -> Option<&'static str> {
    match shown {
        Shown::Drawing(_) => None,
        Shown::Locked => Some("Locked canvas"),
        Shown::Missing => Some("Canvas not found"),
        Shown::NotCanvas => Some("This note isn't a canvas"),
        Shown::Invalid(_) => Some("Couldn't draw this canvas"),
    }
}

#[cfg(test)]
mod tests;
