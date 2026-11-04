//! Folders of Markdown files: plain ones, Obsidian vaults and Logseq
//! graphs. Each `.md`, `.markdown` or `.txt` file is a note; its folder is
//! its notebook; other files become attachments when a note links to them.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use kanso_core::canvas::{Canvas, EXTENSION as CANVAS_EXTENSION};
use kanso_core::tags::inline_tag_names;

use crate::library::frontmatter;
use crate::library::links::{self, Kind};
use crate::library::resolve::is_note_path;
use crate::library::tree::{self, Tree, TreeFile, extension, file_name, parent_of, stem};
use crate::library::{Skipped, Source, SourceNote, dates};

/// The kind of folder, for what it skips and how it names notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    /// A title is the front matter's, else the first `# Heading`, else
    /// the file's name.
    Plain,
    /// A title is the front matter's, else the file's name, as Obsidian
    /// shows it. `.obsidian/` and `.trash/` are skipped.
    Obsidian,
    /// Pages and journals; `title::`, `tags::` and `alias::` properties;
    /// `a___b.md` is the page `a/b`. Logseq's own `logseq/` folder is skipped.
    Logseq,
}

/// Note files larger than this are skipped: a note holds at most about a
/// million characters.
const MAX_NOTE_FILE: u64 = 8 * 1024 * 1024;

pub struct MarkdownSource {
    tree: Tree,
    /// Stripped from the tree's paths (a zipped vault's top folder).
    prefix: String,
    name: String,
    flavor: Flavor,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    attachment_dirs: Vec<String>,
    /// For a single file: its key, and that it has no notebook.
    single_key: Option<String>,
}

impl MarkdownSource {
    /// A folder, or a zip of one.
    pub fn open(path: &Path, flavor: Flavor) -> anyhow::Result<MarkdownSource> {
        let is_zip = path.is_file()
            && extension(&path.file_name().unwrap_or_default().to_string_lossy()) == "zip";
        if path.is_file() && !is_zip {
            anyhow::ensure!(
                is_note_path(&path.file_name().unwrap_or_default().to_string_lossy()),
                "{} is not a folder or a Markdown file",
                path.display()
            );
            return MarkdownSource::single(path);
        }
        let mut tree = if is_zip {
            Tree::zip(path)?
        } else {
            Tree::dir(path)?
        };
        let mut name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| if is_zip { stem(n) } else { n })
            .unwrap_or("Imported")
            .to_owned();
        let mut listed = tree.files()?;
        let mut prefix = String::new();
        if is_zip {
            // A zipped folder: its one top folder is the root.
            let first = listed
                .first()
                .and_then(|f| f.path.split_once('/'))
                .map(|(top, _)| top.to_owned());
            if let Some(top) = first
                && listed
                    .iter()
                    .all(|f| f.path.starts_with(&format!("{top}/")))
            {
                for file in &mut listed {
                    file.path = file.path[top.len() + 1..].to_owned();
                }
                prefix = format!("{top}/");
                name = top;
            }
        }
        let mut source = MarkdownSource {
            tree,
            prefix,
            name,
            flavor,
            notes: Vec::new(),
            files: Vec::new(),
            skipped: Vec::new(),
            attachment_dirs: Vec::new(),
            single_key: None,
        };
        if !is_zip {
            source.note_hidden_folders(path);
            if flavor == Flavor::Obsidian {
                source.attachment_dirs = obsidian_attachment_dirs(path);
            }
        }
        source.classify(listed);
        Ok(source)
    }

    /// One Markdown file, with the files beside it that it links to.
    pub fn single(path: &Path) -> anyhow::Result<MarkdownSource> {
        let path = path
            .canonicalize()
            .with_context(|| format!("cannot open {}", path.display()))?;
        let dir = path.parent().context("the file has no folder")?.to_owned();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("the file name is not valid Unicode")?
            .to_owned();
        let meta = std::fs::metadata(&path)?;
        anyhow::ensure!(
            meta.len() <= MAX_NOTE_FILE,
            "{name} is too large for a note"
        );
        let mut tree = Tree::Single(path.clone());
        let note = tree
            .files()?
            .into_iter()
            .next()
            .context("the file can't be read")?;
        let text = tree.read_text(&name, MAX_NOTE_FILE)?;
        let files = linked_files_beside(&dir, &text);
        Ok(MarkdownSource {
            tree: Tree::Dir(dir),
            prefix: String::new(),
            name: String::new(),
            flavor: Flavor::Plain,
            notes: vec![note],
            files,
            skipped: Vec::new(),
            attachment_dirs: Vec::new(),
            single_key: Some(path.to_string_lossy().into_owned()),
        })
    }

    /// Reports the hidden folders the walk leaves out that matter.
    fn note_hidden_folders(&mut self, root: &Path) {
        let known: &[(&str, &str)] = match self.flavor {
            Flavor::Obsidian => &[
                (".obsidian", "Obsidian's settings and plugins"),
                (".trash", "Obsidian's trash"),
            ],
            _ => &[],
        };
        for (folder, reason) in known {
            if root.join(folder).is_dir() {
                self.skipped
                    .push(Skipped::new(format!("{folder}/"), *reason));
            }
        }
    }

    fn classify(&mut self, listed: Vec<TreeFile>) {
        let mut logseq_skipped = false;
        for file in listed {
            let ext = extension(&file.path);
            if self.flavor == Flavor::Logseq
                && (file.path.starts_with("logseq/") || file.path.starts_with("version-files/"))
            {
                if !logseq_skipped {
                    self.skipped.push(Skipped::new(
                        "logseq/",
                        "Logseq's settings, backups and caches",
                    ));
                    logseq_skipped = true;
                }
                continue;
            }
            if ext == CANVAS_EXTENSION || is_note_path(&file.path) {
                if file.size > MAX_NOTE_FILE {
                    self.skipped
                        .push(Skipped::new(file.path, "too large for a note"));
                } else {
                    self.notes.push(file);
                }
            } else {
                self.files.push(file);
            }
        }
    }
}

impl Source for MarkdownSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn files(&self) -> &[TreeFile] {
        &self.files
    }

    fn attachment_dirs(&self) -> Vec<String> {
        self.attachment_dirs.clone()
    }

    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let notes = self.notes.clone();
        for file in notes {
            let full_path = format!("{}{}", self.prefix, file.path);
            let read_path = if self.single_key.is_some() {
                file_name(&file.path).to_owned()
            } else {
                full_path
            };
            let text = match self.tree.read_text(&read_path, MAX_NOTE_FILE) {
                Ok(text) => text,
                Err(err) => anyhow::bail!("cannot read {}: {err}", file.path),
            };
            let mut note = if extension(&file.path) == CANVAS_EXTENSION {
                match canvas_note(&file, &text) {
                    Ok(note) => note,
                    Err(why) => {
                        // Both passes meet it; it is reported once.
                        if !self.skipped.iter().any(|s| s.path == file.path) {
                            self.skipped.push(Skipped::new(file.path.clone(), why));
                        }
                        continue;
                    }
                }
            } else {
                parse_note(&file, &text, self.flavor)
            };
            if let Some(key) = &self.single_key {
                note.key = key.clone();
                note.folder.clear();
            }
            each(note)?;
        }
        Ok(())
    }

    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        let full = format!("{}{path}", self.prefix);
        self.tree.read(&full, limit)
    }

    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// The note in a Markdown file. Pure apart from the file's metadata, which
/// is passed in.
pub fn parse_note(file: &TreeFile, text: &str, flavor: Flavor) -> SourceNote {
    let mut meta = frontmatter::Meta::default();
    let mut body = text;
    if let Some((front, rest)) = frontmatter::split(text) {
        meta = frontmatter::meta(front);
        body = rest;
    }
    let mut body = body.to_owned();
    if flavor == Flavor::Logseq {
        let (props, rest) = logseq_properties(&body);
        if meta.title.is_none() {
            meta.title = props.title;
        }
        meta.tags.extend(props.tags);
        meta.aliases.extend(props.aliases);
        body = rest;
    }
    let name = file_name(&file.path);
    let file_title = match flavor {
        Flavor::Logseq => logseq_title(&file.path),
        _ => stem(name).to_owned(),
    };
    let heading = first_heading(&body);
    let title = match (&meta.title, flavor, &heading) {
        (Some(title), _, _) => title.clone(),
        (None, Flavor::Plain, Some((heading, _))) => heading.clone(),
        _ => file_title.clone(),
    };
    // The heading is the title: don't show it twice.
    if let Some((heading, end)) = &heading
        && kanso_core::links::normalize_title(heading) == kanso_core::links::normalize_title(&title)
    {
        body = body[*end..].trim_start_matches('\n').to_owned();
    }
    let mut tags = meta.tags.clone();
    tags.extend(inline_tag_names(&body));
    let mut seen = BTreeSet::new();
    tags.retain(|t| seen.insert(t.to_lowercase()));

    let body = body.trim_start_matches('\n');
    let body = if meta.rest.is_empty() {
        body.to_owned()
    } else {
        format!("---\n{}---\n\n{body}", meta.rest)
    };
    let folder = parent_of(&file.path)
        .split('/')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    SourceNote {
        key: file.path.clone(),
        path: Some(file.path.clone()),
        folder,
        title,
        body,
        tags,
        aliases: meta.aliases,
        created: meta
            .created
            .as_deref()
            .and_then(dates::parse)
            .or(file.created)
            .or(file.modified),
        updated: meta
            .updated
            .as_deref()
            .and_then(dates::parse)
            .or(file.modified),
        status: meta.status,
        ..SourceNote::default()
    }
}

/// The canvas in a canvas file: a note of the file's name whose body is the
/// file, for the import to point at what it made. Why it can't be one, if it
/// can't.
pub fn canvas_note(file: &TreeFile, text: &str) -> Result<SourceNote, String> {
    Canvas::from_obsidian(text).map_err(|why| format!("Not a canvas Kanso can open: {why}"))?;
    let folder = parent_of(&file.path)
        .split('/')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(SourceNote {
        key: file.path.clone(),
        path: Some(file.path.clone()),
        folder,
        title: stem(file_name(&file.path)).to_owned(),
        body: text.to_owned(),
        created: file.created.or(file.modified),
        updated: file.modified,
        canvas: true,
        ..SourceNote::default()
    })
}

/// The first non-blank line when it is a `# Heading`: its text and the
/// byte where the line ends.
fn first_heading(body: &str) -> Option<(String, usize)> {
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let content = line.trim_end_matches('\n');
        if content.trim().is_empty() {
            offset += line.len();
            continue;
        }
        let rest = content.strip_prefix("# ")?;
        let text = rest.trim().trim_end_matches('#').trim();
        return (!text.is_empty()).then(|| (text.to_owned(), offset + line.len()));
    }
    None
}

/// Logseq's page properties: the leading `key:: value` lines.
#[derive(Default)]
struct LogseqProps {
    title: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
}

fn logseq_properties(body: &str) -> (LogseqProps, String) {
    let mut props = LogseqProps::default();
    let mut kept = Vec::new();
    let mut lines = body.split_inclusive('\n');
    let mut rest_start = 0;
    for line in lines.by_ref() {
        let Some((key, value)) = line.trim_end().split_once(":: ") else {
            break;
        };
        if key.is_empty() || key.contains(' ') {
            break;
        }
        rest_start += line.len();
        let list = || {
            value
                .split(',')
                .map(|v| {
                    v.trim()
                        .trim_start_matches('#')
                        .trim_matches(['[', ']'])
                        .trim()
                })
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        match key.to_lowercase().as_str() {
            "title" => props.title = Some(value.trim().to_owned()),
            "tags" => props.tags = list(),
            "alias" => props.aliases = list(),
            _ => kept.push(line),
        }
    }
    let mut out: String = kept.concat();
    out.push_str(&body[rest_start..]);
    (props, out)
}

/// A Logseq page's title from its file: `a___b.md` and `a%2Fb.md` are
/// `a/b`; a journal `2024_03_01.md` is `2024-03-01`.
pub fn logseq_title(path: &str) -> String {
    let name = stem(file_name(path));
    if path.starts_with("journals/")
        && name.len() == 10
        && name.as_bytes()[4] == b'_'
        && name.as_bytes()[7] == b'_'
    {
        return name.replace('_', "-");
    }
    let name = name.replace("___", "/");
    percent_encoding::percent_decode_str(&name)
        .decode_utf8()
        .map(|s| s.into_owned())
        .unwrap_or(name)
}

/// Obsidian's attachment folder (`.obsidian/app.json`'s
/// `attachmentFolderPath`): `/` is the vault's root, `./` the note's folder.
fn obsidian_attachment_dirs(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join(".obsidian/app.json")) else {
        return Vec::new();
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    match config
        .get("attachmentFolderPath")
        .and_then(|v| v.as_str())
        .map(str::trim)
    {
        Some("/") | Some("") | None => Vec::new(),
        Some(dir) => vec![dir.trim_end_matches('/').to_owned()],
    }
}

/// The files beside a single note that its links point at.
fn linked_files_beside(dir: &Path, text: &str) -> Vec<TreeFile> {
    let mut seen = BTreeSet::new();
    let mut files = Vec::new();
    for found in links::scan(text) {
        let path = match found.kind {
            Kind::Wiki | Kind::Embed => Some(found.target.clone()),
            _ => links::as_relative_path(&found.target),
        };
        let Some(path) = path.and_then(|p| tree::join("", &p)) else {
            continue;
        };
        if is_note_path(&path) || !seen.insert(path.clone()) {
            continue;
        }
        let full: PathBuf = dir.join(&path);
        let Ok(meta) = std::fs::metadata(&full) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let ms = |t: io::Result<std::time::SystemTime>| {
            t.ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| i64::try_from(d.as_millis()).ok())
        };
        files.push(TreeFile {
            path,
            size: meta.len(),
            modified: ms(meta.modified()),
            created: ms(meta.created()),
        });
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> TreeFile {
        TreeFile {
            path: path.to_owned(),
            size: 0,
            modified: Some(1_000),
            created: None,
        }
    }

    #[test]
    fn plain_titles_come_from_the_heading() {
        let note = parse_note(
            &file("Work/plan.md"),
            "\n# The Plan\n\nBody #idea\n",
            Flavor::Plain,
        );
        assert_eq!(note.title, "The Plan");
        assert_eq!(note.body, "Body #idea\n");
        assert_eq!(note.tags, ["idea"]);
        assert_eq!(note.folder, ["Work"]);
        assert_eq!(note.updated, Some(1_000));
        assert_eq!(note.created, Some(1_000));
    }

    #[test]
    fn obsidian_titles_are_file_names() {
        let note = parse_note(&file("plan.md"), "# Something else\nBody", Flavor::Obsidian);
        assert_eq!(note.title, "plan");
        assert_eq!(note.body, "# Something else\nBody");
        let note = parse_note(&file("Plan.md"), "# Plan\nBody", Flavor::Obsidian);
        assert_eq!(note.body, "Body");
    }

    #[test]
    fn front_matter_wins_and_the_rest_stays() {
        let text = "---\ntitle: Real title\ntags: [a, b/c]\ncreated: 2024-03-01T09:30:00Z\ncssclass: wide\n---\n# Heading\nBody";
        let note = parse_note(&file("x.md"), text, Flavor::Plain);
        assert_eq!(note.title, "Real title");
        assert_eq!(note.tags, ["a", "b/c"]);
        assert_eq!(note.created, Some(1_709_285_400_000));
        assert_eq!(note.body, "---\ncssclass: wide\n---\n\n# Heading\nBody");
    }

    #[test]
    fn logseq_pages() {
        let note = parse_note(
            &file("pages/proj___acme.md"),
            "tags:: work, [[clients]]\nalias:: ACME\nicon:: x\n- first block\n",
            Flavor::Logseq,
        );
        assert_eq!(note.title, "proj/acme");
        assert_eq!(note.tags, ["work", "clients"]);
        assert_eq!(note.aliases, ["ACME"]);
        assert_eq!(note.body, "icon:: x\n- first block\n");
        assert_eq!(logseq_title("journals/2024_03_01.md"), "2024-03-01");
        assert_eq!(logseq_title("pages/a%2Fb.md"), "a/b");
    }
}
