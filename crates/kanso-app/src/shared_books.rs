//! Shared notebooks, the app's side (issue #390 › Shared studio notebooks):
//! their background sync beside shared notes', invitations to accept or
//! decline, the live room of a notebook while one of its notes is open, and
//! the Shared section of the sidebar, which lists the notebooks joined here
//! with their members and how their sync stands, and the invitations that
//! wait.
//!
//! The engine does every step that reaches the server or changes a key;
//! what it says comes back to [`Collab`]. Offline, nothing is said beyond
//! the status line: the next sync tries again.

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_collab::{BookLiveNotice, Engine, NoteEdit, book_copy};
use kanso_store::SharedStatus;

use crate::book_share_model::{
    BookNews, BookRow, BookTone, avatars, book_tone, initial, invitation_words, status_words,
};
use crate::collab_model::Collab;
use crate::library::{Filter, Library};
use crate::services::notify_windows;
use crate::sidebar::{Sidebar, SidebarEvent};
use crate::theme::{Colors, cursor_colour};
use crate::zoom::px;

/// Avatars a Shared row shows before "+n".
const AVATARS: usize = 3;

/// Syncs every notebook joined here and reads what the app shows of them.
/// Runs off the main thread.
pub fn sync_books(engine: &Engine) -> BookNews {
    let mut news = BookNews::default();
    for (share_id, result) in engine.sync_books() {
        match result {
            Ok(sync) => {
                if let Some(ended) = sync.ended {
                    news.ended.push(ended.message);
                }
                news.brought_back.extend(sync.brought_back);
            }
            Err(err) if err.is_offline() => news.offline = true,
            Err(err) if err.is_client_too_old() => news.needs_update = true,
            Err(err) => eprintln!("kanso: syncing shared notebook {share_id}: {err}"),
        }
    }
    match engine.joinable_books() {
        Ok(books) => news.invitations = Some(books),
        Err(err) if err.is_offline() => news.offline = true,
        Err(err) if err.is_client_too_old() => news.needs_update = true,
        Err(err) => eprintln!("kanso: reading notebook invitations: {err}"),
    }
    news.rows = Some(read_rows(engine));
    news.edits = engine.book_edits();
    news
}

/// The notebooks joined here as the library has them, without the server.
pub fn read_rows(engine: &Engine) -> Vec<BookRow> {
    let mut rows: Vec<BookRow> = engine
        .shared_books()
        .unwrap_or_default()
        .into_iter()
        .filter(|b| b.status == SharedStatus::Active)
        .filter_map(|b| {
            let view = engine.book_view(&b.share_id).ok()?;
            let waiting = engine.book_waiting(&b.share_id).unwrap_or(0);
            Some(BookRow { view, waiting })
        })
        .collect();
    rows.sort_by(|a, b| {
        (a.view.name.to_lowercase(), &a.view.share_id)
            .cmp(&(b.view.name.to_lowercase(), &b.view.share_id))
    });
    rows
}

/// Keeps what a sync brought and tells the person what ended.
pub fn take_news(news: BookNews, cx: &mut App) {
    let collab = cx.default_global::<Collab>();
    if let Some(rows) = news.rows {
        collab.books = rows;
    }
    if let Some(invitations) = news.invitations {
        collab.book_invitations = invitations;
    }
    collab.book_edits = by_note(news.edits);
    collab.book_update_needed = news.needs_update;
    // A room whose notebook ended closes.
    let joined: Vec<String> = collab
        .books
        .iter()
        .map(|b| b.view.share_id.clone())
        .collect();
    collab.book_rooms.retain(|share, _| joined.contains(share));
    for message in news.ended {
        notify_windows(move || Notification::info(message.clone()), cx);
    }
    tell_brought_back(&news.brought_back, cx);
    cx.refresh_windows();
}

/// Tells the person that notes they deleted are back, because someone
/// edited them before seeing them go (the phone says the same).
fn tell_brought_back(titles: &[String], cx: &mut App) {
    for title in titles {
        let message = book_copy::brought_back_notice(title);
        notify_windows(move || Notification::info(message.clone()), cx);
    }
}

/// Reads the notebooks and edits again from the library (after a step on
/// one, or a live change), without the server.
pub fn refresh_local(cx: &mut App) {
    let Some(engine) = crate::collab::current(cx) else {
        return;
    };
    let rows = read_rows(&engine);
    let edits = engine.book_edits();
    let collab = cx.default_global::<Collab>();
    collab.books = rows;
    collab.book_edits = by_note(edits);
    cx.refresh_windows();
}

fn by_note(edits: Vec<NoteEdit>) -> std::collections::HashMap<String, NoteEdit> {
    edits.into_iter().map(|e| (e.note_id.clone(), e)).collect()
}

/// Joins a notebook this account was invited to (or that another device
/// of the account joined): its notes come in now.
pub fn accept(share_id: String, cx: &mut App) {
    let gone = share_id.clone();
    crate::collab::run(
        cx,
        move |engine| {
            let joined = engine.join_book(&share_id)?;
            let name = engine
                .book_view(&share_id)
                .map(|v| v.name)
                .unwrap_or_default();
            Ok((joined, name))
        },
        move |result, cx| {
            match result {
                Ok((joined, name)) => {
                    cx.default_global::<Collab>()
                        .book_invitations
                        .retain(|b| b.share_id != gone);
                    crate::metrics::record(cx, crate::metrics::Feature::SharedNote);
                    let words = format!(
                        "You joined \"{name}\". {} in it.",
                        if joined.notes == 1 {
                            "1 note is".to_owned()
                        } else {
                            format!("{} notes are", joined.notes)
                        }
                    );
                    notify_windows(move || Notification::info(words.clone()), cx);
                }
                Err(words) => notify_windows(move || Notification::error(words.clone()), cx),
            }
            refresh_local(cx);
            crate::collab::sync_soon(cx);
        },
    );
}

/// Declines a notebook invitation.
pub fn decline(share_id: String, cx: &mut App) {
    let gone = share_id.clone();
    cx.default_global::<Collab>()
        .book_invitations
        .retain(|b| b.share_id != gone);
    cx.refresh_windows();
    crate::collab::run(
        cx,
        move |engine| engine.decline_book(&share_id),
        |result, cx| {
            if let Err(words) = result {
                notify_windows(move || Notification::error(words.clone()), cx);
                crate::collab::sync_soon(cx);
            }
        },
    );
}

// ------------------------------------------------------------------- live

/// Keeps the notebook's live room open (one per notebook, however many of
/// its notes are open) and names the note open in it.
pub fn open_note(share_id: &str, stream: &str, cx: &mut App) {
    let Some(engine) = crate::collab::current(cx) else {
        return;
    };
    let fresh = !cx
        .default_global::<Collab>()
        .book_rooms
        .contains_key(share_id);
    if fresh {
        let (tx, rx) = async_channel::unbounded::<BookLiveNotice>();
        let handle = Engine::go_live_book(
            &engine,
            share_id,
            Arc::new(move |notice| {
                tx.try_send(notice).ok();
            }),
        );
        cx.default_global::<Collab>()
            .book_rooms
            .insert(share_id.to_owned(), handle);
        cx.spawn(async move |cx| {
            while let Ok(first) = rx.recv().await {
                let mut batch = vec![first];
                while let Ok(more) = rx.try_recv() {
                    batch.push(more);
                }
                cx.update(|cx| on_live(batch, cx));
            }
        })
        .detach();
    }
    engine.book_open_note(share_id, stream);
}

/// The note closed in the editor; the room closes with the notebook's last
/// open note.
pub fn close_note(share_id: &str, stream: &str, cx: &mut App) {
    let open: Vec<(String, String)> = crate::editor::peers::open_editors(cx)
        .iter()
        .filter_map(|e| {
            e.read(cx)
                .book_note_open()
                .map(|(s, n)| (s.to_owned(), n.to_owned()))
        })
        .collect();
    // Another editor may show the same note, or another of the notebook's.
    if !open.iter().any(|(s, n)| s == share_id && n == stream)
        && let Some(engine) = crate::collab::current(cx)
    {
        engine.book_close_note(share_id, stream);
    }
    let still_open = open.iter().any(|(s, _)| s == share_id);
    if !still_open && cx.has_global::<Collab>() {
        let collab = cx.global_mut::<Collab>();
        collab.book_rooms.remove(share_id);
        collab.book_live.remove(share_id);
    }
}

/// What a notebook's room said.
fn on_live(batch: Vec<BookLiveNotice>, cx: &mut App) {
    let mut notes = false;
    let mut presence = false;
    let mut ended = false;
    for notice in batch {
        match notice {
            BookLiveNotice::Notes { .. } | BookLiveNotice::Text { .. } => notes = true,
            BookLiveNotice::Presence { .. } => presence = true,
            BookLiveNotice::Comments { .. } => {}
            BookLiveNotice::Connection { share_id, live } => {
                let collab = cx.default_global::<Collab>();
                if live {
                    collab.book_live.insert(share_id);
                    // Connected again: the status line catches up.
                    collab.offline = false;
                } else {
                    collab.book_live.remove(&share_id);
                }
                cx.refresh_windows();
            }
            BookLiveNotice::Ended { .. } => ended = true,
            BookLiveNotice::BroughtBack { titles, .. } => {
                notes = true;
                tell_brought_back(&titles, cx);
            }
        }
    }
    if notes {
        refresh_local(cx);
    }
    if presence {
        crate::workspace::for_each_editor(cx, |editor, cx| editor.refresh_book_cursors(cx));
    }
    if ended {
        crate::collab::sync_soon(cx);
    }
}

// ---------------------------------------------------------------- sidebar

/// The Shared section: the notebooks joined here, then the invitations.
/// Nothing while there are none.
// no-prop-test: drawing; `avatars`, `status_words` and `book_tone` carry
// the logic, and the Shared sidebar snapshots show the section.
pub fn render_section(
    sidebar: &Entity<Sidebar>,
    library: &Entity<Library>,
    selected: Option<&Filter>,
    cx: &App,
) -> Option<AnyElement> {
    let collab = cx.try_global::<Collab>()?;
    if collab.books.is_empty() && collab.book_invitations.is_empty() && !collab.book_update_needed {
        return None;
    }
    let colors = Colors::get(cx);
    let dark = cx.theme().is_dark();
    let mut section = v_flex().debug_selector(|| "shared-books".into());
    for (ix, row) in collab.books.iter().enumerate() {
        let view = &row.view;
        let book_id = view.book_id.clone();
        let filter = Filter::Book(book_id.clone());
        let is_selected = selected == Some(&filter);
        let tone = book_tone(collab.offline, row.waiting);
        let status = status_words(collab.offline, collab.syncing, row.waiting);
        let (shown, more) = avatars(&view.members, AVATARS);
        let faces = h_flex()
            .flex_none()
            .children(shown.iter().enumerate().map(|(i, member)| {
                div()
                    .size(px(17.))
                    .when(i > 0, |el| el.ml(px(-5.)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border_1()
                    .border_color(colors.sidebar)
                    .bg(cursor_colour(colour_of(&member.account), dark))
                    .text_size(px(9.5))
                    .font_semibold()
                    .text_color(colors.editor)
                    .child(initial(&member.username))
            }))
            .when(more > 0, |el| {
                el.child(
                    div()
                        .ml(px(3.))
                        .text_size(px(11.))
                        .text_color(colors.quiet(is_selected))
                        .child(format!("+{more}")),
                )
            });
        let glyph = match tone {
            BookTone::Offline => Some((Lucide::CloudOff, Hsla::from(colors.warning))),
            BookTone::Waiting => Some((Lucide::CloudUpload, colors.quiet(is_selected).into())),
            BookTone::UpToDate => None,
        };
        let library = library.clone();
        let sidebar = sidebar.clone();
        let share_book = book_id.clone();
        let name = if view.name.trim().is_empty() {
            "Untitled notebook".to_owned()
        } else {
            view.name.clone()
        };
        section = section.child(
            div()
                .id(("shared-book", ix))
                .debug_selector(move || format!("shared-book-{ix}"))
                .h(px(28.))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(10.))
                .mx_2()
                .pr(px(10.))
                .pl(px(10.))
                .rounded(px(6.))
                .text_size(px(13.5))
                .cursor_pointer()
                .when(is_selected, |el| el.bg(colors.sidebar_active))
                .when(!is_selected, |el| {
                    el.hover(|el| el.bg(colors.sidebar_hover))
                })
                .child(
                    Icon::new(Lucide::Users)
                        .size(px(15.))
                        .text_color(colors.text_secondary),
                )
                .child(div().flex_1().min_w_0().truncate().child(name))
                .child(faces)
                .children(glyph.map(|(icon, tint)| {
                    div()
                        .id(("shared-book-status", ix))
                        .child(Icon::new(icon).size(px(13.)).text_color(tint))
                }))
                .tooltip(move |window, cx| Tooltip::new(status.clone()).build(window, cx))
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| {
                        library.set_filter(Filter::Book(book_id.clone()), cx)
                    });
                })
                .context_menu(move |menu, _, _| {
                    let sidebar = sidebar.clone();
                    let id = share_book.clone();
                    menu.item(
                        PopupMenuItem::new("Share Notebook…").on_click(move |_, _, cx| {
                            sidebar
                                .update(cx, |_, cx| cx.emit(SidebarEvent::ShareBook(id.clone())));
                        }),
                    )
                }),
        );
    }
    // The status line: offline with what waits, or up to date.
    if !collab.books.is_empty() {
        let waiting: usize = collab.books.iter().map(|b| b.waiting).sum();
        section = section.child(
            h_flex()
                .mx_2()
                .pl(px(10.))
                .h(px(20.))
                .gap(px(6.))
                .text_size(px(11.5))
                .text_color(if collab.offline {
                    colors.warning.into()
                } else {
                    Hsla::from(colors.text_muted)
                })
                .debug_selector(|| "shared-books-status".into())
                .child(
                    Icon::new(if collab.offline {
                        Lucide::CloudOff
                    } else {
                        Lucide::CloudCheck
                    })
                    .size(px(12.)),
                )
                .child(status_words(collab.offline, collab.syncing, waiting)),
        );
    }
    if collab.book_update_needed {
        // An invitation this Kanso can't open: the server won't list one to a
        // client that's too old, so the section says what to do instead.
        section = section.child(
            h_flex()
                .id("book-update-needed")
                .debug_selector(|| "book-update-needed".into())
                .gap(px(10.))
                .mx_2()
                .mt(px(6.))
                .px(px(12.))
                .py(px(9.))
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(13.))
                .text_color(colors.text)
                .child(
                    Icon::new(Lucide::UserPlus)
                        .size(px(15.))
                        .text_color(colors.text_secondary),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(kanso_collab::UPDATE_FOR_INVITATION),
                ),
        );
    }
    for (ix, invitation) in collab.book_invitations.iter().enumerate() {
        let (accept_id, decline_id) = (invitation.share_id.clone(), invitation.share_id.clone());
        let invited = invitation.status == "invited";
        section = section.child(
            v_flex()
                .id(("book-invitation", ix))
                .debug_selector(move || format!("book-invitation-{ix}"))
                .gap(px(8.))
                .mx_2()
                .mt(px(6.))
                .px(px(12.))
                .py(px(9.))
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(13.))
                .text_color(colors.text)
                .child(
                    h_flex()
                        .gap(px(10.))
                        .child(
                            Icon::new(Lucide::UserPlus)
                                .size(px(15.))
                                .text_color(colors.text_secondary),
                        )
                        .child(div().flex_1().min_w_0().child(invitation_words(
                            &invitation.owner_username,
                            invitation.role,
                            &invitation.status,
                        ))),
                )
                .child(
                    h_flex()
                        .gap(px(6.))
                        .justify_end()
                        .when(invited, |el| {
                            el.child(
                                Button::new(("book-decline", ix))
                                    .ghost()
                                    .xsmall()
                                    .label("Decline")
                                    .debug_selector(move || format!("book-decline-{ix}"))
                                    .on_click(move |_, _, cx| decline(decline_id.clone(), cx)),
                            )
                        })
                        .child(
                            Button::new(("book-accept", ix))
                                .primary()
                                .xsmall()
                                .label(if invited { "Accept" } else { "Open here" })
                                .debug_selector(move || format!("book-accept-{ix}"))
                                .on_click(move |_, _, cx| accept(accept_id.clone(), cx)),
                        ),
                ),
        );
    }
    Some(section.into_any_element())
}

/// Which avatar colour an account gets: the same one its caret has.
fn colour_of(account: &str) -> usize {
    kanso_collab::engine::colour_of(account)
}
