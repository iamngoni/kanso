//! The model's paragraph, made safe to draw: citations written as `[[Title]]`
//! become real links to the notes that were on the page, and nothing else
//! is ever linked.
//!
//! A title the model invented becomes plain text, without brackets and
//! without a link that would create a note when clicked. The page shows at
//! most about 600 characters of the paragraph, with *More*.

use kanso_core::links::normalize_title;

use crate::links::id_link;

/// The page shows this many characters of the paragraph before *More*.
pub const SHOWN_CHARS: usize = 600;

/// Rewrites every `[[Title]]` and `[[Title|shown]]` of `text` whose title is
/// in `pairs` (`(note id, title)`, the notes the model was given) as a
/// Markdown link to that note. Any other `[[...]]` becomes its plain words,
/// and no `[[` or `]]` survives.
pub fn link_citations(text: &str, pairs: &[(String, String)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            // No close: the brackets go, the words stay.
            rest = after;
            continue;
        };
        let inner = &after[..end];
        // A title never holds a bracket or a break; if this does, it was not
        // a citation: drop the opening brackets and read on from there.
        if inner.contains(['[', ']', '\n']) {
            rest = after;
            continue;
        }
        let (title, shown) = match inner.split_once('|') {
            Some((title, shown)) => (title.trim(), shown.trim()),
            None => (inner.trim(), inner.trim()),
        };
        let wanted = normalize_title(title);
        let found = pairs
            .iter()
            .find(|(_, t)| normalize_title(t) == wanted && !wanted.is_empty());
        let label = if shown.is_empty() { title } else { shown };
        match found {
            Some((id, _)) => out.push_str(&id_link(id, label)),
            None => out.push_str(label),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out.replace("[[", "").replace("]]", "")
}

/// The paragraph cut for the page: at a sentence end or a space near
/// [`SHOWN_CHARS`], with whether it was cut. A Markdown link is never cut in
/// the middle.
pub fn shown(text: &str) -> (String, bool) {
    if text.chars().count() <= SHOWN_CHARS {
        return (text.to_owned(), false);
    }
    let mut cut = text
        .char_indices()
        .nth(SHOWN_CHARS)
        .map_or(text.len(), |(i, _)| i);
    // Do not end inside `[label](url)`.
    let head = &text[..cut];
    if let Some(open) = head.rfind('[') {
        let closed = head[open..]
            .find("](")
            .and_then(|p| head[open + p..].find(')'));
        if closed.is_none() {
            cut = open;
        }
    }
    let head = &text[..cut];
    let end = head
        .rfind(['.', '!', '?'])
        .filter(|i| *i > SHOWN_CHARS / 2)
        .map_or_else(|| head.rfind(' ').unwrap_or(head.len()), |i| i + 1);
    (text[..end].trim_end().to_owned(), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn pairs() -> Vec<(String, String)> {
        vec![
            (
                "note:aaa".into(),
                "Excalidraw in Kanso: better rendering".into(),
            ),
            ("note:bbb".into(), "Kanvoya Build Sequence".into()),
        ]
    }

    #[test]
    fn known_titles_become_links_and_unknown_ones_plain_words() {
        let text = "See [[Excalidraw in Kanso: better rendering]] and [[kanvoya build sequence|the sequence]] but not [[Made Up]].";
        let got = link_citations(text, &pairs());
        assert!(
            got.contains("[Excalidraw in Kanso: better rendering](") && got.contains("note/aaa)")
        );
        assert!(got.contains("[the sequence](") && got.contains("note/bbb)"));
        assert!(got.ends_with("but not Made Up."));
        assert!(!got.contains("[["));
        assert!(!got.contains("kanso-new"));
    }

    #[test]
    fn stray_brackets_are_removed() {
        assert_eq!(link_citations("a [[ b", &[]), "a  b");
        assert_eq!(link_citations("a ]] b", &[]), "a  b");
        assert_eq!(link_citations("[[a [[b]] c", &[]), "a b c");
    }

    #[test]
    fn a_long_paragraph_is_cut_at_a_sentence() {
        let sentence = "This is a sentence about the week. ";
        let long = sentence.repeat(40);
        let (cut, was) = shown(&long);
        assert!(was);
        assert!(cut.chars().count() <= SHOWN_CHARS);
        assert!(cut.ends_with('.'));
        let (same, was) = shown("short");
        assert!(!was);
        assert_eq!(same, "short");
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// Every link in the output points at a note that was on the page,
        /// and no `[[` survives, whatever the model wrote.
        #[test]
        fn only_known_notes_are_linked_and_no_brackets_survive(
            parts in prop::collection::vec(
                prop_oneof![
                    "[a-zA-Z ,.:|]{0,12}".prop_map(String::from),
                    Just("[[".to_owned()), Just("]]".to_owned()),
                    Just("[[Kanvoya Build Sequence]]".to_owned()),
                    Just("[[Excalidraw in Kanso: better rendering|x]]".to_owned()),
                    Just("[[Invented]]".to_owned()),
                ],
                0..14,
            )
        ) {
            let text = parts.concat();
            let known = pairs();
            let out = link_citations(&text, &known);
            prop_assert!(!out.contains("[["));
            prop_assert!(!out.contains("]]"));
            prop_assert!(!out.contains("kanso-new"));
            let prefix = kanso_core::links::NOTE_LINK_PREFIX;
            let mut rest = out.as_str();
            while let Some(i) = rest.find(prefix) {
                let after = &rest[i + prefix.len()..];
                let id: String = after.chars().take_while(|c| *c != ')').collect();
                prop_assert!(known.iter().any(|(k, _)| k.strip_prefix("note:") == Some(id.as_str())));
                rest = after;
            }
        }

        /// The cut paragraph never passes the cap and is part of the text.
        #[test]
        fn the_cut_is_bounded(words in prop::collection::vec("[a-z]{1,10}[.]?", 1..400)) {
            let text = words.join(" ");
            let (cut, _) = shown(&text);
            prop_assert!(cut.chars().count() <= SHOWN_CHARS);
            prop_assert!(text.starts_with(&cut));
        }
    }
}
