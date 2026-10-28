//! Zed theme families: one JSON file with an author and a list of themes,
//! each a `style` of UI colors, `players` (the caret and selection) and
//! `syntax` styles keyed by tree-sitter capture name (schema:
//! https://zed.dev/schema/themes/v0.2.0.json).
//!
//! Each theme in the family becomes one Kanso theme. Zed's colors are
//! `#rrggbbaa` and often translucent; each is painted over the surface it
//! sits on, except the selection, which Kanso also draws translucent.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::color;
use crate::theme::Rgb;
use crate::theme::file::{Appearance, CAPTURES, Color, ThemeFile, TokenStyle, contrast, luminance};

/// Whether `value` is a Zed theme family.
pub fn is_family(value: &Value) -> bool {
    value.get("themes").is_some_and(Value::is_array)
}

/// Every theme in the family.
// no-prop-test: covered by `prop_every_theme_in_a_family_imports` through
// `family`, which this only checks the shape for.
pub fn import(value: &Value) -> Result<Vec<ThemeFile>, String> {
    let Some(themes) = value.get("themes").and_then(Value::as_array) else {
        return Err("This file isn't a Zed theme family: it has no \"themes\" list.".into());
    };
    if themes.is_empty() {
        return Err("This Zed theme family has no themes in it.".into());
    }
    let family_text = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let author = family_text("author").unwrap_or_else(|| "unknown".into());
    let license = family_text("license").unwrap_or_else(|| "See source".into());
    let source = family_text("source").or_else(|| family_text("url"));
    let family_name = family_text("name");
    themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let Some(theme) = theme.as_object() else {
                return Err(format!(
                    "Theme {} in this Zed family isn't an object.",
                    i + 1
                ));
            };
            let name = theme
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .or_else(|| family_name.as_ref().map(|f| format!("{f} {}", i + 1)))
                .unwrap_or_else(|| format!("Zed Theme {}", i + 1));
            let empty = Map::new();
            let style = theme
                .get("style")
                .and_then(Value::as_object)
                .unwrap_or(&empty);
            Ok(convert(
                name,
                author.clone(),
                license.clone(),
                source.clone(),
                theme.get("appearance").and_then(Value::as_str),
                style,
            ))
        })
        .collect()
}

/// One Zed theme as a Kanso theme.
fn convert(
    name: String,
    author: String,
    license: String,
    source: Option<String>,
    appearance: Option<&str>,
    style: &Map<String, Value>,
) -> ThemeFile {
    let get = |key: &str| {
        style
            .get(key)
            .and_then(Value::as_str)
            .and_then(color::parse)
    };
    let first = |keys: &[&str]| keys.iter().find_map(|k| get(k));
    let player = style
        .get("players")
        .and_then(Value::as_array)
        .and_then(|p| p.first())
        .and_then(Value::as_object);
    let player_color = |key: &str| {
        player
            .and_then(|p| p.get(key))
            .and_then(Value::as_str)
            .and_then(color::parse)
    };
    let appearance = match appearance {
        Some("dark") => Appearance::Dark,
        Some("light") => Appearance::Light,
        _ => {
            // As the colors say: light text on a dark editor is dark.
            let grey = Rgb(0x808080);
            let bg = first(&["editor.background", "background"]).map(|c| c.over(grey));
            let text = first(&["text", "editor.foreground"]);
            match (bg, text) {
                (Some(bg), Some(text)) if luminance(text.over(bg)) > luminance(bg) => {
                    Appearance::Dark
                }
                (Some(bg), None) if luminance(bg) < 0.18 => Appearance::Dark,
                _ => Appearance::Light,
            }
        }
    };
    let dark = appearance.is_dark();
    let backdrop = Rgb(if dark { 0x1E1E1E } else { 0xF5F5F5 });
    let app = first(&["background", "title_bar.background"]).map(|c| c.over(backdrop));
    let base = app.unwrap_or(backdrop);
    let editor = first(&["editor.background", "background"])
        .map(|c| c.over(base))
        .unwrap_or(base);
    let sidebar = get("panel.background").map(|c| c.over(base));
    let list = first(&["surface.background", "panel.background"]).map(|c| c.over(base));
    let elevated = get("elevated_surface.background").map(|c| c.over(base));
    let text = first(&["text", "editor.foreground"])
        .map(|c| c.over(editor))
        .unwrap_or(if dark { Rgb(0xDDDDDD) } else { Rgb(0x222222) });
    let accent = get("text.accent")
        .or_else(|| player_color("cursor"))
        .or_else(|| get("link_text.hover"))
        .map(|c| c.over(editor))
        .unwrap_or(Rgb(if dark { 0x60A5FA } else { 0x2563EB }));

    let mut ui = BTreeMap::new();
    let mut put = |key: &'static str, rgb: Option<Rgb>| {
        if let Some(rgb) = rgb {
            ui.insert(key, Color::opaque(rgb.0));
        }
    };
    let on =
        |keys: &[&str], under: Option<Rgb>| first(keys).map(|c| c.over(under.unwrap_or(editor)));
    put("editor", Some(editor));
    put("text", Some(text));
    put("accent", Some(accent));
    put("app", app);
    put("sidebar", sidebar);
    put("list", list);
    put("elevated", elevated);
    put("fill", on(&["element.background"], Some(editor)));
    put("divider", on(&["border.variant", "border"], Some(editor)));
    put("text_secondary", on(&["text.muted"], list));
    put("text_muted", on(&["text.placeholder", "text.muted"], list));
    put("link", on(&["link_text.hover"], Some(editor)));
    // Highlights Kanso draws body text on, when it reads there.
    let highlight = |rgb: Option<Rgb>| rgb.filter(|bg| contrast(text, *bg) >= 4.5);
    put("row_active", highlight(on(&["element.selected"], list)));
    put("row_hover", highlight(on(&["element.hover"], list)));
    put(
        "sidebar_active",
        highlight(on(&["ghost_element.selected"], sidebar)),
    );
    put(
        "sidebar_hover",
        highlight(on(&["ghost_element.hover"], sidebar)),
    );
    put("hover", highlight(on(&["ghost_element.hover"], elevated)));
    put(
        "selected",
        highlight(on(&["ghost_element.selected"], Some(editor))),
    );
    put(
        "active_line",
        on(&["editor.active_line.background"], Some(editor)),
    );
    put("success", on(&["success"], Some(editor)));
    put("warning", on(&["warning"], Some(editor)));
    put("danger", on(&["error"], Some(editor)));
    put("caret", player_color("cursor").map(|c| c.over(editor)));
    if let Some(selection) = player_color("selection").filter(|c| c.alpha > 0) {
        ui.insert("selection", selection);
    }

    let syntax = style
        .get("syntax")
        .and_then(Value::as_object)
        .map(|captures| syntax(captures, editor))
        .unwrap_or_default();
    ThemeFile {
        name,
        author,
        license,
        source,
        description: None,
        appearance,
        ui,
        syntax,
        preview: BTreeMap::new(),
    }
}

/// Captures Kanso gives Markdown, which Zed themes color as code: an enum
/// color isn't a task box's.
const MARKDOWN_ONLY: &[&str] = &["primary", "variant", "enum", "tag.doctype"];

/// Captures Zed themes use that `CAPTURES` doesn't name.
const EXTRA: &[(&str, &str)] = &[
    ("text.literal", "code"),
    ("string.regex", "string_special"),
    ("string.special.symbol", "string_special"),
    ("variable.parameter", "variable"),
    ("punctuation.markup", "marker"),
    ("link_text.markup", "link"),
    ("link_uri.markup", "url"),
    ("title.markup", "heading"),
    ("emphasis.markup", "emphasis"),
    ("emphasis.strong.markup", "strong"),
];

/// The Kanso syntax keys `capture` colors, and how specifically: an exact
/// name beats a parent (`keyword` for `keyword.control`), and a nearer
/// parent beats a farther one.
fn keys_for(capture: &str) -> Vec<(&'static str, usize)> {
    let mut out = Vec::new();
    let mut name = capture;
    let mut distance = 0;
    loop {
        let exact = CAPTURES
            .iter()
            .filter(|(c, _)| *c == name && !MARKDOWN_ONLY.contains(c))
            .chain(EXTRA.iter().filter(|(c, _)| *c == name));
        for (_, key) in exact {
            out.push((*key, distance));
        }
        if !out.is_empty() {
            return out;
        }
        match name.rsplit_once('.') {
            Some((parent, _)) => {
                name = parent;
                distance += 1;
            }
            None => return out,
        }
    }
}

fn syntax(captures: &Map<String, Value>, editor: Rgb) -> BTreeMap<&'static str, TokenStyle> {
    let mut best: BTreeMap<&'static str, (usize, TokenStyle)> = BTreeMap::new();
    for (capture, value) in captures {
        let Some(style) = token(value, editor) else {
            continue;
        };
        for (key, distance) in keys_for(capture) {
            if best.get(key).is_none_or(|(d, _)| distance < *d) {
                best.insert(key, (distance, style));
            }
        }
    }
    best.into_iter().map(|(k, (_, s))| (k, s)).collect()
}

/// A Zed style (`{ color, font_style, font_weight }`, or a bare color) as
/// a Kanso one, its color painted over the editor.
fn token(value: &Value, editor: Rgb) -> Option<TokenStyle> {
    let style = match value {
        Value::String(s) => TokenStyle::color(color::parse(s)?.over(editor)),
        Value::Object(o) => TokenStyle {
            color: o
                .get("color")
                .and_then(Value::as_str)
                .and_then(color::parse)
                .map(|c| c.over(editor)),
            italic: match o.get("font_style").and_then(Value::as_str) {
                Some("italic" | "oblique") => Some(true),
                Some("normal") => Some(false),
                _ => None,
            },
            weight: o
                .get("font_weight")
                .and_then(Value::as_f64)
                .filter(|w| w.is_finite())
                .map(|w| ((w / 100.).round() * 100.).clamp(100., 900.) as u16),
        },
        _ => return None,
    };
    (!style.is_empty()).then_some(style)
}

/// JSON with comments and trailing commas, as Zed's own files allow, made
/// strict.
pub fn strip_jsonc(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_string = false;
    let mut kept = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        let skip_to = match (b, bytes.get(i + 1)) {
            (b'"', _) => {
                in_string = true;
                None
            }
            (b'/', Some(b'/')) => Some(text[i..].find('\n').map_or(bytes.len(), |e| i + e)),
            (b'/', Some(b'*')) => Some(
                text[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |e| i + 2 + e + 2),
            ),
            (b',', _) => {
                // A comma before a closing bracket, past spaces and comments.
                let rest = strip_leading(&text[i + 1..]);
                rest.starts_with(['}', ']']).then_some(i + 1)
            }
            _ => None,
        };
        if let Some(end) = skip_to {
            out.push_str(&text[kept..i]);
            i = end.min(bytes.len());
            kept = i;
            continue;
        }
        i += 1;
    }
    out.push_str(&text[kept.min(text.len())..]);
    out
}

/// `text` past its leading spaces and comments.
fn strip_leading(mut text: &str) -> &str {
    loop {
        let trimmed = text.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//") {
            text = rest.find('\n').map_or("", |e| &rest[e..]);
        } else if let Some(rest) = trimmed.strip_prefix("/*") {
            text = rest.find("*/").map_or("", |e| &rest[e + 2..]);
        } else {
            return trimmed;
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{import, keys_for, strip_jsonc};
    use crate::theme::file::{Appearance, ThemeFile};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 128,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn hex() -> impl Strategy<Value = String> {
        (0u32..=0xFF_FFFF, prop::option::of(0u8..=255)).prop_map(|(rgb, alpha)| match alpha {
            Some(a) => format!("#{rgb:06x}{a:02x}"),
            None => format!("#{rgb:06x}"),
        })
    }

    /// A Zed theme with any colors for the keys Kanso reads, some missing.
    fn zed_theme() -> impl Strategy<Value = Value> {
        let keys = [
            "background",
            "editor.background",
            "editor.foreground",
            "text",
            "text.muted",
            "text.placeholder",
            "text.accent",
            "border",
            "border.variant",
            "element.background",
            "element.hover",
            "element.selected",
            "ghost_element.hover",
            "ghost_element.selected",
            "elevated_surface.background",
            "surface.background",
            "panel.background",
            "editor.active_line.background",
            "success",
            "warning",
            "error",
        ];
        (
            "[A-Za-z][A-Za-z0-9 ]{0,20}",
            prop::option::of(prop::sample::select(vec!["light", "dark"])),
            prop::collection::vec(prop::option::of(hex()), keys.len()),
            prop::collection::vec(
                (
                    prop::sample::select(vec![
                        "keyword",
                        "string",
                        "comment",
                        "title",
                        "emphasis.strong",
                        "keyword.control",
                        "punctuation.list_marker",
                        "enum",
                    ]),
                    hex(),
                    any::<bool>(),
                ),
                0..8,
            ),
            hex(),
            hex(),
        )
            .prop_map(
                move |(name, appearance, colors, syntax, cursor, selection)| {
                    let mut style = serde_json::Map::new();
                    for (key, color) in keys.iter().zip(colors) {
                        if let Some(color) = color {
                            style.insert((*key).into(), color.into());
                        }
                    }
                    let mut captures = serde_json::Map::new();
                    for (capture, color, italic) in syntax {
                        let style = if italic {
                            json!({ "color": color, "font_style": "italic", "font_weight": 700 })
                        } else {
                            json!({ "color": color })
                        };
                        captures.insert(capture.into(), style);
                    }
                    style.insert("syntax".into(), Value::Object(captures));
                    style.insert(
                        "players".into(),
                        json!([{ "cursor": cursor, "selection": selection }]),
                    );
                    let mut theme = serde_json::Map::new();
                    theme.insert("name".into(), name.into());
                    if let Some(a) = appearance {
                        theme.insert("appearance".into(), a.into());
                    }
                    theme.insert("style".into(), Value::Object(style));
                    Value::Object(theme)
                },
            )
    }

    proptest! {
        #![proptest_config(config())]

        /// Completeness: a family of N themes gives N Kanso themes, in
        /// order, each of which writes a file that reads back as itself.
        #[test]
        fn prop_every_theme_in_a_family_imports(themes in prop::collection::vec(zed_theme(), 1..5)) {
            let family = json!({ "name": "Family", "author": "Someone", "themes": themes.clone() });
            let imported = import(&family).unwrap();
            prop_assert_eq!(imported.len(), themes.len());
            for (theme, zed) in imported.iter().zip(&themes) {
                prop_assert_eq!(theme.name.as_str(), zed["name"].as_str().unwrap().trim());
                if let Some(a) = zed.get("appearance").and_then(Value::as_str) {
                    prop_assert_eq!(theme.appearance.as_str(), a);
                }
                let text = theme.to_json_string();
                let back = ThemeFile::parse(&text);
                prop_assert_eq!(back.as_ref(), Ok(theme), "{}", text);
                prop_assert_eq!(theme.author.as_str(), "Someone");
                prop_assert_eq!(theme.license.as_str(), "See source");
            }
        }

        /// Comments and trailing commas don't change what a strict JSON
        /// document says.
        #[test]
        fn prop_jsonc_strips_to_the_same_value(keys in prop::collection::vec("[a-z/*]{1,6}", 0..6)) {
            let strict: serde_json::Map<String, Value> = keys.iter().map(|k| (k.clone(), json!([k, "a//b", "/*x*/"]))).collect();
            let strict = Value::Object(strict);
            let mut loose = String::from("{ // a comment\n");
            for (k, v) in strict.as_object().unwrap() {
                loose.push_str(&format!("/* before */ {}: {}, // after\n", json!(k), v));
            }
            loose.push('}');
            let back: Value = serde_json::from_str(&strip_jsonc(&loose)).unwrap();
            prop_assert_eq!(back, strict);
        }
    }

    #[test]
    fn capture_names_map_to_keys() {
        assert_eq!(keys_for("keyword"), vec![("keyword", 0)]);
        assert_eq!(keys_for("keyword.control.flow"), vec![("keyword", 2)]);
        assert_eq!(keys_for("title"), vec![("heading", 0)]);
        assert_eq!(
            keys_for("text.literal"),
            vec![("code_block", 0), ("code", 0)]
        );
        assert!(
            keys_for("enum").is_empty(),
            "an enum's color isn't a task box's"
        );
        assert!(keys_for("nonsense").is_empty());
    }

    /// A small family in Zed's own shape: translucent colors painted over
    /// their surfaces, the selection kept translucent, and a more specific
    /// capture losing to the exact one.
    #[test]
    fn a_small_family() {
        let family = json!({
            "$schema": "https://zed.dev/schema/themes/v0.2.0.json",
            "name": "Sample",
            "author": "Zed Industries",
            "themes": [
                {
                    "name": "Sample Dark",
                    "appearance": "dark",
                    "style": {
                        "background": "#1e1e2eff",
                        "editor.background": "#181825ff",
                        "text": "#cdd6f4ff",
                        "text.muted": "#a6adc8ff",
                        "text.accent": "#89b4faff",
                        "border.variant": "#ffffff1a",
                        "panel.background": "#11111bff",
                        "element.selected": "#ffffff26",
                        "players": [{ "cursor": "#f5e0dcff", "selection": "#89b4fa40" }],
                        "error": "#f38ba8ff",
                        "syntax": {
                            "keyword": { "color": "#cba6f7ff", "font_style": null, "font_weight": null },
                            "keyword.control": { "color": "#ff0000ff" },
                            "comment": { "color": "#9399b2ff", "font_style": "italic" },
                            "title": { "color": "#f38ba8ff", "font_weight": 800 }
                        }
                    }
                },
                { "name": "Sample Light", "appearance": "light", "style": { "editor.background": "#eff1f5", "text": "#4c4f69" } }
            ]
        });
        let themes = import(&family).unwrap();
        assert_eq!(themes.len(), 2);
        let dark = &themes[0];
        assert_eq!(dark.appearance, Appearance::Dark);
        assert_eq!(dark.ui["editor"].hex(), "#181825");
        assert_eq!(dark.ui["accent"].hex(), "#89B4FA");
        assert_eq!(dark.ui["caret"].hex(), "#F5E0DC");
        assert_eq!(dark.ui["selection"].hex(), "#89B4FA40");
        assert_eq!(
            dark.ui["divider"].hex(),
            "#30303B",
            "white at 10% over the editor"
        );
        assert_eq!(
            dark.ui["row_active"].hex(),
            "#34343D",
            "white at 15% over the panel"
        );
        assert_eq!(dark.syntax["keyword"].color.unwrap().hex(), "#CBA6F7");
        assert_eq!(dark.syntax["comment"].italic, Some(true));
        assert_eq!(dark.syntax["heading"].weight, Some(800));
        assert_eq!(dark.license, "See source");
        let light = &themes[1];
        assert_eq!(light.appearance, Appearance::Light);
        assert_eq!(light.ui["text"].hex(), "#4C4F69");
        assert!(import(&json!({ "themes": [] })).is_err());
    }
}
