//! Cleaning what a recogniser heard into text worth keeping. Pure.

use super::model::Segment;

/// A pause between two segments longer than this starts a new paragraph.
pub const PAUSE_MS: u64 = 2_000;

/// `c` in capitals, when that is exactly one character that lowercases back
/// to `c`; so cleaning never changes a word beyond its case.
fn capital(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) if u.to_lowercase().eq(std::iter::once(c)) => u,
        _ => c,
    }
}

/// One paragraph: its words separated by single spaces, and the first
/// letter, and the first letter after `.`, `?` or `!`, in capitals.
fn tidy_paragraph(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut upper_next = true;
    for word in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        for c in word.chars() {
            if upper_next && c.is_alphabetic() {
                out.push(capital(c));
                upper_next = false;
            } else {
                out.push(c);
                if c.is_alphanumeric() {
                    upper_next = false;
                }
            }
        }
        if word.ends_with(['.', '?', '!', '…']) {
            upper_next = true;
        }
    }
    out
}

/// The transcript as paragraphs: spaces collapsed, the first word of each
/// sentence in capitals. Existing blank-line breaks stay. Never loses a
/// word and changes nothing but spacing and capitals; cleaning clean text
/// changes nothing.
pub fn clean_transcript(text: &str) -> String {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.trim().is_empty() {
                paragraphs.push(tidy_paragraph(&current));
            }
            current.clear();
        } else {
            current.push_str(line);
            current.push(' ');
        }
    }
    if !current.trim().is_empty() {
        paragraphs.push(tidy_paragraph(&current));
    }
    paragraphs.join("\n\n")
}

/// The text of `segments` with a blank line wherever the pause before a
/// segment was longer than [`PAUSE_MS`], then cleaned.
pub fn transcript_from_segments(segments: &[Segment]) -> String {
    let mut out = String::new();
    let mut last_end: Option<u64> = None;
    for segment in segments {
        let text = segment.text.trim();
        if text.is_empty() {
            continue;
        }
        if let Some(end) = last_end {
            let gap = segment.start_ms.saturating_sub(end);
            out.push_str(if gap > PAUSE_MS { "\n\n" } else { " " });
        }
        out.push_str(text);
        last_end = Some(segment.end_ms.max(segment.start_ms));
    }
    clean_transcript(&out)
}

/// The note's title for a transcript: its first sentence, cut like any
/// capture's title. Empty text gives "Voice note".
pub fn voice_title(transcript: &str) -> String {
    let flat = transcript.split_whitespace().collect::<Vec<_>>().join(" ");
    let end = flat
        .char_indices()
        .find(|&(at, c)| {
            matches!(c, '.' | '?' | '!')
                && flat[at + c.len_utf8()..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        })
        .map(|(at, c)| at + c.len_utf8());
    let sentence = end.map_or(flat.as_str(), |end| &flat[..end]);
    let sentence = sentence.trim_end_matches(['.', '!']).trim();
    if sentence.is_empty() {
        "Voice note".to_owned()
    } else {
        crate::capture::cut_title(&clean_transcript(sentence))
    }
}

/// "0:24" (under ten minutes) or "52 min": a duration for a summary line.
pub fn duration_label(duration_ms: u64) -> String {
    let secs = duration_ms / 1000;
    if secs < 600 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{} min", secs / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_lowercase).collect()
    }

    fn seg(text: &str, start: u64, end: u64) -> Segment {
        Segment {
            text: text.into(),
            start_ms: start,
            end_ms: end,
        }
    }

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x0076_6f69_6365),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        #[test]
        fn cleaning_never_loses_a_word(text in "(\\PC|\\s){0,200}") {
            prop_assert_eq!(words(&clean_transcript(&text)), words(&text));
        }

        #[test]
        fn cleaning_is_idempotent(text in "(\\PC|\\s){0,200}") {
            let once = clean_transcript(&text);
            prop_assert_eq!(clean_transcript(&once), once);
        }

        #[test]
        fn segments_never_lose_a_word(
            parts in proptest::collection::vec(("[a-z ]{0,12}", 0u64..5000, 0u64..5000), 0..8)
        ) {
            let segments: Vec<Segment> = parts
                .iter()
                .map(|(t, a, b)| seg(t, *a, *b))
                .collect();
            let joined: String = parts.iter().map(|(t, ..)| t.as_str()).collect::<Vec<_>>().join(" ");
            prop_assert_eq!(words(&transcript_from_segments(&segments)), words(&joined));
        }

        #[test]
        fn a_title_is_short_and_never_empty(text in "\\PC{0,300}") {
            let title = voice_title(&text);
            prop_assert!(!title.is_empty());
            prop_assert!(title.chars().count() <= crate::capture::TITLE_WORDS_LEN + 1);
        }
    }

    // no-prop-test: fixed sentences with known results.
    #[test]
    fn examples() {
        assert_eq!(
            clean_transcript("  remind me to call.   the plumber\n\n tomorrow "),
            "Remind me to call. The plumber\n\nTomorrow"
        );
        assert_eq!(
            voice_title("call the plumber. then lunch"),
            "Call the plumber"
        );
        assert_eq!(voice_title("   "), "Voice note");
        assert_eq!(duration_label(24_000), "0:24");
        assert_eq!(duration_label(52 * 60_000), "52 min");
    }

    // no-prop-test: a gap of exactly two seconds is not a pause.
    #[test]
    fn pauses_longer_than_two_seconds_break_paragraphs() {
        let got = transcript_from_segments(&[
            seg("first part", 0, 1000),
            seg("same breath", 3000, 4000),
            seg("new thought", 6500, 7000),
        ]);
        assert_eq!(got, "First part same breath\n\nNew thought");
    }
}
