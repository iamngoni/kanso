//! Fuzzy matching for the command palette: whether what was typed picks out
//! a candidate, how well, and the candidates in order of fit.
//!
//! A query matches when its characters appear in the candidate in order,
//! ignoring case, so "mvtr" finds "Move to trash". Of all the ways to line
//! the characters up, the best one counts: characters right after the
//! previous match and characters that start a word score extra, and every
//! character skipped costs a little.

/// What each matched character is worth.
const MATCH: i64 = 16;
/// Extra for a character right after the previous one matched. At least
/// [`WORD_START`], so a run never scores less than the same characters
/// spread out over word starts.
const CONSECUTIVE: i64 = 12;
/// Extra for a character that starts a word: the first one, one after a
/// space or punctuation, or a capital after a lowercase letter.
const WORD_START: i64 = 8;
/// Cost of each character skipped between two matched ones.
const GAP: i64 = 1;
/// Cost of each character before the first match, up to [`MAX_LEADING`]
/// characters, so that "Trash" fits "trash" better than "Move to trash".
const LEADING: i64 = 1;
const MAX_LEADING: i64 = 10;

/// Below every real score.
const NONE: i64 = i64::MIN / 4;

/// How well `query` fits `candidate`, or `None` when its characters don't
/// all appear in order (ignoring case). An empty query fits everything with
/// a score of 0. Higher is better.
pub fn score(query: &str, candidate: &str) -> Option<i64> {
    let query: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    if query.is_empty() {
        return Some(0);
    }
    let text = fold(candidate);
    if !is_subsequence(&query, &text) {
        return None;
    }
    let bonus = |j: usize| if text[j].1 { WORD_START } else { 0 };

    // best[j]: the best score of the query so far with its last character
    // matched at text[j], or NONE.
    let mut best: Vec<i64> = (0..text.len())
        .map(|j| {
            if text[j].0 == query[0] {
                MATCH + bonus(j) - LEADING * (j as i64).min(MAX_LEADING)
            } else {
                NONE
            }
        })
        .collect();
    for (i, &want) in query.iter().enumerate().skip(1) {
        let mut next = vec![NONE; text.len()];
        // The best `best[k] + GAP * k` over k <= j - 2: matching at j after
        // a match at k skips j - k - 1 characters.
        let mut after_gap = NONE;
        for j in i..text.len() {
            if j >= 2 && best[j - 2] > NONE {
                after_gap = after_gap.max(best[j - 2] + GAP * (j as i64 - 2));
            }
            if text[j].0 != want {
                continue;
            }
            let mut here = NONE;
            if best[j - 1] > NONE {
                here = best[j - 1] + MATCH + CONSECUTIVE.max(bonus(j));
            }
            if after_gap > NONE {
                here = here.max(after_gap - GAP * (j as i64 - 1) + MATCH + bonus(j));
            }
            next[j] = here;
        }
        best = next;
    }
    best.into_iter().max().filter(|&s| s > NONE)
}

/// The candidates `query` fits, best first; equally good ones keep their
/// order. The palette scores labels and keywords together and ranks those
/// scores with [`rank_scores`]; this is the plain-strings form.
#[cfg(test)]
pub fn rank<S: AsRef<str>>(query: &str, candidates: &[S]) -> Vec<usize> {
    let scores: Vec<Option<i64>> = candidates
        .iter()
        .map(|c| score(query, c.as_ref()))
        .collect();
    rank_scores(&scores)
}

/// The indices of the scores that are there, highest first; equal scores
/// keep their order.
pub fn rank_scores(scores: &[Option<i64>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..scores.len()).filter(|&i| scores[i].is_some()).collect();
    // A stable sort, so ties stay in index order.
    order.sort_by_key(|&i| std::cmp::Reverse(scores[i]));
    order
}

/// The candidate in lowercase, each character with whether it starts a word.
/// A character whose lowercase is longer (like 'İ') starts a word with its
/// first piece only.
fn fold(candidate: &str) -> Vec<(char, bool)> {
    let mut out = Vec::with_capacity(candidate.len());
    let mut previous: Option<char> = None;
    for c in candidate.chars() {
        let starts = match previous {
            None => true,
            Some(p) => !p.is_alphanumeric() || (p.is_lowercase() && c.is_uppercase()),
        };
        for (k, lower) in c.to_lowercase().enumerate() {
            out.push((lower, starts && k == 0));
        }
        previous = Some(c);
    }
    out
}

fn is_subsequence(query: &[char], text: &[(char, bool)]) -> bool {
    let mut text = text.iter();
    query.iter().all(|q| text.any(|(c, _)| c == q))
}

#[cfg(test)]
mod tests {
    use super::{rank, rank_scores, score};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The reference: each folded query character found after the previous.
    fn folded_subsequence(query: &str, candidate: &str) -> bool {
        let mut text = candidate.chars().flat_map(char::to_lowercase);
        query
            .chars()
            .flat_map(char::to_lowercase)
            .all(|q| text.any(|c| c == q))
    }

    /// Prefixes of `s` at character boundaries, shortest first.
    fn prefixes(s: &str) -> Vec<&str> {
        s.char_indices().map(|(i, _)| &s[..i]).chain([s]).collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Oracle: a query fits exactly when its case-folded characters are
        /// a subsequence of the case-folded candidate.
        #[test]
        fn prop_score_is_some_iff_folded_subsequence(
            query in "[a-dA-DİßΣσ ]{0,5}",
            candidate in "[a-fA-FİßΣσς -]{0,14}",
        ) {
            prop_assert_eq!(
                score(&query, &candidate).is_some(),
                folded_subsequence(&query, &candidate)
            );
        }

        /// Invariant: whatever a query fits, every prefix of it fits too.
        #[test]
        fn prop_score_is_prefix_closed(query in "[a-eA-E ]{0,6}", candidate in "[a-eA-E -]{0,16}") {
            if score(&query, &candidate).is_some() {
                for prefix in prefixes(&query) {
                    prop_assert!(score(prefix, &candidate).is_some(), "{:?} of {:?}", prefix, query);
                }
            }
        }

        /// Metamorphic: the query as one run scores at least as well as the
        /// same characters spread out with other letters between them, even
        /// when each spread-out character starts a word of its own. (Then
        /// nothing before them may be part of the query: "kn a" is a fair
        /// better fit for "ka" than "ka" inside "kka".)
        #[test]
        fn prop_contiguous_scores_at_least_scattered(
            query in "[a-m]{1,6}",
            before in "[a-z ]{0,8}",
            after in "[a-z ]{0,8}",
            fillers in prop::collection::vec("[n-z]{1,3}", 6),
            word_starts in any::<bool>(),
        ) {
            let before: String = if word_starts {
                before.chars().filter(|c| !query.contains(*c)).collect()
            } else {
                before
            };
            // Nor may the tail: its own copy of the query's letters would
            // give the spread a second run, at a word start the run itself
            // never had, which is a different alignment, not a fairer one.
            let after: String = after.chars().filter(|c| !query.contains(*c)).collect();
            let together = format!("{before}{query}{after}");
            let mut spread = before.clone();
            for (ix, c) in query.chars().enumerate() {
                if ix > 0 {
                    spread.push_str(&fillers[ix - 1]);
                    if word_starts {
                        spread.push(' ');
                    }
                }
                spread.push(c);
            }
            spread.push_str(&after);
            let (Some(together_score), Some(spread_score)) =
                (score(&query, &together), score(&query, &spread))
            else {
                return Err(TestCaseError::fail("both contain the query"));
            };
            prop_assert!(together_score >= spread_score, "{} < {} for {:?} vs {:?}", together_score, spread_score, together, spread);
        }

        /// Invariant: case in the query never changes the score.
        #[test]
        fn prop_score_ignores_query_case(query in "[a-zA-Z ]{0,6}", candidate in "[a-zA-Z -]{0,16}") {
            prop_assert_eq!(score(&query, &candidate), score(&query.to_uppercase(), &candidate));
            prop_assert_eq!(score(&query, &candidate), score(&query.to_lowercase(), &candidate));
        }

        /// Invariant: ranking returns each fitting score's index once and
        /// nothing else, best first, ties in their original order.
        #[test]
        fn prop_rank_scores_is_a_stable_sort_of_the_matches(
            scores in prop::collection::vec(prop::option::of(-5i64..5), 0..24),
        ) {
            let order = rank_scores(&scores);
            let mut seen = order.clone();
            seen.sort_unstable();
            let matches: Vec<usize> = (0..scores.len()).filter(|&i| scores[i].is_some()).collect();
            prop_assert_eq!(seen, matches);
            for pair in order.windows(2) {
                let (a, b) = (scores[pair[0]].unwrap(), scores[pair[1]].unwrap());
                prop_assert!(a > b || (a == b && pair[0] < pair[1]), "{:?}", order);
            }
        }

        /// Composition: ranking candidates ranks their scores.
        #[test]
        fn prop_rank_ranks_the_scores(query in "[a-c]{0,3}", candidates in prop::collection::vec("[a-d ]{0,8}", 0..12)) {
            let scores: Vec<Option<i64>> = candidates.iter().map(|c| score(&query, c)).collect();
            prop_assert_eq!(rank(&query, &candidates), rank_scores(&scores));
        }
    }

    #[test]
    fn examples() {
        assert_eq!(score("", "anything"), Some(0));
        assert_eq!(score("", ""), Some(0));
        assert_eq!(score("a", ""), None);
        assert!(score("mvtr", "Move to trash").is_some());
        assert!(score("NOTE", "new note").is_some());
        assert_eq!(score("ab", "ba"), None);
        // German sharp s folds to itself; capital I with dot to two pieces.
        assert!(score("i", "İstanbul").is_some());
        assert!(score("σ", "ΣΑΣ").is_some());
    }

    #[test]
    fn a_word_start_beats_the_middle_of_a_word() {
        let start = score("pin", "Pin note").unwrap();
        let middle = score("pin", "Spinning").unwrap();
        assert!(start > middle, "{start} vs {middle}");
    }

    #[test]
    fn earlier_matches_rank_first() {
        let candidates = ["Move to trash", "Trash", "Notes"];
        assert_eq!(rank("trash", &candidates), vec![1, 0]);
        assert_eq!(rank("", &candidates), vec![0, 1, 2]);
        assert_eq!(rank("zzz", &candidates), Vec::<usize>::new());
    }

    #[test]
    fn a_run_beats_initials_scattered_through_words() {
        // "sync" as a word, and as letters of "Some yearly notes, cats".
        let run = score("sync", "Weekly sync notes").unwrap();
        let scattered = score("sync", "Some yearly notes, cats").unwrap();
        assert!(run > scattered, "{run} vs {scattered}");
    }

    #[test]
    fn camel_case_starts_a_word() {
        assert!(score("ip", "importPath").unwrap() > score("ip", "importpath").unwrap());
    }
}
