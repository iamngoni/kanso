//! Note keys (docs/ACCOUNTS.md › Collaboration › Keys for shared notes): a
//! random key per shared note per epoch, wrapped to each member's box key
//! with an ephemeral X25519 agreement, committed to in the owner's signed
//! member list, and sealed at rest on the device under `vault_key`.
//!
//! Notifications use the same wrap with their own info and AAD.

use std::fmt;

use curve25519_dalek::montgomery::MontgomeryPoint;
use zeroize::Zeroizing;

use crate::codec::{
    FORMAT_V1, NONCE_LEN, aead_open, aead_seal, b64, hex, hkdf32, open, put_lp, random, seal,
    sha256, unb64,
};
use crate::identity::{Identity, x25519};
use crate::model::Role;

/// A note's key for one epoch. Zeroized on drop; `Debug` shows nothing.
#[derive(Clone)]
pub struct NoteKey(Zeroizing<[u8; 32]>);

impl fmt::Debug for NoteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NoteKey(<redacted>)")
    }
}

impl PartialEq for NoteKey {
    fn eq(&self, other: &NoteKey) -> bool {
        self.0[..] == other.0[..]
    }
}

impl NoteKey {
    pub fn generate() -> NoteKey {
        NoteKey(Zeroizing::new(random::<32>()))
    }

    pub fn from_bytes(bytes: [u8; 32]) -> NoteKey {
        NoteKey(Zeroizing::new(bytes))
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// `hex(SHA-256("kanso-note-key-commit" || 0x01 || lp(share_id) ||
    /// u32(epoch) || note_key))`.
    pub fn commit(&self, share_id: &str, epoch: u32) -> String {
        let mut input = b"kanso-note-key-commit".to_vec();
        input.push(FORMAT_V1);
        put_lp(&mut input, share_id.as_bytes());
        input.extend_from_slice(&epoch.to_be_bytes());
        input.extend_from_slice(&self.0[..]);
        hex(&sha256(&[&input]))
    }
}

/// A wrapped value's length: format, ephemeral key, nonce, 32 bytes, tag.
pub const WRAPPED_LEN: usize = 1 + 32 + NONCE_LEN + 32 + 16;

/// `0x01 || E || nonce(24) || XChaCha20-Poly1305(k, nonce, aad, plaintext)`
/// with `k = HKDF(ikm X25519(e, B), salt E || B, info)` for a fresh `(e, E)`.
pub(crate) fn box_seal(
    their_box: &[u8; 32],
    info: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> Option<Vec<u8>> {
    let ephemeral = Zeroizing::new(random::<32>());
    let ephemeral_pub = MontgomeryPoint::mul_base_clamped(*ephemeral).to_bytes();
    let shared = x25519(&ephemeral, their_box)?;
    let salt = [&ephemeral_pub[..], &their_box[..]].concat();
    let key = hkdf32(&shared[..], Some(&salt), info);
    let nonce = random::<NONCE_LEN>();
    let mut out = vec![FORMAT_V1];
    out.extend_from_slice(&ephemeral_pub);
    out.extend_from_slice(&nonce);
    out.extend(aead_seal(&key, &nonce, aad, plaintext));
    Some(out)
}

pub(crate) fn box_open(
    identity: &Identity,
    info: &[u8],
    aad: &[u8],
    sealed: &[u8],
) -> Option<Zeroizing<Vec<u8>>> {
    if sealed.len() < 1 + 32 + NONCE_LEN + 16 || sealed[0] != FORMAT_V1 {
        return None;
    }
    let ephemeral_pub: [u8; 32] = sealed[1..33].try_into().ok()?;
    let nonce: [u8; NONCE_LEN] = sealed[33..33 + NONCE_LEN].try_into().ok()?;
    let shared = identity.agree(&ephemeral_pub)?;
    let salt = [&ephemeral_pub[..], &identity.box_pub()[..]].concat();
    let key = hkdf32(&shared[..], Some(&salt), info);
    aead_open(&key, &nonce, aad, &sealed[33 + NONCE_LEN..])
}

fn wrap_aad(share_id: &str, epoch: u32, member: &str, role: Role) -> Vec<u8> {
    let mut aad = b"kanso-note-key".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    put_lp(&mut aad, member.as_bytes());
    aad.push(role.byte());
    aad
}

/// Wraps `key` to a member, as base64url; `None` for a box key that gives an
/// all-zero agreement (never a real one).
pub fn wrap(
    key: &NoteKey,
    share_id: &str,
    epoch: u32,
    member: &str,
    role: Role,
    member_box: &[u8; 32],
) -> Option<String> {
    box_seal(
        member_box,
        b"kanso/v1/note-key",
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
) -> Option<NoteKey> {
    let sealed = unb64(wrapped)?;
    let plain = box_open(
        identity,
        b"kanso/v1/note-key",
        &wrap_aad(share_id, epoch, member, role),
        &sealed,
    )?;
    Some(NoteKey::from_bytes(plain[..].try_into().ok()?))
}

fn notice_aad(recipient: &str, share_id: &str, sender: &str) -> Vec<u8> {
    let mut aad = b"kanso-notice".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, recipient.as_bytes());
    put_lp(&mut aad, share_id.as_bytes());
    put_lp(&mut aad, sender.as_bytes());
    aad
}

/// Seals a mention's notification to its recipient.
pub fn seal_notice(
    recipient: &str,
    recipient_box: &[u8; 32],
    share_id: &str,
    sender: &str,
    plaintext: &[u8],
) -> Option<String> {
    box_seal(
        recipient_box,
        b"kanso/v1/notice",
        &notice_aad(recipient, share_id, sender),
        plaintext,
    )
    .map(|sealed| b64(&sealed))
}

pub fn open_notice(
    identity: &Identity,
    recipient: &str,
    share_id: &str,
    sender: &str,
    sealed: &str,
) -> Option<Zeroizing<Vec<u8>>> {
    box_open(
        identity,
        b"kanso/v1/notice",
        &notice_aad(recipient, share_id, sender),
        &unb64(sealed)?,
    )
}

fn vault_aad(share_id: &str, epoch: u32) -> Vec<u8> {
    let mut aad = b"kanso-vault".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    aad
}

/// A note key sealed for the library (`collab_keys`).
pub fn vault_seal(vault_key: &[u8; 32], share_id: &str, epoch: u32, key: &NoteKey) -> Vec<u8> {
    seal(vault_key, &vault_aad(share_id, epoch), &key.0[..])
}

pub fn vault_open(
    vault_key: &[u8; 32],
    share_id: &str,
    epoch: u32,
    sealed: &[u8],
) -> Option<NoteKey> {
    let plain = open(vault_key, &vault_aad(share_id, epoch), sealed)?;
    Some(NoteKey::from_bytes(plain[..].try_into().ok()?))
}

/// An invite link's secret and what the server keeps of it.
pub struct LinkSecret(Zeroizing<[u8; 32]>);

impl fmt::Debug for LinkSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LinkSecret(<redacted>)")
    }
}

impl LinkSecret {
    pub fn generate() -> LinkSecret {
        LinkSecret(Zeroizing::new(random::<32>()))
    }

    pub fn parse(text: &str) -> Option<LinkSecret> {
        Some(LinkSecret(Zeroizing::new(crate::codec::unb64_array::<32>(
            text,
        )?)))
    }

    pub fn to_text(&self) -> String {
        b64(&self.0[..])
    }

    /// `HKDF(secret, info "kanso/v1/join-proof")`, as base64url.
    pub fn proof(&self) -> String {
        b64(&hkdf32(&self.0[..], None, b"kanso/v1/join-proof")[..])
    }

    /// `hex(SHA-256(proof))`, what the server keeps.
    pub fn proof_hash(&self) -> String {
        let proof = hkdf32(&self.0[..], None, b"kanso/v1/join-proof");
        hex(&sha256(&[&proof[..]]))
    }
}

/// `<server>/j/<link_id>#<secret>`.
pub fn invite_link(server: &str, link_id: &str, secret: &LinkSecret) -> String {
    format!(
        "{}/j/{}#{}",
        server.trim_end_matches('/'),
        link_id,
        secret.to_text()
    )
}

/// The server, link ID and secret of an invite link, or `None`.
pub fn parse_invite_link(text: &str) -> Option<(String, String, LinkSecret)> {
    let (before, secret) = text.trim().split_once('#')?;
    let (server, link_id) = before.rsplit_once("/j/")?;
    if !crate::codec::is_id(link_id) || kanso_sync::check_endpoint(server).is_err() {
        return None;
    }
    Some((
        server.trim_end_matches('/').to_owned(),
        link_id.to_owned(),
        LinkSecret::parse(secret)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(crate::tests::config(48))]

        /// A wrap opens for its member, note, epoch and role only: the
        /// associated data binds all four.
        #[test]
        fn prop_wrap_opens_only_for_what_it_binds(seed in any::<[u8; 32]>(), other in any::<[u8; 32]>(),
                                                   epoch in 1u32..1000, role in prop::sample::select(Role::ALL.to_vec())) {
            let member = Identity::from_secrets([1; 32], seed);
            let stranger = Identity::from_secrets([1; 32], other);
            let key = NoteKey::generate();
            let wrapped = wrap(&key, "s1", epoch, "a_m", role, &member.box_pub()).unwrap();
            prop_assert_eq!(wrapped.len(), 140);
            prop_assert_eq!(unwrap(&member, &wrapped, "s1", epoch, "a_m", role), Some(key.clone()));
            prop_assert!(unwrap(&member, &wrapped, "s2", epoch, "a_m", role).is_none());
            prop_assert!(unwrap(&member, &wrapped, "s1", epoch + 1, "a_m", role).is_none());
            prop_assert!(unwrap(&member, &wrapped, "s1", epoch, "a_x", role).is_none());
            for other_role in Role::ALL.into_iter().filter(|r| *r != role) {
                prop_assert!(unwrap(&member, &wrapped, "s1", epoch, "a_m", other_role).is_none());
            }
            if stranger.box_pub() != member.box_pub() {
                prop_assert!(unwrap(&stranger, &wrapped, "s1", epoch, "a_m", role).is_none());
            }
        }

        /// The commitment names one key: two keys never commit alike, and an
        /// epoch or note changes it.
        #[test]
        fn prop_commitments_differ(a in any::<[u8; 32]>(), b in any::<[u8; 32]>(), epoch in 1u32..100) {
            let ka = NoteKey::from_bytes(a);
            let kb = NoteKey::from_bytes(b);
            prop_assert_eq!(ka.commit("s", epoch) == kb.commit("s", epoch), a == b);
            prop_assert_ne!(ka.commit("s", epoch), ka.commit("s", epoch + 1));
            prop_assert_ne!(ka.commit("s", epoch), ka.commit("t", epoch));
        }

        /// The vault opens its own note's epoch only.
        #[test]
        fn prop_vault_round_trips(vault in any::<[u8; 32]>(), key in any::<[u8; 32]>(), epoch in 1u32..50) {
            let key = NoteKey::from_bytes(key);
            let sealed = vault_seal(&vault, "s", epoch, &key);
            prop_assert_eq!(vault_open(&vault, "s", epoch, &sealed), Some(key));
            prop_assert!(vault_open(&vault, "s", epoch + 1, &sealed).is_none());
        }
    }

    #[test]
    fn invite_links_round_trip() {
        let secret = LinkSecret::generate();
        let id = crate::codec::new_id();
        let link = invite_link("https://sync.kansoapp.app/", &id, &secret);
        let (server, link_id, parsed) = parse_invite_link(&link).unwrap();
        assert_eq!(server, "https://sync.kansoapp.app");
        assert_eq!(link_id, id);
        assert_eq!(parsed.proof_hash(), secret.proof_hash());
        assert!(parse_invite_link("https://sync.kansoapp.app/j/short#x").is_none());
        assert!(parse_invite_link("http://example.com/j/AAAAAAAAAAAAAAAAAAAAAA#AAAA").is_none());
    }
}
