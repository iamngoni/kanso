//! The plain data People & Projects hands the shell (issue #200): the list
//! of pages, the suggestions, an open page's live sections and the
//! settings. No logic lives here; `people.rs` builds them from the engine's
//! (`kanso_engine::entities`, `kanso_engine::pages`).

/// Whether a page is about a person or a project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeopleKind {
    Person,
    Project,
}

/// Settings › People & Projects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleSettings {
    /// Suggest people and projects.
    pub suggest: bool,
    /// The notebook new pages go in; empty is the one named
    /// "People & Projects", made on first use.
    pub keep_in: String,
    /// Write summaries with AI.
    pub write_with_ai: bool,
    /// When AI sections are rewritten: `auto` (what the provider suits),
    /// `open` (when a stale page opens) or `press` (only on Refresh).
    pub refresh: String,
    /// Use AI to find people and projects.
    pub detect_with_ai: bool,
}

/// What the library drawer row and the list header need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleStatus {
    pub pages: u32,
    pub suggestions: u32,
    /// "3 new", or empty.
    pub new_label: String,
    /// Suggestions are switched on.
    pub suggest_on: bool,
    /// AI is on and summaries are wanted.
    pub ai_on: bool,
}

/// A person's or project's page in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeoplePage {
    pub id: String,
    pub title: String,
    pub kind: PeopleKind,
    pub notes: u32,
    /// "14 notes · last 28 Oct".
    pub detail: String,
}

/// A name Kanso suggests a page for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleSuggestion {
    /// What dismissing it names.
    pub key: String,
    pub name: String,
    pub kind: PeopleKind,
    /// "person · mentioned in 14 notes, last on 28 Oct".
    pub detail: String,
    /// `Also written as "@sam".`, or empty.
    pub also_line: String,
    /// When this may be another name of an existing page: its ID and title.
    pub alias_of_id: String,
    pub alias_of_title: String,
}

/// The lines a sentence rests on, as they were sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSource {
    pub note_id: String,
    pub title: String,
    /// `2026-10-28`, or empty.
    pub updated: String,
    /// 0-based, inclusive.
    pub start: u32,
    pub end: u32,
    pub quote: String,
    /// "Weekly sync · line 12 · 28 Oct".
    pub label: String,
}

/// A task a line can tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageTask {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    pub checked: bool,
}

/// One line of a live section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageItem {
    pub text: String,
    /// `you` or `them` in a promise; empty otherwise.
    pub who: String,
    pub sources: Vec<PageSource>,
    /// Muted words with the line ("Weekly sync · 28 Oct", "28 Oct").
    pub meta: String,
    /// The month group a timeline line sits in.
    pub group: String,
    pub task: Option<PageTask>,
    /// A note the line opens.
    pub target: String,
    /// `person` or `project` for a related page; empty otherwise.
    pub tag: String,
    /// The line comes from a note others can edit.
    pub shared: bool,
}

/// Where a section stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSectionState {
    Fresh,
    /// Written from notes that have since changed.
    Stale,
    /// Kept by the person.
    Pinned,
    /// AI is on and the section was never written.
    Unwritten,
    /// Shows `empty_line` instead of lines.
    Off,
}

/// One live section of a page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSection {
    pub key: String,
    pub title: String,
    /// Written by AI (else computed).
    pub ai: bool,
    pub items: Vec<PageItem>,
    pub state: PageSectionState,
    /// Notes changed since an AI section was written.
    pub changed: u32,
    /// When an AI section was written, in milliseconds; 0 if never.
    pub written_ms: i64,
    /// Lines in all, when `items` holds the first few.
    pub total: u32,
    /// "Based on the 60 most recent mentions.", or empty.
    pub note: String,
    /// What the section says in place of lines, or empty.
    pub empty_line: String,
    /// "Out of date · 3 notes changed", "Sources changed since", or empty.
    pub stale_line: String,
}

/// A section hidden on this page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageHidden {
    pub key: String,
    pub title: String,
}

/// An open page's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageHeaderInfo {
    pub title: String,
    pub kind: PeopleKind,
    pub aliases: Vec<String>,
    pub notes: u32,
    pub last_mentioned: String,
    /// "Also known as Sam, @sam · 14 notes · last mentioned 28 Oct".
    pub line: String,
}

/// The live part of a person's or project's page, below the note's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagePanel {
    pub header: PageHeaderInfo,
    pub sections: Vec<PageSection>,
    pub hidden: Vec<PageHidden>,
    /// AI is on, so Refresh does something.
    pub ai_on: bool,
    /// Sections that want rewriting when the page opens, by the person's
    /// refresh setting.
    pub due: Vec<String>,
}

/// What a merge did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleMerged {
    pub into: String,
    pub links_retitled: u32,
}
