//! The cryptography of biometric unlock, on its own so it can be read whole.
//!
//! The platform keeps a secret `B` that only a biometric check releases.
//! Enrolling makes an X25519 pair: the private half is sealed with `B`, and
//! the lock keys are sealed to the public half. So the keys can be sealed
//! again at any time with no check, and opened only with `B`.
//!
//! XChaCha20-Poly1305 seals, HKDF-SHA256 derives every key, and the library's
//! ID is bound in as associated data, so an item of one library never opens
//! in another. Every sealed value is `nonce (24 bytes) || ciphertext || tag`.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use curve25519_dalek::montgomery::MontgomeryPoint;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

/// The length of `B`, a private key and a public key.
pub const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

const PRIVATE_INFO: &[u8] = b"kanso lock biometric private key v1";
const KEYS_INFO: &[u8] = b"kanso lock biometric keys v1";

/// An X25519 key pair.
pub struct KeyPair {
    secret: Zeroizing<[u8; KEY_LEN]>,
    pub public: [u8; KEY_LEN],
}

impl KeyPair {
    /// A new random pair.
    pub fn generate() -> KeyPair {
        let secret = Zeroizing::new(random::<KEY_LEN>());
        let public = MontgomeryPoint::mul_base_clamped(*secret).to_bytes();
        KeyPair { secret, public }
    }

    /// The private half, to be sealed.
    pub fn secret(&self) -> &[u8; KEY_LEN] {
        &self.secret
    }
}

fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    bytes
}

fn derive(ikm: &[u8], salt: Option<&[u8]>, info: &[u8]) -> Zeroizing<[u8; KEY_LEN]> {
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    Hkdf::<Sha256>::new(salt, ikm)
        .expand(info, &mut out[..])
        .expect("HKDF-SHA256 gives 32 bytes");
    out
}

fn seal(key: &[u8; KEY_LEN], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let nonce = random::<NONCE_LEN>();
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("a 32-byte key");
    let body = cipher
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("XChaCha20-Poly1305 seals every length used here");
    let mut out = Vec::with_capacity(NONCE_LEN + body.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&body);
    out
}

fn open(key: &[u8; KEY_LEN], aad: &[u8], sealed: &[u8]) -> Option<Zeroizing<Vec<u8>>> {
    if sealed.len() < NONCE_LEN + TAG_LEN {
        return None;
    }
    let (nonce, body) = sealed.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce.try_into().ok()?;
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("a 32-byte key");
    cipher
        .decrypt(&XNonce::from(nonce), Payload { msg: body, aad })
        .ok()
        .map(Zeroizing::new)
}

fn aad(label: &[u8], library_id: &str) -> Vec<u8> {
    let mut aad = label.to_vec();
    aad.push(0);
    aad.extend_from_slice(library_id.as_bytes());
    aad
}

/// Seals the private key with the platform's secret `b`.
pub fn seal_private(b: &[u8; KEY_LEN], library_id: &str, private: &[u8; KEY_LEN]) -> Vec<u8> {
    seal(
        &derive(b, None, PRIVATE_INFO),
        &aad(PRIVATE_INFO, library_id),
        private,
    )
}

/// The private key `b` unseals; `None` when `b` isn't the secret it was
/// sealed with, or the value is damaged.
pub fn open_private(
    b: &[u8; KEY_LEN],
    library_id: &str,
    sealed: &[u8],
) -> Option<Zeroizing<[u8; KEY_LEN]>> {
    let opened = open(
        &derive(b, None, PRIVATE_INFO),
        &aad(PRIVATE_INFO, library_id),
        sealed,
    )?;
    let private: [u8; KEY_LEN] = opened.as_slice().try_into().ok()?;
    Some(Zeroizing::new(private))
}

/// The key two X25519 keys share, `None` for a low-order point (an all-zero
/// result), which no honest key gives.
fn shared(secret: &[u8; KEY_LEN], other: &[u8; KEY_LEN]) -> Option<Zeroizing<[u8; KEY_LEN]>> {
    let dh = Zeroizing::new(MontgomeryPoint(*other).mul_clamped(*secret).to_bytes());
    (!dh.iter().all(|&byte| byte == 0)).then_some(dh)
}

fn keys_key(
    dh: &[u8; KEY_LEN],
    ephemeral: &[u8; KEY_LEN],
    public: &[u8; KEY_LEN],
) -> Zeroizing<[u8; KEY_LEN]> {
    let mut salt = [0u8; 2 * KEY_LEN];
    salt[..KEY_LEN].copy_from_slice(ephemeral);
    salt[KEY_LEN..].copy_from_slice(public);
    derive(dh, Some(&salt), KEYS_INFO)
}

/// Seals `plaintext` to `public`: a new ephemeral key, returned with the
/// sealed value. `None` when `public` is a low-order point.
pub fn seal_to(
    public: &[u8; KEY_LEN],
    library_id: &str,
    plaintext: &[u8],
) -> Option<([u8; KEY_LEN], Vec<u8>)> {
    let ephemeral = KeyPair::generate();
    let dh = shared(ephemeral.secret(), public)?;
    let key = keys_key(&dh, &ephemeral.public, public);
    let sealed = seal(&key, &aad(KEYS_INFO, library_id), plaintext);
    Some((ephemeral.public, sealed))
}

/// What [`seal_to`] sealed to the public key of `private`.
pub fn open_from(
    private: &[u8; KEY_LEN],
    ephemeral: &[u8; KEY_LEN],
    library_id: &str,
    sealed: &[u8],
) -> Option<Zeroizing<Vec<u8>>> {
    let public = MontgomeryPoint::mul_base_clamped(*private).to_bytes();
    let dh = shared(private, ephemeral)?;
    let key = keys_key(&dh, ephemeral, &public);
    open(&key, &aad(KEYS_INFO, library_id), sealed)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Whatever the text and the library, what is sealed to a pair opens
        /// with its private key, and with nothing else.
        #[test]
        fn prop_sealed_to_a_key_opens_only_with_its_private_half(
            plain in prop::collection::vec(any::<u8>(), 0..300),
            library in "l_[0-9a-f]{8}",
            other in "l_[0-9a-f]{8}",
        ) {
            let pair = KeyPair::generate();
            let stranger = KeyPair::generate();
            let (ephemeral, sealed) = seal_to(&pair.public, &library, &plain).unwrap();
            let opened = open_from(pair.secret(), &ephemeral, &library, &sealed).unwrap();
            prop_assert_eq!(opened.as_slice(), plain.as_slice());
            prop_assert!(open_from(stranger.secret(), &ephemeral, &library, &sealed).is_none());
            if other != library {
                prop_assert!(open_from(pair.secret(), &ephemeral, &other, &sealed).is_none());
            }
            // A flipped bit anywhere is refused.
            let mut damaged = sealed.clone();
            let at = plain.len() % damaged.len();
            damaged[at] ^= 1;
            prop_assert!(open_from(pair.secret(), &ephemeral, &library, &damaged).is_none());
        }

        /// The private key opens with `B` and with no other secret.
        #[test]
        fn prop_the_private_key_opens_only_with_its_secret(
            b in prop::array::uniform32(any::<u8>()),
            wrong in prop::array::uniform32(any::<u8>()),
            library in "l_[0-9a-f]{8}",
        ) {
            let pair = KeyPair::generate();
            let sealed = seal_private(&b, &library, pair.secret());
            let opened = open_private(&b, &library, &sealed).unwrap();
            prop_assert_eq!(&opened[..], &pair.secret()[..]);
            if wrong != b {
                prop_assert!(open_private(&wrong, &library, &sealed).is_none());
            }
            prop_assert!(open_private(&b, "l_other", &sealed).is_none());
            prop_assert!(open_private(&b, &library, &sealed[..sealed.len() - 1]).is_none());
        }
    }

    // no-prop-test: the one input that must be refused, a point of order one.
    #[test]
    fn a_low_order_public_key_is_refused() {
        assert!(seal_to(&[0u8; KEY_LEN], "l_a", b"keys").is_none());
        let mut one = [0u8; KEY_LEN];
        one[0] = 1;
        assert!(seal_to(&one, "l_a", b"keys").is_none());
    }
}
