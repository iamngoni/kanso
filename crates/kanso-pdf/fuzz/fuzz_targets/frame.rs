//! Fuzzes the worker protocol's decoder (issue #43, step 9): the bytes the
//! app reads from the worker, and the bytes the worker reads from the app.
//! Whatever arrives, the decoder returns a message or an error. It never
//! panics, never allocates past its limits, and what it accepts it can
//! write and read back.
//!
//! `cargo +nightly fuzz run frame` from `crates/kanso-pdf`; `scripts/pdf-fuzz.sh`
//! seeds it and runs it for a while.

#![no_main]

use kanso_pdf::protocol::{
    BYTES_PER_PIXEL, MAX_BLOB_BYTES, MAX_LONG_EDGE_PX, MAX_PAGE_BYTES, MAX_PAGE_CHARS, Request,
    Response, check_document, check_page_size, is_tidy, page_is_sound, plain_text_pages,
    read_frame, render_size, text_page_is_sound, text_window, write_frame,
};
use kanso_pdf::text::PageText;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The worker's side of the pipe: requests, with the PDF at its limit.
    let mut input = data;
    if let Ok(Some((request, blob))) = read_frame::<Request>(&mut input, MAX_BLOB_BYTES) {
        assert!(blob.len() <= MAX_BLOB_BYTES);
        // What was accepted writes out and reads back the same.
        let mut wire = Vec::new();
        write_frame(&mut wire, &request, &blob).expect("an accepted request writes");
        let (again, again_blob) = read_frame::<Request>(&mut wire.as_slice(), MAX_BLOB_BYTES)
            .expect("a written request reads")
            .expect("and is a frame");
        assert_eq!(again, request);
        assert_eq!(again_blob, blob);
    }

    // The app's side: responses, with a page at its limit. Every frame in a
    // row is read until the input ends or a frame is refused.
    let mut input = data;
    while let Ok(Some((response, blob))) = read_frame::<Response>(&mut input, MAX_PAGE_BYTES) {
        assert!(blob.len() <= MAX_PAGE_BYTES);
        match response {
            Response::Opened { pages, sizes, .. } => {
                // The app's checks never panic, whatever the numbers.
                let _ = check_document(pages, &sizes);
                for [w, h] in sizes {
                    let _ = check_page_size(w, h);
                    if let Some((rw, rh)) = render_size(w, h, pages.wrapping_mul(97)) {
                        assert!(rw >= 1 && rh >= 1);
                        assert!(rw <= MAX_LONG_EDGE_PX && rh <= MAX_LONG_EDGE_PX);
                        assert!(rw as usize * rh as usize * BYTES_PER_PIXEL <= MAX_PAGE_BYTES);
                    }
                }
            }
            Response::Page { width, height, .. } => {
                // A page is believed only if the bytes are exactly its size.
                if page_is_sound(width, height, blob.len()) {
                    assert_eq!(
                        blob.len(),
                        width as usize * height as usize * BYTES_PER_PIXEL
                    );
                    assert!(blob.len() <= MAX_PAGE_BYTES);
                }
            }
            Response::TextPage { page, chars, .. } => {
                // A page's text is believed only if it is the page's, as
                // long as the header says and within the cap.
                if let Ok(text) = serde_json::from_slice::<PageText>(&blob) {
                    if text_page_is_sound(page, [612.0, 792.0], chars, &text) {
                        assert!(text.chars.len() <= MAX_PAGE_CHARS);
                        assert_eq!(text.chars.len(), chars as usize);
                        assert!(text.chars.iter().all(|c| c.quad.is_valid() && c.turns < 4));
                    }
                }
            }
            Response::PlainText { from, lens, .. } => {
                // Plain text is believed only as the window asked for, in
                // pieces that add up to the bytes and are tidy.
                for ask in [(from, 1), (from, u32::MAX), (0, lens.len() as u32)] {
                    if let Some(pages) = plain_text_pages(from.saturating_add(70), ask, from, &lens, &blob) {
                        let window = text_window(from.saturating_add(70), ask.0, ask.1).unwrap();
                        assert_eq!(pages.len(), window.len());
                        assert_eq!(pages.iter().map(String::len).sum::<usize>(), blob.len());
                        assert!(pages.iter().all(|p| is_tidy(p)));
                    }
                }
            }
            Response::Closed | Response::Error { .. } => {}
        }
    }

    // Tight limits: a blob over the limit is refused from its length alone.
    for max_blob in [0, 16, 4096] {
        if let Ok(Some((_, blob))) = read_frame::<Request>(&mut &*data, max_blob) {
            assert!(blob.len() <= max_blob);
        }
    }
});
