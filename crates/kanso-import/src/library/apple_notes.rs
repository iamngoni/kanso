//! Reads Apple Notes' Core Data store through a read-only SQLite snapshot.
//!
//! The snapshot is retained for this source's lifetime so both note passes see
//! identical data. Apple attachment files are read lazily and independently;
//! one absent iCloud file never prevents its note from being imported.

use std::collections::{HashMap, HashSet};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
#[cfg(target_os = "macos")]
use std::time::Duration;

use anyhow::Context as _;
use rusqlite::Connection;
#[cfg(target_os = "macos")]
use rusqlite::backup::{Backup, StepResult};
#[cfg(target_os = "macos")]
use rusqlite::{OpenFlags, Row};
use tempfile::TempDir;

use super::report::Skipped;
use super::tree::TreeFile;
use super::{Embedded, Source, SourceNote};

#[path = "apple_notes/convert.rs"]
mod convert;
#[path = "apple_notes/proto.rs"]
mod proto;

/// The database inside Apple Notes' group container.
pub const DATABASE_NAME: &str = "NoteStore.sqlite";
/// The group container macOS gives Apple Notes.
pub const GROUP_CONTAINER_NAME: &str = "group.com.apple.notes";

/// Where Apple Notes keeps its data for `home`.
pub fn default_data_root(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Group Containers")
        .join(GROUP_CONTAINER_NAME)
}

/// The database in Apple Notes' standard data root for `home`.
pub fn default_database_path(home: &Path) -> PathBuf {
    default_data_root(home).join(DATABASE_NAME)
}
#[cfg(target_os = "macos")]
const SNAPSHOT_ATTEMPTS: u32 = 20;
const CORE_TIME_OFFSET_SECONDS: f64 = 978_307_200.0;
const MAX_COMPRESSED_NOTE_BYTES: usize = 8 * 1024 * 1024;
const MAX_NOTE_PROTO_BYTES: usize = 16 * 1024 * 1024;
const MAX_ATTACHMENT_BYTES: u64 = kanso_core::model::MAX_ATTACHMENT_SIZE as u64;
const MAX_PDF_BYTES: u64 = kanso_core::model::MAX_PDF_SIZE as u64;
const INLINE_ATTACHMENT_PREFIX: &str = "com.apple.notes.inlinetextattachment.";

/// A stable, read-only view of one Apple Notes database.
pub struct AppleNotesSource {
    name: String,
    database: Connection,
    _snapshot: TempDir,
    data_root: PathBuf,
    entities: EntityIds,
    columns: HashSet<String>,
    accounts: HashMap<i64, Account>,
    folders: HashMap<i64, Folder>,
    notes: Vec<NoteMeta>,
    note_keys: HashMap<String, String>,
    multi_account: bool,
    skipped: Vec<Skipped>,
    skipped_keys: HashSet<(String, String)>,
}

#[derive(Clone, Copy)]
struct EntityIds {
    // Only the loaders read these, and they run on macOS alone.
    #[cfg(target_os = "macos")]
    account: Option<i64>,
    #[cfg(target_os = "macos")]
    folder: i64,
    #[cfg(target_os = "macos")]
    note: i64,
    attachment: Option<i64>,
    media: Option<i64>,
}

#[derive(Clone)]
struct Account {
    name: String,
    identifier: String,
}

#[derive(Clone)]
struct Folder {
    title: String,
    identifier: String,
    parent: Option<i64>,
    owner: Option<i64>,
    kind: i64,
}

#[derive(Clone)]
struct NoteMeta {
    pk: i64,
    title: String,
    key: String,
    #[cfg(target_os = "macos")]
    identifier: Option<String>,
    folder: Option<i64>,
    creation1: Option<f64>,
    creation2: Option<f64>,
    creation3: Option<f64>,
    modification: Option<f64>,
    password_protected: bool,
    pinned: bool,
}

struct AttachmentResult {
    markdown: String,
    embedded: Vec<Embedded>,
    skipped: Vec<(String, String)>,
    tags: Vec<String>,
}

#[derive(Default)]
struct AttachmentObject {
    pk: Option<i64>,
    alt_text: Option<String>,
    target_identifier: Option<String>,
    url: Option<String>,
    title: Option<String>,
    media: Option<i64>,
    mergeable_data: Option<Vec<u8>>,
    mergeable_data_len: Option<i64>,
}

/// Confirm that the Notes folder, database and WAL sidecars are readable.
///
/// macOS can keep a grant for `NoteStore.sqlite` from an earlier file picker
/// while still denying its `-wal`/`-shm` sidecars. Opening the database alone
/// then fails inside SQLite with `SQLITE_CANTOPEN`. Probe the folder and each
/// sidecar up front so the app can ask for the folder grant the Notes database
/// actually needs.
pub fn check_read_access(database_path: &Path) -> io::Result<()> {
    let data_root = database_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Apple Notes database path has no parent directory",
        )
    })?;
    std::fs::read_dir(data_root)?;
    std::fs::File::open(database_path)?;
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = database_path.as_os_str().to_owned();
        sidecar.push(suffix);
        match std::fs::File::open(PathBuf::from(sidecar)) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

/// Open an Apple Notes group-container folder or a `NoteStore.sqlite` file.
///
/// On platforms other than macOS this function is available but opening fails
/// with an explicit unsupported-platform error.
pub fn open(path: &Path) -> anyhow::Result<AppleNotesSource> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        anyhow::bail!("Apple Notes import is supported only on macOS");
    }

    #[cfg(target_os = "macos")]
    {
        AppleNotesSource::open_macos(path)
    }
}

impl AppleNotesSource {
    #[cfg(target_os = "macos")]
    fn open_macos(path: &Path) -> anyhow::Result<Self> {
        let input = expand_home(path);
        let database_path = if input.is_dir() {
            input.join(DATABASE_NAME)
        } else {
            input
        };
        anyhow::ensure!(
            database_path.is_file(),
            "Apple Notes database not found at {}. Open Notes once, or choose the group.com.apple.notes folder or NoteStore.sqlite",
            database_path.display()
        );
        match check_read_access(&database_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => anyhow::bail!(
                "macOS did not allow Kanso to read Apple Notes. Allow access when macOS asks, or choose the group.com.apple.notes folder again."
            ),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("could not read Apple Notes at {}", database_path.display())
                });
            }
        }

        let data_root = database_path
            .parent()
            .context("Apple Notes database path has no parent directory")?;
        let data_root = std::fs::canonicalize(data_root)
            .context("could not resolve the Apple Notes data root")?;

        let snapshot =
            TempDir::new().context("could not create an Apple Notes snapshot directory")?;
        let snapshot_path = snapshot.path().join(DATABASE_NAME);
        snapshot_database(&database_path, &snapshot_path)?;
        let database = Connection::open_with_flags(
            &snapshot_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("could not open the read-only Apple Notes snapshot")?;

        let table_names = table_names(&database)?;
        anyhow::ensure!(
            table_names.contains("z_primarykey") && table_names.contains("ziccloudsyncingobject"),
            "unsupported Apple Notes database: expected Z_PRIMARYKEY and ZICCloudSyncingObject tables"
        );
        anyhow::ensure!(
            table_names.contains("zicnotedata"),
            "unsupported Apple Notes database: ZICNoteData table is missing"
        );

        let key_columns = table_columns(&database, "Z_PRIMARYKEY")?;
        anyhow::ensure!(
            ["z_name", "z_ent"]
                .iter()
                .all(|column| key_columns.contains(*column)),
            "unsupported Apple Notes database: Z_PRIMARYKEY is missing Z_NAME or Z_ENT"
        );
        let columns = table_columns(&database, "ZICCloudSyncingObject")?;
        let data_columns = table_columns(&database, "ZICNoteData")?;
        anyhow::ensure!(
            ["z_pk", "z_ent"].iter().all(|c| columns.contains(*c)),
            "unsupported Apple Notes database: ZICCloudSyncingObject is missing Z_PK or Z_ENT"
        );
        anyhow::ensure!(
            ["z_pk", "znote", "zdata"]
                .iter()
                .all(|c| data_columns.contains(*c)),
            "unsupported Apple Notes database: ZICNoteData is missing Z_PK, ZNOTE or ZDATA"
        );
        let entities = load_entities(&database)?;
        let accounts = load_accounts(&database, entities, &columns)?;
        let folders = load_folders(&database, entities, &columns)?;
        let notes = load_notes(&database, entities, &columns)?;
        let note_keys = notes
            .iter()
            .filter_map(|note| {
                note.identifier
                    .as_ref()
                    .map(|identifier| (normalize_identifier(identifier), note.key.clone()))
            })
            .collect();
        let multi_account = accounts.len() > 1;

        Ok(Self {
            name: "Apple Notes".to_owned(),
            database,
            _snapshot: snapshot,
            data_root,
            entities,
            columns,
            accounts,
            folders,
            notes,
            note_keys,
            multi_account,
            skipped: Vec::new(),
            skipped_keys: HashSet::new(),
        })
    }

    fn note_body(&mut self, note: &NoteMeta) -> anyhow::Result<Option<proto::Note>> {
        let stored = {
            let mut statement = self.database.prepare(
                "SELECT substr(ZDATA, 1, ?2), length(ZDATA) FROM ZICNoteData WHERE ZNOTE = ?1 ORDER BY Z_PK LIMIT 1",
            )
            .context("could not prepare the Apple Notes body query")?;
            let mut rows = statement
                .query((note.pk, MAX_COMPRESSED_NOTE_BYTES as i64 + 1))
                .with_context(|| format!("could not read body for note {}", note.title))?;
            match rows.next()? {
                Some(row) => Some((
                    row.get::<_, Option<Vec<u8>>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                )),
                None => None,
            }
        };
        let Some((bytes, size)) = stored else {
            self.push_skipped(
                note.title.clone(),
                "note body has not downloaded from iCloud (ZICNoteData row is missing)",
            );
            return Ok(None);
        };
        let Some(size) = size else {
            self.push_skipped(note.title.clone(), "note body is empty");
            return Ok(None);
        };
        if size < 0 || size as usize > MAX_COMPRESSED_NOTE_BYTES {
            self.push_skipped(
                note.title.clone(),
                format!(
                    "compressed note body exceeds the {MAX_COMPRESSED_NOTE_BYTES}-byte read limit"
                ),
            );
            return Ok(None);
        }
        let Some(bytes) = bytes else {
            self.push_skipped(note.title.clone(), "note body is NULL in ZICNoteData");
            return Ok(None);
        };
        match proto::decode_note(&bytes, MAX_NOTE_PROTO_BYTES) {
            Ok(note_proto) => Ok(Some(note_proto)),
            Err(error) => {
                self.push_skipped(
                    note.title.clone(),
                    format!("note body is malformed or unsupported: {error}"),
                );
                Ok(None)
            }
        }
    }

    fn note_folder_path(&self, note: &NoteMeta) -> Vec<String> {
        let mut path = Vec::new();
        if self.multi_account
            && let Some(folder) = note.folder.and_then(|id| self.folders.get(&id))
            && let Some(owner) = folder.owner
            && let Some(name) = self.account_folder_name(owner)
        {
            path.push(name);
        }
        let mut ancestors = Vec::new();
        let mut current = note.folder;
        let mut seen = HashSet::new();
        while let Some(id) = current {
            if !seen.insert(id) {
                break;
            }
            let Some(folder) = self.folders.get(&id) else {
                break;
            };
            if matches!(folder.kind, 1 | 3) {
                break;
            }
            if !folder.identifier.starts_with("DefaultFolder") {
                let title = folder.title.trim();
                if !title.is_empty() {
                    ancestors.push(title.to_owned());
                }
            }
            current = folder.parent;
        }
        ancestors.reverse();
        path.extend(ancestors);
        path
    }

    fn note_trashed(&self, note: &NoteMeta) -> bool {
        note.folder
            .and_then(|id| self.folders.get(&id))
            .is_some_and(|folder| folder.kind == 1)
    }

    fn resolve_attachment(
        &mut self,
        note: &NoteMeta,
        identifier: &str,
        uti: &str,
    ) -> AttachmentResult {
        let is_hashtag = is_inline_attachment(uti, "hashtag");
        if is_hashtag || is_inline_attachment(uti, "mention") {
            let text = self
                .object_by_identifier(identifier)
                .and_then(|row| row.alt_text)
                .unwrap_or_else(|| identifier.to_owned());
            let prefix = if is_hashtag {
                if text.starts_with('#') { "" } else { "#" }
            } else if text.starts_with('@') {
                ""
            } else {
                "@"
            };
            return AttachmentResult {
                markdown: format!("{prefix}{text}"),
                embedded: Vec::new(),
                skipped: Vec::new(),
                tags: if is_hashtag { vec![text] } else { Vec::new() },
            };
        }

        let Some(row) = self.object_by_identifier(identifier) else {
            return self.unavailable(
                note,
                format!("attachment {identifier} is missing from ZICCloudSyncingObject"),
                uti,
            );
        };
        let pk = row.pk;

        if is_inline_attachment(uti, "link") {
            let target = row.target_identifier;
            let Some(target) = target else {
                return self.unavailable(
                    note,
                    format!("internal note link {identifier} has no target"),
                    uti,
                );
            };
            return self.internal_link(&target, row.alt_text);
        }

        // Other inline attachments (iOS 18 Math Notes results, for one) have
        // no file behind them; ZALTTEXT holds the text Notes displays.
        if uti.starts_with(INLINE_ATTACHMENT_PREFIX) {
            let Some(text) = row.alt_text.filter(|value| !value.trim().is_empty()) else {
                return self.unavailable(
                    note,
                    format!("inline attachment {identifier} has no text"),
                    uti,
                );
            };
            return AttachmentResult {
                markdown: text,
                embedded: Vec::new(),
                skipped: Vec::new(),
                tags: Vec::new(),
            };
        }

        if uti == "public.url" {
            let url = row.url.unwrap_or_default();
            let card_title = row
                .title
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| url.clone());
            return AttachmentResult {
                markdown: format!("[**{}**]({})", escape_label(&card_title), escape_url(&url)),
                embedded: Vec::new(),
                skipped: Vec::new(),
                tags: Vec::new(),
            };
        }

        if uti == "com.apple.notes.table" {
            if row
                .mergeable_data_len
                .is_some_and(|size| size < 0 || size as usize > MAX_COMPRESSED_NOTE_BYTES)
            {
                return self.unavailable(
                    note,
                    format!("table {identifier} exceeds the compressed protobuf read limit"),
                    uti,
                );
            }
            let blob = row.mergeable_data;
            let Some(blob) = blob else {
                return self.unavailable(
                    note,
                    format!("table {identifier} has no downloaded CRDT data"),
                    uti,
                );
            };
            let table = match proto::decode_mergeable(&blob, MAX_NOTE_PROTO_BYTES) {
                Ok(value) => value,
                Err(error) => {
                    return self.unavailable(
                        note,
                        format!("table {identifier} is malformed: {error}"),
                        uti,
                    );
                }
            };
            let markdown = match convert::render_table(&table) {
                Ok(markdown) => markdown,
                Err(error) => {
                    return self.unavailable(note, format!("table {identifier} {error}"), uti);
                }
            };
            return AttachmentResult {
                markdown,
                embedded: Vec::new(),
                skipped: Vec::new(),
                tags: Vec::new(),
            };
        }

        if uti == "com.apple.paper.doc.scan" {
            let generation =
                pk.and_then(|pk| self.attachment_generation(pk, "ZFALLBACKPDFGENERATION"));
            let Some(generation) = generation else {
                return self.unavailable(
                    note,
                    format!("scan {identifier} has not downloaded its PDF fallback"),
                    uti,
                );
            };
            let relative = PathBuf::from("FallbackPDFs")
                .join(identifier)
                .join(generation)
                .join("FallbackPDF.pdf");
            let mut candidates = Vec::new();
            if let Some(account) = self.account_for(note) {
                candidates.push(
                    PathBuf::from("Accounts")
                        .join(&account.identifier)
                        .join(&relative),
                );
            }
            candidates.push(relative);
            return self.load_embedded_candidates(note, identifier, &candidates, "Scan.pdf", uti);
        }

        if uti == "com.apple.notes.gallery" {
            if row
                .mergeable_data_len
                .is_some_and(|size| size < 0 || size as usize > MAX_COMPRESSED_NOTE_BYTES)
            {
                return self.unavailable(
                    note,
                    format!("scan {identifier} exceeds the compressed protobuf read limit"),
                    uti,
                );
            }
            let blob = row.mergeable_data;
            let Some(blob) = blob else {
                return self.unavailable(
                    note,
                    format!("scan {identifier} has not downloaded its page data"),
                    uti,
                );
            };
            let scan = match proto::decode_mergeable(&blob, MAX_NOTE_PROTO_BYTES) {
                Ok(value) => value,
                Err(error) => {
                    return self.unavailable(
                        note,
                        format!("scan {identifier} is malformed: {error}"),
                        uti,
                    );
                }
            };
            let page_ids = proto::scan_attachment_identifiers(&scan);
            if page_ids.is_empty() {
                return self.unavailable(
                    note,
                    format!("scan {identifier} contains no page references"),
                    uti,
                );
            }
            let mut embedded = Vec::new();
            let mut skipped = Vec::new();
            let mut images = Vec::new();
            for page_identifier in page_ids {
                let page = self.resolve_scan_page(note, &page_identifier);
                if let Some(file) = page.embedded.into_iter().next() {
                    images.push(format!(
                        "![{}](kanso-import:embedded/{})",
                        escape_label(&file.name),
                        file.key
                    ));
                    embedded.push(file);
                }
                skipped.extend(page.skipped);
            }
            if images.is_empty() {
                return AttachmentResult {
                    markdown: "**(scan pages unavailable)**".to_owned(),
                    embedded,
                    skipped,
                    tags: Vec::new(),
                };
            }
            return AttachmentResult {
                markdown: format!("\n{}\n", images.join("\n")),
                embedded,
                skipped,
                tags: Vec::new(),
            };
        }

        if is_drawing(uti) {
            let Some(pk) = pk else {
                return self.unavailable(
                    note,
                    format!("drawing {identifier} has no attachment row"),
                    uti,
                );
            };
            return self.resolve_drawing(note, pk, identifier);
        }

        let media_pk = row.media;
        let Some(media_pk) = media_pk else {
            return self.unavailable(
                note,
                format!("attachment {identifier} has no downloaded media row"),
                uti,
            );
        };
        self.resolve_media(note, media_pk, identifier, uti)
    }

    fn internal_link(&self, target: &str, label: Option<String>) -> AttachmentResult {
        let identifier = target
            .strip_prefix("applenotes:note/")
            .unwrap_or(target)
            .split('?')
            .next()
            .unwrap_or(target);
        let key = self.note_keys.get(&normalize_identifier(identifier));
        match key {
            Some(key) => {
                let display = label
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| key.clone());
                AttachmentResult {
                    markdown: format!("[{}](kanso-import:note/{})", escape_label(&display), key),
                    embedded: Vec::new(),
                    skipped: Vec::new(),
                    tags: Vec::new(),
                }
            }
            None => AttachmentResult {
                markdown: "(unknown note link)".to_owned(),
                embedded: Vec::new(),
                skipped: vec![(
                    "Apple Notes link".to_owned(),
                    format!("linked note {identifier} is not present in this database"),
                )],
                tags: Vec::new(),
            },
        }
    }

    fn resolve_media(
        &mut self,
        note: &NoteMeta,
        media_pk: i64,
        identity: &str,
        uti: &str,
    ) -> AttachmentResult {
        let Some(media_entity) = self.entities.media else {
            return self.unavailable(
                note,
                format!("attachment {identity} cannot be read: ICMedia entity is absent"),
                uti,
            );
        };
        let media_identifier = column_expr(&self.columns, "ZIDENTIFIER", "NULL");
        let media_filename = column_expr(&self.columns, "ZFILENAME", "NULL");
        let generation = column_expr(&self.columns, "ZGENERATION1", "NULL");
        let sql = format!(
            "SELECT {media_identifier}, {media_filename}, {generation} FROM ZICCloudSyncingObject WHERE Z_PK = ?1 AND Z_ENT = ?2"
        );
        let row = match self
            .database
            .query_row(&sql, (media_pk, media_entity), |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            }) {
            Ok(row) => row,
            Err(_) => {
                return self.unavailable(
                    note,
                    format!("attachment {identity} media row is missing"),
                    uti,
                );
            }
        };
        let (identifier, filename, generation) = row;
        let Some(filename) = filename.filter(|name| !name.is_empty()) else {
            return self.unavailable(
                note,
                format!("attachment {identity} has no media filename"),
                uti,
            );
        };
        let Some(identifier) = identifier else {
            return self.unavailable(
                note,
                format!("attachment {identity} media identifier is missing"),
                uti,
            );
        };
        let mut relative = PathBuf::from("Accounts")
            .join(
                self.account_for(note)
                    .map_or("", |account| account.identifier.as_str()),
            )
            .join("Media")
            .join(&identifier);
        let mut legacy = PathBuf::from("Media").join(&identifier);
        if let Some(generation) = generation.as_deref().filter(|value| !value.is_empty()) {
            relative.push(generation);
            legacy.push(generation);
        }
        relative.push(filename.clone());
        legacy.push(filename.clone());
        let mut candidates = Vec::new();
        if self.account_for(note).is_some() {
            candidates.push(relative);
        }
        candidates.push(legacy);
        self.load_embedded_candidates(note, identity, &candidates, &filename, uti)
    }

    fn resolve_drawing(&mut self, note: &NoteMeta, pk: i64, identity: &str) -> AttachmentResult {
        let fallback = column_expr(&self.columns, "ZFALLBACKIMAGEGENERATION", "NULL");
        let width = column_expr(&self.columns, "ZSIZEWIDTH", "NULL");
        let mergeable = if self.columns.contains("zmergeabledata1") {
            "length(ZMERGEABLEDATA1)".to_owned()
        } else {
            "NULL".to_owned()
        };
        let sql = format!(
            "SELECT {fallback}, {width}, {mergeable} FROM ZICCloudSyncingObject WHERE Z_PK = ?1"
        );
        let row = self.database.query_row(&sql, [pk], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        });
        let Ok((generation, width, mergeable)) = row else {
            return self.unavailable(
                note,
                format!("drawing {identity} attachment row is missing"),
                "public.jpeg",
            );
        };
        if generation.is_none() && width.unwrap_or(0) == 0 && mergeable.unwrap_or(0) == 0 {
            return self.unavailable(
                note,
                format!("drawing {identity} has not downloaded from iCloud"),
                "public.jpeg",
            );
        }
        let (relative, name, content_type) =
            if let Some(generation) = generation.filter(|value| !value.is_empty()) {
                (
                    PathBuf::from("FallbackImages")
                        .join(identity)
                        .join(generation)
                        .join("FallbackImage.png"),
                    "Drawing.png",
                    "public.png",
                )
            } else {
                (
                    PathBuf::from("FallbackImages").join(format!("{identity}.jpg")),
                    "Drawing.jpg",
                    "public.jpeg",
                )
            };
        let mut candidates = Vec::new();
        if let Some(account) = self.account_for(note) {
            candidates.push(
                PathBuf::from("Accounts")
                    .join(&account.identifier)
                    .join(&relative),
            );
        }
        candidates.push(relative);
        self.load_embedded_candidates(note, identity, &candidates, name, content_type)
    }

    fn resolve_scan_page(&mut self, note: &NoteMeta, identifier: &str) -> AttachmentResult {
        let Some(entity) = self.entities.attachment else {
            return self.unavailable(
                note,
                format!("scan page {identifier} has no attachment entity"),
                "public.jpeg",
            );
        };
        let width = column_expr(&self.columns, "ZSIZEWIDTH", "NULL");
        let height = column_expr(&self.columns, "ZSIZEHEIGHT", "NULL");
        let media = column_expr(&self.columns, "ZMEDIA", "NULL");
        let sql = format!(
            "SELECT {width}, {height}, {media} FROM ZICCloudSyncingObject WHERE ZIDENTIFIER = ?1 AND Z_ENT = ?2"
        );
        let row = self.database.query_row(&sql, (identifier, entity), |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        });
        let Ok((width, height, media)) = row else {
            return self.unavailable(
                note,
                format!("scan page {identifier} is missing"),
                "public.jpeg",
            );
        };
        let width = width.unwrap_or(0);
        let height = height.unwrap_or(0);
        if width > 0 && height > 0 {
            let relative =
                PathBuf::from("Previews").join(format!("{identifier}-1-{width}x{height}-0.jpeg"));
            let mut candidates = Vec::new();
            if let Some(account) = self.account_for(note) {
                candidates.push(
                    PathBuf::from("Accounts")
                        .join(&account.identifier)
                        .join(&relative),
                );
            }
            candidates.push(relative);
            let preview = self.load_embedded_candidates(
                note,
                identifier,
                &candidates,
                "Scan Page.jpg",
                "image/jpeg",
            );
            if !preview.embedded.is_empty() {
                return preview;
            }
        }
        if let Some(media_pk) = media {
            return self.resolve_media(note, media_pk, identifier, "image/jpeg");
        }
        self.unavailable(
            note,
            format!("scan page {identifier} has no downloaded preview or original media"),
            "image/jpeg",
        )
    }

    fn load_embedded_candidates(
        &self,
        note: &NoteMeta,
        identity: &str,
        relatives: &[PathBuf],
        name: &str,
        uti: &str,
    ) -> AttachmentResult {
        let mut last_error = None;
        for relative in relatives {
            let path = match safe_join(&self.data_root, relative) {
                Ok(path) => path,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    last_error = Some(error.to_string());
                    continue;
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                    break;
                }
            };
            let limit = if content_type_for(uti, name).as_deref() == Some("application/pdf") {
                MAX_PDF_BYTES
            } else {
                MAX_ATTACHMENT_BYTES
            };
            match read_limited_file(&path, limit) {
                Ok(data) => {
                    let key = embedded_key(&note.key, note.pk, identity);
                    let name = if name.is_empty() { "Attachment" } else { name };
                    let content_type = content_type_for(uti, name);
                    return AttachmentResult {
                        markdown: embedded_reference(name, &key, content_type.as_deref()),
                        embedded: vec![Embedded {
                            key,
                            name: name.to_owned(),
                            content_type,
                            data,
                        }],
                        skipped: Vec::new(),
                        tags: Vec::new(),
                    };
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    last_error = Some(error.to_string());
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                    break;
                }
            }
        }
        let locations = relatives
            .iter()
            .map(|relative| relative.display().to_string())
            .collect::<Vec<_>>()
            .join(" or ");
        self.unavailable(
            note,
            format!(
                "attachment {identity} is unavailable at {locations}: {}",
                last_error.unwrap_or_else(|| "file was not found".to_owned())
            ),
            uti,
        )
    }

    fn attachment_generation(&self, pk: i64, column: &str) -> Option<String> {
        if !self.columns.contains(&column.to_ascii_lowercase()) {
            return None;
        }
        let sql = format!("SELECT {column} FROM ZICCloudSyncingObject WHERE Z_PK = ?1");
        self.database
            .query_row(&sql, [pk], |row| row.get::<_, Option<String>>(0))
            .ok()
            .flatten()
    }

    fn unavailable(&self, note: &NoteMeta, reason: String, uti: &str) -> AttachmentResult {
        AttachmentResult {
            markdown: format!("**(attachment unavailable: {})**", plain_label(uti)),
            embedded: Vec::new(),
            skipped: vec![(note.title.clone(), reason)],
            tags: Vec::new(),
        }
    }

    fn account_for(&self, note: &NoteMeta) -> Option<&Account> {
        note.folder
            .and_then(|id| self.folders.get(&id))
            .and_then(|folder| folder.owner)
            .and_then(|id| self.accounts.get(&id))
    }

    fn account_folder_name(&self, account_id: i64) -> Option<String> {
        let account = self.accounts.get(&account_id)?;
        let name = nonempty_name(&account.name, &account.identifier);
        let duplicate_name = self.accounts.values().any(|other| {
            other.identifier != account.identifier
                && !other.name.trim().is_empty()
                && other.name.trim().eq_ignore_ascii_case(name.as_str())
        });
        Some(if duplicate_name {
            format!("{name} ({})", account.identifier)
        } else {
            name
        })
    }

    fn object_by_identifier(&self, identifier: &str) -> Option<AttachmentObject> {
        object_by_identifier(&self.database, &self.columns, identifier)
    }

    fn push_skipped(&mut self, path: impl Into<String>, reason: impl Into<String>) {
        let path = path.into();
        let reason = reason.into();
        if self.skipped_keys.insert((path.clone(), reason.clone())) {
            self.skipped.push(Skipped::new(path, reason));
        }
    }
}

impl Source for AppleNotesSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn files(&self) -> &[TreeFile] {
        &[]
    }

    fn notes(
        &mut self,
        full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let notes = self.notes.clone();
        for note in notes {
            if note.password_protected {
                self.push_skipped(
                    note.title.clone(),
                    "password-protected notes are not imported",
                );
                continue;
            }
            let Some(document) = self.note_body(&note)? else {
                continue;
            };
            let source_title = convert::note_title(&document.note_text, &note.title);
            let title = if source_title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                source_title
            };
            let folder = self.note_folder_path(&note);
            let converted = if full {
                let mut skipped = Vec::new();
                let note_keys = self.note_keys.clone();
                let mut resolve = |identifier: &str, uti: &str| {
                    let result = self.resolve_attachment(&note, identifier, uti);
                    skipped.extend(result.skipped.clone());
                    result
                };
                let rendered = convert::render_note(
                    &document,
                    &title,
                    &note.key,
                    &note_keys,
                    true,
                    &mut resolve,
                );
                for (path, reason) in skipped {
                    self.push_skipped(path, reason);
                }
                rendered
            } else {
                convert::RenderedNote {
                    body: String::new(),
                    embedded: Vec::new(),
                    tags: Vec::new(),
                }
            };
            let trashed = self.note_trashed(&note);
            each(SourceNote {
                key: note.key,
                path: None,
                folder,
                title,
                body: converted.body,
                tags: converted.tags,
                aliases: Vec::new(),
                created: core_time(
                    note.creation3
                        .filter(|value| *value > 0.0)
                        .or_else(|| note.creation2.filter(|value| *value > 0.0))
                        .or_else(|| note.creation1.filter(|value| *value > 0.0)),
                ),
                updated: core_time(note.modification),
                pinned: note.pinned,
                status: None,
                trashed,
                embedded: converted.embedded,
            })?;
        }
        Ok(())
    }

    fn read(&mut self, path: &str, _limit: u64) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Apple Notes attachments are embedded in notes; no source file named {path}"),
        ))
    }

    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// Copy the database in one backup step, under one read transaction.
///
/// An incremental backup restarts whenever another connection writes the
/// source, so a large store Notes.app keeps syncing might never finish.
#[cfg(target_os = "macos")]
fn snapshot_database(source_path: &Path, snapshot_path: &Path) -> anyhow::Result<()> {
    let source = Connection::open_with_flags(
        source_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| {
        format!(
            "could not open Apple Notes database {} read-only",
            source_path.display()
        )
    })?;
    let mut destination = Connection::open_with_flags(
        snapshot_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("could not create the Apple Notes snapshot database")?;
    {
        let backup = Backup::new(&source, &mut destination)
            .context("could not start the Apple Notes SQLite snapshot")?;
        let mut attempts = 0;
        loop {
            match backup
                .step(-1)
                .context("could not finish the Apple Notes SQLite snapshot")?
            {
                StepResult::Done => break,
                _ if attempts + 1 < SNAPSHOT_ATTEMPTS => {
                    attempts += 1;
                    std::thread::sleep(Duration::from_millis(100));
                }
                _ => anyhow::bail!(
                    "could not snapshot the Apple Notes database: it stayed locked (quit Notes and try again)"
                ),
            }
        }
    }
    destination
        .close()
        .map_err(|(_, error)| error)
        .context("could not close the Apple Notes snapshot database")?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn expand_home(path: &Path) -> PathBuf {
    if path == Path::new("~") {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf());
    }
    if let Some(value) = path.to_str().and_then(|value| value.strip_prefix("~/")) {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(value))
            .unwrap_or_else(|| path.to_path_buf());
    }
    path.to_path_buf()
}

#[cfg(target_os = "macos")]
fn table_names(database: &Connection) -> anyhow::Result<HashSet<String>> {
    let mut statement = database.prepare("SELECT name FROM sqlite_master WHERE type = 'table'")?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(names
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .collect())
}

#[cfg(target_os = "macos")]
fn table_columns(database: &Connection, table: &str) -> anyhow::Result<HashSet<String>> {
    let mut statement = database.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(names
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .collect())
}

#[cfg(target_os = "macos")]
fn load_entities(database: &Connection) -> anyhow::Result<EntityIds> {
    let mut statement = database.prepare("SELECT Z_NAME, Z_ENT FROM Z_PRIMARYKEY")?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("could not read entity IDs from Z_PRIMARYKEY")?;
    let ids: HashMap<String, i64> = rows
        .into_iter()
        .map(|(name, id)| (name.to_ascii_lowercase(), id))
        .collect();
    let required = |name: &str| {
        ids.get(&name.to_ascii_lowercase())
            .copied()
            .with_context(|| {
                format!("unsupported Apple Notes database: Z_PRIMARYKEY has no {name} entity")
            })
    };
    Ok(EntityIds {
        account: ids.get("icaccount").copied(),
        folder: required("ICFolder")?,
        note: required("ICNote")?,
        attachment: ids.get("icattachment").copied(),
        media: ids.get("icmedia").copied(),
    })
}

#[cfg(target_os = "macos")]
fn load_accounts(
    database: &Connection,
    ids: EntityIds,
    columns: &HashSet<String>,
) -> anyhow::Result<HashMap<i64, Account>> {
    let Some(entity) = ids.account else {
        return Ok(HashMap::new());
    };
    let name = column_expr(columns, "ZNAME", "NULL");
    let identifier = column_expr(columns, "ZIDENTIFIER", "NULL");
    let sql = format!(
        "SELECT Z_PK, {name}, {identifier} FROM ZICCloudSyncingObject WHERE Z_ENT = ?1 ORDER BY Z_PK"
    );
    let mut statement = database.prepare(&sql)?;
    let rows = statement.query_map([entity], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;
    let mut accounts = HashMap::new();
    for row in rows {
        let (pk, name, identifier) = row?;
        accounts.insert(
            pk,
            Account {
                name: name.unwrap_or_default(),
                identifier: identifier
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| format!("account-{pk}")),
            },
        );
    }
    Ok(accounts)
}

#[cfg(target_os = "macos")]
fn load_folders(
    database: &Connection,
    ids: EntityIds,
    columns: &HashSet<String>,
) -> anyhow::Result<HashMap<i64, Folder>> {
    let title = column_expr(columns, "ZTITLE2", "NULL");
    let identifier = column_expr(columns, "ZIDENTIFIER", "NULL");
    let parent = column_expr(columns, "ZPARENT", "NULL");
    let owner = column_expr(columns, "ZOWNER", "NULL");
    let kind = column_expr(columns, "ZFOLDERTYPE", "0");
    let live = live_filter(columns);
    let sql = format!(
        "SELECT Z_PK, {title}, {identifier}, {parent}, {owner}, {kind} FROM ZICCloudSyncingObject WHERE Z_ENT = ?1{live} ORDER BY Z_PK"
    );
    let mut statement = database.prepare(&sql)?;
    let rows = statement.query_map([ids.folder], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<i64>>(5)?.unwrap_or(0),
        ))
    })?;
    let mut folders = HashMap::new();
    for row in rows {
        let (pk, title, identifier, parent, owner, kind) = row?;
        folders.insert(
            pk,
            Folder {
                title,
                identifier,
                parent,
                owner,
                kind,
            },
        );
    }
    Ok(folders)
}

#[cfg(target_os = "macos")]
fn load_notes(
    database: &Connection,
    ids: EntityIds,
    columns: &HashSet<String>,
) -> anyhow::Result<Vec<NoteMeta>> {
    let title = column_expr(columns, "ZTITLE1", "NULL");
    let identifier = column_expr(columns, "ZIDENTIFIER", "NULL");
    let folder = column_expr(columns, "ZFOLDER", "NULL");
    let creation1 = column_expr(columns, "ZCREATIONDATE1", "NULL");
    let creation2 = column_expr(columns, "ZCREATIONDATE2", "NULL");
    let creation3 = column_expr(columns, "ZCREATIONDATE3", "NULL");
    let modification = column_expr(columns, "ZMODIFICATIONDATE1", "NULL");
    let locked = column_expr(columns, "ZISPASSWORDPROTECTED", "0");
    let pinned = column_expr(columns, "ZISPINNED", "0");
    let live = live_filter(columns);
    let sql = format!(
        "SELECT Z_PK, {title}, {identifier}, {folder}, {creation1}, {creation2}, {creation3}, {modification}, {locked}, {pinned} FROM ZICCloudSyncingObject WHERE Z_ENT = ?1{live} ORDER BY Z_PK"
    );
    let mut statement = database.prepare(&sql)?;
    let rows = statement.query_map([ids.note], read_note_meta)?;
    let mut notes = Vec::new();
    for row in rows {
        let (
            pk,
            title,
            identifier,
            folder,
            creation1,
            creation2,
            creation3,
            modification,
            locked,
            pinned,
        ) = row?;
        let identifier = identifier.filter(|value| !value.trim().is_empty());
        let key = identifier
            .as_ref()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| format!("apple-note-pk-{pk}"));
        notes.push(NoteMeta {
            pk,
            title: title.unwrap_or_default(),
            key,
            identifier,
            folder,
            creation1,
            creation2,
            creation3,
            modification,
            password_protected: locked.unwrap_or(0) != 0,
            pinned: pinned.unwrap_or(0) != 0,
        });
    }
    Ok(notes)
}

#[cfg(target_os = "macos")]
type NoteMetaRow = (
    i64,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<i64>,
    Option<i64>,
);

#[cfg(target_os = "macos")]
fn read_note_meta(row: &Row<'_>) -> rusqlite::Result<NoteMetaRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
    ))
}

/// Leave out rows Notes has marked for deletion: they are purged at the next
/// sync, unlike notes in Recently Deleted. Older stores lack the column.
#[cfg(target_os = "macos")]
fn live_filter(columns: &HashSet<String>) -> &'static str {
    if columns.contains("zmarkedfordeletion") {
        " AND COALESCE(ZMARKEDFORDELETION, 0) = 0"
    } else {
        ""
    }
}

/// Find an attachment's row, preferring an exact identifier match: that one
/// can use the ZIDENTIFIER index, which a case-insensitive match cannot.
fn object_by_identifier(
    database: &Connection,
    columns: &HashSet<String>,
    identifier: &str,
) -> Option<AttachmentObject> {
    let pk = column_expr(columns, "Z_PK", "NULL");
    let alt_text = column_expr(columns, "ZALTTEXT", "NULL");
    let target = column_expr(columns, "ZTOKENCONTENTIDENTIFIER", "NULL");
    let url = column_expr(columns, "ZURLSTRING", "NULL");
    let title = column_expr(columns, "ZTITLE", "NULL");
    let media = column_expr(columns, "ZMEDIA", "NULL");
    let (mergeable, mergeable_len) = if columns.contains("zmergeabledata1") {
        (
            format!(
                "substr(ZMERGEABLEDATA1, 1, {})",
                MAX_COMPRESSED_NOTE_BYTES + 1
            ),
            "length(ZMERGEABLEDATA1)",
        )
    } else {
        ("NULL".to_owned(), "NULL")
    };
    ["", " COLLATE NOCASE"].into_iter().find_map(|collation| {
        let sql = format!(
            "SELECT {pk}, {alt_text}, {target}, {url}, {title}, {media}, {mergeable}, {mergeable_len} FROM ZICCloudSyncingObject WHERE ZIDENTIFIER = ?1{collation} ORDER BY Z_PK LIMIT 1"
        );
        database
            .query_row(&sql, [identifier], |row| {
                Ok(AttachmentObject {
                    pk: row.get(0)?,
                    alt_text: row.get(1)?,
                    target_identifier: row.get(2)?,
                    url: row.get(3)?,
                    title: row.get(4)?,
                    media: row.get(5)?,
                    mergeable_data: row.get(6)?,
                    mergeable_data_len: row.get(7)?,
                })
            })
            .ok()
    })
}

fn column_expr(columns: &HashSet<String>, name: &str, fallback: &str) -> String {
    if columns.contains(&name.to_ascii_lowercase()) {
        name.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn core_time(value: Option<f64>) -> Option<i64> {
    let seconds = value?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }
    ((seconds + CORE_TIME_OFFSET_SECONDS) * 1000.0)
        .floor()
        .to_i64_checked()
}

trait CheckedFloatTime {
    fn to_i64_checked(self) -> Option<i64>;
}

impl CheckedFloatTime for f64 {
    fn to_i64_checked(self) -> Option<i64> {
        (self >= i64::MIN as f64 && self <= i64::MAX as f64).then_some(self as i64)
    }
}

fn normalize_identifier(identifier: &str) -> String {
    identifier.trim().to_ascii_lowercase()
}

fn nonempty_name(name: &str, fallback: &str) -> String {
    if name.trim().is_empty() {
        fallback.to_owned()
    } else {
        name.trim().to_owned()
    }
}

fn embedded_key(note_key: &str, note_pk: i64, identity: &str) -> String {
    let mut key = String::from("apple-notes-");
    for byte in note_key.bytes() {
        use std::fmt::Write as _;
        let _ = write!(key, "{byte:02x}");
    }
    key.push('-');
    use std::fmt::Write as _;
    let _ = write!(key, "{note_pk:x}-");
    for byte in identity.bytes() {
        let _ = write!(key, "{byte:02x}");
    }
    key
}

fn safe_join(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    let mut candidate = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "attachment path contains an unsafe component",
            ));
        };
        candidate.push(name);
        match std::fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "attachment path contains a symbolic link",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(error),
            Err(error) => return Err(error),
        }
    }

    let resolved = std::fs::canonicalize(&candidate)?;
    if !resolved.starts_with(root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "attachment path resolves outside the Apple Notes data root",
        ));
    }
    Ok(resolved)
}

fn read_limited_file(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    if size > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "attachment exceeds the read limit",
        ));
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "attachment exceeds the read limit",
        ));
    }
    Ok(bytes)
}

fn is_inline_attachment(uti: &str, suffix: &str) -> bool {
    uti == format!("com.apple.notes.inlinetextattachment.{suffix}")
}

fn is_drawing(uti: &str) -> bool {
    matches!(
        uti,
        "com.apple.paper" | "com.apple.drawing" | "com.apple.drawing.2"
    )
}

fn plain_label(uti: &str) -> &str {
    match uti {
        "image/jpeg" | "public.jpeg" | "public.png" => "image",
        "com.apple.notes.gallery" | "com.apple.paper.doc.scan" => "scan",
        "com.apple.paper" | "com.apple.drawing" | "com.apple.drawing.2" => "drawing",
        "com.apple.notes.table" => "table",
        "public.url" => "URL card",
        _ if uti.starts_with(INLINE_ATTACHMENT_PREFIX) => "inline text",
        _ => "file",
    }
}

fn content_type_for(uti: &str, name: &str) -> Option<String> {
    let known = match uti {
        "public.png" | "image/png" => Some("image/png"),
        "public.jpeg" | "public.jpg" | "image/jpeg" => Some("image/jpeg"),
        "public.heic" | "public.heif" => Some("image/heic"),
        "public.tiff" => Some("image/tiff"),
        "com.compuserve.gif" => Some("image/gif"),
        "com.adobe.pdf" | "application/pdf" => Some("application/pdf"),
        "public.mpeg-4" | "public.movie" => Some("video/mp4"),
        "public.mp3" => Some("audio/mpeg"),
        "public.m4a-audio" => Some("audio/mp4"),
        "com.microsoft.waveform-audio" => Some("audio/wav"),
        "com.apple.paper" | "com.apple.drawing" | "com.apple.drawing.2" => Some("image/jpeg"),
        "com.apple.notes.gallery" | "com.apple.paper.doc.scan" => Some("image/jpeg"),
        _ => None,
    };
    known
        .map(str::to_owned)
        .or_else(|| extension_content_type(name))
}

fn extension_content_type(name: &str) -> Option<String> {
    match Path::new(name)
        .extension()?
        .to_str()?
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => Some("image/png".to_owned()),
        "jpg" | "jpeg" => Some("image/jpeg".to_owned()),
        "gif" => Some("image/gif".to_owned()),
        "webp" => Some("image/webp".to_owned()),
        "pdf" => Some("application/pdf".to_owned()),
        "mp3" => Some("audio/mpeg".to_owned()),
        "m4a" => Some("audio/mp4".to_owned()),
        "wav" => Some("audio/wav".to_owned()),
        "mp4" | "mov" => Some("video/mp4".to_owned()),
        _ => None,
    }
}

fn embedded_reference(name: &str, key: &str, content_type: Option<&str>) -> String {
    if content_type.is_some_and(|kind| kind.starts_with("image/")) {
        format!(
            "![{}](kanso-import:embedded/{key})",
            convert::escape_markdown_label(name)
        )
    } else {
        format!(
            "[{}](kanso-import:embedded/{key})",
            convert::escape_markdown_label(name)
        )
    }
}

fn escape_label(value: &str) -> String {
    value.replace(['[', ']'], "")
}

fn escape_url(value: &str) -> String {
    value
        .replace(')', "%29")
        .replace('(', "%28")
        .replace(' ', "%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_table() -> (Connection, HashSet<String>) {
        let database = Connection::open_in_memory().unwrap();
        database
            .execute_batch(
                "CREATE TABLE ZICCloudSyncingObject (Z_PK INTEGER PRIMARY KEY, ZIDENTIFIER TEXT, ZALTTEXT TEXT);
                 CREATE INDEX identifier ON ZICCloudSyncingObject (ZIDENTIFIER);
                 INSERT INTO ZICCloudSyncingObject VALUES (1, 'calc-1', 'lower'), (2, 'CALC-1', 'upper');",
            )
            .unwrap();
        let columns = ["z_pk", "zidentifier", "zalttext"]
            .map(str::to_owned)
            .into();
        (database, columns)
    }

    #[test]
    fn exact_identifier_match_wins_over_an_earlier_case_variant() {
        let (database, columns) = object_table();
        let alt = |identifier| {
            object_by_identifier(&database, &columns, identifier).and_then(|row| row.alt_text)
        };
        assert_eq!(alt("CALC-1").as_deref(), Some("upper"));
        assert_eq!(alt("calc-1").as_deref(), Some("lower"));
        // No exact match: the case-insensitive fallback takes the first row.
        assert_eq!(alt("Calc-1").as_deref(), Some("lower"));
        assert!(alt("calc-2").is_none());
    }

    #[test]
    fn default_location_is_the_notes_group_container() {
        let home = Path::new("/Users/example");
        assert_eq!(
            default_data_root(home),
            home.join("Library/Group Containers/group.com.apple.notes")
        );
        assert_eq!(
            default_database_path(home),
            home.join("Library/Group Containers/group.com.apple.notes/NoteStore.sqlite")
        );
    }

    #[test]
    fn access_probe_allows_a_database_without_wal_sidecars() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join(DATABASE_NAME);
        std::fs::write(&database, b"SQLite format 3\0").unwrap();
        check_read_access(&database).expect("a standalone database is readable");

        std::fs::remove_file(&database).unwrap();
        assert_eq!(
            check_read_access(&database).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn unknown_inline_attachments_are_labelled_as_inline_text() {
        assert_eq!(
            plain_label("com.apple.notes.inlinetextattachment.calculateresult"),
            "inline text"
        );
        assert_eq!(plain_label("public.data"), "file");
    }

    /// A writer committing throughout the copy restarted the old incremental
    /// backup each time, so it never finished; a single step always does.
    #[cfg(target_os = "macos")]
    #[test]
    fn snapshot_finishes_while_another_connection_keeps_writing() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let directory = tempfile::tempdir().unwrap();
        let source_path = directory.path().join(DATABASE_NAME);
        let writer = Connection::open(&source_path).unwrap();
        writer
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE filler (id INTEGER PRIMARY KEY, data BLOB);
                 WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 2000)
                 INSERT INTO filler (data) SELECT randomblob(1024) FROM n;",
            )
            .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let writing = {
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                writer.busy_timeout(Duration::from_secs(1)).unwrap();
                while !stop.load(Ordering::Relaxed) {
                    let _ = writer.execute("INSERT INTO filler (data) VALUES (randomblob(64))", []);
                }
            })
        };

        let snapshot_path = directory.path().join("snapshot.sqlite");
        let result = snapshot_database(&source_path, &snapshot_path);
        stop.store(true, Ordering::Relaxed);
        writing.join().unwrap();
        result.unwrap();
        let rows: i64 = Connection::open(&snapshot_path)
            .unwrap()
            .query_row("SELECT count(*) FROM filler", [], |row| row.get(0))
            .unwrap();
        assert!(rows >= 2000, "{rows}");
    }
}
