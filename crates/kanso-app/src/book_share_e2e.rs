//! Shared notebooks in the app end to end, against a local Worker (issue
//! #390 slice 38). `cloud/scripts/book-share-e2e.sh` starts `wrangler dev
//! --local` with sign-ups open and runs:
//!
//! ```text
//! KANSO_BOOK_E2E_ENDPOINT=http://127.0.0.1:8787 \
//!     cargo test -p kanso-app book_share_e2e -- --ignored --test-threads=1
//! ```
//!
//! Real accounts are made (production key derivation), and the app's own
//! engine talks HTTP and the notebook's live socket to the Worker. The
//! owner's side drives the Share notebook sheet: share, invite, change a
//! role, remove, stop. The member's side drives the Shared section and the
//! sheet: accept, "Edited by", leave, decline. Every account name is new to
//! the run, and nothing but a server on this computer is talked to.

use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::TestAppContext;
use kanso_collab::{Account, Engine, MemberStanding, Role};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{DeviceIdentity, HttpBackend, MemorySecrets, Password};
use serde_json::json;

use crate::book_share_model::BookStep;
use crate::book_share_tests::{open_app, settle, studio};

const PASSWORD: &str = "correct horse battery book e2e";

/// The Worker to talk to: only one on this computer.
fn endpoint() -> Option<String> {
    let endpoint = std::env::var("KANSO_BOOK_E2E_ENDPOINT").ok()?;
    let rest = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .expect("KANSO_BOOK_E2E_ENDPOINT is an http(s) URL");
    let host = rest.split(['/', ':']).next().unwrap_or_default();
    assert!(
        host == "127.0.0.1" || host == "localhost",
        "refusing {endpoint}: these tests only talk to a Worker on this computer"
    );
    Some(endpoint.trim_end_matches('/').to_owned())
}

/// One account, signed up now, with its own library and engine over HTTP.
struct Peer {
    engine: Arc<Engine>,
    store: Arc<Store>,
    username: String,
    _dir: tempfile::TempDir,
}

fn sign_up(endpoint: &str, username: &str, n: u8) -> Peer {
    let password = Password::new(PASSWORD, username).unwrap();
    let device = DeviceIdentity::from_parts(&format!("l_{n:032x}"), &format!("Mac {n}"), [n; 32]);
    let mut tries = 0;
    let signed_up = loop {
        match kanso_sync::sign_up(
            &HttpBackend,
            &MemorySecrets::new(),
            endpoint,
            username,
            &password,
            &device,
            &|_| {},
        ) {
            Ok(done) => break done,
            // The Worker limits sign-ups per address: wait it out.
            Err(err) if format!("{err:?}").contains("RateLimited") && tries < 6 => {
                tries += 1;
                std::thread::sleep(Duration::from_secs(10));
            }
            Err(err) => panic!("signing {username} up: {err:?}"),
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let engine = Arc::new(Engine::new(
        store.clone(),
        Arc::new(kanso_collab::http::HttpCollab),
        Account::of(&signed_up.session),
    ));
    engine.identity().expect("identity keys published");
    Peer {
        engine,
        store,
        username: username.to_owned(),
        _dir: dir,
    }
}

/// Names no other run uses.
fn names(prefix: &str) -> impl Fn(&str) -> String {
    let run = format!("{:x}", kanso_core::now_ms());
    let prefix = prefix.to_owned();
    move |who| format!("{prefix}{run}{who}")
}

fn share_of(peer: &Peer, book: &str) -> String {
    peer.engine
        .shared_books()
        .unwrap()
        .into_iter()
        .find(|b| b.book_id == book && b.status == kanso_store::SharedStatus::Active)
        .expect("the notebook is shared")
        .share_id
}

/// Waits for `check`, settling the app between tries.
fn wait_until(
    cx: &mut TestAppContext,
    what: &str,
    mut check: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !check(cx) {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(200));
        settle(cx);
    }
}

#[gpui_kit::test]
#[ignore = "needs a local Worker: cloud/scripts/book-share-e2e.sh"]
fn the_owner_shares_invites_changes_removes_and_stops(cx: &mut TestAppContext) {
    let Some(endpoint) = endpoint() else {
        eprintln!("skipped: set KANSO_BOOK_E2E_ENDPOINT (see cloud/scripts/book-share-e2e.sh)");
        return;
    };
    let name = names("bo");
    let owner = sign_up(&endpoint, &name("a"), 41);
    let dan = sign_up(&endpoint, &name("b"), 42);
    let mira = sign_up(&endpoint, &name("c"), 43);
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());

    // Share it with dan from the sheet; the first share sends every note.
    let sheet = app.sheet(cx, &book);
    app.invite(cx, &sheet, &dan.username, Role::Edit);
    wait_until(cx, "the share", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    let share = share_of(&owner, &book);
    let joined = dan.engine.join_book(&share).unwrap();
    assert_eq!(joined.notes, 2);

    // Invite mira: the same key.
    app.invite(cx, &sheet, &mira.username, Role::Comment);
    wait_until(cx, "the invitation", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    assert_eq!(mira.engine.book_invitations().unwrap().len(), 1);
    let view = owner.engine.book_view_fresh(&share).unwrap();
    assert_eq!(view.epoch, 1);
    let mira_row = view
        .members
        .iter()
        .find(|m| m.username == mira.username)
        .unwrap();
    assert_eq!(mira_row.standing, MemberStanding::Invited);

    // Lower dan to view: a new key, which dan opens.
    let dan_id = dan.engine.caller().account_id.clone();
    let who = dan_id.clone();
    app.on_sheet(cx, &sheet, move |s, _, cx| s.set_role(who, Role::View, cx));
    wait_until(cx, "the role", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    assert_eq!(owner.engine.book_view(&share).unwrap().epoch, 2);
    dan.engine.sync_book(&share).unwrap();
    assert_eq!(
        dan.engine.book_view(&share).unwrap().my_role,
        Some(Role::View)
    );

    // Remove mira, who never joined: another key.
    let mira_id = mira.engine.caller().account_id.clone();
    let mira_name = mira.username.clone();
    app.on_sheet(cx, &sheet, move |s, _, cx| s.remove(mira_id, mira_name, cx));
    wait_until(cx, "the removal", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    assert_eq!(owner.engine.book_view(&share).unwrap().members.len(), 2);
    assert!(mira.engine.book_invitations().unwrap().is_empty());

    // Stop sharing: dan's copy is private now.
    app.on_sheet(cx, &sheet, |s, _, cx| s.ask_end(cx));
    assert_eq!(
        cx.read(|cx| sheet.read(cx).step.clone()),
        BookStep::ConfirmEnd
    );
    app.on_sheet(cx, &sheet, |s, _, cx| s.end(cx));
    wait_until(cx, "stopping", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    let ended = dan
        .engine
        .sync_book(&share)
        .map(|s| s.ended.is_some())
        .unwrap_or(true)
        || dan
            .engine
            .shared_books()
            .unwrap()
            .iter()
            .all(|b| b.status != kanso_store::SharedStatus::Active);
    assert!(ended, "the notebook ended for dan");
}

#[gpui_kit::test]
#[ignore = "needs a local Worker: cloud/scripts/book-share-e2e.sh"]
fn the_member_accepts_sees_edits_leaves_and_declines(cx: &mut TestAppContext) {
    let Some(endpoint) = endpoint() else {
        eprintln!("skipped: set KANSO_BOOK_E2E_ENDPOINT (see cloud/scripts/book-share-e2e.sh)");
        return;
    };
    let name = names("bm");
    let owner = sign_up(&endpoint, &name("a"), 51);
    let member = sign_up(&endpoint, &name("b"), 52);
    let (book, notes) = studio(&owner.store, "Studio");
    owner.engine.person(&member.username).unwrap();
    let share = owner
        .engine
        .share_book(
            &book,
            &[(member.username.clone(), Role::Edit)],
            false,
            &mut |_| {},
        )
        .unwrap()
        .share_id;
    let app = open_app(cx, member.store.clone(), member.engine.clone());

    // The invitation shows in the Shared section; accepting brings it in.
    app.sync(cx);
    wait_until(cx, "the invitation", |cx| {
        app.collab(cx, |c| c.book_invitations.len() == 1)
    });
    cx.update(|cx| crate::shared_books::accept(share.clone(), cx));
    wait_until(cx, "joining", |cx| app.collab(cx, |c| c.books.len() == 1));
    let joined = app.collab(cx, |c| c.books[0].view.book_id.clone());

    // The owner edits; the member's list says so, by the signed name.
    owner
        .store
        .put(
            DocType::Note,
            json!({"_id": notes[0], "body": "Shoot on Sunday"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    owner.engine.sync_book(&share).unwrap();
    let theirs = member
        .store
        .notes_in_books(std::slice::from_ref(&joined))
        .unwrap()
        .into_iter()
        .find(|n| n.title == "Brief")
        .unwrap()
        .id;
    let owner_name = owner.username.clone();
    wait_until(cx, "Edited by", |cx| {
        app.sync(cx);
        app.collab(cx, |c| {
            c.book_edits
                .get(&theirs)
                .is_some_and(|e| e.name == owner_name && !e.mine)
        })
    });
    assert_eq!(
        member.store.note(&theirs).unwrap().unwrap().body,
        "Shoot on Sunday"
    );

    // Leave from the sheet: a private copy stays.
    let sheet = app.sheet(cx, &joined);
    app.on_sheet(cx, &sheet, |s, _, cx| s.ask_end(cx));
    app.on_sheet(cx, &sheet, |s, _, cx| s.end(cx));
    wait_until(cx, "leaving", |cx| {
        cx.read(|cx| sheet.read(cx).busy.is_none())
    });
    assert_eq!(cx.read(|cx| sheet.read(cx).error.clone()), None);
    let copy = member
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.id == joined)
        .unwrap();
    assert!(copy.name.ends_with("(no longer shared)"));

    // A second notebook, declined from the Shared section.
    let (second, _) = studio(&owner.store, "Launch");
    let declined = owner
        .engine
        .share_book(
            &second,
            &[(member.username.clone(), Role::View)],
            false,
            &mut |_| {},
        )
        .unwrap()
        .share_id;
    app.sync(cx);
    wait_until(cx, "the second invitation", |cx| {
        app.collab(cx, |c| {
            c.book_invitations.iter().any(|b| b.share_id == declined)
        })
    });
    cx.update(|cx| crate::shared_books::decline(declined.clone(), cx));
    settle(cx);
    assert!(member.engine.book_invitations().unwrap().is_empty());
    let view = owner.engine.book_view_fresh(&declined).unwrap();
    let them = view.members.iter().find(|m| !m.me).unwrap();
    assert_eq!(them.standing, MemberStanding::Left);
}
