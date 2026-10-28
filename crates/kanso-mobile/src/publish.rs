//! Publishing for an open mobile library. The account session stays in the
//! secret store; only the public link and engine-owned summary cross the edge.

use std::sync::Arc;

use kanso_core::now_ms;
use kanso_engine::export::PageStyle;
use kanso_engine::publish::{self, Source, site};
use kanso_render::Palette;
use kanso_sync::Session;
use kanso_sync::share::{self, Expiry, HttpShares, KansoPublish, ShareApi};

use crate::library::{Inner, Library};
use crate::model::{PaletteDto, PreviewOptions};
use crate::publish_model::{NotePublishInfo, NotebookPlan, NotebookPublishInfo, PublishError};

/// Publish operations scoped to one open library.
#[derive(Clone)]
pub struct Publisher {
    inner: Arc<Inner>,
}

impl Library {
    /// Published links live with this library's encrypted notes and notebooks.
    pub fn publisher(&self) -> Publisher {
        Publisher {
            inner: self.inner.clone(),
        }
    }
}

impl Publisher {
    /// Resolves the owner-only link with its key for copying or sharing.
    pub fn link(&self, publish: &KansoPublish) -> Result<String, PublishError> {
        let session = self.session()?;
        Ok(publish.link(&session.server))
    }

    /// Reads a note's current link and the engine's lock refusal.
    pub fn note(&self, id: &str) -> Result<NotePublishInfo, PublishError> {
        let note = self
            .inner
            .store
            .note(id)
            .map_err(store_error)?
            .ok_or_else(|| PublishError::Failed("Note not found.".into()))?;
        let in_locked = self
            .inner
            .store
            .note_in_locked_book(id)
            .map_err(store_error)?;
        let refusal = publish::refusal(note.is_locked(), in_locked).map(str::to_owned);
        let current = KansoPublish::from_extra(&note.extra);
        let needs_update = current.as_ref().is_some_and(|current| {
            publish::needs_update(
                current,
                &note.title,
                &note.body,
                publish::expiry_choice(current),
                now_ms(),
            )
        });
        Ok(NotePublishInfo {
            id: note.id,
            title: note.title,
            body: note.body,
            rev: note.rev.unwrap_or_default(),
            publish: current,
            needs_update,
            refusal,
        })
    }

    /// Reads the notebook's current link and the exact site plan the engine
    /// will build, including locked notes and attachment limits.
    pub fn notebook(&self, id: &str) -> Result<NotebookPlan, PublishError> {
        let plan = site::plan(&self.inner.store, id)
            .map_err(|err| PublishError::Failed(err.to_string()))?;
        let books = self.inner.store.books().map_err(store_error)?;
        let book = books
            .iter()
            .find(|book| book.id == id)
            .ok_or_else(|| PublishError::Failed("Notebook not found.".into()))?;
        let contents = site::contents_line(&plan);
        let locked = site::locked_line(&plan);
        let needs_update = plan
            .publish
            .as_ref()
            .is_some_and(|current| plan.is_outdated(current) || current.is_expired(now_ms()));
        let refusal = book.is_locked().then(|| {
            publish::refusal(false, true)
                .unwrap_or("This notebook can't be published.")
                .to_owned()
        });
        let info = NotebookPublishInfo {
            id: plan.book_id.clone(),
            name: plan.name.clone(),
            publish: plan.publish.clone(),
            needs_update,
            contents,
            locked,
            refusal,
            note_count: plan.notes.len() as u32,
            file_count: plan.files.len() as u32,
        };
        Ok(NotebookPlan { info, plan })
    }

    /// Publishes or updates a note and stores its share metadata on that note.
    pub fn publish_note(
        &self,
        id: &str,
        expiry: Expiry,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<KansoPublish, PublishError> {
        let api = HttpShares;
        let palette = crate::renderer::palette_of(palette);
        let style = page_style(options);
        self.publish_note_with(&api, id, expiry, &palette, &style, now_ms())
    }

    /// Uses an injected API so tests never perform network requests.
    pub fn publish_note_with(
        &self,
        api: &dyn ShareApi,
        id: &str,
        expiry: Expiry,
        palette: &Palette,
        style: &PageStyle,
        at: i64,
    ) -> Result<KansoPublish, PublishError> {
        let session = self.session()?;
        publish_note_with(
            api,
            &self.inner.store,
            &session,
            PublishRequest {
                id,
                expiry,
                palette,
                style,
                at,
            },
        )
    }

    /// Removes a note's public link, then removes the metadata from the note.
    pub fn unpublish_note(&self, id: &str) -> Result<(), PublishError> {
        self.unpublish_note_with(&HttpShares, id)
    }

    /// Test seam for the share server.
    pub fn unpublish_note_with(&self, api: &dyn ShareApi, id: &str) -> Result<(), PublishError> {
        let session = self.session()?;
        let note = self
            .inner
            .store
            .note(id)
            .map_err(store_error)?
            .ok_or_else(|| PublishError::Failed("Note not found.".into()))?;
        if let Some(current) = KansoPublish::from_extra(&note.extra) {
            share::unpublish(api, &session, &current)
                .map_err(|err| PublishError::Failed(publish::error_text(&err)))?;
        }
        publish::store_publish(&self.inner.store, id, None)
            .map_err(|err| PublishError::Failed(err.to_string()))
    }

    /// Publishes or updates the notebook site, keeping the link on the book.
    pub fn publish_notebook(
        &self,
        id: &str,
        expiry: Expiry,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<KansoPublish, PublishError> {
        let palette = crate::renderer::palette_of(palette);
        let style = page_style(options);
        self.publish_notebook_with(&HttpShares, id, expiry, &palette, &style, now_ms())
    }

    /// Uses an injected API so tests never perform network requests.
    pub fn publish_notebook_with(
        &self,
        api: &dyn ShareApi,
        id: &str,
        expiry: Expiry,
        palette: &Palette,
        style: &PageStyle,
        at: i64,
    ) -> Result<KansoPublish, PublishError> {
        let session = self.session()?;
        publish_notebook_with(
            api,
            &self.inner.store,
            &session,
            PublishRequest {
                id,
                expiry,
                palette,
                style,
                at,
            },
        )
    }

    /// Removes the site and its metadata from the notebook.
    pub fn unpublish_notebook(&self, id: &str) -> Result<(), PublishError> {
        self.unpublish_notebook_with(&HttpShares, id)
    }

    /// Test seam for the share server.
    pub fn unpublish_notebook_with(
        &self,
        api: &dyn ShareApi,
        id: &str,
    ) -> Result<(), PublishError> {
        let session = self.session()?;
        let book = self
            .inner
            .store
            .books()
            .map_err(store_error)?
            .into_iter()
            .find(|book| book.id == id)
            .ok_or_else(|| PublishError::Failed("Notebook not found.".into()))?;
        if let Some(current) = site::book_publish(&book) {
            share::unpublish(api, &session, &current)
                .map_err(|err| PublishError::Failed(publish::error_text(&err)))?;
        }
        site::store_book_publish(&self.inner.store, id, None)
            .map_err(|err| PublishError::Failed(err.to_string()))
    }

    fn session(&self) -> Result<Session, PublishError> {
        crate::syncing::SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()
        .map_err(|err| PublishError::Failed(err.to_string()))?
        .ok_or(PublishError::SignedOut)
    }
}

struct PublishRequest<'a> {
    id: &'a str,
    expiry: Expiry,
    palette: &'a Palette,
    style: &'a PageStyle,
    at: i64,
}

fn publish_note_with(
    api: &dyn ShareApi,
    store: &kanso_store::Store,
    session: &Session,
    request: PublishRequest<'_>,
) -> Result<KansoPublish, PublishError> {
    let PublishRequest {
        id,
        expiry,
        palette,
        style,
        at,
    } = request;
    let note = store
        .note(id)
        .map_err(store_error)?
        .ok_or_else(|| PublishError::Failed("Note not found.".into()))?;
    let in_locked = store.note_in_locked_book(id).map_err(store_error)?;
    if let Some(reason) = publish::refusal(note.is_locked(), in_locked) {
        return Err(PublishError::Refused(reason.into()));
    }
    let source = Source {
        id: note.id.clone(),
        title: note.title,
        body: note.body,
        rev: note.rev.unwrap_or_default(),
        publish: KansoPublish::from_extra(&note.extra),
    };
    let result = publish::publish_note(
        api,
        session,
        store,
        &source,
        palette,
        style,
        expiry.at(at),
        at,
    )
    .map_err(|err| PublishError::Failed(publish::error_text(&err)))?;
    publish::store_publish(store, id, Some(&result))
        .map_err(|err| PublishError::Failed(err.to_string()))?;
    Ok(result)
}

fn publish_notebook_with(
    api: &dyn ShareApi,
    store: &kanso_store::Store,
    session: &Session,
    request: PublishRequest<'_>,
) -> Result<KansoPublish, PublishError> {
    let PublishRequest {
        id,
        expiry,
        palette,
        style,
        at,
    } = request;
    let plan = site::plan(store, id).map_err(|err| PublishError::Failed(err.to_string()))?;
    let book = store
        .books()
        .map_err(store_error)?
        .into_iter()
        .find(|book| book.id == id)
        .ok_or_else(|| PublishError::Failed("Notebook not found.".into()))?;
    if book.is_locked() {
        let reason = publish::refusal(false, true).unwrap_or("This notebook can't be published.");
        return Err(PublishError::Refused(reason.into()));
    }
    let result = site::publish(
        api,
        session,
        &plan,
        store,
        palette,
        style,
        expiry.at(at),
        at,
    )
    .map_err(|err| PublishError::Failed(site::error_text(&err)))?;
    site::store_book_publish(store, id, Some(&result))
        .map_err(|err| PublishError::Failed(err.to_string()))?;
    Ok(result)
}

fn page_style(options: PreviewOptions) -> PageStyle {
    PageStyle {
        font: options.font,
        mono: options.mono,
        size: options.size,
        // Published pages use the selected palette and faces; syntax colors
        // are intentionally absent until the editor theme is available here.
        code: None,
    }
}

fn store_error(error: impl std::fmt::Display) -> PublishError {
    PublishError::Failed(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::{PublishRequest, publish_note_with, publish_notebook_with};
    use crate::publish_model::PublishError;
    use kanso_core::{DocType, links::FILE_LINK_PREFIX};
    use kanso_engine::export::PageStyle;
    use kanso_engine::publish::{self, site};
    use kanso_render::Palette;
    use kanso_store::{Origin, Put, Store};
    use kanso_sync::ApiError;
    use kanso_sync::Session;
    use kanso_sync::share::{Expiry, ShareApi, Uploaded};
    use serde_json::json;

    #[derive(Default)]
    struct FakeShares {
        uploaded: Mutex<HashMap<String, Vec<u8>>>,
        puts: Mutex<Vec<String>>,
        deletes: Mutex<Vec<String>>,
    }

    impl ShareApi for FakeShares {
        fn put_share(
            &self,
            _session: &Session,
            share_id: &str,
            sealed: &[u8],
            _title: &str,
            _expires_at: Option<i64>,
        ) -> Result<Uploaded, ApiError> {
            self.uploaded
                .lock()
                .unwrap()
                .insert(share_id.into(), sealed.into());
            self.puts.lock().unwrap().push(share_id.into());
            Ok(Uploaded::default())
        }

        fn delete_share(&self, _session: &Session, share_id: &str) -> Result<(), ApiError> {
            self.uploaded.lock().unwrap().remove(share_id);
            self.deletes.lock().unwrap().push(share_id.into());
            Ok(())
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("publish.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:test", "name": "Test"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        (dir, store)
    }

    fn put(store: &Store, kind: DocType, value: serde_json::Value) {
        store.put(kind, value, Put::Upsert, Origin::Ui).unwrap();
    }

    fn note(store: &Store, id: &str, title: &str, body: &str, book: &str) {
        put(
            store,
            DocType::Note,
            json!({"_id": id, "title": title, "body": body, "bookId": book}),
        );
    }

    fn session() -> &'static Session {
        crate::syncing::test_session()
    }

    #[test]
    fn signed_out_publish_is_a_typed_error_before_any_upload() {
        use std::sync::Arc;

        use crate::Library;
        use kanso_sync::{MemorySecrets, MemoryServer};

        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        let error = library
            .publisher()
            .publish_note(
                "note:missing",
                Expiry::Never,
                crate::model::PaletteDto {
                    dark: false,
                    background: 0,
                    text: 0,
                    muted: 0,
                    accent: 0,
                    border: 0,
                    surface: 0,
                    heading: None,
                    quote: 0,
                    task_done: 0,
                    code_text: None,
                    inline_code_background: 0,
                    inline_code_text: None,
                    table_header_background: None,
                },
                crate::model::PreviewOptions::default(),
            )
            .unwrap_err();
        assert_eq!(error, PublishError::SignedOut);
    }

    #[test]
    fn first_publish_update_unpublish_and_expired_link_replacement_use_fake_api() {
        let (_dir, store) = store();
        note(&store, "note:a", "Before", "First body", "book:test");
        let shares = FakeShares::default();
        let first = publish_note_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "note:a",
                expiry: Expiry::Never,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 10_000,
            },
        )
        .unwrap();
        let stored = store.note("note:a").unwrap().unwrap();
        assert_eq!(publish::refusal(false, false), None);
        assert_eq!(
            kanso_sync::share::KansoPublish::from_extra(&stored.extra),
            Some(first.clone())
        );

        let mut changed = store.note("note:a").unwrap().unwrap();
        changed.body = "Updated body".into();
        put(
            &store,
            DocType::Note,
            serde_json::to_value(changed).unwrap(),
        );
        let updated = publish_note_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "note:a",
                expiry: Expiry::Never,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 20_000,
            },
        )
        .unwrap();
        assert_eq!(
            updated.share_id, first.share_id,
            "a live link updates in place"
        );
        assert_eq!(
            shares.puts.lock().unwrap().as_slice(),
            [first.share_id.as_str(), first.share_id.as_str()]
        );

        kanso_sync::share::unpublish(&shares, session(), &updated).unwrap();
        publish::store_publish(&store, "note:a", None).unwrap();
        assert!(
            kanso_sync::share::KansoPublish::from_extra(
                &store.note("note:a").unwrap().unwrap().extra
            )
            .is_none()
        );
        assert_eq!(
            shares.deletes.lock().unwrap().as_slice(),
            [first.share_id.as_str()]
        );

        let expiring = publish_note_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "note:a",
                expiry: Expiry::Day,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 30_000,
            },
        )
        .unwrap();
        let replacement = publish_note_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "note:a",
                expiry: Expiry::Day,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 86_430_001,
            },
        )
        .unwrap();
        assert_ne!(
            replacement.share_id, expiring.share_id,
            "expired links get a new ID and key"
        );
    }

    #[test]
    fn locked_note_refusal_is_engine_wording_and_not_uploaded() {
        let (_dir, store) = store();
        note(&store, "note:secret", "Secret", "", "book:test");
        let mut secret = store.note("note:secret").unwrap().unwrap();
        secret
            .extra
            .insert(kanso_core::model::LOCK_FIELD.into(), json!({"v": 1}));
        store
            .lock_note(serde_json::to_value(secret).unwrap())
            .unwrap();
        let shares = FakeShares::default();
        let error = publish_note_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "note:secret",
                expiry: Expiry::Never,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 10_000,
            },
        )
        .unwrap_err();
        assert_eq!(
            error,
            PublishError::Refused(publish::refusal(true, false).unwrap().to_owned())
        );
        assert!(shares.puts.lock().unwrap().is_empty());
    }

    #[test]
    fn notebook_plan_counts_attachments_and_build_rejects_over_limit() {
        use base64::Engine as _;

        let (_dir, store) = store();
        put(
            &store,
            DocType::Book,
            json!({"_id": "book:work", "name": "Work"}),
        );
        let attachment = base64::engine::general_purpose::STANDARD.encode(vec![0; 6 * 1024 * 1024]);
        put(
            &store,
            DocType::File,
            json!({"_id": "file:image", "name": "image.png", "contentType": "image/png",
                "_attachments": {"index": {"content_type": "image/png", "data": attachment}}}),
        );
        let attachment = base64::engine::general_purpose::STANDARD.encode(vec![1; 6 * 1024 * 1024]);
        put(
            &store,
            DocType::File,
            json!({"_id": "file:second", "name": "second.pdf", "contentType": "application/pdf",
                "_attachments": {"index": {"content_type": "application/pdf", "data": attachment}}}),
        );
        note(
            &store,
            "note:plan",
            "Plan",
            &format!("![diagram]({FILE_LINK_PREFIX}image)\n\n[Download]({FILE_LINK_PREFIX}second)"),
            "book:work",
        );
        let plan = site::plan(&store, "book:work").unwrap();
        assert_eq!(plan.notes.len(), 1);
        assert_eq!(plan.files.len(), 2);
        assert_eq!(site::contents_line(&plan), "1 note, 1 image and 1 file.");
        let shares = FakeShares::default();
        let success = publish_notebook_with(
            &shares,
            &store,
            session(),
            PublishRequest {
                id: "book:work",
                expiry: Expiry::Never,
                palette: &Palette::LIGHT,
                style: &PageStyle::default(),
                at: 10_000,
            },
        );
        let error = match success {
            Err(error) => error,
            Ok(_) => panic!("an oversized site unexpectedly uploaded"),
        };
        assert!(error.to_string().contains("over 10 MB"));
        assert!(shares.puts.lock().unwrap().is_empty());
    }
}
