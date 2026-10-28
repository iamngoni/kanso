//! Capture: what a quick note typed in a small window, and what something
//! shared from another app, becomes in the library. The desktop's Quick
//! Capture and command line, and the phones' share sheets, widgets and
//! shortcuts, all end here.
//!
//! [`split_capture`] gives typed text its title. [`save_capture`] saves it
//! as a note. A share ([`Shared`]: a subject, text, a link and files) is
//! composed into a title and a body by [`compose`] and saved by
//! [`save_shared`], whose note and attachments have IDs made from the
//! share's own, so saving one twice makes one note.
//!
//! Nothing is ever written into a locked notebook: a note typed there would
//! sit in plain text under a lock, so both saves refuse
//! ([`CaptureError::Locked`]).

use std::fmt;

use kanso_core::DocType;
use kanso_core::links::LINK_SCHEME;
use kanso_core::model::MAX_BODY_LEN;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::attach::{file_stem, link_text, prepare_and_store};

/// A first line longer than this is cut for the title.
pub const TITLE_WORDS_LEN: usize = 80;

/// Most files a share may bring; later ones are left out.
pub const MAX_SHARED_FILES: usize = 20;

/// The title of a share with nothing to name it by.
const UNTITLED: &str = "Shared note";

/// Whether `text` is a single web address.
pub fn is_url(text: &str) -> bool {
    let text = text.trim();
    let rest = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .or_else(|| text.strip_prefix("mailto:"));
    rest.is_some_and(|rest| !rest.is_empty() && !rest.contains(char::is_whitespace))
        || text.starts_with(LINK_SCHEME)
            && !text.contains(char::is_whitespace)
            && text.len() > LINK_SCHEME.len()
}

/// Why a capture wasn't saved.
#[derive(Debug)]
pub enum CaptureError {
    /// The notebook it would go in is locked, or inside one that is.
    Locked,
    /// A share with no text, no link and no image or PDF that could be
    /// attached. Carries what was said about the files that were left out.
    Empty(Vec<String>),
    /// A share's ID that isn't 32 lowercase hex characters.
    BadId,
    /// The store failed.
    Store(StoreError),
}

impl CaptureError {
    /// Whether trying again can't change the result: the notebook is locked,
    /// the share is empty or has no valid ID, or the store refused what was
    /// asked as invalid. A busy database or a disk error is worth another
    /// try.
    pub fn is_final(&self) -> bool {
        match self {
            CaptureError::Locked | CaptureError::Empty(_) | CaptureError::BadId => true,
            CaptureError::Store(err) => matches!(err, StoreError::Invalid(_)),
        }
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::Locked => {
                f.write_str("That notebook is locked, so nothing was saved into it.")
            }
            CaptureError::Empty(_) => {
                f.write_str("There was nothing to save: no text, link, image or PDF.")
            }
            CaptureError::BadId => f.write_str("That share has no valid ID."),
            CaptureError::Store(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CaptureError::Store(err) => Some(err),
            _ => None,
        }
    }
}

impl From<StoreError> for CaptureError {
    fn from(err: StoreError) -> CaptureError {
        CaptureError::Store(err)
    }
}

/// A capture's title and body. A heading or a short first line is the
/// title, and the lines after it the body. A long first line, or one that
/// starts a list or a quote, is cut short for the title and the body keeps
/// all of the text, so nothing typed is lost. `None` for text that is only
/// spaces. Pure.
pub fn split_capture(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let (first, rest) = text.split_once('\n').unwrap_or((text, ""));
    let first = first.trim();
    let rest = without_blank_lead(rest.trim_end());
    let heading = first
        .strip_prefix('#')
        .map(|h| h.trim_start_matches('#'))
        .filter(|h| h.starts_with(' '))
        .map(str::trim)
        .filter(|h| !h.is_empty());
    if let Some(heading) = heading {
        return Some((cut_title(heading), rest.to_owned()));
    }
    let marker = first.starts_with(['-', '*', '>', '+', '|', '`', '#']);
    if !marker && first.chars().count() <= TITLE_WORDS_LEN {
        return Some((first.to_owned(), rest.to_owned()));
    }
    let plain = first.trim_start_matches(['-', '*', '>', '+', '|', '`', '#', ' ']);
    let plain = if plain.is_empty() { first } else { plain };
    Some((cut_title(plain), text.to_owned()))
}

/// `text` from its first line with more than spaces in it.
fn without_blank_lead(mut text: &str) -> &str {
    while let Some((line, rest)) = text.split_once('\n') {
        if !line.trim().is_empty() {
            break;
        }
        text = rest;
    }
    if text.trim().is_empty() { "" } else { text }
}

/// At most [`TITLE_WORDS_LEN`] characters, cut at a space, with "…" when
/// cut.
fn cut_title(line: &str) -> String {
    if line.chars().count() <= TITLE_WORDS_LEN {
        return line.to_owned();
    }
    let cut: String = line.chars().take(TITLE_WORDS_LEN).collect();
    let cut = match cut.rfind(' ') {
        Some(at) if at > TITLE_WORDS_LEN / 2 => &cut[..at],
        _ => cut.as_str(),
    };
    format!("{}…", cut.trim_end())
}

/// Saves `text` as a note in `book`, or the Inbox (made if needed).
/// Returns the note's ID, or `None` for empty text. Refuses a locked
/// notebook ([`CaptureError::Locked`]), the Inbox included.
pub fn save_capture(
    store: &Store,
    text: &str,
    book: Option<&str>,
    origin: Origin,
) -> Result<Option<String>, CaptureError> {
    let Some((title, body)) = split_capture(text) else {
        return Ok(None);
    };
    let book = match book {
        Some(book) if store.get(book)?.is_some() => book.to_owned(),
        _ => store.role_book(Role::Inbox, origin)?,
    };
    if store.locked_books()?.contains(&book) {
        return Err(CaptureError::Locked);
    }
    let created = store.put(
        DocType::Note,
        json!({"title": title, "body": body, "bookId": book}),
        Put::Upsert,
        origin,
    )?;
    Ok(Some(created.id))
}

/// Something shared into Kanso from another app. Every part may be missing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shared {
    /// The share's subject, or the page's title.
    pub subject: Option<String>,
    /// Shared text.
    pub text: Option<String>,
    /// A web address.
    pub url: Option<String>,
    pub files: Vec<SharedFile>,
}

/// A file that came with a share.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFile {
    /// The file's name as the sharing app has it.
    pub name: String,
    /// The content type the sharing app gave. Only a hint: the file's
    /// extension and first bytes decide what it is.
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

/// What [`save_shared`] made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Captured {
    /// The note's ID (`note:cap-…`).
    pub id: String,
    /// What was left out and why, a sentence each: files that weren't images
    /// or PDFs or were too large, files past [`MAX_SHARED_FILES`], text cut
    /// to fit a note. Empty when `existed`.
    pub problems: Vec<String>,
    /// The note was already there from an earlier save of this share, and
    /// nothing was written.
    pub existed: bool,
}

/// A part of a share without its surrounding spaces, if anything is left.
fn some(part: &Option<String>) -> Option<&str> {
    part.as_deref()
        .map(str::trim)
        .filter(|part| !part.is_empty())
}

/// A share's title and body, with `attachments` (the Markdown of the files
/// that were attached) each a paragraph of its own at the end. `None` when
/// there is nothing in it. Pure.
///
/// The title is the subject, else the first line of the text
/// ([`split_capture`]), else the link's host and path, else the first
/// file's name, else "Shared note"; at most [`TITLE_WORDS_LEN`] characters
/// and "…", never empty. The body is the rest of the text, then the link,
/// as `[subject](address)` when there is a subject and as `<address>` when
/// not, then the attachments. Text that is only an address is the link.
pub fn compose(shared: &Shared, attachments: &[String]) -> Option<(String, String)> {
    let subject = some(&shared.subject);
    let mut text = some(&shared.text);
    let mut url = some(&shared.url);
    if let Some(only) = text.filter(|text| is_url(text)) {
        match url {
            None => {
                url = Some(only);
                text = None;
            }
            Some(url) if url == only => text = None,
            Some(_) => {}
        }
    }
    if subject.is_none() && text.is_none() && url.is_none() && attachments.is_empty() {
        return None;
    }

    let mut paragraphs: Vec<String> = Vec::new();
    let title = if let Some(subject) = subject {
        paragraphs.extend(text.map(str::to_owned));
        cut_title(subject.lines().find(|l| !l.trim().is_empty())?.trim())
    } else if let Some((title, rest)) = text.and_then(split_capture) {
        paragraphs.push(rest);
        title
    } else if let Some(title) = url.map(url_title) {
        title
    } else if let Some(file) = shared.files.first() {
        cut_title(file_stem(&file.name).trim())
    } else {
        UNTITLED.to_owned()
    };
    let title = if title.trim().is_empty() {
        UNTITLED.to_owned()
    } else {
        title
    };
    if let Some(url) = url {
        paragraphs.push(link_markdown(subject, url));
    }
    paragraphs.extend(attachments.iter().cloned());
    paragraphs.retain(|paragraph| !paragraph.trim().is_empty());
    Some((title, paragraphs.join("\n\n")))
}

/// A title for a note that is a link: the address without its scheme, its
/// query and `www.`, e.g. "example.com/a/b".
fn url_title(url: &str) -> String {
    let url = url.lines().next().unwrap_or_default().trim();
    let rest = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .or_else(|| url.strip_prefix("mailto:"))
        .unwrap_or(url);
    let rest = rest.split(['?', '#']).next().unwrap_or_default();
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let rest = rest.trim_end_matches('/');
    if rest.is_empty() {
        UNTITLED.to_owned()
    } else {
        cut_title(rest)
    }
}

/// The link to a shared address: `[subject](address)`, or the bare
/// `<address>` when there is no subject, or it is the address itself.
fn link_markdown(subject: Option<&str>, url: &str) -> String {
    let target: String = url
        .chars()
        .flat_map(|c| match c {
            c if c.is_whitespace() => "%20".chars().collect::<Vec<_>>(),
            '(' => "%28".chars().collect(),
            ')' => "%29".chars().collect(),
            '<' => "%3C".chars().collect(),
            '>' => "%3E".chars().collect(),
            c => vec![c],
        })
        .collect();
    match subject {
        Some(subject) if subject != url => {
            let subject: Vec<&str> = subject.split_whitespace().collect();
            format!("[{}]({target})", link_text(&subject.join(" ")))
        }
        _ if is_url(url) => format!("<{target}>"),
        _ => url.to_owned(),
    }
}

/// Whether `id` is a share's ID: 32 lowercase hex characters, which is also
/// the name of its folder on a phone.
pub fn is_drop_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Saves a share as a note in the Inbox (made if needed), with its images
/// and PDFs attached. `drop_id` is the share's ID, 32 lowercase hex
/// characters: the note is `note:cap-<drop_id>` and its files
/// `file:cap-<drop_id>-<n>`, `n` counting from 0 over the share's files. A
/// share already saved (its note is there) is returned as it is and nothing
/// is written, so saving again after a crash is safe.
///
/// A file that isn't an image or a PDF, is too large or is past
/// [`MAX_SHARED_FILES`] is left out and named in
/// [`Captured::problems`]. A locked Inbox refuses
/// ([`CaptureError::Locked`]) before anything is written, and a share with
/// nothing to save is an error ([`CaptureError::Empty`]).
pub fn save_shared(
    store: &Store,
    shared: &Shared,
    drop_id: &str,
    origin: Origin,
) -> Result<Captured, CaptureError> {
    if !is_drop_id(drop_id) {
        return Err(CaptureError::BadId);
    }
    let note_id = format!("note:cap-{drop_id}");
    let saved = |store: &Store| -> Result<Option<Captured>, StoreError> {
        Ok(store.get(&note_id)?.map(|_| Captured {
            id: note_id.clone(),
            problems: Vec::new(),
            existed: true,
        }))
    };
    if let Some(captured) = saved(store)? {
        return Ok(captured);
    }
    if compose(shared, &[]).is_none() && shared.files.is_empty() {
        return Err(CaptureError::Empty(Vec::new()));
    }
    let inbox = store.role_book(Role::Inbox, origin)?;
    if store.locked_books()?.contains(&inbox) {
        return Err(CaptureError::Locked);
    }

    let mut problems = Vec::new();
    let mut attachments = Vec::new();
    for (n, file) in shared.files.iter().enumerate() {
        let label = if file.name.trim().is_empty() {
            "A file"
        } else {
            file.name.trim()
        };
        if n >= MAX_SHARED_FILES {
            if n == MAX_SHARED_FILES {
                let left = shared.files.len() - MAX_SHARED_FILES;
                problems.push(format!(
                    "A share can have up to {MAX_SHARED_FILES} files: the last {left} weren't attached."
                ));
            }
            continue;
        }
        let id = format!("file:cap-{drop_id}-{n}");
        let hint = file.content_type.as_deref();
        match prepare_and_store(store, &file.bytes, &file.name, hint, &id) {
            Ok(written) => attachments.push(written?.markdown),
            Err(reason) => problems.push(format!("{label}: {reason}")),
        }
    }

    let Some((title, mut body)) = compose(shared, &attachments) else {
        return Err(CaptureError::Empty(problems));
    };
    if body.chars().count() > MAX_BODY_LEN {
        body = body.chars().take(MAX_BODY_LEN).collect();
        problems.push("The shared text was too long for a note and was cut.".into());
    }
    let put = store.put(
        DocType::Note,
        json!({"_id": note_id, "title": title, "body": body, "bookId": inbox}),
        Put::Upsert,
        origin,
    );
    match put {
        Ok(written) => Ok(Captured {
            id: written.id,
            problems,
            existed: false,
        }),
        // Another save of this share got there first.
        Err(StoreError::Conflict) => saved(store)?.ok_or(CaptureError::Store(StoreError::Conflict)),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::model::MAX_TITLE_LEN;
    use proptest::prelude::*;

    #[test]
    fn urls() {
        assert!(is_url("https://kanso.app"));
        assert!(is_url("  http://a.b/c?d=e  "));
        assert!(is_url("inkdrop://note/abc"));
        assert!(!is_url("https://"));
        assert!(!is_url("see https://kanso.app"));
        assert!(!is_url("kanso.app"));
    }

    #[test]
    fn split_examples() {
        let s = |t: &str| split_capture(t);
        assert_eq!(s("  \n "), None);
        assert_eq!(s("Buy milk"), Some(("Buy milk".into(), String::new())));
        assert_eq!(
            s("# Idea\n\nA longer thought.\n"),
            Some(("Idea".into(), "A longer thought.".into()))
        );
        assert_eq!(
            s("Call Sam\nabout the lease"),
            Some(("Call Sam".into(), "about the lease".into()))
        );
        assert_eq!(
            s("- milk\n- eggs"),
            Some(("milk".into(), "- milk\n- eggs".into()))
        );
        let long = "word ".repeat(40);
        let (title, body) = s(&long).unwrap();
        assert!(title.ends_with('…') && title.chars().count() <= TITLE_WORDS_LEN + 1);
        assert_eq!(body, long.trim());
        let (title, body) = s(&format!("{long}\nmore")).unwrap();
        assert!(title.ends_with('…'));
        assert!(body.starts_with("word word"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariants: nothing typed is lost (every word is in the title or
        /// the body), the title is one line a note can take, and a body
        /// never starts with a blank line or ends with spaces.
        #[test]
        fn prop_split_keeps_every_word(text in "[a-z#*> \n-]{0,200}") {
            match split_capture(&text) {
                None => prop_assert!(text.trim().is_empty()),
                Some((title, body)) => {
                    prop_assert!(!title.contains('\n'));
                    prop_assert!(title.chars().count() <= MAX_TITLE_LEN);
                    prop_assert_eq!(body.trim_end(), body.as_str());
                    prop_assert!(body.lines().next().is_none_or(|line| !line.trim().is_empty()));
                    let kept = format!("{title} {body}");
                    for word in text.split_whitespace() {
                        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
                        prop_assert!(word.is_empty() || kept.contains(word), "{:?} lost from {:?}", word, text);
                    }
                }
            }
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    /// Writes a notebook: a name, a parent and whether it is locked.
    fn put_book(store: &Store, id: &str, parent: Option<&str>, locked: bool) -> String {
        let mut book = json!({"_id": id, "name": id, "parentBookId": parent});
        if locked {
            book["kansoLock"] =
                json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
        }
        // A notebook that is there already is changed, not replaced.
        let mode = if store.get(id).unwrap().is_some() {
            Put::Merge
        } else {
            Put::Upsert
        };
        store
            .put(DocType::Book, book, mode, Origin::Sync)
            .unwrap()
            .id
    }

    /// Saving puts the note in the Inbox, made on first use, or in the
    /// notebook chosen.
    #[test]
    fn captures_go_to_the_inbox() {
        let (_dir, store) = store();
        assert_eq!(save_capture(&store, " \n", None, Origin::Ui).unwrap(), None);
        let id = save_capture(&store, "Buy milk\n- and eggs", None, Origin::Ui)
            .unwrap()
            .unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(
            (
                note.title.as_str(),
                note.body.as_str(),
                note.book_id.as_str()
            ),
            ("Buy milk", "- and eggs", "book:inbox")
        );
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Ideas"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let id = save_capture(&store, "Kites", Some(&book), Origin::Ui)
            .unwrap()
            .unwrap();
        assert_eq!(store.note(&id).unwrap().unwrap().book_id, book);
        // A notebook deleted meanwhile: the Inbox instead.
        let id = save_capture(&store, "Late", Some("book:gone"), Origin::Ui)
            .unwrap()
            .unwrap();
        assert_eq!(store.note(&id).unwrap().unwrap().book_id, "book:inbox");
    }

    /// A locked notebook, or one inside a locked notebook, takes nothing.
    #[test]
    fn captures_are_refused_by_locked_notebooks() {
        let (_dir, store) = store();
        let outer = put_book(&store, "book:outer", None, true);
        let inner = put_book(&store, "book:inner", Some(&outer), false);
        let open = put_book(&store, "book:open", None, false);
        for book in [&outer, &inner] {
            let err = save_capture(&store, "Secret", Some(book), Origin::Ui).unwrap_err();
            assert!(matches!(err, CaptureError::Locked), "{err}");
            assert_eq!(
                err.to_string(),
                "That notebook is locked, so nothing was saved into it."
            );
        }
        assert!(store.notes_in_books(&[outer, inner]).unwrap().is_empty());
        assert!(save_capture(&store, "Fine", Some(&open), Origin::Ui).is_ok());
        // The Inbox itself, locked: even the default is refused.
        put_book(&store, "book:inbox", None, true);
        let err = save_capture(&store, "Secret", None, Origin::Ui).unwrap_err();
        assert!(matches!(err, CaptureError::Locked), "{err}");
    }

    fn shared(subject: Option<&str>, text: Option<&str>, url: Option<&str>) -> Shared {
        Shared {
            subject: subject.map(str::to_owned),
            text: text.map(str::to_owned),
            url: url.map(str::to_owned),
            files: Vec::new(),
        }
    }

    fn file(name: &str, bytes: &[u8]) -> SharedFile {
        SharedFile {
            name: name.into(),
            content_type: None,
            bytes: bytes.to_vec(),
        }
    }

    fn composed(shared: &Shared, attachments: &[&str]) -> Option<(String, String)> {
        let attachments: Vec<String> = attachments.iter().map(|a| (*a).to_owned()).collect();
        compose(shared, &attachments)
    }

    #[test]
    fn compose_examples() {
        let link = "https://example.com/a/b?x=1";
        // A page shared with its title.
        assert_eq!(
            composed(&shared(Some("A page"), None, Some(link)), &[]),
            Some(("A page".into(), format!("[A page]({link})")))
        );
        // The same, with the page's address as the text.
        assert_eq!(
            composed(&shared(Some("A page"), Some(link), None), &[]),
            Some(("A page".into(), format!("[A page]({link})")))
        );
        // No subject: the link's host and path.
        assert_eq!(
            composed(&shared(None, None, Some(link)), &[]),
            Some(("example.com/a/b".into(), format!("<{link}>")))
        );
        assert_eq!(
            composed(&shared(None, Some(" https://www.kanso.app/ "), None), &[]),
            Some(("kanso.app".into(), "<https://www.kanso.app/>".into()))
        );
        // Text with a link: the first line is the title, the link follows.
        assert_eq!(
            composed(
                &shared(None, Some("Read this\nit's good"), Some(link)),
                &["![](inkdrop://file:a)"]
            ),
            Some((
                "Read this".into(),
                format!("it's good\n\n<{link}>\n\n![](inkdrop://file:a)")
            ))
        );
        // A subject that is the address makes a bare link, and all the text stays.
        assert_eq!(
            composed(&shared(Some(link), Some("Notes\nmore"), Some(link)), &[]),
            Some((link.into(), format!("Notes\nmore\n\n<{link}>")))
        );
        // A subject with brackets and lines, and an address with spaces.
        assert_eq!(
            composed(
                &shared(Some("[a]\n b"), None, Some("https://x.io/a b(1)")),
                &[]
            ),
            Some((
                "[a]".into(),
                "[\\[a\\] b](https://x.io/a%20b%281%29)".into()
            ))
        );
        // Only files: the first one's name, then the generic title.
        let mut only_files = Shared::default();
        only_files.files.push(file("Report 2026.pdf", b""));
        assert_eq!(
            composed(&only_files, &["[Report 2026.pdf](inkdrop://file:a)"]),
            Some((
                "Report 2026".into(),
                "[Report 2026.pdf](inkdrop://file:a)".into()
            ))
        );
        assert_eq!(
            composed(&Shared::default(), &["![](inkdrop://file:a)"]),
            Some(("Shared note".into(), "![](inkdrop://file:a)".into()))
        );
        // Nothing.
        assert_eq!(
            composed(&shared(Some(" "), Some("\n"), Some("")), &[]),
            None
        );
        // A long first line is cut for the title, and the body keeps it whole.
        let long = "word ".repeat(40);
        let (title, body) = composed(&shared(None, Some(&long), None), &[]).unwrap();
        assert!(title.ends_with('…'));
        assert_eq!(body, long.trim());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariants: whatever is shared, if the title isn't empty it is
        /// at most 81 characters on one line, none of the text or the
        /// address is lost, the attachments come last, each its own
        /// paragraph, and a share with something in it always composes.
        #[test]
        fn prop_compose_loses_nothing(
            subject in prop::option::of("[a-zA-Z0-9 \\[\\]\n]{0,120}"),
            text in prop::option::of("[a-z#*> \n-]{0,200}"),
            url in prop::option::of("https://[a-z]{1,8}\\.[a-z]{2,3}(/[a-z0-9]{0,6}){0,3}"),
            files in prop::collection::vec("[a-zA-Z0-9 ._]{0,30}", 0..3),
            attachments in prop::collection::vec("!\\[\\]\\(inkdrop://file:[a-z0-9]{1,8}\\)", 0..3),
        ) {
            let shared = Shared {
                subject: subject.clone(),
                text: text.clone(),
                url: url.clone(),
                files: files.iter().map(|name| file(name, b"")).collect(),
            };
            let blank = |part: &Option<String>| part.as_deref().is_none_or(|p| p.trim().is_empty());
            let nothing = blank(&subject) && blank(&text) && blank(&url) && attachments.is_empty();
            let Some((title, body)) = compose(&shared, &attachments) else {
                prop_assert!(nothing);
                return Ok(());
            };
            prop_assert!(!nothing);
            prop_assert!(!title.trim().is_empty());
            prop_assert!(title.chars().count() <= TITLE_WORDS_LEN + 1, "{title:?}");
            prop_assert!(!title.contains('\n'));
            if let Some(url) = url.as_deref().map(str::trim).filter(|u| !u.is_empty()) {
                prop_assert!(body.contains(url), "{url:?} lost from {body:?}");
            }
            if let Some(text) = text.as_deref() {
                let kept = format!("{title}\n{body}");
                for word in text.split_whitespace() {
                    let word = word.trim_matches(|c: char| !c.is_alphanumeric());
                    prop_assert!(word.is_empty() || kept.contains(word), "{word:?} lost from {text:?}");
                }
            }
            // The attachments end the body, in order, each its own paragraph.
            if !attachments.is_empty() {
                let tail = attachments.join("\n\n");
                prop_assert!(body.ends_with(&tail), "{body:?}");
                let head = &body[..body.len() - tail.len()];
                prop_assert!(head.is_empty() || head.ends_with("\n\n"));
            }
            prop_assert!(!body.starts_with('\n') && !body.ends_with('\n'));
        }
    }

    const PDF: &[u8] = b"%PDF-1.7\n%%EOF\n";
    const DROP: &str = "0123456789abcdef0123456789abcdef";

    fn png(seed: u8) -> Vec<u8> {
        let image =
            image::RgbaImage::from_fn(2, 2, |x, y| image::Rgba([seed, x as u8, y as u8, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut out, image::ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn a_share_becomes_a_note_with_its_files() {
        let (_dir, store) = store();
        let mut share = shared(Some("Trip"), Some("Packing list"), Some("https://t.io/x"));
        share.files = vec![
            file("beach.png", &png(1)),
            file("notes.txt", b"hello"),
            file("tickets.pdf", PDF),
        ];
        let saved = save_shared(&store, &share, DROP, Origin::Ui).unwrap();
        assert_eq!(saved.id, format!("note:cap-{DROP}"));
        assert!(!saved.existed);
        assert_eq!(
            saved.problems,
            vec!["notes.txt: Only images and PDFs can be attached: PNG, JPEG, GIF, SVG, HEIC, WebP, TIFF, BMP or PDF.".to_owned()]
        );
        let note = store.note(&saved.id).unwrap().unwrap();
        assert_eq!(note.title, "Trip");
        assert_eq!(note.book_id, "book:inbox");
        assert_eq!(
            note.body,
            format!(
                "Packing list\n\n[Trip](https://t.io/x)\n\n![](inkdrop://file:cap-{DROP}-0)\n\n[tickets.pdf](inkdrop://file:cap-{DROP}-2)"
            )
        );
        let (content_type, _) = store
            .attachment(&format!("file:cap-{DROP}-2"))
            .unwrap()
            .unwrap();
        assert_eq!(content_type, "application/pdf");
        assert!(store.get(&format!("file:cap-{DROP}-1")).unwrap().is_none());
    }

    #[test]
    fn saving_a_share_again_writes_nothing() {
        let (_dir, store) = store();
        let mut share = shared(None, Some("Hi"), None);
        share.files = vec![file("a.png", &png(2))];
        let first = save_shared(&store, &share, DROP, Origin::Ui).unwrap();
        let rev = store.get(&first.id).unwrap().unwrap().rev;
        let second = save_shared(&store, &share, DROP, Origin::Ui).unwrap();
        assert_eq!(second.id, first.id);
        assert!(second.existed && second.problems.is_empty());
        assert_eq!(store.get(&first.id).unwrap().unwrap().rev, rev);

        // A crash between the files and the note: the files are there, the
        // note isn't. The retry makes the note, over the same files.
        let other = "f".repeat(32);
        let file_id = format!("file:cap-{other}-0");
        crate::attach::attach_image_as(&store, &png(2), "png", Some("a.png"), Some(&file_id))
            .unwrap();
        let healed = save_shared(&store, &share, &other, Origin::Ui).unwrap();
        let note = store.note(&healed.id).unwrap().unwrap();
        assert!(note.body.contains(&file_id), "{}", note.body);
    }

    #[test]
    fn refusals_say_why() {
        let (_dir, store) = store();
        let err =
            save_shared(&store, &shared(None, Some("x"), None), "ABC", Origin::Ui).unwrap_err();
        assert_eq!(err.to_string(), "That share has no valid ID.");
        assert!(err.is_final());

        let err = save_shared(&store, &Shared::default(), DROP, Origin::Ui).unwrap_err();
        assert_eq!(
            err.to_string(),
            "There was nothing to save: no text, link, image or PDF."
        );
        assert!(store.get("book:inbox").unwrap().is_none());

        // Only a file that can't be attached: the same, with the reason.
        let share = Shared {
            files: vec![file("a.zip", b"PK"), file("", &[0u8; 3])],
            ..Shared::default()
        };
        let CaptureError::Empty(problems) =
            save_shared(&store, &share, DROP, Origin::Ui).unwrap_err()
        else {
            panic!("expected an empty share");
        };
        assert_eq!(problems.len(), 2);
        assert!(
            problems[0].starts_with("a.zip: Only images and PDFs"),
            "{problems:?}"
        );
        assert!(problems[1].starts_with("A file: "), "{problems:?}");
        assert!(
            store
                .notes_in_books(&["book:inbox".into()])
                .unwrap()
                .is_empty()
        );

        // A locked Inbox takes nothing: no note, no files.
        put_book(&store, "book:inbox", None, true);
        let mut share = shared(None, Some("Secret"), None);
        share.files = vec![file("a.png", &png(3))];
        let err = save_shared(&store, &share, DROP, Origin::Ui).unwrap_err();
        assert!(
            matches!(err, CaptureError::Locked) && err.is_final(),
            "{err}"
        );
        assert!(store.get(&format!("note:cap-{DROP}")).unwrap().is_none());
        assert!(store.get(&format!("file:cap-{DROP}-0")).unwrap().is_none());
    }

    #[test]
    fn too_many_and_too_large_files_are_left_out() {
        let (_dir, store) = store();
        let mut share = shared(Some("Photos"), None, None);
        let pic = png(4);
        share.files = (0..23).map(|n| file(&format!("{n}.png"), &pic)).collect();
        let saved = save_shared(&store, &share, DROP, Origin::Ui).unwrap();
        assert_eq!(
            saved.problems,
            vec!["A share can have up to 20 files: the last 3 weren't attached.".to_owned()]
        );
        let body = store.note(&saved.id).unwrap().unwrap().body;
        assert_eq!(body.matches("inkdrop://file:").count(), 20);

        let drop = "e".repeat(32);
        let mut big = b"%PDF-".to_vec();
        big.resize(10 * 1024 * 1024 + 1, 0);
        let mut share = shared(Some("Big"), None, None);
        share.files = vec![file("big.pdf", &big)];
        let saved = save_shared(&store, &share, &drop, Origin::Ui).unwrap();
        assert_eq!(
            saved.problems,
            vec!["big.pdf: That PDF is 10 MB. Attachments can be up to 10 MB.".to_owned()]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        /// Invariant: saving a share twice, with the same ID, leaves one
        /// note and one set of files, with the same IDs and nothing
        /// rewritten.
        #[test]
        fn prop_saving_twice_makes_one_note(
            drop_id in "[0-9a-f]{32}",
            subject in prop::option::of("[a-z ]{1,20}"),
            text in prop::option::of("[a-z \n]{0,40}"),
            names in prop::collection::vec(prop::sample::select(vec!["a.png", "b.pdf", "c.txt", "d.jpg"]), 0..4),
        ) {
            let (_dir, store) = store();
            let share = Shared {
                subject,
                text,
                url: None,
                files: names.iter().map(|name| match name.rsplit('.').next() {
                    Some("png") => file(name, &png(7)),
                    Some("pdf") => file(name, PDF),
                    _ => file(name, b"junk"),
                }).collect(),
            };
            let Ok(first) = save_shared(&store, &share, &drop_id, Origin::Ui) else {
                // Nothing to save: and nothing was made.
                let note_id = format!("note:cap-{drop_id}");
                prop_assert!(store.get(&note_id).unwrap().is_none());
                return Ok(());
            };
            let docs = |store: &Store| -> Vec<(String, String)> {
                let mut ids = vec![format!("note:cap-{drop_id}")];
                ids.extend((0..names.len()).map(|n| format!("file:cap-{drop_id}-{n}")));
                ids.iter()
                    .filter_map(|id| store.get(id).unwrap())
                    .map(|doc| (doc.id, doc.rev))
                    .collect()
            };
            let before = docs(&store);
            let second = save_shared(&store, &share, &drop_id, Origin::Ui).unwrap();
            prop_assert_eq!(&second.id, &first.id);
            prop_assert!(second.existed);
            prop_assert_eq!(docs(&store), before);
            let in_inbox = store.notes_in_books(&["book:inbox".into()]).unwrap();
            prop_assert_eq!(in_inbox.len(), 1);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Invariant: over notebook trees with locks anywhere, a capture is
        /// saved in a notebook exactly when neither it nor a notebook above
        /// it is locked, and when refused nothing at all is written.
        #[test]
        fn prop_nothing_is_written_into_a_locked_notebook(
            // For each of 5 notebooks: its parent (an earlier one) and its lock.
            tree in prop::collection::vec((prop::option::of(0usize..5), any::<bool>()), 5),
            target in 0usize..6,
            shared_inbox in any::<bool>(),
        ) {
            let (_dir, store) = store();
            let ids: Vec<String> = (0..5).map(|n| format!("book:b{n}")).collect();
            for (n, (parent, locked)) in tree.iter().enumerate() {
                // A parent must come before its child, so the tree has no cycle.
                let parent = parent.filter(|p| *p < n).map(|p| ids[p].as_str());
                put_book(&store, &ids[n], parent, *locked);
            }
            // The Inbox hangs from the first notebook, so it can be inside a lock.
            let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
            put_book(&store, &inbox, Some(&ids[0]), false);
            let book = if target < 5 { Some(ids[target].as_str()) } else { None };
            let target_book = book.unwrap_or(&inbox).to_owned();
            let all: Vec<String> = ids.iter().cloned().chain([inbox.clone()]).collect();
            let notes = |store: &Store| store.notes_in_books(&all).unwrap().len();
            let before = notes(&store);
            let locked = store.locked_books().unwrap().contains(&target_book);

            let typed = save_capture(&store, "Hello", book, Origin::Ui);
            let share = shared(None, Some("Hello"), None);
            let sent = if shared_inbox || book.is_none() {
                Some(save_shared(&store, &share, DROP, Origin::Ui))
            } else {
                None
            };
            let inbox_locked = store.locked_books().unwrap().contains(&inbox);
            prop_assert_eq!(typed.is_err(), locked);
            if locked {
                prop_assert!(matches!(typed, Err(CaptureError::Locked)));
            }
            if let Some(sent) = sent {
                prop_assert_eq!(sent.is_err(), inbox_locked);
            }
            let made = usize::from(!locked) + usize::from(sent_ok(&store));
            prop_assert_eq!(notes(&store), before + made);
            for (id, locked_now) in ids.iter().map(|id| (id, store.locked_books().unwrap().contains(id))) {
                if locked_now {
                    prop_assert!(store.notes_in_books(std::slice::from_ref(id)).unwrap().is_empty(), "{id}");
                }
            }
            prop_assert!(!inbox_locked || store.notes_in_books(&[inbox]).unwrap().is_empty());
        }
    }

    /// Whether the share with `DROP` was saved.
    fn sent_ok(store: &Store) -> bool {
        store.get(&format!("note:cap-{DROP}")).unwrap().is_some()
    }
}
