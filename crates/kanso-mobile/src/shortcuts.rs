//! The desktop's keyboard shortcuts, for a phone or tablet with a hardware
//! keyboard (issue #48, package Q): the engine's one catalog, the entries a
//! phone binds, as plain data the shell turns into its key map and the
//! shortcut overlay.

use kanso_engine::shortcuts::{self, Shortcut, ShortcutContext, ShortcutGroup};

use crate::model::{ShortcutEntry, ShortcutPlace, ShortcutSection};

/// Every shortcut a phone or tablet binds, in the order of the desktop's
/// menus. Not Quit, which the system does, and not the Quick Capture
/// window's keys.
pub fn shortcut_catalog() -> Vec<ShortcutEntry> {
    shortcuts::on_mobile().map(entry).collect()
}

fn entry(shortcut: &Shortcut) -> ShortcutEntry {
    ShortcutEntry {
        id: shortcut.id.to_owned(),
        mac: shortcut.mac.to_owned(),
        other: shortcut.other.to_owned(),
        place: place(shortcut.context),
        label: shortcut.label.to_owned(),
        section: section(shortcut.group),
    }
}

fn place(context: ShortcutContext) -> ShortcutPlace {
    match context {
        ShortcutContext::Global => ShortcutPlace::Global,
        ShortcutContext::NoteList => ShortcutPlace::NoteList,
        ShortcutContext::Editor => ShortcutPlace::Editor,
        ShortcutContext::FocusMode => ShortcutPlace::FocusMode,
        ShortcutContext::NoteWindow => ShortcutPlace::NoteWindow,
        ShortcutContext::Palette => ShortcutPlace::Palette,
        ShortcutContext::TasksView => ShortcutPlace::TasksView,
        ShortcutContext::BoardView => ShortcutPlace::BoardView,
        ShortcutContext::CalendarView => ShortcutPlace::CalendarView,
        ShortcutContext::GraphView => ShortcutPlace::GraphView,
        ShortcutContext::Settings => ShortcutPlace::Settings,
        ShortcutContext::SettingsNav => ShortcutPlace::SettingsNav,
        ShortcutContext::Capture => ShortcutPlace::Capture,
    }
}

fn section(group: ShortcutGroup) -> ShortcutSection {
    match group {
        ShortcutGroup::App => ShortcutSection::App,
        ShortcutGroup::File => ShortcutSection::File,
        ShortcutGroup::Edit => ShortcutSection::Edit,
        ShortcutGroup::Format => ShortcutSection::Format,
        ShortcutGroup::View => ShortcutSection::View,
        ShortcutGroup::Go => ShortcutSection::Go,
        ShortcutGroup::Note => ShortcutSection::Note,
        ShortcutGroup::Window => ShortcutSection::Window,
        ShortcutGroup::Tabs => ShortcutSection::Tabs,
        ShortcutGroup::List => ShortcutSection::List,
        ShortcutGroup::Palette => ShortcutSection::Palette,
        ShortcutGroup::Calendar => ShortcutSection::Calendar,
        ShortcutGroup::Tasks => ShortcutSection::Tasks,
        ShortcutGroup::Board => ShortcutSection::Board,
        ShortcutGroup::Graph => ShortcutSection::Graph,
        ShortcutGroup::Settings => ShortcutSection::Settings,
        ShortcutGroup::Capture => ShortcutSection::Capture,
    }
}
