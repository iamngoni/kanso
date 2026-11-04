//! Ogg Opus back to samples, with seeking by the page index.

use std::io::{Read, Seek, SeekFrom};

use ogg::PacketReader;

use crate::error::AudioError;
use crate::header::{parse_head, parse_tags};
use crate::index::PageIndex;
use crate::model::{Channels, SAMPLE_RATE, SEEK_PREROLL, StreamInfo};

/// The most samples per channel one Opus packet can hold: 120 ms.
const MAX_PACKET_SAMPLES: usize = 5_760;

/// Reads an Ogg Opus file a packet at a time.
pub struct OpusReader<R: Read + Seek> {
    packets: PacketReader<R>,
    decoder: opus::Decoder,
    index: PageIndex,
    info: StreamInfo,
    channels: usize,
    /// Granule position of the start of the next packet.
    position: u64,
    /// Samples before this granule are decoded and dropped (pre-skip, or
    /// the run-up to a seek target).
    emit_from: u64,
    scratch: Vec<f32>,
}

fn is_eof(e: &ogg::OggReadError) -> bool {
    matches!(e, ogg::OggReadError::ReadError(io) if io.kind() == std::io::ErrorKind::UnexpectedEof)
}

impl<R: Read + Seek> OpusReader<R> {
    /// Reads the headers and scans the pages. Fails when the file isn't Ogg
    /// Opus.
    pub fn open(mut reader: R) -> Result<Self, AudioError> {
        let index = PageIndex::scan(&mut reader)?;
        reader.seek(SeekFrom::Start(0))?;
        let mut packets = PacketReader::new(reader);
        let head = packets
            .read_packet()?
            .ok_or_else(|| AudioError::Format("it's empty".into()))?;
        let head = parse_head(&head.data)?;
        let tags = packets
            .read_packet()?
            .ok_or_else(|| AudioError::Format("the OpusTags header is missing".into()))?;
        parse_tags(&tags.data)?;
        let channels = Channels::from_count(usize::from(head.channels))
            .ok_or_else(|| AudioError::Format("only mono and stereo Opus are supported".into()))?;
        let decoder = opus::Decoder::new(
            SAMPLE_RATE,
            match channels {
                Channels::Mono => opus::Channels::Mono,
                Channels::Stereo => opus::Channels::Stereo,
            },
        )?;
        let pre_skip = u64::from(head.pre_skip);
        Ok(OpusReader {
            packets,
            decoder,
            info: StreamInfo {
                channels,
                pre_skip: head.pre_skip,
                total_samples: index.end_granule.saturating_sub(pre_skip),
            },
            index,
            channels: channels.count(),
            position: 0,
            emit_from: pre_skip,
            scratch: vec![0.0; MAX_PACKET_SAMPLES * channels.count()],
        })
    }

    pub fn info(&self) -> StreamInfo {
        self.info
    }

    pub fn index(&self) -> &PageIndex {
        &self.index
    }

    /// Samples per channel already handed out or skipped over, after
    /// pre-skip: where the next [`read`](Self::read) starts.
    pub fn position(&self) -> u64 {
        self.position.saturating_sub(u64::from(self.info.pre_skip))
    }

    /// Decodes the next packet and appends its interleaved samples to `out`,
    /// leaving out pre-skip, the run-up to a seek and anything past the end
    /// granule. Returns the samples per channel appended (possibly 0), or
    /// `None` at the end. A damaged packet is skipped; a file cut short ends
    /// at the last whole page.
    pub fn read(&mut self, out: &mut Vec<f32>) -> Result<Option<usize>, AudioError> {
        let end = self.index.end_granule;
        loop {
            if self.position >= end {
                return Ok(None);
            }
            let packet = match self.packets.read_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(None),
                Err(e) if is_eof(&e) => return Ok(None),
                // A page that fails its checksum is the torn end of a crash.
                Err(ogg::OggReadError::HashMismatch(..)) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            let decoded = match self
                .decoder
                .decode_float(&packet.data, &mut self.scratch, false)
            {
                Ok(n) => n as u64,
                Err(_) => continue,
            };
            let start = self.position;
            self.position += decoded;
            let from = self.emit_from.max(start);
            let to = self.position.min(end);
            if to <= from {
                continue;
            }
            let lo = (from - start) as usize * self.channels;
            let hi = (to - start) as usize * self.channels;
            out.extend_from_slice(&self.scratch[lo..hi]);
            return Ok(Some((to - from) as usize));
        }
    }

    /// Moves to `sample` (per channel, after pre-skip). The next read
    /// starts there: decoding begins at the page before the target, 80 ms
    /// early, and the run-up is dropped. A target past the end seeks to the
    /// end.
    pub fn seek(&mut self, sample: u64) -> Result<(), AudioError> {
        let sample = sample.min(self.info.total_samples);
        let target = sample + u64::from(self.info.pre_skip);
        let Some(page) = self.index.locate(target.saturating_sub(SEEK_PREROLL)) else {
            self.position = self.index.end_granule;
            return Ok(());
        };
        self.packets.seek_bytes(SeekFrom::Start(page.offset))?;
        self.decoder.reset_state()?;
        self.position = page.granule;
        self.emit_from = target;
        Ok(())
    }

    /// Decodes everything left into one buffer.
    pub fn read_to_end(&mut self) -> Result<Vec<f32>, AudioError> {
        let mut out = Vec::new();
        while self.read(&mut out)?.is_some() {}
        Ok(out)
    }
}
