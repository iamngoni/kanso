//! Apple Notes notes that were exported as files: a folder (or zip) of
//! `.html`, `.rtf`, `.txt`, `.md` and `.pdf` documents, the way a phone can
//! hand them over without reading the Notes library (which only a Mac can).
//!
//! Everything lands under one parent notebook, "Apple Notes", with the
//! export's own folders as notebooks inside it. HTML, RTF, text and Markdown
//! become Markdown notes (checklists as tasks); a PDF becomes a note that
//! links the PDF, which is attached. A note's key is its path, so importing
//! again updates it.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use super::html::{html_title, normalize, remove_blocks, rewrite_doc_links};
use super::notes_html::{self, DRAWING};
use super::rtf;
use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote};

/// The parent notebook every note of the export goes under.
pub const NAME: &str = "Apple Notes";

const MAX_NOTE: u64 = 8 * 1024 * 1024;

/// What a file in the export is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    Html,
    Rtf,
    Text,
    Markdown,
    Pdf,
}

/// The form of a note file, by its extension. Pure.
fn form_of(path: &str) -> Option<Form> {
    match extension(path).as_str() {
        "html" | "htm" => Some(Form::Html),
        "rtf" => Some(Form::Rtf),
        "txt" | "text" => Some(Form::Text),
        "md" | "markdown" => Some(Form::Markdown),
        "pdf" => Some(Form::Pdf),
        _ => None,
    }
}

/// Whether `path` is junk a file manager leaves beside the notes.
fn is_junk(path: &str) -> bool {
    path.split('/')
        .any(|part| part.starts_with('.') || part == "__MACOSX")
}

pub struct AppleNotesFilesSource {
    tree: Tree,
    notes: Vec<(TreeFile, Form)>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    targets: HashMap<String, String>,
}

/// Opens a folder, a zip, or one note file.
pub fn open(path: &Path) -> anyhow::Result<AppleNotesFilesSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else if form_of(&path.to_string_lossy()).is_some() {
        Tree::Single(path.to_owned())
    } else {
        Tree::zip(path)?
    };
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for file in tree.files()? {
        if is_junk(&file.path) {
            continue;
        }
        let form = form_of(&file.path);
        if form == Some(Form::Pdf) {
            files.push(file.clone());
        }
        match form {
            Some(_) if file.size > MAX_NOTE && form != Some(Form::Pdf) => {
                skipped.push(Skipped::new(file.path, "too large for a note"));
            }
            Some(form) => notes.push((file, form)),
            None => files.push(file),
        }
    }
    let targets = notes
        .iter()
        .filter(|(_, form)| *form == Form::Html)
        .filter_map(|(f, _)| normalize(&f.path).map(|path| (path, f.path.clone())))
        .collect();
    Ok(AppleNotesFilesSource {
        tree,
        notes,
        files,
        skipped,
        targets,
    })
}

impl AppleNotesFilesSource {
    /// The note a file makes, or `None` with a reason recorded.
    fn convert(&mut self, file: &TreeFile, form: Form) -> Option<SourceNote> {
        let file_title = stem(file_name(&file.path)).trim().to_owned();
        let (title, body) = match form {
            Form::Pdf => {
                // `<…>` keeps spaces in the path; `path/` is not %-decoded.
                let link = format!(
                    "[{}](<kanso-import:path/{}>)",
                    file_name(&file.path).replace(['[', ']'], ""),
                    file.path.replace(['<', '>'], "")
                );
                (file_title, link)
            }
            Form::Html => {
                let html = self.read_text(file)?;
                let rewritten = rewrite_doc_links(&html, &file.path, &self.targets);
                let cleaned = remove_blocks(&rewritten, &["script", "style", "noscript"]);
                let Some(markdown) = notes_html::html_to_markdown(&cleaned).or_else(|| {
                    // Nothing to read, but a picture: a drawing.
                    let lower = cleaned.to_ascii_lowercase();
                    (lower.contains("<img") || lower.contains("<object"))
                        .then(|| DRAWING.to_owned())
                }) else {
                    self.skip(file, "nothing in it Kanso could read");
                    return None;
                };
                let title = html_title(&html).unwrap_or(file_title);
                let body = notes_html::say_picture_only(&notes_html::without_title(
                    &markdown,
                    Some(&title),
                ));
                (title, body)
            }
            Form::Rtf => {
                let bytes = self.read(file)?;
                let body = rtf::to_markdown(&bytes);
                let body = notes_html::without_title(&body, Some(&file_title));
                (file_title, body)
            }
            Form::Text | Form::Markdown => {
                let text = self.read_text(file)?;
                let body = if form == Form::Text {
                    notes_html::tasks_from_glyphs(&notes_html::tidy(&text))
                } else {
                    text.trim().to_owned()
                };
                let body = notes_html::without_title(&body, Some(&file_title));
                (file_title, body)
            }
        };
        Some(SourceNote {
            key: file.path.clone(),
            // A PDF's own path is its attachment's, not a note's.
            path: (form != Form::Pdf).then(|| file.path.clone()),
            folder: parent_of(&file.path)
                .split('/')
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect(),
            title,
            body: body.trim().to_owned(),
            created: file.created.or(file.modified),
            updated: file.modified,
            ..SourceNote::default()
        })
    }

    fn read(&mut self, file: &TreeFile) -> Option<Vec<u8>> {
        match self.tree.read(&file.path, MAX_NOTE) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                self.skip(file, &format!("cannot read it: {err}"));
                None
            }
        }
    }

    fn read_text(&mut self, file: &TreeFile) -> Option<String> {
        match self.tree.read_text(&file.path, MAX_NOTE) {
            Ok(text) => Some(text),
            Err(err) => {
                self.skip(file, &format!("cannot read it: {err}"));
                None
            }
        }
    }

    fn skip(&mut self, file: &TreeFile, reason: &str) {
        if !self.skipped.iter().any(|s| s.path == file.path) {
            self.skipped.push(Skipped::new(file.path.clone(), reason));
        }
    }
}

impl Source for AppleNotesFilesSource {
    fn name(&self) -> String {
        NAME.to_owned()
    }
    fn files(&self) -> &[TreeFile] {
        &self.files
    }
    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for (file, form) in self.notes.clone() {
            if let Some(note) = self.convert(&file, form) {
                each(note)?;
            }
        }
        Ok(())
    }
    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        self.tree.read(path, limit)
    }
    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn forms_by_extension() {
        assert_eq!(form_of("a/Note.HTML"), Some(Form::Html));
        assert_eq!(form_of("a.rtf"), Some(Form::Rtf));
        assert_eq!(form_of("a.txt"), Some(Form::Text));
        assert_eq!(form_of("a.md"), Some(Form::Markdown));
        assert_eq!(form_of("a.pdf"), Some(Form::Pdf));
        assert_eq!(form_of("a.png"), None);
        assert!(is_junk("x/.DS_Store") && is_junk("__MACOSX/a.txt"));
        assert!(!is_junk("Work/Plan.txt"));
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, rng_seed: proptest::test_runner::RngSeed::Fixed(0x4170_706c), ..ProptestConfig::default() })]

        /// A note file's folders are exactly its path's parents, and it is
        /// found the same on every read.
        #[test]
        fn prop_folders_follow_the_path(
            dirs in proptest::collection::vec("[A-Za-z0-9]{1,6}", 0..4),
            name in "[A-Za-z0-9]{1,8}",
        ) {
            let root = tempfile::tempdir().unwrap();
            let mut dir = root.path().to_path_buf();
            for d in &dirs {
                dir.push(d);
            }
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{name}.txt")), "body").unwrap();
            let mut source = open(root.path()).unwrap();
            let mut seen = Vec::new();
            source.notes(true, &mut |n| { seen.push(n); Ok(()) }).unwrap();
            prop_assert_eq!(seen.len(), 1);
            prop_assert_eq!(&seen[0].folder, &dirs);
            prop_assert_eq!(&seen[0].title, &name);
        }
    }
}
