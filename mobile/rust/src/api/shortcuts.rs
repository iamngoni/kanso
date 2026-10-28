//! The desktop's keyboard shortcuts: the catalog a hardware keyboard binds.

use super::types::ShortcutEntry;

/// Every shortcut a phone or tablet binds, in the order of the desktop's
/// menus.
#[flutter_rust_bridge::frb(sync)]
pub fn shortcut_catalog() -> Vec<ShortcutEntry> {
    kanso_mobile::shortcut_catalog()
}
