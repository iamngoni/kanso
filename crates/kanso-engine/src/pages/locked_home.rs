//! The homepage of a locked notebook that the person has opened this
//! session (issue #201): computed sections only, built in memory from the
//! notes the session has opened and dropped with them. Nothing here reads
//! the store, writes a cache or reaches AI; the shell hands over the
//! opened notes of the notebook (and of the notebooks inside it that the
//! same password opens), and nothing built here is kept.

use std::collections::{HashMap, HashSet};

use kanso_core::dates::Date;
use kanso_core::home::{home_text, is_home};
use kanso_core::links::{LinkTarget, normalize_title, note_links, plain_line};
use kanso_core::tasks::task_items;

use super::notebook::{FIRST_TASKS, KEY_NOTES, MAX_TASKS, RECENT, RECENT_DAYS, relative_day};
use super::notebook_model::{HomeHeader, HomeView, LOCKED_LINE, NO_OPEN_TASKS, OpenedNote};
use super::{Item, PageView, Section, SectionSpec, TaskRef};
use crate::ask::Receipt;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

fn spec(key: &'static str, title: &'static str) -> SectionSpec {
    SectionSpec {
        key,
        title,
        kind: super::SectionKind::Computed,
    }
}

fn title_or_untitled(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.trim().to_owned()
    }
}

/// How many notes of `notes` link to each note of it (by ID or by
/// title), a note's links to itself left out.
fn linked_from(notes: &[OpenedNote]) -> HashMap<String, u32> {
    let mut by_title: HashMap<String, &str> = HashMap::new();
    for n in notes {
        by_title
            .entry(normalize_title(&n.title))
            .or_insert(n.id.as_str());
    }
    let ids: HashSet<&str> = notes.iter().map(|n| n.id.as_str()).collect();
    let mut sources: HashMap<String, HashSet<&str>> = HashMap::new();
    for note in notes {
        for link in note_links(&note.body) {
            let target = match link.target {
                LinkTarget::Note(id) => ids.contains(id.as_str()).then_some(id),
                LinkTarget::Title(t) => by_title.get(&t).map(|id| (*id).to_owned()),
            };
            if let Some(target) = target
                && target != note.id
            {
                sources.entry(target).or_default().insert(note.id.as_str());
            }
        }
    }
    sources
        .into_iter()
        .map(|(id, from)| (id, from.len() as u32))
        .collect()
}

/// The homepage of notebook `name` from its `notes` (the opened notes of
/// the notebook and the ones inside it), as of `now`. Key notes, recent
/// activity, open tasks (all of them with `all_tasks`) and the Home note; no AI, no people, no map, and a
/// line saying summaries are off.
pub fn locked_home(
    name: &str,
    book_id: &str,
    notes: &[OpenedNote],
    now: i64,
    all_tasks: bool,
) -> HomeView {
    let mut sections: Vec<Section> = Vec::new();
    // The Home note.
    let mut marked: Vec<&OpenedNote> = notes
        .iter()
        .filter(|n| n.book_id == book_id && is_home(&n.body))
        .collect();
    marked.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.id.cmp(&b.id)));
    if let Some(home) = marked.first() {
        sections.push(Section::computed(
            &spec("home_note", "Home note"),
            vec![Item {
                text: home_text(&home.body).trim().to_owned(),
                meta: Some(title_or_untitled(&home.title)),
                target: Some(home.id.clone()),
                ..Item::default()
            }],
            1,
        ));
    }
    // Key notes: pinned, then the most linked to, then the most recent.
    let counts = linked_from(notes);
    let mut seen: HashSet<&str> = HashSet::new();
    let mut key: Vec<Item> = Vec::new();
    let mut pinned: Vec<&OpenedNote> = notes.iter().filter(|n| n.pinned).collect();
    pinned.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.id.cmp(&b.id)));
    let edited = |n: &OpenedNote| format!("edited {}", relative_day(now, n.updated_at));
    for n in pinned.into_iter().take(KEY_NOTES) {
        seen.insert(n.id.as_str());
        key.push(Item {
            text: title_or_untitled(&n.title),
            meta: Some(format!("Pinned \u{b7} {}", edited(n))),
            target: Some(n.id.clone()),
            ..Item::default()
        });
    }
    let mut linked: Vec<&OpenedNote> = notes
        .iter()
        .filter(|n| counts.contains_key(&n.id))
        .collect();
    linked.sort_by(|a, b| {
        counts[&b.id]
            .cmp(&counts[&a.id])
            .then_with(|| a.id.cmp(&b.id))
    });
    for n in linked {
        if key.len() >= KEY_NOTES {
            break;
        }
        if seen.insert(n.id.as_str()) {
            let c = counts[&n.id];
            key.push(Item {
                text: title_or_untitled(&n.title),
                meta: Some(format!(
                    "Linked from {c} {} \u{b7} {}",
                    if c == 1 { "note" } else { "notes" },
                    edited(n)
                )),
                target: Some(n.id.clone()),
                ..Item::default()
            });
        }
    }
    let mut latest: Vec<&OpenedNote> = notes
        .iter()
        .filter(|n| n.updated_at >= now - RECENT_DAYS * DAY_MS)
        .collect();
    latest.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.id.cmp(&b.id)));
    for n in &latest {
        if key.len() >= KEY_NOTES {
            break;
        }
        if seen.insert(n.id.as_str()) {
            key.push(Item {
                text: title_or_untitled(&n.title),
                meta: Some(edited(n)),
                target: Some(n.id.clone()),
                ..Item::default()
            });
        }
    }
    let total = key.len();
    sections.push(Section::computed(&spec("key_notes", "Key notes"), key, total));
    // Recent activity.
    let mut newest: Vec<&OpenedNote> = notes.iter().collect();
    newest.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.id.cmp(&b.id)));
    let recent: Vec<Item> = newest
        .iter()
        .take(RECENT)
        .map(|n| {
            let new = (n.updated_at - n.created_at).abs() <= 60_000;
            let day = relative_day(now, n.updated_at);
            Item {
                text: title_or_untitled(&n.title),
                meta: Some(if new { "new" } else { "edited" }.to_owned()),
                group: Some(
                    day.chars()
                        .next()
                        .map(|c| c.to_uppercase().chain(day.chars().skip(1)).collect())
                        .unwrap_or_default(),
                ),
                target: Some(n.id.clone()),
                ..Item::default()
            }
        })
        .collect();
    let total = recent.len();
    sections.push(Section::computed(&spec("recent", "Recent activity"), recent, total));
    // Open tasks: dated first, soonest first.
    let mut tasks: Vec<(Option<String>, &OpenedNote, kanso_core::tasks::TaskItem)> = notes
        .iter()
        .flat_map(|n| {
            task_items(&n.body)
                .into_iter()
                .filter(|t| !t.checked)
                .map(move |t| (t.due.clone(), n, t))
        })
        .collect();
    let any_tasks = notes.iter().any(|n| !task_items(&n.body).is_empty());
    tasks.sort_by(|a, b| {
        (a.0.is_none(), &a.0, std::cmp::Reverse(a.1.updated_at), &a.1.id, a.2.line).cmp(&(
            b.0.is_none(),
            &b.0,
            std::cmp::Reverse(b.1.updated_at),
            &b.1.id,
            b.2.line,
        ))
    });
    let open_total = tasks.len();
    let items: Vec<Item> = tasks
        .into_iter()
        .take(if all_tasks { MAX_TASKS } else { FIRST_TASKS })
        .map(|(_, n, t)| Item {
            text: plain_line(&t.text),
            meta: Some(title_or_untitled(&n.title)),
            sources: vec![Receipt {
                n: 1,
                note_id: n.id.clone(),
                title: n.title.clone(),
                updated: Some(Date::from_local_ms(n.updated_at).iso()),
                start: t.line,
                end: t.line,
                quote: kanso_ai::prompt::cut_line(&t.text),
            }],
            task: Some(TaskRef {
                note_id: n.id.clone(),
                line: t.line,
                text: t.text,
                checked: false,
            }),
            ..Item::default()
        })
        .collect();
    let mut section = Section::computed(&spec("open_tasks", "Open tasks"), items, open_total);
    if open_total == 0 && any_tasks {
        section.note = Some(NO_OPEN_TASKS.to_owned());
    }
    sections.push(section);
    sections.retain(|s| !s.items.is_empty() || s.note.is_some());

    let last = notes.iter().map(|n| n.updated_at).max();
    let count = notes.len() as u32;
    let last_edited = last.map(|ms| relative_day(now, ms)).unwrap_or_default();
    let mut line = format!("{count} {}", if count == 1 { "note" } else { "notes" });
    if !last_edited.is_empty() {
        line.push_str(&format!(" \u{b7} last edited {last_edited}"));
    }
    HomeView {
        header: HomeHeader {
            name: name.to_owned(),
            notes: count,
            notebooks_inside: 0,
            last_edited,
            line,
            has_home_note: !marked.is_empty(),
        },
        page: PageView {
            sections,
            hidden: Vec::new(),
            based_on: None,
        },
        footer: Some(LOCKED_LINE.to_owned()),
        locked: false,
        ai_on: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn opened(i: usize, body: String, pinned: bool, at: i64) -> OpenedNote {
        OpenedNote {
            id: format!("note:{i}"),
            title: format!("Note {i}"),
            body,
            book_id: "book:x".into(),
            created_at: at,
            updated_at: at,
            pinned,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// The in-memory homepage lists each open task of the opened notes
        /// once, never more sections than a locked page has, and always
        /// ends with the line that summaries are off.
        #[test]
        fn prop_locked_home_lists_each_open_task_once_and_has_no_ai(
            specs in proptest::collection::vec((0usize..4, 0usize..3, proptest::bool::ANY, 0i64..5), 0..8),
        ) {
            let notes: Vec<OpenedNote> = specs
                .iter()
                .enumerate()
                .map(|(i, (open, done, pinned, at))| {
                    let mut body = String::new();
                    for t in 0..*open { body.push_str(&format!("- [ ] open {i}-{t}\n")); }
                    for t in 0..*done { body.push_str(&format!("- [x] done {i}-{t}\n")); }
                    opened(i, body, *pinned, 1_000_000 + at * 1000)
                })
                .collect();
            let view = locked_home("Vault", "book:x", &notes, 2_000_000, true);
            let want: usize = specs.iter().map(|(o, _, _, _)| *o).sum();
            let tasks = view.page.sections.iter().find(|s| s.key == "open_tasks");
            let got = tasks.map_or(0, |s| s.items.len());
            prop_assert_eq!(got, want.min(MAX_TASKS));
            prop_assert!(view.page.sections.iter().all(|s| s.kind == super::super::SectionKind::Computed));
            prop_assert_eq!(view.footer.as_deref(), Some(LOCKED_LINE));
            prop_assert!(!view.ai_on);
        }
    }
}
