//! The worker protocol (issue #43, step 3): what the app and the helper
//! process say to each other over the helper's stdin and stdout.
//!
//! A frame is `[json_len: u32 LE][blob_len: u32 LE][json][blob]`. The JSON
//! holds the small metadata; the blob holds raw bytes (the PDF going in, a
//! page's BGRA pixels coming out), so a page is never base64. Both lengths
//! are checked against the reader's limits before anything is allocated, so
//! a worker that has been taken over cannot make the app allocate more than
//! it agreed to.
//!
//! Pure: no PDF engine and no process here, so the codec is testable alone.

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

/// Pages a document may have; a longer one is refused.
pub const MAX_PAGES: u32 = 2_000;
/// The longest side a page may declare, in points: PDF's 200-inch limit
/// (14,400) scaled by the largest user unit PDFium honours, with room to
/// spare. A document with a bigger page is refused rather than laid out: a
/// declared size is the PDF's own claim, and the reader would size its
/// scroll column by it.
pub const MAX_PAGE_POINTS: f32 = 75_000.0;
/// The longest side of a rendered page, in pixels.
pub const MAX_LONG_EDGE_PX: u32 = 4_096;
/// The most pixel bytes one rendered page may carry (64 MB of BGRA).
pub const MAX_PAGE_BYTES: usize = 64 * 1024 * 1024;
/// The most JSON metadata one frame may carry.
pub const MAX_JSON_BYTES: usize = 1024 * 1024;
/// The most a frame's blob may carry: a PDF at its size limit, or a page.
pub const MAX_BLOB_BYTES: usize = kanso_core::model::MAX_PDF_SIZE;

/// Bytes per pixel of the worker's bitmaps: blue, green, red, alpha.
pub const BYTES_PER_PIXEL: usize = 4;

/// What the app asks of the worker.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Opens the document held in the frame's blob. A password is used for
    /// this open and never kept.
    Open {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    /// Draws a page, `width_px` wide (the worker clamps it, see
    /// [`render_size`]).
    Render { page: u32, width_px: u32 },
    /// Closes the document; the worker answers [`Response::Closed`] and exits.
    Close,
}

/// What the worker answers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// The document opened. `sizes` are the pages' sizes in points, as shown
    /// (rotation applied), in page order.
    Opened {
        pages: u32,
        sizes: Vec<[f32; 2]>,
        encrypted: bool,
    },
    /// A page drawn; the frame's blob is `width * height * 4` bytes of BGRA.
    Page { page: u32, width: u32, height: u32 },
    /// The document is closed.
    Closed,
    /// The request failed. The worker stays up for everything but a failed
    /// open.
    Error { kind: ErrorKind, message: String },
}

/// Why a request failed, for the reader to word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Not a PDF PDFium can read.
    Unreadable,
    /// It is encrypted and needs a password.
    PasswordRequired,
    /// The password given is wrong.
    WrongPassword,
    /// More than [`MAX_PAGES`] pages.
    TooManyPages,
    /// A page declares a size beyond [`MAX_PAGE_POINTS`].
    PageTooLarge,
    /// The file is longer than the size limit.
    FileTooLarge,
    /// No such page.
    NoSuchPage,
    /// A request out of order or malformed.
    Protocol,
    /// Anything else PDFium or the worker could not do.
    Internal,
}

/// Why a frame could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// The pipe closed or failed. At the start of a frame this is how a
    /// worker that died looks.
    #[error("The PDF reader stopped: {0}")]
    Io(#[from] io::Error),
    /// A length beyond what the reader agreed to take.
    #[error("The PDF reader sent more than it may.")]
    TooLarge,
    /// JSON that is not a known message.
    #[error("The PDF reader sent something unreadable.")]
    Malformed,
}

/// Writes one frame: the message as JSON, then the blob.
pub fn write_frame<M: Serialize>(
    out: &mut impl Write,
    message: &M,
    blob: &[u8],
) -> Result<(), FrameError> {
    let json = serde_json::to_vec(message).map_err(|_| FrameError::Malformed)?;
    if json.len() > MAX_JSON_BYTES || blob.len() > u32::MAX as usize {
        return Err(FrameError::TooLarge);
    }
    let mut head = [0u8; 8];
    head[..4].copy_from_slice(&(json.len() as u32).to_le_bytes());
    head[4..].copy_from_slice(&(blob.len() as u32).to_le_bytes());
    out.write_all(&head)?;
    out.write_all(&json)?;
    out.write_all(blob)?;
    out.flush()?;
    Ok(())
}

/// Reads one frame, refusing a blob longer than `max_blob`. `Ok(None)` is a
/// clean end of the stream before the frame began.
pub fn read_frame<M: for<'de> Deserialize<'de>>(
    input: &mut impl Read,
    max_blob: usize,
) -> Result<Option<(M, Vec<u8>)>, FrameError> {
    let mut head = [0u8; 8];
    // The first byte tells a clean end from a cut-off head.
    match input.read(&mut head[..1])? {
        0 => return Ok(None),
        _ => input.read_exact(&mut head[1..])?,
    }
    let json_len = u32::from_le_bytes([head[0], head[1], head[2], head[3]]) as usize;
    let blob_len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
    if json_len > MAX_JSON_BYTES || blob_len > max_blob {
        return Err(FrameError::TooLarge);
    }
    let json = read_exactly(input, json_len)?;
    let message = serde_json::from_slice(&json).map_err(|_| FrameError::Malformed)?;
    let blob = read_exactly(input, blob_len)?;
    Ok(Some((message, blob)))
}

/// Reads exactly `len` bytes, growing the buffer as they arrive. A length
/// field is a claim: a peer that says 50 MB and sends five bytes costs the
/// reader five bytes, not 50 MB.
fn read_exactly(input: &mut impl Read, len: usize) -> Result<Vec<u8>, FrameError> {
    const FIRST_CHUNK: usize = 64 * 1024;
    let mut buf = Vec::with_capacity(len.min(FIRST_CHUNK));
    let read = input.take(len as u64).read_to_end(&mut buf)?;
    if read != len {
        return Err(FrameError::Io(io::ErrorKind::UnexpectedEof.into()));
    }
    Ok(buf)
}

/// Whether a page may declare this size: both sides finite, above zero and
/// within [`MAX_PAGE_POINTS`]. `Err` says how it fails.
pub fn check_page_size(width_pt: f32, height_pt: f32) -> Result<(), ErrorKind> {
    if !(width_pt.is_finite() && height_pt.is_finite()) || width_pt <= 0.0 || height_pt <= 0.0 {
        return Err(ErrorKind::Unreadable);
    }
    if width_pt > MAX_PAGE_POINTS || height_pt > MAX_PAGE_POINTS {
        return Err(ErrorKind::PageTooLarge);
    }
    Ok(())
}

/// Whether a document may be shown: no more than [`MAX_PAGES`] pages, one
/// size per page, and every size within [`check_page_size`]. The worker
/// checks it before it answers and the app checks it again on the answer.
pub fn check_document(pages: u32, sizes: &[[f32; 2]]) -> Result<(), ErrorKind> {
    if pages > MAX_PAGES {
        return Err(ErrorKind::TooManyPages);
    }
    if sizes.len() != pages as usize {
        return Err(ErrorKind::Protocol);
    }
    sizes.iter().try_for_each(|[w, h]| check_page_size(*w, *h))
}

/// The pixel size to draw a page at: `width_px` wide, the page's shape, then
/// scaled down until the long side is within [`MAX_LONG_EDGE_PX`] and the
/// bitmap within [`MAX_PAGE_BYTES`]. Never zero, never larger than asked.
/// `None` for a page with no usable size.
pub fn render_size(page_width_pt: f32, page_height_pt: f32, width_px: u32) -> Option<(u32, u32)> {
    if !(page_width_pt.is_finite() && page_height_pt.is_finite())
        || page_width_pt <= 0.0
        || page_height_pt <= 0.0
    {
        return None;
    }
    let ratio = f64::from(page_height_pt) / f64::from(page_width_pt);
    let mut width = f64::from(width_px.max(1));
    let mut height = width * ratio;
    let long = width.max(height);
    if long > f64::from(MAX_LONG_EDGE_PX) {
        let scale = f64::from(MAX_LONG_EDGE_PX) / long;
        width *= scale;
        height *= scale;
    }
    let bytes = width * height * BYTES_PER_PIXEL as f64;
    if bytes > MAX_PAGE_BYTES as f64 {
        let scale = (MAX_PAGE_BYTES as f64 / bytes).sqrt();
        width *= scale;
        height *= scale;
    }
    // Rounding down keeps both limits; a sliver of a page stays one pixel.
    let width = (width.floor() as u32).max(1);
    let height = (height.floor() as u32).max(1);
    Some((width, height))
}

/// Whether a page the worker sent is what it said and within the caps: the
/// blob is exactly `width * height * 4` bytes and no larger than a page may
/// be. The app checks this before it builds a texture.
pub fn page_is_sound(width: u32, height: u32, blob_len: usize) -> bool {
    width >= 1
        && height >= 1
        && width <= MAX_LONG_EDGE_PX
        && height <= MAX_LONG_EDGE_PX
        && (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL))
            .is_some_and(|bytes| bytes == blob_len && bytes <= MAX_PAGE_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_request() -> impl Strategy<Value = Request> {
        prop_oneof![
            proptest::option::of(".{0,40}").prop_map(|password| Request::Open { password }),
            (any::<u32>(), any::<u32>())
                .prop_map(|(page, width_px)| Request::Render { page, width_px }),
            Just(Request::Close),
        ]
    }

    fn arb_kind() -> impl Strategy<Value = ErrorKind> {
        prop_oneof![
            Just(ErrorKind::Unreadable),
            Just(ErrorKind::PasswordRequired),
            Just(ErrorKind::WrongPassword),
            Just(ErrorKind::TooManyPages),
            Just(ErrorKind::PageTooLarge),
            Just(ErrorKind::FileTooLarge),
            Just(ErrorKind::NoSuchPage),
            Just(ErrorKind::Protocol),
            Just(ErrorKind::Internal),
        ]
    }

    fn arb_response() -> impl Strategy<Value = Response> {
        prop_oneof![
            (
                0u32..50,
                proptest::collection::vec([0.5f32..2000.0, 0.5f32..2000.0], 0..8),
                any::<bool>()
            )
                .prop_map(|(pages, sizes, encrypted)| Response::Opened {
                    pages,
                    sizes,
                    encrypted
                }),
            (any::<u32>(), 1u32..5000, 1u32..5000).prop_map(|(page, width, height)| {
                Response::Page {
                    page,
                    width,
                    height,
                }
            }),
            Just(Response::Closed),
            (arb_kind(), ".{0,60}").prop_map(|(kind, message)| Response::Error { kind, message }),
        ]
    }

    proptest! {
        /// Whatever is written reads back identical, blob included, and the
        /// stream is then at its clean end.
        #[test]
        fn a_request_frame_round_trips(
            request in arb_request(),
            blob in proptest::collection::vec(any::<u8>(), 0..2048),
        ) {
            let mut wire = Vec::new();
            write_frame(&mut wire, &request, &blob).unwrap();
            let mut input = wire.as_slice();
            let (back, back_blob) = read_frame::<Request>(&mut input, MAX_BLOB_BYTES)
                .unwrap()
                .unwrap();
            prop_assert_eq!(back, request);
            prop_assert_eq!(back_blob, blob);
            prop_assert!(read_frame::<Request>(&mut input, MAX_BLOB_BYTES).unwrap().is_none());
        }

        #[test]
        fn a_response_frame_round_trips(
            response in arb_response(),
            blob in proptest::collection::vec(any::<u8>(), 0..2048),
        ) {
            let mut wire = Vec::new();
            write_frame(&mut wire, &response, &blob).unwrap();
            let (back, back_blob) = read_frame::<Response>(&mut wire.as_slice(), MAX_BLOB_BYTES)
                .unwrap()
                .unwrap();
            prop_assert_eq!(back, response);
            prop_assert_eq!(back_blob, blob);
        }

        /// Frames written one after another read back in order.
        #[test]
        fn frames_in_a_row_stay_apart(
            requests in proptest::collection::vec(
                (arb_request(), proptest::collection::vec(any::<u8>(), 0..64)), 0..8),
        ) {
            let mut wire = Vec::new();
            for (request, blob) in &requests {
                write_frame(&mut wire, request, blob).unwrap();
            }
            let mut input = wire.as_slice();
            for (request, blob) in &requests {
                let (back, back_blob) = read_frame::<Request>(&mut input, MAX_BLOB_BYTES)
                    .unwrap()
                    .unwrap();
                prop_assert_eq!(&back, request);
                prop_assert_eq!(&back_blob, blob);
            }
            prop_assert!(read_frame::<Request>(&mut input, MAX_BLOB_BYTES).unwrap().is_none());
        }

        /// Bytes that are not a frame never panic and never allocate past
        /// the limits: they are an error or a message.
        #[test]
        fn arbitrary_bytes_are_refused_or_read(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
            let _ = read_frame::<Response>(&mut bytes.as_slice(), 1024);
        }

        /// A frame cut anywhere short is an error, never a message.
        #[test]
        fn a_cut_frame_is_an_error(
            request in arb_request(),
            blob in proptest::collection::vec(any::<u8>(), 0..64),
            cut in any::<prop::sample::Index>(),
        ) {
            let mut wire = Vec::new();
            write_frame(&mut wire, &request, &blob).unwrap();
            let keep = cut.index(wire.len());
            let result = read_frame::<Request>(&mut &wire[..keep], MAX_BLOB_BYTES);
            if keep == 0 {
                prop_assert!(matches!(result, Ok(None)));
            } else {
                prop_assert!(result.is_err());
            }
        }

        /// A blob longer than the reader's limit is refused from its length
        /// alone, before it is read.
        #[test]
        fn a_blob_over_the_limit_is_refused(limit in 0usize..64, extra in 1usize..64) {
            let mut wire = Vec::new();
            write_frame(&mut wire, &Request::Close, &vec![7u8; limit + extra]).unwrap();
            let result = read_frame::<Request>(&mut wire.as_slice(), limit);
            prop_assert!(matches!(result, Err(FrameError::TooLarge)));
        }

        /// Whatever size is asked for, a page is never drawn past the caps,
        /// never to nothing, and keeps its shape to a pixel.
        #[test]
        fn a_render_size_is_always_within_the_caps(
            w in 1.0f32..20_000.0,
            h in 1.0f32..20_000.0,
            width_px in any::<u32>(),
        ) {
            let Some((rw, rh)) = render_size(w, h, width_px) else {
                return Err(TestCaseError::fail("a usable page has a size"));
            };
            prop_assert!(rw >= 1 && rh >= 1);
            prop_assert!(rw <= MAX_LONG_EDGE_PX && rh <= MAX_LONG_EDGE_PX);
            prop_assert!(rw as usize * rh as usize * BYTES_PER_PIXEL <= MAX_PAGE_BYTES);
            prop_assert!(rw <= width_px.max(1));
            prop_assert!(page_is_sound(rw, rh, rw as usize * rh as usize * BYTES_PER_PIXEL));
            // Shape: each side was cut by less than a pixel, so the ratio is
            // off by less than 1/width + 1/height of itself (the scaling by
            // the caps is the same on both sides).
            let want = f64::from(w) / f64::from(h);
            let got = f64::from(rw) / f64::from(rh);
            let slack = 1.0 / f64::from(rw) + 1.0 / f64::from(rh);
            prop_assert!((got / want - 1.0).abs() <= slack + 1e-6, "{rw}x{rh} for {w}x{h}");
        }

        /// A size asked for within the caps is given as asked.
        #[test]
        fn a_modest_request_is_given_as_asked(width_px in 1u32..1000) {
            let (rw, rh) = render_size(600.0, 800.0, width_px).unwrap();
            prop_assert_eq!(rw, width_px);
            prop_assert!(rh <= (f64::from(width_px) * 800.0 / 600.0).ceil() as u32);
        }

        /// A page that may be declared may be drawn: whatever the size
        /// check lets through, `render_size` gives a sound bitmap for.
        #[test]
        fn a_page_that_passes_the_size_check_can_be_drawn(
            w in 0.0001f32..20_000.0,
            h in 0.0001f32..20_000.0,
            width_px in any::<u32>(),
        ) {
            match check_page_size(w, h) {
                Ok(()) => {
                    prop_assert!(w <= MAX_PAGE_POINTS && h <= MAX_PAGE_POINTS);
                    let (rw, rh) = render_size(w, h, width_px).unwrap();
                    prop_assert!(page_is_sound(rw, rh, rw as usize * rh as usize * BYTES_PER_PIXEL));
                }
                Err(kind) => {
                    prop_assert_eq!(kind, ErrorKind::PageTooLarge);
                    prop_assert!(w > MAX_PAGE_POINTS || h > MAX_PAGE_POINTS);
                }
            }
        }

        /// A size that is not a number, or not above zero, is never a page.
        #[test]
        fn a_page_without_a_usable_size_is_unreadable(
            good in 1.0f32..1000.0,
            bad in prop_oneof![
                Just(f32::NAN), Just(f32::INFINITY), Just(f32::NEG_INFINITY),
                Just(0.0f32), Just(-0.0f32), -1.0e9f32..0.0,
            ],
        ) {
            prop_assert_eq!(check_page_size(bad, good), Err(ErrorKind::Unreadable));
            prop_assert_eq!(check_page_size(good, bad), Err(ErrorKind::Unreadable));
        }

        /// A document is shown only with its page count within the cap and
        /// one usable size for each page; any one bad page refuses it.
        #[test]
        fn a_document_is_checked_page_by_page(
            sizes in proptest::collection::vec([1.0f32..2000.0, 1.0f32..2000.0], 0..40),
            bad_at in any::<prop::sample::Index>(),
            bad in prop_oneof![Just(f32::NAN), Just(0.0f32), Just(1.0e9f32)],
        ) {
            let pages = sizes.len() as u32;
            prop_assert_eq!(check_document(pages, &sizes), Ok(()));
            // A count that disagrees with the sizes is a protocol failure.
            prop_assert_eq!(check_document(pages + 1, &sizes), Err(ErrorKind::Protocol));
            if !sizes.is_empty() {
                let mut spoiled = sizes.clone();
                spoiled[bad_at.index(sizes.len())][0] = bad;
                prop_assert!(check_document(pages, &spoiled).is_err());
            }
        }

        /// Past the page cap a document is refused whatever its sizes.
        #[test]
        fn too_many_pages_are_refused_before_their_sizes_are_looked_at(
            pages in (MAX_PAGES + 1)..u32::MAX,
        ) {
            prop_assert_eq!(check_document(pages, &[]), Err(ErrorKind::TooManyPages));
        }

        /// Exactly the cap is allowed.
        #[test]
        fn the_page_cap_itself_is_allowed(w in 1.0f32..2000.0, h in 1.0f32..2000.0) {
            let sizes = vec![[w, h]; MAX_PAGES as usize];
            prop_assert_eq!(check_document(MAX_PAGES, &sizes), Ok(()));
        }

        /// A frame that claims a long blob and ends early is an error after
        /// reading what was there, not an allocation of what was claimed.
        #[test]
        fn a_short_blob_is_an_error_not_an_allocation(
            sent in 0usize..200,
            claimed in 1_000_000usize..MAX_BLOB_BYTES,
        ) {
            let json = br#"{"type":"close"}"#;
            let mut wire = Vec::new();
            wire.extend_from_slice(&(json.len() as u32).to_le_bytes());
            wire.extend_from_slice(&(claimed as u32).to_le_bytes());
            wire.extend_from_slice(json);
            wire.extend(std::iter::repeat_n(9u8, sent));
            let result = read_frame::<Request>(&mut wire.as_slice(), MAX_BLOB_BYTES);
            prop_assert!(matches!(result, Err(FrameError::Io(_))));
        }

        #[test]
        fn a_page_that_lies_about_its_size_is_unsound(
            w in 1u32..5000, h in 1u32..5000, off in 1usize..100,
        ) {
            let right = w as usize * h as usize * BYTES_PER_PIXEL;
            prop_assert!(!page_is_sound(w, h, right + off));
            prop_assert!(!page_is_sound(w, h, right.saturating_sub(off).min(right - 1)));
        }
    }

    #[test]
    fn a_page_without_a_size_cannot_be_drawn() {
        assert_eq!(render_size(0.0, 100.0, 500), None);
        assert_eq!(render_size(100.0, f32::NAN, 500), None);
        assert_eq!(render_size(-1.0, 100.0, 500), None);
        assert_eq!(render_size(f32::INFINITY, 100.0, 500), None);
    }

    #[test]
    fn a_huge_ask_is_cut_to_the_long_edge() {
        let (w, h) = render_size(612.0, 792.0, 100_000).unwrap();
        assert_eq!(h, MAX_LONG_EDGE_PX);
        assert!(w < h);
    }

    #[test]
    fn unknown_messages_are_malformed() {
        let mut wire = Vec::new();
        let json = br#"{"type":"format_disk"}"#;
        wire.extend_from_slice(&(json.len() as u32).to_le_bytes());
        wire.extend_from_slice(&0u32.to_le_bytes());
        wire.extend_from_slice(json);
        assert!(matches!(
            read_frame::<Request>(&mut wire.as_slice(), 10),
            Err(FrameError::Malformed)
        ));
    }
}
