//! The phone's update-notice card, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/notices_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{PhoneNotice, PhoneNoticeActionKind, PhoneNotices};

#[frb(mirror(PhoneNoticeActionKind))]
pub enum _PhoneNoticeActionKind {
    None,
    OpenSettings,
    OpenUrl,
}

#[frb(mirror(PhoneNotice))]
pub struct _PhoneNotice {
    pub id: String,
    pub title: String,
    pub what_changed: String,
    pub steps: Vec<String>,
    pub action_kind: PhoneNoticeActionKind,
    pub action_label: String,
    pub action_target: String,
}

#[frb(mirror(PhoneNotices))]
pub struct _PhoneNotices {
    pub card: Option<PhoneNotice>,
    pub total: u32,
    pub count_label: String,
}
