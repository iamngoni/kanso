//! Settings › General › Flashcards (issue #198): the daily limits, the
//! target recall, the daily reminder, the editor's card icons and a reset of
//! the review history. The settings live in the library's store (the limits
//! count answers from every device), not in the app's config file.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;
use kanso_store::CardSettings;

use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;

const NEW_PER_DAY: [u32; 7] = [0, 5, 10, 20, 30, 50, 100];
const REVIEWS_PER_DAY: [u32; 6] = [50, 100, 200, 400, 800, 2000];
const RECALL: [u32; 6] = [80, 85, 90, 93, 95, 97];
const REMINDERS: [&str; 5] = ["08:00", "09:00", "12:00", "18:00", "20:00"];

fn current(cx: &App) -> CardSettings {
    cx.global::<AppState>()
        .store
        .card_settings()
        .unwrap_or_default()
}

fn change(cx: &mut App, edit: impl FnOnce(&mut CardSettings)) {
    let store = cx.global::<AppState>().store.clone();
    let mut settings = store.card_settings().unwrap_or_default();
    edit(&mut settings);
    if let Err(err) = store.set_card_settings(&settings) {
        eprintln!("kanso: flashcard settings: {err}");
    }
    cx.refresh_windows();
}

fn menu<T: Copy + PartialEq + 'static>(
    id: &'static str,
    label: String,
    values: &'static [T],
    shown: impl Fn(T) -> String + Copy + 'static,
    chosen: T,
    apply: impl Fn(&mut CardSettings, T) + Copy + 'static,
) -> impl IntoElement {
    Button::new(id)
        .debug_selector(move || id.into())
        .outline()
        .small()
        .label(label)
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            for &value in values {
                menu = menu.item(
                    PopupMenuItem::new(shown(value))
                        .checked(value == chosen)
                        .on_click(move |_, _, cx| change(cx, |s| apply(s, value))),
                );
            }
            menu
        })
}

/// The Flashcards section, for the General page.
// no-prop-test: layout and controls over the store's plain settings; the
// limits and the recall are clamped and tested in the store.
pub fn flashcards_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let s = current(cx);
    let recall = (s.retention * 100.).round() as u32;
    let reminder = s.reminder.clone();
    let reminder_menu = Button::new("cards-reminder")
        .debug_selector(|| "cards-reminder".into())
        .outline()
        .small()
        .label(reminder.clone().unwrap_or_else(|| "Off".to_owned()))
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            menu = menu.item(
                PopupMenuItem::new("Off")
                    .checked(reminder.is_none())
                    .on_click(|_, _, cx| change(cx, |s| s.reminder = None)),
            );
            for time in REMINDERS {
                menu = menu.item(
                    PopupMenuItem::new(time)
                        .checked(reminder.as_deref() == Some(time))
                        .on_click(move |_, _, cx| change(cx, |s| s.reminder = Some(time.into()))),
                );
            }
            menu
        });
    section(
        "Flashcards",
        vec![
            Row::new(Lucide::Plus, "New cards a day")
                .detail("The most cards you have not seen before that Review offers in a day, counted across your devices.")
                .trailing(menu(
                    "cards-new",
                    s.new_per_day.to_string(),
                    &NEW_PER_DAY,
                    |n| n.to_string(),
                    s.new_per_day,
                    |s, v| s.new_per_day = v,
                )),
            Row::new(Lucide::Clock, "Reviews a day")
                .detail("The most due cards Review offers in a day.")
                .trailing(menu(
                    "cards-reviews",
                    s.reviews_per_day.to_string(),
                    &REVIEWS_PER_DAY,
                    |n| n.to_string(),
                    s.reviews_per_day,
                    |s, v| s.reviews_per_day = v,
                )),
            Row::new(Lucide::Target, "Target recall")
                .detail("How likely you want to be to remember a card when it falls due. Higher means more reviews.")
                .trailing(menu(
                    "cards-recall",
                    format!("{recall}%"),
                    &RECALL,
                    |n| format!("{n}%"),
                    recall,
                    |s, v| s.retention = f64::from(v) / 100.,
                )),
            Row::new(Lucide::Bell, "Daily reminder")
                .detail("A notification at this time saying how many flashcards are due. It never shows a question.")
                .trailing(reminder_menu),
            Row::new(Lucide::Layers, "Show card icons in the editor")
                .detail("Dims the :: and marks the line of a card in the editor.")
                .trailing(div().debug_selector(|| "cards-icons".into()).child(
                    Switch::new("cards-icons")
                        .checked(s.show_icons)
                        .on_click(|on, _, cx| change(cx, |s| s.show_icons = *on)),
                )),
            Row::new(Lucide::Trash, "Reset review history")
                .detail("Every card starts again as new, on all your devices. The cards in your notes stay.")
                .trailing(
                    Button::new("cards-reset")
                        .outline()
                        .small()
                        .label("Reset…")
                        .on_click(|_, window, cx| crate::dialogs::confirm(
                            "Reset review history?",
                            "Every card starts again as new, on all your devices. The cards in your notes stay.",
                            "Reset",
                            window,
                            cx,
                            |_, cx| {
                                let store = cx.global::<AppState>().store.clone();
                                if let Err(err) = store.reset_review_history() {
                                    eprintln!("kanso: reset review history: {err}");
                                }
                                cx.refresh_windows();
                            },
                        )),
                ),
        ],
        colors,
    )
}
