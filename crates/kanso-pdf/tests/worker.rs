//! The worker, started as a real process (issue #43, step 3): pages render,
//! failures are survivable, and the contained worker reaches nothing.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(all(feature = "render", any(target_os = "macos", target_os = "linux")))]

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use kanso_pdf::client::{Deadlines, Worker, WorkerError, WorkerPaths};
use kanso_pdf::fixtures::{REPORT_BARS, encrypted_pdf, report_pdf, sample_pdf as pdf};
use kanso_pdf::protocol::{ErrorKind, MAX_LONG_EDGE_PX, MAX_PAGE_BYTES, MAX_PAGES};

mod common;

use common::{WORKER, library, paths};

fn opened(pages: usize) -> Worker {
    let mut worker = Worker::start(&paths()).expect("the worker starts");
    let doc = worker.open(&pdf(pages), None).expect("the PDF opens");
    assert_eq!(doc.pages as usize, pages);
    worker
}

#[test]
fn a_page_renders_as_bgra_pixels_with_text_on_it() {
    let mut worker = opened(2);
    let page = worker.render(0, 306).expect("page 1 renders");
    assert_eq!((page.width, page.height), (306, 396));
    assert_eq!(page.bgra.len(), 306 * 396 * 4);
    let pixels: Vec<&[u8; 4]> = page.bgra.as_chunks::<4>().0.iter().collect();
    // The corner is the page's white, opaque.
    assert_eq!(*pixels[0], [255, 255, 255, 255]);
    // The text is black on it somewhere.
    assert!(
        pixels.iter().any(|p| p[0] < 64 && p[1] < 64 && p[2] < 64),
        "no text was drawn"
    );
    // Page 2 draws too and differs from page 1 (a different number).
    let second = worker.render(1, 306).expect("page 2 renders");
    assert_ne!(second.bgra, page.bgra);
    worker.close();
}

/// The pixels are BGRA: a blue bar has its blue byte first, so the page can
/// go into a texture without swapping channels.
#[test]
fn colours_keep_their_channels() {
    let mut worker = Worker::start(&paths()).unwrap();
    worker.open(&report_pdf(), None).unwrap();
    // Half size: a point is half a pixel.
    let page = worker.render(0, 306).unwrap();
    for ([x, y], [r, g, b]) in REPORT_BARS {
        let at = ((y / 2.0) as usize * page.width as usize + (x / 2.0) as usize) * 4;
        let pixel = &page.bgra[at..at + 4];
        let near = |got: u8, want: u8| got.abs_diff(want) <= 8;
        assert!(
            near(pixel[0], b) && near(pixel[1], g) && near(pixel[2], r) && pixel[3] == 255,
            "the bar at {x},{y} is {pixel:?} (BGRA), not {r},{g},{b}"
        );
    }
}

#[test]
fn opening_reports_each_pages_size_in_points() {
    let mut worker = Worker::start(&paths()).unwrap();
    let doc = worker.open(&pdf(3), None).unwrap();
    assert_eq!(doc.sizes, vec![[612.0, 792.0]; 3]);
    assert!(!doc.encrypted);
}

#[test]
fn a_request_for_a_huge_page_is_cut_to_the_caps() {
    let mut worker = opened(1);
    let page = worker.render(0, u32::MAX).expect("renders, capped");
    assert!(page.height <= MAX_LONG_EDGE_PX && page.width <= MAX_LONG_EDGE_PX);
    assert!(page.bgra.len() <= MAX_PAGE_BYTES);
    assert_eq!(
        page.bgra.len(),
        page.width as usize * page.height as usize * 4
    );
}

#[test]
fn a_missing_page_is_an_error_and_the_worker_goes_on() {
    let mut worker = opened(1);
    match worker.render(7, 200) {
        Err(WorkerError::Refused { kind, .. }) => assert_eq!(kind, ErrorKind::NoSuchPage),
        other => panic!("{other:?}"),
    }
    assert!(worker.failure().is_none());
    assert!(worker.render(0, 200).is_ok());
}

#[test]
fn something_that_is_not_a_pdf_is_refused_plainly() {
    let mut worker = Worker::start(&paths()).unwrap();
    let error = worker
        .open(b"%PDF-1.7 and then nothing useful", None)
        .unwrap_err();
    match &error {
        WorkerError::Refused { kind, .. } => assert_eq!(*kind, ErrorKind::Unreadable),
        other => panic!("{other:?}"),
    }
    assert_eq!(error.message(), "This PDF couldn't be opened.");
    // A worker that failed to open is finished.
    assert!(worker.failure().is_some());
}

#[test]
fn a_document_over_the_page_limit_is_refused() {
    let mut worker = Worker::start(&paths()).unwrap();
    let error = worker.open(&pdf(MAX_PAGES as usize + 1), None).unwrap_err();
    match error {
        WorkerError::Refused { kind, .. } => assert_eq!(kind, ErrorKind::TooManyPages),
        other => panic!("{other:?}"),
    }
}

#[test]
fn killing_the_worker_is_an_error_for_the_reader_not_a_crash() {
    let mut worker = opened(1);
    worker.render(0, 200).unwrap();
    worker.kill();
    let error = worker.render(0, 200).unwrap_err();
    assert_eq!(error, WorkerError::Crashed);
    assert_eq!(error.message(), "This PDF couldn't be opened.");
    // And it stays that way; nothing panics, nothing hangs.
    assert_eq!(worker.render(0, 200).unwrap_err(), WorkerError::Crashed);
}

#[cfg(unix)]
#[test]
fn a_worker_that_dies_on_its_own_is_the_same_error() {
    // The kill comes from outside, as the system's out-of-memory killer's
    // would: the client sees the pipe close.
    let mut worker = opened(1);
    let pid = worker_pid(&worker);
    let status = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    let error = worker.render(0, 200).unwrap_err();
    assert_eq!(error, WorkerError::Crashed);
}

#[cfg(unix)]
/// A square page at the long-edge cap is 64 MB of pixels, more than a PDF
/// may be: the app still takes it, rather than calling the worker hostile.
#[test]
fn a_page_at_the_full_pixel_cap_arrives() {
    // The same length, so the cross-reference table stays right.
    let mut square = pdf(1);
    let at = square
        .windows(7)
        .position(|w| w == b"612 792")
        .expect("the page box");
    square[at..at + 7].copy_from_slice(b"600 600");
    let mut worker = Worker::start(&paths()).unwrap();
    worker.open(&square, None).unwrap();
    let page = worker
        .render(0, u32::MAX)
        .expect("a full-size page renders");
    assert_eq!(
        (page.width, page.height),
        (MAX_LONG_EDGE_PX, MAX_LONG_EDGE_PX)
    );
    assert_eq!(page.bgra.len(), MAX_PAGE_BYTES);
    assert!(worker.failure().is_none());
}

fn worker_pid(worker: &Worker) -> u32 {
    worker.pid().expect("a running worker has a process id")
}

#[cfg(unix)]
#[test]
fn a_worker_that_hangs_is_killed_at_the_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("hang");
    std::fs::write(&script, "#!/bin/sh\nexec sleep 30\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let fake = WorkerPaths {
        exe: script,
        library: None,
    };
    let mut worker = Worker::start_with(
        &fake,
        Deadlines {
            open: Duration::from_millis(300),
            request: Duration::from_millis(300),
        },
    )
    .unwrap();
    let started = Instant::now();
    let error = worker.open(&pdf(1), None).unwrap_err();
    assert_eq!(error, WorkerError::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(error.message(), "This PDF couldn't be opened.");
}

#[test]
fn a_missing_worker_is_unavailable_not_a_panic() {
    let missing = WorkerPaths {
        exe: "/nonexistent/kanso-pdf-worker".into(),
        library: None,
    };
    assert!(matches!(
        Worker::start(&missing),
        Err(WorkerError::Unavailable(_))
    ));
}

// Containment ---------------------------------------------------------------

/// Runs the worker's containment probe: it contains itself exactly as for a
/// document, then reports what it can still reach.
fn probe(file: &Path, addr: &str, env: &[(&str, &str)]) -> (std::process::ExitStatus, String) {
    let mut command = Command::new(WORKER);
    command
        .env_clear()
        .envs(env.iter().copied())
        .arg("--library")
        .arg(library())
        .arg("--containment-probe")
        .arg(file)
        .arg(addr)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    (child.wait().unwrap(), out)
}

fn reached(report: &str, name: &str) -> String {
    report
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .unwrap_or_else(|| panic!("no {name} in {report:?}"))
        .to_owned()
}

#[test]
fn the_contained_worker_cannot_read_the_home_folder_or_open_a_socket() {
    // A file in the user's home, and a server the machine can reach: the
    // test itself reaches both, so a "denied" below is the sandbox's doing.
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME is set"));
    let dir = tempfile::tempdir_in(&home).unwrap();
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "private").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    assert_eq!(std::fs::read_to_string(&secret).unwrap(), "private");
    assert!(std::net::TcpStream::connect(&addr).is_ok());
    listener.set_nonblocking(true).unwrap();

    // A descriptor the app left inheritable, open on the secret: the
    // sandbox would not stop reads through it, so the worker closes it.
    let leaked = std::fs::File::open(&secret).unwrap();
    {
        use std::os::fd::AsRawFd;
        // SAFETY: clears close-on-exec on a descriptor this test owns.
        assert_eq!(
            unsafe { libc::fcntl(leaked.as_raw_fd(), libc::F_SETFD, 0) },
            0
        );
    }

    let (status, report) = probe(&secret, &addr, &[]);
    drop(leaked);
    assert!(status.success(), "{report}");
    assert_eq!(
        reached(&report, "inherited_fds"),
        "denied",
        "kept the app's files"
    );
    assert_eq!(
        reached(&report, "list_processes"),
        "denied",
        "listed the other programs running"
    );
    assert_eq!(reached(&report, "contained"), "yes");
    assert_eq!(reached(&report, "read_file"), "denied", "read a home file");
    assert_eq!(reached(&report, "list_dir"), "denied", "listed the home");
    assert_eq!(reached(&report, "connect"), "denied", "opened a socket");
    assert_eq!(reached(&report, "spawn"), "denied", "started a program");
    // It is not just broken: the library it needs is still readable.
    assert_eq!(reached(&report, "read_library"), "allowed");
    // Only the test's own control connection ever reached the server.
    assert!(listener.accept().is_ok(), "the control connection");
    assert!(listener.accept().is_err(), "the worker connected");
}

#[test]
fn the_worker_refuses_to_start_without_containment() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("x");
    std::fs::write(&file, "x").unwrap();
    let (status, report) = probe(
        &file,
        "127.0.0.1:1",
        &[(kanso_pdf::contain::FAIL_FOR_TEST, "1")],
    );
    assert_eq!(status.code(), Some(70));
    assert!(report.is_empty(), "it ran uncontained: {report}");

    // The same through the protocol: no PDF is ever opened.
    let mut command = Command::new(WORKER);
    command
        .env_clear()
        .env(kanso_pdf::contain::FAIL_FOR_TEST, "1")
        .arg("--library")
        .arg(library())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut out).unwrap();
    stdin.write_all(b"ignored").ok();
    assert_eq!(child.wait().unwrap().code(), Some(70));
    assert!(out.is_empty());
}

#[test]
fn the_worker_refuses_to_start_without_its_library() {
    let status = Command::new(WORKER)
        .env_clear()
        .arg("--library")
        .arg("/nonexistent/libpdfium")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(70));
}

#[test]
fn an_encrypted_pdf_asks_for_its_password_and_opens_with_it() {
    let bytes = encrypted_pdf("secret");

    let mut asked = Worker::start(&paths()).unwrap();
    match asked.open(&bytes, None).unwrap_err() {
        WorkerError::Refused { kind, .. } => assert_eq!(kind, ErrorKind::PasswordRequired),
        other => panic!("{other:?}"),
    }

    let mut wrong = Worker::start(&paths()).unwrap();
    match wrong.open(&bytes, Some("guess")).unwrap_err() {
        WorkerError::Refused { kind, .. } => assert_eq!(kind, ErrorKind::WrongPassword),
        other => panic!("{other:?}"),
    }

    let mut right = Worker::start(&paths()).unwrap();
    let doc = right.open(&bytes, Some("secret")).unwrap();
    assert!(doc.encrypted);
    let page = right.render(0, 200).unwrap();
    assert!(
        page.bgra
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] < 64 && p[1] < 64 && p[2] < 64),
        "the decrypted text was not drawn"
    );
}
