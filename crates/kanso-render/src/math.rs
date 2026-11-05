//! TeX math as SVG: MathJax 3 with its liteDOM adaptor, run in the shared
//! QuickJS engine ([`crate::js`]). KaTeX's syntax is a subset of what
//! MathJax's TeX input reads, so notes written for KaTeX typeset the same.

use crate::js::{self, Call};
use crate::svg::{RootTag, number};
use crate::{Failure, Outcome, Palette, Svg, TEXT_SIZE};

/// The longest formula typeset; a longer one is shown as its source.
const MAX_SOURCE: usize = 16 * 1024;

/// MathJax's SVG for `source`, or the mistake it found. Blocks on the
/// engine's thread.
fn typeset(source: &str, display: bool) -> Result<String, String> {
    let json = js::call(Call::Tex {
        source: source.to_owned(),
        display,
    })?;
    let answer: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    match (answer["svg"].as_str(), answer["error"].as_str()) {
        (Some(svg), _) => Ok(svg.to_owned()),
        (None, Some(error)) => Err(error.to_owned()),
        (None, None) => Err("MathJax gave nothing back".into()),
    }
}

/// Sets `source` (TeX, without its `$` delimiters) as SVG in the palette's
/// text color.
pub(crate) fn render(source: &str, display: bool, palette: &Palette) -> Outcome {
    let source = source.trim();
    if source.is_empty() {
        return Err(Failure::Invalid("The formula is empty.".into()));
    }
    if source.len() > MAX_SOURCE {
        return Err(Failure::Invalid(
            "The formula is too long to typeset.".into(),
        ));
    }
    let svg = typeset(source, display).map_err(Failure::Invalid)?;
    finish(&svg, palette).ok_or_else(|| Failure::Invalid("MathJax's SVG was unreadable.".into()))
}

/// MathJax's SVG, sized in points from its view box (a thousandth of an em
/// a unit), in the text color, with its baseline found.
pub(crate) fn finish(svg: &str, palette: &Palette) -> Option<Svg> {
    let root = RootTag::parse(svg)?;
    let [x, y, w, h] = root.view_box()?;
    let _ = x;
    let unit = TEXT_SIZE / 1000.;
    let (width, height) = (w * unit, h * unit);
    if !(width.is_finite() && height.is_finite() && width > 0. && height > 0.) {
        return None;
    }
    let color = palette.text.hex();
    let markup = stable_ids(
        &root
            .with("width", number(width))
            .with("height", number(height))
            .without("style")
            .without("role")
            .without("focusable")
            .write(svg),
    )
    .replace("currentColor", &color);
    Some(Svg {
        markup,
        width,
        height,
        // The view box starts at the top of the tallest glyph, `-y` above
        // the baseline.
        baseline: Some(-y * unit),
        notices: Vec::new(),
    })
}

/// `svg` with MathJax's numbered IDs (`MJX-12-…`, counted per engine) made
/// from its content instead: the same formula always draws the same, and
/// two formulas on one page never share an ID for different glyphs. Pure.
fn stable_ids(svg: &str) -> String {
    use std::hash::{Hash as _, Hasher as _};
    let mut rest = svg;
    let mut numbered = String::new();
    while let Some(at) = rest.find("MJX-") {
        let digits = rest[at + 4..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits > 0 && rest.as_bytes().get(at + 4 + digits) == Some(&b'-') {
            numbered = rest[at..at + 4 + digits].to_owned();
            break;
        }
        rest = &rest[at + 4..];
    }
    if numbered.is_empty() {
        return svg.to_owned();
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    svg.replace(&numbered, "MJX").hash(&mut hasher);
    svg.replace(
        &numbered,
        &format!("kmx{:x}", hasher.finish() & 0xFFFF_FFFF),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::TIME_LIMIT;
    use proptest::prelude::*;
    use std::time::Instant;

    #[test]
    fn single_letters_typeset() {
        for source in ["r", "x", "a"] {
            let svg = render(source, false, &Palette::LIGHT).unwrap();
            assert!(svg.width > 3. && svg.height > 5., "{source}: {svg:?}");
        }
    }

    #[test]
    fn formulas_typeset_in_the_text_color() {
        let svg = render(r"\frac{a}{b} + \sqrt{x^2}", false, &Palette::LIGHT).unwrap();
        assert!(svg.markup.starts_with("<svg"));
        assert!(svg.markup.contains(&Palette::LIGHT.text.hex()));
        assert!(!svg.markup.contains("currentColor"));
        assert!(svg.width > 20. && svg.height > 10.);
        let baseline = svg.baseline.unwrap();
        assert!(baseline > 0. && baseline < svg.height);
        // Parses as an SVG, at the size it says.
        let tree = usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).unwrap();
        assert!((tree.size().width() - svg.width).abs() < 0.01);
    }

    #[test]
    fn katex_syntax_typesets() {
        for source in [
            r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}",
            r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
            r"\begin{aligned} x &= 1 \\ y &= 2 \end{aligned}",
            r"\mathbb{R} \to \mathcal{C}",
            r"\operatorname{sgn}(x) \cdot \text{if } x > 0",
            r"\begin{cases} 1 & x > 0 \\ 0 & \text{otherwise} \end{cases}",
            r"\xrightarrow{f} \overbrace{a+b}^{n} \boxed{x}",
            r"\color{red}{x} \cancel{y}",
        ] {
            assert!(render(source, true, &Palette::DARK).is_ok(), "{source}");
        }
    }

    #[test]
    fn mistakes_are_reported_not_drawn() {
        for source in [r"\frac{1}{", r"\badcommand{x}", "", r"x^^"] {
            assert!(
                matches!(
                    render(source, false, &Palette::LIGHT),
                    Err(Failure::Invalid(_))
                ),
                "{source}"
            );
        }
        match render(r"\undefinedthing", false, &Palette::LIGHT) {
            Err(Failure::Invalid(message)) => assert!(message.contains("undefinedthing")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_runaway_macro_stops() {
        let started = Instant::now();
        let outcome = render(r"\def\a{\a\a}\a", false, &Palette::LIGHT);
        assert!(outcome.is_err());
        assert!(started.elapsed() < TIME_LIMIT * 2);
        // And the engine still works after.
        assert!(render("x", false, &Palette::LIGHT).is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Invariants: whatever the input, typesetting answers (it never
        /// hangs or panics), and a drawing is a sized SVG with its
        /// baseline inside it.
        #[test]
        fn prop_typesetting_answers(source in "[a-z0-9+\\-=^_{}\\\\ ()]{0,24}", display in any::<bool>()) {
            if let Ok(svg) = render(&source, display, &Palette::LIGHT) {
                prop_assert!(svg.markup.starts_with("<svg"));
                prop_assert!(svg.width > 0. && svg.height > 0.);
                let baseline = svg.baseline.unwrap();
                prop_assert!(baseline >= 0. && baseline <= svg.height + 0.01);
            }
        }

        /// Metamorphic: the palette changes only the color, never the size.
        #[test]
        fn prop_palette_changes_only_color(n in 0u32..1000) {
            let source = format!(r"\frac{{{n}}}{{x_{n}}}");
            let light = render(&source, false, &Palette::LIGHT).unwrap();
            let dark = render(&source, false, &Palette::DARK).unwrap();
            prop_assert_eq!(light.width, dark.width);
            prop_assert_eq!(light.height, dark.height);
            prop_assert_eq!(
                light.markup.replace(&Palette::LIGHT.text.hex(), "C"),
                dark.markup.replace(&Palette::DARK.text.hex(), "C")
            );
        }
    }
}
