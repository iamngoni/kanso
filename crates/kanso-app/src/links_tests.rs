//! Linked knowledge through the real window: `[[` autocomplete, the
//! inspector's outline and backlinks, unresolved wiki-links that create
//! their note, the offer to update links after a rename, and ⌘P's headings.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::{AppConfig, InspectorTab};
use crate::editor::{EditorEvent, NoteEditor};
use crate::palette::Mode;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    book: String,
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        crate::workspace::bind_keys(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        book,
    }
}

impl Fixture {
    fn note(&self, title: &str, body: &str) -> String {
        self.store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": self.book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// Lets timers (the titles' refresh, autosave, searches) run out.
    fn settle(&self, cx: &mut TestAppContext) {
        for _ in 0..3 {
            cx.executor().advance_clock(Duration::from_millis(700));
            cx.run_until_parked();
        }
    }

    fn draw(&self, cx: &mut TestAppContext) {
        cx.run_until_parked();
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        self.settle(cx);
    }

    fn editor(&self, cx: &mut TestAppContext) -> Entity<NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    fn with_editor<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut NoteEditor, &mut gpui_kit::Window, &mut gpui_kit::Context<NoteEditor>) -> R,
    ) -> R {
        let editor = self.editor(cx);
        let out = self
            .window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| f(editor, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        out
    }

    fn with_workspace<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut Workspace, &mut gpui_kit::Window, &mut gpui_kit::Context<Workspace>) -> R,
    ) -> R {
        let workspace = self.workspace.clone();
        let out = self
            .window
            .update(cx, |_, window, cx| {
                workspace.update(cx, |workspace, cx| f(workspace, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        out
    }

    fn body(&self, cx: &mut TestAppContext) -> String {
        let editor = self.editor(cx);
        cx.read(|cx| editor.read(cx).body_for_test(cx))
    }

    fn open_note(&self, cx: &mut TestAppContext) -> Option<String> {
        let editor = self.editor(cx);
        cx.read(|cx| editor.read(cx).note_id().map(str::to_owned))
    }
}

/// `[[` offers notes by title as it is typed; choosing one writes an
/// ID link in place of the brackets, which undoes in one step. A
/// title no note has is offered as a new note, written as a wiki-link.
// no-prop-test: drives the editor with keys; `wikilinks` has the
// properties of the query and the candidates.
#[gpui_kit::test]
fn wiki_autocomplete_writes_a_note_link(cx: &mut TestAppContext) {
    let f = setup(cx);
    let budget = f.note("Budget 2026", "");
    f.note("Roadmap", "");
    let open = f.note("Draft", "");
    f.select(cx, &open);
    f.with_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.draw(cx);
    cx.simulate_input(f.window, "See [[bud");
    f.draw(cx);
    let labels = f.with_editor(cx, |editor, _, cx| editor.completion_labels_for_test(cx));
    assert_eq!(labels, ["Budget 2026", "New note “bud”"]);
    cx.simulate_keystrokes(f.window, "enter");
    f.draw(cx);
    let suffix = budget.strip_prefix("note:").unwrap();
    assert_eq!(
        f.body(cx),
        format!("See [Budget 2026](kanso://note/{suffix})")
    );
    cx.simulate_keystrokes(f.window, "cmd-z");
    f.draw(cx);
    assert_eq!(f.body(cx), "See [[bud]]");

    // A title no note has: the last entry writes the wiki-link.
    f.with_editor(cx, |editor, window, cx| {
        editor.edit_for_test("", window, cx)
    });
    cx.simulate_input(f.window, "[[Ideas");
    f.draw(cx);
    cx.simulate_keystrokes(f.window, "up enter");
    f.draw(cx);
    assert_eq!(f.body(cx), "[[Ideas]]");

    // Ordinary typing never opens the list.
    cx.simulate_input(f.window, " more words");
    f.draw(cx);
    assert!(
        f.with_editor(cx, |editor, _, cx| editor.completion_labels_for_test(cx))
            .is_empty()
    );
}

/// With Vim keys, `[[` completion works in Insert mode and Enter chooses;
/// the link with the rest of what was typed undoes as one step.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn wiki_autocomplete_in_vim_insert_mode(cx: &mut TestAppContext) {
    let f = setup(cx);
    let budget = f.note("Budget 2026", "");
    let open = f.note("Draft", "");
    f.select(cx, &open);
    f.with_editor(cx, |editor, window, cx| {
        editor.set_settings(
            crate::config::EditorSettings {
                vim: true,
                ..Default::default()
            },
            cx,
        );
        editor.focus_body(window, cx);
    });
    f.draw(cx);
    cx.simulate_keystrokes(f.window, "i");
    cx.simulate_input(f.window, "See [[bud");
    f.draw(cx);
    let labels = f.with_editor(cx, |editor, _, cx| editor.completion_labels_for_test(cx));
    assert_eq!(labels, ["Budget 2026", "New note “bud”"]);
    cx.simulate_keystrokes(f.window, "enter");
    f.draw(cx);
    let suffix = budget.strip_prefix("note:").unwrap();
    let linked = format!("See [Budget 2026](kanso://note/{suffix})");
    assert_eq!(f.body(cx), linked);
    assert_eq!(
        f.with_editor(cx, |editor, _, _| editor.vim_mode()),
        Some(crate::vim::Mode::Insert)
    );
    cx.simulate_keystrokes(f.window, "escape");
    f.draw(cx);
    assert_eq!(
        f.with_editor(cx, |editor, _, _| editor.vim_mode()),
        Some(crate::vim::Mode::Normal)
    );
    cx.simulate_keystrokes(f.window, "u");
    assert_eq!(f.body(cx), "");
    cx.simulate_keystrokes(f.window, "ctrl-r");
    assert_eq!(f.body(cx), linked);
}

/// The preview shows a resolved wiki-link as a link to its note and an
/// unresolved one as a `kanso-new:` link; following that link creates the
/// note with its title and opens it.
// no-prop-test: window glue; `rewrite_for_preview` has the properties.
#[gpui_kit::test]
fn unresolved_wiki_links_create_their_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    let target = f.note("Roadmap", "");
    let open = f.note("Plans", "[[roadmap]] and [[Someday List|later]]");
    f.with_workspace(cx, |workspace, _, cx| {
        workspace.set_view_mode_for_test(crate::config::ViewMode::Preview, cx)
    });
    f.select(cx, &open);
    let shown = f.with_editor(cx, |editor, _, cx| editor.preview_shown_for_test(cx));
    let suffix = target.strip_prefix("note:").unwrap();
    assert_eq!(
        shown,
        format!("[roadmap](kanso://note/{suffix}) and [later](kanso-new:Someday%20List)")
    );
    f.with_workspace(cx, |workspace, window, cx| {
        workspace.follow_link("kanso-new:Someday%20List".into(), window, cx)
    });
    f.settle(cx);
    let created = f.open_note(cx).unwrap();
    let note = f.store.note(&created).unwrap().unwrap();
    assert_eq!(note.title, "Someday List");
    assert_eq!(note.book_id, f.book);
    // Following it again opens the note rather than making another.
    f.select(cx, &open);
    f.with_workspace(cx, |workspace, window, cx| {
        workspace.follow_link("kanso-new:Someday%20List".into(), window, cx)
    });
    f.settle(cx);
    assert_eq!(f.open_note(cx), Some(created));
    let titles = f.store.note_titles().unwrap();
    assert_eq!(
        titles.iter().filter(|(_, t)| t == "Someday List").count(),
        1
    );
}

/// The inspector opens beside the editor and keeps its state; Outline
/// lists the headings as typed and takes the caret to one; Backlinks lists
/// the notes that link here and those that mention the title, and Link
/// makes a mention a link.
// no-prop-test: window glue; `current_heading`, the store's backlinks and
// `link_mention` have the properties.
#[gpui_kit::test]
fn the_inspector_shows_the_outline_and_backlinks(cx: &mut TestAppContext) {
    let f = setup(cx);
    let target = f.note("Roadmap", "# Goals\ntext\n## Next\nmore");
    let suffix = target.strip_prefix("note:").unwrap().to_owned();
    let linker = f.note("Linker", &format!("see [it](kanso://note/{suffix})"));
    let mention = f.note("Mention", "The roadmap slipped.");
    f.select(cx, &target);
    f.with_workspace(cx, |workspace, _, cx| workspace.toggle_inspector(cx));
    f.draw(cx);
    let open = cx.read(|cx| f.workspace.read(cx).inspector_open);
    assert!(open);
    let saved = AppConfig::load(&crate::paths::config_path(f._dir.path())).unwrap();
    assert!(saved.layout.inspector_open);

    // Outline: the headings, and a click on one moves the caret there.
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("inspector-pane").is_some());
    drop(vcx);
    f.with_editor(cx, |editor, window, cx| editor.reveal_line(2, window, cx));
    let caret = f.with_editor(cx, |editor, _, cx| editor.caret_line(cx));
    assert_eq!(caret, 2);

    // Backlinks: the linking note with its line, the mention below.
    f.with_workspace(cx, |workspace, _, cx| {
        workspace.show_inspector(Some(InspectorTab::Backlinks), cx)
    });
    f.draw(cx);
    f.settle(cx);
    f.draw(cx);
    let (linked, mentions) = cx.read(|cx| {
        let workspace = f.workspace.read(cx);
        let linked: Vec<String> = workspace
            .editor
            .read(cx)
            .backlinks()
            .iter()
            .map(|b| b.id.clone())
            .collect();
        let mentions = workspace.inspector.read(cx).mentions_for_test();
        (linked, mentions)
    });
    assert_eq!(linked, std::slice::from_ref(&linker));
    assert_eq!(mentions, std::slice::from_ref(&mention));
    let inspector = cx.read(|cx| f.workspace.read(cx).inspector.clone());
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.link_first_mention_for_test(window, cx)
            })
        })
        .unwrap();
    f.settle(cx);
    assert_eq!(
        f.store.note(&mention).unwrap().unwrap().body,
        format!("The [roadmap](kanso://note/{suffix}) slipped.")
    );
    let linked = f.with_editor(cx, |editor, _, _| editor.backlinks().len());
    assert_eq!(linked, 2);

    // Toggled off, the pane goes and stays gone.
    f.with_workspace(cx, |workspace, _, cx| workspace.toggle_inspector(cx));
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("inspector-pane").is_none());
}

/// The outline shows a note's headings as it opens; typing never waits on
/// parsing them again: the outline catches up once typing pauses.
// no-prop-test: timing glue; `outline::headings` has the properties.
#[gpui_kit::test]
fn the_outline_catches_up_after_typing_pauses(cx: &mut TestAppContext) {
    let f = setup(cx);
    let open = f.note("Plan", "# Goals\ntext\n");
    f.select(cx, &open);
    f.with_workspace(cx, |workspace, _, cx| workspace.toggle_inspector(cx));
    f.draw(cx);
    let inspector = cx.read(|cx| f.workspace.read(cx).inspector.clone());
    let outline = |cx: &mut TestAppContext| cx.read(|cx| inspector.read(cx).outline_for_test());
    assert_eq!(outline(cx), ["Goals"]);

    f.with_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.draw(cx);
    cx.simulate_keystrokes(f.window, "cmd-down");
    cx.simulate_input(f.window, "## Next");
    f.draw(cx);
    assert_eq!(outline(cx), ["Goals"], "parsed on the keystroke");
    cx.executor().advance_clock(Duration::from_millis(200));
    f.draw(cx);
    assert_eq!(outline(cx), ["Goals", "Next"]);
}

/// Renaming a note that wiki-links name by title offers to update them;
/// accepting rewrites them to the new title.
// no-prop-test: window glue; `retitle_wiki_links` has the properties.
#[gpui_kit::test]
fn renaming_offers_to_update_wiki_links(cx: &mut TestAppContext) {
    let f = setup(cx);
    let target = f.note("Old Name", "");
    let a = f.note("A", "see [[old name]] and [[Old Name#Part|p]]");
    f.note("B", "[[Elsewhere]]");
    f.select(cx, &target);
    let editor = f.editor(cx);
    let offers = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = offers.clone();
    let _watch = cx.update(|cx| {
        cx.subscribe(&editor, move |_, event: &EditorEvent, _| {
            if let EditorEvent::Renamed {
                old,
                new,
                links,
                notes,
                ..
            } = event
            {
                seen.lock()
                    .unwrap()
                    .push((old.clone(), new.clone(), *links, *notes));
            }
        })
    });
    // Focus events reach only an active window.
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.draw(cx);
    f.with_editor(cx, |editor, window, cx| editor.focus_title(window, cx));
    f.draw(cx);
    f.with_editor(cx, |editor, window, cx| {
        editor.set_title_for_test("New Name", window, cx)
    });
    f.draw(cx);
    f.with_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.draw(cx);
    f.settle(cx);
    assert_eq!(
        offers.lock().unwrap().clone(),
        [("Old Name".to_owned(), "New Name".to_owned(), 2, 1)]
    );
    assert_eq!(crate::workspace::relink_label(2), "Update 2 links");
    f.store
        .retitle_links("Old Name", "New Name", &target, Origin::Ui)
        .unwrap();
    assert_eq!(
        f.store.note(&a).unwrap().unwrap().body,
        "see [[New Name]] and [[New Name#Part|p]]"
    );
}

/// ⌘P finds headings too; choosing one opens its note with the caret on
/// the heading.
// no-prop-test: window glue; the store's heading search has the examples.
#[gpui_kit::test]
fn cmd_p_finds_headings(cx: &mut TestAppContext) {
    let f = setup(cx);
    let first = f.note("First", "");
    let guide = f.note("Guide", "intro\n\n## Installation steps\nrun it");
    f.select(cx, &first);
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.with_workspace(cx, |workspace, window, cx| {
        workspace
            .palette
            .update(cx, |palette, cx| palette.open(Mode::Notes, window, cx))
    });
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| {
                palette.set_query("installation", window, cx)
            })
        })
        .unwrap();
    f.settle(cx);
    f.draw(cx);
    let labels = cx.read(|cx| palette.read(cx).item_labels());
    assert!(
        labels.contains(&"Installation steps".to_owned()),
        "{labels:?}"
    );
    let ix = labels
        .iter()
        .position(|l| l == "Installation steps")
        .unwrap();
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| palette.choose_for_test(ix, window, cx))
        })
        .unwrap();
    f.settle(cx);
    assert_eq!(f.open_note(cx), Some(guide));
    let caret = f.with_editor(cx, |editor, _, cx| editor.caret_line(cx));
    assert_eq!(caret, 2);
}

/// Rows show a task pill: done of all, and nothing without tasks.
#[test]
fn task_pills() {
    use crate::note_list::task_pill;
    assert_eq!(task_pill(3, 7).as_deref(), Some("3/7"));
    assert_eq!(task_pill(0, 0), None);
    // A count the store never gives is capped rather than shown as more.
    assert_eq!(task_pill(9, 7).as_deref(), Some("7/7"));
}

proptest::proptest! {
    /// The pill reads `done/total` with done never above total, and shows
    /// only for notes with tasks.
    #[test]
    fn prop_task_pill_reads_done_of_total(done in 0u32..50, total in 0u32..50) {
        let pill = crate::note_list::task_pill(done, total);
        proptest::prop_assert_eq!(pill.is_some(), total > 0);
        if let Some(pill) = pill {
            let (d, t) = pill.split_once('/').unwrap();
            proptest::prop_assert_eq!(t.parse::<u32>().unwrap(), total);
            proptest::prop_assert!(d.parse::<u32>().unwrap() <= total);
        }
    }
}
