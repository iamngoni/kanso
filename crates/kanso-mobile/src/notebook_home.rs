//! Notebook homepages on the phone (issue #201): the card above a
//! notebook's notes, the homepage a tap opens, its map, the Home note and
//! the settings. The page, its sections and the AI writing are the
//! engine's (`kanso_engine::pages::notebook`), which the desktop uses too;
//! this is the shell's side: the settings file, the model call, the lock
//! session and the plain data.
//!
//! A locked notebook shows nothing until its password is typed this
//! session; then its computed sections are built in memory from the opened
//! notes and dropped with them. No summary is written for it, and none is
//! kept on disk.

use kanso_core::home::SECTION_KEYS;
use kanso_core::now_ms;
use kanso_engine::pages::notebook::{
    FIRST_TASKS, HomeOptions, NotebookPage, open_home, title_of,
};
use kanso_engine::pages::notebook_actions as actions;
use kanso_engine::pages::notebook_model::{
    ADD_HOME_NOTE, AI_OFF_LINE, ASK_BUTTON, FIRST_VISIT_BUTTON, HOME_MOVED_LINE, HOME_ROW,
    LOCKED_LINE, NO_OPEN_TASKS, OPEN_HOME, OpenedNote,
};
use kanso_engine::pages::{
    self, RefreshPolicy, locked_home::locked_home, write::WriteContext,
};
use kanso_store::Store;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::notebook_home_model::{
    HomeCandidate, HomeCard, HomeHeaderInfo, HomeMap, HomeMapEdge, HomeMapNode, HomePanel,
    HomeSectionSetting, HomeSettings, HomeWords,
};
use crate::people::{People, section_of};
use crate::people_model::PageHidden;

/// The notebook homepage calls of one library.
#[derive(Clone)]
pub struct NotebookHome {
    people: People,
}

impl Library {
    /// Notebook homepages: the card, the page, the map, the Home note.
    pub fn notebook_home(&self) -> NotebookHome {
        NotebookHome {
            people: self.people(),
        }
    }

    /// [`Library::notebook_home`] with the AI handle `ai` (a test's, which
    /// answers without a network).
    #[cfg(any(test, feature = "test-support"))]
    pub fn notebook_home_with_ai(&self, ai: crate::ai::AiHandle) -> NotebookHome {
        NotebookHome {
            people: self.people_with_ai(ai),
        }
    }
}

impl NotebookHome {
    fn store(&self) -> &Store {
        self.people.store()
    }

    fn library(&self) -> Library {
        Library {
            inner: self.people.ai.inner.clone(),
        }
    }

    fn name_of(&self, book_id: &str) -> Result<String, KansoError> {
        Ok(self
            .store()
            .get(book_id)?
            .and_then(|d| {
                d.fields
                    .get("name")
                    .and_then(|n| n.as_str().map(str::to_owned))
            })
            .unwrap_or_default())
    }

    fn locked(&self, book_id: &str) -> Result<bool, KansoError> {
        Ok(self.store().locked_books()?.contains(book_id))
    }

    /// The page of notebook `book_id`, or `None` when it is gone or locked.
    fn page(&self, book_id: &str) -> Result<Option<NotebookPage>, KansoError> {
        Ok(NotebookPage::load(self.store(), book_id, now_ms())?)
    }

    fn ai_on(&self) -> bool {
        let file = self.people.load().entities;
        file.notebook_ai(self.people.ai_on(&file))
    }

    /// The words both platforms show.
    pub fn words(&self) -> HomeWords {
        HomeWords {
            ai_off: AI_OFF_LINE.to_owned(),
            locked: LOCKED_LINE.to_owned(),
            first_visit_button: FIRST_VISIT_BUTTON.to_owned(),
            no_open_tasks: NO_OPEN_TASKS.to_owned(),
            home_row: HOME_ROW.to_owned(),
            ask_button: ASK_BUTTON.to_owned(),
            open_home: OPEN_HOME.to_owned(),
            add_home_note: ADD_HOME_NOTE.to_owned(),
            home_moved: HOME_MOVED_LINE.to_owned(),
            section_keys: SECTION_KEYS.iter().map(|k| (*k).to_owned()).collect(),
        }
    }

    // MARK: Settings

    pub fn settings(&self) -> HomeSettings {
        let file = self.people.load().entities;
        HomeSettings {
            show_homepages: file.notebook_homepages,
            summaries_with_ai: file.notebook_summaries,
            refresh: file.refresh.unwrap_or_else(|| "auto".to_owned()),
        }
    }

    pub fn set_settings(&self, input: HomeSettings) -> Result<HomeSettings, KansoError> {
        guard(|| {
            let mut file = self.people.load();
            file.entities.notebook_homepages = input.show_homepages;
            file.entities.notebook_summaries = input.summaries_with_ai;
            file.entities.refresh = match input.refresh.as_str() {
                "open" | "press" => Some(input.refresh),
                _ => None,
            };
            self.people.save(&file)?;
            Ok(self.settings())
        })
    }

    // MARK: The card and opening

    /// The card above notebook `book_id`'s notes.
    pub fn card(&self, book_id: &str) -> Result<HomeCard, KansoError> {
        guard(|| {
            let file = self.people.load().entities;
            let name = self.name_of(book_id)?;
            let off = HomeCard {
                show: false,
                name: name.clone(),
                about_line: String::new(),
                has_home_note: false,
                opens_directly: false,
            };
            if !file.notebook_homepages || self.locked(book_id)? {
                return Ok(off);
            }
            let cache = self.store().page_cache(book_id)?;
            let about_line = cache
                .sections
                .get("about")
                .map(|s| {
                    s.items
                        .iter()
                        .map(|i| i.text.trim())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let books = self.store().books()?;
            let direct = books
                .iter()
                .find(|b| b.id == book_id)
                .map_or(0, |b| b.count);
            let has_children = books
                .iter()
                .any(|b| b.parent_book_id.as_deref() == Some(book_id));
            Ok(HomeCard {
                show: true,
                name,
                about_line,
                has_home_note: self.store().home_note(book_id)?.is_some(),
                opens_directly: direct == 0 && has_children,
            })
        })
    }

    /// Whether notebook `book_id` opens its homepage in place of an empty
    /// list: homepages are on and it holds only notebooks.
    pub fn opens_directly(&self, book_id: &str) -> Result<bool, KansoError> {
        Ok(self.card(book_id)?.opens_directly)
    }

    fn panel_of(
        &self,
        view: pages::notebook_model::HomeView,
        options: HomeOptions,
        policy: RefreshPolicy,
        locked_open: bool,
    ) -> Result<HomePanel, KansoError> {
        let due = if locked_open {
            Vec::new()
        } else {
            pages::due_for_refresh(
                &view.page,
                policy,
                pages::OpenOptions {
                    ai_on: options.ai_on,
                    now: options.now,
                },
            )
        };
        Ok(HomePanel {
            header: HomeHeaderInfo {
                name: view.header.name.clone(),
                notes: view.header.notes,
                notebooks_inside: view.header.notebooks_inside,
                line: view.header.line.clone(),
                has_home_note: view.header.has_home_note,
                home_note_id: String::new(),
            },
            sections: view.page.sections.iter().map(section_of).collect(),
            hidden: view
                .page
                .hidden
                .iter()
                .map(|(key, title)| PageHidden {
                    key: key.clone(),
                    title: title.clone(),
                })
                .collect(),
            ai_on: view.ai_on,
            due,
            footer: view.footer.unwrap_or_default(),
            locked: view.locked,
            locked_open,
        })
    }

    fn opened(&self, book_id: &str) -> Result<Option<Vec<OpenedNote>>, KansoError> {
        let library = self.library();
        if !library.locks().notebook_is_open(book_id)? {
            return Ok(None);
        }
        let books = self.store().book_subtree(book_id, true)?;
        let mut out = Vec::new();
        for mut note in self.store().notes_in_books(&books)? {
            if note.is_trashed() {
                continue;
            }
            if library.reveal(&mut note).is_closed() {
                continue;
            }
            out.push(OpenedNote {
                id: note.id,
                title: note.title,
                body: note.body,
                book_id: note.book_id,
                created_at: note.created_at,
                updated_at: note.updated_at,
                pinned: note.pinned,
            });
        }
        Ok(Some(out))
    }

    fn open_with(&self, book_id: &str, all: &[&str]) -> Result<Option<HomePanel>, KansoError> {
        let file = self.people.load().entities;
        let options = HomeOptions {
            ai_on: file.notebook_ai(self.people.ai_on(&file)),
            now: now_ms(),
        };
        let policy = self.people.policy(&file);
        if self.locked(book_id)? {
            let Some(notes) = self.opened(book_id)? else {
                return Ok(Some(HomePanel {
                    header: HomeHeaderInfo {
                        name: String::new(),
                        notes: 0,
                        notebooks_inside: 0,
                        line: String::new(),
                        has_home_note: false,
                        home_note_id: String::new(),
                    },
                    sections: Vec::new(),
                    hidden: Vec::new(),
                    ai_on: false,
                    due: Vec::new(),
                    footer: String::new(),
                    locked: true,
                    locked_open: false,
                }));
            };
            let name = self.name_of(book_id)?;
            let view = locked_home(&name, book_id, &notes, options.now, all.contains(&"open_tasks"));
            return Ok(Some(self.panel_of(view, options, policy, true)?));
        }
        let Some(page) = self.page(book_id)? else {
            return Ok(None);
        };
        let view = open_home(self.store(), &page, options, all)?;
        let mut panel = self.panel_of(view, options, policy, false)?;
        panel.header.home_note_id = self
            .store()
            .home_note(book_id)?
            .map(|m| m.id)
            .unwrap_or_default();
        Ok(Some(panel))
    }

    /// Notebook `book_id`'s homepage, or `None` when it is gone. A locked
    /// notebook comes back `locked` (closed) or with its in-memory view
    /// (`locked_open`).
    pub fn open(&self, book_id: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| self.open_with(book_id, &[]))
    }

    /// A computed section with all its lines ("Show all 7").
    pub fn show_all(&self, book_id: &str, key: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| self.open_with(book_id, &[key]))
    }

    /// How many open tasks are shown before "Show all".
    pub fn first_tasks(&self) -> u32 {
        FIRST_TASKS as u32
    }

    /// The notebook's map on a 320 by 200 canvas, laid out with the shared
    /// seed. Heavy enough to call off the UI thread.
    pub fn map(&self, book_id: &str) -> Result<HomeMap, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(HomeMap {
                    width: 320.,
                    height: 200.,
                    nodes: Vec::new(),
                    edges: Vec::new(),
                });
            };
            let map = page.map(self.store())?;
            Ok(HomeMap {
                width: map.width,
                height: map.height,
                nodes: map
                    .nodes
                    .into_iter()
                    .map(|n| HomeMapNode {
                        id: n.id,
                        title: n.title,
                        x: n.x,
                        y: n.y,
                        radius: n.radius,
                        degree: n.degree,
                        stubs: n.stubs,
                    })
                    .collect(),
                edges: map
                    .edges
                    .into_iter()
                    .map(|(a, b)| HomeMapEdge { a, b })
                    .collect(),
            })
        })
    }

    // MARK: AI sections

    fn write_with<T>(
        &self,
        book_id: &str,
        run: impl FnOnce(&NotebookPage, &WriteContext) -> Result<T, kanso_engine::pages::write::WriteError>,
    ) -> Result<Option<T>, KansoError> {
        let Some(page) = self.page(book_id)? else {
            return Ok(None);
        };
        if !self.ai_on() {
            return Err(KansoError::invalid(
                "Notebook summaries are off. Turn on AI in Settings first.",
            ));
        }
        let settings = self.people.ai.enabled()?;
        let provider = self.people.ai.provider_for(&settings)?;
        let provider = held(&provider);
        let context = WriteContext::new(provider.as_ref(), &settings.model, now_ms());
        run(&page, &context)
            .map(Some)
            .map_err(|err| KansoError::invalid(err.to_string()))
    }

    /// Writes one AI section again and returns the page as it is now.
    pub fn refresh_section(&self, book_id: &str, key: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            if self
                .write_with(book_id, |page, ctx| {
                    pages::refresh_section(self.store(), page, key, ctx).map(|_| ())
                })?
                .is_none()
            {
                return Ok(None);
            }
            self.open_with(book_id, &[])
        })
    }

    /// Writes every AI section that is not kept again.
    pub fn refresh_all(&self, book_id: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            if self
                .write_with(book_id, |page, ctx| {
                    pages::refresh_all(self.store(), page, ctx).map(|_| ())
                })?
                .is_none()
            {
                return Ok(None);
            }
            self.open_with(book_id, &[])
        })
    }

    /// Keeps a section's current text (`on`), or stops keeping it.
    pub fn pin(&self, book_id: &str, key: &str, on: bool) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            pages::pin(self.store(), &page, key, on)?;
            self.open_with(book_id, &[])
        })
    }

    /// Hides a section on this homepage, or shows it again.
    pub fn hide(
        &self,
        book_id: &str,
        key: &str,
        hidden: bool,
    ) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            pages::hide(self.store(), &page, key, hidden, now_ms())?;
            self.open_with(book_id, &[])
        })
    }

    /// Shows every hidden section again.
    pub fn show_hidden(&self, book_id: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            pages::show_hidden(self.store(), &page, now_ms())?;
            self.open_with(book_id, &[])
        })
    }

    /// "Not right" on line `index` of an AI section.
    pub fn not_right(
        &self,
        book_id: &str,
        key: &str,
        index: u32,
    ) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            pages::not_right(self.store(), &page, key, index as usize)?;
            self.open_with(book_id, &[])
        })
    }

    /// The section as Markdown, for Copy.
    pub fn section_markdown(&self, book_id: &str, key: &str) -> Result<String, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(String::new());
            };
            let file = self.people.load().entities;
            let view = open_home(
                self.store(),
                &page,
                HomeOptions {
                    ai_on: file.notebook_ai(self.people.ai_on(&file)),
                    now: now_ms(),
                },
                &[],
            )?;
            Ok(view
                .page
                .sections
                .iter()
                .find(|s| s.key == key)
                .map(pages::section_markdown)
                .unwrap_or_default())
        })
    }

    /// Appends a section's lines, with source links, to the Home note
    /// (made if there is none) and returns its note ID.
    pub fn copy_into_home(&self, book_id: &str, key: &str) -> Result<String, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(String::new());
            };
            let markdown = self.section_markdown(book_id, key)?;
            let title = title_of(key).unwrap_or("Section");
            Ok(
                actions::copy_into_home(
                    self.store(),
                    book_id,
                    &page.name,
                    title,
                    &markdown,
                    now_ms(),
                )
                .map_err(|err| KansoError::invalid(err.to_string()))?
                .unwrap_or_default(),
            )
        })
    }

    // MARK: The Home note

    /// The notes of the notebook that can be chosen as its Home note.
    pub fn candidates(&self, book_id: &str) -> Result<Vec<HomeCandidate>, KansoError> {
        guard(|| {
            if self.page(book_id)?.is_none() {
                return Ok(Vec::new());
            }
            let current = self.store().home_note(book_id)?.map(|m| m.id);
            let mut notes = self.store().notes_in_books(&[book_id.to_owned()])?;
            notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));
            let mut out = Vec::new();
            for note in notes {
                if note.is_trashed() || note.is_locked() || self.store().entity(&note.id)?.is_some() {
                    continue;
                }
                out.push(HomeCandidate {
                    current: current.as_deref() == Some(note.id.as_str()),
                    id: note.id,
                    title: note.title,
                });
            }
            Ok(out)
        })
    }

    /// Makes note `note_id` the notebook's Home note.
    pub fn choose_home(&self, book_id: &str, note_id: &str) -> Result<(), KansoError> {
        guard(|| {
            actions::choose_home(self.store(), book_id, note_id, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// Makes "About <Notebook>" as the Home note and returns its note ID.
    pub fn new_home(&self, book_id: &str) -> Result<String, KansoError> {
        guard(|| {
            let name = self.name_of(book_id)?;
            actions::new_home(self.store(), book_id, &name, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// Stops the notebook having a Home note (the note stays).
    pub fn clear_home(&self, book_id: &str) -> Result<bool, KansoError> {
        guard(|| {
            actions::clear_home(self.store(), book_id, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// After a note moved: when it was a Home note and its new notebook
    /// has one, its mark is removed. True when it was, for the toast.
    pub fn after_move(&self, note_id: &str) -> Result<bool, KansoError> {
        guard(|| {
            actions::after_move(self.store(), note_id, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    // MARK: Customise

    /// The sections in the order shown, each with whether it is hidden.
    pub fn sections(&self, book_id: &str) -> Result<Vec<HomeSectionSetting>, KansoError> {
        guard(|| {
            let cache = self.store().page_cache(book_id)?;
            let mut keys: Vec<&str> = SECTION_KEYS.to_vec();
            if let Some(layout) = &cache.layout {
                keys.sort_by_key(|key| {
                    layout
                        .order
                        .iter()
                        .position(|k| k == key)
                        .unwrap_or(layout.order.len() + SECTION_KEYS.iter().position(|k| k == key).unwrap_or(0))
                });
            }
            Ok(keys
                .into_iter()
                .map(|key| HomeSectionSetting {
                    key: key.to_owned(),
                    title: title_of(key).unwrap_or(key).to_owned(),
                    hidden: cache.is_hidden(key),
                })
                .collect())
        })
    }

    /// Sets the order and the hidden sections; they sync with the notebook.
    pub fn customise(
        &self,
        book_id: &str,
        order: Vec<String>,
        hidden: Vec<String>,
    ) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            actions::customise(self.store(), &page, &order, &hidden, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))?;
            self.open_with(book_id, &[])
        })
    }

    /// Puts the sections back in their order and shows them all.
    pub fn reset_layout(&self, book_id: &str) -> Result<Option<HomePanel>, KansoError> {
        guard(|| {
            let Some(page) = self.page(book_id)? else {
                return Ok(None);
            };
            actions::reset_layout(self.store(), &page, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))?;
            self.open_with(book_id, &[])
        })
    }

    /// Whether any section is unwritten and AI could write it: the
    /// first-visit button shows.
    pub fn first_visit(&self, panel: &HomePanel) -> bool {
        panel.ai_on
            && panel
                .sections
                .iter()
                .any(|s| s.ai && s.state == crate::PageSectionState::Unwritten)
    }
}
