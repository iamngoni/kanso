//! Headless rasterization — pure-CPU rendering of a [`SketchDoc`] to a PNG.
//!
//! Used for note-list previews and image export. No GPU required, so it runs on
//! any thread (and could run server-side). Interactive on-device rendering uses
//! the native `wgpu` path; this is the portable, deterministic fallback.

use tiny_skia::{
    Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke as SkStroke, Transform,
};

use crate::format::{Element, ShapeKind, SketchDoc};
use crate::geometry;

/// Render a sketch to PNG bytes at the given pixel size.
///
/// Returns `None` only if the pixmap can't be allocated (zero/oversized
/// dimensions) or PNG encoding fails.
pub fn render_preview(doc: &SketchDoc, width: u32, height: u32) -> Option<Vec<u8>> {
    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    for element in &doc.elements {
        match element {
            Element::Stroke(stroke) => {
                let smoothed = geometry::smooth(&stroke.points, 8);
                if smoothed.len() < 2 {
                    continue;
                }
                let mut pb = PathBuilder::new();
                pb.move_to(smoothed[0].x, smoothed[0].y);
                for p in &smoothed[1..] {
                    pb.line_to(p.x, p.y);
                }
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.anti_alias = true;
                    paint.set_color_rgba8(
                        stroke.color.r,
                        stroke.color.g,
                        stroke.color.b,
                        stroke.color.a,
                    );

                    let sk = SkStroke {
                        width: stroke.base_width.max(0.5),
                        line_cap: LineCap::Round,
                        line_join: LineJoin::Round,
                        ..SkStroke::default()
                    };
                    pixmap.stroke_path(&path, &paint, &sk, Transform::identity(), None);
                }
            }
            Element::Shape(shape) => {
                let mut pb = PathBuilder::new();
                match shape.shape {
                    ShapeKind::Rect => {
                        if let Some(rect) =
                            tiny_skia::Rect::from_xywh(shape.x, shape.y, shape.w, shape.h)
                        {
                            pb.push_rect(rect);
                        }
                    }
                    ShapeKind::Ellipse => {
                        if let Some(rect) =
                            tiny_skia::Rect::from_xywh(shape.x, shape.y, shape.w, shape.h)
                        {
                            pb.push_oval(rect);
                        }
                    }
                    ShapeKind::Line => {
                        pb.move_to(shape.x, shape.y);
                        pb.line_to(shape.x + shape.w, shape.y + shape.h);
                    }
                }
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.anti_alias = true;
                    paint.set_color_rgba8(
                        shape.color.r,
                        shape.color.g,
                        shape.color.b,
                        shape.color.a,
                    );
                    let sk = SkStroke {
                        width: shape.stroke_width.max(0.5),
                        ..SkStroke::default()
                    };
                    pixmap.stroke_path(&path, &paint, &sk, Transform::identity(), None);
                }
            }
            Element::Arrow(arrow) => {
                let mut pb = PathBuilder::new();
                pb.move_to(arrow.from.0, arrow.from.1);
                pb.line_to(arrow.to.0, arrow.to.1);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.anti_alias = true;
                    paint.set_color_rgba8(
                        arrow.color.r,
                        arrow.color.g,
                        arrow.color.b,
                        arrow.color.a,
                    );
                    let sk = SkStroke {
                        width: arrow.stroke_width.max(0.5),
                        line_cap: LineCap::Round,
                        ..SkStroke::default()
                    };
                    pixmap.stroke_path(&path, &paint, &sk, Transform::identity(), None);
                }
            }
            // Text labels are rendered by the native layer (font shaping lives
            // there); previews omit them for now.
            Element::Text(_) => {}
        }
    }

    pixmap.encode_png().ok()
}
