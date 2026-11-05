//! System audio read through `cpal`: the same stream the microphone uses,
//! opened on an output device (Windows loopback) or on a monitor source
//! (PulseAudio and PipeWire), and handed over as a
//! [`SystemAudioSource`].

use crate::error::AudioError;
use crate::mic::{self, Failure, Microphone};
use crate::system_audio::SystemAudioSource;

/// A running `cpal` capture of system audio.
pub(crate) struct CpalSource {
    capture: Microphone,
    failure: Failure,
}

impl SystemAudioSource for CpalSource {
    fn sample_rate(&self) -> u32 {
        self.capture.rate
    }

    fn read(&mut self, out: &mut Vec<f32>) {
        let available = self.capture.samples.slots();
        if available == 0 {
            return;
        }
        if let Ok(chunk) = self.capture.samples.read_chunk(available) {
            let (a, b) = chunk.as_slices();
            out.extend_from_slice(a);
            out.extend_from_slice(b);
            chunk.commit_all();
        }
    }

    fn take_dropped(&mut self) -> u64 {
        self.capture
            .dropped
            .swap(0, std::sync::atomic::Ordering::Relaxed)
    }

    fn failure(&self) -> Option<String> {
        self.failure.lock().ok().and_then(|f| f.clone())
    }
}

/// Maps a device error onto [`AudioError`], in words about system audio.
// no-prop-test: a lookup over error kinds.
pub(crate) fn system_error(e: cpal::Error) -> AudioError {
    match e.kind() {
        cpal::ErrorKind::DeviceNotAvailable => {
            AudioError::SystemAudio("that sound output isn't available any more".into())
        }
        cpal::ErrorKind::PermissionDenied => {
            AudioError::SystemAudio("the system won't let Kanso record sound output".into())
        }
        _ => AudioError::SystemAudio(e.to_string()),
    }
}

/// Starts reading `device` with `config`.
// no-prop-test: needs a sound device; the selection rules and the mono mix have the properties.
pub(crate) fn open(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
) -> Result<Box<dyn SystemAudioSource>, AudioError> {
    let failure: Failure = Default::default();
    let capture = mic::open_device(device, config, failure.clone()).map_err(system_error)?;
    Ok(Box::new(CpalSource { capture, failure }))
}
