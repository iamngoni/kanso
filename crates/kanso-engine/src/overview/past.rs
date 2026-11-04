//! *A note from the past*: at most one note, at the end of the page, shown
//! only when a candidate clears a confidence bar and the page can name its
//! evidence. Chosen once a day and cached, so it does not change as you
//! write; *Not useful* removes it until tomorrow.

use std::collections::HashSet;

use kanso_core::dates::Date;
use kanso_core::properties;
use kanso_core::roles::daily_id;
use kanso_store::{Result, SIMILARITY_CLOSER, Store};
use serde::{Deserialize, Serialize};

use super::model::PastNote;
use crate::brief::Now;

const DAY_MS: i64 = 86_400_000;
/// A meaning neighbour must be at least this close.
pub const MIN_SIMILARITY: f32 = SIMILARITY_CLOSER;
/// ... and this old, in days, with no change since.
pub const MIN_AGE_DAYS: i64 = 30;
/// ... and hold this many words of its own.
pub const MIN_WORDS: usize = 60;
/// An anniversary daily note scores this.
pub const ANNIVERSARY_SCORE: f32 = 0.62;
/// The most seeds searched.
pub const MAX_SEEDS: usize = 5;
const EXCERPT_CHARS: usize = 190;

/// A note today's page is about, and how to say so.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seed {
    pub note_id: String,
    pub why: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Cached {
    /// A fingerprint of the seeds it was chosen for.
    seeds: String,
    note: Option<PastNote>,
    dismissed: bool,
}

fn cache_key(day: &str) -> String {
    format!("overview.past.{day}")
}

fn fingerprint(seeds: &[Seed]) -> String {
    seeds
        .iter()
        .map(|s| s.note_id.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

fn strip_front_matter(body: &str) -> &str {
    properties::split(body).map_or(body, |f| &body[f.body_start..])
}

/// The words of a note's own text: its body without front matter, headings
/// and fences.
pub fn own_words(body: &str) -> usize {
    let text = strip_front_matter(body);
    let mut fenced = false;
    let mut words = 0;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced || trimmed.starts_with('#') {
            continue;
        }
        words += line.split_whitespace().count();
    }
    words
}

fn first_paragraph(body: &str) -> String {
    let text = strip_front_matter(body);
    let para: Vec<&str> = text
        .lines()
        .skip_while(|l| l.trim().is_empty() || l.trim_start().starts_with('#'))
        .take_while(|l| !l.trim().is_empty())
        .collect();
    let joined = para.join(" ");
    let joined = joined.trim();
    if joined.chars().count() <= EXCERPT_CHARS {
        joined.to_owned()
    } else {
        let mut cut: String = joined.chars().take(EXCERPT_CHARS).collect();
        if let Some(i) = cut.rfind(' ') {
            cut.truncate(i);
        }
        format!("{cut}\u{2026}")
    }
}

fn sub_months(date: Date, months: u32) -> Date {
    let index = i64::from(date.year) * 12 + (i64::from(date.month) - 1) - i64::from(months);
    let year = index.div_euclid(12) as i32;
    let month = index.rem_euclid(12) as u32 + 1;
    let day = date
        .day
        .min(kanso_core::dates::days_in_month(year, month));
    Date::new(year, month, day).unwrap_or(date)
}

fn anniversary_label(months: u32) -> &'static str {
    match months {
        12 => "A year ago today",
        6 => "Six months ago today",
        _ => "A month ago today",
    }
}

/// The one note from the past for today, or none. `model` names the
/// embedding model in use; without one only anniversaries are considered.
pub fn past_note(
    store: &Store,
    now: Now,
    model: Option<&str>,
    seeds: &[Seed],
) -> Result<Option<PastNote>> {
    let today = now.today();
    let day = today.iso();
    let key = cache_key(&day);
    let print = fingerprint(seeds);
    if let Some(cached) = store
        .meta(&key)?
        .and_then(|raw| serde_json::from_str::<Cached>(&raw).ok())
    {
        if cached.dismissed {
            return Ok(None);
        }
        if cached.note.is_some() || cached.seeds == print {
            return Ok(cached.note);
        }
    }
    let chosen = choose(store, now, today, model, seeds)?;
    let cached = Cached {
        seeds: print,
        note: chosen.clone(),
        dismissed: false,
    };
    store.set_meta(&key, &serde_json::to_string(&cached).unwrap_or_default())?;
    Ok(chosen)
}

fn choose(
    store: &Store,
    now: Now,
    today: Date,
    model: Option<&str>,
    seeds: &[Seed],
) -> Result<Option<PastNote>> {
    let seeds: Vec<&Seed> = seeds.iter().take(MAX_SEEDS).collect();
    let seed_ids: HashSet<&str> = seeds.iter().map(|s| s.note_id.as_str()).collect();
    let mut linked: HashSet<String> = HashSet::new();
    for seed in &seeds {
        linked.extend(store.linked_note_ids(&seed.note_id)?);
    }
    // (id, score, why, seeds it was chosen for)
    let mut candidates: Vec<(String, f32, String, Vec<String>)> = Vec::new();
    if let Some(model) = model {
        for seed in &seeds {
            let dismissed = store.dismissed_related(&seed.note_id)?;
            let Some(near) = store.related_notes(&seed.note_id, model, 4)? else {
                continue;
            };
            for n in near {
                if n.score < MIN_SIMILARITY
                    || dismissed.contains(&n.id)
                    || linked.contains(&n.id)
                    || seed_ids.contains(n.id.as_str())
                {
                    continue;
                }
                candidates.push((n.id, n.score, seed.why.clone(), vec![seed.note_id.clone()]));
            }
        }
    }
    let daily = daily_id(today);
    for months in [12u32, 6, 1] {
        let id = daily_id(sub_months(today, months));
        if id != daily && !linked.contains(&id) {
            candidates.push((id, ANNIVERSARY_SCORE, anniversary_label(months).to_owned(), Vec::new()));
        }
    }
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen = HashSet::new();
    for (id, score, why, for_seeds) in candidates {
        if !seen.insert(id.clone()) {
            continue;
        }
        let anniversary = for_seeds.is_empty();
        // Locked, trashed and `#ai-exclude` notes are not returned.
        let Some(note) = store.brief_notes(std::slice::from_ref(&id))?.into_iter().next() else {
            continue;
        };
        if !anniversary && now.utc_ms - note.updated_at < MIN_AGE_DAYS * DAY_MS {
            continue;
        }
        let Some(doc) = store.get(&id)? else { continue };
        let body = doc
            .fields
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if own_words(body) < MIN_WORDS {
            continue;
        }
        return Ok(Some(PastNote {
            note_id: id,
            title: note.title,
            excerpt: first_paragraph(body),
            why,
            score,
            seeds: for_seeds,
        }));
    }
    Ok(None)
}

/// *Not useful*: removes today's note and offers no replacement until
/// tomorrow. The note is also remembered as dismissed for the seeds it was
/// chosen for, so it is not offered for them again.
pub fn dismiss_past(store: &Store, now: Now, note: &PastNote) -> Result<()> {
    for seed in &note.seeds {
        store.dismiss_related(seed, &note.note_id)?;
    }
    let cached = Cached {
        seeds: String::new(),
        note: None,
        dismissed: true,
    };
    store.set_meta(
        &cache_key(&now.today().iso()),
        &serde_json::to_string(&cached).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_words_skip_the_template() {
        let body = "---\nkanso: person\n---\n# Title\n\none two three\n\n```\ncode here\n```\nfour";
        assert_eq!(own_words(body), 4);
        assert_eq!(sub_months(Date::new(2026, 3, 31).unwrap(), 1).iso(), "2026-02-28");
        assert_eq!(sub_months(Date::new(2026, 11, 4).unwrap(), 12).iso(), "2025-11-04");
    }

    #[test]
    fn the_first_paragraph_is_cut_at_a_word() {
        let body = format!("# T\n\n{}", "word ".repeat(80));
        let got = first_paragraph(&body);
        assert!(got.ends_with('\u{2026}'));
        assert!(got.chars().count() <= EXCERPT_CHARS + 1);
    }
}
