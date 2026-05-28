//! Sketch commands.

use kanso_types::SketchId;
use kanso_types::payloads::SketchPayload;
use kanso_types::sync::{EntityType, Operation};

use crate::db::{Engine, enqueue_outbox, now_ms};
use crate::error::{EngineError, Result};
use crate::models::Sketch;

impl Engine {
    /// Persist a new sketch for `note_id` and enqueue a sync event.
    pub async fn create_sketch(
        &self,
        note_id: &str,
        title: Option<&str>,
        doc: &kanso_ink::SketchDoc,
    ) -> Result<Sketch> {
        let id = SketchId::new().0;
        let now = now_ms();
        let blob = doc.to_cbor();
        let format_version = doc.format_version as i64;

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO sketches \
             (id, note_id, title, format_version, data_blob, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(note_id)
        .bind(title.map(str::to_string))
        .bind(format_version)
        .bind(&blob)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let (payload_blob, data_cipher) = self.encrypt_blob(&blob)?;
        let payload = SketchPayload {
            note_id: note_id.to_string(),
            title: title.map(str::to_string),
            format_version,
            data_blob: payload_blob,
            data_cipher,
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Sketch,
            &id,
            Operation::SketchCreated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;

        Ok(Sketch {
            id,
            note_id: note_id.to_string(),
            title: title.map(str::to_string),
            format_version,
            data_blob: blob,
            preview_attachment_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Overwrite the CBOR blob for an existing sketch and enqueue a sync event.
    pub async fn update_sketch(&self, sketch_id: &str, doc: &kanso_ink::SketchDoc) -> Result<()> {
        let now = now_ms();
        let blob = doc.to_cbor();
        let format_version = doc.format_version as i64;

        let mut tx = self.pool.begin().await?;

        let row: Option<(String,)> =
            sqlx::query_as("SELECT note_id FROM sketches WHERE id = ?")
                .bind(sketch_id)
                .fetch_optional(&mut *tx)
                .await?;
        let (note_id,) = row.ok_or_else(|| EngineError::NotFound(sketch_id.to_string()))?;

        sqlx::query(
            "UPDATE sketches SET data_blob = ?, format_version = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&blob)
        .bind(format_version)
        .bind(now)
        .bind(sketch_id)
        .execute(&mut *tx)
        .await?;

        let (payload_blob, data_cipher) = self.encrypt_blob(&blob)?;
        let payload = SketchPayload {
            note_id,
            title: None,
            format_version,
            data_blob: payload_blob,
            data_cipher,
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Sketch,
            sketch_id,
            Operation::SketchUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Fetch a single sketch by `sketch_id`, or `None` if it does not exist.
    pub async fn get_sketch(&self, sketch_id: &str) -> Result<Option<Sketch>> {
        let sketch = sqlx::query_as::<_, Sketch>(
            "SELECT id, note_id, title, format_version, data_blob, \
             preview_attachment_id, created_at, updated_at \
             FROM sketches WHERE id = ?",
        )
        .bind(sketch_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(sketch)
    }

    /// Return all sketches belonging to `note_id`, ordered by creation time.
    pub async fn list_sketches(&self, note_id: &str) -> Result<Vec<Sketch>> {
        let sketches = sqlx::query_as::<_, Sketch>(
            "SELECT id, note_id, title, format_version, data_blob, \
             preview_attachment_id, created_at, updated_at \
             FROM sketches WHERE note_id = ? ORDER BY created_at",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(sketches)
    }

    /// Render a raster preview of the sketch at the given dimensions.
    ///
    /// Returns the raw image bytes (PNG or similar, depending on the renderer).
    /// Returns [`EngineError::NotFound`] if the sketch does not exist, or
    /// [`EngineError::Decode`] if the CBOR blob is corrupt or rendering fails.
    pub async fn render_sketch_preview(
        &self,
        sketch_id: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT data_blob FROM sketches WHERE id = ?")
                .bind(sketch_id)
                .fetch_optional(&self.pool)
                .await?;
        let Some((blob,)) = row else {
            return Err(EngineError::NotFound(sketch_id.to_string()));
        };

        let doc = kanso_ink::SketchDoc::from_cbor(&blob)
            .map_err(|e| EngineError::Decode(e.to_string()))?;

        kanso_ink::raster::render_preview(&doc, width, height)
            .ok_or_else(|| EngineError::Decode("sketch preview render failed".to_string()))
    }
}
