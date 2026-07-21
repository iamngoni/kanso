//! Sharing commands for notes and notebooks.
//!
//! This is the local product model for members. Cloud backends can mirror these
//! rows into ACL/stream membership tables without the app inventing a separate
//! shape.

use kanso_types::payloads::{DeletePayload, ShareMemberPayload};
use kanso_types::sync::{EntityType, Operation};

use crate::db::{Engine, enqueue_outbox, insert_tombstone, now_ms};
use crate::error::{EngineError, Result};
use crate::models::{Share, ShareMember};

const MEMBER_COLUMNS: &str = "\
    m.id, m.share_id, s.resource_type, s.resource_id, \
    m.email, m.role, m.status, m.created_at, m.updated_at";

impl Engine {
    pub async fn list_share_members(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<ShareMember>> {
        validate_resource_type(resource_type)?;
        self.ensure_share_resource_exists(resource_type, resource_id)
            .await?;

        let query = format!(
            "SELECT {MEMBER_COLUMNS} \
             FROM share_members m \
             JOIN shares s ON s.id = m.share_id \
             WHERE s.resource_type = ? AND s.resource_id = ? \
             ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'editor' THEN 1 ELSE 2 END, m.email"
        );
        Ok(sqlx::query_as::<_, ShareMember>(&query)
            .bind(resource_type)
            .bind(resource_id)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn add_share_member(
        &self,
        resource_type: &str,
        resource_id: &str,
        email: &str,
        role: &str,
    ) -> Result<ShareMember> {
        validate_resource_type(resource_type)?;
        validate_role(role)?;
        let email = normalize_email(email)?;
        self.ensure_share_resource_exists(resource_type, resource_id)
            .await?;

        let now = now_ms();
        let mut tx = self.pool.begin().await?;
        let share = ensure_share(&mut tx, resource_type, resource_id, now).await?;
        let member_id = format!("sharemember:{}", uuid::Uuid::now_v7());

        sqlx::query(
            "INSERT INTO share_members (id, share_id, email, role, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 'invited', ?, ?) \
             ON CONFLICT (share_id, email) DO UPDATE SET \
                role = excluded.role, \
                status = 'invited', \
                updated_at = excluded.updated_at",
        )
        .bind(&member_id)
        .bind(&share.id)
        .bind(&email)
        .bind(role)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let query = format!(
            "SELECT {MEMBER_COLUMNS} \
             FROM share_members m \
             JOIN shares s ON s.id = m.share_id \
             WHERE s.id = ? AND m.email = ?"
        );
        let member = sqlx::query_as::<_, ShareMember>(&query)
            .bind(&share.id)
            .bind(&email)
            .fetch_one(&mut *tx)
            .await?;
        enqueue_outbox(
            &mut tx,
            EntityType::ShareMember,
            &member.id,
            Operation::ShareMemberAdded,
            serde_json::to_value(ShareMemberPayload {
                resource_type: member.resource_type.clone(),
                resource_id: member.resource_id.clone(),
                email: member.email.clone(),
                role: member.role.clone(),
                status: member.status.clone(),
                created_at: member.created_at,
                updated_at: member.updated_at,
            })?,
            now,
        )
        .await?;
        tx.commit().await?;
        Ok(member)
    }

    pub async fn remove_share_member(&self, member_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;
        let exists: Option<(i64,)> =
            sqlx::query_as("SELECT updated_at FROM share_members WHERE id = ?")
                .bind(member_id)
                .fetch_optional(&mut *tx)
                .await?;
        if exists.is_none() {
            return Err(EngineError::NotFound(member_id.to_string()));
        }

        sqlx::query("DELETE FROM share_members WHERE id = ?")
            .bind(member_id)
            .execute(&mut *tx)
            .await?;
        insert_tombstone(&mut tx, EntityType::ShareMember, member_id, now).await?;
        enqueue_outbox(
            &mut tx,
            EntityType::ShareMember,
            member_id,
            Operation::ShareMemberRemoved,
            serde_json::to_value(DeletePayload { deleted_at: now })?,
            now,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn ensure_share_resource_exists(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<()> {
        let exists: Option<(i64,)> = match resource_type {
            "note" => {
                sqlx::query_as("SELECT 1 FROM notes WHERE id = ? AND deleted_at IS NULL")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            "notebook" => {
                sqlx::query_as("SELECT 1 FROM notebooks WHERE id = ? AND deleted_at IS NULL")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            _ => return Err(invalid_resource_type(resource_type)),
        };
        exists
            .map(|_| ())
            .ok_or_else(|| EngineError::NotFound(resource_id.to_string()))
    }
}

async fn ensure_share(
    conn: &mut sqlx::SqliteConnection,
    resource_type: &str,
    resource_id: &str,
    now: i64,
) -> Result<Share> {
    if let Some(share) = sqlx::query_as::<_, Share>(
        "SELECT id, resource_type, resource_id, created_at, updated_at \
         FROM shares WHERE resource_type = ? AND resource_id = ?",
    )
    .bind(resource_type)
    .bind(resource_id)
    .fetch_optional(&mut *conn)
    .await?
    {
        return Ok(share);
    }

    let id = format!("share:{}", uuid::Uuid::now_v7());
    sqlx::query(
        "INSERT INTO shares (id, resource_type, resource_id, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(resource_type)
    .bind(resource_id)
    .bind(now)
    .bind(now)
    .execute(&mut *conn)
    .await?;

    Ok(Share {
        id,
        resource_type: resource_type.to_string(),
        resource_id: resource_id.to_string(),
        created_at: now,
        updated_at: now,
    })
}

fn validate_resource_type(resource_type: &str) -> Result<()> {
    match resource_type {
        "note" | "notebook" => Ok(()),
        _ => Err(invalid_resource_type(resource_type)),
    }
}

fn invalid_resource_type(resource_type: &str) -> EngineError {
    EngineError::Conflict(format!(
        "share resource type must be note or notebook, got {resource_type}"
    ))
}

fn validate_role(role: &str) -> Result<()> {
    match role {
        "owner" | "editor" | "viewer" => Ok(()),
        _ => Err(EngineError::Conflict(format!(
            "share role must be owner, editor, or viewer, got {role}"
        ))),
    }
}

fn normalize_email(email: &str) -> Result<String> {
    let normalized = email.trim().to_ascii_lowercase();
    if normalized.contains('@') && normalized.len() >= 3 {
        Ok(normalized)
    } else {
        Err(EngineError::Conflict(format!(
            "share member email is invalid: {email}"
        )))
    }
}
