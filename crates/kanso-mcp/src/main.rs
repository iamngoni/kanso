//! Stdio transport entry point for the Kanso MCP server.
//!
//! Reads newline-delimited JSON-RPC 2.0 requests from stdin, dispatches each
//! through [`kanso_mcp::McpServer`], and writes responses (plus a newline) to
//! stdout. The database path is taken from the `KANSO_DB` environment variable,
//! defaulting to `kanso.db` in the current directory.

use kanso_engine::Engine;
use kanso_mcp::McpServer;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = std::env::var("KANSO_DB").unwrap_or_else(|_| "kanso.db".to_string());
    let engine = Engine::open(&db).await.expect("open engine");
    let server = McpServer::new(engine);

    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut out = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if let Some(response) = server.handle(request).await {
            let bytes = serde_json::to_vec(&response).unwrap();
            out.write_all(&bytes).await?;
            out.write_all(b"\n").await?;
            out.flush().await?;
        }
    }

    Ok(())
}
