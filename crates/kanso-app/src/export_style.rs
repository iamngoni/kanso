//! The faces, size and code colors an exported page sets text with: the
//! same ones the preview uses, so a page and the preview look alike.

use std::sync::Arc;

use gpui_kit::base::input::Rope;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::highlighter::{HighlightTheme, SyntaxHighlighter};
use gpui_kit::{App, FontStyle};
use kanso_render::Palette;
use kanso_render::html::{CodeSpan, Highlighter, PageOptions};

/// The preview's typography and code colors, ready for a page.
#[derive(Clone)]
pub struct PageStyle {
    /// The body face, when the settings or the theme name one.
    pub font: Option<String>,
    /// The monospaced face, when it isn't the system's.
    pub mono: Option<String>,
    /// The body text size, in points.
    pub size: f32,
    /// The colors the preview's code blocks are drawn in.
    pub highlight: Option<Arc<HighlightTheme>>,
}

impl Default for PageStyle {
    fn default() -> Self {
        PageStyle {
            font: None,
            mono: None,
            size: crate::config::DEFAULT_FONT_SIZE,
            highlight: None,
        }
    }
}

impl PageStyle {
    /// The style in use: the preview's size and face from Settings › Editor,
    /// the theme's monospaced face, and the preview's code colors.
    // no-prop-test: reads settings and globals; `highlight` has the
    // properties, and the page snapshots show the result.
    pub fn from_cx(cx: &mut App) -> Self {
        let ty = crate::preview::preview_type(cx);
        let mono = Some(cx.theme().mono_font_family.to_string());
        let highlight = cx
            .try_global::<crate::themes::ActiveTheme>()
            .and_then(|active| active.code.clone());
        PageStyle {
            font: Some(ty.family.to_string()),
            mono,
            size: ty.size,
            highlight,
        }
    }

    /// [`from_cx`](Self::from_cx), with code colors that read on `palette`'s
    /// page: a light page gets the person's light theme's colors when the
    /// active ones are made for a dark background. Every page goes through
    /// here, so none sets pale code on paper.
    // no-prop-test: reads settings and globals; `code_for_page` has the
    // properties.
    pub fn for_palette(cx: &mut App, palette: &Palette) -> Self {
        let mut style = Self::from_cx(cx);
        style.highlight = code_for_page(style.highlight.take(), palette, || {
            crate::themes::light_code(cx)
        });
        style
    }

    /// These settings as `page` and `site_index` take them.
    pub fn options(&self) -> PageOptions<'_> {
        PageOptions {
            font: self.font.as_deref(),
            mono: self.mono.as_deref(),
            size: self.size,
            highlight: self.highlight.as_ref().map(|_| self as &dyn Highlighter),
            ..PageOptions::default()
        }
    }
}

impl Highlighter for PageStyle {
    fn highlight(&self, lang: &str, code: &str) -> Option<Vec<CodeSpan>> {
        let theme = self.highlight.as_ref()?;
        let grammar = crate::code_lang::grammar(lang)?;
        let mut highlighter = SyntaxHighlighter::new(grammar);
        highlighter.update(None, &Rope::from_str(code), None);
        let mut spans = Vec::new();
        for (range, style) in highlighter.styles(&(0..code.len()), theme.as_ref()) {
            let bold = style.font_weight.is_some_and(|weight| weight.0 >= 600.);
            let italic = style.font_style == Some(FontStyle::Italic);
            let color = style.color.map(rgb);
            if color.is_none() && !bold && !italic {
                continue;
            }
            spans.push(CodeSpan {
                start: range.start,
                end: range.end,
                color,
                bold,
                italic,
            });
        }
        Some(spans)
    }
}

/// The code colors for a page drawn in `palette`: `active` unless it is
/// made for a dark background and the page is light, when `light` gives the
/// colors to use instead. Without any, the page's code isn't colored.
fn code_for_page(
    active: Option<Arc<HighlightTheme>>,
    palette: &Palette,
    light: impl FnOnce() -> Option<Arc<HighlightTheme>>,
) -> Option<Arc<HighlightTheme>> {
    match active {
        Some(theme) if theme.appearance.is_dark() && !palette.dark => light(),
        active => active,
    }
}

/// An `Hsla` color as `0xRRGGBB`. The syntax colors are opaque where they
/// matter; any translucency is dropped, as the page's code sits on its own
/// fill.
fn rgb(color: gpui_kit::Hsla) -> u32 {
    let rgba: gpui_kit::Rgba = color.into();
    let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u32;
    (channel(rgba.r) << 16) | (channel(rgba.g) << 8) | channel(rgba.b)
}

#[cfg(test)]
mod tests {
    use super::{PageStyle, code_for_page, rgb};
    use crate::themes::{builtins, code_colors};
    use gpui_kit::component::highlighter::HighlightTheme;
    use kanso_render::Palette;
    use kanso_render::html::Highlighter as _;
    use proptest::prelude::*;

    fn style() -> PageStyle {
        PageStyle {
            font: Some("Alegreya".into()),
            mono: Some("Iosevka".into()),
            size: 18.,
            highlight: Some(HighlightTheme::default_light()),
        }
    }

    #[test]
    fn code_is_colored_by_its_language() {
        let style = style();
        let spans = style
            .highlight("rust", "fn main() { let x = 1; }\n")
            .unwrap();
        assert!(!spans.is_empty());
        assert!(spans.iter().any(|span| span.color.is_some()));
        assert!(spans.iter().all(|span| span.start < span.end));
        assert_eq!(style.highlight("dart", "void main() {}"), None);
        assert_eq!(style.highlight("", "plain"), None);
    }

    #[test]
    fn a_page_names_the_faces_and_highlights_code() {
        let style = style();
        let html = kanso_render::html::page_with(
            "Code",
            "```rust\nfn main() {}\n```\n",
            &Palette::LIGHT,
            &|_| None,
            &style.options(),
        );
        assert!(html.contains("font:18px/"), "{html}");
        assert!(html.contains("\"Alegreya\""), "{html}");
        assert!(html.contains("\"Iosevka\""), "{html}");
        assert!(
            html.contains(r#"<span class="kanso-lang">rust</span>"#),
            "{html}"
        );
        assert!(html.contains("<span style=\"color:#"), "{html}");
        assert!(!html.contains("language-rust"));
    }

    /// The built-in theme named `name`.
    fn builtin(name: &str) -> std::sync::Arc<crate::themes::ThemeEntry> {
        builtins()
            .iter()
            .find(|theme| theme.name() == name)
            .unwrap_or_else(|| panic!("no built-in theme {name}"))
            .clone()
    }

    /// A style for a page in `palette` while `active` is the theme in use
    /// and Kanso Light is the person's light theme.
    fn page_style(active: &str, palette: &Palette) -> PageStyle {
        let highlight = code_for_page(code_colors(&builtin(active)), palette, || {
            code_colors(&builtin("Kanso Light"))
        });
        PageStyle {
            highlight,
            ..style()
        }
    }

    /// The light page published while `active` is the theme in use: Kanso's
    /// paper under a dark theme, else the theme's own code background.
    fn light_page(active: &crate::themes::ThemeEntry) -> Palette {
        if active.dark() {
            return Palette::LIGHT;
        }
        let tokens =
            kanso_engine::theme::Tokens::fit(&active.resolved, &active.resolved, &active.resolved);
        Palette {
            surface: kanso_render::Rgb(tokens.preview.code_background.0),
            ..Palette::LIGHT
        }
    }

    /// Code in languages the grammars cover, with every kind of token.
    const SAMPLES: [(&str, &str); 4] = [
        (
            "rust",
            "/// doc\nuse std::fmt;\n#[derive(Debug)]\npub struct A<'a> { x: &'a str, n: u32 }\nimpl A<'_> {\n    pub fn f(&self) -> Option<u8> {\n        let s = \"str\"; let c = 'c'; // note\n        if self.n > 0x1F { return None; }\n        /* block */ println!(\"{}\", 1.5e3); Some(true)\n    }\n}\n",
        ),
        (
            "python",
            "import os\n@decorator\nclass A(Base):\n    \"\"\"doc\"\"\"\n    def f(self, n: int = 3) -> str:\n        # note\n        return f\"{n}\" if n else None\n",
        ),
        (
            "javascript",
            "import x from \"y\";\nconst f = async (a, b) => { // note\n  return a?.b ?? `t${b}` + 0x1f;\n};\nclass A extends B { static n = 1; }\n",
        ),
        (
            "json",
            "{\"a\": [1, 2.5, true, null], \"b\": {\"c\": \"d\"}}\n",
        ),
    ];

    /// The lowest contrast any span of `SAMPLES` has against `palette`'s
    /// code background in `style`, with the color that has it.
    fn worst(style: &PageStyle, palette: &Palette) -> (f64, u32) {
        SAMPLES
            .iter()
            .filter_map(|(lang, code)| style.highlight(lang, code))
            .flatten()
            .filter_map(|span| span.color)
            .map(|color| {
                let ratio = crate::theme_file::contrast(
                    kanso_engine::theme::Rgb(color),
                    kanso_engine::theme::Rgb(palette.surface.0),
                );
                (ratio, color)
            })
            .fold((21., 0), |a, b| if b.0 < a.0 { b } else { a })
    }

    /// Light built-in themes whose own code colors read under 3:1 on their
    /// own code background: published as they are in the preview, so the
    /// page keeps them. A dark theme's page never gets these (it gets the
    /// person's light theme's, Kanso Light's by default).
    const BELOW_THREE_TO_ONE: [&str; 21] = [
        "Catppuccin Latte",
        "Default Light",
        "Dawnfox",
        "Edge Light",
        "Everforest Light",
        "Flexoki Light",
        "Gruvbox Light",
        "Gruvbox Material Light",
        "Horizon Bright",
        "Iceberg Light",
        "Kanagawa Lotus",
        "Material Theme Lighter",
        "Min Light",
        "Night Owl Light",
        "Oxocarbon Light",
        "One Light",
        "Solarized Light",
        "Tokyo Night Day",
        "Tokyo Night Light",
        "Tomorrow",
        "Vitesse Light",
    ];

    /// The list above is exactly the themes that fall short, so it can't
    /// hide a theme that was fixed or one that broke.
    #[test]
    fn the_themes_under_three_to_one_are_the_ones_listed() {
        for theme in builtins() {
            let palette = light_page(theme);
            let (ratio, color) = worst(&page_style(theme.name(), &palette), &palette);
            let low = ratio < 3.;
            assert_eq!(
                low,
                BELOW_THREE_TO_ONE.contains(&theme.name()),
                "{}: {color:06x} reads {ratio:.2}:1",
                theme.name()
            );
        }
    }

    /// The color of the first colored span in `code`, as `0xRRGGBB`.
    fn first_color(style: &PageStyle, code: &str) -> Option<u32> {
        style
            .highlight("rust", code)?
            .into_iter()
            .find_map(|span| span.color)
    }

    #[test]
    fn a_dark_theme_publishes_code_in_kanso_lights_colors() {
        let code = "fn main() {}\n";
        let light = first_color(&page_style("Kanso Light", &Palette::LIGHT), code);
        let dark = first_color(&page_style("Kanso Dark", &Palette::DARK), code);
        let published = first_color(&page_style("Kanso Dark", &Palette::LIGHT), code);
        assert!(light.is_some() && dark.is_some());
        assert_ne!(light, dark, "the two themes color `fn` alike");
        assert_eq!(published, light);
    }

    #[test]
    fn a_light_theme_or_a_dark_page_keeps_its_own_colors() {
        for (active, palette) in [
            ("Kanso Light", &Palette::LIGHT),
            ("Kanso Dark", &Palette::DARK),
        ] {
            let own = code_colors(&builtin(active)).map(|theme| theme.name.clone());
            let chosen = code_for_page(code_colors(&builtin(active)), palette, || {
                panic!("the other theme's colors were asked for")
            });
            assert_eq!(chosen.map(|theme| theme.name.clone()), own);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// Invariant: whichever built-in theme is active, every color a
        /// light page's code gets reads on the page's code background, at
        /// 3:1 or better.
        #[test]
        fn prop_light_page_code_reads_whatever_theme_is_active(
            theme in 0..builtins().len(),
            code in "(fn|let|struct|\"s\"|// c|[0-9]|::|[a-z]| |\\n|;){1,80}",
        ) {
            let active = &builtins()[theme];
            let name = active.name().to_string();
            prop_assume!(!BELOW_THREE_TO_ONE.contains(&name.as_str()));
            let palette = light_page(active);
            let Some(spans) = page_style(&name, &palette).highlight("rust", &code) else {
                return Ok(());
            };
            for color in spans.into_iter().filter_map(|span| span.color) {
                let ratio = crate::theme_file::contrast(
                    kanso_engine::theme::Rgb(color),
                    kanso_engine::theme::Rgb(palette.surface.0),
                );
                prop_assert!(ratio >= 3., "{name}: {color:06x} reads {ratio:.2}:1 in {code:?}");
            }
        }

        /// Invariant: every span a code block gets lies inside it, in order,
        /// and never splits a character.
        #[test]
        fn prop_spans_stay_inside_the_code(code in "(fn|let|struct|\"s\"|[0-9]| |\\n|;|[a-z]){0,120}") {
            let style = style();
            let Some(spans) = style.highlight("rust", &code) else {
                return Ok(());
            };
            let mut last = 0;
            for span in spans {
                prop_assert!(span.start <= span.end);
                prop_assert!(span.end <= code.len());
                prop_assert!(code.is_char_boundary(span.start));
                prop_assert!(code.is_char_boundary(span.end));
                prop_assert!(span.start >= last);
                last = span.end;
            }
        }

        /// Invariant: any color becomes a six-digit hex, whatever its alpha.
        #[test]
        fn prop_colors_are_opaque_hex(h in 0f32..1., s in 0f32..1., l in 0f32..1., a in 0f32..1.) {
            let color = rgb(gpui_kit::Hsla { h, s, l, a });
            prop_assert!(color <= 0xFF_FFFF);
        }
    }
}
