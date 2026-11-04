//! The snapshot: the last Today page, kept in the library's local meta so the
//! first frame can draw at once, before the store is asked anything.
//!
//! It is local to this device. The store drops it in the same write that
//! locks, trashes or deletes a note (`record_overview` in the store), and
//! removing a calendar source clears it too.

use kanso_store::{Result, Store};

use super::model::{Overview, VERSION};

/// The `meta` key the snapshot is kept under.
pub fn snapshot_key() -> &'static str {
    "overview.snapshot"
}

/// Keeps `overview` as the snapshot.
pub fn save_snapshot(store: &Store, overview: &Overview) -> Result<()> {
    let json = serde_json::to_string(overview)?;
    store.set_meta(snapshot_key(), &json)
}

/// The snapshot, when it is of this format and of `day` (`YYYY-MM-DD`).
/// Anything else, or anything that fails to read, is none.
pub fn load_snapshot(store: &Store, day: &str) -> Option<Overview> {
    let raw = store.meta(snapshot_key()).ok()??;
    let overview: Overview = serde_json::from_str(&raw).ok()?;
    (overview.version == VERSION && overview.day == day).then_some(overview)
}
