//! Themes on the phone: the built-in ones, the person's own and the
//! gallery's, fitted as the desktop fits them.
//!
//! A library keeps its themes beside its database file: the person's own in
//! `themes/`, the gallery's in `gallery/themes/` (the gallery's folder, as
//! `gallery_store` finds it). They are loaded as the desktop's Themes page
//! loads them: built-in themes first, in the desktop's order, then the
//! person's own by name; a gallery theme comes last, and is left out when a
//! theme of the same name is already there or the gallery has disabled it. A
//! file that can't be read is a notice, never a row.
//!
//! What is read is kept until a file changes, so highlighting a note in a
//! theme doesn't read the folder at every keystroke.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::time::SystemTime;

use kanso_engine::theme::Tokens;
use kanso_engine::theme::builtin::{BUILTIN, id_of};
use kanso_engine::theme::file::{ResolvedTheme, ThemeFile};
use kanso_engine::theme::import;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::model::{StyledRun, ThemeTokens};
use crate::renderer;
use crate::theme_tokens;
use crate::themes_model::{ThemeImport, ThemeNotice, ThemeOrigin, ThemeRow};

/// The folder of the person's own themes, in the library's folder.
const OWN_FOLDER: &str = "themes";
/// The gallery's folder, in the library's folder.
const GALLERY_FOLDER: &str = "gallery";

/// The themes of one library.
#[derive(Clone)]
pub struct Themes {
    pub(crate) inner: Arc<Inner>,
}

/// One theme, read.
pub(crate) struct Entry {
    pub(crate) id: String,
    file: ThemeFile,
    pub(crate) resolved: ResolvedTheme,
    origin: ThemeOrigin,
    /// The theme fitted alone, made the first time a row needs it.
    alone: OnceLock<ThemeTokens>,
}

impl Entry {
    fn new(id: String, file: ThemeFile, origin: ThemeOrigin) -> Entry {
        let resolved = file.resolve();
        Entry {
            id,
            file,
            resolved,
            origin,
            alone: OnceLock::new(),
        }
    }

    fn name(&self) -> &str {
        &self.file.name
    }

    fn row(&self) -> ThemeRow {
        ThemeRow {
            id: self.id.clone(),
            name: self.file.name.clone(),
            author: self.file.author.clone(),
            license: self.file.license.clone(),
            source: self.file.source.clone(),
            dark: self.resolved.dark,
            origin: self.origin,
            tokens: self
                .alone
                .get_or_init(|| theme_tokens::fit_file(&self.file).0)
                .clone(),
        }
    }
}

/// Every theme of a library and the files that couldn't be used.
pub(crate) struct Loaded {
    entries: Vec<Entry>,
    notices: Vec<ThemeNotice>,
}

impl Loaded {
    pub(crate) fn get(&self, id: &str) -> Result<&Entry, KansoError> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .ok_or(KansoError::NotFound)
    }
}

/// A folder's `.json` files with their sizes and times, sorted by name: what
/// tells that the folder changed.
type Signature = Vec<(String, u64, Option<SystemTime>)>;

fn json_files(dir: &Path) -> Signature {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Signature = read
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            meta.is_file().then(|| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    meta.len(),
                    meta.modified().ok(),
                )
            })
        })
        .collect();
    files.sort();
    files
}

/// The folder a library's files live in: the database file's.
pub(crate) fn folder_of(database: &Path) -> PathBuf {
    match database.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// The built-in themes, read once.
fn builtin_entries() -> &'static [(String, ThemeFile)] {
    static BUILT: OnceLock<Vec<(String, ThemeFile)>> = OnceLock::new();
    BUILT.get_or_init(|| {
        BUILTIN
            .iter()
            .filter_map(|(file, text)| {
                ThemeFile::parse(text)
                    .ok()
                    .map(|theme| (id_of(file).to_owned(), theme))
            })
            .collect()
    })
}

/// `stem`, or, when another theme has it already, `stem` after the way the
/// theme came: ids are unique among the rows.
fn unique(taken: &mut HashSet<String>, stem: &str, origin: ThemeOrigin) -> String {
    let mut id = stem.to_owned();
    if taken.contains(&id) {
        let prefix = match origin {
            ThemeOrigin::Builtin => "builtin",
            ThemeOrigin::Own => "own",
            ThemeOrigin::Gallery => "gallery",
        };
        id = format!("{prefix}-{stem}");
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{prefix}-{stem}-{n}");
            n += 1;
        }
    }
    taken.insert(id.clone());
    id
}

/// Reads the themes of the library whose files are in `folder`.
fn load(folder: &Path) -> Loaded {
    let mut taken = HashSet::new();
    let mut entries: Vec<Entry> = builtin_entries()
        .iter()
        .map(|(id, file)| {
            taken.insert(id.clone());
            Entry::new(id.clone(), file.clone(), ThemeOrigin::Builtin)
        })
        .collect();
    let mut notices = Vec::new();
    let mut notice = |file: &str, message: &str| {
        notices.push(ThemeNotice {
            file: file.to_owned(),
            sentence: format!("{file} {}.", message.trim_end_matches('.')),
        });
    };

    let own_dir = folder.join(OWN_FOLDER);
    let mut own: Vec<Entry> = Vec::new();
    for (name, _, _) in json_files(&own_dir) {
        let parsed = std::fs::read_to_string(own_dir.join(&name))
            .map_err(|err| format!("couldn't be read: {err}"))
            .and_then(|text| ThemeFile::parse(&text));
        match parsed {
            Ok(theme) => {
                let named = |e: &Entry| e.name().eq_ignore_ascii_case(&theme.name);
                if entries.iter().chain(own.iter()).any(named) {
                    notice(
                        &name,
                        &format!(
                            "is named \"{}\", like another theme; give it a name of its own",
                            theme.name
                        ),
                    );
                } else {
                    own.push(Entry::new(
                        name.trim_end_matches(".json").to_owned(),
                        theme,
                        ThemeOrigin::Own,
                    ));
                }
            }
            Err(message) => notice(&name, &message),
        }
    }
    own.sort_by_key(|e| e.name().to_lowercase());
    for mut entry in own {
        entry.id = unique(&mut taken, &entry.id.clone(), ThemeOrigin::Own);
        entries.push(entry);
    }

    // The gallery's themes load after the person's own, so a file of their
    // own with a gallery theme's name wins.
    let gallery = folder.join(GALLERY_FOLDER);
    let disabled: HashSet<String> = kanso_gallery::State::load(&gallery.join("state.json"))
        .disabled
        .into_iter()
        .collect();
    let gallery_dir = gallery.join("themes");
    for (name, _, _) in json_files(&gallery_dir) {
        let stem = name.trim_end_matches(".json");
        if disabled.contains(stem) || !kanso_gallery::is_valid_name(stem) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(gallery_dir.join(&name)) else {
            continue;
        };
        let Ok(theme) = ThemeFile::parse(&text) else {
            continue;
        };
        if entries
            .iter()
            .any(|e| e.name().eq_ignore_ascii_case(&theme.name))
        {
            continue;
        }
        let id = unique(&mut taken, stem, ThemeOrigin::Gallery);
        entries.push(Entry::new(id, theme, ThemeOrigin::Gallery));
    }
    Loaded { entries, notices }
}

/// What was read for each folder, and the files' signature then.
type CachedThemes = (Signature, Arc<Loaded>);
type ThemeCache = Mutex<HashMap<PathBuf, CachedThemes>>;

static KEPT: LazyLock<ThemeCache> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// The themes of the library whose files are in `folder`, read again only if
/// a file changed.
pub(crate) fn loaded(folder: &Path) -> Arc<Loaded> {
    let gallery = folder.join(GALLERY_FOLDER);
    let mut now: Signature = Vec::new();
    for (dir, prefix) in [
        (folder.join(OWN_FOLDER), ""),
        (gallery.join("themes"), "gallery/"),
        (gallery.clone(), "gallery-state/"),
    ] {
        for (name, len, time) in json_files(&dir) {
            // Only the gallery's state file matters of its folder.
            if prefix == "gallery-state/" && name != "state.json" {
                continue;
            }
            now.push((format!("{prefix}{name}"), len, time));
        }
    }
    let mut kept = held(&KEPT);
    if let Some((then, found)) = kept.get(folder)
        && *then == now
    {
        return found.clone();
    }
    let fresh = Arc::new(load(folder));
    kept.insert(folder.to_path_buf(), (now, fresh.clone()));
    fresh
}

impl Themes {
    fn folder(&self) -> PathBuf {
        folder_of(&self.inner.path)
    }

    /// Every theme: the built-in ones in the desktop's order, then the
    /// person's own by name, then the gallery's.
    pub fn list(&self) -> Result<Vec<ThemeRow>, KansoError> {
        guard(|| {
            Ok(loaded(&self.folder())
                .entries
                .iter()
                .map(Entry::row)
                .collect())
        })
    }

    /// The theme files that couldn't be used, each as a sentence.
    pub fn notices(&self) -> Result<Vec<ThemeNotice>, KansoError> {
        guard(|| Ok(loaded(&self.folder()).notices.clone()))
    }

    /// The themes `ui`, `syntax` and `preview` name (IDs from [`Themes::list`])
    /// fitted together, as the desktop fits the three it has in use: `ui`
    /// colors the interface, `syntax` the editor and `preview` the rendered
    /// note. `None` is the `ui` theme. [`KansoError::NotFound`] for an ID that
    /// isn't a theme.
    pub fn fit(
        &self,
        ui: String,
        syntax: Option<String>,
        preview: Option<String>,
    ) -> Result<ThemeTokens, KansoError> {
        guard(|| {
            let fitted = self.fitted(&ui, syntax.as_deref(), preview.as_deref())?;
            Ok(theme_tokens::rows(&fitted))
        })
    }

    fn fitted(
        &self,
        ui: &str,
        syntax: Option<&str>,
        preview: Option<&str>,
    ) -> Result<Tokens, KansoError> {
        let loaded = loaded(&self.folder());
        let ui = loaded.get(ui)?;
        let syntax = syntax.map_or(Ok(ui), |id| loaded.get(id))?;
        let preview = preview.map_or(Ok(ui), |id| loaded.get(id))?;
        Ok(Tokens::fit(
            &ui.resolved,
            &syntax.resolved,
            &preview.resolved,
        ))
    }

    /// The runs of `text` between `start` and `end` (UTF-16 units) that the
    /// syntax of the fitted pair `ui` and `syntax` styles; see
    /// [`crate::Renderer::styled_runs`]. `None` is the `ui` theme.
    pub fn styled_runs(
        &self,
        text: String,
        start: u32,
        end: u32,
        ui: String,
        syntax: Option<String>,
    ) -> Result<Vec<StyledRun>, KansoError> {
        guard(|| {
            let fitted = self.fitted(&ui, syntax.as_deref(), None)?;
            Ok(renderer::styled_runs_in(&text, start, end, &fitted.syntax))
        })
    }

    /// Imports the theme at `path` into the library's themes folder: a theme
    /// package (its folder or `.tgz`), a Zed theme family (`.json`) or a Kanso
    /// theme file. [`KansoError::Invalid`] says why it couldn't be, in a
    /// sentence.
    pub fn import(&self, path: String) -> Result<ThemeImport, KansoError> {
        guard(|| {
            let folder = self.folder();
            let written = import::import_into(Path::new(&path), &folder.join(OWN_FOLDER))
                .map_err(KansoError::invalid)?;
            held(&KEPT).remove(&folder);
            let ids = written
                .iter()
                .map(|w| {
                    w.path
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().into_owned())
                        .unwrap_or_default()
                })
                .collect();
            Ok(ThemeImport {
                ids,
                names: written.into_iter().map(|w| w.name).collect(),
            })
        })
    }
}

/// The syntax styles of a fitted theme, by key.
pub(crate) type Syntax = BTreeMap<&'static str, kanso_engine::theme::file::TokenStyle>;
