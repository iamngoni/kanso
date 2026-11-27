//! The text of every template a notebook starter makes. Each is Markdown
//! with `{{title}}` and `{{date}}` where they help (see
//! [`kanso_core::template`]); each `##` heading is followed by a one-line
//! prompt in a list item, so the note reads as a guide and not as an empty
//! form. The catalogue that names them is in [`crate::presets_model`].

// MARK: Product development

pub const FEATURE_PLAN: &str = "## Goal
- What should people be able to do when this ships, and why does it matter?

## Users and need
- Who asks for this, and what do they do today instead?

## Scope
- What is in, in a few bullets.

## Out of scope
- What this deliberately does not do.

## Approach
- How it will work, and which parts of the product it touches.

## Tasks
- [ ] First step

## Risks and open questions
- What could go wrong, and what is still undecided?

## Done when
- The checks that tell you it is finished.
";

pub const BUG_REPORT: &str = "## Actual behavior
- What happens, and where?

## Expected behavior
- What should happen instead?

## Steps to reproduce
1.

## Root cause
- Why it happens (fill in once known).

## Fix
- What changed, and the PR.

## Related
-
";

pub const PERFORMANCE_INVESTIGATION: &str = "## Symptom
- What is slow or heavy, for whom, and since when?

## Measurement
- How you measured it, and the numbers before any change.

## Hypotheses
- What you think the cause is, most likely first.

## Findings
- What the profile or the measurements showed.

## Change
- What you changed, and the numbers after.

## Follow-up
- What to watch, and what is still slow.
";

pub const EXPERIMENT: &str = "Status: running \u{b7} {{date}}

## Hypothesis
- If we do X, then Y will change, because Z.

## Setup
- What is being tried, for whom, and for how long.

## Success measure
- The number that decides it, and the bar it has to clear.

## Result
- What happened (fill in when it ends).

## Decision
- Ship, change or drop, and why.
";

pub const REFACTORING_PLAN: &str = "## Why now
- What the current code costs us.

## Current shape
- How it is built today, and what is painful about it.

## Target shape
- How it should look when this is done.

## Steps
- [ ] A small step that keeps everything working

## Safety net
- The tests or checks that prove behavior did not change.

## Risks
- What could break, and how you would notice.
";

pub const TEST_PLAN: &str = "## What is under test
- The feature or change, and the version it is in.

## Risks to cover
- What would hurt most if it broke.

## Cases
- [ ] The normal path
- [ ] An empty or missing input
- [ ] A large or slow case
- [ ] An error or a retry

## Environments
- Devices, platforms and accounts to try it on.

## Results
- What passed, what failed, and links to the issues.

## Sign-off
- Who agreed it is ready, and when.
";

pub const CHORE: &str = "## What
- The small job, in one line.

## Why
- What it keeps working or saves later.

## Steps
- [ ] Step one

## Done when
- How you will know it is finished.
";

pub const RELEASE_PLAN: &str = "Status: planned \u{b7} {{date}}

## Version
- The number, and what kind of release this is.

## Highlights
- What people will notice, in plain words.

## Checklist
- [ ] Changes merged and tests green
- [ ] Release notes written
- [ ] Build and upload
- [ ] Announce

## Known issues
- What ships unfixed, and the workaround.

## After release
- What to watch for in the first days.
";

pub const DECISION_RECORD: &str = "Status: proposed \u{b7} {{date}}

## Context
- What forces this decision?

## Decision
- What we will do.

## Consequences
- What gets easier, what gets harder.

## Alternatives
- What else was considered, and why not.
";

// MARK: Knowledge base

pub const LESSON: &str = "## What I learned
- The idea in one or two sentences.

## Why it matters
- Where it changes how you work.

## Key points
-

## Example
- A small case that shows it.

## Questions
- What is still unclear?

## Sources
-
";

pub const RESEARCH_NOTE: &str = "## Question
- What are you trying to find out?

## Sources
-

## Findings
- What the sources say, in your own words.

## Open questions
- What they leave unanswered.

## Conclusion
- What you now believe, and how sure you are.
";

pub const FIX: &str = "## Problem
- The symptom, and the exact error message.

## Cause
- Why it happens.

## Solution
1.

## How to check
- How you know it worked.

## Related
-
";

pub const TOOL_NOTE: &str = "## What it is
- What the tool does, and where to get it.

## Why I use it
- The job it does better than the alternatives.

## Setup
- How to install and configure it.

## Everyday commands
-

## Gotchas
- What tripped you up.

## Links
-
";

pub const ARCHITECTURE_NOTE: &str = "## Overview
- What this part of the system does, in a few sentences.

## Components
- The parts, and what each one owns.

## Data flow
- How a request or a piece of data moves through them.

## Boundaries
- What talks to what, and what must never.

## Trade-offs
- Why it is built this way, and what it costs.

## Related decisions
-
";

// MARK: Side projects

pub const PROJECT_BRIEF: &str = "## What it is
- One sentence that says what this project makes.

## Who it is for
- The person it helps, and the problem they have.

## Why I'm building it
- What you want out of it.

## First version
- The smallest thing that is worth showing someone.

## Next steps
- [ ] First step

## Notes and links
-
";

/// Shared by Side projects and Writing & publishing.
pub const IDEA: &str = "## The idea
- Say it in a sentence.

## Why it's interesting
- What makes it worth your time.

## First step
- The smallest thing that would test it.

## Open questions
-
";

// MARK: Daily journal

pub const DAILY_ENTRY: &str = "## What I did
-

## What blocked me
-

## What's next
- [ ]

## Notes
-
";

// MARK: Reading

pub const BOOK_NOTES: &str = "Author:
Started: {{date}}
Finished:

## Why I picked it up
- What drew you to it.

## Summary
- The book in a few sentences.

## Key ideas
-

## Quotes
- > A line worth keeping (page).

## What I'll do with it
- [ ] One thing to try

## Rating
- Out of five, and one line on why.
";

pub const ARTICLE_NOTES: &str = "Source:
Read: {{date}}

## Summary
- What it says, in a few sentences.

## Key points
-

## Quotes
- > A line worth keeping.

## My take
- Where you agree, and where you don't.

## Follow-up
- [ ] Something to look into
";

// MARK: Writing & publishing

pub const POST_OUTLINE: &str = "Status: outline \u{b7} {{date}}

## Working title
-

## Reader and promise
- Who it is for, and what they will get out of it.

## Outline
1.

## Sources and links
-

## Call to action
- What the reader should do next.

## Publish checklist
- [ ] Draft written
- [ ] Reviewed
- [ ] Title and summary final
- [ ] Published and shared
";

pub const VIDEO_OUTLINE: &str = "Status: outline \u{b7} {{date}}

## Working title
-

## Viewer and promise
- Who it is for, and what they will take away.

## Hook
- The first ten seconds.

## Segments
1.

## Shots and screen recordings
- [ ] A shot or capture you need

## Call to action
- What viewers should do at the end.

## Publish checklist
- [ ] Script done
- [ ] Recorded and edited
- [ ] Title and thumbnail
- [ ] Published and shared
";

pub const MONTHLY_NUMBERS: &str = "## Month
- {{date:%B %Y}}

## Numbers
| Measure | This month | Last month |
|---|---|---|
| Visitors | | |
| Subscribers | | |
| Views | | |

## What worked
-

## What didn't
-

## Next month
- [ ] One thing to try
";

pub const RESOURCE: &str = "Link:

## What it is
- A line on what this is and who made it.

## Why it's useful
- What you would use it for.

## Notes
-
";
