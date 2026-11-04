//! The default microphone through `cpal`, as one channel of `f32` at the
//! device's own rate.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use crate::error::AudioError;

/// Why the device stopped, from its error callback.
pub type Failure = Arc<Mutex<Option<String>>>;

/// Seconds of sound the ring buffer holds between two polls. The reader
/// thread also syncs files to disk, so this leaves room for a slow `fsync`.
const BUFFER_SECS: usize = 4;

/// An open microphone: keep the stream alive to keep recording.
pub struct Microphone {
    pub stream: cpal::Stream,
    pub samples: rtrb::Consumer<f32>,
    pub rate: u32,
}

/// Maps a device error onto [`AudioError`].
// no-prop-test: a lookup over error kinds.
fn device_error(e: cpal::Error) -> AudioError {
    match e.kind() {
        cpal::ErrorKind::PermissionDenied => AudioError::MicrophoneDenied,
        cpal::ErrorKind::DeviceNotAvailable => AudioError::NoMicrophone,
        _ => AudioError::Device(e.to_string()),
    }
}

/// Opens the default input and starts it. Overruns are ignored (a few
/// samples are lost, the recording goes on); any other stream error is put in
/// `failure`.
// no-prop-test: needs a microphone; the resampler and the recorder it feeds have the properties.
pub fn open(failure: Failure) -> Result<Microphone, AudioError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(AudioError::NoMicrophone)?;
    let supported = device.default_input_config().map_err(device_error)?;
    let rate = supported.sample_rate();
    let channels = usize::from(supported.channels()).max(1);
    let config = supported.config();
    let (producer, samples) = rtrb::RingBuffer::new((rate as usize * BUFFER_SECS).max(8_192));

    let on_error = move |e: cpal::Error| {
        if matches!(
            e.kind(),
            cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied
        ) {
            return;
        }
        if let Ok(mut slot) = failure.lock() {
            slot.get_or_insert_with(|| e.to_string());
        }
    };
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, channels, producer, on_error),
        SampleFormat::F64 => build::<f64>(&device, config, channels, producer, on_error),
        SampleFormat::I8 => build::<i8>(&device, config, channels, producer, on_error),
        SampleFormat::I16 => build::<i16>(&device, config, channels, producer, on_error),
        SampleFormat::I24 => build::<cpal::I24>(&device, config, channels, producer, on_error),
        SampleFormat::I32 => build::<i32>(&device, config, channels, producer, on_error),
        SampleFormat::U8 => build::<u8>(&device, config, channels, producer, on_error),
        SampleFormat::U16 => build::<u16>(&device, config, channels, producer, on_error),
        SampleFormat::U32 => build::<u32>(&device, config, channels, producer, on_error),
        other => {
            return Err(AudioError::Device(format!(
                "the microphone sends {other} samples, which Kanso can't read"
            )));
        }
    }
    .map_err(device_error)?;
    stream.play().map_err(device_error)?;
    Ok(Microphone {
        stream,
        samples,
        rate,
    })
}

/// Calls `sink` with the mean of each whole frame of `interleaved`. Allocates
/// nothing, so the microphone's callback can use it; an incomplete last frame
/// is dropped.
pub fn for_each_mono<T: Copy>(
    interleaved: &[T],
    channels: usize,
    to_f32: impl Fn(T) -> f32,
    mut sink: impl FnMut(f32),
) {
    let channels = channels.max(1);
    let scale = 1.0 / channels as f32;
    for frame in interleaved.chunks_exact(channels) {
        let sum: f32 = frame.iter().map(|&sample| to_f32(sample)).sum();
        sink(sum * scale);
    }
}

// no-prop-test: needs a microphone; `for_each_mono` has the properties.
fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    mut producer: rtrb::Producer<f32>,
    on_error: impl FnMut(cpal::Error) + Send + 'static,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream::<T, _, _>(
        config,
        move |data: &[T], _| {
            for_each_mono(
                data,
                channels,
                |s| s.to_sample::<f32>(),
                |mono| {
                    let _ = producer.push(mono);
                },
            );
        },
        on_error,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// One output per whole frame, each the mean of its frame.
        #[test]
        fn prop_mono_is_the_mean_of_each_frame(
            channels in 1usize..5,
            samples in proptest::collection::vec(-1.0f32..=1.0, 0..200),
        ) {
            let mut out = Vec::new();
            for_each_mono(&samples, channels, |s| s, |m| out.push(m));
            prop_assert_eq!(out.len(), samples.len() / channels);
            for (frame, mono) in samples.chunks_exact(channels).zip(&out) {
                let mean = frame.iter().sum::<f32>() / channels as f32;
                prop_assert!((mean - mono).abs() < 1e-5);
            }
        }
    }
}
