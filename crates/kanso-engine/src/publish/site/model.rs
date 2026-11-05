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

    /// Whether the file is audio, such as a recording: left out of a
    /// published notebook unless the owner opts in (issue #38, step 11).
    pub fn is_audio(&self) -> bool {
        crate::recording_privacy::is_audio_type(&self.content_type)
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
    /// The attachments the site carries. Audio is among them only when
    /// `include_audio` is set.
    pub files: Vec<PlannedFile>,
    /// The audio the notebook's notes link, whether or not the site carries
    /// it: the sheet offers its toggle when there is some.
    pub audio: Vec<PlannedFile>,
    /// The owner chose to publish the audio. Off unless asked for, every
    /// time: a recording is private, and a long one would not fit the
    /// bundle anyway.
    pub include_audio: bool,
    /// The canvases the published notes embed, by note ID, as they were
    /// read when planned (a locked one is [`EmbedNote::Locked`], with no
    /// body). A canvas that isn't there is not in the map.
    pub canvases: std::collections::HashMap<String, kanso_render::canvas::embed::EmbedNote>,
    /// The notebook's link, when it is published.
    pub publish: Option<KansoPublish>,
}

impl SitePlan {
    /// What the site is made from, for telling whether it changed since it
    /// was published (`KansoPublish::is_outdated`, with the name as title).
    /// Publishing the audio changes it, so toggling either way shows the
    /// published version as out of date.
    pub fn fingerprint(&self) -> String {
        let notes = fingerprint(
            self.notes
                .iter()
                .map(|n| (n.id.as_str(), n.title.as_str(), n.body.as_str())),
        );
        if self.include_audio && !self.audio.is_empty() {
            format!("{notes}+audio")
        } else {
            notes
        }
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
