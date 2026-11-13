//! Repeat lines after a merge (issue #390 › Repeating tasks). Ticking an
//! open repeating task writes the next one on a new line above it and keeps
//! the ticked line as a log ([`kanso_core::tick`]). When two members tick
//! the same instance offline, each writes that pair, and the CRDT keeps
//! both: the same next task twice.
//!
//! After a merge, a repeat line that repeats the line above it, or the two
//! lines above it (the next line and the log line a tick writes), is
//! removed. The rule reads only the merged text, so every member picks the
//! same lines to remove, removing them twice is removing them once, and
//! lines that differ in any byte are never touched: nothing anyone wrote is
//! lost but the copy.
//!
//! So that the copy is a whole line the rule can see, a tick reaches the
//! CRDT as the two lines it is ([`tick_edits`]), not as the character-level
//! diff of the text, which two members' identical edits would merge into
//! characters that were never written.

use std::ops::Range;

use kanso_core::dates::Date;
use kanso_core::tasks::task_items;
use kanso_core::tick::tick_task_at;
use yrs::{Text, Transact};

use crate::doc::{CollabDoc, units};
use crate::model::Splice;

/// The most lines a copy is: a tick's next line and its log line.
const BLOCK: usize = 2;

/// The byte ranges of `body` to remove: each run of lines that copies the
/// lines above it and starts with an open repeating task, with its line
/// break. Ranges are in order and never overlap.
pub fn duplicate_next_lines(body: &str) -> Vec<Range<usize>> {
    let open: std::collections::BTreeSet<u32> = task_items(body)
        .into_iter()
        .filter(|item| !item.checked && item.repeat.is_some())
        .map(|item| item.line)
        .collect();
    if open.is_empty() {
        return Vec::new();
    }
    // Each line's range and its text without the break.
    let mut lines: Vec<(Range<usize>, &str)> = Vec::new();
    let mut at = 0;
    for raw in body.split_inclusive('\n') {
        lines.push((at..at + raw.len(), raw.strip_suffix('\n').unwrap_or(raw)));
        at += raw.len();
    }
    let mut gone = vec![false; lines.len()];
    let mut i = 1;
    while i < lines.len() {
        if !open.contains(&(i as u32)) {
            i += 1;
            continue;
        }
        let copied = (1..=BLOCK).find(|&k| {
            i >= k && i + k <= lines.len() && (0..k).all(|j| lines[i + j].1 == lines[i + j - k].1)
        });
        match copied {
            Some(k) => {
                gone[i..i + k].fill(true);
                i += k;
            }
            None => i += 1,
        }
    }
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut j = 0;
    while j < lines.len() {
        if !gone[j] {
            j += 1;
            continue;
        }
        let first = j;
        while j + 1 < lines.len() && gone[j + 1] {
            j += 1;
        }
        let last = &lines[j].0;
        let mut start = lines[first].0.start;
        // The last line has no break of its own: the one before it goes.
        if !body[last.clone()].ends_with('\n') {
            start -= 1;
        }
        ranges.push(start..last.end);
        j += 1;
    }
    ranges
}

/// The edits that turn `old` into `new` when `new` is `old` with an open
/// repeating task ticked ([`tick_task_at`]): the next line inserted as a
/// line, the ticked line rewritten as the log line. `None` for any other
/// change.
pub(crate) fn tick_edits(old: &str, new: &str) -> Option<Vec<Splice>> {
    let before: Vec<&str> = old.split_inclusive('\n').collect();
    let after: Vec<&str> = new.split_inclusive('\n').collect();
    if before.is_empty() || after.len() != before.len() + 1 {
        return None;
    }
    let head = before
        .iter()
        .zip(&after)
        .take_while(|(a, b)| a == b)
        .count();
    let tail = before[head..]
        .iter()
        .rev()
        .zip(after[head..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    if before.len() - head - tail != 1 {
        return None;
    }
    let line = before[head].strip_suffix('\n').unwrap_or(before[head]);
    let next = after[head].strip_suffix('\n')?;
    let log = after[head + 1]
        .strip_suffix('\n')
        .unwrap_or(after[head + 1]);
    let open = |text: &str| {
        let items = task_items(text);
        items.len() == 1 && !items[0].checked && items[0].repeat.is_some()
    };
    if line.contains('\r') || !open(line) || !open(next) {
        return None;
    }
    // The log line of a tick doesn't depend on the day it is ticked.
    let ticked = tick_task_at(&format!("{line}\n"), 0, Date::new(2000, 1, 1)?)?;
    if ticked.lines().nth(1) != Some(log) {
        return None;
    }
    let at: usize = before[..head].iter().map(|l| l.len()).sum();
    Some(vec![
        Splice {
            at,
            delete: 0,
            insert: after[head].to_owned(),
        },
        // The break goes with the line, so two of these merge into whole
        // lines.
        Splice {
            at,
            delete: before[head].len(),
            insert: after[head + 1].to_owned(),
        },
    ])
}

impl CollabDoc {
    /// Removes the repeat lines a merge doubled ([`duplicate_next_lines`])
    /// from the body, and returns the update that does it, `None` when
    /// there are none.
    pub fn dedupe_repeat_lines(&self) -> Option<Vec<u8>> {
        let text = self.body_text();
        let ranges = duplicate_next_lines(&text);
        if ranges.is_empty() {
            return None;
        }
        let mut txn = self.doc().transact_mut();
        for range in ranges.iter().rev() {
            self.body.remove_range(
                &mut txn,
                units(&text[..range.start]),
                units(&text[range.clone()]),
            );
        }
        Some(txn.encode_update_v1())
    }
}
