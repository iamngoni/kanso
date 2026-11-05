use super::*;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

const SEED: u64 = 0x0043_0008;

fn q(x0: f64, y0: f64, x1: f64, y1: f64) -> Quad {
    Quad::new(x0, y0, x1, y1)
}

/// Lines of words laid out left to right, top to bottom.
fn page_of(lines: &[&str]) -> OcrPage {
    let mut words = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let y = 0.05 + row as f64 * 0.05;
        let mut x = 0.05;
        for w in line.split(' ') {
            let width = 0.02 * w.chars().count() as f64;
            words.push((w.to_owned(), q(x, y, x + width, y + 0.04)));
            x += width + 0.02;
        }
    }
    OcrPage::new(1, 612.0, 792.0, words)
}

#[test]
fn a_scanned_page_reads_back_as_its_lines() {
    let page = page_of(&["Quarterly revenue grew", "by twelve percent"]);
    assert_eq!(page.text(), "Quarterly revenue grew\nby twelve percent");
}

#[test]
fn a_word_is_selectable_and_highlightable_like_typed_text() {
    let page = page_of(&["alpha beta gamma"]);
    let text = page.to_page_text();
    let doc = TextDoc::new([&text]);
    // A point inside "beta".
    let beta = &page.words[1].quad.0;
    let at = crate::text::Point {
        page: 1,
        x: (beta[0] + beta[2]) / 2.0,
        y: (beta[1] + beta[3]) / 2.0,
    };
    let pos = doc.hit(at).expect("a hit");
    let word = doc.word_at(pos).expect("a word");
    assert_eq!(doc.text(&word), "beta");
    let parts = doc.highlight_parts(&word);
    assert_eq!(parts.text, "beta");
    let quad = parts.quads[&1][0].0;
    assert!(quad[0] <= beta[0] + 1e-3 && quad[2] >= beta[2] - 1e-3);
}

#[test]
fn words_with_no_letters_or_no_area_are_dropped() {
    let page = OcrPage::new(
        2,
        612.0,
        792.0,
        vec![
            ("  ".into(), q(0.1, 0.1, 0.2, 0.2)),
            ("flat".into(), q(0.1, 0.1, 0.1, 0.2)),
            ("kept".into(), q(0.1, 0.1, 0.3, 0.2)),
            ("sp\u{0}li t".into(), q(0.4, 0.1, 0.6, 0.2)),
        ],
    );
    let kept: Vec<&str> = page.words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(kept, ["kept", "split"]);
}

#[test]
fn the_field_round_trips_and_is_compact() {
    let ocr = PdfOcr {
        md5: "abc".into(),
        engine: "vision".into(),
        at: 7,
        pages: vec![
            page_of(&["one two"]),
            OcrPage {
                page: 4,
                ..OcrPage::new(4, 595.0, 842.0, vec![])
            },
        ],
    };
    let value = ocr.to_value();
    assert_eq!(value["pages"][0]["words"][0][0], "one");
    assert_eq!(PdfOcr::from_value(&value), Some(ocr));
}

#[test]
fn a_field_from_elsewhere_is_taken_with_care() {
    let value = json!({
        "md5": "m",
        "pages": [
            {"page": 3, "width": 100.0, "height": 200.0, "words": [["ok", 1000, 1000, 1000, 1000], ["bad"], 5]},
            {"page": 3, "width": 1.0, "height": 1.0, "words": []},
            {"page": 0, "width": 1.0, "height": 1.0, "words": []},
            {"page": "x"},
            {"page": 1, "width": 50.0, "height": 1.0, "words": []},
        ],
    });
    // A page with a malformed word is dropped whole (it cannot be
    // trusted), so the first page 3 goes and the repeat of it is the one
    // kept; page 0 and the page without a number go.
    let ocr = PdfOcr::from_value(&value).unwrap();
    let pages: Vec<u32> = ocr.pages.iter().map(|p| p.page).collect();
    assert_eq!(pages, [1, 3]);
    assert!(PdfOcr::from_value(&json!({"pages": []})).is_none());
    assert!(PdfOcr::from_value(&json!({"md5": "", "pages": []})).is_none());
    assert!(PdfOcr::from_value(&json!("text")).is_none());
}

#[test]
fn a_word_is_a_small_row_of_whole_numbers() {
    let word = OcrWord::new("beta", q(0.1, 0.25, 0.4, 0.3)).unwrap();
    let value = serde_json::to_value(&word).unwrap();
    assert_eq!(value, json!(["beta", 1000, 2500, 3000, 500]));
    assert_eq!(serde_json::from_value::<OcrWord>(value).unwrap(), word);
}

#[test]
fn a_big_scan_fits_a_sync_batch() {
    let words: Vec<(String, Quad)> = (0..MAX_DOC_WORDS)
        .map(|i| {
            let x = (i % 80) as f64 / 100.0;
            ("abcdefgh".to_owned(), q(x, 0.1234, x + 0.0789, 0.1634))
        })
        .collect();
    let pages: Vec<OcrPage> = words
        .chunks(MAX_WORDS_PER_PAGE)
        .enumerate()
        .map(|(i, w)| OcrPage::new(i as u32 + 1, 612.0, 792.0, w.to_vec()))
        .collect();
    let ocr = PdfOcr {
        md5: "m".into(),
        engine: "vision".into(),
        at: 1,
        pages,
    };
    let bytes = ocr.to_value().to_string().len();
    assert!(bytes < 2 * 1024 * 1024, "{bytes} bytes");
}

#[test]
fn the_reading_width_follows_the_page() {
    assert_eq!(read_width_px(612.0), 2125);
    assert_eq!(read_width_px(10.0), 600);
    assert_eq!(read_width_px(f32::MAX), crate::protocol::MAX_LONG_EDGE_PX);
}

#[test]
fn words_of_a_line_are_found_by_utf16_range() {
    let line = "  naïve 😀x  ok ";
    let words = word_ranges(line);
    assert_eq!(
        words.iter().map(|w| w.0).collect::<Vec<_>>(),
        ["naïve", "😀x", "ok"]
    );
    let units: Vec<u16> = line.encode_utf16().collect();
    for (text, start, len) in words {
        assert_eq!(
            String::from_utf16(&units[start..start + len]).unwrap(),
            text
        );
    }
}

#[test]
fn vision_boxes_flip_to_the_top_left() {
    let quad = quad_from_bottom_left(0.1, 0.8, 0.3, 0.1);
    assert_eq!(quad.0, [0.1, 0.1, 0.4, 0.2]);
}

fn word_strategy() -> impl Strategy<Value = (String, Quad)> {
    (
        "[a-zA-Zé0-9.,]{1,12}",
        0.0f64..1.0,
        0.0f64..1.0,
        0.001f64..0.2,
        0.001f64..0.05,
    )
        .prop_map(|(t, x, y, w, h)| (t, Quad::new(x, y, x + w, y + h)))
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(128) })]

    /// Whatever the reader says, a page keeps only sound words, and the
    /// field round-trips exactly.
    #[test]
    fn prop_a_page_is_sound_and_round_trips(
        words in prop::collection::vec(word_strategy(), 0..60),
        w in -10.0f64..2000.0,
        h in -10.0f64..2000.0,
    ) {
        let page = OcrPage::new(5, w, h, words);
        prop_assert!(page.width >= 1.0 && page.height >= 1.0);
        for word in &page.words {
            prop_assert!(word.quad.is_valid());
            prop_assert!(!word.text.is_empty() && word.text.chars().all(|c| !c.is_whitespace()));
        }
        let ocr = PdfOcr { md5: "m".into(), engine: "e".into(), at: 1, pages: vec![page] };
        let value = ocr.to_value();
        let back = PdfOcr::from_value(&value).unwrap();
        prop_assert_eq!(back.to_value().to_string(), value.to_string());
        prop_assert_eq!(back, ocr);
    }

    /// Every word is in the text layer: its letters in order, each inside
    /// the word's box, and search's text has every word.
    #[test]
    fn prop_the_text_layer_has_every_word(words in prop::collection::vec(word_strategy(), 1..40)) {
        let page = OcrPage::new(1, 612.0, 792.0, words);
        let text = page.to_page_text();
        let total: usize = page.words.iter().map(|w| w.text.chars().count()).sum();
        prop_assert_eq!(text.chars.len(), total);
        let mut at = 0;
        for word in &page.words {
            let [x0, y0, x1, y1] = word.quad.0;
            for (i, ch) in word.text.chars().enumerate() {
                let c = &text.chars[at];
                prop_assert_eq!(c.ch, ch);
                prop_assert_eq!(c.space_before, i == 0);
                let [a, b, cc, d] = c.quad.0;
                prop_assert!(a >= x0 - 1e-3 && cc <= x1 + 1e-3 && b >= y0 - 1e-3 && d <= y1 + 1e-3);
                at += 1;
            }
        }
    }

    /// Selecting everything copies every letter once.
    #[test]
    fn prop_select_all_copies_every_letter(words in prop::collection::vec(word_strategy(), 1..40)) {
        let page = OcrPage::new(1, 612.0, 792.0, words);
        let letters = |s: &str| {
            let mut v: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
            v.sort_unstable();
            v
        };
        let expected: String = page.words.iter().map(|w| w.text.as_str()).collect();
        prop_assert_eq!(letters(&page.text()), letters(&expected));
    }

    /// Splitting a line into words and reading them back by UTF-16 range
    /// gives the words, whatever is in the line.
    #[test]
    fn prop_word_ranges_round_trip(line in "\\PC{0,80}") {
        let units: Vec<u16> = line.encode_utf16().collect();
        let words = word_ranges(&line);
        let joined: Vec<&str> = line.split_whitespace().collect();
        prop_assert_eq!(words.iter().map(|w| w.0).collect::<Vec<_>>(), joined);
        for (text, start, len) in words {
            prop_assert_eq!(String::from_utf16(&units[start..start + len]).unwrap(), text);
        }
    }

    /// A field with junk in it never panics.
    #[test]
    fn prop_from_value_survives_junk(raw in "\\PC{0,200}", pages in prop::collection::vec(0u32..5000, 0..30)) {
        let value = json!({
            "md5": "m",
            "pages": pages.iter().map(|p| json!({"page": p, "width": raw, "height": 1.0, "words": []})).collect::<Vec<_>>(),
        });
        let _ = PdfOcr::from_value(&value);
        let value = json!({"md5": raw, "pages": pages});
        let _ = PdfOcr::from_value(&value);
    }
}
