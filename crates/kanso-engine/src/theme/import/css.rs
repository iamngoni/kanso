//! A forgiving reader for the CSS and LESS in theme packages.
//!
//! It doesn't build a full stylesheet model, only what the importer needs:
//! every declaration with the full selector it applies to (LESS nesting
//! flattened, `&` replaced), LESS `@variables`, and `@import`s. Blocks under
//! `@media`, `@supports` and `@layer` count as if they weren't nested;
//! `@font-face` and `@keyframes` are skipped. Nothing here fails: text it
//! can't make sense of is passed over, so a theme with one odd rule still
//! imports.
//!
//! Then it resolves values the way a browser would for colors: LESS
//! variables, `var(--x, fallback)` (to any depth, stopping at cycles) and
//! `light-dark(light, dark)`.

use std::collections::HashMap;

use super::color::{matching_paren, split_commas};

/// One `property: value` and the selector of the rule it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    /// The rule's full selector list, comma-separated, nesting flattened.
    pub selector: String,
    /// Lowercased, except for custom properties (`--name`), whose case
    /// matters.
    pub property: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sheet {
    /// In document order.
    pub declarations: Vec<Declaration>,
    /// LESS `@name: value;`, in document order.
    pub less_variables: Vec<(String, String)>,
    /// What each `@import` names, as written.
    pub imports: Vec<String>,
}

/// Reads a stylesheet; never fails.
pub fn parse(text: &str) -> Sheet {
    let text = strip_comments(text);
    let src = text.as_str();
    let bytes = src.as_bytes();
    let mut sheet = Sheet::default();
    // Each open block's selectors; `None` inside a block that is skipped.
    let mut stack: Vec<Option<Vec<String>>> = vec![Some(Vec::new())];
    let mut pos = 0;
    while pos < bytes.len() {
        let (end, terminator) = scan(src, pos);
        let prelude = src[pos..end].trim();
        let context = stack.last().cloned().flatten();
        match terminator {
            Some(b'{') => {
                let inner = context.and_then(|parents| block_context(prelude, &parents));
                stack.push(inner);
            }
            Some(b';') => statement(prelude, context.as_deref(), &mut sheet),
            _ => {
                statement(prelude, context.as_deref(), &mut sheet);
                if stack.len() > 1 {
                    stack.pop();
                }
            }
        }
        pos = end + 1;
    }
    sheet
}

/// The next top-level `{`, `;` or `}` from `pos`, outside strings,
/// parentheses and LESS's `@{name}`; `(len, None)` at the end.
fn scan(src: &str, pos: usize) -> (usize, Option<u8>) {
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut i = pos;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 1;
            } else if b == q {
                quote = None;
            }
        } else {
            match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' => depth += 1,
                b')' => depth = depth.saturating_sub(1),
                b'{' if i > 0 && bytes[i - 1] == b'@' => {
                    // `@{name}` interpolation: not a block.
                    while i < bytes.len() && bytes[i] != b'}' {
                        i += 1;
                    }
                }
                b'{' | b';' | b'}' if depth == 0 => return (i, Some(b)),
                _ => {}
            }
        }
        i += 1;
    }
    (bytes.len(), None)
}

/// `text` without `/* … */` comments and, as LESS writes them, `// …`
/// comments outside strings and parentheses (so `url(http://…)` stays).
fn strip_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut quote: Option<u8> = None;
    let mut depth = 0usize;
    let mut i = 0;
    let mut kept = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 1;
            } else if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        let next = bytes.get(i + 1).copied();
        match (b, next) {
            (b'"' | b'\'', _) => quote = Some(b),
            (b'(', _) => depth += 1,
            (b')', _) => depth = depth.saturating_sub(1),
            (b'/', Some(b'*')) => {
                out.push_str(&text[kept..i]);
                let end = text[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |e| i + 2 + e + 2);
                out.push(' ');
                i = end;
                kept = end;
                continue;
            }
            (b'/', Some(b'/')) if depth == 0 && (i == 0 || bytes[i - 1] != b':') => {
                out.push_str(&text[kept..i]);
                let end = text[i..].find('\n').map_or(bytes.len(), |e| i + e);
                i = end;
                kept = end;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push_str(&text[kept.min(text.len())..]);
    out
}

/// The selectors inside a block opened by `prelude` within `parents`;
/// `None` for a block whose declarations don't style anything (`@font-face`,
/// `@keyframes`).
fn block_context(prelude: &str, parents: &[String]) -> Option<Vec<String>> {
    if let Some(rule) = prelude.strip_prefix('@') {
        let name: String = rule
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        return match name.to_ascii_lowercase().as_str() {
            "media" | "supports" | "layer" | "container" | "document" | "scope"
            | "starting-style" => Some(parents.to_vec()),
            _ => None,
        };
    }
    // A LESS guard (`.a when (@mode = dark)`) doesn't change the selector.
    let prelude = prelude.split(" when ").next().unwrap_or(prelude);
    let children: Vec<&str> = split_commas(prelude)
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    if children.is_empty() {
        return Some(parents.to_vec());
    }
    let parents: Vec<&str> = if parents.is_empty() {
        vec![""]
    } else {
        parents.iter().map(String::as_str).collect()
    };
    let mut out = Vec::new();
    for parent in &parents {
        for child in &children {
            let combined = if child.contains('&') {
                child.replace('&', parent)
            } else if parent.is_empty() {
                (*child).to_owned()
            } else {
                format!("{parent} {child}")
            };
            out.push(combined.split_whitespace().collect::<Vec<_>>().join(" "));
        }
    }
    Some(out)
}

/// A `;`-ended statement: a declaration, a LESS variable or an `@import`.
fn statement(text: &str, context: Option<&[String]>, sheet: &mut Sheet) {
    let Some(selectors) = context else {
        return;
    };
    if text.is_empty() {
        return;
    }
    if let Some(rest) = text.strip_prefix("@import") {
        if let Some(path) = import_path(rest) {
            sheet.imports.push(path);
        }
        return;
    }
    let Some((name, value)) = text.split_once(':') else {
        return;
    };
    let (name, value) = (name.trim(), value.trim());
    if let Some(variable) = name.strip_prefix('@') {
        if !variable.is_empty()
            && variable
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            sheet
                .less_variables
                .push((variable.to_owned(), value.to_owned()));
        }
        return;
    }
    if name.is_empty() || name.contains(|c: char| c.is_whitespace() || c == '&') {
        return;
    }
    let property = if name.starts_with("--") {
        name.to_owned()
    } else {
        name.to_ascii_lowercase()
    };
    sheet.declarations.push(Declaration {
        selector: selectors.join(", "),
        property,
        value: value.to_owned(),
    });
}

/// What `@import (options) 'path';` or `@import url(path) media;` names.
fn import_path(rest: &str) -> Option<String> {
    let mut rest = rest.trim();
    // LESS import options: `@import (reference) 'x';`
    if rest.starts_with('(') {
        let close = matching_paren(rest, 0)?;
        rest = rest[close + 1..].trim();
    }
    if let Some(inner) = rest.strip_prefix("url(") {
        rest = inner.split(')').next()?.trim();
    }
    let quoted = rest
        .strip_prefix(['"', '\''])
        .and_then(|r| r.split(['"', '\'']).next());
    let path = quoted.unwrap_or_else(|| rest.split_whitespace().next().unwrap_or(""));
    (!path.is_empty()).then(|| path.to_owned())
}

/// How many substitutions one value may take, so a theme whose variables
/// each name several others can't make resolving take forever.
const BUDGET: usize = 10_000;

/// `value` with every `var(--name, fallback)` replaced by what `vars` gives
/// `--name` (itself resolved), or else by its fallback; `None` when a
/// variable has neither, or when variables form a cycle with no fallback.
pub fn resolve_vars(value: &str, vars: &HashMap<String, String>) -> Option<String> {
    let mut budget = BUDGET;
    substitute(value, vars, &mut Vec::new(), &mut budget)
}

fn substitute(
    value: &str,
    vars: &HashMap<String, String>,
    visiting: &mut Vec<String>,
    budget: &mut usize,
) -> Option<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(at) = find_call(rest, "var(") {
        *budget = budget.checked_sub(1)?;
        out.push_str(&rest[..at]);
        let open = at + 3;
        let close = matching_paren(rest, open)?;
        let inner = &rest[open + 1..close];
        let (name, fallback) = match inner.split_once(',') {
            Some((name, fallback)) => (name.trim(), Some(fallback.trim())),
            None => (inner.trim(), None),
        };
        let from_var = match vars.get(name) {
            Some(found) if !visiting.iter().any(|v| v == name) => {
                visiting.push(name.to_owned());
                let resolved = substitute(found, vars, visiting, budget);
                visiting.pop();
                resolved
            }
            _ => None,
        };
        let resolved = match from_var {
            Some(v) => v,
            None => substitute(fallback?, vars, visiting, budget)?,
        };
        out.push_str(&resolved);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// Where `name` (such as `var(`) starts as a call in `text`, not as the end
/// of a longer name.
fn find_call(text: &str, name: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let at = from + found;
        let before = lower[..at].bytes().next_back();
        if !before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
            return Some(at);
        }
        from = at + name.len();
    }
    None
}

/// `value` with each `light-dark(light, dark)` replaced by the side for
/// the theme's appearance.
pub fn resolve_light_dark(value: &str, dark: bool) -> String {
    let mut out = String::new();
    let mut rest = value.to_owned();
    while let Some(at) = find_call(&rest, "light-dark(") {
        let open = at + "light-dark".len();
        let Some(close) = matching_paren(&rest, open) else {
            break;
        };
        let parts = split_commas(&rest[open + 1..close]);
        let chosen = match parts.as_slice() {
            [light, dark_side] => if dark { *dark_side } else { *light }.to_owned(),
            _ => String::new(),
        };
        out.push_str(&rest[..at]);
        rest = format!("{chosen}{}", &rest[close + 1..]);
    }
    out.push_str(&rest);
    out
}

/// `value` with each LESS `@name` replaced by its value from `vars` (itself
/// resolved); a name `vars` lacks, or one in a cycle, is left as written.
pub fn resolve_less(value: &str, vars: &HashMap<String, String>) -> String {
    let mut budget = BUDGET;
    less(value, vars, &mut Vec::new(), &mut budget)
}

fn less(
    value: &str,
    vars: &HashMap<String, String>,
    visiting: &mut Vec<String>,
    budget: &mut usize,
) -> String {
    if !value.contains('@') {
        return value.to_owned();
    }
    let bytes = value.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    let mut kept = 0;
    while i < bytes.len() {
        let starts = bytes[i] == b'@'
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
            && bytes
                .get(i + 1)
                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_' || *b == b'-');
        if !starts {
            i += 1;
            continue;
        }
        let end = (i + 1..bytes.len())
            .find(|&j| !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_'))
            .unwrap_or(bytes.len());
        let name = &value[i + 1..end];
        if let Some(found) = vars.get(name)
            && !visiting.iter().any(|v| v == name)
            && *budget > 0
        {
            *budget -= 1;
            visiting.push(name.to_owned());
            let resolved = less(found, vars, visiting, budget);
            visiting.pop();
            out.push_str(&value[kept..i]);
            out.push_str(&resolved);
            kept = end;
        }
        i = end;
    }
    out.push_str(&value[kept..]);
    out
}

/// Every custom property's value, the later declaration winning.
pub fn custom_properties<'a>(
    declarations: impl IntoIterator<Item = &'a Declaration>,
) -> HashMap<String, String> {
    declarations
        .into_iter()
        .filter(|d| d.property.starts_with("--"))
        .map(|d| (d.property.clone(), d.value.clone()))
        .collect()
}

// MARK: Selectors

/// The last compound of a selector (`.a .b > .c.d:hover` gives
/// `.c.d:hover`).
pub fn last_compound(selector: &str) -> &str {
    let mut depth = 0usize;
    let mut start = 0;
    for (i, b) in selector.bytes().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b' ' | b'>' | b'+' | b'~' if depth == 0 => start = i + 1,
            _ => {}
        }
    }
    selector[start..].trim()
}

/// A compound's parts: its element name, its classes, and whether it has a
/// pseudo-class or pseudo-element (a state, such as `:hover`, or a part
/// such as `::before`), which a theme's plain colors don't come from.
pub struct Compound<'a> {
    pub element: &'a str,
    pub classes: Vec<&'a str>,
    pub pseudo: bool,
}

pub fn compound(text: &str) -> Compound<'_> {
    let element_end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '*'))
        .unwrap_or(text.len());
    let mut classes = Vec::new();
    let mut pseudo = false;
    let bytes = text.as_bytes();
    let mut i = element_end;
    let mut depth = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b':' if depth == 0 => {
                // `:not(…)` narrows; it doesn't change the state.
                if !text[i..].starts_with(":not(") {
                    pseudo = true;
                }
            }
            b'.' if depth == 0 => {
                let end = (i + 1..bytes.len())
                    .find(|&j| {
                        !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_')
                    })
                    .unwrap_or(bytes.len());
                classes.push(&text[i + 1..end]);
                i = end;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    Compound {
        element: &text[..element_end],
        classes,
        pseudo,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{compound, last_compound, parse, resolve_less, resolve_light_dark, resolve_vars};
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

        /// Chains: a variable defined through a chain of `var()`s of any
        /// length resolves to the value at its end.
        #[test]
        fn prop_var_chains_resolve(len in 1usize..60, end in "#[0-9a-f]{6}") {
            let mut vars = HashMap::new();
            for i in 0..len {
                vars.insert(format!("--v{i}"), format!("var(--v{})", i + 1));
            }
            vars.insert(format!("--v{len}"), end.clone());
            prop_assert_eq!(resolve_vars("var(--v0)", &vars), Some(end.clone()));
            prop_assert_eq!(resolve_vars("1px solid var(--v0)", &vars), Some(format!("1px solid {end}")));
        }

        /// Termination: variables that refer to each other in a cycle, of any
        /// length, resolve to nothing, or to the fallback when one is given,
        /// and never loop.
        #[test]
        fn prop_var_cycles_terminate(len in 1usize..40, fallback in "#[0-9a-f]{6}") {
            let mut vars = HashMap::new();
            for i in 0..len {
                vars.insert(format!("--c{i}"), format!("var(--c{})", (i + 1) % len));
            }
            prop_assert_eq!(resolve_vars("var(--c0)", &vars), None);
            prop_assert_eq!(resolve_vars(&format!("var(--c0, {fallback})"), &vars), Some(fallback.clone()));
            // LESS variables in a cycle are left as written.
            let less_vars: HashMap<String, String> =
                (0..len).map(|i| (format!("c{i}"), format!("@c{}", (i + 1) % len))).collect();
            prop_assert!(resolve_less("@c0", &less_vars).starts_with('@'));
        }

        /// Exponential fan-out: variables that each use the next one twice
        /// still resolve (or give up) quickly.
        #[test]
        fn prop_var_fan_out_is_bounded(len in 1usize..40) {
            let mut vars = HashMap::new();
            for i in 0..len {
                vars.insert(format!("--f{i}"), format!("var(--f{n}) var(--f{n})", n = i + 1));
            }
            vars.insert(format!("--f{len}"), "x".into());
            let got = resolve_vars("var(--f0)", &vars);
            if let Some(got) = got {
                prop_assert_eq!(got.split(' ').count(), 1 << len);
            }
        }

        /// Flattening: a declaration nested in LESS blocks of any depth gets
        /// the selectors of every block around it, in order.
        #[test]
        fn prop_nesting_flattens(names in prop::collection::vec("[a-z]{1,8}", 1..8), value in "#[0-9a-f]{6}") {
            let mut text = String::new();
            for name in &names {
                text.push_str(&format!(".{name} {{ "));
            }
            text.push_str(&format!("color: {value};"));
            text.push_str(&" }".repeat(names.len()));
            let sheet = parse(&text);
            prop_assert_eq!(sheet.declarations.len(), 1);
            let want = names.iter().map(|n| format!(".{n}")).collect::<Vec<_>>().join(" ");
            prop_assert_eq!(&sheet.declarations[0].selector, &want);
            prop_assert_eq!(&sheet.declarations[0].value, &value);
        }

        /// Totality: any text parses without panicking, unbalanced braces
        /// and all.
        #[test]
        fn prop_any_text_parses(text in "[a-z{}();:@&.,/*'\" -]{0,80}") {
            let _ = parse(&text);
        }
    }

    #[test]
    fn a_night_owl_style_sheet() {
        let sheet = parse(
            r#"
            /* tokens */
            :root { --color-owl-bg: #001628; --color-owl-border: #45647b; }
            @layer theme.ui {
              :root, body.dark { --editor-background: var(--color-owl-bg);
                --note-search-bar-input-border: 1px solid var(--color-owl-border) }
            }
            @media (prefers-color-scheme: dark) { .theme-x { --text-color: light-dark(#111, #eee); } }
            @font-face { font-family: X; src: url(http://example.com/x.woff); }
            ::-webkit-scrollbar-thumb { background-color: transparent; }
            "#,
        );
        let vars = super::custom_properties(&sheet.declarations);
        assert_eq!(
            resolve_vars("var(--editor-background)", &vars).as_deref(),
            Some("#001628")
        );
        assert_eq!(
            resolve_vars("var(--note-search-bar-input-border)", &vars).as_deref(),
            Some("1px solid #45647b")
        );
        assert_eq!(resolve_light_dark(&vars["--text-color"], true), "#eee");
        assert_eq!(resolve_light_dark(&vars["--text-color"], false), "#111");
        assert!(
            sheet.declarations.iter().all(|d| d.property != "src"),
            "font-face is skipped"
        );
        let thumb = sheet.declarations.last().unwrap();
        assert_eq!(thumb.selector, "::-webkit-scrollbar-thumb");
        // An unknown variable with a fallback takes it.
        assert_eq!(
            resolve_vars("var(--nope, var(--color-owl-bg))", &vars).as_deref(),
            Some("#001628")
        );
        assert_eq!(resolve_vars("hsl(var(--hsl-blue-500))", &vars), None);
    }

    #[test]
    fn a_less_sheet_with_nesting_and_comments() {
        let sheet = parse(
            r#"
            @import './colors';
            @base-yellow: #facf4e;
            @accent: @base-yellow;
            .editor {
              // .cm-variable { color: red; }
              .CodeMirror {
                color: @base06;
                .cm-header { color: @accent; }
                &.cm-s-x .cm-em, .cm-strong { font-style: italic }
              }
            }
            "#,
        );
        assert_eq!(sheet.imports, vec!["./colors".to_owned()]);
        let vars: HashMap<String, String> = sheet.less_variables.iter().cloned().collect();
        let header = sheet
            .declarations
            .iter()
            .find(|d| d.selector.ends_with(".cm-header"))
            .unwrap();
        assert_eq!(header.selector, ".editor .CodeMirror .cm-header");
        assert_eq!(resolve_less(&header.value, &vars), "#facf4e");
        assert!(
            sheet
                .declarations
                .iter()
                .all(|d| !d.selector.contains("cm-variable"))
        );
        let italic = sheet
            .declarations
            .iter()
            .find(|d| d.property == "font-style")
            .unwrap();
        assert_eq!(
            italic.selector,
            ".editor .CodeMirror.cm-s-x .cm-em, .editor .CodeMirror .cm-strong"
        );
        // A variable it doesn't know stays as written.
        assert_eq!(resolve_less("@base06", &vars), "@base06");
        assert_eq!(
            resolve_less("fade(@accent, 13%)", &vars),
            "fade(#facf4e, 13%)"
        );
    }

    #[test]
    fn compounds() {
        assert_eq!(
            last_compound(".editor .CodeMirror > .cm-keyword.cm-link"),
            ".cm-keyword.cm-link"
        );
        let c = compound(".cm-keyword.cm-link");
        assert_eq!(c.classes, vec!["cm-keyword", "cm-link"]);
        assert!(!c.pseudo);
        assert!(compound(".cm-whitespace::before").pseudo);
        assert!(!compound(".cm-inline-code:not(.cm-formatting-code)").pseudo);
        assert_eq!(compound("h1").element, "h1");
        assert_eq!(compound("pre.x").classes, vec!["x"]);
    }
}
