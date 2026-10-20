//! Joplin's exports: a `.jex` file (a tar) or a folder exported as RAW,
//! which hold the same thing. Every item is a `<id>.md` file: its title, a
//! blank line, its body, a blank line, then `key: value` metadata lines.
//! The metadata's `type_` says what the item is: a note (1), a notebook
//! (2), an attachment's description (4, its bytes in `resources/`), a tag
//! (5) or a note's tag (6).
//!
//! A `.jex` is unpacked into a temporary folder that lives as long as the
//! source, so attachments are read from disk one at a time.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::Context as _;

use crate::library::links::{self, Kind};
use crate::library::tree::{Tree, TreeFile, clean_zip_path, extension, file_name, stem};
use crate::library::{Skipped, Source, SourceNote, dates};

/// Item files larger than this are skipped: a note holds at most about a
/// million characters.
const MAX_ITEM_FILE: u64 = 8 * 1024 * 1024;

/// Joplin's item types.
const TYPE_NOTE: &str = "1";
const TYPE_FOLDER: &str = "2";
const TYPE_RESOURCE: &str = "4";
const TYPE_TAG: &str = "5";
const TYPE_NOTE_TAG: &str = "6";

/// One Joplin item as its file holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    /// The first line.
    pub title: String,
    /// What comes between the title's blank line and the metadata's.
    pub body: String,
    /// The metadata, in order, as written (see [`unescape`]).
    pub props: Vec<(String, String)>,
}

impl Item {
    /// The raw value of `key`, if the item has it.
    pub fn raw(&self, key: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The value of `key` unescaped, `None` when missing or empty.
    pub fn get(&self, key: &str) -> Option<String> {
        self.raw(key).filter(|v| !v.is_empty()).map(unescape)
    }

    /// Whether `key` is set to something other than `0` or nothing.
    pub fn flag(&self, key: &str) -> bool {
        self.raw(key).is_some_and(|v| !v.is_empty() && v != "0")
    }

    /// A time from `user_key`, else `key`, in milliseconds.
    fn time(&self, user_key: &str, key: &str) -> Option<i64> {
        [user_key, key]
            .into_iter()
            .filter_map(|k| self.raw(k))
            .filter_map(dates::parse)
            .find(|&ms| ms > 0)
    }
}

/// Reads an item's text (with `\n` line ends) as Joplin does: the metadata
/// is the last run of non-blank lines, which must hold `type_`; before it
/// and a blank line come the title, a blank line, and the body. `None` when
/// the text is not an item.
pub fn parse_item(text: &str) -> Option<Item> {
    let lines: Vec<&str> = text.split('\n').collect();
    let blank = |line: &str| line.trim().is_empty();
    let mut end = lines.len();
    while end > 0 && blank(lines[end - 1]) {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && !blank(lines[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let mut props = Vec::with_capacity(end - start);
    for line in &lines[start..end] {
        let (key, value) = line.trim().split_once(':')?;
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        props.push((key.to_owned(), value.trim().to_owned()));
    }
    if !props.iter().any(|(k, _)| k == "type_") {
        return None;
    }
    // Everything before the blank line that precedes the metadata.
    let content = &lines[..start.saturating_sub(1)];
    let title = content.first().copied().unwrap_or_default().to_owned();
    // Joplin writes a blank line after the title; keep a line that isn't.
    let from = match content.get(1) {
        Some(line) if blank(line) => 2,
        _ => 1,
    };
    let body = content.get(from..).unwrap_or_default().join("\n");
    Some(Item { title, body, props })
}

/// An item's text as Joplin writes it; [`parse_item`] reads it back.
pub fn render_item(item: &Item) -> String {
    let mut parts = vec![item.title.clone()];
    if !item.body.is_empty() {
        parts.push(item.body.clone());
    }
    let props: Vec<String> = item
        .props
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();
    parts.push(props.join("\n"));
    parts.join("\n\n")
}

/// A metadata value with Joplin's `\n` and `\r` escapes undone.
pub fn unescape(value: &str) -> String {
    if value.contains('\\') {
        value.replace("\\n", "\n").replace("\\r", "\r")
    } else {
        value.to_owned()
    }
}

/// The item ID a Joplin link (`:/<32 hex digits>`, maybe with `#anchor`)
/// points at.
pub fn link_id(target: &str) -> Option<&str> {
    let rest = target.trim().strip_prefix(":/")?;
    let id = rest.split_once('#').map_or(rest, |(id, _)| id);
    (id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())).then_some(id)
}

/// `body` with each Joplin link (`[a](:/id)`, `![](:/id)`, `[a]: :/id`,
/// `<img src=":/id">`) whose ID `target` knows pointed at what it returns.
/// Everything else, other links included, is left byte for byte.
pub fn rewrite_links(body: &str, mut target: impl FnMut(&str) -> Option<String>) -> String {
    if !body.contains(":/") {
        return body.to_owned();
    }
    let mut edits = Vec::new();
    for found in links::scan(body) {
        if matches!(found.kind, Kind::Wiki | Kind::Embed) {
            continue;
        }
        if let Some(uri) = link_id(&found.target).and_then(&mut target) {
            edits.push((found.range, uri));
        }
    }
    if edits.is_empty() {
        body.to_owned()
    } else {
        links::apply(body, edits)
    }
}

/// A note item's file, read again on every pass.
struct NoteFile {
    path: String,
}

pub struct JoplinSource {
    name: String,
    tree: Tree,
    /// Where a `.jex` was unpacked; removed when the source is dropped.
    _unpacked: Option<tempfile::TempDir>,
    notes: Vec<NoteFile>,
    /// Notebook ID → its names from the top down.
    folders: HashMap<String, Vec<String>>,
    /// Note ID → its tag names.
    tags: HashMap<String, Vec<String>>,
    note_ids: HashSet<String>,
    /// Resource ID → its file's tree path.
    resources: HashMap<String, String>,
    /// A resource file's tree path → the attachment's name in Joplin.
    names: HashMap<String, String>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
}

/// Opens a `.jex` file or a RAW export's folder.
pub fn open(path: &Path) -> anyhow::Result<JoplinSource> {
    // `Path::file_name`, not the tree's `/`-only helper: on Windows an OS
    // path separates with `\`.
    let base = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = if path.is_dir() {
        base
    } else {
        stem(&base).to_owned()
    };
    let (tree, unpacked) = if path.is_dir() {
        (Tree::dir(path)?, None)
    } else {
        let dir = tempfile::tempdir().context("cannot make a temporary folder")?;
        unpack(path, dir.path())?;
        (Tree::dir(dir.path())?, Some(dir))
    };
    JoplinSource::new(name, tree, unpacked)
}

/// Unpacks a `.jex`'s items and resources into `to`. Only regular files at
/// the top or in `resources/` are written; nothing escapes `to`.
fn unpack(jex: &Path, to: &Path) -> anyhow::Result<()> {
    let file = File::open(jex).with_context(|| format!("cannot open {}", jex.display()))?;
    let mut archive = tar::Archive::new(file);
    let entries = archive
        .entries()
        .with_context(|| format!("{} is not a Joplin export (.jex)", jex.display()))?;
    let mut any = false;
    std::fs::create_dir_all(to.join("resources"))?;
    for entry in entries {
        let mut entry =
            entry.with_context(|| format!("{} is not a Joplin export (.jex)", jex.display()))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let Some(path) = entry
            .path()
            .ok()
            .and_then(|p| p.to_str().and_then(clean_zip_path))
        else {
            continue;
        };
        let wanted = match path.split_once('/') {
            None => extension(&path) == "md",
            Some(("resources", rest)) => !rest.contains('/'),
            Some(_) => false,
        };
        if !wanted {
            continue;
        }
        let mut out = File::create(to.join(&path))
            .with_context(|| format!("cannot unpack {path} from {}", jex.display()))?;
        io::copy(&mut entry, &mut out)
            .with_context(|| format!("cannot unpack {path} from {}", jex.display()))?;
        any = true;
    }
    anyhow::ensure!(
        any,
        "{} holds no Joplin items; is it a .jex export?",
        jex.display()
    );
    Ok(())
}

/// A notebook as its item describes it.
struct Folder {
    title: String,
    parent: Option<String>,
}

impl JoplinSource {
    fn new(
        name: String,
        mut tree: Tree,
        unpacked: Option<tempfile::TempDir>,
    ) -> anyhow::Result<JoplinSource> {
        let listed = tree.files()?;
        let mut skipped = Vec::new();
        let mut notes = Vec::new();
        let mut note_ids = HashSet::new();
        let mut folders: HashMap<String, Folder> = HashMap::new();
        let mut tag_names: HashMap<String, String> = HashMap::new();
        let mut note_tags: Vec<(String, String)> = Vec::new();
        // Resource ID → its created time.
        let mut resource_items: HashMap<String, (Option<i64>, Option<String>)> = HashMap::new();
        let mut other_types: BTreeMap<String, usize> = BTreeMap::new();
        let mut resource_files = Vec::new();
        for file in listed {
            if file.path.starts_with("resources/") {
                resource_files.push(file);
                continue;
            }
            if file.path.contains('/') || extension(&file.path) != "md" {
                skipped.push(Skipped::new(file.path, "not a Joplin item"));
                continue;
            }
            if file.size > MAX_ITEM_FILE {
                skipped.push(Skipped::new(file.path, "too large for a note"));
                continue;
            }
            let text = tree
                .read_text(&file.path, MAX_ITEM_FILE)
                .with_context(|| format!("cannot read {}", file.path))?;
            let Some(item) = parse_item(&text) else {
                skipped.push(Skipped::new(file.path, "not a Joplin item"));
                continue;
            };
            let id = item
                .get("id")
                .unwrap_or_else(|| stem(&file.path).to_owned());
            if item.flag("encryption_applied") {
                skipped.push(Skipped::new(
                    file.path,
                    "encrypted; turn off encryption in Joplin and export again",
                ));
                continue;
            }
            let kind = item.raw("type_").unwrap_or_default().to_owned();
            match kind.as_str() {
                TYPE_NOTE => {
                    if note_ids.insert(id) {
                        notes.push(NoteFile { path: file.path });
                    }
                }
                TYPE_FOLDER => {
                    let parent = item.get("parent_id");
                    folders.insert(
                        id,
                        Folder {
                            title: item.title,
                            parent,
                        },
                    );
                }
                TYPE_RESOURCE => {
                    if item.flag("encryption_blob_encrypted") {
                        skipped.push(Skipped::new(
                            file.path,
                            "an encrypted attachment; turn off encryption in Joplin and export again",
                        ));
                    } else {
                        let title = Some(item.title.trim().to_owned()).filter(|t| !t.is_empty());
                        resource_items
                            .insert(id, (item.time("user_created_time", "created_time"), title));
                    }
                }
                TYPE_TAG => {
                    tag_names.insert(id, item.title);
                }
                TYPE_NOTE_TAG => {
                    if let (Some(note), Some(tag)) = (item.get("note_id"), item.get("tag_id")) {
                        note_tags.push((note, tag));
                    }
                }
                other => *other_types.entry(other.to_owned()).or_default() += 1,
            }
        }
        for (kind, count) in other_types {
            skipped.push(Skipped::new(
                format!("{count} Joplin item(s) of type {kind}"),
                "not notes, notebooks, tags or attachments",
            ));
        }

        let mut resources = HashMap::new();
        let mut names = HashMap::new();
        let mut files = Vec::new();
        for mut file in resource_files {
            let id = stem(file_name(&file.path)).to_owned();
            match resource_items.get(&id) {
                Some((created, title)) => {
                    if let Some(created) = created {
                        file.created = Some(*created);
                    }
                    if let Some(title) = title {
                        names.insert(file.path.clone(), title.clone());
                    }
                    resources.insert(id, file.path.clone());
                    files.push(file);
                }
                None => skipped.push(Skipped::new(
                    file.path,
                    "no Joplin item describes this attachment",
                )),
            }
        }

        let folder_paths = folder_paths(&folders);
        let mut tags: HashMap<String, Vec<String>> = HashMap::new();
        for (note, tag) in note_tags {
            if let Some(name) = tag_names.get(&tag) {
                let names = tags.entry(note).or_default();
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        Ok(JoplinSource {
            name,
            tree,
            _unpacked: unpacked,
            notes,
            folders: folder_paths,
            tags,
            note_ids,
            resources,
            names,
            files,
            skipped,
        })
    }

    fn note(&self, item: &Item, full: bool) -> SourceNote {
        let key = item.get("id").unwrap_or_default();
        let folder = item
            .get("parent_id")
            .and_then(|p| self.folders.get(&p))
            .cloned()
            .unwrap_or_default();
        let mut note = SourceNote {
            key,
            folder,
            title: item.title.clone(),
            ..SourceNote::default()
        };
        if full {
            note.body = rewrite_links(&item.body, |id| {
                if self.note_ids.contains(id) {
                    Some(format!("kanso-import:note/{id}"))
                } else {
                    self.resources
                        .get(id)
                        .map(|path| format!("kanso-import:path/{path}"))
                }
            });
            note.tags = self.tags.get(&note.key).cloned().unwrap_or_default();
            note.created = item.time("user_created_time", "created_time");
            note.updated = item.time("user_updated_time", "updated_time");
            note.trashed = item.flag("deleted_time");
        }
        note
    }
}

/// Every notebook's names from the top down, following `parent_id`. A
/// missing parent makes a notebook a top one; a loop is cut where it closes.
fn folder_paths(folders: &HashMap<String, Folder>) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for id in folders.keys() {
        let mut names = Vec::new();
        let mut seen = HashSet::new();
        let mut at = Some(id);
        while let Some(current) = at {
            if !seen.insert(current) {
                break;
            }
            let Some(folder) = folders.get(current) else {
                break;
            };
            names.push(folder.title.clone());
            at = folder.parent.as_ref();
        }
        names.reverse();
        out.insert(id.clone(), names);
    }
    out
}

impl Source for JoplinSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn files(&self) -> &[TreeFile] {
        &self.files
    }

    fn attachment_name(&self, path: &str) -> Option<String> {
        self.names.get(path).cloned()
    }

    fn notes(
        &mut self,
        full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for ix in 0..self.notes.len() {
            let path = &self.notes[ix].path;
            let text = self
                .tree
                .read_text(path, MAX_ITEM_FILE)
                .with_context(|| format!("cannot read {path}"))?;
            let item =
                parse_item(&text).with_context(|| format!("{path} changed while importing"))?;
            each(self.note(&item, full))?;
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
    use super::*;

    const ID: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn reads_a_note_item() {
        let text = format!(
            "Title\n\nBody line\nkey: looks like metadata\n\nid: {ID}\nparent_id: \ntype_: 1"
        );
        let item = parse_item(&text).unwrap();
        assert_eq!(item.title, "Title");
        assert_eq!(item.body, "Body line\nkey: looks like metadata");
        assert_eq!(item.get("id").as_deref(), Some(ID));
        assert_eq!(item.get("parent_id"), None);
        assert_eq!(render_item(&item), text);
    }

    #[test]
    fn needs_a_type() {
        assert_eq!(parse_item("Title\n\nid: x"), None);
        assert_eq!(parse_item("just text"), None);
        assert_eq!(parse_item(""), None);
    }

    #[test]
    fn link_ids() {
        assert_eq!(link_id(&format!(":/{ID}")), Some(ID));
        assert_eq!(link_id(&format!(":/{ID}#heading")), Some(ID));
        assert_eq!(link_id(":/123"), None);
        assert_eq!(link_id("https://example.com"), None);
    }

    #[test]
    fn rewrites_known_links_only() {
        let body = format!("[a](:/{ID}) ![b](:/{}) `[c](:/{ID})`", "f".repeat(32));
        let out = rewrite_links(&body, |id| (id == ID).then(|| "x".to_owned()));
        assert_eq!(
            out,
            format!("[a](x) ![b](:/{}) `[c](:/{ID})`", "f".repeat(32))
        );
    }
}
