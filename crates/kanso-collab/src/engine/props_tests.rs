//! Shared single notes with properties merged one key at a time, through
//! the open editor: its caret and a comment's place stay on their words when
//! another person's change to the front matter moves the body inside the
//! note's text. Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use proptest::prelude::*;

use super::tests::{Device, body_of, device, note, sync, write_body};
use crate::doc::{Field, apply_splices, remap};
use crate::memory::MemoryCollab;
use crate::model::{Placement, Role};

/// A shares a note with B (who edits) and B accepts; the share's id.
fn shared(owner: &Device, other: &Device, body: &str) -> String {
    let id = note(&owner.store, "note:shared", "Plan", body);
    owner
        .engine
        .person(&other.engine.caller().username)
        .unwrap();
    let share = owner
        .engine
        .share(&id, &[(other.engine.caller().username.clone(), Role::Edit)])
        .unwrap();
    other.engine.refresh().unwrap();
    other.engine.accept(&share).unwrap();
    // Both apps say they keep properties as entries, and the note moves
    // over with the owner's next edit.
    sync(&[owner, other], &share);
    let id = id_of(owner, &share);
    write_body(&owner.store, &id, &format!("{body}\n")).unwrap();
    sync(&[owner, other], &share);
    write_body(&owner.store, &id, body).unwrap();
    sync(&[owner, other], &share);
    share
}

const NOTE: &str = "---\nstage: Idea\ndue: 2026-01-01\n---\nBrief for the shoot\n";

fn id_of(d: &Device, share: &str) -> String {
    d.store.shared_note(share).unwrap().unwrap().note_id
}

#[test]
fn a_shared_notes_properties_are_entries_of_its_document_not_text() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &b, NOTE);
    for d in [&a, &b] {
        let loaded = d.engine.loaded(&share).unwrap();
        let guard = super::lock(&loaded);
        let doc = guard.doc.text().unwrap();
        assert_eq!(doc.props().len(), 2);
        assert_eq!(doc.text(Field::Body), "Brief for the shoot\n");
        assert_eq!(doc.note_text(), NOTE);
    }
    assert_eq!(body_of(&b, &share), NOTE);
    // Two people change different properties offline: both stay, and moving
    // the same card gives one whole stage.
    write_body(
        &a.store,
        &id_of(&a, &share),
        &NOTE.replace("Idea", "Shooting"),
    )
    .unwrap();
    write_body(
        &b.store,
        &id_of(&b, &share),
        &NOTE.replace("2026-01-01", "2026-02-02"),
    )
    .unwrap();
    sync(&[&a, &b], &share);
    let want = "---\nstage: Shooting\ndue: 2026-02-02\n---\nBrief for the shoot\n";
    assert_eq!(body_of(&a, &share), want);
    assert_eq!(body_of(&b, &share), want);
    write_body(
        &a.store,
        &id_of(&a, &share),
        &want.replace("Shooting", "Editing"),
    )
    .unwrap();
    write_body(
        &b.store,
        &id_of(&b, &share),
        &want.replace("Shooting", "Review"),
    )
    .unwrap();
    sync(&[&a, &b], &share);
    let (ta, tb) = (body_of(&a, &share), body_of(&b, &share));
    assert_eq!(ta, tb);
    assert!(
        ta.contains("stage: Editing\n") || ta.contains("stage: Review\n"),
        "{ta:?}"
    );
}

/// What an app that keeps a note's front matter in its body reads: the
/// document's body text, whole.
fn body_field(d: &Device, share: &str) -> String {
    let loaded = d.engine.loaded(share).unwrap();
    let guard = super::lock(&loaded);
    guard.doc.text().unwrap().text(Field::Body)
}

fn props_len(d: &Device, share: &str) -> usize {
    let loaded = d.engine.loaded(share).unwrap();
    let guard = super::lock(&loaded);
    guard.doc.text().unwrap().props().len()
}

#[test]
fn a_shared_note_stays_text_until_everyone_who_edits_has_an_app_that_reads_properties() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let id = note(&a.store, "note:shared", "Plan", NOTE);
    a.engine.person("user2").unwrap();
    let share = a
        .engine
        .share(&id, &[("user2".to_owned(), Role::Edit)])
        .unwrap();
    // B hasn't opened it yet (its app may be an older one): the front matter
    // stays in the body, where an older app reads it, through edits.
    assert_eq!(body_field(&a, &share), NOTE);
    write_body(&a.store, &id, &NOTE.replace("Idea", "Shooting")).unwrap();
    sync(&[&a], &share);
    assert_eq!(props_len(&a, &share), 0);
    assert_eq!(
        body_field(&a, &share),
        NOTE.replace("Idea", "Shooting"),
        "an older app still reads its properties"
    );
    // B's app opens it and says it keeps properties as entries; A's next
    // edit moves the note over.
    b.engine.refresh().unwrap();
    b.engine.accept(&share).unwrap();
    sync(&[&b, &a], &share);
    write_body(&a.store, &id, &NOTE.replace("Idea", "Editing")).unwrap();
    sync(&[&a, &b], &share);
    assert_eq!(props_len(&a, &share), 2);
    assert_eq!(body_field(&a, &share), "Brief for the shoot\n");
    assert_eq!(body_of(&b, &share), NOTE.replace("Idea", "Editing"));
}

#[test]
fn people_who_only_read_dont_hold_a_note_back_and_a_new_one_starts_as_text() {
    let server = MemoryCollab::new();
    let (a, c) = (device(&server, 1, 1), device(&server, 3, 1));
    let id = note(&a.store, "note:shared", "Plan", NOTE);
    a.engine.person("user3").unwrap();
    let share = a
        .engine
        .share(&id, &[("user3".to_owned(), Role::View)])
        .unwrap();
    // New shares start with the front matter in the body.
    assert_eq!(props_len(&a, &share), 0);
    assert_eq!(body_field(&a, &share), NOTE);
    c.engine.refresh().unwrap();
    c.engine.accept(&share).unwrap();
    // A reader can't say anything in the note and doesn't need to: the
    // owner is the only one who edits, so the note moves over on its edit.
    write_body(&a.store, &id, &NOTE.replace("Idea", "Editing")).unwrap();
    sync(&[&a, &c], &share);
    assert_eq!(props_len(&a, &share), 2);
    assert_eq!(body_of(&c, &share), NOTE.replace("Idea", "Editing"));
}

#[test]
fn an_editor_cant_say_an_older_app_reads_properties_for_it() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let id = note(&a.store, "note:shared", "Plan", NOTE);
    a.engine.person("user2").unwrap();
    let share = a
        .engine
        .share(&id, &[("user2".to_owned(), Role::Edit)])
        .unwrap();
    // B hasn't opened it (its app is an older one). A writes B's word with
    // A's own key: it doesn't count, and the note stays text.
    {
        let loaded = a.engine.loaded(&share).unwrap();
        let guard = super::lock(&loaded);
        let b_id = b.engine.caller().account_id.clone();
        let forged = a
            .engine
            .identity()
            .unwrap()
            .sign(&crate::doc_props::parts_ack_message(&share, &b_id));
        guard.doc.text().unwrap().ack_parts(&b_id, &forged);
    }
    write_body(&a.store, &id, &NOTE.replace("Idea", "Editing")).unwrap();
    sync(&[&a], &share);
    assert_eq!(props_len(&a, &share), 0);
    assert_eq!(body_field(&a, &share), NOTE.replace("Idea", "Editing"));
}

#[test]
fn half_typed_front_matter_in_the_editor_holds_across_keystrokes_and_wipes_nothing() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &b, NOTE);
    b.engine.attach(&share).unwrap();
    // Keystroke 1: a property typed without its colon yet.
    let half = NOTE.replace("due:", "sta\ndue:");
    let merge = b.engine.sync_editor(&share, "Plan", &half).unwrap();
    assert!(merge.body.is_empty());
    assert_eq!(merge.body_text, half);
    // Another device changes a key meanwhile.
    write_body(
        &a.store,
        &id_of(&a, &share),
        &NOTE.replace("Idea", "Shooting"),
    )
    .unwrap();
    sync(&[&a], &share);
    // Keystroke 2: still half typed, with a body edit.
    let more = format!("{half}More\n");
    let merge = b.engine.sync_editor(&share, "Plan", &more).unwrap();
    assert!(merge.body.is_empty());
    assert_eq!(merge.body_text, more);
    sync(&[&b, &a], &share);
    assert_eq!(props_len(&a, &share), 2);
    assert_eq!(props_len(&b, &share), 2);
    assert_eq!(body_of(&a, &share), NOTE.replace("Idea", "Shooting"));
    // The property is finished: everything typed lands, once.
    let done = format!("{}More\n", NOTE.replace("due:", "sta: x\ndue:"));
    b.engine.sync_editor(&share, "Plan", &done).unwrap();
    sync(&[&b, &a], &share);
    let want = "---\nstage: Shooting\nsta: x\ndue: 2026-01-01\n---\nBrief for the shoot\nMore\n";
    assert_eq!(body_of(&a, &share), want);
    assert_eq!(body_of(&b, &share), want);
    b.engine.detach(&share);
}

#[test]
fn a_held_edit_in_the_open_editor_is_saved_and_survives_close_quit_and_reload() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &b, NOTE);
    b.engine.attach(&share).unwrap();
    // The front matter is broken and the body edited: held, and saved as
    // typed (the desktop's save and the phones' sync both go through
    // `sync_editor`).
    let typed = format!("{}More words\n", NOTE.replace("due:", "sta\ndue:"));
    let merge = b.engine.sync_editor(&share, "Plan", &typed).unwrap();
    assert!(merge.body.is_empty());
    assert_eq!(body_of(&b, &share), typed);
    // The editor closes, then the app quits and starts again: the library
    // still has what was typed, and nothing was sent or wiped.
    b.engine.detach(&share);
    assert_eq!(body_of(&b, &share), typed);
    b.engine.unload(&share);
    sync(&[&b, &a], &share);
    assert_eq!(body_of(&b, &share), typed);
    assert_eq!(props_len(&a, &share), 2);
    assert_eq!(props_len(&b, &share), 2);
    assert_eq!(body_of(&a, &share), NOTE);
    // Another device changes a key meanwhile; the held text stays as typed.
    write_body(
        &a.store,
        &id_of(&a, &share),
        &NOTE.replace("Idea", "Shooting"),
    )
    .unwrap();
    sync(&[&a, &b], &share);
    assert_eq!(body_of(&b, &share), typed);
    // The front matter is fixed, in the library (the editor is closed): the
    // whole edit lands everywhere, the body edit included.
    let fixed = format!("{}More words\n", NOTE.replace("due:", "sta: x\ndue:"));
    write_body(&b.store, &id_of(&b, &share), &fixed).unwrap();
    sync(&[&b, &a], &share);
    sync(&[&a, &b], &share);
    let want =
        "---\nstage: Shooting\nsta: x\ndue: 2026-01-01\n---\nBrief for the shoot\nMore words\n";
    assert_eq!(body_of(&a, &share), want);
    assert_eq!(body_of(&b, &share), want);
}

#[test]
fn the_editors_caret_and_a_comment_stay_on_their_words_when_the_front_matter_changes() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &b, NOTE);
    // B has the note open, the caret before "shoot".
    b.engine.attach(&share).unwrap();
    let mut editor = body_of(&b, &share);
    let caret = editor.find("shoot").unwrap();
    b.engine
        .add_comment(&share, caret..caret + 5, "Which?")
        .unwrap();
    // A moves the card to a longer stage and adds a property.
    let longer =
        "---\nstage: In production\ndue: 2026-01-01\nowner: Dan\n---\nBrief for the shoot\n";
    write_body(&a.store, &id_of(&a, &share), longer).unwrap();
    sync(&[&a], &share);
    sync(&[&b], &share);
    let merge = b.engine.sync_editor(&share, "Plan", &editor).unwrap();
    assert_eq!(merge.body_text, longer);
    apply_splices(&mut editor, &merge.body);
    assert_eq!(editor, longer);
    let moved = remap(caret, &merge.body);
    assert!(
        editor[moved..].starts_with("shoot"),
        "{editor:?} at {moved}"
    );
    // The comment sits on the same word.
    let comments = b.engine.comments(&share).unwrap();
    let Placement::Anchored(range) = &comments[0].placement else {
        panic!("{:?}", comments[0].placement)
    };
    assert_eq!(&longer[range.clone()], "shoot");
    assert_eq!(range.start, moved);
}

#[test]
fn typing_in_the_editor_and_a_property_change_elsewhere_both_land() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &b, NOTE);
    b.engine.attach(&share).unwrap();
    let mut editor = body_of(&b, &share);
    // B types at the end of the body and in a property's value.
    editor = editor.replace("Idea", "Idea, maybe") + "More\n";
    let caret = editor.len();
    // A changes another property.
    write_body(
        &a.store,
        &id_of(&a, &share),
        &NOTE.replace("2026-01-01", "2026-03-03"),
    )
    .unwrap();
    sync(&[&a], &share);
    b.engine.sync_editor(&share, "Plan", &editor).unwrap();
    sync(&[&b, &a], &share);
    let merge = b.engine.sync_editor(&share, "Plan", &editor).unwrap();
    apply_splices(&mut editor, &merge.body);
    let want = "---\nstage: Idea, maybe\ndue: 2026-03-03\n---\nBrief for the shoot\nMore\n";
    assert_eq!(editor, want);
    assert_eq!(remap(caret, &merge.body), want.len());
    sync(&[&a, &b], &share);
    assert_eq!(body_of(&a, &share), want);
    b.engine.detach(&share);
}

proptest! {
    #![proptest_config(crate::tests::config(24))]

    /// The caret in the open editor stays on its character through any
    /// changes another person makes to the properties, and the editor's text
    /// ends as the library's and the document's.
    #[test]
    fn prop_the_caret_keeps_its_character_through_property_edits(
        at in any::<prop::sample::Index>(),
        rounds in prop::collection::vec((0u8..3, "[a-z0-9]{1,10}", any::<bool>()), 1..5),
    ) {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let share = shared(&a, &b, NOTE);
        b.engine.attach(&share).unwrap();
        let mut editor = body_of(&b, &share);
        let body_start = editor.find("Brief").unwrap();
        let mut caret = body_start + at.index(editor.len() - body_start);
        while !editor.is_char_boundary(caret) { caret -= 1; }
        let wanted = editor[caret..].chars().next();
        for (which, word, added) in rounds {
            let now = body_of(&a, &share);
            let next = match which {
                0 => now.replacen("stage: ", &format!("stage: {word} "), 1),
                1 => now.replacen("due: ", &format!("due: {word} "), 1),
                _ if added => now.replacen("---\n", &format!("---\nextra{}: {word}\n", word.len()), 1),
                _ => now.replacen("stage: Idea\n", "", 1),
            };
            write_body(&a.store, &id_of(&a, &share), &next).unwrap();
            sync(&[&a], &share);
            sync(&[&b], &share);
            let merge = b.engine.sync_editor(&share, "Plan", &editor).unwrap();
            apply_splices(&mut editor, &merge.body);
            caret = remap(caret, &merge.body);
            prop_assert_eq!(&editor, &merge.body_text);
            prop_assert_eq!(&editor, &body_of(&b, &share));
            prop_assert_eq!(editor[caret..].chars().next(), wanted);
        }
        // Everyone ends with the same note, and its text is the document's.
        sync(&[&a, &b], &share);
        prop_assert_eq!(body_of(&a, &share), body_of(&b, &share));
        let loaded = b.engine.loaded(&share).unwrap();
        let guard = super::lock(&loaded);
        prop_assert_eq!(guard.doc.text().unwrap().note_text(), body_of(&b, &share));
    }
}
