//! What the PDF reader shows, as plain data (issue #43, steps 3 and 4).

use kanso_pdf::protocol::MAX_LONG_EDGE_PX;

/// How the reader's body looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReaderState {
    /// The PDF is being read from the library and handed to the worker.
    Loading,
    /// The PDF is encrypted and waits for its password. `wrong` is set when
    /// the one just given was not it.
    NeedsPassword { wrong: bool },
    /// The pages are on show.
    Showing {
        /// The page being read, 1-based.
        page: u32,
        pages: u32,
    },
    /// The PDF could not be opened or the reader stopped; the words are for
    /// the person reading, and "Try Again" starts a fresh worker.
    Failed(String),
}

impl ReaderState {
    /// "Page 3 of 12", or nothing while there is no page.
    pub fn page_label(&self) -> Option<String> {
        match self {
            ReaderState::Showing { page, pages } => Some(format!("Page {page} of {pages}")),
            _ => None,
        }
    }
}

/// What the reader says when the PDF is not a PDF Kanso can read, or the
/// reader is not part of this build.
pub const UNAVAILABLE: &str = "PDFs can't be read in this copy of Kanso.";
/// What the reader says when the file is gone or cannot be read.
pub const NOT_READABLE: &str = "This PDF couldn't be opened.";

/// A page is drawn at a multiple of this many pixels wide, so a window drag or
/// a small zoom step does not draw every page again.
pub const WIDTH_STEP: u32 = 64;
/// How wide a thumbnail is shown, in logical pixels.
pub const THUMB_WIDTH: f32 = 120.;

fn positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0. {
        value
    } else {
        fallback
    }
}

/// The width, in pixels, to ask the worker for a page shown `shown_width`
/// logical pixels wide on a display of `display_scale`: enough for every
/// device pixel, rounded up to a [`WIDTH_STEP`], and no more than the worker
/// will draw. Never zero.
pub fn render_width(shown_width: f32, display_scale: f32) -> u32 {
    let wanted = (positive(shown_width, 1.) * positive(display_scale, 1.)).ceil() as f64;
    let wanted = wanted.min(f64::from(MAX_LONG_EDGE_PX)) as u32;
    (wanted.div_ceil(WIDTH_STEP) * WIDTH_STEP).clamp(WIDTH_STEP, MAX_LONG_EDGE_PX)
}

/// The width to ask for a thumbnail on a display of `display_scale`.
pub fn thumb_render_width(display_scale: f32) -> u32 {
    ((THUMB_WIDTH * positive(display_scale, 1.)).ceil() as u32).max(1)
}

/// The `meta` key that holds "show this PDF as printed" for one attachment.
pub fn original_key(file_id: &str) -> String {
    format!("pdf.original.{file_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// The width asked for is never zero or over the worker's cap, covers
        /// the pixels on screen where the cap allows, and does not shrink when
        /// the page is shown larger.
        #[test]
        fn the_render_width_covers_the_screen_within_the_cap(
            shown in -10f32..3000.,
            display in prop_oneof![Just(f32::NAN), Just(0.), -2f32..4.],
            more in 0f32..500.,
        ) {
            let width = render_width(shown, display);
            prop_assert!(width >= 1 && width <= MAX_LONG_EDGE_PX);
            if shown >= 1. && display > 0. {
                let needed = (shown * display).ceil() as u32;
                prop_assert!(width >= needed.min(MAX_LONG_EDGE_PX));
                prop_assert!(render_width(shown + more, display) >= width);
            }
        }

        /// Widths come in steps, so small changes draw nothing again.
        #[test]
        fn widths_come_in_steps(shown in 1f32..2000., display in 1f32..3.) {
            let width = render_width(shown, display);
            prop_assert!(width % WIDTH_STEP == 0);
        }

        #[test]
        fn a_thumbnail_is_never_zero_wide(display in prop_oneof![Just(f32::NAN), Just(0.), -2f32..4.]) {
            prop_assert!(thumb_render_width(display) >= 1);
        }
    }

    #[test]
    fn the_label_names_the_page_only_while_one_shows() {
        let showing = ReaderState::Showing { page: 1, pages: 12 };
        assert_eq!(showing.page_label().as_deref(), Some("Page 1 of 12"));
        assert_eq!(ReaderState::Loading.page_label(), None);
        assert_eq!(ReaderState::Failed("x".into()).page_label(), None);
        assert_eq!(
            ReaderState::NeedsPassword { wrong: false }.page_label(),
            None
        );
    }

    #[test]
    fn the_original_toggle_is_kept_per_file() {
        assert_ne!(original_key("file:a"), original_key("file:b"));
    }
}
