//! "Encrypting your library… 62%" (issue #243 step 10): what a person sees
//! while an existing library is turned into an encrypted one.
//!
//! The library window does not open until the work is done. The work runs
//! in the process that holds the data folder, before any window exists, so
//! the sheet is a small window of a second, short-lived process
//! (`kanso encrypt-progress`): the first writes its progress to the second's
//! standard input, and the second closes when that ends, however it ends.
//! The second process never opens the library.

use std::io::{BufRead as _, Write as _};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use gpui_kit::component::{StyledExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_store::{EncryptProgress, EncryptStage};

use crate::theme::Colors;

/// The sheet's size.
pub const WIDTH: f32 = 440.;
pub const HEIGHT: f32 = 150.;

/// The title for a stage: "Encrypting your library… 62%".
pub fn title(stage: EncryptStage, percent: u8) -> String {
    let what = match stage {
        EncryptStage::Backups => "backups",
        _ => "library",
    };
    format!("Encrypting your {what}\u{2026} {percent}%")
}

/// The line under the bar for a stage.
pub fn detail(stage: EncryptStage) -> &'static str {
    match stage {
        EncryptStage::Preparing => "Getting ready. Your notes are only read.",
        EncryptStage::Copying => "Copying your notes into an encrypted file.",
        EncryptStage::Checking => "Checking that every note came across.",
        EncryptStage::Finishing => "Putting the encrypted library in place.",
        EncryptStage::Backups => "Encrypting the backups Kanso made before.",
    }
}

const NAMES: [(&str, EncryptStage); 5] = [
    ("preparing", EncryptStage::Preparing),
    ("copying", EncryptStage::Copying),
    ("checking", EncryptStage::Checking),
    ("finishing", EncryptStage::Finishing),
    ("backups", EncryptStage::Backups),
];

/// A progress as one line of the protocol: `copying 62`.
pub fn encode(progress: EncryptProgress) -> String {
    let name = NAMES
        .iter()
        .find(|(_, stage)| *stage == progress.stage)
        .map_or("preparing", |(name, _)| name);
    format!("{name} {}", progress.percent())
}

/// A line of the protocol read back. `None` for anything else.
pub fn decode(line: &str) -> Option<(EncryptStage, u8)> {
    let (name, percent) = line.trim().split_once(' ')?;
    let stage = NAMES.iter().find(|(n, _)| *n == name)?.1;
    Some((stage, percent.parse::<u8>().ok()?.min(100)))
}

/// The sheet's content: the title with the percent, a bar, and a line of
/// what is happening. The window and the snapshots draw this.
// no-prop-test: drawing; the snapshots show it, `title` and the protocol are tested.
pub fn body(stage: EncryptStage, percent: u8, colors: &Colors) -> impl IntoElement {
    v_flex()
        .debug_selector(|| "encrypt-progress".into())
        .w_full()
        .gap(px(12.))
        .child(
            div()
                .debug_selector(|| "encrypt-progress-title".into())
                .text_size(px(15.))
                .font_semibold()
                .text_color(colors.text)
                .child(title(stage, percent)),
        )
        .child(
            div()
                .h(px(6.))
                .w_full()
                .rounded_full()
                .bg(colors.fill)
                .child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(colors.accent)
                        .w(relative(f32::from(percent) / 100.)),
                ),
        )
        .child(
            div()
                .text_size(px(12.5))
                .line_height(px(18.))
                .text_color(colors.text_secondary)
                .child(detail(stage)),
        )
        .child(
            h_flex().child(
                div()
                    .text_size(px(12.))
                    .line_height(px(17.))
                    .text_color(colors.text_muted)
                    .child("This happens once. Kanso opens when it is done."),
            ),
        )
}

/// The window's view.
struct ProgressView {
    stage: EncryptStage,
    percent: u8,
}

impl Render for ProgressView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .size_full()
            .bg(colors.editor)
            .p(px(24.))
            .child(body(self.stage, self.percent, colors))
    }
}

/// The second process: shows the sheet, follows the lines on standard input,
/// and closes when they end.
// no-prop-test: a window around `body` and the tested protocol.
pub fn run_child() -> anyhow::Result<()> {
    let (tx, rx) = async_channel::unbounded::<Option<(EncryptStage, u8)>>();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if let Some(progress) = decode(&line) {
                let _ = tx.send_blocking(Some(progress));
            }
        }
        let _ = tx.send_blocking(None);
    });
    application().run(move |cx| {
        gpui_kit::init(cx);
        let size = size(px(WIDTH), px(HEIGHT));
        let bounds = Bounds::centered(None, size, cx);
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
            cx.new(|_| ProgressView {
                stage: EncryptStage::Preparing,
                percent: 0,
            })
        });
        let Ok((_, view)) = opened else {
            cx.quit();
            return;
        };
        cx.spawn(async move |cx| {
            while let Ok(next) = rx.recv().await {
                let Some((stage, percent)) = next else { break };
                view.update(cx, |view, cx| {
                    // Each stage has its own bar (the backups' starts at
                    // 0 after the library's 100); within one it only grows.
                    view.percent = if stage == view.stage {
                        percent.max(view.percent)
                    } else {
                        percent
                    };
                    view.stage = stage;
                    cx.notify();
                });
            }
            cx.update(|cx| cx.quit());
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}

/// How long the window's process has to close once its input ends, before
/// it is killed: the library window never waits on it for longer.
const CHILD_GRACE: Duration = Duration::from_secs(3);

/// The process that draws the window, as the reporter uses it.
trait ProgressWindow: Send {
    /// Sends one line of the protocol. A process that is gone is ignored.
    fn send(&mut self, line: &str);
    /// Ends its input, which tells it to close.
    fn end_input(&mut self);
    /// Whether it has exited (or can't be asked any more).
    fn exited(&mut self) -> bool;
    /// Stops it for good, and reaps it.
    fn kill(&mut self);
}

/// The real window: `kanso encrypt-progress`, fed on its standard input.
struct ChildWindow {
    child: Child,
    stdin: Option<ChildStdin>,
}

// no-prop-test: a thin wrapper around a child process; the drop's logic is tested with a stand-in.
impl ProgressWindow for ChildWindow {
    fn send(&mut self, line: &str) {
        if let Some(stdin) = &mut self.stdin {
            let _ = writeln!(stdin, "{line}");
            let _ = stdin.flush();
        }
    }

    fn end_input(&mut self) {
        self.stdin = None;
    }

    fn exited(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(None))
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Reports an encryption's progress to a person: in a window while the app
/// starts, on standard error from a command.
pub struct Reporter {
    window: Option<Mutex<Box<dyn ProgressWindow>>>,
    last: Mutex<Option<String>>,
    grace: Duration,
}

impl Reporter {
    /// A reporter that draws the window, or says nothing when it can't.
    /// The window's process gets only progress lines: no key, no path, no
    /// argument but its command's name.
    pub fn window() -> Reporter {
        let spawned = std::env::current_exe().ok().and_then(|exe| {
            let mut child = Command::new(exe)
                .arg("encrypt-progress")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let stdin = child.stdin.take()?;
            Some(Box::new(ChildWindow {
                child,
                stdin: Some(stdin),
            }) as Box<dyn ProgressWindow>)
        });
        Reporter::showing(spawned, CHILD_GRACE)
    }

    /// A reporter that sends its lines to `window`, which is given `grace`
    /// to close once they stop.
    fn showing(window: Option<Box<dyn ProgressWindow>>, grace: Duration) -> Reporter {
        Reporter {
            window: window.map(Mutex::new),
            last: Mutex::new(None),
            grace,
        }
    }

    /// A reporter for a command: a line on standard error at every tenth
    /// percent.
    pub fn stderr() -> Reporter {
        Reporter {
            window: None,
            last: Mutex::new(None),
            grace: CHILD_GRACE,
        }
    }

    /// Passes a progress on.
    pub fn report(&self, progress: EncryptProgress) {
        let line = encode(progress);
        {
            let mut last = self.last.lock().expect("reporter poisoned");
            if last.as_deref() == Some(&line) {
                return;
            }
            *last = Some(line.clone());
        }
        if let Some(window) = &self.window {
            window.lock().expect("reporter poisoned").send(&line);
        } else if progress.percent().is_multiple_of(10) {
            eprintln!("kanso: {}", title(progress.stage, progress.percent()));
        }
    }
}

impl Drop for Reporter {
    fn drop(&mut self) {
        if let Some(window) = self.window.take() {
            let mut window = window.into_inner().expect("reporter poisoned");
            // Its input ends, and it closes. One that doesn't (hung, or
            // stuck opening a window) is killed: the library never waits on
            // a progress bar.
            window.end_input();
            let until = std::time::Instant::now() + self.grace;
            while std::time::Instant::now() < until {
                if window.exited() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            window.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode, title};
    use kanso_store::{EncryptProgress, EncryptStage};
    use proptest::prelude::*;

    fn stage() -> impl Strategy<Value = EncryptStage> {
        prop_oneof![
            Just(EncryptStage::Preparing),
            Just(EncryptStage::Copying),
            Just(EncryptStage::Checking),
            Just(EncryptStage::Finishing),
            Just(EncryptStage::Backups),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// Invariants: a progress survives the line it travels in, a percent
        /// above 100 never reaches the bar, and the title shows the percent.
        #[test]
        fn prop_a_progress_round_trips_through_its_line(stage in stage(), fraction in -1.0f32..2.0) {
            let progress = EncryptProgress { stage, fraction };
            let (got, percent) = decode(&encode(progress)).unwrap();
            assert_eq!(got, stage);
            assert_eq!(percent, progress.percent());
            assert!(percent <= 100);
            let heading = title(stage, percent);
            let suffix = format!("{percent}%");
            assert!(heading.ends_with(&suffix));
        }

    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// Invariant: anything that is not a progress line is ignored.
        #[test]
        fn prop_noise_is_not_a_progress(line in "[a-z0-9 ]{0,12}") {
            if let Some((_, percent)) = decode(&line) {
                assert!(percent <= 100);
            }
        }
    }

    /// A stand-in window process: it exits `exits_after` checks after its
    /// input ends (never, for `None`), and says what was done to it.
    #[derive(Default)]
    struct Seen {
        lines: Vec<String>,
        input_ended: bool,
        killed: bool,
    }

    struct StandIn {
        seen: std::sync::Arc<std::sync::Mutex<Seen>>,
        exits_after: Option<usize>,
        checks: usize,
    }

    impl super::ProgressWindow for StandIn {
        fn send(&mut self, line: &str) {
            self.seen.lock().unwrap().lines.push(line.to_owned());
        }
        fn end_input(&mut self) {
            self.seen.lock().unwrap().input_ended = true;
        }
        fn exited(&mut self) -> bool {
            self.checks += 1;
            self.exits_after.is_some_and(|after| self.checks > after)
        }
        fn kill(&mut self) {
            self.seen.lock().unwrap().killed = true;
        }
    }

    fn stand_in(
        exits_after: Option<usize>,
        grace: std::time::Duration,
    ) -> (super::Reporter, std::sync::Arc<std::sync::Mutex<Seen>>) {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Seen::default()));
        let window = StandIn {
            seen: seen.clone(),
            exits_after,
            checks: 0,
        };
        (
            super::Reporter::showing(Some(Box::new(window)), grace),
            seen,
        )
    }

    /// A window process that never closes doesn't hold the library back:
    /// it is killed after its grace. (The drop used to wait on it forever.)
    #[test]
    fn a_hung_window_process_never_holds_the_library_back() {
        let (reporter, seen) = stand_in(None, std::time::Duration::from_millis(100));
        reporter.report(EncryptProgress {
            stage: EncryptStage::Copying,
            fraction: 0.5,
        });
        let started = std::time::Instant::now();
        drop(reporter);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
        let seen = seen.lock().unwrap();
        assert_eq!(seen.lines, ["copying 50"]);
        assert!(seen.input_ended && seen.killed);
    }

    /// A window process that closes when its input ends is left to close,
    /// never killed.
    #[test]
    fn a_window_process_that_closes_is_not_killed() {
        let (reporter, seen) = stand_in(Some(2), std::time::Duration::from_secs(5));
        drop(reporter);
        let seen = seen.lock().unwrap();
        assert!(seen.input_ended && !seen.killed);
    }

    #[test]
    fn the_title_reads_as_the_design_says() {
        assert_eq!(
            title(EncryptStage::Copying, 62),
            "Encrypting your library\u{2026} 62%"
        );
    }
}
