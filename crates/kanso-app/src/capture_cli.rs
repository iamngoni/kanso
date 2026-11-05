//! `kanso capture "text"`: saves a note to the Inbox (or the notebook
//! `--book` names) from a terminal, a script or a desktop shortcut. With no
//! text (or `-`) it reads standard input, so `pbpaste | kanso capture`
//! works.
//!
//! While Kanso is open it holds the data folder, so the note goes to it over
//! the local link (`url_callback.rs`), which can only add notes, and shows
//! at once; no key is needed. With Kanso closed it is written to the store
//! directly. The title is the text's first line (`capture::split_capture`).

use std::io::Read as _;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store, lock_data_dir};
use serde_json::json;

use crate::capture::split_capture;
use crate::capture_request::CaptureRequest;
use crate::url_callback::{ForwardError, forward_capture};

pub const USAGE: &str = "\
Usage: kanso capture [--book <name or id>] [text…]

Saves a note to the Inbox, or to the notebook --book names. The first line
is its title. With no text, or `-`, the text is read from standard input.";

/// The notebook `wanted` names among `books` (`(id, name)`, outside the
/// trash): by ID, else by name ignoring case. An error says why none
/// matches, or that several share the name. Pure.
pub fn pick_book<'a>(
    books: &'a [(String, String)],
    wanted: &str,
) -> Result<&'a (String, String), String> {
    let wanted = wanted.trim();
    if let Some(book) = books.iter().find(|(id, _)| id == wanted) {
        return Ok(book);
    }
    let named: Vec<_> = books
        .iter()
        .filter(|(_, name)| name.trim().to_lowercase() == wanted.to_lowercase())
        .collect();
    match named.as_slice() {
        [book] => Ok(book),
        [] => Err(format!(
            "there's no notebook named “{wanted}”. Check its spelling, or leave --book off"
        )),
        several => Err(format!(
            "{} notebooks are named “{wanted}”; use the ID of one: {}",
            several.len(),
            several
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// `--book <name or id>` taken out of the arguments.
fn book_arg(rest: &[String]) -> Result<(Option<String>, Vec<String>)> {
    let mut book = None;
    let mut words = Vec::new();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        if arg == "--book" {
            book = Some(
                args.next()
                    .context("--book needs a notebook's name or ID")?
                    .clone(),
            );
        } else if let Some(value) = arg.strip_prefix("--book=") {
            book = Some(value.to_owned());
        } else {
            words.push(arg.clone());
        }
    }
    Ok((book, words))
}

/// Adds `request` as a note in its notebook (by ID, else by name) or the
/// Inbox, in the store `store` opens. Returns the note's ID and the
/// notebook's name. The one place a note from outside the app is made,
/// whether it came over the link or from a closed Kanso. Refuses a locked
/// notebook, the Inbox included: a note put there would sit in plain text
/// under a lock. The error is a plain sentence that says what to do.
pub fn add_note(
    store: &Store,
    request: &CaptureRequest,
    origin: Origin,
) -> Result<(String, String), String> {
    let failed = |error: &dyn std::fmt::Display| {
        format!("Kanso couldn't save the note: {error}. Try again in a moment.")
    };
    let (book, place) = match &request.book {
        Some(wanted) => {
            let books: Vec<(String, String)> = store
                .books()
                .map_err(|e| failed(&e))?
                .into_iter()
                .map(|b| (b.id, b.name))
                .collect();
            let (id, name) = pick_book(&books, wanted)
                .map_err(|reason| format!("Kanso didn't add the note: {reason}."))?;
            (id.clone(), name.clone())
        }
        None => (
            store
                .role_book(Role::Inbox, origin)
                .map_err(|e| failed(&e))?,
            "the Inbox".to_owned(),
        ),
    };
    if store
        .locked_books()
        .map_err(|e| failed(&e))?
        .contains(&book)
    {
        return Err(format!(
            "{place} is locked, so Kanso won't add notes to it from outside. Unlock it in Kanso, or choose another notebook with --book."
        ));
    }
    let made = store
        .put(
            DocType::Note,
            json!({"title": request.title, "body": request.body, "bookId": book}),
            Put::Upsert,
            origin,
        )
        .map_err(|e| failed(&e))?;
    Ok((made.id, place))
}

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let (book, rest) = book_arg(rest)?;
    let text = if rest.is_empty() || rest == ["-"] {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("reading standard input")?;
        text
    } else {
        rest.join(" ")
    };
    let Some((title, body)) = split_capture(&text) else {
        bail!("nothing to capture: the text is empty\n\n{USAGE}");
    };
    let request = CaptureRequest {
        title: title.clone(),
        body,
        book,
    };
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    let place = capture(data_dir, &request)?;
    println!("Saved “{title}” to {place}.");
    Ok(())
}

/// Adds the note to the running Kanso over the local link, or, with Kanso
/// closed, to the store. Returns where it went.
fn capture(data_dir: &Path, request: &CaptureRequest) -> Result<String> {
    let busy = match forward_capture(data_dir, request) {
        Ok(_id) => {
            return Ok(match &request.book {
                Some(book) => format!("“{}”", book.trim()),
                None => "the Inbox".to_owned(),
            });
        }
        // No Kanso answered, or something else holds its port, or this
        // note is too big for the link: with Kanso closed the store takes
        // it directly.
        Err(
            error @ (ForwardError::NotRunning
            | ForwardError::WrongPeer
            | ForwardError::BadCapture(_)),
        ) => error,
        Err(error) => bail!("Kanso didn't add the note: {error}"),
    };
    let Some(_lock) = lock_data_dir(data_dir)? else {
        if let ForwardError::BadCapture(reason) = busy {
            bail!("{reason}");
        }
        bail!(
            "Kanso is open but isn't answering on its local link. Wait a moment and try again; if it keeps failing, quit and reopen Kanso."
        );
    };
    let store = Store::open(crate::paths::db_path(data_dir))?;
    add_note(&store, request, Origin::Api)
        .map(|(_, place)| place)
        .map_err(anyhow::Error::msg)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_store::{Origin, Store};

    use super::*;
    use crate::url_callback::{CaptureSink, FileSink, Router};

    /// A Kanso that is open: it listens on the local link and adds what it
    /// is sent to its store, as `main` wires it.
    fn open_app(dir: &Path) -> Arc<Store> {
        let store = Arc::new(Store::open(crate::paths::db_path(dir)).unwrap());
        let sink: CaptureSink = {
            let store = store.clone();
            Arc::new(move |request| add_note(&store, &request, Origin::Api).map(|(id, _)| id))
        };
        let files: FileSink = Arc::new(|_| {});
        let router: &'static Router = Box::leak(Box::new(Router::new()));
        crate::url_callback::listen_with(dir, router, files, sink).unwrap();
        store
    }

    fn book(store: &Store, name: &str) -> String {
        store
            .put(
                DocType::Book,
                serde_json::json!({"name": name}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn count(store: &Store) -> usize {
        store.search_summaries(&Default::default()).unwrap().len()
    }

    /// `kanso capture` against a running app: no key, no local API; the
    /// note arrives through the link, in the Inbox or the notebook named.
    #[test]
    fn captures_into_a_running_app_over_the_link() {
        let dir = tempfile::tempdir().unwrap();
        // Holding the folder, as the app does.
        let _lock = lock_data_dir(dir.path()).unwrap().unwrap();
        let store = open_app(dir.path());
        run_cli(dir.path(), &["Buy milk\nand eggs".into()]).unwrap();
        run_cli(dir.path(), &["Call Sam".into()]).unwrap();
        let ideas = book(&store, "Ideas");
        run_cli(
            dir.path(),
            &["--book".into(), "ideas".into(), "Kites".into()],
        )
        .unwrap();
        let notes = store.search_summaries(&Default::default()).unwrap();
        assert_eq!(notes.len(), 3);
        let milk = notes.iter().find(|n| n.title == "Buy milk").unwrap();
        assert_eq!(milk.book_id, "book:inbox");
        assert_eq!(store.note(&milk.id).unwrap().unwrap().body, "and eggs");
        let kites = notes.iter().find(|n| n.title == "Kites").unwrap();
        assert_eq!(kites.book_id, ideas);
        // A notebook that doesn't exist: refused, saying how to fix it.
        let said = run_cli(dir.path(), &["--book".into(), "Nope".into(), "x".into()])
            .unwrap_err()
            .to_string();
        assert!(
            said.contains("no notebook named") && said.contains("--book"),
            "{said}"
        );
        assert_eq!(count(&store), 3);
    }

    /// A locked notebook takes nothing from outside, over the link or
    /// straight into the store, and says so plainly.
    #[test]
    fn a_locked_notebook_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let _lock = lock_data_dir(dir.path()).unwrap().unwrap();
        let store = open_app(dir.path());
        store
            .put(
                DocType::Book,
                serde_json::json!({
                    "_id": "book:secret",
                    "name": "Secret",
                    "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"},
                }),
                Put::Upsert,
                Origin::Sync,
            )
            .unwrap();
        assert!(store.locked_books().unwrap().contains("book:secret"));
        let said = run_cli(dir.path(), &["--book".into(), "Secret".into(), "x".into()])
            .unwrap_err()
            .to_string();
        assert!(
            said.contains("is locked") && said.contains("--book"),
            "{said}"
        );
        let request = CaptureRequest {
            title: "x".into(),
            body: String::new(),
            book: Some("book:secret".into()),
        };
        assert!(
            add_note(&store, &request, Origin::Api)
                .unwrap_err()
                .contains("locked")
        );
        assert_eq!(count(&store), 0);
    }

    /// Kanso holds the folder but nothing answers on the link: the command
    /// says what to do, and writes nothing.
    #[test]
    fn an_open_app_that_does_not_answer_is_not_written_around() {
        let dir = tempfile::tempdir().unwrap();
        let _lock = lock_data_dir(dir.path()).unwrap().unwrap();
        let said = run_cli(dir.path(), &["x".into()]).unwrap_err().to_string();
        assert!(said.contains("quit and reopen Kanso"), "{said}");
    }

    /// A note over the link's limit goes to a closed Kanso's store whole,
    /// and is refused, with advice, by an open one.
    #[test]
    fn a_note_too_big_for_the_link_needs_a_closed_app() {
        let dir = tempfile::tempdir().unwrap();
        let big = format!("Big\n{}", "x".repeat(crate::capture_request::MAX_BODY + 1));
        run_cli(dir.path(), std::slice::from_ref(&big)).unwrap();
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        assert_eq!(count(&store), 1);
        drop(store);
        let _lock = lock_data_dir(dir.path()).unwrap().unwrap();
        let said = run_cli(dir.path(), &[big]).unwrap_err().to_string();
        assert!(said.contains("Split it"), "{said}");
    }

    /// Without the app: straight into the store, the Inbox made on the way.
    #[test]
    fn captures_into_the_store_without_the_app() {
        let dir = tempfile::tempdir().unwrap();
        super::run_cli(dir.path(), &["Idea:".into(), "kites".into()]).unwrap();
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let books = store.books().unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "book:inbox");
        assert!(super::run_cli(dir.path(), &[" ".into()]).is_err());
        // --book by ID or name; a name nothing has is an error, and
        // nothing is saved.
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let ideas = store
            .put(
                kanso_core::DocType::Book,
                serde_json::json!({"name": "Ideas"}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap()
            .id;
        drop(store);
        super::run_cli(
            dir.path(),
            &["--book".into(), "IDEAS".into(), "Kites".into()],
        )
        .unwrap();
        super::run_cli(dir.path(), &[format!("--book={ideas}"), "Gliders".into()]).unwrap();
        assert!(super::run_cli(dir.path(), &["--book".into(), "Nope".into(), "x".into()]).is_err());
        assert!(super::run_cli(dir.path(), &["x".into(), "--book".into()]).is_err());
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let titles: Vec<String> = store
            .search_summaries(&Default::default())
            .unwrap()
            .into_iter()
            .filter(|n| n.book_id == ideas)
            .map(|n| n.title)
            .collect();
        assert_eq!(titles.len(), 2, "{titles:?}");
    }

    #[test]
    fn picks_a_book_by_id_or_name() {
        let books = vec![
            ("book:a".to_owned(), "Ideas".to_owned()),
            ("book:b".to_owned(), "Work".to_owned()),
            ("book:c".to_owned(), "work".to_owned()),
        ];
        assert_eq!(super::pick_book(&books, "book:b").unwrap().0, "book:b");
        assert_eq!(super::pick_book(&books, " ideas ").unwrap().0, "book:a");
        assert!(
            super::pick_book(&books, "work")
                .unwrap_err()
                .contains("book:b, book:c")
        );
        assert!(
            super::pick_book(&books, "nope")
                .unwrap_err()
                .contains("no notebook")
        );
    }
}
