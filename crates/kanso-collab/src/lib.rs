//! Kanso's shared notes (docs/ACCOUNTS.md › Collaboration): end-to-end
//! encrypted collaboration between Kanso accounts.
//!
//! A shared note is a CRDT ([`doc`], over Yrs) whose text the library also
//! keeps as plain text, so everything that reads notes keeps working. Each
//! account has identity keys ([`identity`]); each shared note a key per epoch
//! wrapped to its members ([`keys`]) and an owner-signed member list
//! ([`members`]). Everything members write travels as signed, sealed
//! envelopes ([`envelope`]) through the sync server, which stores and
//! relays them without reading them ([`api`], [`http`], [`engine::live`]). [`engine`] puts it
//! together against a library.

pub mod api;
pub mod book;
pub mod book_model;
pub mod canvas_doc;
pub mod codec;
pub mod comments;
pub mod doc;
pub mod engine;
pub mod envelope;
pub mod http;
pub mod identity;
pub mod keys;
pub mod members;
pub mod memory;
pub mod model;
pub mod shared_doc;

pub use api::{Caller, CollabApi, CollabError};
pub use book_model::{BookHistory, BookKey};
pub use canvas_doc::CanvasDoc;
pub use doc::{CollabDoc, Field, Merged, apply_splices, remap, splices};
pub use engine::{Account, Engine, LiveHandle};
pub use identity::{Identity, IdentityError, Published, Verified, check_public, fingerprint};
pub use keys::{LinkSecret, NoteKey, invite_link, parse_invite_link};
pub use model::*;
pub use shared_doc::SharedDoc;

#[cfg(test)]
mod canvas_doc_tests;

#[cfg(test)]
pub(crate) mod tests {
    use proptest::prelude::ProptestConfig;
    use proptest::test_runner::FileFailurePersistence;

    /// Property tests' cases; CI pins the seed with `PROPTEST_RNG_SEED`.
    pub fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }
}
