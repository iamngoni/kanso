//! Example tests for Notion's and Bear's exports, built as zips from the
//! fixtures.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use kanso_core::DocType;
use kanso_import::library::{LibraryReport, SourceKind, import_source, stable_id};
use kanso_store::{Doc, SearchParams, Store};

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn fixture(name: &str) -> PathBuf {
    manifest_dir().join("tests/fixtures").join(name)
}

/// Every file under `dir`, as `(path relative to dir, bytes)`, sorted.
fn files_of(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy();
                out.push((rel.replace('\\', "/"), std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// Notion's IDs make paths too long for a checkout on Windows, so the
/// export is written here rather than checked in.
const A: &str = "0f3c5e2a9b8d4c1e8f7a6b5c4d3e2f1a";
const B: &str = "1a2b3c4d5e6f708192a3b4c5d6e7f809";
const C: &str = "22222222222222222222222222222222";
const D: &str = "33333333333333333333333333333333";

/// A Notion export: a page with a sub-page, a picture and a database with
/// a row page, as `(path, bytes)`.
fn notion_files() -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = vec![
        (
            format!("Work {A}.md"),
            format!("# Work\n\nThe [plan](Work%20{A}/Plan%20{B}.md), the [tasks](Work%20{A}/Tasks%20{C}.csv) and a [web link](https://www.notion.so/Plan-{B}?pvs=4).\n").into_bytes(),
        ),
        (
            format!("Work {A}/Plan {B}.md"),
            format!("# Plan\n\n![Chart](Plan%20{B}/chart.png)\n\nBack to [Work](../Work%20{A}.md).\n").into_bytes(),
        ),
        (
            format!("Work {A}/Plan {B}/chart.png"),
            b"\x89PNG\r\n\x1a\nchart".to_vec(),
        ),
        (
            format!("Work {A}/Tasks {C}.csv"),
            b"Name,Status,Tags\n\"Write, then ship\",Done,urgent\nReview,Open,\n".to_vec(),
        ),
        (
            format!("Work {A}/Tasks {C}/Review {D}.md"),
            b"# Review\n\nStatus: Open\nTags: weekly, team\n\nLook it over.\n".to_vec(),
        ),
        (
            format!("Work {A}/Tasks {C}_all.csv"),
            b"\xef\xbb\xbfName,Status,Tags\n\"Write, then ship\",Done,urgent\nReview,Open,\nArchived one,Done,\n".to_vec(),
        ),
    ];
    files.sort();
    files
}

/// The Notion export written out as a folder.
fn notion_folder(dir: &Path) -> PathBuf {
    let root = dir.join("notion");
    for (path, data) in notion_files() {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, data).unwrap();
    }
    root
}

fn write_zip(to: &Path, files: &[(String, Vec<u8>)]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(to).unwrap());
    for (name, data) in files {
        zip.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

struct Env {
    dir: tempfile::TempDir,
    store: Store,
}

impl Env {
    fn new() -> Env {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        Env { dir, store }
    }

    fn import(&self, kind: SourceKind, path: &Path) -> LibraryReport {
        import_source(kind, path, &self.store, |_| {}).unwrap()
    }

    fn note(&self, title: &str) -> Doc {
        self.store
            .search(&SearchParams::default())
            .unwrap()
            .into_iter()
            .find(|n| n.str_field("title") == Some(title))
            .unwrap_or_else(|| panic!("no note titled {title}"))
    }

    fn book_name(&self, note: &Doc) -> String {
        let book = self
            .store
            .get(note.str_field("bookId").unwrap())
            .unwrap()
            .unwrap();
        book.str_field("name").unwrap().to_owned()
    }
}

fn suffix(id: &str) -> &str {
    id.split_once(':').unwrap().1
}

fn check_notion(env: &Env, report: &LibraryReport) {
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 4, "{report}");
    assert_eq!(report.unresolved_links, 0, "{report}");
    // Notion, Work and Tasks.
    assert_eq!(report.found.notebooks, 3, "{report}");
    let work = env.note("Work");
    let plan = env.note("Plan");
    let tasks = env.note("Tasks");
    let review = env.note("Review");
    assert_eq!(env.book_name(&work), "Notion");
    assert_eq!(env.book_name(&plan), "Work");
    assert_eq!(env.book_name(&review), "Tasks");
    let body = work.str_field("body").unwrap();
    assert!(
        body.contains(&format!("[plan](kanso://note/{})", suffix(&plan.id))),
        "{body}"
    );
    assert!(
        body.contains(&format!("[tasks](kanso://note/{})", suffix(&tasks.id))),
        "{body}"
    );
    assert!(
        body.contains(&format!("[web link](kanso://note/{})", suffix(&plan.id))),
        "{body}"
    );
    let plan_body = plan.str_field("body").unwrap();
    assert!(plan_body.contains("![Chart](kanso://file:"), "{plan_body}");
    assert!(
        plan_body.contains(&format!("[Work](kanso://note/{})", suffix(&work.id))),
        "{plan_body}"
    );
    let table = tasks.str_field("body").unwrap();
    assert!(
        table.starts_with("| Name | Status | Tags |\n| --- | --- | --- |\n"),
        "{table}"
    );
    assert!(table.contains("| [[Review]] | Open |"), "{table}");
    assert!(
        table.contains("| Archived one | Done |"),
        "the _all view: {table}"
    );
    let tags: Vec<String> = review.fields["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| {
            env.store
                .get(id.as_str().unwrap())
                .unwrap()
                .unwrap()
                .str_field("name")
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(tags, ["weekly", "team"]);
}

#[test]
fn a_notion_export_folder() {
    let env = Env::new();
    let folder = notion_folder(env.dir.path());
    let report = env.import(SourceKind::Notion, &folder);
    check_notion(&env, &report);
    let again = env.import(SourceKind::Notion, &folder);
    assert_eq!(again.notes.unchanged, 4, "{again}");
    assert_eq!(again.attachments.unchanged, 1, "{again}");
}

#[test]
fn a_notion_export_zip() {
    let env = Env::new();
    let zip = env.dir.path().join("Export-1234.zip");
    write_zip(&zip, &notion_files());
    let report = env.import(SourceKind::Notion, &zip);
    check_notion(&env, &report);
}

#[test]
fn a_notion_export_split_into_parts_inside_a_folder() {
    let env = Env::new();
    let files: Vec<(String, Vec<u8>)> = notion_files()
        .into_iter()
        .map(|(name, data)| (format!("Export-1234/{name}"), data))
        .collect();
    let (first, second) = files.split_at(files.len() / 2);
    let part1 = env.dir.path().join("part1.zip");
    let part2 = env.dir.path().join("part2.zip");
    write_zip(&part1, first);
    write_zip(&part2, second);
    let outer = env.dir.path().join("Export-1234.zip");
    write_zip(
        &outer,
        &[
            (
                "Export-1234-Part-1.zip".to_owned(),
                std::fs::read(&part1).unwrap(),
            ),
            (
                "Export-1234-Part-2.zip".to_owned(),
                std::fs::read(&part2).unwrap(),
            ),
        ],
    );
    let report = env.import(SourceKind::Notion, &outer);
    check_notion(&env, &report);
}

#[test]
fn a_bear_backup() {
    let env = Env::new();
    let backup = env.dir.path().join("Backup.bear2bk");
    write_zip(&backup, &files_of(&fixture("bear")));
    let report = env.import(SourceKind::Bear, &backup);
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 3, "{report}");
    assert_eq!(report.trashed, 1);
    assert_eq!(report.attachments.created, 1);
    let plan = env.note("Plan");
    assert_eq!(env.book_name(&plan), "Bear");
    assert_eq!(plan.fields["pinned"], true);
    assert_eq!(plan.fields["createdAt"], 1_709_285_400_000i64);
    let body = plan.str_field("body").unwrap();
    assert!(
        body.contains("![pic](kanso://file:") || body.contains("![](kanso://file:"),
        "{body}"
    );
    assert!(body.contains("[[Trip]]"), "{body}");
    assert_eq!(plan.fields["tags"].as_array().unwrap().len(), 2);
    let trip = stable_id(DocType::Note, SourceKind::Bear, "Bear", "BEAR-TRIP");
    let trip = env.store.get(&trip).unwrap().unwrap();
    assert_eq!(trip.str_field("title"), Some("Trip"));
    assert_eq!(trip.str_field("bookId"), Some("trash"));
    let loose = env.note("Loose note");
    assert_eq!(env.book_name(&loose), "Bear");

    let again = env.import(SourceKind::Bear, &backup);
    assert_eq!(again.notes.unchanged, 3, "{again}");
}
