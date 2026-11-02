//! Follow-ups: making "and what did Sam think?" a question that can be
//! searched on its own. The local rewrite adds the subject of the question
//! before it, with no model; the model rewrite (a short request that sends
//! earlier questions and answers and no note text) is only for when that
//! finds nothing, and lives in [`super::turn`].

use super::content_words;
use super::model::Previous;

/// Words that point back at something said before.
pub const PRONOUNS: [&str; 10] = [
    "he", "she", "they", "it", "that", "this", "those", "them", "him", "her",
];

/// The most words the earlier question and titles add.
const ADDED: usize = 12;

/// Whether `question` leans on what came before: it has fewer than three
/// words worth searching for, or a pronoun that points back.
pub fn leans_on_before(question: &str) -> bool {
    let words = content_words(question);
    let pronoun = question
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.to_lowercase())
        .any(|w| PRONOUNS.contains(&w.as_str()));
    words.len() < 3 || pronoun
}

/// `query` made stand-alone with the local rule: when it leans on what came
/// before, the search words of the previous question and the titles of the
/// notes it cited are added. Otherwise `query` as it is.
pub fn standalone(query: &str, previous: Option<&Previous>) -> String {
    let Some(previous) = previous else {
        return query.to_owned();
    };
    if !leans_on_before(query) {
        return query.to_owned();
    }
    let mut seen: Vec<String> = content_words(query);
    let mut added = Vec::new();
    let earlier = content_words(&previous.question);
    let titles = previous.titles.iter().flat_map(|t| content_words(t));
    for word in earlier.into_iter().chain(titles) {
        if added.len() >= ADDED {
            break;
        }
        if !seen.contains(&word) {
            seen.push(word.clone());
            added.push(word);
        }
    }
    if added.is_empty() {
        return query.to_owned();
    }
    format!("{} {}", query.trim(), added.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn previous() -> Previous {
        Previous {
            question: "What did we decide about the yearly pricing plan?".into(),
            titles: vec!["Pricing".into(), "Call with Sam".into()],
            cited: vec![],
        }
    }

    #[test]
    fn a_short_follow_up_gets_the_subject_of_the_question_before() {
        let q = standalone("and what did Sam think?", Some(&previous()));
        assert!(q.starts_with("and what did Sam think?"));
        for word in ["decide", "yearly", "pricing", "plan", "call"] {
            assert!(q.contains(word), "{q}");
        }
    }

    #[test]
    fn a_pronoun_pulls_the_subject_in_even_for_a_long_question() {
        let q = standalone(
            "why did she prefer the monthly billing arrangement instead",
            Some(&previous()),
        );
        assert!(q.contains("pricing"), "{q}");
    }

    #[test]
    fn a_stand_alone_question_is_left_alone() {
        let q = "What is the refund policy for annual subscriptions";
        assert_eq!(standalone(q, Some(&previous())), q);
        assert_eq!(standalone("it", None), "it");
    }

    proptest! {
        /// The rewrite only ever adds words from the question before and the
        /// titles it cited: it can't bring in text from anywhere else.
        #[test]
        fn prop_rewrite_sends_no_note_text(
            query in "[a-z ?]{0,60}",
            question in "[a-z ?]{0,60}",
            titles in prop::collection::vec("[A-Za-z ]{0,20}", 0..4),
        ) {
            let before = Previous { question: question.clone(), titles: titles.clone(), cited: vec![] };
            let out = standalone(&query, Some(&before));
            let allowed: std::collections::HashSet<String> = content_words(&query)
                .into_iter()
                .chain(content_words(&question))
                .chain(titles.iter().flat_map(|t| content_words(t)))
                .collect();
            for word in content_words(&out) {
                prop_assert!(allowed.contains(&word), "{word} came from nowhere");
            }
            prop_assert!(out.starts_with(query.trim()) || out == query);
        }
    }
}
