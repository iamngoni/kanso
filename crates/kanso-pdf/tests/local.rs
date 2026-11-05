//! The phones' reader (issue #43): the worker's loop on a thread of the app,
//! with no process to put a boundary around it. It must give the same pages
//! and the same text layer as the process the desktop starts, keep every cap,
//! and end a hostile file in a clean error.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(all(feature = "render", any(target_os = "macos", target_os = "linux")))]

mod common;

use std::time::Duration;

use common::paths;
use kanso_pdf::client::{Deadlines, Worker, WorkerError};
use kanso_pdf::fixtures::{encrypted_pdf, pages_pdf_on, report_pdf, sample_pdf};
use kanso_pdf::hostile::{Canary, Expect, corpus};
use kanso_pdf::protocol::{ErrorKind, MAX_PAGES};

fn local() -> Worker {
    Worker::start_local(Deadlines::default()).expect("PDFium loads in the app")
}

fn process() -> Worker {
    Worker::start(&paths()).expect("the worker starts")
}

#[test]
fn pdfium_is_available_here() {
    kanso_pdf::local::availability().expect("PDFium loads");
}

#[test]
fn pages_and_text_are_the_desktops_bytes() {
    let rotated = pages_pdf_on(
        &["BT /F1 24 Tf 72 700 Td (Rotated page) Tj ET".to_owned()],
        "[0 0 612 792]",
        90,
    );
    let documents = [sample_pdf(3), report_pdf(), rotated];
    for bytes in documents {
        let mut app = local();
        let mut helper = process();
        let a = app.open(&bytes, None).expect("opens in the app");
        let b = helper.open(&bytes, None).expect("opens in the process");
        assert_eq!(a, b);
        for page in 0..a.pages.min(3) {
            for width in [200, 612, 1_500] {
                let x = app.render(page, width).unwrap();
                let y = helper.render(page, width).unwrap();
                assert_eq!(x, y, "page {page} at {width} differs");
                let x = app.render_as(page, width, true).unwrap();
                let y = helper.render_as(page, width, true).unwrap();
                assert_eq!(x, y, "inverted page {page} at {width} differs");
            }
            assert_eq!(app.text(page).unwrap(), helper.text(page).unwrap());
        }
        assert_eq!(
            app.plain_text(0, 64).unwrap(),
            helper.plain_text(0, 64).unwrap()
        );
        app.close();
        helper.close();
    }
}

#[test]
fn a_password_is_asked_for_and_used_once() {
    let bytes = encrypted_pdf("secret");
    let mut worker = local();
    let error = worker.open(&bytes, None).unwrap_err();
    assert!(matches!(
        error,
        WorkerError::Refused {
            kind: ErrorKind::PasswordRequired,
            ..
        }
    ));
    let mut worker = local();
    assert!(matches!(
        worker.open(&bytes, Some("wrong")).unwrap_err(),
        WorkerError::Refused {
            kind: ErrorKind::WrongPassword,
            ..
        }
    ));
    let mut worker = local();
    let opened = worker.open(&bytes, Some("secret")).unwrap();
    assert!(opened.encrypted);
    assert!(worker.render(0, 300).is_ok());
}

#[test]
fn readers_come_and_go_in_one_process() {
    // The library binds once for the process; a reader after a reader works.
    for _ in 0..5 {
        let mut worker = local();
        worker.open(&sample_pdf(2), None).unwrap();
        assert!(worker.render(1, 300).is_ok());
        worker.close();
    }
    // A reader given up on frees the next.
    let mut worker = local();
    worker.open(&sample_pdf(2), None).unwrap();
    worker.kill();
    assert!(worker.render(0, 300).is_err());
    drop(worker);
    let mut next = local();
    next.open(&sample_pdf(2), None).unwrap();
    assert!(next.render(0, 300).is_ok());
}

#[test]
fn the_page_cap_holds_in_the_app() {
    for name in ["pages-over-limit", "pages-4000"] {
        let sample = corpus(&Canary::unreachable())
            .into_iter()
            .find(|s| s.name == name)
            .unwrap();
        let mut worker = local();
        match worker.open(&sample.bytes, None).unwrap_err() {
            WorkerError::Refused { kind, .. } => assert_eq!(kind, ErrorKind::TooManyPages),
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(MAX_PAGES, 2_000);
    }
}

/// Every file of the corpus that doesn't need a kill to end: a hang can't be
/// killed in the app (see `kanso_pdf::local`), so the files made to hang are
/// the worker process's to test, not this loop's.
#[test]
fn the_hostile_corpus_fails_closed_in_the_app() {
    let deadlines = Deadlines {
        open: Duration::from_secs(30),
        request: Duration::from_secs(10),
    };
    let samples = corpus(&Canary::unreachable());
    let mut checked = 0;
    for sample in samples
        .iter()
        .filter(|s| !matches!(s.expect, Expect::TimesOut))
    {
        let mut worker = Worker::start_local(deadlines).unwrap();
        match worker.open(&sample.bytes, None) {
            Ok(opened) => {
                let last = opened.pages.saturating_sub(1);
                for (page, width) in [(0, 200), (0, u32::MAX), (last, 612), (opened.pages, 200)] {
                    match worker.render(page, width) {
                        Ok(_) | Err(WorkerError::Refused { .. }) => {}
                        Err(other) => panic!("{}: render {other:?}", sample.name),
                    }
                }
                for page in [0, last, opened.pages] {
                    match worker.text(page) {
                        Ok(_) | Err(WorkerError::Refused { .. }) => {}
                        Err(other) => panic!("{}: text {other:?}", sample.name),
                    }
                }
            }
            Err(WorkerError::Refused { .. }) => {}
            Err(other) => panic!("{}: open {other:?}", sample.name),
        }
        checked += 1;
    }
    assert!(checked >= 30, "the corpus shrank to {checked}");
}
