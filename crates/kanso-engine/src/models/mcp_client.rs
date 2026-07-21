/// An approved MCP client (an AI agent / app granted scoped access).
#[derive(Debug, Clone)]
pub struct McpClient {
    pub id: String,
    pub name: String,
    /// Non-zero = trusted; bypasses per-capability checks.
    pub trusted: i64,
    pub created_at: i64,
}

impl_sqlite_from_row!(McpClient {
    id,
    name,
    trusted,
    created_at,
});
