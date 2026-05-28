//! Markdown import/export — the portability ("no lock-in") path.
//!
//! Export writes one `.md` file per note with a small YAML frontmatter header
//! plus the verbatim body. Import reverses it, reading the title from
//! frontmatter (or the filename) and creating notes.

use crate::db::Engine;
use crate::error::Result;
use crate::models::{ExportFile, ImportFile};

impl Engine {
    /// Export every (non-deleted) note in a notebook to Markdown files.
    pub async fn export_notebook_markdown(&self, notebook_id: &str) -> Result<Vec<ExportFile>> {
        let notes = self.list_notes(notebook_id).await?;
        Ok(notes
            .into_iter()
            .map(|note| ExportFile {
                path: format!("{}.md", sanitize_filename(&note.title)),
                content: format!(
                    "---\ntitle: {}\ncreated: {}\nupdated: {}\n---\n\n{}",
                    note.title, note.created_at, note.updated_at, note.body_markdown
                ),
            })
            .collect())
    }

    /// Import Markdown files into a notebook, returning the new note ids.
    pub async fn import_markdown(
        &self,
        notebook_id: &str,
        files: Vec<ImportFile>,
    ) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(files.len());
        for file in files {
            let (title, body) = parse_markdown(&file.filename, &file.content);
            let note = self.create_note(notebook_id, &title, &body).await?;
            ids.push(note.id);
        }
        Ok(ids)
    }
}

/// Make a title safe to use as a filename.
fn sanitize_filename(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() { "untitled".to_string() } else { trimmed.to_string() }
}

/// Split a Markdown document into `(title, body)`, honoring a leading
/// `---`-delimited frontmatter block with a `title:` field.
fn parse_markdown(filename: &str, content: &str) -> (String, String) {
    let fallback_title = filename.strip_suffix(".md").unwrap_or(filename).to_string();

    if let Some(rest) = content.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            let frontmatter = &rest[..end];
            // Body is whatever follows the closing fence (skip its newline +
            // any single blank separator line).
            let after = &rest[end + "\n---".len()..];
            let body = after.strip_prefix('\n').unwrap_or(after);
            let body = body.strip_prefix('\n').unwrap_or(body).to_string();

            let title = frontmatter
                .lines()
                .find_map(|line| line.strip_prefix("title:").map(|t| t.trim().to_string()))
                .filter(|t| !t.is_empty())
                .unwrap_or(fallback_title);
            return (title, body);
        }
    }

    (fallback_title, content.to_string())
}
