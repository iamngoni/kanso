//! Offsets into a text in the units an editor counts in. The engine speaks
//! UTF-8 bytes; Flutter's `TextEditingValue` and the web speak UTF-16 code
//! units. These convert, and never land inside a character: an offset
//! between the two halves of a surrogate pair (or past the end of the text)
//! goes to the start of the character (the end of the text).
//!
//! [`utf16_to_byte`] and [`byte_to_utf16`] walk the text from its start.
//! [`Utf16Index`] answers the same questions in time that doesn't grow with
//! the text, for an editor that asks on every keystroke of a long note.

use std::ops::Range;

/// The UTF-8 byte offset of UTF-16 offset `utf16` in `text`, at a
/// character boundary: the start of the character that unit is in. Past the
/// end it is the text's length. Pure.
pub fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        let next = units + ch.len_utf16();
        if next > utf16 {
            return byte;
        }
        units = next;
    }
    text.len()
}

/// The UTF-16 offset of UTF-8 byte offset `byte` in `text`; an offset inside
/// a character counts as that character's start, and one past the end as
/// the end. Pure.
pub fn byte_to_utf16(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    text[..byte].chars().map(char::len_utf16).sum()
}

/// How many characters apart [`Utf16Index`] keeps its checkpoints.
const CHECKPOINT_EVERY: usize = 1024;

/// A position written both ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Checkpoint {
    byte: usize,
    utf16: usize,
}

/// Conversions between UTF-8 byte offsets and UTF-16 offsets in one text,
/// without walking the text from its start. It keeps a checkpoint every
/// 1,024 characters, so a conversion is a binary search over the
/// checkpoints and a walk of at most one gap, and a splice moves the
/// checkpoints after it and rewrites those around it.
///
/// It doesn't hold the text: each call is given the text the index
/// describes, which must be the one the index was built from with every
/// [`Utf16Index::splice`] since applied. Its answers are those of
/// [`utf16_to_byte`] and [`byte_to_utf16`].
#[derive(Clone, Debug)]
pub struct Utf16Index {
    /// Sorted by both fields, the first at the start of the text.
    points: Vec<Checkpoint>,
    /// The end of the text.
    end: Checkpoint,
    every: usize,
}

impl Utf16Index {
    /// The index of `text`.
    pub fn new(text: &str) -> Self {
        Self::with_gap(text, CHECKPOINT_EVERY)
    }

    /// The index of `text` with a checkpoint every `every` characters; for
    /// tests, which want many checkpoints in a short text.
    pub fn with_gap(text: &str, every: usize) -> Self {
        let every = every.max(1);
        let mut index = Self {
            points: vec![Checkpoint { byte: 0, utf16: 0 }],
            end: Checkpoint { byte: 0, utf16: 0 },
            every,
        };
        let mut points = Vec::new();
        index.end = scan(text, Checkpoint { byte: 0, utf16: 0 }, every, &mut points);
        index.points.extend(points);
        index
    }

    /// The text's length in UTF-16 units.
    pub fn len_utf16(&self) -> usize {
        self.end.utf16
    }

    /// [`utf16_to_byte`] of `text`.
    pub fn utf16_to_byte(&self, text: &str, utf16: usize) -> usize {
        if utf16 >= self.end.utf16 {
            return text.len();
        }
        let point = self.points[self.points.partition_point(|p| p.utf16 <= utf16) - 1];
        let mut units = point.utf16;
        for (byte, ch) in text[point.byte..].char_indices() {
            let next = units + ch.len_utf16();
            if next > utf16 {
                return point.byte + byte;
            }
            units = next;
        }
        text.len()
    }

    /// [`byte_to_utf16`] of `text`.
    pub fn byte_to_utf16(&self, text: &str, byte: usize) -> usize {
        let mut byte = byte.min(text.len());
        while !text.is_char_boundary(byte) {
            byte -= 1;
        }
        let point = self.points[self.points.partition_point(|p| p.byte <= byte) - 1];
        point.utf16
            + text[point.byte..byte]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>()
    }

    /// The text changed: `range` of `old` (byte offsets, on character
    /// boundaries) became `inserted`. Call it with the text from before
    /// the change; the index then describes the text after it.
    pub fn splice(&mut self, old: &str, range: Range<usize>, inserted: &str) {
        let removed_units: usize = old[range.clone()].chars().map(char::len_utf16).sum();
        let inserted_units: usize = inserted.chars().map(char::len_utf16).sum();
        // The checkpoints the change leaves alone, before it...
        let first = self.points.partition_point(|p| p.byte <= range.start) - 1;
        // ...and after it, which only move.
        let last = self.points.partition_point(|p| p.byte <= range.end);
        let from = self.points[first];
        let to_byte = self.points.get(last).map_or(old.len(), |p| p.byte);
        // The text around the change as it is now, to put checkpoints in.
        let mut local = String::with_capacity(to_byte - from.byte + inserted.len());
        local.push_str(&old[from.byte..range.start]);
        local.push_str(inserted);
        local.push_str(&old[range.end..to_byte]);
        let mut fresh = Vec::new();
        let local_end = scan(&local, from, self.every, &mut fresh);
        let shift = |p: &mut Checkpoint| {
            p.byte = p.byte - range.len() + inserted.len();
            p.utf16 = p.utf16 - removed_units + inserted_units;
        };
        for point in &mut self.points[last..] {
            shift(point);
        }
        if last == self.points.len() {
            self.end = local_end;
        } else {
            shift(&mut self.end);
        }
        self.points.splice(first + 1..last, fresh);
    }
}

/// Appends to `points` a checkpoint every `every` characters of `text`,
/// which starts at `start`; returns where `text` ends.
fn scan(text: &str, start: Checkpoint, every: usize, points: &mut Vec<Checkpoint>) -> Checkpoint {
    let (mut byte, mut utf16) = (start.byte, start.utf16);
    for (count, ch) in text.chars().enumerate() {
        if count > 0 && count % every == 0 {
            points.push(Checkpoint { byte, utf16 });
        }
        byte += ch.len_utf8();
        utf16 += ch.len_utf16();
    }
    Checkpoint { byte, utf16 }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    #[test]
    fn offsets_convert_across_characters_of_every_width() {
        // 'é' is 2 bytes and 1 unit, '日' 3 and 1, '🙂' 4 and 2.
        let text = "aé日🙂b";
        assert_eq!(utf16_to_byte(text, 0), 0);
        assert_eq!(utf16_to_byte(text, 1), 1);
        assert_eq!(utf16_to_byte(text, 2), 3);
        assert_eq!(utf16_to_byte(text, 3), 6);
        // Inside the pair: the start of the emoji.
        assert_eq!(utf16_to_byte(text, 4), 6);
        assert_eq!(utf16_to_byte(text, 5), 10);
        assert_eq!(utf16_to_byte(text, 6), 11);
        assert_eq!(utf16_to_byte(text, 99), 11);
        assert_eq!(byte_to_utf16(text, 6), 3);
        assert_eq!(byte_to_utf16(text, 10), 5);
        // Inside a character: its start.
        assert_eq!(byte_to_utf16(text, 7), 3);
        assert_eq!(byte_to_utf16(text, 99), 6);
        assert_eq!(utf16_to_byte("", 3), 0);
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Every character boundary converts to a UTF-16 offset and back
        /// to itself.
        #[test]
        fn prop_boundaries_round_trip(text in "\\PC{0,24}") {
            for (byte, _) in text.char_indices().chain([(text.len(), ' ')]) {
                let units = byte_to_utf16(&text, byte);
                prop_assert_eq!(utf16_to_byte(&text, units), byte);
            }
        }

        /// A UTF-16 offset anywhere gives a character boundary, no later
        /// than the offset, and a larger offset never gives a smaller one.
        #[test]
        fn prop_utf16_offsets_floor_to_a_boundary_and_never_go_back(
            text in "\\PC{0,24}",
            a in 0usize..60,
            b in 0usize..60,
        ) {
            let (a, b) = (a.min(b), a.max(b));
            let (low, high) = (utf16_to_byte(&text, a), utf16_to_byte(&text, b));
            prop_assert!(text.is_char_boundary(low) && text.is_char_boundary(high));
            prop_assert!(low <= high && high <= text.len());
            prop_assert!(byte_to_utf16(&text, low) <= a);
        }

        /// A byte offset anywhere (inside a character too) gives a UTF-16
        /// offset that is never past the text and never goes back as the
        /// byte offset grows.
        #[test]
        fn prop_byte_offsets_never_go_back(text in "\\PC{0,24}", a in 0usize..120, b in 0usize..120) {
            let (a, b) = (a.min(b), a.max(b));
            let total: usize = text.chars().map(char::len_utf16).sum();
            prop_assert!(byte_to_utf16(&text, a) <= byte_to_utf16(&text, b));
            prop_assert!(byte_to_utf16(&text, b) <= total);
        }
    }

    // MARK: Utf16Index

    /// The pieces of a splice: two fractions of the old text's length, put
    /// on character boundaries, and the text that goes in.
    fn splices() -> impl Strategy<Value = Vec<(f64, f64, String)>> {
        prop::collection::vec((0f64..=1., 0f64..=1., "\\PC{0,40}"), 0..12)
    }

    fn floor_boundary(text: &str, fraction: f64) -> usize {
        let mut at = (text.len() as f64 * fraction) as usize;
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        at
    }

    /// Checks `index` against the free functions on every character
    /// boundary and on every UTF-16 offset.
    fn agrees(index: &Utf16Index, text: &str) -> Result<(), TestCaseError> {
        let total: usize = text.chars().map(char::len_utf16).sum();
        prop_assert_eq!(index.len_utf16(), total);
        for byte in 0..=text.len() + 1 {
            prop_assert_eq!(index.byte_to_utf16(text, byte), byte_to_utf16(text, byte));
        }
        for units in 0..=total + 1 {
            prop_assert_eq!(index.utf16_to_byte(text, units), utf16_to_byte(text, units));
        }
        Ok(())
    }

    #[test]
    fn index_examples() {
        let text = "aé日🙂b".repeat(5);
        let index = Utf16Index::with_gap(&text, 3);
        assert_eq!(index.len_utf16(), 30);
        assert_eq!(index.utf16_to_byte(&text, 4), 6);
        assert_eq!(index.utf16_to_byte(&text, 99), text.len());
        assert_eq!(index.byte_to_utf16(&text, 7), 3);
        let mut index = Utf16Index::new("");
        index.splice("", 0..0, "🙂");
        assert_eq!(index.len_utf16(), 2);
        assert_eq!(index.utf16_to_byte("🙂", 1), 0);
    }

    #[test]
    fn a_long_text_with_the_default_gap_converts_and_splices() {
        let mut text = "line é日🙂\n".repeat(900);
        let mut index = Utf16Index::new(&text);
        assert!(index.points.len() > 4);
        assert_eq!(
            index.len_utf16(),
            text.chars().map(char::len_utf16).sum::<usize>()
        );
        for (range, inserted) in [
            (5000..5003, "🙂🙂"),
            (0..0, "x"),
            (100..4000, ""),
            (text.len() - 3..text.len(), "end"),
        ] {
            let (mut start, mut end) = (range.start.min(text.len()), range.end.min(text.len()));
            while !text.is_char_boundary(start) {
                start -= 1;
            }
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            index.splice(&text, start..end, inserted);
            text.replace_range(start..end, inserted);
            for probe in (0..=text.len()).step_by(37) {
                assert_eq!(
                    index.byte_to_utf16(&text, probe),
                    byte_to_utf16(&text, probe)
                );
            }
            for probe in (0..index.len_utf16() + 2).step_by(41) {
                assert_eq!(
                    index.utf16_to_byte(&text, probe),
                    utf16_to_byte(&text, probe)
                );
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, ..config() })]

        /// A new index agrees with the walking functions, whatever the
        /// gap between checkpoints.
        #[test]
        fn prop_index_equals_the_walking_functions(text in "\\PC{0,60}", gap in 1usize..9) {
            agrees(&Utf16Index::with_gap(&text, gap), &text)?;
        }

        /// After any sequence of splices, the index still equals the
        /// walking functions on the spliced text, and equals an index built
        /// from that text afresh.
        #[test]
        fn prop_index_survives_any_splices(
            text in "\\PC{0,60}",
            splices in splices(),
            gap in 1usize..9,
        ) {
            let mut text = text;
            let mut index = Utf16Index::with_gap(&text, gap);
            for (a, b, inserted) in splices {
                let (a, b) = (floor_boundary(&text, a), floor_boundary(&text, b));
                let range = a.min(b)..a.max(b);
                index.splice(&text, range.clone(), &inserted);
                text.replace_range(range, &inserted);
                agrees(&index, &text)?;
                // No gap between checkpoints, nor after the last one, is
                // longer than the index promises.
                let mut ends = index.points.iter().map(|p| p.byte).collect::<Vec<_>>();
                ends.push(text.len());
                for pair in ends.windows(2) {
                    prop_assert!(pair[0] < pair[1] || text.is_empty() && pair[0] == pair[1]);
                    prop_assert!(text[pair[0]..pair[1]].chars().count() <= gap);
                }
            }
        }
    }
}
