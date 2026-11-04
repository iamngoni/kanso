//! System audio on Linux: a PulseAudio or PipeWire monitor source.
//!
//! A sink's monitor is an ordinary source named `<sink>.monitor` that
//! carries what the sink plays. `cpal`'s PulseAudio host lists sources, so
//! the monitors are the sources the rules in [`crate::system_audio`] pick
//! out. PipeWire answers the same protocol through `pipewire-pulse`, which
//! nearly every PipeWire desktop runs, so one host covers both. With no
//! sound server, or one without a monitor, the choice is unavailable and the
//! Record sheet says why; bare ALSA has no monitors.

use cpal::traits::{DeviceTrait, HostTrait};

use crate::error::AudioError;
use crate::loopback::{self, system_error};
use crate::system_audio::{
    Availability, SystemAudioBackend, SystemAudioDevice, SystemAudioSource, SystemAudioTarget,
    Unavailable, is_monitor_source, pick_default_monitor, resolve_device,
};

/// A sink's monitor source.
pub(crate) struct Monitor;

fn host() -> Result<cpal::Host, ()> {
    cpal::host_from_id(cpal::HostId::PulseAudio).map_err(|_| ())
}

/// Every monitor source, the default sink's first, with the `cpal` device
/// behind each entry.
fn monitors(host: &cpal::Host) -> Vec<(SystemAudioDevice, cpal::Device)> {
    let default_sink = host
        .default_output_device()
        .and_then(|d| d.id().ok())
        .map(|id| id.id().to_string());
    let Ok(all) = host.input_devices() else {
        return Vec::new();
    };
    let mut found: Vec<(String, SystemAudioDevice, cpal::Device)> = all
        .filter_map(|device| {
            let id = device.id().ok()?;
            let source = id.id().to_string();
            if !is_monitor_source(&source) {
                return None;
            }
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| source.clone());
            Some((
                source,
                SystemAudioDevice {
                    id: id.to_string(),
                    name,
                    is_default: false,
                },
                device,
            ))
        })
        .collect();
    let names: Vec<String> = found.iter().map(|(source, _, _)| source.clone()).collect();
    if let Some(ix) = pick_default_monitor(&names, default_sink.as_deref()) {
        found[ix].1.is_default = true;
        found.swap(0, ix);
    }
    found
        .into_iter()
        .map(|(_, d, device)| (d, device))
        .collect()
}

impl SystemAudioBackend for Monitor {
    fn availability(&self) -> Availability {
        match host() {
            Err(()) => Availability::Unavailable(Unavailable::NoSoundServer),
            Ok(host) if monitors(&host).is_empty() => {
                Availability::Unavailable(Unavailable::NoMonitorSource)
            }
            Ok(_) => Availability::Available,
        }
    }

    fn devices(&self) -> Vec<SystemAudioDevice> {
        host()
            .map(|host| monitors(&host).into_iter().map(|(d, _)| d).collect())
            .unwrap_or_default()
    }

    fn open(&self, target: &SystemAudioTarget) -> Result<Box<dyn SystemAudioSource>, AudioError> {
        let host =
            host().map_err(|()| AudioError::SystemAudio(Unavailable::NoSoundServer.message()))?;
        let monitors = monitors(&host);
        let devices: Vec<SystemAudioDevice> = monitors.iter().map(|(d, _)| d.clone()).collect();
        if devices.is_empty() {
            return Err(AudioError::SystemAudio(
                Unavailable::NoMonitorSource.message(),
            ));
        }
        let chosen = resolve_device(&devices, target)?.id.clone();
        let (_, device) = monitors
            .into_iter()
            .find(|(d, _)| d.id == chosen)
            .ok_or_else(|| AudioError::SystemAudio("that monitor isn't there".into()))?;
        let config = device.default_input_config().map_err(system_error)?;
        loopback::open(&device, config)
    }
}
