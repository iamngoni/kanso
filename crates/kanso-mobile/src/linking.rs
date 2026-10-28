//! Links and getting around: a note's outline, its backlinks and unlinked
//! mentions, Link on a mention, Open Note (⌘P) and the command palette
//! (⌘K). The decisions are `kanso_engine`'s (`outline`, `navigate`,
//! `palette`, `links::mention`); this keeps the locked notes out and hands
//! back plain rows.
//!
//! While the lock session is closed, a locked note (sealed, or in a locked
//! notebook) is in none of it: not as a backlink, a mention, an Open Note
//! result or a palette result, by title or by heading. Once the session is
//! open it is found like any other note, from the text the session opens.

use std::collections::BTreeSet;
use std::sync::Arc;

use kanso_core::model::TRASH_BOOK_ID;
use kanso_engine::links::mention;
use kanso_engine::navigate::model::{LockedNotes, OpenedNote};
use kanso_engine::navigate::{backlinks, quick_open};
use kanso_engine::outline::{self, Heading};
use kanso_engine::palette;
use kanso_engine::palette::model as engine;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::linking_model::*;
use crate::model::Saved;

/// Links and navigation of one library.
#[derive(Clone)]
pub struct Linking {
    pub(crate) inner: Arc<Inner>,
}

impl Linking {
    fn library(&self) -> Library {
        Library {
            inner: self.inner.clone(),
        }
    }

    /// The headings of `text`, in order: the outline of the note the shell
    /// holds the text of, so a locked note that is open has one too, and
    /// its text never goes back to Rust to be stored for this.
    pub fn outline(&self, text: &str) -> Vec<OutlineHeading> {
        outline::headings(text).into_iter().map(heading).collect()
    }

    /// The heading the caret on `line` (0-based) is under: the last one at
    /// or above it, as an index into `headings`.
    pub fn current_heading(&self, headings: &[OutlineHeading], line: u32) -> Option<u32> {
        let headings: Vec<Heading> = headings
            .iter()
            .map(|h| Heading {
                level: h.level,
                text: h.text.clone(),
                line: h.line,
            })
            .collect();
        outline::current_heading(&headings, line).map(|ix| ix as u32)
    }

    /// The notes that link to note `id`, most recently changed first. Empty
    /// while `id` is a locked note that is closed.
    pub fn backlinks(&self, id: &str) -> Result<Vec<LinkRef>, KansoError> {
        guard(|| {
            let locks = self.locked_notes()?;
            let found = backlinks::backlinks(&self.inner.store, id, &locks)?;
            Ok(found.into_iter().map(link_ref).collect())
        })
    }

    /// The notes that say note `id`'s title as words of their own without
    /// linking to it, most recently changed first.
    pub fn unlinked_mentions(&self, id: &str) -> Result<Vec<LinkRef>, KansoError> {
        guard(|| {
            let locks = self.locked_notes()?;
            let found = backlinks::unlinked_mentions(&self.inner.store, id, &locks)?;
            Ok(found.into_iter().map(link_ref).collect())
        })
    }

    /// Makes the mention on `line` (0-based) in note `mention_id` a link to
    /// note `target_id`, in one write; the words stay as written. `None`
    /// when that line has no mention any more (nothing is written), and
    /// [`KansoError::Locked`] for a locked note that is closed. The note is
    /// saved as an edit would be, so a locked note stays sealed.
    pub fn link_mention(
        &self,
        target_id: &str,
        mention_id: &str,
        line: u32,
    ) -> Result<Option<Saved>, KansoError> {
        guard(|| {
            let library = self.library();
            if self.inner.store.note_is_locked(target_id)?
                && !self.locked_notes()?.can_read(target_id)
            {
                return Err(KansoError::Locked);
            }
            let Some((note, text)) = library.readable_text(mention_id)? else {
                return Err(KansoError::Locked);
            };
            let Some(body) = mention::mention_linked(&self.inner.store, target_id, &text, line)?
            else {
                return Ok(None);
            };
            let rev = note.rev.clone().unwrap_or_default();
            library.save(mention_id, &rev, body).map(Some)
        })
    }

    /// Open Note: the notes (the most recently edited ones for an empty
    /// `query`) and the headings that fit it, in sections to list. `open` is
    /// the note open now, which is checked.
    pub fn quick_open(
        &self,
        query: &str,
        open: Option<String>,
    ) -> Result<Vec<PaletteSection>, KansoError> {
        guard(|| {
            let locks = self.locked_notes()?;
            let found = quick_open::search(&self.inner.store, query, true, &locks)?;
            let names = self.book_names()?;
            let sections = quick_open::sections(query, &found, open.as_deref(), &names);
            Ok(sections.into_iter().map(section).collect())
        })
    }

    /// The command palette for `query`: the commands the app has handlers
    /// for and that apply now, the places of the library, and, once something
    /// is typed, the notes Open Note would find.
    pub fn palette(
        &self,
        query: &str,
        context: PaletteContext,
    ) -> Result<Vec<PaletteSection>, KansoError> {
        guard(|| {
            let ctx = self.engine_context(query, context)?;
            Ok(palette::search(query, &ctx)
                .into_iter()
                .map(section)
                .collect())
        })
    }

    fn engine_context(
        &self,
        query: &str,
        context: PaletteContext,
    ) -> Result<engine::Context, KansoError> {
        let store = &self.inner.store;
        let note = match &context.open_note_id {
            Some(id) => {
                let note = store.note(id)?;
                match note {
                    Some(note) => Some(engine::OpenNote {
                        id: note.id.clone(),
                        locked: store.note_is_locked(id)?,
                        trashed: note.book_id == TRASH_BOOK_ID,
                        pinned: note.pinned,
                    }),
                    None => None,
                }
            }
            None => None,
        };
        let counts = store.status_counts()?;
        let library = self.library();
        let books = library
            .notebooks()?
            .into_iter()
            .map(|book| engine::BookSource {
                id: book.id,
                name: book.name,
                depth: book.depth,
                count: book.count,
            })
            .collect();
        let places = engine::PlaceSources {
            all: counts.all,
            pinned: counts.pinned,
            recent: library.overview()?.recent,
            overdue: context.overdue_tasks,
            trash: counts.trash,
            shared: store.shared_with_me()?.len() as u32,
            by_status: counts
                .by_status
                .iter()
                .map(|(status, count)| (status.as_str().to_owned(), *count))
                .collect(),
            books,
            showing: context.showing.map(place_in),
        };
        let notes = if query.trim().is_empty() {
            Vec::new()
        } else {
            let locks = self.locked_notes()?;
            quick_open::search(store, query, false, &locks)?.notes
        };
        Ok(engine::Context {
            note,
            signed_in: context.signed_in,
            session_held: context.session_held,
            ai_on: context.ai_on,
            settings_open: context.settings_open,
            image_line: context.image_line,
            audio_line: context.audio_line,
            shared_note: context.shared_note,
            handlers: context.handlers,
            places,
            notes,
            book_names: self.book_names()?,
        })
    }

    /// Every notebook's name by ID.
    fn book_names(&self) -> Result<Vec<(String, String)>, KansoError> {
        Ok(self
            .inner
            .store
            .books()?
            .into_iter()
            .map(|book| (book.id, book.name))
            .collect())
    }

    /// The locked notes this session has opened, with their text; none while
    /// it is closed.
    fn locked_notes(&self) -> Result<LockedNotes, KansoError> {
        if !held(&self.inner.keys).is_unlocked() {
            return Ok(LockedNotes::closed());
        }
        let store = &self.inner.store;
        let library = self.library();
        let mut ids: BTreeSet<String> = BTreeSet::new();
        ids.extend(store.locked_notes()?.into_iter().map(|n| n.id));
        ids.extend(store.unsealed_in_locked_books()?.into_iter().map(|n| n.id));
        let mut opened = Vec::new();
        for id in ids {
            let mut note = library.stored(&id)?;
            if note.book_id == TRASH_BOOK_ID || library.reveal(&mut note).is_closed() {
                continue;
            }
            opened.push(OpenedNote {
                id: note.id,
                title: note.title,
                body: note.body,
                book_id: note.book_id,
                updated_at: note.updated_at,
            });
        }
        Ok(LockedNotes { opened })
    }
}

fn heading(h: Heading) -> OutlineHeading {
    OutlineHeading {
        level: h.level,
        text: h.text,
        line: h.line,
    }
}

fn link_ref(b: kanso_store::Backlink) -> LinkRef {
    LinkRef {
        id: b.id,
        title: b.title,
        line: b.line,
        context: b.context,
    }
}

fn section(s: engine::Section) -> PaletteSection {
    PaletteSection {
        label: s.label,
        entries: s.entries.into_iter().map(entry).collect(),
    }
}

fn entry(e: engine::Entry) -> PaletteEntry {
    PaletteEntry {
        target: match e.target {
            engine::Target::Command { id, picker } => PaletteTarget::Command {
                id,
                picker: picker.map(picker_out),
            },
            engine::Target::Place(place) => PaletteTarget::Place {
                place: place_out(place),
            },
            engine::Target::Note { id } => PaletteTarget::Note { id },
            engine::Target::Heading { note_id, line } => PaletteTarget::Heading { note_id, line },
        },
        label: e.label,
        icon: e.icon,
        description: e.description,
        hint: e.hint,
        checked: e.checked,
        depth: e.depth,
        status: e.status,
    }
}

fn picker_out(p: engine::Picker) -> PalettePicker {
    match p {
        engine::Picker::Notebook => PalettePicker::Notebook,
        engine::Picker::Tag => PalettePicker::Tag,
        engine::Picker::Status => PalettePicker::Status,
        engine::Picker::GoToNotebook => PalettePicker::GoToNotebook,
        engine::Picker::GoToTag => PalettePicker::GoToTag,
        engine::Picker::WordGoal => PalettePicker::WordGoal,
        engine::Picker::Template => PalettePicker::Template,
        engine::Picker::ImageSize => PalettePicker::ImageSize,
    }
}

fn place_out(p: engine::Place) -> PalettePlace {
    match p {
        engine::Place::AllNotes => PalettePlace::AllNotes,
        engine::Place::Pinned => PalettePlace::Pinned,
        engine::Place::Recent => PalettePlace::Recent,
        engine::Place::Tasks => PalettePlace::Tasks,
        engine::Place::Notebook(id) => PalettePlace::Notebook { id },
        engine::Place::Status(name) => PalettePlace::Status { name },
        engine::Place::Shared => PalettePlace::Shared,
        engine::Place::Trash => PalettePlace::Trash,
    }
}

fn place_in(p: PalettePlace) -> engine::Place {
    match p {
        PalettePlace::AllNotes => engine::Place::AllNotes,
        PalettePlace::Pinned => engine::Place::Pinned,
        PalettePlace::Recent => engine::Place::Recent,
        PalettePlace::Tasks => engine::Place::Tasks,
        PalettePlace::Notebook { id } => engine::Place::Notebook(id),
        PalettePlace::Status { name } => engine::Place::Status(name),
        PalettePlace::Shared => engine::Place::Shared,
        PalettePlace::Trash => engine::Place::Trash,
    }
}
