//! Repeat lines after a merge: two members ticking the same repeating task
//! offline each write the next line, and the merge keeps one. Property
//! tests run on the seed `PROPTEST_RNG_SEED` pins.

use proptest::prelude::*;

use crate::doc::{CollabDoc, Field};
use crate::doc_dedupe::duplicate_next_lines;

const NEXT: &str = "- [ ] film Pixel review @2026-11-21T15:00 every week";
const DONE: &str = "- [x] film Pixel review @2026-11-14T15:00";

fn deduped(text: &str) -> String {
    let mut out = text.to_owned();
    for range in duplicate_next_lines(text).iter().rev() {
        out.replace_range(range.clone(), "");
    }
    out
}

#[test]
fn an_open_repeating_line_equal_to_the_one_above_collapses() {
    let text = format!("{NEXT}\n{NEXT}\n{DONE}\n");
    assert_eq!(deduped(&text), format!("{NEXT}\n{DONE}\n"));
}

#[test]
fn three_in_a_row_and_one_at_the_end_without_a_break_collapse_to_one() {
    let text = format!("{NEXT}\n{NEXT}\n{NEXT}\n{DONE}\n");
    assert_eq!(deduped(&text), format!("{NEXT}\n{DONE}\n"));
    let bare = format!("{DONE}\n{NEXT}\n{NEXT}");
    assert_eq!(deduped(&bare), format!("{DONE}\n{NEXT}"));
    let all = format!("{NEXT}\n{NEXT}\n{NEXT}");
    assert_eq!(deduped(&all), NEXT);
}

#[test]
fn lines_that_differ_in_any_byte_are_kept() {
    let other = "- [ ] film Pixel review @2026-11-21T15:00 every week #a";
    let text = format!("{NEXT}\n{other}\n{NEXT}\n");
    assert_eq!(deduped(&text), text);
    let spaced = format!("{NEXT}\n{NEXT} \n");
    assert_eq!(deduped(&spaced), spaced);
}

#[test]
fn only_open_tasks_with_a_repeat_are_ever_removed() {
    for line in [
        "- [ ] plain task",
        "- [ ] due @2026-11-21",
        DONE,
        "text",
        "",
    ] {
        let text = format!("{line}\n{line}\n");
        assert_eq!(deduped(&text), text, "{line:?}");
    }
    // In a code block it is no task.
    let fenced = format!("```\n{NEXT}\n{NEXT}\n```\n");
    assert_eq!(deduped(&fenced), fenced);
}

#[test]
fn a_merged_document_loses_the_copy_and_both_members_converge() {
    let base = CollabDoc::from_text("Shoots", &format!("{NEXT}\n"));
    let state = base.encode_state();
    let (a, b) = (
        CollabDoc::load(&state).unwrap(),
        CollabDoc::load(&state).unwrap(),
    );
    let after = format!("{NEXT}\n{NEXT}\n");
    let from_a = a.set_text(Field::Body, &after).unwrap();
    let from_b = b.set_text(Field::Body, &after).unwrap();
    a.apply(&from_b).unwrap();
    b.apply(&from_a).unwrap();
    assert_eq!(a.text(Field::Body).matches(NEXT).count(), 3);
    let (da, db) = (
        a.dedupe_repeat_lines().unwrap(),
        b.dedupe_repeat_lines().unwrap(),
    );
    a.apply(&db).unwrap();
    b.apply(&da).unwrap();
    assert_eq!(a.text(Field::Body), b.text(Field::Body));
    assert_eq!(a.text(Field::Body), format!("{NEXT}\n"));
    // Removing it again changes nothing.
    assert!(a.dedupe_repeat_lines().is_none());
}

#[test]
fn a_next_line_and_its_log_line_written_twice_collapse_to_one_pair() {
    let text = format!("{NEXT}\n{DONE}\n{NEXT}\n{DONE}\nafter\n");
    assert_eq!(deduped(&text), format!("{NEXT}\n{DONE}\nafter\n"));
    // A different next line (ticked another day) keeps its pair.
    let later = "- [ ] film Pixel review @2026-11-28T15:00 every week";
    let text = format!("{NEXT}\n{DONE}\n{later}\n{DONE}\n");
    assert_eq!(deduped(&text), text);
}

const TICKED: &str = "- [ ] film Pixel review @2026-11-21T15:00 every week\n\
                      - [x] film Pixel review @2026-11-14T15:00\n";

fn start_text() -> String {
    "- [ ] film Pixel review @2026-11-14T15:00 every week\n".to_owned()
}

#[test]
fn a_tick_is_written_as_two_whole_lines() {
    use crate::doc_dedupe::tick_edits;
    let edits = tick_edits(&start_text(), TICKED).unwrap();
    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].insert,
        TICKED.lines().next().unwrap().to_owned() + "\n"
    );
    assert_eq!(edits[0].delete, 0);
    // Anything else isn't one.
    assert!(tick_edits(&start_text(), &format!("{}more\n", start_text())).is_none());
    assert!(tick_edits("plain\n", "plain\nplain\n").is_none());
    assert!(tick_edits(&start_text(), &start_text()).is_none());
}

#[test]
fn two_members_ticking_the_same_task_offline_end_with_one_pair() {
    let base = CollabDoc::from_text("Shoots", &format!("intro\n{}tail\n", start_text()));
    let state = base.encode_state();
    let (a, b) = (
        CollabDoc::load(&state).unwrap(),
        CollabDoc::load(&state).unwrap(),
    );
    let ticked = format!("intro\n{TICKED}tail\n");
    let from_a = a.set_text(Field::Body, &ticked).unwrap();
    let from_b = b.set_text(Field::Body, &ticked).unwrap();
    a.apply(&from_b).unwrap();
    b.apply(&from_a).unwrap();
    assert_eq!(a.text(Field::Body), b.text(Field::Body));
    let (da, db) = (
        a.dedupe_repeat_lines().unwrap(),
        b.dedupe_repeat_lines().unwrap(),
    );
    a.apply(&db).unwrap();
    b.apply(&da).unwrap();
    assert_eq!(a.text(Field::Body), ticked);
    assert_eq!(b.text(Field::Body), ticked);
}

fn line_pool() -> Vec<String> {
    vec![
        NEXT.to_owned(),
        "- [ ] pay rent @2026-12-01 every month".to_owned(),
        DONE.to_owned(),
        "- [ ] plain".to_owned(),
        "notes".to_owned(),
    ]
}

proptest! {
    #![proptest_config(crate::tests::config(24))]

    /// The pure rule is idempotent, never removes a line that isn't a copy
    /// and keeps every distinct line.
    #[test]
    fn prop_dedupe_is_idempotent_and_loses_no_distinct_line(
        picks in prop::collection::vec(0usize..5, 0..14),
        end_break in any::<bool>(),
    ) {
        let pool = line_pool();
        let mut text = picks.iter().map(|i| pool[*i].as_str()).collect::<Vec<_>>().join("\n");
        if end_break && !text.is_empty() { text.push('\n'); }
        let once = deduped(&text);
        prop_assert_eq!(&deduped(&once), &once);
        for line in text.lines() {
            prop_assert!(once.lines().any(|l| l == line), "{line:?} lost");
        }
        prop_assert!(once.len() <= text.len());
    }

    /// Whatever order the members' edits arrive in, each removes the copy
    /// from what it merged, and after they exchange that they hold the
    /// same text, with no copy left.
    #[test]
    fn prop_merge_then_dedupe_is_order_independent(
        base_picks in prop::collection::vec(0usize..5, 1..5),
        a_inserts in prop::collection::vec((0usize..6, 0usize..5), 0..4),
        b_inserts in prop::collection::vec((0usize..6, 0usize..5), 0..4),
        a_first in any::<bool>(),
    ) {
        let pool = line_pool();
        let lines = |picks: &[usize]| picks.iter().map(|i| pool[*i].clone()).collect::<Vec<_>>();
        let join = |lines: &[String]| {
            let mut text = lines.join("\n");
            text.push('\n');
            text
        };
        let base = lines(&base_picks);
        let state = CollabDoc::from_text("t", &join(&base)).encode_state();
        let (a, b) = (CollabDoc::load(&state).unwrap(), CollabDoc::load(&state).unwrap());
        let edit = |doc: &CollabDoc, inserts: &[(usize, usize)]| {
            let mut now = base.clone();
            for (at, pick) in inserts {
                now.insert((*at).min(now.len()), pool[*pick].clone());
            }
            doc.set_text(Field::Body, &join(&now))
        };
        let (from_a, from_b) = (edit(&a, &a_inserts), edit(&b, &b_inserts));
        let (first, second) = if a_first { (&from_a, &from_b) } else { (&from_b, &from_a) };
        // A third replica that takes the updates in the order given: the
        // merge itself is the same whichever.
        let c = CollabDoc::load(&state).unwrap();
        for update in [first, second].into_iter().flatten() { c.apply(update).unwrap(); }
        for update in [&from_b, &from_a].into_iter().flatten() { a.apply(update).ok(); }
        for update in [&from_a, &from_b].into_iter().flatten() { b.apply(update).ok(); }
        prop_assert_eq!(a.text(Field::Body), b.text(Field::Body));
        prop_assert_eq!(a.text(Field::Body), c.text(Field::Body));
        let before = a.text(Field::Body);
        let (da, db) = (a.dedupe_repeat_lines(), b.dedupe_repeat_lines());
        if let Some(db) = &db { a.apply(db).unwrap(); }
        if let Some(da) = &da { b.apply(da).unwrap(); }
        prop_assert_eq!(a.text(Field::Body), b.text(Field::Body));
        prop_assert!(duplicate_next_lines(&a.text(Field::Body)).is_empty());
        prop_assert!(a.dedupe_repeat_lines().is_none());
        // The same text whether it was removed once or by both.
        prop_assert_eq!(a.text(Field::Body), deduped(&before));
        for line in before.lines() {
            prop_assert!(a.text(Field::Body).lines().any(|l| l == line));
        }
    }

    /// Two members tick the same one of several repeating tasks offline,
    /// one of them also adding a line: after they trade updates and each
    /// removes the copy, both hold the text one tick and the added line
    /// make, in either order of arrival.
    #[test]
    fn prop_a_tick_made_twice_lands_once(
        count in 1usize..5,
        which in 0usize..5,
        extra in any::<bool>(),
        a_first in any::<bool>(),
    ) {
        let which = which % count;
        let task = |i: usize| format!("- [ ] task {i} @2026-11-14 every week\n");
        let base: String = (0..count).map(|i| format!("{}note {i}\n", task(i))).collect();
        let tick = |text: &str| {
            kanso_core::tick::tick_task_at(
                text,
                which,
                kanso_core::dates::Date::new(2026, 11, 14).unwrap(),
            )
            .unwrap()
        };
        let state = CollabDoc::from_text("t", &base).encode_state();
        let (a, b) = (CollabDoc::load(&state).unwrap(), CollabDoc::load(&state).unwrap());
        let mut b_text = base.clone();
        if extra {
            b_text.push_str("added\n");
            b.set_text(Field::Body, &b_text);
        }
        let ticked_a = tick(&base);
        let ticked_b = tick(&b_text);
        let from_a = a.set_text(Field::Body, &ticked_a).unwrap();
        // B's updates since the base, as one.
        b.set_text(Field::Body, &ticked_b).unwrap();
        let from_b = b.encode_state();
        let (first, second) = if a_first { (&from_a, &from_b) } else { (&from_b, &from_a) };
        let c = CollabDoc::load(&state).unwrap();
        c.apply(first).unwrap();
        c.apply(second).unwrap();
        a.apply(&from_b).unwrap();
        b.apply(&from_a).unwrap();
        prop_assert_eq!(a.text(Field::Body), b.text(Field::Body));
        prop_assert_eq!(a.text(Field::Body), c.text(Field::Body));
        let (da, db) = (a.dedupe_repeat_lines(), b.dedupe_repeat_lines());
        if let Some(db) = &db { a.apply(db).unwrap(); }
        if let Some(da) = &da { b.apply(da).unwrap(); }
        prop_assert_eq!(a.text(Field::Body), b.text(Field::Body));
        let want = if extra { tick(&b_text) } else { ticked_a.clone() };
        prop_assert_eq!(a.text(Field::Body), want);
    }
}
