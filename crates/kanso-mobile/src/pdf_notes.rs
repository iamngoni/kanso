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

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{PdfHighlightChange, PdfHighlightRow, PdfNotes};
use crate::pdf_reader_model::PdfEdited;

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

/// The list for one PDF of `body`.
pub(crate) fn notes_in(body: &str, file: &str, name: &str) -> PdfNotes {
    let annots = Annotations::from_body(body, file, "");
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

impl Library {
    /// The highlights each PDF has in note `note_id`, for the list the
    /// preview's "4 highlights" line opens. [`KansoError::Locked`] while the
    /// note is locked and closed.
    pub fn pdf_highlights(&self, note_id: &str) -> Result<Vec<PdfNotes>, KansoError> {
        guard(|| {
            let (_, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            Ok(files_in(&body)
                .into_iter()
                .map(|file| {
                    let name = self.pdf_name(&file);
                    notes_in(&body, &file, &name)
                })
                .collect())
        })
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
