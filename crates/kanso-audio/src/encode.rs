//! Opus in Ogg, written as the audio arrives.
//!
//! [`PartWriter`] takes interleaved 48 kHz samples, cuts them into 20 ms
//! frames, encodes each with libopus and writes it to an Ogg file. A page
//! ends every second and the file is synced every two, so a killed process
//! leaves a playable file that is missing seconds, not the recording. When a
//! part would pass 9 MiB or 60 minutes it is closed with a proper end of
//! stream and the next one opens, its encoder starting afresh, so every part
//! plays on its own and the parts joined in order are the whole recording.

use std::fs::File;
use std::io::{self, Write};

use ogg::{PacketWriteEndInfo, PacketWriter};

use crate::error::AudioError;
use crate::header::{VENDOR, build_head, build_tags};
use crate::model::{
    Channels, EncodeConfig, FRAME_SAMPLES, OpenPart, OpusHead, PAGE_FRAMES, PartInfo, SAMPLE_RATE,
    SYNC_FRAMES,
};

/// The most bytes libopus gives one packet.
const MAX_PACKET: usize = 1275;

/// Somewhere a part is written, that can be forced to disk.
pub trait DurableWrite: Write {
    /// Makes what was written survive a power cut (`fsync` for a file).
    fn sync(&mut self) -> io::Result<()>;
}

impl DurableWrite for File {
    fn sync(&mut self) -> io::Result<()> {
        self.sync_data()
    }
}

impl DurableWrite for Vec<u8> {
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Where the parts of a recording go: opens part `index`.
pub trait PartSink {
    type Out: DurableWrite;
    fn open(&mut self, index: u32) -> io::Result<Self::Out>;
}

impl<F, W> PartSink for F
where
    F: FnMut(u32) -> io::Result<W>,
    W: DurableWrite,
{
    type Out = W;
    fn open(&mut self, index: u32) -> io::Result<W> {
        self(index)
    }
}

/// Counts what passes through to the file.
struct Counting<W> {
    inner: W,
    written: u64,
}

impl<W: Write> Write for Counting<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// An upper bound on the bytes Ogg adds for `segments` lacing values: the
/// table itself and a 27-byte header for every 255 of them.
pub fn page_overhead(segments: u64) -> u64 {
    segments + 27 * (segments / 255 + 1)
}

/// The lacing values a packet of `len` bytes takes.
fn segments_of(len: usize) -> u64 {
    (len / 255 + 1) as u64
}

struct Part<W: DurableWrite> {
    writer: PacketWriter<'static, Counting<W>>,
    serial: u32,
    index: u32,
    /// Frames encoded into this part.
    frames: u64,
    /// Samples per channel of real audio in it.
    samples: u64,
    /// Payload bytes and lacing values written since the last page ended.
    pending_payload: u64,
    pending_segments: u64,
}

impl<W: DurableWrite> Part<W> {
    fn file(&mut self) -> &mut Counting<W> {
        self.writer.inner_mut()
    }

    fn written(&self) -> u64 {
        self.writer.inner().written
    }

    /// Bytes the part would be if it ended now, as an upper bound.
    fn worst_size(&self, extra_payload: u64, extra_segments: u64) -> u64 {
        self.written()
            + self.pending_payload
            + extra_payload
            + page_overhead(self.pending_segments + extra_segments)
    }
}

/// Writes a recording as Ogg Opus parts.
pub struct PartWriter<S: PartSink> {
    sink: S,
    config: EncodeConfig,
    encoder: opus::Encoder,
    pre_skip: u16,
    /// Samples taken in but not yet a whole frame.
    frame: Vec<f32>,
    current: Option<Part<S::Out>>,
    done: Vec<PartInfo>,
    next_index: u32,
}

impl<S: PartSink> PartWriter<S> {
    /// A writer for a recording with `config`, its parts opened from `sink`
    /// as they are needed. Nothing is created until audio arrives.
    pub fn new(sink: S, config: EncodeConfig) -> Result<Self, AudioError> {
        config.validate()?;
        let channels = match config.channels {
            Channels::Mono => opus::Channels::Mono,
            Channels::Stereo => opus::Channels::Stereo,
        };
        let mut encoder = opus::Encoder::new(SAMPLE_RATE, channels, opus::Application::Voip)?;
        encoder.set_bitrate(opus::Bitrate::Bits(config.bitrate))?;
        let pre_skip = u16::try_from(encoder.get_lookahead()?).unwrap_or(312);
        Ok(PartWriter {
            sink,
            frame: Vec::with_capacity(FRAME_SAMPLES * config.channels.count()),
            config,
            encoder,
            pre_skip,
            current: None,
            done: Vec::new(),
            next_index: 0,
        })
    }

    /// Takes interleaved samples (`-1.0..=1.0`), writing every whole frame
    /// they complete.
    pub fn push(&mut self, pcm: &[f32]) -> Result<(), AudioError> {
        let frame_len = FRAME_SAMPLES * self.config.channels.count();
        let mut rest = pcm;
        while !rest.is_empty() {
            let take = (frame_len - self.frame.len()).min(rest.len());
            self.frame.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if self.frame.len() == frame_len {
                let frame = std::mem::take(&mut self.frame);
                self.write_frame(&frame, FRAME_SAMPLES)?;
                self.frame = frame;
                self.frame.clear();
            }
        }
        Ok(())
    }

    /// Forces what has been written to disk now (for a pause or a
    /// checkpoint), without ending the page in progress.
    pub fn sync(&mut self) -> Result<(), AudioError> {
        if let Some(part) = &mut self.current {
            part.file().inner.sync()?;
        }
        Ok(())
    }

    /// The parts finished so far.
    pub fn finished(&self) -> &[PartInfo] {
        &self.done
    }

    /// The part being written, if any.
    pub fn open_part(&self) -> Option<OpenPart> {
        self.current.as_ref().map(|p| OpenPart {
            index: p.index,
            bytes: p.written(),
            samples: p.samples,
        })
    }

    /// Writes the last partial frame (padded with silence, and trimmed off
    /// again by the end granule), ends the stream and returns every part.
    pub fn finish(mut self) -> Result<Vec<PartInfo>, AudioError> {
        let channels = self.config.channels.count();
        if !self.frame.is_empty() {
            let real = self.frame.len() / channels;
            let mut frame = std::mem::take(&mut self.frame);
            frame.resize(FRAME_SAMPLES * channels, 0.0);
            self.write_frame(&frame, real)?;
        }
        self.close_part()?;
        Ok(self.done)
    }

    fn write_frame(&mut self, frame: &[f32], real: usize) -> Result<(), AudioError> {
        // A partial frame is the last: it never starts a new part.
        let whole = real == FRAME_SAMPLES;
        let split = match &self.current {
            Some(p) => {
                p.frames >= self.config.max_part_frames
                    // Room for this packet and the one that flushes the
                    // encoder when the part ends, each at its largest.
                    || (whole
                        && p.worst_size(
                            2 * MAX_PACKET as u64,
                            2 * segments_of(MAX_PACKET),
                        ) > self.config.max_part_bytes)
            }
            None => false,
        };
        if split {
            self.close_part()?;
        }
        if self.current.is_none() {
            self.open_part_now()?;
        }

        let mut packet = vec![0u8; MAX_PACKET];
        let len = self.encoder.encode_float(frame, &mut packet)?;
        packet.truncate(len);

        let part = self.current.as_mut().expect("a part is open");
        part.frames += 1;
        part.samples += real as u64;
        part.pending_payload += len as u64;
        part.pending_segments += segments_of(len);
        // Samples decoded so far, pre-skip among them (RFC 7845 section 4).
        let granule = part.frames * FRAME_SAMPLES as u64;
        let ends_page = whole && part.frames.is_multiple_of(PAGE_FRAMES);
        let info = if ends_page {
            PacketWriteEndInfo::EndPage
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        part.writer
            .write_packet(packet, part.serial, info, granule)?;
        if ends_page {
            part.pending_payload = 0;
            part.pending_segments = 0;
            part.file().flush()?;
            if part.frames.is_multiple_of(SYNC_FRAMES) {
                part.file().inner.sync()?;
            }
        }
        Ok(())
    }

    fn open_part_now(&mut self) -> Result<(), AudioError> {
        let index = self.next_index;
        let out = self.sink.open(index)?;
        let serial = self.config.serial.wrapping_add(index);
        let mut writer = PacketWriter::new(Counting {
            inner: out,
            written: 0,
        });
        let head = build_head(&OpusHead {
            channels: self.config.channels.count() as u8,
            pre_skip: self.pre_skip,
            input_rate: SAMPLE_RATE,
            gain: 0,
        });
        writer.write_packet(head, serial, PacketWriteEndInfo::EndPage, 0)?;
        let tags = build_tags(VENDOR, &[("ENCODER", "kanso-audio")]);
        writer.write_packet(tags, serial, PacketWriteEndInfo::EndPage, 0)?;
        writer.inner_mut().flush()?;
        writer.inner_mut().inner.sync()?;
        self.next_index += 1;
        self.current = Some(Part {
            writer,
            serial,
            index,
            frames: 0,
            samples: 0,
            pending_payload: 0,
            pending_segments: 0,
        });
        Ok(())
    }

    /// Ends the open part: one silent frame pushes the encoder's delay out
    /// as audio, the end granule trims it back to the real length, and the
    /// encoder starts clean for the next part.
    fn close_part(&mut self) -> Result<(), AudioError> {
        let Some(mut part) = self.current.take() else {
            return Ok(());
        };
        let silence = vec![0.0f32; FRAME_SAMPLES * self.config.channels.count()];
        let mut packet = vec![0u8; MAX_PACKET];
        let len = self.encoder.encode_float(&silence, &mut packet)?;
        packet.truncate(len);
        let granule = u64::from(self.pre_skip) + part.samples;
        part.writer
            .write_packet(packet, part.serial, PacketWriteEndInfo::EndStream, granule)?;
        part.file().flush()?;
        part.file().inner.sync()?;
        self.done.push(PartInfo {
            index: part.index,
            bytes: part.written(),
            samples: part.samples,
        });
        self.encoder.reset_state()?;
        Ok(())
    }
}
