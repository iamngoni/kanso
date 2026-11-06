//! A plugin's panel in the inspector.
//!
//! The plugin sends a tree of nodes (`kanso_plugins::panel_model`); Kanso
//! draws each one itself, under a header that names the plugin. Nothing in
//! the tree can reach outside the panel, and what the person types stays
//! here until they press a button.

use gpui_component::scroll::ScrollableElement as _;
use std::collections::{BTreeMap, HashMap};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_plugins::panel_model::{Node, TextStyle, Tone};
use serde_json::Value;

use crate::plugin_panel_model::{on_of, opens, seed, text_of};
use crate::plugins::Plugins;
use crate::theme::{Colors, Rgb};
use crate::zoom::px;

/// The values a new panel tree `root` starts with: what the person left in
/// the open panel's inputs that `root` still has, and the nodes' own values
/// for the rest ([`seed`]).
pub fn initial_values(root: &Node, values: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    seed(root, values)
}

/// Shows the inspector's panel: opens the inspector in the main window.
pub fn reveal(cx: &mut App) {
    cx.refresh_windows();
    cx.defer(|cx| {
        if let Some((_, workspace)) = crate::workspace::main_workspace(cx) {
            workspace.update(cx, |workspace, cx| workspace.show_inspector(None, cx));
        }
        cx.refresh_windows();
    });
}

/// The text fields of the open panel, kept between draws so what is typed
/// survives. They belong to one plugin and go when the panel closes or the
/// plugin changes.
struct PanelFields {
    plugin: String,
    fields: HashMap<String, Entity<InputState>>,
    subscriptions: Vec<Subscription>,
}

impl Global for PanelFields {}

/// The field for input `id`, made on first use and brought to `text` when
/// the panel's value was changed from outside.
fn field(
    id: &str,
    placeholder: &str,
    text: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    let existing = cx.global::<PanelFields>().fields.get(id).cloned();
    if let Some(input) = existing {
        if input.read(cx).value().as_ref() != text {
            input.update(cx, |input, cx| input.set_value(text.to_owned(), window, cx));
        }
        return input;
    }
    let placeholder = placeholder.to_owned();
    let text = text.to_owned();
    let input = cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(placeholder)
            .default_value(text)
    });
    let key = id.to_owned();
    let subscription = cx.subscribe(&input, move |input, event: &InputEvent, cx| {
        if matches!(event, InputEvent::Change) {
            let value = input.read(cx).value().to_string();
            crate::plugins::set_panel_value(&key, Value::String(value), cx);
        }
    });
    let fields = cx.global_mut::<PanelFields>();
    fields.fields.insert(id.to_owned(), input.clone());
    fields.subscriptions.push(subscription);
    input
}

/// What drawing the tree needs besides the tree.
struct Draw<'a> {
    colors: Colors,
    values: &'a BTreeMap<String, Value>,
    busy: bool,
}

/// The open panel, drawn for the inspector; None when no panel is open.
pub fn element(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
    let state = cx
        .try_global::<Plugins>()
        .and_then(|plugins| plugins.panel.clone());
    let Some(state) = state else {
        if cx.has_global::<PanelFields>() {
            cx.remove_global::<PanelFields>();
        }
        return None;
    };
    // Values follow the tree (they were seeded when it arrived; this only
    // fills in a panel set some other way, without changing state).
    let values = seed(&state.panel.root, &state.values);
    let same = cx
        .try_global::<PanelFields>()
        .is_some_and(|fields| fields.plugin == state.panel.plugin_id);
    if !same {
        cx.set_global(PanelFields {
            plugin: state.panel.plugin_id.clone(),
            fields: HashMap::new(),
            subscriptions: Vec::new(),
        });
    }
    let colors = *Colors::get(cx);
    let draw = Draw {
        colors,
        values: &values,
        busy: state.busy,
    };
    let mut next = 0;
    let body = node(&state.panel.root, &draw, &mut next, window, cx);
    let header = h_flex()
        .px(px(12.))
        .py(px(9.))
        .gap(px(8.))
        .items_center()
        .border_b_1()
        .border_color(colors.divider)
        .child(
            Icon::new(Lucide::Puzzle)
                .size(px(14.))
                .text_color(colors.text_secondary),
        )
        .child(
            h_flex()
                .flex_1()
                .min_w_0()
                .gap(px(6.))
                .items_baseline()
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .font_medium()
                        .text_color(colors.text)
                        .child(state.panel.plugin_name.clone()),
                )
                .child(
                    div()
                        .flex_none()
                        .text_size(px(11.))
                        .text_color(colors.text_muted)
                        .child(state.panel.plugin_version.clone()),
                ),
        )
        .when(state.developer, |row| {
            row.child(chip("Developer", colors.warning, &colors).flex_none())
        })
        .child(
            div()
                .id("plugin-panel-close")
                .flex_none()
                .p(px(4.))
                .rounded(px(5.))
                .cursor_pointer()
                .hover(|style| style.bg(colors.hover))
                .tooltip(|window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new("Close this panel").build(window, cx)
                })
                .on_click(|_, _, cx| crate::plugins::close_panel(cx))
                .child(
                    Icon::new(Lucide::Close)
                        .size(px(12.))
                        .text_color(colors.text_secondary),
                ),
        );
    let footer = div()
        .px(px(12.))
        .py(px(9.))
        .border_t_1()
        .border_color(colors.divider)
        .text_size(px(11.))
        .text_color(colors.text_muted)
        .child(format!(
            "Drawn by {}. It can't see anything you type here until you press a button.",
            state.panel.plugin_name
        ));
    Some(
        v_flex()
            .size_full()
            .child(header)
            .child(
                div()
                    .id("plugin-panel-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p(px(12.))
                    .child(body),
            )
            .child(footer)
            .into_any_element(),
    )
}

/// A small rounded label in `tint`.
fn chip(label: &str, tint: Rgb, colors: &Colors) -> Div {
    div()
        .px(px(7.))
        .py(px(1.))
        .rounded(px(9.))
        .border_1()
        .border_color(Hsla::from(tint).opacity(0.45))
        .bg(Hsla::from(tint).opacity(0.14))
        .text_size(px(11.))
        .text_color(if tint == colors.text_muted {
            colors.text_secondary
        } else {
            tint
        })
        .child(label.to_owned())
}

/// A label above a control.
fn labelled(label: &str, control: impl IntoElement, colors: &Colors) -> Div {
    v_flex()
        .gap(px(4.))
        .child(
            div()
                .text_size(px(11.5))
                .text_color(colors.text_secondary)
                .child(label.to_owned()),
        )
        .child(control)
}

/// One node, and its children, drawn.
fn node(n: &Node, draw: &Draw, next: &mut u64, window: &mut Window, cx: &mut App) -> AnyElement {
    let colors = draw.colors;
    *next += 1;
    let ix = *next;
    match n {
        Node::Text { value, style } => div()
            .text_size(px(match style {
                TextStyle::Caption => 11.,
                _ => 13.,
            }))
            .text_color(match style {
                TextStyle::Body => colors.text,
                TextStyle::Muted => colors.text_secondary,
                TextStyle::Caption => colors.text_muted,
            })
            .child(value.clone())
            .into_any_element(),
        Node::Heading { value } => div()
            .text_size(px(15.))
            .font_semibold()
            .text_color(colors.text)
            .child(value.clone())
            .into_any_element(),
        Node::Row { children, gap } => {
            let mut row = h_flex()
                .flex_wrap()
                .items_center()
                .gap(px(gap.unwrap_or(8.)));
            for child in children {
                row = row.child(node(child, draw, next, window, cx));
            }
            row.into_any_element()
        }
        Node::Column { children, gap } => {
            let mut column = v_flex().gap(px(gap.unwrap_or(10.)));
            for child in children {
                column = column.child(node(child, draw, next, window, cx));
            }
            column.into_any_element()
        }
        Node::List { items } => {
            let mut list = v_flex().gap(px(4.));
            for item in items {
                list = list.child(
                    h_flex()
                        .gap(px(8.))
                        .items_start()
                        .child(div().mt(px(1.)).text_color(colors.text_muted).child("•"))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(node(item, draw, next, window, cx)),
                        ),
                );
            }
            list.into_any_element()
        }
        Node::Button { id, label, tone } => {
            let id = id.clone();
            let button = Button::new(ElementId::NamedInteger("plugin-button".into(), ix))
                .small()
                .label(label.clone())
                .disabled(draw.busy)
                .on_click(move |_, _, cx| crate::plugins::press(id.clone(), cx));
            match tone {
                Tone::Primary => button.primary(),
                Tone::Danger => button.danger(),
                Tone::Normal => button.outline(),
            }
            .into_any_element()
        }
        Node::Input {
            id,
            label,
            placeholder,
            ..
        } => {
            let text = text_of(draw.values.get(id));
            let input = field(id, placeholder, &text, window, cx);
            labelled(label, Input::new(&input), &colors).into_any_element()
        }
        Node::Toggle { id, label, value } => {
            let on = on_of(draw.values.get(id), *value);
            let id = id.clone();
            h_flex()
                .gap(px(10.))
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child(label.clone()),
                )
                .child(
                    Switch::new(ElementId::NamedInteger("plugin-toggle".into(), ix))
                        .checked(on)
                        .on_click(move |on, window, cx| {
                            crate::plugins::set_panel_value(&id, Value::Bool(*on), cx);
                            window.refresh();
                        }),
                )
                .into_any_element()
        }
        Node::Select {
            id,
            label,
            options,
            value,
        } => {
            let current = draw
                .values
                .get(id)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| value.clone());
            let shown = current.clone().unwrap_or_else(|| "Choose…".to_owned());
            let options = options.clone();
            let id = id.clone();
            let menu = Button::new(ElementId::NamedInteger("plugin-select".into(), ix))
                .outline()
                .small()
                .label(shown)
                .dropdown_caret(true)
                .dropdown_menu(move |mut menu, _, _| {
                    for option in &options {
                        let choose = option.clone();
                        let id = id.clone();
                        menu = menu.item(
                            PopupMenuItem::new(option.clone())
                                .checked(current.as_deref() == Some(option.as_str()))
                                .on_click(move |_, window, cx| {
                                    crate::plugins::set_panel_value(
                                        &id,
                                        Value::String(choose.clone()),
                                        cx,
                                    );
                                    window.refresh();
                                }),
                        );
                    }
                    menu
                });
            labelled(label, div().child(menu), &colors).into_any_element()
        }
        Node::Image { .. } => h_flex()
            .gap(px(8.))
            .items_center()
            .justify_center()
            .h(px(72.))
            .rounded(px(6.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.fill)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .child(Icon::new(Lucide::Image).size(px(14.)))
            .child("Image from a note")
            .into_any_element(),
        Node::Link { label, url } => {
            if opens(url) {
                let target = url.clone();
                v_flex()
                    .child(
                        div()
                            .id(ElementId::NamedInteger("plugin-link".into(), ix))
                            .cursor_pointer()
                            .text_size(px(13.))
                            .text_color(colors.link)
                            .underline()
                            .on_click(move |_, _, cx| cx.open_url(&target))
                            .child(label.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(colors.text_muted)
                            .child(url.clone()),
                    )
                    .into_any_element()
            } else {
                div()
                    .text_size(px(13.))
                    .text_color(colors.text_secondary)
                    .child(format!("{label} ({url})"))
                    .into_any_element()
            }
        }
        Node::Badge { value } => chip(value, colors.text_muted, &colors).into_any_element(),
        Node::Progress { value } => {
            let fraction = if value.is_finite() {
                value.clamp(0., 1.)
            } else {
                0.
            };
            div()
                .h(px(6.))
                .w_full()
                .rounded(px(3.))
                .bg(colors.divider)
                .child(
                    div()
                        .h_full()
                        .w(relative(fraction))
                        .rounded(px(3.))
                        .bg(colors.accent),
                )
                .into_any_element()
        }
        Node::Divider => div()
            .h(px(1.))
            .w_full()
            .bg(colors.divider)
            .into_any_element(),
    }
}
