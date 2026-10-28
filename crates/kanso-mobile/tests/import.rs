//! Importing and opening files through the facade: plans write nothing,
//! imports report what the plan said, importing again (even from a copy
//! under another folder, as the app's staging makes) adds nothing, and
//! opening a document again updates its note.

mod common;

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use common::{Phone, lone_phone};
use kanso_import::library::SourceKind;
use kanso_mobile::import::{import_kinds_for, import_sources};
use kanso_mobile::{ImportEvent, ImportSummary, OpenedFile};
use proptest::prelude::*;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kanso-import/tests/fixtures")
        .join(name)
}

fn notes_in(phone: &Phone) -> u32 {
    phone.library.overview().unwrap().all
}

/// Every event of a plan or an import, until the thread is done.
fn events(phone: &Phone, plan: bool, kind: &str, path: &Path) -> Vec<ImportEvent> {
    let (tx, rx) = mpsc::channel();
    let sink = move |event| tx.send(event).is_ok();
    let path = path.to_string_lossy();
    if plan {
        phone.library.import_plan(kind, &path, sink);
    } else {
        phone.library.import(kind, &path, sink);
    }
    let mut all = Vec::new();
    loop {
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(event) => all.push(event),
            Err(mpsc::RecvTimeoutError::Disconnected) => return all,
            Err(mpsc::RecvTimeoutError::Timeout) => panic!("the import never finished: {all:?}"),
        }
    }
}

/// The events must be progress, then one `Done`.
fn done(events: Vec<ImportEvent>) -> ImportSummary {
    let (last, before) = events.split_last().expect("some events");
    assert!(
        before.iter().all(|e| matches!(e, ImportEvent::Progress(_))),
        "{events:?}"
    );
    match last {
        ImportEvent::Done(summary) => summary.clone(),
        other => panic!("expected Done, got {other:?}"),
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn copy_tree(from: &Path, to: &Path) {
    if from.is_dir() {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        }
    } else {
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(from, to).unwrap();
    }
}

/// Tiny exports built on the spot, for the sources the fixtures miss.
fn built(dir: &Path) -> Vec<(&'static str, PathBuf)> {
    write(
        &dir.join("pages/Page.html"),
        "<html><head><title>Page</title></head><body><h1>Page</h1><p>Hello</p></body></html>",
    );
    write(
        &dir.join("roam.json"),
        r#"[{"title":"Roam page","uid":"page-uid","create-time":1600000000000,"edit-time":1600000000000,"children":[{"string":"hello","uid":"abc"}]}]"#,
    );
    write(
        &dir.join("keep.json"),
        r##"{"color":"DEFAULT","isTrashed":false,"isPinned":false,"isArchived":false,"textContent":"buy milk","title":"Shopping","userEditedTimestampUsec":1600000000000000,"createdTimestampUsec":1600000000000000}"##,
    );
    write(
        &dir.join("logseq/pages/Alpha.md"),
        "- first block\n- second block\n",
    );
    write(&dir.join("upnote/Beta.md"), "# Beta\n\nbody\n");
    write(
        &dir.join("bundle.textbundle/text.markdown"),
        "# Bundled\n\ntext\n",
    );
    write(
        &dir.join("bundle.textbundle/info.json"),
        r#"{"version":2,"type":"net.daringfireball.markdown"}"#,
    );
    vec![
        ("html", dir.join("pages")),
        ("roam", dir.join("roam.json")),
        ("google-keep", dir.join("keep.json")),
        ("logseq", dir.join("logseq")),
        ("upnote", dir.join("upnote")),
        ("textbundle", dir.join("bundle.textbundle")),
    ]
}

fn sources(dir: &Path) -> Vec<(&'static str, PathBuf)> {
    let mut all = vec![
        ("markdown", fixture("markdown")),
        ("obsidian", fixture("obsidian")),
        ("evernote", fixture("evernote/Travel.enex")),
        ("joplin", fixture("joplin")),
        ("bear", fixture("bear")),
        ("standard-notes", fixture("standard-notes/backup.txt")),
        ("simplenote", fixture("simplenote/notes.json")),
    ];
    all.extend(built(dir));
    all
}

// no-prop-test: runs the fixtures end to end; the generated folders have
// their property test below.
#[test]
fn a_plan_writes_nothing_and_the_import_matches_it() {
    let scratch = tempfile::tempdir().unwrap();
    for (kind, path) in sources(scratch.path()) {
        let phone = lone_phone();
        let before = notes_in(&phone);
        let plan = done(events(&phone, true, kind, &path));
        assert!(plan.dry_run, "{kind}");
        assert!(!plan.empty && plan.found.notes > 0, "{kind}: {plan:?}");
        assert_eq!(plan.kind, kind);
        assert_eq!(notes_in(&phone), before, "{kind}: a plan wrote");
        assert!(plan.first_note.is_none() || plan.dry_run);

        let all = events(&phone, false, kind, &path);
        assert!(
            all.len() >= 2 && matches!(all[0], ImportEvent::Progress(_)),
            "{kind}: {all:?}"
        );
        let report = done(all);
        assert!(!report.dry_run);
        assert_eq!(report.found, plan.found, "{kind}");
        assert!(report.rejected.is_empty(), "{kind}: {:?}", report.rejected);
        let now = notes_in(&phone);
        assert!(now > before, "{kind}");
        assert!(u64::from(now - before) <= report.found.notes, "{kind}");
        assert!(report.lines[0].starts_with(&format!("{} note", report.found.notes)));
        assert!(
            report.lines.iter().any(|l| l.contains("were added")),
            "{kind}: {:?}",
            report.lines
        );
    }
}

// no-prop-test: the fixtures copied under another parent folder.
#[test]
fn importing_a_copy_under_another_folder_adds_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    for (kind, path) in sources(scratch.path()) {
        let phone = lone_phone();
        let first = done(events(&phone, false, kind, &path));
        let count = notes_in(&phone);
        let name = path.file_name().unwrap();
        let copy = scratch.path().join("elsewhere").join(kind).join(name);
        copy_tree(&path, &copy);
        let second = done(events(&phone, false, kind, &copy));
        assert_eq!(notes_in(&phone), count, "{kind}: a copy made duplicates");
        assert_eq!(second.found, first.found, "{kind}");
        assert!(
            second.lines.iter().any(|l| l.starts_with("0 new,")),
            "{kind}: {:?}",
            second.lines
        );
        assert!(
            second.lines.iter().any(|l| l.contains("unchanged")),
            "{kind}: {:?}",
            second.lines
        );
    }
}

#[derive(Clone, Debug)]
struct Page {
    folder: Option<u8>,
    title: String,
    body: String,
    tags: Vec<String>,
}

fn pages() -> impl Strategy<Value = Vec<Page>> {
    proptest::collection::vec(
        (
            proptest::option::of(0u8..3),
            "[A-Za-z]{3,10}",
            "[A-Za-z ]{0,40}",
            proptest::collection::vec("[a-z]{2,6}", 0..3),
        )
            .prop_map(|(folder, title, body, tags)| Page {
                folder,
                title,
                body,
                tags,
            }),
        1..8,
    )
}

fn write_pages(root: &Path, pages: &[Page]) {
    for (ix, page) in pages.iter().enumerate() {
        let folder = page.folder.map(|f| format!("sub{f}/")).unwrap_or_default();
        let front = if page.tags.is_empty() {
            String::new()
        } else {
            format!("---\ntags: [{}]\n---\n", page.tags.join(", "))
        };
        write(
            &root.join(format!("{folder}{ix}-{}.md", page.title)),
            &format!("{front}# {}\n\n{}\n", page.title, page.body),
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// A Markdown folder imported from two different parent folders is one
    /// note per file, and the second import creates nothing.
    #[test]
    fn prop_a_markdown_folder_imported_twice_makes_no_duplicates(pages in pages()) {
        let scratch = tempfile::tempdir().unwrap();
        let first = scratch.path().join("one/Notes");
        let second = scratch.path().join("two/Notes");
        write_pages(&first, &pages);
        copy_tree(&first, &second);
        let phone = lone_phone();
        let report = done(events(&phone, false, "markdown", &first));
        prop_assert_eq!(report.found.notes, pages.len() as u64);
        prop_assert_eq!(notes_in(&phone) as usize, pages.len());
        let again = done(events(&phone, false, "markdown", &second));
        prop_assert_eq!(notes_in(&phone) as usize, pages.len());
        prop_assert!(again.lines.iter().any(|l| l.starts_with("0 new,")), "{:?}", again.lines);
    }
}

#[test]
fn an_unknown_kind_fails_once() {
    let phone = lone_phone();
    let dir = tempfile::tempdir().unwrap();
    for kind in ["nonsense", "apple-notes"] {
        let all = events(&phone, false, kind, dir.path());
        assert_eq!(all.len(), 1, "{all:?}");
        assert!(matches!(&all[0], ImportEvent::Failed(m) if m.contains("unknown kind")));
    }
    let all = events(&phone, true, "markdown", &dir.path().join("missing"));
    assert_eq!(all.len(), 1);
    assert!(matches!(&all[0], ImportEvent::Failed(m) if m.contains("doesn't exist")));
}

#[test]
fn a_sink_that_hangs_up_ends_the_thread_quietly() {
    let phone = lone_phone();
    let (tx, rx) = mpsc::channel();
    phone.library.import(
        "markdown",
        &fixture("markdown").to_string_lossy(),
        move |event| {
            tx.send(event).ok();
            false
        },
    );
    let mut seen = 0;
    while rx.recv_timeout(Duration::from_secs(60)).is_ok() {
        seen += 1;
    }
    assert_eq!(seen, 1, "nothing is sent after the sink said no");
    // The library still works.
    assert!(phone.library.overview().is_ok());
}

#[test]
fn an_import_is_announced_on_the_change_stream() {
    let phone = lone_phone();
    let (tx, rx) = mpsc::channel();
    phone.library.changes(move |ids| tx.send(ids).is_ok());
    let summary = done(events(&phone, false, "markdown", &fixture("markdown")));
    let first = summary.first_note.expect("a first note");
    let mut seen = Vec::new();
    while !seen.contains(&first) {
        seen.extend(rx.recv_timeout(Duration::from_secs(10)).expect("changes"));
    }
}

#[test]
fn the_sources_are_the_phone_ones_in_order() {
    let sources = import_sources();
    assert_eq!(sources.len(), 16);
    let expected: Vec<&str> = SourceKind::ALL
        .iter()
        .filter(|k| **k != SourceKind::AppleNotes)
        .map(|k| k.as_str())
        .collect();
    let kinds: Vec<&str> = sources.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(kinds, expected);
    assert!(sources.iter().all(|s| s.picks_files || s.picks_folders));
    assert!(
        sources
            .iter()
            .all(|s| !s.label.is_empty() && !s.hint.is_empty())
    );
    let markdown = &sources[0];
    assert!(markdown.picks_folders && !markdown.picks_files);
    assert_eq!(import_kinds_for("/x/Work.enex"), ["evernote"]);
}

fn opened(path: &Path, name: &str, identity: Option<&str>) -> OpenedFile {
    OpenedFile {
        path: path.to_string_lossy().into_owned(),
        name: name.to_owned(),
        identity: identity.map(str::to_owned),
    }
}

#[test]
fn a_document_opened_again_updates_its_note() {
    let phone = lone_phone();
    let scratch = tempfile::tempdir().unwrap();
    let staging = scratch.path().join("staging");
    let identity = "content://provider/doc/42";
    let (one, two) = (
        scratch.path().join("tmp1/x.md"),
        scratch.path().join("tmp2/x.md"),
    );
    write(&one, "# Title\n\nfirst");
    write(&two, "# Title\n\nsecond");
    let first = phone
        .library
        .open_documents(
            vec![opened(&one, "x.md", Some(identity))],
            staging.to_str().unwrap(),
        )
        .unwrap();
    assert!(first.problems.is_empty(), "{first:?}");
    let id = first.note_id.expect("a note");
    let second = phone
        .library
        .open_documents(
            vec![opened(&two, "x.md", Some(identity))],
            staging.to_str().unwrap(),
        )
        .unwrap();
    assert_eq!(second.note_id.as_deref(), Some(id.as_str()));
    let body = phone.library.note(&id).unwrap().body.unwrap();
    assert!(body.contains("second") && !body.contains("first"), "{body}");
    assert_eq!(notes_in(&phone), 1);
    // The staged copies are gone.
    let left: Vec<_> = std::fs::read_dir(&staging).unwrap().collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn a_file_without_identity_is_read_in_place() {
    let phone = lone_phone();
    let scratch = tempfile::tempdir().unwrap();
    let file = scratch.path().join("README.md");
    write(&file, "# R\n\none");
    let staging = scratch.path().join("staging");
    let open = |file: &Path| {
        phone
            .library
            .open_documents(
                vec![opened(file, "README.md", None)],
                staging.to_str().unwrap(),
            )
            .unwrap()
    };
    let id = open(&file).note_id.unwrap();
    assert!(file.exists());
    write(&file, "# R\n\ntwo");
    assert_eq!(open(&file).note_id.as_deref(), Some(id.as_str()));
    assert!(
        phone
            .library
            .note(&id)
            .unwrap()
            .body
            .unwrap()
            .contains("two")
    );
    assert!(!staging.exists());
}

#[test]
fn other_files_are_problems_and_nothing_is_nothing() {
    let phone = lone_phone();
    let scratch = tempfile::tempdir().unwrap();
    let staging = scratch.path().join("staging");
    let pdf = scratch.path().join("a.pdf");
    let docx = scratch.path().join("b.docx");
    write(&pdf, "x");
    write(&docx, "x");
    let got = phone
        .library
        .open_documents(
            vec![
                opened(&pdf, "a.pdf", Some("content://a")),
                opened(&docx, "b.docx", None),
                opened(
                    &scratch.path().join("gone.md"),
                    "gone.md",
                    Some("content://gone"),
                ),
            ],
            staging.to_str().unwrap(),
        )
        .unwrap();
    assert_eq!(got.note_id, None);
    assert_eq!(got.problems.len(), 3, "{:?}", got.problems);
    assert!(
        got.problems.contains(
            &"a.pdf isn't a Markdown or text file. To bring in another app's export, use Import…"
                .to_owned()
        )
    );
    assert!(got.problems[0].starts_with("gone.md: couldn't read it"));
    let nothing = phone
        .library
        .open_documents(Vec::new(), staging.to_str().unwrap())
        .unwrap();
    assert_eq!((nothing.note_id, nothing.problems), (None, Vec::new()));
}

#[test]
fn opening_a_document_is_announced_on_the_change_stream() {
    let phone = lone_phone();
    let (tx, rx) = mpsc::channel();
    phone.library.changes(move |ids| tx.send(ids).is_ok());
    let scratch = tempfile::tempdir().unwrap();
    let file = scratch.path().join("n.md");
    write(&file, "# N\n\nx");
    let id = phone
        .library
        .open_documents(vec![opened(&file, "n.md", None)], "unused")
        .unwrap()
        .note_id
        .unwrap();
    let mut seen = Vec::new();
    while !seen.contains(&id) {
        seen.extend(rx.recv_timeout(Duration::from_secs(10)).expect("changes"));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Whatever the identity, opening the document from two different
    /// temporary paths is one note with the latest text, and nothing stays
    /// in the staging folder. A different identity is a different note.
    #[test]
    fn prop_an_identity_is_one_note(identity in "[ -~]{1,60}", other in "[ -~]{1,60}") {
        prop_assume!(identity != other);
        let phone = lone_phone();
        let scratch = tempfile::tempdir().unwrap();
        let staging = scratch.path().join("staging");
        let open = |tmp: &str, text: &str, identity: &str| {
            let file = scratch.path().join(tmp).join("doc.md");
            write(&file, text);
            phone
                .library
                .open_documents(vec![opened(&file, "doc.md", Some(identity))], staging.to_str().unwrap())
                .unwrap()
        };
        let first = open("a", "# D\n\none", &identity).note_id.unwrap();
        let second = open("b", "# D\n\ntwo", &identity).note_id.unwrap();
        prop_assert_eq!(&first, &second);
        prop_assert!(phone.library.note(&first).unwrap().body.unwrap().contains("two"));
        let elsewhere = open("c", "# D\n\nthree", &other).note_id.unwrap();
        prop_assert_ne!(&first, &elsewhere);
        prop_assert_eq!(std::fs::read_dir(&staging).unwrap().count(), 0);
    }
}
