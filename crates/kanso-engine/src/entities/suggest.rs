//! Suggestions: running detection over the library in the background,
//! keeping what the person turned away (and syncing "never" through the
//! notebook the pages live in), and listing what is left to offer.
//!
//! Detection reads only notes that are not locked, in the trash or tagged
//! `#ai-exclude`, writes nothing but the local suggestions table, and
//! never makes a page.

use std::collections::HashSet;

use kanso_core::entities::{self, EntityKind, normalize_name};
use kanso_store::{CandidateRow, Store};

use super::detect::{self, Context, Input, SNOOZE_EXTRA};
use super::settings::{DEFAULT_NOTEBOOK, EntitySettings};

/// Detection runs at most this often, in milliseconds.
pub const MIN_GAP_MS: i64 = 10 * 60 * 1000;

/// A suggestion as the list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// The name as it is most often written.
    pub name: String,
    /// The key dismissals use.
    pub key: String,
    pub kind: EntityKind,
    pub notes: u32,
    /// "28 Oct".
    pub last_seen: Option<String>,
    /// Other ways it is written ("Sam", "@sam"), without the name itself.
    pub also: Vec<String>,
    /// The page this may be another name of: `(note ID, title)`.
    pub alias_of: Option<(String, String)>,
}

impl Suggestion {
    /// "person · mentioned in 14 notes, last on 28 Oct".
    pub fn detail(&self) -> String {
        let notes = format!(
            "{} {} {}",
            if self.alias_of.is_some() { "in" } else { "mentioned in" },
            self.notes,
            if self.notes == 1 { "note" } else { "notes" }
        );
        match &self.last_seen {
            Some(day) => format!("{} \u{b7} {notes}, last on {day}", self.kind.as_str()),
            None => format!("{} \u{b7} {notes}", self.kind.as_str()),
        }
    }

    /// `Also written as "Sam", "@sam".`
    pub fn also_line(&self) -> Option<String> {
        (!self.also.is_empty()).then(|| {
            format!(
                "Also written as {}.",
                self.also
                    .iter()
                    .map(|f| format!("\u{201c}{f}\u{201d}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

/// The notebook dismissals sync in: the chosen one or the default, when it
/// exists. Never made for this.
pub fn dismissal_book(store: &Store, settings: &EntitySettings) -> Option<String> {
    let books = store.books().ok()?;
    if let Some(chosen) = &settings.keep_in
        && books.iter().any(|b| &b.id == chosen)
    {
        return Some(chosen.clone());
    }
    books
        .iter()
        .find(|b| b.name.trim() == DEFAULT_NOTEBOOK)
        .map(|b| b.id.clone())
}

/// Brings "never" dismissals and the notebook's list together: names turned
/// away on another device arrive, and names turned away here that the
/// notebook lacks are written to it. Until the notebook exists, they stay
/// local.
pub fn sync_dismissals(store: &Store, settings: &EntitySettings, now: i64) -> kanso_store::Result<()> {
    let Some(book) = dismissal_book(store, settings) else {
        return Ok(());
    };
    let local: HashSet<String> = store
        .dismissed()?
        .into_iter()
        .filter(|d| d.kind == "never")
        .map(|d| d.name)
        .collect();
    let remote: HashSet<String> = store.book_dismissed(&book)?.into_iter().collect();
    for name in remote.difference(&local) {
        store.dismiss_name(name, true, 0, now)?;
    }
    if !local.is_subset(&remote) {
        let mut all: Vec<String> = remote.union(&local).cloned().collect();
        all.sort();
        store.set_book_dismissed(&book, &all)?;
    }
    Ok(())
}

/// What detection should know about this library.
pub fn context(store: &Store) -> kanso_store::Result<Context> {
    let mut ctx = Context::default();
    for (_, title) in store.note_titles()? {
        ctx.known_titles.insert(normalize_name(&title));
    }
    for page in store.entities()? {
        for name in &page.names {
            ctx.entity_names.insert(name.clone(), page.id.clone());
        }
    }
    for tag in store.tags()? {
        ctx.excluded.insert(normalize_name(&tag.name));
    }
    for book in store.books()? {
        ctx.excluded.insert(normalize_name(&book.name));
    }
    for row in store.dismissed()? {
        if row.kind == "never" {
            ctx.excluded.insert(row.name);
        } else {
            ctx.snoozed.insert(row.name, row.notes);
        }
    }
    Ok(ctx)
}

/// Runs detection over the library and stores what it finds, unless it ran
/// less than [`MIN_GAP_MS`] ago (`last_run`, which it updates). Returns how
/// many suggestions there are, or `None` when it did not run.
pub fn refresh(
    store: &Store,
    settings: &EntitySettings,
    now: i64,
    last_run: &mut i64,
    force: bool,
) -> kanso_store::Result<Option<usize>> {
    if !settings.suggest || (!force && now - *last_run < MIN_GAP_MS && *last_run != 0) {
        return Ok(None);
    }
    *last_run = now;
    sync_dismissals(store, settings, now)?;
    let ctx = context(store)?;
    let mut detector = detect::Detector::new(ctx);
    store.for_each_detection_note(|note| {
        // A page's own text is not evidence of anyone.
        if entities::kind_of(&note.body).is_some() {
            return;
        }
        detector.feed(&Input {
            title: &note.title,
            body: &note.body,
            created_at: note.created_at,
            tags: &note.tags,
        });
    })?;
    let rows = detector.finish();
    store.set_candidates(&rows)?;
    Ok(Some(rows.len()))
}

/// The suggestions to offer now: stored candidates minus anything since
/// turned away, named by a page, or snoozed and not yet grown.
pub fn suggestions(store: &Store, settings: &EntitySettings) -> kanso_store::Result<Vec<Suggestion>> {
    if !settings.suggest {
        return Ok(Vec::new());
    }
    let dismissed = store.dismissed()?;
    let pages = store.entities()?;
    let mut out = Vec::new();
    for row in store.candidates()? {
        if pages.iter().any(|p| p.names.contains(&row.name)) {
            continue;
        }
        if let Some(d) = dismissed.iter().find(|d| d.name == row.name)
            && (d.kind == "never" || row.notes < d.notes + SNOOZE_EXTRA)
        {
            continue;
        }
        let name = row
            .forms
            .first()
            .map(|f| f.trim_start_matches('@').to_owned())
            .unwrap_or_else(|| row.name.clone());
        let also: Vec<String> = row
            .forms
            .iter()
            .skip(1)
            .filter(|f| normalize_name(f) != normalize_name(&name))
            .cloned()
            .collect();
        let alias_of = row
            .alias_of
            .as_ref()
            .and_then(|id| pages.iter().find(|p| &p.id == id))
            .map(|p| (p.id.clone(), p.title.clone()));
        out.push(Suggestion {
            key: row.name.clone(),
            name,
            kind: row.kind,
            notes: row.notes,
            last_seen: super::page::date_label(row.last_seen),
            also,
            alias_of,
        });
    }
    Ok(out)
}

/// How the header counts them: "3 new".
pub fn count_label(count: usize) -> Option<String> {
    (count > 0).then(|| format!("{count} new"))
}

/// Turns a suggestion away: for good (`never`) or until it has five more
/// notes. The "never" list also goes to the notebook when it exists.
pub fn dismiss(
    store: &Store,
    settings: &EntitySettings,
    suggestion: &Suggestion,
    never: bool,
    now: i64,
) -> kanso_store::Result<()> {
    store.dismiss_name(&suggestion.key, never, suggestion.notes, now)?;
    if never {
        sync_dismissals(store, settings, now)?;
    }
    Ok(())
}

/// Takes a dismissed name back ("Suggest again"), here and in the notebook.
pub fn suggest_again(
    store: &Store,
    settings: &EntitySettings,
    name: &str,
    now: i64,
) -> kanso_store::Result<()> {
    store.undismiss_name(name)?;
    if let Some(book) = dismissal_book(store, settings) {
        let key = normalize_name(name);
        let kept: Vec<String> = store
            .book_dismissed(&book)?
            .into_iter()
            .filter(|n| normalize_name(n) != key)
            .collect();
        store.set_book_dismissed(&book, &kept)?;
    }
    let _ = now;
    Ok(())
}

/// A candidate as a row for tests and callers that build their own.
pub fn candidate(name: &str, kind: EntityKind, notes: u32) -> CandidateRow {
    CandidateRow {
        name: normalize_name(name),
        kind,
        notes,
        last_seen: 0,
        forms: vec![name.to_owned()],
        score: f64::from(notes),
        alias_of: None,
    }
}
