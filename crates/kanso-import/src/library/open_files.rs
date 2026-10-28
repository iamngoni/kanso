//! Opening single files the system hands over (File › Open…, a share on a
//! phone): the same rule on every app.

use std::path::{Path, PathBuf};

use kanso_store::Store;
use sha1::{Digest as _, Sha1};

use super::{import_markdown_file, resolve};

/// Imports each Markdown or text file into the Inbox; returns the last note
/// and what went wrong with the others. Opening a file again updates its
/// note.
pub fn open_note_files(paths: &[PathBuf], store: &Store) -> (Option<String>, Vec<String>) {
    let mut last = None;
    let mut problems = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !resolve::is_note_path(&name) {
            problems.push(format!(
                "{name} isn't a Markdown or text file. To bring in another app's export, use Import…"
            ));
            continue;
        }
        match import_markdown_file(path, store) {
            Ok(report) if report.is_clean() => last = report.first_note.or(last),
            Ok(report) => problems.push(format!("{name}: {}", report.rejected.join("; "))),
            Err(err) => problems.push(format!("{name}: {err:#}")),
        }
    }
    (last, problems)
}

/// The folder a document the platform names by `identity` is copied into
/// before it is opened: 16 hex characters of a SHA-1 of the identity. The
/// same identity always gets the same folder, so opening a document again
/// finds the note it made even though each copy has a new temporary path.
/// Pure.
pub fn staging_folder_name(identity: &str) -> String {
    let digest = Sha1::digest(identity.as_bytes());
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

/// Where the copy of a document named `name` with `identity` goes under
/// `staging`. Pure.
pub fn staged_path(staging: &Path, identity: &str, name: &str) -> PathBuf {
    staging.join(staging_folder_name(identity)).join(name)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// The folder is a pure function of the identity: 16 lowercase hex
        /// characters, and the same for the same identity.
        #[test]
        fn prop_staging_folder_is_stable_hex(identity in "\\PC{0,80}") {
            let name = staging_folder_name(&identity);
            prop_assert_eq!(name.len(), 16);
            prop_assert!(name.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
            prop_assert_eq!(name, staging_folder_name(&identity));
        }

        /// Different identities get different folders (a 64-bit hash of a
        /// handful of strings does not collide).
        #[test]
        fn prop_different_identities_get_different_folders(
            identities in proptest::collection::hash_set("[ -~]{1,40}", 2..8)
        ) {
            let names: std::collections::HashSet<_> =
                identities.iter().map(|i| staging_folder_name(i)).collect();
            prop_assert_eq!(names.len(), identities.len());
        }

        /// The staged path ends in the document's name, inside its folder.
        #[test]
        fn prop_staged_path_keeps_the_name(identity in "[ -~]{1,40}", name in "[a-z]{1,8}\\.md") {
            let path = staged_path(Path::new("/s"), &identity, &name);
            prop_assert_eq!(path.file_name().unwrap().to_string_lossy(), name);
            prop_assert_eq!(
                path.parent().unwrap().file_name().unwrap().to_string_lossy(),
                staging_folder_name(&identity)
            );
        }
    }

    // no-prop-test: a file kept in place; the rule is the same loop as the
    // proptested names above.
    #[test]
    fn a_pdf_is_a_problem_and_a_note_is_opened() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("db.sqlite")).unwrap();
        let pdf = dir.path().join("x.pdf");
        std::fs::write(&pdf, "x").unwrap();
        let md = dir.path().join("a.md");
        std::fs::write(&md, "# A\n\nbody").unwrap();
        let (last, problems) = open_note_files(&[pdf, md], &store);
        assert!(last.is_some());
        assert_eq!(problems.len(), 1);
        assert!(problems[0].starts_with("x.pdf isn't a Markdown or text file."));
    }
}
