//! The data of a generated page: its sections, their lines and where each
//! line came from. No logic beyond small accessors.

use crate::ask::Receipt;

/// How a section is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    /// Built on open from the derived tables, with no AI. Never stored.
    Computed,
    /// Written by AI from the lines that mention the subject, with receipts,
    /// and cached on the page's own document.
    Ai,
}

impl SectionKind {
    /// The small tag at a section's right edge.
    pub fn tag(self) -> &'static str {
        match self {
            SectionKind::Computed => "computed",
            SectionKind::Ai => "AI",
        }
    }
}

/// A section a page kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionSpec {
    /// "open_items", "promises", "about", ...
    pub key: &'static str,
    /// "Open items".
    pub title: &'static str,
    pub kind: SectionKind,
}

/// A task on a line a page draws on, so a section can tick it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRef {
    pub note_id: String,
    pub line: u32,
    /// The words after the box, as written.
    pub text: String,
    pub checked: bool,
}

/// One line of a section.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub text: String,
    /// "you" or "them" in a promise.
    pub who: Option<String>,
    /// The lines it rests on: one link each. A computed line has one.
    pub sources: Vec<Receipt>,
    /// Muted text at the end of the line ("Weekly sync · 28 Oct").
    pub meta: Option<String>,
    /// A group heading the line sits under ("October 2026").
    pub group: Option<String>,
    /// A task the line can tick.
    pub task: Option<TaskRef>,
    /// A note the line opens (related notes and pages).
    pub target: Option<String>,
    /// "person" or "project" for a related page.
    pub tag: Option<String>,
    /// The source is a note others can edit.
    pub shared: bool,
}

/// Why a section shows no lines of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffReason {
    /// AI is off and nothing was ever written.
    AiOff,
}

impl OffReason {
    /// What the section says instead.
    pub fn text(self) -> &'static str {
        match self {
            OffReason::AiOff => AI_OFF_LINE,
        }
    }
}

/// The line an AI section shows while AI is off.
pub const AI_OFF_LINE: &str =
    "Turn on AI in Settings to have Kanso summarise what you discussed and promised.";
/// What an AI section says when the notes held nothing for it.
pub const NOTHING_FOUND: &str = "Nothing found in your notes.";
/// What a locked page says in place of its live sections.
pub const LOCKED_PAGE: &str = "Live sections are off for locked pages.";

/// Where a section stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionState {
    /// Computed, or written from the notes as they are now.
    Fresh,
    /// Written from notes that have since changed: `changed` of them.
    Stale { changed: usize },
    /// Kept by the person: never rewritten on its own.
    Pinned { at: i64, changed: usize },
    /// Never written yet.
    Unwritten,
    /// Shows a line instead of lines.
    Off(OffReason),
}

/// One section of an open page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub key: String,
    pub title: String,
    pub kind: SectionKind,
    pub items: Vec<Item>,
    pub state: SectionState,
    /// When an AI section was written (ms since the epoch).
    pub written_at: Option<i64>,
    /// How many lines there are in all when `items` holds the first few.
    pub total: usize,
    /// A line under the title ("Based on the 60 most recent mentions.").
    pub note: Option<String>,
    /// The model had nothing to say: `items` is empty and that is the answer.
    pub none: bool,
}

impl Section {
    /// A computed section with these lines.
    pub fn computed(spec: &SectionSpec, items: Vec<Item>, total: usize) -> Section {
        Section {
            key: spec.key.to_owned(),
            title: spec.title.to_owned(),
            kind: SectionKind::Computed,
            items,
            state: SectionState::Fresh,
            written_at: None,
            total,
            note: None,
            none: false,
        }
    }

    /// Whether the section can be refreshed by pressing Refresh.
    pub fn refreshable(&self) -> bool {
        self.kind == SectionKind::Ai
    }

    /// Whether the section is out of date or was never written, and not kept.
    pub fn wants_refresh(&self) -> bool {
        matches!(
            self.state,
            SectionState::Stale { .. } | SectionState::Unwritten
        )
    }

    /// "Out of date · 3 notes changed" or "Sources changed since".
    pub fn stale_line(&self) -> Option<String> {
        match self.state {
            SectionState::Stale { changed } => Some(format!(
                "Out of date \u{b7} {changed} {} changed",
                if changed == 1 { "note" } else { "notes" }
            )),
            SectionState::Pinned { changed, .. } if changed > 0 => {
                Some("Sources changed since".to_owned())
            }
            _ => None,
        }
    }
}

/// An open page: its sections in order, and which ones are hidden.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageView {
    pub sections: Vec<Section>,
    /// `(key, title)` of sections hidden for this page.
    pub hidden: Vec<(String, String)>,
    /// The page's own notes were too many to read: how many mentions its AI
    /// sections were based on.
    pub based_on: Option<u32>,
}

/// A note a page draws on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceNote {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    /// The note may be sent to AI.
    pub ai_ok: bool,
}

/// When AI sections are rewritten without being asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RefreshPolicy {
    /// When a stale or unwritten page opens, at most once an hour.
    OnOpen,
    /// Only when Refresh is pressed.
    #[default]
    OnPress,
}

/// What the person sees when a section's lines are all gone.
pub fn empty_line(section: &Section) -> Option<&'static str> {
    match section.state {
        SectionState::Off(reason) => Some(reason.text()),
        _ if section.kind == SectionKind::Ai && section.items.is_empty() && section.none => {
            Some(NOTHING_FOUND)
        }
        _ => None,
    }
}
