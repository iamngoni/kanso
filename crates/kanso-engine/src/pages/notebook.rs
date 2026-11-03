//! A notebook's homepage as a [`PageSource`] (issue #201): its Home note,
//! an AI summary, key notes, recent activity, open tasks, people and
//! projects, decisions, a small map and the notebooks inside it.
//!
//! The computed sections are built on open from the derived tables over
//! the notebook's subtree and never stored. The two AI sections (About and
//! Decisions) are written by [`super::write::write_section`] and cached on
//! the notebook's own document. Locked notebooks and notes are never
//! walked into: not even a count includes them.

use std::collections::{HashMap, HashSet};


use kanso_ai::Excerpt;
use kanso_core::dates::Date;
use kanso_core::home::{SECTION_KEYS, decision_lines, home_text};
use kanso_core::links::plain_line;
use kanso_core::passages::{Passage, passages};
use kanso_store::{HomeNoteRow, Store, TaskQuery};

use super::notebook_model::{
    AI_OFF_LINE, HomeHeader, HomeView, MapNode, MapView, NO_OPEN_TASKS,
};
use super::{
    Item, OpenOptions, PASSAGES_PER_NOTE, PageSource, Passages, Section, SectionKind,
    SectionSpec, SectionState, SourceNote, open,
};
use crate::ask::receipts::{MAX_RANGE, short_date};
use crate::ask::{Candidate, Receipt, eligible_note};
#[cfg(feature = "discovery")]
use crate::discovery::graph::{Graph, SEED, Viewport, layout, node_radius};

/// Key notes shown.
pub const KEY_NOTES: usize = 6;
/// Changes listed under Recent activity.
pub const RECENT: usize = 10;
/// Tasks shown before "Show all".
pub const FIRST_TASKS: usize = 5;
/// Tasks a section holds in all.
pub const MAX_TASKS: usize = 200;
/// People and projects listed.
pub const MAX_PEOPLE: usize = 8;
/// Decisions kept.
pub const MAX_DECISIONS: usize = 8;
/// Notes in the map.
pub const MAP_NOTES: usize = 40;
/// Days a note counts as recently edited for key notes.
pub const RECENT_DAYS: i64 = 90;
/// Notes the decision finder reads the bodies of, newest first.
pub const DECISION_SCAN: usize = 300;
/// Recent notes About reads an opening passage of.
pub const ABOUT_RECENT: usize = 10;
/// The map canvas, in points.
pub const MAP_SIZE: [f32; 2] = [320., 200.];

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

const SPECS: [SectionSpec; 9] = [
    SectionSpec {
        key: "home_note",
        title: "Home note",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "about",
        title: "About",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "key_notes",
        title: "Key notes",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "recent",
        title: "Recent activity",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "open_tasks",
        title: "Open tasks",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "people",
        title: "People & projects",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "decisions",
        title: "Decisions",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "map",
        title: "Map",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "inside",
        title: "Inside this notebook",
        kind: SectionKind::Computed,
    },
];

/// The title of a section by key.
pub fn title_of(key: &str) -> Option<&'static str> {
    SPECS.iter().find(|s| s.key == key).map(|s| s.title)
}

/// A notebook's homepage.
#[derive(Clone, Debug)]
pub struct NotebookPage {
    pub id: String,
    pub name: String,
    /// When the page is opened (ms since the epoch), for "Today" and the
    /// 90-day window.
    pub now: i64,
}

/// "today", "yesterday" or "28 Oct" for a time, as of `now`.
pub fn relative_day(now: i64, ms: i64) -> String {
    let (today, day) = (Date::from_local_ms(now), Date::from_local_ms(ms));
    if day == today {
        "today".to_owned()
    } else if day == today.add_days(-1) {
        "yesterday".to_owned()
    } else {
        short_date(&day.iso()).unwrap_or_default()
    }
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn title_or_untitled(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.trim().to_owned()
    }
}

fn receipt(n: usize, note_id: &str, title: &str, updated: Option<String>, line: u32, text: &str) -> Receipt {
    Receipt {
        n,
        note_id: note_id.to_owned(),
        title: title.to_owned(),
        updated,
        start: line,
        end: line,
        quote: kanso_ai::prompt::cut_line(text),
    }
}

impl NotebookPage {
    pub fn new(id: &str, name: &str, now: i64) -> NotebookPage {
        NotebookPage {
            id: id.to_owned(),
            name: name.to_owned(),
            now,
        }
    }

    /// The page of notebook `id`, or `None` when it is gone or locked
    /// (a locked notebook has no homepage on disk).
    pub fn load(store: &Store, id: &str, now: i64) -> kanso_store::Result<Option<NotebookPage>> {
        if store.book_subtree(id, false)?.is_empty() {
            return Ok(None);
        }
        let name = store
            .get(id)?
            .and_then(|d| d.fields.get("name").and_then(|n| n.as_str().map(str::to_owned)))
            .unwrap_or_default();
        Ok(Some(NotebookPage::new(id, &name, now)))
    }

    /// The header line.
    pub fn header(&self, store: &Store) -> kanso_store::Result<HomeHeader> {
        let stats = store.home_stats(&self.id)?;
        let last = stats
            .last_edited
            .map(|ms| relative_day(self.now, ms))
            .unwrap_or_default();
        let mut line = plural(stats.notes, "note", "notes");
        if stats.notebooks_inside > 0 {
            line.push_str(&format!(
                " \u{b7} {} inside",
                plural(stats.notebooks_inside, "notebook", "notebooks")
            ));
        }
        if !last.is_empty() {
            line.push_str(&format!(" \u{b7} last edited {last}"));
        }
        Ok(HomeHeader {
            name: self.name.clone(),
            notes: stats.notes,
            notebooks_inside: stats.notebooks_inside,
            last_edited: last,
            line,
            has_home_note: store.home_note(&self.id)?.is_some(),
        })
    }

    fn group_of(&self, ms: i64) -> String {
        capitalised(&relative_day(self.now, ms))
    }

    /// The key notes: pinned ones first, then the most linked to from
    /// inside the notebook, then the most edited in the last 90 days.
    pub fn key_note_items(&self, store: &Store) -> kanso_store::Result<Vec<Item>> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut items: Vec<Item> = Vec::new();
        let linked = store.home_linked(&self.id, 64)?;
        let counts: HashMap<&str, u32> = linked
            .iter()
            .map(|l| (l.note_id.as_str(), l.linked_from))
            .collect();
        let edited = |ms: i64| format!("edited {}", relative_day(self.now, ms));
        let linked_from = |n: u32| format!("Linked from {}", plural(n, "note", "notes"));
        for note in store.home_pinned(&self.id, KEY_NOTES)? {
            let mut why = vec!["Pinned".to_owned()];
            if let Some(n) = counts.get(note.id.as_str()) {
                why.push(linked_from(*n));
            }
            why.push(edited(note.updated_at));
            seen.insert(note.id.clone());
            items.push(note_item(&note, why.join(" \u{b7} ")));
        }
        for link in &linked {
            if items.len() >= KEY_NOTES {
                break;
            }
            if !seen.insert(link.note_id.clone()) {
                continue;
            }
            let Some(doc) = store.get(&link.note_id)? else {
                continue;
            };
            let title = doc
                .fields
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_owned();
            let updated = doc
                .fields
                .get("updatedAt")
                .and_then(|t| t.as_i64())
                .unwrap_or(0);
            items.push(Item {
                text: title_or_untitled(&title),
                meta: Some(format!(
                    "{} \u{b7} {}",
                    linked_from(link.linked_from),
                    edited(updated)
                )),
                target: Some(link.note_id.clone()),
                ..Item::default()
            });
        }
        if items.len() < KEY_NOTES {
            let since = self.now - RECENT_DAYS * DAY_MS;
            for note in store.home_edited_since(&self.id, since, KEY_NOTES * 2)? {
                if items.len() >= KEY_NOTES {
                    break;
                }
                if !seen.insert(note.id.clone()) {
                    continue;
                }
                let why = if note.revisions >= 2 {
                    format!(
                        "Edited {} times lately \u{b7} {}",
                        note.revisions,
                        edited(note.updated_at)
                    )
                } else {
                    capitalised(&edited(note.updated_at))
                };
                items.push(note_item(&note, why));
            }
        }
        items.truncate(KEY_NOTES);
        Ok(items)
    }

    fn recent_items(&self, store: &Store) -> kanso_store::Result<Vec<Item>> {
        let notes = store.home_recent(&self.id, RECENT)?;
        Ok(notes
            .into_iter()
            .map(|n| {
                let new = (n.updated_at - n.created_at).abs() <= 60_000;
                let verb = if new { "new" } else { "edited" };
                let inside = n.book_id != self.id;
                let title = title_or_untitled(&n.title);
                Item {
                    text: if inside && !n.book_name.trim().is_empty() {
                        format!("{}/{}", n.book_name.trim(), title)
                    } else {
                        title
                    },
                    meta: Some(verb.to_owned()),
                    group: Some(self.group_of(n.updated_at)),
                    target: Some(n.id),
                    ..Item::default()
                }
            })
            .collect())
    }

    fn task_section(&self, store: &Store, all: bool) -> kanso_store::Result<Section> {
        let spec = SPECS[4];
        let subtree = store.book_subtree(&self.id, false)?;
        let total = store.home_open_task_count(&self.id)? as usize;
        let rows = store.tasks(&TaskQuery {
            book_ids: Some(subtree),
            limit: Some(if all { MAX_TASKS } else { FIRST_TASKS }),
            ..TaskQuery::default()
        })?;
        let items = rows
            .into_iter()
            .map(|t| {
                let title = title_or_untitled(&t.note_title);
                let line = t.text.clone();
                Item {
                    text: plain_line(&t.text),
                    meta: Some(title.clone()),
                    sources: vec![receipt(
                        1,
                        &t.note_id,
                        &t.note_title,
                        Some(Date::from_local_ms(t.note_updated_at).iso()),
                        t.line,
                        &line,
                    )],
                    task: Some(super::TaskRef {
                        note_id: t.note_id,
                        line: t.line,
                        text: t.text,
                        checked: false,
                    }),
                    ..Item::default()
                }
            })
            .collect();
        let mut section = Section::computed(&spec, items, total);
        if total == 0 && store.home_task_count(&self.id)? > 0 {
            section.note = Some(NO_OPEN_TASKS.to_owned());
        }
        Ok(section)
    }

    /// The map's nodes and links, laid out on a [`MAP_SIZE`] canvas with
    /// the shared seed: the same library draws the same map every time.
    /// The most linked [`MAP_NOTES`] notes of the notebook and the ones
    /// inside it; a link to a note outside is a stub on its end.
    #[cfg(not(feature = "discovery"))]
    pub fn map(&self, _store: &Store) -> kanso_store::Result<MapView> {
        Ok(MapView {
            width: MAP_SIZE[0],
            height: MAP_SIZE[1],
            ..MapView::default()
        })
    }

    /// See the other build of this method.
    #[cfg(feature = "discovery")]
    pub fn map(&self, store: &Store) -> kanso_store::Result<MapView> {
        let (notes, links) = store.home_graph_rows(&self.id)?;
        let inside: HashSet<String> = notes.iter().map(|n| n.id.clone()).collect();
        let graph = Graph::build(notes, &links);
        let mut order: Vec<usize> = (0..graph.len()).collect();
        order.sort_by(|&a, &b| {
            graph
                .degree(b)
                .cmp(&graph.degree(a))
                .then_with(|| graph.nodes[a].id.cmp(&graph.nodes[b].id))
        });
        order.truncate(MAP_NOTES);
        order.sort_unstable();
        let shown = graph.subgraph(&order);
        if shown.is_empty() {
            return Ok(MapView {
                width: MAP_SIZE[0],
                height: MAP_SIZE[1],
                ..MapView::default()
            });
        }
        // Stubs: links from a shown note to a readable note outside.
        let kept: HashSet<&str> = shown.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut stubs: HashMap<String, HashSet<String>> = HashMap::new();
        let mut resolved: HashMap<String, Option<String>> = HashMap::new();
        for link in &links {
            if !kept.contains(link.from.as_str()) {
                continue;
            }
            let target = match (&link.target_id, &link.target_title) {
                (Some(id), _) => Some(id.clone()),
                (None, Some(title)) => resolved
                    .entry(title.clone())
                    .or_insert_with(|| store.resolve_title(title).ok().flatten().map(|(id, _)| id))
                    .clone(),
                (None, None) => None,
            };
            let Some(target) = target else { continue };
            if inside.contains(&target) || target == link.from {
                continue;
            }
            if store.note_is_locked(&target).unwrap_or(true) {
                continue;
            }
            if store.get(&target)?.is_some() {
                stubs.entry(link.from.clone()).or_default().insert(target);
            }
        }
        let mut pos = layout(&shown, SEED);
        let view = Viewport::fit(&pos, MAP_SIZE, 18., 1.5);
        for p in &mut pos {
            *p = view.to_screen(*p, MAP_SIZE);
        }
        let nodes = shown
            .nodes
            .iter()
            .enumerate()
            .map(|(ix, n)| MapNode {
                id: n.id.clone(),
                title: title_or_untitled(&n.title),
                x: pos[ix][0],
                y: pos[ix][1],
                radius: node_radius(shown.degree(ix)),
                degree: shown.degree(ix) as u32,
                stubs: stubs.get(&n.id).map_or(0, |s| s.len() as u32),
            })
            .collect();
        Ok(MapView {
            width: MAP_SIZE[0],
            height: MAP_SIZE[1],
            nodes,
            edges: shown.edges.clone(),
        })
    }

    fn map_items(&self, store: &Store) -> kanso_store::Result<Vec<Item>> {
        // The section lists the map's notes, most linked first; the picture
        // is [`NotebookPage::map`].
        let map = self.map(store)?;
        let mut nodes = map.nodes;
        nodes.sort_by(|a, b| b.degree.cmp(&a.degree).then_with(|| a.id.cmp(&b.id)));
        Ok(nodes
            .into_iter()
            .map(|n| Item {
                text: n.title,
                meta: Some(plural(n.degree, "link", "links")),
                target: Some(n.id),
                ..Item::default()
            })
            .collect())
    }

    fn inside_items(&self, store: &Store) -> kanso_store::Result<Vec<Item>> {
        Ok(store
            .home_children(&self.id)?
            .into_iter()
            .map(|c| {
                if c.locked {
                    return Item {
                        text: c.name,
                        tag: Some("locked".to_owned()),
                        target: Some(c.id),
                        ..Item::default()
                    };
                }
                let mut meta = plural(c.notes, "note", "notes");
                if let Some(ms) = c.last_edited {
                    meta.push_str(&format!(" \u{b7} edited {}", relative_day(self.now, ms)));
                }
                if c.open_tasks > 0 {
                    meta.push_str(&format!(" \u{b7} {} open", c.open_tasks));
                }
                Item {
                    text: c.name,
                    meta: Some(meta),
                    target: Some(c.id),
                    ..Item::default()
                }
            })
            .collect())
    }

    fn people_items(&self, store: &Store) -> kanso_store::Result<Vec<Item>> {
        Ok(store
            .home_entities(&self.id, MAX_PEOPLE)?
            .into_iter()
            .map(|e| Item {
                text: title_or_untitled(&e.title),
                meta: Some(plural(e.notes, "note", "notes")),
                target: Some(e.id),
                tag: Some(e.kind),
                ..Item::default()
            })
            .collect())
    }

    /// The notes About and Decisions read, newest first, that may be sent
    /// to AI.
    fn eligible_recent(
        &self,
        store: &Store,
        limit: usize,
    ) -> kanso_store::Result<Vec<(HomeNoteRow, Candidate)>> {
        let mut out = Vec::new();
        for row in store.home_recent(&self.id, limit * 2)? {
            if out.len() >= limit {
                break;
            }
            if let Some(candidate) = eligible_note(store, &row.id)? {
                out.push((row, candidate));
            }
        }
        Ok(out)
    }
}

fn note_item(note: &HomeNoteRow, why: String) -> Item {
    Item {
        text: title_or_untitled(&note.title),
        meta: Some(why),
        target: Some(note.id.clone()),
        ..Item::default()
    }
}

/// The numbered lines of `chosen` passages of a note, for an excerpt.
fn excerpt_of(candidate: &Candidate, chosen: &[&Passage]) -> (Excerpt, usize) {
    let lines: Vec<&str> = candidate.body.split('\n').collect();
    let mut numbered: Vec<(u32, String)> = Vec::new();
    let mut sorted: Vec<&&Passage> = chosen.iter().collect();
    sorted.sort_by_key(|p| p.start);
    for p in sorted {
        for n in p.start..=p.end.min(p.start + MAX_RANGE * 4) {
            if let Some(text) = lines.get(n as usize) {
                numbered.push((n, text.trim_end_matches('\r').to_owned()));
            }
        }
    }
    let cost = numbered
        .iter()
        .map(|(_, t)| t.chars().count() + 8)
        .sum::<usize>()
        + 160;
    (
        Excerpt {
            id: candidate.id.clone(),
            title: candidate.title.clone(),
            text: String::new(),
            lines: numbered,
            updated: Some(Date::from_local_ms(candidate.updated_at).iso()),
            notebook: None,
        },
        cost,
    )
}

/// Adds `chosen` passages of `candidate` to `excerpts` if they fit.
/// Returns false when the budget is spent.
fn take(
    excerpts: &mut Vec<Excerpt>,
    seen: &mut HashSet<String>,
    used: &mut usize,
    budget: usize,
    candidate: &Candidate,
    chosen: &[&Passage],
) -> bool {
    if chosen.is_empty() || seen.contains(&candidate.id) {
        return true;
    }
    let (excerpt, cost) = excerpt_of(candidate, chosen);
    if excerpt.lines.is_empty() {
        return true;
    }
    if *used + cost > budget && !excerpts.is_empty() {
        return false;
    }
    *used += cost;
    seen.insert(candidate.id.clone());
    excerpts.push(excerpt);
    true
}

impl PageSource for NotebookPage {
    fn doc_id(&self) -> &str {
        &self.id
    }

    fn sections(&self) -> &[SectionSpec] {
        &SPECS
    }

    fn source_notes(&self, store: &Store) -> kanso_store::Result<Vec<SourceNote>> {
        Ok(store
            .home_recent(&self.id, 500)?
            .into_iter()
            .map(|n| SourceNote {
                ai_ok: crate::ask::note_is_eligible(store, &n.id),
                id: n.id,
                title: n.title,
                created_at: n.created_at,
            })
            .collect())
    }

    fn passages_for(
        &self,
        store: &Store,
        section: &str,
        budget: usize,
    ) -> kanso_store::Result<Passages> {
        let mut excerpts = Vec::new();
        let mut seen = HashSet::new();
        let mut used = 0usize;
        match section {
            "about" => {
                // The Home note first, then the opening of each key note,
                // then the opening of the most recently edited notes.
                if let Some(mark) = store.home_note(&self.id)?
                    && let Some(c) = eligible_note(store, &mark.id)?
                {
                    let all = passages(&c.body);
                    let chosen: Vec<&Passage> = all.iter().take(PASSAGES_PER_NOTE).collect();
                    take(&mut excerpts, &mut seen, &mut used, budget, &c, &chosen);
                }
                'fill: {
                    for item in self.key_note_items(store)? {
                        let Some(id) = item.target else { continue };
                        let Some(c) = eligible_note(store, &id)? else {
                            continue;
                        };
                        let all = passages(&c.body);
                        let chosen: Vec<&Passage> = all.iter().take(1).collect();
                        if !take(&mut excerpts, &mut seen, &mut used, budget, &c, &chosen) {
                            break 'fill;
                        }
                    }
                    for (_, c) in self.eligible_recent(store, ABOUT_RECENT)? {
                        let all = passages(&c.body);
                        let chosen: Vec<&Passage> = all.iter().take(1).collect();
                        if !take(&mut excerpts, &mut seen, &mut used, budget, &c, &chosen) {
                            break 'fill;
                        }
                    }
                }
            }
            _ => {
                // Decisions: the lines that sound like one, found here
                // first so the model reads the right ones; the rest of the
                // budget is the openings of recent notes.
                let recent = self.eligible_recent(store, DECISION_SCAN)?;
                'cues: {
                    for (_, c) in &recent {
                        let cues = decision_lines(&c.body);
                        if cues.is_empty() {
                            continue;
                        }
                        let all = passages(&c.body);
                        let mut chosen: Vec<&Passage> = Vec::new();
                        for line in cues {
                            if let Some(p) = all.iter().find(|p| p.start <= line && line <= p.end)
                                && !chosen.iter().any(|c| c.start == p.start)
                            {
                                chosen.push(p);
                            }
                            if chosen.len() >= PASSAGES_PER_NOTE {
                                break;
                            }
                        }
                        if !take(&mut excerpts, &mut seen, &mut used, budget, c, &chosen) {
                            break 'cues;
                        }
                    }
                    for (_, c) in recent.iter().take(ABOUT_RECENT) {
                        let all = passages(&c.body);
                        let chosen: Vec<&Passage> = all.iter().take(1).collect();
                        if !take(&mut excerpts, &mut seen, &mut used, budget, c, &chosen) {
                            break 'cues;
                        }
                    }
                }
            }
        }
        Ok(Passages {
            excerpts,
            based_on: None,
        })
    }

    fn compute(&self, store: &Store, section: &str, all: bool) -> kanso_store::Result<Section> {
        let spec = SPECS
            .iter()
            .find(|s| s.key == section)
            .copied()
            .unwrap_or(SPECS[2]);
        let items = match section {
            "home_note" => match store.home_note(&self.id)? {
                Some(mark) => {
                    let body = store
                        .get(&mark.id)?
                        .and_then(|d| d.fields.get("body").and_then(|b| b.as_str().map(str::to_owned)))
                        .unwrap_or_default();
                    vec![Item {
                        text: home_text(&body).trim().to_owned(),
                        meta: Some(title_or_untitled(&mark.title)),
                        target: Some(mark.id),
                        ..Item::default()
                    }]
                }
                None => Vec::new(),
            },
            "key_notes" => self.key_note_items(store)?,
            "recent" => self.recent_items(store)?,
            "open_tasks" => return self.task_section(store, all),
            "people" => self.people_items(store)?,
            "map" => self.map_items(store)?,
            "inside" => self.inside_items(store)?,
            _ => Vec::new(),
        };
        let total = items.len();
        Ok(Section::computed(&spec, items, total))
    }

    fn ai_instruction(&self, section: &str) -> String {
        let name = kanso_ai::prompt::escape_excerpt(self.name.trim());
        match section {
            "about" => format!(
                "Describe in two to four sentences what the notebook \"{name}\" is for and what is happening in it lately, from these lines only. Write each sentence on its own line starting with '- ', and cite every sentence. Do not list notes. If the lines say nothing about it, reply NONE."
            ),
            _ => format!(
                "List decisions recorded in these lines of the notebook \"{name}\", newest first: what was decided and, if stated, why and when. At most {MAX_DECISIONS}. One per line starting with '- ', each cited. If none, reply NONE."
            ),
        }
    }

    fn max_items(&self, section: &str) -> Option<usize> {
        match section {
            "decisions" => Some(MAX_DECISIONS),
            "about" => Some(4),
            _ => None,
        }
    }
}

/// What opening a homepage needs besides the notebook.
#[derive(Clone, Copy, Debug)]
pub struct HomeOptions {
    /// AI sections are written: AI is on and the person wants summaries.
    pub ai_on: bool,
    pub now: i64,
}

/// Opens notebook `page`'s homepage: computed sections now, AI sections from
/// the cache, in the order the person chose. Sections with nothing to show
/// are left out, except Open tasks when its tasks are all done. A notebook
/// that is locked gives a view marked `locked` and nothing else.
pub fn open_home(
    store: &Store,
    page: &NotebookPage,
    options: HomeOptions,
    show_all: &[&str],
) -> kanso_store::Result<HomeView> {
    if store.book_subtree(&page.id, false)?.is_empty() {
        return Ok(HomeView {
            locked: true,
            ..HomeView::default()
        });
    }
    let mut view = open(
        store,
        page,
        OpenOptions {
            ai_on: options.ai_on,
            now: options.now,
        },
        show_all,
    )?;
    let mut ai_off = false;
    view.sections.retain(|s| match s.kind {
        SectionKind::Ai => {
            let off = matches!(s.state, SectionState::Off(_)) && s.items.is_empty();
            ai_off |= off;
            !off
        }
        SectionKind::Computed => !s.items.is_empty() || s.note.is_some(),
    });
    let layout = store.page_cache(&page.id)?.layout;
    if let Some(layout) = layout {
        let rank = |key: &str| {
            layout
                .order
                .iter()
                .position(|k| k == key)
                .or_else(|| {
                    SECTION_KEYS
                        .iter()
                        .position(|k| *k == key)
                        .map(|p| layout.order.len() + p)
                })
                .unwrap_or(usize::MAX)
        };
        view.sections.sort_by_key(|s| rank(&s.key));
    }
    Ok(HomeView {
        header: page.header(store)?,
        page: view,
        footer: ai_off.then(|| AI_OFF_LINE.to_owned()),
        locked: false,
        ai_on: options.ai_on,
    })
}
