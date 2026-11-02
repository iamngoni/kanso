//! Passages: the parts of a note that resurfacing quotes.
//!
//! A note's one vector finds the right note but cannot say which part
//! matched, so each note is also cut into passages (paragraphs merged to
//! [`MIN_WORDS`]..[`MAX_WORDS`] words, at most [`MAX_PASSAGES`]) and each
//! passage gets a vector of its own, kept as int8 to stay small. Passages
//! are made from the same words as the note's vector (the first
//! `TEXT_CHARS` characters of the body), in the same batch, and replaced
//! whenever the note's vector is.

use kanso_core::links::plain_line;

/// A passage is merged from paragraphs until it has this many words.
pub const MIN_WORDS: usize = 40;
/// A passage is never longer than this many words.
pub const MAX_WORDS: usize = 120;
/// A note has at most this many passages.
pub const MAX_PASSAGES: usize = 24;

/// A part of a note's body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Passage {
    /// The order in the note, from 0.
    pub ord: usize,
    /// The byte range in the body.
    pub start: usize,
    pub end: usize,
    /// The passage as plain words, which is what is embedded.
    pub text: String,
}

/// A block of the body: a byte range and how many plain words it holds.
struct Block {
    start: usize,
    end: usize,
    words: usize,
}

/// Markdown as plain words on one line: fenced code is left out, and so
/// are emphasis and code markers, link targets and block markers.
pub fn plain_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let plain = plain_line(line);
        if plain.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&plain);
    }
    out
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// The blank-line separated blocks of `body[..limit]`, outside fences,
/// each cut into pieces of at most [`MAX_WORDS`] raw words.
fn blocks(body: &str, limit: usize) -> Vec<Block> {
    let mut out = Vec::new();
    let mut in_fence = false;
    // The current block: where it starts and ends.
    let mut open: Option<(usize, usize)> = None;
    let mut offset = 0;
    for line in body[..limit].split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        let trimmed = content.trim_start();
        let fence = trimmed.starts_with("```") || trimmed.starts_with("~~~");
        if fence {
            in_fence = !in_fence;
        }
        if fence || in_fence || content.trim().is_empty() {
            if let Some((first, last)) = open.take() {
                split_long(body, first, last, &mut out);
            }
            continue;
        }
        let end = start + content.len();
        open = Some(match open {
            Some((first, _)) => (first, end),
            None => (start, end),
        });
    }
    if let Some((first, last)) = open.take() {
        split_long(body, first, last, &mut out);
    }
    out
}

/// Pushes `body[start..end]` as blocks of at most [`MAX_WORDS`] words.
fn split_long(body: &str, start: usize, end: usize, out: &mut Vec<Block>) {
    let slice = &body[start..end];
    let mut piece_start: Option<usize> = None;
    let mut piece_end = 0;
    let mut count = 0;
    let mut cursor = 0;
    for word in slice.split_whitespace() {
        let at = cursor + slice[cursor..].find(word).unwrap_or(0);
        cursor = at + word.len();
        if count == MAX_WORDS {
            push_piece(body, start, piece_start.take(), piece_end, out);
            count = 0;
        }
        if piece_start.is_none() {
            piece_start = Some(at);
        }
        piece_end = cursor;
        count += 1;
    }
    push_piece(body, start, piece_start, piece_end, out);
}

fn push_piece(body: &str, base: usize, from: Option<usize>, to: usize, out: &mut Vec<Block>) {
    let Some(from) = from else { return };
    let (start, end) = (base + from, base + to);
    let words = word_count(&plain_text(&body[start..end]));
    if words > 0 {
        out.push(Block { start, end, words });
    }
}

/// The passages of `body`: its paragraphs merged to between [`MIN_WORDS`]
/// and [`MAX_WORDS`] words (a note's last one may be shorter), in order,
/// never overlapping, at most [`MAX_PASSAGES`], from the first
/// `char_limit` characters of the body.
pub fn passages(body: &str, char_limit: usize) -> Vec<Passage> {
    let limit = body
        .char_indices()
        .nth(char_limit)
        .map_or(body.len(), |(at, _)| at);
    let mut merged: Vec<Block> = Vec::new();
    for block in blocks(body, limit) {
        match merged.last_mut() {
            Some(last) if last.words < MIN_WORDS && last.words + block.words <= MAX_WORDS => {
                last.end = block.end;
                last.words += block.words;
            }
            _ => merged.push(block),
        }
    }
    // A short last passage joins the one before it when it fits.
    if merged.len() > 1 {
        let last = &merged[merged.len() - 1];
        let before = &merged[merged.len() - 2];
        if last.words < MIN_WORDS && before.words + last.words <= MAX_WORDS {
            let last = merged.pop().expect("two passages");
            let before = merged.last_mut().expect("one passage");
            before.end = last.end;
            before.words += last.words;
        }
    }
    merged.truncate(MAX_PASSAGES);
    merged
        .into_iter()
        .enumerate()
        .map(|(ord, block)| Passage {
            ord,
            start: block.start,
            end: block.end,
            text: plain_text(&body[block.start..block.end]),
        })
        .collect()
}

/// A vector as int8: unit length, then scaled so its largest number is
/// 127. The scale is what turns the numbers back.
pub fn quantize(vector: &[f32]) -> (f32, Vec<i8>) {
    let norm = vector
        .iter()
        .map(|x| f64::from(*x) * f64::from(*x))
        .sum::<f64>()
        .sqrt();
    if norm <= f64::EPSILON || !norm.is_finite() {
        return (0., vec![0; vector.len()]);
    }
    let unit: Vec<f32> = vector
        .iter()
        .map(|x| (f64::from(*x) / norm) as f32)
        .collect();
    let max = unit.iter().fold(0f32, |m, x| m.max(x.abs()));
    if max <= 0. {
        return (0., vec![0; vector.len()]);
    }
    let scale = max / 127.;
    let numbers = unit
        .iter()
        .map(|x| (x / scale).round().clamp(-127., 127.) as i8)
        .collect();
    (scale, numbers)
}

/// The cosine of two vectors made by [`quantize`].
pub fn dot_i8(a: &[i8], scale_a: f32, b: &[i8], scale_b: f32) -> f32 {
    if a.len() != b.len() {
        return 0.;
    }
    let dot: i32 = a
        .iter()
        .zip(b)
        .map(|(x, y)| i32::from(*x) * i32::from(*y))
        .sum();
    (dot as f32 * scale_a * scale_b).clamp(-1., 1.)
}

/// An int8 vector as stored: its scale as a little-endian f32, then its
/// numbers.
pub fn encode_i8(scale: f32, numbers: &[i8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + numbers.len());
    out.extend_from_slice(&scale.to_le_bytes());
    out.extend(numbers.iter().map(|n| *n as u8));
    out
}

/// A stored int8 vector, or `None` when the bytes don't hold `dim` numbers.
pub fn decode_i8(bytes: &[u8], dim: usize) -> Option<(f32, Vec<i8>)> {
    if bytes.len() != dim + 4 {
        return None;
    }
    let scale = f32::from_le_bytes(bytes[..4].try_into().ok()?);
    Some((scale, bytes[4..].iter().map(|b| *b as i8).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::cosine;
    use proptest::prelude::*;

    fn body_strategy() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                "[a-z]{1,8}( [a-z]{1,8}){0,30}\n?",
                Just("\n\n".to_owned()),
                Just("```\ncode here\n```\n".to_owned()),
                Just("- item é ü\n".to_owned()),
            ],
            0..40,
        )
        .prop_map(|parts| parts.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: None,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0070_6173_7367),
            ..ProptestConfig::default()
        })]

        /// Passages are in order, apart, inside the body, on character
        /// boundaries, and no longer than the limit.
        #[test]
        fn prop_passages_are_ordered_apart_and_inside_the_body(body in body_strategy()) {
            let found = passages(&body, 6_000);
            prop_assert!(found.len() <= MAX_PASSAGES);
            let mut end = 0;
            for (ord, p) in found.iter().enumerate() {
                prop_assert_eq!(p.ord, ord);
                prop_assert!(p.start >= end, "overlap at {}", ord);
                prop_assert!(p.start < p.end && p.end <= body.len());
                prop_assert!(body.is_char_boundary(p.start) && body.is_char_boundary(p.end));
                prop_assert!(word_count(&p.text) <= MAX_WORDS);
                prop_assert!(!p.text.is_empty());
                end = p.end;
            }
        }

        /// Every word of a short body outside fenced code is in a passage
        /// (unless the 24 passages ran out first).
        #[test]
        fn prop_passages_cover_the_words(body in body_strategy()) {
            let found = passages(&body, 6_000);
            if found.len() < MAX_PASSAGES && body.len() <= 6_000 {
                let in_passages: usize = found.iter().map(|p| word_count(&p.text)).sum();
                prop_assert_eq!(in_passages, word_count(&plain_text(&body)));
            }
        }

        /// Quantizing to int8 keeps the cosine within 0.02.
        #[test]
        fn prop_int8_keeps_the_cosine_within_two_hundredths(
            a in proptest::collection::vec(-1f32..1f32, 64),
            b in proptest::collection::vec(-1f32..1f32, 64),
        ) {
            let exact = cosine(&a, &b);
            let (sa, qa) = quantize(&a);
            let (sb, qb) = quantize(&b);
            let approx = dot_i8(&qa, sa, &qb, sb);
            prop_assert!((exact - approx).abs() <= 0.02, "{} vs {}", exact, approx);
        }
    }

    #[test]
    fn short_paragraphs_merge_and_long_ones_split() {
        let short = "one two three.\n\nfour five six.\n\nseven eight.";
        let found = passages(short, 6_000);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "one two three. four five six. seven eight.");
        let long = vec!["word"; 300].join(" ");
        let found = passages(&long, 6_000);
        assert_eq!(found.len(), 3);
        assert_eq!(word_count(&found[0].text), 120);
        assert_eq!(word_count(&found[2].text), 60);
        assert!(passages("", 6_000).is_empty());
        assert!(passages("```\nonly code\n```", 6_000).is_empty());
    }

    #[test]
    fn int8_vectors_round_trip_through_bytes() {
        let (scale, numbers) = quantize(&[0.5, -0.25, 0.1, 0.9]);
        let bytes = encode_i8(scale, &numbers);
        assert_eq!(decode_i8(&bytes, 4), Some((scale, numbers)));
        assert_eq!(decode_i8(&bytes, 5), None);
        let (zero_scale, zeros) = quantize(&[0., 0.]);
        assert_eq!((zero_scale, zeros), (0., vec![0, 0]));
    }
}
