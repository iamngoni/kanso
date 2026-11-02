//! The plugin sheets' decisions, apart from drawing: what the install
//! button says and when it works, the grant the person's choices make, and
//! how a note on the review sheet is summed up.

use std::collections::BTreeSet;

use kanso_plugins::batch::ReviewNote;
use kanso_plugins::model::{Grant, Manifest};

/// What the install button says. Pure.
pub fn install_label(update: bool, key_changed: bool) -> &'static str {
    match (update, key_changed) {
        (_, true) => "Install anyway (key changed)",
        (true, false) => "Update",
        (false, false) => "Install",
    }
}

/// Whether the install button works. A developer folder needs its path
/// typed exactly and the trust box ticked; anything else is always ready.
/// Pure.
pub fn install_enabled(folder: Option<&str>, typed: &str, trusted: bool) -> bool {
    match folder {
        None => true,
        Some(path) => trusted && !path.is_empty() && typed == path,
    }
}

/// The grant the sheet's choices make from `base`: the ticked notebooks
/// (cut to those offered), every notebook if asked for and switched on,
/// and the review switch. Notebooks it may change are the ticked ones when
/// the manifest asks to change notes. Never grants beyond `manifest`. Pure.
pub fn chosen_grant(
    base: &Grant,
    manifest: &Manifest,
    offered: &[String],
    ticked: &BTreeSet<String>,
    all: bool,
    ask_before_changing: bool,
) -> Grant {
    let asks = &manifest.permissions.notes;
    let mut grant = base.clone();
    grant.ask_before_changing = ask_before_changing;
    let books: BTreeSet<String> = offered
        .iter()
        .filter(|id| ticked.contains(*id))
        .cloned()
        .collect();
    let may_read_all = asks.read.as_ref().is_some_and(|r| r.all);
    grant.notes.read_all = all && may_read_all;
    if asks.read.is_some() {
        grant.notes.read_books = books.clone();
    }
    grant.notes.write_books = if asks.write.is_empty() || asks.read.is_none() {
        BTreeSet::new()
    } else {
        books
    };
    if grant.notes.write_books.is_empty() {
        grant.notes.create = false;
    } else {
        grant.notes.create = asks.create;
    }
    grant
}

/// The ticks the sheet starts with: the notebooks the grant already gives,
/// among those offered. Pure.
pub fn starting_ticks(grant: &Grant, offered: &[String]) -> BTreeSet<String> {
    offered
        .iter()
        .filter(|id| grant.notes.read_books.contains(*id) || grant.notes.write_books.contains(*id))
        .cloned()
        .collect()
}

/// One note on the review sheet in a line: "Text, Tags · +3 −1". Pure.
pub fn note_summary(note: &ReviewNote) -> String {
    let what = if note.changes.is_empty() {
        "Changed".to_owned()
    } else {
        note.changes.join(", ")
    };
    match (note.added, note.removed) {
        (0, 0) => what,
        (a, 0) => format!("{what} · +{a}"),
        (0, r) => format!("{what} · −{r}"),
        (a, r) => format!("{what} · +{a} −{r}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{chosen_grant, install_enabled, install_label, note_summary, starting_ticks};
    use kanso_plugins::batch::ReviewNote;
    use kanso_plugins::model::{
        Grant, Manifest, NotesRequest, Permissions, Publisher, ReadRequest,
    };
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn manifest(read: bool, all: bool, write: bool) -> Manifest {
        Manifest {
            schema: 1,
            api_version: 1,
            id: "com.example.x".into(),
            name: "X".into(),
            version: "1.0.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "P".into(),
                key: "ed25519:00".into(),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions: Permissions {
                notes: NotesRequest {
                    read: read.then(|| ReadRequest {
                        all,
                        ..ReadRequest::default()
                    }),
                    write: if write { vec!["x".into()] } else { Vec::new() },
                    create: write,
                },
                ..Permissions::default()
            },
        }
    }

    fn base() -> Grant {
        kanso_plugins::grant::initial(&manifest(true, true, true))
    }

    fn note(changes: Vec<&'static str>, added: usize, removed: usize) -> ReviewNote {
        ReviewNote {
            id: None,
            title: "T".into(),
            changes,
            added,
            removed,
            lines: Vec::new(),
        }
    }

    #[test]
    fn labels() {
        assert_eq!(install_label(false, false), "Install");
        assert_eq!(install_label(true, false), "Update");
        assert_eq!(install_label(true, true), "Install anyway (key changed)");
        assert_eq!(install_label(false, true), "Install anyway (key changed)");
    }

    #[test]
    fn a_developer_folder_needs_the_path_and_the_tick() {
        assert!(install_enabled(None, "", false));
        assert!(!install_enabled(Some("/a/b"), "/a/b", false));
        assert!(!install_enabled(Some("/a/b"), "/a/", true));
        assert!(!install_enabled(Some("/a/b"), "/A/B", true));
        assert!(install_enabled(Some("/a/b"), "/a/b", true));
    }

    #[test]
    fn the_grant_follows_the_ticks() {
        let offered = vec!["book:a".to_owned(), "book:b".to_owned()];
        let ticked: BTreeSet<String> = ["book:a".to_owned(), "book:gone".to_owned()].into();
        let grant = chosen_grant(
            &base(),
            &manifest(true, true, true),
            &offered,
            &ticked,
            false,
            false,
        );
        assert_eq!(grant.notes.read_books, ["book:a".to_owned()].into());
        assert_eq!(grant.notes.write_books, ["book:a".to_owned()].into());
        assert!(!grant.ask_before_changing);
        assert!(grant.notes.create);
        let none = chosen_grant(
            &base(),
            &manifest(true, true, true),
            &offered,
            &BTreeSet::new(),
            true,
            true,
        );
        assert!(none.notes.read_all && !none.notes.create);
        assert_eq!(
            starting_ticks(&grant, &offered),
            ["book:a".to_owned()].into()
        );
    }

    #[test]
    fn summaries() {
        assert_eq!(
            note_summary(&note(vec!["Text", "Tags"], 3, 1)),
            "Text, Tags · +3 −1"
        );
        assert_eq!(note_summary(&note(vec!["Title"], 0, 0)), "Title");
        assert_eq!(note_summary(&note(vec![], 2, 0)), "Changed · +2");
        assert_eq!(note_summary(&note(vec!["Text"], 0, 4)), "Text · −4");
    }

    proptest! {
        /// Invariant: the key-changed label appears exactly when the key changed.
        #[test]
        fn prop_the_label_marks_a_changed_key(update: bool, changed: bool) {
            prop_assert_eq!(install_label(update, changed).contains("key changed"), changed);
        }

        /// Invariant: a developer folder installs only when the typed path
        /// equals the folder and the box is ticked; other sources always.
        #[test]
        fn prop_a_folder_needs_both(path in "[a-z/]{1,12}", typed in "[a-z/]{0,12}", tick: bool) {
            prop_assert_eq!(
                install_enabled(Some(&path), &typed, tick),
                tick && typed == path
            );
            prop_assert!(install_enabled(None, &typed, tick));
        }

        /// Invariant: whatever is ticked, the grant stays inside the manifest
        /// and never names a notebook that wasn't offered.
        #[test]
        fn prop_the_grant_stays_inside(
            read: bool, all: bool, write: bool, switch: bool, ask: bool,
            ticks in proptest::collection::btree_set("book:[a-d]", 0..4),
        ) {
            let m = manifest(read, all, write);
            let offered: Vec<String> = ["book:a", "book:b", "book:c"].map(String::from).to_vec();
            let start = kanso_plugins::grant::initial(&m);
            let grant = chosen_grant(&start, &m, &offered, &ticks, switch, ask);
            prop_assert!(kanso_plugins::grant::check(&grant, &m, &Default::default()).is_ok());
            prop_assert!(grant.notes.read_books.iter().all(|b| offered.contains(b)));
            prop_assert_eq!(grant.ask_before_changing, ask);
        }

        /// Invariant: a summary always carries the counts that are not zero.
        #[test]
        fn prop_the_summary_has_the_counts(added in 0usize..99, removed in 0usize..99) {
            let text = note_summary(&note(vec!["Text"], added, removed));
            prop_assert_eq!(text.contains(&format!("+{added}")), added > 0);
            prop_assert_eq!(text.contains(&format!("−{removed}")), removed > 0);
        }
    }
}
