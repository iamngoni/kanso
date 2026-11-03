//! The title bar Kanso draws on Windows and Linux, where the system draws no
//! menu bar. As in Zed, it takes the system title bar's place: the menus on
//! the left, the rest of the bar to drag the window by, and on Windows the
//! minimise, maximise and close buttons, marked as the window's own so the
//! system still offers snap layouts over maximise and handles double clicks.
//! Under Linux server-side decorations the system's title bar stays above
//! it with its own buttons. On macOS the menus are in the menu bar and the
//! window has none of this.

use gpui_kit::base::GlobalState;
use gpui_kit::component::TitleBar;
use gpui_kit::component::menu::AppMenuBar;
use gpui_kit::*;

use crate::theme::Colors;

/// Whether windows on this platform draw the bar.
pub const SHOWN: bool = !cfg!(target_os = "macos");

/// Hands the menus `cx.set_menus` stored to the in-window menu bar, which
/// reads them from [`GlobalState`]. Call after every `cx.set_menus`.
pub fn publish_menus(cx: &mut App) {
    if !cx.has_global::<GlobalState>() {
        return;
    }
    let menus = cx.get_menus().unwrap_or_default();
    GlobalState::global_mut(cx).set_app_menus(menus);
}

/// The bar, with `menus` in it.
pub fn render(menus: &Entity<AppMenuBar>, colors: &Colors) -> impl IntoElement {
    TitleBar::new()
        .bg(colors.sidebar)
        .border_color(colors.divider)
        .child(
            // Over the bar's drag area, so on Windows a press on a menu
            // opens it instead of starting a move. Only as wide as the
            // menus: the rest of the bar still drags.
            div()
                .id("menus")
                .debug_selector(|| "title-menus".into())
                .h_full()
                .flex()
                .items_center()
                .occlude()
                .child(menus.clone()),
        )
}
