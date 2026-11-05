//! Recording a note from the microphone (issue #38, step 4): the session
//! that runs while the recording bar shows, and what happens when it ends.
//!
//! One recording runs at a time, for the whole app: it goes on while
//! windows come and go, and the bar (`recording_bar`) follows. **Stop**
//! finalises the file on a thread, attaches its parts and makes or updates
//! the note (`kanso_engine::recording_flow`), opens the note and starts
//! transcribing it. A crash leaves the in-progress folder behind; the next
//! launch offers **Recover** (finish and transcribe what is there) or
//! **Discard**. A full disk, the cap or an unplugged microphone end the
//! recording the same way as Stop, with a notice, keeping what was written.
//!
//! Quitting while recording asks, then stops first. A transcription cut off
//! by a quit leaves a marker the next launch offers to finish.

use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_audio::recovery;
use kanso_audio::session::{self, in_progress_dir, now_ms, remove_session};
use kanso_audio::{
    AudioError, Capture, CaptureStatus, Channels, Finished, RecordSource, RecorderOptions,
    Recoverable, ScreenAccess, StopReason, SystemAudioTarget,
};
use kanso_engine::recording_flow::{
    self, Destination, Recorded, SavedRecording, SystemSource, WhatToRecord, save_recording_or_new,
    save_system_track, system_session_id, system_track_of,
};
use kanso_store::Origin;
use kanso_voice::audio::meter_level;
use kanso_voice::permission::{self, Permission};

use crate::recording_model::{BarState, CrashBanner, Notice, PendingTranscript};
use crate::transcribe::{self, Source};
use crate::workspace::{self, AppState};

actions!(kanso, [RecordAudio]);

/// How often the bar is redrawn and the recorder asked how it is.
const TICK: Duration = Duration::from_millis(200);
/// The file that remembers a transcription a quit cut off.
const PENDING_FILE: &str = "pending-transcript.json";

pub use kanso_engine::recording_flow::RecordPlan;

/// The recording in progress: the microphone (which carries system audio on
/// its right channel when both are recorded) or system audio alone.
struct Active {
    /// The microphone, or the microphone and system audio together as one
    /// stereo recording; `None` for system audio alone.
    mic: Option<Capture>,
    /// System audio recorded alone.
    system: Option<Capture>,
    /// The session ID.
    id: String,
    plan: RecordPlan,
    /// The recording's status; its level is the louder channel's.
    status: CaptureStatus,
    started_ms: u64,
    beats: u64,
    warned: bool,
    /// The system track ended on its own (the app quit, the capture was
    /// stopped); the microphone goes on and the person has been told.
    system_lost: bool,
    _ticker: Task<()>,
    _awake: Awake,
}

/// The app's recording, if any, and what the last run left behind.
#[derive(Default)]
pub struct Recording {
    active: Option<Active>,
    /// Stopping and saving: the next recording waits.
    saving: bool,
    found: Vec<Recoverable>,
    pending: Option<PendingTranscript>,
    /// What the snapshots draw instead of a real recording.
    #[cfg(test)]
    shown: Option<Notice>,
}

impl Global for Recording {}

/// Makes the strip show `notice`, as the snapshots draw it, with no
/// microphone open.
#[cfg(test)]
pub fn show_for_snapshot(notice: Notice, cx: &mut App) {
    cx.default_global::<Recording>().shown = Some(notice);
}

// MARK: Reading the state

/// Whether a recording is running (or being saved).
pub fn busy(cx: &App) -> bool {
    cx.try_global::<Recording>()
        .is_some_and(|r| r.active.is_some() || r.saving)
}

/// Whether a recording is running now.
pub fn is_recording(cx: &App) -> bool {
    cx.try_global::<Recording>()
        .is_some_and(|r| r.active.is_some())
}

/// What the strip under the title bar says, if anything.
pub fn notice(cx: &App) -> Option<Notice> {
    let recording = cx.try_global::<Recording>()?;
    #[cfg(test)]
    if let Some(shown) = &recording.shown {
        return Some(shown.clone());
    }
    if let Some(active) = &recording.active {
        return Some(Notice::Recording(BarState {
            seconds: (active.status.seconds()) as u64,
            level: meter_level(active.status.rms),
            paused: active.status.paused,
            label: active.plan.bar_label(),
            dot_lit: !active.status.paused && active.beats % 4 < 3,
        }));
    }
    if recording.saving {
        return None;
    }
    if let Some(found) = recording
        .found
        .iter()
        .find(|r| !is_companion(&recording.found, &r.id))
    {
        let started = found.session.as_ref().map(|s| s.started_at_ms);
        let at = started.and_then(|ms| recording_flow::time_of_day(ms as i64));
        return Some(Notice::Crash(CrashBanner {
            id: found.id.clone(),
            text: recording_flow::recovery_text(at.as_deref(), found.duration_secs() as u64),
        }));
    }
    recording.pending.clone().map(Notice::Pending)
}

/// Whether `id` is a system audio track whose microphone recording was left
/// too: the two are one recording, offered once, under the microphone's.
fn is_companion(found: &[Recoverable], id: &str) -> bool {
    system_track_of(id).is_some_and(|base| found.iter().any(|r| r.id == base))
}

// MARK: Keeping the Mac awake

/// Stops the Mac from sleeping while it lives. On macOS that is
/// `caffeinate -i` watching this process; elsewhere nothing is held.
struct Awake(#[cfg(target_os = "macos")] Option<std::process::Child>);

impl Awake {
    fn hold() -> Self {
        #[cfg(target_os = "macos")]
        {
            let child = std::process::Command::new("/usr/bin/caffeinate")
                .args(["-i", "-w", &std::process::id().to_string()])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();
            Awake(child)
        }
        #[cfg(not(target_os = "macos"))]
        Awake()
    }
}

#[cfg(target_os = "macos")]
impl Drop for Awake {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

// MARK: Starting

/// Shows `notification` on the main window, or else the first one. It
/// waits for the update in progress to end: a click or an action arrives
/// inside its window's update, and that window can't be reached from in
/// there (nor found by `main_workspace`), so the notice would be lost.
fn notify(notification: Notification, cx: &mut App) {
    cx.defer(move |cx| notify_now(notification, cx));
}

fn notify_now(notification: Notification, cx: &mut App) {
    if let Some((handle, _)) = workspace::main_workspace(cx) {
        handle
            .update(cx, |_, window, cx| {
                window.push_notification(notification, cx)
            })
            .ok();
        return;
    }
    if let Some(handle) = cx.windows().first().copied() {
        handle
            .update(cx, |_, window, cx| {
                window.push_notification(notification, cx)
            })
            .ok();
    }
}

/// What to tell the person about a recorder error.
pub fn error_words(err: &kanso_audio::AudioError) -> String {
    match err {
        AudioError::MicrophoneDenied => {
            "Kanso isn't allowed to use the microphone. Allow it in your system's privacy settings, then try again.".into()
        }
        AudioError::NoMicrophone => "There's no microphone to record from.".into(),
        AudioError::ScreenRecordingDenied => kanso_audio::denied_words(),
        AudioError::SystemAudio(why) => why.clone(),
        other => format!("Couldn't start recording: {other}"),
    }
}

/// Starts recording as `plan` says, after asking the system for the
/// microphone the first time (when it is recorded).
///
/// Screen Recording is never asked for here: the Record sheet asks when
/// system audio is turned on, and a recording only looks. When it isn't
/// allowed, system audio alone doesn't start, and the microphone records on
/// its own with a notice that says how to turn it on.
// no-prop-test: glue over the microphone; the plan and the note flow have
// the properties in `kanso_engine::recording_flow`.
pub fn start(plan: RecordPlan, cx: &mut App) {
    if busy(cx) {
        notify(
            Notification::info("A recording is already running. Stop it before starting another."),
            cx,
        );
        return;
    }
    let mut plan = plan;
    // Beside macOS, which records one app, system audio is the sound
    // output (the default, or the one the sheet picked).
    if !cfg!(target_os = "macos") && plan.what.uses_system() && plan.system.is_none() {
        plan.system = Some(SystemSource::Everything);
    }
    if let Some(problem) = plan.problem() {
        notify(Notification::info(problem), cx);
        return;
    }
    if plan.what.uses_system() {
        if let Some(why) = system_unavailable() {
            notify(Notification::error(why), cx);
            if !plan.what.uses_microphone() {
                return;
            }
            // The microphone records anyway.
            plan.what = WhatToRecord::Microphone;
            plan.system = None;
        } else if cfg!(target_os = "macos") && kanso_audio::screen_access() != ScreenAccess::Allowed
        {
            notify(
                Notification::error(error_words(&AudioError::ScreenRecordingDenied)),
                cx,
            );
            if !plan.what.uses_microphone() {
                return;
            }
            // The microphone records anyway.
            plan.what = WhatToRecord::Microphone;
            plan.system = None;
        }
    }
    if !plan.what.uses_microphone() {
        begin(plan, cx);
        return;
    }
    match permission::status() {
        Permission::Granted => begin(plan, cx),
        Permission::Denied => notify(
            Notification::error(error_words(&AudioError::MicrophoneDenied)),
            cx,
        ),
        Permission::Undetermined => {
            let (tx, rx) = async_channel::bounded(1);
            permission::request(move |granted| {
                tx.try_send(granted).ok();
            });
            cx.spawn(async move |cx| {
                let granted = rx.recv().await.unwrap_or(false);
                cx.update(|cx| {
                    if granted {
                        begin(plan, cx);
                    } else {
                        notify(
                            Notification::error(error_words(&AudioError::MicrophoneDenied)),
                            cx,
                        );
                    }
                });
            })
            .detach();
        }
    }
}

/// What the system capture listens to for `source`: the app on a Mac, else
/// the output `output` the sheet picked (the default when none).
fn target_of(source: &SystemSource, output: Option<&str>) -> SystemAudioTarget {
    match (source, output) {
        (SystemSource::App { bundle_id, .. }, _) => SystemAudioTarget::App(bundle_id.clone()),
        (SystemSource::Everything, Some(id)) => SystemAudioTarget::Device(id.to_owned()),
        (SystemSource::Everything, None) => SystemAudioTarget::Default,
    }
}

/// Why this computer can't record system audio now, in the person's words;
/// `None` where it can. A Mac's answer is its Screen Recording permission,
/// which `start` checks on its own.
fn system_unavailable() -> Option<String> {
    if cfg!(target_os = "macos") {
        return None;
    }
    match kanso_audio::platform_backend() {
        Some(backend) => backend.availability().reason(),
        None => Some(kanso_audio::Unavailable::Unsupported.message()),
    }
}

fn begin(plan: RecordPlan, cx: &mut App) {
    if busy(cx) {
        return;
    }
    // A note that is gone or locked can't take the audio: say so now, not
    // after an hour of recording.
    let store = cx.global::<AppState>().store.clone();
    if let Err(err) = recording_flow::check_destination(&store, &plan.destination) {
        notify(Notification::error(err.to_string()), cx);
        return;
    }
    let mut plan = plan;
    let root = in_progress_dir(&cx.global::<AppState>().data_dir);
    let id = session::new_id();
    let note_id = plan.destination.note_id().map(str::to_owned);
    let target = plan
        .system
        .as_ref()
        .map(|source| target_of(source, plan.system_output.as_deref()));
    let mut mic = None;
    let mut system = None;
    // The microphone and system audio together are one stereo recording,
    // you on the left and the meeting on the right.
    if plan.what == WhatToRecord::MicrophoneAndSystem
        && let Some(target) = target.clone()
    {
        let mut options = RecorderOptions::new(
            Channels::Stereo,
            vec![RecordSource::Microphone, RecordSource::SystemAudio],
        );
        options.id = Some(id.clone());
        options.note_id = note_id.clone();
        match Capture::start_both(&root, options, target) {
            Ok(capture) => mic = Some(capture),
            Err(err @ (AudioError::MicrophoneDenied | AudioError::NoMicrophone)) => {
                notify(Notification::error(error_words(&err)), cx);
                return;
            }
            Err(err) => {
                // The microphone records on, on its own.
                notify(Notification::error(error_words(&err)), cx);
                plan.what = WhatToRecord::Microphone;
                plan.system = None;
            }
        }
    }
    if mic.is_none() && plan.what.uses_microphone() {
        let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
        options.id = Some(id.clone());
        options.note_id = note_id.clone();
        match Capture::start(&root, options) {
            Ok(capture) => mic = Some(capture),
            Err(err) => {
                notify(Notification::error(error_words(&err)), cx);
                return;
            }
        }
    }
    if plan.what == WhatToRecord::SystemOnly
        && let Some(target) = target
    {
        let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::SystemAudio]);
        options.id = Some(id.clone());
        options.note_id = note_id;
        match Capture::start_system(&root, options, target) {
            Ok(capture) => system = Some(capture),
            Err(err) => {
                notify(Notification::error(error_words(&err)), cx);
                return;
            }
        }
    }
    let Some(status) = mic
        .as_ref()
        .or(system.as_ref())
        .map(|capture| capture.status())
    else {
        return;
    };
    let ticker = cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            let stop = cx.update(tick);
            if stop {
                break;
            }
        }
    });
    cx.default_global::<Recording>().active = Some(Active {
        mic,
        system,
        id,
        plan,
        status,
        started_ms: now_ms(),
        beats: 0,
        warned: false,
        system_lost: false,
        _ticker: ticker,
        _awake: Awake::hold(),
    });
    cx.refresh_windows();
}

/// Reads the recorder; true when the recording is over (the ticker stops).
fn tick(cx: &mut App) -> bool {
    let Some(active) = cx.default_global::<Recording>().active.as_mut() else {
        return true;
    };
    // One capture is the recording: the microphone (with system audio on
    // its right channel, when both are recorded), or system audio alone.
    let Some(status) = active
        .mic
        .as_ref()
        .or(active.system.as_ref())
        .map(Capture::status)
    else {
        return true;
    };
    let mut lost = None;
    // With both sources in one capture, the system side can stop while the
    // microphone goes on.
    if let Some(why) = status.system_failure.clone()
        && !active.system_lost
    {
        active.system_lost = true;
        lost = Some(StopReason::Device(why));
    }
    active.beats += 1;
    let warn_now = status.warned && !active.warned;
    active.warned = status.warned;
    let ended = status.ended.clone();
    active.status = status;
    if let Some(reason) = lost {
        let why = match reason {
            StopReason::Device(why) | StopReason::Disk(why) => why,
            StopReason::Cap | StopReason::Requested => "it reached its limit".to_owned(),
        };
        notify(
            Notification::error(format!(
                "System audio stopped ({why}). The microphone is still recording."
            )),
            cx,
        );
    }
    if warn_now {
        notify(
            Notification::info(
                "This recording has been running for 5 hours. It stops at 6 and keeps what it has.",
            ),
            cx,
        );
    }
    cx.refresh_windows();
    if ended.is_some() {
        finish(Finish::default(), cx);
        return true;
    }
    false
}

// MARK: While recording

pub fn pause(cx: &mut App) {
    if let Some(active) = cx.default_global::<Recording>().active.as_mut() {
        for capture in active.mic.iter().chain(&active.system) {
            capture.pause();
        }
        active.status.paused = true;
    }
    cx.refresh_windows();
}

pub fn resume(cx: &mut App) {
    if let Some(active) = cx.default_global::<Recording>().active.as_mut() {
        for capture in active.mic.iter().chain(&active.system) {
            capture.resume();
        }
        active.status.paused = false;
    }
    cx.refresh_windows();
}

/// Stops and keeps the recording.
pub fn stop(cx: &mut App) {
    finish(Finish::default(), cx);
}

/// Asks once, then deletes the recording.
pub fn discard(window: &mut Window, cx: &mut App) {
    if !is_recording(cx) {
        return;
    }
    crate::dialogs::confirm(
        "Discard this recording?",
        "The audio recorded so far is deleted and can't be brought back.",
        "Discard",
        window,
        cx,
        |_, cx| discard_now(cx),
    );
}

fn discard_now(cx: &mut App) {
    let Some(active) = cx.default_global::<Recording>().active.take() else {
        return;
    };
    cx.default_global::<Recording>().saving = true;
    let (tx, rx) = async_channel::bounded(1);
    let captures: Vec<Capture> = active.mic.into_iter().chain(active.system).collect();
    std::thread::spawn(move || {
        let mut result = Ok(());
        for capture in captures {
            if let Err(err) = capture.discard() {
                result = Err(err);
            }
        }
        let _ = tx.try_send(result);
    });
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = rx.recv().await;
        cx.update(|cx| {
            cx.default_global::<Recording>().saving = false;
            if let Ok(Err(err)) = result {
                notify(
                    Notification::error(format!("Couldn't delete the recording: {err}")),
                    cx,
                );
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Stopping

/// What runs once a stopped recording is saved.
pub type AfterSave = Box<dyn FnOnce(&mut App)>;

/// What happens after a recording is stopped.
#[derive(Default)]
pub struct Finish {
    /// Run once the recording is saved (quitting waits for this).
    pub then: Option<AfterSave>,
    /// Leave the note closed and the transcription for the next launch:
    /// the app is quitting.
    pub quitting: bool,
}

/// What the thread that stopped the recording brings back.
pub(crate) struct Done {
    pub(crate) saved: SavedRecording,
    pub(crate) duration_secs: u64,
    pub(crate) reason: StopReason,
    /// The system audio track's files, when one was saved beside the
    /// microphone's.
    pub(crate) system_file_ids: Vec<String>,
    /// Something that didn't go to plan and the person should know.
    pub(crate) warning: Option<String>,
}

impl Done {
    /// The files to transcribe: the recording, whose two channels give the
    /// You and Meeting labels. Only a system track a crash left beside a
    /// microphone recording of its own (made before the two were mixed)
    /// is transcribed instead of it.
    fn transcribe_ids(&self) -> Vec<String> {
        if self.system_file_ids.is_empty() {
            self.saved.file_ids.clone()
        } else {
            self.system_file_ids.clone()
        }
    }
}

/// Stops the recording on a thread, attaches its parts, makes or updates
/// the note, then (unless quitting) opens it and starts transcribing.
// no-prop-test: glue; `save_recording` and the recorder have the properties.
pub fn finish(finish: Finish, cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    let recording = cx.default_global::<Recording>();
    let Some(active) = recording.active.take() else {
        if let Some(then) = finish.then {
            then(cx);
        }
        return;
    };
    recording.saving = true;
    let Active {
        mic,
        system,
        id,
        plan,
        started_ms,
        ..
    } = active;
    let (tx, rx) = async_channel::bounded(1);
    let thread_id = id.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-recording-stop".into())
        .spawn(move || {
            let result = stop_and_save(&store, mic, system, &thread_id, &plan, started_ms);
            let _ = tx.try_send(result);
        });
    if spawned.is_err() {
        cx.default_global::<Recording>().saving = false;
        notify(
            Notification::error("Couldn't stop the recording. It's kept and can be recovered."),
            cx,
        );
        scan(cx);
        return;
    }
    cx.refresh_windows();
    let Finish { then, quitting } = finish;
    cx.spawn(async move |cx| {
        let result = rx
            .recv()
            .await
            .unwrap_or_else(|_| Err("the recorder stopped".to_owned()));
        cx.update(|cx| {
            cx.default_global::<Recording>().saving = false;
            match result {
                Ok(done) => saved(done, quitting, cx),
                Err(why) => {
                    notify(
                        Notification::error(format!(
                            "Couldn't save the recording: {why} Kanso keeps the audio and offers to recover it."
                        )),
                        cx,
                    );
                    scan(cx);
                }
            }
            cx.refresh_windows();
            if let Some(then) = then {
                then(cx);
            }
        });
    })
    .detach();
}

/// Stops `capture`, finishing a torn folder when the stop itself fails, as a
/// full disk can make it.
fn stop_one(capture: Capture) -> Result<(Finished, StopReason), String> {
    let dir = capture.dir().to_path_buf();
    match capture.stop() {
        Ok(stopped) => Ok((stopped.finished, stopped.reason)),
        Err(err) => {
            let found = recovery::find(&dir).ok_or_else(|| err.to_string())?;
            let finished = recovery::recover(&found).map_err(|e| e.to_string())?;
            let reason = match err {
                kanso_audio::AudioError::Device(why) => StopReason::Device(why),
                other => StopReason::Disk(other.to_string()),
            };
            Ok((finished, reason))
        }
    }
}

/// Stops the captures, saves the first (the microphone, or system audio
/// alone) and then the system track beside it, and clears the session
/// folders.
fn stop_and_save(
    store: &kanso_store::Store,
    mic: Option<Capture>,
    system: Option<Capture>,
    id: &str,
    plan: &RecordPlan,
    started_ms: u64,
) -> Result<Done, String> {
    // Both stop before either is saved, so they end together.
    let first = mic.map(stop_one);
    let second = system.map(stop_one);
    let (first, track) = match (first, second) {
        (Some(first), second) => (first, second),
        (None, Some(only)) => (only, None),
        (None, None) => return Err("There was no audio to keep.".to_owned()),
    };
    let (finished, reason) = first?;
    let done = save_finished(
        store,
        &finished,
        id,
        plan.destination.clone(),
        plan.what,
        started_ms,
    )?;
    let mut done = Done { reason, ..done };
    if let Some(track) = track {
        add_system_track(store, &mut done, track, id);
    }
    Ok(done)
}

/// Saves a stopped system track beside the recording in `done`. A track
/// that can't be saved is reported and kept for Recover; the microphone's
/// recording stands.
fn add_system_track(
    store: &kanso_store::Store,
    done: &mut Done,
    track: Result<(Finished, StopReason), String>,
    id: &str,
) {
    let saved = track.and_then(|(finished, _)| {
        let parts: Vec<PathBuf> = finished
            .parts
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        let secs = finished.parts.iter().map(|(_, p)| p.samples).sum::<u64>()
            / u64::from(kanso_audio::SAMPLE_RATE);
        let ids = save_system_track(
            store,
            &done.saved.note_id,
            &system_session_id(id),
            &parts,
            secs,
            Origin::Ui,
        )
        .map_err(|e| e.to_string())?;
        let _ = remove_session(&finished.dir);
        Ok(ids.file_ids)
    });
    match saved {
        Ok(ids) => done.system_file_ids = ids,
        Err(why) => {
            done.warning = Some(format!(
                "The system audio couldn't be saved: {why} Kanso keeps it and offers to recover it."
            ));
        }
    }
}

fn save_finished(
    store: &kanso_store::Store,
    finished: &Finished,
    id: &str,
    destination: Destination,
    what: WhatToRecord,
    started_ms: u64,
) -> Result<Done, String> {
    let duration_secs = finished.session.samples / u64::from(kanso_audio::SAMPLE_RATE);
    let duration_secs = duration_secs.max(
        finished.parts.iter().map(|(_, p)| p.samples).sum::<u64>()
            / u64::from(kanso_audio::SAMPLE_RATE),
    );
    let recorded = Recorded {
        session_id: id.to_owned(),
        parts: finished
            .parts
            .iter()
            .map(|(path, _)| path.clone())
            .collect(),
        duration_secs,
        mic_only: what.mic_only(),
        when: recording_flow::when_text(started_ms as i64),
    };
    // A note deleted or locked while recording sends the audio to a new
    // note rather than leaving it out of the library.
    let saved = match save_recording_or_new(store, &recorded, &destination, Origin::Ui) {
        Ok(saved) => saved,
        Err(recording_flow::FlowError::Empty) => {
            let _ = remove_session(&finished.dir);
            return Err("There was no audio to keep.".to_owned());
        }
        Err(err) => return Err(err.to_string()),
    };
    // The bytes now live in the library like any attachment.
    let _ = remove_session(&finished.dir);
    Ok(Done {
        saved,
        duration_secs,
        reason: StopReason::Requested,
        system_file_ids: Vec::new(),
        warning: None,
    })
}

/// Tells the person how the recording ended, then opens the note and
/// transcribes it.
fn saved(done: Done, quitting: bool, cx: &mut App) {
    match &done.reason {
        StopReason::Requested => {}
        StopReason::Cap => notify(
            Notification::info("The recording reached its 6-hour limit and was saved."),
            cx,
        ),
        StopReason::Device(why) => notify(
            Notification::error(format!(
                "The recording device stopped ({why}). What was recorded is saved."
            )),
            cx,
        ),
        StopReason::Disk(why) => notify(
            Notification::error(format!(
                "The disk is full ({why}). Recording stopped, and what was written is saved."
            )),
            cx,
        ),
    }
    if let Some(warning) = &done.warning {
        notify(Notification::error(warning.clone()), cx);
    }
    let pending = PendingTranscript {
        note_id: done.saved.note_id.clone(),
        file_ids: done.transcribe_ids(),
        duration_secs: done.duration_secs,
    };
    if quitting {
        write_pending(cx, &pending);
        return;
    }
    open_and_transcribe(&pending, cx);
}

/// Opens the note the recording is in and transcribes it there.
fn open_and_transcribe(pending: &PendingTranscript, cx: &mut App) {
    let Some((handle, workspace)) = workspace::main_workspace(cx) else {
        write_pending(cx, pending);
        notify(
            Notification::info("The recording is saved. Open Kanso to transcribe it."),
            cx,
        );
        return;
    };
    let pending = pending.clone();
    handle
        .update(cx, |_, window, cx| {
            let (library, editor) = {
                let workspace = workspace.read(cx);
                (workspace.library.clone(), workspace.editor.clone())
            };
            crate::journal::open_made(
                &library,
                &editor,
                pending.note_id.clone(),
                None,
                false,
                window,
                cx,
            );
            start_transcription(&pending, editor, window, cx);
        })
        .ok();
}

/// Starts transcribing the recording in `pending`, remembering it until it
/// is done.
fn start_transcription(
    pending: &PendingTranscript,
    editor: Entity<crate::editor::NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    write_pending(cx, pending);
    let source = Source::Recording {
        file_ids: pending.file_ids.clone(),
        duration: Duration::from_secs(pending.duration_secs),
    };
    transcribe::start_for(pending.note_id.clone(), editor, source, window, cx);
}

// MARK: The pending-transcript marker

fn pending_path(cx: &App) -> PathBuf {
    cx.global::<AppState>()
        .data_dir
        .join("recordings")
        .join(PENDING_FILE)
}

fn write_pending(cx: &mut App, pending: &PendingTranscript) {
    let path = pending_path(cx);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_vec(pending) {
        let _ = std::fs::write(&path, json);
    }
    cx.default_global::<Recording>().pending = Some(pending.clone());
}

/// Forgets the marker: the transcription finished or was cancelled.
pub fn clear_pending(cx: &mut App) {
    let _ = std::fs::remove_file(pending_path(cx));
    cx.default_global::<Recording>().pending = None;
    cx.refresh_windows();
}

/// Offers "Finish transcribing" for a transcription a quit cut off.
pub fn finish_transcribing(window: &mut Window, cx: &mut App) {
    let Some(pending) = cx.default_global::<Recording>().pending.clone() else {
        return;
    };
    if transcribe::busy(cx) {
        window.push_notification(
            Notification::info("One recording is transcribed at a time. Wait for it to finish."),
            cx,
        );
        return;
    }
    // The click arrives inside its window's update; the main window (often
    // that same one) can only be reached once the update ends.
    cx.defer(move |cx| open_and_transcribe(&pending, cx));
}

// MARK: After a crash

/// Looks for recordings a crash left, and for a transcription a quit cut
/// off. Call once at launch, and after a save failed.
pub fn scan(cx: &mut App) {
    let Some(state) = cx.try_global::<AppState>() else {
        return;
    };
    let root = in_progress_dir(&state.data_dir);
    let pending: Option<PendingTranscript> = std::fs::read(pending_path(cx))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let active = cx
        .try_global::<Recording>()
        .and_then(|r| r.active.as_ref().map(|a| a.id.clone()));
    let mut found = recovery::scan(&root, active.as_deref());
    // The system track of the recording running now is not left behind.
    if let Some(active) = &active {
        let track = system_session_id(active);
        found.retain(|r| r.id != track);
    }
    let recording = cx.default_global::<Recording>();
    recording.found = found;
    if recording.pending.is_none() {
        recording.pending = pending;
    }
    cx.refresh_windows();
}

/// Recover: finishes the partial file, attaches it and transcribes what
/// is there.
// no-prop-test: glue over `recovery::recover` and `save_recording`.
pub fn recover(id: &str, cx: &mut App) {
    if busy(cx) {
        return;
    }
    let Some(at) = cx
        .default_global::<Recording>()
        .found
        .iter()
        .position(|r| r.id == id)
    else {
        return;
    };
    let found = cx.default_global::<Recording>().found.remove(at);
    // Its system audio track, if a crash left one, is part of the same
    // recording.
    let companion = take_companion(id, cx);
    cx.default_global::<Recording>().saving = true;
    cx.refresh_windows();
    let store = cx.global::<AppState>().store.clone();
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = tx.try_send(recover_with(&store, &found, companion.as_ref()));
    });
    cx.spawn(async move |cx| {
        let result = rx
            .recv()
            .await
            .unwrap_or_else(|_| Err("the recovery stopped".to_owned()));
        cx.update(|cx| {
            cx.default_global::<Recording>().saving = false;
            match result {
                Ok(done) => saved(done, false, cx),
                Err(why) => {
                    notify(
                        Notification::error(format!("Couldn't recover the recording: {why}")),
                        cx,
                    );
                    scan(cx);
                }
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Takes the system audio track recorded beside session `id` out of the
/// list of recordings left behind.
fn take_companion(id: &str, cx: &mut App) -> Option<Recoverable> {
    let track = system_session_id(id);
    let found = &mut cx.default_global::<Recording>().found;
    let at = found.iter().position(|r| r.id == track)?;
    Some(found.remove(at))
}

/// Finishes a left-behind recording and saves it: into the note it was for
/// if that is still there, else a new one.
#[cfg(test)]
pub(crate) fn recover_now(store: &kanso_store::Store, found: &Recoverable) -> Result<Done, String> {
    recover_with(store, found, None)
}

/// [`recover_now`], with the system audio track a crash left beside the
/// microphone's. A track left on its own goes onto the note its
/// microphone recording made, when that is there, else into a note of its
/// own.
pub(crate) fn recover_with(
    store: &kanso_store::Store,
    found: &Recoverable,
    companion: Option<&Recoverable>,
) -> Result<Done, String> {
    let finished = recovery::recover(found).map_err(|e| e.to_string())?;
    let session = finished.session.clone();
    let note_exists = |id: &str| store.note(id).ok().flatten().is_some();
    let system_only = session.sources.contains(&RecordSource::SystemAudio)
        && !session.sources.contains(&RecordSource::Microphone);
    if system_only && let Some(base) = system_track_of(&found.id) {
        let target = session
            .note_id
            .clone()
            .filter(|note| note_exists(note))
            .or_else(|| Some(recording_flow::note_id(base)).filter(|note| note_exists(note)));
        if let Some(note) = target {
            let parts: Vec<PathBuf> = finished
                .parts
                .iter()
                .map(|(path, _)| path.clone())
                .collect();
            let samples: u64 = finished.parts.iter().map(|(_, p)| p.samples).sum();
            let duration_secs = samples / u64::from(kanso_audio::SAMPLE_RATE);
            let track =
                save_system_track(store, &note, &found.id, &parts, duration_secs, Origin::Ui)
                    .map_err(|e| e.to_string())?;
            let _ = remove_session(&finished.dir);
            return Ok(Done {
                saved: SavedRecording {
                    note_id: note,
                    file_ids: track.file_ids.clone(),
                    created: false,
                },
                duration_secs,
                reason: StopReason::Requested,
                system_file_ids: track.file_ids,
                warning: None,
            });
        }
    }
    let destination = match session.note_id.as_deref() {
        Some(note) if note_exists(note) => Destination::ThisNote {
            note_id: note.to_owned(),
        },
        _ => Destination::NewNote { book_id: None },
    };
    let what = if system_only {
        WhatToRecord::SystemOnly
    } else if companion.is_some() || session.sources.contains(&RecordSource::SystemAudio) {
        WhatToRecord::MicrophoneAndSystem
    } else {
        WhatToRecord::Microphone
    };
    let mut done = save_finished(
        store,
        &finished,
        &found.id,
        destination,
        what,
        session.started_at_ms,
    )?;
    if let Some(companion) = companion {
        let track = recovery::recover(companion)
            .map(|finished| (finished, StopReason::Requested))
            .map_err(|e| e.to_string());
        add_system_track(store, &mut done, track, &found.id);
    }
    Ok(done)
}

/// Discard: asks, then deletes the left-behind recording.
pub fn discard_found(id: &str, window: &mut Window, cx: &mut App) {
    let id = id.to_owned();
    crate::dialogs::confirm(
        "Discard the unfinished recording?",
        "The audio Kanso saved before it closed is deleted and can't be brought back.",
        "Discard",
        window,
        cx,
        move |_, cx| {
            let state = cx.default_global::<Recording>();
            let track = system_session_id(&id);
            // A recording and the system track beside it go together.
            while let Some(at) = state.found.iter().position(|r| r.id == id || r.id == track) {
                let found = state.found.remove(at);
                let _ = remove_session(&found.dir);
            }
            cx.refresh_windows();
        },
    );
}

// MARK: Quitting

/// Whether quitting must ask first.
pub fn confirm_quit(cx: &App) -> bool {
    is_recording(cx)
}

/// "Stop and keep the recording?" Stop finalises first, then quits.
pub fn ask_before_quit(quit: fn(&mut App), cx: &mut App) {
    // Quit arrives inside its window's update, where the main window can't
    // be found or reached: asking from there would quit without asking.
    cx.defer(move |cx| ask_now(quit, cx));
}

fn ask_now(quit: fn(&mut App), cx: &mut App) {
    let Some((handle, _)) = workspace::main_workspace(cx) else {
        finish(
            Finish {
                then: Some(Box::new(quit)),
                quitting: true,
            },
            cx,
        );
        return;
    };
    handle
        .update(cx, |_, window, cx| {
            window.open_alert_dialog(cx, move |alert, _, _| {
                alert
                    .title("Quit Kanso?")
                    .description("Stop and keep the recording?")
                    .show_cancel(true)
                    .ok_text("Stop and Quit")
                    .on_ok(move |_, _, cx| {
                        finish(
                            Finish {
                                then: Some(Box::new(quit)),
                                quitting: true,
                            },
                            cx,
                        );
                        true
                    })
            });
        })
        .ok();
}

/// A quit that gave no chance to ask (the Dock, logging out): the recording
/// stops and its folder stays, so the next launch offers to recover it.
// no-prop-test: glue.
pub fn stop_for_exit(cx: &mut App) {
    if let Some(active) = cx.default_global::<Recording>().active.take() {
        for capture in active.mic.into_iter().chain(active.system) {
            let _ = capture.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::recording_model::PendingTranscript;

    #[test]
    fn the_marker_round_trips_through_json() {
        let pending = PendingTranscript {
            note_id: "note:rec-a".into(),
            file_ids: vec!["file:rec-a-1".into(), "file:rec-a-2".into()],
            duration_secs: 4000,
        };
        let json = serde_json::to_vec(&pending).unwrap();
        assert_eq!(
            serde_json::from_slice::<PendingTranscript>(&json).unwrap(),
            pending
        );
    }
}
