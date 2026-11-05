//! Handwriting search: the words in ink on canvases and sketches, read on
//! this device so the app's search finds a canvas by what was written on it.
//!
//! On macOS the same Vision reader that reads attached images reads each
//! layer of ink, drawn as black strokes on a light page, in the background
//! and one layer at a time: only layers whose strokes changed since they
//! were last read (`kanso_engine::discovery::ink_ocr`). The words go into
//! the canvas's own body as `kansoInkOcr`, so they are sealed with a locked
//! canvas and sync with it; Windows and Linux have no reader and still
//! search words another device read. Reading waits for ink to be still, so
//! it never happens during a stroke, and it never opens a locked canvas.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::ocr::system_reader;
use crate::workspace::AppState;
use gpui_kit::{App, Global, Task};
use kanso_engine::discovery::ink_ocr::{self, Next, Pass};

pub use kanso_engine::discovery::ocr::Status;

/// The pause between layers, so reading a big canvas never crowds the app.
const LAYER_PAUSE: Duration = Duration::from_millis(250);
/// How long a note stays still before its ink is read: a canvas is saved
/// after every stroke, so this is what keeps recognition out of the drawing.
const SETTLE: Duration = Duration::from_secs(4);

/// Whether this device can read handwriting at all.
pub fn supported() -> bool {
    crate::ocr::supported()
}

#[derive(Default)]
pub struct InkText {
    pub status: Status,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for InkText {}

fn set_status(status: Status, cx: &mut App) {
    let state = cx.default_global::<InkText>();
    if state.status != status {
        state.status = status;
        cx.refresh_windows();
    }
}

/// Starts reading at launch when it is on and this device can.
pub fn init(cx: &mut App) {
    cx.default_global::<InkText>();
    if cx.global::<AppState>().config.discovery.ink_text {
        start(cx);
    }
}

/// Turns reading on or off, from Settings.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.ink_text = on);
    if on {
        start(cx);
    } else {
        let state = cx.default_global::<InkText>();
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
    let state = cx.default_global::<InkText>();
    state.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    state.cancel = cancel.clone();
    let task = cx.spawn(async move |cx| {
        // What this run has looked at and what waits, from layer to layer:
        // each look reads only the notes written since the last one.
        let mut pass = Pass::new(SETTLE);
        loop {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            // Pictures and the reader run off the app's thread...
            let (store2, reader2) = (store.clone(), reader.clone());
            let (next, back) = cx
                .background_executor()
                .spawn(async move {
                    let next = ink_ocr::read(&store2, &*reader2, &mut pass);
                    (next, pass)
                })
                .await;
            pass = back;
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            match next {
                Ok(Next::Read(read)) => {
                    // ...and the words are kept on it, where nothing can
                    // edit the note between the look and the write. A note
                    // being edited right now waits until it is still.
                    let kept = cx.update(|cx| {
                        if crate::editor::ink_glue::editing(&read.ink.note, cx) {
                            pass.postpone(&read.ink.note);
                            return Ok(false);
                        }
                        ink_ocr::keep_read(&store, &mut pass, &read)
                    });
                    if let Err(err) = kept {
                        eprintln!("kanso: reading handwriting stopped: {err}");
                        cx.update(|cx| set_status(Status::Idle, cx));
                        return;
                    }
                    let left = pass.waiting();
                    cx.update(|cx| set_status(Status::Reading { left }, cx));
                    cx.background_executor().timer(LAYER_PAUSE).await;
                }
                Ok(Next::Settling(left)) => {
                    let waiting = pass.waiting();
                    cx.update(|cx| set_status(Status::Reading { left: waiting }, cx));
                    cx.background_executor().timer(left.max(LAYER_PAUSE)).await;
                }
                Ok(Next::Done) => {
                    cx.update(|cx| set_status(Status::Idle, cx));
                    // Wait for a note to change, then for things to settle.
                    loop {
                        let Ok(change) = changes.recv().await else {
                            return;
                        };
                        // A note (a stroke saved, a canvas unlocked), or what
                        // decides whether one may be read.
                        if change.id.starts_with("note:")
                            || change.id.starts_with("tag:")
                            || change.id.starts_with("book:")
                        {
                            break;
                        }
                    }
                    while changes.try_recv().is_ok() {}
                }
                Err(err) => {
                    eprintln!("kanso: reading handwriting stopped: {err}");
                    cx.update(|cx| set_status(Status::Idle, cx));
                    return;
                }
            }
            // Changes are looked at through the store's sequence, not this
            // feed: what queued up while reading is not needed.
            while changes.try_recv().is_ok() {}
        }
    });
    let state = cx.default_global::<InkText>();
    state.task = Some(task);
    if state.status == Status::Off {
        state.status = Status::Idle;
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use kanso_core::canvas::{INK, Node, Point, Stroke};

    use super::*;

    /// Pen samples along a polyline, every 3 points, as a pen reports them.
    fn along(id: &str, corners: &[(f64, f64)]) -> Stroke {
        let mut points = Vec::new();
        for pair in corners.windows(2) {
            let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
            let steps = (((x1 - x0).hypot(y1 - y0)) / 3.0).ceil().max(1.0) as usize;
            for i in 0..steps {
                let t = i as f64 / steps as f64;
                points.push(Point::new(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, 0.6));
            }
        }
        if let Some(&(x, y)) = corners.last() {
            points.push(Point::new(x, y, 0.6));
        }
        Stroke {
            id: id.into(),
            tool: "pen".into(),
            color: "#1971c2".into(),
            width: 4.0,
            points,
            ..Stroke::default()
        }
    }

    /// "HELLO" in block capitals, drawn stroke by stroke in blue ink.
    fn hello() -> Node {
        let mut strokes = Vec::new();
        let mut x = 0.0;
        let h = 70.0;
        let w = 44.0;
        let gap = 26.0;
        // H
        strokes.push(along("h1", &[(x, 0.0), (x, h)]));
        strokes.push(along("h2", &[(x + w, 0.0), (x + w, h)]));
        strokes.push(along("h3", &[(x, h / 2.0), (x + w, h / 2.0)]));
        x += w + gap;
        // E
        strokes.push(along("e1", &[(x + w, 0.0), (x, 0.0), (x, h), (x + w, h)]));
        strokes.push(along("e2", &[(x, h / 2.0), (x + w * 0.8, h / 2.0)]));
        x += w + gap;
        // L, L
        for name in ["l1", "l2"] {
            strokes.push(along(name, &[(x, 0.0), (x, h), (x + w, h)]));
            x += w + gap;
        }
        // O
        let ring: Vec<(f64, f64)> = (0..=24)
            .map(|i| {
                let a = f64::from(i) / 24.0 * std::f64::consts::TAU;
                (x + w / 2.0 + a.cos() * w / 2.0, h / 2.0 + a.sin() * h / 2.0)
            })
            .collect();
        strokes.push(along("o1", &ring));
        Node {
            id: "i1".into(),
            kind: INK.into(),
            width: x + w,
            height: h,
            strokes,
            ..Node::default()
        }
    }

    /// Reads a real set of strokes with Vision, through the same picture the
    /// app makes.
    #[test]
    #[ignore = "uses the system's Vision framework; run on demand"]
    fn vision_reads_handwriting_strokes() {
        let node = hello();
        let png = kanso_render::canvas::recognize::ink_png(&node).unwrap();
        if let Ok(dir) = std::env::var("KANSO_INK_PNG_DIR") {
            std::fs::write(std::path::Path::new(&dir).join("hello-ink.png"), &png).unwrap();
        }
        let reader = system_reader().unwrap();
        let text = reader.read(&png).unwrap();
        assert!(text.to_lowercase().contains("hello"), "read {text:?}");
    }
}
