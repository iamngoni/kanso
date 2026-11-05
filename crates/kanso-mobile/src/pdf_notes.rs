//! A PDF's highlights in a note, on the phone (issue #43, step 6).
//!
//! The phone has no PDF reader, but a note that was read on a computer
//! carries its highlights in a `kanso-pdf` block. The preview shows them as
//! one quiet line ([`kanso_render::pdf_notes`]); this is what the line opens:
//! the list, and the changes the desktop reader's panel makes (a comment, a
//! colour, a delete, a quote put back). The rules are `kanso_pdf::highlights`,
//! so the two ends write the same text.

use kanso_core::links::FILE_LINK_PREFIX;
use kanso_pdf::annotation::find_blocks;
use kanso_pdf::highlights::{Annotations, Located, Status, locate};
use kanso_pdf::model::Color;
use kanso_store::pdf_ink;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{PdfHighlightChange, PdfHighlightRow, PdfInkRow, PdfNotes, Saved};
use kanso_core::Note;
use kanso_engine::lock::SealedInk;
use kanso_sync::lock::Envelope;

/// Phone PDF export stays off until its isolated renderer boundary is chosen
/// and reviewed (PR #310). The shell intentionally has no export UI yet.
pub const PDF_EXPORT_ENABLED: bool = false;
const PDF_EXPORT_DISABLED: &str =
    "PDF export on phones is disabled until an isolated renderer is approved.";

fn color_name(color: Color) -> &'static str {
    match color {
        Color::Yellow => "yellow",
        Color::Green => "green",
        Color::Blue => "blue",
        Color::Pink => "pink",
    }
}

fn parse_color(name: &str) -> Result<Color, KansoError> {
    Color::ALL
        .into_iter()
        .find(|c| color_name(*c) == name)
        .ok_or_else(|| KansoError::invalid("That isn't a highlight colour."))
}

/// The files that have a block in `body`, in order, once each.
fn files_in(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for span in find_blocks(body) {
        let json = body[span.json].trim();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json)
            && let Some(file) = value.get("file").and_then(|f| f.as_str())
            && !out.iter().any(|f| f == file)
        {
            out.push(file.to_owned());
        }
    }
    out
}

/// The list for one PDF of `body`, as written in the note (no ink attachment).
#[cfg(test)]
pub(crate) fn notes_in(body: &str, file: &str, name: &str) -> PdfNotes {
    notes_of(&Annotations::from_body(body, file, ""), body, file, name)
}

/// The list for one PDF, from its highlights `annots` (with the pencil ink
/// read in, when it lives in its attachment).
fn notes_of(annots: &Annotations, body: &str, file: &str, name: &str) -> PdfNotes {
    let (editable, why) = match &annots.status {
        Status::Editable => (true, None),
        Status::Restorable => (false, None),
        Status::Unreadable(why) => (false, Some(why.clone())),
    };
    let highlights = annots
        .rows(body)
        .into_iter()
        .map(|row| PdfHighlightRow {
            id: row.id,
            page: row.page,
            color: color_name(row.color).to_owned(),
            text: row.text,
            comment: row.comment,
            in_note: row.in_note,
        })
        .collect();
    PdfNotes {
        file: file.to_owned(),
        name: name.to_owned(),
        url: format!(
            "{FILE_LINK_PREFIX}{}",
            file.strip_prefix("file:").unwrap_or(file)
        ),
        editable,
        why,
        highlights,
        scanned_pages: 0,
        ink: annots
            .ink_rows()
            .into_iter()
            .map(|row| PdfInkRow {
                page: row.page,
                strokes: row.strokes as u32,
            })
            .collect(),
    }
}

/// Applies `change` to the highlight `id` of `file` in `body`: the new body,
/// or `None` when it changes nothing.
pub(crate) fn change_in(
    body: &str,
    file: &str,
    name: &str,
    id: &str,
    change: &PdfHighlightChange,
) -> Result<Option<String>, KansoError> {
    let invalid = |e: kanso_pdf::PdfError| KansoError::invalid(e.to_string());
    if matches!(locate(body, file), Located::Missing) {
        return Err(KansoError::invalid(
            "This note has no highlights for that PDF.",
        ));
    }
    let mut annots = Annotations::from_body(body, file, "");
    if annots.get(id).is_none() {
        return Err(KansoError::invalid("That highlight no longer exists."));
    }
    let edit = match change {
        PdfHighlightChange::Comment { comment } => {
            annots.set_comment(id, Some(comment)).map_err(invalid)?;
            annots.save_edit(body).map_err(invalid)?
        }
        PdfHighlightChange::Color { color } => {
            annots.set_color(id, parse_color(color)?).map_err(invalid)?;
            annots.save_edit(body).map_err(invalid)?
        }
        PdfHighlightChange::Delete => {
            annots.delete(id).map_err(invalid)?;
            annots.save_edit(body).map_err(invalid)?
        }
        PdfHighlightChange::AddQuote => annots.quote_edit(body, id, name),
    };
    Ok(edit.map(|e| e.apply(body)))
}

/// `body` with the pencil marks on `page` of `file` cleared, or `None` when
/// there were none. Ink in an attachment is read from `store` and written
/// back there.
pub(crate) fn clear_ink_in(
    store: &kanso_store::Store,
    note_id: &str,
    body: &str,
    file: &str,
    page: u32,
    files: &dyn pdf_ink::InkFiles,
) -> Result<Option<String>, KansoError> {
    let invalid = |e: kanso_pdf::PdfError| KansoError::invalid(e.to_string());
    if matches!(locate(body, file), Located::Missing) {
        return Err(KansoError::invalid(
            "This note has no pencil marks for that PDF.",
        ));
    }
    let mut annots = Annotations::from_body(body, file, "");
    pdf_ink::load_ink_with(store, &mut annots, files).map_err(KansoError::invalid)?;
    if !annots
        .clear_page_ink(page, kanso_core::now_ms())
        .map_err(invalid)?
    {
        return Ok(None);
    }
    pdf_ink::publish_ink_with(store, &mut annots, note_id, body, files)
        .map_err(KansoError::invalid)?;
    Ok(annots
        .save_edit(body)
        .map_err(invalid)?
        .map(|e| e.apply(body)))
}

impl Library {
    /// Whether PDF export can be offered on this phone build. This is a
    /// feature-flagged bridge hook only; no in-process PDFium call exists.
    pub fn pdf_export_enabled(&self) -> bool {
        PDF_EXPORT_ENABLED
    }

    /// Reserved bridge hook for the eventual isolated phone renderer. It
    /// fails closed while the phone feature flag remains off.
    pub fn pdf_export_copy(&self, _note_id: &str, _file: &str) -> Result<Vec<u8>, KansoError> {
        guard(|| Err(KansoError::invalid(PDF_EXPORT_DISABLED)))
    }

    /// The highlights each PDF has in note `note_id`, for the list the
    /// preview's "4 highlights" line opens. [`KansoError::Locked`] while the
    /// note is locked and closed.
    pub fn pdf_highlights(&self, note_id: &str) -> Result<Vec<PdfNotes>, KansoError> {
        guard(|| {
            let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            let files = self.ink_files(&note);
            Ok(files_in(&body)
                .into_iter()
                .map(|file| {
                    let name = self.pdf_name(&file);
                    let mut annots = Annotations::from_body(&body, &file, "");
                    // Ink that outgrew the note is read from its attachment.
                    // One that has not arrived yet is listed when it does.
                    let _ = pdf_ink::load_ink_with(&self.inner.store, &mut annots, &files);
                    let mut notes = notes_of(&annots, &body, &file, &name);
                    notes.scanned_pages = self.scanned_pages(&file);
                    notes
                })
                .collect())
        })
    }

    /// Changes one highlight of `file` in note `note_id`, which was read at
    /// `base_rev`: one write of the note's text, as the desktop reader makes.
    /// With a newer revision ([`KansoError::Conflict`]) nothing is written.
    pub fn pdf_highlight_change(
        &self,
        note_id: &str,
        base_rev: &str,
        file: &str,
        id: &str,
        change: PdfHighlightChange,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            let rev = note.rev.clone().unwrap_or_default();
            if rev != base_rev {
                return Err(KansoError::Conflict { current_rev: rev });
            }
            let name = self.pdf_name(file);
            match change_in(&body, file, &name, id, &change)? {
                Some(next) => self.save(note_id, &rev, next),
                None => Ok(Saved {
                    id: note_id.to_owned(),
                    rev,
                }),
            }
        })
    }

    /// The pages of PDF `file` whose words a computer read from their
    /// pictures (step 8); the store hands them over only while the bytes are
    /// the ones they were read from.
    pub(crate) fn scanned_pages(&self, file: &str) -> u32 {
        self.inner
            .store
            .pdf_ocr(file)
            .ok()
            .flatten()
            .map_or(0, |ocr| {
                ocr.pages.iter().filter(|p| !p.words.is_empty()).count() as u32
            })
    }

    /// Clears all the pencil marks on `page` of `file` in note `note_id`,
    /// which was read at `base_rev`: one write of the note's text (and, for
    /// ink that lives in its attachment, of that). With a newer revision
    /// ([`KansoError::Conflict`]) nothing is written.
    pub fn pdf_ink_clear(
        &self,
        note_id: &str,
        base_rev: &str,
        file: &str,
        page: u32,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            let rev = note.rev.clone().unwrap_or_default();
            if rev != base_rev {
                return Err(KansoError::Conflict { current_rev: rev });
            }
            let files = self.ink_files(&note);
            match clear_ink_in(&self.inner.store, note_id, &body, file, page, &files)? {
                Some(next) => self.save(note_id, &rev, next),
                None => Ok(Saved {
                    id: note_id.to_owned(),
                    rev,
                }),
            }
        })
    }

    /// The lock-aware byte store for `note`'s ink. A locked note uses its
    /// held lock key for a new sealed attachment; a plain note stays plain.
    fn ink_files(&self, note: &Note) -> SealedInk {
        let keys = held(&self.inner.keys).file_keys();
        if note.is_locked() {
            if let Some(Ok(envelope)) = Envelope::from_extra(&note.extra)
                && let Some(key) = held(&self.inner.keys).fitting(&envelope)
            {
                return SealedInk::new(keys, note, key);
            }
            return SealedInk::reading(keys);
        }
        SealedInk::plain(keys)
    }

    fn pdf_name(&self, file: &str) -> String {
        self.inner
            .store
            .file_meta(file)
            .ok()
            .flatten()
            .map_or_else(|| "PDF".to_owned(), |meta| meta.name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kanso_pdf::model::Quad;
    use kanso_pdf::text::HighlightParts;
    use proptest::prelude::*;

    use super::*;

    const FILE: &str = "file:abc123";

    fn body_with(n: usize) -> String {
        let mut a = Annotations::new(FILE, "m");
        for i in 0..n {
            let mut quads = BTreeMap::new();
            quads.insert(1 + i as u32, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
            a.add(
                HighlightParts {
                    quads,
                    text: format!("words {i}"),
                },
                Color::ALL[i % 4],
                i as i64,
            )
            .unwrap();
        }
        a.save_edit("Notes").unwrap().unwrap().apply("Notes")
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        /// Whatever is done to a highlight, the note reads back with exactly
        /// that change and the text around the block is as it was.
        #[test]
        fn a_change_reads_back(n in 1usize..8, pick in 0usize..8, what in 0u8..4, text in "[a-z]{1,20}") {
            let body = body_with(n);
            let rows = notes_in(&body, FILE, "R.pdf").highlights;
            let id = rows[pick % rows.len()].id.clone();
            let change = match what {
                0 => PdfHighlightChange::Comment { comment: text.clone() },
                1 => PdfHighlightChange::Color { color: "pink".into() },
                2 => PdfHighlightChange::AddQuote,
                _ => PdfHighlightChange::Delete,
            };
            let next = change_in(&body, FILE, "R.pdf", &id, &change).unwrap().unwrap_or_else(|| body.clone());
            let after = notes_in(&next, FILE, "R.pdf").highlights;
            prop_assert!(next.starts_with("Notes"));
            match change {
                PdfHighlightChange::Comment { .. } => {
                    let row = after.iter().find(|r| r.id == id).unwrap();
                    prop_assert_eq!(row.comment.as_deref(), Some(text.as_str()));
                }
                PdfHighlightChange::Color { .. } => {
                    prop_assert_eq!(after.iter().find(|r| r.id == id).unwrap().color.as_str(), "pink");
                }
                PdfHighlightChange::AddQuote => {
                    prop_assert!(after.iter().find(|r| r.id == id).unwrap().in_note);
                    // Once only: asking again is no change.
                    prop_assert_eq!(change_in(&next, FILE, "R.pdf", &id, &PdfHighlightChange::AddQuote).unwrap(), None);
                }
                PdfHighlightChange::Delete => {
                    prop_assert!(after.iter().all(|r| r.id != id));
                    prop_assert_eq!(after.len(), rows.len() - 1);
                }
            }
        }
    }

    #[test]
    fn a_block_that_cannot_be_read_is_not_changed() {
        let broken = "```kanso-pdf\n{\"file\":\"file:abc123\",\"highlights\":[oops\n```";
        let notes = notes_in(broken, FILE, "R.pdf");
        assert!(!notes.editable && notes.why.is_some());
        assert!(change_in(broken, FILE, "R.pdf", "x", &PdfHighlightChange::Delete).is_err());
    }

    #[test]
    fn files_are_listed_once_in_order() {
        let a = body_with(1);
        let b = a.replace("abc123", "zzz999");
        let both = format!("{a}\n\n{b}\n\n{a}");
        assert_eq!(files_in(&both), vec!["file:abc123", "file:zzz999"]);
    }
}
