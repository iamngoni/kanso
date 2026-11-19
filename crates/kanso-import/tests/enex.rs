//! Tests for importing Evernote exports (`.enex`): example imports of the
//! fixtures in `tests/fixtures/evernote`, and property tests for the ENML
//! converter. CI pins the seed with `PROPTEST_RNG_SEED`.

use std::path::{Path, PathBuf};

use kanso_import::library::enex::{Resources, convert, enml_to_markdown};
use kanso_import::library::{IMPORT_FIELD, LibraryReport, SourceKind, import_source};
use kanso_store::{Doc, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn fixture(name: &str) -> PathBuf {
    manifest_dir().join("tests/fixtures/evernote").join(name)
}

/// An empty store in a temporary folder.
struct Env {
    _dir: tempfile::TempDir,
    store: Store,
}

impl Env {
    fn new() -> Env {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        Env { _dir: dir, store }
    }

    fn import(&self, path: &Path) -> LibraryReport {
        import_source(SourceKind::Evernote, path, &self.store, |_| {}).unwrap()
    }

    fn notes(&self) -> Vec<Doc> {
        let mut notes: Vec<Doc> = self
            .store
            .search(&kanso_store::SearchParams::default())
            .unwrap();
        notes.sort_by(|a, b| a.str_field("title").cmp(&b.str_field("title")));
        notes
    }

    fn note(&self, title: &str) -> Doc {
        self.notes()
            .into_iter()
            .find(|n| n.str_field("title") == Some(title))
            .unwrap_or_else(|| panic!("no note titled {title}"))
    }

    fn book(&self, note: &Doc) -> Doc {
        self.store
            .get(note.str_field("bookId").unwrap())
            .unwrap()
            .unwrap()
    }

    fn tag_names(&self, note: &Doc) -> Vec<String> {
        let mut names: Vec<String> = note.fields["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| {
                let tag = self.store.get(id.as_str().unwrap()).unwrap().unwrap();
                tag.str_field("name").unwrap().to_owned()
            })
            .collect();
        names.sort();
        names
    }
}

/// The ID suffix of the attachment linked right after `before` in `body`.
fn file_suffix(body: &str, before: &str) -> String {
    let at = body
        .find(before)
        .unwrap_or_else(|| panic!("{before} not in {body}"));
    let rest = &body[at..];
    let start = rest.find("kanso://file:").unwrap() + "kanso://file:".len();
    rest[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

fn png_bytes() -> Vec<u8> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(PNG)
        .unwrap()
}

#[test]
fn an_evernote_export_comes_in_whole() {
    let env = Env::new();
    let report = env.import(&fixture("Travel.enex"));
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.source, "Travel");
    assert_eq!(
        report.found.notes, 3,
        "the broken note is left out: {report}"
    );
    assert_eq!(report.notes.created, 3);
    assert_eq!(report.notebooks.created, 1);
    assert_eq!(
        report.found.attachments, 2,
        "the shared picture is one: {report}"
    );
    assert_eq!(report.attachments.created, 2, "{report}");
    assert_eq!(report.unresolved_links, 0);
    let mut skipped: Vec<(&str, &str)> = report
        .skipped
        .iter()
        .map(|s| (s.path.as_str(), s.reason.as_str()))
        .collect();
    skipped.sort();
    assert_eq!(skipped.len(), 2, "{skipped:?}");
    assert_eq!(skipped[0].0, "Travel.enex › Broken");
    assert!(skipped[0].1.contains("</titel>"), "{skipped:?}");
    assert_eq!(skipped[1].0, "Travel.enex › Safe");
    assert!(skipped[1].1.contains("encrypted"), "{skipped:?}");

    let packing = env.note("Packing list");
    let body = packing.str_field("body").unwrap();
    let dot = file_suffix(body, "![dot.png](");
    assert_eq!(
        body,
        format!(
            "Things to bring — **don’t** forget!\n\n- [x] Passport\n- [ ] Charger\n\n\
             ![dot.png](kanso://file:{dot})"
        )
    );
    assert_eq!(env.tag_names(&packing), ["lists", "travel"]);
    assert_eq!(packing.fields["createdAt"], json!(1_577_880_000_000i64));
    assert_eq!(packing.fields["updatedAt"], json!(1_577_952_000_000i64));
    assert_eq!(env.book(&packing).str_field("name"), Some("Travel"));
    assert_eq!(
        packing.fields[IMPORT_FIELD]["key"],
        "Travel/Packing list@20200101T120000Z"
    );

    let itinerary = env.note("Itinerary & tickets");
    let body = itinerary.str_field("body").unwrap();
    let tickets = file_suffix(body, "[tickets.pdf](");
    assert_eq!(
        body,
        format!(
            "## Days\n\n| Day | City |\n| --- | --- |\n| 1 | Lisbon |\n| 2 | Porto \\| Braga |\n\n\
             ```\ngate = \"B12\"\nseat = 14\n```\n\n\
             Tickets: [tickets.pdf](kanso://file:{tickets})\n\n\
             ![dot.png](kanso://file:{dot})\n\n\
             See [the trip page](https://example.com/trip)."
        )
    );
    assert_eq!(env.tag_names(&itinerary), ["travel"]);

    let (content_type, picture) = env
        .store
        .attachment(&format!("file:{dot}"))
        .unwrap()
        .unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(picture.data, png_bytes());
    let (content_type, pdf) = env
        .store
        .attachment(&format!("file:{tickets}"))
        .unwrap()
        .unwrap();
    assert_eq!(content_type, "application/pdf");
    assert_eq!(pdf.data, b"%PDF-1.4\n%tiny fixture\n");
    let pdf_doc = env.store.get(&format!("file:{tickets}")).unwrap().unwrap();
    assert_eq!(pdf_doc.str_field("name"), Some("tickets.pdf"));

    let safe = env.note("Safe");
    assert_eq!(
        safe.str_field("body"),
        Some("Combination:\n\n[Encrypted content not imported]")
    );
}

#[test]
fn importing_an_export_again_changes_nothing() {
    let env = Env::new();
    let path = fixture("Travel.enex");
    env.import(&path);
    let before = env.notes();
    let again = env.import(&path);
    assert_eq!(again.notes.unchanged, 3, "{again}");
    assert_eq!(again.notes.written(), 0, "{again}");
    assert_eq!(again.notebooks.unchanged, 1);
    assert_eq!(again.attachments.written(), 0, "{again}");
    assert_eq!(again.tags_created, 0);
    assert_eq!(env.notes(), before, "nothing was rewritten");
}

#[test]
fn a_folder_of_exports_makes_a_notebook_each() {
    let env = Env::new();
    let report = env.import(&fixture("notebooks"));
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.source, "notebooks");
    assert_eq!(report.found.notes, 3, "{report}");
    // The folder's notebook, Home and Work.
    assert_eq!(report.notebooks.created, 3, "{report}");

    let groceries = env.note("Groceries");
    assert_eq!(groceries.str_field("body"), Some("Milk, eggs"));
    let home = env.book(&groceries);
    assert_eq!(home.str_field("name"), Some("Home"));
    let root = env
        .store
        .get(home.str_field("parentBookId").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(root.str_field("name"), Some("notebooks"));

    // A note with the same title and date in each export is two notes.
    let standups: Vec<Doc> = env
        .notes()
        .into_iter()
        .filter(|n| n.str_field("title") == Some("Standup"))
        .collect();
    assert_eq!(standups.len(), 2);
    let work = standups
        .iter()
        .find(|n| env.book(n).str_field("name") == Some("Work"))
        .unwrap();
    assert_eq!(work.str_field("body"), Some("- Yesterday\n- Today"));
    assert_eq!(
        work.fields[IMPORT_FIELD]["key"],
        "Work.enex/Standup@20230301T090000Z"
    );
    assert_eq!(
        env.book(work).str_field("parentBookId"),
        Some(root.id.as_str())
    );
}

// ---------------------------------------------------------------------------
// Properties of the ENML converter.

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A run of words in a paragraph.
#[derive(Clone, Debug)]
enum Run {
    Plain(Vec<String>),
    Bold(Vec<String>),
    Italic(Vec<String>),
    Link(Vec<String>, String),
}

/// A block of a generated note.
#[derive(Clone, Debug)]
enum GenBlock {
    Paragraph(Vec<Run>),
    Heading(usize, Vec<String>),
    Bullets(Vec<Vec<String>>),
    Task(bool, Vec<String>),
}

fn words() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z]{1,8}", 1..5)
}

fn run() -> impl Strategy<Value = Run> {
    prop_oneof![
        3 => words().prop_map(Run::Plain),
        1 => words().prop_map(Run::Bold),
        1 => words().prop_map(Run::Italic),
        1 => (words(), "[a-z]{1,6}").prop_map(|(w, slug)| Run::Link(w, slug)),
    ]
}

fn block() -> impl Strategy<Value = GenBlock> {
    prop_oneof![
        3 => prop::collection::vec(run(), 1..5).prop_map(GenBlock::Paragraph),
        1 => (1usize..=6, words()).prop_map(|(level, w)| GenBlock::Heading(level, w)),
        1 => prop::collection::vec(words(), 1..4).prop_map(GenBlock::Bullets),
        1 => (any::<bool>(), words()).prop_map(|(done, w)| GenBlock::Task(done, w)),
    ]
}

/// Words as ENML text, with the whitespace between them varied.
fn enml_words(words: &[String], gap: &str) -> String {
    words.join(gap)
}

/// The ENML for `blocks` and the Markdown it should become.
fn render(blocks: &[GenBlock], gap: &str) -> (String, String) {
    let mut enml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE en-note SYSTEM \"http://xml.evernote.com/pub/enml2.dtd\">\n<en-note>",
    );
    let mut md = String::new();
    for (ix, block) in blocks.iter().enumerate() {
        if ix > 0 {
            let tight =
                matches!(block, GenBlock::Task(..)) && matches!(blocks[ix - 1], GenBlock::Task(..));
            md.push_str(if tight { "\n" } else { "\n\n" });
            enml.push_str(gap);
        }
        match block {
            GenBlock::Paragraph(runs) => {
                let mut parts_enml = Vec::new();
                let mut parts_md = Vec::new();
                for run in runs {
                    let (e, m) = match run {
                        Run::Plain(w) => (enml_words(w, gap), w.join(" ")),
                        Run::Bold(w) => (
                            format!("<b>{}</b>", enml_words(w, gap)),
                            format!("**{}**", w.join(" ")),
                        ),
                        Run::Italic(w) => (
                            format!("<i>{}</i>", enml_words(w, gap)),
                            format!("*{}*", w.join(" ")),
                        ),
                        Run::Link(w, slug) => (
                            format!(
                                "<a href=\"https://example.com/{slug}\">{}</a>",
                                enml_words(w, gap)
                            ),
                            format!("[{}](https://example.com/{slug})", w.join(" ")),
                        ),
                    };
                    parts_enml.push(e);
                    parts_md.push(m);
                }
                enml.push_str(&format!("<div>{}</div>", parts_enml.join(gap)));
                md.push_str(&parts_md.join(" "));
            }
            GenBlock::Heading(level, w) => {
                enml.push_str(&format!("<h{level}>{}</h{level}>", enml_words(w, gap)));
                md.push_str(&format!("{} {}", "#".repeat(*level), w.join(" ")));
            }
            GenBlock::Bullets(items) => {
                enml.push_str("<ul>");
                let mut lines = Vec::new();
                for item in items {
                    enml.push_str(&format!("{gap}<li>{}</li>", enml_words(item, gap)));
                    lines.push(format!("- {}", item.join(" ")));
                }
                enml.push_str("</ul>");
                md.push_str(&lines.join("\n"));
            }
            GenBlock::Task(done, w) => {
                enml.push_str(&format!(
                    "<div><en-todo checked=\"{done}\"/>{}</div>",
                    enml_words(w, gap)
                ));
                md.push_str(&format!(
                    "- [{}] {}",
                    if *done { "x" } else { " " },
                    w.join(" ")
                ));
            }
        }
    }
    enml.push_str("</en-note>");
    (enml, md)
}

/// Text as XML character data.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Markdown text with its backslash escapes undone.
fn md_unescape(md: &str) -> String {
    let mut out = String::new();
    let mut chars = md.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(&next) = chars.peek()
            && next.is_ascii_punctuation()
        {
            out.push(next);
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Bits of ENML, well-formed or not, to build tag soup from.
const SOUP: &[&str] = &[
    "<div>",
    "</div>",
    "<p>",
    "</p>",
    "<b>",
    "</b>",
    "<i>",
    "</i>",
    "<s>",
    "</s>",
    "<u>",
    "</u>",
    "<span style=\"color:red\">",
    "</span>",
    "<code>",
    "</code>",
    "<a href=\"https://x.y/(z)\">",
    "<a>",
    "</a>",
    "<ul>",
    "</ul>",
    "<ol start=\"2\">",
    "</ol>",
    "<li>",
    "</li>",
    "<ul style=\"--en-todo:true;\">",
    "<li style=\"--en-checked:true;\">",
    "<table>",
    "</table>",
    "<tr>",
    "</tr>",
    "<td>",
    "</td>",
    "<th>",
    "</th>",
    "<tbody>",
    "</tbody>",
    "<h1>",
    "</h1>",
    "<h3>",
    "</h3>",
    "<pre>",
    "</pre>",
    "<blockquote>",
    "</blockquote>",
    "<div style=\"-en-codeblock:true\">",
    "<en-crypt hint=\"x\">",
    "</en-crypt>",
    "<br/>",
    "<br>",
    "<hr/>",
    "<en-todo checked=\"true\"/>",
    "<en-todo/>",
    "<en-media hash=\"abc\" type=\"image/png\"/>",
    "<en-media hash=\"def\"/>",
    "<img src=\"https://x.y/p.png\" alt=\"[p]\"/>",
    "<img src=\"data:x\"/>",
    "<style>",
    "</style>",
    "<en-note>",
    "</en-note>",
    "word",
    " ",
    "\n",
    "  ",
    "&nbsp;",
    "&mdash;",
    "&bogus;",
    "&#65;",
    "&#xZZ;",
    "&",
    "<",
    ">",
    "*_[]#",
    "1.",
    "- ",
    "|",
    "`",
    "~~",
    "\\",
    "!",
    "<!-- c -->",
    "<![CDATA[x<y]]>",
    "<?pi x?>",
    "<div",
    "\"",
    "=",
];

proptest! {
    #![proptest_config(config(256))]

    /// A note made of paragraphs, headings, bold and italic runs, lists,
    /// tasks and links becomes exactly the Markdown it was made from.
    #[test]
    fn generated_notes_round_trip(
        blocks in prop::collection::vec(block(), 1..8),
        gap in prop::sample::select(vec![" ", "\n", "  ", "\n  "]),
    ) {
        let (enml, expected) = render(&blocks, gap);
        prop_assert_eq!(enml_to_markdown(&enml, &Resources::new()), expected, "{}", enml);
    }

    /// Text with XML and Markdown characters in it comes back as the same
    /// text once the Markdown escapes are undone.
    #[test]
    fn text_survives_as_text(
        chars in prop::collection::vec(
            prop::sample::select(vec![
                'a', 'b', 'Z', '0', '1', '9', ' ', ' ', '\n', '\t', '\u{a0}', '&', '<', '>', '"',
                '\'', '*', '_', '`', '[', ']', '(', ')', '#', '-', '+', '.', '!', '|', '~', '\\',
                '=', ':', ';', '{', '}', '/', 'é', '—',
            ]),
            0..60,
        ),
    ) {
        let text: String = chars.into_iter().collect();
        let enml = format!("<en-note><div>{}</div></en-note>", xml_escape(&text));
        let md = enml_to_markdown(&enml, &Resources::new());
        prop_assert_eq!(normalize(&md_unescape(&md)), normalize(&text), "{}", md);
        prop_assert!(!md.contains('\n'), "one paragraph stays one line: {:?}", md);
    }

    /// Any tag soup converts without panicking, to trimmed Markdown.
    #[test]
    fn tag_soup_never_panics(pieces in prop::collection::vec(prop::sample::select(SOUP.to_vec()), 0..60)) {
        let enml = pieces.concat();
        let mut resources = Resources::new();
        resources.insert("abc".into(), ("p.png".into(), "image/png".into()));
        let converted = convert(&enml, &resources);
        prop_assert_eq!(converted.markdown.trim(), converted.markdown.as_str());
        prop_assert!(!converted.markdown.contains("embedded/def"), "unknown media is left out");
    }
}
