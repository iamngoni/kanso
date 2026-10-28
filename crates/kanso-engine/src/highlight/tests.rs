use std::collections::BTreeMap;
use std::ops::Range;

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use super::{Highlighter, Span, Token, blocks, spans, spans_uncapped, whole_document_spans};
use crate::strategies::markdown::{calm_notes, notes};
use crate::theme::file::{CAPTURES, SYNTAX_KEYS, TokenStyle};
use crate::theme::{Rgb, style};

fn config() -> ProptestConfig {
    // 256 cases, or `PROPTEST_CASES`.
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A range of `text` between two of its character boundaries.
fn range_of(text: &str, a: prop::sample::Index, b: prop::sample::Index) -> Range<usize> {
    let bounds: Vec<usize> = text
        .char_indices()
        .map(|(i, _)| i)
        .chain([text.len()])
        .collect();
    let (a, b) = (bounds[a.index(bounds.len())], bounds[b.index(bounds.len())]);
    a.min(b)..a.max(b)
}

/// What differs first between two lists of spans, for a failure message.
fn difference(text: &str, cut: &Spans, whole: &Spans) -> String {
    let at = cut.iter().zip(whole.iter()).position(|(a, b)| a != b);
    let at = at.unwrap_or(cut.len().min(whole.len()));
    format!(
        "{} spans cut, {} whole; first difference at span {at}: {:?} against {:?}; text {:?}; blocks {:?}",
        cut.len(),
        whole.len(),
        cut.get(at),
        whole.get(at),
        text,
        blocks::cut(text),
    )
}

/// Notes' broken parts: a table the grammar's recovery swallows what
/// follows into, and a fence that is never closed.
const BROKEN: [&str; 3] = [
    "|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |",
    "```rust\nfn broken() {",
    "| a | b |\n| - |\n| **x** |\n|",
];

type Spans = [Span];

fn of<'a>(text: &'a str, spans: &'a [Span]) -> Vec<(&'a str, Vec<Token>)> {
    spans
        .iter()
        .map(|s| (&text[s.range.clone()], s.tokens.clone()))
        .collect()
}

/// The last token a span of `text` has over the text `needle` (its first
/// match), or none when no span covers its first byte.
fn token_at(text: &str, needle: &str) -> Option<Token> {
    let at = text.find(needle).unwrap();
    spans(text, 0..text.len())
        .into_iter()
        .find(|s| s.range.contains(&at))
        .and_then(|s| s.tokens.last().copied())
}

/// One edit of a note: where, how much it deletes and what it inserts.
fn edit() -> impl Strategy<Value = (prop::sample::Index, usize, String)> {
    let insert = prop_oneof![
        3 => proptest::sample::select(vec![
            "\n", "\n\n", " ", "a", "é", "日", "🙂", "```", "~~~", "- ", "# ", "> ", "<!--", "-->",
            "---", "  ", "*", "`", "[", "](", "<script>", "</script>", "|", "\n\n```rust\n",
            "\n\n  ```\n",
        ])
        .prop_map(str::to_owned),
        1 => "[ -~\n]{0,12}",
    ];
    (any::<prop::sample::Index>(), 0usize..6, insert)
}

/// Applies an edit: delete `delete` characters from a character boundary,
/// then insert there. Returns the byte range the inserted text now holds.
fn apply(
    text: &mut String,
    (at, delete, insert): &(prop::sample::Index, usize, String),
) -> Range<usize> {
    let bounds: Vec<usize> = text
        .char_indices()
        .map(|(i, _)| i)
        .chain([text.len()])
        .collect();
    let start = bounds[at.index(bounds.len())];
    let end = text[start..]
        .char_indices()
        .nth(*delete)
        .map_or(text.len(), |(i, _)| start + i);
    text.replace_range(start..end, insert);
    start..start + insert.len()
}

proptest! {
    #![proptest_config(config())]

    /// H1: spans lie inside the range, on character boundaries, sorted and
    /// apart, and none is empty or without a token.
    #[test]
    fn prop_h1_spans_are_in_bounds_and_ordered(
        text in notes(true),
        a in any::<prop::sample::Index>(),
        b in any::<prop::sample::Index>(),
    ) {
        let range = range_of(&text, a, b);
        let spans = spans(&text, range.clone());
        let mut end = range.start;
        for span in &spans {
            prop_assert!(!span.range.is_empty(), "{span:?}");
            prop_assert!(!span.tokens.is_empty(), "{span:?}");
            prop_assert!(span.range.start >= end, "{span:?} after {end}");
            prop_assert!(span.range.end <= range.end, "{span:?} in {range:?}");
            prop_assert!(text.is_char_boundary(span.range.start));
            prop_assert!(text.is_char_boundary(span.range.end));
            end = span.range.end;
        }
    }

    /// H2: one highlighter lives through a run of edits, with a cache so
    /// small that blocks fall out, and always returns what the full
    /// recompute does.
    #[test]
    fn prop_h2_cached_spans_equal_the_full_recompute(
        mut text in notes(true),
        edits in prop::collection::vec(edit(), 1..8),
        probes in prop::collection::vec((any::<prop::sample::Index>(), any::<prop::sample::Index>()), 1..4),
        capacity in prop_oneof![Just(0usize), Just(600), Just(4000), Just(2 * 1024 * 1024)],
    ) {
        let mut highlighter = Highlighter::with_capacity(capacity);
        for edit in &edits {
            apply(&mut text, edit);
            for (a, b) in &probes {
                let range = range_of(&text, *a, *b);
                prop_assert_eq!(
                    highlighter.spans(&text, range.clone()),
                    spans(&text, range),
                );
            }
            let whole = 0..text.len();
            prop_assert_eq!(highlighter.spans(&text, whole.clone()), spans(&text, whole));
        }
    }

    /// H3: split a range at a point, and the two halves, joined at the seam,
    /// are the whole.
    #[test]
    fn prop_h3_ranges_split_and_join(
        text in notes(true),
        a in any::<prop::sample::Index>(),
        b in any::<prop::sample::Index>(),
        c in any::<prop::sample::Index>(),
    ) {
        let mut cuts = [range_of(&text, a, a).start, range_of(&text, b, b).start, range_of(&text, c, c).start];
        cuts.sort_unstable();
        let [a, b, c] = cuts;
        let mut left = spans(&text, a..b);
        let right = spans(&text, b..c);
        if let (Some(last), Some(first)) = (left.last_mut(), right.first())
            && last.range.end == first.range.start
            && last.tokens == first.tokens
        {
            last.range.end = first.range.end;
            left.extend(right.into_iter().skip(1));
        } else {
            left.extend(right);
        }
        prop_assert_eq!(left, spans(&text, a..c));
    }

    /// H4: cutting a note into blocks gives the spans one parse of the whole
    /// note does.
    #[test]
    fn prop_h4_blocks_equal_the_whole_parse(text in notes(true)) {
        let (cut, whole) = (spans(&text, 0..text.len()), whole_document_spans(&text));
        prop_assert!(cut == whole, "{}", difference(&text, &cut, &whole));
    }

    /// H4, on notes edited at random: the edits make the unusual shapes (a
    /// fence opened late, a comment closed twice) the generator doesn't.
    #[test]
    fn prop_h4_holds_after_edits(
        mut text in notes(true),
        edits in prop::collection::vec(edit(), 1..10),
    ) {
        for edit in &edits {
            apply(&mut text, edit);
        }
        let (cut, whole) = (spans(&text, 0..text.len()), whole_document_spans(&text));
        prop_assert!(cut == whole, "{}", difference(&text, &cut, &whole));
    }

    /// H6: a range of a note with a syntax error in it gets the spans it has
    /// when the error's parse goes on to the end of the note, though it
    /// stops short of it.
    #[test]
    fn prop_h6_a_cut_short_error_parse_gives_the_spans_of_the_whole_error_parse(
        before in notes(true),
        broken in proptest::sample::select(BROKEN.to_vec()),
        after in notes(true),
        edits in prop::collection::vec(edit(), 0..4),
        probes in prop::collection::vec((any::<prop::sample::Index>(), any::<prop::sample::Index>()), 1..5),
        capacity in prop_oneof![Just(0usize), Just(4000), Just(2 * 1024 * 1024)],
    ) {
        let mut text = format!("{before}\n\n{broken}\n\n{after}");
        for edit in &edits {
            apply(&mut text, edit);
        }
        let mut highlighter = Highlighter::with_capacity(capacity);
        for (a, b) in &probes {
            let range = range_of(&text, *a, *b);
            let expected = spans_uncapped(&text, range.clone());
            prop_assert_eq!(spans(&text, range.clone()), expected.clone(), "range {:?} of {:?}", range, text);
            prop_assert_eq!(highlighter.spans(&text, range.clone()), expected, "cached, range {:?} of {:?}", range, text);
        }
    }

    /// The blocks cover the note, one after another.
    #[test]
    fn prop_blocks_partition_the_note(text in notes(true)) {
        let blocks = blocks::cut(&text);
        let mut end = 0;
        for block in &blocks {
            prop_assert_eq!(block.start, end);
            prop_assert!(block.end > block.start);
            prop_assert!(text.is_char_boundary(block.end));
            end = block.end;
        }
        prop_assert_eq!(end, text.len());
    }

    /// H5: a block is parsed when its text is new and not otherwise: the
    /// cache neither misses what it holds nor holds what it parses twice.
    /// (With a syntax error somewhere the rest of the note after it is one
    /// block, which is new each time; those notes are left out.)
    #[test]
    fn prop_h5_only_new_blocks_are_parsed(
        mut text in notes(true),
        edits in prop::collection::vec(edit(), 1..4),
    ) {
        let mut highlighter = Highlighter::new();
        let mut known: std::collections::HashSet<(String, bool)> = std::collections::HashSet::new();
        let blocks_of = |text: &str| -> std::collections::HashSet<(String, bool)> {
            blocks::cut(text)
                .into_iter()
                .map(|b| (text[b.clone()].to_owned(), b.start == 0))
                .collect()
        };
        let clean = |text: &str| {
            let mut parsers = super::block::Parsers::default();
            blocks::cut(text)
                .into_iter()
                .all(|b| super::block::is_clean(&mut parsers, &text[b.clone()], b.start == 0))
        };
        for edit in &edits {
            prop_assume!(clean(&text));
            highlighter.spans(&text, 0..text.len());
            known.extend(blocks_of(&text));
            let parsed = highlighter.blocks_parsed();
            apply(&mut text, edit);
            prop_assume!(clean(&text));
            highlighter.spans(&text, 0..text.len());
            let fresh = blocks_of(&text).difference(&known).count();
            prop_assert_eq!(highlighter.blocks_parsed() - parsed, fresh);
        }
    }

    /// H5, locality: in a note without fences and the like, typing a few
    /// characters (which may split a paragraph or join two) parses the
    /// blocks the typing is in, and at most two more.
    #[test]
    fn prop_h5_typing_parses_few_blocks(
        mut text in calm_notes(),
        at in any::<prop::sample::Index>(),
        typed in proptest::sample::select(vec!["a", "é", "日", "🙂", " ", "\n", "\n\n", "# ", "- ", "> ", "*"]),
    ) {
        let mut highlighter = Highlighter::new();
        highlighter.spans(&text, 0..text.len());
        let before = highlighter.blocks_parsed();
        let inserted = apply(&mut text, &(at, 0, typed.to_owned()));
        highlighter.spans(&text, 0..text.len());
        let touched = blocks::cut(&text)
            .into_iter()
            .filter(|b| b.start <= inserted.end && b.end >= inserted.start)
            .count();
        let parsed = highlighter.blocks_parsed() - before;
        prop_assert!(parsed <= touched + 2, "{parsed} parsed, {touched} touched");
    }
}

#[test]
#[cfg(feature = "code-grammars")]
fn headings_fences_tasks_and_front_matter_get_their_tokens() {
    let text = "# Title\n\n- [ ] todo\n\n```rust\nfn main() { let x = \"a\"; }\n```\n";
    let spans = spans(text, 0..text.len());
    let tokens = |needle: &str| -> Vec<Token> {
        let at = text.find(needle).unwrap();
        spans
            .iter()
            .find(|s| s.range.contains(&at))
            .map(|s| s.tokens.clone())
            .unwrap_or_default()
    };
    assert_eq!(tokens("#"), [Token::BlockMarker]);
    assert_eq!(tokens("Title"), [Token::Heading]);
    assert_eq!(tokens("[ ]").last(), Some(&Token::Task));
    assert_eq!(tokens("fn").last(), Some(&Token::Keyword));
    assert_eq!(tokens("\"a\"").last(), Some(&Token::String));
    assert_eq!(tokens("main").last(), Some(&Token::Function));
    // The fence's marker and info string recede like other markers.
    assert_eq!(tokens("```rust").last(), Some(&Token::Marker));

    // Front matter is drawn as front matter, with its YAML's own colors over it.
    let front = "---\ntitle: a\n---\n\ntext";
    let at = front.find("title").unwrap();
    let inside = super::spans(front, 0..front.len())
        .into_iter()
        .find(|s| s.range.contains(&at))
        .unwrap();
    assert_eq!(inside.tokens, [Token::FrontMatter, Token::Property]);
    // `---` after the first line is a rule, not front matter.
    let rule = "text\n\n---\n\nmore";
    assert_eq!(token_at(rule, "---"), Some(Token::Hr));
}

#[test]
fn inline_markers_and_links_are_spans() {
    let text = "a *b* `c` [d](e) <https://f.g> \\*\n";
    let spans = spans(text, 0..text.len());
    let pieces = of(text, &spans);
    let has = |piece: &str, token: Token| {
        pieces
            .iter()
            .any(|(t, tokens)| *t == piece && tokens.last() == Some(&token))
    };
    // The delimiters recede inside the emphasis.
    assert!(
        pieces.contains(&("*", vec![Token::Emphasis, Token::Marker])),
        "{pieces:?}"
    );
    assert!(pieces.contains(&("b", vec![Token::Emphasis])), "{pieces:?}");
    assert!(has("`c`", Token::Code), "{pieces:?}");
    assert!(has("d", Token::Link), "{pieces:?}");
    assert!(has("e", Token::Url), "{pieces:?}");
}

#[test]
fn a_range_gets_the_spans_inside_it() {
    let text = "# One\n\ntext *two*\n\n# Three\n";
    let all = spans(text, 0..text.len());
    let start = text.find("text").unwrap();
    let end = start + "text *two*".len();
    let part = spans(text, start..end);
    assert!(!part.is_empty());
    assert!(
        part.iter()
            .all(|s| s.range.start >= start && s.range.end <= end)
    );
    // The part is what the whole has there.
    let clipped: Vec<_> = all
        .iter()
        .filter(|s| s.range.end > start && s.range.start < end)
        .map(|s| {
            (
                s.range.start.max(start)..s.range.end.min(end),
                s.tokens.clone(),
            )
        })
        .collect();
    let got: Vec<_> = part
        .iter()
        .map(|s| (s.range.clone(), s.tokens.clone()))
        .collect();
    assert_eq!(got, clipped);
    // An empty or backwards range, and one past the end, give nothing.
    assert!(spans(text, 3..3).is_empty());
    assert!(spans(text, 100..200).is_empty());
    assert!(spans("", 0..10).is_empty());
}

#[test]
fn a_range_inside_a_character_is_widened() {
    let text = "# 日本語\n";
    assert_eq!(
        spans(text, 3..4),
        spans(text, 2..text.char_indices().nth(3).unwrap().0)
    );
}

#[test]
fn typing_in_one_paragraph_parses_one_block() {
    let paragraphs: Vec<String> = (0..40)
        .map(|i| format!("paragraph *{i}* with `code`"))
        .collect();
    let mut text = paragraphs.join("\n\n");
    let mut highlighter = Highlighter::new();
    highlighter.spans(&text, 0..text.len());
    assert_eq!(highlighter.blocks_parsed(), 40);
    // Typing in the middle of the note.
    let at = text.find("paragraph *20*").unwrap() + 3;
    text.insert(at, 'x');
    highlighter.spans(&text, 0..text.len());
    assert_eq!(highlighter.blocks_parsed(), 41);
    // Nothing changed: nothing to parse.
    highlighter.spans(&text, 0..text.len());
    assert_eq!(highlighter.blocks_parsed(), 41);
    // Typing at the top moves every paragraph below it, and none is parsed.
    text.insert(0, 'y');
    highlighter.spans(&text, 0..text.len());
    assert_eq!(highlighter.blocks_parsed(), 42);
    // Only what is on screen is looked at.
    let mut fresh = Highlighter::new();
    let start = text.find("paragraph *30*").unwrap();
    fresh.spans(&text, start..start + 10);
    assert_eq!(fresh.blocks_parsed(), 1);
}

#[test]
fn a_block_that_follows_another_reads_front_matter_as_a_rule() {
    // The same text after a blank line is a rule, a paragraph, a rule.
    let after = "x\n\n---\nfoo\n\n---\n";
    let at = after.find("---").unwrap();
    assert_eq!(token_at(after, "---"), Some(Token::Hr), "{at}");
    assert_eq!(spans(after, 0..after.len()), whole_document_spans(after));
}

#[test]
#[cfg(feature = "code-grammars")]
fn every_grammar_has_a_query_that_compiles() {
    use super::{GRAMMARS, registry};
    for names in GRAMMARS {
        let grammar = registry::get(names[0]).unwrap_or_else(|| panic!("{} missing", names[0]));
        assert!(
            grammar.highlights().is_some(),
            "{}'s highlight query does not compile",
            names[0]
        );
    }
    for name in ["markdown", "markdown_inline"] {
        assert!(
            registry::get(name).unwrap().highlights().is_some(),
            "{name}"
        );
    }
    assert!(registry::get("markdown").unwrap().injections().is_some());
    assert_eq!(registry::all().len(), GRAMMARS.len() + 1);
}

#[test]
#[cfg(feature = "code-grammars")]
fn every_grammar_colors_code() {
    use super::GRAMMARS;
    let samples = [
        ("bash", "if [ -f a ]; then echo \"$A\"; fi"),
        ("c", "int main() { return 0; }"),
        ("cpp", "class A { int b; };"),
        ("csharp", "public class A { string B = \"x\"; }"),
        ("css", "a { color: red; }"),
        ("diff", "@@ -1 +1 @@\n-a\n+b"),
        ("go", "func main() { x := 1 }"),
        ("html", "<div class=\"a\">b</div>"),
        ("java", "public class A { int b = 1; }"),
        ("javascript", "const a = () => 1;"),
        ("json", "{\"a\": [1, true]}"),
        ("kotlin", "fun main() { val a = 1 }"),
        ("lua", "local a = function() end"),
        ("markdown", "# h"),
        ("php", "<?php function f() { return 1; }"),
        ("python", "def f(x):\n    return 1"),
        ("ruby", "def f; 1; end"),
        ("rust", "fn main() { let x = 1; }"),
        ("sql", "select a from b where c = 1;"),
        ("swift", "struct A { let b = \"x\" }"),
        ("toml", "[a]\nb = \"c\""),
        ("tsx", "interface A { b: string }"),
        ("typescript", "let a: number = 1;"),
        ("yaml", "a:\n  - b"),
    ];
    assert_eq!(samples.len(), GRAMMARS.len());
    for (language, code) in samples {
        let text = format!("```{language}\n{code}\n```\n");
        let inside = text.find(code).unwrap()..text.find(code).unwrap() + code.len();
        let colored = spans(&text, 0..text.len())
            .into_iter()
            .filter(|s| s.range.start >= inside.start && s.range.end <= inside.end)
            .count();
        assert!(colored >= 1, "{language}: no span in {code:?}");
    }
}

#[test]
#[cfg(feature = "code-grammars")]
fn unknown_and_aliased_fences() {
    let rust = "```rust\nfn a() {}\n```\n";
    let alias = "```rs,ignore\nfn a() {}\n```\n";
    let unknown = "```dart\nfn a() {}\n```\n";
    let inside = |text: &str| {
        let start = text.find("fn").unwrap();
        spans(text, start..start + 2)
    };
    assert_eq!(inside(rust)[0].tokens.last(), Some(&Token::Keyword));
    assert_eq!(inside(alias)[0].tokens.last(), Some(&Token::Keyword));
    // A language Kanso has no grammar for: a plain code block.
    assert_eq!(inside(unknown)[0].tokens, [Token::CodeBlock]);
}

#[test]
fn tokens_and_theme_keys_agree() {
    let keys: Vec<&str> = Token::ALL.iter().map(|t| t.key()).collect();
    let expected: Vec<&str> = SYNTAX_KEYS
        .iter()
        .map(|(key, _)| *key)
        .filter(|key| *key != "text")
        .collect();
    assert_eq!(keys, expected);
    for token in Token::ALL {
        assert_eq!(Token::from_key(token.key()), Some(token));
    }
    assert_eq!(Token::from_key("text"), None);
    for (capture, key) in CAPTURES {
        assert_eq!(
            Token::from_capture(capture).map(Token::key),
            Some(*key),
            "{capture}"
        );
    }
}

#[test]
fn captures_fall_back_to_the_part_before_the_dot() {
    assert_eq!(
        Token::from_capture("keyword.modifier"),
        Some(Token::Keyword)
    );
    assert_eq!(
        Token::from_capture("function.builtin"),
        Some(Token::Function)
    );
    assert_eq!(
        Token::from_capture("variable.parameter"),
        Some(Token::Variable)
    );
    // The name itself first.
    assert_eq!(
        Token::from_capture("variable.special"),
        Some(Token::VariableSpecial)
    );
    assert_eq!(
        Token::from_capture("string.special.key"),
        Some(Token::String)
    );
    // No theme key colors these.
    for none in [
        "none",
        "hint",
        "predictive",
        "local.scope",
        "text.uri",
        "",
        "_x",
    ] {
        assert_eq!(Token::from_capture(none), None, "{none}");
    }
}

#[test]
fn a_later_token_wins_attribute_by_attribute() {
    let mut syntax: BTreeMap<&'static str, TokenStyle> = BTreeMap::new();
    syntax.insert(
        "code_block",
        TokenStyle {
            color: Some(Rgb(1)),
            italic: Some(true),
            weight: None,
        },
    );
    syntax.insert("keyword", TokenStyle::color(Rgb(2)));
    syntax.insert(
        "strong",
        TokenStyle {
            color: None,
            italic: None,
            weight: Some(700),
        },
    );
    let style = |tokens: &[Token]| style(tokens, &syntax);
    assert_eq!(style(&[]), TokenStyle::default());
    assert_eq!(style(&[Token::Keyword]), TokenStyle::color(Rgb(2)));
    // The keyword's color over the block's, the block's slant kept.
    assert_eq!(
        style(&[Token::CodeBlock, Token::Keyword]),
        TokenStyle {
            color: Some(Rgb(2)),
            italic: Some(true),
            weight: None
        }
    );
    // The other way round, the block's color wins.
    assert_eq!(
        style(&[Token::Keyword, Token::CodeBlock]).color,
        Some(Rgb(1))
    );
    // A token the theme says nothing about changes nothing.
    assert_eq!(
        style(&[Token::Keyword, Token::Tag]),
        TokenStyle::color(Rgb(2))
    );
    assert_eq!(
        style(&[Token::Keyword, Token::Strong]),
        TokenStyle {
            color: Some(Rgb(2)),
            italic: None,
            weight: Some(700)
        }
    );
}

#[test]
fn a_syntax_error_makes_the_rest_of_the_note_one_block() {
    // A broken table: the grammar's recovery swallows what follows it.
    let text = "intro\n\n|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |\n\n\nafter *x*\n\nlast `y`\n";
    let mut parsers = super::block::Parsers::default();
    let broken = text.find("|\n 日本語").unwrap();
    assert!(!super::block::is_clean(
        &mut parsers,
        &text[broken..],
        false
    ));

    assert_eq!(spans(text, 0..text.len()), whole_document_spans(text));
    let mut highlighter = Highlighter::new();
    assert_eq!(
        highlighter.spans(text, 0..text.len()),
        spans(text, 0..text.len())
    );
    // The first block, and the broken one with all that follows it.
    assert_eq!(highlighter.blocks_parsed(), 2);
    // A range further down is read from the broken block's parse again: it
    // is made for the range and not kept (it is one block more parsed).
    let after = text.find("last").unwrap();
    assert_eq!(
        highlighter.spans(text, after..text.len()),
        spans(text, after..text.len())
    );
    assert_eq!(highlighter.blocks_parsed(), 3);
}

// no-prop-test: the equality with the uncapped parse is property H6; this is
// the cost it buys, on a fixed note.
#[test]
fn an_error_early_in_a_note_is_not_parsed_to_its_end_for_a_range_near_it() {
    let broken = "|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |\n\n\n";
    let mut text = format!("intro\n\n{broken}");
    for n in 0..200 {
        text.push_str(&format!("Paragraph {n} with *emphasis*.\n\n"));
    }

    // A range that is most of the parse that is made for it: the parse is
    // kept, and typing past its end does not make it again.
    let near = 0..text.find("Paragraph 3 ").unwrap();
    let mut highlighter = Highlighter::new();
    let first = highlighter.spans(&text, near.clone());
    assert_eq!(first, spans_uncapped(&text, near.clone()));
    let parsed = highlighter.blocks_parsed();
    let end = text.len();
    text.insert(end - 2, 'x');
    assert_eq!(highlighter.spans(&text, near.clone()), first);
    assert_eq!(highlighter.blocks_parsed(), parsed);
    // The cache holds the cut parse, not the rest of the note.
    assert!(highlighter.bytes < text.len() / 2, "{}", highlighter.bytes);

    // A small range far into it: the parse is read for the range and not
    // kept, so each keystroke makes it again (once), and gets the colors
    // the parse to the end of the note gives.
    let from = text.find("Paragraph 150 ").unwrap();
    let far = from..from + 60;
    let mut highlighter = Highlighter::new();
    let expected = spans_uncapped(&text, far.clone());
    assert_eq!(highlighter.spans(&text, far.clone()), expected);
    let parsed = highlighter.blocks_parsed();
    for _ in 0..3 {
        // Typed at the end of the parse's reach, after the range.
        let end = text.len();
        text.insert(end - 2, 'y');
        assert_eq!(highlighter.spans(&text, far.clone()), expected);
    }
    assert_eq!(highlighter.blocks_parsed(), parsed + 3);
    assert!(highlighter.bytes < text.len() / 2, "{}", highlighter.bytes);
}

#[test]
fn a_note_without_a_last_line_ending_reads_like_one_with() {
    // Without a line ending, the grammar reads a bare `# ` differently
    // depending on what comes before it.
    let a = "x\n\n**a**\n# ";
    let b = "- a\n\n\n# b\n\n\n**a**\n# ";
    assert_eq!(spans(a, 0..a.len()), spans(&format!("{a}\n"), 0..a.len()));
    assert_eq!(spans(b, 0..b.len()), whole_document_spans(b));
}

fn token() -> impl Strategy<Value = Token> {
    proptest::sample::select(Token::ALL.to_vec())
}

proptest! {
    #![proptest_config(config())]

    /// Flattening: each byte is drawn with the tokens of the captures over
    /// it, those starting earlier first (and, from one start, the one made
    /// first), bytes under no capture have none, and neighbors with the same
    /// tokens are one span.
    #[test]
    fn prop_flattening_keeps_every_capture_over_its_bytes(
        items in prop::collection::vec((0usize..30, 1usize..12, token()), 0..10),
    ) {
        let items: Vec<(Range<usize>, Token)> =
            items.into_iter().map(|(s, l, t)| (s..s + l, t)).collect();
        let spans = super::block::flatten(&items);
        let mut previous: Option<&Span> = None;
        for span in &spans {
            prop_assert!(!span.range.is_empty() && !span.tokens.is_empty());
            if let Some(previous) = previous {
                prop_assert!(previous.range.end <= span.range.start);
                prop_assert!(
                    previous.range.end != span.range.start || previous.tokens != span.tokens,
                    "{previous:?} and {span:?} are one span"
                );
            }
            previous = Some(span);
        }
        for byte in 0..45 {
            let mut covering: Vec<(usize, usize, Token)> = items
                .iter()
                .enumerate()
                .filter(|(_, (range, _))| range.contains(&byte))
                .map(|(i, (range, token))| (range.start, i, *token))
                .collect();
            covering.sort();
            let expected: Vec<Token> = covering.into_iter().map(|(_, _, t)| t).collect();
            let got = spans
                .iter()
                .find(|s| s.range.contains(&byte))
                .map(|s| s.tokens.clone())
                .unwrap_or_default();
            prop_assert_eq!(got, expected, "byte {}", byte);
        }
    }

    /// A capture the table doesn't name is the token of the part before its
    /// first dot.
    #[test]
    fn prop_captures_fall_back_to_their_prefix(
        head in prop_oneof![
            proptest::sample::select(vec!["keyword", "string", "function", "variable", "punctuation", "text", "none"])
                .prop_map(str::to_owned),
            "[a-z_]{1,8}",
        ],
        tail in "[a-z_]{1,8}(\\.[a-z_]{1,8}){0,2}",
    ) {
        let name = format!("{head}.{tail}");
        prop_assume!(!CAPTURES.iter().any(|(capture, _)| *capture == name));
        prop_assert_eq!(Token::from_capture(&name), Token::from_capture(&head), "{}", name);
    }

    /// Styling is laying one token's style over the tokens before it.
    #[test]
    fn prop_styles_lay_over_what_is_under(
        tokens in prop::collection::vec(token(), 0..5),
        extra in token(),
        colors in prop::collection::vec((prop::option::of(0u32..0x100_0000), prop::option::of(any::<bool>()), prop::option::of(100u16..900)), 37),
    ) {
        let syntax: BTreeMap<&'static str, TokenStyle> = Token::ALL
            .iter()
            .zip(colors)
            .map(|(token, (color, italic, weight))| {
                (token.key(), TokenStyle { color: color.map(Rgb), italic, weight })
            })
            .collect();
        let mut longer = tokens.clone();
        longer.push(extra);
        prop_assert_eq!(
            style(&longer, &syntax),
            style(&[extra], &syntax).or(style(&tokens, &syntax)),
        );
    }

    /// A range is widened to character boundaries within the text.
    #[test]
    fn prop_ranges_are_normalized(text in "[a-é日🙂 \\n]{0,20}", start in 0usize..40, end in 0usize..40) {
        let range = super::normalize(&text, start..end);
        prop_assert!(range.start <= range.end && range.end <= text.len());
        prop_assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
        // Widened, never narrowed (to what is in the text).
        let (start, end) = (start.min(text.len()), end.min(text.len()));
        if start <= end {
            prop_assert!(range.start <= start && range.end >= end);
        }
    }
}
