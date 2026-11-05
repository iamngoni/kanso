//! How the editor parses Markdown source for highlighting.
//!
//! The component library parses the inline Markdown of a note (emphasis,
//! code spans, links) as one combined document, putting back the text
//! between the paragraphs it keeps. A fenced code block between two such
//! paragraphs then reads as an inline code span, so its opening fence is
//! drawn in the code color, and a stray backtick in one list item pairs up
//! with one in the next. Kanso parses each paragraph on its own instead, as
//! Markdown does. The library's queries also leave some markers uncaptured,
//! and Kanso adds captures for them; those patches are the engine's
//! (`kanso_engine::highlight::queries`), so that the mobile app's
//! highlighter, which is the engine's own, reads the same queries.
//!
//! Separate parses are capped at 512 a note; past that, paragraphs keep
//! their block styles but lose inline ones. One note in the real library
//! (about 2,900 styled paragraphs) reaches it, and the combined parse cut
//! that note short too. Highlighting every note of that library took 3.9 s
//! this way against 4.9 s combined.

use std::sync::Once;

use gpui_kit::component::highlighter::LanguageRegistry;
use kanso_engine::highlight::queries;

/// Makes the editor's Markdown parse each paragraph's inline text on its own
/// and mute every marker. Only editors created afterwards pick it up; it does
/// nothing the second time.
pub fn ensure_registered() {
    static DONE: Once = Once::new();
    DONE.call_once(|| {
        let registry = LanguageRegistry::singleton();
        // no-prop-test: fixed edits to the library's queries, made by the
        // engine's functions; the tests below check what they highlight.
        if let Some(mut inline) = registry.language("markdown_inline") {
            inline.highlights = queries::markdown_inline_highlights(&inline.highlights).into();
            registry.register("markdown_inline", &inline);
        }
        let Some(mut config) = registry.language("markdown") else {
            return;
        };
        config.injections = queries::markdown_injections(&config.injections).into();
        config.highlights = queries::markdown_highlights(&config.highlights).into();
        registry.register("markdown", &config);
    });
}

#[cfg(test)]
mod tests {
    use super::ensure_registered;
    use crate::theme::{DARK, LIGHT, Rgb, theme_set};
    use gpui_kit::Hsla;
    use gpui_kit::base::input::Rope;
    use gpui_kit::component::highlighter::{HighlightTheme, LanguageRegistry, SyntaxHighlighter};
    use gpui_kit::component::{ThemeMode, ThemeSet};

    /// The color each byte of `source` is drawn in, with Kanso's theme.
    fn colors(source: &str, dark: bool) -> Vec<Option<Hsla>> {
        let set: ThemeSet = serde_json::from_value(theme_set(None)).unwrap();
        let theme = HighlightTheme {
            name: "Kanso".into(),
            appearance: if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            style: set.themes[usize::from(dark)].highlight.clone().unwrap(),
        };
        let mut highlighter = SyntaxHighlighter::new("markdown");
        highlighter.update(None, &Rope::from_str(source), None);
        let mut colors = vec![None; source.len()];
        for (range, style) in highlighter.styles(&(0..source.len()), &theme) {
            for color in &mut colors[range] {
                *color = style.color;
            }
        }
        colors
    }

    /// The colors of every byte of `needle`'s first match.
    fn colors_of(source: &str, colors: &[Option<Hsla>], needle: &str) -> Vec<Option<Hsla>> {
        let start = source.find(needle).unwrap();
        colors[start..start + needle.len()].to_vec()
    }

    /// A fenced block between paragraphs keeps its quiet fence and label,
    /// while inline code around it is still code, in both themes.
    #[test]
    fn fences_recede_and_code_spans_stay_code() {
        ensure_registered();
        let source = "*a*\n\n```json\n{}\n```\n\nx `y`";
        for (dark, c) in [(true, &DARK), (false, &LIGHT)] {
            let colors = colors(source, dark);
            let color = |rgb: Rgb| Some(Hsla::from(rgb));
            let quiet = [color(c.text_muted), color(c.text_secondary)];
            let fence = colors_of(source, &colors, "```json");
            assert!(fence.iter().all(|c| quiet.contains(c)), "{dark}: {fence:?}");
            assert!(!fence.contains(&color(c.code_text)), "{dark}");
            let muted = color(c.text_muted);
            let closing = &colors[source.rfind("```").unwrap()..][..3];
            assert!(closing.iter().all(|c| *c == muted), "{dark}: {closing:?}");
            assert_eq!(
                colors_of(source, &colors, "`y`"),
                vec![color(c.code_text); 3]
            );
            assert_eq!(colors_of(source, &colors, "*a*")[0], muted);
        }
    }

    /// Table pipes and delimiter rows, task boxes and the brackets around
    /// links, images and reference definitions recede like every other
    /// marker, while link text keeps the link color, in both themes.
    #[test]
    fn table_task_and_link_markers_recede() {
        ensure_registered();
        let source = "| a |\n|:--|\n| b |\n\n- [x] done\n- [ ] todo\n\n\
                      [t](u) ![i](p) [f][r] [s]\n\n[r]: https://e.x\n";
        for (dark, c) in [(true, &DARK), (false, &LIGHT)] {
            let colors = colors(source, dark);
            let muted = Some(Hsla::from(c.text_muted));
            let link = Some(Hsla::from(c.link));
            // `m` marks a muted byte, `l` a link-colored one, and `.` one in
            // the text color.
            let expect = |needle: &str, pattern: &str| {
                let expected: Vec<_> = pattern
                    .bytes()
                    .map(|b| match b {
                        b'm' => muted,
                        b'l' => link,
                        _ => None,
                    })
                    .collect();
                assert_eq!(
                    colors_of(source, &colors, needle),
                    expected,
                    "{dark}: {needle}"
                );
            };
            expect("| a |", "m...m");
            expect("|:--|", "mmmmm");
            expect("| b |", "m...m");
            expect("[x]", "mmm");
            expect("[ ]", "mmm");
            expect("[t](u)", "mlmmmm");
            expect("![i](p)", "mmlmmmm");
            expect("[f][r]", "mlmlll");
            expect("[s]", "mlm");
            expect("[r]: https://e.x", "mlmm.mmmmmmmmmmm");
        }
    }

    /// Backticks in separate list items don't pair up into one code span.
    #[test]
    fn code_spans_stay_inside_their_list_item() {
        ensure_registered();
        let source = "- `one\n- two` and\n- `three`\n";
        let colors = colors(source, true);
        let code = Some(Hsla::from(DARK.code_text));
        assert!(!colors_of(source, &colors, "one").contains(&code));
        assert!(!colors_of(source, &colors, "two").contains(&code));
        assert_eq!(colors_of(source, &colors, "`three`"), vec![code; 7]);
    }

    /// The edit takes: the library's query no longer combines inline text,
    /// which it would if the directive were spelled differently.
    #[test]
    fn inline_text_is_no_longer_combined() {
        ensure_registered();
        let config = LanguageRegistry::singleton().language("markdown").unwrap();
        assert!(config.injections.contains("markdown_inline"));
        assert!(!config.injections.contains("injection.combined"));
    }

    /// The engine's own highlighter (`kanso_engine::highlight`, which the
    /// mobile app uses) against the editor's: the same note colored the same
    /// way, byte for byte, with every token in a color of its own.
    ///
    /// They agree on notes in the domain below, and differ, on purpose, in
    /// these ways (each is asserted by `known_differences`):
    ///
    /// - A fence names its language by more spellings in the engine
    ///   (`code_lang::GRAMMARS`: `zsh`, `golang`, `jsx`, `svg`) and in any
    ///   case; the library knows `ktm`, `php3`, `phtml`, `markdown_inline` and
    ///   the like, and only in lower case. Notes in the domain name grammars
    ///   by their own names.
    /// - The library stops injecting after 512 separate parses a note, and
    ///   gives up on a parse that takes over 20 ms, so a long note or a slow
    ///   moment colors less. The engine has neither: its output depends on
    ///   the text alone. Notes in the domain have fewer than 512 paragraphs
    ///   and fences, and parse fast.
    /// - Broken code in a fence is colored by tree-sitter's recovery from the
    ///   errors, which depends on where in the text the code sits: the library
    ///   parses it in place, the engine as a text of its own (so that a
    ///   paragraph's colors don't depend on the rest of the note). Notes in
    ///   the domain hold valid code, unindented. (Valid code parses the same
    ///   anywhere.)
    /// - The engine parses a note in blocks between blank lines, and from
    ///   the first block with a syntax error on (a broken table, whose
    ///   recovery swallows what follows) as a whole, which gives what the
    ///   whole note would; the fixtures include such a table.
    mod parity {
        use super::*;
        use kanso_engine::highlight::{self, Token};
        use kanso_engine::strategies::markdown::{LANGUAGES, agreeing_notes, valid_code};
        use kanso_engine::theme::file::CAPTURES;
        use proptest::prelude::*;
        use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};

        /// A theme that colors each token its own color, and the colors.
        fn palette() -> (HighlightTheme, Vec<(Hsla, Token)>) {
            let hex = |token: Token| {
                let i = Token::ALL.iter().position(|t| *t == token).unwrap() as u32;
                format!("#{:02x}{:02x}{:02x}", 16 + i * 6, 40 + i * 5, 220 - i * 4)
            };
            let mut syntax = serde_json::Map::new();
            for (capture, key) in CAPTURES {
                let token = Token::from_key(key).unwrap();
                syntax.insert(
                    (*capture).to_owned(),
                    serde_json::json!({ "color": hex(token) }),
                );
            }
            let theme = HighlightTheme {
                name: "Parity".into(),
                appearance: ThemeMode::Dark,
                style: serde_json::from_value(serde_json::json!({ "syntax": syntax })).unwrap(),
            };
            let colors = Token::ALL
                .iter()
                .map(|token| {
                    let capture = CAPTURES
                        .iter()
                        .find(|(_, key)| *key == token.key())
                        .unwrap()
                        .0;
                    let color = theme.style.syntax.style(capture).unwrap().color.unwrap();
                    (color, *token)
                })
                .collect();
            (theme, colors)
        }

        /// The token each byte is drawn as by the editor's highlighter.
        fn desktop(
            source: &str,
            theme: &HighlightTheme,
            colors: &[(Hsla, Token)],
        ) -> Vec<Option<Token>> {
            ensure_registered();
            crate::code_lang::ensure_registered();
            // The library gives up on an injection parse after 20 ms, which
            // is plenty at rest and not under a loaded machine's full
            // parallel run: it would color less, and the two would
            // "disagree" about the machine and not the note. The engine has
            // no deadline, so none is the like-for-like comparison; a
            // minute only ends a parse that never would.
            gpui_kit::component::highlighter::set_injection_parse_timeout(Some(
                std::time::Duration::from_secs(60),
            ));
            let mut highlighter = SyntaxHighlighter::new("markdown");
            highlighter.update(None, &Rope::from_str(source), None);
            let mut tokens = vec![None; source.len()];
            for (range, style) in highlighter.styles(&(0..source.len()), theme) {
                let token = style
                    .color
                    .map(|c| colors.iter().find(|(color, _)| *color == c).unwrap().1);
                for byte in &mut tokens[range] {
                    *byte = token;
                }
            }
            tokens
        }

        /// The token each byte is drawn as by the engine's: the last of those
        /// over it, which wins.
        fn engine(source: &str) -> Vec<Option<Token>> {
            let mut tokens = vec![None; source.len()];
            for span in highlight::spans(source, 0..source.len()) {
                for byte in &mut tokens[span.range] {
                    *byte = span.tokens.last().copied();
                }
            }
            tokens
        }

        /// `None` when they agree, and otherwise where they first don't.
        fn disagreement(
            source: &str,
            theme: &HighlightTheme,
            colors: &[(Hsla, Token)],
        ) -> Option<String> {
            let (desktop, engine) = (desktop(source, theme, colors), engine(source));
            let at = desktop.iter().zip(&engine).position(|(a, b)| a != b)?;
            let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
            let line_end = source[at..].find('\n').map_or(source.len(), |i| at + i);
            Some(format!(
                "byte {at} of {:?}: editor {:?}, engine {:?}; in line {:?}",
                source,
                desktop[at],
                engine[at],
                &source[line_start..line_end],
            ))
        }

        /// Notes written to cover what Markdown has.
        fn fixtures() -> Vec<String> {
            let mut notes: Vec<String> = [
                "# Title\n\nA paragraph with *emphasis*, **strong**, `code`, a [link](https://e.x \"t\"), an ![image](p.png) and <https://auto.link>.\n",
                "Setext one\n==========\n\nSetext two\n----------\n\n---\n\n***\n\n___\n",
                "- one\n- two\n  - nested\n    - deeper\n\n1. first\n2. second\n\n- [ ] todo\n- [x] done\n",
                "> quote\n> continues *here*\n>\n> > nested\n\nafter the quote\n",
                "| a | b |\n|:--|--:|\n| c | [d](e) |\n| *f* | `g` |\n\nafter\n",
                "[ref]: https://e.x \"Title\"\n\nSee [ref], [the ref][ref] and [ref][].\n",
                "---\ntitle: A note\ntags: [a, b]\n---\n\n# Body\n",
                "+++\ntitle = \"toml\"\n+++\n\n# Body\n",
                "```rust\nfn main() {\n    let x = \"a\";\n}\n```\n\ntext after\n",
                "~~~python\ndef f(x):\n    return x\n~~~\n\n```\nno language\n```\n",
                "```unknown\nfn main() {}\n```\n",
                "<div>\nhtml *block*\n</div>\n\n<!-- a\n\ncomment -->\n\n<script>\nvar a = 1;\n</script>\n",
                "Escapes: \\* \\_ \\` \\# and a hard break\\\nnext line.\n",
                "日本語の *強調* と **太字**、`コード`、[リンク](https://e.x)。🙂 emoji *🙂*.\n",
                "    indented code\n\nparagraph\n\n    more code\n",
                "A line with a `\n\nand a * lone marker\n\n[unclosed link\n",
                "- a `b\n- c` d\n- `e`\n",
                "# Heading with *emphasis* and [link](u)\n\n## Second\n\n###### Sixth\n",
                "Footnote-like [^1] and a tag #tag and a wiki-link [[Other note]].\n",
                "1. one\n\n   continued paragraph\n\n   ```bash\n   echo hi\n   ```\n\n2. two\n",
                "```rust\n/// Docs.\nfn f(&self) -> Option<&'a str> {\n    let s = \"a\\n\";\n    'outer: loop { break 'outer; }\n    Some(s)\n}\n```\n\n```javascript\nconst r = /a+/g;\nconst s = `x${a}y`;\nlabel: for (;;) { continue label; }\n```\n",
                "intro\n\n|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |\n\n\nafter *x*\n\nlast `y`\n",
                "",
                "just text without a line ending",
            ]
            .iter()
            .map(|note| (*note).to_owned())
            .collect();
            for language in LANGUAGES {
                for code in valid_code(language) {
                    notes.push(format!(
                        "Before *it*.\n\n```{language}\n{code}```\n\nAfter `it`.\n"
                    ));
                }
            }
            notes
        }

        #[test]
        fn the_editor_and_the_engine_color_fixture_notes_alike() {
            let (theme, colors) = palette();
            let notes = fixtures();
            let failures: Vec<String> = notes
                .iter()
                .filter_map(|note| disagreement(note, &theme, &colors))
                .collect();
            assert!(failures.is_empty(), "{failures:#?}");
            // Agreeing on nothing would prove nothing: the notes draw most
            // of the tokens with the editor's highlighter.
            let seen: std::collections::BTreeSet<Token> = notes
                .iter()
                .flat_map(|note| desktop(note, &theme, &colors))
                .flatten()
                .collect();
            assert!(seen.len() >= 30, "the notes draw only {seen:?}");
        }

        /// The library's injection query read any block after a leading
        /// rule and before another as YAML, not only the one between them
        /// (`(thematic_break) (_) (thematic_break)` skips siblings), and the
        /// two highlighters disagreed on which blocks got it: a paragraph
        /// with a `[text][r]` reference link without a definition, or a
        /// `> [[wiki]]` quote, was drawn as a YAML flow sequence by one.
        /// Neither is YAML in Markdown; now only a block that touches both
        /// rules is read that way.
        #[test]
        fn blocks_that_are_not_between_two_rules_are_not_yaml() {
            let (theme, colors) = palette();
            for note in [
                "---\n\n\n- a\n\n- a\n\n\n\n`日本語` 日本語 a\n<https://a.b> [[wiki]] [aa](aa)\n日本語 aaa\n\n\n> [[]]\n***\n",
                "---\n\n\n- a\n\n- a\n\n\n\n`日本語` 日本語 a\n<https://a.b> [[wiki]] [aa](aa)\n日本語 aaa\n\n\n> [[wiki]]\n\n\n***\n\n\n",
                "[ddycff][r] [63](jxoey) #tag",
                "[ddycff][r] [63](jxoey) #tag\n",
                "---\n\n[ddycff][r] [63](jxoey) #tag\n\n---\n",
            ] {
                assert_eq!(disagreement(note, &theme, &colors), None, "{note:?}");
            }
        }

        #[test]
        fn the_editor_and_the_engine_color_generated_notes_alike() {
            let (theme, colors) = palette();
            let mut runner = TestRunner::new(Config {
                // 160 unless PROPTEST_CASES says more.
                cases: std::env::var("PROPTEST_CASES")
                    .ok()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(160),
                failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/proptest-regressions/markdown_source.txt"
                )))),
                ..Config::default()
            });
            runner
                .run(&agreeing_notes(), |note| {
                    match disagreement(&note, &theme, &colors) {
                        None => Ok(()),
                        Some(why) => Err(TestCaseError::fail(why)),
                    }
                })
                .unwrap();
        }

        /// The differences listed above exist: where they stop, the list does
        /// too.
        #[test]
        fn known_differences() {
            let (theme, colors) = palette();
            let at = |source: &str, needle: &str| source.find(needle).unwrap();
            // Spellings only the engine knows, and the case of one both do:
            // the editor leaves the code plain.
            for (info, code) in [
                ("zsh", "echo \"$A\""),
                ("RUST", "fn echo() {}"),
                ("golang", "package main"),
                ("jsx", "const a = 1;"),
            ] {
                let note = format!("```{info}\n{code}\n```\n");
                let (editor, engine) = (desktop(&note, &theme, &colors), engine(&note));
                let code = at(&note, code);
                assert_eq!(editor[code], Some(Token::CodeBlock), "{info}");
                assert_ne!(engine[code], Some(Token::CodeBlock), "{info}");
            }
            // Past 512 parses the editor stops coloring inside paragraphs.
            let note: String = (0..600).map(|i| format!("para *{i}*\n\n")).collect();
            let last = note.rfind("*5").unwrap() + 1;
            assert_eq!(engine(&note)[last], Some(Token::Emphasis));
            assert_eq!(desktop(&note, &theme, &colors)[last], None);
        }

        /// The engine's patched copies of the library's Markdown queries are
        /// the ones the registry holds after this module's patching: a
        /// library update that changes a query fails here instead of
        /// drifting.
        #[test]
        fn the_engines_queries_are_the_registrys() {
            ensure_registered();
            let registry = LanguageRegistry::singleton();
            let engine = kanso_engine::highlight::queries::in_use();
            let markdown = registry.language("markdown").unwrap();
            let inline = registry.language("markdown_inline").unwrap();
            assert_eq!(markdown.highlights.as_ref(), engine.highlights);
            assert_eq!(markdown.injections.as_ref(), engine.injections);
            assert_eq!(inline.highlights.as_ref(), engine.inline_highlights);
        }
    }
}
