//! The What's new sheet, declared again for the generator. What the fields
//! mean is said where the facade declares them
//! (`crates/kanso-mobile/src/whats_new_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{PhoneWhatsNew, PhoneWhatsNewEarlier, PhoneWhatsNewSection};

#[frb(mirror(PhoneWhatsNewSection))]
pub struct _PhoneWhatsNewSection {
    pub title: String,
    pub items: Vec<String>,
}

#[frb(mirror(PhoneWhatsNewEarlier))]
pub struct _PhoneWhatsNewEarlier {
    pub version: String,
    pub sections: Vec<PhoneWhatsNewSection>,
}

#[frb(mirror(PhoneWhatsNew))]
pub struct _PhoneWhatsNew {
    pub version: String,
    pub date: String,
    pub summary: String,
    pub video_path: String,
    pub sections: Vec<PhoneWhatsNewSection>,
    pub earlier: Vec<PhoneWhatsNewEarlier>,
    pub release_url: String,
}
