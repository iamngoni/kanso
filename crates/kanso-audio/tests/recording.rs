//! Round trips, parts, headers, seeking and crash safety, through libopus
//! and the Ogg container for real.

use std::io::{self, Cursor, Write};
use std::sync::{Arc, Mutex};

use kanso_audio::header::{parse_head, parse_tags};
use kanso_audio::{
    Channels, DurableWrite, EncodeConfig, FRAME_SAMPLES, OpusReader, PageIndex, PartInfo,
    PartWriter, SAMPLE_RATE, open_bytes,
};
use ogg::PacketReader;
use proptest::prelude::*;

/// A steady two-tone signal, per channel (the right is a fifth above the
/// left), plus a little deterministic noise so the encoder has bytes to
/// write.
fn signal(samples: usize, channels: usize, seed: u32) -> Vec<f32> {
    let mut state = seed | 1;
    let mut out = Vec::with_capacity(samples * channels);
    for i in 0..samples {
        for c in 0..channels {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5;
            let hz = if c == 0 { 330.0 } else { 495.0 };
            let t = i as f32 / SAMPLE_RATE as f32;
            out.push(0.4 * (std::f32::consts::TAU * hz * t).sin() + 0.02 * noise);
        }
    }
    out
}

/// A sink that keeps every part in memory, and counts syncs.
#[derive(Clone, Default)]
struct Memory {
    parts: Arc<Mutex<Vec<Arc<Mutex<Shared>>>>>,
}

#[derive(Default)]
struct Shared {
    bytes: Vec<u8>,
    syncs: usize,
}

struct MemoryPart(Arc<Mutex<Shared>>);

impl Write for MemoryPart {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableWrite for MemoryPart {
    fn sync(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().syncs += 1;
        Ok(())
    }
}

impl Memory {
    fn sink(&self) -> impl FnMut(u32) -> io::Result<MemoryPart> + use<> {
        let parts = self.parts.clone();
        move |index| {
            let shared = Arc::new(Mutex::new(Shared::default()));
            let mut parts = parts.lock().unwrap();
            assert_eq!(index as usize, parts.len(), "parts open in order");
            parts.push(shared.clone());
            Ok(MemoryPart(shared))
        }
    }

    fn bytes(&self, part: usize) -> Vec<u8> {
        self.parts.lock().unwrap()[part]
            .lock()
            .unwrap()
            .bytes
            .clone()
    }

    fn syncs(&self, part: usize) -> usize {
        self.parts.lock().unwrap()[part].lock().unwrap().syncs
    }
}

fn record(pcm: &[f32], config: EncodeConfig, chunk: usize) -> (Memory, Vec<PartInfo>) {
    let memory = Memory::default();
    let mut writer = PartWriter::new(memory.sink(), config).unwrap();
    for piece in pcm.chunks(chunk.max(1)) {
        writer.push(piece).unwrap();
    }
    let parts = writer.finish().unwrap();
    (memory, parts)
}

fn decode(bytes: Vec<u8>) -> (OpusReader<Cursor<Vec<u8>>>, Vec<f32>) {
    let mut reader = OpusReader::open(Cursor::new(bytes)).unwrap();
    let pcm = reader.read_to_end().unwrap();
    (reader, pcm)
}

/// How much of `a`'s energy `b` accounts for: 1.0 when they are the same.
fn similarity(a: &[f32], b: &[f32]) -> f64 {
    let (mut dot, mut ea, mut eb) = (0f64, 0f64, 0f64);
    for (x, y) in a.iter().zip(b) {
        dot += f64::from(*x) * f64::from(*y);
        ea += f64::from(*x).powi(2);
        eb += f64::from(*y).powi(2);
    }
    if ea == 0.0 || eb == 0.0 {
        return if ea == eb { 1.0 } else { 0.0 };
    }
    dot / (ea.sqrt() * eb.sqrt())
}

#[test]
fn the_headers_are_correct() {
    let pcm = signal(SAMPLE_RATE as usize, 2, 1);
    let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Stereo), 777);
    let bytes = memory.bytes(0);
    // Page 1 holds OpusHead alone, page 2 OpusTags alone, both granule 0.
    assert_eq!(&bytes[..4], b"OggS");
    assert_eq!(
        bytes[5] & 0x02,
        0x02,
        "the first page is marked beginning of stream"
    );
    assert_eq!(&bytes[28..36], b"OpusHead");
    let mut packets = PacketReader::new(Cursor::new(bytes));
    let head = packets.read_packet().unwrap().unwrap();
    assert!(head.first_in_stream() && head.last_in_page());
    assert_eq!(head.absgp_page(), 0);
    let head = parse_head(&head.data).unwrap();
    assert_eq!(head.channels, 2);
    assert_eq!(head.input_rate, SAMPLE_RATE);
    assert_eq!(head.gain, 0);
    assert_eq!(head.pre_skip, 312, "libopus's lookahead at 48 kHz");
    let tags = packets.read_packet().unwrap().unwrap();
    assert_eq!(tags.absgp_page(), 0);
    let (vendor, comments) = parse_tags(&tags.data).unwrap();
    assert!(vendor.contains("libopus"));
    assert_eq!(comments, ["ENCODER=kanso-audio"]);
}

#[test]
fn a_recording_round_trips_to_its_exact_length() {
    for channels in [Channels::Mono, Channels::Stereo] {
        let n = channels.count();
        // Not a whole number of frames: the tail is trimmed back off.
        let samples = SAMPLE_RATE as usize * 3 + 123;
        let pcm = signal(samples, n, 7);
        let mut config = EncodeConfig::new(channels);
        config.bitrate = 64_000;
        let (memory, parts) = record(&pcm, config, 4_001);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].samples, samples as u64);
        assert_eq!(parts[0].bytes, memory.bytes(0).len() as u64);
        let (reader, out) = decode(memory.bytes(0));
        assert_eq!(reader.info().total_samples, samples as u64);
        assert_eq!(reader.info().channels, channels);
        assert_eq!(out.len(), pcm.len());
        let s = similarity(&pcm, &out);
        assert!(s > 0.95, "{channels:?} came back with similarity {s}");
    }
}

#[test]
fn a_page_ends_every_second_and_the_file_syncs_every_two() {
    let memory = Memory::default();
    let mut writer = PartWriter::new(memory.sink(), EncodeConfig::new(Channels::Mono)).unwrap();
    let second = signal(SAMPLE_RATE as usize, 1, 3);
    writer.push(&second).unwrap();
    // Headers synced; one whole second written, its page out.
    let after_one = Index::of(&memory.bytes(0));
    assert_eq!(after_one.entries.len(), 1, "one audio page after a second");
    assert_eq!(after_one.end_granule, 48_000);
    assert_eq!(memory.syncs(0), 1);
    writer.push(&second).unwrap();
    assert_eq!(memory.syncs(0), 2, "synced at two seconds");
    writer.push(&second[..FRAME_SAMPLES * 10]).unwrap();
    assert_eq!(Index::of(&memory.bytes(0)).entries.len(), 2);
    assert_eq!(writer.open_part().unwrap().index, 0);
    drop(writer);
    // Killed here: the file is playable up to the last page.
    let (reader, out) = decode(memory.bytes(0));
    // The encoder's last 312 samples were still inside it when the
    // process died.
    assert_eq!(reader.info().total_samples, 96_000 - 312);
    assert_eq!(out.len(), 96_000 - 312);
}

struct Index;
impl Index {
    fn of(bytes: &[u8]) -> PageIndex {
        PageIndex::scan(&mut Cursor::new(bytes.to_vec())).unwrap()
    }
}

#[test]
fn parts_split_at_the_byte_limit_and_each_plays_alone() {
    let samples = SAMPLE_RATE as usize * 40;
    let pcm = signal(samples, 2, 11);
    let mut config = EncodeConfig::new(Channels::Stereo);
    config.max_part_bytes = 20 * 1024;
    let (memory, parts) = record(&pcm, config.clone(), 9_999);
    assert!(parts.len() >= 3, "got {} parts", parts.len());
    let mut total = 0;
    let mut joined = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        assert_eq!(part.index as usize, i);
        assert!(part.bytes <= config.max_part_bytes, "{part:?}");
        assert_eq!(part.bytes, memory.bytes(i).len() as u64);
        let (reader, out) = decode(memory.bytes(i));
        assert_eq!(reader.info().total_samples, part.samples);
        assert_eq!(out.len() as u64, part.samples * 2);
        total += part.samples;
        joined.extend(out);
    }
    assert_eq!(
        total, samples as u64,
        "no audio lost or repeated across parts"
    );
    let s = similarity(&pcm, &joined);
    assert!(s > 0.95, "joined parts have similarity {s}");
}

#[test]
fn parts_split_at_the_time_limit() {
    let mut config = EncodeConfig::new(Channels::Mono);
    config.max_part_frames = 150; // three seconds standing in for sixty minutes
    let pcm = signal(SAMPLE_RATE as usize * 10, 1, 5);
    let (_, parts) = record(&pcm, config, 960);
    assert_eq!(
        parts.iter().map(|p| p.samples).collect::<Vec<_>>(),
        [144_000, 144_000, 144_000, 48_000]
    );
    // The real limits.
    assert_eq!(
        kanso_audio::MAX_PART_FRAMES * FRAME_SAMPLES as u64,
        60 * 60 * 48_000
    );
    assert_eq!(kanso_audio::MAX_PART_BYTES, 9 * 1024 * 1024);
}

#[test]
fn an_hour_at_speech_rates_fits_in_a_part() {
    // 16 kbps mono is 7.2 MB an hour: under 9 MiB, so the time limit binds
    // first; 24 kbps stereo is 10.8 MB an hour, so the size limit binds.
    let mono_hour = 16_000u64 / 8 * 3_600;
    let stereo_hour = 24_000u64 / 8 * 3_600;
    assert!(mono_hour < kanso_audio::MAX_PART_BYTES);
    assert!(stereo_hour > kanso_audio::MAX_PART_BYTES);
}

#[test]
fn a_crash_leaves_a_playable_file() {
    let pcm = signal(SAMPLE_RATE as usize * 6, 1, 9);
    let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Mono), 1_000);
    let bytes = memory.bytes(0);
    let (_, full) = decode(bytes.clone());
    let index = Index::of(&bytes);
    let last = index.entries.last().unwrap().offset as usize;
    // Cut anywhere inside the last page, and anywhere after the headers.
    for cut in [last + 1, last + 30, bytes.len() - 1] {
        let (reader, out) = decode(bytes[..cut].to_vec());
        assert_eq!(reader.info().total_samples, 6 * 48_000 - 312);
        assert_eq!(out.len(), 6 * 48_000 - 312);
        assert_eq!(&out[..], &full[..out.len()]);
    }
    // Cut before any audio page: opens, plays nothing.
    let first = index.entries[0].offset as usize;
    let (_, none) = decode(bytes[..first + 10].to_vec());
    assert!(none.is_empty());
}

#[test]
fn the_page_table_lists_every_audio_page() {
    let pcm = signal(SAMPLE_RATE as usize * 5, 1, 2);
    let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Mono), 1_920);
    let bytes = memory.bytes(0);
    let index = Index::of(&bytes);
    // Five whole seconds, then the page that ends the stream.
    assert_eq!(index.entries.len(), 6);
    assert_eq!(index.entries[0].granule, 0);
    for (i, entry) in index.entries.iter().enumerate() {
        assert_eq!(&bytes[entry.offset as usize..][..4], b"OggS");
        if i > 0 {
            assert!(entry.offset > index.entries[i - 1].offset);
            assert_eq!(entry.granule, 48_000 * i as u64);
        }
    }
    assert_eq!(
        index.end_granule,
        312 + 5 * 48_000,
        "the last page trims to the real length"
    );
}

#[test]
fn opus_is_told_from_everything_else_by_its_magic() {
    let pcm = signal(SAMPLE_RATE as usize, 1, 4);
    let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Mono), 960);
    // Whatever the extension says.
    for extension in [None, Some("opus"), Some("ogg"), Some("wav")] {
        let mut source = open_bytes(memory.bytes(0), extension).unwrap();
        assert_eq!(source.sample_rate(), 48_000);
        assert_eq!(source.total_frames(), Some(48_000));
        let (mut all, mut buf) = (Vec::new(), Vec::new());
        while let Some(ch) = source.read(&mut buf).unwrap() {
            assert_eq!(ch, 1);
            all.extend_from_slice(&buf);
        }
        assert_eq!(all.len(), 48_000);
    }
    assert!(open_bytes(b"not audio at all, not at all, not at all".to_vec(), None).is_err());
}

/// A 16-bit PCM WAV of a ramp, one value per frame, so a position can be
/// read off a sample.
fn ramp_wav(rate: u32, channels: u16, frames: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let data = frames * usize::from(channels) * 2;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data as u32).to_le_bytes());
    for i in 0..frames {
        for _ in 0..channels {
            out.extend_from_slice(&((i % 30_000) as i16).to_le_bytes());
        }
    }
    out
}

#[test]
fn wav_goes_through_symphonia_and_seeks() {
    let mut source = open_bytes(ramp_wav(44_100, 2, 20_000), Some("wav")).unwrap();
    assert_eq!(source.sample_rate(), 44_100);
    assert_eq!(source.total_frames(), Some(20_000));
    let mut buf = Vec::new();
    assert_eq!(source.read(&mut buf).unwrap(), Some(2));
    source.seek(12_345).unwrap();
    let mut all = Vec::new();
    while source.read(&mut buf).unwrap().is_some() {
        all.extend_from_slice(&buf);
    }
    assert_eq!(all.len(), (20_000 - 12_345) * 2);
    let first = (all[0] * 32_768.0).round() as i32;
    assert_eq!(first, 12_345);
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
        ..ProptestConfig::with_cases(24)
    })]

    /// Whatever the length, channel count, chunking and part limit: no
    /// audio is lost or repeated, every part is under the limit, and the
    /// decoded length is exactly the recorded length.
    #[test]
    fn prop_round_trip_keeps_every_sample(
        samples in 1usize..200_000,
        stereo in any::<bool>(),
        chunk in 1usize..30_000,
        limit_kib in 16u64..64,
        seed in any::<u32>(),
    ) {
        let channels = if stereo { Channels::Stereo } else { Channels::Mono };
        let pcm = signal(samples, channels.count(), seed);
        let mut config = EncodeConfig::new(channels);
        config.max_part_bytes = limit_kib * 1024;
        let (memory, parts) = record(&pcm, config.clone(), chunk * channels.count());
        prop_assert_eq!(parts.iter().map(|p| p.samples).sum::<u64>(), samples as u64);
        let mut joined = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            prop_assert!(part.bytes <= config.max_part_bytes);
            let (reader, out) = decode(memory.bytes(i));
            prop_assert_eq!(reader.info().total_samples, part.samples);
            joined.extend(out);
        }
        prop_assert_eq!(joined.len(), pcm.len());
    }

    /// Seeking to any sample then reading to the end gives exactly the rest
    /// of the file, and the same audio as decoding straight through.
    #[test]
    fn prop_seek_lands_on_the_target(
        samples in 5_000usize..300_000,
        target_per_mille in 0u64..=1_000,
        seed in any::<u32>(),
    ) {
        let pcm = signal(samples, 1, seed);
        let mut config = EncodeConfig::new(Channels::Mono);
        config.bitrate = 64_000;
        let (memory, _) = record(&pcm, config, 5_000);
        let bytes = memory.bytes(0);
        let (_, full) = decode(bytes.clone());
        let mut reader = OpusReader::open(Cursor::new(bytes)).unwrap();
        let target = (samples as u64 * target_per_mille / 1_000) as usize;
        reader.seek(target as u64).unwrap();
        let tail = reader.read_to_end().unwrap();
        prop_assert_eq!(tail.len(), full.len() - target);
        // 80 ms of run-up is enough for the decoder to match a straight run.
        let want = &full[target..];
        let settled = (SAMPLE_RATE as usize / 5).min(tail.len());
        if tail.len() > settled {
            let s = similarity(&want[settled..], &tail[settled..]);
            prop_assert!(s > 0.999, "similarity {s} at {target}");
        }
        // And the very first samples are close too.
        let head = 480.min(tail.len());
        prop_assert!(similarity(&want[..head], &tail[..head]) > 0.9);
    }

    /// Seeking back and forth in one reader always gives the right length.
    #[test]
    fn prop_repeated_seeks_stay_exact(
        targets in proptest::collection::vec(0u64..=100, 1..8),
    ) {
        let samples = 200_000usize;
        let pcm = signal(samples, 2, 77);
        let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Stereo), 8_000);
        let mut reader = OpusReader::open(Cursor::new(memory.bytes(0))).unwrap();
        for t in targets {
            let target = samples as u64 * t / 100;
            reader.seek(target).unwrap();
            let tail = reader.read_to_end().unwrap();
            prop_assert_eq!(tail.len() as u64, (samples as u64 - target) * 2);
        }
    }
}

/// A last page that is all there but torn inside (the length reached the
/// disk, not every byte) fails its checksum: it is left out of the length
/// as well as the audio, so the duration matches what plays.
#[test]
fn a_torn_last_page_is_left_out_of_the_length() {
    let pcm = signal(SAMPLE_RATE as usize * 6, 1, 9);
    let (memory, _) = record(&pcm, EncodeConfig::new(Channels::Mono), 1_000);
    let mut bytes = memory.bytes(0);
    let last = Index::of(&bytes).entries.last().unwrap().offset as usize;
    let n = bytes.len();
    bytes[n - 3] ^= 0x55;
    let mut reader = OpusReader::open(Cursor::new(bytes)).unwrap();
    let out = reader.read_to_end().unwrap();
    assert_eq!(out.len() as u64, reader.info().total_samples);
    assert!(
        reader
            .index()
            .entries
            .iter()
            .all(|e| (e.offset as usize) < last)
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
        ..ProptestConfig::with_cases(256)
    })]

    /// Flipped, cut or overwritten bytes anywhere in a recording never
    /// panic: opening, reading, seeking and Symphonia's fallback all return.
    #[test]
    fn prop_damaged_recordings_never_panic(
        flips in proptest::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 0..12),
        cut in any::<prop::sample::Index>(),
        seek in 0u64..200_000,
        stereo in any::<bool>(),
    ) {
        let channels = if stereo { Channels::Stereo } else { Channels::Mono };
        let pcm = signal(SAMPLE_RATE as usize * 3, channels.count(), 3);
        let (memory, _) = record(&pcm, EncodeConfig::new(channels), 4_000);
        let mut bytes = memory.bytes(0);
        for (at, b) in &flips {
            let i = at.index(bytes.len());
            bytes[i] ^= b | 1;
        }
        bytes.truncate(cut.index(bytes.len() + 1));
        if let Ok(mut reader) = OpusReader::open(Cursor::new(bytes.clone())) {
            let total = reader.info().total_samples;
            let out = reader.read_to_end();
            if let Ok(out) = out {
                prop_assert!(out.len() as u64 <= total * channels.count() as u64);
            }
            let _ = reader.seek(seek);
            let _ = reader.read_to_end();
        }
        if let Ok(mut source) = open_bytes(bytes, None) {
            let mut buf = Vec::new();
            for _ in 0..10_000 {
                match source.read(&mut buf) {
                    Ok(Some(_)) => {}
                    _ => break,
                }
            }
            let _ = source.seek(seek);
        }
    }

    /// Pages forged with the largest sizes and granules Ogg can declare
    /// never make the scanner allocate or read past the bytes there.
    #[test]
    fn prop_forged_pages_are_bounded(
        pages in proptest::collection::vec((any::<u64>(), any::<u8>(), any::<u8>()), 1..20),
    ) {
        let mut bytes = Vec::new();
        for (granule, flags, segments) in pages {
            bytes.extend_from_slice(b"OggS\0");
            bytes.push(flags);
            bytes.extend_from_slice(&granule.to_le_bytes());
            bytes.extend_from_slice(&[0; 12]);
            bytes.push(segments);
            bytes.extend(std::iter::repeat_n(255u8, usize::from(segments)));
            bytes.extend_from_slice(&[0; 100]);
        }
        let index = PageIndex::scan(&mut Cursor::new(bytes.clone())).unwrap();
        prop_assert!(index.entries.iter().all(|e| (e.offset as usize) < bytes.len()));
        let _ = OpusReader::open(Cursor::new(bytes));
    }
}
