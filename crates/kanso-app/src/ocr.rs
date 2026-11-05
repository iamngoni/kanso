//! Text in images: the words in image attachments, read on this device so
//! the app's search finds a note by what its pictures say.
//!
//! On macOS, Apple's Vision framework reads them (`VNRecognizeTextRequest`,
//! accurate, with language correction), in the background, one image at a
//! time: images attached since, or never read, or whose bytes changed. The
//! text goes into the file's `kansoOcr` (`kanso-store/src/ocr.rs`), which
//! syncs sealed with the file, so other devices search it too without
//! reading the image again. Windows and Linux have no reader: they skip it
//! and still search text another device read.
//!
//! The reader is behind [`TextReader`], so tests use a fake. Which image is
//! next, and keeping what was read, are the engine's
//! (`kanso_engine::discovery::ocr`), which the phones read through their own
//! reader with. Only images a note shows that may be read are offered: not
//! those only a locked note or a note tagged `#ai-exclude` shows.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::workspace::AppState;
use gpui_kit::{App, Global, Task};

pub use kanso_engine::discovery::ocr::{Status, TextReader, read_next};

/// The pause between images, so reading many never crowds the app.
const IMAGE_PAUSE: Duration = Duration::from_millis(120);
/// How long attachments stay still before new ones are read.
const SETTLE: Duration = Duration::from_millis(2000);

/// The system's reader, where there is one.
pub fn system_reader() -> Option<Arc<dyn TextReader>> {
    #[cfg(target_os = "macos")]
    {
        Some(Arc::new(vision::VisionReader))
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// The system's reader of words with their places, where there is one: what
/// scanned PDF pages are read with (`pdf/ocr.rs`).
pub fn word_reader() -> Option<Arc<dyn kanso_engine::discovery::pdf_ocr::WordReader>> {
    #[cfg(target_os = "macos")]
    {
        Some(Arc::new(vision::VisionReader))
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Whether this device can read images at all.
pub fn supported() -> bool {
    cfg!(target_os = "macos")
}

#[derive(Default)]
pub struct ImageText {
    pub status: Status,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for ImageText {}

fn set_status(status: Status, cx: &mut App) {
    let state = cx.default_global::<ImageText>();
    if state.status != status {
        state.status = status;
        cx.refresh_windows();
    }
}

/// Starts reading at launch when it is on and this device can.
pub fn init(cx: &mut App) {
    cx.default_global::<ImageText>();
    if cx.global::<AppState>().config.discovery.image_text {
        start(cx);
    }
}

/// Turns reading on or off, from Settings.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.image_text = on);
    crate::pdf::ocr::refresh(cx);
    if on {
        start(cx);
    } else {
        let state = cx.default_global::<ImageText>();
        state.cancel.store(true, Ordering::Relaxed);
        state.task = None;
        set_status(Status::Off, cx);
    }
}

fn start(cx: &mut App) {
    let Some(reader) = system_reader() else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let changes = store.subscribe();
    let state = cx.default_global::<ImageText>();
    state.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    state.cancel = cancel.clone();
    let task = cx.spawn(async move |cx| {
        loop {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let (store2, reader2) = (store.clone(), reader.clone());
            let waiting = cx
                .background_executor()
                .spawn(async move { read_next(&store2, &*reader2) })
                .await;
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            match waiting {
                Ok(0) => {
                    cx.update(|cx| set_status(Status::Idle, cx));
                    // Wait for a file to change, then for things to settle.
                    loop {
                        let Ok(change) = changes.recv().await else {
                            return;
                        };
                        // A new picture, or a note that may now show one
                        // (or no longer be locked or excluded).
                        if (change.id.starts_with("file:") && !change.deleted)
                            || change.id.starts_with("note:")
                            || change.id.starts_with("tag:")
                        {
                            break;
                        }
                    }
                    cx.background_executor().timer(SETTLE).await;
                    while changes.try_recv().is_ok() {}
                }
                Ok(n) => {
                    cx.update(|cx| set_status(Status::Reading { left: n - 1 }, cx));
                    cx.background_executor().timer(IMAGE_PAUSE).await;
                }
                Err(err) => {
                    eprintln!("kanso: reading text in images stopped: {err}");
                    cx.update(|cx| set_status(Status::Idle, cx));
                    return;
                }
            }
        }
    });
    let state = cx.default_global::<ImageText>();
    state.task = Some(task);
    if state.status == Status::Off {
        state.status = Status::Idle;
    }
}

#[cfg(target_os = "macos")]
mod vision {
    //! Apple's Vision framework: `VNRecognizeTextRequest` on the image's
    //! bytes, accurate, with language correction.

    use kanso_pdf::ocr::{quad_from_bottom_left, turns_from_corners, word_ranges};
    use objc2::AnyThread as _;
    use objc2::rc::{Retained, autoreleasepool};
    use objc2_foundation::{NSArray, NSData, NSDictionary, NSRange};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
    };

    pub struct VisionReader;

    impl super::TextReader for VisionReader {
        fn engine(&self) -> &'static str {
            "vision"
        }

        fn read(&self, image: &[u8]) -> Result<String, String> {
            autoreleasepool(|_| {
                let data = NSData::with_bytes(image);
                let handler = VNImageRequestHandler::initWithData_options(
                    VNImageRequestHandler::alloc(),
                    &data,
                    &NSDictionary::new(),
                );
                let request = VNRecognizeTextRequest::new();
                request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
                request.setUsesLanguageCorrection(true);
                let as_request: Retained<VNRequest> =
                    Retained::into_super(Retained::into_super(request.clone()));
                let requests = NSArray::from_retained_slice(&[as_request]);
                handler
                    .performRequests_error(&requests)
                    .map_err(|err| err.localizedDescription().to_string())?;
                let mut lines = Vec::new();
                if let Some(results) = request.results() {
                    for observation in results.iter() {
                        if let Some(best) = observation.topCandidates(1).firstObject() {
                            lines.push(best.string().to_string());
                        }
                    }
                }
                Ok(lines.join("\n"))
            })
        }
    }

    impl kanso_engine::discovery::pdf_ocr::WordReader for VisionReader {
        fn engine(&self) -> &'static str {
            "vision"
        }

        /// Each recognised line's words, with the box Vision gives the
        /// range of the word in the line (`boundingBox(for:)`), flipped to
        /// the top-left origin.
        fn read_words(&self, image: &[u8]) -> Result<Vec<kanso_pdf::ReadWord>, String> {
            autoreleasepool(|_| {
                let data = NSData::with_bytes(image);
                let handler = VNImageRequestHandler::initWithData_options(
                    VNImageRequestHandler::alloc(),
                    &data,
                    &NSDictionary::new(),
                );
                let request = VNRecognizeTextRequest::new();
                request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
                request.setUsesLanguageCorrection(true);
                let as_request: Retained<VNRequest> =
                    Retained::into_super(Retained::into_super(request.clone()));
                let requests = NSArray::from_retained_slice(&[as_request]);
                handler
                    .performRequests_error(&requests)
                    .map_err(|err| err.localizedDescription().to_string())?;
                let mut words = Vec::new();
                if let Some(results) = request.results() {
                    for observation in results.iter() {
                        let Some(best) = observation.topCandidates(1).firstObject() else {
                            continue;
                        };
                        let line = best.string().to_string();
                        for (word, start, len) in word_ranges(&line) {
                            // SAFETY: the range lies inside the candidate's
                            // own string, in the UTF-16 units Vision counts.
                            let Ok(found) = (unsafe {
                                best.boundingBoxForRange_error(NSRange::new(start, len))
                            }) else {
                                continue;
                            };
                            // SAFETY: plain getters on an observation Vision made.
                            let (b, top_left, top_right) =
                                unsafe { (found.boundingBox(), found.topLeft(), found.topRight()) };
                            words.push(kanso_pdf::ReadWord {
                                text: word.to_owned(),
                                quad: quad_from_bottom_left(
                                    b.origin.x,
                                    b.origin.y,
                                    b.size.width,
                                    b.size.height,
                                ),
                                // A word on a turned line reads along the
                                // top edge of its own rectangle.
                                turns: turns_from_corners(
                                    (top_left.x, top_left.y),
                                    (top_right.x, top_right.y),
                                ),
                            });
                        }
                    }
                }
                Ok(words)
            })
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// "KANSO" in big block letters, black on white, as a PNG.
    fn kanso_png() -> Vec<u8> {
        const GLYPHS: [[&str; 7]; 5] = [
            [
                "X...X", "X..X.", "X.X..", "XX...", "X.X..", "X..X.", "X...X",
            ],
            [
                ".XXX.", "X...X", "X...X", "XXXXX", "X...X", "X...X", "X...X",
            ],
            [
                "X...X", "XX..X", "X.X.X", "X..XX", "X...X", "X...X", "X...X",
            ],
            [
                ".XXXX", "X....", "X....", ".XXX.", "....X", "....X", "XXXX.",
            ],
            [
                ".XXX.", "X...X", "X...X", "X...X", "X...X", "X...X", ".XXX.",
            ],
        ];
        let scale = 12u32;
        let (w, h) = ((5 * 6 + 4) * scale, 11 * scale);
        let image = image::RgbImage::from_fn(w, h, |x, y| {
            let (cx, cy) = (x / scale, y / scale);
            let ink = (2..9).contains(&cy)
                && cx >= 2
                && (cx - 2) % 6 < 5
                && ((cx - 2) / 6) < 5
                && GLYPHS[((cx - 2) / 6) as usize][(cy - 2) as usize].as_bytes()
                    [((cx - 2) % 6) as usize]
                    == b'X';
            if ink {
                image::Rgb([0, 0, 0])
            } else {
                image::Rgb([255, 255, 255])
            }
        });
        let mut png = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    /// Reads a real picture with Vision.
    #[test]
    #[ignore = "uses the system's Vision framework; run on demand"]
    fn vision_reads_words() {
        let text = vision::VisionReader.read(&kanso_png()).unwrap();
        assert!(text.to_lowercase().contains("kanso"), "read {text:?}");
    }

    /// Reads a real picture's word with its box, which lies where the
    /// letters are: the middle band of the picture, from the left edge.
    #[test]
    #[ignore = "uses the system's Vision framework; run on demand"]
    fn vision_reads_word_boxes() {
        use kanso_engine::discovery::pdf_ocr::WordReader as _;
        let words = vision::VisionReader.read_words(&kanso_png()).unwrap();
        let kanso_pdf::ReadWord { text, quad, turns } = words
            .iter()
            .find(|w| w.text.to_lowercase().contains("kanso"))
            .unwrap_or_else(|| panic!("read {words:?}"));
        assert_eq!(*turns, 0, "{text} reads left to right");
        let [x0, y0, x1, y1] = quad.0;
        assert!(x0 < 0.2 && x1 > 0.7, "{text} at {quad:?}");
        assert!(y0 > 0.1 && y1 < 0.9 && y0 < y1, "{text} at {quad:?}");
    }
}
