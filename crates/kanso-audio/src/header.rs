//! The two header packets of an Ogg Opus stream (RFC 7845 section 5). Pure.

use crate::error::AudioError;
use crate::model::OpusHead;

const HEAD_MAGIC: &[u8; 8] = b"OpusHead";
const TAGS_MAGIC: &[u8; 8] = b"OpusTags";
/// The vendor string written into `OpusTags`.
pub const VENDOR: &str = "kanso-audio (libopus)";

/// Whether `bytes` begin an Ogg Opus stream: an Ogg page whose first packet
/// is `OpusHead`. Looks at the first 64 bytes only.
pub fn sniff(bytes: &[u8]) -> bool {
    bytes.starts_with(b"OggS")
        && bytes[..bytes.len().min(64)]
            .windows(HEAD_MAGIC.len())
            .any(|w| w == HEAD_MAGIC)
}

/// The 19-byte `OpusHead` packet.
pub fn build_head(head: &OpusHead) -> Vec<u8> {
    let mut out = Vec::with_capacity(19);
    out.extend_from_slice(HEAD_MAGIC);
    out.push(1);
    out.push(head.channels);
    out.extend_from_slice(&head.pre_skip.to_le_bytes());
    out.extend_from_slice(&head.input_rate.to_le_bytes());
    out.extend_from_slice(&head.gain.to_le_bytes());
    out.push(0); // channel mapping family 0: mono or stereo
    out
}

/// Reads an `OpusHead` packet. Refuses other versions, and channel mappings
/// beyond family 0.
pub fn parse_head(packet: &[u8]) -> Result<OpusHead, AudioError> {
    if packet.len() < 19 || &packet[..8] != HEAD_MAGIC {
        return Err(AudioError::Format("the OpusHead header is missing".into()));
    }
    if packet[8] >> 4 != 0 {
        return Err(AudioError::Format("it's a newer Opus version".into()));
    }
    let channels = packet[9];
    if !(1..=2).contains(&channels) || packet[18] != 0 {
        return Err(AudioError::Format(
            "only mono and stereo Opus are supported".into(),
        ));
    }
    Ok(OpusHead {
        channels,
        pre_skip: u16::from_le_bytes([packet[10], packet[11]]),
        input_rate: u32::from_le_bytes([packet[12], packet[13], packet[14], packet[15]]),
        gain: i16::from_le_bytes([packet[16], packet[17]]),
    })
}

/// The `OpusTags` packet: the vendor and `NAME=value` comments.
pub fn build_tags(vendor: &str, comments: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(TAGS_MAGIC);
    out.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    out.extend_from_slice(vendor.as_bytes());
    out.extend_from_slice(&(comments.len() as u32).to_le_bytes());
    for (name, value) in comments {
        let comment = format!("{name}={value}");
        out.extend_from_slice(&(comment.len() as u32).to_le_bytes());
        out.extend_from_slice(comment.as_bytes());
    }
    out
}

/// Reads an `OpusTags` packet into its vendor and comments.
pub fn parse_tags(packet: &[u8]) -> Result<(String, Vec<String>), AudioError> {
    let bad = || AudioError::Format("the OpusTags header is damaged".into());
    if packet.len() < 16 || &packet[..8] != TAGS_MAGIC {
        return Err(bad());
    }
    let mut at = 8;
    let take = |at: &mut usize| -> Result<&[u8], AudioError> {
        let len_bytes = packet.get(*at..*at + 4).ok_or_else(bad)?;
        let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        let start = *at + 4;
        let body = packet
            .get(start..start.checked_add(len).ok_or_else(bad)?)
            .ok_or_else(bad)?;
        *at = start + len;
        Ok(body)
    };
    let vendor = String::from_utf8_lossy(take(&mut at)?).into_owned();
    let count_bytes = packet.get(at..at + 4).ok_or_else(bad)?;
    let count = u32::from_le_bytes(count_bytes.try_into().unwrap());
    at += 4;
    let mut comments = Vec::new();
    for _ in 0..count {
        comments.push(String::from_utf8_lossy(take(&mut at)?).into_owned());
    }
    Ok((vendor, comments))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_head_has_the_layout_of_the_rfc() {
        let head = OpusHead {
            channels: 2,
            pre_skip: 312,
            input_rate: 48_000,
            gain: 0,
        };
        let bytes = build_head(&head);
        assert_eq!(bytes.len(), 19);
        assert_eq!(&bytes[..8], b"OpusHead");
        assert_eq!(bytes[8], 1);
        assert_eq!(bytes[9], 2);
        assert_eq!(&bytes[10..12], &312u16.to_le_bytes());
        assert_eq!(&bytes[12..16], &48_000u32.to_le_bytes());
        assert_eq!(bytes[18], 0);
        assert_eq!(parse_head(&bytes).unwrap(), head);
    }

    #[test]
    fn the_tags_have_the_layout_of_the_rfc() {
        let bytes = build_tags("v", &[("ENCODER", "kanso")]);
        assert_eq!(&bytes[..8], b"OpusTags");
        assert_eq!(&bytes[8..12], &1u32.to_le_bytes());
        assert_eq!(bytes[12], b'v');
        assert_eq!(&bytes[13..17], &1u32.to_le_bytes());
        assert_eq!(&bytes[17..21], &13u32.to_le_bytes());
        assert_eq!(&bytes[21..], b"ENCODER=kanso");
        let (vendor, comments) = parse_tags(&bytes).unwrap();
        assert_eq!(vendor, "v");
        assert_eq!(comments, ["ENCODER=kanso"]);
    }

    #[test]
    fn what_is_not_an_opus_head_is_refused() {
        assert!(parse_head(b"OpusHead").is_err());
        let mut bytes = build_head(&OpusHead {
            channels: 1,
            pre_skip: 0,
            input_rate: 48_000,
            gain: 0,
        });
        bytes[8] = 0x20;
        assert!(parse_head(&bytes).is_err());
        bytes[8] = 1;
        bytes[18] = 1;
        assert!(parse_head(&bytes).is_err());
        assert!(parse_tags(b"OpusTags\xff\xff\xff\xff").is_err());
        assert!(!sniff(b"RIFF"));
        assert!(!sniff(
            b"OggS\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x1eOpusHeax"
        ));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// Any head survives a round trip, and the tags too.
        #[test]
        fn prop_headers_round_trip(
            channels in 1u8..=2,
            pre_skip in any::<u16>(),
            rate in any::<u32>(),
            gain in any::<i16>(),
            vendor in ".{0,40}",
            comments in proptest::collection::vec(("[A-Z]{1,8}", ".{0,30}"), 0..5),
        ) {
            let head = OpusHead { channels, pre_skip, input_rate: rate, gain };
            prop_assert_eq!(parse_head(&build_head(&head)).unwrap(), head);
            let pairs: Vec<(&str, &str)> =
                comments.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            let (v, c) = parse_tags(&build_tags(&vendor, &pairs)).unwrap();
            prop_assert_eq!(v, vendor);
            let want: Vec<String> = comments.iter().map(|(a, b)| format!("{a}={b}")).collect();
            prop_assert_eq!(c, want);
        }

        /// Parsing arbitrary bytes never panics.
        #[test]
        fn prop_parsing_garbage_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
            let _ = parse_head(&bytes);
            let _ = parse_tags(&bytes);
            let _ = sniff(&bytes);
        }
    }
}
