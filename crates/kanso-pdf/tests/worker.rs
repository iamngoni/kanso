//! The worker, started as a real process (issue #43, step 3): pages render,
//! failures are survivable, and the contained worker reaches nothing.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(feature = "render")]

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use kanso_pdf::client::{Deadlines, Worker, WorkerError, WorkerPaths};
use kanso_pdf::protocol::{ErrorKind, MAX_LONG_EDGE_PX, MAX_PAGE_BYTES, MAX_PAGES};

const WORKER: &str = env!("CARGO_BIN_EXE_kanso-pdf-worker");

fn library() -> PathBuf {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "mac-arm64",
        ("macos", _) => "mac-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", _) => "linux-x64",
        ("windows", "aarch64") => "win-arm64",
        _ => "win-x64",
    };
    let (sub, name) = if cfg!(target_os = "macos") {
        ("lib", "libpdfium.dylib")
    } else if cfg!(windows) {
        ("bin", "pdfium.dll")
    } else {
        ("lib", "libpdfium.so")
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform)
        .join(sub)
        .join(name);
    assert!(
        path.is_file(),
        "PDFium is missing: run scripts/fetch-pdfium.sh ({path:?})"
    );
    path.canonicalize().unwrap()
}

fn paths() -> WorkerPaths {
    WorkerPaths {
        exe: WORKER.into(),
        library: Some(library()),
    }
}

/// A PDF of `pages` Letter pages; each says "Kanso page N" in 36 pt Helvetica.
fn pdf(pages: usize) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();
    // 1 catalog, 2 pages, 3 font, then a page and a content stream each.
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".into());
    let kids: Vec<String> = (0..pages).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {pages} >>",
        kids.join(" ")
    ));
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into());
    for i in 0..pages {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        let content = format!("BT /F1 36 Tf 72 700 Td (Kanso page {}) Tj ET", i + 1);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// A one-page PDF encrypted with the standard handler (RC4, 40-bit, revision
/// 2) under this user password, built by hand from the algorithms of the PDF
/// specification (7.6.3) so the password paths are tested for real.
fn encrypted_pdf(user_password: &str) -> Vec<u8> {
    use md5::{Digest, Md5};
    const PAD: [u8; 32] = [
        0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01,
        0x08, 0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53,
        0x69, 0x7A,
    ];
    fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut s: Vec<u8> = (0..=255).collect();
        let mut j = 0usize;
        for i in 0..256 {
            j = (j + s[i] as usize + key[i % key.len()] as usize) % 256;
            s.swap(i, j);
        }
        let (mut i, mut j) = (0usize, 0usize);
        data.iter()
            .map(|byte| {
                i = (i + 1) % 256;
                j = (j + s[i] as usize) % 256;
                s.swap(i, j);
                byte ^ s[(s[i] as usize + s[j] as usize) % 256]
            })
            .collect()
    }
    let owner = [0x5Au8; 32];
    let id = [0x11u8; 16];
    let permissions: i32 = -4;
    let mut padded = user_password.as_bytes().to_vec();
    padded.extend_from_slice(&PAD);
    padded.truncate(32);
    let mut hash = Md5::new();
    hash.update(&padded);
    hash.update(owner);
    hash.update(permissions.to_le_bytes());
    hash.update(id);
    let key = hash.finalize()[..5].to_vec();
    let user = rc4(&key, &PAD);
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();

    // The content stream is object 5, generation 0.
    let mut object_key = key.clone();
    object_key.extend_from_slice(&[5, 0, 0, 0, 0]);
    let object_key = Md5::digest(&object_key)[..10].to_vec();
    let content = rc4(&object_key, b"BT /F1 36 Tf 72 700 Td (Locked) Tj ET");

    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    let mut object = |out: &mut Vec<u8>, number: usize, body: &[u8]| {
        offsets.push(out.len());
        out.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    object(&mut out, 1, b"<< /Type /Catalog /Pages 2 0 R >>");
    object(&mut out, 2, b"<< /Type /Pages /Kids [4 0 R] /Count 1 >>");
    object(
        &mut out,
        3,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    );
    object(
        &mut out,
        4,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>",
    );
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend_from_slice(&content);
    stream.extend_from_slice(b"\nendstream");
    object(&mut out, 5, &stream);
    object(
        &mut out,
        6,
        format!(
            "<< /Filter /Standard /V 1 /R 2 /O <{}> /U <{}> /P {permissions} >>",
            hex(&owner),
            hex(&user)
        )
        .as_bytes(),
    );
    let xref = out.len();
    out.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size 7 /Root 1 0 R /Encrypt 6 0 R /ID [<{0}> <{0}>] >>\nstartxref\n{xref}\n%%EOF\n",
            hex(&id)
        )
        .as_bytes(),
    );
    out
}

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

#[cfg(any(target_os = "macos", target_os = "linux"))]
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

    let (status, report) = probe(&secret, &addr, &[]);
    assert!(status.success(), "{report}");
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

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
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
