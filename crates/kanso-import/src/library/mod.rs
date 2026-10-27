//! Imports other note apps' exports and folders of Markdown files.
//!
//! Every source becomes a stream of [`SourceNote`]s, read twice: once to
//! learn every note's title and path, so links between notes can be pointed
//! at what the import makes, and once to write them. Nothing is kept in
//! memory but that index; notes and attachments are read one at a time.
//!
//! Documents get IDs derived from the source kind, the source's name and a
//! stable key (a vault-relative path, or the other app's own ID), so
//! importing again finds what the last import made. Each imported document
//! carries a `kansoImport` field recording the source key, a digest of what
//! was imported and the revision generation the import wrote. On a later
//! import a document is:
//!
//! - left alone when the digest is the same (nothing changed in the source);
//! - updated when the source changed and the document is still at the
//!   generation the import wrote (nobody edited it in Kanso);
//! - kept when it was edited in Kanso since (its generation is higher), or
//!   deleted there. An import never reverts a change made in Kanso.

pub mod apple_journal;
pub mod apple_notes;
pub mod bear;
pub mod dates;
pub mod detect;
pub mod enex;
pub mod frontmatter;
pub mod google_keep;
pub mod html;
pub mod joplin;
pub mod jsonexports;
pub mod links;
pub mod markdown;
pub mod notion;
pub mod open_files;
mod report;
pub mod resolve;
pub mod roam;
pub mod textbundle;
pub mod tomboy;
pub mod tree;
pub mod words;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::io;
use std::path::Path;

use anyhow::Context as _;
use base64::Engine as _;
use kanso_core::links::{FILE_LINK_PREFIX, NOTE_LINK_PREFIX};
use kanso_core::model::{MAX_ATTACHMENT_SIZE, MAX_BODY_LEN, MAX_NAME_LEN, MAX_TITLE_LEN};
use kanso_core::roles::Role;
use kanso_core::{DocType, Rev, TRASH_BOOK_ID};
use kanso_store::{Attachment, Origin, Store, StoreError};
use serde_json::{Map, Value, json};
use sha1::{Digest as _, Sha1};

pub use detect::kinds_for;
pub use open_files::{open_note_files, staged_path, staging_folder_name};
pub use report::{Counts, LibraryReport, Outcomes, Skipped};

use crate::library::links::Kind;
use crate::library::resolve::{Index, IndexedNote, Target};
use crate::library::tree::{TreeFile, extension, file_name, stem};

/// The field on imported documents that records where they came from.
pub const IMPORT_FIELD: &str = "kansoImport";

/// The kinds of source Kanso imports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Markdown,
    Obsidian,
    Logseq,
    Notion,
    Bear,
    Evernote,
    Joplin,
    StandardNotes,
    Simplenote,
    UpNote,
    AppleNotes,
    AppleJournal,
    GoogleKeep,
    Html,
    Roam,
    TextBundle,
    Tomboy,
}

impl SourceKind {
    pub const ALL: [SourceKind; 17] = [
        SourceKind::Markdown,
        SourceKind::Obsidian,
        SourceKind::Notion,
        SourceKind::Evernote,
        SourceKind::Bear,
        SourceKind::Joplin,
        SourceKind::Logseq,
        SourceKind::StandardNotes,
        SourceKind::Simplenote,
        SourceKind::UpNote,
        SourceKind::AppleNotes,
        SourceKind::AppleJournal,
        SourceKind::GoogleKeep,
        SourceKind::Html,
        SourceKind::Roam,
        SourceKind::TextBundle,
        SourceKind::Tomboy,
    ];

    /// The name the command line takes, and the one recorded on documents.
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Markdown => "markdown",
            SourceKind::Obsidian => "obsidian",
            SourceKind::Logseq => "logseq",
            SourceKind::Notion => "notion",
            SourceKind::Bear => "bear",
            SourceKind::Evernote => "evernote",
            SourceKind::Joplin => "joplin",
            SourceKind::StandardNotes => "standard-notes",
            SourceKind::Simplenote => "simplenote",
            SourceKind::UpNote => "upnote",
            SourceKind::AppleNotes => "apple-notes",
            SourceKind::AppleJournal => "apple-journal",
            SourceKind::GoogleKeep => "google-keep",
            SourceKind::Html => "html",
            SourceKind::Roam => "roam",
            SourceKind::TextBundle => "textbundle",
            SourceKind::Tomboy => "tomboy",
        }
    }

    pub fn parse(name: &str) -> Option<SourceKind> {
        let name = name.to_ascii_lowercase();
        let name = match name.as_str() {
            "md" | "folder" => "markdown",
            "enex" => "evernote",
            "standardnotes" | "sn" => "standard-notes",
            "apple notes" | "applenotes" => "apple-notes",
            "apple journal" | "applejournal" => "apple-journal",
            "google keep" | "googlekeep" | "keep" => "google-keep",
            "roam json" | "roam-json" => "roam",
            "textbundle" | "text pack" | "textpack" => "textbundle",
            "gnote" => "tomboy",
            other => other,
        };
        SourceKind::ALL.into_iter().find(|k| k.as_str() == name)
    }

    /// The name shown in the app.
    pub fn label(self) -> &'static str {
        match self {
            SourceKind::Markdown => "Markdown folder",
            SourceKind::Obsidian => "Obsidian vault",
            SourceKind::Logseq => "Logseq graph",
            SourceKind::Notion => "Notion export",
            SourceKind::Bear => "Bear backup",
            SourceKind::Evernote => "Evernote export",
            SourceKind::Joplin => "Joplin export",
            SourceKind::StandardNotes => "Standard Notes backup",
            SourceKind::Simplenote => "Simplenote export",
            SourceKind::UpNote => "UpNote export",
            SourceKind::AppleNotes => "Apple Notes",
            SourceKind::AppleJournal => "Apple Journal",
            SourceKind::GoogleKeep => "Google Keep",
            SourceKind::Html => "HTML files",
            SourceKind::Roam => "Roam export",
            SourceKind::TextBundle => "TextBundle/TextPack",
            SourceKind::Tomboy => "Tomboy/Gnote",
        }
    }

    /// What to pick, in a sentence for the app.
    pub fn hint(self) -> &'static str {
        match self {
            SourceKind::Markdown => {
                "A folder of .md, .markdown or .txt files. Subfolders become notebooks."
            }
            SourceKind::Obsidian => {
                "The vault's folder. Wiki-links, embeds, tags and attachments come along; .obsidian settings and canvases don't."
            }
            SourceKind::Logseq => "The graph's folder, with pages/ and journals/.",
            SourceKind::Notion => {
                "The .zip from Settings › Export (Markdown & CSV), or its unzipped folder."
            }
            SourceKind::Bear => {
                "A .bear2bk backup, or a folder from Export Notes (TextBundle or Markdown)."
            }
            SourceKind::Evernote => "An .enex file, or a folder of them (one per notebook).",
            SourceKind::Joplin => "A .jex file, or a folder exported as RAW.",
            SourceKind::StandardNotes => {
                "A decrypted backup: the .txt or .json file, or the .zip it came in."
            }
            SourceKind::Simplenote => "The .zip from Export Notes, or the notes.json in it.",
            SourceKind::UpNote => "A folder from Export › Markdown.",
            SourceKind::AppleNotes => {
                "Kanso finds the Notes library on this Mac. macOS may ask you to allow read-only access; locked or undownloaded notes are listed."
            }
            SourceKind::AppleJournal => {
                "Journal entry .html files, or the folder Apple Journal exports them to."
            }
            SourceKind::GoogleKeep => {
                "A Keep .json file, or the Google Takeout .zip/folder with its notes and attachments."
            }
            SourceKind::Html => {
                "An .html/.htm file, a folder of pages, or a .zip. Local links and files come along."
            }
            SourceKind::Roam => "A Roam graph export in JSON.",
            SourceKind::TextBundle => {
                "A .textbundle folder, a .textpack, or a .zip holding one or more bundles."
            }
            SourceKind::Tomboy => "A .note file, or a Tomboy/Gnote notes folder.",
        }
    }

    /// Whether the source is usually a folder (else a file).
    pub fn picks_folder(self) -> bool {
        matches!(
            self,
            SourceKind::Markdown
                | SourceKind::Obsidian
                | SourceKind::Logseq
                | SourceKind::UpNote
                | SourceKind::AppleJournal
                | SourceKind::GoogleKeep
                | SourceKind::Html
                | SourceKind::TextBundle
                | SourceKind::Tomboy
        )
    }

    /// File extensions the app's file picker offers for this source.
    pub fn file_extensions(self) -> &'static [&'static str] {
        match self {
            SourceKind::Notion => &["zip"],
            SourceKind::AppleNotes => &["sqlite"],
            SourceKind::AppleJournal => &["html", "htm", "zip"],
            SourceKind::GoogleKeep => &["json", "zip"],
            SourceKind::Html => &["html", "htm", "zip"],
            SourceKind::Roam => &["json"],
            SourceKind::TextBundle => &["textbundle", "textpack", "zip"],
            SourceKind::Tomboy => &["note"],
            SourceKind::Bear => &["bear2bk", "zip"],
            SourceKind::Evernote => &["enex"],
            SourceKind::Joplin => &["jex"],
            SourceKind::StandardNotes => &["txt", "json", "zip"],
            SourceKind::Simplenote => &["zip", "json"],
            _ => &["md", "markdown", "txt"],
        }
    }
}

/// A note as a source gives it, before links are pointed at Kanso.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceNote {
    /// Stable within the source: what makes importing again find it.
    pub key: String,
    /// The note's file in the source's tree, for its relative links.
    pub path: Option<String>,
    /// Notebook names from the source's root down to the note's notebook.
    pub folder: Vec<String>,
    pub title: String,
    /// Markdown. Links may use `kanso-import:note/<key>`,
    /// `kanso-import:path/<tree path>` and `kanso-import:embedded/<key>`.
    pub body: String,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub created: Option<i64>,
    pub updated: Option<i64>,
    pub pinned: bool,
    /// The note's status, when the source names one Kanso knows.
    pub status: Option<kanso_core::NoteStatus>,
    pub trashed: bool,
    /// Attachments carried in the note itself (Evernote's resources).
    pub embedded: Vec<Embedded>,
}

/// An attachment that comes inside a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Embedded {
    pub key: String,
    pub name: String,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// A source of notes: a format reading a [`tree::Tree`] or a file.
pub trait Source {
    /// The top notebook's name: the vault's or the export's.
    fn name(&self) -> String;

    /// Every non-note file that notes may link to.
    fn files(&self) -> &[TreeFile];

    /// The name an attachment had in the app, when it isn't its file's.
    fn attachment_name(&self, _path: &str) -> Option<String> {
        None
    }

    /// Folders searched for attachments after the note's own (see
    /// [`resolve::Index::new`]).
    fn attachment_dirs(&self) -> Vec<String> {
        Vec::new()
    }

    /// Calls `each` with every note, in the same order every time. With
    /// `full` false only keys, paths, folders, titles and aliases are needed.
    fn notes(
        &mut self,
        full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;

    /// The bytes of a file from [`Source::files`].
    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>>;

    /// What the source left out and why, known once `notes` has run.
    fn skipped(&mut self) -> Vec<Skipped>;
}

/// Opens `path` as a source of `kind`.
pub fn open(kind: SourceKind, path: &Path) -> anyhow::Result<Box<dyn Source>> {
    anyhow::ensure!(path.exists(), "{} doesn't exist", path.display());
    Ok(match kind {
        SourceKind::Markdown | SourceKind::UpNote => Box::new(markdown::MarkdownSource::open(
            path,
            markdown::Flavor::Plain,
        )?),
        SourceKind::Obsidian => Box::new(markdown::MarkdownSource::open(
            path,
            markdown::Flavor::Obsidian,
        )?),
        SourceKind::Logseq => Box::new(markdown::MarkdownSource::open(
            path,
            markdown::Flavor::Logseq,
        )?),
        SourceKind::Notion => Box::new(notion::open(path)?),
        SourceKind::Bear => Box::new(bear::open(path)?),
        SourceKind::Evernote => Box::new(enex::open(path)?),
        SourceKind::Joplin => Box::new(joplin::open(path)?),
        SourceKind::StandardNotes => Box::new(jsonexports::standard_notes(path)?),
        SourceKind::Simplenote => Box::new(jsonexports::simplenote(path)?),
        SourceKind::AppleNotes => Box::new(apple_notes::open(path)?),
        SourceKind::AppleJournal => Box::new(apple_journal::open(path)?),
        SourceKind::GoogleKeep => Box::new(google_keep::open(path)?),
        SourceKind::Html => Box::new(html::open(path)?),
        SourceKind::Roam => Box::new(roam::open(path)?),
        SourceKind::TextBundle => Box::new(textbundle::open(path)?),
        SourceKind::Tomboy => Box::new(tomboy::open(path)?),
    })
}

/// The stage an import is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryPhase {
    /// Reading every note once to learn titles and paths.
    Reading,
    Notebooks,
    Notes,
}

/// Progress through one phase: `done` of `total` (0 when not known yet).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryProgress {
    pub phase: LibraryPhase,
    pub done: usize,
    pub total: usize,
}

/// How [`run`] runs.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Only work out what would happen; write nothing.
    pub dry_run: bool,
    /// Put every note in the Inbox instead of notebooks of its own (for
    /// opening a single file).
    pub into_inbox: bool,
}

/// What an import would do, without writing anything.
pub fn preview(
    kind: SourceKind,
    path: &Path,
    store: &Store,
    progress: impl FnMut(LibraryProgress),
) -> anyhow::Result<LibraryReport> {
    let mut source = open(kind, path)?;
    run(
        kind,
        source.as_mut(),
        store,
        &Options {
            dry_run: true,
            into_inbox: false,
        },
        progress,
    )
}

/// Imports `path` as a source of `kind`.
pub fn import_source(
    kind: SourceKind,
    path: &Path,
    store: &Store,
    progress: impl FnMut(LibraryProgress),
) -> anyhow::Result<LibraryReport> {
    let mut source = open(kind, path)?;
    run(kind, source.as_mut(), store, &Options::default(), progress)
}

/// Imports one Markdown file into the Inbox, with the files it links to
/// beside it. Opening the same file again updates the same note. The
/// report's `first_note` is the note.
pub fn import_markdown_file(path: &Path, store: &Store) -> anyhow::Result<LibraryReport> {
    let mut source = markdown::MarkdownSource::single(path)?;
    run(
        SourceKind::Markdown,
        &mut source,
        store,
        &Options {
            dry_run: false,
            into_inbox: true,
        },
        |_| {},
    )
}

/// A note as the first pass saw it.
struct Entry {
    key: String,
    folder: Vec<String>,
    title: String,
    id: String,
}

/// Imports (or, with `dry_run`, previews) everything in `source`.
pub fn run(
    kind: SourceKind,
    source: &mut dyn Source,
    store: &Store,
    options: &Options,
    mut progress: impl FnMut(LibraryProgress),
) -> anyhow::Result<LibraryReport> {
    let name = clean_name(&source.name());
    let ids = Ids {
        kind,
        source: if options.into_inbox {
            String::new()
        } else {
            name.clone()
        },
    };
    let mut report = LibraryReport {
        kind: Some(kind),
        source: name.clone(),
        dry_run: options.dry_run,
        ..LibraryReport::default()
    };

    // Pass 1: every note's key, path and title.
    let mut entries: Vec<Entry> = Vec::new();
    let mut indexed: Vec<IndexedNote> = Vec::new();
    let mut seen_keys = HashSet::new();
    progress(LibraryProgress {
        phase: LibraryPhase::Reading,
        done: 0,
        total: 0,
    });
    source.notes(false, &mut |note| {
        let mut key = note.key.clone();
        let mut n = 2;
        while !seen_keys.insert(key.clone()) {
            key = format!("{}#{n}", note.key);
            n += 1;
        }
        let title = clean_title(&note.title);
        entries.push(Entry {
            id: ids.id(DocType::Note, &key),
            key,
            folder: note.folder.iter().map(|f| clean_name(f)).collect(),
            title: title.clone(),
        });
        indexed.push(IndexedNote {
            path: note.path,
            title,
            aliases: note.aliases,
        });
        progress(LibraryProgress {
            phase: LibraryPhase::Reading,
            done: entries.len(),
            total: 0,
        });
        Ok(())
    })?;
    let files: Vec<TreeFile> = source.files().to_vec();
    let index = Index::new(
        indexed,
        files.iter().map(|f| f.path.clone()).collect(),
        source.attachment_dirs(),
    );
    let by_key: HashMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(ix, e)| (e.key.as_str(), ix))
        .collect();
    report.found.notes = entries.len();

    // Notebooks: the source's own, then one per folder.
    let mut writer = Writer {
        store,
        ids: &ids,
        dry_run: options.dry_run,
    };
    let mut book_ids: HashMap<Vec<String>, String> = HashMap::new();
    if options.into_inbox {
        let inbox = if options.dry_run {
            store.find_role_book(Role::Inbox)?.unwrap_or_default()
        } else {
            store.role_book(Role::Inbox, Origin::Import)?
        };
        book_ids.insert(Vec::new(), inbox);
    } else {
        let folders: BTreeSet<Vec<String>> = entries
            .iter()
            .flat_map(|e| (0..=e.folder.len()).map(|n| e.folder[..n].to_vec()))
            .collect();
        report.found.notebooks = folders.len();
        let total = folders.len();
        for (done, folder) in folders.into_iter().enumerate() {
            let parent = (!folder.is_empty())
                .then(|| book_ids.get(&folder[..folder.len() - 1]).cloned())
                .flatten();
            let book_name = folder.last().cloned().unwrap_or_else(|| name.clone());
            let key = format!("book:{}", folder.join("/"));
            let id = ids.id(DocType::Book, &key);
            let mut fields = Map::new();
            fields.insert("name".into(), book_name.into());
            fields.insert(
                "parentBookId".into(),
                parent.map_or(Value::Null, Value::String),
            );
            let outcome = writer.write(DocType::Book, &id, &key, fields, None)?;
            report.notebooks.add(outcome, &id, &mut report.rejected);
            // A notebook deleted in Kanso stays deleted: its notes go to
            // the nearest notebook that is still there, or the Inbox.
            let live = options.dry_run || store.get(&id)?.is_some();
            let target = if live {
                id
            } else {
                match folder.len() {
                    0 => store.role_book(Role::Inbox, Origin::Import)?,
                    n => book_ids
                        .get(&folder[..n - 1])
                        .cloned()
                        .context("a parent notebook is missing")?,
                }
            };
            book_ids.insert(folder, target);
            progress(LibraryProgress {
                phase: LibraryPhase::Notebooks,
                done: done + 1,
                total,
            });
        }
    }

    // Tags already in the store, by lowercased name.
    let mut tags = Tags {
        ids: store
            .tags()?
            .into_iter()
            .map(|t| (t.name.to_lowercase(), t.id))
            .collect(),
        seen: HashSet::new(),
    };

    // Pass 2: write notes. Attachments' IDs come from their paths, so
    // notes can point at them before they are written.
    let mut needed: BTreeSet<usize> = BTreeSet::new();
    let mut embedded_seen: HashSet<String> = HashSet::new();
    let mut too_big: BTreeSet<usize> = BTreeSet::new();
    let total = entries.len();
    let mut done = 0;
    progress(LibraryProgress {
        phase: LibraryPhase::Notes,
        done,
        total,
    });
    let rewriter = Rewriter {
        ids: &ids,
        index: &index,
        entries: &entries,
        by_key: &by_key,
        files: &files,
    };
    source.notes(true, &mut |note| {
        let Some(entry) = entries.get(done) else {
            anyhow::bail!("the source changed while it was being imported");
        };
        done += 1;
        if note.key != entry.key && !entry.key.starts_with(&format!("{}#", note.key)) {
            anyhow::bail!("the source changed while it was being imported");
        }
        let mut embedded = HashMap::new();
        for file in &note.embedded {
            let key = format!("embedded:{}", file.key);
            let id = ids.id(DocType::File, &key);
            // Notes sharing an attachment share one file.
            if !embedded_seen.insert(file.key.clone()) {
                embedded.insert(file.key.clone(), id);
                continue;
            }
            report.found.attachments += 1;
            if file.data.len() > MAX_ATTACHMENT_SIZE {
                report.skipped.push(Skipped::new(
                    format!("{} › {}", entry.title, file.name),
                    format!("larger than {} MB", MAX_ATTACHMENT_SIZE / (1024 * 1024)),
                ));
                continue;
            }
            let content_type = file
                .content_type
                .clone()
                .unwrap_or_else(|| content_type(&file.name).to_owned());
            let fields = file_fields(&file.name, &content_type, note.created);
            let attachment = Attachment::new(file.data.clone());
            let outcome = writer.write(
                DocType::File,
                &id,
                &key,
                fields,
                Some(Blob::Bytes(attachment)),
            )?;
            report.attachments.add(outcome, &id, &mut report.rejected);
            embedded.insert(file.key.clone(), id);
        }
        let body = rewriter.rewrite(&note, &embedded, &mut needed, &mut too_big, &mut report);
        if body.chars().count() > MAX_BODY_LEN {
            report.rejected.push(format!(
                "{}: the note is longer than {MAX_BODY_LEN} characters",
                note.path.as_deref().unwrap_or(&entry.title)
            ));
        } else {
            let book = if note.trashed {
                TRASH_BOOK_ID.to_owned()
            } else {
                book_ids
                    .get(&entry.folder)
                    .or_else(|| book_ids.get(&Vec::new()))
                    .cloned()
                    .unwrap_or_default()
            };
            let tag_ids = tags.resolve(&note.tags, store, options.dry_run, &mut report)?;
            let mut fields = Map::new();
            fields.insert("title".into(), entry.title.clone().into());
            fields.insert("body".into(), body.into());
            fields.insert("bookId".into(), book.into());
            fields.insert("tags".into(), tag_ids.into());
            fields.insert("pinned".into(), note.pinned.into());
            if let Some(status) = note.status {
                fields.insert("status".into(), status.as_str().into());
            }
            if let Some(created) = note.created.or(note.updated) {
                fields.insert("createdAt".into(), created.into());
            }
            if let Some(updated) = note.updated.or(note.created) {
                fields.insert("updatedAt".into(), updated.into());
            }
            let outcome = writer.write(DocType::Note, &entry.id, &entry.key, fields, None)?;
            report.notes.add(outcome, &entry.id, &mut report.rejected);
            report.trashed += usize::from(note.trashed);
            if report.first_note.is_none() {
                report.first_note = Some(entry.id.clone());
            }
        }
        progress(LibraryProgress {
            phase: LibraryPhase::Notes,
            done,
            total,
        });
        Ok(())
    })?;
    report.found.tags = tags.seen.len();

    // Pass 3: the files notes link to.
    for &ix in &needed {
        let file = &files[ix];
        let key = format!("file:{}", file.path);
        let id = ids.id(DocType::File, &key);
        report.found.attachments += 1;
        let name = source
            .attachment_name(&file.path)
            .unwrap_or_else(|| file_name(&file.path).to_owned());
        // The type follows the file, whatever the app called it.
        let kind = content_type(file_name(&file.path));
        let fields = file_fields(&name, kind, file.created.or(file.modified));
        let blob = if options.dry_run {
            Blob::Planned(file.size)
        } else {
            match source.read(&file.path, MAX_ATTACHMENT_SIZE as u64) {
                Ok(data) => Blob::Bytes(Attachment::new(data)),
                Err(err) => {
                    report
                        .rejected
                        .push(format!("{}: cannot read it: {err}", file.path));
                    continue;
                }
            }
        };
        let outcome = writer.write(DocType::File, &id, &key, fields, Some(blob))?;
        report.attachments.add(outcome, &id, &mut report.rejected);
    }
    for &ix in &too_big {
        report.skipped.push(Skipped::new(
            files[ix].path.clone(),
            format!(
                "larger than {} MB, the most an attachment can be",
                MAX_ATTACHMENT_SIZE / (1024 * 1024)
            ),
        ));
    }
    let mut skipped = source.skipped();
    for (ix, file) in files.iter().enumerate() {
        if !needed.contains(&ix) && !too_big.contains(&ix) {
            skipped.push(Skipped::new(
                file.path.clone(),
                "no note links to it".to_owned(),
            ));
        }
    }
    report.skipped.extend(skipped);
    Ok(report)
}

/// Derives document IDs from the source and a key.
struct Ids {
    kind: SourceKind,
    /// The source's name, so two vaults with a `README.md` each get two
    /// notes. Empty for single files opened into the Inbox.
    source: String,
}

impl Ids {
    fn id(&self, doc_type: DocType, key: &str) -> String {
        stable_id(doc_type, self.kind, &self.source, key)
    }
}

/// The ID a document from `kind`'s source `source` with `key` gets: the
/// type's prefix and 12 URL-safe characters of a SHA-1 of the three.
pub fn stable_id(doc_type: DocType, kind: SourceKind, source: &str, key: &str) -> String {
    let digest = Sha1::digest(format!("{}\0{source}\0{key}", kind.as_str()).as_bytes());
    let suffix = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..9]);
    format!("{}:{suffix}", doc_type.as_str())
}

/// The part of an ID after its `type:` prefix, as ID links use it.
fn suffix(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, s)| s)
}

/// Tags by name, made when missing.
struct Tags {
    ids: HashMap<String, String>,
    /// Every distinct name notes used, lowercased.
    seen: HashSet<String>,
}

impl Tags {
    fn resolve(
        &mut self,
        names: &[String],
        store: &Store,
        dry_run: bool,
        report: &mut LibraryReport,
    ) -> anyhow::Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        for name in names {
            let name = name.trim().trim_start_matches('#').trim();
            if name.is_empty() {
                continue;
            }
            if name.chars().count() > MAX_NAME_LEN {
                report.skipped.push(Skipped::new(
                    format!("#{name}"),
                    format!("tag names are at most {MAX_NAME_LEN} characters"),
                ));
                continue;
            }
            let lower = name.to_lowercase();
            self.seen.insert(lower.clone());
            let id = match self.ids.get(&lower) {
                Some(id) => id.clone(),
                None if dry_run => {
                    self.ids.insert(lower, String::new());
                    report.tags_created += 1;
                    continue;
                }
                None => {
                    let id = store.tag_named(name, Origin::Import)?;
                    self.ids.insert(lower, id.clone());
                    report.tags_created += 1;
                    id
                }
            };
            if !id.is_empty() && !out.contains(&id) {
                out.push(id);
            }
        }
        Ok(out)
    }
}

/// What a write did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Created,
    Updated,
    Unchanged,
    /// Changed in Kanso since it was imported; left as it is.
    KeptEdited,
    /// Deleted in Kanso; not brought back.
    KeptDeleted,
    /// The store refused it, with its reason.
    Rejected(String),
}

/// An attachment's bytes, or (when previewing) only its size.
enum Blob {
    Bytes(Attachment),
    Planned(u64),
}

struct Writer<'a> {
    store: &'a Store,
    ids: &'a Ids,
    dry_run: bool,
}

impl Writer<'_> {
    /// Writes one document unless the store has it as imported, or it was
    /// changed in Kanso since (see the module's documentation).
    fn write(
        &mut self,
        doc_type: DocType,
        id: &str,
        key: &str,
        mut fields: Map<String, Value>,
        blob: Option<Blob>,
    ) -> anyhow::Result<Outcome> {
        let current = self.store.get(id)?;
        if let Some(Blob::Planned(size)) = blob {
            // A preview reads no attachment: one already here with the same
            // size is taken to be the same.
            return Ok(match current {
                None => Outcome::Created,
                Some(doc) => {
                    let same =
                        doc.fields.get("contentLength").and_then(Value::as_u64) == Some(size);
                    if same {
                        Outcome::Unchanged
                    } else if mark_generation(&doc) == rev_generation(&doc.rev) {
                        Outcome::Updated
                    } else {
                        Outcome::KeptEdited
                    }
                }
            });
        }
        let attachment = match blob {
            Some(Blob::Bytes(attachment)) => Some(attachment),
            _ => None,
        };
        if let Some(attachment) = &attachment {
            fields.insert("md5digest".into(), attachment.md5_hex().into());
            fields.insert("contentLength".into(), attachment.data.len().into());
        }
        let digest = digest_of(&fields);
        let generation = match &current {
            None => 1,
            Some(doc) => {
                let marked = doc.fields.get(IMPORT_FIELD);
                if marked.and_then(|m| m.get("digest")).and_then(Value::as_str)
                    == Some(digest.as_str())
                {
                    return Ok(Outcome::Unchanged);
                }
                let generation = rev_generation(&doc.rev);
                if marked.is_none() || mark_generation(doc) != generation {
                    return Ok(Outcome::KeptEdited);
                }
                generation.unwrap_or(0) + 1
            }
        };
        let updated = current.is_some();
        if self.dry_run {
            return Ok(if updated {
                Outcome::Updated
            } else {
                Outcome::Created
            });
        }
        fields.insert(
            IMPORT_FIELD.into(),
            json!({
                "source": self.ids.kind.as_str(),
                "key": key,
                "digest": digest,
                "generation": generation,
            }),
        );
        let hash = Rev::next(None, &Value::Object(fields.clone())).hash;
        let rev = Rev { generation, hash }.to_string();
        match self
            .store
            .import_doc(doc_type, id, &rev, fields, attachment)
        {
            Ok(true) if updated => Ok(Outcome::Updated),
            Ok(true) => Ok(Outcome::Created),
            Ok(false) => Ok(Outcome::KeptDeleted),
            Err(StoreError::Invalid(reason)) => Ok(Outcome::Rejected(reason)),
            Err(err) => Err(err).with_context(|| format!("cannot import {key}")),
        }
    }
}

fn rev_generation(rev: &str) -> Option<u64> {
    rev.parse::<Rev>().ok().map(|r| r.generation)
}

fn mark_generation(doc: &kanso_store::Doc) -> Option<u64> {
    doc.fields
        .get(IMPORT_FIELD)
        .and_then(|m| m.get("generation"))
        .and_then(Value::as_u64)
}

/// A digest of what an import writes, to tell whether the source changed.
fn digest_of(fields: &Map<String, Value>) -> String {
    let mut keys: Vec<&String> = fields.keys().collect();
    keys.sort();
    let mut hasher = Sha1::new();
    for key in keys {
        hasher.update(key.as_bytes());
        hasher.update([0]);
        hasher.update(fields[key].to_string().as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn file_fields(name: &str, content_type: &str, created: Option<i64>) -> Map<String, Value> {
    let mut fields = Map::new();
    let name: String = if name.chars().count() > 128 {
        let ext = extension(name);
        let keep = 127usize.saturating_sub(ext.chars().count());
        let mut short: String = stem(name).chars().take(keep).collect();
        if !ext.is_empty() {
            short.push('.');
            short.push_str(&ext);
        }
        short
    } else if name.is_empty() {
        "attachment".to_owned()
    } else {
        name.to_owned()
    };
    fields.insert("name".into(), name.into());
    fields.insert("contentType".into(), content_type.into());
    if let Some(created) = created {
        fields.insert("createdAt".into(), created.into());
    }
    fields
}

/// The content type of a file, by its extension.
pub fn content_type(name: &str) -> &'static str {
    match extension(name).as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "heic" => "image/heic",
        "heif" => "image/heif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "zip" => "application/zip",
        "excalidraw" => "application/json",
        _ => "application/octet-stream",
    }
}

fn is_image(name: &str) -> bool {
    content_type(name).starts_with("image/")
}

/// A notebook name Kanso accepts: whitespace collapsed, at most
/// [`MAX_NAME_LEN`] characters, `Untitled` when empty.
pub fn clean_name(name: &str) -> String {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return "Untitled".to_owned();
    }
    name.chars().take(MAX_NAME_LEN).collect()
}

/// A note title Kanso accepts: one line, at most [`MAX_TITLE_LEN`]
/// characters, `Untitled` when empty.
pub fn clean_title(title: &str) -> String {
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() {
        return "Untitled".to_owned();
    }
    title.chars().take(MAX_TITLE_LEN).collect()
}

/// Points a note's links at what the import made.
struct Rewriter<'a> {
    ids: &'a Ids,
    index: &'a Index,
    entries: &'a [Entry],
    by_key: &'a HashMap<&'a str, usize>,
    files: &'a [TreeFile],
}

/// A pseudo-link sources write for targets they know by key.
const PSEUDO: &str = "kanso-import:";

impl Rewriter<'_> {
    fn rewrite(
        &self,
        note: &SourceNote,
        embedded: &HashMap<String, String>,
        needed: &mut BTreeSet<usize>,
        too_big: &mut BTreeSet<usize>,
        report: &mut LibraryReport,
    ) -> String {
        let from = note.path.as_deref();
        let mut edits = Vec::new();
        for found in links::scan(&note.body) {
            let mut file_target = |ix: usize, needed: &mut BTreeSet<usize>| -> Option<String> {
                if self.files[ix].size > MAX_ATTACHMENT_SIZE as u64 {
                    too_big.insert(ix);
                    return None;
                }
                needed.insert(ix);
                Some(self.file_id(ix))
            };
            match found.kind {
                Kind::Wiki | Kind::Embed => match self.index.wiki(from, &found.target) {
                    Some(Target::Note(ix)) => {
                        if found.kind == Kind::Embed {
                            report.embeds_as_links += 1;
                        }
                        edits.push((found.range.clone(), self.wiki_text(&found, ix)));
                    }
                    Some(Target::File(ix)) => {
                        let Some(id) = file_target(ix, needed) else {
                            continue;
                        };
                        let path = &self.files[ix].path;
                        let text = attachment_link(
                            found.kind == Kind::Embed,
                            file_name(path),
                            found.label.as_deref(),
                            &id,
                        );
                        edits.push((found.range.clone(), text));
                    }
                    None => report.unresolved_links += 1,
                },
                Kind::Link | Kind::Image | Kind::Definition | Kind::HtmlImage => {
                    let target = if let Some(pseudo) = found.target.strip_prefix(PSEUDO) {
                        if let Some(key) = pseudo.strip_prefix("note/") {
                            // As written, then `%`-decoded: a key in a
                            // Markdown link may need encoding (`a%20b.html`).
                            self.by_key
                                .get(key)
                                .or_else(|| {
                                    let decoded =
                                        percent_encoding::percent_decode_str(key).decode_utf8();
                                    self.by_key.get(decoded.ok()?.as_ref())
                                })
                                .map(|&ix| Target::Note(ix))
                        } else if let Some(key) = pseudo.strip_prefix("embedded/") {
                            match embedded.get(key) {
                                Some(id) => {
                                    edits.push((
                                        found.range.clone(),
                                        format!("{FILE_LINK_PREFIX}{}", suffix(id)),
                                    ));
                                }
                                None => report.unresolved_links += 1,
                            }
                            continue;
                        } else if let Some(path) = pseudo.strip_prefix("path/") {
                            self.index.path(None, path)
                        } else {
                            None
                        }
                    } else {
                        links::as_relative_path(&found.target)
                            .and_then(|path| self.index.path(from, &path))
                    };
                    let uri = match target {
                        Some(Target::Note(ix)) => Some(format!(
                            "{NOTE_LINK_PREFIX}{}",
                            suffix(&self.entries[ix].id)
                        )),
                        Some(Target::File(ix)) => file_target(ix, needed)
                            .map(|id| format!("{FILE_LINK_PREFIX}{}", suffix(&id))),
                        None => {
                            let pseudo = found.target.starts_with(PSEUDO);
                            if pseudo || looks_local(&found.target) {
                                report.unresolved_links += 1;
                            }
                            None
                        }
                    };
                    if let Some(uri) = uri {
                        edits.push((found.range.clone(), uri));
                    }
                }
            }
        }
        links::apply(&note.body, edits)
    }

    fn file_id(&self, ix: usize) -> String {
        // The same ID pass 3 gives the file.
        self.ids
            .id(DocType::File, &format!("file:{}", self.files[ix].path))
    }

    /// `[[Title#heading|label]]` for a wiki-link to the note at `ix`,
    /// keeping what it read as.
    fn wiki_text(&self, found: &links::Found, ix: usize) -> String {
        let title = &self.entries[ix].title;
        let label = found.label.clone().or_else(|| {
            let written = found.target.trim();
            let same = kanso_core::links::normalize_title(written)
                == kanso_core::links::normalize_title(title);
            (!same).then(|| match &found.heading {
                Some(h) => format!("{} › {h}", tree::stem(file_name(written))),
                None => tree::stem(file_name(written)).to_owned(),
            })
        });
        // A block reference (`#^id`) has no heading in Kanso.
        let heading = found.heading.as_deref().filter(|h| !h.starts_with('^'));
        if title.contains(['[', ']', '|', '#']) {
            let text = label.as_deref().unwrap_or(title);
            return format!(
                "[{}]({NOTE_LINK_PREFIX}{})",
                text.replace(['[', ']'], ""),
                suffix(&self.entries[ix].id)
            );
        }
        let mut text = format!("[[{title}");
        if let Some(heading) = heading {
            text.push('#');
            text.push_str(heading);
        }
        if let Some(label) = label {
            text.push('|');
            text.push_str(&label);
        }
        text.push_str("]]");
        text
    }
}

/// Whether a link target is a local file rather than a URL or an anchor.
fn looks_local(target: &str) -> bool {
    links::as_relative_path(target).is_some_and(|path| !extension(&path).is_empty())
}

/// What a wiki-link or embed of an attachment becomes: an image for an
/// embedded picture (`![[pic.png|300]]` keeps its width), a link for the rest.
fn attachment_link(embed: bool, name: &str, label: Option<&str>, id: &str) -> String {
    let uri = format!("{FILE_LINK_PREFIX}{}", suffix(id));
    let alt = stem(name).replace(['[', ']'], "");
    if embed && is_image(name) {
        match label {
            Some(width) if is_size(width) => format!("![{alt}|{width}]({uri})"),
            Some(label) => format!("![{}]({uri})", label.replace(['[', ']'], "")),
            None => format!("![{alt}]({uri})"),
        }
    } else {
        let text = label.filter(|l| !is_size(l)).unwrap_or(name);
        format!("[{}]({uri})", text.replace(['[', ']'], ""))
    }
}

/// `300` or `300x200`: an embed's size.
fn is_size(text: &str) -> bool {
    let mut parts = text.split('x');
    let digits = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    parts.next().is_some_and(digits) && parts.all(digits)
}
