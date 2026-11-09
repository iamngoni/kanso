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

use gpui_component::scroll::ScrollableElement as _;
use std::io::{Read as _, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Textarea, TextareaState};
use gpui_kit::component::{Disableable as _, Icon, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::startup_error_model::{Action, Failure, Screen, describe};
use crate::startup_update::{Backend, Phase, Real, Recovery, Way};
use crate::theme::Colors;

/// The window's width.
pub const WIDTH: f32 = 560.;

/// The exit code of the window's process for Try again, and for Quit (or a
/// closed window). The choice is also its one line of standard output: on
/// macOS, quitting GPUI ends the process through AppKit with code 0 before
/// `run` returns, so neither is left to that.
const RETRY_CODE: i32 = 10;
const QUIT_CODE: i32 = 1;

/// What the person chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    TryAgain,
    Quit,
}

impl Choice {
    /// The line the window's process writes for this choice.
    fn line(self) -> &'static str {
        match self {
            Choice::TryAgain => "try-again",
            Choice::Quit => "quit",
        }
    }

    fn code(self) -> i32 {
        match self {
            Choice::TryAgain => RETRY_CODE,
            Choice::Quit => QUIT_CODE,
        }
    }
}

/// The choice the window's process made, from what it wrote and its exit
/// code: the line when there is one, else the code. `None` when it died
/// before anyone chose (no display, a crash).
pub fn answer(stdout: &str, code: Option<i32>) -> Option<Choice> {
    let said = stdout.lines().map(str::trim).find_map(|line| {
        [Choice::TryAgain, Choice::Quit]
            .into_iter()
            .find(|choice| choice.line() == line)
    });
    said.or(match code {
        Some(RETRY_CODE) => Some(Choice::TryAgain),
        Some(QUIT_CODE) => Some(Choice::Quit),
        _ => None,
    })
}

/// A rough height for `screen`'s window: its text wraps at about this many
/// characters per line. Counted low, since a long path breaks only at its
/// slashes; text that still doesn't fit scrolls above the buttons.
pub fn height_for(screen: &Screen) -> f32 {
    fn lines(text: &str, per_line: usize) -> f32 {
        text.chars().count().div_ceil(per_line).max(1) as f32
    }
    let steps: f32 = screen
        .steps
        .iter()
        .map(|step| lines(step, 56) * 19. + 10.)
        .sum();
    let height = 48. + 24. + lines(&screen.message, 54) * 20. + 14. + steps + 16. + 36. + 12.;
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

/// Where "Send feedback" is.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Feedback {
    Idle,
    /// The text box is open.
    Writing,
    Sending,
    Sent,
    Failed(String),
}

/// What the updater's thread tells the window.
enum Install {
    Progress(u64, u64),
    Done(Result<(), crate::updates::Problem>),
}

/// The window's view.
struct ErrorView {
    screen: Screen,
    copied: bool,
    /// `None` when this build never updates.
    updater: Option<Arc<Real>>,
    recovery: Option<Recovery>,
    apt_copied: bool,
    feedback: Feedback,
    words: Entity<TextareaState>,
}

impl ErrorView {
    fn new(screen: Screen, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let updater = Real::new(&screen.folder).map(Arc::new);
        let recovery = updater.as_ref().map(|real| {
            Recovery::new(
                Way::of(&real.install),
                screen.bug,
                &crate::updates::current_version(),
            )
        });
        let words = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder(crate::startup_feedback::PLACEHOLDER)
        });
        let mut view = ErrorView {
            screen,
            copied: false,
            updater,
            recovery,
            apt_copied: false,
            feedback: Feedback::Idle,
            words,
        };
        view.check(cx);
        view
    }

    /// Checks for an update (also Retry).
    fn check(&mut self, cx: &mut Context<Self>) {
        let (Some(real), Some(recovery)) = (self.updater.clone(), self.recovery.as_mut()) else {
            return;
        };
        recovery.begin_check();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { real.check() })
                .await;
            this.update(cx, |view, cx| {
                if let Some(recovery) = view.recovery.as_mut() {
                    recovery.checked(result);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// "Update now": downloads, installs and restarts, as Restart now does.
    fn update_now(&mut self, cx: &mut Context<Self>) {
        let (Some(real), Some(recovery)) = (self.updater.clone(), self.recovery.as_mut()) else {
            return;
        };
        let Some(update) = recovery.update_now() else {
            return;
        };
        cx.notify();
        let (tx, rx) = async_channel::unbounded();
        let spawned = std::thread::Builder::new()
            .name("kanso-startup-update".into())
            .spawn(move || {
                let progress_tx = tx.clone();
                let mut progress = |done: u64, total: u64| {
                    progress_tx.try_send(Install::Progress(done, total)).ok();
                };
                let result = real.install(&update, &mut progress, &AtomicBool::new(false));
                tx.try_send(Install::Done(result)).ok();
            });
        if spawned.is_err() {
            recovery.installed(Err(crate::updates::Problem::Other));
            return;
        }
        cx.spawn(async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let done = matches!(message, Install::Done(_));
                let restarted = this
                    .update(cx, |view, cx| {
                        let mut restarted = false;
                        if let Some(recovery) = view.recovery.as_mut() {
                            match message {
                                Install::Progress(done, total) => recovery.progress(done, total),
                                Install::Done(Ok(())) => restarted = true,
                                Install::Done(result) => recovery.installed(result),
                            }
                        }
                        cx.notify();
                        restarted
                    })
                    .unwrap_or(false);
                if restarted {
                    // The new Kanso is on its way; this window's job is done.
                    finish(Choice::Quit);
                }
                if done {
                    break;
                }
            }
        })
        .detach();
    }

    fn send_feedback(&mut self, cx: &mut Context<Self>) {
        if self.feedback == Feedback::Sending {
            return;
        }
        self.feedback = Feedback::Sending;
        cx.notify();
        let own = self.words.read(cx).value().to_string();
        let details = self.screen.details.clone();
        let folder = self.screen.folder.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let app = crate::startup_feedback::app_info(Path::new(&folder));
                    crate::startup_feedback::send(
                        kanso_sync::feedback::HttpTransport::new(),
                        &crate::flavour::CURRENT.keychain(),
                        &details,
                        &own,
                        &app,
                    )
                })
                .await;
            this.update(cx, |view, cx| {
                view.feedback = match result {
                    Ok(()) => Feedback::Sent,
                    Err(why) => Feedback::Failed(why),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The update section: what the check found and what to do about it.
    fn update_section(&self, colors: &Colors, cx: &mut Context<Self>) -> Option<AnyElement> {
        let recovery = self.recovery.as_ref()?;
        let button = |id: &'static str, text: &'static str| {
            Button::new(SharedString::from(id))
                .debug_selector(move || id.into())
                .label(text)
        };
        let mut row = h_flex().gap(px(8.)).justify_end();
        match (&recovery.phase, recovery.way) {
            (Phase::Available(_), Way::Replace) => {
                row = row.child(
                    button("startup-update-now", "Update now")
                        .primary()
                        .on_click(cx.listener(|view, _, _, cx| view.update_now(cx))),
                );
            }
            (Phase::Available(_), Way::Apt) => {
                let copied = self.apt_copied;
                row = row.child(
                    button(
                        "startup-update-apt",
                        if copied { "Copied" } else { "Copy command" },
                    )
                    .primary()
                    .on_click(cx.listener(|view, _, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            kanso_update::install::APT_UPGRADE.to_owned(),
                        ));
                        view.apt_copied = true;
                        cx.notify();
                    })),
                );
            }
            (Phase::Available(update), Way::Page) => {
                let url = update.release_url.clone();
                let policy = self.updater.as_ref().map(|real| real.policy);
                row = row.child(
                    button("startup-update-page", "Open download page")
                        .primary()
                        .on_click(move |_, _, cx| {
                            if policy.is_some_and(|p| p.check_release_page(&url).is_ok()) {
                                cx.open_url(&url);
                            }
                        }),
                );
            }
            _ if recovery.can_retry() => {
                row = row.child(
                    button("startup-update-retry", "Retry")
                        .primary()
                        .on_click(cx.listener(|view, _, _, cx| view.check(cx))),
                );
            }
            _ => {}
        }
        let apt = matches!(
            (&recovery.phase, recovery.way),
            (Phase::Available(_), Way::Apt)
        )
        .then(|| {
            div()
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(format!(
                    "Run “{}” in a terminal.",
                    kanso_update::install::APT_UPGRADE
                ))
        });
        Some(
            v_flex()
                .debug_selector(|| "startup-update".into())
                .gap(px(8.))
                .child(
                    div()
                        .debug_selector(|| "startup-update-headline".into())
                        .text_size(px(13.5))
                        .line_height(px(20.))
                        .font_medium()
                        .text_color(colors.text)
                        .child(recovery.headline()),
                )
                .children(apt)
                .child(row)
                .into_any_element(),
        )
    }

    /// The feedback section.
    fn feedback_section(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let mut col = v_flex()
            .debug_selector(|| "startup-feedback".into())
            .gap(px(8.));
        match &self.feedback {
            Feedback::Sent => {
                return col
                    .child(
                        div()
                            .debug_selector(|| "startup-feedback-sent".into())
                            .text_size(px(13.5))
                            .text_color(colors.text)
                            .child(crate::startup_feedback::SENT),
                    )
                    .into_any_element();
            }
            Feedback::Idle => {
                return col
                    .child(
                        h_flex().justify_end().child(
                            Button::new("startup-feedback-open")
                                .debug_selector(|| "startup-feedback-open".into())
                                .ghost()
                                .label("Send feedback")
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.feedback = Feedback::Writing;
                                    cx.notify();
                                })),
                        ),
                    )
                    .into_any_element();
            }
            _ => {}
        }
        let sending = self.feedback == Feedback::Sending;
        col = col.child(Textarea::new(&self.words).h(px(80.)).disabled(sending));
        if let Feedback::Failed(why) = &self.feedback {
            col = col.child(
                div()
                    .text_size(px(13.))
                    .text_color(colors.warning)
                    .child(why.clone()),
            );
        }
        let label = match &self.feedback {
            Feedback::Failed(_) => "Retry",
            Feedback::Sending => "Sending…",
            _ => "Send",
        };
        col.child(
            h_flex().justify_end().child(
                Button::new("startup-feedback-send")
                    .debug_selector(|| "startup-feedback-send".into())
                    .primary()
                    .label(label)
                    .disabled(sending)
                    .on_click(cx.listener(|view, _, _, cx| view.send_feedback(cx))),
            ),
        )
        .into_any_element()
    }
}

impl Render for ErrorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let weak = cx.weak_entity();
        let details = self.screen.details.clone();
        let folder = self.screen.folder.clone();
        let on: Handler = Rc::new(move |action, _, cx| {
            match action {
                Action::TryAgain => finish(Choice::TryAgain),
                Action::Quit => finish(Choice::Quit),
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
        let update = self.update_section(colors, cx);
        let feedback = self.feedback_section(colors, cx);
        v_flex()
            .size_full()
            .bg(colors.editor)
            .p(px(24.))
            .gap(px(16.))
            // Text the window's height didn't allow for scrolls here, above
            // the buttons, instead of running under them.
            .child(
                div()
                    .id("startup-error-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(
                        v_flex()
                            .gap(px(16.))
                            .child(body(&self.screen, colors))
                            .children(update)
                            .child(feedback),
                    ),
            )
            .child(buttons(&self.screen, self.copied, on))
    }
}

/// Says `choice` on standard output and ends the window's process with its
/// code, there and then: nothing in it needs saving.
fn finish(choice: Choice) -> ! {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{}", choice.line());
    let _ = out.flush();
    std::process::exit(choice.code());
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
    // The update and feedback sections sit under the text.
    let height = (height_for(&screen) + 170.).min(720.);
    // The assets hold the warning icon the title wears.
    application()
        .with_assets(crate::assets::KansoAssets)
        .run(move |cx| {
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
                window.on_window_should_close(cx, |_, _| finish(Choice::Quit));
                cx.new(|cx| ErrorView::new(screen, window, cx))
            });
            if opened.is_err() {
                cx.quit();
                return;
            }
            cx.activate(true);
        });
    // Every choice ends the process in `finish`; getting here means no
    // window opened, which is no answer.
    std::process::exit(NO_WINDOW_CODE);
}

/// The exit code when no window could be opened.
const NO_WINDOW_CODE: i32 = 2;

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
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let json = serde_json::to_string(screen).ok();
    let wrote = match (child.stdin.take(), json) {
        (Some(mut stdin), Some(json)) => writeln!(stdin, "{json}").is_ok(),
        _ => false,
    };
    // Its input closed, the window draws; its output is the choice.
    let mut said = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut said);
    }
    let status = child.wait().ok()?;
    if !wrote {
        return None;
    }
    // A process that died before it drew anything (no display) is no
    // answer: the caller falls back to standard error.
    answer(&said, status.code())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use proptest::prelude::*;

    use super::{
        Choice, Failure, NO_WINDOW_CODE, QUIT_CODE, RETRY_CODE, answer, describe, height_for,
    };

    /// The choice is read from the window's output, whatever the code: on
    /// macOS the process can end with 0 through AppKit. With no line, the
    /// code decides, and a window that never opened is no answer.
    #[test]
    fn the_choice_comes_from_the_line_then_the_code() {
        assert_eq!(answer("try-again\n", Some(0)), Some(Choice::TryAgain));
        assert_eq!(answer("quit\n", Some(0)), Some(Choice::Quit));
        assert_eq!(answer("", Some(RETRY_CODE)), Some(Choice::TryAgain));
        assert_eq!(answer("", Some(QUIT_CODE)), Some(Choice::Quit));
        assert_eq!(answer("", Some(0)), None);
        assert_eq!(answer("", Some(NO_WINDOW_CODE)), None);
        assert_eq!(answer("", None), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0063_686f_6963),
            ..ProptestConfig::with_cases(256)
        })]

        /// Invariant: a choice's own line always reads back as that choice,
        /// whatever else came out and whatever the exit code.
        #[test]
        fn prop_a_choice_line_reads_back(
            try_again in any::<bool>(),
            noise in "[a-z ]{0,12}",
            code in proptest::option::of(-3i32..20),
        ) {
            prop_assume!(noise.trim() != Choice::Quit.line());
            let choice = if try_again { Choice::TryAgain } else { Choice::Quit };
            let out = format!("{noise}\n{}\n", choice.line());
            prop_assert_eq!(answer(&out, code), Some(choice));
        }
    }

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
