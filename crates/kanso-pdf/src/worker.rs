//! The helper process (issue #43, step 3): PDFium behind the protocol.
//!
//! `kanso-pdf-worker` is started by the app with an empty environment and
//! only its stdin and stdout. It contains itself (see [`crate::contain`]),
//! and only then loads PDFium, opens the bytes it is sent (never a path) and
//! draws the pages it is asked for. It has no store, no keys, no network
//! code and no window; a crash or a kill costs the app one reader.

use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use pdfium_render::prelude::*;

use crate::client::refusal_words;
use crate::contain::{self, Allow, ContainError, MEMORY_BUDGET};
use crate::invert::{self, PixelRect};
use crate::model::Quad;
use crate::protocol::{
    BYTES_PER_PIXEL, ErrorKind, FrameError, MAX_BLOB_BYTES, MAX_PAGE_CHARS, MAX_PAGES, Request,
    Response, check_document, page_is_sound, read_frame, render_size, text_window, tidy_text,
    write_frame,
};
use crate::text::{PageText, TextChar};

/// Exit code for a worker that could not be contained or could not start.
pub const EXIT_UNSAFE: u8 = 70;
/// Exit code for a worker that went over its memory budget.
pub const EXIT_OVER_BUDGET: u8 = 75;

/// What the command line asked for.
#[derive(Debug, Default, PartialEq, Eq)]
struct Args {
    /// The PDFium library, when not beside the worker.
    library: Option<PathBuf>,
    /// `--containment-probe <file> <address>`: contain, then report whether a
    /// file and a socket can still be reached (the containment tests).
    probe: Option<(PathBuf, String)>,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--library" => {
                parsed.library = Some(args.next().ok_or("--library needs a path")?.into());
            }
            "--containment-probe" => {
                let file = args.next().ok_or("--containment-probe needs a file")?;
                let addr = args.next().ok_or("--containment-probe needs an address")?;
                parsed.probe = Some((file.into(), addr));
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(parsed)
}

/// Where the PDFium library may be: beside the worker, or for a macOS app
/// bundle in `Contents/Frameworks`.
fn library_candidates(exe_dir: &Path) -> Vec<PathBuf> {
    let name = Pdfium::pdfium_platform_library_name();
    let mut found = Vec::new();
    if cfg!(target_os = "macos")
        && let Some(contents) = exe_dir.parent()
    {
        found.push(contents.join("Frameworks").join(&name));
    }
    found.push(exe_dir.join(&name));
    found
}

fn find_library(args: &Args, exe_dir: &Path) -> Option<PathBuf> {
    match &args.library {
        Some(path) => path.is_file().then(|| path.clone()),
        None => library_candidates(exe_dir)
            .into_iter()
            .find(|path| path.is_file()),
    }
    .and_then(|path| path.canonicalize().ok())
}

/// Runs the worker; the binary's whole `main`.
pub fn run() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("kanso-pdf-worker: {message}");
            return ExitCode::from(2);
        }
    };
    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.canonicalize().ok())
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    else {
        eprintln!("kanso-pdf-worker: can't tell where it is");
        return ExitCode::from(EXIT_UNSAFE);
    };
    let Some(library) = find_library(&args, &exe_dir) else {
        eprintln!("kanso-pdf-worker: the PDFium library is missing");
        return ExitCode::from(EXIT_UNSAFE);
    };
    let mut read_dirs = vec![exe_dir];
    if let Some(dir) = library.parent() {
        read_dirs.push(dir.to_path_buf());
    }
    let allow = Allow { read_dirs };

    if let Some((file, addr)) = args.probe {
        return match contain::contain(&allow) {
            Ok(()) => {
                probe(&file, &addr, &library);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("kanso-pdf-worker: {e}");
                ExitCode::from(EXIT_UNSAFE)
            }
        };
    }

    // Contained first; only then is PDFium loaded.
    let pdfium = contain::contained_then(
        || contain::contain(&allow),
        || {
            Pdfium::bind_to_library(&library)
                .map(Pdfium::new)
                .map_err(|e| ContainError(format!("couldn't load PDFium: {e}")))
        },
    );
    let pdfium = match pdfium {
        Ok(pdfium) => pdfium,
        Err(e) => {
            eprintln!("kanso-pdf-worker: {e}");
            return ExitCode::from(EXIT_UNSAFE);
        }
    };
    serve(
        &pdfium,
        &mut BufReader::new(io::stdin().lock()),
        &mut BufWriter::new(io::stdout().lock()),
    )
}

/// What a contained worker can still reach, one `name=result` line each.
fn probe(file: &Path, addr: &str, library: &Path) {
    let word = |reached: bool| if reached { "allowed" } else { "denied" };
    println!("contained=yes");
    println!(
        "read_file={}",
        word(
            std::fs::File::open(file)
                .and_then(|mut f| f.read(&mut [0u8; 1]))
                .is_ok()
        )
    );
    println!(
        "list_dir={}",
        word(
            file.parent()
                .is_some_and(|dir| std::fs::read_dir(dir).is_ok())
        )
    );
    println!(
        "connect={}",
        word(std::net::TcpStream::connect(addr).is_ok())
    );
    println!(
        "spawn={}",
        word(
            std::process::Command::new("/bin/echo")
                .arg("x")
                .output()
                .is_ok()
        )
    );
    println!("list_processes={}", word(lists_processes()));
    println!("inherited_fds={}", word(has_inherited_fds()));
    println!(
        "read_library={}",
        word(
            std::fs::File::open(library)
                .and_then(|mut f| f.read(&mut [0u8; 1]))
                .is_ok()
        )
    );
    io::stdout().flush().ok();
}

/// Whether the other processes on the system can be listed: the process
/// table on macOS (`kern.proc.all`), `/proc` on Linux.
#[cfg(target_os = "macos")]
fn lists_processes() -> bool {
    // SAFETY: a size-only sysctl query.
    unsafe {
        let mut mib = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_ALL, 0];
        let mut len = 0usize;
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            std::ptr::null_mut(),
            &mut len,
            std::ptr::null_mut(),
            0,
        ) == 0
    }
}

#[cfg(not(target_os = "macos"))]
fn lists_processes() -> bool {
    std::fs::read_dir("/proc").is_ok()
}

/// Whether a descriptor beyond stdin, stdout and stderr is open.
#[cfg(unix)]
fn has_inherited_fds() -> bool {
    // SAFETY: fcntl on a number that may not be open just fails.
    (3..1024).any(|fd| unsafe { libc::fcntl(fd, libc::F_GETFD) } != -1)
}

#[cfg(not(unix))]
fn has_inherited_fds() -> bool {
    false
}

/// A failed request; the words are the app's own for the kind.
fn failure(kind: ErrorKind) -> (Response, Vec<u8>) {
    (
        Response::Error {
            kind,
            message: refusal_words(kind).to_owned(),
        },
        Vec::new(),
    )
}

/// Answers requests until the app closes the document or the pipe.
fn serve(pdfium: &Pdfium, input: &mut impl Read, output: &mut impl Write) -> ExitCode {
    let mut document: Option<PdfDocument<'_>> = None;
    serve_document(pdfium, input, output, true, None, &mut document)
}

/// [`serve`], for the loop the phones run inside the app (see
/// [`crate::local`]). There the process is the whole app, so its resident
/// size is not the reader's to judge: `watch_memory` is off, and the page and
/// document caps are the budget.
///
/// PDFium is not safe to enter from two threads at once, whatever the
/// binding's per-call lock (readers on separate threads crashed it in the
/// tests), and in one process every reader shares it. So each request, and
/// the closing of the document, is handled holding `gate`: one reader is
/// inside PDFium at a time, and the others wait their turn between
/// requests, not while they wait for the app to ask.
pub(crate) fn serve_with(
    pdfium: &Pdfium,
    input: &mut impl Read,
    output: &mut impl Write,
    watch_memory: bool,
    gate: &std::sync::Mutex<()>,
) -> ExitCode {
    let mut document: Option<PdfDocument<'_>> = None;
    let code = serve_document(
        pdfium,
        input,
        output,
        watch_memory,
        Some(gate),
        &mut document,
    );
    let _turn = gate
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    drop(document);
    code
}

fn serve_document<'a>(
    pdfium: &'a Pdfium,
    input: &mut impl Read,
    output: &mut impl Write,
    watch_memory: bool,
    gate: Option<&std::sync::Mutex<()>>,
    document: &mut Option<PdfDocument<'a>>,
) -> ExitCode {
    loop {
        let (request, blob) = match read_frame::<Request>(input, MAX_BLOB_BYTES) {
            Ok(Some(frame)) => frame,
            // The app let go.
            Ok(None) => return ExitCode::SUCCESS,
            Err(FrameError::Malformed | FrameError::TooLarge) => {
                let (response, blob) = failure(ErrorKind::Protocol);
                write_frame(output, &response, &blob).ok();
                return ExitCode::from(2);
            }
            Err(FrameError::Io(_)) => return ExitCode::from(2),
        };
        // This request is handled alone (see [`serve_with`]); the turn ends
        // with the iteration.
        let _turn = gate.map(|g| g.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let (response, blob, done) = match request {
            Request::Open { password } => {
                let (response, blob) = if document.is_some() {
                    failure(ErrorKind::Protocol)
                } else {
                    match open(pdfium, blob, password.as_deref()) {
                        Ok((opened, doc)) => {
                            *document = Some(doc);
                            (opened, Vec::new())
                        }
                        Err(failed) => failed,
                    }
                };
                (response, blob, false)
            }
            Request::Render {
                page,
                width_px,
                smart_invert,
            } => {
                let (response, blob) = match &*document {
                    Some(doc) => render(doc, page, width_px, smart_invert),
                    None => failure(ErrorKind::Protocol),
                };
                (response, blob, false)
            }
            Request::Text { page } => {
                let (response, blob) = match &*document {
                    Some(doc) => text_page(doc, page),
                    None => failure(ErrorKind::Protocol),
                };
                (response, blob, false)
            }
            Request::PlainText { from, count } => {
                let (response, blob) = match &*document {
                    Some(doc) => plain_text(doc, from, count),
                    None => failure(ErrorKind::Protocol),
                };
                (response, blob, false)
            }
            Request::Close => (Response::Closed, Vec::new(), true),
        };
        if write_frame(output, &response, &blob).is_err() {
            return ExitCode::from(2);
        }
        if done {
            return ExitCode::SUCCESS;
        }
        // macOS treats the address-space limit as advice; the resident size
        // is the budget there, and the same check backs it up elsewhere.
        if watch_memory && contain::resident_bytes().is_some_and(|bytes| bytes > MEMORY_BUDGET) {
            return ExitCode::from(EXIT_OVER_BUDGET);
        }
    }
}

fn open<'a>(
    pdfium: &'a Pdfium,
    bytes: Vec<u8>,
    password: Option<&str>,
) -> Result<(Response, PdfDocument<'a>), (Response, Vec<u8>)> {
    let doc = pdfium
        .load_pdf_from_byte_vec(bytes, password)
        .map_err(|e| match e {
            PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => {
                if password.is_some() {
                    failure(ErrorKind::WrongPassword)
                } else {
                    failure(ErrorKind::PasswordRequired)
                }
            }
            _ => failure(ErrorKind::Unreadable),
        })?;
    let pages = u32::try_from(doc.pages().len()).unwrap_or(u32::MAX);
    // The count is checked before any per-page work: a page tree that
    // claims millions of pages is refused without being walked.
    if pages > MAX_PAGES {
        return Err(failure(ErrorKind::TooManyPages));
    }
    let sizes: Vec<[f32; 2]> = doc
        .pages()
        .page_sizes()
        .map_err(|_| failure(ErrorKind::Unreadable))?
        .iter()
        .map(|rect| [rect.width().value, rect.height().value])
        .collect();
    // Every page's declared size is the PDF's claim: an absurd one refuses
    // the document, so no layout is ever sized by it.
    check_document(pages, &sizes).map_err(failure)?;
    // Only -1 is "no security handler"; a newer revision than the wrapper
    // knows (AES-256's 5 and 6) is encrypted too.
    let encrypted = !matches!(
        doc.permissions().security_handler_revision(),
        Ok(PdfSecurityHandlerRevision::Unprotected)
    );
    Ok((
        Response::Opened {
            pages,
            sizes,
            encrypted,
        },
        doc,
    ))
}

/// How a page is drawn: every flag that matters set on purpose, none left to
/// the wrapper's defaults (which draw form widgets and annotations, and
/// swap the bitmap to RGBA).
///
/// - No form data and no annotations: nothing a PDF's fields or annotation
///   appearances hold is drawn (issue #43, "Security and privacy"). The
///   reader draws its own highlights over the texture.
/// - No platform text: PDFium draws every glyph itself instead of handing
///   them to the system's font code.
/// - The image cache is limited, so a page of large images stays within the
///   worker's memory budget.
/// - BGRA, as PDFium writes it: GPUI's textures are BGRA.
/// - A white page, cleared before drawing, whatever the PDF's own backdrop.
fn render_config(width: u32, height: u32) -> PdfRenderConfig {
    PdfRenderConfig::new()
        .set_target_size(width as i32, height as i32)
        .set_format(PdfBitmapFormat::BGRA)
        .set_reverse_byte_order(false)
        .clear_before_rendering(true)
        .set_clear_color(PdfColor::WHITE)
        .render_form_data(false)
        .render_annotations(false)
        .disable_native_text_rendering(true)
        .limit_render_image_cache_size(true)
        .use_lcd_text_rendering(false)
        .use_grayscale_rendering(false)
        .use_print_quality(false)
        .force_half_tone(false)
        .render_fills_as_strokes(false)
}

fn render(
    doc: &PdfDocument<'_>,
    index: u32,
    width_px: u32,
    smart_invert: bool,
) -> (Response, Vec<u8>) {
    let Ok(page_index) = PdfPageIndex::try_from(index) else {
        return failure(ErrorKind::NoSuchPage);
    };
    let Ok(page) = doc.pages().get(page_index) else {
        return failure(ErrorKind::NoSuchPage);
    };
    let Some((width, height)) = render_size(page.width().value, page.height().value, width_px)
    else {
        return failure(ErrorKind::Unreadable);
    };
    let config = render_config(width, height);
    let Ok(bitmap) = page.render_with_config(&config) else {
        return failure(ErrorKind::Internal);
    };
    let (w, h) = (bitmap.width() as u32, bitmap.height() as u32);
    let mut bytes = bitmap.as_raw_bytes();
    if !page_is_sound(w, h, bytes.len()) || bytes.len() != w as usize * h as usize * BYTES_PER_PIXEL
    {
        return failure(ErrorKind::Internal);
    }
    if smart_invert {
        let keep = invert::keep_rects(&picture_boxes(&page, &config, w, h), w, h);
        invert::smart_invert(&mut bytes, w, h, &keep);
    }
    (
        Response::Page {
            page: index,
            width: w,
            height: h,
        },
        bytes,
    )
}

/// Where the page's pictures are in the bitmap just drawn, in pixels. Only
/// the page's own picture objects are looked at (not those inside a form
/// XObject); the same transform PDFium drew the page with places them, so a
/// rotated page is right.
fn picture_boxes(page: &PdfPage<'_>, config: &PdfRenderConfig, w: u32, h: u32) -> Vec<PixelRect> {
    let mut boxes = Vec::new();
    for object in page.objects().iter() {
        if object.as_image_object().is_none() {
            continue;
        }
        let Ok(bounds) = object.bounds() else {
            continue;
        };
        let rect = bounds.to_rect();
        let corner = |x, y| {
            page.points_to_pixels(x, y, config)
                .ok()
                .map(|(x, y)| (x as f32, y as f32))
        };
        if let (Some(a), Some(b)) = (
            corner(rect.left(), rect.top()),
            corner(rect.right(), rect.bottom()),
        ) && let Some(rect) = PixelRect::from_corners(a, b, w, h)
        {
            boxes.push(rect);
        }
    }
    boxes
}

/// A page's number of clockwise quarter turns from `/Rotate`.
fn page_turns(page: &PdfPage<'_>) -> u8 {
    match page.rotation() {
        Ok(PdfPageRenderRotation::Degrees90) => 1,
        Ok(PdfPageRenderRotation::Degrees180) => 2,
        Ok(PdfPageRenderRotation::Degrees270) => 3,
        _ => 0,
    }
}

/// Where a point in the page's own (unrotated) space lies on the page as
/// displayed, as fractions of the displayed page from the top-left. `bbox`
/// is the unrotated page box `[left, bottom, right, top]`, which is where
/// PDFium's character boxes are measured from (the crop box can sit
/// anywhere, not at the origin).
fn displayed(turns: u8, bbox: [f64; 4], x: f64, y: f64) -> (f64, f64) {
    let [left, bottom, right, top] = bbox;
    let u = (x - left) / (right - left);
    let v = 1.0 - (y - bottom) / (top - bottom);
    match turns % 4 {
        0 => (u, v),
        1 => (1.0 - v, u),
        2 => (1.0 - u, 1.0 - v),
        _ => (v, 1.0 - u),
    }
}

/// A character's quad on the displayed page, from its box in page space.
fn displayed_quad(turns: u8, bbox: [f64; 4], l: f64, b: f64, r: f64, t: f64) -> Quad {
    let (ax, ay) = displayed(turns, bbox, l, b);
    let (bx, by) = displayed(turns, bbox, r, t);
    Quad::new(ax, ay, bx, by)
}

/// The text direction PDFium reports for a character as clockwise quarter
/// turns, 0 to 3. PDFium's angle is in radians and runs clockwise on the
/// page: text that reads top to bottom is 90 degrees, bottom to top 270
/// (checked against real glyphs in `tests/text.rs`).
fn char_turns(radians: f32) -> u8 {
    let quarter = f64::from(radians).to_degrees() / 90.0;
    if !quarter.is_finite() {
        return 0;
    }
    (quarter.round() as i64).rem_euclid(4) as u8
}

/// The page's text with every character's box. Spaces, line breaks and the
/// characters PDFium generated are not sent, as are any without a box; a
/// space or break (set by the PDF or generated by PDFium) is carried as the
/// `space_before` mark on the leftmost character of the word after it
/// ([`PageText::anchor_spaces`]). The page is cut at
/// [`MAX_PAGE_CHARS`] characters.
fn text_page(doc: &PdfDocument<'_>, index: u32) -> (Response, Vec<u8>) {
    let Ok(page_index) = PdfPageIndex::try_from(index) else {
        return failure(ErrorKind::NoSuchPage);
    };
    let Ok(page) = doc.pages().get(page_index) else {
        return failure(ErrorKind::NoSuchPage);
    };
    let turns = page_turns(&page);
    let (width, height) = (
        f64::from(page.width().value),
        f64::from(page.height().value),
    );
    let page_text = |chars| PageText {
        page: index + 1,
        width,
        height,
        rotation: u16::from(turns) * 90,
        chars,
    };
    let Ok(text) = page.text() else {
        // A page PDFium can't read the text of has none.
        return text_reply(index, page_text(Vec::new()), false);
    };
    // PDFium's character boxes are in the unrotated page's space.
    let bbox = page
        .boundaries()
        .bounding()
        .ok()
        .map(|b| b.bounds)
        .map(|r| {
            [
                f64::from(r.left().value),
                f64::from(r.bottom().value),
                f64::from(r.right().value),
                f64::from(r.top().value),
            ]
        })
        .filter(|[l, b, r, t]| r > l && t > b);
    let Some(bbox) = bbox else {
        return failure(ErrorKind::Internal);
    };
    let mut chars = Vec::new();
    let mut truncated = false;
    let mut space = false;
    for c in text.chars().iter() {
        let Some(ch) = c.unicode_char() else { continue };
        if ch.is_whitespace() {
            // A space, a tab, a line break: PDFium's, generated or not.
            space = true;
            continue;
        }
        if ch.is_control() || c.is_generated().unwrap_or(false) {
            continue;
        }
        let Ok(rect) = c.tight_bounds() else { continue };
        if chars.len() == MAX_PAGE_CHARS {
            truncated = true;
            break;
        }
        let quad = displayed_quad(
            turns,
            bbox,
            f64::from(rect.left().value),
            f64::from(rect.bottom().value),
            f64::from(rect.right().value),
            f64::from(rect.top().value),
        );
        chars.push(TextChar {
            ch,
            quad,
            turns: c.angle_radians().map_or(0, char_turns),
            space_before: std::mem::take(&mut space),
        });
    }
    let mut page = page_text(chars);
    page.anchor_spaces();
    text_reply(index, page, truncated)
}

fn text_reply(index: u32, page_text: PageText, truncated: bool) -> (Response, Vec<u8>) {
    match serde_json::to_vec(&page_text) {
        Ok(json) => (
            Response::TextPage {
                page: index,
                chars: u32::try_from(page_text.chars.len()).unwrap_or(u32::MAX),
                truncated,
            },
            json,
        ),
        Err(_) => failure(ErrorKind::Internal),
    }
}

/// The plain text of a window of pages: each page's characters as PDFium
/// reads them, tidied (see [`tidy_text`]) and cut at [`MAX_PAGE_CHARS`], all
/// in one blob with a length for each page.
fn plain_text(doc: &PdfDocument<'_>, from: u32, count: u32) -> (Response, Vec<u8>) {
    let pages = u32::try_from(doc.pages().len()).unwrap_or(u32::MAX);
    let Some(window) = text_window(pages, from, count) else {
        return failure(ErrorKind::NoSuchPage);
    };
    let mut lens = Vec::new();
    let mut blob = Vec::new();
    let mut truncated = false;
    for index in window {
        let Ok(page_index) = PdfPageIndex::try_from(index) else {
            return failure(ErrorKind::NoSuchPage);
        };
        let Ok(page) = doc.pages().get(page_index) else {
            return failure(ErrorKind::NoSuchPage);
        };
        // A page whose text PDFium can't read has none.
        let (text, cut) = match page.text() {
            Ok(text) => tidy_text(text.chars().iter().filter_map(|c| c.unicode_char())),
            Err(_) => (String::new(), false),
        };
        truncated |= cut;
        lens.push(u32::try_from(text.len()).unwrap_or(u32::MAX));
        blob.extend_from_slice(text.as_bytes());
    }
    (
        Response::PlainText {
            from,
            lens,
            truncated,
        },
        blob,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::annotated_pdf;

    /// One PDFium for the whole test process, one test at a time: binding
    /// the library twice in one process fails, PDFium is not safe to use
    /// from two threads at once, and the tests run side by side. Hold the
    /// guard for the whole test.
    fn pdfium() -> (std::sync::MutexGuard<'static, ()>, &'static Pdfium) {
        static PDFIUM: std::sync::OnceLock<Pdfium> = std::sync::OnceLock::new();
        static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = ONE_AT_A_TIME
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let pdfium = PDFIUM.get_or_init(|| {
            let library = crate::client::dev_library()
                .expect("PDFium is missing: run scripts/fetch-pdfium.sh");
            Pdfium::new(Pdfium::bind_to_library(library).expect("PDFium loads"))
        });
        (guard, pdfium)
    }

    proptest::proptest! {
        /// A point on the page lands on the displayed page, whichever way it
        /// is turned and wherever the page box sits; the corners go where a
        /// turned page puts them (top-left of an upright page, after one
        /// clockwise turn, is the top-right).
        #[test]
        fn a_point_on_the_page_lands_on_the_displayed_page(
            left in -500.0f64..500.0, bottom in -500.0f64..500.0,
            w in 1.0f64..2000.0, h in 1.0f64..2000.0,
            fx in 0.0f64..=1.0, fy in 0.0f64..=1.0,
            turns in 0u8..8,
        ) {
            let bbox = [left, bottom, left + w, bottom + h];
            let (x, y) = displayed(turns, bbox, left + fx * w, bottom + fy * h);
            proptest::prop_assert!((-1e-9..=1.0 + 1e-9).contains(&x) && (-1e-9..=1.0 + 1e-9).contains(&y));
            // Four turns are none.
            let again = displayed(turns + 4, bbox, left + fx * w, bottom + fy * h);
            proptest::prop_assert_eq!((x, y), again);
        }

        /// A character's quad is the same box as its corners, in order and
        /// on the page.
        #[test]
        fn a_characters_quad_is_ordered_and_on_the_page(
            w in 10.0f64..1000.0, h in 10.0f64..1000.0,
            a in 0.0f64..=1.0, b in 0.0f64..=1.0, c in 0.0f64..=1.0, d in 0.0f64..=1.0,
            turns in 0u8..4,
        ) {
            let q = displayed_quad(turns, [0.0, 0.0, w, h], a * w, b * h, c * w, d * h);
            proptest::prop_assert!(q.is_valid());
        }

        /// A direction in quarter turns comes back as those turns, and a
        /// small wobble in the angle doesn't change it.
        #[test]
        fn a_direction_is_read_in_quarter_turns(k in 0u32..16, wobble in -0.2f64..0.2) {
            let radians = (f64::from(k) * 90.0 + wobble * 90.0).to_radians() as f32;
            proptest::prop_assert_eq!(u32::from(char_turns(radians)), k % 4);
        }
    }

    #[test]
    fn an_angle_that_is_not_a_number_is_no_turn() {
        assert_eq!(char_turns(f32::NAN), 0);
        assert_eq!(char_turns(f32::INFINITY), 0);
    }

    #[test]
    fn the_corners_of_a_turned_page() {
        let bbox = [0.0, 0.0, 100.0, 200.0];
        // The page's top-left corner is (0, 200) in page space.
        assert_eq!(displayed(0, bbox, 0.0, 200.0), (0.0, 0.0));
        assert_eq!(displayed(1, bbox, 0.0, 200.0), (1.0, 0.0));
        assert_eq!(displayed(2, bbox, 0.0, 200.0), (1.0, 1.0));
        assert_eq!(displayed(3, bbox, 0.0, 200.0), (0.0, 1.0));
    }

    fn coloured_pixels(bgra: &[u8]) -> usize {
        bgra.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0].abs_diff(p[2]) > 60 || p[1].abs_diff(p[2]) > 60)
            .count()
    }

    /// The render flags are what keeps annotations and form fields off the
    /// page: the same document, drawn in this process with the worker's
    /// flags and then with those two turned on, shows the difference.
    /// (The worker as a process is tested in `tests/hostile.rs`.)
    #[test]
    fn the_render_flags_are_what_keep_annotations_and_fields_off_the_page() {
        let (_one_at_a_time, pdfium) = pdfium();
        let doc = pdfium
            .load_pdf_from_byte_vec(annotated_pdf(), None)
            .expect("the fixture opens");
        let page = doc.pages().get(0).unwrap();
        let draw = |config: PdfRenderConfig| {
            let bitmap = page.render_with_config(&config).unwrap();
            coloured_pixels(&bitmap.as_raw_bytes())
        };
        assert_eq!(draw(render_config(612, 792)), 0);
        let with_extras = render_config(612, 792)
            .render_annotations(true)
            .render_form_data(true);
        assert!(
            draw(with_extras) > 1000,
            "the fixture draws nothing with the flags on, so the test proves nothing"
        );
    }

    /// No action in a document fires, even with no sandbox at all: the
    /// corpus's JavaScript, launch, URI, submit-form, remote-file, embedded
    /// file and XFA samples are opened and drawn here, in this unconfined
    /// test process, through the worker's own `open` and `render` (so with
    /// the form-fill environment `pdfium-render` makes on every load). A
    /// live server and a marker file are what the actions name; neither is
    /// touched. `tests/hostile.rs` checks the same through the contained
    /// worker, where the sandbox would stop them anyway; this is what shows
    /// PDFium itself never runs them: no action entry point is called, every
    /// form-fill callback is null, and the build has no script engine.
    #[test]
    fn no_action_fires_even_without_the_sandbox() {
        use crate::hostile::{Canary, Expect, corpus};
        use std::net::TcpListener;

        let (_one_at_a_time, pdfium) = pdfium();
        let dir = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let canary = Canary {
            url: format!("http://{}/kanso-canary", listener.local_addr().unwrap()),
            marker: dir.path().join("launched"),
        };
        let names = [
            "javascript-actions",
            "javascript-form-fields",
            "launch-action",
            "uri-action",
            "submit-form-action",
            "remote-file-actions",
            "embedded-file",
            "xfa-form",
        ];
        let samples: Vec<_> = corpus(&canary)
            .into_iter()
            .filter(|s| names.contains(&s.name))
            .collect();
        assert_eq!(samples.len(), names.len(), "an action sample went missing");
        for sample in samples {
            assert_eq!(sample.expect, Expect::Opens, "{}", sample.name);
            let Ok((Response::Opened { pages, .. }, doc)) = open(pdfium, sample.bytes, None) else {
                panic!("{} did not open, so it tests nothing", sample.name);
            };
            for page in 0..pages {
                let (response, _) = render(&doc, page, 612, false);
                assert!(
                    matches!(response, Response::Page { .. }),
                    "{}: {response:?}",
                    sample.name
                );
            }
            drop(doc);
        }
        // Anything an action started would have had time to land.
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(
            listener.accept().is_err(),
            "an action reached the canary server"
        );
        assert!(!canary.marker.exists(), "an action created the canary file");
    }

    #[test]
    fn options_are_parsed_and_unknown_ones_refused() {
        let args = |v: &[&str]| parse_args(v.iter().map(|s| (*s).to_owned()));
        assert_eq!(args(&[]).unwrap(), Args::default());
        assert_eq!(
            args(&["--library", "/x/libpdfium.dylib"]).unwrap().library,
            Some("/x/libpdfium.dylib".into())
        );
        assert!(args(&["--library"]).is_err());
        assert!(args(&["--open", "/etc/passwd"]).is_err());
        assert_eq!(
            args(&["--containment-probe", "/f", "127.0.0.1:1"])
                .unwrap()
                .probe,
            Some(("/f".into(), "127.0.0.1:1".into()))
        );
    }
}
