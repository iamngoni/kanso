//! A shared note open in the editor, against the in-memory server: another
//! member's edit merges in without moving the caret, typing here reaches
//! them through the engine, and a note shared to read stays read only.

use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_collab::memory::MemoryCollab;
use kanso_collab::{Account, Caller, Engine, Role};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;
use zeroize::Zeroizing;

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

pub(crate) struct Device {
    pub(crate) engine: Arc<Engine>,
    pub(crate) store: Arc<Store>,
    _dir: tempfile::TempDir,
}

pub(crate) fn device(server: &MemoryCollab, n: u8) -> Device {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let engine = engine_over(server, store.clone(), n, &format!("user{n}"));
    Device {
        engine,
        store,
        _dir: dir,
    }
}

/// The engine of account `n`, called `username`, over `store`, its
/// identity keys made.
pub(crate) fn engine_over(
    server: &MemoryCollab,
    store: Arc<Store>,
    n: u8,
    username: &str,
) -> Arc<Engine> {
    let hex = |prefix: &str, n: u8| format!("{prefix}_{}", format!("{n:02x}").repeat(16));
    let account = Account {
        caller: Caller {
            server: "https://sync.kanso.test".into(),
            account_id: hex("a", n),
            username: username.to_owned(),
            device_id: hex("d", n),
            authorization: Zeroizing::new("Bearer test".into()),
        },
        roots: kanso_sync::CollabRoots {
            identity_key: Zeroizing::new([n; 32]),
            vault_key: Zeroizing::new([n.wrapping_add(100); 32]),
        },
    };
    let engine = Arc::new(Engine::new(store, Arc::new(server.clone()), account));
    engine.identity().unwrap();
    engine
}

/// user1 shares "Hello world" with user2 at `role`; user2 accepts.
fn shared(owner: &Device, member: &Device, role: Role) -> (String, String) {
    shared_body(owner, member, role, "Hello world")
}

/// user1 shares a note of `body` with user2 at `role`; user2 accepts.
/// The share's ID and user2's note's.
pub(crate) fn shared_body(
    owner: &Device,
    member: &Device,
    role: Role,
    body: &str,
) -> (String, String) {
    let book = owner
        .store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let id = owner
        .store
        .put(
            DocType::Note,
            json!({"title": "Plan", "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share(&id, &[("user2".to_owned(), role)])
        .unwrap();
    member.engine.refresh().unwrap();
    member.engine.accept(&share).unwrap();
    let note = member.store.note_with_share(&share).unwrap().unwrap();
    (share, note)
}

struct App {
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
}

fn open(cx: &mut TestAppContext, member: &Device, note: &str) -> App {
    cx.executor().allow_parking();
    let (store, engine) = (member.store.clone(), member.engine.clone());
    let data_dir = member._dir.path().to_path_buf();
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
    let library = cx.read(|cx| workspace.read(cx).library.clone());
    let id = note.to_owned();
    library.update(cx, |library, cx| library.select(Some(id), cx));
    cx.run_until_parked();
    App { window, workspace }
}

impl App {
    fn editor(&self, cx: &mut TestAppContext) -> Entity<crate::editor::NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    fn body(&self, cx: &mut TestAppContext) -> String {
        let editor = self.editor(cx);
        cx.read(|cx| editor.read(cx).body_for_test(cx))
    }

    fn selection(&self, cx: &mut TestAppContext) -> std::ops::Range<usize> {
        let body = cx.read(|cx| self.workspace.read(cx).editor.read(cx).body_state());
        cx.read(|cx| body.read(cx).selected_range())
    }

    fn update(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(
            &mut crate::editor::NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<crate::editor::NoteEditor>,
        ),
    ) {
        let editor = self.editor(cx);
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |e, cx| f(e, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
    }
}

pub(crate) fn body_of(device: &Device, share: &str) -> String {
    let id = device.store.note_with_share(share).unwrap().unwrap();
    device.store.note(&id).unwrap().unwrap().body
}

#[gpui_kit::test]
fn shared_notes_merge_in_the_editor(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (share, note) = shared(&owner, &member, Role::Edit);
    let app = open(cx, &member, &note);
    assert_eq!(app.body(cx), "Hello world");
    let editor = app.editor(cx);
    let (role, footer) = cx.read(|cx| {
        let editor = editor.read(cx);
        (editor.shared_role(), editor.footer_share_for_test(cx))
    });
    assert_eq!(role, Some(Role::Edit));
    assert!(footer.unwrap().starts_with("Shared · Can edit"));

    // The caret after "Hello"; the owner writes before it.
    app.update(cx, |e, _, cx| e.select_for_test(5..5, cx));
    owner
        .store
        .put(
            DocType::Note,
            json!({"_id": owner.store.note_with_share(&share).unwrap().unwrap(), "body": "Hey! Hello world"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    owner.engine.sync_note(&share).unwrap();
    member.engine.sync_note(&share).unwrap();
    app.update(cx, |e, window, cx| e.collab_sync_for_test(window, cx));
    assert_eq!(app.body(cx), "Hey! Hello world");
    assert_eq!(app.selection(cx), 10..10, "the caret stayed after Hello");

    // Typing here reaches the owner through the engine.
    app.update(cx, |e, window, cx| {
        e.edit_for_test("Hey! Hello brave world", window, cx);
        e.collab_sync_for_test(window, cx);
    });
    for _ in 0..2 {
        member.engine.sync_note(&share).unwrap();
        owner.engine.sync_note(&share).unwrap();
    }
    assert_eq!(body_of(&owner, &share), "Hey! Hello brave world");
    assert_eq!(body_of(&member, &share), "Hey! Hello brave world");
}

#[gpui_kit::test]
fn a_note_shared_to_read_is_read_only(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (share, note) = shared(&owner, &member, Role::View);
    let app = open(cx, &member, &note);
    let editor = app.editor(cx);
    let footer = cx.read(|cx| editor.read(cx).footer_share_for_test(cx));
    assert!(footer.unwrap().contains("Read only"));
    // Whatever gets into the text is put back from the shared note.
    app.update(cx, |e, window, cx| {
        e.edit_for_test("Changed", window, cx);
        e.collab_sync_for_test(window, cx);
    });
    assert_eq!(app.body(cx), "Hello world");
    owner.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&owner, &share), "Hello world");
}

#[gpui_kit::test]
fn comments_list_in_the_inspector(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (share, note) = shared(&owner, &member, Role::Comment);
    // The owner comments on "world", mentioning user2.
    let theirs = owner
        .engine
        .add_comment(&share, 6..11, "Which one, @user2?")
        .unwrap();
    owner.engine.sync_note(&share).unwrap();
    member.engine.sync_note(&share).unwrap();
    let app = open(cx, &member, &note);
    let inspector = cx.read(|cx| app.workspace.read(cx).inspector.clone());
    let editor = app.editor(cx);
    app.window
        .update(cx, |_, _, cx| {
            app.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(crate::config::InspectorTab::Comments), cx)
            })
        })
        .unwrap();
    cx.run_until_parked();
    let listed = |cx: &mut TestAppContext| cx.read(|cx| inspector.read(cx).comments_for_test(cx));
    assert_eq!(listed(cx), [(theirs.clone(), false)]);
    let members = cx.read(|cx| {
        editor
            .read(cx)
            .shared_comments()
            .map(|(_, _, members)| members.to_vec())
            .unwrap()
    });
    assert!(members.contains(&"user2".to_owned()), "{members:?}");

    // Commenting here on "Hello" lists it first, in text order, pending
    // until it is sent.
    app.update(cx, |e, window, cx| {
        e.select_for_test(0..5, cx);
        e.comment_on_selection("Say hi", window, cx).unwrap();
    });
    let comments = cx.read(|cx| {
        editor
            .read(cx)
            .shared_comments()
            .map(|(_, comments, _)| comments.to_vec())
            .unwrap()
    });
    let mine = comments.iter().find(|v| v.mine).unwrap().clone();
    assert!(mine.pending);
    assert_eq!(
        listed(cx),
        [(mine.comment.id.clone(), false), (theirs.clone(), false)]
    );

    // Clicking the owner's comment selects its words.
    app.update(cx, |e, window, cx| e.select_comment(&theirs, window, cx));
    assert_eq!(app.selection(cx), 6..11);

    // Edit this account's own; resolve the owner's and open it again.
    let state = |cx: &mut TestAppContext, id: &str| {
        cx.read(|cx| {
            editor
                .read(cx)
                .shared_comments()
                .and_then(|(_, comments, _)| comments.iter().find(|v| v.comment.id == id).cloned())
        })
    };
    app.update(cx, |e, _, cx| {
        e.edit_comment(&mine.comment.id, "Say hello", cx).unwrap()
    });
    assert_eq!(
        state(cx, &mine.comment.id).unwrap().comment.body,
        "Say hello"
    );
    app.update(cx, |e, _, cx| e.resolve_comment(&theirs, true, cx).unwrap());
    assert!(state(cx, &theirs).unwrap().comment.resolved);
    app.update(cx, |e, _, cx| {
        e.resolve_comment(&theirs, false, cx).unwrap()
    });
    assert!(!state(cx, &theirs).unwrap().comment.resolved);

    // The owner deletes "world": their comment is orphaned, listed last.
    member.engine.sync_note(&share).unwrap();
    owner
        .store
        .put(
            DocType::Note,
            json!({"_id": owner.store.note_with_share(&share).unwrap().unwrap(), "body": "Hello"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    owner.engine.sync_note(&share).unwrap();
    member.engine.sync_note(&share).unwrap();
    app.update(cx, |e, window, cx| {
        e.collab_sync_for_test(window, cx);
        e.refresh_comments(cx);
    });
    assert_eq!(app.body(cx), "Hello");
    assert_eq!(
        listed(cx),
        [(mine.comment.id.clone(), false), (theirs.clone(), true)]
    );

    // Deleting this account's own comment takes it off the list; the
    // owner sees what was done.
    app.update(cx, |e, _, cx| {
        e.delete_comment(&mine.comment.id, cx).unwrap()
    });
    assert_eq!(listed(cx), [(theirs.clone(), true)]);
    member.engine.sync_note(&share).unwrap();
    owner.engine.sync_note(&share).unwrap();
    let seen = owner.engine.comments(&share).unwrap();
    assert_eq!(seen.len(), 1);
    assert!(!seen[0].comment.resolved);
}

#[gpui_kit::test]
fn an_invitation_can_be_verified_before_accepting(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let book = owner
        .store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let id = owner
        .store
        .put(
            DocType::Note,
            json!({"title": "Plan", "body": "Hello", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    owner.engine.person("user2").unwrap();
    owner
        .engine
        .share(&id, &[("user2".to_owned(), Role::Comment)])
        .unwrap();
    member.engine.refresh().unwrap();
    let invitations = member.engine.invitations();
    assert_eq!(invitations.len(), 1);
    let owner_person = invitations[0].owner.clone();
    assert!(!owner_person.verified);
    // The fingerprint shown is the one pinned for the owner's keys.
    assert_eq!(
        crate::collab_text::fingerprint_groups(&owner_person.fingerprint).len(),
        6
    );
    let engine = member.engine.clone();
    cx.update(|cx| {
        crate::collab::use_engine(engine, cx);
        cx.global_mut::<crate::collab_model::Collab>().invitations = invitations;
        crate::collab::verify_owner(owner_person.account_id.clone(), cx);
    });
    cx.run_until_parked();
    let shown = cx.read(|cx| {
        cx.global::<crate::collab_model::Collab>().invitations[0]
            .owner
            .verified
    });
    assert!(shown, "the invitation says Verified at once");
    assert!(
        member
            .engine
            .person_of(&owner_person.account_id)
            .unwrap()
            .verified
    );
}

#[gpui_kit::test]
fn a_refused_edit_shows_its_problem(cx: &mut TestAppContext) {
    use kanso_collab::engine::STATE_LIMIT;

    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (_, note) = shared(&owner, &member, Role::Edit);
    let app = open(cx, &member, &note);
    let editor = app.editor(cx);
    let footer =
        |cx: &mut TestAppContext| cx.read(|cx| editor.read(cx).footer_share_for_test(cx).unwrap());
    assert!(!footer(cx).contains(crate::collab_text::NEAR_LIMIT));

    // Grown close to the limit, in four-byte characters so the text stays
    // within a note's length: the footer says it is near.
    let near = "😀".repeat(1_030_000);
    assert!(near.len() * 5 >= STATE_LIMIT * 4 && near.len() < STATE_LIMIT);
    app.update(cx, |e, window, cx| {
        e.edit_for_test(&near, window, cx);
        e.collab_sync_for_test(window, cx);
    });
    assert!(
        footer(cx).contains(crate::collab_text::NEAR_LIMIT),
        "{}",
        footer(cx)
    );
    let early = cx.read(|cx| editor.read(cx).shared_problems().to_vec());
    assert!(early.is_empty(), "{early:?}");

    // Past the limit: the edit is refused, and the banner says why.
    let past = format!("{near}{}", "😀".repeat(18_000));
    assert!(past.len() > STATE_LIMIT);
    assert!(past.chars().count() <= kanso_core::model::MAX_BODY_LEN);
    app.update(cx, |e, window, cx| {
        e.edit_for_test(&past, window, cx);
        e.collab_sync_for_test(window, cx);
    });
    let problems = cx.read(|cx| editor.read(cx).shared_problems().to_vec());
    assert!(
        problems.iter().any(|p| p.message.contains("size limit")),
        "{problems:?}"
    );
    assert!(crate::collab_text::problem_banner(&problems).is_some());

    // Dismissed, it goes.
    app.update(cx, |e, _, cx| e.dismiss_problems(cx));
    assert!(cx.read(|cx| editor.read(cx).shared_problems().is_empty()));
}

fn canvas_pen(points: &[(f64, f64)]) -> kanso_core::canvas::Stroke {
    kanso_core::canvas::Stroke {
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 2.0,
        points: points
            .iter()
            .map(|&(x, y)| kanso_core::canvas::Point::new(x, y, 0.5))
            .collect(),
        ..Default::default()
    }
}

#[gpui_kit::test]
fn a_shared_canvas_takes_the_others_drawing_and_sends_its_own(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let book = owner
        .store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let id = owner
        .store
        .put(
            DocType::Note,
            json!({
                "title": "Board",
                "bookId": book,
                "doctype": "canvas",
                "body": kanso_core::canvas::to_body(&kanso_core::canvas::Canvas::default()),
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share(&id, &[("user2".to_owned(), Role::Edit)])
        .unwrap();
    member.engine.refresh().unwrap();
    member.engine.accept(&share).unwrap();
    let note = member.store.note_with_share(&share).unwrap().unwrap();
    let app = open(cx, &member, &note);
    let editor = app.editor(cx);
    let view = cx.read(|cx| editor.read(cx).canvas_for_test());
    let strokes = |cx: &mut TestAppContext| {
        view.update(cx, |v, _| {
            v.state_for_test()
                .map_or(0, |s| s.canvas.live_strokes().count())
        })
    };
    assert_eq!(strokes(cx), 0);

    // The owner draws; the room says so; the member's canvas takes it.
    owner.engine.attach(&share).unwrap();
    let mut mine = owner.engine.canvas(&share).unwrap();
    mine.draw(canvas_pen(&[(0.0, 0.0), (30.0, 30.0)]), 1)
        .unwrap();
    owner.engine.sync_canvas(&share, &mine).unwrap();
    owner.engine.sync_note(&share).unwrap();
    member.engine.sync_note(&share).unwrap();
    app.update(cx, |e, _, cx| e.canvas_sync_for_test(cx));
    assert_eq!(strokes(cx), 1);

    // The member draws; it reaches the owner through the engine.
    view.update(cx, |v, _| {
        v.state_for_test()
            .unwrap()
            .canvas
            .draw(canvas_pen(&[(50.0, 0.0), (80.0, 30.0)]), 2)
            .unwrap();
    });
    app.update(cx, |e, _, cx| e.canvas_sync_for_test(cx));
    for _ in 0..2 {
        member.engine.sync_note(&share).unwrap();
        owner.engine.sync_note(&share).unwrap();
    }
    let seen = owner.engine.sync_canvas(&share, &mine).unwrap().canvas;
    assert_eq!(seen.live_strokes().count(), 2);
    // The text editor stays out of it.
    let editor = app.editor(cx);
    assert!(cx.read(|cx| !editor.read(cx).has_unsaved_edits(cx)));
}
