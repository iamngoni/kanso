//! Split mode's two sides: where the divider between the Markdown source
//! and its preview may sit, and how far the preview scrolls when the source
//! does.
//!
//! The divider is kept as a ratio of the editor's width, so it stays put
//! relative to the pane as the window changes. Scrolling is synced loosely:
//! the preview follows the source's share of its own scroll range, so both
//! start at the top and reach the end together, without matching blocks.

use crate::config::DEFAULT_SPLIT_RATIO;

/// Neither side of the split gets narrower than this, in points.
pub const SPLIT_MIN_SIDE: f32 = 280.;

/// What the source editor leaves below its last line, in lines, when half
/// its height is less (the editor's own rule).
const BOTTOM_ROOM_LINES: f32 = 3.;

/// The source side's width with the divider at `ratio` of an editor `width`
/// points wide. Each side keeps [`SPLIT_MIN_SIDE`] when there is room for
/// both, and they share a narrower editor equally. A ratio that isn't a
/// number counts as the default.
pub fn split_left(ratio: f32, width: f32) -> f32 {
    let width = if width.is_finite() { width.max(0.) } else { 0. };
    let ratio = if ratio.is_finite() {
        ratio.clamp(0., 1.)
    } else {
        DEFAULT_SPLIT_RATIO
    };
    if width < 2. * SPLIT_MIN_SIDE {
        return width / 2.;
    }
    (ratio * width).clamp(SPLIT_MIN_SIDE, width - SPLIT_MIN_SIDE)
}

/// The narrowest either side may be dragged in an editor `width` points
/// wide: [`SPLIT_MIN_SIDE`], or half the editor when that is less, so the
/// two minimums always fit and the preview never runs past the window.
pub fn split_min_side(width: f32) -> f32 {
    SPLIT_MIN_SIDE.min(width.max(0.) / 2.)
}

/// The ratio to keep for a source side `left` points wide in an editor
/// `width` points wide: between 0 and 1, and the default when the width
/// isn't known.
pub fn split_ratio(left: f32, width: f32) -> f32 {
    if !(width.is_finite() && width > 0. && left.is_finite()) {
        return DEFAULT_SPLIT_RATIO;
    }
    (left / width).clamp(0., 1.)
}

/// How far the preview scrolls, from 0 to `dst_max`, when the source is
/// scrolled `src` of its `src_max`: the same share of each. A source that
/// can't scroll leaves the preview at the top.
pub fn scroll_sync(src: f32, src_max: f32, dst_max: f32) -> f32 {
    if !(src_max.is_finite() && src_max > 0. && dst_max.is_finite() && dst_max > 0.) {
        return 0.;
    }
    let src = if src.is_finite() { src } else { 0. };
    (src.clamp(0., src_max) / src_max * dst_max).clamp(0., dst_max)
}

/// The rows lines of these lengths (in characters) take when each wraps
/// every `columns` characters: an empty line still takes one.
pub fn wrapped_rows(line_lengths: impl IntoIterator<Item = usize>, columns: usize) -> usize {
    let columns = columns.max(1);
    line_lengths
        .into_iter()
        .map(|len| len.div_ceil(columns).max(1))
        .sum()
}

/// About how far the source editor can scroll, in points: its `rows` at
/// `line_height` each, plus the room it leaves below the last line (half
/// the viewport, at least three lines), less the `viewport` itself. The
/// editor doesn't say how tall its text is, so the rows are estimated with
/// [`wrapped_rows`]; the preview only needs to follow loosely.
pub fn source_scroll_max(rows: usize, line_height: f32, viewport: f32) -> f32 {
    if !(line_height.is_finite() && line_height > 0. && viewport.is_finite()) {
        return 0.;
    }
    let viewport = viewport.max(0.);
    let room = (viewport / 2.).max(BOTTOM_ROOM_LINES * line_height);
    (rows as f32 * line_height + room - viewport).max(0.)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Ratios anywhere, including outside 0..1 and not numbers at all.
    fn any_ratio() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => -0.5f32..1.5,
            1 => Just(f32::NAN),
            1 => Just(f32::INFINITY),
            1 => Just(0.),
            1 => Just(1.),
        ]
    }

    /// How far f32 arithmetic on scroll offsets of this size may drift: a
    /// few units in the last place of the largest value involved, never
    /// less than the old fixed 0.01 point. A fixed 0.01 is below one ulp
    /// (0.0156) once a note is a few thousand rows tall.
    fn scroll_tolerance(text: f32, viewport: f32) -> f32 {
        let largest = text.abs().max(viewport.abs()).max(1.);
        (8. * largest * f32::EPSILON).max(1e-2)
    }

    /// The shrunk case from a flaky run (issue 206): 4432 rows of 38.09573
    /// in a 514.66406 viewport. The room below the last line comes out
    /// 0.0195 under half the viewport, which is f32 rounding of a sum near
    /// 170000 (spacing 0.0156), not a wrong estimate; the check's fixed
    /// 0.01 tolerance was under that spacing.
    #[test]
    fn the_scroll_range_of_a_tall_note_holds_to_float_precision() {
        let (rows, line_height, viewport) = (4432usize, 38.09573_f32, 514.66406_f32);
        let max = source_scroll_max(rows, line_height, viewport);
        let text = rows as f32 * line_height;
        let tol = scroll_tolerance(text, viewport);
        assert!(max + viewport >= text - tol, "{max} + {viewport} < {text}");
        let below = max + viewport - text;
        assert!(below >= 3. * line_height - tol, "{below}");
        assert!(below >= viewport / 2. - tol, "{below} < half of {viewport}");
        assert!(source_scroll_max(rows + 1, line_height, viewport) >= max);
        assert!(source_scroll_max(rows, line_height, viewport) <= max + tol);
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: the source side fits the editor; with room for both,
        /// each side keeps its minimum; a narrower editor is shared equally.
        #[test]
        fn prop_both_sides_keep_their_minimum(ratio in any_ratio(), width in 0f32..3000.) {
            let left = split_left(ratio, width);
            prop_assert!((0. ..=width).contains(&left), "{} of {}", left, width);
            if width >= 2. * SPLIT_MIN_SIDE {
                prop_assert!(left >= SPLIT_MIN_SIDE, "{}", left);
                prop_assert!(width - left >= SPLIT_MIN_SIDE - 1e-3, "{} of {}", left, width);
            } else {
                prop_assert_eq!(left, width / 2.);
            }
        }

        /// Monotonicity, and the divider sits exactly where the ratio says
        /// whenever that leaves both sides their minimum.
        #[test]
        fn prop_the_divider_follows_the_ratio(a in 0f32..=1., b in 0f32..=1., width in 560f32..3000.) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(split_left(lo, width) <= split_left(hi, width));
            let wanted = lo * width;
            if (SPLIT_MIN_SIDE..=width - SPLIT_MIN_SIDE).contains(&wanted) {
                prop_assert_eq!(split_left(lo, width), wanted);
            }
        }

        /// Round trip: the ratio kept for a divider puts it back where it
        /// was at the same width, and is always a usable ratio.
        #[test]
        fn prop_kept_ratios_put_the_divider_back(left in -100f32..3000., width in 560f32..3000.) {
            let ratio = split_ratio(left, width);
            prop_assert!((0. ..=1.).contains(&ratio));
            let placed = split_left(ratio, width);
            if (SPLIT_MIN_SIDE..=width - SPLIT_MIN_SIDE).contains(&left) {
                prop_assert!((placed - left).abs() < 1e-2, "{} became {}", left, placed);
            }
            // Placing it again changes nothing.
            prop_assert!((split_left(split_ratio(placed, width), width) - placed).abs() < 1e-2);
        }

        /// The preview stays within its range, follows the source in the same
        /// direction, and the ends meet: top to top, bottom to bottom.
        #[test]
        fn prop_scroll_sync_maps_ends_to_ends(
            a in -100f32..5000.,
            b in -100f32..5000.,
            src_max in 0f32..5000.,
            dst_max in 0f32..5000.,
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let at = |src| scroll_sync(src, src_max, dst_max);
            prop_assert!(at(lo) <= at(hi));
            prop_assert!((0. ..=dst_max).contains(&at(lo)));
            prop_assert_eq!(at(0.), 0.);
            if src_max > 0. {
                prop_assert!((at(src_max) - dst_max).abs() <= dst_max * 1e-6);
                // Metamorphic: twice the preview's range, twice as far.
                prop_assert!((scroll_sync(lo, src_max, 2. * dst_max) - 2. * at(lo)).abs() <= 1e-2);
            } else {
                prop_assert_eq!(at(hi), 0.);
            }
        }

        /// Rows: at least one per line, exactly one per line when nothing
        /// wraps, more for longer lines and fewer for wider columns.
        #[test]
        fn prop_wrapped_rows_grow_with_the_text(
            lines in prop::collection::vec(0usize..400, 0..40),
            extra in 0usize..400,
            columns in 1usize..120,
            wider in 0usize..60,
        ) {
            let rows = wrapped_rows(lines.iter().copied(), columns);
            prop_assert!(rows >= lines.len());
            let longest = lines.iter().copied().max().unwrap_or(0);
            prop_assert_eq!(wrapped_rows(lines.iter().copied(), longest.max(1)), lines.len());
            let mut more = lines.clone();
            more.push(extra);
            prop_assert!(wrapped_rows(more.iter().copied(), columns) > rows);
            let mut longer = lines.clone();
            if let Some(first) = longer.first_mut() {
                *first += extra;
            }
            prop_assert!(wrapped_rows(longer.iter().copied(), columns) >= rows);
            if extra > columns && !lines.is_empty() {
                prop_assert!(wrapped_rows(longer.iter().copied(), columns) > rows);
            }
            prop_assert!(wrapped_rows(lines.iter().copied(), columns + wider) <= rows);
        }

        /// The estimated range is never negative and grows with the rows;
        /// a taller viewport never adds to it; scrolled to the end, the
        /// last row is in view with at least three lines of room below it;
        /// a viewport three lines tall or more never scrolls past the text
        /// itself; and a note in the top half of a tall viewport can't
        /// scroll at all.
        #[test]
        fn prop_source_scroll_max_keeps_the_last_line_in_reach(
            rows in 0usize..5000,
            more in 1usize..100,
            line_height in 10f32..40.,
            viewport in 0f32..1500.,
            taller in 0f32..800.,
        ) {
            let max = source_scroll_max(rows, line_height, viewport);
            let text = rows as f32 * line_height;
            let tol = scroll_tolerance(text, viewport);
            prop_assert!(max >= 0.);
            prop_assert!(source_scroll_max(rows + more, line_height, viewport) >= max);
            prop_assert!(source_scroll_max(rows, line_height, viewport + taller) <= max + tol);
            prop_assert!(max + viewport >= text - tol, "{} + {} < {}", max, viewport, text);
            // The editor's own rule, written out rather than borrowed from
            // the code under test: three lines, or half the viewport.
            if max > 0. {
                let below = max + viewport - text;
                prop_assert!(below >= 3. * line_height - tol, "{}", below);
                prop_assert!(below >= viewport / 2. - tol, "{}", below);
            }
            if viewport >= 3. * line_height {
                prop_assert!(max <= text + tol, "{} past {}", max, text);
            }
            if viewport >= 6. * line_height && text <= viewport / 2. {
                prop_assert_eq!(max, 0.);
            }
        }

        /// The drag minimum never exceeds the fixed one, two of them always
        /// fit the editor, it is the fixed one whenever both sides have
        /// room, it grows with the editor, and the placed divider keeps it.
        #[test]
        fn prop_split_min_side_fits_the_editor(
            ratio in any_ratio(),
            a in -100f32..3000.,
            b in -100f32..3000.,
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let min = split_min_side(lo);
            prop_assert!((0. ..=SPLIT_MIN_SIDE).contains(&min));
            prop_assert!(2. * min <= lo.max(0.));
            if lo >= 2. * SPLIT_MIN_SIDE {
                prop_assert_eq!(min, SPLIT_MIN_SIDE);
            }
            prop_assert!(min <= split_min_side(hi));
            let width = lo.max(0.);
            let left = split_left(ratio, width);
            prop_assert!(left >= min - 1e-3 && width - left >= min - 1e-3, "{} of {}", left, width);
        }
    }

    #[test]
    fn split_examples() {
        assert_eq!(split_left(0.5, 1000.), 500.);
        assert_eq!(split_left(0.1, 1000.), SPLIT_MIN_SIDE);
        assert_eq!(split_left(0.9, 1000.), 1000. - SPLIT_MIN_SIDE);
        assert_eq!(split_left(0.8, 400.), 200.);
        assert_eq!(split_left(f32::NAN, 1000.), 500.);
        assert_eq!(split_ratio(300., 1000.), 0.3);
        assert_eq!(split_ratio(300., 0.), DEFAULT_SPLIT_RATIO);
        assert_eq!(split_ratio(f32::NAN, 1000.), DEFAULT_SPLIT_RATIO);
        assert_eq!(split_min_side(1000.), SPLIT_MIN_SIDE);
        // A 480 point editor, as a narrow window leaves it: 240 a side.
        assert_eq!(split_min_side(480.), 240.);
        assert_eq!(split_min_side(-10.), 0.);
        assert_eq!(split_min_side(f32::NAN), 0.);
    }

    #[test]
    fn scroll_sync_examples() {
        assert_eq!(scroll_sync(0., 1000., 400.), 0.);
        assert_eq!(scroll_sync(500., 1000., 400.), 200.);
        assert_eq!(scroll_sync(1000., 1000., 400.), 400.);
        // Past the estimated end the preview waits at its own.
        assert_eq!(scroll_sync(1200., 1000., 400.), 400.);
        assert_eq!(scroll_sync(300., 0., 400.), 0.);
        assert_eq!(scroll_sync(300., 1000., 0.), 0.);
        assert_eq!(scroll_sync(f32::NAN, 1000., 400.), 0.);
    }

    #[test]
    fn row_examples() {
        assert_eq!(wrapped_rows([0, 10, 81, 160], 80), 1 + 1 + 2 + 2);
        assert_eq!(wrapped_rows(std::iter::empty(), 80), 0);
        assert_eq!(wrapped_rows([5], 0), 5);
        // 100 rows of 20 points in a 600 point viewport, 300 below the last.
        assert_eq!(source_scroll_max(100, 20., 600.), 1700.);
        assert_eq!(source_scroll_max(10, 20., 600.), 0.);
        // A short viewport keeps three lines below the last: 2000 + 60 - 40.
        assert_eq!(source_scroll_max(100, 20., 40.), 2020.);
    }
}
