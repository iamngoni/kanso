//! What a notebook can be made as, and the words of the Video studio preset.
//! The rules that make one are in [`crate::presets`].

/// What a new notebook starts as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotebookKind {
    /// Nothing set up.
    Blank,
    /// A board of video stages, dates and two templates.
    VideoStudio,
}

impl NotebookKind {
    /// The choices New Notebook offers, in order.
    pub const ALL: [NotebookKind; 2] = [NotebookKind::Blank, NotebookKind::VideoStudio];

    pub fn label(self) -> &'static str {
        match self {
            NotebookKind::Blank => "Blank notebook",
            NotebookKind::VideoStudio => "Video studio",
        }
    }

    /// The line under the choice.
    pub fn description(self) -> &'static str {
        match self {
            NotebookKind::Blank => "An empty notebook for notes.",
            NotebookKind::VideoStudio => {
                "Video studio sets up a board (Idea \u{2192} Scripting \u{2192} Shooting \u{2192} Editing \u{2192} Published), a Schedule, and two templates: Video brief and Shot list. You can rename or change any of it later."
            }
        }
    }
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
