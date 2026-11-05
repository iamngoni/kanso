use super::*;
use crate::model::Quad;
use proptest::prelude::*;
use std::collections::BTreeMap;

const FILE: &str = "file:abc123";

fn parts(page: u32, words: &str) -> HighlightParts {
    let mut quads = BTreeMap::new();
    quads.insert(page, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
    HighlightParts {
        quads,
        text: words.into(),
    }
}

fn two_pages() -> HighlightParts {
    let mut quads = BTreeMap::new();
    quads.insert(4, vec![Quad::new(0.1, 0.9, 0.9, 0.95)]);
    quads.insert(5, vec![Quad::new(0.1, 0.05, 0.4, 0.1)]);
    HighlightParts {
        quads,
        text: "a sentence that runs\nacross a page break".into(),
    }
}

#[derive(Clone, Debug)]
enum Op {
    Add(u32, String, usize),
    Delete(usize),
    Comment(usize, String),
    Recolour(usize, usize),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (1u32..40, "[a-zA-Z0-9 .,<>*_#|~&-]{1,60}", 0usize..4)
            .prop_map(|(p, t, c)| Op::Add(p, t, c)),
        (0usize..8).prop_map(Op::Delete),
        (0usize..8, "[a-z \n]{0,40}").prop_map(|(i, c)| Op::Comment(i, c)),
        (0usize..8, 0usize..4).prop_map(|(i, c)| Op::Recolour(i, c)),
    ]
}

fn run(ops: &[Op]) -> Annotations {
    let mut a = Annotations::new(FILE, "0f1e");
    for (n, op) in ops.iter().enumerate() {
        let ids: Vec<String> = a.block.highlights.iter().map(|h| h.id.clone()).collect();
        let pick = |i: usize| ids.get(i % ids.len().max(1)).cloned().unwrap_or_default();
        match op {
            Op::Add(p, t, c) => {
                let _ = a.add(parts(*p, t), Color::ALL[*c], n as i64);
            }
            Op::Delete(i) => {
                let _ = a.delete(&pick(*i));
            }
            Op::Comment(i, c) => {
                let _ = a.set_comment(&pick(*i), Some(c));
            }
            Op::Recolour(i, c) => {
                let _ = a.set_color(&pick(*i), Color::ALL[*c]);
            }
        }
    }
    a
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    /// Whatever a burst of work leaves, one edit writes it into the note, the
    /// note reads back to the same highlights, and nothing more is to save.
    #[test]
    fn a_burst_is_one_edit_and_reads_back(
        ops in proptest::collection::vec(op(), 0..24),
        before in "[a-z #\n]{0,80}",
    ) {
        let a = run(&ops);
        let edit = a.save_edit(&before).unwrap();
        let after = match &edit {
            Some(e) => e.apply(&before),
            None => before.clone(),
        };
        let back = Annotations::from_body(&after, FILE, "0f1e");
        prop_assert_eq!(&back.block.highlights, &a.block.highlights);
        prop_assert_eq!(back.save_edit(&after).unwrap(), None);
        // The text around the block is untouched.
        prop_assert!(after.starts_with(before.trim_end()));
    }

    /// A quote goes in once; asking again changes nothing; the list sees it.
    #[test]
    fn a_quote_goes_in_once(text in "[a-zA-Z0-9][a-zA-Z0-9 .,<>*_#|~&!-]{0,79}", body in "[a-z #\n]{0,60}") {
        let mut a = Annotations::new(FILE, "m");
        let id = a.add(parts(2, &text), Color::Green, 1).unwrap();
        let edit = a.quote_edit(&body, &id, "Report.pdf").unwrap();
        let once = edit.apply(&body);
        prop_assert!(quote_ids(&once, FILE).contains(&id));
        prop_assert_eq!(a.quote_edit(&once, &id, "Report.pdf"), None);
        prop_assert!(a.rows(&once).iter().all(|r| r.in_note));
        // Without the quote the flag is down; the highlight stays.
        prop_assert!(a.rows(&body).iter().all(|r| !r.in_note));
    }

    /// Links to another PDF, or with no highlight id, are never this PDF's quotes.
    #[test]
    fn other_pdfs_links_are_not_ours(id in "[a-z0-9]{1,8}", other in "[a-z0-9]{1,8}") {
        prop_assume!(other != "abc123");
        let body = format!("[x](inkdrop://file:{other}#page=1&h={id}) [y](inkdrop://file:abc123#page=2)");
        prop_assert!(quote_ids(&body, FILE).is_empty());
    }
}

#[test]
fn a_cross_page_highlight_keeps_both_pages_and_the_first_is_its_page() {
    let mut a = Annotations::new(FILE, "m");
    let id = a.add(two_pages(), Color::Blue, 7).unwrap();
    let h = a.get(&id).unwrap();
    assert_eq!(h.page, 4);
    assert_eq!(h.quads.keys().copied().collect::<Vec<_>>(), vec![4, 5]);
    assert_eq!(h.text, "a sentence that runs across a page break");
    let body = a.save_edit("").unwrap().unwrap().apply("");
    let back = Annotations::from_body(&body, FILE, "m");
    assert_eq!(back.get(&id), a.get(&id));
    assert!(a.quote(&id).unwrap().contains("#page=4&h="));
}

#[test]
fn an_empty_selection_makes_no_highlight() {
    let mut a = Annotations::new(FILE, "m");
    assert!(a.add(parts(1, "   "), Color::Yellow, 1).is_err());
    let none = HighlightParts {
        quads: BTreeMap::new(),
        text: "words".into(),
    };
    assert!(a.add(none, Color::Yellow, 1).is_err());
    assert!(a.block.highlights.is_empty());
}

#[test]
fn a_full_block_refuses_a_new_highlight_in_plain_words() {
    let mut a = Annotations::new(FILE, "m");
    let long = "x ".repeat(450);
    let mut refused = None;
    for i in 0..4000u32 {
        let page = 1 + i % 90;
        let mut p = parts(page, &long);
        let q = p.quads.get_mut(&page).unwrap();
        for _ in 0..30 {
            q.push(Quad::new(0.123456, 0.123456, 0.654321, 0.234567));
        }
        if let Err(e) = a.add(p, Color::Pink, i as i64) {
            refused = Some(e);
            break;
        }
    }
    assert_eq!(refused, Some(PdfError::BlockFull));
    assert!(a.block.to_json().is_ok(), "the block stayed savable");
}

#[test]
fn a_block_that_left_the_note_is_read_only_until_restored() {
    let mut a = Annotations::new(FILE, "m");
    let id = a.add(parts(3, "Revenue rose"), Color::Yellow, 1).unwrap();
    let body = a.save_edit("Notes").unwrap().unwrap().apply("Notes");
    let mut view = Annotations::from_body(&body, FILE, "m");
    // The note is restored to an older version without the block.
    assert!(view.sync("Notes"));
    assert_eq!(view.status, Status::Restorable);
    assert_eq!(view.block.highlights.len(), 1, "still on screen");
    assert!(view.delete(&id).is_err());
    assert!(view.set_comment(&id, Some("x")).is_err());
    assert!(view.add(parts(1, "more"), Color::Pink, 2).is_err());
    assert!(view.save_edit("Notes").is_err());
    let restored = view.restore_edit("Notes").unwrap().apply("Notes");
    let mut after = view.clone();
    after.sync(&restored);
    assert_eq!(after.status, Status::Editable);
    assert_eq!(after.block, view.block);
}

#[test]
fn a_block_nobody_can_read_is_never_written_over() {
    let broken = "text\n\n```kanso-pdf\n{\"file\": \"file:abc123\", \"version\": 1, \"highlights\": [oops\n```\n";
    let mut a = Annotations::from_body(broken, FILE, "m");
    assert!(matches!(a.status, Status::Unreadable(_)));
    assert!(a.add(parts(1, "x y"), Color::Yellow, 1).is_err());
    assert!(a.save_edit(broken).is_err());
    assert!(a.restore_edit(broken).is_err());
    // A hand-fixed block reads again.
    let fixed = "```kanso-pdf\n{\"file\":\"file:abc123\",\"md5\":\"m\",\"version\":1,\"highlights\":[],\"ink\":[]}\n```";
    assert!(a.sync(fixed));
    assert!(a.is_editable());
}

#[test]
fn a_newer_blocks_version_is_read_only() {
    let newer = "```kanso-pdf\n{\"file\":\"file:abc123\",\"md5\":\"m\",\"version\":9,\"highlights\":[],\"ink\":[]}\n```";
    let a = Annotations::from_body(newer, FILE, "m");
    assert!(matches!(a.status, Status::Unreadable(_)));
}

#[test]
fn deleting_a_quote_keeps_the_highlight_and_the_list_offers_it_back() {
    let mut a = Annotations::new(FILE, "m");
    let id = a
        .add(parts(4, "Revenue rose 12 percent"), Color::Yellow, 1)
        .unwrap();
    let body = a.quote_edit("", &id, "Q3 report.pdf").unwrap().apply("");
    assert!(body.starts_with("## Highlights\n\n### Q3 report.pdf\n\n> Revenue rose 12 percent"));
    assert!(a.rows(&body)[0].in_note);
    let without = "## Highlights\n";
    assert!(!a.rows(without)[0].in_note);
    let again = a
        .quote_edit(without, &id, "Q3 report.pdf")
        .unwrap()
        .apply(without);
    assert!(quote_ids(&again, FILE).contains(&id));
}

#[test]
fn the_list_is_by_page_then_position() {
    let mut a = Annotations::new(FILE, "m");
    let late = a.add(parts(9, "nine"), Color::Yellow, 1).unwrap();
    let mut low = parts(2, "two low");
    low.quads.insert(2, vec![Quad::new(0.1, 0.7, 0.5, 0.8)]);
    let b = a.add(low, Color::Green, 2).unwrap();
    let c = a.add(parts(2, "two high"), Color::Blue, 3).unwrap();
    let order: Vec<String> = a.rows("").into_iter().map(|r| r.id).collect();
    assert_eq!(order, vec![c, b, late]);
}

#[test]
fn a_point_finds_the_newest_highlight_under_it() {
    let mut a = Annotations::new(FILE, "m");
    let first = a.add(parts(1, "one"), Color::Yellow, 1).unwrap();
    let second = a.add(parts(1, "two"), Color::Pink, 2).unwrap();
    assert_ne!(first, second);
    assert_eq!(a.hit(1, 0.3, 0.15).unwrap().id, second);
    assert!(a.hit(1, 0.9, 0.9).is_none());
    assert!(a.hit(2, 0.3, 0.15).is_none());
}
