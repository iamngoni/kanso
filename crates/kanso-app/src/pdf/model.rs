//! What the PDF reader shows, as plain data (issue #43, step 3).

/// How the reader's body looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReaderState {
    /// The PDF is being read from the library and handed to the worker.
    Loading,
    /// A page is on show.
    Showing {
        /// 1-based.
        page: u32,
        pages: u32,
        /// The page's drawn size in pixels, for its shape.
        width: u32,
        height: u32,
    },
    /// The PDF could not be opened or the reader stopped; the words are for
    /// the person reading, and "Try Again" starts a fresh worker.
    Failed(String),
}

impl ReaderState {
    /// "Page 3 of 12", or nothing while there is no page.
    pub fn page_label(&self) -> Option<String> {
        match self {
            ReaderState::Showing { page, pages, .. } => Some(format!("Page {page} of {pages}")),
            _ => None,
        }
    }
}

/// What the reader says when the PDF is not a PDF Kanso can read, or the
/// reader is not part of this build.
pub const UNAVAILABLE: &str = "PDFs can't be read in this copy of Kanso.";
/// What the reader says when the file is gone or cannot be read.
pub const NOT_READABLE: &str = "This PDF couldn't be opened.";

/// The width, in pixels, to ask the worker for: the reader's page column at
/// this display scale. The worker caps it again.
pub fn render_width(column_points: f32, scale: f32) -> u32 {
    let scale = if scale.is_finite() && scale > 0. {
        scale
    } else {
        1.
    };
    ((column_points.max(1.) * scale).round() as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// The width asked for is never zero and follows the display scale.
        #[test]
        fn the_render_width_is_positive_and_scales(
            points in -10f32..5000.,
            scale in prop_oneof![Just(f32::NAN), Just(0.), -2f32..4.],
        ) {
            let width = render_width(points, scale);
            prop_assert!(width >= 1);
            if scale > 0. && points >= 1. {
                prop_assert_eq!(width, ((points * scale).round() as u32).max(1));
            }
        }
    }

    #[test]
    fn the_label_names_the_page_only_while_one_shows() {
        let showing = ReaderState::Showing {
            page: 1,
            pages: 12,
            width: 10,
            height: 10,
        };
        assert_eq!(showing.page_label().as_deref(), Some("Page 1 of 12"));
        assert_eq!(ReaderState::Loading.page_label(), None);
        assert_eq!(ReaderState::Failed("x".into()).page_label(), None);
    }
}
