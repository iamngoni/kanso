//! Locked notes: a note whose text only opens with the lock password.
//!
//! The password gives a key through Argon2id over a random 16-byte salt. A
//! locked note's text is sealed as `0x01 || nonce(24) ||
//! XChaCha20-Poly1305(key, nonce, aad, {"body": …})`, with a fresh random
//! nonce for every save and `aad = "kanso/v1/lock" || lp(note_id)`, so a
//! sealed text can't be moved onto another note. The result is kept in the
//! note, as `note.extra.kansoLock` ([`Envelope`]), with the salt and Argon2
//! parameters beside it, and the note's `body` is left empty. It syncs inside
//! the note's own sealed document, so the account's other devices open it
//! with the same password and nothing else has to travel.
//!
//! Notes locked on one device share its salt, so unlocking derives one key;
//! a note locked elsewhere has another salt, and the password derives a key
//! for it too. There is no way back from a forgotten password.
//!
//! Sealed attachments ride on the same lock. Each file gets its own random
//! 32-byte [`FileKey`]; the bytes are stored as a [`seal_container`]
//! (`0x01 || segments` header, then per segment `kind`, little-endian
//! plaintext length and an `0x01 || nonce(24) || ciphertext || tag(16)`
//! seal), bound to the file and this seal generation by its AAD, and the
//! file key itself is wrapped under the note's key ([`wrap_file_key`]) and
//! kept beside the note's sealed text. Opening the note unwraps the file
//! keys ([`unwrap_file_key`]); the stored containers are never rewritten.
//! Changing the lock password re-wraps the file keys, not the bytes.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use unicode_normalization::UnicodeNormalization as _;
use zeroize::Zeroizing;

use crate::crypto::{
    FORMAT_V1, KdfParams, NONCE_LEN, Opened, b64, hkdf32, lp, open_raw, random, seal_raw, unb64,
};

/// Where a note keeps its sealed text, beside its other fields.
pub const LOCK_KEY: &str = kanso_core::model::LOCK_FIELD;

/// What new lock keys use: 256 MiB, t = 3, p = 4. Unlocking derives once per
/// salt, so a fraction of a second, while guessing stays costly.
pub const LOCK_KDF: KdfParams = KdfParams {
    m_kib: 262_144,
    t: 3,
    p: 4,
};

/// Shortest and longest lock password, in Unicode scalar values after NFC.
pub const MIN_LOCK_PASSWORD_CHARS: usize = 8;
pub const MAX_LOCK_PASSWORD_CHARS: usize = 256;

const AAD_PREFIX: &[u8] = b"kanso/v1/lock";

// -- Sealed attachments ----------------------------------------------------
//
// A sealed attachment's bytes are a container of one or more segments, each
// sealed with the file's random key and a fresh nonce, bound to the file
// and this seal generation. The file key is wrapped under a key derived
// from the note's lock key (HKDF), with the note's salt chained into the
// wrap's AAD, so a wrap from before a password change does not open after
// it. All lengths are little-endian, as the container format says.

/// The format byte of a sealed attachment's container (`blobs.data`).
pub const FILE_CONTAINER_FORMAT: u8 = 0x01;

/// Segment kinds: the content segment holds exactly the attached bytes; the
/// metadata segment (a later step) holds canonical JSON
/// `{name, contentType, publicIn}`.
pub const SEGMENT_CONTENT: u8 = 0;
pub const SEGMENT_METADATA: u8 = 1;

/// Bytes a segment adds beyond its plaintext: the format byte, the 24-byte
/// nonce and the 16-byte tag.
const SEGMENT_OVERHEAD: usize = 1 + NONCE_LEN + 16;

/// Bytes a content-only container adds beyond its plaintext: the 2-byte
/// header plus the segment's kind, 4-byte length and [`SEGMENT_OVERHEAD`].
const CONTAINER_OVERHEAD: usize = 2 + 1 + 4 + SEGMENT_OVERHEAD;

const FILE_AAD_PREFIX: &[u8] = b"kanso/v1/lock-file";
const FILE_KEY_AAD_PREFIX: &[u8] = b"kanso/v1/lock-file-key";
const FILE_KEY_HKDF_INFO: &[u8] = b"kanso/v1/lock-file-key";

const SEAL_ID_LEN: usize = 16;
const FILE_KEY_LEN: usize = 32;

/// Appends `lp(bytes)`: a 2-byte big-endian byte length, then the bytes,
/// matching [`lp`] for fixed binary values such as a seal ID.
fn lp_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = u16::try_from(bytes.len()).expect("AAD binary fields fit in 16 bits");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(bytes);
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SealError {
    /// The container (or a wrapped file key) doesn't parse: a bad length, a
    /// truncated or trailing chunk, or an unknown segment kind.
    #[error("the sealed attachment is damaged")]
    Malformed,
    /// The container (or a wrapped file key) was made by a newer Kanso.
    #[error("the attachment was sealed by a newer Kanso; update Kanso to open it")]
    TooNew,
    /// The container doesn't open under this key and AAD: the wrong file,
    /// seal generation, note, salt or password.
    #[error("the attachment doesn't open under this key")]
    WrongKey,
}

/// One plaintext segment of a sealed attachment's container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedSegment {
    /// [`SEGMENT_CONTENT`] (or [`SEGMENT_METADATA`] in a later step).
    pub kind: u8,
    /// The plaintext of this segment.
    pub bytes: Vec<u8>,
}

/// A random 32-byte key for one seal generation, from the system's random
/// source ([`crate::crypto::random`]). Never derived from the password and
/// never stored unwrapped. Zeroized on drop; never printed.
pub struct FileKey(Zeroizing<[u8; FILE_KEY_LEN]>);

impl fmt::Debug for FileKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FileKey(<redacted>)")
    }
}

impl FileKey {
    pub fn random() -> FileKey {
        FileKey(Zeroizing::new(random::<FILE_KEY_LEN>()))
    }

    /// A key held in memory, as [`unwrap_file_key`] returns.
    pub fn from_bytes(bytes: [u8; FILE_KEY_LEN]) -> FileKey {
        FileKey(Zeroizing::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; FILE_KEY_LEN] {
        &self.0
    }
}

/// A random 16-byte value naming one seal generation. It binds every
/// segment's AAD (and the wrapped file key's) to this generation, so a
/// container moved to another file, or paired with another generation's
/// wrapped key, does not open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SealId([u8; SEAL_ID_LEN]);

impl SealId {
    pub fn random() -> SealId {
        SealId(random::<SEAL_ID_LEN>())
    }

    pub fn from_bytes(bytes: [u8; SEAL_ID_LEN]) -> SealId {
        SealId(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; SEAL_ID_LEN] {
        &self.0
    }

    /// Base64url, as the file document keeps it (`kansoFileLock.seal`).
    pub fn encode(&self) -> String {
        b64(&self.0)
    }

    pub fn decode(text: &str) -> Option<SealId> {
        unb64(text)
            .and_then(|bytes| bytes.try_into().ok())
            .map(SealId)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LockError {
    #[error("use at least {MIN_LOCK_PASSWORD_CHARS} characters")]
    TooShort,
    #[error("use at most {MAX_LOCK_PASSWORD_CHARS} characters")]
    TooLong,
    /// The text didn't open under this key: the wrong password, or a sealed
    /// text that was changed.
    #[error("that isn't the lock password")]
    WrongPassword,
    /// The note's `kansoLock` isn't an envelope this version reads.
    #[error("the note's locked text is damaged")]
    Malformed,
    #[error("the note was locked by a newer Kanso; update Kanso to open it")]
    TooNew,
    #[error("key derivation failed: {0}")]
    Kdf(String),
}

/// A lock password, normalized to Unicode NFC. Zeroized on drop; never
/// printed.
pub struct LockPassword(Zeroizing<String>);

impl LockPassword {
    pub fn new(text: &str) -> Result<LockPassword, LockError> {
        let normalized = Zeroizing::new(text.nfc().collect::<String>());
        let chars = normalized.chars().count();
        if chars < MIN_LOCK_PASSWORD_CHARS {
            return Err(LockError::TooShort);
        }
        if chars > MAX_LOCK_PASSWORD_CHARS {
            return Err(LockError::TooLong);
        }
        Ok(LockPassword(normalized))
    }
}

impl fmt::Debug for LockPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LockPassword(<redacted>)")
    }
}

/// A locked note's `kansoLock`: how its key is derived and its sealed text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// The format of `data`: 1.
    pub v: u8,
    /// The Argon2id salt, base64url.
    pub salt: String,
    pub m: u32,
    pub t: u32,
    pub p: u32,
    /// The sealed text, base64url.
    pub data: String,
}

impl Envelope {
    /// The note's envelope, if it is locked; `Err` for a `kansoLock` that
    /// isn't one.
    pub fn from_extra(extra: &Map<String, Value>) -> Option<Result<Envelope, LockError>> {
        let value = extra.get(LOCK_KEY)?;
        Some(match serde_json::from_value::<Envelope>(value.clone()) {
            Ok(envelope) if envelope.v > FORMAT_V1 => Err(LockError::TooNew),
            Ok(envelope) => Ok(envelope),
            Err(_) => Err(LockError::Malformed),
        })
    }

    /// Keeps `envelope` in a note's fields, or takes the lock out.
    pub fn store(envelope: Option<&Envelope>, extra: &mut Map<String, Value>) {
        match envelope.and_then(|e| serde_json::to_value(e).ok()) {
            Some(value) => extra.insert(LOCK_KEY.to_owned(), value),
            None => extra.remove(LOCK_KEY),
        };
    }

    fn salt(&self) -> Option<[u8; 16]> {
        unb64(&self.salt)?.try_into().ok()
    }

    fn params(&self) -> KdfParams {
        KdfParams {
            m_kib: self.m,
            t: self.t,
            p: self.p,
        }
    }
}

/// The text a locked note seals.
#[derive(Serialize, Deserialize)]
struct Sealed {
    body: String,
}

/// A key derived from the lock password. Zeroized on drop; never printed.
pub struct LockKey {
    key: Zeroizing<[u8; 32]>,
    salt: [u8; 16],
    params: KdfParams,
}

impl fmt::Debug for LockKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LockKey(<redacted>)")
    }
}

impl LockKey {
    /// A key for locking notes on this device, with a new random salt.
    pub fn create(password: &LockPassword, params: KdfParams) -> Result<LockKey, LockError> {
        LockKey::derive(password, random::<16>(), params)
    }

    /// The key `password` gives for `envelope`'s salt and parameters. It may
    /// still be the wrong one: [`LockKey::open`] tells. Parameters outside
    /// [`KdfParams::TEST`]..[`KdfParams::CEILING`] are refused before any
    /// work, so a crafted envelope can't make unlocking take all memory.
    pub fn for_envelope(
        password: &LockPassword,
        envelope: &Envelope,
    ) -> Result<LockKey, LockError> {
        let salt = envelope.salt().ok_or(LockError::Malformed)?;
        let params = envelope.params();
        if !params.is_within(&KdfParams::TEST) {
            return Err(LockError::Malformed);
        }
        LockKey::derive(password, salt, params)
    }

    fn derive(
        password: &LockPassword,
        salt: [u8; 16],
        params: KdfParams,
    ) -> Result<LockKey, LockError> {
        let kdf = |e: argon2::Error| LockError::Kdf(e.to_string());
        let argon = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(params.m_kib, params.t, params.p, Some(32)).map_err(kdf)?,
        );
        let mut key = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(password.0.as_bytes(), &salt, &mut key[..])
            .map_err(kdf)?;
        Ok(LockKey { key, salt, params })
    }

    /// The key with its salt and parameters, as the Keychain keeps it for
    /// Touch ID: `1:<key>:<salt>:<m>:<t>:<p>`, base64url and decimal.
    pub fn to_secret(&self) -> Zeroizing<String> {
        let key = Zeroizing::new(b64(&self.key[..]));
        Zeroizing::new(format!(
            "{FORMAT_V1}:{}:{}:{}:{}:{}",
            key.as_str(),
            b64(&self.salt),
            self.params.m_kib,
            self.params.t,
            self.params.p
        ))
    }

    /// A key [`LockKey::to_secret`] kept.
    pub fn from_secret(text: &str) -> Result<LockKey, LockError> {
        let parts: Vec<&str> = text.split(':').collect();
        let [version, key, salt, m, t, p] = parts[..] else {
            return Err(LockError::Malformed);
        };
        match version.parse::<u8>() {
            Ok(FORMAT_V1) => {}
            Ok(_) => return Err(LockError::TooNew),
            Err(_) => return Err(LockError::Malformed),
        }
        let key = Zeroizing::new(unb64(key).ok_or(LockError::Malformed)?);
        let key =
            Zeroizing::new(<[u8; 32]>::try_from(key.as_slice()).map_err(|_| LockError::Malformed)?);
        let salt = unb64(salt)
            .and_then(|salt| salt.try_into().ok())
            .ok_or(LockError::Malformed)?;
        let number = |text: &str| text.parse::<u32>().map_err(|_| LockError::Malformed);
        Ok(LockKey {
            key,
            salt,
            params: KdfParams {
                m_kib: number(m)?,
                t: number(t)?,
                p: number(p)?,
            },
        })
    }

    /// Whether this key was derived for `envelope`'s salt and parameters,
    /// and so is the one to try on it.
    pub fn fits(&self, envelope: &Envelope) -> bool {
        envelope.salt() == Some(self.salt) && envelope.params() == self.params
    }

    /// Whether `other` was derived with the same salt and parameters: the
    /// same key, if from the same password.
    pub fn same_kdf(&self, other: &LockKey) -> bool {
        self.salt == other.salt && self.params == other.params
    }

    /// Seals `body` for note `note_id`, with a fresh nonce.
    pub fn seal(&self, note_id: &str, body: &str) -> Envelope {
        self.seal_with_nonce(note_id, body, &random::<NONCE_LEN>())
    }

    fn seal_with_nonce(&self, note_id: &str, body: &str, nonce: &[u8; NONCE_LEN]) -> Envelope {
        let plaintext = Zeroizing::new(
            serde_json::to_vec(&Sealed {
                body: body.to_owned(),
            })
            .expect("a string serializes"),
        );
        Envelope {
            v: FORMAT_V1,
            salt: b64(&self.salt),
            m: self.params.m_kib,
            t: self.params.t,
            p: self.params.p,
            data: b64(&seal_raw(&self.key, nonce, &aad(note_id), &plaintext)),
        }
    }

    /// The text `envelope` seals for note `note_id`.
    pub fn open(&self, note_id: &str, envelope: &Envelope) -> Result<Zeroizing<String>, LockError> {
        if envelope.v > FORMAT_V1 {
            return Err(LockError::TooNew);
        }
        let sealed = unb64(&envelope.data).ok_or(LockError::Malformed)?;
        match open_raw(&self.key, &aad(note_id), &sealed) {
            Opened::Opened(plaintext) => {
                let sealed: Sealed =
                    serde_json::from_slice(&plaintext).map_err(|_| LockError::Malformed)?;
                Ok(Zeroizing::new(sealed.body))
            }
            Opened::TooNew => Err(LockError::TooNew),
            Opened::Undecryptable => Err(LockError::WrongPassword),
        }
    }
}

fn aad(note_id: &str) -> Vec<u8> {
    let mut aad = AAD_PREFIX.to_vec();
    lp(&mut aad, note_id);
    aad
}

/// A segment's AAD: `"kanso/v1/lock-file" || 0x01 || lp(file_id) ||
/// lp(seal_id) || kind`, binding the bytes to the file, the seal
/// generation and the segment's role.
fn file_aad(file_id: &str, seal_id: &SealId, kind: u8) -> Vec<u8> {
    let mut aad = FILE_AAD_PREFIX.to_vec();
    aad.push(FILE_CONTAINER_FORMAT);
    lp(&mut aad, file_id);
    lp_bytes(&mut aad, &seal_id.0);
    aad.push(kind);
    aad
}

/// A wrapped file key's AAD: `"kanso/v1/lock-file-key" || 0x01 ||
/// lp(note_id) || lp(file_id) || lp(seal_id) || lp(envelope.salt)`. The
/// salt makes a wrap from before a password change fail after it, and the
/// note makes a wrap for another note fail here.
fn file_key_aad(envelope: &Envelope, note_id: &str, file_id: &str, seal_id: &SealId) -> Vec<u8> {
    let mut aad = FILE_KEY_AAD_PREFIX.to_vec();
    aad.push(FILE_CONTAINER_FORMAT);
    lp(&mut aad, note_id);
    lp(&mut aad, file_id);
    lp_bytes(&mut aad, &seal_id.0);
    lp(&mut aad, &envelope.salt);
    aad
}

/// Seals `segments` as one container for `file_id` under this seal
/// generation, each segment under the same [`FileKey`] with its own fresh
/// nonce. The content segment holds exactly the bytes that were attached;
/// callers pass at least it. The container overhead is 48 bytes for the
/// content-only form.
pub fn seal_container(
    file_id: &str,
    seal_id: &SealId,
    file_key: &FileKey,
    segments: &[SealedSegment],
) -> Vec<u8> {
    assert!(segments.len() <= u8::MAX as usize, "at most 255 segments");
    let mut out = Vec::with_capacity(
        CONTAINER_OVERHEAD + segments.iter().map(|s| s.bytes.len()).sum::<usize>(),
    );
    out.push(FILE_CONTAINER_FORMAT);
    out.push(segments.len() as u8);
    for segment in segments {
        out.push(segment.kind);
        let len = u32::try_from(segment.bytes.len()).expect("a segment fits in 4 GiB");
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&seal_raw(
            file_key.as_bytes(),
            &random::<NONCE_LEN>(),
            &file_aad(file_id, seal_id, segment.kind),
            &segment.bytes,
        ));
    }
    out
}

/// Opens a [`seal_container`]: checks every length and the AAD of every
/// segment, and returns each segment's plaintext. A container of a newer
/// format is [`SealError::TooNew`]; one truncated, with a bad length or
/// trailing bytes is [`SealError::Malformed`]; one under another file,
/// seal generation or key is [`SealError::WrongKey`].
pub fn open_container(
    file_id: &str,
    seal_id: &SealId,
    file_key: &FileKey,
    container: &[u8],
) -> Result<Vec<SealedSegment>, SealError> {
    let (&format, mut rest) = container.split_first().ok_or(SealError::Malformed)?;
    if format > FILE_CONTAINER_FORMAT {
        return Err(SealError::TooNew);
    }
    if format != FILE_CONTAINER_FORMAT {
        return Err(SealError::Malformed);
    }
    let (&segments, after) = rest.split_first().ok_or(SealError::Malformed)?;
    rest = after;
    let mut opened = Vec::with_capacity(segments as usize);
    for _ in 0..segments {
        let (&kind, after) = rest.split_first().ok_or(SealError::Malformed)?;
        if kind != SEGMENT_CONTENT && kind != SEGMENT_METADATA {
            return Err(SealError::Malformed);
        }
        rest = after;
        // The length is the plaintext length; the sealed chunk holds that
        // plus the format byte, nonce and tag.
        let length = read_u32_le(rest)?;
        rest = &rest[4..];
        let length = usize::try_from(length).map_err(|_| SealError::Malformed)?;
        let sealed_len = length
            .checked_add(SEGMENT_OVERHEAD)
            .ok_or(SealError::Malformed)?;
        let Some(sealed) = rest.get(..sealed_len) else {
            return Err(SealError::Malformed);
        };
        rest = &rest[sealed_len..];
        let bytes = match open_raw(
            file_key.as_bytes(),
            &file_aad(file_id, seal_id, kind),
            sealed,
        ) {
            Opened::Opened(bytes) => bytes.to_vec(),
            Opened::TooNew => return Err(SealError::TooNew),
            Opened::Undecryptable => return Err(SealError::WrongKey),
        };
        opened.push(SealedSegment { kind, bytes });
    }
    if !rest.is_empty() {
        return Err(SealError::Malformed);
    }
    Ok(opened)
}

/// Reads a little-endian `u32` from `bytes`, which must hold at least 4.
fn read_u32_le(bytes: &[u8]) -> Result<u32, SealError> {
    let mut four = [0u8; 4];
    four.copy_from_slice(bytes.get(..4).ok_or(SealError::Malformed)?);
    Ok(u32::from_le_bytes(four))
}

/// Wraps `file_key` under the note's lock key, to sit in the note's
/// `kansoFiles` list: `HKDF-SHA256(note_lock_key, "kanso/v1/lock-file-key")`
/// gives the wrapping key, then
/// `0x01 || nonce(24) || XChaCha20-Poly1305(wrap_key, nonce, aad, file_key)`
/// with [`file_key_aad`], so the wrap names the note, file, seal generation
/// and salt. 73 bytes before base64.
pub fn wrap_file_key(
    lock_key: &LockKey,
    envelope: &Envelope,
    note_id: &str,
    file_id: &str,
    seal_id: &SealId,
    file_key: &FileKey,
) -> Vec<u8> {
    let wrap_key = hkdf32(&lock_key.key[..], None, FILE_KEY_HKDF_INFO);
    seal_raw(
        &wrap_key,
        &random::<NONCE_LEN>(),
        &file_key_aad(envelope, note_id, file_id, seal_id),
        file_key.as_bytes(),
    )
}

/// Opens what [`wrap_file_key`] sealed: the file key for `note_id`, `file_id`
/// and `seal_id`, only under the key derived for `envelope`'s salt and
/// parameters. A wrap for another note, file, generation or salt — or under
/// another password — is [`SealError::WrongKey`].
pub fn unwrap_file_key(
    lock_key: &LockKey,
    envelope: &Envelope,
    note_id: &str,
    file_id: &str,
    seal_id: &SealId,
    wrapped: &[u8],
) -> Result<FileKey, SealError> {
    let wrap_key = hkdf32(&lock_key.key[..], None, FILE_KEY_HKDF_INFO);
    match open_raw(
        &wrap_key,
        &file_key_aad(envelope, note_id, file_id, seal_id),
        wrapped,
    ) {
        Opened::Opened(bytes) => bytes
            .as_slice()
            .try_into()
            .map(FileKey::from_bytes)
            .map_err(|_| SealError::Malformed),
        Opened::TooNew => Err(SealError::TooNew),
        Opened::Undecryptable => Err(SealError::WrongKey),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn password(text: &str) -> LockPassword {
        LockPassword::new(text).unwrap()
    }

    fn key(text: &str) -> LockKey {
        LockKey::create(&password(text), KdfParams::TEST).unwrap()
    }

    #[test]
    fn passwords_need_eight_characters_after_nfc() {
        assert_eq!(
            LockPassword::new("seven77").unwrap_err(),
            LockError::TooShort
        );
        assert!(LockPassword::new("eight888").is_ok());
        // "é" as e + combining acute is one character once composed.
        assert_eq!(
            LockPassword::new("cafe\u{301}xyz").unwrap_err(),
            LockError::TooShort
        );
        assert_eq!(
            LockPassword::new(&"x".repeat(257)).unwrap_err(),
            LockError::TooLong
        );
    }

    #[test]
    fn the_password_opens_on_another_device_with_the_envelopes_salt() {
        let mine = key("correct horse");
        let envelope = mine.seal("note:a", "secret plans");
        let theirs = LockKey::for_envelope(&password("correct horse"), &envelope).unwrap();
        assert!(theirs.fits(&envelope));
        assert_eq!(
            theirs.open("note:a", &envelope).unwrap().as_str(),
            "secret plans"
        );
    }

    #[test]
    fn a_wrong_password_or_another_note_does_not_open() {
        let envelope = key("correct horse").seal("note:a", "secret plans");
        let wrong = LockKey::for_envelope(&password("battery staple"), &envelope).unwrap();
        assert_eq!(
            wrong.open("note:a", &envelope).unwrap_err(),
            LockError::WrongPassword
        );
        let right = LockKey::for_envelope(&password("correct horse"), &envelope).unwrap();
        assert_eq!(
            right.open("note:b", &envelope).unwrap_err(),
            LockError::WrongPassword
        );
    }

    #[test]
    fn an_envelope_says_nothing_of_its_text_and_keys_fit_only_their_salt() {
        let first = key("correct horse");
        let second = key("correct horse");
        let envelope = first.seal("note:a", "secret plans");
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(!json.contains("secret"));
        assert!(first.fits(&envelope));
        assert!(!second.fits(&envelope));
        // Each save gets its own nonce.
        assert_ne!(
            first.seal("note:a", "x").data,
            first.seal("note:a", "x").data
        );
    }

    #[test]
    fn damaged_too_new_and_unsafe_envelopes_are_refused() {
        let k = key("correct horse");
        let mut envelope = k.seal("note:a", "secret");
        let mut bad = envelope.clone();
        bad.data.push('!');
        assert_eq!(k.open("note:a", &bad).unwrap_err(), LockError::Malformed);
        let mut huge = envelope.clone();
        huge.m = KdfParams::CEILING.m_kib + 1;
        assert_eq!(
            LockKey::for_envelope(&password("correct horse"), &huge).unwrap_err(),
            LockError::Malformed
        );
        envelope.v = 2;
        let mut extra = Map::new();
        Envelope::store(Some(&envelope), &mut extra);
        assert_eq!(Envelope::from_extra(&extra), Some(Err(LockError::TooNew)));
        extra.insert(LOCK_KEY.into(), Value::String("nonsense".into()));
        assert_eq!(
            Envelope::from_extra(&extra),
            Some(Err(LockError::Malformed))
        );
        Envelope::store(None, &mut extra);
        assert_eq!(Envelope::from_extra(&extra), None);
    }

    #[test]
    fn a_kept_key_opens_what_the_original_sealed() {
        let k = key("correct horse");
        let envelope = k.seal("note:a", "secret");
        let kept = LockKey::from_secret(&k.to_secret()).unwrap();
        assert!(kept.fits(&envelope) && kept.same_kdf(&k));
        assert_eq!(kept.open("note:a", &envelope).unwrap().as_str(), "secret");
        assert_eq!(
            LockKey::from_secret("{}").unwrap_err(),
            LockError::Malformed
        );
        assert!(!format!("{kept:?}").contains(&k.to_secret()[..10]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]
        /// Invariant: whatever the text, sealing then opening gives it back,
        /// and the envelope round-trips through a note's fields.
        #[test]
        fn prop_seal_then_open_round_trips(body in "\\PC{0,200}", nonce in any::<[u8; 24]>()) {
            let k = key("correct horse");
            let envelope = k.seal_with_nonce("note:a", &body, &nonce);
            let mut extra = Map::new();
            Envelope::store(Some(&envelope), &mut extra);
            let back = Envelope::from_extra(&extra).unwrap().unwrap();
            let opened = k.open("note:a", &back).unwrap();
            prop_assert_eq!(opened.as_str(), body.as_str());
        }
    }

    // -- Sealed attachments ------------------------------------------------

    const FILE_ID: &str = "file:AbCd1234";
    const NOTE_ID: &str = "note:Plans";
    const CANARY: &[u8] = b"kanso-canary-4471";

    fn file_key() -> FileKey {
        FileKey::random()
    }

    fn content(bytes: &[u8]) -> SealedSegment {
        SealedSegment {
            kind: SEGMENT_CONTENT,
            bytes: bytes.to_vec(),
        }
    }

    #[test]
    fn a_container_round_trips_without_its_plaintext() {
        let mut plaintext = CANARY.to_vec();
        plaintext.extend_from_slice(b" and the rest of the picture");
        let seal = SealId::random();
        let fk = file_key();
        let container = seal_container(FILE_ID, &seal, &fk, &[content(&plaintext)]);
        // The content-only container adds exactly 48 bytes, and the
        // plaintext never appears in it.
        assert_eq!(container.len(), plaintext.len() + 48);
        assert!(!container.windows(CANARY.len()).any(|w| w == CANARY));
        let opened = open_container(FILE_ID, &seal, &fk, &container).unwrap();
        assert_eq!(opened, vec![content(&plaintext)]);
        // A fresh seal of the same bytes is a different container.
        assert_ne!(
            seal_container(FILE_ID, &seal, &fk, &[content(&plaintext)]),
            container
        );
    }

    #[test]
    fn a_container_holds_its_segments_in_order() {
        let seal = SealId::random();
        let fk = file_key();
        let segments = vec![
            SealedSegment {
                kind: SEGMENT_METADATA,
                bytes: br#"{"name":"shot.png","contentType":"image/png","publicIn":[]}"#.to_vec(),
            },
            content(b"bytes"),
        ];
        let container = seal_container(FILE_ID, &seal, &fk, &segments);
        let opened = open_container(FILE_ID, &seal, &fk, &container).unwrap();
        assert_eq!(opened, segments);
    }

    #[test]
    fn boundary_sizes_seal_and_open() {
        use kanso_core::model::MAX_ATTACHMENT_SIZE;
        let seal = SealId::random();
        let fk = file_key();
        for size in [0usize, 1, MAX_ATTACHMENT_SIZE, MAX_ATTACHMENT_SIZE + 1] {
            let plaintext = vec![0x5a; size];
            let container = seal_container(FILE_ID, &seal, &fk, &[content(&plaintext)]);
            let opened = open_container(FILE_ID, &seal, &fk, &container).unwrap();
            assert_eq!(opened[0].bytes, plaintext, "size {size}");
        }
    }

    #[test]
    fn malformed_truncated_and_too_new_containers_are_refused() {
        let seal = SealId::random();
        let fk = file_key();
        let plaintext = b"some bytes".to_vec();
        let container = seal_container(FILE_ID, &seal, &fk, &[content(&plaintext)]);
        // Every strict prefix is truncated.
        for cut in 0..container.len() {
            let opened = open_container(FILE_ID, &seal, &fk, &container[..cut]);
            match opened {
                Err(SealError::Malformed) => {}
                // The sealed segment's inner format byte survives some
                // truncations, but a cut container never opens.
                Err(SealError::WrongKey) => {}
                other => panic!("cut {cut} of {} opened as {other:?}", container.len()),
            }
        }
        // Trailing bytes.
        let mut trailing = container.clone();
        trailing.push(0);
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &trailing).unwrap_err(),
            SealError::Malformed
        );
        // An unknown segment kind.
        let mut bad_kind = container.clone();
        bad_kind[2] = 7;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &bad_kind).unwrap_err(),
            SealError::Malformed
        );
        // A future container format.
        let mut too_new = container.clone();
        too_new[0] = 0x02;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &too_new).unwrap_err(),
            SealError::TooNew
        );
        // Not a container format at all.
        let mut wrong_format = container.clone();
        wrong_format[0] = 0x00;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &wrong_format).unwrap_err(),
            SealError::Malformed
        );
        // A declared length that runs past the container.
        let mut bad_length = container.clone();
        bad_length[5] = 0xff;
        bad_length[6] = 0xff;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &bad_length).unwrap_err(),
            SealError::Malformed
        );
        // A segment count that runs past the container.
        let mut bad_count = container.clone();
        bad_count[1] = 2;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &bad_count).unwrap_err(),
            SealError::Malformed
        );
        // A future sealed segment format inside.
        let mut too_new_segment = container.clone();
        too_new_segment[7] = 0x02;
        assert_eq!(
            open_container(FILE_ID, &seal, &fk, &too_new_segment).unwrap_err(),
            SealError::TooNew
        );
    }

    #[test]
    fn the_aad_binds_the_file_the_seal_and_the_key() {
        let seal = SealId::random();
        let fk = file_key();
        let container = seal_container(FILE_ID, &seal, &fk, &[content(b"bytes")]);
        assert_eq!(
            open_container("file:another", &seal, &fk, &container).unwrap_err(),
            SealError::WrongKey
        );
        assert_eq!(
            open_container(FILE_ID, &SealId::random(), &fk, &container).unwrap_err(),
            SealError::WrongKey
        );
        assert_eq!(
            open_container(FILE_ID, &seal, &file_key(), &container).unwrap_err(),
            SealError::WrongKey
        );
    }

    #[test]
    fn wrapped_file_keys_round_trip_and_bind_the_note_the_salt_and_the_generation() {
        let k = key("correct horse");
        let envelope = k.seal(NOTE_ID, "plans");
        let seal = SealId::random();
        let fk = file_key();
        let wrapped = wrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &fk);
        assert_eq!(wrapped.len(), 73);
        assert_eq!(
            unwrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &wrapped)
                .unwrap()
                .as_bytes(),
            fk.as_bytes()
        );
        // Another note, file, generation, salt or password does not open.
        assert_eq!(
            unwrap_file_key(&k, &envelope, "note:other", FILE_ID, &seal, &wrapped).unwrap_err(),
            SealError::WrongKey
        );
        assert_eq!(
            unwrap_file_key(&k, &envelope, NOTE_ID, "file:other", &seal, &wrapped).unwrap_err(),
            SealError::WrongKey
        );
        assert_eq!(
            unwrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &SealId::random(), &wrapped)
                .unwrap_err(),
            SealError::WrongKey
        );
        let rekeyed = key("correct horse");
        let rekeyed_envelope = rekeyed.seal(NOTE_ID, "plans");
        assert_eq!(
            unwrap_file_key(
                &rekeyed,
                &rekeyed_envelope,
                NOTE_ID,
                FILE_ID,
                &seal,
                &wrapped
            )
            .unwrap_err(),
            SealError::WrongKey
        );
        let wrong = key("battery staple");
        let wrong_envelope = wrong.seal(NOTE_ID, "plans");
        assert_eq!(
            unwrap_file_key(&wrong, &wrong_envelope, NOTE_ID, FILE_ID, &seal, &wrapped)
                .unwrap_err(),
            SealError::WrongKey
        );
        // A damaged or future wrap.
        let mut short = wrapped.clone();
        short.truncate(5);
        assert_eq!(
            unwrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &short).unwrap_err(),
            SealError::WrongKey
        );
        let mut too_new = wrapped.clone();
        too_new[0] = 0x02;
        assert_eq!(
            unwrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &too_new).unwrap_err(),
            SealError::TooNew
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        /// Invariant: whatever the bytes and however they are split into
        /// segments, sealing then opening gives them back, and the seal
        /// never holds the plaintext canary.
        #[test]
        fn prop_seal_container_then_open_round_trips(
            chunks in proptest::collection::vec(any::<Vec<u8>>(), 1..=3),
            metadata in any::<Vec<u8>>(),
        ) {
            let seal = SealId::random();
            let fk = file_key();
            let plaintext: Vec<u8> = chunks.iter().flatten().copied().collect();
            let mut segments: Vec<SealedSegment> = chunks
                .into_iter()
                .map(|bytes| SealedSegment { kind: SEGMENT_CONTENT, bytes })
                .collect();
            segments.push(SealedSegment { kind: SEGMENT_METADATA, bytes: metadata });
            let container = seal_container(FILE_ID, &seal, &fk, &segments);
            let opened = open_container(FILE_ID, &seal, &fk, &container).unwrap();
            prop_assert_eq!(&opened, &segments);
            let content: Vec<u8> = opened
                .iter()
                .filter(|s| s.kind == SEGMENT_CONTENT)
                .flat_map(|s| s.bytes.iter().copied())
                .collect();
            prop_assert_eq!(content, plaintext);
        }

        /// Invariant: a container never holds a distinctive plaintext prefix,
        /// whatever the bytes around it.
        #[test]
        fn prop_container_never_holds_plaintext(tail in any::<Vec<u8>>()) {
            let mut plaintext = CANARY.to_vec();
            plaintext.extend_from_slice(&tail);
            let seal = SealId::random();
            let fk = file_key();
            let container = seal_container(FILE_ID, &seal, &fk, &[content(&plaintext)]);
            prop_assert!(!container.windows(CANARY.len()).any(|w| w == CANARY));
        }

        /// Invariant: a wrap made under a random seal opens under the same
        /// note, file, generation and salt, and the wrapped key carries the
        /// file key back whatever it was.
        #[test]
        fn prop_wrapped_file_keys_round_trip(bytes in any::<[u8; 32]>()) {
            let k = key("correct horse");
            let envelope = k.seal(NOTE_ID, "plans");
            let seal = SealId::random();
            let fk = FileKey::from_bytes(bytes);
            let wrapped = wrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &fk);
            let unwrapped = unwrap_file_key(&k, &envelope, NOTE_ID, FILE_ID, &seal, &wrapped).unwrap();
            prop_assert_eq!(unwrapped.as_bytes(), &bytes);
        }
    }
}
