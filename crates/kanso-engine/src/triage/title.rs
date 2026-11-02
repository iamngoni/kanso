//! The title rule Triage uses with no AI: only a clearly weak title gets a
//! suggestion, and it is the first full sentence of the note, trimmed.

use kanso_core::links::plain_line;

use crate::capture::TITLE_WORDS_LEN;

/// Whether `title` is clearly weak: empty, a date or other text with no
/// letters, "Untitled", or a first line that was cut short (more than
/// [`TITLE_WORDS_LEN`] characters, or ending in "…"). Pure.
pub fn is_weak(title: &str) -> bool {
    let title = title.trim();
    title.is_empty()
        || title.chars().count() > TITLE_WORDS_LEN
        || title.ends_with('…')
        || !title.chars().any(char::is_alphabetic)
        || title.to_lowercase().starts_with("untitled")
}

/// A note's words as one line: each line without its Markdown markers and
/// link brackets, blank lines dropped, whitespace collapsed. Pure.
pub fn plain_text(body: &str) -> String {
    body.lines()
        .map(plain_line)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The first full sentence of `body`, without its closing full stop, cut
/// at a space when longer than [`TITLE_WORDS_LEN`] characters. It is always
/// a prefix of [`plain_text`]. `None` for a body with no words. Pure.
pub fn first_sentence(body: &str) -> Option<String> {
    let text = plain_text(body);
    let chars: Vec<char> = text.chars().collect();
    let mut end = chars.len();
    for (at, c) in chars.iter().enumerate() {
        if matches!(c, '.' | '!' | '?' | '。')
            && chars
                .get(at + 1)
                .is_none_or(|next| next.is_whitespace() || *c == '。')
        {
            end = at + 1;
            break;
        }
    }
    let mut sentence: Vec<char> = chars[..end].to_vec();
    if sentence.last() == Some(&'.') {
        sentence.pop();
    }
    if sentence.len() > TITLE_WORDS_LEN {
        sentence.truncate(TITLE_WORDS_LEN);
        // Cut at a space unless that would leave very little.
        if chars
            .get(TITLE_WORDS_LEN)
            .is_some_and(|c| !c.is_whitespace())
            && let Some(at) = sentence.iter().rposition(|c| c.is_whitespace())
            && at > TITLE_WORDS_LEN / 2
        {
            sentence.truncate(at);
        }
    }
    let title: String = sentence.into_iter().collect();
    let title = title.trim().to_owned();
    (!title.is_empty()).then_some(title)
}

/// The title to suggest for a note, if its own is weak and the sentence
/// would be a change. Pure.
pub fn suggest(title: &str, body: &str) -> Option<String> {
    if !is_weak(title) {
        return None;
    }
    first_sentence(body).filter(|t| t.trim() != title.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn weak_titles() {
        for weak in [
            "",
            "  ",
            "Untitled",
            "untitled 3",
            "2025-11-02",
            "12:30",
            "Call the…",
        ] {
            assert!(is_weak(weak), "{weak:?}");
        }
        assert!(is_weak(&"a".repeat(81)));
        for fine in ["Plumber", "Boiler service 2025", "Idea"] {
            assert!(!is_weak(fine), "{fine:?}");
        }
    }

    #[test]
    fn the_first_sentence_is_the_title() {
        assert_eq!(
            first_sentence("remind me to call the plumber. Then email Sam.").as_deref(),
            Some("remind me to call the plumber")
        );
        assert_eq!(
            first_sentence("# Heading\n\nSecond line! More.").as_deref(),
            Some("Heading Second line!")
        );
        assert_eq!(first_sentence("   \n\n"), None);
        assert_eq!(suggest("Plumber", "Anything at all."), None);
        assert_eq!(
            suggest("Untitled", "Buy milk and eggs.").as_deref(),
            Some("Buy milk and eggs")
        );
    }

    #[test]
    fn a_long_sentence_is_cut_at_a_space() {
        let body = "word ".repeat(40);
        let title = first_sentence(&body).unwrap();
        assert!(title.chars().count() <= TITLE_WORDS_LEN);
        assert!(title.ends_with("word"));
    }

    proptest! {
        #![proptest_config(config())]

        /// The suggested title is a prefix of the note's words and never
        /// longer than the title limit.
        #[test]
        fn the_title_is_a_prefix_of_the_body(
            body in "(([a-zé日 ]{0,14}[.!?]?|#{1,2} |- |> |\\[\\[x\\]\\] ){1,5}\n{0,2}){0,6}"
        ) {
            if let Some(title) = first_sentence(&body) {
                prop_assert!(title.chars().count() <= TITLE_WORDS_LEN);
                prop_assert!(!title.is_empty());
                prop_assert!(plain_text(&body).starts_with(&title));
            }
        }

        /// Only a weak title ever gets a suggestion.
        #[test]
        fn a_good_title_gets_none(title in "[a-z]{3,20}", body in "[a-z. ]{0,60}") {
            prop_assert!(suggest(&title, &body).is_none());
        }
    }
}
