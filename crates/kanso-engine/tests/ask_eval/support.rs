//! Loads the evaluation library into a store. Shared by the test and the
//! manual example.

#![allow(dead_code)]

use std::collections::HashMap;

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_store::{Origin, Put, Store};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Library {
    pub today: String,
    pub books: Vec<BookSpec>,
    pub notes: Vec<NoteSpec>,
}

#[derive(Deserialize)]
pub struct BookSpec {
    pub name: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Deserialize)]
pub struct NoteSpec {
    pub title: String,
    pub book: String,
    pub body: String,
    pub created: i64,
    pub updated: i64,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct Spot {
    pub title: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Deserialize, Clone)]
pub struct Question {
    pub id: String,
    pub category: String,
    pub question: String,
    pub answerable: bool,
    pub expected: Vec<Spot>,
    pub scope: Option<ScopeSpec>,
    pub bait: Vec<String>,
    pub after: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct ScopeSpec {
    pub book: String,
}

/// A loaded library: the store and the IDs of everything in it.
pub struct Loaded {
    pub _dir: tempfile::TempDir,
    pub store: Store,
    pub today: Date,
    /// Note title to ID (the first with that title).
    pub ids: HashMap<String, String>,
    pub books: HashMap<String, String>,
}

pub fn parse_date(text: &str) -> Date {
    let mut parts = text.split('-').map(|p| p.parse::<i64>().unwrap());
    let (y, m, d) = (
        parts.next().unwrap() as i32,
        parts.next().unwrap() as u32,
        parts.next().unwrap() as u32,
    );
    Date::new(y, m, d).unwrap()
}

pub fn library() -> Library {
    serde_json::from_str(include_str!("library.json")).expect("library.json parses")
}

pub fn questions() -> Vec<Question> {
    serde_json::from_str(include_str!("questions.json")).expect("questions.json parses")
}

pub fn load() -> Loaded {
    let library = library();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let mut books: HashMap<String, String> = HashMap::new();
    for spec in &library.books {
        let mut fields = json!({"name": spec.name});
        if let Some(parent) = &spec.parent {
            fields["parentBookId"] = json!(books[parent]);
        }
        let id = store
            .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        books.insert(spec.name.clone(), id);
    }
    let exclude = store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let mut ids = HashMap::new();
    for spec in &library.notes {
        let tags: Vec<String> = spec.tags.iter().map(|_| exclude.clone()).collect();
        let id = store
            .put(
                DocType::Note,
                json!({"title": spec.title, "body": spec.body, "bookId": books[&spec.book],
                       "tags": tags, "createdAt": spec.created, "updatedAt": spec.updated}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        ids.entry(spec.title.clone()).or_insert(id);
    }
    // Locked last, so the notes could be written into it.
    for spec in library.books.iter().filter(|b| b.locked) {
        store
            .put(
                DocType::Book,
                json!({"_id": books[&spec.name], "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
    }
    Loaded {
        _dir: dir,
        store,
        today: parse_date(&library.today),
        ids,
        books,
    }
}
