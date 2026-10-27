//! Compiles the built-in icon themes in, but only what the phone draws
//! (`themes/icon-themes`, the kanso-themes submodule; the desktop's
//! `kanso-app/build.rs` compiles all of it, 28 MB). `app-icons.txt` lists the
//! icons the phone's app uses; for each theme this writes `builtin_icons.rs`
//! into `OUT_DIR` with the theme's JSON cut down to those icons and the SVGs
//! they name, as `include_bytes!`s. `src/icons.rs` resolves an icon by the
//! same rule: the library slot's SVG if the theme has one, else the theme's
//! SVG for the icon.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// The icons the phone draws: Lucide name, and the library slot it stands
/// for, if any.
fn app_icons(text: &str) -> Vec<(String, Option<String>)> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next().unwrap_or_default().to_owned();
            (name, words.next().map(str::to_owned))
        })
        .collect()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=app-icons.txt");
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let list =
        std::fs::read_to_string(manifest_dir.join("app-icons.txt")).expect("reading app-icons.txt");
    let icons = app_icons(&list);
    let root = manifest_dir.join("../../themes/icon-themes");
    // The themes are the kanso-themes repository, checked out as a
    // submodule; without it Kanso would build with no icon themes at all.
    assert!(
        root.is_dir(),
        "themes/icon-themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
    );
    println!("cargo:rerun-if-changed={}", root.display());
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let json_dir = out_dir.join("icon-themes");
    std::fs::create_dir_all(&json_dir).expect("creating the icon themes folder");

    let mut code = String::from(
        "/// The built-in icon themes, cut down to the phone's icons: (ID, theme JSON, SVGs by path).\n\
         pub const BUILTIN_ICON_FILES: &[BuiltinIconTheme] = &[\n",
    );
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&root)
        .map(|read| read.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    folders.sort();
    for folder in folders.into_iter().filter(|p| p.is_dir()) {
        let id = folder.file_name().unwrap().to_string_lossy().into_owned();
        // The schema's folder isn't a theme.
        if id == "schema" {
            continue;
        }
        let json = std::fs::read_dir(&folder)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "json"));
        let Some(json) = json else {
            continue;
        };
        println!("cargo:rerun-if-changed={}", json.display());
        let text = std::fs::read_to_string(&json).expect("reading a theme file");
        let Ok(Value::Object(file)) = serde_json::from_str::<Value>(&text) else {
            panic!("the icon theme {id} isn't a JSON object");
        };
        let (trimmed, paths) = cut_down(&file, &icons);
        let trimmed_path = json_dir.join(format!("{id}.json"));
        std::fs::write(&trimmed_path, serde_json::to_string(&trimmed).unwrap())
            .expect("writing a theme file");
        writeln!(
            code,
            "    ({id:?}, include_str!({:?}), &[",
            trimmed_path.display().to_string()
        )
        .unwrap();
        for rel in paths {
            let path = folder.join(&rel);
            if !is_inside(&folder, &path) {
                println!("cargo:warning=icon theme {id}: {rel} isn't a file in its folder");
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            writeln!(
                code,
                "        ({rel:?}, include_bytes!({:?})),",
                path.display().to_string()
            )
            .unwrap();
        }
        code.push_str("    ]),\n");
    }
    code.push_str("];\n");
    std::fs::write(out_dir.join("builtin_icons.rs"), code).expect("writing builtin_icons.rs");
}

/// The theme `file` with only the icons and slots the phone draws, and the
/// paths of the SVGs those name, in order.
fn cut_down(
    file: &Map<String, Value>,
    icons: &[(String, Option<String>)],
) -> (Map<String, Value>, BTreeSet<String>) {
    let mut out = Map::new();
    for key in [
        "schema_version",
        "name",
        "author",
        "license",
        "source",
        "description",
    ] {
        if let Some(value) = file.get(key) {
            out.insert(key.to_owned(), value.clone());
        }
    }
    let mut paths = BTreeSet::new();
    let mut kept_icons = Map::new();
    let mut kept_library = Map::new();
    for (name, slot) in icons {
        if let Some(path) = file
            .get("icons")
            .and_then(|map| map.get(name))
            .and_then(Value::as_str)
        {
            kept_icons.insert(name.clone(), path.into());
            paths.insert(path.to_owned());
        }
        if let Some(slot) = slot
            && let Some(path) = file
                .get("library")
                .and_then(|map| map.get(slot))
                .and_then(Value::as_str)
        {
            kept_library.insert(slot.clone(), path.into());
            paths.insert(path.to_owned());
        }
    }
    if !kept_icons.is_empty() {
        out.insert("icons".to_owned(), kept_icons.into());
    }
    if !kept_library.is_empty() {
        out.insert("library".to_owned(), kept_library.into());
    }
    (out, paths)
}

/// Whether `path` is a file under `folder`, links resolved.
fn is_inside(folder: &Path, path: &Path) -> bool {
    match (folder.canonicalize(), path.canonicalize()) {
        (Ok(folder), Ok(path)) => path.starts_with(folder) && path.is_file(),
        _ => false,
    }
}
