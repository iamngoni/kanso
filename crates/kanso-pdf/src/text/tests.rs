use proptest::prelude::*;

use super::find::fold_str;
use super::fixtures::*;
use super::{CharPos, PageText, Point, Selection, TextChar, TextDoc};
use crate::model::{MAX_QUADS, MAX_QUOTE_CHARS, Quad};

fn all_text(doc: &TextDoc) -> String {
    doc.text(&doc.select_all().expect("text"))
}

fn doc_of(pages: &[PageText]) -> TextDoc {
    TextDoc::new(pages)
}

fn strip_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// A row of characters at `(x, y)` on an unrotated page.
fn row(text: &str, x: f64, y: f64) -> Vec<Placed> {
    let all: Vec<char> = text.chars().collect();
    all.iter()
        .enumerate()
        .filter(|(_, c)| **c != ' ')
        .map(|(i, &ch)| {
            let x0 = x + i as f64 * CHAR_W;
            Placed {
                ch,
                rect: [x0, y, x0 + CHAR_W, y + CHAR_H],
                space_before: i > 0 && all[i - 1] == ' ',
            }
        })
        .collect()
}

fn words(s: &str) -> Words {
    s.split(' ').map(str::to_owned).collect()
}

// ---- Fixtures ---------------------------------------------------------

#[test]
fn two_column_page_reads_left_column_then_right() {
    let cols = vec![
        vec![
            vec![words("The quick brown"), words("fox jumps over")],
            vec![words("the lazy dog")],
        ],
        vec![vec![words("Pack my box"), words("with five dozen")]],
    ];
    let doc = doc_of(&[page(1, &cols, 0)]);
    assert_eq!(
        all_text(&doc),
        "The quick brown\nfox jumps over\n\nthe lazy dog\n\nPack my box\nwith five dozen"
    );
    // A line is one column's line, not the row across the gutter.
    let p = doc.page(1).unwrap();
    assert_eq!(p.lines.len(), 5);
    assert_eq!(p.blocks.len(), 3);
}

#[test]
fn a_full_width_title_reads_before_both_columns() {
    let cols = vec![
        column_of(&["Left one", "left two"]),
        column_of(&["Right one", "right two"]),
    ];
    let (mut placed, w, h) = place(&cols);
    // 60 characters: wide enough to cross the gutter.
    let title = "A Very Long Title That Reaches Across Both Columns Of Page";
    let mut all = row(title, MARGIN, 10.0);
    all.append(&mut placed);
    let doc = doc_of(&[page_from(1, &all, w, h, 0)]);
    assert_eq!(
        all_text(&doc),
        format!("{title}\n\nLeft one\nleft two\n\nRight one\nright two")
    );
}

#[test]
fn a_short_note_in_the_corner_is_not_a_column() {
    // A date at the top right, then one column of text below it.
    let cols = vec![column_of(&["Body text line one", "line two of body"])];
    let (mut placed, w, h) = place(&cols);
    let mut all = row("12 May", 290.0, 10.0);
    all.append(&mut placed);
    let doc = doc_of(&[page_from(1, &all, w, h, 0)]);
    assert_eq!(
        all_text(&doc),
        "12 May\n\nBody text line one\nline two of body"
    );
}

#[test]
fn rotated_pages_read_in_the_same_order() {
    let cols = vec![
        column_of(&["HELLO WORLD", "second line"]),
        column_of(&["right column"]),
    ];
    let want = "HELLO WORLD\nsecond line\n\nright column";
    for rotation in [0, 90, 180, 270] {
        let doc = doc_of(&[page(1, &cols, rotation)]);
        assert_eq!(all_text(&doc), want, "rotation {rotation}");
    }
}

#[test]
fn the_rotation_fixture_matches_what_a_turned_page_looks_like() {
    // On a page turned 90 degrees clockwise, the top-left of the original
    // becomes the top-right, and its first line reads downwards.
    let placed = row("AB", 20.0, 20.0);
    let p = page_from(1, &placed, 200.0, 100.0, 90);
    assert_eq!((p.width, p.height), (100.0, 200.0));
    let a = p.chars[0].quad.0;
    // x = 100 - v: 100 - 28 .. 100 - 20; y = u: 20 .. 26.
    assert_eq!(a, [0.72, 0.1, 0.8, 0.13]);
    let b = p.chars[1].quad.0;
    assert!(b[1] > a[1], "the second letter is below the first");
}

#[test]
fn the_selection_follows_a_pointer_on_a_rotated_page() {
    let cols = vec![column_of(&["HELLO WORLD", "second line"])];
    let doc = doc_of(&[page(1, &cols, 90)]);
    // Glyph 6 is the W; the pointer on its box picks it.
    let at = centre(&doc, 1, 6);
    let pos = doc.hit(at).unwrap();
    assert_eq!(pos.index, 6);
    let w = doc.word_at(pos).unwrap();
    assert_eq!(doc.text(&w), "WORLD");
    let l = doc.line_at(pos).unwrap();
    assert_eq!(doc.text(&l), "HELLO WORLD");
    // Quads are one band across the whole line, inside the page.
    let q = doc.quads(&l);
    assert_eq!(q[&1].len(), 1);
    assert!(q[&1][0].is_valid());
}

#[test]
fn sideways_text_on_an_upright_page_is_its_own_group() {
    let cols = vec![column_of(&["Main text runs", "across the page"])];
    let (placed, w, h) = place(&cols);
    let mut p = page_from(1, &placed, w, h, 0);
    // "SIDE" runs bottom to top at the left edge; S is the lowest.
    for (i, ch) in "SIDE".chars().enumerate() {
        let y1 = 90.0 - i as f64 * CHAR_W;
        p.chars.push(TextChar {
            ch,
            quad: Quad::new(10.0 / w, (y1 - CHAR_W) / h, 18.0 / w, y1 / h),
            turns: 3,
            space_before: false,
        });
    }
    let doc = doc_of(&[p]);
    assert_eq!(all_text(&doc), "Main text runs\nacross the page\n\nSIDE");
}

#[test]
fn a_page_in_json_has_the_shape_the_worker_sends() {
    let json = r#"{"page":4,"width":612.0,"height":792.0,"rotation":90,
        "chars":[{"ch":"H","quad":[0.1,0.1,0.12,0.13]},
                 {"ch":"i","quad":[0.12,0.1,0.13,0.13],"turns":3}]}"#;
    let p: PageText = serde_json::from_str(json).unwrap();
    assert_eq!(p.rotation, 90);
    assert_eq!(p.chars[0].turns, 0);
    assert_eq!(p.chars[1].turns, 3);
    let back = serde_json::to_string(&p).unwrap();
    assert!(!back.contains("\"turns\":0"));
    assert!(!back.contains("space_before"));
    let spaced = r#"{"ch":"x","quad":[0.1,0.1,0.12,0.13],"space_before":true}"#;
    let c: TextChar = serde_json::from_str(spaced).unwrap();
    assert!(c.space_before);
    assert!(
        serde_json::to_string(&c)
            .unwrap()
            .contains("\"space_before\":true")
    );
    assert_eq!(serde_json::from_str::<PageText>(&back).unwrap(), p);
}

/// A page of `text` set in one row, characters `advance` wide and `height`
/// tall, touching, a word gap of `gap`; `reported` says whether PDFium's
/// spaces are on the characters.
fn row_page(text: &str, advance: f64, height: f64, gap: f64, reported: bool) -> PageText {
    let mut placed = Vec::new();
    let mut x = 40.0;
    let mut space = false;
    for ch in text.chars() {
        if ch == ' ' {
            x += gap;
            space = true;
            continue;
        }
        placed.push(Placed {
            ch,
            rect: [x, 40.0, x + advance, 40.0 + height],
            space_before: reported && std::mem::take(&mut space),
        });
        x += advance;
        space = false;
    }
    page_from(1, &placed, 600.0, 200.0, 0)
}

#[test]
fn words_break_where_pdfium_put_a_space_whatever_the_gap() {
    // A space PDFium reported breaks the word with no gap at all.
    let touching = row_page("ab cd", 6.0, 8.0, 0.0, true);
    assert_eq!(all_text(&doc_of(&[touching])), "ab cd");
    // And once PDFium has said where the spaces are, a gap as wide as a
    // word space does not make one where it did not put it.
    let mut moved = row_page("ab cd", 6.0, 8.0, 5.0, true);
    moved.chars[2].space_before = false;
    moved.chars[1].space_before = true;
    assert_eq!(all_text(&doc_of(&[moved])), "a bcd");
}

#[test]
fn without_reported_spaces_the_fallback_scales_with_the_advance_not_the_height() {
    // The same word gap, 2.5 points, is a space between narrow tall letters
    // and nothing between wide short ones: only the advance tells.
    let narrow = row_page("ab cd", 3.0, 12.0, 2.5, false);
    assert_eq!(all_text(&doc_of(&[narrow])), "ab cd");
    let wide = row_page("ab cd", 12.0, 3.0, 2.5, false);
    assert_eq!(all_text(&doc_of(&[wide])), "abcd");
}

#[test]
fn generated_spaces_and_breaks_and_junk_boxes_are_harmless() {
    let cols = vec![column_of(&["ab cd"])];
    let mut p = page(1, &cols, 0);
    p.chars.push(TextChar {
        ch: ' ',
        quad: Quad([0.0; 4]),
        turns: 0,
        space_before: false,
    });
    p.chars.push(TextChar {
        ch: '\n',
        quad: Quad([0.5; 4]),
        turns: 0,
        space_before: false,
    });
    p.chars.push(TextChar {
        ch: '\u{2}',
        quad: Quad([f64::NAN, 0.0, 1.0, 1.0]),
        turns: 0,
        space_before: false,
    });
    assert_eq!(all_text(&doc_of(&[p])), "ab cd");
    let empty = doc_of(&[PageText {
        page: 1,
        width: 0.0,
        height: 0.0,
        rotation: 0,
        chars: vec![],
    }]);
    assert!(empty.select_all().is_none());
    assert!(
        empty
            .hit(Point {
                page: 1,
                x: 0.5,
                y: 0.5
            })
            .is_none()
    );
}

// ---- Selection --------------------------------------------------------

fn two_column_doc() -> TextDoc {
    let cols = vec![
        column_of(&["alpha beta gamma", "delta epsilon zeta"]),
        column_of(&["eta theta iota", "kappa lambda mu"]),
    ];
    doc_of(&[page(1, &cols, 0)])
}

#[test]
fn drag_inside_a_line_copies_that_part() {
    let doc = two_column_doc();
    // From the b of beta to the a that ends gamma.
    let sel = doc.select(centre(&doc, 1, 5), centre(&doc, 1, 13)).unwrap();
    assert_eq!(doc.text(&sel), "beta gamma");
}

#[test]
fn drag_down_a_column_copies_its_lines() {
    let doc = two_column_doc();
    let sel = doc.select(centre(&doc, 1, 5), centre(&doc, 1, 20)).unwrap();
    assert_eq!(doc.text(&sel), "beta gamma\ndelta ep");
}

#[test]
fn drag_from_the_left_column_to_the_right_follows_reading_order() {
    let doc = two_column_doc();
    let sel = doc
        .select(centre(&doc, 1, 28), centre(&doc, 1, 40))
        .unwrap();
    // Left column's rest, then the right column up to the pointer.
    let t = doc.text(&sel);
    assert_eq!(t, "ta\n\neta theta iot");
}

#[test]
fn a_pointer_in_the_margin_picks_the_nearest_character() {
    let doc = two_column_doc();
    let below = Point {
        page: 1,
        x: 0.05,
        y: 0.99,
    };
    let pos = doc.hit(below).unwrap();
    assert!(doc.page(1).unwrap().glyphs.len() > pos.index);
}

#[test]
fn word_line_and_paragraph_expansion() {
    let cols = vec![vec![
        vec![words("It's a fine-day, isn't it?"), words("Yes indeed")],
        vec![words("Next one")],
    ]];
    let doc = doc_of(&[page(1, &cols, 0)]);
    let at = |i| CharPos { page: 1, index: i };
    // "It's" keeps its apostrophe.
    assert_eq!(doc.text(&doc.word_at(at(1)).unwrap()), "It's");
    // The comma is its own run, not part of "day".
    let t = all_text(&doc);
    let comma = t
        .chars()
        .filter(|c| !c.is_whitespace())
        .position(|c| c == ',')
        .unwrap();
    assert_eq!(doc.text(&doc.word_at(at(comma)).unwrap()), ",");
    assert_eq!(doc.text(&doc.word_at(at(comma - 1)).unwrap()), "day");
    assert_eq!(
        doc.text(&doc.line_at(at(3)).unwrap()),
        "It's a fine-day, isn't it?"
    );
    assert_eq!(
        doc.text(&doc.paragraph_at(at(3)).unwrap()),
        "It's a fine-day, isn't it?\nYes indeed"
    );
}

#[test]
fn selecting_across_a_page_break_joins_with_one_newline() {
    let p1 = page(1, &[column_of(&["first page ends with the"])], 0);
    let p2 = page(2, &[column_of(&["words that continue it"])], 90);
    let doc = doc_of(&[p1, p2]);
    let a = doc.hit(centre(&doc, 1, 13)).unwrap(); // "with"
    let b = doc.hit(centre(&doc, 2, 8)).unwrap(); // up to "that"
    let sel = Selection::between(a, b);
    assert_eq!(doc.text(&sel), "with the\nwords that");
    // Quads come out by page, in each page's own displayed box.
    let q = doc.quads(&sel);
    assert_eq!(q.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
    let parts = doc.highlight_parts(&sel);
    assert_eq!(parts.text, "with the\nwords that");
    assert!(parts.quads.values().flatten().all(Quad::is_valid));
}

#[test]
fn extend_moves_the_nearer_end() {
    let doc = two_column_doc();
    let sel = doc.select(centre(&doc, 1, 3), centre(&doc, 1, 8)).unwrap();
    let later = doc.hit(centre(&doc, 1, 12)).unwrap();
    let longer = sel.extend_to(later);
    assert_eq!(longer.start, sel.start);
    assert_eq!(longer.end, later);
    let inside = doc.hit(centre(&doc, 1, 4)).unwrap();
    assert_eq!(sel.extend_to(inside).start, inside);
}

// ---- Quads ------------------------------------------------------------

#[test]
fn a_one_line_selection_is_one_quad_as_tall_as_the_line() {
    let doc = two_column_doc();
    let sel = doc.select(centre(&doc, 1, 6), centre(&doc, 1, 9)).unwrap();
    let q = &doc.quads(&sel)[&1];
    assert_eq!(q.len(), 1);
    let p = doc.page(1).unwrap();
    let first = p.glyphs[6].quad.0;
    let last = p.glyphs[9].quad.0;
    assert_eq!(q[0].0[0], first[0]);
    assert_eq!(q[0].0[2], last[2]);
    assert_eq!(q[0].0[1], p.lines[0].band.0[1]);
    assert_eq!(q[0].0[3], p.lines[0].band.0[3]);
}

#[test]
fn a_selection_over_many_lines_keeps_to_the_quad_cap() {
    let lines: Vec<Vec<String>> = (0..60).map(|i| vec![format!("line{i}")]).collect();
    let doc = doc_of(&[page(1, &[vec![lines]], 0)]);
    let sel = doc.select_all().unwrap();
    assert_eq!(doc.quads(&sel)[&1].len(), 60);
    let parts = doc.highlight_parts(&sel);
    assert!(parts.quads.values().map(Vec::len).sum::<usize>() <= MAX_QUADS);
    assert!(parts.text.chars().count() <= MAX_QUOTE_CHARS);
    // The merged quads still cover the whole selection.
    let all = &parts.quads[&1];
    assert!(all.iter().all(Quad::is_valid));
    let top = all.iter().map(|q| q.0[1]).fold(1.0, f64::min);
    let bottom = all.iter().map(|q| q.0[3]).fold(0.0, f64::max);
    let p = doc.page(1).unwrap();
    assert!(top <= p.lines[0].band.0[1] && bottom >= p.lines[59].band.0[3]);
}

// ---- Find -------------------------------------------------------------

#[test]
fn find_ignores_case_and_accents() {
    let cols = vec![column_of(&["Café Résumé naïve", "STRASSE ﬁne"])];
    let doc = doc_of(&[page(1, &cols, 0)]);
    let idx = doc.find_index();
    let text = |q: &str| -> Vec<String> { idx.hits(q).iter().map(|s| doc.text(s)).collect() };
    assert_eq!(text("cafe"), vec!["Café"]);
    assert_eq!(text("RESUME"), vec!["Résumé"]);
    // Every e, with or without an accent, in every case.
    assert_eq!(text("é").len(), 6);
    assert_eq!(text("naive"), vec!["naïve"]);
    // A hit may run over a line break and over several spaces.
    assert_eq!(text("naive   strasse"), vec!["naïve\nSTRASSE"]);
    // Ligatures fold to their letters.
    assert_eq!(text("fine"), vec!["ﬁne"]);
    assert!(text("").is_empty());
    assert!(text("   ").is_empty());
    assert!(text("zzz").is_empty());
}

#[test]
fn find_runs_across_a_page_break() {
    let p1 = page(1, &[column_of(&["ends with new"])], 0);
    let p2 = page(2, &[column_of(&["york city"])], 0);
    let doc = doc_of(&[p1, p2]);
    let hits = doc.find_index().hits("new york");
    assert_eq!(hits.len(), 1);
    let q = doc.quads(&hits[0]);
    assert_eq!(q.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn the_seventeenth_hit() {
    let lines: Vec<Vec<String>> = (0..25).map(|i| vec![format!("cat{i}")]).collect();
    let doc = doc_of(&[page(1, &[vec![lines]], 0)]);
    let idx = doc.find_index();
    assert_eq!(idx.count("CAT"), 25);
    let hit = idx.nth("cat", 16).unwrap();
    assert_eq!(doc.text(&hit), "cat");
    // The 17th hit is the word on the 17th line.
    let line = doc.line_at(hit.start).unwrap();
    assert_eq!(doc.text(&line), "cat16");
    assert!(idx.nth("cat", 25).is_none());
}

#[test]
fn hits_do_not_overlap() {
    let doc = doc_of(&[page(1, &[column_of(&["aaaa aaa"])], 0)]);
    let idx = doc.find_index();
    // "aa" fits twice in "aaaa" and once in "aaa" (the space breaks it).
    assert_eq!(idx.count("aa"), 3);
}

// ---- Properties -------------------------------------------------------

fn word_strategy() -> impl Strategy<Value = String> {
    "[a-zA-ZéÉüñ]{1,5}"
}

fn columns_strategy() -> impl Strategy<Value = Vec<Column>> {
    let line = prop::collection::vec(word_strategy(), 1..4);
    let para = prop::collection::vec(line, 1..4);
    let column = prop::collection::vec(para, 1..3);
    prop::collection::vec(column, 1..3)
}

fn shuffled(mut p: PageText, seed: u64) -> PageText {
    fastrand::Rng::with_seed(seed).shuffle(&mut p.chars);
    p
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Selecting all and copying gives every character once, in reading
    /// order, whatever the page's rotation and the order the worker sent.
    #[test]
    fn select_all_copies_every_char_once_in_reading_order(
        cols in columns_strategy(),
        rotation in prop::sample::select(vec![0u16, 90, 180, 270]),
        seed in any::<u64>(),
    ) {
        let doc = doc_of(&[shuffled(page(1, &cols, rotation), seed)]);
        let copied = all_text(&doc);
        prop_assert_eq!(&copied, &expected(&cols));
        let want: String = cols.iter().flatten().flatten().flatten().map(String::as_str).collect();
        prop_assert_eq!(strip_ws(&copied), want);
    }

    /// The same across two pages.
    #[test]
    fn pages_join_with_one_newline(
        a in columns_strategy(),
        b in columns_strategy(),
        ra in prop::sample::select(vec![0u16, 90, 180, 270]),
        rb in prop::sample::select(vec![0u16, 90, 180, 270]),
    ) {
        let doc = doc_of(&[page(2, &b, rb), page(1, &a, ra)]);
        prop_assert_eq!(all_text(&doc), format!("{}\n{}", expected(&a), expected(&b)));
    }

    /// A selection is the same from either end, copies a contiguous run, and
    /// every character in it is copied.
    #[test]
    fn selection_is_symmetric_and_contiguous(
        cols in columns_strategy(),
        rotation in prop::sample::select(vec![0u16, 90, 180, 270]),
        i in any::<prop::sample::Index>(),
        j in any::<prop::sample::Index>(),
    ) {
        let doc = doc_of(&[page(1, &cols, rotation)]);
        let n = doc.page(1).unwrap().glyphs.len();
        let (i, j) = (i.index(n), j.index(n));
        let (a, b) = (centre(&doc, 1, i), centre(&doc, 1, j));
        let ab = doc.select(a, b).unwrap();
        let ba = doc.select(b, a).unwrap();
        prop_assert_eq!(ab, ba);
        prop_assert_eq!((ab.start.index, ab.end.index), (i.min(j), i.max(j)));
        let copied = doc.text(&ab);
        prop_assert!(all_text(&doc).contains(&copied));
        prop_assert_eq!(strip_ws(&copied).chars().count(), i.max(j) - i.min(j) + 1);
        // Quads are on the page and non-empty.
        let quads = doc.quads(&ab);
        prop_assert!(!quads[&1].is_empty());
        prop_assert!(quads[&1].iter().all(Quad::is_valid));
    }

    /// Word, line and paragraph expansions contain the click, nest, and
    /// copy a run with no break inside a word.
    #[test]
    fn expansions_nest(
        cols in columns_strategy(),
        rotation in prop::sample::select(vec![0u16, 90, 180, 270]),
        i in any::<prop::sample::Index>(),
    ) {
        let doc = doc_of(&[page(1, &cols, rotation)]);
        let n = doc.page(1).unwrap().glyphs.len();
        let pos = CharPos { page: 1, index: i.index(n) };
        let w = doc.word_at(pos).unwrap();
        let l = doc.line_at(pos).unwrap();
        let p = doc.paragraph_at(pos).unwrap();
        for s in [w, l, p] {
            prop_assert!(s.contains(pos));
        }
        prop_assert!(l.start <= w.start && w.end <= l.end);
        prop_assert!(p.start <= l.start && l.end <= p.end);
        prop_assert!(!doc.text(&w).contains(char::is_whitespace));
    }

    /// Find's hits are what a plain search of the copied text finds.
    #[test]
    fn find_matches_a_naive_search_of_the_copied_text(
        a in columns_strategy(),
        b in columns_strategy(),
        rotation in prop::sample::select(vec![0u16, 90, 180, 270]),
        query in prop_oneof!["[a-eéE ]{1,3}", "[a-zA-Z]{1,2}"],
    ) {
        let doc = doc_of(&[page(1, &a, rotation), page(2, &b, 0)]);
        let idx = doc.find_index();
        let copied = all_text(&doc);
        let hay = fold_str(&copied);
        let needle = fold_str(&query);
        let naive = if needle.is_empty() { 0 } else { hay.matches(needle.as_str()).count() };
        let hits = idx.hits(&query);
        prop_assert_eq!(hits.len(), naive);
        prop_assert_eq!(idx.count(&query), naive);
        for (k, h) in hits.iter().enumerate() {
            prop_assert_eq!(&fold_str(&doc.text(h)), &needle);
            prop_assert_eq!(idx.nth(&query, k), Some(*h));
            if k > 0 {
                prop_assert!(hits[k - 1].end < h.start);
            }
        }
        prop_assert_eq!(idx.nth(&query, hits.len()), None);
    }

    /// A query cut from the copied text, with its case and accents changed,
    /// is found at least where it was cut from.
    #[test]
    fn a_phrase_from_the_page_is_found(
        cols in columns_strategy(),
        from in any::<prop::sample::Index>(),
        len in 1usize..8,
    ) {
        let doc = doc_of(&[page(1, &cols, 0)]);
        let copied: Vec<char> = all_text(&doc).chars().collect();
        let start = from.index(copied.len());
        let phrase: String = copied[start..(start + len).min(copied.len())].iter().collect();
        let mangled = phrase.to_uppercase().replace('É', "E").replace('Ü', "U").replace('Ñ', "N");
        prop_assume!(!fold_str(&phrase).is_empty());
        prop_assert!(doc.find_index().count(&mangled) >= 1);
    }
}
