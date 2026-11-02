//! Finding what the model quoted in the transcript. A decision or action
//! item stands only if some quote of it is really there.

/// The fewest letters and digits a quote may have to count.
pub const MIN_QUOTE: usize = 4;

/// The transcript's letters and digits, lowercased, with where each came
/// from (its character index in the transcript) and a space between words.
fn flatten(text: &str) -> (Vec<char>, Vec<usize>) {
    let mut chars = Vec::new();
    let mut at = Vec::new();
    let mut gap = false;
    for (ix, c) in text.chars().enumerate() {
        if c.is_alphanumeric() {
            if gap && !chars.is_empty() {
                chars.push(' ');
                at.push(ix);
            }
            gap = false;
            for l in c.to_lowercase() {
                chars.push(l);
                at.push(ix);
            }
        } else if !matches!(c, '\'' | '’') {
            // Spaces and punctuation alike separate words; an apostrophe
            // doesn't ("can't" and "cant" read alike).
            gap = true;
        }
    }
    (chars, at)
}

/// Where `quote` is in `transcript`, as a range of character indexes
/// (`start..end`, end exclusive). Case, punctuation and spacing are ignored;
/// the words must be there in order. `None` for a quote with fewer than
/// [`MIN_QUOTE`] letters and digits, or one that isn't there. Pure.
pub fn locate_quote(transcript: &str, quote: &str) -> Option<(usize, usize)> {
    let (needle, _) = flatten(quote);
    let needle: Vec<char> = {
        let mut n = needle;
        while n.last() == Some(&' ') {
            n.pop();
        }
        n
    };
    if needle.iter().filter(|c| **c != ' ').count() < MIN_QUOTE {
        return None;
    }
    let (hay, at) = flatten(transcript);
    if needle.len() > hay.len() {
        return None;
    }
    let start = (0..=hay.len() - needle.len()).find(|&s| hay[s..s + needle.len()] == needle[..])?;
    let first = at[start];
    let last = at[start + needle.len() - 1];
    Some((first, last + 1))
}

/// The quotes in `quotes` that are in the transcript, as ranges.
pub fn valid_quotes(transcript: &str, quotes: &[String]) -> Vec<(usize, usize)> {
    quotes
        .iter()
        .filter_map(|q| locate_quote(transcript, q))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    // no-prop-test: fixed sentences with known ranges.
    #[test]
    fn examples() {
        let t = "So we will ship the beta on the sixth, Priya writes the checklist.";
        let (s, e) = locate_quote(t, "Ship the BETA on the sixth").unwrap();
        assert_eq!(&t[s..e], "ship the beta on the sixth");
        assert!(locate_quote(t, "ship the beta on the seventh").is_none());
        assert!(locate_quote(t, "so").is_none(), "too short to count");
        assert!(locate_quote(t, "").is_none());
    }

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x0071_756f_7465),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        /// A stretch copied out of the transcript is found, inside it.
        #[test]
        fn a_copied_stretch_is_found_in_range(
            words in proptest::collection::vec("[a-z]{2,8}", 3..20),
            from in 0usize..10,
            len in 2usize..8,
        ) {
            let text = words.join(" ");
            let from = from % words.len();
            let to = (from + len).min(words.len());
            let quote = words[from..to].join(" ");
            if quote.chars().filter(|c| c.is_alphanumeric()).count() >= MIN_QUOTE {
                let (s, e) = locate_quote(&text, &quote).expect("copied text is found");
                prop_assert!(s < e && e <= text.chars().count());
            }
        }

        /// Whatever is asked, a range is always inside the transcript.
        #[test]
        fn a_range_is_always_inside(text in "\\PC{0,120}", quote in "\\PC{0,40}") {
            if let Some((s, e)) = locate_quote(&text, &quote) {
                prop_assert!(s < e && e <= text.chars().count());
            }
        }
    }
}
