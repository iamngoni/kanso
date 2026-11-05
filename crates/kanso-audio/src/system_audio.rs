//! Recording what the computer is playing (issue #38, steps 7 to 9): the
//! platform-neutral side.
//!
//! Each platform provides a [`SystemAudioBackend`]: Windows reads the
//! output device's mix with WASAPI loopback, Linux reads the sound server's
//! monitor source, and macOS (ScreenCaptureKit) plugs in the same way. A
//! backend says whether it can work on this computer ([`Availability`]),
//! lists what can be recorded ([`SystemAudioDevice`]) and opens one
//! ([`SystemAudioTarget`]) as a [`SystemAudioSource`]: mono `f32` at the
//! source's own rate, which is what the microphone gives the recorder, so
//! both feed the same 48 kHz resampler. Mixing the two and keeping their
//! clocks together is a later step; nothing here does it.
//!
//! The rules that choose a device live here, free of any sound API, so they
//! are tested on every platform.

use crate::error::AudioError;

/// One thing the computer can play to and that can be recorded: an output
/// device on Windows, a sink's monitor source on Linux.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemAudioDevice {
    /// Stable across restarts; what [`SystemAudioTarget::Device`] holds.
    pub id: String,
    /// What the picker shows.
    pub name: String,
    /// Where the computer sends sound unless told otherwise.
    pub is_default: bool,
}

/// What to record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SystemAudioTarget {
    /// Whatever the computer plays through its default output.
    #[default]
    Default,
    /// One device from [`SystemAudioBackend::devices`], by its `id`.
    Device(String),
    /// One running application's sound, by its identifier (macOS).
    App(String),
}

/// Why system audio can't be recorded on this computer. The Record sheet
/// shows [`message`](Self::message) beside the disabled choice.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Unavailable {
    /// This platform has no way to record system audio yet.
    Unsupported,
    /// No sound server answers (Linux without PulseAudio or PipeWire).
    NoSoundServer,
    /// The sound server has nothing to record from: no monitor source.
    NoMonitorSource,
    /// There is no output device to record.
    NoOutputDevice,
    /// The system refused, with what to tell the person (a permission).
    Denied(String),
}

impl Unavailable {
    pub fn message(&self) -> String {
        match self {
            Unavailable::Unsupported => "System audio can't be recorded on this system.".into(),
            Unavailable::NoSoundServer => {
                "System audio needs PulseAudio or PipeWire, and neither is running.".into()
            }
            Unavailable::NoMonitorSource => "Your sound server has no monitor source to record \
                from. A sink's monitor is what lets other programs record what it plays."
                .into(),
            Unavailable::NoOutputDevice => {
                "There is no sound output to record. Connect speakers or headphones.".into()
            }
            Unavailable::Denied(why) => why.clone(),
        }
    }
}

/// Whether system audio can be recorded here, now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable(Unavailable),
}

impl Availability {
    pub fn is_available(&self) -> bool {
        matches!(self, Availability::Available)
    }

    /// The reason to show, when it isn't.
    pub fn reason(&self) -> Option<String> {
        match self {
            Availability::Available => None,
            Availability::Unavailable(why) => Some(why.message()),
        }
    }
}

/// An open capture of system audio. Keep it alive to keep recording.
///
/// A source is not `Send`: sound APIs tie a stream to the thread that
/// opened it, so open it on the thread that reads it, as the microphone is.
///
/// A source may deliver nothing while the computer is silent (WASAPI
/// loopback sends no packets then). That is silence, not a failure; the
/// reader keeps its own clock and fills the gap.
pub trait SystemAudioSource {
    /// Samples per second of what [`read`](Self::read) returns.
    fn sample_rate(&self) -> u32;

    /// Appends the mono samples that arrived since the last call to `out`.
    fn read(&mut self, out: &mut Vec<f32>);

    /// Samples lost because the reader fell behind since the last call; the
    /// reader puts back as much silence so the clock stays true.
    fn take_dropped(&mut self) -> u64;

    /// Why the capture stopped by itself (the device was unplugged), once it
    /// has.
    fn failure(&self) -> Option<String>;
}

/// A platform's way to record system audio.
pub trait SystemAudioBackend {
    /// Whether it can work on this computer right now.
    fn availability(&self) -> Availability;

    /// What can be recorded, the default first. Empty when unavailable.
    fn devices(&self) -> Vec<SystemAudioDevice>;

    /// Starts recording `target`.
    fn open(&self, target: &SystemAudioTarget) -> Result<Box<dyn SystemAudioSource>, AudioError>;
}

/// This platform's backend; `None` where none exists yet. macOS adds its
/// own arm here.
// no-prop-test: picks a type by platform.
pub fn platform_backend() -> Option<Box<dyn SystemAudioBackend>> {
    #[cfg(windows)]
    return Some(Box::new(crate::system_audio_windows::Wasapi));
    #[cfg(target_os = "linux")]
    return Some(Box::new(crate::system_audio_linux::Monitor));
    #[cfg(not(any(windows, target_os = "linux")))]
    None
}

/// The device `target` names among `devices`. The default is the one marked
/// default, else the first; a named device that is gone is an error rather
/// than a quiet switch to another, so a recording never captures something
/// the person didn't pick.
pub fn resolve_device<'a>(
    devices: &'a [SystemAudioDevice],
    target: &SystemAudioTarget,
) -> Result<&'a SystemAudioDevice, AudioError> {
    match target {
        SystemAudioTarget::Default => devices
            .iter()
            .find(|d| d.is_default)
            .or_else(|| devices.first())
            .ok_or_else(|| AudioError::SystemAudio(Unavailable::NoOutputDevice.message())),
        SystemAudioTarget::Device(id) => devices.iter().find(|d| &d.id == id).ok_or_else(|| {
            AudioError::SystemAudio("that sound output isn't there any more".into())
        }),
        SystemAudioTarget::App(_) => Err(AudioError::SystemAudio(
            "this system records a device, not one application".into(),
        )),
    }
}

/// Whether a PulseAudio or PipeWire source is a sink's monitor: those are
/// named after the sink with `.monitor` added.
pub fn is_monitor_source(source_name: &str) -> bool {
    source_name.ends_with(".monitor")
}

/// Which of `monitors` to record by default: the monitor of the default
/// sink when it is there, else the first. `monitors` are source names.
pub fn pick_default_monitor(monitors: &[String], default_sink: Option<&str>) -> Option<usize> {
    default_sink
        .and_then(|sink| {
            let wanted = format!("{sink}.monitor");
            monitors.iter().position(|m| *m == wanted)
        })
        .or(if monitors.is_empty() { None } else { Some(0) })
}

/// The picker's choices: the default output first as "Default output", then
/// every device by name. Names that repeat get the end of their id so two
/// identical "Speakers" can be told apart. Pure.
pub fn picker_labels(devices: &[SystemAudioDevice]) -> Vec<(SystemAudioTarget, String)> {
    let mut out = vec![(SystemAudioTarget::Default, "Default output".to_string())];
    for device in devices {
        let same = devices.iter().filter(|d| d.name == device.name).count();
        let mut label = device.name.clone();
        if same > 1 {
            let skip = device.id.chars().count().saturating_sub(4);
            let tail: String = device.id.chars().skip(skip).collect();
            label = format!("{label} ({tail})");
        }
        out.push((SystemAudioTarget::Device(device.id.clone()), label));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn device(id: &str, default: bool) -> SystemAudioDevice {
        SystemAudioDevice {
            id: id.into(),
            name: format!("Output {id}"),
            is_default: default,
        }
    }

    fn devices() -> impl Strategy<Value = Vec<SystemAudioDevice>> {
        proptest::collection::vec(("[a-z0-9.]{1,12}", any::<bool>()), 0..8).prop_map(|raw| {
            raw.into_iter()
                .enumerate()
                // Unique ids, as a sound server gives.
                .map(|(i, (id, default))| device(&format!("{i}-{id}"), default))
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The default target is the marked device, else the first; it
        /// fails only when there is nothing.
        #[test]
        fn prop_default_is_the_marked_device_or_the_first(devices in devices()) {
            let got = resolve_device(&devices, &SystemAudioTarget::Default);
            match devices.iter().find(|d| d.is_default).or_else(|| devices.first()) {
                Some(want) => prop_assert_eq!(got.unwrap(), want),
                None => prop_assert!(got.is_err()),
            }
        }

        /// A named device resolves to exactly itself; a missing one never
        /// resolves to another.
        #[test]
        fn prop_named_device_is_itself_or_an_error(devices in devices(), pick in 0usize..10) {
            if let Some(want) = devices.get(pick) {
                let got = resolve_device(&devices, &SystemAudioTarget::Device(want.id.clone()));
                prop_assert_eq!(got.unwrap(), want);
            }
            let missing = SystemAudioTarget::Device("no-such-device/".into());
            prop_assert!(resolve_device(&devices, &missing).is_err());
        }

        /// The monitor chosen is a real one, the default sink's when it is
        /// listed, and there is one exactly when any is listed.
        #[test]
        fn prop_monitor_pick(
            sinks in proptest::collection::hash_set("[a-z0-9_]{1,10}", 0..6),
            default in proptest::option::of("[a-z0-9_]{1,10}"),
        ) {
            let monitors: Vec<String> = sinks.iter().map(|s| format!("{s}.monitor")).collect();
            let got = pick_default_monitor(&monitors, default.as_deref());
            prop_assert_eq!(got.is_some(), !monitors.is_empty());
            if let Some(ix) = got {
                prop_assert!(ix < monitors.len());
                if let Some(sink) = &default
                    && sinks.contains(sink)
                {
                    prop_assert_eq!(&monitors[ix], &format!("{sink}.monitor"));
                }
            }
        }

        /// Every sink's monitor name is a monitor; a name without the suffix
        /// is not.
        #[test]
        fn prop_monitor_names(sink in "[a-zA-Z0-9_.:-]{1,24}") {
            let monitor = format!("{sink}.monitor");
            prop_assert!(is_monitor_source(&monitor));
            prop_assert_eq!(is_monitor_source(&sink), sink.ends_with(".monitor"));
        }

        /// The picker offers the default, then every device once.
        #[test]
        fn prop_picker_is_complete(devices in devices()) {
            let labels = picker_labels(&devices);
            prop_assert_eq!(labels.len(), devices.len() + 1);
            prop_assert_eq!(&labels[0].0, &SystemAudioTarget::Default);
            for d in &devices {
                prop_assert!(
                    labels.iter().any(|(t, _)| *t == SystemAudioTarget::Device(d.id.clone()))
                );
            }
        }

        /// Devices that share a name are told apart; names that are already
        /// unique are shown as they are.
        #[test]
        fn prop_picker_labels_are_unambiguous(
            names in proptest::collection::vec("[A-Z][a-z]{2,6}", 1..6),
        ) {
            let devices: Vec<_> = names
                .iter()
                .enumerate()
                .map(|(i, name)| SystemAudioDevice {
                    id: format!("endpoint-{i:04}"),
                    name: name.clone(),
                    is_default: i == 0,
                })
                .collect();
            let labels = picker_labels(&devices);
            let texts: Vec<&String> = labels[1..].iter().map(|(_, l)| l).collect();
            for (i, text) in texts.iter().enumerate() {
                for (j, other) in texts.iter().enumerate() {
                    prop_assert!(i == j || text != other);
                }
                if names.iter().filter(|n| **n == names[i]).count() == 1 {
                    prop_assert_eq!(*text, &names[i]);
                }
            }
        }
    }

    #[test]
    fn named_app_is_refused_where_only_devices_exist() {
        let devices = [device("a", true)];
        let err = resolve_device(&devices, &SystemAudioTarget::App("zoom".into()));
        assert!(err.is_err());
    }

    #[test]
    fn availability_reason_is_shown_only_when_unavailable() {
        assert_eq!(Availability::Available.reason(), None);
        let no = Availability::Unavailable(Unavailable::NoMonitorSource);
        assert!(no.reason().unwrap().contains("monitor source"));
        assert!(!no.is_available());
    }

    #[test]
    fn monitor_of_the_default_sink_wins() {
        let monitors = vec!["a.monitor".to_string(), "b.monitor".to_string()];
        assert_eq!(pick_default_monitor(&monitors, Some("b")), Some(1));
        assert_eq!(pick_default_monitor(&monitors, Some("zzz")), Some(0));
        assert_eq!(pick_default_monitor(&[], Some("a")), None);
    }
}
