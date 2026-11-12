//! Facts derived from a note body: task-list counts and ID links,
//! and the due dates tasks carry (`@2026-10-25`, `@2026-10-25T09:00`).

use crate::dates::{Date, DateTime};
use crate::repeat::repeat_token;
use crate::repeat_model::Repeat;

/// Counts of GitHub-style task-list items (`- [ ]` / `- [x]`) outside code fences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskCounts {
    pub total: u32,
    pub checked: u32,
}

/// Counts the task-list items in a markdown body.
pub fn count_tasks(body: &str) -> TaskCounts {
    let mut counts = TaskCounts::default();
    for line in lines_outside_fences(body) {
        if let Some(checked) = task_marker(line) {
            counts.total += 1;
            counts.checked += u32::from(checked);
        }
    }
    counts
}

/// A task-list item: where it is, whether it is done, its words and its
/// due date.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    /// The line (0-based) it is on.
    pub line: u32,
    pub checked: bool,
    /// The text after the checkbox, trimmed.
    pub text: String,
    /// `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, from an `@2026-10-25` (or
    /// `@2026-10-25T09:00`) in the text.
    pub due: Option<String>,
    /// The `every …` words right after the due date, when there are any
    /// (see [`crate::repeat`]).
    pub repeat: Option<Repeat>,
}

/// The task-list items in a markdown body, outside code fences, in order.
/// [`count_tasks`] counts the same items.
pub fn task_items(body: &str) -> Vec<TaskItem> {
    let mut items = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    for (ix, line) in body.lines().enumerate() {
        if let Some(open) = fence_marker(line.trim_start()) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if let Some(checked) = task_marker(line) {
            let at = line.find('[').map_or(line.len(), |at| at + 3);
            let text = line[at..].trim().to_owned();
            items.push(TaskItem {
                line: ix as u32,
                checked,
                due: due_date(&text),
                repeat: repeat_token(&text).map(|(_, repeat)| repeat),
                text,
            });
        }
    }
    items
}

/// The first `@YYYY-MM-DD` (with an optional `THH:MM`) in a task's text
/// that is a real date, as written without the `@`.
pub fn due_date(text: &str) -> Option<String> {
    due_token(text).map(|range| text[range.start + 1..range.end].to_owned())
}

/// Where the first due date is in a task's text: the byte range of the
/// `@YYYY-MM-DD` (or `@YYYY-MM-DDTHH:MM`) token, `@` included. The `@`
/// starts the text or follows a space, a tab or `(`, the date is a real
/// one, and the token ends the word.
pub fn due_token(text: &str) -> Option<std::ops::Range<usize>> {
    text.match_indices('@').find_map(|(at, _)| {
        if at > 0 && !text[..at].ends_with([' ', '\t', '(']) {
            return None;
        }
        let rest = &text[at + 1..];
        let date = rest.get(..10)?;
        let b = date.as_bytes();
        let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
        if !(digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10)) {
            return None;
        }
        let year: u32 = date[0..4].parse().ok()?;
        let month: u32 = date[5..7].parse().ok()?;
        let day: u32 = date[8..10].parse().ok()?;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        let mut end = 10;
        if let Some(time) = rest.get(10..16)
            && let [b'T', h1, h2, b':', m1, m2] = time.as_bytes()
            && [h1, h2, m1, m2].iter().all(|c| c.is_ascii_digit())
        {
            let hour = (h1 - b'0') * 10 + (h2 - b'0');
            let minute = (m1 - b'0') * 10 + (m2 - b'0');
            if hour < 24 && minute < 60 {
                end = 16;
            }
        }
        // The date must end the word.
        if rest[end..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric())
        {
            return None;
        }
        Some(at..at + 1 + end)
    })
}

/// A task's due date, with the time of day when it has one: what
/// `YYYY-MM-DD` and `YYYY-MM-DDTHH:MM` say.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Due {
    pub date: Date,
    /// `(hour, minute)`, local wall-clock time.
    pub time: Option<(u32, u32)>,
}

impl Due {
    /// Reads `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, as [`due_date`] gives them.
    pub fn parse(text: &str) -> Option<Due> {
        let date = Date::parse_iso(text.get(..10)?)?;
        let time = match text.get(10..)? {
            "" => None,
            rest => {
                let [b'T', h1, h2, b':', m1, m2] = rest.as_bytes() else {
                    return None;
                };
                if ![h1, h2, m1, m2].iter().all(|c| c.is_ascii_digit()) {
                    return None;
                }
                let hour = u32::from((h1 - b'0') * 10 + (h2 - b'0'));
                let minute = u32::from((m1 - b'0') * 10 + (m2 - b'0'));
                if hour >= 24 || minute >= 60 {
                    return None;
                }
                Some((hour, minute))
            }
        };
        Some(Due { date, time })
    }

    /// `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`: what [`Due::parse`] reads back.
    pub fn format(&self) -> String {
        match self.time {
            None => self.date.iso(),
            Some((hour, minute)) => format!("{}T{hour:02}:{minute:02}", self.date.iso()),
        }
    }

    /// The moment it falls due in local milliseconds since the epoch (a
    /// local time with the zone's offset added, as `dates` counts): its
    /// time, or the start of its day.
    pub fn local_ms(&self) -> i64 {
        let (hour, minute) = self.time.unwrap_or((0, 0));
        self.date.to_days() * DAY_MS + i64::from(hour) * 3_600_000 + i64::from(minute) * 60_000
    }
}

const DAY_MS: i64 = 86_400_000;

/// Where a task belongs in the Tasks view, relative to now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskGroup {
    /// Not done, and its day (or its time today) has passed.
    Overdue,
    /// Not done and due today, at or after now.
    Today,
    /// Not done and due after today.
    Upcoming,
    /// Not done and without a date.
    NoDate,
    /// Done, whatever its date.
    Done,
}

impl TaskGroup {
    /// The groups in the order the Tasks view shows them.
    pub const ALL: [TaskGroup; 5] = [
        TaskGroup::Overdue,
        TaskGroup::Today,
        TaskGroup::Upcoming,
        TaskGroup::NoDate,
        TaskGroup::Done,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TaskGroup::Overdue => "Overdue",
            TaskGroup::Today => "Today",
            TaskGroup::Upcoming => "Upcoming",
            TaskGroup::NoDate => "No date",
            TaskGroup::Done => "Completed",
        }
    }
}

/// The group of a task due `due` (or never), done or not, at local time
/// `now`.
pub fn task_group(due: Option<Due>, checked: bool, now: DateTime) -> TaskGroup {
    if checked {
        return TaskGroup::Done;
    }
    let Some(due) = due else {
        return TaskGroup::NoDate;
    };
    if due.date < now.date {
        return TaskGroup::Overdue;
    }
    if due.date > now.date {
        return TaskGroup::Upcoming;
    }
    match due.time {
        Some(time) if time < (now.hour, now.minute) => TaskGroup::Overdue,
        _ => TaskGroup::Today,
    }
}

/// How a due date reads beside its task: "Today", "Tomorrow", "Yesterday",
/// "Fri 25 Oct" (with the year when it isn't this one), and the time after
/// it when it has one: "Tomorrow 09:00". Pure.
pub fn due_label(due: Due, today: Date) -> String {
    let days = due.date.to_days() - today.to_days();
    let day = match days {
        0 => "Today".to_owned(),
        1 => "Tomorrow".to_owned(),
        -1 => "Yesterday".to_owned(),
        _ if due.date.year == today.year => {
            crate::dates::format(&DateTime::at_midnight(due.date), "%a %e %b")
        }
        _ => crate::dates::format(&DateTime::at_midnight(due.date), "%a %e %b %Y"),
    };
    match due.time {
        Some((hour, minute)) => format!("{day} {hour:02}:{minute:02}"),
        None => day,
    }
}

/// A due date's place in a body: the `@date` token's byte range and
/// whether its task is done.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DueSpan {
    pub range: std::ops::Range<usize>,
    pub due: Due,
    pub checked: bool,
}

/// The due-date tokens of the task-list items in a body, outside code
/// fences, in order: at most one a task, the one [`task_items`] reads.
pub fn due_spans(body: &str) -> Vec<DueSpan> {
    let mut spans = Vec::new();
    if !body.contains('@') {
        return spans;
    }
    let mut fence: Option<(u8, usize)> = None;
    let mut offset = 0;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        let start = offset;
        offset += raw.len();
        if let Some(open) = fence_marker(line.trim_start()) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some() || !line.contains('@') {
            continue;
        }
        let Some(checked) = task_marker(line) else {
            continue;
        };
        let at = line.find('[').map_or(line.len(), |at| at + 3);
        let text = &line[at..];
        if let Some(range) = due_token(text)
            && let Some(due) = Due::parse(&text[range.start + 1..range.end])
        {
            spans.push(DueSpan {
                range: start + at + range.start..start + at + range.end,
                due,
                checked,
            });
        }
    }
    spans
}

/// What [`set_task_checked`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskEdit {
    /// The body with the task's box ticked or cleared.
    Changed(String),
    /// The task was already that way.
    Unchanged,
    /// No task with that text is on that line, and no single other line
    /// has it: the body changed since the task was read.
    Missing,
}

/// Ticks (or clears) the task with text `text`, looked for on `line`
/// (0-based) first, then on any one other line that has it. The text is
/// what [`task_items`] gives.
pub fn set_task_checked(body: &str, line: u32, text: &str, checked: bool) -> TaskEdit {
    let items = task_items(body);
    let target = items
        .iter()
        .position(|t| t.line == line && t.text == text)
        .or_else(|| {
            let mut same = items.iter().enumerate().filter(|(_, t)| t.text == text);
            match (same.next(), same.next()) {
                (Some((ix, _)), None) => Some(ix),
                _ => None,
            }
        });
    let Some(index) = target else {
        return TaskEdit::Missing;
    };
    if items[index].checked == checked {
        return TaskEdit::Unchanged;
    }
    match toggle_task(body, index) {
        Some(body) => TaskEdit::Changed(body),
        None => TaskEdit::Missing,
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Toggles the `index`-th task-list item (0-based, in document order).
/// Returns `None` when the body has fewer tasks.
pub fn toggle_task(body: &str, index: usize) -> Option<String> {
    let mut seen = 0;
    let mut offset = 0;
    let mut fence: Option<(u8, usize)> = None;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
        } else if fence.is_none()
            && let Some(checked) = task_marker(line.trim_end_matches(['\n', '\r']))
        {
            if seen == index {
                let bracket = offset + line.find('[')?;
                let mut out = String::with_capacity(body.len());
                out.push_str(&body[..bracket + 1]);
                out.push(if checked { ' ' } else { 'x' });
                out.push_str(&body[bracket + 2..]);
                return Some(out);
            }
            seen += 1;
        }
        offset += line.len();
    }
    None
}

/// IDs (`note:…`) of the notes a body links to with ID links, in order, deduplicated.
pub fn linked_note_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for suffix in uri_suffixes(body, crate::links::NOTE_LINK_PREFIX) {
        let id = format!("note:{suffix}");
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// IDs (`file:…`) of the attachments a body embeds with attachment links, deduplicated.
pub fn linked_file_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for suffix in uri_suffixes(body, crate::links::FILE_LINK_PREFIX) {
        let id = format!("file:{suffix}");
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// IDs (`file:…`) of the attachments a note shows, deduplicated: the ones
/// its text links, and, for a canvas, the ones its attachment cards point
/// at (`file:<id>` with no link prefix). Who owns a file, what a lock seals
/// and what may stay plain all go by this. Pure.
pub fn shown_file_ids(body: &str) -> Vec<String> {
    let mut ids = linked_file_ids(body);
    if crate::canvas::fenced(body).is_some()
        && let Ok(canvas) = crate::canvas::parse(body)
    {
        for id in canvas.attachment_ids() {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

fn uri_suffixes<'a>(body: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    body.match_indices(prefix).filter_map(move |(at, _)| {
        let rest = &body[at + prefix.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(rest.len());
        (end > 0).then(|| &rest[..end])
    })
}

fn lines_outside_fences(body: &str) -> impl Iterator<Item = &str> {
    let mut fence: Option<(u8, usize)> = None;
    body.lines().filter(move |line| {
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            return false;
        }
        fence.is_none()
    })
}

/// `(fence char, run length)` when the line opens or closes a code fence.
pub(crate) fn fence_marker(trimmed: &str) -> Option<(u8, usize)> {
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let run = trimmed.bytes().take_while(|&b| b == first).count();
    (run >= 3).then_some((first, run))
}

/// `Some(checked)` when the line is a task-list item.
pub(crate) fn task_marker(line: &str) -> Option<bool> {
    let mut rest = line.trim_start();
    while let Some(after) = rest.strip_prefix('>') {
        rest = after.trim_start();
    }
    let after_marker = if let Some(after) = rest.strip_prefix(['-', '*', '+']) {
        after
    } else {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || digits > 9 {
            return None;
        }
        rest[digits..].strip_prefix(['.', ')'])?
    };
    if !after_marker.starts_with([' ', '\t']) {
        return None;
    }
    let item = after_marker.trim_start();
    let checked = match item.get(..3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    // Like GFM (micromark), the checkbox needs whitespace and then some text
    // after it; a bare `- [ ]` is a plain list item.
    let text = &item[3..];
    (text.starts_with([' ', '\t']) && !text.trim().is_empty()).then_some(checked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_tasks_outside_code_fences() {
        let body = "# Todo\n- [ ] one\n- [x] two\n  * [X] nested\n1. [ ] numbered\n\
                    > - [ ] quoted\n```\n- [ ] in code\n```\n- [] not a task\n-[ ] no space\n";
        assert_eq!(
            count_tasks(body),
            TaskCounts {
                total: 5,
                checked: 2
            }
        );
    }

    #[test]
    fn toggles_the_nth_task() {
        let body = "- [ ] a\n```\n- [ ] code\n```\n- [x] b\n";
        assert_eq!(
            toggle_task(body, 0).unwrap(),
            "- [x] a\n```\n- [ ] code\n```\n- [x] b\n"
        );
        assert_eq!(
            toggle_task(body, 1).unwrap(),
            "- [ ] a\n```\n- [ ] code\n```\n- [ ] b\n"
        );
        assert_eq!(toggle_task(body, 2), None);
    }

    #[test]
    fn toggling_twice_normalizes_uppercase_x() {
        let once = toggle_task("* [X] nested", 0).unwrap();
        assert_eq!(once, "* [ ] nested");
        assert_eq!(toggle_task(&once, 0).unwrap(), "* [x] nested");
    }

    #[test]
    fn extracts_links() {
        let body = "See [a](inkdrop://note/abc-1_2) and [b](inkdrop://note/abc-1_2) \
                    ![img](inkdrop://file:XyZ9) inkdrop://note/";
        assert_eq!(linked_note_ids(body), vec!["note:abc-1_2"]);
        assert_eq!(linked_file_ids(body), vec!["file:XyZ9"]);
    }

    #[test]
    fn task_items_carry_text_and_due_dates() {
        let body = "- [ ] Pay rent @2026-11-01\n```\n- [ ] no\n```\n* [x] Call @2026-10-25T09:30 ok\n- [ ] bad @2026-02-30 x@2026-01-01\n";
        let items = task_items(body);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].due.as_deref(), Some("2026-11-01"));
        assert_eq!(items[0].text, "Pay rent @2026-11-01");
        assert_eq!((items[1].line, items[1].checked), (4, true));
        assert_eq!(items[1].due.as_deref(), Some("2026-10-25T09:30"));
        assert_eq!(items[2].due, None);
        assert_eq!(due_date("@2024-02-29"), Some("2024-02-29".into()));
        assert_eq!(due_date("@2026-10-25T25:00"), None);
    }

    fn canvas_with_cards(files: &[String], text: &str) -> String {
        use crate::canvas::{Canvas, Node, to_body};
        let mut nodes: Vec<Node> = files
            .iter()
            .enumerate()
            .map(|(ix, file)| Node {
                id: format!("c{ix}"),
                kind: "file".into(),
                file: Some(file.clone()),
                width: 100.0,
                height: 80.0,
                ..Node::default()
            })
            .collect();
        nodes.push(Node {
            id: "t".into(),
            kind: "text".into(),
            text: Some(text.into()),
            width: 100.0,
            height: 80.0,
            ..Node::default()
        });
        to_body(&Canvas {
            nodes,
            ..Canvas::default()
        })
    }

    #[test]
    fn a_canvas_shows_the_files_its_cards_point_at() {
        let body = canvas_with_cards(
            &["file:card1".into(), "note:n1".into()],
            "see ![](inkdrop://file:linked1)",
        );
        assert!(crate::canvas::parse(&body).is_ok(), "{body}");
        // The text link alone is all `linked_file_ids` finds.
        assert_eq!(linked_file_ids(&body), vec!["file:linked1".to_owned()]);
        assert_eq!(
            shown_file_ids(&body),
            vec!["file:linked1".to_owned(), "file:card1".to_owned()]
        );
        // A note that isn't a canvas shows what it links.
        assert_eq!(
            shown_file_ids("file:card1 ![](inkdrop://file:x)"),
            vec!["file:x".to_owned()]
        );
    }

    proptest::proptest! {
        /// Every card's attachment and every linked one is shown, each once,
        /// and nothing else: a note card is not an attachment.
        #[test]
        fn prop_a_canvas_shows_every_card_and_link_once(
            cards in proptest::collection::vec("[a-z0-9]{1,8}", 0..6),
            notes in proptest::collection::vec("[a-z0-9]{1,8}", 0..3),
            linked in proptest::collection::vec("[a-z0-9]{1,8}", 0..3),
        ) {
            let mut files: Vec<String> = cards.iter().map(|c| format!("file:{c}")).collect();
            files.extend(notes.iter().map(|n| format!("note:{n}")));
            let text: String = linked.iter().map(|l| format!("![](inkdrop://file:{l}) ")).collect();
            let body = canvas_with_cards(&files, &text);
            let shown = shown_file_ids(&body);
            let unique: std::collections::HashSet<&String> = shown.iter().collect();
            proptest::prop_assert_eq!(unique.len(), shown.len());
            let want: std::collections::HashSet<String> =
                linked.iter().chain(cards.iter()).map(|c| format!("file:{c}")).collect();
            let got: std::collections::HashSet<String> = shown.into_iter().collect();
            proptest::prop_assert_eq!(got, want);
        }
    }

    proptest::proptest! {
        /// The items agree with the counts, whatever the text.
        #[test]
        fn prop_items_agree_with_counts(body in "([-*] \\[[ x]\\] [a-z@0-9-]{0,12}\n|```\n|[a-z ]{0,8}\n){0,12}") {
            let items = task_items(&body);
            let counts = count_tasks(&body);
            proptest::prop_assert_eq!(items.len() as u32, counts.total);
            proptest::prop_assert_eq!(items.iter().filter(|i| i.checked).count() as u32, counts.checked);
        }
    }

    #[test]
    fn a_checkbox_without_text_is_not_a_task() {
        assert_eq!(count_tasks("- [ ]\n* [x]   \n- [ ] real\n").total, 1);
        assert_eq!(
            toggle_task("- [ ]\n- [ ] real\n", 0).as_deref(),
            Some("- [ ]\n- [x] real\n")
        );
    }
}

#[cfg(test)]
mod due_tests {
    use super::*;
    use proptest::prelude::*;

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime {
        DateTime {
            date: Date::new(y, m, d).unwrap(),
            hour: h,
            minute: min,
            second: 0,
        }
    }

    fn due() -> impl Strategy<Value = Due> {
        (0i64..2_900_000, prop::option::of((0u32..24, 0u32..60))).prop_map(|(days, time)| Due {
            date: Date::from_days(days - 700_000),
            time,
        })
    }

    #[test]
    fn due_examples() {
        let due = Due::parse("2026-10-25T09:05").unwrap();
        assert_eq!(due.time, Some((9, 5)));
        assert_eq!(due.format(), "2026-10-25T09:05");
        assert_eq!(Due::parse("2026-10-25").unwrap().time, None);
        assert_eq!(Due::parse("2026-10-25T24:00"), None);
        assert_eq!(Due::parse("2026-10-25T09:00x"), None);
        assert_eq!(Due::parse("2026-02-30"), None);
        assert_eq!(due_token("Call (@2026-10-25) now"), Some(6..17));
        let today = Date::new(2026, 10, 20).unwrap();
        let label = |s: &str| due_label(Due::parse(s).unwrap(), today);
        assert_eq!(label("2026-10-20"), "Today");
        assert_eq!(label("2026-10-21T09:00"), "Tomorrow 09:00");
        assert_eq!(label("2026-10-19"), "Yesterday");
        assert_eq!(label("2026-10-23"), "Fri 23 Oct");
        assert_eq!(label("2027-01-05"), "Tue 5 Jan 2027");
    }

    #[test]
    fn grouping_examples() {
        let now = at(2026, 10, 20, 10, 30);
        let g = |s: Option<&str>, checked| task_group(s.and_then(Due::parse), checked, now);
        assert_eq!(g(Some("2026-10-19"), false), TaskGroup::Overdue);
        assert_eq!(g(Some("2026-10-20T10:29"), false), TaskGroup::Overdue);
        assert_eq!(g(Some("2026-10-20T10:30"), false), TaskGroup::Today);
        assert_eq!(g(Some("2026-10-20"), false), TaskGroup::Today);
        assert_eq!(g(Some("2026-10-21"), false), TaskGroup::Upcoming);
        assert_eq!(g(None, false), TaskGroup::NoDate);
        assert_eq!(g(Some("2026-10-19"), true), TaskGroup::Done);
    }

    #[test]
    fn spans_and_check_offs() {
        let body = "# Plan\n- [ ] Pay rent @2026-11-01 soon\n```\n- [ ] @2026-01-01 code\n```\n* [x] Call @2026-10-25T09:30\n- [ ] no date\n";
        let spans = due_spans(body);
        assert_eq!(spans.len(), 2);
        assert_eq!(&body[spans[0].range.clone()], "@2026-11-01");
        assert!(!spans[0].checked);
        assert_eq!(&body[spans[1].range.clone()], "@2026-10-25T09:30");
        assert!(spans[1].checked);
        let TaskEdit::Changed(done) = set_task_checked(body, 1, "Pay rent @2026-11-01 soon", true)
        else {
            panic!("not changed");
        };
        assert!(done.contains("- [x] Pay rent"));
        assert_eq!(
            set_task_checked(&done, 1, "Pay rent @2026-11-01 soon", true),
            TaskEdit::Unchanged
        );
        // Moved down a line, it is still found by its words.
        let moved = format!("intro\n{body}");
        assert!(matches!(
            set_task_checked(&moved, 1, "Pay rent @2026-11-01 soon", true),
            TaskEdit::Changed(_)
        ));
        assert_eq!(
            set_task_checked(body, 1, "Pay the rent", true),
            TaskEdit::Missing
        );
        // The same words on two other lines: which one is meant is unknown.
        let twice = "- [ ] a\n- [ ] a\n";
        assert_eq!(set_task_checked(twice, 5, "a", true), TaskEdit::Missing);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round trip: a due date written out reads back the same, and as
        /// the due date of a task that carries it.
        #[test]
        fn prop_due_round_trips(due in due(), words in "[a-z ]{0,10}") {
            let text = due.format();
            prop_assert_eq!(Due::parse(&text), Some(due));
            let task = format!("- [ ] x{words} @{text}");
            prop_assert_eq!(due_date(task[6..].trim()), Some(text.clone()));
            let spans = due_spans(&task);
            prop_assert_eq!(spans.len(), 1);
            prop_assert_eq!(&task[spans[0].range.clone()], format!("@{text}"));
            prop_assert_eq!(spans[0].due, due);
        }

        /// Invariant: every task lands in exactly one group: done tasks in
        /// Done, open ones without a date in No date, and an open dated
        /// task is overdue exactly when its moment has passed (its day, or
        /// its time today), today when it is later today, and upcoming
        /// after today.
        #[test]
        fn prop_each_task_in_one_group(due in prop::option::of(due()), checked: bool, now_days in -1000i64..1000, h in 0u32..24, m in 0u32..60) {
            let now = DateTime { date: Date::from_days(now_days), hour: h, minute: m, second: 0 };
            let group = task_group(due, checked, now);
            prop_assert_eq!(TaskGroup::ALL.iter().filter(|g| **g == group).count(), 1);
            prop_assert_eq!(group == TaskGroup::Done, checked);
            prop_assert_eq!(group == TaskGroup::NoDate, !checked && due.is_none());
            if let Some(due) = due && !checked {
                let now_ms = now.date.to_days() * DAY_MS + i64::from(h) * 3_600_000 + i64::from(m) * 60_000;
                let passed = due.date < now.date
                    || (due.date == now.date && due.time.is_some() && due.local_ms() < now_ms);
                prop_assert_eq!(group == TaskGroup::Overdue, passed);
                prop_assert_eq!(group == TaskGroup::Today, due.date == now.date && !passed);
                prop_assert_eq!(group == TaskGroup::Upcoming, due.date > now.date);
            }
        }

        /// Round trip: ticking a task and clearing it again gives the body
        /// back, and ticking changes only its box.
        #[test]
        fn prop_check_off_round_trips(body in "([-*] \\[ \\] [a-z@0-9-]{1,12}\n|```\n|[a-z ]{0,8}\n){0,12}") {
            for item in task_items(&body) {
                let TaskEdit::Changed(done) = set_task_checked(&body, item.line, &item.text, true) else {
                    prop_assert!(false, "an open task wasn't ticked");
                    unreachable!();
                };
                prop_assert_eq!(done.len(), body.len());
                prop_assert_eq!(count_tasks(&done).checked, count_tasks(&body).checked + 1);
                prop_assert_eq!(set_task_checked(&done, item.line, &item.text, false), TaskEdit::Changed(body.clone()));
            }
        }
    }
}
