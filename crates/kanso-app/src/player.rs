//! The recording player card (issue #38, steps 5 and 6): over a note's audio
//! line in the preview, a play button, a scrubber with the time at both
//! ends, the speed and the volume.
//!
//! ```text
//! (▶)  12:34 ━━━━━●─────────── 52:00   1×   🔊━━━
//! ```
//!
//! The view plays nothing itself. It owns a [`Player`] from `kanso-audio`:
//! a decoder thread filling a ring buffer for the sound card, and an atomic
//! position this card reads ten times a second while something plays, and
//! not at all while it is paused. One card, so one player, per preview (a
//! window has one): it stops when the note changes or the window closes.
//!
//! The audio opens on the first Play, not when the note does, so a note
//! with a recording in it doesn't take the speaker until asked. The length
//! is read from the files in the background so the card can show it.
//!
//! When a part's bytes haven't reached this device the card says so and
//! offers Sync Now; it checks again every couple of seconds and turns into
//! the player when they arrive.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_audio::play::{PartInput, Player, measure};
use kanso_engine::recording_player::{self, AudioMoment, PlayerKey, RecordingRef, clock};
use kanso_store::AttachmentState;
use kanso_store::Store;

use crate::player_model::{
    Phase, PlayerEvent, Shown, at_fraction, extension_for, fraction, is_audio, next_speed,
    progress, speed_label,
};
use crate::theme::Colors;
use crate::zoom::px;

/// Files of a locked note opened with its password: by file ID, the content
/// type and the bytes.
pub type OpenedFiles = Arc<Mutex<HashMap<String, (String, Vec<u8>)>>>;

/// How often the card asks the player where it is, while something plays.
const TICK: Duration = Duration::from_millis(100);
/// How often the card looks for bytes that sync was fetching.
const RECHECK_EVERY: u32 = 20;
/// The widest the card grows: the preview's readable measure.
const CARD_MAX_WIDTH: f32 = 560.;

/// Where to go once the player is open.
#[derive(Clone, Copy, Debug, Default)]
struct Want {
    seek: Option<Seek>,
    play: bool,
}

#[derive(Clone, Copy, Debug)]
enum Seek {
    Us(u64),
    Fraction(f32),
}

/// Why the parts couldn't be read.
enum LoadError {
    NotHere,
    Missing,
    Failed(String),
}

pub struct RecordingPlayer {
    store: Arc<Store>,
    opened_files: OpenedFiles,
    /// The file keys of the notes this session has open.
    files: crate::attachments::FileKeys,
    note: Option<String>,
    body: SharedString,
    /// Whether a file is audio, by file ID, for lines the user wrote.
    audio: HashMap<String, bool>,
    recording: Option<RecordingRef>,
    phase: Phase,
    player: Option<Player>,
    /// The length once known, before the player is open.
    length_us: Option<u64>,
    want: Option<Want>,
    speed: f32,
    volume: f32,
    /// The volume before it was muted.
    unmuted: f32,
    opening: Option<Task<()>>,
    measuring: Option<Task<()>>,
    ticker: Option<Task<()>>,
    ticks: u32,
    focus: FocusHandle,
    /// In a slideshow the keys belong to the slides: the card is for the
    /// mouse there and never takes Space or the arrows.
    slide_keys: bool,
    track: Rc<Cell<Bounds<Pixels>>>,
    volume_track: Rc<Cell<Bounds<Pixels>>>,
    /// The scrubber's thumb while it is dragged: the seek happens on release.
    scrub: Option<f32>,
    shown: Shown,
}

impl RecordingPlayer {
    pub fn new(store: Arc<Store>, opened_files: OpenedFiles, cx: &mut Context<Self>) -> Self {
        RecordingPlayer {
            store,
            opened_files,
            files: crate::attachments::FileKeys::of(cx),
            note: None,
            body: SharedString::default(),
            audio: HashMap::new(),
            recording: None,
            phase: Phase::Idle,
            player: None,
            length_us: None,
            want: None,
            speed: 1.0,
            volume: 1.0,
            unmuted: 1.0,
            opening: None,
            measuring: None,
            ticker: None,
            ticks: 0,
            focus: cx.focus_handle(),
            slide_keys: false,
            track: Rc::default(),
            volume_track: Rc::default(),
            scrub: None,
            shown: Shown {
                phase: Phase::Idle,
                playing: false,
                position_secs: 0,
                length_secs: 0,
                speed_milli: 1_000,
                volume_milli: 1_000,
            },
        }
    }

    /// The note the preview shows. A different one stops what plays.
    pub fn set_note(&mut self, id: Option<&str>, cx: &mut Context<Self>) {
        if self.note.as_deref() == id {
            return;
        }
        self.note = id.map(str::to_owned);
        self.audio.clear();
        self.body = SharedString::default();
        self.recording = None;
        self.unload(cx);
        cx.notify();
    }

    /// The note's text, as it is now: finds the recording's line, and stops
    /// and forgets the player when the recording is no longer what it was.
    pub fn set_body(&mut self, text: &SharedString, cx: &mut Context<Self>) {
        if self.body == *text {
            return;
        }
        self.body = text.clone();
        let found = self.pick(text);
        let same = match (&self.recording, &found) {
            (Some(a), Some(b)) => a.file_ids == b.file_ids,
            (None, None) => true,
            _ => false,
        };
        self.recording = found;
        if !same {
            self.unload(cx);
        }
        cx.notify();
    }

    /// The recording the card is for: the one already shown if the text
    /// still has it, else the first line Kanso wrote, else the first line
    /// of links to audio the user dropped in.
    fn pick(&mut self, text: &str) -> Option<RecordingRef> {
        let lines = recording_player::link_lines(text);
        if let Some(current) = &self.recording
            && let Some(same) = lines.iter().find(|l| l.file_ids == current.file_ids)
        {
            return Some(same.clone());
        }
        if let Some(made) = lines.iter().find(|l| l.made_here) {
            return Some(made.clone());
        }
        for line in &lines {
            if self.is_audio_file(&line.file_ids[0]) {
                return Some(line.clone());
            }
        }
        None
    }

    fn is_audio_file(&mut self, id: &str) -> bool {
        if let Some(known) = self.audio.get(id) {
            return *known;
        }
        let opened = self
            .opened_files
            .lock()
            .ok()
            .and_then(|files| files.get(id).map(|(ty, _)| is_audio(ty)));
        let audio = opened.unwrap_or_else(|| {
            self.store
                .file_meta(id)
                .ok()
                .flatten()
                .is_some_and(|meta| is_audio(&meta.content_type))
        });
        self.audio.insert(id.to_owned(), audio);
        audio
    }

    // MARK: Reading the parts

    /// Where the bytes of `ids` stand on this device.
    fn availability(&self, ids: &[String]) -> Result<(), LoadError> {
        let mut missing = false;
        for id in ids {
            if self.opened_files.lock().is_ok_and(|f| f.contains_key(id)) {
                continue;
            }
            match self.store.attachment_state(id) {
                Ok(AttachmentState::Plain) => {
                    if !matches!(self.store.blob_md5(id), Ok(Some(_))) {
                        missing = true;
                    }
                }
                Ok(AttachmentState::Sealed) | Ok(AttachmentState::Missing) => {
                    return Err(LoadError::Missing);
                }
                Err(e) => return Err(LoadError::Failed(e.to_string())),
            }
        }
        if missing {
            Err(LoadError::NotHere)
        } else {
            Ok(())
        }
    }

    /// Checks the bytes and, when they are here and the length isn't known,
    /// reads it in the background.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(recording) = self.recording.clone() else {
            return;
        };
        match self.availability(&recording.file_ids) {
            Err(LoadError::NotHere) => {
                if self.phase != Phase::NotHere {
                    self.phase = Phase::NotHere;
                    cx.notify();
                }
                self.ensure_ticker(cx);
            }
            Err(LoadError::Missing) => {
                self.phase = Phase::Missing;
                cx.notify();
            }
            Err(LoadError::Failed(e)) => {
                self.phase = Phase::Failed(e);
                cx.notify();
            }
            Ok(()) => {
                if matches!(self.phase, Phase::NotHere | Phase::Missing) {
                    self.phase = Phase::Idle;
                    cx.notify();
                }
                if self.length_us.is_none() && self.measuring.is_none() && self.player.is_none() {
                    self.measure_in_background(&recording, cx);
                }
            }
        }
    }

    fn measure_in_background(&mut self, recording: &RecordingRef, cx: &mut Context<Self>) {
        let (store, opened, keys) = (
            self.store.clone(),
            self.opened_files.clone(),
            self.files.clone(),
        );
        let ids = recording.file_ids.clone();
        let task = cx.background_executor().spawn(async move {
            let parts = load_parts(&store, &opened, &keys, &ids).ok()?;
            Some((ids, measure(&parts).ok()?.total_us()))
        });
        self.measuring = Some(cx.spawn(async move |this, cx| {
            let found = task.await;
            this.update(cx, |this, cx| {
                this.measuring = None;
                if let Some((ids, length)) = found
                    && this.recording.as_ref().is_some_and(|r| r.file_ids == ids)
                {
                    this.length_us = Some(length);
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    // MARK: Playing

    /// Stops what plays and forgets the player, going back to a card that
    /// hasn't been opened.
    pub fn unload(&mut self, cx: &mut Context<Self>) {
        self.player = None;
        self.opening = None;
        self.measuring = None;
        self.ticker = None;
        self.want = None;
        self.length_us = None;
        self.scrub = None;
        self.phase = Phase::Idle;
        if self.recording.is_some() {
            self.refresh(cx);
        }
    }

    /// Opens the player if it isn't, then does `want`.
    fn open_then(&mut self, want: Want, cx: &mut Context<Self>) {
        if self.player.is_some() {
            self.apply(want);
            self.ensure_ticker(cx);
            cx.notify();
            return;
        }
        if matches!(self.phase, Phase::Opening) {
            self.want = Some(want);
            return;
        }
        let Some(recording) = self.recording.clone() else {
            return;
        };
        match self.availability(&recording.file_ids) {
            Err(LoadError::NotHere) => {
                self.phase = Phase::NotHere;
                self.ensure_ticker(cx);
                cx.notify();
                return;
            }
            Err(LoadError::Missing) => {
                self.phase = Phase::Missing;
                cx.notify();
                return;
            }
            Err(LoadError::Failed(e)) => {
                self.phase = Phase::Failed(e);
                cx.notify();
                return;
            }
            Ok(()) => {}
        }
        self.phase = Phase::Opening;
        self.want = Some(want);
        let (store, opened, keys) = (
            self.store.clone(),
            self.opened_files.clone(),
            self.files.clone(),
        );
        let ids = recording.file_ids.clone();
        let wanted = ids.clone();
        let task = cx.background_executor().spawn(async move {
            let parts = load_parts(&store, &opened, &keys, &wanted)?;
            Player::open(parts).map_err(|e| LoadError::Failed(e.to_string()))
        });
        self.opening = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.opened(ids, result, cx))
                .ok();
        }));
        cx.notify();
    }

    fn opened(
        &mut self,
        ids: Vec<String>,
        result: Result<Player, LoadError>,
        cx: &mut Context<Self>,
    ) {
        self.opening = None;
        if !self.recording.as_ref().is_some_and(|r| r.file_ids == ids) {
            // Another recording, or another note, while this opened.
            return;
        }
        match result {
            Ok(player) => {
                player.set_speed(self.speed);
                player.set_volume(self.volume);
                self.length_us = Some(player.length_us());
                self.player = Some(player);
                self.phase = Phase::Ready;
                if let Some(want) = self.want.take() {
                    self.apply(want);
                }
                self.ensure_ticker(cx);
            }
            Err(LoadError::NotHere) => self.phase = Phase::NotHere,
            Err(LoadError::Missing) => self.phase = Phase::Missing,
            Err(LoadError::Failed(e)) => self.phase = Phase::Failed(e),
        }
        cx.notify();
    }

    fn apply(&mut self, want: Want) {
        let Some(player) = &self.player else { return };
        match want.seek {
            Some(Seek::Us(at)) => player.seek_us(at),
            Some(Seek::Fraction(f)) => player.seek_us(at_fraction(f, player.length_us())),
            None => {}
        }
        if want.play {
            player.play();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        match &self.player {
            Some(player) => {
                player.toggle();
                self.ensure_ticker(cx);
                cx.notify();
            }
            None => self.open_then(
                Want {
                    seek: None,
                    play: true,
                },
                cx,
            ),
        }
    }

    /// Moves by `secs` (negative is back); nothing when nothing is open.
    pub fn skip(&mut self, secs: i32, cx: &mut Context<Self>) {
        if let Some(player) = &self.player {
            player.skip_us(i64::from(secs) * 1_000_000);
            self.ensure_ticker(cx);
            cx.notify();
        }
    }

    /// A timestamp link was clicked: plays that moment of the recording the
    /// link names. Whether the note has such a recording.
    pub fn play_moment(&mut self, moment: &AudioMoment, cx: &mut Context<Self>) -> bool {
        let Some(recording) = recording_player::recording_with(&self.body, &moment.file_id) else {
            return false;
        };
        if self.recording.as_ref().map(|r| &r.file_ids) != Some(&recording.file_ids) {
            self.recording = Some(recording);
            self.unload(cx);
        }
        self.open_then(
            Want {
                seek: Some(Seek::Us(u64::from(moment.seconds) * 1_000_000)),
                play: true,
            },
            cx,
        );
        cx.emit(PlayerEvent::Moment);
        true
    }

    fn cycle_speed(&mut self, cx: &mut Context<Self>) {
        self.speed = next_speed(self.speed);
        if let Some(player) = &self.player {
            player.set_speed(self.speed);
        }
        cx.notify();
    }

    fn set_volume(&mut self, volume: f32, cx: &mut Context<Self>) {
        self.volume = volume.clamp(0.0, 1.0);
        if self.volume > 0.0 {
            self.unmuted = self.volume;
        }
        if let Some(player) = &self.player {
            player.set_volume(self.volume);
        }
        cx.notify();
    }

    fn toggle_mute(&mut self, cx: &mut Context<Self>) {
        let to = if self.volume > 0.0 { 0.0 } else { self.unmuted };
        self.set_volume(to, cx);
    }

    /// Leaves the keyboard to the slides: the card plays and pauses from the
    /// mouse only, and a click on it does not take the focus.
    pub fn leave_keys_to_slides(&mut self) {
        self.slide_keys = true;
    }

    /// A key the card takes while it has the keyboard. Whether it did.
    /// `on_card` says the card itself has the focus, not one of its buttons
    /// (reached with Tab): Space on a button presses that button, so the
    /// card leaves Space alone there rather than playing as well.
    fn key(&mut self, keystroke: &Keystroke, on_card: bool, cx: &mut Context<Self>) -> bool {
        let m = &keystroke.modifiers;
        if self.slide_keys || m.control || m.alt || m.platform || m.function {
            return false;
        }
        match recording_player::key_action(&keystroke.key, m.shift) {
            Some(PlayerKey::PlayPause) if !on_card => return false,
            Some(PlayerKey::PlayPause) => self.toggle(cx),
            Some(PlayerKey::Skip(secs)) => self.skip(secs, cx),
            None => return false,
        }
        true
    }

    // MARK: The clock

    fn snapshot(&self) -> Shown {
        let (position, length, playing) = match &self.player {
            Some(p) => (p.position_us(), p.length_us(), p.is_playing()),
            None => (0, self.length_us.unwrap_or(0), false),
        };
        Shown {
            phase: self.phase.clone(),
            playing,
            position_secs: position / 1_000_000,
            length_secs: length / 1_000_000,
            speed_milli: (self.speed * 1_000.0).round() as u32,
            volume_milli: (self.volume * 1_000.0).round() as u32,
        }
    }

    fn ensure_ticker(&mut self, cx: &mut Context<Self>) {
        if self.ticker.is_some() {
            return;
        }
        self.ticker = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let keep = this.update(cx, |this, cx| this.tick(cx));
                if !matches!(keep, Ok(true)) {
                    break;
                }
            }
        }));
    }

    /// One tenth of a second on: redraws when what the card shows changed.
    /// Whether to go on ticking.
    fn tick(&mut self, cx: &mut Context<Self>) -> bool {
        self.ticks = self.ticks.wrapping_add(1);
        if matches!(self.phase, Phase::NotHere) {
            if self.ticks.is_multiple_of(RECHECK_EVERY) {
                self.refresh(cx);
            }
            return true;
        }
        let mut keep = matches!(self.phase, Phase::Opening);
        if let Some(player) = &self.player {
            let status = player.status();
            if let Some(error) = status.error {
                self.phase = Phase::Failed(error);
            }
            keep = status.playing;
        }
        let shown = self.snapshot();
        if shown != self.shown {
            self.shown = shown;
            cx.notify();
        }
        if !keep {
            // Paused or finished: a last look above, then quiet until the
            // next action wakes it.
            self.ticker = None;
        }
        keep
    }

    // MARK: Drawing

    fn scrub_at(&mut self, x: Pixels, cx: &mut Context<Self>) {
        let bounds = self.track.get();
        let f = fraction(
            f32::from(x),
            f32::from(bounds.origin.x),
            f32::from(bounds.size.width),
        );
        self.scrub = Some(f);
        cx.notify();
    }

    fn scrub_end(&mut self, cx: &mut Context<Self>) {
        let Some(f) = self.scrub.take() else { return };
        self.open_then(
            Want {
                seek: Some(Seek::Fraction(f)),
                play: false,
            },
            cx,
        );
    }

    fn volume_at(&mut self, x: Pixels, cx: &mut Context<Self>) {
        let bounds = self.volume_track.get();
        let f = fraction(
            f32::from(x),
            f32::from(bounds.origin.x),
            f32::from(bounds.size.width),
        );
        self.set_volume(f, cx);
    }
}

/// Reads the parts' bytes from the store (or from the files a locked note
/// opened), in order.
fn load_parts(
    store: &Store,
    opened: &OpenedFiles,
    keys: &crate::attachments::FileKeys,
    ids: &[String],
) -> Result<Vec<PartInput>, LoadError> {
    let mut parts = Vec::with_capacity(ids.len());
    for id in ids {
        let from_lock = opened.lock().ok().and_then(|f| f.get(id).cloned());
        let (content_type, bytes) = match from_lock {
            Some(file) => file,
            None => match crate::attachments::open_with(store, id, keys) {
                crate::attachments::Opened::Opened(file) => (file.content_type, file.bytes),
                crate::attachments::Opened::Locked | crate::attachments::Opened::Missing => {
                    return Err(if matches!(store.file_meta(id), Ok(Some(_))) {
                        LoadError::NotHere
                    } else {
                        LoadError::Missing
                    });
                }
            },
        };
        parts.push(PartInput::new(bytes, extension_for(&content_type)));
    }
    Ok(parts)
}

impl EventEmitter<PlayerEvent> for RecordingPlayer {}

impl Render for RecordingPlayer {
    // no-prop-test: drawing; the tests render the card and drive it.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(_) = self.recording else {
            return div().into_any_element();
        };
        let colors = Colors::get(cx);
        let shown = self.snapshot();
        let body = match &shown.phase {
            Phase::NotHere => self.unavailable(
                Lucide::CloudOff,
                "Audio not on this device yet",
                Some(
                    Button::new("player-sync-now")
                        .debug_selector(|| "player-sync-now".into())
                        .small()
                        .label("Sync Now")
                        .on_click(|_, _, cx| crate::services::sync_now(cx))
                        .into_any_element(),
                ),
                colors,
            ),
            Phase::Missing => self.unavailable(
                Lucide::TriangleAlert,
                "This recording's audio is missing",
                None,
                colors,
            ),
            Phase::Failed(why) => self.unavailable(
                Lucide::TriangleAlert,
                why,
                Some(
                    Button::new("player-retry")
                        .debug_selector(|| "player-retry".into())
                        .small()
                        .label("Try Again")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.unload(cx);
                            this.toggle(cx);
                        }))
                        .into_any_element(),
                ),
                colors,
            ),
            _ => self.controls(&shown, colors, cx),
        };
        v_flex()
            .id("recording-player")
            .key_context("RecordingPlayer")
            .track_focus(&self.focus)
            .debug_selector(|| "recording-player".into())
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let on_card = this.focus.is_focused(window);
                if this.key(&event.keystroke, on_card, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if !this.slide_keys {
                        window.focus(&this.focus, cx);
                    }
                }),
            )
            .flex_none()
            .w_full()
            .pl(px(crate::preview::TEXT_INSET))
            .pr(px(32.))
            .pt(px(12.))
            .pb(px(4.))
            .child(
                div()
                    .w_full()
                    .max_w(px(CARD_MAX_WIDTH))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.fill)
                    .px(px(12.))
                    .py(px(10.))
                    .child(body),
            )
            .into_any_element()
    }
}

impl RecordingPlayer {
    fn unavailable(
        &self,
        icon: Lucide,
        words: &str,
        action: Option<AnyElement>,
        colors: &Colors,
    ) -> AnyElement {
        h_flex()
            .gap(px(10.))
            .items_center()
            .child(
                Icon::new(icon)
                    .size(px(16.))
                    .text_color(colors.text_secondary),
            )
            .child(
                div()
                    .debug_selector(|| "player-unavailable".into())
                    .flex_1()
                    .text_size(px(13.))
                    .text_color(colors.text_secondary)
                    .child(words.to_owned()),
            )
            .children(action)
            .into_any_element()
    }

    fn controls(&self, shown: &Shown, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let length_us = self.length_us.unwrap_or(shown.length_secs * 1_000_000);
        let position_us = match self.scrub {
            Some(f) => at_fraction(f, length_us),
            None => self
                .player
                .as_ref()
                .map_or(0, |p| p.position_us().min(length_us.max(p.length_us()))),
        };
        let along = self
            .scrub
            .unwrap_or_else(|| progress(position_us, length_us));
        let playing = shown.playing;
        let opening = matches!(shown.phase, Phase::Opening);
        let total = if length_us > 0 {
            clock(length_us)
        } else {
            "–:––".to_owned()
        };
        let time = |id: &'static str, text: String, align_right: bool| {
            div()
                .debug_selector(move || id.into())
                .w(px(46.))
                .flex_none()
                .text_size(px(12.))
                .text_color(colors.text_secondary)
                .when(align_right, |el| el.text_right())
                .child(text)
        };
        let play = div()
            .id("player-toggle")
            .debug_selector(|| "player-toggle".into())
            .flex_none()
            .size(px(32.))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .bg(colors.button)
            .when(opening, |el| el.opacity(0.6))
            .child(
                Icon::new(if playing { Lucide::Pause } else { Lucide::Play })
                    .size(px(15.))
                    .text_color(colors.button_text),
            )
            .on_click(cx.listener(|this, _, _, cx| this.toggle(cx)));
        let track = self.track.clone();
        let scrubber = div()
            .id("player-scrub")
            .debug_selector(|| "player-scrub".into())
            .relative()
            .flex_1()
            .h(px(20.))
            .cursor_pointer()
            .child(
                canvas(move |bounds, _, _| track.set(bounds), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(8.))
                    .h(px(4.))
                    .rounded_full()
                    .bg(colors.divider),
            )
            .child(
                div()
                    .debug_selector(|| "player-progress".into())
                    .absolute()
                    .left_0()
                    .top(px(8.))
                    .h(px(4.))
                    .w(relative(along))
                    .rounded_full()
                    .bg(colors.accent),
            )
            .child(
                div()
                    .absolute()
                    .top(px(4.))
                    .left(relative(along))
                    .ml(px(-6.))
                    .size(px(12.))
                    .rounded_full()
                    .bg(colors.accent)
                    .border_2()
                    .border_color(colors.fill),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.scrub_at(event.position.x, cx)
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.scrub.is_some() && event.dragging() {
                    this.scrub_at(event.position.x, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.scrub_end(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.scrub_end(cx)),
            );
        let speed = Button::new("player-speed")
            .debug_selector(|| "player-speed".into())
            .ghost()
            .xsmall()
            .label(speed_label(self.speed))
            .tooltip("Playback speed")
            .on_click(cx.listener(|this, _, _, cx| this.cycle_speed(cx)));
        let muted = self.volume <= 0.0;
        let volume_track = self.volume_track.clone();
        let volume = h_flex()
            .gap(px(4.))
            .items_center()
            .child(
                Button::new("player-mute")
                    .debug_selector(|| "player-mute".into())
                    .ghost()
                    .xsmall()
                    .icon(
                        Icon::new(if muted {
                            Lucide::VolumeX
                        } else {
                            Lucide::Volume2
                        })
                        .text_color(colors.text_secondary),
                    )
                    .tooltip(if muted { "Unmute" } else { "Mute" })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_mute(cx))),
            )
            .child(
                div()
                    .id("player-volume")
                    .debug_selector(|| "player-volume".into())
                    .relative()
                    .w(px(56.))
                    .h(px(20.))
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |bounds, _, _| volume_track.set(bounds),
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(8.))
                            .h(px(4.))
                            .rounded_full()
                            .bg(colors.divider),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top(px(8.))
                            .h(px(4.))
                            .w(relative(self.volume))
                            .rounded_full()
                            .bg(colors.text_secondary),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _, cx| {
                            this.volume_at(event.position.x, cx)
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                        if event.dragging() {
                            this.volume_at(event.position.x, cx);
                        }
                    })),
            );
        h_flex()
            .gap(px(10.))
            .items_center()
            .child(play)
            .child(time("player-elapsed", clock(position_us), false))
            .child(scrubber)
            .child(time("player-total", total, true))
            .child(speed)
            .child(volume)
            .into_any_element()
    }
}

/// What the tests look at and drive.
#[cfg(test)]
impl RecordingPlayer {
    pub(crate) fn phase(&self) -> &Phase {
        &self.phase
    }

    pub(crate) fn length_secs(&self) -> Option<u64> {
        self.length_us.map(|us| us / 1_000_000)
    }

    pub(crate) fn has_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub(crate) fn is_open(&self) -> bool {
        self.player.is_some()
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.player.as_ref().is_some_and(|p| p.is_playing())
    }

    pub(crate) fn position_us(&self) -> u64 {
        self.player.as_ref().map_or(0, |p| p.position_us())
    }

    /// No sound from the test run.
    pub(crate) fn silence(&mut self) {
        self.volume = 0.0;
    }

    pub(crate) fn key_for_test(
        &mut self,
        keystroke: &Keystroke,
        on_card: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        self.key(keystroke, on_card, cx)
    }
}
