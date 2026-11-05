//! A rendered page as a GPUI texture (issue #43, "Page images in GPUI").
//!
//! PDFium draws BGRA, which is what GPUI's `RenderImage` holds, so the bytes
//! go in as they are: no PNG, no decode, no channel swap. `image::RgbaImage`
//! is only the container type; its "red" byte is the page's blue.

use std::sync::Arc;

use gpui_kit::RenderImage;
use image::{Frame, RgbaImage};
use kanso_pdf::client::RenderedPage;

/// The page as a texture, or `None` when its bytes are not `width * height *
/// 4` (the client checks this too; a texture must never be built from less).
pub fn texture(page: RenderedPage) -> Option<Arc<RenderImage>> {
    let RenderedPage {
        width,
        height,
        bgra,
        ..
    } = page;
    // `from_raw` accepts a buffer that is too long; a texture is exact.
    if bgra.len()
        != (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?
    {
        return None;
    }
    let buffer = RgbaImage::from_raw(width, height, bgra)?;
    Some(Arc::new(RenderImage::new(vec![Frame::new(buffer)])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn page(width: u32, height: u32, bgra: Vec<u8>) -> RenderedPage {
        RenderedPage {
            page: 0,
            width,
            height,
            bgra,
        }
    }

    proptest! {
        /// The texture holds the very bytes the worker sent, in the same
        /// order, at the size the worker said.
        #[test]
        fn the_pixels_arrive_untouched(
            width in 1u32..40,
            height in 1u32..40,
            seed in any::<u8>(),
        ) {
            let bytes: Vec<u8> = (0..width * height * 4)
                .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
                .collect();
            let image = texture(page(width, height, bytes.clone())).unwrap();
            prop_assert_eq!(image.as_bytes(0).unwrap(), bytes.as_slice());
            let size = image.size(0);
            prop_assert_eq!((size.width.0 as u32, size.height.0 as u32), (width, height));
        }

        /// Bytes that don't fill the page never become a texture.
        #[test]
        fn a_short_or_long_buffer_is_refused(
            width in 1u32..30,
            height in 1u32..30,
            off in 1usize..8,
            longer in any::<bool>(),
        ) {
            let right = (width * height * 4) as usize;
            let len = if longer { right + off } else { right.saturating_sub(off) };
            prop_assume!(len != right);
            prop_assert!(texture(page(width, height, vec![0; len])).is_none());
        }
    }
}
