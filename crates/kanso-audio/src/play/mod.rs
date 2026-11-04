//! Playing a recording back.
//!
//! [`Player`] opens the parts of one recording (Ogg Opus through libopus,
//! anything else Symphonia reads) on the default speaker: a decoder thread
//! fills a ring buffer, the sound card's callback drains it, and an atomic
//! position that the interface reads ten times a second says where the
//! listener is. Seeking goes through the Ogg page index, so a three-hour
//! recording seeks as fast as a short one; parts play gaplessly as one
//! timeline; 0.75× to 2× keeps the pitch ([`stretch`]).
//!
//! [`engine`] is the same player with no sound card, which is what the
//! property tests drive.

pub mod convert;
pub mod device;
pub mod engine;
pub mod stretch;
pub mod timeline;

pub use device::Player;
pub use engine::{Control, Engine, PartInput, Status, measure};
pub use stretch::SPEEDS;
pub use timeline::Timeline;
