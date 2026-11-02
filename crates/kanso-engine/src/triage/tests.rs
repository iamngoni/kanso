//! Triage against a real store, with a model that counts topic words.

use std::collections::HashSet;

use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_store::{Embedder, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use super::*;

/// Counts topic words: notes about the same topic point the same way, and
/// notes about none have no direction (and so are never near anything).
struct Topics;

const WORDS: [&str; 8] = [
    "plumber", "boiler", "leak", "pipe", "meeting", "budget", "sales", "client",
];

impl Embedder for Topics {
    fn model(&self) -> &str {
        "topics"
    }
    fn dim(&self) -> usize {
        WORDS.len()
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                let t = t.to_lowercase();
                WORDS.iter().map(|w| t.matches(w).count() as f32).collect()
            })
            .collect()
    }
}

struct Lib {
    _dir: tempfile::TempDir,
    store: Store,
    inbox: String,
    home: String,
    work: String,
}

fn book(store: &Store, id: &str, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": name}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn lib() -> Lib {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let home = book(&store, "book:home", "Home");
    let work = book(&store, "book:work", "Work");
    Lib {
        _dir: dir,
        store,
        inbox,
        home,
        work,
    }
}

fn note(store: &Store, book: &str, title: &str, body: &str, tags: &[&str]) -> String {
    let tags: Vec<String> = tags
        .iter()
        .map(|t| store.tag_named(t, Origin::Ui).unwrap())
        .collect();
    store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book, "tags": tags}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn index(store: &Store) {
    store.set_embedding_model(Some("topics")).unwrap();
    let queue = store.embedding_queue("topics").unwrap();
    let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
    let vectors = Topics.embed(&texts);
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .zip(vectors)
        .map(|(p, v)| (p.id, p.hash, v))
        .collect();
    store.put_embeddings("topics", &rows).unwrap();
}

/// A library with three plumbing notes in Home (tagged #house), two sales
/// notes in Work, and a plumbing note waiting in the Inbox.
fn filled() -> (Lib, String) {
    let l = lib();
    note(
        &l.store,
        &l.home,
        "Boiler service",
        "boiler plumber pipe",
        &["house"],
    );
    note(
        &l.store,
        &l.home,
        "Flat repairs 2026",
        "leak pipe plumber",
        &["house"],
    );
    note(
        &l.store,
        &l.home,
        "Bathroom",
        "pipe leak boiler",
        &["house", "todo"],
    );
    note(
        &l.store,
        &l.work,
        "Sales plan",
        "sales client budget",
        &["work"],
    );
    note(&l.store, &l.work, "Meeting", "meeting client", &["work"]);
    let waiting = note(
        &l.store,
        &l.inbox,
        "Untitled",
        "Call the plumber about the leak, boiler and pipe. See flat repairs 2026 too.",
        &[],
    );
    index(&l.store);
    (l, waiting)
}

fn opts(l: &Lib) -> TriageOptions {
    TriageOptions::new(l.inbox.clone())
}

fn of(s: &[Suggestion], field: Field) -> Vec<&Suggestion> {
    s.iter().filter(|s| s.field == field).collect()
}

#[test]
fn a_plumbing_note_is_suggested_home_house_a_title_and_a_link() {
    let (l, waiting) = filled();
    let got = suggest(&l.store, &waiting, &opts(&l)).unwrap();
    let book = of(&got, Field::Notebook);
    assert_eq!(book.len(), 1);
    assert_eq!(book[0].value, l.home);
    assert_eq!(book[0].label, "Home");
    assert!(!book[0].because.is_empty() && book[0].because.len() <= BECAUSE_MAX);
    let tags = of(&got, Field::Tag);
    assert_eq!(tags[0].label, "house");
    assert!(tags.len() <= 3);
    let title = of(&got, Field::Title);
    assert_eq!(
        title[0].value,
        "Call the plumber about the leak, boiler and pipe"
    );
    let links = of(&got, Field::Link);
    assert!(links.iter().any(|l| l.label == "Flat repairs 2026"));
    for s in &got {
        assert!((0. ..=1.).contains(&s.score), "{s:?}");
        assert_eq!(s.source, Source::Library);
    }
}

#[test]
fn a_rejection_is_not_suggested_again() {
    let (l, waiting) = filled();
    let got = suggest(&l.store, &waiting, &opts(&l)).unwrap();
    let book = of(&got, Field::Notebook)[0].clone();
    reject(&l.store, &waiting, &book, 5).unwrap();
    let again = suggest(&l.store, &waiting, &opts(&l)).unwrap();
    assert!(of(&again, Field::Notebook).is_empty());
    unreject(&l.store, &waiting, &book).unwrap();
    assert_eq!(
        of(
            &suggest(&l.store, &waiting, &opts(&l)).unwrap(),
            Field::Notebook
        )
        .len(),
        1
    );
}

#[test]
fn a_mixed_neighbourhood_suggests_no_notebook() {
    let l = lib();
    note(&l.store, &l.home, "A", "plumber pipe", &[]);
    note(&l.store, &l.work, "B", "plumber pipe", &[]);
    let waiting = note(&l.store, &l.inbox, "Idea", "plumber pipe", &[]);
    index(&l.store);
    let got = suggest(&l.store, &waiting, &opts(&l)).unwrap();
    assert!(of(&got, Field::Notebook).is_empty());
}

#[test]
fn with_related_notes_off_only_titles_and_links_come() {
    let l = lib();
    note(&l.store, &l.home, "Flat repairs 2026", "x", &[]);
    let waiting = note(
        &l.store,
        &l.inbox,
        "",
        "Look at flat repairs 2026 soon.",
        &[],
    );
    let card = card(&l.store, &waiting, &opts(&l)).unwrap().unwrap();
    assert_eq!(card.missing, Some(Reason::RelatedOff));
    assert!(of(&card.suggestions, Field::Notebook).is_empty());
    assert!(of(&card.suggestions, Field::Tag).is_empty());
    assert_eq!(of(&card.suggestions, Field::Link).len(), 1);
    assert_eq!(of(&card.suggestions, Field::Title).len(), 1);
}

#[test]
fn settings_turn_titles_links_and_tags_down() {
    let (l, waiting) = filled();
    let mut o = opts(&l);
    o.suggest_titles = false;
    o.suggest_links = false;
    o.max_tags = 1;
    let got = suggest(&l.store, &waiting, &o).unwrap();
    assert!(of(&got, Field::Title).is_empty() && of(&got, Field::Link).is_empty());
    assert_eq!(of(&got, Field::Tag).len(), 1);
}

fn new_tag(note_id: &str, rev: &str, name: &str) -> Filing {
    Filing {
        note_id: note_id.into(),
        rev: rev.into(),
        accepted: vec![Suggestion {
            field: Field::Tag,
            value: name.into(),
            label: name.into(),
            score: 0.5,
            because: vec![],
            source: Source::Library,
            is_new: true,
        }],
    }
}

#[test]
fn undo_stamps_the_time_and_takes_back_the_tags_it_made() {
    let (l, waiting) = filled();
    let rev = l.store.note(&waiting).unwrap().unwrap().rev.unwrap();
    let before = l.store.note(&waiting).unwrap().unwrap().updated_at;
    let filed = file::file(&l.store, &[new_tag(&waiting, &rev, "freshly-made")], 5_000).unwrap();
    assert_eq!(filed.new_tags.len(), 1);
    assert_eq!(l.store.note(&waiting).unwrap().unwrap().updated_at, 5_000);
    assert_eq!(
        file::undo(&l.store, &filed.edits, &filed.new_tags, 9_000).unwrap(),
        1
    );
    let back = l.store.note(&waiting).unwrap().unwrap();
    // Stamped, never restored to the old time.
    assert_eq!(back.updated_at, 9_000);
    assert!(back.updated_at > before.min(5_000));
    assert!(back.tags.is_empty());
    assert!(
        l.store
            .tags()
            .unwrap()
            .iter()
            .all(|t| t.name != "freshly-made")
    );
    // The note is waiting in Triage again.
    assert_eq!(l.store.inbox_waiting(&l.inbox).unwrap().len(), 1);
}

#[test]
fn undo_keeps_a_made_tag_another_note_now_uses() {
    let (l, waiting) = filled();
    let rev = l.store.note(&waiting).unwrap().unwrap().rev.unwrap();
    let filed = file::file(&l.store, &[new_tag(&waiting, &rev, "shared-tag")], 5_000).unwrap();
    let tag = filed.new_tags[0].clone();
    note(&l.store, &l.work, "Other", "client", &["shared-tag"]);
    file::undo(&l.store, &filed.edits, &filed.new_tags, 9_000).unwrap();
    assert!(l.store.tags().unwrap().iter().any(|t| t.id == tag));
}

#[test]
fn a_tag_that_already_existed_is_not_removed_by_undo() {
    let (l, waiting) = filled();
    let rev = l.store.note(&waiting).unwrap().unwrap().rev.unwrap();
    let filed = file::file(&l.store, &[new_tag(&waiting, &rev, "house")], 5_000).unwrap();
    assert!(filed.new_tags.is_empty());
}

#[test]
fn filing_refuses_a_note_edited_after_planning() {
    let (l, waiting) = filled();
    let rev = l.store.note(&waiting).unwrap().unwrap().rev.unwrap();
    let planned = file::plan(&l.store, &[new_tag(&waiting, &rev, "x")], 5_000).unwrap();
    assert!(planned.stale.is_empty());
    // An edit lands between planning and the write: the revision check
    // inside the write catches it, and the note is reported stale.
    l.store
        .merge_many(
            DocType::Note,
            vec![json!({"_id": waiting, "body": "typed since"})],
            Origin::Ui,
        )
        .unwrap();
    let filed = file::file(&l.store, &[new_tag(&waiting, &rev, "x")], 5_000).unwrap();
    assert_eq!(filed.stale, vec![waiting.clone()]);
    assert!(filed.edits.is_empty());
    assert_eq!(l.store.note(&waiting).unwrap().unwrap().body, "typed since");
}

#[test]
fn cards_for_only_computes_the_notes_asked_for() {
    let (l, waiting) = filled();
    let second = note(&l.store, &l.inbox, "More", "budget sales client", &[]);
    index(&l.store);
    let both = cards(&l.store, &opts(&l), 10).unwrap();
    assert_eq!(both.len(), 2);
    let one = cards_for(&l.store, std::slice::from_ref(&second), &opts(&l)).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(
        one[0],
        both.iter().find(|c| c.note_id == second).cloned().unwrap()
    );
    assert_eq!(
        card(&l.store, &waiting, &opts(&l)).unwrap(),
        both.into_iter().find(|c| c.note_id == waiting)
    );
}

#[test]
fn a_note_outside_the_inbox_or_in_the_trash_has_no_card() {
    let (l, _) = filled();
    let elsewhere = note(&l.store, &l.home, "Home note", "plumber", &[]);
    assert!(card(&l.store, &elsewhere, &opts(&l)).unwrap().is_none());
    assert!(card(&l.store, "note:missing", &opts(&l)).unwrap().is_none());
}

#[test]
fn cards_list_what_waits_and_filing_takes_a_note_off_the_list() {
    let (l, waiting) = filled();
    let all = cards(&l.store, &opts(&l), 10).unwrap();
    assert_eq!(all.len(), 1);
    let card = &all[0];
    let accepted: Vec<Suggestion> = card
        .suggestions
        .iter()
        .filter(|s| s.field != Field::Link)
        .cloned()
        .collect();
    let filing = Filing {
        note_id: waiting.clone(),
        rev: card.rev.clone(),
        accepted,
    };
    let preview = file::plan(&l.store, std::slice::from_ref(&filing), 1_000).unwrap();
    assert_eq!(preview.moved, 1);
    let filed = file::file(&l.store, &[filing], 1_000).unwrap();
    assert_eq!(filed.edits.len(), 1);
    let note = l.store.note(&waiting).unwrap().unwrap();
    assert_eq!(note.book_id, l.home);
    assert_eq!(
        note.title,
        "Call the plumber about the leak, boiler and pipe"
    );
    assert!(!note.tags.is_empty());
    assert_eq!(note.extra["kansoTriaged"], json!({"v": 1, "at": 1_000}));
    // Filed notes leave Triage; undo brings the note back as it was.
    assert!(l.store.inbox_waiting(&l.inbox).unwrap().is_empty());
    assert_eq!(
        file::undo(&l.store, &filed.edits, &filed.new_tags, 2_000).unwrap(),
        1
    );
    let back = l.store.note(&waiting).unwrap().unwrap();
    assert_eq!(back.book_id, l.inbox);
    assert_eq!(back.title, "Untitled");
    assert!(back.tags.is_empty());
    assert_eq!(l.store.inbox_waiting(&l.inbox).unwrap().len(), 1);
}

#[test]
fn a_note_edited_since_its_card_is_skipped() {
    let (l, waiting) = filled();
    let card = card(&l.store, &waiting, &opts(&l)).unwrap().unwrap();
    l.store
        .merge_many(
            DocType::Note,
            vec![json!({"_id": waiting, "body": "changed elsewhere"})],
            Origin::Sync,
        )
        .unwrap();
    let filing = Filing {
        note_id: waiting.clone(),
        rev: card.rev.clone(),
        accepted: card.suggestions.clone(),
    };
    let filed = file::file(&l.store, &[filing], 2_000).unwrap();
    assert!(filed.edits.is_empty());
    assert_eq!(filed.stale, std::slice::from_ref(&waiting));
    let note = l.store.note(&waiting).unwrap().unwrap();
    assert_eq!(note.book_id, l.inbox);
    assert_eq!(note.body, "changed elsewhere");
}

#[test]
fn a_new_tag_is_made_when_filing() {
    let (l, waiting) = filled();
    let card = card(&l.store, &waiting, &opts(&l)).unwrap().unwrap();
    let new = Suggestion {
        field: Field::Tag,
        value: "Garden".into(),
        label: "Garden".into(),
        score: 0.5,
        because: vec![],
        source: Source::Library,
        is_new: true,
    };
    let filing = Filing {
        note_id: waiting.clone(),
        rev: card.rev,
        accepted: vec![new],
    };
    let preview = file::plan(&l.store, std::slice::from_ref(&filing), 3_000).unwrap();
    assert_eq!(preview.tags, 1);
    assert!(l.store.tags().unwrap().iter().all(|t| t.name != "Garden"));
    file::file(&l.store, &[filing], 3_000).unwrap();
    let tag = l
        .store
        .tags()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "Garden")
        .unwrap();
    assert!(
        l.store
            .note(&waiting)
            .unwrap()
            .unwrap()
            .tags
            .contains(&tag.id)
    );
}

#[test]
fn the_confirmation_says_what_will_happen() {
    let filed = Filed {
        moved: 4,
        tags: 9,
        titles: 3,
        links: 6,
        ..Filed::default()
    };
    assert_eq!(
        filed.sentence(),
        "Moves 4 to notebooks, adds 9 tags, 3 titles, 6 links."
    );
    assert_eq!(Filed::default().sentence(), "Marks them filed.");
}

fn is_book(store: &Store, id: &str) -> bool {
    store
        .get(id)
        .unwrap()
        .is_some_and(|d| d.doc_type == DocType::Book)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Wherever the other notes are, nothing suggested is the Inbox, the
    /// trash, a book that isn't one, a note already linked or the note
    /// itself; scores stay in range; and asking twice answers the same.
    #[test]
    fn suggestions_are_clean(
        places in prop::collection::vec(0usize..4, 1..7),
        topics in prop::collection::vec(0usize..2, 1..7),
        linked in any::<bool>(),
    ) {
        let l = lib();
        let spots = [l.home.clone(), l.work.clone(), "trash".to_owned(), l.inbox.clone()];
        let bodies = ["plumber boiler pipe", "sales client budget"];
        let mut ids = Vec::new();
        for (i, (p, t)) in places.iter().zip(&topics).enumerate() {
            ids.push(note(&l.store, &spots[*p], &format!("Note number {i}"), bodies[*t], &["t"]));
        }
        let body = if linked {
            "plumber boiler pipe [[Note number 0]] and note number 1 again".to_owned()
        } else {
            "plumber boiler pipe note number 1 again".to_owned()
        };
        let waiting = note(&l.store, &l.inbox, "Untitled", &body, &[]);
        index(&l.store);
        let o = opts(&l);
        let got = suggest(&l.store, &waiting, &o).unwrap();
        let linked_now = l.store.links_from(&waiting).unwrap();
        for s in &got {
            prop_assert!((0. ..=1.).contains(&s.score));
            prop_assert!(s.because.len() <= BECAUSE_MAX);
            prop_assert!(s.value != waiting);
            match s.field {
                Field::Notebook => {
                    prop_assert!(s.value != l.inbox && s.value != "trash");
                    prop_assert!(is_book(&l.store, &s.value));
                }
                Field::Link => prop_assert!(!linked_now.contains(&s.value)),
                _ => {}
            }
            let evidence: HashSet<&String> = s.because.iter().collect();
            for e in evidence {
                let n = l.store.note(e).unwrap().unwrap();
                prop_assert!(n.book_id != "trash");
            }
        }
        prop_assert_eq!(got, suggest(&l.store, &waiting, &o).unwrap());
        let _ = ids;
    }
}
