//! The Kanso product engine.
//!
//! Owns product truth: notebooks, notes, tags, Markdown indexing, FTS search,
//! revisions, soft deletes, and the sync outbox — all behind a command API the
//! native apps call via UniFFI. Native apps never touch the tables directly.

mod db;
mod error;
mod markdown;
mod notebooks;
mod notes;
mod sync;
mod tags;

pub use db::Engine;
pub use error::{EngineError, Result};
pub use notebooks::Notebook;
pub use notes::Note;
pub use tags::Tag;

/// Markdown extraction (headings, links, Kanso references, tasks).
pub mod md {
    pub use crate::markdown::{Extracted, RefKind, Reference, Task, extract};
}

#[cfg(test)]
mod tests {
    use crate::Engine;

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

        // FTS finds the note by body content.
        let hits = engine.search_notes("outbox").await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, note.id);

        // Updating snapshots a revision and re-indexes.
        engine
            .update_note_body(&note.id, "# Sync v2\n\nfully rewritten body")
            .await
            .unwrap();
        assert_eq!(engine.search_notes("outbox").await.unwrap().len(), 0);
        assert_eq!(engine.search_notes("rewritten").await.unwrap().len(), 1);

        // Notebook-create + note-create + note-update = 3 outbox events.
        assert_eq!(engine.pending_outbox_count().await.unwrap(), 3);

        let ops = engine.get_pending_sync_ops(10).await.unwrap();
        assert_eq!(ops.len(), 3);
        // Sequences are monotonic and ordered.
        assert!(ops.windows(2).all(|w| w[0].local_sequence < w[1].local_sequence));

        // Acknowledging clears them from pending.
        let ids: Vec<String> = ops.iter().map(|o| o.id.to_string()).collect();
        engine.mark_sync_ops_acknowledged(&ids).await.unwrap();
        assert_eq!(engine.pending_outbox_count().await.unwrap(), 0);
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
}
