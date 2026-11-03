//! Settings › General › People & Projects: whether Kanso suggests pages,
//! which notebook they are kept in, whether and when AI writes the
//! summaries, and the names turned away for good (`people_view.rs`,
//! `page_view.rs`).

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;
use kanso_core::now_ms;
use kanso_engine::entities::{DEFAULT_NOTEBOOK, suggest};

use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;

/// What the refresh menu offers: the setting's value and its words.
const REFRESH_CHOICES: [(Option<&str>, &str); 3] = [
    (None, "What suits your model"),
    (Some("open"), "When I open an out-of-date page"),
    (Some("press"), "Only when I press Refresh"),
];

/// The words for the refresh setting `value`.
pub fn refresh_label(value: Option<&str>) -> &'static str {
    REFRESH_CHOICES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or(REFRESH_CHOICES[0].1, |(_, label)| label)
}

/// The People & Projects section, for the General page.
// no-prop-test: layout and switches over plain settings; the decisions they
// make are `people_data`'s and the engine's.
pub fn people_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let state = cx.global::<AppState>();
    let settings = state.config.people_projects.clone();
    let store = state.store.clone();

    let books = store.books().unwrap_or_default();
    let kept_in = settings
        .keep_in
        .as_deref()
        .and_then(|id| books.iter().find(|b| b.id == id))
        .map_or_else(|| DEFAULT_NOTEBOOK.to_owned(), |b| b.name.clone());
    let keep_menu = Button::new("people-keep-in")
        .debug_selector(|| "people-keep-in".into())
        .outline()
        .small()
        .label(kept_in)
        .dropdown_caret(true)
        .dropdown_menu({
            let current = settings.keep_in.clone();
            move |mut menu, _, _| {
                menu = menu.item(
                    PopupMenuItem::new(DEFAULT_NOTEBOOK)
                        .checked(current.is_none())
                        .on_click(|_, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.people_projects.keep_in = None
                            });
                        }),
                );
                for book in &books {
                    let id = book.id.clone();
                    menu = menu.item(
                        PopupMenuItem::new(book.name.clone())
                            .checked(current.as_deref() == Some(book.id.as_str()))
                            .on_click(move |_, _, cx| {
                                let id = id.clone();
                                AppState::update_config(cx, |config| {
                                    config.people_projects.keep_in = Some(id)
                                });
                            }),
                    );
                }
                menu
            }
        });

    let refresh_menu = Button::new("people-refresh")
        .debug_selector(|| "people-refresh".into())
        .outline()
        .small()
        .label(refresh_label(settings.refresh.as_deref()))
        .dropdown_caret(true)
        .dropdown_menu({
            let current = settings.refresh.clone();
            move |mut menu, _, _| {
                for (value, label) in REFRESH_CHOICES {
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(current.as_deref() == value)
                            .on_click(move |_, _, cx| {
                                AppState::update_config(cx, |config| {
                                    config.people_projects.refresh = value.map(str::to_owned)
                                });
                            }),
                    );
                }
                menu
            }
        });

    let mut rows = vec![
        Row::new(Lucide::Users, "Suggest people and projects")
            .detail("Looks for names that keep coming up in your notes, on this computer only. Nothing becomes a page until you say so.")
            .trailing(div().debug_selector(|| "people-suggest".into()).child(
                Switch::new("people-suggest")
                    .checked(settings.suggest)
                    .on_click(|on, _, cx| {
                        AppState::update_config(cx, |config| {
                            config.people_projects.suggest = *on
                        });
                    }),
            )),
        Row::new(Lucide::FolderOpen, "Keep pages in")
            .detail("The notebook new person and project pages go in.")
            .trailing(keep_menu),
        Row::new(Lucide::Sparkles, "Write summaries with AI")
            .detail("Off keeps the computed sections only. Every line the model writes cites the notes it came from.")
            .trailing(div().debug_selector(|| "people-write-ai".into()).child(
                Switch::new("people-write-ai")
                    .checked(settings.write_with_ai)
                    .on_click(|on, _, cx| {
                        AppState::update_config(cx, |config| {
                            config.people_projects.write_with_ai = *on
                        });
                    }),
            )),
        Row::new(Lucide::RefreshCw, "Refresh AI sections")
            .detail("When the summaries are written again.")
            .trailing(refresh_menu),
        Row::new(Lucide::Sparkles, "Use AI to find people and projects")
            .detail("Once a day at most, your model labels the suggestions as people, projects or neither. It never makes a page.")
            .trailing(div().debug_selector(|| "people-detect-ai".into()).child(
                Switch::new("people-detect-ai")
                    .checked(settings.detect_with_ai)
                    .on_click(|on, _, cx| {
                        AppState::update_config(cx, |config| {
                            config.people_projects.detect_with_ai = *on
                        });
                    }),
            )),
    ];

    // The names turned away for good, each with "Suggest again".
    let dismissed: Vec<String> = store
        .dismissed()
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.kind == "never")
        .map(|d| d.name)
        .collect();
    for (ix, name) in dismissed.into_iter().enumerate() {
        let store = store.clone();
        let again = name.clone();
        rows.push(
            Row::new(Lucide::EyeOff, name.clone())
                .detail("Dismissed: not a person or project.")
                .trailing(
                    Button::new(("people-suggest-again", ix))
                        .outline()
                        .small()
                        .label("Suggest again")
                        .on_click(move |_, _, cx| {
                            let settings = cx.global::<AppState>().config.people_projects.clone();
                            let _ = suggest::suggest_again(&store, &settings, &again, now_ms());
                            cx.refresh_windows();
                        }),
                ),
        );
    }
    section("People & Projects", rows, colors)
}

/// What the notebook refresh menu offers: the setting's value and its words
/// (the setting is the one people's pages use).
const NOTEBOOK_REFRESH_CHOICES: [(Option<&str>, &str); 3] = [
    (None, "What suits your model"),
    (Some("open"), "When I open a notebook, if out of date"),
    (Some("press"), "Only when I press Refresh"),
];

/// The words for the notebook refresh setting `value`.
pub fn notebook_refresh_label(value: Option<&str>) -> &'static str {
    NOTEBOOK_REFRESH_CHOICES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or(NOTEBOOK_REFRESH_CHOICES[0].1, |(_, label)| label)
}

/// The Notebooks section, for the General page: whether notebooks show a
/// homepage, whether AI writes its summary and decisions, and when they
/// are written again.
// no-prop-test: layout and switches over plain settings; the decisions they
// make are `home_data`'s and the engine's.
pub fn notebooks_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let settings = cx.global::<AppState>().config.people_projects.clone();
    let refresh_menu = Button::new("notebook-refresh")
        .debug_selector(|| "notebook-refresh".into())
        .outline()
        .small()
        .label(notebook_refresh_label(settings.refresh.as_deref()))
        .dropdown_caret(true)
        .dropdown_menu({
            let current = settings.refresh.clone();
            move |mut menu, _, _| {
                for (value, label) in NOTEBOOK_REFRESH_CHOICES {
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(current.as_deref() == value)
                            .on_click(move |_, _, cx| {
                                AppState::update_config(cx, |config| {
                                    config.people_projects.refresh = value.map(str::to_owned)
                                });
                            }),
                    );
                }
                menu
            }
        });
    section(
        "Notebooks",
        vec![
            Row::new(Lucide::House, "Show notebook homepages")
                .detail("A notebook opens to what it is about, its key notes, recent activity and open tasks. Off keeps the plain note list.")
                .trailing(div().debug_selector(|| "notebook-homepages".into()).child(
                    Switch::new("notebook-homepages")
                        .checked(settings.notebook_homepages)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.people_projects.notebook_homepages = *on
                            });
                            cx.refresh_windows();
                        }),
                )),
            Row::new(Lucide::Sparkles, "Write notebook summaries with AI")
                .detail("A summary and the decisions in a notebook, every line citing the notes it came from. Never for locked notebooks.")
                .trailing(div().debug_selector(|| "notebook-summaries".into()).child(
                    Switch::new("notebook-summaries")
                        .checked(settings.notebook_summaries)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.people_projects.notebook_summaries = *on
                            });
                        }),
                )),
            Row::new(Lucide::RefreshCw, "Refresh summaries")
                .detail("When summaries are written again. The same setting as for people and projects.")
                .trailing(refresh_menu),
        ],
        colors,
    )
}
