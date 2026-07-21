//! UniFFI bindings: a synchronous facade over the async engine.
//!
//! Native apps (Swift/Kotlin) call these blocking methods; we drive the async
//! engine on an owned Tokio runtime via `block_on`. This matches the
//! architecture note about keeping the command API synchronous under UniFFI.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kanso_engine::Engine;

uniffi::setup_scaffolding!();

// ── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum KansoError {
    #[error("{message}")]
    Engine { message: String },
}

impl From<kanso_engine::EngineError> for KansoError {
    fn from(e: kanso_engine::EngineError) -> Self {
        KansoError::Engine {
            message: e.to_string(),
        }
    }
}

impl From<String> for KansoError {
    fn from(message: String) -> Self {
        KansoError::Engine { message }
    }
}

// ── Transfer records (DTO layer) ─────────────────────────────────────────────

/// Flat representation of a notebook, safe to cross the FFI boundary.
#[derive(uniffi::Record)]
pub struct NotebookDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

/// Flat representation of a note.
///
/// `pinned` and `favorite` are booleans; the engine stores them as `i64`
/// (SQLite has no boolean type) so we convert on the way out.
#[derive(uniffi::Record)]
pub struct NoteDto {
    pub id: String,
    pub notebook_id: String,
    pub title: String,
    pub body_markdown: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub favorite: bool,
    pub status: String,
}

/// Flat representation of a tag.
#[derive(uniffi::Record)]
pub struct TagDto {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(uniffi::Record)]
pub struct AttachmentDto {
    pub id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub local_path: Option<String>,
    pub remote_key: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(uniffi::Record)]
pub struct NewAttachmentDto {
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub local_path: Option<String>,
}

#[derive(uniffi::Record)]
pub struct TaskItemDto {
    pub id: String,
    pub note_id: String,
    pub text: String,
    pub checked: bool,
}

#[derive(uniffi::Record)]
pub struct NoteLinkDto {
    pub source_note_id: String,
    pub target_ref: String,
    pub link_kind: String,
}

#[derive(uniffi::Record)]
pub struct RevisionDto {
    pub id: String,
    pub note_id: String,
    pub body_markdown: String,
    pub reason: Option<String>,
    pub source: String,
    pub created_at: i64,
}

#[derive(uniffi::Record)]
pub struct ExportFileDto {
    pub path: String,
    pub content: String,
}

#[derive(uniffi::Record)]
pub struct ImportFileDto {
    pub filename: String,
    pub content: String,
}

#[derive(uniffi::Record)]
pub struct AuthSessionDto {
    pub token: String,
    pub user_id: String,
    pub device_id: String,
}

#[derive(uniffi::Record)]
pub struct SyncReportDto {
    pub pushed: u32,
    pub applied: u32,
    pub conflicted: u32,
    pub deleted: u32,
    pub skipped: u32,
    pub uploaded_blobs: u32,
    pub downloaded_blobs: u32,
}

#[derive(uniffi::Record)]
pub struct McpClientDto {
    pub id: String,
    pub name: String,
    pub trusted: bool,
    pub created_at: i64,
}

#[derive(uniffi::Record)]
pub struct SkillDto {
    pub id: String,
    pub title: String,
    pub body_markdown: String,
    pub scope: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(uniffi::Record)]
pub struct SkillRunDto {
    pub id: String,
    pub skill_id: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub mode: String,
    pub status: String,
    pub output_summary: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(uniffi::Record)]
pub struct ShareMemberDto {
    pub id: String,
    pub share_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

// ── From conversions ─────────────────────────────────────────────────────────

impl From<kanso_engine::Notebook> for NotebookDto {
    fn from(nb: kanso_engine::Notebook) -> Self {
        NotebookDto {
            id: nb.id,
            name: nb.name,
            parent_id: nb.parent_id,
        }
    }
}

impl From<kanso_engine::Note> for NoteDto {
    fn from(n: kanso_engine::Note) -> Self {
        NoteDto {
            id: n.id,
            notebook_id: n.notebook_id,
            title: n.title,
            body_markdown: n.body_markdown,
            created_at: n.created_at,
            updated_at: n.updated_at,
            pinned: n.pinned != 0,
            favorite: n.favorite != 0,
            status: n.status,
        }
    }
}

impl From<kanso_engine::Tag> for TagDto {
    fn from(t: kanso_engine::Tag) -> Self {
        TagDto {
            id: t.id,
            name: t.name,
            color: t.color,
        }
    }
}

impl From<kanso_engine::Attachment> for AttachmentDto {
    fn from(attachment: kanso_engine::Attachment) -> Self {
        Self {
            id: attachment.id,
            note_id: attachment.note_id,
            filename: attachment.filename,
            mime_type: attachment.mime_type,
            size_bytes: attachment.size_bytes,
            content_hash: attachment.content_hash,
            local_path: attachment.local_path,
            remote_key: attachment.remote_key,
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
}

impl From<kanso_engine::TaskItem> for TaskItemDto {
    fn from(task: kanso_engine::TaskItem) -> Self {
        Self {
            id: task.id,
            note_id: task.note_id,
            text: task.text,
            checked: task.checked != 0,
        }
    }
}

impl From<kanso_engine::NoteLink> for NoteLinkDto {
    fn from(link: kanso_engine::NoteLink) -> Self {
        Self {
            source_note_id: link.source_note_id,
            target_ref: link.target_ref,
            link_kind: link.link_kind,
        }
    }
}

impl From<kanso_engine::Revision> for RevisionDto {
    fn from(revision: kanso_engine::Revision) -> Self {
        Self {
            id: revision.id,
            note_id: revision.note_id,
            body_markdown: revision.body_markdown,
            reason: revision.reason,
            source: revision.source,
            created_at: revision.created_at,
        }
    }
}

impl From<kanso_engine::ExportFile> for ExportFileDto {
    fn from(file: kanso_engine::ExportFile) -> Self {
        Self {
            path: file.path,
            content: file.content,
        }
    }
}

impl From<kanso_types::AuthResponse> for AuthSessionDto {
    fn from(auth: kanso_types::AuthResponse) -> Self {
        Self {
            token: auth.token,
            user_id: auth.user_id,
            device_id: auth.device_id,
        }
    }
}

impl From<kanso_engine::SyncReport> for SyncReportDto {
    fn from(report: kanso_engine::SyncReport) -> Self {
        Self {
            pushed: report.pushed as u32,
            applied: report.applied as u32,
            conflicted: report.conflicted as u32,
            deleted: report.deleted as u32,
            skipped: report.skipped as u32,
            uploaded_blobs: 0,
            downloaded_blobs: 0,
        }
    }
}

impl From<kanso_engine::McpClient> for McpClientDto {
    fn from(client: kanso_engine::McpClient) -> Self {
        Self {
            id: client.id,
            name: client.name,
            trusted: client.trusted != 0,
            created_at: client.created_at,
        }
    }
}

impl From<kanso_engine::Skill> for SkillDto {
    fn from(skill: kanso_engine::Skill) -> Self {
        Self {
            id: skill.id,
            title: skill.title,
            body_markdown: skill.body_markdown,
            scope: skill.scope,
            enabled: skill.enabled != 0,
            created_at: skill.created_at,
            updated_at: skill.updated_at,
        }
    }
}

impl From<kanso_engine::SkillRun> for SkillRunDto {
    fn from(run: kanso_engine::SkillRun) -> Self {
        Self {
            id: run.id,
            skill_id: run.skill_id,
            target_type: run.target_type,
            target_id: run.target_id,
            mode: run.mode,
            status: run.status,
            output_summary: run.output_summary,
            created_at: run.created_at,
            completed_at: run.completed_at,
        }
    }
}

impl From<kanso_engine::ShareMember> for ShareMemberDto {
    fn from(member: kanso_engine::ShareMember) -> Self {
        Self {
            id: member.id,
            share_id: member.share_id,
            resource_type: member.resource_type,
            resource_id: member.resource_id,
            email: member.email,
            role: member.role,
            status: member.status,
            created_at: member.created_at,
            updated_at: member.updated_at,
        }
    }
}

// ── Ink / sketch records ──────────────────────────────────────────────────────

/// One captured stylus sample. The native layer (PencilKit-free; raw
/// `UITouch`/`NSEvent`/`MotionEvent`) normalizes its input into these.
#[derive(uniffi::Record)]
pub struct InkPoint {
    pub x: f32,
    pub y: f32,
    /// 0.0–1.0 pen pressure; pass 1.0 for mouse/finger without force.
    pub pressure: f32,
}

#[derive(uniffi::Record)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// One captured stroke: a point list plus styling.
#[derive(uniffi::Record)]
pub struct InkStroke {
    pub points: Vec<InkPoint>,
    pub color: ColorRgba,
    pub width: f32,
}

/// Flat representation of a sketch.
#[derive(uniffi::Record)]
pub struct SketchDto {
    pub id: String,
    pub note_id: String,
    pub title: Option<String>,
}

impl From<kanso_engine::Sketch> for SketchDto {
    fn from(s: kanso_engine::Sketch) -> Self {
        SketchDto {
            id: s.id,
            note_id: s.note_id,
            title: s.title,
        }
    }
}

/// Build a canonical `kanso_ink::SketchDoc` from captured strokes.
fn build_sketch_doc(strokes: &[InkStroke]) -> kanso_ink::SketchDoc {
    use kanso_ink::{Background, Element, Point, Rgba, SketchDoc, Stroke, Tool};

    let mut doc = SketchDoc::new();
    doc.background = Background::Blank;
    for stroke in strokes {
        doc.elements.push(Element::Stroke(Stroke {
            points: stroke
                .points
                .iter()
                .map(|p| Point {
                    x: p.x,
                    y: p.y,
                    pressure: p.pressure,
                    tilt: 0.0,
                    t: 0.0,
                })
                .collect(),
            color: Rgba {
                r: stroke.color.r,
                g: stroke.color.g,
                b: stroke.color.b,
                a: stroke.color.a,
            },
            base_width: stroke.width,
            tool: Tool::Pen,
        }));
    }
    doc
}

fn extract_ink_strokes(doc: &kanso_ink::SketchDoc) -> Vec<InkStroke> {
    doc.elements
        .iter()
        .filter_map(|element| {
            let kanso_ink::Element::Stroke(stroke) = element else {
                return None;
            };
            Some(InkStroke {
                points: stroke
                    .points
                    .iter()
                    .map(|point| InkPoint {
                        x: point.x,
                        y: point.y,
                        pressure: point.pressure,
                    })
                    .collect(),
                color: ColorRgba {
                    r: stroke.color.r,
                    g: stroke.color.g,
                    b: stroke.color.b,
                    a: stroke.color.a,
                },
                width: stroke.base_width,
            })
        })
        .collect()
}

// ── KansoEngine object ────────────────────────────────────────────────────────

/// The primary FFI object.  Wraps an owned Tokio runtime so every method can
/// block the calling thread while the async engine runs.
///
/// Swift/Kotlin hold an `Arc<KansoEngine>`; UniFFI handles the reference count
/// on both sides.
#[derive(uniffi::Object)]
pub struct KansoEngine {
    rt: tokio::runtime::Runtime,
    inner: Engine,
}

async fn upload_local_blobs(
    engine: &Engine,
    base_url: &str,
    token: &str,
) -> Result<u32, KansoError> {
    let mut uploaded = 0;
    for attachment in engine.list_all_attachments().await? {
        let Some(path) = attachment.local_path.as_deref() else {
            continue;
        };
        let path = Path::new(path);
        if !path.is_file() {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let uploaded_hash = kanso_client::put_blob(base_url, token, &bytes).await?;
        if uploaded_hash != attachment.content_hash {
            return Err(KansoError::Engine {
                message: format!(
                    "attachment content hash mismatch for {}",
                    attachment.filename
                ),
            });
        }
        uploaded += 1;
    }
    Ok(uploaded)
}

async fn download_missing_blobs(
    engine: &Engine,
    base_url: &str,
    token: &str,
    attachment_dir: &str,
) -> Result<u32, KansoError> {
    let root = PathBuf::from(attachment_dir);
    fs::create_dir_all(&root).map_err(io_error)?;

    let mut downloaded = 0;
    for attachment in engine.list_all_attachments().await? {
        if attachment
            .local_path
            .as_deref()
            .map(|path| Path::new(path).is_file())
            .unwrap_or(false)
        {
            continue;
        }

        let Some(bytes) = kanso_client::get_blob(base_url, token, &attachment.content_hash).await?
        else {
            continue;
        };
        let dir = root.join(&attachment.content_hash);
        fs::create_dir_all(&dir).map_err(io_error)?;
        let path = dir.join(safe_filename(&attachment.filename));
        fs::write(&path, bytes).map_err(io_error)?;
        engine
            .set_attachment_local_path(&attachment.id, &path.to_string_lossy())
            .await?;
        downloaded += 1;
    }
    Ok(downloaded)
}

fn safe_filename(filename: &str) -> String {
    let cleaned: String = filename
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '\0' => '_',
            _ => ch,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "attachment".to_string()
    } else {
        trimmed.to_string()
    }
}

fn io_error(error: std::io::Error) -> KansoError {
    KansoError::Engine {
        message: error.to_string(),
    }
}

fn derive_encryption_key(passphrase: &str, salt: &str) -> Result<[u8; 32], KansoError> {
    let trimmed = passphrase.trim();
    if trimmed.len() < 8 {
        return Err(KansoError::Engine {
            message: "backup encryption key must be at least 8 characters".to_string(),
        });
    }
    if salt.as_bytes().len() < 8 {
        return Err(KansoError::Engine {
            message: "backup encryption salt must be at least 8 bytes".to_string(),
        });
    }

    let key =
        kanso_crypto::derive_key(trimmed, salt.as_bytes()).map_err(|error| KansoError::Engine {
            message: error.to_string(),
        })?;
    Ok(*key)
}

#[uniffi::export]
impl KansoEngine {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Open (or create) a persistent database at `path`.
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, KansoError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| KansoError::Engine {
            message: e.to_string(),
        })?;
        let inner = rt.block_on(Engine::open(&path))?;
        Ok(Arc::new(Self { rt, inner }))
    }

    /// Open a persistent database with client-side backup encryption enabled.
    ///
    /// Local SQLite remains plaintext for search/indexing. Outbound sync
    /// payloads encrypt note bodies and sketch blobs before they leave device.
    #[uniffi::constructor]
    pub fn open_with_encryption_passphrase(
        path: String,
        passphrase: String,
        salt: String,
    ) -> Result<Arc<Self>, KansoError> {
        let key = derive_encryption_key(&passphrase, &salt)?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| KansoError::Engine {
            message: e.to_string(),
        })?;
        let inner = rt.block_on(Engine::open(&path))?.with_encryption_key(key);
        Ok(Arc::new(Self { rt, inner }))
    }

    /// Open a transient in-memory database (useful for tests / previews).
    #[uniffi::constructor]
    pub fn open_in_memory() -> Result<Arc<Self>, KansoError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| KansoError::Engine {
            message: e.to_string(),
        })?;
        let inner = rt.block_on(Engine::open_in_memory())?;
        Ok(Arc::new(Self { rt, inner }))
    }

    /// Open an encrypted transient in-memory database for tests/previews.
    #[uniffi::constructor]
    pub fn open_in_memory_with_encryption_passphrase(
        passphrase: String,
        salt: String,
    ) -> Result<Arc<Self>, KansoError> {
        let key = derive_encryption_key(&passphrase, &salt)?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| KansoError::Engine {
            message: e.to_string(),
        })?;
        let inner = rt
            .block_on(Engine::open_in_memory())?
            .with_encryption_key(key);
        Ok(Arc::new(Self { rt, inner }))
    }

    // ── Notebooks ────────────────────────────────────────────────────────────

    pub fn create_notebook(
        &self,
        name: String,
        parent_id: Option<String>,
    ) -> Result<NotebookDto, KansoError> {
        let nb = self
            .rt
            .block_on(self.inner.create_notebook(&name, parent_id.as_deref()))?;
        Ok(nb.into())
    }

    pub fn list_notebooks(&self) -> Result<Vec<NotebookDto>, KansoError> {
        let notebooks = self.rt.block_on(self.inner.list_notebooks())?;
        Ok(notebooks.into_iter().map(Into::into).collect())
    }

    pub fn rename_notebook(&self, notebook_id: String, name: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.rename_notebook(&notebook_id, &name))?;
        Ok(())
    }

    pub fn delete_notebook(&self, notebook_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.delete_notebook(&notebook_id))?;
        Ok(())
    }

    pub fn move_notebook(
        &self,
        notebook_id: String,
        parent_id: Option<String>,
    ) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.move_notebook(&notebook_id, parent_id.as_deref()))?;
        Ok(())
    }

    pub fn list_root_notebooks(&self) -> Result<Vec<NotebookDto>, KansoError> {
        let notebooks = self.rt.block_on(self.inner.list_root_notebooks())?;
        Ok(notebooks.into_iter().map(Into::into).collect())
    }

    pub fn list_child_notebooks(&self, parent_id: String) -> Result<Vec<NotebookDto>, KansoError> {
        let notebooks = self
            .rt
            .block_on(self.inner.list_child_notebooks(&parent_id))?;
        Ok(notebooks.into_iter().map(Into::into).collect())
    }

    // ── Notes ────────────────────────────────────────────────────────────────

    pub fn create_note(
        &self,
        notebook_id: String,
        title: String,
        body_markdown: String,
    ) -> Result<NoteDto, KansoError> {
        let note =
            self.rt
                .block_on(self.inner.create_note(&notebook_id, &title, &body_markdown))?;
        Ok(note.into())
    }

    pub fn update_note_body(
        &self,
        note_id: String,
        body_markdown: String,
    ) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.update_note_body(&note_id, &body_markdown))?;
        Ok(())
    }

    pub fn rename_note(&self, note_id: String, title: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.rename_note(&note_id, &title))?;
        Ok(())
    }

    pub fn get_note(&self, note_id: String) -> Result<Option<NoteDto>, KansoError> {
        let note = self.rt.block_on(self.inner.get_note(&note_id))?;
        Ok(note.map(Into::into))
    }

    pub fn list_notes(&self, notebook_id: String) -> Result<Vec<NoteDto>, KansoError> {
        let notes = self.rt.block_on(self.inner.list_notes(&notebook_id))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    pub fn search_notes(&self, query: String) -> Result<Vec<NoteDto>, KansoError> {
        let notes = self.rt.block_on(self.inner.search_notes(&query))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    pub fn render_note_html(&self, note_id: String) -> Result<String, KansoError> {
        let html = self.rt.block_on(self.inner.render_note_html(&note_id))?;
        Ok(html)
    }

    pub fn delete_note(&self, note_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.delete_note(&note_id))?;
        Ok(())
    }

    pub fn restore_note(&self, note_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.restore_note(&note_id))?;
        Ok(())
    }

    pub fn purge_note(&self, note_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.purge_note(&note_id))?;
        Ok(())
    }

    pub fn list_trash(&self) -> Result<Vec<NoteDto>, KansoError> {
        let notes = self.rt.block_on(self.inner.list_trash())?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    pub fn move_note(&self, note_id: String, notebook_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.move_note(&note_id, &notebook_id))?;
        Ok(())
    }

    pub fn create_daily_note(&self, notebook_id: String) -> Result<NoteDto, KansoError> {
        let note = self
            .rt
            .block_on(self.inner.create_daily_note(&notebook_id))?;
        Ok(note.into())
    }

    pub fn set_note_pinned(&self, note_id: String, pinned: bool) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.set_note_pinned(&note_id, pinned))?;
        Ok(())
    }

    pub fn set_note_favorite(&self, note_id: String, favorite: bool) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.set_note_favorite(&note_id, favorite))?;
        Ok(())
    }

    pub fn set_note_status(&self, note_id: String, status: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.set_note_status(&note_id, &status))?;
        Ok(())
    }

    // ── Tags ─────────────────────────────────────────────────────────────────

    pub fn create_tag(&self, name: String) -> Result<TagDto, KansoError> {
        let tag = self.rt.block_on(self.inner.create_tag(&name))?;
        Ok(tag.into())
    }

    pub fn tag_note(&self, note_id: String, tag_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.tag_note(&note_id, &tag_id))?;
        Ok(())
    }

    pub fn untag_note(&self, note_id: String, tag_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.untag_note(&note_id, &tag_id))?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<TagDto>, KansoError> {
        let tags = self.rt.block_on(self.inner.list_tags())?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    pub fn tags_for_note(&self, note_id: String) -> Result<Vec<TagDto>, KansoError> {
        let tags = self.rt.block_on(self.inner.tags_for_note(&note_id))?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    pub fn notes_with_tag(&self, tag_id: String) -> Result<Vec<NoteDto>, KansoError> {
        let notes = self.rt.block_on(self.inner.notes_with_tag(&tag_id))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    // ── Attachments ─────────────────────────────────────────────────────────

    pub fn attach_file(
        &self,
        note_id: String,
        input: NewAttachmentDto,
    ) -> Result<AttachmentDto, KansoError> {
        let attachment = self.rt.block_on(self.inner.attach_file(
            &note_id,
            kanso_engine::NewAttachment {
                filename: input.filename,
                mime_type: input.mime_type,
                size_bytes: input.size_bytes,
                content_hash: input.content_hash,
                local_path: input.local_path,
            },
        ))?;
        Ok(attachment.into())
    }

    pub fn list_attachments(&self, note_id: String) -> Result<Vec<AttachmentDto>, KansoError> {
        let attachments = self.rt.block_on(self.inner.list_attachments(&note_id))?;
        Ok(attachments.into_iter().map(Into::into).collect())
    }

    pub fn delete_attachment(&self, attachment_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.delete_attachment(&attachment_id))?;
        Ok(())
    }

    // ── Derived note graph ──────────────────────────────────────────────────

    pub fn backlinks(&self, note_id: String) -> Result<Vec<NoteDto>, KansoError> {
        let notes = self.rt.block_on(self.inner.backlinks(&note_id))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    pub fn outgoing_links(&self, note_id: String) -> Result<Vec<NoteLinkDto>, KansoError> {
        let links = self.rt.block_on(self.inner.outgoing_links(&note_id))?;
        Ok(links.into_iter().map(Into::into).collect())
    }

    pub fn list_tasks(&self, notebook_id: String) -> Result<Vec<TaskItemDto>, KansoError> {
        let tasks = self.rt.block_on(self.inner.list_tasks(&notebook_id))?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub fn list_open_tasks(&self, notebook_id: String) -> Result<Vec<TaskItemDto>, KansoError> {
        let tasks = self.rt.block_on(self.inner.list_open_tasks(&notebook_id))?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub fn set_task_checked(&self, task_id: String, checked: bool) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.set_task_checked(&task_id, checked))?;
        Ok(())
    }

    // ── Revisions / conflicts ───────────────────────────────────────────────

    pub fn list_revisions(&self, note_id: String) -> Result<Vec<RevisionDto>, KansoError> {
        let revisions = self.rt.block_on(self.inner.list_revisions(&note_id))?;
        Ok(revisions.into_iter().map(Into::into).collect())
    }

    pub fn list_conflicts(&self, note_id: String) -> Result<Vec<RevisionDto>, KansoError> {
        let revisions = self.rt.block_on(self.inner.list_conflicts(&note_id))?;
        Ok(revisions.into_iter().map(Into::into).collect())
    }

    pub fn restore_revision(&self, note_id: String, revision_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.restore_revision(&note_id, &revision_id))?;
        Ok(())
    }

    // ── Markdown import/export ─────────────────────────────────────────────

    pub fn export_notebook_markdown(
        &self,
        notebook_id: String,
    ) -> Result<Vec<ExportFileDto>, KansoError> {
        let files = self
            .rt
            .block_on(self.inner.export_notebook_markdown(&notebook_id))?;
        Ok(files.into_iter().map(Into::into).collect())
    }

    pub fn import_markdown(
        &self,
        notebook_id: String,
        files: Vec<ImportFileDto>,
    ) -> Result<Vec<String>, KansoError> {
        let files = files
            .into_iter()
            .map(|file| kanso_engine::ImportFile {
                filename: file.filename,
                content: file.content,
            })
            .collect();
        Ok(self
            .rt
            .block_on(self.inner.import_markdown(&notebook_id, files))?)
    }

    // ── Sync / auth ─────────────────────────────────────────────────────────

    pub fn register_http(
        &self,
        base_url: String,
        email: String,
        password: String,
    ) -> Result<AuthSessionDto, KansoError> {
        let auth = self
            .rt
            .block_on(kanso_client::register(&base_url, &email, &password))?;
        Ok(auth.into())
    }

    pub fn login_http(
        &self,
        base_url: String,
        email: String,
        password: String,
    ) -> Result<AuthSessionDto, KansoError> {
        let auth = self
            .rt
            .block_on(kanso_client::login(&base_url, &email, &password))?;
        Ok(auth.into())
    }

    pub fn refresh_http(
        &self,
        base_url: String,
        token: String,
    ) -> Result<AuthSessionDto, KansoError> {
        let auth = self.rt.block_on(kanso_client::refresh(&base_url, &token))?;
        Ok(auth.into())
    }

    pub fn sync_http(
        &self,
        base_url: String,
        token: String,
        device_id: String,
    ) -> Result<SyncReportDto, KansoError> {
        let transport = kanso_client::HttpSyncTransport::new(base_url, Some(token));
        let report = self.rt.block_on(self.inner.sync(&device_id, &transport))?;
        Ok(report.into())
    }

    pub fn sync_http_with_blobs(
        &self,
        base_url: String,
        token: String,
        device_id: String,
        attachment_dir: String,
    ) -> Result<SyncReportDto, KansoError> {
        self.rt.block_on(async {
            let uploaded = upload_local_blobs(&self.inner, &base_url, &token).await?;
            let transport =
                kanso_client::HttpSyncTransport::new(base_url.clone(), Some(token.clone()));
            let mut report: SyncReportDto = self.inner.sync(&device_id, &transport).await?.into();
            let downloaded =
                download_missing_blobs(&self.inner, &base_url, &token, &attachment_dir).await?;
            report.uploaded_blobs = uploaded;
            report.downloaded_blobs = downloaded;
            Ok::<_, KansoError>(report)
        })
    }

    // ── MCP access / Skills ───────────────────────────────────────────────────

    pub fn register_mcp_client(&self, name: String) -> Result<McpClientDto, KansoError> {
        let client = self.rt.block_on(self.inner.register_mcp_client(&name))?;
        Ok(client.into())
    }

    pub fn list_mcp_clients(&self) -> Result<Vec<McpClientDto>, KansoError> {
        let clients = self.rt.block_on(self.inner.list_mcp_clients())?;
        Ok(clients.into_iter().map(Into::into).collect())
    }

    pub fn list_mcp_capabilities(&self, client_id: String) -> Result<Vec<String>, KansoError> {
        Ok(self
            .rt
            .block_on(self.inner.list_mcp_capabilities(&client_id))?)
    }

    pub fn grant_mcp_capability(
        &self,
        client_id: String,
        capability: String,
    ) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.grant_capability(&client_id, &capability))?;
        Ok(())
    }

    pub fn revoke_mcp_capability(
        &self,
        client_id: String,
        capability: String,
    ) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.revoke_capability(&client_id, &capability))?;
        Ok(())
    }

    pub fn set_mcp_client_trusted(
        &self,
        client_id: String,
        trusted: bool,
    ) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.set_mcp_client_trusted(&client_id, trusted))?;
        Ok(())
    }

    pub fn create_skill(
        &self,
        title: String,
        body_markdown: String,
        scope: String,
    ) -> Result<SkillDto, KansoError> {
        let skill = self
            .rt
            .block_on(self.inner.create_skill(&title, &body_markdown, &scope))?;
        Ok(skill.into())
    }

    pub fn list_skills(&self) -> Result<Vec<SkillDto>, KansoError> {
        let skills = self.rt.block_on(self.inner.list_skills())?;
        Ok(skills.into_iter().map(Into::into).collect())
    }

    pub fn update_skill(
        &self,
        skill_id: String,
        title: String,
        body_markdown: String,
        scope: String,
        enabled: bool,
    ) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.update_skill(
            &skill_id,
            &title,
            &body_markdown,
            &scope,
            enabled,
        ))?;
        Ok(())
    }

    pub fn delete_skill(&self, skill_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.delete_skill(&skill_id))?;
        Ok(())
    }

    pub fn start_skill_run(
        &self,
        skill_id: String,
        target_type: Option<String>,
        target_id: Option<String>,
        mode: String,
    ) -> Result<SkillRunDto, KansoError> {
        let run = self.rt.block_on(self.inner.start_skill_run(
            &skill_id,
            target_type.as_deref(),
            target_id.as_deref(),
            &mode,
        ))?;
        Ok(run.into())
    }

    pub fn complete_skill_run(
        &self,
        run_id: String,
        status: String,
        output_summary: String,
    ) -> Result<(), KansoError> {
        self.rt.block_on(
            self.inner
                .complete_skill_run(&run_id, &status, &output_summary),
        )?;
        Ok(())
    }

    pub fn list_skill_runs(&self, skill_id: String) -> Result<Vec<SkillRunDto>, KansoError> {
        let runs = self.rt.block_on(self.inner.list_skill_runs(&skill_id))?;
        Ok(runs.into_iter().map(Into::into).collect())
    }

    // ── Sharing ───────────────────────────────────────────────────────────────

    pub fn list_share_members(
        &self,
        resource_type: String,
        resource_id: String,
    ) -> Result<Vec<ShareMemberDto>, KansoError> {
        let members = self
            .rt
            .block_on(self.inner.list_share_members(&resource_type, &resource_id))?;
        Ok(members.into_iter().map(Into::into).collect())
    }

    pub fn add_share_member(
        &self,
        resource_type: String,
        resource_id: String,
        email: String,
        role: String,
    ) -> Result<ShareMemberDto, KansoError> {
        let member = self.rt.block_on(self.inner.add_share_member(
            &resource_type,
            &resource_id,
            &email,
            &role,
        ))?;
        Ok(member.into())
    }

    pub fn remove_share_member(&self, member_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.remove_share_member(&member_id))?;
        Ok(())
    }

    // ── Sketches ─────────────────────────────────────────────────────────────

    /// Persist captured strokes as a sketch on a note. The native layer captures
    /// raw stylus input and hands the normalized strokes here; the engine stores
    /// the canonical CBOR document.
    pub fn create_sketch(
        &self,
        note_id: String,
        title: Option<String>,
        strokes: Vec<InkStroke>,
    ) -> Result<SketchDto, KansoError> {
        let doc = build_sketch_doc(&strokes);
        let sketch =
            self.rt
                .block_on(self.inner.create_sketch(&note_id, title.as_deref(), &doc))?;
        Ok(sketch.into())
    }

    pub fn list_sketches(&self, note_id: String) -> Result<Vec<SketchDto>, KansoError> {
        let sketches = self.rt.block_on(self.inner.list_sketches(&note_id))?;
        Ok(sketches.into_iter().map(Into::into).collect())
    }

    pub fn get_sketch_strokes(&self, sketch_id: String) -> Result<Vec<InkStroke>, KansoError> {
        let Some(sketch) = self.rt.block_on(self.inner.get_sketch(&sketch_id))? else {
            return Err(KansoError::Engine {
                message: format!("sketch not found: {sketch_id}"),
            });
        };
        let doc =
            kanso_ink::SketchDoc::from_cbor(&sketch.data_blob).map_err(|e| KansoError::Engine {
                message: format!("sketch decode failed: {e}"),
            })?;
        Ok(extract_ink_strokes(&doc))
    }

    pub fn update_sketch(
        &self,
        sketch_id: String,
        strokes: Vec<InkStroke>,
    ) -> Result<(), KansoError> {
        let doc = build_sketch_doc(&strokes);
        self.rt
            .block_on(self.inner.update_sketch(&sketch_id, &doc))?;
        Ok(())
    }

    /// Render a sketch preview to PNG bytes (headless `tiny-skia`).
    pub fn render_sketch_preview(
        &self,
        sketch_id: String,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, KansoError> {
        let png = self
            .rt
            .block_on(self.inner.render_sketch_preview(&sketch_id, width, height))?;
        Ok(png)
    }
}

#[cfg(test)]
mod tests {
    use super::KansoEngine;
    use kanso_types::sync::RemoteChange;

    #[test]
    fn ffi_trash_restore_and_purge_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let notebook = engine.create_notebook("Trash FFI".to_string(), None)?;
        let note = engine.create_note(
            notebook.id,
            "Recoverable note".to_string(),
            "This should survive a soft delete.".to_string(),
        )?;

        engine.delete_note(note.id.clone())?;
        let trashed = engine.list_trash()?;
        assert_eq!(trashed.len(), 1);
        assert_eq!(trashed[0].title, "Recoverable note");
        assert!(engine.get_note(note.id.clone())?.is_none());

        engine.restore_note(note.id.clone())?;
        assert!(engine.list_trash()?.is_empty());
        assert!(engine.get_note(note.id.clone())?.is_some());
        engine.set_note_status(note.id.clone(), "completed".to_string())?;
        engine.set_note_favorite(note.id.clone(), true)?;
        let restored = engine.get_note(note.id.clone())?.unwrap();
        assert_eq!(restored.status, "completed");
        assert!(restored.favorite);

        engine.delete_note(note.id.clone())?;
        engine.purge_note(note.id.clone())?;
        assert!(engine.list_trash()?.is_empty());
        assert!(engine.get_note(note.id)?.is_none());

        Ok(())
    }

    #[test]
    fn ffi_encrypted_constructor_keeps_plaintext_off_sync_payloads()
    -> Result<(), Box<dyn std::error::Error>> {
        let passphrase = "correct horse battery staple".to_string();
        let salt = "ffi-encryption-salt".to_string();
        let engine = KansoEngine::open_in_memory_with_encryption_passphrase(
            passphrase.clone(),
            salt.clone(),
        )?;
        let notebook = engine.create_notebook("Encrypted FFI".to_string(), None)?;
        let note = engine.create_note(
            notebook.id,
            "Secret body".to_string(),
            "do not sync this plaintext".to_string(),
        )?;

        let ops = engine.rt.block_on(engine.inner.get_pending_sync_ops(10))?;
        let wire = serde_json::to_string(&ops)?;
        assert!(
            !wire.contains("do not sync this plaintext"),
            "plaintext leaked through the FFI sync boundary"
        );
        assert!(wire.contains("body_cipher"));

        let replica = KansoEngine::open_in_memory_with_encryption_passphrase(passphrase, salt)?;
        for (i, event) in ops.iter().enumerate() {
            let change = RemoteChange {
                server_sequence: i as i64 + 1,
                event: event.clone(),
            };
            replica
                .rt
                .block_on(replica.inner.apply_remote_change(&change))?;
        }

        let synced = replica
            .get_note(note.id)?
            .expect("replica should decrypt the note");
        assert_eq!(synced.body_markdown, "do not sync this plaintext");

        Ok(())
    }

    #[test]
    fn ffi_attachment_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let notebook = engine.create_notebook("Attachments FFI".to_string(), None)?;
        let note = engine.create_note(
            notebook.id,
            "Image note".to_string(),
            "![[attachment:sample.png]]".to_string(),
        )?;

        let attachment = engine.attach_file(
            note.id.clone(),
            super::NewAttachmentDto {
                filename: "sample.png".to_string(),
                mime_type: "image/png".to_string(),
                size_bytes: 22,
                content_hash: "9b1bb083f0872e54f8a94dea4d9b9934f95a4bd4c2ebf488df2dc6c0005a7d27"
                    .to_string(),
                local_path: Some("/tmp/sample.png".to_string()),
            },
        )?;

        let attachments = engine.list_attachments(note.id)?;
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].id, attachment.id);
        assert_eq!(attachments[0].filename, "sample.png");

        engine.delete_attachment(attachment.id)?;
        assert!(
            engine
                .list_attachments(attachments[0].note_id.clone())?
                .is_empty()
        );

        Ok(())
    }

    #[test]
    fn ffi_sketch_strokes_can_be_loaded_and_updated() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let notebook = engine.create_notebook("Sketch Editing FFI".to_string(), None)?;
        let note = engine.create_note(
            notebook.id,
            "Editable sketch".to_string(),
            "![[sketch:placeholder]]".to_string(),
        )?;
        let original = super::InkStroke {
            points: vec![
                super::InkPoint {
                    x: 10.0,
                    y: 12.0,
                    pressure: 0.8,
                },
                super::InkPoint {
                    x: 80.0,
                    y: 84.0,
                    pressure: 1.0,
                },
            ],
            color: super::ColorRgba {
                r: 20,
                g: 80,
                b: 180,
                a: 255,
            },
            width: 3.0,
        };
        let sketch = engine.create_sketch(note.id, Some("flow".to_string()), vec![original])?;

        let loaded = engine.get_sketch_strokes(sketch.id.clone())?;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].points.len(), 2);
        assert_eq!(loaded[0].color.r, 20);
        assert_eq!(loaded[0].width, 3.0);

        let edited = super::InkStroke {
            points: vec![
                super::InkPoint {
                    x: 12.0,
                    y: 14.0,
                    pressure: 0.9,
                },
                super::InkPoint {
                    x: 160.0,
                    y: 120.0,
                    pressure: 1.0,
                },
                super::InkPoint {
                    x: 180.0,
                    y: 150.0,
                    pressure: 0.7,
                },
            ],
            color: super::ColorRgba {
                r: 180,
                g: 40,
                b: 30,
                a: 255,
            },
            width: 5.0,
        };
        engine.update_sketch(sketch.id.clone(), vec![edited])?;

        let reloaded = engine.get_sketch_strokes(sketch.id.clone())?;
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].points.len(), 3);
        assert_eq!(reloaded[0].points[1].x, 160.0);
        assert_eq!(reloaded[0].color.r, 180);
        assert_eq!(reloaded[0].width, 5.0);
        assert!(
            !engine
                .render_sketch_preview(sketch.id, 240, 160)?
                .is_empty()
        );

        Ok(())
    }

    #[test]
    fn ffi_revision_restore_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let notebook = engine.create_notebook("History FFI".to_string(), None)?;
        let note = engine.create_note(
            notebook.id,
            "Recoverable body".to_string(),
            "first version".to_string(),
        )?;

        engine.update_note_body(note.id.clone(), "second version".to_string())?;
        engine.update_note_body(note.id.clone(), "third version".to_string())?;

        let revisions = engine.list_revisions(note.id.clone())?;
        assert_eq!(revisions.len(), 2);
        assert_eq!(revisions[0].body_markdown, "second version");
        assert_eq!(revisions[1].body_markdown, "first version");

        engine.restore_revision(note.id.clone(), revisions[1].id.clone())?;
        let restored = engine.get_note(note.id)?.unwrap();
        assert_eq!(restored.body_markdown, "first version");

        Ok(())
    }

    #[test]
    fn ffi_notebook_management_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let parent = engine.create_notebook("Projects".to_string(), None)?;
        let child = engine.create_notebook("Kanso".to_string(), Some(parent.id.clone()))?;

        assert_eq!(engine.list_root_notebooks()?.len(), 1);
        assert_eq!(engine.list_child_notebooks(parent.id.clone())?.len(), 1);

        engine.rename_notebook(child.id.clone(), "Kanso Notes".to_string())?;
        assert_eq!(
            engine
                .list_child_notebooks(parent.id.clone())?
                .first()
                .map(|notebook| notebook.name.as_str()),
            Some("Kanso Notes")
        );

        engine.move_notebook(child.id.clone(), None)?;
        assert_eq!(engine.list_root_notebooks()?.len(), 2);
        assert!(engine.list_child_notebooks(parent.id.clone())?.is_empty());

        engine.delete_notebook(child.id.clone())?;
        assert_eq!(engine.list_notebooks()?.len(), 1);

        Ok(())
    }

    #[test]
    fn ffi_mcp_and_skills_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;

        let client = engine.register_mcp_client("Claude Desktop".to_string())?;
        assert!(client.id.starts_with("mcpclient:"));
        assert!(!client.trusted);
        assert_eq!(engine.list_mcp_clients()?.len(), 1);
        assert!(engine.list_mcp_capabilities(client.id.clone())?.is_empty());

        engine.grant_mcp_capability(client.id.clone(), "read".to_string())?;
        engine.grant_mcp_capability(client.id.clone(), "run_skill".to_string())?;
        let caps = engine.list_mcp_capabilities(client.id.clone())?;
        assert_eq!(caps, vec!["read".to_string(), "run_skill".to_string()]);

        engine.revoke_mcp_capability(client.id.clone(), "read".to_string())?;
        assert_eq!(
            engine.list_mcp_capabilities(client.id.clone())?,
            vec!["run_skill".to_string()]
        );
        engine.set_mcp_client_trusted(client.id, true)?;
        assert!(engine.list_mcp_clients()?[0].trusted);

        let skill = engine.create_skill(
            "Summarize note".to_string(),
            "Summarize the selected note with citations.".to_string(),
            "note".to_string(),
        )?;
        assert!(skill.enabled);
        assert_eq!(engine.list_skills()?.len(), 1);

        engine.update_skill(
            skill.id.clone(),
            "Summarize selected note".to_string(),
            "Return a concise summary of the selected note.".to_string(),
            "project".to_string(),
            false,
        )?;
        let updated = engine.list_skills()?;
        assert_eq!(updated[0].title, "Summarize selected note");
        assert_eq!(updated[0].scope, "project");
        assert!(!updated[0].enabled);

        let run = engine.start_skill_run(
            skill.id.clone(),
            Some("note".to_string()),
            Some("note:sample".to_string()),
            "dry_run".to_string(),
        )?;
        engine.complete_skill_run(
            run.id,
            "completed".to_string(),
            "No changes applied.".to_string(),
        )?;
        let runs = engine.list_skill_runs(skill.id.clone())?;
        assert_eq!(runs.len(), 1);
        assert_eq!(
            runs[0].output_summary.as_deref(),
            Some("No changes applied.")
        );

        engine.delete_skill(skill.id)?;
        assert!(engine.list_skills()?.is_empty());

        Ok(())
    }

    #[test]
    fn ffi_share_members_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let engine = KansoEngine::open_in_memory()?;
        let notebook = engine.create_notebook("Shared Notebook".to_string(), None)?;
        let note = engine.create_note(
            notebook.id.clone(),
            "Shared Note".to_string(),
            "Invite people here.".to_string(),
        )?;

        let note_member = engine.add_share_member(
            "note".to_string(),
            note.id.clone(),
            "Person@Example.com".to_string(),
            "editor".to_string(),
        )?;
        assert_eq!(note_member.email, "person@example.com");
        assert_eq!(note_member.role, "editor");
        assert_eq!(note_member.resource_type, "note");
        assert_eq!(
            engine
                .list_share_members("note".to_string(), note.id)?
                .len(),
            1
        );

        let notebook_member = engine.add_share_member(
            "notebook".to_string(),
            notebook.id.clone(),
            "viewer@example.com".to_string(),
            "viewer".to_string(),
        )?;
        assert_eq!(notebook_member.resource_type, "notebook");
        assert_eq!(
            engine
                .list_share_members("notebook".to_string(), notebook.id)?
                .len(),
            1
        );

        engine.remove_share_member(note_member.id)?;

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_pushes_and_pulls_between_devices() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let email = format!("ffi-sync-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let device_a = KansoEngine::open_in_memory()?;
        let device_b = KansoEngine::open_in_memory()?;

        let auth_a = device_a.register_http(base_url.clone(), email.clone(), password.clone())?;
        let auth_b = device_b.login_http(base_url.clone(), email, password)?;
        assert_eq!(auth_a.user_id, auth_b.user_id);
        assert_ne!(auth_a.device_id, auth_b.device_id);

        let notebook = device_a.create_notebook("FFI Sync".to_string(), None)?;
        let note = device_a.create_note(
            notebook.id,
            "Wrangler bridge".to_string(),
            "Created through the UniFFI HTTP sync bridge.".to_string(),
        )?;

        let pushed = device_a.sync_http(base_url.clone(), auth_a.token, auth_a.device_id)?;
        assert!(
            pushed.pushed >= 2,
            "expected notebook and note events to push"
        );

        let pulled = device_b.sync_http(base_url, auth_b.token, auth_b.device_id)?;
        assert!(
            pulled.applied >= 2,
            "expected notebook and note events to apply"
        );

        let replicated = device_b
            .get_note(note.id)?
            .expect("replicated note should exist on device B");
        assert_eq!(replicated.title, "Wrangler bridge");
        assert_eq!(
            replicated.body_markdown,
            "Created through the UniFFI HTTP sync bridge."
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_pushes_and_pulls_encrypted_note_payloads()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let email = format!("ffi-e2ee-sync-{unique}@example.test");
        let password = "correct horse battery staple".to_string();
        let backup_passphrase = "shared backup encryption key".to_string();
        let backup_salt = "live-http-encryption-salt".to_string();
        let marker = format!("KANSO_FFI_E2EE_MARKER_{unique}");

        let device_a = KansoEngine::open_in_memory_with_encryption_passphrase(
            backup_passphrase.clone(),
            backup_salt.clone(),
        )?;
        let device_b =
            KansoEngine::open_in_memory_with_encryption_passphrase(backup_passphrase, backup_salt)?;

        let auth_a = device_a.register_http(base_url.clone(), email.clone(), password.clone())?;
        let auth_b = device_b.login_http(base_url.clone(), email, password)?;
        assert_eq!(auth_a.user_id, auth_b.user_id);
        assert_ne!(auth_a.device_id, auth_b.device_id);

        let notebook = device_a.create_notebook("Encrypted FFI Sync".to_string(), None)?;
        let note = device_a.create_note(
            notebook.id,
            "Encrypted Wrangler bridge".to_string(),
            marker.clone(),
        )?;

        let pushed = device_a.sync_http(base_url.clone(), auth_a.token, auth_a.device_id)?;
        assert!(
            pushed.pushed >= 2,
            "expected notebook and encrypted note events to push"
        );

        let pulled = device_b.sync_http(base_url, auth_b.token, auth_b.device_id)?;
        assert!(
            pulled.applied >= 2,
            "expected encrypted note events to apply"
        );

        let replicated = device_b
            .get_note(note.id)?
            .expect("replicated note should exist on device B");
        assert_eq!(replicated.title, "Encrypted Wrangler bridge");
        assert_eq!(replicated.body_markdown, marker);

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_transfers_share_members() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let email = format!("ffi-share-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let device_a = KansoEngine::open_in_memory()?;
        let device_b = KansoEngine::open_in_memory()?;

        let auth_a = device_a.register_http(base_url.clone(), email.clone(), password.clone())?;
        let auth_b = device_b.login_http(base_url.clone(), email, password)?;

        let notebook = device_a.create_notebook("Shared Team".to_string(), None)?;
        let note = device_a.create_note(
            notebook.id,
            "Share metadata".to_string(),
            "Members should back up through D1 sync.".to_string(),
        )?;
        let member = device_a.add_share_member(
            "note".to_string(),
            note.id.clone(),
            "Editor@Example.com".to_string(),
            "editor".to_string(),
        )?;

        let pushed = device_a.sync_http(
            base_url.clone(),
            auth_a.token.clone(),
            auth_a.device_id.clone(),
        )?;
        assert!(
            pushed.pushed >= 3,
            "expected notebook, note, and member events"
        );

        let pulled = device_b.sync_http(
            base_url.clone(),
            auth_b.token.clone(),
            auth_b.device_id.clone(),
        )?;
        assert!(
            pulled.applied >= 3,
            "expected notebook, note, and member events"
        );

        let members = device_b.list_share_members("note".to_string(), note.id.clone())?;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].id, member.id);
        assert_eq!(members[0].email, "editor@example.com");
        assert_eq!(members[0].role, "editor");

        device_a.remove_share_member(member.id)?;
        let removed = device_a.sync_http(base_url.clone(), auth_a.token, auth_a.device_id)?;
        assert!(removed.pushed >= 1, "expected member removal to push");
        let pulled_removal = device_b.sync_http(base_url, auth_b.token, auth_b.device_id)?;
        assert!(
            pulled_removal.deleted >= 1,
            "expected member removal to apply"
        );
        assert!(
            device_b
                .list_share_members("note".to_string(), note.id)?
                .is_empty()
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_shares_note_with_second_account() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-share-owner-{unique}@example.test");
        let member_email = format!("ffi-share-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let member_device = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let member_auth =
            member_device.register_http(base_url.clone(), member_email.clone(), password)?;
        assert_ne!(owner_auth.user_id, member_auth.user_id);

        let notebook = owner.create_notebook("Shared Source".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Cross-account note".to_string(),
            "Original body before sharing.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let private_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert_eq!(
            private_pull.applied, 0,
            "member should not see owner content before sharing"
        );

        let invite = owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            member_email,
            "viewer".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let shared_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(
            shared_pull.applied >= 3,
            "expected notebook, note, and share member backfill"
        );
        let replicated = member_device
            .get_note(note.id.clone())?
            .expect("shared note should be visible to the member account");
        assert_eq!(replicated.title, "Cross-account note");
        assert_eq!(replicated.body_markdown, "Original body before sharing.");
        assert_eq!(
            member_device
                .list_share_members("note".to_string(), note.id.clone())?
                .len(),
            1
        );

        owner.update_note_body(note.id.clone(), "Updated for the member.".to_string())?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        let update_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(update_pull.applied >= 1, "expected shared note update");
        assert_eq!(
            member_device
                .get_note(note.id.clone())?
                .expect("member still has shared note")
                .body_markdown,
            "Updated for the member."
        );

        owner.remove_share_member(invite.id)?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        let removal_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(removal_pull.deleted >= 1, "expected share removal to apply");
        assert!(
            member_device
                .list_share_members("note".to_string(), note.id.clone())?
                .is_empty()
        );

        owner.update_note_body(note.id.clone(), "Post-removal private edit.".to_string())?;
        owner.sync_http(base_url.clone(), owner_auth.token, owner_auth.device_id)?;
        let post_removal_pull =
            member_device.sync_http(base_url, member_auth.token, member_auth.device_id)?;
        assert_eq!(
            post_removal_pull.applied, 0,
            "removed member should not receive later edits"
        );
        assert_eq!(
            member_device
                .get_note(note.id)?
                .expect("removed member keeps prior local copy")
                .body_markdown,
            "Updated for the member."
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_backfills_share_for_later_registered_member()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-pending-owner-{unique}@example.test");
        let member_email = format!("ffi-pending-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;

        let notebook = owner.create_notebook("Pending Invite".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Invited before signup".to_string(),
            "This should appear after registration.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let invite = owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            member_email.clone(),
            "editor".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let member_device = KansoEngine::open_in_memory()?;
        let member_auth = member_device.register_http(base_url.clone(), member_email, password)?;
        assert_ne!(owner_auth.user_id, member_auth.user_id);

        let pulled = member_device.sync_http(base_url, member_auth.token, member_auth.device_id)?;
        assert!(
            pulled.applied >= 3,
            "expected notebook, note, and share member backfill after registration"
        );
        assert_eq!(
            member_device
                .get_note(note.id.clone())?
                .expect("pending invite should backfill shared note")
                .body_markdown,
            "This should appear after registration."
        );
        let members = member_device.list_share_members("note".to_string(), note.id)?;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].id, invite.id);

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_shares_notebook_with_second_account() -> Result<(), Box<dyn std::error::Error>>
    {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-notebook-owner-{unique}@example.test");
        let member_email = format!("ffi-notebook-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let member_device = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let member_auth =
            member_device.register_http(base_url.clone(), member_email.clone(), password)?;
        assert_ne!(owner_auth.user_id, member_auth.user_id);

        let notebook = owner.create_notebook("Notebook Share".to_string(), None)?;
        let first = owner.create_note(
            notebook.id.clone(),
            "First shared page".to_string(),
            "Notebook body one.".to_string(),
        )?;
        let second = owner.create_note(
            notebook.id.clone(),
            "Second shared page".to_string(),
            "Notebook body two.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let private_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert_eq!(
            private_pull.applied, 0,
            "member should not see notebook content before sharing"
        );

        let invite = owner.add_share_member(
            "notebook".to_string(),
            notebook.id.clone(),
            member_email,
            "viewer".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let shared_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(
            shared_pull.applied >= 4,
            "expected notebook, notes, and share member backfill"
        );
        assert_eq!(
            member_device.list_notes(notebook.id.clone())?.len(),
            2,
            "shared notebook should include existing notes"
        );
        assert_eq!(
            member_device
                .get_note(first.id.clone())?
                .expect("first shared note should exist")
                .body_markdown,
            "Notebook body one."
        );
        assert_eq!(
            member_device
                .list_share_members("notebook".to_string(), notebook.id.clone())?
                .len(),
            1
        );

        owner.update_note_body(second.id.clone(), "Notebook body two, updated.".to_string())?;
        let third = owner.create_note(
            notebook.id.clone(),
            "Third shared page".to_string(),
            "Created after sharing.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        let update_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(
            update_pull.applied >= 2,
            "expected existing-note update and new note in shared notebook"
        );
        assert_eq!(
            member_device
                .get_note(second.id.clone())?
                .expect("second shared note should still exist")
                .body_markdown,
            "Notebook body two, updated."
        );
        assert_eq!(
            member_device
                .get_note(third.id.clone())?
                .expect("new shared note should replicate")
                .body_markdown,
            "Created after sharing."
        );

        owner.remove_share_member(invite.id)?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        let removal_pull = member_device.sync_http(
            base_url.clone(),
            member_auth.token.clone(),
            member_auth.device_id.clone(),
        )?;
        assert!(removal_pull.deleted >= 1, "expected notebook share removal");
        assert!(
            member_device
                .list_share_members("notebook".to_string(), notebook.id.clone())?
                .is_empty()
        );

        owner.update_note_body(
            third.id.clone(),
            "Private after notebook unshare.".to_string(),
        )?;
        owner.sync_http(base_url.clone(), owner_auth.token, owner_auth.device_id)?;
        let post_removal_pull =
            member_device.sync_http(base_url, member_auth.token, member_auth.device_id)?;
        assert_eq!(
            post_removal_pull.applied, 0,
            "removed notebook member should not receive later edits"
        );
        assert_eq!(
            member_device
                .get_note(third.id)?
                .expect("removed member keeps prior local copy")
                .body_markdown,
            "Created after sharing."
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_editor_writeback_reaches_owner() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-editor-owner-{unique}@example.test");
        let editor_email = format!("ffi-editor-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let editor = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let editor_auth = editor.register_http(base_url.clone(), editor_email.clone(), password)?;

        let notebook = owner.create_notebook("Editor Writeback".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Shared editable note".to_string(),
            "Owner draft.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            editor_email,
            "editor".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        editor.sync_http(
            base_url.clone(),
            editor_auth.token.clone(),
            editor_auth.device_id.clone(),
        )?;
        editor.update_note_body(note.id.clone(), "Edited by member.".to_string())?;
        let pushed =
            editor.sync_http(base_url.clone(), editor_auth.token, editor_auth.device_id)?;
        assert!(pushed.pushed >= 1, "editor edit should push");

        let pulled = owner.sync_http(base_url, owner_auth.token, owner_auth.device_id)?;
        assert!(pulled.applied >= 1, "owner should pull editor edit");
        assert_eq!(
            owner
                .get_note(note.id)?
                .expect("owner keeps shared note")
                .body_markdown,
            "Edited by member."
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_viewer_write_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-viewer-owner-{unique}@example.test");
        let viewer_email = format!("ffi-viewer-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let viewer = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let viewer_auth = viewer.register_http(base_url.clone(), viewer_email.clone(), password)?;

        let notebook = owner.create_notebook("Viewer Denial".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Readonly note".to_string(),
            "Owner-only body.".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            viewer_email,
            "viewer".to_string(),
        )?;
        owner.sync_http(base_url.clone(), owner_auth.token, owner_auth.device_id)?;

        viewer.sync_http(
            base_url.clone(),
            viewer_auth.token.clone(),
            viewer_auth.device_id.clone(),
        )?;
        viewer.update_note_body(note.id, "Viewer should not sync this.".to_string())?;
        let denied = viewer.sync_http(base_url, viewer_auth.token, viewer_auth.device_id);
        assert!(denied.is_err(), "viewer write should be rejected by Worker");

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_editor_can_create_note_in_shared_notebook()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-editor-notebook-owner-{unique}@example.test");
        let editor_email = format!("ffi-editor-notebook-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let editor = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let editor_auth = editor.register_http(base_url.clone(), editor_email.clone(), password)?;

        let notebook = owner.create_notebook("Notebook Editor".to_string(), None)?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;
        owner.add_share_member(
            "notebook".to_string(),
            notebook.id.clone(),
            editor_email,
            "editor".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        editor.sync_http(
            base_url.clone(),
            editor_auth.token.clone(),
            editor_auth.device_id.clone(),
        )?;
        let member_note = editor.create_note(
            notebook.id,
            "Member-created page".to_string(),
            "Created by notebook editor.".to_string(),
        )?;
        editor.sync_http(base_url.clone(), editor_auth.token, editor_auth.device_id)?;

        let pulled = owner.sync_http(base_url, owner_auth.token, owner_auth.device_id)?;
        assert!(
            pulled.applied >= 1,
            "owner should receive member-created note"
        );
        assert_eq!(
            owner
                .get_note(member_note.id)?
                .expect("member-created note should reach owner")
                .body_markdown,
            "Created by notebook editor."
        );

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_shares_attachment_blob_with_second_account()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-attachment-owner-{unique}@example.test");
        let member_email = format!("ffi-attachment-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let member_device = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let member_auth =
            member_device.register_http(base_url.clone(), member_email.clone(), password)?;

        let dir_owner = std::env::temp_dir().join(format!("kanso-share-blob-owner-{unique}"));
        let dir_member = std::env::temp_dir().join(format!("kanso-share-blob-member-{unique}"));
        std::fs::create_dir_all(&dir_owner)?;
        std::fs::create_dir_all(&dir_member)?;
        let source_file = dir_owner.join("shared.txt");
        std::fs::write(&source_file, b"hello kanso attachment")?;

        let notebook = owner.create_notebook("Shared Attachment".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Attachment across accounts".to_string(),
            "Blob should follow the share.".to_string(),
        )?;
        owner.attach_file(
            note.id.clone(),
            super::NewAttachmentDto {
                filename: "shared.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size_bytes: 22,
                content_hash: "9b1bb083f0872e54f8a94dea4d9b9934f95a4bd4c2ebf488df2dc6c0005a7d27"
                    .to_string(),
                local_path: Some(source_file.to_string_lossy().to_string()),
            },
        )?;
        let pushed = owner.sync_http_with_blobs(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
            dir_owner.to_string_lossy().to_string(),
        )?;
        assert!(pushed.uploaded_blobs >= 1);

        owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            member_email,
            "viewer".to_string(),
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        let pulled = member_device.sync_http_with_blobs(
            base_url,
            member_auth.token,
            member_auth.device_id,
            dir_member.to_string_lossy().to_string(),
        )?;
        assert!(
            pulled.applied >= 4,
            "expected note, share, and attachment events"
        );
        assert!(
            pulled.downloaded_blobs >= 1,
            "expected shared blob download"
        );

        let attachments = member_device.list_attachments(note.id)?;
        assert_eq!(attachments.len(), 1);
        let local_path = attachments[0]
            .local_path
            .as_ref()
            .expect("download should set local attachment path");
        assert_eq!(std::fs::read(local_path)?, b"hello kanso attachment");

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_shares_sketch_with_second_account() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let owner_email = format!("ffi-sketch-owner-{unique}@example.test");
        let member_email = format!("ffi-sketch-member-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let owner = KansoEngine::open_in_memory()?;
        let member_device = KansoEngine::open_in_memory()?;

        let owner_auth = owner.register_http(base_url.clone(), owner_email, password.clone())?;
        let member_auth =
            member_device.register_http(base_url.clone(), member_email.clone(), password)?;

        let notebook = owner.create_notebook("Shared Sketch".to_string(), None)?;
        let note = owner.create_note(
            notebook.id,
            "Sketch across accounts".to_string(),
            "Sketch should follow the share.".to_string(),
        )?;
        let sketch = owner.create_sketch(
            note.id.clone(),
            Some("diagram".to_string()),
            vec![super::InkStroke {
                points: vec![
                    super::InkPoint {
                        x: 10.0,
                        y: 10.0,
                        pressure: 1.0,
                    },
                    super::InkPoint {
                        x: 90.0,
                        y: 80.0,
                        pressure: 1.0,
                    },
                ],
                color: super::ColorRgba {
                    r: 20,
                    g: 80,
                    b: 180,
                    a: 255,
                },
                width: 3.0,
            }],
        )?;
        owner.sync_http(
            base_url.clone(),
            owner_auth.token.clone(),
            owner_auth.device_id.clone(),
        )?;

        owner.add_share_member(
            "note".to_string(),
            note.id.clone(),
            member_email,
            "viewer".to_string(),
        )?;
        owner.sync_http(base_url.clone(), owner_auth.token, owner_auth.device_id)?;

        let pulled = member_device.sync_http(base_url, member_auth.token, member_auth.device_id)?;
        assert!(
            pulled.applied >= 4,
            "expected note, share, and sketch events"
        );

        let sketches = member_device.list_sketches(note.id)?;
        assert_eq!(sketches.len(), 1);
        assert_eq!(sketches[0].id, sketch.id);
        let preview = member_device.render_sketch_preview(sketches[0].id.clone(), 160, 120)?;
        assert!(!preview.is_empty(), "shared sketch should render a preview");

        Ok(())
    }

    #[test]
    #[ignore = "requires a live Kanso Cloud-compatible server; set KANSO_TEST_HTTP_BASE"]
    fn ffi_http_sync_transfers_attachment_blobs() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = std::env::var("KANSO_TEST_HTTP_BASE")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let email = format!("ffi-blob-{unique}@example.test");
        let password = "correct horse battery staple".to_string();

        let device_a = KansoEngine::open_in_memory()?;
        let device_b = KansoEngine::open_in_memory()?;

        let auth_a = device_a.register_http(base_url.clone(), email.clone(), password.clone())?;
        let auth_b = device_b.login_http(base_url.clone(), email, password)?;

        let dir_a = std::env::temp_dir().join(format!("kanso-ffi-blob-a-{unique}"));
        let dir_b = std::env::temp_dir().join(format!("kanso-ffi-blob-b-{unique}"));
        std::fs::create_dir_all(&dir_a)?;
        std::fs::create_dir_all(&dir_b)?;
        let file_a = dir_a.join("sample.txt");
        std::fs::write(&file_a, b"hello kanso attachment")?;

        let notebook = device_a.create_notebook("Blob Sync".to_string(), None)?;
        let note = device_a.create_note(
            notebook.id,
            "Blob-backed note".to_string(),
            "Attachment backup.".to_string(),
        )?;
        let attachment = device_a.attach_file(
            note.id.clone(),
            super::NewAttachmentDto {
                filename: "sample.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size_bytes: 22,
                content_hash: "9b1bb083f0872e54f8a94dea4d9b9934f95a4bd4c2ebf488df2dc6c0005a7d27"
                    .to_string(),
                local_path: Some(file_a.to_string_lossy().to_string()),
            },
        )?;
        let attachment_ref = attachment
            .id
            .strip_prefix("attachment:")
            .unwrap_or(&attachment.id)
            .to_string();
        device_a.update_note_body(
            note.id.clone(),
            format!("Attachment backup.\n\n![[attachment:{attachment_ref}]]"),
        )?;

        let pushed = device_a.sync_http_with_blobs(
            base_url.clone(),
            auth_a.token,
            auth_a.device_id,
            dir_a.to_string_lossy().to_string(),
        )?;
        assert!(pushed.uploaded_blobs >= 1);
        assert!(pushed.pushed >= 3);

        let pulled = device_b.sync_http_with_blobs(
            base_url,
            auth_b.token,
            auth_b.device_id,
            dir_b.to_string_lossy().to_string(),
        )?;
        assert!(pulled.applied >= 3);
        assert!(pulled.downloaded_blobs >= 1);

        let attachments = device_b.list_attachments(note.id.clone())?;
        assert_eq!(attachments.len(), 1);
        let restored_path = attachments[0]
            .local_path
            .as_deref()
            .expect("downloaded blob should have a local path");
        assert_eq!(std::fs::read(restored_path)?, b"hello kanso attachment");

        let html = device_b.render_note_html(note.id)?;
        assert!(html.contains("data-kanso-kind=\"attachment\""));
        assert!(html.contains("sample.txt"));

        let _ = std::fs::remove_dir_all(dir_a);
        let _ = std::fs::remove_dir_all(dir_b);

        Ok(())
    }
}
