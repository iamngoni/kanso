//! Settings › General › Brief and review: open loops, the morning brief and
//! the weekly review, when they notify, and whether a done loop is marked
//! in the note.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, h_flex};
use gpui_kit::*;
use kanso_engine::brief::BriefSettings;

use crate::notify_support;
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// A time as the stepper shows it.
fn clock_text(hour: u32, minute: u32) -> String {
    format!("{hour:02}:{minute:02}")
}

/// `minutes` of the day moved by `delta`, wrapping around midnight.
pub fn step_minutes(hour: u32, minute: u32, delta: i32) -> (u32, u32) {
    let total = (hour * 60 + minute) as i32 + delta;
    let total = total.rem_euclid(24 * 60) as u32;
    (total / 60, total % 60)
}

fn change(cx: &mut App, change: impl FnOnce(&mut BriefSettings)) {
    AppState::update_config(cx, |config| change(&mut config.briefs));
    crate::briefs::poke(cx);
}

/// A time with a step back and a step forward, 15 minutes each.
fn stepper(
    id: &'static str,
    hour: u32,
    minute: u32,
    colors: &Colors,
    set: fn(&mut BriefSettings, u32, u32),
) -> AnyElement {
    let step = move |delta: i32| {
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            let (h, m) = step_minutes(hour, minute, delta);
            change(cx, |s| set(s, h, m));
        }
    };
    h_flex()
        .gap(px(2.))
        .items_center()
        .child(
            Button::new(SharedString::from(format!("{id}-earlier")))
                .ghost()
                .small()
                .icon(Lucide::ChevronLeft)
                .tooltip("15 minutes earlier")
                .on_click(step(-15)),
        )
        .child(
            div()
                .w(px(48.))
                .text_center()
                .text_size(px(13.))
                .text_color(colors.text)
                .child(clock_text(hour, minute)),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-later")))
                .ghost()
                .small()
                .icon(Lucide::ChevronRight)
                .tooltip("15 minutes later")
                .on_click(step(15)),
        )
        .into_any_element()
}

fn switch(id: &'static str, on: bool, set: fn(&mut BriefSettings, bool)) -> AnyElement {
    Switch::new(id)
        .checked(on)
        .on_click(move |on, _, cx| {
            let on = *on;
            change(cx, |s| set(s, on));
        })
        .into_any_element()
}

/// The section.
// no-prop-test: drawing; `step_minutes` has the test, and the settings snapshots show it.
pub fn briefs_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let s = cx
        .try_global::<AppState>()
        .map(|state| state.config.briefs.clone())
        .unwrap_or_default();
    let permission =
        notify_support::settings_note(notify_support::availability(), crate::flavour::CURRENT.name);
    let day = s.review_day.min(6) as usize;
    let review_day = Button::new("review-day")
        .outline()
        .small()
        .label(WEEKDAYS[day])
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            for (ix, name) in WEEKDAYS.iter().enumerate() {
                menu = menu.item(
                    PopupMenuItem::new(*name)
                        .checked(ix == day)
                        .on_click(move |_, _, cx| change(cx, |s| s.review_day = ix as u32)),
                );
            }
            menu
        });
    let brief_days = Button::new("brief-days")
        .outline()
        .small()
        .label(if s.brief_weekdays_only {
            "Weekdays"
        } else {
            "Every day"
        })
        .dropdown_caret(true)
        .dropdown_menu(move |menu, _, _| {
            menu.item(
                PopupMenuItem::new("Every day")
                    .checked(!s.brief_weekdays_only)
                    .on_click(|_, _, cx| change(cx, |s| s.brief_weekdays_only = false)),
            )
            .item(
                PopupMenuItem::new("Weekdays")
                    .checked(s.brief_weekdays_only)
                    .on_click(|_, _, cx| change(cx, |s| s.brief_weekdays_only = true)),
            )
        });
    section(
        "Brief and review",
        vec![
            Row::new(Lucide::CircleDot, "Open loops")
                .detail("Finds things your notes say you will do, wait for or check, like “I'll send the deck” or “waiting on Rudo”. Nothing leaves this Mac.")
                .trailing(switch("loops", s.loops, |s, on| s.loops = on)),
            Row::new(Lucide::Clock, "Morning brief")
                .detail(
                    permission
                        .clone()
                        .unwrap_or_else(|| "A notification each morning with how much is due and open. Clicking it opens the brief.".to_owned()),
                )
                .trailing(brief_days)
                .trailing(stepper(
                    "brief-time",
                    s.brief_hour,
                    s.brief_minute,
                    colors,
                    |s, h, m| {
                        s.brief_hour = h;
                        s.brief_minute = m;
                    },
                ))
                .trailing(switch("brief", s.brief, |s, on| s.brief = on)),
            Row::new(Lucide::CalendarDays, "Weekly review")
                .detail("A notification once a week. Clicking it opens the review of the last seven days.")
                .trailing(review_day)
                .trailing(stepper(
                    "review-time",
                    s.review_hour,
                    s.review_minute,
                    colors,
                    |s, h, m| {
                        s.review_hour = h;
                        s.review_minute = m;
                    },
                ))
                .trailing(switch("review", s.review, |s, on| s.review = on)),
            Row::new(Lucide::Bell, "Notify me of the brief")
                .detail("Shows the notification at the time above. Turn it off to open the brief yourself.")
                .trailing(switch("notify-brief", s.notify_brief, |s, on| {
                    s.notify_brief = on
                })),
            Row::new(Lucide::Bell, "Notify me of the review")
                .trailing(switch("notify-review", s.notify_review, |s, on| {
                    s.notify_review = on
                })),
            Row::new(Lucide::Check, "Mark done loops in the note")
                .detail("Adds a tick after a sentence when you mark its loop done. Off, Kanso remembers it without touching your text.")
                .trailing(switch("mark-done", s.mark_done_in_note, |s, on| {
                    s.mark_done_in_note = on
                })),
        ],
        colors,
    )
}

#[cfg(test)]
mod tests {
    use super::step_minutes;
    use proptest::prelude::*;

    proptest! {
        /// Stepping a time stays a time, and stepping back undoes it.
        #[test]
        fn prop_steps_wrap_and_undo(h in 0u32..24, m in 0u32..60, delta in -600i32..600) {
            let (nh, nm) = step_minutes(h, m, delta);
            prop_assert!(nh < 24 && nm < 60);
            prop_assert_eq!(step_minutes(nh, nm, -delta), (h, m));
        }
    }

    #[test]
    fn midnight_wraps() {
        assert_eq!(step_minutes(23, 50, 15), (0, 5));
        assert_eq!(step_minutes(0, 5, -15), (23, 50));
    }
}
