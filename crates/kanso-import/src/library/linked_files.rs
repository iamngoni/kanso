//! Notes that follow the file they were opened from (issue #432): opening a
//! Markdown file with File › Open… remembers the file, and a refresh imports
//! it again when it changed on disk. The refresh is one-way, file to note,
//! and goes through [`import_markdown_file`], so a note changed in Kanso is
//! never overwritten: it stops following its file instead.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use kanso_store::Store;
use serde::{Deserialize, Serialize};

use super::import_markdown_file;

/// A note made by opening a file, and the file it follows. Kept on this
/// device only: a path means nothing on another one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedFile {
    pub note: String,
    pub path: PathBuf,
    /// When the file was last changed (ms since 1970) as it was last read;
    /// `None` when that wasn't known, so the next refresh reads it.
    #[serde(default)]
    pub modified_ms: Option<i64>,
}

/// What a refresh did with one linked file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refresh {
    /// The file changed and the note now has its text.
    Updated,
    /// The file is as it was, or reads the same.
    Unchanged,
    /// The note was changed in Kanso, so it was left as it is and stops
    /// following the file.
    EditedInKanso,
    /// The note was deleted in Kanso; the link is dropped.
    NoteDeleted,
    /// The file isn't there (moved, deleted, a drive not mounted). The note
    /// is left as it is and the link kept, for when it comes back.
    Missing,
    /// It couldn't be read or written, with why. The link is kept.
    Failed(String),
}

impl Refresh {
    /// Whether the note keeps following its file after this. Pure.
    pub fn keeps_link(&self) -> bool {
        !matches!(self, Refresh::EditedInKanso | Refresh::NoteDeleted)
    }
}

/// One linked file's refresh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refreshed {
    pub note: String,
    pub path: PathBuf,
    pub refresh: Refresh,
}

/// `links` with `link` recorded: it replaces any link of the same note and
/// any link of the same file, so a note follows one file and a file feeds
/// one note. Two files with the same name in different folders make the
/// same note; the one opened last is the one it follows. Pure.
pub fn record_link(links: &[LinkedFile], link: LinkedFile) -> Vec<LinkedFile> {
    let mut out: Vec<LinkedFile> = links
        .iter()
        .filter(|l| l.note != link.note && l.path != link.path)
        .cloned()
        .collect();
    out.push(link);
    out
}

/// When `path` was last changed, in ms since 1970, if it can be read.
pub fn modified_ms(path: &Path) -> Option<i64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let since = modified.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(since.as_millis()).ok()
}

/// Imports each linked file again whose modification time changed since it
/// was last read. Returns the links to keep, with their new times, and what
/// happened to each one that was looked at. A file whose time is the same
/// is not read and not reported.
pub fn refresh_linked_files(
    links: &[LinkedFile],
    store: &Store,
) -> (Vec<LinkedFile>, Vec<Refreshed>) {
    let mut kept = Vec::with_capacity(links.len());
    let mut done = Vec::new();
    for link in links {
        let modified = modified_ms(&link.path);
        if modified.is_some() && modified == link.modified_ms {
            kept.push(link.clone());
            continue;
        }
        let refresh = if modified.is_none() && !link.path.is_file() {
            Refresh::Missing
        } else {
            refresh_one(link, store)
        };
        if refresh.keeps_link() {
            let modified_ms = match refresh {
                // Read again next time.
                Refresh::Missing | Refresh::Failed(_) => link.modified_ms,
                _ => modified,
            };
            kept.push(LinkedFile {
                modified_ms,
                ..link.clone()
            });
        }
        done.push(Refreshed {
            note: link.note.clone(),
            path: link.path.clone(),
            refresh,
        });
    }
    (kept, done)
}

fn refresh_one(link: &LinkedFile, store: &Store) -> Refresh {
    let report = match import_markdown_file(&link.path, store) {
        Ok(report) => report,
        Err(err) => return Refresh::Failed(format!("{err:#}")),
    };
    if report.first_note.as_deref() != Some(link.note.as_str()) {
        return Refresh::Failed("the file now makes another note".into());
    }
    let notes = &report.notes;
    if notes.kept_deleted > 0 {
        Refresh::NoteDeleted
    } else if notes.kept_edited > 0 {
        Refresh::EditedInKanso
    } else if !report.rejected.is_empty() {
        Refresh::Failed(report.rejected.join("; "))
    } else if notes.updated > 0 || notes.created > 0 {
        Refresh::Updated
    } else {
        Refresh::Unchanged
    }
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;
    use crate::library::open_files::open_note_files_linked;

    fn store(dir: &Path) -> Store {
        Store::open(dir.join("db.sqlite")).unwrap()
    }

    /// Opens `file` and returns its link.
    fn open(file: &Path, store: &Store) -> LinkedFile {
        let opened = open_note_files_linked(&[file.to_path_buf()], store);
        assert!(opened.problems.is_empty(), "{:?}", opened.problems);
        opened.linked.into_iter().next().expect("a link")
    }

    fn body(store: &Store, id: &str) -> String {
        store.note(id).unwrap().unwrap().body
    }

    /// Writes `text` and moves the file's time on, as an editor saving it a
    /// moment later would (file systems keep times coarsely).
    fn change(file: &Path, text: &str, link: &LinkedFile) {
        std::fs::write(file, text).unwrap();
        let later = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_millis(link.modified_ms.unwrap_or(0) as u64 + 5_000);
        std::fs::File::options()
            .write(true)
            .open(file)
            .unwrap()
            .set_modified(later)
            .unwrap();
    }

    #[test]
    fn a_changed_file_updates_its_note() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let file = dir.path().join("plan.md");
        std::fs::write(&file, "# Plan\n\nfirst").unwrap();
        let link = open(&file, &store);
        change(&file, "# Plan\n\nsecond", &link);
        let (kept, done) = refresh_linked_files(std::slice::from_ref(&link), &store);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].refresh, Refresh::Updated);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].modified_ms, modified_ms(&file));
        let text = body(&store, &link.note);
        assert!(text.contains("second") && !text.contains("first"), "{text}");
    }

    #[test]
    fn an_unchanged_file_is_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let file = dir.path().join("plan.md");
        std::fs::write(&file, "# Plan\n\nfirst").unwrap();
        let link = open(&file, &store);
        let rev = store.get(&link.note).unwrap().unwrap().rev;
        let (kept, done) = refresh_linked_files(std::slice::from_ref(&link), &store);
        assert!(done.is_empty());
        assert_eq!(kept, vec![link.clone()]);
        assert_eq!(store.get(&link.note).unwrap().unwrap().rev, rev);
    }

    #[test]
    fn a_note_edited_in_kanso_keeps_its_edit_and_stops_following() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let file = dir.path().join("plan.md");
        std::fs::write(&file, "# Plan\n\nfirst").unwrap();
        let link = open(&file, &store);
        store
            .put(
                DocType::Note,
                json!({"_id": link.note, "body": "mine"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        change(&file, "# Plan\n\nsecond", &link);
        let (kept, done) = refresh_linked_files(std::slice::from_ref(&link), &store);
        assert_eq!(done[0].refresh, Refresh::EditedInKanso);
        assert!(kept.is_empty());
        assert_eq!(body(&store, &link.note), "mine");
    }

    #[test]
    fn a_note_deleted_in_kanso_stays_deleted_and_stops_following() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let file = dir.path().join("plan.md");
        std::fs::write(&file, "# Plan\n\nfirst").unwrap();
        let link = open(&file, &store);
        store.delete(&link.note, Origin::Ui).unwrap();
        change(&file, "# Plan\n\nsecond", &link);
        let (kept, done) = refresh_linked_files(std::slice::from_ref(&link), &store);
        assert_eq!(done[0].refresh, Refresh::NoteDeleted);
        assert!(kept.is_empty());
        assert!(store.note(&link.note).unwrap().is_none());
    }

    #[test]
    fn a_missing_file_leaves_the_note_and_keeps_the_link() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let file = dir.path().join("plan.md");
        std::fs::write(&file, "# Plan\n\nfirst").unwrap();
        let link = open(&file, &store);
        let before = body(&store, &link.note);
        std::fs::remove_file(&file).unwrap();
        let (kept, done) = refresh_linked_files(std::slice::from_ref(&link), &store);
        assert_eq!(done[0].refresh, Refresh::Missing);
        assert_eq!(kept, vec![link.clone()]);
        assert_eq!(body(&store, &link.note), before);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(16)
        })]

        /// Refreshing again with nothing changed on disk reads nothing and
        /// writes nothing, whatever the file said.
        #[test]
        fn prop_refreshing_twice_writes_nothing(first in "[a-z ]{1,40}", second in "[a-z ]{1,40}") {
            let dir = tempfile::tempdir().unwrap();
            let store = store(dir.path());
            let file = dir.path().join("note.md");
            std::fs::write(&file, format!("# Note\n\n{first}")).unwrap();
            let link = open(&file, &store);
            change(&file, &format!("# Note\n\n{second}"), &link);
            let (kept, _) = refresh_linked_files(std::slice::from_ref(&link), &store);
            let rev = store.get(&link.note).unwrap().unwrap().rev;
            let (again, done) = refresh_linked_files(&kept, &store);
            prop_assert!(done.is_empty());
            prop_assert_eq!(again, kept);
            prop_assert_eq!(store.get(&link.note).unwrap().unwrap().rev, rev);
        }
    }

    fn link_strategy() -> impl Strategy<Value = LinkedFile> {
        ("[a-c]", "[x-z]", proptest::option::of(0i64..3)).prop_map(|(note, path, modified_ms)| {
            LinkedFile {
                note,
                path: PathBuf::from(path),
                modified_ms,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// After any recordings, a note follows at most one file and a file
        /// feeds at most one note, and the latest recording is there.
        #[test]
        fn prop_record_link_keeps_one_per_note_and_file(
            links in proptest::collection::vec(link_strategy(), 0..12)
        ) {
            let mut list = Vec::new();
            for link in &links {
                list = record_link(&list, link.clone());
                prop_assert!(list.contains(link));
            }
            let notes: std::collections::HashSet<_> = list.iter().map(|l| &l.note).collect();
            let paths: std::collections::HashSet<_> = list.iter().map(|l| &l.path).collect();
            prop_assert_eq!(notes.len(), list.len());
            prop_assert_eq!(paths.len(), list.len());
            if let Some(last) = links.last() {
                prop_assert_eq!(record_link(&list, last.clone()), list);
            }
        }
    }
}
