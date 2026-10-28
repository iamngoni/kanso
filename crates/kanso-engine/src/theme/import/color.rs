//! Reading the colors CSS and LESS stylesheets write: `#rgb` to
//! `#rrggbbaa`, `rgb()`/`rgba()` and `hsl()`/`hsla()` in both their comma
//! and space syntaxes, `transparent`, the common named colors, `color-mix()`
//! in sRGB, and the LESS color functions themes use (`fade`, `fadein`,
//! `fadeout`, `lighten`, `darken`).
//!
//! Theme packages write their colors in every one of these forms, and a
//! value is often a shorthand with a color inside it (`1px solid #333`), so
//! [`extract`] finds the first color among a value's words.

use crate::theme::Rgb;
use crate::theme::file::Color;

/// The color `text` names, or `None` when it isn't one this reads (a
/// length, `inherit`, `currentColor`, an unresolved `var()`).
pub fn parse(text: &str) -> Option<Color> {
    let text = text.trim();
    let text = text
        .strip_suffix("!important")
        .map(str::trim_end)
        .unwrap_or(text);
    let lower = text.to_ascii_lowercase();
    if let Some(digits) = lower.strip_prefix('#') {
        return parse_hex(digits);
    }
    if let Some((name, args)) = function(&lower) {
        return match name {
            "rgb" | "rgba" => rgb_function(args),
            "hsl" | "hsla" => hsl_function(args),
            "fade" | "fadein" | "fadeout" | "lighten" | "darken" => less_function(name, args),
            "color-mix" => color_mix(args),
            _ => None,
        };
    }
    named(&lower)
}

/// The first color among `value`'s words: the whole value when it is one,
/// else the color in a shorthand such as `1px solid hsl(0 0% 15%)`.
pub fn extract(value: &str) -> Option<Color> {
    parse(value).or_else(|| words(value).into_iter().find_map(parse))
}

/// `value` split at its top-level spaces and commas, keeping each function
/// call whole.
fn words(value: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, b) in value.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b' ' | b'\t' | b'\n' | b'\r' | b',' if depth == 0 => {
                if i > start {
                    out.push(&value[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < value.len() {
        out.push(&value[start..]);
    }
    out
}

/// `name(args)` when `text` is exactly one function call.
fn function(text: &str) -> Option<(&str, &str)> {
    let open = text.find('(')?;
    let name = text[..open].trim();
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphabetic() || b == b'-') {
        return None;
    }
    let close = matching_paren(text, open)?;
    if !text[close + 1..].trim().is_empty() {
        return None;
    }
    Some((name, &text[open + 1..close]))
}

/// The index of the `)` that closes the `(` at `open`.
pub fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in text.bytes().enumerate().skip(open) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// `text` split at its top-level commas.
pub fn split_commas(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(text[start..].trim());
    out
}

fn parse_hex(digits: &str) -> Option<Color> {
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let nibble = |i: usize| {
        u32::from_str_radix(&digits[i..=i], 16)
            .ok()
            .map(|n| n * 0x11)
    };
    let byte = |i: usize| u32::from_str_radix(&digits[i..i + 2], 16).ok();
    let (r, g, b, a) = match digits.len() {
        3 => (nibble(0)?, nibble(1)?, nibble(2)?, 0xFF),
        4 => (nibble(0)?, nibble(1)?, nibble(2)?, nibble(3)?),
        6 => (byte(0)?, byte(2)?, byte(4)?, 0xFF),
        8 => (byte(0)?, byte(2)?, byte(4)?, byte(6)?),
        _ => return None,
    };
    Some(Color {
        rgb: Rgb((r << 16) | (g << 8) | b),
        alpha: a as u8,
    })
}

/// A function's arguments: three channels and an optional alpha, from
/// `a, b, c[, d]` or `a b c[ / d]`.
fn channels(args: &str) -> Option<([&str; 3], Option<&str>)> {
    let parts = split_commas(args);
    let (main, alpha): (Vec<&str>, Option<&str>) = if parts.len() > 1 {
        match parts.len() {
            3 => (parts, None),
            4 => (parts[..3].to_vec(), Some(parts[3])),
            _ => return None,
        }
    } else {
        let (main, alpha) = match args.split_once('/') {
            Some((main, alpha)) => (main, Some(alpha.trim())),
            None => (args, None),
        };
        (main.split_whitespace().collect(), alpha)
    };
    let [a, b, c] = main.as_slice() else {
        return None;
    };
    Some(([a, b, c], alpha))
}

/// A number, or a percentage as a fraction of `full`.
fn number(text: &str, full: f64) -> Option<f64> {
    let text = text.trim();
    let value = match text.strip_suffix('%') {
        Some(pct) => pct.trim().parse::<f64>().ok()? / 100. * full,
        None => text.parse::<f64>().ok()?,
    };
    value.is_finite().then_some(value)
}

fn alpha_byte(text: Option<&str>) -> Option<u8> {
    match text {
        None => Some(0xFF),
        Some(text) => Some((number(text, 1.)?.clamp(0., 1.) * 255.).round() as u8),
    }
}

fn rgb_function(args: &str) -> Option<Color> {
    // LESS's `rgba(@color, 0.5)`, once the variable is in place.
    let parts = split_commas(args);
    if parts.len() == 2
        && let Some(color) = parse(parts[0])
    {
        return Some(Color {
            rgb: color.rgb,
            alpha: alpha_byte(Some(parts[1]))?,
        });
    }
    let ([r, g, b], alpha) = channels(args)?;
    let channel = |t: &str| number(t, 255.).map(|v| v.clamp(0., 255.).round() as u32);
    Some(Color {
        rgb: Rgb((channel(r)? << 16) | (channel(g)? << 8) | channel(b)?),
        alpha: alpha_byte(alpha)?,
    })
}

/// A hue in degrees, from a bare number or one in `deg`, `grad`, `rad` or
/// `turn`.
fn hue(text: &str) -> Option<f64> {
    let text = text.trim();
    let units: [(&str, f64); 4] = [
        ("deg", 1.),
        ("grad", 0.9),
        ("rad", 180. / std::f64::consts::PI),
        ("turn", 360.),
    ];
    for (unit, scale) in units {
        if let Some(n) = text.strip_suffix(unit) {
            return n
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(|v| v * scale);
        }
    }
    text.parse::<f64>().ok().filter(|v| v.is_finite())
}

fn hsl_function(args: &str) -> Option<Color> {
    let ([h, s, l], alpha) = channels(args)?;
    // `hsl(213 94% 68%)` and the older `hsl(213, 0.94, 0.68)` both appear.
    let fraction = |t: &str| {
        let t = t.trim();
        if t.ends_with('%') {
            number(t, 1.)
        } else {
            number(t, 1.).map(|v| if v > 1. { v / 100. } else { v })
        }
    };
    Some(Color {
        rgb: hsl_to_rgb(
            hue(h)?,
            fraction(s)?.clamp(0., 1.),
            fraction(l)?.clamp(0., 1.),
        ),
        alpha: alpha_byte(alpha)?,
    })
}

/// HSL (hue in degrees, saturation and lightness 0 to 1) as sRGB.
pub fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> Rgb {
    let h = hue.rem_euclid(360.) / 360.;
    let q = if lightness < 0.5 {
        lightness * (1. + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2. * lightness - q;
    let channel = |t: f64| {
        let t = t.rem_euclid(1.);
        let v = if t < 1. / 6. {
            p + (q - p) * 6. * t
        } else if t < 0.5 {
            q
        } else if t < 2. / 3. {
            p + (q - p) * (2. / 3. - t) * 6.
        } else {
            p
        };
        (v.clamp(0., 1.) * 255.).round() as u32
    };
    Rgb((channel(h + 1. / 3.) << 16) | (channel(h) << 8) | channel(h - 1. / 3.))
}

/// sRGB as HSL: hue in degrees, saturation and lightness 0 to 1.
pub fn rgb_to_hsl(rgb: Rgb) -> (f64, f64, f64) {
    let [r, g, b] = [16, 8, 0].map(|s| f64::from((rgb.0 >> s) & 0xFF) / 255.);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let lightness = (max + min) / 2.;
    let chroma = max - min;
    if chroma == 0. {
        return (0., 0., lightness);
    }
    let saturation = chroma / (1. - (2. * lightness - 1.).abs());
    let hue = if max == r {
        60. * ((g - b) / chroma).rem_euclid(6.)
    } else if max == g {
        60. * ((b - r) / chroma + 2.)
    } else {
        60. * ((r - g) / chroma + 4.)
    };
    (hue, saturation.clamp(0., 1.), lightness)
}

/// LESS's color functions: `fade(@c, 13%)` sets the alpha, `fadein` and
/// `fadeout` move it, `lighten` and `darken` move the HSL lightness.
fn less_function(name: &str, args: &str) -> Option<Color> {
    let [color, amount] = split_commas(args)[..] else {
        return None;
    };
    let color = parse(color)?;
    let amount = number(amount, 1.)?;
    let alpha = f64::from(color.alpha) / 255.;
    let with_alpha = |a: f64| Color {
        rgb: color.rgb,
        alpha: (a.clamp(0., 1.) * 255.).round() as u8,
    };
    let with_lightness = |delta: f64| {
        let (h, s, l) = rgb_to_hsl(color.rgb);
        Color {
            rgb: hsl_to_rgb(h, s, (l + delta).clamp(0., 1.)),
            alpha: color.alpha,
        }
    };
    Some(match name {
        "fade" => with_alpha(amount),
        "fadein" => with_alpha(alpha + amount),
        "fadeout" => with_alpha(alpha - amount),
        "lighten" => with_lightness(amount),
        _ => with_lightness(-amount),
    })
}

/// `color-mix(in srgb, A p%, B q%)`, mixed in sRGB.
fn color_mix(args: &str) -> Option<Color> {
    let parts = split_commas(args);
    let [space, a, b] = parts[..] else {
        return None;
    };
    if !space.starts_with("in ") {
        return None;
    }
    // A color and its optional percentage, in either order.
    let side = |text: &str| -> Option<(Color, Option<f64>)> {
        let words = words(text);
        let color = words.iter().find_map(|w| parse(w))?;
        let pct = words
            .iter()
            .find(|w| w.ends_with('%'))
            .and_then(|w| number(w, 1.));
        Some((color, pct))
    };
    let ((ca, pa), (cb, pb)) = (side(a)?, side(b)?);
    let wa = match (pa, pb) {
        (Some(pa), Some(pb)) if pa + pb > 0. => pa / (pa + pb),
        (Some(pa), _) => pa,
        (None, Some(pb)) => 1. - pb,
        (None, None) => 0.5,
    }
    .clamp(0., 1.);
    let alpha = f64::from(ca.alpha) * wa + f64::from(cb.alpha) * (1. - wa);
    Some(Color {
        rgb: crate::theme::file::mix(ca.rgb, cb.rgb, wa),
        alpha: alpha.round() as u8,
    })
}

/// The CSS named colors themes use, and `transparent`.
fn named(name: &str) -> Option<Color> {
    let rgb = match name {
        "transparent" => {
            return Some(Color {
                rgb: Rgb(0),
                alpha: 0,
            });
        }
        "black" => 0x000000,
        "white" => 0xFFFFFF,
        "red" => 0xFF0000,
        "green" => 0x008000,
        "lime" => 0x00FF00,
        "blue" => 0x0000FF,
        "yellow" => 0xFFFF00,
        "cyan" | "aqua" => 0x00FFFF,
        "magenta" | "fuchsia" => 0xFF00FF,
        "orange" => 0xFFA500,
        "purple" => 0x800080,
        "pink" => 0xFFC0CB,
        "brown" => 0xA52A2A,
        "gray" | "grey" => 0x808080,
        "darkgray" | "darkgrey" => 0xA9A9A9,
        "lightgray" | "lightgrey" => 0xD3D3D3,
        "dimgray" | "dimgrey" => 0x696969,
        "silver" => 0xC0C0C0,
        "gainsboro" => 0xDCDCDC,
        "whitesmoke" => 0xF5F5F5,
        "maroon" => 0x800000,
        "olive" => 0x808000,
        "navy" => 0x000080,
        "teal" => 0x008080,
        "gold" => 0xFFD700,
        "indigo" => 0x4B0082,
        "violet" => 0xEE82EE,
        "tomato" => 0xFF6347,
        "coral" => 0xFF7F50,
        "salmon" => 0xFA8072,
        "crimson" => 0xDC143C,
        "orchid" => 0xDA70D6,
        "plum" => 0xDDA0DD,
        "khaki" => 0xF0E68C,
        "tan" => 0xD2B48C,
        "beige" => 0xF5F5DC,
        "ivory" => 0xFFFFF0,
        "linen" => 0xFAF0E6,
        "snow" => 0xFFFAFA,
        "skyblue" => 0x87CEEB,
        "steelblue" => 0x4682B4,
        "royalblue" => 0x4169E1,
        "dodgerblue" => 0x1E90FF,
        "slategray" | "slategrey" => 0x708090,
        "darkslategray" | "darkslategrey" => 0x2F4F4F,
        "seagreen" => 0x2E8B57,
        "forestgreen" => 0x228B22,
        "darkgreen" => 0x006400,
        "darkred" => 0x8B0000,
        "darkblue" => 0x00008B,
        "darkorange" => 0xFF8C00,
        "goldenrod" => 0xDAA520,
        "turquoise" => 0x40E0D0,
        "hotpink" => 0xFF69B4,
        "deeppink" => 0xFF1493,
        "rebeccapurple" => 0x663399,
        _ => return None,
    };
    Some(Color::opaque(rgb))
}

#[cfg(test)]
mod tests {
    use super::{extract, hsl_to_rgb, parse, rgb_to_hsl};
    use crate::theme::Rgb;
    use crate::theme::file::Color;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn channels(rgb: u32) -> [u32; 3] {
        [(rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF]
    }

    /// Whether two colors are within one step on every channel.
    fn close(a: Rgb, b: Rgb) -> bool {
        channels(a.0)
            .iter()
            .zip(channels(b.0))
            .all(|(x, y)| x.abs_diff(y) <= 1)
    }

    /// HSL from sRGB, written independently of the code under test: the
    /// textbook formula over the channels' extremes.
    fn reference_hsl(rgb: u32) -> (f64, f64, f64) {
        let [r, g, b] = channels(rgb).map(|c| f64::from(c) / 255.);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.;
        let d = max - min;
        if d == 0. {
            return (0., 0., l * 100.);
        }
        let s = if l > 0.5 {
            d / (2. - max - min)
        } else {
            d / (max + min)
        };
        let h = if max == r {
            (g - b) / d + if g < b { 6. } else { 0. }
        } else if max == g {
            (b - r) / d + 2.
        } else {
            (r - g) / d + 4.
        };
        (h * 60., s * 100., l * 100.)
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: every way of writing an opaque sRGB color in hex or
        /// `rgb()` reads back as exactly that color.
        #[test]
        fn prop_hex_and_rgb_read_exactly(rgb in 0u32..=0xFF_FFFF, alpha in 0u8..=255) {
            let [r, g, b] = channels(rgb);
            let want = Color::opaque(rgb);
            for text in [
                format!("#{rgb:06x}"),
                format!("#{rgb:06X}"),
                format!("rgb({r}, {g}, {b})"),
                format!("rgb({r} {g} {b})"),
                format!("RGBA({r},{g},{b},1)"),
                format!("rgb({r} {g} {b} / 100%)"),
            ] {
                prop_assert_eq!(parse(&text), Some(want), "{}", text);
            }
            let with_alpha = Color { rgb: Rgb(rgb), alpha };
            prop_assert_eq!(parse(&format!("#{rgb:06x}{alpha:02x}")), Some(with_alpha));
            prop_assert_eq!(Color::parse(&with_alpha.hex()).and_then(|c| parse(&c.hex())), Some(with_alpha));
            // An alpha written as a fraction lands on the same byte.
            let fraction = f64::from(alpha) / 255.;
            prop_assert_eq!(parse(&format!("rgba({r}, {g}, {b}, {fraction})")), Some(with_alpha));
        }

        /// Agreement: any color written in `hsl()` (in each syntax theme
        /// packages use) reads within one step per channel of the color it came from.
        #[test]
        fn prop_hsl_agrees_with_the_color(rgb in 0u32..=0xFF_FFFF) {
            let (h, s, l) = reference_hsl(rgb);
            for text in [
                format!("hsl({h:.4}, {s:.4}%, {l:.4}%)"),
                format!("hsl({h:.4}deg {s:.4}% {l:.4}%)"),
                format!("hsla({h:.4}, {s:.4}%, {l:.4}%, 1)"),
                format!("hsl({h:.4}deg {s:.4}% {l:.4}% / 100%)"),
                format!("hsl({:.6}turn {s:.4}% {l:.4}%)", h / 360.),
            ] {
                let got = parse(&text).map(|c| c.rgb);
                prop_assert!(got.is_some_and(|got| close(got, Rgb(rgb))), "{} read as {:?}", text, got);
            }
            // And the converter's own inverse comes back to the same color.
            let (h, s, l) = rgb_to_hsl(Rgb(rgb));
            prop_assert!(close(hsl_to_rgb(h, s, l), Rgb(rgb)));
        }

        /// Shorthands: a color inside a border or background shorthand is
        /// found whatever surrounds it.
        #[test]
        fn prop_a_color_is_found_in_a_shorthand(rgb in 0u32..=0xFF_FFFF, width in 0u8..9, style in prop::sample::select(vec!["solid", "dotted", "dashed"])) {
            let [r, g, b] = channels(rgb);
            for color in [format!("#{rgb:06x}"), format!("rgb({r}, {g}, {b})")] {
                let value = format!("{width}px {style} {color}");
                prop_assert_eq!(extract(&value), Some(Color::opaque(rgb)), "{}", value);
            }
        }

        /// Totality: no text makes the reader panic.
        #[test]
        fn prop_any_text_reads_without_panicking(text in "[#a-z0-9(),./% -]{0,40}") {
            let _ = parse(&text);
            let _ = extract(&text);
        }
    }

    #[test]
    fn the_forms_theme_packages_write() {
        let rgb = |text: &str| parse(text).map(|c| (c.rgb.hex(), c.alpha));
        assert_eq!(rgb("#fff"), Some(("#FFFFFF".into(), 255)));
        assert_eq!(rgb("#0008"), Some(("#000000".into(), 0x88)));
        assert_eq!(rgb("hsl(0deg 0% 98% / 87%)"), Some(("#FAFAFA".into(), 222)));
        assert_eq!(rgb("hsla(0, 0%, 100%, .5)"), Some(("#FFFFFF".into(), 128)));
        assert_eq!(
            rgb("rgba(255, 255, 255, 0.02)"),
            Some(("#FFFFFF".into(), 5))
        );
        assert_eq!(rgb("rgb(100% 0% 0%)"), Some(("#FF0000".into(), 255)));
        assert_eq!(rgb("transparent"), Some(("#000000".into(), 0)));
        assert_eq!(rgb("White"), Some(("#FFFFFF".into(), 255)));
        assert_eq!(rgb("#123456 !important"), Some(("#123456".into(), 255)));
        assert_eq!(rgb("fade(#707d8b, 13%)"), Some(("#707D8B".into(), 33)));
        assert_eq!(rgb("darken(#ffffff, 50%)"), Some(("#808080".into(), 255)));
        assert_eq!(rgb("rgba(#102030, 0.5)"), Some(("#102030".into(), 128)));
        assert_eq!(
            rgb("color-mix(in srgb, #000000 50%, #ffffff)"),
            Some(("#808080".into(), 255))
        );
        for not_a_color in [
            "inherit",
            "currentColor",
            "1px",
            "var(--x)",
            "#12345",
            "rgb(1, 2)",
            "hsl(x y z)",
        ] {
            assert_eq!(parse(not_a_color), None, "{not_a_color}");
        }
        assert_eq!(
            extract("1px solid hsl(0deg 0% 15% / 75%)").map(|c| c.alpha),
            Some(191)
        );
        assert_eq!(extract("1.07142857rem"), None);
    }
}
