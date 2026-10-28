//! The editing session: one open note, and everything the editor decides
//! about it.
//!
//! The shell's text field draws and takes input; Rust holds the text. An
//! [`EditorSession`] (made by [`Library::edit`]) holds the note's visible text
//! (its front matter is kept apart and shown as properties), the title, the
//! undo history, the autosave state and any change made elsewhere that waits
//! for an answer. The shell hands it every change to its field ([`apply`]),
//! asks it for the rest (a newline that continues a list, a format, an undo)
//! and for the colors of the part of the text it draws.
//!
//! Offsets here are UTF-16 units of the visible text, the unit Flutter's
//! fields count in. The hot calls (`apply`, `select`, `command`, `undo`,
//! `redo`, `set_title`) do no parsing and no I/O: they splice, update an
//! index and group a history step, and are meant to be called on the shell's
//! UI isolate. The rest may take longer and are meant to run off it.
//!
//! # Saving
//!
//! Typing is saved [`AUTOSAVE_DELAY`] after the last change, by a thread of
//! the library's (`kanso-editors`). A save follows the desktop editor's
//! `write_over`: it writes the title and text, gives the note the `#tags` the
//! text names (leaving out one the caret is in), seals a locked note and
//! keeps it open for another auto-lock time, and when only a sync moved the
//! revision, writes over it. Otherwise a change made elsewhere is a conflict:
//! nothing is saved until [`EditorSession::keep_mine`] (which writes over
//! theirs, keeping theirs in the note's history) or
//! [`EditorSession::use_theirs`] (which keeps the typed text in history).
//! Changes made elsewhere to a note with no unsaved typing just reload it.
//!
//! # Shared notes
//!
//! While a shared note's live room is open, its text reaches the library
//! only through the room's merge (`SharedNotes::editor_merge`), as on the
//! desktop: when typing pauses, and as soon as the room says other people's
//! edits are waiting. The session takes the merged text in without moving
//! the caret; typing that came while the merge ran is made again on top of
//! it and merged next time. Taking other people's text starts the history
//! over, and the shell is sent [`EditorEvent::Reloaded`].
//!
//! # Locked notes
//!
//! A session on a locked note works while the note is open. When the lock
//! closes, the session is saved sealed first, and then its text is wiped from
//! memory and [`EditorEvent::Closed`] is sent; every call after that returns
//! [`KansoError::Locked`]. Nothing a session returns, sends or completes
//! holds a closed note's text.
//!
//! [`apply`]: EditorSession::apply
//! [`AUTOSAVE_DELAY`]: kanso_engine::saving::AUTOSAVE_DELAY

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

use kanso_collab::engine::rebase;
use kanso_collab::{EditorMerge, apply_splices, remap, splices};
use kanso_core::model::{LOCK_FIELD, TRASH_BOOK_ID};
use kanso_core::properties;
use kanso_core::{DocType, Note, now_ms};
use kanso_engine::edit::history::{Grouping, History};
use kanso_engine::edit::pairing::in_fenced_code;
use kanso_engine::edit::{self, Edit, list::TabIntent, markdown};
use kanso_engine::highlight::{Highlighter, Token};
use kanso_engine::links::tags::{apply_inline_tags, choices, tag_query, tag_starts_at};
use kanso_engine::links::{self, LinkAtTarget, WikiIndex};
use kanso_engine::lock::LockView;
use kanso_engine::saving::{Autosave, has_text, same_content, text_caught_up, writes_the_same};
use kanso_engine::text::Utf16Index;
use kanso_engine::theme::CURSOR_COLOURS;
use kanso_store::{Change, Origin, Put, StoreError};
use zeroize::Zeroize;

use crate::boundary::{guard, held};
use crate::builtin_themes;
use crate::comments_model::CommentPlacement;
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{
    Completion, CompletionItem, CompletionKind, EditCommand, EditOutcome, EditorContext,
    EditorEvent, EditorRun, EditorSnapshot, FormatKind, HistoryState, LinkTarget, LockState,
    Property, Range16, Replacement, RunBatch, SaveState, ShareFile, TextChange,
};
use crate::notes::property_row;
use crate::sharing::SharedNotes;
use crate::sharing_model::CursorMark;
use crate::stream::Listeners;

/// Why a locked note takes no attachment.
const LOCKED_ATTACHMENT: &str =
    "Files can't be attached to a locked note yet: they would be stored unsealed.";

/// How long the titles for completion are kept before they are read again
/// (the desktop's `WIKI_REFRESH_DELAY`).
const TITLES_REFRESH: Duration = Duration::from_millis(400);
/// How far an idle autosave thread sleeps before it looks for a closed
/// library.
const IDLE_WAIT: Duration = Duration::from_secs(5);
/// How far back from the caret the shell's completion hint looks.
const LOOK_BACK: usize = 400;
/// Whether tables are lined up as the caret leaves them (the desktop's
/// default).
const FORMAT_TABLES: bool = true;

// MARK: The sessions of a library

/// The sessions open on a library, and the threads that serve them: one
/// that saves typing when its time comes, and one that tells each session
/// of a change made to its note.
pub(crate) struct Editors {
    sessions: Mutex<Vec<Weak<Shared>>>,
    wake: Arc<Wake>,
    started: Mutex<bool>,
    /// A library made for tests saves when told to ([`EditorSession::tick`])
    /// rather than by the clock.
    manual: bool,
    /// The revisions the library's sessions wrote lately, so that a session
    /// can tell a save by a peer (another window's editor of the same note,
    /// whose text it already has) from one made elsewhere.
    written: Mutex<VecDeque<String>>,
}

/// How many written revisions are remembered.
const WRITTEN_KEPT: usize = 32;

/// What wakes the autosave thread: a count that goes up with every change
/// to when something is due.
struct Wake {
    generation: Mutex<u64>,
    changed: Condvar,
}

impl Editors {
    pub(crate) fn new(manual: bool) -> Self {
        Editors {
            sessions: Mutex::new(Vec::new()),
            wake: Arc::new(Wake {
                generation: Mutex::new(0),
                changed: Condvar::new(),
            }),
            started: Mutex::new(false),
            manual,
            written: Mutex::new(VecDeque::new()),
        }
    }

    /// Remembers that a session wrote revision `rev`.
    fn note_written(&self, rev: &str) {
        let mut written = held(&self.written);
        written.push_back(rev.to_owned());
        while written.len() > WRITTEN_KEPT {
            written.pop_front();
        }
    }

    /// Whether a session of this library wrote revision `rev`.
    fn wrote(&self, rev: &str) -> bool {
        held(&self.written).iter().any(|seen| seen == rev)
    }

    /// The sessions still open.
    pub(crate) fn live(&self) -> Vec<Arc<Shared>> {
        let mut sessions = held(&self.sessions);
        sessions.retain(|session| session.strong_count() > 0);
        sessions.iter().filter_map(Weak::upgrade).collect()
    }

    fn register(&self, session: &Arc<Shared>) {
        let mut sessions = held(&self.sessions);
        sessions.retain(|session| session.strong_count() > 0);
        sessions.push(Arc::downgrade(session));
    }

    pub(crate) fn wake(&self) {
        *held(&self.wake.generation) += 1;
        self.wake.changed.notify_all();
    }

    /// Starts the threads, once, with the store's changes subscribed to
    /// before anything is read.
    fn start(&self, inner: &Arc<Inner>) {
        let mut started = held(&self.started);
        if *started {
            return;
        }
        *started = true;
        let receiver = inner.store.subscribe();
        let library = Arc::downgrade(inner);
        let spawned = std::thread::Builder::new()
            .name("kanso-editor-changes".into())
            .spawn(move || {
                while let Ok(change) = receiver.recv_blocking() {
                    let Some(inner) = library.upgrade() else {
                        return;
                    };
                    for session in inner.editors.live() {
                        if session.id == change.id {
                            let _ = guard(|| {
                                session.on_store_change(&change);
                                Ok(())
                            });
                        }
                    }
                }
            });
        if let Err(err) = spawned {
            eprintln!("kanso: couldn't start the editor change stream: {err}");
        }
        if self.manual {
            return;
        }
        let library = Arc::downgrade(inner);
        let wake = self.wake.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-editors".into())
            .spawn(move || autosave_loop(library, wake));
        if let Err(err) = spawned {
            eprintln!("kanso: couldn't start the autosave thread: {err}");
        }
    }

    /// Before the lock closes: saves, sealed, the sessions on locked notes,
    /// and refuses their edits from now on. Returns every session open, and
    /// whether it is on a locked note, for [`Editors::finish_lock`].
    pub(crate) fn begin_lock(&self) -> Vec<(Arc<Shared>, bool)> {
        self.live()
            .into_iter()
            .map(|session| {
                let locked = session.begin_lock();
                (session, locked)
            })
            .collect()
    }

    /// After the keys are forgotten: wipes the text of the sessions on
    /// locked notes and tells their listeners.
    pub(crate) fn finish_lock(sessions: Vec<(Arc<Shared>, bool)>) {
        for (session, locked) in sessions {
            session.finish_lock(locked);
        }
    }
}

/// Saves typing when its time comes, until the library is closed.
fn autosave_loop(library: Weak<Inner>, wake: Arc<Wake>) {
    loop {
        let seen = *held(&wake.generation);
        let Some(inner) = library.upgrade() else {
            return;
        };
        let sessions = inner.editors.live();
        // Nothing here keeps the library open while this waits.
        drop(inner);
        let now = Instant::now();
        let mut next: Option<Instant> = None;
        for session in &sessions {
            let due = guard(|| Ok(session.autosave_step(now))).ok().flatten();
            next = match (next, due) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
        drop(sessions);
        let wait = next.map_or(IDLE_WAIT, |at| {
            at.saturating_duration_since(Instant::now()).min(IDLE_WAIT)
        });
        let generation = held(&wake.generation);
        if *generation != seen {
            continue;
        }
        let _ = wake
            .changed
            .wait_timeout_while(generation, wait, |generation| *generation == seen);
    }
}

// MARK: The session

/// One note open for editing. Cheap to clone: clones are the same session.
#[derive(Clone)]
pub struct EditorSession {
    shared: Arc<Shared>,
}

/// What a session is, shared with the library's threads.
pub(crate) struct Shared {
    inner: Arc<Inner>,
    id: String,
    state: Mutex<State>,
    /// The cache of highlighted blocks, apart from the state so that
    /// highlighting never holds up typing.
    highlighter: Mutex<Highlighter>,
    listeners: Listeners<EditorEvent>,
    titles: Mutex<Option<Titles>>,
    /// Held for the length of a save, so that two never write at once. The
    /// state is not held then: typing goes on while the library is written.
    gate: Mutex<()>,
}

/// Why a session is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    /// [`EditorSession::close`].
    Closed,
    /// The note was locked.
    Locked,
    /// The note was deleted, and the person let it go.
    Gone,
}

impl End {
    fn error(self) -> KansoError {
        match self {
            End::Closed => KansoError::invalid("The editor is closed."),
            End::Locked => KansoError::Locked,
            End::Gone => KansoError::NotFound,
        }
    }
}

/// What happened elsewhere to the open note while it had unsaved typing.
enum Incoming {
    /// A newer version was saved.
    Changed(Box<Note>),
    /// The note was deleted.
    Deleted,
}

/// A write of a note, prepared with the state's lock and done without it.
struct Job {
    base: Note,
    title: String,
    body: String,
    lock: LockView,
    /// What is done to the note before it is written: its `#tags`.
    change: Box<dyn FnOnce(&mut Note) + Send>,
    origin: Origin,
}

/// How a write ended.
enum Done {
    /// The note was written, as `note`.
    Written {
        note: Box<Note>,
        sealed_now: bool,
    },
    /// The library already had this very text, in `latest`.
    Same(Box<Note>),
    /// Someone else saved `latest` first.
    Conflict(Box<Note>),
    /// The library has no such note any more.
    Gone,
    Failed(String),
}

struct State {
    /// The note as last loaded or saved, with its revision and its whole
    /// text (front matter included), open if it is locked and open.
    note: Note,
    /// Plain, or open: a session is never on a closed note.
    lock: LockView,
    /// The front matter, with the line that closes it: the start of the
    /// note's text, kept out of the visible text.
    hidden: String,
    /// The note's text after its front matter.
    text: String,
    index: Utf16Index,
    title: String,
    /// The selection in `text`, in bytes.
    selection: Range<usize>,
    /// Whether an input method is composing text.
    composing: bool,
    version: u64,
    history: History,
    autosave: Autosave,
    incoming: Option<Incoming>,
    save: SaveState,
    end: Option<End>,
    /// The lock is closing: edits are refused while the session is saved.
    closing: bool,
    /// A write is under way with the state's lock let go (typing goes on);
    /// changes made to the note meanwhile wait in `deferred`, to be taken
    /// after it.
    writing: bool,
    deferred: Vec<Change>,
    /// The title or text changed in this session and the other sessions on
    /// the note have not been told yet.
    echo: bool,
}

/// The text of a note as one session has it, handed to the other sessions
/// open on the same note (a note in two windows).
struct PeerText {
    title: String,
    hidden: String,
    text: String,
}

/// What completion knows of the library, read again when it is old.
struct Titles {
    read: Instant,
    index: Arc<WikiIndex>,
    tags: Arc<Vec<String>>,
}

impl Library {
    /// Opens note `id` for editing. [`KansoError::Locked`] while the note is
    /// locked and closed; [`KansoError::NotFound`] when there is no such
    /// note.
    pub fn edit(&self, id: &str) -> Result<EditorSession, KansoError> {
        guard(|| {
            // Subscribed before the note is read, so no change is missed.
            self.inner.editors.start(&self.inner);
            let mut note = self.stored(id)?;
            let lock = self.reveal(&mut note);
            if lock.is_closed() {
                return Err(KansoError::Locked);
            }
            let state = State::new(note, lock);
            let shared = Arc::new(Shared {
                inner: self.inner.clone(),
                id: id.to_owned(),
                state: Mutex::new(state),
                highlighter: Mutex::new(Highlighter::new()),
                listeners: Listeners::default(),
                titles: Mutex::new(None),
                gate: Mutex::new(()),
            });
            self.inner.editors.register(&shared);
            Ok(EditorSession { shared })
        })
    }
}

impl EditorSession {
    /// The note as it is now.
    pub fn snapshot(&self) -> Result<EditorSnapshot, KansoError> {
        guard(|| self.shared.run(|st, _| Ok(st.snapshot())))
    }

    /// The shell's field changed. Hot path.
    ///
    /// The change is refused when it was made against an older version of
    /// the text than the session has (the text changed meanwhile: ask for the
    /// snapshot), or names units outside the text. An offset inside a
    /// surrogate pair is taken to mean the whole character.
    pub fn apply(&self, change: TextChange) -> Result<EditOutcome, KansoError> {
        guard(|| {
            let outcome = self.shared.run(|st, _| self.shared.type_in(st, change))?;
            self.tell_caret(None);
            Ok(outcome)
        })
    }

    /// The selection changed with no change to the text. Hot path. Says what
    /// is under the caret.
    pub fn select(&self, selection: Range16, composing: bool) -> Result<EditorContext, KansoError> {
        guard(|| {
            let context = self.shared.run(|st, _| {
                st.selection = st.bytes(selection);
                if st.composing && !composing {
                    st.history.close_step();
                }
                st.composing = composing;
                st.history.selection_changed(&st.selection);
                Ok(st.context())
            })?;
            self.tell_caret(Some(selection));
            Ok(context)
        })
    }

    /// Tells the note's room where the caret is, for the others to see: the
    /// end of `selection` the person moves, or the end of the selection the
    /// session has. Nothing for a note that isn't open live.
    fn tell_caret(&self, selection: Option<Range16>) {
        let sharing = self.shared.sharing();
        if !sharing.merges(&self.shared.id) {
            return;
        }
        let Ok((anchor, head)) = self.shared.run(|st, _| {
            let hidden = st.hidden.len();
            let at = |units: u32| hidden + st.index.utf16_to_byte(&st.text, units as usize);
            Ok(match selection {
                Some(selection) => (at(selection.start), at(selection.end)),
                None => (hidden + st.selection.start, hidden + st.selection.end),
            })
        }) else {
            return;
        };
        sharing.set_caret(&self.shared.id, anchor, head);
    }

    /// Other people's carets and selections in the note, in UTF-16 units of
    /// the visible text, with their colours, as the desktop draws them. None
    /// for a note that isn't open live; a caret in the front matter is at
    /// the start.
    pub fn remote_cursors(&self) -> Result<Vec<CursorMark>, KansoError> {
        guard(|| {
            let cursors = self.shared.sharing().cursors(&self.shared.id);
            if cursors.is_empty() {
                return Ok(Vec::new());
            }
            self.shared.run(|st, _| {
                let place =
                    |byte: u32| visible_units(&st.text, &st.index, st.hidden.len(), byte as usize);
                Ok(cursors
                    .iter()
                    .map(|cursor| {
                        let (anchor, head) = (place(cursor.anchor), place(cursor.head));
                        let (light, dark) =
                            CURSOR_COLOURS[cursor.colour as usize % CURSOR_COLOURS.len()];
                        CursorMark {
                            name: cursor.name.clone(),
                            caret: head,
                            start: anchor.min(head),
                            end: anchor.max(head),
                            light: light.0,
                            dark: dark.0,
                        }
                    })
                    .collect())
            })
        })
    }

    /// A change the editor decides on. `None` when there is nothing for the
    /// command to do (the shell then types what it was going to). Hot path.
    pub fn command(&self, command: EditCommand) -> Result<Option<EditOutcome>, KansoError> {
        guard(|| {
            self.shared
                .run(|st, _| Ok(self.shared.command(st, command)))
        })
    }

    /// Stores `file`, an image or a PDF, as an attachment and puts it in the
    /// text over the selection as a paragraph of its own, as the desktop
    /// does with a pasted or dropped file: one undo step. Refused for a
    /// locked note, whose attachments would be stored unsealed (#44), and,
    /// in a sentence for the person, for a file that is neither.
    pub fn attach(&self, file: ShareFile) -> Result<EditOutcome, KansoError> {
        guard(|| {
            let plain = |st: &State| {
                if st.lock == LockView::Plain {
                    Ok(())
                } else {
                    Err(KansoError::invalid(LOCKED_ATTACHMENT))
                }
            };
            self.shared.run(|st, _| plain(st))?;
            // Written with the state let go: typing goes on meanwhile.
            let attached = kanso_engine::attach::attach_file(
                &self.shared.inner.store,
                &file.bytes,
                &file.name,
                file.content_type.as_deref(),
                None,
            )
            .map_err(KansoError::invalid)?;
            self.shared.run(|st, _| {
                plain(st)?;
                let edit = edit::block_insert(&st.text, st.selection.clone(), &attached.markdown);
                Ok(self.shared.edited(st, edit, Some(Grouping::Own)))
            })
        })
    }

    /// Undoes the last step. Hot path.
    pub fn undo(&self) -> Result<Option<EditOutcome>, KansoError> {
        guard(|| self.shared.run(|st, _| Ok(self.shared.step(st, true))))
    }

    /// Redoes the last undone step. Hot path.
    pub fn redo(&self) -> Result<Option<EditOutcome>, KansoError> {
        guard(|| self.shared.run(|st, _| Ok(self.shared.step(st, false))))
    }

    /// The title changed. Hot path.
    pub fn set_title(&self, title: String) -> Result<(), KansoError> {
        guard(|| {
            self.shared.run(|st, _| {
                if st.title != title {
                    st.title = title;
                    self.shared.touched(st);
                }
                Ok(())
            })
        })
    }

    /// The colors of `start..end` of the text in the built-in theme
    /// `theme_id`, as of the version in the batch. Text between runs is
    /// plain.
    pub fn styled_runs(
        &self,
        start: u32,
        end: u32,
        theme_id: String,
    ) -> Result<RunBatch, KansoError> {
        guard(|| {
            let theme = builtin_themes::get(&theme_id)?;
            let (text, index, version) = self
                .shared
                .run(|st, _| Ok((st.text.clone(), st.index.clone(), st.version)))?;
            let units = index.len_utf16() as u32;
            let (start, end) = (start.min(units), end.min(units));
            let from = index.utf16_to_byte(&text, start as usize);
            let to = index.utf16_to_byte(&text, end as usize).max(from);
            let spans = held(&self.shared.highlighter).spans(&text, from..to);
            let runs = spans
                .into_iter()
                .filter_map(|span| {
                    let style = kanso_engine::theme::style(&span.tokens, &theme.fitted.syntax);
                    let quiet = span.tokens.iter().any(|token| {
                        matches!(
                            token,
                            Token::Code
                                | Token::CodeBlock
                                | Token::Url
                                | Token::Html
                                | Token::FrontMatter
                        )
                    });
                    (!style.is_empty() || quiet).then(|| EditorRun {
                        range: Range16 {
                            start: index.byte_to_utf16(&text, span.range.start) as u32,
                            end: index.byte_to_utf16(&text, span.range.end) as u32,
                        },
                        color: style.color.map(|color| color.0),
                        italic: style.italic,
                        weight: style.weight,
                        quiet,
                    })
                })
                .collect();
            Ok(RunBatch { version, runs })
        })
    }

    /// What the caret completes: `[[` with note titles, `#` with tags. Titles
    /// only: nothing of a note's text. `None` when the caret is in neither.
    pub fn completion(&self) -> Result<Option<Completion>, KansoError> {
        guard(|| {
            // What is read under the lock is a line and the text before a
            // `#`; the library is read after it.
            let spot = self.shared.run(|st, _| Ok(st.completion_spot()))?;
            let Some(spot) = spot else {
                return Ok(None);
            };
            self.shared.complete(spot)
        })
    }

    /// The link at UTF-16 offset `offset` of the text: a note, a title no
    /// note has, or an address.
    pub fn link_at(&self, offset: u32) -> Result<Option<LinkTarget>, KansoError> {
        guard(|| {
            let found = self.shared.run(|st, _| {
                let at = st.index.utf16_to_byte(&st.text, offset as usize);
                let start = markdown::line_start(&st.text, at);
                let end = markdown::line_end(&st.text, at);
                Ok(links::link_at(&st.text[start..end], at - start))
            })?;
            let Some(found) = found else {
                return Ok(None);
            };
            Ok(Some(match found.target {
                LinkAtTarget::Note { id } => LinkTarget::Note { id },
                LinkAtTarget::Url { url } => LinkTarget::Web { url },
                LinkAtTarget::Wiki { title } => {
                    match self.shared.inner.store.resolve_title(&title)? {
                        Some((id, _)) => LinkTarget::Note { id },
                        None => LinkTarget::NewNote { title },
                    }
                }
            }))
        })
    }

    /// Sets property `key` to `value` (typed as the property it replaces, or
    /// as it looks, for a new one) and returns the properties. With `index`,
    /// the value of the property in that row changes and `key` is ignored;
    /// without, `key` is set, added if the note has no such property. The
    /// visible text does not change.
    pub fn set_property(
        &self,
        index: Option<u32>,
        key: String,
        value: String,
    ) -> Result<Vec<Property>, KansoError> {
        guard(|| {
            self.shared.run(|st, _| {
                let now = properties::parse(&st.hidden);
                let hidden = match index {
                    Some(row) => {
                        let (_, like) = now.get(row as usize).ok_or(KansoError::NotFound)?;
                        let value = properties::typed(&value, Some(like));
                        properties::set_property_at(&st.hidden, row as usize, &value)
                    }
                    None => {
                        if key.trim().is_empty() {
                            return Err(KansoError::invalid("A property needs a name."));
                        }
                        let like = properties::get(&now, &key);
                        let value = properties::typed(&value, like);
                        properties::set_property(&st.hidden, &key, &value)
                    }
                };
                self.shared.hidden_changed(st, hidden);
                Ok(st.properties())
            })
        })
    }

    /// Removes the property in row `index` and returns the rest.
    pub fn remove_property(&self, index: u32) -> Result<Vec<Property>, KansoError> {
        guard(|| {
            self.shared.run(|st, _| {
                if index as usize >= properties::parse(&st.hidden).len() {
                    return Err(KansoError::NotFound);
                }
                let hidden = properties::remove_property_at(&st.hidden, index as usize);
                self.shared.hidden_changed(st, hidden);
                Ok(st.properties())
            })
        })
    }

    /// Saves now (leaving the note, a pause, a switch to Preview, ⌘S). With a
    /// change made elsewhere waiting for an answer this keeps the typed
    /// text, as the desktop does, so leaving never loses it: theirs stays in
    /// the note's history.
    pub fn flush(&self) -> Result<SaveState, KansoError> {
        guard(|| {
            self.shared.run(|_, _| Ok(()))?;
            let mut events = Vec::new();
            self.shared.flush_now(&mut events);
            self.shared.send(events);
            self.shared.run(|st, _| Ok(st.save.clone()))
        })
    }

    /// Comments on the selected words of a shared note, the desktop's
    /// Comment on Selection: the typing goes into the room first, so the
    /// words are the ones everyone has. Answers with the new comment's ID;
    /// refused in the desktop's words with no words selected, for a note
    /// that isn't shared, or for a role that may only read.
    pub fn comment_on_selection(&self, body: String) -> Result<String, KansoError> {
        guard(|| {
            self.flush()?;
            let (start, end) = self.shared.run(|st, _| {
                let before = units_of(&st.hidden, st.hidden.len());
                let selection = st.range16(&st.selection);
                Ok((before + selection.start, before + selection.end))
            })?;
            self.shared
                .sharing()
                .add_comment(self.shared.id.clone(), start, end, body)
        })
    }

    /// The words the note's open comments are on, in UTF-16 units of the
    /// visible text, to tint as the desktop does. None for a note that
    /// isn't shared, and none for comments whose words were deleted.
    pub fn comment_marks(&self) -> Result<Vec<Range16>, KansoError> {
        guard(|| {
            let Ok(comments) = self.shared.sharing().comments(self.shared.id.clone()) else {
                return Ok(Vec::new());
            };
            self.shared.run(|st, _| {
                let before = units_of(&st.hidden, st.hidden.len());
                let len = st.index.len_utf16() as u32;
                Ok(comments
                    .iter()
                    .filter(|row| !row.resolved)
                    .filter_map(|row| match row.placement {
                        CommentPlacement::Anchored { start, end } => {
                            let start = start.saturating_sub(before).min(len);
                            let end = end.saturating_sub(before).min(len);
                            (start < end).then_some(Range16 { start, end })
                        }
                        CommentPlacement::Orphaned { .. } => None,
                    })
                    .collect())
            })
        })
    }

    /// Selects the words comment `comment_id` is on (where they were, for
    /// one whose words were deleted) and answers with the selection, in
    /// UTF-16 units of the visible text; `None` when there is no such
    /// comment.
    pub fn select_comment(&self, comment_id: String) -> Result<Option<Range16>, KansoError> {
        guard(|| {
            let Ok(comments) = self.shared.sharing().comments(self.shared.id.clone()) else {
                return Ok(None);
            };
            let Some(row) = comments.into_iter().find(|row| row.id == comment_id) else {
                return Ok(None);
            };
            self.shared.run(|st, _| {
                let before = units_of(&st.hidden, st.hidden.len());
                let (start, end) = match row.placement {
                    CommentPlacement::Anchored { start, end } => (start, end),
                    CommentPlacement::Orphaned { at } => (at, at),
                };
                let selection = st.bytes(Range16 {
                    start: start.saturating_sub(before),
                    end: end.saturating_sub(before),
                });
                st.selection = selection;
                Ok(Some(st.range16(&st.selection)))
            })
        })
    }

    /// Keeps the typed text over the change made elsewhere. Theirs stays in
    /// the note's history; a deleted note comes back.
    pub fn keep_mine(&self) -> Result<SaveState, KansoError> {
        guard(|| {
            self.shared.run(|_, _| Ok(()))?;
            let mut events = Vec::new();
            self.shared.keep_mine_now(&mut events);
            self.shared.send(events);
            self.shared.run(|st, _| Ok(st.save.clone()))
        })
    }

    /// Takes the change made elsewhere; the typed text goes to the note's
    /// history. For a note deleted elsewhere this lets it go: the session
    /// ends ([`EditorEvent::Closed`]) and the call returns
    /// [`KansoError::NotFound`].
    pub fn use_theirs(&self) -> Result<EditorSnapshot, KansoError> {
        guard(|| {
            self.shared.run(|st, events| {
                let Some(incoming) = st.incoming.take() else {
                    return Ok(st.snapshot());
                };
                self.shared.stash_draft(st, events);
                match incoming {
                    Incoming::Changed(theirs) => {
                        self.shared.show(st, *theirs, true);
                        Ok(st.snapshot())
                    }
                    Incoming::Deleted => {
                        st.wipe();
                        st.end = Some(End::Gone);
                        events.push(EditorEvent::Closed);
                        Err(KansoError::NotFound)
                    }
                }
            })
        })
    }

    /// Calls `sink` with what happens to the note that the shell didn't ask
    /// for, until it returns `false`.
    pub fn events(&self, sink: impl FnMut(EditorEvent) -> bool + Send + 'static) {
        self.shared.listeners.add(sink);
    }

    /// Saves and ends the session. With a question about a change made
    /// elsewhere open, the typed text is kept (see [`EditorSession::flush`]).
    pub fn close(&self) -> Result<SaveState, KansoError> {
        guard(|| {
            self.shared.run(|_, _| Ok(()))?;
            let mut events = Vec::new();
            self.shared.flush_now(&mut events);
            self.shared.send(events);
            self.shared.run(|st, _| {
                let save = st.save.clone();
                st.wipe();
                st.end = Some(End::Closed);
                Ok(save)
            })
        })
    }

    /// Saves the typing that is due at `now`, as the library's autosave
    /// thread does by the clock. A library made for tests has no such
    /// thread.
    #[cfg(any(test, feature = "test-support"))]
    pub fn tick(&self, now: Instant) {
        let _ = guard(|| Ok(self.shared.autosave_step(now)));
    }

    /// How many blocks of the text the highlighter has parsed since the
    /// session began.
    #[cfg(any(test, feature = "test-support"))]
    pub fn blocks_parsed(&self) -> usize {
        held(&self.shared.highlighter).blocks_parsed()
    }
}

// MARK: The state

impl State {
    fn new(note: Note, lock: LockView) -> State {
        let (hidden, text) = split_front(&note.body);
        State {
            index: Utf16Index::new(&text),
            title: note.title.clone(),
            save: SaveState::Saved {
                rev: note.rev.clone().unwrap_or_default(),
                at_ms: note.updated_at,
            },
            note,
            lock,
            hidden,
            text,
            selection: 0..0,
            composing: false,
            version: 0,
            history: History::default(),
            autosave: Autosave::default(),
            incoming: None,
            end: None,
            closing: false,
            writing: false,
            deferred: Vec::new(),
            echo: false,
        }
    }

    /// The note's whole text, front matter first.
    fn full_body(&self) -> String {
        let mut body = String::with_capacity(self.hidden.len() + self.text.len());
        body.push_str(&self.hidden);
        body.push_str(&self.text);
        body
    }

    /// Whether the title or the text differ from the note as last saved.
    fn unsaved(&self) -> bool {
        self.title != self.note.title
            || self
                .note
                .body
                .strip_prefix(self.hidden.as_str())
                .is_none_or(|rest| rest != self.text)
    }

    fn history_state(&self) -> HistoryState {
        HistoryState {
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }

    fn properties(&self) -> Vec<Property> {
        properties::parse(&self.hidden)
            .into_iter()
            .map(|(key, value)| property_row(key, &value))
            .collect()
    }

    fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            id: self.note.id.clone(),
            title: self.title.clone(),
            text: self.text.clone(),
            rev: self.note.rev.clone().unwrap_or_default(),
            lock: match self.lock {
                LockView::Open => LockState::Open,
                _ => LockState::Plain,
            },
            properties: self.properties(),
            version: self.version,
            selection: self.range16(&self.selection),
            history: self.history_state(),
            save: self.save.clone(),
        }
    }

    /// A byte offset as UTF-16 units.
    fn units(&self, byte: usize) -> u32 {
        self.index.byte_to_utf16(&self.text, byte) as u32
    }

    fn range16(&self, range: &Range<usize>) -> Range16 {
        Range16 {
            start: self.units(range.start),
            end: self.units(range.end),
        }
    }

    /// A selection in UTF-16 units as bytes at character boundaries, in
    /// order.
    fn bytes(&self, selection: Range16) -> Range<usize> {
        let at = |units: u32| self.index.utf16_to_byte(&self.text, units as usize);
        let (a, b) = (at(selection.start), at(selection.end));
        a.min(b)..a.max(b)
    }

    /// The caret's line up to the caret, at most [`LOOK_BACK`] bytes.
    fn line_before_caret(&self) -> (&str, usize) {
        let caret = self.selection.end.min(self.text.len());
        let start = markdown::line_start(&self.text, caret);
        let mut from = start.max(caret.saturating_sub(LOOK_BACK));
        while !self.text.is_char_boundary(from) {
            from += 1;
        }
        (&self.text[from..caret], from)
    }

    fn context(&self) -> EditorContext {
        let caret = self.selection.end.min(self.text.len());
        let intent = edit::list::tab_intent(&self.text, caret);
        let (before, _) = self.line_before_caret();
        let completion_here =
            links::completion_query(before).is_some() || tag_query(before).is_some();
        EditorContext {
            in_list: intent == TabIntent::List,
            in_table: intent == TabIntent::Table,
            in_code: in_fenced_code(&self.text, caret),
            completion_here,
            history: self.history_state(),
        }
    }

    /// What the caret may be completing, read under the lock: what is copied
    /// here is a line and the text before a `#`, and the library is read
    /// after the lock is let go.
    fn completion_spot(&self) -> Option<Spot> {
        let caret = self.selection.end.min(self.text.len());
        let start = markdown::line_start(&self.text, caret);
        let end = markdown::line_end(&self.text, caret);
        let (before, from) = self.line_before_caret();
        let tag = tag_query(before).map(|(hash, query)| {
            let at = from + hash;
            TagSpot {
                query: query.to_owned(),
                replace: self.units(at)..self.units(caret),
                before_hash: self.text[..at].to_owned(),
            }
        });
        let wiki = links::completion_query(before).map(|_| WikiSpot {
            line: self.text[start..end].to_owned(),
            caret: caret - start,
            line_start: self.units(start),
        });
        (tag.is_some() || wiki.is_some()).then_some(Spot { tag, wiki })
    }

    /// Wipes the text from memory: a locked note has closed, or the session
    /// is over.
    fn wipe(&mut self) {
        self.text.zeroize();
        self.hidden.zeroize();
        self.title.zeroize();
        self.note.body.zeroize();
        self.note.title.zeroize();
        if let Some(Incoming::Changed(theirs)) = &mut self.incoming {
            theirs.body.zeroize();
            theirs.title.zeroize();
        }
        self.incoming = None;
        self.deferred.clear();
        self.index = Utf16Index::new("");
        self.history = History::default();
        self.selection = 0..0;
        self.autosave.saved();
    }
}

/// What the caret may be completing.
struct Spot {
    tag: Option<TagSpot>,
    wiki: Option<WikiSpot>,
}

struct TagSpot {
    query: String,
    /// The `#` to the caret, in UTF-16 units.
    replace: Range<u32>,
    /// The whole text before the `#`, to tell whether a tag starts there.
    before_hash: String,
}

struct WikiSpot {
    line: String,
    /// The caret in the line, in bytes.
    caret: usize,
    /// The line's start in the text, in UTF-16 units.
    line_start: u32,
}

/// The front matter of `body` (with its closing line) and the rest.
fn split_front(body: &str) -> (String, String) {
    match properties::split(body) {
        Some(front) => (
            body[..front.body_start].to_owned(),
            body[front.body_start..].to_owned(),
        ),
        None => (String::new(), body.to_owned()),
    }
}

// MARK: Typing and commands

impl Shared {
    /// Runs `f` on the state, then tells the listeners what it said. Refused
    /// once the session is over or while its lock is closing.
    fn run<T>(
        &self,
        f: impl FnOnce(&mut State, &mut Vec<EditorEvent>) -> Result<T, KansoError>,
    ) -> Result<T, KansoError> {
        let mut events = Vec::new();
        let (result, echo) = {
            let mut st = held(&self.state);
            if let Some(end) = st.end {
                return Err(end.error());
            }
            if st.closing {
                return Err(KansoError::Locked);
            }
            let result = f(&mut st, &mut events);
            (result, st.take_echo())
        };
        self.send(events);
        // Told with this session's state free: two sessions telling each
        // other at once would otherwise wait for each other.
        if let Some(text) = echo {
            self.echo_to_peers(&text);
        }
        result
    }

    /// Runs `f` on the state whatever its condition: for the library's own
    /// threads.
    fn with_state<T>(&self, f: impl FnOnce(&mut State, &mut Vec<EditorEvent>) -> T) -> T {
        let mut events = Vec::new();
        let out = f(&mut held(&self.state), &mut events);
        self.send(events);
        out
    }

    fn send(&self, events: Vec<EditorEvent>) {
        for event in events {
            self.listeners.send(event);
        }
    }

    /// The text changed: a new version, and typing to be saved.
    fn changed(&self, st: &mut State) {
        st.version += 1;
        self.touched(st);
    }

    /// Something that is saved with the text changed.
    fn touched(&self, st: &mut State) {
        // With the question about a change made elsewhere open, nothing is
        // written until it is answered.
        if st.incoming.is_none() {
            st.autosave.changed(Instant::now());
            st.save = SaveState::Pending;
            st.echo = true;
            self.inner.editors.wake();
        }
    }

    fn hidden_changed(&self, st: &mut State, hidden: String) {
        if hidden != st.hidden {
            st.hidden = hidden;
            self.touched(st);
        }
    }

    fn outcome(&self, st: &State, replace: Option<Replacement>) -> EditOutcome {
        EditOutcome {
            version: st.version,
            replace,
            selection: st.range16(&st.selection),
            history: st.history_state(),
        }
    }

    /// A change to the field, made by the person or the input method.
    fn type_in(&self, st: &mut State, change: TextChange) -> Result<EditOutcome, KansoError> {
        if change.base_version != st.version {
            return Err(KansoError::invalid("The text changed meanwhile."));
        }
        let units = st.index.len_utf16() as u32;
        if change.start > change.end || change.end > units {
            return Err(KansoError::invalid("That change is outside the text."));
        }
        // A range that cuts a surrogate pair takes in the whole character;
        // an insertion point inside one goes before it.
        let start = st.index.utf16_to_byte(&st.text, change.start as usize);
        let mut end = st.index.utf16_to_byte(&st.text, change.end as usize);
        if change.start != change.end
            && st.index.byte_to_utf16(&st.text, end) < change.end as usize
            && let Some(ch) = st.text[end..].chars().next()
        {
            end += ch.len_utf8();
        }
        let range = start..end;
        let composing_after = change.composing.is_some();
        let before = st.selection.clone();
        let removed = st.text[range.clone()].to_owned();
        st.index.splice(&st.text, range.clone(), &change.text);
        st.text.replace_range(range.clone(), &change.text);
        st.selection = st.bytes(change.selection);
        let edit = Edit {
            range,
            text: change.text,
            selection: st.selection.clone(),
        };
        if !(edit.range.is_empty() && edit.text.is_empty()) {
            let grouping = if st.composing || composing_after {
                Grouping::Composing
            } else if typed_by_hand(&removed, &edit.text) {
                Grouping::Typing
            } else {
                Grouping::Own
            };
            st.history
                .record(&removed, &edit, &before, grouping, Instant::now());
            self.changed(st);
        }
        if st.composing && !composing_after {
            st.history.close_step();
        }
        st.composing = composing_after;
        Ok(self.outcome(st, None))
    }

    /// Applies an edit the editor decided on, or the history gave back.
    /// `record` is how it joins the history; `None` for an undo or redo,
    /// which the history has already moved.
    fn edited(&self, st: &mut State, edit: Edit, record: Option<Grouping>) -> EditOutcome {
        st.composing = false;
        let selection = edit.selection.clone();
        if st.text[edit.range.clone()] == edit.text {
            // Only the selection moves.
            st.selection = selection;
            return self.outcome(st, None);
        }
        let before = st.selection.clone();
        let removed = st.text[edit.range.clone()].to_owned();
        let replacement = Replacement {
            start: st.units(edit.range.start),
            end: st.units(edit.range.end),
            text: edit.text.clone(),
        };
        st.index.splice(&st.text, edit.range.clone(), &edit.text);
        st.text.replace_range(edit.range.clone(), &edit.text);
        st.selection = selection;
        if let Some(grouping) = record {
            st.history
                .record(&removed, &edit, &before, grouping, Instant::now());
            st.history.close_step();
        }
        self.changed(st);
        self.outcome(st, Some(replacement))
    }

    fn command(&self, st: &mut State, command: EditCommand) -> Option<EditOutcome> {
        let selection = st.selection.clone();
        let edit = match command {
            EditCommand::Enter { plain } => {
                if plain {
                    return None;
                }
                edit::enter_continuing_lists(&st.text, selection, FORMAT_TABLES)?
            }
            EditCommand::Tab { out } => edit::tab(&st.text, selection.clone(), out, FORMAT_TABLES)
                .or_else(|| edit::indent(&st.text, selection, out))?,
            EditCommand::Format { format } => {
                markdown::format(format_of(format), &st.text, selection)
            }
            EditCommand::CycleHeading => markdown::cycle_heading(&st.text, selection),
            EditCommand::WikiLink => markdown::wiki_brackets(&st.text, selection),
            EditCommand::Insert { range, text } => {
                let range = st.bytes(range);
                let caret = range.start + text.len();
                Edit {
                    range,
                    text,
                    selection: caret..caret,
                }
            }
        };
        Some(self.edited(st, edit, Some(Grouping::Own)))
    }

    /// An undo or a redo.
    fn step(&self, st: &mut State, undo: bool) -> Option<EditOutcome> {
        let edit = if undo {
            st.history.undo()?
        } else {
            st.history.redo()?
        };
        Some(self.edited(st, edit, None))
    }
}

/// Whether a change is what one key does: a character typed, or one deleted.
/// Bigger changes (a paste, a cut, a word chosen from the keyboard's
/// suggestions, an autocorrection) are steps of their own.
fn typed_by_hand(removed: &str, inserted: &str) -> bool {
    let (removed, inserted) = (removed.chars().count(), inserted.chars().count());
    matches!((removed, inserted), (_, 1) | (0 | 1, 0))
}

fn format_of(format: FormatKind) -> markdown::Format {
    match format {
        FormatKind::Bold => markdown::Format::Bold,
        FormatKind::Italic => markdown::Format::Italic,
        FormatKind::Strikethrough => markdown::Format::Strikethrough,
        FormatKind::Code => markdown::Format::Code,
        FormatKind::Bulleted => markdown::Format::Bulleted,
        FormatKind::Numbered => markdown::Format::Numbered,
        FormatKind::Task => markdown::Format::Task,
        FormatKind::Quote => markdown::Format::Quote,
        FormatKind::Link => markdown::Format::Link,
        FormatKind::Image => markdown::Format::Image,
        FormatKind::Table => markdown::Format::Table,
        FormatKind::Rule => markdown::Format::Rule,
        FormatKind::Heading1 => markdown::Format::Heading(1),
        FormatKind::Heading2 => markdown::Format::Heading(2),
        FormatKind::Heading3 => markdown::Format::Heading(3),
        FormatKind::Heading4 => markdown::Format::Heading(4),
        FormatKind::Heading5 => markdown::Format::Heading(5),
        FormatKind::Heading6 => markdown::Format::Heading(6),
        FormatKind::CodeBlock => markdown::Format::CodeBlock,
    }
}

// MARK: Completion

impl Shared {
    /// The titles and tags completion offers, read again when old.
    fn titles(&self) -> Result<(Arc<WikiIndex>, Arc<Vec<String>>), KansoError> {
        let mut cached = held(&self.titles);
        if let Some(titles) = &*cached
            && titles.read.elapsed() < TITLES_REFRESH
        {
            return Ok((titles.index.clone(), titles.tags.clone()));
        }
        let store = &self.inner.store;
        let index = Arc::new(WikiIndex::new(store.note_titles()?));
        let tags: Arc<Vec<String>> =
            Arc::new(store.tags()?.into_iter().map(|tag| tag.name).collect());
        *cached = Some(Titles {
            read: Instant::now(),
            index: index.clone(),
            tags: tags.clone(),
        });
        Ok((index, tags))
    }

    fn complete(&self, spot: Spot) -> Result<Option<Completion>, KansoError> {
        let (index, tag_names) = self.titles()?;
        // Whether a tag would start there, given everything before it (code,
        // links, headings, front matter): the store's own rule.
        if let Some(tag) = spot.tag
            && tag_starts_at(&tag.before_hash)
        {
            let names: Vec<&str> = tag_names.iter().map(String::as_str).collect();
            let items: Vec<CompletionItem> = choices(&names, &[], &tag.query)
                .into_iter()
                .map(|choice| CompletionItem {
                    label: format!("#{}", choice.name),
                    detail: None,
                    highlight: None,
                    insert: format!("#{}", choice.name),
                    new_note: false,
                })
                .collect();
            return Ok(Some(Completion {
                kind: CompletionKind::Tag,
                replace: Range16 {
                    start: tag.replace.start,
                    end: tag.replace.end,
                },
                items,
            }));
        }
        let Some(WikiSpot {
            line,
            caret,
            line_start,
        }) = spot.wiki
        else {
            return Ok(None);
        };
        let Some(found) = links::wiki_choices(&line, caret, &index) else {
            return Ok(None);
        };
        let paths = self.notebook_paths()?;
        let at = |byte: usize| line_start + units_of(&line, byte);
        let items = found
            .items
            .into_iter()
            .map(|choice| {
                let detail = (!choice.new_note)
                    .then(|| note_of(&choice.insert))
                    .flatten()
                    .and_then(|id| self.notebook_path(&id, &paths));
                CompletionItem {
                    highlight: (!choice.highlight.is_empty()).then(|| Range16 {
                        start: 0,
                        end: choice.highlight.encode_utf16().count() as u32,
                    }),
                    label: choice.label,
                    detail,
                    insert: choice.insert,
                    new_note: choice.new_note,
                }
            })
            .collect();
        Ok(Some(Completion {
            kind: CompletionKind::Wiki,
            replace: Range16 {
                start: at(found.replace.start),
                end: at(found.replace.end),
            },
            items,
        }))
    }

    /// Notebook ID to its path (`Projects / Kanso`).
    fn notebook_paths(&self) -> Result<std::collections::HashMap<String, String>, KansoError> {
        let books = self.inner.store.books()?;
        let by_id: std::collections::HashMap<&str, &kanso_core::Book> =
            books.iter().map(|book| (book.id.as_str(), book)).collect();
        Ok(books
            .iter()
            .map(|book| {
                let mut names = vec![book.name.as_str()];
                let mut parent = book.parent_book_id.as_deref();
                // A cycle a sync could make ends the walk.
                for _ in 0..books.len() {
                    let Some(up) = parent.and_then(|id| by_id.get(id)) else {
                        break;
                    };
                    names.push(up.name.as_str());
                    parent = up.parent_book_id.as_deref();
                }
                names.reverse();
                (book.id.clone(), names.join(" / "))
            })
            .collect())
    }

    /// The path of the notebook note `id` is in. Only the notebook is read
    /// from the note.
    fn notebook_path(
        &self,
        id: &str,
        paths: &std::collections::HashMap<String, String>,
    ) -> Option<String> {
        let note = self.inner.store.note(id).ok().flatten()?;
        paths.get(&note.book_id).cloned()
    }
}

/// UTF-16 units in `line[..byte]`.
/// A byte offset of a note's whole body as UTF-16 units of `text`, the
/// body after its `hidden` bytes of front matter: the start for one in the
/// front matter, the end for one past the text, and the start of the
/// character for one inside it.
fn visible_units(text: &str, index: &Utf16Index, hidden: usize, byte: usize) -> u32 {
    let mut at = byte.saturating_sub(hidden).min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    index.byte_to_utf16(text, at) as u32
}

fn units_of(line: &str, byte: usize) -> u32 {
    kanso_engine::text::byte_to_utf16(line, byte) as u32
}

/// The note ID in an inserted `[Title](<ID link>)`.
fn note_of(insert: &str) -> Option<String> {
    let (_, url) = insert.rsplit_once("](")?;
    links::linked_note(url.trim_end_matches(')'))
}

// MARK: Saving

impl Shared {
    /// Saves the typing that is due at `now`, or merges what waits in the
    /// note's live room. Says when the next save is due.
    fn autosave_step(&self, now: Instant) -> Option<Instant> {
        let sharing = self.sharing();
        let merging = sharing.merges(&self.id);
        let waiting = merging && sharing.edits_waiting(&self.id);
        {
            let mut st = held(&self.state);
            if st.end.is_some() || st.closing {
                return None;
            }
            if st.incoming.is_some() {
                st.autosave.saved();
                return None;
            }
            if !waiting && !st.autosave.due(now) {
                return st.autosave.deadline();
            }
            st.autosave.saved();
        }
        let mut events = Vec::new();
        {
            let _gate = held(&self.gate);
            if !(merging && self.merge(&mut events)) {
                let job = self.prepare_save(&mut held(&self.state), true);
                if let Some(job) = job {
                    self.write(job, &mut events);
                }
            }
        }
        self.send(events);
        held(&self.state).autosave.deadline()
    }

    fn sharing(&self) -> SharedNotes {
        SharedNotes {
            inner: self.inner.clone(),
        }
    }

    /// Merges the session's text with the note's live room: the library
    /// gets the merge, and the session takes what other people wrote. The
    /// gate is held. Says whether the note had a room to merge through (a
    /// note without one is saved as any other).
    fn merge(&self, events: &mut Vec<EditorEvent>) -> bool {
        let sent = {
            let mut st = held(&self.state);
            // A locked note is never shared; a question about a change
            // made elsewhere waits for its answer.
            if st.end.is_some() || st.closing || st.incoming.is_some() || st.lock != LockView::Plain
            {
                return false;
            }
            st.writing = true;
            (st.title.clone(), st.full_body())
        };
        let merged = self.sharing().editor_merge(&self.id, &sent.0, &sent.1);
        let mut st = held(&self.state);
        st.writing = false;
        let had_room = match merged {
            Ok(Some(merge)) => {
                self.take_merge(&mut st, events, &sent, &merge);
                true
            }
            Ok(None) => false,
            Err(err) => {
                fail(&mut st, events, format!("Couldn't save: {err}"));
                true
            }
        };
        for change in std::mem::take(&mut st.deferred) {
            self.absorb_change(&mut st, events, &change);
        }
        had_room
    }

    /// Puts a merge of `sent` in the session; the desktop editor's
    /// `collab_apply`. The caret and the selection stay with their text.
    fn take_merge(
        &self,
        st: &mut State,
        events: &mut Vec<EditorEvent>,
        sent: &(String, String),
        merge: &EditorMerge,
    ) {
        let now = (st.title.clone(), st.full_body());
        let typed_since = now != *sent;
        let (title_edits, body_edits) = if typed_since {
            (
                splices(&now.0, &rebase(&sent.0, &now.0, &merge.title_text)),
                splices(&now.1, &rebase(&sent.1, &now.1, &merge.body_text)),
            )
        } else {
            (merge.title.clone(), merge.body.clone())
        };
        // The library has the merge now, under a new revision.
        if let Ok(Some(mut latest)) = self.inner.store.note(&self.id) {
            held(&self.inner.keys).reveal(&mut latest, Some(&self.inner.store));
            st.note = latest;
        }
        if !title_edits.is_empty() || !body_edits.is_empty() {
            let (mut title, mut body) = now;
            apply_splices(&mut title, &title_edits);
            apply_splices(&mut body, &body_edits);
            let before = st.hidden.len();
            let start = remap(before + st.selection.start, &body_edits);
            let end = remap(before + st.selection.end, &body_edits);
            let (hidden, text) = split_front(&body);
            let place = |at: usize| {
                let mut at = at.saturating_sub(hidden.len()).min(text.len());
                while !text.is_char_boundary(at) {
                    at -= 1;
                }
                at
            };
            st.selection = place(start).min(place(end))..place(start).max(place(end));
            st.title = title;
            st.hidden = hidden;
            st.text = text;
            st.index = Utf16Index::new(&st.text);
            st.composing = false;
            st.history.clear();
            st.version += 1;
            events.push(EditorEvent::Reloaded {
                snapshot: st.snapshot(),
            });
        }
        if typed_since {
            st.save = SaveState::Pending;
            st.autosave.changed(Instant::now());
            self.inner.editors.wake();
        } else {
            let rev = st.note.rev.clone().unwrap_or_default();
            let at_ms = now_ms();
            st.save = SaveState::Saved {
                rev: rev.clone(),
                at_ms,
            };
            events.push(EditorEvent::Saved { rev, at_ms });
        }
    }

    /// The write that saves the title and text if they changed since the
    /// last save. With `typing`, a `#tag` the caret is still in is left out
    /// of the tags.
    fn prepare_save(&self, st: &mut State, typing: bool) -> Option<Job> {
        if st.end.is_some() || st.incoming.is_some() || !st.unsaved() {
            return None;
        }
        let caret = typing.then(|| st.hidden.len() + st.selection.end);
        let inline = st.lock == LockView::Plain;
        let store = self.inner.store.clone();
        let base = st.note.clone();
        Some(self.job(
            st,
            base,
            move |note| {
                if inline {
                    apply_inline_tags(&store, note, caret);
                }
            },
            Origin::Ui,
        ))
    }

    /// The write of `base` with the session's text and `change` applied; the
    /// desktop editor's `write_over`. `st.note` only changes once it has
    /// succeeded.
    fn job(
        &self,
        st: &mut State,
        base: Note,
        change: impl FnOnce(&mut Note) + Send + 'static,
        origin: Origin,
    ) -> Job {
        st.writing = true;
        Job {
            base,
            title: st.title.clone(),
            body: st.full_body(),
            lock: st.lock.clone(),
            change: Box::new(change),
            origin,
        }
    }

    /// Does `job` with the state free, then puts what came of it in the
    /// state. The gate is held. Says whether it wrote.
    fn write(&self, job: Job, events: &mut Vec<EditorEvent>) -> bool {
        let done = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.perform(job)))
            .unwrap_or_else(|payload| {
                Done::Failed(format!(
                    "Couldn't save: {}",
                    crate::boundary::panic_message(payload.as_ref())
                ))
            });
        let mut st = held(&self.state);
        self.finish(&mut st, events, done)
    }

    /// Writes the job's note: the desktop editor's `write_over`, without the
    /// editor. Retries once on top of a revision a sync moved.
    fn perform(&self, job: Job) -> Done {
        let store = &self.inner.store;
        let Job {
            base,
            title,
            body,
            lock,
            change,
            origin,
        } = job;
        let mut note = base.clone();
        note.title = title;
        note.body = body;
        change(&mut note);
        note.updated_at = now_ms();
        // A note going into a locked notebook goes sealed, and is open.
        let mut sealed_now = false;
        if !note.is_locked()
            && store
                .locked_books()
                .is_ok_and(|books| books.contains(&note.book_id))
        {
            let sealed = held(&self.inner.keys).seal_for_notebook(&mut note, store);
            if let Err(words) = sealed {
                return Done::Failed(format!("Couldn't save: {words}"));
            }
            sealed_now = true;
        }
        let open = lock == LockView::Open || sealed_now;
        // A sync can move the revision between reading and writing without
        // changing the note: then the write goes on top of the new revision.
        let mut retried = false;
        loop {
            // A locked note goes to the store sealed; the session keeps its
            // text.
            let mut stored = note.clone();
            if stored.is_locked() {
                let sealed = if open {
                    held(&self.inner.keys).seal(&mut stored, Instant::now())
                } else if stored.body.is_empty() {
                    Ok(())
                } else {
                    Err("Unlock the note to change its text.".to_owned())
                };
                if let Err(words) = sealed {
                    return Done::Failed(format!("Couldn't save: {words}"));
                }
            }
            let value = match serde_json::to_value(&stored) {
                Ok(value) => value,
                Err(err) => return Done::Failed(format!("Couldn't save: {err}")),
            };
            match store.put(DocType::Note, value, Put::Upsert, origin) {
                Ok(result) => {
                    note.rev = Some(result.rev);
                    return Done::Written {
                        note: Box::new(note),
                        sealed_now,
                    };
                }
                Err(StoreError::Conflict) => {
                    let Ok(Some(mut latest)) = store.note(&note.id) else {
                        return Done::Gone;
                    };
                    held(&self.inner.keys).reveal(&mut latest, Some(store));
                    // Another writer saved this very text first: its
                    // version is this one.
                    if writes_the_same(&note, &latest) {
                        return Done::Same(Box::new(latest));
                    }
                    if !retried
                        && (same_content(&base, &latest) || text_caught_up(&base, &latest, &note))
                    {
                        retried = true;
                        note.rev = latest.rev;
                        continue;
                    }
                    // Someone else saved first; theirs is shown next to
                    // ours.
                    return Done::Conflict(Box::new(latest));
                }
                Err(err) => return Done::Failed(format!("Couldn't save: {err}")),
            }
        }
    }

    /// What came of a write, in the state; then the changes made to the note
    /// while it was under way. Says whether the note was written.
    fn finish(&self, st: &mut State, events: &mut Vec<EditorEvent>, done: Done) -> bool {
        st.writing = false;
        let written = match done {
            Done::Written { note, sealed_now } => {
                if sealed_now {
                    st.lock = LockView::Open;
                }
                // The write was based on the current version, so there is
                // nothing left to choose between.
                st.incoming = None;
                let rev = note.rev.clone().unwrap_or_default();
                self.inner.editors.note_written(&rev);
                st.note = *note;
                let at_ms = now_ms();
                events.push(EditorEvent::Saved {
                    rev: rev.clone(),
                    at_ms,
                });
                self.settled(st, rev, at_ms);
                true
            }
            Done::Same(latest) => {
                st.note = *latest;
                st.incoming = None;
                let rev = st.note.rev.clone().unwrap_or_default();
                self.inner.editors.note_written(&rev);
                self.settled(st, rev, now_ms());
                true
            }
            Done::Conflict(latest) => {
                self.incoming(st, events, Incoming::Changed(latest));
                false
            }
            Done::Gone => false,
            Done::Failed(message) => fail(st, events, message),
        };
        for change in std::mem::take(&mut st.deferred) {
            self.absorb_change(st, events, &change);
        }
        written
    }

    /// The state once the library has the text as of the last write: saved,
    /// or, when typing came while it was under way, waiting its turn.
    fn settled(&self, st: &mut State, rev: String, at_ms: i64) {
        if st.unsaved() {
            st.save = SaveState::Pending;
            st.autosave.changed(Instant::now());
            self.inner.editors.wake();
        } else {
            st.save = SaveState::Saved { rev, at_ms };
        }
    }

    /// A change made elsewhere waits for an answer.
    fn incoming(&self, st: &mut State, events: &mut Vec<EditorEvent>, incoming: Incoming) {
        let deleted = matches!(incoming, Incoming::Deleted);
        st.autosave.saved();
        st.incoming = Some(incoming);
        st.save = SaveState::Conflict { deleted };
        events.push(EditorEvent::Incoming { deleted });
    }

    /// Saves any pending edits now, taking the gate.
    fn flush_now(&self, events: &mut Vec<EditorEvent>) {
        let _gate = held(&self.gate);
        self.flush_gated(events);
    }

    /// Saves any pending edits now. With the question about a change made
    /// elsewhere still open, this keeps the typed text, so that leaving the
    /// note or locking never loses it; the other version stays in history.
    /// The gate is held.
    fn flush_gated(&self, events: &mut Vec<EditorEvent>) {
        held(&self.state).autosave.saved();
        // A shared note in its room is saved by merging.
        if self.sharing().merges(&self.id) && self.merge(events) {
            Self::settle_pending(&mut held(&self.state));
            return;
        }
        // Another writer can win the race between reading and writing; try
        // again on top of its version.
        for _ in 0..3 {
            let (job, taken) = {
                let mut st = held(&self.state);
                if st.end.is_some() {
                    return;
                }
                if st.incoming.is_some() {
                    match self.prepare_keep_mine(&mut st) {
                        Some((job, incoming)) => (job, Some(incoming)),
                        None => return,
                    }
                } else {
                    match self.prepare_save(&mut st, false) {
                        Some(job) => (job, None),
                        None => {
                            Self::settle_pending(&mut st);
                            return;
                        }
                    }
                }
            };
            let written = self.write(job, events);
            let mut st = held(&self.state);
            if let Some(incoming) = taken
                && !written
                && st.incoming.is_none()
            {
                // Not a conflict: leave the question open rather than drop it.
                st.incoming = Some(incoming);
            }
            if st.incoming.is_none() {
                Self::settle_pending(&mut st);
                return;
            }
        }
        let mut st = held(&self.state);
        self.stash_draft(&mut st, events);
    }

    /// Nothing waits to be saved any more.
    fn settle_pending(st: &mut State) {
        if !st.unsaved() && st.save == SaveState::Pending {
            st.save = saved_as(&st.note);
        }
    }

    /// Keeps the typed text over the change made elsewhere. A newer version
    /// stays in history; a deleted note comes back. Takes the gate.
    fn keep_mine_now(&self, events: &mut Vec<EditorEvent>) {
        let _gate = held(&self.gate);
        let prepared = {
            let mut st = held(&self.state);
            if st.end.is_some() {
                return;
            }
            self.prepare_keep_mine(&mut st)
        };
        let Some((job, incoming)) = prepared else {
            return;
        };
        let written = self.write(job, events);
        let mut st = held(&self.state);
        if !written && st.incoming.is_none() {
            // Not a conflict: leave the question open rather than drop it.
            st.incoming = Some(incoming);
        }
    }

    /// The write that keeps the typed text over the change made elsewhere,
    /// with the question taken out of the state until it is done.
    fn prepare_keep_mine(&self, st: &mut State) -> Option<(Job, Incoming)> {
        let incoming = st.incoming.take()?;
        let base = match &incoming {
            Incoming::Changed(theirs) => (**theirs).clone(),
            Incoming::Deleted => st.note.clone(),
        };
        let store = self.inner.store.clone();
        let inline = st.lock == LockView::Plain;
        // The note's notebook may have been deleted along with it.
        let rehome = move |note: &mut Note| {
            if note.book_id != TRASH_BOOK_ID
                && store.get(&note.book_id).ok().flatten().is_none()
                && let Some(book) = store
                    .books()
                    .ok()
                    .and_then(|books| books.first().map(|book| book.id.clone()))
            {
                note.book_id = book;
            }
            if inline {
                apply_inline_tags(&store, note, None);
            }
        };
        // An API-origin write always snapshots the version it replaces.
        Some((self.job(st, base, rehome, Origin::Api), incoming))
    }

    /// Keeps the typed text in the open note's history.
    fn stash_draft(&self, st: &mut State, events: &mut Vec<EditorEvent>) {
        let note = &st.note;
        let body = st.full_body();
        let mut fields = serde_json::Map::new();
        // A locked note's draft is kept sealed, as its other versions are.
        let (body, lock) = if note.is_locked() && st.lock == LockView::Open {
            let mut sealed = Note {
                body,
                ..note.clone()
            };
            if held(&self.inner.keys)
                .seal(&mut sealed, Instant::now())
                .is_err()
            {
                return;
            }
            (String::new(), sealed.extra.get(LOCK_FIELD).cloned())
        } else if note.is_locked() {
            (String::new(), None)
        } else {
            (body, None)
        };
        fields.insert("title".into(), st.title.clone().into());
        fields.insert("body".into(), body.into());
        fields.insert("bookId".into(), note.book_id.clone().into());
        if let Some(lock) = lock {
            fields.insert(LOCK_FIELD.into(), lock);
        }
        let rev = note.rev.clone().unwrap_or_default();
        if let Err(err) = self
            .inner
            .store
            .record_revision(&note.id, &rev, &fields, Origin::Ui)
        {
            let message = format!("Couldn't keep your draft: {err}");
            st.save = SaveState::Failed {
                message: message.clone(),
            };
            events.push(EditorEvent::Failed { message });
        }
    }

    /// Reacts to a write to the open note made outside this session; the
    /// desktop editor's `on_store_change`.
    fn on_store_change(&self, change: &Change) {
        self.with_state(|st, events| {
            if st.end.is_some() || st.closing {
                return;
            }
            // Our own write is under way: what it did is told when it is done.
            if st.writing {
                st.deferred.push(change.clone());
                return;
            }
            self.absorb_change(st, events, change);
        });
    }

    fn absorb_change(&self, st: &mut State, events: &mut Vec<EditorEvent>, change: &Change) {
        if change.id != st.note.id || st.note.rev.as_deref() == Some(change.rev.as_str()) {
            return;
        }
        let latest = if change.deleted {
            None
        } else {
            let Ok(Some(latest)) = self.inner.store.note(&change.id) else {
                return;
            };
            Some(latest)
        };
        self.absorb(st, events, latest);
    }

    /// Reads the note as the library has it now, and absorbs it if it isn't
    /// the version the session has.
    fn catch_up(&self, st: &mut State, events: &mut Vec<EditorEvent>) {
        match self.inner.store.note(&self.id) {
            Ok(Some(latest)) if latest.rev != st.note.rev => self.absorb(st, events, Some(latest)),
            Ok(None) => self.absorb(st, events, None),
            _ => {}
        }
    }

    /// The library has `latest` for the open note (`None`: it was deleted),
    /// which isn't the version the session has.
    fn absorb(&self, st: &mut State, events: &mut Vec<EditorEvent>, latest: Option<Note>) {
        let incoming = match latest {
            None => Incoming::Deleted,
            Some(mut latest) => {
                let view = held(&self.inner.keys).reveal(&mut latest, Some(&self.inner.store));
                if view.is_closed() {
                    // Locked elsewhere, and this phone doesn't hold the key.
                    self.end_locked(st, events);
                    return;
                }
                if same_content(&st.note, &latest) {
                    // Only the revision moved (a sync): the session takes it
                    // and carries on, unsaved typing and all.
                    st.note.rev = latest.rev;
                    return;
                }
                // Locked (or unlocked) with its text as the session last
                // had it: only the lock changed, and what is typed goes on
                // being saved, sealed or not.
                if st.note.is_locked() != latest.is_locked()
                    && has_text(&latest, &st.note.title, &st.note.body)
                    && latest.book_id == st.note.book_id
                {
                    st.lock = view;
                    st.note = latest;
                    return;
                }
                // Another writer saved what this session shows.
                if st.lock_matches(&latest) && has_text(&latest, &st.title, &st.full_body()) {
                    let was_pending = matches!(st.save, SaveState::Pending);
                    st.note = latest;
                    st.incoming = None;
                    st.save = saved_as(&st.note);
                    if was_pending && let SaveState::Saved { rev, at_ms } = st.save.clone() {
                        // A peer's save of this text: this session is saved
                        // by it, and its listener is told.
                        events.push(EditorEvent::Saved { rev, at_ms });
                    }
                    return;
                }
                // A peer saved the text it had when it was written; this
                // session has typed on since (and the peer has been told),
                // so its base moves to the peer's version and what it has
                // goes on being saved over it.
                if st.unsaved()
                    && st.lock_matches(&latest)
                    && self
                        .inner
                        .editors
                        .wrote(latest.rev.as_deref().unwrap_or_default())
                {
                    st.note = latest;
                    return;
                }
                Incoming::Changed(Box::new(latest))
            }
        };
        if st.unsaved() {
            self.incoming(st, events, incoming);
            return;
        }
        match incoming {
            Incoming::Changed(latest) => {
                self.show(st, *latest, true);
                events.push(EditorEvent::Reloaded {
                    snapshot: st.snapshot(),
                });
            }
            Incoming::Deleted => {
                st.wipe();
                st.end = Some(End::Gone);
                events.push(EditorEvent::Closed);
            }
        }
    }

    /// Puts `note` in the session: its text is the session's now and the
    /// history starts over, as when the desktop editor shows a note. With
    /// `keep_place` the selection stays where it was, as far as the text
    /// reaches.
    fn show(&self, st: &mut State, note: Note, keep_place: bool) {
        let (hidden, text) = split_front(&note.body);
        st.lock = if note.is_locked() {
            LockView::Open
        } else {
            LockView::Plain
        };
        st.title = note.title.clone();
        st.hidden = hidden;
        st.text = text;
        st.index = Utf16Index::new(&st.text);
        st.selection = if keep_place {
            let clamp = |at: usize| {
                let mut at = at.min(st.text.len());
                while !st.text.is_char_boundary(at) {
                    at -= 1;
                }
                at
            };
            clamp(st.selection.start)..clamp(st.selection.end)
        } else {
            0..0
        };
        st.composing = false;
        st.history.clear();
        st.autosave.saved();
        st.incoming = None;
        st.save = saved_as(&note);
        st.note = note;
        st.version += 1;
    }

    // MARK: Peers

    /// Hands the text to the other sessions open on this note, in this
    /// library: a note in two windows shows what is typed in either at once.
    /// Only the session typed in saves; the others take the save when it
    /// reaches them through the store, their text being the same.
    fn echo_to_peers(&self, text: &PeerText) {
        for peer in self.inner.editors.live() {
            if peer.id != self.id || std::ptr::eq(Arc::as_ptr(&peer), self) {
                continue;
            }
            let _ = guard(|| {
                peer.take_peer_text(text);
                Ok(())
            });
        }
    }

    /// A peer's text becomes this session's, with the caret kept where it
    /// was in the text around the change, and nothing to save here.
    fn take_peer_text(&self, peer: &PeerText) {
        self.with_state(|st, events| {
            if st.end.is_some() || st.closing || st.incoming.is_some() {
                return;
            }
            if st.title == peer.title && st.hidden == peer.hidden && st.text == peer.text {
                return;
            }
            let selection = shifted(&st.text, &peer.text, &st.selection);
            st.title = peer.title.clone();
            st.hidden = peer.hidden.clone();
            st.text = peer.text.clone();
            st.index = Utf16Index::new(&st.text);
            st.selection = selection;
            st.composing = false;
            // The history is of the typing here; the peer's typing is not
            // in it, so a step back could not be told from a lost edit.
            st.history.clear();
            st.version += 1;
            if st.unsaved() {
                st.save = SaveState::Pending;
            }
            events.push(EditorEvent::Reloaded {
                snapshot: st.snapshot(),
            });
        });
    }

    // MARK: The lock

    /// The lock is about to close: if this session is on a locked note, it
    /// refuses edits and is saved, sealed. Says whether it is such a
    /// session.
    fn begin_lock(&self) -> bool {
        let mut events = Vec::new();
        let locked = {
            let _gate = held(&self.gate);
            let locked = {
                let mut st = held(&self.state);
                if st.end.is_some() {
                    false
                } else {
                    // The note may have been locked since the session last
                    // heard.
                    self.catch_up(&mut st, &mut events);
                    if st.end.is_some() || st.lock == LockView::Plain {
                        false
                    } else {
                        st.closing = true;
                        true
                    }
                }
            };
            if locked {
                self.flush_gated(&mut events);
            }
            locked
        };
        self.send(events);
        locked
    }

    /// The keys are forgotten: nothing of a locked note stays here. A
    /// session on a note locked meanwhile closes too.
    fn finish_lock(&self, was_locked: bool) {
        let _gate = held(&self.gate);
        self.with_state(|st, events| {
            if st.end.is_some() {
                return;
            }
            if was_locked {
                self.end_locked(st, events);
            } else {
                self.catch_up(st, events);
            }
        });
    }

    fn end_locked(&self, st: &mut State, events: &mut Vec<EditorEvent>) {
        st.wipe();
        st.closing = false;
        st.end = Some(End::Locked);
        // The blocks the highlighter kept are the text.
        *held(&self.highlighter) = Highlighter::new();
        *held(&self.titles) = None;
        events.push(EditorEvent::Closed);
    }
}

/// `selection` of `old`, moved to where the same place is in `new`, which
/// differs from it by one change: what lies before the change stays, what
/// lies after moves with it, and what is inside it goes to its start.
fn shifted(old: &str, new: &str, selection: &Range<usize>) -> Range<usize> {
    let mut prefix = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(prefix) || !new.is_char_boundary(prefix) {
        prefix -= 1;
    }
    let room = old.len().min(new.len()) - prefix;
    let mut suffix = old
        .bytes()
        .rev()
        .zip(new.bytes().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix) {
        suffix -= 1;
    }
    let old_end = old.len() - suffix;
    let new_end = new.len() - suffix;
    let moved = |at: usize| {
        if at <= prefix {
            at
        } else if at >= old_end {
            new_end + (at - old_end)
        } else {
            prefix
        }
    };
    moved(selection.start)..moved(selection.end)
}

impl State {
    /// The text to hand to the other sessions on the note, if it changed
    /// since they were last told.
    fn take_echo(&mut self) -> Option<PeerText> {
        if !std::mem::take(&mut self.echo) || self.end.is_some() || self.closing {
            return None;
        }
        Some(PeerText {
            title: self.title.clone(),
            hidden: self.hidden.clone(),
            text: self.text.clone(),
        })
    }

    /// Whether `latest` is locked as the open note is.
    fn lock_matches(&self, latest: &Note) -> bool {
        self.note.is_locked() == latest.is_locked()
    }
}

impl Drop for Shared {
    /// Whatever is unsaved is saved; with a question about a change made
    /// elsewhere open, the typed text is kept and theirs stays in history.
    fn drop(&mut self) {
        let _ = guard(|| {
            let mut events = Vec::new();
            if held(&self.state).end.is_none() {
                self.flush_now(&mut events);
            }
            self.send(events);
            Ok(())
        });
    }
}

/// A save that failed: the session says why, and the typing waits for a
/// retry. Always `false`.
fn fail(st: &mut State, events: &mut Vec<EditorEvent>, message: String) -> bool {
    st.autosave.saved();
    st.save = SaveState::Failed {
        message: message.clone(),
    };
    events.push(EditorEvent::Failed { message });
    false
}

fn saved_as(note: &Note) -> SaveState {
    SaveState::Saved {
        rev: note.rev.clone().unwrap_or_default(),
        at_ms: now_ms(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_store::Change;
    use kanso_sync::{MemorySecrets, MemoryServer};

    use super::*;
    use crate::model::NewNote;

    fn session(body: &str) -> (Library, EditorSession, String) {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        // The folder lives as long as the process: the library holds files.
        std::mem::forget(dir);
        let id = library
            .create(NewNote {
                title: "T".into(),
                body: body.into(),
                ..NewNote::default()
            })
            .unwrap()
            .id;
        let session = library.edit(&id).unwrap();
        (library, session, id)
    }

    fn type_at_end(session: &EditorSession, text: &str) {
        let snapshot = session.snapshot().unwrap();
        let end = snapshot.text.encode_utf16().count() as u32;
        session
            .apply(TextChange {
                base_version: snapshot.version,
                start: end,
                end,
                text: text.into(),
                selection: Range16 {
                    start: end + 1,
                    end: end + 1,
                },
                composing: None,
            })
            .unwrap();
    }

    // no-prop-test: the race of a write under way with the state let go,
    // set up step by step; the whole session is property-tested in
    // tests/editor.rs.
    #[test]
    fn typing_during_a_write_waits_and_the_writes_own_change_is_no_conflict() {
        let (library, session, id) = session("base");
        let shared = &session.shared;
        type_at_end(&session, "!");
        // The write is prepared, then typing goes on while it is under way.
        let _gate = held(&shared.gate);
        let job = shared.prepare_save(&mut held(&shared.state), true).unwrap();
        type_at_end(&session, "?");
        let done = shared.perform(job);
        let Done::Written { note, .. } = &done else {
            panic!("not written");
        };
        let rev = note.rev.clone().unwrap();
        // The library's change for that very write is heard before the
        // write is told of: it waits, and is no change made elsewhere.
        shared.on_store_change(&Change {
            id: id.clone(),
            rev: rev.clone(),
            seq: 0,
            deleted: false,
            origin: Origin::Ui,
        });
        // (The library's own thread may have heard it too.)
        assert!(!held(&shared.state).deferred.is_empty());
        let mut events = Vec::new();
        assert!(shared.finish(&mut held(&shared.state), &mut events, done));
        let st = held(&shared.state);
        assert!(st.deferred.is_empty() && !st.writing);
        assert!(st.incoming.is_none());
        assert_eq!(st.note.rev.as_deref(), Some(rev.as_str()));
        // What was typed meanwhile is still to be saved.
        assert!(st.unsaved() && st.autosave.pending());
        assert_eq!(st.save, SaveState::Pending);
        assert!(matches!(events.as_slice(), [EditorEvent::Saved { .. }]));
        assert_eq!(library.note(&id).unwrap().body.as_deref(), Some("base!"));
    }

    // no-prop-test: the same race, with another writer's change in it.
    #[test]
    fn another_writers_change_during_a_write_is_asked_about_after_it() {
        let (library, session, id) = session("base");
        let shared = &session.shared;
        type_at_end(&session, "!");
        let _gate = held(&shared.gate);
        let job = shared.prepare_save(&mut held(&shared.state), true).unwrap();
        let done = shared.perform(job);
        // Someone else saves on top of it, and the session hears of it while
        // it is still finishing.
        let rev = library.note(&id).unwrap().rev;
        let theirs = library.save(&id, &rev, "theirs".into()).unwrap();
        shared.on_store_change(&Change {
            id: id.clone(),
            rev: theirs.rev,
            seq: 0,
            deleted: false,
            origin: Origin::Ui,
        });
        type_at_end(&session, "?");
        let mut events = Vec::new();
        shared.finish(&mut held(&shared.state), &mut events, done);
        assert_eq!(
            session.snapshot().unwrap().save,
            SaveState::Conflict { deleted: false }
        );
        assert!(events.contains(&EditorEvent::Incoming { deleted: false }));
    }

    #[test]
    fn a_caret_is_placed_in_the_text_after_the_front_matter() {
        let text = "Alpha 😀 beta";
        let index = Utf16Index::new(text);
        // 4 bytes of front matter: the text starts at byte 4.
        assert_eq!(visible_units(text, &index, 4, 0), 0);
        assert_eq!(visible_units(text, &index, 4, 4 + 6), 6);
        // Inside the emoji: its start. After it: two units on.
        assert_eq!(visible_units(text, &index, 4, 4 + 8), 6);
        assert_eq!(visible_units(text, &index, 4, 4 + 10), 8);
        assert_eq!(visible_units(text, &index, 4, 1000), 13);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        /// Wherever someone's caret is, it lands on a character of the
        /// visible text, at the units the text before it takes.
        #[test]
        fn prop_a_caret_lands_on_a_character_of_the_text(
            text in "[a-zé😀\u{301}\n]{0,24}",
            hidden in 0usize..12,
            byte in 0usize..120,
        ) {
            let index = Utf16Index::new(&text);
            let units = visible_units(&text, &index, hidden, byte) as usize;
            let total = text.encode_utf16().count();
            proptest::prop_assert!(units <= total);
            // The oracle: count the units of the text up to where it lands.
            let mut at = byte.saturating_sub(hidden).min(text.len());
            while !text.is_char_boundary(at) {
                at -= 1;
            }
            proptest::prop_assert_eq!(units, text[..at].encode_utf16().count());
            // Further on in the body is never earlier in the text.
            let next = visible_units(&text, &index, hidden, byte + 1) as usize;
            proptest::prop_assert!(next >= units);
        }
    }
}
