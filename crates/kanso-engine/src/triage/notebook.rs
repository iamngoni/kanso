//! The notebook to suggest: where a note's nearest notes live. A wrong
//! notebook costs more than none, so it is suggested only when it clearly
//! leads.

use std::collections::BTreeMap;

use super::model::BECAUSE_MAX;

/// How many of a note's nearest notes are asked.
pub const NEIGHBOURS: usize = 20;
/// A near note counts for its notebook from this similarity.
pub const MIN_SIMILARITY: f32 = 0.45;
/// The least a notebook's similarities must add up to.
pub const MIN_TOTAL: f32 = 0.9;
/// How far ahead of the next notebook it must be (1.25 is 25% more).
pub const LEAD: f32 = 1.25;
/// What the sum is divided by for a score from 0 to 1.
const SCORE_SCALE: f32 = 2.;

/// One near note's vote for the notebook it is in.
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub book: String,
    pub note: String,
    pub similarity: f32,
}

/// The notebook chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub book: String,
    /// The similarities added up.
    pub total: f32,
    /// From 0 to 1.
    pub score: f32,
    /// Its nearest notes, up to [`BECAUSE_MAX`].
    pub because: Vec<String>,
}

/// Every notebook `allowed` accepts that has a vote at least
/// [`MIN_SIMILARITY`], best first (ties by ID), each with its notes. These
/// are the candidates an AI may rank. Pure and deterministic.
pub fn rank(votes: &[Vote], allowed: impl Fn(&str) -> bool) -> Vec<Pick> {
    let mut sums: BTreeMap<&str, f32> = BTreeMap::new();
    for vote in votes {
        if vote.similarity >= MIN_SIMILARITY && allowed(&vote.book) {
            *sums.entry(vote.book.as_str()).or_default() += vote.similarity.min(1.);
        }
    }
    let mut ranked: Vec<(&str, f32)> = sums.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    ranked
        .into_iter()
        .map(|(book, total)| {
            let mut own: Vec<&Vote> = votes
                .iter()
                .filter(|v| v.book == book && v.similarity >= MIN_SIMILARITY)
                .collect();
            own.sort_by(|a, b| {
                b.similarity
                    .total_cmp(&a.similarity)
                    .then_with(|| a.note.cmp(&b.note))
            });
            Pick {
                book: book.to_owned(),
                total,
                score: (total / SCORE_SCALE).clamp(0., 1.),
                because: own
                    .into_iter()
                    .take(BECAUSE_MAX)
                    .map(|v| v.note.clone())
                    .collect(),
            }
        })
        .collect()
}

/// The notebook the `votes` choose, if one does: the top of [`rank`] when
/// it reaches [`MIN_TOTAL`] and leads the next by [`LEAD`]. Pure.
pub fn pick(votes: &[Vote], allowed: impl Fn(&str) -> bool) -> Option<Pick> {
    let ranked = rank(votes, allowed);
    let top = ranked.first()?;
    if top.total < MIN_TOTAL {
        return None;
    }
    if let Some(next) = ranked.get(1)
        && top.total < next.total * LEAD
    {
        return None;
    }
    Some(top.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn vote(book: &str, note: &str, similarity: f32) -> Vote {
        Vote {
            book: book.into(),
            note: note.into(),
            similarity,
        }
    }

    #[test]
    fn a_clear_leader_is_picked() {
        let votes = [
            vote("home", "n1", 0.8),
            vote("home", "n2", 0.6),
            vote("work", "n3", 0.5),
        ];
        let pick = pick(&votes, |_| true).unwrap();
        assert_eq!(pick.book, "home");
        assert_eq!(pick.because, ["n1", "n2"]);
    }

    #[test]
    fn a_weak_or_close_race_picks_none() {
        // Too little.
        assert!(pick(&[vote("home", "n1", 0.8)], |_| true).is_none());
        // Below the similarity floor.
        assert!(
            pick(&[vote("home", "n1", 0.4), vote("home", "n2", 0.4)], |_| {
                true
            })
            .is_none()
        );
        // Within 25%.
        let votes = [
            vote("home", "n1", 0.8),
            vote("home", "n2", 0.7),
            vote("work", "n3", 0.8),
            vote("work", "n4", 0.6),
        ];
        assert!(pick(&votes, |_| true).is_none());
    }

    #[test]
    fn a_refused_notebook_is_never_picked() {
        let votes = [vote("inbox", "n1", 0.9), vote("inbox", "n2", 0.9)];
        assert!(pick(&votes, |b| b != "inbox").is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// The pick is allowed, scored in range, cites at most three notes
        /// and is the same for the same votes.
        #[test]
        fn the_pick_is_allowed_and_in_range(
            raw in prop::collection::vec((0usize..4, -1f32..1.2), 0..20),
            refused in 0usize..4,
        ) {
            let votes: Vec<Vote> = raw
                .iter()
                .enumerate()
                .map(|(i, (b, s))| vote(&format!("b{b}"), &format!("n{i}"), *s))
                .collect();
            let refused = format!("b{refused}");
            let allowed = |b: &str| b != refused;
            let a = pick(&votes, allowed);
            prop_assert_eq!(&a, &pick(&votes, allowed));
            if let Some(p) = a {
                prop_assert!(p.book != refused);
                prop_assert!((0. ..=1.).contains(&p.score));
                prop_assert!(p.because.len() <= BECAUSE_MAX && !p.because.is_empty());
            }
        }
    }
}
