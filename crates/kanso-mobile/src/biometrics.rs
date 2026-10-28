//! Unlocking the locked notes with a face or a fingerprint.
//!
//! The platform keeps a secret `B` that a biometric check releases and that
//! stops working when the enrolled biometrics change (`mobile/docs/locked-
//! notes.md` has the contract). Rust keeps, in the library's secret store,
//! one item that only `B` opens: the lock keys sealed to an X25519 key whose
//! private half is sealed with `B` ([`crypto`] says how). Neither `B`, the
//! keys nor the lock password ever reach the shell.
//!
//! A check is a ticket. [`LockSession::begin_biometric_enrollment`] and
//! [`LockSession::begin_biometric_unlock`] hand one out; the plugin runs the
//! prompt and gives its answer to [`deliver`], once, within [`TICKET_LIFE`].
//!
//! While biometrics are on, every change to the session's keys seals them
//! again to the kept public key, which needs no check (the desktop's
//! `remember_keys`).

mod crypto;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_engine::lock::Session as KeySession;
use kanso_sync::Machine;
use kanso_sync::lock::LockKey;
use zeroize::Zeroizing;

use crypto::{KEY_LEN, KeyPair};

use crate::boundary::held;
use crate::errors::KansoError;
use crate::library::Inner;
use crate::locks::LockSession;
use crate::model::LockEvent;

/// How long a ticket stays good.
pub const TICKET_LIFE: Duration = Duration::from_secs(5 * 60);

/// What the plugin says happened, the `status` of [`deliver`].
pub mod status {
    /// The check passed; the secret is `B`.
    pub const OK: i32 = 0;
    /// The person cancelled.
    pub const CANCELLED: i32 = 1;
    /// The biometrics changed or were removed: `B` is gone.
    pub const INVALIDATED: i32 = 2;
    /// Biometrics can't be used now.
    pub const UNAVAILABLE: i32 = 3;
    /// The check failed.
    pub const FAILED: i32 = 4;
}

/// What [`deliver`] answers.
pub mod answer {
    /// The answer was taken.
    pub const APPLIED: i32 = 1;
    /// There is no such ticket (used, expired or never given), or its
    /// library is closed.
    pub const NO_TICKET: i32 = 0;
    /// The secret didn't open the kept keys; they are forgotten.
    pub const WRONG_SECRET: i32 = -1;
    /// Something broke: the secret store refused, or the notes were locked
    /// before the keys could be kept (enrolling), or again before the keys
    /// could open them (unlocking: the notes stay closed).
    pub const FAULT: i32 = -5;
}

/// The first line of the item.
const ITEM_V1: &str = "kanso-lock-biometric/1";

/// The first line of the sealed keys, as `kanso_sync`'s `lock-keys` item
/// has it (`save_lock_keys`): each line after it is one `LockKey::to_secret`.
const LOCK_KEYS_V1: &str = "kanso-lock-keys/1";

/// The secret store item of a library's biometric keys.
pub fn item_name(library_id: &str) -> String {
    format!("lock-biometric-v1:{library_id}")
}

/// What a ticket is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Enroll,
    Unlock,
}

struct Ticket {
    kind: Kind,
    session: Weak<Inner>,
    issued: Instant,
    /// The session's lock generation when the ticket was given: an unlock
    /// answered after the notes were locked again doesn't reopen them.
    generation: u64,
}

/// The tickets given out and not answered, by number. One process holds many
/// libraries at most briefly, so one registry serves them all.
static TICKETS: LazyLock<Mutex<HashMap<u64, Ticket>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The largest ticket: below 2^53, so every layer between here and the
/// plugin (the bridge, the platform channel, a JavaScript number in a test)
/// carries it as the same integer.
const MAX_TICKET: u64 = (1 << 53) - 1;

fn issue(kind: Kind, inner: &Arc<Inner>, now: Instant) -> u64 {
    let mut tickets = held(&TICKETS);
    tickets.retain(|_, ticket| {
        now.saturating_duration_since(ticket.issued) < TICKET_LIFE
            && ticket.session.strong_count() > 0
    });
    loop {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("the system's random source failed");
        let number = u64::from_le_bytes(bytes) & MAX_TICKET;
        if number != 0 && !tickets.contains_key(&number) {
            tickets.insert(
                number,
                Ticket {
                    kind,
                    session: Arc::downgrade(inner),
                    issued: now,
                    generation: inner.lock_gen.load(std::sync::atomic::Ordering::SeqCst),
                },
            );
            return number;
        }
    }
}

/// Takes the answer of the platform's check for `ticket`. `secret` is `B`
/// when `status` is [`status::OK`]; anything but 32 bytes then counts as a
/// failed check. Returns an [`answer`] code.
///
/// An enrolment keeps the session's keys for `B`. An unlock opens them with
/// it and adds them to the session. A secret that doesn't open them, or an
/// invalidated answer, forgets the kept keys. A ticket answers once.
pub fn deliver(ticket: u64, status: i32, secret: Option<&[u8]>) -> i32 {
    deliver_at(ticket, status, secret, Instant::now())
}

fn deliver_at(ticket: u64, status: i32, secret: Option<&[u8]>, now: Instant) -> i32 {
    let taken = held(&TICKETS).remove(&ticket);
    let Some(taken) = taken else {
        return answer::NO_TICKET;
    };
    if now.saturating_duration_since(taken.issued) >= TICKET_LIFE {
        return answer::NO_TICKET;
    }
    let Some(inner) = taken.session.upgrade() else {
        return answer::NO_TICKET;
    };
    match status {
        status::OK => {}
        status::INVALIDATED => {
            return match forget(&inner) {
                Ok(()) => answer::APPLIED,
                Err(_) => answer::FAULT,
            };
        }
        _ => return answer::APPLIED,
    }
    let b: Option<Zeroizing<[u8; KEY_LEN]>> = secret
        .and_then(|secret| <[u8; KEY_LEN]>::try_from(secret).ok())
        .map(Zeroizing::new);
    let Some(b) = b else {
        // Not a secret: the same as a check that failed.
        return answer::APPLIED;
    };
    let done = match taken.kind {
        Kind::Enroll => enroll(&inner, &b),
        Kind::Unlock => unlock(&inner, &b, taken.generation),
    };
    done.unwrap_or(answer::FAULT)
}

impl LockSession {
    /// The library's ID, which names the platform's biometric key or item.
    /// It isn't a secret and doesn't depend on the phone's name.
    pub fn library_id(&self) -> Result<String, KansoError> {
        crate::boundary::guard(|| library_id(&self.inner))
    }

    /// Whether biometrics can open the locked notes on this phone.
    pub fn biometrics_on(&self) -> Result<bool, KansoError> {
        crate::boundary::guard(|| Ok(load(&self.inner)?.is_some()))
    }

    /// Starts turning biometrics on: a ticket for the platform's enrolment.
    /// The locked notes must be open ([`KansoError::Locked`] otherwise),
    /// for their keys are what is kept.
    pub fn begin_biometric_enrollment(&self) -> Result<u64, KansoError> {
        crate::boundary::guard(|| {
            if !held(&self.inner.keys).is_unlocked() {
                return Err(KansoError::Locked);
            }
            Ok(issue(Kind::Enroll, &self.inner, Instant::now()))
        })
    }

    /// Starts opening the locked notes with biometrics: a ticket for the
    /// platform's check, or `None` when biometrics are off.
    pub fn begin_biometric_unlock(&self) -> Result<Option<u64>, KansoError> {
        crate::boundary::guard(|| {
            if load(&self.inner)?.is_none() {
                return Ok(None);
            }
            Ok(Some(issue(Kind::Unlock, &self.inner, Instant::now())))
        })
    }

    /// Forgets the keys kept for biometrics. The platform's own key or item
    /// is deleted by its plugin.
    pub fn turn_off_biometrics(&self) -> Result<(), KansoError> {
        crate::boundary::guard(|| forget(&self.inner))
    }
}

pub(crate) fn library_id(inner: &Inner) -> Result<String, KansoError> {
    // No hardware ID and no name: the ID is the library's own, and a copied
    // library file is the engine's to notice on the desktop.
    let machine = Machine {
        hardware_uuid: None,
        computer_name: String::new(),
    };
    kanso_engine::lock::library_id(&inner.store, &machine).map_err(KansoError::storage)
}

/// What the secret store keeps.
struct Item {
    public: [u8; KEY_LEN],
    sealed_private: Vec<u8>,
    ephemeral: [u8; KEY_LEN],
    sealed_keys: Vec<u8>,
}

impl Item {
    fn encode(&self) -> String {
        let b64 = |bytes: &[u8]| URL_SAFE_NO_PAD.encode(bytes);
        [
            ITEM_V1.to_owned(),
            b64(&self.public),
            b64(&self.sealed_private),
            b64(&self.ephemeral),
            b64(&self.sealed_keys),
        ]
        .join("\n")
    }

    /// `None` for an item this version can't read.
    fn decode(text: &str) -> Option<Item> {
        // `split` rather than `lines`: `encode` joins with newlines, so a
        // trailing field that is empty (no sealed keys yet) is a line of its
        // own, which `lines` would drop.
        let mut parts = text.split('\n');
        if parts.next() != Some(ITEM_V1) {
            return None;
        }
        let mut next = || URL_SAFE_NO_PAD.decode(parts.next()?).ok();
        let public = next()?.try_into().ok()?;
        let sealed_private = next()?;
        let ephemeral = next()?.try_into().ok()?;
        let sealed_keys = next()?;
        Some(Item {
            public,
            sealed_private,
            ephemeral,
            sealed_keys,
        })
    }
}

/// The kept item, with the library's ID. An item this version can't read
/// counts as none.
fn load(inner: &Inner) -> Result<Option<(String, Item)>, KansoError> {
    let id = library_id(inner)?;
    let value = inner.secrets.get(&item_name(&id))?;
    Ok(value
        .and_then(|text| Item::decode(&text))
        .map(|item| (id, item)))
}

fn forget(inner: &Inner) -> Result<(), KansoError> {
    let id = library_id(inner)?;
    inner.secrets.delete(&item_name(&id))?;
    Ok(())
}

/// Forgets the kept keys after something went wrong, so biometric unlock is
/// off and the password is the way in (a kept item of old keys would open
/// the wrong ones). A failure to forget is only logged.
pub(crate) fn forget_quietly(inner: &Inner) {
    if forget(inner).is_err() {
        eprintln!("kanso: couldn't forget the lock keys kept for biometrics");
    }
}

/// The payload: the keys as the desktop keeps them for Touch ID.
fn payload(keys: &[Arc<LockKey>]) -> Zeroizing<String> {
    let secrets: Vec<Zeroizing<String>> = keys.iter().map(|key| key.to_secret()).collect();
    // Sized up front: growing a `String` would leave the key text behind in
    // the memory it moved from.
    let size = LOCK_KEYS_V1.len() + secrets.iter().map(|secret| 1 + secret.len()).sum::<usize>();
    let mut text = Zeroizing::new(String::with_capacity(size));
    text.push_str(LOCK_KEYS_V1);
    for secret in &secrets {
        text.push('\n');
        text.push_str(secret);
    }
    text
}

fn parse_payload(text: &str) -> Vec<LockKey> {
    let mut lines = text.lines();
    if lines.next() != Some(LOCK_KEYS_V1) {
        return Vec::new();
    }
    lines
        .filter_map(|line| LockKey::from_secret(line).ok())
        .collect()
}

fn session_keys(keys: &Mutex<KeySession>) -> Vec<Arc<LockKey>> {
    held(keys).keys().to_vec()
}

/// Turns biometrics on with the session's keys and the secret `b`.
fn enroll(inner: &Inner, b: &[u8; KEY_LEN]) -> Result<i32, KansoError> {
    let keys = session_keys(&inner.keys);
    if keys.is_empty() {
        return Ok(answer::FAULT);
    }
    let id = library_id(inner)?;
    let pair = KeyPair::generate();
    let Some((ephemeral, sealed_keys)) =
        crypto::seal_to(&pair.public, &id, payload(&keys).as_bytes())
    else {
        return Ok(answer::FAULT);
    };
    let item = Item {
        public: pair.public,
        sealed_private: crypto::seal_private(b, &id, pair.secret()),
        ephemeral,
        sealed_keys,
    };
    let text = Zeroizing::new(item.encode());
    inner.secrets.set(&item_name(&id), &text)?;
    Ok(answer::APPLIED)
}

/// Opens the kept keys with `b` and adds them to the session.
fn unlock(inner: &Arc<Inner>, b: &[u8; KEY_LEN], since: u64) -> Result<i32, KansoError> {
    let Some((id, item)) = load(inner)? else {
        return Ok(answer::WRONG_SECRET);
    };
    let opened = crypto::open_private(b, &id, &item.sealed_private)
        .and_then(|private| crypto::open_from(&private, &item.ephemeral, &id, &item.sealed_keys))
        .and_then(|plain| std::str::from_utf8(&plain).ok().map(parse_payload))
        .filter(|keys| !keys.is_empty());
    let Some(keys) = opened else {
        // Not the secret these keys were sealed with: they are no use.
        inner.secrets.delete(&item_name(&id))?;
        return Ok(answer::WRONG_SECRET);
    };
    let added = LockSession {
        inner: inner.clone(),
    }
    .add_keys(keys, false, since);
    // Locked again while the check ran: the keys are dropped, the notes
    // stay closed.
    Ok(if added {
        answer::APPLIED
    } else {
        answer::FAULT
    })
}

/// Seals the session's keys again to the kept public key, when biometrics
/// are on, with no check. When that fails the kept keys are the old ones, so
/// they are forgotten: biometric unlock falls back to the password.
pub(crate) fn keep(inner: &Inner) {
    let result = (|| -> Result<(), KansoError> {
        let Some((id, mut item)) = load(inner)? else {
            return Ok(());
        };
        let keys = session_keys(&inner.keys);
        if keys.is_empty() {
            return Ok(());
        }
        let Some((ephemeral, sealed_keys)) =
            crypto::seal_to(&item.public, &id, payload(&keys).as_bytes())
        else {
            return Err(KansoError::storage("sealing the lock keys failed"));
        };
        item.ephemeral = ephemeral;
        item.sealed_keys = sealed_keys;
        let text = Zeroizing::new(item.encode());
        inner.secrets.set(&item_name(&id), &text)?;
        Ok(())
    })();
    if result.is_err() {
        eprintln!("kanso: couldn't keep the lock keys for biometrics");
        forget_quietly(inner);
    }
}

impl LockSession {
    /// Holds `keys` unless the notes were locked since generation `since`
    /// (then they are dropped, and this says `false`), and tells listeners
    /// if the notes weren't open before. `keep_for_biometrics` seals the
    /// session's keys again for biometrics.
    pub(crate) fn add_keys(
        &self,
        keys: Vec<LockKey>,
        keep_for_biometrics: bool,
        since: u64,
    ) -> bool {
        let was_open = {
            let mut held_keys = held(&self.inner.keys);
            // Checked under the mutex `close` changes it under.
            if self
                .inner
                .lock_gen
                .load(std::sync::atomic::Ordering::SeqCst)
                != since
            {
                return false;
            }
            let was_open = held_keys.is_unlocked();
            let now = Instant::now();
            for key in keys {
                held_keys.add(key, now);
            }
            was_open
        };
        // Notes put in a locked notebook before a key was held are sealed
        // now.
        self.seal_unsealed();
        if keep_for_biometrics {
            keep(&self.inner);
        }
        if !was_open {
            self.inner.lock_events.send(LockEvent::Unlocked);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_sync::{MemorySecrets, MemoryServer};

    use super::*;
    use crate::Library;

    const B: [u8; KEY_LEN] = [7; KEY_LEN];

    fn library(dir: &std::path::Path) -> Library {
        Library::open_for_test(
            &dir.join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap()
    }

    // no-prop-test: the clock decides; two instants either side of the
    // limit and one answer each.
    #[test]
    fn a_ticket_expires_after_five_minutes_and_answers_once() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(dir.path());
        let now = Instant::now();
        let fresh = issue(Kind::Enroll, &library.inner, now);
        let stale = issue(Kind::Enroll, &library.inner, now);
        let used = issue(Kind::Enroll, &library.inner, now);
        // Nothing is open, so enrolling is a fault, but the ticket is spent.
        let just_inside = now + TICKET_LIFE - Duration::from_secs(1);
        assert_eq!(
            deliver_at(fresh, status::OK, Some(&B), just_inside),
            answer::FAULT
        );
        assert_eq!(
            deliver_at(fresh, status::OK, Some(&B), just_inside),
            answer::NO_TICKET
        );
        assert_eq!(
            deliver_at(stale, status::OK, Some(&B), now + TICKET_LIFE),
            answer::NO_TICKET
        );
        assert_eq!(
            deliver_at(used, status::CANCELLED, None, now),
            answer::APPLIED
        );
        assert_eq!(
            deliver_at(used, status::CANCELLED, None, now),
            answer::NO_TICKET
        );
        assert_eq!(deliver(0, status::OK, Some(&B)), answer::NO_TICKET);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(8))]

        /// Every ticket is a nonzero integer below 2^53, and none repeats.
        #[test]
        fn prop_tickets_are_small_distinct_and_never_zero(n in 1usize..60) {
            let dir = tempfile::tempdir().unwrap();
            let library = library(dir.path());
            let mut seen = std::collections::HashSet::new();
            for _ in 0..n {
                let ticket = issue(Kind::Unlock, &library.inner, Instant::now());
                proptest::prop_assert!(ticket != 0 && ticket <= MAX_TICKET);
                proptest::prop_assert!(seen.insert(ticket));
            }
        }
    }

    // no-prop-test: a ticket of a library that is closed.
    #[test]
    fn a_ticket_of_a_closed_library_is_no_ticket() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(dir.path());
        let ticket = issue(Kind::Unlock, &library.inner, Instant::now());
        drop(library);
        assert_eq!(deliver(ticket, status::OK, Some(&B)), answer::NO_TICKET);
    }

    // no-prop-test: an item with nothing sealed yet is still read back.
    #[test]
    fn an_item_with_no_sealed_keys_round_trips() {
        let item = Item {
            public: [7; KEY_LEN],
            sealed_private: Vec::new(),
            ephemeral: [9; KEY_LEN],
            sealed_keys: Vec::new(),
        };
        let back = Item::decode(&item.encode()).expect("an item with empty fields reads back");
        assert_eq!(back.public, item.public);
        assert_eq!(back.sealed_private, item.sealed_private);
        assert_eq!(back.ephemeral, item.ephemeral);
        assert_eq!(back.sealed_keys, item.sealed_keys);
    }

    proptest::proptest! {
        /// An item reads back as it was written, and no other text reads.
        #[test]
        fn prop_an_item_round_trips(
            public in proptest::array::uniform32(proptest::prelude::any::<u8>()),
            ephemeral in proptest::array::uniform32(proptest::prelude::any::<u8>()),
            private in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..80),
            keys in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..400),
            junk in "\\PC{0,60}",
        ) {
            let item = Item { public, sealed_private: private, ephemeral, sealed_keys: keys };
            let back = Item::decode(&item.encode()).unwrap();
            proptest::prop_assert_eq!(back.public, item.public);
            proptest::prop_assert_eq!(back.sealed_private, item.sealed_private);
            proptest::prop_assert_eq!(back.ephemeral, item.ephemeral);
            proptest::prop_assert_eq!(back.sealed_keys, item.sealed_keys);
            proptest::prop_assert!(Item::decode(&junk).is_none());
        }
    }
}
