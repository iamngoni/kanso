//! Imports local HTML documents as Markdown, preserving their tree paths.

use std::{collections::HashMap, io, path::Path};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use super::links::as_relative_path;
use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

/// The deepest element nesting handed to `htmd`, which recurses once per
/// level: deeper HTML could overflow an import thread's stack.
pub(super) const MAX_DEPTH: usize = 256;

/// What a rewritten local path encodes, so `htmd` writes it as one Markdown
/// link destination (it leaves spaces as they are and escapes parentheses).
pub(super) const PATH: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'\'')
    .add(b'%')
    .add(b'(')
    .add(b')')
    .add(b'<')
    .add(b'>')
    .add(b'\\');

pub struct HtmlSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
    targets: HashMap<String, String>,
}

/// Opens one HTML document, a local folder, or a ZIP archive.
pub fn open(path: &Path) -> anyhow::Result<HtmlSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else if matches!(extension(&path.to_string_lossy()).as_str(), "html" | "htm") {
        Tree::Single(path.to_owned())
    } else {
        Tree::zip(path)?
    };
    let listed = tree.files()?;
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for file in listed {
        if matches!(extension(&file.path).as_str(), "html" | "htm") {
            if file.size > MAX_NOTE {
                skipped.push(Skipped::new(file.path, "too large for an HTML note"));
            } else {
                notes.push(file);
            }
        } else {
            files.push(file);
        }
    }
    let targets = notes
        .iter()
        .filter_map(|f| normalize(&f.path).map(|path| (path, f.path.clone())))
        .collect();
    Ok(HtmlSource {
        tree,
        notes,
        files,
        skipped,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("HTML")
            .to_owned(),
        targets,
    })
}

impl Source for HtmlSource {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn files(&self) -> &[TreeFile] {
        &self.files
    }
    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for file in self.notes.clone() {
            let html = self
                .tree
                .read_text(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let rewritten = rewrite_doc_links(&html, &file.path, &self.targets);
            let cleaned = remove_blocks(&rewritten, &["script", "style", "noscript"]);
            let Some(body) = to_markdown(&cleaned) else {
                if !self.skipped.iter().any(|s| s.path == file.path) {
                    self.skipped
                        .push(Skipped::new(file.path, "HTML nested too deeply to convert"));
                }
                continue;
            };
            let title = html_title(&html)
                .unwrap_or_else(|| stem(file_name(&file.path)).replace(['_', '-'], " "));
            each(SourceNote {
                key: file.path.clone(),
                path: Some(file.path.clone()),
                folder: parent_of(&file.path)
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
                title,
                body: body.trim().to_owned(),
                ..SourceNote::default()
            })?;
        }
        Ok(())
    }
    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        self.tree.read(path, limit)
    }
    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// `html` as Markdown, or `None` when it nests deeper than [`MAX_DEPTH`]
/// (or `htmd` fails).
pub(super) fn to_markdown(html: &str) -> Option<String> {
    if nesting_depth(html) > MAX_DEPTH {
        return None;
    }
    htmd::convert(html).ok()
}

/// How deeply `html`'s elements nest, from its tags alone. Void elements
/// don't nest, and the end tags HTML lets a page leave out (`<p>`, `<li>`,
/// table cells) are inferred, so real pages are rarely overcounted.
pub(super) fn nesting_depth(html: &str) -> usize {
    const VOID: &[&str] = &[
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];
    // An open one of these ends at the next of the same name...
    const PEERS: &[&str] = &["li", "dt", "dd", "option", "tr", "td", "th"];
    // ...within the same one of these.
    const SCOPES: &[&str] = &["ul", "ol", "dl", "select", "table"];
    // These end an open `<p>`.
    const BLOCKS: &[&str] = &[
        "p",
        "div",
        "ul",
        "ol",
        "dl",
        "table",
        "blockquote",
        "pre",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "section",
        "article",
        "header",
        "footer",
        "nav",
        "aside",
        "form",
        "figure",
        "li",
        "dt",
        "dd",
    ];
    let mut stack: Vec<String> = Vec::new();
    let mut deepest = 0;
    let mut cursor = 0;
    while let Some(rel) = html[cursor..].find('<') {
        let rest = &html[cursor + rel + 1..];
        let until = if rest.starts_with("!--") { "-->" } else { ">" };
        let Some(end) = rest.find(until) else {
            break;
        };
        cursor += rel + 1 + end + until.len();
        let tag = &rest[..end];
        let closing = tag.starts_with('/');
        let name = tag
            .trim_start_matches('/')
            .split(|c: char| !c.is_ascii_alphanumeric())
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if closing {
            if let Some(at) = stack.iter().rposition(|open| *open == name) {
                stack.truncate(at);
            }
            continue;
        }
        if VOID.contains(&name.as_str()) || tag.ends_with('/') {
            continue;
        }
        if BLOCKS.contains(&name.as_str()) && stack.last().is_some_and(|open| open == "p") {
            stack.pop();
        }
        if PEERS.contains(&name.as_str())
            && let Some(at) = stack
                .iter()
                .rposition(|open| *open == name || SCOPES.contains(&open.as_str()))
                .filter(|&at| stack[at] == name)
        {
            stack.truncate(at);
        }
        stack.push(name);
        deepest = deepest.max(stack.len());
    }
    deepest
}

pub(super) fn html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    for tag in ["title", "h1"] {
        let Some(start) = lower.find(&format!("<{tag}")) else {
            continue;
        };
        let Some(gt) = lower[start..].find('>').map(|offset| offset + start + 1) else {
            continue;
        };
        let Some(end) = lower[gt..]
            .find(&format!("</{tag}"))
            .map(|offset| offset + gt)
        else {
            continue;
        };
        let value = plain_text(&html[gt..end]);
        if !value.trim().is_empty() {
            return Some(value.trim().to_owned());
        }
    }
    None
}

/// `html` with each local `href` and `src` `%`-encoded, and a link to
/// another document in the import as `kanso-import:note/<its path>`.
pub(super) fn rewrite_doc_links(
    html: &str,
    path: &str,
    targets: &HashMap<String, String>,
) -> String {
    let html = rewrite_attribute(html, "href=", path, targets);
    rewrite_attribute(&html, "src=", path, targets)
}

fn rewrite_attribute(
    html: &str,
    attribute: &str,
    path: &str,
    targets: &HashMap<String, String>,
) -> String {
    let lower = html.to_ascii_lowercase();
    let mut result = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find(attribute) {
        let at = cursor + rel;
        let value_at = at + attribute.len();
        result.push_str(&html[cursor..value_at]);
        cursor = value_at;
        // Not the end of another attribute's name (`data-src=`).
        if !html[..at].ends_with(|c: char| c.is_ascii_whitespace()) {
            continue;
        }
        let quote = html.as_bytes().get(value_at).copied().unwrap_or(b' ');
        if quote != b'\'' && quote != b'"' {
            continue;
        }
        let value_start = value_at + 1;
        let Some(end_rel) = html[value_start..].find(char::from(quote)) else {
            result.push_str(&html[value_at..]);
            return result;
        };
        let end = value_start + end_rel;
        let raw = &html[value_start..end];
        result.push(char::from(quote));
        match as_relative_path(raw).filter(|local| !local.starts_with('/')) {
            Some(local) => {
                let dest = normalize(&format!("{}/{}", parent_of(path), local));
                if let Some(target) = dest.as_ref().and_then(|dest| targets.get(dest)) {
                    result.push_str("kanso-import:note/");
                    result.extend(utf8_percent_encode(target, PATH));
                } else {
                    result.extend(utf8_percent_encode(&local, PATH));
                    result.push_str(&raw[raw.find(['?', '#']).unwrap_or(raw.len())..]);
                }
            }
            None => result.push_str(raw),
        }
        result.push(char::from(quote));
        cursor = end + 1;
    }
    result.push_str(&html[cursor..]);
    result
}

pub(super) fn normalize(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    let normalized = path.replace('\\', "/");
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// Drops each `<tag …>…</tag>` block in one pass over `input`.
pub(super) fn remove_blocks(input: &str, tags: &[&str]) -> String {
    let lower = input.to_ascii_lowercase();
    let mut out = String::with_capacity(input.len());
    let (mut kept, mut at) = (0, 0);
    while let Some(rel) = lower[at..].find('<') {
        let start = at + rel;
        let Some(tag) = tags
            .iter()
            .find(|tag| lower[start + 1..].starts_with(**tag))
        else {
            at = start + 1;
            continue;
        };
        out.push_str(&input[kept..start]);
        let end = lower[start..].find(&format!("</{tag}")).and_then(|close| {
            let close = start + close;
            lower[close..].find('>').map(|gt| close + gt + 1)
        });
        let Some(end) = end else {
            kept = input.len();
            break;
        };
        (kept, at) = (end, end);
    }
    out.push_str(&input[kept..]);
    out
}

fn plain_text(value: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in value.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn links_local_pages_and_ignores_scripts() {
        let targets = HashMap::from([("dir/next.html".into(), "dir/next.html".into())]);
        let converted = rewrite_doc_links(
            "<a href='next.html'>Next</a><script>bad()</script>",
            "dir/start.html",
            &targets,
        );
        assert!(converted.contains("kanso-import:note/dir/next.html"));
        assert!(!remove_blocks(&converted, &["script"]).contains("bad()"));
    }
    #[test]
    fn removes_every_block_in_one_pass() {
        assert_eq!(
            remove_blocks(
                "a<SCRIPT>x</script >b<style>y</style>c<noscript>z",
                &["script", "style", "noscript"]
            ),
            "abc"
        );
        let many = "<p>keep</p><script>drop()</script>".repeat(50_000);
        assert_eq!(
            remove_blocks(&many, &["script"]),
            "<p>keep</p>".repeat(50_000)
        );
    }
    #[test]
    fn derives_title_and_normalizes_paths() {
        assert_eq!(
            html_title("<title> A &amp; B </title>"),
            Some("A & B".into())
        );
        assert_eq!(normalize("a/b/../c.html"), Some("a/c.html".into()));
        assert_eq!(normalize("../c.html"), None);
    }

    #[test]
    fn encodes_local_paths_so_they_stay_one_destination() {
        let targets = HashMap::from([("My Page (1).html".into(), "My Page (1).html".into())]);
        let html = "<a href=\"My%20Page%20(1).html#top\">p</a><img src='a b (2).png'>\
                    <img data-src='x y.png'><a href='https://e.com/a b'>w</a>";
        let converted = rewrite_doc_links(html, "start.html", &targets);
        assert!(converted.contains("\"kanso-import:note/My%20Page%20%281%29.html\""));
        assert!(converted.contains("'a%20b%20%282%29.png'"));
        assert!(converted.contains("data-src='x y.png'"));
        assert!(converted.contains("'https://e.com/a b'"));
        let markdown = to_markdown(&converted).unwrap();
        assert!(markdown.contains("(kanso-import:note/My%20Page%20%281%29.html)"));
        assert!(markdown.contains("(a%20b%20%282%29.png)"));
    }

    #[test]
    fn infers_omitted_end_tags_when_measuring_depth() {
        assert_eq!(nesting_depth("<p>a<br><img src=x><p>b<!-- <div> -->"), 1);
        // Each `<ul>` ends the `<p>` before it: ul li ul li ul li p.
        assert_eq!(nesting_depth(&"<ul><li>a<p>b".repeat(3)), 7);
        assert_eq!(
            nesting_depth(&format!("<ul>{}</ul>", "<li><p>x".repeat(500))),
            3
        );
        assert_eq!(
            nesting_depth(&format!("<table>{}</table>", "<tr><td>a<td>b".repeat(500))),
            3
        );
        assert_eq!(nesting_depth(&"<div>".repeat(300)), 300);
        assert_eq!(nesting_depth("<div><span>a</span></div><div/>"), 2);
    }

    #[test]
    fn converts_up_to_the_depth_limit_on_a_small_stack() {
        let nested = |depth: usize, tag: &str| {
            format!(
                "{}x{}",
                format!("<{tag}>").repeat(depth),
                format!("</{tag}>").repeat(depth)
            )
        };
        let lists = format!(
            "{}x{}",
            "<ul><li>".repeat(MAX_DEPTH / 2),
            "</li></ul>".repeat(MAX_DEPTH / 2)
        );
        let pages = [
            nested(MAX_DEPTH, "div"),
            nested(MAX_DEPTH, "blockquote"),
            nested(MAX_DEPTH, "em"),
            lists,
        ];
        // An import thread's default stack, in a debug build's larger frames.
        let converted = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || pages.iter().all(|page| to_markdown(page).is_some()))
            .unwrap()
            .join()
            .unwrap();
        assert!(converted);
        assert_eq!(to_markdown(&nested(MAX_DEPTH + 1, "div")), None);
        assert_eq!(to_markdown(&nested(100_000, "div")), None);
    }
}
