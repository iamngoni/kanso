//! Working with one PDF's highlights in a note (issue #43, step 6).
//!
//! The reader on the desktop and the highlights sheet on a phone both keep a
//! PDF's highlights in an [`Annotations`]: the block as last read from the
//! note body, and whether it may be changed. Every rule about the block lives
//! here, as pure functions over strings, so both ends behave the same:
//!
//! * a block that cannot be read is never written over;
//! * a block that vanished from the body (the note was restored to an older
//!   version) leaves the last highlights on screen, read-only, until the
//!   person restores them as one edit;
//! * a highlight, a comment or a delete is a change to the block, saved as one
//!   edit by [`Annotations::save_edit`].

use std::collections::BTreeSet;

use crate::PdfError;
use crate::annotation::new_highlight_id;
use crate::link::PdfLink;
use crate::model::{Color, Highlight, MAX_COMMENT_CHARS, PdfBlock};
use crate::quote::{Edit, append_quote, render_quote};
use crate::text::HighlightParts;

/// What is said when an edit is refused because the block is read-only.
pub const READ_ONLY: &str =
    "These highlights are read-only. Restore them to the note to change them.";

/// Where a PDF's block stands in a body.
#[derive(Clone, Debug, PartialEq)]
pub enum Located {
    /// The body has a block for this PDF and it reads cleanly.
    Found(PdfBlock),
    /// The body has no block for this PDF.
    Missing,
    /// The body has one for this PDF that cannot be read.
    Broken(PdfError),
}

/// Finds the block for `file` (`file:<id>`) in `body`. Several blocks for
/// the same PDF (two editors of a shared note each wrote one) read as one,
/// their highlights united; when any of them can't be read, none is used.
pub fn locate(body: &str, file: &str) -> Located {
    match crate::merge::united(body, file) {
        Ok(Some(block)) => Located::Found(block),
        Ok(None) => Located::Missing,
        Err(e) => Located::Broken(e),
    }
}

/// Whether the highlights can be changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Editable,
    /// The block left the body; the last highlights are shown until restored.
    Restorable,
    /// The block in the body cannot be read; nothing is written over it.
    Unreadable(String),
}

/// One PDF's highlights, as the note has them.
#[derive(Clone, Debug, PartialEq)]
pub struct Annotations {
    pub block: PdfBlock,
    pub status: Status,
}

/// One row of the highlights list.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub page: u32,
    pub color: Color,
    pub text: String,
    pub comment: Option<String>,
    /// Whether the quote for it is still in the note.
    pub in_note: bool,
}

impl Annotations {
    /// An empty set for a PDF with no highlights yet.
    pub fn new(file: &str, md5: &str) -> Self {
        Annotations {
            block: PdfBlock::new(file, md5),
            status: Status::Editable,
        }
    }

    /// Reads the highlights of `file` out of `body`.
    pub fn from_body(body: &str, file: &str, md5: &str) -> Self {
        let mut a = Annotations::new(file, md5);
        a.sync(body);
        a
    }

    pub fn file(&self) -> &str {
        &self.block.file
    }

    /// Follows the body. A block that reads replaces the highlights; one that
    /// is gone leaves them as read-only when there are any; one that cannot be
    /// read leaves them untouched and read-only. Whether anything changed.
    pub fn sync(&mut self, body: &str) -> bool {
        let before = self.clone();
        match locate(body, &self.block.file) {
            Located::Found(block) => {
                self.block = block;
                self.status = Status::Editable;
            }
            Located::Missing => {
                if !self.block.highlights.is_empty() {
                    self.status = Status::Restorable;
                } else {
                    self.status = Status::Editable;
                }
            }
            Located::Broken(e) => self.status = Status::Unreadable(e.to_string()),
        }
        *self != before
    }

    pub fn is_editable(&self) -> bool {
        self.status == Status::Editable
    }

    fn check(&self) -> Result<(), PdfError> {
        match &self.status {
            Status::Editable => Ok(()),
            Status::Restorable => Err(PdfError::Invalid(READ_ONLY.into())),
            Status::Unreadable(why) => Err(PdfError::Invalid(why.clone())),
        }
    }

    /// Adds a highlight for a selection; the new highlight's id.
    pub fn add(
        &mut self,
        parts: HighlightParts,
        color: Color,
        now_ms: i64,
    ) -> Result<String, PdfError> {
        self.check()?;
        let h = new_highlight(parts, color, now_ms).ok_or_else(|| {
            PdfError::Invalid("Select some text on the page to highlight it.".into())
        })?;
        let id = h.id.clone();
        self.block.add_highlight(h)?;
        Ok(id)
    }

    /// Removes a highlight; whether it was there.
    pub fn delete(&mut self, id: &str) -> Result<bool, PdfError> {
        self.check()?;
        Ok(self.block.remove_highlight(id))
    }

    /// Sets or clears (blank) a highlight's comment.
    pub fn set_comment(&mut self, id: &str, comment: Option<&str>) -> Result<(), PdfError> {
        self.check()?;
        self.block.set_comment(id, comment)
    }

    /// Changes a highlight's colour.
    pub fn set_color(&mut self, id: &str, color: Color) -> Result<(), PdfError> {
        self.check()?;
        let h = self
            .block
            .highlights
            .iter_mut()
            .find(|h| h.id == id)
            .ok_or_else(|| PdfError::Invalid("That highlight no longer exists.".into()))?;
        if h.color != color {
            h.color = color;
            h.rev = h.rev.saturating_add(1);
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Highlight> {
        self.block.highlights.iter().find(|h| h.id == id)
    }

    /// The highlight drawn at a point of a page (normalised, top-left), the
    /// newest when several overlap.
    pub fn hit(&self, page: u32, x: f64, y: f64) -> Option<&Highlight> {
        self.block.highlights.iter().rev().find(|h| {
            h.quads
                .get(&page)
                .is_some_and(|qs| qs.iter().any(|q| contains(q, x, y)))
        })
    }

    /// The edit that saves the highlights into `body`, united with the
    /// version of them the body holds now: highlights another editor (or
    /// device) added since are kept, and the ones deleted here stay deleted.
    /// `None` when the body already says all of it. A block for this PDF that
    /// can't be read is an error, and is never written over.
    pub fn save_edit(&self, body: &str) -> Result<Option<Edit>, PdfError> {
        self.check()?;
        crate::merge::save_into(body, &self.block)
    }

    /// The edit that writes the last highlights back after the block left
    /// the note (united with a block that came back meanwhile, if any).
    /// `None` when the body already has them all.
    pub fn restore_edit(&self, body: &str) -> Result<Option<Edit>, PdfError> {
        if let Status::Unreadable(why) = &self.status {
            return Err(PdfError::Invalid(why.clone()));
        }
        crate::merge::save_into(body, &self.block)
    }

    /// The Markdown quote for a highlight, to put under `## Highlights`.
    pub fn quote(&self, id: &str) -> Option<String> {
        self.get(id).map(|h| quote_of(&self.block.file, h))
    }

    /// The edit that puts a highlight's quote into `body`, once.
    pub fn quote_edit(&self, body: &str, id: &str, pdf_name: &str) -> Option<Edit> {
        let quote = self.quote(id)?;
        if quote_ids(body, &self.block.file).contains(id) {
            return None;
        }
        Some(append_quote(body, pdf_name, &quote))
    }

    /// The list: by page, then in the order they were made.
    pub fn rows(&self, body: &str) -> Vec<Row> {
        let present = quote_ids(body, &self.block.file);
        let mut rows: Vec<(&Highlight, Row)> = self
            .block
            .highlights
            .iter()
            .map(|h| {
                (
                    h,
                    Row {
                        id: h.id.clone(),
                        page: h.page,
                        color: h.color,
                        text: h.text.clone(),
                        comment: h.comment.clone(),
                        in_note: present.contains(&h.id),
                    },
                )
            })
            .collect();
        rows.sort_by(|(a, _), (b, _)| {
            (a.page, first_y(a), a.created_at)
                .partial_cmp(&(b.page, first_y(b), b.created_at))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rows.into_iter().map(|(_, r)| r).collect()
    }
}

fn first_y(h: &Highlight) -> f64 {
    h.quads
        .get(&h.page)
        .and_then(|q| q.first())
        .map_or(0., |q| q.0[1])
}

fn contains(q: &crate::model::Quad, x: f64, y: f64) -> bool {
    x >= q.0[0] && x <= q.0[2] && y >= q.0[1] && y <= q.0[3]
}

/// A highlight made of a selection, or `None` when the selection is empty.
/// The quoted text is whitespace-collapsed and cut to its cap.
pub fn new_highlight(parts: HighlightParts, color: Color, now_ms: i64) -> Option<Highlight> {
    let text: String = parts
        .text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(crate::model::MAX_QUOTE_CHARS)
        .collect();
    let quads: std::collections::BTreeMap<u32, Vec<crate::model::Quad>> = parts
        .quads
        .into_iter()
        .filter(|(_, q)| !q.is_empty())
        .collect();
    let page = *quads.keys().next()?;
    if text.is_empty() {
        return None;
    }
    Some(Highlight {
        id: new_highlight_id(),
        page,
        quads,
        text,
        color,
        comment: None,
        created_at: now_ms,
        rev: 0,
        extra: Default::default(),
    })
}

/// The Markdown quote for `h`, linking to its page and id.
pub fn quote_of(file: &str, h: &Highlight) -> String {
    let link = PdfLink {
        file_id: file.strip_prefix("file:").unwrap_or(file).to_string(),
        page: Some(h.page),
        highlight: Some(h.id.clone()),
    };
    render_quote(&h.text, &link)
}

/// The highlight ids whose quotes are in `body` for `file` (`file:<id>`):
/// every `inkdrop://file:<id>#…h=<hid>` link there.
pub fn quote_ids(body: &str, file: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let want = file.strip_prefix("file:").unwrap_or(file);
    let mut rest = body;
    while let Some(i) = rest.find(kanso_core::links::FILE_LINK_PREFIX) {
        let tail = &rest[i..];
        let end = tail
            .find(|c: char| c.is_whitespace() || c == ')' || c == '>' || c == '"')
            .unwrap_or(tail.len());
        if let Some(link) = PdfLink::parse(&tail[..end])
            && link.file_id == want
        {
            out.extend(link.highlight);
        }
        rest = &tail[kanso_core::links::FILE_LINK_PREFIX.len().min(tail.len())..];
    }
    out
}

/// Caps a comment the way the block does, for callers that take free text.
pub fn clip_comment(comment: &str) -> String {
    comment.chars().take(MAX_COMMENT_CHARS).collect()
}

#[cfg(test)]
mod tests;
