//! What Empty Trash asks (issue #44). Locked notes are deleted with the rest,
//! so the question counts them, and says their sealed attachments go too.

use kanso_store::TrashCounts;

/// The confirmation's title and description for `counts`. The line about
/// locked notes appears only when there are some.
pub fn empty_trash_question(counts: TrashCounts) -> (String, String) {
    let notes = plural(counts.notes, "note");
    let title = if counts.locked == 0 {
        format!("Delete {notes} in the trash permanently?")
    } else {
        let (locked, theirs) = if counts.locked == 1 {
            ("1 locked note".to_owned(), "its")
        } else {
            (format!("{} locked notes", counts.locked), "their")
        };
        format!("Delete {notes} permanently, including {locked} and {theirs} sealed attachments?")
    };
    (title, "They can't be restored afterwards.".to_owned())
}

fn plural(n: u32, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_locked_line_appears_only_with_locked_notes() {
        let plain = empty_trash_question(TrashCounts {
            notes: 12,
            locked: 0,
        });
        assert_eq!(plain.0, "Delete 12 notes in the trash permanently?");
        let locked = empty_trash_question(TrashCounts {
            notes: 12,
            locked: 2,
        });
        assert_eq!(
            locked.0,
            "Delete 12 notes permanently, including 2 locked notes and their sealed attachments?"
        );
        let one = empty_trash_question(TrashCounts {
            notes: 3,
            locked: 1,
        });
        assert!(
            one.0
                .contains("including 1 locked note and its sealed attachments")
        );
    }

    proptest! {
        /// Invariant: the question names exactly the counts it is given.
        #[test]
        fn prop_the_question_counts_what_it_is_given(notes in 1u32..10_000, locked in 0u32..10_000) {
            let locked = locked.min(notes);
            let (title, _) = empty_trash_question(TrashCounts { notes, locked });
            prop_assert!(title.contains(&notes.to_string()));
            prop_assert_eq!(title.contains("locked"), locked > 0);
            if locked > 0 {
                let expected = format!("{locked} locked note");
                prop_assert!(title.contains(&expected));
            }
        }
    }
}
