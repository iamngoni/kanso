//! Key derivation, wrapping and sealing (docs/ACCOUNTS.md › Constructions).
//! Pure: no I/O, no clock; randomness only from `getrandom`.
//!
//! Argon2id, HKDF-SHA256, HMAC-SHA256 and XChaCha20-Poly1305, all from
//! RustCrypto; there are no custom primitives. Every value is pinned by
//! `tests/fixtures/crypto-v1.json`, which the tests below check and which was
//! cross-checked against Node's own `crypto` (Argon2, HKDF, HMAC and
//! ChaCha20-Poly1305 with HChaCha20 for the extended nonce).

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hmac::{Hmac, Mac};
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};
use unicode_normalization::UnicodeNormalization as _;
use zeroize::Zeroizing;

use crate::wire::{WireDoc, is_wire_id, is_wire_rev};
use kanso_store::SyncDoc;

// ---------------------------------------------------------------------------
// Encodings and primitives
// ---------------------------------------------------------------------------

/// The format byte of every wrap and sealed value (format version 1).
pub const FORMAT_V1: u8 = 0x01;
pub(crate) const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
/// What sealing adds to a plaintext: the format byte, the nonce and the tag.
pub const SEAL_OVERHEAD: usize = 1 + NONCE_LEN + TAG_LEN;

/// `N` bytes from the system's random source (`getrandom`).
pub(crate) fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    bytes
}

/// Unpadded base64url (RFC 4648 §5).
pub(crate) fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Strict unpadded base64url: no padding, no other alphabet, and canonical
/// (re-encoding gives the same text).
pub(crate) fn unb64(text: &str) -> Option<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD.decode(text).ok()?;
    (b64(&bytes) == text).then_some(bytes)
}

/// Lowercase hex.
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `hex(SHA-256(bytes))`: how the server stores auth keys, tokens and proofs.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// Appends `lp(x)`: a 2-byte big-endian byte length, then the UTF-8 bytes.
/// Every caller passes a value checked against its wire pattern (or this
/// client's own), all far below 65,535 bytes; a longer one is clamped, which
/// only makes its AAD differ from any real one.
pub(crate) fn lp(out: &mut Vec<u8>, x: &str) {
    let len = u16::try_from(x.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(x.as_bytes());
}

/// HKDF-SHA256 with the default all-zero salt unless one is given.
fn hkdf(ikm: &[u8], salt: Option<&[u8]>, info: &[u8], out: &mut [u8]) {
    hkdf::Hkdf::<Sha256>::new(salt, ikm)
        .expand(info, out)
        .expect("HKDF-SHA256 gives up to 8,160 bytes");
}

pub(crate) fn hkdf32(ikm: &[u8], salt: Option<&[u8]>, info: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    hkdf(ikm, salt, info, &mut out[..]);
    out
}

/// HMAC-SHA256 over the concatenation of `parts`.
pub(crate) fn hmac(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC takes keys of any length");
    for part in parts {
        mac.update(part);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&mac.finalize().into_bytes());
    out
}

/// `0x01 || nonce || XChaCha20-Poly1305(key, nonce, aad, plaintext)`.
pub(crate) fn seal_raw(
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    aad: &[u8],
    plaintext: &[u8],
) -> Vec<u8> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("a 32-byte key");
    let body = cipher
        .encrypt(
            &XNonce::from(*nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("XChaCha20-Poly1305 seals every length used here");
    let mut out = Vec::with_capacity(1 + NONCE_LEN + body.len());
    out.push(FORMAT_V1);
    out.extend_from_slice(nonce);
    out.extend_from_slice(&body);
    out
}

/// Opens [`seal_raw`]'s output: a format byte above 1 is too new; anything
/// else that doesn't open under this key and AAD is undecryptable.
pub(crate) fn open_raw(key: &[u8; 32], aad: &[u8], sealed: &[u8]) -> Opened<Zeroizing<Vec<u8>>> {
    match sealed.first() {
        Some(&format) if format > FORMAT_V1 => Opened::TooNew,
        Some(&FORMAT_V1) if sealed.len() >= SEAL_OVERHEAD => {
            let mut nonce = [0u8; NONCE_LEN];
            nonce.copy_from_slice(&sealed[1..=NONCE_LEN]);
            let cipher = XChaCha20Poly1305::new_from_slice(key).expect("a 32-byte key");
            match cipher.decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: &sealed[1 + NONCE_LEN..],
                    aad,
                },
            ) {
                Ok(plaintext) => Opened::Opened(Zeroizing::new(plaintext)),
                Err(_) => Opened::Undecryptable,
            }
        }
        _ => Opened::Undecryptable,
    }
}

// ---------------------------------------------------------------------------
// KDF parameters
// ---------------------------------------------------------------------------

/// Argon2id parameters (version 0x13, 32-byte output). The client takes its
/// floor and default from its backend ([`crate::Backend::kdf_floor`],
/// [`crate::Backend::kdf_default`]); there is no cargo feature that lowers
/// the production floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// The production floor (`HttpBackend`, and the Worker regardless).
    pub const FLOOR: KdfParams = KdfParams {
        m_kib: 1_048_576,
        t: 3,
        p: 1,
    };
    /// What new accounts use in production: 1 GiB, t = 3, p = 4.
    pub const DEFAULT: KdfParams = KdfParams {
        m_kib: 1_048_576,
        t: 3,
        p: 4,
    };
    /// `MemoryServer`'s floor and default, so tests derive in milliseconds.
    pub const TEST: KdfParams = KdfParams {
        m_kib: 64,
        t: 1,
        p: 1,
    };
    /// The most any server may demand: about 2 s and 2 GiB on an M4 Pro.
    pub const CEILING: KdfParams = KdfParams {
        m_kib: 2_097_152,
        t: 8,
        p: 8,
    };

    /// Whether every parameter lies between `floor` and [`KdfParams::CEILING`]
    /// (inclusive). A prelogin answer outside is refused as `UnsafeKdf`
    /// before any key is derived or sent.
    pub fn is_within(&self, floor: &KdfParams) -> bool {
        (floor.m_kib..=Self::CEILING.m_kib).contains(&self.m_kib)
            && (floor.t..=Self::CEILING.t).contains(&self.t)
            && (floor.p.max(1)..=Self::CEILING.p).contains(&self.p)
    }
}

// ---------------------------------------------------------------------------
// Password
// ---------------------------------------------------------------------------

/// Shortest and longest password, in Unicode scalar values after NFC.
pub const MIN_PASSWORD_CHARS: usize = 12;
pub const MAX_PASSWORD_CHARS: usize = 256;

/// Why a password was refused. Only the client checks these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PasswordError {
    #[error("use at least {MIN_PASSWORD_CHARS} characters")]
    TooShort,
    #[error("use at most {MAX_PASSWORD_CHARS} characters")]
    TooLong,
    #[error("the password is the username")]
    SameAsUsername,
}

/// A password, normalized to Unicode NFC. Zeroized on drop; never printed.
pub struct Password(Zeroizing<String>);

impl Password {
    /// Normalizes `text` to NFC and checks it: 12–256 Unicode scalar values,
    /// and not equal to the username ignoring case (`username` is compared
    /// in its normalized form when it is a valid username, else as given,
    /// both lowercased). Pure (docs/ACCOUNTS.md › Password and KDF).
    pub fn new(text: &str, username: &str) -> Result<Password, PasswordError> {
        let normalized = Zeroizing::new(text.nfc().collect::<String>());
        let chars = normalized.chars().count();
        if chars < MIN_PASSWORD_CHARS {
            return Err(PasswordError::TooShort);
        }
        if chars > MAX_PASSWORD_CHARS {
            return Err(PasswordError::TooLong);
        }
        let name = crate::username::normalize_username(username)
            .unwrap_or_else(|_| username.nfc().collect());
        if same_ignoring_case(&normalized, &name) {
            return Err(PasswordError::SameAsUsername);
        }
        Ok(Password(normalized))
    }

    /// The NFC text, as UTF-8 bytes for Argon2.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Whether the password is `username`, ignoring case, compared without
    /// a lowercased copy of the password.
    pub(crate) fn is_username(&self, username: &str) -> bool {
        same_ignoring_case(&self.0, username)
    }
}

/// `a` and `b` lowercased, character by character, are equal; nothing is
/// copied.
fn same_ignoring_case(a: &str, b: &str) -> bool {
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// Recovery key
// ---------------------------------------------------------------------------

/// Crockford base32, as the recovery key is shown.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// The checksum modulus: prime and below 32².
const CHECK_MODULUS: u128 = 1021;

/// Why recovery-key text was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RecoveryKeyError {
    /// A character outside Crockford base32 (after dropping spaces and
    /// hyphens, uppercasing, and reading O as 0 and I/L as 1). `U` is refused.
    #[error("a recovery key only has the letters and digits shown")]
    BadCharacter,
    /// Not 28 characters.
    #[error("a recovery key has 28 letters and digits")]
    WrongLength,
    /// The first character above 7, a check value of 1021 or more, or a
    /// checksum that doesn't match: a typo.
    #[error("the recovery key has a typo")]
    Checksum,
}

/// A recovery key: 16 random bytes, shown as 28 Crockford base32 characters
/// in 7 groups of 4 (26 data characters, then `V mod 1021` as 2 characters).
/// Zeroized on drop; never printed by `Debug`. See docs/ACCOUNTS.md ›
/// Recovery key.
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryKey(Zeroizing<[u8; 16]>);

impl RecoveryKey {
    /// A new random key.
    pub fn generate() -> RecoveryKey {
        RecoveryKey::from_bytes(random())
    }

    /// The key holding exactly these bytes (golden vectors and tests).
    pub fn from_bytes(bytes: [u8; 16]) -> RecoveryKey {
        RecoveryKey(Zeroizing::new(bytes))
    }

    pub(crate) fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Lenient parsing: drops spaces and hyphens, uppercases, reads `O` as
    /// `0` and `I`/`L` as `1`, refuses `U` and every other character, then
    /// requires exactly 28 characters, a first character below `8`, a check
    /// value below 1021 and a matching checksum. Pure.
    pub fn parse(text: &str) -> Result<RecoveryKey, RecoveryKeyError> {
        // The key's digits and value are zeroized once read.
        let mut digits: Zeroizing<Vec<u128>> = Zeroizing::new(Vec::with_capacity(28));
        for c in text.chars() {
            let c = match c.to_ascii_uppercase() {
                ' ' | '-' => continue,
                'O' => '0',
                'I' | 'L' => '1',
                c => c,
            };
            let digit = ALPHABET
                .iter()
                .position(|&a| c.is_ascii() && a == c as u8)
                .ok_or(RecoveryKeyError::BadCharacter)?;
            digits.push(digit as u128);
        }
        if digits.len() != 28 {
            return Err(RecoveryKeyError::WrongLength);
        }
        if digits[0] >= 8 {
            return Err(RecoveryKeyError::Checksum);
        }
        let value = Zeroizing::new(digits[..26].iter().fold(0u128, |v, d| (v << 5) | d));
        let check = digits[26] * 32 + digits[27];
        if check >= CHECK_MODULUS || check != *value % CHECK_MODULUS {
            return Err(RecoveryKeyError::Checksum);
        }
        Ok(RecoveryKey::from_bytes(value.to_be_bytes()))
    }

    /// The key as shown: `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`. Pure.
    pub fn display(&self) -> String {
        let value = Zeroizing::new(u128::from_be_bytes(*self.0));
        let check = *value % CHECK_MODULUS;
        let mut digits: Zeroizing<Vec<u8>> = Zeroizing::new(
            (0..26)
                .map(|i| ALPHABET[((*value >> (5 * (25 - i))) & 31) as usize])
                .collect(),
        );
        digits.push(ALPHABET[(check / 32) as usize]);
        digits.push(ALPHABET[(check % 32) as usize]);
        digits
            .chunks(4)
            .map(|group| std::str::from_utf8(group).expect("ASCII"))
            .collect::<Vec<_>>()
            .join("-")
    }
}

impl fmt::Debug for RecoveryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryKey(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

macro_rules! secret32 {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        /// 32 bytes, zeroized on drop, redacted in `Debug`.
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name(Zeroizing<[u8; 32]>);

        impl $name {
            /// The key holding exactly these bytes (golden vectors, Keychain).
            pub fn from_bytes(bytes: [u8; 32]) -> $name {
                $name(Zeroizing::new(bytes))
            }

            #[allow(dead_code, reason = "not every key type reads its bytes back")]
            pub(crate) fn bytes(&self) -> &[u8; 32] {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

secret32!(
    /// The auth key, HKDF(master, "kanso/v1/auth"), or the recovery auth key,
    /// HKDF(recovery secret, salt, "kanso/v1/recovery-auth"). Sent as 43
    /// base64url characters; the server keeps only its SHA-256.
    AuthKey
);
secret32!(
    /// A key-encryption key: HKDF(master, "kanso/v1/kek") or the recovery
    /// KEK. Never leaves the Mac.
    Kek
);
secret32!(
    /// The random data key: root of every content key. Kept in the Keychain.
    DataKey
);
secret32!(
    /// The device proof, HKDF(device secret, info "kanso/v1/device-proof" ||
    /// lp(origin) || lp(account_id)). Sent at sign-up, sign-in and recovery;
    /// the server keeps its SHA-256 per session, and only a matching proof
    /// exempts a sign-in from `login-user` (security re-check 5).
    DeviceProof
);

impl DataKey {
    /// A new random data key.
    pub fn generate() -> DataKey {
        DataKey::from_bytes(random())
    }
}

impl AuthKey {
    /// 43 unpadded base64url characters, as the wire carries it.
    pub fn to_base64url(&self) -> String {
        b64(&self.0[..])
    }
}

impl DeviceProof {
    /// 43 unpadded base64url characters, as the wire carries it.
    pub fn to_base64url(&self) -> String {
        b64(&self.0[..])
    }
}

/// The two keys a password (or a recovery key) gives.
#[derive(Debug)]
pub struct DerivedKeys {
    /// Sent to the server.
    pub auth_key: AuthKey,
    /// Wraps the data key; never sent.
    pub kek: Kek,
}

/// Which key wrapped the data key; the purpose byte of the wrap's AAD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapPurpose {
    /// `0x50` (`P`): under the password KEK.
    Password,
    /// `0x52` (`R`): under the recovery KEK.
    Recovery,
    /// `0x4B` (`K`): under a passkey's or security key's PRF-derived KEK.
    Passkey,
}

impl WrapPurpose {
    fn byte(self) -> u8 {
        match self {
            WrapPurpose::Password => b'P',
            WrapPurpose::Recovery => b'R',
            WrapPurpose::Passkey => b'K',
        }
    }
}

/// The fixed WebAuthn PRF input every Kanso passkey is asked for:
/// `SHA-256("kanso/v1/passkey-prf" || 0x01)`, 32 bytes (docs/ACCOUNTS.md ›
/// Keys for passkeys and security keys). It is a protocol constant: changing
/// it makes every passkey wrap unopenable, so a golden vector pins it.
pub const PRF_INPUT: [u8; 32] = [
    0x98, 0x43, 0x20, 0xbf, 0xab, 0xbd, 0x34, 0x26, 0x97, 0xaf, 0x83, 0x64, 0x7d, 0xc7, 0x74, 0xe3,
    0x07, 0xf9, 0x8e, 0x62, 0xec, 0xf2, 0x7d, 0xd5, 0xd8, 0x4f, 0x8f, 0x00, 0x6d, 0x86, 0xfd, 0x59,
];

/// `passkey_kek = HKDF(ikm = prf_out,
///                     salt = "kanso/v1/passkey" || 0x01 || lp(origin),
///                     info = "kanso/v1/passkey-kek" || 0x01 || lp(account_id)
///                            || lp(credential_id))`.
/// `credential_id` is the raw credential ID in unpadded base64url, so a wrap
/// moved to another credential derives another key and fails to open.
pub fn passkey_kek(prf_out: &[u8; 32], origin: &str, account_id: &str, credential_id: &str) -> Kek {
    let mut salt = b"kanso/v1/passkey".to_vec();
    salt.push(FORMAT_V1);
    lp(&mut salt, origin);
    let mut info = b"kanso/v1/passkey-kek".to_vec();
    info.push(FORMAT_V1);
    lp(&mut info, account_id);
    lp(&mut info, credential_id);
    Kek(hkdf32(prf_out, Some(&salt), &info))
}

/// Why opening keys failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    /// A wrap didn't open under this KEK and account, or the data key's key
    /// ID differs from the server's.
    #[error("the keys don't match")]
    KeyMismatch,
    /// Argon2 refused the parameters or ran out of memory.
    #[error("key derivation failed: {0}")]
    Kdf(String),
}

/// `"kanso/v1/salt" || lp(origin) || lp(username) || salt16`.
fn salt_input(origin: &str, username: &str, salt: &[u8; 16]) -> Vec<u8> {
    let mut input = b"kanso/v1/salt".to_vec();
    lp(&mut input, origin);
    lp(&mut input, username);
    input.extend_from_slice(salt);
    input
}

/// `Argon2id(password, salt_input)` then HKDF into the auth key and the KEK,
/// zeroizing the master key. `salt_input = "kanso/v1/salt" || lp(origin) ||
/// lp(username) || salt16`; `username` is normalized (docs/ACCOUNTS.md ›
/// Password and KDF, › Keys). The caller checks `params` against its
/// backend's floor and the ceilings first.
pub fn password_keys(
    password: &Password,
    origin: &str,
    username: &str,
    salt: &[u8; 16],
    params: KdfParams,
) -> Result<DerivedKeys, CryptoError> {
    let kdf = |e: argon2::Error| CryptoError::Kdf(e.to_string());
    let argon = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(params.m_kib, params.t, params.p, Some(32)).map_err(kdf)?,
    );
    let mut master = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(
            password.as_bytes(),
            &salt_input(origin, username, salt),
            &mut master[..],
        )
        .map_err(kdf)?;
    Ok(DerivedKeys {
        auth_key: AuthKey(hkdf32(&master[..], None, b"kanso/v1/auth")),
        kek: Kek(hkdf32(&master[..], None, b"kanso/v1/kek")),
    })
}

/// The recovery auth key and recovery KEK: HKDF(ikm = recovery secret, salt
/// = "kanso/v1/recovery" || lp(origin) || lp(username), info
/// "kanso/v1/recovery-auth" / "kanso/v1/recovery-kek").
pub fn recovery_keys(key: &RecoveryKey, origin: &str, username: &str) -> DerivedKeys {
    let mut salt = b"kanso/v1/recovery".to_vec();
    lp(&mut salt, origin);
    lp(&mut salt, username);
    DerivedKeys {
        auth_key: AuthKey(hkdf32(
            &key.bytes()[..],
            Some(&salt),
            b"kanso/v1/recovery-auth",
        )),
        kek: Kek(hkdf32(
            &key.bytes()[..],
            Some(&salt),
            b"kanso/v1/recovery-kek",
        )),
    }
}

/// `"kanso-wrap" || 0x01 || purpose || lp(account_id)`.
fn wrap_aad(purpose: WrapPurpose, account_id: &str) -> Vec<u8> {
    let mut aad = b"kanso-wrap".to_vec();
    aad.push(FORMAT_V1);
    aad.push(purpose.byte());
    lp(&mut aad, account_id);
    aad
}

/// `0x01 || nonce(24) || XChaCha20-Poly1305(kek, nonce, aad, data_key)` as 98
/// base64url characters, where `aad = "kanso-wrap" || 0x01 || purpose ||
/// lp(account_id)` (the account ID binding is security re-check 3).
pub fn wrap(kek: &Kek, purpose: WrapPurpose, account_id: &str, data_key: &DataKey) -> String {
    wrap_with_nonce(kek, purpose, account_id, data_key, &random())
}

pub(crate) fn wrap_with_nonce(
    kek: &Kek,
    purpose: WrapPurpose,
    account_id: &str,
    data_key: &DataKey,
    nonce: &[u8; NONCE_LEN],
) -> String {
    b64(&seal_raw(
        &kek.0,
        nonce,
        &wrap_aad(purpose, account_id),
        &data_key.0[..],
    ))
}

/// Opens a wrap made by [`wrap`] with the same KEK, purpose and account ID;
/// anything else is [`CryptoError::KeyMismatch`].
pub fn unwrap(
    kek: &Kek,
    purpose: WrapPurpose,
    account_id: &str,
    wrapped: &str,
) -> Result<DataKey, CryptoError> {
    let sealed = unb64(wrapped).ok_or(CryptoError::KeyMismatch)?;
    if sealed.len() != SEAL_OVERHEAD + 32 {
        return Err(CryptoError::KeyMismatch);
    }
    match open_raw(&kek.0, &wrap_aad(purpose, account_id), &sealed) {
        Opened::Opened(plain) if plain.len() == 32 => {
            let mut key = Zeroizing::new([0u8; 32]);
            key.copy_from_slice(&plain);
            Ok(DataKey::from_bytes(*key))
        }
        _ => Err(CryptoError::KeyMismatch),
    }
}

/// HKDF(data_key, "kanso/v1/key-id", 8 bytes) as 16 hex digits.
pub fn key_id(data_key: &DataKey) -> String {
    let mut id = [0u8; 8];
    hkdf(&data_key.0[..], None, b"kanso/v1/key-id", &mut id);
    hex(&id)
}

/// This Mac's device ID in an account: `d_ + hex(HMAC-SHA256(device_secret,
/// account_id)[0..16])` (L2).
pub(crate) fn device_id(device_secret: &[u8; 32], account_id: &str) -> String {
    format!(
        "d_{}",
        hex(&hmac(device_secret, &[account_id.as_bytes()])[..16])
    )
}

/// HKDF(ikm device_secret, info "kanso/v1/device-proof" || lp(origin) ||
/// lp(account_id)), 32 bytes (security re-check 5).
pub(crate) fn device_proof(
    device_secret: &[u8; 32],
    origin: &str,
    account_id: &str,
) -> DeviceProof {
    let mut info = b"kanso/v1/device-proof".to_vec();
    lp(&mut info, origin);
    lp(&mut info, account_id);
    DeviceProof(hkdf32(device_secret, None, &info))
}

/// What opening a sealed value gave (docs/ACCOUNTS.md › Sealed documents).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opened<T> {
    Opened(T),
    /// Format byte `0x01`, but the AEAD, the JSON shape, the wire ID or
    /// revision, the kind, or a tombstone's empty fields didn't check out.
    Undecryptable,
    /// A format byte above `0x01`: "Update Kanso". Never overwrites anything.
    TooNew,
}

impl<T> Opened<T> {
    fn map<U>(self, f: impl FnOnce(T) -> Option<U>) -> Opened<U> {
        match self {
            Opened::Opened(value) => f(value).map_or(Opened::Undecryptable, Opened::Opened),
            Opened::Undecryptable => Opened::Undecryptable,
            Opened::TooNew => Opened::TooNew,
        }
    }
}

/// The content keys, derived from the data key with HKDF. Zeroized on drop.
pub struct ContentKeys {
    doc_key: Zeroizing<[u8; 32]>,
    file_key: Zeroizing<[u8; 32]>,
    id_key: Zeroizing<[u8; 32]>,
    rev_key: Zeroizing<[u8; 32]>,
    device_name_key: Zeroizing<[u8; 32]>,
}

impl fmt::Debug for ContentKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ContentKeys(<redacted>)")
    }
}

/// Writes `value` as canonical JSON: no spaces, object keys sorted by their
/// UTF-8 bytes, strings and numbers as `serde_json` writes them. A sealed
/// document's plaintext is `{"id":…,"rev":…,"fields":…}` in that order, so
/// its bytes don't depend on whether `serde_json` keeps insertion order (a
/// feature some workspace crates turn on) and the golden vectors hold for
/// every build.
fn canonical_json(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => canonical_object(map, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical_json(item, out);
            }
            out.push(']');
        }
        other => out.push_str(&serde_json::to_string(other).expect("JSON values always serialize")),
    }
}

fn canonical_object(map: &Map<String, Value>, out: &mut String) {
    let mut entries: Vec<(&String, &Value)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    out.push('{');
    for (i, (key, value)) in entries.into_iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&serde_json::to_string(key).expect("strings always serialize"));
        out.push(':');
        canonical_json(value, out);
    }
    out.push('}');
}

/// A sealed document's plaintext: `{"id", "rev", "fields"}` as canonical
/// JSON; a tombstone's fields are `{}`.
fn doc_plaintext(doc: &SyncDoc) -> Zeroizing<Vec<u8>> {
    let empty = Map::new();
    let fields = if doc.deleted { &empty } else { &doc.fields };
    let mut text = Zeroizing::new(String::from("{\"id\":"));
    canonical_json(&Value::String(doc.id.clone()), &mut text);
    text.push_str(",\"rev\":");
    canonical_json(&Value::String(doc.rev.clone()), &mut text);
    text.push_str(",\"fields\":");
    canonical_object(fields, &mut text);
    text.push('}');
    Zeroizing::new(std::mem::take(&mut *text).into_bytes())
}

/// The length of `doc`'s sealed text (base64url without padding of the
/// format byte, nonce, ciphertext and tag), what the server charges.
pub(crate) fn sealed_doc_len(doc: &SyncDoc) -> usize {
    let raw = SEAL_OVERHEAD + doc_plaintext(doc).len();
    (4 * raw).div_ceil(3)
}

/// The kind prefix of a local or wire ID (`note`, `book`, `tag`, `file`).
fn kind_of(id: &str) -> &str {
    id.split_once(':').map_or("", |(kind, _)| kind)
}

/// `"kanso-doc" || 0x01 || lp(account_id) || lp(wire_id) || lp(wire_rev) || deleted`.
fn doc_aad(account_id: &str, wire_id: &str, wire_rev: &str, deleted: bool) -> Vec<u8> {
    let mut aad = b"kanso-doc".to_vec();
    aad.push(FORMAT_V1);
    lp(&mut aad, account_id);
    lp(&mut aad, wire_id);
    lp(&mut aad, wire_rev);
    aad.push(u8::from(deleted));
    aad
}

/// `"kanso-file" || 0x01 || lp(account_id) || lp(wire_id)`.
fn file_aad(account_id: &str, wire_id: &str) -> Vec<u8> {
    let mut aad = b"kanso-file".to_vec();
    aad.push(FORMAT_V1);
    lp(&mut aad, account_id);
    lp(&mut aad, wire_id);
    aad
}

/// `"kanso-device" || 0x01 || lp(account_id) || lp(device_id)`.
fn name_aad(account_id: &str, device_id: &str) -> Vec<u8> {
    let mut aad = b"kanso-device".to_vec();
    aad.push(FORMAT_V1);
    lp(&mut aad, account_id);
    lp(&mut aad, device_id);
    aad
}

/// The keys shared notes start from (docs/ACCOUNTS.md › Collaboration),
/// derived from the data key: `identity_key` seals the account's identity
/// keys for its devices, `vault_key` seals note keys at rest in the library.
/// Zeroized on drop.
pub struct CollabRoots {
    pub identity_key: Zeroizing<[u8; 32]>,
    pub vault_key: Zeroizing<[u8; 32]>,
}

impl fmt::Debug for CollabRoots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CollabRoots(<redacted>)")
    }
}

impl CollabRoots {
    /// HKDF(`data_key`, info `"kanso/v1/identity"`) and HKDF(`data_key`,
    /// info `"kanso/v1/collab-vault"`).
    pub fn derive(data_key: &DataKey) -> CollabRoots {
        CollabRoots {
            identity_key: hkdf32(&data_key.0[..], None, b"kanso/v1/identity"),
            vault_key: hkdf32(&data_key.0[..], None, b"kanso/v1/collab-vault"),
        }
    }
}

impl ContentKeys {
    /// Derives `doc_key`, `file_key`, `id_key`, `rev_key` and
    /// `device_name_key` from the data key.
    pub fn derive(data_key: &DataKey) -> ContentKeys {
        let key = |info: &[u8]| hkdf32(&data_key.0[..], None, info);
        ContentKeys {
            doc_key: key(b"kanso/v1/doc"),
            file_key: key(b"kanso/v1/file"),
            id_key: key(b"kanso/v1/wire-id"),
            rev_key: key(b"kanso/v1/wire-rev"),
            device_name_key: key(b"kanso/v1/device-name"),
        }
    }

    /// `kind + ":" + base64url(HMAC-SHA256(id_key, local_id)[0..16])`.
    pub fn wire_id(&self, local_id: &str) -> String {
        let mac = hmac(&self.id_key[..], &[local_id.as_bytes()]);
        format!("{}:{}", kind_of(local_id), b64(&mac[..16]))
    }

    /// `hex(HMAC-SHA256(rev_key, lp(local_id) || local_rev)[0..16])`.
    pub fn wire_rev(&self, local_id: &str, local_rev: &str) -> String {
        let mut id = Vec::new();
        lp(&mut id, local_id);
        let mac = hmac(&self.rev_key[..], &[&id, local_rev.as_bytes()]);
        hex(&mac[..16])
    }

    /// Seals a local document for `account_id`: its wire ID and revision, its
    /// deleted flag, and the sealed text with AAD `"kanso-doc" || 0x01 ||
    /// lp(account_id) || lp(wire_id) || lp(wire_rev) || deleted`. The
    /// plaintext is `{"id", "rev", "fields"}` as canonical JSON (see
    /// `canonical_json`); a tombstone's fields are `{}`.
    pub fn seal_doc(&self, account_id: &str, doc: &SyncDoc) -> WireDoc {
        self.seal_doc_with_nonce(account_id, doc, &random())
    }

    pub(crate) fn seal_doc_with_nonce(
        &self,
        account_id: &str,
        doc: &SyncDoc,
        nonce: &[u8; NONCE_LEN],
    ) -> WireDoc {
        let id = self.wire_id(&doc.id);
        let rev = self.wire_rev(&doc.id, &doc.rev);
        let plaintext = doc_plaintext(doc);
        let aad = doc_aad(account_id, &id, &rev, doc.deleted);
        let sealed = b64(&seal_raw(&self.doc_key, nonce, &aad, &plaintext));
        WireDoc {
            id,
            rev,
            deleted: doc.deleted,
            sealed,
        }
    }

    /// Opens a stored document, checking everything [`Opened::Undecryptable`]
    /// lists: the AEAD under this account, wire ID, revision and deleted
    /// flag; the JSON shape (unknown top-level keys are ignored); that the
    /// local ID and revision give the wire ID and revision back; that the
    /// kinds agree; and that a tombstone has no fields. A format byte above
    /// 1 is [`Opened::TooNew`].
    pub fn open_doc(&self, account_id: &str, wire: &WireDoc) -> Opened<SyncDoc> {
        let Some(sealed) = unb64(&wire.sealed) else {
            return Opened::Undecryptable;
        };
        if sealed.first().is_some_and(|&format| format > FORMAT_V1) {
            return Opened::TooNew;
        }
        if !is_wire_id(&wire.id) || !is_wire_rev(&wire.rev) {
            return Opened::Undecryptable;
        }
        let aad = doc_aad(account_id, &wire.id, &wire.rev, wire.deleted);
        open_raw(&self.doc_key, &aad, &sealed).map(|plaintext| {
            let Ok(Value::Object(mut plain)) = serde_json::from_slice::<Value>(&plaintext) else {
                return None;
            };
            let (Some(Value::String(id)), Some(Value::String(rev)), Some(Value::Object(fields))) = (
                plain.remove("id"),
                plain.remove("rev"),
                plain.remove("fields"),
            ) else {
                return None;
            };
            let kind_ok =
                kanso_core::DocType::of_id(&id).is_some_and(|t| t.as_str() == kind_of(&wire.id));
            let ok = kind_ok
                && self.wire_id(&id) == wire.id
                && self.wire_rev(&id, &rev) == wire.rev
                && (!wire.deleted || fields.is_empty());
            ok.then_some(SyncDoc {
                id,
                rev,
                deleted: wire.deleted,
                fields,
                base_rev: None,
            })
        })
    }

    /// `0x01 || nonce(24) || XChaCha20-Poly1305(file_key, nonce, "kanso-file"
    /// || 0x01 || lp(account_id) || lp(wire_id), bytes)`.
    pub fn seal_file(&self, account_id: &str, wire_id: &str, bytes: &[u8]) -> Vec<u8> {
        self.seal_file_with_nonce(account_id, wire_id, bytes, &random())
    }

    pub(crate) fn seal_file_with_nonce(
        &self,
        account_id: &str,
        wire_id: &str,
        bytes: &[u8],
        nonce: &[u8; NONCE_LEN],
    ) -> Vec<u8> {
        seal_raw(&self.file_key, nonce, &file_aad(account_id, wire_id), bytes)
    }

    pub fn open_file(&self, account_id: &str, wire_id: &str, sealed: &[u8]) -> Opened<Vec<u8>> {
        open_raw(&self.file_key, &file_aad(account_id, wire_id), sealed)
            .map(|plain| Some(plain.to_vec()))
    }

    /// Seals a device name with AAD `"kanso-device" || 0x01 ||
    /// lp(account_id) || lp(device_id)`: 55–400 base64url characters. The
    /// name passes [`crate::device_name`] first, so it is at most 64 scalar
    /// values.
    pub fn seal_name(&self, account_id: &str, device_id: &str, name: &str) -> String {
        self.seal_name_with_nonce(account_id, device_id, name, &random())
    }

    pub(crate) fn seal_name_with_nonce(
        &self,
        account_id: &str,
        device_id: &str,
        name: &str,
        nonce: &[u8; NONCE_LEN],
    ) -> String {
        let name = crate::binding::device_name(name);
        b64(&seal_raw(
            &self.device_name_key,
            nonce,
            &name_aad(account_id, device_id),
            name.as_bytes(),
        ))
    }

    pub fn open_name(&self, account_id: &str, device_id: &str, sealed: &str) -> Opened<String> {
        let Some(sealed) = unb64(sealed) else {
            return Opened::Undecryptable;
        };
        open_raw(
            &self.device_name_key,
            &name_aad(account_id, device_id),
            &sealed,
        )
        .map(|plain| String::from_utf8(plain.to_vec()).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn password_examples() {
        assert!(Password::new("correct horse battery", "alice").is_ok());
        assert_eq!(
            Password::new("short", "alice").unwrap_err(),
            PasswordError::TooShort
        );
        assert_eq!(
            Password::new(&"x".repeat(257), "alice").unwrap_err(),
            PasswordError::TooLong
        );
        assert!(Password::new(&"x".repeat(256), "alice").is_ok());
        // Twelve characters, counted as scalar values after NFC: "é" composed
        // from e + U+0301 counts once.
        assert_eq!(
            Password::new("abcdefghije\u{301}", "alice").unwrap_err(),
            PasswordError::TooShort
        );
        assert!(Password::new("abcdefghijke\u{301}", "alice").is_ok());
        assert_eq!(
            Password::new("Correct-Horse", "correct-horse").unwrap_err(),
            PasswordError::SameAsUsername
        );
        assert_eq!(
            Password::new("CORRECT-HORSE", " Correct-Horse\t").unwrap_err(),
            PasswordError::SameAsUsername
        );
        assert!(
            format!("{:?}", Password::new("correct horse battery", "a").unwrap())
                .contains("redacted")
        );
    }

    #[test]
    fn recovery_key_examples() {
        let key = RecoveryKey::from_bytes([0; 16]);
        assert_eq!(key.display(), "0000-0000-0000-0000-0000-0000-0000");
        let max = RecoveryKey::from_bytes([0xff; 16]);
        let shown = max.display();
        assert!(shown.starts_with('7'), "{shown}");
        assert_eq!(RecoveryKey::parse(&shown).unwrap(), max);
        // Lenient: lowercase, O for 0, I and L for 1, no hyphens, spaces.
        let key = RecoveryKey::from_bytes(
            *b"\x01\x23\x45\x67\x89\xab\xcd\xef\x10\x32\x54\x76\x98\xba\xdc\xfe",
        );
        let shown = key.display();
        let typed = shown
            .replace('-', " ")
            .to_lowercase()
            .replace('0', "o")
            .replace('1', "l");
        assert_eq!(RecoveryKey::parse(&typed).unwrap(), key);
        assert_eq!(RecoveryKey::parse(&shown.replace('-', "")).unwrap(), key);
        assert_eq!(
            RecoveryKey::parse("0000-0000-0000-0000-0000-0000-000U"),
            Err(RecoveryKeyError::BadCharacter)
        );
        assert_eq!(
            RecoveryKey::parse("0000-0000-0000-0000-0000-0000-000"),
            Err(RecoveryKeyError::WrongLength)
        );
        assert_eq!(
            RecoveryKey::parse("8000-0000-0000-0000-0000-0000-0000"),
            Err(RecoveryKeyError::Checksum)
        );
        // The spec's layout example has no real checksum.
        assert_eq!(
            RecoveryKey::parse("0K7M-QX9P-3J2H-ZD8W-5TNB-1RGV-E6CA"),
            Err(RecoveryKeyError::Checksum)
        );
        assert!(format!("{key:?}").contains("redacted"));
    }

    #[test]
    fn kdf_bounds() {
        assert!(KdfParams::DEFAULT.is_within(&KdfParams::FLOOR));
        assert!(KdfParams::CEILING.is_within(&KdfParams::FLOOR));
        assert!(!KdfParams::TEST.is_within(&KdfParams::FLOOR));
        assert!(KdfParams::TEST.is_within(&KdfParams::TEST));
        assert!(
            !KdfParams {
                m_kib: 2_097_153,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::FLOOR)
        );
        assert!(
            !KdfParams {
                t: 9,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::FLOOR)
        );
        assert!(
            !KdfParams {
                p: 0,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::TEST)
        );
    }

    fn unhex<const N: usize>(text: &Value) -> [u8; N] {
        let text = text.as_str().expect("a hex string");
        let bytes: Vec<u8> = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
            .collect();
        bytes.try_into().expect("the right length")
    }

    fn golden() -> Value {
        serde_json::from_str(include_str!("../tests/fixtures/crypto-v1.json"))
            .expect("crypto-v1.json")
    }

    fn golden_password(v: &Value) -> (String, String, Password, [u8; 16]) {
        let i = &v["inputs"];
        let origin = i["origin"].as_str().unwrap().to_owned();
        let username = i["username"].as_str().unwrap().to_owned();
        let password = Password::new(i["password"].as_str().unwrap(), &username).unwrap();
        (origin, username, password, unhex(&i["salt"]))
    }

    fn params(v: &Value) -> KdfParams {
        KdfParams {
            m_kib: v["m_kib"].as_u64().unwrap() as u32,
            t: v["t"].as_u64().unwrap() as u32,
            p: v["p"].as_u64().unwrap() as u32,
        }
    }

    /// Golden vectors (`tests/fixtures/crypto-v1.json`, written by an
    /// independent Node implementation): every derivation, wrap, seal and
    /// encoding of format 1 gives exactly the recorded output.
    #[test]
    fn golden_vectors_match_crypto_v1() {
        let v = golden();
        let (i, o) = (&v["inputs"], &v["outputs"]);
        let (origin, username, password, salt) = golden_password(&v);
        let account = i["account_id"].as_str().unwrap();
        let nonce = |name: &str| unhex::<24>(&i["nonces"][name]);

        assert_eq!(hex(&salt_input(&origin, &username, &salt)), o["salt_input"]);
        let keys = password_keys(&password, &origin, &username, &salt, params(&i["kdf"])).unwrap();
        assert_eq!(keys.auth_key.to_base64url(), o["auth_key"]);
        assert_eq!(hex(keys.kek.bytes()), o["kek"]);

        let recovery_key = RecoveryKey::from_bytes(unhex(&i["recovery_key_bytes"]));
        assert_eq!(recovery_key.display(), o["recovery_key"]);
        assert_eq!(
            RecoveryKey::parse(o["recovery_key"].as_str().unwrap()).unwrap(),
            recovery_key
        );
        let mut recovery_salt = b"kanso/v1/recovery".to_vec();
        lp(&mut recovery_salt, &origin);
        lp(&mut recovery_salt, &username);
        assert_eq!(hex(&recovery_salt), o["recovery_salt"]);
        let recovery = recovery_keys(&recovery_key, &origin, &username);
        assert_eq!(recovery.auth_key.to_base64url(), o["recovery_auth_key"]);
        assert_eq!(hex(recovery.kek.bytes()), o["recovery_kek"]);

        let data_key = DataKey::from_bytes(unhex(&i["data_key"]));
        let wrapped = wrap_with_nonce(
            &keys.kek,
            WrapPurpose::Password,
            account,
            &data_key,
            &nonce("wrap_password"),
        );
        assert_eq!(wrapped, o["wrap_password"]);
        assert_eq!(wrapped.len(), 98);
        let wrapped_r = wrap_with_nonce(
            &recovery.kek,
            WrapPurpose::Recovery,
            account,
            &data_key,
            &nonce("wrap_recovery"),
        );
        assert_eq!(wrapped_r, o["wrap_recovery"]);
        assert_eq!(
            unwrap(&keys.kek, WrapPurpose::Password, account, &wrapped).unwrap(),
            data_key
        );
        assert_eq!(
            unwrap(&recovery.kek, WrapPurpose::Recovery, account, &wrapped_r).unwrap(),
            data_key
        );
        assert_eq!(key_id(&data_key), o["key_id"]);

        // Passkeys: the fixed PRF input, the CTAP salt, the user handle, the
        // passkey KEK's salt and info, and one wrap with purpose 0x4B.
        let passkey = &i["passkey"];
        let prf = unhex::<32>(&passkey["prf_out"]);
        let credential = passkey["credential_id"].as_str().unwrap();
        assert_eq!(hex(&PRF_INPUT), o["prf_input"]);
        assert_eq!(hex(&crate::security_key::ctap_salt()), o["ctap_salt"]);
        assert_eq!(hex(&crate::passkey::user_handle(account)), o["user_handle"]);
        let mut passkey_salt = b"kanso/v1/passkey".to_vec();
        passkey_salt.push(FORMAT_V1);
        lp(&mut passkey_salt, &origin);
        assert_eq!(hex(&passkey_salt), o["passkey_salt"]);
        let mut passkey_info = b"kanso/v1/passkey-kek".to_vec();
        passkey_info.push(FORMAT_V1);
        lp(&mut passkey_info, account);
        lp(&mut passkey_info, credential);
        assert_eq!(hex(&passkey_info), o["passkey_info"]);
        let passkey_kek = passkey_kek(&prf, &origin, account, credential);
        assert_eq!(hex(passkey_kek.bytes()), o["passkey_kek"]);
        assert_eq!(
            hex(&wrap_aad(WrapPurpose::Passkey, account)),
            o["passkey_aad"]
        );
        let wrapped_p = wrap_with_nonce(
            &passkey_kek,
            WrapPurpose::Passkey,
            account,
            &data_key,
            &unhex::<24>(&passkey["nonce"]),
        );
        assert_eq!(wrapped_p, o["wrap_passkey"]);
        assert_eq!(
            crate::passkey::open_passkey_wrapped_key(
                &prf,
                &origin,
                account,
                &unb64(credential).unwrap(),
                &wrapped_p
            )
            .unwrap(),
            data_key
        );

        let content = ContentKeys::derive(&data_key);
        let derived = [
            ("doc", &content.doc_key),
            ("file", &content.file_key),
            ("wire-id", &content.id_key),
            ("wire-rev", &content.rev_key),
            ("device-name", &content.device_name_key),
        ];
        for (name, key) in derived {
            assert_eq!(hex(&key[..]), o["content_keys"][name], "{name}");
        }

        let doc = SyncDoc {
            id: i["doc"]["id"].as_str().unwrap().to_owned(),
            rev: i["doc"]["rev"].as_str().unwrap().to_owned(),
            deleted: false,
            fields: i["doc"]["fields"].as_object().unwrap().clone(),
            base_rev: None,
        };
        let sealed = content.seal_doc_with_nonce(account, &doc, &nonce("doc"));
        let expected: WireDoc = serde_json::from_value(o["sealed_doc"].clone()).unwrap();
        assert_eq!(sealed, expected);
        assert_eq!(content.wire_id(&doc.id), expected.id);
        assert_eq!(content.wire_rev(&doc.id, &doc.rev), expected.rev);
        assert_eq!(content.open_doc(account, &sealed), Opened::Opened(doc));

        let tombstone = SyncDoc {
            id: i["tombstone"]["id"].as_str().unwrap().to_owned(),
            rev: i["tombstone"]["rev"].as_str().unwrap().to_owned(),
            deleted: true,
            fields: Map::new(),
            base_rev: None,
        };
        let sealed = content.seal_doc_with_nonce(account, &tombstone, &nonce("tombstone"));
        let expected: WireDoc = serde_json::from_value(o["sealed_tombstone"].clone()).unwrap();
        assert_eq!(sealed, expected);
        assert_eq!(
            content.open_doc(account, &sealed),
            Opened::Opened(tombstone)
        );

        let file_id = i["file"]["id"].as_str().unwrap();
        let file_wire = content.wire_id(file_id);
        assert_eq!(file_wire, o["file_wire_id"]);
        let bytes = unhex::<72>(&i["file"]["bytes"]);
        let sealed = content.seal_file_with_nonce(account, &file_wire, &bytes, &nonce("file"));
        assert_eq!(b64(&sealed), o["sealed_file"]);
        assert_eq!(
            content.open_file(account, &file_wire, &sealed),
            Opened::Opened(bytes.to_vec())
        );

        let secret: [u8; 32] = unhex(&i["device_secret"]);
        let device = device_id(&secret, account);
        assert_eq!(device, o["device_id"]);
        assert_eq!(
            device_proof(&secret, &origin, account).to_base64url(),
            o["device_proof"]
        );
        let name = i["device_name"].as_str().unwrap();
        let sealed = content.seal_name_with_nonce(account, &device, name, &nonce("name"));
        assert_eq!(sealed, o["sealed_name"]);
        assert_eq!(
            content.open_name(account, &device, &sealed),
            Opened::Opened(name.to_owned())
        );
    }

    /// The golden password vector at the production parameters.
    #[test]
    #[ignore = "Argon2id at the production parameters needs 1 GiB and about a second"]
    fn golden_vector_at_production_parameters() {
        let v = golden();
        let (origin, username, password, salt) = golden_password(&v);
        let keys = password_keys(
            &password,
            &origin,
            &username,
            &salt,
            params(&v["production"]["kdf"]),
        )
        .unwrap();
        assert_eq!(keys.auth_key.to_base64url(), v["production"]["auth_key"]);
        assert_eq!(hex(keys.kek.bytes()), v["production"]["kek"]);
    }

    fn key_bytes() -> impl Strategy<Value = [u8; 16]> {
        prop_oneof![Just([0u8; 16]), Just([0xffu8; 16]), any::<[u8; 16]>(),]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// The copy-free comparison agrees with lowercasing both, for the
        /// usernames Kanso allows and passwords of any case.
        #[test]
        fn prop_same_ignoring_case_is_lowercase_equality(
            name in "[a-z0-9_]{3,20}",
            flips in prop::collection::vec(any::<bool>(), 20),
            other in "\\PC{0,24}",
        ) {
            let flipped: String = name
                .chars()
                .zip(flips.iter().cycle())
                .map(|(c, &up)| if up { c.to_ascii_uppercase() } else { c })
                .collect();
            prop_assert!(same_ignoring_case(&flipped, &name));
            prop_assert_eq!(
                same_ignoring_case(&other, &name),
                other.to_lowercase() == name.to_lowercase()
            );
        }

        /// Round trip: a displayed key parses back to the same bytes.
        #[test]
        fn prop_recovery_key_round_trips(bytes in key_bytes()) {
            let key = RecoveryKey::from_bytes(bytes);
            let shown = key.display();
            prop_assert_eq!(shown.len(), 34);
            prop_assert_eq!(RecoveryKey::parse(&shown).unwrap(), key);
        }

        /// Invariant: every single-character substitution is refused.
        #[test]
        fn prop_every_substitution_is_detected(bytes in key_bytes(), at in 0..28usize, with in 0..32usize) {
            let shown: Vec<u8> = RecoveryKey::from_bytes(bytes).display().bytes().filter(|&b| b != b'-').collect();
            prop_assume!(shown[at] != ALPHABET[with]);
            let mut typo = shown.clone();
            typo[at] = ALPHABET[with];
            prop_assert!(RecoveryKey::parse(std::str::from_utf8(&typo).unwrap()).is_err());
        }

        /// Invariant: every adjacent transposition within the 26 data
        /// characters is refused.
        #[test]
        fn prop_every_adjacent_transposition_is_detected(bytes in key_bytes(), at in 0..25usize) {
            let shown: Vec<u8> = RecoveryKey::from_bytes(bytes).display().bytes().filter(|&b| b != b'-').collect();
            if shown[at] == shown[at + 1] {
                return Ok(());
            }
            let mut typo = shown.clone();
            typo.swap(at, at + 1);
            prop_assert!(RecoveryKey::parse(std::str::from_utf8(&typo).unwrap()).is_err());
        }

        /// Metamorphic: NFC and NFD spellings of a password are the same password.
        #[test]
        fn prop_nfc_and_nfd_passwords_are_equal(text in "[a-zé\u{e9}\u{301}ñÅ ]{12,40}") {
            let nfc: String = text.nfc().collect();
            let nfd: String = text.nfd().collect();
            match (Password::new(&nfc, "someone"), Password::new(&nfd, "someone")) {
                (Ok(a), Ok(b)) => prop_assert_eq!(a.as_bytes(), b.as_bytes()),
                (a, b) => prop_assert_eq!(a.map(|_| ()), b.map(|_| ())),
            }
        }

        /// Refinement: an accepted password has 12–256 scalar values in NFC
        /// and differs from the username ignoring case.
        #[test]
        fn prop_accepted_passwords_are_refined(text in "\\PC{0,300}", username in "[a-zA-Z0-9]{3,20}") {
            if let Ok(password) = Password::new(&text, &username) {
                let stored = std::str::from_utf8(password.as_bytes()).unwrap();
                prop_assert!((MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&stored.chars().count()));
                prop_assert_eq!(stored.nfc().collect::<String>(), stored);
                prop_assert_ne!(stored.to_lowercase(), username.to_lowercase());
            }
        }
    }
}
