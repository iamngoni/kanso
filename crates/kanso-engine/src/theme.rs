//! The colors the views draw with: Kanso's own two themes, warm graphite
//! (dark, the primary one) and soft paper (light), are [`DARK`] and
//! [`LIGHT`]; `themes/kanso-dark.json` and `themes/kanso-light.json` give the
//! same colors, and every other theme is read from a file the same way
//! ([`mod@file`]). [`Tokens`] fits three themes together for drawing.
//!
//! The apps turn these into whatever their toolkit draws with: the desktop
//! app's component library reads a theme file built from [`Colors`], and the
//! mobile app's `ThemeData` from the same fields. The views read [`Colors`]
//! directly for what a library can't express: three pane surfaces,
//! secondary text, and selected rows.
//!
//! Nothing here depends on a toolkit: a color is `0xRRGGBB`.

use std::collections::BTreeMap;

pub mod builtin;
pub mod file;
pub mod import;

use file::{PreviewColors, ResolvedTheme, TokenStyle, fit_preview, fit_syntax};

/// An sRGB color, `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u32);

impl Rgb {
    /// `#RRGGBB`.
    pub fn hex(self) -> String {
        format!("#{:06X}", self.0 & 0xFF_FFFF)
    }

    /// `#RRGGBBAA`, with `alpha` out of 255.
    pub fn hex_alpha(self, alpha: u8) -> String {
        format!("{}{alpha:02X}", self.hex())
    }
}

/// One theme's colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colors {
    /// Behind everything; the title bar.
    pub app: Rgb,
    pub sidebar: Rgb,
    pub list: Rgb,
    /// The editor and preview, and the theme's `background`.
    pub editor: Rgb,
    /// Menus and popovers.
    pub elevated: Rgb,
    /// Chips, banners and other quiet fills on the panes: a step off every
    /// pane, in both themes. (In light mode the elevated color is the
    /// lightest surface of all, so a fill in it would vanish.)
    pub fill: Rgb,
    pub text: Rgb,
    pub text_secondary: Rgb,
    pub text_muted: Rgb,
    pub divider: Rgb,
    pub accent: Rgb,
    /// Link text: the accent, shaded to stay readable on every surface.
    pub link: Rgb,
    /// The selected and hovered note row, on [`Self::list`].
    pub row_active: Rgb,
    pub row_hover: Rgb,
    /// The selected and hovered sidebar row, on [`Self::sidebar`].
    pub sidebar_active: Rgb,
    pub sidebar_hover: Rgb,
    /// Menu items and ghost buttons under the pointer.
    pub hover: Rgb,
    /// A selected ghost button, such as the current view mode: plainly
    /// stronger than a hover, in both themes.
    pub selected: Rgb,
    /// The row with the cursor, when line numbers show.
    pub active_line: Rgb,
    /// Primary buttons and the text on them.
    pub button: Rgb,
    pub button_text: Rgb,
    /// Code blocks in the preview.
    pub code: Rgb,
    /// Inline code, and literals in code blocks.
    pub code_text: Rgb,
    /// Keywords and strings in code blocks: present, but quiet.
    pub syntax_keyword: Rgb,
    pub syntax_string: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
}

pub const DARK: Colors = Colors {
    app: Rgb(0x181816),
    sidebar: Rgb(0x1D1D1A),
    list: Rgb(0x232320),
    editor: Rgb(0x282824),
    elevated: Rgb(0x30302B),
    fill: Rgb(0x30302B),
    text: Rgb(0xF1EFE7),
    text_secondary: Rgb(0xB7B3A8),
    text_muted: Rgb(0x7E7A70),
    divider: Rgb(0x3A3933),
    accent: Rgb(0x7E96B3),
    link: Rgb(0x869DB9),
    row_active: Rgb(0x30302B),
    row_hover: Rgb(0x2A2A26),
    sidebar_active: Rgb(0x2E2E2A),
    sidebar_hover: Rgb(0x252522),
    hover: Rgb(0x3A3933),
    selected: Rgb(0x3A3933),
    active_line: Rgb(0x2D2D29),
    button: Rgb(0x4A6385),
    button_text: Rgb(0xF1EFE7),
    code: Rgb(0x30302B),
    code_text: Rgb(0xCDB892),
    syntax_keyword: Rgb(0x8FB0D9),
    syntax_string: Rgb(0xA8C98A),
    success: Rgb(0x7BAA78),
    warning: Rgb(0xC49A5A),
    danger: Rgb(0xC96E63),
};

pub const LIGHT: Colors = Colors {
    app: Rgb(0xF4F1EA),
    sidebar: Rgb(0xEBE7DD),
    list: Rgb(0xF8F5EF),
    editor: Rgb(0xFCFAF4),
    elevated: Rgb(0xFFFDF8),
    fill: Rgb(0xE6E0D4),
    text: Rgb(0x26241F),
    text_secondary: Rgb(0x655F55),
    // The spec's #948B7D reads below 3:1 on most surfaces.
    text_muted: Rgb(0x857C6E),
    divider: Rgb(0xDDD6C9),
    accent: Rgb(0x5E7895),
    // The accent itself is under 4.5:1 for text.
    link: Rgb(0x4D6684),
    row_active: Rgb(0xECE7DC),
    row_hover: Rgb(0xF2EEE6),
    sidebar_active: Rgb(0xE2DDD0),
    sidebar_hover: Rgb(0xE5E0D5),
    hover: Rgb(0xF0ECE3),
    selected: Rgb(0xE6E0D4),
    active_line: Rgb(0xF6F3EB),
    button: Rgb(0x4D6684),
    button_text: Rgb(0xFFFDF8),
    code: Rgb(0xF4F1EA),
    code_text: Rgb(0x7D5B32),
    syntax_keyword: Rgb(0x3F6496),
    syntax_string: Rgb(0x4A7A35),
    // The spec's status colors are for the dark theme; on paper they fall
    // under 3:1, so they are shaded the way muted text and links are.
    success: Rgb(0x4F7F4C),
    warning: Rgb(0x8A6424),
    danger: Rgb(0xA6493F),
};

impl Colors {
    /// Quiet text that still says something (a row's time, a count, a
    /// hint, a shortcut): muted on a pane, and secondary on a `selected`
    /// row, whose fill takes muted text under 4.5:1.
    // no-prop-test: two cases per theme; `quiet_text_reads_on_selected_rows`
    // checks every one.
    pub fn quiet(&self, selected: bool) -> Rgb {
        if selected {
            self.text_secondary
        } else {
            self.text_muted
        }
    }
}

/// The three themes in use, fitted for drawing: the UI theme's own colors,
/// the syntax theme's styles shaded to read on the UI's editor, the preview
/// theme's colors shaded to read on its panes, and the colors of code blocks
/// in the preview. A dark syntax theme under a light UI still reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    /// The UI theme's name.
    pub name: String,
    pub dark: bool,
    pub ui: Colors,
    pub caret: Rgb,
    /// Selected text in the editor.
    pub selection: file::Color,
    /// The syntax theme's styles, fitted to the UI's editor and text.
    pub syntax: BTreeMap<&'static str, TokenStyle>,
    /// The preview theme's colors, fitted to the UI's editor surface.
    pub preview: PreviewColors,
    /// The styles a preview's code blocks are drawn in: the same as
    /// [`Self::syntax`] when the preview theme is the syntax theme, and
    /// otherwise the preview theme's own, fitted to its code background.
    pub code: BTreeMap<&'static str, TokenStyle>,
}

impl Tokens {
    /// Fits the themes in use: `ui` colors the panes and controls, `syntax`
    /// the editor's Markdown and code, `preview` the rendered note.
    pub fn fit(ui: &ResolvedTheme, syntax: &ResolvedTheme, preview: &ResolvedTheme) -> Tokens {
        let colors = ui.colors;
        let fitted = fit_syntax(&syntax.syntax, colors.editor, colors.text);
        let preview_fitted = fit_preview(&preview.preview, &colors);
        let code = if preview.name != syntax.name {
            fit_syntax(&preview.syntax, preview_fitted.code_background, colors.text)
        } else {
            fitted.clone()
        };
        Tokens {
            name: ui.name.clone(),
            dark: ui.dark,
            ui: colors,
            caret: ui.caret,
            selection: ui.selection,
            syntax: fitted,
            preview: preview_fitted,
            code,
        }
    }
}

/// How text drawn with `tokens` (outermost first, as a
/// [`crate::highlight::Span`] lists them) is styled: the style of each
/// token laid over the ones before it, attribute by attribute, so a later
/// token's color, slant or weight wins and what it leaves unset shows what is
/// under it. Tokens the theme gives no style leave the rest as it was.
#[cfg(feature = "highlight")]
pub fn style(
    tokens: &[crate::highlight::Token],
    syntax: &BTreeMap<&'static str, TokenStyle>,
) -> TokenStyle {
    tokens.iter().fold(TokenStyle::default(), |under, token| {
        syntax
            .get(token.key())
            .copied()
            .unwrap_or_default()
            .or(under)
    })
}

/// How strongly the accent tints the palette's selected row over the
/// elevated panel, in the dark theme and the light one: enough to show, and
/// little enough that secondary text on it still reads at 4.5:1.
pub const PALETTE_SELECTED_ALPHA: (f32, f32) = (0.22, 0.18);

/// Other people's cursors in a shared note, eight hues apart: each as it
/// shows on a light editor and on a dark one, at 3:1 or more against both
/// themes' editors, so a thin caret and its name read on either.
pub const CURSOR_COLOURS: [(Rgb, Rgb); 8] = [
    (Rgb(0x2F6AAE), Rgb(0x7FB0EC)),
    (Rgb(0x2D7A4D), Rgb(0x76C59C)),
    (Rgb(0xA6531A), Rgb(0xEDA160)),
    (Rgb(0x7550AE), Rgb(0xB89CEE)),
    (Rgb(0xB03A3A), Rgb(0xEE8484)),
    (Rgb(0x1E7474), Rgb(0x6CCBCB)),
    (Rgb(0x9E3B80), Rgb(0xE68BC6)),
    (Rgb(0x666C1E), Rgb(0xC6C672)),
];

/// The most saturated a tag color gets.
const TAG_SATURATION_MAX: f32 = 0.42;
/// The lightness band tag colors are kept in, on dark and light surfaces.
const TAG_LIGHTNESS_DARK: (f32, f32) = (0.50, 0.76);
const TAG_LIGHTNESS_LIGHT: (f32, f32) = (0.32, 0.50);
/// The input lightness that is spread across the band. Tag palettes sit in
/// it; lighter and darker colors take the band's ends.
const TAG_LIGHTNESS_SPREAD: (f32, f32) = (0.40, 0.60);

/// A tag's color calmed for display: the same hue, no more saturated than
/// it was or than the saturation cap, and light or dark enough to read on
/// the theme's surfaces. Differences between colors survive it, so Orange
/// and Brown, or Violet and Purple, stay apart. The stored color doesn't
/// change.
pub fn muted_tag_color(rgb: u32, dark: bool) -> u32 {
    let (hue, saturation, lightness) = to_hsl(rgb);
    let (saturation, lightness) = calm(saturation, lightness, dark);
    from_hsl(hue, saturation, lightness)
}

/// The saturation and lightness [`muted_tag_color`] gives a color. Both keep
/// their order instead of being cut off: saturation is compressed toward the
/// cap by a square root, so a stronger color stays stronger, and lightness is
/// stretched across the band, so a darker color stays darker.
fn calm(saturation: f32, lightness: f32, dark: bool) -> (f32, f32) {
    let (low, high) = if dark {
        TAG_LIGHTNESS_DARK
    } else {
        TAG_LIGHTNESS_LIGHT
    };
    let (from, to) = TAG_LIGHTNESS_SPREAD;
    let spread = ((lightness - from) / (to - from)).clamp(0., 1.);
    (
        saturation.min(TAG_SATURATION_MAX * saturation.sqrt()),
        low + (high - low) * spread,
    )
}

/// Hue in degrees `0..360`, saturation and lightness in `0..=1`.
fn to_hsl(rgb: u32) -> (f32, f32, f32) {
    let [r, g, b] = [16, 8, 0].map(|shift| ((rgb >> shift) & 0xFF) as f32 / 255.);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
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
    (hue, saturation.min(1.), lightness)
}

fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> u32 {
    let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;
    let sector = hue.rem_euclid(360.) / 60.;
    let x = chroma * (1. - (sector.rem_euclid(2.) - 1.).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.),
        1 => (x, chroma, 0.),
        2 => (0., chroma, x),
        3 => (0., x, chroma),
        4 => (x, 0., chroma),
        _ => (chroma, 0., x),
    };
    let m = lightness - chroma / 2.;
    let byte = |v: f32| ((v + m).clamp(0., 1.) * 255.).round() as u32;
    (byte(r) << 16) | (byte(g) << 8) | byte(b)
}

#[cfg(test)]
mod tests {
    use kanso_core::TagColor;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::file::{ThemeFile, contrast, mix};
    use super::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The WCAG contrast ratio of two colors: 1 for the same color, 21 for
    /// black on white.
    fn contrast_ratio(a: u32, b: u32) -> f64 {
        contrast(Rgb(a), Rgb(b))
    }

    fn rgb() -> impl Strategy<Value = u32> {
        0u32..=0xFF_FFFF
    }

    /// Every cursor colour reads at 3:1 on its theme's editor, and the
    /// eight stay apart from each other.
    #[test]
    fn cursor_colours_read_on_both_themes() {
        for (light, dark) in CURSOR_COLOURS {
            assert!(contrast_ratio(light.0, LIGHT.editor.0) >= 3.0, "{light:?}");
            assert!(contrast_ratio(dark.0, DARK.editor.0) >= 3.0, "{dark:?}");
        }
        for (i, a) in CURSOR_COLOURS.iter().enumerate() {
            for b in &CURSOR_COLOURS[i + 1..] {
                assert_ne!(a.0, b.0);
                assert_ne!(a.1, b.1);
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Symmetry and range: order doesn't matter, and the ratio is
        /// between 1 and 21, exactly 1 for a color against itself.
        #[test]
        fn prop_contrast_is_symmetric_and_bounded(a in rgb(), b in rgb()) {
            let ratio = contrast_ratio(a, b);
            prop_assert_eq!(ratio, contrast_ratio(b, a));
            prop_assert!((1.0..=21.0).contains(&ratio), "{ratio}");
            prop_assert_eq!(contrast_ratio(a, a), 1.0);
        }

        /// Monotonicity: a lighter grey stands out more against black and
        /// less against white.
        #[test]
        fn prop_contrast_grows_with_distance(a in 0u32..=255, b in 0u32..=255) {
            let (dark, light) = (a.min(b), a.max(b));
            let grey = |v: u32| v * 0x01_0101;
            prop_assert!(contrast_ratio(grey(light), 0) >= contrast_ratio(grey(dark), 0));
            prop_assert!(contrast_ratio(grey(light), 0xFF_FFFF) <= contrast_ratio(grey(dark), 0xFF_FFFF));
        }

        /// Invariant: a tag color keeps its hue, is never more saturated than
        /// it was or than the cap, and sits in its theme's lightness band.
        /// One step of 8-bit rounding is allowed for.
        #[test]
        fn prop_muted_tag_colors_keep_hue_and_calm_down(color in rgb(), dark in any::<bool>()) {
            let (hue, saturation, _) = to_hsl(color);
            let out = muted_tag_color(color, dark);
            let (out_hue, out_saturation, out_lightness) = to_hsl(out);
            let step = 1.5 / 255.;
            let (low, high) = if dark { TAG_LIGHTNESS_DARK } else { TAG_LIGHTNESS_LIGHT };
            prop_assert!(out_lightness >= low - step && out_lightness <= high + step, "{out_lightness}");
            // Saturation is measured against the band's own chroma limit, so
            // rounding near the band's edges costs a little more.
            let slack = 4. * step / (1. - (2. * out_lightness - 1.).abs());
            prop_assert!(out_saturation <= saturation.min(TAG_SATURATION_MAX) + slack,
                "{saturation} -> {out_saturation}");
            let chroma = (1. - (2. * out_lightness - 1.).abs()) * out_saturation;
            if chroma >= 0.15 {
                let diff = (out_hue - hue).abs();
                prop_assert!(diff.min(360. - diff) <= 3., "{hue} -> {out_hue}");
            }
        }

        /// Order preservation: a more saturated or lighter color never
        /// comes out less saturated or darker, and saturation doesn't depend
        /// on lightness or the other way round.
        #[test]
        fn prop_calming_keeps_order(a in unit(), b in unit(), other in unit(), dark in any::<bool>()) {
            let (less, more) = (a.min(b), a.max(b));
            let (s_less, l_less) = calm(less, other, dark);
            let (s_more, l_more) = calm(more, other, dark);
            prop_assert!(s_less <= s_more, "{less} -> {s_less}, {more} -> {s_more}");
            prop_assert_eq!(l_less, l_more);
            let (s_less, l_less) = calm(other, less, dark);
            let (s_more, l_more) = calm(other, more, dark);
            prop_assert!(l_less <= l_more, "{less} -> {l_less}, {more} -> {l_more}");
            prop_assert_eq!(s_less, s_more);
        }

        /// Strictness: different saturations stay different all the way up,
        /// and so do lightnesses across the spread, rather than meeting at
        /// the cap or the band's edge.
        #[test]
        fn prop_calming_keeps_colors_apart(a in unit(), gap in 0.01f32..1., dark in any::<bool>()) {
            let b = (a + gap).min(1.);
            prop_assume!(b - a >= 0.01);
            prop_assert!(calm(a, 0.5, dark).0 < calm(b, 0.5, dark).0, "{a} {b}");
            let (from, to) = TAG_LIGHTNESS_SPREAD;
            let (l1, l2) = (from + (to - from) * a, from + (to - from) * b);
            prop_assert!(calm(0.5, l1, dark).1 < calm(0.5, l2, dark).1, "{l1} {l2}");
        }
    }

    fn unit() -> impl Strategy<Value = f32> {
        0f32..=1.
    }

    /// CIE L*a*b* of an sRGB color (D65), for color differences.
    fn lab(rgb: u32) -> [f64; 3] {
        let [r, g, b] = [16, 8, 0].map(|shift| {
            let c = f64::from((rgb >> shift) & 0xFF) / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        });
        let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
        let f = |t: f64| {
            if t > 0.008856 {
                t.cbrt()
            } else {
                7.787 * t + 16. / 116.
            }
        };
        let (fx, fy, fz) = (f(x), f(y), f(z));
        [116. * fy - 16., 500. * (fx - fy), 200. * (fy - fz)]
    }

    /// The CIE76 difference of two colors: about 2 is just noticeable, 10
    /// tells two small dots apart at a glance.
    fn delta_e(a: u32, b: u32) -> f64 {
        let (a, b) = (lab(a), lab(b));
        (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
    }

    /// Every tag color stays distinct from every other in both themes, and
    /// readable as a dot on the surfaces tags sit on. The chromatic colors
    /// stay clearly apart; the three neutrals only differ in lightness, so
    /// they get a lower bar.
    #[test]
    fn tag_colors_stay_apart() {
        let neutral =
            |c: TagColor| matches!(c, TagColor::Default | TagColor::Grey | TagColor::Black);
        for (name, dark, c) in [("dark", true, &DARK), ("light", false, &LIGHT)] {
            for (ix, a) in TagColor::ALL.iter().enumerate() {
                let shown = muted_tag_color(a.rgb(), dark);
                for bg in [c.sidebar, c.list, c.editor, c.elevated, c.fill] {
                    let ratio = contrast_ratio(shown, bg.0);
                    assert!(ratio >= 3.0, "{name}: {a:?} on {bg:?} is {ratio:.2}");
                }
                for b in &TagColor::ALL[ix + 1..] {
                    let min = if neutral(*a) && neutral(*b) {
                        5.0
                    } else {
                        10.0
                    };
                    let diff = delta_e(shown, muted_tag_color(b.rgb(), dark));
                    assert!(diff >= min, "{name}: {a:?} and {b:?} are {diff:.1} apart");
                }
            }
        }
        // The pair that used to come out the same.
        let (orange, brown) = (TagColor::Orange.rgb(), TagColor::Brown.rgb());
        assert!(delta_e(muted_tag_color(orange, true), muted_tag_color(brown, true)) > 10.);
    }

    #[test]
    fn contrast_examples() {
        assert_eq!(contrast_ratio(0, 0xFF_FFFF), 21.0);
        assert!((contrast_ratio(0x767676, 0xFF_FFFF) - 4.54).abs() < 0.01);
    }

    /// Every text color reads on every surface it can sit on, in both
    /// themes: primary text at 7:1, secondary at 4.5:1, muted at 3:1, links
    /// at 4.5:1. Muted text and links don't sit on the brief hover fill of a
    /// menu item, and links aren't drawn on sidebar rows. Status colors,
    /// drawn as icons and dots, read at 3:1 on every surface.
    #[test]
    fn text_contrast_on_every_surface() {
        for (name, c) in [("dark", &DARK), ("light", &LIGHT)] {
            let surfaces = [
                ("app", c.app, true),
                ("sidebar", c.sidebar, false),
                ("list", c.list, true),
                ("editor", c.editor, true),
                ("elevated", c.elevated, true),
                ("fill", c.fill, true),
                ("row_active", c.row_active, true),
                ("row_hover", c.row_hover, true),
                ("sidebar_active", c.sidebar_active, false),
                ("sidebar_hover", c.sidebar_hover, false),
                ("code", c.code, true),
            ];
            let all_surfaces = surfaces
                .iter()
                .map(|&(surface, bg, _)| (surface, bg))
                .chain([("hover", c.hover), ("selected", c.selected)]);
            for (surface, bg) in all_surfaces {
                for (status, fg) in [
                    ("success", c.success),
                    ("warning", c.warning),
                    ("danger", c.danger),
                ] {
                    let ratio = contrast_ratio(fg.0, bg.0);
                    assert!(
                        ratio >= 3.0,
                        "{name}: {status} on {surface} is {ratio:.2}, wants 3"
                    );
                }
            }
            // A chip or banner shows on the panes it sits on.
            for (surface, bg) in [("list", c.list), ("editor", c.editor)] {
                let ratio = contrast_ratio(c.fill.0, bg.0);
                assert!(ratio >= 1.1, "{name}: fill on {surface} is {ratio:.2}");
            }
            for (surface, bg, links) in surfaces {
                for (text, fg, min) in [
                    ("text", c.text, 7.0),
                    ("text_secondary", c.text_secondary, 4.5),
                    ("text_muted", c.text_muted, 3.0),
                    ("link", if links { c.link } else { c.text }, 4.5),
                ] {
                    let ratio = contrast_ratio(fg.0, bg.0);
                    assert!(
                        ratio >= min,
                        "{name}: {text} on {surface} is {ratio:.2}, wants {min}"
                    );
                }
            }
            for fg in [c.text, c.text_secondary] {
                assert!(
                    contrast_ratio(fg.0, c.hover.0) >= 4.5,
                    "{name}: {fg:?} on hover"
                );
            }
            // Code text sits on the code background and the editor.
            for bg in [c.code, c.editor] {
                for fg in [c.code_text, c.syntax_keyword, c.syntax_string] {
                    assert!(
                        contrast_ratio(fg.0, bg.0) >= 4.5,
                        "{name}: {fg:?} on {bg:?}"
                    );
                }
            }
            assert!(
                contrast_ratio(c.button_text.0, c.button.0) >= 4.5,
                "{name}: button"
            );
            // A switch's thumb shows on its track, off and on, and an off
            // track shows on the panes (it is the muted text color, checked
            // on every surface above).
            let thumb = if name == "dark" { c.text } else { c.elevated };
            for track in [c.text_muted, c.button] {
                assert!(
                    contrast_ratio(thumb.0, track.0) >= 3.0,
                    "{name}: switch thumb on {track:?}"
                );
            }
            // Text on a status color, as in a destructive button.
            let on_status = if name == "dark" { c.app } else { c.button_text };
            for fill in [c.success, c.warning, c.danger] {
                assert!(
                    contrast_ratio(on_status.0, fill.0) >= 4.5,
                    "{name}: text on {fill:?}"
                );
            }
            // Selected rows are a subtle fill, but one that shows against
            // their pane. A chip on a selected row takes the pane's color,
            // so it shows by the same step: in the dark theme no fill can be
            // a step off the selected row and still carry muted text at 3:1.
            assert!(
                contrast_ratio(c.row_active.0, c.list.0) >= 1.08,
                "{name}: row"
            );
            assert!(
                contrast_ratio(c.sidebar_active.0, c.sidebar.0) >= 1.08,
                "{name}: sidebar row"
            );
            // A selected ghost button shows on the editor, more than one
            // under the pointer does (the library hovers a ghost button with
            // the accent fill, at half strength in dark mode).
            let ghost_hover = if name == "dark" {
                mix(c.hover, c.editor, 0.5)
            } else {
                c.hover
            };
            let selected = contrast_ratio(c.selected.0, c.editor.0);
            assert!(selected >= 1.2, "{name}: selected is {selected:.2}");
            for hover in [
                ghost_hover,
                if name == "dark" { c.elevated } else { c.hover },
            ] {
                assert!(
                    selected > contrast_ratio(hover.0, c.editor.0),
                    "{name}: selected vs {hover:?}"
                );
            }
            assert!(contrast_ratio(c.text.0, c.selected.0) >= 7.0, "{name}");
            assert!(
                contrast_ratio(c.text_secondary.0, c.selected.0) >= 4.5,
                "{name}"
            );
            // The palette's selected row: the accent over the elevated
            // panel. Its label is primary text; its icon, hint and keys
            // turn secondary on it, and it shows against the panel.
            let alpha = if name == "dark" {
                PALETTE_SELECTED_ALPHA.0
            } else {
                PALETTE_SELECTED_ALPHA.1
            };
            let palette_selected = mix(c.accent, c.elevated, f64::from(alpha));
            for (text, fg, min) in [
                ("text", c.text, 7.0),
                ("text_secondary", c.text_secondary, 4.5),
            ] {
                let ratio = contrast_ratio(fg.0, palette_selected.0);
                assert!(
                    ratio >= min,
                    "{name}: {text} on the palette's selected row is {ratio:.2}, wants {min}"
                );
            }
            assert!(
                contrast_ratio(palette_selected.0, c.elevated.0) >= 1.2,
                "{name}: palette selected row"
            );
        }
    }

    /// Times, counts and hints are small (11.5 to 13 px), so on a selected
    /// row, note or sidebar or palette, they take the 4.5:1 that small
    /// text needs; muted text falls short of it on every one of those
    /// fills, which is why they don't stay muted there.
    #[test]
    fn quiet_text_reads_on_selected_rows() {
        for (name, c, alpha) in [
            ("dark", &DARK, PALETTE_SELECTED_ALPHA.0),
            ("light", &LIGHT, PALETTE_SELECTED_ALPHA.1),
        ] {
            assert_eq!(c.quiet(false), c.text_muted, "{name}");
            let palette_selected = mix(c.accent, c.elevated, f64::from(alpha));
            for (fill, bg) in [
                ("row_active", c.row_active),
                ("sidebar_active", c.sidebar_active),
                ("palette", palette_selected),
            ] {
                let ratio = contrast_ratio(c.quiet(true).0, bg.0);
                assert!(ratio >= 4.5, "{name}: quiet text on {fill} is {ratio:.2}");
                assert!(
                    contrast_ratio(c.text_muted.0, bg.0) < 4.5,
                    "{name}: muted text would do on {fill}"
                );
            }
        }
    }

    /// The spec's own light muted text and accent fall short as text, and
    /// its (dark) status colors fall short on paper, which is why [`LIGHT`]
    /// shades them. Its elevated color is the lightest light surface, so it
    /// can't be a fill there.
    #[test]
    fn spec_colors_that_were_shaded() {
        let spec_muted = 0x948B7D;
        assert!(contrast_ratio(spec_muted, LIGHT.sidebar.0) < 3.0);
        let spec_accent = 0x5E7895;
        assert!(contrast_ratio(spec_accent, LIGHT.sidebar.0) < 4.5);
        assert!(contrast_ratio(DARK.accent.0, DARK.elevated.0) < 4.5);
        for spec_status in [DARK.success, DARK.warning, DARK.danger] {
            assert!(contrast_ratio(spec_status.0, LIGHT.sidebar.0) < 3.0);
        }
        assert!(contrast_ratio(LIGHT.elevated.0, LIGHT.list.0) < 1.1);
    }

    #[test]
    fn tag_examples() {
        // Orange stays orange, and black becomes a readable grey.
        let (hue, saturation, _) = to_hsl(muted_tag_color(TagColor::Orange.rgb(), true));
        assert!(
            (20.0..30.0).contains(&hue) && saturation <= TAG_SATURATION_MAX + 0.01,
            "{hue} {saturation}"
        );
        // A grey keeps no tint, and a weak color isn't calmed further.
        assert_eq!(calm(0., 0.5, true).0, 0.);
        assert_eq!(calm(0.1, 0.5, true).0, 0.1);
        let black = muted_tag_color(TagColor::Black.rgb(), true);
        assert!(contrast_ratio(black, DARK.sidebar.0) >= 3.0);
        assert_eq!(from_hsl(0., 0., 1.), 0xFF_FFFF);
        assert_eq!(from_hsl(120., 1., 0.5), 0x00_FF00);
    }

    /// The crate's folder: cargo's at run time (the phone runners set it to where
    /// they copied the fixtures), else where it was built.
    fn manifest_dir() -> std::path::PathBuf {
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(Into::into)
            .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
    }

    /// The themes in the repository's `themes/` folder, as `index.json`
    /// lists them: `(file, text, parsed)`.
    fn theme_files() -> Vec<(String, String, ThemeFile)> {
        let root = manifest_dir().join("../../themes");
        assert!(
            root.is_dir(),
            "themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
        );
        let index: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("index.json")).unwrap())
                .unwrap();
        index["themes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let file = entry["file"].as_str().unwrap().to_owned();
                let text = std::fs::read_to_string(root.join(&file))
                    .unwrap_or_else(|err| panic!("{file}: {err}"));
                let theme = ThemeFile::parse(&text).unwrap_or_else(|err| panic!("{file} {err}"));
                (file, text, theme)
            })
            .collect()
    }

    /// Every file `themes/index.json` lists reads, under a name of its own,
    /// in the form `kanso theme fmt` writes, and every theme file in the
    /// folder is listed.
    #[test]
    fn every_theme_file_parses() {
        let files = theme_files();
        assert!(files.len() >= 2, "{} themes", files.len());
        let mut names = std::collections::HashSet::new();
        for (file, text, theme) in &files {
            assert!(
                names.insert(theme.name.to_lowercase()),
                "{file}: a name used twice"
            );
            assert!(!theme.license.is_empty(), "{file}: no license");
            assert_eq!(theme.to_json_string(), *text, "{file} isn't formatted");
            // Resolving gives a theme that agrees with its appearance.
            let resolved = theme.resolve();
            assert_eq!(resolved.dark, theme.appearance.is_dark(), "{file}");
        }
        let root = manifest_dir().join("../../themes");
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.extension().is_some_and(|x| x == "json") && name != "index.json" {
                assert!(
                    files.iter().any(|(file, _, _)| *file == name),
                    "{name} isn't listed"
                );
            }
        }
    }

    /// Kanso's own two files give the palettes above, the caret and
    /// selection of the accent, and fitting them to themselves changes
    /// nothing.
    #[test]
    fn kanso_theme_files_are_the_palettes() {
        let files = theme_files();
        for (file, expected) in [("kanso-light.json", &LIGHT), ("kanso-dark.json", &DARK)] {
            let theme = files
                .iter()
                .find(|(f, _, _)| f == file)
                .unwrap()
                .2
                .resolve();
            assert_eq!(&theme.colors, expected, "{file}");
            assert_eq!(theme.caret, expected.accent);
            assert_eq!(theme.selection.rgb, expected.accent);
            assert_eq!(theme.selection.alpha, 0x4D);
            let tokens = Tokens::fit(&theme, &theme, &theme);
            assert_eq!(tokens.ui, *expected);
            assert_eq!(tokens.syntax, theme.syntax);
            assert_eq!(tokens.code, tokens.syntax);
            assert_eq!(tokens.preview, theme.preview);
            assert_eq!(
                (tokens.name.as_str(), tokens.dark),
                (theme.name.as_str(), theme.dark)
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, ..config() })]

        /// Any three of the themes fit together: the syntax styles read at
        /// 3:1 on the UI's editor, the preview's text at 4.5:1 on it, code
        /// in the preview is the syntax's own when the preview theme is the
        /// syntax theme and reads at 3:1 on the code background when it
        /// isn't (where the UI's text reads on that background), and the UI
        /// theme's own parts come through as they are.
        #[test]
        fn prop_fitted_themes_read(ui in 0usize..81, syntax in 0usize..81, preview in 0usize..81) {
            let files = theme_files();
            let pick = |ix: usize| files[ix % files.len()].2.resolve();
            let (ui, syntax, preview) = (pick(ui), pick(syntax), pick(preview));
            let tokens = Tokens::fit(&ui, &syntax, &preview);
            prop_assert_eq!(tokens.ui, ui.colors);
            prop_assert_eq!(tokens.caret, ui.caret);
            prop_assert_eq!(tokens.selection, ui.selection);
            // Shading goes toward the UI's text, so it can only work where that reads.
            prop_assume!(contrast(ui.colors.text, ui.colors.editor) >= 3.0);
            for style in tokens.syntax.values() {
                if let Some(color) = style.color {
                    prop_assert!(contrast(color, ui.colors.editor) >= 3.0, "{color:?}");
                }
            }
            prop_assert!(contrast(tokens.preview.text, ui.colors.editor) >= 4.5);
            if preview.name == syntax.name {
                prop_assert_eq!(&tokens.code, &tokens.syntax);
            } else if contrast(ui.colors.text, tokens.preview.code_background) >= 3.0 {
                // Code colors are shaded toward the UI's text too, which
                // can't read on a code background of its own color.
                for style in tokens.code.values() {
                    if let Some(color) = style.color {
                        prop_assert!(contrast(color, tokens.preview.code_background) >= 3.0, "{color:?}");
                    }
                }
            }
        }
    }
}
