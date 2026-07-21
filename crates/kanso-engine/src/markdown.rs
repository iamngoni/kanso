//! Markdown indexing helpers.
//!
//! CommonMark/GFM structure (headings, links) comes from `comrak`'s AST —
//! spec compliance is the portability promise, so we don't hand-roll it.
//! Kanso-specific reference syntax (`[[note]]`, `![[sketch:id]]`,
//! `![[attachment:name]]`) and task items are scanned directly, which is robust
//! across comrak versions and avoids depending on extension-specific AST nodes.

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, markdown_to_html, parse_document};

#[derive(Debug, Default)]
pub struct Extracted {
    pub headings: Vec<String>,
    pub links: Vec<String>,
    pub refs: Vec<Reference>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub kind: RefKind,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Note,
    Sketch,
    Attachment,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub line_index: usize,
    pub text: String,
    pub checked: bool,
}

/// Parse a Markdown body and pull out everything the engine indexes.
pub fn extract(markdown: &str) -> Extracted {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, markdown, &options);

    let mut headings = Vec::new();
    let mut links = Vec::new();

    for node in root.descendants() {
        let value = &node.data.borrow().value;
        match value {
            NodeValue::Heading(_) => {
                let mut text = String::new();
                for child in node.descendants() {
                    if let NodeValue::Text(t) = &child.data.borrow().value {
                        text.push_str(t);
                    }
                }
                if !text.is_empty() {
                    headings.push(text);
                }
            }
            NodeValue::Link(link) => links.push(link.url.clone()),
            _ => {}
        }
    }

    Extracted {
        headings,
        links,
        refs: extract_refs(markdown),
        tasks: extract_tasks(markdown),
    }
}

/// Render Markdown for native preview surfaces.
///
/// This intentionally lives in the engine, not the Swift layer, so Kanso's
/// CommonMark/GFM behavior and `[[...]]` reference syntax have one source of
/// truth across preview, export, search-derived references, and future clients.
#[allow(dead_code)]
pub fn render_html(markdown: &str) -> String {
    render_html_with_reference_blocks(markdown, |_, _| None)
}

pub(crate) fn render_html_with_reference_blocks<F>(markdown: &str, mut render_block: F) -> String
where
    F: FnMut(RefKind, &str) -> Option<String>,
{
    let mut html = String::new();
    let mut paragraph = String::new();

    for line in markdown.lines() {
        if let Some((kind, target)) = reference_block(line) {
            flush_markdown_chunk(&mut paragraph, &mut html);
            let block = render_block(kind, &target)
                .unwrap_or_else(|| default_reference_block(kind, &target));
            html.push_str(&block);
            html.push('\n');
        } else {
            paragraph.push_str(&rewrite_inline_references(line));
            paragraph.push('\n');
        }
    }

    flush_markdown_chunk(&mut paragraph, &mut html);
    html
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.header_id_prefix = Some("kanso-heading-".to_string());
    options.render.escape = true;
    options
}

fn flush_markdown_chunk(markdown: &mut String, html: &mut String) {
    if markdown.trim().is_empty() {
        markdown.clear();
        return;
    }
    html.push_str(&markdown_to_html(markdown, &markdown_options()));
    markdown.clear();
}

fn reference_block(line: &str) -> Option<(RefKind, String)> {
    let trimmed = line.trim();
    if !(trimmed.starts_with("![[") && trimmed.ends_with("]]")) {
        return None;
    }

    let inner = trimmed.strip_prefix("![[")?.strip_suffix("]]")?.trim();
    if let Some(sketch) = inner
        .strip_prefix("sketch:")
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some((RefKind::Sketch, sketch.to_string()));
    }

    if let Some(attachment) = inner
        .strip_prefix("attachment:")
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some((RefKind::Attachment, attachment.to_string()));
    }

    None
}

fn default_reference_block(kind: RefKind, target: &str) -> String {
    match kind {
        RefKind::Sketch => format!(
            "<a class=\"kanso-block-link\" href=\"kanso://sketch/{}\">\
             <figure class=\"kanso-block kanso-sketch\" data-kanso-kind=\"sketch\" data-kanso-target=\"{}\">\
             <div class=\"kanso-block-icon\">Sketch</div>\
             <figcaption>{}</figcaption>\
             </figure>\
             </a>",
            percent_encode(target),
            escape_attr(target),
            escape_html(target)
        ),
        RefKind::Attachment => format!(
            "<a class=\"kanso-block-link\" href=\"kanso://attachment/{}\">\
             <figure class=\"kanso-block kanso-attachment\" data-kanso-kind=\"attachment\" data-kanso-target=\"{}\">\
             <div class=\"kanso-block-icon\">Attachment</div>\
             <figcaption>{}</figcaption>\
             </figure>\
             </a>",
            percent_encode(target),
            escape_attr(target),
            escape_html(target)
        ),
        RefKind::Note => String::new(),
    }
}

fn rewrite_inline_references(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;

    while i + 1 < bytes.len() {
        let embed =
            bytes[i] == b'!' && i + 2 < bytes.len() && bytes[i + 1] == b'[' && bytes[i + 2] == b'[';
        let wiki = bytes[i] == b'[' && bytes[i + 1] == b'[';
        if embed || wiki {
            let start = if embed { i + 3 } else { i + 2 };
            if let Some(rel_end) = line[start..].find("]]") {
                let inner = line[start..start + rel_end].trim();
                if !inner.is_empty() {
                    let (label, url) = if embed {
                        if let Some(target) = inner.strip_prefix("sketch:").map(str::trim) {
                            (
                                format!("Sketch: {target}"),
                                format!("kanso://sketch/{}", percent_encode(target)),
                            )
                        } else if let Some(target) =
                            inner.strip_prefix("attachment:").map(str::trim)
                        {
                            (
                                format!("Attachment: {target}"),
                                format!("kanso://attachment/{}", percent_encode(target)),
                            )
                        } else {
                            (
                                inner.to_string(),
                                format!("kanso://embed/{}", percent_encode(inner)),
                            )
                        }
                    } else if let Some((target, label)) = split_wiki_note(inner) {
                        (label, format!("kanso://note/{}", percent_encode(&target)))
                    } else {
                        (
                            inner.to_string(),
                            format!("kanso://note/{}", percent_encode(inner)),
                        )
                    };
                    out.push('[');
                    out.push_str(&escape_markdown_link_text(&label));
                    out.push_str("](");
                    out.push_str(&url);
                    out.push(')');
                    i = start + rel_end + 2;
                    continue;
                }
            }
        }

        out.push(line[i..].chars().next().expect("valid utf-8"));
        i += line[i..].chars().next().expect("valid utf-8").len_utf8();
    }

    if i < line.len() {
        out.push_str(&line[i..]);
    }
    out
}

fn escape_markdown_link_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub(crate) fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn split_wiki_note(inner: &str) -> Option<(String, String)> {
    let mut parts = inner.splitn(2, '|');
    let target = parts.next()?.trim();
    if target.is_empty() {
        return None;
    }

    let label = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(target);

    Some((target.to_string(), label.to_string()))
}

pub(crate) fn escape_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Scan `[[...]]` / `![[...]]` references.
fn extract_refs(markdown: &str) -> Vec<Reference> {
    let bytes = markdown.as_bytes();
    let mut refs = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            if let Some(rel_end) = markdown[i + 2..].find("]]") {
                let inner = &markdown[i + 2..i + 2 + rel_end];
                let reference = if let Some(rest) = inner.strip_prefix("sketch:") {
                    Reference {
                        kind: RefKind::Sketch,
                        target: rest.trim().to_string(),
                    }
                } else if let Some(rest) = inner.strip_prefix("attachment:") {
                    Reference {
                        kind: RefKind::Attachment,
                        target: rest.trim().to_string(),
                    }
                } else if let Some((target, _)) = split_wiki_note(inner) {
                    Reference {
                        kind: RefKind::Note,
                        target,
                    }
                } else {
                    i = i + 2 + rel_end + 2;
                    continue;
                };
                if !reference.target.is_empty() {
                    refs.push(reference);
                }
                i = i + 2 + rel_end + 2;
                continue;
            }
        }
        i += 1;
    }
    refs
}

/// Scan GFM-style task list items.
fn extract_tasks(markdown: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    for (line_index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim_start();
        let checked = if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            Some(true)
        } else if trimmed.starts_with("- [ ]") {
            Some(false)
        } else {
            None
        };
        if let Some(checked) = checked {
            let text = trimmed[5..].trim().to_string();
            if !text.is_empty() {
                tasks.push(Task {
                    line_index,
                    text,
                    checked,
                });
            }
        }
    }
    tasks
}
