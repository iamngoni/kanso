//! Electron `safeStorage` on macOS, which is Chromium's OSCrypt: `v10`
//! followed by AES-128-CBC data, keyed by PBKDF2 of the secret in the
//! Keychain item "<App> Safe Storage".
//!
//! An Electron app keeps its local server credentials this way. Nothing
//! here prints or logs the secret or the plaintext.

use aes::cipher::{
    BlockDecryptMut as _, BlockEncryptMut as _, KeyIvInit as _, block_padding::Pkcs7,
};
use anyhow::{Context as _, bail};
use base64::Engine as _;
use serde_json::Value;

use crate::config::{LocalServerConfig, LocalServerCredentials, parse_credentials};

const SALT: &[u8] = b"saltysalt";
const ITERATIONS: u32 = 1003;
const IV: [u8; 16] = [b' '; 16];
const PREFIXES: [&[u8]; 2] = [b"v10", b"v11"];

type Encryptor = cbc::Encryptor<aes::Aes128>;
type Decryptor = cbc::Decryptor<aes::Aes128>;

fn derive_key(secret: &str) -> [u8; 16] {
    let mut key = [0u8; 16];
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(secret.as_bytes(), SALT, ITERATIONS, &mut key);
    key
}

/// Encrypts `plaintext` the way Electron does, with the `v10` prefix.
pub fn encrypt(plaintext: &[u8], secret: &str) -> Vec<u8> {
    let key = derive_key(secret);
    let mut out = b"v10".to_vec();
    out.extend(Encryptor::new(&key.into(), &IV.into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext));
    out
}

/// Decrypts `v10`/`v11` data produced by Electron's `safeStorage`.
pub fn decrypt(data: &[u8], secret: &str) -> anyhow::Result<Vec<u8>> {
    let Some(body) = PREFIXES.iter().find_map(|p| data.strip_prefix(*p)) else {
        bail!("not safeStorage data (no v10/v11 prefix)");
    };
    let key = derive_key(secret);
    Decryptor::new(&key.into(), &IV.into())
        .decrypt_padded_vec_mut::<Pkcs7>(body)
        .map_err(|_| anyhow::anyhow!("couldn't decrypt: wrong Keychain secret or damaged data"))
}

impl LocalServerConfig {
    /// Decrypts `Encrypted` credentials with the Keychain `secret`, turning
    /// them into `Basic` ones. Other credentials are left as they are.
    pub fn decrypt_credentials(&mut self, secret: &str) -> anyhow::Result<()> {
        let LocalServerCredentials::Encrypted { ciphertext } = &self.credentials else {
            return Ok(());
        };
        let data = base64::engine::general_purpose::STANDARD
            .decode(ciphertext.trim())
            .context("the encrypted credentials aren't base64")?;
        let plaintext = String::from_utf8(decrypt(&data, secret)?)
            .context("the decrypted credentials aren't text")?;
        let value = serde_json::from_str::<Value>(&plaintext)
            .ok()
            .filter(Value::is_object)
            .unwrap_or(Value::String(plaintext));
        match parse_credentials(Some(&value)) {
            credentials @ LocalServerCredentials::Basic { .. } => {
                self.credentials = credentials;
                Ok(())
            }
            _ => bail!("the decrypted credentials aren't in a known format"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn unhex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Oracle: vectors from OpenSSL (`openssl kdf … PBKDF2` with SHA1,
    /// salt "saltysalt", 1003 iterations, then `openssl enc -aes-128-cbc`
    /// with an IV of 16 spaces).
    #[test]
    fn matches_openssl() {
        assert_eq!(
            hex(&derive_key("peanuts")),
            "d9a09d499b4e1b7461f28e67972c6dbd"
        );
        let mut data = b"v10".to_vec();
        data.extend(unhex("3ab2ebd9ae16b7e068e47851d75484cc"));
        assert_eq!(decrypt(&data, "peanuts").unwrap(), b"bob:hunter2");
        assert_eq!(encrypt(b"bob:hunter2", "peanuts"), data);
    }

    #[test]
    fn decrypts_encrypted_credentials_into_basic_ones() {
        let data = encrypt(b"alice:s3cret:with:colons", "keychain-secret");
        let ciphertext = base64::engine::general_purpose::STANDARD.encode(&data);
        let mut config = LocalServerConfig {
            enabled: true,
            port: 19840,
            bind_address: "127.0.0.1".into(),
            credentials: LocalServerCredentials::Encrypted { ciphertext },
        };
        assert!(config.clone().decrypt_credentials("wrong").is_err());
        config.decrypt_credentials("keychain-secret").unwrap();
        assert_eq!(config.username(), Some("alice"));
        assert_eq!(config.password(), Some("s3cret:with:colons"));
    }
}
