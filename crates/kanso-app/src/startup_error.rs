//! The window for a library that won't open (issue #243, step 10).
//!
//! Launched from the Finder or the dock, Kanso has no terminal, so an error
//! that only reaches standard error looks like an app that never opens. The
//! error happens before GPUI runs, and GPUI can't be started twice in one
//! process, so the window is a second, short-lived process, like the
//! encryption progress window (`encrypt_progress`): `kanso startup-error`
//! reads one line of JSON from its standard input (a [`Screen`]: only text,
//! never a key) and exits with the button the person chose. The app, which
//! holds no window yet, starts again when that is Try again.

use std::io::{Read as _, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::startup_error_model::{Action, Failure, Screen, describe};
use crate::theme::Colors;

/// The window's width.
pub const WIDTH: f32 = 560.;

/// The exit code of the window's process for Try again. Anything else is
/// Quit (or a closed window).
const RETRY_CODE: i32 = 10;
const QUIT_CODE: i32 = 1;

/// What the person chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    TryAgain,
    Quit,
}

/// A rough height for `screen`'s window: its text wraps at about this many
/// characters per line.
pub fn height_for(screen: &Screen) -> f32 {
    fn lines(text: &str, per_line: usize) -> f32 {
        text.chars().count().div_ceil(per_line).max(1) as f32
    }
    let steps: f32 = screen
        .steps
        .iter()
        .map(|step| lines(step, 62) * 19. + 10.)
        .sum();
    let height = 48. + 24. + lines(&screen.message, 66) * 20. + 14. + steps + 14. + 36.;
    height.clamp(260., 640.)
}

/// The window's content: the title, what happened, and the numbered steps.
/// The window and the snapshots draw this.
// no-prop-test: drawing; the snapshots show it, `describe` is tested.
pub fn body(screen: &Screen, colors: &Colors) -> impl IntoElement {
    let steps = screen.steps.iter().enumerate().map(|(ix, step)| {
        h_flex()
            .gap(px(10.))
            .items_start()
            .child(
                div()
                    .flex_none()
                    .w(px(20.))
                    .h(px(20.))
                    .rounded_full()
                    .bg(colors.fill)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.5))
                    .font_medium()
                    .text_color(colors.text_secondary)
                    .child((ix + 1).to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pt(px(1.))
                    .text_size(px(13.5))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(step.clone()),
            )
    });
    v_flex()
        .debug_selector(|| "startup-error".into())
        .w_full()
        .gap(px(14.))
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(
                    Icon::new(Lucide::TriangleAlert)
                        .size(px(16.))
                        .flex_none()
                        .text_color(colors.warning),
                )
                .child(
                    div()
                        .debug_selector(|| "startup-error-title".into())
                        .text_size(px(15.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child(screen.title.clone()),
                ),
        )
        .child(
            div()
                .debug_selector(|| "startup-error-message".into())
                .text_size(px(13.5))
                .line_height(px(20.))
                .text_color(colors.text_secondary)
                .child(screen.message.clone()),
        )
        .child(
            v_flex()
                .debug_selector(|| "startup-error-steps".into())
                .gap(px(10.))
                .children(steps),
        )
}

/// What a button does when chosen.
pub type Handler = Rc<dyn Fn(Action, &mut Window, &mut App)>;

/// The label of a button.
fn label(action: Action, copied: bool) -> &'static str {
    match action {
        Action::TryAgain => "Try again",
        Action::ShowFolder => "Show data folder",
        Action::CopyDetails if copied => "Copied",
        Action::CopyDetails => "Copy details",
        Action::Quit => "Quit",
    }
}

/// The buttons, in the order they sit: the way out on the left of the row's
/// end, Try again last and primary (or Quit, when there is nothing to try).
// no-prop-test: drawing; the snapshots show it, `describe` decides which.
pub fn buttons(screen: &Screen, copied: bool, on: Handler) -> impl IntoElement {
    let retry = screen.retry();
    let button = |action: Action| {
        let on = on.clone();
        let id = format!("startup-error-{action:?}");
        let base = Button::new(SharedString::from(id.clone()))
            .debug_selector(move || id.clone())
            .label(label(action, copied))
            .on_click(move |_, window, cx| on(action, window, cx));
        match action {
            Action::TryAgain => base.primary(),
            Action::Quit if !retry => base.primary(),
            _ => base.ghost(),
        }
    };
    h_flex()
        .w_full()
        .justify_end()
        .gap(px(8.))
        .children(
            screen
                .actions
                .iter()
                .copied()
                .filter(|action| *action != Action::TryAgain)
                .map(button),
        )
        .when(retry, |row| row.child(button(Action::TryAgain)))
}

/// The window's view.
struct ErrorView {
    screen: Screen,
    copied: bool,
}

impl Render for ErrorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let weak = cx.weak_entity();
        let details = self.screen.details.clone();
        let folder = self.screen.folder.clone();
        let on: Handler = Rc::new(move |action, _, cx| {
            match action {
                Action::TryAgain => finish(RETRY_CODE, cx),
                Action::Quit => finish(QUIT_CODE, cx),
                Action::ShowFolder => reveal(Path::new(&folder)),
                Action::CopyDetails => {
                    cx.write_to_clipboard(ClipboardItem::new_string(details.clone()));
                    weak.update(cx, |view, cx| {
                        view.copied = true;
                        cx.notify();
                    })
                    .ok();
                }
            };
        });
        v_flex()
            .size_full()
            .bg(colors.editor)
            .p(px(24.))
            .gap(px(16.))
            .child(div().flex_1().min_h_0().child(body(&self.screen, colors)))
            .child(buttons(&self.screen, self.copied, on))
    }
}

/// What the process exits with, set when a button is chosen.
static EXIT: AtomicI32 = AtomicI32::new(QUIT_CODE);

fn finish(code: i32, cx: &mut App) {
    EXIT.store(code, Ordering::SeqCst);
    cx.quit();
}

/// Opens `folder` in the system's file manager, or the nearest folder above
/// it that exists.
pub fn reveal(folder: &Path) {
    let Some(target) = folder.ancestors().find(|path| path.is_dir()) else {
        return;
    };
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    let _ = Command::new(program)
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// The second process: reads the screen from standard input, shows it, and
/// exits with the choice.
// no-prop-test: a window around `body` and the tested `describe`.
pub fn run_child() -> anyhow::Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;
    let screen: Screen = serde_json::from_str(text.trim())?;
    let height = height_for(&screen);
    application().run(move |cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(WIDTH), px(height)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(crate::flavour::CURRENT.name.into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            is_resizable: false,
            is_minimizable: false,
            focus: true,
            show: true,
            #[cfg(target_os = "linux")]
            app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
            ..Default::default()
        };
        let opened = gpui_kit::open_window(options, cx, |window, cx| {
            gpui_kit::component::Theme::sync_system_appearance(Some(window), cx);
            // Closing the window is Quit.
            window.on_window_should_close(cx, |_, cx| {
                finish(QUIT_CODE, cx);
                true
            });
            cx.new(|_| ErrorView {
                screen,
                copied: false,
            })
        });
        if opened.is_err() {
            cx.quit();
            return;
        }
        cx.activate(true);
    });
    std::process::exit(EXIT.load(Ordering::SeqCst));
}

/// Shows the window for `failure` and waits for a choice. `None` when no
/// window could be shown; the caller then has only standard error.
// no-prop-test: starts a process; `describe` and the protocol's JSON are tested.
pub fn show(failure: &Failure, data_dir: &Path) -> Option<Choice> {
    show_screen(&describe(failure, data_dir))
}

fn show_screen(screen: &Screen) -> Option<Choice> {
    let exe = std::env::current_exe().ok()?;
    // The window's process gets the screen's text and no argument but its
    // command's name.
    let mut child = Command::new(exe)
        .arg("startup-error")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    let json = serde_json::to_string(screen).ok()?;
    let wrote = writeln!(stdin, "{json}").is_ok();
    drop(stdin);
    let status = child.wait().ok()?;
    if !wrote {
        return None;
    }
    // A process that died before it drew anything (no display) is no
    // answer: the caller falls back to standard error.
    match status.code() {
        Some(RETRY_CODE) => Some(Choice::TryAgain),
        Some(QUIT_CODE) => Some(Choice::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Failure, describe, height_for};

    /// The window's size grows with its text and stays on screen.
    #[test]
    fn the_window_fits_its_text() {
        let short = describe(&Failure::Locked, Path::new("/k"));
        let mut long = short.clone();
        long.steps = vec!["word ".repeat(200); 6];
        assert!(height_for(&long) > height_for(&short));
        assert!(height_for(&long) <= 640.);
        assert!(height_for(&short) >= 260.);
    }
}
