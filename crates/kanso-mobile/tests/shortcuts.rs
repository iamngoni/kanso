//! The shortcut catalog crosses the facade whole: every entry a phone binds,
//! in the engine's order, and none it doesn't.

use kanso_engine::shortcuts::{SHORTCUTS, ShortcutContext};
use kanso_mobile::{ShortcutPlace, shortcut_catalog};
use proptest::prelude::*;

#[test]
fn the_catalog_crosses_whole() {
    let crossed = shortcut_catalog();
    let mobile: Vec<_> = SHORTCUTS.iter().filter(|s| s.on_mobile).collect();
    assert_eq!(crossed.len(), mobile.len());
    assert!(!crossed.is_empty());
    for (entry, shortcut) in crossed.iter().zip(&mobile) {
        assert_eq!(entry.id, shortcut.id);
        assert_eq!(entry.mac, shortcut.mac);
        assert_eq!(entry.other, shortcut.other);
        assert_eq!(entry.label, shortcut.label);
    }
}

#[test]
fn quit_and_the_capture_window_stay_behind() {
    let crossed = shortcut_catalog();
    assert!(crossed.iter().all(|entry| entry.id != "quit"));
    assert!(
        crossed
            .iter()
            .all(|entry| entry.place != ShortcutPlace::Capture)
    );
    let held_back = SHORTCUTS
        .iter()
        .filter(|s| s.id == "quit" || s.context == ShortcutContext::Capture)
        .count();
    assert_eq!(crossed.len() + held_back, SHORTCUTS.len());
}

proptest! {
    /// Invariant: whichever entry of the engine's catalog is picked, it
    /// crosses with its keys and label unchanged if a phone binds it, and
    /// does not cross if not.
    #[test]
    fn prop_an_entry_crosses_exactly_when_a_phone_binds_it(ix in 0..SHORTCUTS.len()) {
        let shortcut = &SHORTCUTS[ix];
        let found = shortcut_catalog().into_iter().any(|entry| {
            entry.id == shortcut.id
                && entry.mac == shortcut.mac
                && entry.other == shortcut.other
                && entry.label == shortcut.label
        });
        prop_assert_eq!(found, shortcut.on_mobile);
    }
}
