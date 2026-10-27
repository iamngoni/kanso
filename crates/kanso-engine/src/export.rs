//! Exporting notes: as Markdown files, one `.md` per note in folders named
//! after its notebooks, attachments in `_attachments/`, and in-app links
//! rewritten to relative paths so the export reads well in any Markdown
//! editor (files already in the chosen folder are never replaced; new ones
//! get numbered names instead), and as web pages (`page`), which the apps
//! save as HTML or print to PDF and which `publish` seals at a link.
//!
//! The desktop and the mobile app call the same code, so a page looks the
//! same wherever it was made. `style` has the faces and code colors a page
//! is set in.

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use anyhow::Context as _;
use kanso_core::links::{FILE_LINK_PREFIX, LINK_SCHEME, NOTE_LINK_PREFIX};
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::properties::{self, Value};
use kanso_core::{Book, Note, NoteStatus};
use kanso_render::Palette;
use kanso_store::{SearchParams, Store};

mod model;
mod style;

pub use model::{CodeColors, ExportReport, PageStyle};
pub use style::code_for_page;

const MAX_NAME_CHARS: usize = 100;
const ATTACHMENTS_DIR: &str = "_attachments";

/// A file or folder name that is safe on macOS, Windows and Linux.
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let truncated: String = collapsed.chars().take(MAX_NAME_CHARS).collect();
    let trimmed = truncated.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() {
        "Untitled".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// `base` + `extension`, numbered when the name is taken. Names compare
/// case-insensitively, as on macOS.
pub fn unique_name(base: &str, extension: &str, taken: &mut HashSet<String>) -> String {
    let mut n = 1;
    loop {
        let name = if n == 1 {
            format!("{base}{extension}")
        } else {
            format!("{base} {n}{extension}")
        };
        if taken.insert(name.to_lowercase()) {
            return name;
        }
        n += 1;
    }
}

/// The path from directory `from` to `to`, both relative to the export root,
/// with `/` separators.
pub fn relative_path(from: &Path, to: &Path) -> String {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_owned(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

/// Percent-encodes the characters that would end a Markdown link target.
pub fn encode_link(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '%' => out.push_str("%25"),
            c => out.push(c),
        }
    }
    out
}

/// Replaces note links ([`NOTE_LINK_PREFIX`]`<id>`) and attachment links
/// ([`FILE_LINK_PREFIX`]`<id>`) with what the
/// lookups return for the full IDs (`note:…`, `file:…`). Links the lookups
/// don't know are left as they are.
pub fn rewrite_links(
    body: &str,
    note_link: impl Fn(&str) -> Option<String>,
    file_link: impl Fn(&str) -> Option<String>,
) -> String {
    const NOTE: &str = NOTE_LINK_PREFIX;
    const FILE: &str = FILE_LINK_PREFIX;
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find(LINK_SCHEME) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let (prefix, kind) = if tail.starts_with(NOTE) {
            (NOTE, "note:")
        } else if tail.starts_with(FILE) {
            (FILE, "file:")
        } else {
            out.push_str(LINK_SCHEME);
            rest = &tail[LINK_SCHEME.len()..];
            continue;
        };
        let suffix_len = tail[prefix.len()..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(tail.len() - prefix.len());
        let suffix = &tail[prefix.len()..prefix.len() + suffix_len];
        let id = format!("{kind}{suffix}");
        let replacement = if suffix.is_empty() {
            None
        } else if kind == "note:" {
            note_link(&id)
        } else {
            file_link(&id)
        };
        match replacement {
            Some(link) => out.push_str(&link),
            None => out.push_str(&tail[..prefix.len() + suffix_len]),
        }
        rest = &tail[prefix.len() + suffix_len..];
    }
    out.push_str(rest);
    out
}

/// The note's text for an export: one YAML front-matter block, the user's
/// own entries byte-for-byte, and Kanso's keys added only where the user
/// hasn't written them. When the user's front matter already has one of
/// Kanso's keys, Kanso's value goes beside it as `kanso-<key>` so an import
/// reads both; tags, which can hold both sets, are written once as a union.
pub fn export_text(note: &Note, tag_names: &[String], body: &str) -> String {
    let quote = |s: &str| serde_json::to_string(s).unwrap_or_default();
    let mut kanso: Vec<(&str, String)> = vec![("title", quote(&note.title))];
    if !tag_names.is_empty() {
        let tags: Vec<String> = tag_names.iter().map(|t| quote(t)).collect();
        kanso.push(("tags", format!("[{}]", tags.join(", "))));
    }
    if note.status != NoteStatus::None {
        kanso.push(("status", note.status.as_str().to_owned()));
    }
    if note.pinned {
        kanso.push(("pinned", "true".to_owned()));
    }
    kanso.push(("created", iso_time(note.created_at)));
    kanso.push(("updated", iso_time(note.updated_at)));
    let Some(front) = properties::split(body) else {
        let eol = properties::line_ending(body);
        let mut out = format!("---{eol}");
        for (key, value) in kanso {
            out.push_str(&format!("{key}: {value}{eol}"));
        }
        out.push_str(&format!("---{eol}{eol}"));
        out.push_str(body);
        return out;
    };
    let eol = front.eol.to_owned();
    let mut out = body.to_owned();
    // Tags can be merged without losing either set, so a user's `tags` and
    // Kanso's become one entry the importer splits back into both.
    let mut tags_merged = false;
    if !tag_names.is_empty() {
        let parsed = properties::parse(&out);
        if let Some(user) = properties::get(&parsed, "tags") {
            let mut union = tags_from(Some(user));
            for tag in tag_names {
                if !union.iter().any(|kept| kept.eq_ignore_ascii_case(tag)) {
                    union.push(tag.clone());
                }
            }
            if !union.is_empty() {
                let list = Value::List(union.into_iter().map(Value::Text).collect());
                out = properties::set_property(&out, "tags", &list);
                tags_merged = true;
            }
        }
    }
    let written: HashSet<String> = properties::parse(&out)
        .into_iter()
        .map(|(key, _)| key.to_lowercase())
        .collect();
    for (key, value) in kanso {
        if key == "tags" && tags_merged {
            continue;
        }
        let name = if written.contains(key) {
            format!("kanso-{key}")
        } else {
            key.to_owned()
        };
        insert_before_close(&mut out, &format!("{name}: {value}{eol}"));
    }
    out
}

/// The tags a user's value names, as the importer reads them.
fn tags_from(value: Option<&Value>) -> Vec<String> {
    let split = |text: &str| -> Vec<String> {
        text.split(|c: char| c == ',' || c.is_whitespace())
            .map(|tag| tag.trim().trim_start_matches('#').trim_end_matches('/'))
            .filter(|tag| !tag.is_empty())
            .map(str::to_owned)
            .collect()
    };
    match value {
        Some(Value::Text(text)) => split(text),
        Some(Value::List(items)) => items.iter().flat_map(|item| split(&item.text())).collect(),
        _ => Vec::new(),
    }
}

/// Writes one line at the end of the note's front matter.
fn insert_before_close(out: &mut String, line: &str) {
    if let Some(front) = properties::split(out) {
        let at = front.close_start;
        out.insert_str(at, line);
    }
}

/// `YYYY-MM-DD` in UTC, as an export's front matter dates a note.
fn format_date(ms: i64) -> String {
    let days = ms.div_euclid(86_400_000);
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn iso_time(ms: i64) -> String {
    let date = format_date(ms);
    let secs = ms.div_euclid(1000).rem_euclid(86_400);
    format!(
        "{date}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        secs / 60 % 60,
        secs % 60
    )
}

/// Folder path of each notebook, relative to the export root.
fn book_dirs(books: &[Book]) -> HashMap<String, PathBuf> {
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    let mut taken: HashMap<Option<String>, HashSet<String>> = HashMap::new();
    // A top-level notebook must not share the attachments folder.
    taken
        .entry(None)
        .or_default()
        .insert(ATTACHMENTS_DIR.to_lowercase());
    for (_, book) in crate::list::notebook_tree(books) {
        let parent = book.parent_book_id.clone().filter(|p| dirs.contains_key(p));
        let parent_dir = parent.as_ref().map(|p| dirs[p].clone()).unwrap_or_default();
        let name = unique_name(
            &sanitize_file_name(&book.name),
            "",
            taken.entry(parent).or_default(),
        );
        dirs.insert(book.id.clone(), parent_dir.join(name));
    }
    dirs
}

/// Writes notes into `root`. With `only`, just those notes (and their
/// attachments); otherwise every note outside the trash.
pub fn export(store: &Store, root: &Path, only: Option<&[String]>) -> anyhow::Result<ExportReport> {
    let books = store.books()?;
    let tags: HashMap<String, String> = store.tags()?.into_iter().map(|t| (t.id, t.name)).collect();
    let dirs = book_dirs(&books);
    let notes: Vec<Note> = match only {
        Some(ids) => ids
            .iter()
            .filter_map(|id| store.note(id).ok().flatten())
            .collect(),
        None => store
            .search(&SearchParams::default())?
            .into_iter()
            .filter_map(|doc| serde_json::from_value(doc.to_api(Default::default())).ok())
            .collect(),
    };
    let notes: Vec<Note> = notes
        .into_iter()
        .filter(|n| n.book_id != TRASH_BOOK_ID)
        .collect();
    // Locked notes, and every note in a locked notebook, stay out.
    let locked_books = store.locked_books()?;
    let (locked, notes): (Vec<Note>, Vec<Note>) = notes
        .into_iter()
        .partition(|note| note.is_locked() || locked_books.contains(&note.book_id));

    // Decide every path first so links between notes can point at them.
    // Names in use, on disk or by notebook folders, are taken.
    let mut taken: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for dir in dirs.values() {
        if let (Some(parent), Some(name)) = (dir.parent(), dir.file_name()) {
            taken
                .entry(parent.to_path_buf())
                .or_insert_with(|| existing_names(&root.join(parent)))
                .insert(name.to_string_lossy().to_lowercase());
        }
    }
    let mut note_paths: HashMap<String, PathBuf> = HashMap::new();
    for note in &notes {
        let dir = dirs.get(&note.book_id).cloned().unwrap_or_default();
        let name = unique_name(
            &sanitize_file_name(&note.title),
            ".md",
            taken
                .entry(dir.clone())
                .or_insert_with(|| existing_names(&root.join(&dir))),
        );
        note_paths.insert(note.id.clone(), dir.join(name));
    }
    let mut file_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut attachment_names = existing_names(&root.join(ATTACHMENTS_DIR));
    for note in &notes {
        for file_id in kanso_core::tasks::linked_file_ids(&note.body) {
            if file_paths.contains_key(&file_id) {
                continue;
            }
            let Some(meta) = store.file_meta(&file_id)? else {
                continue;
            };
            let (stem, extension) = split_extension(&meta.name);
            let name = unique_name(&sanitize_file_name(stem), &extension, &mut attachment_names);
            file_paths.insert(file_id, Path::new(ATTACHMENTS_DIR).join(name));
        }
    }

    let mut report = ExportReport {
        locked: locked.len(),
        ..ExportReport::default()
    };
    for (file_id, path) in &file_paths {
        if let Some((_, attachment)) = store.attachment(file_id)? {
            write_new(&root.join(path), &attachment.data)?;
            report.attachments += 1;
        }
    }
    for note in &notes {
        let path = &note_paths[&note.id];
        let dir = path.parent().unwrap_or(Path::new(""));
        let body = rewrite_links(
            &note.body,
            |id| {
                note_paths
                    .get(id)
                    .map(|to| encode_link(&relative_path(dir, to)))
            },
            |id| {
                file_paths
                    .get(id)
                    .map(|to| encode_link(&relative_path(dir, to)))
            },
        );
        let tag_names: Vec<String> = note
            .tags
            .iter()
            .filter_map(|t| tags.get(t).cloned())
            .collect();
        let text = export_text(note, &tag_names, &body);
        write_new(&root.join(path), text.as_bytes())?;
        report.notes += 1;
    }
    Ok(report)
}

fn split_extension(name: &str) -> (&str, String) {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() && ext.len() <= 8 => {
            (stem, format!(".{}", sanitize_file_name(ext)))
        }
        _ => (name, String::new()),
    }
}

/// Names already in `dir`, lowercased.
fn existing_names(dir: &Path) -> HashSet<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

/// Writes a file that must not exist yet.
fn write_new(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?;
    file.write_all(data)
        .with_context(|| format!("writing {}", path.display()))
}

/// The page for a note: its title and body, in `palette` and `style`, with
/// its attachments read from `store` and its query views drawn. Blocks while
/// math and diagrams are drawn.
pub fn page(
    store: &Store,
    title: &str,
    body: &str,
    palette: &Palette,
    self_id: Option<&str>,
    style: &PageStyle,
) -> String {
    let body = with_queries(store, body, self_id);
    page_without_queries(store, title, &body, palette, style)
}

/// The page as it was before query views: a `query` fence stays code.
pub fn page_without_queries(
    store: &Store,
    title: &str,
    body: &str,
    palette: &Palette,
    style: &PageStyle,
) -> String {
    let attachments = |id: &str| {
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    kanso_render::html::page_with(title, body, palette, &attachments, &style.options())
}

/// `body` with each query fence drawn as the table it makes.
pub fn with_queries(store: &Store, body: &str, self_id: Option<&str>) -> String {
    // Most pages have no query: don't read every note to find that out.
    if !kanso_core::query_view::contains_query(body) {
        return body.to_owned();
    }
    let Ok(rows) = store.query_view_notes() else {
        return body.to_owned();
    };
    let notes: Vec<kanso_core::query_view::ViewNote> = rows
        .into_iter()
        .map(kanso_core::query_view::ViewNote::from)
        .collect();
    kanso_core::query_view::render_export(body, &notes, self_id)
}

/// A file name for the note's export.
pub fn file_name(title: &str, extension: &str) -> String {
    format!("{}.{extension}", sanitize_file_name(title))
}

/// The colors to print in: `palette` when it is light, else Kanso's light
/// theme.
pub fn print_palette(palette: Palette) -> Palette {
    if palette.dark {
        Palette::LIGHT
    } else {
        palette
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PageStyle, Palette, encode_link, export, export_text, file_name, format_date, page,
        page_without_queries, print_palette, relative_path, rewrite_links, sanitize_file_name,
        unique_name,
    };
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;
    use std::collections::{BTreeMap, HashSet};
    use std::path::{Path, PathBuf};

    fn note_with(title: &str, body: &str) -> kanso_core::Note {
        kanso_core::Note {
            id: "note:test".into(),
            rev: None,
            title: title.into(),
            body: body.into(),
            book_id: "book:test".into(),
            tags: Vec::new(),
            status: kanso_core::NoteStatus::None,
            share: kanso_core::NoteShare::default(),
            pinned: false,
            doctype: "markdown".into(),
            created_at: 0,
            updated_at: 0,
            timestamp: 0,
            num_of_tasks: 0,
            num_of_checked_tasks: 0,
            extra: Default::default(),
        }
    }

    #[test]
    fn user_front_matter_is_kept_in_one_block() {
        let note = note_with("Plan", "");
        let body = "---\ntitle: My title\nowner: Ana  # keep\n---\n\n# Plan\n";
        let text = export_text(&note, &["work".to_owned()], body);
        assert_eq!(text.matches("---").count(), 2, "{text}");
        assert!(text.contains("title: My title\n"), "{text}");
        assert!(text.contains("owner: Ana  # keep"), "{text}");
        assert!(text.contains("tags: [\"work\"]"), "{text}");
        assert!(text.contains("created:"), "{text}");
        assert!(text.ends_with("\n\n# Plan\n"), "{text}");
        // Both titles survive: the user's line untouched, Kanso's beside it
        // under a key the importer reads.
        assert!(text.contains("kanso-title: \"Plan\""), "{text}");
    }

    #[test]
    fn colliding_user_keys_keep_both_values() {
        let mut note = note_with("Plan", "");
        note.status = kanso_core::NoteStatus::Active;
        note.pinned = true;
        let body = "---\ntitle: My title\ntags: [project]\nstatus: dropped\npinned: false\nowner: Ana\n---\n\n# Plan\n";
        let text = export_text(&note, &["work".to_owned()], body);
        // The user's entries stay, and Kanso's values go beside them.
        assert!(text.contains("title: My title\n"), "{text}");
        assert!(text.contains("owner: Ana\n"), "{text}");
        assert!(text.contains("status: dropped\n"), "{text}");
        assert!(text.contains("pinned: false\n"), "{text}");
        assert!(text.contains("kanso-title: \"Plan\""), "{text}");
        assert!(text.contains("kanso-status: active\n"), "{text}");
        assert!(text.contains("kanso-pinned: true\n"), "{text}");
        // Tags are one union entry.
        assert_eq!(text.matches("tags:").count(), 1, "{text}");
        assert!(text.contains("project"), "{text}");
        assert!(text.contains("work"), "{text}");

        let (front, rest) = kanso_import::library::frontmatter::split(&text).unwrap();
        let meta = kanso_import::library::frontmatter::meta(front);
        assert_eq!(meta.title.as_deref(), Some("Plan"));
        assert_eq!(meta.tags, ["project", "work"]);
        assert_eq!(meta.status, Some(kanso_core::NoteStatus::Active));
        assert!(meta.pinned);
        // The user's own entries come back as written.
        assert_eq!(
            meta.rest,
            "title: My title\nstatus: dropped\npinned: false\nowner: Ana\n"
        );
        assert_eq!(rest, "\n# Plan\n");
    }

    fn round_trip_note(
        title: &str,
        status: kanso_core::NoteStatus,
        pinned: bool,
    ) -> kanso_core::Note {
        let mut note = note_with(title, "");
        note.status = status;
        note.pinned = pinned;
        note
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Exporting and importing again loses nothing when the user's own
        /// front matter uses Kanso's keys: the body, tags, title, status and
        /// pinned state all come back.
        #[test]
        fn prop_export_import_round_trip_with_collisions(
            title in "[A-Za-z0-9]+( [A-Za-z0-9]+)*",
            tag in "[A-Za-z0-9_-]{1,12}",
            user_title in "[A-Za-z0-9]+( [A-Za-z0-9]+)*",
            user_tag in "[A-Za-z0-9_-]{1,12}",
            owner in "[A-Za-z0-9]{1,12}",
            status_ix in 0usize..5,
            pinned in any::<bool>(),
            prose in "[A-Za-z0-9 .,]{0,40}",
        ) {
            let status = kanso_core::NoteStatus::ALL[status_ix];
            let note = round_trip_note(&title, status, pinned);
            let body = format!(
                "---\ntitle: {user_title}\ntags: [{user_tag}]\nowner: {owner}\n---\n\n{prose}\n"
            );
            let text = export_text(&note, std::slice::from_ref(&tag), &body);
            let (front, rest) = kanso_import::library::frontmatter::split(&text)
                .expect("the export has front matter");
            let meta = kanso_import::library::frontmatter::meta(front);
            prop_assert_eq!(meta.title.as_deref(), Some(title.as_str()));
            prop_assert!(meta.tags.iter().any(|t| t == &tag), "{:?}", meta.tags);
            // YAML types some tag text: an all-digit tag such as `00` reads
            // as the number zero, and `true` as a boolean. The parser is the
            // desktop's, so the contract here is that every tag as Kanso
            // reads it comes back, not every byte the user typed.
            let user_tags = super::tags_from(kanso_core::properties::get(
                &kanso_core::properties::parse(&body),
                "tags",
            ));
            for kept in &user_tags {
                prop_assert!(meta.tags.iter().any(|t| t == kept), "{:?}", meta.tags);
            }
            let expected_status = (status != kanso_core::NoteStatus::None).then_some(status);
            prop_assert_eq!(meta.status, expected_status);
            prop_assert_eq!(meta.pinned, pinned);
            let expected_rest = format!("\n{prose}\n");
            prop_assert_eq!(rest, expected_rest.as_str());
            // The user's own title and properties stay in the note.
            let expected_rest = format!("title: {user_title}\nowner: {owner}");
            prop_assert_eq!(meta.rest.trim(), expected_rest.as_str());
            prop_assert!(text.contains(&format!("title: {user_title}\n")), "{}", text);
        }
    }

    #[test]
    fn exports_make_one_block_when_there_was_none() {
        let note = note_with("Plan", "");
        let text = export_text(&note, &[], "# Plan\n");
        assert!(text.starts_with("---\ntitle: \"Plan\"\n"), "{text}");
        assert!(text.contains("\n---\n\n# Plan\n"), "{text}");
        assert_eq!(text.matches("---").count(), 2, "{text}");
    }

    /// Every file under `root`, relative path to contents.
    fn read_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let data = std::fs::read(&path).unwrap();
                    out.insert(path.strip_prefix(root).unwrap().to_path_buf(), data);
                }
            }
        }
        out
    }

    /// A store with notebooks named `books` and notes `(book index, title)`.
    fn library(books: &[&str], notes: &[(usize, &str)]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let ids: Vec<String> = books
            .iter()
            .map(|name| {
                store
                    .put(
                        DocType::Book,
                        json!({ "name": name }),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap()
                    .id
            })
            .collect();
        for (book, title) in notes {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": format!("body of {title}"), "bookId": ids[*book]}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        (dir, store)
    }

    #[test]
    fn exporting_twice_keeps_the_first_export() {
        let (_dir, store) = library(&["Work"], &[(0, "Plan"), (0, "plan")]);
        let out = tempfile::tempdir().unwrap();
        assert_eq!(export(&store, out.path(), None).unwrap().notes, 2);
        let first = read_tree(out.path());
        assert_eq!(export(&store, out.path(), None).unwrap().notes, 2);
        let second = read_tree(out.path());
        assert_eq!(second.len(), 4);
        for (path, data) in &first {
            assert_eq!(second.get(path), Some(data), "{} changed", path.display());
        }
    }

    #[test]
    fn a_notebook_named_like_the_attachments_folder_gets_its_own() {
        let (_dir, store) = library(&["_Attachments"], &[(0, "image.png")]);
        let encoded = "aGVsbG8=";
        let file = store
            .put(
                DocType::File,
                json!({
                    "name": "image.png",
                    "contentType": "image/png",
                    "publicIn": [],
                    "_attachments": {"index": {"content_type": "image/png", "data": encoded}},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let suffix = file.strip_prefix("file:").unwrap();
        let book = store.books().unwrap()[0].id.clone();
        store
            .put(
                DocType::Note,
                json!({"title": "pic", "body": format!("![](inkdrop://file:{suffix})"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let out = tempfile::tempdir().unwrap();
        let report = export(&store, out.path(), None).unwrap();
        assert_eq!((report.notes, report.attachments), (2, 1));
        let tree = read_tree(out.path());
        assert_eq!(tree[Path::new("_attachments/image.png")], b"hello");
        assert!(tree.contains_key(Path::new("_Attachments 2/image.png.md")));
        assert!(tree.contains_key(Path::new("_Attachments 2/pic.md")));
    }

    #[test]
    fn a_note_exports_with_its_images() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let id = store
            .put(
                DocType::File,
                json!({
                    "name": "cat.png",
                    "contentType": "image/png",
                    "_attachments": {"index": {"content_type": "image/png", "data": "iVBORw=="}},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let body = format!("Area $\\pi r^2$.\n\n![cat](inkdrop://{id})\n");
        let html = page(
            &store,
            "Circles",
            &body,
            &Palette::DARK,
            None,
            &PageStyle::default(),
        );
        assert!(html.contains("<title>Circles</title>"));
        assert!(html.contains("src=\"data:image/png;base64,"));
        assert!(html.contains("kanso-math"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn a_page_draws_query_views() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("q.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Clients"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = |title: &str, body: &str| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        note("Ana", "---\nclient: Acme\n---\n# Ana\n");
        // Sealed by hand: the store won't take text beside a lock.
        store
            .put(
                DocType::Note,
                json!({"_id": "note:sealed", "title": "Sealed", "body": "",
                       "bookId": book, "kansoLock": {"salt": "s", "nonce": "n", "data": "d"}}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let self_id = note("Self", "```query\nfrom: has:client\ncolumns: client\n```\n");
        let body = "```query\nfrom: has:client\ncolumns: client\n```\n";
        let style = PageStyle::default();
        let html = page(&store, "Self", body, &Palette::DARK, Some(&self_id), &style);
        assert!(html.contains("<table>"), "{html}");
        assert!(html.contains("Ana"), "{html}");
        assert!(html.contains("Acme"), "{html}");
        assert!(!html.contains("Sealed"), "{html}");
        assert!(!html.contains("Self</td>"), "{html}");
        assert!(
            page_without_queries(&store, "Self", body, &Palette::DARK, &style)
                .contains("language-query")
        );
    }

    #[test]
    fn pdfs_print_in_light_colors() {
        assert_eq!(print_palette(Palette::DARK), Palette::LIGHT);
        assert_eq!(print_palette(Palette::LIGHT), Palette::LIGHT);
        let sepia = Palette {
            background: kanso_render::Rgb(0xF4ECD8),
            ..Palette::LIGHT
        };
        assert_eq!(print_palette(sepia), sepia);
    }

    #[test]
    fn dates_are_utc_calendar_days() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(951_782_400_000), "2000-02-29");
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn gen_dir() -> impl Strategy<Value = PathBuf> {
        prop::collection::vec("[a-c]{1,2}", 0..4).prop_map(|parts| parts.iter().collect())
    }

    /// Resolves `rel` (with `..`) against `dir`, the way a Markdown viewer does.
    fn resolve(dir: &Path, rel: &str) -> PathBuf {
        let mut parts: Vec<String> = dir
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        for part in rel.split('/') {
            if part == ".." {
                parts.pop();
            } else {
                parts.push(part.to_owned());
            }
        }
        parts.iter().collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Refinement: names are non-empty, short, free of path and reserved
        /// characters, and don't start or end with a dot or space.
        #[test]
        fn prop_sanitized_names_are_safe(name in "\\PC{0,200}") {
            let safe = sanitize_file_name(&name);
            prop_assert!(!safe.is_empty());
            prop_assert!(safe.chars().count() <= 100);
            prop_assert!(!safe.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']));
            prop_assert!(!safe.chars().any(char::is_control));
            prop_assert!(!safe.starts_with(['.', ' ']) && !safe.ends_with(['.', ' ']));
        }

        /// Idempotence: sanitizing a sanitized name changes nothing.
        #[test]
        fn prop_sanitize_is_idempotent(name in "\\PC{0,200}") {
            let once = sanitize_file_name(&name);
            prop_assert_eq!(sanitize_file_name(&once), once);
        }

        /// Invariant: names handed out are distinct, ignoring case.
        #[test]
        fn prop_unique_names_never_collide(bases in prop::collection::vec("[aA]{1,2}", 1..20)) {
            let mut taken = HashSet::new();
            let names: Vec<String> = bases.iter().map(|b| unique_name(b, ".md", &mut taken)).collect();
            let lower: HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
            prop_assert_eq!(lower.len(), names.len());
        }

        /// Oracle: following the relative path from `from` lands on `to`.
        #[test]
        fn prop_relative_path_resolves_to_target(from in gen_dir(), to_dir in gen_dir(), file in "[a-z]{1,4}\\.md") {
            let to = to_dir.join(file);
            prop_assert_eq!(resolve(&from, &relative_path(&from, &to)), to);
        }

        /// Metamorphic: rewriting with lookups that know nothing leaves the
        /// body as it was.
        #[test]
        fn prop_unknown_links_are_left_alone(body in "(\\PC|inkdrop://note/[a-z0-9]{0,5}|inkdrop://file:[a-z0-9]{0,5}|inkdrop://){0,30}") {
            prop_assert_eq!(rewrite_links(&body, |_| None, |_| None), body);
        }

        /// Round-trip: rewriting links to placeholders and back restores the body.
        #[test]
        fn prop_rewrite_round_trips(parts in prop::collection::vec((any::<bool>(), "[a-zA-Z0-9_-]{1,8}", "[ .)]{1,6}"), 0..10)) {
            let body: String = parts.iter().map(|(note, id, tail)| {
                format!("{}{id}{tail}", if *note { "inkdrop://note/" } else { "inkdrop://file:" })
            }).collect();
            let placeholder = rewrite_links(&body, |id| Some(format!("<<{id}>>")), |id| Some(format!("<<{id}>>")));
            prop_assert!(!placeholder.contains("inkdrop://"));
            let mut restored = placeholder.clone();
            for (note, id, _) in &parts {
                let (full, uri) = if *note {
                    (format!("<<note:{id}>>"), format!("inkdrop://note/{id}"))
                } else {
                    (format!("<<file:{id}>>"), format!("inkdrop://file:{id}"))
                };
                restored = restored.replacen(&full, &uri, 1);
            }
            prop_assert_eq!(restored, body);
        }

        /// Invariant: exporting into a folder that already has files keeps
        /// them byte for byte and adds one file per note.
        #[test]
        fn prop_export_never_replaces_files(
            books in prop::collection::vec("[aA_]{1,3}", 1..4),
            notes in prop::collection::vec((0usize..4, "[aA. ]{0,3}"), 0..8),
            existing in prop::collection::vec(("[aA_]{1,3}", "[aA. ]{0,3}"), 0..6),
        ) {
            let books: Vec<&str> = books.iter().map(String::as_str).collect();
            let notes: Vec<(usize, &str)> = notes.iter().map(|(b, t)| (b % books.len(), t.as_str())).collect();
            let (_dir, store) = library(&books, &notes);
            let out = tempfile::tempdir().unwrap();
            for (folder, file) in &existing {
                let dir = out.path().join(folder);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(format!("{}.md", sanitize_file_name(file))), b"mine").unwrap();
            }
            let before = read_tree(out.path());
            let report = export(&store, out.path(), None).unwrap();
            let after = read_tree(out.path());
            for (path, data) in &before {
                prop_assert_eq!(after.get(path), Some(data));
            }
            prop_assert_eq!(report.notes, notes.len());
            prop_assert_eq!(after.len(), before.len() + notes.len());
        }

        /// Invariant: an export's file name is safe on every platform and
        /// ends in its extension.
        #[test]
        fn prop_file_names_are_safe(title in "\\PC{0,80}") {
            let name = file_name(&title, "html");
            prop_assert!(name.ends_with(".html"));
            prop_assert!(!name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']));
        }

        /// Oracle: stepping from the epoch a day at a time gives the same
        /// calendar day as the closed formula.
        #[test]
        fn prop_format_date_matches_day_stepping(days in -800i64..80_000) {
            let (mut y, mut m, mut d) = (1970i64, 1i64, 1i64);
            let leap = |y: i64| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let length = |y: i64, m: i64| match m {
                2 => 28 + i64::from(leap(y)),
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            let step = if days >= 0 { 1 } else { -1 };
            for _ in 0..days.abs() {
                if step == 1 {
                    d += 1;
                    if d > length(y, m) { d = 1; m += 1; if m > 12 { m = 1; y += 1; } }
                } else {
                    d -= 1;
                    if d < 1 { m -= 1; if m < 1 { m = 12; y -= 1; } d = length(y, m); }
                }
            }
            prop_assert_eq!(format_date(days * 86_400_000 + 3_600_000), format!("{y:04}-{m:02}-{d:02}"));
        }

        /// Invariant: an encoded link has no character that ends a link target.
        #[test]
        fn prop_encoded_links_have_no_breaking_characters(path in "[ -~]{0,40}") {
            let encoded = encode_link(&path);
            prop_assert!(!encoded.contains([' ', '(', ')', '<', '>']));
        }
    }

    #[test]
    fn examples() {
        assert_eq!(sanitize_file_name("  a/b: c?  "), "a-b- c-");
        assert_eq!(sanitize_file_name("..."), "Untitled");
        assert_eq!(
            relative_path(Path::new("Work/Projects"), Path::new("Archive/x.md")),
            "../../Archive/x.md"
        );
        assert_eq!(
            rewrite_links(
                "see [it](inkdrop://note/abc) and ![](inkdrop://file:xyz)",
                |id| Some(format!("{id}.md")),
                |_| None
            ),
            "see [it](note:abc.md) and ![](inkdrop://file:xyz)"
        );
    }
}
