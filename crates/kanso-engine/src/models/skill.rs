use sqlx::FromRow;

/// A Markdown-defined, inspectable agent behavior.
#[derive(Debug, Clone, FromRow)]
pub struct Skill {
    pub id: String,
    pub title: String,
    pub body_markdown: String,
    /// `global` | `notebook` | `note` | `project`.
    pub scope: String,
    pub enabled: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A record of one skill execution (dry run, review, or apply).
#[derive(Debug, Clone, FromRow)]
pub struct SkillRun {
    pub id: String,
    pub skill_id: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    /// `dry_run` | `review_changes` | `apply_changes`.
    pub mode: String,
    /// `running` | `completed` | `failed` | `rejected`.
    pub status: String,
    pub output_summary: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}
