//! Bringing themes in from theme packages and Zed. The desktop's `kanso
//! theme` command and the phones' import call this.
//!
//! An import reads a theme package (an npm-style package folder, or the
//! `.tgz` its registry serves), a Zed theme family (`.json`), or a Kanso theme file,
//! and writes each theme it finds into a themes folder as a Kanso theme
//! file. What a theme doesn't map onto Kanso confidently is left for Kanso
//! to derive. Then each imported theme gets a legibility pass: a color that
//! wouldn't read where Kanso draws it is shaded until it does, and a color
//! that already reads is left alone.
//!
//! The same checks (`problems`, in `theme::file`) back `kanso theme check`, so
//! a theme written by hand is held to what the built-in ones are.

mod color;
mod css;
mod package;
mod package_base;
mod zed;

pub use zed::strip_jsonc;

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::theme::Rgb;
use crate::theme::builtin::BUILTIN;
use crate::theme::file::{Color, ThemeFile, TokenStyle, contrast, ensure_contrast, luminance, mix};

/// The theme file at `path`, read.
pub fn read_theme(path: &Path) -> Result<ThemeFile, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("couldn't be read: {err}"))?;
    ThemeFile::parse(&text)
}

/// The index of the theme files in `dir`: each one's file, metadata and
/// headline colors, sorted by file name.
pub fn index(dir: &Path) -> Result<Value, String> {
    let read = std::fs::read_dir(dir)
        .map_err(|err| format!("{} couldn't be read: {err}", dir.display()))?;
    let mut files: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
        .filter(|p| p.file_name().is_some_and(|n| n != "index.json"))
        .collect();
    files.sort();
    let mut themes = Vec::new();
    for path in files {
        let file = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let theme = read_theme(&path).map_err(|err| format!("{file} {err}"))?;
        let c = theme.resolve().colors;
        let mut entry = Map::new();
        entry.insert("file".into(), file.into());
        entry.insert("name".into(), theme.name.clone().into());
        entry.insert("author".into(), theme.author.clone().into());
        entry.insert("license".into(), theme.license.clone().into());
        entry.insert(
            "source".into(),
            theme.source.clone().map_or(Value::Null, Value::from),
        );
        entry.insert("appearance".into(), theme.appearance.as_str().into());
        let mut colors = Map::new();
        for (key, rgb) in [
            ("editor", c.editor),
            ("text", c.text),
            ("accent", c.accent),
            ("sidebar", c.sidebar),
            ("list", c.list),
        ] {
            colors.insert(key.into(), rgb.hex().into());
        }
        entry.insert("colors".into(), Value::Object(colors));
        themes.push(Value::Object(entry));
    }
    Ok(json!({ "schema_version": 1, "themes": themes }))
}

// MARK: Importing

/// Converts the theme at `source` into `themes_dir`; returns the names of
/// the themes written.
// no-prop-test: reads and writes files; `importing_writes_files_without_clobbering`
// and the CLI test cover it.
pub fn import_path(source: &Path, themes_dir: &Path) -> Result<Vec<String>, String> {
    Ok(import_into(source, themes_dir)?
        .into_iter()
        .map(|w| w.name)
        .collect())
}

/// A theme file an import wrote.
pub struct Written {
    pub name: String,
    pub path: PathBuf,
}

/// [`import_path`], with the file each theme went to.
// no-prop-test: reads and writes files; `importing_writes_files_without_clobbering`
// covers it.
pub fn import_into(source: &Path, dir: &Path) -> Result<Vec<Written>, String> {
    let themes = read_source(source)?;
    std::fs::create_dir_all(dir)
        .map_err(|err| format!("The themes folder couldn't be made: {err}."))?;
    let builtin = |name: &str| {
        builtin_names()
            .iter()
            .any(|builtin| builtin.eq_ignore_ascii_case(name))
    };
    let mut written = Vec::new();
    for mut theme in themes {
        // Kanso shows only the built-in theme under a built-in's name.
        if builtin(&theme.name) {
            theme.name = format!("{} (Imported)", theme.name);
        }
        let path = target_path(dir, &theme.name);
        std::fs::write(&path, theme.to_json_string())
            .map_err(|err| format!("{} couldn't be written: {err}.", path.display()))?;
        written.push(Written {
            name: theme.name,
            path,
        });
    }
    Ok(written)
}

/// The names of the built-in themes, read once.
fn builtin_names() -> &'static [String] {
    static NAMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        BUILTIN
            .iter()
            .filter_map(|(_, text)| ThemeFile::parse(text).ok().map(|theme| theme.name))
            .collect()
    })
}

/// Where a theme named `name` goes in `dir`: `<slug>.json`, unless that
/// file holds a different theme, then `<slug>-2.json` and on. A file that
/// holds the same theme (by name) is replaced, so importing again updates
/// it.
fn target_path(dir: &Path, name: &str) -> PathBuf {
    let slug = slug(name);
    for n in 1.. {
        let file = if n == 1 {
            format!("{slug}.json")
        } else {
            format!("{slug}-{n}.json")
        };
        let path = dir.join(file);
        let same = || read_theme(&path).is_ok_and(|t| t.name.eq_ignore_ascii_case(name));
        if !path.exists() || same() {
            return path;
        }
    }
    unreachable!("some numbered name is free")
}

/// A file name for a theme: lowercase ASCII letters and digits joined by
/// single hyphens, at most 64 characters; "theme" when nothing is left.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.truncate(64);
    let out = out.trim_end_matches('-');
    if out.is_empty() {
        "theme".into()
    } else {
        out.to_owned()
    }
}

/// Every theme in `source`, made legible (except a Kanso theme file, which
/// is taken as written).
// no-prop-test: dispatches on what `source` is; each importer has its own
// property tests, and `importing_writes_files_without_clobbering` runs each
// kind of source.
pub fn read_source(source: &Path) -> Result<Vec<ThemeFile>, String> {
    if !source.exists() {
        return Err(format!("{} doesn't exist.", source.display()));
    }
    let file_name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let mut themes = if source.is_dir() {
        let zed_themes = source.join("themes");
        if !source.join("package.json").is_file() && zed_themes.is_dir() {
            // A Zed extension: its theme families are in `themes/`.
            let mut files: Vec<PathBuf> = std::fs::read_dir(&zed_themes)
                .map_err(|err| format!("Its themes folder couldn't be read: {err}."))?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            files.sort();
            let mut all = Vec::new();
            for file in files {
                all.extend(read_json(&file)?);
            }
            all
        } else {
            vec![package::import(source)?]
        }
    } else if file_name.ends_with(".tgz") || file_name.ends_with(".tar.gz") {
        vec![read_tarball(source)?]
    } else if file_name.ends_with(".json") {
        return read_json(source);
    } else {
        return Err(
            "Kanso imports a theme package (its folder or a .tgz) or a Zed theme (.json); this is neither."
                .into(),
        );
    };
    if themes.is_empty() {
        return Err("There are no themes in it.".into());
    }
    for theme in &mut themes {
        make_legible(theme);
    }
    Ok(themes)
}

/// A `.json` file: a Zed theme family (made legible), or a Kanso theme
/// (checked, and taken as written).
fn read_json(path: &Path) -> Result<Vec<ThemeFile>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|err| format!("It couldn't be read: {err}."))?;
    let value: Value = serde_json::from_str(&text)
        .or_else(|_| serde_json::from_str(&zed::strip_jsonc(&text)))
        .map_err(|err| format!("It isn't valid JSON: {err}."))?;
    if zed::is_family(&value) {
        let mut themes = zed::import(&value)?;
        for theme in &mut themes {
            make_legible(theme);
        }
        return Ok(themes);
    }
    if value.get("ui").is_some() {
        return ThemeFile::from_value(&value)
            .map(|theme| vec![theme])
            .map_err(|err| format!("This Kanso theme {err}."));
    }
    Err("This JSON file isn't a theme Kanso can import: it is neither a Zed theme family nor a Kanso theme.".into())
}

/// A theme package tarball, unpacked in process into a folder of its own in
/// the temporary folder, removed afterwards.
fn read_tarball(path: &Path) -> Result<ThemeFile, String> {
    read_tarball_in(path, &std::env::temp_dir())
}

/// The most an archive may unpack to, and the most files it may hold: well
/// above a theme package's own size.
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 20_000;

/// [`read_tarball`], unpacking in a new folder inside `parent`.
fn read_tarball_in(path: &Path, parent: &Path) -> Result<ThemeFile, String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = parent.join(format!(
        "kanso-theme-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    // `create_dir`, not `create_dir_all`: the folder must be new.
    std::fs::create_dir(&dir)
        .map_err(|err| format!("A folder to unpack it in couldn't be made: {err}."))?;
    let result = (|| {
        unpack(path, &dir)?;
        // npm packs a package under `package/`.
        let root = [dir.join("package"), dir.clone()]
            .into_iter()
            .find(|d| d.join("package.json").is_file())
            .or_else(|| {
                std::fs::read_dir(&dir)
                    .ok()?
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .find(|p| p.join("package.json").is_file())
            })
            .ok_or_else(|| {
                "This package isn't a theme package: it has no package.json.".to_owned()
            })?;
        package::import(&root)
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Unpacks the gzipped tar at `path` into `dir`, in process (a phone can't
/// run `tar`): regular files and folders only (links and devices are
/// skipped), each at a path of plain parts inside `dir`, at most
/// [`MAX_ARCHIVE_ENTRIES`] of them and [`MAX_ARCHIVE_BYTES`] in all, so an
/// archive can't write outside the folder or fill the disk.
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

// MARK: Legibility

// The checks live in `theme::file`, so the phones' gallery holds a
// downloaded theme to the same ones.
pub use super::file::problems;
use super::file::{preview_selection, surfaces};

/// White or black, whichever stands further from `color`.
fn far_end(color: Rgb) -> Rgb {
    let (white, black) = (Rgb(0xFFFFFF), Rgb(0x000000));
    if contrast(white, color) >= contrast(black, color) {
        white
    } else {
        black
    }
}

/// The end of the scale on `surface`'s side of `text`: black when the text
/// is the lighter, white when it is the darker. A surface moved toward it
/// only gains contrast with the text.
fn away_from(text: Rgb, surface: Rgb) -> Rgb {
    if luminance(text) > luminance(surface) {
        Rgb(0x000000)
    } else {
        Rgb(0xFFFFFF)
    }
}

/// The first of `candidates` that `ok` accepts.
fn first_ok(candidates: impl IntoIterator<Item = Rgb>, ok: impl Fn(Rgb) -> bool) -> Option<Rgb> {
    candidates.into_iter().find(|c| ok(*c))
}

/// Shades the colors of `theme` that don't read, until they do (see
/// [`problems`]); colors that already read are kept. A color Kanso would
/// derive is written into the theme only when it needs changing.
// no-prop-test: covered by `prop_made_legible_themes_pass` and
// `prop_legible_themes_are_kept`.
pub fn make_legible(theme: &mut ThemeFile) {
    for _ in 0..4 {
        let before = theme.clone();
        legibility_pass(theme);
        if *theme == before {
            break;
        }
    }
}

fn set_ui(theme: &mut ThemeFile, key: &'static str, rgb: Rgb) {
    theme.ui.insert(key, Color::opaque(rgb.0));
}

fn legibility_pass(theme: &mut ThemeFile) {
    // Body text on the editor first: everything else is fitted around it.
    let c = theme.resolve().colors;
    if contrast(c.text, c.editor) < 4.5 {
        set_ui(
            theme,
            "text",
            ensure_contrast(c.text, &[c.editor], 4.5, far_end(c.editor)),
        );
    }
    // Surfaces that the text doesn't read on move away from it.
    let c = theme.resolve().colors;
    let away = away_from(c.text, c.editor);
    for (key, bg) in surfaces(&c) {
        if contrast(c.text, bg) >= 4.5 {
            continue;
        }
        let fixed = ensure_contrast(bg, &[c.text], 4.5, away);
        if key == "code" {
            theme
                .preview
                .insert("code_background", Color::opaque(fixed.0));
        } else {
            set_ui(theme, key, fixed);
        }
    }
    // Selected rows that don't show on their pane.
    let c = theme.resolve().colors;
    for (key, row, pane, pane_key) in [
        ("row_active", c.row_active, c.list, "list"),
        ("sidebar_active", c.sidebar_active, c.sidebar, "sidebar"),
    ] {
        if contrast(row, pane) >= 1.05 {
            continue;
        }
        let reads = |r: Rgb| contrast(r, pane) >= 1.05 && contrast(c.text, r) >= 4.5;
        let steps = (1..=40).map(|i| f64::from(i) / 80.);
        let toward_text = steps.clone().map(|t| mix(c.text, pane, t));
        let away_side = steps.map(|t| mix(away, pane, t));
        if let Some(fixed) = first_ok(toward_text.chain(away_side), reads) {
            set_ui(theme, key, fixed);
        } else {
            // No room beside the pane: move the pane away from the text.
            let pane_fixed = mix(away, pane, 0.5);
            set_ui(theme, pane_key, pane_fixed);
        }
    }
    let c = theme.resolve().colors;
    let all: Vec<Rgb> = surfaces(&c).iter().map(|(_, bg)| *bg).collect();
    let panes = [c.sidebar, c.list, c.editor, c.elevated];
    let mut fit = |key: &'static str, color: Rgb, on: &[Rgb], min: f64, toward: Rgb| {
        if on.iter().any(|bg| contrast(color, *bg) < min) {
            set_ui(theme, key, ensure_contrast(color, on, min, toward));
        }
    };
    fit("text_secondary", c.text_secondary, &all, 3.0, c.text);
    fit("text_muted", c.text_muted, &panes, 3.0, c.text);
    fit("success", c.success, &panes, 3.0, c.text);
    fit("warning", c.warning, &panes, 3.0, c.text);
    fit("danger", c.danger, &panes, 3.0, c.text);
    fit("link", c.link, &[c.list, c.editor, c.elevated], 4.5, c.text);
    let link = theme.resolve().colors.link;
    if contrast(link, c.sidebar) < 3.0 {
        set_ui(
            theme,
            "link",
            ensure_contrast(
                link,
                &[c.sidebar, c.list, c.editor, c.elevated],
                4.5,
                c.text,
            ),
        );
    }
    let t = theme.resolve();
    if contrast(t.caret, c.editor) < 3.0 {
        set_ui(
            theme,
            "caret",
            ensure_contrast(t.caret, &[c.editor], 3.0, c.text),
        );
    }
    // The button, darkened or lightened under its text as the default light
    // theme needs (white on a mid blue); then the text, if that can't do.
    let c = theme.resolve().colors;
    if contrast(c.button_text, c.button) < 4.5 {
        let button = ensure_contrast(c.button, &[c.button_text], 4.5, far_end(c.button_text));
        set_ui(theme, "button", button);
        if contrast(c.button_text, button) < 4.5 {
            set_ui(
                theme,
                "button_text",
                ensure_contrast(c.button_text, &[button], 4.5, far_end(button)),
            );
        }
    }
    fit_selection(theme);
    fit_syntax(theme);
    fit_preview(theme);
}

/// A selection that shows on the editor and keeps the text readable: the
/// theme's own color at another strength, else the accent's, else a shade
/// on the editor's side of the text. When the text reads too narrowly on
/// the editor for any selection to show under it, the text moves first.
fn fit_selection(theme: &mut ThemeFile) {
    let t = theme.resolve();
    let c = &t.colors;
    let ok = |s: Color| {
        let drawn = s.over(c.editor);
        contrast(drawn, c.editor) >= 1.15 && contrast(c.text, drawn) >= 4.5
    };
    if ok(t.selection) {
        return;
    }
    let alphas = (8..=240u8).step_by(8);
    let found = [
        t.selection.rgb,
        c.accent,
        away_from(c.text, c.editor),
        c.text,
    ]
    .into_iter()
    .flat_map(|rgb| alphas.clone().map(move |alpha| Color { rgb, alpha }))
    .find(|s| ok(*s));
    match found {
        Some(found) => {
            theme.ui.insert("selection", found);
        }
        None => {
            // Room for a selection: the text 15% further from the editor.
            let text_side = away_from(c.editor, c.text);
            set_ui(
                theme,
                "text",
                ensure_contrast(c.text, &[c.editor], 4.5 * 1.2, text_side),
            );
        }
    }
}

/// Syntax colors under 3:1 on the editor, shaded toward the text.
fn fit_syntax(theme: &mut ThemeFile) {
    let t = theme.resolve();
    let (editor, text) = (t.colors.editor, t.colors.text);
    for (key, style) in &t.syntax {
        let Some(color) = style.color else { continue };
        if contrast(color, editor) >= 3.0 {
            continue;
        }
        let fixed = ensure_contrast(color, &[editor], 3.0, text);
        let own = theme.syntax.get(key).copied().unwrap_or_default();
        theme.syntax.insert(
            key,
            TokenStyle {
                color: Some(fixed),
                ..own
            },
        );
    }
}

/// Preview colors that don't read on what they are drawn on.
fn fit_preview(theme: &mut ThemeFile) {
    let t = theme.resolve();
    let c = &t.colors;
    let p = t.preview;
    let mut set = |key: &'static str, rgb: Rgb| {
        theme.preview.insert(key, Color::opaque(rgb.0));
    };
    let mut text = p.text;
    if contrast(text, c.editor) < 4.5 {
        text = ensure_contrast(text, &[c.editor], 4.5, c.text);
        set("text", text);
    }
    for (key, color, min) in [("link", p.link, 4.5), ("quote", p.quote, 4.5)] {
        if contrast(color, c.editor) < min {
            set(key, ensure_contrast(color, &[c.editor], min, c.text));
        }
    }
    if let Some(heading) = p.heading.filter(|h| contrast(*h, c.editor) < 3.0) {
        set(
            "heading",
            ensure_contrast(heading, &[c.editor], 3.0, c.text),
        );
    }
    for (key, fg, bg) in [
        ("code_text", p.code_text.unwrap_or(text), p.code_background),
        (
            "inline_code_text",
            p.inline_code_text.unwrap_or(text),
            p.inline_code_background,
        ),
    ] {
        if contrast(fg, bg) < 4.5 {
            set(key, ensure_contrast(fg, &[bg], 4.5, far_end(bg)));
        }
    }
    if let Some(head) = p
        .table_header_background
        .filter(|h| contrast(text, *h) < 4.5)
    {
        set(
            "table_header_background",
            ensure_contrast(head, &[text], 4.5, away_from(text, c.editor)),
        );
    }
    let t = theme.resolve();
    if contrast(text, preview_selection(&t)) < 4.5 {
        let rgb = t.preview.selection.map_or(t.colors.accent, |s| s.rgb);
        let found = (0..=0x80u8)
            .rev()
            .step_by(4)
            .map(|alpha| Color { rgb, alpha })
            .find(|s| contrast(text, s.over(t.colors.editor)) >= 4.5);
        if let Some(found) = found {
            theme.preview.insert("selection", found);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{make_legible, problems, read_source, read_tarball_in, slug, target_path};
    use crate::theme::file::{
        Appearance, Color, PREVIEW_KEYS, SYNTAX_KEYS, ThemeFile, TokenStyle, UI_KEYS,
    };
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn color() -> impl Strategy<Value = Color> {
        (0u32..=0xFF_FFFF).prop_map(Color::opaque)
    }

    /// A theme with any colors: the required three, and any of the rest.
    fn theme() -> impl Strategy<Value = ThemeFile> {
        let ui_keys: Vec<&'static str> = UI_KEYS.iter().map(|(k, _)| *k).skip(3).collect();
        let syntax_keys: Vec<&'static str> = SYNTAX_KEYS.iter().map(|(k, _)| *k).collect();
        let preview_keys: Vec<&'static str> = PREVIEW_KEYS.iter().map(|(k, _)| *k).collect();
        (
            any::<bool>(),
            color(),
            color(),
            color(),
            prop::collection::btree_map(prop::sample::select(ui_keys), color(), 0..10),
            prop::collection::btree_map(prop::sample::select(syntax_keys), color(), 0..10),
            prop::collection::btree_map(prop::sample::select(preview_keys), color(), 0..5),
        )
            .prop_map(|(dark, editor, text, accent, ui, syntax, preview)| {
                let mut all =
                    BTreeMap::from([("editor", editor), ("text", text), ("accent", accent)]);
                all.extend(ui);
                ThemeFile {
                    name: "Any".into(),
                    author: "a".into(),
                    license: "MIT".into(),
                    source: None,
                    description: None,
                    appearance: if dark {
                        Appearance::Dark
                    } else {
                        Appearance::Light
                    },
                    ui: all,
                    syntax: syntax
                        .into_iter()
                        .map(|(k, c)| (k, TokenStyle::color(c.rgb)))
                        .collect(),
                    preview,
                }
            })
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Legibility: whatever colors a theme comes with, once made
        /// legible it passes every check the built-in themes pass.
        #[test]
        fn prop_made_legible_themes_pass(mut theme in theme()) {
            make_legible(&mut theme);
            let problems = problems(&theme);
            prop_assert!(problems.is_empty(), "{:#?}\n{}", problems, theme.to_json_string());
        }

        /// Restraint: a theme that already reads is left exactly as it is,
        /// and making a theme legible twice changes nothing the second time.
        #[test]
        fn prop_legible_themes_are_kept(mut theme in theme()) {
            if problems(&theme).is_empty() {
                let before = theme.clone();
                make_legible(&mut theme);
                prop_assert_eq!(theme, before);
            } else {
                make_legible(&mut theme);
                let once = theme.clone();
                make_legible(&mut theme);
                prop_assert_eq!(theme, once);
            }
        }

        /// Slugs are safe file names: only lowercase ASCII letters, digits
        /// and single inner hyphens, never empty, at most 64 long; and the
        /// same name always gives the same slug.
        #[test]
        fn prop_slugs_are_safe_and_stable(name in "\\PC{0,80}") {
            let s = slug(&name);
            prop_assert!(!s.is_empty() && s.len() <= 64);
            prop_assert!(s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'));
            prop_assert!(!s.starts_with('-') && !s.ends_with('-') && !s.contains("--"));
            prop_assert_eq!(slug(&name), s.clone());
            // Case and punctuation don't make a different file.
            prop_assert_eq!(slug(&name.to_ascii_uppercase()), slug(&name.to_ascii_lowercase()));
        }
    }

    fn refs() -> Option<std::path::PathBuf> {
        std::env::var_os("KANSO_THEME_REFS").map(Into::into)
    }

    /// Importing writes `<slug>.json`, replaces the same theme on a second
    /// import, and never overwrites a different theme that has the slug.
    #[test]
    fn importing_writes_files_without_clobbering() {
        let dir = tempfile::tempdir().unwrap();
        let family = dir.path().join("family.json");
        std::fs::write(
            &family,
            r##"{ // Zed allows comments
                "name": "Fam", "author": "Me",
                "themes": [
                  {"name": "Fam Dark", "appearance": "dark", "style": {"editor.background": "#101010", "text": "#eeeeee"}},
                  {"name": "Fam Light", "appearance": "light", "style": {"editor.background": "#fafafa", "text": "#202020",}},
                ]
            }"##,
        )
        .unwrap();
        let out = dir.path().join("out");
        let names = super::import_path(&family, &out).unwrap();
        assert_eq!(names, vec!["Fam Dark", "Fam Light"]);
        assert!(out.join("fam-dark.json").is_file() && out.join("fam-light.json").is_file());
        // Again: the same files, updated.
        super::import_path(&family, &out).unwrap();
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), 2);
        // A different theme already at the slug is kept.
        std::fs::write(
            out.join("fam-dark.json"),
            r##"{"name": "FAM-DARK!", "appearance": "dark", "ui": {"editor": "#000000", "text": "#ffffff", "accent": "#ff0000"}}"##,
        )
        .unwrap();
        assert_eq!(target_path(&out, "Fam Dark"), out.join("fam-dark-2.json"));
        // A Kanso theme is copied as written.
        let kanso = dir.path().join("mine.json");
        std::fs::write(
            &kanso,
            r##"{"name": "Mine", "appearance": "light", "ui": {"editor": "#ffffff", "text": "#000000", "accent": "#0000ff"}}"##,
        )
        .unwrap();
        assert_eq!(super::import_path(&kanso, &out).unwrap(), vec!["Mine"]);
        // A built-in's name gets a suffix, so both show.
        std::fs::write(
            &kanso,
            r##"{"name": "kanso dark", "appearance": "dark", "ui": {"editor": "#000000", "text": "#ffffff", "accent": "#0000ff"}}"##,
        )
        .unwrap();
        assert_eq!(
            super::import_path(&kanso, &out).unwrap(),
            vec!["kanso dark (Imported)"]
        );
        // What isn't a theme says so.
        let txt = dir.path().join("notes.txt");
        std::fs::write(&txt, "hi").unwrap();
        assert!(
            super::import_path(&txt, &out)
                .unwrap_err()
                .contains("neither")
        );
        assert!(
            super::import_path(dir.path(), &out)
                .unwrap_err()
                .contains("no package.json")
        );
        let missing = dir.path().join("missing");
        assert!(
            super::import_path(&missing, &out)
                .unwrap_err()
                .contains("doesn't exist")
        );
        let bad = dir.path().join("bad.tgz");
        std::fs::write(&bad, "not gzip").unwrap();
        assert!(
            super::import_path(&bad, &out)
                .unwrap_err()
                .contains("couldn't be unpacked")
        );
    }

    /// The Default Light and Default Dark theme packages, when reference
    /// copies are at `$KANSO_THEME_REFS` (in folders named after their
    /// upstream repositories, `<owner>_<repo>`), import as near-white and
    /// near-black themes with a blue accent, and read.
    #[test]
    fn the_default_themes_import() {
        let Some(refs) = refs() else {
            return;
        };
        for (folder, dark) in [
            ("inkdropapp_default-light", false),
            ("inkdropapp_default-dark", true),
        ] {
            let themes = read_source(&refs.join(folder)).unwrap();
            let theme = &themes[0];
            assert_eq!(theme.appearance.is_dark(), dark);
            assert!(
                problems(theme).is_empty(),
                "{folder}: {:?}",
                problems(theme)
            );
            let editor = theme.ui["editor"].rgb;
            assert!(
                if dark {
                    editor.0 < 0x202020
                } else {
                    editor.0 > 0xF0F0F0
                },
                "{}",
                editor.hex()
            );
            let accent = theme.ui["accent"].rgb.0;
            assert!((accent & 0xFF) > ((accent >> 16) & 0xFF), "a blue accent");
        }
    }

    /// A gzipped tar holding a file for each of `names`, written raw so a
    /// name the `tar` crate refuses (`..`, an absolute path) can be tried.
    fn archive_of(names: &[String], at: &std::path::Path) {
        let file = std::fs::File::create(at).unwrap();
        let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
        for name in names {
            let mut header = tar::Header::new_gnu();
            let bytes = name.as_bytes();
            header.as_old_mut().name[..bytes.len()].copy_from_slice(bytes);
            header.set_size(2);
            header.set_mode(0o644);
            header.set_cksum();
            std::io::Write::write_all(&mut gz, header.as_bytes()).unwrap();
            let mut block = [0u8; 512];
            block[..2].copy_from_slice(b"hi");
            std::io::Write::write_all(&mut gz, &block).unwrap();
        }
        std::io::Write::write_all(&mut gz, &[0u8; 1024]).unwrap();
        gz.finish().unwrap();
    }

    proptest! {
        #![proptest_config(config(64))]

        /// Unpacking stays inside its folder: whatever the entry names are
        /// (`..`, absolute paths, dots), nothing appears beside the folder,
        /// and every file that does appear is in it.
        #[test]
        fn prop_unpacking_never_writes_outside_the_folder(
            names in prop::collection::vec(
                prop::collection::vec(
                    prop::sample::select(vec!["a", "..", ".", "", "b.txt", "package"]),
                    1..5,
                ),
                0..6,
            ),
        ) {
            let names: Vec<String> = names.iter().map(|parts| parts.join("/")).collect();
            let outer = tempfile::tempdir().unwrap();
            let archive = outer.path().join("x.tgz");
            archive_of(&names, &archive);
            let dir = outer.path().join("unpacked");
            std::fs::create_dir(&dir).unwrap();
            let _ = super::unpack(&archive, &dir);
            let mut beside: Vec<_> = std::fs::read_dir(outer.path())
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            beside.sort();
            prop_assert_eq!(beside, vec!["unpacked", "x.tgz"]);
        }
    }

    /// A package packed as npm packs it (under `package/`) imports as the
    /// folder does, and the unpacked copy is removed.
    #[test]
    fn a_tarball_imports_like_its_folder() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("staging/package");
        std::fs::create_dir_all(pkg.join("styles")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name": "tiny-light", "theme": true, "themeAppearance": "light", "styleSheets": ["ui.css"]}"#,
        )
        .unwrap();
        std::fs::write(
            pkg.join("styles/ui.css"),
            ":root { --primary-color: #0a58ca; }",
        )
        .unwrap();
        let tgz = dir.path().join("tiny-light-1.0.0.tgz");
        let status = std::process::Command::new("tar")
            .arg("-czf")
            .arg(&tgz)
            .arg("-C")
            .arg(dir.path().join("staging"))
            .arg("package")
            .status()
            .unwrap();
        assert!(status.success());
        let from_tarball = read_source(&tgz).unwrap();
        assert_eq!(from_tarball, read_source(&pkg).unwrap());
        assert_eq!(from_tarball[0].ui["accent"].hex(), "#0A58CA");
        // Unpacked in a folder of its own, which is gone afterwards.
        let parent = dir.path().join("unpack");
        std::fs::create_dir(&parent).unwrap();
        let mut unpacked = read_tarball_in(&tgz, &parent).unwrap();
        make_legible(&mut unpacked);
        assert_eq!(unpacked, from_tarball[0]);
        assert_eq!(
            std::fs::read_dir(&parent).unwrap().count(),
            0,
            "the unpacked copy is removed"
        );
    }
}
