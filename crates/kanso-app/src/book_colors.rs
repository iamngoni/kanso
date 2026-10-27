//! The color a notebook can show, so it's easy to find in the sidebar: kept
//! in the notebook as `#rrggbb` ([`kanso_core::model::BOOK_COLOR_FIELD`]) so
//! it syncs, and the dialog that chooses one from a grid of swatches, a color
//! picker, or hex typed in.

use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::TagColor;
pub use kanso_engine::catalogue::book_colors::{book_color, hex, parse_hex, presets};

use crate::book_icons::NotebookIcon;
use crate::editor::rgb_color;
use crate::theme::{Colors, Rgb, muted_tag_color};

/// How strongly a notebook's color washes its row: resting, under the
/// pointer, and while it is the one showing. Light enough that the theme's
/// text reads on any color, even white on a light theme.
pub const WASH_REST: f32 = 0.16;
pub const WASH_HOVER: f32 = 0.24;
pub const WASH_SELECTED: f32 = 0.34;

/// A row's background in `rgb` at `alpha`.
pub fn wash(rgb: u32, alpha: f32) -> Hsla {
    Hsla {
        a: alpha,
        ..rgb_color(rgb)
    }
}

/// The icon's color on a row washed in `rgb`: the same hue, calmed as tags
/// are so it reads on the theme.
pub fn icon_tint(rgb: u32, dark: bool) -> Rgb {
    Rgb(muted_tag_color(rgb, dark))
}

fn to_rgb(color: Hsla) -> u32 {
    let rgba = Rgba::from(color);
    let channel = |c: f32| (c.clamp(0., 1.) * 255.).round() as u32;
    (channel(rgba.r) << 16) | (channel(rgba.g) << 8) | channel(rgba.b)
}

const COLUMNS: usize = 7;
const SWATCH: f32 = 32.;
const GAP: f32 = 8.;
/// The dialog's width: the grid and the dialog's padding.
pub const DIALOG_WIDTH: f32 = COLUMNS as f32 * (SWATCH + GAP) + 64.;

type PickHandler = Rc<dyn Fn(Option<u32>, &mut Window, &mut App)>;

/// The dialog's body: a preview of the notebook's row, the swatches, and
/// the picker and hex field for any other color.
pub struct ColorDialog {
    name: SharedString,
    icon: NotebookIcon,
    /// The color the notebook kept when the dialog opened.
    initial: Option<u32>,
    chosen: Option<u32>,
    picker: Entity<ColorPickerState>,
    hex: Entity<InputState>,
    on_pick: PickHandler,
    _subscriptions: [Subscription; 2],
}

impl ColorDialog {
    fn new(
        name: SharedString,
        icon: NotebookIcon,
        chosen: Option<u32>,
        on_pick: PickHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let picker = cx.new(|cx| {
            let state = ColorPickerState::new(window, cx);
            match chosen {
                Some(rgb) => state.default_value(rgb_color(rgb)),
                None => state,
            }
        });
        let hex = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("#f8f8f8");
            if let Some(rgb) = chosen {
                state.set_value(self::hex(rgb), window, cx);
            }
            state
        });
        let picked = cx.subscribe_in(
            &picker,
            window,
            |this: &mut Self, _, event: &ColorPickerEvent, window, cx| {
                let ColorPickerEvent::Change(color) = event;
                if let Some(color) = color {
                    this.choose(Some(to_rgb(*color)), false, window, cx);
                }
            },
        );
        let typed = cx.subscribe_in(
            &hex,
            window,
            |this: &mut Self, input, event: &InputEvent, window, cx| match event {
                // A whole color as it's typed shows at once, and an empty
                // field is no color; the rest wait.
                InputEvent::Change => {
                    let text = input.read(cx).value();
                    if text.trim().is_empty() {
                        this.chosen = None;
                        this.picker
                            .update(cx, |picker, cx| picker.clear_value(window, cx));
                    } else if let Some(rgb) = parse_hex(&text) {
                        this.chosen = Some(rgb);
                        this.picker.update(cx, |picker, cx| {
                            picker.set_value(rgb_color(rgb), window, cx)
                        });
                    }
                    cx.notify();
                }
                InputEvent::PressEnter { .. } if this.typed_is_valid(cx) => this.done(window, cx),
                _ => {}
            },
        );
        ColorDialog {
            name,
            icon,
            initial: chosen,
            chosen,
            picker,
            hex,
            on_pick,
            _subscriptions: [picked, typed],
        }
    }

    /// Makes `rgb` the color shown, keeping the picker and, unless it was
    /// typed there, the hex field in step.
    fn choose(
        &mut self,
        rgb: Option<u32>,
        sync_picker: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.chosen = rgb;
        let text = rgb.map(hex).unwrap_or_default();
        self.hex
            .update(cx, |input, cx| input.set_value(text, window, cx));
        if sync_picker {
            self.picker.update(cx, |picker, cx| match rgb {
                Some(rgb) => picker.set_value(rgb_color(rgb), window, cx),
                None => picker.clear_value(window, cx),
            });
        }
        cx.notify();
    }

    /// Whether the hex field is empty or holds a color.
    fn typed_is_valid(&self, cx: &App) -> bool {
        let text = self.hex.read(cx).value();
        text.trim().is_empty() || parse_hex(&text).is_some()
    }

    /// Closes the dialog and hands on the color chosen, unless it is the one
    /// the notebook already keeps, so nothing is written for nothing.
    fn done(&self, window: &mut Window, cx: &mut App) {
        let chosen = self.chosen;
        window.close_dialog(cx);
        if chosen != self.initial {
            (self.on_pick)(chosen, window, cx);
        }
    }

    /// The row as the sidebar will draw it with the color chosen.
    fn preview(&self, cx: &App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let tint = self
            .chosen
            .map_or(colors.text_secondary, |rgb| icon_tint(rgb, dark));
        // The wash is see-through, so it lies over the sidebar's own color
        // as it will in the sidebar, not over the dialog's.
        div()
            .rounded(px(6.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.sidebar)
            .child(
                h_flex()
                    .h(px(32.))
                    .px(px(10.))
                    .gap(px(10.))
                    .rounded(px(6.))
                    .when_some(self.chosen, |el, rgb| el.bg(wash(rgb, WASH_SELECTED)))
                    .text_size(px(13.5))
                    .text_color(colors.text)
                    .child(Icon::from(self.icon.clone()).size(px(15.)).text_color(tint))
                    .child(div().flex_1().min_w_0().truncate().child(self.name.clone())),
            )
    }

    fn swatch(&self, ix: usize, color: Option<TagColor>, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = Colors::get(cx);
        let rgb = color.map(TagColor::rgb);
        let selected = self.chosen == rgb;
        let label: SharedString = color.map_or("No color".into(), |c| format!("{c:?}").into());
        let selector = label.to_lowercase().replace(' ', "-");
        div()
            .id(("book-color", ix))
            .debug_selector(move || format!("book-color-{selector}"))
            .size(px(SWATCH))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .cursor_pointer()
            .border_2()
            .border_color(if selected {
                colors.accent
            } else {
                colors.divider
            })
            .map(|el| match rgb {
                Some(rgb) => el.bg(rgb_color(rgb)),
                None => el.bg(colors.sidebar),
            })
            .when(rgb.is_none(), |el| {
                el.child(
                    Icon::new(Lucide::Close)
                        .size(px(14.))
                        .text_color(colors.text_muted),
                )
            })
            .when(selected && rgb.is_some(), |el| {
                el.child(
                    Icon::new(Lucide::Check)
                        .size(px(14.))
                        .text_color(gpui_kit::white()),
                )
            })
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx)
            })
            .on_click(cx.listener(move |this, _, window, cx| this.choose(rgb, true, window, cx)))
    }
}

// no-prop-test: drawing; `parse_hex` has the properties and the UI test
// drives the dialog.
impl Render for ColorDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let heading = |text: &'static str| {
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text_muted)
                .child(text)
        };
        let mut cells: Vec<_> = std::iter::once(None)
            .chain(presets().into_iter().map(Some))
            .enumerate()
            .map(|(ix, color)| self.swatch(ix, color, cx))
            .collect();
        let mut rows = Vec::new();
        while !cells.is_empty() {
            let rest = cells.split_off(cells.len().min(COLUMNS));
            rows.push(
                h_flex()
                    .gap(px(GAP))
                    .children(std::mem::replace(&mut cells, rest)),
            );
        }
        let invalid = !self.typed_is_valid(cx);
        v_flex()
            .gap(px(12.))
            .child(self.preview(cx))
            .child(heading("Colors"))
            .child(v_flex().gap(px(GAP)).children(rows))
            .child(heading("Custom"))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(ColorPicker::new(&self.picker).accessibility_label("Pick any color"))
                    .child(div().flex_1().child(Input::new(&self.hex).small())),
            )
            .when(invalid, |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child("Type a color as hex, like #f8f8f8 or #fa0."),
                )
            })
    }
}

/// Opens the dialog for notebook `name`, which shows `icon` and keeps the
/// color `current`, with the keyboard in the hex field. On Done, `on_pick`
/// gets the color chosen, or `None` for no color, and the dialog closes.
/// Returns the dialog's body.
pub fn open_dialog(
    name: &str,
    icon: NotebookIcon,
    current: Option<u32>,
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(Option<u32>, &mut Window, &mut App) + 'static,
) -> Entity<ColorDialog> {
    let title = SharedString::from(format!("Color for \u{201c}{name}\u{201d}"));
    let on_pick: PickHandler = Rc::new(on_pick);
    let name = SharedString::from(name.to_owned());
    let dialog = cx.new(|cx| ColorDialog::new(name, icon, current, on_pick, window, cx));
    let hex = dialog.read(cx).hex.clone();
    let first_build = std::cell::Cell::new(true);
    let view = dialog.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            hex.update(cx, |hex, cx| hex.focus(window, cx));
        }
        let done = view.clone();
        alert
            .title(title.clone())
            .width(px(DIALOG_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("book-color-cancel")
                            .small()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("book-color-done")
                            .debug_selector(|| "book-color-done".into())
                            .small()
                            .primary()
                            .label("Done")
                            .on_click(move |_, window, cx| {
                                done.update(cx, |dialog, cx| dialog.done(window, cx));
                            }),
                    ),
            )
    });
    dialog
}
