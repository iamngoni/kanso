//! Vim keys in the note's source: the editor's side of [`crate::vim`].
//!
//! With the setting on, a keystroke interceptor sees each key before the
//! text editor and the app's shortcuts do. In Normal and Visual modes Vim
//! takes the keys (⌘ shortcuts and the app's Ctrl ones aside); in Insert
//! mode only Escape, so typing, input methods, pairing, list continuation
//! and completions are the editor's own. With the setting off there is no
//! interceptor at all.
//!
//! Each Normal mode change is one edit, so one undo step. An Insert mode
//! session (with the command that began it, as `cw` or `o`) is made one
//! step when Escape ends it: the session begins with a mark in the undo
//! history (an invisible character put in and taken out), and Escape
//! undoes back to that mark and makes the whole change again as one edit.
//! If the mark can't be found (it was undone past), the history is put back
//! as it was.
//!
//! Normal mode shows a block caret over the character, in place of the
//! editor's own (which it paints, for that frame, in a colour that can't be
//! seen: the editor has no setting to hide it); Visual modes select the
//! text as the editor selects it.

use std::ops::Range;
use std::rc::Rc;

use gpui_kit::base::input::{DiagnosticColors, Escape, InputEditorStyle, Redo, RopeExt as _, Undo};
use gpui_kit::component::button::{Button, ButtonRounded, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, IconName, Selectable as _, Sizable as _};
use gpui_kit::*;

use super::{NoteEditor, apply_edit};
use crate::markdown_edit::Edit;
use crate::vim::{
    Clipboard, Effect, Input, Key, Keyed, Left, Mode, Route, Session, Step, clamp_normal,
};
use crate::zoom::px;

/// What marks the start of an Insert mode session in the undo history: a
/// zero-width space, put in and taken out at once.
const MARK: &str = "\u{200B}";

pub(super) struct VimState {
    session: Session,
    _intercept: Subscription,
}

impl VimState {
    /// One more edit to the text while Insert mode lasts.
    pub(super) fn count_edit(&mut self) {
        self.session.count_edit();
    }
}

/// The system clipboard's text, read through the app.
struct AppClipboard<'a>(&'a mut App);

impl Clipboard for AppClipboard<'_> {
    fn read(&mut self) -> Option<String> {
        self.0.read_from_clipboard().and_then(|item| item.text())
    }
}

/// The key Vim takes for a keystroke; `None` for ⌘ shortcuts, the app's
/// Ctrl ones and keys Vim leaves alone, which go on as usual.
// no-prop-test: a table of key names; the window tests type through it.
fn vim_key(keystroke: &Keystroke) -> Option<Key> {
    let modifiers = keystroke.modifiers;
    if modifiers.platform || modifiers.function {
        return None;
    }
    if modifiers.control {
        return match keystroke.key.as_str() {
            "r" => Some(Key::Redo),
            "[" => Some(Key::Esc),
            _ => None,
        };
    }
    Some(match keystroke.key.as_str() {
        "escape" => Key::Esc,
        "enter" => Key::Enter,
        "backspace" => Key::Backspace,
        "tab" => Key::Tab,
        "left" => Key::Left,
        "right" => Key::Right,
        "up" => Key::Up,
        "down" => Key::Down,
        "home" => Key::Home,
        "end" => Key::End,
        _ => {
            let typed = keystroke.key_char.as_deref()?;
            let mut chars = typed.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            Key::Char(ch)
        }
    })
}

impl NoteEditor {
    /// Whether Vim is on and not in Insert mode, so the keys are Vim's.
    pub(super) fn vim_holds_keys(&self) -> bool {
        self.vim
            .as_ref()
            .is_some_and(|state| state.session.holds_keys())
    }

    /// Turns Vim keys on (in Normal mode) or off.
    // no-prop-test: sets up or drops the interceptor; the window tests
    // turn it on and off.
    pub(super) fn set_vim(&mut self, on: bool, cx: &mut Context<Self>) {
        if on == self.vim.is_some() {
            return;
        }
        self.vim = on.then(|| {
            let editor = cx.entity().downgrade();
            let intercept = cx.intercept_keystrokes(move |event, window, cx| {
                if let Some(editor) = editor.upgrade() {
                    editor.update(cx, |editor, cx| {
                        editor.vim_keystroke(&event.keystroke, window, cx)
                    });
                }
            });
            VimState {
                session: Session::new(),
                _intercept: intercept,
            }
        });
        self.vim_footer(cx);
        cx.notify();
    }

    /// Back to Normal mode, as when another note opens.
    pub(super) fn vim_reset(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = self.vim.as_mut() {
            state.session.reset();
            self.vim_footer(cx);
        }
    }

    /// The text was set anew (and its undo history with it): an Insert
    /// session going on can't be made one step any more.
    pub(super) fn vim_forget_session(&mut self) {
        if let Some(state) = self.vim.as_mut() {
            state.session.forget_session();
        }
    }

    /// Someone else's edits were merged into the text: Vim's cursor and
    /// Visual selection follow their text, a Normal mode cursor stays on a
    /// character, and an Insert session can't be made one undo step any
    /// more (its text changed under it).
    pub(super) fn vim_after_merge(
        &mut self,
        splices: &[kanso_collab::Splice],
        cx: &mut Context<Self>,
    ) {
        let text = self.body.read(cx).value();
        let Some(state) = self.vim.as_mut() else {
            return;
        };
        let normal = state.session.mode() == Mode::Normal;
        let (head, shown) = crate::collab_text::map_vim(
            state.session.head(),
            state.session.shown().cloned(),
            normal,
            splices,
            &text,
        );
        state.session.follow(head, shown);
        if normal {
            let at = clamp_normal(&text, self.body.read(cx).cursor());
            self.body
                .update(cx, |body, cx| body.set_selected_range(at..at, cx));
        }
        self.vim_footer(cx);
    }

    /// The mode in the footer, and in Visual modes where Vim's cursor is
    /// (the editor's is at the selection's end).
    fn vim_footer(&self, cx: &mut Context<Self>) {
        let status = self.vim.as_ref().map(|state| state.session.status());
        let cursor = self.vim.as_ref().and_then(|state| {
            state.session.mode().is_visual().then(|| {
                self.body
                    .read(cx)
                    .text()
                    .offset_to_position(state.session.head())
            })
        });
        self.footer
            .update(cx, |footer, cx| footer.set_vim(status, cursor, cx));
    }

    /// The first and last lines (0-based) wholly in view, for `H`, `M`
    /// and `L`.
    // no-prop-test: reads the editor's layout; `lines_in_view` checks it.
    pub(super) fn vim_view(&self, cx: &App) -> Option<(usize, usize)> {
        let body = self.body.read(cx);
        let rows = body.visible_row_range()?;
        let bounds = body.text_bounds().unwrap_or_else(|| body.input_bounds());
        let text = body.text();
        let shown = |row: &usize| {
            let at = text.line_start_offset(*row);
            body.range_to_bounds(&(at..at)).is_some_and(|line| {
                line.top() >= bounds.top() - px(0.5) && line.bottom() <= bounds.bottom() + px(0.5)
            })
        };
        let first = rows.clone().find(shown)?;
        let last = rows.rev().find(shown)?;
        Some((first, last.max(first)))
    }

    /// A keystroke anywhere in the app: Vim's when the note's text has the
    /// keyboard.
    // no-prop-test: key glue; `vim` has the properties and the window
    // tests type through it.
    fn vim_keystroke(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.vim.as_ref() else {
            return;
        };
        let Some(key) = vim_key(keystroke) else {
            return;
        };
        let route = state.session.route(key);
        // Typing is the editor's own: only Escape leaves Insert mode.
        if route == Route::Typing {
            return;
        }
        // The keyboard elsewhere, or an input method composing: not Vim's.
        if !self.body_takes_edits(window, cx) {
            return;
        }
        if route == Route::Leave {
            cx.stop_propagation();
            self.vim_leave_insert(window, cx);
            return;
        }
        let (text, selection, caret) = {
            let body = self.body.read(cx);
            (body.value(), body.selected_range(), body.cursor())
        };
        let view = self.vim_view(cx);
        let read_only = self.text_read_only;
        let Some(state) = self.vim.as_mut() else {
            return;
        };
        let input = Input {
            text: &text,
            selection,
            caret,
            view,
            read_only,
        };
        match state.session.key(key, &input, &mut AppClipboard(cx)) {
            Keyed::Ignored => {}
            Keyed::Refused => {
                cx.stop_propagation();
                self.vim_footer(cx);
            }
            Keyed::Handled(step) => {
                cx.stop_propagation();
                self.vim_apply(step, &text, window, cx);
            }
        }
    }

    /// Makes what a key did show in the editor.
    // no-prop-test: editor glue; the window tests check each part.
    fn vim_apply(
        &mut self,
        step: Step,
        text: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if step.entering {
            // The mark the session's undo steps are gathered back to.
            let cursor = step.cursor;
            self.body.update(cx, |state, cx| {
                apply_edit(state, text, edit(cursor..cursor, MARK, cursor), window, cx);
                let marked = state.value();
                let end = cursor + MARK.len();
                apply_edit(state, &marked, edit(cursor..end, "", cursor), window, cx);
            });
        }
        let selection = step.selection();
        self.body.update(cx, |state, cx| match step.change() {
            Some(change) => apply_edit(state, text, change, window, cx),
            None => {
                if state.selected_range() != selection {
                    state.set_selected_range(selection.clone(), cx);
                }
            }
        });
        let mut undo = None;
        for effect in &step.outcome.effects {
            match effect {
                Effect::Copy(text) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))
                }
                Effect::Undo(n) => undo = Some((true, *n)),
                Effect::Redo(n) => undo = Some((false, *n)),
                Effect::Save => self.flush(cx),
                Effect::Close => {
                    self.flush(cx);
                    window.dispatch_action(Box::new(crate::workspace::FocusNoteList), cx);
                }
            }
        }
        if let Some((back, n)) = undo {
            self.vim_history(back, n, window, cx);
        }
        self.vim_footer(cx);
        cx.notify();
    }

    /// `u` and Ctrl-R: the editor's own undo and redo, `n` steps, with the
    /// cursor where the change was.
    // no-prop-test: editor glue; `undo_and_redo_through_the_editor` checks it.
    fn vim_history(&mut self, back: bool, n: usize, window: &mut Window, cx: &mut Context<Self>) {
        for _ in 0..n {
            if back {
                self.body_action(&Undo, window, cx);
            } else {
                self.body_action(&Redo, window, cx);
            }
        }
        self.body.update(cx, |state, cx| {
            let text = state.value();
            let at = clamp_normal(&text, state.selected_range().start);
            state.set_selected_range(at..at, cx);
        });
    }

    /// Runs one of the text editor's actions at once.
    fn body_action(&self, action: &dyn Action, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.body.read(cx).focus_handle(cx);
        focus.dispatch_action(action, window, cx);
    }

    /// Escape in Insert mode: a count types the text again, the whole
    /// change becomes one undo step, and the cursor steps back.
    // no-prop-test: editor glue; the window tests undo sessions through it.
    fn vim_leave_insert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // An open completion list closes with the same Escape.
        self.body.update(cx, |state, cx| {
            if state.completion_menu_state().open {
                state.route_overlay_action(Box::new(Escape), window, cx);
            }
        });
        let (now, caret) = {
            let body = self.body.read(cx);
            (body.value(), body.cursor())
        };
        let Some(state) = self.vim.as_mut() else {
            return;
        };
        let left = state.session.leave_insert(&now, caret);
        if !self.vim_collapse(&left, window, cx) {
            self.body.update(cx, |state, cx| {
                apply_edit(state, &now, left.last(), window, cx)
            });
        }
        self.vim_footer(cx);
        cx.notify();
    }

    /// Makes the Insert mode session one undo step: undoes back to its mark
    /// (at most `edits` steps and a few), then makes the change from the
    /// base to `done` as one edit. Whether it could; if not, the history is
    /// put back as it was.
    // no-prop-test: drives the editor's undo; `one_change_is_one_undo_step`
    // checks it, the fallback included.
    fn vim_collapse(&mut self, left: &Left, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let (Some(base), Some(marked), Some(change)) =
            (left.base.as_ref(), left.marked(MARK), left.whole())
        else {
            return false;
        };
        let base = base.text.as_str();
        let is = |this: &Self, text: &str, cx: &App| {
            let body = this.body.read(cx);
            body.text().len() == text.len() && body.value() == text
        };
        let mut undone = 0;
        let mut found = false;
        while undone < left.edits + 4 {
            self.body_action(&Undo, window, cx);
            undone += 1;
            if is(self, &marked, cx) {
                found = true;
                break;
            }
        }
        if found {
            // The mark going in, the step before.
            self.body_action(&Undo, window, cx);
            undone += 1;
            found = is(self, base, cx);
        }
        if !found {
            for _ in 0..undone {
                self.body_action(&Redo, window, cx);
            }
            return false;
        }
        self.body
            .update(cx, |state, cx| apply_edit(state, base, change, window, cx));
        true
    }

    /// The block caret over the character under Normal (and Visual) mode's
    /// cursor, drawn over the text editor once it has laid out.
    // no-prop-test: painting; the Vim snapshots show it.
    pub(super) fn vim_caret(&self, window: &Window, cx: &App) -> Option<AnyElement> {
        let state = self.vim.as_ref()?;
        let mode = state.session.mode();
        if mode == Mode::Insert {
            return None;
        }
        let body = self.body.read(cx);
        if !body.focus_handle(cx).is_focused(window) {
            return None;
        }
        let at = if mode.is_visual() {
            state.session.head()
        } else {
            body.cursor()
        };
        let text = body.text();
        let at = at.min(text.len());
        // The character under the cursor, read from a little of the text.
        let mut end = (at + 32).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let piece = text.slice(at..end).to_string();
        let width_of = crate::vim::grapheme_at(&piece);
        let range = at..at + width_of;
        let body = self.body.clone();
        let styled = self.body.clone();
        let mut color = cx.theme().caret;
        color.a = 0.4;
        let fallback = px(self.font_size * 0.6);
        Some(
            canvas(
                // After the editor has laid out and before it paints.
                move |_, _, cx| {
                    let style = body_style(cx);
                    styled.update(cx, |state, _| state.set_editor_style(style));
                },
                move |bounds, _, window, cx| {
                    let state = body.read(cx);
                    let Some(start) = state.range_to_bounds(&(range.start..range.start)) else {
                        return;
                    };
                    let width = state
                        .range_to_bounds(&range)
                        .filter(|b| b.origin.y == start.origin.y && b.size.width > px(1.))
                        .map_or(fallback, |b| b.size.width);
                    let line = start.size.height;
                    let height = line * 0.85;
                    let block = Bounds::new(
                        point(start.origin.x, start.origin.y + (line - height) / 2.),
                        size(width, height),
                    );
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        window.paint_quad(fill(block, color));
                    });
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .into_any_element(),
        )
    }
}

/// The text editor's style as its `Input` sets it from the theme on each
/// render, but with a caret that can't be seen (not quite transparent,
/// which would mean "unset" and fall back to the text's colour). Set
/// between the editor's layout and its paint, it holds for that frame; the
/// next render sets the theme's again.
// no-prop-test: theme glue; the Vim snapshots show no line caret beside
// the block.
fn body_style(cx: &App) -> InputEditorStyle {
    let theme = cx.theme();
    let mut caret = theme.caret;
    caret.a = 0.001;
    InputEditorStyle {
        foreground: theme.foreground,
        muted_foreground: theme.muted_foreground,
        // `Theme::editor_background`, which is the crate's own.
        background: theme
            .highlight_theme
            .style
            .editor_background
            .unwrap_or_else(|| theme.input_background()),
        border: theme.border,
        selection: theme.selection,
        caret,
        diagnostics: DiagnosticColors {
            error: theme.highlight_theme.style.status.error(cx),
            warning: theme.highlight_theme.style.status.warning(cx),
            info: theme.highlight_theme.style.status.info(cx),
            hint: theme.highlight_theme.style.status.hint(cx),
        },
        highlight_styles: theme.highlight_theme.clone(),
        editor_invisible: theme.highlight_theme.style.editor_invisible,
        editor_active_line: theme.highlight_theme.style.editor_active_line,
        editor_gutter_background: theme.highlight_theme.style.editor_gutter_background,
        fold_icon_renderer: Some(Rc::new(|ix, is_folded| {
            Button::new(("fold-icon", ix))
                .ghost()
                .icon(if is_folded {
                    IconName::ChevronRight
                } else {
                    IconName::ChevronDown
                })
                .xsmall()
                .rounded(ButtonRounded::Small)
                .size(px(14.))
                .selected(is_folded)
                .into_any_element()
        })),
    }
}

/// An edit of `range` to `text` with the cursor at `cursor` after it.
fn edit(range: Range<usize>, text: &str, cursor: usize) -> Edit {
    Edit {
        range,
        text: text.to_owned(),
        selection: cursor..cursor,
    }
}

#[cfg(test)]
impl NoteEditor {
    /// The Vim mode, with Vim keys on.
    pub fn vim_mode(&self) -> Option<Mode> {
        self.vim.as_ref().map(|state| state.session.mode())
    }
}
