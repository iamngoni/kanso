//! A note's outline, without a window: which heading the caret is under.
//!
//! The headings themselves are `kanso_core::outline::headings`, parsed from
//! the text the note view holds; this adds the one rule the inspector and
//! the phones' outline sheet both follow to mark where the person is.

pub use kanso_core::outline::{Heading, headings};

/// The heading the caret on `line` (0-based) is under: the last one at or
/// above it, as an index into `headings`. `None` above the first heading.
/// Pure.
pub fn current_heading(headings: &[Heading], line: u32) -> Option<usize> {
    headings.iter().rposition(|h| h.line <= line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn heading(line: u32) -> Heading {
        Heading {
            level: 1,
            text: format!("H{line}"),
            line,
        }
    }

    // no-prop-test: three fixed positions against a small outline; the
    // property over all outlines is the next test.
    #[test]
    fn the_caret_is_under_the_last_heading_at_or_above_it() {
        let outline = [heading(2), heading(10), heading(30)];
        assert_eq!(current_heading(&outline, 0), None);
        assert_eq!(current_heading(&outline, 2), Some(0));
        assert_eq!(current_heading(&outline, 29), Some(1));
        assert_eq!(current_heading(&outline, 30), Some(2));
        assert_eq!(current_heading(&outline, 500), Some(2));
        assert_eq!(current_heading(&[], 4), None);
    }

    proptest! {
        /// The answer is a heading at or above the line, and no later
        /// heading is also at or above it.
        #[test]
        fn prop_current_heading_is_the_last_one_at_or_above(
            mut lines in proptest::collection::vec(0u32..200, 0..12),
            at in 0u32..220,
        ) {
            lines.sort_unstable();
            let outline: Vec<Heading> = lines.iter().map(|&l| heading(l)).collect();
            match current_heading(&outline, at) {
                Some(ix) => {
                    prop_assert!(outline[ix].line <= at);
                    prop_assert!(outline[ix + 1..].iter().all(|h| h.line > at));
                }
                None => prop_assert!(outline.iter().all(|h| h.line > at)),
            }
        }

        /// On parsed text too: the heading found sits on or above the line.
        #[test]
        fn prop_current_heading_of_parsed_text(
            body in "(#{1,3} [a-z]{1,6}\n|[a-z ]{0,8}\n){0,12}",
            at in 0u32..16,
        ) {
            let outline = headings(&body);
            if let Some(ix) = current_heading(&outline, at) {
                prop_assert!(outline[ix].line <= at);
            }
        }
    }
}
