//! First-party MCP server: exposes the Kanso engine to agents over JSON-RPC 2.0
//! (stdio transport). Permissioned tool surface for read/search/create/update.

use kanso_engine::Engine;
use serde_json::{Value, json};

/// An MCP server that owns a [`Engine`] and handles JSON-RPC 2.0 dispatch.
///
/// The server is intentionally stateless beyond the engine handle: every
/// `handle` call is pure given the same database state, making it straightforward
/// to test without a running process.
pub struct McpServer {
    engine: Engine,
    /// When set, every tool call is checked against this client's granted
    /// capabilities. `None` = unrestricted (local/dev / trusted host process).
    client_id: Option<String>,
}

impl McpServer {
    /// Wrap an already-opened engine in an unrestricted MCP server.
    pub fn new(engine: Engine) -> Self {
        Self {
            engine,
            client_id: None,
        }
    }

    /// Wrap an engine for a specific approved client; tool calls are gated by
    /// that client's capabilities (see `Engine::client_can`).
    pub fn with_client(engine: Engine, client_id: impl Into<String>) -> Self {
        Self {
            engine,
            client_id: Some(client_id.into()),
        }
    }

    /// Handle one JSON-RPC request value. Returns `Some(response)` for requests
    /// (objects that carry an `id`) and `None` for notifications (no `id`).
    pub async fn handle(&self, request: Value) -> Option<Value> {
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");

        // Notifications carry no `id` and require no response.
        let id = id?;

        let result: Result<Value, (i64, String)> = match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "kanso-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            })),

            "ping" => Ok(json!({})),

            "tools/list" => Ok(json!({ "tools": tool_definitions() })),

            "tools/call" => self.call_tool(request.get("params")).await,

            other => Err((-32601, format!("method not found: {other}"))),
        };

        Some(match result {
            Ok(value) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": value,
            }),
            Err((code, message)) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": message },
            }),
        })
    }

    /// Validate and route a `tools/call` request to [`Self::run_tool`].
    async fn call_tool(&self, params: Option<&Value>) -> Result<Value, (i64, String)> {
        let params = params.ok_or((-32602, "missing params".to_string()))?;
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or((-32602, "missing tool name".to_string()))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        let text = self.run_tool(name, &args).await.map_err(|e| (-32000, e))?;
        Ok(json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false,
        }))
    }

    /// Execute a named tool, returning a human/agent-readable text result or an
    /// error string that becomes a JSON-RPC `-32000` application error.
    async fn run_tool(&self, name: &str, args: &Value) -> Result<String, String> {
        // Enforce capability when scoped to a client.
        if let Some(client_id) = &self.client_id {
            let capability = capability_for(name);
            if !self
                .engine
                .client_can(client_id, capability)
                .await
                .map_err(|e| e.to_string())?
            {
                return Err(format!(
                    "permission denied: '{name}' requires the '{capability}' capability"
                ));
            }
        }

        // Helper: extract a string field from the arguments, defaulting to "".
        let s = |k: &str| {
            args.get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };

        match name {
            "list_notebooks" => {
                let nbs = self
                    .engine
                    .list_notebooks()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(nbs
                    .into_iter()
                    .map(|n| format!("{}\t{}", n.id, n.name))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "create_notebook" => {
                let nb = self
                    .engine
                    .create_notebook(&s("name"), None)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("created notebook {} ({})", nb.name, nb.id))
            }

            "list_notes" => {
                let notes = self
                    .engine
                    .list_notes(&s("notebook_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(notes
                    .into_iter()
                    .map(|n| format!("{}\t{}", n.id, n.title))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "search_notes" => {
                let notes = self
                    .engine
                    .search_notes(&s("query"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(notes
                    .into_iter()
                    .map(|n| format!("{}\t{}", n.id, n.title))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "create_note" => {
                let note = self
                    .engine
                    .create_note(&s("notebook_id"), &s("title"), &s("body_markdown"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("created note {} ({})", note.title, note.id))
            }

            "get_note" => {
                match self
                    .engine
                    .get_note(&s("note_id"))
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(n) => Ok(format!("# {}\n\n{}", n.title, n.body_markdown)),
                    None => Ok("note not found".to_string()),
                }
            }

            "update_note" => {
                self.engine
                    .update_note_body(&s("note_id"), &s("body_markdown"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("updated".to_string())
            }

            "create_tag" => {
                let tag = self
                    .engine
                    .create_tag(&s("name"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("created tag {} ({})", tag.name, tag.id))
            }

            "list_tags" => {
                let tags = self.engine.list_tags().await.map_err(|e| e.to_string())?;
                Ok(tags
                    .into_iter()
                    .map(|t| format!("{}\t{}", t.id, t.name))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "list_tasks" => {
                let tasks = self
                    .engine
                    .list_tasks(&s("notebook_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(tasks
                    .into_iter()
                    .map(|t| format!("[{}] {}", if t.checked != 0 { "x" } else { " " }, t.text))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "backlinks" => {
                let notes = self
                    .engine
                    .backlinks(&s("note_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(notes
                    .into_iter()
                    .map(|n| format!("{}\t{}", n.id, n.title))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "create_daily_note" => {
                let note = self
                    .engine
                    .create_daily_note(&s("notebook_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("daily note {} ({})", note.title, note.id))
            }

            "delete_note" => {
                self.engine
                    .delete_note(&s("note_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("deleted".to_string())
            }

            "tag_note" => {
                self.engine
                    .tag_note(&s("note_id"), &s("tag_id"))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("tagged".to_string())
            }

            "list_skills" => {
                let skills = self.engine.list_skills().await.map_err(|e| e.to_string())?;
                Ok(skills
                    .into_iter()
                    .map(|sk| {
                        format!(
                            "{}\t{}\t{}",
                            sk.id,
                            sk.title,
                            if sk.enabled != 0 {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            "create_skill" => {
                let scope = {
                    let v = s("scope");
                    if v.is_empty() {
                        "global".to_string()
                    } else {
                        v
                    }
                };
                let skill = self
                    .engine
                    .create_skill(&s("title"), &s("body_markdown"), &scope)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("created skill {} ({})", skill.title, skill.id))
            }

            "run_skill" => {
                let mode = {
                    let v = s("mode");
                    if v.is_empty() {
                        "dry_run".to_string()
                    } else {
                        v
                    }
                };
                let run = self
                    .engine
                    .start_skill_run(&s("skill_id"), None, None, &mode)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("started skill run {} (mode {})", run.id, run.mode))
            }

            other => Err(format!("unknown tool: {other}")),
        }
    }
}

/// The capability a tool requires: `read`, `write`, or `delete`.
fn capability_for(tool: &str) -> &'static str {
    match tool {
        "delete_note" => "delete",
        "run_skill" => "run_skill",
        "create_notebook" | "create_note" | "update_note" | "create_tag" | "tag_note"
        | "create_daily_note" | "create_skill" => "write",
        // reads: list_notebooks, list_notes, search_notes, get_note, list_tags,
        // list_tasks, backlinks, list_skills
        _ => "read",
    }
}

/// JSON-Schema tool definitions advertised via `tools/list`.
///
/// Each entry follows the MCP specification shape:
/// `{ name, description, inputSchema: { type: "object", properties, required? } }`.
fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_notebooks",
            "description": "List all notebooks.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "create_notebook",
            "description": "Create a notebook.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "list_notes",
            "description": "List notes in a notebook.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "notebook_id": { "type": "string" }
                },
                "required": ["notebook_id"]
            }
        },
        {
            "name": "search_notes",
            "description": "Full-text search notes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "create_note",
            "description": "Create a note in a notebook.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "notebook_id": { "type": "string" },
                    "title": { "type": "string" },
                    "body_markdown": { "type": "string" }
                },
                "required": ["notebook_id", "title"]
            }
        },
        {
            "name": "get_note",
            "description": "Read a note by id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "note_id": { "type": "string" }
                },
                "required": ["note_id"]
            }
        },
        {
            "name": "update_note",
            "description": "Replace a note's body.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "note_id": { "type": "string" },
                    "body_markdown": { "type": "string" }
                },
                "required": ["note_id", "body_markdown"]
            }
        },
        {
            "name": "create_tag",
            "description": "Create a tag.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "list_tags",
            "description": "List all tags.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "list_tasks",
            "description": "List all tasks in a notebook.",
            "inputSchema": {
                "type": "object",
                "properties": { "notebook_id": { "type": "string" } },
                "required": ["notebook_id"]
            }
        },
        {
            "name": "backlinks",
            "description": "List notes that link to the given note.",
            "inputSchema": {
                "type": "object",
                "properties": { "note_id": { "type": "string" } },
                "required": ["note_id"]
            }
        },
        {
            "name": "create_daily_note",
            "description": "Get or create today's daily note in a notebook.",
            "inputSchema": {
                "type": "object",
                "properties": { "notebook_id": { "type": "string" } },
                "required": ["notebook_id"]
            }
        },
        {
            "name": "delete_note",
            "description": "Soft-delete a note.",
            "inputSchema": {
                "type": "object",
                "properties": { "note_id": { "type": "string" } },
                "required": ["note_id"]
            }
        },
        {
            "name": "tag_note",
            "description": "Attach a tag to a note.",
            "inputSchema": {
                "type": "object",
                "properties": { "note_id": { "type": "string" }, "tag_id": { "type": "string" } },
                "required": ["note_id", "tag_id"]
            }
        },
        {
            "name": "list_skills",
            "description": "List all skills.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "create_skill",
            "description": "Create a Markdown-defined skill.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "body_markdown": { "type": "string" },
                    "scope": { "type": "string", "description": "global | notebook | note | project" }
                },
                "required": ["title", "body_markdown"]
            }
        },
        {
            "name": "run_skill",
            "description": "Start a skill run (records a run; default mode dry_run).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string" },
                    "mode": { "type": "string", "description": "dry_run | review_changes | apply_changes" }
                },
                "required": ["skill_id"]
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_engine::Engine;

    async fn server() -> McpServer {
        McpServer::new(Engine::open_in_memory().await.unwrap())
    }

    /// `initialize` returns the server name in `result.serverInfo.name`.
    #[tokio::test]
    async fn initialize_returns_server_info() {
        let srv = server().await;
        let resp = srv
            .handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }))
            .await
            .expect("initialize should return a response");

        assert_eq!(resp["result"]["serverInfo"]["name"], "kanso-mcp");
    }

    /// `tools/list` returns a non-empty array that includes `create_note`.
    #[tokio::test]
    async fn tools_list_includes_create_note() {
        let srv = server().await;
        let resp = srv
            .handle(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
            .await
            .expect("tools/list should return a response");

        let tools = resp["result"]["tools"]
            .as_array()
            .expect("tools must be an array");
        assert!(!tools.is_empty(), "tools array must not be empty");

        let has_create_note = tools.iter().any(|t| t["name"] == "create_note");
        assert!(has_create_note, "tools list must contain create_note");
    }

    /// A notification (object with `method` but no `id`) causes `handle` to return `None`.
    #[tokio::test]
    async fn notification_returns_none() {
        let srv = server().await;
        let result = srv
            .handle(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .await;

        assert!(
            result.is_none(),
            "notifications must not produce a response"
        );
    }

    /// `create_notebook` followed by `list_notebooks` reflects the new entry.
    #[tokio::test]
    async fn create_then_list_notebooks() {
        let srv = server().await;

        // Create the notebook.
        let create_resp = srv
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "create_notebook",
                    "arguments": { "name": "Research" }
                }
            }))
            .await
            .expect("create_notebook should return a response");

        assert_eq!(
            create_resp["result"]["isError"], false,
            "create_notebook must not error"
        );

        // List notebooks and confirm "Research" appears in the text content.
        let list_resp = srv
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "list_notebooks",
                    "arguments": {}
                }
            }))
            .await
            .expect("list_notebooks should return a response");

        let text = list_resp["result"]["content"][0]["text"]
            .as_str()
            .expect("response must carry text content");

        assert!(
            text.contains("Research"),
            "list_notebooks output must contain the notebook name; got: {text}"
        );
    }

    /// An unknown method produces a JSON-RPC error with code -32601.
    #[tokio::test]
    async fn unknown_method_returns_error_32601() {
        let srv = server().await;
        let resp = srv
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "no_such_method"
            }))
            .await
            .expect("should return an error response, not None");

        assert_eq!(
            resp["error"]["code"], -32601,
            "unknown method must yield error code -32601"
        );
    }

    /// A client-scoped server enforces granted capabilities per tool.
    #[tokio::test]
    async fn capability_gating_blocks_ungranted_tools() {
        let engine = Engine::open_in_memory().await.unwrap();
        let client = engine.register_mcp_client("agent").await.unwrap();
        engine.grant_capability(&client.id, "read").await.unwrap();

        let srv = McpServer::with_client(engine, client.id);

        // A read tool is allowed.
        let read = srv
            .handle(json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "list_notebooks", "arguments": {} }
            }))
            .await
            .unwrap();
        assert!(read.get("result").is_some(), "read tool should be allowed");

        // A write tool (not granted) is denied with an application error.
        let write = srv
            .handle(json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": { "name": "create_notebook", "arguments": { "name": "X" } }
            }))
            .await
            .unwrap();
        assert_eq!(write["error"]["code"], -32000, "ungranted write must error");
        assert!(
            write["error"]["message"]
                .as_str()
                .unwrap()
                .contains("permission denied"),
            "error should explain the denial"
        );
    }
}
