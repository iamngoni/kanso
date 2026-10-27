//! The faces, size and code colors an exported page sets text with: the
//! same ones the preview uses, so a page and the preview look alike. The
//! style is the engine's (`kanso_engine::export::PageStyle`), which the
//! mobile app fills from its own settings; this reads the desktop's.

use gpui_kit::App;
use gpui_kit::component::ActiveTheme as _;
use kanso_render::Palette;

pub use kanso_engine::export::PageStyle;

/// The style in use: the preview's size and face from Settings › Editor, the
/// theme's monospaced face, and the preview's code colors.
// no-prop-test: reads settings and globals; the engine's `PageStyle` has the
// properties, and the page snapshots show the result.
pub fn from_cx(cx: &mut App) -> PageStyle {
    let ty = crate::preview::preview_type(cx);
    let mono = Some(cx.theme().mono_font_family.to_string());
    let code = cx
        .try_global::<crate::themes::ActiveTheme>()
        .map(|active| active.page_code.clone());
    PageStyle {
        font: Some(ty.family.to_string()),
        mono,
        size: ty.size,
        code,
    }
}

/// [`from_cx`], with code colors that read on `palette`'s page: a light page
/// gets the person's light theme's colors when the active ones are made for
/// a dark background. Every page goes through here, so none sets pale code on
/// paper.
// no-prop-test: reads settings and globals; the engine's `code_for_page` has
// the properties.
pub fn for_palette(cx: &mut App, palette: &Palette) -> PageStyle {
    let style = from_cx(cx);
    PageStyle::for_palette(
        style.font,
        style.mono,
        style.size,
        style.code,
        palette,
        || Some(crate::themes::light_code(cx)),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui_kit::base::input::Rope;
    use gpui_kit::component::highlighter::{HighlightTheme, SyntaxHighlighter};
    use gpui_kit::{FontStyle, Hsla, Rgba};
    use kanso_engine::export::{CodeColors, PageStyle};
    use kanso_engine::strategies::markdown::{LANGUAGES, valid_code};
    use kanso_render::html::Highlighter as _;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestRunner};

    use crate::themes::{ThemeEntry, builtins, code_colors};

    /// What a byte of code is drawn as: its color as `0xRRGGBB`, whether it
    /// is bold, and whether it is italic. Plain text is `(None, false, false)`.
    type Look = (Option<u32>, bool, bool);

    /// An `Hsla` color as `0xRRGGBB`, the way the desktop's page highlighter
    /// made it: any translucency dropped, as a page's code sits on its own
    /// fill.
    fn rgb(color: Hsla) -> u32 {
        let rgba: Rgba = color.into();
        let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u32;
        (channel(rgba.r) << 16) | (channel(rgba.g) << 8) | channel(rgba.b)
    }

    /// The built-in theme named `name`.
    fn builtin(name: &str) -> Arc<ThemeEntry> {
        builtins()
            .iter()
            .find(|theme| theme.name() == name)
            .unwrap_or_else(|| panic!("no built-in theme {name}"))
            .clone()
    }

    /// How the component library drew `code` in `language`, byte by byte,
    /// when the preview's code blocks were drawn with `theme`: the
    /// highlighter the desktop's page used before the engine's, with the
    /// same mapping from a style to a look.
    fn desktop(language: &str, code: &str, theme: &HighlightTheme) -> Vec<Look> {
        crate::markdown_source::ensure_registered();
        crate::code_lang::ensure_registered();
        let grammar = crate::code_lang::grammar(language).expect("a grammar");
        let mut highlighter = SyntaxHighlighter::new(grammar);
        highlighter.update(None, &Rope::from_str(code), None);
        let mut looks = vec![(None, false, false); code.len()];
        for (range, style) in highlighter.styles(&(0..code.len()), theme) {
            let bold = style.font_weight.is_some_and(|weight| weight.0 >= 600.);
            let italic = style.font_style == Some(FontStyle::Italic);
            let color = style.color.map(rgb);
            if color.is_none() && !bold && !italic {
                continue;
            }
            for look in &mut looks[range] {
                *look = (color, bold, italic);
            }
        }
        looks
    }

    /// How the engine's page highlighter draws `code` in `language`, byte by
    /// byte, with `colors`.
    fn engine(language: &str, code: &str, colors: &CodeColors) -> Vec<Look> {
        let style = PageStyle {
            code: Some(colors.clone()),
            ..PageStyle::default()
        };
        let mut looks = vec![(None, false, false); code.len()];
        for span in style.highlight(language, code).expect("a grammar") {
            for look in &mut looks[span.start..span.end] {
                *look = (span.color, span.bold, span.italic);
            }
        }
        looks
    }

    /// `None` when both draw `code` alike, and otherwise where they first
    /// don't.
    fn disagreement(language: &str, code: &str, theme: &str, previewed: &str) -> Option<String> {
        let (ui, preview) = (builtin(theme), builtin(previewed));
        // The themes as `apply` fits them: the person's theme everywhere,
        // and `previewed` for the preview alone.
        let tokens =
            kanso_engine::theme::Tokens::fit(&ui.resolved, &ui.resolved, &preview.resolved);
        let highlight = crate::themes::code_highlight(&ui.resolved, &preview.resolved, &tokens)
            .expect("colors");
        let colors = CodeColors::for_themes(&ui.resolved, &ui.resolved, &preview.resolved);
        let (desktop, engine) = (
            desktop(language, code, &highlight),
            engine(language, code, &colors),
        );
        let at = desktop.iter().zip(&engine).position(|(a, b)| a != b)?;
        Some(format!(
            "{language} in {theme} (preview {previewed}), byte {at} of {code:?}: library {:?}, engine {:?}",
            desktop[at], engine[at],
        ))
    }

    /// A fixed set of themes: Kanso's two and a few of other makers, light
    /// and dark.
    const THEMES: [&str; 6] = [
        "Kanso Light",
        "Kanso Dark",
        "Nord",
        "Dracula",
        "Solarized Light",
        "Gruvbox Dark",
    ];

    /// Every language, every snippet of valid code in it, every theme above:
    /// a page colors code as the preview did, byte for byte, in color, weight
    /// and slant.
    ///
    /// This is what lets a page made by the engine (on the desktop and on the
    /// phone) look exactly as the desktop's pages did. They agree on every
    /// language Kanso highlights, for code that parses and for the first
    /// lines of it, which is code that doesn't. The library's highlighter
    /// also runs the injections of the language it was asked for (a script
    /// inside HTML, the inline Markdown inside Markdown); a fence in a note
    /// never did that in the engine, and a page shows what a fence shows, so
    /// no snippet of those two uses an injection.
    #[test]
    fn the_engine_colors_code_as_the_preview_did() {
        let mut failures = Vec::new();
        for theme in THEMES {
            for language in LANGUAGES {
                for code in valid_code(language) {
                    failures.extend(disagreement(language, code, theme, theme));
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    /// Agreeing on plain text would prove nothing: the snippets draw colors
    /// in every language, and the themes draw them differently.
    #[test]
    fn the_snippets_draw_colors_that_differ_between_themes() {
        for language in LANGUAGES {
            let code = valid_code(language)[0];
            let colored = |theme: &str| {
                let colors = CodeColors::of_theme(&builtin(theme).resolved);
                engine(language, code, &colors)
            };
            let (light, dark) = (colored("Kanso Light"), colored("Kanso Dark"));
            assert!(
                light.iter().any(|(color, _, _)| color.is_some()),
                "{language} has no color"
            );
            assert_ne!(light, dark, "{language} looks the same in both themes");
        }
    }

    /// The colors of the engine's `CodeColors` and the library's theme are
    /// fitted alike when the preview theme is not the syntax theme.
    #[test]
    fn a_preview_theme_of_its_own_colors_code_alike() {
        let mut failures = Vec::new();
        for (theme, previewed) in [
            ("Kanso Light", "Nord"),
            ("Kanso Dark", "Solarized Light"),
            ("Dracula", "Kanso Light"),
        ] {
            for language in ["rust", "python", "json", "css"] {
                for code in valid_code(language) {
                    failures.extend(disagreement(language, code, theme, previewed));
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    /// The same holds for the colors of any built-in theme, and for broken
    /// code (a snippet cut after some of its lines).
    #[test]
    fn the_engine_colors_code_as_the_preview_did_in_any_theme() {
        let mut runner = TestRunner::new(Config {
            cases: 96,
            failure_persistence: None,
            ..Config::default()
        });
        let strategy = (
            0..builtins().len(),
            proptest::sample::select(LANGUAGES.to_vec()),
            any::<prop::sample::Index>(),
            any::<prop::sample::Index>(),
        );
        runner
            .run(&strategy, |(theme, language, pick, cut)| {
                let snippets = valid_code(language);
                let snippet = snippets[pick.index(snippets.len())];
                let lines: Vec<&str> = snippet.split_inclusive('\n').collect();
                let code = lines[..1 + cut.index(lines.len())].concat();
                let name = builtins()[theme].name().to_owned();
                match disagreement(language, &code, &name, &name) {
                    None => Ok(()),
                    Some(why) => Err(TestCaseError::fail(why)),
                }
            })
            .unwrap();
    }

    /// The colors the engine builds for a theme in use are the colors the
    /// desktop's theme (and so its previews) were built from: the same
    /// styles for every token.
    #[test]
    fn code_colors_are_the_themes_own() {
        for theme in builtins() {
            let colors = CodeColors::of_theme(&theme.resolved);
            assert_eq!(colors.dark, theme.dark(), "{}", theme.name());
            assert!(code_colors(theme).is_some(), "{}", theme.name());
        }
    }

    /// A page's default text size is the app's.
    #[test]
    fn the_page_and_the_app_start_at_one_text_size() {
        assert_eq!(PageStyle::default().size, crate::config::DEFAULT_FONT_SIZE);
    }
}
