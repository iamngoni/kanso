//! PDF text and annotations through the local API and MCP.
//!
//! This module authorizes the owning note before opening PDF bytes. Annotation
//! changes are merged into the note body and saved as one ordinary note
//! revision; the PDF itself is never modified.

use kanso_core::id::is_valid_id;
use kanso_core::{DocType, Note, now_ms};
use kanso_pdf::client::{Worker, locate};
use kanso_pdf::highlights::{Annotations, Row};
use kanso_pdf::model::Color;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::{Value, json};

use crate::api;
use crate::canvas::Host;
use crate::error::{ApiError, ApiResult};
use crate::scope::Visibility;

const MAX_QUERY_CHARS: usize = 256;
const MAX_RESULTS: usize = 100;
const MAX_PAGE: u32 = 20_000;
const WRITE_RETRIES: usize = 5;

/// A PDF attached to a visible, unlocked note.
struct VisiblePdf<'a> {
    note: Note,
    file_id: &'a str,
    md5: String,
    name: String,
}

/// Inputs for adding one anchored highlight through either transport.
pub(crate) struct AddHighlight<'a> {
    pub note_id: &'a str,
    pub file_id: &'a str,
    pub page: u32,
    pub quote: &'a str,
    pub color: &'a str,
    pub comment: Option<&'a str>,
    pub include_quote: bool,
}

/// Inputs for editing one existing highlight through either transport.
pub(crate) struct EditHighlight<'a> {
    pub note_id: &'a str,
    pub file_id: &'a str,
    pub highlight_id: &'a str,
    pub color: Option<&'a str>,
    pub comment: Option<Option<&'a str>>,
}

/// Checks the note before consulting attachment metadata or bytes. Agents
/// receive the same not-found result for missing, locked, excluded and
/// out-of-scope notes.
fn open<'a>(
    store: &Store,
    visibility: &Visibility,
    note_id: &str,
    file_id: &'a str,
) -> ApiResult<VisiblePdf<'a>> {
    if !is_valid_id(DocType::Note, note_id) || !is_valid_id(DocType::File, file_id) {
        return Err(ApiError::not_found());
    }
    if store.note_is_locked(note_id)? || store.note_in_locked_book(note_id)? {
        return Err(ApiError::not_found());
    }
    if !visibility.allows(note_id) {
        return Err(ApiError::not_found());
    }
    let note = visibility.note(note_id).ok_or_else(ApiError::not_found)?;
    let owns = store.files_of_note(note_id)?.iter().any(|id| id == file_id);
    let all_owners_visible = store
        .file_owners(file_id)?
        .iter()
        .all(|owner| visibility.note(owner).is_some());
    if !owns || !all_owners_visible {
        return Err(ApiError::not_found());
    }
    let doc = store.get(file_id)?.ok_or_else(ApiError::not_found)?;
    if doc.doc_type != DocType::File
        || !doc
            .fields
            .get("contentType")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("application/pdf"))
        || doc.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
    {
        return Err(ApiError::not_found());
    }
    let md5 = doc
        .fields
        .get("md5digest")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let name = doc
        .fields
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("PDF")
        .to_owned();
    visibility.show(&note.id);
    Ok(VisiblePdf {
        note,
        file_id,
        md5,
        name,
    })
}

/// Searches only PDF text already extracted by the background indexer.
pub(crate) fn search(
    store: &Store,
    visibility: &Visibility,
    query: &str,
    limit: usize,
) -> ApiResult<Value> {
    let query = query.trim();
    if query.is_empty()
        || query.chars().count() > MAX_QUERY_CHARS
        || limit == 0
        || limit > MAX_RESULTS
    {
        return Err(ApiError::bad_request(
            "Use a search phrase of 1–256 characters and a limit from 1 to 100.",
        ));
    }
    let needle = folded(query);
    let mut hits = Vec::new();
    // Traverse only notes the caller may see; never report a file ID or text
    // before its owning note has passed visibility checks.
    for note in store.plain_notes_with_files()? {
        if !visibility.allows(&note.id) {
            continue;
        }
        for file_id in store.files_of_note(&note.id)? {
            let Ok(Some(file)) = store.get(&file_id) else {
                continue;
            };
            if file.doc_type != DocType::File
                || file.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
                || !file
                    .fields
                    .get("contentType")
                    .and_then(Value::as_str)
                    .is_some_and(|content_type| {
                        content_type.eq_ignore_ascii_case("application/pdf")
                    })
            {
                continue;
            }
            if !store.pdf_indexed(&file_id)? {
                continue;
            }
            if store
                .file_owners(&file_id)?
                .iter()
                .any(|owner| visibility.note(owner).is_none())
            {
                continue;
            }
            for (index, text) in store.pdf_pages(&file_id)?.into_iter().enumerate() {
                if folded(&text).contains(&needle) {
                    visibility.show(&note.id);
                    hits.push(json!({"noteId": note.id, "fileId": file_id, "page": index + 1, "text": excerpt(&text, &needle)}));
                    if hits.len() == limit {
                        return Ok(json!({"results": hits}));
                    }
                }
            }
        }
    }
    Ok(json!({"results": hits}))
}

/// Reads one indexed page as text. Page numbers are one-based.
pub(crate) fn read_page(
    store: &Store,
    visibility: &Visibility,
    note_id: &str,
    file_id: &str,
    page: u32,
) -> ApiResult<Value> {
    if !(1..=MAX_PAGE).contains(&page) {
        return Err(ApiError::bad_request("The page number is out of range."));
    }
    let pdf = open(store, visibility, note_id, file_id)?;
    if !store.pdf_indexed(pdf.file_id)? {
        return Err(ApiError::not_found());
    }
    let pages = store.pdf_pages(pdf.file_id)?;
    let text = pages
        .get((page - 1) as usize)
        .ok_or_else(ApiError::not_found)?;
    Ok(json!({"noteId": pdf.note.id, "fileId": pdf.file_id, "page": page, "text": text}))
}

/// Lists the note's highlights and comments for one PDF.
pub(crate) fn list(
    store: &Store,
    visibility: &Visibility,
    note_id: &str,
    file_id: &str,
) -> ApiResult<Value> {
    let pdf = open(store, visibility, note_id, file_id)?;
    let anns = Annotations::from_body(&pdf.note.body, pdf.file_id, &pdf.md5);
    let rows = anns
        .rows(&pdf.note.body)
        .into_iter()
        .map(row_json)
        .collect::<Vec<_>>();
    Ok(
        json!({"noteId": pdf.note.id, "fileId": pdf.file_id, "rev": pdf.note.rev, "highlights": rows}),
    )
}

/// Adds a highlight by resolving the exact quoted text against PDFium's page
/// characters, using the same normalized character quads as the reader.
pub(crate) fn add(
    store: &Store,
    host: &Host,
    visibility: &Visibility,
    input: AddHighlight<'_>,
) -> ApiResult<Value> {
    let AddHighlight {
        note_id,
        file_id,
        page,
        quote,
        color,
        comment,
        include_quote,
    } = input;
    if !(1..=MAX_PAGE).contains(&page)
        || quote.is_empty()
        || quote.chars().count() > kanso_pdf::MAX_QUOTE_CHARS
    {
        return Err(ApiError::bad_request(
            "Choose a valid page and a quote of at most 1,000 characters.",
        ));
    }
    if comment.is_some_and(|value| value.chars().count() > kanso_pdf::MAX_COMMENT_CHARS) {
        return Err(ApiError::bad_request("The comment is too long."));
    }
    let color = parse_color(color)?;
    for attempt in 0..WRITE_RETRIES {
        let pdf = open(store, visibility, note_id, file_id)?;
        let page_text = worker_page(store, pdf.file_id, &pdf.md5, page)?;
        let parts = kanso_pdf::TextDoc::new(std::iter::once(&page_text))
            .find_quote(page, quote)
            .ok_or_else(|| ApiError::bad_request("That quote was not found on the page."))?;
        let mut annotations = Annotations::from_body(&pdf.note.body, pdf.file_id, &pdf.md5);
        let id = annotations.add(parts, color, now_ms()).map_err(pdf_error)?;
        if let Some(comment) = comment {
            annotations
                .set_comment(&id, Some(comment))
                .map_err(pdf_error)?;
        }
        let quote = include_quote.then_some((id.as_str(), pdf.name.as_str()));
        match save(store, host, &pdf.note, &annotations, quote) {
            Ok((rev, _)) => {
                return Ok(
                    json!({"ok": true, "id": pdf.note.id, "fileId": pdf.file_id, "highlightId": id, "rev": rev}),
                );
            }
            Err(StoreError::Conflict) if attempt + 1 < WRITE_RETRIES => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::conflict())
}

/// Changes a highlight's comment while preserving annotations merged by a
/// concurrent editor.
pub(crate) fn comment(
    store: &Store,
    host: &Host,
    visibility: &Visibility,
    note_id: &str,
    file_id: &str,
    highlight_id: &str,
    value: Option<&str>,
) -> ApiResult<Value> {
    if !valid_highlight_id(highlight_id)
        || value.is_some_and(|s| s.chars().count() > kanso_pdf::MAX_COMMENT_CHARS)
    {
        return Err(ApiError::bad_request(
            "The highlight ID or comment is invalid.",
        ));
    }
    update(store, host, visibility, note_id, file_id, |annotations| {
        annotations.set_comment(highlight_id, value)
    })
}

/// Changes a highlight's colour and/or comment. Omitted comment is preserved;
/// an empty or null comment clears it.
pub(crate) fn edit(
    store: &Store,
    host: &Host,
    visibility: &Visibility,
    input: EditHighlight<'_>,
) -> ApiResult<Value> {
    let EditHighlight {
        note_id,
        file_id,
        highlight_id,
        color,
        comment,
    } = input;
    if !valid_highlight_id(highlight_id) || (color.is_none() && comment.is_none()) {
        return Err(ApiError::bad_request(
            "The highlight ID or edit is invalid.",
        ));
    }
    if comment
        .flatten()
        .is_some_and(|text| text.chars().count() > kanso_pdf::MAX_COMMENT_CHARS)
    {
        return Err(ApiError::bad_request("The comment is too long."));
    }
    let color = color.map(parse_color).transpose()?;
    update(store, host, visibility, note_id, file_id, |annotations| {
        if let Some(color) = color {
            annotations.set_color(highlight_id, color)?;
        }
        if let Some(comment) = comment {
            annotations.set_comment(highlight_id, comment)?;
        }
        Ok(())
    })
}

/// Deletes a highlight by its stable ID.
pub(crate) fn delete(
    store: &Store,
    host: &Host,
    visibility: &Visibility,
    note_id: &str,
    file_id: &str,
    highlight_id: &str,
) -> ApiResult<Value> {
    if !valid_highlight_id(highlight_id) {
        return Err(ApiError::bad_request("The highlight ID is invalid."));
    }
    update(store, host, visibility, note_id, file_id, |annotations| {
        annotations.delete(highlight_id).and_then(|deleted| {
            deleted.then_some(()).ok_or_else(|| {
                kanso_pdf::PdfError::Invalid("That highlight no longer exists.".into())
            })
        })
    })
}

fn update(
    store: &Store,
    host: &Host,
    visibility: &Visibility,
    note_id: &str,
    file_id: &str,
    change: impl Fn(&mut Annotations) -> Result<(), kanso_pdf::PdfError>,
) -> ApiResult<Value> {
    for attempt in 0..WRITE_RETRIES {
        let pdf = open(store, visibility, note_id, file_id)?;
        let mut annotations = Annotations::from_body(&pdf.note.body, pdf.file_id, &pdf.md5);
        change(&mut annotations).map_err(pdf_error)?;
        match save(store, host, &pdf.note, &annotations, None) {
            Ok((rev, _)) => {
                return Ok(
                    json!({"ok": true, "id": pdf.note.id, "fileId": pdf.file_id, "rev": rev}),
                );
            }
            Err(StoreError::Conflict) if attempt + 1 < WRITE_RETRIES => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::conflict())
}

fn save(
    store: &Store,
    host: &Host,
    note: &Note,
    annotations: &Annotations,
    quote: Option<(&str, &str)>,
) -> Result<(String, String), StoreError> {
    let quoted = quote
        .and_then(|(id, name)| annotations.quote_edit(&note.body, id, name))
        .map(|edit| edit.apply(&note.body))
        .unwrap_or_else(|| note.body.clone());
    let edit = annotations
        .save_edit(&quoted)
        .map_err(|_| StoreError::Conflict)?;
    let body = match edit {
        Some(edit) => edit.apply(&quoted),
        None if quoted != note.body => quoted,
        None => {
            return Ok((note.rev.clone().unwrap_or_default(), note.id.clone()));
        }
    };
    save_body(store, host, note, body)
}

fn save_body(
    store: &Store,
    host: &Host,
    note: &Note,
    body: String,
) -> Result<(String, String), StoreError> {
    let mut fields = serde_json::Map::new();
    fields.insert("_id".into(), note.id.clone().into());
    fields.insert("_rev".into(), note.rev.clone().unwrap_or_default().into());
    fields.insert("body".into(), body.into());
    let body = fields
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if note.extra.contains_key(kanso_core::model::SHARE_FIELD) {
        let Some(host) = host else {
            return Err(StoreError::Conflict);
        };
        if !host
            .edit_note_body(&note.id, &note.title, body)
            .map_err(|_| StoreError::Conflict)?
        {
            return Err(StoreError::Conflict);
        }
        let latest = store
            .note(&note.id)
            .map_err(|_| StoreError::Conflict)?
            .ok_or(StoreError::Conflict)?;
        return Ok((latest.rev.unwrap_or_default(), latest.id));
    }
    let result = store.put_keeping(
        DocType::Note,
        Value::Object(fields),
        Put::Merge,
        Origin::Api,
        &api::app_only_names(DocType::Note),
    )?;
    Ok((result.rev, result.id))
}

fn worker_page(
    store: &Store,
    file_id: &str,
    expected_md5: &str,
    page: u32,
) -> ApiResult<kanso_pdf::PageText> {
    let (_, attachment) = store.attachment(file_id)?.ok_or_else(ApiError::not_found)?;
    if attachment.digest() != expected_md5 {
        return Err(ApiError::conflict());
    }
    let paths = locate().ok_or_else(|| ApiError::internal("The PDF reader is unavailable."))?;
    let mut worker =
        Worker::start(&paths).map_err(|_| ApiError::internal("The PDF reader is unavailable."))?;
    let opened = worker
        .open(&attachment.data, None)
        .map_err(|_| ApiError::bad_request("This PDF could not be opened."))?;
    if page > opened.pages {
        return Err(ApiError::not_found());
    }
    let text = worker
        .text(page - 1)
        .map_err(|_| ApiError::bad_request("This PDF page could not be read."))?;
    if !text.chars.is_empty() {
        return Ok(text);
    }
    // Scans have no PDF text layer. Use the synced OCR word boxes the
    // reader uses so quotes on another device resolve to the same quads.
    store
        .pdf_ocr(file_id)?
        .and_then(|ocr| {
            ocr.pages
                .into_iter()
                .find(|page_text| page_text.page == page)
        })
        .map(|page| page.to_page_text())
        .ok_or_else(|| ApiError::bad_request("That PDF page has no selectable text."))
}

fn parse_color(value: &str) -> ApiResult<Color> {
    match value {
        "yellow" => Ok(Color::Yellow),
        "green" => Ok(Color::Green),
        "blue" => Ok(Color::Blue),
        "pink" => Ok(Color::Pink),
        _ => Err(ApiError::bad_request("Choose yellow, green, blue or pink.")),
    }
}
fn valid_highlight_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}
fn pdf_error(error: kanso_pdf::PdfError) -> ApiError {
    ApiError::bad_request(error.to_string())
}
fn row_json(row: Row) -> Value {
    json!({"id": row.id, "page": row.page, "color": format!("{:?}", row.color).to_lowercase(), "text": row.text, "comment": row.comment, "inNote": row.in_note})
}
fn folded(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).collect()
}
fn excerpt(text: &str, needle: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut folded = Vec::new();
    let mut owners = Vec::new();
    for (index, ch) in chars.iter().enumerate() {
        for part in ch.to_lowercase() {
            folded.push(part);
            owners.push(index);
        }
    }
    let query: Vec<char> = needle.chars().flat_map(char::to_lowercase).collect();
    let Some(start) = folded
        .windows(query.len())
        .position(|window| window == query)
    else {
        return chars.iter().take(240).collect();
    };
    let Some(&first) = owners.get(start) else {
        return String::new();
    };
    let Some(&last) = owners.get(start + query.len().saturating_sub(1)) else {
        return String::new();
    };
    let from = first.saturating_sub(80);
    let to = (last + 120).min(chars.len());
    chars[from..to].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::roles::Role;
    use kanso_pdf::model::Quad;
    use kanso_pdf::text::model::HighlightParts;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Unicode case folding and excerpt bounds never produce an invalid
        /// slice or an oversized response.
        #[test]
        fn excerpts_are_bounded_for_arbitrary_text(text in any::<String>(), query in ".{1,32}") {
            let excerpted = excerpt(&text, &query);
            prop_assert!(excerpted.chars().count() <= 240);
            if folded(&text).contains(&folded(&query)) {
                prop_assert!(folded(&excerpted).contains(&folded(&query)));
            }
        }
    }

    #[test]
    fn an_annotation_is_one_note_revision_and_round_trips_to_the_reader_block() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(directory.path().join("kanso.db")).unwrap();
        let book = store.role_book(Role::Inbox, Origin::Api).unwrap();
        let note_id = store
            .put(
                DocType::Note,
                json!({"title":"Report", "body":"", "bookId":book}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
            .id;
        let note = store.note(&note_id).unwrap().unwrap();
        let before = store.last_seq().unwrap();
        let mut annotations = Annotations::new("file:report", "md5-test");
        let id = annotations
            .add(
                HighlightParts {
                    quads: std::collections::BTreeMap::from([(
                        4,
                        vec![Quad::new(0.1, 0.1, 0.3, 0.2)],
                    )]),
                    text: "Revenue rose".into(),
                },
                Color::Yellow,
                1,
            )
            .unwrap();
        annotations
            .set_comment(&id, Some("Check the figure"))
            .unwrap();

        save(
            &store,
            &None,
            &note,
            &annotations,
            Some((&id, "report.pdf")),
        )
        .unwrap();

        assert_eq!(store.last_seq().unwrap(), before + 1);
        let saved = store.note(&note_id).unwrap().unwrap();
        let reader = Annotations::from_body(&saved.body, "file:report", "md5-test");
        assert_eq!(reader.get(&id).unwrap().text, "Revenue rose");
        assert_eq!(
            reader.get(&id).unwrap().comment.as_deref(),
            Some("Check the figure")
        );
        assert_eq!(reader.get(&id).unwrap().quads[&4].len(), 1);
        assert!(reader.rows(&saved.body)[0].in_note);
    }
}
