//! A worker that has been taken over (issue #43, step 9). The app trusts
//! nothing the worker says: a stand-in worker sends replies a real one never
//! would, and the client ends it and reports a plain error. No PDFium here.

#![cfg(unix)]

mod stand_in;

use std::path::Path;
use std::time::{Duration, Instant};

use kanso_pdf::client::{Deadlines, Worker, WorkerError, WorkerPaths};
use kanso_pdf::fixtures::sample_pdf;
use kanso_pdf::protocol::{ErrorKind, MAX_PAGES, Response, write_frame};

/// The deadline given to every stand-in worker, open and request alike. It is
/// the bound the client enforces, and the tests below rest on it: the stand-in
/// stalls for 30 s (or floods without end), so a client that did not cut it
/// off itself would be cut off by the watchdog after `DEADLINE` and report
/// [`WorkerError::TimedOut`], which none of the tests accept. Reaching the
/// expected error is the proof of the cut-off, with no wall-clock assertion.
///
/// It is far above what the client needs (milliseconds, once the worker runs)
/// because it must also outlast a loaded machine; it is still under the
/// stall, so the proof holds.
const DEADLINE: Duration = Duration::from_secs(10);

/// How long to wait for a stand-in worker to start running. Only a ceiling for
/// a failure message, not a pass criterion.
const STARTUP_CEILING: Duration = Duration::from_secs(120);

/// A worker that answers every request with `reply` and then waits.
fn start(dir: &Path, reply: &[u8]) -> Worker {
    start_then(dir, reply, "exec sleep 30")
}

/// As [`start`], with the shell command it ends on.
fn start_then(dir: &Path, reply: &[u8], then: &str) -> Worker {
    let canned = dir.join("reply");
    std::fs::write(&canned, reply).unwrap();
    launch(dir, &format!("cat '{}'\n{then}", canned.display()))
}

/// Starts a stand-in worker running the shell `body`, and returns once the
/// script is running.
///
/// The first run of a script that was just written is slow on macOS (the OS
/// checks it before it runs) and the checks queue up when many tests start
/// workers at once: a run of seconds under load, against milliseconds for the
/// behaviour under test. That start-up is not what these tests measure, so
/// the script marks the moment it begins and the client's deadlines (which
/// start when a request is sent) are only reached once the marker is there.
fn launch(dir: &Path, body: &str) -> Worker {
    let ready = dir.join("ready");
    let script = dir.join("worker");
    stand_in::write_script(&script, &format!(": > '{}'\n{body}", ready.display()));
    let worker = Worker::start_with(
        &WorkerPaths {
            exe: script,
            library: None,
        },
        Deadlines {
            open: DEADLINE,
            request: DEADLINE,
        },
    )
    .unwrap();
    let waiting = Instant::now();
    while !ready.exists() {
        assert!(
            waiting.elapsed() < STARTUP_CEILING,
            "the stand-in worker never started"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    worker
}

fn frame(response: &Response, blob: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    write_frame(&mut out, response, blob).unwrap();
    out
}

fn opened(pages: u32, sizes: Vec<[f32; 2]>) -> Response {
    Response::Opened {
        pages,
        sizes,
        encrypted: false,
    }
}

/// The reply is refused as unexpected, quickly, and the worker is ended.
fn assert_not_trusted(reply: &[u8], what: &str) {
    let dir = tempfile::tempdir().unwrap();
    let mut worker = start(dir.path(), reply);
    // `BadReply` and not `TimedOut`: refused on sight, not at the deadline.
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    assert_eq!(error, WorkerError::BadReply, "{what}: {error:?}");
    assert!(worker.failure().is_some(), "{what}: the worker was kept");
    assert_eq!(error.message(), "The PDF reader sent something unexpected.");
}

#[test]
fn a_reply_that_claims_more_than_may_be_sent_is_refused() {
    // A blob length of 4 GiB - 1 in the head, then nothing.
    let mut head = Vec::new();
    head.extend_from_slice(&2u32.to_le_bytes());
    head.extend_from_slice(&u32::MAX.to_le_bytes());
    head.extend_from_slice(b"{}");
    assert_not_trusted(&head, "a 4 GiB blob");
    // And a JSON length past its cap.
    let mut head = Vec::new();
    head.extend_from_slice(&u32::MAX.to_le_bytes());
    head.extend_from_slice(&0u32.to_le_bytes());
    assert_not_trusted(&head, "a 4 GiB JSON");
}

#[test]
fn a_reply_that_is_not_a_message_is_refused() {
    let mut wire = Vec::new();
    let json = br#"{"type":"opened","pages":"many"}"#;
    wire.extend_from_slice(&(json.len() as u32).to_le_bytes());
    wire.extend_from_slice(&0u32.to_le_bytes());
    wire.extend_from_slice(json);
    assert_not_trusted(&wire, "wrong types");
    assert_not_trusted(&[0x7f; 64], "noise");
}

#[test]
fn a_document_that_should_have_been_refused_is_not_shown() {
    assert_not_trusted(
        &frame(
            &opened(MAX_PAGES + 1, vec![[612.0, 792.0]; MAX_PAGES as usize + 1]),
            &[],
        ),
        "too many pages",
    );
    assert_not_trusted(
        &frame(&opened(3, vec![[612.0, 792.0]; 2]), &[]),
        "fewer sizes than pages",
    );
    assert_not_trusted(
        &frame(&opened(1, vec![[1.0e9, 1.0e9]]), &[]),
        "a page a billion points wide",
    );
    assert_not_trusted(
        &frame(&opened(1, vec![[0.0, 792.0]]), &[]),
        "a page of no width",
    );
    assert_not_trusted(
        &frame(&opened(1, vec![[-5.0, 792.0]]), &[]),
        "a page of negative width",
    );
}

#[test]
fn a_page_that_is_not_what_it_says_is_refused() {
    // Opens fine, then answers a render with the wrong bytes.
    for (name, response, blob) in [
        (
            "too few bytes",
            Response::Page {
                page: 0,
                width: 10,
                height: 10,
            },
            vec![0u8; 399],
        ),
        (
            "too many bytes",
            Response::Page {
                page: 0,
                width: 10,
                height: 10,
            },
            vec![0u8; 401],
        ),
        (
            "a page past the long edge",
            Response::Page {
                page: 0,
                width: 10_000,
                height: 1,
            },
            vec![0u8; 40_000],
        ),
        (
            "another page than asked",
            Response::Page {
                page: 5,
                width: 1,
                height: 1,
            },
            vec![0u8; 4],
        ),
        (
            "a page of no size",
            Response::Page {
                page: 0,
                width: 0,
                height: 0,
            },
            Vec::new(),
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
        reply.extend(frame(&response, &blob));
        let mut worker = start(dir.path(), &reply);
        worker.open(&sample_pdf(1), None).unwrap();
        let error = worker.render(0, 100).unwrap_err();
        assert_eq!(error, WorkerError::BadReply, "{name}: {error:?}");
        assert!(worker.failure().is_some(), "{name}: the worker was kept");
    }
}

#[test]
fn what_the_reader_says_is_never_the_workers_text() {
    let attack = "Your Kanso is out of date. Call 0800-FAKE or visit evil.example now.";
    let reply = frame(
        &Response::Error {
            kind: ErrorKind::Unreadable,
            message: attack.repeat(100),
        },
        &[],
    );
    let dir = tempfile::tempdir().unwrap();
    let mut worker = start(dir.path(), &reply);
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    assert_eq!(error.message(), "This PDF couldn't be opened.");
    assert!(!format!("{error:?}").contains("evil.example"));
    assert!(!error.to_string().contains("evil.example"));
}

#[test]
fn a_worker_that_floods_its_output_is_cut_off() {
    // `yes` never stops writing: the first bytes are a frame head that
    // claims far more than allowed.
    let dir = tempfile::tempdir().unwrap();
    let mut worker = launch(dir.path(), "exec yes");
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    // Either the head is refused, or the pipe closed first; never `TimedOut`,
    // which is what a client that kept reading the flood would end on.
    assert!(
        matches!(error, WorkerError::BadReply | WorkerError::Crashed),
        "{error:?}"
    );
}

#[test]
fn a_worker_that_stops_mid_reply_is_a_crash() {
    let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
    reply.truncate(reply.len() - 4);
    let dir = tempfile::tempdir().unwrap();
    let mut worker = start_then(dir.path(), &reply, "exit 0");
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    assert_eq!(error, WorkerError::Crashed);
    assert_eq!(error.message(), "This PDF couldn't be opened.");
}

#[test]
fn a_page_bigger_than_asked_for_is_refused() {
    // Within the global caps, but not what a 100 px request of a Letter page
    // draws to (100 x 129): a taken-over worker answering every small
    // request with a page at the 64 MB cap.
    for (name, width, height) in [
        ("the whole cap", 4096, 4096),
        ("a pixel too wide", 101, 129),
        ("a pixel too tall", 100, 130),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
        reply.extend(frame(
            &Response::Page {
                page: 0,
                width,
                height,
            },
            &vec![0u8; width as usize * height as usize * 4],
        ));
        let mut worker = start(dir.path(), &reply);
        worker.open(&sample_pdf(1), None).unwrap();
        let error = worker.render(0, 100).unwrap_err();
        assert_eq!(error, WorkerError::BadReply, "{name}: {error:?}");
        assert!(worker.failure().is_some(), "{name}: the worker was kept");
    }
    // The size the request asks for is believed.
    let dir = tempfile::tempdir().unwrap();
    let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
    reply.extend(frame(
        &Response::Page {
            page: 0,
            width: 100,
            height: 129,
        },
        &vec![0u8; 100 * 129 * 4],
    ));
    let mut worker = start(dir.path(), &reply);
    worker.open(&sample_pdf(1), None).unwrap();
    let page = worker.render(0, 100).unwrap();
    assert_eq!((page.width, page.height), (100, 129));
}

#[test]
fn a_page_the_document_does_not_have_is_refused() {
    // The worker says page 3 of a one-page document drew.
    let dir = tempfile::tempdir().unwrap();
    let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
    reply.extend(frame(
        &Response::Page {
            page: 3,
            width: 1,
            height: 1,
        },
        &[0u8; 4],
    ));
    let mut worker = start(dir.path(), &reply);
    worker.open(&sample_pdf(1), None).unwrap();
    assert_eq!(worker.render(3, 100).unwrap_err(), WorkerError::BadReply);
}

#[test]
fn only_a_page_carries_bytes() {
    // An open answer, or a refusal, with a blob attached.
    assert_not_trusted(
        &frame(&opened(1, vec![[612.0, 792.0]]), &[7u8; 4096]),
        "an open with bytes",
    );
    assert_not_trusted(
        &frame(
            &Response::Error {
                kind: ErrorKind::Unreadable,
                message: String::new(),
            },
            &[7u8; 4096],
        ),
        "a refusal with bytes",
    );
}

// Text replies are held to the request like pages are (issue #43, steps 5
// and 7): the page asked for, the size the open reported, the count the
// header gave, the caps, and plain content.

use kanso_pdf::model::Quad;
use kanso_pdf::protocol::MAX_PAGE_CHARS;
use kanso_pdf::text::{PageText, TextChar};

fn letter_text(page: u32, chars: &str) -> PageText {
    PageText {
        page,
        width: 612.0,
        height: 792.0,
        rotation: 0,
        chars: chars
            .chars()
            .enumerate()
            .map(|(i, ch)| TextChar {
                ch,
                quad: Quad::new(0.1 + i as f64 * 0.01, 0.1, 0.11 + i as f64 * 0.01, 0.12),
                turns: 0,
                space_before: false,
            })
            .collect(),
    }
}

/// Opens a one-page document, then asks for its text and gets `reply`.
fn ask_text(reply: Vec<u8>) -> Result<PageText, WorkerError> {
    let dir = tempfile::tempdir().unwrap();
    let mut wire = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
    wire.extend(reply);
    let mut worker = start(dir.path(), &wire);
    worker.open(&sample_pdf(1), None).unwrap();
    let result = worker.text(0);
    if result.is_err() {
        assert!(worker.failure().is_some(), "the worker was kept");
    }
    result
}

fn text_frame(page: u32, chars: u32, blob: &[u8]) -> Vec<u8> {
    frame(
        &Response::TextPage {
            page,
            chars,
            truncated: false,
        },
        blob,
    )
}

#[test]
fn a_text_page_that_is_what_was_asked_for_is_believed() {
    let text = letter_text(1, "Hello");
    let json = serde_json::to_vec(&text).unwrap();
    assert_eq!(ask_text(text_frame(0, 5, &json)).unwrap(), text);
}

/// The space mark is a plain flag: it is carried as sent, however a worker
/// that has been taken over sets it, and it adds no character to the count.
#[test]
fn the_space_mark_is_carried_and_changes_no_count() {
    let mut text = letter_text(1, "Hello");
    for c in &mut text.chars {
        c.space_before = true;
    }
    let json = serde_json::to_vec(&text).unwrap();
    assert_eq!(ask_text(text_frame(0, 5, &json)).unwrap(), text);
    // A mark for a space is not a character: a count that includes one is
    // not the count sent.
    assert!(ask_text(text_frame(0, 6, &json)).is_err());
}

#[test]
fn a_text_page_that_is_not_what_was_asked_for_is_refused() {
    let good = letter_text(1, "Hello");
    let json = |text: &PageText| serde_json::to_vec(text).unwrap();
    // The wrong page in the header, or in the text.
    let mut spoiled = vec![("another page answered", text_frame(2, 5, &json(&good)))];
    spoiled.push((
        "the text of another page",
        text_frame(0, 5, &json(&letter_text(4, "Hello"))),
    ));
    // The count in the header is not the count sent.
    spoiled.push(("a count that lies", text_frame(0, 4, &json(&good))));
    spoiled.push((
        "a count past the cap",
        text_frame(0, u32::MAX, &json(&good)),
    ));
    // A page of another size than the one opened.
    let mut wide = good.clone();
    wide.width = 5000.0;
    spoiled.push(("another size", text_frame(0, 5, &json(&wide))));
    let mut turned = good.clone();
    turned.rotation = 45;
    spoiled.push((
        "a rotation that isn't one",
        text_frame(0, 5, &json(&turned)),
    ));
    // Boxes off the page, backwards, or characters that aren't text.
    let mut off = good.clone();
    off.chars[0].quad = Quad([0.5, 0.5, 0.1, 0.1]);
    spoiled.push(("a box that is backwards", text_frame(0, 5, &json(&off))));
    let mut control = good.clone();
    control.chars[0].ch = '\u{1b}';
    spoiled.push(("a control character", text_frame(0, 5, &json(&control))));
    let mut spun = good.clone();
    spun.chars[0].turns = 9;
    spoiled.push(("a direction that isn't one", text_frame(0, 5, &json(&spun))));
    // Not a page's text at all.
    spoiled.push(("not JSON", text_frame(0, 5, b"hello")));
    spoiled.push(("nothing", text_frame(0, 5, b"")));
    // Another kind of answer to the request.
    spoiled.push((
        "a page of pixels",
        frame(
            &Response::Page {
                page: 0,
                width: 1,
                height: 1,
            },
            &[0u8; 4],
        ),
    ));
    spoiled.push(("closed", frame(&Response::Closed, &[])));
    spoiled.push(("text with no blob but a count", text_frame(0, 5, &[])));
    // Text sent to a close, or an open answer with a text blob.
    for (name, reply) in spoiled {
        assert_eq!(
            ask_text(reply).unwrap_err(),
            WorkerError::BadReply,
            "{name}"
        );
    }
}

#[test]
fn a_page_of_text_past_the_cap_is_refused() {
    let big = letter_text(1, &"a".repeat(MAX_PAGE_CHARS + 1));
    let json = serde_json::to_vec(&big).unwrap();
    let error = ask_text(text_frame(0, big.chars.len() as u32, &json)).unwrap_err();
    assert_eq!(error, WorkerError::BadReply);
}

/// Opens a document of `pages` pages and asks for the plain text of pages
/// `from` and on (`count`), getting `reply`.
fn ask_plain(
    pages: u32,
    (from, count): (u32, u32),
    reply: Vec<u8>,
) -> Result<Vec<String>, WorkerError> {
    let dir = tempfile::tempdir().unwrap();
    let mut wire = frame(&opened(pages, vec![[612.0, 792.0]; pages as usize]), &[]);
    wire.extend(reply);
    let mut worker = start(dir.path(), &wire);
    worker.open(&sample_pdf(1), None).unwrap();
    let result = worker.plain_text(from, count);
    // A refusal leaves the worker up; anything else it said ends it.
    if result == Err(WorkerError::BadReply) {
        assert!(worker.failure().is_some(), "the worker was kept");
    }
    result
}

fn plain_frame(from: u32, lens: &[u32], blob: &[u8]) -> Vec<u8> {
    frame(
        &Response::PlainText {
            from,
            lens: lens.to_vec(),
            truncated: false,
        },
        blob,
    )
}

#[test]
fn plain_text_that_is_what_was_asked_for_is_believed() {
    assert_eq!(
        ask_plain(3, (1, 5), plain_frame(1, &[3, 0], b"one")).unwrap(),
        ["one", ""]
    );
    assert_eq!(
        ask_plain(3, (0, 1), plain_frame(0, &[6], "héllo".as_bytes())).unwrap(),
        ["héllo"]
    );
}

#[test]
fn plain_text_that_is_not_what_was_asked_for_is_refused() {
    let big = "a".repeat(MAX_PAGE_CHARS + 1);
    type Case = (&'static str, u32, (u32, u32), Vec<u8>);
    let cases: Vec<Case> = vec![
        ("another start", 3, (1, 2), plain_frame(0, &[1, 1], b"ab")),
        (
            "too many pages",
            3,
            (0, 2),
            plain_frame(0, &[1, 1, 1], b"abc"),
        ),
        ("too few pages", 3, (0, 3), plain_frame(0, &[1, 1], b"ab")),
        (
            "lengths past the bytes",
            3,
            (0, 2),
            plain_frame(0, &[1, 9], b"ab"),
        ),
        (
            "bytes past the lengths",
            3,
            (0, 2),
            plain_frame(0, &[1, 1], b"abc"),
        ),
        (
            "overflowing lengths",
            3,
            (0, 2),
            plain_frame(0, &[u32::MAX, u32::MAX], b""),
        ),
        ("not UTF-8", 3, (0, 1), plain_frame(0, &[2], &[0xff, 0xfe])),
        (
            "a control character",
            3,
            (0, 1),
            plain_frame(0, &[3], b"a\x07b"),
        ),
        ("a NUL", 3, (0, 1), plain_frame(0, &[3], b"a\0b")),
        (
            "a page past the cap",
            3,
            (0, 1),
            plain_frame(0, &[big.len() as u32], big.as_bytes()),
        ),
        (
            "the pixels of a page",
            3,
            (0, 1),
            frame(
                &Response::Page {
                    page: 0,
                    width: 1,
                    height: 1,
                },
                &[0; 4],
            ),
        ),
        ("a page's text", 3, (0, 1), text_frame(0, 0, b"{}")),
    ];
    for (name, pages, ask, reply) in cases {
        assert_eq!(
            ask_plain(pages, ask, reply).unwrap_err(),
            WorkerError::BadReply,
            "{name}"
        );
    }
}

#[test]
fn a_window_past_the_end_is_not_even_asked() {
    // No reply is ever read: the client knows the document has 3 pages.
    let error = ask_plain(3, (3, 1), Vec::new()).unwrap_err();
    assert!(
        matches!(
            error,
            WorkerError::Refused {
                kind: ErrorKind::NoSuchPage,
                ..
            }
        ),
        "{error:?}"
    );
}
