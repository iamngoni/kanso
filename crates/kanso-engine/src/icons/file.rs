//! Kanso's icon theme files: a folder holding one JSON document, in the
//! format `themes/icon-themes/schema/kanso-icon-theme-v1.json` describes (and
//! `themes/icon-themes/README.md` explains), and the SVGs it names. Reading
//! them is here; reading a folder off the disk is the desktop app's.
//!
//! A theme gives SVGs in three maps, each optional: `icons` draws the UI
//! icons by their names (`search`, `chevron-down`), `library` draws the
//! library's own icons by their slots (`notebook-open`, `inbox`), and
//! `notebook_icons` adds icons to the notebook icon picker. Anything a
//! theme leaves out is drawn as before, icon by icon.
//!
//! The folder comes from a person, so it is read as untrusted: the JSON and
//! each SVG are capped in size, and the icons in number; a path names a
//! file inside the folder (no `..`, no absolute paths, and a link that leads
//! out of the folder isn't followed); names are checked. As with color
//! themes, a file is read leniently about what is missing and strictly
//! about what is there: an unknown key is an error naming it. An SVG that
//! can't be used is skipped, with a notice, and the rest of the theme still
//! draws.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Map, Value};

use super::model::{
    ExtraEntry, ExtraIcon, Fallback, IconTheme, IconThemeFile, Notice, Origin, Slot,
};

/// The format version this build reads and writes.
pub const SCHEMA_VERSION: u64 = 1;

/// Where the schema is published, for a theme's `$schema`.
pub const SCHEMA_URL: &str = "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/icon-themes/schema/kanso-icon-theme-v1.json";

/// Largest theme JSON read.
pub const MAX_JSON_BYTES: u64 = 512 * 1024;
/// Largest SVG read.
pub const MAX_SVG_BYTES: u64 = 128 * 1024;
/// Most entries one theme may name, in its three maps together.
pub const MAX_ENTRIES: usize = 2_000;
/// Most bytes of SVG one theme may hold in all; the rest are skipped.
pub const MAX_THEME_BYTES: u64 = 16 * 1024 * 1024;
/// Longest path to an SVG, in bytes.
pub const MAX_PATH_BYTES: usize = 200;

/// What is wrong with a theme file, in a sentence that names the key.
pub type ParseError = String;

/// The metadata keys a file may carry beside its maps.
const TOP_KEYS: &[&str] = &[
    "$schema",
    "schema_version",
    "name",
    "author",
    "license",
    "source",
    "description",
    "icons",
    "library",
    "notebook_icons",
];

/// Whether `name` can name a theme's folder or one of its icons: lowercase
/// ASCII letters, digits, dashes and underscores, starting with a letter or
/// digit, at most 64 long. Pure.
pub fn is_valid_name(name: &str) -> bool {
    name.len() <= 64
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Whether `path` can name an SVG in a theme's folder: relative, `/`
/// between parts, each part of ASCII letters, digits, `.`, `-` and `_`,
/// none of them empty, `.` or `..`, ending in `.svg`. Pure.
pub fn is_valid_svg_path(path: &str) -> bool {
    path.len() <= MAX_PATH_BYTES
        && path.to_ascii_lowercase().ends_with(".svg")
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        })
}

/// "rocket-launch" as "Rocket launch". Pure.
pub fn label_from_name(name: &str) -> String {
    let spaced = name.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => spaced,
    }
}

/// Whether `bytes` look like an SVG Kanso can draw: UTF-8 with an `<svg`
/// element. Pure.
pub fn looks_like_svg(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok_and(|text| text.contains("<svg"))
}

impl IconThemeFile {
    /// Reads a theme file. `known` says whether Kanso draws a UI icon of a
    /// name, which a theme's `icons` may name.
    pub fn parse(text: &str, known: &dyn Fn(&str) -> bool) -> Result<IconThemeFile, ParseError> {
        let value: Value =
            serde_json::from_str(text).map_err(|err| format!("isn't valid JSON: {err}"))?;
        IconThemeFile::from_value(&value, known)
    }

    /// Reads a theme file already parsed as JSON; see [`IconThemeFile::parse`].
    pub fn from_value(
        value: &Value,
        known: &dyn Fn(&str) -> bool,
    ) -> Result<IconThemeFile, ParseError> {
        let Value::Object(top) = value else {
            return Err("should be a JSON object".into());
        };
        for key in top.keys() {
            if !TOP_KEYS.contains(&key.as_str()) {
                return Err(format!(
                    "has an unknown key `{key}`; an icon theme has {}",
                    TOP_KEYS.join(", ")
                ));
            }
        }
        match top.get("schema_version") {
            None => {}
            Some(Value::Number(n)) if n.as_u64() == Some(SCHEMA_VERSION) => {}
            Some(other) => {
                return Err(format!(
                    "is schema_version {other}; this Kanso reads version {SCHEMA_VERSION}"
                ));
            }
        }
        let text = |key: &str, required: bool| -> Result<Option<String>, ParseError> {
            match top.get(key) {
                None if required => Err(format!("needs a `{key}`")),
                None => Ok(None),
                Some(Value::String(s)) if required && s.trim().is_empty() => {
                    Err(format!("has an empty `{key}`"))
                }
                Some(Value::String(s)) => Ok(Some(s.trim().to_owned())),
                Some(_) => Err(format!("`{key}` should be text")),
            }
        };
        let name = text("name", true)?.unwrap_or_default();
        let icons = paths(top.get("icons"), "icons", |key| {
            known(key).then(|| key.to_owned()).ok_or_else(|| {
                format!("has an unknown icon `icons.{key}`; it isn't an icon Kanso draws")
            })
        })?;
        let library = paths(top.get("library"), "library", |key| {
            Slot::parse(key).ok_or_else(|| {
                format!(
                    "has an unknown key `library.{key}`; `library` has {}",
                    Slot::ALL.map(Slot::key).join(", ")
                )
            })
        })?;
        let notebook_icons = extras(top.get("notebook_icons"))?;
        if icons.len() + library.len() + notebook_icons.len() > MAX_ENTRIES {
            return Err(format!("names more than {MAX_ENTRIES} icons"));
        }
        Ok(IconThemeFile {
            name,
            author: text("author", false)?.unwrap_or_default(),
            license: text("license", false)?.unwrap_or_default(),
            source: text("source", false)?,
            description: text("description", false)?,
            icons,
            library,
            notebook_icons,
        })
    }

    /// The file, as JSON with its keys in the documented order.
    pub fn to_value(&self) -> Value {
        let mut top = Map::new();
        top.insert("$schema".into(), SCHEMA_URL.into());
        top.insert("schema_version".into(), SCHEMA_VERSION.into());
        top.insert("name".into(), self.name.clone().into());
        if !self.author.is_empty() {
            top.insert("author".into(), self.author.clone().into());
        }
        if !self.license.is_empty() {
            top.insert("license".into(), self.license.clone().into());
        }
        if let Some(source) = &self.source {
            top.insert("source".into(), source.clone().into());
        }
        if let Some(description) = &self.description {
            top.insert("description".into(), description.clone().into());
        }
        if !self.icons.is_empty() {
            let icons: Map<String, Value> = self
                .icons
                .iter()
                .map(|(k, v)| (k.clone(), v.clone().into()))
                .collect();
            top.insert("icons".into(), Value::Object(icons));
        }
        if !self.library.is_empty() {
            let library: Map<String, Value> = Slot::ALL
                .iter()
                .filter_map(|slot| {
                    Some((
                        slot.key().to_owned(),
                        self.library.get(slot)?.clone().into(),
                    ))
                })
                .collect();
            top.insert("library".into(), Value::Object(library));
        }
        if !self.notebook_icons.is_empty() {
            let extras: Map<String, Value> = self
                .notebook_icons
                .iter()
                .map(|(name, entry)| {
                    let value = match (&entry.label, &entry.keywords) {
                        (None, None) => entry.file.clone().into(),
                        (label, keywords) => {
                            let mut object = Map::new();
                            object.insert("file".into(), entry.file.clone().into());
                            if let Some(label) = label {
                                object.insert("label".into(), label.clone().into());
                            }
                            if let Some(keywords) = keywords {
                                object.insert("keywords".into(), keywords.clone().into());
                            }
                            Value::Object(object)
                        }
                    };
                    (name.clone(), value)
                })
                .collect();
            top.insert("notebook_icons".into(), Value::Object(extras));
        }
        Value::Object(top)
    }

    /// The file as text, pretty-printed, ending in a newline.
    pub fn to_json_string(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_value()).unwrap_or_default();
        text.push('\n');
        text
    }
}

/// A map of keys to SVG paths, each key checked by `key`.
fn paths<K: Ord>(
    value: Option<&Value>,
    section: &str,
    key: impl Fn(&str) -> Result<K, ParseError>,
) -> Result<BTreeMap<K, String>, ParseError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(map) = value else {
        return Err(format!("`{section}` should be an object of icons"));
    };
    if map.len() > MAX_ENTRIES {
        return Err(format!("names more than {MAX_ENTRIES} icons"));
    }
    let mut out = BTreeMap::new();
    for (name, path) in map {
        let k = key(name)?;
        out.insert(k, svg_path(path, &format!("{section}.{name}"))?);
    }
    Ok(out)
}

/// An SVG path at `at`, checked.
fn svg_path(value: &Value, at: &str) -> Result<String, ParseError> {
    match value {
        Value::String(path) if is_valid_svg_path(path) => Ok(path.clone()),
        Value::String(path) => Err(format!(
            "`{at}` is \"{}\"; it should be the path of an .svg file inside the theme's folder, such as \"icons/search.svg\"",
            path.chars().take(80).collect::<String>()
        )),
        _ => Err(format!("`{at}` should be the path of an .svg file")),
    }
}

/// The `notebook_icons` map: names to a path, or to `{file, label,
/// keywords}`.
fn extras(value: Option<&Value>) -> Result<BTreeMap<String, ExtraEntry>, ParseError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(map) = value else {
        return Err("`notebook_icons` should be an object of icons".into());
    };
    if map.len() > MAX_ENTRIES {
        return Err(format!("names more than {MAX_ENTRIES} icons"));
    }
    let mut out = BTreeMap::new();
    for (name, entry) in map {
        let at = format!("notebook_icons.{name}");
        if !is_valid_name(name) {
            return Err(format!(
                "has an icon named `{at}`; name it with lowercase letters, digits and dashes"
            ));
        }
        let entry = match entry {
            Value::Object(fields) => {
                for key in fields.keys() {
                    if !["file", "label", "keywords"].contains(&key.as_str()) {
                        return Err(format!(
                            "has an unknown key `{at}.{key}`; an icon has file, label and keywords"
                        ));
                    }
                }
                let words = |key: &str| -> Result<Option<String>, ParseError> {
                    match fields.get(key) {
                        None => Ok(None),
                        Some(Value::String(s)) => {
                            Ok(Some(s.trim().chars().take(200).collect::<String>()))
                        }
                        Some(_) => Err(format!("`{at}.{key}` should be text")),
                    }
                };
                ExtraEntry {
                    file: svg_path(
                        fields
                            .get("file")
                            .ok_or_else(|| format!("needs `{at}.file`"))?,
                        &format!("{at}.file"),
                    )?,
                    label: words("label")?.filter(|l| !l.is_empty()),
                    keywords: words("keywords")?.filter(|k| !k.is_empty()),
                }
            }
            other => ExtraEntry {
                file: svg_path(other, &at)?,
                label: None,
                keywords: None,
            },
        };
        out.insert(name.clone(), entry);
    }
    Ok(out)
}

// MARK: Reading a theme's SVGs

/// Where a theme's SVGs are read from: its folder, or the built-in themes'
/// files compiled in.
pub trait SvgFiles {
    /// The file at `path` (already checked by [`is_valid_svg_path`]), at
    /// most `limit` bytes long.
    fn read(&self, path: &str, limit: u64) -> Result<Vec<u8>, String>;
}

/// Files in memory, by path: the built-in themes, and tests.
pub struct Files<'a>(pub &'a [(&'a str, &'a [u8])]);

impl SvgFiles for Files<'_> {
    fn read(&self, path: &str, limit: u64) -> Result<Vec<u8>, String> {
        let bytes = self
            .0
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| *bytes)
            .ok_or_else(|| "isn't in the theme's folder".to_owned())?;
        if bytes.len() as u64 > limit {
            return Err(format!("is larger than {} KB", limit.div_ceil(1024)));
        }
        Ok(bytes.to_vec())
    }
}

/// A 64-bit FNV-1a fingerprint: the same for the same bytes on every
/// build, unlike the standard library's hasher.
#[derive(Clone, Copy)]
pub struct Fingerprint(u64);

impl Default for Fingerprint {
    fn default() -> Self {
        Fingerprint(0xcbf2_9ce4_8422_2325)
    }
}

impl Fingerprint {
    pub fn add(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }

    /// Adds a field, its length first, so `ab`,`c` and `a`,`bc` differ.
    pub fn field(&mut self, bytes: &[u8]) {
        self.add(&(bytes.len() as u64).to_le_bytes());
        self.add(bytes);
    }

    pub fn finish(self) -> u64 {
        self.0
    }
}

/// The key a theme's icon paths carry: its ID and a fingerprint of every
/// SVG it draws and where. Pure.
pub fn theme_key(theme: &IconTheme) -> String {
    let mut print = Fingerprint::default();
    for (name, svg) in &theme.icons {
        print.field(b"icon");
        print.field(name.as_bytes());
        print.field(svg);
    }
    for (slot, svg) in &theme.library {
        print.field(b"library");
        print.field(slot.key().as_bytes());
        print.field(svg);
    }
    for extra in &theme.extras {
        print.field(b"extra");
        print.field(extra.name.as_bytes());
        print.field(&extra.svg);
    }
    format!("{}-{:016x}", theme.id, print.finish())
}

/// Makes an SVG safe to draw, or says it can't be: what a theme from a
/// person or the gallery is passed through.
pub type Sanitize<'a> = &'a dyn Fn(&[u8]) -> Result<Vec<u8>, String>;

/// The theme `file` describes, its SVGs read from `files`, under folder
/// name `id`. An SVG that can't be used is left out with a notice saying
/// why (its icon is then drawn as if the theme didn't name it), and so is
/// any past [`MAX_THEME_BYTES`].
pub fn load(
    id: &str,
    file: &IconThemeFile,
    files: &dyn SvgFiles,
    path: Option<PathBuf>,
    origin: Origin,
    sanitize: Option<Sanitize>,
    notices: &mut Vec<Notice>,
) -> IconTheme {
    // Built-ins are compiled from a reviewed tag; every theme read from
    // disk, a person's or the gallery's, is sanitized before it is drawn.
    let mut read: HashMap<String, Option<Arc<[u8]>>> = HashMap::new();
    let mut spent = 0u64;
    let mut over = false;
    let mut svg = |path: &str, notices: &mut Vec<Notice>| -> Option<Arc<[u8]>> {
        if let Some(done) = read.get(path) {
            return done.clone();
        }
        let result = if over {
            None
        } else {
            match files.read(path, MAX_SVG_BYTES) {
                Ok(bytes) if !looks_like_svg(&bytes) => {
                    notices.push(Notice {
                        file: format!("{id}/{path}"),
                        message: "isn't an SVG".into(),
                    });
                    None
                }
                Ok(bytes) if sanitize.is_some_and(|clean| clean(&bytes).is_err()) => {
                    notices.push(Notice {
                        file: format!("{id}/{path}"),
                        message: "has something Kanso won't draw in an icon".into(),
                    });
                    None
                }
                Ok(bytes) if spent + bytes.len() as u64 > MAX_THEME_BYTES => {
                    over = true;
                    notices.push(Notice {
                        file: id.to_owned(),
                        message: format!(
                            "holds more than {} MB of icons; the rest are left out",
                            MAX_THEME_BYTES / (1024 * 1024)
                        ),
                    });
                    None
                }
                Ok(bytes) => {
                    let bytes = match sanitize {
                        Some(clean) => clean(&bytes).unwrap_or(bytes),
                        None => bytes,
                    };
                    spent += bytes.len() as u64;
                    Some(Arc::from(bytes))
                }
                Err(message) => {
                    notices.push(Notice {
                        file: format!("{id}/{path}"),
                        message,
                    });
                    None
                }
            }
        };
        read.insert(path.to_owned(), result.clone());
        result
    };
    let icons = file
        .icons
        .iter()
        .filter_map(|(name, path)| Some((name.clone(), svg(path, notices)?)))
        .collect();
    let library = file
        .library
        .iter()
        .filter_map(|(slot, path)| Some((*slot, svg(path, notices)?)))
        .collect();
    let extras = file
        .notebook_icons
        .iter()
        .filter_map(|(name, entry)| {
            Some(ExtraIcon {
                name: name.clone(),
                label: entry.label.clone().unwrap_or_else(|| label_from_name(name)),
                keywords: entry.keywords.clone().unwrap_or_default(),
                svg: svg(&entry.file, notices)?,
            })
        })
        .collect();
    let mut theme = IconTheme {
        id: id.to_owned(),
        name: file.name.clone(),
        author: file.author.clone(),
        license: file.license.clone(),
        source: file.source.clone(),
        description: file.description.clone(),
        icons,
        library,
        extras,
        path,
        origin,
        key: String::new(),
    };
    theme.key = theme_key(&theme);
    theme
}

/// What a slot shows under `theme`, as the UI icon's name it falls back to
/// or the slot whose SVG the theme draws. Pure.
pub fn slot_source(theme: &IconTheme, slot: Slot) -> SlotSource {
    let mut at = slot;
    // The fallbacks form a short chain with no loops (`slots_fall_back_to_icons`).
    for _ in 0..Slot::ALL.len() {
        if theme.library.contains_key(&at) {
            return SlotSource::Library(at);
        }
        match at.fallback() {
            Fallback::Slot(next) => at = next,
            Fallback::Icon(name) => return SlotSource::Icon(name),
        }
    }
    SlotSource::Icon("notebook")
}

/// Where a slot's icon comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotSource {
    /// The theme's SVG for this slot.
    Library(Slot),
    /// The UI icon of this name.
    Icon(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    const SVG: &str =
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 12h16"/></svg>"#;

    /// The UI icons the tests' themes may name.
    const KNOWN: [&str; 6] = ["search", "check", "plus", "notebook", "lock", "pin"];

    fn known(name: &str) -> bool {
        KNOWN.contains(&name)
    }

    fn parse(text: &str) -> Result<IconThemeFile, ParseError> {
        IconThemeFile::parse(text, &known)
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn names_and_paths() {
        assert!(is_valid_name("phosphor"));
        assert!(is_valid_name("my-icons_2"));
        for bad in ["", "Mine", "-x", "a/b", "a b", "é", "..", &"a".repeat(65)] {
            assert!(!is_valid_name(bad), "{bad}");
        }
        for good in ["search.svg", "icons/search.svg", "a/b/C-1_x.SVG"] {
            assert!(is_valid_svg_path(good), "{good}");
        }
        for bad in [
            "",
            "search.png",
            "/etc/x.svg",
            "../x.svg",
            "icons/../../x.svg",
            "./x.svg",
            "icons//x.svg",
            "C:\\x.svg",
            "c:x.svg",
            "icons\\x.svg",
            "ä.svg",
            "a b.svg",
        ] {
            assert!(!is_valid_svg_path(bad), "{bad}");
        }
        assert_eq!(label_from_name("rocket-launch"), "Rocket launch");
        assert!(looks_like_svg(SVG.as_bytes()));
        assert!(!looks_like_svg(b"<html>"));
    }

    /// The smallest theme there is, a full one, and what is wrong with
    /// broken ones, in words that name the key.
    #[test]
    fn minimal_full_and_broken_files() {
        let tiny = parse(r#"{"name": "Tiny"}"#).unwrap();
        assert_eq!(tiny.name, "Tiny");
        assert!(tiny.icons.is_empty() && tiny.library.is_empty());
        let full = parse(
            r#"{"$schema": "x", "schema_version": 1, "name": "Full", "author": "A", "license": "MIT",
                "icons": {"search": "icons/search.svg"},
                "library": {"notebook-open": "open.svg", "inbox": "tray.svg"},
                "notebook_icons": {"planet": "planet.svg",
                                   "moon": {"file": "moon.svg", "label": "Night", "keywords": "dark sky"}}}"#,
        )
        .unwrap();
        assert_eq!(full.icons["search"], "icons/search.svg");
        assert_eq!(full.library[&Slot::NotebookOpen], "open.svg");
        assert_eq!(full.notebook_icons["moon"].label.as_deref(), Some("Night"));
        assert_eq!(parse(&full.to_json_string()).unwrap(), full);
        let cases = [
            ("{", "isn't valid JSON"),
            ("[]", "should be a JSON object"),
            ("{}", "needs a `name`"),
            (r#"{"name": " "}"#, "empty `name`"),
            (r#"{"name": "X", "colors": {}}"#, "unknown key `colors`"),
            (r#"{"name": "X", "schema_version": 2}"#, "schema_version 2"),
            (
                r#"{"name": "X", "icons": {"serch": "a.svg"}}"#,
                "unknown icon `icons.serch`",
            ),
            (
                r#"{"name": "X", "library": {"inbx": "a.svg"}}"#,
                "unknown key `library.inbx`",
            ),
            (
                r#"{"name": "X", "icons": {"search": "../a.svg"}}"#,
                "`icons.search` is \"../a.svg\"",
            ),
            (
                r#"{"name": "X", "icons": {"search": 3}}"#,
                "`icons.search` should be the path",
            ),
            (
                r#"{"name": "X", "notebook_icons": {"Big": "a.svg"}}"#,
                "`notebook_icons.Big`",
            ),
            (
                r#"{"name": "X", "notebook_icons": {"a": {"file": "a.svg", "colour": "red"}}}"#,
                "unknown key `notebook_icons.a.colour`",
            ),
            (
                r#"{"name": "X", "notebook_icons": {"a": {"label": "A"}}}"#,
                "needs `notebook_icons.a.file`",
            ),
        ];
        for (text, expected) in cases {
            let err = parse(text).unwrap_err();
            assert!(err.contains(expected), "{text}: {err}");
        }
    }

    fn empty_theme() -> IconTheme {
        load(
            "e",
            &IconThemeFile {
                name: "E".into(),
                ..Default::default()
            },
            &Files(&[]),
            None,
            Origin::Own,
            None,
            &mut Vec::new(),
        )
    }

    /// The fallbacks form chains that end in a UI icon Kanso draws.
    #[test]
    fn slots_fall_back_to_icons() {
        let empty = empty_theme();
        for slot in Slot::ALL {
            assert_eq!(Slot::parse(slot.key()), Some(slot));
            let SlotSource::Icon(name) = slot_source(&empty, slot) else {
                panic!("{slot:?}");
            };
            assert!(is_valid_name(name), "{slot:?} falls back to {name}");
        }
        assert_eq!(
            slot_source(&empty, Slot::NotebookOpen),
            SlotSource::Icon("notebook")
        );
    }

    /// A slot the theme draws is its own; one it leaves out follows the
    /// chain, slot by slot, to a UI icon.
    #[test]
    fn a_theme_draws_the_slots_it_names_and_falls_back_for_the_rest() {
        let file =
            parse(r#"{"name": "T", "library": {"notebook": "n.svg", "trash": "t.svg"}}"#).unwrap();
        let table: [(&str, &[u8]); 2] = [("n.svg", SVG.as_bytes()), ("t.svg", SVG.as_bytes())];
        let theme = load(
            "t",
            &file,
            &Files(&table),
            None,
            Origin::Builtin,
            None,
            &mut Vec::new(),
        );
        assert_eq!(
            slot_source(&theme, Slot::Trash),
            SlotSource::Library(Slot::Trash)
        );
        // The Inbox is not drawn, but falls back to the notebook slot, which is.
        assert_eq!(
            slot_source(&theme, Slot::Inbox),
            SlotSource::Library(Slot::Notebook)
        );
        assert_eq!(slot_source(&theme, Slot::Pinned), SlotSource::Icon("pin"));
    }

    /// Past the SVG caps and the sanitizer, an icon is left out with a
    /// notice, and the rest still draws.
    #[test]
    fn a_sanitizer_can_refuse_an_svg() {
        let file =
            parse(r#"{"name": "T", "icons": {"search": "a.svg", "plus": "b.svg"}}"#).unwrap();
        let bad = br#"<svg><script/></svg>"#;
        let table: [(&str, &[u8]); 2] = [("a.svg", SVG.as_bytes()), ("b.svg", bad)];
        let sanitize = |bytes: &[u8]| {
            if bytes.windows(7).any(|w| w == b"<script") {
                Err("script".to_owned())
            } else {
                Ok(bytes.to_vec())
            }
        };
        let mut notices = Vec::new();
        let theme = load(
            "t",
            &file,
            &Files(&table),
            Some(PathBuf::from("/x")),
            Origin::Own,
            Some(&sanitize),
            &mut notices,
        );
        assert!(theme.icons.contains_key("search") && !theme.icons.contains_key("plus"));
        assert_eq!(notices.len(), 1);
        assert!(notices[0].sentence().contains("t/b.svg"));
    }

    /// JSON of any shape, from the keys a theme file has and some it
    /// doesn't, with values of any type.
    fn any_json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            "(\\PC{0,12}|icons/[a-z]{1,5}\\.svg|\\.\\./x\\.svg)".prop_map(Value::from),
        ];
        let key = prop_oneof![
            Just("name".to_owned()),
            Just("icons".to_owned()),
            Just("library".to_owned()),
            Just("notebook_icons".to_owned()),
            Just("search".to_owned()),
            Just("inbox".to_owned()),
            Just("file".to_owned()),
            "[a-z-]{1,8}",
        ];
        leaf.prop_recursive(3, 40, 6, move |inner| {
            prop::collection::btree_map(key.clone(), inner, 0..6)
                .prop_map(|map| Value::Object(map.into_iter().collect()))
        })
    }

    proptest! {
        #![proptest_config(config())]

        /// Whatever JSON comes in, reading it never panics; what reads is
        /// within the caps, names only icons Kanso draws and paths inside
        /// the folder, and writes back to the same file.
        #[test]
        fn prop_parser_never_panics_and_keeps_its_promises(value in any_json()) {
            if let Ok(file) = IconThemeFile::from_value(&value, &known) {
                prop_assert!(!file.name.trim().is_empty());
                prop_assert!(file.icons.len() + file.library.len() + file.notebook_icons.len() <= MAX_ENTRIES);
                prop_assert!(file.icons.keys().all(|name| known(name)));
                prop_assert!(file.icons.values().chain(file.library.values()).all(|p| is_valid_svg_path(p)));
                prop_assert!(file.notebook_icons.iter().all(|(name, e)| is_valid_name(name) && is_valid_svg_path(&e.file)));
                prop_assert_eq!(parse(&file.to_json_string()).unwrap(), file);
            }
        }

        /// Any text at all: never a panic.
        #[test]
        fn prop_any_text_never_panics(text in "\\PC{0,200}") {
            let _ = parse(&text);
        }

        /// A valid path never leaves the folder: no part climbs, none is
        /// empty, and it doesn't start at the root or a drive.
        #[test]
        fn prop_valid_paths_stay_inside(path in "[a-zA-Z0-9./\\\\:_-]{0,24}") {
            if is_valid_svg_path(&path) {
                let parts: Vec<&str> = path.split('/').collect();
                prop_assert!(parts.iter().all(|p| !p.is_empty() && *p != ".." && *p != "."));
                prop_assert!(!path.contains('\\') && !path.contains(':') && !path.starts_with('/'));
                let joined = std::path::Path::new("/root/theme").join(&path);
                prop_assert!(joined.starts_with("/root/theme"));
                prop_assert!(joined.components().all(|c| !matches!(c, std::path::Component::ParentDir)));
            }
        }

        /// Names: valid ones are short, lowercase and safe as one path part.
        #[test]
        fn prop_valid_names_are_one_safe_part(name in "\\PC{0,70}") {
            if is_valid_name(&name) {
                prop_assert!(name.len() <= 64);
                prop_assert!(!name.contains('/') && !name.contains('\\') && name != ".." && !name.starts_with('.'));
            }
        }

        /// However many icons a theme names, reading it stays within the
        /// caps: no SVG past its size, none past the theme's budget.
        #[test]
        fn prop_loading_respects_the_caps(sizes in prop::collection::vec(0usize..4, 0..20)) {
            let svgs: Vec<(String, Vec<u8>)> = sizes
                .iter()
                .enumerate()
                .map(|(ix, size)| {
                    let body = "z".repeat([0, 100, MAX_SVG_BYTES as usize, MAX_SVG_BYTES as usize + 1][*size]);
                    (format!("i{ix}.svg"), format!("<svg>{body}</svg>").into_bytes())
                })
                .collect();
            let table: Vec<(&str, &[u8])> = svgs.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
            let names: Vec<String> = (0..sizes.len()).map(|ix| format!("icon-{ix}")).collect();
            let file = IconThemeFile {
                name: "Caps".into(),
                icons: names.iter().enumerate().map(|(ix, n)| (n.clone(), format!("i{ix}.svg"))).collect(),
                ..Default::default()
            };
            let mut notices = Vec::new();
            let theme = load("caps", &file, &Files(&table), None, Origin::Own, None, &mut notices);
            prop_assert!(theme.icons.values().all(|svg| svg.len() as u64 <= MAX_SVG_BYTES));
            let total: u64 = theme.icons.values().map(|svg| svg.len() as u64).sum();
            prop_assert!(total <= MAX_THEME_BYTES);
            prop_assert_eq!(theme.icons.len() + notices.len(), names.len());
        }

        /// A theme's key changes with any one of its SVGs and with its ID,
        /// and not otherwise.
        #[test]
        fn prop_theme_key_follows_the_svgs(a in "[a-z]{1,6}", b in "[a-z]{1,6}") {
            let make = |id: &str, body: &str| {
                let table = [("a.svg", format!("<svg>{body}</svg>").into_bytes())];
                let table: Vec<(&str, &[u8])> = table.iter().map(|(n, b)| (*n, b.as_slice())).collect();
                let file = IconThemeFile {
                    name: "K".into(),
                    icons: [("search".to_owned(), "a.svg".to_owned())].into_iter().collect(),
                    ..Default::default()
                };
                load(id, &file, &Files(&table), None, Origin::Builtin, None, &mut Vec::new()).key
            };
            prop_assert_eq!(make("k", &a), make("k", &a));
            prop_assert_eq!(make("k", &a) == make("k", &b), a == b);
            prop_assert_ne!(make("k", &a), make("j", &a));
        }
    }
}
