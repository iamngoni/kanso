//! Recurring themes: the tags that came up most on a week's notes, and the
//! words and phrases that appeared in at least three different notes.
//!
//! Counted as document frequency (a word in one note counts once), after
//! dropping stop words and words under four letters. Two-word phrases are
//! counted the same way and preferred when they reach the threshold. Pure.

use std::collections::{BTreeMap, BTreeSet};

use kanso_store::ThemeNote;

/// A term must be in this many different notes.
pub const MIN_NOTES: usize = 3;
/// The most words and phrases listed.
pub const MAX_TERMS: usize = 8;
/// The most tags listed.
pub const MAX_TAGS: usize = 5;
/// Words shorter than this (in letters) are left out.
pub const MIN_LETTERS: usize = 4;

/// What a theme is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeKind {
    Tag,
    Word,
    Phrase,
}

/// A recurring theme and how many notes it was in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub term: String,
    pub notes: u32,
    pub kind: ThemeKind,
}

const STOP_WORDS: &[&str] = &[
    "about",
    "above",
    "after",
    "again",
    "against",
    "also",
    "always",
    "among",
    "another",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "cannot",
    "could",
    "didn't",
    "does",
    "doesn't",
    "doing",
    "don't",
    "done",
    "down",
    "during",
    "each",
    "either",
    "else",
    "even",
    "ever",
    "every",
    "from",
    "further",
    "gets",
    "getting",
    "going",
    "gone",
    "have",
    "haven't",
    "having",
    "here",
    "hers",
    "herself",
    "himself",
    "into",
    "isn't",
    "itself",
    "just",
    "keep",
    "kind",
    "know",
    "last",
    "later",
    "less",
    "like",
    "little",
    "look",
    "looks",
    "made",
    "make",
    "makes",
    "many",
    "maybe",
    "might",
    "more",
    "most",
    "much",
    "must",
    "myself",
    "need",
    "needs",
    "never",
    "next",
    "nothing",
    "once",
    "only",
    "other",
    "ours",
    "ourselves",
    "over",
    "same",
    "should",
    "since",
    "some",
    "something",
    "still",
    "such",
    "take",
    "than",
    "that",
    "that's",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "there's",
    "these",
    "they",
    "they're",
    "thing",
    "things",
    "think",
    "this",
    "those",
    "though",
    "through",
    "today",
    "together",
    "tomorrow",
    "under",
    "until",
    "upon",
    "very",
    "want",
    "wants",
    "well",
    "were",
    "what",
    "what's",
    "when",
    "where",
    "whether",
    "which",
    "while",
    "will",
    "with",
    "within",
    "without",
    "would",
    "wouldn't",
    "yesterday",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "note",
    "notes",
    "todo",
    "https",
    "http",
    "www",
];

fn is_stop(word: &str) -> bool {
    STOP_WORDS.contains(&word)
}

/// The words of `text` worth counting, in order, with `None` standing for a
/// break (a stop word, a short word or punctuation) that a phrase can't
/// cross. Text in code fences is skipped.
fn words(text: &str) -> Vec<Option<String>> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fenced = !fenced;
            out.push(None);
            continue;
        }
        if fenced {
            continue;
        }
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut Vec<Option<String>>| {
            if word.is_empty() {
                return;
            }
            let w = std::mem::take(word);
            let letters = w.chars().filter(|c| c.is_alphabetic()).count();
            out.push((letters >= MIN_LETTERS && !is_stop(&w)).then_some(w));
        };
        for c in line.chars() {
            if c.is_alphanumeric() || c == '\'' || c == '’' {
                word.extend(c.to_lowercase().map(|c| if c == '’' { '\'' } else { c }));
            } else {
                flush(&mut word, &mut out);
                // Punctuation inside a sentence ends a phrase.
                if matches!(
                    c,
                    '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']' | '|'
                ) {
                    out.push(None);
                }
            }
        }
        flush(&mut word, &mut out);
        out.push(None);
    }
    out
}

/// The terms of one note: its words and its two-word phrases, each once.
fn terms_of(note: &ThemeNote) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut singles = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    let text = format!("{}\n{}", note.title, note.body);
    let list = words(&text);
    for (ix, word) in list.iter().enumerate() {
        let Some(word) = word else { continue };
        singles.insert(word.clone());
        if let Some(Some(next)) = list.get(ix + 1) {
            pairs.insert(format!("{word} {next}"));
        }
    }
    (singles, pairs)
}

/// The tags that were on the most notes, then the words and phrases that
/// were in at least [`MIN_NOTES`] notes. Ties break by name. Notes are
/// expected to be ones the brief may show (not locked, not excluded).
pub fn themes(notes: &[ThemeNote]) -> Vec<Theme> {
    let mut tag_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut singles: BTreeMap<String, u32> = BTreeMap::new();
    let mut pairs: BTreeMap<String, u32> = BTreeMap::new();
    for note in notes {
        for tag in note.tags.iter().collect::<BTreeSet<_>>() {
            *tag_counts.entry(tag.clone()).or_default() += 1;
        }
        let (s, p) = terms_of(note);
        for term in s {
            *singles.entry(term).or_default() += 1;
        }
        for term in p {
            *pairs.entry(term).or_default() += 1;
        }
    }
    let mut out: Vec<Theme> = {
        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        tags.into_iter()
            .take(MAX_TAGS)
            .map(|(term, notes)| Theme {
                term,
                notes,
                kind: ThemeKind::Tag,
            })
            .collect()
    };
    let mut phrases: Vec<(String, u32)> = pairs
        .into_iter()
        .filter(|(_, n)| *n as usize >= MIN_NOTES)
        .collect();
    phrases.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    phrases.truncate(MAX_TERMS);
    let covered: BTreeSet<&str> = phrases.iter().flat_map(|(p, _)| p.split(' ')).collect();
    let mut words: Vec<(String, u32)> = singles
        .into_iter()
        .filter(|(w, n)| *n as usize >= MIN_NOTES && !covered.contains(w.as_str()))
        .collect();
    words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut terms: Vec<Theme> = phrases
        .into_iter()
        .map(|(term, notes)| Theme {
            term,
            notes,
            kind: ThemeKind::Phrase,
        })
        .chain(words.into_iter().map(|(term, notes)| Theme {
            term,
            notes,
            kind: ThemeKind::Word,
        }))
        .collect();
    terms.sort_by(|a, b| {
        b.notes
            .cmp(&a.notes)
            .then_with(|| (a.kind as u8).cmp(&(b.kind as u8)).reverse())
            .then_with(|| a.term.cmp(&b.term))
    });
    terms.truncate(MAX_TERMS);
    out.extend(terms);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn note(id: &str, title: &str, body: &str, tags: &[&str]) -> ThemeNote {
        ThemeNote {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
        }
    }

    #[test]
    fn counts_documents_not_occurrences() {
        let notes = vec![
            note("a", "Garden", "tomato tomato tomato tomato", &[]),
            note("b", "Soup", "tomato basil", &[]),
        ];
        assert!(themes(&notes).is_empty(), "two notes aren't enough");
        let mut notes = notes;
        notes.push(note("c", "Pasta", "Tomato sauce", &[]));
        let got = themes(&notes);
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].term.as_str(), got[0].notes), ("tomato", 3));
    }

    #[test]
    fn phrases_are_preferred_over_their_words() {
        let notes = vec![
            note("a", "x", "The product roadmap needs work", &["planning"]),
            note(
                "b",
                "y",
                "Reviewed product roadmap again",
                &["planning", "work"],
            ),
            note("c", "z", "A product roadmap draft", &[]),
        ];
        let got = themes(&notes);
        let terms: Vec<_> = got.iter().map(|t| (t.term.as_str(), t.kind)).collect();
        assert_eq!(
            terms,
            [
                ("planning", ThemeKind::Tag),
                ("work", ThemeKind::Tag),
                ("product roadmap", ThemeKind::Phrase),
            ]
        );
    }

    #[test]
    fn stop_words_short_words_and_code_are_left_out() {
        let notes: Vec<_> = (0..3)
            .map(|i| {
                note(
                    &format!("n{i}"),
                    "t",
                    "This that with from cat dog\n```\nsecretcode secretcode\n```\n",
                    &[],
                )
            })
            .collect();
        assert!(themes(&notes).is_empty());
    }

    #[test]
    fn phrases_dont_cross_punctuation() {
        let notes: Vec<_> = (0..3)
            .map(|i| {
                note(
                    &format!("n{i}"),
                    "t",
                    "budget meeting. planning session",
                    &[],
                )
            })
            .collect();
        let got = themes(&notes);
        assert!(got.iter().all(|t| t.term != "meeting planning"));
        assert!(got.iter().any(|t| t.term == "budget meeting"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Every theme is in at least the threshold of notes (tags
        /// excepted), at most the cap are listed, and the notes' order
        /// doesn't matter.
        #[test]
        fn prop_themes_meet_the_threshold_and_ignore_order(
            bodies in prop::collection::vec("[a-z ]{0,60}", 0..8),
            tags in prop::collection::vec(prop::collection::vec("[ab]", 0..3), 0..8),
        ) {
            let notes: Vec<ThemeNote> = bodies.iter().enumerate()
                .map(|(i, b)| ThemeNote {
                    id: format!("n{i}"), title: String::new(), body: b.clone(),
                    tags: tags.get(i).cloned().unwrap_or_default(),
                })
                .collect();
            let got = themes(&notes);
            prop_assert!(got.iter().filter(|t| t.kind != ThemeKind::Tag).count() <= MAX_TERMS);
            prop_assert!(got.iter().filter(|t| t.kind == ThemeKind::Tag).count() <= MAX_TAGS);
            for theme in &got {
                prop_assert!(theme.notes as usize <= notes.len());
                if theme.kind != ThemeKind::Tag {
                    prop_assert!(theme.notes as usize >= MIN_NOTES);
                }
            }
            let mut reversed = notes.clone();
            reversed.reverse();
            prop_assert_eq!(themes(&reversed), got);
        }
    }
}
