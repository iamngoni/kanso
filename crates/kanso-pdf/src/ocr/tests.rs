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
    // A malformed word is dropped, not trusted, and its page keeps the
    // sound ones; the repeat of page 3 goes, and so do page 0 and the page
    // without a number.
    let ocr = PdfOcr::from_value(&value).unwrap();
    let pages: Vec<u32> = ocr.pages.iter().map(|p| p.page).collect();
    assert_eq!(pages, [1, 3]);
    let three = ocr.page(3).unwrap();
    assert_eq!(three.width, 100.0);
    let kept: Vec<&str> = three.words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(kept, ["ok"]);
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

/// Numbers from anywhere: huge, negative, fractional or of the wrong kind
/// are held to the page or refused, never trusted and never a panic.
#[test]
fn hostile_numbers_in_a_word_are_held_to_the_page() {
    let value = json!({
        "md5": "m",
        "pages": [{"page": 1, "width": -5.0, "height": 1e300, "words": [
            ["big", i64::MAX, i64::MAX, i64::MAX, i64::MAX],
            ["neg", i64::MIN, i64::MIN, i64::MAX, i64::MAX],
            ["frac", 0.5, 1, 2, 3],
            ["text", "1", 2, 3, 4],
            ["short", 1, 2],
            ["turned", 100, 100, 200, 900, -7],
            ["extra", 100, 100, 200, 300, 0, "more", [1, 2]],
        ]}],
    });
    let ocr = PdfOcr::from_value(&value).unwrap();
    let page = ocr.page(1).unwrap();
    assert!(page.width >= 1.0 && page.height <= 75_000.0);
    let kept: Vec<(&str, u8)> = page
        .words
        .iter()
        .map(|w| (w.text.as_str(), w.turns))
        .collect();
    // "big" starts past the page's edge (no area once clipped); "neg" is
    // clipped to the whole page; -7 turns is 1 turn.
    assert_eq!(kept, [("neg", 0), ("turned", 1), ("extra", 0)]);
    for word in &page.words {
        assert!(word.quad.is_valid(), "{word:?}");
    }
}

/// A field with far more pages and words than may be kept keeps the caps.
#[test]
fn a_field_with_absurd_counts_is_capped() {
    let word = json!(["w", 1, 1, 10, 10]);
    let page =
        |n: usize| json!({"page": n, "width": 1.0, "height": 1.0, "words": vec![word.clone(); 50]});
    let pages: Vec<Value> = (1..=MAX_OCR_PAGES + 500).map(page).collect();
    let ocr = PdfOcr::from_value(&json!({"md5": "m", "pages": pages})).unwrap();
    assert_eq!(ocr.pages.len(), MAX_OCR_PAGES);
    let words: usize = ocr.pages.iter().map(|p| p.words.len()).sum();
    assert_eq!(words, MAX_DOC_WORDS.min(MAX_OCR_PAGES * 50));
    // Every page the same number: one kept.
    let same: Vec<Value> = (0..20_000).map(|_| page(7)).collect();
    let ocr = PdfOcr::from_value(&json!({"md5": "m", "pages": same})).unwrap();
    assert_eq!(ocr.pages.len(), 1);
    // One page over its cap keeps the first words.
    let crowded = json!({"md5": "m", "pages": [{"page": 1, "width": 1.0, "height": 1.0,
        "words": vec![word.clone(); MAX_WORDS_PER_PAGE + 100]}]});
    let ocr = PdfOcr::from_value(&crowded).unwrap();
    assert_eq!(ocr.pages[0].words.len(), MAX_WORDS_PER_PAGE);
    // Tags past their length are cut; an MD5 that long is no MD5.
    let long = "x".repeat(10_000);
    let ocr = PdfOcr::from_value(&json!({"md5": "m", "engine": long, "pages": []})).unwrap();
    assert_eq!(ocr.engine.chars().count(), MAX_TAG_CHARS);
    assert!(PdfOcr::from_value(&json!({"md5": long, "pages": []})).is_none());
}

/// A box too thin to measure in ten-thousandths still reads back.
#[test]
fn a_sliver_of_a_word_reads_back() {
    let word = OcrWord::new("i", Quad([0.5, 0.5, 0.50001, 0.52])).unwrap();
    let value = serde_json::to_value(&word).unwrap();
    assert_eq!(value, json!(["i", 5000, 5000, 1, 200]));
    let back: OcrWord = serde_json::from_value(value).unwrap();
    assert_eq!(back.text, "i");
}

/// A word that reads down the page is split down its box, top first, and
/// one that reads up is split from the bottom; the text layer keeps the
/// way each runs, so selecting it takes its letters in order.
#[test]
fn a_turned_word_is_split_along_the_way_it_reads() {
    let down = ReadWord {
        text: "DOWN".into(),
        quad: q(0.10, 0.10, 0.14, 0.50),
        turns: 1,
    };
    let up = ReadWord {
        text: "UP".into(),
        quad: q(0.80, 0.20, 0.84, 0.60),
        turns: 3,
    };
    let back = ReadWord {
        text: "AB".into(),
        quad: q(0.30, 0.80, 0.50, 0.84),
        turns: 2,
    };
    let page = OcrPage::read(1, 612.0, 792.0, [down, up, back]);
    let text = page.to_page_text();
    let letters: Vec<(char, u8, [f64; 4])> = text
        .chars
        .iter()
        .map(|c| (c.ch, c.turns, c.quad.0))
        .collect();
    let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
    // D is at the top of its box, N at the bottom.
    assert_eq!((letters[0].0, letters[0].1), ('D', 1));
    assert!(near(letters[0].2[1], 0.10) && near(letters[0].2[3], 0.20));
    assert!(near(letters[3].2[3], 0.50));
    // U is at the bottom of its box.
    assert_eq!((letters[4].0, letters[4].1), ('U', 3));
    assert!(near(letters[4].2[3], 0.60) && near(letters[4].2[1], 0.40));
    // Upside down: A is on the right.
    assert_eq!((letters[6].0, letters[6].1), ('A', 2));
    assert!(near(letters[6].2[2], 0.50));
    // Selecting everything copies each word in its reading order.
    let doc = TextDoc::new([&text]);
    let all = doc.select_all().unwrap();
    let copied = doc.text(&all);
    for word in ["DOWN", "UP", "AB"] {
        assert!(copied.contains(word), "{word} in {copied:?}");
    }
    // The field keeps the turns, and only when they aren't 0.
    let value = serde_json::to_value(&page.words[0]).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 6);
    let ocr = PdfOcr {
        md5: "m".into(),
        engine: "e".into(),
        at: 0,
        pages: vec![page],
    };
    assert_eq!(PdfOcr::from_value(&ocr.to_value()), Some(ocr));
}

#[test]
fn corners_say_which_way_a_word_reads() {
    // Vision measures from the bottom-left: y grows upward.
    assert_eq!(turns_from_corners((0.1, 0.9), (0.5, 0.9)), 0);
    assert_eq!(turns_from_corners((0.1, 0.9), (0.1, 0.5)), 1);
    assert_eq!(turns_from_corners((0.5, 0.1), (0.1, 0.1)), 2);
    assert_eq!(turns_from_corners((0.1, 0.1), (0.1, 0.5)), 3);
    // Slanted a little: still left to right.
    assert_eq!(turns_from_corners((0.1, 0.5), (0.5, 0.45)), 0);
    assert_eq!(turns_from_corners((0.1, 0.5), (0.1, 0.5)), 0);
    assert_eq!(turns_from_corners((f64::NAN, 0.5), (0.1, 0.5)), 0);
}

/// A JSON number of any kind: whole, huge, negative, fractional.
fn any_number() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<i64>().prop_map(Value::from),
        (-20_000i64..30_000).prop_map(Value::from),
        any::<f64>()
            .prop_filter("JSON has no NaN", |f| f.is_finite())
            .prop_map(Value::from),
        Just(Value::from(u64::MAX)),
    ]
}

/// Anything at all, a little deep.
fn any_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::from),
        any_number(),
        "\\PC{0,12}".prop_map(Value::from),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..5).prop_map(Value::from),
            prop::collection::btree_map("[a-z]{1,6}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// A word, mostly shaped like one.
fn hostile_word() -> impl Strategy<Value = Value> {
    prop_oneof![
        8 => ("\\PC{0,80}", prop::collection::vec(any_number(), 0..8)).prop_map(|(t, n)| {
            let mut v = vec![Value::from(t)];
            v.extend(n);
            Value::from(v)
        }),
        1 => any_json(),
    ]
}

/// A page, mostly shaped like one.
fn hostile_page() -> impl Strategy<Value = Value> {
    prop_oneof![
        8 => (
            any_number(),
            any_number(),
            any_number(),
            prop::collection::vec(hostile_word(), 0..12)
        )
            .prop_map(|(page, w, h, words)| {
                json!({"page": page, "width": w, "height": h, "words": words})
            }),
        1 => (0u32..4).prop_map(|p| json!({"page": p, "width": 1.0, "height": 1.0, "words": "x"})),
        1 => any_json(),
    ]
}

/// A whole field, mostly shaped like one.
fn hostile_field() -> impl Strategy<Value = Value> {
    prop_oneof![
        8 => (
            prop_oneof![
                Just(Value::from("m")),
                any_json(),
                "\\PC{0,80}".prop_map(Value::from)
            ],
            any_json(),
            prop::collection::vec(hostile_page(), 0..12)
        )
            .prop_map(|(md5, engine, pages)| {
                json!({"md5": md5, "engine": engine, "at": 1, "pages": pages})
            }),
        1 => any_json(),
    ]
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

    /// Any field at all (any JSON a device or the sync server could send in
    /// its place) is refused or read within the caps: every page numbered
    /// within the document, in order and once, every word sound, never a
    /// panic.
    #[test]
    fn prop_any_field_is_refused_or_capped(value in hostile_field()) {
        if let Some(ocr) = PdfOcr::from_value(&value) {
            prop_assert!(!ocr.md5.is_empty() && ocr.md5.len() <= MAX_TAG_CHARS);
            prop_assert!(ocr.engine.chars().count() <= MAX_TAG_CHARS);
            prop_assert!(ocr.pages.len() <= MAX_OCR_PAGES);
            prop_assert!(ocr.pages.windows(2).all(|w| w[0].page < w[1].page));
            let mut words = 0;
            for page in &ocr.pages {
                prop_assert!(page.page >= 1 && page.page as usize <= MAX_OCR_PAGES);
                prop_assert!((1.0..=75_000.0).contains(&page.width));
                prop_assert!((1.0..=75_000.0).contains(&page.height));
                prop_assert!(page.words.len() <= MAX_WORDS_PER_PAGE);
                for word in &page.words {
                    prop_assert!(word.quad.is_valid());
                    prop_assert!(word.turns < 4);
                    prop_assert!(!word.text.is_empty());
                    prop_assert!(word.text.chars().count() <= MAX_WORD_CHARS);
                }
                words += page.words.len();
                // The text layer takes any page that was kept.
                let _ = page.to_page_text();
            }
            prop_assert!(words <= MAX_DOC_WORDS);
            // What was kept writes and reads back as itself.
            prop_assert_eq!(PdfOcr::from_value(&ocr.to_value()), Some(ocr));
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

/// Lines of `words` laid out apart on page 1, as a reader gives them.
fn laid_out(lines: &[Vec<String>]) -> OcrPage {
    let mut words = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let y = 0.05 + row as f64 * 0.06;
        let mut x = 0.02;
        for w in line {
            let width = 0.012 * w.chars().count() as f64;
            words.push((w.clone(), q(x, y, x + width, y + 0.04)));
            x += width + 0.015;
        }
    }
    OcrPage::new(1, 612.0, 792.0, words)
}

/// A highlight on the word under the middle of `word`'s box, made with the
/// reader's own selection and the highlights code, as for typed text.
fn highlight_word(
    a: &mut crate::Annotations,
    page: &OcrPage,
    word: usize,
    color: crate::Color,
) -> String {
    let text = page.to_page_text();
    let doc = TextDoc::new([&text]);
    let [x0, y0, x1, y1] = page.words[word].quad.0;
    let at = crate::text::Point {
        page: 1,
        x: (x0 + x1) / 2.0,
        y: (y0 + y1) / 2.0,
    };
    let range = doc.word_at(doc.hit(at).expect("a hit")).expect("a word");
    a.add(doc.highlight_parts(&range), color, 1).expect("added")
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(64) })]

    /// A highlight on a scanned word goes through the same block, merge and
    /// hit code as one on typed text: it saves into the note, reads back,
    /// is found under the word, and survives a merge with another device's
    /// highlight made on the words that device got by sync.
    #[test]
    fn prop_a_highlight_on_a_scanned_word_round_trips(
        lines in prop::collection::vec(prop::collection::vec("[A-Za-z0-9]{1,9}", 1..6), 1..5),
        pick in any::<prop::sample::Index>(),
        other in any::<prop::sample::Index>(),
    ) {
        let page = laid_out(&lines);
        let n = page.words.len();
        let (pick, other) = (pick.index(n), other.index(n));
        let word = page.words[pick].text.clone();

        // The Mac that read the scan highlights a word.
        let mut mac = crate::Annotations::new("file:scan", "md5");
        let id = highlight_word(&mut mac, &page, pick, crate::Color::Yellow);
        let body = "Notes\n".to_owned();
        let body = mac.save_edit(&body).unwrap().expect("an edit").apply(&body);
        let back = crate::Annotations::from_body(&body, "file:scan", "md5");
        prop_assert!(back.is_editable());
        let h = back.get(&id).expect("read back");
        prop_assert_eq!(&h.text, &word);
        prop_assert_eq!(h.page, 1);
        prop_assert_eq!(h, mac.get(&id).unwrap());
        let [x0, y0, x1, y1] = page.words[pick].quad.0;
        prop_assert_eq!(back.hit(1, (x0 + x1) / 2.0, (y0 + y1) / 2.0).map(|h| h.id.as_str()), Some(id.as_str()));
        prop_assert_eq!(back.save_edit(&body).unwrap(), None);

        // Another device has only the synced words, and highlights there.
        let synced = PdfOcr { md5: "md5".into(), engine: "vision".into(), at: 1, pages: vec![page.clone()] };
        let synced = PdfOcr::from_value(&synced.to_value()).unwrap();
        let elsewhere_page = synced.page(1).unwrap().clone();
        prop_assert_eq!(elsewhere_page.to_page_text().chars.len(), page.to_page_text().chars.len());
        let mut pc = crate::Annotations::new("file:scan", "md5");
        let theirs = highlight_word(&mut pc, &elsewhere_page, other, crate::Color::Blue);
        prop_assert_eq!(&pc.get(&theirs).unwrap().text, &page.words[other].text);
        let their_body = pc.save_edit("Notes\n").unwrap().unwrap().apply("Notes\n");

        // Sync keeps both sides' highlights.
        let merged = crate::merge::merge_bodies(&body, &their_body).expect("a merge");
        let both = crate::Annotations::from_body(&merged, "file:scan", "md5");
        prop_assert_eq!(both.get(&id), mac.get(&id));
        prop_assert_eq!(both.get(&theirs), pc.get(&theirs));
    }
}
