//! The generated-page core: a page is a list of sections, each computed from
//! the derived tables or written by AI from numbered lines with receipts.
//!
//! It knows nothing about people or notebooks: a page kind supplies a
//! [`PageSource`]. People and project pages (`entities::page`) and notebook
//! homepages are both built on it.
//!
//! Computed sections are built on open and never stored. AI sections are
//! written by [`write::write_section`], checked with Ask's receipts (an item
//! whose lines fail the check is dropped, not flagged), and cached in the
//! page's own document (`kansoPage`, see [`kanso_core::page_cache`]) with a
//! fingerprint of the notes they were written from, so an out-of-date
//! section can say so. Nothing here reads a clock: the caller passes `now`.

pub mod model;
pub mod write;

use std::collections::{BTreeSet, HashMap, HashSet};

use kanso_ai::Excerpt;
use kanso_core::page_cache::{CachedSection, CachedSource, not_right_key};
use kanso_store::Store;
use sha2::{Digest, Sha256};

pub use model::*;

use crate::ask::{Receipt, note_is_eligible};

/// How much an AI section may read, in characters.
pub const PAGE_BUDGET: usize = 16_000;
/// Passages one note may give a section.
pub const PASSAGES_PER_NOTE: usize = 3;
/// A section is rewritten on open at most this often, in milliseconds.
pub const AUTO_REFRESH_GAP_MS: i64 = 60 * 60 * 1000;

/// What a section may read, and what it left out.
#[derive(Clone, Debug, Default)]
pub struct Passages {
    pub excerpts: Vec<Excerpt>,
    /// How many mentions it was based on when there were more than it
    /// could read.
    pub based_on: Option<u32>,
}

/// A kind of page: where its lines come from.
pub trait PageSource {
    /// The document the page's cache lives on.
    fn doc_id(&self) -> &str;
    /// The sections, in order.
    fn sections(&self) -> &[SectionSpec];
    /// The notes this page draws on, eligible ones only, newest first.
    fn source_notes(&self, store: &Store) -> kanso_store::Result<Vec<SourceNote>>;
    /// The passages an AI section may read, with line numbers, only from
    /// notes that may be sent to AI, newest first, within `budget`.
    fn passages_for(
        &self,
        store: &Store,
        section: &str,
        budget: usize,
    ) -> kanso_store::Result<Passages>;
    /// Builds one computed section. With `all`, every line instead of the
    /// first few.
    fn compute(&self, store: &Store, section: &str, all: bool) -> kanso_store::Result<Section>;
    /// The instruction for one AI section.
    fn ai_instruction(&self, section: &str) -> String;
}

fn spec_of<'a>(source: &'a dyn PageSource, key: &str) -> Option<&'a SectionSpec> {
    source.sections().iter().find(|spec| spec.key == key)
}

/// A short hash of a note's text, for the list of notes a section was
/// written from.
pub fn note_hash(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    digest.iter().take(4).map(|b| format!("{b:02x}")).collect()
}

/// The fingerprint of a list of `(note id, hash)`: the same list in any
/// order gives the same fingerprint, and any note added, removed or edited
/// gives another.
pub fn fingerprint(inputs: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = inputs.iter().collect();
    sorted.sort();
    sorted.dedup();
    let mut hasher = Sha256::new();
    for (id, hash) in sorted {
        hasher.update(id.as_bytes());
        hasher.update([0]);
        hasher.update(hash.as_bytes());
        hasher.update([1]);
    }
    let hex: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

/// How many notes were added, removed or edited between two lists.
pub fn changed_between(old: &[(String, String)], new: &[(String, String)]) -> usize {
    let old: HashMap<&str, &str> = old.iter().map(|(i, h)| (i.as_str(), h.as_str())).collect();
    let new: HashMap<&str, &str> = new.iter().map(|(i, h)| (i.as_str(), h.as_str())).collect();
    let edited_or_removed = old
        .iter()
        .filter(|(id, hash)| new.get(*id) != Some(hash))
        .count();
    let added = new.keys().filter(|id| !old.contains_key(*id)).count();
    edited_or_removed + added
}

/// The `(note id, hash)` list an AI section would be written from now.
pub fn current_inputs(
    store: &Store,
    source: &dyn PageSource,
    section: &str,
) -> kanso_store::Result<Vec<(String, String)>> {
    let passages = source.passages_for(store, section, PAGE_BUDGET)?;
    inputs_of(store, &passages.excerpts)
}

/// The `(note id, hash)` list of the notes `excerpts` come from.
pub fn inputs_of(
    store: &Store,
    excerpts: &[Excerpt],
) -> kanso_store::Result<Vec<(String, String)>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for excerpt in excerpts {
        if !seen.insert(excerpt.id.clone()) {
            continue;
        }
        let body = store
            .get(&excerpt.id)?
            .and_then(|doc| {
                doc.fields
                    .get("body")
                    .and_then(|b| b.as_str().map(str::to_owned))
            })
            .unwrap_or_default();
        out.push((excerpt.id.clone(), note_hash(&body)));
    }
    Ok(out)
}

/// A cached source as a receipt.
pub fn receipt_of(n: usize, source: &CachedSource) -> Receipt {
    Receipt {
        n,
        note_id: source.note.clone(),
        title: source.title.clone(),
        updated: source.updated.clone(),
        start: source.start,
        end: source.end,
        quote: source.quote.clone(),
    }
}

/// A receipt as a cached source.
pub fn cached_source(receipt: &Receipt) -> CachedSource {
    CachedSource {
        note: receipt.note_id.clone(),
        start: receipt.start,
        end: receipt.end,
        quote: receipt.quote.clone(),
        title: receipt.title.clone(),
        updated: receipt.updated.clone(),
    }
}

/// What opening a page needs to know about the person's settings.
#[derive(Clone, Copy, Debug)]
pub struct OpenOptions {
    /// AI is on and a model is chosen.
    pub ai_on: bool,
    pub now: i64,
}

/// Opens a page: its computed sections now, its AI sections from the cache
/// with their state against the notes as they are now. Lines whose sources
/// are no longer allowed to feed AI (locked, in the trash, kept out) are
/// dropped from the shown section before anything else, and the cache is
/// rewritten without them.
pub fn open(
    store: &Store,
    source: &dyn PageSource,
    options: OpenOptions,
    show_all: &[&str],
) -> kanso_store::Result<PageView> {
    let mut cache = store.page_cache(source.doc_id())?;
    let mut view = PageView::default();
    let mut dropped = false;
    let mut allowed: HashMap<String, bool> = HashMap::new();
    for spec in source.sections() {
        if cache.is_hidden(spec.key) {
            view.hidden
                .push((spec.key.to_owned(), spec.title.to_owned()));
            continue;
        }
        match spec.kind {
            SectionKind::Computed => {
                view.sections.push(source.compute(
                    store,
                    spec.key,
                    show_all.contains(&spec.key),
                )?);
            }
            SectionKind::Ai => {
                let Some(cached) = cache.sections.get_mut(spec.key) else {
                    let state = if options.ai_on {
                        SectionState::Unwritten
                    } else {
                        SectionState::Off(OffReason::AiOff)
                    };
                    view.sections.push(Section {
                        key: spec.key.to_owned(),
                        title: spec.title.to_owned(),
                        kind: SectionKind::Ai,
                        items: Vec::new(),
                        state,
                        written_at: None,
                        total: 0,
                        note: None,
                        none: false,
                    });
                    continue;
                };
                let before = cached.items.len();
                cached.items.retain(|item| {
                    item.sources.iter().all(|s| {
                        *allowed
                            .entry(s.note.clone())
                            .or_insert_with(|| note_is_eligible(store, &s.note))
                    })
                });
                if cached.items.len() != before {
                    dropped = true;
                }
                let now_inputs = current_inputs(store, source, spec.key)?;
                let changed = if fingerprint(&now_inputs) == cached.fingerprint {
                    0
                } else {
                    changed_between(&cached.inputs, &now_inputs).max(1)
                };
                let state = match (cached.pinned, changed) {
                    (true, changed) => SectionState::Pinned {
                        at: cached.written_at,
                        changed,
                    },
                    (false, 0) => SectionState::Fresh,
                    (false, changed) => SectionState::Stale { changed },
                };
                let items: Vec<Item> = cached
                    .items
                    .iter()
                    .map(|item| Item {
                        text: item.text.clone(),
                        who: item.who.clone(),
                        sources: item
                            .sources
                            .iter()
                            .enumerate()
                            .map(|(ix, s)| receipt_of(ix + 1, s))
                            .collect(),
                        ..Item::default()
                    })
                    .collect();
                let note = cached.based_on.map(based_on_line);
                if let Some(n) = cached.based_on {
                    view.based_on = Some(n);
                }
                view.sections.push(Section {
                    key: spec.key.to_owned(),
                    title: spec.title.to_owned(),
                    kind: SectionKind::Ai,
                    total: items.len(),
                    items,
                    state,
                    written_at: Some(cached.written_at),
                    note,
                    none: cached.none,
                });
            }
        }
    }
    if dropped {
        store.set_page_cache(source.doc_id(), &cache)?;
    }
    Ok(view)
}

/// "Based on the 60 most recent mentions."
pub fn based_on_line(mentions: u32) -> String {
    format!("Based on the {mentions} most recent mentions.")
}

/// The sections of `view` a policy would rewrite now: stale or unwritten,
/// not kept, and not written within the last hour. Nothing at all when AI
/// is off.
pub fn due_for_refresh(
    view: &PageView,
    policy: RefreshPolicy,
    options: OpenOptions,
) -> Vec<String> {
    if !options.ai_on || policy != RefreshPolicy::OnOpen {
        return Vec::new();
    }
    view.sections
        .iter()
        .filter(|s| s.kind == SectionKind::Ai && s.wants_refresh())
        .filter(|s| {
            s.written_at
                .is_none_or(|at| options.now - at >= AUTO_REFRESH_GAP_MS)
        })
        .map(|s| s.key.clone())
        .collect()
}

/// Writes one AI section again and stores it. A kept section stays kept,
/// with its new text.
pub fn refresh_section(
    store: &Store,
    source: &dyn PageSource,
    key: &str,
    context: &write::WriteContext,
) -> Result<CachedSection, write::WriteError> {
    let spec = spec_of(source, key).ok_or(write::WriteError::NoSuchSection)?;
    if spec.kind != SectionKind::Ai {
        return Err(write::WriteError::NoSuchSection);
    }
    let mut cache = store.page_cache(source.doc_id())?;
    let (pinned, kept_at) = cache
        .sections
        .get(key)
        .map_or((false, 0), |s| (s.pinned, s.kept_at));
    let mut written = write::write_section(store, source, key, &cache.not_right, context)?;
    written.pinned = pinned;
    written.kept_at = kept_at;
    cache.sections.insert(key.to_owned(), written.clone());
    store.set_page_cache(source.doc_id(), &cache)?;
    Ok(written)
}

/// Refreshes every AI section that is not kept. Returns the keys written.
pub fn refresh_all(
    store: &Store,
    source: &dyn PageSource,
    context: &write::WriteContext,
) -> Result<Vec<String>, write::WriteError> {
    let cache = store.page_cache(source.doc_id())?;
    let mut done = Vec::new();
    for spec in source.sections() {
        if spec.kind != SectionKind::Ai
            || cache.sections.get(spec.key).is_some_and(|s| s.pinned)
            || cache.is_hidden(spec.key)
        {
            continue;
        }
        refresh_section(store, source, spec.key, context)?;
        done.push(spec.key.to_owned());
    }
    Ok(done)
}

/// Keeps (or stops keeping) a section's current text. A section that was
/// never written can't be kept.
pub fn pin(
    store: &Store,
    source: &dyn PageSource,
    key: &str,
    on: bool,
    now: i64,
) -> kanso_store::Result<bool> {
    let mut cache = store.page_cache(source.doc_id())?;
    let Some(section) = cache.sections.get_mut(key) else {
        return Ok(false);
    };
    if section.pinned == on {
        return Ok(false);
    }
    section.pinned = on;
    section.kept_at = now;
    store.set_page_cache(source.doc_id(), &cache)
}

/// Hides a section on this page, or shows it again.
pub fn hide(
    store: &Store,
    source: &dyn PageSource,
    key: &str,
    hidden: bool,
    now: i64,
) -> kanso_store::Result<bool> {
    let mut cache = store.page_cache(source.doc_id())?;
    if cache.is_hidden(key) == hidden {
        return Ok(false);
    }
    cache.hidden.insert(
        key.to_owned(),
        kanso_core::page_cache::HiddenMark { at: now, hidden },
    );
    store.set_page_cache(source.doc_id(), &cache)
}

/// Shows every hidden section again.
pub fn show_hidden(store: &Store, source: &dyn PageSource, now: i64) -> kanso_store::Result<bool> {
    let cache = store.page_cache(source.doc_id())?;
    let keys: Vec<String> = cache
        .hidden
        .iter()
        .filter(|(_, mark)| mark.hidden)
        .map(|(key, _)| key.clone())
        .collect();
    let mut changed = false;
    for key in keys {
        changed |= hide(store, source, &key, false, now)?;
    }
    Ok(changed)
}

/// Removes line `index` of an AI section and remembers it was wrong, so the
/// next rewrite is told and won't repeat it.
pub fn not_right(
    store: &Store,
    source: &dyn PageSource,
    key: &str,
    index: usize,
) -> kanso_store::Result<bool> {
    let mut cache = store.page_cache(source.doc_id())?;
    let Some(section) = cache.sections.get_mut(key) else {
        return Ok(false);
    };
    if index >= section.items.len() {
        return Ok(false);
    }
    let item = section.items.remove(index);
    let (note, line) = item
        .sources
        .first()
        .map_or((String::new(), 0), |s| (s.note.clone(), s.start));
    cache
        .not_right
        .insert(not_right_key(&note, line, &item.text));
    store.set_page_cache(source.doc_id(), &cache)
}

/// The words of the memories, for telling the model.
pub fn not_right_words(memories: &BTreeSet<String>) -> Vec<String> {
    memories
        .iter()
        .filter_map(|m| m.split_once(':').map(|(_, words)| words.to_owned()))
        .filter(|w| !w.is_empty())
        .collect()
}

/// A section's lines as Markdown, with each source as a link to its note
/// and line (`[Weekly sync](inkdrop://note/<id>#L12)`), for copying into the
/// person's own text.
pub fn section_markdown(section: &Section) -> String {
    let mut out = String::new();
    for item in &section.items {
        out.push_str("- ");
        if let Some(who) = &item.who {
            out.push_str(if who == "you" { "You: " } else { "Them: " });
        }
        out.push_str(item.text.trim());
        for receipt in &item.sources {
            let id = receipt
                .note_id
                .strip_prefix("note:")
                .unwrap_or(&receipt.note_id);
            let title = if receipt.title.trim().is_empty() {
                "Untitled"
            } else {
                receipt.title.trim()
            };
            let anchor = kanso_core::links::line_anchor_text(receipt.start, receipt.end);
            out.push_str(&format!(
                " [{}]({}{id}#{anchor})",
                title.replace(['[', ']'], ""),
                kanso_core::links::NOTE_LINK_PREFIX
            ));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn pair() -> impl Strategy<Value = (String, String)> {
        ("[a-d]{1,2}", "[0-3]").prop_map(|(i, h)| (format!("note:{i}"), h))
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// The fingerprint changes exactly when the set of sources, or one
        /// of their hashes, changes: never with the order.
        #[test]
        fn prop_fingerprint_changes_iff_a_source_changes(
            a in proptest::collection::vec(pair(), 0..6),
            b in proptest::collection::vec(pair(), 0..6),
        ) {
            let mut sa = a.clone();
            sa.sort();
            sa.dedup();
            let mut sb = b.clone();
            sb.sort();
            sb.dedup();
            prop_assert_eq!(fingerprint(&a) == fingerprint(&b), sa == sb);
            let mut reversed = a.clone();
            reversed.reverse();
            prop_assert_eq!(fingerprint(&a), fingerprint(&reversed));
        }
    }

    // no-prop-test: three literal lists with known counts.
    #[test]
    fn changed_counts_added_removed_and_edited() {
        let old = vec![
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "1".to_owned()),
        ];
        let new = vec![
            ("a".to_owned(), "2".to_owned()),
            ("c".to_owned(), "1".to_owned()),
        ];
        assert_eq!(changed_between(&old, &new), 3);
        assert_eq!(changed_between(&old, &old), 0);
    }
}
