//! Settings › General › Inbox: the Triage bar above the Inbox's notes and
//! which suggestions a Triage card makes (`triage_view.rs`).

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use crate::config::MAX_TAGS_LIMIT;
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;

/// The Inbox section, for the General page.
// no-prop-test: layout and switches over plain settings.
pub fn inbox_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let inbox = cx.global::<AppState>().config.inbox;
    let tags_menu = Button::new("inbox-max-tags")
        .debug_selector(|| "inbox-max-tags".into())
        .outline()
        .small()
        .label(inbox.max_tags.to_string())
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            for n in 1..=MAX_TAGS_LIMIT {
                menu = menu.item(
                    PopupMenuItem::new(n.to_string())
                        .checked(inbox.max_tags == n)
                        .on_click(move |_, _, cx| {
                            AppState::update_config(cx, |config| config.inbox.max_tags = n);
                        }),
                );
            }
            menu
        });
    section(
        "Inbox",
        vec![
            Row::new(Lucide::Inbox, "Show Triage bar")
                .detail("Above the Inbox's notes: how many wait to be filed, and a button to go through them.")
                .trailing(div().debug_selector(|| "inbox-bar".into()).child(
                    Switch::new("inbox-bar")
                        .checked(inbox.show_triage_bar)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.inbox.show_triage_bar = *on
                            });
                        }),
                )),
            Row::new(Lucide::Heading, "Suggest titles")
                .detail("Offers the first sentence as the title when a note's own is empty, a date, “Untitled” or cut short.")
                .trailing(div().debug_selector(|| "inbox-titles".into()).child(
                    Switch::new("inbox-titles")
                        .checked(inbox.suggest_titles)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.inbox.suggest_titles = *on
                            });
                        }),
                )),
            Row::new(Lucide::Link, "Suggest links")
                .detail("Offers links to notes the text names, and to near notes, as a Related line at the end of the note.")
                .trailing(div().debug_selector(|| "inbox-links".into()).child(
                    Switch::new("inbox-links")
                        .checked(inbox.suggest_links)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.inbox.suggest_links = *on
                            });
                        }),
                )),
            Row::new(Lucide::Tag, "Maximum tags per note")
                .detail("The most tags a card suggests for one note.")
                .trailing(tags_menu),
        ],
        colors,
    )
}
