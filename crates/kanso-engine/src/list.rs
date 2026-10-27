//! What the note list shows and in what order: the places (all notes, pinned,
//! recent, a notebook, a tag, the trash), the search each one is, the sort,
//! the narrowing by status or pinned notes, and how a list is grouped under
//! date headers (issue #48).
//!
//! The desktop's list and the phones' list both ask here; neither decides
//! for itself. The words of the headers ("Today", "May 25") are the caller's,
//! since the phones say them in the person's language: [`grouping`] only
//! numbers days and says where a header goes.

use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{Book, Clause, Field, NoteStatus, Query};
use kanso_store::{NoteSummary, SearchParams, Sort, Store, StoreError};

pub mod grouping;

/// How far back Recent reaches.
pub const RECENT_DAYS: i64 = 7;
/// Milliseconds in a day.
pub const DAY_MS: i64 = 86_400_000;

/// What the note list shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Pinned,
    /// Notes changed in the last [`RECENT_DAYS`] days, newest first.
    Recent,
    Book(String),
    Tag(String),
    /// A level of the tag tree (`work` for `work/acme`): notes with a tag
    /// at or under it.
    TagPath(String),
    Status(NoteStatus),
    Trash,
    /// The tasks across notes, in the Tasks view (`crate::tasks_view`),
    /// in the note list's place.
    Tasks,
    /// Notes other accounts shared with this one (docs/ACCOUNTS.md ›
    /// Collaboration), with invitations waiting above them.
    Shared,
}

impl Filter {
    /// The query clauses this filter adds to a search.
    pub fn clauses(&self) -> Vec<Clause> {
        let clause = |field, value: &str| Clause {
            negated: false,
            field: Some(field),
            value: value.to_owned(),
        };
        match self {
            Filter::All | Filter::Pinned | Filter::Recent | Filter::Tasks | Filter::Shared => {
                Vec::new()
            }
            Filter::Book(id) => vec![clause(Field::BookId, id)],
            Filter::Tag(id) => vec![clause(Field::TagId, id)],
            Filter::TagPath(path) => vec![clause(Field::Tag, path)],
            Filter::Status(status) => vec![clause(Field::Status, status.as_str())],
            Filter::Trash => vec![clause(Field::BookId, TRASH_BOOK_ID)],
        }
    }
}

/// Builds the search for a filter and the text in the search box.
pub fn search_params(filter: &Filter, search: &str, sort: Sort, descending: bool) -> SearchParams {
    let mut query = Query::parse(search);
    query.clauses.extend(filter.clauses());
    SearchParams {
        query,
        sort,
        descending,
        limit: None,
        skip: 0,
        // The app finds notes by the words in their pictures too.
        ocr: true,
    }
}

/// Narrows a search to one status, for the note list's filter menu.
pub fn refine_status(params: &mut SearchParams, status: Option<NoteStatus>) {
    if let Some(status) = status {
        params
            .query
            .clauses
            .push(Clause::field(Field::Status, status.as_str()));
    }
}

/// Pinned notes first, keeping the sort order within each group.
pub fn pinned_first(notes: &mut [NoteSummary]) {
    notes.sort_by_key(|note| !note.pinned);
}

/// The notes changed at or after `since_ms`, newest first; notes changed at
/// the same moment keep their order.
pub fn recent_first(mut notes: Vec<NoteSummary>, since_ms: i64) -> Vec<NoteSummary> {
    notes.retain(|note| note.updated_at >= since_ms);
    notes.sort_by_key(|note| std::cmp::Reverse(note.updated_at));
    notes
}

/// The notes matching `params` changed since `since_ms`, newest first. Asks
/// the store for the newest notes a page at a time, so a large library
/// isn't read whole for a short list.
fn recent_notes(
    store: &Store,
    params: &SearchParams,
    since_ms: i64,
) -> Result<Vec<NoteSummary>, StoreError> {
    let mut limit = 64;
    loop {
        let page = SearchParams {
            query: params.query.clone(),
            sort: Sort::UpdatedAt,
            descending: true,
            limit: Some(limit),
            skip: 0,
            ocr: params.ocr,
        };
        let notes = store.search_summaries(&page)?;
        let done = notes.len() < limit || notes.last().is_some_and(|n| n.updated_at < since_ms);
        if done {
            return Ok(recent_first(notes, since_ms));
        }
        limit *= 4;
    }
}

/// The notes a list of `filter` shows for `params`: Recent's newest first,
/// the rest pinned first, and only the pinned ones when `pinned_only`.
/// The note list and the board read their notes through this.
pub fn list_notes(
    store: &Store,
    filter: &Filter,
    params: &SearchParams,
    pinned_only: bool,
    recent_since: i64,
) -> Result<Vec<NoteSummary>, StoreError> {
    let mut notes = match filter {
        Filter::Recent => recent_notes(store, params, recent_since)?,
        Filter::Shared => {
            let shared: std::collections::HashSet<String> =
                store.shared_with_me()?.into_iter().collect();
            let mut notes = store.search_summaries(params)?;
            notes.retain(|note| shared.contains(&note.id));
            pinned_first(&mut notes);
            notes
        }
        _ => {
            let mut notes = store.search_summaries(params)?;
            pinned_first(&mut notes);
            notes
        }
    };
    if pinned_only || *filter == Filter::Pinned {
        notes.retain(|note| note.pinned);
    }
    Ok(notes)
}

/// Which date a list is grouped by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateKey {
    Updated,
    Created,
}

/// How a list is laid out for a filter, a narrowing and a sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// The sort the list is in: Recent is always by edit time, whatever was
    /// asked.
    pub sort: Sort,
    /// Whether the notes sit under headers for their days.
    pub by_date: bool,
    /// The date the headers (and the rows' times) are of.
    pub date: DateKey,
    /// Whether pinned notes sit under their own header. They stay among the
    /// days in Recent, and a list of pinned notes only groups by date too.
    pub group_pinned: bool,
}

/// The layout of the list for `filter`, with `pinned_only` narrowing it,
/// sorted by `sort`.
pub fn shape(filter: &Filter, pinned_only: bool, sort: Sort) -> Shape {
    let recent = *filter == Filter::Recent;
    let sort = if recent { Sort::UpdatedAt } else { sort };
    Shape {
        sort,
        by_date: matches!(sort, Sort::UpdatedAt | Sort::CreatedAt),
        date: if sort == Sort::CreatedAt {
            DateKey::Created
        } else {
            DateKey::Updated
        },
        group_pinned: !recent && *filter != Filter::Pinned && !pinned_only,
    }
}

/// Notebook `id` and its parents, outermost first: the path a breadcrumb
/// shows. A parent that is missing ends it, and so does a parent already on
/// it, should the notebooks form a cycle.
pub fn book_path<'a>(id: &str, books: &'a [Book]) -> Vec<&'a Book> {
    let mut path: Vec<&Book> = Vec::new();
    let mut next = Some(id);
    while let Some(id) = next {
        let Some(book) = books.iter().find(|b| b.id == id) else {
            break;
        };
        if path.iter().any(|b| b.id == book.id) {
            break;
        }
        path.push(book);
        next = book.parent_book_id.as_deref();
    }
    path.reverse();
    path
}

/// Whether notebook `id` is locked: by its own lock, or by one of the
/// notebooks it is inside (`kanso_store`'s `locked_books`, from the books
/// the app has).
pub fn is_book_locked(id: &str, books: &[Book]) -> bool {
    book_path(id, books).iter().any(|book| book.is_locked())
}

/// Whether a notebook inside `id`, at any depth, has notes of its own. A
/// notebook can hold only notebooks, its notes all a level or more down;
/// it isn't empty then. Should the notebooks form a cycle, each is looked
/// at once.
pub fn has_notes_inside(id: &str, books: &[Book]) -> bool {
    let mut seen = vec![id];
    let mut parents = vec![id];
    while let Some(parent) = parents.pop() {
        for book in books
            .iter()
            .filter(|b| b.parent_book_id.as_deref() == Some(parent))
        {
            if seen.contains(&book.id.as_str()) {
                continue;
            }
            if book.count > 0 {
                return true;
            }
            seen.push(&book.id);
            parents.push(&book.id);
        }
    }
    false
}

/// Orders notebooks depth-first by name, children under their parent. A book
/// whose parent is missing is shown at the top level.
pub fn notebook_tree(books: &[Book]) -> Vec<(usize, &Book)> {
    fn visit<'a>(
        parent: Option<&str>,
        depth: usize,
        books: &'a [Book],
        out: &mut Vec<(usize, &'a Book)>,
    ) {
        let mut children: Vec<&Book> = books
            .iter()
            .filter(|b| match (parent, b.parent_book_id.as_deref()) {
                (None, None) => true,
                (None, Some(p)) => !books.iter().any(|other| other.id == p),
                (Some(parent), Some(p)) => p == parent,
                (Some(_), None) => false,
            })
            .collect();
        children.sort_by(|a, b| {
            (a.order, a.name.to_lowercase(), &a.id).cmp(&(b.order, b.name.to_lowercase(), &b.id))
        });
        for book in children {
            if out.iter().any(|(_, b)| b.id == book.id) {
                continue;
            }
            out.push((depth, book));
            visit(Some(&book.id), depth + 1, books, out);
        }
    }
    let mut out = Vec::with_capacity(books.len());
    visit(None, 0, books, &mut out);
    // Books in a parent cycle are unreachable from the top; still show them.
    for book in books {
        if !out.iter().any(|(_, b)| b.id == book.id) {
            out.push((0, book));
            visit(Some(&book.id), 1, books, &mut out);
        }
    }
    out
}

/// What narrows the list a new note is made from, beyond its place: the
/// note shows in that list only if it carries the same status or pin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Narrowing {
    pub status: Option<NoteStatus>,
    pub pinned: bool,
}

/// Where a new note goes and what it starts with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewNoteTarget {
    /// The notebook, which exists; `None` when there is no notebook at all
    /// and the caller makes one named [`FALLBACK_BOOK_NAME`].
    pub book_id: Option<String>,
    /// Tag IDs the note starts with.
    pub tags: Vec<String>,
    /// A level of the tag tree the note is tagged with. Making that tag may
    /// need the store, so the caller does it.
    pub tag_path: Option<String>,
    pub status: Option<NoteStatus>,
    pub pinned: bool,
}

/// The name of the notebook made when there is none.
pub const FALLBACK_BOOK_NAME: &str = "Notes";

/// Where a new note made from the list at `filter` goes (issue #48).
///
/// - A notebook's list: that notebook, even when it is locked (the caller
///   seals the note, or refuses when the lock is closed).
/// - Anything else: `preferred`, if it names a notebook that exists and isn't
///   locked, otherwise the first notebook, otherwise none (the caller makes
///   one).
/// - A tag's list tags the note, the pinned list pins it, a status list sets
///   the status, and the trash's list is the same as any other that isn't a
///   notebook: a note never goes into the trash.
///
/// The list's own narrowing carries over, so the new note shows in it.
pub fn new_note_target(
    filter: &Filter,
    narrowing: Narrowing,
    preferred: Option<&str>,
    books: &[Book],
) -> NewNoteTarget {
    let real = |book: &&Book| book.id != TRASH_BOOK_ID;
    let existing = |id: &str| books.iter().filter(real).find(|book| book.id == id);
    let own = match filter {
        Filter::Book(id) => existing(id).map(|book| book.id.clone()),
        _ => None,
    };
    let preferred = || {
        preferred
            .and_then(existing)
            .filter(|book| !is_book_locked(&book.id, books))
            .map(|book| book.id.clone())
    };
    let book_id = own
        .or_else(preferred)
        .or_else(|| books.iter().find(real).map(|book| book.id.clone()));
    let mut target = NewNoteTarget {
        book_id,
        status: narrowing.status,
        pinned: narrowing.pinned,
        ..NewNoteTarget::default()
    };
    match filter {
        Filter::Tag(tag) => target.tags.push(tag.clone()),
        Filter::TagPath(path) => target.tag_path = Some(path.clone()),
        Filter::Status(status) => target.status = narrowing.status.or(Some(*status)),
        Filter::Pinned => target.pinned = true,
        _ => {}
    }
    target
}

#[cfg(test)]
mod tests {
    use super::{
        DateKey, Filter, Narrowing, book_path, has_notes_inside, is_book_locked, list_notes,
        new_note_target, notebook_tree, pinned_first, recent_first, recent_notes, refine_status,
        search_params, shape,
    };
    use crate::lock::book_subtree;
    use kanso_core::{Book, NoteFacts, NoteStatus, Query};
    use kanso_store::{NoteSummary, Sort};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::Map;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn book(id: usize, parent: Option<usize>, name: &str) -> Book {
        Book {
            id: format!("book:{id}"),
            rev: None,
            name: name.into(),
            parent_book_id: parent.map(|p| format!("book:{p}")),
            order: None,
            count: 0,
            created_at: 0,
            updated_at: 0,
            extra: Map::new(),
        }
    }

    /// Forests where each book's parent has a smaller index (no cycles), and
    /// some parents point at books that don't exist.
    fn gen_books() -> impl Strategy<Value = Vec<Book>> {
        prop::collection::vec((prop::option::of(0usize..40), "[a-c]{1,3}"), 0..20).prop_map(
            |specs| {
                specs
                    .into_iter()
                    .enumerate()
                    .map(|(ix, (parent, name))| {
                        book(ix, parent.filter(|p| *p < ix || *p >= 20), &name)
                    })
                    .collect()
            },
        )
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariant: the tree lists every notebook exactly once, and each
        /// child comes after its parent, one level deeper.
        #[test]
        fn prop_notebook_tree_lists_each_book_once_under_its_parent(books in gen_books()) {
            let tree = notebook_tree(&books);
            prop_assert_eq!(tree.len(), books.len());
            for (ix, (depth, book)) in tree.iter().enumerate() {
                prop_assert_eq!(tree.iter().filter(|(_, b)| b.id == book.id).count(), 1);
                match book.parent_book_id.as_deref().and_then(|p| tree.iter().position(|(_, b)| b.id == p)) {
                    Some(parent_ix) => {
                        prop_assert!(parent_ix < ix);
                        prop_assert_eq!(*depth, tree[parent_ix].0 + 1);
                    }
                    None => prop_assert_eq!(*depth, 0),
                }
            }
        }

        /// Metamorphic: a filter only narrows a search, and its clauses come
        /// after the ones typed in the search box.
        #[test]
        fn prop_filter_appends_its_clauses(search in "[a-z :\"-]{0,20}", which in 0usize..7) {
            let filter = [
                Filter::All,
                Filter::Pinned,
                Filter::Book("book:x".into()),
                Filter::Tag("tag:y".into()),
                Filter::Status(NoteStatus::Active),
                Filter::Trash,
                Filter::Shared,
            ][which].clone();
            let typed = Query::parse(&search);
            let params = search_params(&filter, &search, Sort::Title, false);
            prop_assert_eq!(&params.query.clauses[..typed.clauses.len()], &typed.clauses[..]);
            prop_assert_eq!(&params.query.clauses[typed.clauses.len()..], &filter.clauses()[..]);
            prop_assert_eq!(params.query.includes_trash(), filter == Filter::Trash || typed.includes_trash());
        }

        /// Invariants: the path ends at the notebook, each step is the
        /// previous one's child, nothing repeats, and it is never longer than
        /// the list of notebooks, even when parents form cycles.
        #[test]
        fn prop_book_path_walks_up_to_the_top(
            parents in prop::collection::vec(prop::option::of(0usize..12), 0..10),
            target in 0usize..12,
        ) {
            // Any parent at all, so cycles and missing parents both occur.
            let books: Vec<Book> = parents.iter().enumerate().map(|(ix, p)| book(ix, *p, "b")).collect();
            let id = format!("book:{target}");
            let path = book_path(&id, &books);
            prop_assert!(path.len() <= books.len());
            prop_assert_eq!(path.is_empty(), target >= books.len());
            if let Some(last) = path.last() {
                prop_assert_eq!(&last.id, &id);
            }
            for pair in path.windows(2) {
                prop_assert_eq!(pair[1].parent_book_id.as_deref(), Some(pair[0].id.as_str()));
            }
            for (ix, b) in path.iter().enumerate() {
                prop_assert!(path[ix + 1..].iter().all(|other| other.id != b.id));
            }
            // The walk stops only at the top, a missing parent, or a cycle.
            if let Some(first) = path.first() {
                match first.parent_book_id.as_deref() {
                    None => {}
                    Some(parent) => prop_assert!(
                        !books.iter().any(|b| b.id == parent) || path.iter().any(|b| b.id == parent)
                    ),
                }
            }
        }

        /// Model: a notebook is inside a notebook's subtree exactly when the
        /// second is on the first one's path up to the top, cycles and
        /// missing parents included; and a notebook is locked exactly when
        /// a notebook on its path has a lock, which is when it is in the
        /// subtree of some notebook with a lock: locking that one seals it.
        #[test]
        fn prop_a_locked_notebook_reaches_every_notebook_inside_it(
            specs in prop::collection::vec((prop::option::of(0usize..12), any::<bool>()), 1..10),
            target in any::<prop::sample::Index>(),
        ) {
            let books: Vec<Book> = specs
                .iter()
                .enumerate()
                .map(|(ix, (parent, locked))| {
                    let mut b = book(ix, *parent, "b");
                    if *locked {
                        b.extra.insert(kanso_core::model::LOCK_FIELD.into(), serde_json::json!({}));
                    }
                    b
                })
                .collect();
            let root = &books[target.index(books.len())].id;
            let inside = book_subtree(root, &books);
            prop_assert_eq!(inside.first(), Some(root));
            for b in &books {
                let below = book_path(&b.id, &books).iter().any(|on| &on.id == root);
                prop_assert_eq!(inside.contains(&b.id), below, "{}", b.id);
                let by_subtree = books
                    .iter()
                    .filter(|lock| lock.is_locked())
                    .any(|lock| book_subtree(&lock.id, &books).contains(&b.id));
                prop_assert_eq!(is_book_locked(&b.id, &books), by_subtree, "{}", b.id);
            }
            prop_assert_eq!(
                inside.len(),
                inside.iter().collect::<std::collections::HashSet<_>>().len()
            );
        }

        /// Model: a notebook holds notes inside it exactly when some other
        /// notebook with notes has it on its path up to the top. The path
        /// walks parents bottom-up; the function walks children top-down.
        #[test]
        fn prop_notes_inside_are_below_on_some_path(
            specs in prop::collection::vec((prop::option::of(0usize..12), 0u32..3), 1..10),
            target in any::<prop::sample::Index>(),
        ) {
            // Any parent at all, so cycles and missing parents both occur.
            let books: Vec<Book> = specs
                .iter()
                .enumerate()
                .map(|(ix, (parent, count))| Book { count: *count, ..book(ix, *parent, "b") })
                .collect();
            let id = books[target.index(books.len())].id.clone();
            let expected = books.iter().any(|b| {
                b.id != id && b.count > 0 && book_path(&b.id, &books).iter().any(|p| p.id == id)
            });
            prop_assert_eq!(has_notes_inside(&id, &books), expected);
        }

        /// Model: Recent is exactly the notes changed at or after the cut,
        /// newest first, with ties in their original order.
        #[test]
        fn prop_recent_first_keeps_new_notes_newest_first(
            times in prop::collection::vec(0i64..20, 0..30),
            since in 0i64..22,
        ) {
            let notes: Vec<NoteSummary> = times.iter().enumerate().map(|(ix, &t)| summary(ix, t)).collect();
            let mut expected: Vec<(i64, usize)> = times.iter().enumerate()
                .filter(|(_, t)| **t >= since).map(|(ix, t)| (-t, ix)).collect();
            expected.sort();
            let got: Vec<String> = recent_first(notes, since).into_iter().map(|n| n.id).collect();
            let want: Vec<String> = expected.into_iter().map(|(_, ix)| format!("note:{ix}")).collect();
            prop_assert_eq!(got, want);
        }

        /// Metamorphic: narrowing to a status keeps exactly the notes the
        /// search found that have that status; no status changes nothing.
        #[test]
        fn prop_refine_status_narrows_to_that_status(
            search in "[a-z :-]{0,12}",
            want in prop::option::of(0usize..5),
            notes in prop::collection::vec((0usize..5, "[a-z ]{0,8}"), 0..12),
        ) {
            let want = want.map(|ix| NoteStatus::ALL[ix]);
            let plain = search_params(&Filter::All, &search, Sort::UpdatedAt, true);
            let mut refined = plain.clone();
            refine_status(&mut refined, want);
            if want.is_none() {
                prop_assert_eq!(&refined.query, &plain.query);
            }
            for (status, title) in &notes {
                let status = NoteStatus::ALL[*status];
                let facts = NoteFacts {
                    title, body: "", book_id: "book:x", book_name: None,
                    status: status.as_str(), tag_ids: &[], tag_names: &[],
                };
                let expected = plain.query.matches(&facts) && want.is_none_or(|w| w == status);
                prop_assert_eq!(refined.query.matches(&facts), expected);
            }
        }

        /// Invariant: pinned notes come first and each group keeps its order.
        #[test]
        fn prop_pinned_first_is_a_stable_partition(pins in prop::collection::vec(any::<bool>(), 0..30)) {
            let mut notes: Vec<NoteSummary> = pins.iter().enumerate()
                .map(|(ix, &pinned)| NoteSummary { pinned, ..summary(ix, 0) }).collect();
            let expected: Vec<String> = notes.iter().filter(|n| n.pinned)
                .chain(notes.iter().filter(|n| !n.pinned)).map(|n| n.id.clone()).collect();
            pinned_first(&mut notes);
            prop_assert_eq!(notes.into_iter().map(|n| n.id).collect::<Vec<_>>(), expected);
        }
    }

    fn summary(ix: usize, updated_at: i64) -> NoteSummary {
        NoteSummary {
            id: format!("note:{ix}"),
            rev: String::new(),
            title: String::new(),
            snippet: String::new(),
            book_id: String::new(),
            status: NoteStatus::None,
            tags: Vec::new(),
            pinned: false,
            updated_at,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    #[test]
    fn book_path_examples() {
        let books = vec![
            book(1, None, "Work"),
            book(2, Some(1), "Projects"),
            book(3, Some(2), "Kanso"),
            book(4, Some(5), "Loop"),
            book(5, Some(4), "Back"),
        ];
        let names = |id: &str| -> Vec<String> {
            book_path(id, &books)
                .into_iter()
                .map(|b| b.name.clone())
                .collect()
        };
        assert_eq!(names("book:3"), ["Work", "Projects", "Kanso"]);
        assert_eq!(names("book:1"), ["Work"]);
        assert_eq!(names("book:4"), ["Back", "Loop"]);
        assert!(names("book:9").is_empty());
    }

    #[test]
    fn notes_inside_examples() {
        let counted = |id, parent, count| Book {
            count,
            ..book(id, parent, "b")
        };
        let books = vec![
            // Clients holds its notes in a sub-notebook.
            counted(1, None, 0),
            counted(2, Some(1), 17),
            // Projects holds one empty notebook.
            counted(3, None, 0),
            counted(4, Some(3), 0),
            // Archive's notes are two levels down.
            counted(5, None, 0),
            counted(6, Some(5), 0),
            counted(7, Some(6), 2),
            // A loop with notes only at the start.
            counted(8, Some(9), 4),
            counted(9, Some(8), 0),
        ];
        let inside = |id: &str| has_notes_inside(id, &books);
        assert!(inside("book:1"));
        assert!(!inside("book:2"), "notes of its own only");
        assert!(!inside("book:3"));
        assert!(inside("book:5"));
        assert!(inside("book:6"));
        assert!(inside("book:9"));
        assert!(!inside("book:8"), "its own notes come back round the loop");
        assert!(!inside("book:99"));
    }

    /// Recent reads the store a page at a time and still finds every recent
    /// note past the first page, and none of the old or trashed ones.
    #[test]
    fn recent_notes_page_through_the_store() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, SearchParams, Store};
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        for ix in 0..150i64 {
            let book_id = if ix == 100 {
                "trash".to_owned()
            } else {
                book.clone()
            };
            store
                .put(
                    DocType::Note,
                    serde_json::json!({"title": format!("n{ix}"), "bookId": book_id, "updatedAt": 1_000 + ix}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        let recent = recent_notes(&store, &SearchParams::default(), 1_010).unwrap();
        assert_eq!(recent.len(), 139);
        assert_eq!(recent.first().unwrap().updated_at, 1_149);
        assert!(recent.windows(2).all(|w| w[0].updated_at > w[1].updated_at));
        let typed = search_params(&Filter::Recent, "title:n14", Sort::Title, false);
        let found: Vec<String> = recent_notes(&store, &typed, 1_010)
            .unwrap()
            .into_iter()
            .map(|n| n.title)
            .collect();
        assert_eq!(
            found,
            [
                "n149", "n148", "n147", "n146", "n145", "n144", "n143", "n142", "n141", "n140",
                "n14"
            ]
        );
    }

    #[test]
    fn nested_books_follow_their_parent() {
        let books = vec![
            book(1, None, "Work"),
            book(2, Some(1), "Projects"),
            book(3, None, "Archive"),
        ];
        let tree: Vec<(usize, &str)> = notebook_tree(&books)
            .into_iter()
            .map(|(d, b)| (d, b.name.as_str()))
            .collect();
        assert_eq!(tree, vec![(0, "Archive"), (0, "Work"), (1, "Projects")]);
    }

    /// Shared with me lists the notes others shared with this account:
    /// not the ones it shares itself, not private ones, and not trashed
    /// ones; the search box still narrows it.
    #[test]
    fn shared_with_me_lists_only_others_notes() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, SharedNote, SharedStatus, Store};
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = |title: &str, book: &str| {
            store
                .put(
                    DocType::Note,
                    serde_json::json!({"title": title, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let theirs = note("Theirs", &book);
        let mine = note("Mine", &book);
        let trashed = note("Trashed", "trash");
        note("Private", &book);
        for (n, (id, role)) in [(&theirs, "edit"), (&mine, "owner"), (&trashed, "view")]
            .into_iter()
            .enumerate()
        {
            store
                .put_shared_note(&SharedNote {
                    share_id: format!("s{n}"),
                    note_id: id.clone(),
                    owner_id: "a_owner".into(),
                    owner_name: "user2".into(),
                    role: role.into(),
                    status: SharedStatus::Active,
                    epoch: 1,
                    list: String::new(),
                    list_sig: String::new(),
                    state: Vec::new(),
                    applied_seq: 0,
                    since_snapshot: 0,
                    bytes_since_snapshot: 0,
                    counter: 0,
                    needs_snapshot: false,
                    updated_at: 0,
                })
                .unwrap();
        }
        let listed = |search: &str| -> Vec<String> {
            let params = search_params(&Filter::Shared, search, Sort::Title, false);
            list_notes(&store, &Filter::Shared, &params, false, 0)
                .unwrap()
                .into_iter()
                .map(|n| n.title)
                .collect()
        };
        assert_eq!(listed(""), ["Theirs"]);
        assert!(listed("nothing-like-it").is_empty());
    }

    #[test]
    fn trash_filter_searches_the_trash() {
        let params = search_params(&Filter::Trash, "hello", Sort::UpdatedAt, true);
        assert!(params.query.includes_trash());
        assert_eq!(params.query.to_string(), "hello bookId:trash");
    }

    fn any_filter() -> impl Strategy<Value = Filter> {
        prop_oneof![
            Just(Filter::All),
            Just(Filter::Pinned),
            Just(Filter::Recent),
            Just(Filter::Trash),
            Just(Filter::Shared),
            "[a-z]{1,4}".prop_map(|id| Filter::Book(format!("book:{id}"))),
            "[a-z]{1,4}".prop_map(|id| Filter::Tag(format!("tag:{id}"))),
            "[a-z]{1,4}".prop_map(Filter::TagPath),
            prop::sample::select(NoteStatus::ALL.to_vec()).prop_map(Filter::Status),
        ]
    }

    fn any_sort() -> impl Strategy<Value = Sort> {
        prop::sample::select(vec![Sort::UpdatedAt, Sort::CreatedAt, Sort::Title])
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariant: Recent is by edit time whatever the sort and the
        /// narrowing; every other place keeps the sort it was asked for.
        #[test]
        fn prop_recent_ignores_the_sort(sort in any_sort(), pinned_only: bool, filter in any_filter()) {
            let shape = shape(&filter, pinned_only, sort);
            if filter == Filter::Recent {
                prop_assert_eq!(shape.sort, Sort::UpdatedAt);
                prop_assert_eq!(shape.date, DateKey::Updated);
                prop_assert!(shape.by_date);
            } else {
                prop_assert_eq!(shape.sort, sort);
            }
        }

        /// Invariant: pinned notes sit under their own header only in a list
        /// that is not Recent, not Pinned and not narrowed to pinned notes.
        #[test]
        fn prop_pinned_group_only_in_mixed_lists(sort in any_sort(), pinned_only: bool, filter in any_filter()) {
            let grouped = shape(&filter, pinned_only, sort).group_pinned;
            let mixed = filter != Filter::Recent && filter != Filter::Pinned && !pinned_only;
            prop_assert_eq!(grouped, mixed);
        }

        /// Invariant: the list is under date headers exactly when its sort is
        /// a date, and the date it groups by is the one it sorts by.
        #[test]
        fn prop_headers_exactly_for_date_sorts(sort in any_sort(), pinned_only: bool, filter in any_filter()) {
            let shape = shape(&filter, pinned_only, sort);
            prop_assert_eq!(shape.by_date, shape.sort != Sort::Title);
            if shape.sort == Sort::CreatedAt {
                prop_assert_eq!(shape.date, DateKey::Created);
            } else {
                prop_assert_eq!(shape.date, DateKey::Updated);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 16, ..config() })]

        /// Model: narrowed to pinned notes, a place lists exactly its pinned
        /// notes, in the order the unnarrowed list has them.
        #[test]
        fn prop_pinned_only_keeps_the_pinned_in_order(
            notes in prop::collection::vec((any::<bool>(), 0i64..40), 0..14),
            descending: bool,
        ) {
            use kanso_core::DocType;
            use kanso_store::{Origin, Put, Store};
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open(dir.path().join("k.db")).unwrap();
            let book = store
                .put(DocType::Book, serde_json::json!({"name": "B"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            for (ix, (pinned, at)) in notes.iter().enumerate() {
                store
                    .put(
                        DocType::Note,
                        serde_json::json!({
                            "title": format!("n{ix:02}"), "bookId": book,
                            "pinned": pinned, "updatedAt": 1_000 + at,
                        }),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            let params = search_params(&Filter::All, "", Sort::UpdatedAt, descending);
            let all = list_notes(&store, &Filter::All, &params, false, 0).unwrap();
            let only = list_notes(&store, &Filter::All, &params, true, 0).unwrap();
            prop_assert!(only.iter().all(|n| n.pinned));
            let expected: Vec<&str> = all.iter().filter(|n| n.pinned).map(|n| n.id.as_str()).collect();
            let got: Vec<&str> = only.iter().map(|n| n.id.as_str()).collect();
            prop_assert_eq!(got, expected);
            prop_assert_eq!(only.len(), notes.iter().filter(|(p, _)| *p).count());
            // The Pinned place is the same list.
            let place = list_notes(&store, &Filter::Pinned, &params, false, 0).unwrap();
            prop_assert_eq!(place, only);
        }
    }
    /// A filter of every kind, over the notebooks `book:0..20` and a few tags.
    fn gen_filter() -> impl Strategy<Value = Filter> {
        prop_oneof![
            Just(Filter::All),
            Just(Filter::Pinned),
            Just(Filter::Recent),
            Just(Filter::Trash),
            Just(Filter::Tasks),
            (0usize..24).prop_map(|ix| Filter::Book(format!("book:{ix}"))),
            "[a-c]{1,2}".prop_map(Filter::Tag),
            "[a-c]{1,2}(/[a-c]{1,2})?".prop_map(Filter::TagPath),
            Just(Filter::Status(NoteStatus::Active)),
        ]
    }

    fn gen_narrowing() -> impl Strategy<Value = Narrowing> {
        (prop::option::of(Just(NoteStatus::Completed)), any::<bool>())
            .prop_map(|(status, pinned)| Narrowing { status, pinned })
    }

    /// Books, some of them locked.
    fn gen_locked_books() -> impl Strategy<Value = Vec<Book>> {
        (gen_books(), prop::collection::vec(any::<bool>(), 20)).prop_map(|(mut books, locks)| {
            for (book, locked) in books.iter_mut().zip(locks) {
                if locked {
                    book.extra.insert(
                        kanso_core::model::LOCK_FIELD.to_owned(),
                        serde_json::json!({}),
                    );
                }
            }
            books
        })
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariant: a new note goes into a notebook that exists and isn't
        /// the trash, and only a library with no notebook gets none.
        #[test]
        fn prop_new_note_goes_into_an_existing_notebook(
            books in gen_locked_books(),
            filter in gen_filter(),
            narrowing in gen_narrowing(),
            preferred in prop::option::of((0usize..24).prop_map(|ix| format!("book:{ix}"))),
        ) {
            let target = new_note_target(&filter, narrowing, preferred.as_deref(), &books);
            match &target.book_id {
                Some(id) => prop_assert!(books.iter().any(|b| &b.id == id) && id != "trash"),
                None => prop_assert!(books.is_empty()),
            }
        }

        /// A notebook's own list wins over the preference.
        #[test]
        fn prop_a_notebook_place_wins(
            books in gen_locked_books().prop_filter("some", |b| !b.is_empty()),
            at in 0usize..20,
            preferred in prop::option::of((0usize..24).prop_map(|ix| format!("book:{ix}"))),
        ) {
            let id = books[at % books.len()].id.clone();
            let target = new_note_target(&Filter::Book(id.clone()), Narrowing::default(), preferred.as_deref(), &books);
            prop_assert_eq!(target.book_id, Some(id));
        }

        /// Outside a notebook's list the preference is used when it exists
        /// and is unlocked, and is otherwise ignored for the first notebook.
        #[test]
        fn prop_a_locked_or_unknown_preference_falls_back(
            books in gen_locked_books(),
            preferred in (0usize..24).prop_map(|ix| format!("book:{ix}")),
        ) {
            let target = new_note_target(&Filter::All, Narrowing::default(), Some(&preferred), &books);
            let usable = books.iter().any(|b| b.id == preferred) && !is_book_locked(&preferred, &books);
            if usable {
                prop_assert_eq!(target.book_id.as_deref(), Some(preferred.as_str()));
            } else {
                prop_assert_eq!(target.book_id, books.first().map(|b| b.id.clone()));
            }
        }

        /// A tag, pinned or status place carries over to the note, and the
        /// list's narrowing is kept.
        #[test]
        fn prop_the_place_and_narrowing_carry_over(
            books in gen_books(),
            filter in gen_filter(),
            narrowing in gen_narrowing(),
        ) {
            let target = new_note_target(&filter, narrowing, None, &books);
            match &filter {
                Filter::Tag(tag) => prop_assert_eq!(&target.tags, &vec![tag.clone()]),
                Filter::TagPath(path) => prop_assert_eq!(target.tag_path.as_ref(), Some(path)),
                _ => {
                    prop_assert!(target.tags.is_empty());
                    prop_assert!(target.tag_path.is_none());
                }
            }
            prop_assert_eq!(target.pinned, narrowing.pinned || filter == Filter::Pinned);
            match &filter {
                Filter::Status(status) => {
                    prop_assert_eq!(target.status, narrowing.status.or(Some(*status)));
                }
                _ => prop_assert_eq!(target.status, narrowing.status),
            }
        }
    }
}
