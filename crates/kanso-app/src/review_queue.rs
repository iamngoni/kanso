//! The review queue: the edits an agent suggested, waiting for the person.
//!
//! The agent never writes a note. What it suggests (`kanso-server`'s
//! `ReviewQueue`) waits here as a card with the note's title, the agent's
//! name and a diff, and **Apply** is the only thing that writes it, once,
//! through the store as the agent's edit that you approved. A note that
//! changed since the agent read it refuses to apply and shows both versions.
//! Suggestions live in memory: they expire after an hour and are gone when
//! Kanso quits.

use gpui_component::scroll::ScrollableElement as _;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_server::{ApplyError, ChangeId, ProposedChange, ReviewEvent, ReviewQueue};
use kanso_store::Store;

use crate::clock::Clock;
use crate::diff_view::hunk_lines;
use crate::history_diff::{Hunk, hunks};
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of the review sheet.
pub const SHEET_WIDTH: f32 = 720.;
/// How much of a note's text the conflict view shows of each version.
const COMPARE_CHARS: usize = 4_000;

/// "1 suggested edit" or "3 suggested edits".
pub fn pending_line(count: usize) -> String {
    match count {
        1 => "1 suggested edit".to_owned(),
        n => format!("{n} suggested edits"),
    }
}

/// What a card says the agent suggests: an edit to a note, or a new note
/// in notebook `book`.
pub fn heading(change: &ProposedChange, book: Option<&str>) -> String {
    let title = |title: &str| {
        if title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            title.trim().to_owned()
        }
    };
    if change.is_new_note() {
        match book {
            Some(book) => format!("New note “{}” in {book}", title(&change.new_title)),
            None => format!("New note “{}”", title(&change.new_title)),
        }
    } else if change.old_title == change.new_title {
        title(&change.old_title)
    } else {
        format!(
            "{} → {}",
            title(&change.old_title),
            title(&change.new_title)
        )
    }
}

/// The queue the app shares with the agents' server, when there is one.
pub fn queue(cx: &App) -> Option<ReviewQueue> {
    cx.try_global::<Services>()
        .map(|services| services.agent_tokens.reviews().clone())
}

/// The diff of one suggestion: the title's lines, then the text's.
struct Diff {
    /// What it was worked out from: the lengths of the new title and text.
    sig: (usize, usize),
    title: Rc<Vec<Hunk>>,
    body: Rc<Vec<Hunk>>,
}

/// The sheet listing what is waiting.
pub struct ReviewSheet {
    queue: ReviewQueue,
    store: Arc<Store>,
    /// Suggestions Apply refused because the note changed, with what the
    /// note says now. They stay until discarded.
    conflicts: HashMap<ChangeId, ApplyError>,
    /// Ones this sheet applied or discarded, so a second click on a card
    /// that is going away says nothing.
    settled: HashSet<ChangeId>,
    /// The last thing that happened, or why Apply refused.
    notice: Option<String>,
    diffs: HashMap<ChangeId, Diff>,
    /// Shown inside the agent review sheet: no introduction and nothing at
    /// all while no suggestion waits.
    embedded: bool,
    _watch: Task<()>,
}

impl ReviewSheet {
    pub fn new(queue: ReviewQueue, store: Arc<Store>, cx: &mut Context<Self>) -> ReviewSheet {
        let events = queue.subscribe();
        let watch = cx.spawn(async move |this, cx| {
            while events.recv().await.is_ok() {
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        ReviewSheet {
            queue,
            store,
            conflicts: HashMap::new(),
            settled: HashSet::new(),
            notice: None,
            diffs: HashMap::new(),
            embedded: false,
            _watch: watch,
        }
    }

    /// The sheet as a section of the agent review sheet.
    pub fn embedded(queue: ReviewQueue, store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        let mut sheet = ReviewSheet::new(queue, store, cx);
        sheet.embedded = true;
        sheet
    }

    /// Applies suggestion `id`: writes it as the agent's edit, once. A note
    /// that changed since the agent read it is refused and both versions
    /// are shown.
    pub fn apply(&mut self, id: ChangeId, cx: &mut Context<Self>) {
        match self.queue.apply(&self.store, id) {
            Ok(applied) => {
                self.settled.insert(id);
                self.conflicts.remove(&id);
                let title = applied.title.trim();
                self.notice = Some(if title.is_empty() {
                    "Applied.".to_owned()
                } else {
                    format!("Applied to “{title}”.")
                });
            }
            Err(ApplyError::Gone) if self.settled.contains(&id) => {}
            Err(err @ ApplyError::Conflict { .. }) => {
                self.conflicts.insert(id, err);
                self.notice = None;
            }
            Err(err) => {
                self.conflicts.remove(&id);
                self.notice = Some(err.message().to_owned());
            }
        }
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn notice_for_test(&self) -> Option<String> {
        self.notice.clone()
    }

    #[cfg(test)]
    pub(crate) fn is_conflict_for_test(&self, id: ChangeId) -> bool {
        self.conflicts.contains_key(&id)
    }

    /// Throws suggestion `id` away.
    pub fn discard(&mut self, id: ChangeId, cx: &mut Context<Self>) {
        if self.queue.discard(id) {
            self.settled.insert(id);
            self.notice = Some("Discarded.".to_owned());
        }
        self.conflicts.remove(&id);
        cx.notify();
    }

    /// The suggestion's diff, worked out once per text.
    fn diff(&mut self, change: &ProposedChange) -> (Rc<Vec<Hunk>>, Rc<Vec<Hunk>>) {
        let sig = (change.new_title.len(), change.new_body.len());
        if let Some(diff) = self.diffs.get(&change.id)
            && diff.sig == sig
        {
            return (diff.title.clone(), diff.body.clone());
        }
        let title = Rc::new(if change.old_title == change.new_title {
            Vec::new()
        } else {
            hunks(&change.old_title, &change.new_title)
        });
        let body = Rc::new(hunks(&change.old_body, &change.new_body));
        self.diffs.insert(
            change.id,
            Diff {
                sig,
                title: title.clone(),
                body: body.clone(),
            },
        );
        (title, body)
    }

    fn book_name(&self, change: &ProposedChange) -> Option<String> {
        let book = change.book_id.as_deref()?;
        let books = self.store.books().ok()?;
        books
            .into_iter()
            .find(|candidate| candidate.id == book)
            .map(|book| book.name)
    }

    fn render_card(
        &mut self,
        ix: usize,
        change: ProposedChange,
        colors: &Colors,
        mono: Option<&'static str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = change.id;
        let (title_diff, body_diff) = self.diff(&change);
        let conflict = self.conflicts.get(&id).cloned();
        let book = self.book_name(&change);
        let when = Clock::edited_label(cx, change.created_at);
        let section = |hunks: &Rc<Vec<Hunk>>, label: &'static str| {
            (!hunks.is_empty()).then(|| {
                v_flex()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_medium()
                            .text_color(colors.text_secondary)
                            .child(label),
                    )
                    .children(hunks.iter().map(|hunk| {
                        div()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(colors.divider)
                            .overflow_hidden()
                            .child(hunk_lines(hunk, colors, mono))
                    }))
            })
        };
        let unchanged = title_diff.is_empty() && body_diff.is_empty();
        let compare = |label: &'static str, title: &str, body: &str| {
            let shown: String = body.chars().take(COMPARE_CHARS).collect();
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_medium()
                        .text_color(colors.text_secondary)
                        .child(label),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .font_medium()
                        .text_color(colors.text)
                        .child(title.to_owned()),
                )
                .child(
                    div()
                        .max_h(px(180.))
                        .overflow_hidden()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text)
                        .when_some(mono, |el, font| el.font_family(font))
                        .child(shown),
                )
        };
        v_flex()
            .id(("suggestion", ix))
            .flex_none()
            .gap(px(8.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(heading(&change, book.as_deref())),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(16.))
                            .text_color(colors.text_muted)
                            .child(format!("Suggested by {} · {when}", change.agent)),
                    ),
            )
            .when(unchanged, |el| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child("This suggestion doesn't change the text or the title."),
                )
            })
            .children(section(&title_diff, "Title"))
            .children(section(&body_diff, "Text"))
            .when_some(conflict, |el, conflict| {
                let ApplyError::Conflict {
                    current_title,
                    current_body,
                } = &conflict
                else {
                    return el;
                };
                el.child(
                    v_flex()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_medium()
                                .text_color(colors.danger)
                                .child(conflict.message().to_owned()),
                        )
                        .child(
                            h_flex()
                                .items_start()
                                .gap(px(12.))
                                .child(compare(
                                    "The agent's version",
                                    &change.new_title,
                                    &change.new_body,
                                ))
                                .child(compare("The note now", current_title, current_body)),
                        ),
                )
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .justify_end()
                    .child(
                        Button::new(("suggestion-discard", ix))
                            .outline()
                            .small()
                            .label("Discard")
                            .on_click(cx.listener(move |this, _, _, cx| this.discard(id, cx))),
                    )
                    .child(
                        Button::new(("suggestion-apply", ix))
                            .primary()
                            .small()
                            .label("Apply")
                            .disabled(self.conflicts.contains_key(&id))
                            .on_click(cx.listener(move |this, _, _, cx| this.apply(id, cx))),
                    ),
            )
            .into_any_element()
    }
}

// no-prop-test: drawing; the cards' words are `heading` and `pending_line`.
impl Render for ReviewSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mono = crate::theme::mono_font(cx);
        let pending = self.queue.list();
        // A suggestion that has gone has no diff or conflict to keep.
        self.diffs
            .retain(|id, _| pending.iter().any(|c| c.id == *id));
        self.conflicts
            .retain(|id, _| pending.iter().any(|c| c.id == *id));
        let cards: Vec<AnyElement> = pending
            .into_iter()
            .enumerate()
            .map(|(ix, change)| self.render_card(ix, change, colors, mono, cx))
            .collect();
        if self.embedded && cards.is_empty() {
            return div().into_any_element();
        }
        let embedded = self.embedded;
        v_flex()
            .gap(px(10.))
            .when(embedded, |el| {
                el.child(
                    div()
                        .pt(px(4.))
                        .text_size(px(13.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child("Suggested while you asked"),
                )
            })
            .when(!embedded, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child("Nothing changes until you apply a suggestion. Suggestions are kept in memory for an hour and are gone when Kanso quits."),
                )
            })
            .when_some(self.notice.clone(), |el, notice| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child(notice),
                )
            })
            .when(cards.is_empty(), |el| {
                el.child(
                    div()
                        .py_6()
                        .text_size(px(13.5))
                        .text_color(colors.text_secondary)
                        .child("No suggested edits are waiting."),
                )
            })
            .child(if embedded {
                v_flex()
                    .id("review-cards")
                    .gap(px(10.))
                    .children(cards)
                    .into_any_element()
            } else {
                v_flex()
                    .id("review-cards")
                    .gap(px(10.))
                    .max_h(px(480.))
                    .overflow_y_scrollbar()
                    .children(cards)
                    .into_any_element()
            })
            .into_any_element()
    }
}

/// The sheet while it is open, so a second suggestion doesn't open another.
struct OpenSheet(WeakEntity<ReviewSheet>);

impl Global for OpenSheet {}

/// Opens the review sheet, or brings back the one that is open.
pub fn open_review(window: &mut Window, cx: &mut App) -> Option<Entity<ReviewSheet>> {
    if let Some(open) = cx
        .try_global::<OpenSheet>()
        .and_then(|open| open.0.upgrade())
    {
        return Some(open);
    }
    let queue = queue(cx)?;
    let store = cx.global::<AppState>().store.clone();
    let sheet = cx.new(|cx| ReviewSheet::new(queue, store, cx));
    cx.set_global(OpenSheet(sheet.downgrade()));
    let view = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Suggested edits")
            .width(px(SHEET_WIDTH))
            .child(view.clone())
            .show_cancel(false)
            .ok_text("Close")
    });
    Some(sheet)
}

/// Whether the review sheet is open now.
pub fn is_open(cx: &App) -> bool {
    cx.try_global::<OpenSheet>()
        .is_some_and(|open| open.0.upgrade().is_some())
}

/// Whether the window has the keyboard, which is when a suggestion opens
/// the sheet. A headless test window never does; a test can say it does.
fn focused(window: &Window, cx: &App) -> bool {
    #[cfg(test)]
    if cx.has_global::<AssumeFocused>() {
        return true;
    }
    let _ = cx;
    window.is_window_active()
}

/// Makes `focused` true in a test.
#[cfg(test)]
pub(crate) struct AssumeFocused;

#[cfg(test)]
impl Global for AssumeFocused {}

/// Whether a change to document `id` can make a waiting suggestion
/// unappliable: a notebook or tag (locked, excluded), or a note it targets.
pub fn can_affect(id: &str, waiting: &[ProposedChange]) -> bool {
    id.starts_with("book:")
        || id.starts_with("tag:")
        || waiting
            .iter()
            .any(|change| change.note_id.as_deref() == Some(id))
}

/// Watches the queue for this window. When a suggestion arrives while the
/// window is focused, the sheet opens. When a note, notebook or tag changes
/// so that a waiting suggestion can no longer be applied (locked, trashed,
/// `#ai-exclude`d), the suggestion is dropped. The Ask panel shows the
/// count on its own.
pub fn watch(window: &mut Window, cx: &mut Context<crate::workspace::Workspace>) -> Vec<Task<()>> {
    let Some(queue) = queue(cx) else {
        return Vec::new();
    };
    let events = queue.subscribe();
    let opening = cx.spawn_in(window, async move |this, cx| {
        while let Ok(event) = events.recv().await {
            if !matches!(event, ReviewEvent::Arrived(_)) {
                continue;
            }
            let alive = this.update_in(cx, |_, window, cx| {
                if focused(window, cx) && !is_open(cx) {
                    open_review(window, cx);
                }
            });
            if alive.is_err() {
                break;
            }
        }
    });
    let store = cx.global::<AppState>().store.clone();
    let changes = store.subscribe();
    let dropping = cx.spawn(async move |_, _| {
        while let Ok(change) = changes.recv().await {
            let waiting = queue.list();
            if !waiting.is_empty() && can_affect(&change.id, &waiting) {
                queue.drop_unappliable(&store);
            }
        }
    });
    vec![opening, dropping]
}

#[cfg(test)]
mod tests {
    use super::{heading, pending_line};
    use kanso_server::{AgentScope, NewChange, ReviewQueue, TokenStore};
    use proptest::prelude::*;
    use std::sync::Arc;

    fn change(note: Option<&str>, old: &str, new: &str) -> kanso_server::ProposedChange {
        let queue = ReviewQueue::new();
        let minted = TokenStore::new().mint(AgentScope::ask(None));
        let id = queue
            .stage(NewChange {
                note_id: note.map(str::to_owned),
                base_revision: note.map(|_| "1-a".to_owned()),
                book_id: note.is_none().then(|| "book:b".to_owned()),
                old_title: old.to_owned(),
                old_body: String::new(),
                new_title: new.to_owned(),
                new_body: String::new(),
                agent: "Claude Agent".to_owned(),
                token: minted.id,
                scope: Arc::new(AgentScope::ask(None)),
            })
            .unwrap();
        queue.get(id).unwrap()
    }

    #[test]
    fn the_cards_name_what_is_suggested() {
        assert_eq!(pending_line(1), "1 suggested edit");
        assert_eq!(pending_line(2), "2 suggested edits");
        assert_eq!(
            heading(&change(Some("note:a"), "Plan", "Plan"), None),
            "Plan"
        );
        assert_eq!(
            heading(&change(Some("note:a"), "Plan", "Plan B"), None),
            "Plan → Plan B"
        );
        assert_eq!(heading(&change(Some("note:a"), " ", " "), None), "Untitled");
        assert_eq!(
            heading(&change(None, "", "Ideas"), Some("Work")),
            "New note “Ideas” in Work"
        );
    }

    proptest! {
        /// The count line is singular for one and plural otherwise, and
        /// always starts with the number.
        #[test]
        fn prop_pending_line_counts(n in 0usize..10_000) {
            let line = pending_line(n);
            prop_assert!(line.starts_with(&n.to_string()));
            prop_assert_eq!(line.ends_with("edit"), n == 1);
            prop_assert_eq!(line.ends_with("edits"), n != 1);
        }

        /// A heading is one line, never empty, and names the agent's new
        /// title for a new note.
        #[test]
        fn prop_heading_is_one_line(old in "[ -~]{0,20}", new in "[ -~]{0,20}", edit in any::<bool>()) {
            let found = change(edit.then_some("note:a"), &old, &new);
            let line = heading(&found, Some("Book"));
            prop_assert!(!line.is_empty() && !line.contains('\n'));
            if !edit && !new.trim().is_empty() {
                prop_assert!(line.contains(new.trim()));
            }
        }
    }
}
