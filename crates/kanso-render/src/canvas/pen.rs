//! How each pen turns a stroke's samples into an outline: the tools' shapes,
//! colors and opacity. Shared by the SVG renderer, the desktop view and the
//! phone, so a stroke looks the same everywhere.

use kanso_core::canvas::Stroke;

use crate::Palette;
use crate::freehand::{self, Options, Sample};

/// Samples a stroke is outlined from. A stroke longer than this keeps its
/// first samples.
pub const MAX_OUTLINE_SAMPLES: usize = freehand::MAX_SAMPLES;

/// The pressure a mouse, a trackpad or a finger writes: the stroke then gets
/// its width from speed.
pub const SIMULATED: f64 = 0.5;

/// A stroke's samples for the outliner.
pub fn samples(stroke: &Stroke) -> Vec<Sample> {
    stroke
        .points
        .iter()
        .take(MAX_OUTLINE_SAMPLES)
        .map(|p| Sample::new(p.x, p.y, p.pressure))
        .collect()
}

/// Whether the stroke carries no real pressure: every sample is
/// [`SIMULATED`].
pub fn simulated(stroke: &Stroke) -> bool {
    stroke
        .points
        .iter()
        .all(|p| (p.pressure - SIMULATED).abs() < 1e-9)
}

/// The shape of `stroke`'s pen. A pencil widens as it leans (a low altitude
/// angle); a marker and a highlighter keep one width.
pub fn options(stroke: &Stroke, complete: bool) -> Options {
    let width = if stroke.width.is_finite() && stroke.width > 0.0 {
        stroke.width.min(200.0)
    } else {
        2.0
    };
    let base = Options {
        size: width,
        simulate_pressure: simulated(stroke),
        complete,
        ..Options::default()
    };
    match stroke.tool.as_str() {
        "pencil" => Options {
            size: width * lean(stroke),
            thinning: 0.6,
            smoothing: 0.4,
            streamline: 0.3,
            ..base
        },
        "marker" => Options {
            thinning: 0.0,
            smoothing: 0.6,
            streamline: 0.5,
            simulate_pressure: false,
            ..base
        },
        "highlighter" => Options {
            size: width * 4.0,
            thinning: 0.0,
            smoothing: 0.7,
            streamline: 0.6,
            simulate_pressure: false,
            ..base
        },
        _ => Options {
            thinning: 0.5,
            smoothing: 0.5,
            streamline: 0.5,
            ..base
        },
    }
}

/// 1 for an upright pencil, up to 2.2 for one lying flat.
fn lean(stroke: &Stroke) -> f64 {
    let alts: Vec<f64> = stroke
        .points
        .iter()
        .filter_map(|p| p.tilt.map(|t| t.1))
        .collect();
    if alts.is_empty() {
        return 1.0;
    }
    let mean = alts.iter().sum::<f64>() / alts.len() as f64;
    let flat = 1.0 - (mean / std::f64::consts::FRAC_PI_2).clamp(0.0, 1.0);
    1.0 + flat * 1.2
}

/// The opacity a pen draws at.
pub fn opacity(tool: &str) -> f64 {
    match tool {
        "highlighter" => 0.35,
        "pencil" => 0.85,
        _ => 1.0,
    }
}

/// The outline of `stroke` in its node's coordinates; `complete` for a
/// finished stroke (the last sample is kept as it is). Pure.
pub fn outline(stroke: &Stroke, complete: bool) -> Vec<[f64; 2]> {
    freehand::outline(&samples(stroke), &options(stroke, complete))
}

/// `color` as a pen colour on `palette`: a near-black pen turns to the page's
/// text colour on a dark page and a near-white one on a light page, so the
/// ink never vanishes into the background. Anything that isn't a hex colour
/// draws as the text colour.
pub fn ink_color(color: &str, palette: &Palette) -> String {
    let Some(rgb) = parse_hex(color) else {
        return palette.text.hex();
    };
    let luma = (0.299 * rgb.0 as f64 + 0.587 * rgb.1 as f64 + 0.114 * rgb.2 as f64) / 255.0;
    if (palette.dark && luma < 0.18) || (!palette.dark && luma > 0.85) {
        return palette.text.hex();
    }
    format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
}

/// `#rgb` or `#rrggbb`.
pub fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    if !hex.is_ascii() {
        return None;
    }
    let byte = |s: &str| u8::from_str_radix(s, 16).ok();
    match hex.len() {
        6 => Some((byte(&hex[0..2])?, byte(&hex[2..4])?, byte(&hex[4..6])?)),
        3 => {
            let n = |i: usize| byte(&hex[i..=i]).map(|v| v * 17);
            Some((n(0)?, n(1)?, n(2)?))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::canvas::Point;
    use proptest::prelude::*;

    fn stroke(tool: &str, pressure: f64) -> Stroke {
        Stroke {
            id: "s".into(),
            tool: tool.into(),
            color: "#112233".into(),
            width: 3.0,
            points: (0..20)
                .map(|i| Point::new(i as f64 * 4.0, (i as f64).sin() * 5.0, pressure))
                .collect(),
            ..Stroke::default()
        }
    }

    #[test]
    fn a_mouse_stroke_simulates_pressure() {
        assert!(simulated(&stroke("pen", 0.5)));
        assert!(!simulated(&stroke("pen", 0.8)));
        assert!(options(&stroke("pen", 0.5), true).simulate_pressure);
        assert!(!options(&stroke("pen", 0.8), true).simulate_pressure);
    }

    #[test]
    fn tools_differ() {
        let pen = options(&stroke("pen", 0.8), true);
        let marker = options(&stroke("marker", 0.8), true);
        let hi = options(&stroke("highlighter", 0.8), true);
        assert_eq!(marker.thinning, 0.0);
        assert!(hi.size > pen.size * 3.0);
        assert!(opacity("highlighter") < opacity("pen"));
    }

    #[test]
    fn a_leaning_pencil_is_broader() {
        let mut upright = stroke("pencil", 0.8);
        let mut flat = stroke("pencil", 0.8);
        for p in &mut upright.points {
            p.tilt = Some((0.0, std::f64::consts::FRAC_PI_2));
        }
        for p in &mut flat.points {
            p.tilt = Some((0.0, 0.2));
        }
        assert!(options(&flat, true).size > options(&upright, true).size * 1.5);
        assert_eq!(options(&stroke("pencil", 0.8), true).size, 3.0);
    }

    #[test]
    fn near_black_ink_follows_the_page() {
        let mut p = Palette::DARK;
        assert_eq!(ink_color("#1e1e1e", &p), p.text.hex());
        assert_eq!(ink_color("#e03131", &p), "#e03131");
        p = Palette::LIGHT;
        assert_eq!(ink_color("#ffffff", &p), p.text.hex());
        assert_eq!(ink_color("#fff", &Palette::DARK), "#ffffff");
        assert_eq!(ink_color("red", &p), p.text.hex());
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(parse_hex("#0a0B0c"), Some((10, 11, 12)));
        assert_eq!(parse_hex("#abc"), Some((0xaa, 0xbb, 0xcc)));
        assert_eq!(parse_hex("abc"), None);
        assert_eq!(parse_hex("#ééé"), None);
        assert_eq!(parse_hex("#12"), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: any hex-ish string parses or is refused without a
        /// panic, and a parsed color survives `ink_color` as a hex color.
        #[test]
        fn prop_colors_never_panic(text in "\\PC{0,9}") {
            let _ = parse_hex(&text);
            let c = ink_color(&text, &Palette::LIGHT);
            prop_assert!(parse_hex(&c).is_some());
        }
    }
}
