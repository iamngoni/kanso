//! Linking an unlinked mention: the note that says another's title as words
//! of its own gets a link to it, in one write.

use kanso_core::links::{NOTE_LINK_PREFIX, link_mention as link_in_body};
use kanso_core::{DocType, Note};
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

/// The ID link to note `id`: what a link to it is written as.
pub fn note_uri(id: &str) -> String {
    format!(
        "{NOTE_LINK_PREFIX}{}",
        id.strip_prefix("note:").unwrap_or(id)
    )
}

/// `mention`'s text with the first mention of note `target`'s title on
/// `line` (0-based) made a link to it, keeping the words as they were
/// written. `None` when the target has no title to mention or the line has
/// no such mention: nothing to change. Every other line is as it was.
pub fn mention_linked(
    store: &Store,
    target: &str,
    mention_body: &str,
    line: u32,
) -> Result<Option<String>, StoreError> {
    let Some(title) = store.note(target)?.map(|n| n.title) else {
        return Ok(None);
    };
    Ok(link_in_body(mention_body, line, &title, &note_uri(target)))
}

/// Makes the mention on `line` in note `mention` a link to note `target`, in
/// one write, and says the note's new revision. `None` when there was no
/// mention there (the note is not written).
///
/// A locked note's text is sealed by the app; this writes plain text, so it
/// is for notes that aren't locked.
pub fn link_mention(
    store: &Store,
    target: &str,
    mention: &str,
    line: u32,
) -> Result<Option<String>, StoreError> {
    let Some(note): Option<Note> = store.note(mention)? else {
        return Ok(None);
    };
    let Some(body) = mention_linked(store, target, &note.body, line)? else {
        return Ok(None);
    };
    let written = store.put(
        DocType::Note,
        json!({"_id": mention, "_rev": note.rev, "body": body}),
        Put::Merge,
        Origin::Ui,
    )?;
    Ok(Some(written.rev))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::links::note_links;
    use proptest::prelude::*;

    fn store() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        // A note needs a book that exists (the store checks the reference).
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        (dir, store, book)
    }

    fn put(store: &Store, book: &str, title: &str, body: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

        /// Linking a mention writes a link to the target on that line and
        /// leaves every other line as it was; the mention's words are kept.
        #[test]
        fn prop_link_mention_changes_only_its_line(
            title in "[A-Z][a-z]{3,8}",
            before in proptest::collection::vec("[a-z ]{0,12}", 0..4),
            after in proptest::collection::vec("[a-z ]{0,12}", 0..4),
            lead in "[a-z]{0,6} ",
            tail in " [a-z]{0,6}",
        ) {
            let (_dir, store, book) = store();
            let target = put(&store, &book, &title, "");
            let line = format!("{lead}{}{tail}", title.to_lowercase());
            let mut lines = before.clone();
            lines.push(line.clone());
            lines.extend(after.clone());
            let body = lines.join("\n");
            let mention = put(&store, &book, "Other", &body);
            let at = before.len() as u32;
            let rev = link_mention(&store, &target, &mention, at).unwrap();
            prop_assert!(rev.is_some());
            let written = store.note(&mention).unwrap().unwrap();
            let new_lines: Vec<&str> = written.body.split('\n').collect();
            prop_assert_eq!(new_lines.len(), lines.len());
            for (ix, (old, new)) in lines.iter().zip(&new_lines).enumerate() {
                if ix as u32 != at {
                    prop_assert_eq!(old.as_str(), *new);
                }
            }
            let words = title.to_lowercase();
            let expected = format!("[{words}]({})", note_uri(&target));
            prop_assert!(new_lines[at as usize].contains(&expected), "{:?}", new_lines);
            // The note now links to the target, so no mention is left on
            // that line to link a second time.
            prop_assert!(note_links(&written.body).iter().any(|l| l.line == at));
            prop_assert_eq!(link_mention(&store, &target, &mention, at).unwrap(), None);
            prop_assert_eq!(store.note(&mention).unwrap().unwrap().body, written.body);
        }

        /// A line with no mention is a no-op: nothing is written.
        #[test]
        fn prop_no_mention_writes_nothing(
            title in "[A-Z][a-z]{3,8}",
            body in "[0-9 ]{0,20}(\n[0-9 ]{0,10}){0,3}",
            line in 0u32..6,
        ) {
            let (_dir, store, book) = store();
            let target = put(&store, &book, &title, "");
            let mention = put(&store, &book, "Other", &body);
            let before = store.note(&mention).unwrap().unwrap();
            prop_assert_eq!(link_mention(&store, &target, &mention, line).unwrap(), None);
            let after = store.note(&mention).unwrap().unwrap();
            prop_assert_eq!(before.rev, after.rev);
            prop_assert_eq!(before.body, after.body);
        }
    }

    // no-prop-test: a fixed note and the exact bytes written; the property
    // tests above cover the shape for any text.
    #[test]
    fn the_mention_keeps_its_case_and_neighbours() {
        let (_dir, store, book) = store();
        let target = put(&store, &book, "Zettelkasten", "");
        let mention = put(
            &store,
            &book,
            "Other",
            "intro\nI use my zettelkasten daily.\noutro",
        );
        assert!(
            link_mention(&store, &target, &mention, 1)
                .unwrap()
                .is_some()
        );
        let body = store.note(&mention).unwrap().unwrap().body;
        assert_eq!(
            body,
            format!(
                "intro\nI use my [zettelkasten]({}) daily.\noutro",
                note_uri(&target)
            )
        );
    }
}
