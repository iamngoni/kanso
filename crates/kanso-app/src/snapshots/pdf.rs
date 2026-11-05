//! Snapshot states for the PDF reader (issue #43, step 3): page 1 of a report
//! the worker drew, and the reader after its worker was killed.
//!
//! These start the real `kanso-pdf-worker`, so they need
//! `scripts/fetch-pdfium.sh` and `cargo build -p kanso-pdf --features render`.

use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::pdf::{PdfReader, ReaderState};
use crate::workspace::{AppState, Workspace};

const NAME: &str = "Quarterly report.pdf";

/// Lets the worker answer: the headless executor runs its thread, but the
/// process needs real time.
fn wait_for(cx: &mut HeadlessAppContext, reader: &Entity<PdfReader>) {
    for _ in 0..100 {
        cx.run_until_parked();
        if !cx.update(|cx| matches!(reader.read(cx).state(), ReaderState::Loading)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the PDF worker never answered");
}

fn open_reader(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) -> Entity<PdfReader> {
    let paths = crate::pdf::tests::paths().unwrap_or_else(|why| panic!("{why}"));
    let id =
        kanso_engine::attach::attach_pdf(store, &kanso_pdf::fixtures::report_pdf(), NAME, None)
            .unwrap();
    let app_store = cx.update(|cx| cx.global::<AppState>().store.clone());
    let reader = cx
        .update_window(window, |_, window, cx| {
            let scale = window.scale_factor();
            let reader = cx.new(|cx| {
                PdfReader::with_paths(app_store, id, NAME.into(), 1, scale, Some(paths), cx)
            });
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.show_pdf(reader.clone(), window, cx));
            reader
        })
        .unwrap();
    wait_for(cx, &reader);
    reader
}

/// Page 1 of a report, beside the note.
pub(super) fn reader_page_one(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(cx, window, workspace, store);
    assert!(matches!(
        cx.update(|cx| reader.read(cx).state().clone()),
        ReaderState::Showing { page: 1, .. }
    ));
}

/// The worker killed under the reader: an error and Try Again, not a crash.
pub(super) fn reader_worker_killed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let reader = open_reader(cx, window, workspace, store);
    let pid = cx
        .update(|cx| reader.read(cx).worker_pid())
        .expect("the worker is running");
    std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .unwrap();
    cx.update(|cx| reader.update(cx, |reader, cx| reader.show_page(0, cx)));
    wait_for(cx, &reader);
    assert!(matches!(
        cx.update(|cx| reader.read(cx).state().clone()),
        ReaderState::Failed(_)
    ));
}
