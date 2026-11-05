//! Snapshot states for the PDF reader (issue #43, steps 3 and 4): the
//! continuous column with its thumbnails, smart invert in a dark reader and
//! the page as printed, the password prompt, and the worker killed under it.
//!
//! These start the real `kanso-pdf-worker`, so they need
//! `scripts/fetch-pdfium.sh` and `cargo build -p kanso-pdf --features render`.

use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::pdf::{PdfReader, ReaderState};
use crate::workspace::{AppState, Workspace};

const NAME: &str = "Quarterly report.pdf";

/// Draws a frame.
fn draw(cx: &mut HeadlessAppContext, window: AnyWindowHandle) {
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .ok();
    cx.run_until_parked();
}

/// Lets the worker answer and the pages draw: the headless executor runs its
/// thread, but the process needs real time.
fn wait_until(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    reader: &Entity<PdfReader>,
    done: impl Fn(&PdfReader) -> bool,
) {
    for _ in 0..200 {
        draw(cx, window);
        if cx.update(|cx| done(reader.read(cx))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the PDF reader never got there");
}

fn open_reader(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
    pdf: &[u8],
) -> Entity<PdfReader> {
    let paths = crate::pdf::tests::paths().unwrap_or_else(|why| panic!("{why}"));
    // The worker answers from its own thread, in real time.
    cx.allow_parking();
    let id = kanso_engine::attach::attach_pdf(store, pdf, NAME, None).unwrap();
    let app_store = cx.update(|cx| cx.global::<AppState>().store.clone());
    let reader = cx
        .update_window(window, |_, window, cx| {
            let reader = cx.new(|cx| {
                PdfReader::with_paths(app_store, id, NAME.into(), 1, Some(paths), window, cx)
            });
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.show_pdf(reader.clone(), window, cx));
            reader
        })
        .unwrap();
    wait_until(cx, window, &reader, PdfReader::settled);
    reader
}

/// The report's first page, its next pages below it, and the thumbnails.
pub(super) fn reader_page_one(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::report_pdf(),
    );
    assert!(matches!(
        cx.update(|cx| reader.read(cx).state().clone()),
        ReaderState::Showing { page: 1, .. }
    ));
}

/// A photo, a red and a blue chart and black text: in a dark theme the text
/// is light and the photo and the bars keep their colours.
pub(super) fn reader_smart_invert(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::photo_pdf(),
    );
}

/// The same page shown as printed, from the toggle.
pub(super) fn reader_original_page(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::photo_pdf(),
    );
    cx.update_window(window, |_, window, cx| {
        reader.update(cx, |reader, cx| reader.pick_original(true, window, cx));
    })
    .unwrap();
    // The page is drawn again; wait for the printed one.
    wait_until(cx, window, &reader, |reader| {
        reader
            .page_texture(0)
            .is_some_and(|image| image.as_bytes(0).unwrap()[..4] == [255, 255, 255, 255])
    });
}

/// An encrypted PDF asking for its password.
pub(super) fn reader_password(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::encrypted_pdf("secret"),
    );
    assert_eq!(
        cx.update(|cx| reader.read(cx).state().clone()),
        ReaderState::NeedsPassword { wrong: false }
    );
}

/// The prompt after a wrong password.
pub(super) fn reader_password_wrong(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::encrypted_pdf("secret"),
    );
    cx.update_window(window, |_, window, cx| {
        reader.update(cx, |reader, cx| reader.type_password("not it", window, cx));
    })
    .unwrap();
    wait_until(cx, window, &reader, |reader| {
        *reader.state() == ReaderState::NeedsPassword { wrong: true }
    });
}

/// The worker killed under the reader: an error and Try Again, not a crash.
pub(super) fn reader_worker_killed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(
        cx,
        window,
        workspace,
        store,
        &kanso_pdf::fixtures::report_pdf(),
    );
    let pid = cx
        .update(|cx| reader.read(cx).worker_pid())
        .expect("the worker is running");
    std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .unwrap();
    // Drawing the pages again (as printed, now) finds the worker gone.
    cx.update_window(window, |_, window, cx| {
        reader.update(cx, |reader, cx| reader.pick_original(true, window, cx));
    })
    .unwrap();
    wait_until(cx, window, &reader, |reader| {
        matches!(reader.state(), ReaderState::Failed(_))
    });
}
