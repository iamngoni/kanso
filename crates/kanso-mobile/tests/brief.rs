//! Open loops, the morning brief and the weekly review on the phone.

mod common;

use common::*;
use kanso_mobile::{BriefPrefs, LoopAction, OpenLoopKind};
use proptest::prelude::*;

// no-prop-test: a fixed note, found and closed.
#[test]
fn a_loop_is_listed_and_closing_it_removes_it() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(
        library,
        "Plans",
        "I'll send the deck to canaryperson.\nWaiting on Rudo for the invoice.\n",
    );
    let board = library.open_loops().unwrap();
    assert!(board.enabled);
    assert_eq!(board.total, 2);
    let mine = board
        .items
        .iter()
        .find(|item| item.kind == OpenLoopKind::Mine)
        .expect("a loop of mine");
    assert_eq!(mine.note_id, id);
    assert_eq!(mine.age_label, "today");
    assert!(
        library
            .close_loop(&id, &mine.key, mine.line, LoopAction::Done)
            .unwrap()
    );
    let after = library.open_loops().unwrap();
    assert_eq!(after.total, 1);
    assert_eq!(after.items[0].kind, OpenLoopKind::Waiting);
    // Not a loop hides it for good; Make Task writes a task line.
    let waiting = &after.items[0];
    library
        .close_loop(&id, &waiting.key, waiting.line, LoopAction::MakeTask)
        .unwrap();
    assert_eq!(library.open_loops().unwrap().total, 0);
    let body = library.note(&id).unwrap().body.unwrap_or_default();
    assert!(body.contains("- [ ] Waiting on Rudo"), "{body}");
}

// no-prop-test: a fixed note and the settings switch.
#[test]
fn turning_loops_off_empties_the_list() {
    let phone = lone_phone();
    let library = &phone.library;
    note(library, "Plans", "I'll send the deck.\n");
    assert_eq!(library.open_loops().unwrap().total, 1);
    let mut prefs = library.brief_prefs().unwrap();
    prefs.loops = false;
    library.set_brief_prefs(prefs).unwrap();
    let board = library.open_loops().unwrap();
    assert!(!board.enabled);
    assert!(board.items.is_empty());
}

// no-prop-test: a fixed note, the brief and the review of this week.
#[test]
fn the_brief_and_the_review_hold_the_loop_and_save_once() {
    let phone = lone_phone();
    let library = &phone.library;
    note(library, "Plans", "I'll call canaryperson about it.\n");
    let brief = library.brief().unwrap();
    assert_eq!(brief.loops_total, 1);
    assert!(!brief.is_empty);
    let review = library.review(None).unwrap();
    assert!(review.is_this_week);
    assert_eq!(review.still_open.len(), 1);
    assert_eq!(review.written.len(), 1);
    assert_eq!(review.saved_note_id, None);
    let saved = library.save_review(review.from.clone(), false).unwrap();
    assert!(!saved.exists);
    let again = library.save_review(review.from.clone(), false).unwrap();
    assert!(again.exists);
    assert_eq!(again.note_id, saved.note_id);
    let replaced = library.save_review(review.from.clone(), true).unwrap();
    assert!(!replaced.exists);
    assert_eq!(
        library.review(Some(review.from)).unwrap().saved_note_id,
        Some(saved.note_id)
    );
    assert!(library.review(Some("nonsense".into())).is_err());
}

// no-prop-test: a fixed note locked, then looked for everywhere.
#[test]
fn a_locked_note_gives_no_loop_brief_or_review_line() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(library, "Secret", "I'll call canarysecret tomorrow.\n");
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    assert_eq!(library.open_loops().unwrap().total, 0);
    let seen = format!(
        "{:?}{:?}{:?}",
        library.open_loops().unwrap(),
        library.brief().unwrap(),
        library.review(None).unwrap()
    );
    assert!(!seen.contains("canarysecret"), "{seen}");
}

// no-prop-test: the plan for fixed settings.
#[test]
fn the_plan_has_fixed_text_and_nothing_when_the_system_says_no() {
    let phone = lone_phone();
    let library = &phone.library;
    assert!(library.brief_plan(false).unwrap().is_empty());
    let plan = library.brief_plan(true).unwrap();
    assert!(!plan.is_empty());
    for notice in &plan {
        assert!(notice.hour < 24 && notice.minute < 60);
        assert!(notice.id.starts_with(if notice.is_review {
            "review-"
        } else {
            "brief-"
        }));
    }
}

proptest! {
    /// Whatever is saved reads back clamped to times and days that exist.
    #[test]
    fn prop_saved_settings_are_clamped(h in 0u32..100, m in 0u32..200, day in 0u32..20) {
        let phone = lone_phone();
        let mut prefs: BriefPrefs = phone.library.brief_prefs().unwrap();
        prefs.brief_hour = h;
        prefs.brief_minute = m;
        prefs.review_day = day;
        let got = phone.library.set_brief_prefs(prefs).unwrap();
        prop_assert!(got.brief_hour < 24 && got.brief_minute < 60 && got.review_day < 7);
        prop_assert_eq!(phone.library.brief_prefs().unwrap(), got);
    }
}
