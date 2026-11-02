//! Settings › General's Quick capture and Daily notes sections: the
//! system-wide shortcut (recorded by pressing it), whether it is on, what
//! to do where it can't work, and the template new daily notes start from.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Origin;

use crate::hotkey::{self, Shortcut, Status};
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The shortcut's control: its keys, and Change, which takes the next
/// shortcut pressed.
pub struct ShortcutRecorder {
    /// While waiting for a shortcut: what catches the keys before the
    /// app's own shortcuts do.
    recording: Option<Subscription>,
    /// The last thing pressed while recording that can't be a shortcut.
    hint: Option<&'static str>,
    /// Records the Voice capture shortcut (unset by default) instead of the
    /// Quick note one.
    voice: bool,
}

impl ShortcutRecorder {
    pub fn new() -> Self {
        ShortcutRecorder {
            recording: None,
            hint: None,
            voice: false,
        }
    }

    /// The recorder of the Voice capture shortcut.
    pub fn voice() -> Self {
        ShortcutRecorder {
            voice: true,
            ..ShortcutRecorder::new()
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        // The shortcut itself would go to the system, not here.
        hotkey::suspend(cx);
        let view = cx.entity().downgrade();
        self.recording = Some(cx.intercept_keystrokes(move |event, _, cx| {
            let keystroke = event.keystroke.clone();
            cx.stop_propagation();
            view.update(cx, |this, cx| this.pressed(&keystroke, cx))
                .ok();
        }));
        self.hint = None;
        cx.notify();
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        self.recording = None;
        self.hint = None;
        hotkey::apply(cx);
        cx.notify();
    }

    fn pressed(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        if keystroke.key == "escape" && !keystroke.modifiers.modified() {
            return self.stop(cx);
        }
        match Shortcut::from_keystroke(keystroke) {
            Some(shortcut) => {
                let voice = self.voice;
                AppState::update_config(cx, |config| {
                    if voice {
                        config.capture.voice_shortcut = shortcut.text();
                    } else {
                        config.capture.shortcut = shortcut.text();
                    }
                    config.capture.enabled = true;
                });
                self.stop(cx);
            }
            None => {
                self.hint = Some(if cfg!(target_os = "macos") {
                    "Hold ⌃, ⌥ or ⌘ with a key"
                } else {
                    "Hold Ctrl, Alt or Super with a key"
                });
                cx.notify();
            }
        }
    }
}

impl Default for ShortcutRecorder {
    fn default() -> Self {
        ShortcutRecorder::new()
    }
}

impl Render for ShortcutRecorder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let config = cx.global::<AppState>().config.capture.clone();
        let written = if self.voice {
            &config.voice_shortcut
        } else {
            &config.shortcut
        };
        let label = Shortcut::parse(written).map_or_else(|| "None".to_owned(), |s| s.label());
        let recording = self.is_recording();
        let voice = self.voice;
        let keys = div()
            .debug_selector(|| "capture-shortcut".into())
            .px(px(8.))
            .h(px(24.))
            .flex()
            .items_center()
            .rounded(px(6.))
            .border_1()
            .border_color(if recording {
                colors.accent
            } else {
                colors.divider
            })
            .bg(colors.fill)
            .text_size(px(12.5))
            .text_color(if recording {
                colors.text_secondary
            } else {
                colors.text
            })
            .child(if recording {
                self.hint.unwrap_or("Press a shortcut…").to_owned()
            } else {
                label
            });
        h_flex()
            .gap(px(8.))
            .child(keys)
            .child(
                Button::new("capture-shortcut-change")
                    .outline()
                    .small()
                    .label(if recording { "Cancel" } else { "Change…" })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.is_recording() {
                            this.stop(cx)
                        } else {
                            this.start(cx)
                        }
                    })),
            )
            .when(
                voice && !config.voice_shortcut.is_empty() && !recording,
                |row| {
                    row.child(
                        Button::new("voice-shortcut-clear")
                            .ghost()
                            .small()
                            .label("Clear")
                            .on_click(|_, _, cx| {
                                AppState::update_config(cx, |config| {
                                    config.capture.voice_shortcut.clear()
                                });
                                hotkey::apply(cx);
                            }),
                    )
                },
            )
            .when(!voice, |row| {
                row.child(
                    div().debug_selector(|| "capture-enabled".into()).child(
                        Switch::new("capture-enabled")
                            .checked(config.enabled)
                            .on_click(|on, _, cx| {
                                AppState::update_config(cx, |config| config.capture.enabled = *on);
                                hotkey::apply(cx);
                            }),
                    ),
                )
            })
    }
}

/// What the Quick note row says under its title. Pure.
pub fn capture_detail(status: &Status, label: &str) -> String {
    let opens = "Opens a small window over any app to jot something down; it saves to the Inbox.";
    match status {
        Status::Unavailable(why) => hotkey::unavailable_note(why, label),
        Status::Off => {
            format!("{opens} The shortcut is off; Quick note in the palette still opens it.")
        }
        Status::On(_) => opens.to_owned(),
    }
}

/// What the Voice capture row says. Pure.
pub fn voice_capture_detail(status: &Status, label: &str) -> String {
    let opens = "A second shortcut that opens Quick note already listening: speak, and your words are written. Leave it unset if you only want the mic button.";
    match status {
        Status::Unavailable(why) => hotkey::unavailable_note(why, label),
        Status::Off | Status::On(_) => opens.to_owned(),
    }
}

pub fn capture_section(
    recorder: &Entity<ShortcutRecorder>,
    voice_recorder: &Entity<ShortcutRecorder>,
    cx: &App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let config = &cx.global::<AppState>().config.capture;
    let label =
        Shortcut::parse(&config.shortcut).map_or_else(|| "The shortcut".to_owned(), |s| s.label());
    let detail = capture_detail(&hotkey::status(cx), &label);
    let voice_label = Shortcut::parse(&config.voice_shortcut)
        .map_or_else(|| "The shortcut".to_owned(), |s| s.label());
    let voice_detail = voice_capture_detail(&hotkey::voice_status_now(cx), &voice_label);
    section(
        "Quick capture",
        vec![
            Row::new(Lucide::Zap, "Quick note shortcut")
                .detail(detail)
                .trailing(recorder.clone()),
            Row::new(Lucide::Mic, "Voice capture shortcut")
                .detail(voice_detail)
                .trailing(voice_recorder.clone()),
        ],
        colors,
    )
}

/// Daily notes: the template ⌘D's new notes start from.
pub fn journal_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let store = cx.global::<AppState>().store.clone();
    let templates = store.templates().unwrap_or_default();
    let current = store.daily_template().ok().flatten();
    let current_title = current
        .as_ref()
        .and_then(|id| templates.iter().find(|(t, _)| t == id))
        .map(|(_, title)| title.clone())
        .map(|title| {
            if title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                title
            }
        });
    let no_templates = templates.is_empty();
    let menu = Button::new("daily-template")
        .debug_selector(|| "daily-template".into())
        .outline()
        .small()
        .label(current_title.unwrap_or_else(|| "None".to_owned()))
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            let set = |id: Option<String>| {
                let store = store.clone();
                move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Err(err) = store.set_daily_template(id.as_deref(), Origin::Ui) {
                        window.push_notification(
                            Notification::error(format!("Couldn't set the daily template: {err}")),
                            cx,
                        );
                    }
                    window.refresh();
                }
            };
            menu = menu.item(
                PopupMenuItem::new("None")
                    .checked(current.is_none())
                    .on_click(set(None)),
            );
            if !templates.is_empty() {
                menu = menu.separator();
            }
            for (id, title) in &templates {
                let title = if title.trim().is_empty() {
                    "Untitled".to_owned()
                } else {
                    title.clone()
                };
                menu = menu.item(
                    PopupMenuItem::new(title)
                        .checked(current.as_deref() == Some(id.as_str()))
                        .on_click(set(Some(id.clone()))),
                );
            }
            menu
        });
    let detail = if no_templates {
        "New daily notes start empty. Notes in the Templates notebook can be chosen here; New template in the palette makes one."
    } else {
        "New daily notes start from this template, with {{date}}, {{time}} and {{cursor}} filled in."
    };
    section(
        "Daily notes",
        vec![
            Row::new(Lucide::CalendarDays, "Daily template")
                .detail(detail)
                .trailing(menu),
        ],
        colors,
    )
}

#[cfg(test)]
mod tests {
    use super::capture_detail;
    use crate::hotkey::{Status, Unavailable};

    // no-prop-test: three fixed sentences.
    #[test]
    fn voice_details() {
        assert!(super::voice_capture_detail(&Status::Off, "x").contains("listening"));
        assert!(
            super::voice_capture_detail(&Status::Unavailable(Unavailable::Invalid), "x")
                .contains("can't be used")
        );
    }

    #[test]
    fn details() {
        assert!(capture_detail(&Status::Off, "x").contains("off"));
        assert!(
            capture_detail(&Status::Unavailable(Unavailable::Wayland), "x")
                .contains("kanso capture")
        );
        assert!(
            capture_detail(
                &Status::Unavailable(Unavailable::Taken("e".into())),
                "⌃⌥Space"
            )
            .starts_with("⌃⌥Space")
        );
    }
}
