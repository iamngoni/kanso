//! The note history dialog: earlier versions, newest first, with what
//! changed since each (section by section, each one to copy) or the
//! version itself, and a way to bring one back.

use std::rc::Rc;

use gpui_kit::base::text::TextView;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Revision;

use crate::clock::Clock;
use crate::diff_view::hunk_lines;
use crate::editor::NoteEditor;
use crate::grouping::as_title;
use crate::history_diff::{Hunk, hunks};
use crate::theme::Colors;
use crate::zoom::px;
use kanso_engine::history::{authors_label, origin_label};

/// The dialog's width and its body's height, and the width of its list of
/// versions.
pub const DIALOG_WIDTH: f32 = 820.;
const BODY_HEIGHT: f32 = 440.;
const LIST_WIDTH: f32 = 200.;

/// What the right of the dialog shows for the selected version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pane {
    /// What changed from the version to the note now.
    #[default]
    Changes,
    /// The version itself, as the preview shows a note.
    Version,
}

pub struct HistoryView {
    revisions: Vec<Revision>,
    selected: usize,
    scroll: ScrollHandle,
    /// The note's text now, to compare with.
    current: String,
    /// Who changed a shared note since its last saved version.
    current_authors: Option<String>,
    pub(crate) pane: Pane,
    /// The section last copied, for its button to say so.
    copied: Option<(usize, usize)>,
    /// The selected version's changes, worked out once per selection.
    hunks: Option<(usize, Rc<Vec<Hunk>>)>,
}

impl HistoryView {
    fn selected(&self) -> Option<&Revision> {
        self.revisions.get(self.selected)
    }

    fn hunks(&mut self) -> Rc<Vec<Hunk>> {
        if let Some((at, hunks)) = &self.hunks
            && *at == self.selected
        {
            return hunks.clone();
        }
        let old = self
            .selected()
            .map(|r| field(r, "body"))
            .unwrap_or_default();
        let found = Rc::new(hunks(&old, &self.current));
        self.hunks = Some((self.selected, found.clone()));
        found
    }

    fn render_pane_switch(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let current = self.pane;
        let tab = |id: &'static str, label: &'static str, pane: Pane, cx: &mut Context<Self>| {
            let on = current == pane;
            div()
                .id(id)
                .px(px(10.))
                .h(px(24.))
                .flex()
                .items_center()
                .rounded(px(5.))
                .text_size(px(12.5))
                .cursor_pointer()
                .when(on, |el| {
                    el.bg(colors.elevated)
                        .text_color(colors.text)
                        .font_medium()
                        .shadow_xs()
                })
                .when(!on, |el| el.text_color(colors.text_secondary))
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.pane = pane;
                    cx.notify();
                }))
        };
        h_flex()
            .flex_none()
            .p(px(2.))
            .gap(px(2.))
            .rounded(px(7.))
            .bg(colors.fill)
            .child(tab("pane-changes", "Changes", Pane::Changes, cx))
            .child(tab("pane-version", "This version", Pane::Version, cx))
    }

    fn render_changes(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let found = self.hunks();
        if found.is_empty() {
            return div()
                .pt_4()
                .text_size(px(13.5))
                .text_color(colors.text_secondary)
                .child("This version's text is the same as the note's now.")
                .into_any_element();
        }
        let mono = crate::theme::mono_font(cx);
        let selected = self.selected;
        let copied = self.copied;
        let mut sections = Vec::new();
        for (ix, hunk) in found.iter().enumerate() {
            let (added, removed) = hunk.counts();
            let span = match hunk.old_span() {
                Some((a, b)) if a == b => format!("Line {a}"),
                Some((a, b)) => format!("Lines {a}–{b}"),
                None => "New lines".to_owned(),
            };
            let text = hunk.old_text();
            let was_copied = copied == Some((selected, ix));
            let section = v_flex()
                .flex_none()
                .rounded(px(8.))
                .border_1()
                .border_color(colors.divider)
                .overflow_hidden()
                .child(
                    h_flex()
                        .h(px(30.))
                        .pl(px(10.))
                        .pr(px(4.))
                        .gap(px(8.))
                        .bg(colors.fill)
                        .border_b_1()
                        .border_color(colors.divider)
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_medium()
                                .text_color(colors.text_secondary)
                                .child(span),
                        )
                        .when(added > 0, |el| {
                            el.child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.success)
                                    .child(format!("+{added}")),
                            )
                        })
                        .when(removed > 0, |el| {
                            el.child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.danger)
                                    .child(format!("−{removed}")),
                            )
                        })
                        .child(div().flex_1())
                        .child(
                            Button::new(("copy-section", ix))
                                .ghost()
                                .xsmall()
                                .icon(if was_copied {
                                    IconName::Check
                                } else {
                                    IconName::Copy
                                })
                                .label(if was_copied { "Copied" } else { "Copy section" })
                                .tooltip("Copies this section as the version has it")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                                    this.copied = Some((selected, ix));
                                    cx.notify();
                                })),
                        ),
                )
                .child(hunk_lines(hunk, colors, mono));
            sections.push(section);
        }
        v_flex()
            .gap(px(12.))
            .pb(px(8.))
            .children(sections)
            .into_any_element()
    }
}

fn field(revision: &Revision, key: &str) -> String {
    revision
        .fields
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned()
}

// no-prop-test: drawing; the history snapshots show it in both themes.
impl Render for HistoryView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        if self.revisions.is_empty() {
            return div()
                .py_8()
                .text_size(px(13.5))
                .text_color(colors.text_secondary)
                .child("No earlier versions yet. Kanso keeps one every few minutes while you edit, and one before every change made elsewhere.")
                .into_any_element();
        }
        let rows: Vec<_> = self
            .revisions
            .iter()
            .enumerate()
            .map(|(ix, revision)| {
                let selected = ix == self.selected;
                v_flex()
                    .id(("revision", ix))
                    .px(px(10.))
                    .py(px(7.))
                    .gap(px(1.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    // The selected version as the selected note in the list.
                    .when(selected, |el| el.bg(colors.row_active))
                    .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .font_medium()
                            .text_color(colors.text)
                            // As the editor's header says when the note was
                            // edited, and the list names the day.
                            .child(as_title(&Clock::edited_label(cx, revision.saved_at))),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(16.))
                            .text_color(colors.quiet(selected))
                            .child(origin_label(&revision.origin)),
                    )
                    .when_some(authors_label(revision.author.as_deref()), |el, authors| {
                        el.child(
                            div()
                                .text_size(px(12.))
                                .line_height(px(16.))
                                .truncate()
                                .text_color(colors.quiet(selected))
                                .child(authors),
                        )
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = ix;
                        cx.notify();
                    }))
            })
            .collect();
        let (title, body) = self
            .selected()
            .map(|r| (field(r, "title"), field(r, "body")))
            .unwrap_or_default();
        let content = match self.pane {
            Pane::Changes => self.render_changes(cx),
            Pane::Version => {
                let body = crate::preview_chunks::showable(&body).into_owned();
                let dark = cx.theme().is_dark();
                // The version as the preview shows a note, a size smaller
                // to suit the dialog.
                TextView::markdown(("revision-text", self.selected), body)
                    .style(crate::preview::text_style(
                        colors,
                        &crate::themes::preview_colors(cx),
                        dark,
                    ))
                    .text_size(px(14.))
                    .line_height(relative(1.6))
                    .text_color(colors.text)
                    .into_any_element()
            }
        };
        let switch = self.render_pane_switch(cx);
        h_flex()
            .h(px(BODY_HEIGHT))
            .gap(px(14.))
            .items_start()
            .child(
                div()
                    .id("revisions")
                    .w(px(LIST_WIDTH))
                    .flex_none()
                    .h_full()
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .gap(px(2.))
                            // The note as it is now, when others changed it
                            // since its last version.
                            .when_some(self.current_authors.clone(), |el, authors| {
                                el.child(
                                    v_flex()
                                        .id("current-authors")
                                        .px(px(10.))
                                        .py(px(7.))
                                        .gap(px(1.))
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .line_height(px(18.))
                                                .font_medium()
                                                .text_color(colors.text)
                                                .child("Now"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.))
                                                .line_height(px(16.))
                                                .truncate()
                                                .text_color(colors.text_secondary)
                                                .child(authors),
                                        ),
                                )
                            })
                            .children(rows),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .border_l_1()
                    .border_color(colors.divider)
                    .pl(px(18.))
                    .child(
                        h_flex()
                            .pb(px(10.))
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(18.))
                                    .line_height(px(24.))
                                    .font_semibold()
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .child(switch),
                    )
                    .child(
                        div()
                            .id("revision-body")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .child(content),
                    ),
            )
            .into_any_element()
    }
}

/// Opens the history of the note in `editor`.
pub fn open_history(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let Some(id) = editor.read(cx).note_id().map(str::to_owned) else {
        return;
    };
    let store = cx.global::<crate::workspace::AppState>().store.clone();
    let revisions = store.revisions(&id).unwrap_or_default();
    let has_revisions = !revisions.is_empty();
    let current = editor.read(cx).body_text(cx).to_string();
    let shared_authors = store.shared_authors(&id).unwrap_or_default();
    let current_authors = authors_label(shared_authors.iter().map(String::as_str));
    let view = cx.new(|_| HistoryView {
        revisions,
        selected: 0,
        scroll: ScrollHandle::new(),
        current,
        current_authors,
        pane: Pane::default(),
        copied: None,
        hunks: None,
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        let view_for_ok = view.clone();
        let editor = editor.clone();
        alert
            .title("Note history")
            .width(px(DIALOG_WIDTH))
            .child(view.clone())
            .show_cancel(has_revisions)
            .cancel_text("Close")
            .ok_text(if has_revisions {
                "Restore this version"
            } else {
                "Close"
            })
            .on_ok(move |_, window, cx| {
                let chosen = view_for_ok
                    .read(cx)
                    .selected()
                    .map(|r| (field(r, "title"), field(r, "body")));
                if let Some((title, body)) = chosen {
                    editor.update(cx, |editor, cx| editor.restore(title, body, window, cx));
                }
                true
            })
    });
}
