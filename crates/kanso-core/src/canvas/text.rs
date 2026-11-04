//! What a canvas says in words: the short Markdown summary the local API and
//! MCP list, and the text the search index reads instead of the JSON.

use super::model::{Canvas, InkOcr, Node};

/// The longest summary the API lists, in characters.
pub const SUMMARY_CAP: usize = 4_000;

impl Canvas {
    /// A short Markdown list of the canvas: text cards, note-card titles,
    /// links, group labels and recognized handwriting, in stacking order,
    /// capped at [`SUMMARY_CAP`] characters. Pure.
    pub fn summary(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for node in self.live_nodes() {
            let line = match node.kind.as_str() {
                "text" => node.text.as_deref().map(one_line),
                "file" => node
                    .title
                    .as_deref()
                    .or(node.file.as_deref())
                    .map(|t| format!("Card: {}", one_line(t))),
                "link" => node
                    .url
                    .as_deref()
                    .map(|u| format!("Link: {}", one_line(u))),
                "group" => node
                    .label
                    .as_deref()
                    .map(|l| format!("Group: {}", one_line(l))),
                _ => None,
            };
            if let Some(line) = line.filter(|l| !l.is_empty()) {
                lines.push(format!("- {line}"));
            }
            if let Some(text) = self.ocr_of(node) {
                lines.push(format!("- Handwriting: {}", one_line(text)));
            }
        }
        cap(lines.join("\n"), SUMMARY_CAP)
    }

    /// The words the search index reads for this canvas: every text card, every
    /// card's title and the recognized handwriting. The JSON is never
    /// indexed. Pure.
    pub fn search_text(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for node in self.live_nodes() {
            if node.kind == "text"
                && let Some(text) = node.text.as_deref()
            {
                parts.push(text);
            }
            if let Some(title) = node.title.as_deref() {
                parts.push(title);
            }
            if let Some(text) = self.ocr_of(node) {
                parts.push(text);
            }
        }
        parts.retain(|p| !p.trim().is_empty());
        parts.join("\n")
    }

    /// The live nodes, in stacking order.
    pub fn live_nodes(&self) -> impl Iterator<Item = &Node> {
        let mut nodes: Vec<&Node> = self.nodes.iter().filter(|n| !n.deleted).collect();
        nodes.sort_by(|a, b| {
            a.kanso_z
                .total_cmp(&b.kanso_z)
                .then_with(|| a.id.cmp(&b.id))
        });
        nodes.into_iter()
    }

    fn ocr_of(&self, node: &Node) -> Option<&str> {
        self.kanso_ink_ocr
            .iter()
            .find(|o: &&InkOcr| o.node == node.id)
            .map(|o| o.text.as_str())
            .filter(|t| !t.trim().is_empty())
    }
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cap(text: String, max: usize) -> String {
    if text.chars().count() <= max {
        return text;
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
