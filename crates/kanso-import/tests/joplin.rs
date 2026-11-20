//! Joplin's exports, a RAW folder and the same as a `.jex`, imported end to
//! end against `tests/fixtures/joplin`, and property tests of reading items
//! and rewriting their links. CI pins the seed with `PROPTEST_RNG_SEED`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use kanso_core::{DocType, TRASH_BOOK_ID};
use kanso_import::library::joplin::{Item, parse_item, render_item, rewrite_links};
use kanso_import::library::{LibraryReport, SourceKind, dates, import_source, stable_id};
use kanso_store::{Doc, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const PLAN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MEETING: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OLD: &str = "cccccccccccccccccccccccccccccccc";
const SKETCH: &str = "dddddddddddddddddddddddddddddddd";

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn fixture() -> PathBuf {
    manifest_dir().join("tests/fixtures/joplin")
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

    fn import(&self, path: &Path) -> LibraryReport {
        import_source(SourceKind::Joplin, path, &self.store, |_| {}).unwrap()
    }

    fn doc(&self, id: &str) -> Doc {
        self.store
            .get(id)
            .unwrap()
            .unwrap_or_else(|| panic!("no {id}"))
    }

    fn note(&self, source: &str, key: &str) -> Doc {
        self.doc(&stable_id(DocType::Note, SourceKind::Joplin, source, key))
    }

    fn book_name(&self, id: &str) -> String {
        self.doc(id).str_field("name").unwrap().to_owned()
    }

    fn tag_names(&self, note: &Doc) -> Vec<String> {
        let mut names: Vec<String> = note.fields["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| self.book_name(id.as_str().unwrap()))
            .collect();
        names.sort();
        names
    }

    /// The fixture packed as a `.jex` (a plain tar).
    fn jex(&self) -> PathBuf {
        let path = self.dir.path().join("export.jex");
        let file = std::fs::File::create(&path).unwrap();
        let mut tar = tar::Builder::new(file);
        tar.append_dir_all(".", fixture()).unwrap();
        tar.finish().unwrap();
        path
    }
}

fn ms(text: &str) -> i64 {
    dates::parse(text).unwrap()
}

fn suffix(id: &str) -> &str {
    id.split_once(':').unwrap().1
}

/// Checks everything the fixture holds came in, under source `source`.
fn check(env: &Env, source: &str, report: &LibraryReport) {
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 3, "{report}");
    assert_eq!(report.notes.created, 3, "{report}");
    // The export, Work and Work › Projects.
    assert_eq!(report.notebooks.created, 3, "{report}");
    assert_eq!(report.attachments.created, 1, "{report}");
    assert_eq!(report.trashed, 1);
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.path == format!("{}.md", "9".repeat(32)) && s.reason.contains("encrypted")),
        "{:?}",
        report.skipped
    );

    let plan = env.note(source, PLAN);
    let meeting = env.note(source, MEETING);
    let body = plan.str_field("body").unwrap();
    let meeting_id = stable_id(DocType::Note, SourceKind::Joplin, source, MEETING);
    assert!(
        body.starts_with(&format!(
            "See [the meeting](kanso://note/{}) and the sketch:\n\n![sketch.png](kanso://file:",
            suffix(&meeting_id)
        )),
        "{body}"
    );
    let file_id = stable_id(
        DocType::File,
        SourceKind::Joplin,
        source,
        &format!("file:resources/{SKETCH}.png"),
    );
    assert!(
        body.contains(&format!("![sketch.png](kanso://file:{})", suffix(&file_id))),
        "{body}"
    );
    assert!(
        body.contains(&format!(
            "<img src=\"kanso://file:{}\" width=\"200\"/>",
            suffix(&file_id)
        )),
        "{body}"
    );
    assert!(
        body.contains(&format!("[missing one](:/{})", "0".repeat(32))),
        "{body}"
    );
    assert!(
        body.ends_with("stays.\n\nstatus: draft\nowner: me"),
        "{body}"
    );
    assert_eq!(plan.str_field("title"), Some("Plan"));
    assert_eq!(env.tag_names(&plan), ["area/home", "urgent"]);
    // The user's own times win over the sync ones.
    assert_eq!(plan.fields["createdAt"], ms("2024-01-10T08:00:00.000Z"));
    assert_eq!(plan.fields["updatedAt"], ms("2024-01-11T08:00:00.000Z"));
    let projects = env.doc(plan.str_field("bookId").unwrap());
    assert_eq!(projects.str_field("name"), Some("Projects"));
    let work_id = projects.str_field("parentBookId").unwrap();
    assert_eq!(env.book_name(work_id), "Work");
    let top = env.doc(work_id);
    assert_eq!(
        env.book_name(top.str_field("parentBookId").unwrap()),
        source
    );

    let (content_type, attachment) = env.store.attachment(&file_id).unwrap().unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(attachment.data, b"\x89PNG\r\n\x1a\nfixture");

    // The to-do comes in as a note in its notebook.
    assert_eq!(meeting.str_field("body"), Some("- [ ] agenda\n- [x] room"));
    assert_eq!(meeting.str_field("bookId"), Some(work_id));
    assert_eq!(env.tag_names(&meeting), ["urgent"]);
    assert_eq!(meeting.fields["createdAt"], ms("2024-03-01T09:30:00.000Z"));

    let old = env.note(source, OLD);
    assert_eq!(old.str_field("bookId"), Some(TRASH_BOOK_ID));
    assert_eq!(old.str_field("body"), Some("Thrown away."));
}

fn check_again(env: &Env, path: &Path) {
    let again = env.import(path);
    assert_eq!(again.notes.unchanged, 3, "{again}");
    assert_eq!(again.notes.written(), 0, "{again}");
    assert_eq!(again.notebooks.unchanged, 3, "{again}");
    assert_eq!(again.attachments.unchanged, 1, "{again}");
    assert_eq!(again.tags_created, 0, "{again}");
}

#[test]
fn a_raw_export_comes_in_whole() {
    let env = Env::new();
    let report = env.import(&fixture());
    check(&env, "joplin", &report);
    check_again(&env, &fixture());
}

#[test]
fn a_jex_comes_in_the_same() {
    let env = Env::new();
    let jex = env.jex();
    let report = env.import(&jex);
    check(&env, "export", &report);
    check_again(&env, &jex);
}

#[test]
fn a_file_that_is_not_a_jex_is_refused() {
    let env = Env::new();
    let path = env.dir.path().join("broken.jex");
    std::fs::write(&path, b"not a tar at all").unwrap();
    assert!(import_source(SourceKind::Joplin, &path, &env.store, |_| {}).is_err());
}

// ---------------------------------------------------------------------------
// Properties

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A body line: often one that looks like metadata.
fn body_line() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z_]{1,8}: [a-z0-9 ]{0,8}",
        Just(String::new()),
        Just("   ".to_owned()),
        "[^\n\r]{0,16}",
    ]
}

fn body() -> impl Strategy<Value = String> {
    prop::collection::vec(body_line(), 0..8).prop_map(|lines| lines.join("\n"))
}

fn props() -> impl Strategy<Value = Vec<(String, String)>> {
    let value = "([A-Za-z0-9:./_-]{1,10}( [A-Za-z0-9:./_-]{1,5})?)?";
    (
        prop::collection::vec(("[a-z][a-z_]{0,10}", value), 0..6),
        "[0-9]{1,2}",
        any::<prop::sample::Index>(),
    )
        .prop_map(|(mut props, kind, at)| {
            let at = at.index(props.len() + 1);
            props.insert(at, ("type_".to_owned(), kind));
            props
        })
}

fn item() -> impl Strategy<Value = Item> {
    ("[^\n\r]{0,20}", body(), props()).prop_map(|(title, body, props)| Item { title, body, props })
}

/// Text and Joplin links, some to IDs the rewrite knows.
#[derive(Clone, Debug)]
enum Piece {
    Text(String),
    Link {
        label: String,
        id: String,
        image: bool,
    },
    Html {
        id: String,
    },
}

fn id() -> impl Strategy<Value = String> {
    "[0-9a-f]{32}"
}

fn piece() -> impl Strategy<Value = Piece> {
    prop_oneof![
        // No brackets, backticks, angle brackets or runs of spaces.
        "[a-zA-Z0-9.,\n-]( ?[a-zA-Z0-9.,:/\n-]){0,12}".prop_map(Piece::Text),
        ("[a-zA-Z0-9 ]{0,8}", id(), any::<bool>()).prop_map(|(label, id, image)| Piece::Link {
            label,
            id,
            image
        }),
        id().prop_map(|id| Piece::Html { id }),
    ]
}

fn render(pieces: &[Piece], known: &HashSet<String>, rewrite: bool) -> String {
    let target = |id: &str| {
        if rewrite && known.contains(id) {
            format!("new/{id}")
        } else {
            format!(":/{id}")
        }
    };
    let mut out = String::new();
    for piece in pieces {
        // A space keeps neighbours apart: `]` then `(` would join them.
        out.push(' ');
        match piece {
            Piece::Text(text) => out.push_str(text),
            Piece::Link { label, id, image } => {
                if *image {
                    out.push('!');
                }
                out.push_str(&format!("[{label}]({})", target(id)));
            }
            Piece::Html { id } => out.push_str(&format!("<img src=\"{}\">", target(id))),
        }
    }
    out
}

proptest! {
    #![proptest_config(config(256))]

    #[test]
    fn items_read_back_as_written(item in item()) {
        let text = render_item(&item);
        prop_assert_eq!(parse_item(&text), Some(item));
    }

    #[test]
    fn bodies_ending_in_metadata_like_lines_are_kept(
        head in body(),
        tail in prop::collection::vec("[a-z_]{1,8}: [a-z0-9]{0,8}", 1..5),
        props in props(),
    ) {
        let body = format!("{head}\n{}", tail.join("\n"));
        let item = Item { title: "Title".to_owned(), body: body.clone(), props: props.clone() };
        let read = parse_item(&render_item(&item)).unwrap();
        prop_assert_eq!(read.body, body);
        prop_assert_eq!(read.props, props);
    }

    #[test]
    fn rewriting_touches_only_known_links(
        pieces in prop::collection::vec(piece(), 0..12),
        pick in prop::collection::vec(any::<bool>(), 12),
    ) {
        let ids: Vec<&String> = pieces
            .iter()
            .filter_map(|p| match p {
                Piece::Link { id, .. } | Piece::Html { id } => Some(id),
                Piece::Text(_) => None,
            })
            .collect();
        let known: HashSet<String> = ids
            .iter()
            .zip(&pick)
            .filter(|(_, keep)| **keep)
            .map(|(id, _)| (*id).clone())
            .collect();
        let body = render(&pieces, &known, false);
        let out = rewrite_links(&body, |id| known.contains(id).then(|| format!("new/{id}")));
        prop_assert_eq!(out, render(&pieces, &known, true));
    }
}
