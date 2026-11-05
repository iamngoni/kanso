//! Hostile PDFs through the real worker (issue #43, step 9): every file of
//! the corpus (`kanso_pdf::hostile`) is opened and drawn by the contained
//! `kanso-pdf-worker`, and every one ends in a clean error or a capped
//! render. None crashes the worker, none outlasts its deadline, and nothing
//! a document's actions name is ever reached.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(all(feature = "render", any(target_os = "macos", target_os = "linux")))]

mod common;

use std::fmt;
use std::net::TcpListener;
use std::time::{Duration, Instant};

use common::paths;
use kanso_pdf::client::{Deadlines, Worker, WorkerError};
use kanso_pdf::fixtures::{annotated_pdf, report_pdf};
use kanso_pdf::hostile::{Canary, Expect, Sample, corpus};
use kanso_pdf::protocol::{ErrorKind, MAX_LONG_EDGE_PX, MAX_PAGE_BYTES, MAX_PAGES, page_is_sound};
use proptest::prelude::*;

/// How long opening a file may take, whatever it is. Generous: the point of
/// the deadlines is that they end, not that a slow machine fails.
const OPEN: Duration = Duration::from_secs(30);

/// What happened to one sample.
#[derive(Debug, PartialEq)]
enum Outcome {
    /// It opened with this many pages; these pages were drawn, and
    /// `refused` further draws were turned down cleanly.
    Opened {
        pages: u32,
        drawn: usize,
        refused: usize,
    },
    /// The open was refused.
    Refused(ErrorKind),
    /// A deadline passed and the worker was killed.
    TimedOut,
    /// The worker died by itself.
    Crashed,
    /// Anything else: the worker said something it must not.
    Misbehaved(String),
}

impl Outcome {
    fn of(error: WorkerError) -> Outcome {
        match error {
            WorkerError::Refused { kind, .. } => Outcome::Refused(kind),
            WorkerError::TimedOut => Outcome::TimedOut,
            WorkerError::Crashed => Outcome::Crashed,
            other => Outcome::Misbehaved(format!("{other:?}")),
        }
    }
}

/// Opens `bytes` in a fresh worker and draws a spread of its pages at a
/// spread of sizes, checking every page that comes back against the caps.
fn run(bytes: &[u8], deadlines: Deadlines) -> Outcome {
    let mut worker = Worker::start_with(&paths(), deadlines).expect("the worker starts");
    let opened = match worker.open(bytes, None) {
        Ok(opened) => opened,
        Err(error) => return Outcome::of(error),
    };
    assert!(
        opened.pages <= MAX_PAGES,
        "a document over the page cap opened"
    );
    let last = opened.pages.saturating_sub(1);
    let mut asks = vec![(0, 200), (0, 612), (0, u32::MAX), (last, 612)];
    asks.push((opened.pages / 2, 306));
    // A page that is not there is a clean refusal, not a crash.
    asks.push((opened.pages, 200));
    let (mut drawn, mut refused) = (0, 0);
    for (page, width) in asks {
        match worker.render(page, width) {
            Ok(rendered) => {
                assert!(
                    page_is_sound(rendered.width, rendered.height, rendered.bgra.len()),
                    "page {page} came back unsound"
                );
                assert!(rendered.width <= MAX_LONG_EDGE_PX && rendered.height <= MAX_LONG_EDGE_PX);
                assert!(rendered.bgra.len() <= MAX_PAGE_BYTES);
                drawn += 1;
            }
            Err(WorkerError::Refused { .. }) => refused += 1,
            Err(other) => return Outcome::of(other),
        }
    }
    // The text of a hostile file is asked for too: a page's characters
    // (the client checks each reply against the request and the caps), a
    // page that isn't there, and the whole document's words.
    for page in [0, last, opened.pages] {
        match worker.text(page) {
            Ok(_) => {}
            Err(WorkerError::Refused { .. }) => refused += 1,
            Err(other) => return Outcome::of(other),
        }
    }
    match worker.document_text(Duration::from_secs(60), &|| false) {
        Ok(text) => assert_eq!(text.pages.len(), opened.pages as usize),
        Err(WorkerError::Refused { .. }) => refused += 1,
        Err(other) => return Outcome::of(other),
    }
    worker.close();
    Outcome::Opened {
        pages: opened.pages,
        drawn,
        refused,
    }
}

/// Whether the outcome is one the sample allows.
fn allowed(expect: Expect, outcome: &Outcome) -> bool {
    match (expect, outcome) {
        (Expect::Opens, Outcome::Opened { drawn, .. }) => *drawn > 0,
        (Expect::Refused(kinds), Outcome::Refused(kind)) => kinds.contains(kind),
        // A file that may or may not open may also be too slow: the deadline
        // is the cap on time, and a killed worker is a clean end.
        (Expect::OpensOrRefused | Expect::TimesOut, outcome) => {
            !matches!(outcome, Outcome::Crashed | Outcome::Misbehaved(_))
        }
        _ => false,
    }
}

struct Report<'a>(&'a [(String, Expect, Outcome, Duration)]);

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, expect, outcome, took) in self.0 {
            writeln!(
                f,
                "{name:28} {took:>8.2?}  expected {expect:?}, got {outcome:?}"
            )?;
        }
        Ok(())
    }
}

fn run_sample(sample: &Sample, deadlines: Deadlines) -> (Outcome, Duration) {
    let started = Instant::now();
    let outcome = run(&sample.bytes, deadlines);
    (outcome, started.elapsed())
}

/// The whole corpus, one worker per file. The actions in it point at a
/// server and a file that this test watches.
#[test]
fn every_hostile_file_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let canary = Canary {
        url: format!("http://{}/kanso-canary", listener.local_addr().unwrap()),
        marker: dir.path().join("launched"),
    };
    let samples = corpus(&canary);
    assert!(
        samples.len() >= 40,
        "the corpus shrank to {}",
        samples.len()
    );

    let deadlines = Deadlines {
        open: OPEN,
        request: Duration::from_secs(5),
    };
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for sample in &samples {
        let (outcome, took) = run_sample(sample, deadlines);
        // Nothing may run past its deadline plus the time to notice.
        let slack = OPEN + Duration::from_secs(10);
        let fine = allowed(sample.expect, &outcome) && took < slack;
        report.push((sample.name.to_owned(), sample.expect, outcome, took));
        if !fine {
            failures.push(sample.name);
        }
    }
    if std::env::var_os("KANSO_PDF_CORPUS_REPORT").is_some() {
        println!("{}", Report(&report));
    }
    assert!(
        failures.is_empty(),
        "{failures:?} did not fail closed:\n{}",
        Report(&report)
    );

    // No document's action reached the network or started a program.
    assert!(
        listener.accept().is_err(),
        "something connected to the canary server"
    );
    assert!(!canary.marker.exists(), "something created the canary file");
}

#[test]
fn the_corpus_covers_what_the_issue_names() {
    let names: Vec<&str> = corpus(&Canary::unreachable())
        .iter()
        .map(|s| s.name)
        .collect();
    for needed in [
        "truncated-half",
        "xref-prev-self-loop",
        "xref-prev-cycle",
        "mediabox-1e9",
        "nesting-arrays",
        "javascript-actions",
        "embedded-file",
        "launch-action",
        "uri-action",
        "pages-over-limit",
        "empty",
        "junk-random",
    ] {
        assert!(names.contains(&needed), "no {needed} in the corpus");
    }
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), names.len(), "two samples share a name");
}

fn sample(name: &str) -> Sample {
    corpus(&Canary::unreachable())
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no {name} in the corpus"))
}

// Page, pixel and time caps ---------------------------------------------------

#[test]
fn a_document_over_the_page_limit_is_refused_cleanly() {
    for name in ["pages-over-limit", "pages-4000"] {
        let mut worker = Worker::start(&paths()).unwrap();
        let started = Instant::now();
        let error = worker.open(&sample(name).bytes, None).unwrap_err();
        match &error {
            WorkerError::Refused { kind, .. } => assert_eq!(*kind, ErrorKind::TooManyPages),
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(error.message(), "This PDF has more than 2,000 pages.");
        assert_eq!(MAX_PAGES, 2_000);
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "{name} took long to refuse"
        );
        // The refusal ends the worker; later calls say so without hanging.
        assert!(worker.failure().is_some());
        assert!(worker.render(0, 200).is_err());
    }
}

#[test]
fn a_document_at_the_page_limit_is_shown() {
    let mut worker = Worker::start(&paths()).unwrap();
    let opened = worker.open(&sample("pages-at-limit").bytes, None).unwrap();
    assert_eq!(opened.pages, MAX_PAGES);
    assert!(worker.render(MAX_PAGES - 1, 200).is_ok());
    assert!(worker.render(MAX_PAGES, 200).is_err());
}

#[test]
fn a_page_of_absurd_declared_size_is_refused_or_capped() {
    for name in [
        "mediabox-1e9",
        "mediabox-sliver",
        "mediabox-float-max",
        "user-unit-huge",
    ] {
        match run(&sample(name).bytes, Deadlines::default()) {
            Outcome::Refused(kind) => assert!(
                matches!(kind, ErrorKind::PageTooLarge | ErrorKind::Unreadable),
                "{name}: {kind:?}"
            ),
            // Or PDFium itself brought the size within bounds, and every
            // page drawn was within the caps (checked by `run`).
            Outcome::Opened { drawn, .. } => assert!(drawn > 0, "{name} opened and drew nothing"),
            other => panic!("{name}: {other:?}"),
        }
    }
}

/// The time cap, with the real worker: a file that makes PDFium draw for
/// longer than the request deadline gets the worker killed, the reader gets
/// "couldn't be opened", the process is gone, and the next call does not
/// wait.
#[test]
fn a_page_that_takes_too_long_gets_the_worker_killed() {
    let deadlines = Deadlines {
        open: OPEN,
        request: Duration::from_millis(1500),
    };
    let mut worker = Worker::start_with(&paths(), deadlines).unwrap();
    let bomb = sample("form-bomb").bytes;
    let opened = match worker.open(&bomb, None) {
        Ok(opened) => opened,
        Err(error) => panic!("the form bomb should open, then stall on drawing: {error:?}"),
    };
    assert_eq!(opened.pages, 1);
    let pid = worker.pid().expect("a running worker has a process id");

    let started = Instant::now();
    let error = worker.render(0, 612).unwrap_err();
    let took = started.elapsed();
    assert_eq!(error, WorkerError::TimedOut, "drawing a bomb took {took:?}");
    assert_eq!(error.message(), "This PDF couldn't be opened.");
    assert!(
        took < Duration::from_secs(8),
        "the kill came after {took:?}"
    );

    // Ended for good, and quickly.
    let again = Instant::now();
    assert_eq!(worker.render(0, 612).unwrap_err(), WorkerError::TimedOut);
    assert!(again.elapsed() < Duration::from_millis(500));

    // The process itself is gone, not just ignored.
    drop(worker);
    let gone = (0..50).any(|_| {
        std::thread::sleep(Duration::from_millis(100));
        // Signal 0 only asks whether the process still exists.
        unsafe { libc::kill(pid as i32, 0) != 0 }
    });
    assert!(gone, "worker {pid} is still running after its deadline");
}

#[test]
fn a_file_over_the_size_limit_is_refused_without_sending_it() {
    let mut worker = Worker::start(&paths()).unwrap();
    let huge = vec![b'%'; kanso_pdf::protocol::MAX_BLOB_BYTES + 1];
    match worker.open(&huge, None).unwrap_err() {
        WorkerError::Refused { kind, .. } => assert_eq!(kind, ErrorKind::FileTooLarge),
        other => panic!("{other:?}"),
    }
}

// Render flags -----------------------------------------------------------------

/// Annotations and form widgets are never drawn: a page with a red square
/// annotation and a green text field draws as the plain white page it is.
#[test]
fn annotations_and_form_fields_are_not_drawn() {
    let mut worker = Worker::start(&paths()).unwrap();
    worker.open(&annotated_pdf(), None).unwrap();
    let page = worker.render(0, 612).unwrap();
    let coloured = page
        .bgra
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0].abs_diff(p[2]) > 60 || p[1].abs_diff(p[2]) > 60)
        .count();
    assert_eq!(coloured, 0, "{coloured} pixels of an annotation were drawn");
    // The body text is there, so the page was drawn at all.
    assert!(
        page.bgra
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] < 64 && p[1] < 64 && p[2] < 64)
    );
}

// Mutation -----------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    /// A real document with bytes flipped, cut or repeated: whatever comes
    /// of it, the worker neither crashes nor hangs, and every page it
    /// draws is within the caps.
    #[test]
    fn a_damaged_document_fails_closed(
        flips in proptest::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 0..24),
        cut in proptest::option::of(any::<prop::sample::Index>()),
        repeat in proptest::option::of((any::<prop::sample::Index>(), 1usize..64)),
    ) {
        let mut bytes = report_pdf();
        for (at, value) in &flips {
            let i = at.index(bytes.len());
            bytes[i] = *value;
        }
        if let Some((at, len)) = repeat {
            let start = at.index(bytes.len());
            let end = (start + len).min(bytes.len());
            let piece = bytes[start..end].to_vec();
            bytes.splice(start..start, piece);
        }
        if let Some(at) = cut {
            let keep = at.index(bytes.len() + 1);
            bytes.truncate(keep);
        }
        let outcome = run(&bytes, Deadlines { open: OPEN, request: Duration::from_secs(5) });
        prop_assert!(
            !matches!(outcome, Outcome::Crashed | Outcome::Misbehaved(_)),
            "{outcome:?}"
        );
    }
}
