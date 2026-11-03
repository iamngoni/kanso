//! A person's or a project's page as a [`PageSource`]: its open items,
//! promises (decisions for a project), what was discussed, a timeline and
//! what is related.

use std::collections::HashMap;

use kanso_ai::Excerpt;
use kanso_core::dates::Date;
use kanso_core::entities::{EntityKind, own_text};
use kanso_core::links::{line_text, plain_line};
use kanso_store::{MentionRow, Store};

use crate::ask::Receipt;
use crate::ask::receipts::{MAX_RANGE, short_date};
use crate::pages::{
    Item, PASSAGES_PER_NOTE, PageSource, Passages, Section, SectionKind, SectionSpec, SourceNote,
    TaskRef,
};

/// Lines shown before "Show all".
pub const FIRST_LINES: usize = 6;
/// The most lines a computed section holds.
pub const MAX_LINES: usize = 200;
/// The most open items shown.
pub const MAX_OPEN_ITEMS: usize = 40;
/// The most related pages and notes of each kind.
pub const MAX_RELATED: usize = 8;
/// Notes a page's sources list holds.
pub const MAX_SOURCES: usize = 500;
/// Mentions beyond which an AI section says how many it read.
pub const MANY_MENTIONS: usize = 1_000;
/// How long a timeline line gets, in characters.
const LINE_CHARS: usize = 110;

const PERSON: [SectionSpec; 5] = [
    SectionSpec {
        key: "open_items",
        title: "Open items",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "promises",
        title: "Promises",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "discussed",
        title: "What you discussed",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "timeline",
        title: "Timeline",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "related",
        title: "Related",
        kind: SectionKind::Computed,
    },
];

const PROJECT: [SectionSpec; 5] = [
    SectionSpec {
        key: "open_items",
        title: "Open items",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "decisions",
        title: "Decisions",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "discussed",
        title: "What you discussed",
        kind: SectionKind::Ai,
    },
    SectionSpec {
        key: "timeline",
        title: "Timeline",
        kind: SectionKind::Computed,
    },
    SectionSpec {
        key: "related",
        title: "Related",
        kind: SectionKind::Computed,
    },
];

/// A person's or project's page.
#[derive(Clone, Debug)]
pub struct EntityPage {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    /// Normalized names, title first.
    pub names: Vec<String>,
}

/// The header of an open page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageHeader {
    pub title: String,
    pub kind: EntityKind,
    /// The other names, as written.
    pub aliases: Vec<String>,
    pub notes: u32,
    /// "28 Oct", when anything mentions it.
    pub last_mentioned: Option<String>,
}

impl EntityPage {
    /// The page note `id` is, when it is one that can be read.
    pub fn load(store: &Store, id: &str) -> kanso_store::Result<Option<EntityPage>> {
        Ok(store.entity(id)?.map(|row| EntityPage {
            id: row.id,
            kind: row.kind,
            title: row.title,
            names: row.names,
        }))
    }

    /// The header: name, other names, how many notes, the latest date.
    pub fn header(&self, store: &Store) -> kanso_store::Result<PageHeader> {
        let body = store
            .get(&self.id)?
            .and_then(|d| {
                d.fields
                    .get("body")
                    .and_then(|b| b.as_str().map(str::to_owned))
            })
            .unwrap_or_default();
        let (notes, last) = store.entity_mention_stats(&self.id)?;
        Ok(PageHeader {
            title: self.title.clone(),
            kind: self.kind,
            aliases: kanso_core::entities::written_aliases(&self.title, &body),
            notes,
            last_mentioned: (notes > 0).then(|| date_label(last)).flatten(),
        })
    }

    /// What the person wrote on the page, without its front matter.
    pub fn own_text(&self, store: &Store) -> kanso_store::Result<String> {
        Ok(store
            .get(&self.id)?
            .and_then(|d| {
                d.fields
                    .get("body")
                    .and_then(|b| b.as_str().map(|b| own_text(b).to_owned()))
            })
            .unwrap_or_default())
    }

    fn display(&self) -> String {
        self.title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(['<', '>'], "")
    }
}

/// "28 Oct" for a time in milliseconds since the epoch.
pub fn date_label(ms: i64) -> Option<String> {
    short_date(&Date::from_local_ms(ms).iso())
}

/// The date a note counts as for a timeline: a daily note's title date,
/// else when it was made.
pub fn note_date(title: &str, created_at: i64) -> Date {
    title
        .trim()
        .get(..10)
        .and_then(Date::parse_iso)
        .unwrap_or_else(|| Date::from_local_ms(created_at))
}

/// A line shortened to plain words: markers out, cut at a word.
pub fn shorten(line: &str) -> String {
    let plain = plain_line(line);
    let plain = plain
        .trim_start_matches("[ ] ")
        .trim_start_matches("[x] ")
        .trim_start_matches("[X] ")
        .trim();
    if plain.chars().count() <= LINE_CHARS {
        return plain.to_owned();
    }
    let cut: String = plain.chars().take(LINE_CHARS - 1).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{}\u{2026}", cut.trim_end())
}

/// Note bodies read once while a section is built.
struct Bodies<'a> {
    store: &'a Store,
    cache: HashMap<String, String>,
}

impl<'a> Bodies<'a> {
    fn new(store: &'a Store) -> Self {
        Bodies {
            store,
            cache: HashMap::new(),
        }
    }

    fn body(&mut self, id: &str) -> &str {
        if !self.cache.contains_key(id) {
            let body = self
                .store
                .get(id)
                .ok()
                .flatten()
                .and_then(|d| {
                    d.fields
                        .get("body")
                        .and_then(|b| b.as_str().map(str::to_owned))
                })
                .unwrap_or_default();
            self.cache.insert(id.to_owned(), body);
        }
        &self.cache[id]
    }

    fn line(&mut self, id: &str, line: u32) -> String {
        line_text(self.body(id), line).trim_end().to_owned()
    }
}

/// A receipt for one line of a note, as it reads now.
fn receipt(
    n: usize,
    note_id: &str,
    title: &str,
    updated: Option<String>,
    line: u32,
    text: &str,
) -> Receipt {
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

impl PageSource for EntityPage {
    fn doc_id(&self) -> &str {
        &self.id
    }

    fn sections(&self) -> &[SectionSpec] {
        match self.kind {
            EntityKind::Person => &PERSON,
            EntityKind::Project => &PROJECT,
        }
    }

    fn source_notes(&self, store: &Store) -> kanso_store::Result<Vec<SourceNote>> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for m in store.entity_mentions(&self.id, MAX_SOURCES * 4)? {
            if seen.insert(m.note_id.clone()) {
                out.push(SourceNote {
                    id: m.note_id,
                    title: m.title,
                    created_at: m.created_at,
                    ai_ok: m.ai_ok,
                });
            }
            if out.len() >= MAX_SOURCES {
                break;
            }
        }
        for (id, title) in store.entity_related_notes(&self.id, MAX_RELATED)? {
            if seen.insert(id.clone()) {
                out.push(SourceNote {
                    id,
                    title,
                    created_at: 0,
                    ai_ok: false,
                });
            }
        }
        Ok(out)
    }

    fn passages_for(
        &self,
        store: &Store,
        _section: &str,
        budget: usize,
    ) -> kanso_store::Result<Passages> {
        let mentions: Vec<MentionRow> = store
            .entity_mentions(&self.id, MANY_MENTIONS * 2)?
            .into_iter()
            .filter(|m| m.ai_ok)
            .collect();
        let total = mentions.len();
        let mut order: Vec<String> = Vec::new();
        let mut by_note: HashMap<String, Vec<&MentionRow>> = HashMap::new();
        for m in &mentions {
            by_note.entry(m.note_id.clone()).or_insert_with(|| {
                order.push(m.note_id.clone());
                Vec::new()
            });
            if let Some(v) = by_note.get_mut(&m.note_id) {
                v.push(m);
            }
        }
        let mut excerpts = Vec::new();
        let mut used = 0usize;
        let mut taken_mentions = 0usize;
        'notes: for note_id in &order {
            let rows = &by_note[note_id];
            let Some(doc) = store.get(note_id)? else {
                continue;
            };
            let body = doc
                .fields
                .get("body")
                .and_then(|b| b.as_str())
                .unwrap_or("")
                .to_owned();
            let all = kanso_core::passages::passages(&body);
            let lines: Vec<&str> = body.split('\n').collect();
            let mut chosen: Vec<&kanso_core::passages::Passage> = Vec::new();
            for row in rows {
                if let Some(p) = all
                    .iter()
                    .find(|p| p.start <= row.line && row.line <= p.end)
                    && !chosen.iter().any(|c| c.start == p.start)
                {
                    chosen.push(p);
                }
                if chosen.len() >= PASSAGES_PER_NOTE {
                    break;
                }
            }
            if chosen.is_empty() {
                continue;
            }
            chosen.sort_by_key(|p| p.start);
            let mut numbered: Vec<(u32, String)> = Vec::new();
            for p in &chosen {
                for n in p.start..=p.end.min(p.start + MAX_RANGE * 4) {
                    if let Some(text) = lines.get(n as usize) {
                        numbered.push((n, text.trim_end_matches('\r').to_owned()));
                    }
                }
            }
            let cost: usize = numbered
                .iter()
                .map(|(_, t)| t.chars().count() + 8)
                .sum::<usize>()
                + 160;
            if used + cost > budget && !excerpts.is_empty() {
                break 'notes;
            }
            used += cost;
            taken_mentions += rows.len();
            excerpts.push(Excerpt {
                id: note_id.clone(),
                title: rows[0].title.clone(),
                text: String::new(),
                lines: numbered,
                updated: date_label_iso(rows[0].updated_at),
                notebook: None,
            });
        }
        let based_on =
            (total >= MANY_MENTIONS && taken_mentions < total).then_some(taken_mentions as u32);
        Ok(Passages { excerpts, based_on })
    }

    fn compute(&self, store: &Store, section: &str, all: bool) -> kanso_store::Result<Section> {
        let spec = self
            .sections()
            .iter()
            .find(|s| s.key == section)
            .copied()
            .unwrap_or(SectionSpec {
                key: "open_items",
                title: "Open items",
                kind: SectionKind::Computed,
            });
        let mut bodies = Bodies::new(store);
        match section {
            "open_items" => {
                let tasks = store.entity_tasks(&self.id, MAX_OPEN_ITEMS)?;
                let items: Vec<Item> = tasks
                    .into_iter()
                    .map(|t| {
                        let line = bodies.line(&t.note_id, t.line);
                        Item {
                            text: kanso_core::links::plain_line(&t.text),
                            meta: Some(format!(
                                "{}{}",
                                t.title_or_untitled(),
                                date_label(t.note_updated_at)
                                    .map(|d| format!(" \u{b7} {d}"))
                                    .unwrap_or_default()
                            )),
                            sources: vec![receipt(
                                1,
                                &t.note_id,
                                &t.note_title,
                                date_label_iso(t.note_updated_at),
                                t.line,
                                &line,
                            )],
                            task: Some(TaskRef {
                                note_id: t.note_id,
                                line: t.line,
                                text: t.text,
                                checked: false,
                            }),
                            ..Item::default()
                        }
                    })
                    .collect();
                let total = items.len();
                Ok(Section::computed(&spec, items, total))
            }
            "timeline" => {
                let mentions = store.entity_mentions(&self.id, MAX_LINES)?;
                let total = store.entity_mention_stats(&self.id)?.0 as usize;
                let mut rows: Vec<(Date, MentionRow)> = mentions
                    .into_iter()
                    .map(|m| (note_date(&m.title, m.created_at), m))
                    .collect();
                rows.sort_by(|a, b| {
                    b.0.cmp(&a.0)
                        .then(a.1.note_id.cmp(&b.1.note_id))
                        .then(a.1.line.cmp(&b.1.line))
                });
                let limit = if all { MAX_LINES } else { FIRST_LINES };
                let items: Vec<Item> = rows
                    .into_iter()
                    .take(limit)
                    .map(|(date, m)| {
                        let line = bodies.line(&m.note_id, m.line);
                        Item {
                            text: format!(
                                "{} \u{2014} \u{201c}{}\u{201d}",
                                if m.title.trim().is_empty() {
                                    "Untitled"
                                } else {
                                    m.title.trim()
                                },
                                shorten(&line)
                            ),
                            meta: short_date(&date.iso()),
                            group: Some(format!(
                                "{} {}",
                                kanso_core::dates::MONTHS[date.month as usize - 1],
                                date.year
                            )),
                            sources: vec![receipt(
                                1,
                                &m.note_id,
                                &m.title,
                                short_iso(date),
                                m.line,
                                &line,
                            )],
                            shared: m.shared,
                            ..Item::default()
                        }
                    })
                    .collect();
                Ok(Section::computed(&spec, items, total))
            }
            "related" => {
                let mut items = Vec::new();
                let people = store.co_entities(&self.id, MAX_RELATED * 2)?;
                let (mut first, mut second): (Vec<_>, Vec<_>) =
                    people.into_iter().partition(|(e, _)| {
                        (e.kind == EntityKind::Person) == (self.kind == EntityKind::Project)
                    });
                // A project lists its people first; a person its projects.
                first.truncate(MAX_RELATED);
                second.truncate(MAX_RELATED);
                for (entity, n) in first.into_iter().chain(second) {
                    items.push(Item {
                        text: entity.title.clone(),
                        target: Some(entity.id.clone()),
                        tag: Some(entity.kind.as_str().to_owned()),
                        meta: Some(format!("{n} {}", if n == 1 { "note" } else { "notes" })),
                        ..Item::default()
                    });
                }
                for (id, title) in store.entity_related_notes(&self.id, MAX_RELATED)? {
                    let page = store.entity(&id)?;
                    items.push(Item {
                        text: if title.trim().is_empty() {
                            "Untitled".into()
                        } else {
                            title
                        },
                        target: Some(id),
                        tag: page.map(|p| p.kind.as_str().to_owned()),
                        ..Item::default()
                    });
                }
                let total = items.len();
                Ok(Section::computed(&spec, items, total))
            }
            _ => Ok(Section::computed(&spec, Vec::new(), 0)),
        }
    }

    fn ai_instruction(&self, section: &str) -> String {
        let name = self.display();
        let what = match self.kind {
            EntityKind::Person => format!("with {name}"),
            EntityKind::Project => format!("about the project {name}"),
        };
        match section {
            "promises" => format!(
                "List commitments between the user and {name}. Each item: who promised what to whom, and by when if stated. Begin each item with 'YOU:' if the user promised it, or 'THEM:' if {name} promised it. Use only these lines. One item per line starting with '- ', each with its citations. If there are none, reply NONE."
            ),
            "decisions" => format!(
                "List the decisions made {what}, such as 'We chose X over Y'. Each item states the decision and, if given, why. Use only these lines. One item per line starting with '- ', each with its citations. If there are none, reply NONE."
            ),
            _ => format!(
                "Summarise what the user discussed {what}: one topic per item, in a short sentence that says what was said or decided. Use only these lines. One item per line starting with '- ', each with its citations. If there is nothing, reply NONE."
            ),
        }
    }
}

trait TaskTitle {
    fn title_or_untitled(&self) -> String;
}

impl TaskTitle for kanso_store::EntityTask {
    fn title_or_untitled(&self) -> String {
        if self.note_title.trim().is_empty() {
            "Untitled".into()
        } else {
            self.note_title.trim().to_owned()
        }
    }
}

fn date_label_iso(ms: i64) -> Option<String> {
    Some(Date::from_local_ms(ms).iso())
}

fn short_iso(date: Date) -> Option<String> {
    Some(date.iso())
}
