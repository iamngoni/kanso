//! Which import sources a file or folder another app shared could be: by its
//! extension and, where that does not settle it, a cheap look inside (a
//! zip's entry names, the start of a JSON file, a folder's first few
//! levels). A guess to offer, best first; the person picks.

use std::fs::File;
use std::io::Read as _;
use std::path::Path;

use super::SourceKind;
use super::tree::extension;

/// How many entries of a zip or folder are looked at.
const LOOK_AT: usize = 2000;
/// How deep a folder is looked into.
const DEPTH: usize = 3;
/// How much of a JSON or text file is read.
const PEEK: u64 = 64 * 1024;

/// The import sources `path`, a file or a folder, could be, best first.
/// Never [`SourceKind::AppleNotes`], which reads a Mac's own database. Every
/// kind returned for a file lists the file's extension in
/// [`SourceKind::file_extensions`]; every kind returned for a folder picks
/// folders. A path that doesn't exist is judged by its name alone.
pub fn kinds_for(path: &Path) -> Vec<SourceKind> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = extension(&name);
    let mut kinds = if path.is_dir() {
        for_folder(path, &ext)
    } else {
        for_file(path, &ext)
    };
    kinds.retain(|kind| *kind != SourceKind::AppleNotes);
    let mut seen = Vec::new();
    kinds.retain(|kind| {
        let fresh = !seen.contains(kind);
        seen.push(*kind);
        fresh
    });
    kinds
}

fn for_file(path: &Path, ext: &str) -> Vec<SourceKind> {
    use SourceKind::*;
    match ext {
        "enex" => vec![Evernote],
        "jex" => vec![Joplin],
        "bear2bk" => vec![Bear],
        "textbundle" | "textpack" => vec![TextBundle],
        "note" => vec![Tomboy],
        "html" | "htm" => vec![Html, AppleJournal],
        "md" | "markdown" => vec![Markdown],
        "txt" => {
            if peek(path).trim_start().starts_with('{') {
                vec![StandardNotes, Markdown]
            } else {
                vec![Markdown]
            }
        }
        "json" => json_kinds(&peek(path)),
        "zip" => zip_kinds(path),
        _ => Vec::new(),
    }
}

/// The start of a text file, lossily; empty when it can't be read.
fn peek(path: &Path) -> String {
    let mut bytes = Vec::new();
    if let Ok(file) = File::open(path) {
        file.take(PEEK).read_to_end(&mut bytes).ok();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// JSON exports, told apart by the field names near the start.
fn json_kinds(start: &str) -> Vec<SourceKind> {
    use SourceKind::*;
    let mut kinds = Vec::new();
    if start.contains("\"activeNotes\"") || start.contains("\"trashedNotes\"") {
        kinds.push(Simplenote);
    }
    if start.contains("\"content_type\"") || start.contains("\"items\"") {
        kinds.push(StandardNotes);
    }
    if start.contains("\"textContent\"")
        || start.contains("\"listContent\"")
        || start.contains("\"isTrashed\"")
    {
        kinds.push(GoogleKeep);
    }
    if start.trim_start().starts_with('[')
        && (start.contains("\"children\"") || start.contains("\"edit-time\""))
    {
        kinds.push(Roam);
    }
    kinds.extend([GoogleKeep, Roam, Simplenote, StandardNotes]);
    kinds
}

fn zip_kinds(path: &Path) -> Vec<SourceKind> {
    use SourceKind::*;
    let mut names: Vec<String> = Vec::new();
    if let Ok(file) = File::open(path)
        && let Ok(archive) = zip::ZipArchive::new(file)
    {
        names = archive
            .file_names()
            .take(LOOK_AT)
            .map(|n| n.to_ascii_lowercase())
            .collect();
    }
    let has = |needle: &str| names.iter().any(|n| n.contains(needle));
    let ends = |suffix: &str| names.iter().any(|n| n.ends_with(suffix));
    let mut kinds = Vec::new();
    if has("takeout/keep") || has("keep/") {
        kinds.push(GoogleKeep);
    }
    if ends("notes.json") || has("source/notes.json") {
        kinds.push(Simplenote);
    }
    if has("standard notes backup") || (ends(".txt") && ends(".json") && !ends(".md")) {
        kinds.push(StandardNotes);
    }
    if has(".textbundle/") || ends("text.markdown") {
        kinds.push(TextBundle);
        kinds.push(Bear);
    }
    if ends(".md") && (ends(".csv") || names.iter().any(|n| has_notion_id(n))) {
        kinds.push(Notion);
    }
    if ends(".html") || ends(".htm") {
        kinds.push(Html);
        kinds.push(AppleJournal);
    }
    kinds.extend([
        Notion,
        Simplenote,
        StandardNotes,
        GoogleKeep,
        Html,
        TextBundle,
        Bear,
        AppleJournal,
    ]);
    kinds
}

/// Whether a path has a name ending in a space and 32 hex characters, as
/// Notion's exports do.
fn has_notion_id(name: &str) -> bool {
    name.split('/').any(|part| {
        let stem = part.rsplit_once('.').map_or(part, |(s, _)| s);
        stem.len() > 33
            && stem.is_char_boundary(stem.len() - 33)
            && stem[stem.len() - 33..].starts_with(' ')
            && stem[stem.len() - 32..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
    })
}

/// What a folder's first levels hold.
#[derive(Default)]
struct Look {
    obsidian: bool,
    logseq_pages: bool,
    logseq_journals: bool,
    enex: bool,
    textbundle: bool,
    jex: bool,
    tomboy: bool,
    html: bool,
    markdown: bool,
    keep_json: bool,
    notion_ids: bool,
    joplin_raw: bool,
}

fn for_folder(path: &Path, ext: &str) -> Vec<SourceKind> {
    use SourceKind::*;
    if ext == "textbundle" {
        return vec![TextBundle];
    }
    let mut look = Look::default();
    let mut seen = 0;
    let mut stack = vec![(path.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > LOOK_AT {
                break;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower = name.to_ascii_lowercase();
            let is_dir = entry.path().is_dir();
            if is_dir {
                if depth == 0 {
                    look.obsidian |= lower == ".obsidian";
                    look.logseq_pages |= lower == "pages";
                    look.logseq_journals |= lower == "journals";
                }
                if lower.ends_with(".textbundle") {
                    look.textbundle = true;
                } else if depth + 1 < DEPTH {
                    stack.push((entry.path(), depth + 1));
                }
                continue;
            }
            match extension(&lower).as_str() {
                "enex" => look.enex = true,
                "jex" => look.jex = true,
                "note" => look.tomboy = true,
                "html" | "htm" => look.html = true,
                "json" => look.keep_json = true,
                "md" | "markdown" | "txt" => {
                    look.markdown = true;
                    look.notion_ids |= has_notion_id(&lower);
                    if !look.joplin_raw && lower.ends_with(".md") {
                        look.joplin_raw = peek(&entry.path()).contains("\ntype_: ");
                    }
                }
                _ => {}
            }
        }
    }
    let mut kinds = Vec::new();
    if look.obsidian {
        kinds.push(Obsidian);
    }
    if look.logseq_pages || look.logseq_journals {
        kinds.push(Logseq);
    }
    if look.enex {
        kinds.push(Evernote);
    }
    if look.textbundle {
        kinds.push(TextBundle);
        kinds.push(Bear);
    }
    if look.joplin_raw || look.jex {
        kinds.push(Joplin);
    }
    if look.notion_ids {
        kinds.push(Notion);
    }
    if look.tomboy {
        kinds.push(Tomboy);
    }
    if look.keep_json {
        kinds.push(GoogleKeep);
    }
    if look.html {
        kinds.push(Html);
        kinds.push(AppleJournal);
    }
    if look.markdown || kinds.is_empty() {
        kinds.push(Markdown);
        kinds.push(Obsidian);
        kinds.push(Logseq);
        kinds.push(UpNote);
    }
    kinds
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use SourceKind::*;

    #[test]
    fn extensions_name_their_source() {
        let kinds = |name: &str| kinds_for(Path::new(name));
        assert_eq!(kinds("/x/Work.enex"), [Evernote]);
        assert_eq!(kinds("/x/All.JEX"), [Joplin]);
        assert_eq!(kinds("/x/b.bear2bk"), [Bear]);
        assert_eq!(kinds("/x/b.textpack"), [TextBundle]);
        assert_eq!(kinds("/x/b.note"), [Tomboy]);
        assert_eq!(kinds("/x/p.html"), [Html, AppleJournal]);
        assert_eq!(kinds("/x/p.md"), [Markdown]);
        assert_eq!(kinds("/x/p.pdf"), Vec::<SourceKind>::new());
        assert_eq!(kinds("/x/NoteStore.sqlite"), Vec::<SourceKind>::new());
    }

    #[test]
    fn json_is_told_apart_by_its_fields() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, text: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, text).unwrap();
            path
        };
        let simple = write("notes.json", r#"{"activeNotes":[],"trashedNotes":[]}"#);
        assert_eq!(kinds_for(&simple)[0], Simplenote);
        let keep = write(
            "k.json",
            r#"{"color":"DEFAULT","isTrashed":false,"textContent":"x"}"#,
        );
        assert_eq!(kinds_for(&keep)[0], GoogleKeep);
        let roam = write("r.json", r#"[{"title":"A","children":[]}]"#);
        assert_eq!(kinds_for(&roam)[0], Roam);
        let sn = write("sn.txt", r#"{"items":[{"content_type":"Note"}]}"#);
        assert_eq!(kinds_for(&sn), [StandardNotes, Markdown]);
        let plain = write("plain.txt", "just a note");
        assert_eq!(kinds_for(&plain), [Markdown]);
    }

    #[test]
    fn a_zip_is_told_by_its_entries() {
        use std::io::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let make = |name: &str, entries: &[&str]| {
            let path = dir.path().join(name);
            let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
            for entry in entries {
                zip.start_file(*entry, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(b"x").unwrap();
            }
            zip.finish().unwrap();
            path
        };
        let keep = make("t.zip", &["Takeout/Keep/a.json"]);
        assert_eq!(kinds_for(&keep)[0], GoogleKeep);
        let simple = make("s.zip", &["source/notes.json"]);
        assert_eq!(kinds_for(&simple)[0], Simplenote);
        let notion = make(
            "n.zip",
            &["Page 0123456789abcdef0123456789abcdef.md", "Db.csv"],
        );
        assert_eq!(kinds_for(&notion)[0], Notion);
        let bundle = make("b.zip", &["a.textbundle/text.markdown"]);
        assert_eq!(kinds_for(&bundle)[0], TextBundle);
        let pages = make("h.zip", &["site/index.html"]);
        assert_eq!(kinds_for(&pages)[0], Html);
        // Ambiguous or unreadable: several, never Apple Notes.
        let unknown = make("u.zip", &["readme"]);
        assert!(kinds_for(&unknown).len() > 3);
    }

    #[test]
    fn a_folder_is_told_by_what_is_in_it() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault");
        std::fs::create_dir_all(vault.join(".obsidian")).unwrap();
        std::fs::write(vault.join("a.md"), "# A").unwrap();
        assert_eq!(kinds_for(&vault)[0], Obsidian);
        assert!(kinds_for(&vault).contains(&Markdown));
        let graph = dir.path().join("graph");
        std::fs::create_dir_all(graph.join("pages")).unwrap();
        std::fs::create_dir_all(graph.join("journals")).unwrap();
        assert_eq!(kinds_for(&graph)[0], Logseq);
        let enex = dir.path().join("enex");
        std::fs::create_dir_all(&enex).unwrap();
        std::fs::write(enex.join("a.enex"), "<en-export/>").unwrap();
        assert_eq!(kinds_for(&enex), [Evernote]);
        let plain = dir.path().join("plain");
        std::fs::create_dir_all(&plain).unwrap();
        std::fs::write(plain.join("a.md"), "# A").unwrap();
        assert_eq!(kinds_for(&plain)[0], Markdown);
        let empty = dir.path().join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(kinds_for(&empty)[0], Markdown);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Whatever the file is called, every kind offered lists its
        /// extension, and the Mac's own database is never offered.
        #[test]
        fn prop_a_file_kind_lists_the_extension(
            stem in "[a-zA-Z0-9 ]{1,10}",
            ext in prop::sample::select(vec![
                "enex", "jex", "bear2bk", "textbundle", "textpack", "note", "html", "htm",
                "json", "zip", "md", "markdown", "txt", "pdf", "sqlite", "ENEX", "Zip",
            ]),
        ) {
            let path = std::path::PathBuf::from(format!("/nowhere/{stem}.{ext}"));
            let lowered = ext.to_ascii_lowercase();
            for kind in kinds_for(&path) {
                prop_assert!(kind != AppleNotes);
                prop_assert!(kind.file_extensions().contains(&lowered.as_str()), "{kind:?} for .{ext}");
            }
        }

        /// Every kind offered for a folder picks folders, whatever is in it.
        #[test]
        fn prop_a_folder_kind_picks_folders(
            files in proptest::collection::vec(
                ("[a-z0-9 ]{1,8}", prop::sample::select(vec!["md", "enex", "html", "json", "note", "jex", "txt", "png"])),
                0..6,
            ),
            obsidian in any::<bool>(),
        ) {
            let dir = tempfile::tempdir().unwrap();
            if obsidian {
                std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
            }
            for (stem, ext) in &files {
                std::fs::write(dir.path().join(format!("{stem}.{ext}")), "{}").unwrap();
            }
            let kinds = kinds_for(dir.path());
            prop_assert!(!kinds.is_empty());
            for kind in &kinds {
                prop_assert!(crate::library::words::picks(*kind).1, "{kind:?}");
                prop_assert!(*kind != AppleNotes);
            }
            let mut unique = kinds.clone();
            unique.dedup();
            prop_assert_eq!(unique.len(), kinds.len());
        }
    }
}
