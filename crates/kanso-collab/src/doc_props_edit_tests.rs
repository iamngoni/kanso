//! A person's edit of a note's text taken one property at a time
//! ([`CollabDoc::edit_note_in_parts`]), and positions in the body through
//! changes to the front matter ([`CollabDoc::note_sticky`]).

use proptest::prelude::*;

use crate::doc::CollabDoc;
use crate::tests::config;

const PART: usize = usize::MAX;

/// A value as the text after its colon.
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

    /// Taking a local edit of one property onto a note that has merged
    /// another person's change to a different property keeps both, and an
    /// edit of the body is rebased past it.
    #[test]
    fn prop_a_local_edit_never_undoes_a_merged_property(
        entries in gen_entries(),
        body in gen_body(),
        pick in any::<prop::sample::Index>(),
        word_a in "[a-z0-9]{1,8}",
        word_b in "[a-z0-9]{1,8}",
        tail in "[a-z]{1,5}",
    ) {
        prop_assume!(entries.len() >= 2);
        let base = note(&entries, &body);
        let i = pick.index(entries.len());
        let j = (i + 1) % entries.len();
        let (mine, theirs) = pair(&base);
        // B changes property j; A has merged it before taking its own edit.
        let mut other = entries.clone();
        other[j].1 = rest(0, &format!("{word_b}b"));
        deliver(&mine, &edit(&theirs, &note(&other, &body)));
        // A's own edit, made on `base`: property i and the end of the body.
        let mut local = entries.clone();
        local[i].1 = rest(0, &format!("{word_a}a"));
        let local_text = note(&local, &format!("{body}{tail}"));
        let sent = mine.edit_note_in_parts(&base, &local_text, PART);
        let mut want = entries.clone();
        want[i].1 = local[i].1.clone();
        want[j].1 = other[j].1.clone();
        prop_assert_eq!(mine.note_text(), note(&want, &format!("{body}{tail}")));
        // What it sent brings a replica of B's to the same note.
        deliver(&theirs, &sent);
        prop_assert_eq!(theirs.note_text(), mine.note_text());
    }

    /// Changing a property the other person changed too takes the local
    /// value whole, never a mix of the two.
    #[test]
    fn prop_a_local_value_replaces_a_merged_one_whole(
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
        let (mine, theirs) = pair(&base);
        let mut other = entries.clone();
        other[i].1 = rest(kind_b, &format!("{word_b}b"));
        deliver(&mine, &edit(&theirs, &note(&other, &body)));
        let mut local = entries.clone();
        local[i].1 = rest(kind_a, &format!("{word_a}a"));
        let sent = mine.edit_note_in_parts(&base, &note(&local, &body), PART);
        prop_assert_eq!(mine.note_text(), note(&local, &body));
        deliver(&theirs, &sent);
        prop_assert_eq!(theirs.note_text(), mine.note_text());
    }

    /// A position in the body follows its character when another person
    /// adds, changes or removes properties, which moves the body inside the
    /// note's text.
    #[test]
    fn prop_a_caret_in_the_body_follows_its_character_through_property_edits(
        entries in gen_entries(),
        body in "[A-Za-z0-9 ]{1,40}",
        at in any::<prop::sample::Index>(),
        word in "[a-z0-9]{1,12}",
        grow in 0usize..3,
    ) {
        let base = note(&entries, &body);
        let (mine, theirs) = pair(&base);
        let text = mine.note_text();
        let start = text.len() - body.len();
        let offset = start + at.index(body.len());
        let wanted = text.as_bytes()[offset];
        let sticky = mine.note_sticky(offset);
        // B rewrites the front matter: a changed value, a new key, a removed one.
        let mut other = entries.clone();
        other[0].1 = rest(0, &word.repeat(1 + grow));
        other.push((format!("added{grow}"), rest(1, &word)));
        if other.len() > 3 {
            other.remove(1);
        }
        deliver(&mine, &edit(&theirs, &note(&other, &body)));
        let now = mine.note_text();
        let moved = mine.note_resolve(&sticky).unwrap();
        prop_assert_eq!(now.as_bytes()[moved], wanted);
        prop_assert_eq!(moved, now.len() - body.len() + (offset - start));
    }
}

#[test]
fn a_position_in_the_front_matter_sticks_to_the_start_of_the_body() {
    let doc = CollabDoc::from_note_text("T", "---\nstage: Idea\n---\nBrief\n");
    assert_eq!(doc.front_len(), "---\nstage: Idea\n---\n".len());
    let sticky = doc.note_sticky(3);
    assert_eq!(doc.note_resolve(&sticky), Some(doc.front_len()));
    let none = CollabDoc::from_note_text("T", "plain\n");
    assert_eq!(none.front_len(), 0);
    assert_eq!(none.note_resolve(&none.note_sticky(2)), Some(2));
}

#[test]
fn a_reordered_property_list_and_a_removed_property_are_taken() {
    let base = "---\na: 1\nb: 2\nc: 3\n---\nx\n";
    let doc = CollabDoc::from_note_text("", base);
    let sent = doc.edit_note_in_parts(base, "---\nc: 3\nb: 2\na: 1\n---\nx\n", PART);
    assert!(!sent.is_empty());
    assert_eq!(doc.note_text(), "---\nc: 3\nb: 2\na: 1\n---\nx\n");
    let before = doc.note_text();
    doc.edit_note_in_parts(&before, "---\nc: 3\na: 1\n---\nx\n", PART);
    assert_eq!(doc.note_text(), "---\nc: 3\na: 1\n---\nx\n");
    // A person removing every property leaves the plain body.
    let before = doc.note_text();
    doc.edit_note_in_parts(&before, "x\n", PART);
    assert_eq!(doc.note_text(), "x\n");
    assert!(doc.props().is_empty());
}

#[test]
fn a_property_removed_by_someone_else_stays_removed_when_the_edit_left_it_alone() {
    let base = "---\na: 1\nb: 2\n---\nx\n";
    let (mine, theirs) = pair(base);
    deliver(&mine, &edit(&theirs, "---\na: 1\n---\nx\n"));
    mine.edit_note_in_parts(base, "---\na: 1\nb: 2\n---\nx\nmore\n", PART);
    assert_eq!(mine.note_text(), "---\na: 1\n---\nx\nmore\n");
}
