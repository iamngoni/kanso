//! What can be done to one note, as a menu: the same items, in the same
//! order, from a right click on its row in the list and from the editor's
//! "More" button. The workspace carries each choice out
//! (`Workspace::run_note_command`). A right click among several selected
//! notes shows the items that make sense for all of them.

use std::rc::Rc;

use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::*;
use kanso_core::NoteStatus;

/// One item of the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteCommand {
    TogglePinned,
    Duplicate,
    /// Copies the note's `[[Title]]` wiki-link (`note_link`).
    CopyLink,
    /// Copies Kanso's own link to the note, by ID.
    CopyKansoLink,
    /// The note in a tab of the main window (`tabs`).
    OpenInNewTab,
    /// The note in a window of its own (`note_window.rs`).
    OpenInNewWindow,
    /// The note as a sticky note, above other windows (`sticky.rs`).
    OpenAsSticky,
    /// The note as slides, full screen (`presentation.rs`).
    Present,
    SetStatus(NoteStatus),
    ShowHistory,
    Export,
    Publish,
    Share,
    /// Seals the note's text with the lock password (`locked.rs`).
    Lock,
    /// Keeps a locked note's text as plain text again.
    RemoveLock,
    /// Starts an agent task that may read the selected notes
    /// (`agent_task_panel.rs`).
    AskAgent,
    Trash,
}

/// What a choice does: given the command, carries it out.
pub type Run = Rc<dyn Fn(NoteCommand, &mut Window, &mut App)>;

/// Adds the items for a note that is `pinned` or not, `locked` or not and
/// has `status` to `menu`, each calling `run` with its command.
pub fn items(
    menu: PopupMenu,
    pinned: bool,
    locked: bool,
    status: NoteStatus,
    run: Run,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let item = |label: &'static str, command: NoteCommand| {
        let run = run.clone();
        PopupMenuItem::new(label).on_click(move |_, window, cx| run(command, window, cx))
    };
    let statuses = run.clone();
    menu.item(item(
        if pinned { "Unpin Note" } else { "Pin Note" },
        NoteCommand::TogglePinned,
    ))
    .item(item("Duplicate", NoteCommand::Duplicate))
    .item(item("Copy Note Link", NoteCommand::CopyLink))
    .item(item("Copy Kanso Link", NoteCommand::CopyKansoLink))
    .item(item("Open in New Tab", NoteCommand::OpenInNewTab))
    .item(item("Open in New Window", NoteCommand::OpenInNewWindow))
    .item(item("Open as Sticky", NoteCommand::OpenAsSticky))
    .item(item("Present", NoteCommand::Present))
    .submenu("Status", window, cx, move |mut menu, _, _| {
        for option in NoteStatus::ALL {
            let run = statuses.clone();
            menu = menu.item(
                PopupMenuItem::new(option.label())
                    .checked(option == status)
                    .on_click(move |_, window, cx| run(NoteCommand::SetStatus(option), window, cx)),
            );
        }
        menu
    })
    .separator()
    .item(item("Show History…", NoteCommand::ShowHistory))
    .item(item("Export Note…", NoteCommand::Export))
    .item(item("Share…", NoteCommand::Share))
    .item(item("Publish to the Web…", NoteCommand::Publish))
    .separator()
    .item(if locked {
        item("Remove Lock…", NoteCommand::RemoveLock)
    } else {
        item("Lock Note…", NoteCommand::Lock)
    })
    .separator()
    .item(item("Move to Trash", NoteCommand::Trash))
}

/// Adds the items for `count` selected notes to `menu`, each calling `run`
/// with its command, which the workspace carries out for all of them.
/// `all_pinned` says whether pinning would unpin them instead.
pub fn several_items(
    menu: PopupMenu,
    count: usize,
    all_pinned: bool,
    run: Run,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let item = |label: String, command: NoteCommand| {
        let run = run.clone();
        PopupMenuItem::new(label).on_click(move |_, window, cx| run(command, window, cx))
    };
    let statuses = run.clone();
    let ai = cx.global::<crate::workspace::AppState>().config.ai.enabled;
    let menu = menu
        .label(format!("{count} Notes Selected"))
        .separator()
        .item(item(
            if all_pinned {
                format!("Unpin {count} Notes")
            } else {
                format!("Pin {count} Notes")
            },
            NoteCommand::TogglePinned,
        ))
        .submenu("Status", window, cx, move |mut menu, _, _| {
            for option in NoteStatus::ALL {
                let run = statuses.clone();
                menu = menu.item(PopupMenuItem::new(option.label()).on_click(
                    move |_, window, cx| run(NoteCommand::SetStatus(option), window, cx),
                ));
            }
            menu
        })
        .separator()
        .item(item(format!("Export {count} Notes…"), NoteCommand::Export));
    let menu = if ai {
        menu.item(item(
            format!("Ask an Agent About {count} Notes…"),
            NoteCommand::AskAgent,
        ))
    } else {
        menu
    };
    menu.separator().item(item(
        format!("Move {count} Notes to Trash"),
        NoteCommand::Trash,
    ))
}
