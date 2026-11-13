//! Shared notes in the editor (docs/ACCOUNTS.md › Collaboration): the open
//! note's side of `kanso_collab`.
//!
//! A shared note this account is in is attached to the engine while it is
//! open, and its live room kept connected. Its text reaches the library only
//! through `Engine::sync_editor`, once typing pauses and whenever the room
//! says edits are waiting, never through the editor's own save. The merge
//! says how the editor's text must change; those splices go in without
//! moving the caret, the selection or Vim's cursor. The note's other fields
//! (notebook, tags, status) save as any note's do, with the library's text.
//!
//! Other people's carets show as thin coloured lines with their names, and
//! their selections tinted; comments tint the words they are on. A note
//! shared to view or comment on is read only here.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::base::input::TextDecoration;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::{ActiveTheme as _, Sizable as _};
use gpui_kit::*;
use kanso_collab::engine::rebase;
use kanso_collab::{CommentView, Engine, LiveNotice, Problem, Role, Splice, splices};

use super::NoteEditor;
use crate::collab_model::OpenShare;
use crate::collab_text::{
    comment_ranges, comment_target, map_selection, problem_banner, remote_marks, share_footer,
    share_mark,
};
use crate::theme::{Colors, cursor_colour};
use crate::zoom::px;

/// How long typing pauses before the editor's text is merged.
const SYNC_DELAY: Duration = Duration::from_millis(200);
/// The name label beside a remote caret.
const LABEL_SIZE: f32 = 10.5;
const LABEL_HEIGHT: f32 = 15.;

impl NoteEditor {
    /// Whether the open note's text goes through the engine.
    pub(super) fn collab_attached(&self) -> bool {
        self.shared.as_ref().is_some_and(|s| !s.ended && !s.canvas)
    }

    /// Whether the open note is a shared canvas the engine holds.
    pub(super) fn collab_canvas(&self) -> bool {
        self.shared.as_ref().is_some_and(|s| !s.ended && s.canvas)
    }

    /// The canvas view's read-only state for `note`: as the account and the
    /// trash say, and also for a shared canvas that can't be drawn on here,
    /// because it was shared to read or this is a note's own window (which
    /// isn't in the room, so its drawing could never be saved).
    pub(super) fn canvas_read_only(&self, note: &kanso_core::Note) -> bool {
        self.account_read_only
            || note.is_trashed()
            || share_mark(&note.extra)
                .is_some_and(|(_, role, _)| self.standalone || !role.may_edit())
    }

    /// Whether the open note is a shared canvas shown as text: its body is
    /// the drawing's, so it is read only there.
    pub(super) fn shared_canvas_as_text(&self) -> bool {
        self.canvas_as_text
            && self
                .note
                .as_ref()
                .is_some_and(|n| n.doctype == "canvas" && share_mark(&n.extra).is_some())
    }

    /// Whether the open note was shared with this account to view or
    /// comment on only.
    pub(super) fn collab_read_only(&self) -> bool {
        self.collab_read_only
    }

    /// After a note is shown: attaches to the shared note it is, and leaves
    /// the one before. A shared note open without an engine (signed out)
    /// is still read only for a role that can't edit.
    // no-prop-test: engine and editor glue; `shared_notes_merge_in_the_editor`
    // drives it.
    pub(super) fn collab_follow(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mark = self.note.as_ref().and_then(|n| share_mark(&n.extra));
        // A note's own window doesn't join its room: the engine holds one
        // editor per shared note, the main window's. The note shows read
        // only there, as the library has it.
        if self.standalone {
            self.set_text_read_only(mark.is_some(), cx);
            return;
        }
        if let Some(open) = &self.shared {
            let same_note =
                self.note.as_ref().map(|n| n.id.as_str()) == Some(open.note_id.as_str());
            match &mark {
                Some((id, _, _)) if *id == open.share_id && !open.ended => return,
                // A private copy now: the note stays as it is, saying so.
                None if open.ended && same_note => return,
                _ => {}
            }
        }
        // The same note of a shared notebook, shown again: it stays in the room.
        if let Some(open) = &self.book_note
            && mark.as_ref().is_some_and(|(id, _, _)| *id == open.share_id)
            && self
                .note
                .as_ref()
                .and_then(|n| book_stream(&n.extra))
                .as_deref()
                == Some(open.stream.as_str())
        {
            return;
        }
        self.collab_detach(cx);
        let Some((share_id, role, _)) = mark else {
            self.set_text_read_only(false, cx);
            return;
        };
        // A note of a shared notebook: its text goes through the library
        // (the notebook's sync merges it), and its room shows who else is
        // in it.
        if let Some(stream) = self.note.as_ref().and_then(|n| book_stream(&n.extra)) {
            self.book_follow(share_id, stream, role, cx);
            return;
        }
        let Some(engine) = crate::collab::engine(cx).filter(|e| e.attach(&share_id).is_ok()) else {
            self.set_text_read_only(!role.may_edit(), cx);
            return;
        };
        let role = engine.role(&share_id).unwrap_or(role);
        let canvas = engine.kind(&share_id) == Some(kanso_store::SharedKind::Canvas);
        let (tx, rx) = async_channel::unbounded::<LiveNotice>();
        let live = Engine::go_live(
            &engine,
            &share_id,
            Arc::new(move |notice| {
                tx.try_send(notice).ok();
            }),
        );
        let notices_task = cx.spawn_in(window, async move |this, cx| {
            while let Ok(first) = rx.recv().await {
                let mut batch = vec![first];
                while let Ok(more) = rx.try_recv() {
                    batch.push(more);
                }
                if this
                    .update_in(cx, |this, window, cx| this.on_live(batch, window, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let note_id = self.note.as_ref().map(|n| n.id.clone()).unwrap_or_default();
        self.shared = Some(OpenShare {
            share_id: share_id.clone(),
            note_id,
            role,
            canvas,
            engine: engine.clone(),
            live: Some(live),
            connected: false,
            ended: false,
            synced: self.current_text(cx),
            pending: None,
            caret_sent: None,
            cursors: Vec::new(),
            window: window.window_handle(),
            highlights: None,
            selections: None,
            comments: Vec::new(),
            members: Vec::new(),
            problems: Vec::new(),
            size: None,
            sync_task: None,
            _notices: notices_task,
        });
        self.set_text_read_only(!role.may_edit(), cx);
        if canvas {
            // The canvas view draws on the engine's document from now on.
            let read_only = self.account_read_only
                || !role.may_edit()
                || self.note.as_ref().is_some_and(|n| n.is_trashed());
            self.canvas.update(cx, |view, cx| {
                view.share(engine, &share_id, cx);
                view.set_read_only(read_only, cx);
            });
            self.collab_footer(cx);
            self.refresh_problems(cx);
            return;
        }
        // What other writers put in the library meanwhile comes in now.
        self.collab_sync(window, cx);
        self.refresh_comments(cx);
        self.refresh_cursors(cx);
        self.refresh_problems(cx);
    }

    /// Leaves the shared note: its room closes, and merges go to the
    /// library again without waiting for the editor.
    pub(super) fn collab_detach(&mut self, cx: &mut Context<Self>) {
        self.book_detach(cx);
        let Some(open) = self.shared.take() else {
            return;
        };
        if open.canvas {
            self.canvas.update(cx, |view, cx| view.unshare(cx));
        }
        if !open.ended {
            open.engine.detach(&open.share_id);
        }
        for tints in [open.highlights, open.selections].into_iter().flatten() {
            tints.clear(cx);
        }
        self.set_text_read_only(false, cx);
        self.collab_footer(cx);
        cx.notify();
    }

    /// The account signed out: the open shared note closes its room.
    pub fn collab_signed_out(&mut self, cx: &mut Context<Self>) {
        self.collab_detach(cx);
        let role = self
            .note
            .as_ref()
            .and_then(|n| share_mark(&n.extra))
            .map(|(_, role, _)| role);
        self.set_text_read_only(role.is_some_and(|r| !r.may_edit()), cx);
    }

    /// Makes the title and text read only, or not.
    fn set_text_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        if self.collab_read_only == read_only {
            return;
        }
        self.collab_read_only = read_only;
        self.refresh_text_read_only(cx);
    }

    /// What the footer says about the open note's sharing.
    fn collab_footer(&mut self, cx: &mut Context<Self>) {
        let label = self
            .shared
            .as_ref()
            .map(|open| share_footer(open.role, open.connected, open.ended, open.size));
        self.footer
            .update(cx, |footer, cx| footer.set_share(label, cx));
    }

    /// The text changed here: it is merged once typing pauses.
    pub(super) fn collab_edited(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_mut().filter(|s| !s.ended) else {
            return;
        };
        let handle = open.window;
        let editor = cx.entity().downgrade();
        open.sync_task = Some(cx.spawn(async move |_, cx| {
            cx.background_executor().timer(SYNC_DELAY).await;
            handle
                .update(cx, |_, window, cx| {
                    editor
                        .update(cx, |editor, cx| editor.collab_sync(window, cx))
                        .ok();
                })
                .ok();
        }));
        cx.notify();
    }

    /// Whether a merge is waiting to run, for the header's save state.
    pub(super) fn collab_pending(&self) -> bool {
        self.shared
            .as_ref()
            .is_some_and(|s| s.sync_task.is_some() || s.pending.is_some())
    }

    /// Merges the editor's text with what arrived, now, and puts the result
    /// in the editor.
    pub(super) fn collab_sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // An input method composing: the merge waits for it.
        let composing = self.body.update(cx, |state, cx| {
            state.marked_text_range(window, cx).is_some()
        });
        if composing {
            self.collab_edited(cx);
            return;
        }
        self.collab_apply_pending(window, cx);
        let Some(merged) = self.collab_merge(cx) else {
            return;
        };
        let (sent, merge) = merged;
        self.collab_apply(sent, merge, window, cx);
    }

    /// Merges now where there is no window to hand (a save on quitting, or
    /// before another write): the library gets the merged text at once, and
    /// the editor takes it as soon as its window can.
    pub(super) fn collab_sync_quiet(&mut self, cx: &mut Context<Self>) {
        let Some((sent, merge)) = self.collab_merge(cx) else {
            return;
        };
        if merge.title.is_empty() && merge.body.is_empty() {
            self.collab_settled(merge.title_text, merge.body_text, cx);
            return;
        }
        let Some(open) = self.shared.as_mut() else {
            return;
        };
        let handle = open.window;
        open.pending = Some((sent.0, sent.1, merge));
        let editor = cx.entity().downgrade();
        cx.defer(move |cx| {
            handle
                .update(cx, |_, window, cx| {
                    editor
                        .update(cx, |editor, cx| editor.collab_apply_pending(window, cx))
                        .ok();
                })
                .ok();
        });
    }

    fn collab_apply_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((title, body, merge)) = self.shared.as_mut().and_then(|s| s.pending.take()) else {
            return;
        };
        self.collab_apply((title, body), merge, window, cx);
    }

    /// Hands the engine the editor's text; the text sent and what came back.
    fn collab_merge(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<((String, String), kanso_collab::EditorMerge)> {
        let open = self.shared.as_mut().filter(|s| !s.ended)?;
        open.sync_task = None;
        if open.pending.is_some() {
            // The editor hasn't taken the last merge yet: its text is old.
            return None;
        }
        let (share_id, engine) = (open.share_id.clone(), open.engine.clone());
        let sent = self.current_text(cx);
        let result = engine.sync_editor(&share_id, &sent.0, &sent.1);
        // A refused edit (at the size limit) or one kept private says why.
        self.refresh_problems(cx);
        match result {
            Ok(merge) => Some((sent, merge)),
            Err(err) => {
                eprintln!("kanso: merging the shared note: {err}");
                None
            }
        }
    }

    /// Puts a merge in the editor. Typing since the text was sent is kept,
    /// made again on the merged text, and merged next time.
    // no-prop-test: editor glue; `collab_text` has the properties of the
    // mapping, and `shared_notes_merge_in_the_editor` types through it.
    fn collab_apply(
        &mut self,
        sent: (String, String),
        merge: kanso_collab::EditorMerge,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let now = self.current_text(cx);
        let typed_since = now != sent;
        let (title_edits, body_edits) = if typed_since {
            (
                splices(&now.0, &rebase(&sent.0, &now.0, &merge.title_text)),
                splices(&now.1, &rebase(&sent.1, &now.1, &merge.body_text)),
            )
        } else {
            (merge.title.clone(), merge.body.clone())
        };
        if !title_edits.is_empty() {
            self.title.update(cx, |state, cx| {
                let range = state.selected_range();
                let head = state.cursor();
                let anchor = if head == range.start {
                    range.end
                } else {
                    range.start
                };
                for edit in &title_edits {
                    state.set_selected_range(edit.at..edit.at + edit.delete, cx);
                    state.replace(edit.insert.clone(), window, cx);
                }
                let (anchor, head) = map_selection(anchor, head, &title_edits);
                state.set_selected_range(anchor..head, cx);
            });
        }
        if !body_edits.is_empty() {
            self.splice_body(&body_edits, window, cx);
        }
        self.collab_settled(merge.title_text, merge.body_text, cx);
        if typed_since {
            self.collab_edited(cx);
        }
        self.refresh_cursors(cx);
    }

    /// Puts splices in the body as someone else's edits: the caret, the
    /// selection (backwards or not), Vim's cursor and the scroll position
    /// stay with the text they were at.
    pub(super) fn splice_body(
        &mut self,
        edits: &[Splice],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.body.update(cx, |state, cx| {
            let range = state.selected_range();
            let head = state.cursor();
            let anchor = if head == range.start {
                range.end
            } else {
                range.start
            };
            let scroll = state.scroll_offset();
            for edit in edits {
                state.set_selected_range(edit.at..edit.at + edit.delete, cx);
                state.replace(edit.insert.clone(), window, cx);
            }
            let (anchor, head) = map_selection(anchor, head, edits);
            state.set_selected_range(anchor..head, cx);
            state.set_scroll_offset(scroll, cx);
        });
        // The pairs typed here were at offsets the edits moved.
        self.pairs.clear();
        self.vim_after_merge(edits, cx);
    }

    /// The library has the merged text: the note as the editor knows it
    /// takes it, with its new revision.
    fn collab_settled(&mut self, title: String, body: String, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_mut() else {
            return;
        };
        open.synced = (title.clone(), body.clone());
        if let Ok(Some(latest)) = self.store.note(&open.note_id) {
            self.note = Some(latest);
        } else if let Some(note) = &mut self.note {
            note.title = title;
            note.body = body;
        }
        cx.notify();
    }

    /// The open shared note was written elsewhere: by the engine (a role
    /// changed, sharing ended), the local API, or another field saved.
    pub(super) fn collab_store_change(
        &mut self,
        deleted: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.shared.as_ref().map(|s| s.note_id.clone()) else {
            return;
        };
        if deleted {
            self.collab_detach(cx);
            self.show(None, false, window, cx);
            return;
        }
        let Ok(Some(latest)) = self.store.note(&id) else {
            return;
        };
        match share_mark(&latest.extra) {
            // Not shared any more: a private copy, as the room may not have
            // said yet.
            None => self.collab_ended(window, cx),
            Some((_, role, _)) => {
                let library = (latest.title.clone(), latest.body.clone());
                let old_title = self.note.as_ref().map(|n| n.title.clone());
                let title = latest.title.clone();
                self.note = Some(latest);
                let Some(open) = self.shared.as_mut() else {
                    return;
                };
                let role = open.engine.role(&open.share_id).unwrap_or(role);
                if open.role != role {
                    open.role = role;
                    let is_canvas = open.canvas;
                    self.set_text_read_only(!role.may_edit(), cx);
                    if is_canvas {
                        let read_only = self.account_read_only || !role.may_edit();
                        self.canvas
                            .update(cx, |view, cx| view.set_read_only(read_only, cx));
                    }
                    self.collab_footer(cx);
                }
                // A canvas is merged by drawing: only its title comes from
                // the library (the body is the engine's projection).
                if self.shared.as_ref().is_some_and(|s| s.canvas) {
                    let shown = self.title.read(cx).value().to_string();
                    if old_title.as_deref() == Some(shown.as_str()) && shown != title {
                        self.title
                            .update(cx, |state, cx| state.set_value(title, window, cx));
                    }
                    // Shown as text (read only), it follows the drawing.
                    if self.canvas_as_text {
                        let body = library.1;
                        self.body.update(cx, |state, cx| {
                            if state.value().as_ref() != body {
                                state.set_value(body, window, cx);
                            }
                        });
                    }
                    cx.notify();
                    return;
                }
                // Another writer changed the text: the engine takes it in.
                if self.shared.as_ref().is_some_and(|s| s.synced != library) {
                    self.collab_sync(window, cx);
                }
                cx.notify();
            }
        }
    }

    /// Sharing ended while the note was open: it is a private copy now,
    /// edited and saved as any note. Typing not merged yet stays.
    fn collab_ended(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_mut() else {
            return;
        };
        if open.ended {
            return;
        }
        open.ended = true;
        open.live = None;
        open.sync_task = None;
        open.pending = None;
        open.cursors.clear();
        open.engine.detach(&open.share_id);
        for tints in [open.highlights.take(), open.selections.take()]
            .into_iter()
            .flatten()
        {
            tints.clear(cx);
        }
        let id = open.note_id.clone();
        let was_canvas = open.canvas;
        self.set_text_read_only(false, cx);
        if was_canvas {
            self.canvas.update(cx, |view, cx| view.unshare(cx));
        }
        if let Ok(Some(latest)) = self.store.note(&id) {
            // The canvas goes on as a private one, as it was last written.
            if was_canvas {
                let read_only = self.account_read_only || latest.is_trashed();
                self.canvas_body = Some(latest.body.clone());
                let (id, body) = (latest.id.clone(), latest.body.clone());
                self.canvas
                    .update(cx, |view, cx| view.open(&id, &body, read_only, cx));
            }
            self.note = Some(latest);
        }
        self.collab_footer(cx);
        if self.has_unsaved_edits(cx) {
            self.schedule_autosave(cx);
        }
        cx.notify();
    }

    /// What the live room said.
    fn on_live(&mut self, batch: Vec<LiveNotice>, window: &mut Window, cx: &mut Context<Self>) {
        let canvas = self.shared.as_ref().is_some_and(|s| s.canvas);
        let mut text = false;
        let mut presence = false;
        let mut comments = false;
        let mut problems = false;
        for notice in batch {
            let Some(open) = self.shared.as_mut().filter(|s| !s.ended) else {
                return;
            };
            match notice {
                LiveNotice::Text { share_id } => text |= share_id == open.share_id,
                LiveNotice::Presence { share_id } => presence |= share_id == open.share_id,
                LiveNotice::Comments { share_id } => comments |= share_id == open.share_id,
                LiveNotice::Problem { share_id } => problems |= share_id == open.share_id,
                LiveNotice::Connection { share_id, live } if share_id == open.share_id => {
                    open.connected = live;
                    self.collab_footer(cx);
                }
                LiveNotice::Ended { share_id } if share_id == open.share_id => {
                    self.collab_ended(window, cx);
                    return;
                }
                _ => {}
            }
        }
        if text && canvas {
            self.canvas.update(cx, |view, cx| view.sync_shared(cx));
        } else if text {
            self.collab_sync(window, cx);
        }
        if presence && canvas {
            self.canvas.update(cx, |view, cx| view.refresh_pointers(cx));
        } else if presence {
            self.refresh_cursors(cx);
        }
        if comments {
            self.refresh_comments(cx);
        }
        if problems {
            self.refresh_problems(cx);
        }
    }

    /// Reads what the engine couldn't do for the open note, and how much of
    /// its size limit it uses; the banner and the footer follow. A note
    /// that became a private copy keeps its problems.
    pub fn refresh_problems(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_mut() else {
            return;
        };
        let problems: Vec<Problem> = open
            .engine
            .problems()
            .into_iter()
            .filter(|p| p.share_id == open.share_id)
            .collect();
        let size = if open.ended {
            None
        } else {
            open.engine.size_use(&open.share_id)
        };
        if open.problems != problems || open.size != size {
            open.problems = problems;
            open.size = size;
            cx.notify();
        }
        self.collab_footer(cx);
    }

    /// Clears the open note's problems, as its banner's Dismiss does.
    pub fn dismiss_problems(&mut self, cx: &mut Context<Self>) {
        if let Some(open) = self.shared.as_ref() {
            open.engine.dismiss_problems(Some(&open.share_id));
        }
        self.refresh_problems(cx);
    }

    /// The open shared note's problems, for the banner above its text.
    pub fn shared_problems(&self) -> &[Problem] {
        self.shared.as_ref().map_or(&[], |s| s.problems.as_slice())
    }

    /// The banner telling the open shared note's problems, with Dismiss.
    // no-prop-test: drawing; `problem_banner` has the properties and
    // `a_refused_edit_shows_its_problem` drives it.
    pub(super) fn render_problems(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let text = problem_banner(self.shared_problems())?;
        let colors = Colors::get(cx);
        Some(
            gpui_kit::component::h_flex()
                .debug_selector(|| "shared-problems".into())
                .flex_none()
                .gap_2()
                .mx(px(super::TEXT_INSET - 12.))
                .my_1()
                .px(px(12.))
                .py(px(8.))
                .rounded(px(6.))
                .bg(Hsla::from(colors.warning).opacity(0.14))
                .text_size(px(13.))
                .text_color(colors.text)
                .child(div().flex_1().child(text))
                .child(
                    gpui_kit::component::button::Button::new("dismiss-problems")
                        .ghost()
                        .small()
                        .label("Dismiss")
                        .on_click(cx.listener(|this, _, _, cx| this.dismiss_problems(cx))),
                )
                .into_any_element(),
        )
    }

    /// Tells the room where the caret is, when it moved.
    pub(super) fn collab_caret(&mut self, cx: &mut Context<Self>) {
        self.book_caret(cx);
        let Some(open) = self.shared.as_mut().filter(|s| !s.ended) else {
            return;
        };
        let body = self.body.read(cx);
        let range = body.selected_range();
        let head = body.cursor();
        let anchor = if head == range.start {
            range.end
        } else {
            range.start
        };
        if open.caret_sent != Some((anchor, head)) {
            open.caret_sent = Some((anchor, head));
            open.engine.set_caret(&open.share_id, anchor, head);
        }
    }

    /// Places other people's cursors in the text again, and tints their
    /// selections.
    pub(super) fn refresh_cursors(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_ref().filter(|s| !s.ended) else {
            return;
        };
        let text = self.body.read(cx).value();
        let marks = remote_marks(&open.engine.cursors(&open.share_id), &text);
        let dark = cx.theme().is_dark();
        let tints: Vec<TextDecoration> = marks
            .iter()
            .filter(|m| !m.selection.is_empty())
            .map(|m| {
                TextDecoration::new(
                    m.selection.clone(),
                    HighlightStyle {
                        background_color: Some(cursor_colour(m.colour, dark).opacity(0.22)),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let collection = self.shared.as_ref().and_then(|s| s.selections.clone());
        let made = match collection {
            Some(collection) => {
                collection.set(tints, cx);
                None
            }
            None if tints.is_empty() => None,
            None => Some(self.body.update(cx, |state, cx| {
                state.create_decorations_collection(tints, cx)
            })),
        };
        if let Some(open) = self.shared.as_mut() {
            if made.is_some() {
                open.selections = made;
            }
            if open.cursors != marks {
                open.cursors = marks;
                cx.notify();
            }
        }
    }

    /// Tints the words the note's open comments are on.
    pub fn refresh_comments(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.shared.as_ref().filter(|s| !s.ended) else {
            return;
        };
        let comments = open.engine.comments(&open.share_id).unwrap_or_default();
        let members = note_members(&open.engine, &open.share_id, &comments);
        let text = self.body.read(cx).value();
        let colors = Colors::get(cx);
        let tints: Vec<TextDecoration> = comment_ranges(&comments, &text)
            .into_iter()
            .map(|range| {
                TextDecoration::new(
                    range,
                    HighlightStyle {
                        background_color: Some(Hsla::from(colors.warning).opacity(0.2)),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let collection = open.highlights.clone();
        let made = match collection {
            Some(collection) => {
                collection.set(tints, cx);
                None
            }
            None if tints.is_empty() => None,
            None => Some(self.body.update(cx, |state, cx| {
                state.create_decorations_collection(tints, cx)
            })),
        };
        if let Some(open) = self.shared.as_mut() {
            if made.is_some() {
                open.highlights = made;
            }
            open.comments = comments;
            open.members = members;
        }
        cx.notify();
    }

    /// The open shared note's comments, this account's role in it and the
    /// usernames in it; `None` for a note that isn't shared (any more).
    pub fn shared_comments(&self) -> Option<(Role, &[CommentView], &[String])> {
        self.shared
            .as_ref()
            .filter(|s| !s.ended)
            .map(|s| (s.role, s.comments.as_slice(), s.members.as_slice()))
    }

    /// Selects the words comment `id` is on, or puts the caret where an
    /// orphaned comment's words were, and shows them.
    pub fn select_comment(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(placement) = self
            .shared_comments()
            .and_then(|(_, comments, _)| comments.iter().find(|v| v.comment.id == id))
            .map(|v| v.placement.clone())
        else {
            return;
        };
        let text = self.body.read(cx).value();
        let range = comment_target(&placement, &text);
        if self.view_mode == crate::config::ViewMode::Preview {
            let line = text[..range.start].matches('\n').count();
            self.reveal_line(u32::try_from(line).unwrap_or(u32::MAX), window, cx);
            return;
        }
        self.body.update(cx, |state, cx| {
            state.set_selected_range(range, cx);
            state.focus(window, cx);
        });
        cx.notify();
    }

    /// Changes the words of comment `id`, one of this account's.
    pub fn edit_comment(
        &mut self,
        id: &str,
        body: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return Err("A comment needs some words.".into());
        }
        self.comment_op(cx, |engine, share_id| {
            engine.edit_comment(share_id, id, body)
        })
    }

    /// Deletes comment `id`, one of this account's.
    pub fn delete_comment(&mut self, id: &str, cx: &mut Context<Self>) -> Result<(), String> {
        self.comment_op(cx, |engine, share_id| engine.delete_comment(share_id, id))
    }

    /// Resolves comment `id`, or opens it again.
    pub fn resolve_comment(
        &mut self,
        id: &str,
        resolved: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.comment_op(cx, |engine, share_id| {
            engine.resolve_comment(share_id, id, resolved)
        })
    }

    /// Writes a comment event for the open shared note, then shows the
    /// comments as they are now.
    fn comment_op(
        &mut self,
        cx: &mut Context<Self>,
        op: impl FnOnce(&Engine, &str) -> kanso_collab::api::Result<()>,
    ) -> Result<(), String> {
        let open = self
            .shared
            .as_ref()
            .filter(|s| !s.ended)
            .ok_or("The note isn't shared any more.")?;
        if !open.role.may_comment() {
            return Err("You can view this note, but not comment on it.".into());
        }
        op(&open.engine, &open.share_id).map_err(|err| crate::collab::error_text(&err))?;
        self.refresh_comments(cx);
        Ok(())
    }

    /// Comments on the selected words of the open shared note.
    pub fn comment_on_selection(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let open = self
            .shared
            .as_ref()
            .filter(|s| !s.ended)
            .ok_or("Comments are for shared notes. Share the note first.")?;
        if !open.role.may_comment() {
            return Err("You can view this note, but not comment on it.".into());
        }
        // The engine's text first, so the selection is in it.
        self.collab_sync(window, cx);
        let range = self.body.read(cx).selected_range();
        if range.is_empty() {
            return Err("Select the words to comment on first.".into());
        }
        let open = self
            .shared
            .as_ref()
            .ok_or("The note isn't shared any more.")?;
        open.engine
            .add_comment(&open.share_id, range, text)
            .map_err(|err| crate::collab::error_text(&err))?;
        self.refresh_comments(cx);
        Ok(())
    }

    /// Other people's carets with their names, over the text once it has
    /// laid out.
    // no-prop-test: painting; `remote_marks` has the properties and the
    // Comments snapshot shows a remote caret.
    pub(super) fn remote_carets(&self, window: &Window, cx: &App) -> Option<AnyElement> {
        let marks = match (&self.shared, &self.book_note) {
            (Some(open), _) if !open.ended => open.cursors.clone(),
            (_, Some(book)) => book.cursors.clone(),
            _ => return None,
        };
        if marks.is_empty() {
            return None;
        }
        let body = self.body.clone();
        let dark = cx.theme().is_dark();
        let label_text = Hsla::from(Colors::get(cx).editor);
        let font = window.text_style().font();
        Some(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, cx| {
                    for mark in &marks {
                        let Some(at) = body.read(cx).range_to_bounds(&(mark.caret..mark.caret))
                        else {
                            continue;
                        };
                        let colour = cursor_colour(mark.colour, dark);
                        let caret = Bounds::new(
                            point(at.origin.x - px(1.), at.origin.y),
                            size(px(2.), at.size.height),
                        );
                        let name: SharedString = mark.name.clone().into();
                        let run = TextRun {
                            len: name.len(),
                            font: font.clone(),
                            color: label_text,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        let line =
                            window
                                .text_system()
                                .shape_line(name, px(LABEL_SIZE), &[run], None);
                        let height = px(LABEL_HEIGHT);
                        // Above the line, or below it on the first line shown.
                        let mut top = at.origin.y - height;
                        if top < bounds.top() {
                            top = at.origin.y + at.size.height;
                        }
                        let label = Bounds::new(
                            point(at.origin.x - px(1.), top),
                            size(line.width + px(8.), height),
                        );
                        window.with_content_mask(Some(ContentMask { bounds }), |window| {
                            window.paint_quad(fill(caret, colour));
                            window.paint_quad(fill(label, colour).corner_radii(px(3.)));
                            line.paint(
                                point(label.origin.x + px(4.), top + px(1.)),
                                px(13.),
                                TextAlign::Left,
                                None,
                                window,
                                cx,
                            )
                            .ok();
                        });
                    }
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .into_any_element(),
        )
    }

    /// The role the open note was shared with this account at, if it is
    /// a shared note.
    pub fn shared_role(&self) -> Option<Role> {
        self.shared.as_ref().filter(|s| !s.ended).map(|s| s.role)
    }
}

impl NoteEditor {
    /// Joins the notebook's room for a note of it: one room per notebook,
    /// kept while any of its notes is open. A note's own window doesn't
    /// (only the main window's editor is in the room). The role the signed
    /// list gives this account now decides whether the text can be typed in.
    // no-prop-test: engine and editor glue; `book_note_shows_who_is_in_it`
    // drives it.
    fn book_follow(
        &mut self,
        share_id: String,
        stream: String,
        role: Role,
        cx: &mut Context<Self>,
    ) {
        let engine = crate::collab::current(cx);
        let role = engine
            .as_ref()
            .and_then(|e| e.book_view(&share_id).ok())
            .and_then(|v| v.my_role)
            .unwrap_or(role);
        self.set_text_read_only(!role.may_edit(), cx);
        if engine.is_none() || self.standalone {
            return;
        }
        crate::shared_books::open_note(&share_id, &stream, cx);
        self.book_note = Some(crate::collab_model::OpenBookNote {
            share_id,
            stream,
            caret_sent: None,
            cursors: Vec::new(),
            selections: None,
        });
        self.book_caret(cx);
        self.refresh_book_cursors(cx);
    }

    /// Leaves the notebook note's room; the room closes with the notebook's
    /// last open note.
    fn book_detach(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.book_note.take() else {
            return;
        };
        if let Some(tints) = open.selections {
            tints.clear(cx);
        }
        // Once this update is over, so the other editors can be read.
        cx.defer(move |cx| crate::shared_books::close_note(&open.share_id, &open.stream, cx));
        cx.notify();
    }

    /// The notebook and note log open here, if a notebook note is.
    pub fn book_note_open(&self) -> Option<(&str, &str)> {
        self.book_note
            .as_ref()
            .map(|b| (b.share_id.as_str(), b.stream.as_str()))
    }

    /// A collaborator's version of a note of a shared notebook arrived over
    /// unsaved typing: when no change of theirs touches a change typed here
    /// (`merge_apart`), both are kept, theirs going in as splices that leave
    /// the caret where it was, and the merge is saved as typing is. Only
    /// edits that meet are asked about. Whether it was merged.
    // no-prop-test: editor glue; `merge_apart` has the properties, and
    // `a_notebook_edit_apart_from_the_typing_merges` types through it.
    pub(super) fn book_merge_in(
        &mut self,
        latest: &kanso_core::Note,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(note) = self.note.as_ref() else {
            return false;
        };
        if book_stream(&note.extra).is_none()
            || self.canvas_body.is_some()
            || note.is_locked()
            || latest.is_locked()
            || latest.book_id != note.book_id
        {
            return false;
        }
        let now = self.current_text(cx);
        let Some(title) = kanso_collab::merge_apart(&note.title, &now.0, &latest.title) else {
            return false;
        };
        let Some(body) = kanso_collab::merge_apart(&note.body, &now.1, &latest.body) else {
            return false;
        };
        let (title_edits, body_edits) = (splices(&now.0, &title), splices(&now.1, &body));
        self.note = Some(latest.clone());
        self.incoming = None;
        if !title_edits.is_empty() {
            self.title.update(cx, |state, cx| {
                let range = state.selected_range();
                let head = state.cursor();
                let anchor = if head == range.start {
                    range.end
                } else {
                    range.start
                };
                for edit in &title_edits {
                    state.set_selected_range(edit.at..edit.at + edit.delete, cx);
                    state.replace(edit.insert.clone(), window, cx);
                }
                let (anchor, head) = map_selection(anchor, head, &title_edits);
                state.set_selected_range(anchor..head, cx);
            });
        }
        if !body_edits.is_empty() {
            self.splice_body(&body_edits, window, cx);
        }
        if self.has_unsaved_edits(cx) {
            self.schedule_autosave(cx);
        }
        cx.notify();
        true
    }

    /// Tells the notebook's room where the caret is, when it moved.
    fn book_caret(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.book_note.as_mut() else {
            return;
        };
        let body = self.body.read(cx);
        let range = body.selected_range();
        let head = body.cursor();
        let anchor = if head == range.start {
            range.end
        } else {
            range.start
        };
        if open.caret_sent != Some((anchor, head)) {
            open.caret_sent = Some((anchor, head));
            if let Some(engine) = crate::collab::current(cx) {
                engine.set_book_caret(&open.share_id, &open.stream, anchor, head);
            }
        }
    }

    /// Places the other members' cursors in the notebook note, with the
    /// names the notebook's signed member list gives them.
    pub fn refresh_book_cursors(&mut self, cx: &mut Context<Self>) {
        let Some(open) = self.book_note.as_ref() else {
            return;
        };
        let Some(engine) = crate::collab::current(cx) else {
            return;
        };
        let text = self.body.read(cx).value();
        let marks = remote_marks(&engine.book_cursors(&open.share_id, &open.stream), &text);
        let dark = cx.theme().is_dark();
        let tints: Vec<TextDecoration> = marks
            .iter()
            .filter(|m| !m.selection.is_empty())
            .map(|m| {
                TextDecoration::new(
                    m.selection.clone(),
                    HighlightStyle {
                        background_color: Some(cursor_colour(m.colour, dark).opacity(0.22)),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let made = match open.selections.clone() {
            Some(collection) => {
                collection.set(tints, cx);
                None
            }
            None if tints.is_empty() => None,
            None => Some(self.body.update(cx, |state, cx| {
                state.create_decorations_collection(tints, cx)
            })),
        };
        if let Some(open) = self.book_note.as_mut() {
            if made.is_some() {
                open.selections = made;
            }
            if open.cursors != marks {
                open.cursors = marks;
                cx.notify();
            }
        }
    }

    /// Who else is in the open notebook note, for tests.
    #[cfg(test)]
    pub fn book_cursors_for_test(&self) -> Vec<String> {
        self.book_note
            .as_ref()
            .map(|b| b.cursors.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default()
    }
}

/// The note's log in a shared notebook, if its share mark is a notebook's.
/// Pure.
pub(crate) fn book_stream(extra: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    let mark = extra.get(kanso_core::model::SHARE_FIELD)?.as_object()?;
    if mark.get("book").and_then(|b| b.as_bool()) != Some(true) {
        return None;
    }
    mark.get("stream")?.as_str().map(str::to_owned)
}

/// The usernames in a shared note, to set @mentions apart: its members as
/// last refreshed, and whoever wrote its comments.
fn note_members(engine: &Engine, share_id: &str, comments: &[CommentView]) -> Vec<String> {
    let mut names: Vec<String> = engine
        .share_view(share_id)
        .map(|view| {
            std::iter::once(view.owner.username)
                .chain(view.members.into_iter().map(|m| m.person.username))
                .collect()
        })
        .unwrap_or_default();
    names.extend(comments.iter().map(|v| v.comment.author_name.clone()));
    names.push(engine.caller().username.clone());
    names.retain(|n| !n.is_empty());
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
impl NoteEditor {
    pub fn footer_share_for_test(&self, cx: &App) -> Option<String> {
        self.footer.read(cx).share_shown().map(str::to_owned)
    }

    pub fn select_for_test(&mut self, range: std::ops::Range<usize>, cx: &mut Context<Self>) {
        self.body
            .update(cx, |state, cx| state.set_selected_range(range, cx));
    }

    pub fn canvas_for_test(&self) -> Entity<crate::canvas_view::CanvasView> {
        self.canvas.clone()
    }

    pub fn canvas_sync_for_test(&mut self, cx: &mut Context<Self>) {
        self.canvas.update(cx, |view, cx| view.sync_shared(cx));
    }

    pub fn collab_sync_for_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.collab_sync(window, cx);
    }
}
