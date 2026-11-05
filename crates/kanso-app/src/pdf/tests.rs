//! The reader with a real worker, headless (issue #43, step 3): page 1 draws
//! as a texture, the Back button and Escape close it, and a killed worker is
//! an error in the reader with Try Again, not a crash.
//!
//! These start `kanso-pdf-worker` and need PDFium, so they need both built:
//! `scripts/fetch-pdfium.sh` and `cargo build -p kanso-pdf --features
//! render`. Without them they say so and stop, rather than pass.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, VisualTestContext};
use kanso_pdf::client::WorkerPaths;
use kanso_pdf::fixtures::sample_pdf;
use kanso_store::Store;

use super::{PdfEvent, PdfReader, ReaderState};

/// The worker and library a development build has, or why not.
pub(crate) fn paths() -> Result<WorkerPaths, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // target/<profile>/deps/<test> -> target/<profile>/kanso-pdf-worker
    let worker = exe
        .parent()
        .and_then(Path::parent)
        .map(|dir| dir.join(kanso_pdf::client::WORKER_NAME))
        .filter(|path| path.is_file())
        .ok_or("kanso-pdf-worker isn't built: cargo build -p kanso-pdf --features render")?;
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
    let library: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform)
        .join(sub)
        .join(name);
    if !library.is_file() {
        return Err("PDFium isn't fetched: scripts/fetch-pdfium.sh".into());
    }
    Ok(WorkerPaths {
        exe: worker,
        library: Some(library.canonicalize().map_err(|e| e.to_string())?),
    })
}

struct Fixture {
    _dir: tempfile::TempDir,
    window: AnyWindowHandle,
    reader: Entity<PdfReader>,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

/// A reader for a two-page PDF attachment, with the worker running.
fn setup(cx: &mut TestAppContext, paths: WorkerPaths, pdf: Vec<u8>, name: &str) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let id = kanso_engine::attach::attach_pdf(&store, &pdf, name, None).unwrap();
    let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (window, reader) = cx.update(|cx| {
        gpui_kit::init(cx);
        let (window, reader) = gpui_kit::open_window(Default::default(), cx, |_, cx| {
            cx.new(|cx| {
                PdfReader::with_paths(
                    store.clone(),
                    id.clone(),
                    name.to_owned(),
                    1,
                    2.0,
                    Some(paths),
                    cx,
                )
            })
        })
        .unwrap();
        (window, reader)
    });
    let flag = closed.clone();
    cx.update(|cx| {
        cx.subscribe(&reader, move |_, event: &PdfEvent, _| match event {
            PdfEvent::Close => flag.store(true, std::sync::atomic::Ordering::SeqCst),
        })
        .detach();
    });
    let fixture = Fixture {
        _dir: dir,
        window,
        reader,
        closed,
    };
    settle(cx, &fixture);
    fixture
}

/// Lets the worker answer and draws the result.
fn settle(cx: &mut TestAppContext, f: &Fixture) {
    for _ in 0..40 {
        cx.run_until_parked();
        let busy = cx.read(|cx| matches!(f.reader.read(cx).state(), ReaderState::Loading));
        if !busy {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    f.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    cx.run_until_parked();
}

fn has(cx: &mut TestAppContext, f: &Fixture, selector: &'static str) -> bool {
    VisualTestContext::from_window(f.window, cx)
        .debug_bounds(selector)
        .is_some()
}

fn state(cx: &mut TestAppContext, f: &Fixture) -> ReaderState {
    cx.read(|cx| f.reader.read(cx).state().clone())
}

#[gpui_kit::test]
fn page_one_draws_as_a_texture(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(2), "report.pdf");
    let ReaderState::Showing {
        page,
        pages,
        width,
        height,
    } = state(cx, &f)
    else {
        panic!("{:?}", state(cx, &f));
    };
    assert_eq!((page, pages), (1, 2));
    // 880 points at 2x: a Letter page's shape.
    assert_eq!((width, height), (1760, 2277));
    assert!(has(cx, &f, "pdf-page"), "the page isn't drawn");
    assert!(has(cx, &f, "pdf-page-label"));
    assert!(!has(cx, &f, "pdf-error"));
    let image = cx.read(|cx| f.reader.read(cx).image.clone()).unwrap();
    let bytes = image.as_bytes(0).unwrap();
    assert_eq!(bytes.len(), 1760 * 2277 * 4);
    assert_eq!(
        bytes[..4],
        [255, 255, 255, 255],
        "the page's corner is white"
    );
    assert!(
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] < 64 && p[1] < 64 && p[2] < 64),
        "the page has no text on it"
    );
}

#[cfg(unix)]
#[gpui_kit::test]
fn a_killed_worker_is_an_error_in_the_reader_and_try_again_works(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(2), "report.pdf");
    let pid = cx
        .read(|cx| f.reader.read(cx).worker_pid())
        .expect("the worker is running");
    assert!(
        std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    // Asking for another page finds the worker gone.
    f.reader.update(cx, |reader, cx| reader.show_page(1, cx));
    settle(cx, &f);
    assert_eq!(
        state(cx, &f),
        ReaderState::Failed("This PDF couldn't be opened.".into())
    );
    assert!(has(cx, &f, "pdf-error"));
    assert!(has(cx, &f, "pdf-retry"));
    assert!(!has(cx, &f, "pdf-page"));

    // Try Again starts a fresh worker.
    f.reader.update(cx, |reader, cx| reader.show_page(0, cx));
    settle(cx, &f);
    assert!(matches!(
        state(cx, &f),
        ReaderState::Showing { page: 1, .. }
    ));
}

#[gpui_kit::test]
fn a_missing_worker_says_so_instead_of_crashing(cx: &mut TestAppContext) {
    let missing = WorkerPaths {
        exe: "/nonexistent/kanso-pdf-worker".into(),
        library: None,
    };
    let f = setup(cx, missing, sample_pdf(1), "report.pdf");
    assert!(matches!(state(cx, &f), ReaderState::Failed(_)));
    assert!(has(cx, &f, "pdf-error"));
}

#[gpui_kit::test]
fn a_file_that_is_not_a_pdf_to_the_worker_is_an_error(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    // Starts with %PDF- so it can be attached, but is not a document.
    let f = setup(cx, paths, b"%PDF-1.7 not really".to_vec(), "broken.pdf");
    assert_eq!(
        state(cx, &f),
        ReaderState::Failed("This PDF couldn't be opened.".into())
    );
}

#[gpui_kit::test]
fn back_to_note_and_escape_close_the_reader(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(1), "report.pdf");
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let focus = vcx.read(|cx| f.reader.read(cx).focus_handle().clone());
    vcx.update(|window, cx| window.focus(&focus, cx));
    vcx.simulate_keystrokes("escape");
    assert!(f.closed.load(std::sync::atomic::Ordering::SeqCst));

    f.closed.store(false, std::sync::atomic::Ordering::SeqCst);
    let back = vcx.debug_bounds("pdf-back").unwrap().center();
    vcx.simulate_click(back, Default::default());
    assert!(f.closed.load(std::sync::atomic::Ordering::SeqCst));
}
