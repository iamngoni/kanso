//! The note calls of [`Library`]: lists, counts, one note, saving, creating,
//! changing several notes, notebooks, tags, search, properties and query
//! views.
//!
//! A locked note shows its title and its lock in a list and nothing else
//! while it is closed (and its snippet, tags, status and tasks never, open or
//! not): its text is only ever returned by [`Library::note`] and read for
//! [`Library::properties`] and the preview when a key this session holds
//! opens it. A place that is made of what a note holds (a tag, a status, a
//! tag, status or tag search) leaves a locked note out.

use std::time::Instant;

use kanso_core::model::{BOOK_COLOR_FIELD, BOOK_ICON_FIELD, TRASH_BOOK_ID};
use kanso_core::query::{Clause, Field, Query};
use kanso_core::query_view::ViewNote;
use kanso_core::roles::{ROLE_FIELD, Role};
use kanso_core::{DocType, Note, NoteStatus, new_id, now_ms};
use kanso_engine::links::tags::apply_inline_tags;
use kanso_engine::list::{self, DAY_MS, DateKey, Filter, RECENT_DAYS};
use kanso_engine::lock::LockView;
use kanso_store::{NoteSummary, Origin, Put, SearchParams, Sort};
use serde_json::{Value, json};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::matching::ranges;
use crate::model::{
    Changed, DateField, Hit, LibraryOverview, ListPlace, ListQuery, ListShape, LockState, NewNote,
    NoteChange, NoteList, NoteRow, NoteView, NotebookRole, NotebookRow, Property, PropertyKind,
    Saved, SearchScope, SortKey, StatusCount, Table, TableRow, TagRow,
};
use crate::undo::Undoable;

impl Library {
    /// The notes a place lists, with the layout the desktop gives that list:
    /// pinned notes first (Recent is newest edit first, whatever the sort),
    /// narrowed to one status or to pinned notes when asked. A locked note is
    /// listed with its title and its lock; its snippet, tags, status and
    /// tasks are empty, and a place made of tags, a status or a tag search
    /// leaves it out.
    pub fn list(&self, query: ListQuery) -> Result<NoteList, KansoError> {
        guard(|| {
            let (notes, filter, sort) = self.listed(&query)?;
            let shape = list::shape(&filter, query.pinned_only, sort);
            Ok(NoteList {
                rows: notes.into_iter().map(row).collect(),
                shape: ListShape {
                    sort: match shape.sort {
                        Sort::UpdatedAt => SortKey::Updated,
                        Sort::CreatedAt => SortKey::Created,
                        Sort::Title => SortKey::Title,
                    },
                    by_date: shape.by_date,
                    date: match shape.date {
                        DateKey::Updated => DateField::Updated,
                        DateKey::Created => DateField::Created,
                    },
                    group_pinned: shape.group_pinned,
                },
            })
        })
    }

    /// The notes `query` lists, in order, as the store has them (a locked
    /// note's snippet and tags are for [`row`] to hide), and the filter and
    /// sort that made them.
    pub(crate) fn listed(
        &self,
        query: &ListQuery,
    ) -> Result<(Vec<NoteSummary>, Filter, Sort), KansoError> {
        let filter = match &query.place {
            ListPlace::All => Filter::All,
            ListPlace::Pinned => Filter::Pinned,
            ListPlace::Recent => Filter::Recent,
            ListPlace::Notebook { id } => Filter::Book(id.clone()),
            ListPlace::Tag { id } => Filter::Tag(id.clone()),
            ListPlace::Status { status } => Filter::Status(parse_status(status)?),
            ListPlace::TagPath { path } => Filter::TagPath(path.clone()),
            ListPlace::Trash => Filter::Trash,
        };
        let sort = match query.sort {
            SortKey::Updated => Sort::UpdatedAt,
            SortKey::Created => Sort::CreatedAt,
            SortKey::Title => Sort::Title,
        };
        let mut params = list::search_params(&filter, "", sort, query.descending);
        let status = match &query.status_only {
            Some(status) => Some(
                NoteStatus::parse(status)
                    .ok_or_else(|| KansoError::invalid(format!("“{status}” isn't a status.")))?,
            ),
            None => None,
        };
        list::refine_status(&mut params, status);
        let mut notes = list::list_notes(
            &self.inner.store,
            &filter,
            &params,
            query.pinned_only,
            recent_since(),
        )?;
        if status.is_some()
            || matches!(
                query.place,
                ListPlace::Tag { .. } | ListPlace::Status { .. } | ListPlace::TagPath { .. }
            )
        {
            notes.retain(|note| !note.locked);
        }
        Ok((notes, filter, sort))
    }

    /// How many notes each place of the library lists.
    pub fn overview(&self) -> Result<LibraryOverview, KansoError> {
        guard(|| {
            let counts = self.inner.store.status_counts()?;
            Ok(LibraryOverview {
                all: counts.all,
                pinned: counts.pinned,
                recent: self.inner.store.count_recent(recent_since())?,
                trash: counts.trash,
                by_status: NoteStatus::ALL
                    .into_iter()
                    .filter(|status| *status != NoteStatus::None)
                    .map(|status| StatusCount {
                        status: status.as_str().to_owned(),
                        count: counts.by_status.get(&status).copied().unwrap_or(0),
                    })
                    .collect(),
            })
        })
    }

    /// Makes one change to every note in `ids` in one write, and says how
    /// many notes it wrote (none when they were already as asked), the
    /// sentence for the notice that offers Undo, and the ticket for
    /// [`Library::undo`]. Pinning pins them all, or unpins them when they
    /// are all pinned.
    ///
    /// Nothing is written when an ID isn't a note ([`KansoError::NotFound`]),
    /// or when unlocked notes would go into a locked notebook or a note
    /// would leave one ([`KansoError::Locked`]): those go one at a time, with
    /// the lock open, as on the desktop. A locked note can be pinned, trashed
    /// and moved between unlocked notebooks.
    pub fn change(&self, ids: Vec<String>, change: NoteChange) -> Result<Changed, KansoError> {
        guard(|| {
            let applied = self.apply(ids, change)?;
            Ok(Changed {
                count: applied.count,
                undo: if applied.edits.is_empty() {
                    None
                } else {
                    Some(self.keep_undo(Undoable::Fields(applied.edits)))
                },
                message: applied.notice,
            })
        })
    }

    /// One note. Its text is `None` while it is locked and closed; reading
    /// a note in a locked notebook seals it first when a key is held.
    pub fn note(&self, id: &str) -> Result<NoteView, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            let state = self.reveal(&mut note);
            Ok(view(note, state))
        })
    }

    /// Replaces the note's text with `body`. `base_rev` is the revision the
    /// caller read: if the note changed since, nothing is written and the
    /// error says its revision now. A locked note takes it sealed, and stays
    /// open for another auto-lock time; a closed one refuses with
    /// [`KansoError::Locked`].
    pub fn save(&self, id: &str, base_rev: &str, body: String) -> Result<Saved, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            // Reading a note in a locked notebook seals it, which moves its
            // revision.
            let mut shown = note.clone();
            let state = self.reveal(&mut shown);
            if shown.rev != note.rev {
                note = self.stored(id)?;
            }
            if note.rev.as_deref() != Some(base_rev) {
                return Err(KansoError::Conflict {
                    current_rev: note.rev.unwrap_or_default(),
                });
            }
            if state.is_closed() {
                return Err(KansoError::Locked);
            }
            note.body = body;
            note.updated_at = now_ms();
            // The text's `#tags` are the note's tags. A locked note's stay
            // in its text: its tags are not shown.
            if state == LockView::Plain {
                apply_inline_tags(&self.inner.store, &mut note, None);
            }
            if state == LockView::Open {
                held(&self.inner.keys)
                    .seal(&mut note, Instant::now())
                    .map_err(|_| KansoError::Locked)?;
            }
            let value =
                serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
            match self
                .inner
                .store
                .put(DocType::Note, value, Put::Upsert, Origin::Ui)
            {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: self.stored(id).ok().and_then(|n| n.rev).unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    /// Makes a note. In a locked notebook it is sealed, which needs the lock
    /// open ([`KansoError::Locked`] otherwise).
    pub fn create(&self, new: NewNote) -> Result<Saved, KansoError> {
        guard(|| {
            let (book_id, tags, pinned) = self.destination(&new)?;
            let now = now_ms();
            let mut note: Note = serde_json::from_value(json!({
                "_id": new_id(DocType::Note),
                "title": new.title,
                "body": new.body,
                "bookId": book_id,
                "tags": tags,
                "pinned": pinned,
                "createdAt": now,
                "updatedAt": now,
            }))
            .map_err(|err| KansoError::invalid(err.to_string()))?;
            let in_locked_notebook = self.inner.store.locked_books()?.contains(&book_id);
            if !in_locked_notebook {
                apply_inline_tags(&self.inner.store, &mut note, None);
            }
            let stored = if in_locked_notebook {
                held(&self.inner.keys)
                    .seal_for_notebook(&mut note, &self.inner.store)
                    .map_err(|_| KansoError::Locked)?
            } else {
                note
            };
            let value = serde_json::to_value(&stored)
                .map_err(|err| KansoError::storage(err.to_string()))?;
            let written = self
                .inner
                .store
                .put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
            Ok(Saved {
                id: written.id,
                rev: written.rev,
            })
        })
    }

    /// Moves the note to the trash. A locked note stays locked there.
    pub fn trash(&self, id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.stored(id)?;
            self.inner.store.put(
                DocType::Note,
                json!({"_id": id, "bookId": TRASH_BOOK_ID}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(())
        })
    }

    /// The notebooks in tree order: each one is followed by the notebooks
    /// inside it, in the order the desktop's sidebar has them.
    pub fn notebooks(&self) -> Result<Vec<NotebookRow>, KansoError> {
        guard(|| Ok(notebook_rows(&self.inner.store.books()?)))
    }

    /// Makes a notebook, inside `parent_id` if given ([`KansoError::NotFound`]
    /// when there is no such notebook), and returns it as the sidebar lists
    /// it. The name is trimmed; an empty one is refused.
    pub fn create_notebook(
        &self,
        name: String,
        parent_id: Option<String>,
    ) -> Result<NotebookRow, KansoError> {
        guard(|| {
            let name = name.trim();
            if name.is_empty() {
                return Err(KansoError::invalid("A notebook needs a name."));
            }
            if let Some(parent) = &parent_id
                && !self
                    .inner
                    .store
                    .books()?
                    .iter()
                    .any(|book| &book.id == parent)
            {
                return Err(KansoError::NotFound);
            }
            let created = self.inner.store.put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent_id}),
                Put::Upsert,
                Origin::Ui,
            )?;
            notebook_rows(&self.inner.store.books()?)
                .into_iter()
                .find(|row| row.id == created.id)
                .ok_or(KansoError::NotFound)
        })
    }

    /// The tags, by name.
    pub fn tags(&self) -> Result<Vec<TagRow>, KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .tags()?
                .into_iter()
                .map(|tag| TagRow {
                    color: tag.color.rgb(),
                    id: tag.id,
                    name: tag.name,
                    count: tag.count,
                })
                .collect())
        })
    }

    /// Searches `scope` for `text`, which may use the search language
    /// (`tag:work "two words" -status:dropped`). The hits come with where
    /// the words matched. A locked note is found by its title; its text isn't
    /// searched, its snippet stays empty, and a search by tag or status
    /// (typed, or the Tags scope) leaves it out. `limit` 0 is no limit.
    ///
    /// - `All`: titles and texts (and the words in images).
    /// - `Titles`: the bare words must be in the title.
    /// - `Tags`: the bare words must name a tag, or a tag it is inside
    ///   (`work` finds `work/acme`).
    /// - `Notebook`: as `All`, within one notebook.
    pub fn search(
        &self,
        text: &str,
        scope: SearchScope,
        limit: u32,
    ) -> Result<Vec<Hit>, KansoError> {
        guard(|| {
            let typed = Query::parse(text);
            if typed.is_empty() {
                return Ok(Vec::new());
            }
            let query = scoped(typed, &scope);
            let by_content = query.clauses.iter().any(|clause| {
                matches!(
                    clause.field,
                    Some(Field::Tag | Field::TagId | Field::Status)
                )
            });
            let params = SearchParams {
                query: query.clone(),
                // A limit counts what is kept, so it comes after leaving
                // locked notes out.
                limit: (limit > 0 && !by_content).then_some(limit as usize),
                ocr: true,
                ..SearchParams::default()
            };
            let title_terms: Vec<&str> = query.terms_for(Field::Title).collect();
            let body_terms: Vec<&str> = query.terms_for(Field::Body).collect();
            let mut found = self.inner.store.search_summaries(&params)?;
            if by_content {
                found.retain(|summary| !summary.locked);
                if limit > 0 {
                    found.truncate(limit as usize);
                }
            }
            Ok(found
                .into_iter()
                .map(|summary| {
                    let row = row(summary);
                    Hit {
                        title_ranges: ranges(&row.title, &title_terms),
                        snippet_ranges: ranges(&row.snippet, &body_terms),
                        row,
                    }
                })
                .collect())
        })
    }

    /// The properties in a note's front matter. Empty while the note is
    /// locked and closed.
    pub fn properties(&self, id: &str) -> Result<Vec<Property>, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            if self.reveal(&mut note).is_closed() {
                return Ok(Vec::new());
            }
            Ok(kanso_core::properties::parse(&note.body)
                .into_iter()
                .map(|(key, value)| property_row(key, &value))
                .collect())
        })
    }

    /// Runs a query view (the body of a `query` fence) and returns its
    /// table. Locked notes and the trash are never listed; `self_id` leaves
    /// out the note the query is in.
    pub fn query_view(&self, source: &str, self_id: Option<String>) -> Result<Table, KansoError> {
        guard(|| {
            let spec = kanso_core::query_view::parse(source)
                .map_err(|err| KansoError::invalid(err.message))?;
            let notes: Vec<ViewNote> = self
                .inner
                .store
                .query_view_notes()?
                .into_iter()
                .map(ViewNote::from)
                .collect();
            let rows = kanso_core::query_view::evaluate(&spec, &notes, self_id.as_deref());
            Ok(Table {
                columns: spec.labels(),
                rows: rows
                    .into_iter()
                    .map(|row| TableRow {
                        id: row.id,
                        title: row.title,
                        cells: row
                            .cells
                            .iter()
                            .map(|cell| cell.as_ref().map(kanso_core::properties::Value::text))
                            .collect(),
                    })
                    .collect(),
            })
        })
    }

    /// The note as the store has it (sealed text empty), or not found.
    pub(crate) fn stored(&self, id: &str) -> Result<Note, KansoError> {
        if DocType::of_id(id) != Some(DocType::Note) {
            return Err(KansoError::NotFound);
        }
        self.inner.store.note(id)?.ok_or(KansoError::NotFound)
    }

    /// Opens `note`'s text when this session's keys do, and says what its
    /// lock allows. A closed note's text is emptied.
    pub(crate) fn reveal(&self, note: &mut Note) -> LockView {
        held(&self.inner.keys).reveal(note, Some(&self.inner.store))
    }

    /// The text of note `id` if it can be read now.
    pub(crate) fn readable_text(&self, id: &str) -> Result<Option<(Note, String)>, KansoError> {
        let mut note = self.stored(id)?;
        if self.reveal(&mut note).is_closed() {
            return Ok(None);
        }
        let text = note.body.clone();
        Ok(Some((note, text)))
    }

    /// The notebook, tags and pin a new note starts with: what the engine
    /// decides for its place and preference, and the explicit notebook
    /// (which must exist) over it.
    fn destination(&self, new: &NewNote) -> Result<(String, Vec<String>, bool), KansoError> {
        let books = self.inner.store.books()?;
        let filter = match &new.place {
            None | Some(ListPlace::All) => Filter::All,
            Some(ListPlace::Pinned) => Filter::Pinned,
            Some(ListPlace::Recent) => Filter::Recent,
            Some(ListPlace::Notebook { id }) => Filter::Book(id.clone()),
            Some(ListPlace::Tag { id }) => Filter::Tag(id.clone()),
            Some(ListPlace::Status { status }) => Filter::Status(parse_status(status)?),
            Some(ListPlace::TagPath { path }) => Filter::TagPath(path.clone()),
            Some(ListPlace::Trash) => Filter::Trash,
        };
        let target = list::new_note_target(
            &filter,
            list::Narrowing {
                status: None,
                pinned: new.pinned,
            },
            new.preferred_notebook_id.as_deref(),
            &books,
        );
        let book_id = match (&new.notebook_id, target.book_id) {
            (Some(id), _) => books
                .iter()
                .find(|book| &book.id == id)
                .map(|book| book.id.clone())
                .ok_or(KansoError::NotFound)?,
            (None, Some(id)) => id,
            (None, None) => {
                self.inner
                    .store
                    .put(
                        DocType::Book,
                        json!({"name": list::FALLBACK_BOOK_NAME}),
                        Put::Upsert,
                        Origin::Ui,
                    )?
                    .id
            }
        };
        let mut tags = target.tags;
        for tag in &new.tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
        Ok((book_id, tags, target.pinned))
    }
}

/// The time Recent reaches back to.
fn recent_since() -> i64 {
    now_ms() - RECENT_DAYS * DAY_MS
}

/// `query` as `scope` narrows it: the bare words of Titles are title words,
/// those of Tags are tag names, and a notebook's search is in that notebook.
fn scoped(mut query: Query, scope: &SearchScope) -> Query {
    let field = match scope {
        SearchScope::Titles => Some(Field::Title),
        SearchScope::Tags => Some(Field::Tag),
        SearchScope::All | SearchScope::Notebook { .. } => None,
    };
    if let Some(field) = field {
        for clause in &mut query.clauses {
            if clause.field.is_none() {
                clause.field = Some(field);
            }
        }
    }
    if let SearchScope::Notebook { id } = scope {
        query
            .clauses
            .push(Clause::field(Field::BookId, id.as_str()));
    }
    query
}

/// A status by its stored name, or [`KansoError::Invalid`].
pub(crate) fn parse_status(status: &str) -> Result<NoteStatus, KansoError> {
    NoteStatus::parse(status)
        .ok_or_else(|| KansoError::invalid(format!("“{status}” isn't a status.")))
}

/// The notebooks as rows, in tree order.
pub(crate) fn notebook_rows(books: &[kanso_core::Book]) -> Vec<NotebookRow> {
    let tree = list::notebook_tree(books);
    let depths: Vec<usize> = tree.iter().map(|(depth, _)| *depth).collect();
    tree.into_iter()
        .enumerate()
        .map(|(at, (depth, book))| NotebookRow {
            has_children: depths.get(at + 1).is_some_and(|next| *next > depth),
            locked: book.is_locked(),
            icon: book
                .extra
                .get(BOOK_ICON_FIELD)
                .and_then(Value::as_str)
                .map(str::to_owned),
            color: book
                .extra
                .get(BOOK_COLOR_FIELD)
                .and_then(Value::as_str)
                .map(str::to_owned),
            role: Role::of_book(&book.id, book.extra.get(ROLE_FIELD).and_then(Value::as_str)).map(
                |role| match role {
                    Role::Inbox => NotebookRole::Inbox,
                    Role::Journal => NotebookRole::Journal,
                    Role::Templates => NotebookRole::Templates,
                },
            ),
            depth: depth as u32,
            id: book.id.clone(),
            name: book.name.clone(),
            parent_id: book.parent_book_id.clone(),
            count: book.count,
        })
        .collect()
}

/// A list row. A locked note's row is its title and its lock: whatever the
/// store holds of its snippet, tags, status and tasks, none of it is passed
/// on.
pub(crate) fn row(summary: kanso_store::NoteSummary) -> NoteRow {
    let locked = summary.locked;
    NoteRow {
        id: summary.id,
        rev: summary.rev,
        title: summary.title,
        snippet: if locked {
            String::new()
        } else {
            summary.snippet
        },
        notebook_id: summary.book_id,
        status: if locked {
            NoteStatus::None.as_str().to_owned()
        } else {
            summary.status.as_str().to_owned()
        },
        tags: if locked { Vec::new() } else { summary.tags },
        pinned: summary.pinned,
        created_ms: summary.created_at,
        updated_ms: summary.updated_at,
        tasks_done: if locked { 0 } else { summary.tasks_done },
        tasks_total: if locked { 0 } else { summary.tasks_total },
        locked,
    }
}

fn view(note: Note, state: LockView) -> NoteView {
    let lock = match state {
        LockView::Plain => LockState::Plain,
        LockView::Open => LockState::Open,
        LockView::Closed(reason) => LockState::Closed { reason },
    };
    // A closed note is its title and lock, as in the lists: its tags and
    // status are its content too.
    let closed = lock.is_closed();
    NoteView {
        id: note.id,
        rev: note.rev.unwrap_or_default(),
        title: note.title,
        body: (!closed).then_some(note.body),
        lock,
        notebook_id: note.book_id,
        tags: if closed { Vec::new() } else { note.tags },
        status: if closed {
            NoteStatus::None.as_str().to_owned()
        } else {
            note.status.as_str().to_owned()
        },
        pinned: note.pinned,
        created_ms: note.created_at,
        updated_ms: note.updated_at,
    }
}

pub(crate) fn property_row(key: String, value: &kanso_core::properties::Value) -> Property {
    use kanso_core::properties::Value as V;
    let (kind, items) = match value {
        V::Text(_) => (PropertyKind::Text, Vec::new()),
        V::Number(_) => (PropertyKind::Number, Vec::new()),
        V::Bool(_) => (PropertyKind::Bool, Vec::new()),
        V::Date(_) => (PropertyKind::Date, Vec::new()),
        V::List(items) => (PropertyKind::List, items.iter().map(V::text).collect()),
        V::Raw(_) => (PropertyKind::Raw, Vec::new()),
    };
    Property {
        key,
        kind,
        text: value.text(),
        items,
    }
}
