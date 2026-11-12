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

#[cfg(test)]
#[path = "tick_tests.rs"]
mod tests;
