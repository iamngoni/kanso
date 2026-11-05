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
use crate::protocol::{
    BYTES_PER_PIXEL, ErrorKind, FrameError, MAX_BLOB_BYTES, MAX_PAGES, Request, Response,
    check_document, page_is_sound, read_frame, render_size, write_frame,
};

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
        let (response, blob, done) = match request {
            Request::Open { password } => {
                let (response, blob) = if document.is_some() {
                    failure(ErrorKind::Protocol)
                } else {
                    match open(pdfium, blob, password.as_deref()) {
                        Ok((opened, doc)) => {
                            document = Some(doc);
                            (opened, Vec::new())
                        }
                        Err(failed) => failed,
                    }
                };
                (response, blob, false)
            }
            Request::Render { page, width_px } => {
                let (response, blob) = match &document {
                    Some(doc) => render(doc, page, width_px),
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
        if contain::resident_bytes().is_some_and(|bytes| bytes > MEMORY_BUDGET) {
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

fn render(doc: &PdfDocument<'_>, index: u32, width_px: u32) -> (Response, Vec<u8>) {
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
    let Ok(bitmap) = page.render_with_config(&render_config(width, height)) else {
        return failure(ErrorKind::Internal);
    };
    let (w, h) = (bitmap.width() as u32, bitmap.height() as u32);
    let bytes = bitmap.as_raw_bytes();
    if !page_is_sound(w, h, bytes.len()) || bytes.len() != w as usize * h as usize * BYTES_PER_PIXEL
    {
        return failure(ErrorKind::Internal);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::annotated_pdf;

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
        let library =
            crate::client::dev_library().expect("PDFium is missing: run scripts/fetch-pdfium.sh");
        let pdfium = Pdfium::new(Pdfium::bind_to_library(library).expect("PDFium loads"));
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
