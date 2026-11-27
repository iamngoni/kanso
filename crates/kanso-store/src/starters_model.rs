//! What a notebook can be made as: the starter catalogue, and the words of
//! the Video studio preset. The rules that make one are in
//! [`crate::starters`]; the template texts are in [`crate::starters_text`].

use chrono::{Datelike as _, NaiveDate};
use kanso_core::TagColor;

use crate::starters_text as t;

/// What a new notebook starts as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotebookKind {
    /// Nothing set up.
    Blank,
    /// Sub-notebooks for features, bugs and the rest of one product's work.
    Product,
    /// Learning, research, fixes, tools and architecture.
    Knowledge,
    /// A first project and a place for ideas.
    SideProjects,
    /// A notebook for the year, with today's entry.
    Journal,
    /// Books and articles.
    Reading,
    /// Ideas, posts, videos, numbers and resources.
    Publishing,
    /// A board of video stages, dates and two templates.
    VideoStudio,
}

impl NotebookKind {
    /// The choices New Notebook offers, in order.
    pub const ALL: [NotebookKind; 8] = [
        NotebookKind::Blank,
        NotebookKind::Product,
        NotebookKind::Knowledge,
        NotebookKind::SideProjects,
        NotebookKind::Journal,
        NotebookKind::Reading,
        NotebookKind::Publishing,
        NotebookKind::VideoStudio,
    ];

    /// What the API and MCP call it.
    pub fn key(self) -> &'static str {
        match self {
            NotebookKind::Blank => "blank",
            NotebookKind::Product => "product",
            NotebookKind::Knowledge => "knowledge",
            NotebookKind::SideProjects => "side-projects",
            NotebookKind::Journal => "journal",
            NotebookKind::Reading => "reading",
            NotebookKind::Publishing => "publishing",
            NotebookKind::VideoStudio => "video-studio",
        }
    }

    /// The kind an API value names.
    pub fn parse(key: &str) -> Option<NotebookKind> {
        NotebookKind::ALL.into_iter().find(|kind| kind.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            NotebookKind::Blank => "Blank notebook",
            NotebookKind::Product => "Product development",
            NotebookKind::Knowledge => "Knowledge base",
            NotebookKind::SideProjects => "Side projects",
            NotebookKind::Journal => "Daily journal",
            NotebookKind::Reading => "Reading",
            NotebookKind::Publishing => "Writing & publishing",
            NotebookKind::VideoStudio => "Video studio",
        }
    }

    /// The line under the choice.
    pub fn description(self) -> &'static str {
        match self {
            NotebookKind::Blank => "An empty notebook for notes.",
            NotebookKind::Product => {
                "Plan features, track bugs and keep one product's work together."
            }
            NotebookKind::Knowledge => {
                "Keep what you learn: lessons, research, fixes and the tools you use."
            }
            NotebookKind::SideProjects => "One notebook per project, and a place for ideas.",
            NotebookKind::Journal => {
                "A short daily entry: what you did, what blocked you, what's next."
            }
            NotebookKind::Reading => "Notes on the books and articles you read.",
            NotebookKind::Publishing => "Plan and write posts and videos, from idea to published.",
            NotebookKind::VideoStudio => {
                "Video studio sets up a board (Idea \u{2192} Scripting \u{2192} Shooting \u{2192} Editing \u{2192} Published), a Schedule, and two templates: Video brief and Shot list. You can rename or change any of it later."
            }
        }
    }

    /// The notebook's icon: a name from
    /// `CHOICES` of the engine's book icons.
    pub fn icon(self) -> &'static str {
        match self {
            NotebookKind::Blank => "notebook",
            NotebookKind::Product => "rocket",
            NotebookKind::Knowledge => "brain",
            NotebookKind::SideProjects => "puzzle",
            NotebookKind::Journal => "notebook-pen",
            NotebookKind::Reading => "library",
            NotebookKind::Publishing => "feather",
            NotebookKind::VideoStudio => "clapperboard",
        }
    }

    /// Whether it makes notes, which a locked notebook can't take unsealed.
    pub fn makes_notes(self) -> bool {
        !matches!(self, NotebookKind::Blank | NotebookKind::VideoStudio)
    }

    /// The sub-notebooks, tags and templates it makes, or none for Blank and
    /// Video studio. `today` names the Daily journal's year and first note.
    pub fn starter(self, today: NaiveDate) -> Option<Starter> {
        let (children, tags) = match self {
            NotebookKind::Blank | NotebookKind::VideoStudio => return None,
            NotebookKind::Product => (
                vec![
                    book(
                        "Features",
                        "lightbulb",
                        "product/feature-plan",
                        "Feature plan",
                        t::FEATURE_PLAN,
                    ),
                    book(
                        "Bugs",
                        "bug",
                        "product/bug-report",
                        "Bug report",
                        t::BUG_REPORT,
                    ),
                    book(
                        "Performance",
                        "zap",
                        "product/performance-investigation",
                        "Performance investigation",
                        t::PERFORMANCE_INVESTIGATION,
                    ),
                    book(
                        "Experiments",
                        "flask-conical",
                        "product/experiment",
                        "Experiment",
                        t::EXPERIMENT,
                    ),
                    book(
                        "Refactoring",
                        "wrench",
                        "product/refactoring-plan",
                        "Refactoring plan",
                        t::REFACTORING_PLAN,
                    ),
                    book(
                        "Testing",
                        "list-checks",
                        "product/test-plan",
                        "Test plan",
                        t::TEST_PLAN,
                    ),
                    book("Chores", "hammer", "product/chore", "Chore", t::CHORE),
                    book(
                        "Release notes",
                        "megaphone",
                        "product/release-plan",
                        "Release plan",
                        t::RELEASE_PLAN,
                    ),
                    book(
                        "Decisions",
                        "git-branch",
                        "product/decision-record",
                        "Decision record",
                        t::DECISION_RECORD,
                    ),
                ],
                vec![
                    tag("web", TagColor::Blue),
                    tag("server", TagColor::Violet),
                    tag("mobile", TagColor::Green),
                    tag("docs", TagColor::Grey),
                    tag("security", TagColor::Red),
                    tag("important", TagColor::Orange),
                ],
            ),
            NotebookKind::Knowledge => (
                vec![
                    book(
                        "Learning",
                        "graduation-cap",
                        "knowledge/lesson",
                        "Lesson",
                        t::LESSON,
                    ),
                    book(
                        "Research",
                        "microscope",
                        "knowledge/research-note",
                        "Research note",
                        t::RESEARCH_NOTE,
                    ),
                    book("Troubleshooting", "wrench", "knowledge/fix", "Fix", t::FIX),
                    book(
                        "Tools",
                        "hammer",
                        "knowledge/tool-note",
                        "Tool note",
                        t::TOOL_NOTE,
                    ),
                    book(
                        "Architecture",
                        "building-2",
                        "knowledge/architecture-note",
                        "Architecture note",
                        t::ARCHITECTURE_NOTE,
                    ),
                ],
                vec![
                    tag("important", TagColor::Orange),
                    tag("read-later", TagColor::Teal),
                ],
            ),
            NotebookKind::SideProjects => (
                vec![
                    book(
                        "My first project",
                        "rocket",
                        "side-projects/project-brief",
                        "Project brief",
                        t::PROJECT_BRIEF,
                    ),
                    book("Ideas", "lightbulb", "idea", "Idea", t::IDEA),
                ],
                Vec::new(),
            ),
            NotebookKind::Journal => {
                let mut year = book(
                    &today.year().to_string(),
                    "calendar",
                    "journal/daily-entry",
                    "Daily entry",
                    t::DAILY_ENTRY,
                );
                year.note_title = today.format("%Y-%m-%d").to_string();
                (vec![year], Vec::new())
            }
            NotebookKind::Reading => (
                vec![
                    book(
                        "Books",
                        "book-open",
                        "reading/book-notes",
                        "Book notes",
                        t::BOOK_NOTES,
                    ),
                    book(
                        "Articles",
                        "newspaper",
                        "reading/article-notes",
                        "Article notes",
                        t::ARTICLE_NOTES,
                    ),
                ],
                vec![
                    tag("to-read", TagColor::Blue),
                    tag("favorite", TagColor::Pink),
                ],
            ),
            NotebookKind::Publishing => (
                vec![
                    book("Ideas", "lightbulb", "idea", "Idea", t::IDEA),
                    book(
                        "Posts",
                        "file-text",
                        "publishing/post-outline",
                        "Post outline",
                        t::POST_OUTLINE,
                    ),
                    book(
                        "Videos",
                        "video",
                        "publishing/video-outline",
                        "Video outline",
                        t::VIDEO_OUTLINE,
                    ),
                    book(
                        "Analytics",
                        "chart-line",
                        "publishing/monthly-numbers",
                        "Monthly numbers",
                        t::MONTHLY_NUMBERS,
                    ),
                    book(
                        "Resources",
                        "bookmark",
                        "publishing/resource",
                        "Resource",
                        t::RESOURCE,
                    ),
                ],
                vec![
                    tag("draft", TagColor::Grey),
                    tag("in-review", TagColor::Yellow),
                    tag("published", TagColor::Green),
                ],
            ),
        };
        Some(Starter { children, tags })
    }
}

/// A starter's notebook with one template, and a note titled like it.
fn book(
    name: &str,
    icon: &'static str,
    key: &'static str,
    title: &'static str,
    body: &'static str,
) -> StarterBook {
    StarterBook {
        name: name.to_owned(),
        icon,
        templates: vec![StarterTemplate { key, title, body }],
        note_title: title.to_owned(),
    }
}

fn tag(name: &'static str, color: TagColor) -> StarterTag {
    StarterTag { name, color }
}

/// What a starter makes beside its notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Starter {
    /// The notebooks inside it, in order.
    pub children: Vec<StarterBook>,
    pub tags: Vec<StarterTag>,
}

/// A notebook inside a starter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterBook {
    pub name: String,
    /// A name from `CHOICES` of the engine's book icons.
    pub icon: &'static str,
    /// The notebook names these, so New from template lists them first.
    /// Its one note is made from the first.
    pub templates: Vec<StarterTemplate>,
    /// The title of the note made in it.
    pub note_title: String,
}

/// A template a starter puts in the Templates notebook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StarterTemplate {
    /// Marks the template note, so the next starter reuses it
    /// (`"product/bug-report"`).
    pub key: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

/// A tag a starter makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StarterTag {
    pub name: &'static str,
    pub color: TagColor,
}

/// The stages of a studio's board, left to right.
pub const STUDIO_STAGES: [&str; 5] = ["Idea", "Scripting", "Shooting", "Editing", "Published"];

/// The title of the first studio template.
pub const VIDEO_BRIEF: &str = "Video brief";
/// The title of the second studio template.
pub const SHOT_LIST: &str = "Shot list";

/// The Video brief template's text.
pub const VIDEO_BRIEF_BODY: &str = "---
stage: Idea
embargo:
publish:
sponsor:
owner:
---
# {{title}}

## Angle
One sentence: why this video, why now.

## Key points
-

## Shot list
- [ ] A-roll intro
- [ ] B-roll product hero
- [ ] Screen recording

## Review
- [ ] Script review @{{date}}
- [ ] Rough cut review
- [ ] Final review
";

/// The Shot list template's text: lens, location and timing columns, and a
/// task for each shot.
pub const SHOT_LIST_BODY: &str = "# {{title}}

| Shot | Lens | Location | Timing |
|---|---|---|---|
| A-roll intro | 35mm | Studio | 0:00 |
| B-roll product hero | 85mm | Studio | 0:30 |
| Screen recording | | Desk | 1:00 |

## Shots
- [ ] A-roll intro
- [ ] B-roll product hero
- [ ] Screen recording
";
