//! The built-in icon themes (`themes/icon-themes/`), as the phone draws
//! them. `build.rs` compiles in, for each theme, only the SVGs for the icons
//! the app uses (`app-icons.txt`); each theme is read once, the first time
//! it is asked for, with the engine's reader (`kanso_engine::icons`) the
//! desktop reads them with, and kept.
//!
//! An icon a theme leaves out is Kanso's own Lucide icon, icon by icon: the
//! answer is `None` and the shell draws the Lucide SVG it bundles.

use std::sync::{Arc, LazyLock};

use kanso_engine::icons::file::{Files, load};
use kanso_engine::icons::model::{IconTheme, IconThemeFile, Origin, Slot};

use crate::errors::KansoError;
use crate::model::{IconThemeRow, ThemedIcon};

/// A built-in icon theme as compiled in: its ID, its JSON and its SVGs by path.
type BuiltinIconTheme = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static [u8])],
);

include!(concat!(env!("OUT_DIR"), "/builtin_icons.rs"));

/// The icons the phone draws and the library slots they stand for, one per
/// line (see the file).
const APP_ICONS: &str = include_str!("../app-icons.txt");

/// The theme drawn when none is chosen: Kanso's own Lucide icons.
pub(crate) const DEFAULT_ID: &str = "lucide";

/// The icons that show what a theme looks like, in order.
pub(crate) const PREVIEW: [&str; 5] = ["notebook", "file-text", "pin", "tag", "trash"];

/// An icon the phone draws: its Lucide name and the library slot it stands
/// for, if it stands for one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppIcon {
    pub name: &'static str,
    pub slot: Option<Slot>,
}

/// Every icon the phone draws, in the file's order. Pure.
pub(crate) fn app_icons() -> Vec<AppIcon> {
    APP_ICONS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next().unwrap_or_default();
            let slot = words.next().and_then(Slot::parse);
            AppIcon { name, slot }
        })
        .collect()
}

/// What `theme` draws for `icon`: the SVG of the library slot the icon
/// stands for if the theme draws that slot, else its SVG for the icon;
/// `None` when it draws neither, and Lucide's icon shows. Pure.
pub(crate) fn resolve(theme: &IconTheme, icon: AppIcon) -> Option<&Arc<[u8]>> {
    icon.slot
        .and_then(|slot| theme.library.get(&slot))
        .or_else(|| theme.icons.get(icon.name))
}

/// A built-in theme as read: the engine's model, kept for the life of the app.
static THEMES: LazyLock<Vec<Arc<IconTheme>>> = LazyLock::new(|| {
    let mut themes: Vec<Arc<IconTheme>> = BUILTIN_ICON_FILES
        .iter()
        .filter_map(|(id, json, files)| {
            let file = IconThemeFile::parse(json, &|_| true)
                .map_err(|err| eprintln!("kanso: the built-in icon theme {id} {err}"))
                .ok()?;
            let theme = load(
                id,
                &file,
                &Files(files),
                None,
                Origin::Builtin,
                None,
                &mut Vec::new(),
            );
            Some(Arc::new(theme))
        })
        .collect();
    // The default first, then by name, as the desktop's Settings lists them.
    themes.sort_by_key(|theme| (theme.id != DEFAULT_ID, theme.name.to_lowercase()));
    themes
});

fn themed(name: &str, svg: Option<&Arc<[u8]>>) -> ThemedIcon {
    ThemedIcon {
        name: name.to_owned(),
        svg: svg.map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
    }
}

fn get(id: &str) -> Result<Arc<IconTheme>, KansoError> {
    THEMES
        .iter()
        .find(|theme| theme.id == id)
        .cloned()
        .ok_or(KansoError::NotFound)
}

fn row(theme: &IconTheme) -> IconThemeRow {
    let preview = PREVIEW
        .iter()
        .map(|name| {
            let icon = app_icons()
                .into_iter()
                .find(|icon| icon.name == *name)
                .unwrap_or(AppIcon { name, slot: None });
            themed(name, resolve(theme, icon))
        })
        .collect();
    IconThemeRow {
        id: theme.id.clone(),
        name: theme.name.clone(),
        author: theme.author.clone(),
        license: theme.license.clone(),
        source: theme.source.clone(),
        description: theme.description.clone(),
        icon_count: theme.count() as u32,
        preview,
    }
}

/// Every built-in icon theme, Lucide first, then by name.
pub(crate) fn all() -> Result<Vec<IconThemeRow>, KansoError> {
    Ok(THEMES.iter().map(|theme| row(theme)).collect())
}

/// Every icon the phone draws, as theme `id` draws it. [`KansoError::NotFound`]
/// for an ID that isn't a built-in theme.
pub(crate) fn icons_of(id: &str) -> Result<Vec<ThemedIcon>, KansoError> {
    let theme = get(id)?;
    Ok(app_icons()
        .into_iter()
        .map(|icon| themed(icon.name, resolve(&theme, icon)))
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use proptest::prelude::*;

    use super::*;

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The Lucide names `KansoIcons` in the Flutter app lists.
    fn dart_names() -> BTreeSet<String> {
        let text = std::fs::read_to_string(repo().join("mobile/lib/core/configs/kanso_icons.dart"))
            .expect("reading kanso_icons.dart");
        text.lines()
            .filter_map(|line| {
                let line = line.trim();
                let start = line.find("('")? + 2;
                let end = line[start..].find("')")? + start;
                // An enum value: `name('lucide-name'),` or the last with `;`.
                let tail = &line[end + 2..];
                (tail == "," || tail == ";").then(|| line[start..end].to_owned())
            })
            .collect()
    }

    fn svgs(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                svgs(&path, &format!("{prefix}{name}/"), out);
            } else if name.ends_with(".svg") {
                out.push((format!("{prefix}{name}"), std::fs::read(&path).unwrap()));
            }
        }
    }

    /// Every theme in the repository, whole, as the desktop reads it:
    /// (ID, theme). Read once.
    fn whole() -> &'static Vec<(String, IconTheme)> {
        static WHOLE: LazyLock<Vec<(String, IconTheme)>> = LazyLock::new(|| {
            let root = repo().join("themes/icon-themes");
            let mut folders: Vec<_> = std::fs::read_dir(root)
                .unwrap()
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n != "schema"))
                .collect();
            folders.sort();
            folders
                .into_iter()
                .filter_map(|folder| {
                    let id = folder.file_name()?.to_string_lossy().into_owned();
                    let json = std::fs::read_dir(&folder)
                        .ok()?
                        .filter_map(Result::ok)
                        .map(|e| e.path())
                        .find(|p| p.extension().is_some_and(|x| x == "json"))?;
                    let file =
                        IconThemeFile::parse(&std::fs::read_to_string(json).unwrap(), &|_| true)
                            .unwrap_or_else(|err| panic!("{id} {err}"));
                    let mut found = Vec::new();
                    svgs(&folder, "", &mut found);
                    let table: Vec<(&str, &[u8])> = found
                        .iter()
                        .map(|(n, b)| (n.as_str(), b.as_slice()))
                        .collect();
                    let theme = load(
                        &id,
                        &file,
                        &Files(&table),
                        None,
                        Origin::Builtin,
                        None,
                        &mut Vec::new(),
                    );
                    Some((id, theme))
                })
                .collect()
        });
        &WHOLE
    }

    /// The list is the Flutter app's: every `KansoIcons` name is here and
    /// nothing else, and each has its Lucide SVG bundled for the icons a
    /// theme leaves out.
    // no-prop-test: a fixed file read against a fixed list.
    #[test]
    fn the_list_is_the_flutter_apps_icons() {
        let ours: Vec<&str> = app_icons().iter().map(|icon| icon.name).collect();
        let unique: BTreeSet<&str> = ours.iter().copied().collect();
        assert_eq!(unique.len(), ours.len(), "a name is listed twice");
        let dart = dart_names();
        assert!(dart.len() > 50, "found only {} names in Dart", dart.len());
        let ours: BTreeSet<String> = ours.into_iter().map(str::to_owned).collect();
        assert_eq!(
            ours.difference(&dart).collect::<Vec<_>>(),
            Vec::<&String>::new(),
            "listed here but not in kanso_icons.dart"
        );
        assert_eq!(
            dart.difference(&ours).collect::<Vec<_>>(),
            Vec::<&String>::new(),
            "in kanso_icons.dart but not listed in app-icons.txt"
        );
        for name in &ours {
            let svg = repo().join(format!("mobile/assets/icons/lucide/{name}.svg"));
            assert!(svg.is_file(), "no Lucide SVG bundled for {name}");
        }
        // Every slot named in the list is one the engine knows.
        for line in APP_ICONS.lines().filter(|l| !l.starts_with('#')) {
            if let Some(slot) = line.split_whitespace().nth(1) {
                assert!(Slot::parse(slot).is_some(), "{line}");
            }
        }
        // The preview strip is made of icons the app draws.
        for name in PREVIEW {
            assert!(ours.contains(name), "{name}");
        }
    }

    /// Lucide first, then by name; every folder of the repository is there.
    // no-prop-test: the shipped themes, all of them.
    #[test]
    fn every_shipped_theme_is_listed_lucide_first() {
        let rows = all().unwrap();
        assert_eq!(rows[0].id, DEFAULT_ID);
        assert_eq!(rows[0].name, "Lucide");
        let rest: Vec<String> = rows[1..].iter().map(|r| r.name.to_lowercase()).collect();
        let mut sorted = rest.clone();
        sorted.sort();
        assert_eq!(rest, sorted);
        let shipped: BTreeSet<String> = whole().iter().map(|(id, _)| id.clone()).collect();
        let listed: BTreeSet<String> = rows.iter().map(|r| r.id.clone()).collect();
        assert_eq!(listed, shipped);
        assert!(rows.len() >= 20);
        for row in &rows {
            assert!(!row.name.trim().is_empty(), "{}", row.id);
            let names: Vec<&str> = row.preview.iter().map(|i| i.name.as_str()).collect();
            assert_eq!(names, PREVIEW, "{}", row.id);
        }
        // Lucide leaves everything out; a theme with icons of its own shows some.
        assert!(rows[0].preview.iter().all(|icon| icon.svg.is_none()));
        let phosphor = rows.iter().find(|r| r.id == "phosphor").unwrap();
        assert!(phosphor.preview.iter().any(|icon| icon.svg.is_some()));
        assert_eq!(phosphor.license, "MIT");
    }

    /// Every theme answers for every icon the app draws, in the list's
    /// order, and what it draws is an SVG.
    // no-prop-test: the shipped themes, all of them.
    #[test]
    fn every_theme_answers_for_every_icon_with_an_svg_or_lucide() {
        let names: Vec<&str> = app_icons().iter().map(|icon| icon.name).collect();
        for row in all().unwrap() {
            let icons = icons_of(&row.id).unwrap();
            let got: Vec<&str> = icons.iter().map(|i| i.name.as_str()).collect();
            assert_eq!(got, names, "{}", row.id);
            for icon in &icons {
                if let Some(svg) = &icon.svg {
                    assert!(
                        !svg.trim().is_empty() && svg.contains("<svg"),
                        "{} draws {} as {svg:?}",
                        row.id,
                        icon.name
                    );
                }
            }
            let drawn = icons.iter().filter(|i| i.svg.is_some()).count();
            if row.id != DEFAULT_ID {
                assert!(drawn > 0, "{} draws none of the app's icons", row.id);
            }
        }
    }

    /// The cut-down theme draws exactly what the whole one does, icon by icon:
    /// the same SVG where the theme has one, Lucide where it has none.
    // no-prop-test: the shipped themes, all of them; the property is below.
    #[test]
    fn a_cut_down_theme_resolves_as_the_whole_one_does() {
        for (id, whole) in whole() {
            let cut = get(id).unwrap();
            for icon in app_icons() {
                assert_eq!(
                    resolve(&cut, icon).map(|svg| svg.to_vec()),
                    resolve(whole, icon).map(|svg| svg.to_vec()),
                    "{id}: {}",
                    icon.name
                );
            }
        }
    }

    proptest! {
        /// For any theme and any icon, a slot the theme draws wins over its
        /// icon, which wins over Lucide, and the cut-down theme agrees with
        /// the whole one.
        #[test]
        fn prop_resolution_falls_back_icon_by_icon(theme in 0usize..64, icon in 0usize..512) {
            let (id, full) = &whole()[theme % whole().len()];
            let icons = app_icons();
            let icon = icons[icon % icons.len()];
            let cut = get(id).unwrap();
            let got = resolve(&cut, icon).map(|svg| svg.to_vec());
            prop_assert_eq!(&got, &resolve(full, icon).map(|svg| svg.to_vec()));
            let slot = icon.slot.and_then(|s| full.library.get(&s));
            let own = full.icons.get(icon.name);
            let expected = slot.or(own).map(|svg| svg.to_vec());
            prop_assert_eq!(got, expected);
        }

        /// An ID that isn't a built-in theme is not found, whatever it says.
        #[test]
        fn prop_unknown_ids_are_not_found(id in "\\PC{0,24}") {
            prop_assume!(!THEMES.iter().any(|theme| theme.id == id));
            prop_assert_eq!(icons_of(&id), Err(KansoError::NotFound));
        }
    }
}
