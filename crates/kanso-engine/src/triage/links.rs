//! Link suggestions: existing notes a note could point to. Shared with
//! "Resurfacing related notes while you write" (issue #197), which ranks
//! the same candidates for the note being typed: nothing here is specific
//! to the Inbox.
//!
//! A candidate is a note whose title appears in the text (whole words, at
//! least two words or six characters), or one of the nearest notes by
//! meaning (at least [`NEAR_SIMILARITY`] similar, the nearest
//! [`NEAR_COUNT`]). A note that is already linked, the note itself, and a
//! title that can't be a wiki-link are never candidates. Accepting a
//! candidate writes one `Related:` line at the end of the body
//! ([`with_related_line`]) and never edits a sentence.

use std::collections::{HashMap, HashSet};

use kanso_core::links::normalize_title;
use kanso_store::{RelatedNote, Result, Store};

/// How similar a nearest note must be to be suggested.
pub const NEAR_SIMILARITY: f32 = 0.6;
/// How many of the nearest notes are considered.
pub const NEAR_COUNT: usize = 5;
/// The score of a note whose title is in the text.
pub const MENTION_SCORE: f32 = 0.8;
/// The most candidates returned.
pub const MAX_LINKS: usize = 6;
/// The fewest characters of a one-word title that counts as a mention.
const MENTION_MIN_CHARS: usize = 6;
/// The line a link is written on.
pub const RELATED_PREFIX: &str = "Related:";

/// A note that could be linked.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkCandidate {
    pub id: String,
    pub title: String,
    /// From 0 to 1.
    pub score: f32,
    /// Whether its title is in the text.
    pub mentioned: bool,
}

/// The lowercase words of `text`: runs of letters and digits.
pub fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether `title` appears in `text_words` as whole words, and is long
/// enough to mean something. Pure.
pub fn mentions(text_words: &[String], title: &str) -> bool {
    let title_words = words(title);
    let long_enough = title_words.len() >= 2
        || title_words
            .first()
            .is_some_and(|w| w.chars().count() >= MENTION_MIN_CHARS);
    long_enough
        && text_words
            .windows(title_words.len())
            .any(|window| window == title_words.as_slice())
}

/// Whether `title` can be written as `[[title]]` and found again.
pub fn linkable(title: &str) -> bool {
    !title.trim().is_empty() && !title.contains(['[', ']', '|', '#', '\n', '\r'])
}

/// The candidates for a note: `text` is its words, `titles` the notes that
/// can be linked (`Store::linkable_titles`), `near` its nearest notes by
/// meaning (most similar first), `linked` the notes it already links to
/// and `own_id` itself. Best first, ties by ID; at most [`MAX_LINKS`].
/// Pure.
pub fn rank(
    text: &str,
    titles: &[(String, String)],
    near: &[RelatedNote],
    linked: &HashSet<String>,
    own_id: &str,
) -> Vec<LinkCandidate> {
    // A title two notes share can't be linked by title.
    let mut count: HashMap<String, usize> = HashMap::new();
    for (_, title) in titles {
        *count.entry(normalize_title(title)).or_default() += 1;
    }
    let usable = |id: &str, title: &str| {
        id != own_id
            && !linked.contains(id)
            && linkable(title)
            && count.get(&normalize_title(title)) == Some(&1)
    };
    let text_words = words(text);
    let mut found: HashMap<String, LinkCandidate> = HashMap::new();
    for (id, title) in titles {
        if usable(id, title) && mentions(&text_words, title) {
            found.insert(
                id.clone(),
                LinkCandidate {
                    id: id.clone(),
                    title: title.clone(),
                    score: MENTION_SCORE,
                    mentioned: true,
                },
            );
        }
    }
    for note in near.iter().take(NEAR_COUNT) {
        if note.score < NEAR_SIMILARITY {
            continue;
        }
        let Some((_, title)) = titles.iter().find(|(id, _)| *id == note.id) else {
            continue;
        };
        if !usable(&note.id, title) {
            continue;
        }
        let score = note.score.clamp(0., 1.);
        found
            .entry(note.id.clone())
            .and_modify(|c| c.score = c.score.max(score))
            .or_insert_with(|| LinkCandidate {
                id: note.id.clone(),
                title: title.clone(),
                score,
                mentioned: false,
            });
    }
    let mut all: Vec<LinkCandidate> = found.into_values().collect();
    all.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    all.truncate(MAX_LINKS);
    all
}

/// The candidates for note `id` whose words are `text`, read from the
/// store; `model` is the embedding model, if related notes are on.
pub fn candidates(
    store: &Store,
    id: &str,
    text: &str,
    model: Option<&str>,
) -> Result<Vec<LinkCandidate>> {
    let titles = store.linkable_titles()?;
    let near = match model {
        Some(model) => store
            .nearest_notes(id, model, NEAR_COUNT)?
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let linked = store.links_from(id)?;
    Ok(rank(text, &titles, &near, &linked, id))
}

/// `body` with `titles` linked on its last line: `Related: [[A]], [[B]]`,
/// added to an existing `Related:` line when the body ends with one. The
/// body before it is untouched. A title that can't be written as a link
/// (it holds `]]`, `|`, `[`, `#` or a line break) and a title given twice
/// are left out. Pure.
pub fn with_related_line(body: &str, titles: &[&str]) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    for title in titles {
        let title = title.trim();
        if !linkable(title) || seen.contains(&normalize_title(title)) {
            continue;
        }
        seen.push(normalize_title(title));
        links.push(format!("[[{title}]]"));
    }
    if links.is_empty() {
        return body.to_owned();
    }
    let trimmed = body.trim_end();
    if trimmed.is_empty() {
        return format!("{RELATED_PREFIX} {}", links.join(", "));
    }
    let last = trimmed.rsplit('\n').next().unwrap_or(trimmed);
    if last.starts_with(RELATED_PREFIX) {
        let new: Vec<&String> = links
            .iter()
            .filter(|l| !last.contains(l.as_str()))
            .collect();
        if new.is_empty() {
            return body.to_owned();
        }
        let joined: Vec<&str> = new.iter().map(|l| l.as_str()).collect();
        return format!("{trimmed}, {}", joined.join(", "));
    }
    format!("{trimmed}\n\n{RELATED_PREFIX} {}", links.join(", "))
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

    fn titles() -> Vec<(String, String)> {
        vec![
            ("note:a".into(), "Flat repairs 2026".into()),
            ("note:b".into(), "Boiler".into()),
            ("note:c".into(), "Onboarding emails".into()),
            ("note:d".into(), "Dup".into()),
            ("note:e".into(), "dup".into()),
            ("note:f".into(), "Broken [title]".into()),
        ]
    }

    fn near(id: &str, score: f32) -> RelatedNote {
        RelatedNote {
            id: id.into(),
            title: String::new(),
            score,
        }
    }

    #[test]
    fn a_title_in_the_text_is_a_mention() {
        let c = rank(
            "Call about the flat repairs 2026 budget, and the boiler.",
            &titles(),
            &[],
            &HashSet::new(),
            "note:x",
        );
        // "Boiler" is one word of six characters: counts.
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|c| c.mentioned && c.score == MENTION_SCORE));
        // Whole words only.
        let c = rank("unboilered", &titles(), &[], &HashSet::new(), "note:x");
        assert!(c.is_empty());
    }

    #[test]
    fn near_notes_need_the_similarity_and_a_unique_title() {
        let near = [
            near("note:c", 0.7),
            near("note:a", 0.5),
            near("note:d", 0.9),
        ];
        let c = rank("", &titles(), &near, &HashSet::new(), "note:x");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].id, "note:c");
        assert!(!c[0].mentioned);
    }

    #[test]
    fn linked_and_own_notes_are_left_out() {
        let near = [near("note:c", 0.7), near("note:x", 0.9)];
        let linked = HashSet::from(["note:c".to_owned()]);
        assert!(rank("", &titles(), &near, &linked, "note:x").is_empty());
    }

    #[test]
    fn the_related_line_skips_titles_that_cannot_be_links() {
        assert_eq!(
            with_related_line("Hi", &["A]]b", "C|d", "E\nF", "Good", "good", "Good "]),
            "Hi\n\nRelated: [[Good]]"
        );
        // Nothing usable: the body is untouched.
        assert_eq!(with_related_line("Hi", &["a]]", "b|c"]), "Hi");
    }

    #[test]
    fn the_related_line_is_added_or_extended() {
        assert_eq!(
            with_related_line("Hello", &["A"]),
            "Hello\n\nRelated: [[A]]"
        );
        assert_eq!(with_related_line("", &["A", "B"]), "Related: [[A]], [[B]]");
        assert_eq!(
            with_related_line("Hello\n\nRelated: [[A]]\n", &["A", "B"]),
            "Hello\n\nRelated: [[A]], [[B]]"
        );
        assert_eq!(with_related_line("Hello", &[]), "Hello");
    }

    proptest! {
        #![proptest_config(config())]

        /// Scores are in range, the list is short and sorted, and no note
        /// that is linked, the note itself or without a usable title is in
        /// it.
        #[test]
        fn candidates_are_clean_and_ranked(
            text in "[a-z ]{0,60}",
            sims in prop::collection::vec(-1f32..1.5, 0..8),
            linked in prop::collection::vec(0usize..6, 0..3),
            own in 0usize..6,
        ) {
            let titles = titles();
            let near: Vec<RelatedNote> = sims
                .iter()
                .enumerate()
                .map(|(i, s)| near(&titles[i % 6].0, *s))
                .collect();
            let linked: HashSet<String> = linked.iter().map(|i| titles[*i].0.clone()).collect();
            let own_id = &titles[own].0;
            let out = rank(&text, &titles, &near, &linked, own_id);
            prop_assert!(out.len() <= MAX_LINKS);
            for c in &out {
                prop_assert!((0. ..=1.).contains(&c.score));
                prop_assert!(!linked.contains(&c.id));
                prop_assert!(&c.id != own_id);
                prop_assert!(linkable(&c.title));
                prop_assert!(c.id != "note:d" && c.id != "note:e" && c.id != "note:f");
            }
            prop_assert!(out.windows(2).all(|w| w[0].score >= w[1].score));
            // Equal input, equal output.
            prop_assert_eq!(out, rank(&text, &titles, &near, &linked, own_id));
        }

        /// The related line only adds to the end of the body.
        #[test]
        fn the_related_line_only_appends(body in "[a-z \n]{0,40}", pick in prop::collection::vec(0usize..3, 0..3)) {
            let names = ["A", "B", "C"];
            let chosen: Vec<&str> = pick.iter().map(|i| names[*i]).collect();
            let out = with_related_line(&body, &chosen);
            if chosen.is_empty() {
                prop_assert_eq!(out, body);
            } else {
                prop_assert!(out.starts_with(body.trim_end()));
                for name in &chosen {
                    let link = format!("[[{name}]]");
                    prop_assert!(out.contains(&link));
                }
            }
        }
    }
}
