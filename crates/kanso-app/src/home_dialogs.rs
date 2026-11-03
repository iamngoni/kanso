//! Customise home (issue #201): the homepage's sections in a list, each
//! with a drag handle, a switch to show or hide it, and arrows that move it
//! one place (for the keyboard and for a trackpad that drags badly). The
//! order and the hidden sections sync with the notebook.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::*;

use crate::home_view::NotebookHomeView;
use crate::theme::Colors;
use crate::zoom::px;

const DIALOG_WIDTH: f32 = 380.;

/// A section in the list: its key, its title, and whether it shows.
type Row = (String, String, bool);

/// The list Customise home edits.
pub struct CustomiseDialog {
    rows: Vec<Row>,
}

impl CustomiseDialog {
    pub fn new(rows: Vec<Row>) -> Self {
        CustomiseDialog { rows }
    }

    /// Moves row `ix` up (`by` -1) or down (1).
    pub fn move_row(&mut self, ix: usize, by: isize, cx: &mut Context<Self>) {
        let to = ix.saturating_add_signed(by);
        if ix < self.rows.len() && to < self.rows.len() && to != ix {
            self.rows.swap(ix, to);
            cx.notify();
        }
    }

    /// Puts row `from` where row `to` is, as a dropped handle does.
    pub fn reorder(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        if from < self.rows.len() && to < self.rows.len() && from != to {
            let row = self.rows.remove(from);
            self.rows.insert(to, row);
            cx.notify();
        }
    }

    pub fn show(&mut self, ix: usize, shown: bool, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.get_mut(ix) {
            row.2 = !shown;
            cx.notify();
        }
    }

    /// The order chosen and the keys hidden.
    pub fn result(&self) -> (Vec<String>, Vec<String>) {
        (
            self.rows.iter().map(|r| r.0.clone()).collect(),
            self.rows
                .iter()
                .filter(|r| r.2)
                .map(|r| r.0.clone())
                .collect(),
        )
    }
}

/// A section's row being dragged by its handle.
#[derive(Clone)]
struct DraggedRow {
    from: usize,
    title: SharedString,
}

impl Render for DraggedRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .px(px(10.))
            .py(px(5.))
            .rounded(px(6.))
            .border_1()
            .border_color(Hsla::from(colors.accent).opacity(0.6))
            .bg(colors.elevated)
            .shadow_md()
            .text_size(px(13.))
            .text_color(colors.text)
            .child(self.title.clone())
    }
}

impl Render for CustomiseDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let last = self.rows.len().saturating_sub(1);
        v_flex()
            .debug_selector(|| "home-customise".into())
            .w_full()
            .gap(px(2.))
            .children(
                self.rows
                    .iter()
                    .enumerate()
                    .map(|(ix, (_, title, hidden))| {
                        let (up, down, toggle) = (view.clone(), view.clone(), view.clone());
                        let dropped = view.clone();
                        let accent = Hsla::from(colors.accent);
                        h_flex()
                            .id(("home-row", ix))
                            .h(px(34.))
                            .gap(px(8.))
                            .px(px(2.))
                            .rounded(px(6.))
                            .items_center()
                            .drag_over::<DraggedRow>(move |style, _, _, _| {
                                style.bg(accent.opacity(0.12))
                            })
                            .on_drop(move |drag: &DraggedRow, _, cx| {
                                dropped
                                    .update(cx, |this, cx| this.reorder(drag.from, ix, cx))
                                    .ok();
                            })
                            .child(
                                div()
                                    .id(("home-grip", ix))
                                    .debug_selector(move || format!("home-grip-{ix}"))
                                    .cursor_grab()
                                    .text_color(colors.text_muted)
                                    .child(Icon::new(Lucide::GripVertical).size(px(14.)))
                                    .on_drag(
                                        DraggedRow {
                                            from: ix,
                                            title: title.clone().into(),
                                        },
                                        |drag, _, _, cx| cx.new(|_| drag.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(13.5))
                                    .text_color(if *hidden {
                                        colors.text_muted
                                    } else {
                                        colors.text
                                    })
                                    .child(title.clone()),
                            )
                            .child(
                                Button::new(("home-up", ix))
                                    .ghost()
                                    .xsmall()
                                    .icon(Lucide::ChevronUp)
                                    .disabled(ix == 0)
                                    .on_click(move |_, _, cx| {
                                        up.update(cx, |this, cx| this.move_row(ix, -1, cx)).ok();
                                    }),
                            )
                            .child(
                                Button::new(("home-down", ix))
                                    .ghost()
                                    .xsmall()
                                    .icon(Lucide::ChevronDown)
                                    .disabled(ix == last)
                                    .on_click(move |_, _, cx| {
                                        down.update(cx, |this, cx| this.move_row(ix, 1, cx)).ok();
                                    }),
                            )
                            .child(Switch::new(("home-shown", ix)).checked(!*hidden).on_click(
                                move |on, _, cx| {
                                    let on = *on;
                                    toggle.update(cx, |this, cx| this.show(ix, on, cx)).ok();
                                },
                            ))
                    }),
            )
    }
}

/// Opens Customise home for the homepage `home`. Done writes the order and
/// the hidden sections; Reset puts them back.
pub fn open_customise(home: WeakEntity<NotebookHomeView>, window: &mut Window, cx: &mut App) {
    let Some(rows) = home.upgrade().map(|h| h.read(cx).layout_rows()) else {
        return;
    };
    let dialog = cx.new(|_| CustomiseDialog::new(rows));
    let view = dialog.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (done, reset) = (view.clone(), view.clone());
        let (home_done, home_reset) = (home.clone(), home.clone());
        alert
            .title("Customise home")
            .width(px(DIALOG_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_between()
                    .child(
                        Button::new("home-customise-reset")
                            .small()
                            .ghost()
                            .label("Reset")
                            .on_click(move |_, window, cx| {
                                let _ = &reset;
                                home_reset
                                    .update(cx, |home, cx| {
                                        home.customise(Vec::new(), Vec::new(), cx)
                                    })
                                    .ok();
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("home-customise-cancel")
                                    .small()
                                    .label("Cancel")
                                    .on_click(|_, window, cx| window.close_dialog(cx)),
                            )
                            .child(
                                Button::new("home-customise-done")
                                    .debug_selector(|| "home-customise-done".into())
                                    .small()
                                    .primary()
                                    .label("Done")
                                    .on_click(move |_, window, cx| {
                                        let (order, hidden) = done.read(cx).result();
                                        home_done
                                            .update(cx, |home, cx| {
                                                home.customise(order, hidden, cx)
                                            })
                                            .ok();
                                        window.close_dialog(cx);
                                    }),
                            ),
                    ),
            )
    });
}
