//! The mobile Publish sheet's plain values, separate from its store and
//! session runtime. Links remain in the note or notebook's encrypted extra.

use kanso_engine::publish::site::SitePlan;
use kanso_sync::share::{Expiry, KansoPublish};

/// A typed failure at the publish boundary. The signed-out case stays
/// distinguishable so the app can show its sign-in action.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PublishError {
    #[error("sign in to publish")]
    SignedOut,
    #[error("{0}")]
    Refused(String),
    #[error("{0}")]
    Failed(String),
}

/// What the note sheet needs to describe and act on.
#[derive(Clone, Debug, PartialEq)]
pub struct NotePublishInfo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub rev: String,
    pub publish: Option<KansoPublish>,
    pub needs_update: bool,
    pub refusal: Option<String>,
}

/// What the notebook sheet needs to describe and act on.
#[derive(Clone, Debug, PartialEq)]
pub struct NotebookPublishInfo {
    pub id: String,
    pub name: String,
    pub publish: Option<KansoPublish>,
    pub needs_update: bool,
    pub contents: String,
    pub locked: String,
    pub refusal: Option<String>,
    pub note_count: u32,
    pub file_count: u32,
}

/// A snapshot of the engine plan for tests and the native bridge.
#[derive(Clone, Debug, PartialEq)]
pub struct NotebookPlan {
    pub info: NotebookPublishInfo,
    pub(crate) plan: SitePlan,
}

/// The expiry options are the engine's own choices, with stable labels.
pub fn expiry_options() -> Vec<(Expiry, String)> {
    Expiry::ALL
        .into_iter()
        .map(|expiry| (expiry, expiry.label().to_owned()))
        .collect()
}

/// The engine's closest matching expiry for a previously published link.
pub fn expiry_choice(publish: &KansoPublish) -> Expiry {
    kanso_engine::publish::expiry_choice(publish)
}
