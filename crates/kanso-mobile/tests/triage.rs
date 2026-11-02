//! Inbox triage on a phone (issue #195): the cards, what filing writes and
//! how Undo takes it back, rejections that stay, the model that only ranks
//! what the library found, and that a note kept out of AI is never sent.
//!
//! Related notes use a tiny embedder that counts topic words, so the
//! suggestions are exact.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_mobile::{
    KansoError, Triage, TriageChip, TriageField, TriageFiling, TriageMissing, TriageSettings,
};
use kanso_store::{Embedder, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

const WORDS: [&str; 8] = [
    "plumber", "boiler", "leak", "pipe", "meeting", "budget", "sales", "client",
];

/// Counts topic words: notes about one topic point the same way.
struct Topics;

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

fn store(phone: &Phone) -> Store {
    Store::open(phone.path()).unwrap()
}

fn book(store: &Store, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({"name": name}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
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

struct World {
    phone: Phone,
    store: Store,
    inbox: String,
    home: String,
    waiting: String,
}

/// Three plumbing notes in Home (tagged #house), two sales notes in Work,
/// a plumbing note waiting in the Inbox.
fn world() -> World {
    let phone = lone_phone();
    let store = store(&phone);
    let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let home = book(&store, "Home");
    let work = book(&store, "Work");
    note(
        &store,
        &home,
        "Boiler service",
        "boiler plumber pipe",
        &["house"],
    );
    note(
        &store,
        &home,
        "Flat repairs 2026",
        "leak pipe plumber",
        &["house"],
    );
    note(&store, &home, "Bathroom", "pipe leak boiler", &["house"]);
    note(
        &store,
        &work,
        "Sales plan",
        "sales client budget",
        &["work"],
    );
    note(&store, &work, "Meeting", "meeting client", &["work"]);
    let waiting = note(
        &store,
        &inbox,
        "Untitled",
        "Call the plumber about the leak, boiler and pipe. See flat repairs 2026 too.",
        &[],
    );
    index(&store);
    World {
        phone,
        store,
        inbox,
        home,
        waiting,
    }
}

fn chips_of(card: &kanso_mobile::TriageCard, field: TriageField) -> Vec<&TriageChip> {
    card.chips.iter().filter(|c| c.field == field).collect()
}

fn filing(
    card: &kanso_mobile::TriageCard,
    mut accept: impl FnMut(&TriageChip) -> bool,
) -> TriageFiling {
    TriageFiling {
        note_id: card.note_id.clone(),
        rev: card.rev.clone(),
        accepted: card.chips.iter().filter(|c| accept(c)).cloned().collect(),
    }
}

// no-prop-test: the settings' defaults and one round trip.
#[test]
fn settings_default_as_specified_and_round_trip() {
    let phone = lone_phone();
    let triage = phone.library.triage();
    let first = triage.settings();
    assert!(first.show_bar && first.suggest_titles && first.suggest_links);
    assert_eq!(first.max_tags, 3);
    assert!(!first.use_ai, "the model is off until it is turned on");
    let saved = triage
        .set_settings(TriageSettings {
            show_bar: false,
            suggest_titles: false,
            suggest_links: true,
            max_tags: 99,
            use_ai: true,
        })
        .unwrap();
    assert_eq!(saved.max_tags, 5, "kept to the limit");
    assert!(!saved.show_bar && saved.use_ai);
    assert_eq!(phone.library.triage().settings(), saved);
}

// no-prop-test: one library read through the facade; the rules are
// properties in kanso_engine::triage.
#[test]
fn a_card_has_a_notebook_tags_a_title_and_a_link() {
    let w = world();
    let triage = w.phone.library.triage();
    let status = triage.status().unwrap();
    assert_eq!(status.waiting, 1);
    assert!(status.related_on && !status.ai_on);

    let cards = triage.cards(10).unwrap();
    assert_eq!(cards.len(), 1);
    let card = &cards[0];
    assert_eq!(card.note_id, w.waiting);
    assert_eq!(card.missing, TriageMissing::Nothing);
    let book = chips_of(card, TriageField::Notebook);
    assert_eq!(book.len(), 1);
    assert_eq!(
        (book[0].value.as_str(), book[0].label.as_str()),
        (w.home.as_str(), "Home")
    );
    assert!(!book[0].because.is_empty() && book[0].because.len() <= 3);
    assert!(book[0].because.iter().all(|n| !n.title.is_empty()));
    assert_eq!(chips_of(card, TriageField::Tag)[0].label, "house");
    assert!(
        chips_of(card, TriageField::Title)[0]
            .value
            .starts_with("Call the plumber")
    );
    assert!(
        chips_of(card, TriageField::Link)
            .iter()
            .any(|c| c.label == "Flat repairs 2026")
    );
    assert!(
        card.chips
            .iter()
            .all(|c| !c.ai && (0. ..=1.).contains(&c.score))
    );
}

// no-prop-test: related notes off, read once.
#[test]
fn without_related_notes_only_titles_and_links_come() {
    let w = world();
    w.store.set_embedding_model(None).unwrap();
    let triage = w.phone.library.triage();
    assert!(!triage.status().unwrap().related_on);
    let card = triage.card(&w.waiting).unwrap().unwrap();
    assert_eq!(card.missing, TriageMissing::RelatedOff);
    assert!(chips_of(&card, TriageField::Notebook).is_empty());
    assert!(chips_of(&card, TriageField::Tag).is_empty());
    assert!(!chips_of(&card, TriageField::Title).is_empty());
}

// no-prop-test: one filing and its Undo, end to end.
#[test]
fn filing_writes_only_what_was_accepted_and_undo_puts_it_back() {
    let w = world();
    let triage = w.phone.library.triage();
    let card = triage.cards(10).unwrap().remove(0);
    let batch = vec![filing(&card, |c| {
        matches!(c.field, TriageField::Notebook | TriageField::Tag)
    })];
    let plan = triage.plan(batch.clone()).unwrap();
    assert_eq!((plan.filed, plan.moved), (1, 1));
    assert_eq!(plan.undo, 0, "a plan writes nothing");
    assert!(plan.sentence.starts_with("Moves 1 to notebooks"));
    let still = w.store.note(&w.waiting).unwrap().unwrap();
    assert_eq!(
        (still.book_id.as_str(), still.title.as_str()),
        (w.inbox.as_str(), "Untitled")
    );

    let filed = triage.file(batch).unwrap();
    assert_eq!(filed.filed, 1);
    assert!(filed.undo > 0);
    let after = w.store.note(&w.waiting).unwrap().unwrap();
    assert_eq!(after.book_id, w.home);
    assert_eq!(after.title, "Untitled", "the title chip wasn't accepted");
    assert!(!after.tags.is_empty());
    assert!(after.extra.contains_key("kansoTriaged"));
    assert_eq!(triage.status().unwrap().waiting, 0);

    assert_eq!(w.phone.library.undo(filed.undo).unwrap(), 1);
    let back = w.store.note(&w.waiting).unwrap().unwrap();
    assert_eq!(back.book_id, w.inbox);
    assert!(back.tags.is_empty());
    assert_eq!(triage.status().unwrap().waiting, 1);
    assert_eq!(w.phone.library.undo(filed.undo).unwrap(), 0, "used once");
}

// no-prop-test: the stale check, once.
#[test]
fn a_note_changed_since_its_card_is_left_alone() {
    let w = world();
    let triage = w.phone.library.triage();
    let card = triage.cards(10).unwrap().remove(0);
    w.store
        .merge_many(
            DocType::Note,
            vec![json!({"_id": w.waiting, "body": "edited on another device"})],
            Origin::Sync,
        )
        .unwrap();
    let filed = triage.file(vec![filing(&card, |_| true)]).unwrap();
    assert_eq!(filed.filed, 0);
    assert_eq!(filed.stale, std::slice::from_ref(&w.waiting));
    assert_eq!(filed.undo, 0);
    let note = w.store.note(&w.waiting).unwrap().unwrap();
    assert_eq!(note.book_id, w.inbox);
    assert_eq!(note.body, "edited on another device");
}

// no-prop-test: rejection memory and the typed values, once each.
#[test]
fn rejections_stay_and_typed_values_resolve() {
    let w = world();
    let triage = w.phone.library.triage();
    let card = triage.cards(10).unwrap().remove(0);
    let tag = chips_of(&card, TriageField::Tag)[0].clone();
    triage
        .reject(&w.waiting, TriageField::Tag, &tag.value)
        .unwrap();
    let again = triage.card(&w.waiting).unwrap().unwrap();
    assert!(
        !chips_of(&again, TriageField::Tag)
            .iter()
            .any(|c| c.value == tag.value)
    );
    triage
        .unreject(&w.waiting, TriageField::Tag, &tag.value)
        .unwrap();
    assert!(
        chips_of(&triage.card(&w.waiting).unwrap().unwrap(), TriageField::Tag)
            .iter()
            .any(|c| c.value == tag.value)
    );

    let book = triage.resolve(TriageField::Notebook, "work").unwrap();
    assert_eq!(book.label, "Work");
    assert!(triage.resolve(TriageField::Notebook, "Inbox").is_err());
    let new = triage.resolve(TriageField::Tag, "#Garden").unwrap();
    assert!(new.is_new);
    let filed = triage
        .file(vec![TriageFiling {
            note_id: w.waiting.clone(),
            rev: card.rev.clone(),
            accepted: vec![new],
        }])
        .unwrap();
    assert_eq!(filed.tags, 1);
    assert!(w.store.tags().unwrap().iter().any(|t| t.name == "Garden"));
}

fn ai_triage(w: &World, answer: &str) -> (Triage, std::sync::Arc<Scripted>) {
    let model = Scripted::new(answer);
    let ai = ai_on(&w.phone.library, &model);
    let triage = w.phone.library.triage_with_ai(ai);
    let mut settings = triage.settings();
    settings.use_ai = true;
    triage.set_settings(settings).unwrap();
    (triage, model)
}

// no-prop-test: scripted model replies, each checked once.
#[test]
fn the_model_only_ranks_what_the_library_found_and_cites_it() {
    let w = world();
    let (triage, model) = ai_triage(&w, "{}");
    let card = triage.cards(10).unwrap().remove(0);
    let book = chips_of(&card, TriageField::Notebook)[0].clone();
    let tag = chips_of(&card, TriageField::Tag)[0].clone();
    let answer = json!({
        "notebook": book.value,
        "tags": [tag.value, "tag:invented"],
        "links": ["note:invented"],
        "title": "Plumber and the boiler"
    })
    .to_string();
    let (triage, model2) = ai_triage(&w, &answer);
    drop(model);
    let chips = triage.refine(&w.waiting).unwrap();
    assert!(chips.iter().all(|c| !c.because.is_empty()), "{chips:?}");
    let ai: Vec<_> = chips.iter().filter(|c| c.ai).collect();
    assert!(
        ai.iter()
            .any(|c| c.field == TriageField::Notebook && c.value == book.value)
    );
    assert!(
        ai.iter()
            .any(|c| c.field == TriageField::Title && c.value == "Plumber and the boiler")
    );
    assert!(!chips.iter().any(|c| c.value.contains("invented")));
    // What was sent: the one note and candidate names, nothing else.
    let sent = model2.everything_sent();
    assert!(sent.contains("Call the plumber"));
    assert!(sent.contains("Home"));
    assert!(
        !sent.contains("sales client budget"),
        "no other note's text"
    );
    assert_eq!(model2.request_count(), 1);
}

// no-prop-test: refusals, each checked once.
#[test]
fn a_note_kept_out_of_ai_is_never_sent_and_off_means_off() {
    let w = world();
    let (triage, model) = ai_triage(&w, "{}");
    // Tagged #ai-exclude: refused, nothing sent.
    let tag = w.store.tag_named("ai-exclude", Origin::Ui).unwrap();
    w.store
        .merge_many(
            DocType::Note,
            vec![json!({"_id": w.waiting, "tags": [tag]})],
            Origin::Ui,
        )
        .unwrap();
    let err = triage.refine(&w.waiting).unwrap_err();
    assert!(
        matches!(&err, KansoError::Invalid { message } if message.contains("kept out")),
        "{err:?}"
    );
    assert_eq!(model.request_count(), 0);

    // The Inbox switch off: refused, nothing sent.
    let mut settings = triage.settings();
    settings.use_ai = false;
    triage.set_settings(settings).unwrap();
    assert!(triage.refine(&w.waiting).is_err());
    assert_eq!(model.request_count(), 0);
    assert!(!triage.status().unwrap().ai_on);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Whatever subset of chips is accepted, filing then Undo gives the
    /// note back as it was.
    #[test]
    fn file_then_undo_restores_the_note(mask in prop::collection::vec(any::<bool>(), 0..10)) {
        let w = world();
        let triage = w.phone.library.triage();
        let before = w.store.note(&w.waiting).unwrap().unwrap();
        let card = triage.cards(10).unwrap().remove(0);
        let mut i = 0;
        let batch = filing(&card, |_| {
            let keep = mask.get(i).copied().unwrap_or(false);
            i += 1;
            keep
        });
        let filed = triage.file(vec![batch]).unwrap();
        prop_assert_eq!(filed.filed, 1);
        w.phone.library.undo(filed.undo).unwrap();
        let after = w.store.note(&w.waiting).unwrap().unwrap();
        prop_assert_eq!(&after.book_id, &before.book_id);
        prop_assert_eq!(&after.title, &before.title);
        prop_assert_eq!(&after.body, &before.body);
        prop_assert_eq!(&after.tags, &before.tags);
    }
}
