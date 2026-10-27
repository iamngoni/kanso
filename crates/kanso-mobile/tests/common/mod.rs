//! Shared by the integration tests: libraries on temporary folders, with
//! in-memory secrets and an in-memory server.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use kanso_core::DocType;
use kanso_mobile::{Library, MemorySecrets, NewNote};
use kanso_store::{Origin, Put, Store};
use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};
use serde_json::json;

/// The lock password every test uses (at least 8 characters).
pub const LOCK_PASSWORD: &str = "open sesame";
/// The account every test signs in to.
pub const ACCOUNT: &str = "ada";
pub const ACCOUNT_PASSWORD: &str = "correct horse battery";

/// A library on its own folder.
pub struct Phone {
    pub dir: tempfile::TempDir,
    pub library: Library,
    pub secrets: Arc<MemorySecrets>,
}

impl Phone {
    pub fn path(&self) -> PathBuf {
        self.dir.path().join("library.db")
    }
}

/// An empty library whose secrets are its own and whose server is `server`.
pub fn phone(server: &Arc<MemoryServer>) -> Phone {
    phone_with(server, |_| {})
}

/// A library made after `seed` has written to its store.
pub fn phone_with(server: &Arc<MemoryServer>, seed: impl FnOnce(&Store)) -> Phone {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.db");
    {
        let store = Store::open(&path).unwrap();
        seed(&store);
    }
    let secrets = Arc::new(MemorySecrets::new());
    let library = Library::open_for_test(&path, secrets.clone(), server.clone()).unwrap();
    Phone {
        dir,
        library,
        secrets,
    }
}

/// An empty library on a server of its own.
pub fn lone_phone() -> Phone {
    phone(&Arc::new(MemoryServer::new()))
}

/// Makes the account on `server` (its session is dropped: phones sign in).
pub fn make_account(server: &MemoryServer) {
    let device = DeviceIdentity::from_parts("l_ffffffffffffffffffffffffffffffff", "Setup", [9; 32]);
    let password = Password::new(ACCOUNT_PASSWORD, ACCOUNT).unwrap();
    kanso_sync::sign_up(
        server,
        &MemorySecrets::new(),
        DEFAULT_SERVER,
        ACCOUNT,
        &password,
        &device,
        &|_| {},
    )
    .unwrap();
}

/// Signs `phone` in to the account.
pub fn sign_in(phone: &Phone, device_name: &str) {
    phone
        .library
        .sync()
        .sign_in_password(
            DEFAULT_SERVER.to_owned(),
            ACCOUNT.to_owned(),
            ACCOUNT_PASSWORD.to_owned(),
            device_name.to_owned(),
        )
        .unwrap();
}

/// A note with a title and a body in the first notebook.
pub fn note(library: &Library, title: &str, body: &str) -> String {
    library
        .create(NewNote {
            notebook_id: None,
            title: title.to_owned(),
            body: body.to_owned(),
            ..NewNote::default()
        })
        .unwrap()
        .id
}

/// Writes a notebook straight into `store`.
pub fn book(store: &Store, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({"name": name}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Writes a tag straight into `store`.
pub fn tag(store: &Store, name: &str) -> String {
    store
        .put(DocType::Tag, json!({"name": name}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

/// Writes a note straight into `store`.
pub fn raw_note(store: &Store, fields: serde_json::Value) -> String {
    store
        .put(DocType::Note, fields, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// A long note of `words` words, generated: headings, paragraphs, lists,
/// tasks, tables, fenced code, links, wiki-links and a paragraph of CJK and
/// emoji, every block closed, so that none of it has a syntax error. The
/// same words every time.
pub fn long_note(words: usize) -> String {
    let mut out = String::with_capacity(words * 7);
    let mut written = 0;
    let mut n = 0;
    while written < words {
        let block = match n % 9 {
            0 => format!("## Section {n}\n\n"),
            1 => format!(
                "Paragraph {n} has some *emphasis*, some **strong** words, `code` and a [link](https://example.com/{n}) in it, then {} more plain words after those.\n\n",
                "lorem ipsum dolor sit amet ".repeat(4)
            ),
            2 => format!("- item {n} one\n- item {n} two\n  - nested {n}\n- item {n} three\n\n"),
            3 => format!("- [ ] task {n} open\n- [x] task {n} done\n\n"),
            4 => format!("| a {n} | b |\n| --- | --- |\n| one | two |\n| three | four |\n\n"),
            5 => format!("```rust\nfn block_{n}() -> u32 {{\n    {n}\n}}\n```\n\n"),
            6 => format!("See [[Note {n}]] and #tag{} for more.\n\n", n % 7),
            7 => format!("日本語の段落 {n} です。絵文字 🙂🎉 も入っています。\n\n"),
            _ => format!("> quoted {n} line one\n> quoted {n} line two\n\n"),
        };
        written += block.split_whitespace().count();
        out.push_str(&block);
        n += 1;
    }
    out
}

/// `long_note`, with the broken table the Dart fixture
/// (`mobile/integration_test/helpers/long_note.dart`) has after three
/// quarters of its words: a syntax error that makes the highlighter parse
/// all that follows as one block.
pub fn long_note_with_error(words: usize) -> String {
    let mut out = String::new();
    let mut broken = false;
    let mut counted = 0;
    for block in long_note(words).split_inclusive("\n\n") {
        if !broken && counted >= words * 3 / 4 {
            broken = true;
            out.push_str("|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |\n\n\nafter the broken table\n\n");
        }
        counted += block.split_whitespace().count();
        out.push_str(block);
    }
    out
}
