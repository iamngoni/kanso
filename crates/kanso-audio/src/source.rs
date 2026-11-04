//! Any audio file as interleaved `f32` samples: Ogg Opus through libopus,
//! M4A, MP3, FLAC, WAV and the rest through Symphonia.

use std::io::{Read, Seek, SeekFrom};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

use crate::decode::OpusReader;
use crate::error::AudioError;
use crate::header::sniff;
use crate::model::SAMPLE_RATE;

/// Samples out of a file, a packet at a time.
pub trait PcmSource: Send {
    /// Samples per second per channel.
    fn sample_rate(&self) -> u32;

    /// Frames (samples per channel) in the file, when it says.
    fn total_frames(&self) -> Option<u64>;

    /// Clears `out`, then fills it with the next packet's interleaved
    /// samples. Returns the channel count, or `None` at the end.
    fn read(&mut self, out: &mut Vec<f32>) -> Result<Option<usize>, AudioError>;

    /// Moves to `frame`; the next read starts there or just before it (a
    /// lossy format may land on its own packet boundary).
    fn seek(&mut self, frame: u64) -> Result<(), AudioError>;
}

struct OpusSource<R: Read + Seek + Send>(OpusReader<R>);

impl<R: Read + Seek + Send> PcmSource for OpusSource<R> {
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_frames(&self) -> Option<u64> {
        Some(self.0.info().total_samples)
    }

    fn read(&mut self, out: &mut Vec<f32>) -> Result<Option<usize>, AudioError> {
        out.clear();
        Ok(self.0.read(out)?.map(|_| self.0.info().channels.count()))
    }

    fn seek(&mut self, frame: u64) -> Result<(), AudioError> {
        self.0.seek(frame)
    }
}

struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    rate: u32,
    total: Option<u64>,
    buffer: Option<SampleBuffer<f32>>,
    /// Frames to drop from the next packets after an inexact seek.
    skip: u64,
}

impl SymphoniaSource {
    fn open(stream: MediaSourceStream, extension: Option<&str>) -> Result<Self, AudioError> {
        let mut hint = Hint::new();
        if let Some(extension) = extension {
            hint.with_extension(extension);
        }
        let probed = symphonia::default::get_probe().format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(AudioError::NoSound)?;
        let rate = track
            .codec_params
            .sample_rate
            .filter(|rate| *rate > 0)
            .ok_or(AudioError::NoRate)?;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;
        Ok(SymphoniaSource {
            track_id: track.id,
            total: track.codec_params.n_frames,
            format,
            decoder,
            rate,
            buffer: None,
            skip: 0,
        })
    }
}

impl PcmSource for SymphoniaSource {
    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_frames(&self) -> Option<u64> {
        self.total
    }

    fn read(&mut self, out: &mut Vec<f32>) -> Result<Option<usize>, AudioError> {
        out.clear();
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                // A damaged packet is skipped, as players do.
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(e.into()),
            };
            let spec = *decoded.spec();
            let channels = spec.channels.count().max(1);
            let samples = match &mut self.buffer {
                Some(b) if b.capacity() >= decoded.capacity() * channels => b,
                slot => slot.insert(SampleBuffer::new(decoded.capacity() as u64, spec)),
            };
            samples.copy_interleaved_ref(decoded);
            let all = samples.samples();
            let frames = all.len() / channels;
            let drop = (self.skip as usize).min(frames);
            self.skip -= drop as u64;
            if drop == frames && frames > 0 {
                continue;
            }
            out.extend_from_slice(&all[drop * channels..]);
            return Ok(Some(channels));
        }
    }

    fn seek(&mut self, frame: u64) -> Result<(), AudioError> {
        let time = Time::from(frame as f64 / f64::from(self.rate));
        let seeked = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: Some(self.track_id),
            },
        )?;
        self.decoder.reset();
        // Accurate mode lands on a packet at or before the target.
        let tb = self
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .and_then(|t| t.codec_params.time_base);
        self.skip = match tb {
            Some(tb) if seeked.required_ts > seeked.actual_ts => {
                let t = tb.calc_time(seeked.required_ts - seeked.actual_ts);
                ((t.seconds as f64 + t.frac) * f64::from(self.rate)).round() as u64
            }
            _ => 0,
        };
        Ok(())
    }
}

/// Opens `source` as samples, telling Ogg Opus from everything else by its
/// first page (`OggS` carrying `OpusHead`). `extension` hints Symphonia.
pub fn open(
    mut source: Box<dyn MediaSource>,
    extension: Option<&str>,
) -> Result<Box<dyn PcmSource>, AudioError> {
    if source.is_seekable() {
        let mut magic = [0u8; 64];
        let mut got = 0;
        while got < magic.len() {
            match source.read(&mut magic[got..])? {
                0 => break,
                n => got += n,
            }
        }
        source.seek(SeekFrom::Start(0))?;
        if sniff(&magic[..got]) {
            return Ok(Box::new(OpusSource(OpusReader::open(source)?)));
        }
    }
    let stream = MediaSourceStream::new(source, Default::default());
    Ok(Box::new(SymphoniaSource::open(stream, extension)?))
}

/// [`open`] over bytes in memory.
pub fn open_bytes(
    bytes: Vec<u8>,
    extension: Option<&str>,
) -> Result<Box<dyn PcmSource>, AudioError> {
    open(Box::new(std::io::Cursor::new(bytes)), extension)
}
