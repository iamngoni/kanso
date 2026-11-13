//! Typed dates through the real editor: the hint as typing rests, Tab and
//! Escape, undo, the `@` list's repeats, and the repeat mark on due chips.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::writing_tests::Open;

const PHRASE: &str = " 2026-11-13 3pm every week";
const STORED: &str = "- [ ] film @2026-11-13T15:00 every week";

/// A task open with the caret at its end, `PHRASE` typed after it.
fn typed(cx: &mut TestAppContext) -> Open {
    let open = Open::new(cx, "- [ ] film\nplain");
    open.select(cx, 10..10);
    open.input(cx, PHRASE);
    open
}

/// Lets typing rest, so the hint is worked out.
fn rest(cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(400));
    cx.run_until_parked();
}

fn hint(open: &Open, cx: &mut TestAppContext) -> Option<(String, String)> {
    open.with(cx, |editor, _, cx| editor.when_hint_for_test(cx))
}

/// The hint shows once typing rests, says what the line will become, goes
/// when typing goes on, and the text is never rewritten by itself.
// no-prop-test: drives the editor; `when_hint` has the properties.
#[gpui_kit::test]
fn the_hint_appears_and_disappears(cx: &mut TestAppContext) {
    let open = typed(cx);
    // At once, nothing; the editor waits for typing to rest.
    assert_eq!(hint(&open, cx), None);
    rest(cx);
    let (label, stored) = hint(&open, cx).expect("a hint once typing rests");
    assert_eq!(stored, "@2026-11-13T15:00 every week");
    assert!(label.ends_with("15:00 \u{b7} every week"), "{label}");
    // Nothing was rewritten.
    assert_eq!(open.text(cx).0, format!("- [ ] film{PHRASE}\nplain"));
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(open.text(cx).0, format!("- [ ] film{PHRASE}\nplain"));

    // Typing on takes it away at once; the line no longer ends in a date.
    open.input(cx, "x");
    assert_eq!(hint(&open, cx), None);
    rest(cx);
    assert_eq!(hint(&open, cx), None);
    open.keys(cx, "backspace");
    rest(cx);
    assert!(hint(&open, cx).is_some());

    // The caret off the line: no hint there.
    let end = open.text(cx).0.len();
    open.select(cx, end..end);
    assert_eq!(hint(&open, cx), None);
}

/// Tab writes the stored form with the caret after it, in one undo step
/// that puts the phrase back; the hint goes.
// no-prop-test: drives the editor; `when_hint` has the properties.
#[gpui_kit::test]
fn tab_applies_and_undo_restores(cx: &mut TestAppContext) {
    let open = typed(cx);
    rest(cx);
    open.keys(cx, "tab");
    let (text, caret) = open.text(cx);
    assert_eq!(text, format!("{STORED}\nplain"));
    assert_eq!(caret, STORED.len()..STORED.len());
    assert_eq!(hint(&open, cx), None);
    rest(cx);
    assert_eq!(hint(&open, cx), None, "the stored form needs no hint");

    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, format!("- [ ] film{PHRASE}\nplain"));
    open.keys(cx, "cmd-shift-z");
    assert_eq!(open.text(cx).0, format!("{STORED}\nplain"));
}

/// With no hint showing, Tab is the editor's own: it moves a list item.
// no-prop-test: drives the editor; `when_hint` has the properties.
#[gpui_kit::test]
fn tab_without_a_hint_is_the_normal_tab(cx: &mut TestAppContext) {
    let open = Open::new(cx, "- a\n- b");
    open.select(cx, 7..7);
    rest(cx);
    open.keys(cx, "tab");
    assert_eq!(open.text(cx).0, "- a\n  - b");
}

/// Escape sends the hint away and leaves the text alone, and Tab then is
/// not the hint's.
// no-prop-test: drives the editor; `when_hint` has the properties.
#[gpui_kit::test]
fn escape_leaves_the_text_alone(cx: &mut TestAppContext) {
    let open = typed(cx);
    rest(cx);
    assert!(hint(&open, cx).is_some());
    open.keys(cx, "escape");
    assert_eq!(hint(&open, cx), None);
    assert_eq!(open.text(cx).0, format!("- [ ] film{PHRASE}\nplain"));
    rest(cx);
    assert_eq!(hint(&open, cx), None, "it stays away until the next typing");
    open.keys(cx, "tab");
    assert!(
        !open.text(cx).0.contains('@'),
        "Tab after Escape does not write the date: {:?}",
        open.text(cx)
    );
}

/// After a date the `@` list offers the repeats, narrows as `every w` is
/// typed, and Enter writes the one chosen.
// no-prop-test: drives the editor; `kanso_engine::tasks` has the properties.
#[gpui_kit::test]
fn the_at_list_offers_repeats_after_a_date(cx: &mut TestAppContext) {
    let open = Open::new(cx, "- [ ] film @2026-11-13T15:00");
    open.select(cx, 28..28);
    open.input(cx, " ");
    open.with(cx, |_, window, _| window.refresh());
    let labels = open.with(cx, |editor, _, cx| editor.completion_labels_for_test(cx));
    assert_eq!(
        labels,
        ["Every day", "Every week", "Every weekday", "Every month"]
    );
    open.input(cx, "w");
    open.with(cx, |_, window, _| window.refresh());
    let labels = open.with(cx, |editor, _, cx| editor.completion_labels_for_test(cx));
    assert_eq!(labels, ["Every week", "Every weekday"]);
    open.keys(cx, "enter");
    assert_eq!(open.text(cx).0, "- [ ] film @2026-11-13T15:00 every week");
}

/// Words after the date that are not the start of a repeat close the list.
// no-prop-test: drives the editor; `kanso_engine::tasks` has the properties.
#[gpui_kit::test]
fn the_repeat_list_leaves_other_words_alone(cx: &mut TestAppContext) {
    let open = Open::new(cx, "- [ ] film @2026-11-13");
    open.select(cx, 22..22);
    open.input(cx, " call");
    open.with(cx, |_, window, _| window.refresh());
    let labels = open.with(cx, |editor, _, cx| editor.completion_labels_for_test(cx));
    assert!(labels.is_empty(), "{labels:?}");
    assert_eq!(open.text(cx).0, "- [ ] film @2026-11-13 call");
}

mod chip {
    use kanso_core::dates::{Date, DateTime};
    use kanso_core::repeat::parse_repeat;
    use kanso_core::tasks::Due;

    use crate::due::{chip_label, chip_tooltip, repeat_label, rewrite_for_preview};

    fn now() -> DateTime {
        DateTime {
            date: Date::new(2026, 11, 10).unwrap(),
            hour: 9,
            minute: 0,
            second: 0,
        }
    }

    /// A repeating task's chip ends in the mark, and its tooltip says how
    /// often; a task that does not repeat has neither.
    // no-prop-test: words for a chip; `kanso_engine::tasks` has the
    // properties.
    #[test]
    fn a_repeating_chip_shows_the_mark_and_the_repeat() {
        let due = Due::parse("2026-11-13T15:00").unwrap();
        let (repeat, _) = parse_repeat("every week").unwrap();
        assert_eq!(
            repeat_label(chip_label(due, false, now()), Some(repeat)),
            "Fri 13 Nov 15:00 \u{21bb}"
        );
        assert_eq!(
            chip_tooltip(due, Some(repeat)),
            "Due Friday, 13 November 2026 at 15:00 \u{b7} Repeats every week"
        );
        assert_eq!(
            repeat_label(chip_label(due, false, now()), None),
            "Fri 13 Nov 15:00"
        );
        assert_eq!(
            chip_tooltip(due, None),
            "Due Friday, 13 November 2026 at 15:00"
        );
    }

    /// The preview is handed the repeat inside its chip's link.
    // no-prop-test: delegates to kanso_engine::tasks, tested there.
    #[test]
    fn the_preview_link_carries_the_repeat() {
        let text = "- [ ] film @2026-11-13T15:00 every week".into();
        let out = rewrite_for_preview(&text);
        assert_eq!(
            out.as_ref(),
            "- [ ] film [@2026-11-13T15:00 every week](kanso-due:2026-11-13T15:00;repeat=every_week)"
        );
    }

    /// Live mode's chip takes the repeat's words, and the mark.
    // no-prop-test: delegates to the engine's repeat parser, tested there.
    #[test]
    fn live_mode_chips_take_the_repeat_words() {
        let text = "- [ ] film @2026-11-13T15:00 every week\n- [ ] plain @2026-11-14";
        let chips = crate::live_marks::due_labels(text, now(), &crate::theme::LIGHT);
        assert_eq!(chips.len(), 2);
        assert_eq!(
            &text[chips[0].range.clone()],
            "@2026-11-13T15:00 every week"
        );
        assert_eq!(chips[0].label, "Fri 13 Nov 15:00 \u{21bb}");
        assert_eq!(&text[chips[1].range.clone()], "@2026-11-14");
        assert!(!chips[1].label.contains('\u{21bb}'));
    }
}
