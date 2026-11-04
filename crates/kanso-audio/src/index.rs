//! The page table of an Ogg file: where each page starts and where the audio
//! stands there, so a seek goes to a page by binary search instead of
//! decoding from the start.
//!
//! Scanning reads each page's 27-byte header and segment table and skips its
//! body, so a 10 MB part takes milliseconds. A last page cut short by a crash
//! is left out, as every Ogg reader leaves it out, and so is a page that is
//! all there but fails its checksum (the length reached the disk before
//! every byte did): the decoder stops at such a page, so the index must too,
//! or the duration would count audio that never plays.

use std::io::{self, Read, Seek, SeekFrom};

use crate::model::PageEntry;

const CAPTURE: &[u8; 4] = b"OggS";
const HEADER_LEN: u64 = 27;
const CONTINUED: u8 = 0x01;
/// The largest body a page can declare: 255 lacing values of 255.
const MAX_BODY: usize = 255 * 255;

/// The Ogg page checksum: CRC-32, polynomial 0x04c11db7, not reflected,
/// starting at 0, over the page with its checksum field zeroed.
fn crc_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for &b in bytes {
        crc ^= u32::from(b) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04c1_1db7
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// One page header, as far as the index needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageHeader {
    pub offset: u64,
    pub continued: bool,
    /// `None` when no packet ends on the page.
    pub granule: Option<u64>,
    /// Bytes of header, segment table and body.
    pub len: u64,
}

/// Reads the page header at the reader's position and moves past the page.
/// `None` at the end of the file, at a page that isn't there whole, at a
/// page that fails its checksum, or at bytes that aren't a page.
pub fn read_page_header<R: Read + Seek>(
    reader: &mut R,
    offset: u64,
    file_len: u64,
) -> io::Result<Option<PageHeader>> {
    if offset + HEADER_LEN > file_len {
        return Ok(None);
    }
    let mut header = [0u8; HEADER_LEN as usize];
    reader.read_exact(&mut header)?;
    if &header[..4] != CAPTURE || header[4] != 0 {
        return Ok(None);
    }
    let segments = usize::from(header[26]);
    let mut table = [0u8; 255];
    reader.read_exact(&mut table[..segments])?;
    let body: u64 = table[..segments].iter().map(|b| u64::from(*b)).sum();
    let len = HEADER_LEN + segments as u64 + body;
    if offset + len > file_len {
        return Ok(None);
    }
    let mut data = vec![0u8; body as usize];
    debug_assert!(data.len() <= MAX_BODY);
    reader.read_exact(&mut data)?;
    let stored = u32::from_le_bytes(header[22..26].try_into().unwrap());
    header[22..26].fill(0);
    let crc = crc_update(
        crc_update(crc_update(0, &header), &table[..segments]),
        &data,
    );
    if crc != stored {
        return Ok(None);
    }
    let granule = i64::from_le_bytes(header[6..14].try_into().unwrap());
    Ok(Some(PageHeader {
        offset,
        continued: header[5] & CONTINUED != 0,
        granule: u64::try_from(granule).ok(),
        len,
    }))
}

/// The pages of one Ogg Opus stream that audio can be started from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageIndex {
    /// Pages after the two headers that start on a packet boundary, in file
    /// order; both fields only grow.
    pub entries: Vec<PageEntry>,
    /// The granule at the end of the last whole page.
    pub end_granule: u64,
    /// The byte offset just past the last whole page: where a file cut short
    /// by a crash can be truncated to be clean.
    pub end_offset: u64,
}

impl PageIndex {
    /// Scans `reader` from the start.
    pub fn scan<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let file_len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;
        let mut index = PageIndex::default();
        let mut offset = 0u64;
        let mut in_headers = true;
        let mut granule = 0u64;
        while let Some(page) = read_page_header(reader, offset, file_len)? {
            offset += page.len;
            // The identification and comment headers carry granule 0 (or none
            // when the tags run over a page); audio starts at the first page
            // that has one.
            if in_headers {
                if page.granule.is_none_or(|g| g == 0) {
                    continue;
                }
                in_headers = false;
            }
            if !page.continued {
                index.entries.push(PageEntry {
                    offset: page.offset,
                    granule,
                });
            }
            if let Some(end) = page.granule {
                granule = end;
            }
        }
        index.end_granule = granule;
        index.end_offset = offset;
        Ok(index)
    }

    /// The last page that starts at or before `granule`, or the first page
    /// when `granule` is earlier than all of them. `None` when the file has
    /// no audio pages.
    pub fn locate(&self, granule: u64) -> Option<PageEntry> {
        let after = self.entries.partition_point(|e| e.granule <= granule);
        self.entries
            .get(after.saturating_sub(1))
            .or(self.entries.first())
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    fn entry(offset: u64, granule: u64) -> PageEntry {
        PageEntry { offset, granule }
    }

    #[test]
    fn locate_picks_the_page_at_or_before() {
        let index = PageIndex {
            entries: vec![entry(100, 0), entry(200, 50), entry(300, 100)],
            end_granule: 150,
            ..PageIndex::default()
        };
        assert_eq!(index.locate(0), Some(entry(100, 0)));
        assert_eq!(index.locate(49), Some(entry(100, 0)));
        assert_eq!(index.locate(50), Some(entry(200, 50)));
        assert_eq!(index.locate(10_000), Some(entry(300, 100)));
        assert_eq!(PageIndex::default().locate(5), None);
    }

    #[test]
    fn scanning_garbage_finds_nothing() {
        let index = PageIndex::scan(&mut Cursor::new(vec![7u8; 500])).unwrap();
        assert!(index.entries.is_empty());
        let index = PageIndex::scan(&mut Cursor::new(Vec::new())).unwrap();
        assert_eq!(index.end_granule, 0);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// `locate` returns the greatest entry not past the target, and
        /// scanning never panics on arbitrary bytes.
        #[test]
        fn prop_locate_is_the_floor(
            steps in proptest::collection::vec(1u64..5_000, 1..60),
            target in 0u64..400_000,
            junk in proptest::collection::vec(any::<u8>(), 0..300),
        ) {
            let mut granule = 0;
            let mut entries = Vec::new();
            for (i, step) in steps.iter().enumerate() {
                entries.push(entry(i as u64 * 100, granule));
                granule += step;
            }
            let index = PageIndex { entries: entries.clone(), end_granule: granule, ..PageIndex::default() };
            let found = index.locate(target).unwrap();
            if target >= entries[0].granule {
                prop_assert!(found.granule <= target);
                let next = entries.iter().find(|e| e.granule > found.granule);
                prop_assert!(next.is_none_or(|n| n.granule > target));
            } else {
                prop_assert_eq!(found, entries[0]);
            }
            let _ = PageIndex::scan(&mut Cursor::new(junk));
        }
    }
}
