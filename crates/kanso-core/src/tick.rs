//! Ticking a task: the one function every place that ticks goes through
//! (the editor's box, the Tasks and Overview views, the board, a
//! reminder's Mark Done, MCP `kanso-set-task-done`, the phone).
//!
//! A task without `every …` just flips its box. Ticking an open repeating
//! one keeps the ticked line as a log (without its `every …`, so it never
//! spawns again) and writes the next one on a new line above it:
//!
//! ```markdown
//! - [ ] film Pixel review @2026-11-21T15:00 every week
//! - [x] film Pixel review @2026-11-14T15:00
//! ```
//!
//! The next due date is [`crate::repeat_model::Repeat::next`] counted from
//! `today`, the day it is ticked. Pure; the body is changed in one piece,
//! so one write is one undo step, and an undo puts back the body it was
//! made from.

use crate::dates::Date;
use crate::repeat::repeat_token;
use crate::tasks::{Due, TaskItem, due_token, task_items, toggle_task};

/// The body with the `index`-th task (0-based, in document order, as
/// [`task_items`] numbers them) ticked or cleared. `today` is the local
/// day it happens on. `None` when the body has no such task.
///
/// Ticking an open task with a repeat writes the next one above it (see
/// the module docs); anything else changes only the box, as
/// [`toggle_task`] does. A checked task with a repeat (an old log line)
/// is only ever cleared.
pub fn tick_task_at(body: &str, index: usize, today: Date) -> Option<String> {
    let items = task_items(body);
    let item = items.get(index)?;
    if !item.checked
        && let Some(written) = tick_repeating(body, item, today)
    {
        return Some(written);
    }
    toggle_task(body, index)
}

/// [`tick_task_at`], and when `link` is set, the tasks nested under or over
/// it follow (nesting is the indentation of task-list lines):
///
/// - Ticking an open task also ticks every open, non-repeating task under
///   it; then, if every task directly under its parent is done, the parent
///   (when it does not repeat) is ticked too, and so up the tree.
/// - Clearing a task also clears each ticked task above it; the tasks under
///   it stay as they are.
///
/// A repeating task is never ticked by a cascade, and counts as open when
/// the "all done" check looks at a parent's children. With `link` unset
/// this is exactly [`tick_task_at`].
pub fn tick_task_linked(body: &str, index: usize, today: Date, link: bool) -> Option<String> {
    if !link {
        return tick_task_at(body, index, today);
    }
    let before = task_items(body);
    let was_checked = before.get(index)?.checked;
    let mut out = tick_task_at(body, index, today)?;
    // A repeating tick writes its next one above: the ticked line moves down.
    let at = index + usize::from(task_items(&out).len() > before.len());
    let items = task_items(&out);
    let depth = indents(&out, &items);
    let mut flips: Vec<usize> = Vec::new();
    if was_checked {
        let mut child = at;
        while let Some(parent) = parent_of(&depth, child) {
            if items[parent].checked {
                flips.push(parent);
            }
            child = parent;
        }
    } else {
        let end = (at + 1..items.len())
            .find(|&j| depth[j] <= depth[at])
            .unwrap_or(items.len());
        flips.extend((at + 1..end).filter(|&j| !items[j].checked && items[j].repeat.is_none()));
        let mut checked: Vec<bool> = items.iter().map(|t| t.checked).collect();
        for &j in &flips {
            checked[j] = true;
        }
        let mut child = at;
        while let Some(parent) = parent_of(&depth, child) {
            if checked[parent] {
                // Already done: what is above it may be waiting on it.
                child = parent;
                continue;
            }
            if items[parent].repeat.is_some() {
                break;
            }
            let all_done = (parent + 1..items.len())
                .take_while(|&j| depth[j] > depth[parent])
                .filter(|&j| parent_of(&depth, j) == Some(parent))
                .all(|j| checked[j] && items[j].repeat.is_none());
            if !all_done {
                break;
            }
            checked[parent] = true;
            flips.push(parent);
            child = parent;
        }
    }
    for j in flips {
        out = toggle_task(&out, j)?;
    }
    Some(out)
}

/// Whether ticking the `index`-th task with `link` set writes something
/// other than the plain tick: the UI asks before it does.
pub fn would_cascade(body: &str, index: usize, today: Date) -> bool {
    tick_task_linked(body, index, today, true) != tick_task_at(body, index, today)
}

/// The indentation width of each task's line (a tab counts four).
fn indents(body: &str, items: &[TaskItem]) -> Vec<usize> {
    let lines: Vec<&str> = body.lines().collect();
    items
        .iter()
        .map(|item| {
            lines[item.line as usize]
                .chars()
                .take_while(|c| matches!(c, ' ' | '\t'))
                .map(|c| if c == '\t' { 4 } else { 1 })
                .sum()
        })
        .collect()
}

/// The task (by index) a task is nested under: the nearest one above it
/// that is indented less.
fn parent_of(depth: &[usize], child: usize) -> Option<usize> {
    (0..child).rev().find(|&j| depth[j] < depth[child])
}

/// The body with `item` (open, repeating) ticked and its next one above,
/// or `None` when it is not a repeating task with a real due date.
fn tick_repeating(body: &str, item: &TaskItem, today: Date) -> Option<String> {
    let repeat = item.repeat?;
    let due = Due::parse(item.due.as_deref()?)?;
    let (start, raw) = body
        .split_inclusive('\n')
        .scan(0usize, |offset, line| {
            let at = *offset;
            *offset += line.len();
            Some((at, line))
        })
        .nth(item.line as usize)?;
    let line = raw.trim_end_matches(['\n', '\r']);
    let eol = &raw[line.len()..];
    // `task_items` reads the text from three bytes after the bracket.
    let bracket = line.find('[')?;
    let text_at = bracket + 3;
    let text = line.get(text_at..)?;
    let due_at = due_token(text)?;
    let (repeat_at, _) = repeat_token(text)?;

    let next = repeat.next(&due, today);
    let mut next_line = String::with_capacity(line.len() + 8);
    next_line.push_str(&line[..text_at]);
    next_line.push_str(&text[..due_at.start + 1]);
    next_line.push_str(&next.format());
    next_line.push_str(&text[due_at.end..]);

    let mut done_line = String::with_capacity(line.len());
    done_line.push_str(&line[..bracket + 1]);
    done_line.push('x');
    done_line.push_str(&line[bracket + 2..text_at]);
    done_line.push_str(text[..repeat_at.start].trim_end_matches([' ', '\t']));
    done_line.push_str(&text[repeat_at.end..]);

    // The last line may have no line break; the next one still gets its own.
    let next_eol = if eol.is_empty() {
        if body.contains("\r\n") { "\r\n" } else { "\n" }
    } else {
        eol
    };
    let mut out = String::with_capacity(body.len() + next_line.len() + 2);
    out.push_str(&body[..start]);
    out.push_str(&next_line);
    out.push_str(next_eol);
    out.push_str(&done_line);
    out.push_str(eol);
    out.push_str(&body[start + raw.len()..]);
    Some(out)
}

/// The body with the open task on `line` (0-based) no longer repeating: its
/// `every …` words gone, its due date and every other byte as they were
/// (the chip menu's "Stop repeating"). `None` when that line holds no open
/// task with a repeat.
pub fn stop_repeating(body: &str, line: u32) -> Option<String> {
    let item = task_items(body)
        .into_iter()
        .find(|item| item.line == line)?;
    if item.checked || item.repeat.is_none() {
        return None;
    }
    let (start, raw) = body
        .split_inclusive('\n')
        .scan(0usize, |offset, line| {
            let at = *offset;
            *offset += line.len();
            Some((at, line))
        })
        .nth(line as usize)?;
    let text_line = raw.trim_end_matches(['\n', '\r']);
    // `task_items` reads the text from three bytes after the bracket.
    let text_at = text_line.find('[')? + 3;
    let text = text_line.get(text_at..)?;
    let (repeat_at, _) = repeat_token(text)?;
    let mut out = String::with_capacity(body.len());
    out.push_str(&body[..start + text_at]);
    out.push_str(text[..repeat_at.start].trim_end_matches([' ', '\t']));
    out.push_str(&text[repeat_at.end..]);
    out.push_str(&body[start + text_line.len()..]);
    Some(out)
}

#[cfg(test)]
#[path = "tick_tests.rs"]
mod tests;
