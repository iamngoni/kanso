//! The Vim state machine against Vim's own behaviour (examples), against a
//! small reference model (motions with `d` and `y`), and its invariants:
//! the cursor always on a character in the text, `u` undoing any change,
//! `.` doing what typing the change again does, and `yy` then `p`.

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use super::text::*;
use super::*;

pub(super) fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 2048,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// The system clipboard, as a test has it.
#[derive(Default)]
pub(super) struct FakeClip(pub(super) Option<String>);

impl Clipboard for FakeClip {
    fn read(&mut self) -> Option<String> {
        self.0.clone()
    }
}

/// What the editor does around the state machine: types in Insert mode,
/// applies each outcome, keeps an undo stack where each Vim change (an
/// Insert mode session with the command that began it) is one step, and
/// the clipboard.
pub(super) struct Buf {
    pub(super) text: String,
    pub(super) cursor: usize,
    pub(super) vim: Vim,
    clip: FakeClip,
    undo: Vec<(String, usize)>,
    redo: Vec<(String, usize)>,
    /// The text and cursor before the command that began Insert mode.
    base: Option<(String, usize)>,
    /// The text and cursor as Insert mode began.
    entry: Option<(String, usize)>,
    effects: Vec<Effect>,
    consumed: bool,
}

impl Buf {
    pub(super) fn new(text: &str, cursor: usize) -> Buf {
        Buf {
            text: text.to_owned(),
            cursor: clamp_normal(text, cursor),
            vim: Vim::new(),
            clip: FakeClip::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            base: None,
            entry: None,
            effects: Vec::new(),
            consumed: true,
        }
    }

    /// A text with `|` where the cursor is.
    fn at(marked: &str) -> Buf {
        let cursor = marked.find('|').expect("a cursor");
        Buf::new(&marked.replacen('|', "", 1), cursor)
    }

    fn marked(&self) -> String {
        let mut text = self.text.clone();
        text.insert(self.cursor, '|');
        text
    }

    pub(super) fn key(&mut self, key: Key) {
        if self.vim.mode() == Mode::Insert {
            self.insert_key(key);
            return;
        }
        let before = (self.text.clone(), self.cursor);
        let was = self.vim.mode();
        let out = self
            .vim
            .handle_key(key, &self.text, self.cursor, &mut self.clip);
        self.consumed = out.consumed;
        let edited = self.apply(&out);
        for effect in &out.effects {
            match effect {
                Effect::Copy(text) => self.clip.0 = Some(text.clone()),
                Effect::Undo(n) => {
                    for _ in 0..*n {
                        if let Some(state) = self.undo.pop() {
                            self.redo.push((self.text.clone(), self.cursor));
                            (self.text, self.cursor) = state;
                        }
                    }
                    self.cursor = clamp_normal(&self.text, self.cursor);
                }
                Effect::Redo(n) => {
                    for _ in 0..*n {
                        if let Some(state) = self.redo.pop() {
                            self.undo.push((self.text.clone(), self.cursor));
                            (self.text, self.cursor) = state;
                        }
                    }
                    self.cursor = clamp_normal(&self.text, self.cursor);
                }
                _ => {}
            }
        }
        self.effects.extend(out.effects);
        if out.mode == Mode::Insert && was != Mode::Insert {
            self.base = Some(before);
            self.entry = Some((self.text.clone(), self.cursor));
        } else if edited {
            self.undo.push(before);
            self.redo.clear();
        }
    }

    fn insert_key(&mut self, key: Key) {
        match key {
            Key::Esc => {
                let (entry, entry_cursor) = self.entry.take().unwrap_or_default();
                let out = self
                    .vim
                    .leave_insert(&entry, entry_cursor, &self.text, self.cursor);
                self.apply(&out);
                if let Some(base) = self.base.take()
                    && base.0 != self.text
                {
                    self.undo.push(base);
                    self.redo.clear();
                }
            }
            Key::Char(c) => {
                self.text.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            Key::Enter => {
                self.text.insert(self.cursor, '\n');
                self.cursor += 1;
            }
            Key::Backspace => {
                if let Some(c) = self.text[..self.cursor].chars().next_back() {
                    self.cursor -= c.len_utf8();
                    self.text.remove(self.cursor);
                }
            }
            _ => {}
        }
    }

    fn apply(&mut self, out: &Outcome) -> bool {
        if let Some((range, with)) = &out.edit {
            self.text.replace_range(range.clone(), with);
        }
        self.cursor = out.cursor;
        out.edit.is_some()
    }

    /// Types keys written the way Vim's docs write them: `<Esc>`, `<CR>`,
    /// `<BS>` and `<C-r>` for those keys, anything else as itself.
    fn keys(&mut self, keys: &str) -> &mut Buf {
        for key in parse_keys(keys) {
            self.key(key);
        }
        self
    }
}

fn parse_keys(keys: &str) -> Vec<Key> {
    let mut out = Vec::new();
    let mut rest = keys;
    while let Some(c) = rest.chars().next() {
        let named = [
            ("<Esc>", Key::Esc),
            ("<CR>", Key::Enter),
            ("<BS>", Key::Backspace),
            ("<C-r>", Key::Redo),
            ("<Tab>", Key::Tab),
        ]
        .into_iter()
        .find(|(name, _)| rest.starts_with(name));
        match named {
            Some((name, key)) => {
                out.push(key);
                rest = &rest[name.len()..];
            }
            None => {
                out.push(Key::Char(c));
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    out
}

/// `keys` typed into `start` (with `|` for the cursor) gives `expect`.
#[track_caller]
fn check(start: &str, keys: &str, expect: &str) {
    let mut buf = Buf::at(start);
    buf.keys(keys);
    assert_eq!(buf.marked(), expect, "{start:?} after {keys:?}");
}

// ---- Motions ----

#[test]
fn moving_by_characters_and_lines() {
    check("|abc", "l", "a|bc");
    check("|abc", "5l", "ab|c");
    check("ab|c", "l", "ab|c");
    check("a|bc", "h", "|abc");
    check("abcd\nx\nab|cd", "kk", "ab|cd\nx\nabcd");
    // The column is kept through a short line.
    check("abc|d\nx\nabcd", "jj", "abcd\nx\nabc|d");
    check("|a\nb\nc", "2j", "a\nb\n|c");
    check("|a\nb", "5j", "a\n|b");
    check("ab|c\nabcdef", "$j", "abc\nabcde|f");
    check("  a|b", "0", "|  ab");
    check("  a|b", "^", "  |ab");
    check("|ab\ncd", "$", "a|b\ncd");
    check("|ab\ncd", "2$", "ab\nc|d");
    check("a\n  b\n|c", "gg", "|a\n  b\nc");
    check("|a\n  b\nc", "G", "a\n  b\n|c");
    check("|a\n  b\nc", "2G", "a\n  |b\nc");
    check("|a\n  b", "+", "a\n  |b");
    check("a\n  |b", "-", "|a\n  b");
}

#[test]
fn moving_by_words() {
    check("|foo bar", "w", "foo |bar");
    check("|foo.bar", "w", "foo|.bar");
    check("|foo.bar baz", "W", "foo.bar |baz");
    check("|foo\nbar", "w", "foo\n|bar");
    // An empty line is a word of its own.
    check("|a\n\nb", "w", "a\n|\nb");
    check("|a\n\nb", "2w", "a\n\n|b");
    check("foo |bar", "b", "|foo bar");
    check("foo\n|bar", "b", "|foo\nbar");
    check("a\n\n|b", "b", "a\n|\nb");
    check("|foo bar", "e", "fo|o bar");
    check("fo|o bar", "e", "foo ba|r");
    check("|foo.bar baz", "E", "foo.ba|r baz");
    check("foo.bar |baz", "B", "|foo.bar baz");
    // The last word: `w` stops on the last character.
    check("foo |bar", "w", "foo ba|r");
}

#[test]
fn finding_characters() {
    check("|a,b,c", "f,", "a|,b,c");
    check("|a,b,c", "f,;", "a,b|,c");
    check("|a,b,c", "f,;,", "a|,b,c");
    check("|a,b,c", "2f,", "a,b|,c");
    check("a,b,|c", "F,", "a,b|,c");
    check("|a,b,c", "t,", "|a,b,c");
    // `;` after `t` doesn't stay stuck before the same comma.
    check("|a,b,c", "t,;", "a,|b,c");
    check("a,b,|c", "T,", "a,b,|c");
    check("|abc", "fz", "|abc");
}

#[test]
fn paragraphs_and_brackets() {
    check("|a\nb\n\nc\nd", "}", "a\nb\n|\nc\nd");
    check("|a\nb\n\nc\nd", "2}", "a\nb\n\nc\n|d");
    check("a\nb\n\nc\n|d", "{", "a\nb\n|\nc\nd");
    check("|(a [b])", "%", "(a [b]|)");
    check("(a [b]|)", "%", "|(a [b])");
    check("x |= (a)", "%", "x = (a|)");
}

#[test]
fn graphemes_are_single_steps() {
    check("|👨‍👩‍👧x", "l", "👨‍👩‍👧|x");
    check("|e\u{301}x", "l", "e\u{301}|x");
    check("|👍👍", "x", "|👍");
    check("a|é", "x", "|a");
    check("|日本語", "$", "日本|語");
}

// ---- Operators ----

#[test]
fn delete_with_motions() {
    check("foo |bar baz", "dw", "foo |baz");
    check("|a b c", "2dw", "|c");
    check("|a b c d e", "2d2w", "|e");
    // `dw` on a line's last word stops at its end.
    check("|foo\nbar", "dw", "|\nbar");
    check("foo |bar\nbaz", "dw", "foo| \nbaz");
    check("|foo   \n  bar", "dw", "|\n  bar");
    // On an empty line `dw` takes the line.
    check("a\n|\nb", "dw", "a\n|b");
    check("foo ba|r", "dw", "foo b|a");
    check("foo |bar", "db", "|bar");
    check("|foo bar", "de", "| bar");
    check("|a,b,c", "dt,", "|,b,c");
    check("|a,b,c", "df,", "|b,c");
    check("ab|cd", "d0", "|cd");
    check("ab|cd", "d$", "a|b");
    check("ab|cd", "D", "a|b");
    check("|a\nb\nc", "dj", "|c");
    check("a\nb\n|c", "dk", "|a");
    check("|a\nb\nc", "dG", "|");
    check("a\n|b\nc", "dgg", "|c");
    check("x\n|a\nb\n\nc", "d}", "x\n|\nc");
    check("ab|c", "dh", "a|c");
    check("|abc", "dh", "|abc");
}

#[test]
fn delete_lines_and_characters() {
    check("a\n|b\nc", "dd", "a\n|c");
    check("a\n|b", "dd", "|a");
    check("|abc", "dd", "|");
    check("|a\nb\nc\nd", "3dd", "|d");
    check("a\n|b\nc", "5dd", "|a");
    check("a\n  |b\n  c", "dd", "a\n  |c");
    check("|abc", "x", "|bc");
    check("|abc", "2x", "|c");
    check("ab|c", "x", "a|b");
    check("a|bc", "5x", "|a");
    check("ab|c", "X", "a|c");
    check("abc|d", "2X", "a|d");
    check("|\nb", "x", "|\nb");
}

#[test]
fn change_with_motions() {
    // `cw` on a word changes to its end, like `ce`.
    check("|foo bar", "cwx<Esc>", "|x bar");
    check("|foo bar", "cex<Esc>", "|x bar");
    check("|a b", "cwX<Esc>", "|X b");
    check("|foo bar baz", "c2wx<Esc>", "|x baz");
    check("foo|  bar", "cwx<Esc>", "foo|xbar");
    check("ab|cd", "Cx<Esc>", "ab|x");
    check("  |ab\ncd", "ccx<Esc>", "  |x\ncd");
    check("  |ab\ncd", "Sx<Esc>", "  |x\ncd");
    check("|ab", "sx<Esc>", "|xb");
    check("|abc", "2sx<Esc>", "|xc");
    check("|a\nb\nc", "cjx<Esc>", "|x\nc");
}

#[test]
fn yank_and_put() {
    check("|a\nb", "yyp", "a\n|a\nb");
    check("|a\nb", "yyjP", "a\n|a\nb");
    check("a\n|b", "yyp", "a\nb\n|b");
    check("|a\nb", "yy2p", "a\n|a\na\nb");
    check("|ab", "xp", "b|a");
    check("|a\nb", "ddp", "b\n|a");
    check("|foo bar", "yw$p", "foo barfoo| ");
    check("|foo bar", "ywP", "foo| foo bar");
    check("|foo bar", "yiwA <Esc>p", "foo bar fo|o");
    check("|", "yyp", "\n|");
    check("|ab", "yl3p", "aaa|ab");
    // A multi-line text goes in with the cursor at its start.
    check("|ab\ncd", "vjy$p", "ab|ab\nc\ncd");
}

#[test]
fn the_register_follows_the_clipboard() {
    let mut buf = Buf::at("|one\ntwo");
    buf.keys("yy");
    assert_eq!(buf.clip.0.as_deref(), Some("one\n"));
    // Copied elsewhere: `p` puts that.
    buf.clip.0 = Some("X".into());
    buf.keys("p");
    assert_eq!(buf.marked(), "o|Xne\ntwo");
    // A delete fills the register but leaves the clipboard; `p` puts the
    // deleted text.
    buf.keys("0x$p");
    assert_eq!(buf.marked(), "Xne|o\ntwo");
    assert_eq!(buf.clip.0.as_deref(), Some("X"));
}

#[test]
fn other_commands() {
    check("|a\n  b", "J", "a| b");
    check("|a\nb\nc", "3J", "a b| c");
    check("|a \nb", "J", "a |b");
    check("|a\n)", "J", "a|)");
    check("|\nb", "J", "|b");
    check("a\n|b", "J", "a\n|b");
    check("|abc", "2~", "AB|c");
    check("|aB", "5~", "A|b");
    check("|abc", "rx", "|xbc");
    check("|abc", "3rx", "xx|x");
    check("|abc", "4rx", "|abc");
    check("a|bc", "r<CR>", "a\n|c");
    check("  |a", "ob<Esc>", "  a\n  |b");
    check("  |a", "Ob<Esc>", "  |b\n  a");
    check("|ab", "Ac<Esc>", "ab|c");
    check("  a|b", "Ix<Esc>", "  |xab");
    check("a|b", "ix<Esc>", "a|xb");
    check("a|b", "ax<Esc>", "ab|x");
    check("|", "3ihi<Esc>", "hihih|i");
    check("|a", "2ox<Esc>", "a\nx\n|x");
    check("|ab", "i<Esc>", "|ab");
}

#[test]
fn text_objects() {
    check("foo b|ar baz", "diw", "foo | baz");
    check("foo b|ar baz", "daw", "foo |baz");
    check("foo b|ar", "daw", "fo|o");
    check("foo.b|ar", "diW", "|");
    check("foo b|ar baz", "ciwx<Esc>", "foo |x baz");
    check("f(a|, b)", "di(", "f(|)");
    check("f(a|, b)", "da(", "|f");
    check("f(a|, b)", "dib", "f(|)");
    check("f((a|))", "d2i(", "f(|)");
    check("x [a|] y", "ci[z<Esc>", "x [|z] y");
    check("fn {\n    |a;\n}", "di{", "fn {\n|}");
    check("say \"h|i there\" ok", "ci\"x<Esc>", "say \"|x\" ok");
    check("say \"h|i\" ok", "da\"", "say |ok");
    check("|say 'hi'", "di'", "say '|'");
    check("|a\nb\n\nc", "dip", "|\nc");
    check("|a\nb\n\nc", "dap", "|c");
    check("a\n\n|b", "dap", "|a");
    check("f(a|", "di(", "f(|a");
}

#[test]
fn repeat_with_dot() {
    check("|a b c d", "dw..", "|d");
    check("|a b c d e", "dw2.", "|d e");
    check("|foo foo foo", "cwx<Esc>w.", "x |x foo");
    check("|a\nb\nc", "A;<Esc>j.j.", "a;\nb;\nc|;");
    check("|a\nb", "ddu.", "|b");
    check("|abc", "rx.", "|xbc");
    check("|aaa", "~.", "AA|a");
    check("|a\nb", "ox<Esc>.", "a\nx\n|x\nb");
    check("|ab ab", "ciwX<Esc>ww.", "X |X");
    // Yanks and motions aren't changes.
    check("|a b c", "xyyw.", " | c");
}

#[test]
fn repeat_visual_changes_with_dot() {
    // The same size from the cursor: so many characters on one line...
    check("|abcdef", "vldl.", "c|f");
    check("|abcd\nefgh", "vlrxj.", "xxcd\n|xxgh");
    check("|ab ab", "v~w.", "Ab |Ab");
    check("|abc abc", "vlcz<Esc>w.", "zc |zc");
    // ...so many lines...
    check("|a\nb\nc\nd", "Vjd.", "|");
    check("|a\nb\nc", "V>j.", "  a\n  |b\nc");
    check("|a\nb\nc\nd", "VjJj.", "a b\nc| d");
    // ...or up to the same column on the last line.
    check("|abc\ndef\nghi", "vjd.", "|hi");
    // Counts and nothing to repeat.
    check("|a\nb", "V3>j.", "      a\n      |b");
    check("|abc", "vly.", "|abc");
}

#[test]
fn shifting_lines() {
    check("|a\nb", ">>", "  |a\nb");
    check("|a\nb\nc", "2>>", "  |a\n  b\nc");
    check("|a\nb\nc", ">j", "  |a\n  b\nc");
    check("a\n|b", ">k", "  |a\n  b");
    check("|a\n\nb", ">2j", "  |a\n\n  b");
    check("    |a", "<<", "  |a");
    check(" |a", "<<", "|a");
    check("\t|a", "<<", "|a");
    check("|a", "<<", "|a");
    check("|a\nb", ">>j.", "  a\n  |b");
    check("|a\nb\nc", "Vj>", "  |a\n  b\nc");
    check("|a\nb", "Vj2>", "    |a\n    b");
    check("    |a\n  b", "Vj<", "  |a\nb");
    check("|a b", ">>u", "|a b");
    // Counted with a motion; the cursor on the first line's first character.
    check("x|y\nz", ">ap", "  |xy\n  z");
}

#[test]
fn moving_back_to_word_ends() {
    check("abc de|f", "ge", "ab|c def");
    check("abc d|ef", "ge", "ab|c def");
    check("abc |def", "ge", "ab|c def");
    check("abc def g|h", "2ge", "ab|c def gh");
    check("a.b c|d", "ge", "a.|b cd");
    check("a.b c|d", "gE", "a.|b cd");
    check("ab.c |d", "gE", "ab.|c d");
    check("ab\n|cd", "ge", "a|b\ncd");
    check("ab\n\n|cd", "ge", "ab\n|\ncd");
    check("|ab", "ge", "|ab");
    // Inclusive with an operator: the end of the word before goes too.
    check("abc de|f", "dge", "a|b");
    check("abc de|f", "vged", "a|b");
}

#[test]
fn moving_to_lines_in_view() {
    let text = "a\n b\nc\nd\ne\nf\ng";
    let mut buf = Buf::new(text, 0);
    buf.vim.set_view(Some((1, 5)));
    buf.keys("H");
    assert_eq!(buf.marked(), "a\n |b\nc\nd\ne\nf\ng");
    buf.keys("L");
    assert_eq!(buf.marked(), "a\n b\nc\nd\ne\n|f\ng");
    buf.keys("M");
    assert_eq!(buf.marked(), "a\n b\nc\n|d\ne\nf\ng");
    buf.keys("2H");
    assert_eq!(buf.marked(), "a\n b\n|c\nd\ne\nf\ng");
    buf.keys("3L");
    assert_eq!(buf.marked(), "a\n b\nc\n|d\ne\nf\ng");
    buf.keys("9H");
    assert_eq!(buf.marked(), "a\n b\nc\nd\ne\n|f\ng");
    // Linewise with an operator.
    buf.keys("dH");
    assert_eq!(buf.marked(), "a\n|g");
    // Without a view, the whole text.
    check("a\n|b\nc", "L", "a\nb\n|c");
    check("a\n|b\nc", "H", "|a\nb\nc");
    check("a\nb\n|c", "dM", "|a");
}

#[test]
fn undo_and_redo() {
    check("|a b", "dwu", "|a b");
    check("|a b", "dwu<C-r>", "|b");
    check("|ab", "xxuu", "|ab");
    check("|ab", "xx2u", "|ab");
    check("|foo", "cwbar<Esc>u", "|foo");
    check("|a", "ox<CR>y<Esc>u", "|a");
}

#[test]
fn visual_modes() {
    check("|abc def", "vld", "|c def");
    check("|abc def", "vey", "|abc def");
    check("|a\nb\nc", "Vjd", "|c");
    check("a\n|b\nc", "Vd", "a\n|c");
    check("|abc", "vlc<Esc>", "|c");
    check("|abc", "vlcx<Esc>", "|xc");
    check("|abc", "v$~", "|ABC");
    check("|abc", "vlU", "|ABc");
    check("|ABC", "vlu", "|abC");
    check("foo b|ar baz", "viwd", "foo | baz");
    check("|abc", "vlrx", "|xxc");
    check("|a\nb\nc", "VjJ", "a| b\nc");
    check("|ab cd", "yiwwviwp", "ab |ab");
    check("|ab\ncd", "yyjVp", "ab\n|ab");
    check("ab|c", "vhho<Esc>", "ab|c");
    check("ab|c", "vhhod", "|");
    check("|abc", "vl<Esc>", "a|bc");
    check("|a\nb", "vjVd", "|");
    check("|a\nb", "Vv<Esc>", "|a\nb");
}

#[test]
fn ex_commands() {
    let mut buf = Buf::at("|a\nb\nc");
    buf.keys(":w<CR>");
    assert_eq!(buf.effects, vec![Effect::Save]);
    buf.effects.clear();
    buf.keys(":q<CR>");
    assert_eq!(buf.effects, vec![Effect::Close]);
    buf.effects.clear();
    buf.keys(":wq<CR>");
    assert_eq!(buf.effects, vec![Effect::Save, Effect::Close]);
    buf.effects.clear();
    buf.keys(":set nu<CR>");
    assert!(buf.effects.is_empty());
    buf.keys(":3<CR>");
    assert_eq!(buf.marked(), "a\nb\n|c");
    buf.keys(":x<BS><BS>:abc<Esc>j");
    assert!(buf.effects.is_empty());
    let mut buf = Buf::at("|a");
    buf.keys(":w");
    assert_eq!(buf.vim.status().pending, ":w");
}

#[test]
fn escape_and_the_footer() {
    let mut buf = Buf::at("|a");
    buf.keys("<Esc>");
    assert!(!buf.consumed, "nothing pending: Escape is the app's");
    buf.keys("2d");
    assert_eq!(
        buf.vim.status(),
        Status {
            mode: Mode::Normal,
            pending: "2d".into()
        }
    );
    buf.keys("<Esc>");
    assert!(buf.consumed);
    assert_eq!(buf.vim.status().pending, "");
    buf.keys("v");
    assert_eq!(buf.vim.status().mode.label(), "VISUAL");
    buf.keys("V");
    assert_eq!(buf.vim.status().mode.label(), "V-LINE");
    buf.keys("<Esc>i");
    assert_eq!(buf.vim.status().mode.label(), "INSERT");
}

#[test]
fn visual_selection_is_shown() {
    let mut vim = Vim::new();
    let mut clip = FakeClip::default();
    let out = vim.handle_key(Key::Char('v'), "abc\ndef", 1, &mut clip);
    assert_eq!(out.selection, Some(1..2));
    let out = vim.handle_key(Key::Char('j'), "abc\ndef", 1, &mut clip);
    assert_eq!((out.cursor, out.selection), (5, Some(1..6)));
    let out = vim.handle_key(Key::Char('V'), "abc\ndef", 1, &mut clip);
    assert_eq!(out.selection, Some(0..7));
}

#[test]
fn diff_finds_the_one_change() {
    assert_eq!(diff("abc", "abc"), None);
    assert_eq!(diff("abc", "aXc"), Some((1..2, "X".into())));
    assert_eq!(diff("aé", "aè"), Some((1..3, "è".into())));
    assert_eq!(diff("", "x"), Some((0..0, "x".into())));
}

// ---- Properties ----

/// Texts with the awkward parts: blank and empty lines, multibyte
/// letters, emoji (one with joiners), a combining accent, brackets and
/// quotes.
pub(super) fn gen_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "a",
            "b",
            "c",
            "_",
            " ",
            " ",
            "\n",
            "\n",
            "é",
            "日",
            "👍",
            "👨‍👩‍👧",
            "e\u{301}",
            "(",
            ")",
            "\"",
            ".",
            "\t",
        ]),
        0..40,
    )
    .prop_map(|parts| parts.concat())
}

pub(super) fn gen_key() -> impl Strategy<Value = Key> {
    prop_oneof![
        8 => prop::sample::select(
            "hjklwbeWBE0^$G+-;,{}%fFtTgdcyxXDCsSrJ~pPoOaAiIuvVY.:123qz\"'()pw ab"
                .chars()
                .collect::<Vec<_>>()
        )
        .prop_map(Key::Char),
        1 => prop::sample::select(vec![
            Key::Esc,
            Key::Esc,
            Key::Enter,
            Key::Backspace,
            Key::Redo,
            Key::Tab,
            Key::Left,
            Key::Up,
            Key::Char('é'),
            Key::Char('👍'),
        ]),
    ]
}

/// Changes `.` and `u` are checked with: each one complete, back in
/// Normal mode.
fn gen_change() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "dw",
        "dd",
        "x",
        "X",
        "D",
        "J",
        "~",
        "rz",
        "p",
        "P",
        "cwxy<Esc>",
        "ciwq<Esc>",
        "ccnew<Esc>",
        "Snew<Esc>",
        "ox<Esc>",
        "Oy<Esc>",
        "Az<Esc>",
        "Iw<Esc>",
        "ab<Esc>",
        "ic<Esc>",
        "3x",
        "2dd",
        "d2w",
        "de",
        "db",
        "d$",
        "dj",
        "dk",
        "dip",
        "dap",
        "di(",
        "da\"",
        "s!<Esc>",
        "C.<Esc>",
        "2ihi<Esc>",
        "dt ",
        "df.",
        "dG",
        "dgg",
        "d}",
        "c$x<Esc>",
        "3~",
        "2p",
        ">>",
        "<<",
        "2>>",
        ">j",
        "<k",
        "dge",
        "vld",
        "vlrx",
        "v~",
        "vlcz<Esc>",
        "Vd",
        "Vjd",
        "Vj>",
        "V<",
        "VjJ",
    ])
}

/// A cursor somewhere in `text`, as an offset to clamp.
pub(super) fn gen_case() -> impl Strategy<Value = (String, usize)> {
    gen_text().prop_flat_map(|text| {
        let len = text.len();
        (Just(text), 0..=len)
    })
}

/// The cursor is on a character of the text (or the start of an empty
/// line) in Normal mode, on a char boundary in Insert mode, and a Visual
/// selection lies in the text.
fn assert_invariants(buf: &Buf) -> Result<(), TestCaseError> {
    prop_assert!(buf.cursor <= buf.text.len());
    prop_assert!(buf.text.is_char_boundary(buf.cursor));
    if buf.vim.mode() == Mode::Normal {
        prop_assert_eq!(
            buf.cursor,
            clamp_normal(&buf.text, buf.cursor),
            "{:?}",
            buf.marked()
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(config())]

    /// No key sequence panics, and the cursor stays where it may be.
    #[test]
    fn prop_any_keys_keep_the_cursor_in_the_text(
        (text, cursor) in gen_case(),
        keys in prop::collection::vec(gen_key(), 0..60),
    ) {
        let mut buf = Buf::new(&text, cursor);
        for key in keys {
            buf.key(key);
            assert_invariants(&buf)?;
            if buf.vim.mode().is_visual() {
                let out = buf.vim.handle_key(Key::Tab, &buf.text.clone(), buf.cursor, &mut FakeClip::default());
                let shown = out.selection.unwrap();
                prop_assert!(shown.start <= shown.end && shown.end <= buf.text.len());
                prop_assert!(buf.text.is_char_boundary(shown.start) && buf.text.is_char_boundary(shown.end));
            }
        }
    }

    /// `u` after any change gives the text back as it was before it.
    #[test]
    fn prop_undo_restores_the_text(
        (text, cursor) in gen_case(),
        before in prop::collection::vec(gen_change(), 0..3),
        change in gen_change(),
    ) {
        let mut buf = Buf::new(&text, cursor);
        buf.clip.0 = Some("clip".into());
        for keys in before {
            buf.keys(keys);
        }
        let was = buf.text.clone();
        buf.keys(change);
        prop_assert_eq!(buf.vim.mode(), Mode::Normal);
        if buf.text == was {
            // Nothing changed (`x` on an empty line, a word typed over
            // itself): there is no step to undo.
            return Ok(());
        }
        buf.keys("u");
        prop_assert_eq!(&buf.text, &was, "{} then u", change);
        assert_invariants(&buf)?;
    }

    /// `.` does what typing the change again does.
    #[test]
    fn prop_dot_repeats_the_change(
        (text, cursor) in gen_case(),
        change in gen_change(),
        moves in prop::sample::select(vec!["", "j", "w", "0", "$", "k", "b"]),
        count in prop::option::of(1usize..4),
    ) {
        let mut buf = Buf::new(&text, cursor);
        buf.clip.0 = Some("clip".into());
        buf.keys(change);
        if buf.text == text {
            // It did nothing here (`df.` with no `.`), so `.` has none of
            // its own to repeat.
            return Ok(());
        }
        buf.keys(moves);
        let mut again = Buf {
            text: buf.text.clone(),
            cursor: buf.cursor,
            vim: Vim::new(),
            clip: FakeClip(buf.clip.0.clone()),
            undo: Vec::new(),
            redo: Vec::new(),
            base: None,
            entry: None,
            effects: Vec::new(),
            consumed: true,
        };
        // The register as the first change left it.
        again.vim.register = buf.vim.register.clone();
        again.vim.clip_seen = buf.vim.clip_seen.clone();
        // A count before `.` replaces the change's own, so it is only
        // given to changes typed without one, and not to Visual mode ones
        // (where a count before `v` isn't the change's).
        let count = count.filter(|_| {
            !change.contains(|c: char| c.is_ascii_digit()) && !change.starts_with(['v', 'V'])
        });
        let typed = match count {
            Some(n) => format!("{n}{change}"),
            None => change.to_owned(),
        };
        let dot = match count {
            Some(n) => format!("{n}."),
            None => ".".to_owned(),
        };
        again.keys(&typed);
        let keys = |vim: &Vim| {
            vim.last_change
                .as_ref()
                .map(|change| (change.keys.clone(), change.visual))
        };
        if keys(&again.vim) != keys(&buf.vim) {
            // Typed again it failed here (`2c$` on the last line), and the
            // keys after it did something else; or its selection is
            // another size here (`vl` at a line's end), where `.` takes
            // the first one's size, as in Vim.
            return Ok(());
        }
        buf.keys(&dot);
        prop_assert_eq!(buf.marked(), again.marked(), "{} then {}", change, dot);
    }

    /// `yy` then `p` puts a copy of the line below it; `P` above.
    #[test]
    fn prop_yank_line_then_put_round_trips((text, cursor) in gen_case(), above in any::<bool>()) {
        let mut buf = Buf::new(&text, cursor);
        let start = line_start(&text, buf.cursor);
        let end = line_end(&text, buf.cursor);
        let line = text[start..end].to_owned();
        buf.keys(if above { "yyP" } else { "yyp" });
        let mut lines: Vec<&str> = text.split('\n').collect();
        let row = text[..start].matches('\n').count();
        lines.insert(row, &line);
        prop_assert_eq!(&buf.text, &lines.join("\n"));
        let new_row = if above { row } else { row + 1 };
        prop_assert_eq!(buf.text[..buf.cursor].matches('\n').count(), new_row);
        buf.keys("u");
        prop_assert_eq!(&buf.text, &text);
    }
}

// ---- The reference model ----

/// A deliberately plain Vim for a few commands: lines of characters, a row
/// and a column, and one register.
#[derive(Clone, Debug)]
struct Model {
    lines: Vec<Vec<char>>,
    row: usize,
    col: usize,
    /// The column `j` and `k` aim for; `usize::MAX` after `$`.
    want: usize,
    reg: Option<(Vec<Vec<char>>, bool)>,
}

impl Model {
    fn new(text: &str, cursor: usize) -> Model {
        let lines: Vec<Vec<char>> = text.split('\n').map(|l| l.chars().collect()).collect();
        let row = text[..cursor].matches('\n').count();
        let start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
        let col = text[start..cursor].chars().count();
        let mut model = Model {
            lines,
            row,
            col,
            want: col,
            reg: None,
        };
        model.clamp();
        model.want = model.col;
        model
    }

    fn len(&self) -> usize {
        self.lines[self.row].len()
    }

    fn clamp(&mut self) {
        self.row = self.row.min(self.lines.len() - 1);
        self.col = self.col.min(self.len().saturating_sub(1));
    }

    fn first_non_blank(&mut self) {
        let line = &self.lines[self.row];
        self.col = line
            .iter()
            .position(|c| *c != ' ')
            .unwrap_or(line.len().saturating_sub(1));
        self.want = self.col;
    }

    fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cursor(&self) -> usize {
        let mut at = 0;
        for line in &self.lines[..self.row] {
            at += line.iter().map(|c| c.len_utf8()).sum::<usize>() + 1;
        }
        at + self.lines[self.row][..self.col]
            .iter()
            .map(|c| c.len_utf8())
            .sum::<usize>()
    }

    fn vertical(&mut self, down: bool) -> bool {
        if down && self.row + 1 < self.lines.len() {
            self.row += 1;
        } else if !down && self.row > 0 {
            self.row -= 1;
        } else {
            return false;
        }
        self.col = self.want.min(self.len().saturating_sub(1));
        true
    }

    fn delete_rows(&mut self, first: usize, last: usize) {
        let taken: Vec<Vec<char>> = self.lines.drain(first..=last).collect();
        self.reg = Some((taken, true));
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        self.row = first.min(self.lines.len() - 1);
        self.first_non_blank();
    }

    fn command(&mut self, command: &str) {
        match command {
            "h" => {
                self.col = self.col.saturating_sub(1);
                self.want = self.col;
            }
            "l" => {
                if self.col + 1 < self.len() {
                    self.col += 1;
                }
                self.want = self.col;
            }
            "j" => {
                self.vertical(true);
            }
            "k" => {
                self.vertical(false);
            }
            "0" => {
                self.col = 0;
                self.want = 0;
            }
            "$" => {
                self.col = self.len().saturating_sub(1);
                self.want = usize::MAX;
            }
            "x" | "dl" => {
                if self.len() > 0 {
                    let c = self.lines[self.row].remove(self.col);
                    self.reg = Some((vec![vec![c]], false));
                    self.clamp();
                }
                self.want = self.col;
            }
            "dh" => {
                if self.col > 0 {
                    let c = self.lines[self.row].remove(self.col - 1);
                    self.reg = Some((vec![vec![c]], false));
                    self.col -= 1;
                }
                self.want = self.col;
            }
            "d$" => {
                if self.len() > 0 {
                    let taken = self.lines[self.row].split_off(self.col);
                    self.reg = Some((vec![taken], false));
                    self.clamp();
                }
                self.want = self.col;
            }
            "dd" => self.delete_rows(self.row, self.row),
            "dj" => {
                if self.row + 1 < self.lines.len() {
                    self.delete_rows(self.row, self.row + 1);
                }
            }
            "dk" => {
                if self.row > 0 {
                    self.delete_rows(self.row - 1, self.row);
                }
            }
            // Every operator sets the column `j` and `k` keep, as in Vim.
            "yy" => {
                self.reg = Some((vec![self.lines[self.row].clone()], true));
                self.want = self.col;
            }
            "y$" => {
                if self.len() > 0 {
                    self.reg = Some((vec![self.lines[self.row][self.col..].to_vec()], false));
                }
                self.want = self.col;
            }
            "p" | "P" => {
                let Some((reg, linewise)) = self.reg.clone() else {
                    return;
                };
                if linewise {
                    let at = if command == "p" {
                        self.row + 1
                    } else {
                        self.row
                    };
                    for (i, line) in reg.into_iter().enumerate() {
                        self.lines.insert(at + i, line);
                    }
                    self.row = at;
                    self.first_non_blank();
                } else {
                    let piece = reg[0].clone();
                    let at = if command == "p" && self.len() > 0 {
                        self.col + 1
                    } else {
                        self.col
                    };
                    for (i, c) in piece.iter().enumerate() {
                        self.lines[self.row].insert(at + i, *c);
                    }
                    self.col = at + piece.len() - 1;
                    self.want = self.col;
                }
            }
            _ => unreachable!(),
        }
    }
}

fn gen_model_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec!["a", "b", " ", "\n", "é", "\n"]),
        0..30,
    )
    .prop_map(|parts| parts.concat())
}

proptest! {
    #![proptest_config(config())]

    /// Motions with `d` and `y`, step for step against the plain model.
    #[test]
    fn prop_matches_the_reference_model(
        text in gen_model_text(),
        at in any::<prop::sample::Index>(),
        commands in prop::collection::vec(
            prop::sample::select(vec![
                "h", "j", "k", "l", "0", "$", "x", "dl", "dh", "d$", "dd", "dj", "dk", "yy",
                "y$", "p", "P",
            ]),
            0..25,
        ),
    ) {
        let cursor = if text.is_empty() { 0 } else { at.index(text.len() + 1) };
        let cursor = floor_char(&text, cursor);
        let mut buf = Buf::new(&text, cursor);
        let mut model = Model::new(&text, buf.cursor);
        for command in commands {
            buf.keys(command);
            model.command(command);
            prop_assert_eq!(&buf.text, &model.text(), "after {}", command);
            prop_assert_eq!(buf.cursor, model.cursor(), "after {} in {:?}", command, buf.marked());
        }
    }
}
