use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;
use crate::tasks::{TaskEdit, set_task_checked};

const TODAY: Date = Date {
    year: 2026,
    month: 11,
    day: 14,
};

#[test]
fn ticking_a_weekly_task_writes_the_next_one_above() {
    let body = "# Shoots\n- [ ] film Pixel review @2026-11-14T15:00 every week\n- [ ] other\n";
    assert_eq!(
        tick_task_at(body, 0, TODAY).unwrap(),
        "# Shoots\n- [ ] film Pixel review @2026-11-21T15:00 every week\n\
         - [x] film Pixel review @2026-11-14T15:00\n- [ ] other\n"
    );
}

#[test]
fn words_after_the_repeat_stay_on_both_lines() {
    let body = "  - [ ] pay @2026-11-14 every month #bills\n";
    assert_eq!(
        tick_task_at(body, 0, TODAY).unwrap(),
        "  - [ ] pay @2026-12-14 every month #bills\n  - [x] pay @2026-11-14 #bills\n"
    );
}

#[test]
fn after_done_counts_from_the_tick_day_or_the_due_day_if_later() {
    let late = "- [ ] a @2026-11-01 every 3 days after done\n";
    assert!(
        tick_task_at(late, 0, TODAY)
            .unwrap()
            .starts_with("- [ ] a @2026-11-17 every 3 days after done\n")
    );
    let early = "- [ ] a @2026-11-20 every 3 days after done\n";
    assert!(
        tick_task_at(early, 0, TODAY)
            .unwrap()
            .starts_with("- [ ] a @2026-11-23 every 3 days after done\n")
    );
}

#[test]
fn an_overdue_task_jumps_to_the_next_future_date() {
    let body = "- [ ] a @2026-10-01 every week\n";
    // Weekly from 1 Oct, ticked on 14 Nov: the next one is after the 14th.
    let out = tick_task_at(body, 0, TODAY).unwrap();
    let next = task_items(&out)[0].due.clone().unwrap();
    assert!(next.as_str() > "2026-11-14", "{next}");
}

#[test]
fn the_last_line_without_a_break_and_crlf_bodies() {
    assert_eq!(
        tick_task_at("- [ ] a @2026-11-14 every day", 0, TODAY).unwrap(),
        "- [ ] a @2026-11-15 every day\n- [x] a @2026-11-14"
    );
    assert_eq!(
        tick_task_at("x\r\n- [ ] a @2026-11-14 every day\r\n", 0, TODAY).unwrap(),
        "x\r\n- [ ] a @2026-11-15 every day\r\n- [x] a @2026-11-14\r\n"
    );
}

#[test]
fn a_plain_task_a_done_log_and_a_missing_task_behave_as_before() {
    let body = "- [ ] a @2026-11-14\n- [x] b @2026-11-07\n";
    assert_eq!(
        tick_task_at(body, 0, TODAY).unwrap(),
        "- [x] a @2026-11-14\n- [x] b @2026-11-07\n"
    );
    assert_eq!(
        tick_task_at(body, 1, TODAY).unwrap(),
        "- [ ] a @2026-11-14\n- [ ] b @2026-11-07\n"
    );
    assert_eq!(tick_task_at(body, 2, TODAY), None);
    // An `every` with no due date is not a repeat.
    assert_eq!(
        tick_task_at("- [ ] a every week\n", 0, TODAY).unwrap(),
        "- [x] a every week\n"
    );
}

#[test]
fn ticking_the_same_task_twice_writes_one_next_line() {
    let body = "- [ ] film @2026-11-14T15:00 every week\n";
    let text = "film @2026-11-14T15:00 every week";
    let TaskEdit::Changed(once) = set_task_checked(body, 0, text, true, TODAY) else {
        panic!("not ticked");
    };
    // A replay of the same tick finds no such task any more.
    assert_eq!(
        set_task_checked(&once, 0, text, true, TODAY),
        TaskEdit::Missing
    );
    assert_eq!(once.matches("every week").count(), 1);
}

#[test]
fn unticking_the_log_line_keeps_the_next_one() {
    let body = "- [ ] film @2026-11-14T15:00 every week\n";
    let once = tick_task_at(body, 0, TODAY).unwrap();
    let TaskEdit::Changed(back) =
        set_task_checked(&once, 1, "film @2026-11-14T15:00", false, TODAY)
    else {
        panic!("not cleared");
    };
    assert_eq!(
        back,
        "- [ ] film @2026-11-21T15:00 every week\n- [ ] film @2026-11-14T15:00\n"
    );
}

// MARK: Properties

fn config() -> Config {
    Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0074_6963_6b34),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn gen_rule() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("every day"),
        Just("every week"),
        Just("every 2 weeks on mon and thu"),
        Just("every month"),
        Just("every month on the 31st"),
        Just("every month on the last day"),
        Just("every year"),
        Just("every 3 days after done"),
        Just("every weekday"),
    ]
}

fn gen_line() -> impl Strategy<Value = String> {
    (
        prop_oneof![Just("- [ ] "), Just("  * [ ] "), Just("1. [ ] ")],
        "[a-z]{1,8}",
        -20_000i64..60_000,
        prop::option::of((0u32..24, 0u32..60)),
        prop::option::of(gen_rule()),
        prop_oneof![Just(""), Just(" #tag")],
    )
        .prop_map(|(lead, words, days, time, rule, tail)| {
            let date = Date::from_days(days);
            let due = Due { date, time };
            match rule {
                Some(rule) => format!("{lead}{words} @{} {rule}{tail}", due.format()),
                None => format!("{lead}{words} @{}{tail}", due.format()),
            }
        })
}

fn gen_body() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(
            prop_oneof![gen_line(), Just("plain".to_owned()), Just("```".to_owned())],
            1..8,
        ),
        any::<bool>(),
    )
        .prop_map(|(lines, end)| {
            let mut body = lines.join("\n");
            if end {
                body.push('\n');
            }
            body
        })
}

proptest! {
    #![proptest_config(config())]

    /// A task with no repeat ticks exactly as before: only its box changes.
    #[test]
    fn prop_a_plain_task_ticks_as_it_did(body in gen_body(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        for (ix, item) in task_items(&body).iter().enumerate() {
            if item.repeat.is_none() {
                prop_assert_eq!(tick_task_at(&body, ix, today), toggle_task(&body, ix));
            }
        }
    }

    /// Ticking a repeating task adds exactly one line, whose due is strictly
    /// after both the old due and the day it is ticked, and the ticked line
    /// no longer repeats.
    #[test]
    fn prop_the_next_line_is_strictly_after(body in gen_body(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        let before = task_items(&body);
        for (ix, item) in before.iter().enumerate() {
            let Some(_) = item.repeat else { continue };
            let out = tick_task_at(&body, ix, today).unwrap();
            let after = task_items(&out);
            prop_assert_eq!(after.len(), before.len() + 1);
            let next = &after[ix];
            let done = &after[ix + 1];
            prop_assert!(!next.checked && next.repeat.is_some());
            prop_assert!(done.checked && done.repeat.is_none());
            let old = Due::parse(item.due.as_deref().unwrap()).unwrap();
            let new = Due::parse(next.due.as_deref().unwrap()).unwrap();
            prop_assert!(new.date > old.date);
            prop_assert!(new.date > today);
            prop_assert_eq!(new.time, old.time);
            prop_assert_eq!(done.due.as_deref(), item.due.as_deref());
            // Every other line is as it was.
            let at = item.line as usize;
            let keep = |s: &str| -> Vec<String> {
                s.lines().enumerate().filter(|(i, _)| *i != at && *i != at + 1).map(|(_, l)| l.to_owned()).collect()
            };
            let original: Vec<String> = body.lines().enumerate().filter(|(i, _)| *i != at).map(|(_, l)| l.to_owned()).collect();
            prop_assert_eq!(keep(&out).len() + 2, out.lines().count());
            prop_assert_eq!(keep(&out), original);
        }
    }

    /// The same tick sent twice (a sync replay) writes one next line: the
    /// second finds no open task with those words where it was.
    #[test]
    fn prop_a_replayed_tick_writes_nothing(body in gen_body(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        for item in task_items(&body).into_iter().filter(|t| t.repeat.is_some() && !t.checked) {
            let once = match set_task_checked(&body, item.line, &item.text, true, today) {
                TaskEdit::Changed(once) => once,
                // Two identical lines: the tick is refused, nothing to replay.
                TaskEdit::Missing => continue,
                TaskEdit::Unchanged => unreachable!("it was open"),
            };
            let again = set_task_checked(&once, item.line, &item.text, true, today);
            prop_assert!(!matches!(again, TaskEdit::Changed(_)), "ticked twice: {again:?}");
        }
    }
}

// MARK: Stop repeating

#[test]
fn stop_repeating_drops_the_every_words_and_keeps_the_date() {
    let body = "# Shoots\n- [ ] film @2026-11-14T15:00 every week #video\n- [ ] other\n";
    assert_eq!(
        stop_repeating(body, 1).unwrap(),
        "# Shoots\n- [ ] film @2026-11-14T15:00 #video\n- [ ] other\n"
    );
    let last = "- [ ] water @2026-11-12 every 3 days after done";
    assert_eq!(stop_repeating(last, 0).unwrap(), "- [ ] water @2026-11-12");
    let crlf = "- [ ] pay @2026-11-30 every month on the last day\r\nnext\r\n";
    assert_eq!(
        stop_repeating(crlf, 0).unwrap(),
        "- [ ] pay @2026-11-30\r\nnext\r\n"
    );
}

#[test]
fn stop_repeating_leaves_what_does_not_repeat() {
    assert_eq!(stop_repeating("- [ ] a @2026-11-14\n", 0), None);
    assert_eq!(stop_repeating("- [x] a @2026-11-14 every week\n", 0), None);
    assert_eq!(stop_repeating("plain every week\n", 0), None);
    assert_eq!(stop_repeating("- [ ] a @2026-11-14 every week\n", 1), None);
    assert_eq!(
        stop_repeating("```\n- [ ] a @2026-11-14 every week\n```\n", 1),
        None
    );
}

proptest! {
    #![proptest_config(config())]

    /// Stopping a repeat changes only that task's repeat: every task keeps
    /// its box, its due date and its place, every other line is untouched,
    /// and a second stop finds nothing to do.
    #[test]
    fn prop_stop_repeating_changes_only_the_repeat(body in gen_body(), line in 0u32..8) {
        let before = task_items(&body);
        let target = before.iter().find(|item| item.line == line);
        let Some(out) = stop_repeating(&body, line) else {
            prop_assert!(target.is_none_or(|item| item.checked || item.repeat.is_none()));
            return Ok(());
        };
        let item = target.unwrap();
        prop_assert!(!item.checked && item.repeat.is_some());
        let after = task_items(&out);
        prop_assert_eq!(after.len(), before.len());
        for (old, new) in before.iter().zip(&after) {
            prop_assert_eq!(old.line, new.line);
            prop_assert_eq!(old.checked, new.checked);
            prop_assert_eq!(&old.due, &new.due);
            if old.line == line {
                prop_assert!(new.repeat.is_none());
            } else {
                prop_assert_eq!(old.repeat, new.repeat);
            }
        }
        let others = |s: &str| -> Vec<String> {
            s.lines().enumerate().filter(|(i, _)| *i as u32 != line).map(|(_, l)| l.to_owned()).collect()
        };
        prop_assert_eq!(others(&out), others(&body));
        prop_assert_eq!(out.ends_with('\n'), body.ends_with('\n'));
        prop_assert_eq!(stop_repeating(&out, line), None);
    }
}

// MARK: Linked nested tasks

const TREE: &str = "- [ ] parent\n  - [ ] one\n    - [ ] deep\n  - [ ] two\n- [ ] other\n";

#[test]
fn ticking_a_parent_ticks_everything_under_it() {
    assert_eq!(
        tick_task_linked(TREE, 0, TODAY, true).unwrap(),
        "- [x] parent\n  - [x] one\n    - [x] deep\n  - [x] two\n- [ ] other\n"
    );
}

#[test]
fn ticking_the_last_child_ticks_the_parent_up_the_tree() {
    let body = "- [ ] parent\n  - [x] one\n    - [ ] deep\n  - [x] two\n";
    assert_eq!(
        tick_task_linked(body, 2, TODAY, true).unwrap(),
        "- [x] parent\n  - [x] one\n    - [x] deep\n  - [x] two\n"
    );
    // One child still open: the parent stays open.
    let open = "- [ ] parent\n  - [ ] one\n  - [ ] two\n";
    assert_eq!(
        tick_task_linked(open, 1, TODAY, true).unwrap(),
        "- [ ] parent\n  - [x] one\n  - [ ] two\n"
    );
}

#[test]
fn clearing_a_child_reopens_its_ancestors_and_leaves_its_children() {
    let body = "- [x] parent\n  - [x] one\n    - [x] deep\n  - [x] two\n";
    assert_eq!(
        tick_task_linked(body, 1, TODAY, true).unwrap(),
        "- [ ] parent\n  - [ ] one\n    - [x] deep\n  - [x] two\n"
    );
    // Clearing a parent leaves its children alone.
    assert_eq!(
        tick_task_linked(body, 0, TODAY, true).unwrap(),
        "- [ ] parent\n  - [x] one\n    - [x] deep\n  - [x] two\n"
    );
}

#[test]
fn a_repeating_child_is_never_cascaded_and_counts_as_open() {
    let body = "- [ ] parent\n  - [ ] plain\n  - [ ] water @2026-11-14 every day\n";
    assert_eq!(
        tick_task_linked(body, 0, TODAY, true).unwrap(),
        "- [x] parent\n  - [x] plain\n  - [ ] water @2026-11-14 every day\n"
    );
    // The plain child is the last one done, but the repeating one is open.
    assert_eq!(
        tick_task_linked(body, 1, TODAY, true).unwrap(),
        "- [ ] parent\n  - [x] plain\n  - [ ] water @2026-11-14 every day\n"
    );
}

#[test]
fn ticking_a_repeating_child_cascades_on_the_body_it_wrote() {
    // The next one goes above; the log line is the one that is ticked, and
    // the new open repeating line keeps the parent open.
    let body = "- [ ] parent\n  - [ ] water @2026-11-14 every day\n    - [ ] sub\n";
    assert_eq!(
        tick_task_linked(body, 1, TODAY, true).unwrap(),
        "- [ ] parent\n  - [ ] water @2026-11-15 every day\n  - [x] water @2026-11-14\n    - [x] sub\n"
    );
}

#[test]
fn code_fences_and_other_lines_are_not_part_of_the_tree() {
    let body = "- [ ] parent\n```\n  - [ ] in fence\n```\n  - [ ] child\ntext\n";
    assert_eq!(
        tick_task_linked(body, 0, TODAY, true).unwrap(),
        "- [x] parent\n```\n  - [ ] in fence\n```\n  - [x] child\ntext\n"
    );
}

#[test]
fn would_cascade_is_true_only_when_the_link_changes_the_tick() {
    assert!(would_cascade(TREE, 0, TODAY));
    assert!(!would_cascade(TREE, 4, TODAY));
    assert!(!would_cascade(TREE, 9, TODAY));
    // The only child ticks the parent.
    assert!(would_cascade("- [ ] a\n  - [ ] b\n", 1, TODAY));
}

fn gen_tree_line() -> impl Strategy<Value = String> {
    (
        prop_oneof![Just(""), Just("  "), Just("    "), Just("\t")],
        prop_oneof![Just("[ ]"), Just("[x]")],
        "[a-z]{1,5}",
        prop::option::of(gen_rule()),
    )
        .prop_map(|(lead, boxed, words, rule)| match rule {
            Some(rule) => format!("{lead}- {boxed} {words} @2026-11-14 {rule}"),
            None => format!("{lead}- {boxed} {words}"),
        })
}

fn gen_tree() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            4 => gen_tree_line(),
            1 => Just("plain".to_owned()),
            1 => Just("```".to_owned()),
        ],
        1..9,
    )
    .prop_map(|lines| lines.join("\n") + "\n")
}

/// The body with every task's box blanked, to compare what is not a box.
fn without_boxes(body: &str) -> String {
    body.lines()
        .map(|line| {
            if crate::tasks::task_marker(line).is_some() {
                line.replacen("[x]", "[ ]", 1).replacen("[X]", "[ ]", 1)
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

proptest! {
    #![proptest_config(config())]

    /// Unlinked is the plain tick, whatever the tree looks like.
    #[test]
    fn prop_unlinked_equals_the_plain_tick(body in gen_tree(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        for ix in 0..=task_items(&body).len() {
            prop_assert_eq!(tick_task_linked(&body, ix, today, false), tick_task_at(&body, ix, today));
        }
    }

    /// Linking changes only boxes: apart from them the body is the plain
    /// tick's, and the lines that are not tasks are byte for byte the same.
    #[test]
    fn prop_linking_only_changes_boxes(body in gen_tree(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        for ix in 0..task_items(&body).len() {
            let plain = tick_task_at(&body, ix, today).unwrap();
            let linked = tick_task_linked(&body, ix, today, true).unwrap();
            prop_assert_eq!(without_boxes(&linked), without_boxes(&plain));
            prop_assert_eq!(linked.ends_with('\n'), plain.ends_with('\n'));
            let others = |s: &str| -> Vec<String> {
                s.lines().filter(|l| crate::tasks::task_marker(l).is_none()).map(str::to_owned).collect()
            };
            prop_assert_eq!(others(&linked), others(&plain));
        }
    }

    /// After ticking an open parent, every non-repeating task under it is done.
    #[test]
    fn prop_ticking_a_parent_ticks_its_descendants(body in gen_tree(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        let before = task_items(&body);
        let depth = indents(&body, &before);
        for ix in 0..before.len() {
            if before[ix].checked || before[ix].repeat.is_some() {
                continue;
            }
            let out = tick_task_linked(&body, ix, today, true).unwrap();
            let after = task_items(&out);
            prop_assert_eq!(after.len(), before.len());
            prop_assert!(after[ix].checked);
            for j in ix + 1..before.len() {
                if depth[j] <= depth[ix] {
                    break;
                }
                prop_assert!(after[j].checked || after[j].repeat.is_some());
            }
        }
    }

    /// Ticking an open leaf and then clearing it, both linked, leaves the
    /// leaf and every task above it open.
    #[test]
    fn prop_clearing_a_leaf_reopens_the_ancestors(body in gen_tree(), today in (-20_000i64..60_000).prop_map(Date::from_days)) {
        let before = task_items(&body);
        let depth = indents(&body, &before);
        for ix in 0..before.len() {
            let leaf = before[ix].repeat.is_none()
                && !before[ix].checked
                && (ix + 1 == before.len() || depth[ix + 1] <= depth[ix]);
            if !leaf {
                continue;
            }
            let ticked = tick_task_linked(&body, ix, today, true).unwrap();
            let cleared = tick_task_linked(&ticked, ix, today, true).unwrap();
            let items = task_items(&cleared);
            let depth_after = indents(&cleared, &items);
            prop_assert!(!items[ix].checked);
            let mut child = ix;
            while let Some(parent) = parent_of(&depth_after, child) {
                prop_assert!(!items[parent].checked);
                child = parent;
            }
        }
    }
}
