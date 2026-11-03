//! The Ask evaluation: retrieval over a fixed synthetic library of about 280
//! notes in 12 notebooks, and the receipt checker over fixed answers.
//!
//! The bait rule is a hard failure: no locked or `#ai-exclude` note is ever
//! retrieved. The recall numbers are printed and held to a floor, so a change
//! that makes retrieval worse fails here. Regenerate the fixtures with
//! `python3 scripts/ask_eval_library.py`.

#[path = "ask_eval/support.rs"]
mod support;

use std::collections::{BTreeMap, HashMap};

use kanso_engine::ask::{
    AskRequest, AskScope, Banner, Previous, Shown, ShownExcerpt, finalize, retrieve_with,
};
use serde::Deserialize;
use support::*;

/// Whether `excerpts` hold a line inside the spot.
fn covers(loaded: &Loaded, excerpts: &[kanso_ai::Excerpt], spot: &Spot) -> bool {
    let Some(id) = loaded.ids.get(&spot.title) else {
        return false;
    };
    excerpts.iter().filter(|e| &e.id == id).any(|e| {
        e.lines
            .iter()
            .any(|(n, _)| *n >= spot.start && *n <= spot.end)
    })
}

fn request(loaded: &Loaded, question: &Question, previous: Option<Previous>) -> AskRequest {
    let mut request = AskRequest::new(&question.question, loaded.today);
    if let Some(scope) = &question.scope {
        request.scope = AskScope::Book(loaded.books[&scope.book].clone());
    }
    request.previous = previous;
    request
}

#[test]
fn retrieval_finds_the_lines_and_never_the_bait() {
    let loaded = load();
    let questions = questions();
    assert!(questions.len() >= 70, "{} questions", questions.len());
    let by_id: HashMap<&str, &Question> = questions.iter().map(|q| (q.id.as_str(), q)).collect();
    let bait_ids: Vec<String> = ["Vault passwords", "Board salary plan", "Acquisition rumour"]
        .iter()
        .map(|t| loaded.ids[*t].clone())
        .collect();

    // category -> (found, expected)
    let mut table: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut misses = Vec::new();
    for question in &questions {
        let previous = question.after.as_ref().map(|parent| {
            let parent = by_id[parent.as_str()];
            Previous {
                question: parent.question.clone(),
                titles: parent.expected.iter().map(|s| s.title.clone()).collect(),
                cited: parent
                    .expected
                    .iter()
                    .map(|s| (loaded.ids[&s.title].clone(), s.start, s.end))
                    .collect(),
            }
        });
        let found = retrieve_with(&loaded.store, &request(&loaded, question, previous))
            .expect("retrieval runs");
        // Hard rule: nothing from a locked or excluded note, ever.
        for excerpt in &found.excerpts {
            assert!(
                !bait_ids.contains(&excerpt.id),
                "{}: {} was retrieved",
                question.id,
                excerpt.title
            );
        }
        for closest in &found.closest {
            assert!(
                !bait_ids.contains(&closest.note_id),
                "{}: bait in closest",
                question.id
            );
        }
        let entry = table.entry(question.category.clone()).or_default();
        for spot in &question.expected {
            entry.1 += 1;
            if covers(&loaded, &found.excerpts, spot) {
                entry.0 += 1;
            } else {
                misses.push(format!("{} -> {}", question.id, spot.title));
            }
        }
    }
    let (mut found_all, mut expected_all) = (0, 0);
    println!(
        "{:<14} {:>6} {:>6} {:>7}",
        "category", "found", "of", "recall"
    );
    for (category, (found, expected)) in &table {
        println!(
            "{category:<14} {found:>6} {expected:>6} {:>6.0}%",
            100.0 * *found as f64 / (*expected).max(1) as f64
        );
        found_all += found;
        expected_all += expected;
    }
    let recall = found_all as f64 / expected_all as f64;
    println!(
        "overall recall {:.1}% ({found_all}/{expected_all})",
        recall * 100.0
    );
    for miss in &misses {
        println!("missed: {miss}");
    }
    // The floors are what the retrieval does today, kept so it cannot get
    // worse. Synthesis questions name no shared word with their notes, so the
    // model's own tool calls make up the rest.
    assert!(recall >= 0.75, "recall fell to {:.1}%", recall * 100.0);
    let (lookups, of) = table["lookup"];
    assert!(lookups as f64 / of as f64 >= 0.9, "lookup recall fell");
}

#[test]
fn a_scoped_question_stays_in_its_notebook() {
    let loaded = load();
    let book = &loaded.books["Meetings"];
    let mut request = AskRequest::new("what did Sam think about the plan", loaded.today);
    request.scope = AskScope::Book(book.clone());
    let found = retrieve_with(&loaded.store, &request).unwrap();
    assert!(!found.excerpts.is_empty());
    for excerpt in &found.excerpts {
        let note = loaded.store.note(&excerpt.id).unwrap().unwrap();
        assert_eq!(&note.book_id, book, "{} is outside Meetings", excerpt.title);
    }
}

#[derive(Deserialize)]
struct AnswerCase {
    file: String,
    /// What was shown: note title and the 0-based lines.
    shown: Vec<Spot>,
    valid: usize,
    unsupported: usize,
    #[serde(default)]
    not_in_notes: bool,
    #[serde(default)]
    no_sources: bool,
}

#[test]
fn the_checker_flags_what_the_answers_do_not_support() {
    let loaded = load();
    let library = library();
    let cases: Vec<AnswerCase> =
        serde_json::from_str(include_str!("ask_eval/answers.json")).expect("answers.json");
    assert!(cases.len() >= 10);
    for case in cases {
        let answer = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/ask_eval/answers")
                .join(&case.file),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case.file));
        let shown = Shown::new();
        for (ix, spot) in case.shown.iter().enumerate() {
            let note = library
                .notes
                .iter()
                .find(|n| n.title == spot.title)
                .unwrap();
            let all: Vec<&str> = note.body.split('\n').collect();
            shown.register(ShownExcerpt {
                number: ix + 1,
                id: loaded.ids[&spot.title].clone(),
                title: spot.title.clone(),
                updated: None,
                lines: (spot.start..=spot.end)
                    .map(|n| (n, all[n as usize].to_owned()))
                    .collect(),
            });
        }
        let done = finalize(&answer, &shown, true, &Default::default());
        assert_eq!(done.receipts.len(), case.valid, "{}: valid", case.file);
        assert_eq!(
            done.unverified, case.unsupported,
            "{}: unsupported",
            case.file
        );
        assert_eq!(done.not_in_notes, case.not_in_notes, "{}", case.file);
        assert_eq!(
            done.banner == Some(Banner::NoSources),
            case.no_sources,
            "{}",
            case.file
        );
    }
}
