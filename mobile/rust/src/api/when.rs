//! Typed dates in the editor: the hint for a phrase at the end of a task
//! and the `@` date list, from the desktop's parser.

use flutter_rust_bridge::frb;
use kanso_mobile::{DateChoices, KansoError, Library, LocalTime, WhenHint};

/// The hint for one line of the note at the phone's local time `now`, or
/// `None`. `month_first` reads `11/14` as November 14th.
#[frb(sync)]
pub fn when_parse(
    line: String,
    now: LocalTime,
    month_first: bool,
) -> Result<Option<WhenHint>, KansoError> {
    Library::when_parse(&line, now, month_first)
}

/// The `@` list for the text of the caret's line up to the caret, or `None`.
#[frb(sync)]
pub fn date_choices(
    before_caret: String,
    now: LocalTime,
) -> Result<Option<DateChoices>, KansoError> {
    Library::date_choices(&before_caret, now)
}

/// How the date and repeat at the end of `text` read in Quick Capture's Task
/// mode (`Fri 13 Nov, 15:00 · every week`), or `None`.
#[frb(sync)]
pub fn capture_task_hint(
    text: String,
    now: LocalTime,
    month_first: bool,
) -> Result<Option<String>, KansoError> {
    Library::capture_task_hint(&text, now, month_first)
}
