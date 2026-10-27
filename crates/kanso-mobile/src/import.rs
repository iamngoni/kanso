//! Importing: another app's export or a folder of Markdown, and opening the
//! files other apps share.
//!
//! The words are the desktop's ([`kanso_import::library::words`]), so both
//! apps say the same thing. An import runs on a thread of its own with a big
//! stack (converting a deeply nested page recurses) and reports as a stream
//! of [`ImportEvent`]s: some `Progress`, then exactly one `Done` or one
//! `Failed`. Notes it writes reach [`Library::changes`] like any write.

use std::path::{Path, PathBuf};

use kanso_import::library::words::{picks, progress_label, rejected_lines, skipped_lines, summary};
use kanso_import::library::{
    LibraryPhase, LibraryProgress, LibraryReport, SourceKind, import_source, kinds_for,
    open_note_files, preview, staged_path,
};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{
    ImportEvent, ImportFound, ImportPhase, ImportProgress, ImportSource, ImportSummary,
    OpenedDocuments, OpenedFile,
};

/// The stack the import thread gets, as on the desktop.
const IMPORT_STACK: usize = 16 * 1024 * 1024;

/// Whether a phone can read `kind`: all but the one that reads a Mac's own
/// notes database.
fn on_phones(kind: SourceKind) -> bool {
    kind != SourceKind::AppleNotes
}

/// Where a phone can import from, in the order the picker lists them.
pub fn import_sources() -> Vec<ImportSource> {
    SourceKind::ALL
        .into_iter()
        .filter(|kind| on_phones(*kind))
        .map(|kind| {
            let (picks_files, picks_folders) = picks(kind);
            ImportSource {
                kind: kind.as_str().to_owned(),
                label: kind.label().to_owned(),
                hint: kind.hint().to_owned(),
                picks_files,
                picks_folders,
                extensions: kind
                    .file_extensions()
                    .iter()
                    .map(|e| (*e).to_owned())
                    .collect(),
            }
        })
        .collect()
}

/// What the file or folder at `path` could be, best first, as the names
/// [`Library::import`] takes. Empty when it is none of them.
pub fn import_kinds_for(path: &str) -> Vec<String> {
    kinds_for(Path::new(path))
        .into_iter()
        .map(|kind| kind.as_str().to_owned())
        .collect()
}

fn phase(phase: LibraryPhase) -> ImportPhase {
    match phase {
        LibraryPhase::Reading => ImportPhase::Reading,
        LibraryPhase::Notebooks => ImportPhase::Notebooks,
        LibraryPhase::Notes => ImportPhase::Notes,
    }
}

/// An event for the engine's progress. Pure.
pub(crate) fn progress_event(progress: Option<LibraryProgress>) -> ImportEvent {
    let (label, fraction) = progress_label(progress);
    ImportEvent::Progress(ImportProgress {
        phase: progress.map(|p| phase(p.phase)),
        label,
        done: progress.map_or(0, |p| p.done as u64),
        total: progress.map_or(0, |p| p.total as u64),
        fraction: f64::from(fraction),
    })
}

/// A report in the shell's words. Pure.
pub(crate) fn summarize(kind: SourceKind, report: &LibraryReport) -> ImportSummary {
    ImportSummary {
        kind: report.kind.unwrap_or(kind).as_str().to_owned(),
        dry_run: report.dry_run,
        found: ImportFound {
            notes: report.found.notes as u64,
            notebooks: report.found.notebooks as u64,
            tags: report.found.tags as u64,
            attachments: report.found.attachments as u64,
        },
        empty: report.found.notes == 0,
        lines: summary(report),
        skipped: skipped_lines(report),
        rejected: rejected_lines(report),
        first_note: report.first_note.clone(),
    }
}

impl Library {
    /// Reads the export or folder at `path` as a `kind` (one of
    /// [`import_sources`]' names) and sends what an import would do, writing
    /// nothing: `Progress` events, then one `Done` with `dry_run` set, or
    /// one `Failed`. Returns at once; the work runs on its own thread, and
    /// stops reporting when `sink` returns `false`.
    pub fn import_plan(
        &self,
        kind: &str,
        path: &str,
        sink: impl FnMut(ImportEvent) -> bool + Send + 'static,
    ) {
        self.start_import(true, kind, path, sink);
    }

    /// [`Library::import_plan`], then writes the notes. Importing again
    /// updates what the last import made and never undoes a change made in
    /// Kanso. The source is only read.
    pub fn import(
        &self,
        kind: &str,
        path: &str,
        sink: impl FnMut(ImportEvent) -> bool + Send + 'static,
    ) {
        self.start_import(false, kind, path, sink);
    }

    fn start_import(
        &self,
        dry_run: bool,
        kind: &str,
        path: &str,
        sink: impl FnMut(ImportEvent) -> bool + Send + 'static,
    ) {
        let store = self.inner.store.clone();
        let (kind, path) = (kind.to_owned(), PathBuf::from(path));
        // The sink is shared with the thread's start, in case it can't start.
        let sink = std::sync::Arc::new(std::sync::Mutex::new(sink));
        let thread_sink = sink.clone();
        let spawned = std::thread::Builder::new()
            .name("library-import".into())
            .stack_size(IMPORT_STACK)
            .spawn(move || {
                let mut sink = crate::boundary::held(&thread_sink);
                let mut listening = true;
                let mut send = |event: ImportEvent| {
                    if listening {
                        listening = (*sink)(event);
                    }
                    listening
                };
                let ran = guard(|| {
                    Ok(run(dry_run, &kind, &path, &store, &mut |p| {
                        send(progress_event(p))
                    }))
                });
                let last = match ran {
                    Ok(Ok(summary)) => ImportEvent::Done(summary),
                    Ok(Err(message)) => ImportEvent::Failed(message),
                    Err(err) => ImportEvent::Failed(err.to_string()),
                };
                send(last);
            });
        if let Err(err) = spawned {
            let mut sink = crate::boundary::held(&sink);
            (*sink)(ImportEvent::Failed(format!(
                "could not start the import: {err}"
            )));
        }
    }

    /// Opens the Markdown or text files the system handed over: each is
    /// imported into the Inbox, and the last is the note to show. A document
    /// opened again updates its note.
    ///
    /// A file with an `identity` is a temporary copy: it is copied under
    /// `staging` to a folder named for the identity, so each copy of the
    /// same document is the same file and the same note, and the staged copy
    /// is removed afterwards. A file without one is read where it is.
    pub fn open_documents(
        &self,
        files: Vec<OpenedFile>,
        staging: &str,
    ) -> Result<OpenedDocuments, KansoError> {
        guard(|| {
            let mut paths: Vec<PathBuf> = Vec::new();
            let mut staged: Vec<PathBuf> = Vec::new();
            let mut problems = Vec::new();
            for file in &files {
                let Some(identity) = &file.identity else {
                    paths.push(PathBuf::from(&file.path));
                    continue;
                };
                match stage(Path::new(staging), identity, file) {
                    Ok(copy) => {
                        paths.push(copy.clone());
                        staged.push(copy);
                    }
                    Err(problem) => problems.push(problem),
                }
            }
            let (note_id, mut more) = open_note_files(&paths, &self.inner.store);
            for copy in &staged {
                std::fs::remove_file(copy).ok();
                if let Some(folder) = copy.parent() {
                    std::fs::remove_dir(folder).ok();
                }
            }
            problems.append(&mut more);
            Ok(OpenedDocuments { note_id, problems })
        })
    }
}

/// Copies `file` to its place under `staging`; the path made real, or a
/// sentence for what went wrong.
fn stage(staging: &Path, identity: &str, file: &OpenedFile) -> Result<PathBuf, String> {
    let name = Path::new(&file.name)
        .file_name()
        .or_else(|| Path::new(&file.path).file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let wrong = |err: std::io::Error| format!("{name}: couldn't read it: {err}");
    let target = staged_path(staging, identity, &name);
    std::fs::create_dir_all(target.parent().unwrap_or(staging)).map_err(wrong)?;
    std::fs::copy(&file.path, &target).map_err(wrong)?;
    target.canonicalize().map_err(wrong)
}

/// Runs the plan or the import; the summary, or the sentence the shell shows.
fn run(
    dry_run: bool,
    kind: &str,
    path: &Path,
    store: &kanso_store::Store,
    progress: &mut dyn FnMut(Option<LibraryProgress>) -> bool,
) -> Result<ImportSummary, String> {
    let Some(kind) = SourceKind::parse(kind).filter(|k| on_phones(*k)) else {
        return Err(format!("unknown kind `{kind}`"));
    };
    if !path.exists() {
        return Err(format!("{} doesn't exist", path.display()));
    }
    progress(None);
    let each = |p: LibraryProgress| {
        progress(Some(p));
    };
    let report = if dry_run {
        preview(kind, path, store, each)
    } else {
        import_source(kind, path, store, each)
    }
    .map_err(|err| format!("{err:#}"))?;
    Ok(summarize(kind, &report))
}
