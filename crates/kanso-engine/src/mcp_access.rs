//! MCP access control — approved clients and their granted capabilities.
//!
//! The engine owns the permission model so every surface (the in-process MCP
//! server, a sidecar, the desktop UI) enforces the same rules. Capabilities are
//! coarse strings: `read`, `write`, `delete`, `run_skill`.

use kanso_types::McpClientId;

use crate::db::{Engine, now_ms};
use crate::error::Result;
use crate::models::McpClient;

impl Engine {
    pub async fn register_mcp_client(&self, name: &str) -> Result<McpClient> {
        let id = McpClientId::new().0;
        let now = now_ms();
        sqlx::query("INSERT INTO mcp_clients (id, name, trusted, created_at) VALUES (?, ?, 0, ?)")
            .bind(&id)
            .bind(name)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(McpClient {
            id,
            name: name.to_string(),
            trusted: 0,
            created_at: now,
        })
    }

    pub async fn list_mcp_clients(&self) -> Result<Vec<McpClient>> {
        Ok(sqlx::query_as::<_, McpClient>(
            "SELECT id, name, trusted, created_at FROM mcp_clients ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn list_mcp_capabilities(&self, client_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT capability FROM mcp_permissions WHERE client_id = ? ORDER BY capability",
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(capability,)| capability).collect())
    }

    pub async fn grant_capability(&self, client_id: &str, capability: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO mcp_permissions (client_id, capability) VALUES (?, ?)")
            .bind(client_id)
            .bind(capability)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_capability(&self, client_id: &str, capability: &str) -> Result<()> {
        sqlx::query("DELETE FROM mcp_permissions WHERE client_id = ? AND capability = ?")
            .bind(client_id)
            .bind(capability)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_mcp_client_trusted(&self, client_id: &str, trusted: bool) -> Result<()> {
        sqlx::query("UPDATE mcp_clients SET trusted = ? WHERE id = ?")
            .bind(trusted as i64)
            .bind(client_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Whether `client_id` may perform `capability`. Unknown clients are denied;
    /// trusted clients are always allowed.
    pub async fn client_can(&self, client_id: &str, capability: &str) -> Result<bool> {
        let client: Option<(i64,)> = sqlx::query_as("SELECT trusted FROM mcp_clients WHERE id = ?")
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await?;
        match client {
            None => Ok(false),
            Some((trusted,)) if trusted != 0 => Ok(true),
            _ => {
                let granted: Option<(i64,)> = sqlx::query_as(
                    "SELECT 1 FROM mcp_permissions WHERE client_id = ? AND capability = ?",
                )
                .bind(client_id)
                .bind(capability)
                .fetch_optional(&self.pool)
                .await?;
                Ok(granted.is_some())
            }
        }
    }
}
