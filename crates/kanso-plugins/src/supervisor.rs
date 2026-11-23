//! Running a plugin: one `kanso-plugin-host` process per plugin in use,
//! owned here from start to kill.
//!
//! The host starts with an empty environment, the plugin's own folder as
//! its working directory and a pipe as its only input. Every request has a
//! wall deadline: past it the process is killed and the request fails
//! ("did not finish"), whatever the script was doing. Time the app spends
//! answering the plugin's own calls doesn't count against it. Calls are
//! rate-limited, and each is answered by a [`Calls`] (the app's
//! [`crate::router::Router`]) one at a time, in order.
//!
//! [`Health`] counts what goes wrong over a session: two crashes in ten
//! minutes, three failed starts or three refused calls turn the plugin off
//! until the person presses Try again.

use std::collections::VecDeque;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kanso_plugin_host::protocol::{
    Budgets, CODE_LIMIT, CODE_UNAVAILABLE, Failure, FromHost, MAX_FROM_HOST, MAX_TO_HOST,
    PluginInfo, Registered, Theme, ToHost, read_message, write_message,
};
use serde_json::{Map, Value};

use crate::model::Disabled;

/// The longest a command, a press or a render may take, wall-clock.
pub const WALL_DEADLINE: Duration = Duration::from_secs(5);
/// The longest a host may take to start and run its entry script.
pub const STARTUP_DEADLINE: Duration = Duration::from_secs(1);
/// Calls to the app a plugin may make in a minute.
pub const CALLS_PER_MINUTE: usize = 60;
/// Crashes within [`CRASH_WINDOW`] that turn a plugin off.
pub const CRASHES_TO_DISABLE: usize = 2;
pub const CRASH_WINDOW: Duration = Duration::from_secs(10 * 60);
/// Failed starts in a session that turn a plugin off.
pub const FAILED_STARTS_TO_DISABLE: u32 = 3;
/// Refused calls in a session that turn a plugin off.
pub const REFUSALS_TO_DISABLE: u32 = 3;
/// The most a plugin's log keeps, in bytes.
pub const LOG_BYTES: usize = 200 * 1024;

/// Answers a plugin's calls to the app.
pub trait Calls {
    fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure>;
}

impl Calls for crate::router::Router<'_> {
    fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure> {
        crate::router::Router::call(self, call, args)
    }
}

/// Why a request failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RunError {
    #[error("it did not finish")]
    Timeout,
    #[error("it stopped unexpectedly: {0}")]
    Crashed(String),
    #[error("it couldn't start: {0}")]
    FailedToStart(String),
    #[error("{}", .0.message)]
    Failed(Failure),
}

impl RunError {
    /// Whether the process is gone (and counts against the plugin's health).
    pub fn lost_the_process(&self) -> bool {
        matches!(self, RunError::Timeout | RunError::Crashed(_))
    }
}

/// What has gone wrong with one plugin this session.
#[derive(Clone, Debug, Default)]
pub struct Health {
    crashes: VecDeque<Instant>,
    failed_starts: u32,
    refusals: u32,
}

impl Health {
    /// Counts a crash (a kill at the deadline is one); turns the plugin off
    /// at the second in [`CRASH_WINDOW`].
    pub fn crashed(&mut self, now: Instant) -> Option<Disabled> {
        self.crashes.push_back(now);
        while self
            .crashes
            .front()
            .is_some_and(|at| now.duration_since(*at) > CRASH_WINDOW)
        {
            self.crashes.pop_front();
        }
        (self.crashes.len() >= CRASHES_TO_DISABLE).then_some(Disabled::Crashed)
    }

    /// Counts a failed start; turns the plugin off at the third.
    pub fn failed_to_start(&mut self) -> Option<Disabled> {
        self.failed_starts += 1;
        (self.failed_starts >= FAILED_STARTS_TO_DISABLE).then_some(Disabled::FailedToStart)
    }

    /// Counts refused calls; turns the plugin off at the third.
    pub fn refused(&mut self, count: u32) -> Option<Disabled> {
        self.refusals += count;
        (self.refusals >= REFUSALS_TO_DISABLE).then_some(Disabled::Refused)
    }

    /// Try again: a clean slate.
    pub fn reset(&mut self) {
        *self = Health::default();
    }

    pub fn refusals(&self) -> u32 {
        self.refusals
    }
}

/// A plugin's `kanso.log` lines, newest last, at most [`LOG_BYTES`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Log {
    lines: VecDeque<(String, String)>,
    bytes: usize,
}

impl Log {
    pub fn push(&mut self, level: &str, text: &str) {
        let level: String = level.chars().take(8).collect();
        let text: String = text.chars().take(4096).collect();
        self.bytes += level.len() + text.len();
        self.lines.push_back((level, text));
        while self.bytes > LOG_BYTES {
            match self.lines.pop_front() {
                Some((l, t)) => self.bytes -= l.len() + t.len(),
                None => break,
            }
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &(String, String)> {
        self.lines.iter()
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

/// Calls in the last minute, to hold a plugin to [`CALLS_PER_MINUTE`].
#[derive(Clone, Debug, Default)]
pub struct CallWindow {
    at: VecDeque<Instant>,
}

impl CallWindow {
    /// Whether one more call may go through at `now`; counts it if so.
    pub fn admit(&mut self, now: Instant) -> bool {
        while self
            .at
            .front()
            .is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(60))
        {
            self.at.pop_front();
        }
        if self.at.len() >= CALLS_PER_MINUTE {
            return false;
        }
        self.at.push_back(now);
        true
    }
}

/// Where the host binary is: in `../lib/<this program's name>` first (Kanso
/// Nightly's install on Linux keeps its own host there, so it never runs
/// Kanso's from `~/.local/bin`), then beside this program (in the app
/// bundle, or `target/<profile>/` while developing; a test runs from
/// `deps/` inside it). A debug build also takes `KANSO_PLUGIN_HOST`.
pub fn find_host() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("KANSO_PLUGIN_HOST") {
        return Some(PathBuf::from(path));
    }
    let name = format!("kanso-plugin-host{}", std::env::consts::EXE_SUFFIX);
    let exe = std::env::current_exe().ok()?;
    host_candidates(&exe, &name)
        .into_iter()
        .find(|path| path.is_file())
}

/// Where [`find_host`] looks for `name` for a program at `exe`, in order.
fn host_candidates(exe: &Path, name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let Some(dir) = exe.parent() else {
        return candidates;
    };
    if let (Some(parent), Some(program)) = (dir.parent(), exe.file_name()) {
        candidates.push(parent.join("lib").join(program).join(name));
    }
    candidates.push(dir.join(name));
    if let Some(parent) = dir.parent() {
        candidates.push(parent.join(name));
    }
    candidates
}

/// A running host.
pub struct Host {
    child: Child,
    stdin: ChildStdin,
    messages: mpsc::Receiver<Result<FromHost, String>>,
    next_id: u64,
    window: CallWindow,
    pub info: PluginInfo,
}

impl Drop for Host {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Reads the host's messages on a thread of its own, so a request can wait
/// with a deadline. Stops at the end of output or the first bad line.
fn read_messages(stdout: std::process::ChildStdout) -> mpsc::Receiver<Result<FromHost, String>> {
    let (send, receive) = mpsc::channel();
    std::thread::Builder::new()
        .name("kanso-plugin-pipe".into())
        .spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_message::<FromHost>(&mut reader, MAX_FROM_HOST) {
                    Ok(Some(message)) => {
                        if send.send(Ok(message)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => return,
                    Err(err) => {
                        send.send(Err(err.to_string())).ok();
                        return;
                    }
                }
            }
        })
        .expect("starting a plugin's pipe reader");
    receive
}

impl Host {
    /// Starts `host_exe` for plugin `info` in `cwd` and runs its entry
    /// `code`, within [`STARTUP_DEADLINE`].
    pub fn start(
        host_exe: &Path,
        cwd: &Path,
        info: PluginInfo,
        code: String,
        budgets: Budgets,
    ) -> Result<(Host, Registered), RunError> {
        Host::start_within(host_exe, cwd, info, code, budgets, STARTUP_DEADLINE)
    }

    /// [`Host::start`] with another deadline, for tests.
    pub fn start_within(
        host_exe: &Path,
        cwd: &Path,
        info: PluginInfo,
        code: String,
        budgets: Budgets,
        deadline: Duration,
    ) -> Result<(Host, Registered), RunError> {
        let mut command = Command::new(host_exe);
        command
            .env_clear()
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        if let Some(root) = std::env::var_os("SystemRoot") {
            // Windows can't start a program with no SystemRoot at all.
            command.env("SystemRoot", root);
        }
        let mut child = command
            .spawn()
            .map_err(|e| RunError::FailedToStart(e.to_string()))?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            child.kill().ok();
            return Err(RunError::FailedToStart("no pipe".into()));
        };
        let mut host = Host {
            child,
            stdin,
            messages: read_messages(stdout),
            next_id: 1,
            window: CallWindow::default(),
            info: info.clone(),
        };
        host.send(&ToHost::Start {
            plugin: info,
            code,
            budgets,
        })
        .map_err(|e| RunError::FailedToStart(e.to_string()))?;
        match host.messages.recv_timeout(deadline) {
            Ok(Ok(FromHost::Started {
                ok: true,
                registered,
                ..
            })) => Ok((host, registered)),
            Ok(Ok(FromHost::Started { error, .. })) => Err(RunError::FailedToStart(
                error.map_or_else(|| "its script failed".into(), |e| e.message),
            )),
            Ok(Ok(_)) | Ok(Err(_)) => Err(RunError::FailedToStart("it sent something else".into())),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(RunError::FailedToStart("it took too long to start".into()))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(RunError::FailedToStart("it stopped while starting".into()))
            }
        }
    }

    fn send(&mut self, message: &ToHost) -> std::io::Result<()> {
        write_message(&mut self.stdin, message, MAX_TO_HOST).map_err(|e| match e {
            kanso_plugin_host::protocol::FrameError::Io(err) => err,
            other => std::io::Error::other(other.to_string()),
        })
    }

    /// Kills the process now.
    pub fn kill(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }

    /// Asks the host to stop, then makes sure it has.
    pub fn stop(mut self) {
        self.send(&ToHost::Stop).ok();
        let until = Instant::now() + Duration::from_millis(200);
        while Instant::now() < until {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        self.kill();
    }

    /// Whether the process is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Runs command `command` with `args`.
    pub fn invoke(
        &mut self,
        command: &str,
        args: Value,
        calls: &mut dyn Calls,
        log: &mut Log,
    ) -> Result<Value, RunError> {
        let id = self.take_id();
        self.request(
            ToHost::Invoke {
                id,
                command: command.to_owned(),
                args,
            },
            id,
            WALL_DEADLINE,
            calls,
            log,
        )
    }

    /// Draws a fenced block. A renderer can call nothing, so `calls` only
    /// ever refuses.
    pub fn render(
        &mut self,
        language: &str,
        source: &str,
        theme: Theme,
        calls: &mut dyn Calls,
        log: &mut Log,
    ) -> Result<String, RunError> {
        let id = self.take_id();
        let value = self.request(
            ToHost::Render {
                id,
                language: language.to_owned(),
                source: source.to_owned(),
                theme,
            },
            id,
            WALL_DEADLINE,
            calls,
            log,
        )?;
        match value {
            Value::String(svg) => Ok(svg),
            _ => Err(RunError::Failed(Failure::new(
                kanso_plugin_host::protocol::CODE_INVALID,
                "a renderer must return SVG text",
            ))),
        }
    }

    /// A button in the plugin's panel was pressed.
    pub fn press(
        &mut self,
        button: &str,
        values: Map<String, Value>,
        calls: &mut dyn Calls,
        log: &mut Log,
    ) -> Result<Value, RunError> {
        let id = self.take_id();
        self.request(
            ToHost::Press {
                id,
                button: button.to_owned(),
                values,
            },
            id,
            WALL_DEADLINE,
            calls,
            log,
        )
    }

    /// Sends `message` (request `id`) and answers the host's calls until it
    /// finishes, within `wall` of the host's own time. Kills the host past
    /// the deadline or on anything out of protocol.
    pub fn request(
        &mut self,
        message: ToHost,
        id: u64,
        wall: Duration,
        calls: &mut dyn Calls,
        log: &mut Log,
    ) -> Result<Value, RunError> {
        if self.send(&message).is_err() {
            self.kill();
            return Err(RunError::Crashed("its pipe closed".into()));
        }
        let mut deadline = Instant::now() + wall;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let received = match self.messages.recv_timeout(left) {
                Ok(received) => received,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.kill();
                    return Err(RunError::Timeout);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.kill();
                    return Err(RunError::Crashed("it exited".into()));
                }
            };
            match received {
                Ok(FromHost::Result {
                    id: done,
                    ok,
                    value,
                    error,
                }) if done == id => {
                    return if ok {
                        Ok(value)
                    } else {
                        Err(RunError::Failed(error.unwrap_or_else(|| {
                            Failure::new(CODE_UNAVAILABLE, "it failed")
                        })))
                    };
                }
                Ok(FromHost::Host {
                    id: asking,
                    call,
                    args,
                }) if asking == id => {
                    let began = Instant::now();
                    // A name outside the protocol goes to the router,
                    // which refuses and counts it.
                    let answer = if !self.window.admit(began) {
                        Err(Failure::new(
                            CODE_LIMIT,
                            format!("a plugin may call Kanso {CALLS_PER_MINUTE} times a minute"),
                        ))
                    } else {
                        calls.call(&call, args)
                    };
                    // The app's time doesn't count against the plugin.
                    deadline += began.elapsed();
                    let reply = match answer {
                        Ok(value) => ToHost::Reply {
                            id,
                            ok: true,
                            value,
                            error: None,
                        },
                        Err(error) => ToHost::Reply {
                            id,
                            ok: false,
                            value: Value::Null,
                            error: Some(error),
                        },
                    };
                    if self.send(&reply).is_err() {
                        self.kill();
                        return Err(RunError::Crashed("its pipe closed".into()));
                    }
                }
                Ok(FromHost::Log { level, text }) => log.push(&level, &text),
                Ok(_) => {
                    self.kill();
                    return Err(RunError::Crashed("it broke the protocol".into()));
                }
                Err(reason) => {
                    self.kill();
                    return Err(RunError::Crashed(reason));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn two_crashes_in_ten_minutes_disable() {
        let mut health = Health::default();
        let start = Instant::now();
        assert_eq!(health.crashed(start), None);
        // Eleven minutes on, the first no longer counts.
        assert_eq!(health.crashed(start + Duration::from_secs(11 * 60)), None);
        assert_eq!(
            health.crashed(start + Duration::from_secs(12 * 60)),
            Some(Disabled::Crashed)
        );
    }

    #[test]
    fn three_failed_starts_or_refusals_disable() {
        let mut health = Health::default();
        assert_eq!(health.failed_to_start(), None);
        assert_eq!(health.failed_to_start(), None);
        assert_eq!(health.failed_to_start(), Some(Disabled::FailedToStart));
        let mut health = Health::default();
        assert_eq!(health.refused(2), None);
        assert_eq!(health.refused(1), Some(Disabled::Refused));
        health.reset();
        assert_eq!(health.refused(0), None);
    }

    #[test]
    fn the_nightly_host_is_looked_for_in_its_own_lib_folder_first() {
        let name = "kanso-plugin-host";
        let nightly = host_candidates(Path::new("/home/a/.local/bin/kanso-nightly"), name);
        assert_eq!(
            nightly,
            [
                "/home/a/.local/lib/kanso-nightly/kanso-plugin-host",
                "/home/a/.local/bin/kanso-plugin-host",
                "/home/a/.local/kanso-plugin-host",
            ]
            .map(PathBuf::from)
        );
        // Kanso's own never reaches into Kanso Nightly's folder.
        let stable = host_candidates(Path::new("/home/a/.local/bin/kanso"), name);
        assert!(
            stable
                .iter()
                .all(|p| !p.to_string_lossy().contains("nightly"))
        );
        assert_eq!(
            stable[1],
            PathBuf::from("/home/a/.local/bin/kanso-plugin-host")
        );
        assert!(host_candidates(Path::new("/"), name).is_empty());
    }

    proptest! {
        /// Invariant: whatever the program's name, its own lib folder is
        /// tried first and beside it comes before the folder above.
        #[test]
        fn prop_own_lib_folder_comes_first(program in "[a-z][a-z-]{0,12}", root in "[a-z]{1,6}") {
            let exe = PathBuf::from(format!("/{root}/bin/{program}"));
            let found = host_candidates(&exe, "h");
            prop_assert_eq!(found.len(), 3);
            prop_assert_eq!(&found[0], &PathBuf::from(format!("/{root}/lib/{program}/h")));
            prop_assert_eq!(&found[1], &PathBuf::from(format!("/{root}/bin/h")));
        }

        /// Invariant: no 60-second window ever admits more than the cap.
        #[test]
        fn prop_the_call_window_holds(gaps in prop::collection::vec(0u64..3_000, 1..400)) {
            let mut window = CallWindow::default();
            let start = Instant::now();
            let mut at = Duration::ZERO;
            let mut admitted: Vec<Duration> = Vec::new();
            for gap in gaps {
                at += Duration::from_millis(gap);
                if window.admit(start + at) {
                    admitted.push(at);
                }
            }
            for (i, first) in admitted.iter().enumerate() {
                let within = admitted[i..].iter().take_while(|t| **t - *first < Duration::from_secs(60)).count();
                prop_assert!(within <= CALLS_PER_MINUTE);
            }
        }

        /// Invariant: the log never holds more than its cap, and keeps the
        /// newest line.
        #[test]
        fn prop_the_log_stays_bounded(lines in prop::collection::vec("\\PC{0,300}", 1..2000)) {
            let mut log = Log::default();
            for line in &lines {
                log.push("info", line);
            }
            prop_assert!(log.bytes() <= LOG_BYTES);
            let last: String = lines.last().unwrap().chars().take(4096).collect();
            prop_assert_eq!(&log.lines().last().unwrap().1, &last);
        }
    }
}
