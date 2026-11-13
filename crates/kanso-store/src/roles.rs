//! Notebooks with a job (Journal, Inbox, Templates), daily notes and notes
//! made from templates. See `kanso_core::roles` for the fixed IDs.
//!
//! Getting a role notebook or a daily note makes it when it is missing,
//! inside one write transaction, so two callers (the app and the local API)
//! never both make it.

use kanso_core::dates::{Date, DateTime};
use kanso_core::model::{Note, TRASH_BOOK_ID};
use kanso_core::roles::{DAILY_PREFIX, ROLE_FIELD, Role, daily_id, daily_title};
use kanso_core::template;
use kanso_core::{DocType, now_ms};
use rusqlite::Transaction;
use serde_json::{Map, Value, json};

use crate::validate::{Refs, normalize};
use crate::write::{Synced, load_row, next_rev, store_row};
use crate::{Change, Origin, Result, Store, StoreError, WriteResult};

/// What a new note is made with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewNote {
    pub title: String,
    pub body: String,
    /// Tag IDs; each must exist.
    pub tags: Vec<String>,
}

/// A daily note found or made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyNote {
    pub id: String,
    /// Made now, rather than found.
    pub created: bool,
    /// Where a new note's template put `{{cursor}}`, as a byte offset into
    /// its body.
    pub cursor: Option<usize>,
}

/// The Journal's field naming the template new daily notes are made from.
pub const DAILY_TEMPLATE_FIELD: &str = "kansoDailyTemplate";

/// A note made from a template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FromTemplate {
    pub id: String,
    pub rev: String,
    pub title: String,
    /// Where `{{cursor}}` was in the body, as a byte offset.
    pub cursor: Option<usize>,
}

/// Writes `fields` as a new live document `id` (over a tombstone, if
/// there is one) and returns its revision.
pub(crate) fn create(
    tx: &Transaction,
    doc_type: DocType,
    id: &str,
    fields: Map<String, Value>,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    let previous = load_row(tx, id)?;
    if previous.as_ref().is_some_and(|p| !p.deleted) {
        return Err(StoreError::Conflict);
    }
    let fields = normalize(doc_type, id, fields, None, now_ms(), Refs::Check(tx))?;
    let rev = next_rev(previous.as_ref(), &Value::Object(fields.clone()))?;
    store_row(
        tx,
        id,
        doc_type,
        &rev,
        false,
        &fields,
        previous.as_ref(),
        Synced::Keep,
        origin,
        changes,
    )?;
    Ok(rev)
}

/// The notebook with `role` inside a write, made when missing: the one with
/// the fixed ID, else any live notebook whose `kansoRole` says so.
pub(crate) fn role_book_in(
    tx: &Transaction,
    role: Role,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    let id = role.book_id();
    if load_row(tx, id)?.is_some_and(|row| !row.deleted && row.doc_type == DocType::Book) {
        return Ok(id.to_owned());
    }
    let other: Option<String> = tx
        .query_row(
            "SELECT id FROM docs WHERE type = 'book' AND deleted = 0
             AND json_extract(json, '$.kansoRole') = ? ORDER BY id LIMIT 1",
            [role.as_str()],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            e => Err(e),
        })?;
    if let Some(other) = other {
        return Ok(other);
    }
    let fields = json!({
        "name": role.default_name(),
        "parentBookId": null,
        ROLE_FIELD: role.as_str(),
    });
    let Value::Object(fields) = fields else {
        unreachable!("a JSON object");
    };
    create(tx, DocType::Book, id, fields, origin, changes)?;
    Ok(id.to_owned())
}

impl Store {
    /// The notebook with `role`, made with its default name when there is
    /// none. Returns its ID.
    pub fn role_book(&self, role: Role, origin: Origin) -> Result<String> {
        self.write(|tx, changes| role_book_in(tx, role, origin, changes))
    }

    /// The live notebook with `role`, without making it.
    pub fn find_role_book(&self, role: Role) -> Result<Option<String>> {
        self.read(|conn| {
            let row: Option<String> = conn
                .query_row(
                    "SELECT id FROM docs WHERE type = 'book' AND deleted = 0
                     AND (id = ?1 OR json_extract(json, '$.kansoRole') = ?2)
                     ORDER BY id = ?1 DESC, id LIMIT 1",
                    [role.book_id(), role.as_str()],
                    |row| row.get(0),
                )
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    e => Err(e),
                })?;
            Ok(row)
        })
    }

    /// The daily note for `date`, made in the Journal notebook (made too,
    /// if needed) when there is none, with the title `YYYY-MM-DD` and
    /// `new`'s body and tags. A daily note in the trash comes back to the
    /// Journal; one moved to another notebook stays there.
    pub fn daily_note(&self, date: Date, new: &NewNote, origin: Origin) -> Result<DailyNote> {
        let id = daily_id(date);
        self.write(|tx, changes| {
            let existing = load_row(tx, &id)?.filter(|row| !row.deleted);
            if let Some(row) = existing {
                if row.fields.get("bookId").and_then(Value::as_str) == Some(TRASH_BOOK_ID) {
                    let journal = role_book_in(tx, Role::Journal, origin, changes)?;
                    crate::write::rewrite_note(tx, &id, origin, changes, |fields| {
                        fields.insert("bookId".into(), journal.into());
                        fields.insert("updatedAt".into(), now_ms().into());
                    })?;
                }
                return Ok(DailyNote {
                    id,
                    created: false,
                    cursor: None,
                });
            }
            let journal = role_book_in(tx, Role::Journal, origin, changes)?;
            let title = if new.title.trim().is_empty() {
                daily_title(date)
            } else {
                new.title.clone()
            };
            let mut fields = Map::new();
            fields.insert("title".into(), title.into());
            fields.insert("body".into(), new.body.clone().into());
            fields.insert("bookId".into(), journal.into());
            fields.insert("tags".into(), new.tags.clone().into());
            create(tx, DocType::Note, &id, fields, origin, changes)?;
            Ok(DailyNote {
                id,
                created: true,
                cursor: None,
            })
        })
    }

    /// The template new daily notes are made from, as the Journal names it
    /// (it syncs with the Journal), when that note is still there.
    pub fn daily_template(&self) -> Result<Option<String>> {
        let Some(journal) = self.find_role_book(Role::Journal)? else {
            return Ok(None);
        };
        let named = self
            .get(&journal)?
            .and_then(|doc| doc.fields.get(DAILY_TEMPLATE_FIELD).cloned())
            .and_then(|v| v.as_str().map(str::to_owned));
        Ok(match named {
            Some(id) if self.note(&id)?.is_some_and(|n| !n.is_trashed()) => Some(id),
            _ => None,
        })
    }

    /// Note `id` as a template is read: a locked one (sealed, or in a locked
    /// notebook) gives its title and tags but none of its text, so copying
    /// it never brings its words out of the lock.
    fn template_note(&self, id: &str) -> Result<Option<Note>> {
        let Some(mut note) = self.note(id)? else {
            return Ok(None);
        };
        if note.is_locked() || self.note_is_locked(id)? {
            note.body.clear();
        }
        Ok(Some(note))
    }

    /// Makes `template` (a note's ID, or `None` for none) the one new daily
    /// notes are made from; the Journal is made if needed.
    pub fn set_daily_template(&self, template: Option<&str>, origin: Origin) -> Result<()> {
        let journal = self.role_book(Role::Journal, origin)?;
        self.put(
            DocType::Book,
            json!({"_id": journal, DAILY_TEMPLATE_FIELD: template}),
            crate::Put::Merge,
            origin,
        )?;
        Ok(())
    }

    /// Opens the daily note for `now.date`, making it when missing from the
    /// daily template (filled in for that day, at `now`'s time) or empty.
    pub fn open_daily(&self, now: DateTime, origin: Origin) -> Result<DailyNote> {
        let title = daily_title(now.date);
        let mut cursor = None;
        let new = match self
            .daily_template()?
            .map(|id| self.template_note(&id))
            .transpose()?
        {
            Some(Some(template)) => {
                let (_, body) =
                    template::instantiate(&template.title, &template.body, Some(&title), now);
                cursor = body.cursor;
                NewNote {
                    title,
                    body: body.text,
                    tags: template.tags,
                }
            }
            _ => NewNote {
                title,
                ..NewNote::default()
            },
        };
        let mut daily = self.daily_note(now.date, &new, origin)?;
        if daily.created {
            daily.cursor = cursor;
        }
        Ok(daily)
    }

    /// The days that have a daily note outside the trash, oldest first.
    pub fn daily_dates(&self) -> Result<Vec<Date>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                 AND IFNULL(book_id, '') != 'trash' AND substr(id, 1, ?2) = ?1",
            )?;
            let ids = stmt.query_map(
                rusqlite::params![DAILY_PREFIX, DAILY_PREFIX.len() as i64],
                |row| row.get::<_, String>(0),
            )?;
            let mut dates: Vec<Date> = ids
                .filter_map(|id| id.ok())
                .filter_map(|id| kanso_core::roles::daily_date(&id))
                .collect();
            dates.sort();
            Ok(dates)
        })
    }

    /// The notes in the Templates notebook (and the notebooks inside it),
    /// outside the trash, as `(id, title)` sorted by title.
    pub fn templates(&self) -> Result<Vec<(String, String)>> {
        let Some(book) = self.find_role_book(Role::Templates)? else {
            return Ok(Vec::new());
        };
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "WITH RECURSIVE inside(id) AS (
                     SELECT ?1
                     UNION SELECT b.id FROM docs b JOIN inside ON b.parent_book_id = inside.id
                     WHERE b.type = 'book' AND b.deleted = 0)
                 SELECT d.id, IFNULL(d.title, '') FROM docs d
                 WHERE d.type = 'note' AND d.deleted = 0 AND d.book_id IN (SELECT id FROM inside)
                 ORDER BY lower(d.title), d.id",
            )?;
            let rows = stmt.query_map([book], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Makes a note from template `template_id`, its variables filled in
    /// for `now` (see `kanso_core::template`). The title is `title`, or the
    /// template's own title filled in; the note goes into `book` (the
    /// Inbox, made if needed, when `None`) with the template's tags, and
    /// remembers its template in `sourceTemplateId`.
    pub fn create_from_template(
        &self,
        template_id: &str,
        title: Option<&str>,
        book: Option<&str>,
        now: DateTime,
        origin: Origin,
    ) -> Result<FromTemplate> {
        let template = self
            .template_note(template_id)?
            .ok_or(StoreError::NotFound)?;
        let (title, body) = template::instantiate(&template.title, &template.body, title, now);
        let book = match book {
            Some(book) => book.to_owned(),
            None => self.role_book(Role::Inbox, origin)?,
        };
        let fields = json!({
            "title": title,
            "body": body.text,
            "bookId": book,
            "tags": template.tags,
            "sourceTemplateId": template.id,
        });
        let WriteResult { id, rev } =
            self.put(DocType::Note, fields, crate::Put::Upsert, origin)?;
        Ok(FromTemplate {
            id,
            rev,
            title,
            cursor: body.cursor,
        })
    }

    /// Every `(note ID, tag ID)` pair of live notes outside the trash, for
    /// the tag tree's counts.
    pub fn tag_links(&self) -> Result<Vec<(String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT nt.note_id, nt.tag_id FROM note_tags nt
                 JOIN docs d ON d.id = nt.note_id AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'",
            )?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The tag named `name` (ignoring ASCII case), made when there is none.
    /// Returns its ID.
    pub fn tag_named(&self, name: &str, origin: Origin) -> Result<String> {
        let name = name.trim();
        self.write(|tx, changes| {
            let found: Option<String> = tx
                .query_row(
                    "SELECT id FROM docs WHERE type = 'tag' AND deleted = 0
                     AND lower(json_extract(json, '$.name')) = lower(?) ORDER BY id LIMIT 1",
                    [name],
                    |row| row.get(0),
                )
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    e => Err(e),
                })?;
            if let Some(id) = found {
                return Ok(id);
            }
            let id = crate::write::fresh_id(tx, DocType::Tag)?;
            let mut fields = Map::new();
            fields.insert("name".into(), name.into());
            create(tx, DocType::Tag, &id, fields, origin, changes)?;
            Ok(id)
        })
    }
}
