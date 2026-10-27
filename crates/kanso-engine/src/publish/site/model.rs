//! What publishing a notebook would put on the web, and why it can fail:
//! plain data.

use kanso_sync::share::{KansoPublish, ShareError};
use kanso_sync::site::{SiteError, fingerprint};

/// A note the site will show.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlannedNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub snippet: String,
    pub slug: String,
    /// The notebook inside the published one it is in, as "Plans › 2026";
    /// `None` for the published notebook's own notes.
    pub group: Option<String>,
}

/// An attachment a published note uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedFile {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

impl PlannedFile {
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }
}

/// What publishing a notebook would put on the web, before anything is
/// rendered: for the sheet to show, and to build from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SitePlan {
    pub book_id: String,
    pub name: String,
    pub notes: Vec<PlannedNote>,
    /// The titles of the locked notes left out.
    pub locked: Vec<String>,
    pub files: Vec<PlannedFile>,
    /// The notebook's link, when it is published.
    pub publish: Option<KansoPublish>,
}

impl SitePlan {
    /// What the site is made from, for telling whether it changed since it
    /// was published (`KansoPublish::is_outdated`, with the name as title).
    pub fn fingerprint(&self) -> String {
        fingerprint(
            self.notes
                .iter()
                .map(|n| (n.id.as_str(), n.title.as_str(), n.body.as_str())),
        )
    }

    /// Whether `publish` shows something other than what the notebook
    /// holds now.
    pub fn is_outdated(&self, publish: &KansoPublish) -> bool {
        publish.is_outdated(&self.name, &self.fingerprint())
    }
}

/// Why a notebook couldn't be published.
#[derive(Debug)]
pub enum PublishError {
    /// The site came out inconsistent: a bug, never the notebook's fault.
    Site(SiteError),
    Share(ShareError),
}

impl From<SiteError> for PublishError {
    fn from(err: SiteError) -> PublishError {
        PublishError::Site(err)
    }
}

impl From<ShareError> for PublishError {
    fn from(err: ShareError) -> PublishError {
        PublishError::Share(err)
    }
}
