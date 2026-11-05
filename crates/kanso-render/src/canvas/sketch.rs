//! An inline sketch drawn as one SVG: the fixed box a `sketch` fence holds,
//! on the surface colour with a quiet frame, and the ink in it. The preview
//! of a shared note, the HTML and PDF exports, a published page and the
//! phone's note preview draw a sketch from here.

use kanso_core::canvas::{Bounds, Canvas};

use crate::{Failure, Outcome, Palette, Svg, svg};

/// The corner radius of the frame, in points.
const RADIUS: f64 = 8.0;

/// The sketch in a fence's JSON, when it is one of Kanso's (it has its box
/// and its ID). `None` for any other `sketch` code block. Pure.
pub fn parse(source: &str) -> Option<Canvas> {
    let canvas: Canvas = serde_json::from_str(source).ok()?;
    canvas.kanso_sketch?;
    canvas.validate_sketch().ok()?;
    Some(canvas)
}

/// The sketch `source` (a fence's JSON) as an SVG: its whole box, never more,
/// whatever its ink does. Pure.
pub fn render(source: &str, palette: &Palette) -> Outcome {
    let Some(canvas) = parse(source) else {
        return Err(Failure::Invalid("This isn't a Kanso sketch.".into()));
    };
    let Some(sketch) = canvas.kanso_sketch else {
        return Err(Failure::Invalid("This isn't a Kanso sketch.".into()));
    };
    let (w, h) = (f64::from(sketch.w), f64::from(sketch.h));
    // The page colour is the box's own: the sketch sits on the surface.
    let on_surface = Palette {
        background: palette.surface,
        ..*palette
    };
    let scene = super::Scene {
        area: Some(Bounds {
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
        }),
        ..super::Scene::default()
    };
    let drawn = super::render(&canvas, &scene, &on_surface)?;
    // Ink that strays outside the box is clipped by the viewBox; the frame
    // goes on top, inside the box.
    let frame = format!(
        r#"<rect x="0.75" y="0.75" width="{}" height="{}" rx="{RADIUS}" fill="none" stroke="{}" stroke-width="1.5"/>"#,
        svg::number((w - 1.5) as f32),
        svg::number((h - 1.5) as f32),
        palette.border.hex()
    );
    let mut markup = drawn.markup;
    if let Some(at) = markup.rfind("</svg>") {
        markup.insert_str(at, &frame);
    }
    Ok(Svg { markup, ..drawn })
}
