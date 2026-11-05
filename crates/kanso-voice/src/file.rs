//! Transcribing an audio file: decoded a packet at a time (Ogg Opus by
//! libopus, found by its OpusHead; the rest by Symphonia, both in
//! `kanso-audio`),
//! mixed down and resampled to 16 kHz as it comes, and cut at pauses
//! ([`chunk::plan_cut`]) into pieces the model takes one by one, so only a
//! piece (at most 25 seconds) is ever held and progress can be told.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub use symphonia::core::io::MediaSource;

use kanso_audio::{Speaker, energy, speaker_of};

use crate::VoiceError;
use crate::audio::{MODEL_RATE, Resampler16k, downmix};
use crate::chunk::{plan_cut, split};
use crate::engine::Parakeet;
use crate::text::join_pieces;

/// The audio files Kanso reads, by extension, with their content type.
pub const FORMATS: [(&str, &str); 12] = [
    ("m4a", "audio/mp4"),
    ("mp4", "audio/mp4"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("aif", "audio/aiff"),
    ("aiff", "audio/aiff"),
    ("caf", "audio/x-caf"),
    ("flac", "audio/flac"),
    ("ogg", "audio/ogg"),
    ("oga", "audio/ogg"),
    ("opus", "audio/ogg"),
    ("alac", "audio/mp4"),
];

/// The content type of an audio file Kanso reads, by its extension (any
/// case). Pure.
pub fn content_type(extension: &str) -> Option<&'static str> {
    let extension = extension.to_ascii_lowercase();
    FORMATS
        .iter()
        .find(|(ext, _)| *ext == extension)
        .map(|(_, content_type)| *content_type)
}

/// The extension Symphonia is hinted with for a content type. Pure.
pub fn extension(content_type: &str) -> Option<&'static str> {
    let content_type = content_type.to_ascii_lowercase();
    let content_type = match content_type.as_str() {
        "audio/x-m4a" | "audio/m4a" => "audio/mp4",
        "audio/x-wav" | "audio/wave" => "audio/wav",
        "audio/x-aiff" => "audio/aiff",
        "audio/mp3" => "audio/mpeg",
        "audio/x-flac" => "audio/flac",
        other => other,
    };
    FORMATS
        .iter()
        .find(|(_, ty)| *ty == content_type)
        .map(|(ext, _)| *ext)
}

/// How far through a file transcription is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// How much of the recording has been read.
    pub done: Duration,
    /// How long it is, when the file says.
    pub total: Option<Duration>,
}

impl Progress {
    /// The share done, 0 to 1, when the length is known. Pure.
    pub fn share(&self) -> Option<f32> {
        let total = self.total?.as_secs_f64();
        if total <= 0. {
            return Some(1.);
        }
        Some((self.done.as_secs_f64() / total).clamp(0., 1.) as f32)
    }
}

/// What [`decode_pieces`] hands each piece to.
pub type PieceSink<'a> = dyn FnMut(&[f32], Progress) -> Result<(), VoiceError> + 'a;

/// What [`decode_labelled`] hands each piece to: the piece, how far the file
/// has been read, and who the two channels say is speaking.
pub type LabelledSink<'a> =
    dyn FnMut(&[f32], Progress, Option<Speaker>) -> Result<(), VoiceError> + 'a;

/// Decodes `source` (its extension, when known, as a hint) to 16 kHz mono
/// and hands it to `piece` cut at pauses, in order, with how far the file
/// has been read. Stops with [`VoiceError::Cancelled`] once `cancel` is set.
pub fn decode_pieces(
    source: Box<dyn MediaSource>,
    extension: Option<&str>,
    cancel: &AtomicBool,
    piece: &mut PieceSink<'_>,
) -> Result<(), VoiceError> {
    decode(source, extension, cancel, false, &mut |samples, at, _| {
        piece(samples, at)
    })
}

/// [`decode_pieces`] for a recording made in Kanso. When it has two channels
/// (the microphone on the left, system audio on the right), each piece also
/// says who is speaking: the channel with more energy over that piece.
/// Mono recordings give `None`.
pub fn decode_labelled(
    source: Box<dyn MediaSource>,
    extension: Option<&str>,
    cancel: &AtomicBool,
    piece: &mut LabelledSink<'_>,
) -> Result<(), VoiceError> {
    decode(source, extension, cancel, true, piece)
}

/// Each channel of a two-channel file at 16 kHz, kept beside the mix so a
/// piece can be attributed.
struct Stereo {
    left: Option<Resampler16k>,
    right: Option<Resampler16k>,
    pending_left: Vec<f32>,
    pending_right: Vec<f32>,
}

impl Stereo {
    fn new(rate: u32) -> Result<Self, VoiceError> {
        Ok(Stereo {
            left: Some(Resampler16k::new(rate).map_err(VoiceError::Audio)?),
            right: Some(Resampler16k::new(rate).map_err(VoiceError::Audio)?),
            pending_left: Vec::new(),
            pending_right: Vec::new(),
        })
    }

    /// Resamples `samples` (interleaved pairs) and appends the mix of the
    /// two channels to `mono`.
    fn push(&mut self, samples: &[f32], mono: &mut Vec<f32>) {
        let (left, right): (Vec<f32>, Vec<f32>) = samples
            .as_chunks::<2>()
            .0
            .iter()
            .map(|f| (f[0], f[1]))
            .unzip();
        let start = self.pending_left.len();
        if let (Some(l), Some(r)) = (self.left.as_mut(), self.right.as_mut()) {
            l.push(&left, &mut self.pending_left);
            r.push(&right, &mut self.pending_right);
        }
        mix_into(
            &self.pending_left[start..],
            &self.pending_right[start..],
            mono,
        );
    }

    fn finish(&mut self, mono: &mut Vec<f32>) {
        let start = self.pending_left.len();
        if let Some(l) = self.left.take() {
            l.finish(&mut self.pending_left);
        }
        if let Some(r) = self.right.take() {
            r.finish(&mut self.pending_right);
        }
        mix_into(
            &self.pending_left[start..],
            &self.pending_right[start..],
            mono,
        );
    }

    /// Who speaks in the first `len` samples, which are then dropped.
    fn take(&mut self, len: usize) -> Option<Speaker> {
        let len = len
            .min(self.pending_left.len())
            .min(self.pending_right.len());
        let speaker = speaker_of(
            energy(&self.pending_left[..len]),
            energy(&self.pending_right[..len]),
        );
        self.pending_left.drain(..len);
        self.pending_right.drain(..len);
        speaker
    }
}

/// Appends the mean of `left` and `right` (the shorter's length) to `out`.
fn mix_into(left: &[f32], right: &[f32], out: &mut Vec<f32>) {
    out.extend(left.iter().zip(right).map(|(l, r)| (l + r) * 0.5));
}

fn decode(
    source: Box<dyn MediaSource>,
    extension: Option<&str>,
    cancel: &AtomicBool,
    labelled: bool,
    piece: &mut LabelledSink<'_>,
) -> Result<(), VoiceError> {
    let unreadable = |e: kanso_audio::AudioError| VoiceError::Audio(e.to_string());
    // Ogg Opus (told by its OpusHead, whatever the extension) goes through
    // libopus; everything else through Symphonia.
    let mut audio = kanso_audio::open(source, extension).map_err(unreadable)?;
    let rate = audio.sample_rate();
    let total = audio
        .total_frames()
        .map(|frames| Duration::from_secs_f64(frames as f64 / f64::from(rate)));
    let mut resampler = Resampler16k::new(rate).map_err(VoiceError::Audio)?;
    let mut stereo: Option<Stereo> = None;
    let mut pending: Vec<f32> = Vec::new();
    let mut samples: Vec<f32> = Vec::new();
    let mut frames_read = 0u64;
    let progress = |frames: u64| Progress {
        done: Duration::from_secs_f64(frames as f64 / f64::from(rate)),
        total,
    };
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(VoiceError::Cancelled);
        }
        let Some(channels) = audio.read(&mut samples).map_err(unreadable)? else {
            break;
        };
        if labelled && channels == 2 {
            if stereo.is_none() {
                stereo = Some(Stereo::new(rate)?);
            }
            frames_read += (samples.len() / 2) as u64;
            if let Some(both) = stereo.as_mut() {
                both.push(&samples, &mut pending);
            }
        } else {
            let mono = downmix(&samples, channels);
            frames_read += mono.len() as u64;
            resampler.push(&mono, &mut pending);
        }
        while let Some(cut) = plan_cut(&pending, MODEL_RATE) {
            let speaker = stereo.as_mut().and_then(|both| both.take(cut));
            piece(&pending[..cut], progress(frames_read), speaker)?;
            pending.drain(..cut);
        }
    }
    if let Some(both) = stereo.as_mut() {
        both.finish(&mut pending);
    } else {
        resampler.finish(&mut pending);
    }
    for range in split(&pending, MODEL_RATE) {
        if cancel.load(Ordering::Relaxed) {
            return Err(VoiceError::Cancelled);
        }
        let speaker = stereo.as_ref().map(|both| {
            let end = range
                .end
                .min(both.pending_left.len())
                .min(both.pending_right.len());
            let start = range.start.min(end);
            speaker_of(
                energy(&both.pending_left[start..end]),
                energy(&both.pending_right[start..end]),
            )
        });
        piece(&pending[range], progress(frames_read), speaker.flatten())?;
    }
    Ok(())
}

/// One transcribed piece and where in the recording it starts.
#[derive(Clone, Debug, PartialEq)]
pub struct Timed {
    /// Time from the start of the recording.
    pub start: Duration,
    pub text: String,
    /// Who spoke, when the recording keeps the microphone and system audio
    /// on separate channels.
    pub speaker: Option<Speaker>,
}

/// The time `samples` 16 kHz samples into a recording. Pure.
pub fn time_at(samples: usize) -> Duration {
    Duration::from_secs_f64(samples as f64 / f64::from(MODEL_RATE))
}

/// Transcribes the audio in `source` with the model in `model_dir`, a
/// piece at a time with where each starts, telling `progress` how far it
/// has got after each piece. Pieces with no words are left out; nothing
/// heard at all is [`VoiceError::NothingHeard`].
// no-prop-test: runs the real model; `time_at` and `decode_pieces`
// have the properties.
pub fn transcribe_timed(
    source: Box<dyn MediaSource>,
    extension: Option<&str>,
    model_dir: &std::path::Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<Vec<Timed>, VoiceError> {
    let mut model = Parakeet::load(model_dir)?;
    let mut pieces = Vec::new();
    let mut samples_before = 0usize;
    decode_pieces(source, extension, cancel, &mut |samples, at| {
        let start = time_at(samples_before);
        samples_before += samples.len();
        let text = model.transcribe(samples, cancel)?;
        if !text.trim().is_empty() {
            pieces.push(Timed {
                start,
                text,
                speaker: None,
            });
        }
        progress(at);
        Ok(())
    })?;
    if pieces.is_empty() {
        return Err(VoiceError::NothingHeard);
    }
    Ok(pieces)
}

/// One part of a recording to transcribe: its bytes' reader and the
/// extension hint.
pub type Part = (Box<dyn MediaSource>, Option<String>);

/// Transcribes a recording made of several `parts` as one timeline: the
/// model loads once, and each piece's start counts from the start of the
/// first part. `total` is the whole recording's length when known, for the
/// progress. Nothing heard is [`VoiceError::NothingHeard`].
// no-prop-test: runs the real model; `time_at` and `decode_pieces` have
// the properties.
pub fn transcribe_parts(
    parts: Vec<Part>,
    total: Option<Duration>,
    model_dir: &std::path::Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<Vec<Timed>, VoiceError> {
    let mut model = Parakeet::load(model_dir)?;
    let mut pieces = Vec::new();
    let mut samples_before = 0usize;
    for (source, extension) in parts {
        let base = time_at(samples_before);
        decode_labelled(
            source,
            extension.as_deref(),
            cancel,
            &mut |samples, at, speaker| {
                let start = time_at(samples_before);
                samples_before += samples.len();
                let text = model.transcribe(samples, cancel)?;
                if !text.trim().is_empty() {
                    pieces.push(Timed {
                        start,
                        text,
                        speaker,
                    });
                }
                progress(Progress {
                    done: base + at.done,
                    total,
                });
                Ok(())
            },
        )?;
    }
    if pieces.is_empty() {
        return Err(VoiceError::NothingHeard);
    }
    Ok(pieces)
}

/// Transcribes the audio in `source` with the model in `model_dir`,
/// telling `progress` how far it has got after each piece. Nothing heard
/// is [`VoiceError::NothingHeard`].
// no-prop-test: wraps `transcribe_timed`; `join_pieces` has the properties.
pub fn transcribe(
    source: Box<dyn MediaSource>,
    extension: Option<&str>,
    model_dir: &std::path::Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<String, VoiceError> {
    let pieces = transcribe_timed(source, extension, model_dir, cancel, progress)?;
    let text = join_pieces(pieces.iter().map(|p| p.text.as_str()));
    if text.trim().is_empty() {
        return Err(VoiceError::NothingHeard);
    }
    Ok(text)
}

/// An audio file's bytes, as Symphonia reads them.
pub fn from_bytes(bytes: Vec<u8>) -> Box<dyn MediaSource> {
    Box::new(std::io::Cursor::new(bytes))
}

/// An audio file on disk, as Symphonia reads it.
pub fn from_path(path: &std::path::Path) -> Result<Box<dyn MediaSource>, VoiceError> {
    let file = std::fs::File::open(path).map_err(|e| VoiceError::Audio(e.to_string()))?;
    Ok(Box::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::MAX_SECONDS;
    use proptest::prelude::*;

    /// A WAV of `frames` frames of a quiet tone with pauses, 16-bit.
    fn wav(rate: u32, channels: u16, frames: usize) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        for i in 0..frames {
            // A second of tone, then half a second of silence.
            let t = i as f32 / rate as f32;
            let on = t.fract() < 0.66;
            let s = if on {
                (t * 440. * std::f32::consts::TAU).sin() * 0.2
            } else {
                0.
            };
            for _ in 0..channels {
                writer.write_sample((s * 32_767.) as i16).unwrap();
            }
        }
        writer.finalize().unwrap();
        bytes.into_inner()
    }

    fn decode(
        bytes: Vec<u8>,
        cancel: &AtomicBool,
    ) -> Result<(Vec<usize>, Vec<Progress>), VoiceError> {
        let mut lengths = Vec::new();
        let mut seen = Vec::new();
        decode_pieces(
            from_bytes(bytes),
            Some("wav"),
            cancel,
            &mut |samples, at| {
                lengths.push(samples.len());
                seen.push(at);
                Ok(())
            },
        )?;
        Ok((lengths, seen))
    }

    /// A stereo WAV of tone and pauses, the left channel at `left` gain and
    /// the right at `right`.
    fn wav_lr(rate: u32, frames: usize, left: f32, right: f32) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        for i in 0..frames {
            let t = i as f32 / rate as f32;
            let s = if t.fract() < 0.66 {
                (t * 440. * std::f32::consts::TAU).sin() * 0.3
            } else {
                0.
            };
            writer.write_sample((s * left * 32_767.) as i16).unwrap();
            writer.write_sample((s * right * 32_767.) as i16).unwrap();
        }
        writer.finalize().unwrap();
        bytes.into_inner()
    }

    fn labels(bytes: Vec<u8>) -> (usize, Vec<Option<Speaker>>) {
        let mut total = 0;
        let mut speakers = Vec::new();
        decode_labelled(
            from_bytes(bytes),
            Some("wav"),
            &AtomicBool::new(false),
            &mut |samples, _, speaker| {
                total += samples.len();
                speakers.push(speaker);
                Ok(())
            },
        )
        .unwrap();
        (total, speakers)
    }

    #[test]
    fn a_stereo_recording_labels_the_louder_channel_and_a_mono_one_nobody() {
        let (_, you) = labels(wav_lr(16_000, 16_000 * 40, 0.8, 0.1));
        assert!(you.len() >= 2 && you.iter().all(|s| *s == Some(Speaker::You)));
        let (_, meeting) = labels(wav_lr(16_000, 16_000 * 40, 0.1, 0.8));
        assert!(meeting.iter().all(|s| *s == Some(Speaker::Meeting)));
        let (_, even) = labels(wav_lr(16_000, 16_000 * 10, 0.5, 0.5));
        assert!(even.iter().all(Option::is_none));
        let (_, mono) = labels(wav(16_000, 1, 16_000 * 10));
        assert!(mono.iter().all(Option::is_none));
        // The plain decoder never labels, and hears the same pieces.
        let (lengths, _) = decode(
            wav_lr(16_000, 16_000 * 40, 0.8, 0.1),
            &AtomicBool::new(false),
        )
        .unwrap();
        let (total, _) = labels(wav_lr(16_000, 16_000 * 40, 0.8, 0.1));
        assert_eq!(lengths.iter().sum::<usize>(), total);
    }

    #[test]
    fn the_spoken_fixture_decodes_whole() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hello.wav"
        ))
        .unwrap();
        let reader = hound::WavReader::new(std::io::Cursor::new(&bytes)).unwrap();
        let frames = reader.duration() as usize;
        let (lengths, progress) = decode(bytes, &AtomicBool::new(false)).unwrap();
        assert_eq!(lengths.iter().sum::<usize>(), frames);
        let last = progress.last().unwrap();
        assert_eq!(last.share(), Some(1.));
    }

    #[test]
    fn what_isnt_audio_is_refused_and_a_cancel_stops() {
        let err = decode(b"not audio at all".to_vec(), &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, VoiceError::Audio(_)), "{err:?}");
        let cancel = AtomicBool::new(true);
        let err = decode(wav(16_000, 1, 16_000), &cancel).unwrap_err();
        assert!(matches!(err, VoiceError::Cancelled));
    }

    /// Two seconds of tone as an Ogg Opus recording.
    fn opus_recording(channels: kanso_audio::Channels) -> Vec<u8> {
        use std::sync::{Arc, Mutex};

        struct Out(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for Out {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl kanso_audio::DurableWrite for Out {
            fn sync(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let n = channels.count();
        let pcm: Vec<f32> = (0..96_000 * n)
            .map(|i| ((i / n) as f32 * 0.03).sin() * 0.3)
            .collect();
        let shared = Arc::new(Mutex::new(Vec::new()));
        let handle = shared.clone();
        let mut writer = kanso_audio::PartWriter::new(
            move |_| Ok(Out(handle.clone())),
            kanso_audio::EncodeConfig::new(channels),
        )
        .unwrap();
        writer.push(&pcm).unwrap();
        writer.finish().unwrap();
        shared.lock().unwrap().clone()
    }

    #[test]
    fn ogg_opus_decodes_by_its_magic_whatever_the_extension() {
        for channels in [kanso_audio::Channels::Mono, kanso_audio::Channels::Stereo] {
            let bytes = opus_recording(channels);
            for extension in [Some("opus"), Some("ogg"), None] {
                let mut seen = Vec::new();
                decode_pieces(
                    from_bytes(bytes.clone()),
                    extension,
                    &AtomicBool::new(false),
                    &mut |samples, at| {
                        seen.push((samples.len(), at));
                        Ok(())
                    },
                )
                .unwrap();
                // Two seconds at 16 kHz, give or take the resampler's edge.
                let total: usize = seen.iter().map(|(n, _)| n).sum();
                assert!((31_000..=33_000).contains(&total), "{total}");
                assert_eq!(seen.last().unwrap().1.share(), Some(1.));
            }
        }
    }

    #[test]
    fn formats_go_both_ways() {
        assert_eq!(content_type("opus"), Some("audio/ogg"));
        assert_eq!(extension("audio/ogg"), Some("ogg"));
        assert_eq!(content_type("M4A"), Some("audio/mp4"));
        assert_eq!(content_type("txt"), None);
        assert_eq!(extension("audio/x-m4a"), Some("m4a"));
        assert_eq!(extension("audio/mpeg"), Some("mp3"));
        assert_eq!(extension("image/png"), None);
        for (ext, ty) in FORMATS {
            assert_eq!(content_type(ext), Some(ty));
            assert_eq!(content_type(extension(ty).unwrap()), Some(ty));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        })]

        /// A file of any common rate and channel count decodes to as many
        /// 16 kHz samples as its length takes, in pieces the model takes at
        /// once, with progress that only grows and ends at the whole file.
        #[test]
        fn prop_files_decode_to_their_length_in_pieces(
            rate in prop::sample::select(vec![8_000u32, 16_000, 22_050, 44_100, 48_000]),
            channels in 1u16..3,
            seconds in 0.2f32..70.,
        ) {
            let frames = (seconds * rate as f32) as usize;
            let (lengths, progress) = decode(wav(rate, channels, frames), &AtomicBool::new(false)).unwrap();
            let expected = (frames as f64 * 16_000. / f64::from(rate)).round() as usize;
            let got: usize = lengths.iter().sum();
            prop_assert!(got.abs_diff(expected) <= 1, "{got} vs {expected}");
            for len in &lengths {
                prop_assert!(*len <= MAX_SECONDS as usize * 16_000);
            }
            for pair in progress.windows(2) {
                prop_assert!(pair[0].done <= pair[1].done);
            }
            let last = progress.last().unwrap();
            prop_assert_eq!(last.share(), Some(1.));
        }

        /// Each piece of a stereo recording is labelled by the louder
        /// channel, whatever the rate, and the pieces still add up to the
        /// file's length.
        #[test]
        fn prop_pieces_follow_the_louder_channel(
            rate in prop::sample::select(vec![16_000u32, 44_100, 48_000]),
            seconds in 3.0f32..45.,
            left in 0.05f32..1.0,
            right in 0.05f32..1.0,
        ) {
            prop_assume!((left - right).abs() > 0.05);
            let frames = (seconds * rate as f32) as usize;
            let (total, speakers) = labels(wav_lr(rate, frames, left, right));
            let expected = (frames as f64 * 16_000. / f64::from(rate)).round() as usize;
            prop_assert!(total.abs_diff(expected) <= 1);
            let want = if left > right { Speaker::You } else { Speaker::Meeting };
            for speaker in speakers {
                prop_assert_eq!(speaker, Some(want));
            }
        }

        /// Time is samples over 16 kHz, in step as pieces add up.
        #[test]
        fn prop_time_at_adds_up(a in 0usize..10_000_000, b in 0usize..10_000_000) {
            let sum = time_at(a).as_secs_f64() + time_at(b).as_secs_f64();
            prop_assert!((time_at(a + b).as_secs_f64() - sum).abs() < 1e-6);
            prop_assert_eq!(time_at(0), Duration::ZERO);
        }

        /// The share done stays within 0 to 1.
        #[test]
        fn prop_share_is_bounded(done in 0u64..10_000, total in 0u64..10_000) {
            let p = Progress { done: Duration::from_millis(done), total: Some(Duration::from_millis(total)) };
            let share = p.share().unwrap();
            prop_assert!((0. ..=1.).contains(&share));
            prop_assert_eq!(Progress { total: None, ..p }.share(), None);
        }
    }
}
