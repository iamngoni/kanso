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
        .map(|s| {
            let link = moment_link(file_id, s.start_secs);
            match &s.speaker {
                Some(who) => format!("{link} {who}: {}", clean(&s.text)),
                None => format!("{link} {}", clean(&s.text)),
            }
        })
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

/// Where `file_id`'s marked transcript is in `body`: from the opening
/// marker to the end of the closing one. A block counts only when it is
/// still machine text for this recording: its own markers (an opening
/// marker with no close before the next opening one is skipped), and
/// every line between them a paragraph that starts with a moment link to
/// `file_id`. So another recording's block, a block the user has written
/// their own lines into, and prose between a deleted closing marker and a
/// later block are never matched, and so never replaced. Pure.
pub fn transcript_range(body: &str, file_id: &str) -> Option<std::ops::Range<usize>> {
    let mut from = 0;
    while let Some(open) = body[from..].find(TRANSCRIPT_OPEN).map(|at| at + from) {
        let inner_start = open + TRANSCRIPT_OPEN.len();
        let close = body[inner_start..].find(TRANSCRIPT_CLOSE)? + inner_start;
        let inner = &body[inner_start..close];
        if let Some(nested) = inner.find(TRANSCRIPT_OPEN) {
            from = inner_start + nested;
            continue;
        }
        if is_machine_text(inner, file_id) {
            return Some(open..close + TRANSCRIPT_CLOSE.len());
        }
        from = close + TRANSCRIPT_CLOSE.len();
    }
    None
}

/// Whether every non-blank line of `inner` starts with a moment link to
/// `file_id`, and there is at least one. Pure.
fn is_machine_text(inner: &str, file_id: &str) -> bool {
    let target = format!("]({AUDIO_SCHEME}{file_id}?t=");
    let mut any = false;
    for line in inner.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let linked = line.starts_with('[')
            && line
                .find("](")
                .is_some_and(|at| line[at..].starts_with(&target));
        if !linked {
            return false;
        }
        any = true;
    }
    any
}

/// `body` with `file_id`'s marked transcript swapped for `block`, or
/// `None` when there is none to replace ([`transcript_range`]).
/// Everything outside the markers is untouched. Pure.
pub fn replace_transcript(body: &str, file_id: &str, block: &str) -> Option<String> {
    let range = transcript_range(body, file_id)?;
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
            speaker: None,
        }
    }

    #[test]
    fn labels_come_before_the_words_and_the_block_is_still_found() {
        let said = |start_secs, who, text: &str| Segment {
            start_secs,
            text: text.into(),
            speaker: who,
        };
        let block = render_transcript(
            "file:abc",
            &[
                said(0, Some("You".into()), "Hello."),
                said(22, Some("Meeting".into()), "Hi."),
                said(40, None, "Unlabelled."),
            ],
        )
        .unwrap();
        assert!(block.contains("[00:00](kanso-audio://file:abc?t=0) You: Hello."));
        assert!(block.contains("[00:22](kanso-audio://file:abc?t=22) Meeting: Hi."));
        assert!(block.contains("?t=40) Unlabelled."));
        let body = format!("Mine\n\n{block}\n\nTail");
        assert_eq!(
            transcript_range(&body, "file:abc").map(|r| &body[r]),
            Some(block.as_str())
        );
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
        let out = replace_transcript(&body, "file:a", &new).unwrap();
        assert_eq!(out, format!("# T\n\nMy prose\n\n{new}\n\nAfter"));
        assert_eq!(replace_transcript("no markers", "file:a", &new), None);
        assert_eq!(replace_transcript(TRANSCRIPT_OPEN, "file:a", &new), None);
    }

    #[test]
    fn edited_or_foreign_markers_never_cost_prose() {
        let a = render_transcript("file:a", &[seg(0, "old a")]).unwrap();
        let b = render_transcript("file:b", &[seg(0, "old b")]).unwrap();
        let new = render_transcript("file:b", &[seg(0, "new b")]).unwrap();
        // Two recordings: only b's block is b's.
        let body = format!("{a}\n\nMine\n\n{b}");
        assert_eq!(
            replace_transcript(&body, "file:b", &new).unwrap(),
            format!("{a}\n\nMine\n\n{new}")
        );
        // a's closing marker deleted: a's open must not pair with b's close.
        let cut = a.replace(TRANSCRIPT_CLOSE, "");
        let body = format!("{cut}\n\nMine\n\n{b}");
        assert_eq!(transcript_range(&body, "file:a"), None);
        assert_eq!(
            replace_transcript(&body, "file:b", &new).unwrap(),
            format!("{cut}\n\nMine\n\n{new}")
        );
        // A line of the user's own inside the markers: hands off.
        let mine = a.replace("old a", "old a\n\nMy own thought");
        assert_eq!(transcript_range(&mine, "file:a"), None);
        // Opening marker deleted: nothing to replace.
        assert_eq!(
            transcript_range(&a.replace(TRANSCRIPT_OPEN, ""), "file:a"),
            None
        );
        // file:a must not match file:ab.
        assert_eq!(transcript_range(&a, "file:ab"), None);
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
            let range = transcript_range(&body, &id).unwrap();
            prop_assert_eq!(&body[range], block.as_str());
            let out = replace_transcript(&body, &id, "X").unwrap();
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
