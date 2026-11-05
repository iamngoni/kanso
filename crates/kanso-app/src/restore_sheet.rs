//! Settings › Data › Backups › Restore from backup… (issue #243 step 11).
//!
//! Choose a backup file. One this computer's key opens (or a plain one) goes
//! straight to the question; one made with another key, as on another
//! computer, asks for the library key's 24 words first. Restore writes the
//! backup, re-encrypted with this library's key, beside the library
//! (`kanso_store::stage_restore`); the next launch swaps it in and keeps the
//! library it replaces as a backup. Nothing about the library changes until
//! then. The words are read from the field once, parsed, and the field is
//! emptied; the parsed key is wiped when it goes out of scope.

use std::path::PathBuf;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Disableable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::{LibraryFile, LibraryKey, StoreError, backup_name, check_backup, stage_restore};

use crate::clock::Clock;
use crate::library_key_model::{
    STAGED, needs_words, restore_problem, restore_refusal, words_problem,
};
use crate::paths;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of the sheet.
pub const SHEET_WIDTH: f32 = 520.;

/// Asks for a backup file, then opens the sheet for it.
pub fn start(window: &mut Window, cx: &mut App) {
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Choose".into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            cx.update(|window, cx| {
                begin(path, window, cx);
            })
            .ok();
        })
        .detach();
}

/// Looks at the backup at `path`: opens the sheet, with the words field when
/// this computer's key doesn't open it, or says why it can't be restored.
pub fn begin(path: PathBuf, window: &mut Window, cx: &mut App) -> Option<WeakEntity<RestoreSheet>> {
    if let Some(why) = signed_in_refusal(cx) {
        window.push_notification(Notification::error(why), cx);
        return None;
    }
    let key = cx.global::<AppState>().store.library_key().cloned();
    let kind = LibraryFile::of(&path);
    let checked = check_backup(&path, key.as_ref());
    let asks_for_words = match &checked {
        Ok(()) => false,
        Err(StoreError::WrongKey) => needs_words(kind, false),
        Err(err) => {
            window.push_notification(Notification::error(restore_problem(err, false)), cx);
            return None;
        }
    };
    let sheet = cx.new(|cx| RestoreSheet::new(path, asks_for_words, window, cx));
    let weak = sheet.downgrade();
    open(sheet, window, cx);
    Some(weak)
}

/// Why a restore can't be staged now: the library is signed in to sync.
fn signed_in_refusal(cx: &App) -> Option<&'static str> {
    let services = cx.try_global::<crate::services::Services>()?;
    restore_refusal(services.signed_in(), services.has_locked_session())
}

/// The sheet: what is being restored, the words if they're needed, and the
/// reason the last try failed.
pub struct RestoreSheet {
    path: PathBuf,
    made: Option<String>,
    words: Option<Entity<TextareaState>>,
    error: Option<String>,
    busy: bool,
    _typing: Option<Subscription>,
}

impl RestoreSheet {
    fn new(
        path: PathBuf,
        asks_for_words: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> RestoreSheet {
        let made = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(backup_name)
            .map(|name| Clock::edited_label(cx, name.made_at_ms));
        let words = asks_for_words.then(|| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder("Type or paste the 24 words"))
        });
        let typing = words.as_ref().map(|words| {
            cx.subscribe_in(words, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.error = None;
                    cx.notify();
                }
            })
        });
        RestoreSheet {
            path,
            made,
            words,
            error: None,
            busy: false,
            _typing: typing,
        }
    }

    /// Types `text` into the words field, as a person would paste it.
    #[cfg(test)]
    pub(crate) fn type_words_for_test(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let words = self.words.clone().expect("this sheet asks for words");
        words.update(cx, |words, cx| words.set_value(text.to_owned(), window, cx));
    }

    #[cfg(test)]
    pub(crate) fn error_for_test(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn ready(&self, cx: &App) -> bool {
        !self.busy
            && self
                .words
                .as_ref()
                .is_none_or(|words| !words.read(cx).value().trim().is_empty())
    }

    /// Reads the words (if they're asked for), then stages the restore off
    /// the main thread.
    fn restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Signed in since the sheet opened.
        if let Some(why) = signed_in_refusal(cx) {
            self.error = Some(why.to_owned());
            cx.notify();
            return;
        }
        let state = cx.global::<AppState>();
        let db = paths::db_path(&state.data_dir);
        let Some(library_key) = state.store.library_key().cloned() else {
            return;
        };
        let typed = self.words.is_some();
        let backup_key = match &self.words {
            Some(words) => {
                let text = words.read(cx).value();
                let parsed = LibraryKey::from_words(&text);
                match parsed {
                    Ok(key) => {
                        // The field has done its job.
                        words.update(cx, |words, cx| words.set_value("", window, cx));
                        Some(key)
                    }
                    Err(err) => {
                        self.error = Some(words_problem(err));
                        cx.notify();
                        return;
                    }
                }
            }
            // This computer's key opens it, or it is plain.
            None => {
                (LibraryFile::of(&self.path) == LibraryFile::Encrypted).then(|| library_key.clone())
            }
        };
        self.busy = true;
        self.error = None;
        cx.notify();
        let path = self.path.clone();
        let task = cx.background_spawn(async move {
            stage_restore(&db, &path, backup_key.as_ref(), &library_key)
        });
        cx.spawn_in(window, async move |this, cx| {
            let staged = task.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match staged {
                    Ok(()) => {
                        window.close_dialog(cx);
                        window.push_notification(
                            Notification::success(STAGED)
                                .autohide(false)
                                .action(|_, _, _| {
                                    Button::new("restore-quit")
                                        .label("Quit Kanso")
                                        .debug_selector(|| "restore-quit".into())
                                        .on_click(|_, _, cx| crate::quit(cx))
                                }),
                            cx,
                        );
                    }
                    Err(err) => this.error = Some(restore_problem(&err, typed)),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for RestoreSheet {
    // no-prop-test: drawing; the wording is `library_key_model`.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let name = self
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let what = match &self.made {
            Some(made) => format!("The backup made {made}."),
            None => format!("The file {name}."),
        };
        v_flex()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(format!(
                        "{what} It replaces this library when you open Kanso again. The library you have now is kept as a backup."
                    )),
            )
            .when_some(self.words.as_ref(), |el, words| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child("This backup is encrypted with another key. Enter the 24 words of the library it came from."),
                )
                .child(
                    div()
                        .debug_selector(|| "restore-words".into())
                        .child(Textarea::new(words).h(px(96.)).disabled(self.busy)),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .id("restore-error")
                        .debug_selector(|| "restore-error".into())
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// The sheet's buttons; they read the sheet.
struct RestoreFooter {
    sheet: Entity<RestoreSheet>,
    _observing: Subscription,
}

// no-prop-test: drawing and clicks.
impl Render for RestoreFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ready = self.sheet.read(cx).ready(cx);
        let busy = self.sheet.read(cx).busy;
        let target = self.sheet.clone();
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(
                Button::new("restore-cancel")
                    .label("Cancel")
                    .disabled(busy)
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(
                Button::new("restore-go")
                    .debug_selector(|| "restore-go".into())
                    .primary()
                    .label("Restore")
                    .loading(busy)
                    .disabled(!ready)
                    .on_click(move |_, window, cx| {
                        target.update(cx, |sheet, cx| sheet.restore(window, cx))
                    }),
            )
    }
}

fn open(sheet: Entity<RestoreSheet>, window: &mut Window, cx: &mut App) {
    let footer = cx.new(|cx| RestoreFooter {
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
        sheet: sheet.clone(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Restore from backup")
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(footer.clone())
    });
}

/// The sheet for `path`, asking for words or not, for the snapshots.
#[cfg(test)]
pub(crate) fn open_for_test(
    path: &std::path::Path,
    asks_for_words: bool,
    error: Option<&str>,
    window: &mut Window,
    cx: &mut App,
) -> WeakEntity<RestoreSheet> {
    let sheet = cx.new(|cx| {
        let mut sheet = RestoreSheet::new(path.to_owned(), asks_for_words, window, cx);
        sheet.error = error.map(str::to_owned);
        sheet
    });
    let weak = sheet.downgrade();
    open(sheet, window, cx);
    weak
}
