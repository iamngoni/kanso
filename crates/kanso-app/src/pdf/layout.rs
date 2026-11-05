//! Where each page sits in the continuous column, and which are on screen
//! (issue #43, step 4). Pure arithmetic over the pages' sizes in points, so
//! the maths is tested without a window.
//!
//! Units: page sizes are PDF points; a layout is built at one `scale` (logical
//! pixels per point), so its positions are the reader's own pixels, apart from
//! the app's zoom.

use std::ops::Range;

/// The space between two pages, in logical pixels.
pub const PAGE_GAP: f32 = 12.;
/// The space around the column, in logical pixels.
pub const MARGIN: f32 = 24.;
/// The widest a page grows under Fit width, in logical pixels: a wide window
/// gets a readable column, not a poster.
pub const FIT_MAX_WIDTH: f32 = 960.;
/// The size used for a page whose size is not a usable number (a US Letter).
const FALLBACK_SIZE: [f32; 2] = [612., 792.];

/// The reader's zoom: Fit width, or a percentage of the page's real size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zoom {
    FitWidth,
    Percent(u32),
}

impl Zoom {
    /// The smallest and largest zoom, in percent.
    pub const RANGE: (u32, u32) = (50, 400);
    /// What Zoom In and Zoom Out step through, in percent.
    pub const STEPS: [u32; 9] = [50, 75, 100, 125, 150, 200, 250, 300, 400];
    /// Actual size.
    pub const ACTUAL: Zoom = Zoom::Percent(100);

    /// A percentage held within [`Zoom::RANGE`].
    pub fn percent(percent: u32) -> Zoom {
        Zoom::Percent(percent.clamp(Self::RANGE.0, Self::RANGE.1))
    }

    /// The next step in (`direction` > 0) or out (< 0) from the zoom now in
    /// force, `shown_percent` being what it comes to on screen (which is the
    /// only number Fit width has). Never leaves [`Zoom::RANGE`]; at an end it
    /// stays.
    pub fn step(shown_percent: u32, direction: i32) -> Zoom {
        let next = if direction >= 0 {
            Self::STEPS
                .iter()
                .copied()
                .find(|step| *step > shown_percent)
                .unwrap_or(Self::RANGE.1)
        } else {
            Self::STEPS
                .iter()
                .rev()
                .copied()
                .find(|step| *step < shown_percent)
                .unwrap_or(Self::RANGE.0)
        };
        Zoom::percent(next)
    }

    /// Logical pixels per point for a column `avail` pixels wide of pages up to
    /// `widest` points wide.
    pub fn scale(self, widest: f32, avail: f32) -> f32 {
        match self {
            Zoom::Percent(percent) => percent as f32 / 100.,
            Zoom::FitWidth => {
                let widest = if widest.is_finite() && widest > 0. {
                    widest
                } else {
                    FALLBACK_SIZE[0]
                };
                let width = avail.max(1.).min(FIT_MAX_WIDTH);
                (width / widest).max(0.05)
            }
        }
    }
}

/// A place in the document that survives a change of scale: a page and how far
/// down it, from 0 (its top) to 1 (its bottom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub page: usize,
    pub fraction: f32,
}

/// The column of pages at one scale.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// Each page's size in logical pixels.
    sizes: Vec<[f32; 2]>,
    /// Where each page starts, from the top of the column; one extra entry
    /// is the column's height.
    tops: Vec<f32>,
}

impl Layout {
    /// Pages at `scale` logical pixels per point, a `gap` between them and a
    /// `pad` above the first and below the last.
    pub fn new(points: &[[f32; 2]], scale: f32, gap: f32, pad: f32) -> Layout {
        let scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        Layout::from_sizes(
            points
                .iter()
                .map(|size| {
                    let [w, h] = usable(*size);
                    [w * scale, h * scale]
                })
                .collect(),
            gap,
            pad,
        )
    }

    /// Pages all `width` pixels wide, each as tall as its shape makes it:
    /// the thumbnail rail.
    pub fn uniform_width(points: &[[f32; 2]], width: f32, gap: f32, pad: f32) -> Layout {
        let width = if width.is_finite() && width > 0. {
            width
        } else {
            1.
        };
        Layout::from_sizes(
            points
                .iter()
                .map(|size| {
                    let [w, h] = usable(*size);
                    [width, (width * h / w).max(1.)]
                })
                .collect(),
            gap,
            pad,
        )
    }

    fn from_sizes(sizes: Vec<[f32; 2]>, gap: f32, pad: f32) -> Layout {
        let mut tops = Vec::with_capacity(sizes.len() + 1);
        let mut y = pad.max(0.);
        for size in &sizes {
            tops.push(y);
            y += size[1] + gap.max(0.);
        }
        // The last page has no gap after it, only the padding.
        if !sizes.is_empty() {
            y = y - gap.max(0.) + pad.max(0.);
        }
        tops.push(y);
        Layout { sizes, tops }
    }

    pub fn pages(&self) -> usize {
        self.sizes.len()
    }

    /// The height of the whole column.
    pub fn total_height(&self) -> f32 {
        self.tops.last().copied().unwrap_or(0.)
    }

    /// The width of the widest page.
    pub fn max_width(&self) -> f32 {
        self.sizes.iter().map(|size| size[0]).fold(0., f32::max)
    }

    pub fn top(&self, page: usize) -> f32 {
        self.tops.get(page).copied().unwrap_or(self.total_height())
    }

    pub fn size(&self, page: usize) -> [f32; 2] {
        self.sizes.get(page).copied().unwrap_or([0., 0.])
    }

    /// The pages that show in a window of `height` pixels whose top is
    /// `scroll_top` pixels down the column: every page that has any of itself
    /// in it, and no other.
    pub fn visible(&self, scroll_top: f32, height: f32) -> Range<usize> {
        let n = self.pages();
        if n == 0 || !scroll_top.is_finite() || !height.is_finite() || height <= 0. {
            return 0..0;
        }
        let (top, bottom) = (scroll_top, scroll_top + height);
        // The first page whose bottom is below the window's top.
        let first = self.partition(|page| self.top(page) + self.size(page)[1] <= top);
        // The first page that starts at or below the window's bottom.
        let end = self.partition(|page| self.top(page) < bottom);
        if first >= end { 0..0 } else { first..end }
    }

    /// The first page for which `before` is false, for a `before` that is true
    /// for a prefix of the pages.
    fn partition(&self, before: impl Fn(usize) -> bool) -> usize {
        let (mut low, mut high) = (0, self.pages());
        while low < high {
            let mid = (low + high) / 2;
            if before(mid) {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low
    }

    /// The page the person is reading: the one with the most of the window,
    /// the earlier on a tie.
    pub fn current(&self, scroll_top: f32, height: f32) -> usize {
        let visible = self.visible(scroll_top, height);
        let (top, bottom) = (scroll_top, scroll_top + height);
        let mut best = (visible.start, f32::MIN);
        for page in visible {
            let shown = (self.top(page) + self.size(page)[1]).min(bottom) - self.top(page).max(top);
            if shown > best.1 {
                best = (page, shown);
            }
        }
        if self.pages() == 0 { 0 } else { best.0.min(self.pages() - 1) }
    }

    /// The anchor at `y` pixels down the column.
    pub fn anchor_at(&self, y: f32) -> Anchor {
        if self.pages() == 0 {
            return Anchor {
                page: 0,
                fraction: 0.,
            };
        }
        // The last page that starts at or above `y`.
        let page = self
            .partition(|page| self.top(page) <= y)
            .saturating_sub(1)
            .min(self.pages() - 1);
        let height = self.size(page)[1].max(1.);
        Anchor {
            page,
            fraction: ((y - self.top(page)) / height).clamp(0., 1.),
        }
    }

    /// How far down the column an anchor is, at this layout's scale.
    pub fn offset_of(&self, anchor: Anchor) -> f32 {
        self.top(anchor.page) + anchor.fraction.clamp(0., 1.) * self.size(anchor.page)[1]
    }

    /// The farthest the window can be scrolled.
    pub fn max_scroll(&self, height: f32) -> f32 {
        (self.total_height() - height.max(0.)).max(0.)
    }
}

fn usable(size: [f32; 2]) -> [f32; 2] {
    if size.iter().all(|v| v.is_finite() && *v > 0.) {
        size
    } else {
        FALLBACK_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn sizes() -> impl Strategy<Value = Vec<[f32; 2]>> {
        prop::collection::vec([50f32..1500., 50f32..1500.], 0..60)
    }

    proptest! {
        /// Pages are in order, never overlap, and the column is as tall as the
        /// pages, the gaps and the padding make it.
        #[test]
        fn pages_stack_in_order_without_overlap(
            sizes in sizes(),
            scale in 0.1f32..4.,
            gap in 0f32..30.,
            pad in 0f32..40.,
        ) {
            let layout = Layout::new(&sizes, scale, gap, pad);
            prop_assert_eq!(layout.pages(), sizes.len());
            for page in 1..layout.pages() {
                let end = layout.top(page - 1) + layout.size(page - 1)[1];
                prop_assert!(layout.top(page) >= end - 0.01);
            }
            if !sizes.is_empty() {
                let tall: f32 = sizes.iter().map(|s| s[1] * scale).sum::<f32>()
                    + gap * (sizes.len() - 1) as f32
                    + 2. * pad;
                prop_assert!((layout.total_height() - tall).abs() < 0.05 * (1. + tall.abs() * 1e-3), "{} vs {tall}", layout.total_height());
            }
        }

        /// The visible range is exactly the pages that intersect the window.
        #[test]
        fn the_visible_range_is_exactly_the_pages_in_the_window(
            sizes in sizes(),
            scale in 0.1f32..3.,
            scroll in -200f32..60_000.,
            height in 1f32..3000.,
        ) {
            let layout = Layout::new(&sizes, scale, PAGE_GAP, MARGIN);
            let range = layout.visible(scroll, height);
            prop_assert!(range.end <= layout.pages());
            for page in 0..layout.pages() {
                let (top, bottom) = (layout.top(page), layout.top(page) + layout.size(page)[1]);
                let intersects = top < scroll + height && bottom > scroll;
                prop_assert_eq!(range.contains(&page), intersects, "page {}", page);
            }
        }

        /// Scrolling down never moves the visible range back.
        #[test]
        fn the_visible_range_only_moves_forward_with_the_scroll(
            sizes in sizes(),
            a in 0f32..40_000.,
            b in 0f32..40_000.,
            height in 1f32..2000.,
        ) {
            let layout = Layout::new(&sizes, 1., PAGE_GAP, MARGIN);
            let (a, b) = (a.min(b), a.max(b));
            let (ra, rb) = (layout.visible(a, height), layout.visible(b, height));
            if !ra.is_empty() && !rb.is_empty() {
                prop_assert!(ra.start <= rb.start && ra.end <= rb.end);
            }
        }

        /// The current page is one of the visible ones, and the only page
        /// when there is one.
        #[test]
        fn the_current_page_is_a_visible_one(
            sizes in prop::collection::vec([50f32..1500., 50f32..1500.], 1..40),
            scroll in 0f32..40_000.,
            height in 1f32..2000.,
        ) {
            let layout = Layout::new(&sizes, 1., PAGE_GAP, MARGIN);
            let current = layout.current(scroll, height);
            prop_assert!(current < layout.pages());
            let visible = layout.visible(scroll, height);
            if !visible.is_empty() {
                prop_assert!(visible.contains(&current));
            }
        }

        /// A place in the document keeps its page and its fraction through a
        /// change of scale, so a zoom does not lose the reader's place.
        #[test]
        fn an_anchor_survives_a_change_of_scale(
            sizes in prop::collection::vec([50f32..1500., 50f32..1500.], 1..40),
            y in 0f32..50_000.,
            from in 0.25f32..4.,
            to in 0.25f32..4.,
        ) {
            let before = Layout::new(&sizes, from, PAGE_GAP, MARGIN);
            let y = y.min(before.total_height());
            let anchor = before.anchor_at(y);
            let after = Layout::new(&sizes, to, PAGE_GAP, MARGIN);
            let moved = after.anchor_at(after.offset_of(anchor));
            prop_assert_eq!(moved.page, anchor.page);
            prop_assert!((moved.fraction - anchor.fraction).abs() < 0.01);
            // And at the same scale it is the same place.
            let same = before.offset_of(anchor);
            prop_assert!((same - y).abs() < 0.05 * (1. + y * 1e-3) || anchor.fraction >= 1. || anchor.fraction <= 0.);
        }

        /// Thumbnails are all the one width and keep each page's shape.
        #[test]
        fn thumbnails_are_one_width_and_keep_the_shape(sizes in sizes(), width in 20f32..300.) {
            let layout = Layout::uniform_width(&sizes, width, 8., 8.);
            for (page, size) in sizes.iter().enumerate() {
                let [w, h] = layout.size(page);
                prop_assert!((w - width).abs() < 0.001);
                prop_assert!((h - (width * size[1] / size[0]).max(1.)).abs() < 0.01);
            }
        }

        /// Zoom steps stay in range, and go the way they were asked.
        #[test]
        fn zoom_steps_stay_in_range(shown in 1u32..900, up in any::<bool>()) {
            let zoom = Zoom::step(shown, if up { 1 } else { -1 });
            let Zoom::Percent(percent) = zoom else { return Err(TestCaseError::fail("not a percentage")) };
            prop_assert!((Zoom::RANGE.0..=Zoom::RANGE.1).contains(&percent));
            if up && shown < Zoom::RANGE.1 { prop_assert!(percent > shown.min(Zoom::RANGE.1 - 1)) }
            if !up && shown > Zoom::RANGE.0 { prop_assert!(percent < shown.max(Zoom::RANGE.0 + 1)) }
        }

        /// Fit width never makes a page wider than the column or the cap.
        #[test]
        fn fit_width_fits(widest in 100f32..2000., avail in 100f32..3000.) {
            let scale = Zoom::FitWidth.scale(widest, avail);
            prop_assert!(widest * scale <= avail.min(FIT_MAX_WIDTH) + 0.01);
            prop_assert!(scale > 0.);
        }
    }

    #[test]
    fn an_empty_document_has_nothing_to_show() {
        let layout = Layout::new(&[], 1., PAGE_GAP, MARGIN);
        assert_eq!(layout.visible(0., 800.), 0..0);
        assert_eq!(layout.total_height(), 0.);
        assert_eq!(layout.current(0., 800.), 0);
    }

    #[test]
    fn a_page_that_is_not_a_size_is_a_letter_page() {
        let layout = Layout::new(&[[f32::NAN, 10.], [0., 0.]], 1., 0., 0.);
        assert_eq!(layout.size(0), FALLBACK_SIZE);
        assert_eq!(layout.size(1), FALLBACK_SIZE);
    }

    #[test]
    fn zoom_steps_walk_the_ladder() {
        assert_eq!(Zoom::step(100, 1), Zoom::Percent(125));
        assert_eq!(Zoom::step(100, -1), Zoom::Percent(75));
        assert_eq!(Zoom::step(118, 1), Zoom::Percent(125));
        assert_eq!(Zoom::step(118, -1), Zoom::Percent(100));
        assert_eq!(Zoom::step(400, 1), Zoom::Percent(400));
        assert_eq!(Zoom::step(50, -1), Zoom::Percent(50));
        assert_eq!(Zoom::percent(9999), Zoom::Percent(400));
    }
}
