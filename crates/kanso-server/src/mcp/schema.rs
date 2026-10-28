//! Kanso's MCP tool definitions and argument
//! validation equivalent to `@inkdropapp/mcp-server` 1.5.0.
//!
//! `tools.json` is the Node server's `tools/list` answer, verbatim, so names,
//! descriptions and input schemas match it exactly. Validation follows zod's
//! rules for the subset of JSON Schema those definitions use and reports
//! issues in zod's words, in schema order: unknown arguments are dropped and
//! defaults filled in, as zod's parse does.
//!
//! Kanso's own tools, named `kanso-…`, are defined in `kanso_tools.json` in
//! the same form and listed after the original 17, which stay untouched.

use std::fmt::Write as _;
use std::sync::OnceLock;

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde_json::{Map, Value};

const TOOLS_JSON: &str = include_str!("tools.json");
/// Kanso's own tools, named `kanso-…`, listed after the 17.
const KANSO_TOOLS_JSON: &str = include_str!("kanso_tools.json");

pub(crate) struct Tool {
    pub name: String,
    /// Input properties in definition order, which is zod's issue order.
    properties: Vec<(String, Value)>,
    required: Vec<String>,
}

struct Catalog {
    definitions: Value,
    tools: Vec<Tool>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut definitions: Value =
            serde_json::from_str(TOOLS_JSON).expect("tools.json is valid JSON");
        let kanso: Value =
            serde_json::from_str(KANSO_TOOLS_JSON).expect("kanso_tools.json is valid JSON");
        if let (Some(all), Some(kanso)) = (definitions.as_array_mut(), kanso.as_array()) {
            all.extend(kanso.iter().cloned());
        }
        let mut raw: Vec<RawTool> =
            serde_json::from_str(TOOLS_JSON).expect("tools.json lists tools");
        raw.extend(
            serde_json::from_str::<Vec<RawTool>>(KANSO_TOOLS_JSON)
                .expect("kanso_tools.json lists tools"),
        );
        let tools = raw
            .into_iter()
            .map(|tool| Tool {
                name: tool.name,
                properties: tool.input_schema.properties.0,
                required: tool.input_schema.required,
            })
            .collect();
        Catalog { definitions, tools }
    })
}

/// The `tools` array of a `tools/list` answer.
pub(crate) fn definitions() -> &'static Value {
    &catalog().definitions
}

pub(crate) fn tool(name: &str) -> Option<&'static Tool> {
    catalog().tools.iter().find(|tool| tool.name == name)
}

#[derive(Deserialize)]
struct RawTool {
    name: String,
    #[serde(rename = "inputSchema")]
    input_schema: RawSchema,
}

#[derive(Deserialize)]
struct RawSchema {
    #[serde(default)]
    properties: Ordered,
    #[serde(default)]
    required: Vec<String>,
}

/// A JSON object's entries in document order, whatever `serde_json::Map` keeps.
#[derive(Default)]
struct Ordered(Vec<(String, Value)>);

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedVisitor;
        impl<'de> Visitor<'de> for OrderedVisitor {
            type Value = Ordered;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Value>()? {
                    entries.push(entry);
                }
                Ok(Ordered(entries))
            }
        }
        deserializer.deserialize_map(OrderedVisitor)
    }
}

enum Segment {
    Key(String),
    Index(usize),
}

struct Issue {
    path: String,
    message: String,
}

fn issue(path: &[Segment], message: String) -> Issue {
    Issue {
        path: dot_path(path),
        message,
    }
}

/// zod's `getDotPath`: `tags[0]`, `a.b`.
fn dot_path(path: &[Segment]) -> String {
    let mut out = String::new();
    for segment in path {
        match segment {
            Segment::Index(i) => {
                let _ = write!(out, "[{i}]");
            }
            Segment::Key(key) if is_identifier(key) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(key);
            }
            Segment::Key(key) => {
                let _ = write!(out, "[{}]", Value::from(key.as_str()));
            }
        }
    }
    out
}

fn is_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

impl Tool {
    /// The arguments as the tool sees them after zod's parse, or zod's issue
    /// list (one issue per line, `<message> at <path>`).
    pub fn validate(&self, arguments: Option<&Value>) -> Result<Map<String, Value>, String> {
        let empty = Map::new();
        let args = match arguments {
            None | Some(Value::Null) => &empty,
            Some(Value::Object(args)) => args,
            Some(other) => {
                return Err(format!(
                    "Invalid input: expected object, received {}",
                    received(other)
                ));
            }
        };
        let mut parsed = Map::new();
        let mut issues = Vec::new();
        for (key, schema) in &self.properties {
            let mut path = vec![Segment::Key(key.clone())];
            match args.get(key) {
                Some(value) => {
                    check(schema, value, &mut path, &mut issues);
                    parsed.insert(key.clone(), value.clone());
                }
                None => {
                    if let Some(default) = schema.get("default") {
                        parsed.insert(key.clone(), default.clone());
                    } else if self.required.contains(key) {
                        issues.push(issue(
                            &path,
                            format!(
                                "Invalid input: expected {}, received undefined",
                                expected_type(schema)
                            ),
                        ));
                    }
                }
            }
        }
        if issues.is_empty() {
            return Ok(parsed);
        }
        Err(issues
            .iter()
            .map(|issue| {
                if issue.path.is_empty() {
                    issue.message.clone()
                } else {
                    format!("{} at {}", issue.message, issue.path)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

fn expected_type(schema: &Value) -> &str {
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        return ty;
    }
    schema
        .get("anyOf")
        .and_then(Value::as_array)
        .and_then(|options| options.first())
        .map_or("unknown", expected_type)
}

fn received(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn has_type(value: &Value, ty: &str) -> bool {
    match ty {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn check(schema: &Value, value: &Value, path: &mut Vec<Segment>, issues: &mut Vec<Issue>) {
    if let Some(options) = schema.get("anyOf").and_then(Value::as_array) {
        let accepted = options.iter().any(|option| {
            let mut probe = Vec::new();
            check(option, value, path, &mut probe);
            probe.is_empty()
        });
        if !accepted {
            // zod reports a `.nullable()` failure against the non-null schema.
            let primary = options
                .iter()
                .find(|o| o.get("type").and_then(Value::as_str) != Some("null"))
                .or(options.first());
            if let Some(primary) = primary {
                check(primary, value, path, issues);
            }
        }
        return;
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array) {
        if !options.contains(value) {
            let choices: Vec<String> = options.iter().map(Value::to_string).collect();
            issues.push(issue(
                path,
                format!("Invalid option: expected one of {}", choices.join("|")),
            ));
        }
        return;
    }
    if let Some(ty) = schema.get("type").and_then(Value::as_str)
        && !has_type(value, ty)
    {
        issues.push(issue(
            path,
            format!("Invalid input: expected {ty}, received {}", received(value)),
        ));
        return;
    }
    match value {
        Value::String(s) => check_string(schema, s, path, issues),
        Value::Array(items) => {
            if let Some(item_schema) = schema.get("items") {
                for (i, item) in items.iter().enumerate() {
                    path.push(Segment::Index(i));
                    check(item_schema, item, path, issues);
                    path.pop();
                }
            }
        }
        _ => {}
    }
}

fn check_string(schema: &Value, s: &str, path: &[Segment], issues: &mut Vec<Issue>) {
    // zod measures strings in UTF-16 code units, like JavaScript.
    let len = s.encode_utf16().count() as u64;
    if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
        && len < min
    {
        issues.push(issue(
            path,
            format!("Too small: expected string to have >={min} characters"),
        ));
    }
    if let Some(max) = schema.get("maxLength").and_then(Value::as_u64)
        && len > max
    {
        issues.push(issue(
            path,
            format!("Too big: expected string to have <={max} characters"),
        ));
    }
    let Some(pattern) = schema.get("pattern").and_then(Value::as_str) else {
        return;
    };
    if schema.get("format").and_then(Value::as_str) == Some("starts_with") {
        let prefix = pattern.strip_prefix('^').unwrap_or(pattern);
        let prefix = prefix.strip_suffix(".*").unwrap_or(prefix);
        if !s.starts_with(prefix) {
            issues.push(issue(
                path,
                format!("Invalid string: must start with {}", Value::from(prefix)),
            ));
        }
    } else if !regex_lite::Regex::new(pattern).is_ok_and(|re| re.is_match(s)) {
        issues.push(issue(
            path,
            format!("Invalid string: must match pattern /{pattern}/"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn validate(name: &str, args: Value) -> Result<Map<String, Value>, String> {
        tool(name).unwrap().validate(Some(&args))
    }

    #[test]
    fn kanso_tools_follow_the_inkdrop_tools_with_their_prefix() {
        let names: Vec<&str> = definitions()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        // Kanso's own, sorted by name.
        assert_eq!(
            &names[18..],
            [
                "kanso-attach-file",
                "kanso-create-daily-note",
                "kanso-create-from-template",
                "kanso-export-html",
                "kanso-get-outline",
                "kanso-get-properties",
                "kanso-list-attachments",
                "kanso-list-backlinks",
                "kanso-list-note-links",
                "kanso-list-tags",
                "kanso-list-tasks",
                "kanso-list-versions",
                "kanso-move-note",
                "kanso-read-attachment",
                "kanso-read-version",
                "kanso-related-notes",
                "kanso-resolve-wikilink",
                "kanso-run-query-view",
                "kanso-search-by-meaning",
                "kanso-set-task-done"
            ]
        );
        assert!(validate("kanso-list-tasks", json!({})).is_ok());
        assert!(validate("kanso-list-tasks", json!({"status": "later"})).is_err());
        assert!(validate("kanso-list-tasks", json!({"dueBefore": "soon"})).is_err());
        assert!(validate("kanso-list-tasks", json!({"dueAfter": "2026-10-25"})).is_ok());
        assert!(validate("kanso-create-daily-note", json!({})).is_ok());
        assert!(
            validate(
                "kanso-create-from-template",
                json!({"templateId": "note:t"})
            )
            .is_err()
        );
        assert!(
            validate(
                "kanso-related-notes",
                json!({"noteId": "note:a", "limit": 3})
            )
            .is_ok()
        );
        assert!(
            validate(
                "kanso-related-notes",
                json!({"noteId": "note:a", "limit": "3"})
            )
            .is_err()
        );
        assert!(validate("kanso-get-outline", json!({})).is_err());
        assert!(validate("kanso-resolve-wikilink", json!({"link": "[[A]]"})).is_ok());
        assert!(validate("kanso-search-by-meaning", json!({"query": "fruit"})).is_ok());
        assert!(validate("kanso-search-by-meaning", json!({"limit": 3})).is_err());
    }

    #[test]
    fn the_catalog_has_the_inkdrop_tools_in_order() {
        let names: Vec<&str> = definitions()
            .as_array()
            .unwrap()
            .iter()
            .take(18)
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names[..18],
            [
                "read-note",
                "read-backlinks",
                "search-notes",
                "list-notes",
                "list-templates",
                "create-note",
                "update-note",
                "patch-note",
                "list-tags",
                "read-tag",
                "create-tag",
                "update-tag",
                "list-notebooks",
                "read-book",
                "create-book",
                "update-book",
                "create-file",
                "read-file",
            ]
        );
        let order: Vec<&str> = tool("list-notes")
            .unwrap()
            .properties
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(
            order,
            ["bookId", "tagIds", "keyword", "sort", "descending", "limit"]
        );
    }

    // Expected messages were captured from @inkdropapp/mcp-server 1.5.0.
    #[test]
    fn issues_read_like_zod() {
        assert_eq!(
            validate("create-note", json!({})).unwrap_err(),
            "Invalid input: expected string, received undefined at bookId\n\
             Invalid input: expected string, received undefined at title\n\
             Invalid input: expected string, received undefined at body"
        );
        assert_eq!(
            validate(
                "create-note",
                json!({"bookId": 5, "title": "x".repeat(200), "body": "b"})
            )
            .unwrap_err(),
            "Invalid input: expected string, received number at bookId\n\
             Too big: expected string to have <=128 characters at title"
        );
        assert_eq!(
            validate(
                "create-note",
                json!({"bookId": "book:x", "title": "t", "body": "b", "status": "later", "tags": ["x", 3]})
            )
            .unwrap_err(),
            "Invalid option: expected one of \"none\"|\"active\"|\"onHold\"|\"completed\"|\"dropped\" at status\n\
             Invalid string: must start with \"tag:\" at tags[0]\n\
             Invalid input: expected string, received number at tags[1]"
        );
        assert_eq!(
            validate("update-book", json!({"_id": "bk", "parentBookId": "x"})).unwrap_err(),
            "Too small: expected string to have >=6 characters at _id\n\
             Invalid string: must match pattern /^book:/ at _id\n\
             Too small: expected string to have >=6 characters at parentBookId\n\
             Invalid string: must match pattern /^book:/ at parentBookId"
        );
        assert_eq!(
            validate(
                "list-notes",
                json!({"limit": "ten", "descending": "yes", "tagIds": "tag:x"})
            )
            .unwrap_err(),
            "Invalid input: expected array, received string at tagIds\n\
             Invalid input: expected boolean, received string at descending\n\
             Invalid input: expected number, received string at limit"
        );
        assert_eq!(
            validate("read-note", json!({"noteId": null})).unwrap_err(),
            "Invalid input: expected string, received null at noteId"
        );
        assert_eq!(
            validate(
                "patch-note",
                json!({"_id": "note:abcd", "_rev": "1-a", "old_string": "", "new_string": 1})
            )
            .unwrap_err(),
            "Too small: expected string to have >=1 characters at old_string\n\
             Invalid input: expected string, received number at new_string"
        );
        assert_eq!(
            validate(
                "create-note",
                json!({"bookId": "x", "title": "a", "body": "b"})
            )
            .unwrap_err(),
            "Too small: expected string to have >=5 characters at bookId\n\
             Invalid string: must match pattern /^(book:|trash$|template$)/ at bookId"
        );
    }

    #[test]
    fn parsing_drops_unknown_keys_and_fills_defaults() {
        let parsed = validate("create-tag", json!({"name": "x", "extra": 1})).unwrap();
        assert_eq!(
            Value::Object(parsed),
            json!({"name": "x", "color": "default"})
        );
        let parsed = validate("list-notes", json!({})).unwrap();
        assert_eq!(
            Value::Object(parsed),
            json!({"tagIds": [], "sort": "updatedAt", "descending": true, "limit": 100})
        );
        assert!(
            validate(
                "update-book",
                json!({"_id": "book:abc", "parentBookId": null})
            )
            .is_ok()
        );
        assert!(
            validate(
                "create-note",
                json!({"bookId": "trash", "title": "", "body": ""})
            )
            .is_ok()
        );
        assert!(
            validate("list-tags", json!({"anything": 1}))
                .unwrap()
                .is_empty()
        );
        assert!(tool("read-note").unwrap().validate(None).is_err());
    }
}
