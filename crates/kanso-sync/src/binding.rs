//! A library's link to an account, and this Mac's identity in it
//! (docs/ACCOUNTS.md › Device identity, › Library binding). Frozen by step 1;
//! the bodies that touch the store or the system are step 2a. "The same
//! account" always means the same `(origin, account_id)` (security re-check 3).

use std::fmt;
use std::path::{Path, PathBuf};

use kanso_core::DocType;
use kanso_store::{Store, StoreError, SyncGuard};
use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

use crate::account::Session;
use crate::crypto::{DeviceProof, hex, random};
use crate::keychain::{SecretError, SecretStore};

/// The library's `meta` keys for v2 sync (H5). Only `unbind` touches the
/// legacy cursor.
pub mod meta_keys {
    /// The origin of the account's server (`Session::origin`).
    pub const ORIGIN: &str = "sync.v2.origin";
    /// The account the library syncs with.
    pub const ACCOUNT_ID: &str = "sync.v2.account_id";
    /// Its username.
    pub const USERNAME: &str = "sync.v2.username";
    /// The server URL.
    pub const SERVER: &str = "sync.v2.server";
    /// The cursor: the last change sequence pulled.
    pub const LAST_SEQ: &str = "sync.v2.last_seq";
    /// The epoch last seen (M1).
    pub const EPOCH: &str = "sync.v2.epoch";
    /// The file ID where the last sync's search for missing attachment bytes
    /// stopped; the next one goes on after it.
    pub const REFETCH_AFTER: &str = "sync.v2.refetch_after";
    /// The files whose bytes a refetch missed, as a JSON object of file ID
    /// to the time (ms) it may ask again: a day later (review-verify 4).
    pub const REFETCH_MISSED: &str = "sync.v2.refetch_missed";
    /// A remembered 413 on a push: `<kind> <used> <limit> <at_ms> <level>`
    /// (docs/ACCOUNTS.md › Sync with encryption › Quota).
    pub const QUOTA_DOCS: &str = "sync.v2.quota.docs";
    /// A remembered 413 on an upload, in the same form.
    pub const QUOTA_FILES: &str = "sync.v2.quota.files";
    /// After a 429 `uploads-global`: uploads wait until this time (ms).
    pub const UPLOADS_UNTIL: &str = "sync.v2.uploads_until";
    /// Device identity: `l_` + 32 hex; names the Keychain items.
    pub const LIBRARY_ID: &str = "sync.v2.library_id";
    /// Device identity: 32 random bytes as hex.
    pub const DEVICE_SECRET: &str = "sync.v2.device_secret";
    /// Device identity: `hex(SHA-256("kanso-hw" || IOPlatformUUID))[0..32]`,
    /// or empty when `ioreg` gives no UUID.
    pub const HARDWARE: &str = "sync.v2.hardware";
    /// The v1 cursor, which the rollback path resets to 0.
    pub const LEGACY_LAST_SEQ: &str = "sync.last_seq";

    /// The sync state that belongs to one account and goes when the library
    /// is bound to another or unbound (the cursor aside).
    pub const PER_ACCOUNT: &[&str] = &[
        EPOCH,
        REFETCH_AFTER,
        REFETCH_MISSED,
        QUOTA_DOCS,
        QUOTA_FILES,
        UPLOADS_UNTIL,
    ];
}

/// The guard every sync write of `session` carries: the library must still
/// be bound to the session's `(origin, account_id)`, checked inside the
/// store's write transaction. Use as `SyncGuard { expect: &guard_pairs(&s) }`.
pub fn guard_pairs(session: &Session) -> [(&'static str, &str); 2] {
    [
        (meta_keys::ORIGIN, session.origin.as_str()),
        (meta_keys::ACCOUNT_ID, session.account_id.as_str()),
    ]
}

/// Builds the [`SyncGuard`] over [`guard_pairs`].
pub fn guard<'a>(pairs: &'a [(&'a str, &'a str)]) -> SyncGuard<'a> {
    SyncGuard { expect: pairs }
}

/// What this computer reports about itself: on a Mac its hardware UUID
/// (from `ioreg -rd1 -c IOPlatformExpertDevice`) and its ComputerName
/// (`scutil --get ComputerName`); on Linux its machine ID and host name.
/// Tests build it directly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    pub hardware_uuid: Option<String>,
    pub computer_name: String,
}

// no-prop-test: runs two system commands; the parsing below is tested.
impl Machine {
    /// Reads this Mac: `ioreg -rd1 -c IOPlatformExpertDevice` for the
    /// hardware UUID (none when it doesn't answer) and `scutil --get
    /// ComputerName` for the name.
    pub fn this_mac() -> Machine {
        let run = |program: &str, args: &[&str]| {
            std::process::Command::new(program)
                .args(args)
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
                .unwrap_or_default()
        };
        Machine {
            hardware_uuid: if cfg!(target_os = "macos") {
                platform_uuid(&run(
                    "/usr/sbin/ioreg",
                    &["-rd1", "-c", "IOPlatformExpertDevice"],
                ))
            } else if cfg!(windows) {
                None
            } else {
                // systemd's machine ID (D-Bus's on older systems): a copied
                // data folder on another server then reads as a copy. A
                // library that recorded none before simply records it.
                ["/etc/machine-id", "/var/lib/dbus/machine-id"]
                    .iter()
                    .find_map(|path| machine_id(&std::fs::read_to_string(path).ok()?))
            },
            computer_name: if cfg!(target_os = "macos") {
                run("/usr/sbin/scutil", &["--get", "ComputerName"])
            } else {
                host_name()
            }
            .trim_end_matches(['\r', '\n'])
            .to_owned(),
        }
    }

    /// `hex(SHA-256("kanso-hw" || IOPlatformUUID))[0..32]`, or empty when
    /// there is no UUID.
    pub fn hardware_hash(&self) -> String {
        match &self.hardware_uuid {
            Some(uuid) if !uuid.is_empty() => {
                let digest = Sha256::new()
                    .chain_update(b"kanso-hw")
                    .chain_update(uuid.as_bytes())
                    .finalize();
                hex(&digest)[..32].to_owned()
            }
            _ => String::new(),
        }
    }
}

/// The host name off macOS: `COMPUTERNAME` on Windows, the kernel's host
/// name on Linux and other Unixes. Empty when there is none.
fn host_name() -> String {
    if cfg!(windows) {
        std::env::var("COMPUTERNAME").unwrap_or_default()
    } else {
        std::fs::read_to_string("/proc/sys/kernel/hostname")
            .or_else(|_| std::fs::read_to_string("/etc/hostname"))
            .unwrap_or_default()
    }
}

/// The name a device goes by when it reports none.
pub const DEFAULT_DEVICE_NAME: &str = if cfg!(target_os = "macos") {
    "Mac"
} else if cfg!(windows) {
    "Windows PC"
} else if cfg!(target_os = "linux") {
    "Linux computer"
} else {
    "Computer"
};

/// The machine ID in `/etc/machine-id`: 32 hex digits on one line, or
/// `None` for an empty or uninitialized one. Pure.
pub fn machine_id(text: &str) -> Option<String> {
    let id = text.trim();
    (id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()) && id.bytes().any(|b| b != b'0'))
        .then(|| id.to_ascii_lowercase())
}

/// The `"IOPlatformUUID" = "…"` value in `ioreg` output.
fn platform_uuid(ioreg: &str) -> Option<String> {
    ioreg.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "\"IOPlatformUUID\"")
            .then(|| value.trim().trim_matches('"').to_owned())
            .filter(|uuid| !uuid.is_empty())
    })
}

/// The device name rule (docs/ACCOUNTS.md › Sealed device names): control
/// characters removed, trimmed, cut to 64 Unicode scalar values (and trimmed
/// again, so the rule is idempotent), or [`DEFAULT_DEVICE_NAME`] when that leaves nothing. Pure.
pub fn device_name(computer_name: &str) -> String {
    let cleaned: String = computer_name.chars().filter(|c| !c.is_control()).collect();
    let cut: String = cleaned.trim().chars().take(64).collect();
    let name = cut.trim_end();
    if name.is_empty() {
        DEFAULT_DEVICE_NAME.to_owned()
    } else {
        name.to_owned()
    }
}

/// This library's identity on this Mac. The device secret is private.
pub struct DeviceIdentity {
    /// `l_` + 32 hex; names the Keychain items.
    pub library_id: String,
    /// [`device_name`] of this Mac, sealed before it is sent.
    pub name: String,
    device_secret: Zeroizing<[u8; 32]>,
}

impl DeviceIdentity {
    /// An identity from its parts (tests and golden vectors; the app uses
    /// [`device_identity`]).
    pub fn from_parts(library_id: &str, name: &str, device_secret: [u8; 32]) -> DeviceIdentity {
        DeviceIdentity {
            library_id: library_id.to_owned(),
            name: device_name(name),
            device_secret: Zeroizing::new(device_secret),
        }
    }

    /// This Mac's ID in `account_id`: `d_ + hex(HMAC-SHA256(device_secret,
    /// account_id)[0..16])`, unlinkable across accounts (L2).
    pub fn device_id(&self, account_id: &str) -> String {
        crate::crypto::device_id(&self.device_secret, account_id)
    }

    /// This Mac's proof for `(origin, account_id)`: HKDF(ikm device_secret,
    /// info "kanso/v1/device-proof" || lp(origin) || lp(account_id)). Only
    /// a matching proof exempts a sign-in from `login-user` (re-check 5).
    pub fn device_proof(&self, origin: &str, account_id: &str) -> DeviceProof {
        crate::crypto::device_proof(&self.device_secret, origin, account_id)
    }
}

/// `l_` and 32 lowercase hex digits.
fn is_library_id(id: &str) -> bool {
    id.strip_prefix("l_").is_some_and(|hex| {
        hex.len() == 32 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    })
}

fn parse_secret(text: &str) -> Option<Zeroizing<[u8; 32]>> {
    if text.len() != 64 {
        return None;
    }
    let mut secret = Zeroizing::new([0u8; 32]);
    for (i, byte) in secret.iter_mut().enumerate() {
        *byte = u8::from_str_radix(text.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(secret)
}

impl fmt::Debug for DeviceIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceIdentity")
            .field("library_id", &self.library_id)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Reads or creates the library's identity (`sync.v2.library_id`,
/// `sync.v2.device_secret`, `sync.v2.hardware`). When the stored hardware
/// hash differs from `machine`'s, the library was copied: the library ID and
/// device secret are regenerated (the copy is then signed out) and the
/// binding and cursor stay. Sign-out and unbind keep all three values.
pub fn device_identity(store: &Store, machine: &Machine) -> Result<DeviceIdentity, StoreError> {
    let hardware = machine.hardware_hash();
    let library_id = store.meta(meta_keys::LIBRARY_ID)?;
    let secret = store
        .meta(meta_keys::DEVICE_SECRET)?
        .as_deref()
        .and_then(parse_secret);
    let stored_hardware = store.meta(meta_keys::HARDWARE)?.unwrap_or_default();
    let copied = !stored_hardware.is_empty() && !hardware.is_empty() && stored_hardware != hardware;
    match (library_id, secret) {
        (Some(library_id), Some(device_secret)) if is_library_id(&library_id) && !copied => {
            if stored_hardware.is_empty() && !hardware.is_empty() {
                store.set_meta(meta_keys::HARDWARE, &hardware)?;
            }
            Ok(DeviceIdentity {
                library_id,
                name: device_name(&machine.computer_name),
                device_secret,
            })
        }
        _ => {
            // New, unreadable, or copied from another Mac: a new identity.
            // The binding and cursor stay, so signing in resumes as a new
            // device of the same account.
            let library_id = format!("l_{}", hex(&random::<16>()));
            let device_secret = Zeroizing::new(random::<32>());
            let secret_hex = Zeroizing::new(hex(&device_secret[..]));
            store.set_metas(&[
                (meta_keys::LIBRARY_ID, Some(library_id.as_str())),
                (meta_keys::DEVICE_SECRET, Some(secret_hex.as_str())),
                (meta_keys::HARDWARE, Some(hardware.as_str())),
            ])?;
            Ok(DeviceIdentity {
                library_id,
                name: device_name(&machine.computer_name),
                device_secret,
            })
        }
    }
}

/// What the library holds, relative to the `(origin, account_id)` being
/// signed into. Checked in this order: bound to that pair → `Same`; no live
/// documents → `Empty`; bound to another pair → `Other`; else `Unbound`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalLibrary {
    /// No live documents (and not bound to this account).
    Empty,
    /// Already bound to this `(origin, account_id)`.
    Same,
    /// Notes that never belonged to an account. `notes` counts live notes;
    /// `unsynced` is [`Store::pending_count`].
    Unbound { notes: usize, unsynced: usize },
    /// Bound to another account (or the same account ID on another origin).
    Other {
        username: String,
        notes: usize,
        unsynced: usize,
    },
}

/// Reads the library's binding and counts. `notes` counts live notes;
/// "empty" means no live documents of any kind.
pub fn local_library(
    store: &Store,
    origin: &str,
    account_id: &str,
) -> Result<LocalLibrary, StoreError> {
    let bound_origin = store.meta(meta_keys::ORIGIN)?;
    let bound_account = store.meta(meta_keys::ACCOUNT_ID)?;
    if bound_origin.as_deref() == Some(origin) && bound_account.as_deref() == Some(account_id) {
        return Ok(LocalLibrary::Same);
    }
    let counts = store.counts()?;
    if counts.values().all(|&n| n == 0) {
        return Ok(LocalLibrary::Empty);
    }
    let notes = counts.get(&DocType::Note).copied().unwrap_or(0);
    let unsynced = store.pending_count()?;
    if bound_origin.is_some() || bound_account.is_some() {
        return Ok(LocalLibrary::Other {
            username: store.meta(meta_keys::USERNAME)?.unwrap_or_default(),
            notes,
            unsynced,
        });
    }
    Ok(LocalLibrary::Unbound { notes, unsynced })
}

/// The meta entries a bind writes: the account's origin, ID, username and
/// server, a cursor of 0, and none of the previous account's sync state.
fn bound_meta(session: &Session) -> Vec<(&'static str, Option<&str>)> {
    let mut meta = vec![
        (meta_keys::ORIGIN, Some(session.origin.as_str())),
        (meta_keys::ACCOUNT_ID, Some(session.account_id.as_str())),
        (meta_keys::USERNAME, Some(session.username.as_str())),
        (meta_keys::SERVER, Some(session.server.as_str())),
        (meta_keys::LAST_SEQ, Some("0")),
    ];
    meta.extend(meta_keys::PER_ACCOUNT.iter().map(|key| (*key, None)));
    meta
}

/// The flow that is about to bind the library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Flow {
    SignUp,
    SignIn,
    Recover,
}

/// What a flow does with the library (docs/ACCOUNTS.md › Library binding,
/// the plan table).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    /// Bind fresh with no question, then upload (sign-up) or pull.
    Fresh,
    /// Already bound to this account: nothing to bind.
    Resume,
    /// Sign-up while the notes came from another account: before signing
    /// up, ask "This Mac's notes came from *from*. Upload them to the new
    /// account?" [Upload to new account] ([`Choice::Add`]) [Cancel].
    AskBeforeSignUp { from: String, notes: usize },
    /// Sign-in or recovery into a library with notes: after authenticating
    /// and before saving anything, ask "This Mac already has notes"
    /// [Add them] [Replace them] [Cancel]. `from` is the other account's
    /// username, when the notes came from one.
    Ask {
        from: Option<String>,
        notes: usize,
        unsynced: usize,
    },
}

/// The user's answer to a [`Plan`] question.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Choice {
    /// Merge the library into the account (also how [`Plan::Fresh`] binds).
    Add,
    /// Back up the library, then replace it with the account's notes.
    Replace,
    /// Bind nothing; a new session is signed out.
    Cancel,
}

/// The plan for `flow` on `library`. Pure: the table, implemented in step 1.
pub fn plan(flow: Flow, library: &LocalLibrary) -> Plan {
    match (flow, library) {
        (_, LocalLibrary::Same) => Plan::Resume,
        (_, LocalLibrary::Empty) => Plan::Fresh,
        (Flow::SignUp, LocalLibrary::Unbound { .. }) => Plan::Fresh,
        (
            Flow::SignUp,
            LocalLibrary::Other {
                username, notes, ..
            },
        ) => Plan::AskBeforeSignUp {
            from: username.clone(),
            notes: *notes,
        },
        (Flow::SignIn | Flow::Recover, LocalLibrary::Unbound { notes, unsynced }) => Plan::Ask {
            from: None,
            notes: *notes,
            unsynced: *unsynced,
        },
        (
            Flow::SignIn | Flow::Recover,
            LocalLibrary::Other {
                username,
                notes,
                unsynced,
            },
        ) => Plan::Ask {
            from: Some(username.clone()),
            notes: *notes,
            unsynced: *unsynced,
        },
    }
}

/// Why binding failed.
#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Keychain(#[from] SecretError),
}

/// Binds the library to `session`'s account. The caller has cancelled and
/// joined the running sync.
///
/// - [`Choice::Add`] (and fresh binds): `reset_sync` writing the origin,
///   account, username and server, `last_seq = 0`, and removing the epoch.
/// - [`Choice::Replace`]: `Store::backup_now(backups_dir, now)`, then
///   `replace_library` with the same meta; returns the backup's path.
/// - [`Choice::Cancel`]: changes nothing and returns `None`.
///
/// Binding a library already bound to the session's `(origin, account_id)`
/// with [`Choice::Add`] only refreshes the username and server: it is
/// [`Plan::Resume`], and resetting it would push the whole library again.
pub fn bind(
    store: &Store,
    session: &Session,
    choice: Choice,
    backups_dir: &Path,
    now_ms: i64,
) -> Result<Option<PathBuf>, BindError> {
    match choice {
        Choice::Cancel => Ok(None),
        Choice::Add => {
            if local_library(store, &session.origin, &session.account_id)? == LocalLibrary::Same {
                store.set_metas(&[
                    (meta_keys::USERNAME, Some(session.username.as_str())),
                    (meta_keys::SERVER, Some(session.server.as_str())),
                ])?;
            } else {
                store.reset_sync(&bound_meta(session))?;
            }
            Ok(None)
        }
        Choice::Replace => {
            let backup = store.backup_now(backups_dir, now_ms)?;
            store.replace_library(&bound_meta(session))?;
            Ok(Some(backup))
        }
    }
}

/// Unbinds (after deleting the account, and for rollback): `reset_sync`
/// removing the origin, account, username, server, cursor and epoch,
/// keeping the device identity, and setting the legacy `sync.last_seq = 0`;
/// then deletes `sync-account:<library>`.
pub fn unbind(store: &Store, secrets: &dyn SecretStore) -> Result<(), BindError> {
    let mut meta = vec![
        (meta_keys::ORIGIN, None),
        (meta_keys::ACCOUNT_ID, None),
        (meta_keys::USERNAME, None),
        (meta_keys::SERVER, None),
        (meta_keys::LAST_SEQ, None),
        (meta_keys::LEGACY_LAST_SEQ, Some("0")),
    ];
    meta.extend(meta_keys::PER_ACCOUNT.iter().map(|key| (*key, None)));
    store.reset_sync(&meta)?;
    if let Some(library_id) = store.meta(meta_keys::LIBRARY_ID)? {
        crate::keychain::delete_session(secrets, &library_id)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// The store's list of the sync keys a restore keeps
    /// (`DEVICE_IDENTITY_KEYS`) is the device identity here, no more and no
    /// less: the binding, the cursor and everything per account are dropped,
    /// so the next sign-in asks before merging (PR #301 review).
    #[test]
    fn a_restore_keeps_exactly_the_device_identity() {
        let mut kept = vec![
            meta_keys::LIBRARY_ID,
            meta_keys::DEVICE_SECRET,
            meta_keys::HARDWARE,
        ];
        let mut stored = kanso_store::DEVICE_IDENTITY_KEYS.to_vec();
        kept.sort_unstable();
        stored.sort_unstable();
        assert_eq!(kept, stored);
        for key in [
            meta_keys::ORIGIN,
            meta_keys::ACCOUNT_ID,
            meta_keys::USERNAME,
            meta_keys::SERVER,
            meta_keys::LAST_SEQ,
            meta_keys::LEGACY_LAST_SEQ,
        ]
        .iter()
        .chain(meta_keys::PER_ACCOUNT)
        {
            assert!(!stored.contains(key), "{key}");
        }
    }

    #[test]
    fn device_name_examples() {
        assert_eq!(device_name("Ngoni’s MacBook Pro"), "Ngoni’s MacBook Pro");
        assert_eq!(device_name("  Studio\n"), "Studio");
        assert_eq!(device_name("a\u{0}b\u{7f}c"), "abc");
        assert_eq!(device_name(""), DEFAULT_DEVICE_NAME);
        assert_eq!(device_name(" \t\u{1b} "), DEFAULT_DEVICE_NAME);
        assert_eq!(device_name(&"é".repeat(70)).chars().count(), 64);
    }

    #[test]
    fn plan_table() {
        let unbound = LocalLibrary::Unbound {
            notes: 3,
            unsynced: 1,
        };
        let other = LocalLibrary::Other {
            username: "alex".into(),
            notes: 3,
            unsynced: 1,
        };
        assert_eq!(plan(Flow::SignUp, &LocalLibrary::Empty), Plan::Fresh);
        assert_eq!(plan(Flow::SignUp, &unbound), Plan::Fresh);
        assert_eq!(
            plan(Flow::SignUp, &other),
            Plan::AskBeforeSignUp {
                from: "alex".into(),
                notes: 3
            }
        );
        for flow in [Flow::SignIn, Flow::Recover] {
            assert_eq!(plan(flow, &LocalLibrary::Empty), Plan::Fresh);
            assert_eq!(plan(flow, &LocalLibrary::Same), Plan::Resume);
            assert_eq!(
                plan(flow, &unbound),
                Plan::Ask {
                    from: None,
                    notes: 3,
                    unsynced: 1
                }
            );
            assert_eq!(
                plan(flow, &other),
                Plan::Ask {
                    from: Some("alex".into()),
                    notes: 3,
                    unsynced: 1
                }
            );
        }
    }

    fn gen_library() -> impl Strategy<Value = LocalLibrary> {
        prop_oneof![
            Just(LocalLibrary::Empty),
            Just(LocalLibrary::Same),
            (1..50usize, 0..50usize)
                .prop_map(|(notes, unsynced)| LocalLibrary::Unbound { notes, unsynced }),
            ("[a-z]{3,8}", 1..50usize, 0..50usize).prop_map(|(username, notes, unsynced)| {
                LocalLibrary::Other {
                    username,
                    notes,
                    unsynced,
                }
            }),
        ]
    }

    fn gen_flow() -> impl Strategy<Value = Flow> {
        prop_oneof![Just(Flow::SignUp), Just(Flow::SignIn), Just(Flow::Recover)]
    }

    #[test]
    fn machine_ids_are_read_or_refused() {
        assert_eq!(
            machine_id("0123456789ABCDEF0123456789abcdef\n").as_deref(),
            Some("0123456789abcdef0123456789abcdef")
        );
        assert_eq!(machine_id(""), None);
        assert_eq!(machine_id("uninitialized\n"), None);
        assert_eq!(machine_id(&"0".repeat(32)), None);
    }

    proptest! {
        /// Any 32 hex digits (not all zero) are an ID, whatever the case or
        /// surrounding whitespace; anything shorter, longer or not hex isn't.
        #[test]
        fn prop_machine_id(id in "[0-9a-fA-F]{32}", pad in "[ \n\t]{0,3}", other in "[0-9a-z]{0,40}") {
            let expected = id.bytes().any(|b| b != b'0').then(|| id.to_ascii_lowercase());
            prop_assert_eq!(machine_id(&format!("{pad}{id}{pad}")), expected.clone());
            if let Some(found) = machine_id(&other) {
                prop_assert_eq!(found.len(), 32);
                prop_assert_eq!(other.len(), 32);
            }
            // The same ID always hashes the same, and differs from none.
            let machine = |uuid: Option<String>| Machine { hardware_uuid: uuid, computer_name: String::new() };
            if let Some(found) = expected {
                prop_assert_eq!(machine(Some(found.clone())).hardware_hash().len(), 32);
                prop_assert_ne!(machine(Some(found)).hardware_hash(), machine(None).hardware_hash());
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: notes reach another account only through a question,
        /// and only notes that came from elsewhere are asked about.
        #[test]
        fn prop_notes_reach_another_account_only_by_asking(flow in gen_flow(), library in gen_library()) {
            let decided = plan(flow, &library);
            let has_foreign_notes = match &library {
                LocalLibrary::Other { .. } => true,
                LocalLibrary::Unbound { .. } => flow != Flow::SignUp,
                _ => false,
            };
            let asks = matches!(decided, Plan::Ask { .. } | Plan::AskBeforeSignUp { .. });
            prop_assert_eq!(asks, has_foreign_notes);
            prop_assert_eq!(decided == Plan::Resume, library == LocalLibrary::Same);
            if flow == Flow::SignUp {
                prop_assert!(!matches!(decided, Plan::Ask { .. }), "sign-up asks before, never after");
            }
        }

        /// Idempotence and refinement of the device name rule.
        #[test]
        fn prop_device_names_are_clean(input in "\\PC{0,80}|[ \t\n\u{1b}\u{7f}a-z]{0,80}") {
            let name = device_name(&input);
            prop_assert_eq!(device_name(&name), name.clone());
            prop_assert!(!name.is_empty() && name.chars().count() <= 64);
            prop_assert!(!name.chars().any(char::is_control));
            prop_assert_eq!(name.trim(), name.as_str());
        }
    }
}
