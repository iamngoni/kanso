//! Kanso's theme files: one JSON document per theme, in the format
//! `themes/schema/kanso-theme-v1.json` describes (and `themes/README.md`
//! explains).
//!
//! A theme names its appearance and gives colors in three sections: `ui`
//! (the panes, text and controls), `syntax` (the Markdown source and code in
//! the editor) and `preview` (the rendered note). Only `ui.editor`, `ui.text`
//! and `ui.accent` are required; every other color is derived from those, so a
//! theme can give a palette's few colors and still cover the whole app. A
//! derived text color is shaded until it reads on the surfaces it sits on,
//! and so is every syntax and preview color once it is paired with the panes
//! it is drawn on, which may be another theme's.
//!
//! Files are read leniently about what is missing and strictly about what is
//! there: an unknown key or a color that isn't `#RRGGBB` or `#RRGGBBAA` is an
//! error naming the key, so a typo doesn't pass silently.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{Colors, Rgb};

/// The format version this build reads and writes.
pub const SCHEMA_VERSION: u64 = 1;

/// Where the schema is published, for a theme's `$schema`.
pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/schema/kanso-theme-v1.json";

/// An sRGB color with an alpha, as a theme file writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub rgb: Rgb,
    pub alpha: u8,
}

impl Color {
    pub const fn opaque(rgb: u32) -> Color {
        Color {
            rgb: Rgb(rgb),
            alpha: 0xFF,
        }
    }

    /// `#RRGGBB`, or `#RRGGBBAA` when it isn't opaque.
    pub fn hex(self) -> String {
        if self.alpha == 0xFF {
            self.rgb.hex()
        } else {
            format!("{}{:02X}", self.rgb.hex(), self.alpha)
        }
    }

    /// Reads `#RRGGBB` or `#RRGGBBAA`, in either case.
    pub fn parse(text: &str) -> Option<Color> {
        let digits = text.strip_prefix('#')?;
        if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => Some(Color::opaque(value)),
            8 => Some(Color {
                rgb: Rgb(value >> 8),
                alpha: (value & 0xFF) as u8,
            }),
            _ => None,
        }
    }

    /// This color painted over `backdrop`.
    pub fn over(self, backdrop: Rgb) -> Rgb {
        mix(self.rgb, backdrop, f64::from(self.alpha) / 255.)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

impl Appearance {
    pub fn is_dark(self) -> bool {
        self == Appearance::Dark
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        }
    }
}

/// How a syntax token is drawn: any of a color, a slant and a weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenStyle {
    pub color: Option<Rgb>,
    pub italic: Option<bool>,
    /// 100 to 900.
    pub weight: Option<u16>,
}

impl TokenStyle {
    pub const fn color(rgb: Rgb) -> TokenStyle {
        TokenStyle {
            color: Some(rgb),
            italic: None,
            weight: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == TokenStyle::default()
    }

    /// The style a theme file writes: a bare color when that is all it is.
    fn to_json(self) -> Value {
        match self {
            TokenStyle {
                color: Some(color),
                italic: None,
                weight: None,
            } => Value::String(color.hex()),
            _ => {
                let mut map = Map::new();
                if let Some(color) = self.color {
                    map.insert("color".into(), color.hex().into());
                }
                if let Some(italic) = self.italic {
                    map.insert(
                        "font_style".into(),
                        if italic { "italic" } else { "normal" }.into(),
                    );
                }
                if let Some(weight) = self.weight {
                    map.insert("font_weight".into(), weight.into());
                }
                Value::Object(map)
            }
        }
    }

    /// The style object for a highlight theme, as the desktop's component
    /// library reads it.
    pub fn to_highlight_json(self) -> Value {
        let mut map = Map::new();
        if let Some(color) = self.color {
            map.insert("color".into(), color.hex().into());
        }
        if let Some(italic) = self.italic {
            map.insert(
                "font_style".into(),
                if italic { "italic" } else { "normal" }.into(),
            );
        }
        if let Some(weight) = self.weight {
            map.insert("font_weight".into(), weight.into());
        }
        Value::Object(map)
    }

    /// `self` with what it leaves unset taken from `base`.
    pub fn or(self, base: TokenStyle) -> TokenStyle {
        TokenStyle {
            color: self.color.or(base.color),
            italic: self.italic.or(base.italic),
            weight: self.weight.or(base.weight),
        }
    }
}

/// The `ui` section's keys, in the order files list them, with what each
/// colors. Only the first three are required.
pub const UI_KEYS: &[(&str, &str)] = &[
    (
        "editor",
        "The editor and preview pane: the main surface. Required.",
    ),
    ("text", "Body text on every surface. Required."),
    (
        "accent",
        "The theme's accent: focus rings, the caret, the Active status. Required.",
    ),
    ("app", "Behind everything: the window's top band."),
    ("sidebar", "The sidebar of notebooks and tags."),
    ("list", "The note list."),
    ("elevated", "Menus, popovers and the command palette."),
    ("fill", "Chips, banners and other quiet fills on the panes."),
    (
        "text_secondary",
        "Text that is read but not primary: previews of notes, details.",
    ),
    (
        "text_muted",
        "Quiet text: times, counts, hints, Markdown markers.",
    ),
    (
        "divider",
        "Lines between panes and rows, and control borders.",
    ),
    ("link", "Link text in the UI."),
    ("row_active", "The selected note in the list."),
    ("row_hover", "A note under the pointer."),
    ("sidebar_active", "The selected sidebar row."),
    ("sidebar_hover", "A sidebar row under the pointer."),
    ("hover", "Menu items and ghost buttons under the pointer."),
    (
        "selected",
        "A selected ghost button, such as the current view mode.",
    ),
    (
        "active_line",
        "The editor line with the cursor, when line numbers show.",
    ),
    ("button", "Primary buttons."),
    ("button_text", "Text on primary buttons."),
    ("success", "Success: the Completed status, synced."),
    ("warning", "Warnings: the On hold status."),
    ("danger", "Errors and destructive actions."),
    (
        "selection",
        "Selected text in the editor, usually translucent (#RRGGBBAA).",
    ),
    ("caret", "The editor's caret."),
];

/// The `syntax` section's keys: Markdown in the editor, then code in fenced
/// blocks (tree-sitter capture names).
pub const SYNTAX_KEYS: &[(&str, &str)] = &[
    ("text", "Plain Markdown text in the editor."),
    ("heading", "Heading text."),
    ("block_marker", "Heading `#` and quote `>` markers."),
    (
        "marker",
        "Inline markers: `*`, backticks, code fences, table pipes, link brackets.",
    ),
    ("list_marker", "List bullets and numbers."),
    ("task", "Task boxes, `[ ]` and `[x]`."),
    ("hr", "Horizontal rules."),
    ("quote", "Quoted text."),
    ("link", "Link text."),
    ("url", "Link destinations and autolinks."),
    ("code", "Inline code."),
    ("code_block", "Code blocks without a language."),
    ("emphasis", "*Emphasis* (italic by default)."),
    ("strong", "**Strong** text (bold by default)."),
    ("front_matter", "YAML front matter between `---` lines."),
    ("html", "HTML blocks."),
    ("escape", "Backslash escapes."),
    ("comment", "Code: comments."),
    ("comment_doc", "Code: documentation comments."),
    ("keyword", "Code: keywords."),
    ("string", "Code: strings."),
    (
        "string_special",
        "Code: special strings, such as regular expressions.",
    ),
    ("function", "Code: function names."),
    ("type", "Code: type names."),
    (
        "type_constructor",
        "Code: constructors and enum variants. (Not `constructor`, which JavaScript tools trip over.)",
    ),
    ("number", "Code: numbers."),
    ("boolean", "Code: true and false."),
    ("constant", "Code: constants."),
    ("variable", "Code: variables."),
    (
        "variable_special",
        "Code: special variables, such as `self`.",
    ),
    ("property", "Code: properties and fields."),
    ("attribute", "Code: attributes and annotations."),
    ("tag", "Code: HTML and XML tags."),
    ("label", "Code: labels."),
    ("operator", "Code: operators."),
    ("punctuation", "Code: punctuation."),
    ("bracket", "Code: brackets."),
    (
        "embedded",
        "Code: embedded code, such as template expressions.",
    ),
];

/// The `preview` section's keys.
pub const PREVIEW_KEYS: &[(&str, &str)] = &[
    ("text", "Body text."),
    ("heading", "Headings."),
    ("link", "Links."),
    ("quote", "Quoted text and the bar beside it."),
    (
        "task_done",
        "Finished tasks, and list bullets and numbers: text that steps back.",
    ),
    ("border", "Table borders and horizontal rules."),
    ("code_background", "Code blocks."),
    ("code_text", "Text in code blocks without a language."),
    ("inline_code_background", "Inline code."),
    ("inline_code_text", "Inline code's text."),
    ("table_header_background", "A table's header row."),
    (
        "selection",
        "Selected text, usually translucent (#RRGGBBAA).",
    ),
];

/// Which syntax key colors each highlight capture the editor and code blocks
/// use. `primary`, `variant`, `enum`, `preproc` and `tag.doctype` are names
/// no bundled code grammar uses, given to Markdown's quotes, rules, task
/// boxes, front matter and HTML (see `markdown_source.rs`).
pub const CAPTURES: &[(&str, &str)] = &[
    ("title", "heading"),
    ("punctuation.special", "block_marker"),
    ("punctuation.list_marker", "list_marker"),
    ("punctuation.delimiter", "marker"),
    ("text.literal", "code_block"),
    ("text.code.span", "code"),
    ("link_text", "link"),
    ("link_uri", "url"),
    ("emphasis", "emphasis"),
    ("emphasis.strong", "strong"),
    ("string.escape", "escape"),
    ("primary", "quote"),
    ("variant", "hr"),
    ("enum", "task"),
    ("preproc", "front_matter"),
    ("tag.doctype", "html"),
    ("comment", "comment"),
    ("comment.doc", "comment_doc"),
    ("keyword", "keyword"),
    ("tag", "tag"),
    ("constructor", "type_constructor"),
    ("string", "string"),
    ("string.special", "string_special"),
    ("string.special.symbol", "string_special"),
    ("string.regex", "string_special"),
    ("number", "number"),
    ("boolean", "boolean"),
    ("constant", "constant"),
    ("variable.special", "variable_special"),
    ("type", "type"),
    ("attribute", "attribute"),
    ("label", "label"),
    ("operator", "operator"),
    ("punctuation", "punctuation"),
    ("punctuation.bracket", "bracket"),
    ("function", "function"),
    ("property", "property"),
    ("variable", "variable"),
    ("embedded", "embedded"),
];

/// A theme as its file gives it: only the colors it names.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeFile {
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub appearance: Appearance,
    pub ui: BTreeMap<&'static str, Color>,
    pub syntax: BTreeMap<&'static str, TokenStyle>,
    pub preview: BTreeMap<&'static str, Color>,
}

/// The metadata keys a file may carry beside its sections.
const TOP_KEYS: &[&str] = &[
    "$schema",
    "schema_version",
    "name",
    "author",
    "license",
    "source",
    "description",
    "appearance",
    "ui",
    "syntax",
    "preview",
];

/// The key in `keys` spelled `key`, as a `'static` name.
fn known(keys: &[(&'static str, &str)], key: &str) -> Option<&'static str> {
    keys.iter().find(|(k, _)| *k == key).map(|(k, _)| *k)
}

fn list(keys: &[(&str, &str)]) -> String {
    keys.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
}

/// What is wrong with a theme file, in a sentence that names the key.
pub type ParseError = String;

impl ThemeFile {
    /// Reads a theme file.
    pub fn parse(text: &str) -> Result<ThemeFile, ParseError> {
        let value: Value =
            serde_json::from_str(text).map_err(|err| format!("isn't valid JSON: {err}"))?;
        ThemeFile::from_value(&value)
    }

    pub fn from_value(value: &Value) -> Result<ThemeFile, ParseError> {
        let Value::Object(top) = value else {
            return Err("should be a JSON object".into());
        };
        for key in top.keys() {
            if !TOP_KEYS.contains(&key.as_str()) {
                return Err(format!(
                    "has an unknown key `{key}`; a theme has {}",
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
        let appearance = match text("appearance", true)?.as_deref() {
            Some("light") => Appearance::Light,
            Some("dark") => Appearance::Dark,
            Some(other) => {
                return Err(format!(
                    "`appearance` is \"{other}\"; it should be \"light\" or \"dark\""
                ));
            }
            None => unreachable!(),
        };
        let ui = colors(top.get("ui"), "ui", UI_KEYS)?;
        for required in ["editor", "text", "accent"] {
            if !ui.contains_key(required) {
                return Err(format!(
                    "needs `ui.{required}`; a theme gives at least ui.editor, ui.text and ui.accent"
                ));
            }
        }
        Ok(ThemeFile {
            name,
            author: text("author", false)?.unwrap_or_default(),
            license: text("license", false)?.unwrap_or_default(),
            source: text("source", false)?,
            description: text("description", false)?,
            appearance,
            ui,
            syntax: tokens(top.get("syntax"))?,
            preview: colors(top.get("preview"), "preview", PREVIEW_KEYS)?,
        })
    }

    /// The file, as JSON with its keys in the documented order.
    pub fn to_value(&self) -> Value {
        let mut top = Map::new();
        top.insert("$schema".into(), SCHEMA_URL.into());
        top.insert("schema_version".into(), SCHEMA_VERSION.into());
        top.insert("name".into(), self.name.clone().into());
        top.insert("author".into(), self.author.clone().into());
        top.insert("license".into(), self.license.clone().into());
        if let Some(source) = &self.source {
            top.insert("source".into(), source.clone().into());
        }
        if let Some(description) = &self.description {
            top.insert("description".into(), description.clone().into());
        }
        top.insert("appearance".into(), self.appearance.as_str().into());
        let section = |keys: &[(&'static str, &str)], get: &dyn Fn(&str) -> Option<Value>| {
            let mut map = Map::new();
            for (key, _) in keys {
                if let Some(value) = get(key) {
                    map.insert((*key).into(), value);
                }
            }
            Value::Object(map)
        };
        top.insert(
            "ui".into(),
            section(UI_KEYS, &|k| self.ui.get(k).map(|c| c.hex().into())),
        );
        top.insert(
            "syntax".into(),
            section(SYNTAX_KEYS, &|k| self.syntax.get(k).map(|s| s.to_json())),
        );
        top.insert(
            "preview".into(),
            section(PREVIEW_KEYS, &|k| {
                self.preview.get(k).map(|c| c.hex().into())
            }),
        );
        Value::Object(top)
    }

    /// The file's text, pretty-printed with a final newline.
    pub fn to_json_string(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_value()).unwrap_or_default();
        text.push('\n');
        text
    }
}

fn section<'a>(
    value: Option<&'a Value>,
    name: &str,
) -> Result<Option<&'a Map<String, Value>>, ParseError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map)),
        Some(_) => Err(format!("`{name}` should be an object of colors")),
    }
}

fn color_at(value: &Value, path: &str) -> Result<Color, ParseError> {
    match value {
        Value::String(s) => Color::parse(s.trim()).ok_or_else(|| {
            format!("`{path}` is \"{s}\", which isn't a color; write #RRGGBB or #RRGGBBAA")
        }),
        _ => Err(format!("`{path}` should be a color such as \"#RRGGBB\"")),
    }
}

fn colors(
    value: Option<&Value>,
    name: &str,
    keys: &[(&'static str, &str)],
) -> Result<BTreeMap<&'static str, Color>, ParseError> {
    let mut out = BTreeMap::new();
    let Some(map) = section(value, name)? else {
        return Ok(out);
    };
    for (key, value) in map {
        let Some(key) = known(keys, key) else {
            return Err(format!(
                "has an unknown key `{name}.{key}`; `{name}` has {}",
                list(keys)
            ));
        };
        out.insert(key, color_at(value, &format!("{name}.{key}"))?);
    }
    Ok(out)
}

fn tokens(value: Option<&Value>) -> Result<BTreeMap<&'static str, TokenStyle>, ParseError> {
    let mut out = BTreeMap::new();
    let Some(map) = section(value, "syntax")? else {
        return Ok(out);
    };
    for (key, value) in map {
        let Some(key) = known(SYNTAX_KEYS, key) else {
            return Err(format!(
                "has an unknown key `syntax.{key}`; `syntax` has {}",
                list(SYNTAX_KEYS)
            ));
        };
        let path = format!("syntax.{key}");
        let style = match value {
            Value::String(_) => TokenStyle::color(color_at(value, &path)?.rgb),
            Value::Object(fields) => {
                let mut style = TokenStyle::default();
                for (field, v) in fields {
                    match (field.as_str(), v) {
                        ("color", v) => {
                            style.color = Some(color_at(v, &format!("{path}.color"))?.rgb)
                        }
                        ("font_style", Value::String(s)) if s == "italic" => {
                            style.italic = Some(true)
                        }
                        ("font_style", Value::String(s)) if s == "normal" => {
                            style.italic = Some(false)
                        }
                        ("font_weight", Value::Number(n))
                            if n.as_u64().is_some_and(|w| (100..=900).contains(&w)) =>
                        {
                            style.weight = n.as_u64().map(|w| w as u16);
                        }
                        ("font_style", _) => {
                            return Err(format!(
                                "`{path}.font_style` should be \"italic\" or \"normal\""
                            ));
                        }
                        ("font_weight", _) => {
                            return Err(format!(
                                "`{path}.font_weight` should be a number from 100 to 900"
                            ));
                        }
                        (other, _) => {
                            return Err(format!(
                                "has an unknown key `{path}.{other}`; a style has color, font_style and font_weight"
                            ));
                        }
                    }
                }
                style
            }
            _ => {
                return Err(format!(
                    "`{path}` should be a color, or an object with color, font_style and font_weight"
                ));
            }
        };
        out.insert(key, style);
    }
    Ok(out)
}

// MARK: Resolving

/// A theme with every color in place.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTheme {
    pub name: String,
    pub dark: bool,
    pub colors: Colors,
    /// Selected text in the editor.
    pub selection: Color,
    pub caret: Rgb,
    /// Every syntax key's style, before it is fitted to a surface.
    pub syntax: BTreeMap<&'static str, TokenStyle>,
    pub preview: PreviewColors,
}

/// The preview's colors. `None` leaves the part in the body text's color, or
/// (for a table's header) in the code background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewColors {
    pub text: Rgb,
    pub heading: Option<Rgb>,
    pub link: Rgb,
    pub quote: Rgb,
    /// Finished tasks and list markers, which step back from the text.
    pub task_done: Rgb,
    pub border: Rgb,
    pub code_background: Rgb,
    pub code_text: Option<Rgb>,
    pub inline_code_background: Rgb,
    pub inline_code_text: Option<Rgb>,
    pub table_header_background: Option<Rgb>,
    /// `None`: the accent at 30%.
    pub selection: Option<Color>,
}

const BLACK: Rgb = Rgb(0x000000);
const WHITE: Rgb = Rgb(0xFFFFFF);

/// `top` over `bottom` at `alpha` (0 to 1), in sRGB.
pub fn mix(top: Rgb, bottom: Rgb, alpha: f64) -> Rgb {
    let alpha = alpha.clamp(0., 1.);
    let channel = |shift: u32| {
        let (t, b) = ((top.0 >> shift) & 0xFF, (bottom.0 >> shift) & 0xFF);
        (f64::from(t) * alpha + f64::from(b) * (1. - alpha)).round() as u32
    };
    Rgb((channel(16) << 16) | (channel(8) << 8) | channel(0))
}

/// WCAG relative luminance, 0 (black) to 1 (white).
pub fn luminance(rgb: Rgb) -> f64 {
    let channel = |shift: u32| {
        let c = f64::from((rgb.0 >> shift) & 0xFF) / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}

/// The WCAG contrast ratio of two colors, 1 to 21.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    ((light + 0.05) / (dark + 0.05)).clamp(1., 21.)
}

/// `color`, moved toward `toward` in small steps until it reads at `min`
/// against every one of `backgrounds`; `toward` itself if nothing short of
/// it does. A color that already reads comes back unchanged.
pub fn ensure_contrast(color: Rgb, backgrounds: &[Rgb], min: f64, toward: Rgb) -> Rgb {
    let reads = |c: Rgb| backgrounds.iter().all(|bg| contrast(c, *bg) >= min);
    (0..=40)
        .map(|step| mix(toward, color, f64::from(step) / 40.))
        .find(|c| reads(*c))
        .unwrap_or(toward)
}

// MARK: Legibility

/// The pane surfaces text is drawn on, by name.
pub fn surfaces(c: &Colors) -> [(&'static str, Rgb); 13] {
    [
        ("app", c.app),
        ("sidebar", c.sidebar),
        ("list", c.list),
        ("editor", c.editor),
        ("elevated", c.elevated),
        ("fill", c.fill),
        ("row_active", c.row_active),
        ("row_hover", c.row_hover),
        ("sidebar_active", c.sidebar_active),
        ("sidebar_hover", c.sidebar_hover),
        ("hover", c.hover),
        ("selected", c.selected),
        ("code", c.code),
    ]
}

/// Every place a theme's colors don't read, in a sentence each: body text
/// at 4.5:1 on every surface, secondary text at 3:1 there, quiet text,
/// links and status colors on the panes, the caret, button text, the
/// selection, selected rows, and every syntax and preview color on what it
/// is drawn on. These are the checks the built-in themes pass.
// no-prop-test: the checks themselves; `prop_made_legible_themes_pass` holds
// every imported theme to them.
pub fn problems(theme: &ThemeFile) -> Vec<String> {
    let t = theme.resolve();
    let c = &t.colors;
    let mut out = Vec::new();
    let mut want = |what: String, ratio: f64, min: f64| {
        if ratio < min {
            out.push(format!(
                "{what} is {ratio:.2}:1; it should be at least {min}:1"
            ));
        }
    };
    for (surface, bg) in surfaces(c) {
        want(format!("text on {surface}"), contrast(c.text, bg), 4.5);
        want(
            format!("text_secondary on {surface}"),
            contrast(c.text_secondary, bg),
            3.0,
        );
    }
    for (surface, bg) in [
        ("sidebar", c.sidebar),
        ("list", c.list),
        ("editor", c.editor),
        ("elevated", c.elevated),
    ] {
        want(
            format!("text_muted on {surface}"),
            contrast(c.text_muted, bg),
            3.0,
        );
        let link_min = if surface == "sidebar" { 3.0 } else { 4.5 };
        want(format!("link on {surface}"), contrast(c.link, bg), link_min);
        for (status, fg) in [
            ("success", c.success),
            ("warning", c.warning),
            ("danger", c.danger),
        ] {
            want(format!("{status} on {surface}"), contrast(fg, bg), 3.0);
        }
    }
    want("caret on editor".into(), contrast(t.caret, c.editor), 3.0);
    want(
        "button_text on button".into(),
        contrast(c.button_text, c.button),
        4.5,
    );
    let selection = t.selection.over(c.editor);
    want(
        "selection on editor".into(),
        contrast(selection, c.editor),
        1.15,
    );
    want("text on selection".into(), contrast(c.text, selection), 4.5);
    want(
        "row_active on list".into(),
        contrast(c.row_active, c.list),
        1.05,
    );
    want(
        "sidebar_active on sidebar".into(),
        contrast(c.sidebar_active, c.sidebar),
        1.05,
    );
    for (key, style) in &t.syntax {
        if let Some(color) = style.color {
            want(
                format!("syntax.{key} on editor"),
                contrast(color, c.editor),
                3.0,
            );
        }
    }
    let p = &t.preview;
    want(
        "preview text on editor".into(),
        contrast(p.text, c.editor),
        4.5,
    );
    want(
        "preview link on editor".into(),
        contrast(p.link, c.editor),
        4.5,
    );
    want(
        "preview quote on editor".into(),
        contrast(p.quote, c.editor),
        4.5,
    );
    if let Some(heading) = p.heading {
        want(
            "preview heading on editor".into(),
            contrast(heading, c.editor),
            3.0,
        );
    }
    want(
        "preview code text on code_background".into(),
        contrast(p.code_text.unwrap_or(p.text), p.code_background),
        4.5,
    );
    want(
        "preview inline code text on inline_code_background".into(),
        contrast(
            p.inline_code_text.unwrap_or(p.text),
            p.inline_code_background,
        ),
        4.5,
    );
    if let Some(head) = p.table_header_background {
        want(
            "preview text on table_header_background".into(),
            contrast(p.text, head),
            4.5,
        );
    }
    want(
        "preview text on the preview's selection".into(),
        contrast(p.text, preview_selection(&t)),
        4.5,
    );
    out
}

/// The preview's selection as drawn over the editor.
pub fn preview_selection(t: &ResolvedTheme) -> Rgb {
    match t.preview.selection {
        Some(s) => s.over(t.colors.editor),
        None => mix(t.colors.accent, t.colors.editor, 0.3),
    }
}

/// The end of the scale away from `background`: white on a dark surface,
/// black on a light one.
fn far_end(background: Rgb) -> Rgb {
    if contrast(WHITE, background) >= contrast(BLACK, background) {
        WHITE
    } else {
        BLACK
    }
}

impl ThemeFile {
    /// Every color, from the file's or derived from its others.
    pub fn resolve(&self) -> ResolvedTheme {
        let dark = self.appearance.is_dark();
        let given = |key: &str| self.ui.get(key).map(|c| c.rgb);
        // The required three are checked when the file is read; a theme
        // built in code without them falls back to Kanso's.
        let fallback = if dark { super::DARK } else { super::LIGHT };
        let editor = given("editor").unwrap_or(fallback.editor);
        let text = given("text").unwrap_or(fallback.text);
        let accent = given("accent").unwrap_or(fallback.accent);
        // A pane set back from the editor: darker in a dark theme, a step
        // toward the text in a light one.
        let recess = |amount: f64| {
            if dark {
                mix(BLACK, editor, amount)
            } else {
                mix(text, editor, amount)
            }
        };
        let tint = |base: Rgb, amount: f64| mix(text, base, amount);
        let app = given("app").unwrap_or_else(|| recess(if dark { 0.40 } else { 0.04 }));
        let sidebar = given("sidebar").unwrap_or_else(|| recess(if dark { 0.28 } else { 0.07 }));
        let list = given("list").unwrap_or_else(|| recess(if dark { 0.12 } else { 0.02 }));
        let elevated = given("elevated").unwrap_or_else(|| {
            if dark {
                tint(editor, 0.05)
            } else {
                mix(WHITE, editor, 0.6)
            }
        });
        let fill =
            given("fill").unwrap_or_else(|| if dark { elevated } else { tint(editor, 0.10) });
        let divider =
            given("divider").unwrap_or_else(|| tint(editor, if dark { 0.09 } else { 0.14 }));
        let hover =
            given("hover").unwrap_or_else(|| if dark { divider } else { tint(editor, 0.055) });
        let selected = given("selected").unwrap_or(if dark { divider } else { fill });
        let row_active = given("row_active").unwrap_or_else(|| tint(list, 0.07));
        let row_hover = given("row_hover").unwrap_or_else(|| tint(list, 0.035));
        let sidebar_active = given("sidebar_active").unwrap_or_else(|| tint(sidebar, 0.08));
        let sidebar_hover = given("sidebar_hover").unwrap_or_else(|| tint(sidebar, 0.04));
        let active_line = given("active_line").unwrap_or_else(|| tint(editor, 0.025));
        let surfaces = [
            app,
            sidebar,
            list,
            editor,
            elevated,
            fill,
            row_active,
            row_hover,
            sidebar_active,
            sidebar_hover,
            hover,
            selected,
        ];
        let text_secondary = given("text_secondary")
            .unwrap_or_else(|| ensure_contrast(mix(editor, text, 0.28), &surfaces, 4.5, text));
        let text_muted = given("text_muted")
            .unwrap_or_else(|| ensure_contrast(mix(editor, text, 0.45), &surfaces, 3.0, text));
        let link = given("link").unwrap_or_else(|| {
            ensure_contrast(accent, &[editor, list, elevated, row_active], 4.5, text)
        });
        let button_text = given("button_text").unwrap_or_else(|| {
            // The pane's own color on the accent when it reads there, as in
            // a light theme; the text color otherwise.
            if contrast(editor, accent) >= contrast(text, accent) {
                editor
            } else {
                text
            }
        });
        let button = given("button")
            .unwrap_or_else(|| ensure_contrast(accent, &[button_text], 4.5, far_end(button_text)));
        let status = |key: &str, dark_default: u32, light_default: u32| {
            given(key).unwrap_or_else(|| {
                let base = Rgb(if dark { dark_default } else { light_default });
                ensure_contrast(base, &[sidebar, list, editor, elevated], 3.0, text)
            })
        };
        let success = status("success", 0x7BAA78, 0x4F7F4C);
        let warning = status("warning", 0xC49A5A, 0x8A6424);
        let danger = status("danger", 0xC96E63, 0xA6493F);
        let selection = self.ui.get("selection").copied().unwrap_or(Color {
            rgb: accent,
            alpha: 0x4D,
        });
        let caret = given("caret").unwrap_or(accent);

        let mut colors = Colors {
            app,
            sidebar,
            list,
            editor,
            elevated,
            fill,
            text,
            text_secondary,
            text_muted,
            divider,
            accent,
            link,
            row_active,
            row_hover,
            sidebar_active,
            sidebar_hover,
            hover,
            selected,
            active_line,
            button,
            button_text,
            // Filled in below, from the syntax and preview sections.
            code: fill,
            code_text: text,
            syntax_keyword: accent,
            syntax_string: success,
            success,
            warning,
            danger,
        };
        let syntax = self.resolve_syntax(&colors);
        let preview = self.resolve_preview(&colors);
        colors.code = preview.code_background;
        colors.code_text = syntax["code"].color.unwrap_or(text);
        colors.syntax_keyword = syntax["keyword"].color.unwrap_or(accent);
        colors.syntax_string = syntax["string"].color.unwrap_or(success);
        ResolvedTheme {
            name: self.name.clone(),
            dark,
            colors,
            selection,
            caret,
            syntax,
            preview,
        }
    }

    fn resolve_syntax(&self, ui: &Colors) -> BTreeMap<&'static str, TokenStyle> {
        let bold = TokenStyle {
            weight: Some(700),
            ..TokenStyle::default()
        };
        let italic = TokenStyle {
            italic: Some(true),
            ..TokenStyle::default()
        };
        let color = TokenStyle::color;
        // Each key's default: a fixed style, or another key's.
        enum Fallback {
            Style(TokenStyle),
            Key(&'static str),
        }
        use Fallback::{Key, Style};
        let default = |key: &str| -> Fallback {
            match key {
                "text" => Style(color(ui.text)),
                "heading" => Style(TokenStyle {
                    color: Some(ui.text),
                    ..bold
                }),
                "marker" => Style(color(ui.text_muted)),
                "block_marker" | "list_marker" | "url" | "escape" => Key("marker"),
                "task" | "hr" => Key("list_marker"),
                "link" => Style(color(ui.link)),
                "code" => Key("constant"),
                "code_block" => Style(color(ui.text_secondary)),
                "emphasis" => Style(italic),
                "strong" => Style(bold),
                "quote" | "front_matter" | "html" => Style(TokenStyle::default()),
                "comment" => Style(TokenStyle {
                    color: Some(ui.text_muted),
                    ..italic
                }),
                "comment_doc" => Key("comment"),
                "keyword" => Style(color(ui.accent)),
                "tag" => Key("keyword"),
                "type_constructor" | "attribute" | "label" => Key("type"),
                "string" => Style(color(ui.success)),
                "string_special" => Key("string"),
                "number" | "boolean" | "variable_special" => Key("constant"),
                "constant" => Style(color(ui.warning)),
                "type" | "operator" | "punctuation" => Style(color(ui.text_secondary)),
                "bracket" => Key("punctuation"),
                "property" => Key("variable"),
                _ => Style(color(ui.text)),
            }
        };
        // Headings, emphasis and strong text keep their weight and slant
        // when a theme gives them only a color.
        let keep = |key: &str| match key {
            "heading" | "strong" => bold,
            "emphasis" => italic,
            _ => TokenStyle::default(),
        };
        fn lookup(
            key: &'static str,
            file: &BTreeMap<&'static str, TokenStyle>,
            default: &dyn Fn(&str) -> Fallback,
            keep: &dyn Fn(&str) -> TokenStyle,
            depth: usize,
        ) -> TokenStyle {
            if let Some(style) = file.get(key) {
                return style.or(keep(key));
            }
            match default(key) {
                Fallback::Style(style) => style,
                Fallback::Key(other) if depth < 8 => {
                    let other = known(SYNTAX_KEYS, other).unwrap_or("text");
                    lookup(other, file, default, keep, depth + 1)
                }
                Fallback::Key(_) => TokenStyle::default(),
            }
        }
        SYNTAX_KEYS
            .iter()
            .map(|(key, _)| (*key, lookup(key, &self.syntax, &default, &keep, 0)))
            .collect()
    }

    fn resolve_preview(&self, ui: &Colors) -> PreviewColors {
        let given = |key: &str| self.preview.get(key).map(|c| c.rgb);
        let code_background = self
            .preview
            .get("code_background")
            .map(|c| c.over(ui.editor))
            .unwrap_or(ui.fill);
        PreviewColors {
            text: given("text").unwrap_or(ui.text),
            heading: given("heading"),
            link: given("link").unwrap_or(ui.link),
            quote: given("quote").unwrap_or(ui.text_secondary),
            // Not the quote's color: a theme that makes quotes stand out
            // would make finished tasks stand out with them.
            task_done: given("task_done").unwrap_or(ui.text_secondary),
            border: self
                .preview
                .get("border")
                .map(|c| c.over(ui.editor))
                .unwrap_or(ui.divider),
            code_background,
            code_text: given("code_text"),
            inline_code_background: self
                .preview
                .get("inline_code_background")
                .map(|c| c.over(ui.editor))
                .unwrap_or(code_background),
            inline_code_text: given("inline_code_text"),
            table_header_background: self
                .preview
                .get("table_header_background")
                .map(|c| c.over(ui.editor)),
            selection: self.preview.get("selection").copied(),
        }
    }
}

/// Syntax styles fitted to the editor they are drawn on: each color shaded
/// toward `text` until it reads at 3:1 on `surface`.
pub fn fit_syntax(
    syntax: &BTreeMap<&'static str, TokenStyle>,
    surface: Rgb,
    text: Rgb,
) -> BTreeMap<&'static str, TokenStyle> {
    syntax
        .iter()
        .map(|(key, style)| {
            let color = style
                .color
                .map(|c| ensure_contrast(c, &[surface], 3.0, text));
            (*key, TokenStyle { color, ..*style })
        })
        .collect()
}

/// Preview colors fitted to the pane they are drawn on: text at 4.5:1 on
/// the pane, links and quotes too, headings and finished tasks at 3:1, and
/// text on code and on
/// a table's header at 4.5:1 on their own fills (a dark code block from a
/// dark preview theme keeps light text on a light page). A theme's own
/// preview on its own panes comes back unchanged when it already reads.
pub fn fit_preview(preview: &PreviewColors, ui: &Colors) -> PreviewColors {
    let surface = ui.editor;
    let text = ensure_contrast(preview.text, &[surface], 4.5, ui.text);
    // Text on a fill: the given color, or the body text, shaded away from
    // the fill when it doesn't read there. `None` while the body text does.
    let on = |fill: Rgb, given: Option<Rgb>| {
        let start = given.unwrap_or(text);
        if contrast(start, fill) >= 4.5 {
            given
        } else {
            Some(ensure_contrast(start, &[fill], 4.5, far_end(fill)))
        }
    };
    let table_header_background = preview
        .table_header_background
        .filter(|head| contrast(text, *head) >= 4.5);
    PreviewColors {
        text,
        heading: preview
            .heading
            .map(|c| ensure_contrast(c, &[surface], 3.0, ui.text)),
        link: ensure_contrast(preview.link, &[surface], 4.5, ui.text),
        quote: ensure_contrast(preview.quote, &[surface], 4.5, ui.text),
        // Stepping back is the point, so 3:1 is enough, as for headings.
        task_done: ensure_contrast(preview.task_done, &[surface], 3.0, ui.text),
        border: preview.border,
        code_background: preview.code_background,
        code_text: on(preview.code_background, preview.code_text),
        inline_code_background: preview.inline_code_background,
        inline_code_text: on(preview.inline_code_background, preview.inline_code_text),
        table_header_background,
        selection: preview.selection,
    }
}

/// The JSON Schema for theme files, generated from the key tables so it
/// can't drift from what Kanso reads. `themes/schema/kanso-theme-v1.json` is
/// this, written out.
pub fn schema() -> Value {
    let color = json!({ "$ref": "#/$defs/color" });
    let props = |keys: &[(&str, &str)], item: &Value| {
        let mut map = Map::new();
        for (key, description) in keys {
            let mut entry = item.clone();
            if let Value::Object(entry) = &mut entry {
                entry.insert("description".into(), (*description).into());
            }
            map.insert((*key).into(), entry);
        }
        Value::Object(map)
    };
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_URL,
        "title": "Kanso theme",
        "description": "One Kanso theme: its appearance, and colors for the UI, the editor's syntax and the preview. Only ui.editor, ui.text and ui.accent are required; Kanso derives the rest.",
        "type": "object",
        "required": ["name", "appearance", "ui"],
        "additionalProperties": false,
        "properties": {
            "$schema": { "type": "string" },
            "schema_version": { "const": SCHEMA_VERSION, "description": "The format version, 1." },
            "name": { "type": "string", "minLength": 1, "description": "The name shown under the theme's card; unique among themes." },
            "author": { "type": "string" },
            "license": { "type": "string", "description": "An SPDX license identifier, such as MIT." },
            "source": { "type": "string", "description": "Where the palette comes from." },
            "description": { "type": "string" },
            "appearance": { "enum": ["light", "dark"] },
            "ui": {
                "type": "object",
                "additionalProperties": false,
                "required": ["editor", "text", "accent"],
                "properties": props(UI_KEYS, &color),
            },
            "syntax": {
                "type": "object",
                "additionalProperties": false,
                "properties": props(SYNTAX_KEYS, &json!({ "$ref": "#/$defs/token" })),
            },
            "preview": {
                "type": "object",
                "additionalProperties": false,
                "properties": props(PREVIEW_KEYS, &color),
            },
        },
        "$defs": {
            "color": {
                "type": "string",
                "pattern": "^#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$",
                "description": "#RRGGBB, or #RRGGBBAA with an alpha.",
            },
            "token": {
                "oneOf": [
                    { "$ref": "#/$defs/color" },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "color": { "$ref": "#/$defs/color" },
                            "font_style": { "enum": ["normal", "italic"] },
                            "font_weight": { "type": "integer", "minimum": 100, "maximum": 900 },
                        },
                    },
                ],
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{
        Appearance, Color, PREVIEW_KEYS, SYNTAX_KEYS, ThemeFile, TokenStyle, UI_KEYS, contrast,
        ensure_contrast, fit_preview, fit_syntax, mix, schema,
    };
    use crate::theme::Rgb;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use std::collections::BTreeMap;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn rgb() -> impl Strategy<Value = Rgb> {
        (0u32..=0xFF_FFFF).prop_map(Rgb)
    }

    fn color() -> impl Strategy<Value = Color> {
        (rgb(), prop_oneof![Just(0xFFu8), any::<u8>()])
            .prop_map(|(rgb, alpha)| Color { rgb, alpha })
    }

    fn token() -> impl Strategy<Value = TokenStyle> {
        (
            prop::option::of(rgb()),
            prop::option::of(any::<bool>()),
            prop::option::of(prop::sample::select(vec![100u16, 400, 600, 700, 900])),
        )
            .prop_map(|(color, italic, weight)| TokenStyle {
                color,
                italic,
                weight,
            })
    }

    fn section<T: std::fmt::Debug + Clone + 'static>(
        keys: &'static [(&'static str, &'static str)],
        value: impl Strategy<Value = T>,
    ) -> impl Strategy<Value = BTreeMap<&'static str, T>> {
        prop::collection::vec(
            (
                prop::sample::select(keys.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                value,
            ),
            0..12,
        )
        .prop_map(|pairs| pairs.into_iter().collect())
    }

    /// Any theme file: a light or dark theme with its three required colors
    /// and any others.
    fn theme() -> impl Strategy<Value = ThemeFile> {
        (
            "[A-Za-z][A-Za-z0-9 ]{0,15}",
            any::<bool>(),
            (rgb(), rgb(), rgb()),
            section(UI_KEYS, color()),
            section(SYNTAX_KEYS, token()),
            section(PREVIEW_KEYS, color()),
            prop::option::of("https://[a-z]{1,10}\\.dev"),
        )
            .prop_map(
                |(name, dark, (editor, text, accent), mut ui, syntax, preview, source)| {
                    ui.insert(
                        "editor",
                        Color {
                            rgb: editor,
                            alpha: 0xFF,
                        },
                    );
                    ui.insert(
                        "text",
                        Color {
                            rgb: text,
                            alpha: 0xFF,
                        },
                    );
                    ui.insert(
                        "accent",
                        Color {
                            rgb: accent,
                            alpha: 0xFF,
                        },
                    );
                    // An empty style is written as `{}`, which reads back the same.
                    ThemeFile {
                        name: name.trim().to_owned(),
                        author: "Someone".into(),
                        license: "MIT".into(),
                        source,
                        description: None,
                        appearance: if dark {
                            Appearance::Dark
                        } else {
                            Appearance::Light
                        },
                        ui,
                        syntax,
                        preview,
                    }
                },
            )
            .prop_filter("a name", |t| !t.name.is_empty())
    }

    /// A readable minimal theme: text at 7:1 or more on its editor.
    fn readable() -> impl Strategy<Value = ThemeFile> {
        (rgb(), rgb(), rgb())
            .prop_filter("text reads", |(editor, text, _)| {
                contrast(*editor, *text) >= 7.0
            })
            .prop_map(|(editor, text, accent)| {
                let dark = super::luminance(text) > super::luminance(editor);
                let mut ui = BTreeMap::new();
                ui.insert(
                    "editor",
                    Color {
                        rgb: editor,
                        alpha: 0xFF,
                    },
                );
                ui.insert(
                    "text",
                    Color {
                        rgb: text,
                        alpha: 0xFF,
                    },
                );
                ui.insert(
                    "accent",
                    Color {
                        rgb: accent,
                        alpha: 0xFF,
                    },
                );
                ThemeFile {
                    name: "Generated".into(),
                    author: String::new(),
                    license: String::new(),
                    source: None,
                    description: None,
                    appearance: if dark {
                        Appearance::Dark
                    } else {
                        Appearance::Light
                    },
                    ui,
                    syntax: BTreeMap::new(),
                    preview: BTreeMap::new(),
                }
            })
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: a color reads back from what it writes.
        #[test]
        fn prop_colors_round_trip(c in color()) {
            prop_assert_eq!(Color::parse(&c.hex()), Some(c));
            prop_assert_eq!(Color::parse(&c.hex().to_lowercase()), Some(c));
        }

        /// Round trip: a theme reads back from the file it writes, and the
        /// file is written the same way twice.
        #[test]
        fn prop_theme_files_round_trip(t in theme()) {
            let text = t.to_json_string();
            let back = ThemeFile::parse(&text).map_err(TestCaseError::fail)?;
            prop_assert_eq!(&back, &t);
            prop_assert_eq!(back.to_json_string(), text);
        }

        /// A key nobody knows is an error that names it, wherever it is.
        #[test]
        fn prop_unknown_keys_are_named(t in theme(), key in "[a-z]{3,10}_x", where_ in 0usize..4) {
            let mut value = t.to_value();
            let path = match where_ {
                0 => { value[&key] = "x".into(); key.clone() }
                1 => { value["ui"][&key] = "#000000".into(); format!("ui.{key}") }
                2 => { value["syntax"][&key] = "#000000".into(); format!("syntax.{key}") }
                _ => { value["preview"][&key] = "#000000".into(); format!("preview.{key}") }
            };
            let err = ThemeFile::from_value(&value).unwrap_err();
            prop_assert!(err.contains(&format!("`{path}`")), "{}", err);
        }

        /// Shading toward a color: the result reads at the asked ratio on
        /// every background whenever the color it moves toward does, and a
        /// color that already reads is left alone.
        #[test]
        fn prop_ensure_contrast_reads_or_stays(c in rgb(), bgs in prop::collection::vec(rgb(), 1..4), toward in rgb(), min in 1.0f64..7.0) {
            let out = ensure_contrast(c, &bgs, min, toward);
            let reads = |x: Rgb| bgs.iter().all(|bg| contrast(x, *bg) >= min);
            if reads(c) {
                prop_assert_eq!(out, c);
            } else if reads(toward) {
                prop_assert!(reads(out), "{:?}", out);
            }
        }

        /// Mixing stays between its ends, channel by channel.
        #[test]
        fn prop_mix_stays_between(a in rgb(), b in rgb(), t in 0.0f64..=1.0) {
            let m = mix(a, b, t);
            for shift in [16, 8, 0] {
                let (x, y, z) = ((a.0 >> shift) & 0xFF, (b.0 >> shift) & 0xFF, (m.0 >> shift) & 0xFF);
                prop_assert!(x.min(y) <= z && z <= x.max(y));
            }
            prop_assert_eq!(mix(a, b, 1.0), a);
            prop_assert_eq!(mix(a, b, 0.0), b);
        }

        /// A theme of three readable colors comes out whole and legible: the
        /// derived secondary, quiet and link colors read on every surface as
        /// well as the text itself allows, and every syntax and preview
        /// color reads on the editor once fitted.
        #[test]
        fn prop_derived_colors_read(t in readable()) {
            let r = t.resolve();
            let c = &r.colors;
            let surfaces = [c.app, c.sidebar, c.list, c.editor, c.elevated, c.fill, c.row_active,
                c.row_hover, c.sidebar_active, c.sidebar_hover, c.hover, c.selected];
            let worst = |fg: Rgb| surfaces.iter().map(|bg| contrast(fg, *bg)).fold(21., f64::min);
            let text = worst(c.text);
            prop_assert!(worst(c.text_secondary) >= 4.5f64.min(text) - 0.01, "{:?}", c);
            prop_assert!(worst(c.text_muted) >= 3.0f64.min(text) - 0.01, "{:?}", c);
            prop_assert!(contrast(c.button_text, c.button) >= 4.5 - 0.01);
            let fitted = fit_syntax(&r.syntax, c.editor, c.text);
            for (key, style) in &fitted {
                if let Some(color) = style.color {
                    prop_assert!(contrast(color, c.editor) >= 3.0, "{} {:?}", key, color);
                }
            }
            let p = fit_preview(&r.preview, c);
            prop_assert!(contrast(p.text, c.editor) >= 4.5);
            prop_assert!(contrast(p.link, c.editor) >= 4.5);
        }

        /// Syntax colors fitted to any surface read on it at 3:1 when the
        /// text does, keep their weight and slant, and are left alone when
        /// they already read.
        #[test]
        fn prop_fitting_reads_and_keeps_style(styles in section(SYNTAX_KEYS, token()), surface in rgb(), text in rgb()) {
            let fitted = fit_syntax(&styles, surface, text);
            for (key, style) in &styles {
                let out = fitted[key];
                prop_assert_eq!((out.italic, out.weight), (style.italic, style.weight));
                if let (Some(before), Some(after)) = (style.color, out.color) {
                    if contrast(before, surface) >= 3.0 {
                        prop_assert_eq!(before, after);
                    } else if contrast(text, surface) >= 3.0 {
                        prop_assert!(contrast(after, surface) >= 3.0);
                    }
                }
            }
        }
    }

    #[test]
    fn colors_parse() {
        assert_eq!(Color::parse("#7E96B3"), Some(Color::opaque(0x7E96B3)));
        assert_eq!(
            Color::parse("#7e96b34d"),
            Some(Color {
                rgb: Rgb(0x7E96B3),
                alpha: 0x4D
            })
        );
        for bad in [
            "7E96B3",
            "#7E96B",
            "#GGGGGG",
            "#7E96B34D0",
            "",
            "#",
            "#+12345",
        ] {
            assert_eq!(Color::parse(bad), None, "{bad}");
        }
        assert_eq!(
            Color {
                rgb: Rgb(0xFFFFFF),
                alpha: 0x80
            }
            .over(Rgb(0)),
            Rgb(0x808080)
        );
    }

    /// The smallest theme there is, and what is wrong with broken ones, in
    /// words that name the problem.
    #[test]
    fn minimal_and_broken_files() {
        let minimal = r##"{"name": "Tiny", "appearance": "light",
            "ui": {"editor": "#FFFFFF", "text": "#222222", "accent": "#3366CC"}}"##;
        let theme = ThemeFile::parse(minimal).unwrap();
        let r = theme.resolve();
        assert!(!r.dark);
        assert_eq!(r.colors.editor, Rgb(0xFFFFFF));
        assert!(contrast(r.colors.link, r.colors.editor) >= 4.5);
        // Headings keep their weight when a theme colors them.
        let colored = minimal.replace(r#""ui""#, r##""syntax": {"heading": "#AA0000"}, "ui""##);
        let heading = ThemeFile::parse(&colored).unwrap().resolve().syntax["heading"];
        assert_eq!(
            (heading.color, heading.weight),
            (Some(Rgb(0xAA0000)), Some(700))
        );
        let cases = [
            ("{", "isn't valid JSON"),
            ("[]", "should be a JSON object"),
            (r##"{"appearance": "dark"}"##, "needs a `name`"),
            (
                r##"{"name": "X", "appearance": "dim"}"##,
                "\"light\" or \"dark\"",
            ),
            (
                r##"{"name": "X", "appearance": "dark", "ui": {"editor": "#000000"}}"##,
                "needs `ui.text`",
            ),
            (
                r##"{"name": "X", "appearance": "dark", "ui": {"editor": "black"}}"##,
                "`ui.editor` is \"black\"",
            ),
            (
                r##"{"name": "X", "appearance": "dark", "schema_version": 2}"##,
                "schema_version 2",
            ),
            (
                r##"{"name": "X", "appearance": "dark", "ui": {"editor": "#000000", "text": "#FFFFFF", "accent": "#FF0000"}, "syntax": {"keyword": {"font_style": "oblique"}}}"##,
                "`syntax.keyword.font_style`",
            ),
            (
                r##"{"name": "X", "appearance": "dark", "colors": {}}"##,
                "unknown key `colors`",
            ),
        ];
        for (text, expected) in cases {
            let err = ThemeFile::parse(text).unwrap_err();
            assert!(err.contains(expected), "{text}: {err}");
        }
    }

    /// The crate's folder: cargo's at run time (the phone runners set it to where
    /// they copied the fixtures), else where it was built.
    fn manifest_dir() -> std::path::PathBuf {
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(Into::into)
            .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
    }

    /// The schema in the repository is the one the key tables make.
    #[test]
    fn schema_file_is_current() {
        let path = manifest_dir().join("../../themes/schema/kanso-theme-v1.json");
        let mut expected = serde_json::to_string_pretty(&schema()).unwrap();
        expected.push('\n');
        if std::env::var_os("KANSO_THEME_FMT").is_some() {
            std::fs::write(&path, &expected).unwrap();
        }
        let written = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            written == expected,
            "themes/schema/kanso-theme-v1.json is out of date (commit it in the kanso-themes submodule); run KANSO_THEME_FMT=1 cargo test -p kanso-engine schema_file_is_current"
        );
    }
}
