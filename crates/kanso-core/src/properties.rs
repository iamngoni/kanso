//! A note's properties: the YAML front matter at the top of its text.
//!
//! Only the YAML notes use is understood: `key: value` lines, quoted
//! strings, numbers, booleans, ISO dates, inline lists (`[a, b]`) and block
//! lists (`- a`). Anything else, such as a nested map, a block scalar or an
//! anchor, is kept as [`Value::Raw`]; parsing never fails.
//!
//! [`set_property`] changes one value and leaves every other byte alone,
//! adding front matter when the note has none. An entry ends at its last
//! indented continuation line or `- item` line: comments and blank lines
//! after it belong to no entry and are never touched.

use crate::dates::Date;

/// A property's value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),
    /// `YYYY-MM-DD`.
    Date(String),
    List(Vec<Value>),
    /// Anything the parser doesn't understand: a nested map, a block scalar,
    /// a flow map, an anchor, an alias or a tag. Kept as written, and never
    /// rewritten by the inspector.
    Raw(String),
}

impl Value {
    /// The value as a line of text: lists comma-separated, dates as written.
    pub fn text(&self) -> String {
        match self {
            Value::Text(text) => text.clone(),
            Value::Number(number) => number_text(*number),
            Value::Bool(on) => on.to_string(),
            Value::Date(date) => date.clone(),
            Value::List(items) => items
                .iter()
                .map(|item| item.text())
                .collect::<Vec<_>>()
                .join(", "),
            Value::Raw(raw) => raw.clone(),
        }
    }

    /// Whether the value says nothing: no text, no items.
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Text(text) => text.is_empty(),
            Value::List(items) => items.is_empty(),
            Value::Raw(raw) => raw.trim().is_empty(),
            _ => false,
        }
    }

    /// Whether the value is kept as written because it isn't understood.
    pub fn is_raw(&self) -> bool {
        matches!(self, Value::Raw(_))
    }
}

/// A note's text split at its front matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontMatter<'a> {
    /// Between the opening and closing lines.
    pub front: &'a str,
    /// Where `front` starts in the note.
    pub front_start: usize,
    /// Where the closing `---` or `...` line starts.
    pub close_start: usize,
    /// Where the text after the closing line starts.
    pub body_start: usize,
    /// The line ending the opening line uses.
    pub eol: &'a str,
}

/// The front matter of `body`, if it opens with `---` and closes with
/// `---` or `...`. A UTF-8 BOM before the opening line and trailing
/// whitespace on either marker line are allowed.
pub fn split(body: &str) -> Option<FrontMatter<'_>> {
    let bom = if body.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let rest = &body[bom..];
    // The opening line needs its newline: without one there is no closing.
    let first_end = rest.find('\n')?;
    let first = rest[..first_end]
        .strip_suffix('\r')
        .unwrap_or(&rest[..first_end]);
    if first.trim_end_matches([' ', '\t']) != "---" {
        return None;
    }
    let opening = bom + first_end + 1;
    let eol = if rest[..first_end].ends_with('\r') {
        "\r\n"
    } else {
        "\n"
    };
    let mut offset = opening;
    for line in body[opening..].split_inclusive('\n') {
        let end = offset + line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        let marker = content.trim_end_matches([' ', '\t']);
        if marker == "---" || marker == "..." {
            let front = &body[opening..offset];
            // Two rules around prose are not front matter, nor is a block
            // holding anything but properties. Without this, a note that
            // starts with `---` and has another rule later would hide, edit
            // and export the prose between them as if it were properties.
            if !is_front_matter(front) {
                return None;
            }
            return Some(FrontMatter {
                front,
                front_start: opening,
                close_start: offset,
                body_start: end,
                eol: &body[opening - eol.len()..opening],
            });
        }
        offset = end;
    }
    None
}

/// Whether the text between the rules is properties and not prose. Every
/// top-level line must be blank, a comment, a key line or a continuation of
/// the entry above; an indented line is a continuation.
fn is_front_matter(front: &str) -> bool {
    front.split_inclusive('\n').all(|line| {
        let content = line.trim_end_matches(['\n', '\r']);
        content.trim().is_empty()
            || content.starts_with([' ', '\t'])
            || content.trim_start().starts_with('#')
            || key_line(content).is_some()
            || continuation(content)
    })
}

/// The front matter between the delimiters, if `body` has any.
pub fn front_matter(body: &str) -> Option<&str> {
    split(body).map(|front| front.front)
}

/// The properties in `body`'s front matter, in the order they are written.
pub fn parse(body: &str) -> Vec<(String, Value)> {
    let Some(front) = split(body) else {
        return Vec::new();
    };
    parse_front(front.front)
}

/// The properties in front-matter text (the lines between the delimiters).
pub fn parse_front(front: &str) -> Vec<(String, Value)> {
    entries(front)
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect()
}

/// The value of property `key`, ignoring case; the last one written wins.
pub fn get<'a>(properties: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    let key = key.trim();
    properties
        .iter()
        .rev()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

/// `body` with property `key` set to `value`. The front matter is added if
/// the note has none; every other byte stays as it was. When `key` already
/// has `value`, `body` is returned unchanged.
pub fn set_property(body: &str, key: &str, value: &Value) -> String {
    let key = key.trim();
    if key.is_empty() {
        return body.to_owned();
    }
    let Some(front) = split(body) else {
        let eol = line_ending(body);
        let bom = if body.starts_with('\u{feff}') {
            '\u{feff}'.len_utf8()
        } else {
            0
        };
        let line = format!("{}: {}{eol}", quote_key(key), render(value));
        return format!("{}---{eol}{line}---{eol}{}", &body[..bom], &body[bom..]);
    };
    let found = entries(front.front)
        .into_iter()
        .rev()
        .find(|entry| entry.key.eq_ignore_ascii_case(key));
    match found {
        // Nothing to do: the value is already what was asked for.
        Some(entry) if entry.value == *value => body.to_owned(),
        Some(entry) => replace_entry(body, &front, &entry, value),
        None => {
            let at = front.close_start;
            let line = format!("{}: {}{}", quote_key(key), render(value), front.eol);
            format!("{}{}{}", &body[..at], line, &body[at..])
        }
    }
}

/// `body` with the property at `index` set to `value`, in the order
/// [`parse`] returns. Unlike [`set_property`], which edits the last entry a
/// key names, this edits exactly the row shown — the first of two duplicate
/// keys included. Every other byte stays as it was, and an index out of
/// range leaves `body` alone.
pub fn set_property_at(body: &str, index: usize, value: &Value) -> String {
    let Some(front) = split(body) else {
        return body.to_owned();
    };
    let entries = entries(front.front);
    let Some(entry) = entries.get(index) else {
        return body.to_owned();
    };
    if entry.value == *value {
        return body.to_owned();
    }
    replace_entry(body, &front, entry, value)
}

/// `body` without the property at `index`, in the order [`parse`] returns:
/// its key line and everything that belongs to its value. Every other byte
/// stays as it was (the comments and blank lines around it too, and the
/// front matter itself when it was the only entry). An index out of range
/// leaves `body` alone.
pub fn remove_property_at(body: &str, index: usize) -> String {
    let Some(front) = split(body) else {
        return body.to_owned();
    };
    let entries = entries(front.front);
    let Some(entry) = entries.get(index) else {
        return body.to_owned();
    };
    let start = front.front_start + entry.start;
    let end = front.front_start + entry.end;
    format!("{}{}", &body[..start], &body[end..])
}

/// `body` with `entry`'s value replaced, keeping its key line's comment and
/// any comment lines inside a block list. Comments from inside the old value
/// sit after the new line, so the value's shape may change without losing
/// the note's words.
fn replace_entry(body: &str, front: &FrontMatter<'_>, entry: &Entry, value: &Value) -> String {
    let start = front.front_start + entry.start;
    let end = front.front_start + entry.end;
    let prefix = &body[start..front.front_start + entry.rest_start];
    let mut line = format!("{prefix} {}", render(value));
    // Any scalar, list or comment can follow a value; only a raw value,
    // which is written exactly as it was, can't carry one safely.
    if !value.is_raw()
        && let Some(comment) = &entry.comment
    {
        if !comment.starts_with([' ', '\t']) {
            line.push(' ');
        }
        line.push_str(comment);
    }
    line.push_str(front.eol);
    for old in body[start..end].split_inclusive('\n').skip(1) {
        let content = old.trim_end_matches(['\n', '\r']);
        if content.trim_start().starts_with('#') {
            line.push_str(old);
        }
    }
    format!("{}{}{}", &body[..start], line, &body[end..])
}

/// The value a typed field should set, when the property it edits held
/// `like`: a number stays a number when it parses, a date stays a date, a
/// list is written comma-separated, and a new property takes the type its
/// text looks like.
pub fn typed(text: &str, like: Option<&Value>) -> Value {
    let text = text.trim();
    match like {
        Some(Value::Number(_)) => text
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Value::Number)
            .unwrap_or_else(|| Value::Text(text.to_owned())),
        Some(Value::Date(_)) => Date::parse_iso(text)
            .map(|date| Value::Date(date.iso()))
            .unwrap_or_else(|| Value::Text(text.to_owned())),
        Some(Value::Bool(on)) => match text.to_ascii_lowercase().as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::Bool(*on),
        },
        Some(Value::List(_)) => Value::List(
            split_flow(text)
                .into_iter()
                .map(|item| scalar(&item))
                .collect(),
        ),
        Some(Value::Raw(_)) => Value::Raw(text.to_owned()),
        Some(Value::Text(_)) => Value::Text(text.to_owned()),
        None => scalar(text),
    }
}

/// The line ending `text` uses: `\r\n` when its first line ends that way,
/// else `\n`.
pub fn line_ending(text: &str) -> &'static str {
    match text.find('\n') {
        Some(at) if text[..at].ends_with('\r') => "\r\n",
        _ => "\n",
    }
}

struct Entry {
    key: String,
    value: Value,
    /// Where the entry starts and ends in the front matter.
    start: usize,
    end: usize,
    /// Where the value starts (just after the key's colon).
    rest_start: usize,
    /// The inline comment on the key line, if any, as written.
    comment: Option<String>,
}

/// The top-level entries of front matter, with where each one is.
fn entries(front: &str) -> Vec<Entry> {
    struct Raw {
        key: String,
        start: usize,
        rest_start: usize,
        comment: Option<String>,
        lines: Vec<(usize, String)>,
    }
    let mut raw: Vec<Raw> = Vec::new();
    let mut offset = 0;
    let mut open = false;
    for line in front.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        if let Some((key, rest)) = key_line(content) {
            raw.push(Raw {
                key,
                start: offset,
                rest_start: offset + (content.len() - rest.len()),
                comment: split_comment(rest).1.map(|comment| {
                    let at = rest.len() - comment.len();
                    let end = rest[..at].trim_end().len();
                    rest[end..].to_owned()
                }),
                lines: vec![(offset, line.to_owned())],
            });
            open = true;
        } else if open && continuation(content) {
            if let Some(entry) = raw.last_mut() {
                entry.lines.push((offset, line.to_owned()));
            }
        } else {
            // A top-level line that isn't an indented continuation or a
            // list item belongs to no entry: it stays exactly as written.
            open = false;
        }
        offset += line.len();
    }
    raw.into_iter()
        .map(|entry| {
            let block_scalar = entry
                .lines
                .first()
                .and_then(|(_, line)| key_line(line.trim_end_matches(['\n', '\r'])))
                .is_some_and(|(_, rest)| rest.trim_start().starts_with(['|', '>']));
            let mut lines = entry.lines;
            // Trailing blank lines and comments belong to no entry.
            while lines.len() > 1 {
                let text = lines
                    .last()
                    .map(|(_, line)| line.trim_end_matches(['\n', '\r']))
                    .unwrap_or("");
                let blank = text.trim().is_empty();
                let comment = if block_scalar {
                    text.starts_with('#')
                } else {
                    text.trim_start().starts_with('#')
                };
                if blank || comment {
                    lines.pop();
                } else {
                    break;
                }
            }
            let (value, _) = value_of(&lines);
            let last = lines
                .last()
                .map(|(at, line)| at + line.len())
                .unwrap_or(entry.start);
            Entry {
                key: entry.key,
                value,
                start: entry.start,
                end: last,
                rest_start: entry.rest_start,
                comment: entry.comment,
            }
        })
        .collect()
}

/// `key: rest` if `line` is a top-level key line.
fn key_line(line: &str) -> Option<(String, &str)> {
    split_key(line)
}

/// Whether a line after a key belongs to its entry: an indented line, a
/// blank line, a comment, or a block-list item. Anything else at the top
/// level is not part of the value.
fn continuation(line: &str) -> bool {
    line.is_empty()
        || line.starts_with([' ', '\t', '#'])
        || line == "-"
        || line.starts_with("- ")
        || line.starts_with("-\t")
}

/// `key: rest` → (key, rest). The colon must be followed by a space or the
/// end of the line; quoted keys are unquoted.
pub fn split_key(line: &str) -> Option<(String, &str)> {
    let first = line.chars().next()?;
    if matches!(first, ' ' | '\t' | '#') {
        return None;
    }
    // `- item` is a list item, not a key; `-k: v` is a key.
    if first == '-' && (line.len() == 1 || line[1..].starts_with([' ', '\t'])) {
        return None;
    }
    let (key, rest) = if first == '"' || first == '\'' {
        let (key, rest) = unquote_key(line, first)?;
        (key, rest.strip_prefix(':')?)
    } else {
        let mut at = None;
        for (ix, c) in line.char_indices() {
            if c == ':' {
                let next = line[ix + 1..].chars().next();
                if next.is_none_or(|n| n == ' ' || n == '\t') {
                    at = Some(ix);
                    break;
                }
            }
        }
        let at = at?;
        (line[..at].trim_end().to_owned(), &line[at + 1..])
    };
    if key.is_empty() || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    Some((key, rest))
}

/// Unquotes a key at the start of `line`, returning it and the text after
/// its closing quote.
fn unquote_key(line: &str, quote: char) -> Option<(String, &str)> {
    let mut out = String::new();
    let mut chars = line[1..].char_indices().peekable();
    while let Some((ix, c)) = chars.next() {
        if quote == '"' && c == '\\' {
            match chars.next() {
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                Some((_, 'r')) => out.push('\r'),
                Some((_, other)) => out.push(other),
                None => out.push('\\'),
            }
            continue;
        }
        if c == quote {
            if quote == '\'' && chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
                out.push('\'');
                continue;
            }
            let end = 1 + ix + c.len_utf8();
            return Some((out, &line[end..]));
        }
        out.push(c);
    }
    None
}

/// The value of an entry, and its text as written after the key.
fn value_of(lines: &[(usize, String)]) -> (Value, String) {
    let first = lines[0].1.trim_end_matches(['\n', '\r']);
    let Some((_, rest)) = split_key(first) else {
        return (Value::Text(String::new()), String::new());
    };
    let more: Vec<&str> = lines[1..]
        .iter()
        .map(|(_, line)| line.trim_end_matches(['\n', '\r']))
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect();
    let raw = || raw_of(rest, lines);
    let (value_part, _) = split_comment(rest);
    let trimmed = value_part.trim();
    if trimmed.starts_with('|') || trimmed.starts_with('>') {
        // A block scalar: kept as written, never rewritten.
        return (Value::Raw(raw()), rest.to_owned());
    }
    if trimmed.starts_with('{') || trimmed.starts_with(['&', '*', '!']) {
        return (Value::Raw(raw()), rest.to_owned());
    }
    if trimmed.starts_with('[') && !trimmed.ends_with(']') {
        return (Value::Raw(raw()), rest.to_owned());
    }
    if trimmed.is_empty() {
        if more.is_empty() {
            return (Value::Text(String::new()), rest.to_owned());
        }
        if more.iter().all(|line| line.trim_start().starts_with('-')) {
            let items: Vec<&str> = more
                .iter()
                .map(|line| line.trim_start().strip_prefix('-').unwrap_or("").trim())
                .collect();
            // A list of maps, anchors, aliases or nested lists isn't a list
            // of text items: keep it as written so editing can't flatten it.
            if items.iter().any(|item| !is_text_item(item)) {
                return (Value::Raw(raw()), rest.to_owned());
            }
            let items = items.iter().map(|item| scalar(item)).collect();
            return (Value::List(items), rest.to_owned());
        }
        // A nested map, or anything else not understood: its lines as
        // written.
        return (Value::Raw(raw()), rest.to_owned());
    }
    if !more.is_empty() {
        // A quoted or flow value continued on the next lines, or something
        // we don't interpret, kept as written.
        if trimmed.starts_with(['"', '\'', '[', '{', '&', '*', '!']) {
            return (Value::Raw(raw()), rest.to_owned());
        }
        let mut text = trimmed.to_owned();
        for line in more {
            text.push(' ');
            text.push_str(line.trim());
        }
        return (
            Value::Text(strip_comment(&text).to_owned()),
            rest.to_owned(),
        );
    }
    if let Some(inner) = trimmed.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        let items = split_flow(inner)
            .into_iter()
            .map(|item| scalar(&item))
            .collect();
        return (Value::List(items), rest.to_owned());
    }
    (scalar(trimmed), rest.to_owned())
}

/// The entry's value as written, from just after the colon on.
fn raw_of(rest: &str, lines: &[(usize, String)]) -> String {
    let mut raw = rest.trim_start().to_owned();
    for (_, line) in &lines[1..] {
        raw.push_str(line);
    }
    raw
}

/// Whether a block-list item reads as a text item. A map, anchor, alias,
/// tag, block scalar or nested list is not a text item: the list holding it
/// stays [`Value::Raw`].
fn is_text_item(item: &str) -> bool {
    let item = item.trim();
    if item.is_empty() {
        return true;
    }
    if item.starts_with(['[', '{', '&', '*', '!', '|', '>', '?']) {
        return false;
    }
    if item == "-" || item.starts_with("- ") || item.starts_with("-\t") {
        return false;
    }
    // A quoted item is text whatever it says.
    if item.starts_with(['"', '\'']) {
        return true;
    }
    split_key(item).is_none()
}

/// A scalar as YAML reads it: a quoted or plain string, a number, a boolean
/// or an ISO date.
fn scalar(text: &str) -> Value {
    let (text, _) = split_comment(text.trim());
    let text = text.trim();
    if let Some(quoted) = unquote_exact(text) {
        return Value::Text(quoted);
    }
    if text.is_empty() {
        return Value::Text(String::new());
    }
    if text.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if text.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Some(date) = Date::parse_iso(text) {
        return Value::Date(date.iso());
    }
    if let Ok(number) = text.parse::<i64>() {
        return Value::Number(number as f64);
    }
    if let Ok(number) = text.parse::<f64>()
        && number.is_finite()
    {
        return Value::Number(number);
    }
    Value::Text(text.to_owned())
}

/// A quoted scalar `text` with nothing after it.
fn unquote_exact(text: &str) -> Option<String> {
    let mut chars = text.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut out = String::new();
    while let Some(c) = chars.next() {
        match (quote, c) {
            ('"', '\\') => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            },
            ('\'', '\'') if chars.as_str().starts_with('\'') => {
                chars.next();
                out.push('\'');
            }
            (_, c) if c == quote => {
                return chars.as_str().is_empty().then_some(out);
            }
            _ => out.push(c),
        }
    }
    None
}

/// Splits `text` at a trailing ` # comment`, outside quotes: the value
/// before it and the comment as written, `#` first.
fn split_comment(text: &str) -> (&str, Option<&str>) {
    let mut quote: Option<char> = None;
    let mut chars = text.char_indices().peekable();
    while let Some((ix, c)) = chars.next() {
        match quote {
            Some(q) if c == q => {
                if q == '\'' && chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    quote = None;
                }
            }
            Some('"') if c == '\\' => {
                chars.next();
            }
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' => {
                if ix == 0 {
                    return ("", Some(text));
                }
                if text[..ix].ends_with([' ', '\t']) {
                    return (text[..ix].trim_end(), Some(&text[ix..]));
                }
            }
            None => {}
        }
    }
    (text, None)
}

/// A plain scalar without a trailing ` # comment`. Quoted text keeps its `#`.
fn strip_comment(text: &str) -> &str {
    split_comment(text).0
}

/// Splits `a, "b, c", d` at the commas outside quotes.
fn split_flow(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == q => {
                if q == '\'' && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    current.push(chars.next().unwrap_or('\''));
                    continue;
                }
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
            None if c == ',' => {
                items.push(current.trim().to_owned());
                current.clear();
            }
            None => current.push(c),
        }
    }
    if !current.trim().is_empty() || !items.is_empty() {
        items.push(current.trim().to_owned());
    }
    items.retain(|item| !item.is_empty());
    items
}

/// A value as YAML that [`parse`] reads back as itself.
fn render(value: &Value) -> String {
    match value {
        Value::Text(text) => quote(text),
        Value::Number(number) => number_text(*number),
        Value::Bool(on) => on.to_string(),
        Value::Date(date) => date.clone(),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(render).collect::<Vec<_>>().join(", ")
        ),
        Value::Raw(raw) => raw.clone(),
    }
}

fn number_text(number: f64) -> String {
    if number.fract() == 0. && number.abs() < 9e15 {
        format!("{}", number as i64)
    } else {
        format!("{number}")
    }
}

/// Text YAML reads as something other than a string: its 1.1/1.2 keywords
/// and number forms Kanso's own parser leaves as text (hex, octal, binary,
/// infinities and NaN). Such text is quoted, so it reads back as itself.
fn yaml_reserved(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "~" | "yes" | "no" | "on" | "off" | "true" | "false" | "y" | "n"
    ) {
        return true;
    }
    let unsigned = lower
        .strip_prefix(['+', '-'])
        .unwrap_or(lower.as_str())
        .replace('_', "");
    if unsigned == ".inf" || unsigned == ".nan" {
        return true;
    }
    let digits = |s: &str, radix: u32| !s.is_empty() && s.chars().all(|c| c.is_digit(radix));
    if let Some(rest) = unsigned.strip_prefix("0x") {
        return digits(rest, 16);
    }
    if let Some(rest) = unsigned.strip_prefix("0o") {
        return digits(rest, 8);
    }
    if let Some(rest) = unsigned.strip_prefix("0b") {
        return digits(rest, 2);
    }
    // YAML 1.1 numbers may use `_` separators or sexagesimal `a:b` parts.
    let compact = unsigned.replace('_', "");
    if !compact.is_empty() && compact.parse::<f64>().is_ok() {
        return true;
    }
    compact.contains(':')
        && compact
            .split(':')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// `text` as a plain scalar when that reads back as itself, quoted when not.
fn quote(text: &str) -> String {
    let plain = !text.is_empty()
        && text.trim() == text
        && !text.starts_with([
            '"', '\'', '[', '{', '|', '>', '-', '#', '&', '*', '!', '%', '@', '`', '?',
        ])
        && !text.contains(": ")
        && !text.contains(" #")
        && !text.contains(['\n', '\r', '\t'])
        && !text.ends_with(':')
        && !yaml_reserved(text)
        && matches!(scalar(text), Value::Text(ref kept) if kept == text);
    if plain {
        text.to_owned()
    } else {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{escaped}\"")
    }
}

/// `key` as a YAML key that [`split_key`] reads back as itself.
fn quote_key(key: &str) -> String {
    let plain = !key.is_empty()
        && key.trim() == key
        && !key.starts_with([
            '"', '\'', '#', '&', '*', '!', '%', '@', '`', '-', '?', '[', '{', '|', '>',
        ])
        && !key.contains(['\n', '\r', '\t'])
        && !key.contains(": ")
        && !key.contains(" #")
        && !key.ends_with(':');
    if plain {
        key.to_owned()
    } else {
        let escaped = key.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use saphyr::LoadableYamlNode as _;

    #[test]
    fn reads_the_yaml_notes_use() {
        let body = "---
client: Acme
rating: 4
done: true
due: 2026-11-01
people: [Ana, Ben]
empty:
quoted: \"Plan: v2\"
tags:
  - work
  - '#project/acme'
nested:
  a: 1
  b: 2
---
# Note
";
        let properties = parse(body);
        assert_eq!(
            properties,
            vec![
                ("client".to_owned(), Value::Text("Acme".into())),
                ("rating".to_owned(), Value::Number(4.)),
                ("done".to_owned(), Value::Bool(true)),
                ("due".to_owned(), Value::Date("2026-11-01".into())),
                (
                    "people".to_owned(),
                    Value::List(vec![Value::Text("Ana".into()), Value::Text("Ben".into()),])
                ),
                ("empty".to_owned(), Value::Text(String::new())),
                ("quoted".to_owned(), Value::Text("Plan: v2".into())),
                (
                    "tags".to_owned(),
                    Value::List(vec![
                        Value::Text("work".into()),
                        Value::Text("#project/acme".into()),
                    ])
                ),
                ("nested".to_owned(), Value::Raw("  a: 1\n  b: 2\n".into())),
            ]
        );
    }

    #[test]
    fn closing_with_three_dots_and_crlf() {
        assert_eq!(
            parse("---\r\na: 1\r\n...\r\nBody"),
            vec![("a".to_owned(), Value::Number(1.))]
        );
        assert!(parse("---\nno close").is_empty());
        assert!(parse("no front matter").is_empty());
    }

    #[test]
    fn a_bom_and_trailing_spaces_still_open_front_matter() {
        let body = "\u{feff}--- \na: 1\n---  \nBody";
        assert_eq!(parse(body), vec![("a".to_owned(), Value::Number(1.))]);
        assert_eq!(
            set_property(body, "a", &Value::Number(2.)),
            "\u{feff}--- \na: 2\n---  \nBody"
        );
        let added = set_property(body, "b", &Value::Number(3.));
        assert_eq!(added, "\u{feff}--- \na: 1\nb: 3\n---  \nBody");
    }

    #[test]
    fn new_front_matter_uses_the_bodys_line_ending() {
        assert_eq!(
            set_property("# Note\r\n", "b", &Value::Number(2.)),
            "---\r\nb: 2\r\n---\r\n# Note\r\n"
        );
        assert_eq!(
            set_property("\u{feff}# Note\r\n", "b", &Value::Number(2.)),
            "\u{feff}---\r\nb: 2\r\n---\r\n# Note\r\n"
        );
    }

    #[test]
    fn edits_inside_crlf_front_matter_keep_crlf() {
        let body = "---\r\na: 1\r\nb: 2\r\n---\r\nBody\r\n";
        assert_eq!(
            set_property(body, "a", &Value::Number(3.)),
            "---\r\na: 3\r\nb: 2\r\n---\r\nBody\r\n"
        );
        assert_eq!(
            set_property(body, "c", &Value::Number(3.)),
            "---\r\na: 1\r\nb: 2\r\nc: 3\r\n---\r\nBody\r\n"
        );
    }

    #[test]
    fn set_changes_one_value_and_nothing_else() {
        let body = "---\nclient: Acme  # the one\nrating: 4\n---\n# Note\n\nBody.\n";
        let changed = set_property(body, "client", &Value::Text("Globex".into()));
        assert_eq!(
            changed,
            "---\nclient: Globex  # the one\nrating: 4\n---\n# Note\n\nBody.\n"
        );
        let added = set_property(&changed, "due", &Value::Date("2026-11-01".into()));
        assert_eq!(
            added,
            "---\nclient: Globex  # the one\nrating: 4\ndue: 2026-11-01\n---\n# Note\n\nBody.\n"
        );
    }

    #[test]
    fn comments_and_blank_lines_between_entries_are_kept() {
        let body =
            "---\ntitle: X\n# meta below\nrating: 4\n\nclient: Acme # keep\n\n# tail\n---\nBody";
        assert_eq!(
            set_property(body, "title", &Value::Text("Y".into())),
            "---\ntitle: \"Y\"\n# meta below\nrating: 4\n\nclient: Acme # keep\n\n# tail\n---\nBody"
        );
        assert_eq!(
            set_property(body, "rating", &Value::Number(5.)),
            "---\ntitle: X\n# meta below\nrating: 5\n\nclient: Acme # keep\n\n# tail\n---\nBody"
        );
        assert_eq!(
            set_property(body, "client", &Value::Text("Globex".into())),
            "---\ntitle: X\n# meta below\nrating: 4\n\nclient: Globex # keep\n\n# tail\n---\nBody"
        );
    }

    #[test]
    fn prose_between_rules_is_not_front_matter() {
        let body = "---\n# Heading\nSome prose.\n---\nMore";
        assert_eq!(split(body), None);
        assert!(parse(body).is_empty());
        assert_eq!(
            set_property(body, "a", &Value::Number(1.)),
            "---\na: 1\n---\n---\n# Heading\nSome prose.\n---\nMore"
        );
    }

    #[test]
    fn a_top_level_line_that_is_not_a_key_ends_the_front_matter() {
        let body = "---\na: 1\nnot a key\n  indented after it\nb: 2\n---\n";
        assert_eq!(split(body), None);
        assert!(parse(body).is_empty());
    }

    #[test]
    fn an_entry_ends_at_its_last_continuation_line() {
        let body = "---\na: one\n  two\n# between\nb: 2\n---\n";
        assert_eq!(
            set_property(body, "a", &Value::Text("X".into())),
            "---\na: X\n# between\nb: 2\n---\n"
        );
    }

    #[test]
    fn a_leading_dash_key_is_not_a_list_item() {
        let body = "---\ntags:\n  - x\n-k: v\n---\n";
        assert_eq!(
            parse(body),
            vec![
                (
                    "tags".to_owned(),
                    Value::List(vec![Value::Text("x".into())])
                ),
                ("-k".to_owned(), Value::Text("v".into())),
            ]
        );
        assert_eq!(
            set_property(body, "-k", &Value::Text("w".into())),
            "---\ntags:\n  - x\n-k: w\n---\n"
        );
        let added = set_property(body, "-added", &Value::Text("v".into()));
        assert_eq!(
            parse(&added),
            vec![
                (
                    "tags".to_owned(),
                    Value::List(vec![Value::Text("x".into())])
                ),
                ("-k".to_owned(), Value::Text("v".into())),
                ("-added".to_owned(), Value::Text("v".into())),
            ]
        );
    }

    #[test]
    fn set_property_at_edits_each_duplicate() {
        let body = "---\na: 1\na: 2\n---\n";
        assert_eq!(
            set_property_at(body, 0, &Value::Number(3.)),
            "---\na: 3\na: 2\n---\n"
        );
        assert_eq!(
            set_property_at(body, 1, &Value::Number(4.)),
            "---\na: 1\na: 4\n---\n"
        );
        assert_eq!(set_property_at(body, 9, &Value::Number(9.)), body);
    }

    #[test]
    fn comments_around_a_value_survive_editing() {
        let block = "---\ntags:\n  - a\n  # why b\n  - b\n---\nBody\n";
        let changed = set_property(
            block,
            "tags",
            &Value::List(vec![Value::Text("a".into()), Value::Text("c".into())]),
        );
        assert!(changed.contains("tags: [a, c]\n"), "{changed}");
        assert!(changed.contains("  # why b\n"), "{changed}");

        let key_line = "---\ntags: # mine\n  - a\n  - b\n---\n";
        let changed = set_property(
            key_line,
            "tags",
            &Value::List(vec![Value::Text("a".into()), Value::Text("d".into())]),
        );
        assert!(changed.contains("tags: [a, d] # mine\n"), "{changed}");

        let quoted = "---\ntitle: \"A: b\" # keep me\n---\n";
        let changed = set_property(quoted, "title", &Value::Text("C: d".into()));
        assert_eq!(changed, "---\ntitle: \"C: d\" # keep me\n---\n");
    }

    #[test]
    fn lists_holding_structures_are_raw() {
        for item in [
            "  - name: A\n",
            "  - &x a\n",
            "  - *x\n",
            "  - [a, b]\n",
            "  - - nested\n",
            "  - !tag a\n",
        ] {
            let body = format!("---\nitems:\n{item}---\n");
            let (_, value) = parse(&body).into_iter().next().expect(item);
            assert!(value.is_raw(), "{item}: {value:?}");
        }
        let body = "---\nitems:\n  - a\n  - b\n---\n";
        assert_eq!(
            parse(body),
            vec![(
                "items".to_owned(),
                Value::List(vec![Value::Text("a".into()), Value::Text("b".into())])
            )]
        );
    }

    #[test]
    fn yaml_keywords_and_number_forms_are_quoted_as_text() {
        for text in [
            "null", "~", "yes", "No", "on", "off", "0x10", "0o7", "0b101", ".inf", ".NaN", "+.Inf",
            "1_000", "1:20",
        ] {
            let body = set_property("", "key", &Value::Text(text.into()));
            assert_eq!(
                parse(&body),
                vec![("key".to_owned(), Value::Text(text.into()))],
                "{text}: {body}"
            );
            assert!(body.contains('"'), "{text}: {body}");
        }
    }

    #[test]
    fn duplicate_keys_set_the_one_get_reads() {
        let body = "---\na: 1\na: 2\n---\n";
        assert_eq!(get(&parse(body), "a"), Some(&Value::Number(2.)));
        assert_eq!(
            set_property(body, "a", &Value::Number(3.)),
            "---\na: 1\na: 3\n---\n"
        );
        assert_eq!(
            get(&parse(&set_property(body, "a", &Value::Number(3.))), "a"),
            Some(&Value::Number(3.))
        );
    }

    #[test]
    fn set_adds_front_matter_when_there_is_none() {
        assert_eq!(
            set_property("# Note\n", "done", &Value::Bool(true)),
            "---\ndone: true\n---\n# Note\n"
        );
        assert_eq!(
            set_property("---\nno close", "done", &Value::Bool(true)),
            "---\ndone: true\n---\n---\nno close"
        );
    }

    #[test]
    fn setting_a_value_to_what_it_reads_leaves_the_text_alone() {
        for text in [
            "nested:\n  a: 1\n  b: 2\n",
            "block: |\n  one\n  two\n",
            "folded: >\n  one\n  two\n",
            "flow: {a: 1}\n",
            "anchor: &a x\n",
            "zip: 01234\n",
            "version: 1.10\n",
            "quoted: \"Plan: v2\" # c\n",
            "list:\n  - a\n  - b\n",
            "plain: a plain value\n",
        ] {
            let body = format!("---\n{text}---\nBody\n");
            for (key, value) in parse(&body) {
                assert_eq!(&set_property(&body, &key, &value), &body, "{key}");
            }
        }
    }

    #[test]
    fn text_that_looks_like_another_type_is_quoted() {
        for (text, value) in [
            ("4", Value::Text("4".into())),
            ("true", Value::Text("true".into())),
            ("2026-11-01", Value::Text("2026-11-01".into())),
            ("[a]", Value::Text("[a]".into())),
            ("", Value::Text(String::new())),
        ] {
            let body = set_property("", "key", &value);
            assert_eq!(parse(&body), vec![("key".to_owned(), value)], "{text}");
        }
    }

    #[test]
    fn typed_keeps_a_property_its_type() {
        assert_eq!(
            typed("5", Some(&Value::Text("4".into()))),
            Value::Text("5".into())
        );
        assert_eq!(typed("5", Some(&Value::Number(4.))), Value::Number(5.));
        assert_eq!(
            typed("a, b", Some(&Value::List(Vec::new()))),
            Value::List(vec![Value::Text("a".into()), Value::Text("b".into())])
        );
        assert_eq!(typed("4", None), Value::Number(4.));
        assert_eq!(typed("2026-11-01", None), Value::Date("2026-11-01".into()));
    }

    fn property_text() -> impl Strategy<Value = String> {
        "[A-Za-z0-9 _:/.-]{0,20}"
    }

    fn property_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            property_text().prop_map(Value::Text),
            (-10000i64..10000).prop_map(|n| Value::Number(n as f64)),
            any::<bool>().prop_map(Value::Bool),
            (2000i32..2100, 1u32..13, 1u32..29)
                .prop_map(|(y, m, d)| { Value::Date(format!("{y:04}-{m:02}-{d:02}")) }),
            prop::collection::vec(property_text().prop_map(Value::Text), 0..5)
                .prop_map(Value::List),
        ]
    }

    /// A front matter of keys unique ignoring case (a later `o` replaces an
    /// `O`), and a body after it.
    fn document() -> impl Strategy<Value = (Vec<(String, Value)>, String)> {
        (
            prop::collection::btree_map(
                "[a-z]([a-z0-9_ ]{0,10}[a-z0-9_])?",
                property_value(),
                0..8,
            ),
            "[^\\u{0}]{0,60}",
        )
            .prop_map(|(entries, body)| (entries.into_iter().collect(), body))
    }

    /// Weird keys: punctuation YAML cares about, whitespace, and Unicode.
    fn any_key() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-zA-Z][a-zA-Z0-9_ ]{0,10}",
            "[-?\\[\\]{|>#&*!%@`][a-zA-Z0-9 ]{0,6}",
            "[a-zA-Z0-9]+: [a-zA-Z0-9]+",
            "[a-zA-Z0-9]+ # [a-zA-Z0-9]+",
            " [a-z]{0,5}",
            "[a-z]{0,5} ",
            "\\PC{1,8}",
        ]
        .prop_map(|key| key.trim_end_matches(['\n', '\r']).to_owned())
        .prop_filter("not empty", |key| !key.trim().is_empty())
    }

    /// Lines an Obsidian user may have in front matter, crudely.
    fn front_line() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-z][a-z0-9_ ]{0,6}: [A-Za-z0-9 #:\\\\\"]{0,12}",
            Just("# a comment".to_owned()),
            Just(String::new()),
            Just("nested:".to_owned()),
            Just("  a: 1".to_owned()),
            Just("block: |".to_owned()),
            Just("  line".to_owned()),
            Just("  - item".to_owned()),
            Just("- item".to_owned()),
            Just("dup: 1".to_owned()),
            Just("dup: 2".to_owned()),
            Just("\"quoted key\": value".to_owned()),
            Just("weird \u{1f600}: \"v\\\\ \\\" #\"".to_owned()),
            Just("version: 1.10".to_owned()),
            Just("zip: 01234".to_owned()),
        ]
    }

    fn arbitrary_document() -> impl Strategy<Value = String> {
        (
            prop::collection::vec(front_line(), 0..12),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(|(lines, crlf, bom)| {
                let eol = if crlf { "\r\n" } else { "\n" };
                let mut text = String::new();
                if bom {
                    text.push('\u{feff}');
                }
                text.push_str("---");
                text.push_str(eol);
                for line in lines {
                    text.push_str(&line);
                    text.push_str(eol);
                }
                text.push_str("---");
                text.push_str(eol);
                text.push_str("# Body\n");
                text
            })
    }

    /// A document of one-line entries with unique keys, and where each
    /// entry starts and ends, computed independently of `entries()`.
    #[allow(clippy::type_complexity)]
    fn span_document() -> impl Strategy<Value = (String, Vec<(usize, String, usize, usize)>)> {
        prop::collection::vec(
            (property_text(), prop::option::of("[A-Za-z0-9 ]{0,6}")),
            1..6,
        )
        .prop_map(|entries| {
            let mut text = String::from("---\n");
            let mut spans = Vec::new();
            for (index, (value, comment)) in entries.into_iter().enumerate() {
                let key = format!("k{index}");
                let start = text.len();
                text.push_str(&key);
                text.push_str(": ");
                text.push_str(&value);
                if let Some(comment) = comment {
                    text.push_str(" # ");
                    text.push_str(&comment);
                }
                text.push('\n');
                spans.push((index, key, start, text.len()));
            }
            text.push_str("---\nBody\n");
            (text, spans)
        })
    }

    /// Front matter that is guaranteed invalid: a top-level prose line
    /// follows the properties.
    fn properties_then_prose() -> impl Strategy<Value = String> {
        (
            prop::collection::vec(
                prop_oneof![
                    Just("# comment".to_owned()),
                    Just("  - item".to_owned()),
                    "[a-z][a-z0-9]{0,5}: [a-z0-9 ]{0,6}".prop_map(|line| line),
                ],
                0..8,
            ),
            "[A-Za-z0-9 .,]{0,40}",
        )
            .prop_map(|(lines, tail)| {
                let mut text = String::from("---\n");
                for line in lines {
                    text.push_str(&line);
                    text.push('\n');
                }
                text.push_str("Some prose here.\n---\n");
                text.push_str(&tail);
                text
            })
    }

    /// Text mixing every escape `quote` writes, and characters that mean
    /// something to YAML or Markdown.
    fn any_text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                r"[ A-Za-z0-9:/#.\[\]{}]{1,8}".prop_map(|s| s),
                Just("\n".to_owned()),
                Just("\r".to_owned()),
                Just("\t".to_owned()),
                Just("\\".to_owned()),
                Just("\"".to_owned()),
            ],
            0..8,
        )
        .prop_map(|parts| parts.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Round trip: writing a property and reading it back gives the
        /// same value.
        #[test]
        fn prop_set_then_parse_round_trips(
            (entries, body) in document(),
            key in "[a-zA-Z]([a-zA-Z0-9_ ]{0,10}[a-zA-Z0-9_])?",
            value in property_value(),
        ) {
            let mut text = body;
            for (name, value) in &entries {
                text = set_property(&text, name, value);
            }
            let before = parse(&text);
            for (name, value) in &entries {
                prop_assert_eq!(get(&before, name), Some(value), "{}", name);
            }
            let text = set_property(&text, &key, &value);
            let after = parse(&text);
            prop_assert_eq!(get(&after, &key), Some(&value));
            // Every other property is unchanged.
            for (name, kept) in &entries {
                if !name.eq_ignore_ascii_case(key.trim()) {
                    prop_assert_eq!(get(&after, name), Some(kept), "{}", name);
                }
            }
        }

        /// A key of any shape round-trips through the text.
        #[test]
        fn prop_any_key_round_trips(key in any_key(), value in property_value()) {
            let body = set_property("", &key, &value);
            let parsed = parse(&body);
            prop_assert_eq!(get(&parsed, &key), Some(&value), "{}", key);
        }

        /// Setting a property never changes the text after the front matter.
        #[test]
        fn prop_set_keeps_the_body(
            front in "([a-z][a-z0-9]{0,6}: [A-Za-z0-9 .]{0,10}\n){0,5}",
            body in "[^\\u{0}]{0,80}",
            key in "[a-z][a-z0-9]{0,8}",
            value in property_value(),
        ) {
            let text = format!("---\n{front}---\n{body}");
            let changed = set_property(&text, &key, &value);
            let before = split(&text).map(|f| &text[f.body_start..]).unwrap_or("");
            let after = split(&changed).map(|f| &changed[f.body_start..]).unwrap_or("");
            prop_assert_eq!(before, after);
        }

        /// Setting a property changes only bytes inside the entry `get`
        /// reads; comments and blank lines around it stay put.
        #[test]
        fn prop_set_touches_only_the_entry_span(
            (text, spans) in span_document(),
            value in property_value(),
        ) {
            for (index, key, start, end) in spans {
                let changed = set_property_at(&text, index, &value);
                prop_assert_eq!(&changed[..start], &text[..start], "{}", key);
                prop_assert!(changed[start..].ends_with(&text[end..]), "{}", key);
                let parsed = parse(&changed);
                prop_assert_eq!(parsed.get(index).map(|(k, _)| k.as_str()), Some(key.as_str()));
                prop_assert_eq!(parsed.get(index).map(|(_, v)| v), Some(&value));
            }
        }

        /// Removing the property at an index takes exactly that entry's
        /// bytes: everything before and after stays, and the parse loses
        /// that one entry and keeps the others in order.
        #[test]
        fn prop_remove_takes_exactly_the_entry(
            (text, spans) in span_document(),
            pick in 0usize..8,
        ) {
            let index = pick % spans.len();
            let (_, key, start, end) = &spans[index];
            let removed = remove_property_at(&text, index);
            prop_assert_eq!(&removed[..*start], &text[..*start], "{}", key);
            prop_assert_eq!(&removed[*start..], &text[*end..], "{}", key);
            let mut expected = parse(&text);
            expected.remove(index);
            prop_assert_eq!(parse(&removed), expected);
            // Out of range, or with no front matter: nothing changes.
            prop_assert_eq!(remove_property_at(&text, spans.len()), text.clone());
        }

        /// Removing from any document whatever (block lists, block
        /// scalars, comments) loses exactly one entry and no other byte of
        /// the body after the front matter.
        #[test]
        fn prop_remove_loses_one_entry_of_any_document(text in arbitrary_document(), pick in 0usize..8) {
            let before = parse(&text);
            if before.is_empty() {
                prop_assert_eq!(remove_property_at(&text, pick), text);
                return Ok(());
            }
            let index = pick % before.len();
            let removed = remove_property_at(&text, index);
            let mut expected = before;
            expected.remove(index);
            prop_assert_eq!(parse(&removed), expected);
            prop_assert!(removed.len() < text.len());
            let tail = |t: &str| split(t).map(|f| t[f.body_start..].to_owned());
            prop_assert_eq!(tail(&removed), tail(&text));
        }

        /// Setting a key to the value it already reads leaves every byte as
        /// it was, including number lexemes and block scalars.
        #[test]
        fn prop_set_own_value_is_identity(text in arbitrary_document()) {
            let Some(front) = split(&text) else {
                return Ok(());
            };
            let parsed = parse_front(front.front);
            let mut seen = std::collections::HashSet::new();
            for (key, _) in &parsed {
                if !seen.insert(key.to_lowercase()) {
                    continue;
                }
                let value = get(&parsed, key).expect("the key was just read");
                prop_assert_eq!(set_property(&text, key, value), text.clone(), "{}", key);
            }
        }

        /// Any text parses without panicking, and so does setting in it.
        #[test]
        fn prop_any_text_parses(text in "[\\s\\S]{0,200}") {
            let _ = parse(&text);
            let _ = set_property(&text, "key", &Value::Text("v".into()));
        }

        /// A block of properties with prose after it is not front matter:
        /// the prose is never hidden, edited or exported as properties.
        #[test]
        fn prop_prose_after_rules_is_not_front_matter(text in properties_then_prose()) {
            prop_assert!(split(&text).is_none(), "{text:?}");
            prop_assert!(parse(&text).is_empty());
        }

        /// Every escape `quote` writes reads back as the text it was given.
        #[test]
        fn prop_text_round_trips_through_quotes(text in any_text()) {
            let body = set_property("", "key", &Value::Text(text.clone()));
            let parsed = parse(&body);
            prop_assert_eq!(get(&parsed, "key"), Some(&Value::Text(text.clone())), "{:?}", body);
        }
    }

    #[test]
    fn unsupported_values_are_raw() {
        for text in [
            "nested:\n  a: 1\n",
            "block: |\n  one\n  two\n",
            "folded: >\n  one\n",
            "flow: {a: 1, b: 2}\n",
            "anchor: &a x\n",
            "alias: *a\n",
            "tag: !thing x\n",
            "open: [a, b\n",
        ] {
            let body = format!("---\n{text}---\n");
            let (_, value) = parse(&body).into_iter().next().expect(text);
            assert!(value.is_raw(), "{text}: {value:?}");
        }
    }

    fn yaml_matches(found: &saphyr::Yaml<'_>, value: &Value) -> bool {
        match value {
            Value::Text(text) | Value::Date(text) => found.as_str() == Some(text),
            Value::Number(number) => {
                found.as_integer() == Some(*number as i64)
                    || (number.fract() != 0.0
                        && found.as_str().and_then(|s| s.parse::<f64>().ok()) == Some(*number))
            }
            Value::Bool(on) => found.as_bool() == Some(*on),
            Value::List(items) => match found {
                saphyr::Yaml::Sequence(sequence) => {
                    sequence.len() == items.len()
                        && sequence
                            .iter()
                            .zip(items)
                            .all(|(found, item)| yaml_matches(found, item))
                }
                _ => false,
            },
            Value::Raw(raw) => found.as_str() == Some(raw),
        }
    }

    fn yaml_lookup<'a>(doc: &'a saphyr::Yaml<'a>, key: &str) -> Option<&'a saphyr::Yaml<'a>> {
        match doc {
            saphyr::Yaml::Mapping(map) => map
                .iter()
                .find(|(name, _)| name.as_str() == Some(key))
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// The supported subset reads the same in Kanso and in a YAML parser.
    #[test]
    fn the_supported_subset_reads_like_yaml() {
        let entries = vec![
            ("client", Value::Text("Acme".into())),
            ("rating", Value::Number(4.)),
            ("done", Value::Bool(true)),
            ("due", Value::Date("2026-11-01".into())),
            (
                "people",
                Value::List(vec![Value::Text("Ana".into()), Value::Text("Ben".into())]),
            ),
            ("quoted", Value::Text("Plan: v2".into())),
            ("unicode", Value::Text("caf\u{e9} \u{1f600}".into())),
        ];
        let mut text = String::new();
        for (key, value) in &entries {
            text.push_str(key);
            text.push_str(": ");
            text.push_str(&render(value));
            text.push('\n');
        }
        let docs = saphyr::Yaml::load_from_str(&text).expect("valid YAML");
        let yaml = docs.first().expect("one document");
        for (key, value) in &entries {
            let found = yaml_lookup(yaml, key).unwrap_or_else(|| panic!("{key} missing"));
            assert!(yaml_matches(found, value), "{key}: {found:?}");
        }
        let expected: Vec<(String, Value)> = entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect();
        assert_eq!(parse_front(&text), expected);
    }

    proptest! {
        /// On the supported subset both parsers read the same values.
        #[test]
        fn prop_supported_values_match_yaml(
            entries in prop::collection::btree_map(
                "[a-z][a-z0-9_]{0,8}",
                prop_oneof![
                    property_text().prop_map(Value::Text),
                    (-10_000i64..10_000).prop_map(|n| Value::Number(n as f64)),
                    any::<bool>().prop_map(Value::Bool),
                ],
                1..6,
            ),
        ) {
            let mut text = String::new();
            for (key, value) in &entries {
                text.push_str(key);
                text.push_str(": ");
                text.push_str(&render(value));
                text.push('\n');
            }
            let docs = saphyr::Yaml::load_from_str(&text)
                .map_err(|err| TestCaseError::fail(format!("rejected {text:?}: {err}")))?;
            let yaml = docs.first().ok_or_else(|| TestCaseError::fail("no document"))?;
            for (key, value) in &entries {
                let found = yaml_lookup(yaml, key)
                    .ok_or_else(|| TestCaseError::fail(format!("{key} missing from {text:?}")))?;
                prop_assert!(yaml_matches(found, value), "{}: {:?}", key, found);
            }
            prop_assert_eq!(parse_front(&text), entries.into_iter().collect::<Vec<_>>());
        }
    }
}
