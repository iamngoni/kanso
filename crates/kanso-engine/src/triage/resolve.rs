//! Editing a chip: the suggestion a typed value makes for a field, or why
//! it can't. A notebook, note or tag has to be one the library has (a tag
//! may be new); a title is any text. Both apps use it, so a chip means the
//! same on the desktop and on the phone.

use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::roles::Role;
use kanso_store::Store;

use super::links::linkable;
use super::model::{Field, Source, Suggestion};

/// The suggestion `text` makes for `field`, or a plain reason it can't.
pub fn resolve(store: &Store, field: Field, text: &str) -> Result<Suggestion, String> {
    let text = text.trim();
    let same = |a: &str| a.trim().to_lowercase() == text.to_lowercase();
    let make = |value: String, label: String, is_new: bool| Suggestion {
        field,
        value,
        label,
        score: 1.,
        because: Vec::new(),
        source: Source::Library,
        is_new,
    };
    match field {
        Field::Notebook => {
            let books = store.books().map_err(|e| e.to_string())?;
            let locked = store.locked_books().map_err(|e| e.to_string())?;
            let book = books
                .iter()
                .find(|b| same(&b.name))
                .ok_or_else(|| format!("There is no notebook named “{text}”."))?;
            if book.id == Role::Inbox.book_id() || book.id == TRASH_BOOK_ID {
                return Err("Choose a notebook to file into.".into());
            }
            if locked.contains(&book.id) {
                return Err("A locked notebook can't be filed into from here.".into());
            }
            Ok(make(book.id.clone(), book.name.clone(), false))
        }
        Field::Tag => {
            let name = text.trim_start_matches('#').trim();
            if name.is_empty() {
                return Err("Type a tag.".into());
            }
            let tags = store.tags().map_err(|e| e.to_string())?;
            Ok(
                match tags.iter().find(|t| name.eq_ignore_ascii_case(&t.name)) {
                    Some(tag) => make(tag.id.clone(), tag.name.clone(), false),
                    None => make(name.to_owned(), name.to_owned(), true),
                },
            )
        }
        Field::Title => {
            if text.is_empty() {
                return Err("Type a title.".into());
            }
            Ok(make(text.to_owned(), text.to_owned(), false))
        }
        Field::Link => {
            let titles = store.linkable_titles().map_err(|e| e.to_string())?;
            let found: Vec<_> = titles.iter().filter(|(_, t)| same(t)).collect();
            match found.as_slice() {
                [(id, title)] if linkable(title) => Ok(make(id.clone(), title.clone(), false)),
                [] => Err(format!("There is no note titled “{text}”.")),
                _ => Err("More than one note has that title.".into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    fn library() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let home = store
            .put(
                DocType::Book,
                json!({"name": "Home"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"title": "Boiler service", "body": "x", "bookId": home}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        store.tag_named("house", Origin::Ui).unwrap();
        store.role_book(Role::Inbox, Origin::Ui).unwrap();
        (dir, store, home)
    }

    #[test]
    fn values_resolve_against_the_library() {
        let (_d, store, home) = library();
        let book = resolve(&store, Field::Notebook, " home ").unwrap();
        assert_eq!((book.value, book.label), (home, "Home".to_owned()));
        assert!(resolve(&store, Field::Notebook, "Nowhere").is_err());
        assert!(resolve(&store, Field::Notebook, "Inbox").is_err());
        let tag = resolve(&store, Field::Tag, "#HOUSE").unwrap();
        assert!(!tag.is_new && tag.label == "house");
        let new = resolve(&store, Field::Tag, "Garden").unwrap();
        assert!(new.is_new && new.value == "Garden");
        assert!(resolve(&store, Field::Tag, "#").is_err());
        let link = resolve(&store, Field::Link, "boiler SERVICE").unwrap();
        assert_eq!(link.label, "Boiler service");
        assert!(resolve(&store, Field::Link, "Nothing here").is_err());
        assert_eq!(resolve(&store, Field::Title, " Hi ").unwrap().value, "Hi");
        assert!(resolve(&store, Field::Title, "  ").is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A resolved suggestion is for the field asked, has a label, and
        /// is new only for a tag.
        #[test]
        fn a_resolved_chip_is_for_its_field(text in "\\PC{0,20}", field in 0usize..4) {
            let (_d, store, _) = library();
            let field = Field::ALL[field];
            if let Ok(s) = resolve(&store, field, &text) {
                prop_assert_eq!(s.field, field);
                prop_assert!(!s.label.trim().is_empty() && !s.value.trim().is_empty());
                prop_assert!(!s.is_new || field == Field::Tag);
                prop_assert!((0. ..=1.).contains(&s.score));
            }
        }
    }
}
