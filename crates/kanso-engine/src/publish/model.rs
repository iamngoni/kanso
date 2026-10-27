//! What the Publish sheet works on: plain data.

use kanso_sync::share::KansoPublish;

/// What the sheet publishes: the open note as typed.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub body: String,
    pub rev: String,
    pub publish: Option<KansoPublish>,
}

/// What the sheet shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Publishing needs an account.
    SignedOut,
    Unpublished,
    Published,
}

/// A task the sheet is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Busy {
    Publishing,
    Updating,
    Unpublishing,
}
