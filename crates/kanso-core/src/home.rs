//! Notebook homepages (issue #201): the pure parts. A notebook's Home note
//! is an ordinary note marked `kanso: home` in its front matter; the
//! decisions section reads the lines of the notebook that sound like a
//! decision before the model sees anything.

use crate::properties::{self, Value};

/// The front matter key shared with people and project pages.
pub const HOME_KEY: &str = "kanso";
/// The value that makes a note its notebook's Home note.
pub const HOME_VALUE: &str = "home";

/// The sections a homepage can have, in the order they are shown until the
/// person changes it.
pub const SECTION_KEYS: [&str; 9] = [
    "home_note",
    "about",
    "key_notes",
    "recent",
    "open_tasks",
    "people",
    "decisions",
    "map",
    "inside",
];

/// Whether `body` marks its note as a Home note.
pub fn is_home(body: &str) -> bool {
    let props = properties::parse(body);
    matches!(
        properties::get(&props, HOME_KEY),
        Some(Value::Text(text)) if text.trim().eq_ignore_ascii_case(HOME_VALUE)
    )
}

/// `body` marked as a Home note. A note that is a person's or project's
/// page keeps its own mark: the body is returned as it was.
pub fn mark_home(body: &str) -> String {
    let props = properties::parse(body);
    if matches!(properties::get(&props, HOME_KEY), Some(Value::Text(text))
        if !text.trim().eq_ignore_ascii_case(HOME_VALUE) && !text.trim().is_empty())
    {
        return body.to_owned();
    }
    properties::set_property(body, HOME_KEY, &Value::Text(HOME_VALUE.to_owned()))
}

/// `body` without the Home note mark; anything else stays.
pub fn unmark_home(body: &str) -> String {
    let props = properties::parse(body);
    let Some(index) = props.iter().rposition(|(key, value)| {
        key.eq_ignore_ascii_case(HOME_KEY)
            && matches!(value, Value::Text(t) if t.trim().eq_ignore_ascii_case(HOME_VALUE))
    }) else {
        return body.to_owned();
    };
    properties::remove_property_at(body, index)
}

/// What a Home note shows: its text without the front matter.
pub fn home_text(body: &str) -> &str {
    crate::entities::own_text(body)
}

/// The title of the Home note made for a notebook.
pub fn new_home_title(book_name: &str) -> String {
    format!("About {}", book_name.trim())
}

const CUES: [&str; 9] = [
    "decided",
    "decision",
    "we chose",
    "agreed",
    "going with",
    "settled on",
    "will not",
    "won't",
    "rejected",
];

/// Whether a line sounds like a decision: one of the cue words
/// (case-insensitive, curly apostrophes read as straight), or a
/// `Decision:` heading or property.
pub fn is_decision_line(line: &str) -> bool {
    let lower = line.to_lowercase().replace('\u{2019}', "'");
    CUES.iter().any(|cue| lower.contains(cue))
}

/// The 0-based lines of `body` that sound like a decision, front matter
/// included (a `decision: ...` property counts), in order.
pub fn decision_lines(body: &str) -> Vec<u32> {
    body.split('\n')
        .enumerate()
        .filter(|(_, line)| is_decision_line(line))
        .map(|(ix, _)| ix as u32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// Every line built with a cue is found, at any case and with any
        /// words around it, and no line built without one is.
        #[test]
        fn prop_decision_cues_find_every_cue_line(
            lines in proptest::collection::vec(
                (proptest::bool::ANY, 0usize..9, "[a-z ]{0,20}", "[a-z ]{0,20}", proptest::bool::ANY),
                0..12,
            )
        ) {
            let mut text = Vec::new();
            let mut expected = Vec::new();
            for (ix, (cue, which, before, after, upper)) in lines.iter().enumerate() {
                // The filler never spells a cue by accident.
                let plain = |s: &str| s.replace("agreed", "").replace("decid", "").replace("chose", "")
                    .replace("going with", "").replace("settled on", "").replace("will not", "")
                    .replace("won't", "").replace("rejected", "");
                let (before, after) = (plain(before), plain(after));
                let line = if *cue {
                    let word = if *upper { CUES[*which].to_uppercase() } else { CUES[*which].to_owned() };
                    format!("{before}{word}{after}")
                } else {
                    format!("{before}{after}")
                };
                if *cue {
                    expected.push(ix as u32);
                }
                text.push(line);
            }
            prop_assert_eq!(decision_lines(&text.join("\n")), expected);
        }

        /// Marking and unmarking leaves the rest of the note alone.
        #[test]
        fn prop_marking_home_round_trips(body in "[a-z \n]{0,40}") {
            let marked = mark_home(&body);
            prop_assert!(is_home(&marked));
            prop_assert_eq!(mark_home(&marked), marked.clone());
            let back = unmark_home(&marked);
            prop_assert!(!is_home(&back));
            prop_assert_eq!(home_text(&marked).trim(), home_text(&back).trim());
        }
    }

    // no-prop-test: a literal person page.
    #[test]
    fn a_person_page_is_not_made_a_home_note() {
        let body = "---\nkanso: person\n---\nSam";
        assert_eq!(mark_home(body), body);
        assert!(!is_home(body));
    }

    // no-prop-test: two literal spellings.
    #[test]
    fn decision_headings_and_curly_apostrophes_count() {
        assert!(is_decision_line("## Decision: use the 2019 set"));
        assert!(is_decision_line("We won\u{2019}t ship it"));
        assert!(!is_decision_line("Notes from the call"));
    }
}
