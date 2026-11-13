//! The words shown when someone is removed from a shared notebook, leaves
//! it, or the owner stops sharing it, and the name of the private copy
//! (issue #390 › Roles › Removing someone). One place, so both apps say
//! the same thing.

use crate::book_model::BookEndReason;

/// What a notebook is called once it is a private copy.
pub fn copy_name(name: &str) -> String {
    format!("{name} (no longer shared)")
}

/// The question that confirms removing someone: its title.
pub fn remove_title(who: &str, name: &str) -> String {
    format!("Remove {who} from \"{name}\"?")
}

/// What removing someone does, under the title.
pub fn remove_body(who: &str) -> String {
    format!(
        "{who} keeps the notes they already have on their devices but sees nothing new. \
         Kanso changes the notebook's key now."
    )
}

/// The question that confirms leaving.
pub fn leave_title(name: &str) -> String {
    format!("Leave \"{name}\"? You'll keep a private copy.")
}

/// The question that confirms stopping sharing.
pub fn stop_title(name: &str) -> String {
    format!("Stop sharing \"{name}\"? Everyone keeps a private copy of its notes as they are now.")
}

/// What the owner is told once a notebook of theirs isn't shared any more:
/// their notebook stays as it is.
pub fn owner_notice(name: &str) -> String {
    format!("\"{name}\" isn't shared any more. Everyone keeps a private copy.")
}

/// What a person is told once a notebook has ended for them.
pub fn ended_notice(reason: BookEndReason, name: &str) -> String {
    let copy = copy_name(name);
    match reason {
        BookEndReason::Removed => {
            format!("You were removed from \"{name}\". Your copy is in \"{copy}\".")
        }
        BookEndReason::Left => format!("You left \"{name}\". Your copy is in \"{copy}\"."),
        BookEndReason::Stopped => {
            format!("\"{name}\" isn't shared any more. Your copy is in \"{copy}\".")
        }
        BookEndReason::Gone => {
            format!("You can no longer open \"{name}\". Your copy is in \"{copy}\".")
        }
    }
}
