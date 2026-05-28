//! UniFFI bindings: a synchronous facade over the async engine.
//!
//! Native apps (Swift/Kotlin) call these blocking methods; we drive the async
//! engine on an owned Tokio runtime via `block_on`. This matches the
//! architecture note about keeping the command API synchronous under UniFFI.

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

    /// Open a transient in-memory database (useful for tests / previews).
    #[uniffi::constructor]
    pub fn open_in_memory() -> Result<Arc<Self>, KansoError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| KansoError::Engine {
            message: e.to_string(),
        })?;
        let inner = rt.block_on(Engine::open_in_memory())?;
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

    // ── Notes ────────────────────────────────────────────────────────────────

    pub fn create_note(
        &self,
        notebook_id: String,
        title: String,
        body_markdown: String,
    ) -> Result<NoteDto, KansoError> {
        let note = self
            .rt
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

    pub fn delete_note(&self, note_id: String) -> Result<(), KansoError> {
        self.rt.block_on(self.inner.delete_note(&note_id))?;
        Ok(())
    }

    pub fn move_note(&self, note_id: String, notebook_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.move_note(&note_id, &notebook_id))?;
        Ok(())
    }

    // ── Tags ─────────────────────────────────────────────────────────────────

    pub fn create_tag(&self, name: String) -> Result<TagDto, KansoError> {
        let tag = self.rt.block_on(self.inner.create_tag(&name))?;
        Ok(tag.into())
    }

    pub fn tag_note(&self, note_id: String, tag_id: String) -> Result<(), KansoError> {
        self.rt
            .block_on(self.inner.tag_note(&note_id, &tag_id))?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<TagDto>, KansoError> {
        let tags = self.rt.block_on(self.inner.list_tags())?;
        Ok(tags.into_iter().map(Into::into).collect())
    }
}
