//! What encrypting an existing library is made of (issue #243 step 10): its
//! steps, its progress, its options and what it reports. The work is in
//! [`crate::encrypt`].

use serde::{Deserialize, Serialize};

/// A point in the conversion, after which a crash is recovered from on the
/// next launch. A test kills the process at each one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EncryptStep {
    /// The marker says a copy is being made.
    MarkerWritten,
    /// The encrypted copy is written, not yet checked.
    Exported,
    /// The copy opens with the key and matches the library, and is on disk.
    Verified,
    /// The marker says the copy is good and the swap may begin.
    ReadyMarked,
    /// The plain library's write-ahead log and shared memory are gone.
    WalRemoved,
    /// The plain library is kept under a second name.
    SafetyLinked,
    /// The encrypted copy is the library.
    Swapped,
    /// The library opened encrypted and matched the plain one it replaced.
    Confirmed,
    /// The plaintext is overwritten and deleted.
    PlainShredded,
    /// The marker is gone: the conversion is over.
    Finished,
}

impl EncryptStep {
    /// Every step, in order.
    pub const ALL: [EncryptStep; 10] = [
        EncryptStep::MarkerWritten,
        EncryptStep::Exported,
        EncryptStep::Verified,
        EncryptStep::ReadyMarked,
        EncryptStep::WalRemoved,
        EncryptStep::SafetyLinked,
        EncryptStep::Swapped,
        EncryptStep::Confirmed,
        EncryptStep::PlainShredded,
        EncryptStep::Finished,
    ];
}

/// What the conversion is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncryptStage {
    Preparing,
    Copying,
    Checking,
    Finishing,
    /// Re-encrypting the backups.
    Backups,
}

/// How far along the conversion is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EncryptProgress {
    pub stage: EncryptStage,
    /// From 0 to 1, never going back.
    pub fraction: f32,
}

impl EncryptProgress {
    /// The whole percent for "Encrypting your library… 62%".
    pub fn percent(self) -> u8 {
        (self.fraction.clamp(0.0, 1.0) * 100.0).floor() as u8
    }
}

/// How a conversion is watched. All of it is optional.
#[derive(Default)]
pub struct EncryptOptions<'a> {
    /// Called as work goes on, from the converting thread and from a timer.
    pub progress: Option<&'a (dyn Fn(EncryptProgress) + Sync)>,
    /// Called after each [`EncryptStep`]. A test kills the process, or
    /// panics, here to prove the next launch recovers.
    pub after_step: Option<&'a (dyn Fn(EncryptStep) + Sync)>,
    /// A cap on the encrypted copy's size in bytes, so a test can stand in
    /// for a full disk.
    pub max_copy_bytes: Option<u64>,
}

/// What `encrypt_library` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encrypted {
    /// The library was encrypted already; nothing was done.
    AlreadyEncrypted,
    /// A plain library is now encrypted.
    Converted,
    /// A conversion a crash interrupted was finished.
    Resumed,
}

/// What the re-encrypting of backups did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackupsReport {
    /// Backups that were plain and are encrypted now.
    pub converted: usize,
    /// Backups that could not be: the file and why. They are left as they
    /// were, plain, and tried again at the next launch.
    pub failed: Vec<(std::path::PathBuf, String)>,
}

/// How far a conversion got, kept beside the library so a crash can be
/// recovered from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Phase {
    /// A copy is being made; the plain library is untouched.
    Exporting,
    /// The copy is checked. The swap is under way or done; the plain
    /// library is kept until the encrypted one is confirmed.
    Ready,
    /// The encrypted library is confirmed; the plaintext is being deleted.
    Cleanup,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Marker {
    pub v: u32,
    pub phase: Phase,
}
