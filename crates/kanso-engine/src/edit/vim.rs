//! Vim keys for the note's source: a state machine with no UI in it. The
//! editor hands it each key with the text and the cursor; it answers with
//! at most one replacement to make (so every Vim change undoes in one
//! step), where the cursor goes, the mode, and anything the editor does
//! for it (undo, redo, copy, save).
//!
//! Insert mode is the editor's own typing: the editor only tells the state
//! machine when Escape ends it ([`Vim::leave_insert`]), with the text from
//! before and after, so input methods, pairing, list continuation and
//! completions work as they do without Vim, and `.` can type the same
//! text again.
//!
//! Covered: Normal, Insert, Visual and Visual Line modes; the motions
//! `h j k l w b e ge W B E gE 0 ^ $ gg G H M L f F t T ; , { } % + -`;
//! counts; the operators `d c y > <` over motions and text objects, and
//! `dd cc yy >> <<`; `x X D C s S r J ~ p P o O a A i I Y`; `.` (a Visual
//! mode change again over a selection the same size from the cursor);
//! `u` and Ctrl-R; one register, kept in step with the system clipboard;
//! and `:w`, `:q`, `:wq`, `:x` and `:N`.
//!
//! [`Session`] is an editor's side of it: the bookkeeping around the state
//! machine that doesn't draw anything.
//!
//! Offsets are UTF-8 byte offsets. A "character" to the cursor is a
//! grapheme cluster, so an emoji or an accented letter is one step.

mod motion;
mod object;
mod session;
#[cfg(test)]
mod tests;
mod text;

use std::ops::Range;

use motion::{Context as MotionContext, FindKind, Kind, Motion, Want};
use text::*;

pub use session::{Base, Input, Keyed, Left, Route, Session, Step};
pub use text::{clamp_normal, diff};

/// How long the character (grapheme) `text` starts with is; 0 when it
/// starts with a line break or is empty.
pub fn grapheme_at(text: &str) -> usize {
    next_in_line(text, 0).unwrap_or(0)
}

/// The most any count repeats a command, so a mistyped `99999p` can't
/// stall the editor.
const MAX_COUNT: usize = 10_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// `NORMAL`: keys are commands.
    #[default]
    Normal,
    /// `INSERT`: keys are text.
    Insert,
    /// `VISUAL`: an operator takes the selection.
    Visual,
    /// `V-LINE`: operators take whole lines of the selection.
    VisualLine,
}

impl Mode {
    /// The mode as the footer names it.
    pub fn label(self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
        }
    }

    /// Whether an operator here takes the selection (`VISUAL` or
    /// `V-LINE`).
    pub fn is_visual(self) -> bool {
        matches!(self, Mode::Visual | Mode::VisualLine)
    }
}

/// A key as Vim takes it. Keys with ⌘ (or the app's Ctrl shortcuts) never
/// get here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A character key.
    Char(char),
    /// Escape: leaves Insert mode, or drops what was pending.
    Esc,
    /// As `+` in Normal mode: the next line's start.
    Enter,
    /// As `h` in Normal mode.
    Backspace,
    /// Vim has no use for Tab; the editor keeps its own indenting.
    Tab,
    /// The arrow keys, as `h j k l`.
    Left,
    Right,
    Up,
    Down,
    /// As `0` and `$`.
    Home,
    End,
    /// Ctrl-R.
    Redo,
}

/// Something the editor does for a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// `u`: undo this many steps.
    Undo(usize),
    /// Ctrl-R: redo this many steps.
    Redo(usize),
    /// A yank: put this on the system clipboard.
    Copy(String),
    /// `:w`.
    Save,
    /// `:q`: done with the note.
    Close,
}

/// Where the system clipboard's text comes from, read only when `p` needs
/// it (or a delete notes what it holds).
pub trait Clipboard {
    fn read(&mut self) -> Option<String>;
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The one replacement to make, on the text as it was handed in.
    pub edit: Option<(Range<usize>, String)>,
    /// Where the cursor goes, in the text after the edit.
    pub cursor: usize,
    /// In Visual modes, the text to show selected.
    pub selection: Option<Range<usize>>,
    /// The mode to show afterwards.
    pub mode: Mode,
    /// What the editor does for the command: undo, redo, copy, save, close.
    pub effects: Vec<Effect>,
    /// Whether the key was Vim's. Escape in Normal mode with nothing
    /// pending isn't, so it does what it does without Vim.
    pub consumed: bool,
}

/// What the footer shows: the mode, and a command being typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    /// The mode the footer names.
    pub mode: Mode,
    /// The command being typed, as `:` and its text, or the keys so far.
    pub pending: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Register {
    text: String,
    linewise: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Delete,
    Change,
    Yank,
    /// `>` (false) and `<` (true): whole lines in or out by one indent.
    Shift(bool),
}

impl Op {
    fn from_char(c: char) -> Option<Op> {
        match c {
            'd' => Some(Op::Delete),
            'c' => Some(Op::Change),
            'y' => Some(Op::Yank),
            '>' => Some(Op::Shift(false)),
            '<' => Some(Op::Shift(true)),
            _ => None,
        }
    }
}

/// One level of indent for `>` and `<`: the editor's own (two spaces, as
/// Tab indents and list items nest).
const INDENT: &str = "  ";

/// The size of a Visual selection a change was made over, which `.` takes
/// again from the cursor, as Vim does: so many lines, and on the last one
/// so many characters (on one line) or up to the same column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Region {
    mode: Mode,
    /// Lines after the first.
    lines: usize,
    /// Characters, for a selection on one line; else the last line's
    /// column.
    width: usize,
}

/// What the next key completes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Wait {
    #[default]
    Nothing,
    /// After `g`.
    G,
    Find(FindKind),
    /// After `r`.
    Replace,
    /// After `i` (true) or `a` with an operator or in Visual mode.
    Object(bool),
}

#[derive(Clone, Copy, Debug, Default)]
struct Pending {
    count: Option<usize>,
    op: Option<Op>,
    op_count: Option<usize>,
    wait: Wait,
}

/// The text typed in Insert mode, as `.` types it again: so many
/// characters taken back before where it started and after, then `text`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inserted {
    back: usize,
    fwd: usize,
    text: String,
}

/// How a count repeats what was typed in Insert mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Repeat {
    /// After a change: no repeat.
    Never,
    /// `3ihi` types "hihihi".
    Text,
    /// `3ohi` opens three lines of "hi".
    Line,
}

/// A change `.` repeats: its keys without their counts, the count, and
/// what was typed if it went into Insert mode.
#[derive(Clone, Debug)]
struct Change {
    keys: Vec<Key>,
    count: Option<usize>,
    insert: Option<Inserted>,
    /// For a change made in Visual mode: the selection it was made over,
    /// and `keys` are the Visual mode keys.
    visual: Option<Region>,
}

/// The Insert mode a command began.
#[derive(Clone, Debug)]
struct Insertion {
    keys: Vec<Key>,
    count: Option<usize>,
    repeat: Repeat,
    visual: Option<Region>,
}

/// The text being changed, with the cursor.
struct Work {
    text: String,
    cursor: usize,
}

impl Work {
    fn replace(&mut self, range: Range<usize>, with: &str) {
        self.text.replace_range(range, with);
    }
}

#[derive(Default)]
pub struct Vim {
    mode: Mode,
    pending: Pending,
    /// The keys of the command being typed, without counts, for `.`.
    keys: Vec<Key>,
    /// The command being typed, as the footer shows it.
    shown: String,
    /// Visual mode's fixed end and moving end (the cursor).
    anchor: usize,
    head: usize,
    want: Option<Want>,
    register: Register,
    /// The clipboard's text when the register was last filled, to tell
    /// whether something was copied elsewhere since.
    clip_seen: Option<String>,
    last_find: Option<(FindKind, char)>,
    last_change: Option<Change>,
    session: Option<Insertion>,
    /// The ex command being typed after `:`.
    ex: Option<String>,
    /// While `.` replays a change, which mustn't record itself.
    replaying: bool,
    /// The selection a Visual mode change being made started from.
    region: Option<Region>,
    /// The first and last lines (0-based) the editor shows, for `H`, `M`
    /// and `L`; the whole text when unknown.
    view: Option<(usize, usize)>,
}

/// The ranges operators take.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Span {
    Chars(Range<usize>),
    /// From the start of the first line to the end of the last (before its
    /// newline).
    Lines(usize, usize),
}

impl Vim {
    /// A state machine in Normal mode with nothing pending, as when a note
    /// opens.
    pub fn new() -> Vim {
        Vim::default()
    }

    /// The mode the editor is in.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// What the footer shows: the mode and a command being typed.
    pub fn status(&self) -> Status {
        Status {
            mode: self.mode,
            pending: match &self.ex {
                Some(ex) => format!(":{ex}"),
                None => self.shown.clone(),
            },
        }
    }

    /// The first and last lines (0-based) the editor shows, for `H`, `M`
    /// and `L`.
    pub fn set_view(&mut self, view: Option<(usize, usize)>) {
        self.view = view;
    }

    /// Back to Normal mode with nothing pending, as when a note opens.
    pub fn reset(&mut self) {
        self.mode = Mode::Normal;
        self.clear_pending();
        self.session = None;
        self.ex = None;
        self.want = None;
    }

    /// Takes `key` in Normal or Visual mode, on `text` with the editor's
    /// cursor at `cursor`. (In Insert mode the editor types; Escape there
    /// goes to [`Vim::leave_insert`].)
    pub fn handle_key(
        &mut self,
        key: Key,
        text: &str,
        cursor: usize,
        clip: &mut dyn Clipboard,
    ) -> Outcome {
        let mut work = Work {
            text: text.to_owned(),
            cursor: clamp_normal(text, cursor),
        };
        let mut effects = Vec::new();
        let consumed = match self.mode {
            Mode::Insert => false,
            Mode::Normal => self.normal_key(key, &mut work, clip, &mut effects),
            Mode::Visual | Mode::VisualLine => {
                self.anchor = clamp_normal(text, self.anchor);
                self.head = clamp_normal(text, self.head);
                self.visual_key(key, &mut work, clip, &mut effects)
            }
        };
        self.outcome(text, work, effects, consumed)
    }

    /// Escape in Insert mode: `entry` is the text as Insert mode began at
    /// `entry_cursor`, `text` and `cursor` what they are now. Repeats what
    /// was typed for a count, steps the cursor back, and remembers it all
    /// for `.`.
    pub fn leave_insert(
        &mut self,
        entry: &str,
        entry_cursor: usize,
        text: &str,
        cursor: usize,
    ) -> Outcome {
        let inserted = inserted(entry, entry_cursor, text);
        let mut work = Work {
            text: text.to_owned(),
            cursor: floor_grapheme(text, cursor),
        };
        let session = self.session.take();
        if let Some(session) = &session {
            // A Visual mode change has no keys to repeat.
            if !self.replaying && !session.keys.is_empty() {
                self.last_change = Some(Change {
                    keys: session.keys.clone(),
                    count: session.count,
                    insert: Some(inserted.clone()),
                    visual: session.visual,
                });
            }
        }
        self.finish_insert(&mut work, &inserted, session.as_ref());
        self.outcome(text, work, Vec::new(), true)
    }

    fn outcome(&self, original: &str, work: Work, effects: Vec<Effect>, consumed: bool) -> Outcome {
        let edit = diff(original, &work.text);
        let (cursor, selection) = match self.mode {
            Mode::Insert => (floor_grapheme(&work.text, work.cursor), None),
            Mode::Normal => (clamp_normal(&work.text, work.cursor), None),
            Mode::Visual | Mode::VisualLine => (self.head, Some(self.visual_shown(&work.text))),
        };
        Outcome {
            edit,
            cursor,
            selection,
            mode: self.mode,
            effects,
            consumed,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = Pending::default();
        self.keys.clear();
        self.shown.clear();
        self.region = None;
    }

    fn has_pending(&self) -> bool {
        self.pending.count.is_some()
            || self.pending.op.is_some()
            || self.pending.wait != Wait::Nothing
    }

    /// The count for the command: the one before the operator times the
    /// one after it.
    fn total_count(&self) -> Option<usize> {
        match (self.pending.count, self.pending.op_count) {
            (None, None) => None,
            (a, b) => Some((a.unwrap_or(1) * b.unwrap_or(1)).clamp(1, MAX_COUNT)),
        }
    }

    fn count1(&self) -> usize {
        self.total_count().unwrap_or(1)
    }

    fn show(&mut self, key: Key) {
        if let Some(c) = key_char(key) {
            self.shown.push(c);
        }
    }

    /// A digit adds to a count, unless `0` would start one (then it is a
    /// motion).
    fn take_digit(&mut self, key: Key) -> bool {
        let Key::Char(c) = key else {
            return false;
        };
        let Some(d) = c.to_digit(10) else {
            return false;
        };
        let slot = if self.pending.op.is_some() {
            &mut self.pending.op_count
        } else {
            &mut self.pending.count
        };
        if d == 0 && slot.is_none() {
            return false;
        }
        *slot = Some(
            slot.unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(d as usize)
                .min(999_999),
        );
        self.shown.push(c);
        true
    }

    /// A finished command: a change is kept for `.`; anything else is
    /// forgotten.
    fn done(&mut self, change: bool) {
        if change && !self.replaying {
            self.last_change = Some(Change {
                keys: std::mem::take(&mut self.keys),
                count: self.total_count(),
                insert: None,
                visual: self.region,
            });
        }
        self.clear_pending();
    }

    /// Into Insert mode at `at`, from the command typed so far.
    fn enter_insert(&mut self, work: &mut Work, at: usize, repeat: Repeat) {
        work.cursor = at;
        self.session = Some(Insertion {
            keys: std::mem::take(&mut self.keys),
            count: self.total_count(),
            repeat,
            visual: self.region,
        });
        self.mode = Mode::Insert;
        self.want = None;
        self.clear_pending();
    }

    fn motion_context(&self, operator: bool) -> MotionContext {
        MotionContext {
            count: self.total_count(),
            operator,
            want: self.want,
            last_find: self.last_find,
            view: self.view,
        }
    }

    // ---- Normal mode ----

    fn normal_key(
        &mut self,
        key: Key,
        work: &mut Work,
        clip: &mut dyn Clipboard,
        effects: &mut Vec<Effect>,
    ) -> bool {
        if self.ex.is_some() {
            self.ex_key(key, work, effects);
            return true;
        }
        if key == Key::Esc {
            let had = self.has_pending();
            self.clear_pending();
            return had;
        }
        match self.pending.wait {
            Wait::Nothing => {}
            Wait::G => {
                self.pending.wait = Wait::Nothing;
                self.keys.push(key);
                match g_motion(key) {
                    Some(motion) => self.motion(motion, work, clip, effects),
                    None => self.clear_pending(),
                }
                return true;
            }
            Wait::Find(kind) => {
                self.pending.wait = Wait::Nothing;
                self.keys.push(key);
                match key {
                    Key::Char(c) => {
                        self.last_find = Some((kind, c));
                        self.motion(Motion::Find(kind, c), work, clip, effects);
                    }
                    _ => self.clear_pending(),
                }
                return true;
            }
            Wait::Replace => {
                self.pending.wait = Wait::Nothing;
                self.keys.push(key);
                let with = match key {
                    Key::Char(c) => Some(c.to_string()),
                    Key::Enter => Some("\n".to_owned()),
                    _ => None,
                };
                let changed = with.is_some_and(|with| self.replace_chars(work, &with));
                self.done(changed);
                return true;
            }
            Wait::Object(inner) => {
                self.pending.wait = Wait::Nothing;
                self.keys.push(key);
                let span = match key {
                    Key::Char(c) => {
                        object::object(&work.text, work.cursor, c, inner, self.count1())
                    }
                    _ => None,
                };
                match (span, self.pending.op) {
                    (Some(span), Some(op)) => {
                        let span = if span.linewise {
                            lines_of(&work.text, span.range.start, span.range.end)
                        } else {
                            Span::Chars(span.range)
                        };
                        self.operate(op, span, work, clip, effects);
                    }
                    _ => self.clear_pending(),
                }
                return true;
            }
        }
        if self.take_digit(key) {
            return true;
        }
        if let Some(motion) = self.motion_for(key) {
            self.keys.push(key);
            self.motion(motion, work, clip, effects);
            return true;
        }
        if key == Key::Redo {
            effects.push(Effect::Redo(self.count1()));
            self.clear_pending();
            return true;
        }
        self.keys.push(key);
        self.show(key);
        let Key::Char(c) = key else {
            // Tab and the like do nothing in Normal mode.
            self.clear_pending();
            return true;
        };
        if let Some(op) = self.pending.op {
            match c {
                'i' | 'a' => self.pending.wait = Wait::Object(c == 'i'),
                'g' => self.pending.wait = Wait::G,
                'f' => self.pending.wait = Wait::Find(FindKind::To),
                'F' => self.pending.wait = Wait::Find(FindKind::BackTo),
                't' => self.pending.wait = Wait::Find(FindKind::Till),
                'T' => self.pending.wait = Wait::Find(FindKind::BackTill),
                _ if Op::from_char(c) == Some(op) => {
                    let n = self.count1();
                    let first = line_start(&work.text, work.cursor);
                    let mut last = first;
                    for _ in 1..n {
                        match next_line(&work.text, last) {
                            Some(next) => last = next,
                            None => break,
                        }
                    }
                    let span = Span::Lines(first, line_end(&work.text, last));
                    self.operate(op, span, work, clip, effects);
                }
                _ => self.clear_pending(),
            }
            return true;
        }
        let at = work.cursor;
        let n = self.count1();
        match c {
            'd' | 'c' | 'y' | '>' | '<' => self.pending.op = Op::from_char(c),
            'g' => self.pending.wait = Wait::G,
            'f' => self.pending.wait = Wait::Find(FindKind::To),
            'F' => self.pending.wait = Wait::Find(FindKind::BackTo),
            't' => self.pending.wait = Wait::Find(FindKind::Till),
            'T' => self.pending.wait = Wait::Find(FindKind::BackTill),
            'r' => self.pending.wait = Wait::Replace,
            'x' | 'X' | 's' => {
                let span = if c == 'X' {
                    let mut start = at;
                    for _ in 0..n {
                        match prev_in_line(&work.text, start) {
                            Some(prev) => start = prev,
                            None => break,
                        }
                    }
                    Span::Chars(start..at)
                } else {
                    Span::Chars(at..chars_ahead(&work.text, at, n))
                };
                let op = if c == 's' { Op::Change } else { Op::Delete };
                if op == Op::Delete && span == Span::Chars(at..at) {
                    self.want = None;
                    self.clear_pending();
                } else {
                    self.operate(op, span, work, clip, effects);
                }
            }
            'D' | 'C' => {
                self.pending.op = Some(if c == 'D' { Op::Delete } else { Op::Change });
                self.motion(Motion::LineEnd, work, clip, effects);
            }
            'S' | 'Y' => {
                let op = if c == 'S' { Op::Change } else { Op::Yank };
                let mut last = line_start(&work.text, at);
                for _ in 1..n {
                    match next_line(&work.text, last) {
                        Some(next) => last = next,
                        None => break,
                    }
                }
                let span = Span::Lines(line_start(&work.text, at), line_end(&work.text, last));
                self.operate(op, span, work, clip, effects);
            }
            'J' => {
                let changed = join(work, line_start(&work.text, at), n.max(2));
                self.done(changed);
            }
            '~' => {
                let end = chars_ahead(&work.text, at, n);
                let changed = end > at && {
                    let toggled = toggle_case(&work.text[at..end]);
                    work.replace(at..end, &toggled);
                    let end = at + toggled.len();
                    work.cursor = if at_line_end(&work.text, end) {
                        last_in_line(&work.text, at)
                    } else {
                        end
                    };
                    true
                };
                self.want = None;
                self.done(changed);
            }
            'p' | 'P' => {
                let changed = self.put(c == 'P', n, work, clip);
                self.done(changed);
            }
            'i' => self.enter_insert(work, at, Repeat::Text),
            'a' => {
                let after = next_in_line(&work.text, at).unwrap_or(at);
                self.enter_insert(work, after, Repeat::Text);
            }
            'A' => {
                let end = line_end(&work.text, at);
                self.enter_insert(work, end, Repeat::Text);
            }
            'I' => {
                let start = line_start(&work.text, at);
                let indent = start + indent_of(&work.text, at).len();
                self.enter_insert(work, indent, Repeat::Text);
            }
            'o' | 'O' => {
                let indent = indent_of(&work.text, at).to_owned();
                let into = if c == 'o' {
                    let end = line_end(&work.text, at);
                    work.replace(end..end, &format!("\n{indent}"));
                    end + 1 + indent.len()
                } else {
                    let start = line_start(&work.text, at);
                    work.replace(start..start, &format!("{indent}\n"));
                    start + indent.len()
                };
                self.enter_insert(work, into, Repeat::Line);
            }
            'u' => {
                effects.push(Effect::Undo(n));
                self.want = None;
                self.clear_pending();
            }
            '.' => {
                let count = self.total_count();
                self.clear_pending();
                self.repeat(count, work, clip, effects);
            }
            'v' | 'V' => {
                self.mode = if c == 'v' {
                    Mode::Visual
                } else {
                    Mode::VisualLine
                };
                self.anchor = at;
                self.head = at;
                self.clear_pending();
            }
            ':' => {
                self.ex = Some(String::new());
                self.clear_pending();
            }
            _ => self.clear_pending(),
        }
        true
    }

    /// The motion a key names on its own, if any.
    fn motion_for(&self, key: Key) -> Option<Motion> {
        Some(match key {
            Key::Char('h') | Key::Left | Key::Backspace => Motion::Left,
            Key::Char('l') | Key::Right | Key::Char(' ') => Motion::Right,
            Key::Char('j') | Key::Down => Motion::Down,
            Key::Char('k') | Key::Up => Motion::Up,
            Key::Char('w') => Motion::WordForward(false),
            Key::Char('W') => Motion::WordForward(true),
            Key::Char('b') => Motion::WordBack(false),
            Key::Char('B') => Motion::WordBack(true),
            Key::Char('e') => Motion::WordEnd(false),
            Key::Char('E') => Motion::WordEnd(true),
            Key::Char('0') | Key::Home => Motion::LineStart,
            Key::Char('^') => Motion::FirstNonBlank,
            Key::Char('$') | Key::End => Motion::LineEnd,
            Key::Char('G') => Motion::LastLine,
            Key::Char('+') | Key::Enter => Motion::NextLineStart,
            Key::Char('-') => Motion::PrevLineStart,
            Key::Char(';') => Motion::RepeatFind(false),
            Key::Char(',') => Motion::RepeatFind(true),
            Key::Char('}') => Motion::ParagraphForward,
            Key::Char('{') => Motion::ParagraphBack,
            Key::Char('%') => Motion::Bracket,
            Key::Char('H') => Motion::ViewTop,
            Key::Char('M') => Motion::ViewMiddle,
            Key::Char('L') => Motion::ViewBottom,
            _ => return None,
        })
    }

    /// A motion: moves the cursor, or with an operator waiting, operates
    /// over what it moves across.
    fn motion(
        &mut self,
        motion: Motion,
        work: &mut Work,
        clip: &mut dyn Clipboard,
        effects: &mut Vec<Effect>,
    ) {
        let from = work.cursor;
        let vertical = matches!(motion, Motion::Up | Motion::Down);
        if vertical && self.want.is_none() {
            self.want = Some(Want::Column(column(&work.text, from)));
        }
        let Some(op) = self.pending.op else {
            let target = motion::eval(&work.text, from, motion, &self.motion_context(false));
            if let Some((to, _)) = target {
                work.cursor = if vertical {
                    to
                } else {
                    clamp_normal(&work.text, to)
                };
            }
            self.keep_want(motion, target.is_some());
            self.clear_pending();
            return;
        };
        let cx = self.motion_context(true);
        let on_word = class_at(&work.text, from, false) != 0;
        let target = match motion {
            // `cw` on a word changes to its end, like `ce`.
            Motion::WordForward(big) if op == Op::Change && on_word => {
                word_end_for_cw(&work.text, from, big, self.count1())
            }
            _ => motion::eval(&work.text, from, motion, &cx),
        };
        match target {
            Some((to, kind)) => {
                let span = span_of(&work.text, from, to, kind);
                self.operate(op, span, work, clip, effects);
            }
            None => {
                self.keep_want(motion, false);
                self.clear_pending();
            }
        }
    }

    /// Applies `op` over `span`; the command it finishes is done.
    fn operate(
        &mut self,
        op: Op,
        span: Span,
        work: &mut Work,
        clip: &mut dyn Clipboard,
        effects: &mut Vec<Effect>,
    ) {
        let from = work.cursor;
        // `>` and `<` take the lines the motion touches.
        if let Op::Shift(out) = op {
            let (first, end) = match span {
                Span::Lines(first, end) => (first, end),
                Span::Chars(range) => {
                    let last = if range.end > range.start && is_line_start(&work.text, range.end) {
                        range.end - 1
                    } else {
                        range.end
                    };
                    (
                        line_start(&work.text, range.start),
                        line_end(&work.text, last),
                    )
                }
            };
            let changed = shift_lines(work, first, end, out, 1);
            work.cursor = first_non_blank(&work.text, first);
            self.want = None;
            self.done(changed);
            return;
        }
        // Nothing to take (`x` on an empty line, `y$` on one): nothing
        // happens, and the register keeps what it had.
        if op != Op::Change && matches!(&span, Span::Chars(range) if range.is_empty()) {
            self.want = None;
            self.clear_pending();
            return;
        }
        let register = match &span {
            Span::Chars(range) => Register {
                text: work.text[range.clone()].to_owned(),
                linewise: false,
            },
            Span::Lines(start, end) => Register {
                text: format!("{}\n", &work.text[*start..*end]),
                linewise: true,
            },
        };
        self.want = None;
        match op {
            // Taken above.
            Op::Shift(_) => {}
            Op::Yank => {
                effects.push(Effect::Copy(register.text.clone()));
                self.clip_seen = Some(register.text.clone());
                self.register = register;
                work.cursor = match &span {
                    Span::Chars(range) => range.start,
                    Span::Lines(start, _) if from < *start => {
                        at_column(&work.text, *start, column(&work.text, from), false)
                    }
                    Span::Lines(..) => from,
                };
                self.done(false);
            }
            Op::Delete => {
                if matches!(&span, Span::Chars(range) if range.is_empty()) {
                    self.clear_pending();
                    return;
                }
                self.clip_seen = clip.read();
                self.register = register;
                match span {
                    Span::Chars(range) => {
                        work.replace(range.clone(), "");
                        work.cursor = range.start;
                    }
                    Span::Lines(start, end) => {
                        let len = work.text.len();
                        // The lines go with a line break: their last one's,
                        // or at the end of the text the one before them.
                        let (cut, after) = if end < len {
                            (start..end + 1, start)
                        } else if start > 0 {
                            (start - 1..end, line_start(&work.text, start - 1))
                        } else {
                            (start..end, 0)
                        };
                        work.replace(cut, "");
                        work.cursor = first_non_blank(&work.text, after.min(work.text.len()));
                    }
                }
                self.done(true);
            }
            Op::Change => {
                self.clip_seen = clip.read();
                self.register = register;
                match span {
                    Span::Chars(range) => {
                        work.replace(range.clone(), "");
                        self.enter_insert(work, range.start, Repeat::Never);
                    }
                    Span::Lines(start, end) => {
                        // The lines' text goes; the first one's indent stays.
                        let indent = indent_of(&work.text, start).to_owned();
                        work.replace(start..end, &indent);
                        self.enter_insert(work, start + indent.len(), Repeat::Never);
                    }
                }
            }
        }
    }

    /// `r`: the next `count` characters each become `with` (all of them
    /// one line break, for Enter). Nothing changes when there are fewer.
    fn replace_chars(&mut self, work: &mut Work, with: &str) -> bool {
        let at = work.cursor;
        let n = self.count1();
        let mut end = at;
        for _ in 0..n {
            match next_in_line(&work.text, end) {
                Some(next) if !at_line_end(&work.text, end) => end = next,
                _ => return false,
            }
        }
        if with == "\n" {
            work.replace(at..end, "\n");
            work.cursor = at + 1;
        } else {
            work.replace(at..end, &with.repeat(n));
            work.cursor = at + with.len() * (n - 1);
        }
        self.want = None;
        true
    }

    /// `p` or `P`, `count` times. A register of whole lines goes in below
    /// (or above) the cursor's line; text goes in after (or at) the cursor.
    fn put(
        &mut self,
        before: bool,
        count: usize,
        work: &mut Work,
        clip: &mut dyn Clipboard,
    ) -> bool {
        if let Some(text) = clip.read()
            && Some(&text) != self.clip_seen.as_ref()
        {
            // Copied elsewhere since: that's what goes in.
            self.register = Register {
                linewise: text.ends_with('\n'),
                text: text.clone(),
            };
            self.clip_seen = Some(text);
        }
        if self.register.text.is_empty() {
            return false;
        }
        let at = work.cursor;
        self.want = None;
        if self.register.linewise {
            let lines = self.register.text.repeat(count);
            let into = if before {
                let start = line_start(&work.text, at);
                work.replace(start..start, &lines);
                start
            } else {
                let end = line_end(&work.text, at);
                if end < work.text.len() {
                    work.replace(end + 1..end + 1, &lines);
                    end + 1
                } else {
                    let lines = format!("\n{}", &lines[..lines.len() - 1]);
                    work.replace(end..end, &lines);
                    end + 1
                }
            };
            work.cursor = first_non_blank(&work.text, into);
        } else {
            let text = self.register.text.repeat(count);
            let into = if before || at_line_end(&work.text, at) {
                at
            } else {
                next_in_line(&work.text, at).unwrap_or(at)
            };
            work.replace(into..into, &text);
            let end = into + text.len();
            work.cursor = if text.contains('\n') {
                into
            } else {
                prev_in_line(&work.text, end).unwrap_or(into)
            };
        }
        true
    }

    /// `.`: the last change again, with `count` in place of its own.
    fn repeat(
        &mut self,
        count: Option<usize>,
        work: &mut Work,
        clip: &mut dyn Clipboard,
        effects: &mut Vec<Effect>,
    ) {
        let Some(change) = self.last_change.clone() else {
            return;
        };
        self.replaying = true;
        match change.visual {
            // A Visual mode change: a selection the same size from the
            // cursor, then the same keys.
            Some(region) => {
                self.mode = region.mode;
                self.anchor = work.cursor;
                self.head = region_head(&work.text, work.cursor, region);
                self.want = None;
                self.pending.count = count.or(change.count);
                for key in &change.keys {
                    self.visual_key(*key, work, clip, effects);
                }
                if self.mode.is_visual() {
                    self.leave_visual(work);
                }
            }
            None => {
                self.pending.count = count.or(change.count);
                for key in &change.keys {
                    self.normal_key(*key, work, clip, effects);
                }
            }
        }
        if self.mode == Mode::Insert {
            let inserted = change.insert.clone().unwrap_or_default();
            let at = work.cursor;
            let mut start = at;
            for _ in 0..inserted.back {
                match work.text[..start].chars().next_back() {
                    Some(c) => start -= c.len_utf8(),
                    None => break,
                }
            }
            let mut end = at;
            for _ in 0..inserted.fwd {
                match work.text[end..].chars().next() {
                    Some(c) => end += c.len_utf8(),
                    None => break,
                }
            }
            work.replace(start..end, &inserted.text);
            work.cursor = start + inserted.text.len();
            let session = self.session.take();
            self.finish_insert(work, &inserted, session.as_ref());
        }
        self.clear_pending();
        self.replaying = false;
    }

    /// The end of Insert mode: a count types the text again, and the
    /// cursor steps back onto the last character typed.
    fn finish_insert(&mut self, work: &mut Work, inserted: &Inserted, session: Option<&Insertion>) {
        if let Some(session) = session {
            let extra = session.count.unwrap_or(1).clamp(1, MAX_COUNT) - 1;
            if extra > 0 {
                match session.repeat {
                    Repeat::Never => {}
                    Repeat::Text => {
                        let more = inserted.text.repeat(extra);
                        let at = work.cursor;
                        work.replace(at..at, &more);
                        work.cursor = at + more.len();
                    }
                    Repeat::Line => {
                        let indent = indent_of(&work.text, work.cursor).to_owned();
                        let line = format!("\n{indent}{}", inserted.text);
                        let more = line.repeat(extra);
                        let end = line_end(&work.text, work.cursor);
                        work.replace(end..end, &more);
                        work.cursor = end + more.len();
                    }
                }
            }
        }
        self.mode = Mode::Normal;
        self.want = None;
        let at = floor_grapheme(&work.text, work.cursor);
        work.cursor = prev_in_line(&work.text, at).unwrap_or(at);
    }

    // ---- Visual modes ----

    fn visual_key(
        &mut self,
        key: Key,
        work: &mut Work,
        clip: &mut dyn Clipboard,
        effects: &mut Vec<Effect>,
    ) -> bool {
        if self.ex.is_some() {
            self.ex_key(key, work, effects);
            return true;
        }
        if key == Key::Esc {
            if self.has_pending() {
                self.clear_pending();
            } else {
                self.leave_visual(work);
            }
            return true;
        }
        match self.pending.wait {
            Wait::Nothing => {}
            Wait::G => {
                if let Some(motion) = g_motion(key) {
                    self.visual_motion(motion, work);
                }
                self.clear_pending();
                return true;
            }
            Wait::Find(kind) => {
                if let Key::Char(c) = key {
                    self.last_find = Some((kind, c));
                    self.visual_motion(Motion::Find(kind, c), work);
                }
                self.clear_pending();
                return true;
            }
            Wait::Replace => {
                self.clear_pending();
                if let Key::Char(c) = key {
                    let region = self.selection_region(&work.text);
                    self.visual_done(vec![Key::Char('r'), key], None, region);
                    let span = self.visual_span(&work.text);
                    let range = span_range(&span);
                    let replaced: String = work.text[range.clone()]
                        .chars()
                        .map(|ch| if ch == '\n' { ch } else { c })
                        .collect();
                    work.replace(range.clone(), &replaced);
                    self.mode = Mode::Normal;
                    work.cursor = range.start;
                }
                return true;
            }
            Wait::Object(inner) => {
                self.clear_pending();
                if let Key::Char(c) = key
                    && let Some(span) = object::object(&work.text, self.head, c, inner, 1)
                {
                    if span.linewise {
                        self.mode = Mode::VisualLine;
                    }
                    let last = prev_grapheme_or(&work.text, span.range.end, span.range.start);
                    if self.anchor == self.head {
                        self.anchor = span.range.start;
                    }
                    self.head = if self.head >= self.anchor {
                        last.max(self.anchor)
                    } else {
                        span.range.start
                    };
                }
                return true;
            }
        }
        if self.take_digit(key) {
            return true;
        }
        if let Some(motion) = self.motion_for(key) {
            self.visual_motion(motion, work);
            self.clear_pending();
            return true;
        }
        self.show(key);
        let Key::Char(c) = key else {
            self.clear_pending();
            return true;
        };
        let linewise_cmd = matches!(c, 'X' | 'D' | 'Y' | 'C' | 'S' | 'R');
        match c {
            'g' => self.pending.wait = Wait::G,
            'f' => self.pending.wait = Wait::Find(FindKind::To),
            'F' => self.pending.wait = Wait::Find(FindKind::BackTo),
            't' => self.pending.wait = Wait::Find(FindKind::Till),
            'T' => self.pending.wait = Wait::Find(FindKind::BackTill),
            'r' => self.pending.wait = Wait::Replace,
            'i' | 'a' => self.pending.wait = Wait::Object(c == 'i'),
            'o' => {
                std::mem::swap(&mut self.anchor, &mut self.head);
                self.clear_pending();
            }
            'v' | 'V' => {
                let want = if c == 'v' {
                    Mode::Visual
                } else {
                    Mode::VisualLine
                };
                if self.mode == want {
                    self.leave_visual(work);
                } else {
                    self.mode = want;
                }
                self.clear_pending();
            }
            'd' | 'x' | 'X' | 'D' | 'y' | 'Y' | 'c' | 's' | 'C' | 'S' | 'R' => {
                let span = if linewise_cmd {
                    let (a, b) = self.ordered();
                    lines_of(&work.text, a, b)
                } else {
                    self.visual_span(&work.text)
                };
                let op = match c {
                    'y' | 'Y' => Op::Yank,
                    'c' | 's' | 'C' | 'S' | 'R' => Op::Change,
                    _ => Op::Delete,
                };
                let region = self.selection_region(&work.text);
                self.mode = Mode::Normal;
                work.cursor = self.ordered().0;
                self.clear_pending();
                // A delete or change is kept for `.` with the selection's
                // size.
                self.keys = vec![Key::Char(c)];
                self.region = Some(region);
                self.operate(op, span, work, clip, effects);
            }
            '>' | '<' => {
                let region = self.selection_region(&work.text);
                let count = self.total_count();
                let (a, b) = self.ordered();
                let first = line_start(&work.text, a);
                let end = line_end(&work.text, b);
                let changed = shift_lines(work, first, end, c == '<', count.unwrap_or(1));
                self.mode = Mode::Normal;
                work.cursor = first_non_blank(&work.text, first);
                self.clear_pending();
                if changed {
                    self.visual_done(vec![Key::Char(c)], count, region);
                }
            }
            '~' | 'u' | 'U' => {
                let region = self.selection_region(&work.text);
                self.visual_done(vec![Key::Char(c)], None, region);
                let span = self.visual_span(&work.text);
                let range = span_range(&span);
                let old = &work.text[range.clone()];
                let new = match c {
                    '~' => toggle_case(old),
                    'u' => old.to_lowercase(),
                    _ => old.to_uppercase(),
                };
                work.replace(range.clone(), &new);
                self.mode = Mode::Normal;
                work.cursor = range.start;
                self.clear_pending();
            }
            'J' => {
                let region = self.selection_region(&work.text);
                self.visual_done(vec![Key::Char(c)], None, region);
                let (a, b) = self.ordered();
                let first = line_start(&work.text, a);
                let mut lines = 1;
                let mut at = first;
                let last = line_start(&work.text, b);
                while at < last {
                    match next_line(&work.text, at) {
                        Some(next) => {
                            at = next;
                            lines += 1;
                        }
                        None => break,
                    }
                }
                join(work, first, lines.max(2));
                self.mode = Mode::Normal;
                self.clear_pending();
            }
            'p' | 'P' => {
                let span = self.visual_span(&work.text);
                if let Some(text) = clip.read()
                    && Some(&text) != self.clip_seen.as_ref()
                {
                    self.register = Register {
                        linewise: text.ends_with('\n'),
                        text: text.clone(),
                    };
                    self.clip_seen = Some(text);
                }
                let put = self.register.clone();
                if !put.text.is_empty() {
                    let (cut, replaced, with) = match span {
                        // Lines replace lines, their line break and all.
                        Span::Lines(start, end) => {
                            let mut with = put.text.clone();
                            if !put.linewise {
                                with.push('\n');
                            }
                            let replaced = Register {
                                text: format!("{}\n", &work.text[start..end]),
                                linewise: true,
                            };
                            if end < work.text.len() {
                                (start..end + 1, replaced, with)
                            } else {
                                (start..end, replaced, with.trim_end_matches('\n').to_owned())
                            }
                        }
                        Span::Chars(range) => {
                            let with = if put.linewise {
                                format!("\n{}", put.text)
                            } else {
                                put.text.clone()
                            };
                            let replaced = Register {
                                text: work.text[range.clone()].to_owned(),
                                linewise: false,
                            };
                            (range, replaced, with)
                        }
                    };
                    // What was selected is in the register now.
                    self.register = replaced;
                    self.clip_seen = clip.read();
                    work.replace(cut.clone(), &with);
                    work.cursor = cut.start;
                }
                self.mode = Mode::Normal;
                self.clear_pending();
            }
            ':' => {
                self.leave_visual(work);
                self.ex = Some(String::new());
                self.clear_pending();
            }
            _ => self.clear_pending(),
        }
        true
    }

    /// The size of the selection, for `.` to take again.
    fn selection_region(&self, text: &str) -> Region {
        let (a, b) = self.ordered();
        let last = line_start(text, b);
        let mut lines = 0;
        let mut at = line_start(text, a);
        while at < last {
            match next_line(text, at) {
                Some(next) => {
                    at = next;
                    lines += 1;
                }
                None => break,
            }
        }
        let width = if lines == 0 {
            column(text, b) - column(text, a) + 1
        } else {
            column(text, b)
        };
        Region {
            mode: self.mode,
            lines,
            width,
        }
    }

    /// A change made in Visual mode, kept for `.` (unless `.` is making
    /// it).
    fn visual_done(&mut self, keys: Vec<Key>, count: Option<usize>, region: Region) {
        if !self.replaying {
            self.last_change = Some(Change {
                keys,
                count,
                insert: None,
                visual: Some(region),
            });
        }
    }

    fn leave_visual(&mut self, work: &mut Work) {
        self.mode = Mode::Normal;
        work.cursor = self.head;
        self.clear_pending();
    }

    fn visual_motion(&mut self, motion: Motion, work: &mut Work) {
        let vertical = matches!(motion, Motion::Up | Motion::Down);
        if vertical && self.want.is_none() {
            self.want = Some(Want::Column(column(&work.text, self.head)));
        }
        let target = motion::eval(&work.text, self.head, motion, &self.motion_context(false));
        if let Some((to, _)) = target {
            self.head = if vertical {
                to
            } else {
                clamp_normal(&work.text, to)
            };
        }
        self.keep_want(motion, target.is_some());
        work.cursor = self.head;
    }

    /// The column `j` and `k` keep after `motion`: theirs, the end of the
    /// line after `$`, or wherever the cursor is now after any other
    /// (even one that couldn't move, as in Vim).
    fn keep_want(&mut self, motion: Motion, moved: bool) {
        self.want = match motion {
            Motion::Up | Motion::Down => self.want,
            Motion::LineEnd if moved => Some(Want::End),
            _ => None,
        };
    }

    /// The two ends of the selection, first one first.
    fn ordered(&self) -> (usize, usize) {
        (self.anchor.min(self.head), self.anchor.max(self.head))
    }

    /// What the selection covers, for an operator: the characters from one
    /// end to the other (an empty line's break included), or whole lines.
    fn visual_span(&self, text: &str) -> Span {
        let (a, b) = self.ordered();
        if self.mode == Mode::VisualLine {
            return lines_of(text, a, b);
        }
        let end = if b < text.len() && text.as_bytes()[b] == b'\n' {
            b + 1
        } else {
            b + grapheme_len(text, b)
        };
        Span::Chars(a..end.min(text.len()))
    }

    /// The selection as the editor shows it.
    fn visual_shown(&self, text: &str) -> Range<usize> {
        match self.visual_span(text) {
            Span::Chars(range) => range,
            Span::Lines(start, end) => start..end,
        }
    }

    // ---- Ex commands ----

    fn ex_key(&mut self, key: Key, work: &mut Work, effects: &mut Vec<Effect>) {
        let Some(ex) = self.ex.as_mut() else {
            return;
        };
        match key {
            Key::Esc => self.ex = None,
            Key::Backspace => {
                if ex.pop().is_none() {
                    self.ex = None;
                }
            }
            Key::Char(c) => ex.push(c),
            Key::Enter => {
                let command = self.ex.take().unwrap_or_default();
                self.run_ex(command.trim(), work, effects);
            }
            _ => {}
        }
    }

    /// Runs an ex command. Ones Kanso has no use for do nothing.
    fn run_ex(&mut self, command: &str, work: &mut Work, effects: &mut Vec<Effect>) {
        match command {
            "w" | "w!" | "write" | "up" | "update" => effects.push(Effect::Save),
            "q" | "q!" | "quit" | "clo" | "close" => effects.push(Effect::Close),
            "wq" | "wq!" | "x" | "x!" | "xit" | "exit" => {
                effects.push(Effect::Save);
                effects.push(Effect::Close);
            }
            _ => {
                if let Ok(line) = command.parse::<usize>() {
                    let start = nth_line(&work.text, line.max(1) - 1);
                    work.cursor = first_non_blank(&work.text, start);
                    self.want = None;
                }
            }
        }
    }
}

/// What a key types, as the footer shows it.
/// The motion `g` and `key` make.
fn g_motion(key: Key) -> Option<Motion> {
    match key {
        Key::Char('g') => Some(Motion::FirstLine),
        Key::Char('e') => Some(Motion::WordEndBack(false)),
        Key::Char('E') => Some(Motion::WordEndBack(true)),
        _ => None,
    }
}

fn key_char(key: Key) -> Option<char> {
    match key {
        Key::Char(c) => Some(c),
        _ => None,
    }
}

/// The text inserted in Insert mode: from the text at its start and at its
/// end, relative to where it started. Typing at the cursor (the usual
/// case) is found exactly, even beside the same characters.
fn inserted(entry: &str, entry_cursor: usize, text: &str) -> Inserted {
    let at = floor_char(entry, entry_cursor);
    if text.len() >= entry.len() {
        let added = text.len() - entry.len();
        if text.starts_with(&entry[..at])
            && text.is_char_boundary(at + added)
            && text[at + added..] == entry[at..]
        {
            return Inserted {
                back: 0,
                fwd: 0,
                text: text[at..at + added].to_owned(),
            };
        }
    }
    match diff(entry, text) {
        None => Inserted::default(),
        Some((range, new)) => {
            if range.start <= at && at <= range.end {
                Inserted {
                    back: entry[range.start..at].chars().count(),
                    fwd: entry[at..range.end].chars().count(),
                    text: new,
                }
            } else {
                Inserted {
                    back: 0,
                    fwd: 0,
                    text: new,
                }
            }
        }
    }
}

/// The end of `count` characters from `at` on its line.
fn chars_ahead(text: &str, at: usize, count: usize) -> usize {
    let mut end = at;
    for _ in 0..count {
        match next_in_line(text, end) {
            Some(next) => end = next,
            None => break,
        }
    }
    end
}

/// The start of the grapheme before `end`, not before `floor`.
fn prev_grapheme_or(text: &str, end: usize, floor: usize) -> usize {
    if end <= floor {
        return floor;
    }
    if is_line_start(text, end) {
        // The span ends with a line break: its last character is that.
        return (end - 1).max(floor);
    }
    prev_in_line(text, end).unwrap_or(floor).max(floor)
}

/// The whole lines from `a`'s to `b`'s.
fn lines_of(text: &str, a: usize, b: usize) -> Span {
    Span::Lines(line_start(text, a), line_end(text, b))
}

/// The text a span covers, newline after whole lines aside.
fn span_range(span: &Span) -> Range<usize> {
    match span {
        Span::Chars(range) => range.clone(),
        Span::Lines(start, end) => *start..*end,
    }
}

/// What an operator over a motion from `from` to `to` takes. An exclusive
/// motion that ends at the start of a later line stops at the end of the
/// line before, and takes whole lines when it began in the indent (Vim's
/// `:help exclusive` rules).
fn span_of(text: &str, from: usize, to: usize, kind: Kind) -> Span {
    let (a, b) = (from.min(to), from.max(to));
    match kind {
        Kind::Linewise => lines_of(text, a, b),
        Kind::Inclusive => Span::Chars(a..b + grapheme_len(text, b)),
        Kind::Exclusive => {
            if b > a && is_line_start(text, b) && line_start(text, a) < line_start(text, b) {
                if in_indent(text, a) {
                    return lines_of(text, a, b - 1);
                }
                return Span::Chars(a..b - 1);
            }
            Span::Chars(a..b)
        }
    }
}

/// `cw`: to the end of the word under the cursor.
fn word_end_for_cw(text: &str, from: usize, big: bool, count: usize) -> Option<(usize, Kind)> {
    let end = motion::word_end(text, from, big, count, true).unwrap_or(text.len());
    Some((end, Kind::Inclusive))
}

/// Joins `lines` lines from the one starting at `first`: each line break
/// and the next line's indent become one space, or nothing after a blank,
/// before an empty line or a `)`, or on an empty line. The cursor goes
/// where the last two lines met. Whether anything was joined.
fn join(work: &mut Work, first: usize, lines: usize) -> bool {
    let mut joined = false;
    let start = first;
    for _ in 1..lines {
        let end = line_end(&work.text, start);
        if end >= work.text.len() {
            break;
        }
        let next = end + 1;
        let content = next + indent_of(&work.text, next).len();
        let current_empty = line_start(&work.text, end) == end;
        let ends_blank = end > start && matches!(work.text.as_bytes()[end - 1], b' ' | b'\t');
        let next_bare = at_line_end(&work.text, content) || work.text[content..].starts_with(')');
        let space = if current_empty || ends_blank || next_bare {
            ""
        } else {
            " "
        };
        work.replace(end..content, space);
        work.cursor = end;
        joined = true;
    }
    joined
}

/// Upper case to lower and lower to upper, character by character.
/// Where a selection the size of `region` from `at` ends.
fn region_head(text: &str, at: usize, region: Region) -> usize {
    let mut start = line_start(text, at);
    for _ in 0..region.lines {
        match next_line(text, start) {
            Some(next) => start = next,
            None => break,
        }
    }
    match region.mode {
        Mode::VisualLine => start,
        _ if region.lines == 0 => at_column(
            text,
            start,
            column(text, at) + region.width.max(1) - 1,
            false,
        ),
        _ => at_column(text, start, region.width, false),
    }
}

/// `>` and `<`, `times` over: the lines from the one starting at `first`
/// to the one ending at `end` go in by [`INDENT`] (but empty lines), or out
/// by as much of it as they have. Whether anything changed.
fn shift_lines(work: &mut Work, first: usize, end: usize, out: bool, times: usize) -> bool {
    let old = &work.text[first..end];
    let times = times.clamp(1, MAX_COUNT);
    let shifted: Vec<String> = old
        .split('\n')
        .map(|line| {
            if out {
                let mut line = line;
                for _ in 0..times {
                    line = match line.strip_prefix('\t') {
                        Some(rest) => rest,
                        None => {
                            let blanks = line.len() - line.trim_start_matches(' ').len();
                            &line[blanks.min(INDENT.len())..]
                        }
                    };
                }
                line.to_owned()
            } else if line.is_empty() {
                String::new()
            } else {
                format!("{}{line}", INDENT.repeat(times))
            }
        })
        .collect();
    let new = shifted.join("\n");
    if new == old {
        return false;
    }
    work.replace(first..end, &new);
    true
}

fn toggle_case(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            let swapped: Vec<char> = if c.is_lowercase() {
                c.to_uppercase().collect()
            } else if c.is_uppercase() {
                c.to_lowercase().collect()
            } else {
                vec![c]
            };
            swapped
        })
        .collect()
}
