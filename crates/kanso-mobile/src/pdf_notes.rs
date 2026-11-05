//! A PDF's highlights in a note, on the phone (issue #43, step 6).
//!
//! A note's PDF annotations live in a `kanso-pdf` block, whether read on
//! desktop or phone. The preview shows them as
//! one quiet line ([`kanso_render::pdf_notes`]); this is what the line opens:
//! the list, and the changes the desktop reader's panel makes (a comment, a
//! colour, a delete, a quote put back). The rules are `kanso_pdf::highlights`,
//! so the two ends write the same text.

use std::collections::HashSet;

use kanso_core::canvas::{Point, Stroke, unique_id};
use kanso_core::links::FILE_LINK_PREFIX;
use kanso_pdf::annotation::find_blocks;
use kanso_pdf::highlights::{Annotations, Located, Status, locate};
use kanso_pdf::model::Color;
use kanso_store::pdf_ink;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{PdfHighlightChange, PdfHighlightRow, PdfInkRow, PdfNotes, Saved};
use crate::pdf_reader_model::{PdfEdited, PdfInkPage, PdfInkPoint, PdfInkStroke};
use kanso_core::Note;
use kanso_engine::lock::SealedInk;
use kanso_sync::lock::Envelope;

/// The most times a change is made again on a note that moved under it.
const RETRIES: usize = 6;

pub(crate) fn color_name(color: Color) -> &'static str {
    match color {
        Color::Yellow => "yellow",
        Color::Green => "green",
        Color::Blue => "blue",
        Color::Pink => "pink",
    }
}

pub(crate) fn parse_color(name: &str) -> Result<Color, KansoError> {
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
    // A highlight another device removed meanwhile: the change has nothing to
    // act on, and the caller says so ([`PdfEdited::gone`]).
    if annots.get(id).is_none() {
        return Err(KansoError::NotFound);
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

/// Adds one stroke to the page in `body`, using the same validation,
/// companion-file sealing and block writer as the desktop reader.
fn add_ink_in(
    store: &kanso_store::Store,
    note_id: &str,
    body: &str,
    file: &str,
    page: u32,
    stroke: &Stroke,
    files: &dyn pdf_ink::InkFiles,
) -> Result<Option<String>, KansoError> {
    let mut annots = Annotations::from_body(body, file, "");
    pdf_ink::load_ink_with(store, &mut annots, files).map_err(KansoError::invalid)?;
    let current = annots.page_ink(page);
    let mut incoming = kanso_pdf::ink::PageInk::new(page);
    incoming.strokes.push(stroke.clone());
    let merged = kanso_pdf::ink::merge_page(&current, &incoming);
    annots
        .set_page_ink(merged)
        .map_err(|error| KansoError::invalid(error.to_string()))?;
    pdf_ink::publish_ink_with(store, &mut annots, note_id, body, files)
        .map_err(KansoError::invalid)?;
    annots
        .save_edit(body)
        .map_err(|error| KansoError::invalid(error.to_string()))
        .map(|edit| edit.map(|edit| edit.apply(body)))
}

impl Library {
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
                    notes_of(&annots, &body, &file, &name)
                })
                .collect())
        })
    }

    /// The strokes the PDF reader draws, from the same annotation block and
    /// sealed companion file the desktop reader uses.
    pub(crate) fn pdf_ink_pages(
        &self,
        note: &Note,
        body: &str,
        file: &str,
    ) -> Result<Vec<PdfInkPage>, KansoError> {
        let md5 = self
            .inner
            .store
            .file_meta(file)?
            .and_then(|meta| meta.md5digest)
            .unwrap_or_default();
        let mut annots = Annotations::from_body(body, file, &md5);
        let files = self.ink_files(note);
        pdf_ink::load_ink_with(&self.inner.store, &mut annots, &files)
            .map_err(KansoError::invalid)?;
        Ok(annots
            .ink()
            .iter()
            .map(|page| PdfInkPage {
                page: page.page,
                strokes: page
                    .live()
                    .map(|stroke| PdfInkStroke {
                        tool: stroke.tool.clone(),
                        color: stroke.color.clone(),
                        width: stroke.width,
                        points: stroke
                            .points
                            .iter()
                            .map(|point| PdfInkPoint {
                                x: point.x,
                                y: point.y,
                                pressure: point.pressure,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect())
    }

    /// Adds one pencil stroke to a page, making it again on newer text when
    /// the note changes while it is being saved.
    pub fn pdf_ink_add(
        &self,
        note_id: &str,
        base_rev: &str,
        file: &str,
        page: u32,
        input: PdfInkStroke,
    ) -> Result<PdfEdited, KansoError> {
        guard(|| {
            if input.points.is_empty() {
                return Err(KansoError::invalid("Draw a stroke before saving it."));
            }
            if input.points.len() > kanso_core::canvas::MAX_STROKE_POINTS {
                return Err(KansoError::invalid("That stroke has too many points."));
            }
            if !matches!(input.tool.as_str(), "pen" | "pencil" | "marker" | "highlighter") {
                return Err(KansoError::invalid("That isn't a pencil tool."));
            }
            if ![
                "#1e1e1e",
                "#e03131",
                "#e8590c",
                "#f08c00",
                "#2f9e44",
                "#1971c2",
                "#9c36b5",
            ]
            .contains(&input.color.as_str())
            {
                return Err(KansoError::invalid("That isn't a pencil colour."));
            }
            if input.points.iter().any(|point| {
                !point.pressure.is_finite() || !(0.0..=1.0).contains(&point.pressure)
            }) {
                return Err(KansoError::invalid("That stroke has invalid pressure."));
            }
            let now = kanso_core::now_ms();
            let stroke = Stroke {
                id: unique_id("s-", &HashSet::new()),
                tool: input.tool,
                color: input.color,
                width: input.width,
                points: input
                    .points
                    .into_iter()
                    .map(|point| Point::new(point.x, point.y, point.pressure))
                    .collect(),
                kanso_version: 1,
                kanso_updated: now,
                kanso_z: now as f64,
                ..Stroke::default()
            };
            let done = write_with_retry(self, note_id, base_rev, |body| {
                let files = self.ink_files_for_body(note_id)?;
                add_ink_in(
                    &self.inner.store,
                    note_id,
                    body,
                    file,
                    page,
                    &stroke,
                    &files,
                )
            })?;
            Ok(PdfEdited {
                rev: done.rev,
                id: None,
                merged: done.merged,
                gone: false,
            })
        })
    }

    fn ink_files_for_body(&self, note_id: &str) -> Result<SealedInk, KansoError> {
        let (note, _) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
        Ok(self.ink_files(&note))
    }

    /// Changes one highlight of `file` in note `note_id`, which was read at
    /// `base_rev`: one write of the note's text, as the desktop reader makes.
    ///
    /// A note that moved since (another device synced, or the editor saved)
    /// does not refuse the change: it is made again on the note's text now,
    /// where the highlights united by [`kanso_pdf::merge`] keep everything
    /// the other side did ([`PdfEdited::merged`] says it happened). Only a
    /// highlight that was deleted meanwhile has nothing to change; that is
    /// [`PdfEdited::gone`], and nothing is written.
    pub fn pdf_highlight_change(
        &self,
        note_id: &str,
        base_rev: &str,
        file: &str,
        id: &str,
        change: PdfHighlightChange,
    ) -> Result<PdfEdited, KansoError> {
        guard(|| {
            let name = self.pdf_name(file);
            let done = write_with_retry(self, note_id, base_rev, |body| {
                change_in(body, file, &name, id, &change)
            });
            match done {
                Ok(written) => Ok(PdfEdited {
                    rev: written.rev,
                    id: None,
                    merged: written.merged,
                    gone: false,
                }),
                Err(KansoError::NotFound) => {
                    let (note, _) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
                    Ok(PdfEdited {
                        rev: note.rev.unwrap_or_default(),
                        id: None,
                        merged: true,
                        gone: true,
                    })
                }
                Err(error) => Err(error),
            }
        })
    }

    /// Clears all the pencil marks on `page` of `file` in note `note_id`,
    /// which was read at `base_rev`: one write of the note's text (and, for
    /// ink that lives in its attachment, of that). If the note moved, the
    /// page is cleared again on its newest ink before saving.
    pub fn pdf_ink_clear(
        &self,
        note_id: &str,
        base_rev: &str,
        file: &str,
        page: u32,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            let written = write_with_retry(self, note_id, base_rev, |body| {
                let files = self.ink_files_for_body(note_id)?;
                clear_ink_in(&self.inner.store, note_id, body, file, page, &files)
            })?;
            Ok(Saved {
                id: note_id.to_owned(),
                rev: written.rev,
            })
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

/// What [`write_with_retry`] did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Written {
    /// The note's revision after.
    pub rev: String,
    /// The note had moved since `base_rev`, or moved again while the change
    /// was being saved, and the change was made on what it holds now.
    pub merged: bool,
}

/// Makes a change to a note's text and saves it, again on the note's newer
/// text when it moved, up to [`RETRIES`] times. `edit` is given the text as it
/// is now and answers the text to save, or `None` for no change; it is run
/// afresh on every try, so what it makes is made from the latest text and
/// nothing is written over another side's work. A change that runs out of
/// tries is a conflict, told to the person, never dropped quietly.
pub(crate) fn write_with_retry(
    library: &Library,
    note_id: &str,
    base_rev: &str,
    edit: impl Fn(&str) -> Result<Option<String>, KansoError>,
) -> Result<Written, KansoError> {
    let mut merged = false;
    let mut last = String::new();
    for _ in 0..RETRIES {
        let (note, body) = library.readable_text(note_id)?.ok_or(KansoError::Locked)?;
        let rev = note.rev.clone().unwrap_or_default();
        if rev != base_rev {
            merged = true;
        }
        let Some(next) = edit(&body)? else {
            return Ok(Written { rev, merged });
        };
        match library.save(note_id, &rev, next) {
            Ok(saved) => {
                return Ok(Written {
                    rev: saved.rev,
                    merged,
                });
            }
            Err(KansoError::Conflict { current_rev }) => {
                merged = true;
                last = current_rev;
            }
            Err(error) => return Err(error),
        }
    }
    Err(KansoError::Conflict { current_rev: last })
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
