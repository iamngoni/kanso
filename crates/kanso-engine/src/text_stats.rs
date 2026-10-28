//! Counts the editor's footer shows: words in a note, numbers with
//! thousands separators and the caret's line and column. Also what the
//! editor reads off its text a piece at a time (the pieces of its rope), so
//! a long note isn't copied for it.

/// The words in Markdown text: runs between whitespace with at least one
/// letter or digit in them. Markup on its own, like `#`, `-`, `|` or a
/// table's `---`, isn't a word; `**bold**` and `[a](b)` are one each.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .count()
}

/// `n` with a comma between each group of three digits: 2,841.
pub fn format_count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (ix, digit) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// "1 word" or "2,841 words".
pub fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", format_count(n), if n == 1 { one } else { many })
}

/// The line and column of byte `byte` in `text`, both counted from 0: the
/// line is the number of newlines before it, the column the characters
/// since the last one (not bytes, so a caret after "é" is at column 1). A
/// `byte` past the end is the end, and one inside a character is the start
/// of that character.
pub fn caret_line_col(text: &str, byte: usize) -> (usize, usize) {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    let before = &text[..byte];
    let line = before.bytes().filter(|&b| b == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, before[line_start..].chars().count())
}

/// The length in characters of each line of the text made of `pieces`, as
/// `text.split('\n')` would find them: a trailing newline ends with an
/// empty line.
pub fn line_lengths<'a>(pieces: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let mut lengths = vec![0];
    for piece in pieces {
        let mut lines = piece.split('\n');
        if let Some(first) = lines.next()
            && let Some(last) = lengths.last_mut()
        {
            *last += first.chars().count();
        }
        lengths.extend(lines.map(|line| line.chars().count()));
    }
    lengths
}

/// Whether the text made of `pieces` is `text`.
pub fn pieces_eq<'a>(pieces: impl IntoIterator<Item = &'a str>, text: &str) -> bool {
    let mut rest = text;
    for piece in pieces {
        match rest.strip_prefix(piece) {
            Some(after) => rest = after,
            None => return false,
        }
    }
    rest.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Text that mixes words, markup and every kind of whitespace.
    fn text() -> impl Strategy<Value = String> {
        "([a-zé0-9#*|>`\\[\\]()-]{0,6}[ \t\n\u{a0}]{0,3}){0,12}"
    }

    /// `text` cut into pieces at the given points, as a rope hands it out.
    fn pieces(text: &str, cuts: &[usize]) -> Vec<String> {
        let mut points: Vec<usize> = cuts
            .iter()
            .map(|c| c % (text.len() + 1))
            .filter(|c| text.is_char_boundary(*c))
            .collect();
        points.push(0);
        points.push(text.len());
        points.sort_unstable();
        points.dedup();
        points
            .windows(2)
            .map(|w| text[w[0]..w[1]].to_owned())
            .collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Equivalence: lines measured a piece at a time are the lines of the
        /// whole text, wherever it is cut.
        #[test]
        fn prop_line_lengths_match_the_whole_text(text in "[a-zé\n ]{0,60}", cuts in prop::collection::vec(any::<usize>(), 0..6)) {
            let expected: Vec<usize> = text.split('\n').map(|l| l.chars().count()).collect();
            let pieces = pieces(&text, &cuts);
            prop_assert_eq!(line_lengths(pieces.iter().map(String::as_str)), expected);
        }

        /// Equivalence: the caret's line and column are where `split('\n')`
        /// puts a byte, and a caret at the end is the last line's length.
        #[test]
        fn prop_caret_line_col_matches_split(text in "[a-zé\n 日]{0,60}", at in any::<prop::sample::Index>()) {
            let boundaries: Vec<usize> = text.char_indices().map(|(i, _)| i).chain([text.len()]).collect();
            let byte = boundaries[at.index(boundaries.len())];
            let (line, col) = caret_line_col(&text, byte);
            let lines: Vec<&str> = text.split('\n').collect();
            let mut start = 0;
            let mut expected = (0, 0);
            for (ix, l) in lines.iter().enumerate() {
                if byte <= start + l.len() {
                    expected = (ix, text[start..byte].chars().count());
                    break;
                }
                start += l.len() + 1;
            }
            prop_assert_eq!((line, col), expected);
            prop_assert!(line < lines.len());
            prop_assert!(col <= lines[line].chars().count());
            prop_assert_eq!(caret_line_col(&text, text.len()), (lines.len() - 1, lines[lines.len() - 1].chars().count()));
        }

        /// Equivalence: comparing a piece at a time is comparing the texts.
        #[test]
        fn prop_pieces_eq_is_string_equality(a in "[ab\n]{0,12}", b in "[ab\n]{0,12}", cuts in prop::collection::vec(any::<usize>(), 0..5)) {
            let pieces = pieces(&a, &cuts);
            prop_assert_eq!(pieces_eq(pieces.iter().map(String::as_str), &b), a == b);
        }

        /// Additivity: two texts joined by a space have the words of both.
        #[test]
        fn prop_word_count_adds_up(a in text(), b in text()) {
            prop_assert_eq!(word_count(&format!("{a} {b}")), word_count(&a) + word_count(&b));
        }

        /// Whitespace doesn't count: any run of it can stand in for any
        /// other without changing the count.
        #[test]
        fn prop_word_count_ignores_the_kind_of_whitespace(
            tokens in prop::collection::vec("[a-z#|*-]{1,5}", 0..12),
            gaps in prop::collection::vec("[ \t\n\r]{1,4}", 12),
        ) {
            let spaced = tokens.join(" ");
            let mixed: String = tokens.iter().zip(&gaps).map(|(t, g)| format!("{t}{g}")).collect();
            prop_assert_eq!(word_count(&spaced), word_count(&mixed));
        }

        /// Bound: words are a subset of the whitespace-separated runs, and
        /// runs of markup alone never count.
        #[test]
        fn prop_word_count_skips_markup(text in text(), markup in prop::collection::vec("[#|*>`-]{1,4}", 0..10)) {
            prop_assert!(word_count(&text) <= text.split_whitespace().count());
            prop_assert_eq!(word_count(&markup.join(" ")), 0);
            prop_assert_eq!(word_count(&format!("{} {text}", markup.join("\n"))), word_count(&text));
        }

        /// Round trip and shape: without its commas the text is the number,
        /// and the commas split it into groups of three after a first group
        /// of one to three digits.
        #[test]
        fn prop_format_count_groups_by_three(n in any::<usize>()) {
            let shown = format_count(n);
            prop_assert_eq!(shown.replace(',', "").parse::<usize>().unwrap(), n);
            let groups: Vec<&str> = shown.split(',').collect();
            prop_assert!((1..=3).contains(&groups[0].len()), "{}", shown);
            prop_assert!(groups[1..].iter().all(|g| g.len() == 3), "{}", shown);
            prop_assert!(groups[0] == "0" || !groups[0].starts_with('0'), "{}", shown);
        }
    }

    #[test]
    fn word_count_examples() {
        assert_eq!(word_count(""), 0);
        assert_eq!(
            word_count("# MCP support model\n\n- Enable **secure** access."),
            6
        );
        assert_eq!(word_count("| a | b |\n|---|---|"), 2);
        assert_eq!(
            word_count("[protocol overview](https://example.com) here"),
            3
        );
    }

    #[test]
    fn caret_line_col_examples() {
        assert_eq!(caret_line_col("", 0), (0, 0));
        assert_eq!(caret_line_col("ab\ncd", 4), (1, 1));
        assert_eq!(caret_line_col("ab\n", 3), (1, 0));
        assert_eq!(caret_line_col("é\nx", 2), (0, 1));
        // Inside "é" is its start; past the end is the end.
        assert_eq!(caret_line_col("é", 1), (0, 0));
        assert_eq!(caret_line_col("ab", 99), (0, 2));
    }

    #[test]
    fn format_count_examples() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(2841), "2,841");
        assert_eq!(format_count(1_234_567), "1,234,567");
        assert_eq!(plural(1, "word", "words"), "1 word");
        assert_eq!(plural(2841, "char", "chars"), "2,841 chars");
    }
}
