//! A Vim editor's side of [`Vim`]: what it keeps between keys and decides
//! around them, with nothing drawn.
//!
//! The state machine takes the text, the cursor and a key. An editor also
//! has to tell a Visual mode's own selection from a click, keep the text as
//! a command began Insert mode (for `.`, and to make the whole Insert
//! session one undo step), pass the lines in view for `H`, `M` and `L`, and
//! refuse changes in a note that is read-only. [`Session`] does those; the
//! editor applies each [`Step`] to its text, does the [`Effect`](super::Effect)s, and calls
//! [`Session::leave_insert`] when Escape ends Insert mode.
//!
//! Insert mode is the editor's own typing: [`Session::route`] says which
//! keys it takes (all but Escape), and it reports each edit it makes in
//! Insert mode with [`Session::count_edit`], so [`Left::edits`] bounds how
//! far back its undo history has to look.

use super::{Clipboard, Key, Mode, Outcome, Status, Vim, diff};
use crate::edit::{Edit, Selection};

/// Where a key goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Insert mode: the editor's own typing, input methods and writing aids.
    Typing,
    /// Escape in Insert mode: the editor calls [`Session::leave_insert`].
    Leave,
    /// Normal and Visual modes: [`Session::key`].
    Vim,
}

/// The editor as a key finds it.
#[derive(Clone, Debug)]
pub struct Input<'a> {
    /// The note's text, as it is now.
    pub text: &'a str,
    /// Where the editor's selection is.
    pub selection: Selection,
    /// The editor's caret, which is the selection's end.
    pub caret: usize,
    /// The first and last lines (0-based) wholly in view, for `H`, `M` and
    /// `L`; `None` when the editor can't say.
    pub view: Option<(usize, usize)>,
    /// A note shared to read: Vim moves through it, but changes nothing.
    pub read_only: bool,
}

/// What a key came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyed {
    /// Not Vim's: the editor goes on with it as usual (Escape in Normal
    /// mode with nothing pending, a key Vim has no use for, or a key
    /// [`Session::route`] sends elsewhere).
    Ignored,
    /// Vim's, but a change or Insert mode in a read-only note: Vim is back
    /// in Normal mode with nothing pending, and the text stays as it is.
    Refused,
    /// Vim's, with what to apply.
    Handled(Step),
}

/// A key Vim took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// What the key did.
    pub outcome: Outcome,
    /// Where Vim's cursor was in the text handed in: the selection's start,
    /// or the caret for none.
    pub cursor: usize,
    /// Whether this key began Insert mode.
    pub entering: bool,
}

impl Step {
    /// The one change to make, with the selection to leave after it.
    pub fn change(&self) -> Option<Edit> {
        let (range, text) = self.outcome.edit.clone()?;
        Some(Edit {
            range,
            text,
            selection: self.selection(),
        })
    }

    /// What to leave selected: the shown selection in a Visual mode, else
    /// the cursor.
    pub fn selection(&self) -> Selection {
        let cursor = self.outcome.cursor;
        self.outcome.selection.clone().unwrap_or(cursor..cursor)
    }

    /// The text after the change, from the text it was made on.
    pub fn text_after(&self, text: &str) -> String {
        let mut done = text.to_owned();
        if let Some((range, with)) = &self.outcome.edit {
            done.replace_range(range.clone(), with);
        }
        done
    }
}

/// The text before the command that began Insert mode, and where in it the
/// session's mark (the cursor then) goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base {
    /// The text before the command that began Insert mode.
    pub text: String,
    /// Where the session's mark (the cursor then) goes in it.
    pub mark: usize,
}

/// Escape ended Insert mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Left {
    /// The change to the text as it is, which repeats what was typed for a
    /// count.
    pub edit: Option<(Selection, String)>,
    /// Where the cursor goes.
    pub cursor: usize,
    /// The text after that change.
    pub done: String,
    /// The text before the command that began the session; `None` when it
    /// was lost (the text was set anew, or merged with someone else's).
    pub base: Option<Base>,
    /// Edits made during the session: as many undo steps as there can be.
    pub edits: usize,
}

impl Left {
    /// The base with `mark` put in at its place: the text the editor's
    /// history holds just after the session began, if it put the mark in.
    pub fn marked(&self, mark: &str) -> Option<String> {
        let base = self.base.as_ref()?;
        let before = base.text.get(..base.mark)?;
        Some(format!("{before}{mark}{}", &base.text[base.mark..]))
    }

    /// The whole session as one change to the base, for an editor that has
    /// undone back to it.
    pub fn whole(&self) -> Option<Edit> {
        let base = self.base.as_ref()?;
        Some(match diff(&base.text, &self.done) {
            Some((range, text)) => Edit {
                range,
                text,
                selection: self.cursor..self.cursor,
            },
            None => self.nothing(),
        })
    }

    /// The change to the text as it is, for an editor that can't make the
    /// session one step.
    pub fn last(&self) -> Edit {
        match self.edit.clone() {
            Some((range, text)) => Edit {
                range,
                text,
                selection: self.cursor..self.cursor,
            },
            None => self.nothing(),
        }
    }

    /// No change, with the cursor after it.
    fn nothing(&self) -> Edit {
        Edit {
            range: self.cursor..self.cursor,
            text: String::new(),
            selection: self.cursor..self.cursor,
        }
    }
}

/// Vim in one editor.
#[derive(Default)]
pub struct Session {
    vim: Vim,
    /// The text before the command that began Insert mode, and where the
    /// mark went in it.
    base: Option<Base>,
    /// The text as Insert mode began, and where.
    entry: Option<(String, usize)>,
    /// Edits since Insert mode began.
    edits: usize,
    /// The selection a Visual mode last showed, to tell a click from it.
    shown: Option<Selection>,
    /// Where Visual mode's cursor is.
    head: usize,
}

impl Session {
    /// In Normal mode.
    pub fn new() -> Session {
        Session::default()
    }

    /// The mode the editor is in.
    pub fn mode(&self) -> Mode {
        self.vim.mode()
    }

    /// What the footer shows.
    pub fn status(&self) -> Status {
        self.vim.status()
    }

    /// Whether the keys are Vim's: not in Insert mode.
    pub fn holds_keys(&self) -> bool {
        self.vim.mode() != Mode::Insert
    }

    /// Where Visual mode's cursor is (the editor's is at the selection's
    /// end).
    pub fn head(&self) -> usize {
        self.head
    }

    /// The selection a Visual mode last showed.
    pub fn shown(&self) -> Option<&Selection> {
        self.shown.as_ref()
    }

    /// Back to Normal mode, as when another note opens.
    pub fn reset(&mut self) {
        self.vim.reset();
        self.base = None;
        self.entry = None;
        self.shown = None;
        self.edits = 0;
    }

    /// The text was set anew (and its undo history with it): an Insert
    /// session going on can't be made one step any more.
    pub fn forget_session(&mut self) {
        self.base = None;
        self.entry = None;
    }

    /// Someone else's edits were merged into the text: Vim's cursor and
    /// Visual selection are now `head` and `shown`, and an Insert session
    /// can't be made one undo step any more (its text changed under it).
    pub fn follow(&mut self, head: usize, shown: Option<Selection>) {
        self.head = head;
        self.shown = shown;
        self.forget_session();
    }

    /// One more edit to the text while Insert mode lasts.
    pub fn count_edit(&mut self) {
        if self.vim.mode() == Mode::Insert {
            self.edits += 1;
        }
    }

    /// Where `key` goes: Insert mode types, and only Escape ends it.
    pub fn route(&self, key: Key) -> Route {
        match (self.vim.mode(), key) {
            (Mode::Insert, Key::Esc) => Route::Leave,
            (Mode::Insert, _) => Route::Typing,
            _ => Route::Vim,
        }
    }

    /// Takes `key` in Normal or Visual mode. (A key [`Session::route`]
    /// sends elsewhere is ignored.)
    pub fn key(&mut self, key: Key, input: &Input<'_>, clipboard: &mut dyn Clipboard) -> Keyed {
        if self.route(key) != Route::Vim {
            return Keyed::Ignored;
        }
        // A selection Vim didn't make (a click or drag, select all, undo)
        // ends a Visual mode; in Normal mode the cursor is where it starts.
        if self.vim.mode().is_visual() && self.shown.as_ref() != Some(&input.selection) {
            self.vim.reset();
        }
        let cursor = if input.selection.is_empty() {
            input.caret
        } else {
            input.selection.start
        };
        let was = self.vim.mode();
        self.vim.set_view(input.view);
        let outcome = self.vim.handle_key(key, input.text, cursor, clipboard);
        if !outcome.consumed {
            return Keyed::Ignored;
        }
        if input.read_only && (outcome.edit.is_some() || outcome.mode == Mode::Insert) {
            self.vim.reset();
            return Keyed::Refused;
        }
        let step = Step {
            entering: outcome.mode == Mode::Insert && was != Mode::Insert,
            outcome,
            cursor,
        };
        self.shown = step.outcome.selection.clone();
        self.head = step.outcome.cursor;
        if step.entering {
            self.base = Some(Base {
                text: input.text.to_owned(),
                mark: cursor,
            });
            self.entry = Some((step.text_after(input.text), step.outcome.cursor));
            self.edits = 0;
        }
        Keyed::Handled(step)
    }

    /// Escape in Insert mode, with the text and caret as they are: a count
    /// types the text again, and the cursor steps back.
    pub fn leave_insert(&mut self, now: &str, caret: usize) -> Left {
        let (entry, entry_cursor) = self.entry.take().unwrap_or_else(|| (now.to_owned(), caret));
        let out = self.vim.leave_insert(&entry, entry_cursor, now, caret);
        let mut done = now.to_owned();
        if let Some((range, with)) = &out.edit {
            done.replace_range(range.clone(), with);
        }
        self.shown = None;
        Left {
            edit: out.edit,
            cursor: out.cursor,
            done,
            base: self.base.take(),
            edits: std::mem::take(&mut self.edits),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::super::Effect;
    use super::super::tests::{Buf, FakeClip, config, gen_case, gen_key};
    use super::*;

    /// What marks the start of an Insert mode session in an editor's undo
    /// history, as the desktop editor puts it.
    const MARK: &str = "\u{200B}";

    /// An editor around a [`Session`]: types in Insert mode, applies each
    /// step, and checks what the session hands it as it goes.
    struct Editor {
        text: String,
        cursor: usize,
        session: Session,
        clip: FakeClip,
        read_only: bool,
    }

    impl Editor {
        fn new(text: &str, cursor: usize, read_only: bool) -> Editor {
            Editor {
                text: text.to_owned(),
                cursor: super::super::clamp_normal(text, cursor),
                session: Session::new(),
                clip: FakeClip::default(),
                read_only,
            }
        }

        fn key(&mut self, key: Key) -> Result<(), TestCaseError> {
            match self.session.route(key) {
                Route::Typing => self.type_key(key),
                Route::Leave => return self.leave(),
                Route::Vim => {}
            }
            let selection = self
                .session
                .shown()
                .filter(|_| self.session.mode().is_visual())
                .cloned()
                .unwrap_or(self.cursor..self.cursor);
            let input = Input {
                text: &self.text.clone(),
                caret: selection.end,
                selection,
                view: None,
                read_only: self.read_only,
            };
            let before = self.text.clone();
            match self.session.key(key, &input, &mut self.clip) {
                Keyed::Ignored => {}
                Keyed::Refused => {
                    prop_assert!(self.read_only);
                    prop_assert_eq!(self.session.mode(), Mode::Normal);
                }
                Keyed::Handled(step) => {
                    let after = step.text_after(&before);
                    if let Some(change) = step.change() {
                        prop_assert_eq!(change.apply(&before), after.clone());
                        prop_assert_eq!(&change.selection, &step.selection());
                    }
                    self.text = after;
                    self.cursor = step.outcome.cursor;
                    for effect in &step.outcome.effects {
                        if let Effect::Copy(text) = effect {
                            self.clip.0 = Some(text.clone());
                        }
                    }
                    if step.entering {
                        prop_assert_eq!(self.session.mode(), Mode::Insert);
                        prop_assert_eq!(
                            self.session.entry.clone(),
                            Some((self.text.clone(), self.cursor))
                        );
                        prop_assert_eq!(
                            self.session.base.clone(),
                            Some(Base {
                                text: before,
                                mark: step.cursor
                            })
                        );
                        prop_assert_eq!(self.session.edits, 0);
                    }
                }
            }
            Ok(())
        }

        /// The editor's own typing.
        fn type_key(&mut self, key: Key) {
            let typed = match key {
                Key::Char(c) => Some(c),
                Key::Enter => Some('\n'),
                _ => None,
            };
            if let Some(c) = typed {
                self.text.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.session.count_edit();
            } else if key == Key::Backspace
                && let Some(c) = self.text[..self.cursor].chars().next_back()
            {
                self.cursor -= c.len_utf8();
                self.text.remove(self.cursor);
                self.session.count_edit();
            }
        }

        fn leave(&mut self) -> Result<(), TestCaseError> {
            let now = self.text.clone();
            let left = self.session.leave_insert(&now, self.cursor);
            let mut done = now.clone();
            if let Some((range, with)) = &left.edit {
                done.replace_range(range.clone(), with);
            }
            prop_assert_eq!(&left.done, &done);
            prop_assert_eq!(left.last().apply(&now), done.clone());
            // The session as one change to the base, however the editor
            // got back to it.
            if let Some(base) = &left.base {
                let whole = left.whole().expect("a base");
                prop_assert_eq!(whole.apply(&base.text), done.clone());
                let marked = left.marked(MARK).expect("a mark on a boundary");
                prop_assert_eq!(marked.replacen(MARK, "", 1), base.text.clone());
                prop_assert_eq!(&marked[..base.mark], &base.text[..base.mark]);
            }
            prop_assert!(self.session.base.is_none() && self.session.entry.is_none());
            prop_assert_eq!(self.session.edits, 0);
            prop_assert_eq!(self.session.mode(), Mode::Normal);
            self.text = done;
            self.cursor = left.cursor;
            Ok(())
        }
    }

    /// Keys without undo and redo, which the editor does with its history.
    fn gen_script() -> impl Strategy<Value = Vec<Key>> {
        prop::collection::vec(
            gen_key().prop_map(|key| match key {
                Key::Char('u') | Key::Redo => Key::Esc,
                key => key,
            }),
            0..60,
        )
    }

    proptest! {
        #![proptest_config(config())]

        /// A key script through a session does what the state machine alone
        /// does with an editor that keeps its own bookkeeping: the same text
        /// and mode after every key, and the text is only ever changed by
        /// the steps' edits and the editor's own typing.
        #[test]
        fn a_session_is_the_state_machine_with_its_bookkeeping(
            (text, cursor) in gen_case(),
            script in gen_script(),
        ) {
            let mut editor = Editor::new(&text, cursor, false);
            let mut plain = Buf::new(&text, cursor);
            for key in script {
                let visual = editor.session.mode().is_visual();
                editor.key(key)?;
                plain.key(key);
                prop_assert_eq!(&editor.text, &plain.text);
                prop_assert_eq!(editor.session.mode(), plain.vim.mode());
                prop_assert_eq!(editor.session.status(), plain.vim.status());
                // Out of Visual mode the editor's selection starts where
                // the state machine's cursor was, not at its head: the
                // plain run follows the editor's cursor from there.
                if visual {
                    plain.cursor = editor.cursor;
                } else if !editor.session.mode().is_visual() {
                    prop_assert_eq!(editor.cursor, plain.cursor);
                }
            }
        }

        /// In a note shared to read, Vim moves but changes nothing and never
        /// enters Insert mode.
        #[test]
        fn a_read_only_note_is_never_changed(
            (text, cursor) in gen_case(),
            script in gen_script(),
        ) {
            let mut editor = Editor::new(&text, cursor, true);
            for key in script {
                editor.key(key)?;
                prop_assert_eq!(&editor.text, &text);
                prop_assert_ne!(editor.session.mode(), Mode::Insert);
            }
        }

        /// A selection that isn't the one Visual mode showed ends Visual
        /// mode, and merging someone's edits forgets the Insert session.
        #[test]
        fn a_foreign_selection_ends_visual_mode((text, cursor) in gen_case()) {
            let mut editor = Editor::new(&text, cursor, false);
            editor.key(Key::Char('v'))?;
            prop_assert!(editor.session.mode().is_visual());
            // A selection of the same text is not a foreign one.
            prop_assume!(editor.session.shown() != Some(&(0..0)));
            let input = Input {
                text: &text,
                selection: 0..0,
                caret: 0,
                view: None,
                read_only: false,
            };
            let keyed = editor.session.key(Key::Char('l'), &input, &mut editor.clip);
            // Vim started over in Normal mode, and took the key there.
            prop_assert_eq!(editor.session.mode(), Mode::Normal);
            prop_assert!(matches!(keyed, Keyed::Handled(_)));
            editor.session.follow(0, None);
            prop_assert!(editor.session.base.is_none() && editor.session.entry.is_none());
        }
    }
}
