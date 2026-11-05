//! Smart invert for dark mode (issue #43, "Dark theme uses smart invert"):
//! a page drawn dark where it is text, and left alone where it is a picture.
//!
//! The worker draws the page as it is, then runs [`smart_invert`] over the
//! bitmap. Outside the pictures' boxes, every pixel keeps its hue and how
//! coloured it is (its chroma) and has its lightness turned over, so black
//! text becomes light text, the white page becomes a dark page, and a red
//! line stays red and a blue chart stays blue. A paper that is almost white
//! stays almost grey: keeping HSL saturation instead would blow a tint of
//! one level into a strong colour. Inside a picture's box the original pixels stay.
//! A scan, one picture that fills the page, counts as text and is inverted
//! whole, since leaving it would put a white sheet in a dark reader.
//!
//! Pure: pixels in, pixels out, no PDF engine, so it is testable alone.

use crate::protocol::BYTES_PER_PIXEL;

/// A picture covering at least this share of the page is a scan, and is
/// treated as text.
pub const SCAN_COVERAGE: f64 = 0.85;

/// The lightness a white page is turned into (a dark graphite, not black, so
/// the glare is gone and the page still has an edge against the window).
pub const PAGE_LIGHTNESS: f32 = 0.105;
/// The lightness black text is turned into (a soft white).
pub const INK_LIGHTNESS: f32 = 0.915;

/// A box of pixels: `left` and `top` included, `right` and `bottom` not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl PixelRect {
    /// The box of device coordinates (any two opposite corners, in any
    /// order), grown to whole pixels and clamped to a `width` x `height`
    /// bitmap. `None` when nothing of it is inside, or a coordinate is not a
    /// number.
    pub fn from_corners(
        a: (f32, f32),
        b: (f32, f32),
        width: u32,
        height: u32,
    ) -> Option<PixelRect> {
        if ![a.0, a.1, b.0, b.1].iter().all(|v| v.is_finite()) {
            return None;
        }
        let clamp = |v: f32, max: u32| f64::from(v).clamp(0.0, f64::from(max)) as u32;
        let rect = PixelRect {
            left: clamp(a.0.min(b.0).floor(), width),
            top: clamp(a.1.min(b.1).floor(), height),
            right: clamp(a.0.max(b.0).ceil(), width),
            bottom: clamp(a.1.max(b.1).ceil(), height),
        };
        (rect.right > rect.left && rect.bottom > rect.top).then_some(rect)
    }

    fn area(&self) -> u64 {
        u64::from(self.right - self.left) * u64::from(self.bottom - self.top)
    }
}

/// The boxes to keep as drawn: the pictures, less any that is a scan, and
/// less anything outside the bitmap.
pub fn keep_rects(pictures: &[PixelRect], width: u32, height: u32) -> Vec<PixelRect> {
    let page = u64::from(width) * u64::from(height);
    pictures
        .iter()
        .filter_map(|rect| {
            let inside = PixelRect {
                left: rect.left.min(width),
                top: rect.top.min(height),
                right: rect.right.min(width),
                bottom: rect.bottom.min(height),
            };
            (inside.right > inside.left && inside.bottom > inside.top).then_some(inside)
        })
        .filter(|rect| (rect.area() as f64) < SCAN_COVERAGE * page as f64)
        .collect()
}

/// Turns one pixel's lightness over, keeping its hue and chroma.
fn invert_pixel([b, g, r]: [u8; 3]) -> [u8; 3] {
    let (r, g, b) = (
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
    );
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let lightness = (max + min) / 2.0;
    let spread = max - min;
    let target = PAGE_LIGHTNESS + (1.0 - lightness) * (INK_LIGHTNESS - PAGE_LIGHTNESS);
    // The colour's offsets from grey keep their direction (the hue) and their
    // size (the chroma), as far as the new lightness has room for. Not the
    // HSL saturation: near white and near black it is huge for a tiny
    // spread, so an off-white paper (255, 255, 254) came out dark olive
    // (54, 54, 0) and a scan's paper turned to coloured speckle.
    let room = 1.0 - (2.0 * target - 1.0).abs();
    let scale = if spread <= f32::EPSILON {
        0.0
    } else {
        spread.min(room) / spread
    };
    let out = |v: f32| ((target + (v - lightness) * scale).clamp(0.0, 1.0) * 255.0).round() as u8;
    [out(b), out(g), out(r)]
}

/// Inverts a BGRA bitmap in place except inside `keep` (see [`keep_rects`]).
/// Alpha is untouched. A buffer that is not `width * height * 4` bytes is left
/// as it is.
pub fn smart_invert(bgra: &mut [u8], width: u32, height: u32, keep: &[PixelRect]) {
    let (w, h) = (width as usize, height as usize);
    if w == 0
        || w.checked_mul(h)
            .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL))
            != Some(bgra.len())
    {
        return;
    }
    // Greys are most of a page: one table for all of them.
    let greys: Vec<u8> = (0..=255u8).map(|v| invert_pixel([v, v, v])[0]).collect();
    for (y, row) in bgra.chunks_exact_mut(w * BYTES_PER_PIXEL).enumerate() {
        let y = y as u32;
        let mut spans: Vec<(usize, usize)> = keep
            .iter()
            .filter(|rect| rect.top <= y && y < rect.bottom)
            .map(|rect| (rect.left as usize, rect.right as usize))
            .collect();
        spans.sort_unstable();
        let mut from = 0usize;
        for (left, right) in spans.into_iter().chain(std::iter::once((w, w))) {
            let left = left.min(w);
            if left > from {
                invert_run(
                    &mut row[from * BYTES_PER_PIXEL..left * BYTES_PER_PIXEL],
                    &greys,
                );
            }
            from = from.max(right.min(w));
        }
    }
}

fn invert_run(run: &mut [u8], greys: &[u8]) {
    for pixel in run.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
        if pixel[0] == pixel[1] && pixel[1] == pixel[2] {
            let v = greys[usize::from(pixel[0])];
            pixel[..3].copy_from_slice(&[v, v, v]);
        } else {
            let [b, g, r] = invert_pixel([pixel[0], pixel[1], pixel[2]]);
            pixel[..3].copy_from_slice(&[b, g, r]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn bgra(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let mut out = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let [b, g, r] = f(x, y);
                out.extend_from_slice(&[b, g, r, 255]);
            }
        }
        out
    }

    fn hue(b: u8, g: u8, r: u8) -> f32 {
        let (r, g, b) = (f32::from(r), f32::from(g), f32::from(b));
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let d = max - min;
        if d == 0.0 {
            return 0.0;
        }
        let h = if max == r {
            ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h * 60.0
    }

    fn lightness(b: u8, g: u8, r: u8) -> f32 {
        (f32::from(r.max(g).max(b)) + f32::from(r.min(g).min(b))) / 510.0
    }

    #[test]
    fn white_becomes_the_dark_page_and_black_the_soft_white_ink() {
        let [b, ..] = invert_pixel([255, 255, 255]);
        assert!((f32::from(b) / 255.0 - PAGE_LIGHTNESS).abs() < 0.01);
        let [b, ..] = invert_pixel([0, 0, 0]);
        assert!((f32::from(b) / 255.0 - INK_LIGHTNESS).abs() < 0.01);
    }

    #[test]
    fn a_saturated_red_and_blue_stay_red_and_blue() {
        let [b, g, r] = invert_pixel([51, 51, 230]);
        assert!(r > g && r > b, "{r} {g} {b}");
        let [b, g, r] = invert_pixel([217, 115, 51]);
        assert!(b > g && b > r, "{r} {g} {b}");
    }

    /// Paper that is not quite white (an off-white, cream, the paper of a
    /// scan) and ink that is not quite black come out nearly grey: no more
    /// coloured than they went in.
    #[test]
    fn nearly_white_and_nearly_black_stay_nearly_grey() {
        for [b, g, r] in [
            [254, 255, 255],
            [253, 255, 254],
            [255, 252, 250],
            [240, 250, 255],
            [236, 244, 246],
            [0, 0, 2],
            [32, 31, 35],
        ] {
            let [b2, g2, r2] = invert_pixel([b, g, r]);
            let before = r.max(g).max(b) - r.min(g).min(b);
            let after = r2.max(g2).max(b2) - r2.min(g2).min(b2);
            assert!(after <= before + 1, "({r},{g},{b}) became ({r2},{g2},{b2})");
        }
        // And paper stays a dark page, ink a light one.
        assert!(invert_pixel([240, 250, 255]).iter().all(|&v| v < 60));
        assert!(invert_pixel([0, 0, 2]).iter().all(|&v| v > 200));
    }

    #[test]
    fn the_pictures_box_is_kept_and_the_rest_is_turned() {
        let (w, h) = (20, 10);
        let page = bgra(w, h, |_, _| [255, 255, 255]);
        let photo = PixelRect {
            left: 5,
            top: 2,
            right: 12,
            bottom: 8,
        };
        let mut out = page.clone();
        smart_invert(&mut out, w, h, &keep_rects(&[photo], w, h));
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let inside = (5..12).contains(&x) && (2..8).contains(&y);
                if inside {
                    assert_eq!(out[i..i + 4], page[i..i + 4]);
                } else {
                    assert!(out[i] < 60, "({x},{y}) is still light");
                    assert_eq!(out[i + 3], 255);
                }
            }
        }
    }

    #[test]
    fn a_scan_is_inverted_whole_and_a_small_picture_is_not() {
        let (w, h) = (40, 40);
        let scan = PixelRect {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        assert!(keep_rects(&[scan], w, h).is_empty());
        let almost = PixelRect {
            left: 1,
            top: 1,
            right: w - 1,
            bottom: h - 1,
        };
        assert!(keep_rects(&[almost], w, h).is_empty());
        let photo = PixelRect {
            left: 4,
            top: 4,
            right: 20,
            bottom: 20,
        };
        assert_eq!(keep_rects(&[scan, photo], w, h), vec![photo]);
    }

    #[test]
    fn a_buffer_of_the_wrong_length_is_left_alone() {
        let mut short = vec![255u8; 11];
        smart_invert(&mut short, 2, 2, &[]);
        assert_eq!(short, vec![255u8; 11]);
    }

    #[test]
    fn corners_become_a_clamped_box() {
        assert_eq!(
            PixelRect::from_corners((9.5, 1.2), (2.1, 6.9), 8, 5),
            Some(PixelRect {
                left: 2,
                top: 1,
                right: 8,
                bottom: 5
            })
        );
        assert_eq!(PixelRect::from_corners((20., 0.), (30., 4.), 8, 5), None);
        assert_eq!(
            PixelRect::from_corners((f32::NAN, 0.), (3., 4.), 8, 5),
            None
        );
        assert_eq!(PixelRect::from_corners((-4., -4.), (-1., -1.), 8, 5), None);
    }

    proptest! {
        /// Whatever the boxes (inside the bitmap or not), pixels inside a kept
        /// box are byte-for-byte what was drawn, alpha is never touched, and
        /// the bitmap keeps its length.
        #[test]
        fn kept_boxes_are_untouched_and_alpha_is_kept(
            w in 1u32..24,
            h in 1u32..24,
            seed in any::<u8>(),
            rects in prop::collection::vec((0u32..40, 0u32..40, 0u32..40, 0u32..40), 0..4),
        ) {
            let page = bgra(w, h, |x, y| {
                [(x * 11) as u8 ^ seed, (y * 7) as u8, (x + y) as u8 ^ seed.rotate_left(3)]
            });
            let rects: Vec<PixelRect> = rects
                .into_iter()
                .map(|(a, b, c, d)| PixelRect {
                    left: a.min(c),
                    top: b.min(d),
                    right: a.max(c),
                    bottom: b.max(d),
                })
                .collect();
            let keep = keep_rects(&rects, w, h);
            let mut out = page.clone();
            smart_invert(&mut out, w, h, &keep);
            prop_assert_eq!(out.len(), page.len());
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    prop_assert_eq!(out[i + 3], page[i + 3]);
                    if keep.iter().any(|r| r.left <= x && x < r.right && r.top <= y && y < r.bottom) {
                        prop_assert_eq!(&out[i..i + 4], &page[i..i + 4]);
                    }
                }
            }
        }

        /// A darker grey never comes out lighter than a lighter one: the page
        /// stays readable, only turned over.
        #[test]
        fn lightness_is_reversed_for_greys(a in any::<u8>(), b in any::<u8>()) {
            let (x, y) = (invert_pixel([a, a, a])[0], invert_pixel([b, b, b])[0]);
            if a < b { prop_assert!(x >= y) } else if a > b { prop_assert!(x <= y) }
        }

        /// A colour keeps its hue (within rounding) and stays coloured.
        #[test]
        fn colours_keep_their_hue(b in any::<u8>(), g in any::<u8>(), r in any::<u8>()) {
            let spread = r.max(g).max(b) - r.min(g).min(b);
            prop_assume!(spread >= 90);
            // Hue is unstable where the target is nearly grey.
            prop_assume!((0.2..0.8).contains(&lightness(b, g, r)));
            let [b2, g2, r2] = invert_pixel([b, g, r]);
            let diff = (hue(b, g, r) - hue(b2, g2, r2)).abs();
            prop_assert!(diff.min(360.0 - diff) < 12.0, "{diff}");
            let after = r2.max(g2).max(b2) - r2.min(g2).min(b2);
            prop_assert!(after >= 40, "{after}");
        }

        /// No pixel comes out more coloured than it went in (within
        /// rounding), so a faint tint is never blown up.
        #[test]
        fn a_colour_never_gets_more_colourful(b in any::<u8>(), g in any::<u8>(), r in any::<u8>()) {
            let [b2, g2, r2] = invert_pixel([b, g, r]);
            let before = r.max(g).max(b) - r.min(g).min(b);
            let after = r2.max(g2).max(b2) - r2.min(g2).min(b2);
            prop_assert!(after <= before + 1, "({r},{g},{b}) became ({r2},{g2},{b2})");
        }
    }
}
