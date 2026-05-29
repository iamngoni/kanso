use sqlx::FromRow;

/// An approved MCP client (an AI agent / app granted scoped access).
#[derive(Debug, Clone, FromRow)]
pub struct McpClient {
    pub id: String,
    pub name: String,
    /// Non-zero = trusted; bypasses per-capability checks.
    pub trusted: i64,
    pub created_at: i64,
}
