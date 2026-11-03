//! What the person can do with people and projects: make a page, add an
//! alias, merge two pages. Nothing here runs without being asked for, and
//! nothing is created or merged on a guess.

use kanso_core::DocType;
use kanso_core::entities::{self, EntityKind};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use super::settings::{DEFAULT_NOTEBOOK, EntitySettings};

/// Why an action was refused.
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("{0}")]
    Store(#[from] kanso_store::StoreError),
    #[error("Give it a name first.")]
    NoName,
    #[error("That isn't a person or project page.")]
    NotAPage,
    #[error("A page can't be merged with itself.")]
    SameNote,
    #[error("Pick a person with a person, or a project with a project.")]
    KindsDiffer,
}

/// The notebook new pages go in: the one chosen in settings if it is still
/// there and is not locked, else the notebook named
/// [`DEFAULT_NOTEBOOK`], made if need be.
pub fn keep_book(store: &Store, settings: &EntitySettings) -> Result<String, ActionError> {
    let locked = store.locked_books()?;
    let books = store.books()?;
    if let Some(chosen) = &settings.keep_in
        && books.iter().any(|b| &b.id == chosen)
        && !locked.contains(chosen)
    {
        return Ok(chosen.clone());
    }
    if let Some(found) = books
        .iter()
        .find(|b| b.name.trim() == DEFAULT_NOTEBOOK && !locked.contains(&b.id))
    {
        return Ok(found.id.clone());
    }
    let made = store.put(
        DocType::Book,
        json!({ "name": DEFAULT_NOTEBOOK }),
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(made.id)
}

/// Makes the page for `name` and returns its note ID. When a page already
/// answers to the name, that one is returned and nothing is made.
pub fn create_page(
    store: &Store,
    settings: &EntitySettings,
    kind: EntityKind,
    name: &str,
    aliases: &[String],
    now: i64,
) -> Result<String, ActionError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(ActionError::NoName);
    }
    if let Some(existing) = store.entities_named(&name)?.first() {
        return Ok(existing.id.clone());
    }
    let book = keep_book(store, settings)?;
    let aliases: Vec<String> = aliases
        .iter()
        .filter(|a| entities::normalize_name(a) != entities::normalize_name(&name))
        .cloned()
        .collect();
    let made = store.put(
        DocType::Note,
        json!({
            "title": name,
            "body": entities::new_page_body(kind, &aliases),
            "bookId": book,
            "createdAt": now,
            "updatedAt": now,
        }),
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(made.id)
}

/// Adds `alias` to the page's `aliases`, as an edit of the person's own
/// note. Returns whether anything changed.
pub fn add_alias(store: &Store, id: &str, alias: &str, now: i64) -> Result<bool, ActionError> {
    let Some(page) = store.entity(id)? else {
        return Err(ActionError::NotAPage);
    };
    let doc = store.get(id)?.ok_or(ActionError::NotAPage)?;
    let body = doc
        .fields
        .get("body")
        .and_then(|b| b.as_str())
        .unwrap_or("")
        .to_owned();
    let next = entities::add_alias(&page.title, &body, alias);
    if next == body {
        return Ok(false);
    }
    store.put(
        DocType::Note,
        json!({ "_id": id, "body": next, "updatedAt": now }),
        Put::Merge,
        Origin::Ui,
    )?;
    Ok(true)
}

/// Appends `markdown` (a section's lines, with their sources as links) to
/// the end of the person's own text on page `id`, where it becomes theirs to
/// edit. The section stays live. Returns whether anything was written.
pub fn copy_into_note(
    store: &Store,
    id: &str,
    heading: &str,
    markdown: &str,
    now: i64,
) -> Result<bool, ActionError> {
    if store.entity(id)?.is_none() {
        return Err(ActionError::NotAPage);
    }
    let body = store
        .get(id)?
        .and_then(|d| d.fields.get("body").and_then(|b| b.as_str().map(str::to_owned)))
        .unwrap_or_default();
    if markdown.trim().is_empty() {
        return Ok(false);
    }
    let mut next = body.clone();
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&format!("\n## {}\n\n{}", heading.trim(), markdown.trim_end()));
    next.push('\n');
    store.put(
        DocType::Note,
        json!({ "_id": id, "body": next, "updatedAt": now }),
        Put::Merge,
        Origin::Ui,
    )?;
    Ok(true)
}

/// What a merge did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Merged {
    /// The page that stays.
    pub into: String,
    /// Links in other notes retitled to the kept page.
    pub links_retitled: usize,
}

/// Merges page `from` into page `into`. The target keeps its title; the
/// other page's title and aliases become its aliases, its own text is
/// appended under "Merged from <title>", every `[[Other]]` link is retitled,
/// and the other page goes to the trash, so the merge can be undone from
/// there and from the target's history.
pub fn merge(store: &Store, into: &str, from: &str, now: i64) -> Result<Merged, ActionError> {
    if into == from {
        return Err(ActionError::SameNote);
    }
    let (Some(target), Some(other)) = (store.entity(into)?, store.entity(from)?) else {
        return Err(ActionError::NotAPage);
    };
    if target.kind != other.kind {
        return Err(ActionError::KindsDiffer);
    }
    let body_of = |id: &str| -> Result<String, ActionError> {
        Ok(store
            .get(id)?
            .and_then(|d| d.fields.get("body").and_then(|b| b.as_str().map(str::to_owned)))
            .unwrap_or_default())
    };
    let (target_body, other_body) = (body_of(into)?, body_of(from)?);
    let mut next = target_body.clone();
    for alias in std::iter::once(other.title.clone())
        .chain(entities::written_aliases(&other.title, &other_body))
    {
        next = entities::add_alias(&target.title, &next, &alias);
    }
    let own = entities::own_text(&other_body).trim();
    if !own.is_empty() {
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(&format!("\n## Merged from {}\n\n{own}\n", other.title));
    }
    store.put(
        DocType::Note,
        json!({ "_id": into, "body": next, "updatedAt": now }),
        Put::Merge,
        Origin::Ui,
    )?;
    let links_retitled = store.retitle_links(&other.title, &target.title, into, Origin::Ui)?;
    store.put(
        DocType::Note,
        json!({ "_id": from, "bookId": kanso_core::TRASH_BOOK_ID, "updatedAt": now }),
        Put::Merge,
        Origin::Ui,
    )?;
    Ok(Merged {
        into: into.to_owned(),
        links_retitled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    // no-prop-test: a scripted sequence of user actions with fixed names.
    #[test]
    fn making_adding_and_merging_pages() {
        let (_dir, store) = store();
        let settings = EntitySettings::default();
        let sam = create_page(&store, &settings, EntityKind::Person, "Sam Moyo", &[], 10).unwrap();
        // Asking again returns the same page.
        assert_eq!(
            create_page(&store, &settings, EntityKind::Person, " sam   moyo ", &[], 11).unwrap(),
            sam
        );
        let book = store.get(&sam).unwrap().unwrap().fields["bookId"].clone();
        assert_eq!(
            store
                .books()
                .unwrap()
                .iter()
                .filter(|b| b.name == DEFAULT_NOTEBOOK)
                .count(),
            1
        );
        let other = create_page(
            &store,
            &settings,
            EntityKind::Person,
            "S. Moyo",
            &["Sammy".into()],
            12,
        )
        .unwrap();
        assert_eq!(store.get(&other).unwrap().unwrap().fields["bookId"], book);
        store
            .put(
                DocType::Note,
                json!({ "_id": other, "body": format!("{}Likes tea.\n", store.note(&other).unwrap().unwrap().body) }),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let linker = store
            .put(
                DocType::Note,
                json!({"title": "Linker", "body": "see [[S. Moyo]]\n", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        assert!(add_alias(&store, &sam, "Sam", 13).unwrap());
        assert!(!add_alias(&store, &sam, "sam", 14).unwrap());
        let done = merge(&store, &sam, &other, 15).unwrap();
        assert_eq!(done.links_retitled, 1);
        let kept = store.note(&sam).unwrap().unwrap().body;
        assert!(kept.contains("Merged from S. Moyo"));
        assert!(kept.contains("Likes tea."));
        let names = store.entity(&sam).unwrap().unwrap().names;
        assert!(names.contains(&"s. moyo".to_owned()) && names.contains(&"sammy".to_owned()));
        assert_eq!(store.note(&linker).unwrap().unwrap().body, "see [[Sam Moyo]]\n");
        // The other page is in the trash, not gone.
        assert_eq!(
            store.get(&other).unwrap().unwrap().fields["bookId"],
            kanso_core::TRASH_BOOK_ID
        );
        assert!(store.entity(&other).unwrap().is_none());
        assert!(copy_into_note(&store, &sam, "Promises", "- Send the deck [Linker](inkdrop://note/x#L1)\n", 17).unwrap());
        let copied = store.note(&sam).unwrap().unwrap().body;
        assert!(copied.contains("## Promises\n\n- Send the deck"), "{copied}");
        assert!(!copy_into_note(&store, &sam, "Promises", "  ", 18).unwrap());
        assert!(matches!(
            merge(&store, &sam, &sam, 16),
            Err(ActionError::SameNote)
        ));
        assert!(matches!(
            create_page(&store, &settings, EntityKind::Project, "  ", &[], 1),
            Err(ActionError::NoName)
        ));
    }
}
