//! Refining a card with the person's own AI. One call per card: the model
//! ranks the library's candidates and suggests a title ([`kanso_ai::triage`]);
//! it never invents a notebook, tag or link. Every AI chip carries the
//! notes its candidate came from, and one without a cited note is not
//! shown. A note that is locked, trashed or tagged `#ai-exclude` is never
//! sent.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_ai::ChatProvider;
use kanso_ai::triage::{Ask, Candidate, Reply};
use kanso_store::Store;

use super::model::{Card, Field, Source, Suggestion, TriageOptions};
use crate::ask::note_is_eligible;

/// Refused before anything is sent.
pub const KEPT_OUT: &str = "This note is kept out of AI.";

fn candidates_of(card: &Card, field: Field) -> Vec<Candidate> {
    card.candidates
        .iter()
        .filter(|s| s.field == field)
        .map(|s| Candidate {
            id: s.value.clone(),
            name: s.label.clone(),
        })
        .collect()
}

/// The suggestions once the model's `reply` is read: its choices replace
/// the library's for notebook, tags, links and title, where it made one;
/// each carries the notes of the candidate it chose ([`Source::Ai`]). A
/// chip with no cited note is dropped, and so is one `rejected` says the
/// person turned down. Pure.
pub fn apply_reply(
    card: &Card,
    reply: &Reply,
    opts: &TriageOptions,
    rejected: impl Fn(&Suggestion) -> bool,
) -> Vec<Suggestion> {
    let pick = |field: Field, id: &str| -> Option<Suggestion> {
        let found = card
            .candidates
            .iter()
            .find(|s| s.field == field && s.value == id)?;
        (!found.because.is_empty()).then(|| Suggestion {
            source: Source::Ai,
            ..found.clone()
        })
    };
    let chosen = |field: Field, ids: &[String]| -> Vec<Suggestion> {
        ids.iter()
            .filter_map(|id| pick(field, id))
            .filter(|s| !rejected(s))
            .collect()
    };
    let library = |field: Field| -> Vec<Suggestion> {
        card.suggestions
            .iter()
            .filter(|s| s.field == field)
            .cloned()
            .collect()
    };
    let mut out = Vec::new();
    let book = chosen(
        Field::Notebook,
        &reply.notebook.iter().cloned().collect::<Vec<_>>(),
    );
    out.extend(if book.is_empty() {
        library(Field::Notebook)
    } else {
        book
    });
    let tags = chosen(Field::Tag, &reply.tags);
    out.extend(if tags.is_empty() {
        library(Field::Tag)
    } else {
        tags.into_iter().take(opts.max_tags).collect()
    });
    let title = reply.title.as_ref().and_then(|t| {
        let s = Suggestion {
            field: Field::Title,
            value: t.clone(),
            label: t.clone(),
            score: 0.7,
            // The title is drawn from the note itself.
            because: vec![card.note_id.clone()],
            source: Source::Ai,
            is_new: false,
        };
        (!rejected(&s)).then_some(s)
    });
    out.extend(title.map_or_else(|| library(Field::Title), |t| vec![t]));
    let links = chosen(Field::Link, &reply.links);
    out.extend(if links.is_empty() {
        library(Field::Link)
    } else {
        links
    });
    out
}

/// Refines `card` (for the note whose title is `title` and words `text`)
/// with `provider`. The error is a plain reason, for "AI suggestions
/// unavailable: <reason>"; the card keeps its library suggestions then.
pub fn refine(
    store: &Store,
    provider: &dyn ChatProvider,
    card: &Card,
    title: &str,
    text: &str,
    opts: &TriageOptions,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<Suggestion>, String> {
    if !note_is_eligible(store, &card.note_id) {
        return Err(KEPT_OUT.to_owned());
    }
    let notebooks = candidates_of(card, Field::Notebook);
    let tags = candidates_of(card, Field::Tag);
    let links = candidates_of(card, Field::Link);
    if notebooks.is_empty() && tags.is_empty() && links.is_empty() && text.trim().is_empty() {
        return Ok(card.suggestions.clone());
    }
    let reply = kanso_ai::triage::triage(
        provider,
        &Ask {
            title,
            text,
            notebooks: &notebooks,
            tags: &tags,
            links: &links,
        },
        cancel,
    )?;
    let dismissed = store
        .triage_dismissed(&card.note_id)
        .map_err(|e| e.to_string())?;
    Ok(apply_reply(card, &reply, opts, |s| {
        dismissed
            .iter()
            .any(|(f, v)| f == s.field.as_str() && *v == s.value)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn s(field: Field, value: &str, because: &[&str]) -> Suggestion {
        Suggestion {
            field,
            value: value.into(),
            label: value.into(),
            score: 0.5,
            because: because.iter().map(|b| (*b).to_owned()).collect(),
            source: Source::Library,
            is_new: false,
        }
    }

    fn card() -> Card {
        let candidates = vec![
            s(Field::Notebook, "b1", &["n1"]),
            s(Field::Notebook, "b2", &["n2"]),
            s(Field::Tag, "t1", &["n1"]),
            s(Field::Tag, "t2", &[]),
            s(Field::Link, "n9", &["n9"]),
        ];
        Card {
            note_id: "note:x".into(),
            rev: "1-a".into(),
            suggestions: vec![candidates[0].clone(), candidates[2].clone()],
            candidates,
            missing: None,
        }
    }

    #[test]
    fn the_model_ranks_among_candidates_and_cites_them() {
        let reply = Reply {
            notebook: Some("b2".into()),
            tags: vec!["t1".into(), "t2".into()],
            links: vec![],
            title: Some("A title".into()),
        };
        let got = apply_reply(&card(), &reply, &TriageOptions::new("inbox"), |_| false);
        let book = got.iter().find(|s| s.field == Field::Notebook).unwrap();
        assert_eq!((book.value.as_str(), book.source), ("b2", Source::Ai));
        // t2 has no cited note, so it is not shown.
        let tags: Vec<_> = got.iter().filter(|s| s.field == Field::Tag).collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].because, ["n1"]);
        let title = got.iter().find(|s| s.field == Field::Title).unwrap();
        assert_eq!(title.because, ["note:x"]);
        // Link: the model chose none, so the library's (none) stays.
        assert!(got.iter().all(|s| s.field != Field::Link));
    }

    #[test]
    fn a_silent_model_keeps_the_library() {
        let got = apply_reply(
            &card(),
            &Reply::default(),
            &TriageOptions::new("inbox"),
            |_| false,
        );
        assert_eq!(got, card().suggestions);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Every AI chip is a candidate and cites at least one note.
        #[test]
        fn ai_chips_are_candidates_with_notes(
            nb in prop::option::of("[a-d][0-9]?"),
            tags in prop::collection::vec("[a-d][0-9]?", 0..5),
            links in prop::collection::vec("[a-d][0-9]?", 0..5),
        ) {
            let card = card();
            let reply = Reply { notebook: nb, tags, links, title: None };
            for chip in apply_reply(&card, &reply, &TriageOptions::new("inbox"), |_| false) {
                if chip.source == Source::Ai {
                    prop_assert!(!chip.because.is_empty());
                    prop_assert!(card.candidates.iter().any(|c| c.field == chip.field && c.value == chip.value));
                }
            }
        }
    }
}
