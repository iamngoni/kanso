//! What the phone's system actions (iOS App Intents) read from the library.

/// A note "Open Kanso note" offers in Shortcuts, Siri and Spotlight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntentNote {
    pub id: String,
    pub title: String,
    /// The notebook's name, shown under the title.
    pub notebook: String,
}
