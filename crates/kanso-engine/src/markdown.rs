//! Markdown indexing helpers.
//!
//! CommonMark/GFM structure (headings, links) comes from `comrak`'s AST —
//! spec compliance is the portability promise, so we don't hand-roll it.
//! Kanso-specific reference syntax (`[[note]]`, `![[sketch:id]]`,
//! `![[attachment:name]]`) and task items are scanned directly, which is robust
//! across comrak versions and avoids depending on extension-specific AST nodes.

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};

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
                    Reference { kind: RefKind::Sketch, target: rest.trim().to_string() }
                } else if let Some(rest) = inner.strip_prefix("attachment:") {
                    Reference { kind: RefKind::Attachment, target: rest.trim().to_string() }
                } else {
                    Reference { kind: RefKind::Note, target: inner.trim().to_string() }
                };
                refs.push(reference);
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
    for line in markdown.lines() {
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
                tasks.push(Task { text, checked });
            }
        }
    }
    tasks
}
