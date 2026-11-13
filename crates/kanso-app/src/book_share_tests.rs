//! Shared notebooks in the app, against the in-memory server that follows
//! the Worker (issue #390 slice 38): the Share notebook sheet invites,
//! changes roles, removes, leaves and stops; the Shared section accepts and
//! declines invitations; the note list says who edited a note last; and a
//! notebook note shows who else is in it. The same steps run against a
//! local Worker in `book_share_e2e.rs`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_collab::memory::MemoryCollab;
use kanso_collab::{Engine, Role};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::book_share_model::BookStep;
use crate::book_share_sheet::BookShareSheet;
use crate::collab_model::Collab;
use crate::collab_tests::{Device, device};
use crate::config::AppConfig;
use crate::library::Filter;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

/// A notebook called `name` with two notes, in `store`. Its id and the
/// notes' ids.
pub(crate) fn studio(store: &Store, name: &str) -> (String, Vec<String>) {
    let book = store
        .put(
            DocType::Book,
            json!({ "name": name }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let notes = [("Brief", "Shoot on Friday"), ("Script", "Scene one")]
        .into_iter()
        .map(|(title, body)| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        })
        .collect();
    (book, notes)
}

/// The app's window over a library and its account's engine.
pub(crate) struct App {
    pub(crate) window: AnyWindowHandle,
    pub(crate) workspace: Entity<Workspace>,
    _data: tempfile::TempDir,
}

pub(crate) fn open_app(cx: &mut TestAppContext, store: Arc<Store>, engine: Arc<Engine>) -> App {
    cx.executor().allow_parking();
    let data = tempfile::tempdir().unwrap();
    let data_dir: PathBuf = data.path().to_path_buf();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        crate::collab::use_engine(engine, cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    App {
        window,
        workspace,
        _data: data,
    }
}

impl App {
    /// Opens the Share notebook sheet for `book`.
    pub(crate) fn sheet(&self, cx: &mut TestAppContext, book: &str) -> Entity<BookShareSheet> {
        let book = book.to_owned();
        let sheet = self
            .window
            .update(cx, |_, window, cx| {
                crate::book_share_sheet::open_book_share(book, window, cx)
            })
            .unwrap()
            .unwrap();
        settle(cx);
        sheet
    }

    pub(crate) fn on_sheet(
        &self,
        cx: &mut TestAppContext,
        sheet: &Entity<BookShareSheet>,
        f: impl FnOnce(
            &mut BookShareSheet,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<BookShareSheet>,
        ),
    ) {
        self.window
            .update(cx, |_, window, cx| {
                sheet.update(cx, |s, cx| f(s, window, cx))
            })
            .unwrap();
        settle(cx);
    }

    /// Types a username and presses Invite; a first share shows their
    /// fingerprint, which is marked verified, as a person would after
    /// comparing it.
    pub(crate) fn invite(
        &self,
        cx: &mut TestAppContext,
        sheet: &Entity<BookShareSheet>,
        username: &str,
        role: Role,
    ) {
        self.on_sheet(cx, sheet, |s, window, cx| {
            s.type_username(username, window, cx);
            s.role = role;
            s.invite(window, cx);
        });
        let step = cx.read(|cx| sheet.read(cx).step.clone());
        if let BookStep::Verify { person, role } = step {
            self.on_sheet(cx, sheet, |s, _, cx| {
                s.send_invite(person.username, role, true, cx)
            });
        }
    }

    pub(crate) fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// Syncs shared notes and notebooks as the app does in the background.
    pub(crate) fn sync(&self, cx: &mut TestAppContext) {
        cx.update(crate::collab::sync_soon);
        settle(cx);
    }

    pub(crate) fn collab<T>(&self, cx: &mut TestAppContext, f: impl FnOnce(&Collab) -> T) -> T {
        cx.read(|cx| f(cx.global::<Collab>()))
    }
}

/// Runs everything queued, including the engine's calls off the main thread.
pub(crate) fn settle(cx: &mut TestAppContext) {
    for _ in 0..3 {
        cx.run_until_parked();
    }
}

fn error_of(cx: &mut TestAppContext, sheet: &Entity<BookShareSheet>) -> Option<String> {
    cx.read(|cx| sheet.read(cx).error.as_ref().map(|e| e.to_string()))
}

fn done_of(cx: &mut TestAppContext, sheet: &Entity<BookShareSheet>) -> String {
    cx.read(|cx| sheet.read(cx).done.as_ref().map(|e| e.to_string()))
        .unwrap_or_default()
}

/// The usernames and roles the sheet lists.
fn people(cx: &mut TestAppContext, sheet: &Entity<BookShareSheet>) -> Vec<(String, Role)> {
    cx.read(|cx| {
        sheet
            .read(cx)
            .view
            .as_ref()
            .map(|v| {
                v.members
                    .iter()
                    .map(|m| (m.username.clone(), m.role))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The debug selector of a note's "Edited by" line.
fn selector(note: &str) -> &'static str {
    Box::leak(format!("note-edited-by-{note}").into_boxed_str())
}

fn account_of(d: &Device) -> String {
    d.engine.caller().account_id.clone()
}

fn share_of(d: &Device, book: &str) -> String {
    d.engine
        .shared_books()
        .unwrap()
        .into_iter()
        .find(|b| b.book_id == book)
        .unwrap()
        .share_id
}

fn epoch_of(d: &Device, share: &str) -> u32 {
    d.store.shared_book(share).unwrap().unwrap().epoch
}

/// The owner shares `book` with user2 at `role` through the sheet; user2
/// joins. The share's id.
fn share_through_sheet(
    cx: &mut TestAppContext,
    app: &App,
    owner: &Device,
    member: &Device,
    book: &str,
    role: Role,
) -> String {
    let sheet = app.sheet(cx, book);
    app.invite(cx, &sheet, "user2", role);
    assert_eq!(error_of(cx, &sheet), None);
    let share = share_of(owner, book);
    member.engine.join_book(&share).unwrap();
    share
}

#[gpui_kit::test]
fn sharing_a_notebook_from_the_sheet_invites_them(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    let sheet = app.sheet(cx, &book);
    assert_eq!(cx.read(|cx| sheet.read(cx).step.clone()), BookStep::Main);
    app.invite(cx, &sheet, "user2", Role::Edit);
    assert_eq!(error_of(cx, &sheet), None);
    assert!(done_of(cx, &sheet).starts_with("Shared \"Studio\" with user2"));
    assert_eq!(
        people(cx, &sheet),
        vec![("user1".into(), Role::Owner), ("user2".into(), Role::Edit)]
    );
    let invited = member.engine.book_invitations().unwrap();
    assert_eq!(invited.len(), 1);
    assert_eq!(invited[0].role, Role::Edit);
    // The sidebar's Shared section lists it, up to date.
    app.sync(cx);
    let rows = app.collab(cx, |c| c.books.len());
    assert_eq!(rows, 1);
}

#[gpui_kit::test]
fn inviting_more_people_keeps_the_key(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member, third) = (device(&server, 1), device(&server, 2), device(&server, 3));
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    let share = share_through_sheet(cx, &app, &owner, &member, &book, Role::Edit);
    let sheet = app.sheet(cx, &book);
    app.invite(cx, &sheet, "user3", Role::Comment);
    assert_eq!(error_of(cx, &sheet), None);
    assert!(done_of(cx, &sheet).starts_with("Invited user3"));
    assert_eq!(people(cx, &sheet).len(), 3);
    assert_eq!(epoch_of(&owner, &share), 1);
    let joined = third.engine.join_book(&share).unwrap();
    assert_eq!(joined.notes, 2);
}

#[gpui_kit::test]
fn a_role_changes_from_the_member_menu(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    let share = share_through_sheet(cx, &app, &owner, &member, &book, Role::Edit);
    let sheet = app.sheet(cx, &book);
    let them = account_of(&member);
    // Lowered: the notebook gets a new key.
    let who = them.clone();
    app.on_sheet(cx, &sheet, move |s, _, cx| s.set_role(who, Role::View, cx));
    assert_eq!(error_of(cx, &sheet), None);
    assert!(people(cx, &sheet).contains(&("user2".into(), Role::View)));
    assert_eq!(epoch_of(&owner, &share), 2);
    // Raised: the same key, a new wrap.
    let who = them.clone();
    app.on_sheet(cx, &sheet, move |s, _, cx| {
        s.set_role(who, Role::Manager, cx)
    });
    assert!(people(cx, &sheet).contains(&("user2".into(), Role::Manager)));
    assert_eq!(epoch_of(&owner, &share), 2);
    member.engine.sync_book(&share).unwrap();
    assert_eq!(
        member.engine.book_view(&share).unwrap().my_role,
        Some(Role::Manager)
    );
}

#[gpui_kit::test]
fn removing_someone_asks_then_changes_the_key(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    let share = share_through_sheet(cx, &app, &owner, &member, &book, Role::Edit);
    let sheet = app.sheet(cx, &book);
    let them = account_of(&member);
    let who = them.clone();
    app.on_sheet(cx, &sheet, move |s, _, cx| {
        s.ask_remove(who, "user2".into(), cx)
    });
    assert!(matches!(
        cx.read(|cx| sheet.read(cx).step.clone()),
        BookStep::ConfirmRemove { .. }
    ));
    app.on_sheet(cx, &sheet, move |s, _, cx| {
        s.remove(them, "user2".into(), cx)
    });
    assert_eq!(error_of(cx, &sheet), None);
    assert_eq!(people(cx, &sheet), vec![("user1".into(), Role::Owner)]);
    assert!(done_of(cx, &sheet).contains("new key"));
    assert_eq!(epoch_of(&owner, &share), 2);
    // They hear of it at their next sync, and keep a private copy.
    let sync = member.engine.sync_book(&share).unwrap();
    let ended = sync.ended.expect("the notebook ended for them");
    // Removed, or gone: the server answers both with the same 404.
    assert!(matches!(
        ended.reason,
        kanso_collab::BookEndReason::Removed | kanso_collab::BookEndReason::Gone
    ));
    assert!(ended.message.contains("Studio (no longer shared)"));
}

#[gpui_kit::test]
fn leaving_keeps_a_private_copy(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap()
        .share_id;
    let joined = member.engine.join_book(&share).unwrap();
    let app = open_app(cx, member.store.clone(), member.engine.clone());
    app.sync(cx);
    assert_eq!(app.collab(cx, |c| c.books.len()), 1);
    let sheet = app.sheet(cx, &joined.book_id);
    // A member sees who is in it, and has no invite field or role menus.
    assert_eq!(people(cx, &sheet).len(), 2);
    app.on_sheet(cx, &sheet, |s, _, cx| s.ask_end(cx));
    assert_eq!(
        cx.read(|cx| sheet.read(cx).step.clone()),
        BookStep::ConfirmEnd
    );
    app.on_sheet(cx, &sheet, |s, _, cx| s.end(cx));
    assert_eq!(error_of(cx, &sheet), None);
    assert!(done_of(cx, &sheet).contains("(no longer shared)"));
    assert_eq!(app.collab(cx, |c| c.books.len()), 0);
    let copy = member
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.id == joined.book_id)
        .unwrap();
    assert_eq!(copy.name, "Studio (no longer shared)");
    assert_eq!(
        member
            .store
            .notes_in_books(std::slice::from_ref(&joined.book_id))
            .unwrap()
            .len(),
        2
    );
}

#[gpui_kit::test]
fn stopping_sharing_ends_it_for_everyone(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    let share = share_through_sheet(cx, &app, &owner, &member, &book, Role::Edit);
    let sheet = app.sheet(cx, &book);
    app.on_sheet(cx, &sheet, |s, _, cx| s.ask_end(cx));
    app.on_sheet(cx, &sheet, |s, _, cx| s.end(cx));
    assert_eq!(error_of(cx, &sheet), None);
    assert!(done_of(cx, &sheet).contains("isn't shared any more"));
    assert!(cx.read(|cx| sheet.read(cx).share_id.is_none()));
    let sync = member.engine.sync_book(&share);
    let ended = match sync {
        Ok(sync) => sync.ended.is_some(),
        Err(_) => member
            .engine
            .shared_books()
            .unwrap()
            .iter()
            .all(|b| b.status != kanso_store::SharedStatus::Active),
    };
    assert!(ended, "the member's notebook ended");
}

#[gpui_kit::test]
fn an_invitation_is_accepted_from_the_shared_section(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(
            &book,
            &[("user2".into(), Role::Comment)],
            false,
            &mut |_| {},
        )
        .unwrap()
        .share_id;
    let app = open_app(cx, member.store.clone(), member.engine.clone());
    app.sync(cx);
    let invited = app.collab(cx, |c| c.book_invitations.clone());
    assert_eq!(invited.len(), 1);
    assert_eq!(invited[0].owner_username, "user1");
    app.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(app.window, cx);
    assert!(vcx.debug_bounds("book-invitation-0").is_some());
    vcx.update(|_, cx| crate::shared_books::accept(share.clone(), cx));
    settle(cx);
    assert!(app.collab(cx, |c| c.book_invitations.is_empty()));
    let rows = app.collab(cx, |c| c.books.clone());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].view.name, "Studio");
    assert_eq!(rows[0].view.my_role, Some(Role::Comment));
    assert_eq!(
        member
            .store
            .notes_in_books(&[rows[0].view.book_id.clone()])
            .unwrap()
            .len(),
        2
    );
}

#[gpui_kit::test]
fn an_invitation_is_declined_from_the_shared_section(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, _) = studio(&owner.store, "Studio");
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap()
        .share_id;
    let app = open_app(cx, member.store.clone(), member.engine.clone());
    app.sync(cx);
    assert_eq!(app.collab(cx, |c| c.book_invitations.len()), 1);
    cx.update(|cx| crate::shared_books::decline(share.clone(), cx));
    settle(cx);
    assert!(app.collab(cx, |c| c.book_invitations.is_empty()));
    assert!(member.engine.book_invitations().unwrap().is_empty());
    // The owner sees it declined, and the next cut takes them off the list.
    let view = owner.engine.book_view_fresh(&share).unwrap();
    let them = view.members.iter().find(|m| !m.me).unwrap();
    assert_eq!(them.standing, kanso_collab::MemberStanding::Left);
    assert_eq!(owner.engine.cut_left_members(&share).unwrap(), 1);
}

#[gpui_kit::test]
fn the_note_list_says_who_edited_a_note(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, notes) = studio(&owner.store, "Studio");
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap()
        .share_id;
    let joined = member.engine.join_book(&share).unwrap();
    let theirs = member
        .store
        .notes_in_books(std::slice::from_ref(&joined.book_id))
        .unwrap()
        .into_iter()
        .find(|n| n.title == "Brief")
        .unwrap()
        .id;
    member
        .store
        .put(
            DocType::Note,
            json!({"_id": theirs, "body": "Shoot on Saturday"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    member.engine.sync_book(&share).unwrap();
    // The owner's own edit to the other note says nothing.
    owner
        .store
        .put(
            DocType::Note,
            json!({"_id": notes[1], "body": "Scene two"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let app = open_app(cx, owner.store.clone(), owner.engine.clone());
    app.sync(cx);
    let edits = app.collab(cx, |c| c.book_edits.clone());
    let edit = edits.get(&notes[0]).expect("an edit by user2");
    assert_eq!(edit.name, "user2");
    assert!(!edit.mine);
    assert!(edits.get(&notes[1]).is_none_or(|e| e.mine));
    let library = cx.read(|cx| app.workspace.read(cx).library.clone());
    let id = book.clone();
    library.update(cx, |library, cx| library.set_filter(Filter::Book(id), cx));
    app.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(app.window, cx);
    assert!(
        vcx.debug_bounds(selector(&notes[0])).is_some(),
        "Edited by user2 shows under the note"
    );
    assert!(vcx.debug_bounds(selector(&notes[1])).is_none());
}

#[gpui_kit::test]
fn a_notebook_note_shows_who_is_in_it(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (book, notes) = studio(&owner.store, "Studio");
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap()
        .share_id;
    member.engine.join_book(&share).unwrap();
    // The owner is in the note, its caret after "Shoot".
    let stream = owner
        .store
        .book_stream_of_note(&notes[0])
        .unwrap()
        .unwrap()
        .stream;
    let notices: kanso_collab::engine::BookNotify = Arc::new(|_| {});
    let _room = Engine::go_live_book(&owner.engine, &share, notices);
    owner.engine.book_open_note(&share, &stream);
    owner.engine.set_book_caret(&share, &stream, 5, 5);
    // The member opens the same note in the app.
    let app = open_app(cx, member.store.clone(), member.engine.clone());
    let theirs = member
        .store
        .book_stream(&share, &stream)
        .unwrap()
        .unwrap()
        .note_id
        .unwrap();
    let library = cx.read(|cx| app.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| library.select(Some(theirs), cx));
    settle(cx);
    let editor = cx.read(|cx| app.workspace.read(cx).editor.clone());
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        settle(cx);
        let names = cx.read(|cx| editor.read(cx).book_cursors_for_test());
        if names == ["user1"] {
            break;
        }
        assert!(Instant::now() < deadline, "the owner's caret never showed");
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(app.collab(cx, |c| c.book_rooms.contains_key(&share)));
}
