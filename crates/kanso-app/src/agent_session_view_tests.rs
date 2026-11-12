//! The session view in a headless window: what a transcript draws as, in
//! order, with runs of tool calls folded. The runner is not involved.

use std::sync::Arc;

use gpui_kit::{AppContext as _, TestAppContext, VisualTestContext};
use kanso_engine::agent_task::session::{Item, PermissionAnswer, RoundState, ToolKind, ToolStatus};
use kanso_server::Counts;
use kanso_store::Store;

use crate::agent_session_view::AgentSessionView;
use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::AppState;

fn tool(id: &str, status: ToolStatus) -> Item {
    Item::Tool {
        id: id.to_owned(),
        title: format!("Read {id}"),
        kind: ToolKind::Read,
        status,
    }
}

fn transcript() -> Vec<Item> {
    let mut items = vec![
        Item::User {
            text: "Tidy my trips".into(),
            at_ms: 0,
        },
        Item::Thought {
            text: "Looking at what is there".into(),
            ms: 4000,
        },
    ];
    items.extend((0..4).map(|n| tool(&n.to_string(), ToolStatus::Completed)));
    items.extend([
        Item::Agent {
            text: "I found **three** notes.".into(),
            at_ms: 0,
        },
        Item::Permission {
            title: "Read a file".into(),
            kind: ToolKind::Read,
            answer: PermissionAnswer::Rejected,
        },
        tool("last", ToolStatus::Failed),
        Item::Proposed {
            round: 1,
            counts: Counts {
                creates: 1,
                edits: 2,
                moves: 0,
                merges: 0,
            },
            state: RoundState::ToReview,
        },
        Item::Notice {
            text: "Stopped".into(),
            at_ms: 0,
        },
    ]);
    items
}

#[gpui_kit::test]
fn a_transcript_draws_in_order_with_tool_runs_folded(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    let (window, _view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |_, cx| {
            cx.new(|cx| AgentSessionView::from_items(&transcript(), true, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    let mut vcx = VisualTestContext::from_window(window, cx);
    // Blocks, by position: you, thought, the four tools folded into one,
    // agent, permission, the failed tool alone, the proposal, the notice.
    let order = [
        "session-you-0",
        "session-thought-1",
        "session-tools-2",
        "session-agent-3",
        "session-permission-4",
        "session-tool-5",
        "session-proposal-6",
        "session-notice-7",
    ];
    let tops: Vec<_> = order
        .into_iter()
        .map(|selector| {
            vcx.debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is drawn"))
                .origin
                .y
        })
        .collect();
    assert!(
        tops.windows(2).all(|pair| pair[0] < pair[1]),
        "drawn top to bottom: {tops:?}"
    );
    // Folded, so no row of its own for the first four tools.
    assert!(vcx.debug_bounds("session-tool-2").is_none());
    assert!(vcx.debug_bounds("session-tool-3").is_none());
    assert!(vcx.debug_bounds("session-header").is_some());
}
