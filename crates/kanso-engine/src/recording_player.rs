//! A recording in a note, for the player (issue #38, steps 5 and 6): finding
//! the recording's line, which parts it has, and what the keys do. Pure, so
//! the desktop and the phones read a note the same way.

use kanso_core::links::LINK_SCHEME;

pub use crate::recording_note::{AudioMoment, parse_moment};
pub use crate::recording_player_model::{PlayerKey, RecordingRef};

/// Seconds the arrow keys move.
pub const SKIP_SHORT_SECS: i32 = 5;
/// Seconds the arrow keys move with Shift.
pub const SKIP_LONG_SECS: i32 = 30;

/// The speeds the player offers, as written on its button.
pub const SPEED_LABELS: [(&str, f32); 5] = [
    ("0.75×", 0.75),
    ("1×", 1.0),
    ("1.25×", 1.25),
    ("1.5×", 1.5),
    ("2×", 2.0),
];

/// The file IDs on `line` when it is made only of attachment links
/// (`[Audio · part 1](inkdrop://file:a) · [part 2](inkdrop://file:b)`),
/// separated by dots or spaces; `None` for any other line. Images
/// (`![…](…)`) don't count. Pure.
fn file_links_only(line: &str) -> Option<Vec<String>> {
    let mut ids = Vec::new();
    let mut rest = line.trim();
    while !rest.is_empty() {
        rest = rest.strip_prefix('[')?;
        let close = rest.find("](")?;
        let after = &rest[close + 2..];
        let end = after.find(')')?;
        let target = &after[..end];
        let id = target.strip_prefix(LINK_SCHEME)?;
        let valid = id.starts_with("file:")
            && id.len() > "file:".len()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'));
        if !valid {
            return None;
        }
        if !ids.iter().any(|seen| seen == id) {
            ids.push(id.to_owned());
        }
        rest = after[end + 1..].trim_start_matches([' ', '\t', '·', '•', ',']);
    }
    (!ids.is_empty()).then_some(ids)
}

/// Every line of `body` that is only attachment links, in order, outside
/// code fences and the front matter. Pure.
pub fn link_lines(body: &str) -> Vec<RecordingRef> {
    let mut found = Vec::new();
    let mut fence: Option<&str> = None;
    let mut in_front_matter = false;
    for (line_no, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if line_no == 0 && line.trim_end() == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            if line.trim_end() == "---" || line.trim_end() == "..." {
                in_front_matter = false;
            }
            continue;
        }
        if let Some(open) = fence {
            if trimmed.starts_with(open) {
                fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            fence = Some("```");
            continue;
        }
        if trimmed.starts_with("~~~") {
            fence = Some("~~~");
            continue;
        }
        if let Some(file_ids) = file_links_only(line) {
            found.push(RecordingRef {
                file_ids,
                line: line_no,
                made_here: trimmed.starts_with("[Audio"),
            });
        }
    }
    found
}

/// The recording `file_id` is a part of: the line of links that holds it.
/// Pure.
pub fn recording_with(body: &str, file_id: &str) -> Option<RecordingRef> {
    link_lines(body)
        .into_iter()
        .find(|r| r.file_ids.iter().any(|id| id == file_id))
}

/// The timestamp link under byte `offset` of `text`, if it is on one:
/// `[00:12](kanso-audio://file:abc?t=12)` with the caret anywhere from its
/// `[` to its `)`. For a click in the source, where links are plain text.
/// Pure.
pub fn moment_at(text: &str, offset: usize) -> Option<AudioMoment> {
    if offset > text.len() || !text.is_char_boundary(offset) {
        return None;
    }
    let line_start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let line_end = text[offset..]
        .find('\n')
        .map_or(text.len(), |at| offset + at);
    let line = &text[line_start..line_end];
    let at = offset - line_start;
    let target = format!("]({}", crate::recording_note::AUDIO_SCHEME);
    let mut from = 0;
    while let Some(found) = line[from..].find(&target) {
        let close = from + found;
        let url_start = close + 2;
        let Some(end) = line[url_start..].find(')').map(|e| url_start + e) else {
            break;
        };
        if let Some(open) = line[..close].rfind('[')
            && (open..=end).contains(&at)
        {
            return parse_moment(&line[url_start..end]);
        }
        from = url_start;
    }
    None
}

/// What a key does, if it is one the player takes: Space plays and pauses,
/// ← and → move 5 seconds, Shift 30. `key` is the key's name (`space`,
/// `left`, `right`). Pure.
pub fn key_action(key: &str, shift: bool) -> Option<PlayerKey> {
    let step = if shift {
        SKIP_LONG_SECS
    } else {
        SKIP_SHORT_SECS
    };
    match key {
        "space" | " " if !shift => Some(PlayerKey::PlayPause),
        "left" => Some(PlayerKey::Skip(-step)),
        "right" => Some(PlayerKey::Skip(step)),
        _ => None,
    }
}

/// The time on the scrubber's ends: `12:34`, or `1:02:03` past an hour.
/// Pure.
pub fn clock(us: u64) -> String {
    crate::recording_note::stamp(u32::try_from(us / 1_000_000).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording_flow::audio_line;
    use proptest::prelude::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("file:rec-{i}")).collect()
    }

    #[test]
    fn a_kanso_recording_line_gives_its_parts_in_order() {
        let body = format!("# Meeting\n\n{}\n\nNotes", audio_line(&ids(3), 3 * 3600));
        assert_eq!(
            link_lines(&body),
            vec![RecordingRef {
                file_ids: ids(3),
                line: 2,
                made_here: true
            }]
        );
    }

    #[test]
    fn prose_images_fences_and_front_matter_are_not_recordings() {
        let body = "---\ntitle: x\n[a](inkdrop://file:front)\n---\n\
            See [a](inkdrop://file:in-text) here\n\
            ![pic](inkdrop://file:image)\n\
            ```\n[a](inkdrop://file:fenced)\n```\n\
            [voice.m4a](inkdrop://file:dropped)\n";
        let lines = link_lines(body);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].file_ids, vec!["file:dropped".to_owned()]);
        assert!(!lines[0].made_here);
        assert_eq!(recording_with(body, "file:dropped").unwrap().line, 9);
        assert_eq!(recording_with(body, "file:image"), None);
    }

    #[test]
    fn keys_do_what_the_design_says() {
        assert_eq!(key_action("space", false), Some(PlayerKey::PlayPause));
        assert_eq!(key_action("left", false), Some(PlayerKey::Skip(-5)));
        assert_eq!(key_action("right", true), Some(PlayerKey::Skip(30)));
        assert_eq!(key_action("left", true), Some(PlayerKey::Skip(-30)));
        assert_eq!(key_action("a", false), None);
        assert_eq!(clock(3_723_000_000), "1:02:03");
        assert_eq!(clock(754_000_000), "12:34");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The line Kanso writes reads back as the same parts, wherever it
        /// sits in the note and whatever is around it.
        #[test]
        fn prop_the_audio_line_reads_back(
            parts in 1usize..8,
            secs in 1u64..40_000,
            before in "[A-Za-z #\n]{0,60}",
            after in "[A-Za-z #\n]{0,60}",
        ) {
            let ids = ids(parts);
            let line = audio_line(&ids, secs);
            let body = format!("{before}\n{line}\n{after}");
            let found = recording_with(&body, &ids[parts - 1]).expect("found");
            prop_assert_eq!(found.file_ids, ids);
            prop_assert!(found.made_here);
            prop_assert_eq!(body.lines().nth(found.line).unwrap(), line.as_str());
        }

        /// Arbitrary text never panics, and every recording found is made of
        /// well-formed file IDs.
        #[test]
        fn prop_any_text_is_safe(body in ".{0,300}") {
            for r in link_lines(&body) {
                prop_assert!(!r.file_ids.is_empty());
                for id in r.file_ids {
                    prop_assert!(id.starts_with("file:"));
                }
            }
        }

        /// A caret anywhere on a timestamp link finds its moment; anywhere
        /// off it finds nothing.
        #[test]
        fn prop_a_caret_on_a_timestamp_finds_it(
            before in "[A-Za-z .]{0,30}",
            after in "[A-Za-z .]{0,30}",
            secs in 0u32..40_000,
            inside in 0usize..1000,
        ) {
            let link = crate::recording_note::moment_link("file:abc", secs);
            let text = format!("line one\n{before}{link}{after}\nline three");
            let start = "line one\n".len() + before.len();
            let at = start + inside % (link.len() + 1);
            let found = moment_at(&text, at).expect("on the link");
            prop_assert_eq!(found, AudioMoment { file_id: "file:abc".into(), seconds: secs });
            prop_assert_eq!(moment_at(&text, 0), None);
            prop_assert_eq!(moment_at(&text, text.len()), None);
            prop_assert_eq!(moment_at(&text, start + link.len() + 1 + after.len()), None);
        }

        /// The keys move by whole steps, back and forward alike.
        #[test]
        fn prop_skips_are_symmetric(shift in any::<bool>()) {
            let (Some(PlayerKey::Skip(back)), Some(PlayerKey::Skip(fwd))) =
                (key_action("left", shift), key_action("right", shift)) else {
                return Err(TestCaseError::fail("arrows must skip"));
            };
            prop_assert_eq!(back, -fwd);
        }
    }
}
