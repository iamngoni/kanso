//! Shared notes, the app's side (docs/ACCOUNTS.md › Collaboration): the
//! signed-in account's engine, made for its session the first time it is
//! needed and dropped when the session goes; the background sync of shared
//! notes, invitations and mentions on the account sync's cadence; joining
//! by link; and what errors mean to the person sharing.
//!
//! The engine's calls wait on the network, so they run off the main thread
//! and their news comes back to [`Collab`]. Offline, nothing is said: the
//! next sync tries again, and an open note's live room keeps trying on its
//! own.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_collab::{Account, CollabError, Engine, Invitation};
use kanso_store::StoredNotice;

use crate::collab_model::Collab;
use crate::library::Library;
use crate::services::{Services, notify_windows};
use crate::workspace::AppState;

actions!(
    kanso,
    [ShareNote, JoinSharedNote, CommentOnSelection, ShowComments]
);

/// The engine of the signed-in account, made now if the session is new.
/// `None` while signed out, and in tests and snapshots without a server.
pub fn engine(cx: &mut App) -> Option<Arc<Engine>> {
    let session = cx.try_global::<Services>().and_then(Services::session);
    let api = cx.try_global::<Services>().and_then(|s| s.collab.clone());
    let store = cx.try_global::<AppState>().map(|s| s.store.clone());
    // The local API changes shared canvases through this engine.
    let host = cx
        .try_global::<Services>()
        .map(|s| s.shared_canvases.clone());
    let collab = cx.default_global::<Collab>();
    #[cfg(test)]
    if collab.pinned {
        return collab.engine.clone();
    }
    match (session, api, store) {
        (Some(session), Some(api), Some(store)) => {
            let same = collab
                .session
                .as_ref()
                .is_some_and(|s| Arc::ptr_eq(s, &session));
            if !same {
                let engine = Arc::new(Engine::new(store, api, Account::of(&session)));
                if let Some(host) = &host {
                    host.set(Some(engine.clone()));
                }
                *collab = Collab {
                    engine: Some(engine),
                    session: Some(session),
                    ..Collab::default()
                };
            }
            collab.engine.clone()
        }
        _ => {
            if collab.engine.is_some() {
                *collab = Collab::default();
            }
            if let Some(host) = &host {
                host.set(None);
            }
            None
        }
    }
}

/// The engine already made for the signed-in session, without making one.
pub fn current(cx: &App) -> Option<Arc<Engine>> {
    let collab = cx.try_global::<Collab>()?;
    #[cfg(test)]
    if collab.pinned {
        return collab.engine.clone();
    }
    let session = cx.try_global::<Services>().and_then(Services::session)?;
    collab
        .session
        .as_ref()
        .is_some_and(|s| Arc::ptr_eq(s, &session))
        .then(|| collab.engine.clone())
        .flatten()
}

/// The session went: the engine goes with it, and open shared notes close
/// their rooms.
pub fn signed_out(cx: &mut App) {
    #[cfg(test)]
    if cx.try_global::<Collab>().is_some_and(|c| c.pinned) {
        return;
    }
    if let Some(services) = cx.try_global::<Services>() {
        services.shared_canvases.set(None);
    }
    if cx.try_global::<Collab>().is_some() {
        cx.set_global(Collab::default());
    }
    // A window can't be reached from inside its own update.
    cx.defer(|cx| {
        crate::workspace::for_each_editor(cx, |editor, cx| editor.collab_signed_out(cx));
    });
}

/// Syncs shared notes now, on a background thread: invitations, notes
/// shared with this account, every shared note's log, and mentions. A
/// request while one runs runs after it.
pub fn sync_soon(cx: &mut App) {
    let Some(engine) = engine(cx) else {
        return;
    };
    let collab = cx.global_mut::<Collab>();
    if collab.syncing {
        collab.again = true;
        return;
    }
    collab.syncing = true;
    let task = cx.background_executor().spawn({
        let engine = engine.clone();
        async move {
            let result = engine.sync_all();
            // Shared notebooks after shared notes: one's trouble doesn't
            // stop the other.
            let books = crate::shared_books::sync_books(&engine);
            let fingerprint = engine.fingerprint().ok();
            let notices = engine.notices().unwrap_or_default();
            (result, fingerprint, engine.invitations(), notices, books)
        }
    });
    cx.spawn(async move |cx| {
        let (result, fingerprint, invitations, notices, books) = task.await;
        cx.update(|cx| {
            finish_sync(
                &engine,
                result,
                fingerprint,
                invitations,
                notices,
                books,
                cx,
            )
        });
    })
    .detach();
}

#[allow(clippy::too_many_arguments)]
fn finish_sync(
    engine: &Arc<Engine>,
    result: kanso_collab::api::Result<Vec<String>>,
    fingerprint: Option<String>,
    invitations: Vec<Invitation>,
    notices: Vec<StoredNotice>,
    books: crate::book_share_model::BookNews,
    cx: &mut App,
) {
    let current = current(cx).is_some_and(|e| Arc::ptr_eq(&e, engine));
    let collab = cx.default_global::<Collab>();
    collab.syncing = false;
    let again = std::mem::take(&mut collab.again);
    if !current {
        return;
    }
    match &result {
        Ok(_) => collab.offline = false,
        Err(err) if err.is_offline() => collab.offline = true,
        Err(err) => {
            collab.offline = false;
            eprintln!("kanso: syncing shared notes: {err}");
        }
    }
    if fingerprint.is_some() {
        collab.fingerprint = fingerprint;
    }
    // Invitations as last refreshed: an offline sync keeps the ones shown.
    if result.is_ok() {
        collab.invitations = invitations;
    }
    collab.notices = notices;
    if books.offline {
        collab.offline = true;
    }
    crate::shared_books::take_news(books, cx);
    // Comments the sync brought show in the open note.
    crate::workspace::for_each_editor(cx, |editor, cx| {
        editor.refresh_comments(cx);
        editor.refresh_problems(cx);
    });
    cx.refresh_windows();
    if again {
        sync_soon(cx);
    }
}

/// Syncs shared notes every `interval` while the task lives.
pub fn periodic(interval: Duration, cx: &mut App) -> Task<()> {
    cx.spawn(async move |cx| {
        loop {
            cx.update(sync_soon);
            cx.background_executor().timer(interval).await;
        }
    })
}

/// Runs `work` with the engine off the main thread, then `done` with what
/// it returned. Without an engine (signed out), `done` gets an error.
pub fn run<T: Send + 'static>(
    cx: &mut App,
    work: impl FnOnce(&Engine) -> kanso_collab::api::Result<T> + Send + 'static,
    done: impl FnOnce(Result<T, String>, &mut App) + 'static,
) {
    let Some(engine) = engine(cx) else {
        done(
            Err("Sharing needs your Kanso account. Sign in first.".into()),
            cx,
        );
        return;
    };
    let task = cx
        .background_executor()
        .spawn(async move { work(&engine).map_err(|err| error_text(&err)) });
    cx.spawn(async move |cx| {
        let result = task.await;
        cx.update(|cx| done(result, cx));
    })
    .detach();
}

/// Accepts a note someone shared: it comes in with the next sync, which
/// starts now.
pub fn accept(share_id: String, cx: &mut App) {
    run(
        cx,
        move |engine| engine.accept(&share_id),
        |result, cx| {
            if let Err(words) = result {
                notify_windows(move || Notification::error(words.clone()), cx);
            }
            sync_soon(cx);
        },
    );
}

/// Marks the keys pinned for the owner of an invitation as compared and
/// verified; the invitation says so at once.
pub fn verify_owner(account_id: String, cx: &mut App) {
    let account = account_id.clone();
    run(
        cx,
        move |engine| engine.verify_person(&account),
        move |result, cx| match result {
            Ok(()) => {
                for invitation in &mut cx.default_global::<Collab>().invitations {
                    if invitation.owner.account_id == account_id {
                        invitation.owner.verified = true;
                    }
                }
                cx.refresh_windows();
            }
            Err(words) => notify_windows(move || Notification::error(words.clone()), cx),
        },
    );
}

/// Declines a note someone shared.
pub fn decline(share_id: String, cx: &mut App) {
    let gone = share_id.clone();
    cx.default_global::<Collab>()
        .invitations
        .retain(|i| i.share_id != gone);
    cx.refresh_windows();
    run(
        cx,
        move |engine| engine.decline(&share_id),
        |result, cx| {
            if let Err(words) = result {
                notify_windows(move || Notification::error(words.clone()), cx);
                sync_soon(cx);
            }
        },
    );
}

/// Asks for an invite link and asks to join the note it is for. The owner
/// lets the account in; the note then shows in Shared with me.
pub fn join_dialog(window: &mut Window, cx: &mut App) {
    crate::dialogs::prompt_text(
        "Join a shared note",
        "",
        "Ask to join",
        window,
        cx,
        |link, _, cx| join(link, cx),
    );
}

/// Asks to join the note an invite link is for.
pub fn join(link: String, cx: &mut App) {
    run(
        cx,
        move |engine| engine.join(link.trim()),
        |result, cx| match result {
            Ok(joined) => {
                let words = format!(
                    "Asked {} to let you in. The note shows in Shared with me once they approve.",
                    joined.owner_username
                );
                notify_windows(move || Notification::info(words.clone()), cx);
                sync_soon(cx);
            }
            Err(words) => notify_windows(move || Notification::error(words.clone()), cx),
        },
    );
}

/// Opens the note a mention is in, and marks the note's mentions read.
pub fn open_notice(notice: &StoredNotice, library: &Entity<Library>, cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    mark_read(Some(notice.share_id.clone()), cx);
    match store.note_with_share(&notice.share_id) {
        Ok(Some(id)) => library.update(cx, |library, cx| library.select(Some(id), cx)),
        _ => notify_windows(
            || Notification::info("That note isn't shared with you any more."),
            cx,
        ),
    }
}

/// Marks mentions read: one note's, or all of them.
pub fn mark_read(share_id: Option<String>, cx: &mut App) {
    let Some(engine) = engine(cx) else {
        return;
    };
    if let Err(err) = engine.mark_notices_read(share_id.as_deref()) {
        eprintln!("kanso: marking mentions read: {err}");
    }
    let collab = cx.default_global::<Collab>();
    for notice in &mut collab.notices {
        if share_id.as_ref().is_none_or(|s| *s == notice.share_id) {
            notice.read = true;
        }
    }
    cx.refresh_windows();
}

/// What a failed sharing step means, in plain words. Pure.
pub fn error_text(err: &CollabError) -> String {
    match err {
        CollabError::NoSuchUser(name) => format!(
            "There's no Kanso account called {name} that can share yet. They need to sign in to Kanso once first."
        ),
        CollabError::KeyChanged(name) => {
            format!("{name}'s keys changed since you last shared with them.")
        }
        CollabError::NotShareable(words) | CollabError::Refused(words) => words.clone(),
        CollabError::BookList(kanso_collab::BookListError::Forked) => {
            "The notebook's history forked: two different lists of people follow the same one, so Kanso stopped."
                .into()
        }
        CollabError::Api { .. } if err.is_client_too_old() => {
            kanso_collab::UPDATE_FOR_BOOKS.into()
        }
        CollabError::Transport(_) => {
            "Couldn't reach your sync server. Check your connection and try again.".into()
        }
        CollabError::Api { status: 401, .. } => {
            "Your account signed this device out. Sign in again to share.".into()
        }
        CollabError::Api { status: 403, .. } => "Your role in this note doesn't allow that.".into(),
        CollabError::Api { status: 404, .. } => {
            "Your sync server doesn't have that any more. It may have been stopped or cancelled."
                .into()
        }
        CollabError::Api { status: 409, .. } => {
            "Someone changed the note's sharing at the same time. Try again.".into()
        }
        CollabError::Api { status: 410, .. } => "That invite link has expired.".into(),
        CollabError::Api { status: 413, .. } => {
            "That's more than your sync server takes for a shared note.".into()
        }
        CollabError::Api { status: 429, .. } => {
            "Your sync server is busy. Try again in a minute.".into()
        }
        other => format!("Couldn't do that ({other})."),
    }
}

#[cfg(test)]
pub(crate) fn use_engine(engine: Arc<Engine>, cx: &mut App) {
    cx.set_global(Collab {
        engine: Some(engine),
        pinned: true,
        ..Collab::default()
    });
}

#[cfg(test)]
mod tests {
    use super::error_text;
    use kanso_collab::CollabError;
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn errors_read_as_sentences() {
        assert!(error_text(&CollabError::Transport("dns".into())).contains("connection"));
        assert!(error_text(&CollabError::NoSuchUser("bob".into())).contains("bob"));
        assert_eq!(
            error_text(&CollabError::NotShareable(
                "The note is in the trash.".into()
            )),
            "The note is in the trash."
        );
        assert!(error_text(&CollabError::api(429, "rate_limited", json!({}))).contains("busy"));
    }

    proptest! {
        /// Every error says something, as a sentence, and never shows the
        /// server's raw body.
        #[test]
        fn prop_every_error_is_a_sentence(status in 400u16..600, code in "[a-z_]{1,12}") {
            let words = error_text(&CollabError::api(status, &code, json!({"secret": "body"})));
            prop_assert!(words.ends_with('.'));
            prop_assert!(!words.contains("secret"));
        }
    }
}
