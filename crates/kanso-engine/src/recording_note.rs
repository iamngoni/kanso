//! The shape of a note that holds a recording: a timestamped transcript
//! between invisible markers, each paragraph linking to its moment with a
//! `kanso-audio://file:<id>?t=<seconds>` link. Re-transcribing replaces
//! exactly what is between the markers, and never prose written around
//! them. The Markdown is the truth: there is no hidden map. Pure.

pub use crate::recording_note_model::{AudioMoment, Segment};

/// The scheme of a link to a moment in a recording.
pub const AUDIO_SCHEME: &str = "kanso-audio://";
/// Opens the transcript.
pub const TRANSCRIPT_OPEN: &str = "<!-- kanso:transcript -->";
/// Closes the transcript.
pub const TRANSCRIPT_CLOSE: &str = "<!-- /kanso:transcript -->";

/// `00:12`, or `1:02:03` past an hour. Pure.
pub fn stamp(seconds: u32) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// The link for a moment: `[00:12](kanso-audio://file:abc?t=12)`. Pure.
pub fn moment_link(file_id: &str, seconds: u32) -> String {
    format!("[{}]({AUDIO_SCHEME}{file_id}?t={seconds})", stamp(seconds))
}

/// The moment a `kanso-audio:` URL names; no `t` is the start. Pure.
pub fn parse_moment(url: &str) -> Option<AudioMoment> {
    let rest = url.strip_prefix(AUDIO_SCHEME)?;
    let (id, query) = rest.split_once('?').unwrap_or((rest, ""));
    let valid = id.len() > "file:".len()
        && id.starts_with("file:")
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'));
    if !valid {
        return None;
    }
    let seconds = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("t="))
        .map_or(Some(0), |t| t.parse().ok())?;
    Some(AudioMoment {
        file_id: id.to_owned(),
        seconds,
    })
}

/// The transcript block: one paragraph per segment, each with its own
/// timestamp, between the markers. Empty segments are left out; nothing
/// but empty ones is `None`. Pure.
pub fn render_transcript(file_id: &str, segments: &[Segment]) -> Option<String> {
    let paragraphs: Vec<String> = segments
        .iter()
        .filter(|s| !s.text.trim().is_empty())
        .map(|s| format!("{} {}", moment_link(file_id, s.start_secs), clean(&s.text)))
        .collect();
    if paragraphs.is_empty() {
        return None;
    }
    Some(format!(
        "{TRANSCRIPT_OPEN}\n{}\n{TRANSCRIPT_CLOSE}",
        paragraphs.join("\n\n")
    ))
}

/// Words on one line, and never a marker of ours. Pure.
fn clean(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("<!--", "< !--")
}

/// Where the marked transcript is in `body`: from the opening marker to
/// the end of the closing one. Pure.
pub fn transcript_range(body: &str) -> Option<std::ops::Range<usize>> {
    let start = body.find(TRANSCRIPT_OPEN)?;
    let close = body[start..].find(TRANSCRIPT_CLOSE)? + start;
    Some(start..close + TRANSCRIPT_CLOSE.len())
}

/// `body` with the marked transcript swapped for `block`, or `None` when
/// the markers are gone (the caller then asks before writing anywhere).
/// Everything outside the markers is untouched. Pure.
pub fn replace_transcript(body: &str, block: &str) -> Option<String> {
    let range = transcript_range(body)?;
    Some(format!(
        "{}{block}{}",
        &body[..range.start],
        &body[range.end..]
    ))
}

/// The title of a recording's note, from a date text such as `23 Oct 2026,
/// 14:05`. Within the title limit. Pure.
pub fn title(mic_only: bool, when: &str) -> String {
    let kind = if mic_only { "Recording" } else { "Meeting" };
    format!("{kind} — {}", when.chars().take(100).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn seg(start_secs: u32, text: &str) -> Segment {
        Segment {
            start_secs,
            text: text.into(),
        }
    }

    #[test]
    fn the_block_matches_the_design() {
        let block = render_transcript(
            "file:abc123",
            &[seg(0, "We started."), seg(22, "The beta is on track.")],
        )
        .unwrap();
        assert_eq!(
            block,
            "<!-- kanso:transcript -->\n[00:00](kanso-audio://file:abc123?t=0) We started.\n\n[00:22](kanso-audio://file:abc123?t=22) The beta is on track.\n<!-- /kanso:transcript -->"
        );
        assert_eq!(render_transcript("file:a", &[seg(0, "  ")]), None);
        assert_eq!(stamp(3723), "1:02:03");
        assert_eq!(stamp(59), "00:59");
    }

    #[test]
    fn retranscribing_replaces_only_the_marked_part() {
        let old = render_transcript("file:a", &[seg(0, "old")]).unwrap();
        let new = render_transcript("file:a", &[seg(0, "new")]).unwrap();
        let body = format!("# T\n\nMy prose\n\n{old}\n\nAfter");
        let out = replace_transcript(&body, &new).unwrap();
        assert_eq!(out, format!("# T\n\nMy prose\n\n{new}\n\nAfter"));
        assert_eq!(replace_transcript("no markers", &new), None);
        assert_eq!(replace_transcript(TRANSCRIPT_OPEN, &new), None);
    }

    #[test]
    fn moments_parse() {
        assert_eq!(
            parse_moment("kanso-audio://file:abc?t=12"),
            Some(AudioMoment {
                file_id: "file:abc".into(),
                seconds: 12
            })
        );
        assert_eq!(parse_moment("kanso-audio://file:abc").unwrap().seconds, 0);
        assert_eq!(parse_moment("kanso-audio://file:abc?t=x"), None);
        assert_eq!(parse_moment("kanso-audio://note:abc?t=1"), None);
        assert_eq!(parse_moment("inkdrop://file:abc?t=1"), None);
        assert!(title(false, "23 Oct 2026, 14:05").starts_with("Meeting — "));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        /// Every link a transcript writes parses back to its segment's
        /// start, and the markers are found exactly around the block.
        #[test]
        fn prop_links_round_trip_and_block_is_found(
            id in "file:[a-z0-9]{3,12}",
            starts in prop::collection::vec(0u32..30_000, 1..8),
            words in "[A-Za-z]{1,8}( [a-z]{1,6}){0,5}",
            before in "[a-z \n]{0,20}",
            after in "[a-z \n]{0,20}",
        ) {
            let segments: Vec<Segment> = starts.iter().map(|s| seg(*s, &words)).collect();
            let block = render_transcript(&id, &segments).unwrap();
            for s in &starts {
                let link = moment_link(&id, *s);
                prop_assert!(block.contains(&link));
                let url = link.split('(').nth(1).unwrap().trim_end_matches(')');
                prop_assert_eq!(parse_moment(url), Some(AudioMoment { file_id: id.clone(), seconds: *s }));
            }
            let body = format!("{before}{block}{after}");
            let range = transcript_range(&body).unwrap();
            prop_assert_eq!(&body[range], block.as_str());
            let out = replace_transcript(&body, "X").unwrap();
            prop_assert_eq!(out, format!("{before}X{after}"));
        }

        /// Words never smuggle in a marker.
        #[test]
        fn prop_words_cannot_break_the_markers(text in ".{0,60}") {
            if let Some(block) = render_transcript("file:a", &[seg(1, &text)]) {
                prop_assert_eq!(block.matches(TRANSCRIPT_OPEN).count(), 1);
                prop_assert_eq!(block.matches(TRANSCRIPT_CLOSE).count(), 1);
            }
        }
    }
}
