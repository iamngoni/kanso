//! The player on a real sound card: cpal output, and the thread that keeps
//! the ring full.
//!
//! One thread owns the stream and runs the decoder, so the player is `Send`
//! and `Sync` whatever the platform's stream type is (Core Audio's and
//! AAudio's are not always). Dropping the player stops the thread, which
//! closes the stream.

use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::engine::{Control, Engine, Output, PartInput, Shared, Step, measure};
use crate::error::AudioError;

/// How long the decoder rests when the ring is full.
const REST: Duration = Duration::from_millis(4);

/// One recording on the speakers.
pub struct Player {
    control: Control,
    thread: Option<JoinHandle<()>>,
}

impl Deref for Player {
    type Target = Control;
    fn deref(&self) -> &Control {
        &self.control
    }
}

fn device_error(e: cpal::Error) -> AudioError {
    AudioError::Device(e.to_string())
}

impl Player {
    /// Opens `parts` (one recording, in order) on the default output. The
    /// recording is paused at its start.
    // no-prop-test: needs a sound card; the engine it drives has the properties.
    pub fn open(parts: Vec<PartInput>) -> Result<Player, AudioError> {
        if parts.is_empty() {
            return Err(AudioError::NoSound);
        }
        let timeline = measure(&parts)?;
        let shared = Shared::new(timeline.total_us());
        let control = Control::new(shared.clone());
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), AudioError>>(1);
        let thread = thread::Builder::new()
            .name("kanso-player".into())
            .spawn(move || run(shared, parts, timeline, ready_tx))
            .map_err(AudioError::Io)?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Player {
                control,
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(AudioError::Device("the player thread died".into())),
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.control.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

// no-prop-test: needs a sound card; the engine it drives has the properties.
fn run(
    shared: Arc<Shared>,
    parts: Vec<PartInput>,
    timeline: super::timeline::Timeline,
    ready: mpsc::SyncSender<Result<(), AudioError>>,
) {
    let built = (|| {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::Device("there's no speaker to play on".into()))?;
        let supported = device.default_output_config().map_err(device_error)?;
        let rate = supported.sample_rate();
        let channels = usize::from(supported.channels()).max(1);
        let config = supported.config();
        let engine = Engine::new(shared.clone(), parts, timeline, rate);
        let Engine {
            decoder, output, ..
        } = engine;
        let failure = shared.clone();
        let on_error = move |e: cpal::Error| {
            if !matches!(
                e.kind(),
                cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied
            ) {
                failure.device_failed(e.to_string());
            }
        };
        let stream = match supported.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, config, channels, output, on_error),
            SampleFormat::F64 => build::<f64>(&device, config, channels, output, on_error),
            SampleFormat::I8 => build::<i8>(&device, config, channels, output, on_error),
            SampleFormat::I16 => build::<i16>(&device, config, channels, output, on_error),
            SampleFormat::I32 => build::<i32>(&device, config, channels, output, on_error),
            SampleFormat::U8 => build::<u8>(&device, config, channels, output, on_error),
            SampleFormat::U16 => build::<u16>(&device, config, channels, output, on_error),
            SampleFormat::U32 => build::<u32>(&device, config, channels, output, on_error),
            other => {
                return Err(AudioError::Device(format!(
                    "the speaker takes {other} samples, which Kanso can't write"
                )));
            }
        }
        .map_err(device_error)?;
        stream.play().map_err(device_error)?;
        Ok((stream, decoder))
    })();
    let (stream, mut decoder) = match built {
        Ok(built) => {
            let _ = ready.send(Ok(()));
            built
        }
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    while !shared.stopped() {
        if decoder.step() != Step::Worked {
            thread::park_timeout(REST);
        }
    }
    drop(stream);
}

// no-prop-test: needs a sound card; `Output::render` has the properties.
fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    mut output: Output,
    on_error: impl FnMut(cpal::Error) + Send + 'static,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample + FromSample<f32>,
{
    // Sized for the biggest buffer a card is likely to ask for, so the
    // callback doesn't allocate.
    let mut scratch: Vec<f32> = Vec::with_capacity(16_384);
    device.build_output_stream::<T, _, _>(
        config,
        move |data: &mut [T], _| {
            scratch.resize(data.len(), 0.0);
            output.render(&mut scratch, channels);
            for (slot, value) in data.iter_mut().zip(&scratch) {
                *slot = T::from_sample(*value);
            }
        },
        on_error,
        None,
    )
}
