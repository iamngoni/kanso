//! Skill commands.
//!
//! Skills are first-party, editable, Markdown-defined behaviors plus a run log.
//! They are local engine config (the actual agent execution happens in the host
//! app / MCP client); the engine owns their definitions and run records.

use kanso_types::{SkillId, SkillRunId};

use crate::db::{Engine, now_ms};
use crate::error::{EngineError, Result};
use crate::models::{Skill, SkillRun};

const SKILL_COLUMNS: &str = "id, title, body_markdown, scope, enabled, created_at, updated_at";
const RUN_COLUMNS: &str =
    "id, skill_id, target_type, target_id, mode, status, output_summary, created_at, completed_at";

impl Engine {
    pub async fn create_skill(
        &self,
        title: &str,
        body_markdown: &str,
        scope: &str,
    ) -> Result<Skill> {
        let id = SkillId::new().0;
        let now = now_ms();
        sqlx::query(
            "INSERT INTO skills (id, title, body_markdown, scope, enabled, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(title)
        .bind(body_markdown)
        .bind(scope)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(Skill {
            id,
            title: title.to_string(),
            body_markdown: body_markdown.to_string(),
            scope: scope.to_string(),
            enabled: 1,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let query = format!("SELECT {SKILL_COLUMNS} FROM skills ORDER BY title");
        Ok(sqlx::query_as::<_, Skill>(&query)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn update_skill(
        &self,
        id: &str,
        title: &str,
        body_markdown: &str,
        scope: &str,
        enabled: bool,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE skills SET title = ?, body_markdown = ?, scope = ?, enabled = ?, updated_at = ? WHERE id = ?",
        )
        .bind(title)
        .bind(body_markdown)
        .bind(scope)
        .bind(enabled as i64)
        .bind(now_ms())
        .bind(id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub async fn delete_skill(&self, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM skill_runs WHERE skill_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let result = sqlx::query("DELETE FROM skills WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Open a run record (status `running`). The host executes the skill, then
    /// calls [`Engine::complete_skill_run`].
    pub async fn start_skill_run(
        &self,
        skill_id: &str,
        target_type: Option<&str>,
        target_id: Option<&str>,
        mode: &str,
    ) -> Result<SkillRun> {
        let id = SkillRunId::new().0;
        let now = now_ms();
        sqlx::query(
            "INSERT INTO skill_runs (id, skill_id, target_type, target_id, mode, status, created_at) \
             VALUES (?, ?, ?, ?, ?, 'running', ?)",
        )
        .bind(&id)
        .bind(skill_id)
        .bind(target_type)
        .bind(target_id)
        .bind(mode)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(SkillRun {
            id,
            skill_id: skill_id.to_string(),
            target_type: target_type.map(str::to_string),
            target_id: target_id.map(str::to_string),
            mode: mode.to_string(),
            status: "running".to_string(),
            output_summary: None,
            created_at: now,
            completed_at: None,
        })
    }

    pub async fn complete_skill_run(
        &self,
        run_id: &str,
        status: &str,
        output_summary: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE skill_runs SET status = ?, output_summary = ?, completed_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(output_summary)
        .bind(now_ms())
        .bind(run_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(run_id.to_string()));
        }
        Ok(())
    }

    pub async fn list_skill_runs(&self, skill_id: &str) -> Result<Vec<SkillRun>> {
        let query = format!(
            "SELECT {RUN_COLUMNS} FROM skill_runs WHERE skill_id = ? ORDER BY created_at DESC"
        );
        Ok(sqlx::query_as::<_, SkillRun>(&query)
            .bind(skill_id)
            .fetch_all(&self.pool)
            .await?)
    }
}
