//! The weekly review as Markdown, for "Save as Note": ordinary text with a
//! link to every source note. It doesn't update later.

use std::collections::BTreeMap;
use std::fmt::Write;

use kanso_core::dates::{Date, DateTime, format};
use kanso_core::tasks::{Due, task_items};

use super::{Review, ThemeKind};

/// The title a saved review gets: "Weekly review — 2026-W45".
pub fn review_title(from: Date) -> String {
    format!("Weekly review \u{2014} {}", iso_week_label(from))
}

/// `2026-W45` for the ISO week holding `date` (its year is the year of the
/// week's Thursday).
pub fn iso_week_label(date: Date) -> String {
    let thursday = date.add_days(3 - date.weekday() as i64);
    format!("{:04}-W{:02}", thursday.year, date.iso_week())
}

/// A wiki-link to a note by title, with the characters that would break
/// the link taken out.
fn link(title: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| {
            if matches!(c, '[' | ']' | '|' | '\n') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let clean = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        "[[Untitled]]".to_owned()
    } else {
        format!("[[{clean}]]")
    }
}

fn day(date: Date) -> String {
    format(&DateTime::at_midnight(date), "%a %e %b %Y")
}

/// A task's or loop's words as one line, without its due date token, in
/// curly quotes: the saved note is ordinary text, and quoted words are
/// never read as new loops.
fn task_text(text: &str) -> String {
    let shown = crate::tasks::display_text(text);
    let plain: String = shown
        .chars()
        .filter(|c| !matches!(c, '"' | '\u{201c}' | '\u{201d}' | '`'))
        .collect();
    format!(
        "\u{201c}{}\u{201d}",
        plain.split_whitespace().collect::<Vec<_>>().join(" ")
    )
}

/// The review as a note body: a heading per section, with the sentences
/// and tasks it lists and a link to each note they come from.
pub fn review_markdown(review: &Review) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}\n", review_title(review.from));
    let _ = writeln!(out, "{} \u{2013} {}\n", day(review.from), day(review.to));

    let _ = writeln!(out, "## What you wrote\n");
    if review.written.is_empty() {
        let _ = writeln!(out, "Nothing this week.\n");
    } else {
        let _ = writeln!(out, "{}\n", review.counts_text());
        let mut by_book: BTreeMap<&str, Vec<&super::WrittenNote>> = BTreeMap::new();
        for note in &review.written {
            by_book
                .entry(note.book_name.as_str())
                .or_default()
                .push(note);
        }
        for (book, notes) in by_book {
            let name = if book.is_empty() { "No notebook" } else { book };
            let _ = writeln!(out, "### {name}\n");
            for note in notes {
                let _ = writeln!(
                    out,
                    "- {}{}",
                    link(&note.title),
                    if note.created { " (new)" } else { "" }
                );
            }
            out.push('\n');
        }
    }

    let _ = writeln!(out, "## What you finished\n");
    if review.finished.is_empty() {
        let _ = writeln!(out, "Nothing ticked off.\n");
    } else {
        for item in &review.finished {
            let _ = writeln!(
                out,
                "- {} \u{2014} {}",
                task_text(&item.text),
                link(&item.note_title)
            );
        }
        out.push('\n');
    }

    let _ = writeln!(out, "## Still open\n");
    if review.still_open.is_empty() && review.overdue.is_empty() {
        let _ = writeln!(out, "Nothing open.\n");
    } else {
        for task in &review.overdue {
            let due = task.due.as_deref().and_then(Due::parse);
            let _ = writeln!(
                out,
                "- Overdue{}: {} \u{2014} {}",
                due.map_or(String::new(), |d| format!(" since {}", day(d.date))),
                task_text(&task.text),
                link(&task.note_title)
            );
        }
        for item in &review.still_open {
            let _ = writeln!(
                out,
                "- {} \u{2014} {}",
                task_text(&item.text),
                link(&item.note_title)
            );
        }
        out.push('\n');
    }

    if !review.themes.is_empty() {
        let _ = writeln!(out, "## Recurring themes\n");
        for theme in &review.themes {
            let term = match theme.kind {
                ThemeKind::Tag => format!("#{}", theme.term),
                ThemeKind::Word | ThemeKind::Phrase => theme.term.clone(),
            };
            let _ = writeln!(out, "- {term} ({} notes)", theme.notes);
        }
        out.push('\n');
    }

    let _ = writeln!(out, "## Next week\n");
    if review.next.is_empty() {
        let _ = writeln!(out, "Nothing due.");
    } else {
        for task in &review.next {
            let due = task.due.as_deref().and_then(Due::parse);
            let _ = writeln!(
                out,
                "- {} \u{2014} {}{}",
                task_text(&task.text),
                link(&task.note_title),
                due.map_or(String::new(), |d| format!(", due {}", day(d.date)))
            );
        }
    }
    // Tasks inside the saved text would become new tasks; none are written.
    debug_assert!(task_items(&out).is_empty());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brief::{Theme, WrittenNote};
    use kanso_core::loops::LoopKind;
    use kanso_store::{FinishedRow, LoopRow, LoopSource, TaskRow};

    fn review() -> Review {
        let note = |id: &str, title: &str, book: &str, created: bool| WrittenNote {
            id: id.into(),
            title: title.into(),
            book_id: format!("book:{book}"),
            book_name: book.into(),
            created,
            updated_at: 0,
        };
        Review {
            from: Date::new(2026, 11, 2).unwrap(),
            to: Date::new(2026, 11, 8).unwrap(),
            written: vec![
                note("note:a", "Plans", "Work", true),
                note("note:b", "Budget [draft]", "Work", false),
                note("note:c", "Recipes", "Home", false),
            ],
            finished: vec![FinishedRow {
                note_id: "note:a".into(),
                note_title: "Plans".into(),
                key: "k".into(),
                kind: "task".into(),
                text: "pay rent @2026-11-03".into(),
                at: 0,
            }],
            still_open: vec![LoopRow {
                note_id: "note:b".into(),
                note_title: "Budget".into(),
                book_id: "book:work".into(),
                line: 3,
                kind: LoopKind::Mine,
                text: "I'll call Rudo.".into(),
                key: "k2".into(),
                due: None,
                source: LoopSource::Rule,
                note_updated_at: 0,
            }],
            overdue: vec![TaskRow {
                note_id: "note:c".into(),
                note_title: "Recipes".into(),
                book_id: "book:home".into(),
                line: 0,
                checked: false,
                text: "buy flour @2026-10-30".into(),
                due: Some("2026-10-30".into()),
                note_updated_at: 0,
            }],
            themes: vec![
                Theme {
                    term: "planning".into(),
                    notes: 4,
                    kind: ThemeKind::Tag,
                },
                Theme {
                    term: "product roadmap".into(),
                    notes: 3,
                    kind: ThemeKind::Phrase,
                },
            ],
            next: vec![TaskRow {
                note_id: "note:a".into(),
                note_title: "Plans".into(),
                book_id: "book:work".into(),
                line: 1,
                checked: false,
                text: "send deck @2026-11-10".into(),
                due: Some("2026-11-10".into()),
                note_updated_at: 0,
            }],
        }
    }

    #[test]
    fn the_title_names_the_iso_week() {
        assert_eq!(
            review_title(Date::new(2026, 11, 2).unwrap()),
            "Weekly review \u{2014} 2026-W45"
        );
        // The year of the week's Thursday.
        assert_eq!(iso_week_label(Date::new(2025, 12, 29).unwrap()), "2026-W01");
        assert_eq!(iso_week_label(Date::new(2027, 1, 1).unwrap()), "2026-W53");
    }

    #[test]
    fn the_saved_review_reads_as_ordinary_markdown() {
        let expected = "\
# Weekly review \u{2014} 2026-W45

Mon 2 Nov 2026 \u{2013} Sun 8 Nov 2026

## What you wrote

1 new, 2 changed

### Home

- [[Recipes]]

### Work

- [[Plans]] (new)
- [[Budget draft]]

## What you finished

- \u{201c}pay rent\u{201d} \u{2014} [[Plans]]

## Still open

- Overdue since Fri 30 Oct 2026: \u{201c}buy flour\u{201d} \u{2014} [[Recipes]]
- \u{201c}I'll call Rudo.\u{201d} \u{2014} [[Budget]]

## Recurring themes

- #planning (4 notes)
- product roadmap (3 notes)

## Next week

- \u{201c}send deck\u{201d} \u{2014} [[Plans]], due Tue 10 Nov 2026
";
        assert_eq!(review_markdown(&review()), expected);
    }

    #[test]
    fn the_saved_review_has_no_tasks_and_no_loops_of_its_own() {
        let text = review_markdown(&review());
        assert!(task_items(&text).is_empty());
        assert!(kanso_core::loops::loop_items(&text).is_empty());
    }
}
