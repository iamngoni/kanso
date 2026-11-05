//! Locking with pictures and files in it: what the dialogs say about them,
//! the progress sheet a long lock shows, and the work behind Lock Note,
//! Lock Notebook and Remove Lock (`kanso_engine::lock::files` does the
//! sealing).
//!
//! The bytes are read and encrypted once, so a note with many or large
//! attachments takes a while: the work runs on a thread of its own with a
//! sheet that counts the files, and Cancel stops it between two files,
//! leaving the note plain with what it needs to finish or undo
//! (`NoteEditor`'s "Locking didn't finish" bar).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::lock::files::{Phase, Progress, Summary};

use crate::theme::Colors;
use crate::zoom::px;

/// Files from which a lock shows its progress sheet.
pub const SHEET_FILES: usize = 4;
/// Bytes from which a lock shows its progress sheet.
pub const SHEET_BYTES: u64 = 8 * 1024 * 1024;

/// The width of the progress sheet.
pub const SHEET_WIDTH: f32 = 440.;

/// A size in plain words: "84 MB", "512 KB".
pub fn size_words(bytes: u64) -> String {
    const KB: f64 = 1024.;
    let bytes = bytes as f64;
    if bytes >= KB * KB * 10. {
        format!("{:.0} MB", bytes / (KB * KB))
    } else if bytes >= KB * KB {
        format!("{:.1} MB", bytes / (KB * KB))
    } else if bytes >= KB {
        format!("{:.0} KB", (bytes / KB).max(1.))
    } else {
        "under 1 KB".to_owned()
    }
}

fn attachments(n: usize) -> String {
    format!("{n} attachment{}", if n == 1 { "" } else { "s" })
}

/// What the sheet says it is doing: "Sealing 12 attachments, 84 MB".
pub fn sheet_line(summary: &Summary) -> String {
    format!(
        "Sealing {}, {}",
        attachments(summary.files),
        size_words(summary.bytes)
    )
}

/// What the sheet says Remove Lock is doing.
pub fn unsealing_line(files: usize) -> String {
    format!("Unsealing {}", attachments(files))
}

/// Said after a lock that made sealed copies of files other notes show.
pub fn copied_words(copied: usize) -> String {
    format!(
        "Locked. {} also used by other notes, so this note has its own sealed {}.",
        attachments(copied),
        if copied == 1 { "copy" } else { "copies" },
    )
}

/// Said after Remove Lock gave the note plain copies of files another locked
/// note still shows.
pub fn unsealed_words(copied: usize) -> String {
    format!(
        "Lock removed. {} {} copied for this note, because another locked note still shows the sealed {}.",
        attachments(copied),
        if copied == 1 { "was" } else { "were" },
        if copied == 1 { "one" } else { "ones" },
    )
}

/// What Kanso tells, once, the first time it seals the pictures and files of
/// a note that was locked before: the change in how locked notes behave,
/// and what to do about other devices.
pub const FILES_NOTICE: &str = "Locked notes now seal their pictures and files too. A note locked before is sealed the first time you edit it. Update Kanso on every device before you change the lock password: an older version can't re-seal the files, and they would stop opening.";

/// The sentence the lock dialogs gain: the note's pictures and files are
/// sealed with it, and a file another note also shows is copied for it.
/// Empty when the note has none.
pub fn dialog_sentence(summary: &Summary) -> String {
    if summary.files == 0 {
        return String::new();
    }
    let mut words = format!(
        "Its {} ({}) {} sealed too.",
        if summary.files == 1 {
            "picture or file"
        } else {
            "pictures and files"
        },
        size_words(summary.bytes),
        if summary.files == 1 { "is" } else { "are" },
    );
    if summary.shared > 0 {
        words.push_str(&format!(
            " {} {} also used by other notes: the locked note keeps its own sealed {}, and the other notes keep theirs.",
            attachments(summary.shared),
            if summary.shared == 1 { "is" } else { "are" },
            if summary.shared == 1 { "copy" } else { "copies" },
        ));
    }
    words
}

/// The bar a note shows when a lock stopped between its files.
pub fn interrupted_words(files: usize, sealed: usize) -> String {
    format!(
        "Locking this note didn't finish: {sealed} of its {files} pictures and files are sealed, and the note isn't locked yet. Finish locking, or undo it to keep everything as it was."
    )
}

/// Whether a lock this size shows the sheet.
pub fn wants_sheet(summary: &Summary) -> bool {
    summary.files >= SHEET_FILES || summary.bytes >= SHEET_BYTES
}

/// The progress sheet's state: what it counts, how far it is, and the flag
/// Cancel sets.
pub struct LockSheet {
    line: SharedString,
    progress: Progress,
    cancel: Arc<AtomicBool>,
}

impl LockSheet {
    pub fn new(line: String, summary: &Summary, cancel: Arc<AtomicBool>) -> Self {
        LockSheet {
            line: line.into(),
            progress: Progress {
                total: summary.files,
                bytes_total: summary.bytes,
                ..Progress::default()
            },
            cancel,
        }
    }

    /// Where the sealing is now.
    pub fn show(&mut self, progress: Progress, cx: &mut Context<Self>) {
        self.progress = progress;
        cx.notify();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// The share of the files sealed, 0 to 1.
    pub fn fraction(&self) -> f32 {
        if self.progress.total == 0 {
            0.
        } else {
            (self.progress.done as f32 / self.progress.total as f32).clamp(0., 1.)
        }
    }

    /// "3 of 12", or what the lock does while it isn't sealing a file.
    pub fn count_words(&self) -> String {
        match self.progress.phase {
            Phase::Preparing => "Getting ready".to_owned(),
            Phase::Locking => "Locking the note".to_owned(),
            Phase::Sealing => format!("{} of {}", self.progress.done, self.progress.total),
        }
    }
}

// no-prop-test: drawing; the snapshots show the sheet part way through.
impl Render for LockSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let fraction = self.fraction();
        let cancelled = self.is_cancelled();
        v_flex()
            .gap(px(10.))
            .pb(px(4.))
            .debug_selector(|| "lock-sheet".into())
            .child(
                div()
                    .text_size(px(13.5))
                    .line_height(px(19.))
                    .text_color(colors.text_secondary)
                    .child(self.line.clone()),
            )
            .child(
                div()
                    .debug_selector(|| "lock-sheet-bar".into())
                    .h(px(6.))
                    .w_full()
                    .rounded(px(3.))
                    .bg(colors.fill)
                    .child(
                        div()
                            .h_full()
                            .w(relative(fraction))
                            .rounded(px(3.))
                            .bg(colors.accent),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(self.count_words())
                    .when(cancelled, |el| el.child("Stopping after this file")),
            )
    }
}

/// The sheet's Cancel button, in the dialog's footer.
struct SheetFooter {
    sheet: Entity<LockSheet>,
    _observing: Subscription,
}

impl SheetFooter {
    fn new(sheet: Entity<LockSheet>, cx: &mut Context<Self>) -> Self {
        let observing = cx.observe(&sheet, |_, _, cx| cx.notify());
        SheetFooter {
            sheet,
            _observing: observing,
        }
    }
}

// no-prop-test: drawing.
impl Render for SheetFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cancelled = self.sheet.read(cx).is_cancelled();
        let flag = self.sheet.read(cx).cancel.clone();
        let sheet = self.sheet.clone();
        h_flex().w_full().justify_end().child(
            Button::new("lock-sheet-cancel")
                .label("Cancel")
                .disabled(cancelled)
                .debug_selector(|| "lock-sheet-cancel".into())
                .on_click(move |_, _, cx| {
                    flag.store(true, Ordering::SeqCst);
                    // "Stopping after this file" shows now, not at the
                    // next file.
                    sheet.update(cx, |_, cx| cx.notify());
                }),
        )
    }
}

/// The progress sheet, open in `window`.
pub fn open_sheet(
    title: &str,
    line: String,
    summary: &Summary,
    window: &mut Window,
    cx: &mut App,
) -> Entity<LockSheet> {
    let sheet = cx.new(|_| LockSheet::new(line, summary, Arc::default()));
    let footer = cx.new(|cx| SheetFooter::new(sheet.clone(), cx));
    let (title, body) = (SharedString::from(title.to_owned()), sheet.clone());
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(body.clone())
            .footer(footer.clone())
            // Escape doesn't dismiss it: Cancel stops the work.
            .on_cancel(|_, _, _| false)
    });
    sheet
}

/// What the work thread tells the window.
enum Event<T> {
    Progress(Progress),
    Finished(Result<T, String>),
}

/// Runs `work` and then `done` with its result, on the main thread. A lock
/// of few and small files runs at once; a larger one runs on a thread of
/// its own behind the progress sheet. `work` gets the callback that reports
/// progress and stops it between two files (Cancel does) when it returns an
/// error.
pub fn run<T: Send + 'static>(
    title: &'static str,
    line: String,
    summary: Summary,
    window: &mut Window,
    cx: &mut App,
    work: impl FnOnce(&mut dyn FnMut(&Progress) -> Result<(), String>) -> Result<T, String>
    + Send
    + 'static,
    done: impl FnOnce(Result<T, String>, &mut Window, &mut App) + 'static,
) {
    if !wants_sheet(&summary) {
        let result = work(&mut |_| Ok(()));
        // After the update that asked, which `done` often updates again.
        window.defer(cx, move |window, cx| done(result, window, cx));
        return;
    }
    let sheet = open_sheet(title, line, &summary, window, cx);
    let cancel = sheet.read(cx).cancel.clone();
    let (events, listening) = async_channel::unbounded::<Event<T>>();
    let handle = window.window_handle();
    let spawned = std::thread::Builder::new()
        .name("kanso-lock".into())
        .spawn(move || {
            let sender = events.clone();
            let result = work(&mut |progress| {
                sender.try_send(Event::Progress(*progress)).ok();
                if cancel.load(Ordering::SeqCst) {
                    Err("Locking was cancelled.".to_owned())
                } else {
                    Ok(())
                }
            });
            events.try_send(Event::Finished(result)).ok();
        });
    if spawned.is_err() {
        window.close_dialog(cx);
        done(Err("Couldn't start. Try again.".into()), window, cx);
        return;
    }
    cx.spawn(async move |cx| {
        while let Ok(event) = listening.recv().await {
            match event {
                Event::Progress(progress) => {
                    sheet.update(cx, |sheet, cx| sheet.show(progress, cx));
                }
                Event::Finished(result) => {
                    cx.update(|cx| {
                        handle
                            .update(cx, |_, window, cx| {
                                window.close_dialog(cx);
                                done(result, window, cx);
                            })
                            .ok();
                    });
                    return;
                }
            }
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[::core::prelude::v1::test]
    fn the_sheet_names_the_files_and_their_size() {
        let summary = Summary {
            files: 12,
            bytes: 84 * 1024 * 1024,
            shared: 0,
        };
        assert_eq!(sheet_line(&summary), "Sealing 12 attachments, 84 MB");
        let one = Summary {
            files: 1,
            bytes: 2_300_000,
            shared: 0,
        };
        assert_eq!(sheet_line(&one), "Sealing 1 attachment, 2.2 MB");
    }

    #[::core::prelude::v1::test]
    fn the_dialog_says_when_a_copy_is_made() {
        let none = Summary::default();
        assert_eq!(dialog_sentence(&none), "");
        let some = Summary {
            files: 3,
            bytes: 3 * 1024 * 1024,
            shared: 2,
        };
        let words = dialog_sentence(&some);
        assert!(
            words.contains("pictures and files (3.0 MB) are sealed too"),
            "{words}"
        );
        assert!(
            words.contains("2 attachments are also used by other notes"),
            "{words}"
        );
        assert!(words.contains("keeps its own sealed copies"), "{words}");
        let one = Summary {
            files: 1,
            bytes: 10,
            shared: 1,
        };
        assert!(dialog_sentence(&one).contains("1 attachment is also used by other notes"));
    }

    proptest! {
        /// The sheet shows from four files or eight megabytes, and not for
        /// less; the dialog mentions a copy exactly when a file is shared.
        #[::core::prelude::v1::test]
        fn prop_the_sheet_and_the_copy_line_follow_the_summary(
            files in 0usize..40,
            bytes in 0u64..64 * 1024 * 1024,
            shared in 0usize..40,
        ) {
            let summary = Summary { files, bytes, shared: shared.min(files) };
            prop_assert_eq!(
                wants_sheet(&summary),
                files >= SHEET_FILES || bytes >= SHEET_BYTES
            );
            let words = dialog_sentence(&summary);
            prop_assert_eq!(words.is_empty(), files == 0);
            prop_assert_eq!(
                words.contains("also used by other notes"),
                files > 0 && summary.shared > 0
            );
        }
    }
}
