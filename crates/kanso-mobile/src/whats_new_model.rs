//! What the phone's What's new sheet is handed: the release's video (when
//! it has one and it passed its checks), its title line, a one-line summary
//! and its notes as plain lists, with the releases skipped since the last
//! update after them.

/// One section of a release's notes: "New", "Improved", "Fixed".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhoneWhatsNewSection {
    pub title: String,
    pub items: Vec<String>,
}

/// A release skipped since the last update.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhoneWhatsNewEarlier {
    pub version: String,
    pub sections: Vec<PhoneWhatsNewSection>,
}

/// The sheet, once, after an update.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhoneWhatsNew {
    pub version: String,
    /// `YYYY-MM-DD`, or empty.
    pub date: String,
    /// "2 new features, 1 improvement and 3 fixes.", or empty.
    pub summary: String,
    /// The checked, cached animated WebP; empty without one.
    pub video_path: String,
    pub sections: Vec<PhoneWhatsNewSection>,
    /// Newest first; empty when nothing was skipped.
    pub earlier: Vec<PhoneWhatsNewEarlier>,
    /// The release's page on GitHub.
    pub release_url: String,
}
