//! System audio on Windows 10 and 11: WASAPI loopback in shared mode.
//!
//! `cpal`'s WASAPI host turns an output (render) device opened as an input
//! into a loopback capture (`AUDCLNT_STREAMFLAGS_LOOPBACK`), so the device's
//! own mix is what arrives. No permission dialog is involved. The device's
//! default format is used as it is (often 32-bit float, 48 kHz, stereo, or
//! 6 to 8 channels on a surround setup); the shared mono mix and the
//! resampler take it from there.
//!
//! Loopback sends nothing while no program is playing; see
//! [`SystemAudioSource`].

use cpal::traits::{DeviceTrait, HostTrait};

use crate::error::AudioError;
use crate::loopback::{self, system_error};
use crate::system_audio::{
    Availability, SystemAudioBackend, SystemAudioDevice, SystemAudioSource, SystemAudioTarget,
    Unavailable, resolve_device,
};

/// WASAPI loopback.
pub(crate) struct Wasapi;

fn host() -> Result<cpal::Host, AudioError> {
    cpal::host_from_id(cpal::HostId::Wasapi).map_err(system_error)
}

/// Every active output, with the `cpal` device behind each entry.
fn outputs(host: &cpal::Host) -> Vec<(SystemAudioDevice, cpal::Device)> {
    let default = host
        .default_output_device()
        .and_then(|d| d.id().ok())
        .map(|id| id.to_string());
    let Ok(all) = host.output_devices() else {
        return Vec::new();
    };
    all.filter_map(|device| {
        let id = device.id().ok()?.to_string();
        let name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| id.clone());
        Some((
            SystemAudioDevice {
                is_default: default.as_deref() == Some(id.as_str()),
                id,
                name,
            },
            device,
        ))
    })
    .collect()
}

impl SystemAudioBackend for Wasapi {
    fn availability(&self) -> Availability {
        match host() {
            Ok(host) if !outputs(&host).is_empty() => Availability::Available,
            _ => Availability::Unavailable(Unavailable::NoOutputDevice),
        }
    }

    fn devices(&self) -> Vec<SystemAudioDevice> {
        host()
            .map(|host| outputs(&host).into_iter().map(|(d, _)| d).collect())
            .unwrap_or_default()
    }

    fn open(&self, target: &SystemAudioTarget) -> Result<Box<dyn SystemAudioSource>, AudioError> {
        let host = host()?;
        let outputs = outputs(&host);
        let devices: Vec<SystemAudioDevice> = outputs.iter().map(|(d, _)| d.clone()).collect();
        let chosen = resolve_device(&devices, target)?.id.clone();
        let (_, device) = outputs
            .into_iter()
            .find(|(d, _)| d.id == chosen)
            .ok_or_else(|| AudioError::SystemAudio("that sound output isn't there".into()))?;
        // A render device has no input configuration; its output format is
        // the one loopback delivers.
        let config = device.default_output_config().map_err(system_error)?;
        loopback::open(&device, config)
    }
}
