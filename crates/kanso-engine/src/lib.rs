//! The Kanso product engine.
//!
//! Owns product truth: notebooks, notes, tags, attachments, sketches, Markdown
//! indexing, FTS search, revisions, soft deletes, the sync outbox, and inbound
//! remote-change application — all behind a command API the native apps call via
//! UniFFI. Native apps never touch the tables directly.

mod attachments;
mod db;
mod error;
mod markdown;
mod models;
mod notebooks;
mod notes;
mod remote;
mod sketches;
mod sync;
mod sync_client;
mod tags;

pub use db::Engine;
pub use error::{EngineError, Result};
pub use models::{ApplyOutcome, Attachment, NewAttachment, Note, Notebook, Sketch, SyncReport, Tag};
pub use sync_client::SyncTransport;

/// Markdown extraction (headings, links, Kanso references, tasks).
pub mod md {
    pub use crate::markdown::{Extracted, RefKind, Reference, Task, extract};
}

#[cfg(test)]
mod tests {
    use crate::{ApplyOutcome, Engine, NewAttachment};
    use kanso_types::payloads::NoteUpdatedPayload;
    use kanso_types::sync::{EntityType, Operation, OutboxEvent, RemoteChange};

    #[tokio::test]
    async fn full_product_loop() {
        let engine = Engine::open_in_memory().await.unwrap();

        let notebook = engine.create_notebook("Research", None).await.unwrap();
        assert!(notebook.id.starts_with("notebook:"));

        let note = engine
            .create_note(
                &notebook.id,
                "Sync flow",
                "# Sync\n\n- [ ] design the outbox\n\nSee [[Product Direction]] and ![[sketch:sync-flow]].",
            )
            .await
            .unwrap();
        assert!(note.id.starts_with("note:"));

        let hits = engine.search_notes("outbox").await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, note.id);

        engine
            .update_note_body(&note.id, "# Sync v2\n\nfully rewritten body")
            .await
            .unwrap();
        assert_eq!(engine.search_notes("outbox").await.unwrap().len(), 0);
        assert_eq!(engine.search_notes("rewritten").await.unwrap().len(), 1);

        // notebook-create + note-create + note-update = 3 outbox events.
        assert_eq!(engine.pending_outbox_count().await.unwrap(), 3);

        let ops = engine.get_pending_sync_ops(10).await.unwrap();
        assert_eq!(ops.len(), 3);
        assert!(ops.windows(2).all(|w| w[0].local_sequence < w[1].local_sequence));

        let ids: Vec<String> = ops.iter().map(|o| o.id.to_string()).collect();
        engine.mark_sync_ops_acknowledged(&ids).await.unwrap();
        assert_eq!(engine.pending_outbox_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn delete_restore_and_move() {
        let engine = Engine::open_in_memory().await.unwrap();
        let a = engine.create_notebook("A", None).await.unwrap();
        let b = engine.create_notebook("B", None).await.unwrap();
        let note = engine.create_note(&a.id, "n", "body").await.unwrap();

        engine.delete_note(&note.id).await.unwrap();
        assert!(engine.get_note(&note.id).await.unwrap().is_none());
        assert_eq!(engine.search_notes("body").await.unwrap().len(), 0);
        assert_eq!(engine.list_notes(&a.id).await.unwrap().len(), 0);

        engine.restore_note(&note.id).await.unwrap();
        assert!(engine.get_note(&note.id).await.unwrap().is_some());
        assert_eq!(engine.search_notes("body").await.unwrap().len(), 1);

        engine.move_note(&note.id, &b.id).await.unwrap();
        assert_eq!(engine.list_notes(&a.id).await.unwrap().len(), 0);
        assert_eq!(engine.list_notes(&b.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn attachments_crud() {
        let engine = Engine::open_in_memory().await.unwrap();
        let nb = engine.create_notebook("A", None).await.unwrap();
        let note = engine.create_note(&nb.id, "n", "body").await.unwrap();

        let att = engine
            .attach_file(
                &note.id,
                NewAttachment {
                    filename: "diagram.png".into(),
                    mime_type: "image/png".into(),
                    size_bytes: 1024,
                    content_hash: "abc123".into(),
                    local_path: Some("/tmp/diagram.png".into()),
                },
            )
            .await
            .unwrap();
        assert!(att.id.starts_with("attachment:"));
        assert_eq!(engine.list_attachments(&note.id).await.unwrap().len(), 1);

        engine.delete_attachment(&att.id).await.unwrap();
        assert_eq!(engine.list_attachments(&note.id).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn sketches_and_preview() {
        use kanso_ink::{Background, Element, Point, Rgba, SketchDoc, Stroke, Tool};

        let engine = Engine::open_in_memory().await.unwrap();
        let nb = engine.create_notebook("A", None).await.unwrap();
        let note = engine.create_note(&nb.id, "n", "body ![[sketch:x]]").await.unwrap();

        let mut doc = SketchDoc::new();
        doc.background = Background::Dotted;
        doc.elements.push(Element::Stroke(Stroke {
            points: vec![
                Point { x: 1.0, y: 1.0, pressure: 1.0, tilt: 0.0, t: 0.0 },
                Point { x: 40.0, y: 30.0, pressure: 1.0, tilt: 0.0, t: 1.0 },
                Point { x: 80.0, y: 5.0, pressure: 1.0, tilt: 0.0, t: 2.0 },
            ],
            color: Rgba { r: 0, g: 0, b: 0, a: 255 },
            base_width: 2.0,
            tool: Tool::Pen,
        }));

        let sketch = engine.create_sketch(&note.id, Some("flow"), &doc).await.unwrap();
        assert!(sketch.id.starts_with("sketch:"));
        assert_eq!(engine.list_sketches(&note.id).await.unwrap().len(), 1);

        // Round-trips back to a decodable doc and renders a PNG.
        let fetched = engine.get_sketch(&sketch.id).await.unwrap().unwrap();
        let back = SketchDoc::from_cbor(&fetched.data_blob).unwrap();
        assert_eq!(back.elements.len(), 1);

        let png = engine.render_sketch_preview(&sketch.id, 100, 80).await.unwrap();
        assert!(png.len() > 8 && &png[1..4] == b"PNG");
    }

    #[tokio::test]
    async fn tags_and_listing() {
        let engine = Engine::open_in_memory().await.unwrap();
        let nb = engine.create_notebook("Work", None).await.unwrap();
        let note = engine.create_note(&nb.id, "Meeting", "notes").await.unwrap();

        let tag = engine.create_tag("important").await.unwrap();
        engine.tag_note(&note.id, &tag.id).await.unwrap();

        assert_eq!(engine.list_tags().await.unwrap().len(), 1);
        assert_eq!(engine.list_notes(&nb.id).await.unwrap().len(), 1);
        assert_eq!(engine.list_notebooks().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sync_round_trip_between_devices() {
        // Device A creates and edits.
        let a = Engine::open_in_memory().await.unwrap();
        let nb = a.create_notebook("Research", None).await.unwrap();
        let note = a.create_note(&nb.id, "Title", "body one").await.unwrap();
        a.update_note_body(&note.id, "body two").await.unwrap();

        let ops = a.get_pending_sync_ops(100).await.unwrap();

        // Device B applies A's change log.
        let b = Engine::open_in_memory().await.unwrap();
        for (i, event) in ops.into_iter().enumerate() {
            let change = RemoteChange { server_sequence: i as i64 + 1, event };
            b.apply_remote_change(&change).await.unwrap();
        }

        // B converged on A's state...
        assert_eq!(b.list_notebooks().await.unwrap().len(), 1);
        let note_b = b.get_note(&note.id).await.unwrap().unwrap();
        assert_eq!(note_b.body_markdown, "body two");
        assert_eq!(b.search_notes("two").await.unwrap().len(), 1);

        // ...and applying remote changes does NOT echo back into the outbox.
        assert_eq!(b.pending_outbox_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn stale_remote_update_is_kept_as_conflict() {
        let engine = Engine::open_in_memory().await.unwrap();
        let nb = engine.create_notebook("Research", None).await.unwrap();
        let note = engine.create_note(&nb.id, "Title", "local newer body").await.unwrap();

        // A remote update with an ancient timestamp — older than our local note.
        let stale = NoteUpdatedPayload {
            title: "Title".into(),
            body_markdown: "stale remote body".into(),
            updated_at: 1,
        };
        let change = RemoteChange {
            server_sequence: 999,
            event: OutboxEvent {
                id: uuid::Uuid::now_v7(),
                entity_type: EntityType::Note,
                entity_id: note.id.clone(),
                operation: Operation::NoteUpdated,
                payload: serde_json::to_value(&stale).unwrap(),
                local_sequence: 999,
            },
        };

        let outcome = engine.apply_remote_change(&change).await.unwrap();
        assert_eq!(outcome, ApplyOutcome::Conflicted);

        // Local text is preserved; nothing is silently discarded.
        let local = engine.get_note(&note.id).await.unwrap().unwrap();
        assert_eq!(local.body_markdown, "local newer body");
    }

    #[tokio::test]
    async fn device_sync_loop_converges() {
        use std::sync::{Arc, Mutex};

        // A shared in-memory backend: an ordered log tagged with each event's
        // origin device, so a device never pulls its own events back.
        #[derive(Default)]
        struct Backend {
            log: Mutex<Vec<(String, i64, OutboxEvent)>>,
            high_water: Mutex<i64>,
        }

        struct Transport {
            backend: Arc<Backend>,
        }

        #[async_trait::async_trait]
        impl crate::SyncTransport for Transport {
            async fn push(
                &self,
                device_id: &str,
                _since: i64,
                events: Vec<OutboxEvent>,
            ) -> std::result::Result<(Vec<uuid::Uuid>, i64), String> {
                let mut log = self.backend.log.lock().unwrap();
                let mut hw = self.backend.high_water.lock().unwrap();
                let mut accepted = Vec::new();
                for event in events {
                    if log.iter().any(|(_, _, e)| e.id == event.id) {
                        accepted.push(event.id); // idempotent
                        continue;
                    }
                    *hw += 1;
                    accepted.push(event.id);
                    log.push((device_id.to_string(), *hw, event));
                }
                Ok((accepted, *hw))
            }

            async fn pull(
                &self,
                device_id: &str,
                since: i64,
                limit: i64,
            ) -> std::result::Result<Vec<RemoteChange>, String> {
                let log = self.backend.log.lock().unwrap();
                Ok(log
                    .iter()
                    .filter(|(origin, seq, _)| *seq > since && origin != device_id)
                    .take(limit as usize)
                    .map(|(_, seq, event)| RemoteChange { server_sequence: *seq, event: event.clone() })
                    .collect())
            }
        }

        let backend = Arc::new(Backend::default());
        let a = Engine::open_in_memory().await.unwrap();
        let b = Engine::open_in_memory().await.unwrap();
        let ta = Transport { backend: backend.clone() };
        let tb = Transport { backend: backend.clone() };

        // A creates content and pushes it up.
        let nb = a.create_notebook("Shared", None).await.unwrap();
        let note = a.create_note(&nb.id, "Hello", "from A").await.unwrap();
        let ra = a.sync("device:a", &ta).await.unwrap();
        assert!(ra.pushed >= 2);

        // B pulls and converges.
        let rb = b.sync("device:b", &tb).await.unwrap();
        assert!(rb.applied >= 2);
        assert_eq!(b.get_note(&note.id).await.unwrap().unwrap().body_markdown, "from A");

        // B edits and pushes; A pulls the edit down.
        b.update_note_body(&note.id, "edited by B").await.unwrap();
        b.sync("device:b", &tb).await.unwrap();
        a.sync("device:a", &ta).await.unwrap();
        assert_eq!(a.get_note(&note.id).await.unwrap().unwrap().body_markdown, "edited by B");

        // No device re-applied its own events into its outbox.
        assert_eq!(a.pending_outbox_count().await.unwrap(), 0);
        assert_eq!(b.pending_outbox_count().await.unwrap(), 0);
    }
}
