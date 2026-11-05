//! What a canvas embed is made of and what it becomes. Only shapes live
//! here; the rules are in [`super::embed`].

use std::sync::Arc;

use crate::Svg;

/// Which part of a canvas an embed shows, and how wide: what a
/// [`Kind::Canvas`](crate::Kind::Canvas) job is drawn with besides the
/// canvas itself.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spec {
    /// The canvas note's title, for the card an empty canvas draws.
    pub title: String,
    /// A `group` node to show instead of the content.
    pub group: Option<String>,
    /// The width to show it at, in points.
    pub width: Option<u32>,
}

/// Something a reader of an embed is told, beside the drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Notice {
    /// The embed draws the whole canvas and it is large: over 8,000 points
    /// across, or an SVG of over 2 MB. It still draws.
    WholeCanvas,
    /// The embed names a group the canvas no longer has, so it draws the
    /// content instead.
    GroupMissing,
}

impl Notice {
    /// The words for the reader: the preview's banner and the export's note.
    pub fn text(self) -> &'static str {
        match self {
            Notice::WholeCanvas => "This embeds the whole canvas. Name a group to embed less.",
            Notice::GroupMissing => "That group is gone, so this shows the whole canvas.",
        }
    }
}

/// What the notes know of the canvas an embed names; see [`Notes`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmbedNote {
    /// The note is locked (or in a locked notebook). It carries no body:
    /// an embed of it never reads one.
    Locked,
    /// The note isn't a canvas.
    NotCanvas,
    /// A canvas note: its title and its body (the fenced JSON).
    Open { title: String, body: Arc<str> },
}

/// Finds the note an embed names by its ID. `None` is a note that doesn't
/// exist (or was deleted). `kanso-render` never touches the store itself.
pub type Notes<'a> = &'a dyn Fn(&str) -> Option<EmbedNote>;

/// What an embed shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Shown {
    /// The canvas, drawn; `notices` says what to add beside it.
    Drawing(Svg),
    /// A lock and "Locked canvas": nothing of the canvas is read or drawn.
    Locked,
    /// There is no such note any more.
    Missing,
    /// The note is not a canvas.
    NotCanvas,
    /// The canvas can't be drawn; the reason.
    Invalid(String),
}
