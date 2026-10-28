//! The thin Flutter bridge for publishing a note or notebook site.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    Library, NotePublishInfo, NotebookPublishInfo, PaletteDto, PreviewOptions, PublishError,
    Publisher,
};
use kanso_sync::share::{Expiry, KansoPublish};

/// Expiry offered by the shared publishing engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishExpiry {
    Never,
    Day,
    Week,
    Month,
}

impl From<PublishExpiry> for Expiry {
    fn from(value: PublishExpiry) -> Self {
        match value {
            PublishExpiry::Never => Expiry::Never,
            PublishExpiry::Day => Expiry::Day,
            PublishExpiry::Week => Expiry::Week,
            PublishExpiry::Month => Expiry::Month,
        }
    }
}

/// The engine's expiry choice and its localized-ready source label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishExpiryOption {
    pub expiry: PublishExpiry,
    pub label: String,
}

/// A note's published state, lock refusal and page content for the sheet.
#[derive(Clone, Debug, PartialEq)]
pub struct PublishNoteInfo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub rev: String,
    pub published: Option<PublishLink>,
    pub needs_update: bool,
    pub refusal: Option<String>,
}

/// A notebook's site plan summary.
#[derive(Clone, Debug, PartialEq)]
pub struct PublishNotebookInfo {
    pub id: String,
    pub name: String,
    pub published: Option<PublishLink>,
    pub needs_update: bool,
    pub contents: String,
    pub locked: String,
    pub refusal: Option<String>,
    pub note_count: u32,
    pub file_count: u32,
}

/// A public URL with the key fragment, expiry and last upload time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishLink {
    pub url: String,
    pub expires_at_ms: Option<i64>,
    pub expiry: PublishExpiry,
    pub published_at_ms: i64,
}

#[frb(opaque)]
pub struct KansoPublisher(Publisher);

impl KansoPublisher {
    pub(super) fn new(library: Library) -> Self {
        Self(library.publisher())
    }

    pub fn expiry_options() -> Vec<PublishExpiryOption> {
        kanso_mobile::expiry_options()
            .into_iter()
            .map(|(expiry, label)| PublishExpiryOption {
                expiry: match expiry {
                    Expiry::Never => PublishExpiry::Never,
                    Expiry::Day => PublishExpiry::Day,
                    Expiry::Week => PublishExpiry::Week,
                    Expiry::Month => PublishExpiry::Month,
                },
                label,
            })
            .collect()
    }

    pub fn note(&self, id: String) -> Result<PublishNoteInfo, String> {
        let info = self.0.note(&id).map_err(display_error)?;
        Ok(note_info(info, &self.0))
    }

    pub fn notebook(&self, id: String) -> Result<PublishNotebookInfo, String> {
        let plan = self.0.notebook(&id).map_err(display_error)?;
        Ok(notebook_info(plan.info, &self.0))
    }

    pub fn publish_note(
        &self,
        id: String,
        expiry: PublishExpiry,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<PublishLink, String> {
        let publish = self
            .0
            .publish_note(&id, expiry.into(), palette, options)
            .map_err(display_error)?;
        link_info(publish, &self.0).map_err(display_error)
    }

    pub fn unpublish_note(&self, id: String) -> Result<(), String> {
        self.0.unpublish_note(&id).map_err(display_error)
    }

    pub fn publish_notebook(
        &self,
        id: String,
        expiry: PublishExpiry,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<PublishLink, String> {
        let publish = self
            .0
            .publish_notebook(&id, expiry.into(), palette, options)
            .map_err(display_error)?;
        link_info(publish, &self.0).map_err(display_error)
    }

    pub fn unpublish_notebook(&self, id: String) -> Result<(), String> {
        self.0.unpublish_notebook(&id).map_err(display_error)
    }
}

fn note_info(info: NotePublishInfo, publisher: &Publisher) -> PublishNoteInfo {
    PublishNoteInfo {
        id: info.id,
        title: info.title,
        body: info.body,
        rev: info.rev,
        published: info
            .publish
            .and_then(|publish| link_info(publish, publisher).ok()),
        needs_update: info.needs_update,
        refusal: info.refusal,
    }
}

fn notebook_info(info: NotebookPublishInfo, publisher: &Publisher) -> PublishNotebookInfo {
    PublishNotebookInfo {
        id: info.id,
        name: info.name,
        published: info
            .publish
            .and_then(|publish| link_info(publish, publisher).ok()),
        needs_update: info.needs_update,
        contents: info.contents,
        locked: info.locked,
        refusal: info.refusal,
        note_count: info.note_count,
        file_count: info.file_count,
    }
}

fn link_info(publish: KansoPublish, publisher: &Publisher) -> Result<PublishLink, PublishError> {
    Ok(PublishLink {
        url: publisher.link(&publish)?,
        expires_at_ms: publish.expires_at,
        expiry: match kanso_mobile::expiry_choice(&publish) {
            Expiry::Never => PublishExpiry::Never,
            Expiry::Day => PublishExpiry::Day,
            Expiry::Week => PublishExpiry::Week,
            Expiry::Month => PublishExpiry::Month,
        },
        published_at_ms: publish.published_at,
    })
}

fn display_error(error: PublishError) -> String {
    error.to_string()
}
