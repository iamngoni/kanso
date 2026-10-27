//! Finding your way between notes, without a window: the backlinks and
//! unlinked mentions of a note, and Open Note (⌘P).
//!
//! The desktop's inspector and palette and the phones' sheets ask here. The
//! rule they share: while the lock session is closed, a locked note (sealed
//! itself, or in a locked notebook) never shows in any of it, not by title
//! and not by heading; once the session is open it shows like any other
//! note. The store keeps a locked note's text sealed, so what the session
//! has opened comes in as [`model::LockedNotes`].

pub mod backlinks;
pub mod model;
pub mod quick_open;
