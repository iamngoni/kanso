//! Text from the worker, as a real process (issue #43, steps 5 and 7): a
//! page's characters with boxes that land on the glyphs, the plain text of a
//! document in reading order, rotated pages and text, and the caps.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(all(feature = "render", any(target_os = "macos", target_os = "linux")))]

use std::time::{Duration, Instant};

use kanso_pdf::client::{Worker, WorkerError};
use kanso_pdf::fixtures::{encrypted_pdf, pages_pdf, pages_pdf_on, report_pdf, sample_pdf};
use kanso_pdf::protocol::{ErrorKind, MAX_PAGE_CHARS, MAX_TEXT_WINDOW};
use kanso_pdf::text::{PageText, layout_page};

mod common;

use common::paths;

fn open(bytes: &[u8]) -> Worker {
    let mut worker = Worker::start(&paths()).expect("the worker starts");
    worker.open(bytes, None).expect("the PDF opens");
    worker
}

fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// The letters the text layer reads off the page, in its reading order, with
/// no spacing: direction and order are what is checked here (where it puts
/// spaces is tested on its own, in `text/tests.rs`).
fn read_back(text: &PageText) -> String {
    layout_page(text).glyphs.iter().map(|g| g.ch).collect()
}

/// Dark pixels of a BGRA page, as `(x, y)`.
fn ink(bgra: &[u8], width: usize) -> Vec<(usize, usize)> {
    bgra.as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, p)| p[0] < 200 && p[1] < 200 && p[2] < 200)
        .map(|(i, _)| (i % width, i / width))
        .collect()
}

/// Draws the page 1 pixel to the point and checks the boxes against it:
/// every box has ink in it, and (on a page that draws nothing but text) no
/// ink lies outside all the boxes. Returns
/// how many characters were checked.
fn assert_boxes_land_on_the_glyphs(
    worker: &mut Worker,
    page: u32,
    text: &PageText,
    only_text_is_drawn: bool,
) -> usize {
    let width = text.width.round() as u32;
    let drawn = worker.render(page, width).unwrap();
    let (w, h) = (drawn.width as f64, drawn.height as f64);
    let dark = ink(&drawn.bgra, drawn.width as usize);
    assert!(
        !dark.is_empty(),
        "the page drew no text, so this proves nothing"
    );
    // A pixel of slack each way: the box is tight, the edges anti-aliased.
    let slack = 1.5;
    let rect = |c: &kanso_pdf::text::TextChar| {
        let [x0, y0, x1, y1] = c.quad.0;
        (
            x0 * w - slack,
            y0 * h - slack,
            x1 * w + slack,
            y1 * h + slack,
        )
    };
    let inside = |(x, y): (usize, usize), r: (f64, f64, f64, f64)| {
        let (x, y) = (x as f64 + 0.5, y as f64 + 0.5);
        x >= r.0 && x <= r.2 && y >= r.1 && y <= r.3
    };
    for c in &text.chars {
        let r = rect(c);
        assert!(
            dark.iter().any(|p| inside(*p, r)),
            "no ink in the box of {:?}: {:?}",
            c.ch,
            c.quad
        );
    }
    let stray: Vec<_> = dark
        .iter()
        .filter(|p| only_text_is_drawn && !text.chars.iter().any(|c| inside(**p, rect(c))))
        .take(5)
        .collect();
    assert!(stray.is_empty(), "ink outside every box: {stray:?}");
    text.chars.len()
}

#[test]
fn plain_text_comes_back_in_reading_order() {
    let first = [
        "BT /F1 14 Tf 72 700 Td (Alpha bravo charlie) Tj ET",
        "BT /F1 14 Tf 72 680 Td (Delta echo foxtrot) Tj ET",
        "BT /F1 14 Tf 72 660 Td (Golf hotel india) Tj ET",
    ]
    .join("\n");
    let second = "BT /F1 14 Tf 72 700 Td (Second page words) Tj ET".to_owned();
    let mut worker = open(&pages_pdf(&[first, second]));
    let pages = worker.plain_text(0, 10).expect("the text arrives");
    assert_eq!(pages.len(), 2);
    assert_eq!(
        words(&pages[0]),
        [
            "Alpha", "bravo", "charlie", "Delta", "echo", "foxtrot", "Golf", "hotel", "india"
        ]
    );
    // Lines stay lines, and nothing stray comes with them.
    assert_eq!(
        pages[0].lines().collect::<Vec<_>>(),
        [
            "Alpha bravo charlie",
            "Delta echo foxtrot",
            "Golf hotel india"
        ],
        "{:?}",
        pages[0]
    );
    assert_eq!(pages[1].trim(), "Second page words");
    assert!(!pages[0].contains('\r') && !pages[0].contains('\u{fffe}'));
    // The same document in one go, for search.
    let all = worker
        .document_text(Duration::from_secs(30), &|| false)
        .unwrap();
    assert_eq!(all.pages, pages);
    assert!(!all.truncated);
}

#[test]
fn a_whole_document_is_read_window_by_window() {
    let count = MAX_TEXT_WINDOW as usize * 2 + 5;
    let mut worker = open(&sample_pdf(count));
    // One window is cut to the window cap and the end of the document.
    let window = worker.plain_text(0, u32::MAX).unwrap();
    assert_eq!(window.len(), MAX_TEXT_WINDOW as usize);
    let tail = worker.plain_text(count as u32 - 3, 50).unwrap();
    assert_eq!(tail.len(), 3);
    let all = worker
        .document_text(Duration::from_secs(60), &|| false)
        .unwrap();
    assert_eq!(all.pages.len(), count);
    for (i, page) in all.pages.iter().enumerate() {
        assert_eq!(page.trim(), format!("Kanso page {}", i + 1));
    }
    // Past the end is a plain refusal, and the worker goes on.
    match worker.plain_text(count as u32, 1) {
        Err(WorkerError::Refused { kind, .. }) => assert_eq!(kind, ErrorKind::NoSuchPage),
        other => panic!("{other:?}"),
    }
    assert!(worker.failure().is_none());
    assert!(worker.plain_text(0, 1).is_ok());
}

#[test]
fn a_pages_boxes_land_on_its_glyphs() {
    let mut worker = open(&report_pdf());
    for page in 0..3 {
        let text = worker.text(page).expect("the page's text arrives");
        assert_eq!(text.page, page + 1);
        assert_eq!((text.width, text.height, text.rotation), (612.0, 792.0, 0));
        assert!(!text.chars.is_empty());
        assert!(text.chars.iter().all(|c| c.turns == 0 && c.quad.is_valid()));
        let checked = assert_boxes_land_on_the_glyphs(&mut worker, page, &text, page > 0);
        assert_eq!(checked, text.chars.len());
    }
    // The first page's heading, where it was set: 30 pt bold at (72, 720).
    let text = worker.text(0).unwrap();
    let heading: String = text.chars.iter().take(9).map(|c| c.ch).collect();
    assert_eq!(heading, "Quarterly");
    let q = text.chars[0].quad.0;
    assert!((q[0] - 72.0 / 612.0).abs() < 0.01, "left of the Q: {q:?}");
    let (top, bottom) = (q[1] * 792.0, q[3] * 792.0);
    assert!(
        top > 792.0 - 720.0 - 30.0 && bottom < 792.0 - 720.0 + 10.0,
        "the Q sits {top}..{bottom} from the top"
    );
    // The layout reads the page back as it was written.
    let page = read_back(&text);
    assert!(page.starts_with("QuarterlyReport"), "{page:?}");
    assert!(page.contains("Revenuegrew12percent"), "{page:?}");
}

/// A page rotated by `/Rotate`, with a media box away from the origin: the
/// boxes are on the page as shown, and the text layer reads it back.
#[test]
fn a_rotated_page_reports_its_rotation_and_its_boxes_stay_on_the_glyphs() {
    let content = "BT /F1 36 Tf 172 700 Td (Kanso page 1) Tj ET".to_owned();
    for rotation in [0u16, 90, 180, 270] {
        let bytes = pages_pdf_on(std::slice::from_ref(&content), "[100 50 712 842]", rotation);
        let mut worker = open(&bytes);
        let text = worker.text(0).unwrap();
        assert_eq!(text.rotation, rotation);
        let (w, h) = if rotation % 180 == 0 {
            (612.0, 792.0)
        } else {
            (792.0, 612.0)
        };
        assert_eq!((text.width, text.height), (w, h), "rotated {rotation}");
        // The text is upright on the unrotated page: no character turns.
        assert!(text.chars.iter().all(|c| c.turns == 0));
        let checked = assert_boxes_land_on_the_glyphs(&mut worker, 0, &text, true);
        assert_eq!(checked, "Kansopage1".len(), "rotated {rotation}");
        // The text layer turns the page's rotation into reading order.
        assert_eq!(read_back(&text), "Kansopage1", "rotated {rotation}");
    }
}

/// Text that runs sideways on the page reports which way, and a rotated
/// page adds its own turn on top of it.
#[test]
fn rotated_text_reports_its_direction() {
    // 90 degrees counter-clockwise: bottom to top, which is 3 turns.
    let up = "BT /F1 24 Tf 0 1 -1 0 300 200 Tm (Upward text) Tj ET".to_owned();
    // 90 degrees clockwise: top to bottom, 1 turn.
    let down = "BT /F1 24 Tf 0 -1 1 0 300 600 Tm (Downward text) Tj ET".to_owned();
    // Upside down: 2 turns.
    let flipped = "BT /F1 24 Tf -1 0 0 -1 400 400 Tm (Flipped text) Tj ET".to_owned();
    for (content, turns, said) in [
        (up, 3u8, "Upward text"),
        (down, 1, "Downward text"),
        (flipped, 2, "Flipped text"),
    ] {
        for rotation in [0u16, 90, 270] {
            let bytes = pages_pdf_on(std::slice::from_ref(&content), "[0 0 612 792]", rotation);
            let mut worker = open(&bytes);
            let text = worker.text(0).unwrap();
            assert_eq!(text.rotation, rotation);
            assert!(
                text.chars.iter().all(|c| c.turns == turns),
                "{said} on a page rotated {rotation}: {:?}",
                text.chars.iter().map(|c| c.turns).collect::<Vec<_>>()
            );
            assert_boxes_land_on_the_glyphs(&mut worker, 0, &text, true);
            // However it is turned, it reads as it was written.
            assert_eq!(
                read_back(&text),
                said.replace(' ', ""),
                "page rotated {rotation}"
            );
            // Plain text does not care about direction.
            assert_eq!(worker.plain_text(0, 1).unwrap()[0].trim(), said);
        }
    }
}

#[test]
fn a_page_without_text_has_none() {
    let blank = "0.2 0.4 0.8 rg 72 600 100 100 re f".to_owned();
    let mut worker = open(&pages_pdf(&[blank]));
    let text = worker.text(0).unwrap();
    assert!(text.chars.is_empty());
    assert_eq!(worker.plain_text(0, 1).unwrap(), [String::new()]);
    let all = worker
        .document_text(Duration::from_secs(5), &|| false)
        .unwrap();
    assert_eq!(all.pages, [String::new()]);
}

#[test]
fn a_page_over_the_character_cap_is_cut() {
    // PDF strings stop at 32,767 bytes: the page is many of them.
    let chunk = format!("({}) Tj", "a".repeat(10_000));
    let long = format!(
        "BT /F1 2 Tf 10 700 Td {} ({}) Tj ET",
        vec![chunk; MAX_PAGE_CHARS / 10_000 + 1].join(" "),
        "a".repeat(50)
    );
    let mut worker = open(&pages_pdf(&[long]));
    let text = worker.text(0).expect("a long page still arrives");
    assert_eq!(text.chars.len(), MAX_PAGE_CHARS);
    let plain = worker.plain_text(0, 1).unwrap();
    assert_eq!(plain[0].chars().count(), MAX_PAGE_CHARS);
    let all = worker
        .document_text(Duration::from_secs(60), &|| false)
        .unwrap();
    assert!(all.truncated, "a cut page is said to be cut");
}

#[test]
fn a_missing_page_is_refused_and_the_worker_goes_on() {
    let mut worker = open(&sample_pdf(1));
    match worker.text(4) {
        Err(WorkerError::Refused { kind, .. }) => assert_eq!(kind, ErrorKind::NoSuchPage),
        other => panic!("{other:?}"),
    }
    assert!(worker.failure().is_none());
    assert!(worker.text(0).is_ok());
}

#[test]
fn text_before_open_is_a_protocol_error() {
    let mut worker = Worker::start(&paths()).unwrap();
    match worker.text(0) {
        Err(WorkerError::Refused { kind, .. }) => assert_eq!(kind, ErrorKind::Protocol),
        other => panic!("{other:?}"),
    }
}

#[test]
fn reading_a_document_stops_at_its_budget_and_when_cancelled() {
    let mut worker = open(&sample_pdf(10));
    let started = Instant::now();
    assert_eq!(
        worker.document_text(Duration::ZERO, &|| false).unwrap_err(),
        WorkerError::TimedOut
    );
    assert!(started.elapsed() < Duration::from_secs(4));
    // The worker is ended: every later call says so.
    assert!(worker.failure().is_some());
    assert!(worker.plain_text(0, 1).is_err());

    let mut worker = open(&sample_pdf(10));
    assert_eq!(
        worker
            .document_text(Duration::from_secs(30), &|| true)
            .unwrap_err(),
        WorkerError::Cancelled
    );
    assert!(worker.failure().is_some());
}

#[test]
fn an_encrypted_pdf_has_no_text_to_index_without_its_password() {
    let mut worker = Worker::start(&paths()).unwrap();
    match worker.open(&encrypted_pdf("secret"), None) {
        Err(WorkerError::Refused { kind, .. }) => assert_eq!(kind, ErrorKind::PasswordRequired),
        other => panic!("{other:?}"),
    }
}
