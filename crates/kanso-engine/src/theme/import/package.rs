//! Theme packages: an npm-style package folder with a `package.json`, as
//! Electron notes apps publish their themes.
//!
//! Whole themes (`"theme": true`) style the UI, the editor and the
//! preview with CSS custom properties, and ship a `palette.json` with every
//! one resolved; that is read first, then the stylesheets' own
//! `--name: value` declarations. A variable a theme doesn't set falls back
//! to the default theme of the same appearance, as it does in the app the
//! theme was made for, from the table in `package_base.rs`.
//!
//! Older single-part themes do one part each (`"theme": "ui"`, `"syntax"` or
//! `"preview"`). A UI theme uses the same custom properties. A syntax theme
//! is LESS or CSS styling CodeMirror's `.cm-*` classes, and a preview theme
//! styles `.mde-preview`'s elements; both are read rule by rule. Such a theme
//! becomes a whole Kanso theme whose other parts Kanso derives.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::color;
use super::css::{self, Declaration, Sheet};
use super::package_base;
use crate::theme::Rgb;
use crate::theme::file::{Appearance, Color, ThemeFile, TokenStyle, contrast, luminance};

/// Which parts of the app a package styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// A whole theme: all three.
    Whole,
    Ui,
    Syntax,
    Preview,
}

/// The theme in the theme package at `dir`.
// no-prop-test: reads a folder; `the_default_themes_import`,
// `a_night_owl_style_ui_theme` and `a_material_style_syntax_theme` cover it,
// and the mapping's parts have their own property tests.
pub fn import(dir: &Path) -> Result<ThemeFile, String> {
    let package_path = dir.join("package.json");
    if !package_path.is_file() {
        return Err("This folder isn't a theme package: it has no package.json.".into());
    }
    let package: Value = std::fs::read_to_string(&package_path)
        .map_err(|err| format!("Its package.json couldn't be read: {err}."))
        .and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|err| format!("Its package.json isn't valid JSON: {err}."))
        })?;
    let kind = kind(&package)?;
    let sheets = load_sheets(dir, &package);
    let palette = read_palette(dir)?;
    if sheets.is_empty() && palette.is_none() {
        return Err(
            "This theme package has no stylesheets or palette.json to read its colors from.".into(),
        );
    }
    let declarations = with_less_variables(&sheets);
    let mut own = css::custom_properties(&declarations);
    if let Some(palette) = &palette {
        for (name, value) in palette {
            if let (true, Some(value)) = (name.starts_with("--"), value.as_str()) {
                own.insert(name.clone(), value.to_owned());
            }
        }
    }
    let rules = Rules::read(&declarations, kind);
    let appearance = appearance(&package, palette.as_ref(), &own, &rules, &declarations);
    let dark = appearance.is_dark();
    let base: HashMap<String, String> = package_base::defaults(dark)
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let mut all = base.clone();
    all.extend(own.iter().map(|(k, v)| (k.clone(), v.clone())));
    let vars = Vars {
        own,
        base,
        all,
        dark,
        use_base: matches!(kind, Kind::Whole | Kind::Ui),
    };
    let required_only = Vars {
        use_base: true,
        ..vars.clone()
    };

    let (ui, editor) = match kind {
        Kind::Whole | Kind::Ui => ui_section(&vars),
        Kind::Syntax | Kind::Preview => required_ui(&required_only, &rules),
    };
    let syntax = match kind {
        Kind::Whole | Kind::Syntax => {
            let mut syntax = syntax_section(&vars, editor);
            for (key, style) in rules.syntax(editor) {
                syntax.insert(key, style);
            }
            syntax
        }
        _ => BTreeMap::new(),
    };
    let preview = match kind {
        Kind::Whole | Kind::Preview => {
            let mut preview = preview_section(&vars, editor);
            preview.extend(rules.preview(editor));
            preview
        }
        _ => BTreeMap::new(),
    };
    Ok(ThemeFile {
        name: name(&package),
        author: author(&package),
        license: text(&package, "license").unwrap_or_else(|| "unknown".into()),
        source: source(&package),
        description: text(&package, "description"),
        appearance,
        ui,
        syntax,
        preview,
    })
}

fn text(package: &Value, key: &str) -> Option<String> {
    package
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn kind(package: &Value) -> Result<Kind, String> {
    match package.get("theme") {
        Some(Value::Bool(true)) => Ok(Kind::Whole),
        Some(Value::String(s)) => match s.as_str() {
            "ui" => Ok(Kind::Ui),
            "syntax" => Ok(Kind::Syntax),
            "preview" => Ok(Kind::Preview),
            other => Err(format!(
                "This package's \"theme\" is \"{other}\"; Kanso reads theme packages whose \"theme\" is true, \"ui\", \"syntax\" or \"preview\"."
            )),
        },
        _ => Err(
            "This package isn't a theme package: its package.json doesn't say \"theme\".".into(),
        ),
    }
}

/// The package's stylesheets, each after the files it `@import`s: the ones
/// `styleSheets` names (under `styles/`), else every `.css` and `.less` file
/// in `styles/`. A file that can't be read is passed over.
fn load_sheets(dir: &Path, package: &Value) -> Vec<Sheet> {
    let styles = dir.join("styles");
    let listed: Vec<PathBuf> = match package.get("styleSheets").and_then(Value::as_array) {
        Some(names) => names
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|name| {
                [styles.join(name), dir.join(name)]
                    .into_iter()
                    .find(|p| p.is_file())
            })
            .collect(),
        None => {
            let mut found: Vec<PathBuf> = std::fs::read_dir(&styles)
                .map(|read| {
                    read.filter_map(Result::ok)
                        .map(|e| e.path())
                        .filter(|p| p.extension().is_some_and(|x| x == "css" || x == "less"))
                        .collect()
                })
                .unwrap_or_default();
            found.sort();
            found
        }
    };
    let root = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for path in listed {
        load_sheet(&path, &root, &mut seen, 0, &mut out);
    }
    out
}

/// The largest stylesheet read; compiled UI themes run to a megabyte.
const MAX_SHEET: u64 = 16 * 1024 * 1024;

fn load_sheet(
    path: &Path,
    root: &Path,
    seen: &mut HashSet<PathBuf>,
    depth: usize,
    out: &mut Vec<Sheet>,
) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    // Imports stay inside the package.
    if depth > 8 || !path.starts_with(root) || !seen.insert(path.clone()) {
        return;
    }
    if std::fs::metadata(&path).map_or(true, |m| m.len() > MAX_SHEET) {
        return;
    }
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let sheet = css::parse(&String::from_utf8_lossy(&bytes));
    let folder = path.parent().unwrap_or(root);
    for import in &sheet.imports {
        if import.contains("://") {
            continue;
        }
        let base = folder.join(import);
        let candidates = if base.extension().is_some_and(|x| x == "less" || x == "css") {
            vec![base]
        } else {
            vec![
                base.with_extension("less"),
                base.with_extension("css"),
                base,
            ]
        };
        if let Some(found) = candidates.into_iter().find(|p| p.is_file()) {
            load_sheet(&found, root, seen, depth + 1, out);
        }
    }
    out.push(sheet);
}

fn read_palette(dir: &Path) -> Result<Option<serde_json::Map<String, Value>>, String> {
    let path = dir.join("palette.json");
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("Its palette.json couldn't be read: {err}."))?;
    match serde_json::from_str(&text) {
        Ok(Value::Object(map)) => Ok(Some(map)),
        Ok(_) => Err("Its palette.json should be an object of CSS variables.".into()),
        Err(err) => Err(format!("Its palette.json isn't valid JSON: {err}.")),
    }
}

/// Every declaration, with the sheets' LESS variables in place.
fn with_less_variables(sheets: &[Sheet]) -> Vec<Declaration> {
    let variables: HashMap<String, String> = sheets
        .iter()
        .flat_map(|s| s.less_variables.iter().cloned())
        .collect();
    sheets
        .iter()
        .flat_map(|s| s.declarations.iter())
        .map(|d| Declaration {
            value: css::resolve_less(&d.value, &variables),
            ..d.clone()
        })
        .collect()
}

// MARK: Variables

/// The theme's CSS variables, and the default theme's beneath them.
#[derive(Clone)]
struct Vars {
    /// What the theme sets itself.
    own: HashMap<String, String>,
    /// The default theme of the same appearance.
    base: HashMap<String, String>,
    /// Both, the theme's winning: what `var()` sees.
    all: HashMap<String, String>,
    dark: bool,
    /// Whether a variable the theme doesn't set takes the default theme's; an
    /// older single-part syntax or preview theme leaves the rest to Kanso.
    use_base: bool,
}

impl Vars {
    /// The raw value of the first of `names` that reads as `parse` wants:
    /// the theme's own first, then (when it applies) the default theme's.
    fn find<T>(&self, names: &[&str], parse: impl Fn(&str) -> Option<T>) -> Option<T> {
        let read = |map: &HashMap<String, String>| {
            names.iter().find_map(|name| {
                let raw = map.get(*name)?;
                let resolved = css::resolve_vars(raw, &self.all)?;
                parse(&css::resolve_light_dark(&resolved, self.dark))
            })
        };
        read(&self.own).or_else(|| {
            if self.use_base {
                read(&self.base)
            } else {
                None
            }
        })
    }

    fn color(&self, names: &[&str]) -> Option<Color> {
        self.find(names, color::extract)
    }

    /// The color, painted over `under` when it is translucent.
    fn over(&self, names: &[&str], under: Rgb) -> Option<Rgb> {
        self.color(names).map(|c| c.over(under))
    }

    fn italic(&self, name: Option<&str>) -> Option<bool> {
        self.find(&[name?], font_style)
    }

    fn weight(&self, name: Option<&str>) -> Option<u16> {
        self.find(&[name?], font_weight)
    }
}

fn font_style(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "italic" | "oblique" => Some(true),
        "normal" => Some(false),
        _ => None,
    }
}

fn font_weight(value: &str) -> Option<u16> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bold" | "bolder" => Some(700),
        "normal" => Some(400),
        "lighter" => Some(300),
        n => n
            .parse::<f64>()
            .ok()
            .filter(|w| (1.0..=1000.0).contains(w))
            .map(|w| ((w / 100.).round() * 100.).clamp(100., 900.) as u16),
    }
}

/// What a translucent window shows behind the panes: macOS's window
/// background, roughly.
fn backdrop(dark: bool) -> Rgb {
    Rgb(if dark { 0x1E1E1E } else { 0xF5F5F5 })
}

/// The `ui` section from the package's UI variables, and the editor's color
/// (what syntax and preview colors are painted over).
fn ui_section(vars: &Vars) -> (BTreeMap<&'static str, Color>, Rgb) {
    let mut ui = BTreeMap::new();
    let mut put = |key: &'static str, rgb: Option<Rgb>| {
        if let Some(rgb) = rgb {
            ui.insert(key, Color { rgb, alpha: 0xFF });
        }
    };
    let page = vars.over(&["--page-background"], backdrop(vars.dark));
    let app = page.unwrap_or(backdrop(vars.dark));
    let editor = vars
        .over(&["--editor-background", "--editor-background-color"], app)
        .unwrap_or(app);
    let sidebar = vars.over(&["--sidebar-background"], app);
    let list = vars.over(&["--note-list-bar-background"], app);
    let elevated = vars.over(
        &[
            "--dropdown-menu-background",
            "--popup-background",
            "--editor-tooltip-background-color",
        ],
        editor,
    );
    let accent = vars.over(&["--primary-color"], editor);
    let button = vars.over(
        &["--primary-button-background-color", "--primary-color"],
        editor,
    );
    let text = vars.over(&["--text-color", "--editor-foreground-color"], editor);
    // A highlight Kanso draws body text on; the packages' filled ones (a solid
    // accent under white text) are left for Kanso to derive.
    let highlight =
        |rgb: Option<Rgb>| rgb.filter(|bg| text.is_none_or(|t| contrast(t, *bg) >= 4.5));
    put("editor", Some(editor));
    put("text", text);
    put("accent", accent);
    put("app", page);
    put("sidebar", sidebar);
    put("list", list);
    put("elevated", elevated);
    put(
        "text_secondary",
        vars.over(
            &["--muted-text-color", "--note-list-view-item-color"],
            list.unwrap_or(editor),
        ),
    );
    put(
        "text_muted",
        vars.over(
            &[
                "--sidebar-menu-section-color",
                "--unselected-text-color",
                "--dark-text-color",
            ],
            sidebar.unwrap_or(editor),
        ),
    );
    put(
        "divider",
        vars.over(&["--solid-border-color", "--border-color"], editor),
    );
    put("link", vars.over(&["--link-color"], editor));
    put(
        "row_active",
        highlight(vars.over(
            &["--note-list-view-item-selected-background"],
            list.unwrap_or(editor),
        )),
    );
    put(
        "row_hover",
        highlight(vars.over(
            &["--selection-list-hover-background"],
            list.unwrap_or(editor),
        )),
    );
    put(
        "sidebar_active",
        highlight(vars.over(
            &["--sidebar-menu-item-active-background"],
            sidebar.unwrap_or(editor),
        )),
    );
    put(
        "hover",
        highlight(vars.over(
            &[
                "--dropdown-hovered-item-background",
                "--menu-hover-item-background",
            ],
            elevated.unwrap_or(editor),
        )),
    );
    put(
        "active_line",
        vars.over(&["--editor-active-line-background-color"], editor),
    );
    put("button", button);
    put(
        "button_text",
        vars.over(&["--primary-button-text-color"], button.unwrap_or(editor)),
    );
    put(
        "success",
        vars.over(&["--green-text-color", "--success-color"], editor),
    );
    put(
        "warning",
        vars.over(
            &[
                "--yellow-text-color",
                "--orange-text-color",
                "--warning-color",
            ],
            editor,
        ),
    );
    put(
        "danger",
        vars.over(&["--red-text-color", "--error-color"], editor),
    );
    put("caret", vars.over(&["--editor-caret-color"], editor));
    // Selection stays translucent, as Kanso draws it over the text.
    if let Some(selection) = vars.color(&[
        "--editor-focused-selection-background",
        "--editor-selection-background",
    ]) && selection.alpha > 0
    {
        ui.insert("selection", selection);
    }
    (ui, editor)
}

/// For an older single-part syntax or preview theme: the editor, text and
/// accent Kanso needs, from the theme where it gives them, else the default
/// theme's.
fn required_ui(vars: &Vars, rules: &Rules) -> (BTreeMap<&'static str, Color>, Rgb) {
    let mut ui = BTreeMap::new();
    let app = backdrop(vars.dark);
    let editor = rules
        .background
        .map(|c| c.over(app))
        .or_else(|| vars.over(&["--editor-background", "--editor-background-color"], app))
        .unwrap_or(app);
    let text = rules
        .text
        .map(|c| c.over(editor))
        .or_else(|| vars.over(&["--editor-foreground-color", "--text-color"], editor))
        .unwrap_or(if vars.dark {
            Rgb(0xDADADA)
        } else {
            Rgb(0x222222)
        });
    let accent = vars
        .over(&["--primary-color"], editor)
        .unwrap_or(Rgb(0x3B82F6));
    ui.insert("editor", Color::opaque(editor.0));
    ui.insert("text", Color::opaque(text.0));
    ui.insert("accent", Color::opaque(accent.0));
    for (key, color) in [("caret", rules.caret), ("active_line", rules.active_line)] {
        if let Some(color) = color {
            ui.insert(key, Color::opaque(color.over(editor).0));
        }
    }
    if let Some(selection) = rules.selection.filter(|c| c.alpha > 0) {
        ui.insert("selection", selection);
    }
    (ui, editor)
}

/// Each syntax key, the variables that color it (the first set wins), and
/// the variables for its slant and its weight.
type SyntaxVars = (
    &'static str,
    &'static [&'static str],
    Option<&'static str>,
    Option<&'static str>,
);

const SYNTAX_VARS: &[SyntaxVars] = &[
    (
        "text",
        &["--editor-foreground-color", "--syntax-content-color"],
        None,
        None,
    ),
    (
        "heading",
        &["--syntax-heading-color"],
        None,
        Some("--syntax-heading-font-weight"),
    ),
    (
        "emphasis",
        &["--syntax-emphasis-color"],
        Some("--syntax-emphasis-font-style"),
        None,
    ),
    (
        "strong",
        &["--syntax-strong-color"],
        None,
        Some("--syntax-strong-font-weight"),
    ),
    ("link", &["--syntax-link-color"], None, None),
    ("url", &["--syntax-url-color"], None, None),
    ("code", &["--syntax-monospace-color"], None, None),
    ("code_block", &["--md-codeblock-color"], None, None),
    (
        "quote",
        &["--syntax-quote-color"],
        Some("--syntax-quote-font-style"),
        None,
    ),
    ("list_marker", &["--md-list-mark-color"], None, None),
    (
        "block_marker",
        &["--syntax-meta-color", "--syntax-punctuation-color"],
        None,
        None,
    ),
    (
        "marker",
        &["--syntax-meta-color", "--syntax-punctuation-color"],
        None,
        None,
    ),
    ("hr", &["--syntax-content-separator-color"], None, None),
    ("front_matter", &["--syntax-meta-color"], None, None),
    ("html", &["--syntax-tag-name-color"], None, None),
    (
        "comment",
        &["--syntax-comment-color"],
        Some("--syntax-comment-font-style"),
        None,
    ),
    (
        "comment_doc",
        &["--syntax-doc-comment-color"],
        Some("--syntax-doc-comment-font-style"),
        None,
    ),
    ("keyword", &["--syntax-keyword-color"], None, None),
    ("string", &["--syntax-string-color"], None, None),
    (
        "string_special",
        &["--syntax-string-special-color", "--syntax-regexp-color"],
        None,
        None,
    ),
    ("number", &["--syntax-number-color"], None, None),
    ("boolean", &["--syntax-bool-color"], None, None),
    (
        "constant",
        &[
            "--syntax-name-constant-color",
            "--syntax-variable-name-constant-color",
        ],
        None,
        None,
    ),
    (
        "function",
        &[
            "--syntax-name-function-color",
            "--syntax-variable-name-function-color",
        ],
        None,
        None,
    ),
    ("type", &["--syntax-class-name-color"], None, None),
    ("variable", &["--syntax-variable-name-color"], None, None),
    ("variable_special", &["--syntax-self-color"], None, None),
    ("property", &["--syntax-property-name-color"], None, None),
    ("attribute", &["--syntax-attribute-name-color"], None, None),
    ("tag", &["--syntax-tag-name-color"], None, None),
    ("label", &["--syntax-label-name-color"], None, None),
    ("operator", &["--syntax-operator-color"], None, None),
    ("punctuation", &["--syntax-punctuation-color"], None, None),
    ("bracket", &["--syntax-bracket-color"], None, None),
];

fn syntax_section(vars: &Vars, editor: Rgb) -> BTreeMap<&'static str, TokenStyle> {
    SYNTAX_VARS
        .iter()
        .filter_map(|(key, colors, italic, weight)| {
            let style = TokenStyle {
                color: vars.over(colors, editor),
                italic: vars.italic(*italic),
                weight: vars.weight(*weight),
            };
            (!style.is_empty()).then_some((*key, style))
        })
        .collect()
}

/// Each preview key and the variables that color it; backgrounds are
/// painted over the editor, and text over the background it sits on.
const PREVIEW_VARS: &[(&str, &[&str])] = &[
    ("text", &["--text-color"]),
    ("heading", &["--mde-preview-heading-color"]),
    ("link", &["--mde-preview-link-color"]),
    ("quote", &["--mde-preview-blockquote-text-color"]),
    (
        "border",
        &[
            "--mde-preview-table-border-color",
            "--mde-preview-blockquote-border-color",
        ],
    ),
    (
        "code_background",
        &["--mde-preview-codeblock-background-color"],
    ),
    ("code_text", &["--mde-preview-codeblock-text-color"]),
    (
        "inline_code_background",
        &["--mde-preview-inline-code-background-color"],
    ),
    (
        "inline_code_text",
        &["--mde-preview-inline-code-text-color"],
    ),
    (
        "table_header_background",
        &["--mde-preview-table-head-background-color"],
    ),
];

fn preview_section(vars: &Vars, editor: Rgb) -> BTreeMap<&'static str, Color> {
    let mut out = BTreeMap::new();
    let get = |key: &str| {
        PREVIEW_VARS
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, names)| vars.color(names))
    };
    let code_background = get("code_background").map(|c| c.over(editor));
    let inline_background = get("inline_code_background").map(|c| c.over(editor));
    for (key, _) in PREVIEW_VARS {
        let under = match *key {
            "code_text" => code_background.unwrap_or(editor),
            "inline_code_text" => inline_background.unwrap_or(editor),
            _ => editor,
        };
        if let Some(color) = get(key) {
            out.insert(*key, Color::opaque(color.over(under).0));
        }
    }
    out
}

// MARK: Single-part theme rules

/// A syntax key and the color, slant and weight rules give it.
type FoundStyle = (&'static str, Option<Color>, Option<bool>, Option<u16>);

/// What an older single-part syntax or preview theme's rules say.
#[derive(Default)]
struct Rules {
    /// `.CodeMirror`'s or `.mde-preview`'s own background and color.
    background: Option<Color>,
    text: Option<Color>,
    caret: Option<Color>,
    selection: Option<Color>,
    active_line: Option<Color>,
    /// Syntax key to its color, slant and weight, each as found.
    syntax: Vec<FoundStyle>,
    preview: Vec<(&'static str, Color)>,
}

/// Each syntax key and the CodeMirror classes that style it, most specific
/// first.
const CODEMIRROR: &[(&str, &[&str])] = &[
    ("heading", &["cm-header"]),
    ("strong", &["cm-strong"]),
    ("emphasis", &["cm-em"]),
    ("link", &["cm-link"]),
    ("url", &["cm-url"]),
    ("comment", &["cm-comment"]),
    // CodeMirror's Markdown mode marks inline code as a comment.
    ("code", &["cm-inline-code", "cm-comment"]),
    ("keyword", &["cm-keyword"]),
    ("string", &["cm-string"]),
    ("string_special", &["cm-string-2"]),
    ("number", &["cm-number"]),
    ("constant", &["cm-atom"]),
    ("function", &["cm-def"]),
    ("variable", &["cm-variable", "cm-variable-2"]),
    ("type", &["cm-variable-3", "cm-type", "cm-builtin"]),
    ("property", &["cm-property"]),
    ("attribute", &["cm-attribute"]),
    ("tag", &["cm-tag"]),
    ("html", &["cm-tag"]),
    ("operator", &["cm-operator"]),
    ("punctuation", &["cm-punctuation"]),
    ("bracket", &["cm-bracket"]),
    ("quote", &["cm-quote"]),
    ("hr", &["cm-hr"]),
    ("front_matter", &["cm-meta"]),
    (
        "block_marker",
        &["cm-formatting-header", "cm-formatting-quote"],
    ),
    (
        "list_marker",
        &[
            "cm-formatting-list",
            "cm-formatting-list-ul",
            "cm-formatting-list-ol",
        ],
    ),
    ("task", &["cm-formatting-task"]),
    ("escape", &["cm-formatting-escape"]),
    (
        "marker",
        &[
            "cm-formatting",
            "cm-formatting-code",
            "cm-formatting-strong",
            "cm-formatting-em",
        ],
    ),
];

impl Rules {
    fn read(declarations: &[Declaration], kind: Kind) -> Rules {
        let mut rules = Rules::default();
        match kind {
            Kind::Syntax => rules.read_codemirror(declarations),
            Kind::Preview => rules.read_preview(declarations),
            Kind::Whole | Kind::Ui => {}
        }
        rules
    }

    fn read_codemirror(&mut self, declarations: &[Declaration]) {
        let backgrounds = ["background-color", "background"];
        self.background = best(declarations, &["CodeMirror"], &backgrounds);
        self.text = best(declarations, &["CodeMirror"], &["color"]);
        self.caret = best(
            declarations,
            &["CodeMirror-cursor"],
            &["border-left-color", "border-left", "border-color", "border"],
        );
        self.selection = best(declarations, &["CodeMirror-selected"], &backgrounds);
        self.active_line = best(
            declarations,
            &["CodeMirror-activeline-background"],
            &backgrounds,
        );
        for (key, classes) in CODEMIRROR {
            let color = best(declarations, classes, &["color"]);
            let italic = best_value(declarations, classes, &["font-style"], font_style);
            let weight = best_value(declarations, classes, &["font-weight"], font_weight);
            if color.is_some() || italic.is_some() || weight.is_some() {
                self.syntax.push((key, color, italic, weight));
            }
        }
    }

    fn read_preview(&mut self, declarations: &[Declaration]) {
        for d in declarations {
            for selector in super::color::split_commas(&d.selector) {
                if !selector.contains("mde-preview") {
                    continue;
                }
                let last = css::compound(css::last_compound(selector));
                if last.pseudo {
                    continue;
                }
                let in_pre = selector
                    .split_whitespace()
                    .rev()
                    .skip(1)
                    .any(|s| s.starts_with("pre"));
                let prop = d.property.as_str();
                let background = prop == "background" || prop == "background-color";
                let border = prop.starts_with("border")
                    && !prop.contains("radius")
                    && !prop.contains("width");
                let Some(color) = color::extract(&d.value) else {
                    continue;
                };
                let element = last.element.to_ascii_lowercase();
                let key = match (element.as_str(), prop) {
                    ("", "color") if last.classes.contains(&"mde-preview") => {
                        self.text = Some(color);
                        None
                    }
                    ("", _) if background && last.classes.contains(&"mde-preview") => {
                        self.background = Some(color);
                        None
                    }
                    ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", "color") => Some("heading"),
                    ("a", "color") => Some("link"),
                    ("blockquote", "color") => Some("quote"),
                    ("blockquote" | "table" | "th" | "td" | "tr" | "hr", _) if border => {
                        Some("border")
                    }
                    ("pre", "color") => Some("code_text"),
                    ("pre", _) if background => Some("code_background"),
                    ("code", "color") => Some(if in_pre {
                        "code_text"
                    } else {
                        "inline_code_text"
                    }),
                    ("code", _) if background => Some(if in_pre {
                        "code_background"
                    } else {
                        "inline_code_background"
                    }),
                    ("th" | "thead", _) if background => Some("table_header_background"),
                    _ => None,
                };
                if let Some(key) = key {
                    self.preview.push((key, color));
                }
            }
        }
    }

    /// The syntax styles found, colors painted over `editor`.
    fn syntax(&self, editor: Rgb) -> Vec<(&'static str, TokenStyle)> {
        self.syntax
            .iter()
            .map(|(key, color, italic, weight)| {
                (
                    *key,
                    TokenStyle {
                        color: color.map(|c| c.over(editor)),
                        italic: *italic,
                        weight: *weight,
                    },
                )
            })
            .collect()
    }

    /// The preview colors found, the last rule for each winning; text over
    /// the editor, backgrounds over it too.
    fn preview(&self, editor: Rgb) -> BTreeMap<&'static str, Color> {
        let mut out: BTreeMap<&'static str, Color> = BTreeMap::new();
        for (key, color) in &self.preview {
            out.insert(key, Color::opaque(color.over(editor).0));
        }
        if let Some(text) = self.text {
            out.insert("text", Color::opaque(text.over(editor).0));
        }
        out
    }
}

/// The color the best rule for `classes` gives with one of `properties`.
fn best(declarations: &[Declaration], classes: &[&str], properties: &[&str]) -> Option<Color> {
    best_value(declarations, classes, properties, color::extract)
}

/// The value the rules for `classes` give with one of `properties`, read
/// by `parse`. Only a rule whose last compound is just the class (or
/// `span.` the class) counts: `.cm-keyword.cm-em` styles emphasis inside a
/// keyword, not keywords. An earlier class in `classes` beats a later one,
/// and among equals the later rule wins, as in CSS. States and parts
/// (`:hover`, `::before`) are passed over.
fn best_value<T>(
    declarations: &[Declaration],
    classes: &[&str],
    properties: &[&str],
    parse: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    let mut found: Option<(usize, T)> = None;
    for d in declarations {
        if !properties.contains(&d.property.as_str()) {
            continue;
        }
        for selector in super::color::split_commas(&d.selector) {
            let last = css::compound(css::last_compound(selector));
            let plain = matches!(last.element, "" | "span");
            if last.pseudo || !plain || last.classes.len() != 1 {
                continue;
            }
            let Some(rank) = classes.iter().position(|class| *class == last.classes[0]) else {
                continue;
            };
            if found.as_ref().is_some_and(|(r, _)| *r < rank) {
                continue;
            }
            if let Some(value) = parse(&d.value) {
                found = Some((rank, value));
            }
        }
    }
    found.map(|(_, value)| value)
}

// MARK: Appearance and names

/// Whether the theme is light or dark: as its package says, else as its
/// colors say (light text on a dark editor is a dark theme), else as its
/// `color-scheme` or its name says, else light.
fn appearance(
    package: &Value,
    palette: Option<&serde_json::Map<String, Value>>,
    own: &HashMap<String, String>,
    rules: &Rules,
    declarations: &[Declaration],
) -> Appearance {
    let said = package
        .get("themeAppearance")
        .or_else(|| palette.and_then(|p| p.get("themeAppearance")))
        .and_then(Value::as_str);
    match said {
        Some("dark") => return Appearance::Dark,
        Some("light") => return Appearance::Light,
        _ => {}
    }
    let vars = Vars {
        own: own.clone(),
        base: HashMap::new(),
        all: own.clone(),
        dark: false,
        use_base: false,
    };
    let grey = Rgb(0x808080);
    let background = rules
        .background
        .or_else(|| {
            vars.color(&[
                "--editor-background",
                "--editor-background-color",
                "--page-background",
            ])
        })
        .map(|c| c.over(grey));
    let text = rules
        .text
        .or_else(|| vars.color(&["--text-color", "--editor-foreground-color"]));
    let dark = match (background, text) {
        (Some(bg), Some(text)) => Some(luminance(text.over(bg)) > luminance(bg)),
        (Some(bg), None) => Some(luminance(bg) < 0.18),
        (None, Some(text)) => Some(luminance(text.over(grey)) > 0.4),
        (None, None) => None,
    };
    let scheme = declarations
        .iter()
        .rev()
        .find(|d| d.property == "color-scheme")
        .and_then(|d| match d.value.trim() {
            v if v.starts_with("dark") => Some(true),
            v if v.starts_with("light") => Some(false),
            _ => None,
        });
    let named = || {
        let name = package.get("name").and_then(Value::as_str).unwrap_or("");
        if name.contains("dark") {
            Some(true)
        } else if name.contains("light") {
            Some(false)
        } else {
            None
        }
    };
    if dark.or(scheme).or_else(named).unwrap_or(false) {
        Appearance::Dark
    } else {
        Appearance::Light
    }
}

/// The package's `displayName`, else its `name` in words: `default-dark`
/// becomes "Default Dark".
fn name(package: &Value) -> String {
    if let Some(display) = text(package, "displayName") {
        return display;
    }
    let raw = text(package, "name").unwrap_or_else(|| "Imported Theme".into());
    let raw = raw.rsplit('/').next().unwrap_or(&raw).to_owned();
    title_case(&raw)
}

/// `night-owl-dark-ui` as "Night Owl Dark UI".
pub fn title_case(name: &str) -> String {
    let words: Vec<String> = name
        .split(['-', '_', ' ', '.'])
        .filter(|w| !w.is_empty())
        .map(|word| match word.to_ascii_lowercase().as_str() {
            "ui" | "css" | "gfm" | "md" => word.to_ascii_uppercase(),
            _ => {
                let mut chars = word.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        })
        .collect();
    if words.is_empty() {
        "Imported Theme".into()
    } else {
        words.join(" ")
    }
}

/// `"Name <mail> (url)"` or `{ "name": … }`, as the name alone.
pub fn author(package: &Value) -> String {
    let raw = match package.get("author") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(o)) => o
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        _ => String::new(),
    };
    let name = raw.split(['<', '(']).next().unwrap_or("").trim();
    if name.is_empty() {
        "unknown".into()
    } else {
        name.to_owned()
    }
}

/// The repository as a web address, else the homepage.
fn source(package: &Value) -> Option<String> {
    let repository = match package.get("repository") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(o)) => o.get("url").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    };
    repository
        .and_then(|r| repository_url(&r))
        .or_else(|| text(package, "homepage"))
}

/// A `package.json` repository as an https address: `git+https://…​.git`,
/// `git://`, `git@host:owner/repo`, `github:owner/repo` and `owner/repo` all
/// become `https://host/owner/repo`.
pub fn repository_url(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let mut url = raw.strip_prefix("git+").unwrap_or(raw).to_owned();
    if let Some(rest) = url.strip_prefix("git://") {
        url = format!("https://{rest}");
    } else if let Some(rest) = url.strip_prefix("ssh://git@") {
        url = format!("https://{rest}");
    } else if let Some(rest) = url.strip_prefix("git@") {
        url = format!("https://{}", rest.replacen(':', "/", 1));
    } else if let Some(rest) = url.strip_prefix("github:") {
        url = format!("https://github.com/{rest}");
    } else if !url.contains("://") && url.matches('/').count() == 1 && !url.starts_with('/') {
        url = format!("https://github.com/{url}");
    }
    if let Some(stripped) = url.strip_suffix(".git") {
        url = stripped.to_owned();
    }
    url = url.trim_end_matches('/').to_owned();
    (url.starts_with("https://") || url.starts_with("http://")).then_some(url)
}

/// Every variable the mapping reads, for the test that keeps the base
/// table complete.
#[cfg(test)]
pub(super) fn variables_read() -> Vec<&'static str> {
    let source = include_str!("package.rs");
    let mut names: Vec<&'static str> = source
        .match_indices("\"--")
        .filter_map(|(at, _)| {
            let rest = &source[at + 1..];
            let end = rest.find('"')?;
            let name = &rest[..end];
            name.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                .then_some(name)
        })
        .filter(|n| n.len() > 2)
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{author, import, repository_url, title_case, variables_read};
    use crate::theme::file::{Appearance, contrast};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Words: a package name in any mix of separators becomes its words,
        /// each capitalized, in order.
        #[test]
        fn prop_title_case_keeps_the_words(words in prop::collection::vec("[a-z][a-z0-9]{0,7}", 1..5), sep in prop::sample::select(vec!["-", "_", " "])) {
            let title = title_case(&words.join(sep));
            let got: Vec<&str> = title.split(' ').collect();
            prop_assert_eq!(got.len(), words.len());
            for (word, got) in words.iter().zip(got) {
                prop_assert_eq!(got.to_lowercase(), word.clone());
                prop_assert!(got.chars().next().unwrap().is_uppercase() || got.chars().next().unwrap().is_ascii_digit());
            }
        }

        /// Equivalence: every way npm writes a GitHub repository gives the
        /// same https address.
        #[test]
        fn prop_repository_forms_agree(owner in "[a-z][a-z0-9-]{0,10}", repo in "[a-z][a-z0-9-]{0,10}") {
            let want = format!("https://github.com/{owner}/{repo}");
            for form in [
                format!("git+https://github.com/{owner}/{repo}.git"),
                format!("https://github.com/{owner}/{repo}"),
                format!("git://github.com/{owner}/{repo}.git"),
                format!("git@github.com:{owner}/{repo}.git"),
                format!("github:{owner}/{repo}"),
                format!("{owner}/{repo}"),
            ] {
                prop_assert_eq!(repository_url(&form), Some(want.clone()), "{}", form);
            }
        }

        /// Names: an author string with any email and address reads as the
        /// name alone.
        #[test]
        fn prop_author_is_the_name(name in "[A-Z][a-z]{1,8}( [A-Z][a-z]{1,8})?", mail in "[a-z]{1,6}@[a-z]{1,6}\\.app") {
            for raw in [format!("{name}<{mail}>"), format!("{name} <{mail}> (https://x.app)"), name.clone()] {
                prop_assert_eq!(author(&serde_json::json!({ "author": raw })), name.clone());
            }
            prop_assert_eq!(author(&serde_json::json!({ "author": { "name": name.clone() } })), name);
        }
    }

    /// The base table has a value, in each appearance, for every variable
    /// the mapping reads, and each color variable reads as a color.
    #[test]
    fn the_base_table_covers_the_mapping() {
        for dark in [false, true] {
            let table = super::package_base::defaults(dark);
            for name in variables_read() {
                let value = table.iter().find(|(k, _)| *k == name).map(|(_, v)| *v);
                let value = value.unwrap_or_else(|| panic!("{name} isn't in the base table"));
                if !name.contains("font") {
                    assert!(super::color::extract(value).is_some(), "{name}: {value}");
                }
            }
        }
    }

    fn write(dir: &Path, files: &[(&str, &str)]) {
        for (name, text) in files {
            let path = dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }

    /// A version 5 UI theme in the style of Night Owl: custom properties
    /// that refer to each other across two files, and no appearance.
    #[test]
    fn a_night_owl_style_ui_theme() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &[
                (
                    "package.json",
                    r#"{"name": "night-owl-dark-ui", "theme": "ui", "styleSheets": ["theme.css", "tokens.css"],
                        "author": {"name": "Sami Korpela"}, "license": "MIT",
                        "repository": {"type": "git", "url": "https://github.com/skipadu/inkdrop-night-owl-ui"}}"#,
                ),
                (
                    "styles/theme.css",
                    ":root { --editor-background: var(--color-owl-bg); --sidebar-background: var(--color-owl-bg);
                       --note-list-bar-background: var(--color-owl-bg);
                       --note-list-view-item-selected-background: var(--color-owl-b); }",
                ),
                (
                    "styles/tokens.css",
                    ":root { --color-owl-bg: #001628; --color-owl-b: #102a44; }",
                ),
            ],
        );
        let theme = import(dir.path()).unwrap();
        assert_eq!(theme.name, "Night Owl Dark UI");
        assert_eq!(theme.author, "Sami Korpela");
        assert_eq!(
            theme.source.as_deref(),
            Some("https://github.com/skipadu/inkdrop-night-owl-ui")
        );
        assert_eq!(
            theme.appearance,
            Appearance::Dark,
            "from its dark editor, with no appearance given"
        );
        assert_eq!(theme.ui["editor"].hex(), "#001628");
        assert_eq!(theme.ui["sidebar"].hex(), "#001628");
        assert_eq!(theme.ui["row_active"].hex(), "#102A44");
        // The text comes from the default dark theme, and reads.
        assert!(contrast(theme.ui["text"].rgb, theme.ui["editor"].rgb) > 7.);
        // A UI theme leaves the syntax and preview to Kanso.
        assert!(theme.syntax.is_empty() && theme.preview.is_empty());
    }

    /// A version 5 syntax theme in the style of Material Dark Mod: LESS
    /// variables from an imported file, nested CodeMirror rules, and
    /// overrides that a more specific rule mustn't win.
    #[test]
    fn a_material_style_syntax_theme() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &[
                (
                    "package.json",
                    r#"{"name": "material-dark-mod-syntax", "theme": "syntax", "styleSheets": ["material.less"],
                        "repository": {"type": "git", "url": "git+https://github.com/yuru7/inkdrop-material-dark-mod-syntax.git"}}"#,
                ),
                ("styles/colors.less", "@base00: #2e3235; @base06: #e0e0e0; @base-yellow: #facf4e; @base-purple: #cf6edf; @base-green: #6abf69; @base03: #707d8b;"),
                (
                    "styles/material.less",
                    "@import './colors';
                    .editor {
                      .CodeMirror {
                        color: @base06;
                        background-color: @base00;
                        .cm-header { color: @base-yellow; }
                        .cm-keyword { color: @base-purple; }
                        .cm-list-indent ~ .cm-keyword { color: inherit; }
                        .cm-keyword.cm-formatting-list-ul { color: @base-green; }
                        .cm-quote { color: @base-purple; font-style: italic; }
                        .cm-comment { color: @base03; }
                        .CodeMirror-activeline-background { background-color: fade(@base03, 13%) !important; }
                      }
                      .CodeMirror .CodeMirror-cursor { border-left: 2px solid #ffe9a0; }
                    }",
                ),
            ],
        );
        let theme = import(dir.path()).unwrap();
        assert_eq!(theme.name, "Material Dark Mod Syntax");
        assert_eq!(theme.author, "unknown");
        assert_eq!(
            theme.source.as_deref(),
            Some("https://github.com/yuru7/inkdrop-material-dark-mod-syntax")
        );
        assert_eq!(theme.appearance, Appearance::Dark);
        assert_eq!(theme.ui["editor"].hex(), "#2E3235");
        assert_eq!(theme.ui["text"].hex(), "#E0E0E0");
        assert_eq!(theme.ui["caret"].hex(), "#FFE9A0");
        assert!(theme.ui.contains_key("accent"));
        assert_eq!(theme.syntax["heading"].color.unwrap().hex(), "#FACF4E");
        assert_eq!(
            theme.syntax["keyword"].color.unwrap().hex(),
            "#CF6EDF",
            "the plain rule wins"
        );
        assert_eq!(theme.syntax["quote"].italic, Some(true));
        assert_eq!(
            theme.syntax["code"].color.unwrap().hex(),
            "#707D8B",
            "inline code is a comment"
        );
        // A syntax theme leaves the rest of the UI and the preview to Kanso.
        assert!(!theme.ui.contains_key("sidebar"));
        assert!(theme.preview.is_empty());
    }

    /// A version 6 theme with only a palette subset: what it gives is
    /// used, and the rest comes from the default theme of its appearance.
    #[test]
    fn a_palette_subset() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &[
                (
                    "package.json",
                    r#"{"name": "ocean", "theme": true, "themeAppearance": "dark", "styleSheets": ["ui.css"]}"#,
                ),
                (
                    "styles/ui.css",
                    "@layer theme.ui { :root { --primary-color: hsl(var(--hsl-teal-400)); --hsl-teal-400: 172deg 66% 50%; } }",
                ),
                (
                    "palette.json",
                    r##"{"themeAppearance": "dark", "--editor-background": "hsl(220deg 40% 12%)",
                        "--text-color": "hsl(0deg 0% 100% / 90%)", "--syntax-heading-color": "#ff79c6",
                        "--syntax-heading-font-weight": "bold", "--editor-focused-selection-background": "rgba(255, 255, 255, 0.2)",
                        "--mde-preview-codeblock-background-color": "rgba(255, 255, 255, 0.1)"}"##,
                ),
            ],
        );
        let theme = import(dir.path()).unwrap();
        assert_eq!(theme.name, "Ocean");
        assert_eq!(theme.author, "unknown");
        assert_eq!(theme.license, "unknown");
        assert_eq!(theme.ui["editor"].hex(), "#121B2B");
        assert_eq!(
            theme.ui["text"].hex(),
            "#E8E9EA",
            "white at 90% over the editor"
        );
        assert_eq!(
            theme.ui["accent"].hex(),
            "#2BD4BD",
            "an HSL primitive through var()"
        );
        assert_eq!(theme.ui["selection"].alpha, 51, "selection keeps its alpha");
        assert_eq!(
            theme.ui["sidebar"].hex(),
            "#171717",
            "the default dark theme's sidebar"
        );
        let heading = theme.syntax["heading"];
        assert_eq!(
            (heading.color.unwrap().hex(), heading.weight),
            ("#FF79C6".into(), Some(700))
        );
        assert_eq!(theme.preview["code_background"].hex(), "#2A3241");
    }

    #[test]
    fn not_a_theme() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            import(dir.path()).unwrap_err(),
            "This folder isn't a theme package: it has no package.json."
        );
        write(dir.path(), &[("package.json", r#"{"name": "plugin"}"#)]);
        assert!(
            import(dir.path())
                .unwrap_err()
                .contains("doesn't say \"theme\"")
        );
        write(
            dir.path(),
            &[("package.json", r#"{"name": "x", "theme": true}"#)],
        );
        assert!(
            import(dir.path())
                .unwrap_err()
                .contains("no stylesheets or palette.json")
        );
    }
}
