//! Query views: a `query` fenced block in a note, and the notes it lists.
//!
//! ```text
//! from: book:Clients status:active has:due
//! columns: client, rating, due
//! sort: rating desc
//! limit: 50
//! ```
//!
//! `from:` is the app's search language ([`Query`]), with `has:<key>` for a
//! note that has that property. `columns:` names properties and the built-in
//! `updated`, `created`, `book` and `tags`; the title column is always first.
//! Parsing and evaluation are pure: the app reads the notes on a background
//! thread and passes them in.
//!
//! Locked and trashed notes are never listed, nor is the note a query is in.

use std::cmp::Ordering;
use std::fmt;

use crate::properties::{self, Value};
use crate::query::{NoteFacts, Query};

/// A column of the table after the title.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Column {
    Title,
    Updated,
    Created,
    Book,
    Tags,
    Property(String),
}

impl Column {
    pub fn parse(name: &str) -> Column {
        match name.to_ascii_lowercase().as_str() {
            "title" => Column::Title,
            "updated" => Column::Updated,
            "created" => Column::Created,
            "book" => Column::Book,
            "tags" => Column::Tags,
            _ => Column::Property(name.to_owned()),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Column::Title => "Title".into(),
            Column::Updated => "Updated".into(),
            Column::Created => "Created".into(),
            Column::Book => "Book".into(),
            Column::Tags => "Tags".into(),
            Column::Property(key) => key.clone(),
        }
    }
}

/// A `has:` clause: the note has (or hasn't) a property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Has {
    pub key: String,
    pub negated: bool,
}

/// How the rows are ordered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortSpec {
    pub column: Column,
    pub descending: bool,
}

/// A parsed query block.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewSpec {
    pub from: Query,
    pub has: Vec<Has>,
    /// The columns after the title.
    pub columns: Vec<Column>,
    pub sort: Option<SortSpec>,
    pub limit: Option<usize>,
}

impl ViewSpec {
    /// Every column, the title first.
    pub fn labels(&self) -> Vec<String> {
        let mut labels = vec![Column::Title.label()];
        labels.extend(self.columns.iter().map(Column::label));
        labels
    }
}

/// Why a query block can't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// Reads a query block. Blank lines and lines starting with `#` are ignored.
pub fn parse(source: &str) -> Result<ViewSpec, ParseError> {
    let mut spec = ViewSpec {
        from: Query::default(),
        has: Vec::new(),
        columns: Vec::new(),
        sort: None,
        limit: None,
    };
    let mut seen_from = false;
    let mut seen_columns = false;
    let mut seen_sort = false;
    let mut seen_limit = false;
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(error(format!(
                "“{line}” isn't a key and a value. Write it as key: value."
            )));
        };
        let (key, value) = (key.trim(), value.trim());
        match key.to_ascii_lowercase().as_str() {
            "from" => {
                if seen_from {
                    return Err(error("from: is written twice."));
                }
                seen_from = true;
                let (terms, has) = split_has(value);
                spec.from = Query::parse(&terms.join(" "));
                spec.has = has;
            }
            "columns" => {
                if seen_columns {
                    return Err(error("columns: is written twice."));
                }
                seen_columns = true;
                spec.columns = value
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(Column::parse)
                    .collect();
            }
            "sort" => {
                if seen_sort {
                    return Err(error("sort: is written twice."));
                }
                seen_sort = true;
                let mut parts = value.split_whitespace();
                let Some(column) = parts.next() else {
                    return Err(error("sort: needs a column, as in sort: rating desc."));
                };
                let descending = match parts.next() {
                    None => false,
                    Some("asc") => false,
                    Some("desc") => true,
                    Some(other) => {
                        return Err(error(format!(
                            "“{other}” isn't a sort order. Use asc or desc."
                        )));
                    }
                };
                if parts.next().is_some() {
                    return Err(error("sort: takes one column and asc or desc."));
                }
                spec.sort = Some(SortSpec {
                    column: Column::parse(column),
                    descending,
                });
            }
            "limit" => {
                if seen_limit {
                    return Err(error("limit: is written twice."));
                }
                seen_limit = true;
                spec.limit = Some(value.parse().map_err(|_| {
                    error(format!("“{value}” isn't a number of notes for limit:."))
                })?);
            }
            other => {
                return Err(error(format!(
                    "“{other}” isn't a query key. Use from, columns, sort or limit."
                )));
            }
        }
    }
    Ok(spec)
}

fn error(message: impl Into<String>) -> ParseError {
    ParseError {
        message: message.into(),
    }
}

/// Splits `from:` into the search terms and its `has:` clauses, keeping
/// quoted words together.
fn split_has(value: &str) -> (Vec<String>, Vec<Has>) {
    let mut terms = Vec::new();
    let mut has = Vec::new();
    for word in words(value) {
        let (negated, word) = match word.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, word.as_str()),
        };
        if let Some(key) = word.strip_prefix("has:") {
            let key = key.trim();
            if !key.is_empty() {
                has.push(Has {
                    key: unquote(key),
                    negated,
                });
            }
        } else if negated {
            terms.push(format!("-{word}"));
        } else {
            terms.push(word.to_owned());
        }
    }
    (terms, has)
}

fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == q => {
                quote = None;
                current.push(c);
            }
            Some('"') if c == '\\' => {
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            Some(_) => current.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                current.push(c);
            }
            None if c.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            None => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn unquote(text: &str) -> String {
    let text = text.trim();
    if text.len() >= 2
        && ((text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\'')))
    {
        text[1..text.len() - 1].to_owned()
    } else {
        text.to_owned()
    }
}

/// What a query evaluates against: a note's facts and its properties.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub book_id: String,
    pub book_name: Option<String>,
    pub status: String,
    pub tag_ids: Vec<String>,
    pub tag_names: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    /// Locked, and so never listed.
    pub locked: bool,
    /// In the trash, and so never listed.
    pub trashed: bool,
    pub properties: Vec<(String, Value)>,
}

impl ViewNote {
    fn facts(&self) -> NoteFacts<'_> {
        NoteFacts {
            title: &self.title,
            body: &self.body,
            book_id: &self.book_id,
            book_name: self.book_name.as_deref(),
            status: &self.status,
            tag_ids: &self.tag_ids,
            tag_names: &self.tag_names,
        }
    }

    fn property(&self, key: &str) -> Option<&Value> {
        properties::get(&self.properties, key)
    }
}

/// A row of the table: the note it links to, its title, and a cell for each
/// column after the title.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<Option<Value>>,
}

/// The notes `spec` lists, in its order, without locked or trashed notes
/// and without `self_id`.
pub fn evaluate(spec: &ViewSpec, notes: &[ViewNote], self_id: Option<&str>) -> Vec<ViewRow> {
    let mut rows: Vec<(ViewRow, Option<Value>)> = notes
        .iter()
        .filter(|note| !note.locked && !note.trashed)
        .filter(|note| self_id != Some(note.id.as_str()))
        .filter(|note| spec.from.matches(&note.facts()))
        .filter(|note| {
            spec.has
                .iter()
                .all(|has| note.property(&has.key).is_some() != has.negated)
        })
        .map(|note| {
            let sort = spec
                .sort
                .as_ref()
                .map(|sort| sort_value(note, &sort.column))
                .unwrap_or(None);
            (
                ViewRow {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    cells: spec
                        .columns
                        .iter()
                        .map(|column| column_value(note, column))
                        .collect(),
                },
                sort,
            )
        })
        .collect();
    if let Some(sort) = &spec.sort {
        // Missing values go last in either direction. Built-in dates sort by
        // their instant, not their day, and the note ID breaks ties so the
        // order is the same whatever order the notes came in.
        rows.sort_by(|(a, av), (b, bv)| {
            let ordering = ordered(av, bv, sort.descending);
            if ordering == Ordering::Equal {
                a.id.cmp(&b.id)
            } else {
                ordering
            }
        });
    }
    let mut rows: Vec<ViewRow> = rows.into_iter().map(|(row, _)| row).collect();
    if let Some(limit) = spec.limit {
        rows.truncate(limit);
    }
    rows
}

/// The value a sort compares. Built-in dates use their millisecond instant
/// so two edits on the same day still order by time.
fn sort_value(note: &ViewNote, column: &Column) -> Option<Value> {
    match column {
        Column::Updated => Some(Value::Number(note.updated_at as f64)),
        Column::Created => Some(Value::Number(note.created_at as f64)),
        _ => column_value(note, column),
    }
}

/// One cell of a row, as the note holds it.
pub fn column_value(note: &ViewNote, column: &Column) -> Option<Value> {
    match column {
        Column::Title => Some(Value::Text(note.title.clone())),
        Column::Updated => Some(Value::Date(date_of(note.updated_at))),
        Column::Created => Some(Value::Date(date_of(note.created_at))),
        Column::Book => note
            .book_name
            .as_ref()
            .filter(|name| !name.is_empty())
            .map(|name| Value::Text(name.clone())),
        Column::Tags => {
            if note.tag_names.is_empty() {
                None
            } else {
                Some(Value::Text(note.tag_names.join(", ")))
            }
        }
        Column::Property(key) => note.property(key).cloned(),
    }
}

/// `YYYY-MM-DD` for a time in milliseconds, in UTC as exports write dates.
fn date_of(ms: i64) -> String {
    crate::dates::Date::from_days(ms.div_euclid(86_400_000)).iso()
}

/// A cell as the app writes it: dates named, a boolean a check or a blank,
/// lists comma-separated. Pure.
pub fn cell_text(value: &Value) -> String {
    match value {
        Value::Bool(true) => "✓".into(),
        Value::Bool(false) => String::new(),
        Value::Date(date) => crate::dates::Date::parse_iso(date)
            .map(crate::dates::Date::long_label)
            .unwrap_or_else(|| date.clone()),
        Value::List(items) => items.iter().map(cell_text).collect::<Vec<_>>().join(", "),
        other => other.text(),
    }
}

/// A `query` fence found in a note: where it is in the source and the text
/// between its fences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryFence {
    pub range: std::ops::Range<usize>,
    pub source: String,
    /// Whitespace before the fence on its line, put before each table line.
    pub indent: String,
}

/// Whether a fence language names a query view. The first word counts, so
/// `query extra` is a query.
pub fn is_query_fence(lang: Option<&str>) -> bool {
    lang.and_then(|lang| lang.split_whitespace().next())
        .is_some_and(|lang| lang.eq_ignore_ascii_case("query"))
}

fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    // Front matter stays YAML: a fence inside a block scalar is not a query
    // fence, and the preview hides the block as the editor does.
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// The `query` fences in `body`, found in the Markdown syntax tree. A fence
/// inside another code block is not one.
pub fn query_fences(body: &str) -> Vec<QueryFence> {
    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        markdown::to_mdast(body, &parse_options())
    }));
    let root = match parsed {
        Ok(Ok(root)) => root,
        // A panic or a parse failure leaves the note as it was written.
        _ => return Vec::new(),
    };
    let mut fences = Vec::new();
    collect_fences(&root, body, &mut fences);
    fences
}

fn collect_fences(node: &markdown::mdast::Node, body: &str, fences: &mut Vec<QueryFence>) {
    if let markdown::mdast::Node::Code(code) = node {
        if !is_query_fence(code.lang.as_deref()) {
            return;
        }
        let Some(position) = node.position() else {
            return;
        };
        let start = position.start.offset;
        let end = position.end.offset;
        if start >= end || end > body.len() {
            return;
        }
        let line_start = body[..start].rfind('\n').map_or(0, |at| at + 1);
        let before = &body[line_start..start];
        let indent: String = before
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .collect();
        let range_start = if indent.len() == before.len() {
            line_start
        } else {
            start
        };
        fences.push(QueryFence {
            range: range_start..end,
            source: code.value.clone(),
            indent,
        });
        return;
    }
    for child in node.children().into_iter().flatten() {
        collect_fences(child, body, fences);
    }
}

/// Whether `body` has a query fence.
pub fn contains_query(body: &str) -> bool {
    might_contain_query(body) && !query_fences(body).is_empty()
}

/// Whether `body` could have a query fence, decided without parsing it.
/// A store change asks this on every open note; parsing the Markdown there
/// would block the UI for a note that has no query at all.
pub fn might_contain_query(body: &str) -> bool {
    body.lines().any(|line| {
        let mut rest = line.trim_start_matches([' ', '\t']);
        // A fence may sit in a block quote.
        while let Some(after) = rest.strip_prefix('>') {
            rest = after.trim_start_matches([' ', '\t']);
        }
        let fence = rest.chars().take_while(|c| *c == '`' || *c == '~').count();
        if fence < 3 {
            return false;
        }
        rest[fence..]
            .split_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("query"))
    })
}

/// The body as Markdown for an export: each `query` fence becomes the table
/// it draws, or its source and a badge when it can't be read. Locked and
/// trashed notes, and `self_id`, are left out as in [`evaluate`].
pub fn render_export(body: &str, notes: &[ViewNote], self_id: Option<&str>) -> String {
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    for fence in query_fences(body) {
        out.push_str(&body[at..fence.range.start]);
        match parse(&fence.source) {
            Ok(spec) => {
                // A table starts a block of its own.
                if !out.is_empty() && !out.ends_with("\n\n") {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push('\n');
                }
                let table = table_markdown(&spec, &evaluate(&spec, notes, self_id));
                out.push_str(&indent_lines(&table, &fence.indent));
                out.push_str("\n\n");
            }
            Err(err) => {
                out.push_str(&body[fence.range.clone()]);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&format!("*Query: {}*\n\n", escape_inline(&err.message)));
            }
        }
        at = fence.range.end;
    }
    out.push_str(&body[at..]);
    out
}

/// `text` with `indent` before every line.
fn indent_lines(text: &str, indent: &str) -> String {
    if indent.is_empty() {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + indent.len() * 4);
    for line in text.split_inclusive('\n') {
        out.push_str(indent);
        out.push_str(line);
    }
    out
}

/// The table as a GFM table: a header, then a row for each note, its title
/// first. Markdown, not HTML, so an export keeps the page's raw-HTML rule.
pub fn table_markdown(spec: &ViewSpec, rows: &[ViewRow]) -> String {
    let mut table = String::new();
    let line = |cells: &[String]| {
        let mut line = String::new();
        for cell in cells {
            line.push_str("| ");
            line.push_str(&cell_markdown(cell));
            line.push(' ');
        }
        line.push_str("|\n");
        line
    };
    table.push_str(&line(&spec.labels()));
    table.push_str(&line(
        &spec
            .labels()
            .iter()
            .map(|_| "---".to_owned())
            .collect::<Vec<_>>(),
    ));
    if rows.is_empty() {
        let mut empty = vec!["No notes match.".to_owned()];
        empty.extend((1..spec.labels().len()).map(|_| String::new()));
        table.push_str(&line(&empty));
        return table;
    }
    for row in rows {
        let mut cells = vec![row.title.clone()];
        cells.extend(
            row.cells
                .iter()
                .map(|cell| cell.as_ref().map(export_text).unwrap_or_default()),
        );
        table.push_str(&line(&cells));
    }
    table
}

/// A cell as an export writes it.
fn export_text(value: &Value) -> String {
    match value {
        Value::Bool(true) => "\u{2713}".into(),
        Value::Bool(false) => String::new(),
        Value::Date(date) => crate::dates::Date::parse_iso(date)
            .map(crate::dates::Date::long_label)
            .unwrap_or_else(|| date.clone()),
        Value::List(items) => items.iter().map(export_text).collect::<Vec<_>>().join(", "),
        other => other.text(),
    }
}

/// A table cell: one line, with the pipes and Markdown marks a cell can't
/// hold escaped.
fn cell_markdown(text: &str) -> String {
    let text = text.replace(['\n', '\r'], " ");
    escape_inline(&text).replace('|', "\\|")
}

/// Text that reads as itself inside emphasis.
fn escape_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '*' | '_' | '`' | '[' | ']' | '<' | '>') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `a` before `b`, missing values last whatever the direction. Ties keep
/// their order (the sort is stable).
fn ordered(a: &Option<Value>, b: &Option<Value>, descending: bool) -> Ordering {
    let missing = |value: &Option<Value>| value.as_ref().is_none_or(Value::is_empty);
    match (missing(a), missing(b)) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            let ordering = compare(a.as_ref(), b.as_ref());
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        }
    }
}

fn compare(a: Option<&Value>, b: Option<&Value>) -> Ordering {
    let (Some(a), Some(b)) = (a, b) else {
        return Ordering::Equal;
    };
    let rank = |value: &Value| match value {
        Value::Number(_) => 0,
        Value::Date(_) => 1,
        Value::Bool(_) => 2,
        Value::Text(_) => 3,
        Value::List(_) => 4,
        Value::Raw(_) => 5,
    };
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (Value::Date(a), Value::Date(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Text(a), Value::Text(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
        (Value::List(a), Value::List(b)) => a
            .iter()
            .map(Value::text)
            .collect::<Vec<_>>()
            .join(", ")
            .to_lowercase()
            .cmp(
                &b.iter()
                    .map(Value::text)
                    .collect::<Vec<_>>()
                    .join(", ")
                    .to_lowercase(),
            ),
        _ => rank(a).cmp(&rank(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn note(id: &str, title: &str, properties: Vec<(&str, Value)>) -> ViewNote {
        ViewNote {
            id: format!("note:{id}"),
            title: title.into(),
            body: String::new(),
            book_id: "book:clients".into(),
            book_name: Some("Clients".into()),
            status: "active".into(),
            tag_ids: Vec::new(),
            tag_names: Vec::new(),
            created_at: 0,
            updated_at: 0,
            locked: false,
            trashed: false,
            properties: properties
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        }
    }

    #[test]
    fn parses_a_query() {
        let spec = parse(
            "from: book:Clients status:active has:due\n\
             columns: client, rating, due\n\
             sort: rating desc\n\
             limit: 50\n",
        )
        .unwrap();
        assert_eq!(spec.from, Query::parse("book:Clients status:active"));
        assert_eq!(
            spec.has,
            vec![Has {
                key: "due".into(),
                negated: false
            }]
        );
        assert_eq!(
            spec.columns,
            vec![
                Column::Property("client".into()),
                Column::Property("rating".into()),
                Column::Property("due".into()),
            ]
        );
        assert_eq!(
            spec.sort,
            Some(SortSpec {
                column: Column::Property("rating".into()),
                descending: true
            })
        );
        assert_eq!(spec.limit, Some(50));
    }

    #[test]
    fn a_fence_in_front_matter_is_not_a_query() {
        let body = "---\nnote: |\n  ```query\n  from: has:x\n  ```\n---\nBody\n";
        assert!(query_fences(body).is_empty());
        assert!(!contains_query(body));
    }

    #[test]
    fn a_cheap_check_finds_the_fences_that_matter() {
        for body in [
            "```query\nfrom: has:x\n```",
            "~~~Query\nfrom: has:x\n~~~",
            "> ```query extra\n> from: has:x\n> ```",
            "  ````query\n  from: has:x\n  ````",
        ] {
            assert!(might_contain_query(body), "{body:?}");
            assert!(contains_query(body), "{body:?}");
        }
        for body in [
            "Prose about a query, but no fence.",
            "```js\nconst query = 1;\n```",
            "`query` is inline",
            "```querying\nnot a query\n```",
        ] {
            assert!(!might_contain_query(body), "{body:?}");
            assert!(!contains_query(body), "{body:?}");
        }
    }

    #[test]
    fn bad_syntax_says_what_is_wrong() {
        for source in [
            "rating 4",
            "sort: rating sideways",
            "sort:",
            "limit: many",
            "from: a\nfrom: b",
            "nope: 1",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn evaluation_leaves_out_locked_trashed_and_self() {
        let mut locked = note("locked", "Locked", Vec::new());
        locked.locked = true;
        let mut trashed = note("trashed", "Trashed", Vec::new());
        trashed.trashed = true;
        let notes = vec![
            note("a", "A", vec![("rating", Value::Number(2.))]),
            locked,
            trashed,
            note("self", "Self", Vec::new()),
        ];
        let spec = parse("from: status:active\ncolumns: rating").unwrap();
        let rows = evaluate(&spec, &notes, Some("note:self"));
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, ["note:a"]);
    }

    #[test]
    fn has_filters_by_property() {
        let notes = vec![
            note("a", "A", vec![("due", Value::Date("2026-11-01".into()))]),
            note("b", "B", Vec::new()),
        ];
        let spec = parse("from: has:due").unwrap();
        let rows = evaluate(&spec, &notes, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "A");
        let spec = parse("from: -has:due").unwrap();
        let rows = evaluate(&spec, &notes, None);
        assert_eq!(rows[0].title, "B");
    }

    #[test]
    fn sorting_is_stable_and_missing_values_are_last() {
        let notes = vec![
            note("a", "A", vec![("rating", Value::Text("b".into()))]),
            note("b", "B", Vec::new()),
            note("c", "C", vec![("rating", Value::Number(2.))]),
            note("d", "D", vec![("rating", Value::Number(2.))]),
            note("e", "E", vec![("rating", Value::Bool(true))]),
            note("f", "F", vec![("rating", Value::Date("2026-01-02".into()))]),
        ];
        let spec = parse("sort: rating asc").unwrap();
        let rows = evaluate(&spec, &notes, None);
        let titles: Vec<&str> = rows.iter().map(|row| row.title.as_str()).collect();
        // Numbers, then dates, then booleans, then text; ties in written
        // order; the missing one last.
        assert_eq!(titles, ["C", "D", "F", "E", "A", "B"]);
        let spec = parse("sort: rating desc").unwrap();
        let rows = evaluate(&spec, &notes, None);
        let titles: Vec<&str> = rows.iter().map(|row| row.title.as_str()).collect();
        assert_eq!(titles, ["A", "E", "F", "C", "D", "B"]);
    }

    #[test]
    fn built_in_columns() {
        let mut a = note("a", "A", Vec::new());
        a.updated_at = 86_400_000 * 10;
        a.tag_names = vec!["work".into(), "clients".into()];
        let rows = evaluate(
            &parse("columns: updated, created, book, tags").unwrap(),
            &[a],
            None,
        );
        assert_eq!(
            rows[0].cells,
            vec![
                Some(Value::Date("1970-01-11".into())),
                Some(Value::Date("1970-01-01".into())),
                Some(Value::Text("Clients".into())),
                Some(Value::Text("work, clients".into())),
            ]
        );
    }

    #[test]
    fn an_export_draws_the_same_table() {
        let mut locked = note("locked", "Locked", vec![("rating", Value::Number(5.))]);
        locked.locked = true;
        let notes = vec![
            note(
                "a",
                "Ana",
                vec![
                    ("rating", Value::Number(4.)),
                    ("done", Value::Bool(true)),
                    ("due", Value::Date("2026-11-01".into())),
                ],
            ),
            locked,
        ];
        let body =
            "Before.\n\n```query\nfrom: status:active\ncolumns: rating, done, due\n```\n\nAfter.\n";
        let markdown = render_export(body, &notes, None);
        assert!(markdown.starts_with("Before."), "{markdown}");
        assert!(
            markdown.contains("| Title | rating | done | due |"),
            "{markdown}"
        );
        assert!(
            markdown.contains("| Ana | 4 | \u{2713} | Sunday, 1 November 2026 |"),
            "{markdown}"
        );
        assert!(markdown.contains("After."), "{markdown}");
        assert!(!markdown.contains("Locked"), "{markdown}");
    }

    #[test]
    fn an_export_keeps_a_mistake_as_its_source() {
        let markdown = render_export("```query\nnope: 1\n```\n", &[], None);
        assert!(markdown.contains("```query"), "{markdown}");
        assert!(markdown.contains("*Query:"), "{markdown}");
        assert!(markdown.contains("isn't a query key"), "{markdown}");
    }

    #[test]
    fn built_in_dates_sort_by_their_instant() {
        let mut a = note("a", "A", Vec::new());
        a.updated_at = 1_000;
        let mut b = note("b", "B", Vec::new());
        b.updated_at = 2_000;
        let spec = parse("sort: updated desc\nlimit: 1").unwrap();
        let rows = evaluate(&spec, &[a, b], None);
        assert_eq!(rows[0].title, "B");
    }

    #[test]
    fn ties_break_on_the_note_id() {
        let mut a = note("a", "A", Vec::new());
        a.updated_at = 1_000;
        let mut b = note("b", "B", Vec::new());
        b.updated_at = 1_000;
        // Whatever order the notes arrive in, the order is the same.
        let spec = parse("sort: updated desc").unwrap();
        let rows = evaluate(&spec, &[b.clone(), a.clone()], None);
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            ["note:a", "note:b"]
        );
        let rows = evaluate(&spec, &[a, b], None);
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            ["note:a", "note:b"]
        );
    }

    #[test]
    fn fences_are_found_where_markdown_finds_them() {
        let body = "- item\n\n  ~~~query extra\n  from: has:due\n  columns: due\n  ~~~\n";
        let notes = vec![note(
            "a",
            "Ana",
            vec![("due", Value::Date("2026-11-01".into()))],
        )];
        let markdown = render_export(body, &notes, None);
        assert!(markdown.contains("  | Title | due |"), "{markdown}");
        assert!(markdown.contains("  | Ana |"), "{markdown}");
        assert!(!markdown.contains("~~~"), "{markdown}");
        assert!(contains_query(body));
    }

    #[test]
    fn a_query_fence_inside_a_code_block_is_not_one() {
        let body = "````\n```query\nfrom: has:due\n```\n````\n";
        assert!(!contains_query(body));
        assert_eq!(render_export(body, &[], None), body);
    }

    proptest! {
        /// Locked and trashed notes with any properties never change what a
        /// query shows or what an export writes.
        #[test]
        fn prop_locked_and_trashed_notes_never_show(
            live in prop::collection::vec(("[a-z]{1,6}", "[A-Za-z0-9 #:]{0,12}"), 0..5),
            secret in prop::collection::vec(("[a-z]{1,6}", "[A-Za-z0-9 #:]{0,12}"), 0..5),
        ) {
            let live_props: Vec<(&str, Value)> = live
                .iter()
                .map(|(key, value)| (key.as_str(), Value::Text(value.clone())))
                .collect();
            let mut notes = vec![note("live", "Live", live_props)];
            let base = evaluate(&parse("columns: a, b").unwrap(), &notes, None);
            let base_markdown = render_export("```query\ncolumns: a, b\n```\n", &notes, None);
            for (ix, locked) in [true, false].into_iter().enumerate() {
                let props: Vec<(&str, Value)> = secret
                    .iter()
                    .map(|(key, value)| (key.as_str(), Value::Text(value.clone())))
                    .collect();
                let mut hidden = note(&format!("h{ix}"), "SECRET", props);
                hidden.locked = locked;
                hidden.trashed = !locked;
                hidden.body = "SECRET body".into();
                notes.push(hidden);
            }
            prop_assert_eq!(evaluate(&parse("columns: a, b").unwrap(), &notes, None), base);
            prop_assert_eq!(
                render_export("```query\ncolumns: a, b\n```\n", &notes, None),
                base_markdown
            );
        }

        /// Sorted rows follow their values; missing values come last in
        /// either direction.
        #[test]
        fn prop_sort_order_holds(
            values in prop::collection::vec(prop::option::of(-50i64..50), 0..20),
            descending: bool,
        ) {
            let notes: Vec<ViewNote> = values
                .iter()
                .enumerate()
                .map(|(ix, value)| {
                    let props = value
                        .map(|n| vec![("rating", Value::Number(n as f64))])
                        .unwrap_or_default();
                    note(&format!("n{ix:02}"), &format!("N{ix}"), props)
                })
                .collect();
            let source = if descending {
                "sort: rating desc"
            } else {
                "sort: rating asc"
            };
            let spec = parse(&format!("columns: rating\n{source}")).unwrap();
            let rows = evaluate(&spec, &notes, None);
            let mut seen_missing = false;
            for pair in rows.windows(2) {
                let cell = |row: &ViewRow| match &row.cells[0] {
                    Some(Value::Number(n)) => Some(*n),
                    _ => None,
                };
                match (cell(&pair[0]), cell(&pair[1])) {
                    (None, _) => seen_missing = true,
                    (Some(_), None) => prop_assert!(!seen_missing),
                    (Some(a), Some(b)) => {
                        prop_assert!(!seen_missing, "a value after a missing one");
                        if descending {
                            prop_assert!(a >= b)
                        } else {
                            prop_assert!(a <= b)
                        }
                    }
                }
            }
            if let Some(last) = rows.last()
                && matches!(last.cells[0], Some(Value::Number(_)))
            {
                prop_assert!(!seen_missing);
            }
        }
    }

    proptest! {
        /// Invariant: every row is a listed note, in the columns asked for,
        /// and a limit never keeps more.
        #[test]
        fn prop_rows_match_the_spec(
            count in 0usize..20,
            limit in 0usize..10,
            columns in prop::collection::vec(
                prop_oneof![Just(Column::Tags), Just(Column::Book), Just(Column::Updated),
                    "[a-z]{1,5}".prop_map(Column::Property)],
                0..4,
            ),
        ) {
            let notes: Vec<ViewNote> = (0..count)
                .map(|n| note(&format!("n{n}"), &format!("Note {n}"), Vec::new()))
                .collect();
            let source = format!(
                "columns: {}\nlimit: {limit}",
                columns.iter().map(Column::label).collect::<Vec<_>>().join(", "),
            );
            let spec = parse(&source).unwrap();
            let rows = evaluate(&spec, &notes, None);
            prop_assert!(rows.len() <= notes.len());
            prop_assert!(rows.len() <= limit);
            for row in &rows {
                prop_assert_eq!(row.cells.len(), spec.columns.len());
            }
        }
    }
}
