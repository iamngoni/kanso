//! Icon themes for one library: the built-ins, installed themes and import.
//!
//! File conversion lives in `kanso-engine::icons::import`; this facade reads
//! the library's own and gallery folders with the same checked reader the
//! desktop uses.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kanso_engine::icons::file::{self, Folder};
use kanso_engine::icons::model::{IconTheme, IconThemeFile, Origin};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::icons;
use crate::themes::Themes;
use crate::themes_icons_model::{IconThemeImport, InstalledIconThemeRow};
use crate::themes_model::ThemeOrigin;

impl Themes {
    /// Every built-in, personal and gallery icon theme, in that order.
    pub fn icon_themes(&self) -> Result<Vec<InstalledIconThemeRow>, KansoError> {
        guard(|| {
            let builtins = icons::all()?;
            let mut rows: Vec<InstalledIconThemeRow> = builtins
                .into_iter()
                .map(|theme| InstalledIconThemeRow {
                    id: theme.id,
                    name: theme.name,
                    author: theme.author,
                    license: theme.license,
                    origin: ThemeOrigin::Builtin,
                    icon_count: theme.icon_count,
                })
                .collect();
            let names: HashSet<String> = rows.iter().map(|row| row.name.to_lowercase()).collect();
            let folder = crate::themes::folder_of(&self.inner.path);
            let mut own = read_folders(&folder.join("icon-themes"), Origin::Own, ThemeOrigin::Own);
            let mut gallery = read_folders(
                &folder.join("gallery/icon-themes"),
                Origin::Gallery,
                ThemeOrigin::Gallery,
            );
            own.sort_by_key(|row| row.name.to_lowercase());
            gallery.sort_by_key(|row| row.name.to_lowercase());
            let mut names = names;
            own.retain(|row| names.insert(row.name.to_lowercase()));
            gallery.retain(|row| names.insert(row.name.to_lowercase()));
            rows.extend(own);
            rows.extend(gallery);
            Ok(rows)
        })
    }

    /// Imports an icon theme source into this library's `icon-themes/`
    /// folder, returning the themes written and the importer's report.
    pub fn import_icon_theme(&self, path: String) -> Result<IconThemeImport, KansoError> {
        guard(|| {
            let builtins = icons::all()?;
            let known_icons: HashSet<String> = icons::app_icons()
                .into_iter()
                .map(|icon| icon.name.to_owned())
                .collect();
            let known_icon = |name: &str| known_icons.contains(name);
            let builtin_name = |name: &str| {
                builtins
                    .iter()
                    .any(|theme| theme.name.eq_ignore_ascii_case(name))
            };
            let builtin_id = |id: &str| builtins.iter().any(|theme| theme.id == id);
            let folder = crate::themes::folder_of(&self.inner.path).join("icon-themes");
            let imported = kanso_engine::icons::import::import_path(
                Path::new(&path),
                &folder,
                &known_icon,
                &builtin_name,
                &builtin_id,
            )
            .map_err(KansoError::invalid)?;
            Ok(IconThemeImport {
                ids: imported
                    .reports
                    .iter()
                    .filter_map(|report| {
                        report
                            .folder
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                    })
                    .collect(),
                names: imported
                    .reports
                    .iter()
                    .map(|report| report.name.clone())
                    .collect(),
                summary: imported.summary(),
            })
        })
    }
}

fn read_folders(
    folder: &Path,
    engine_origin: Origin,
    origin: ThemeOrigin,
) -> Vec<InstalledIconThemeRow> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_dir())
        })
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| read_folder(&path, engine_origin, origin))
        .collect()
}

fn read_folder(
    path: &Path,
    engine_origin: Origin,
    origin: ThemeOrigin,
) -> Option<InstalledIconThemeRow> {
    let id = path.file_name()?.to_string_lossy().into_owned();
    if !file::is_valid_name(&id) {
        return None;
    }
    let mut json: Vec<PathBuf> = std::fs::read_dir(path)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|file| {
            file.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect();
    json.sort();
    let [json] = json.as_slice() else {
        return None;
    };
    let metadata = std::fs::symlink_metadata(json).ok()?;
    if !metadata.file_type().is_file() || metadata.len() > file::MAX_JSON_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(json).ok()?;
    let known_icons: HashSet<String> = icons::app_icons()
        .into_iter()
        .map(|icon| icon.name.to_owned())
        .collect();
    let known_icon = |name: &str| known_icons.contains(name);
    let theme_file = IconThemeFile::parse(&text, &known_icon).ok()?;
    let files = Folder::new(path).ok()?;
    let sanitize = |bytes: &[u8]| kanso_gallery::sanitize_svg(bytes).map_err(|err| err.to_string());
    let mut notices = Vec::new();
    let theme: Arc<IconTheme> = Arc::new(file::load(
        &id,
        &theme_file,
        &files,
        Some(path.to_path_buf()),
        engine_origin,
        Some(&sanitize),
        &mut notices,
    ));
    Some(InstalledIconThemeRow {
        id,
        name: theme.name.clone(),
        author: theme.author.clone(),
        license: theme.license.clone(),
        origin,
        icon_count: theme.count().min(u32::MAX as usize) as u32,
    })
}
