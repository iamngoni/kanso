use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed, TestRunner};

use super::*;
use crate::highlights::{Annotations, Located, Status, locate};
use crate::model::{Color, Quad};
use crate::text::HighlightParts;

const FILE: &str = "file:abc123";

/// The tests here run the same cases on every machine.
fn runner(cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(0x6b61_6e73_6f70_6466),
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    })
}

fn parts(page: u32, words: &str) -> HighlightParts {
    let mut quads = BTreeMap::new();
    quads.insert(page, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
    HighlightParts {
        quads,
        text: words.into(),
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
        (1u32..9, "[a-z]{1,12}", 0usize..4).prop_map(|(p, t, c)| Op::Add(p, t, c)),
        (0usize..8).prop_map(Op::Delete),
        (0usize..8, "[a-z]{1,12}").prop_map(|(i, c)| Op::Comment(i, c)),
        (0usize..8, 0usize..4).prop_map(|(i, c)| Op::Recolour(i, c)),
    ]
}

/// What one device did, for checking the merge against.
#[derive(Default, Debug)]
struct Did {
    added: BTreeSet<String>,
    deleted: BTreeSet<String>,
    changed: BTreeSet<String>,
}

fn work(a: &mut Annotations, ops: &[Op], clock: i64) -> Did {
    let mut did = Did::default();
    for (n, op) in ops.iter().enumerate() {
        let ids: Vec<String> = a.block.highlights.iter().map(|h| h.id.clone()).collect();
        let pick = |i: usize| ids.get(i % ids.len().max(1)).cloned();
        match op {
            Op::Add(p, t, c) => {
                let id = a
                    .add(parts(*p, t), Color::ALL[*c], clock + n as i64)
                    .unwrap();
                did.added.insert(id);
            }
            Op::Delete(i) => {
                if let Some(id) = pick(*i) {
                    a.delete(&id).unwrap();
                    did.deleted.insert(id);
                }
            }
            Op::Comment(i, c) => {
                if let Some(id) = pick(*i) {
                    a.set_comment(&id, Some(c)).unwrap();
                    did.changed.insert(id);
                }
            }
            Op::Recolour(i, c) => {
                if let Some(id) = pick(*i) {
                    let before = a.get(&id).unwrap().color;
                    a.set_color(&id, Color::ALL[*c]).unwrap();
                    if before != Color::ALL[*c] {
                        did.changed.insert(id);
                    }
                }
            }
        }
    }
    did
}

fn block_of(body: &str) -> PdfBlock {
    match locate(body, FILE) {
        Located::Found(b) => b,
        other => panic!("no block: {other:?} in {body}"),
    }
}

fn saved(a: &Annotations, body: &str) -> String {
    match a.save_edit(body).unwrap() {
        Some(e) => e.apply(body),
        None => body.to_owned(),
    }
}

fn ids(b: &PdfBlock) -> BTreeSet<String> {
    b.highlights.iter().map(|h| h.id.clone()).collect()
}

/// Two devices start from the same note and each works on the PDF's
/// highlights (and its text) while apart. Whichever version sync lets win,
/// the note ends with the same highlights: everything either added, nothing
/// either deleted, a highlight changed on one side as that side left it.
/// Merging again changes nothing. Pinned seed.
#[test]
fn two_devices_highlighting_at_once_converge_with_no_highlight_lost() {
    let strategy = (
        proptest::collection::vec(op(), 0..10),
        proptest::collection::vec(op(), 0..12),
        proptest::collection::vec(op(), 0..12),
    );
    runner(256)
        .run(&strategy, |(base_ops, ops_a, ops_b)| {
            let mut base = Annotations::new(FILE, "m");
            work(&mut base, &base_ops, 0);
            let base_body = saved(&base, "# Notes\n\nfirst line\n");
            let start = Annotations::from_body(&base_body, FILE, "m");

            let mut a = start.clone();
            let did_a = work(&mut a, &ops_a, 1_000);
            let body_a = saved(&a, &format!("{base_body}\nwritten on the Mac\n"));
            let mut b = start.clone();
            let did_b = work(&mut b, &ops_b, 2_000);
            let body_b = saved(&b, &format!("{base_body}\nwritten on the phone\n"));

            let a_wins = merge_bodies(&body_a, &body_b).unwrap_or_else(|| body_a.clone());
            let b_wins = merge_bodies(&body_b, &body_a).unwrap_or_else(|| body_b.clone());
            let (one, two) = (block_of(&a_wins), block_of(&b_wins));
            prop_assert_eq!(&one, &two, "the order sync met them in decided");
            // The text is the winner's.
            prop_assert!(a_wins.contains("written on the Mac") && !a_wins.contains("phone"));

            let deleted: BTreeSet<String> = did_a.deleted.union(&did_b.deleted).cloned().collect();
            let started = ids(&start.block);
            let want: BTreeSet<String> = started
                .union(&did_a.added)
                .chain(did_b.added.iter())
                .filter(|id| !deleted.contains(*id))
                .cloned()
                .collect();
            prop_assert_eq!(ids(&one), want, "a highlight was lost or came back");
            for id in &deleted {
                prop_assert!(
                    one.removed.contains(id),
                    "{} is not remembered as deleted",
                    id
                );
            }
            // A change made on one side only is that side's.
            for (did, side, other) in [(&did_a, &a, &did_b), (&did_b, &b, &did_a)] {
                for id in did.changed.difference(&other.changed) {
                    if deleted.contains(id) {
                        continue;
                    }
                    let mine = side.get(id).unwrap();
                    let got = one.highlights.iter().find(|h| &h.id == id).unwrap();
                    prop_assert_eq!(&got.comment, &mine.comment);
                    prop_assert_eq!(got.color, mine.color);
                }
            }
            // Merging again, either way, changes nothing.
            prop_assert_eq!(merge_bodies(&a_wins, &body_b), None);
            prop_assert_eq!(merge_bodies(&a_wins, &body_a), None);
            prop_assert_eq!(merge_bodies(&a_wins, &a_wins), None);
            // The merged note reads as one editable block and saves as is.
            let after = Annotations::from_body(&a_wins, FILE, "m");
            prop_assert_eq!(&after.status, &Status::Editable);
            prop_assert_eq!(after.save_edit(&a_wins).unwrap(), None);
            Ok(())
        })
        .unwrap();
}

/// The reader saves after a pause; meanwhile another editor of the note
/// (or the phone) added highlights to the block. The save keeps them.
#[test]
fn a_save_keeps_highlights_another_editor_added_meanwhile() {
    let strategy = (
        proptest::collection::vec(op(), 0..10),
        proptest::collection::vec(op(), 1..10),
        proptest::collection::vec(op(), 1..10),
    );
    runner(256)
        .run(&strategy, |(base_ops, mine, theirs)| {
            let mut base = Annotations::new(FILE, "m");
            work(&mut base, &base_ops, 0);
            let body = saved(&base, "Notes");
            let mut reader = Annotations::from_body(&body, FILE, "m");
            let did_mine = work(&mut reader, &mine, 1_000);
            let mut other = Annotations::from_body(&body, FILE, "m");
            let did_theirs = work(&mut other, &theirs, 2_000);
            let now = saved(&other, &body);
            // The reader's save lands on the body the other editor left.
            let after = saved(&reader, &now);
            let got = ids(&block_of(&after));
            let deleted: BTreeSet<String> = did_mine
                .deleted
                .union(&did_theirs.deleted)
                .cloned()
                .collect();
            for id in did_mine.added.iter().chain(&did_theirs.added) {
                prop_assert_eq!(got.contains(id), !deleted.contains(id), "{}", id);
            }
            prop_assert!(got.is_disjoint(&deleted));
            prop_assert!(after.starts_with("Notes"));
            prop_assert_eq!(find_blocks(&after).len(), 1);
            Ok(())
        })
        .unwrap();
}

/// What a body may hold that this build must never write over: broken JSON
/// that names the PDF, a block from a newer Kanso, a block that breaks the
/// rules, an unclosed fence.
fn unreadable() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(format!(
            "```kanso-pdf\n{{\"file\": \"{FILE}\", \"version\": 1, \"highlights\": [oops\n```"
        )),
        (2u32..50).prop_map(|v| format!(
            "```kanso-pdf\n{{\"file\":\"{FILE}\",\"md5\":\"m\",\"version\":{v},\"highlights\":[{{\"id\":\"new\",\"shape\":\"circle\"}}],\"ink\":[]}}\n```"
        )),
        Just(format!(
            "```kanso-pdf\n{{\"file\":\"{FILE}\",\"md5\":\"m\",\"version\":1,\"highlights\":[{{\"id\":\"aa\",\"page\":0,\"quads\":{{}},\"text\":\"x\"}}],\"ink\":[]}}\n```"
        )),
        // Unclosed: the text after it is inside it.
        Just(format!(
            "```kanso-pdf\n{{\"file\":\"{FILE}\",\"md5\":\"m\",\"version\":1,\"highlights\":[],\"ink\":[]}}\n\nmore words\n"
        )),
    ]
}

/// A block this build can't read is never written over: not by a save, a
/// restore, a quote, a sync merge with any other version, nor by a second
/// readable block for the same PDF beside it. Its bytes stay as they are,
/// and every change to the highlights is refused. Pinned seed.
#[test]
fn an_unreadable_or_newer_block_is_never_written_over() {
    let strategy = (
        unreadable(),
        proptest::collection::vec(op(), 0..10),
        "[a-z \n]{0,40}",
        any::<bool>(),
    );
    runner(256)
        .run(&strategy, |(bad, ops, text, with_good)| {
            // A readable block for the same PDF, made elsewhere.
            let mut good = Annotations::new(FILE, "m");
            work(&mut good, &ops, 0);
            let good_body = saved(&good, "x");
            let body = if with_good {
                format!("{text}\n\n{good_body}\n\n{bad}")
            } else {
                format!("{text}\n\n{bad}")
            };
            let kept = |out: &str| out.contains(bad.trim_end());

            let mut view = Annotations::from_body(&body, FILE, "m");
            prop_assert!(matches!(view.status, Status::Unreadable(_)));
            prop_assert!(view.save_edit(&body).is_err());
            prop_assert!(view.restore_edit(&body).is_err());
            prop_assert!(view.add(parts(1, "words"), Color::Yellow, 1).is_err());
            // Writing the good one's highlights in from another view of them.
            prop_assert!(good.save_edit(&body).is_err());
            prop_assert!(good.restore_edit(&body).is_err());
            if let Some(id) = good.block.highlights.first().map(|h| h.id.clone()) {
                // A quote goes under the heading, never into the block.
                if let Some(edit) = good.quote_edit(&body, &id, "R.pdf") {
                    prop_assert!(kept(&edit.apply(&body)));
                }
            }
            // Sync, whichever side wins.
            if let Some(out) = merge_bodies(&body, &good_body) {
                prop_assert!(kept(&out), "the merge wrote over it: {out}");
            }
            prop_assert_eq!(
                merge_bodies(&good_body, &body),
                None,
                "a broken side was read"
            );
            Ok(())
        })
        .unwrap();
}

/// A block that left the note (an older version was restored) is written
/// back by Restore, with the highlights as they were, and a second Restore
/// changes nothing. Restoring over a block that came back meanwhile keeps
/// both sets.
#[test]
fn restoring_a_block_that_left_the_note() {
    let mut a = Annotations::new(FILE, "m");
    let one = a.add(parts(3, "Revenue rose"), Color::Yellow, 1).unwrap();
    a.set_comment(&one, Some("check")).unwrap();
    let body = saved(&a, "# Notes\n\ntext\n");
    let mut view = Annotations::from_body(&body, FILE, "m");
    let older = "# Notes\n\ntext\n";
    assert!(view.sync(older));
    assert_eq!(view.status, Status::Restorable);
    let back = view.restore_edit(older).unwrap().unwrap().apply(older);
    assert!(back.starts_with("# Notes\n\ntext\n\n```kanso-pdf"));
    assert_eq!(block_of(&back), a.block);
    assert!(view.sync(&back));
    assert_eq!(view.status, Status::Editable);
    assert_eq!(view.restore_edit(&back).unwrap(), None);

    // Meanwhile another device started a new block for the same PDF.
    let mut fresh = Annotations::new(FILE, "m");
    let two = fresh.add(parts(5, "Costs fell"), Color::Green, 9).unwrap();
    let now = saved(&fresh, older);
    let mut view = Annotations::from_body(&body, FILE, "m");
    view.sync(older);
    let both = view.restore_edit(&now).unwrap().unwrap().apply(&now);
    assert_eq!(ids(&block_of(&both)), [one, two].into_iter().collect());
    assert_eq!(find_blocks(&both).len(), 1);
}

/// Two editors of a shared note each wrote the whole block: the text holds
/// two blocks for one PDF. They read as one, and the next save joins them.
#[test]
fn two_blocks_for_one_pdf_read_as_one_and_are_joined_on_save() {
    let mut a = Annotations::new(FILE, "m");
    let x = a.add(parts(1, "one"), Color::Yellow, 1).unwrap();
    let mut b = Annotations::new(FILE, "m");
    let y = b.add(parts(2, "two"), Color::Pink, 2).unwrap();
    let body = format!(
        "Notes\n\n{}\n\nmiddle\n\n{}\n",
        a.block.to_fence().unwrap(),
        b.block.to_fence().unwrap()
    );
    let mut view = Annotations::from_body(&body, FILE, "m");
    assert_eq!(
        ids(&view.block),
        [x.clone(), y.clone()].into_iter().collect()
    );
    view.set_color(&x, Color::Blue).unwrap();
    let after = saved(&view, &body);
    assert_eq!(find_blocks(&after).len(), 1, "{after}");
    assert!(after.contains("middle"));
    let got = block_of(&after);
    assert_eq!(ids(&got), [x.clone(), y].into_iter().collect());
    assert_eq!(
        got.highlights.iter().find(|h| h.id == x).unwrap().color,
        Color::Blue
    );
}

/// The block's merge is commutative, associative and idempotent.
#[test]
fn the_block_merge_is_a_join() {
    let strategy = (
        proptest::collection::vec(op(), 0..8),
        proptest::collection::vec(op(), 0..8),
        proptest::collection::vec(op(), 0..8),
        proptest::collection::vec(op(), 0..8),
    );
    runner(256)
        .run(&strategy, |(base, x, y, z)| {
            let mut start = Annotations::new(FILE, "m");
            work(&mut start, &base, 0);
            let side = |ops: &[Op], clock| {
                let mut a = start.clone();
                work(&mut a, ops, clock);
                a.block
            };
            let (a, b, c) = (side(&x, 100), side(&y, 200), side(&z, 300));
            prop_assert_eq!(merge_blocks(&a, &b), merge_blocks(&b, &a));
            prop_assert_eq!(
                merge_blocks(&merge_blocks(&a, &b), &c),
                merge_blocks(&a, &merge_blocks(&b, &c))
            );
            let ab = merge_blocks(&a, &b);
            prop_assert_eq!(merge_blocks(&ab, &ab), ab.clone());
            prop_assert_eq!(merge_blocks(&ab, &a), ab);
            Ok(())
        })
        .unwrap();
}

#[test]
fn the_edit_between_two_texts_is_the_part_that_differs() {
    for (old, new) in [
        ("abc", "abc"),
        ("abc", "abxc"),
        ("", "x"),
        ("x", ""),
        ("aaaa", "aa"),
        ("日本語", "日本人語"),
        ("é", "e"),
    ] {
        let e = edit_between(old, new);
        assert_eq!(e.apply(old), new, "{old:?} -> {new:?}");
    }
}
