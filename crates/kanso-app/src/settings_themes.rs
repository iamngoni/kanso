//! The Themes page of Settings: three grids of cards, one
//! for the UI theme, one for the editor's syntax and one for the preview,
//! each card a small live picture of its theme with the chosen one
//! outlined. Following the system's light and dark mode outlines a light
//! and a dark UI theme at once, and a click sets the one of its kind.
//! Below them: the themes folder, Reload, Import, and any theme file that
//! couldn't be used. The Icon Themes page is here too: the icon theme
//! chooser, each theme with a row of its icons, and the icon themes folder.

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::config::{ThemePreference, ThemeSettings};
use crate::icon_themes::{self, IconTheme, IconThemes};
use crate::origin::Origin;
use crate::settings::{Row, SettingsView, section};
use crate::theme::{Colors, Rgb, SYSTEM_MONO, hsla};
use crate::theme_file::{PreviewColors, TokenStyle, fit_preview, fit_syntax};
use crate::themes::{self, ThemeEntry, Themes, themes_dir};
use crate::workspace::AppState;
use crate::zoom::px;

/// A card's picture, and the room between cards.
pub const CARD_WIDTH: f32 = 160.;
pub const THUMB_HEIGHT: f32 = 100.;
const CARD_GAP: f32 = 16.;

/// Which of the three choices a grid sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Ui,
    Syntax,
    Preview,
}

/// Whether `name` is outlined in `kind`'s grid, and the badge it wears when
/// the system is followed ("Light" or "Dark"). `None` in the syntax or
/// preview grid stands for "Same as the UI theme".
// no-prop-test: three cases over the settings; `prop_a_click_selects_its_card`
// checks them against `pick`.
pub fn selected(
    settings: &ThemeSettings,
    kind: Kind,
    name: Option<&str>,
) -> (bool, Option<&'static str>) {
    match kind {
        Kind::Ui if settings.follow_system => {
            let name = name.unwrap_or_default();
            match (settings.light == name, settings.dark == name) {
                (true, true) => (true, Some("Light & Dark")),
                (true, false) => (true, Some("Light")),
                (false, true) => (true, Some("Dark")),
                (false, false) => (false, None),
            }
        }
        Kind::Ui => (Some(settings.ui.as_str()) == name, None),
        Kind::Syntax => (settings.syntax.as_deref() == name, None),
        Kind::Preview => (settings.preview.as_deref() == name, None),
    }
}

/// The settings after a click on `name`'s card in `kind`'s grid. A UI
/// theme picked while the system is followed takes the slot of its own
/// appearance, and becomes the fixed choice too, for when following stops.
pub fn pick(settings: &ThemeSettings, kind: Kind, name: Option<&str>, dark: bool) -> ThemeSettings {
    let mut next = settings.clone();
    match kind {
        Kind::Ui => {
            let name = name.unwrap_or(&settings.ui).to_owned();
            if settings.follow_system {
                if dark {
                    next.dark = name.clone();
                } else {
                    next.light = name.clone();
                }
            }
            next.ui = name;
        }
        Kind::Syntax => next.syntax = name.map(str::to_owned),
        Kind::Preview => next.preview = name.map(str::to_owned),
    }
    next
}

/// Selects an installed theme by name, for Browse's Use: the same change a
/// click on an Installed card makes, without needing a window.
pub fn select_installed(name: &str, dark: bool, cx: &mut App) {
    let next = pick(
        &cx.global::<AppState>().config.appearance.themes,
        Kind::Ui,
        Some(name),
        dark,
    );
    let legacy = if next.follow_system {
        ThemePreference::System
    } else if dark {
        ThemePreference::Dark
    } else {
        ThemePreference::Light
    };
    let changed = cx.global::<AppState>().config.appearance.themes != next;
    AppState::update_config(cx, |config| {
        config.appearance.themes = next;
        config.appearance.theme = legacy;
    });
    if changed {
        crate::metrics::record(cx, crate::metrics::Feature::ThemeChanged);
    }
    themes::apply(None, cx);
}

/// Changes the theme settings, saves them, and shows the result everywhere.
fn update(window: &mut Window, cx: &mut App, change: impl FnOnce(&ThemeSettings) -> ThemeSettings) {
    let next = change(&cx.global::<AppState>().config.appearance.themes);
    let legacy = if next.follow_system {
        ThemePreference::System
    } else {
        let dark = cx
            .try_global::<Themes>()
            .and_then(|t| t.get(&next.ui).map(|e| e.dark()))
            .unwrap_or(true);
        if dark {
            ThemePreference::Dark
        } else {
            ThemePreference::Light
        }
    };
    let changed = cx.global::<AppState>().config.appearance.themes != next;
    AppState::update_config(cx, |config| {
        config.appearance.themes = next;
        config.appearance.theme = legacy;
    });
    if changed {
        crate::metrics::record(cx, crate::metrics::Feature::ThemeChanged);
    }
    themes::apply(Some(window), cx);
}

/// Reads the themes folder again, and says what couldn't be used.
pub fn reload(window: &mut Window, cx: &mut App) {
    let notices = themes::reload(cx);
    let count = cx.global::<Themes>().entries.len();
    let message = match notices.len() {
        0 => Notification::success(format!("{count} themes ready.")),
        1 => Notification::warning(notices[0].sentence()),
        n => Notification::warning(format!(
            "{n} theme files couldn't be used; the Themes page lists them."
        )),
    };
    window.push_notification(message, cx);
}

/// Asks for a theme package (a folder or a .tgz) or a Zed theme (.json),
/// converts it into the themes folder, and reloads.
fn import(window: &mut Window, cx: &mut App) {
    let dir = themes_dir(&cx.global::<AppState>().data_dir);
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: true,
        multiple: false,
        prompt: Some("Import Theme".into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(source) = paths.into_iter().next() else {
                return;
            };
            let result = cx
                .background_executor()
                .spawn(async move { crate::theme_import::import_path(&source, &dir) })
                .await;
            cx.update(|window, cx| {
                themes::reload(cx);
                let message = match result {
                    Ok(names) if names.len() == 1 => {
                        Notification::success(format!("Imported {}.", names[0]))
                    }
                    Ok(names) => Notification::success(format!(
                        "Imported {} themes: {}.",
                        names.len(),
                        names.join(", ")
                    )),
                    Err(err) => Notification::error(format!("Couldn't import the theme: {err}")),
                };
                window.push_notification(message, cx);
            })
            .ok();
        })
        .detach();
}

/// The page.
// no-prop-test: layout; the snapshots show it in both themes.
pub fn page(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    if crate::settings_gallery::tab_for(kanso_gallery::Kind::Color, cx)
        == crate::settings_gallery::Tab::Browse
    {
        return crate::settings_gallery::browse(kanso_gallery::Kind::Color, cx);
    }
    let colors = Colors::get(cx);
    let state = cx.global::<AppState>();
    let settings = state.config.appearance.themes.clone();
    let dir = themes_dir(&state.data_dir);
    let auto_update = state.config.gallery.auto_update;
    let Some(themes) = cx.try_global::<Themes>() else {
        return Vec::new();
    };
    let entries = themes.entries.clone();
    let notices = themes.notices.clone();
    let active = cx.try_global::<themes::ActiveTheme>().map(|a| a.ui.clone());

    let ui_cards = entries
        .iter()
        .map(|e| card(Kind::Ui, Some(e), &settings, colors, ui_thumb(e)))
        .collect();
    let same = active.clone().unwrap_or_else(|| entries[0].clone());
    let syntax_cards = std::iter::once(card(
        Kind::Syntax,
        None,
        &settings,
        colors,
        syntax_thumb(&same),
    ))
    .chain(
        entries
            .iter()
            .map(|e| card(Kind::Syntax, Some(e), &settings, colors, syntax_thumb(e))),
    )
    .collect();
    let preview_cards = std::iter::once(card(
        Kind::Preview,
        None,
        &settings,
        colors,
        preview_thumb(&same),
    ))
    .chain(
        entries
            .iter()
            .map(|e| card(Kind::Preview, Some(e), &settings, colors, preview_thumb(e))),
    )
    .collect();

    let follow = settings.follow_system;
    // Above the theme grids, so it stays reachable as families are added.
    let follow_row = section(
        "",
        vec![
            Row::new(
                Lucide::SunMoon,
                "Switch between a light and a dark theme with the system",
            )
            .detail(if follow {
                "Showing the light theme in light mode and the dark one in dark mode."
            } else {
                "Off: the theme stays as picked whatever the system's mode."
            })
            .trailing(
                div()
                    .debug_selector(|| "themes-follow-system".into())
                    .child(
                        Switch::new("themes-follow-system")
                            .checked(follow)
                            .on_click(|on, window, cx| {
                                let on = *on;
                                update(window, cx, |s| ThemeSettings {
                                    follow_system: on,
                                    ..s.clone()
                                })
                            }),
                    ),
            ),
        ],
        colors,
    );
    let mut pages = vec![
        follow_row,
        grid_section(
            "Theme",
            if follow {
                "Colors the sidebar, the note list, menus and every control. Pick one light and one dark theme; Kanso switches between them with the system."
            } else {
                "Colors the sidebar, the note list, menus and every control."
            },
            ui_cards,
            colors,
        ),
        grid_section(
            "Editor syntax",
            "Colors the Markdown source and the code in it.",
            syntax_cards,
            colors,
        ),
        grid_section(
            "Preview",
            "Colors the rendered note: headings, links, quotes and code.",
            preview_cards,
            colors,
        ),
    ];
    let mut rows = vec![
        Row::new(Lucide::FolderOpen, "Themes folder")
            .detail("Add theme files here; Kanso picks up changes on its own.")
            .trailing(
                Button::new("open-themes-folder")
                    .outline()
                    .small()
                    .label("Open Folder")
                    .on_click(move |_, _, cx| {
                        std::fs::create_dir_all(&dir).ok();
                        cx.open_with_system(&dir);
                    }),
            )
            .trailing(
                Button::new("reload-themes")
                    .outline()
                    .small()
                    .label("Reload")
                    .debug_selector(|| "reload-themes".into())
                    .on_click(|_, window, cx| reload(window, cx)),
            ),
        Row::new(Lucide::RefreshCw, "Keep gallery themes up to date")
            .detail("Checks the signed gallery on Kanso's update timer. Off, Refresh still works.")
            .trailing(
                div()
                    .debug_selector(|| "themes-gallery-auto-update".into())
                    .child(
                        Switch::new("themes-gallery-auto-update")
                            .checked(auto_update)
                            .on_click(|on, _, cx| {
                                let on = *on;
                                AppState::update_config(cx, |config| {
                                    config.gallery.auto_update = on;
                                });
                            }),
                    ),
            ),
        Row::new(Lucide::Download, "Import theme")
            .detail("A theme package (its folder or .tgz) or a Zed theme (.json), converted into the themes folder.")
            .trailing(
                Button::new("import-theme")
                    .outline()
                    .small()
                    .label("Import Theme…")
                    .on_click(|_, window, cx| import(window, cx)),
            ),
    ];
    for (ix, notice) in notices.iter().enumerate() {
        rows.push(
            Row::new(Lucide::TriangleAlert, notice.file.clone())
                .detail(notice.sentence())
                .trailing(div().id(("theme-notice", ix))),
        );
    }
    pages.push(section("Theme files", rows, colors));
    pages
}

/// The list-scrollbar choice, kept on the Appearance page above its theme gallery.
pub fn scrollbars_row(cx: &mut Context<SettingsView>) -> AnyElement {
    let colors = Colors::get(cx);
    let always = cx
        .global::<AppState>()
        .config
        .appearance
        .always_show_scrollbars;
    section(
        "",
        vec![
            Row::new(Lucide::PanelRight, "Show scrollbars")
                .detail(if always {
                    "Always show scrollbars when a list has more content."
                } else {
                    "Show scrollbars while scrolling or hovering."
                })
                .trailing(
                    Button::new("scrollbar-visibility")
                        .outline()
                        .small()
                        .label(if always { "Always" } else { "When scrolling" })
                        .debug_selector(|| "scrollbar-visibility".into())
                        .on_click(|_, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.appearance.always_show_scrollbars =
                                    !config.appearance.always_show_scrollbars;
                            });
                            themes::apply(None, cx);
                        }),
                ),
        ],
        colors,
    )
    .into_any_element()
}

/// Settings › Icon Themes: the icon themes' cards and their files.
pub fn icon_theme_page(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    if crate::settings_gallery::tab_for(kanso_gallery::Kind::Icons, cx)
        == crate::settings_gallery::Tab::Browse
    {
        return crate::settings_gallery::browse(kanso_gallery::Kind::Icons, cx);
    }
    vec![icon_theme_section(cx)]
}

/// Reads the icon themes folder again, off the UI thread, and says what
/// couldn't be used.
pub fn reload_icon_themes(window: &mut Window, cx: &mut App) {
    let reload = icon_themes::reload_soon(cx);
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let notices = reload.await;
        cx.update(|cx| {
            let count = cx.global::<IconThemes>().entries.len();
            let message = match notices.len() {
                0 => Notification::success(format!("{count} icon themes ready.")),
                1 => Notification::warning(notices[0].sentence()),
                n => Notification::warning(format!(
                    "{n} icon theme files couldn't be used; the Icon Themes page lists them."
                )),
            };
            handle
                .update(cx, |_, window, cx| window.push_notification(message, cx))
                .ok();
        });
    })
    .detach();
}

/// Asks for an icon theme to import (a Zed icon theme, a theme package
/// carrying one, or a Kanso icon theme folder), converts it into the icon
/// themes folder off the UI thread, reloads, and says what came across.
pub fn import_icon_theme(window: &mut Window, cx: &mut App) {
    let dir = icon_themes::icon_themes_dir(&cx.global::<AppState>().data_dir);
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: true,
        multiple: false,
        prompt: Some("Import Icon Theme".into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(source) = paths.into_iter().next() else {
                return;
            };
            let result = cx
                .background_executor()
                .spawn(async move { crate::icon_theme_import::import_path(&source, &dir) })
                .await;
            cx.update(|window, cx| {
                icon_themes::reload(cx);
                let message = match result {
                    Ok(imports) => Notification::success(imports.summary()),
                    Err(err) => {
                        Notification::error(format!("Couldn't import the icon theme: {err}"))
                    }
                };
                window.push_notification(message, cx);
            })
            .ok();
        })
        .detach();
}

/// What a theme's card says about it, under its name. Pure.
pub fn icon_theme_detail(theme: &IconTheme) -> String {
    let mut parts = Vec::new();
    parts.push(match theme.origin {
        Origin::Builtin => "Built in".to_owned(),
        Origin::Own => "In your icon themes folder".to_owned(),
        Origin::Gallery => "From the gallery".to_owned(),
    });
    if !theme.author.is_empty() {
        parts.push(format!("by {}", theme.author));
    }
    if !theme.license.is_empty() {
        parts.push(theme.license.clone());
    }
    let drawn = theme.icons.len() + theme.library.len();
    parts.push(match drawn {
        0 => "Kanso's own icons".to_owned(),
        1 => "draws 1 icon".to_owned(),
        n => format!("draws {n} icons"),
    });
    match theme.extras.len() {
        0 => {}
        1 => parts.push("1 more for notebooks".to_owned()),
        n => parts.push(format!("{n} more for notebooks")),
    }
    parts.join(" · ")
}

/// The icon theme chooser: a card for each theme with a row of its icons,
/// the chosen one outlined; then the folder, Import, and what couldn't be
/// used.
fn icon_theme_section(cx: &mut Context<SettingsView>) -> AnyElement {
    let colors = Colors::get(cx);
    let dir = icon_themes::icon_themes_dir(&cx.global::<AppState>().data_dir);
    let chosen = cx.global::<AppState>().config.appearance.icon_theme.clone();
    let active = icon_themes::active(cx);
    let (entries, notices) = cx
        .try_global::<IconThemes>()
        .map(|themes| (themes.entries.clone(), themes.notices.clone()))
        .unwrap_or_default();
    let cards = entries
        .iter()
        .map(|theme| icon_theme_card(theme, theme.key == active.key, colors))
        .collect();
    let mut rows = vec![
        Row::new(Lucide::FolderOpen, "Icon themes folder")
            .detail("Add a folder holding an icon theme's .json and its SVGs; Kanso picks up changes on its own.")
            .trailing(
                Button::new("open-icon-themes-folder")
                    .outline()
                    .small()
                    .label("Open Folder")
                    .on_click(move |_, _, cx| {
                        std::fs::create_dir_all(&dir).ok();
                        cx.open_with_system(&dir);
                    }),
            )
            .trailing(
                Button::new("reload-icon-themes")
                    .outline()
                    .small()
                    .label("Reload")
                    .on_click(|_, window, cx| reload_icon_themes(window, cx)),
            ),
        Row::new(Lucide::Download, "Import icon theme")
            .detail("A Zed icon theme (its extension folder or .tar.gz), a theme package carrying one, or a Kanso icon theme folder, converted into the icon themes folder.")
            .trailing(
                Button::new("import-icon-theme")
                    .outline()
                    .small()
                    .label("Import…")
                    .debug_selector(|| "import-icon-theme".into())
                    .on_click(|_, window, cx| import_icon_theme(window, cx)),
            ),
    ];
    // A chosen theme that has gone draws Kanso's own icons meanwhile.
    if let Some(id) = &chosen
        && !entries.iter().any(|theme| &theme.id == id)
    {
        rows.push(Row::new(Lucide::TriangleAlert, id.clone()).detail(
            "The chosen icon theme isn't in the icon themes folder; Kanso's own icons show until it is.",
        ));
    }
    for (ix, notice) in notices.iter().enumerate() {
        rows.push(
            Row::new(Lucide::TriangleAlert, notice.file.clone())
                .detail(notice.sentence())
                .trailing(div().id(("icon-theme-notice", ix))),
        );
    }
    v_flex()
        .child(grid_section(
            "Icon theme",
            "Draws every icon: the sidebar, the toolbar, menus and every control. An icon a theme leaves out stays Kanso's own.",
            cards,
            colors,
        ))
        .child(section("Icon theme files", rows, colors))
        .into_any_element()
}

/// A theme's card: a row of its icons, outlined when chosen, and its name.
fn icon_theme_card(theme: &Arc<IconTheme>, chosen: bool, colors: &Colors) -> AnyElement {
    let icons = icon_themes::card_icons(theme).into_iter().map(|drawn| {
        Icon::default()
            .path(icon_themes::unresolved(drawn))
            .size(px(16.))
            .text_color(colors.text_secondary)
    });
    let id = theme.id.clone();
    let selector = format!("icon-theme-{id}");
    let detail: SharedString = icon_theme_detail(theme).into();
    v_flex()
        .id(SharedString::from(selector.clone()))
        .w(px(CARD_WIDTH * 2. + CARD_GAP))
        .gap(px(6.))
        .cursor_pointer()
        .debug_selector(move || selector.clone())
        .child(
            div()
                .p(px(2.))
                .rounded(px(9.))
                .border_2()
                .border_color(if chosen {
                    Hsla::from(colors.accent)
                } else {
                    transparent_black()
                })
                .child(
                    h_flex()
                        .h(px(44.))
                        .px(px(14.))
                        .gap(px(12.))
                        .justify_center()
                        .rounded(px(6.))
                        .bg(colors.editor)
                        .border_1()
                        .border_color(colors.divider)
                        .children(icons),
                ),
        )
        .child(
            h_flex()
                .justify_center()
                .text_size(px(12.))
                .line_height(px(16.))
                .text_color(if chosen {
                    colors.text
                } else {
                    colors.text_secondary
                })
                .when(chosen, |el| el.font_medium())
                .child(div().truncate().child(theme.name.clone())),
        )
        .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
        .on_click(move |_, _, cx| icon_themes::choose(Some(id.clone()), cx))
        .into_any_element()
}

/// A titled grid of cards.
fn grid_section(
    title: &'static str,
    detail: &'static str,
    cards: Vec<AnyElement>,
    colors: &Colors,
) -> AnyElement {
    v_flex()
        .mt(px(26.))
        .child(
            div()
                .text_size(px(14.))
                .line_height(px(20.))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(title),
        )
        .child(
            div()
                .mb(px(12.))
                .text_size(px(12.5))
                .line_height(px(17.))
                .text_color(colors.text_muted)
                .child(detail),
        )
        .child(h_flex().flex_wrap().gap(px(CARD_GAP)).children(cards))
        .into_any_element()
}

/// One theme's card: its picture, outlined when chosen, and its name.
fn card(
    kind: Kind,
    entry: Option<&Arc<ThemeEntry>>,
    settings: &ThemeSettings,
    colors: &Colors,
    thumb: Div,
) -> AnyElement {
    let name = entry.map(|e| e.name().to_owned());
    let (chosen, badge) = selected(settings, kind, name.as_deref());
    let label: SharedString = match &name {
        Some(name) => name.clone().into(),
        None => "Same as theme".into(),
    };
    let dark = entry.is_some_and(|e| e.dark());
    let origin: SharedString = match entry {
        Some(e) => match (e.origin, &e.path, e.file.author.as_str()) {
            (Origin::Own, Some(path), _) => format!(
                "{} in your themes folder",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            (origin, _, "") => origin.detail().to_owned(),
            (origin, _, author) => format!("{}, by {author}", origin.detail()),
        },
        None => "The UI theme's own".to_owned(),
    }
    .into();
    let id = format!(
        "theme-{}-{}",
        match kind {
            Kind::Ui => "ui",
            Kind::Syntax => "syntax",
            Kind::Preview => "preview",
        },
        entry
            .map(|e| e.id.as_str())
            .unwrap_or("same")
            .to_lowercase()
            .replace(' ', "-")
    );
    let selector = id.clone();
    v_flex()
        .id(SharedString::from(id))
        .w(px(CARD_WIDTH))
        .gap(px(6.))
        .cursor_pointer()
        .debug_selector(move || selector.clone())
        .child(
            div()
                .p(px(2.))
                .rounded(px(9.))
                .border_2()
                .border_color(if chosen {
                    Hsla::from(colors.accent)
                } else {
                    transparent_black()
                })
                .child(
                    thumb
                        .w_full()
                        .h(px(THUMB_HEIGHT))
                        .rounded(px(6.))
                        .overflow_hidden()
                        .border_1()
                        .border_color(colors.divider),
                ),
        )
        .child(
            h_flex()
                .justify_center()
                .gap(px(6.))
                .text_size(px(12.))
                .line_height(px(16.))
                .text_color(if chosen {
                    colors.text
                } else {
                    colors.text_secondary
                })
                .when(chosen, |el| el.font_medium())
                .child(div().truncate().child(label))
                .when_some(badge, |el, badge| {
                    el.child(
                        div()
                            .flex_none()
                            .px(px(5.))
                            .rounded(px(4.))
                            .bg(colors.fill)
                            .text_size(px(10.5))
                            .text_color(colors.text_secondary)
                            .child(badge),
                    )
                }),
        )
        .tooltip(move |window, cx| Tooltip::new(origin.clone()).build(window, cx))
        .on_click(move |_, window, cx| {
            let name = name.clone();
            update(window, cx, |s| pick(s, kind, name.as_deref(), dark));
        })
        .into_any_element()
}

/// A rounded bar standing in for a line of text.
fn bar(width: f32, height: f32, color: Rgb) -> Div {
    div()
        .flex_none()
        .w(px(width))
        .h(px(height))
        .rounded(px(height / 2.))
        .bg(color)
}

/// A UI theme's picture: a small window with its sidebar, its note list
/// with one note selected, and a note in the editor.
pub(crate) fn ui_thumb(entry: &ThemeEntry) -> Div {
    let c = &Colors::from_engine(&entry.resolved.colors);
    let sidebar = v_flex()
        .w(px(34.))
        .h_full()
        .flex_none()
        .bg(c.sidebar)
        .px(px(5.))
        .pt(px(12.))
        .gap(px(5.))
        .child(
            h_flex().gap(px(2.)).children(
                [0xFF5F57, 0xFEBC2E, 0x28C840]
                    .map(|dot| div().size(px(4.)).rounded_full().bg(Rgb(dot))),
            ),
        )
        .child(bar(16., 2.5, c.text_muted))
        .child(
            div()
                .rounded(px(2.))
                .bg(c.sidebar_active)
                .px(px(2.))
                .py(px(2.))
                .child(bar(18., 2.5, c.text)),
        )
        .child(bar(14., 2.5, c.text_secondary))
        .child(bar(17., 2.5, c.text_secondary))
        .child(bar(12., 2.5, c.text_secondary));
    let row = |selected: bool, w: f32| {
        v_flex()
            .px(px(4.))
            .py(px(4.))
            .gap(px(3.))
            .when(selected, |el| el.bg(c.row_active))
            .child(bar(w, 3., c.text))
            .child(bar(w - 6., 2., c.text_muted))
    };
    let list = v_flex()
        .w(px(44.))
        .h_full()
        .flex_none()
        .bg(c.list)
        .border_l_1()
        .border_color(c.divider)
        .pt(px(12.))
        .child(row(true, 30.))
        .child(row(false, 26.))
        .child(row(false, 32.))
        .child(row(false, 22.));
    let editor = v_flex()
        .flex_1()
        .h_full()
        .bg(c.editor)
        .border_l_1()
        .border_color(c.divider)
        .px(px(8.))
        .pt(px(14.))
        .gap(px(5.))
        .child(bar(40., 4., c.text))
        .child(bar(56., 2.5, c.text_secondary))
        .child(
            h_flex()
                .gap(px(3.))
                .child(bar(18., 2.5, c.text_secondary))
                .child(bar(16., 2.5, c.link)),
        )
        .child(bar(48., 2.5, c.text_secondary))
        .child(
            h_flex()
                .gap(px(3.))
                .child(
                    div()
                        .px(px(4.))
                        .py(px(2.))
                        .rounded(px(2.))
                        .bg(c.button)
                        .child(bar(10., 2., c.button_text)),
                )
                .child(bar(14., 2.5, c.accent)),
        );
    h_flex().bg(c.app).child(sidebar).child(list).child(editor)
}

/// A few lines of Markdown, set in a syntax theme's colors on its own editor.
pub(crate) fn syntax_thumb(entry: &ThemeEntry) -> Div {
    let theme = &entry.resolved;
    let surface = theme.colors.editor;
    let s = fit_syntax(&theme.syntax, surface, theme.colors.text);
    let style = |key: &str| s.get(key).copied().unwrap_or_default();
    let text = style("text");
    let span = |words: &'static str, key: &str| run(words, style(key).or_text(text));
    let line = |runs: Vec<Div>| h_flex().children(runs);
    v_flex()
        .bg(hsla(surface))
        .px(px(9.))
        .pt(px(9.))
        .gap(px(2.))
        .font_family(SYSTEM_MONO)
        .text_size(px(9.))
        .line_height(px(13.))
        .child(line(vec![
            span("# ", "block_marker"),
            span("Heading", "heading"),
        ]))
        .child(line(vec![
            span("**", "marker"),
            span("bold", "strong"),
            span("**", "marker"),
            span(" and ", "text"),
            span("_", "marker"),
            span("italic", "emphasis"),
            span("_", "marker"),
        ]))
        .child(line(vec![
            span("`code`", "code"),
            span(", ", "text"),
            span("[", "marker"),
            span("a link", "link"),
            span("](", "marker"),
            span("url", "url"),
            span(")", "marker"),
        ]))
        .child(line(vec![
            span("- ", "list_marker"),
            span("[x] ", "task"),
            span("done", "text"),
        ]))
        .child(line(vec![
            span("> ", "block_marker"),
            span("a quote", "quote"),
        ]))
        .child(line(vec![
            span("let ", "keyword"),
            span("n", "variable"),
            span(" = ", "operator"),
            span("\"hi\"", "string"),
            span("; ", "punctuation"),
            span("// 42", "comment"),
        ]))
}

trait OrText {
    fn or_text(self, text: TokenStyle) -> TokenStyle;
}

impl OrText for TokenStyle {
    fn or_text(self, text: TokenStyle) -> TokenStyle {
        TokenStyle {
            color: self.color.or(text.color),
            ..self
        }
    }
}

/// Words in a token's style.
fn run(words: &'static str, style: TokenStyle) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .when_some(style.color, |el, c| el.text_color(hsla(c)))
        .when(style.italic == Some(true), |el| el.italic())
        .when_some(style.weight, |el, w| {
            el.font_weight(FontWeight(f32::from(w)))
        })
        .child(words)
}

/// A preview theme's picture: a heading, a line with code, a quote and a
/// code block, on its own editor surface.
pub(crate) fn preview_thumb(entry: &ThemeEntry) -> Div {
    let theme = &entry.resolved;
    let p: PreviewColors = fit_preview(&theme.preview.into(), &Colors::from_engine(&theme.colors));
    let code = fit_syntax(&theme.syntax, p.code_background.into(), theme.colors.text);
    let token = |key: &str| code.get(key).copied().unwrap_or_default();
    let code_text = TokenStyle {
        color: Some(p.code_text.unwrap_or(p.text).into()),
        ..TokenStyle::default()
    };
    v_flex()
        .bg(hsla(theme.colors.editor))
        .px(px(10.))
        .pt(px(8.))
        .gap(px(4.))
        .text_size(px(9.5))
        .line_height(px(13.))
        .text_color(p.text)
        .child(
            div()
                .text_size(px(13.))
                .line_height(px(16.))
                .font_weight(FontWeight::BOLD)
                .text_color(p.heading.unwrap_or(p.text))
                .child("Heading"),
        )
        .child(
            h_flex()
                .gap(px(3.))
                .child("Some")
                .child(div().font_weight(FontWeight::BOLD).child("bold,"))
                .child(div().text_color(p.link).child("link,"))
                .child(
                    div()
                        .px(px(2.))
                        .rounded(px(2.))
                        .bg(p.inline_code_background)
                        .font_family(SYSTEM_MONO)
                        .text_size(px(8.5))
                        .when_some(p.inline_code_text, |el, c| el.text_color(c))
                        .child("code"),
                ),
        )
        .child(
            h_flex()
                .gap(px(5.))
                .child(div().w(px(2.)).h(px(11.)).bg(p.border))
                .child(div().text_color(p.quote).child("A quote")),
        )
        .child(
            h_flex()
                .px(px(6.))
                .py(px(4.))
                .rounded(px(3.))
                .bg(p.code_background)
                .font_family(SYSTEM_MONO)
                .text_size(px(8.5))
                .child(run("const ", token("keyword").or_text(code_text)))
                .child(run("x", token("variable").or_text(code_text)))
                .child(run(" = ", token("operator").or_text(code_text)))
                .child(run("1", token("number").or_text(code_text))),
        )
}

#[cfg(test)]
mod tests {
    use super::{Kind, ThemeSettings, pick, selected};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn name() -> impl Strategy<Value = String> {
        prop::sample::select(vec!["Kanso Light", "Kanso Dark", "Nord", "Dracula"])
            .prop_map(str::to_owned)
    }

    fn settings() -> impl Strategy<Value = ThemeSettings> {
        (
            name(),
            prop::option::of(name()),
            prop::option::of(name()),
            any::<bool>(),
            name(),
            name(),
        )
            .prop_map(
                |(ui, syntax, preview, follow_system, light, dark)| ThemeSettings {
                    ui,
                    syntax,
                    preview,
                    follow_system,
                    light,
                    dark,
                },
            )
    }

    fn kind() -> impl Strategy<Value = Kind> {
        prop::sample::select(vec![Kind::Ui, Kind::Syntax, Kind::Preview])
    }

    /// Settings, a grid, the card clicked in it and its appearance.
    fn click() -> impl Strategy<Value = (ThemeSettings, Kind, Option<String>, bool)> {
        (settings(), kind(), prop::option::of(name()), any::<bool>())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A click selects the card clicked, and changes nothing in the
        /// other grids. "Same as theme" (no name) is only a choice for
        /// syntax and preview.
        #[test]
        fn prop_a_click_selects_its_card(case in click()) {
            let (s, kind, clicked, dark) = case;
            prop_assume!(kind != Kind::Ui || clicked.is_some());
            check_click(&s, kind, clicked.as_deref(), dark)?;
        }
    }

    fn check_click(
        s: &ThemeSettings,
        kind: Kind,
        clicked: Option<&str>,
        dark: bool,
    ) -> Result<(), TestCaseError> {
        let next = pick(s, kind, clicked, dark);
        prop_assert!(selected(&next, kind, clicked).0);
        if kind != Kind::Syntax {
            prop_assert_eq!(&next.syntax, &s.syntax);
        }
        if kind != Kind::Preview {
            prop_assert_eq!(&next.preview, &s.preview);
        }
        prop_assert_eq!(next.follow_system, s.follow_system);
        if kind == Kind::Ui && s.follow_system {
            // Only the slot of the theme's own appearance moves.
            let (moved, kept, was) = if dark {
                (&next.dark, &next.light, &s.light)
            } else {
                (&next.light, &next.dark, &s.dark)
            };
            prop_assert_eq!(Some(moved.as_str()), clicked);
            prop_assert_eq!(kept, was);
            prop_assert_eq!(Some(next.ui_for(dark)), clicked);
        }
        if kind == Kind::Ui && !s.follow_system {
            prop_assert_eq!(Some(next.ui_for(!dark)), clicked);
        }
        Ok(())
    }

    #[test]
    fn following_the_system_outlines_both_themes() {
        let mut s = ThemeSettings::default();
        assert_eq!(
            selected(&s, Kind::Ui, Some("Kanso Light")),
            (true, Some("Light"))
        );
        assert_eq!(
            selected(&s, Kind::Ui, Some("Kanso Dark")),
            (true, Some("Dark"))
        );
        assert_eq!(selected(&s, Kind::Syntax, None), (true, None));
        assert_eq!(
            selected(&s, Kind::Syntax, Some("Kanso Dark")),
            (false, None)
        );
        s.follow_system = false;
        assert_eq!(selected(&s, Kind::Ui, Some("Kanso Light")), (false, None));
        assert_eq!(selected(&s, Kind::Ui, Some("Kanso Dark")), (true, None));
    }
}
