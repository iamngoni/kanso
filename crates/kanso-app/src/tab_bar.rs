//! The row of tabs above the editor: the notes open in the window, the one
//! showing, and what can be done to each. The model is in `tabs`; this only
//! draws it and says what was clicked.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::tab_bar::TabBarEvent::{
    Activate, Close, CloseOthers, CloseRight, OpenInNewWindow, Reorder,
};
use crate::tabs::TabMeta;
use crate::theme::Colors;
use crate::zoom::px;

/// The height of the bar, above the editor's own top band.
pub const TAB_BAR_HEIGHT: f32 = 38.;

/// What a tab, or its menu, asks of the workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabBarEvent {
    /// Show the tab at this index.
    Activate(usize),
    Close(usize),
    CloseOthers(usize),
    CloseRight(usize),
    /// Open the note of the tab at this index in a window of its own.
    OpenInNewWindow(String),
    Reorder {
        from: usize,
        to: usize,
    },
}

impl EventEmitter<TabBarEvent> for TabBar {}

/// A tab being dragged to a new place in the row.
#[derive(Clone)]
struct DraggedTab {
    from: usize,
    title: SharedString,
}

impl Render for DraggedTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .max_w(px(200.))
            .px(px(10.))
            .py(px(5.))
            .rounded(px(6.))
            .border_1()
            .border_color(Hsla::from(colors.accent).opacity(0.6))
            .bg(colors.elevated)
            .shadow_md()
            .text_size(px(12.5))
            .text_color(colors.text)
            .truncate()
            .child(self.title.clone())
    }
}

/// The tabs as last drawn.
#[derive(Default)]
pub struct TabBar {
    tabs: Vec<TabMeta>,
    active: usize,
    /// How far the row is scrolled when the tabs run past the window; the
    /// tab showing is kept in view.
    scroll: ScrollHandle,
}

impl TabBar {
    pub fn set(
        &mut self,
        tabs: Vec<TabMeta>,
        active: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tabs != tabs || self.active != active {
            let active_changed = self.active != active;
            self.tabs = tabs;
            self.active = active;
            if active_changed {
                // The row can run past the window, and a tab out of sight
                // with it. Asking on the next frame lets the row lay out
                // first, which it has not on the frame the bar first shows.
                let scroll = self.scroll.clone();
                window.on_next_frame(move |_, _| scroll.scroll_to_item(active));
            }
            cx.notify();
        }
    }

    #[cfg(test)]
    pub(crate) fn scroll_offset(&self) -> gpui_kit::Point<Pixels> {
        self.scroll.offset()
    }

    #[cfg(test)]
    pub(crate) fn scroll_bounds(&self) -> Bounds<Pixels> {
        self.scroll.bounds()
    }

    #[cfg(test)]
    pub(crate) fn scroll_bounds_for_item(&self, index: usize) -> Option<Bounds<Pixels>> {
        self.scroll.bounds_for_item(index)
    }

    #[cfg(test)]
    pub(crate) fn activate_for_test(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set(self.tabs.clone(), index, window, cx);
    }

    /// The "⋯" at the end of the row: every open tab, the showing one
    /// marked, for when the row runs past the window.
    fn all_tabs(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let view = cx.entity();
        let tabs = self.tabs.clone();
        let active = self.active;
        Button::new("all-tabs")
            .ghost()
            .xsmall()
            .flex_none()
            .icon(
                Icon::new(Lucide::Ellipsis)
                    .size(px(14.))
                    .text_color(colors.text_secondary),
            )
            .tooltip("All tabs")
            .dropdown_menu_with_anchor(Anchor::TopRight, move |mut menu, _, _| {
                for (index, tab) in tabs.iter().enumerate() {
                    let view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::new(tab.label().to_owned())
                            .checked(index == active)
                            .on_click(move |_, _, cx| {
                                view.update(cx, |_, cx| cx.emit(Activate(index)));
                            }),
                    );
                }
                menu
            })
            .into_any_element()
    }

    fn tab(&self, index: usize, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let tab = &self.tabs[index];
        let active = index == self.active;
        let view = cx.entity();
        let group = SharedString::from(format!("tab-group-{index}"));
        let id = tab.id.clone();
        let title = tab.label().to_owned();
        let locked = tab.locked;
        let count = self.tabs.len();

        let close = {
            let view = view.clone();
            Button::new(SharedString::from(format!("close-tab-{index}")))
                .ghost()
                .xsmall()
                .icon(
                    Icon::new(Lucide::Close)
                        .size(px(12.))
                        .text_color(colors.text_secondary),
                )
                .tooltip("Close tab")
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    view.update(cx, |_, cx| cx.emit(Close(index)));
                })
        };

        let menu_id = id.clone();

        div()
            .id(SharedString::from(format!("tab-{index}")))
            .debug_selector(move || format!("tab-{index}"))
            .group(group.clone())
            .h(px(30.))
            .max_w(px(220.))
            .min_w(px(72.))
            .flex()
            .items_center()
            .gap(px(6.))
            .pl(px(10.))
            .pr(px(4.))
            .rounded(px(7.))
            .cursor_pointer()
            .when(active, |el| el.bg(colors.fill))
            .when(!active, |el| {
                el.text_color(colors.text_secondary)
                    .hover(|el| el.bg(colors.row_hover))
            })
            .text_size(px(12.5))
            .when(active, |el| el.text_color(colors.text))
            .when(locked, |el| {
                el.child(
                    Icon::new(Lucide::Lock)
                        .size(px(12.))
                        .text_color(colors.text_muted),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(title.clone()),
            )
            .child(
                div()
                    .flex_none()
                    .opacity(0.)
                    .group_hover(group.clone(), |style| style.opacity(1.))
                    .child(close),
            )
            .tooltip({
                let title = title.clone();
                move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(title.clone()).build(window, cx)
                }
            })
            .on_click(cx.listener(move |_, event: &ClickEvent, _, cx| {
                if event.click_count() > 1 {
                    return;
                }
                cx.emit(Activate(index));
            }))
            .on_drag(
                DraggedTab {
                    from: index,
                    title: title.clone().into(),
                },
                |drag, _, _, cx| cx.new(|_| drag.clone()),
            )
            .drag_over::<DraggedTab>({
                let accent = Hsla::from(colors.accent);
                move |style, _, _, _| style.bg(accent.opacity(0.12))
            })
            .on_drop(cx.listener(move |_, drag: &DraggedTab, _, cx| {
                if drag.from != index {
                    cx.emit(Reorder {
                        from: drag.from,
                        to: index,
                    });
                }
            }))
            .context_menu(move |menu, _window, _cx| {
                let view = view.clone();
                let item = |label: &'static str, event: TabBarEvent, disabled: bool| {
                    let view = view.clone();
                    PopupMenuItem::new(label)
                        .disabled(disabled)
                        .on_click(move |_, _, cx| {
                            view.update(cx, |_, cx| cx.emit(event.clone()));
                        })
                };
                menu.separator()
                    .item(item("Close", Close(index), false))
                    .item(item("Close Others", CloseOthers(index), count < 2))
                    .item(item(
                        "Close Tabs to the Right",
                        CloseRight(index),
                        index + 1 >= count,
                    ))
                    .separator()
                    .item(item(
                        "Open in New Window",
                        OpenInNewWindow(menu_id.clone()),
                        false,
                    ))
            })
            .into_any_element()
    }
}

impl Render for TabBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        h_flex()
            .flex_none()
            .w_full()
            .h(px(TAB_BAR_HEIGHT))
            .px(px(8.))
            .gap(px(2.))
            .items_center()
            .bg(colors.editor)
            .border_b_1()
            .border_color(colors.divider)
            .debug_selector(|| "tab-bar".into())
            .child(
                h_flex()
                    .id("tab-row")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .items_center()
                    .gap(px(2.))
                    .overflow_x_scroll()
                    .track_scroll(&self.scroll)
                    .children((0..self.tabs.len()).map(|index| self.tab(index, colors, cx))),
            )
            .child(self.all_tabs(colors, cx))
    }
}
