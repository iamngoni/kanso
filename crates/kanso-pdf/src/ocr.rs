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

/// One word and where it is on the displayed page.
///
/// Wire shape: `["word", x, y, width, height]`, the box in ten-thousandths
/// of the page (whole numbers, which keep the field small).
#[derive(Clone, Debug, PartialEq)]
pub struct OcrWord {
    pub text: String,
    pub quad: Quad,
}

impl OcrWord {
    /// A word, or `None` when there is nothing to keep: no letters, or a box
    /// with no area. The text is trimmed, a run past [`MAX_WORD_CHARS`] is
    /// cut, and control characters and inner white space are dropped (a
    /// word is one unbroken run).
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
        Some(OcrWord { text, quad })
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
        (&self.text, x0, y0, x1 - x0, y1 - y0).serialize(s)
    }
}

impl<'de> Deserialize<'de> for OcrWord {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (text, x, y, w, h): (String, i64, i64, i64, i64) = Deserialize::deserialize(d)?;
        let unit = |n: i64| n.clamp(0, 10_000) as f64 / 10_000.0;
        OcrWord::new(&text, Quad::new(unit(x), unit(y), unit(x + w), unit(y + h)))
            .ok_or_else(|| serde::de::Error::custom("a word with no letters or no area"))
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
        OcrPage {
            page,
            width: points(width),
            height: points(height),
            words: words
                .into_iter()
                .filter_map(|(text, quad)| {
                    OcrWord::new(&text, Quad::new(quad.0[0], quad.0[1], quad.0[2], quad.0[3]))
                })
                .take(MAX_WORDS_PER_PAGE)
                .collect(),
        }
    }

    /// The page as the text layer takes it: each word's letters side by side
    /// across the word's box, the first letter marked as starting a word.
    pub fn to_page_text(&self) -> PageText {
        let mut chars = Vec::new();
        for word in &self.words {
            let letters: Vec<char> = word.text.chars().collect();
            let [x0, y0, x1, y1] = word.quad.0;
            let n = letters.len().max(1) as f64;
            let step = (x1 - x0) / n;
            for (i, ch) in letters.iter().enumerate() {
                let left = x0 + step * i as f64;
                let right = if i + 1 == letters.len() {
                    x1
                } else {
                    left + step
                };
                chars.push(TextChar {
                    ch: *ch,
                    quad: Quad::new(left, y0, right, y1),
                    turns: 0,
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
        let md5 = obj.get("md5")?.as_str()?.to_owned();
        if md5.is_empty() {
            return None;
        }
        let mut pages: Vec<OcrPage> = Vec::new();
        for page in obj.get("pages")?.as_array()? {
            let Ok(mut page) = serde_json::from_value::<OcrPage>(page.clone()) else {
                continue;
            };
            if page.page == 0 || page.page as usize > MAX_OCR_PAGES {
                continue;
            }
            page.width = points(page.width);
            page.height = points(page.height);
            page.words.truncate(MAX_WORDS_PER_PAGE);
            if pages.iter().any(|p| p.page == page.page) || pages.len() >= MAX_OCR_PAGES {
                continue;
            }
            pages.push(page);
        }
        pages.sort_by_key(|p| p.page);
        let mut left = MAX_DOC_WORDS;
        for page in &mut pages {
            page.words.truncate(left);
            left -= page.words.len();
        }
        Some(PdfOcr {
            md5,
            engine: obj
                .get("engine")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
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
