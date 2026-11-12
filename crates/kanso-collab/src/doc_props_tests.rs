//! Properties merged one key at a time ([`crate::doc_props`]).

use proptest::prelude::*;

use crate::doc::{CollabDoc, Field};
use crate::tests::config;

const PART: usize = usize::MAX;

/// A key (distinct keys come from distinct indexes) and a value as the text
/// after its colon.
fn rest(kind: u8, word: &str) -> String {
    match kind % 3 {
        0 => format!(" {word}\n"),
        1 => format!(" [{word}, x]\n"),
        _ => format!("\n  - {word}\n  - y\n"),
    }
}

fn note(entries: &[(String, String)], body: &str) -> String {
    let front: String = entries
        .iter()
        .map(|(key, rest)| format!("{key}:{rest}"))
        .collect();
    format!("---\n{front}---\n{body}")
}

fn gen_entries() -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((0u8..3, "[a-z0-9]{1,8}"), 1..6).prop_map(|rows| {
        rows.into_iter()
            .enumerate()
            .map(|(i, (kind, word))| (format!("k{i}"), rest(kind, &word)))
            .collect()
    })
}

fn gen_body() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 \n]{0,40}"
}

/// A document holding `text`, and a replica of it (a second device).
fn pair(text: &str) -> (CollabDoc, CollabDoc) {
    let a = CollabDoc::new();
    for update in a.set_note_text_in_parts(text, PART) {
        a.apply(&update).unwrap();
    }
    let b = CollabDoc::load(&a.encode_state()).unwrap();
    (a, b)
}

/// What `doc` sends when its note becomes `text`.
fn edit(doc: &CollabDoc, text: &str) -> Vec<Vec<u8>> {
    doc.set_note_text_in_parts(text, PART)
}

fn deliver(to: &CollabDoc, updates: &[Vec<u8>]) {
    for update in updates {
        to.apply(update).unwrap();
    }
}

proptest! {
    #![proptest_config(config(64))]

    /// Two people change different properties offline: both changes stay, in
    /// whichever order the updates arrive.
    #[test]
    fn prop_concurrent_edits_to_different_keys_both_survive(
        entries in gen_entries(),
        body in gen_body(),
        pick in any::<prop::sample::Index>(),
        word_a in "[a-z0-9]{1,8}",
        word_b in "[a-z0-9]{1,8}",
    ) {
        prop_assume!(entries.len() >= 2);
        let base = note(&entries, &body);
        let i = pick.index(entries.len());
        let j = (i + 1) % entries.len();
        let (a, b) = pair(&base);
        let mut mine = entries.clone();
        mine[i].1 = rest(0, &format!("{word_a}a"));
        let mut theirs = entries.clone();
        theirs[j].1 = rest(0, &format!("{word_b}b"));
        let from_a = edit(&a, &note(&mine, &body));
        let from_b = edit(&b, &note(&theirs, &body));
        let (c, d) = (
            CollabDoc::load(&a.encode_state()).unwrap(),
            CollabDoc::load(&b.encode_state()).unwrap(),
        );
        // A hears B, B hears A; two others hear them in the other order.
        deliver(&a, &from_b);
        deliver(&b, &from_a);
        deliver(&c, &from_a);
        deliver(&c, &from_b);
        deliver(&d, &from_b);
        deliver(&d, &from_a);
        let mut want = entries.clone();
        want[i].1 = mine[i].1.clone();
        want[j].1 = theirs[j].1.clone();
        let want = note(&want, &body);
        for doc in [&a, &b, &c, &d] {
            prop_assert_eq!(doc.note_text(), want.clone());
        }
    }

    /// Two people change the same property: every replica ends with the
    /// same one of the two values, whole, and the other properties stay.
    #[test]
    fn prop_same_key_edits_converge_identically(
        entries in gen_entries(),
        body in gen_body(),
        pick in any::<prop::sample::Index>(),
        kind_a in 0u8..3,
        kind_b in 0u8..3,
        word_a in "[a-z0-9]{1,8}",
        word_b in "[a-z0-9]{1,8}",
    ) {
        let base = note(&entries, &body);
        let i = pick.index(entries.len());
        let (a, b) = pair(&base);
        let mut mine = entries.clone();
        mine[i].1 = rest(kind_a, &format!("{word_a}a"));
        let mut theirs = entries.clone();
        theirs[i].1 = rest(kind_b, &format!("{word_b}b"));
        let from_a = edit(&a, &note(&mine, &body));
        let from_b = edit(&b, &note(&theirs, &body));
        let c = CollabDoc::load(&a.encode_state()).unwrap();
        deliver(&a, &from_b);
        deliver(&b, &from_a);
        deliver(&c, &from_b);
        deliver(&c, &from_a);
        prop_assert_eq!(a.note_text(), b.note_text());
        prop_assert_eq!(a.note_text(), c.note_text());
        let text = a.note_text();
        prop_assert!(
            text == note(&mine, &body) || text == note(&theirs, &body),
            "a mix of the two: {text:?}"
        );
    }

    /// The body still merges as text while a property changes beside it.
    #[test]
    fn prop_body_edits_merge_as_before_beside_a_property_edit(
        entries in gen_entries(),
        word in "[a-z0-9]{1,8}",
        head in "[a-z]{1,6}",
        tail in "[a-z]{1,6}",
    ) {
        let body = "middle line\n";
        let base = note(&entries, body);
        let (a, b) = pair(&base);
        let mut props = entries.clone();
        props[0].1 = rest(0, &word);
        let from_a = edit(&a, &note(&props, body));
        let from_b = edit(&b, &note(&entries, &format!("{head}\nmiddle line\n{tail}\n")));
        deliver(&a, &from_b);
        deliver(&b, &from_a);
        let want = note(&props, &format!("{head}\nmiddle line\n{tail}\n"));
        prop_assert_eq!(a.note_text(), want.clone());
        prop_assert_eq!(b.note_text(), want);
    }

    /// Front matter and body come back as written, and writing the same text
    /// again sends nothing.
    #[test]
    fn prop_note_text_round_trips_and_is_stable(entries in gen_entries(), body in gen_body()) {
        let text = note(&entries, &body);
        let doc = CollabDoc::new();
        deliver(&doc, &edit(&doc, &text));
        prop_assert_eq!(doc.note_text(), text.clone());
        prop_assert!(edit(&doc, &text).is_empty());
        // A fresh device loading the state reads the same note.
        let loaded = CollabDoc::load(&doc.encode_state()).unwrap();
        prop_assert_eq!(loaded.note_text(), text);
        prop_assert_eq!(loaded.props(), doc.props());
    }

    /// Any text, with front matter or not, reads back as it was set.
    #[test]
    fn prop_any_text_round_trips(text in "(---\n)?[a-z]{0,3}(: [a-z]{0,3})?\n?(# c\n)?(---\n)?[a-z \n]{0,12}") {
        let doc = CollabDoc::new();
        deliver(&doc, &edit(&doc, &text));
        prop_assert_eq!(doc.note_text(), text);
    }
}

#[test]
fn two_offline_moves_of_one_card_give_one_stage() {
    let base = "---\nstage: Idea\ndue: 2026-12-01\n---\nBrief\n";
    let (a, b) = pair(base);
    let from_a = edit(&a, "---\nstage: Shooting\ndue: 2026-12-01\n---\nBrief\n");
    let from_b = edit(&b, "---\nstage: Editing\ndue: 2026-12-01\n---\nBrief\n");
    deliver(&a, &from_b);
    deliver(&b, &from_a);
    assert_eq!(a.note_text(), b.note_text());
    let text = a.note_text();
    assert!(
        text == "---\nstage: Shooting\ndue: 2026-12-01\n---\nBrief\n"
            || text == "---\nstage: Editing\ndue: 2026-12-01\n---\nBrief\n",
        "{text:?}"
    );
    let stage = kanso_core::properties::parse(&text);
    assert_eq!(stage.len(), 2);
}

#[test]
fn a_stage_and_a_due_date_changed_offline_both_stay() {
    let base = "---\nstage: Idea\ndue: 2026-12-01\n---\nBrief\n";
    let (a, b) = pair(base);
    let from_a = edit(&a, "---\nstage: Shooting\ndue: 2026-12-01\n---\nBrief\n");
    let from_b = edit(&b, "---\nstage: Idea\ndue: 2026-12-09\n---\nBrief\n");
    deliver(&a, &from_b);
    deliver(&b, &from_a);
    let want = "---\nstage: Shooting\ndue: 2026-12-09\n---\nBrief\n";
    assert_eq!(a.note_text(), want);
    assert_eq!(b.note_text(), want);
}

#[test]
fn the_search_copy_has_front_matter_and_the_body_has_none() {
    let doc = CollabDoc::new();
    deliver(&doc, &edit(&doc, "---\nstage: Idea\n---\nBrief\n"));
    assert_eq!(doc.note_text(), "---\nstage: Idea\n---\nBrief\n");
    assert_eq!(doc.text(Field::Body), "Brief\n");
    assert_eq!(doc.props().len(), 1);
}

#[test]
fn a_new_property_lands_where_it_was_written_and_removing_one_removes_it() {
    let doc = CollabDoc::new();
    deliver(&doc, &edit(&doc, "---\na: 1\nc: 3\n---\nx\n"));
    deliver(&doc, &edit(&doc, "---\na: 1\nb: 2\nc: 3\n---\nx\n"));
    assert_eq!(doc.note_text(), "---\na: 1\nb: 2\nc: 3\n---\nx\n");
    deliver(&doc, &edit(&doc, "---\nb: 2\nc: 3\nd: 4\n---\nx\n"));
    assert_eq!(doc.note_text(), "---\nb: 2\nc: 3\nd: 4\n---\nx\n");
    deliver(&doc, &edit(&doc, "x\n"));
    assert_eq!(doc.note_text(), "x\n");
    assert!(doc.props().is_empty());
}

#[test]
fn an_old_document_with_front_matter_in_its_text_still_opens() {
    // The shape every shared note had: the front matter inside the body.
    let old = CollabDoc::from_text("Title", "---\nstage: Idea\n---\nBrief\n");
    let loaded = CollabDoc::load(&old.encode_state()).unwrap();
    assert_eq!(loaded.text(Field::Title), "Title");
    assert!(loaded.props().is_empty());
    assert_eq!(loaded.note_text(), "---\nstage: Idea\n---\nBrief\n");
    assert_eq!(loaded.text(Field::Body), "---\nstage: Idea\n---\nBrief\n");
}

#[test]
fn an_old_document_moves_its_front_matter_into_properties_on_its_first_edit() {
    let old = CollabDoc::from_text("", "---\nstage: Idea\n---\nBrief\n");
    let (a, b) = {
        let a = CollabDoc::load(&old.encode_state()).unwrap();
        let b = CollabDoc::load(&old.encode_state()).unwrap();
        (a, b)
    };
    let from_a = edit(&a, "---\nstage: Shooting\n---\nBrief\n");
    assert_eq!(a.text(Field::Body), "Brief\n");
    assert_eq!(a.props().len(), 1);
    // The other device does the same move to its own copy, offline.
    let from_b = edit(&b, "---\nstage: Idea\n---\nBrief\nMore\n");
    deliver(&a, &from_b);
    deliver(&b, &from_a);
    assert_eq!(a.note_text(), b.note_text());
    let text = a.note_text();
    assert!(text.starts_with("---\nstage: "), "{text:?}");
    assert!(text.contains("Brief\n"), "{text:?}");
    assert!(text.contains("More\n"), "{text:?}");
    assert_eq!(text.matches("---\n").count(), 2, "{text:?}");
}

#[test]
fn front_matter_that_cannot_be_kept_key_by_key_stays_text() {
    for text in [
        "---\n# why\nstage: Idea\n---\nBrief\n",
        "---\r\nstage: Idea\r\n---\r\nBrief\r\n",
        "---\nstage: a\nstage: b\n---\nBrief\n",
        "---\nstage: Idea\n...\nBrief\n",
        "---\n---\nBrief\n",
        "\u{feff}---\nstage: Idea\n---\nBrief\n",
    ] {
        let doc = CollabDoc::new();
        deliver(&doc, &edit(&doc, text));
        assert_eq!(doc.note_text(), text);
        assert!(doc.props().is_empty(), "{text:?}");
        assert_eq!(doc.text(Field::Body), text);
    }
}

#[test]
fn a_merge_reports_properties_changed() {
    let (a, b) = pair("---\nstage: Idea\n---\nBrief\n");
    let update = edit(&a, "---\nstage: Shooting\n---\nBrief\n");
    let merged = b.apply(&update[0]).unwrap();
    assert!(merged.props);
    assert!(merged.body.is_empty());
    assert!(!merged.is_empty());
}
