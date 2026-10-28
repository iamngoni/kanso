//! Importing Zed and package icon themes into a Kanso icon-theme folder.
//!
//! This module owns the file conversion shared by desktop and mobile. It
//! reads local input as untrusted, copies only SVGs that stay inside the
//! source folder, and stages each complete theme before replacing a target.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::file::{
    Fingerprint, Folder, MAX_ENTRIES, MAX_JSON_BYTES, MAX_SVG_BYTES, MAX_THEME_BYTES,
    SvgFiles as _, is_valid_name, is_valid_svg_path, looks_like_svg,
};
use super::model::{ExtraEntry, IconThemeFile, Slot};

/// Largest source JSON file converted by an importer.
const MAX_SOURCE_JSON_BYTES: u64 = 8 * 1024 * 1024;
/// Largest extension or package manifest read.
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
/// Largest licence file carried into an imported theme.
const MAX_LICENSE_BYTES: u64 = 64 * 1024;
/// Most licence files carried into one theme.
const MAX_LICENSE_FILES: usize = 8;
/// Largest archive unpacked by an importer.
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
/// Most entries an archive may contain.
const MAX_ARCHIVE_ENTRIES: usize = 20_000;

// MARK: What an import found

/// One icon theme an import wrote, and what of it came across.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub name: String,
    /// The folder it was written to.
    pub folder: PathBuf,
    /// What was carried over, as `source key → Kanso key`.
    pub mapped: Vec<String>,
    /// What wasn't, with why, as `source key: reason`.
    pub left_out: Vec<String>,
}

impl Report {
    /// One sentence for a notification.
    pub fn summary(&self) -> String {
        format!(
            "Imported {}: {} mapped, {} left to Kanso's own icons.",
            self.name,
            count(self.mapped.len(), "icon"),
            count(self.left_out.len(), "other"),
        )
    }

    /// The summary, then each icon mapped and left out, for the command.
    pub fn details(&self) -> String {
        let mut text = format!("Wrote {} ({})", self.folder.display(), self.name);
        for line in &self.mapped {
            text.push_str(&format!("\n  mapped    {line}"));
        }
        for line in &self.left_out {
            text.push_str(&format!("\n  left out  {line}"));
        }
        text
    }
}

/// What one import wrote, and the themes in the source it didn't, with why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imports {
    pub reports: Vec<Report>,
    /// A sentence for each theme not written: one with no icon Kanso can
    /// use, or a variant drawing the same shapes as one written.
    pub skipped: Vec<String>,
}

impl Imports {
    /// Sentences for a notification.
    pub fn summary(&self) -> String {
        self.reports
            .iter()
            .map(Report::summary)
            .chain(self.skipped.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn count(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what}"),
        n => format!("{n} {what}s"),
    }
}

/// An icon theme read from a source, before it is written: its file, and
/// for each path it names the file that path is copied from.
struct Imported {
    file: IconThemeFile,
    /// The folder the SVGs are read from, all inside it.
    root: PathBuf,
    /// Each path in `file` with the path inside `root` it comes from.
    sources: BTreeMap<String, String>,
    /// The licence files to carry over, inside `root`.
    license_files: Vec<PathBuf>,
    /// The folder name a Kanso icon theme had, to keep.
    id: Option<String>,
    mapped: Vec<String>,
    left_out: Vec<String>,
}

// MARK: Importing

/// Converts the icon themes at `source` into `dir`, the icon themes
/// folder, and says what each brought.
// no-prop-test: reads and writes files; `prop_import_stays_in_its_folder`
// runs it over malformed sources, and the example tests over each kind.
pub fn import_path(
    source: &Path,
    dir: &Path,
    known_icon: &dyn Fn(&str) -> bool,
    is_builtin_name: &dyn Fn(&str) -> bool,
    is_builtin_id: &dyn Fn(&str) -> bool,
) -> Result<Imports, String> {
    if !source.exists() {
        return Err(format!("{} doesn't exist.", source.display()));
    }
    let file_name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if source.is_file() && (file_name.ends_with(".tgz") || file_name.ends_with(".tar.gz")) {
        return with_unpacked(source, |root| {
            let root = package_root(root);
            import_themes(
                read_folder_source(&root, known_icon)?,
                dir,
                is_builtin_name,
                is_builtin_id,
                known_icon,
            )
        });
    }
    let themes = if source.is_dir() {
        read_folder_source(source, known_icon)?
    } else if file_name.ends_with(".json") {
        read_json_source(source, known_icon)?
    } else {
        return Err("Kanso imports a Zed icon theme (.json, or its extension's folder or .tar.gz), a theme package carrying one, or a Kanso icon theme folder; this is none of them.".into());
    };
    import_themes(themes, dir, is_builtin_name, is_builtin_id, known_icon)
}

/// Writes each theme, but for a variant drawing the same shapes as one
/// before it, and one with no icon Kanso can use; an error when none is
/// written, so nothing at all is.
fn import_themes(
    themes: Vec<Imported>,
    dir: &Path,
    is_builtin_name: &dyn Fn(&str) -> bool,
    is_builtin_id: &dyn Fn(&str) -> bool,
    known_icon: &dyn Fn(&str) -> bool,
) -> Result<Imports, String> {
    if themes.is_empty() {
        return Err("There are no icon themes in it.".into());
    }
    let mut skipped = Vec::new();
    let mut seen: Vec<(u64, String)> = Vec::new();
    let mut unique = Vec::new();
    for theme in themes {
        let shape = shape_signature(&theme);
        if let Some(shape) = shape
            && let Some((_, first)) = seen.iter().find(|(s, _)| *s == shape)
        {
            skipped.push(format!(
                "{} draws the same shapes as {first}, and Kanso draws icons in one color, so only {first} was imported.",
                theme.file.name
            ));
            continue;
        }
        if let Some(shape) = shape {
            seen.push((shape, theme.file.name.clone()));
        }
        unique.push(theme);
    }
    std::fs::create_dir_all(dir)
        .map_err(|err| format!("The icon themes folder couldn't be made: {err}."))?;
    let mut reports = Vec::new();
    for theme in unique {
        match write(theme, dir, is_builtin_name, is_builtin_id, known_icon) {
            Ok(report) => reports.push(report),
            Err(err) => skipped.push(err),
        }
    }
    if reports.is_empty() {
        return Err(skipped.join(" "));
    }
    Ok(Imports { reports, skipped })
}

/// A fingerprint of the shapes a theme draws: each icon's key and its SVG
/// with colors and spacing taken out ([`svg_shape`]), so variants that
/// differ only in color match. `None` when it draws nothing or its folder
/// can't be read.
fn shape_signature(theme: &Imported) -> Option<u64> {
    let folder = Folder::new(&theme.root).ok()?;
    let file = &theme.file;
    let mut print = Fingerprint::default();
    let entries = file
        .icons
        .iter()
        .map(|(k, p)| (format!("icons.{k}"), p))
        .chain(
            file.library
                .iter()
                .map(|(k, p)| (format!("library.{}", k.key()), p)),
        )
        .chain(
            file.notebook_icons
                .iter()
                .map(|(k, e)| (format!("notebook_icons.{k}"), &e.file)),
        )
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return None;
    }
    for (key, dest) in entries {
        let from = theme.sources.get(dest).unwrap_or(dest);
        print.field(key.as_bytes());
        match folder.read(from, MAX_SVG_BYTES) {
            Ok(bytes) => print.field(&svg_shape(&bytes)),
            Err(_) => print.field(b"\0unreadable"),
        }
    }
    Some(print.finish())
}

/// An SVG with each color (`#` and hex digits, `rgb(…)`, `hsl(…)`) made
/// one `#` and each run of spacing one space: what Kanso, drawing in one
/// color, shows of it.
/// Pure.
pub fn svg_shape(svg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(svg.len());
    let mut ix = 0;
    while ix < svg.len() {
        let rest = &svg[ix..];
        if rest[0] == b'#' {
            ix += 1;
            while ix < svg.len() && svg[ix].is_ascii_hexdigit() {
                ix += 1;
            }
            out.push(b'#');
            continue;
        }
        let function = [&b"rgba("[..], b"rgb(", b"hsla(", b"hsl("]
            .into_iter()
            .find(|f| rest.len() >= f.len() && rest[..f.len()].eq_ignore_ascii_case(f));
        if let Some(function) = function {
            ix += function.len();
            while ix < svg.len() && svg[ix] != b')' {
                ix += 1;
            }
            ix += 1;
            out.push(b'#');
            continue;
        }
        if rest[0].is_ascii_whitespace() {
            // A run of spacing reads as one space.
            if out.last() != Some(&b' ') {
                out.push(b' ');
            }
        } else {
            out.push(rest[0]);
        }
        ix += 1;
    }
    out
}

/// The icon themes in a folder: a Zed extension's, a theme package's, or a
/// Kanso icon theme.
fn read_folder_source(
    source: &Path,
    known_icon: &dyn Fn(&str) -> bool,
) -> Result<Vec<Imported>, String> {
    let zed = source.join("icon_themes");
    if zed.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&zed)
            .map_err(|err| format!("Its icon_themes folder couldn't be read: {err}."))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        let mut all = Vec::new();
        for file in files {
            all.extend(read_zed_file(&file, source)?);
        }
        return Ok(all);
    }
    if source.join("package.json").is_file() {
        return read_package(source, known_icon);
    }
    read_kanso_folder(source, known_icon).map(|theme| vec![theme])
}

/// A `.json` file: a Zed icon theme, whose SVGs are its extension's (the
/// folder above `icon_themes/`), or a Kanso icon theme, whose are its
/// folder's.
fn read_json_source(
    path: &Path,
    known_icon: &dyn Fn(&str) -> bool,
) -> Result<Vec<Imported>, String> {
    let value = read_json(path, MAX_SOURCE_JSON_BYTES)?;
    let parent = path.parent().unwrap_or(Path::new("."));
    if is_zed_family(&value) {
        let root = if parent.file_name().is_some_and(|n| n == "icon_themes") {
            parent.parent().unwrap_or(parent)
        } else {
            parent
        };
        return read_zed(&value, root);
    }
    read_kanso_folder(parent, known_icon).map(|theme| vec![theme])
}

/// A JSON file of at most `limit` bytes, comments allowed as Zed allows:
/// [`MAX_SOURCE_JSON_BYTES`] for a theme to convert, [`MAX_JSON_BYTES`]
/// for a Kanso theme file.
fn read_json(path: &Path, limit: u64) -> Result<Value, String> {
    let text = read_small(path, limit)?;
    serde_json::from_str(&text)
        .or_else(|_| serde_json::from_str(&crate::theme::import::strip_jsonc(&text)))
        .map_err(|err| format!("It isn't valid JSON: {err}."))
}

/// A text file of at most `limit` bytes.
fn read_small(path: &Path, limit: u64) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|err| format!("It couldn't be read: {err}."))?;
    if meta.len() > limit {
        return Err(format!(
            "{} is larger than {} KB.",
            path.file_name().unwrap_or_default().to_string_lossy(),
            limit / 1024
        ));
    }
    std::fs::read_to_string(path).map_err(|err| format!("It couldn't be read: {err}."))
}

/// A theme package: one carrying a Zed icon theme in `icon_themes/`, or
/// naming one, Zed's or Kanso's, in `package.json`'s `iconTheme`.
fn read_package(dir: &Path, known_icon: &dyn Fn(&str) -> bool) -> Result<Vec<Imported>, String> {
    let package: Value =
        serde_json::from_str(&read_small(&dir.join("package.json"), MAX_MANIFEST_BYTES)?)
            .map_err(|err| format!("Its package.json isn't valid JSON: {err}."))?;
    let named = package
        .get("iconTheme")
        .and_then(Value::as_str)
        .map(|p| p.trim_start_matches("./").to_owned());
    let Some(named) = named.filter(|p| is_valid_json_path(p)) else {
        return Err("This package carries no icon theme Kanso can map: it has no icon_themes folder, and its package.json names no iconTheme.".into());
    };
    let path = inside(dir, &named)
        .ok_or_else(|| "Its iconTheme isn't a file inside the package.".to_owned())?;
    let value = read_json(&path, MAX_SOURCE_JSON_BYTES)?;
    if is_zed_family(&value) {
        // `read_zed` reads the package's own author, license and home.
        return read_zed(&value, dir);
    }
    // A Kanso theme file keeps a Kanso theme file's cap.
    let value = read_json(&path, MAX_JSON_BYTES)?;
    let mut theme = read_kanso_file(&value, path.parent().unwrap_or(dir), None, known_icon)?;
    let manifest = package_manifest(&package);
    if theme.file.license.is_empty() {
        theme.file.license = manifest.license.unwrap_or_default();
    }
    if theme.file.author.is_empty() {
        theme.file.author = manifest.author;
    }
    if theme.file.source.is_none() {
        theme.file.source = manifest.url;
    }
    theme.license_files = license_files(dir);
    Ok(vec![theme])
}

/// Who made a theme, under what license and where it lives, as its
/// extension or package says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Manifest {
    author: String,
    /// An SPDX expression ([`is_spdx`]).
    license: Option<String>,
    /// A web address ([`web_url`]).
    url: Option<String>,
}

/// The manifest of the extension or package in `root`: `package.json`'s,
/// filled in from `extension.toml`'s.
fn manifest(root: &Path) -> Manifest {
    let package = read_small(&root.join("package.json"), MAX_MANIFEST_BYTES)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .map(|package| package_manifest(&package))
        .unwrap_or_default();
    let extension = extension_manifest(root);
    Manifest {
        author: if package.author.is_empty() {
            extension.author
        } else {
            package.author
        },
        license: package.license.or(extension.license),
        url: package.url.or(extension.url),
    }
}

/// `package.json`'s author, license, and repository or homepage. Pure.
fn package_manifest(package: &Value) -> Manifest {
    let license = match package.get("license") {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Object(o)) => o.get("type").and_then(Value::as_str),
        _ => None,
    };
    let repository = match package.get("repository") {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Object(o)) => o.get("url").and_then(Value::as_str),
        _ => None,
    };
    Manifest {
        author: package_author(package),
        license: license.filter(|l| is_spdx(l)).map(|l| l.trim().to_owned()),
        url: repository.and_then(web_url).or_else(|| {
            package
                .get("homepage")
                .and_then(Value::as_str)
                .and_then(web_url)
        }),
    }
}

/// A Zed extension's `extension.toml`: its `authors`, `license` (if it
/// says) and `repository`.
fn extension_manifest(root: &Path) -> Manifest {
    let Some(table) = read_small(&root.join("extension.toml"), MAX_MANIFEST_BYTES)
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
    else {
        return Manifest::default();
    };
    let text = |key: &str| table.get(key).and_then(toml::Value::as_str);
    let authors: Vec<String> = table
        .get("authors")
        .and_then(toml::Value::as_array)
        .map(|authors| {
            authors
                .iter()
                .filter_map(toml::Value::as_str)
                .map(person_name)
                .filter(|name| !name.is_empty())
                .take(8)
                .collect()
        })
        .unwrap_or_default();
    Manifest {
        author: authors.join(", ").chars().take(200).collect(),
        license: text("license")
            .filter(|l| is_spdx(l))
            .map(|l| l.trim().to_owned()),
        url: text("repository").and_then(web_url),
    }
}

/// A person as manifests write one, `Name <email> (site)`, as their name.
/// Pure.
fn person_name(person: &str) -> String {
    person
        .split(['<', '('])
        .next()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(100)
        .collect()
}

/// `text` as a web address, if it is one: `http(s)://` and no spaces, a
/// `git+` in front taken off. Pure.
pub fn web_url(text: &str) -> Option<String> {
    let url = text.trim();
    let url = url.strip_prefix("git+").unwrap_or(url);
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest.split('/').next().unwrap_or_default();
    (!host.is_empty()
        && host.contains('.')
        && url.len() <= 300
        && !url.chars().any(|c| c.is_whitespace() || c.is_control()))
    .then(|| url.to_owned())
}

/// Whether `text` looks like an SPDX license expression (`MIT`,
/// `MIT OR Apache-2.0`). Pure.
pub fn is_spdx(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && text.len() <= 64
        && text.chars().any(|c| c.is_ascii_alphabetic())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " .-+()".contains(c))
}

/// The package's author: a name, or `{"name": …}`.
fn package_author(package: &Value) -> String {
    match package.get("author") {
        Some(Value::String(s)) => s.split('<').next().unwrap_or_default().trim().to_owned(),
        Some(Value::Object(o)) => o
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        _ => String::new(),
    }
}

fn is_valid_json_path(path: &str) -> bool {
    path.strip_suffix(".json")
        .is_some_and(|stem| is_valid_svg_path(&format!("{stem}.svg")))
}

/// `rel` inside `root`, if it is a file there once links are followed.
fn inside(root: &Path, rel: &str) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let full = root.join(rel).canonicalize().ok()?;
    (full.starts_with(&root) && full.is_file()).then_some(full)
}

/// A Kanso icon theme folder.
fn read_kanso_folder(dir: &Path, known_icon: &dyn Fn(&str) -> bool) -> Result<Imported, String> {
    let mut json: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|err| format!("It couldn't be read: {err}."))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    json.sort();
    let [path] = json.as_slice() else {
        return Err("This folder isn't an icon theme Kanso can import: it is neither a Zed extension with icon themes, a theme package, nor a Kanso icon theme folder (one .json and its SVGs).".into());
    };
    let value = read_json(path, MAX_JSON_BYTES)?;
    let id = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| is_valid_name(n));
    read_kanso_file(&value, dir, id, known_icon)
}

/// A Kanso icon theme file, its SVGs in `root`.
fn read_kanso_file(
    value: &Value,
    root: &Path,
    id: Option<String>,
    known_icon: &dyn Fn(&str) -> bool,
) -> Result<Imported, String> {
    let file = IconThemeFile::from_value(value, known_icon)
        .map_err(|err| format!("This icon theme {err}."))?;
    let sources = file
        .icons
        .values()
        .chain(file.library.values())
        .chain(file.notebook_icons.values().map(|e| &e.file))
        .map(|p| (p.clone(), p.clone()))
        .collect();
    let mapped = file
        .icons
        .keys()
        .map(|k| format!("icons.{k}"))
        .chain(file.library.keys().map(|s| format!("library.{}", s.key())))
        .chain(
            file.notebook_icons
                .keys()
                .map(|k| format!("notebook_icons.{k}")),
        )
        .collect();
    Ok(Imported {
        file,
        root: root.to_path_buf(),
        sources,
        license_files: license_files(root),
        id,
        mapped,
        left_out: Vec::new(),
    })
}

/// The licence files in `root` (`LICENSE`, `LICENSE-MIT`, `COPYING.md`…):
/// regular files, not links, small, at most [`MAX_LICENSE_FILES`], in name
/// order.
fn license_files(root: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| is_license_name(&n.to_string_lossy()))
        })
        .filter(|p| {
            std::fs::symlink_metadata(p).is_ok_and(|m| m.is_file() && m.len() <= MAX_LICENSE_BYTES)
        })
        .collect();
    found.sort();
    found.truncate(MAX_LICENSE_FILES);
    found
}

/// Whether a file name is a licence file's, and one safe to write as is:
/// letters, digits, `.`, `-` and `_`, not hidden, not a theme's JSON or an
/// SVG. Pure.
pub fn is_license_name(name: &str) -> bool {
    if name.is_empty()
        || name.len() > 64
        || name.starts_with('.')
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return false;
    }
    let upper = name.to_ascii_uppercase();
    if upper.ends_with(".JSON") || upper.ends_with(".SVG") {
        return false;
    }
    ["LICENSE", "LICENCE", "COPYING", "UNLICENSE"]
        .iter()
        .any(|base| {
            upper
                .strip_prefix(base)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with(['.', '-', '_']))
        })
}

/// The SPDX identifier a license text reads as, for the common ones. Pure.
pub fn spdx_from_text(text: &str) -> Option<&'static str> {
    let head: String = text.chars().take(2000).collect::<String>().to_lowercase();
    [
        ("mit license", "MIT"),
        ("permission is hereby granted, free of charge", "MIT"),
        ("apache license", "Apache-2.0"),
        ("isc license", "ISC"),
        ("mozilla public license", "MPL-2.0"),
        ("creative commons attribution 4.0", "CC-BY-4.0"),
        ("cc0", "CC0-1.0"),
        ("bsd 3-clause", "BSD-3-Clause"),
        (
            "redistribution and use in source and binary forms",
            "BSD-3-Clause",
        ),
        ("gnu general public license", "GPL-3.0"),
    ]
    .into_iter()
    .find(|(words, _)| head.contains(words))
    .map(|(_, id)| id)
}

// MARK: Zed

/// Whether `value` is a Zed icon theme family: `themes`, each with a name
/// and at least one of Zed's icon maps.
pub fn is_zed_family(value: &Value) -> bool {
    value
        .get("themes")
        .and_then(Value::as_array)
        .is_some_and(|themes| {
            themes.iter().any(|t| {
                t.get("name").is_some()
                    && [
                        "directory_icons",
                        "chevron_icons",
                        "file_icons",
                        "file_suffixes",
                        "file_stems",
                        "named_directory_icons",
                    ]
                    .iter()
                    .any(|k| t.get(k).is_some())
            })
        })
}

fn read_zed_file(path: &Path, root: &Path) -> Result<Vec<Imported>, String> {
    let value = read_json(path, MAX_SOURCE_JSON_BYTES)?;
    if !is_zed_family(&value) {
        return Ok(Vec::new());
    }
    read_zed(&value, root)
}

/// The Kanso icon themes in a Zed icon theme family, its SVGs in `root`,
/// the extension's folder.
// no-prop-test: maps over `zed_theme`, which `prop_zed_mapping_is_total`
// covers.
fn read_zed(family: &Value, root: &Path) -> Result<Vec<Imported>, String> {
    let manifest = manifest(root);
    // Who drew the icons, as the extension or package says; the theme
    // file's own `author` is often the format's.
    let author = if manifest.author.is_empty() {
        family
            .get("author")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .chars()
            .take(200)
            .collect()
    } else {
        manifest.author.clone()
    };
    let family_name: String = family
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Zed icon theme")
        .trim()
        .chars()
        .take(80)
        .collect();
    let license_files = license_files(root);
    let license = manifest.license.clone().or_else(|| {
        license_files.iter().find_map(|p| {
            let text = read_small(p, MAX_LICENSE_BYTES).ok()?;
            spdx_from_text(&text).map(str::to_owned)
        })
    });
    let themes = family
        .get("themes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for theme in themes.iter().take(16) {
        let (mut file, sources, mapped, left_out) = zed_theme(theme);
        if file.name.is_empty() {
            continue;
        }
        file.author = author.clone();
        file.license = license.clone().unwrap_or_default();
        file.source = manifest.url.clone();
        file.description = Some(format!("Imported from the Zed icon theme {family_name}."));
        out.push(Imported {
            file,
            root: root.to_path_buf(),
            sources,
            license_files: license_files.clone(),
            id: None,
            mapped,
            left_out,
        });
    }
    Ok(out)
}

/// The names a special notebook may have as a named folder in a Zed theme.
const NAMED: &[(&str, Slot)] = &[
    ("inbox", Slot::Inbox),
    ("journal", Slot::Journal),
    ("journals", Slot::Journal),
    ("diary", Slot::Journal),
    ("daily", Slot::Journal),
    ("templates", Slot::Templates),
    ("template", Slot::Templates),
    ("trash", Slot::Trash),
    (".trash", Slot::Trash),
    ("tags", Slot::Tags),
];

/// The suffixes whose icons stand for an attachment of each kind, in the
/// order they are tried.
const ATTACHMENTS: &[(Slot, &[&str])] = &[
    (Slot::AttachmentPdf, &["pdf"]),
    (
        Slot::AttachmentImage,
        &["png", "jpg", "jpeg", "gif", "webp", "svg"],
    ),
    (
        Slot::AttachmentAudio,
        &["mp3", "m4a", "wav", "ogg", "flac", "aac"],
    ),
];

/// Zed's own suffix → icon key table, for the suffixes in [`ATTACHMENTS`].
/// Zed starts every icon theme's `file_suffixes` from its default theme's
/// and lays the theme's own over them (`ThemeRegistry::load_icon_theme`),
/// then looks the key up in the theme's `file_icons`; so a theme may draw
/// `image` without naming `png`. Taken from `FILE_SUFFIXES_BY_ICON_KEY` in
/// Zed's `crates/theme/src/icon_theme.rs`: PDF is a `document` there.
const ZED_DEFAULT_SUFFIXES: &[(&str, &str)] = &[
    ("pdf", "document"),
    ("png", "image"),
    ("jpg", "image"),
    ("jpeg", "image"),
    ("gif", "image"),
    ("webp", "image"),
    ("svg", "image"),
    ("mp3", "audio"),
    ("m4a", "audio"),
    ("wav", "audio"),
    ("ogg", "audio"),
    ("flac", "audio"),
    ("aac", "audio"),
];

/// A Zed path (`./icons/folder.svg`) as one inside the extension, if it is
/// one Kanso can copy. Pure.
pub fn zed_path(value: &Value) -> Option<String> {
    let mut path = value.as_str()?.trim();
    while let Some(rest) = path.strip_prefix("./") {
        path = rest;
    }
    is_valid_svg_path(path).then(|| path.to_owned())
}

/// One theme of a Zed family as a Kanso theme file (without its author and
/// license), the source of each path, what mapped and what didn't. Pure.
#[allow(clippy::type_complexity)]
pub fn zed_theme(
    theme: &Value,
) -> (
    IconThemeFile,
    BTreeMap<String, String>,
    Vec<String>,
    Vec<String>,
) {
    let mut file = IconThemeFile {
        name: theme
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .chars()
            .take(80)
            .collect(),
        ..Default::default()
    };
    let mut sources = BTreeMap::new();
    let mut mapped = Vec::new();
    let mut left_out = Vec::new();
    let mut take = |from: &str, value: Option<&Value>, slot: Slot, left_out: &mut Vec<String>| {
        let Some(value) = value else {
            return;
        };
        if file.library.contains_key(&slot) {
            return;
        }
        match zed_path(value) {
            Some(path) => {
                // Copied under icons/, as the source names it.
                let dest = format!("icons/{path}").replace("icons/icons/", "icons/");
                if is_valid_svg_path(&dest) {
                    sources.insert(dest.clone(), path);
                    file.library.insert(slot, dest);
                    mapped.push(format!("{from} → library.{}", slot.key()));
                    return;
                }
                left_out.push(format!("{from}: its path is too long"));
            }
            None => left_out.push(format!(
                "{from}: its path isn't an .svg inside the extension"
            )),
        }
    };
    let pair = |key: &str| theme.get(key);
    if let Some(dirs) = pair("directory_icons") {
        take(
            "directory_icons.collapsed",
            dirs.get("collapsed"),
            Slot::Notebook,
            &mut left_out,
        );
        take(
            "directory_icons.expanded",
            dirs.get("expanded"),
            Slot::NotebookOpen,
            &mut left_out,
        );
    }
    if let Some(chevrons) = pair("chevron_icons") {
        take(
            "chevron_icons.collapsed",
            chevrons.get("collapsed"),
            Slot::FoldClosed,
            &mut left_out,
        );
        take(
            "chevron_icons.expanded",
            chevrons.get("expanded"),
            Slot::FoldOpen,
            &mut left_out,
        );
    }
    let file_icons = pair("file_icons").and_then(Value::as_object);
    let icon_path = |key: &str| file_icons.and_then(|icons| icons.get(key)?.get("path"));
    take(
        "file_icons.default",
        icon_path("default"),
        Slot::Note,
        &mut left_out,
    );
    if let Some(named) = pair("named_directory_icons").and_then(Value::as_object) {
        for (name, icons) in named.iter().take(MAX_ENTRIES) {
            let slot = NAMED
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, slot)| *slot);
            match slot {
                Some(slot) => take(
                    &format!("named_directory_icons.{name}"),
                    icons.get("collapsed"),
                    slot,
                    &mut left_out,
                ),
                None => left_out.push(format!(
                    "named_directory_icons.{name}: Kanso has no notebook it stands for"
                )),
            }
        }
    }
    let suffixes = pair("file_suffixes").and_then(Value::as_object);
    for (slot, wanted) in ATTACHMENTS {
        let found = wanted.iter().find_map(|suffix| {
            let own = suffixes.and_then(|s| s.get(*suffix)?.as_str());
            let (from, key) = match own {
                Some(key) => (format!("file_suffixes.{suffix} ({key})"), key),
                None => {
                    let key = ZED_DEFAULT_SUFFIXES
                        .iter()
                        .find(|(s, _)| s == suffix)
                        .map(|(_, key)| *key)?;
                    (
                        format!("file_icons.{key} (Zed's default for .{suffix})"),
                        key,
                    )
                }
            };
            Some((from, icon_path(key)?))
        });
        if let Some((from, path)) = found {
            take(&from, Some(path), *slot, &mut left_out);
        }
    }
    let used: Vec<&str> = ATTACHMENTS
        .iter()
        .flat_map(|(_, s)| s.iter().copied())
        .collect();
    if let Some(suffixes) = suffixes {
        let rest = suffixes
            .keys()
            .filter(|k| !used.contains(&k.as_str()))
            .count();
        if rest > 0 {
            left_out.push(format!(
                "file_suffixes: {} other kinds of file, which notes don't have",
                rest
            ));
        }
    }
    if let Some(stems) = pair("file_stems").and_then(Value::as_object)
        && !stems.is_empty()
    {
        left_out.push(format!(
            "file_stems: {} named files, which notes don't have",
            stems.len()
        ));
    }
    if let Some(icons) = file_icons {
        let rest = icons.keys().filter(|k| *k != "default").count();
        if rest > 0 {
            left_out.push(format!("file_icons: {} icons only those files use", rest));
        }
    }
    (file, sources, mapped, left_out)
}

// MARK: Writing

/// Writes `theme` into `dir`: into a hidden folder first, its used SVGs
/// copied in, then moved into place whole. A folder of the same name
/// holding the same theme (by name) is replaced, so importing again
/// updates it.
fn write(
    mut theme: Imported,
    dir: &Path,
    is_builtin_name: &dyn Fn(&str) -> bool,
    is_builtin_id: &dyn Fn(&str) -> bool,
    known_icon: &dyn Fn(&str) -> bool,
) -> Result<Report, String> {
    let builtin_name = is_builtin_name(&theme.file.name);
    if builtin_name {
        theme.file.name = format!("{} (Imported)", theme.file.name);
    }
    let id = target_id(
        dir,
        theme.id.as_deref(),
        &theme.file.name,
        is_builtin_id,
        known_icon,
    );
    let staging = dir.join(format!(
        ".{id}-importing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    std::fs::create_dir(&staging)
        .map_err(|err| format!("A folder to import it into couldn't be made: {err}."))?;
    let result = fill(&mut theme, &staging, &id);
    let result = result.and_then(|()| {
        let target = dir.join(&id);
        if target.exists() {
            std::fs::remove_dir_all(&target)
                .map_err(|err| format!("{} couldn't be replaced: {err}.", target.display()))?;
        }
        std::fs::rename(&staging, &target)
            .map_err(|err| format!("{} couldn't be written: {err}.", target.display()))?;
        Ok(target)
    });
    match result {
        Ok(folder) => Ok(Report {
            name: theme.file.name,
            folder,
            mapped: theme.mapped,
            left_out: theme.left_out,
        }),
        Err(err) => {
            let _ = std::fs::remove_dir_all(&staging);
            Err(err)
        }
    }
}

/// Copies the SVGs `theme` uses into `folder` and writes its JSON there,
/// leaving out (and saying) any that can't be used.
fn fill(theme: &mut Imported, folder: &Path, id: &str) -> Result<(), String> {
    let source = Folder::new(&theme.root).map_err(|err| format!("Its folder {err}."))?;
    let mut copied: BTreeMap<String, bool> = BTreeMap::new();
    let mut spent = 0u64;
    let mut copy = |dest: &str, left_out: &mut Vec<String>| -> bool {
        if let Some(done) = copied.get(dest) {
            return *done;
        }
        let from = theme
            .sources
            .get(dest)
            .cloned()
            .unwrap_or_else(|| dest.to_owned());
        let ok = match source.read(&from, MAX_SVG_BYTES) {
            Ok(bytes) if !looks_like_svg(&bytes) => {
                left_out.push(format!("{from}: it isn't an SVG"));
                false
            }
            Ok(bytes) if spent + bytes.len() as u64 > MAX_THEME_BYTES => {
                left_out.push(format!(
                    "{from}: the theme's icons pass the size Kanso reads"
                ));
                false
            }
            Ok(bytes) => {
                let bytes = match kanso_gallery::sanitize_svg(&bytes) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        left_out.push(format!("{from}: {err}"));
                        copied.insert(dest.to_owned(), false);
                        return false;
                    }
                };
                spent += bytes.len() as u64;
                // `dest` is checked: relative, no `..`, so inside `folder`.
                let path = folder.join(dest);
                let written = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, &bytes));
                match written {
                    Ok(()) => true,
                    Err(err) => {
                        left_out.push(format!("{from}: it couldn't be written: {err}"));
                        false
                    }
                }
            }
            Err(message) => {
                left_out.push(format!("{from}: it {message}"));
                false
            }
        };
        copied.insert(dest.to_owned(), ok);
        ok
    };
    let file = &mut theme.file;
    let left_out = &mut theme.left_out;
    file.icons
        .retain(|_, path| is_valid_svg_path(path) && copy(path, left_out));
    file.library
        .retain(|_, path| is_valid_svg_path(path) && copy(path, left_out));
    file.notebook_icons.retain(|_, entry: &mut ExtraEntry| {
        is_valid_svg_path(&entry.file) && copy(&entry.file, left_out)
    });
    // What was mapped but couldn't be copied isn't mapped after all.
    let kept: Vec<String> = file
        .icons
        .keys()
        .map(|k| format!("icons.{k}"))
        .chain(file.library.keys().map(|s| format!("library.{}", s.key())))
        .chain(
            file.notebook_icons
                .keys()
                .map(|k| format!("notebook_icons.{k}")),
        )
        .collect();
    theme
        .mapped
        .retain(|line| kept.iter().any(|k| line.ends_with(k.as_str()) || line == k));
    if file.icons.is_empty() && file.library.is_empty() && file.notebook_icons.is_empty() {
        return Err(format!(
            "{} has no icon Kanso can use (no folder, arrow, file or attachment icon came across), so it wasn't imported.",
            file.name
        ));
    }
    for license in &theme.license_files {
        let Some(name) = license
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| is_license_name(n))
        else {
            continue;
        };
        let Ok(text) = read_small(license, MAX_LICENSE_BYTES) else {
            continue;
        };
        if !license.starts_with(&theme.root) {
            continue;
        }
        if theme.file.license.is_empty()
            && let Some(id) = spdx_from_text(&text)
        {
            theme.file.license = id.to_owned();
        }
        // `name` is checked: one plain file name, so inside `folder`.
        std::fs::write(folder.join(&name), text)
            .map_err(|err| format!("Its license couldn't be written: {err}."))?;
    }
    let json = folder.join(format!("{id}.json"));
    std::fs::write(json, theme.file.to_json_string())
        .map_err(|err| format!("Its theme file couldn't be written: {err}."))?;
    Ok(())
}

/// The folder name a theme goes under in `dir`: the one it had, if it was
/// a Kanso folder, else its name as a slug; never a built-in's, and a
/// folder holding another theme gets a number. Pure but for reading `dir`.
fn target_id(
    dir: &Path,
    kept: Option<&str>,
    name: &str,
    is_builtin_id: &dyn Fn(&str) -> bool,
    known_icon: &dyn Fn(&str) -> bool,
) -> String {
    let base = kept
        .map(str::to_owned)
        .unwrap_or_else(|| crate::theme::import::slug(name));
    let base = if is_builtin_id(&base) {
        format!("{base}-imported")
    } else {
        base
    };
    for n in 1.. {
        let id = if n == 1 {
            base.clone()
        } else {
            format!("{}-{n}", base.chars().take(60).collect::<String>())
        };
        let path = dir.join(&id);
        let same = || {
            read_theme_name(&path, known_icon)
                .is_some_and(|theme_name| theme_name.eq_ignore_ascii_case(name))
        };
        if !path.exists() || same() {
            return id;
        }
    }
    unreachable!("some numbered name is free")
}

// MARK: Archives

/// Unpacks the gzipped tar at `path` into a new folder in the temporary
/// folder ([`unpack`]), runs `read` on it, and removes it.
fn with_unpacked<T>(
    path: &Path,
    read: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = std::env::temp_dir().join(format!(
        "kanso-icon-theme-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    // `create_dir`, not `create_dir_all`: the folder must be new.
    std::fs::create_dir(&dir)
        .map_err(|err| format!("A folder to unpack it in couldn't be made: {err}."))?;
    let result = unpack(path, &dir).and_then(|()| read(&dir));
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Unpacks the gzipped tar at `path` into `dir`, in process: regular files
/// and folders only (links and devices are skipped), each at a path of
/// plain parts inside `dir`, at most [`MAX_ARCHIVE_ENTRIES`] of them and
/// [`MAX_ARCHIVE_BYTES`] in all, so an archive can't write outside the
/// folder or fill the disk.
fn unpack(path: &Path, dir: &Path) -> Result<(), String> {
    use std::io::Read;
    let bad = || "It couldn't be unpacked: it isn't a gzipped tar file.".to_owned();
    let failed = |err: std::io::Error| format!("It couldn't be unpacked: {err}.");
    let file = std::fs::File::open(path).map_err(|err| format!("It couldn't be read: {err}."))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let mut total: u64 = 0;
    for (ix, entry) in archive.entries().map_err(|_| bad())?.enumerate() {
        if ix >= MAX_ARCHIVE_ENTRIES {
            return Err(format!("It holds more than {MAX_ARCHIVE_ENTRIES} files."));
        }
        let mut entry = entry.map_err(|_| bad())?;
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            continue;
        }
        let name = entry.path().map_err(|_| bad())?.into_owned();
        let mut target = dir.to_path_buf();
        let mut plain = true;
        for part in name.components() {
            match part {
                std::path::Component::Normal(part) => target.push(part),
                std::path::Component::CurDir => {}
                _ => plain = false,
            }
        }
        if !plain || target == dir {
            continue;
        }
        if kind.is_dir() {
            std::fs::create_dir_all(&target).map_err(failed)?;
            continue;
        }
        let room = MAX_ARCHIVE_BYTES - total;
        let mut bytes = Vec::new();
        (&mut entry)
            .take(room + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| bad())?;
        if bytes.len() as u64 > room {
            return Err(format!(
                "It unpacks to more than {} MB.",
                MAX_ARCHIVE_BYTES / (1024 * 1024)
            ));
        }
        total += bytes.len() as u64;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(failed)?;
        }
        std::fs::write(&target, bytes).map_err(failed)?;
    }
    Ok(())
}

/// Where a package or extension starts in an unpacked archive: npm packs
/// one under `package/`; others hold one folder, or start at the top.
fn package_root(dir: &Path) -> PathBuf {
    let looks = |d: &Path| d.join("package.json").is_file() || d.join("icon_themes").is_dir();
    if looks(dir) {
        return dir.to_path_buf();
    }
    let folders: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|read| {
            read.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    folders
        .iter()
        .find(|d| looks(d))
        .cloned()
        .or_else(|| (folders.len() == 1).then(|| folders[0].clone()))
        .unwrap_or_else(|| dir.to_path_buf())
}

/// Reads the name of the sole JSON theme in an existing folder.
fn read_theme_name(dir: &Path, known_icon: &dyn Fn(&str) -> bool) -> Option<String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    let [path] = files.as_slice() else {
        return None;
    };
    let value = read_json(path, MAX_JSON_BYTES).ok()?;
    IconThemeFile::from_value(&value, known_icon)
        .ok()
        .map(|file| file.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::model::Origin;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M1 1h14v14H1z"/></svg>"#;

    fn import_fixture(source: &Path, dir: &Path) -> Result<Imports, String> {
        let known_icon = |name: &str| matches!(name, "search" | "check");
        let is_builtin_name = |name: &str| name.eq_ignore_ascii_case("Phosphor");
        let is_builtin_id = |id: &str| id == "phosphor";
        super::import_path(source, dir, &known_icon, &is_builtin_name, &is_builtin_id)
    }

    // The desktop used to own this read-back helper. Keep the importer tests
    // self-contained now that the checked file format lives in the engine.
    mod icon_theme_file {
        use std::path::Path;

        use crate::icons::file::{Folder, MAX_SVG_BYTES, SvgFiles, looks_like_svg};
        use crate::icons::model::{IconThemeFile, Notice, Origin};

        pub(super) fn read_folder(
            path: &Path,
            _origin: Origin,
            notices: &mut Vec<Notice>,
        ) -> Option<IconThemeFile> {
            let entry = std::fs::read_dir(path).ok()?.flatten().find(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            })?;
            let text = match std::fs::read_to_string(entry.path()) {
                Ok(text) => text,
                Err(error) => {
                    notices.push(Notice {
                        file: entry.path().display().to_string(),
                        message: format!("couldn't be read: {error}"),
                    });
                    return None;
                }
            };
            let file = match IconThemeFile::parse(&text, &|_| true) {
                Ok(file) => file,
                Err(message) => {
                    notices.push(Notice {
                        file: entry.path().display().to_string(),
                        message,
                    });
                    return None;
                }
            };
            let files = match Folder::new(path) {
                Ok(files) => files,
                Err(message) => {
                    notices.push(Notice {
                        file: path.display().to_string(),
                        message,
                    });
                    return None;
                }
            };
            for name in file
                .icons
                .values()
                .chain(file.library.values())
                .chain(file.notebook_icons.values().map(|entry| &entry.file))
            {
                if let Err(message) = files.read(name, MAX_SVG_BYTES).and_then(|bytes| {
                    looks_like_svg(&bytes)
                        .then_some(())
                        .ok_or_else(|| "isn't an SVG".into())
                }) {
                    notices.push(Notice {
                        file: format!("{}/{}", path.display(), name),
                        message,
                    });
                }
            }
            Some(file)
        }
    }

    fn write(dir: &Path, file: &str, text: &str) {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// A small Zed icon theme extension, as its author would lay it out.
    fn zed_extension(root: &Path) {
        write(
            root,
            "extension.toml",
            "id = \"tiny-icons\"\nname = \"Tiny Icons\"\n",
        );
        write(
            root,
            "LICENSE",
            "MIT License\n\nPermission is hereby granted, free of charge…",
        );
        for icon in [
            "folder",
            "folder-open",
            "chevron-right",
            "chevron-down",
            "file",
            "inbox",
            "pdf",
            "image",
            "audio",
            "rust",
            "unused",
        ] {
            write(root, &format!("icons/{icon}.svg"), SVG);
        }
        write(root, "icons/fake.svg", "not an svg");
        write(
            root,
            "icon_themes/tiny.json",
            &json!({
                "$schema": "https://zed.dev/schema/icon_themes/v0.3.0.json",
                "name": "Tiny Icons",
                "author": "Ada",
                "themes": [{
                    "name": "Tiny",
                    "appearance": "dark",
                    "directory_icons": {"collapsed": "./icons/folder.svg", "expanded": "./icons/folder-open.svg"},
                    "chevron_icons": {"collapsed": "./icons/chevron-right.svg", "expanded": "./icons/chevron-down.svg"},
                    "named_directory_icons": {
                        "Inbox": {"collapsed": "./icons/inbox.svg", "expanded": "./icons/inbox.svg"},
                        "src": {"collapsed": "./icons/folder.svg"},
                        "trash": {"collapsed": "./icons/fake.svg"},
                        "journal": {"collapsed": "../outside.svg"}
                    },
                    "file_suffixes": {"pdf": "pdf", "png": "image", "mp3": "audio", "rs": "rust"},
                    "file_stems": {"Makefile": "rust"},
                    "file_icons": {
                        "default": {"path": "./icons/file.svg"},
                        "pdf": {"path": "./icons/pdf.svg"},
                        "image": {"path": "./icons/image.svg"},
                        "audio": {"path": "./icons/audio.svg"},
                        "rust": {"path": "./icons/rust.svg"}
                    }
                }]
            })
            .to_string(),
        );
    }

    /// A Zed icon theme comes in as a Kanso theme: its folders, arrows,
    /// default file, named folders that line up and attachments mapped;
    /// the rest reported; only the SVGs used copied; its name, author and
    /// license carried over; and Kanso reads the result without a word.
    #[test]
    fn a_zed_icon_theme_imports() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("tiny-icons");
        zed_extension(&ext);
        let out = dir.path().join("icon-themes");
        let reports = import_fixture(&ext, &out).unwrap().reports;
        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.name, "Tiny");
        assert_eq!(report.folder, out.join("tiny"));
        for expected in [
            "directory_icons.collapsed → library.notebook",
            "directory_icons.expanded → library.notebook-open",
            "chevron_icons.collapsed → library.fold-closed",
            "chevron_icons.expanded → library.fold-open",
            "file_icons.default → library.note",
            "named_directory_icons.Inbox → library.inbox",
            "file_suffixes.pdf (pdf) → library.attachment-pdf",
            "file_suffixes.png (image) → library.attachment-image",
            "file_suffixes.mp3 (audio) → library.attachment-audio",
        ] {
            assert!(
                report.mapped.iter().any(|m| m == expected),
                "{expected}: {:?}",
                report.mapped
            );
        }
        for expected in [
            "named_directory_icons.src: Kanso has no notebook",
            "named_directory_icons.journal: its path isn't an .svg inside",
            "icons/fake.svg: it isn't an SVG",
            "file_suffixes: 1 other kinds",
            "file_stems: 1 named files",
        ] {
            assert!(
                report.left_out.iter().any(|m| m.starts_with(expected)),
                "{expected}: {:?}",
                report.left_out
            );
        }
        assert!(
            report
                .summary()
                .starts_with("Imported Tiny: 9 icons mapped"),
            "{}",
            report.summary()
        );
        let mut notices = Vec::new();
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert!(notices.is_empty(), "{notices:?}");
        assert_eq!(
            (theme.author.as_str(), theme.license.as_str()),
            ("Ada", "MIT")
        );
        assert_eq!(theme.library.len(), 9);
        assert!(!theme.library.contains_key(&Slot::Trash));
        assert!(report.folder.join("LICENSE").is_file());
        // Only what is used came across.
        assert!(!report.folder.join("icons/unused.svg").exists());
        assert!(!report.folder.join("icons/rust.svg").exists());
        // Again: the same folder, updated in place, nothing left behind.
        let again = import_fixture(&ext.join("icon_themes/tiny.json"), &out)
            .unwrap()
            .reports;
        assert_eq!(again[0].folder, out.join("tiny"));
        let names: Vec<String> = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["tiny".to_owned()]);
    }

    /// A theme package carrying a Zed icon theme, as a folder and as the
    /// .tgz its registry serves; a package without one says so.
    #[test]
    fn a_theme_package_imports() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg/package");
        zed_extension(&pkg);
        write(
            &pkg,
            "package.json",
            r#"{"name": "tiny-icons", "license": "ISC", "author": {"name": "Pkg"}}"#,
        );
        let out = dir.path().join("out");
        let reports = import_fixture(&pkg, &out).unwrap().reports;
        assert_eq!(reports[0].name, "Tiny");
        let tgz = std::fs::File::create(dir.path().join("tiny.tgz")).unwrap();
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            tgz,
            flate2::Compression::fast(),
        ));
        tar.append_dir_all("package", &pkg).unwrap();
        tar.into_inner().unwrap().finish().unwrap();
        let from_tgz = import_fixture(&dir.path().join("tiny.tgz"), &out)
            .unwrap()
            .reports;
        assert_eq!(from_tgz[0].folder, out.join("tiny"));
        // Named in package.json instead, and in Kanso's own format.
        let named = dir.path().join("named");
        write(
            &named,
            "package.json",
            r#"{"name": "n", "iconTheme": "./theme/kanso.json", "license": "MIT"}"#,
        );
        write(
            &named,
            "theme/kanso.json",
            r#"{"name": "Named", "icons": {"search": "s.svg"}}"#,
        );
        write(&named, "theme/s.svg", SVG);
        let reports = import_fixture(&named, &out).unwrap().reports;
        let mut notices = Vec::new();
        let theme =
            icon_theme_file::read_folder(&reports[0].folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(
            (theme.name.as_str(), theme.license.as_str()),
            ("Named", "MIT")
        );
        assert!(theme.icons.contains_key("search"));
        let bare = dir.path().join("bare");
        write(&bare, "package.json", r#"{"name": "b"}"#);
        assert!(
            import_fixture(&bare, &out)
                .unwrap_err()
                .contains("carries no icon theme")
        );
    }

    /// An archive unpacks only regular files at plain paths inside its
    /// folder, and one that unpacks too large is refused.
    #[test]
    fn an_archive_unpacks_only_inside_its_folder() {
        fn entry(
            tar: &mut tar::Builder<flate2::write::GzEncoder<std::fs::File>>,
            name: &str,
            kind: tar::EntryType,
            data: impl std::io::Read,
            size: u64,
        ) {
            let mut header = tar::Header::new_old();
            // Written raw: `set_path` refuses the names this test needs.
            header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
            header.set_entry_type(kind);
            header.set_size(size);
            header.set_mode(0o644);
            if kind == tar::EntryType::Symlink {
                header.set_link_name("/etc").unwrap();
            }
            header.set_cksum();
            tar.append(&header, data).unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.tgz");
        let new = || {
            tar::Builder::new(flate2::write::GzEncoder::new(
                std::fs::File::create(&path).unwrap(),
                flate2::Compression::fast(),
            ))
        };
        let mut tar = new();
        let svg = SVG.as_bytes();
        for name in ["../escape.svg", "/abs.svg", "a/../../up.svg", "ok/kept.svg"] {
            entry(
                &mut tar,
                name,
                tar::EntryType::Regular,
                svg,
                svg.len() as u64,
            );
        }
        entry(
            &mut tar,
            "link",
            tar::EntryType::Symlink,
            std::io::empty(),
            0,
        );
        tar.into_inner().unwrap().finish().unwrap();
        let out = dir.path().join("out");
        std::fs::create_dir(&out).unwrap();
        unpack(&path, &out).unwrap();
        let mut found: Vec<String> = Vec::new();
        let mut stack = vec![dir.path().to_path_buf()];
        while let Some(at) = stack.pop() {
            for item in std::fs::read_dir(at).unwrap().flatten() {
                let p = item.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    let rel = p.strip_prefix(dir.path()).unwrap();
                    found.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        found.sort();
        assert_eq!(found, ["a.tgz", "out/ok/kept.svg"]);
        let mut tar = new();
        let big = MAX_ARCHIVE_BYTES + 1;
        entry(
            &mut tar,
            "big.svg",
            tar::EntryType::Regular,
            std::io::Read::take(std::io::repeat(b' '), big),
            big,
        );
        tar.into_inner().unwrap().finish().unwrap();
        let bomb = dir.path().join("bomb");
        std::fs::create_dir(&bomb).unwrap();
        assert!(
            unpack(&path, &bomb)
                .unwrap_err()
                .contains("more than 64 MB")
        );
        assert!(std::fs::read_dir(&bomb).unwrap().next().is_none());
        assert!(unpack(&dir.path().join("missing.tgz"), &bomb).is_err());
        std::fs::write(&path, "not gzip").unwrap();
        assert!(
            unpack(&path, &bomb)
                .unwrap_err()
                .contains("isn't a gzipped tar")
        );
    }

    /// A Kanso icon theme folder is checked and copied; one named like a
    /// built-in keeps the built-in's place.
    #[test]
    fn a_kanso_folder_imports_and_builtins_keep_their_names() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("mine");
        write(
            &src,
            "mine.json",
            r#"{"name": "Mine", "icons": {"search": "icons/s.svg", "check": "icons/gone.svg"}}"#,
        );
        write(&src, "icons/s.svg", SVG);
        let out = dir.path().join("out");
        let report = &import_fixture(&src, &out).unwrap().reports[0];
        assert_eq!(report.folder, out.join("mine"));
        assert_eq!(report.mapped, vec!["icons.search".to_owned()]);
        assert!(
            report
                .left_out
                .iter()
                .any(|l| l.starts_with("icons/gone.svg"))
        );
        let named = dir.path().join("phosphor");
        write(
            &named,
            "p.json",
            r#"{"name": "Phosphor", "icons": {"search": "s.svg"}}"#,
        );
        write(&named, "s.svg", SVG);
        let report = &import_fixture(&named, &out).unwrap().reports[0];
        assert_eq!(report.name, "Phosphor (Imported)");
        assert_eq!(report.folder, out.join("phosphor-imported"));
        assert!(import_fixture(&dir.path().join("nope"), &out).is_err());
        write(dir.path(), "x.txt", "x");
        assert!(import_fixture(&dir.path().join("x.txt"), &out).is_err());
    }

    /// A Zed family of one theme, `theme`, in an extension at `root`.
    fn zed_family(root: &Path, themes: Value) {
        write(
            root,
            "icon_themes/family.json",
            &json!({"name": "Family", "author": "Format Author", "themes": themes}).to_string(),
        );
    }

    /// A Zed theme or package JSON past a Kanso theme file's 512 KB
    /// imports, a Kanso theme file past it doesn't, and nothing past the
    /// source cap is read.
    #[test]
    fn large_source_themes_import() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("big");
        write(&ext, "icons/folder.svg", SVG);
        let stems: serde_json::Map<String, Value> = (0..40_000)
            .map(|n| (format!("stem-{n}"), json!("folder")))
            .collect();
        zed_family(
            &ext,
            json!([{
                "name": "Big",
                "directory_icons": {"collapsed": "./icons/folder.svg"},
                "file_stems": stems,
            }]),
        );
        let json = ext.join("icon_themes/family.json");
        assert!(std::fs::metadata(&json).unwrap().len() > MAX_JSON_BYTES);
        let out = dir.path().join("out");
        let imports = import_fixture(&ext, &out).unwrap();
        assert_eq!(imports.reports[0].name, "Big");
        assert!(import_fixture(&json, &out).is_ok());
        // A Kanso theme file keeps its own cap.
        let kanso = dir.path().join("kanso");
        let padding = " ".repeat(MAX_JSON_BYTES as usize);
        write(
            &kanso,
            "k.json",
            &format!(r#"{{"name": "K", "icons": {{"search": "s.svg"}}{padding}}}"#),
        );
        write(&kanso, "s.svg", SVG);
        assert!(
            import_fixture(&kanso, &out)
                .unwrap_err()
                .contains("larger than 512 KB")
        );
        std::fs::write(&json, vec![b' '; MAX_SOURCE_JSON_BYTES as usize + 1]).unwrap();
        assert!(
            import_fixture(&ext, &out)
                .unwrap_err()
                .contains("larger than")
        );
    }

    /// A theme that draws `image`, `audio` and `document` without naming
    /// their suffixes gets the attachment slots through Zed's own suffix
    /// table; a suffix the theme names wins.
    #[test]
    fn attachments_fall_back_to_zeds_suffixes() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("ext");
        for icon in ["image", "audio", "document", "picture"] {
            write(&ext, &format!("icons/{icon}.svg"), SVG);
        }
        let icons = json!({
            "image": {"path": "./icons/image.svg"},
            "audio": {"path": "./icons/audio.svg"},
            "document": {"path": "./icons/document.svg"},
            "picture": {"path": "./icons/picture.svg"},
        });
        let (file, _, mapped, _) =
            zed_theme(&json!({"name": "Defaults", "file_icons": icons.clone()}));
        assert_eq!(
            file.library.get(&Slot::AttachmentImage).map(String::as_str),
            Some("icons/image.svg")
        );
        assert_eq!(
            file.library.get(&Slot::AttachmentAudio).map(String::as_str),
            Some("icons/audio.svg")
        );
        assert_eq!(
            file.library.get(&Slot::AttachmentPdf).map(String::as_str),
            Some("icons/document.svg")
        );
        assert!(
            mapped.iter().any(
                |m| m == "file_icons.image (Zed's default for .png) → library.attachment-image"
            ),
            "{mapped:?}"
        );
        let (file, _, _, _) = zed_theme(&json!({
            "name": "Own",
            "file_suffixes": {"png": "picture"},
            "file_icons": icons,
        }));
        assert_eq!(
            file.library.get(&Slot::AttachmentImage).map(String::as_str),
            Some("icons/picture.svg")
        );
    }

    /// An imported theme's `source` is the extension's or package's web
    /// address, or left out; its author is who the manifest says made it.
    #[test]
    fn source_and_author_come_from_the_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        let ext = dir.path().join("ext");
        zed_extension(&ext);
        write(
            &ext,
            "extension.toml",
            "id = \"tiny-icons\"\nname = \"Tiny Icons\"\nauthors = [\"Grace Hopper <grace@example.com>\", \"Alan\"]\nrepository = \"https://github.com/example/tiny-icons\"\n",
        );
        let report = &import_fixture(&ext, &out).unwrap().reports[0];
        let mut notices = Vec::new();
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.author, "Grace Hopper, Alan");
        assert_eq!(
            theme.source.as_deref(),
            Some("https://github.com/example/tiny-icons")
        );
        assert_eq!(
            theme.description.as_deref(),
            Some("Imported from the Zed icon theme Tiny Icons.")
        );
        // Without one: no source, the format's author as a last resort.
        let bare = dir.path().join("bare");
        zed_extension(&bare);
        let report = &import_fixture(&bare, &out).unwrap().reports[0];
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.source, None);
        assert_eq!(theme.author, "Ada");
        // A package's repository, then its homepage.
        let pkg = dir.path().join("pkg");
        zed_extension(&pkg);
        std::fs::remove_file(pkg.join("extension.toml")).unwrap();
        write(
            &pkg,
            "package.json",
            r#"{"name": "p", "author": "Pat <p@example.com>", "repository": {"type": "git", "url": "git+https://github.com/example/p.git"}}"#,
        );
        let report = &import_fixture(&pkg, &out).unwrap().reports[0];
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.author, "Pat");
        assert_eq!(
            theme.source.as_deref(),
            Some("https://github.com/example/p.git")
        );
        write(
            &pkg,
            "package.json",
            r#"{"name": "p", "repository": "example/p", "homepage": "https://example.com/p"}"#,
        );
        let report = &import_fixture(&pkg, &out).unwrap().reports[0];
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.source.as_deref(), Some("https://example.com/p"));
        assert!(notices.is_empty(), "{notices:?}");
    }

    /// A theme with no icon Kanso can use is refused, and nothing written:
    /// the command fails.
    #[test]
    fn an_empty_import_fails_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("ext");
        write(&ext, "icons/rust.svg", SVG);
        zed_family(
            &ext,
            json!([{
                "name": "Stems Only",
                "file_stems": {"Cargo.toml": "rust"},
                "file_icons": {"rust": {"path": "./icons/rust.svg"}},
            }]),
        );
        let out = dir.path().join("out");
        let err = import_fixture(&ext, &out).unwrap_err();
        assert!(
            err.contains("Stems Only has no icon Kanso can use"),
            "{err}"
        );
        assert!(files(&out).is_empty(), "{:?}", files(&out));
    }

    /// Every licence file comes across, under its own name, and the
    /// manifest's license fills the field.
    #[test]
    fn every_license_file_is_copied() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("ext");
        zed_extension(&ext);
        write(&ext, "LICENSE-APACHE", "Apache License\nVersion 2.0");
        write(&ext, "COPYING.md", "Some words");
        write(&ext, "LICENSE.json", "{}");
        write(&ext, "LICENSES-NOT", "x");
        #[cfg(unix)]
        std::os::unix::fs::symlink(ext.join("LICENSE"), ext.join("LICENSE-LINK")).unwrap();
        write(
            &ext,
            "extension.toml",
            "id = \"t\"\nname = \"T\"\nlicense = \"MIT OR Apache-2.0\"\n",
        );
        let out = dir.path().join("out");
        let report = &import_fixture(&ext, &out).unwrap().reports[0];
        let mut copied: Vec<String> = std::fs::read_dir(&report.folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n != "icons" && !n.ends_with(".json"))
            .collect();
        copied.sort();
        assert_eq!(copied, ["COPYING.md", "LICENSE", "LICENSE-APACHE"]);
        let mut notices = Vec::new();
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.license, "MIT OR Apache-2.0");
        // Without the manifest's, the first licence text that reads as one.
        std::fs::remove_file(ext.join("extension.toml")).unwrap();
        std::fs::remove_file(ext.join("LICENSE")).unwrap();
        let report = &import_fixture(&ext, &out).unwrap().reports[0];
        let theme =
            icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices).unwrap();
        assert_eq!(theme.license, "Apache-2.0");
    }

    #[test]
    fn license_names() {
        for name in [
            "LICENSE",
            "licence",
            "LICENSE-MIT",
            "LICENSE.md",
            "COPYING",
            "COPYING.txt",
            "UNLICENSE",
            "License_APACHE",
        ] {
            assert!(is_license_name(name), "{name}");
        }
        for name in [
            "",
            ".LICENSE",
            "LICENSES",
            "LICENSE.json",
            "LICENSE.svg",
            "README",
            "LICENSE/x",
            "../LICENSE",
            "LICENSE MIT",
        ] {
            assert!(!is_license_name(name), "{name}");
        }
    }

    /// Variants that draw the same shapes in other colors import once,
    /// and the rest are said; a variant with other shapes imports too.
    #[test]
    fn same_shape_variants_import_once() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("ext");
        let svg = |color: &str, d: &str| {
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path fill="{color}" d="{d}"/></svg>"#
            )
        };
        write(
            &ext,
            "icons/dark/folder.svg",
            &svg("#cdd6f4", "M1 1h14v14H1z"),
        );
        write(
            &ext,
            "icons/light/folder.svg",
            &svg("#4C4F69", "M1 1h14v14H1z"),
        );
        write(
            &ext,
            "icons/mocha/folder.svg",
            &svg("rgb(1, 2, 3)", "M1  1h14v14H1z"),
        );
        write(
            &ext,
            "icons/other/folder.svg",
            &svg("#cdd6f4", "M2 2h12v12H2z"),
        );
        let theme = |name: &str, folder: &str| json!({"name": name, "directory_icons": {"collapsed": format!("./icons/{folder}/folder.svg")}});
        zed_family(
            &ext,
            json!([
                theme("Dark", "dark"),
                theme("Light", "light"),
                theme("Mocha", "mocha"),
                theme("Other", "other")
            ]),
        );
        let out = dir.path().join("out");
        let imports = import_fixture(&ext, &out).unwrap();
        let names: Vec<&str> = imports.reports.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Dark", "Other"]);
        assert_eq!(imports.skipped.len(), 2, "{:?}", imports.skipped);
        assert!(imports.skipped[0].starts_with("Light draws the same shapes as Dark"));
        assert!(imports.summary().contains("only Dark was imported"));
        assert!(!out.join("light").exists());
    }

    #[test]
    fn web_urls_and_spdx() {
        assert_eq!(
            web_url(" git+https://github.com/a/b.git ").as_deref(),
            Some("https://github.com/a/b.git")
        );
        assert_eq!(
            web_url("http://example.com").as_deref(),
            Some("http://example.com")
        );
        for bad in [
            "github:a/b",
            "a/b",
            "ftp://x.com",
            "https://",
            "https://x y.com",
            "javascript:alert(1)",
            "https://localhost/x",
        ] {
            assert_eq!(web_url(bad), None, "{bad}");
        }
        assert!(is_spdx("MIT") && is_spdx("(MIT OR Apache-2.0)") && is_spdx("GPL-3.0+"));
        assert!(!is_spdx("") && !is_spdx("SEE LICENSE IN <file>") && !is_spdx("123"));
    }

    #[test]
    fn licenses_read_as_spdx() {
        assert_eq!(spdx_from_text("MIT License\n…"), Some("MIT"));
        assert_eq!(
            spdx_from_text("                                 Apache License\n Version 2.0"),
            Some("Apache-2.0")
        );
        assert_eq!(spdx_from_text("Some words"), None);
    }

    /// The command imports, checks and prints the schema.
    /// Every file under `dir`, relative, with its bytes.
    fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let Ok(read) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in read.filter_map(Result::ok) {
                let path = entry.path();
                let meta = std::fs::symlink_metadata(&path).unwrap();
                if meta.is_dir() {
                    walk(root, &path, out);
                } else {
                    let bytes = if meta.file_type().is_symlink() {
                        b"link".to_vec()
                    } else {
                        std::fs::read(&path).unwrap_or_default()
                    };
                    out.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(dir, dir, &mut out);
        out
    }

    /// A Zed path, from the shapes a careless or hostile theme might write.
    fn hostile_path() -> impl Strategy<Value = Value> {
        prop_oneof![
            Just(json!("./icons/a.svg")),
            Just(json!("icons/b.svg")),
            Just(json!("../secret.svg")),
            Just(json!("./../../secret.svg")),
            Just(json!("/etc/passwd")),
            Just(json!("icons/link.svg")),
            Just(json!("icons/big.svg")),
            Just(json!("C:\\x.svg")),
            Just(json!(42)),
            Just(Value::Null),
            "[a-z./]{0,20}".prop_map(Value::from),
        ]
    }

    fn hostile_theme() -> impl Strategy<Value = Value> {
        (
            prop::option::of("\\PC{0,20}"),
            prop::collection::vec(hostile_path(), 0..8),
            prop::collection::btree_map("[A-Za-z.]{0,8}", hostile_path(), 0..4),
            any::<bool>(),
        )
            .prop_map(|(name, paths, named, garbage)| {
                let p = |ix: usize| paths.get(ix).cloned().unwrap_or(Value::Null);
                let named: serde_json::Map<String, Value> = named
                    .into_iter()
                    .map(|(k, v)| (k, json!({"collapsed": v})))
                    .collect();
                let mut theme = json!({
                    "name": name,
                    "directory_icons": {"collapsed": p(0), "expanded": p(1)},
                    "chevron_icons": {"collapsed": p(2), "expanded": p(3)},
                    "named_directory_icons": named,
                    "file_suffixes": {"pdf": "pdf", "png": p(4)},
                    "file_icons": {"default": {"path": p(5)}, "pdf": {"path": p(6)}, "x": p(7)},
                });
                if garbage {
                    theme["directory_icons"] = json!([1, 2]);
                    theme["file_icons"] = json!("nope");
                }
                json!({"name": "Family", "author": 3, "themes": [theme]})
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// The attachment slots follow Zed's lookup: a suffix's key from the
        /// theme, else from Zed's table, then the theme's `file_icons`; the
        /// first suffix that gets an icon wins.
        #[test]
        fn prop_attachments_follow_zeds_lookup(
            own in prop::collection::btree_map(
                prop::sample::select(vec!["pdf", "png", "jpg", "svg", "mp3", "flac", "rs"]),
                prop::sample::select(vec!["image", "audio", "document", "pic", "none"]),
                0..6,
            ),
            drawn in prop::collection::btree_set(
                prop::sample::select(vec!["image", "audio", "document", "pic"]),
                0..4,
            ),
        ) {
            let icons: serde_json::Map<String, Value> = drawn
                .iter()
                .map(|key| ((*key).to_owned(), json!({"path": format!("./icons/{key}.svg")})))
                .collect();
            let (file, _, _, _) = zed_theme(&json!({
                "name": "T",
                "file_suffixes": own,
                "file_icons": icons,
            }));
            for (slot, wanted) in ATTACHMENTS {
                let expected = wanted.iter().find_map(|suffix| {
                    let key = own.get(suffix).copied().or_else(|| {
                        ZED_DEFAULT_SUFFIXES.iter().find(|(s, _)| s == suffix).map(|(_, k)| *k)
                    })?;
                    drawn.contains(key).then(|| format!("icons/{key}.svg"))
                });
                prop_assert_eq!(file.library.get(slot).cloned(), expected);
            }
        }

        /// An SVG's shape is the same whatever its colors and spacing, and
        /// differs with its path data.
        #[test]
        fn prop_svg_shape_ignores_color(
            a in "#[0-9a-fA-F]{3,8}",
            b in "rgba?\\([0-9, .%]{0,16}\\)",
            spaces in "[ \n\t]{0,4}",
            d in "[MLHVZmlhvz0-9 ]{1,20}",
        ) {
            let svg = |color: &str, d: &str| format!(
                r#"<svg viewBox="0 0 16 16">{spaces}<path fill="{color}" d="{d}"/></svg>"#
            );
            let base = svg(&a, &d);
            prop_assert_eq!(svg_shape(base.as_bytes()), svg_shape(svg(&b, &d).as_bytes()));
            prop_assert_eq!(svg_shape(&svg_shape(base.as_bytes())), svg_shape(base.as_bytes()));
            let other = format!("{d}Q");
            prop_assert_ne!(svg_shape(base.as_bytes()), svg_shape(svg(&a, &other).as_bytes()));
        }

        /// Whatever a manifest says, a source is a web address or nothing,
        /// a license an SPDX-looking expression or nothing, and a licence
        /// file name one plain name.
        #[test]
        fn prop_manifest_values_are_checked(text in "\\PC{0,40}", name in "[A-Za-z0-9./_ -]{0,20}") {
            if let Some(url) = web_url(&text) {
                prop_assert!(url.starts_with("https://") || url.starts_with("http://"));
                prop_assert!(!url.chars().any(char::is_whitespace));
            }
            let package = json!({"license": text, "repository": text, "homepage": text, "author": text});
            let manifest = package_manifest(&package);
            if let Some(license) = &manifest.license {
                prop_assert!(is_spdx(license));
            }
            if is_license_name(&name) {
                prop_assert!(!name.contains('/') && !name.starts_with('.') && !name.contains(' '));
                prop_assert_eq!(Path::new(&name).components().count(), 1);
            }
        }

        /// Whatever an archive's entries are named, it unpacks only inside
        /// its folder; and bytes that aren't an archive are an error, never
        /// a panic.
        #[test]
        fn prop_unpack_stays_in_its_folder(
            names in proptest::collection::vec("[a-z./]{1,12}", 1..6),
            junk in proptest::collection::vec(any::<u8>(), 0..256),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("a.tgz");
            let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
                std::fs::File::create(&path).unwrap(),
                flate2::Compression::fast(),
            ));
            for name in &names {
                let mut header = tar::Header::new_old();
                header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
                header.set_size(3);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append(&header, &b"svg"[..]).unwrap();
            }
            tar.into_inner().unwrap().finish().unwrap();
            let out = dir.path().join("deep/out");
            std::fs::create_dir_all(&out).unwrap();
            let _ = unpack(&path, &out);
            let mut stack = vec![dir.path().to_path_buf()];
            while let Some(at) = stack.pop() {
                for item in std::fs::read_dir(at).unwrap().flatten() {
                    let p = item.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else {
                        prop_assert!(p == path || p.starts_with(&out), "{} escaped", p.display());
                    }
                }
            }
            std::fs::write(&path, &junk).unwrap();
            let fresh = dir.path().join("fresh");
            std::fs::create_dir(&fresh).unwrap();
            let _ = unpack(&path, &fresh);
        }

        /// Whatever a Zed theme says, importing it never panics, never
        /// writes outside the icon themes folder (nothing beside it
        /// changes, and nothing is linked out of it), and what it writes
        /// Kanso reads back.
        #[test]
        fn prop_import_stays_in_its_folder(family in hostile_theme()) {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            let ext = root.join("ext");
            write(root, "secret.svg", SVG);
            write(root, "beside/keep.txt", "keep");
            write(&ext, "icons/a.svg", SVG);
            write(&ext, "icons/b.svg", SVG);
            std::fs::write(ext.join("icons/big.svg"), vec![b' '; MAX_SVG_BYTES as usize + 1]).unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink(root.join("secret.svg"), ext.join("icons/link.svg")).unwrap();
            write(&ext, "icon_themes/t.json", &family.to_string());
            let before_outside: BTreeMap<PathBuf, Vec<u8>> = files(root)
                .into_iter()
                .filter(|(p, _)| !p.starts_with("out"))
                .collect();
            let out = root.join("out");
            let result = std::panic::catch_unwind(|| import_fixture(&ext, &out));
            prop_assert!(result.is_ok(), "import panicked");
            let after_outside: BTreeMap<PathBuf, Vec<u8>> = files(root)
                .into_iter()
                .filter(|(p, _)| !p.starts_with("out"))
                .collect();
            prop_assert_eq!(before_outside, after_outside);
            for (path, bytes) in files(&out) {
                prop_assert!(bytes != b"link", "{} is a link", path.display());
                prop_assert!(!path.starts_with(".."));
                // The secret never comes in under any name.
                let name = path.to_string_lossy().into_owned();
                prop_assert!(!name.contains("secret"), "{}", name);
            }
            if let Ok(imports) = result.unwrap() {
                for report in imports.reports {
                    prop_assert!(report.folder.starts_with(&out));
                    let mut notices = Vec::new();
                    let theme = icon_theme_file::read_folder(&report.folder, Origin::Own, &mut notices);
                    prop_assert!(theme.is_some(), "{:?}", notices);
                    prop_assert!(notices.is_empty(), "{:?}", notices);
                }
            }
        }

        /// The mapping is total and pure: any JSON maps without a panic,
        /// to paths that stay inside the extension and slots only once.
        #[test]
        fn prop_zed_mapping_is_total(theme in hostile_theme()) {
            let (file, sources, mapped, _) = zed_theme(&theme["themes"][0]);
            for path in file.library.values() {
                prop_assert!(is_valid_svg_path(path));
                prop_assert!(is_valid_svg_path(&sources[path]));
            }
            prop_assert_eq!(mapped.len(), file.library.len());
        }
    }
}
