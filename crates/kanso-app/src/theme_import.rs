//! The `kanso theme` command. The importing, the checks and the theme file
//! format are `kanso_engine::theme` (`import`, `file`), so the phone calls the
//! same code.
//!
//! The same checks back `kanso theme check`, so a theme written by hand is
//! held to what the built-in ones are.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde_json::Value;

use crate::themes::themes_dir;

pub use kanso_engine::theme::file::problems;
use kanso_engine::theme::import::{import_into as import_files, index, read_theme};
pub use kanso_engine::theme::import::{import_path, slug, strip_jsonc};

const HELP: &str = "\
Usage: kanso theme <command>

Commands:
  import <path>... [--out <dir>]
                    Convert themes into Kanso theme files: a theme package
                    (its package folder or .tgz), a Zed theme family (.json)
                    or a Kanso theme (.json, checked and copied). They go in
                    --out, or Kanso's themes folder.
  check <file>...   Read Kanso theme files and report any problem: an error
                    in the file, or a color that doesn't read where it is
                    drawn
  fmt <file>...     Rewrite Kanso theme files in the canonical form
  index <dir> [--out <file>]
                    Write an index of the theme files in <dir>
                    (default <dir>/index.json)
  schema [--out <file>]
                    Print the theme file JSON Schema, or write it to <file>
  help              Show this message";

/// `kanso theme …`.
// no-prop-test: argument dispatch; `the_cli_imports_checks_formats_and_indexes`
// runs every command.
pub fn run_cli(data_dir: &Path, args: &[String]) -> anyhow::Result<()> {
    let Some((command, rest)) = args.split_first() else {
        println!("{HELP}");
        return Ok(());
    };
    let (paths, out) = split_out(rest)?;
    match command.as_str() {
        "import" => {
            if paths.is_empty() {
                bail!("`kanso theme import` needs a theme to import\n\n{HELP}");
            }
            let dir = out.unwrap_or_else(|| themes_dir(data_dir));
            let mut failed = 0;
            for path in &paths {
                match import_files(path, &dir) {
                    Ok(written) => {
                        for w in written {
                            println!("Wrote {} ({})", w.path.display(), w.name);
                        }
                    }
                    Err(err) => {
                        failed += 1;
                        eprintln!("{}: {err}", path.display());
                    }
                }
            }
            if failed > 0 {
                bail!("{failed} of {} couldn't be imported", paths.len());
            }
            Ok(())
        }
        "check" => {
            let mut bad = 0;
            for path in &paths {
                let problems = match read_theme(path) {
                    Ok(theme) => problems(&theme),
                    Err(err) => vec![err],
                };
                if problems.is_empty() {
                    println!("{}: reads well", path.display());
                } else {
                    bad += 1;
                    for problem in problems {
                        println!("{}: {problem}", path.display());
                    }
                }
            }
            if bad > 0 {
                bail!("{bad} of {} theme files have problems", paths.len());
            }
            Ok(())
        }
        "fmt" => {
            for path in &paths {
                let theme =
                    read_theme(path).map_err(|err| anyhow::anyhow!("{}: {err}", path.display()))?;
                let text = theme.to_json_string();
                if std::fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
                    println!("{} is already formatted", path.display());
                } else {
                    std::fs::write(path, text)
                        .with_context(|| format!("writing {}", path.display()))?;
                    println!("Formatted {}", path.display());
                }
            }
            Ok(())
        }
        "index" => {
            let [dir] = paths.as_slice() else {
                bail!("`kanso theme index` takes one folder\n\n{HELP}");
            };
            let index = index(dir).map_err(anyhow::Error::msg)?;
            let out = out.unwrap_or_else(|| dir.join("index.json"));
            std::fs::write(&out, pretty(&index))
                .with_context(|| format!("writing {}", out.display()))?;
            println!("Wrote {}", out.display());
            Ok(())
        }
        "schema" => {
            let text = pretty(&crate::theme_file::schema());
            match out {
                Some(out) => {
                    std::fs::write(&out, text)
                        .with_context(|| format!("writing {}", out.display()))?;
                    println!("Wrote {}", out.display());
                }
                None => print!("{text}"),
            }
            Ok(())
        }
        "help" | "-h" | "--help" => {
            println!("{HELP}");
            Ok(())
        }
        other => bail!("unknown theme command `{other}`\n\n{HELP}"),
    }
}

/// The arguments, and `--out`'s value apart from them.
fn split_out(args: &[String]) -> anyhow::Result<(Vec<PathBuf>, Option<PathBuf>)> {
    let mut paths = Vec::new();
    let mut out = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => out = Some(PathBuf::from(args.next().context("--out needs a path")?)),
            other if other.starts_with("--") => bail!("unexpected option `{other}`\n\n{HELP}"),
            other => paths.push(PathBuf::from(other)),
        }
    }
    Ok((paths, out))
}

fn pretty(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    text
}

fn read_theme(path: &Path) -> Result<ThemeFile, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("couldn't be read: {err}"))?;
    ThemeFile::parse(&text)
}

/// The index of the theme files in `dir`: each one's file, metadata and
/// headline colors, sorted by file name.
fn index(dir: &Path) -> Result<Value, String> {
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
struct Written {
    name: String,
    path: PathBuf,
}

fn import_into(source: &Path, dir: &Path) -> Result<Vec<Written>, String> {
    let themes = read_source(source)?;
    std::fs::create_dir_all(dir)
        .map_err(|err| format!("The themes folder couldn't be made: {err}."))?;
    let builtin = |name: &str| {
        crate::themes::builtins()
            .iter()
            .any(|e| e.name().eq_ignore_ascii_case(name))
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

/// A theme package tarball, unpacked with the system's `tar` into a
/// folder of its own in the temporary folder, removed afterwards.
fn read_tarball(path: &Path) -> Result<ThemeFile, String> {
    read_tarball_in(path, &std::env::temp_dir())
}

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
        let status = std::process::Command::new("tar")
            .arg("-xzf")
            .arg(path)
            .arg("-C")
            .arg(&dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|err| format!("It couldn't be unpacked: {err}."))?;
        if !status.success() {
            return Err("It couldn't be unpacked: it isn't a gzipped tar file.".to_owned());
        }
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

// MARK: Legibility

// The checks live in the engine, so the phones' gallery holds a downloaded
// theme to the same ones.
pub use crate::theme_file::problems;
use crate::theme_file::{preview_selection, surfaces};

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
    use super::problems;
    use super::run_cli;
    use crate::theme_file::ThemeFile;

    /// The CLI end to end on a small theme: import, check, fmt, index and
    /// schema.
    #[test]
    fn the_cli_imports_checks_formats_and_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        std::fs::create_dir_all(pkg.join("styles")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name": "grey-dark", "theme": true, "themeAppearance": "dark", "styleSheets": ["ui.css"], "author": "A <a@b.c>", "license": "MIT"}"#,
        )
        .unwrap();
        std::fs::write(
            pkg.join("styles/ui.css"),
            ":root { --editor-background: #222; --text-color: #555; --primary-color: #333; }",
        )
        .unwrap();
        let out = dir.path().join("out");
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let data = dir.path().join("data");
        run_cli(
            &data,
            &args(&[
                "import",
                pkg.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
            ]),
        )
        .unwrap();
        let file = out.join("grey-dark.json");
        let theme = ThemeFile::parse(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(theme.author, "A");
        assert!(problems(&theme).is_empty(), "{:?}", problems(&theme));
        run_cli(&data, &args(&["check", file.to_str().unwrap()])).unwrap();
        assert!(
            !data.exists(),
            "an import with --out leaves the data folder alone"
        );
        // A theme that doesn't read fails the check.
        let unreadable = dir.path().join("unreadable.json");
        std::fs::write(
            &unreadable,
            r##"{"name": "U", "appearance": "dark", "ui": {"editor": "#222222", "text": "#333333", "accent": "#444444"}}"##,
        )
        .unwrap();
        assert!(run_cli(&data, &args(&["check", unreadable.to_str().unwrap()])).is_err());
        // fmt rewrites in the canonical form.
        run_cli(&data, &args(&["fmt", unreadable.to_str().unwrap()])).unwrap();
        let text = std::fs::read_to_string(&unreadable).unwrap();
        assert_eq!(ThemeFile::parse(&text).unwrap().to_json_string(), text);
        // index lists every theme in the folder but itself.
        run_cli(&data, &args(&["index", out.to_str().unwrap()])).unwrap();
        run_cli(&data, &args(&["index", out.to_str().unwrap()])).unwrap();
        let index: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index["schema_version"], 1);
        assert_eq!(index["themes"].as_array().unwrap().len(), 1);
        assert_eq!(index["themes"][0]["file"], "grey-dark.json");
        assert_eq!(index["themes"][0]["name"], "Grey Dark");
        assert_eq!(index["themes"][0]["colors"]["editor"], "#222222");
        let schema = dir.path().join("schema.json");
        run_cli(&data, &args(&["schema", "--out", schema.to_str().unwrap()])).unwrap();
        let text = std::fs::read_to_string(&schema).unwrap();
        assert!(text.ends_with("}\n"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&text).unwrap(),
            crate::theme_file::schema()
        );
        assert!(run_cli(&data, &args(&["nonsense"])).is_err());
        assert!(run_cli(&data, &args(&["help"])).is_ok());
    }
}
