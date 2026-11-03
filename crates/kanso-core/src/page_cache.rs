//! The cache of a generated page: what the AI wrote for each section, the
//! lines it rests on, and what the person did with it (kept it, hid it,
//! said a bullet was wrong).
//!
//! It lives in the `kansoPage` field of the page's own note (or notebook),
//! so it syncs sealed with the document and a phone without AI can show what
//! the Mac wrote. It is never part of the note's text.
//!
//! Pure: reading, writing, merging two devices' copies and keeping the
//! whole under its size cap.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The field a page's cache is stored in.
pub const PAGE_FIELD: &str = "kansoPage";
/// The format version written.
pub const VERSION: u32 = 1;
/// The most the cache may weigh, in bytes of JSON.
pub const MAX_BYTES: usize = 64 * 1024;
/// The most "not right" memories kept.
pub const MAX_NOT_RIGHT: usize = 200;

/// The lines one item rests on, as they were sent.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedSource {
    pub note: String,
    /// 0-based, inclusive.
    pub start: u32,
    pub end: u32,
    pub quote: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// One line of an AI section.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedItem {
    pub text: String,
    /// "you" or "them" for a promise; absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
    pub sources: Vec<CachedSource>,
}

/// One AI section as last written.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedSection {
    pub items: Vec<CachedItem>,
    #[serde(rename = "writtenAt")]
    pub written_at: i64,
    #[serde(default)]
    pub model: String,
    pub fingerprint: String,
    /// The notes that fed it, as `(id, short hash)`, so a stale section can
    /// say how many changed.
    #[serde(default)]
    pub inputs: Vec<(String, String)>,
    #[serde(default)]
    pub pinned: bool,
    /// When the person last kept or stopped keeping it, so the later choice
    /// holds across devices (0 when never touched).
    #[serde(default, rename = "keptAt", skip_serializing_if = "is_zero")]
    pub kept_at: i64,
    /// The model had nothing to say: the section says so rather than being
    /// asked again.
    #[serde(default)]
    pub none: bool,
    /// How many mentions it was based on when there were more than it could
    /// read ("Based on the 60 most recent mentions").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub based_on: Option<u32>,
}

/// Whether a section is hidden, as of when. The latest mark wins, so
/// showing it again on one device holds on the other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenMark {
    pub at: i64,
    pub hidden: bool,
}

/// The order a page's sections are shown in, as of when it was set (ms).
/// The later write wins, so reordering on one device holds on the other.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub order: Vec<String>,
    pub at: i64,
}

/// A page's cache.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageCache {
    #[serde(default)]
    pub v: u32,
    #[serde(default)]
    pub sections: BTreeMap<String, CachedSection>,
    #[serde(default)]
    pub hidden: BTreeMap<String, HiddenMark>,
    /// Bullets the person said were wrong: `note#line:words`.
    #[serde(default, rename = "notRight")]
    pub not_right: BTreeSet<String>,
    /// A notebook homepage's section order (people's pages have none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

impl PageCache {
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
            && self.hidden.is_empty()
            && self.not_right.is_empty()
            && self.layout.is_none()
    }

    /// Whether `key` is hidden for this page.
    pub fn is_hidden(&self, key: &str) -> bool {
        self.hidden.get(key).is_some_and(|mark| mark.hidden)
    }
}

/// The cache a document's fields hold; anything malformed is left out.
pub fn read(fields: &Map<String, Value>) -> PageCache {
    fields
        .get(PAGE_FIELD)
        .and_then(|value| serde_json::from_value::<PageCache>(value.clone()).ok())
        .unwrap_or_default()
}

/// The cache as the field stores it, cut to [`MAX_BYTES`].
pub fn to_value(cache: &PageCache) -> Value {
    let mut cache = cache.clone();
    cache.v = VERSION;
    fit(&mut cache);
    serde_json::to_value(&cache).unwrap_or(Value::Null)
}

fn size(cache: &PageCache) -> usize {
    serde_json::to_string(cache).map_or(0, |s| s.len())
}

/// Drops the oldest items, then the oldest memories, until the cache fits.
pub fn fit(cache: &mut PageCache) {
    while cache.not_right.len() > MAX_NOT_RIGHT {
        let first = cache.not_right.iter().next().cloned();
        if let Some(first) = first {
            cache.not_right.remove(&first);
        }
    }
    while size(cache) > MAX_BYTES {
        // The section holding the most items loses its last one; a section
        // that empties is kept as "written, nothing to show".
        let Some((key, _)) = cache
            .sections
            .iter()
            .filter(|(_, s)| !s.items.is_empty())
            .max_by_key(|(_, s)| s.items.len())
            .map(|(k, s)| (k.clone(), s.items.len()))
        else {
            break;
        };
        if let Some(section) = cache.sections.get_mut(&key) {
            section.items.pop();
        }
    }
    while size(cache) > MAX_BYTES {
        let first = cache.not_right.iter().next().cloned();
        match first {
            Some(first) => {
                cache.not_right.remove(&first);
            }
            None => break,
        }
    }
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

/// Which of two copies of one section wins. When one is kept and the other
/// isn't, the later keep/stop-keeping choice wins (a tie keeps it), so
/// stopping keeping on one device is not undone by the other's stale copy.
/// Otherwise the later write, else a fixed order so merging commutes.
fn section_wins(a: &CachedSection, b: &CachedSection) -> bool {
    if a.pinned != b.pinned && a.kept_at != b.kept_at {
        return a.kept_at > b.kept_at;
    }
    (
        a.pinned,
        a.written_at,
        &a.fingerprint,
        a.items.len(),
        a.kept_at,
    ) >= (
        b.pinned,
        b.written_at,
        &b.fingerprint,
        b.items.len(),
        b.kept_at,
    )
}

/// Two devices' caches as one: each section from whichever copy wins, a
/// section's hidden state from the later mark, the memories of both.
/// Commutative and idempotent.
pub fn merge(a: &PageCache, b: &PageCache) -> PageCache {
    let mut merged = PageCache {
        v: VERSION,
        ..PageCache::default()
    };
    for key in a.sections.keys().chain(b.sections.keys()) {
        let section = match (a.sections.get(key), b.sections.get(key)) {
            (Some(x), Some(y)) => {
                // Exactly equal winners must not depend on the order.
                if section_wins(x, y) && (section_wins(y, x) || x != y) {
                    x.clone()
                } else if section_wins(y, x) {
                    y.clone()
                } else {
                    x.clone()
                }
            }
            (Some(x), None) => x.clone(),
            (None, Some(y)) => y.clone(),
            (None, None) => continue,
        };
        merged.sections.insert(key.clone(), section);
    }
    for key in a.hidden.keys().chain(b.hidden.keys()) {
        let mark = match (a.hidden.get(key), b.hidden.get(key)) {
            (Some(x), Some(y)) => {
                if (x.at, x.hidden) >= (y.at, y.hidden) {
                    *x
                } else {
                    *y
                }
            }
            (Some(x), None) => *x,
            (None, Some(y)) => *y,
            (None, None) => continue,
        };
        merged.hidden.insert(key.clone(), mark);
    }
    merged.not_right = a.not_right.union(&b.not_right).cloned().collect();
    merged.layout = match (&a.layout, &b.layout) {
        (Some(x), Some(y)) => Some(if (x.at, &x.order) >= (y.at, &y.order) {
            x.clone()
        } else {
            y.clone()
        }),
        (Some(x), None) => Some(x.clone()),
        (None, y) => y.clone(),
    };
    fit(&mut merged);
    merged
}

/// A memory of a bullet the person said was wrong.
pub fn not_right_key(note: &str, line: u32, text: &str) -> String {
    let words: String = text
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    format!("{note}#{line}:{words}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn section(written_at: i64, pinned: bool, text: &str) -> CachedSection {
        CachedSection {
            items: vec![CachedItem {
                text: text.into(),
                who: None,
                sources: vec![CachedSource {
                    note: "note:a".into(),
                    start: 1,
                    end: 1,
                    quote: "q".into(),
                    ..CachedSource::default()
                }],
            }],
            written_at,
            fingerprint: format!("f{written_at}{text}"),
            pinned,
            ..CachedSection::default()
        }
    }

    fn cache_strategy() -> impl Strategy<Value = PageCache> {
        let sections = proptest::collection::btree_map(
            prop::sample::select(vec!["promises", "discussed", "about"]).prop_map(str::to_owned),
            (0i64..4, any::<bool>(), "[a-c]{1,3}").prop_map(|(t, p, s)| section(t, p, &s)),
            0..3,
        );
        let hidden = proptest::collection::btree_map(
            prop::sample::select(vec!["timeline", "related"]).prop_map(str::to_owned),
            (0i64..4, any::<bool>()).prop_map(|(at, hidden)| HiddenMark { at, hidden }),
            0..2,
        );
        let not_right = proptest::collection::btree_set("[a-c]{1,3}", 0..3);
        let layout = proptest::option::of(
            (0i64..4, proptest::collection::vec("[a-c]", 0..3))
                .prop_map(|(at, order)| Layout { order, at }),
        );
        (sections, hidden, not_right, layout).prop_map(|(sections, hidden, not_right, layout)| {
            PageCache {
                v: VERSION,
                sections,
                hidden,
                not_right,
                layout,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        #[test]
        fn prop_kanso_page_merge_is_commutative_and_keeps_pinned(a in cache_strategy(), b in cache_strategy()) {
            let ab = merge(&a, &b);
            let ba = merge(&b, &a);
            prop_assert_eq!(&ab, &ba);
            prop_assert_eq!(merge(&ab, &ab), ab.clone());
            // A kept section is never replaced by an unkept one.
            for key in a.sections.keys().chain(b.sections.keys()) {
                let pinned_somewhere = [&a, &b].iter().any(|c| c.sections.get(key).is_some_and(|s| s.pinned));
                if pinned_somewhere {
                    prop_assert!(ab.sections[key].pinned);
                }
            }
        }

        #[test]
        fn prop_cache_round_trips_and_fits(a in cache_strategy()) {
            let value = to_value(&a);
            let mut fields = Map::new();
            fields.insert(PAGE_FIELD.into(), value.clone());
            prop_assert!(value.to_string().len() <= MAX_BYTES);
            prop_assert_eq!(read(&fields), PageCache { v: VERSION, ..a });
        }
    }

    // no-prop-test: one concurrent keep/stop-keeping pair.
    #[test]
    fn stopping_keeping_later_beats_a_stale_kept_copy() {
        let mut kept = section(1, true, "x");
        kept.kept_at = 10;
        let mut unkept = kept.clone();
        unkept.pinned = false;
        unkept.kept_at = 20;
        let (mut a, mut b) = (PageCache::default(), PageCache::default());
        a.sections.insert("promises".into(), kept.clone());
        b.sections.insert("promises".into(), unkept);
        assert!(!merge(&a, &b).sections["promises"].pinned);
        assert!(!merge(&b, &a).sections["promises"].pinned);
        // Keeping it again later wins back.
        kept.kept_at = 30;
        a.sections.insert("promises".into(), kept);
        assert!(merge(&a, &b).sections["promises"].pinned);
    }

    // no-prop-test: one oversized cache, one assertion.
    #[test]
    fn an_oversized_cache_loses_items_not_sections() {
        let mut cache = PageCache::default();
        let mut s = section(1, false, "x");
        s.items = (0..2000)
            .map(|i| CachedItem {
                text: format!("item number {i} with some words to take room up"),
                who: None,
                sources: Vec::new(),
            })
            .collect();
        cache.sections.insert("promises".into(), s);
        let value = to_value(&cache);
        assert!(value.to_string().len() <= MAX_BYTES);
        let mut fields = Map::new();
        fields.insert(PAGE_FIELD.into(), value);
        let back = read(&fields);
        assert!(back.sections.contains_key("promises"));
        assert!(!back.sections["promises"].items.is_empty());
    }
}
