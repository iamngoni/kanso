//! How a page is set: its faces, size and code colors, and how a code block
//! gets its colors from the highlighter.

use kanso_render::Palette;
use kanso_render::html::PageOptions;

use super::model::{CodeColors, PageStyle};
use crate::theme::file::ResolvedTheme;

impl PageStyle {
    /// The body text size when settings don't say, in points.
    pub const DEFAULT_SIZE: f32 = 16.0;

    /// The style for `palette`'s page: `active` code colors, unless they are
    /// made for a dark background and the page is light, when `light` gives
    /// the colors to use instead (see [`code_for_page`]). Every page goes
    /// through here, so none sets pale code on paper.
    // no-prop-test: assembles its arguments; `code_for_page` has the
    // properties.
    pub fn for_palette(
        font: Option<String>,
        mono: Option<String>,
        size: f32,
        active: Option<CodeColors>,
        palette: &Palette,
        light: impl FnOnce() -> Option<CodeColors>,
    ) -> PageStyle {
        PageStyle {
            font,
            mono,
            size,
            code: code_for_page(active, palette, light),
        }
    }

    /// These settings as `page` and `site_index` take them.
    pub fn options(&self) -> PageOptions<'_> {
        PageOptions {
            font: self.font.as_deref(),
            mono: self.mono.as_deref(),
            size: self.size,
            highlight: self.highlighter(),
            // A published or exported page is the one anybody can open:
            // the app's addresses and restrictions are for the phone only.
            app: None,
        }
    }

    #[cfg(feature = "highlight")]
    fn highlighter(&self) -> Option<&dyn kanso_render::html::Highlighter> {
        self.code
            .as_ref()
            .map(|_| self as &dyn kanso_render::html::Highlighter)
    }

    /// Without the highlighter, code stays plain.
    #[cfg(not(feature = "highlight"))]
    fn highlighter(&self) -> Option<&dyn kanso_render::html::Highlighter> {
        None
    }
}

impl Default for PageStyle {
    fn default() -> Self {
        PageStyle {
            font: None,
            mono: None,
            size: PageStyle::DEFAULT_SIZE,
            code: None,
        }
    }
}

impl CodeColors {
    /// The code colors the themes in use give: `ui` colors the panes, `syntax`
    /// the editor and `preview` the rendered note, fitted together
    /// (`Tokens::fit`). When the preview theme is not the syntax theme its
    /// own syntax colors draw the code, on its code background, and its
    /// appearance says whether they are for a dark background.
    pub fn for_themes(
        ui: &ResolvedTheme,
        syntax: &ResolvedTheme,
        preview: &ResolvedTheme,
    ) -> CodeColors {
        let tokens = crate::theme::Tokens::fit(ui, syntax, preview);
        CodeColors {
            dark: if preview.name != syntax.name {
                preview.dark
            } else {
                syntax.dark
            },
            syntax: tokens.code,
        }
    }

    /// The colors `theme`'s code is drawn in when it is the UI, syntax and
    /// preview theme at once.
    pub fn of_theme(theme: &ResolvedTheme) -> CodeColors {
        CodeColors::for_themes(theme, theme, theme)
    }
}

/// The code colors for a page drawn in `palette`: `active` unless it is made
/// for a dark background and the page is light, when `light` gives the
/// colors to use instead. Without any, the page's code isn't colored.
pub fn code_for_page(
    active: Option<CodeColors>,
    palette: &Palette,
    light: impl FnOnce() -> Option<CodeColors>,
) -> Option<CodeColors> {
    match active {
        Some(code) if code.dark && !palette.dark => light(),
        active => active,
    }
}

#[cfg(feature = "highlight")]
impl kanso_render::html::Highlighter for PageStyle {
    fn highlight(&self, lang: &str, code: &str) -> Option<Vec<kanso_render::html::CodeSpan>> {
        use kanso_render::html::CodeSpan;

        let colors = self.code.as_ref()?;
        let spans = crate::highlight::code_spans_direct(lang, code)?;
        Some(
            spans
                .into_iter()
                .filter_map(|span| {
                    let style = crate::theme::style(&span.tokens, &colors.syntax);
                    let bold = style.weight.is_some_and(|weight| weight >= 600);
                    let italic = style.italic == Some(true);
                    let color = style.color.map(|rgb| rgb.0);
                    (color.is_some() || bold || italic).then_some(CodeSpan {
                        start: span.range.start,
                        end: span.range.end,
                        color,
                        bold,
                        italic,
                    })
                })
                .collect(),
        )
    }
}

#[cfg(all(test, feature = "highlight"))]
mod tests {
    use std::sync::OnceLock;

    use super::{CodeColors, PageStyle, code_for_page};
    use crate::theme::Tokens;
    use crate::theme::builtin::BUILTIN;
    use crate::theme::file::{ResolvedTheme, ThemeFile, contrast};
    use kanso_render::Palette;
    use kanso_render::html::Highlighter as _;
    use proptest::prelude::*;

    /// The built-in themes, resolved, in their order.
    fn builtins() -> &'static [ResolvedTheme] {
        static ALL: OnceLock<Vec<ResolvedTheme>> = OnceLock::new();
        ALL.get_or_init(|| {
            BUILTIN
                .iter()
                .map(|(file, text)| ThemeFile::parse(text).expect(file).resolve())
                .collect()
        })
    }

    /// The built-in theme named `name`.
    fn builtin(name: &str) -> &'static ResolvedTheme {
        builtins()
            .iter()
            .find(|theme| theme.name == name)
            .unwrap_or_else(|| panic!("no built-in theme {name}"))
    }

    fn style() -> PageStyle {
        PageStyle {
            font: Some("Alegreya".into()),
            mono: Some("Iosevka".into()),
            size: 18.,
            code: Some(CodeColors::of_theme(builtin("Kanso Light"))),
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
        // Without colors, code stays plain.
        let plain = PageStyle {
            code: None,
            ..style
        };
        assert_eq!(plain.highlight("rust", "fn main() {}"), None);
        assert!(plain.options().highlight.is_none());
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

    /// A style for a page in `palette` while `active` is the theme in use
    /// and Kanso Light is the person's light theme.
    fn page_style(active: &str, palette: &Palette) -> PageStyle {
        PageStyle::for_palette(
            Some("Alegreya".into()),
            None,
            18.,
            Some(CodeColors::of_theme(builtin(active))),
            palette,
            || Some(CodeColors::of_theme(builtin("Kanso Light"))),
        )
    }

    /// The light page published while `active` is the theme in use: Kanso's
    /// paper under a dark theme, else the theme's own code background.
    fn light_page(active: &ResolvedTheme) -> Palette {
        if active.dark {
            return Palette::LIGHT;
        }
        let tokens = Tokens::fit(active, active, active);
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
                let ratio = contrast(
                    crate::theme::Rgb(color),
                    crate::theme::Rgb(palette.surface.0),
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
    // no-prop-test: a fixed list checked against every built-in theme.
    #[test]
    fn the_themes_under_three_to_one_are_the_ones_listed() {
        for theme in builtins() {
            let palette = light_page(theme);
            let (ratio, color) = worst(&page_style(&theme.name, &palette), &palette);
            let low = ratio < 3.;
            assert_eq!(
                low,
                BELOW_THREE_TO_ONE.contains(&theme.name.as_str()),
                "{}: {color:06x} reads {ratio:.2}:1",
                theme.name
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
            let own = CodeColors::of_theme(builtin(active));
            let chosen = code_for_page(Some(own.clone()), palette, || {
                panic!("the other theme's colors were asked for")
            });
            assert_eq!(chosen, Some(own));
        }
        // Without colors there is nothing to replace.
        assert_eq!(
            code_for_page(None, &Palette::LIGHT, || panic!("asked for")),
            None
        );
    }

    #[test]
    fn colors_follow_the_preview_theme_when_it_is_not_the_syntax_theme() {
        let (light, dark) = (builtin("Kanso Light"), builtin("Kanso Dark"));
        let own = CodeColors::for_themes(light, light, light);
        assert_eq!(own, CodeColors::of_theme(light));
        assert!(!own.dark);
        let previewed = CodeColors::for_themes(light, light, dark);
        assert!(previewed.dark);
        assert_ne!(previewed.syntax, own.syntax);
        assert_eq!(
            previewed.syntax,
            Tokens::fit(light, light, dark).code,
            "the preview's own colors, fitted to its code background"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// Invariant: whichever built-in theme is active, every color a
        /// light page's code gets reads on the page's code background, at
        /// 3:1 or better.
        #[test]
        fn prop_light_page_code_reads_whatever_theme_is_active(
            theme in 0..BUILTIN.len(),
            code in "(fn|let|struct|\"s\"|// c|[0-9]|::|[a-z]| |\\n|;){1,80}",
        ) {
            let active = &builtins()[theme];
            prop_assume!(!BELOW_THREE_TO_ONE.contains(&active.name.as_str()));
            let palette = light_page(active);
            let Some(spans) = page_style(&active.name, &palette).highlight("rust", &code) else {
                return Ok(());
            };
            for color in spans.into_iter().filter_map(|span| span.color) {
                let ratio = contrast(
                    crate::theme::Rgb(color),
                    crate::theme::Rgb(palette.surface.0),
                );
                prop_assert!(ratio >= 3., "{}: {color:06x} reads {ratio:.2}:1 in {code:?}", active.name);
            }
        }

        /// Invariant: every span a code block gets lies inside it, in order,
        /// and never splits a character.
        #[test]
        fn prop_spans_stay_inside_the_code(code in "(fn|let|struct|\"s\"|[0-9]| |\\n|;|[a-z]|é|日){0,120}") {
            let style = style();
            let Some(spans) = style.highlight("rust", &code) else {
                return Ok(());
            };
            let mut last = 0;
            for span in spans {
                prop_assert!(span.start < span.end);
                prop_assert!(span.end <= code.len());
                prop_assert!(code.is_char_boundary(span.start));
                prop_assert!(code.is_char_boundary(span.end));
                prop_assert!(span.start >= last);
                last = span.end;
            }
        }

        /// Invariant: a span is bold only when its style has a weight of 600
        /// or more, and every span has a color, a weight or a slant to show.
        #[test]
        fn prop_every_span_has_something_to_show(code in "(fn|let|struct|\"s\"|// c|[0-9]| |\\n|;|[a-z]){0,80}") {
            let style = PageStyle {
                code: Some(CodeColors::of_theme(builtin("Kanso Dark"))),
                ..PageStyle::default()
            };
            for span in style.highlight("rust", &code).unwrap_or_default() {
                prop_assert!(span.color.is_some() || span.bold || span.italic);
            }
        }
    }
}
