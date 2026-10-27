//! Attaching a photo or a PDF from the editor: where it goes in the text,
//! that its file is readable, that Undo takes it out, and that a locked
//! note and a file that is neither are refused.

mod common;

use common::*;
use kanso_mobile::{KansoError, Range16, ShareFile};
use proptest::prelude::*;

const PDF: &[u8] = b"%PDF-1.7\n%%EOF\n";
/// The first bytes of a PNG, which is how the library tells one.
const PNG: &[u8] =
    b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89";

fn file(name: &str, bytes: &[u8]) -> ShareFile {
    ShareFile {
        name: name.to_owned(),
        content_type: None,
        bytes: bytes.to_vec(),
    }
}

fn caret(at: u32) -> Range16 {
    Range16 { start: at, end: at }
}

/// The attachment ID in the first `inkdrop://file:…` link of `text`.
fn linked_id(text: &str) -> String {
    let start = text.find("inkdrop://").expect("a link") + "inkdrop://".len();
    let end = text[start..].find(')').expect("the link's end") + start;
    text[start..end].to_owned()
}

#[test]
fn a_photo_is_a_paragraph_of_its_own_and_undo_takes_it_out() {
    let phone = lone_phone();
    let id = note(&phone.library, "Trip", "Day one");
    let session = phone.library.edit(&id).unwrap();
    session.select(caret(7), false).unwrap();

    let out = session.attach(file("IMG_0042.png", PNG)).unwrap();
    let text = session.snapshot().unwrap().text;
    assert!(text.starts_with("Day one\n\n![](inkdrop://file:"), "{text}");
    assert!(text.ends_with(')'), "{text}");
    assert!(out.version > 0);
    // The viewer reads only what a saved note links.
    session.flush().unwrap();
    let attachment = phone.library.attachment(&linked_id(&text)).unwrap();
    assert_eq!(attachment.bytes, PNG);

    session.undo().unwrap();
    assert_eq!(session.snapshot().unwrap().text, "Day one");
}

#[test]
fn a_pdf_is_linked_by_its_name() {
    let phone = lone_phone();
    let id = note(&phone.library, "Taxes", "");
    let session = phone.library.edit(&id).unwrap();
    session.attach(file("Return.pdf", PDF)).unwrap();
    let text = session.snapshot().unwrap().text;
    assert!(text.starts_with("[Return.pdf](inkdrop://file:"), "{text}");
}

#[test]
fn a_file_that_is_neither_is_refused_in_words_and_the_text_is_kept() {
    let phone = lone_phone();
    let id = note(&phone.library, "Notes", "Kept");
    let session = phone.library.edit(&id).unwrap();
    let refused = session.attach(file("notes.zip", &[0x50, 0x4b, 3, 4]));
    assert!(matches!(refused, Err(KansoError::Invalid { message }) if !message.is_empty()));
    assert_eq!(session.snapshot().unwrap().text, "Kept");
}

#[test]
fn a_locked_note_takes_no_attachment() {
    let phone = lone_phone();
    let id = note(&phone.library, "Secret", "Words");
    phone
        .library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    let session = phone.library.edit(&id).unwrap();
    let refused = session.attach(file("IMG.png", PNG));
    assert!(
        matches!(refused, Err(KansoError::Invalid { .. })),
        "{refused:?}"
    );
    assert_eq!(session.snapshot().unwrap().text, "Words");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Wherever the caret is, the attachment ends up a paragraph of its
    /// own, only newlines are added around it, and the caret is after it.
    #[test]
    fn prop_an_attachment_stands_alone_wherever_it_goes(
        body in "[a-z \n]{0,16}",
        at in any::<prop::sample::Index>(),
    ) {
        let phone = lone_phone();
        let id = note(&phone.library, "Note", &body);
        let session = phone.library.edit(&id).unwrap();
        let before = session.snapshot().unwrap().text;
        let caret_at = at.index(before.len() + 1) as u32;
        session.select(caret(caret_at), false).unwrap();
        session.attach(file("a.pdf", PDF)).unwrap();
        let snapshot = session.snapshot().unwrap();
        let text = snapshot.text;
        let start = text.find("[a.pdf](").unwrap();
        let end = start + text[start..].find(')').unwrap() + 1;
        prop_assert!(start == 0 || text[..start].ends_with("\n\n"));
        prop_assert!(end == text.len() || text[end..].starts_with("\n\n"));
        prop_assert_eq!(snapshot.selection, caret(end as u32));
        let mut rest = text.clone();
        rest.replace_range(start..end, "");
        prop_assert_eq!(rest.replace('\n', ""), before.replace('\n', ""));
    }
}
