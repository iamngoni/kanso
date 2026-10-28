//! The shortcut catalog (`kanso_engine::shortcuts`) and the keys the desktop
//! registers hold each other to account: every key the app binds is in the
//! catalog with the same keys, context and name, and every entry of the
//! catalog is bound. The phones build their key map from the catalog, so
//! this is what keeps theirs exact as the desktop changes.

use std::collections::BTreeSet;

use gpui_kit::{Keystroke, TestAppContext};
use kanso_engine::shortcuts::{SHORTCUTS, ShortcutContext};

/// One key as the desktop registers it: the keystrokes written out in full
/// (so `secondary` is what it is on this system), where it acts and the
/// action's id.
type Bound = (String, ShortcutContext, String);

/// The context a registered key acts in. The note's text is `NoteBody` for
/// the formatting keys and `Input` for the others, and Quick Capture binds
/// its keys on the window and on its text field.
fn context_of(predicate: Option<String>) -> ShortcutContext {
    match predicate.as_deref() {
        None => ShortcutContext::Global,
        Some("NoteList") => ShortcutContext::NoteList,
        Some("NoteBody" | "Input") => ShortcutContext::Editor,
        Some("FocusMode") => ShortcutContext::FocusMode,
        Some("NoteWindow") => ShortcutContext::NoteWindow,
        Some("Palette > Input") => ShortcutContext::Palette,
        Some("TasksView") => ShortcutContext::TasksView,
        Some("BoardView") => ShortcutContext::BoardView,
        Some("Calendar") => ShortcutContext::CalendarView,
        Some("GraphView") => ShortcutContext::GraphView,
        Some("Settings") => ShortcutContext::Settings,
        Some("SettingsNav") => ShortcutContext::SettingsNav,
        Some("QuickCapture" | "QuickCapture > Input") => ShortcutContext::Capture,
        Some(other) => panic!("`{other}` is a key context the catalog doesn't know"),
    }
}

/// An action's name as the catalog writes an id: `kanso::GoToTab1` is
/// `go_to_tab_1`.
fn id_of(action: &str) -> String {
    let name = action.rsplit("::").next().unwrap_or(action);
    let mut id = String::new();
    let mut previous: Option<char> = None;
    for c in name.chars() {
        let starts_word = previous.is_some_and(|p| {
            (c.is_ascii_uppercase() && !p.is_ascii_uppercase())
                || (c.is_ascii_digit() && !p.is_ascii_digit())
        });
        if starts_word {
            id.push('_');
        }
        id.push(c.to_ascii_lowercase());
        previous = Some(c);
    }
    id
}

/// Keys as GPUI writes them in full, whatever order the modifiers came in.
fn written(keys: &str) -> String {
    keys.split(' ')
        .map(|stroke| {
            Keystroke::parse(stroke)
                .unwrap_or_else(|error| panic!("`{stroke}` isn't a keystroke: {error}"))
                .unparse()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every key the app registers when it starts.
fn registered(cx: &mut TestAppContext) -> BTreeSet<Bound> {
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::note_list::bind_keys(cx);
        crate::note_window::bind_keys(cx);
        crate::graph_view::bind_keys(cx);
        crate::board_view::bind_keys(cx);
        crate::capture::bind_keys(cx);
        let keymap = cx.key_bindings();
        let keymap = keymap.borrow();
        keymap
            .bindings()
            .map(|binding| {
                let keys: Vec<String> = binding
                    .keystrokes()
                    .iter()
                    .map(|stroke| stroke.inner().unparse())
                    .collect();
                let context = context_of(binding.predicate().map(|p| p.to_string()));
                (keys.join(" "), context, id_of(binding.action().name()))
            })
            .collect()
    })
}

/// The catalog's entries this system binds.
fn catalogued() -> BTreeSet<Bound> {
    let apple = cfg!(target_os = "macos");
    SHORTCUTS
        .iter()
        .filter(|shortcut| !shortcut.keys(apple).is_empty())
        .map(|shortcut| {
            (
                written(shortcut.keys(apple)),
                shortcut.context,
                shortcut.id.to_string(),
            )
        })
        .collect()
}

/// Both directions in one: what is bound and not catalogued, and what is
/// catalogued and not bound, are both named.
// no-prop-test: compares two fixed tables; `kanso_engine::shortcuts` has the
// properties.
#[gpui_kit::test]
fn the_catalog_is_exactly_the_keys_the_desktop_binds(cx: &mut TestAppContext) {
    let bound = registered(cx);
    let catalogued = catalogued();
    let unlisted: Vec<_> = bound.difference(&catalogued).collect();
    let unbound: Vec<_> = catalogued.difference(&bound).collect();
    assert!(
        unlisted.is_empty() && unbound.is_empty(),
        "bound by the desktop but not in the catalog: {unlisted:#?}\n\
         in the catalog but not bound by the desktop: {unbound:#?}"
    );
}

/// Entries with no keys on this system are the ones the desktop binds on the
/// other: the catalog has no entry that is bound nowhere.
// no-prop-test: checks a fixed table.
#[test]
fn an_entry_without_keys_here_has_them_on_the_other_system() {
    for shortcut in SHORTCUTS {
        assert!(
            !shortcut.mac.is_empty(),
            "{} is bound on neither system",
            shortcut.id
        );
    }
}
