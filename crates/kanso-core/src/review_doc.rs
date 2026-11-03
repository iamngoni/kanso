//! The `review` document: one month of flashcard answers for one note.
//!
//! ```json
//! { "noteId": "note:abc", "month": "2026-11",
//!   "log":   [ {"id":"r_…","card":"<card key>","at":1793000000000,"g":3,"d":"<device>"} ],
//!   "marks": [ {"id":"m_…","card":"<card key>","at":…,"kind":"suspend|resume|reset"} ] }
//! ```
//!
//! Two versions of the same document merge by union of `log` and `marks`
//! on `id` ([`merge`]): commutative, associative and idempotent, so two
//! devices reviewing offline never conflict. The words of a card are not in
//! it, only its key.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::srs::{Grade, Mark, MarkKind, Review};

/// Entries (answers plus marks) a document may hold before a second one
/// is started for the month.
pub const MAX_ENTRIES: usize = 5_000;

/// The document's contents, read from its fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReviewDoc {
    pub note_id: String,
    pub month: String,
    pub log: Vec<Review>,
    pub marks: Vec<Mark>,
}

impl ReviewDoc {
    pub fn new(note_id: &str, month: &str) -> Self {
        ReviewDoc {
            note_id: note_id.to_owned(),
            month: month.to_owned(),
            log: Vec::new(),
            marks: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.log.len() + self.marks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads the fields of a stored document; malformed entries are left out.
    pub fn from_fields(fields: &Map<String, Value>) -> Self {
        let text = |key: &str| {
            fields
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        let list = |key: &str| {
            fields
                .get(key)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        let log = list("log")
            .iter()
            .filter_map(|e| {
                Some(Review {
                    id: e.get("id")?.as_str()?.to_owned(),
                    card: e.get("card")?.as_str()?.to_owned(),
                    at_ms: e.get("at")?.as_i64()?,
                    grade: Grade::from_number(e.get("g")?.as_i64()?)?,
                    device: e.get("d")?.as_str()?.to_owned(),
                })
            })
            .collect();
        let marks = list("marks")
            .iter()
            .filter_map(|e| {
                let kind = match e.get("kind")?.as_str()? {
                    "suspend" => MarkKind::Suspend,
                    "resume" => MarkKind::Resume,
                    "reset" => MarkKind::Reset,
                    _ => return None,
                };
                Some(Mark {
                    id: e.get("id")?.as_str()?.to_owned(),
                    card: e.get("card")?.as_str()?.to_owned(),
                    at_ms: e.get("at")?.as_i64()?,
                    kind,
                })
            })
            .collect();
        ReviewDoc {
            note_id: text("noteId"),
            month: text("month"),
            log,
            marks,
        }
    }

    /// The fields to store: entries sorted by `(at, id)` so equal contents
    /// give equal documents.
    pub fn to_fields(&self) -> Map<String, Value> {
        let mut log = self.log.clone();
        log.sort_by(|a, b| (a.at_ms, &a.id).cmp(&(b.at_ms, &b.id)));
        let mut marks = self.marks.clone();
        marks.sort_by(|a, b| (a.at_ms, &a.id).cmp(&(b.at_ms, &b.id)));
        let mut out = Map::new();
        out.insert("noteId".into(), self.note_id.clone().into());
        out.insert("month".into(), self.month.clone().into());
        out.insert(
            "log".into(),
            Value::Array(
                log.iter()
                    .map(|r| {
                        json!({"id": r.id, "card": r.card, "at": r.at_ms,
                               "g": r.grade.number(), "d": r.device})
                    })
                    .collect(),
            ),
        );
        out.insert(
            "marks".into(),
            Value::Array(
                marks
                    .iter()
                    .map(|m| {
                        let kind = match m.kind {
                            MarkKind::Suspend => "suspend",
                            MarkKind::Resume => "resume",
                            MarkKind::Reset => "reset",
                        };
                        json!({"id": m.id, "card": m.card, "at": m.at_ms, "kind": kind})
                    })
                    .collect(),
            ),
        );
        out
    }
}

/// Both documents' entries, once each by `id` (the earlier `at` kept if
/// two entries share an id). Fields other than the entries come from `a`.
pub fn merge(a: &ReviewDoc, b: &ReviewDoc) -> ReviewDoc {
    let mut log: BTreeMap<&str, &Review> = BTreeMap::new();
    for r in a.log.iter().chain(&b.log) {
        log.entry(&r.id)
            .and_modify(|mine| {
                if (r.at_ms, &r.card, r.grade as u8, &r.device)
                    < (mine.at_ms, &mine.card, mine.grade as u8, &mine.device)
                {
                    *mine = r;
                }
            })
            .or_insert(r);
    }
    let mut marks: BTreeMap<&str, &Mark> = BTreeMap::new();
    for m in a.marks.iter().chain(&b.marks) {
        marks
            .entry(&m.id)
            .and_modify(|mine| {
                if (m.at_ms, &m.card, m.kind as u8) < (mine.at_ms, &mine.card, mine.kind as u8) {
                    *mine = m;
                }
            })
            .or_insert(m);
    }
    ReviewDoc {
        note_id: a.note_id.clone(),
        month: a.month.clone(),
        log: log.into_values().cloned().collect(),
        marks: marks.into_values().cloned().collect(),
    }
}

/// The fields of the union of two stored versions.
pub fn merge_fields(a: &Map<String, Value>, b: &Map<String, Value>) -> Map<String, Value> {
    merge(&ReviewDoc::from_fields(a), &ReviewDoc::from_fields(b)).to_fields()
}

/// The document ID for a note's `index`th document of a month:
/// `review:<hash of note id>-<yyyymm>` and `…-<n>` for later ones.
pub fn doc_id(note_id: &str, month: &str, index: u32) -> String {
    let hash = format!("{:016x}", crate::reminders::fnv1a(note_id));
    let month = month.replace('-', "");
    if index <= 1 {
        format!("review:{hash}-{month}")
    } else {
        format!("review:{hash}-{month}-{index}")
    }
}

/// `yyyy-mm` of a time in milliseconds since the epoch (UTC).
pub fn month_of(at_ms: i64) -> String {
    let days = at_ms.div_euclid(86_400_000);
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn months_are_calendar_months() {
        assert_eq!(month_of(0), "1970-01");
        assert_eq!(month_of(1_793_000_000_000), "2026-10");
        assert_eq!(month_of(1_782_864_000_000), "2026-07");
        assert_eq!(month_of(951_782_400_000), "2000-02");
    }

    #[test]
    fn ids_are_stable_and_distinct() {
        let a = doc_id("note:a", "2026-11", 1);
        assert_eq!(a, doc_id("note:a", "2026-11", 1));
        assert_ne!(a, doc_id("note:a", "2026-11", 2));
        assert_ne!(a, doc_id("note:b", "2026-11", 1));
        assert!(crate::id::is_valid_id(crate::DocType::Review, &a));
        assert_eq!(crate::DocType::of_id(&a), Some(crate::DocType::Review));
    }

    fn entry() -> impl Strategy<Value = Review> {
        (0u32..8, 0i64..50, 1i64..=4, "[ab]").prop_map(|(i, at, g, card)| Review {
            id: format!("r{i}"),
            card,
            at_ms: at,
            grade: Grade::from_number(g).unwrap(),
            device: "d".into(),
        })
    }

    fn mark() -> impl Strategy<Value = Mark> {
        (0u32..6, 0i64..50, 0u8..3, "[ab]").prop_map(|(i, at, k, card)| Mark {
            id: format!("m{i}"),
            card,
            at_ms: at,
            kind: [MarkKind::Suspend, MarkKind::Resume, MarkKind::Reset][k as usize],
        })
    }

    fn doc() -> impl Strategy<Value = ReviewDoc> {
        (
            prop::collection::vec(entry(), 0..8),
            prop::collection::vec(mark(), 0..6),
        )
            .prop_map(|(log, marks)| {
                // Within one document an id appears once.
                let mut seen = std::collections::HashSet::new();
                let log = log
                    .into_iter()
                    .filter(|r| seen.insert(r.id.clone()))
                    .collect();
                let mut seen = std::collections::HashSet::new();
                let marks = marks
                    .into_iter()
                    .filter(|m| seen.insert(m.id.clone()))
                    .collect();
                ReviewDoc {
                    note_id: "note:x".into(),
                    month: "2026-11".into(),
                    log,
                    marks,
                }
            })
    }

    fn canon(d: ReviewDoc) -> Map<String, Value> {
        d.to_fields()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        #[test]
        fn prop_merge_is_commutative_associative_idempotent(a in doc(), b in doc(), c in doc()) {
            prop_assert_eq!(canon(merge(&a, &b)), canon(merge(&b, &a)));
            prop_assert_eq!(
                canon(merge(&merge(&a, &b), &c)),
                canon(merge(&a, &merge(&b, &c)))
            );
            prop_assert_eq!(canon(merge(&a, &a)), canon(a.clone()));
        }

        #[test]
        fn prop_merge_keeps_every_id(a in doc(), b in doc()) {
            let m = merge(&a, &b);
            for r in a.log.iter().chain(&b.log) {
                prop_assert!(m.log.iter().any(|x| x.id == r.id));
            }
            for k in a.marks.iter().chain(&b.marks) {
                prop_assert!(m.marks.iter().any(|x| x.id == k.id));
            }
        }

        #[test]
        fn prop_fields_round_trip(a in doc()) {
            let back = ReviewDoc::from_fields(&a.to_fields());
            prop_assert_eq!(canon(back), canon(a));
        }
    }
}
