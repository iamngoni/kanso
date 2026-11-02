//! Retrieval end to end over small libraries: scope, time, passages, links,
//! follow-ups and the budget.

use super::*;
use crate::ask::model::{AskScope, Previous};
use kanso_ai::{Budget, BudgetSize};
use kanso_core::DocType;
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use serde_json::json;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store, name: &str, parent: Option<&str>) -> String {
    let mut fields = json!({"name": name});
    if let Some(parent) = parent {
        fields["parentBookId"] = json!(parent);
    }
    store
        .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn lock_book(store: &Store, id: &str) {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

fn note(store: &Store, book: &str, title: &str, body: &str) -> String {
    note_at(store, book, title, body, 1_000, vec![])
}

fn note_at(
    store: &Store,
    book: &str,
    title: &str,
    body: &str,
    at: i64,
    tags: Vec<String>,
) -> String {
    store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book, "tags": tags,
                   "createdAt": at, "updatedAt": at}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn today() -> Date {
    Date::new(2026, 10, 20).unwrap()
}

fn ms(y: i32, m: u32, d: u32) -> i64 {
    Date::new(y, m, d).unwrap().to_days() * 86_400_000 + 12 * 3_600_000
}

fn ask(store: &Store, question: &str) -> Retrieval {
    let request = AskRequest::new(question, today());
    retrieve_with(store, &request).expect("retrieval runs")
}

fn ids(found: &Retrieval) -> Vec<&str> {
    found.excerpts.iter().map(|e| e.id.as_str()).collect()
}

#[test]
fn an_excerpt_carries_its_note_lines_exactly() {
    let (_dir, store) = open();
    let b = book(&store, "Work", None);
    let body = "# Pricing\nWe chose the yearly plan.\n40 dollars a year.\n\n# Other\nUnrelated.";
    let id = note(&store, &b, "Pricing", body);
    let found = ask(&store, "yearly plan pricing");
    let excerpt = found.excerpts.iter().find(|e| e.id == id).unwrap();
    let all: Vec<&str> = body.split('\n').collect();
    assert!(!excerpt.lines.is_empty());
    for (n, text) in &excerpt.lines {
        assert_eq!(text, all[*n as usize], "line {n}");
    }
    assert_eq!(excerpt.notebook.as_deref(), Some("Work"));
    assert_eq!(excerpt.updated.as_deref(), Some("1970-01-01"));
}

#[test]
fn the_tail_of_a_long_note_is_found_as_a_passage() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let mut body = String::new();
    for ix in 0..30 {
        body.push_str(&format!(
            "# Part {ix}\n{}\n\n",
            "filler words here ".repeat(30)
        ));
    }
    body.push_str("# Decision\nThe quokka launch is on 14 March.\n");
    let id = note(&store, &b, "Long", &body);
    let found = ask(&store, "when is the quokka launch");
    let excerpt = found.excerpts.iter().find(|e| e.id == id).expect("found");
    let last = body.split('\n').count() as u32 - 2;
    assert!(
        excerpt
            .lines
            .iter()
            .any(|(n, text)| *n == last && text.contains("quokka")),
        "{:?}",
        excerpt.lines
    );
    // Only the passage that matches goes, not the thirty filler sections.
    assert!(excerpt.lines.len() < 10, "{}", excerpt.lines.len());
}

#[test]
fn a_notebook_scope_includes_the_notebooks_inside_it_and_nothing_else() {
    let (_dir, store) = open();
    let work = book(&store, "Work", None);
    let inner = book(&store, "Plans", Some(&work));
    let home = book(&store, "Home", None);
    let a = note(&store, &work, "Pricing work", "The pricing plan is yearly.");
    let b = note(
        &store,
        &inner,
        "Pricing plans",
        "Another pricing plan, monthly.",
    );
    let c = note(&store, &home, "Pricing home", "The household pricing plan.");
    let mut request = AskRequest::new("pricing plan", today());
    request.scope = AskScope::Book(work.clone());
    let found = retrieve_with(&store, &request).unwrap();
    let got = ids(&found);
    assert!(
        got.contains(&a.as_str()) && got.contains(&b.as_str()),
        "{got:?}"
    );
    assert!(!got.contains(&c.as_str()), "{got:?}");
}

#[test]
fn a_tag_scope_and_a_notes_scope_narrow_the_search() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let tag = store
        .put(
            DocType::Tag,
            json!({"name": "Launch"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let tagged = note_at(
        &store,
        &b,
        "Plan A",
        "The pricing plan A.",
        1_000,
        vec![tag],
    );
    let other = note(&store, &b, "Plan B", "The pricing plan B.");
    let mut request = AskRequest::new("pricing plan", today());
    request.scope = AskScope::Tag("#launch".into());
    let found = retrieve_with(&store, &request).unwrap();
    assert_eq!(ids(&found), [tagged.as_str()]);
    request.scope = AskScope::Notes(vec![other.clone()]);
    let found = retrieve_with(&store, &request).unwrap();
    assert_eq!(ids(&found), [other.as_str()]);
}

#[test]
fn a_locked_notebook_scope_is_refused_before_anything_runs() {
    let (_dir, store) = open();
    let secret = book(&store, "Secret", None);
    note(
        &store,
        &secret,
        "Secret pricing",
        "The secret pricing plan is 99.",
    );
    lock_book(&store, &secret);
    let mut request = AskRequest::new("pricing plan", today());
    request.scope = AskScope::Book(secret);
    match retrieve_with(&store, &request) {
        Err(AskError::Scope(refusal)) => {
            assert_eq!(refusal, ScopeRefusal::Locked);
            assert!(refusal.message().contains("Ask never reads locked notes"));
        }
        other => panic!("expected a refusal, got {:?}", other.map(|r| r.query)),
    }
}

#[test]
fn a_written_time_keeps_to_notes_from_then_and_says_when_it_relaxes() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let oct1 = note_at(
        &store,
        &b,
        "Pricing call",
        "We decided the pricing is yearly.",
        ms(2026, 10, 3),
        vec![],
    );
    let oct2 = note_at(
        &store,
        &b,
        "Pricing review",
        "We decided the pricing stays yearly.",
        ms(2026, 10, 12),
        vec![],
    );
    let sep = note_at(
        &store,
        &b,
        "Pricing early",
        "We decided the pricing is monthly.",
        ms(2026, 9, 2),
        vec![],
    );
    let found = ask(&store, "what did we decide about pricing in October");
    let got = ids(&found);
    assert!(
        got.contains(&oct1.as_str()) && got.contains(&oct2.as_str()),
        "{got:?}"
    );
    assert!(!got.contains(&sep.as_str()), "{got:?}");
    let time = found.time.expect("the time was understood");
    assert_eq!(time.label, "October 2026");
    assert!(!time.relaxed);
    assert!(!found.query.to_lowercase().contains("october"));

    // Nothing written in March: the closest notes, said plainly.
    let found = ask(&store, "what did we decide about pricing in March");
    let time = found.time.clone().expect("understood");
    assert!(time.relaxed, "{found:?}");
    assert_eq!(
        time.relaxed_line().unwrap(),
        "No notes written in March 2026 matched; showing the closest."
    );
    assert!(!found.excerpts.is_empty());
}

#[test]
fn dropping_the_time_searches_the_question_as_it_was() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let sep = note_at(
        &store,
        &b,
        "Pricing early",
        "The pricing is monthly.",
        ms(2026, 9, 2),
        vec![],
    );
    let mut request = AskRequest::new("pricing in October", today());
    request.ignore_time = true;
    let found = retrieve_with(&store, &request).unwrap();
    assert!(found.time.is_none());
    assert!(ids(&found).contains(&sep.as_str()));
}

#[test]
fn a_question_that_is_only_a_time_lists_what_was_written_then() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let last_week = note_at(
        &store,
        &b,
        "Monday",
        "Met the team about hiring.",
        ms(2026, 10, 13),
        vec![],
    );
    let old = note_at(
        &store,
        &b,
        "Old",
        "Something from August.",
        ms(2026, 8, 1),
        vec![],
    );
    let found = ask(&store, "what did I write last week");
    assert_eq!(ids(&found), [last_week.as_str()], "{found:?}");
    assert!(!ids(&found).contains(&old.as_str()));
}

#[test]
fn an_about_time_finds_dates_written_in_the_text_and_task_lines() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let due = note(
        &store,
        &b,
        "Launch list",
        "# Tasks\n- [ ] Write the press release @2026-11-02\n- [ ] Book venue @2027-02-01\n",
    );
    let other = note(&store, &b, "Other list", "- [ ] Water plants @2027-03-01\n");
    let found = ask(&store, "what is due before 5 November");
    assert_eq!(
        found.time.as_ref().map(|t| t.label.as_str()),
        Some("Until 4 November 2026")
    );
    assert!(ids(&found).contains(&due.as_str()), "{found:?}");
    assert!(!ids(&found).contains(&other.as_str()));
    // The task's own line is among the lines sent.
    let excerpt = found.excerpts.iter().find(|e| e.id == due).unwrap();
    assert!(
        excerpt
            .lines
            .iter()
            .any(|(_, t)| t.contains("press release"))
    );
}

#[test]
fn a_linked_note_that_matches_a_word_comes_in_with_the_best_note() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let meeting = note(
        &store,
        &b,
        "Meeting 3",
        "Talked about the Atlas rollout and who owns it.",
    );
    let project = note(
        &store,
        &b,
        "Atlas",
        &format!(
            "The Atlas project, see inkdrop://note/{}.\n",
            meeting.trim_start_matches("note:")
        ),
    );
    let unrelated = note(
        &store,
        &b,
        "Garden",
        "Nothing to do with it, no word shared.",
    );
    // The question matches the project note by title; the meeting only
    // through the link and the word rollout.
    let found = ask(&store, "Atlas rollout");
    let got = ids(&found);
    assert!(got.contains(&project.as_str()), "{got:?}");
    assert!(got.contains(&meeting.as_str()), "{got:?}");
    assert!(!got.contains(&unrelated.as_str()));
    // With links off the same question still finds both by their words.
    let mut request = AskRequest::new("Atlas rollout", today());
    request.use_links = false;
    let found = retrieve_with(&store, &request).unwrap();
    assert!(ids(&found).contains(&project.as_str()));
}

#[test]
fn a_follow_up_keeps_the_subject_and_the_lines_the_answer_cited() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let pricing = note(
        &store,
        &b,
        "Pricing",
        "We chose the yearly plan at 40 dollars.\nSam argued the monthly plan churned.",
    );
    note(&store, &b, "Garden", "Sam planted tomatoes.");
    let mut request = AskRequest::new("and what did Sam think?", today());
    request.previous = Some(Previous {
        question: "What did we decide about the yearly pricing plan?".into(),
        titles: vec!["Pricing".into()],
        cited: vec![(pricing.clone(), 0, 0)],
    });
    let found = retrieve_with(&store, &request).unwrap();
    assert_eq!(
        ids(&found).first().copied(),
        Some(pricing.as_str()),
        "{found:?}"
    );
    let excerpt = &found.excerpts[0];
    // The cited line is among the lines, so it can be cited again.
    assert!(excerpt.lines.iter().any(|(n, _)| *n == 0));
    // Alone, the follow-up finds the garden note and not the pricing one.
    let alone =
        retrieve_with(&store, &AskRequest::new("and what did Sam think?", today())).unwrap();
    assert!(!ids(&alone).contains(&pricing.as_str()) || ids(&alone).len() > 1);
}

#[test]
fn locked_and_excluded_notes_never_come_in_through_any_list() {
    let (_dir, store) = open();
    let b = book(&store, "Notes", None);
    let locked_book = book(&store, "Vault", None);
    let exclude = store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let kept = note(
        &store,
        &b,
        "Pricing",
        "The pricing plan is yearly. See the vault.",
    );
    let secret = note(
        &store,
        &locked_book,
        "Vault pricing",
        "The vault pricing plan is 99.",
    );
    let hidden = note_at(
        &store,
        &b,
        "Hidden pricing",
        "The hidden pricing plan is 77.",
        1_000,
        vec![exclude],
    );
    lock_book(&store, &locked_book);
    store
        .put(
            DocType::Note,
            json!({"_id": kept, "body": format!("The pricing plan is yearly. [v](inkdrop://note/{}) [h](inkdrop://note/{})", secret.trim_start_matches("note:"), hidden.trim_start_matches("note:"))}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    for question in [
        "pricing plan",
        "what is the vault pricing",
        "hidden pricing 77",
    ] {
        let found = ask(&store, question);
        for excerpt in &found.excerpts {
            assert!(
                excerpt.id != secret && excerpt.id != hidden,
                "{question}: {}",
                excerpt.id
            );
            assert!(!excerpt.text.contains("99") && !excerpt.text.contains("77"));
        }
        for closest in &found.closest {
            assert!(closest.note_id != secret && closest.note_id != hidden);
        }
    }
}

/// Counts letters: texts with the same letters point the same way.
struct Letters;

impl kanso_store::Embedder for Letters {
    fn model(&self) -> &str {
        "letters"
    }
    fn dim(&self) -> usize {
        26
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                let mut v = vec![0f32; 26];
                for c in t.to_ascii_lowercase().bytes() {
                    if c.is_ascii_lowercase() {
                        v[(c - b'a') as usize] += 1.;
                    }
                }
                v
            })
            .collect()
    }
}

#[test]
fn a_passage_vector_finds_the_tail_of_a_long_note_by_meaning_alone() {
    use kanso_store::Embedder as _;
    let (_dir, store) = open();
    store.set_embedding_model(Some("letters")).unwrap();
    store.lend_embedder(Some(std::sync::Arc::new(Letters)));
    let b = book(&store, "Notes", None);
    let mut body = String::new();
    for ix in 0..12 {
        body.push_str(&format!(
            "# Part {ix}\n{}\n\n",
            "filler words here ".repeat(30)
        ));
    }
    body.push_str("# Last\nzoo zoo zoo\n");
    let id = note(&store, &b, "Long note", &body);
    // Embed the notes, then the passages, as the indexer does.
    let queue = store.embedding_queue("letters").unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|p| {
            let v = Letters.embed(&[p.text.clone()]).remove(0);
            (p.id, p.hash, v)
        })
        .collect();
    store.put_embeddings("letters", &rows).unwrap();
    let passages = store.passage_queue("letters", 100).unwrap();
    let lines: Vec<_> = passages
        .pending
        .into_iter()
        .map(|p| kanso_store::EmbeddedLines {
            vector: Letters.embed(&[p.text.clone()]).remove(0),
            note_id: p.note_id,
            start: p.start,
            end: p.end,
            hash: p.hash,
        })
        .collect();
    store.put_passage_embeddings("letters", &lines).unwrap();
    // No word of the question is in the note; only the letters are.
    let found = ask(&store, "ozo ozo ozo");
    let excerpt = found
        .excerpts
        .iter()
        .find(|e| e.id == id)
        .expect("found by meaning");
    assert!(
        excerpt
            .lines
            .iter()
            .any(|(_, text)| text.contains("zoo zoo zoo")),
        "{:?}",
        excerpt.lines
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// No result is ever outside the scope, whatever the library and scope.
    #[test]
    fn prop_no_result_outside_the_scope(
        placements in prop::collection::vec((0usize..3, any::<bool>()), 1..10),
        scope_kind in 0usize..4,
        scope_book in 0usize..3,
    ) {
        let (_dir, store) = open();
        let books = [
            book(&store, "A", None),
            book(&store, "B", None),
        ];
        let child = book(&store, "A child", Some(&books[0]));
        let all_books = [books[0].clone(), books[1].clone(), child.clone()];
        let tag = store
            .put(DocType::Tag, json!({"name": "pick"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let mut created = Vec::new();
        for (ix, (book_ix, tagged)) in placements.iter().enumerate() {
            let tags = if *tagged { vec![tag.clone()] } else { vec![] };
            let id = note_at(
                &store,
                &all_books[*book_ix],
                &format!("Plan {ix}"),
                "The zebra plan is here.",
                1_000 + ix as i64,
                tags,
            );
            created.push((id, *book_ix, *tagged));
        }
        let scope = match scope_kind {
            0 => AskScope::All,
            1 => AskScope::Book(all_books[scope_book].clone()),
            2 => AskScope::Tag("pick".into()),
            _ => AskScope::Notes(created.iter().step_by(2).map(|(id, _, _)| id.clone()).collect()),
        };
        let mut request = AskRequest::new("zebra plan", today());
        request.scope = scope.clone();
        let found = retrieve_with(&store, &request).unwrap();
        for excerpt in &found.excerpts {
            let (_, book_ix, tagged) = created.iter().find(|(id, _, _)| *id == excerpt.id).unwrap();
            let inside = match &scope {
                AskScope::All => true,
                AskScope::Book(b) => {
                    // A is the parent of its child.
                    let this = &all_books[*book_ix];
                    this == b || (*b == all_books[0] && this == &child)
                }
                AskScope::Tag(_) => *tagged,
                AskScope::Notes(ids) => ids.contains(&excerpt.id),
            };
            prop_assert!(inside, "{} is outside {:?}", excerpt.id, scope);
        }
    }

    /// The prompt never goes past its budget, at any of the three sizes.
    #[test]
    fn prop_prompt_never_exceeds_budget(
        size in 0usize..3,
        bodies in prop::collection::vec("[a-z \n]{50,3000}", 1..8),
    ) {
        let (_dir, store) = open();
        let b = book(&store, "Notes", None);
        for (ix, body) in bodies.iter().enumerate() {
            note(&store, &b, &format!("Note {ix}"), &format!("plan {body}"));
        }
        let mut request = AskRequest::new("plan", today());
        request.budget = Budget::of(BudgetSize::ALL[size]);
        let prepared = crate::ask::prepare(&store, &request).unwrap();
        prop_assert!(prepared.numbered.prompt.chars() <= request.budget.excerpt_chars);
    }
}
