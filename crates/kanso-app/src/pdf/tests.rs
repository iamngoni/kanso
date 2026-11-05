//! The reader with a real worker, headless (issue #43, steps 3 and 4): pages
//! draw as textures and scroll as one column, zoom redraws without blanking,
//! memory stays inside its budget, dark mode turns text and keeps pictures,
//! a locked PDF takes its password without keeping it, and a killed worker is
//! an error in the reader, not a crash.
//!
//! These start `kanso-pdf-worker` and need PDFium, so they need both built:
//! `scripts/fetch-pdfium.sh` and `cargo build -p kanso-pdf --features
//! render`. Without them they say so and stop, rather than pass.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, VisualTestContext};
use kanso_pdf::client::WorkerPaths;
use kanso_pdf::fixtures::{
    CHART_BARS, PHOTO_CENTRE, PHOTO_RGB, encrypted_pdf, photo_pdf, sample_pdf,
};
use kanso_store::Store;

use super::layout::Zoom;
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

fn worker_paths() -> WorkerPaths {
    paths().unwrap()
}

struct Fixture {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    id: String,
    window: AnyWindowHandle,
    reader: Entity<PdfReader>,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

fn open_reader(
    cx: &mut TestAppContext,
    store: &Arc<Store>,
    id: &str,
    name: &str,
    paths: WorkerPaths,
    dark: bool,
) -> (AnyWindowHandle, Entity<PdfReader>) {
    // The worker answers from its own thread, in real time.
    cx.executor().allow_parking();
    cx.update(|cx| {
        gpui_kit::init(cx);
        let (window, reader) = gpui_kit::open_window(Default::default(), cx, |window, cx| {
            if dark {
                Theme::change(ThemeMode::Dark, Some(window), cx);
            }
            cx.new(|cx| {
                PdfReader::with_paths(
                    store.clone(),
                    id.to_owned(),
                    name.to_owned(),
                    1,
                    Some(paths),
                    window,
                    cx,
                )
            })
        })
        .unwrap();
        (window, reader)
    })
}

/// A reader for a PDF attachment, with the worker running.
fn setup_with(
    cx: &mut TestAppContext,
    paths: WorkerPaths,
    pdf: Vec<u8>,
    name: &str,
    dark: bool,
) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let id = kanso_engine::attach::attach_pdf(&store, &pdf, name, None).unwrap();
    let (window, reader) = open_reader(cx, &store, &id, name, paths, dark);
    let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = closed.clone();
    cx.update(|cx| {
        cx.subscribe(&reader, move |_, event: &PdfEvent, _| match event {
            PdfEvent::Close => flag.store(true, std::sync::atomic::Ordering::SeqCst),
            PdfEvent::Write(_) => {}
        })
        .detach();
    });
    let fixture = Fixture {
        dir,
        store,
        id,
        window,
        reader,
        closed,
    };
    drive(cx, &fixture, |reader| {
        !matches!(reader.state(), ReaderState::Loading)
    });
    fixture
}

fn setup(cx: &mut TestAppContext, paths: WorkerPaths, pdf: Vec<u8>, name: &str) -> Fixture {
    setup_with(cx, paths, pdf, name, false)
}

/// Runs the executor and draws frames until `done`, giving the worker real
/// time to answer. Panics after ten seconds.
fn drive(cx: &mut TestAppContext, f: &Fixture, done: impl Fn(&PdfReader) -> bool) {
    let start = Instant::now();
    loop {
        cx.run_until_parked();
        f.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
        if cx.read(|cx| done(f.reader.read(cx))) {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the reader never got there: {:?}",
            cx.read(|cx| f.reader.read(cx).state().clone())
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Every visible page and every thumbnail on screen has its picture.
fn drawn(reader: &PdfReader) -> bool {
    reader
        .visible_pages()
        .all(|page| reader.page_texture(page).is_some())
}

fn has(cx: &mut TestAppContext, f: &Fixture, selector: &'static str) -> bool {
    VisualTestContext::from_window(f.window, cx)
        .debug_bounds(selector)
        .is_some()
}

fn state(cx: &mut TestAppContext, f: &Fixture) -> ReaderState {
    cx.read(|cx| f.reader.read(cx).state().clone())
}

fn act(
    cx: &mut TestAppContext,
    f: &Fixture,
    what: impl FnOnce(&mut PdfReader, &mut gpui_kit::Window, &mut gpui_kit::Context<PdfReader>),
) {
    f.window
        .update(cx, |_, window, cx| {
            f.reader.update(cx, |reader, cx| what(reader, window, cx))
        })
        .unwrap();
}

/// The picture's pixel at a point of the page, as RGB.
fn pixel(image: &gpui_kit::RenderImage, x_pt: f32, y_pt: f32) -> [u8; 3] {
    let size = image.size(0);
    let (w, h) = (size.width.0 as f32, size.height.0 as f32);
    let (x, y) = ((x_pt / 612. * w) as usize, (y_pt / 792. * h) as usize);
    let at = (y * w as usize + x) * 4;
    let bytes = image.as_bytes(0).unwrap();
    [bytes[at + 2], bytes[at + 1], bytes[at]]
}

#[gpui_kit::test]
fn page_one_draws_as_a_texture(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(2), "report.pdf");
    drive(cx, &f, drawn);
    assert_eq!(state(cx, &f), ReaderState::Showing { page: 1, pages: 2 });
    assert!(has(cx, &f, "pdf-page-0"), "the page isn't drawn");
    assert!(has(cx, &f, "pdf-page-label"));
    assert!(!has(cx, &f, "pdf-error"));
    let image = cx.read(|cx| f.reader.read(cx).page_texture(0)).unwrap();
    let bytes = image.as_bytes(0).unwrap();
    let size = image.size(0);
    assert_eq!(
        bytes.len(),
        size.width.0 as usize * size.height.0 as usize * 4
    );
    assert_eq!(bytes[..4], [255, 255, 255, 255], "the corner is white");
    assert!(
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] < 64 && p[1] < 64 && p[2] < 64),
        "the page has no text on it"
    );
}

#[gpui_kit::test]
fn a_hundred_pages_scroll_within_the_pixel_budget(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(100), "long.pdf");
    assert_eq!(
        state(cx, &f),
        ReaderState::Showing {
            page: 1,
            pages: 100
        }
    );
    // A budget of a few pages, so that eviction has to work all the way.
    let one_page = {
        drive(cx, &f, drawn);
        let image = cx.read(|cx| f.reader.read(cx).page_texture(0)).unwrap();
        let size = image.size(0);
        size.width.0 as usize * size.height.0 as usize * 4
    };
    let limit = one_page * 6;
    f.reader
        .update(cx, |reader, _| reader.limit_pages_to(limit));

    let total = cx.read(|cx| f.reader.read(cx).column_height());
    let step = 600.;
    let (mut y, mut steps, mut worst) = (0., 0, Duration::ZERO);
    while y < total {
        act(cx, &f, |reader, window, cx| reader.scroll_to(y, window, cx));
        let asked = Instant::now();
        drive(cx, &f, drawn);
        worst = worst.max(asked.elapsed());
        let (used, limit) = cx.read(|cx| f.reader.read(cx).page_bytes());
        let visible = cx.read(|cx| f.reader.read(cx).visible_pages().len());
        assert!(
            used <= limit.max(visible * one_page * 2),
            "{used} bytes held over a budget of {limit} at {y}"
        );
        y += step;
        steps += 1;
    }
    assert!(steps > 100, "the column was only {steps} steps long");
    let last = cx.read(|cx| f.reader.read(cx).visible_pages().end);
    assert_eq!(last, 100, "the end of the document wasn't reached");
    assert!(
        worst < Duration::from_secs(2),
        "a screenful took {worst:?} to draw"
    );
}

#[gpui_kit::test]
fn zoom_redraws_without_blanking(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(3), "report.pdf");
    drive(cx, &f, drawn);
    let before = cx.read(|cx| f.reader.read(cx).page_texture(0)).unwrap();
    let before_box = VisualTestContext::from_window(f.window, cx)
        .debug_bounds("pdf-page-0")
        .unwrap();

    act(cx, &f, |reader, window, cx| {
        reader.zoom_to(Zoom::Percent(300), window, cx)
    });
    // Drawn at once, before the worker has answered: the page is bigger, and
    // still has its old picture.
    f.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    let after_box = VisualTestContext::from_window(f.window, cx)
        .debug_bounds("pdf-page-0")
        .unwrap();
    assert!(after_box.size.width > before_box.size.width);
    let during = cx.read(|cx| f.reader.read(cx).page_texture(0)).unwrap();
    assert!(Arc::ptr_eq(&before, &during), "the page blanked");

    // Then, once the zoom has stopped moving (the test clock is virtual), it
    // is drawn again at the new size.
    cx.executor().advance_clock(super::ZOOM_SETTLE * 2);
    drive(cx, &f, |reader| {
        reader
            .page_texture(0)
            .is_some_and(|image| image.size(0).width.0 > before.size(0).width.0)
    });
    assert_eq!(cx.read(|cx| f.reader.read(cx).shown_zoom_percent()), 300);
}

#[gpui_kit::test]
fn thumbnails_draw_and_a_click_goes_to_the_page(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(6), "report.pdf");
    drive(cx, &f, |reader| {
        drawn(reader) && (0..3).all(|page| reader.thumb_texture(page).is_some())
    });
    assert!(has(cx, &f, "pdf-rail"));
    let thumb = cx.read(|cx| f.reader.read(cx).thumb_texture(0)).unwrap();
    // About 120 points wide, drawn for the display.
    assert!(thumb.size(0).width.0 >= 120 && thumb.size(0).width.0 <= 480);

    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let at = vcx.debug_bounds("pdf-thumb-4").unwrap().center();
    vcx.simulate_click(at, Default::default());
    drive(cx, &f, |reader| {
        matches!(reader.state(), ReaderState::Showing { page: 5, .. })
    });
}

#[gpui_kit::test]
fn dark_mode_turns_the_text_and_keeps_a_photo_and_a_chart(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup_with(cx, paths, photo_pdf(), "field.pdf", true);
    drive(cx, &f, drawn);
    let image = cx.read(|cx| f.reader.read(cx).page_texture(0)).unwrap();

    // The page was white and is a dark page now; the text is light on it.
    let page = pixel(&image, 10., 10.);
    assert!(page.iter().all(|c| *c < 60), "the page is {page:?}");
    let bytes = image.as_bytes(0).unwrap();
    assert!(
        bytes.as_chunks::<4>().0.iter().any(|p| p[0] > 200),
        "no light text"
    );
    // The photo is the photo's own colour, exactly.
    assert_eq!(pixel(&image, PHOTO_CENTRE[0], PHOTO_CENTRE[1]), PHOTO_RGB);
    // The chart's bars are red and blue still.
    let red = pixel(&image, CHART_BARS[0].0[0], CHART_BARS[0].0[1]);
    assert!(
        red[0] > 120 && red[0] > red[1] + 60 && red[0] > red[2] + 60,
        "{red:?}"
    );
    let blue = pixel(&image, CHART_BARS[1].0[0], CHART_BARS[1].0[1]);
    assert!(blue[2] > 120 && blue[2] > blue[0] + 60, "{blue:?}");

    // The toggle shows the page as printed, and is remembered for this PDF.
    assert!(has(cx, &f, "pdf-original"));
    act(cx, &f, |reader, window, cx| {
        reader.pick_original(true, window, cx)
    });
    drive(cx, &f, |reader| {
        reader.page_texture(0).is_some_and(|image| {
            let size = image.size(0);
            size.width.0 > 0 && image.as_bytes(0).unwrap()[..4] == [255, 255, 255, 255]
        })
    });
    let paths = worker_paths();
    let (window, reader) = open_reader(cx, &f.store, &f.id, "field.pdf", paths, true);
    assert!(cx.read(|cx| reader.read(cx).shows_original()));
    drop((window, reader));
}

#[gpui_kit::test]
fn a_locked_pdf_takes_its_password_and_never_keeps_it(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, encrypted_pdf("hunter22"), "locked.pdf");
    assert_eq!(state(cx, &f), ReaderState::NeedsPassword { wrong: false });
    assert!(has(cx, &f, "pdf-password"));
    assert!(!has(cx, &f, "pdf-password-wrong"));

    act(cx, &f, |reader, window, cx| {
        reader.type_password("guess-one", window, cx)
    });
    drive(cx, &f, |reader| {
        matches!(reader.state(), ReaderState::NeedsPassword { wrong: true })
    });
    assert!(has(cx, &f, "pdf-password-wrong"));

    act(cx, &f, |reader, window, cx| {
        reader.type_password("hunter22", window, cx)
    });
    drive(cx, &f, |reader| {
        matches!(reader.state(), ReaderState::Showing { .. }) && drawn(reader)
    });
    assert!(!has(cx, &f, "pdf-password"));

    // Neither word is in the reader, or anywhere the library keeps things.
    let dump = cx.read(|cx| f.reader.read(cx).debug_dump(cx));
    for word in ["guess-one", "hunter22"] {
        assert!(!dump.contains(word), "the reader holds {word}");
        for entry in std::fs::read_dir(f.dir.path()).unwrap() {
            let bytes = std::fs::read(entry.unwrap().path()).unwrap();
            assert!(
                !bytes.windows(word.len()).any(|w| w == word.as_bytes()),
                "{word} reached the library"
            );
        }
    }

    // A second open asks again.
    let paths = worker_paths();
    let (window, reader) = open_reader(cx, &f.store, &f.id, "locked.pdf", paths, false);
    let again = Fixture {
        dir: tempfile::tempdir().unwrap(),
        store: f.store.clone(),
        id: f.id.clone(),
        window,
        reader,
        closed: f.closed.clone(),
    };
    drive(cx, &again, |reader| {
        !matches!(reader.state(), ReaderState::Loading)
    });
    assert_eq!(
        state(cx, &again),
        ReaderState::NeedsPassword { wrong: false }
    );
}

#[cfg(unix)]
#[gpui_kit::test]
fn a_killed_worker_is_an_error_in_the_reader_and_try_again_works(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(40), "report.pdf");
    drive(cx, &f, drawn);
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
    // Scrolling asks for pages the dead worker can't draw.
    act(cx, &f, |reader, window, cx| {
        reader.scroll_to(20_000., window, cx)
    });
    drive(cx, &f, |reader| {
        matches!(reader.state(), ReaderState::Failed(_))
    });
    assert_eq!(
        state(cx, &f),
        ReaderState::Failed("This PDF couldn't be opened.".into())
    );
    assert!(has(cx, &f, "pdf-error"));
    assert!(has(cx, &f, "pdf-retry"));
    assert!(!has(cx, &f, "pdf-page-0"));

    // Try Again starts a fresh worker.
    act(cx, &f, |reader, window, cx| reader.retry(window, cx));
    drive(cx, &f, |reader| {
        matches!(reader.state(), ReaderState::Showing { .. })
    });
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
fn keys_scroll_zoom_and_close(cx: &mut TestAppContext) {
    let Ok(paths) = paths() else {
        eprintln!("SKIPPED: {}", paths().unwrap_err());
        return;
    };
    let f = setup(cx, paths, sample_pdf(30), "report.pdf");
    drive(cx, &f, drawn);
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let focus = vcx.read(|cx| f.reader.read(cx).focus_handle().clone());
    vcx.update(|window, cx| window.focus(&focus, cx));

    vcx.simulate_keystrokes("pagedown pagedown pagedown");
    drive(
        cx,
        &f,
        |reader| matches!(reader.state(), ReaderState::Showing { page, .. } if *page > 1),
    );
    let fit = cx.read(|cx| f.reader.read(cx).shown_zoom_percent());
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("secondary-=");
    assert!(vcx.read(|cx| f.reader.read(cx).shown_zoom_percent()) > fit);
    vcx.simulate_keystrokes("secondary-0");
    assert_eq!(vcx.read(|cx| f.reader.read(cx).shown_zoom_percent()), 100);

    vcx.simulate_keystrokes("escape");
    assert!(f.closed.load(std::sync::atomic::Ordering::SeqCst));
    f.closed.store(false, std::sync::atomic::Ordering::SeqCst);
    let back = vcx.debug_bounds("pdf-back").unwrap().center();
    vcx.simulate_click(back, Default::default());
    assert!(f.closed.load(std::sync::atomic::Ordering::SeqCst));
}
