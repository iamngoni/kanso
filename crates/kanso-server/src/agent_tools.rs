//! An agent's tool calls, without HTTP.
//!
//! `/mcp` answers an agent's `tools/call` by running [`call_with_state`];
//! a model driven by Kanso's own tool loop (an API key, on desktop or on a
//! phone that starts no server) runs [`call`], which is the same function
//! over the same token store. So both paths enforce the same scope, the
//! same [`Visibility`](crate::scope::Visibility), the same limits and stage
//! the same ops.

use std::sync::Arc;

use kanso_store::Store;
use serde_json::Value;

use crate::AppState;
use crate::mcp::{READ_ONLY, suggest, task, tools};
use crate::scope::{AgentScope, SUGGEST_TOOLS, TASK_TOOLS, Visibility};
use crate::tokens::{TokenId, TokenStore};

/// Runs tool `name` for the agent holding `token` (minted in `tokens` for
/// `scope`) and answers as the MCP `tools/call` result would. Never writes
/// notes.
pub fn call(
    store: &Arc<Store>,
    tokens: &TokenStore,
    token: TokenId,
    scope: &Arc<AgentScope>,
    name: &str,
    arguments: Option<&Value>,
) -> Value {
    let state = AppState {
        store: store.clone(),
        local_files: false,
        tokens: tokens.clone(),
    };
    call_with_state(&state, token, scope, name, arguments)
}

/// The tool definitions `scope` is offered, as `tools/list` lists them.
pub fn definitions_for(scope: &AgentScope) -> Vec<Value> {
    crate::mcp::schema::agent_definitions()
        .as_array()
        .into_iter()
        .flatten()
        .filter(|tool| {
            tool.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| scope.allows_tool(name))
        })
        .cloned()
        .collect()
}

pub(crate) fn call_with_state(
    state: &AppState,
    token: TokenId,
    scope: &Arc<AgentScope>,
    name: &str,
    arguments: Option<&Value>,
) -> Value {
    if !scope.allows_tool(name) {
        return tools::error_result(READ_ONLY);
    }
    if !state.tokens.charge(token) {
        return tools::error_result(
            "This question has used as many note lookups as it may. Answer from what you have read.",
        );
    }
    if let Some(limits) = &scope.task_limits
        && !TASK_TOOLS.contains(&name)
        && !SUGGEST_TOOLS.contains(&name)
        && task_reads(state, token, scope).len() >= limits.max_notes
    {
        return tools::error_result(
            "This task has read as many notes as it may. Propose changes from what you have read, then call kanso-task-done.",
        );
    }
    let Some(mut visible) = Visibility::new(&state.store, scope) else {
        return tools::error_result("Couldn't read the notes.");
    };
    // A list or a search shows no more notes than the task has left to read.
    if let Some(limits) = &scope.task_limits {
        visible.limit_reads(task_reads(state, token, scope), limits.max_notes);
    }
    let answer = match scope.task {
        // A task's proposals stage in its change set.
        Some(task_id) if SUGGEST_TOOLS.contains(&name) || TASK_TOOLS.contains(&name) => task::call(
            state.tokens.changesets(),
            task_id,
            &visible,
            name,
            arguments,
        ),
        _ if SUGGEST_TOOLS.contains(&name) => {
            // Staged for review; the store is only read.
            suggest::call(state, token, scope, &visible, name, arguments)
        }
        _ => tools::call(state, Some(&visible), name, arguments),
    };
    state.tokens.record(token, &visible.shown());
    answer
}

/// The notes a task has been shown so far, over all its tokens.
fn task_reads(state: &AppState, token: TokenId, scope: &AgentScope) -> Vec<String> {
    match scope.task {
        Some(task) => state.tokens.task_reads(task),
        None => state.tokens.reads(token),
    }
}
