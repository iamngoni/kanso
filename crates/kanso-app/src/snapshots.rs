//! Renders the real views to PNG files with GPUI's Metal renderer, so the
//! UI can be reviewed without capturing the screen. Run with
//!
//! ```text
//! KANSO_SNAPSHOT_DIR=/tmp/shots [KANSO_SNAPSHOT_DB=copy-of-kanso.db] \
//!     [KANSO_SNAPSHOT_ONLY=dark-split,light-edit] \
//!     cargo test -p kanso-app snapshots -- --ignored --nocapture
//! ```
//!
//! Without `KANSO_SNAPSHOT_DB` a demo library is used, with the clock fixed
//! so every run draws the same picture. The database is copied first, so the
//! original is never written. Each shot also checks that the panes are
//! painted in their theme's surface colors.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, HeadlessAppContext, Pixels, Size, Window,
    WindowBounds, WindowOptions, px, size,
};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{
    DeviceInfo, Plan, Quota, QuotaKind, RecoveryKey, SignOutReason, SignupState, Step, SyncProblem,
    Usage,
};
use serde_json::json;

use crate::account_page::{AccountPage, SignedOutForm};
use crate::assets::KansoAssets;
use crate::clock::Clock;
use crate::commands::Picker;
use crate::config::{
    AppConfig, EditorFont, InspectorTab, ThemePreference, ThemeSettings, ViewMode,
};
use crate::editor::EditorEvent;
use crate::layout::{BOTTOM_BAND_HEIGHT, rail_width};
use crate::library::{Filter, has_notes_inside};
use crate::palette::{Mode, PANEL_HEIGHT, PANEL_WIDTH};
use crate::services::{
    AccountDetails, AccountMethodsState, AccountState, ApiState, PendingKey, ProviderWait,
    RunningTask, Services, SyncState, TaskKind,
};
use crate::settings::{NAV_WIDTH, SettingsPage};
use crate::theme::{Colors, DARK, LIGHT, Rgb};
use crate::updates::{Phase, Problem as UpdateProblem, Updates};
use crate::workspace::{AppState, Workspace, apply_theme};

mod ai;
mod discovery;
mod plugins_panel;
mod plugins_settings;
mod plugins_sheets;
mod voice;

/// The demo's "now": Thursday 28 May 2026, 09:45 in a zone two hours ahead
/// of UTC.
const DEMO_NOW_MS: i64 = 1_779_954_300_000;
const DEMO_UTC_OFFSET_SECS: i32 = 2 * 3600;
const MINUTE: i64 = 60_000;
const DAY: i64 = 24 * 60 * MINUTE;

const ROADMAP: &str = "# MCP support model\n\nKanso acts as an MCP host. Clients connect to Kanso, discover the tools it offers, and ask before they act. See the [protocol overview](https://modelcontextprotocol.io) for the wire format.\n\n## Goals\n\n- Enable secure, discoverable access to notes through MCP.\n- A first-party experience that is *simple* and **reliable**.\n- Protect notes with clear permissions and an activity log.\n- Work offline when possible; degrade gracefully.\n\n## Core components\n\n1. MCP server (in Kanso)\n2. Tool registry\n3. Permission manager\n\n## Example tool definition\n\n```json\n{\n  \"name\": \"search_notes\",\n  \"description\": \"Search notes by keyword\",\n  \"inputSchema\": {\n    \"type\": \"object\",\n    \"properties\": { \"query\": { \"type\": \"string\" }, \"limit\": { \"type\": \"integer\", \"default\": 20 } }\n  }\n}\n```\n\nEach tool declares its `inputSchema`; the registry validates calls against it.\n\n## Permissions\n\n- [x] User approves each new client\n- [x] Scopes are explicit\n- [ ] Always ask for write operations\n\n> Destructive actions need review unless a client is explicitly trusted.\n\n| Permission | Default |\n|---|---|\n| Read notes | Allowed |\n| Update notes | Ask |\n| Delete notes | Ask |\n";

/// The demo note other notes link to, so its header shows backlinks.
const LINKED_NOTE: &str = "note:mcp-support";

/// A demo library for when no real database is given, with edits spread
/// over today, yesterday and older days.
fn demo_store(path: &Path) -> Store {
    let store = Store::open(path).unwrap();
    let book = |name: &str, parent: Option<&str>| {
        store
            .put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let engineering = book("Engineering", None);
    let platform = book("Platform", Some(&engineering));
    let product = book("Product", None);
    let research = book("Research", None);
    let design = book("Design", None);
    // Two notebooks with icons of their own.
    for (id, icon) in [(&research, "flask-conical"), (&design, "palette")] {
        store
            .put(
                DocType::Book,
                json!({"_id": id, "kansoIcon": icon}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
    }
    let personal = book("Personal", None);
    let tag = |name: &str, color: &str| {
        store
            .put(
                DocType::Tag,
                json!({"name": name, "color": color}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let mcp = tag("mcp", "blue");
    let agents = tag("agents", "green");
    let security = tag("security", "red");
    let roadmap = tag("roadmap", "orange");
    let rust = tag("rust", "brown");
    let today = DEMO_NOW_MS;
    let notes = [
        (
            &engineering,
            "MCP support model",
            ROADMAP.to_owned(),
            vec![mcp.clone(), agents.clone(), security.clone()],
            "active",
            false,
            today - 4 * MINUTE,
        ),
        (
            &engineering,
            "Agent integration plan",
            "# Agent integration plan\n\nHow Kanso works with local agents and external tools; see the [MCP support model](inkdrop://note/mcp-support).\n\n- Read-only by default\n- Writes go through review\n".to_owned(),
            vec![agents.clone()],
            "active",
            false,
            today - 33 * MINUTE,
        ),
        (
            &platform,
            "Tool registry schema",
            "Schema and capabilities for discoverability, as the MCP support model lays out.\n\n```rust\npub struct Tool {\n    pub name: String,\n    pub schema: serde_json::Value,\n}\n```\n".to_owned(),
            vec![mcp, rust.clone()],
            "onHold",
            false,
            today - 51 * MINUTE,
        ),
        (
            &product,
            "Kanso roadmap",
            "# Kanso roadmap\n\nA calm, local-first notes app.\n\n## Next\n\n- [x] Wiki-links and backlinks\n- [x] Local API on port 19840\n- [ ] Command palette\n- [ ] Focus mode\n".to_owned(),
            vec![roadmap.clone()],
            "active",
            true,
            today - DAY - 3 * 60 * MINUTE,
        ),
        (
            &engineering,
            "Weekly sync notes",
            "# Weekly sync\n\n| Topic | Owner | Status | Due | Risk | Next step |\n|---|---|---|---|---|---|\n| Release | Ngoni | On track | Jun 4 | Notarization queue | Submit the build |\n| Docs | Team | Drafting | Jun 11 | Screenshots out of date | Retake them after the theme lands |\n\nAction items live in the tracker.".to_owned(),
            vec![],
            "none",
            false,
            today - DAY - 7 * 60 * MINUTE,
        ),
        (
            &product,
            "Release checklist",
            "---\ntitle: Release checklist\ndate: 2026-05-25\n---\n\n# Release checklist\n\n- [x] Bump the version\n- [ ] Notarize the build\n- [ ] Write the release notes\n".to_owned(),
            vec![roadmap],
            "completed",
            false,
            today - 3 * DAY,
        ),
        (
            &platform,
            "Sync design",
            "# Sync design\n\nDocuments live in D1, attachments in R2. Last writer wins by `updatedAt`; the loser stays in history. Clients reach it through the [MCP support model](inkdrop://note/mcp-support).".to_owned(),
            vec![rust, security],
            "completed",
            false,
            today - 4 * DAY,
        ),
        (
            &design,
            "Notes on a deliberately long title that has to truncate cleanly in the list",
            "Rows, not cards. The active row gets a subtle filled state.".to_owned(),
            vec![],
            "dropped",
            false,
            today - 8 * DAY,
        ),
        (
            &personal,
            "Reading list",
            "# Reading list\n\n1. *The Design of Everyday Things*\n2. **Shape Up**\n3. Crafting Interpreters\n".to_owned(),
            vec![],
            "none",
            false,
            today - 176 * DAY,
        ),
        (
            &TRASH_BOOK_ID.to_owned(),
            "Old meeting notes",
            "Superseded by the weekly sync.".to_owned(),
            vec![],
            "none",
            false,
            today - 30 * DAY,
        ),
    ];
    for (book, title, body, tags, status, pinned, updated) in notes {
        let mut note = json!({"title": title, "body": body, "bookId": book, "tags": tags,
            "status": status, "pinned": pinned,
            "createdAt": updated - 2 * DAY, "updatedAt": updated});
        if title == "MCP support model" {
            note["_id"] = json!(LINKED_NOTE);
        }
        store
            .put(DocType::Note, note, Put::Upsert, Origin::Ui)
            .unwrap();
    }
    store
}

pub(crate) fn context(clock: Option<Clock>) -> HeadlessAppContext {
    let mut cx = HeadlessAppContext::with_platform(
        Arc::new(gpui_apple::AppleTextSystem::new()),
        Arc::new(KansoAssets),
        gpui_kit::platform::current_headless_renderer,
    );
    cx.update(|cx| {
        gpui_kit::init(cx);
        // The app's keymap, so shortcuts show as bound.
        crate::workspace::bind_keys(cx);
        crate::note_list::bind_keys(cx);
        crate::icon_themes::init(cx);
        if let Some(clock) = clock {
            cx.set_global(clock);
        }
    });
    cx
}

/// Notes a shot made as it prepared (stamped with the real clock) moved
/// onto the demo's clock: a minute apart, the last made a minute before
/// the demo's "now", so every run draws the same dates.
fn pin_new_times(store: &Store, typing: Option<&str>) {
    let mut made: Vec<(i64, String)> = store
        .search_summaries(&Default::default())
        .unwrap()
        .into_iter()
        .filter(|note| note.updated_at > DEMO_NOW_MS || note.created_at > DEMO_NOW_MS)
        .filter(|note| Some(note.id.as_str()) != typing)
        .map(|note| (note.created_at, note.id))
        .collect();
    made.sort();
    let count = made.len() as i64;
    for (ix, (_, id)) in made.into_iter().enumerate() {
        let at = DEMO_NOW_MS - (count - ix as i64) * MINUTE;
        store
            .put(
                DocType::Note,
                json!({"_id": id, "createdAt": at, "updatedAt": at}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
    }
}

/// Puts the workspace in the shot's state before drawing.
type Prepare = fn(&mut HeadlessAppContext, AnyWindowHandle, &Entity<Workspace>, &Store);

/// What is left of the editor, for the surface checks.
#[derive(Clone, Copy, PartialEq)]
enum Left {
    /// The sidebar and the note list.
    Sidebar,
    /// The icon rail and the note list.
    Rail,
    /// Nothing: focus mode.
    Nothing,
}

struct Shot {
    name: &'static str,
    theme: ThemePreference,
    /// A UI theme other than Kanso's own, by name, and syntax and preview
    /// themes of their own.
    ui_theme: Option<&'static str>,
    syntax_theme: Option<&'static str>,
    preview_theme: Option<&'static str>,
    view_mode: ViewMode,
    /// The window's size in points; the PNG is twice that.
    size: Size<Pixels>,
    prepare: Prepare,
    left: Left,
    /// How wide a dialog in the shot is, for its surface check.
    dialog_width: f32,
    /// Where the dialog's top-padding surface is sampled, when it doesn't
    /// use the usual one-tenth-height placement.
    dialog_sample_y: Option<f32>,
}

/// The most recently edited note, as after a restart.
fn open_latest(_: &mut HeadlessAppContext, _: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {}

/// Three notes open as tabs, the newest showing, as after opening them from
/// the list.
fn open_tabs(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let ids: Vec<String> = store
        .search_summaries(&Default::default())
        .unwrap()
        .into_iter()
        .take(3)
        .map(|note| note.id)
        .collect();
    for id in ids.iter().skip(1) {
        cx.update_window(window, |_, window, cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.open_note_in_new_tab(id, window, cx)
            })
        })
        .unwrap();
    }
}

/// Engineering opened in the sidebar, showing Platform inside it.
fn notebook_open(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let engineering = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Engineering")
        .unwrap()
        .id;
    let sidebar = cx.update(|cx| workspace.read(cx).sidebar.clone());
    cx.update(|cx| sidebar.update(cx, |sidebar, cx| sidebar.toggle_book(&engineering, cx)));
}

/// The icon picker for a notebook with an icon of its own.
fn book_icon_picker(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::book_icons::open_picker("Design", Some("palette".into()), window, cx, |_, _, _| {});
    })
    .unwrap();
}

/// Draws with the icon theme `id`, as chosen in Settings.
fn use_icon_theme(cx: &mut HeadlessAppContext, id: &str) {
    let id = id.to_owned();
    cx.update(|cx| {
        crate::icon_themes::choose(Some(id), cx);
        cx.refresh_windows();
    });
}

/// The icon picker under Phosphor, searched for the theme's own icons.
fn book_icon_theme_section(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    use_icon_theme(cx, "phosphor");
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let picker = crate::book_icons::open_picker(
            "Ideas",
            Some("phosphor/planet".into()),
            window,
            cx,
            |_, _, _| {},
        );
        picker.update(cx, |picker, cx| picker.search_for("phosphor", window, cx));
    })
    .unwrap();
}

/// Settings › Icon Themes, with Phosphor chosen.
fn settings_icon_themes(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    use_icon_theme(cx, "phosphor");
    show_settings(cx, window, workspace, SettingsPage::IconThemes);
}

/// The app drawn with Phosphor's icons, Engineering open in the sidebar.
fn icon_theme_phosphor(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    use_icon_theme(cx, "phosphor");
    notebook_open(cx, window, workspace, store);
}

/// The icon picker searching for "money".
fn book_icon_search(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let picker = crate::book_icons::open_picker("Finance", None, window, cx, |_, _, _| {});
        picker.update(cx, |picker, cx| picker.search_for("money", window, cx));
    })
    .unwrap();
}

fn set_filter(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>, filter: Filter) {
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.set_filter(filter, cx)));
}

/// The Status group open on Active, while the API's port is taken (by
/// another app, say) and sync can't reach its server: the footer's warnings.
fn status_and_warnings(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_filter(
        cx,
        workspace,
        Filter::Status(kanso_core::NoteStatus::Active),
    );
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.api = ApiState::PortInUse(crate::config::DEFAULT_PORT);
        services.sync = SyncState::Failed(SyncProblem::Offline);
    });
}

/// Recent, with every tag showing and the Status group closed again.
fn recent_all_tags(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_filter(cx, workspace, Filter::Recent);
    cx.update(|cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| sidebar.show_all_tags(cx));
    });
}

/// The sidebar collapsed to its rail, the traffic lights in its top band.
fn collapse_sidebar(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_sidebar = false;
            cx.notify();
        })
    });
}

/// The rail's notebooks button: the palette lists the notebooks to go to.
fn rail_notebooks(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    cx.update_window(window, |_, _, cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |_, cx| {
            cx.emit(crate::sidebar::SidebarEvent::GoToNotebook)
        });
    })
    .unwrap();
}

/// The rail's tags button: the palette lists the tags to go to.
fn rail_tags(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    cx.update_window(window, |_, _, cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |_, cx| cx.emit(crate::sidebar::SidebarEvent::GoToTag));
    })
    .unwrap();
}

/// The pointer resting on the rail's compose button: its tooltip, with the
/// shortcut as bound.
fn rail_tooltip(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    cx.update(|cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| {
            sidebar.reveal_tooltip = Some("New note");
            cx.notify();
        });
    });
}

/// The pointer resting on the rail's button that opens the sidebar again.
fn rail_expand_tooltip(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    reveal_sidebar_tooltip(cx, workspace, "Toggle sidebar");
}

/// The pointer resting on the full sidebar's button that folds it to the
/// rail, at the end of its top band.
fn sidebar_toggle_tooltip(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    reveal_sidebar_tooltip(cx, workspace, "Toggle sidebar");
}

fn reveal_sidebar_tooltip(
    cx: &mut HeadlessAppContext,
    workspace: &Entity<Workspace>,
    label: &'static str,
) {
    cx.update(|cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| {
            sidebar.reveal_tooltip = Some(label);
            cx.notify();
        });
    });
}

/// The local API turned off: the footer's plug is unplugged, a shape of
/// its own rather than the failing one in grey.
fn api_off(cx: &mut HeadlessAppContext, _: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    cx.update(|cx| cx.global_mut::<Services>().api = ApiState::Off);
}

/// A note's history, the newest version selected. The demo keeps versions
/// of the open note from the last few days, each a section shorter; a real
/// library shows the most recently edited note that has earlier versions.
fn history(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let id = cx.update(|cx| {
        workspace
            .read(cx)
            .editor
            .read(cx)
            .note_id()
            .map(str::to_owned)
    });
    let Some(id) = id else {
        return;
    };
    if std::env::var_os("KANSO_SNAPSHOT_DB").is_some() {
        let with_history = store
            .search_summaries(&Default::default())
            .unwrap()
            .into_iter()
            .take(500)
            .find(|n| store.revisions(&n.id).is_ok_and(|r| !r.is_empty()));
        if let Some(note) = with_history {
            let library = cx.update(|cx| workspace.read(cx).library.clone());
            cx.update(|cx| library.update(cx, |library, cx| library.select(Some(note.id), cx)));
            cx.run_until_parked();
        }
    } else {
        let note = store.note(&id).unwrap().unwrap();
        let now = cx.update(|cx| Clock::now_ms(cx));
        // The body up to its `n`th section from the end.
        let without = |n: usize| {
            let starts: Vec<usize> = note.body.match_indices("\n## ").map(|(ix, _)| ix).collect();
            let cut = starts
                .len()
                .checked_sub(n)
                .map_or(note.body.len(), |ix| starts[ix]);
            note.body[..cut].to_owned()
        };
        let edited = note
            .body
            .replace("discover the tools it offers", "list its tools");
        let versions = [
            (3 * DAY, Origin::Import, without(3)),
            (DAY + 5 * 60 * MINUTE, Origin::Sync, without(2)),
            (2 * 60 * MINUTE, Origin::Api, without(1)),
            (12 * MINUTE, Origin::Ui, edited),
        ];
        for (n, (ago, origin, body)) in versions.into_iter().enumerate() {
            let fields = json!({"title": note.title, "body": body});
            store
                .record_revision_at(
                    &id,
                    &format!("{}-demo", n + 1),
                    fields.as_object().unwrap(),
                    origin,
                    now - ago,
                )
                .unwrap();
        }
    }
    // The dialog shows at rest, not partway through sliding in.
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| workspace.show_history(window, cx));
    })
    .unwrap();
}

/// The Publish sheet for the open note: signed out, signed in and not yet
/// published, published, and published but edited since.
fn publish_sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    signed_in: bool,
    published: Option<bool>,
) {
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        let services = cx.global_mut::<Services>();
        if signed_in {
            services.account = AccountState::SignedIn {
                username: DEMO_USER.into(),
            };
        } else {
            services.account = AccountState::SignedOut;
            services.sync = SyncState::Off;
        }
    });
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    if let Some(up_to_date) = published {
        cx.update(|cx| {
            let now = Clock::now_ms(cx);
            let source = editor.read(cx).publish_source(cx).unwrap();
            let body = if up_to_date {
                source.body.clone()
            } else {
                format!("{}\nAn earlier draft.", source.body)
            };
            let publish = kanso_sync::share::KansoPublish {
                share_id: "q3T9xW2LmN8vPz4RkY7aBw".into(),
                key: kanso_sync::share::ShareKey::from_bytes([0x5a; 32]).to_text(),
                published_at: now - 2 * DAY - 3 * 60 * MINUTE,
                published_rev: source.rev.clone(),
                expires_at: Some(now + 5 * DAY - 3 * 60 * MINUTE),
                content_hash: Some(kanso_sync::share::content_hash(&source.title, &body)),
                short_url: None,
                other: Default::default(),
            };
            editor.update(cx, |editor, cx| {
                editor.set_publish(&source.id, Some(publish), cx)
            });
        });
    }
    cx.update_window(window, |_, window, cx| {
        crate::publish::open_publish(editor, window, cx);
    })
    .unwrap();
}

/// Words of a note's prose to comment on: whole words of five letters or
/// more, outside headings, tables and code.
fn prose_words(body: &str) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut line_start = 0;
    let mut in_code = false;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code = !in_code;
        } else if !in_code && !trimmed.starts_with(['#', '|', '!', '>']) {
            let mut at = 0;
            for word in line.split(' ') {
                if word.len() >= 5 && word.chars().all(|c| c.is_ascii_alphabetic()) {
                    out.push(line_start + at..line_start + at + word.len());
                }
                at += word.len() + 1;
            }
        }
        line_start += line.len();
    }
    out
}

/// The open note shared by the demo account with user2, who can edit and
/// has commented, and user3, invited to comment and yet to accept. It has
/// comments on its words, one resolved and one whose words were deleted
/// since; the editor follows it as a shared note.
fn share_demo(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>) {
    use kanso_collab::Role;
    use kanso_collab::memory::MemoryCollab;

    use crate::collab_tests::{device, engine_over};

    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    let Some(id) = cx.update(|cx| editor.read(cx).note_id().map(str::to_owned)) else {
        return;
    };
    let server = MemoryCollab::new();
    let store = cx.update(|cx| cx.global::<AppState>().store.clone());
    let owner = engine_over(&server, store.clone(), 1, DEMO_USER);
    // The other members' devices live as long as the shot.
    let member = Box::leak(Box::new(device(&server, 2)));
    Box::leak(Box::new(device(&server, 3)));
    let body = store.note(&id).unwrap().unwrap().body;
    let words = prose_words(&body);
    if words.len() < 12 {
        return;
    }
    owner.person("user2").unwrap();
    owner.person("user3").unwrap();
    let share = owner
        .share(
            &id,
            &[
                ("user2".to_owned(), Role::Edit),
                ("user3".to_owned(), Role::Comment),
            ],
        )
        .unwrap();
    member.engine.refresh().unwrap();
    member.engine.accept(&share).unwrap();
    owner
        .add_comment(
            &share,
            words[1].clone(),
            "Is this still true since the move?",
        )
        .unwrap();
    let gone = words[words.len() / 2].clone();
    owner
        .add_comment(&share, gone.clone(), "Worth a link to the details here.")
        .unwrap();
    owner.sync_note(&share).unwrap();
    member.engine.sync_note(&share).unwrap();
    member
        .engine
        .add_comment(
            &share,
            words[4].clone(),
            "Yes, @ngoni, I checked it this morning.",
        )
        .unwrap();
    let fixed = member
        .engine
        .add_comment(&share, words[8].clone(), "A typo here, I think.")
        .unwrap();
    member.engine.sync_note(&share).unwrap();
    owner.sync_note(&share).unwrap();
    owner.resolve_comment(&share, &fixed, true).unwrap();
    // The words of one comment are deleted.
    let mut cut = body.clone();
    cut.replace_range(gone, "");
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": cut}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    owner.sync_note(&share).unwrap();
    owner.refresh().unwrap();
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        cx.global_mut::<Services>().account = AccountState::SignedIn {
            username: DEMO_USER.into(),
        };
        crate::collab::use_engine(owner, cx);
    });
    // The note shown again, as a shared note now.
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.select(None, cx)));
    cx.run_until_parked();
    cx.update(|cx| library.update(cx, |library, cx| library.select(Some(id), cx)));
    cx.run_until_parked();
}

/// Shared with me with an invitation waiting, its owner's fingerprint not
/// verified yet, and a mention unread.
fn shared_invitation(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    use kanso_collab::{Invitation, Person, Role};
    let now = cx.update(|cx| Clock::now_ms(cx));
    cx.update(|cx| {
        cx.global_mut::<Services>().account = AccountState::SignedIn {
            username: DEMO_USER.into(),
        };
        cx.set_global(crate::collab_model::Collab {
            invitations: vec![Invitation {
                share_id: "ieMcmzx2eV1orGMyH9iImw".into(),
                owner: Person {
                    account_id: "a_01".into(),
                    username: "mira".into(),
                    fingerprint: "40213 77810 05932 61174 39025 88461".into(),
                    verified: false,
                },
                role: Role::Comment,
            }],
            notices: vec![kanso_store::StoredNotice {
                id: "n1".into(),
                share_id: "s1".into(),
                comment_id: "c1".into(),
                from_id: "a_02".into(),
                from_name: "tendai".into(),
                title: "Launch plan".into(),
                excerpt: "@ngoni can you check the dates?".into(),
                at: now - 20 * MINUTE,
                read: false,
            }],
            ..Default::default()
        });
    });
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.set_filter(Filter::Shared, cx)));
}

/// The Share sheet for a shared note: its members, one still invited.
fn share_sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    share_demo(cx, workspace);
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    cx.update_window(window, |_, window, cx| {
        crate::share_sheet::open_share(editor, window, cx);
    })
    .unwrap();
}

/// A shared note's comments in the inspector, beside the words they tint.
fn comments(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    share_demo(cx, workspace);
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Comments), cx)
        })
    });
}

fn publish_signed_out(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    publish_sheet(cx, window, workspace, false, None);
}

fn publish_unpublished(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    publish_sheet(cx, window, workspace, true, None);
}

fn publish_published(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    publish_sheet(cx, window, workspace, true, Some(true));
}

fn publish_outdated(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    publish_sheet(cx, window, workspace, true, Some(false));
}

/// The Publish sheet for the open note's notebook, signed in: not yet
/// published, and published.
fn publish_site_sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
    published: bool,
) {
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        cx.global_mut::<Services>().account = AccountState::SignedIn {
            username: DEMO_USER.into(),
        };
    });
    let book = cx
        .update(|cx| workspace.read(cx).notebook_to_publish(cx))
        .expect("the open note has a notebook");
    if published {
        let now = cx.update(|cx| Clock::now_ms(cx));
        let plan = kanso_engine::publish::site::plan(store, &book).unwrap();
        let publish = kanso_sync::share::KansoPublish {
            share_id: "r4U0yX3MnO9wQa5SlZ8bCw".into(),
            key: kanso_sync::share::ShareKey::from_bytes([0x3c; 32]).to_text(),
            published_at: now - 3 * DAY,
            published_rev: String::new(),
            expires_at: None,
            content_hash: Some(kanso_sync::share::content_hash(
                &plan.name,
                &plan.fingerprint(),
            )),
            short_url: None,
            other: Default::default(),
        };
        kanso_engine::publish::site::store_book_publish(store, &book, Some(&publish)).unwrap();
    }
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            crate::publish_site::open_publish_site(workspace, &book, window, cx);
        });
    })
    .unwrap();
}

fn publish_site_unpublished(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    publish_site_sheet(cx, window, workspace, store, false);
}

fn publish_site_published(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    publish_site_sheet(cx, window, workspace, store, true);
}

/// The Obsidian fixture vault the importer's tests use.
fn import_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kanso-import/tests/fixtures/obsidian")
}

/// The Import window: choosing where notes are.
fn import_choose(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::library_import::start(window, cx);
    })
    .unwrap();
}

/// The Import window's summary before an Obsidian vault comes in.
fn import_preview(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let path = import_fixture();
    let report = kanso_import::library::preview(
        kanso_import::library::SourceKind::Obsidian,
        &path,
        store,
        |_| {},
    )
    .unwrap();
    cx.update_window(window, |_, window, cx| {
        crate::library_import::open_preview(
            kanso_import::library::SourceKind::Obsidian,
            PathBuf::from("~/Documents/Vault"),
            report,
            window,
            cx,
        );
    })
    .unwrap();
}

/// The Import window's report after the vault came in.
fn import_done(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let report = kanso_import::library::import_source(
        kanso_import::library::SourceKind::Obsidian,
        &import_fixture(),
        store,
        |_| {},
    )
    .unwrap();
    cx.update_window(window, |_, window, cx| {
        crate::library_import::open_done(report, window, cx);
    })
    .unwrap();
}

/// The rail while the Worker can't be reached: the sync icon's shape says
/// so, not only its color.
fn rail_offline(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    set_sync(cx, SyncState::Failed(SyncProblem::Offline));
}

/// The keyboard in the note list: its selected row wears the focus ring.
fn list_focused(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let list = workspace.read(cx).note_list.clone();
        list.update(cx, |list, cx| list.focus_list(window, cx));
    })
    .unwrap();
}

/// Focus mode, as after ⌘⇧D: the note alone, its header out of sight.
fn focus_mode(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.set_focus_mode(true, window, cx)
        });
    })
    .unwrap();
}

/// About 640 words of prose, for the word goal.
fn morning_pages() -> String {
    let paragraph = "The kettle clicks off and the house is still quiet. I write \
        before the day starts asking for things, a page or two about whatever \
        is on my mind, without stopping to fix the sentences. Some mornings it \
        is plans, some mornings it is nothing much, and that is fine too.";
    let words = paragraph.split_whitespace().count();
    let mut body = String::from("# Morning pages\n\n");
    for _ in 0..(640 / words) {
        body.push_str(paragraph);
        body.push_str("\n\n");
    }
    body
}

/// Focus mode on a note aiming for 1,000 words, about two thirds there:
/// the footer's ring and count.
fn focus_word_goal(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Morning pages", &morning_pages());
    cx.run_until_parked();
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.set_word_goal(crate::word_goal::GoalSetting::Words(1000), cx)
        });
    });
    focus_mode(cx, window, workspace, store);
}

/// The word goal list in the palette, with "1.5k" typed.
fn palette_word_goal(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::WordGoal, "1.5k");
}

/// A picture to attach: a soft diagonal gradient with a sun.
fn sample_picture(width: u32, height: u32) -> Vec<u8> {
    let image = image::RgbImage::from_fn(width, height, |x, y| {
        let (fx, fy) = (x as f32 / width as f32, y as f32 / height as f32);
        let (dx, dy) = (fx - 0.7, fy - 0.35);
        if dx * dx * 3.2 + dy * dy * 1.8 < 0.012 {
            return image::Rgb([250, 214, 120]);
        }
        let t = (fx + fy) / 2.;
        image::Rgb([
            (70. + 120. * t) as u8,
            (110. + 60. * (1. - fy)) as u8,
            (170. - 60. * t) as u8,
        ])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

/// Attached pictures in the preview: one set to the small width, one at
/// its own size, with the size buttons showing as on hover.
fn sized_images(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let small =
        crate::smart_paste::attach_image(store, &sample_picture(960, 540), "png", Some("Lake.png"))
            .unwrap();
    let full = crate::smart_paste::attach_image(
        store,
        &sample_picture(1200, 400),
        "png",
        Some("Ridge.png"),
    )
    .unwrap();
    let body = format!(
        "# Field notes\n\nThe lake at seven, before the wind came up.\n\n{}\n\nAnd the ridge from the road back, at its own size.\n\n{}\n",
        crate::smart_paste::sized_image(&format!("inkdrop://{small}"), "", Some(240)),
        crate::smart_paste::image_markdown(&full),
    );
    crate::preview::REVEAL_IMAGE_SIZES.store(true, std::sync::atomic::Ordering::Relaxed);
    open_new_note(cx, workspace, store, "Field notes", &body);
}

/// Math and diagrams: inline and display formulas, two Mermaid diagrams,
/// and the badges for a diagram and a formula that can't be drawn.
const RICH_NOTE: &str = "A circle of radius $r$ has area $\\pi r^2$, and an ellipse $\\pi a b$; it costs $5 and $10 to print.\n\n$$\n\\int_0^{2\\pi} \\frac{d\\theta}{1 + e\\cos\\theta} = \\frac{2\\pi}{\\sqrt{1-e^2}}\n$$\n\n```mermaid\nflowchart LR\n  Draft --> Review{Ready?}\n  Review -->|Yes| Publish\n  Review -->|No| Draft\n```\n\n```mermaid\nsequenceDiagram\n  Kanso->>Worker: Sealed note\n  Worker-->>Kanso: Stored\n```\n\n```mermaid\nsparkleDiagram\n  a --> b\n```\n\nAnd a slip: $\\frac{1}{$ stays as typed.\n";

/// Every element the export has to match the preview on. `{{PICTURE}}`
/// stands for an attached image sized with `|300`.
const PAGE_NOTE: &str = "# Page parity\n\nA paragraph with *emphasis*, **strong text**, `inline code`, a $x^2$ formula, and a [[Wiki link]].\n\n## Second heading\n\n### Third heading\n\n#### Fourth heading\n\n##### Fifth heading\n\n###### Sixth heading\n\n- First item\n- Second item\n  - Nested item\n  - Another nested item\n\n1. Ordered one\n2. Ordered two\n\n- [x] Done task\n- [ ] Open task\n\n> A quote that carries a thought across a line.\n>\n> And a second paragraph in it.\n\n| Name | Count | Note |\n|:-----|------:|:-----|\n| Alpha | 3 | First |\n| Beta | 12 | Second |\n\n```rust\nfn main() {\n    let greeting = \"hello\";\n    println!(\"{greeting}\");\n}\n```\n\n```python\ndef greet(name):\n    return f\"hello {name}\"\n```\n\n```javascript\nconst greet = (name) => `hello ${name}`;\n```\n\n```mermaid\nflowchart LR\n  Draft --> Review{Ready?}\n  Review -->|Yes| Publish\n  Review -->|No| Draft\n```\n\n$$\n\\int_0^1 x\\,dx = \\frac{1}{2}\n$$\n\n{{PICTURE}}\n\n---\n\nBack to [[Page parity#Second heading|the second heading]].\n";

fn page_preview(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let picture =
        crate::smart_paste::attach_image(store, &sample_picture(600, 300), "png", Some("Page.png"))
            .unwrap();
    let body = PAGE_NOTE.replace(
        "{{PICTURE}}",
        &format!("![A picture|300](inkdrop://{picture})"),
    );
    open_new_note(cx, workspace, store, "Page parity", &body);
    settle_drawings(cx, window);
}

/// An Excalidraw drawing in a note.
const DRAWING_NOTE: &str = "How a note travels.\n\n```excalidraw\n{\"type\":\"excalidraw\",\"version\":2,\"elements\":[{\"type\":\"rectangle\",\"x\":0,\"y\":0,\"width\":160,\"height\":80,\"strokeColor\":\"#1e1e1e\",\"backgroundColor\":\"#a5d8ff\",\"fillStyle\":\"hachure\",\"strokeWidth\":2,\"roughness\":1,\"seed\":7,\"roundness\":{\"type\":3}},{\"type\":\"text\",\"x\":20,\"y\":27,\"width\":120,\"height\":25,\"text\":\"Kanso\",\"fontSize\":20,\"fontFamily\":1,\"textAlign\":\"center\",\"strokeColor\":\"#1e1e1e\",\"seed\":8},{\"type\":\"arrow\",\"x\":170,\"y\":40,\"width\":110,\"height\":0,\"points\":[[0,0],[110,0]],\"strokeColor\":\"#1e1e1e\",\"endArrowhead\":\"arrow\",\"strokeWidth\":2,\"roughness\":1,\"seed\":9},{\"type\":\"ellipse\",\"x\":290,\"y\":0,\"width\":140,\"height\":80,\"strokeColor\":\"#e03131\",\"backgroundColor\":\"#ffc9c9\",\"fillStyle\":\"solid\",\"strokeWidth\":2,\"roughness\":1,\"seed\":10},{\"type\":\"text\",\"x\":310,\"y\":27,\"width\":100,\"height\":25,\"text\":\"Worker\",\"fontSize\":20,\"fontFamily\":1,\"textAlign\":\"center\",\"strokeColor\":\"#1e1e1e\",\"seed\":11},{\"type\":\"freedraw\",\"x\":40,\"y\":110,\"width\":300,\"height\":30,\"points\":[[0,10],[40,0],[80,20],[120,5],[160,25],[200,8],[240,22],[300,12]],\"strokeColor\":\"#2f9e44\",\"strokeWidth\":2,\"seed\":12}]}\n```\n\nSealed on the way.\n";

/// Lets math and diagrams finish drawing: they draw on threads of their
/// own, which the test clock doesn't wait for.
fn settle_drawings(cx: &mut HeadlessAppContext, window: AnyWindowHandle) {
    for _ in 0..60 {
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn rich_note(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Orbits", RICH_NOTE);
    settle_drawings(cx, window);
}

fn drawing_note(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    crate::preview::REVEAL_IMAGE_SIZES.store(true, std::sync::atomic::Ordering::Relaxed);
    open_new_note(cx, workspace, store, "Sketch", DRAWING_NOTE);
    settle_drawings(cx, window);
}

/// Pictures sized by a bar (`![Lake|320](…)`), with the size buttons and
/// the drag handle on the right edge showing as on hover.
fn bar_sized_images(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let lake =
        crate::smart_paste::attach_image(store, &sample_picture(960, 540), "png", Some("Lake.png"))
            .unwrap();
    let body = format!(
        "# Field notes\n\nThe lake at seven, written the way Obsidian sizes images.\n\n![Lake|320](inkdrop://{lake})\n\nThe handle on its right edge drags it wider or narrower.\n"
    );
    crate::preview::REVEAL_IMAGE_SIZES.store(true, std::sync::atomic::Ordering::Relaxed);
    open_new_note(cx, workspace, store, "Field notes", &body);
}

/// A picture mid-drag: shown at the width it will be, which the chip over
/// its corner says.
fn dragging_image(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    *crate::preview::PREVIEW_DRAG_WIDTH.lock().unwrap() = Some(416.);
    bar_sized_images(cx, window, workspace, store);
}

/// "Resize image…" from the palette, with the cursor on the image's line
/// and a width typed.
fn palette_resize_image(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    bar_sized_images(cx, window, workspace, store);
    crate::preview::REVEAL_IMAGE_SIZES.store(false, std::sync::atomic::Ordering::Relaxed);
    cx.update_window(window, |_, window, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.reveal_line(4, window, cx));
    })
    .unwrap();
    cx.run_until_parked();
    pick_from_header(cx, window, workspace, Picker::ImageSize, "360");
}

/// "New drawing": an empty drawing, with its actions showing.
fn new_drawing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    crate::preview::REVEAL_IMAGE_SIZES.store(true, std::sync::atomic::Ordering::Relaxed);
    let body = format!(
        "How a note travels.\n\n{}\n\nSketch it in Excalidraw, then paste it back.\n",
        crate::image_size::EMPTY_DRAWING
    );
    open_new_note(cx, workspace, store, "Sketch", &body);
    settle_drawings(cx, window);
}

/// A table as typed, its pipes all over the place.
const ROUGH_TABLE: &str = "# Pantry\n\nWhat to buy this week.\n\n| Item | Qty | Price |\n|:--|:-:|--:|\n| Apples | 6 | 3.20 |\n| Oat milk | 2 | 4.50 |\n| Coffee beans | 1 | 12.00 |\n| Café crème | 12 | 0.90 |\n\nThe rest can wait.\n";

fn table_before(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Pantry", ROUGH_TABLE);
}

/// The same table after Format Table: pipes in line, alignment kept.
fn table_after(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Pantry", ROUGH_TABLE);
    cx.run_until_parked();
    let at = ROUGH_TABLE.find("Apples").unwrap();
    cx.update_window(window, |_, window, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.focus_body(window, cx);
            editor
                .body_state()
                .update(cx, |body, cx| body.set_selected_range(at..at, cx));
            editor.format_table(window, cx);
        });
    })
    .unwrap();
}

/// The formatted table in the monospace face, where its pipes line up on
/// screen too.
fn table_after_mono(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    mono_font(cx, window, workspace, store);
    table_after(cx, window, workspace, store);
}

/// Focus mode with the pointer near the top: the header shows.
fn focus_mode_hover(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    focus_mode(cx, window, workspace, store);
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.reveal_chrome_for_test(cx));
    });
}

/// The divider left at 38% of the editor, as a drag in an earlier session
/// keeps it.
fn split_dragged(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.keep_split_ratio(0.38, cx));
    });
}

/// The source scrolled about halfway: the preview beside it follows.
fn split_scrolled(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    // The editor knows how tall its lines are once it has been drawn.
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    cx.update_window(window, |_, _, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.scroll_source_for_test(0.5, cx));
    })
    .unwrap();
}

/// The first launch: no notes, notebooks or tags, and signed out.
fn first_run(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    for filter in [Filter::All, Filter::Trash] {
        let params = crate::library::search_params(&filter, "", Default::default(), true);
        for note in store.search_summaries(&params).unwrap() {
            store.delete(&note.id, Origin::Ui).unwrap();
        }
    }
    for tag in store.tags().unwrap() {
        store.delete(&tag.id, Origin::Ui).unwrap();
    }
    // A notebook inside one deleted before it is gone already.
    for book in store.books().unwrap() {
        store.delete(&book.id, Origin::Ui).ok();
    }
    sign_out_demo(cx);
    cx.run_until_parked();
}

/// All notes with none open: what the editor says, with the real shortcut.
fn no_note_open(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.select(None, cx)));
}

/// A note in the trash, with its banner.
fn open_trashed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    set_filter(cx, workspace, Filter::Trash);
    let params = crate::library::search_params(&Filter::Trash, "", Default::default(), true);
    if let Some(note) = store.search_summaries(&params).unwrap().into_iter().next() {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| library.update(cx, |library, cx| library.select(Some(note.id), cx)));
    }
}

/// Unsaved typing in the open note while another app saves it: the editor
/// asks which version to keep.
fn conflict(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    let id = cx.update(|cx| editor.read(cx).note_id().map(str::to_owned));
    let Some(id) = id else {
        return;
    };
    cx.update_window(window, |_, window, cx| {
        editor.update(cx, |editor, cx| {
            let body = format!("{}\nA line typed here.", editor.body_for_test(cx));
            editor.edit_for_test(&body, window, cx);
        });
    })
    .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "Changed by an agent over the local API."}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
}

/// An empty notebook, "Research" when there is one: no notes of its own
/// and none in notebooks inside it. Else the first, preferring a name no
/// other notebook has, so the shot can't be read as its namesake.
fn empty_notebook(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books = store.books().unwrap();
    let is_empty = |b: &&kanso_core::Book| b.count == 0 && !has_notes_inside(&b.id, &books);
    let unique = |b: &&kanso_core::Book| books.iter().filter(|o| o.name == b.name).count() == 1;
    let empty = books
        .iter()
        .filter(is_empty)
        .find(|b| b.name == "Research")
        .or_else(|| books.iter().filter(is_empty).find(unique))
        .or_else(|| books.iter().find(is_empty));
    if let Some(book) = empty {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| {
            library.update(cx, |library, cx| {
                library.set_filter(crate::library::Filter::Book(book.id.clone()), cx);
                library.select(None, cx);
            })
        });
    }
}

/// A notebook that holds only notebooks, its notes a level down. The demo
/// has none, so it gets one: "Clients", with "Acme" and a note inside.
fn notebook_of_notebooks(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books = store.books().unwrap();
    let found = books
        .iter()
        .find(|b| b.count == 0 && has_notes_inside(&b.id, &books))
        .map(|b| b.id.clone());
    let id = found.unwrap_or_else(|| {
        let book = |name: &str, parent: Option<&str>| {
            store
                .put(
                    DocType::Book,
                    json!({"name": name, "parentBookId": parent}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let clients = book("Clients", None);
        let acme = book("Acme", Some(&clients));
        let now = cx.update(|cx| Clock::now_ms(cx));
        store
            .put(
                DocType::Note,
                json!({"title": "Kickoff notes", "body": "Scope, people and dates.",
                    "bookId": acme, "createdAt": now - DAY, "updatedAt": now - DAY}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        clients
    });
    cx.run_until_parked();
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| {
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(id), cx);
            library.select(None, cx);
        })
    });
}

/// The note whose first table of five or more columns comes earliest, so
/// the table sits as high in the preview as the library allows.
fn open_wide_table(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    // The line of the first table head separator with five or more columns.
    let wide_table_line = |body: &str| {
        body.lines().position(|line| {
            let line = line.trim();
            line.starts_with('|')
                && line.matches('|').count() >= 6
                && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
        })
    };
    let notes = store.search_summaries(&Default::default()).unwrap();
    let found = notes
        .into_iter()
        .filter_map(|n| {
            let note = store.note(&n.id).ok().flatten()?;
            Some((wide_table_line(&note.body)?, n))
        })
        .min_by_key(|(line, _)| *line)
        .map(|(_, note)| note);
    if let Some(note) = found {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| library.update(cx, |library, cx| library.select(Some(note.id), cx)));
    }
}

/// Types "sync" into the search box, so the field and the list agree.
fn search_sync(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let search = workspace.read(cx).note_list.read(cx).search.clone();
        search.update(cx, |search, cx| search.replace_all("sync", window, cx));
    })
    .unwrap();
}

/// Searches for something no note has: the zero-results state.
fn search_nothing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let search = workspace.read(cx).note_list.read(cx).search.clone();
        search.update(cx, |search, cx| {
            search.replace_all("quarterly okr", window, cx)
        });
    })
    .unwrap();
}

/// The empty search box with focus, which shows the search syntax.
fn focus_search(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let list = workspace.read(cx).note_list.clone();
        list.update(cx, |list, cx| list.focus_search(window, cx));
    })
    .unwrap();
}

/// The pointer over the third note, revealing its actions.
fn hover_third(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.run_until_parked();
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        let third = library.read(cx).notes.get(2).map(|n| n.id.clone());
        let list = workspace.read(cx).note_list.clone();
        list.update(cx, |list, cx| {
            list.hovered = third;
            cx.notify();
        });
    });
}

/// Sorted by title: no date headers.
fn sort_by_title(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        library.update(cx, |library, cx| {
            library.set_sort(kanso_store::Sort::Title, false, cx)
        });
    });
}

/// Narrowed to completed notes from the list's filter menu.
fn only_completed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        library.update(cx, |library, cx| {
            library.set_status_only(Some(kanso_core::NoteStatus::Completed), cx)
        });
    });
}

/// Notes whose titles and excerpts run long, edited just now.
fn long_titles(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    let book = store.books().unwrap()[0].id.clone();
    let now = cx.update(|cx| Clock::now_ms(cx));
    let notes = [
        (
            "A very long note title that keeps going well past the width of the note list column",
            "An excerpt long enough to need two lines in the list and then some more words so that it has to stop with an ellipsis at the end of the second line.",
            "active",
            true,
        ),
        (
            "Überlange Überschrift mit Umlauten, die ebenfalls abgeschnitten werden muss",
            "Kurz.",
            "onHold",
            false,
        ),
        (
            "Supercalifragilisticexpialidocious_without_any_spaces_at_all_in_the_title",
            "Averyveryverylongwordwithoutspacesthatmustnotpushtherowwiderthanthecolumn",
            "none",
            false,
        ),
    ];
    for (ix, (title, body, status, pinned)) in notes.into_iter().enumerate() {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book, "status": status,
                    "pinned": pinned, "createdAt": now - DAY, "updatedAt": now - ix as i64 * MINUTE}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
}

fn mono_font(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.editor_font = EditorFont::Mono;
            cx.notify();
        });
    });
}

/// Settings at `page`, with demo values in place of the user's: an account
/// signed in and API credentials, none of them real.
fn show_settings(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    page: SettingsPage,
) {
    cx.update(|cx| {
        let config = &mut cx.global_mut::<AppState>().config;
        config.sync.server = None;
        config.sync.enabled = true;
        config.server.username = "kanso".into();
        config.server.password = "demo-password-not-real-7Q2x".into();
        let now = Clock::now_ms(cx);
        let services = cx.global_mut::<Services>();
        services.account = AccountState::SignedIn {
            username: DEMO_USER.into(),
        };
        services.sync = SyncState::Synced {
            at_ms: now,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: None,
        };
    });
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.settings.update(cx, |settings, _| {
                settings.data_dir_label = Some("~/Library/Application Support/Kanso".into());
            });
            workspace.open_settings(Some(page), window, cx);
        });
    })
    .unwrap();
}

fn set_sync(cx: &mut HeadlessAppContext, sync: SyncState) {
    cx.update(|cx| cx.global_mut::<Services>().sync = sync);
}

fn settings_general(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::General);
}

/// Zooms the whole app to `percent`, as the View menu does, after the
/// window has opened at actual size.
fn zoom_to(cx: &mut HeadlessAppContext, percent: u32) {
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.appearance.zoom = percent);
        crate::zoom::set(percent, cx);
    });
}

/// The most recent note at the largest zoom.
fn zoom_largest(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    zoom_to(cx, crate::zoom::ZOOM_RANGE.1);
    open_latest(cx, window, workspace, store);
}

/// The most recent note at the smallest zoom.
fn zoom_smallest(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    zoom_to(cx, crate::zoom::ZOOM_RANGE.0);
    open_latest(cx, window, workspace, store);
}

/// Settings › General at 120%, its Zoom stepper showing it.
fn zoom_settings_general(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    zoom_to(cx, 120);
    settings_general(cx, window, workspace, store);
}

fn settings_editor(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Editor);
}

fn settings_appearance(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Appearance);
}

/// Settings › Themes › Browse, with a signed catalog in place and no
/// network: the cards draw their headline-color placeholders, one installed
/// and one with an update.
fn settings_gallery(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        use kanso_gallery::index::{Appearance, FileRef, Index, SCHEMA, Theme};
        let theme = |id: &str, name: &str, appearance: Appearance, version: u64| Theme {
            id: id.into(),
            kind: kanso_gallery::Kind::Color,
            family: Some("nord".into()),
            version,
            name: name.into(),
            appearance: Some(appearance),
            author: "Arctic Ice Studio".into(),
            license: "MIT".into(),
            source: "https://www.nordtheme.com".into(),
            description: "An arctic, north-bluish palette.".into(),
            min_kanso: "0.8.4".into(),
            file: Some(FileRef {
                path: format!("{id}.json"),
                sha256: "ab".repeat(32),
                size: 2048,
            }),
            bundle: None,
            preview: Vec::new(),
            colors: Some(kanso_gallery::HeadlineColors {
                editor: "#2E3440".into(),
                text: "#D8DEE9".into(),
                accent: "#88C0D0".into(),
                sidebar: "#21252E".into(),
                list: "#282E38".into(),
            }),
        };
        let index = Index {
            schema: SCHEMA,
            serial: 42,
            generated_at: "2026-10-23T12:00:00Z".into(),
            source_commit: "8d96298839a6176210cd1e3b376b82ffefc70ab3".into(),
            signing_key_id: "7f3a91c2".into(),
            themes: vec![
                theme("nord-light", "Nord Light", Appearance::Light, 2),
                theme("nord", "Nord", Appearance::Dark, 3),
            ],
            revoked: Vec::new(),
        };
        let mut gallery = crate::settings_gallery::ThemeGallery {
            index: Some(index),
            status: crate::settings_gallery::Status::Ready,
            color_tab: crate::settings_gallery::Tab::Browse,
            ..Default::default()
        };
        gallery.installed.push(kanso_gallery::Installed {
            id: "nord-light".into(),
            kind: kanso_gallery::Kind::Color,
            version: 1,
            name: "Nord Light".into(),
            family: Some("nord".into()),
            digest: "aa".into(),
            current_digest: Some("aa".into()),
            installed_at: "2026-10-23T12:00:00Z".into(),
        });
        cx.set_global(gallery);
    });
    show_settings(cx, window, workspace, SettingsPage::Appearance);
}

/// A note with every Markdown construct a theme colors and code in a few
/// languages, for showing syntax and preview colors side by side.
const THEME_NOTE: &str = "# Theme sampler\n\nSome **bold**, some *italic*, `inline code` and a [link](https://kanso.app).\n\n## Code\n\n```json\n{\n  \"name\": \"Nord\",\n  \"dark\": true,\n  \"size\": 14,\n  \"accent\": null,\n  \"tags\": [\"calm\", \"cool\"],\n  \"escape\": \"tab\\\\there\"\n}\n```\n\n> Colors should help reading, never fight it.\n\n- [x] Pick a palette\n- [ ] Check contrast\n  1. Text on panes\n  2. Code on blocks\n\n---\n\n| Token | Color |\n|---|---|\n| keyword | accent |\n| string | green |\n\nPress <kbd>Cmd</kbd>+<kbd>K</kbd> for commands.\n";

/// Code in several languages, with a quote and tasks between, to see each
/// grammar's colors and the preview's blocks.
const CODE_NOTE: &str = "# Code samples\n\n```rust\n/// A note in the store.\n#[derive(Debug, Clone)]\npub struct Note {\n    pub id: String,\n    pub pinned: bool,\n}\n\nfn main() {\n    let count = notes.iter().filter(|n| n.pinned).count();\n    println!(\"{count} pinned\");\n}\n```\n\n```ts\ninterface Note { id: string; tags: string[] }\nexport async function load(id: string): Promise<Note> {\n  const res = await fetch(`/notes/${id}`);\n  return (await res.json()) as Note; // typed\n}\n```\n\n```python\nimport re\n\ndef slug(title: str) -> str:\n    \"\"\"Lowercase words joined by dashes.\"\"\"\n    return re.sub(r\"[^a-z0-9]+\", \"-\", title.lower()).strip(\"-\")\n```\n\n> Quotes keep their own color; finished tasks step back.\n\n- [x] Highlight Rust, TypeScript and Python\n- [ ] Check every theme\n\n```go\nfunc Sum(xs []int) (total int) {\n\tfor _, x := range xs {\n\t\ttotal += x\n\t}\n\treturn\n}\n```\n\n```sh\n# Back up the library\ncp \"$HOME/Library/Kanso/kanso.db\" ~/backup/ && echo done\n```\n\n```sql\nSELECT title, updated_at FROM notes WHERE pinned = 1 ORDER BY updated_at DESC LIMIT 20;\n```\n\n```yaml\ntheme: Kanso Dark\nsync:\n  interval: 5m\n  enabled: true\n```\n\n```swift\nstruct Note: Identifiable {\n    let id: UUID\n    var title = \"Untitled\"\n}\n```\n";

/// The code samples note, open in the editor.
fn open_code_note(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Code samples", CODE_NOTE);
}

/// Adds a note beside the latest one and opens it.
fn open_new_note(
    cx: &mut HeadlessAppContext,
    workspace: &Entity<Workspace>,
    store: &Store,
    title: &str,
    body: &str,
) {
    let book = store
        .search_summaries(&Default::default())
        .ok()
        .and_then(|notes| notes.into_iter().next())
        .and_then(|n| store.note(&n.id).ok().flatten())
        .map(|note| note.book_id);
    let id = store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book,
                "createdAt": DEMO_NOW_MS - MINUTE, "updatedAt": DEMO_NOW_MS - MINUTE}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| {
        library.update(cx, |library, cx| {
            library.reload(cx);
            library.select(Some(id), cx);
        })
    });
}

/// The theme sampler note, open in the editor.
fn open_theme_note(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Theme sampler", THEME_NOTE);
}

/// The demo account's name.
const DEMO_USER: &str = "ngoni";

fn sign_out_demo(cx: &mut HeadlessAppContext) {
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.account = AccountState::SignedOut;
        services.sync = SyncState::Off;
    });
}

/// What the server says about the demo account: its storage, and three
/// Macs, this one first.
fn demo_details(cx: &mut HeadlessAppContext) {
    let now = cx.update(|cx| Clock::now_ms(cx));
    let device = |id: &str, name: Option<&str>, ago: i64, current: bool| DeviceInfo {
        id: id.into(),
        name: name.map(Into::into),
        created_at: now - 90 * DAY,
        last_seen_at: now - ago,
        current,
    };
    cx.update(|cx| {
        cx.global_mut::<Services>().details = AccountDetails {
            usage: Some(Usage {
                docs: 850,
                doc_bytes: 3_250_586,
                files: 15,
                file_bytes: 14 * 1024 * 1024,
                writes_today: 120,
                quota: Quota {
                    docs: 100_000,
                    doc_bytes: 100 * 1024 * 1024,
                    files: 10_000,
                    file_bytes: 500 * 1024 * 1024,
                    writes_per_day: 10_000,
                },
            }),
            devices: Some(vec![
                device("d_1", Some("Ngoni's MacBook Pro"), 0, true),
                device("d_2", Some("Studio"), 2 * 60 * MINUTE, false),
                device("d_3", None, 3 * DAY, false),
            ]),
            loading: false,
            stale: false,
            failed: None,
        };
    });
}

/// The Account page, signed in, synced a moment ago.
fn settings_account(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Account);
    demo_details(cx);
}

/// The provider-authenticated account while this device is still locked.
fn settings_account_locked(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Account);
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.account = AccountState::SignedInLocked {
            username: DEMO_USER.into(),
        };
        services.locked_has_password = Some(true);
    });
}

fn settings_account_locked_waiting(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    cx.update(|cx| {
        cx.global_mut::<Services>().approval.waiting =
            Some(crate::device_approval::WaitingState::new(1, DEMO_NOW_MS));
    });
}

fn settings_account_locked_code(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    cx.update(|cx| {
        let mut waiting = crate::device_approval::WaitingState::new(1, DEMO_NOW_MS);
        waiting.code = Some("123 456".into());
        waiting.phase = crate::device_approval::WaitPhase::Compare {
            code: "123 456".into(),
        };
        cx.global_mut::<Services>().approval.waiting = Some(waiting);
    });
}

fn settings_account_locked_no_devices(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    cx.update(|cx| {
        let mut waiting = crate::device_approval::WaitingState::new(0, DEMO_NOW_MS);
        waiting.phase = crate::device_approval::WaitPhase::NoDevices;
        waiting.recovery_offered = true;
        cx.global_mut::<Services>().approval.waiting = Some(waiting);
    });
}

fn settings_account_locked_cancelled(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    cx.update(|cx| {
        let mut waiting = crate::device_approval::WaitingState::new(1, DEMO_NOW_MS);
        waiting.phase = crate::device_approval::WaitPhase::Cancelled;
        waiting.recovery_offered = true;
        cx.global_mut::<Services>().approval.waiting = Some(waiting);
    });
}

fn settings_account_locked_expired(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    cx.update(|cx| {
        let mut waiting = crate::device_approval::WaitingState::new(1, DEMO_NOW_MS);
        waiting.phase = crate::device_approval::WaitPhase::Expired;
        waiting.recovery_offered = true;
        cx.global_mut::<Services>().approval.waiting = Some(waiting);
    });
}

fn settings_account_locked_password(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    window
        .update(cx, |_, window, cx| {
            crate::account_page::unlock_password_dialog(window, cx);
        })
        .unwrap();
}

fn settings_account_locked_recovery(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_locked(cx, window, workspace, store);
    window
        .update(cx, |_, window, cx| {
            crate::account_page::recover_dialog(window, cx);
        })
        .unwrap();
}

fn settings_account_syncing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_sync(cx, SyncState::Syncing);
}

/// The signed-in methods list, with a linked provider, an unlinked provider,
/// and a passkey that can unlock.
fn settings_account_methods(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    let now = cx.update(|cx| Clock::now_ms(cx));
    cx.update(|cx| {
        cx.global_mut::<Services>().account_methods = AccountMethodsState {
            last_successful: Some(kanso_sync::AccountMethods {
                has_password: false,
                rp_id: Some("sync.kansoapp.app".into()),
                available: vec![kanso_sync::Provider::Apple, kanso_sync::Provider::Google],
                linked: vec![kanso_sync::provider::LinkedProvider {
                    provider: kanso_sync::Provider::Apple,
                    created_at: now - 14 * DAY,
                }],
                passkeys: vec![kanso_sync::provider::PasskeyInfo {
                    credential_id: vec![1, 2, 3],
                    created_at: now - 30 * DAY,
                    last_used_at: Some(now - 2 * 60 * MINUTE),
                    prf: true,
                    transports: vec!["internal".into()],
                    aaguid: Some(kanso_sync::provider::ICLOUD_KEYCHAIN_AAGUIDS[0].into()),
                }],
            }),
            ..AccountMethodsState::default()
        };
    });
}

/// Linking GitHub from Sign-in methods: the device code waits under the list.
fn settings_account_methods_linking(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account_methods(cx, window, workspace, store);
    cx.update(|cx| {
        cx.global_mut::<Services>().provider_wait = Some(ProviderWait {
            provider: "GitHub",
            user_code: Some("ABCD-1234".into()),
            verification_uri: Some("https://github.com/login/device".into()),
            cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        });
    });
}

/// The account's ways in for the proof sheet's shots, as of a minute ago.
fn set_methods(
    cx: &mut HeadlessAppContext,
    has_password: bool,
    linked: Vec<kanso_sync::Provider>,
    passkeys: usize,
) {
    let now = cx.update(|cx| Clock::now_ms(cx));
    cx.update(|cx| {
        cx.global_mut::<Services>().account_methods = AccountMethodsState {
            last_successful: Some(kanso_sync::AccountMethods {
                has_password,
                rp_id: Some("sync.kansoapp.app".into()),
                available: vec![
                    kanso_sync::Provider::Apple,
                    kanso_sync::Provider::Google,
                    kanso_sync::Provider::Github,
                ],
                linked: linked
                    .into_iter()
                    .map(|provider| kanso_sync::provider::LinkedProvider {
                        provider,
                        created_at: now - 14 * DAY,
                    })
                    .collect(),
                passkeys: (0..passkeys)
                    .map(|n| kanso_sync::provider::PasskeyInfo {
                        credential_id: vec![n as u8 + 1],
                        created_at: now - 30 * DAY,
                        last_used_at: None,
                        prf: true,
                        transports: vec!["internal".into()],
                        aaguid: None,
                    })
                    .collect(),
            }),
            ..AccountMethodsState::default()
        };
    });
}

/// "Confirm it's you": the password or Apple, for a new recovery key.
fn proof_choice(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_methods(cx, true, vec![kanso_sync::Provider::Apple], 0);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::proof_sheet::open(crate::proof_sheet::Action::NewRecoveryKey, window, cx)
    })
    .unwrap();
}

/// Unlinking Apple when it is the only way in: Apple proves it, and the
/// recovery key comes with it.
fn proof_unlink_last(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_methods(cx, false, vec![kanso_sync::Provider::Apple], 0);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::proof_sheet::open(
            crate::proof_sheet::Action::Unlink(kanso_sync::Provider::Apple),
            window,
            cx,
        )
    })
    .unwrap();
}

/// [Set a password…] on an Apple-only account: Apple, the new password and
/// the recovery key.
fn proof_set_password(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_methods(cx, false, vec![kanso_sync::Provider::Apple], 0);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::proof_sheet::open(crate::proof_sheet::Action::SetPassword, window, cx)
    })
    .unwrap();
}

/// Removing a passkey with the password as proof.
fn proof_remove_passkey(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_methods(cx, true, vec![], 2);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::proof_sheet::open(
            crate::proof_sheet::Action::RemovePasskey(vec![1]),
            window,
            cx,
        )
    })
    .unwrap();
}

fn settings_account_offline(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    set_sync(cx, SyncState::Failed(SyncProblem::Offline));
}

/// Too many changes today: sync waits, and says until when.
fn settings_account_paused(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    let now = cx.update(|cx| Clock::now_ms(cx));
    set_sync(
        cx,
        SyncState::Failed(SyncProblem::Paused {
            until: now + 3 * 60 * MINUTE,
        }),
    );
}

/// A sync that finished with a full account.
fn settings_account_full(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    let now = cx.update(|cx| Clock::now_ms(cx));
    set_sync(
        cx,
        SyncState::Synced {
            at_ms: now - MINUTE,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: Some(SyncProblem::QuotaExceeded(QuotaKind::FileBytes)),
        },
    );
}

/// A note changed on two Macs at once: the page names it and opens it.
fn settings_account_conflict(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    let now = cx.update(|cx| Clock::now_ms(cx));
    let first = store
        .search_summaries(&Default::default())
        .unwrap()
        .into_iter()
        .next()
        .map(|note| note.id);
    set_sync(
        cx,
        SyncState::Synced {
            at_ms: now - 3 * MINUTE,
            conflicts: 1,
            conflicted: first.into_iter().collect(),
            problem: None,
        },
    );
}

/// The storage and devices aren't fetched yet.
fn settings_account_loading(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Account);
    cx.update(|cx| {
        cx.global_mut::<Services>().details = AccountDetails {
            loading: true,
            ..AccountDetails::default()
        }
    });
}

/// Signed out: the two ways in.
fn settings_signed_out(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_signed_out(cx, window, workspace);
}

fn show_signed_out(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
) {
    show_settings(cx, window, workspace, SettingsPage::Account);
    sign_out_demo(cx);
}

/// The Account page's own view.
fn account_page(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>) -> Entity<AccountPage> {
    cx.update(|cx| workspace.read(cx).settings.read(cx).account.clone())
}

/// Signed out with `form` open and filled in.
fn open_account_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    form: SignedOutForm,
    values: [&str; 4],
) -> Entity<AccountPage> {
    show_signed_out(cx, window, workspace);
    let page = account_page(cx, workspace);
    cx.update_window(window, |_, window, cx| {
        page.update(cx, |page, cx| {
            page.open_form(form, window, cx);
            page.set_notes(9);
            page.fill(values, window, cx);
        });
    })
    .unwrap();
    page
}

fn settings_create_account(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::CreateAccount,
        [DEMO_USER, "correct horse battery staple", "", ""],
    );
}

/// GitHub device flow: the displayed code and its in-flow cancel control.
fn settings_github_wait(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_create_account(cx, window, workspace, store);
    cx.update(|cx| {
        cx.global_mut::<Services>().provider_wait = Some(ProviderWait {
            provider: "GitHub",
            user_code: Some("ABCD-1234".into()),
            verification_uri: Some("https://github.com/login/device".into()),
            cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        });
    });
}

/// The server takes no new accounts: the line says so and the button waits.
fn settings_create_account_closed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_create_account(cx, window, workspace, store);
    cx.update(|cx| {
        cx.global_mut::<Services>().signup =
            Some((kanso_sync::DEFAULT_SERVER.to_owned(), SignupState::Closed));
    });
}

/// A server of the owner's own, picked with "Use a different server…".
fn settings_create_account_custom(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_create_account(cx, window, workspace, store);
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.sync.server = Some("https://sync.example.com".into());
    });
}

/// What the default server advertises in these shots: a password,
/// passkeys for the entitled relying party with security keys, and the
/// three providers.
fn advertise_every_method(cx: &mut HeadlessAppContext) {
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.auth = Some((
            kanso_sync::DEFAULT_SERVER.to_owned(),
            kanso_sync::ServerAuth {
                password: true,
                passkeys: Some(kanso_sync::PasskeyMethods {
                    rp_id: crate::passkey_ceremony::OFFICIAL_RP_ID.into(),
                    security_key: true,
                }),
                providers: vec![
                    kanso_sync::Provider::Apple,
                    kanso_sync::Provider::Google,
                    kanso_sync::Provider::Github,
                ],
                ..Default::default()
            },
        ));
        services.signup = Some((kanso_sync::DEFAULT_SERVER.to_owned(), SignupState::Open));
        services.platform_passkeys = Some(true);
    });
}

/// Sign in with every method the server advertises: the passkey first,
/// then the providers, then the username form.
fn settings_sign_in_methods(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::SignIn,
        ["", "", "", ""],
    );
    advertise_every_method(cx);
}

/// Create account with every method: Create with a passkey above the
/// providers and the username form.
fn settings_create_methods(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::CreateAccount,
        ["", "", "", ""],
    );
    advertise_every_method(cx);
}

/// Create account after [Create with a username]: the password form alone.
fn settings_create_username(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let page = open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::CreateAccount,
        ["", "", "", ""],
    );
    advertise_every_method(cx);
    cx.update_window(window, |_, window, cx| {
        page.update(cx, |page, cx| page.choose_password_form(window, cx));
    })
    .unwrap();
}

/// Opens `form` over Create account or Sign in, with `username` in place of
/// the random suggestion.
fn open_passkey_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    form: crate::account_page::PasskeyForm,
    username: &str,
) -> Entity<AccountPage> {
    let signed_out = if form.creating {
        SignedOutForm::CreateAccount
    } else {
        SignedOutForm::SignIn
    };
    let page = open_account_form(cx, window, workspace, signed_out, ["", "", "", ""]);
    advertise_every_method(cx);
    cx.update_window(window, |_, window, cx| {
        page.update(cx, |page, cx| {
            page.open_passkey_form(form, window, cx);
            page.fill([username, "", "", ""], window, cx);
        });
    })
    .unwrap();
    page
}

/// Create with a passkey: the suggested username, which can be changed.
fn settings_passkey_username(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_passkey_form(
        cx,
        window,
        workspace,
        crate::account_page::PasskeyForm {
            creating: true,
            security_key: false,
        },
        "quiet-otter-4821",
    );
}

/// Linux: Continue with a security key, username first, after the key
/// asked for its PIN.
fn settings_security_key_pin(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        cx.global_mut::<Services>().security_keys = vec![kanso_fido::SecurityKeyDevice {
            path: "/dev/hidraw3".into(),
            vendor: "1050".into(),
            product: "YubiKey 5 NFC".into(),
            hmac_secret: true,
        }];
    });
    let page = open_passkey_form(
        cx,
        window,
        workspace,
        crate::account_page::PasskeyForm {
            creating: false,
            security_key: true,
        },
        DEMO_USER,
    );
    cx.update(|cx| {
        page.update(cx, |page, cx| {
            page.show_error(crate::account_words::SECURITY_KEY_PIN, cx)
        })
    });
}

/// Creating the account: the keys are being derived.
fn settings_create_account_busy(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::CreateAccount,
        [DEMO_USER, "", "", ""],
    );
    cx.update(|cx| {
        cx.global_mut::<Services>().task = Some(RunningTask {
            kind: TaskKind::SignUp,
            step: Step::DerivingKeys,
        });
    });
}

/// Sign in, after a wrong password.
fn settings_sign_in_error(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let page = open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::SignIn,
        [DEMO_USER, "", "", ""],
    );
    cx.update(|cx| {
        page.update(cx, |page, cx| {
            page.show_error("That username and password don't match.", cx)
        })
    });
}

/// "Forgot your password?", signed out.
fn settings_recover(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::Recover,
        [DEMO_USER, "", "", "K7Q2-9F3C-XXXX"],
    );
}

/// Another Mac changed the password: the reason over the Sign in form.
fn settings_session_ended(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Account);
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.account = AccountState::SessionEnded {
            username: DEMO_USER.into(),
            reason: SignOutReason::PasswordChanged,
        };
        services.sync = SyncState::Off;
    });
}

/// A made-up recovery key: the bytes are no one's.
fn demo_key() -> RecoveryKey {
    RecoveryKey::from_bytes([
        0x3a, 0x91, 0x5c, 0x07, 0xe2, 0x48, 0xbd, 0x16, 0x6f, 0xd0, 0x29, 0x84, 0x73, 0xc5, 0x1e,
        0x0b,
    ])
}

/// Right after creating the account: the key to save.
fn settings_recovery_key(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    show_demo_key(cx, None);
}

fn show_demo_key(cx: &mut HeadlessAppContext, note: Option<&'static str>) {
    cx.update(|cx| {
        cx.global_mut::<Services>().pending_key = Some(PendingKey {
            key: demo_key(),
            username: DEMO_USER.into(),
            server: kanso_sync::DEFAULT_SERVER.into(),
            note,
        });
    });
}

/// A sign-up whose account was created but whose Mac couldn't finish
/// signing in: the key shows with what happened.
fn settings_recovery_key_failed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_create_account(cx, window, workspace, store);
    show_demo_key(cx, Some(crate::account_words::CREATED_NOT_SIGNED_IN));
}

/// Right after a password change: its new recovery key.
fn settings_recovery_key_password(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    show_demo_key(cx, Some(crate::account_words::KEY_AFTER_PASSWORD));
}

/// "New recovery key" whose answer was lost, before the server is asked:
/// keep the old key too.
fn settings_recovery_key_unconfirmed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    show_demo_key(cx, Some(crate::account_words::KEY_UNCONFIRMED));
}

/// The same key once the server said it holds it.
fn settings_recovery_key_taken(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_account(cx, window, workspace, store);
    show_demo_key(cx, Some(crate::account_words::KEY_TAKEN));
}

/// The key screen with "I've saved my recovery key" ticked.
fn settings_recovery_key_saved(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_recovery_key(cx, window, workspace, store);
    let page = account_page(cx, workspace);
    cx.update(|cx| page.update(cx, |page, cx| page.set_key_saved(true, cx)));
}

/// Opens a dialog over the Account page, at rest.
fn account_dialog(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
    open: impl FnOnce(&mut Window, &mut gpui_kit::App),
) {
    settings_account(cx, window, workspace, store);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| open(window, cx))
        .unwrap();
}

/// The question a sign-in asks when this Mac already has notes.
fn binding_question(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_account_form(
        cx,
        window,
        workspace,
        SignedOutForm::SignIn,
        [DEMO_USER, "", "", ""],
    );
    cx.update(|cx| cx.set_reduce_motion(true));
    let question = crate::account_words::binding_question(
        &Plan::Ask {
            from: None,
            notes: 9,
            unsynced: 3,
        },
        DEMO_USER,
    )
    .unwrap();
    cx.update_window(window, |_, window, cx| {
        let choices = question
            .answers
            .iter()
            .map(|answer| crate::dialogs::Described {
                label: answer.label.clone().into(),
                description: answer.description.clone().into(),
            })
            .collect();
        crate::dialogs::choose(
            question.title.clone(),
            question.body.clone(),
            choices,
            window,
            cx,
            |_, _, _| {},
        );
    })
    .unwrap();
}

fn change_password_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        if let Some(form) = crate::account_page::change_password_dialog(window, cx) {
            form.update(cx, |form, cx| {
                form.fill(
                    &["old password here", "a new passphrase", "a new passphrase!"],
                    window,
                    cx,
                );
                form.show_error(crate::account_words::PASSWORDS_DIFFER, cx);
            });
        }
    });
}

/// Setting the lock password for the first locked note, both fields
/// filled; the eye beside each shows what was typed.
fn lock_password_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let editor = workspace.read(cx).editor.clone();
        if let Some(form) = crate::locked::lock_open_note(editor, window, cx) {
            form.update(cx, |form, cx| {
                form.fill(&["correct horse", "correct horse"], window, cx)
            });
        }
    })
    .unwrap();
}

/// The change running: fields disabled, the step on the button.
fn change_password_busy(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        if let Some(form) = crate::account_page::change_password_dialog(window, cx) {
            form.update(cx, |form, cx| form.show_step(Step::DerivingKeys, cx));
        }
    });
}

fn new_recovery_key_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        crate::account_page::new_recovery_key_dialog(window, cx);
    });
}

fn recover_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        crate::account_page::recover_dialog(window, cx);
    });
}

fn delete_account_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        if let Some(form) = crate::account_page::delete_account_dialog(window, cx) {
            form.update(cx, |form, cx| {
                form.fill(&[DEMO_USER, "my password"], window, cx)
            });
        }
    });
}

fn sign_out_confirm(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        crate::account_page::sign_out_dialog(window, cx);
    });
}

/// Signing out with a new recovery key still unsaved.
fn sign_out_unsaved_key(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    account_dialog(cx, window, workspace, store, |window, cx| {
        cx.global_mut::<Services>().pending_key = Some(PendingKey {
            key: demo_key(),
            username: DEMO_USER.into(),
            server: kanso_sync::DEFAULT_SERVER.into(),
            note: None,
        });
        crate::account_page::sign_out_dialog(window, cx);
    });
}

fn server_form(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    settings_create_account(cx, window, workspace, store);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let form = crate::account_page::server_dialog(window, cx);
        form.update(cx, |form, cx| {
            form.fill(&["http://sync.example.com"], window, cx);
            form.show_error(
                "Use an https:// address. Plain http:// only reaches this device (127.0.0.1 or localhost).",
                cx,
            );
        });
    })
    .unwrap();
}

/// The notes with this Mac signed out: the footer asks to sign in.
fn signed_out_footer(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    sign_out_demo(cx);
}

/// The rail signed out, its sync icon quiet.
fn rail_signed_out(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    sign_out_demo(cx);
}

fn settings_api(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::LocalApi);
}

/// Another app holds the API's port.
fn settings_api_port_in_use(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::LocalApi);
    cx.update(|cx| {
        cx.global_mut::<Services>().api = ApiState::PortInUse(crate::config::DEFAULT_PORT);
    });
}

/// Settings beside the sidebar's rail, which keeps the traffic lights; the
/// gear in its bottom band is the place shown.
fn settings_rail(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    show_settings(cx, window, workspace, SettingsPage::Account);
    demo_details(cx);
}

/// ⌘, in focus mode: Settings come up beside the sidebar, out of focus mode.
fn settings_from_focus(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    focus_mode(cx, window, workspace, store);
    show_settings(cx, window, workspace, SettingsPage::Editor);
}

/// The command palette over Settings, asked for a settings page.
fn settings_palette(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_settings(cx, window, workspace, SettingsPage::Appearance);
    open_palette(cx, window, workspace, Mode::Commands, "settings");
}

/// Opens the palette in `mode` and types `query`, as from the keyboard.
fn open_palette(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    mode: Mode,
    query: &str,
) {
    cx.update_window(window, |_, window, cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| {
            palette.open(mode, window, cx);
            if !query.is_empty() {
                palette.set_query(query, window, cx);
            }
        });
    })
    .unwrap();
}

/// ⌘K with nothing typed.
fn palette(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Commands, "");
}

/// ⌘K with "sync" typed: commands and notes together, with the first note
/// selected so the detail pane shows a note.
fn palette_query(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Commands, "sync");
    // The notes arrive after a pause in typing.
    cx.advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| palette.select_first_note(cx));
    });
}

/// What a click on the editor header's notebook path or "Add tag" does:
/// the palette's list opens, and `query` is typed into it.
fn pick_from_header(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    picker: Picker,
    query: &str,
) {
    cx.update_window(window, |_, _, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |_, cx| cx.emit(EditorEvent::Pick(picker)));
    })
    .unwrap();
    // The workspace opens the palette once the event is delivered, after
    // the update above.
    if !query.is_empty() {
        cx.update_window(window, |_, window, cx| {
            let palette = workspace.read(cx).palette.clone();
            palette.update(cx, |palette, cx| palette.set_query(query, window, cx));
        })
        .unwrap();
    }
}

/// Move to notebook, from the header's notebook path.
fn palette_move(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Notebook, "");
}

/// The tag list, from the header's "Add tag": the note's tags checked.
fn palette_tag(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Tag, "");
}

/// A name no tag has, typed into the tag list: it offers to make it.
fn palette_new_tag(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Tag, "ru");
}

/// Open Note (⌘P): the most recently edited notes.
fn note_picker(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Notes, "");
}

// MARK: Linked knowledge

/// The open note's outline beside it, the heading the caret is under lit.
fn inspector_outline(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Outline), cx)
        })
    });
}

/// The notes that link to the open note, and one that only names it.
fn inspector_backlinks(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Backlinks), cx)
        })
    });
}

/// A note with wiki-links: two that resolve, one to a heading under an
/// alias, and one to a note not written yet.
const WIKI_NOTE: &str = "# Agent plan\n\nThe agent work follows the [[MCP support model]], within its [[MCP support model#Goals|goals]].\n\n## Open questions\n\n- [x] Read the [[Kanso roadmap]]\n- [ ] Draft the review flow @2026-10-24\n- [ ] Start a [[Someday list]]\n";

fn wiki_links(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_new_note(cx, workspace, store, "Agent plan", WIKI_NOTE);
}

/// `[[` typed in the source: notes offered by title.
fn wiki_autocomplete(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    use gpui_kit::EntityInputHandler as _;
    let body = "# Agent plan\n\nThe agent work follows the ";
    open_new_note(cx, workspace, store, "Agent plan", body);
    cx.run_until_parked();
    // The titles load after a pause.
    cx.advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.focus_body(window, cx);
            editor.body_state().update(cx, |state, cx| {
                let end = body.len();
                state.set_selected_range(end..end, cx);
                state.replace_text_in_range(None, "[[", window, cx);
                state.replace_text_in_range(None, "s", window, cx);
            });
        });
    })
    .unwrap();
}

/// Two notes with tasks around the demo's Thursday 28 May: two overdue,
/// two today (one at a time), some to come, one undated and one done.
/// Returns the launch plan's ID.
fn seed_tasks(store: &Store) -> String {
    let book = store.books().unwrap()[0].id.clone();
    let put = |title: &str, body: &str| {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let plan = put(
        "Launch plan",
        "# Launch plan\n\nThe public beta goes out on Friday week.\n\n## Before the launch\n\n\
         - [ ] Send the beta invite to the waiting list @2026-05-26\n\
         - [ ] Write the launch post @2026-05-28T15:00\n\
         - [x] Freeze the release branch @2026-05-25\n\
         - [ ] Record the demo video @2026-05-29\n\
         - [ ] Check the launch checklist with Sam @2026-06-02T10:30\n\
         - [ ] Draft a FAQ for support\n\n\
         ## After\n\n- [ ] Thank the beta testers @2026-06-08\n",
    );
    put(
        "Home",
        "# Home\n\n- [ ] Renew the car insurance @2026-05-21\n\
         - [ ] Book the plumber @2026-05-28\n- [ ] Plant the tomatoes @2026-05-31\n",
    );
    plan
}

/// The Tasks view in the list's place, with tasks overdue, today, to come
/// and undated, and the sidebar's overdue badge.
fn tasks_view(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let plan = seed_tasks(store);
    set_filter(cx, workspace, Filter::Tasks);
    cx.run_until_parked();
    let (library, tasks) = cx.update(|cx| {
        let workspace = workspace.read(cx);
        (workspace.library.clone(), workspace.tasks.clone())
    });
    cx.update(|cx| {
        library.update(cx, |library, cx| library.select(Some(plan), cx));
        tasks.update(cx, |tasks, cx| {
            tasks.reload_now(cx);
            tasks.board.step(1, crate::journal::now(cx));
        });
    });
}

/// The board over the window: the demo's notes in a column per status,
/// one card selected.
fn status_board(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show_board(cx, window, workspace, kanso_engine::board::BoardKind::Notes);
}

/// The tasks board: the demo's tasks as To do and Done.
fn task_board(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    seed_tasks(store);
    show_board(cx, window, workspace, kanso_engine::board::BoardKind::Tasks);
}

fn show_board(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    kind: kanso_engine::board::BoardKind,
) {
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| workspace.show_board(kind, window, cx));
    })
    .unwrap();
    cx.update(|cx| {
        let board = workspace.read(cx).board.clone().unwrap();
        board.update(cx, |board, cx| {
            board.reload_now(cx);
            board.board.step_across(1);
            board.board.step_down(1);
        });
    });
}

/// A note with due dates: chips in the preview, warm red when overdue.
fn due_chips(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let plan = seed_tasks(store);
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| {
        library.update(cx, |library, cx| {
            library.reload(cx);
            library.select(Some(plan), cx);
        })
    });
}

/// A note with two misspelled words underlined, red and wavy, beside code,
/// a link, a tag and a fenced block whose words are left alone. A word list
/// stands in for the system's dictionary.
fn spelling(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let body = "# Spelling\n\nThe quick brown fox jumpd over the lazzy dog. Code like \
                `recieve`, links like [Kanso](https://kanso.app/fnord) and tags like \
                #wrok are left alone.\n\n```\nteh fnord\n```\n";
    let known: Vec<&str> = crate::spellcheck::prose_words(body)
        .into_iter()
        .map(|r| &body[r])
        .filter(|w| !matches!(*w, "jumpd" | "lazzy"))
        .collect();
    let words = crate::spellcheck::WordList::new(&known);
    cx.update(|cx| crate::spellcheck::set_dictionary(Arc::new(words), cx));
    let book = store.books().unwrap()[0].id.clone();
    let id = store
        .put(
            DocType::Note,
            json!({"title": "Spelling", "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| {
        library.update(cx, |library, cx| {
            library.reload(cx);
            library.select(Some(id), cx);
        })
    });
}

/// Replace in all notes, finding "release" with a preview of each note's
/// matches and one note left out.
fn replace_sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    seed_tasks(store);
    cx.update(|cx| cx.set_reduce_motion(true));
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    cx.update_window(window, |_, window, cx| {
        let view = crate::replace::open_replace(editor, window, cx);
        view.update(cx, |view, cx| {
            view.find
                .update(cx, |find, cx| find.set_value("release", window, cx));
            view.with
                .update(cx, |with, cx| with.set_value("launch", window, cx));
            view.search_blocking(cx);
            if let Some(last) = view.found.last().map(|n| n.id.clone()) {
                view.excluded.insert(last);
            }
        });
    })
    .unwrap();
}

/// Notes tagged in a tree (`work/acme`, `home/garden`): the sidebar shows
/// the nested tags as rows under the chips, with `work` open and chosen,
/// which lists the notes tagged anywhere under it.
fn tags_tree(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let book = store.books().unwrap()[0].id.clone();
    let tag = |name: &str| store.tag_named(name, Origin::Ui).unwrap();
    let notes = [
        ("Acme kickoff", vec![tag("work/acme")]),
        ("Acme Q3 review", vec![tag("work/acme/q3")]),
        ("Hiring loop", vec![tag("work/internal")]),
        ("Team offsite", vec![tag("work")]),
        ("Raised beds", vec![tag("home/garden")]),
    ];
    for (title, tags) in notes {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": format!("# {title}\n"), "bookId": book, "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    cx.update(|cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| {
            sidebar.open_tag_level("work", cx);
            sidebar.open_tag_level("home", cx);
        });
    });
    set_filter(cx, workspace, Filter::TagPath("work".into()));
}

/// Two templates in the Templates notebook, and New from template… open.
fn template_picker(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let templates = store
        .role_book(kanso_core::roles::Role::Templates, Origin::Ui)
        .unwrap();
    for (title, body) in [
        (
            "Meeting {{date}}",
            "## Attendees\n\n- {{cursor}}\n\n## Notes\n\n## Actions\n\n- [ ] \n",
        ),
        (
            "Weekly review",
            "# Week {{date:%V}}\n\n## Went well\n\n## To change\n",
        ),
    ] {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": templates}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    open_palette(cx, window, workspace, Mode::Pick(Picker::Template), "");
}

/// Today's note (⌘D), made from the daily template, with an earlier day's
/// note to step back to: the ‹ › beside the title.
fn daily_note(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let templates = store
        .role_book(kanso_core::roles::Role::Templates, Origin::Ui)
        .unwrap();
    let template = store
        .put(
            DocType::Note,
            json!({
                "title": "Daily",
                "body": "# {{date:%A %e %B}}\n\n## Plan\n\n- {{cursor}}\n\n## Log\n\n- {{time}} Started the day\n",
                "bookId": templates,
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .set_daily_template(Some(&template), Origin::Ui)
        .unwrap();
    let earlier = kanso_core::dates::Date::new(2026, 5, 26).unwrap();
    store
        .daily_note(
            earlier,
            &kanso_store::NewNote {
                title: kanso_core::roles::daily_title(earlier),
                body: "## Log\n\n- Shipped the importer\n".into(),
                tags: Vec::new(),
            },
            Origin::Ui,
        )
        .unwrap();
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| workspace.open_today(window, cx));
    })
    .unwrap();
}

/// The Journal's calendar over today's note: May 2026 with a few days'
/// notes marked, today (the 28th) ringed, Monday first.
fn journal_calendar(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    daily_note(cx, window, workspace, store);
    for day in [4, 11, 12, 19, 21] {
        let date = kanso_core::dates::Date::new(2026, 5, day).unwrap();
        store
            .daily_note(
                date,
                &kanso_store::NewNote {
                    title: kanso_core::roles::daily_title(date),
                    ..Default::default()
                },
                Origin::Ui,
            )
            .unwrap();
    }
    cx.run_until_parked();
    cx.update(|cx| cx.set_reduce_motion(true));
    let view = cx
        .update_window(window, |_, window, cx| {
            workspace.update(cx, |workspace, cx| {
                crate::calendar_view::open_calendar(workspace, window, cx)
            })
        })
        .unwrap();
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.week_start = 0;
            view.read_days();
            cx.notify();
        })
    });
}

const VIM_NOTE: &str = "# Vim keys\n\nMove with h j k l, by words with w and b, and change a word with cw.\n\n- Delete with d and a motion: dw, d$ or dip\n- Yank with y and put it back with p\n- Visual mode selects; V takes whole lines\n\nEscape leaves Insert mode, and Escape again leaves focus mode.\n";

/// A note with Vim keys on, after `keys` (space-separated keystrokes);
/// `typed` is then typed in Insert mode.
fn vim_note(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
    keys: &str,
    typed: &str,
) {
    open_new_note(cx, workspace, store, "Vim keys", VIM_NOTE);
    cx.run_until_parked();
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    // The editor paints its caret and selection only in the active window,
    // as on screen.
    cx.update_window(window, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        let settings = cx.global::<crate::workspace::AppState>().config.editor;
        editor.update(cx, |editor, cx| {
            editor.set_settings(
                crate::config::EditorSettings {
                    vim: true,
                    ..settings
                },
                cx,
            );
            editor.focus_body(window, cx);
        });
    })
    .unwrap();
    cx.run_until_parked();
    for key in keys.split(' ').filter(|key| !key.is_empty()) {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.dispatch_keystroke(gpui_kit::Keystroke::parse(key).unwrap(), cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
    if !typed.is_empty() {
        use gpui_kit::EntityInputHandler as _;
        cx.update_window(window, |_, window, cx| {
            let body = editor.read(cx).body_state();
            body.update(cx, |state, cx| {
                state.replace_text_in_range(None, typed, window, cx)
            });
        })
        .unwrap();
        cx.run_until_parked();
    }
    // The shot is taken after the clock has moved on a whole number of
    // blinks that leaves the editor's caret in its hidden half; half a
    // second more shows it (Insert mode's line caret, and none beside
    // Normal mode's block).
    cx.advance_clock(std::time::Duration::from_millis(500));
}

/// Vim's Normal mode: the block caret on a word, "NORMAL" in the footer.
fn vim_normal(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    vim_note(cx, window, workspace, store, "j j w w w w", "");
}

/// A command half typed: "2d" waits in the footer.
fn vim_pending(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    vim_note(cx, window, workspace, store, "j j w w w w 2 d", "");
}

/// Insert mode after `cw`, a word typed: the line caret.
fn vim_insert(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    vim_note(cx, window, workspace, store, "j j w w w w c w", "glide");
}

/// Visual mode over a few words.
fn vim_visual(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    vim_note(cx, window, workspace, store, "j j w w w w v e e", "");
}

/// Visual Line mode over two list items.
fn vim_visual_line(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    vim_note(cx, window, workspace, store, "j j j j w shift-v j", "");
}

/// ⌘P with "sync" typed: notes, then the headings that match.
fn palette_headings(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Notes, "sync");
    cx.advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| palette.select_first_heading(cx));
    });
}

// MARK: Updates

/// The demo release's notes, as git-cliff writes a section.
const DEMO_NOTES: &str = "## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.2.0) - 2026-05-27\n\n### New\n\n- Kanso updates itself: new releases download in the background and install on restart ([3f9c2e1](https://github.com/iamngoni/kanso/commit/3f9c2e1))\n- Check for Updates… in the app menu, the palette and Settings ([b81d07a](https://github.com/iamngoni/kanso/commit/b81d07a))\n\n### Improved\n\n- Code blocks color 23 languages ([3709232](https://github.com/iamngoni/kanso/commit/3709232))\n- Quotes and finished tasks have their own colors ([27f54a0](https://github.com/iamngoni/kanso/commit/27f54a0))\n\n### Fixed\n\n- The gap under the title in focus mode is gone ([f5a06dd](https://github.com/iamngoni/kanso/commit/f5a06dd)) (#12)\n\n### Under the hood\n\n- Strip symbols from release builds ([60b6c92](https://github.com/iamngoni/kanso/commit/60b6c92))\n";

fn demo_update() -> kanso_update::Update {
    kanso_update::Update {
        version: "0.2.0".into(),
        pub_date: "2026-05-27T16:00:00Z".into(),
        release_url: "https://github.com/iamngoni/kanso/releases/tag/v0.2.0".into(),
        notes: DEMO_NOTES.into(),
        asset: kanso_update::Asset {
            url: "https://github.com/iamngoni/kanso/releases/download/v0.2.0/Kanso-0.2.0-macos-arm64.app.tar.gz".into(),
            sha256: "ab".repeat(32),
            size: 18_400_000,
        },
    }
}

fn demo_pending() -> kanso_update::Pending {
    let update = demo_update();
    kanso_update::Pending {
        version: update.version,
        payload: "/Applications/.kanso-update/unpacked-0.2.0/Kanso.app".into(),
        target: "/Applications/Kanso.app".into(),
        pub_date: update.pub_date,
        release_url: update.release_url,
        notes: update.notes,
        seal: Default::default(),
    }
}

/// The updater in `phase`, with nothing reachable; installable as from
/// /Applications.
fn set_updates(cx: &mut HeadlessAppContext, phase: Phase, manual: bool) {
    cx.update(|cx| {
        let now = Clock::now_ms(cx);
        let mut updates = Updates::default();
        updates.phase = phase;
        updates.manual = manual;
        updates.last_checked_ms = Some(now - 12 * MINUTE);
        updates.install = Ok(kanso_update::Install {
            target: "/Applications/Kanso.app".into(),
            stage_dir: "/Applications/.kanso-update".into(),
            platform: kanso_update::Platform::MacosArm64,
        });
        cx.set_global(updates);
    });
}

fn update_checking(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Checking, true);
}

fn update_downloading(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(
        cx,
        Phase::Downloading {
            update: demo_update(),
            done: 7_728_000,
            total: 18_400_000,
        },
        false,
    );
}

fn update_ready(cx: &mut HeadlessAppContext, _: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    set_updates(cx, Phase::Ready(demo_pending()), false);
}

fn update_available(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Available(demo_update()), false);
    cx.update(|cx| cx.global_mut::<AppState>().config.updates.auto_install = false);
}

fn update_failed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Failed(UpdateProblem::Verification), false);
}

/// The rail with an update ready.
fn rail_update_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    set_updates(cx, Phase::Ready(demo_pending()), false);
}

/// The ready pill clicked: the sheet with the release's notes.
fn update_sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Ready(demo_pending()), false);
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::updates::open_sheet(window, cx)
    })
    .unwrap();
}

/// The first launch after an update: the toast that offers What's new.
fn update_toast(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::UpToDate, false);
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        let notes = crate::updates::ReleaseNotes::from(&demo_update());
        let updates = cx.global_mut::<Updates>();
        updates.installed_notes = Some(notes.clone());
        updates.just_updated = Some(notes);
    });
    cx.update_window(window, |_, window, cx| crate::updates::announce(window, cx))
        .unwrap();
}

/// Settings › Updates with an update ready, up to date, and in Kanso Dev.
fn settings_updates_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Ready(demo_pending()), false);
    show_settings(cx, window, workspace, SettingsPage::Updates);
}

fn settings_updates_current(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::UpToDate, true);
    cx.update(|cx| {
        cx.global_mut::<Updates>().installed_notes =
            Some(crate::updates::ReleaseNotes::from(&demo_update()));
    });
    show_settings(cx, window, workspace, SettingsPage::Updates);
}

fn settings_updates_failed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Failed(UpdateProblem::Offline), true);
    show_settings(cx, window, workspace, SettingsPage::Updates);
}

/// Kanso Dev: the badge by the name, and its Updates page.
fn dev_flavour(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_updates(cx, Phase::Idle, false);
    cx.update(|cx| {
        let updates = cx.global_mut::<Updates>();
        updates.off = crate::updates::off_reason(
            &crate::flavour::DEV,
            true,
            Some(kanso_update::Platform::MacosArm64),
        );
        updates.install = Err(kanso_update::NotInstallable::DevFlavour);
        updates.last_checked_ms = None;
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| {
            sidebar.badge = crate::flavour::DEV.badge;
            cx.notify();
        });
    });
    show_settings(cx, window, workspace, SettingsPage::Updates);
}

fn open(
    cx: &mut HeadlessAppContext,
    store: Arc<Store>,
    data_dir: PathBuf,
    shot: &Shot,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let mut config = AppConfig::default();
    config.appearance.theme = shot.theme;
    config.appearance.themes = ThemeSettings::from_preference(shot.theme);
    if let Some(name) = shot.ui_theme {
        config.appearance.themes.ui = name.to_owned();
    }
    config.appearance.themes.syntax = shot.syntax_theme.map(str::to_owned);
    config.appearance.themes.preview = shot.preview_theme.map(str::to_owned);
    config.appearance.view_mode = shot.view_mode;
    config.last_note = store
        .search_summaries(&Default::default())
        .ok()
        .and_then(|notes| notes.into_iter().next())
        .map(|n| n.id);
    let now = cx.update(|cx| Clock::now_ms(cx));
    cx.update(|cx| {
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        // The footer as it usually reads, signed in; nothing is listening
        // or syncing, and no server or Keychain is reached.
        let mut services = Services::default();
        services.api = ApiState::Running("http://127.0.0.1:19840".into());
        services.account = AccountState::SignedIn {
            username: DEMO_USER.into(),
        };
        services.sync = SyncState::Synced {
            at_ms: now - 2 * MINUTE,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: None,
        };
        cx.set_global(services);
        apply_theme(None, cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Default::default(),
                    size: shot.size,
                })),
                focus: false,
                show: false,
                ..Default::default()
            },
            cx,
            |window, cx| {
                apply_theme(Some(window), cx);
                cx.new(|cx| {
                    let mut workspace = Workspace::new(window, cx);
                    workspace.placeholder_traffic_lights = true;
                    workspace
                })
            },
        )
    })
    .unwrap()
}

/// The UI colors a shot is drawn in.
fn shot_colors(shot: &Shot) -> &'static Colors {
    let dark = shot.theme == ThemePreference::Dark;
    match shot.ui_theme {
        Some(name) => {
            let entry = crate::themes::builtins()
                .iter()
                .find(|e| e.name() == name)
                .unwrap_or_else(|| panic!("no built-in theme {name}"));
            Box::leak(Box::new(Colors::from_engine(&entry.resolved.colors)))
        }
        None if dark => &DARK,
        None => &LIGHT,
    }
}

/// Checks that the pixel at point `at` (in window points) is `expected`,
/// within `tolerance` per channel.
fn assert_surface(
    image: &image::RgbaImage,
    scale: f32,
    at: (f32, f32),
    expected: Rgb,
    tolerance: u8,
    what: &str,
) {
    let (x, y) = ((at.0 * scale) as u32, (at.1 * scale) as u32);
    let pixel = image.get_pixel(x, y).0;
    let want = [16, 8, 0].map(|shift| ((expected.0 >> shift) & 0xFF) as u8);
    let close = (0..3).all(|i| pixel[i].abs_diff(want[i]) <= tolerance);
    assert!(
        close,
        "{what} at {at:?}: got #{:02X}{:02X}{:02X}, want {}",
        pixel[0],
        pixel[1],
        pixel[2],
        expected.hex()
    );
}

/// The palette's panel is painted in the elevated color, centred in the
/// window. The point sits in its padding, left of the search field.
fn check_palette(image: &image::RgbaImage, shot: &Shot) {
    let colors = shot_colors(shot);
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    let corner = (
        (width - PANEL_WIDTH) / 2.,
        ((height - PANEL_HEIGHT) / 2.).floor(),
    );
    assert_surface(
        image,
        scale,
        (corner.0 + 6., corner.1 + 6.),
        colors.elevated,
        2,
        &format!("{}: palette", shot.name),
    );
}

/// A dialog: its surface in the editor's color, centred across the window
/// a tenth of the way down. The point sits in its top padding.
fn check_dialog(image: &image::RgbaImage, shot: &Shot, dialog_width: f32) {
    let colors = shot_colors(shot);
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    let corner = ((width - dialog_width) / 2., height / 10.);
    let sample_y = shot.dialog_sample_y.unwrap_or(corner.1 + 6.);
    assert_surface(
        image,
        scale,
        (corner.0 + 24., sample_y),
        colors.editor,
        2,
        &format!("{}: dialog", shot.name),
    );
}

/// Settings: the sidebar, then the page column in the list's color and
/// the page in the editor's, each band running across.
fn check_settings_surfaces(image: &image::RgbaImage, shot: &Shot) {
    let colors = shot_colors(shot);
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    // Pane sizes in window points, at the shot's zoom.
    let zoom = crate::zoom::scale();
    let fitted = crate::layout::PaneWidths::default().fitted(width / zoom, None);
    let left = zoom
        * match shot.left {
            Left::Sidebar => fitted.sidebar,
            Left::Rail => rail_width(false),
            Left::Nothing => 0.,
        };
    let nav_width = NAV_WIDTH * zoom;
    // Snapped to a device pixel, as the divider is drawn.
    let band_top = ((height - BOTTOM_BAND_HEIGHT * zoom) * scale).round() / scale;
    let mut checks = vec![
        ((left + 6., height / 2.), colors.list, "page column"),
        (
            (left + nav_width - 6., height / 2.),
            colors.list,
            "page column's right edge",
        ),
        (
            (left + nav_width + 8., 4.),
            colors.editor,
            "page's top band",
        ),
        ((width - 4., height / 2.), colors.editor, "page"),
        (
            (left + 6., band_top),
            colors.divider,
            "page column's bottom divider",
        ),
        (
            (width - 4., band_top),
            colors.divider,
            "page's bottom divider",
        ),
    ];
    if shot.left != Left::Nothing {
        checks.push(((4., height / 2.), colors.sidebar, "sidebar"));
    }
    for (at, expected, what) in checks {
        assert_surface(
            image,
            scale,
            at,
            expected,
            2,
            &format!("{}: {what}", shot.name),
        );
    }
}

/// Each pane is painted in its own surface color. The points sit in
/// padding that no content reaches.
fn check_surfaces(image: &image::RgbaImage, shot: &Shot, inspector: bool) {
    let colors = shot_colors(shot);
    // The inspector, when open, is the window's right edge, in the list's
    // surface.
    let right = if inspector {
        colors.list
    } else {
        colors.editor
    };
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    // Pane sizes in window points, at the shot's zoom.
    // A zoomed window has fewer of them, so the panes may be fitted
    // narrower, as the workspace draws them.
    let zoom = crate::zoom::scale();
    let rail = (shot.left == Left::Rail).then(|| rail_width(false));
    let fitted = crate::layout::PaneWidths::default().fitted(width / zoom, rail);
    let list_width = fitted.list * zoom;
    // The bands are part of their panes: one surface from top to bottom.
    let mut checks = vec![
        ((width - 4., 4.), right, "right pane's top band"),
        (
            (width - 12., height / 2.),
            right,
            "editor, preview or inspector",
        ),
        (
            (width - 4., height - BOTTOM_BAND_HEIGHT * zoom / 2.),
            right,
            "right pane's bottom band",
        ),
    ];
    // Snapped to a device pixel, as the divider is drawn.
    let band_top = ((height - BOTTOM_BAND_HEIGHT * zoom) * scale).round() / scale;
    // The panes keep their widths whatever the window's size; the sidebar
    // and its rail are the same surface.
    let leading = match shot.left {
        Left::Sidebar => Some(fitted.sidebar * zoom),
        Left::Rail => Some(rail_width(false) * zoom),
        Left::Nothing => None,
    };
    match leading {
        Some(leading) => {
            checks.extend([
                ((4., height / 2.), colors.sidebar, "sidebar"),
                ((leading - 6., 4.), colors.sidebar, "sidebar's top band"),
                ((leading + 6., 4.), colors.list, "note list"),
                (
                    (leading + list_width - 6., 4.),
                    colors.list,
                    "note list's right edge",
                ),
                (
                    (leading + list_width + 8., 4.),
                    colors.editor,
                    "editor's left edge",
                ),
                // The bottom band's top edge runs across every pane, with a
                // note open or not.
                ((4., band_top), colors.divider, "sidebar's bottom divider"),
                (
                    (leading + 6., band_top),
                    colors.divider,
                    "note list's bottom divider",
                ),
                (
                    (width - 4., band_top),
                    colors.divider,
                    "editor's bottom divider",
                ),
            ]);
        }
        // Focus mode: the editor's surface edge to edge, and a quiet footer.
        None => checks.extend([
            ((4., 4.), colors.editor, "focus mode's top band"),
            ((4., height / 2.), colors.editor, "focus mode's left margin"),
            (
                (4., band_top),
                colors.editor,
                "focus mode's footer, without a divider",
            ),
        ]),
    }
    for (at, expected, what) in checks {
        assert_surface(
            image,
            scale,
            at,
            expected,
            2,
            &format!("{}: {what}", shot.name),
        );
    }
}

/// One theme's three cards, as the Themes page draws them.
struct ThemeCards(Arc<crate::themes::ThemeEntry>);

impl gpui_kit::Render for ThemeCards {
    fn render(
        &mut self,
        _: &mut Window,
        _: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use crate::settings_themes::{
            CARD_WIDTH, THUMB_HEIGHT, preview_thumb, syntax_thumb, ui_thumb,
        };
        use gpui_kit::{ParentElement as _, Styled as _, div};
        let entry = &self.0;
        let card = |thumb: gpui_kit::Div| {
            thumb
                .flex_none()
                .w(px(CARD_WIDTH))
                .h(px(THUMB_HEIGHT))
                .rounded(px(6.))
                .overflow_hidden()
        };
        div()
            .size_full()
            .flex()
            .gap(px(THUMB_GAP))
            .p(px(THUMB_GAP))
            .bg(crate::theme::hsla(entry.resolved.colors.app))
            .child(card(ui_thumb(entry)))
            .child(card(syntax_thumb(entry)))
            .child(card(preview_thumb(entry)))
    }
}

const THUMB_GAP: f32 = 12.;

/// Renders each theme's three cards (UI, syntax, preview) to
/// `<file stem>.png` in `KANSO_THEME_THUMBS_DIR`: the themes in
/// `KANSO_THEMES_DIR` when set (a checkout of the themes repository, say),
/// else the built-in ones. `themes/scripts/check.sh` runs it.
#[test]
#[ignore = "writes PNGs; run on demand with KANSO_THEME_THUMBS_DIR set"]
fn theme_thumbnails() {
    use crate::settings_themes::{CARD_WIDTH, THUMB_HEIGHT};
    use crate::theme_file::ThemeFile;
    use crate::themes::ThemeEntry;
    let out =
        PathBuf::from(std::env::var("KANSO_THEME_THUMBS_DIR").expect("set KANSO_THEME_THUMBS_DIR"));
    std::fs::create_dir_all(&out).unwrap();
    let themes: Vec<(String, Arc<ThemeEntry>)> = match std::env::var_os("KANSO_THEMES_DIR") {
        Some(dir) => {
            let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| {
                    p.extension().is_some_and(|x| x == "json") && !p.ends_with("index.json")
                })
                .collect();
            files.sort();
            files
                .into_iter()
                .map(|path| {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let theme = ThemeFile::parse(&text)
                        .unwrap_or_else(|e| panic!("{} {e}", path.display()));
                    let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
                    (
                        stem.clone(),
                        Arc::new(ThemeEntry::new(
                            theme,
                            stem,
                            Some(path),
                            crate::origin::Origin::Own,
                        )),
                    )
                })
                .collect()
        }
        None => crate::themes::BUILTIN
            .iter()
            .zip(crate::themes::builtins())
            .map(|((file, _), entry)| (file.trim_end_matches(".json").to_owned(), entry.clone()))
            .collect(),
    };
    let window_size = size(
        px(3. * CARD_WIDTH + 4. * THUMB_GAP),
        px(THUMB_HEIGHT + 2. * THUMB_GAP),
    );
    for (stem, entry) in themes {
        let mut cx = context(None);
        cx.update(|cx| apply_theme(None, cx));
        let window = cx
            .update(|cx| {
                gpui_kit::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Default::default(),
                            size: window_size,
                        })),
                        focus: false,
                        show: false,
                        ..Default::default()
                    },
                    cx,
                    |_, cx| cx.new(|_| ThemeCards(entry.clone())),
                )
            })
            .unwrap()
            .0;
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        let image = cx.capture_screenshot(window).expect("Metal rendering");
        let path = out.join(format!("{stem}.png"));
        image.save(&path).unwrap();
        println!("wrote {}", path.display());
    }
}

/// The quick capture window on its own, in both themes: with a note being
/// written, and asking before Escape discards it.
#[test]
#[ignore = "writes PNGs; run on demand with KANSO_SNAPSHOT_DIR set"]
fn capture_window() {
    use crate::capture::{CaptureView, HEIGHT, WIDTH};
    let out = PathBuf::from(std::env::var("KANSO_SNAPSHOT_DIR").expect("set KANSO_SNAPSHOT_DIR"));
    std::fs::create_dir_all(&out).unwrap();
    for (name, theme, confirming) in [
        ("dark-capture", ThemePreference::Dark, false),
        ("light-capture", ThemePreference::Light, false),
        ("dark-capture-discard", ThemePreference::Dark, true),
        ("light-capture-discard", ThemePreference::Light, true),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(demo_store(&dir.path().join("kanso.db")));
        let mut cx = context(Some(Clock::fixed(DEMO_NOW_MS, DEMO_UTC_OFFSET_SECS)));
        let mut config = AppConfig::default();
        config.appearance.theme = theme;
        config.appearance.themes = ThemeSettings::from_preference(theme);
        cx.update(|cx| {
            cx.set_global(AppState {
                store: store.clone(),
                data_dir: dir.path().to_path_buf(),
                config,
            });
            crate::capture::bind_keys(cx);
            apply_theme(None, cx);
        });
        let (window, view) = cx
            .update(|cx| {
                gpui_kit::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Default::default(),
                            size: size(px(WIDTH), px(HEIGHT)),
                        })),
                        focus: false,
                        show: false,
                        ..Default::default()
                    },
                    cx,
                    |window, cx| {
                        apply_theme(Some(window), cx);
                        cx.new(|cx| CaptureView::new(store.clone(), window, cx))
                    },
                )
            })
            .unwrap();
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.set_text_for_test(
                    "Call Sam about the lease\n\n- bring the signed copy\n- ask about the garden #home/garden",
                    window,
                    cx,
                );
                if confirming {
                    view.cancel(window, cx);
                    assert!(view.is_confirming());
                }
            });
        })
        .unwrap();
        for _ in 0..2 {
            cx.advance_clock(std::time::Duration::from_millis(500));
            cx.run_until_parked();
            cx.update_window(window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        }
        let image = cx.capture_screenshot(window).expect("Metal rendering");
        let path = out.join(format!("{name}.png"));
        image.save(&path).unwrap();
        println!(
            "wrote {} ({}x{})",
            path.display(),
            image.width(),
            image.height()
        );
        // The text area in the editor's surface color.
        let colors = if theme == ThemePreference::Dark {
            &DARK
        } else {
            &LIGHT
        };
        let scale = image.width() as f32 / WIDTH;
        assert_surface(
            &image,
            scale,
            (WIDTH - 12., HEIGHT / 2.),
            colors.editor,
            3,
            name,
        );
    }
}

#[test]
#[ignore = "writes PNGs; run on demand with KANSO_SNAPSHOT_DIR set"]
fn snapshots() {
    let out = PathBuf::from(std::env::var("KANSO_SNAPSHOT_DIR").expect("set KANSO_SNAPSHOT_DIR"));
    std::fs::create_dir_all(&out).unwrap();
    use ThemePreference::{Dark, Light};
    use ViewMode::{Edit, Preview, Split};
    let full = size(px(1440.), px(900.));
    let shot = |name, theme, view_mode, prepare: Prepare| Shot {
        name,
        theme,
        ui_theme: None,
        syntax_theme: None,
        preview_theme: None,
        view_mode,
        size: full,
        prepare,
        left: Left::Sidebar,
        dialog_width: crate::history::DIALOG_WIDTH,
        dialog_sample_y: None,
    };
    // A shot with one of the account's dialogs open.
    let form = |name, theme, prepare: Prepare| Shot {
        dialog_width: crate::dialogs::FORM_WIDTH,
        ..shot(name, theme, Edit, prepare)
    };
    let mut shots = vec![
        shot("dark-split", Dark, Split, open_latest),
        shot("light-split", Light, Split, open_latest),
        shot("dark-edit", Dark, Edit, open_latest),
        shot("light-edit", Light, Edit, open_latest),
        shot("dark-tabs", Dark, Edit, open_tabs),
        shot("light-tabs", Light, Edit, open_tabs),
        shot("dark-notebook-open", Dark, Edit, notebook_open),
        Shot {
            dialog_width: crate::book_icons::PICKER_WIDTH,
            ..shot("dark-book-icons", Dark, Edit, book_icon_picker)
        },
        Shot {
            dialog_width: crate::book_icons::PICKER_WIDTH,
            ..shot("dark-book-icons-search", Dark, Edit, book_icon_search)
        },
        Shot {
            dialog_width: crate::book_icons::PICKER_WIDTH,
            ..shot("dark-book-icons-theme", Dark, Edit, book_icon_theme_section)
        },
        shot(
            "light-settings-icon-themes",
            Light,
            Edit,
            settings_icon_themes,
        ),
        shot("dark-icon-theme-phosphor", Dark, Edit, icon_theme_phosphor),
        shot(
            "light-icon-theme-phosphor",
            Light,
            Split,
            icon_theme_phosphor,
        ),
        shot("dark-preview", Dark, Preview, open_latest),
        shot("light-preview", Light, Preview, open_latest),
        shot("dark-edit-mono", Dark, Edit, mono_font),
        shot("dark-empty", Dark, Split, empty_notebook),
        shot("light-empty", Light, Split, empty_notebook),
        shot(
            "dark-notebook-of-notebooks",
            Dark,
            Edit,
            notebook_of_notebooks,
        ),
        shot("dark-search", Dark, Edit, search_sync),
        // The Status group open and the footer's warnings.
        shot("dark-status-warnings", Dark, Edit, status_and_warnings),
        shot("light-status-warnings", Light, Split, status_and_warnings),
        shot("dark-recent-tags", Dark, Preview, recent_all_tags),
        // The sidebar collapsed to its rail.
        Shot {
            left: Left::Rail,
            ..shot("dark-rail", Dark, Edit, collapse_sidebar)
        },
        Shot {
            left: Left::Rail,
            ..shot("light-rail", Light, Split, collapse_sidebar)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-notebooks", Dark, Edit, rail_notebooks)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-tags", Dark, Edit, rail_tags)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-tooltip", Dark, Edit, rail_tooltip)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-offline", Dark, Edit, rail_offline)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-expand", Dark, Edit, rail_expand_tooltip)
        },
        Shot {
            left: Left::Rail,
            ..shot("light-rail-expand", Light, Split, rail_expand_tooltip)
        },
        // The button that folds the sidebar, its tooltip open.
        shot("dark-sidebar-toggle", Dark, Edit, sidebar_toggle_tooltip),
        shot("dark-api-off", Dark, Edit, api_off),
        // The open note's earlier versions.
        shot("dark-history", Dark, Edit, history),
        shot("light-history", Light, Edit, history),
        // A shared note: who it is shared with, and its comments.
        Shot {
            dialog_width: crate::share_sheet::SHEET_WIDTH,
            ..shot("dark-share-sheet", Dark, Edit, share_sheet)
        },
        Shot {
            dialog_width: crate::share_sheet::SHEET_WIDTH,
            ..shot("light-share-sheet", Light, Edit, share_sheet)
        },
        shot("dark-shared-invitation", Dark, Edit, shared_invitation),
        shot("light-shared-invitation", Light, Edit, shared_invitation),
        shot("dark-comments", Dark, Edit, comments),
        shot("light-comments", Light, Split, comments),
        // Importing: choosing the source, the summary, the report.
        Shot {
            dialog_width: crate::library_import::WIDTH,
            ..shot("dark-import-choose", Dark, Edit, import_choose)
        },
        Shot {
            dialog_width: crate::library_import::WIDTH,
            ..shot("light-import-preview", Light, Edit, import_preview)
        },
        Shot {
            dialog_width: crate::library_import::WIDTH,
            ..shot("dark-import-done", Dark, Edit, import_done)
        },
        // Publishing the open note, in each of its states.
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("dark-publish-signed-out", Dark, Edit, publish_signed_out)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("light-publish-signed-out", Light, Edit, publish_signed_out)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("dark-publish-unpublished", Dark, Edit, publish_unpublished)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot(
                "light-publish-unpublished",
                Light,
                Edit,
                publish_unpublished,
            )
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("dark-publish-published", Dark, Edit, publish_published)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("light-publish-published", Light, Edit, publish_published)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("dark-publish-outdated", Dark, Edit, publish_outdated)
        },
        Shot {
            dialog_width: crate::publish::SHEET_WIDTH,
            ..shot("light-publish-outdated", Light, Edit, publish_outdated)
        },
        Shot {
            dialog_width: crate::publish_site::SHEET_WIDTH,
            ..shot(
                "dark-publish-site-unpublished",
                Dark,
                Edit,
                publish_site_unpublished,
            )
        },
        Shot {
            dialog_width: crate::publish_site::SHEET_WIDTH,
            ..shot(
                "light-publish-site-unpublished",
                Light,
                Edit,
                publish_site_unpublished,
            )
        },
        Shot {
            dialog_width: crate::publish_site::SHEET_WIDTH,
            ..shot(
                "dark-publish-site-published",
                Dark,
                Edit,
                publish_site_published,
            )
        },
        Shot {
            dialog_width: crate::publish_site::SHEET_WIDTH,
            ..shot(
                "light-publish-site-published",
                Light,
                Edit,
                publish_site_published,
            )
        },
        // Focus mode, at rest and with the pointer near the top.
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus", Dark, Edit, focus_mode)
        },
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-hover", Dark, Edit, focus_mode_hover)
        },
        Shot {
            left: Left::Nothing,
            ..shot("light-focus", Light, Edit, focus_mode)
        },
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-split", Dark, Split, focus_mode_hover)
        },
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-split-quiet", Dark, Split, focus_mode)
        },
        // Writing aids: the word goal, sized images, tables formatted.
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-word-goal", Dark, Edit, focus_word_goal)
        },
        Shot {
            left: Left::Nothing,
            ..shot("light-focus-word-goal", Light, Edit, focus_word_goal)
        },
        shot("dark-palette-word-goal", Dark, Edit, palette_word_goal),
        shot("dark-images-sized", Dark, Preview, sized_images),
        shot("light-images-sized", Light, Preview, sized_images),
        // Math, diagrams and drawings in the preview.
        shot("dark-rich-preview", Dark, Preview, rich_note),
        shot("light-rich-preview", Light, Preview, rich_note),
        shot("light-page-preview", Light, Preview, page_preview),
        shot("light-rich-split", Light, Split, rich_note),
        shot("dark-drawing", Dark, Preview, drawing_note),
        shot("light-drawing", Light, Preview, drawing_note),
        shot("dark-images-bar", Dark, Preview, bar_sized_images),
        shot("light-images-bar", Light, Split, bar_sized_images),
        shot("dark-image-dragging", Dark, Preview, dragging_image),
        shot("light-image-dragging", Light, Preview, dragging_image),
        shot(
            "dark-palette-resize-image",
            Dark,
            Split,
            palette_resize_image,
        ),
        shot("dark-new-drawing", Dark, Preview, new_drawing),
        shot("light-new-drawing", Light, Split, new_drawing),
        shot("dark-table-before", Dark, Edit, table_before),
        shot("dark-table-after", Dark, Edit, table_after),
        shot("dark-table-after-mono", Dark, Edit, table_after_mono),
        shot("light-table-after", Light, Split, table_after),
        Shot {
            size: size(px(1440.), px(1200.)),
            ..shot("dark-settings-editor-writing", Dark, Edit, settings_editor)
        },
        Shot {
            size: size(px(1440.), px(1200.)),
            ..shot(
                "light-settings-editor-writing",
                Light,
                Edit,
                settings_editor,
            )
        },
        // The divider where a drag left it, and the preview following the
        // source's scrolling.
        shot("dark-split-dragged", Dark, Split, split_dragged),
        shot("light-split-dragged", Light, Split, split_dragged),
        shot("dark-split-scrolled", Dark, Split, split_scrolled),
        // An empty library, and a library with no note open.
        shot("dark-first-run", Dark, Split, first_run),
        shot("light-first-run", Light, Split, first_run),
        shot("dark-no-note", Dark, Edit, no_note_open),
        shot("dark-trash", Dark, Edit, open_trashed),
        shot("dark-conflict", Dark, Edit, conflict),
        shot("light-search", Light, Edit, search_sync),
        shot("dark-list-no-results", Dark, Edit, search_nothing),
        shot("dark-list-search-focus", Dark, Edit, focus_search),
        shot("dark-list-focused", Dark, Edit, list_focused),
        shot("dark-list-hover", Dark, Edit, hover_third),
        shot("dark-list-title-sort", Dark, Edit, sort_by_title),
        shot("dark-list-filtered", Dark, Edit, only_completed),
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-list-long-titles", Dark, Edit, long_titles)
        },
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("light-list-long-titles", Light, Edit, long_titles)
        },
        // Every date group of the demo at once.
        Shot {
            size: size(px(1440.), px(1100.)),
            ..shot("dark-list-grouped", Dark, Edit, open_latest)
        },
        // A wide window gives the room to the editor, not the panes.
        Shot {
            size: size(px(1800.), px(1000.)),
            ..shot("dark-wide", Dark, Split, open_latest)
        },
        shot("dark-palette", Dark, Edit, palette),
        shot("dark-palette-query", Dark, Edit, palette_query),
        shot("light-palette-query", Light, Edit, palette_query),
        shot("dark-palette-move", Dark, Edit, palette_move),
        shot("dark-template-picker", Dark, Edit, template_picker),
        shot("light-template-picker", Light, Edit, template_picker),
        shot("dark-tags-tree", Dark, Edit, tags_tree),
        shot("light-tags-tree", Light, Edit, tags_tree),
        shot("dark-daily-note", Dark, Edit, daily_note),
        shot("light-daily-note", Light, Split, daily_note),
        Shot {
            dialog_width: crate::calendar_view::SHEET_WIDTH,
            ..shot("dark-journal-calendar", Dark, Edit, journal_calendar)
        },
        Shot {
            dialog_width: crate::calendar_view::SHEET_WIDTH,
            ..shot("light-journal-calendar", Light, Edit, journal_calendar)
        },
        shot("dark-tasks", Dark, Split, tasks_view),
        shot("light-tasks", Light, Split, tasks_view),
        shot("dark-board", Dark, Edit, status_board),
        shot("light-board", Light, Edit, status_board),
        shot("dark-task-board", Dark, Edit, task_board),
        shot("light-task-board", Light, Edit, task_board),
        shot("dark-due-chips", Dark, Preview, due_chips),
        shot("light-due-chips", Light, Split, due_chips),
        shot("light-spelling", Light, Edit, spelling),
        shot("dark-spelling", Dark, Edit, spelling),
        Shot {
            dialog_width: crate::replace::SHEET_WIDTH,
            ..shot("dark-replace", Dark, Edit, replace_sheet)
        },
        Shot {
            dialog_width: crate::replace::SHEET_WIDTH,
            ..shot("light-replace", Light, Edit, replace_sheet)
        },
        shot("dark-palette-tag", Dark, Edit, palette_tag),
        shot("light-palette-new-tag", Light, Edit, palette_new_tag),
        shot("dark-open-note", Dark, Split, note_picker),
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-split-min", Dark, Split, open_latest)
        },
        // A table wider than the narrow preview scrolls instead of clipping.
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-split-table", Dark, Split, open_wide_table)
        },
        // Tall enough for the whole demo note.
        Shot {
            size: size(px(1440.), px(1700.)),
            ..shot("dark-preview-long", Dark, Preview, open_latest)
        },
        Shot {
            size: size(px(1440.), px(1700.)),
            ..shot("light-preview-long", Light, Preview, open_latest)
        },
        // Discovery: the note's graph and related notes beside it, the
        // library's graph, and related notes in Settings in each state.
        shot(
            "dark-inspector-graph",
            Dark,
            Edit,
            discovery::inspector_graph,
        ),
        shot(
            "light-inspector-graph",
            Light,
            Split,
            discovery::inspector_graph,
        ),
        shot(
            "dark-inspector-graph-2",
            Dark,
            Edit,
            discovery::inspector_graph_two,
        ),
        shot("dark-graph", Dark, Edit, discovery::library_graph),
        shot("light-graph", Light, Edit, discovery::library_graph),
        shot(
            "dark-graph-hover",
            Dark,
            Edit,
            discovery::library_graph_hover,
        ),
        shot(
            "light-graph-notebook",
            Light,
            Edit,
            discovery::library_graph_notebook,
        ),
        shot(
            "dark-graph-large",
            Dark,
            Edit,
            discovery::library_graph_large,
        ),
        shot(
            "light-graph-large",
            Light,
            Edit,
            discovery::library_graph_large,
        ),
        shot(
            "dark-inspector-related",
            Dark,
            Edit,
            discovery::inspector_related,
        ),
        shot(
            "light-inspector-related",
            Light,
            Split,
            discovery::inspector_related,
        ),
        shot(
            "dark-palette-meaning",
            Dark,
            Edit,
            discovery::palette_meaning,
        ),
        shot(
            "light-inspector-related-tags",
            Light,
            Edit,
            discovery::inspector_related_tags,
        ),
        shot(
            "dark-palette-suggested-tags",
            Dark,
            Edit,
            discovery::palette_suggested_tags,
        ),
        shot("dark-editor-twin", Dark, Edit, discovery::editor_twin),
        shot("light-editor-twin", Light, Split, discovery::editor_twin),
        shot(
            "dark-inspector-related-off",
            Dark,
            Edit,
            discovery::inspector_related_off,
        ),
        shot(
            "light-inspector-related-off",
            Light,
            Edit,
            discovery::inspector_related_off,
        ),
        shot(
            "dark-inspector-related-downloading",
            Dark,
            Edit,
            discovery::inspector_related_downloading,
        ),
        shot(
            "light-inspector-related-failed",
            Light,
            Edit,
            discovery::inspector_related_failed,
        ),
        shot(
            "dark-settings-related-off",
            Dark,
            Edit,
            discovery::settings_related_off,
        ),
        shot(
            "light-settings-related-off",
            Light,
            Edit,
            discovery::settings_related_off,
        ),
        shot(
            "dark-settings-related-downloading",
            Dark,
            Edit,
            discovery::settings_related_downloading,
        ),
        shot(
            "light-settings-related-downloading",
            Light,
            Edit,
            discovery::settings_related_downloading,
        ),
        shot(
            "dark-settings-related-indexing",
            Dark,
            Edit,
            discovery::settings_related_indexing,
        ),
        shot(
            "light-settings-related-indexing",
            Light,
            Edit,
            discovery::settings_related_indexing,
        ),
        shot(
            "dark-settings-related-ready",
            Dark,
            Edit,
            discovery::settings_related_ready,
        ),
        shot(
            "light-settings-related-ready",
            Light,
            Edit,
            discovery::settings_related_ready,
        ),
        shot(
            "dark-settings-related-failed",
            Dark,
            Edit,
            discovery::settings_related_failed,
        ),
        shot(
            "light-settings-related-failed",
            Light,
            Edit,
            discovery::settings_related_failed,
        ),
        // Settings, in the main window.
        // Zoom: the whole app, at either end and in Settings.
        shot("light-zoom-150-split", Light, Split, zoom_largest),
        shot("dark-zoom-150-split", Dark, Split, zoom_largest),
        shot("light-zoom-80-split", Light, Split, zoom_smallest),
        shot(
            "light-zoom-120-settings-general",
            Light,
            Edit,
            zoom_settings_general,
        ),
        shot("dark-settings-general", Dark, Edit, settings_general),
        shot("light-settings-general", Light, Edit, settings_general),
        shot("dark-settings-editor", Dark, Edit, settings_editor),
        // Voice input: the pill while listening and writing, and Settings'
        // Voice input in each state of the model.
        shot("dark-voice-listening", Dark, Edit, voice::editor_listening),
        shot(
            "light-voice-listening",
            Light,
            Edit,
            voice::editor_listening,
        ),
        shot("dark-voice-writing", Dark, Edit, voice::editor_writing),
        shot(
            "dark-editor-inline-suggestion",
            Dark,
            Edit,
            ai::editor_inline_suggestion,
        ),
        shot(
            "light-editor-inline-suggestion",
            Light,
            Split,
            ai::editor_inline_suggestion,
        ),
        shot(
            "dark-editor-transcribing",
            Dark,
            Edit,
            voice::editor_transcribing,
        ),
        shot(
            "light-editor-transcribing",
            Light,
            Split,
            voice::editor_transcribing,
        ),
        shot(
            "light-settings-voice-missing",
            Light,
            Edit,
            voice::settings_voice_missing,
        ),
        shot(
            "dark-settings-voice-downloading",
            Dark,
            Edit,
            voice::settings_voice_downloading,
        ),
        shot(
            "light-settings-voice-ready",
            Light,
            Edit,
            voice::settings_voice_ready,
        ),
        shot(
            "dark-settings-voice-failed",
            Dark,
            Edit,
            voice::settings_voice_failed,
        ),
        shot("light-settings-ai-api", Light, Edit, ai::settings_ai_api),
        shot(
            "light-settings-ai-agent-ready",
            Light,
            Edit,
            ai::settings_ai_agent_ready,
        ),
        shot(
            "dark-settings-ai-agent-ready",
            Dark,
            Edit,
            ai::settings_ai_agent_ready,
        ),
        shot(
            "light-settings-ai-agent-sign-in",
            Light,
            Edit,
            ai::settings_ai_agent_sign_in,
        ),
        shot(
            "light-settings-ai-agent-not-installed",
            Light,
            Edit,
            ai::settings_ai_agent_not_installed,
        ),
        shot(
            "dark-settings-ai-agent-failed",
            Dark,
            Edit,
            ai::settings_ai_agent_failed,
        ),
        shot(
            "light-settings-ai-agent-custom",
            Light,
            Edit,
            ai::settings_ai_agent_custom,
        ),
        shot(
            "light-settings-ai-agent-registry",
            Light,
            Edit,
            ai::settings_ai_agent_registry,
        ),
        Shot {
            dialog_width: crate::agent_scope::SHEET_WIDTH,
            ..shot(
                "light-settings-ai-agent-scope",
                Light,
                Edit,
                ai::settings_ai_agent_scope,
            )
        },
        shot("dark-settings-appearance", Dark, Edit, settings_appearance),
        shot("dark-settings-gallery-browse", Dark, Edit, settings_gallery),
        shot(
            "light-settings-gallery-browse",
            Light,
            Edit,
            settings_gallery,
        ),
        // The Account page, signed in.
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot("dark-settings-account", Dark, Edit, settings_account)
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot("light-settings-account", Light, Edit, settings_account)
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked",
                Dark,
                Edit,
                settings_account_locked,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-waiting",
                Dark,
                Edit,
                settings_account_locked_waiting,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-code",
                Dark,
                Edit,
                settings_account_locked_code,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-no-devices",
                Dark,
                Edit,
                settings_account_locked_no_devices,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-cancelled",
                Dark,
                Edit,
                settings_account_locked_cancelled,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-start-again",
                Dark,
                Edit,
                settings_account_locked_expired,
            )
        },
        Shot {
            dialog_width: crate::dialogs::FORM_WIDTH,
            dialog_sample_y: Some(30.),
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-password",
                Dark,
                Edit,
                settings_account_locked_password,
            )
        },
        Shot {
            dialog_width: crate::dialogs::FORM_WIDTH,
            dialog_sample_y: Some(30.),
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-locked-recovery",
                Dark,
                Edit,
                settings_account_locked_recovery,
            )
        },
        shot(
            "dark-settings-account-syncing",
            Dark,
            Edit,
            settings_account_syncing,
        ),
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-methods",
                Dark,
                Edit,
                settings_account_methods,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "light-settings-account-methods",
                Light,
                Edit,
                settings_account_methods,
            )
        },
        Shot {
            size: size(px(1440.), px(1400.)),
            ..shot(
                "dark-settings-account-methods-linking",
                Dark,
                Edit,
                settings_account_methods_linking,
            )
        },
        form("dark-proof-choice", Dark, proof_choice),
        form("dark-proof-unlink-last", Dark, proof_unlink_last),
        form("light-proof-set-password", Light, proof_set_password),
        form("dark-proof-remove-passkey", Dark, proof_remove_passkey),
        shot(
            "dark-settings-account-offline",
            Dark,
            Edit,
            settings_account_offline,
        ),
        shot(
            "dark-settings-account-paused",
            Dark,
            Edit,
            settings_account_paused,
        ),
        shot(
            "light-settings-account-full",
            Light,
            Edit,
            settings_account_full,
        ),
        shot(
            "dark-settings-account-conflict",
            Dark,
            Edit,
            settings_account_conflict,
        ),
        shot(
            "dark-settings-account-loading",
            Dark,
            Edit,
            settings_account_loading,
        ),
        // Signed out: the ways in, each form, and a running sign-up.
        shot("dark-settings-signed-out", Dark, Edit, settings_signed_out),
        shot(
            "light-settings-signed-out",
            Light,
            Edit,
            settings_signed_out,
        ),
        shot(
            "dark-settings-create-account",
            Dark,
            Edit,
            settings_create_account,
        ),
        shot(
            "light-settings-create-account",
            Light,
            Edit,
            settings_create_account,
        ),
        shot(
            "dark-settings-github-wait",
            Dark,
            Edit,
            settings_github_wait,
        ),
        shot(
            "dark-settings-create-account-closed",
            Dark,
            Edit,
            settings_create_account_closed,
        ),
        shot(
            "dark-settings-create-account-custom",
            Dark,
            Edit,
            settings_create_account_custom,
        ),
        shot(
            "dark-settings-create-account-busy",
            Dark,
            Edit,
            settings_create_account_busy,
        ),
        shot(
            "light-settings-create-account-busy",
            Light,
            Edit,
            settings_create_account_busy,
        ),
        shot(
            "dark-settings-sign-in-methods",
            Dark,
            Edit,
            settings_sign_in_methods,
        ),
        shot(
            "light-settings-sign-in-methods",
            Light,
            Edit,
            settings_sign_in_methods,
        ),
        shot(
            "dark-settings-create-methods",
            Dark,
            Edit,
            settings_create_methods,
        ),
        shot(
            "light-settings-create-methods",
            Light,
            Edit,
            settings_create_methods,
        ),
        shot(
            "dark-settings-create-username",
            Dark,
            Edit,
            settings_create_username,
        ),
        shot(
            "dark-settings-passkey-username",
            Dark,
            Edit,
            settings_passkey_username,
        ),
        shot(
            "light-settings-passkey-username",
            Light,
            Edit,
            settings_passkey_username,
        ),
        shot(
            "dark-settings-security-key-pin",
            Dark,
            Edit,
            settings_security_key_pin,
        ),
        shot(
            "light-settings-security-key-pin",
            Light,
            Edit,
            settings_security_key_pin,
        ),
        shot("dark-settings-sign-in", Dark, Edit, settings_sign_in_error),
        shot(
            "light-settings-sign-in",
            Light,
            Edit,
            settings_sign_in_error,
        ),
        shot("dark-settings-recover", Dark, Edit, settings_recover),
        shot("light-settings-recover", Light, Edit, settings_recover),
        shot(
            "dark-settings-session-ended",
            Dark,
            Edit,
            settings_session_ended,
        ),
        shot(
            "light-settings-session-ended",
            Light,
            Edit,
            settings_session_ended,
        ),
        // The recovery key, after sign-up, recovery or a new key.
        shot(
            "dark-settings-recovery-key",
            Dark,
            Edit,
            settings_recovery_key,
        ),
        shot(
            "light-settings-recovery-key",
            Light,
            Edit,
            settings_recovery_key,
        ),
        shot(
            "dark-settings-recovery-key-saved",
            Dark,
            Edit,
            settings_recovery_key_saved,
        ),
        shot(
            "dark-settings-recovery-key-password",
            Dark,
            Edit,
            settings_recovery_key_password,
        ),
        // A key that came with a failure, or whose answer was lost.
        shot(
            "dark-settings-recovery-key-failed",
            Dark,
            Edit,
            settings_recovery_key_failed,
        ),
        shot(
            "light-settings-recovery-key-unconfirmed",
            Light,
            Edit,
            settings_recovery_key_unconfirmed,
        ),
        shot(
            "dark-settings-recovery-key-unconfirmed",
            Dark,
            Edit,
            settings_recovery_key_unconfirmed,
        ),
        shot(
            "dark-settings-recovery-key-taken",
            Dark,
            Edit,
            settings_recovery_key_taken,
        ),
        // The account's dialogs.
        form("dark-account-binding-question", Dark, binding_question),
        form("light-account-binding-question", Light, binding_question),
        form("dark-account-change-password", Dark, change_password_form),
        form("light-account-change-password", Light, change_password_form),
        form("dark-lock-password", Dark, lock_password_form),
        form(
            "dark-account-change-password-busy",
            Dark,
            change_password_busy,
        ),
        form("dark-account-new-recovery-key", Dark, new_recovery_key_form),
        form(
            "light-account-new-recovery-key",
            Light,
            new_recovery_key_form,
        ),
        form("dark-account-recover", Dark, recover_form),
        form("dark-account-delete", Dark, delete_account_form),
        form("light-account-delete", Light, delete_account_form),
        // A plain confirmation, at the dialogs' own width.
        Shot {
            dialog_width: 448.,
            ..shot("dark-account-sign-out", Dark, Edit, sign_out_confirm)
        },
        Shot {
            dialog_width: 448.,
            ..shot(
                "dark-account-sign-out-unsaved-key",
                Dark,
                Edit,
                sign_out_unsaved_key,
            )
        },
        form("dark-account-server", Dark, server_form),
        form("light-account-server", Light, server_form),
        // Signed out, in the notes: the footer and the rail.
        shot("dark-signed-out", Dark, Edit, signed_out_footer),
        shot("light-signed-out", Light, Split, signed_out_footer),
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-signed-out", Dark, Edit, rail_signed_out)
        },
        shot("dark-settings-api", Dark, Edit, settings_api),
        shot(
            "dark-settings-api-port-in-use",
            Dark,
            Edit,
            settings_api_port_in_use,
        ),
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-settings-min", Dark, Edit, settings_api)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-settings-rail", Dark, Edit, settings_rail)
        },
        Shot {
            left: Left::Rail,
            ..shot("light-settings-rail", Light, Edit, settings_rail)
        },
        shot("dark-settings-from-focus", Dark, Edit, settings_from_focus),
        shot("dark-settings-palette", Dark, Edit, settings_palette),
        // Updates: the sidebar's pill in each state, the rail, the sheet,
        // the toast after an update, Settings › Updates and Kanso Dev.
        shot("dark-update-checking", Dark, Split, update_checking),
        shot("light-update-checking", Light, Split, update_checking),
        shot("dark-update-downloading", Dark, Split, update_downloading),
        shot("light-update-downloading", Light, Split, update_downloading),
        shot("dark-update-ready", Dark, Split, update_ready),
        shot("light-update-ready", Light, Split, update_ready),
        shot("dark-update-available", Dark, Split, update_available),
        shot("light-update-available", Light, Split, update_available),
        shot("dark-update-failed", Dark, Split, update_failed),
        shot("light-update-failed", Light, Split, update_failed),
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-update-ready", Dark, Edit, rail_update_ready)
        },
        Shot {
            dialog_width: crate::updates::SHEET_WIDTH,
            ..shot("dark-update-sheet", Dark, Split, update_sheet)
        },
        Shot {
            dialog_width: crate::updates::SHEET_WIDTH,
            ..shot("light-update-sheet", Light, Split, update_sheet)
        },
        shot("dark-update-toast", Dark, Split, update_toast),
        shot("light-update-toast", Light, Split, update_toast),
        shot("dark-settings-updates", Dark, Edit, settings_updates_ready),
        shot(
            "light-settings-updates",
            Light,
            Edit,
            settings_updates_ready,
        ),
        shot(
            "dark-settings-updates-current",
            Dark,
            Edit,
            settings_updates_current,
        ),
        shot(
            "light-settings-updates-current",
            Light,
            Edit,
            settings_updates_current,
        ),
        shot(
            "dark-settings-updates-failed",
            Dark,
            Edit,
            settings_updates_failed,
        ),
        // Linked knowledge: the inspector, wiki-links, `[[` and headings
        // in ⌘P.
        shot("dark-inspector-outline", Dark, Edit, inspector_outline),
        shot("light-inspector-outline", Light, Split, inspector_outline),
        shot(
            "dark-inspector-backlinks",
            Dark,
            Preview,
            inspector_backlinks,
        ),
        shot(
            "light-inspector-backlinks",
            Light,
            Edit,
            inspector_backlinks,
        ),
        shot("dark-wiki-links", Dark, Split, wiki_links),
        shot("light-wiki-links", Light, Preview, wiki_links),
        shot("dark-wiki-autocomplete", Dark, Edit, wiki_autocomplete),
        shot("light-wiki-autocomplete", Light, Edit, wiki_autocomplete),
        shot("dark-vim-normal", Dark, Edit, vim_normal),
        shot("light-vim-normal", Light, Edit, vim_normal),
        shot("dark-vim-pending", Dark, Edit, vim_pending),
        shot("light-vim-pending", Light, Edit, vim_pending),
        shot("dark-vim-insert", Dark, Edit, vim_insert),
        shot("light-vim-insert", Light, Edit, vim_insert),
        shot("dark-vim-visual", Dark, Edit, vim_visual),
        shot("light-vim-visual", Light, Edit, vim_visual),
        shot("dark-vim-visual-line", Dark, Edit, vim_visual_line),
        shot("light-vim-visual-line", Light, Edit, vim_visual_line),
        shot("dark-palette-headings", Dark, Edit, palette_headings),
        shot("light-palette-headings", Light, Edit, palette_headings),
        shot("dark-dev-flavour", Dark, Edit, dev_flavour),
        shot("light-dev-flavour", Light, Edit, dev_flavour),
    ];
    // Plugins: Settings › Plugins, the sheets and a panel.
    for (name, prepare, dialog) in plugins_settings::SHOTS
        .iter()
        .chain(plugins_sheets::SHOTS)
        .chain(plugins_panel::SHOTS)
    {
        let theme = if name.starts_with("dark-") {
            Dark
        } else {
            Light
        };
        let mut plugin_shot = shot(name, theme, Edit, *prepare);
        if let Some(width) = dialog {
            plugin_shot.dialog_width = *width;
        }
        // A short confirmation sits near the top, as the account's do, and
        // the developer sheet has its warning where the check usually looks.
        if name.ends_with("-remove") || name.ends_with("-plugin-developer") {
            plugin_shot.dialog_sample_y = Some(30.);
        }
        shots.push(plugin_shot);
    }
    // The Themes page whole, in both of Kanso's themes and in another.
    let tall = size(px(1440.), px(2900.));
    shots.extend([
        Shot {
            size: tall,
            ..shot("dark-settings-themes", Dark, Edit, settings_appearance)
        },
        Shot {
            size: tall,
            ..shot("light-settings-themes", Light, Edit, settings_appearance)
        },
        Shot {
            size: tall,
            ui_theme: Some("Nord"),
            ..shot("nord-settings-themes", Dark, Edit, settings_appearance)
        },
    ]);
    // Every built-in theme: the latest note in Split, and the sampler of
    // Markdown and code. `theme-<file>` and `theme-<file>-code`.
    for entry in crate::themes::builtins() {
        let stem = entry
            .name()
            .to_lowercase()
            .replace(' ', "-")
            .replace('é', "e");
        let leak = |name: String| -> &'static str { Box::leak(name.into_boxed_str()) };
        let name: &'static str = leak(entry.name().to_owned());
        let theme = if entry.dark() { Dark } else { Light };
        shots.push(Shot {
            ui_theme: Some(name),
            ..shot(leak(format!("theme-{stem}")), theme, Split, open_latest)
        });
        shots.push(Shot {
            ui_theme: Some(name),
            ..shot(
                leak(format!("theme-{stem}-code")),
                theme,
                Split,
                open_theme_note,
            )
        });
    }
    // Code blocks in each grammar, in Kanso's themes and in Dracula.
    let code = size(px(1440.), px(2000.));
    shots.extend([
        Shot {
            size: code,
            ..shot("dark-code-languages", Dark, Preview, open_code_note)
        },
        Shot {
            size: code,
            ..shot("light-code-languages", Light, Preview, open_code_note)
        },
        Shot {
            size: code,
            ui_theme: Some("Dracula"),
            ..shot("dracula-code-languages", Dark, Preview, open_code_note)
        },
        Shot {
            size: code,
            ..shot("dark-code-languages-edit", Dark, Edit, open_code_note)
        },
    ]);
    // Themes mixed: one theme's UI, another's syntax and a third's preview,
    // each fitted to the UI's surfaces.
    shots.extend([
        Shot {
            ui_theme: Some("Nord"),
            syntax_theme: Some("Dracula"),
            preview_theme: Some("GitHub Dark"),
            ..shot("mixed-nord-dracula-github", Dark, Split, open_theme_note)
        },
        Shot {
            ui_theme: Some("Solarized Light"),
            syntax_theme: Some("Tokyo Night"),
            preview_theme: Some("Catppuccin Mocha"),
            ..shot("mixed-light-ui-dark-syntax", Light, Split, open_theme_note)
        },
    ]);
    let real_db = std::env::var_os("KANSO_SNAPSHOT_DB");
    let only = std::env::var("KANSO_SNAPSHOT_ONLY").ok();
    for shot in shots.iter().filter(|s| {
        only.as_deref()
            .is_none_or(|o| o.split(',').any(|n| n == s.name))
    }) {
        crate::preview::REVEAL_IMAGE_SIZES.store(false, std::sync::atomic::Ordering::Relaxed);
        *crate::preview::PREVIEW_DRAG_WIDTH.lock().unwrap() = None;
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kanso.db");
        let (store, clock) = match &real_db {
            Some(source) => {
                std::fs::copy(source, &db).unwrap();
                (Store::open(&db).unwrap(), None)
            }
            None => (
                demo_store(&db),
                Some(Clock::fixed(DEMO_NOW_MS, DEMO_UTC_OFFSET_SECS)),
            ),
        };
        let store = Arc::new(store);
        let mut cx = context(clock);
        // Every shot starts at actual size; zoom is per thread.
        cx.update(|cx| crate::zoom::set(crate::zoom::DEFAULT_ZOOM, cx));
        let (window, workspace) = open(&mut cx, store.clone(), dir.path().to_path_buf(), shot);
        cx.run_until_parked();
        (shot.prepare)(&mut cx, window, &workspace, &store);
        if real_db.is_none() {
            // Not under text being typed, which would read as a change
            // made elsewhere.
            cx.run_until_parked();
            let typing = cx.update(|cx| {
                let editor = workspace.read(cx).editor.read(cx);
                editor
                    .has_unsaved_edits(cx)
                    .then(|| editor.note_id().map(str::to_owned))
                    .flatten()
            });
            pin_new_times(&store, typing.as_deref());
        }
        // Search waits for typing to pause.
        cx.advance_clock(std::time::Duration::from_secs(1));
        // Let background work (backlinks, image decoding) settle, then draw.
        // Each round moves the clock on too, so work a draw schedules after
        // a pause, such as Markdown highlighting that ran out of time, is
        // done before the picture is taken.
        for _ in 0..3 {
            cx.advance_clock(std::time::Duration::from_millis(500));
            cx.run_until_parked();
            cx.update_window(window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        }
        if real_db.is_some() {
            // What a reload costs the list on the main thread: grouping the
            // notes by local day and sizing the rows.
            let (rows, took) = cx.update(|cx| {
                let list = workspace.read(cx).note_list.clone();
                list.update(cx, |list, cx| list.time_layout(cx))
            });
            println!("{}: laid out {rows} rows in {took:?}", shot.name);
        }
        let image = cx.capture_screenshot(window).expect("Metal rendering");
        let path = out.join(format!("{}.png", shot.name));
        image.save(&path).unwrap();
        println!(
            "wrote {} ({}x{})",
            path.display(),
            image.width(),
            image.height()
        );
        let palette_open = cx.update(|cx| workspace.read(cx).palette.read(cx).is_open());
        let settings_open = cx.update(|cx| workspace.read(cx).settings_open());
        let graph_open = cx.update(|cx| workspace.read(cx).graph_open);
        // The board is laid out as the graph is: bands in the list's color
        // round the editor's.
        let board_open = cx.update(|cx| workspace.read(cx).board_open);
        let dialog_open = cx
            .update_window(window, |_, window, cx| window.has_active_dialog(cx))
            .unwrap();
        if dialog_open {
            check_dialog(&image, shot, shot.dialog_width);
        } else if palette_open {
            check_palette(&image, shot);
        } else if settings_open {
            check_settings_surfaces(&image, shot);
        } else if graph_open || board_open {
            discovery::check_graph(&image, shot);
        } else {
            let inspector = cx
                .update_window(window, |_, window, cx| {
                    workspace.read(cx).inspector_shown(window).is_some()
                })
                .unwrap();
            check_surfaces(&image, shot, inspector);
        }
    }
}
