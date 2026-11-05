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
use crate::model::{Color, PdfBlock, Quad};
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
            Request::Export => {
                let (response, blob) = match document.as_mut() {
                    Some(doc) if blob.len() <= crate::protocol::MAX_EXPORT_ANNOTATION_BYTES => {
                        match serde_json::from_slice::<PdfBlock>(&blob)
                            .map_err(|_| ())
                            .and_then(|block| export_annotated(doc, &block))
                        {
                            Ok(bytes) => (Response::Exported, bytes),
                            Err(()) => failure(ErrorKind::Internal),
                        }
                    }
                    Some(_) => failure(ErrorKind::Protocol),
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

/// Adds portable markup and sticky-note comments to the in-memory document.
/// PDFium owns the entire write; the attachment bytes and their caller remain
/// untouched. The worker's process and request limits apply to this operation.
fn export_annotated(doc: &mut PdfDocument<'_>, block: &PdfBlock) -> Result<Vec<u8>, ()> {
    use pdfium_render::prelude::{PdfColor, PdfPageAnnotationCommon, PdfQuadPoints, PdfRect};

    // Reuse the block parser's caps and geometry checks at the worker
    // boundary; a malformed bridge request must fail before touching pages.
    let encoded = serde_json::to_string(block).map_err(|_| ())?;
    let block = PdfBlock::parse(&encoded).map_err(|_| ())?;
    let page_count = doc.pages().len() as u32;
    if block
        .highlights
        .iter()
        .flat_map(|highlight| highlight.quads.keys())
        .any(|page| *page == 0 || *page > page_count)
        || block.ink.iter().any(|page| page.validate().is_err())
    {
        return Err(());
    }

    for highlight in &block.highlights {
        for (page_number, quads) in &highlight.quads {
            let page_index =
                i32::try_from(page_number.checked_sub(1).ok_or(())?).map_err(|_| ())?;
            let mut page = doc.pages_mut().get(page_index).map_err(|_| ())?;
            let crop = page
                .boundaries()
                .crop()
                .or_else(|_| page.boundaries().media())
                .map_err(|_| ())?
                .bounds;
            let page_box = [
                f64::from(crop.left().value),
                f64::from(crop.bottom().value),
                f64::from(crop.right().value),
                f64::from(crop.top().value),
            ];
            let turns = page_turns(&page);
            let color = match highlight.color {
                Color::Yellow => PdfColor::new(255, 221, 64, 110),
                Color::Green => PdfColor::new(93, 204, 122, 110),
                Color::Blue => PdfColor::new(87, 160, 255, 110),
                Color::Pink => PdfColor::new(255, 113, 176, 110),
            };
            let annotations = page.annotations_mut();
            if !quads.is_empty() {
                let mut annotation_bounds: Option<PdfRect> = None;
                let mut annotation = annotations.create_highlight_annotation().map_err(|_| ())?;
                annotation.set_stroke_color(color).map_err(|_| ())?;
                annotation.set_fill_color(color).map_err(|_| ())?;
                for quad in quads {
                    let bounds = displayed_quad_to_pdf(*quad, page_box, turns);
                    annotation_bounds = Some(match annotation_bounds {
                        Some(existing) => PdfRect::new_from_values(
                            existing.bottom().value.min(bounds.bottom().value),
                            existing.left().value.min(bounds.left().value),
                            existing.top().value.max(bounds.top().value),
                            existing.right().value.max(bounds.right().value),
                        ),
                        None => bounds,
                    });
                    annotation
                        .attachment_points_mut()
                        .create_attachment_point_at_end(PdfQuadPoints::from_rect(&bounds))
                        .map_err(|_| ())?;
                }
                annotation
                    .set_bounds(annotation_bounds.ok_or(())?)
                    .map_err(|_| ())?;
                if let Some(comment) = highlight.comment.as_deref() {
                    annotation.set_contents(comment).map_err(|_| ())?;
                }
            }
            if let Some(comment) = highlight
                .comment
                .as_deref()
                .filter(|_| *page_number == highlight.page)
            {
                // A Text annotation stores the full comment as a standard PDF
                // note as well as on the markup. A small in-page icon has a
                // valid appearance in PDFium, Preview, Acrobat and browsers.
                let quad = quads
                    .first()
                    .copied()
                    .unwrap_or(Quad([0.04, 0.04, 0.08, 0.08]));
                let bounds = displayed_quad_to_pdf(quad, page_box, turns);
                let mut note = annotations
                    .create_text_annotation(comment)
                    .map_err(|_| ())?;
                note.set_stroke_color(color).map_err(|_| ())?;
                note.set_bounds(bounds).map_err(|_| ())?;
            }
        }
    }

    let mut bytes = doc.save_to_bytes().map_err(|_| ())?;
    if block.ink.iter().any(|page| page.live_count() != 0) {
        append_ink_annotations(&mut bytes, &block.ink, doc).map_err(|_| ())?;
    }
    if !block.highlights.is_empty() {
        append_highlight_appearances(&mut bytes).map_err(|_| ())?;
    }
    if block
        .highlights
        .iter()
        .any(|highlight| highlight.comment.is_some())
    {
        append_comment_popups(&mut bytes).map_err(|_| ())?;
    }
    Ok(bytes)
}

/// Maps a top-left normalized point in the displayed page box to PDF user
/// space, undoing the page's intrinsic rotation and crop-box offset.
fn displayed_to_pdf(x: f64, y: f64, page_box: [f64; 4], turns: u8) -> (f64, f64) {
    let (left, bottom, right, top) = (page_box[0], page_box[1], page_box[2], page_box[3]);
    let (u, v) = match turns % 4 {
        0 => (x, y),
        1 => (y, 1.0 - x),
        2 => (1.0 - x, 1.0 - y),
        _ => (1.0 - y, x),
    };
    (
        left + u * (right - left),
        bottom + (1.0 - v) * (top - bottom),
    )
}

/// Converts a normalized top-left quad to its PDFium page-space rectangle.
fn displayed_quad_to_pdf(quad: Quad, page_box: [f64; 4], turns: u8) -> PdfRect {
    let [x0, y0, x1, y1] = quad.0;
    let points = [
        displayed_to_pdf(x0, y0, page_box, turns),
        displayed_to_pdf(x1, y0, page_box, turns),
        displayed_to_pdf(x1, y1, page_box, turns),
        displayed_to_pdf(x0, y1, page_box, turns),
    ];
    let left = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let bottom = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let right = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let top = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    PdfRect::new_from_values(bottom as f32, left as f32, top as f32, right as f32)
}

/// Writes standard Ink annotations and appearance streams into PDFium's saved
/// copy. lopdf is confined to this worker and is given bounded decompression;
/// it never sees a path or the original library attachment.
fn append_ink_annotations(
    bytes: &mut Vec<u8>,
    pages: &[crate::ink::PageInk],
    source: &PdfDocument<'_>,
) -> Result<(), ()> {
    use lopdf::{Dictionary, Document, LoadOptions, Object, Stream, StringFormat};

    const MAX_DECOMPRESSED_BYTES: usize = 64 * 1024 * 1024;
    let mut pdf = Document::load_mem_with_options(
        bytes,
        LoadOptions::with_max_decompressed_size(MAX_DECOMPRESSED_BYTES),
    )
    .map_err(|_| ())?;
    let page_ids = pdf.get_pages();

    for page_ink in pages {
        let page_id = *page_ids.get(&page_ink.page).ok_or(())?;
        let page_index = i32::try_from(page_ink.page.checked_sub(1).ok_or(())?).map_err(|_| ())?;
        let page = source.pages().get(page_index).map_err(|_| ())?;
        let crop = page
            .boundaries()
            .crop()
            .or_else(|_| page.boundaries().media())
            .map_err(|_| ())?
            .bounds;
        let page_box = [
            f64::from(crop.left().value),
            f64::from(crop.bottom().value),
            f64::from(crop.right().value),
            f64::from(crop.top().value),
        ];
        let turns = page_turns(&page);
        let page_width = page_box[2] - page_box[0];
        if !page_width.is_finite() || page_width <= 0.0 {
            return Err(());
        }

        for stroke in page_ink.live().filter(|stroke| !stroke.points.is_empty()) {
            let [red, green, blue] = stroke_color(&stroke.color).ok_or(())?;
            let points = stroke
                .points
                .iter()
                .map(|point| {
                    displayed_to_pdf(
                        point.x / crate::ink::PAGE_UNITS,
                        point.y / crate::ink::PAGE_UNITS,
                        page_box,
                        turns,
                    )
                })
                .collect::<Vec<_>>();
            let base_width = (stroke.width / crate::ink::PAGE_UNITS * page_width).max(0.5);
            let bounds = ink_bounds(&points, base_width);
            let ink_points = points
                .iter()
                .flat_map(|(x, y)| [Object::Real(*x as f32), Object::Real(*y as f32)])
                .collect::<Vec<_>>();

            let mut appearance = format!("q\n{red:.6} {green:.6} {blue:.6} RG\n1 J 1 j\n");
            if points.len() == 1 {
                let (x, y) = points[0];
                let width = ink_width(base_width, stroke.points[0].pressure);
                appearance.push_str(&format!(
                    "{width:.4} w\n{x:.4} {y:.4} m {x:.4} {y:.4} l S\n"
                ));
            } else {
                for (index, pair) in points.windows(2).enumerate() {
                    let pressure =
                        (stroke.points[index].pressure + stroke.points[index + 1].pressure) / 2.0;
                    let width = ink_width(base_width, pressure);
                    appearance.push_str(&format!(
                        "{width:.4} w\n{:.4} {:.4} m {:.4} {:.4} l S\n",
                        pair[0].0, pair[0].1, pair[1].0, pair[1].1
                    ));
                }
            }
            appearance.push_str("Q\n");

            let mut appearance_dict = Dictionary::new();
            appearance_dict.set("Type", Object::Name(b"XObject".to_vec()));
            appearance_dict.set("Subtype", Object::Name(b"Form".to_vec()));
            appearance_dict.set("FormType", Object::Integer(1));
            appearance_dict.set("BBox", rect_array(bounds));
            let appearance_id =
                pdf.add_object(Stream::new(appearance_dict, appearance.into_bytes()));

            let mut border = Dictionary::new();
            border.set("Type", Object::Name(b"Border".to_vec()));
            border.set("W", Object::Real(base_width as f32));
            border.set("S", Object::Name(b"S".to_vec()));
            let mut appearance_state = Dictionary::new();
            appearance_state.set("N", Object::Reference(appearance_id));

            let mut annotation = Dictionary::new();
            annotation.set("Type", Object::Name(b"Annot".to_vec()));
            annotation.set("Subtype", Object::Name(b"Ink".to_vec()));
            annotation.set("Rect", rect_array(bounds));
            annotation.set("InkList", Object::Array(vec![Object::Array(ink_points)]));
            annotation.set(
                "C",
                Object::Array(vec![
                    Object::Real(red),
                    Object::Real(green),
                    Object::Real(blue),
                ]),
            );
            annotation.set("BS", Object::Dictionary(border));
            annotation.set("AP", Object::Dictionary(appearance_state));
            annotation.set("F", Object::Integer(4));
            annotation.set(
                "NM",
                Object::String(stroke.id.as_bytes().to_vec(), StringFormat::Literal),
            );
            let annotation_id = pdf.add_object(Object::Dictionary(annotation));
            append_page_annotation(&mut pdf, page_id, annotation_id)?;
        }
    }

    let mut written = Vec::new();
    pdf.save_to(&mut written).map_err(|_| ())?;
    if written.len() > crate::protocol::MAX_BLOB_BYTES {
        return Err(());
    }
    *bytes = written;
    Ok(())
}

/// Links each Text comment to a standard Popup annotation and ensures its
/// sticky-note icon has an appearance stream for viewers that do not synthesize one.
fn append_comment_popups(bytes: &mut Vec<u8>) -> Result<(), ()> {
    use lopdf::{Dictionary, Document, LoadOptions, Object, Stream};

    const MAX_DECOMPRESSED_BYTES: usize = 64 * 1024 * 1024;
    let mut pdf = Document::load_mem_with_options(
        bytes,
        LoadOptions::with_max_decompressed_size(MAX_DECOMPRESSED_BYTES),
    )
    .map_err(|_| ())?;
    let pages = pdf.get_pages();
    for page_id in pages.values() {
        let annotations = pdf
            .get_object(*page_id)
            .map_err(|_| ())?
            .as_dict()
            .map_err(|_| ())?
            .get(b"Annots")
            .ok()
            .cloned();
        let Some(annotations) = annotations else {
            continue;
        };
        let annotation_ids = match &annotations {
            Object::Array(entries) => entries.clone(),
            Object::Reference(id) => pdf
                .get_object(*id)
                .map_err(|_| ())?
                .as_array()
                .map_err(|_| ())?
                .clone(),
            _ => return Err(()),
        };
        for (index, entry) in annotation_ids.into_iter().enumerate() {
            let text_id = match entry {
                Object::Reference(id) => id,
                Object::Dictionary(dictionary)
                    if dictionary
                        .get(b"Subtype")
                        .ok()
                        .and_then(|v| v.as_name().ok())
                        == Some(b"Text") =>
                {
                    // PDFium can emit a direct annotation dictionary. Make it
                    // indirect so Popup.Parent can refer to it as the PDF
                    // specification requires.
                    let id = pdf.add_object(Object::Dictionary(dictionary));
                    page_annots_mut(&mut pdf, *page_id, &annotations)?[index] =
                        Object::Reference(id);
                    id
                }
                _ => continue,
            };
            let (rect, comment_color, has_comment, has_popup) = {
                let text = pdf
                    .get_object(text_id)
                    .map_err(|_| ())?
                    .as_dict()
                    .map_err(|_| ())?;
                let is_text =
                    text.get(b"Subtype").ok().and_then(|v| v.as_name().ok()) == Some(b"Text");
                if !is_text {
                    continue;
                }
                (
                    text.get(b"Rect").map_err(|_| ())?.clone(),
                    text.get(b"C").ok().cloned(),
                    text.get(b"Contents").is_ok(),
                    text.get(b"Popup").is_ok(),
                )
            };
            if !has_comment {
                continue;
            }
            let Object::Array(rect_values) = &rect else {
                return Err(());
            };
            if rect_values.len() != 4 {
                return Err(());
            }
            let (left, bottom, right, top) = (
                rect_values[0].as_float().map_err(|_| ())?,
                rect_values[1].as_float().map_err(|_| ())?,
                rect_values[2].as_float().map_err(|_| ())?,
                rect_values[3].as_float().map_err(|_| ())?,
            );
            if !has_popup {
                let mut popup = Dictionary::new();
                popup.set("Type", Object::Name(b"Annot".to_vec()));
                popup.set("Subtype", Object::Name(b"Popup".to_vec()));
                popup.set("Parent", Object::Reference(text_id));
                popup.set(
                    "Rect",
                    Object::Array(vec![
                        Object::Real(left),
                        Object::Real(bottom),
                        Object::Real(right + 144.0),
                        Object::Real(top + 72.0),
                    ]),
                );
                popup.set("F", Object::Integer(4));
                let popup_id = pdf.add_object(Object::Dictionary(popup));
                pdf.get_object_mut(text_id)
                    .map_err(|_| ())?
                    .as_dict_mut()
                    .map_err(|_| ())?
                    .set("Popup", Object::Reference(popup_id));
                page_annots_mut(&mut pdf, *page_id, &annotations)?
                    .push(Object::Reference(popup_id));
            }
            let text = pdf
                .get_object(text_id)
                .map_err(|_| ())?
                .as_dict()
                .map_err(|_| ())?;
            if text.get(b"AP").is_err() {
                let width = (right - left).max(1.0);
                let height = (top - bottom).max(1.0);
                let (red, green, blue) = comment_color
                    .as_ref()
                    .and_then(|value| value.as_array().ok())
                    .filter(|values| values.len() >= 3)
                    .and_then(|values| {
                        Some((
                            values[0].as_float().ok()?,
                            values[1].as_float().ok()?,
                            values[2].as_float().ok()?,
                        ))
                    })
                    .unwrap_or((1.0, 0.83, 0.2));
                let stream = Stream::new(
                    Dictionary::from_iter([
                        (b"Type".to_vec(), Object::Name(b"XObject".to_vec())),
                        (b"Subtype".to_vec(), Object::Name(b"Form".to_vec())),
                        (b"FormType".to_vec(), Object::Integer(1)),
                        (
                            b"BBox".to_vec(),
                            Object::Array(vec![
                                Object::Real(0.0),
                                Object::Real(0.0),
                                Object::Real(width),
                                Object::Real(height),
                            ]),
                        ),
                    ]),
                    format!(
                        "q {red:.4} {green:.4} {blue:.4} rg 0 0 {width:.2} {height:.2} re f 0.2 0.2 0 RG 0.7 w 0.5 0.5 {w:.2} {h:.2} re S Q",
                        w = (width - 1.0).max(0.1),
                        h = (height - 1.0).max(0.1),
                    )
                    .into_bytes(),
                );
                let appearance_id = pdf.add_object(stream);
                let mut appearance = Dictionary::new();
                appearance.set("N", Object::Reference(appearance_id));
                pdf.get_object_mut(text_id)
                    .map_err(|_| ())?
                    .as_dict_mut()
                    .map_err(|_| ())?
                    .set("AP", Object::Dictionary(appearance));
            }
        }
    }
    let mut output = Vec::new();
    pdf.save_to(&mut output).map_err(|_| ())?;
    if output.len() > crate::protocol::MAX_BLOB_BYTES {
        return Err(());
    }
    *bytes = output;
    Ok(())
}

/// Gives PDFium's Highlight markup a translucent Form appearance. Some viewers
/// synthesize highlights from `/C` and `/CA` while others prefer `/AP`; both
/// paths now show the same color without covering the underlying text.
fn append_highlight_appearances(bytes: &mut Vec<u8>) -> Result<(), ()> {
    use lopdf::{Dictionary, Document, LoadOptions, Object, Stream};

    const MAX_DECOMPRESSED_BYTES: usize = 64 * 1024 * 1024;
    let mut pdf = Document::load_mem_with_options(
        bytes,
        LoadOptions::with_max_decompressed_size(MAX_DECOMPRESSED_BYTES),
    )
    .map_err(|_| ())?;
    for page_id in pdf.get_pages().values().copied() {
        let annotations = pdf
            .get_object(page_id)
            .map_err(|_| ())?
            .as_dict()
            .map_err(|_| ())?
            .get(b"Annots")
            .ok()
            .cloned();
        let Some(annotations) = annotations else {
            continue;
        };
        let entries = match &annotations {
            Object::Array(entries) => entries.clone(),
            Object::Reference(id) => pdf
                .get_object(*id)
                .map_err(|_| ())?
                .as_array()
                .map_err(|_| ())?
                .clone(),
            _ => return Err(()),
        };
        for (index, entry) in entries.into_iter().enumerate() {
            let annotation_id = match entry {
                Object::Reference(id) => id,
                Object::Dictionary(dictionary)
                    if dictionary
                        .get(b"Subtype")
                        .ok()
                        .and_then(|value| value.as_name().ok())
                        == Some(b"Highlight") =>
                {
                    let id = pdf.add_object(Object::Dictionary(dictionary));
                    page_annots_mut(&mut pdf, page_id, &annotations)?[index] =
                        Object::Reference(id);
                    id
                }
                _ => continue,
            };
            let (rect, quad_points, color, alpha) = {
                let highlight = pdf
                    .get_object(annotation_id)
                    .map_err(|_| ())?
                    .as_dict()
                    .map_err(|_| ())?;
                if highlight
                    .get(b"Subtype")
                    .ok()
                    .and_then(|value| value.as_name().ok())
                    != Some(b"Highlight")
                {
                    continue;
                }
                (
                    highlight.get(b"Rect").map_err(|_| ())?.clone(),
                    highlight.get(b"QuadPoints").map_err(|_| ())?.clone(),
                    highlight.get(b"C").ok().cloned(),
                    highlight
                        .get(b"CA")
                        .ok()
                        .and_then(|value| value.as_float().ok())
                        .unwrap_or(0.42),
                )
            };
            let rect_values = rect.as_array().map_err(|_| ())?;
            if rect_values.len() != 4 {
                return Err(());
            }
            let mut bounds = Vec::with_capacity(4);
            for value in rect_values {
                bounds.push(f64::from(value.as_float().map_err(|_| ())?));
            }
            let width = (bounds[2] - bounds[0]).max(0.1);
            let height = (bounds[3] - bounds[1]).max(0.1);
            let rgb = color
                .as_ref()
                .and_then(|value| value.as_array().ok())
                .filter(|values| values.len() >= 3)
                .and_then(|values| {
                    Some([
                        values[0].as_float().ok()?,
                        values[1].as_float().ok()?,
                        values[2].as_float().ok()?,
                    ])
                })
                .unwrap_or([1.0, 0.86, 0.25]);
            let points = quad_points.as_array().map_err(|_| ())?;
            if points.is_empty() || points.len() % 8 != 0 || points.len() > 32 * 8 {
                return Err(());
            }
            let opacity = alpha.clamp(0.15, 0.85);
            // Several common viewers (including Preview) ignore transparency
            // graphics state inside annotation appearances. Preblend the
            // annotation tint against white so these viewers still show a
            // light, readable highlight; the opaque appearance is identical
            // everywhere and keeps the text below visible.
            let tint = rgb.map(|channel| channel * opacity + (1.0 - opacity));
            // Keep `/C` in sync as well: some renderers disregard `/AP` and
            // `/CA`, then synthesize the highlight directly from `/C`.
            let highlight = pdf
                .get_object_mut(annotation_id)
                .map_err(|_| ())?
                .as_dict_mut()
                .map_err(|_| ())?;
            highlight.set(
                "C",
                Object::Array(tint.iter().map(|value| Object::Real(*value)).collect()),
            );
            highlight.set(
                "IC",
                Object::Array(tint.iter().map(|value| Object::Real(*value)).collect()),
            );
            highlight.set("CA", Object::Real(1.0));
            let mut content = format!(
                "q /GS1 gs {:.5} {:.5} {:.5} rg\n",
                tint[0], tint[1], tint[2]
            );
            for quad in points.as_chunks::<8>().0 {
                let mut points = Vec::with_capacity(8);
                for value in quad {
                    points.push(f64::from(value.as_float().map_err(|_| ())?));
                }
                let left = points
                    .iter()
                    .step_by(2)
                    .copied()
                    .fold(f64::INFINITY, f64::min);
                let bottom = points
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .copied()
                    .fold(f64::INFINITY, f64::min);
                let right = points
                    .iter()
                    .step_by(2)
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);
                let top = points
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);
                let local_left = left - bounds[0];
                let local_bottom = bottom - bounds[1];
                content.push_str(&format!(
                    "{local_left:.4} {local_bottom:.4} {:.4} {:.4} re f\n",
                    right - left,
                    top - bottom
                ));
            }
            content.push_str("Q\n");
            let mut opacity = Dictionary::new();
            opacity.set("Type", Object::Name(b"ExtGState".to_vec()));
            opacity.set("ca", Object::Real(1.0));
            opacity.set("CA", Object::Real(1.0));
            let opacity_id = pdf.add_object(Object::Dictionary(opacity));
            let mut states = Dictionary::new();
            states.set("GS1", Object::Reference(opacity_id));
            let mut resources = Dictionary::new();
            resources.set("ExtGState", Object::Dictionary(states));
            let appearance = Stream::new(
                Dictionary::from_iter([
                    (b"Type".to_vec(), Object::Name(b"XObject".to_vec())),
                    (b"Subtype".to_vec(), Object::Name(b"Form".to_vec())),
                    (b"FormType".to_vec(), Object::Integer(1)),
                    (
                        b"BBox".to_vec(),
                        Object::Array(vec![
                            Object::Real(0.0),
                            Object::Real(0.0),
                            Object::Real(width as f32),
                            Object::Real(height as f32),
                        ]),
                    ),
                    (b"Resources".to_vec(), Object::Dictionary(resources)),
                    // Transparency in a Form XObject is isolated unless the
                    // form declares a transparency group. Without this group
                    // Preview paints `/ca` as fully opaque even though some
                    // other renderers honor the alpha value.
                    (
                        b"Group".to_vec(),
                        Object::Dictionary(Dictionary::from_iter([
                            (b"Type".to_vec(), Object::Name(b"Group".to_vec())),
                            (b"S".to_vec(), Object::Name(b"Transparency".to_vec())),
                            (b"CS".to_vec(), Object::Name(b"DeviceRGB".to_vec())),
                        ])),
                    ),
                ]),
                content.into_bytes(),
            );
            let appearance_id = pdf.add_object(appearance);
            let mut appearance_state = Dictionary::new();
            appearance_state.set("N", Object::Reference(appearance_id));
            pdf.get_object_mut(annotation_id)
                .map_err(|_| ())?
                .as_dict_mut()
                .map_err(|_| ())?
                .set("AP", Object::Dictionary(appearance_state));
        }
    }
    let mut output = Vec::new();
    pdf.save_to(&mut output).map_err(|_| ())?;
    if output.len() > crate::protocol::MAX_BLOB_BYTES {
        return Err(());
    }
    *bytes = output;
    Ok(())
}

/// Returns a page's `/Annots` array whether the array is direct or indirect.
fn page_annots_mut<'a>(
    pdf: &'a mut lopdf::Document,
    page_id: lopdf::ObjectId,
    annotations: &lopdf::Object,
) -> Result<&'a mut Vec<lopdf::Object>, ()> {
    use lopdf::Object;

    match annotations {
        Object::Array(_) => pdf
            .get_object_mut(page_id)
            .map_err(|_| ())?
            .as_dict_mut()
            .map_err(|_| ())?
            .get_mut(b"Annots")
            .map_err(|_| ())?
            .as_array_mut()
            .map_err(|_| ()),
        Object::Reference(id) => pdf
            .get_object_mut(*id)
            .map_err(|_| ())?
            .as_array_mut()
            .map_err(|_| ()),
        _ => Err(()),
    }
}

/// Appends an annotation reference, supporting direct and indirect `/Annots`
/// arrays without replacing annotations the original PDF contained.
fn append_page_annotation(
    pdf: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    annotation_id: lopdf::ObjectId,
) -> Result<(), ()> {
    use lopdf::Object;

    let existing = pdf
        .get_object(page_id)
        .map_err(|_| ())?
        .as_dict()
        .map_err(|_| ())?
        .get(b"Annots")
        .ok()
        .cloned();
    match existing {
        Some(Object::Array(_)) => {
            pdf.get_object_mut(page_id)
                .map_err(|_| ())?
                .as_dict_mut()
                .map_err(|_| ())?
                .get_mut(b"Annots")
                .map_err(|_| ())?
                .as_array_mut()
                .map_err(|_| ())?
                .push(Object::Reference(annotation_id));
        }
        Some(Object::Reference(array_id)) => {
            pdf.get_object_mut(array_id)
                .map_err(|_| ())?
                .as_array_mut()
                .map_err(|_| ())?
                .push(Object::Reference(annotation_id));
        }
        None => {
            pdf.get_object_mut(page_id)
                .map_err(|_| ())?
                .as_dict_mut()
                .map_err(|_| ())?
                .set(
                    "Annots",
                    Object::Array(vec![Object::Reference(annotation_id)]),
                );
        }
        Some(_) => return Err(()),
    }
    Ok(())
}

fn rect_array(rect: PdfRect) -> lopdf::Object {
    lopdf::Object::Array(vec![
        lopdf::Object::Real(rect.left().value),
        lopdf::Object::Real(rect.bottom().value),
        lopdf::Object::Real(rect.right().value),
        lopdf::Object::Real(rect.top().value),
    ])
}

fn ink_bounds(points: &[(f64, f64)], width: f64) -> PdfRect {
    let padding = width.max(0.5) / 2.0;
    let left = points
        .iter()
        .map(|point| point.0)
        .fold(f64::INFINITY, f64::min)
        - padding;
    let bottom = points
        .iter()
        .map(|point| point.1)
        .fold(f64::INFINITY, f64::min)
        - padding;
    let right = points
        .iter()
        .map(|point| point.0)
        .fold(f64::NEG_INFINITY, f64::max)
        + padding;
    let top = points
        .iter()
        .map(|point| point.1)
        .fold(f64::NEG_INFINITY, f64::max)
        + padding;
    PdfRect::new_from_values(bottom as f32, left as f32, top as f32, right as f32)
}

fn ink_width(base: f64, pressure: f64) -> f64 {
    (base * pressure.clamp(0.1, 1.0)).max(0.25)
}

fn stroke_color(value: &str) -> Option<[f32; 3]> {
    let digits = value.strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let channel = |start| u8::from_str_radix(&digits[start..start + 2], 16).ok();
    Some([
        f32::from(channel(0)?) / 255.0,
        f32::from(channel(2)?) / 255.0,
        f32::from(channel(4)?) / 255.0,
    ])
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
    use pdfium_render::prelude::PdfPageAnnotationCommon;

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

    #[test]
    fn exported_highlight_and_comment_reopen_as_pdf_annotations_without_changing_source() {
        use crate::{Color, Highlight, Quad};
        use md5::{Digest, Md5};
        use std::collections::BTreeMap;

        let (_one_at_a_time, pdfium) = pdfium();
        let original = annotated_pdf();
        let before = Md5::digest(&original);
        let (opened, mut document) = open(pdfium, original.clone(), None).expect("fixture opens");
        assert!(matches!(opened, Response::Opened { pages: 1, .. }));
        let mut block = PdfBlock::new("file:fixture", "");
        block.highlights.push(Highlight {
            id: "roundtrip".into(),
            page: 1,
            quads: BTreeMap::from([(1, vec![Quad::new(0.1, 0.1, 0.4, 0.13)])]),
            text: "Annotated".into(),
            color: Color::Blue,
            comment: Some("Keep this comment".into()),
            created_at: 1,
            rev: 0,
            extra: Default::default(),
        });
        let output = export_annotated(&mut document, &block).expect("annotations export");
        if let Ok(path) = std::env::var("KANSO_PDF_EXPORT_SAMPLE") {
            std::fs::write(path, &output).expect("write requested viewer-check sample");
        }
        assert_eq!(before, Md5::digest(&original), "source MD5 stays unchanged");
        assert!(output.starts_with(b"%PDF-"));
        let reopened = pdfium
            .load_pdf_from_byte_vec(output.clone(), None)
            .expect("export reopens");
        let page = reopened.pages().get(0).expect("first page");
        let annotations = page.annotations();
        let mut highlight_quads = Vec::new();
        let mut comments = Vec::new();
        for annotation in annotations.iter() {
            match annotation {
                PdfPageAnnotation::Highlight(ref highlight) => {
                    highlight_quads.extend(highlight.attachment_points().iter());
                }
                PdfPageAnnotation::Text(ref text) => {
                    if let Some(contents) = text.contents() {
                        comments.push(contents);
                    }
                }
                _ => {}
            }
        }
        assert_eq!(highlight_quads.len(), 1, "QuadPoints survive reopen");
        let quad = &highlight_quads[0];
        assert!((f64::from(quad.left().value) / f64::from(page.width().value) - 0.1).abs() < 0.001);
        assert!(
            (1.0 - f64::from(quad.top().value) / f64::from(page.height().value) - 0.1).abs()
                < 0.001
        );
        assert!(
            comments
                .iter()
                .any(|contents| contents == "Keep this comment")
        );
        let pdf = lopdf::Document::load_mem_with_options(
            &output,
            lopdf::LoadOptions::with_max_decompressed_size(64 * 1024 * 1024),
        )
        .expect("lopdf reads exported comments");
        let page_id = pdf.get_pages()[&1];
        let page_dict = pdf.get_object(page_id).unwrap().as_dict().unwrap();
        let annots = page_dict
            .get_deref(b"Annots", &pdf)
            .unwrap()
            .as_array()
            .unwrap();
        let markup = annots
            .iter()
            .find_map(|entry| {
                let lopdf::Object::Reference(id) = entry else {
                    return None;
                };
                let dict = pdf.get_object(*id).ok()?.as_dict().ok()?;
                (dict.get(b"Subtype").ok()?.as_name().ok()? == b"Highlight").then_some(dict)
            })
            .expect("highlight markup exists");
        assert!(
            markup.get(b"AP").is_ok(),
            "highlight has a portable appearance"
        );
        let highlight_color = markup
            .get(b"C")
            .expect("highlight preserves its color")
            .as_array()
            .expect("highlight color components");
        assert!((highlight_color[0].as_float().unwrap() - 0.716).abs() < 0.01);
        assert!((highlight_color[1].as_float().unwrap() - 0.839).abs() < 0.01);
        assert!((highlight_color[2].as_float().unwrap() - 1.0).abs() < 0.01);
        let highlight_interior_color = markup
            .get(b"IC")
            .expect("readable highlight color works without appearance support")
            .as_array()
            .expect("highlight interior color components");
        assert_eq!(highlight_color, highlight_interior_color);
        let text = annots
            .iter()
            .find_map(|entry| {
                let lopdf::Object::Reference(id) = entry else {
                    return None;
                };
                let dict = pdf.get_object(*id).ok()?.as_dict().ok()?;
                (dict.get(b"Subtype").ok()?.as_name().ok()? == b"Text").then_some(dict)
            })
            .expect("Text comment exists");
        assert!(text.get(b"Popup").is_ok(), "comment has a standard popup");
        assert!(
            text.get(b"AP").is_ok(),
            "comment icon has an appearance stream"
        );
        let color = text
            .get(b"C")
            .expect("comment preserves its color")
            .as_array()
            .expect("comment color components");
        assert!((color[0].as_float().unwrap() - 87.0 / 255.0).abs() < 0.01);
        assert!((color[1].as_float().unwrap() - 160.0 / 255.0).abs() < 0.01);
        assert!((color[2].as_float().unwrap() - 1.0).abs() < 0.01);
        assert!(
            annots.iter().any(|entry| {
                let lopdf::Object::Reference(id) = entry else {
                    return false;
                };
                pdf.get_object(*id)
                    .ok()
                    .and_then(|object| object.as_dict().ok())
                    .and_then(|dict| dict.get(b"Subtype").ok())
                    .and_then(|value| value.as_name().ok())
                    == Some(b"Popup")
            }),
            "popup annotation is also attached to the page"
        );
        let rendered = page
            .render_with_config(&render_config(612, 792).render_annotations(true))
            .expect("exported annotations render");
        assert!(
            coloured_pixels(&rendered.as_raw_bytes()) > 100,
            "the exported marks are not visible when PDFium renders annotations"
        );
    }

    #[test]
    fn a_cross_page_highlight_is_written_on_each_named_page() {
        use crate::{Color, Highlight, Quad};
        use std::collections::BTreeMap;

        let (_one_at_a_time, pdfium) = pdfium();
        let original = crate::fixtures::report_pdf();
        let (_, mut document) = open(pdfium, original, None).expect("report opens");
        assert!(document.pages().len() >= 2, "fixture has multiple pages");
        let mut block = PdfBlock::new("file:report", "");
        block.highlights.push(Highlight {
            id: "crosspage".into(),
            page: 1,
            quads: BTreeMap::from([
                (1, vec![Quad::new(0.1, 0.1, 0.3, 0.12)]),
                (2, vec![Quad::new(0.2, 0.2, 0.4, 0.22)]),
            ]),
            text: "selection across pages".into(),
            color: Color::Green,
            comment: Some("one comment".into()),
            created_at: 1,
            rev: 0,
            extra: Default::default(),
        });
        let output = export_annotated(&mut document, &block).expect("cross-page export");
        let reopened = pdfium
            .load_pdf_from_byte_vec(output, None)
            .expect("reopen copy");
        for index in [0, 1] {
            let page = reopened.pages().get(index).expect("page");
            assert_eq!(
                page.annotations()
                    .iter()
                    .filter(|annotation| annotation.annotation_type()
                        == PdfPageAnnotationType::Highlight)
                    .count(),
                1,
                "highlight remains on page {}",
                index + 1
            );
        }
    }

    #[test]
    fn an_ocr_word_quad_exports_on_a_page_without_a_text_layer() {
        use crate::{Color, Highlight, Quad};
        use std::collections::BTreeMap;

        let (_one_at_a_time, pdfium) = pdfium();
        let mut document = pdfium
            .load_pdf_from_byte_vec(crate::fixtures::pages_pdf(&[String::new()]), None)
            .expect("scanned-page fixture opens");
        let mut block = PdfBlock::new("file:scan", "");
        // OCR word boxes use the same normalized top-left geometry as PDFium
        // character boxes. This fixture has no text layer, as on a scan.
        block.highlights.push(Highlight {
            id: "ocrword1".into(),
            page: 1,
            quads: BTreeMap::from([(1, vec![Quad::new(0.2, 0.35, 0.44, 0.39)])]),
            text: "recognized word".into(),
            color: Color::Yellow,
            comment: None,
            created_at: 1,
            rev: 0,
            extra: Default::default(),
        });
        let output = export_annotated(&mut document, &block).expect("OCR highlight exports");
        let reopened = pdfium
            .load_pdf_from_byte_vec(output, None)
            .expect("export reopens");
        let page = reopened.pages().get(0).expect("page one");
        let quad = page
            .annotations()
            .iter()
            .find_map(|annotation| match annotation {
                PdfPageAnnotation::Highlight(ref highlight) => {
                    highlight.attachment_points().iter().next()
                }
                _ => None,
            })
            .expect("OCR-page highlight survives");
        assert!((f64::from(quad.left().value) / f64::from(page.width().value) - 0.2).abs() < 0.001);
        assert!(
            (1.0 - f64::from(quad.top().value) / f64::from(page.height().value) - 0.35).abs()
                < 0.001
        );
    }

    #[test]
    fn exported_ink_has_an_ink_list_and_an_appearance_after_reopen() {
        use kanso_core::canvas::{Point, Stroke};
        use lopdf::{Document, LoadOptions, Object};
        use md5::Digest;

        let (_one_at_a_time, pdfium) = pdfium();
        let original = annotated_pdf();
        let before = md5::Md5::digest(&original);
        let (_, mut document) = open(pdfium, original.clone(), None).expect("fixture opens");
        let mut block = PdfBlock::new("file:fixture", "");
        let mut page_ink = crate::ink::PageInk::new(1);
        page_ink.strokes.push(Stroke {
            id: "stroke1".into(),
            color: "#e03131".into(),
            width: 10.0,
            points: vec![
                Point::new(100.0, 200.0, 0.25),
                Point::new(200.0, 300.0, 0.9),
            ],
            ..Stroke::default()
        });
        block.ink.push(page_ink);
        let output = export_annotated(&mut document, &block).expect("ink exports");
        assert_eq!(
            before,
            md5::Md5::digest(&original),
            "source MD5 stays unchanged"
        );

        let reopened = pdfium
            .load_pdf_from_byte_vec(output.clone(), None)
            .expect("export reopens through PDFium");
        let types = reopened
            .pages()
            .get(0)
            .expect("page")
            .annotations()
            .iter()
            .map(|annotation| annotation.annotation_type())
            .collect::<Vec<_>>();
        assert!(types.contains(&PdfPageAnnotationType::Ink));

        let pdf = Document::load_mem_with_options(
            &output,
            LoadOptions::with_max_decompressed_size(64 * 1024 * 1024),
        )
        .expect("lopdf reads exported copy");
        let page_id = pdf.get_pages()[&1];
        let page = pdf
            .get_object(page_id)
            .expect("page object")
            .as_dict()
            .unwrap();
        let annots = page.get_deref(b"Annots", &pdf).unwrap().as_array().unwrap();
        let ink_id = annots
            .iter()
            .find_map(|object| {
                let Object::Reference(id) = object else {
                    return None;
                };
                let annotation = pdf.get_object(*id).ok()?.as_dict().ok()?;
                (annotation.get(b"Subtype").ok()?.as_name().ok()? == b"Ink").then_some(*id)
            })
            .expect("Ink annotation exists");
        let ink = pdf.get_object(ink_id).unwrap().as_dict().unwrap();
        let ink_list = ink.get(b"InkList").unwrap().as_array().unwrap();
        let coordinates = ink_list[0].as_array().unwrap();
        assert_eq!(coordinates.len(), 4);
        let first_x = f64::from(coordinates[0].as_float().unwrap());
        let first_y = f64::from(coordinates[1].as_float().unwrap());
        assert!((first_x - 61.2).abs() < 0.1);
        assert!((first_y - 633.6).abs() < 0.1);
        assert!(
            ink.get_deref(b"AP", &pdf).is_ok(),
            "appearance stream is attached"
        );
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

        /// PDFium can reopen exported markup and Ink with the same page and
        /// geometry; source bytes retain the same digest for every generated
        /// annotation.
        #[test]
        fn exported_annotation_geometry_round_trips(
            x in 0.1f64..0.65, y in 0.1f64..0.65,
            width in 0.05f64..0.2, height in 0.02f64..0.08,
            ink_x in 0.1f64..0.8, ink_y in 0.1f64..0.8,
        ) {
            use crate::{Color, Highlight, Quad};
            use kanso_core::canvas::{Point, Stroke};
            use std::collections::BTreeMap;
            use md5::Digest;

            let (_one_at_a_time, pdfium) = pdfium();
            let original = annotated_pdf();
            let before = md5::Md5::digest(&original);
            let (_, mut document) = open(pdfium, original.clone(), None).expect("fixture opens");
            let mut block = PdfBlock::new("file:fixture", "");
            block.highlights.push(Highlight {
                id: "property1".into(), page: 1,
                quads: BTreeMap::from([(1, vec![Quad::new(x, y, x + width, y + height)])]),
                text: "property text".into(), color: Color::Pink,
                comment: Some("property comment".into()), created_at: 1, rev: 0,
                extra: Default::default(),
            });
            let mut ink = crate::ink::PageInk::new(1);
            ink.strokes.push(Stroke {
                id: "property1".into(), color: "#e03131".into(), width: 8.0,
                points: vec![
                    Point::new(ink_x * crate::ink::PAGE_UNITS, ink_y * crate::ink::PAGE_UNITS, 0.5),
                    Point::new((ink_x + 0.05) * crate::ink::PAGE_UNITS,
                        (ink_y + 0.04) * crate::ink::PAGE_UNITS, 0.7),
                ],
                ..Stroke::default()
            });
            block.ink.push(ink);
            let output = export_annotated(&mut document, &block).expect("export succeeds");
            proptest::prop_assert_eq!(before, md5::Md5::digest(&original));
            let reopened = pdfium.load_pdf_from_byte_vec(output.clone(), None).expect("reopen");
            let page = reopened.pages().get(0).expect("page one");
            let quad = page
                .annotations()
                .iter()
                .find_map(|annotation| match annotation {
                    PdfPageAnnotation::Highlight(ref mark) => {
                        mark.attachment_points().iter().next()
                    }
                    _ => None,
                })
                .expect("highlight quad survives");
            proptest::prop_assert!((f64::from(quad.left().value) / f64::from(page.width().value) - x).abs() < 0.001);
            proptest::prop_assert!((1.0 - f64::from(quad.top().value) / f64::from(page.height().value) - y).abs() < 0.001);

            let pdf = lopdf::Document::load_mem_with_options(
                &output, lopdf::LoadOptions::with_max_decompressed_size(64 * 1024 * 1024),
            ).expect("read InkList");
            let page_id = pdf.get_pages()[&1];
            let page_dict = pdf.get_object(page_id).expect("page object").as_dict().expect("page dictionary");
            let annots = page_dict.get_deref(b"Annots", &pdf).expect("annotations").as_array().expect("annotation array");
            let ink_id = annots.iter().find_map(|entry| {
                let lopdf::Object::Reference(id) = entry else { return None };
                let dict = pdf.get_object(*id).ok()?.as_dict().ok()?;
                (dict.get(b"Subtype").ok()?.as_name().ok()? == b"Ink").then_some(*id)
            }).expect("Ink annotation survives");
            let dict = pdf.get_object(ink_id).expect("Ink object").as_dict().expect("Ink dictionary");
            let points = dict.get(b"InkList").expect("InkList").as_array().expect("stroke array")[0]
                .as_array().expect("points");
            let expected_x = ink_x * 612.0;
            let expected_y = (1.0 - ink_y) * 792.0;
            proptest::prop_assert!((f64::from(points[0].as_float().expect("x")) - expected_x).abs() < 0.2);
            proptest::prop_assert!((f64::from(points[1].as_float().expect("y")) - expected_y).abs() < 0.2);
            proptest::prop_assert!((f64::from(points[2].as_float().expect("second x")) - (ink_x + 0.05) * 612.0).abs() < 0.2);
            proptest::prop_assert!((f64::from(points[3].as_float().expect("second y")) - (1.0 - ink_y - 0.04) * 792.0).abs() < 0.2);
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

    /// Writes actual PDFium rasters of an exported annotated page in both
    /// Kanso reader themes, for visual inspection alongside the round-trip
    /// assertions above.
    #[test]
    #[ignore = "writes PNGs; run with KANSO_SNAPSHOT_DIR set"]
    fn exported_page_snapshots_light_and_dark() {
        use crate::{Color, Highlight, Quad};
        use kanso_core::canvas::{Point, Stroke};
        use std::collections::BTreeMap;

        let out = std::path::PathBuf::from(
            std::env::var("KANSO_SNAPSHOT_DIR").expect("set KANSO_SNAPSHOT_DIR"),
        );
        std::fs::create_dir_all(&out).expect("snapshot folder");
        let (_one_at_a_time, pdfium) = pdfium();
        let mut document = pdfium
            .load_pdf_from_byte_vec(crate::fixtures::report_pdf(), None)
            .expect("report fixture opens");
        let mut block = PdfBlock::new("file:report", "");
        block.highlights.push(Highlight {
            id: "snapshot1".into(),
            page: 1,
            quads: BTreeMap::from([(1, vec![Quad::new(0.12, 0.12, 0.62, 0.16)])]),
            text: "Quarterly report".into(),
            color: Color::Blue,
            comment: Some("Check these figures.".into()),
            created_at: 1,
            rev: 0,
            extra: Default::default(),
        });
        let mut ink = crate::ink::PageInk::new(1);
        ink.strokes.push(Stroke {
            id: "snapshot1".into(),
            color: "#e03131".into(),
            width: 8.0,
            points: vec![Point::new(180.0, 390.0, 0.5), Point::new(780.0, 430.0, 0.9)],
            ..Stroke::default()
        });
        block.ink.push(ink);
        let output = export_annotated(&mut document, &block).expect("export annotations");
        std::fs::write(out.join("pdf-exported-copy.pdf"), &output)
            .expect("save exported copy for viewer checks");
        let exported = pdfium
            .load_pdf_from_byte_vec(output, None)
            .expect("export reopens");
        let page = exported.pages().get(0).expect("first page");
        let (width, height) = (
            page.width().value.round() as u32,
            page.height().value.round() as u32,
        );
        let rendered = page
            .render_with_config(&render_config(width, height).render_annotations(true))
            .expect("annotations render");
        let rgba = rendered.as_rgba_bytes();
        image::RgbaImage::from_raw(width, height, rgba.clone())
            .expect("light snapshot pixels")
            .save(out.join("pdf-exported-page-light.png"))
            .expect("save light snapshot");

        let mut bgra = Vec::with_capacity(rgba.len());
        for pixel in rgba.as_chunks::<4>().0 {
            bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
        crate::invert::smart_invert(&mut bgra, width, height, &[]);
        let mut dark_rgba = Vec::with_capacity(bgra.len());
        for pixel in bgra.as_chunks::<4>().0 {
            dark_rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
        image::RgbaImage::from_raw(width, height, dark_rgba)
            .expect("dark snapshot pixels")
            .save(out.join("pdf-exported-page-dark.png"))
            .expect("save dark snapshot");
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
