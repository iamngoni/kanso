//! The words read from the scanned pages of a PDF (issue #43, step 8).
//!
//! A scanned page is one picture: PDFium finds no characters on it. A reader
//! that can recognise text (Apple's Vision on a Mac) is given the page as an
//! image and answers with words and where they are. Those words are kept
//! here as [`OcrWord`]s, boxes normalised like a highlight quad, and a page
//! of them is an [`OcrPage`]. [`OcrPage::to_page_text`] turns it into the
//! same [`PageText`] PDFium's characters make, so the text layer selects,
//! copies, finds and highlights a scan exactly as it does a typed page, at
//! word granularity: a word's box is shared out evenly between its letters.
//!
//! A document's pages are a [`PdfOcr`], which is what the store keeps in the
//! file's `kansoPdfOcr` field (`kanso_core::model::PDF_OCR_FIELD`) and syncs,
//! so Windows and Linux, which cannot read a scan, still search, select and
//! highlight words a Mac read.
//!
//! Everything here is pure, and defensive: the field arrives from another
//! device, so [`PdfOcr::from_value`] drops what is malformed rather than
//! trusting it, and every size is capped.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::model::Quad;
use crate::text::{PageText, TextChar, TextDoc};

/// Words kept for one page; a page over it keeps the first ones.
pub const MAX_WORDS_PER_PAGE: usize = 4_000;
/// Characters in one word; a longer run is not a word of any language.
pub const MAX_WORD_CHARS: usize = 64;
/// Pages one document keeps words for.
pub const MAX_OCR_PAGES: usize = crate::protocol::MAX_PAGES as usize;
/// The most words kept for one document. The field travels in the file's
/// document (about 25 bytes a word), so it is held well under the size of a
/// sync batch (4 MiB, with sealing on top).
pub const MAX_DOC_WORDS: usize = 60_000;
/// The width in pixels a scanned page is drawn at for reading: 250 dpi on a
/// US letter page, within the design's 200 to 300.
pub const READ_DPI: f32 = 250.0;
/// The longest MD5 or engine name kept from a field that came from
/// elsewhere.
pub const MAX_TAG_CHARS: usize = 64;

/// One word and where it is on the displayed page.
///
/// Wire shape: `["word", x, y, width, height]`, the box in ten-thousandths
/// of the page (whole numbers, which keep the field small), and a sixth
/// number, the way the word reads in clockwise quarter turns (as
/// [`TextChar::turns`]), only when it isn't 0.
#[derive(Clone, Debug, PartialEq)]
pub struct OcrWord {
    pub text: String,
    pub quad: Quad,
    /// Which way the word reads on the displayed page, in clockwise quarter
    /// turns: 0 left to right, 1 top to bottom, 2 right to left (upside
    /// down), 3 bottom to top.
    pub turns: u8,
}

impl OcrWord {
    /// A word, or `None` when there is nothing to keep: no letters, or a box
    /// with no area. The text is trimmed, a run past [`MAX_WORD_CHARS`] is
    /// cut, and control characters and inner white space are dropped (a
    /// word is one unbroken run). It reads left to right.
    pub fn new(text: &str, quad: Quad) -> Option<Self> {
        let text: String = text
            .chars()
            .filter(|c| !c.is_control() && !c.is_whitespace())
            .take(MAX_WORD_CHARS)
            .collect();
        let [x0, y0, x1, y1] = quad.0;
        if text.is_empty() || x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(OcrWord {
            text,
            quad,
            turns: 0,
        })
    }

    /// The same word, reading `turns` clockwise quarter turns (taken modulo
    /// four).
    pub fn turned(mut self, turns: u8) -> Self {
        self.turns = turns % 4;
        self
    }
}

/// A word as a reader hands it over: its text, its box on the picture
/// (top-left origin, 0 to 1) and which way it reads (see
/// [`OcrWord::turns`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ReadWord {
    pub text: String,
    pub quad: Quad,
    pub turns: u8,
}

impl From<(String, Quad)> for ReadWord {
    /// A word that reads left to right.
    fn from((text, quad): (String, Quad)) -> Self {
        ReadWord {
            text,
            quad,
            turns: 0,
        }
    }
}

/// A coordinate in ten-thousandths of the page (a quad's values are already
/// kept to four decimals).
fn ten_thousandths(v: f64) -> i64 {
    (v * 10_000.0).round() as i64
}

impl Serialize for OcrWord {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let [x0, y0, x1, y1] = self.quad.0.map(ten_thousandths);
        // A sliver of a box is still one unit wide, so it reads back.
        let (w, h) = ((x1 - x0).max(1), (y1 - y0).max(1));
        if self.turns.is_multiple_of(4) {
            (&self.text, x0, y0, w, h).serialize(s)
        } else {
            (&self.text, x0, y0, w, h, self.turns % 4).serialize(s)
        }
    }
}

impl<'de> Deserialize<'de> for OcrWord {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Row;
        impl<'de> serde::de::Visitor<'de> for Row {
            type Value = OcrWord;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a word: [text, x, y, width, height] and maybe its turns")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<OcrWord, A::Error> {
                use serde::de::Error as _;
                let missing = || A::Error::custom("a word with too few numbers");
                let text: String = seq.next_element()?.ok_or_else(missing)?;
                let mut n = [0i64; 4];
                for slot in &mut n {
                    *slot = seq.next_element()?.ok_or_else(missing)?;
                }
                let turns = seq
                    .next_element::<i64>()?
                    .map_or(0, |t| t.rem_euclid(4) as u8);
                // Anything after is not part of a word.
                while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                // Whole numbers from anywhere, held small so the sums can't
                // overflow.
                let unit = |v: i64| v.clamp(0, 10_000) as f64 / 10_000.0;
                let [x, y, w, h] = n.map(|v| v.clamp(-10_000, 20_000));
                OcrWord::new(&text, Quad::new(unit(x), unit(y), unit(x + w), unit(y + h)))
                    .map(|word| word.turned(turns))
                    .ok_or_else(|| A::Error::custom("a word with no letters or no area"))
            }
        }
        d.deserialize_seq(Row)
    }
}

/// A page's words as they arrive: the sound ones, up to
/// [`MAX_WORDS_PER_PAGE`]. A malformed word is skipped, not trusted, and
/// what is past the cap is not kept.
fn sound_words<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<OcrWord>, D::Error> {
    struct Words;
    impl<'de> serde::de::Visitor<'de> for Words {
        type Value = Vec<OcrWord>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a list of words")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Vec<OcrWord>, A::Error> {
            let mut words = Vec::new();
            while let Some(word) = seq.next_element::<MaybeWord>()? {
                if let Some(word) = word.0
                    && words.len() < MAX_WORDS_PER_PAGE
                {
                    words.push(word);
                }
            }
            Ok(words)
        }
    }
    d.deserialize_seq(Words)
}

/// One element of a page's words: a word, or `None` for anything else.
struct MaybeWord(Option<OcrWord>);

impl<'de> Deserialize<'de> for MaybeWord {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Through a `Value`, so a malformed word is dropped without failing
        // the page. A word is a handful of small values.
        let value = Value::deserialize(d)?;
        Ok(MaybeWord(OcrWord::deserialize(&value).ok()))
    }
}

/// The words of one scanned page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OcrPage {
    /// 1-based page number.
    pub page: u32,
    /// The displayed page's size in points, as [`PageText`] wants it.
    pub width: f64,
    pub height: f64,
    #[serde(deserialize_with = "sound_words")]
    pub words: Vec<OcrWord>,
}

/// A size in points, kept to a tenth, and never zero.
fn points(v: f64) -> f64 {
    let v = if v.is_finite() { v } else { 0.0 };
    ((v.clamp(1.0, 75_000.0)) * 10.0).round() / 10.0
}

impl OcrPage {
    /// A page of `words` (anything with text and a box), cleaned: words with
    /// no letters or no area are dropped, boxes are clipped to the page, and
    /// only the first [`MAX_WORDS_PER_PAGE`] stay. The order is the
    /// reader's, which is reading order.
    pub fn new(
        page: u32,
        width: f64,
        height: f64,
        words: impl IntoIterator<Item = (String, Quad)>,
    ) -> Self {
        Self::read(page, width, height, words.into_iter().map(ReadWord::from))
    }

    /// As [`OcrPage::new`], from words that say which way they read.
    pub fn read(
        page: u32,
        width: f64,
        height: f64,
        words: impl IntoIterator<Item = ReadWord>,
    ) -> Self {
        OcrPage {
            page,
            width: points(width),
            height: points(height),
            words: words
                .into_iter()
                .filter_map(|ReadWord { text, quad, turns }| {
                    let [x0, y0, x1, y1] = quad.0;
                    OcrWord::new(&text, Quad::new(x0, y0, x1, y1)).map(|w| w.turned(turns))
                })
                .take(MAX_WORDS_PER_PAGE)
                .collect(),
        }
    }

    /// The page as the text layer takes it: each word's letters side by side
    /// across the word's box, along the way it reads (left to right, down
    /// the box for a word turned a quarter, and so on), the first letter
    /// marked as starting a word.
    pub fn to_page_text(&self) -> PageText {
        let mut chars = Vec::new();
        for word in &self.words {
            let letters: Vec<char> = word.text.chars().collect();
            let [x0, y0, x1, y1] = word.quad.0;
            let n = letters.len().max(1) as f64;
            let turns = word.turns % 4;
            // `lo` to `hi`, from `a` to `b` of the way along.
            let along = |lo: f64, hi: f64, a: f64, b: f64| (lo + (hi - lo) * a, lo + (hi - lo) * b);
            for (i, ch) in letters.iter().enumerate() {
                // The letter's share of the box along the reading direction.
                let a = i as f64 / n;
                let b = if i + 1 == letters.len() {
                    1.0
                } else {
                    (i + 1) as f64 / n
                };
                let quad = match turns {
                    0 => {
                        let (l, r) = along(x0, x1, a, b);
                        Quad::new(l, y0, r, y1)
                    }
                    1 => {
                        let (t, d) = along(y0, y1, a, b);
                        Quad::new(x0, t, x1, d)
                    }
                    2 => {
                        let (l, r) = along(x0, x1, 1.0 - b, 1.0 - a);
                        Quad::new(l, y0, r, y1)
                    }
                    _ => {
                        let (t, d) = along(y0, y1, 1.0 - b, 1.0 - a);
                        Quad::new(x0, t, x1, d)
                    }
                };
                chars.push(TextChar {
                    ch: *ch,
                    quad,
                    turns,
                    space_before: i == 0,
                });
            }
        }
        PageText {
            page: self.page,
            width: self.width,
            height: self.height,
            rotation: 0,
            chars,
        }
    }

    /// The page's words as text, lines in reading order (the layout the
    /// reader selects with), for search.
    pub fn text(&self) -> String {
        let page = self.to_page_text();
        let doc = TextDoc::new([&page]);
        doc.select_all()
            .map(|all| doc.text(&all))
            .unwrap_or_default()
    }
}

/// The words read from a document's scanned pages, and what they were read
/// from.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfOcr {
    /// The `md5digest` of the PDF the words were read from.
    pub md5: String,
    /// Who read them (`"vision"`).
    pub engine: String,
    /// When, in milliseconds since the epoch.
    pub at: i64,
    /// Pages in ascending order, one entry for each page that was read
    /// (a scan with no words has an entry with none).
    pub pages: Vec<OcrPage>,
}

impl PdfOcr {
    /// The field's value.
    pub fn to_value(&self) -> Value {
        json!({
            "md5": self.md5,
            "engine": self.engine,
            "at": self.at,
            "pages": self.pages,
        })
    }

    /// Reads the field back, keeping what is sound: a field without its MD5
    /// is no field, a page that is malformed or repeated is dropped, pages
    /// are put in order, and the caps hold.
    pub fn from_value(value: &Value) -> Option<PdfOcr> {
        let obj = value.as_object()?;
        let md5 = obj.get("md5")?.as_str()?;
        // An MD5 is 32 characters; anything much longer is not one.
        if md5.is_empty() || md5.len() > MAX_TAG_CHARS {
            return None;
        }
        let mut pages: Vec<OcrPage> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for page in obj.get("pages")?.as_array()? {
            if pages.len() >= MAX_OCR_PAGES {
                break;
            }
            let Ok(mut page) = OcrPage::deserialize(page) else {
                continue;
            };
            if page.page == 0 || page.page as usize > MAX_OCR_PAGES || !seen.insert(page.page) {
                continue;
            }
            page.width = points(page.width);
            page.height = points(page.height);
            page.words.truncate(MAX_WORDS_PER_PAGE);
            pages.push(page);
        }
        pages.sort_by_key(|p| p.page);
        let mut left = MAX_DOC_WORDS;
        for page in &mut pages {
            page.words.truncate(left);
            left -= page.words.len();
        }
        let tag = |key: &str| -> String {
            obj.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .chars()
                .take(MAX_TAG_CHARS)
                .collect()
        };
        Some(PdfOcr {
            md5: md5.to_owned(),
            engine: tag("engine"),
            at: obj.get("at").and_then(Value::as_i64).unwrap_or_default(),
            pages,
        })
    }

    /// The page `page` (1-based), if it was read.
    pub fn page(&self, page: u32) -> Option<&OcrPage> {
        self.pages
            .binary_search_by_key(&page, |p| p.page)
            .ok()
            .map(|i| &self.pages[i])
    }
}

/// A box from a reader that measures from the bottom-left of the picture
/// (Apple's Vision: `x`, `y`, `width`, `height`, each 0 to 1) as a quad,
/// which measures from the top-left.
pub fn quad_from_bottom_left(x: f64, y: f64, width: f64, height: f64) -> Quad {
    Quad::new(x, 1.0 - (y + height), x + width, 1.0 - y)
}

/// Which way a word reads, in clockwise quarter turns on the displayed
/// page, from the top-left and top-right corners of its box as a reader
/// that measures from the bottom-left gives them (Apple's Vision: the
/// corners of the word's own, possibly turned, rectangle). The top edge
/// runs the way the word reads; it is rounded to the nearest quarter. A
/// box with no length reads left to right.
pub fn turns_from_corners(top_left: (f64, f64), top_right: (f64, f64)) -> u8 {
    let dx = top_right.0 - top_left.0;
    // Downward on the page is upward in the reader's measure.
    let down = top_left.1 - top_right.1;
    if !(dx.is_finite() && down.is_finite()) || (dx == 0.0 && down == 0.0) {
        return 0;
    }
    if dx.abs() >= down.abs() {
        if dx >= 0.0 { 0 } else { 2 }
    } else if down > 0.0 {
        1
    } else {
        3
    }
}

/// The words of a line and where each lies in it, as UTF-16 ranges
/// `(start, length)` (the units Apple's string ranges use), splitting at
/// white space.
pub fn word_ranges(line: &str) -> Vec<(&str, usize, usize)> {
    let mut out = Vec::new();
    let mut at = 0usize;
    let mut start: Option<(usize, usize)> = None;
    for (byte, ch) in line.char_indices() {
        if ch.is_whitespace() {
            if let Some((b, u)) = start.take() {
                out.push((&line[b..byte], u, at - u));
            }
        } else if start.is_none() {
            start = Some((byte, at));
        }
        at += ch.len_utf16();
    }
    if let Some((b, u)) = start {
        out.push((&line[b..], u, at - u));
    }
    out
}

/// The width in pixels to draw a page `width_pt` points wide at for
/// reading: [`READ_DPI`], at least 600 and never more than the worker draws
/// (the worker clamps it again).
pub fn read_width_px(width_pt: f32) -> u32 {
    let px = (width_pt.max(1.0) * READ_DPI / 72.0).round();
    (px as u32).clamp(600, crate::protocol::MAX_LONG_EDGE_PX)
}

#[cfg(test)]
mod tests;
