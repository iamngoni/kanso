//! The inspector's Comments tab: a shared note's comments, those on words
//! still in the text in text order, then the ones whose words were deleted
//! under "Orphaned", each with the words it was on. Each shows its author,
//! when it was written and whether it is still on its way; @mentions of
//! people in the note stand out. Anyone who may comment resolves or reopens
//! a comment (resolved ones are dimmed and cut to a line); this account's
//! own comments can be edited and deleted. Clicking one selects its words
//! in the editor.

use gpui_component::scroll::ScrollableElement as _;
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_collab::{CommentView, Placement, Role};

use super::{Inspector, PAD_X};
use crate::clock::Clock;
use crate::collab_text::{comment_groups, comments_summary, mention_runs};
use crate::note_list::quiet_empty;
use crate::theme::{Colors, section_label};
use crate::zoom::px;

impl Inspector {
    /// How many of the open note's comments are open, for the tab's count.
    pub(super) fn open_comments(&self, cx: &App) -> usize {
        self.editor
            .read(cx)
            .shared_comments()
            .map_or(0, |(_, comments, _)| {
                comments.iter().filter(|v| !v.comment.resolved).count()
            })
    }

    /// The bottom band's words on the Comments tab.
    pub(super) fn comments_band(&self, cx: &App) -> String {
        self.editor
            .read(cx)
            .shared_comments()
            .map(|(_, comments, _)| comments_summary(comments))
            .unwrap_or_default()
    }

    pub(super) fn render_comments(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let editor = self.editor.read(cx);
        if editor.note_id().is_none() {
            return quiet_empty(Lucide::MessageSquare, "No note open".into(), None, colors)
                .into_any_element();
        }
        let Some((role, comments, members)) = editor.shared_comments() else {
            return quiet_empty(
                Lucide::MessageSquare,
                "Comments are for shared notes".into(),
                Some("Share this note to talk it over with others.".into()),
                colors,
            )
            .into_any_element();
        };
        if comments.is_empty() {
            let hint = if role.may_comment() {
                "Select some words, then choose Comment on Selection."
            } else {
                "Nobody has commented on this note yet."
            };
            return quiet_empty(
                Lucide::MessageSquare,
                "No comments yet".into(),
                Some(hint.into()),
                colors,
            )
            .into_any_element();
        }
        let comments = comments.to_vec();
        let members = members.to_vec();
        let (anchored, orphaned) = comment_groups(&comments);
        let mut list = v_flex()
            .id("comments")
            .size_full()
            .overflow_y_scrollbar()
            .track_scroll(&self.scroll)
            .px(px(PAD_X))
            .pt(px(6.))
            .pb(px(12.))
            .gap(px(2.));
        for ix in anchored {
            list = list.child(self.comment_row(ix, &comments[ix], role, &members, cx));
        }
        if !orphaned.is_empty() {
            list = list.child(
                section_label(div(), colors)
                    .px(px(8.))
                    .pt(px(12.))
                    .pb(px(4.))
                    .child("Orphaned"),
            );
            for ix in orphaned {
                list = list.child(self.comment_row(ix, &comments[ix], role, &members, cx));
            }
        }
        list.into_any_element()
    }

    // no-prop-test: drawing; `comment_groups` and `mention_runs` have the
    // properties, and `comments_list_in_the_inspector` drives the actions.
    fn comment_row(
        &self,
        ix: usize,
        view: &CommentView,
        role: Role,
        members: &[String],
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = Colors::get(cx);
        let comment = &view.comment;
        let resolved = comment.resolved;
        let orphaned = matches!(view.placement, Placement::Orphaned(_));
        let mut when = Clock::edited_label(cx, comment.created_at);
        if comment.edited_at.is_some() {
            when.push_str(" · Edited");
        }
        if view.pending {
            when.push_str(" · Sending…");
        }
        let accent = Hsla::from(colors.accent);
        let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> =
            mention_runs(&comment.body, members)
                .into_iter()
                .filter(|(_, mention)| *mention)
                .map(|(range, _)| {
                    (
                        range,
                        HighlightStyle {
                            color: Some(accent),
                            font_weight: Some(FontWeight::MEDIUM),
                            ..Default::default()
                        },
                    )
                })
                .collect();
        let body = StyledText::new(comment.body.clone()).with_highlights(highlights);
        let quote = comment.anchor.quote.trim().to_owned();

        let mut actions: Vec<AnyElement> = Vec::new();
        if role.may_comment() {
            let id = comment.id.clone();
            actions.push(self.comment_action(
                ("comment-resolve", ix),
                if resolved { "Reopen" } else { "Resolve" },
                cx,
                move |this, _, cx| {
                    this.editor
                        .update(cx, |editor, cx| editor.resolve_comment(&id, !resolved, cx))
                },
            ));
            if view.mine {
                let (id, words) = (comment.id.clone(), comment.body.clone());
                let editor = self.editor.clone();
                actions.push(
                    Button::new(("comment-edit", ix))
                        .ghost()
                        .xsmall()
                        .child(action_label("Edit", colors))
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            let (id, editor) = (id.clone(), editor.clone());
                            crate::dialogs::prompt_checked(
                                "Edit comment",
                                &words,
                                "Save",
                                window,
                                cx,
                                move |text, _, cx| {
                                    editor.update(cx, |editor, cx| {
                                        editor.edit_comment(&id, &text, cx)
                                    })
                                },
                            );
                        })
                        .into_any_element(),
                );
                let id = comment.id.clone();
                let editor = self.editor.clone();
                actions.push(
                    Button::new(("comment-delete", ix))
                        .ghost()
                        .xsmall()
                        .child(action_label("Delete", colors))
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            let (id, editor) = (id.clone(), editor.clone());
                            crate::dialogs::confirm(
                                "Delete this comment?",
                                "It goes for everyone in the note.",
                                "Delete",
                                window,
                                cx,
                                move |window, cx| {
                                    let result = editor
                                        .update(cx, |editor, cx| editor.delete_comment(&id, cx));
                                    if let Err(words) = result {
                                        window.push_notification(Notification::error(words), cx);
                                    }
                                },
                            );
                        })
                        .into_any_element(),
                );
            }
        }

        let select = self.editor.clone();
        let id = comment.id.clone();
        v_flex()
            .id(("comment", ix))
            .debug_selector(move || format!("comment-{ix}"))
            .w_full()
            .px(px(8.))
            .py(px(7.))
            .gap(px(3.))
            .rounded(px(6.))
            .cursor_pointer()
            .hover(|el| el.bg(colors.row_hover))
            .when(resolved, |el| el.opacity(0.55))
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_none()
                            .max_w(px(140.))
                            .truncate()
                            .text_size(px(12.5))
                            .font_medium()
                            .text_color(colors.text)
                            .child(comment.author_name.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child(when),
                    )
                    .when(resolved, |el| {
                        el.child(
                            div()
                                .flex_none()
                                .text_size(px(11.5))
                                .text_color(colors.text_muted)
                                .child("Resolved"),
                        )
                    }),
            )
            .when(!quote.is_empty(), |el| {
                el.child(
                    div()
                        .pl(px(7.))
                        .border_l_2()
                        .border_color(if orphaned {
                            Hsla::from(colors.divider)
                        } else {
                            Hsla::from(colors.warning).opacity(0.6)
                        })
                        .text_size(px(12.))
                        .line_height(px(16.))
                        .italic()
                        .text_color(colors.text_secondary)
                        .when(orphaned, |el| el.line_through())
                        .line_clamp(if orphaned { 2 } else { 1 })
                        .text_ellipsis()
                        .child(quote),
                )
            })
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(colors.text)
                    .when(resolved, |el| el.line_clamp(1).text_ellipsis())
                    .child(body),
            )
            .when(!actions.is_empty(), |el| {
                el.child(h_flex().gap(px(2.)).ml(px(-6.)).children(actions))
            })
            .on_click(move |_, window, cx| {
                let id = id.clone();
                select.update(cx, |editor, cx| editor.select_comment(&id, window, cx));
            })
    }

    /// A quiet text button under a comment; `run`'s error shows as a
    /// notification.
    fn comment_action(
        &self,
        id: impl Into<ElementId>,
        label: &'static str,
        cx: &mut Context<Self>,
        run: impl Fn(&mut Self, &mut Window, &mut Context<Self>) -> Result<(), String> + 'static,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        Button::new(id)
            .ghost()
            .xsmall()
            .child(action_label(label, colors))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                if let Err(words) = run(this, window, cx) {
                    window.push_notification(Notification::error(words), cx);
                }
            }))
            .into_any_element()
    }
}

fn action_label(label: &'static str, colors: &Colors) -> Div {
    div()
        .px_0p5()
        .text_size(px(11.5))
        .text_color(colors.link)
        .child(label)
}

#[cfg(test)]
impl Inspector {
    /// The comments as the tab lists them: (ID, orphaned), in order.
    pub fn comments_for_test(&self, cx: &App) -> Vec<(String, bool)> {
        let Some((_, comments, _)) = self.editor.read(cx).shared_comments() else {
            return Vec::new();
        };
        let (anchored, orphaned) = comment_groups(comments);
        anchored
            .into_iter()
            .map(|ix| (comments[ix].comment.id.clone(), false))
            .chain(
                orphaned
                    .into_iter()
                    .map(|ix| (comments[ix].comment.id.clone(), true)),
            )
            .collect()
    }
}
