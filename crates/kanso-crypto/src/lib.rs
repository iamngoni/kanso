//! Client-side end-to-end encryption primitives for Kanso.
//!
//! XChaCha20-Poly1305 AEAD with an Argon2id passphrase KDF. The key is derived
//! on-device from the user's passphrase and never leaves it; the server only
//! ever sees ciphertext. Local storage stays plaintext (so FTS keeps working) —
//! encryption is applied at the sync boundary.
//!
//! Wire format of [`encrypt`] output: `nonce (24 bytes) || ciphertext+tag`.

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use thiserror::Error;
use zeroize::Zeroizing;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed (wrong key or tampered ciphertext)")]
    Decrypt,
    #[error("ciphertext too short")]
    Malformed,
}

/// Derive a 32-byte key from a passphrase and salt using Argon2id.
///
/// The returned key zeroizes itself on drop. `salt` must be at least 8 bytes and
/// should be stable per user (store it alongside the account, not the key).
pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2::Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    Ok(key)
}

/// Encrypt `plaintext`, returning `nonce || ciphertext+tag`. A fresh random
/// nonce is generated per call.
pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|_| CryptoError::Encrypt)?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt data produced by [`encrypt`]. Fails on a wrong key or any tampering
/// (the Poly1305 tag is verified).
pub fn decrypt(key: &[u8; KEY_LEN], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < NONCE_LEN {
        return Err(CryptoError::Malformed);
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT: &[u8] = b"kanso-account-salt";

    #[test]
    fn key_derivation_is_deterministic() {
        let k1 = derive_key("correct horse battery staple", SALT).unwrap();
        let k2 = derive_key("correct horse battery staple", SALT).unwrap();
        assert_eq!(*k1, *k2);
        let k3 = derive_key("a different passphrase", SALT).unwrap();
        assert_ne!(*k1, *k3);
    }

    #[test]
    fn encrypt_decrypt_roundtrips() {
        let key = derive_key("pw", SALT).unwrap();
        let plaintext = b"# Secret note\n\nThe server never sees this.";
        let blob = encrypt(&key, plaintext).unwrap();
        assert_ne!(&blob[NONCE_LEN..], &plaintext[..]); // actually encrypted
        assert_eq!(decrypt(&key, &blob).unwrap(), plaintext);
    }

    #[test]
    fn tampering_is_detected() {
        let key = derive_key("pw", SALT).unwrap();
        let mut blob = encrypt(&key, b"important data").unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0x01;
        assert!(decrypt(&key, &blob).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let k1 = derive_key("pw-one", SALT).unwrap();
        let k2 = derive_key("pw-two", SALT).unwrap();
        let blob = encrypt(&k1, b"data").unwrap();
        assert!(decrypt(&k2, &blob).is_err());
    }

    #[test]
    fn short_input_is_malformed() {
        let key = derive_key("pw", SALT).unwrap();
        assert!(matches!(decrypt(&key, b"short"), Err(CryptoError::Malformed)));
    }
}
