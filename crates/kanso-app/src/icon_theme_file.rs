//! Kanso's icon theme files: a folder holding one JSON document, in the
//! format `themes/icon-themes/schema/kanso-icon-theme-v1.json` describes (and
//! `themes/icon-themes/README.md` explains), and the SVGs it names.
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

use std::io::Read as _;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

pub use kanso_engine::icons::file::{
    Files, Fingerprint, MAX_ENTRIES, MAX_JSON_BYTES, MAX_PATH_BYTES, MAX_SVG_BYTES,
    MAX_THEME_BYTES, ParseError, SCHEMA_URL, SCHEMA_VERSION, SlotSource, SvgFiles, is_valid_name,
    is_valid_svg_path, looks_like_svg, slot_source, theme_key,
};

use crate::models::icon_theme::{IconTheme, IconThemeFile, Slot};
use crate::themes::Notice;

/// Whether Kanso draws a UI icon named `name`, one a theme's `icons` can
/// draw in its place.
pub fn is_ui_icon(name: &str) -> bool {
    crate::assets::is_app_icon(name)
}

/// Reads a theme file, its `icons` naming only icons Kanso draws.
pub fn parse(text: &str) -> Result<IconThemeFile, ParseError> {
    IconThemeFile::parse(text, &is_ui_icon)
}

/// Reads a theme file already parsed as JSON, as [`parse`] does.
pub fn from_value(value: &Value) -> Result<IconThemeFile, ParseError> {
    IconThemeFile::from_value(value, &is_ui_icon)
}

/// The theme `file` describes, its SVGs read from `files`, under folder
/// name `id`; see [`kanso_engine::icons::file::load`]. A theme with a
/// `path` is a person's or the gallery's, so its SVGs are sanitized.
pub fn load(
    id: &str,
    file: &IconThemeFile,
    files: &dyn SvgFiles,
    path: Option<PathBuf>,
    origin: crate::origin::Origin,
    notices: &mut Vec<Notice>,
) -> IconTheme {
    let sanitize = |bytes: &[u8]| kanso_gallery::sanitize_svg(bytes).map_err(|err| err.to_string());
    let sanitizing = path.is_some();
    kanso_engine::icons::file::load(
        id,
        file,
        files,
        path,
        origin,
        sanitizing.then_some(&sanitize as kanso_engine::icons::file::Sanitize),
        notices,
    )
}

/// A theme's folder on disk.
pub struct Folder {
    /// The folder, its links resolved.
    root: PathBuf,
}

impl Folder {
    pub fn new(dir: &Path) -> Result<Folder, String> {
        let root = dir
            .canonicalize()
            .map_err(|err| format!("couldn't be read: {err}"))?;
        Ok(Folder { root })
    }
}

impl SvgFiles for Folder {
    fn read(&self, path: &str, limit: u64) -> Result<Vec<u8>, String> {
        if !is_valid_svg_path(path) {
            return Err("isn't a path inside the theme's folder".into());
        }
        let full = self
            .root
            .join(path)
            .canonicalize()
            .map_err(|_| "isn't in the theme's folder".to_owned())?;
        // A link may only lead to another file in the folder.
        if !full.starts_with(&self.root) {
            return Err("leads out of the theme's folder; only files inside it are read".into());
        }
        let meta = std::fs::metadata(&full).map_err(|err| format!("couldn't be read: {err}"))?;
        if !meta.is_file() {
            return Err("isn't a file".into());
        }
        if meta.len() > limit {
            return Err(format!("is larger than {} KB", limit.div_ceil(1024)));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(&full)
            .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
            .map_err(|err| format!("couldn't be read: {err}"))?;
        if bytes.len() as u64 > limit {
            return Err(format!("is larger than {} KB", limit.div_ceil(1024)));
        }
        Ok(bytes)
    }
}

/// Reads the theme in folder `dir`, named after the folder: its one `.json`
/// file and the SVGs it names. `None`, with a notice, when the folder's
/// name or its JSON can't be used.
pub fn read_folder(
    dir: &Path,
    origin: crate::origin::Origin,
    notices: &mut Vec<Notice>,
) -> Option<IconTheme> {
    let id = dir.file_name()?.to_string_lossy().into_owned();
    let notice = |message: String| Notice {
        file: id.clone(),
        message,
    };
    if !is_valid_name(&id) {
        notices.push(notice(
            "isn't an icon theme folder name Kanso can use; name it with lowercase letters, digits and dashes".into(),
        ));
        return None;
    }
    let mut json: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("json"))
        })
        .collect();
    json.sort();
    let path = match json.as_slice() {
        [one] => one.clone(),
        [] => {
            notices.push(notice("has no theme file (.json)".into()));
            return None;
        }
        _ => {
            notices.push(notice(
                "has more than one .json file; an icon theme's folder holds one".into(),
            ));
            return None;
        }
    };
    let file_name = format!(
        "{id}/{}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    let folder = match Folder::new(dir) {
        Ok(folder) => folder,
        Err(message) => {
            notices.push(notice(message));
            return None;
        }
    };
    let parsed = std::fs::symlink_metadata(&path)
        .map_err(|err| format!("couldn't be read: {err}"))
        .and_then(|meta| {
            if !meta.is_file() {
                Err("isn't a file".to_owned())
            } else if meta.len() > MAX_JSON_BYTES {
                Err(format!("is larger than {} KB", MAX_JSON_BYTES / 1024))
            } else {
                std::fs::read_to_string(&path).map_err(|err| format!("couldn't be read: {err}"))
            }
        })
        .and_then(|text| parse(&text));
    match parsed {
        Ok(file) => Some(load(
            &id,
            &file,
            &folder,
            Some(dir.to_path_buf()),
            origin,
            notices,
        )),
        Err(message) => {
            notices.push(Notice {
                file: file_name,
                message,
            });
            None
        }
    }
}

/// [`read_folder`], with a theme that trips it skipped and said so, never
/// taking the app down with it.
pub fn read_folder_safely(
    dir: &Path,
    origin: crate::origin::Origin,
    notices: &mut Vec<Notice>,
) -> Option<IconTheme> {
    let mut theirs = Vec::new();
    let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_folder(dir, origin, &mut theirs)
    }));
    notices.append(&mut theirs);
    read.unwrap_or_else(|_| {
        notices.push(Notice {
            file: dir.file_name()?.to_string_lossy().into_owned(),
            message: "couldn't be read".into(),
        });
        None
    })
}

// MARK: The schema

/// The JSON Schema for icon theme files, generated from the slot table and
/// the icons Kanso draws so it can't drift from what Kanso reads.
/// `themes/icon-themes/schema/kanso-icon-theme-v1.json` is this, written out.
pub fn schema() -> Value {
    let svg = json!({ "$ref": "#/$defs/svg" });
    let mut library = Map::new();
    for slot in Slot::ALL {
        library.insert(
            slot.key().into(),
            json!({ "$ref": "#/$defs/svg", "description": slot.description() }),
        );
    }
    let mut icons = Map::new();
    for name in crate::assets::app_icon_names() {
        icons.insert(name, svg.clone());
    }
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_URL,
        "title": "Kanso icon theme",
        "description": "One Kanso icon theme: SVGs for the UI icons by name, for the library's own icons by slot, and more icons for notebooks. Every map is optional; an icon a theme leaves out is drawn as before.",
        "type": "object",
        "required": ["name"],
        "additionalProperties": false,
        "properties": {
            "$schema": { "type": "string" },
            "schema_version": { "const": SCHEMA_VERSION, "description": "The format version, 1." },
            "name": { "type": "string", "minLength": 1, "description": "The name shown in Settings › Themes; unique among icon themes." },
            "author": { "type": "string" },
            "license": { "type": "string", "description": "An SPDX license identifier for the icons, such as MIT." },
            "source": { "type": "string", "description": "Where the icons come from." },
            "description": { "type": "string" },
            "icons": {
                "type": "object",
                "description": "SVGs for Kanso's UI icons, by their Lucide names.",
                "additionalProperties": false,
                "properties": Value::Object(icons),
            },
            "library": {
                "type": "object",
                "description": "SVGs for the library's own icons. A slot left out falls back to the icon it shows by default.",
                "additionalProperties": false,
                "properties": Value::Object(library),
            },
            "notebook_icons": {
                "type": "object",
                "description": "More icons for the notebook icon picker, by name.",
                "maxProperties": MAX_ENTRIES,
                "propertyNames": { "pattern": "^[a-z0-9][a-z0-9_-]{0,63}$" },
                "additionalProperties": {
                    "oneOf": [
                        { "$ref": "#/$defs/svg" },
                        {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["file"],
                            "properties": {
                                "file": { "$ref": "#/$defs/svg" },
                                "label": { "type": "string" },
                                "keywords": { "type": "string", "description": "More words the picker's search finds it by, spaced." },
                            },
                        },
                    ],
                },
            },
        },
        "$defs": {
            "svg": {
                "type": "string",
                "maxLength": MAX_PATH_BYTES,
                "pattern": "^([A-Za-z0-9._-]+/)*[A-Za-z0-9._-]+\\.[sS][vV][gG]$",
                "description": "The path of an .svg file inside the theme's folder, such as icons/search.svg.",
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_engine::icons::file::label_from_name;

    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    const SVG: &str =
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 12h16"/></svg>"#;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn write(dir: &Path, file: &str, text: &str) {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
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

    /// A folder with a theme: what it can use is read, what it can't is
    /// skipped and said, and the rest of the theme still draws.
    #[test]
    fn a_folder_reads_what_it_can_and_says_what_it_cant() {
        let dir = tempfile::tempdir().unwrap();
        let theme = dir.path().join("mine");
        let outside = dir.path().join("secret.svg");
        std::fs::write(&outside, SVG).unwrap();
        write(
            &theme,
            "mine.json",
            r#"{"name": "Mine", "icons": {"search": "icons/s.svg", "check": "icons/fake.svg",
                "plus": "icons/missing.svg", "minus": "icons/huge.svg", "settings": "icons/link.svg",
                "close": "icons/s.svg"},
                "library": {"inbox": "tray.svg"},
                "notebook_icons": {"planet": "planet.svg"}}"#,
        );
        write(&theme, "icons/s.svg", SVG);
        write(&theme, "icons/fake.svg", "not an svg");
        write(&theme, "tray.svg", SVG);
        write(&theme, "planet.svg", SVG);
        std::fs::write(
            theme.join("icons/huge.svg"),
            vec![b' '; MAX_SVG_BYTES as usize + 1],
        )
        .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, theme.join("icons/link.svg")).unwrap();
        let mut notices = Vec::new();
        let read = read_folder(&theme, crate::origin::Origin::Own, &mut notices).unwrap();
        assert_eq!(read.id, "mine");
        assert_eq!(read.name, "Mine");
        let icons: Vec<&str> = read.icons.keys().map(String::as_str).collect();
        assert_eq!(icons, vec!["close", "search"]);
        // One file named twice is read once and shared.
        assert!(Arc::ptr_eq(&read.icons["close"], &read.icons["search"]));
        assert!(read.library.contains_key(&Slot::Inbox));
        assert_eq!(read.extra("planet").unwrap().label, "Planet");
        assert!(read.key.starts_with("mine-"));
        let said: Vec<String> = notices.iter().map(Notice::sentence).collect();
        for expected in [
            "mine/icons/fake.svg isn't an SVG",
            "mine/icons/missing.svg isn't in the theme's folder",
            "mine/icons/huge.svg is larger",
        ] {
            assert!(
                said.iter().any(|s| s.contains(expected)),
                "{expected}: {said:?}"
            );
        }
        #[cfg(unix)]
        assert!(
            said.iter()
                .any(|s| s.contains("mine/icons/link.svg leads out")),
            "{said:?}"
        );

        // Folders that can't be themes.
        let mut notices = Vec::new();
        write(&dir.path().join("Bad Name"), "a.json", r#"{"name": "A"}"#);
        assert!(
            read_folder(
                &dir.path().join("Bad Name"),
                crate::origin::Origin::Own,
                &mut notices
            )
            .is_none()
        );
        write(&dir.path().join("empty"), "a.svg", SVG);
        assert!(
            read_folder(
                &dir.path().join("empty"),
                crate::origin::Origin::Own,
                &mut notices
            )
            .is_none()
        );
        write(&dir.path().join("two"), "a.json", r#"{"name": "A"}"#);
        write(&dir.path().join("two"), "b.json", r#"{"name": "B"}"#);
        assert!(
            read_folder(
                &dir.path().join("two"),
                crate::origin::Origin::Own,
                &mut notices
            )
            .is_none()
        );
        write(
            &dir.path().join("broken"),
            "b.json",
            r#"{"name": "B", "x": 1}"#,
        );
        assert!(
            read_folder(
                &dir.path().join("broken"),
                crate::origin::Origin::Own,
                &mut notices
            )
            .is_none()
        );
        std::fs::create_dir_all(dir.path().join("big")).unwrap();
        std::fs::write(
            dir.path().join("big/b.json"),
            vec![b' '; MAX_JSON_BYTES as usize + 1],
        )
        .unwrap();
        assert!(
            read_folder(
                &dir.path().join("big"),
                crate::origin::Origin::Own,
                &mut notices
            )
            .is_none()
        );
        let said: Vec<String> = notices.iter().map(Notice::sentence).collect();
        for expected in [
            "Bad Name isn't an icon theme folder name",
            "empty has no theme file",
            "two has more than one .json",
            "broken/b.json has an unknown key `x`",
            "big/b.json is larger than",
        ] {
            assert!(
                said.iter().any(|s| s.contains(expected)),
                "{expected}: {said:?}"
            );
        }
    }

    /// The fallbacks form chains that end in a UI icon Kanso draws.
    #[test]
    fn slots_fall_back_to_icons() {
        let empty = load(
            "e",
            &IconThemeFile {
                name: "E".into(),
                ..Default::default()
            },
            &Files(&[]),
            None,
            crate::origin::Origin::Own,
            &mut Vec::new(),
        );
        for slot in Slot::ALL {
            assert_eq!(Slot::parse(slot.key()), Some(slot));
            let SlotSource::Icon(name) = slot_source(&empty, slot) else {
                panic!("{slot:?}");
            };
            assert!(is_ui_icon(name), "{slot:?} falls back to {name}");
        }
        assert_eq!(
            slot_source(&empty, Slot::NotebookOpen),
            SlotSource::Icon("notebook")
        );
    }

    /// The schema in the repository is the one the tables make.
    #[test]
    fn schema_file_is_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../themes/icon-themes/schema/kanso-icon-theme-v1.json");
        let mut expected = serde_json::to_string_pretty(&schema()).unwrap();
        expected.push('\n');
        if std::env::var_os("KANSO_THEME_FMT").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &expected).unwrap();
        }
        let written = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            written == expected,
            "themes/icon-themes/schema/kanso-icon-theme-v1.json is out of date (commit it in the kanso-themes submodule); run KANSO_THEME_FMT=1 cargo test -p kanso-app icon_theme_file"
        );
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
            if let Ok(file) = from_value(&value) {
                prop_assert!(!file.name.trim().is_empty());
                prop_assert!(file.icons.len() + file.library.len() + file.notebook_icons.len() <= MAX_ENTRIES);
                prop_assert!(file.icons.keys().all(|name| is_ui_icon(name)));
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
                let joined = Path::new("/root/theme").join(&path);
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
            let names: Vec<String> = crate::assets::app_icon_names().into_iter().take(sizes.len()).collect();
            let file = IconThemeFile {
                name: "Caps".into(),
                icons: names.iter().enumerate().map(|(ix, n)| (n.clone(), format!("i{ix}.svg"))).collect(),
                ..Default::default()
            };
            let mut notices = Vec::new();
            let theme = load("caps", &file, &Files(&table), None, crate::origin::Origin::Own, &mut notices);
            prop_assert!(theme.icons.values().all(|svg| svg.len() as u64 <= MAX_SVG_BYTES));
            let total: u64 = theme.icons.values().map(|svg| svg.len() as u64).sum();
            prop_assert!(total <= MAX_THEME_BYTES);
            prop_assert_eq!(theme.icons.len() + notices.len(), names.len());
        }
    }
}
