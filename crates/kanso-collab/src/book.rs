//! Notebook keys (issue #390 › Shared notebooks: keys). One random key per
//! notebook per epoch, `K_b[e]`, is wrapped to each member exactly as a
//! shared note's key is ([`crate::keys`]); every note's key and the
//! manifest's key are derived from it with HKDF, so adding a member costs
//! one wrap however many notes the notebook holds. Removing a member starts
//! epoch `e + 1` with a fresh, unrelated key. A history link seals the
//! previous epoch's key under the new one, so members still read old work.
//!
//! Derived keys are ordinary [`NoteKey`]s, so the sealing code that serves
//! single shared notes serves notebook notes unchanged.

use zeroize::Zeroizing;

use crate::book_model::{BookHistory, BookKey};
use crate::codec::{FORMAT_V1, b64, hex, hkdf32, open, put_lp, random, seal, sha256, unb64};
use crate::identity::Identity;
use crate::keys::{NoteKey, box_open, box_seal};
use crate::model::Role;

const WRAP_INFO: &[u8] = b"kanso/v1/book-key";
const NOTE_INFO: &[u8] = b"kanso/v1/book-note";
const MANIFEST_INFO: &[u8] = b"kanso/v1/book-manifest";

impl BookKey {
    /// A fresh random key, for a new notebook or a new epoch.
    pub fn generate() -> BookKey {
        BookKey::from_bytes(random::<32>())
    }

    /// `HKDF(K_b[e], salt = book_share_id, info = "kanso/v1/book-note" ||
    /// note_uuid)`: the key a note's updates and snapshots are sealed with
    /// in this epoch. Differs per note, per notebook and per epoch.
    pub fn note_key(&self, share_id: &str, note_uuid: &str) -> NoteKey {
        let info = [NOTE_INFO, note_uuid.as_bytes()].concat();
        let key = hkdf32(&self.0[..], Some(share_id.as_bytes()), &info);
        NoteKey::from_bytes(*key)
    }

    /// `HKDF(K_b[e], info = "kanso/v1/book-manifest")`: seals the notebook's
    /// manifest document.
    pub fn manifest_key(&self) -> NoteKey {
        NoteKey::from_bytes(*hkdf32(&self.0[..], None, MANIFEST_INFO))
    }

    /// `hex(SHA-256("kanso-book-key-commit" || 0x01 || lp(share_id) ||
    /// u32(epoch) || K_b[e]))`: what the signed member list carries, so a
    /// member can tell the key they unwrapped is the one the list names.
    pub fn commit(&self, share_id: &str, epoch: u32) -> String {
        let mut input = b"kanso-book-key-commit".to_vec();
        input.push(FORMAT_V1);
        put_lp(&mut input, share_id.as_bytes());
        input.extend_from_slice(&epoch.to_be_bytes());
        input.extend_from_slice(&self.0[..]);
        hex(&sha256(&[&input]))
    }
}

fn wrap_aad(share_id: &str, epoch: u32, member: &str, role: Role) -> Vec<u8> {
    let mut aad = b"kanso-book-key".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    put_lp(&mut aad, member.as_bytes());
    aad.push(role.byte());
    aad
}

/// Wraps `key` to a member, as base64url. The associated data binds the
/// notebook share, the epoch, the member and the role. `None` for a box key
/// that gives an all-zero agreement.
pub fn wrap(
    key: &BookKey,
    share_id: &str,
    epoch: u32,
    member: &str,
    role: Role,
    member_box: &[u8; 32],
) -> Option<String> {
    box_seal(
        member_box,
        WRAP_INFO,
        &wrap_aad(share_id, epoch, member, role),
        &key.0[..],
    )
    .map(|sealed| b64(&sealed))
}

/// Opens a wrap made for `identity` as `member` with `role`; `None` for
/// anything else.
pub fn unwrap(
    identity: &Identity,
    wrapped: &str,
    share_id: &str,
    epoch: u32,
    member: &str,
    role: Role,
) -> Option<BookKey> {
    let plain = box_open(
        identity,
        WRAP_INFO,
        &wrap_aad(share_id, epoch, member, role),
        &unb64(wrapped)?,
    )?;
    Some(BookKey::from_bytes(plain[..].try_into().ok()?))
}

fn history_aad(share_id: &str, epoch: u32) -> Vec<u8> {
    let mut aad = b"kanso-book-history".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    aad
}

/// The key a history link is sealed under, derived from `K_b[epoch]` so the
/// notebook key itself only ever feeds HKDF and never encrypts directly.
fn history_key(key: &BookKey) -> Zeroizing<[u8; 32]> {
    hkdf32(&key.0[..], None, b"kanso/v1/book-history")
}

/// The history link stored with epoch `epoch` (at least 2): `K_b[epoch - 1]`
/// sealed under a key derived from `K_b[epoch]`.
pub fn history_link(share_id: &str, epoch: u32, new: &BookKey, previous: &BookKey) -> Vec<u8> {
    seal(
        &history_key(new),
        &history_aad(share_id, epoch),
        &previous.0[..],
    )
}

/// Opens epoch `epoch`'s history link with `K_b[epoch]`, giving
/// `K_b[epoch - 1]`; `None` for a link of another epoch or notebook, a
/// different key, or damage.
pub fn open_history_link(
    share_id: &str,
    epoch: u32,
    new: &BookKey,
    link: &[u8],
) -> Option<BookKey> {
    let plain = open(&history_key(new), &history_aad(share_id, epoch), link)?;
    Some(BookKey::from_bytes(plain[..].try_into().ok()?))
}

/// Walks back from the key a member holds for `epoch` through `links`
/// (epoch to link) as far as they chain unbroken, giving every earlier
/// epoch's key. A missing or damaged link ends the walk there.
pub fn walk_history<'a>(
    share_id: &str,
    epoch: u32,
    key: &BookKey,
    links: impl Fn(u32) -> Option<&'a [u8]>,
) -> BookHistory {
    let mut history = BookHistory::default();
    let mut at = epoch;
    let mut current = key.clone();
    while at > 1 {
        let Some(previous) = links(at).and_then(|l| open_history_link(share_id, at, &current, l))
        else {
            break;
        };
        at -= 1;
        history.keys.insert(at, previous.clone());
        current = previous;
    }
    history
}

fn vault_aad(share_id: &str, epoch: u32) -> Vec<u8> {
    let mut aad = b"kanso-book-vault".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    aad
}

/// A notebook key sealed for the library under `vault_key`.
pub fn vault_seal(vault_key: &[u8; 32], share_id: &str, epoch: u32, key: &BookKey) -> Vec<u8> {
    seal(vault_key, &vault_aad(share_id, epoch), &key.0[..])
}

pub fn vault_open(
    vault_key: &[u8; 32],
    share_id: &str,
    epoch: u32,
    sealed: &[u8],
) -> Option<BookKey> {
    let plain: Zeroizing<Vec<u8>> = open(vault_key, &vault_aad(share_id, epoch), sealed)?;
    Some(BookKey::from_bytes(plain[..].try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{open as open_sealed, seal as seal_bytes};
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn key(byte: u8) -> BookKey {
        BookKey::from_bytes([byte; 32])
    }

    /// Fixed vectors, computed independently with Python's `hmac` (RFC 5869
    /// HKDF-SHA256) and `hashlib`, so a change to any label, salt or
    /// ordering shows up here.
    #[test]
    fn derivation_vectors() {
        let k = key(0x07);
        assert_eq!(
            hex(k.note_key("share1", "note-a").bytes()),
            "47fd990cc7195c501696a8f489f4152d24d9dfbf78d282236091b890a0fe7ecc"
        );
        assert_eq!(
            hex(k.manifest_key().bytes()),
            "6c31a7a8de4cac0b5ab5431b6ccbf183e245730a42fa9397b92f4002d085d2e2"
        );
        assert_eq!(
            k.commit("share1", 3),
            "62bdb65f4fcd4377ab7abe6f61273eefc8f3f721bff612ec9629f080ac2db1fe"
        );
    }

    proptest! {
        #![proptest_config(crate::tests::config(48))]

        /// A wrap opens for its member, notebook, epoch and role only.
        #[test]
        fn prop_wrap_opens_only_for_what_it_binds(seed in any::<[u8; 32]>(), other in any::<[u8; 32]>(),
                                                   raw in any::<[u8; 32]>(), epoch in 1u32..1000,
                                                   role in prop::sample::select(Role::ALL.to_vec())) {
            let member = Identity::from_secrets([1; 32], seed);
            let stranger = Identity::from_secrets([1; 32], other);
            let k = BookKey::from_bytes(raw);
            let wrapped = wrap(&k, "s1", epoch, "a_m", role, &member.box_pub()).unwrap();
            prop_assert_eq!(wrapped.len(), 140);
            prop_assert_eq!(unwrap(&member, &wrapped, "s1", epoch, "a_m", role), Some(k));
            prop_assert!(unwrap(&member, &wrapped, "s2", epoch, "a_m", role).is_none());
            prop_assert!(unwrap(&member, &wrapped, "s1", epoch + 1, "a_m", role).is_none());
            prop_assert!(unwrap(&member, &wrapped, "s1", epoch, "a_x", role).is_none());
            for r in Role::ALL.into_iter().filter(|r| *r != role) {
                prop_assert!(unwrap(&member, &wrapped, "s1", epoch, "a_m", r).is_none());
            }
            if stranger.box_pub() != member.box_pub() {
                prop_assert!(unwrap(&stranger, &wrapped, "s1", epoch, "a_m", role).is_none());
            }
        }

        /// A notebook wrap is not a note wrap: the two never open as each
        /// other's.
        #[test]
        fn prop_book_wraps_are_not_note_wraps(seed in any::<[u8; 32]>(), raw in any::<[u8; 32]>(), epoch in 1u32..100) {
            let member = Identity::from_secrets([1; 32], seed);
            let book = wrap(&BookKey::from_bytes(raw), "s", epoch, "a", Role::Edit, &member.box_pub()).unwrap();
            prop_assert!(crate::keys::unwrap(&member, &book, "s", epoch, "a", Role::Edit).is_none());
            let note = crate::keys::wrap(&NoteKey::from_bytes(raw), "s", epoch, "a", Role::Edit, &member.box_pub()).unwrap();
            prop_assert!(unwrap(&member, &note, "s", epoch, "a", Role::Edit).is_none());
        }

        /// Derived keys differ per note, per notebook and per epoch's key,
        /// and are never the notebook key or the manifest key.
        #[test]
        fn prop_derived_keys_differ(a in any::<[u8; 32]>(), b in any::<[u8; 32]>(),
                                    n1 in "[a-z0-9-]{1,36}", n2 in "[a-z0-9-]{1,36}") {
            let ka = BookKey::from_bytes(a);
            let kb = BookKey::from_bytes(b);
            prop_assert_eq!(ka.note_key("s", &n1) == ka.note_key("s", &n2), n1 == n2);
            prop_assert_ne!(ka.note_key("s", &n1), ka.note_key("t", &n1));
            prop_assert_eq!(ka.note_key("s", &n1) == kb.note_key("s", &n1), a == b);
            prop_assert_eq!(ka.manifest_key() == kb.manifest_key(), a == b);
            let (nk, mk) = (ka.note_key("s", &n1), ka.manifest_key());
            prop_assert_ne!(nk.bytes(), mk.bytes());
            prop_assert_ne!(nk.bytes(), ka.bytes());
            prop_assert_ne!(mk.bytes(), ka.bytes());
            prop_assert_eq!(ka.note_key("s", &n1), ka.note_key("s", &n1));
        }

        /// The commitment names one key, one epoch and one notebook.
        #[test]
        fn prop_commitments_differ(a in any::<[u8; 32]>(), b in any::<[u8; 32]>(), epoch in 1u32..100) {
            let ka = BookKey::from_bytes(a);
            let kb = BookKey::from_bytes(b);
            prop_assert_eq!(ka.commit("s", epoch) == kb.commit("s", epoch), a == b);
            prop_assert_ne!(ka.commit("s", epoch), ka.commit("s", epoch + 1));
            prop_assert_ne!(ka.commit("s", epoch), ka.commit("t", epoch));
            prop_assert_ne!(ka.commit("s", epoch), NoteKey::from_bytes(a).commit("s", epoch));
        }

        /// An old epoch cannot decrypt a new epoch's notes; the right
        /// epoch's key round-trips.
        #[test]
        fn prop_old_epoch_cannot_open_new_notes(old in any::<[u8; 32]>(), new in any::<[u8; 32]>(),
                                                 note in "[a-z0-9-]{1,36}", plain in prop::collection::vec(any::<u8>(), 0..128),
                                                 aad in prop::collection::vec(any::<u8>(), 0..16)) {
            prop_assume!(old != new);
            let (old, new) = (BookKey::from_bytes(old), BookKey::from_bytes(new));
            let sealed = seal_bytes(new.note_key("s", &note).bytes(), &aad, &plain);
            prop_assert_eq!(open_sealed(new.note_key("s", &note).bytes(), &aad, &sealed).map(|p| p.to_vec()), Some(plain.clone()));
            let other_note = format!("{note}x");
            prop_assert!(open_sealed(old.note_key("s", &note).bytes(), &aad, &sealed).is_none());
            prop_assert!(open_sealed(new.note_key("s", &other_note).bytes(), &aad, &sealed).is_none());
            prop_assert!(open_sealed(new.manifest_key().bytes(), &aad, &sealed).is_none());
        }

        /// A history link opens with the next epoch's key only, for its own
        /// epoch and notebook only.
        #[test]
        fn prop_history_link_round_trips(a in any::<[u8; 32]>(), b in any::<[u8; 32]>(), epoch in 2u32..100) {
            prop_assume!(a != b);
            let (prev, new) = (BookKey::from_bytes(a), BookKey::from_bytes(b));
            let link = history_link("s", epoch, &new, &prev);
            prop_assert_eq!(open_history_link("s", epoch, &new, &link), Some(prev.clone()));
            prop_assert!(open_history_link("s", epoch, &prev, &link).is_none());
            prop_assert!(open_history_link("s", epoch + 1, &new, &link).is_none());
            prop_assert!(open_history_link("t", epoch, &new, &link).is_none());
        }

        /// A member holding the newest epoch reads every earlier epoch
        /// through the links, and a gap stops the walk at the gap.
        #[test]
        fn prop_history_walks_back(seeds in prop::collection::vec(any::<[u8; 32]>(), 2..8), cut in any::<prop::sample::Index>()) {
            let keys: Vec<BookKey> = seeds.iter().map(|s| BookKey::from_bytes(*s)).collect();
            let n = keys.len() as u32;
            let mut links: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
            for e in 2..=n {
                links.insert(e, history_link("s", e, &keys[e as usize - 1], &keys[e as usize - 2]));
            }
            let full = walk_history("s", n, &keys[n as usize - 1], |e| links.get(&e).map(Vec::as_slice));
            for e in 1..n {
                prop_assert_eq!(full.keys.get(&e), Some(&keys[e as usize - 1]));
            }
            let gap = 2 + cut.index((n - 1) as usize) as u32;
            links.remove(&gap);
            let part = walk_history("s", n, &keys[n as usize - 1], |e| links.get(&e).map(Vec::as_slice));
            prop_assert_eq!(part.keys.len() as u32, n - gap);
            prop_assert!(part.keys.keys().all(|e| *e >= gap));
        }

        /// The vault opens its own notebook's epoch only, and is not the
        /// note vault.
        #[test]
        fn prop_vault_round_trips(vault in any::<[u8; 32]>(), raw in any::<[u8; 32]>(), epoch in 1u32..50) {
            let k = BookKey::from_bytes(raw);
            let sealed = vault_seal(&vault, "s", epoch, &k);
            prop_assert_eq!(vault_open(&vault, "s", epoch, &sealed), Some(k));
            prop_assert!(vault_open(&vault, "s", epoch + 1, &sealed).is_none());
            prop_assert!(vault_open(&vault, "t", epoch, &sealed).is_none());
            prop_assert!(crate::keys::vault_open(&vault, "s", epoch, &sealed).is_none());
        }
    }

    #[test]
    fn generated_keys_are_fresh_and_redacted() {
        assert_ne!(BookKey::generate(), BookKey::generate());
        assert_eq!(format!("{:?}", key(1)), "BookKey(<redacted>)");
    }
}
