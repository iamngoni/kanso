//! What related notes' embeddings say about the open note, for the views
//! around it: the tags its nearest notes suggest, for the tag list, `#`
//! autocomplete and the inspector, and a note that says nearly the same
//! as it does, for the editor to mention (`DUPLICATE_SIMILARITY`).
//!
//! Read from the stored vectors, so nothing waits on a model; nothing is
//! there while related notes are off or the note isn't embedded yet, and
//! never for a note that is locked or tagged `#ai-exclude`. This is the
//! state the apps share, with no window or runtime in it: the desktop's
//! `NoteMeaning` entity and the phones' library hold one.
//!
//! Also here: the rule for which words of a search are worth searching by
//! meaning, since the palette and the phones' search ask the same.

use std::collections::HashSet;

use kanso_store::{RelatedNote, Store, TagSuggestion};

/// How many tags are suggested at once.
pub const SUGGESTED_TAGS: usize = 5;

/// The fewest letters and digits a search needs to be searched by meaning.
pub const MEANING_MIN_CHARS: usize = 3;

/// The words searched by meaning: the query, when it is plain words long
/// enough to say something, rather than a search with qualifiers.
pub fn meaning_query(query: &str) -> Option<String> {
    let words = query.trim();
    let plain = !words.contains(':') && !words.starts_with('-') && !words.contains('"');
    (plain && words.chars().filter(|c| c.is_alphanumeric()).count() >= MEANING_MIN_CHARS)
        .then(|| words.to_owned())
}

/// Two notes, in the same order whichever is first.
pub fn pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_owned(), b.to_owned())
    } else {
        (b.to_owned(), a.to_owned())
    }
}

/// Pairs of notes said not to be the same, for as long as the app runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dismissed(HashSet<(String, String)>);

impl Dismissed {
    /// `a` and `b` are different notes after all: from either side.
    pub fn add(&mut self, a: &str, b: &str) {
        self.0.insert(pair(a, b));
    }

    pub fn has(&self, a: &str, b: &str) -> bool {
        self.0.contains(&pair(a, b))
    }
}

/// What the stored vectors say about note `id`: the tags its nearest notes
/// have (none while it has no embedding) and its near twin. Nothing while
/// related notes are off.
pub fn read(store: &Store, id: &str) -> (Vec<TagSuggestion>, Option<RelatedNote>) {
    let Some(model) = store.embedding_model().ok().flatten() else {
        return (Vec::new(), None);
    };
    let suggested = store
        .tag_suggestions(id, &model, SUGGESTED_TAGS)
        .ok()
        .flatten()
        .unwrap_or_default();
    let twin = store.near_duplicate(id, &model).ok().flatten();
    (suggested, twin)
}

/// Tags suggested for a note that `own` (its tags now, by ID) hasn't got,
/// heaviest first, at most [`SUGGESTED_TAGS`].
pub fn suggested_tags(suggested: &[TagSuggestion], own: &[String]) -> Vec<TagSuggestion> {
    suggested
        .iter()
        .filter(|s| !own.contains(&s.tag_id))
        .take(SUGGESTED_TAGS)
        .cloned()
        .collect()
}

/// The meaning of the open note.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meaning {
    note: Option<String>,
    suggested: Vec<TagSuggestion>,
    /// The note that says nearly the same as the open one.
    twin: Option<RelatedNote>,
    dismissed: Dismissed,
}

impl Meaning {
    /// The note open now (`None`: none). Whether it changed; what was read
    /// for the one before is forgotten.
    pub fn follow(&mut self, note: Option<&str>) -> bool {
        if self.note.as_deref() == note {
            return false;
        }
        self.note = note.map(str::to_owned);
        self.suggested.clear();
        self.twin = None;
        true
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// Keeps what was read for the open note; whether it differs from what
    /// was kept.
    pub fn set(&mut self, suggested: Vec<TagSuggestion>, twin: Option<RelatedNote>) -> bool {
        let changed = self.suggested != suggested || self.twin != twin;
        self.suggested = suggested;
        self.twin = twin;
        changed
    }

    /// Tags suggested for the open note that `own` (its tags now) hasn't
    /// got, heaviest first.
    pub fn suggested_tags(&self, own: &[String]) -> Vec<TagSuggestion> {
        suggested_tags(&self.suggested, own)
    }

    /// The note that says nearly the same as the open one, unless that was
    /// dismissed.
    pub fn twin(&self) -> Option<&RelatedNote> {
        let note = self.note.as_ref()?;
        self.twin
            .as_ref()
            .filter(|other| !self.dismissed.has(note, &other.id))
    }

    /// The open note and its near twin are different notes after all: not
    /// mentioned again, from either side. Whether there was a twin.
    pub fn dismiss_twin(&mut self) -> bool {
        match (&self.note, &self.twin) {
            (Some(note), Some(other)) => {
                self.dismissed.add(note, &other.id);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn tag(id: &str, weight: f32) -> TagSuggestion {
        TagSuggestion {
            tag_id: id.into(),
            name: id.into(),
            weight,
        }
    }

    fn twin(id: &str) -> RelatedNote {
        RelatedNote {
            id: id.into(),
            title: id.into(),
            score: 0.97,
        }
    }

    #[test]
    fn meaning_queries_are_plain_words_that_say_something() {
        assert_eq!(meaning_query(" fruit trees "), Some("fruit trees".into()));
        assert_eq!(meaning_query("ab"), None);
        assert_eq!(meaning_query("tag:work"), None);
        assert_eq!(meaning_query("-draft"), None);
        assert_eq!(meaning_query("\"exact words\""), None);
        assert_eq!(meaning_query("a b c"), Some("a b c".into()));
        assert_eq!(meaning_query("?! ."), None);
    }

    #[test]
    fn a_twin_dismissed_stays_dismissed_from_both_sides() {
        let mut meaning = Meaning::default();
        meaning.follow(Some("note:a"));
        meaning.set(vec![], Some(twin("note:b")));
        assert_eq!(meaning.twin().map(|t| t.id.as_str()), Some("note:b"));
        assert!(meaning.dismiss_twin());
        assert!(meaning.twin().is_none());
        // The other note is open now, and its twin is the first.
        meaning.follow(Some("note:b"));
        meaning.set(vec![], Some(twin("note:a")));
        assert!(meaning.twin().is_none());
        // Another note is still mentioned.
        meaning.follow(Some("note:c"));
        meaning.set(vec![], Some(twin("note:a")));
        assert!(meaning.twin().is_some());
        // Nothing to dismiss without a twin.
        meaning.follow(None);
        assert!(!meaning.dismiss_twin());
    }

    #[test]
    fn following_another_note_forgets_what_was_read() {
        let mut meaning = Meaning::default();
        assert!(meaning.follow(Some("note:a")));
        assert!(!meaning.follow(Some("note:a")));
        assert!(meaning.set(vec![tag("tag:x", 1.0)], None));
        assert!(!meaning.set(vec![tag("tag:x", 1.0)], None));
        assert_eq!(meaning.suggested_tags(&[]).len(), 1);
        meaning.follow(Some("note:b"));
        assert!(meaning.suggested_tags(&[]).is_empty());
    }

    proptest! {
        /// Suggested tags are never ones the note has, keep their order and
        /// number at most [`SUGGESTED_TAGS`].
        #[test]
        fn prop_suggestions_leave_out_own_tags(
            ids in prop::collection::vec(0u8..12, 0..12),
            own in prop::collection::vec(0u8..12, 0..6),
        ) {
            let suggested: Vec<TagSuggestion> = ids
                .iter()
                .enumerate()
                .map(|(i, id)| tag(&format!("tag:{id}"), 10. - i as f32 * 0.5))
                .collect();
            let own: Vec<String> = own.iter().map(|id| format!("tag:{id}")).collect();
            let shown = suggested_tags(&suggested, &own);
            prop_assert!(shown.len() <= SUGGESTED_TAGS);
            prop_assert!(shown.iter().all(|s| !own.contains(&s.tag_id)));
            let expected: Vec<&TagSuggestion> = suggested
                .iter()
                .filter(|s| !own.contains(&s.tag_id))
                .take(SUGGESTED_TAGS)
                .collect();
            prop_assert_eq!(shown.iter().collect::<Vec<_>>(), expected);
        }

        /// Dismissing a pair holds whichever note is named first.
        #[test]
        fn prop_a_dismissed_pair_is_unordered(a in "[a-z]{1,6}", b in "[a-z]{1,6}", c in "[a-z]{1,6}") {
            let mut dismissed = Dismissed::default();
            prop_assert!(!dismissed.has(&a, &b));
            dismissed.add(&a, &b);
            prop_assert!(dismissed.has(&a, &b));
            prop_assert!(dismissed.has(&b, &a));
            if c != a && c != b {
                prop_assert!(!dismissed.has(&a, &c));
            }
        }

        /// Plain words of enough letters are searched, and keep their text
        /// without the spaces round them; a qualifier is never.
        #[test]
        fn prop_meaning_queries_follow_the_rule(words in "[a-z ]{0,20}", pad in " {0,3}") {
            let query = format!("{pad}{words}{pad}");
            let letters = words.chars().filter(|c| c.is_alphanumeric()).count();
            match meaning_query(&query) {
                Some(found) => {
                    prop_assert!(letters >= MEANING_MIN_CHARS);
                    prop_assert_eq!(found, query.trim());
                }
                None => prop_assert!(letters < MEANING_MIN_CHARS),
            }
            prop_assert_eq!(meaning_query(&format!("tag:{words}")), None);
        }
    }
}
