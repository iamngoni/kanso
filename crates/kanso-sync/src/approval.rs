//! Device approval's cryptography (docs/ACCOUNTS.md › Unlocking a new device
//! › The protocol): the ephemeral X25519 keys, the requester's commitment,
//! the approval key and six-digit code both devices derive, the sealed
//! device name and the data key sealed to the new device. Pure: no I/O, no
//! clock; randomness only from `getrandom`.
//!
//! `N` is the new device, `A` the approver. Both derive from the same DH
//! secret and bind the account, the request and both public keys (`N`'s
//! first), so a server that substitutes either key ends up with two
//! different secrets and two different codes. `N` commits to its key before
//! it sees `A`'s and reveals it only after, so the server can't search for a
//! key whose code matches.
//!
//! The public keys in the transcript are binary JSON values, so `lp(epk)` is
//! over their 43-character base64url text, not the 32 raw bytes
//! (docs/ACCOUNTS.md › Encodings). The commitment is the exception the spec
//! states separately: it hashes the raw key bytes.

use std::fmt;

use curve25519_dalek::montgomery::MontgomeryPoint;
use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

use crate::crypto::{
    CryptoError, DataKey, FORMAT_V1, NONCE_LEN, Opened, SEAL_OVERHEAD, b64, hkdf32, lp, open_raw,
    random, seal_raw, unb64,
};

/// How many digits the code has; it is shown as two groups of three.
pub const CODE_DIGITS: usize = 6;

const COMMIT_LABEL: &[u8] = b"kanso/v1/approve-commit";
const KEY_SALT: &[u8] = b"kanso/v1/approve";
const KEY_INFO: &[u8] = b"kanso/v1/approve-key";
const CODE_LABEL: &[u8] = b"kanso/v1/approve-code";
const NAME_AAD: &[u8] = b"kanso-approve-name";
const WRAP_AAD: &[u8] = b"kanso-approve";

/// `label || 0x01 || lp(account_id)`.
fn salt(label: &[u8], account_id: &str) -> Vec<u8> {
    let mut out = label.to_vec();
    out.push(FORMAT_V1);
    lp(&mut out, account_id);
    out
}

/// `label || 0x01 || lp(…ids) || lp(epk_N) || lp(epk_A)`, with `before`
/// before the keys and `after` after them.
fn transcript(
    label: &[u8],
    before: &[&str],
    epk_new: &[u8; 32],
    epk_approver: &[u8; 32],
    after: &[&str],
) -> Vec<u8> {
    let mut out = label.to_vec();
    out.push(FORMAT_V1);
    for id in before {
        lp(&mut out, id);
    }
    lp(&mut out, &b64(epk_new));
    lp(&mut out, &b64(epk_approver));
    for id in after {
        lp(&mut out, id);
    }
    out
}

/// `N`'s commitment to its public key, as the request carries it:
/// base64url(SHA-256("kanso/v1/approve-commit" || 0x01 || epk_N)), 43
/// characters.
pub fn commitment(epk_new: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(COMMIT_LABEL);
    hasher.update([FORMAT_V1]);
    hasher.update(epk_new);
    b64(&hasher.finalize())
}

/// Whether `epk_new` is the key `commit` committed to. The approver checks
/// this itself: it doesn't trust the server's check.
pub fn matches_commitment(epk_new: &[u8; 32], commit: &str) -> bool {
    let expected = commitment(epk_new);
    expected.len() == commit.len()
        && expected
            .bytes()
            .zip(commit.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

/// A public key from its 43 base64url characters.
pub fn parse_epk(text: &str) -> Option<[u8; 32]> {
    unb64(text)?.try_into().ok()
}

/// A public key as the wire carries it.
pub fn epk_text(epk: &[u8; 32]) -> String {
    b64(epk)
}

/// Which side of the exchange a device is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// `N`, the device asking to be approved.
    New,
    /// `A`, the signed-in device that approves it.
    Approver,
}

/// One device's ephemeral X25519 key pair for one request. The secret is
/// zeroized on drop and never printed; a key pair is used once.
pub struct ApprovalKeys {
    secret: Zeroizing<[u8; 32]>,
    epk: [u8; 32],
}

impl ApprovalKeys {
    /// A new random key pair.
    pub fn generate() -> ApprovalKeys {
        ApprovalKeys::from_secret(random())
    }

    /// The key pair of this secret (tests and golden vectors).
    pub fn from_secret(secret: [u8; 32]) -> ApprovalKeys {
        let secret = Zeroizing::new(secret);
        let epk = MontgomeryPoint::mul_base_clamped(*secret).to_bytes();
        ApprovalKeys { secret, epk }
    }

    /// The public key.
    pub fn epk(&self) -> [u8; 32] {
        self.epk
    }

    /// The commitment to this key ([`commitment`]); `N` sends it first.
    pub fn commitment(&self) -> String {
        commitment(&self.epk)
    }

    /// The approval key and code, once both public keys are known: `side`
    /// is this device's, `other_epk` the other device's key. An all-zero DH
    /// result (a low-order key) is refused.
    pub fn complete(
        &self,
        side: Side,
        other_epk: &[u8; 32],
        account_id: &str,
        request_id: &str,
    ) -> Result<Approval, CryptoError> {
        let dh = Zeroizing::new(
            MontgomeryPoint(*other_epk)
                .mul_clamped(*self.secret)
                .to_bytes(),
        );
        if dh.iter().all(|&b| b == 0) {
            return Err(CryptoError::KeyMismatch);
        }
        let (epk_new, epk_approver) = match side {
            Side::New => (self.epk, *other_epk),
            Side::Approver => (*other_epk, self.epk),
        };
        let key = hkdf32(
            &dh[..],
            Some(&salt(KEY_SALT, account_id)),
            &transcript(KEY_INFO, &[request_id], &epk_new, &epk_approver, &[]),
        );
        let mut code_bytes = [0u8; 4];
        hkdf::Hkdf::<Sha256>::new(Some(&salt(CODE_LABEL, account_id)), &dh[..])
            .expand(
                &transcript(CODE_LABEL, &[request_id], &epk_new, &epk_approver, &[]),
                &mut code_bytes,
            )
            .expect("HKDF-SHA256 gives 4 bytes");
        Ok(Approval {
            key,
            code: u32::from_be_bytes(code_bytes) % 1_000_000,
            account_id: account_id.to_owned(),
            request_id: request_id.to_owned(),
            epk_new,
            epk_approver,
        })
    }
}

impl fmt::Debug for ApprovalKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApprovalKeys")
            .field("epk", &b64(&self.epk))
            .finish_non_exhaustive()
    }
}

/// What both devices derive for one request: the approval key, which seals
/// the name and the data key, and the code the person compares. Bound to the
/// account, the request and both public keys. The key is zeroized on drop;
/// `Debug` shows neither it nor the code.
pub struct Approval {
    key: Zeroizing<[u8; 32]>,
    code: u32,
    account_id: String,
    request_id: String,
    epk_new: [u8; 32],
    epk_approver: [u8; 32],
}

impl Approval {
    /// The code as six digits in two groups: "123 456".
    pub fn code(&self) -> String {
        let digits = format!("{:06}", self.code);
        format!("{} {}", &digits[..3], &digits[3..])
    }

    /// The new device's public key this approval binds.
    pub fn epk_new(&self) -> [u8; 32] {
        self.epk_new
    }

    /// The approver's public key this approval binds.
    pub fn epk_approver(&self) -> [u8; 32] {
        self.epk_approver
    }

    fn name_aad(&self) -> Vec<u8> {
        transcript(
            NAME_AAD,
            &[&self.account_id, &self.request_id],
            &self.epk_new,
            &self.epk_approver,
            &[],
        )
    }

    fn wrap_aad(&self, new_device: &str, approver_device: &str) -> Vec<u8> {
        transcript(
            WRAP_AAD,
            &[&self.account_id, &self.request_id],
            &self.epk_new,
            &self.epk_approver,
            &[new_device, approver_device],
        )
    }

    /// `N`'s device name sealed for `A`: `0x01 || nonce(24) ||
    /// XChaCha20-Poly1305(approval_key, nonce, aad, name)` as base64url, 55–400
    /// characters. The name passes [`crate::device_name`] first.
    pub fn seal_name(&self, name: &str) -> String {
        self.seal_name_with_nonce(name, &random())
    }

    pub(crate) fn seal_name_with_nonce(&self, name: &str, nonce: &[u8; NONCE_LEN]) -> String {
        let name = crate::binding::device_name(name);
        b64(&seal_raw(
            &self.key,
            nonce,
            &self.name_aad(),
            name.as_bytes(),
        ))
    }

    /// The name `N` sealed, or `None` when it doesn't open under this
    /// approval (another request, another key, or tampered).
    pub fn open_name(&self, sealed: &str) -> Option<String> {
        let sealed = unb64(sealed)?;
        match open_raw(&self.key, &self.name_aad(), &sealed) {
            Opened::Opened(plain) => String::from_utf8(plain.to_vec()).ok(),
            Opened::Undecryptable | Opened::TooNew => None,
        }
    }

    /// The data key sealed to the new device: 73 bytes, 98 base64url
    /// characters, with both device IDs in the AAD.
    pub fn wrap(&self, new_device: &str, approver_device: &str, data_key: &DataKey) -> String {
        self.wrap_with_nonce(new_device, approver_device, data_key, &random())
    }

    pub(crate) fn wrap_with_nonce(
        &self,
        new_device: &str,
        approver_device: &str,
        data_key: &DataKey,
        nonce: &[u8; NONCE_LEN],
    ) -> String {
        b64(&seal_raw(
            &self.key,
            nonce,
            &self.wrap_aad(new_device, approver_device),
            &data_key.bytes()[..],
        ))
    }

    /// Opens [`Approval::wrap`]'s output for exactly these devices; anything
    /// else is [`CryptoError::KeyMismatch`].
    pub fn unwrap(
        &self,
        new_device: &str,
        approver_device: &str,
        wrapped: &str,
    ) -> Result<DataKey, CryptoError> {
        let sealed = unb64(wrapped).ok_or(CryptoError::KeyMismatch)?;
        if sealed.len() != SEAL_OVERHEAD + 32 {
            return Err(CryptoError::KeyMismatch);
        }
        match open_raw(
            &self.key,
            &self.wrap_aad(new_device, approver_device),
            &sealed,
        ) {
            Opened::Opened(plain) if plain.len() == 32 => {
                let mut key = [0u8; 32];
                key.copy_from_slice(&plain);
                let data_key = DataKey::from_bytes(key);
                zeroize::Zeroize::zeroize(&mut key);
                Ok(data_key)
            }
            _ => Err(CryptoError::KeyMismatch),
        }
    }
}

impl fmt::Debug for Approval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Approval")
            .field("account_id", &self.account_id)
            .field("request_id", &self.request_id)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = "a_0123456789abcdef0123456789abcdef";
    const REQUEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    const NEW: &str = "d_00000000000000000000000000000001";
    const OLD: &str = "d_00000000000000000000000000000002";

    fn pair() -> (ApprovalKeys, ApprovalKeys) {
        (
            ApprovalKeys::from_secret([1; 32]),
            ApprovalKeys::from_secret([2; 32]),
        )
    }

    #[test]
    fn both_sides_derive_the_same_code_and_key() {
        let (n, a) = pair();
        let on_n = n.complete(Side::New, &a.epk(), ACCOUNT, REQUEST).unwrap();
        let on_a = a
            .complete(Side::Approver, &n.epk(), ACCOUNT, REQUEST)
            .unwrap();
        assert_eq!(on_n.code(), on_a.code());
        // Pinned by an independent implementation of RFC 7748 and
        // HKDF-SHA256 over the revision-3 transcript. This catches a change
        // from `lp(base64url(epk))` back to raw key bytes.
        assert_eq!(on_n.code(), "130 420");
        let code = on_n.code();
        assert_eq!(code.len(), 7);
        assert_eq!(code.as_bytes()[3], b' ');
        assert!(code.replace(' ', "").bytes().all(|b| b.is_ascii_digit()));
        let sealed = on_n.seal_name("Alex's MacBook Air");
        assert!((55..=400).contains(&sealed.len()), "{}", sealed.len());
        assert_eq!(
            on_a.open_name(&sealed).as_deref(),
            Some("Alex's MacBook Air")
        );
        let data_key = DataKey::from_bytes([9; 32]);
        let wrapped = on_a.wrap(NEW, OLD, &data_key);
        assert_eq!(wrapped.len(), 98);
        assert_eq!(on_n.unwrap(NEW, OLD, &wrapped).unwrap(), data_key);
        assert!(on_n.unwrap(OLD, NEW, &wrapped).is_err());
    }

    #[test]
    fn the_commitment_binds_the_key() {
        let (n, a) = pair();
        assert_eq!(n.commitment().len(), 43);
        assert!(matches_commitment(&n.epk(), &n.commitment()));
        assert!(!matches_commitment(&a.epk(), &n.commitment()));
        assert_eq!(parse_epk(&epk_text(&n.epk())), Some(n.epk()));
        assert_eq!(parse_epk("short"), None);
    }

    #[test]
    fn a_low_order_key_is_refused() {
        let (n, _) = pair();
        assert!(n.complete(Side::New, &[0; 32], ACCOUNT, REQUEST).is_err());
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// Any two keys derive the same code and key on both sides, and the
        /// sealed name and wrapped data key open only for the request and
        /// devices they were made for.
        #[test]
        fn prop_both_sides_agree_for_any_keys(
            n_secret in proptest::array::uniform32(1u8..),
            a_secret in proptest::array::uniform32(1u8..),
            request in "[A-Za-z0-9_-]{43}",
        ) {
            let n = ApprovalKeys::from_secret(n_secret);
            let a = ApprovalKeys::from_secret(a_secret);
            let on_n = n.complete(Side::New, &a.epk(), ACCOUNT, &request).unwrap();
            let on_a = a.complete(Side::Approver, &n.epk(), ACCOUNT, &request).unwrap();
            proptest::prop_assert_eq!(on_n.code(), on_a.code());
            proptest::prop_assert_eq!(on_n.epk_new(), on_a.epk_new());
            proptest::prop_assert_eq!(on_n.epk_approver(), on_a.epk_approver());
            proptest::prop_assert_eq!(on_n.wrap(NEW, OLD, &DataKey::from_bytes([3; 32])).len(), 98);
            let sealed = on_n.seal_name("Alex's MacBook Air");
            let opened = on_a.open_name(&sealed);
            proptest::prop_assert_eq!(opened.as_deref(), Some("Alex's MacBook Air"));
            let other = on_n.seal_name("someone else");
            proptest::prop_assert!(on_a.open_name(&other).is_some());
        }

        /// A substituted approver key gives another approval key and another
        /// code input: the server can't make the two sides agree by choosing
        /// the key it hands each one.
        #[test]
        fn prop_substituted_approver_key_changes_the_code(
            n_secret in proptest::array::uniform32(1u8..),
            a_secret in proptest::array::uniform32(1u8..),
            other_secret in proptest::array::uniform32(1u8..),
            request in "[A-Za-z0-9_-]{43}",
        ) {
            proptest::prop_assume!(a_secret != other_secret);
            let n = ApprovalKeys::from_secret(n_secret);
            let a = ApprovalKeys::from_secret(a_secret);
            let other = ApprovalKeys::from_secret(other_secret);
            let honest = n.complete(Side::New, &a.epk(), ACCOUNT, &request).unwrap();
            let substituted = n.complete(Side::New, &other.epk(), ACCOUNT, &request).unwrap();
            // Different keys always change the transcript, so the approval
            // key differs; a six-digit collision remains possible at 1 in a
            // million, exactly the bound the commitment gives an attacker.
            let honest_key = honest.wrap(NEW, OLD, &DataKey::from_bytes([1; 32]));
            let substituted_key = substituted.wrap(NEW, OLD, &DataKey::from_bytes([1; 32]));
            proptest::prop_assert_ne!(&honest_key, &substituted_key);
            proptest::prop_assert!(honest.unwrap(NEW, OLD, &honest_key).is_ok());
            proptest::prop_assert!(substituted.unwrap(NEW, OLD, &honest_key).is_err());
        }
    }

    #[test]
    fn a_substituted_key_gives_a_different_code() {
        let n = ApprovalKeys::from_secret([7; 32]);
        let honest = n
            .complete(
                Side::New,
                &ApprovalKeys::from_secret([8; 32]).epk(),
                ACCOUNT,
                REQUEST,
            )
            .unwrap();
        for byte in [9u8, 10, 11, 12, 13, 14, 15, 16] {
            let other = ApprovalKeys::from_secret([byte; 32]);
            let substituted = n
                .complete(Side::New, &other.epk(), ACCOUNT, REQUEST)
                .unwrap();
            assert_ne!(
                honest.code(),
                substituted.code(),
                "a substituted key should give another code (one in a million collide)"
            );
        }
    }

    #[test]
    fn a_revealed_key_that_does_not_match_the_commitment_is_refused() {
        let n = ApprovalKeys::from_secret([1; 32]);
        let other = ApprovalKeys::from_secret([2; 32]);
        let commit = n.commitment();
        assert!(matches_commitment(&n.epk(), &commit));
        assert!(!matches_commitment(&other.epk(), &commit));
        // A well-behaved server wouldn't have stored the other key; the
        // approver checks anyway and must refuse.
        assert!(!matches_commitment(&other.epk(), &commit));
    }

    #[test]
    fn a_wrap_opens_only_for_its_own_devices() {
        let n = ApprovalKeys::from_secret([1; 32]);
        let a = ApprovalKeys::from_secret([2; 32]);
        let approval = n.complete(Side::New, &a.epk(), ACCOUNT, REQUEST).unwrap();
        let approver = a
            .complete(Side::Approver, &n.epk(), ACCOUNT, REQUEST)
            .unwrap();
        let key = DataKey::from_bytes([4; 32]);
        let wrapped = approver.wrap(NEW, OLD, &key);
        assert!(approval.unwrap(NEW, OLD, &wrapped).is_ok());
        assert!(approval.unwrap(OLD, NEW, &wrapped).is_err());
        assert!(
            approval
                .unwrap(NEW, "d_00000000000000000000000000000003", &wrapped)
                .is_err()
        );
    }

    #[test]
    fn debug_shows_no_secret_and_no_code() {
        let (n, a) = pair();
        let approval = n.complete(Side::New, &a.epk(), ACCOUNT, REQUEST).unwrap();
        let printed = format!("{n:?} {approval:?}");
        assert!(!printed.contains(&approval.code()), "{printed}");
        assert!(!printed.contains(&approval.code().replace(' ', "")));
    }
}
