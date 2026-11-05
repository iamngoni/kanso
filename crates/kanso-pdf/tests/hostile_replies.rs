//! A worker that has been taken over (issue #43, step 9). The app trusts
//! nothing the worker says: a stand-in worker sends replies a real one never
//! would, and the client ends it and reports a plain error. No PDFium here.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use kanso_pdf::client::{Deadlines, Worker, WorkerError, WorkerPaths};
use kanso_pdf::fixtures::sample_pdf;
use kanso_pdf::protocol::{ErrorKind, MAX_PAGES, Response, write_frame};

/// A worker that answers every request with `reply` and then waits.
fn start(dir: &Path, reply: &[u8]) -> Worker {
    start_then(dir, reply, "exec sleep 30")
}

/// As [`start`], with the shell command it ends on.
fn start_then(dir: &Path, reply: &[u8], then: &str) -> Worker {
    let canned = dir.join("reply");
    std::fs::write(&canned, reply).unwrap();
    let script = dir.join("worker");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ncat '{}'\n{then}\n", canned.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    Worker::start_with(
        &WorkerPaths {
            exe: script,
            library: None,
        },
        Deadlines {
            open: Duration::from_secs(5),
            request: Duration::from_secs(5),
        },
    )
    .unwrap()
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
    let started = Instant::now();
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    assert_eq!(error, WorkerError::BadReply, "{what}: {error:?}");
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "{what} took long"
    );
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
    let script = dir.path().join("flood");
    std::fs::write(&script, "#!/bin/sh\nexec yes\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut worker = Worker::start_with(
        &WorkerPaths {
            exe: script,
            library: None,
        },
        Deadlines {
            open: Duration::from_secs(5),
            request: Duration::from_secs(5),
        },
    )
    .unwrap();
    let started = Instant::now();
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    // Either the head is refused, or the pipe closed first.
    assert!(
        matches!(error, WorkerError::BadReply | WorkerError::Crashed),
        "{error:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[test]
fn a_worker_that_stops_mid_reply_is_a_crash() {
    let mut reply = frame(&opened(1, vec![[612.0, 792.0]]), &[]);
    reply.truncate(reply.len() - 4);
    let dir = tempfile::tempdir().unwrap();
    let mut worker = start_then(dir.path(), &reply, "exit 0");
    let started = Instant::now();
    let error = worker.open(&sample_pdf(1), None).unwrap_err();
    assert_eq!(error, WorkerError::Crashed);
    assert!(started.elapsed() < Duration::from_secs(4));
    assert_eq!(error.message(), "This PDF couldn't be opened.");
}
