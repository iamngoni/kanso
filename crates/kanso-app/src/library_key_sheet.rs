//! The library key's sheet (issue #243 step 11): "Save your library key",
//! which comes up after encryption and at each launch until the person says
//! they saved it, and Settings › Data › Backups › Show key…, which shows the
//! same 24 words on request.
//!
//! The key is shown only when asked for, or on the save sheet. The words
//! live in the sheet alone, in a `Zeroizing` string, and are wiped when it
//! closes (as the New key sheet does for an API key). Copy puts them on the
//! clipboard as a concealed, transient secret that clears in a minute. They
//! are never logged. A view never reads its own entity in its render: the
//! footer reads the sheet.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use zeroize::Zeroizing;

use crate::library_key_model::{
    APPLIED, CONFIRM_BODY, COPIED, NOT_CONFIRMED, SAVE_BODY, SAVE_TITLE, SHOW_BODY, SHOW_TITLE,
    WAITING, key_row_detail, not_applied, numbered, offer_save_sheet, sync_holds_notes,
};
use crate::settings::Row;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;
use crate::{dialogs, services};

/// The width of the sheet.
pub const SHEET_WIDTH: f32 = 520.;

/// Words in a row of the grid.
const COLUMNS: usize = 4;

/// Why the sheet is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Offered by Kanso, right after encryption and at each launch.
    Save,
    /// Asked for with Show key….
    Show,
}

impl Mode {
    fn title(self) -> &'static str {
        match self {
            Mode::Save => SAVE_TITLE,
            Mode::Show => SHOW_TITLE,
        }
    }

    fn body(self) -> &'static str {
        match self {
            Mode::Save => SAVE_BODY,
            Mode::Show => SHOW_BODY,
        }
    }
}

/// The sheet's content: the 24 words, numbered.
pub struct LibraryKeySheet {
    mode: Mode,
    /// The key as words; `None` once wiped.
    words: Option<Zeroizing<String>>,
    already_saved: bool,
    copied: bool,
}

impl LibraryKeySheet {
    /// A sheet for the open library's key, or `None` when the library has
    /// none (it is plain).
    fn new(mode: Mode, cx: &App) -> Option<LibraryKeySheet> {
        let state = cx.global::<AppState>();
        let words = state.store.library_key()?.to_words();
        Some(LibraryKeySheet {
            mode,
            words: Some(words),
            already_saved: key_saved(cx),
            copied: false,
        })
    }

    /// A sheet that shows `words`, for the snapshots: a demo key, never a
    /// real one.
    #[cfg(test)]
    pub(crate) fn with_words_for_test(mode: Mode, words: &str, saved: bool) -> LibraryKeySheet {
        LibraryKeySheet {
            mode,
            words: Some(Zeroizing::new(words.to_owned())),
            already_saved: saved,
            copied: false,
        }
    }

    /// Wipes the words now, without waiting for the sheet to be dropped.
    fn wipe(&mut self) {
        self.words = None;
    }

    #[cfg(test)]
    pub(crate) fn word_count_for_test(&self) -> usize {
        self.words.as_ref().map_or(0, |words| numbered(words).len())
    }

    fn copy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(words) = &self.words else {
            return;
        };
        if crate::clipboard::copy_secret(words, cx) {
            self.copied = true;
            window.push_notification(Notification::info(COPIED), cx);
        } else {
            window.push_notification(Notification::error("Couldn't copy the key."), cx);
        }
        cx.notify();
    }
}

impl Render for LibraryKeySheet {
    // no-prop-test: drawing; `numbered` is tested.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mono = crate::theme::mono_font(cx);
        let rows: Vec<Vec<(usize, String)>> = match &self.words {
            Some(words) => numbered(words)
                .chunks(COLUMNS)
                .map(|row| row.iter().map(|(n, w)| (*n, (*w).to_owned())).collect())
                .collect(),
            None => Vec::new(),
        };
        v_flex()
            .gap(px(14.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(self.mode.body()),
            )
            .child(
                v_flex()
                    .id("library-key-words")
                    .debug_selector(|| "library-key-words".into())
                    .gap(px(6.))
                    .p(px(12.))
                    .rounded(px(8.))
                    .bg(colors.fill)
                    .text_size(px(13.))
                    .text_color(colors.text)
                    .when_some(mono, |el, family| el.font_family(family))
                    .children(rows.into_iter().map(|row| {
                        h_flex()
                            .gap(px(8.))
                            .children(row.into_iter().map(|(n, word)| {
                                h_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .w(px(20.))
                                            .flex_none()
                                            .text_right()
                                            .text_color(colors.text_secondary)
                                            .child(n.to_string()),
                                    )
                                    .child(div().child(word))
                            }))
                    })),
            )
    }
}

/// The sheet's buttons. They read the sheet, so the sheet never reads
/// itself.
struct KeyFooter {
    sheet: Entity<LibraryKeySheet>,
    _observing: Subscription,
}

// no-prop-test: drawing and clicks; what saving does is `mark_saved`.
impl Render for KeyFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self.sheet.read(cx);
        let (mode, saved, copied) = (sheet.mode, sheet.already_saved, sheet.copied);
        let target = self.sheet.clone();
        let copy_target = self.sheet.clone();
        let saved_target = self.sheet.clone();
        let mut buttons = h_flex().w_full().gap_2().justify_end().child(
            Button::new("library-key-copy")
                .debug_selector(|| "library-key-copy".into())
                .outline()
                .label(if copied { "Copied" } else { "Copy" })
                .on_click(move |_, window, cx| {
                    copy_target.update(cx, |sheet, cx| sheet.copy(window, cx))
                }),
        );
        if saved {
            buttons = buttons.child(
                Button::new("library-key-done")
                    .debug_selector(|| "library-key-done".into())
                    .primary()
                    .label("Done")
                    .on_click(move |_, window, cx| {
                        target.update(cx, |sheet, _| sheet.wipe());
                        window.close_dialog(cx);
                    }),
            );
        } else {
            // Until the person says they saved it, the sheet comes back at
            // each launch; closing it is "later".
            buttons = buttons
                .child(
                    Button::new("library-key-later")
                        .debug_selector(|| "library-key-later".into())
                        .label(if mode == Mode::Save { "Later" } else { "Close" })
                        .on_click(move |_, window, cx| {
                            target.update(cx, |sheet, _| sheet.wipe());
                            window.close_dialog(cx);
                        }),
                )
                .child(
                    Button::new("library-key-saved")
                        .debug_selector(|| "library-key-saved".into())
                        .primary()
                        .label("I've saved it")
                        .on_click(move |_, window, cx| {
                            saved_target.update(cx, |sheet, _| sheet.wipe());
                            mark_saved(cx);
                            window.close_dialog(cx);
                        }),
                );
        }
        buttons
    }
}

/// Records that the person saved the key. The save sheet stops coming back,
/// and the encrypted-library notice goes.
pub fn mark_saved(cx: &mut App) {
    let Some(mark) = cx
        .global::<AppState>()
        .store
        .library_key()
        .map(kanso_store::LibraryKey::fingerprint)
    else {
        return;
    };
    AppState::update_config(cx, |config| config.saved_library_key = Some(mark));
    cx.refresh_windows();
}

/// Whether the person said they saved the library's key, and it is still
/// that key.
pub fn key_saved(cx: &App) -> bool {
    cx.try_global::<AppState>().is_some_and(|state| {
        crate::library_key_model::key_saved(
            state.config.saved_library_key.as_deref(),
            state.store.library_key(),
        )
    })
}

fn open(sheet: Entity<LibraryKeySheet>, window: &mut Window, cx: &mut App) {
    let mode = sheet.read(cx).mode;
    let footer = cx.new(|cx| KeyFooter {
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
        sheet: sheet.clone(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(mode.title())
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(footer.clone())
    });
}

/// Opens the sheet for the library's key. Does nothing for a plain library.
/// Returns it, held weakly: the dialog owns it.
pub fn open_sheet(
    mode: Mode,
    window: &mut Window,
    cx: &mut App,
) -> Option<WeakEntity<LibraryKeySheet>> {
    let sheet = LibraryKeySheet::new(mode, cx)?;
    let sheet = cx.new(|_| sheet);
    let weak = sheet.downgrade();
    open(sheet, window, cx);
    Some(weak)
}

/// The sheet showing `words`, for the snapshots.
#[cfg(test)]
pub(crate) fn open_sheet_for_test(
    mode: Mode,
    words: &str,
    saved: bool,
    window: &mut Window,
    cx: &mut App,
) -> WeakEntity<LibraryKeySheet> {
    let sheet = cx.new(|_| LibraryKeySheet::with_words_for_test(mode, words, saved));
    let weak = sheet.downgrade();
    open(sheet, window, cx);
    weak
}

// MARK: At launch

/// How long to wait for the account to be read back before asking whether
/// the library is signed in.
const SETTLE_POLL: std::time::Duration = std::time::Duration::from_millis(250);
const SETTLE_MAX: std::time::Duration = std::time::Duration::from_secs(10);

/// At launch, once the account has been read back: asks the person to save
/// the key when the library is encrypted, sync doesn't hold its notes, and
/// the key hasn't been saved. It does not interrupt a dialog already open.
pub fn offer_at_launch(window: &mut Window, cx: &mut App) {
    // A restore the last run staged was applied (or not) before the library
    // opened; say so.
    match crate::library_open::take_restore_outcome() {
        Some(Ok(())) => window.push_notification(Notification::success(APPLIED), cx),
        Some(Err((why, again))) => {
            window.push_notification(Notification::error(not_applied(&why, again)), cx);
        }
        None => {}
    }
    window
        .spawn(cx, async move |cx| {
            let started = std::time::Instant::now();
            loop {
                let settled = cx
                    .update(|_, cx| {
                        cx.try_global::<services::Services>()
                            .is_none_or(|services| !services.restoring)
                    })
                    .unwrap_or(true);
                if settled || started.elapsed() >= SETTLE_MAX {
                    break;
                }
                cx.background_executor().timer(SETTLE_POLL).await;
            }
            cx.update(offer_now).ok();
        })
        .detach();
}

fn offer_now(window: &mut Window, cx: &mut App) {
    if window.has_active_dialog(cx) || !wants_save_sheet(cx) {
        return;
    }
    open_sheet(Mode::Save, window, cx);
}

fn wants_save_sheet(cx: &App) -> bool {
    let Some(state) = cx.try_global::<AppState>() else {
        return false;
    };
    let signed_in = cx
        .try_global::<services::Services>()
        .is_some_and(|services| services.signed_in() || services.has_locked_session())
        || state.config.sync.account.is_some();
    // A count that can't be read is taken as notes not sent: ask.
    let never_sent = state.store.never_sent_count().unwrap_or(usize::MAX);
    offer_save_sheet(
        state.store.is_encrypted(),
        key_saved(cx),
        sync_holds_notes(signed_in, never_sent),
    )
}

// MARK: Show key…

/// Show key…: asks for Touch ID or the account password (macOS), Windows
/// Hello, or, where the system has neither (Linux), for a confirmation,
/// then opens the sheet.
pub fn show_key(window: &mut Window, cx: &mut App) {
    if !cx.global::<AppState>().store.is_encrypted() {
        return;
    }
    let reason = "show your library key";
    let Some(answer) = crate::touch_id::confirm_owner(reason, cx) else {
        dialogs::confirm(
            "Show your library key?",
            CONFIRM_BODY,
            "Show key",
            window,
            cx,
            |window, cx| {
                open_sheet(Mode::Show, window, cx);
            },
        );
        return;
    };
    window
        .spawn(cx, async move |cx| {
            let confirmed = answer.recv().await.unwrap_or(false);
            cx.update(|window, cx| {
                if confirmed {
                    open_sheet(Mode::Show, window, cx);
                } else {
                    window.push_notification(Notification::warning(NOT_CONFIRMED), cx);
                }
            })
            .ok();
        })
        .detach();
}

// MARK: The Backups rows

/// The rows Settings › General › Data adds under Backups: the library key
/// (when the library has one) and Restore from backup….
pub fn backup_rows(cx: &mut Context<crate::settings::SettingsView>) -> Vec<Row> {
    let state = cx.global::<AppState>();
    let encrypted = state.store.is_encrypted();
    let saved = key_saved(cx);
    let waiting = kanso_store::restore_staged(&crate::paths::db_path(&state.data_dir));
    let mut rows = Vec::new();
    if encrypted {
        rows.push(
            Row::new(Lucide::KeyRound, "Library key")
                .detail(key_row_detail(saved))
                .trailing(
                    Button::new("show-library-key")
                        .outline()
                        .small()
                        .label("Show key\u{2026}")
                        .debug_selector(|| "show-library-key".into())
                        .on_click(|_, window, cx| show_key(window, cx)),
                ),
        );
        // A restore staged by mistake can be called off before the launch
        // that would apply it.
        if waiting {
            rows.push(
                Row::new(Lucide::DatabaseBackup, "Restore waiting")
                    .detail(WAITING)
                    .trailing(
                        Button::new("cancel-restore")
                            .outline()
                            .small()
                            .label("Cancel restore")
                            .debug_selector(|| "cancel-restore".into())
                            .on_click(|_, _, cx| {
                                let db = crate::paths::db_path(&cx.global::<AppState>().data_dir);
                                kanso_store::cancel_staged_restore(&db);
                                cx.refresh_windows();
                            }),
                    ),
            );
        }
        rows.push(
            Row::new(Lucide::DatabaseBackup, "Restore from backup")
                .detail("Replaces this library with a backup. A backup from another computer needs its 24 words. The library you have now is kept as a backup.")
                .trailing(
                    Button::new("restore-backup")
                        .outline()
                        .small()
                        .label("Restore\u{2026}")
                        .debug_selector(|| "restore-backup".into())
                        .on_click(|_, window, cx| crate::restore_sheet::start(window, cx)),
                ),
        );
    }
    rows
}
