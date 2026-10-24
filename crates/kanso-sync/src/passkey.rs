//! Passkey wire shapes and the PRF wrap (docs/ACCOUNTS.md › Passkeys, › Keys
//! for passkeys and security keys).
//!
//! The same shapes carry a platform passkey from macOS or Windows and a Linux
//! security key's CTAP2 result ([`crate::security_key`]). Pure: the platform
//! ceremony happens in the app, and nothing here talks to the network.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::crypto::{
    CryptoError, DataKey, PRF_INPUT, WrapPurpose, b64, passkey_kek, unb64, unwrap, wrap,
};

/// The server-issued options a platform registration ceremony consumes.
/// `origin` is the advertised WebAuthn client-data origin; key wrapping uses
/// the account's canonical server origin separately.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyRegistrationChallenge {
    pub rp_id: String,
    pub origin: String,
    pub challenge: Vec<u8>,
    pub user_handle: Vec<u8>,
    pub username: String,
    pub display_name: String,
    pub exclude_credentials: Vec<Vec<u8>>,
    pub request_prf: bool,
}

/// The server-issued options a platform assertion ceremony consumes.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyAssertionChallenge {
    pub rp_id: String,
    pub origin: String,
    pub challenge: Vec<u8>,
    pub allow_credentials: Vec<Vec<u8>>,
    pub request_prf: bool,
}

/// A platform passkey ceremony did not produce a credential result. This is
/// deliberately non-diagnostic: native errors can contain authenticator or
/// credential details that must not enter app logs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PasskeyCeremonyError {
    #[error("passkeys aren't available on this device")]
    Unavailable,
    #[error("the passkey ceremony was cancelled")]
    Cancelled,
    #[error("the passkey ceremony failed")]
    Failed,
}

/// Platform boundary for the account flows. Native app code implements this
/// with AuthenticationServices/Win32 WebAuthn or Linux libfido2; tests can
/// inject a deterministic ceremony without claiming hardware was exercised.
pub trait PasskeyCeremony {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError>;

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError>;
}

/// The user handle a Kanso account uses:
/// `SHA-256("kanso/v1/webauthn-user" || lp(account_id))[0..16]`. Opaque to the
/// authenticator; the account is always found by credential ID.
pub fn user_handle(account_id: &str) -> Vec<u8> {
    let mut input = b"kanso/v1/webauthn-user".to_vec();
    crate::crypto::lp(&mut input, account_id);
    use sha2::{Digest as _, Sha256};
    Sha256::digest(&input)[..16].to_vec()
}

/// A WebAuthn ceremony's result as the platform returned it, base64url-decoded
/// by the app. `prf` is the 32-byte output when the extension was requested and
/// answered; it is never logged or shown.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyRegistration {
    pub credential_id: Vec<u8>,
    pub attestation_object: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub transports: Vec<String>,
    pub prf: Option<[u8; 32]>,
}

/// A passkey assertion as the platform returned it.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyAssertion {
    pub credential_id: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
    pub prf: Option<[u8; 32]>,
}

/// What the account's passkey list shows: no public key, no subject, no PRF output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyInfo {
    pub credential_id: Vec<u8>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub prf: bool,
    pub transports: Vec<String>,
}

impl PasskeyRegistration {
    /// The credential ID as the wire carries it.
    pub fn credential_id_base64url(&self) -> String {
        b64(&self.credential_id)
    }

    /// The identity of the PRF output, which must never reach a log.
    pub fn prf(&self) -> Option<&[u8; 32]> {
        self.prf.as_ref()
    }
}

impl PasskeyAssertion {
    /// The credential ID as the wire carries it.
    pub fn credential_id_base64url(&self) -> String {
        b64(&self.credential_id)
    }

    pub fn prf(&self) -> Option<&[u8; 32]> {
        self.prf.as_ref()
    }
}

impl Drop for PasskeyRegistration {
    fn drop(&mut self) {
        if let Some(prf_out) = self.prf.as_mut() {
            prf_out.zeroize();
        }
    }
}

impl Drop for PasskeyAssertion {
    fn drop(&mut self) {
        if let Some(prf_out) = self.prf.as_mut() {
            prf_out.zeroize();
        }
    }
}

impl std::fmt::Debug for PasskeyRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasskeyRegistration")
            .field("credential_id", &b64(&self.credential_id))
            .field("transports", &self.transports)
            .field("prf", &self.prf.map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for PasskeyAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasskeyAssertion")
            .field("credential_id", &b64(&self.credential_id))
            .field("prf", &self.prf.map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

/// `passkey_wrapped_data_key(prf_out, data_key)`: 98 base64url characters, the
/// data key under the passkey KEK with purpose `0x4B`. `None` without a PRF
/// output: such a credential signs in but cannot unlock a new device.
pub fn passkey_wrapped_key(
    prf_out: &[u8; 32],
    origin: &str,
    account_id: &str,
    credential_id: &[u8],
    data_key: &DataKey,
) -> String {
    let kek = passkey_kek(prf_out, origin, account_id, &b64(credential_id));
    wrap(&kek, WrapPurpose::Passkey, account_id, data_key)
}

/// Opens a passkey wrap. The key ID is compared by the caller, as for a
/// password; a wrap from another credential, account or origin is
/// [`CryptoError::KeyMismatch`].
pub fn open_passkey_wrapped_key(
    prf_out: &[u8; 32],
    origin: &str,
    account_id: &str,
    credential_id: &[u8],
    wrapped: &str,
) -> Result<DataKey, CryptoError> {
    let kek = passkey_kek(prf_out, origin, account_id, &b64(credential_id));
    unwrap(&kek, WrapPurpose::Passkey, account_id, wrapped)
}

/// Opens a returned passkey wrap only when it authenticates and names the
/// account's current data key. A passkey wrap is never interpreted as a
/// password wrap.
pub fn open_passkey_session_key(
    prf_out: &[u8; 32],
    origin: &str,
    account_id: &str,
    credential_id: &[u8],
    wrapped: Option<&str>,
    expected_key_id: &str,
) -> Result<DataKey, CryptoError> {
    let wrapped = wrapped.ok_or(CryptoError::KeyMismatch)?;
    let key = open_passkey_wrapped_key(prf_out, origin, account_id, credential_id, wrapped)?;
    if crate::crypto::key_id(&key) != expected_key_id {
        return Err(CryptoError::KeyMismatch);
    }
    Ok(key)
}

#[cfg(test)]
mod wrap_tests {
    use super::*;
    use crate::crypto::{DataKey, key_id};

    #[test]
    fn prf_wrap_round_trips_and_cannot_move_between_credentials() {
        let prf = [7; 32];
        let key = DataKey::generate();
        let wrapped = passkey_wrapped_key(
            &prf,
            "https://sync.example",
            "a_account",
            b"credential-a",
            &key,
        );
        let opened = open_passkey_session_key(
            &prf,
            "https://sync.example",
            "a_account",
            b"credential-a",
            Some(&wrapped),
            &key_id(&key),
        )
        .unwrap();
        assert_eq!(key_id(&opened), key_id(&key));
        assert_eq!(
            open_passkey_session_key(
                &prf,
                "https://sync.example",
                "a_account",
                b"credential-b",
                Some(&wrapped),
                &key_id(&key)
            ),
            Err(CryptoError::KeyMismatch)
        );
        assert_eq!(
            open_passkey_session_key(
                &prf,
                "https://sync.example",
                "a_account",
                b"credential-a",
                None,
                &key_id(&key)
            ),
            Err(CryptoError::KeyMismatch)
        );
    }
}

// -------------------------------------------------------------- client data

/// The client data JSON a native ceremony sends, as canonical bytes:
/// `{"type","challenge","origin","crossOrigin":false}` with `challenge` in
/// unpadded base64url. The server checks the type, challenge, origin and that
/// cross-origin isn't true.
pub fn client_data_json(kind: &str, challenge: &[u8], origin: &str) -> Zeroizing<Vec<u8>> {
    let json = format!(
        r#"{{"type":"{}","challenge":"{}","origin":"{}","crossOrigin":false}}"#,
        json_escape(kind),
        b64(challenge),
        json_escape(origin)
    );
    Zeroizing::new(json.into_bytes())
}

/// The attestation object for attestation `"none"`:
/// `{"fmt":"none","attStmt":{},"authData":<bytes>}` as canonical CBOR. The
/// authenticator data is the one the authenticator produced; no certificate
/// chain is ever added or checked.
pub fn attestation_object_none(auth_data: &[u8]) -> Vec<u8> {
    // map(3) { "fmt": "none", "attStmt": {}, "authData": bytes }
    let mut out = vec![0xa3];
    cbor_text(&mut out, "fmt");
    cbor_text(&mut out, "none");
    cbor_text(&mut out, "attStmt");
    out.push(0xa0);
    cbor_text(&mut out, "authData");
    cbor_bytes(&mut out, auth_data);
    out
}

/// The raw credential ID in the attestation object's authenticator data, for
/// a local check after a Linux registration. `None` for malformed data.
pub fn attested_credential_id(auth_data: &[u8]) -> Option<&[u8]> {
    if auth_data.len() < 55 || auth_data[32] & 0x40 == 0 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([auth_data[53], auth_data[54]]));
    auth_data.get(55..55 + length)
}

/// The attestation object's `authData` bytes. `None` for anything but the
/// canonical five-key-free map this client writes.
pub fn attestation_auth_data(attestation_object: &[u8]) -> Option<&[u8]> {
    let mut cursor = CborCursor::new(attestation_object);
    if cursor.byte()? != 0xa3 {
        return None;
    }
    cursor.text()?;
    cursor.skip_text()?;
    cursor.text()?;
    if cursor.byte()? != 0xa0 {
        return None;
    }
    let key = cursor.text()?;
    if key != "authData" {
        return None;
    }
    let bytes = cursor.bytes()?;
    cursor.at_end().then_some(bytes)
}

// --------------------------------------------------------------------- CBOR

fn cbor_head(out: &mut Vec<u8>, major: u8, length: usize) {
    let major = major << 5;
    if length < 24 {
        out.push(major | length as u8);
    } else if length <= u8::MAX as usize {
        out.push(major | 24);
        out.push(length as u8);
    } else if length <= u16::MAX as usize {
        out.push(major | 25);
        out.extend_from_slice(&(length as u16).to_be_bytes());
    } else {
        out.push(major | 26);
        out.extend_from_slice(&(length as u32).to_be_bytes());
    }
}

fn cbor_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    cbor_head(out, 2, bytes.len());
    out.extend_from_slice(bytes);
}

fn cbor_text(out: &mut Vec<u8>, text: &str) {
    cbor_head(out, 3, text.len());
    out.extend_from_slice(text.as_bytes());
}

struct CborCursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> CborCursor<'a> {
    fn new(bytes: &'a [u8]) -> CborCursor<'a> {
        CborCursor { bytes, at: 0 }
    }

    fn byte(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.at)?;
        self.at += 1;
        Some(byte)
    }

    fn length(&mut self, info: u8) -> Option<usize> {
        match info {
            n if n < 24 => Some(usize::from(n)),
            24 => Some(usize::from(self.byte()?)),
            25 => {
                let hi = self.byte()?;
                Some(usize::from(u16::from_be_bytes([hi, self.byte()?])))
            }
            26 => {
                let mut bytes = [0u8; 4];
                for byte in &mut bytes {
                    *byte = self.byte()?;
                }
                Some(u32::from_be_bytes(bytes) as usize)
            }
            _ => None,
        }
    }

    fn text(&mut self) -> Option<&'a str> {
        let initial = self.byte()?;
        if initial >> 5 != 3 {
            return None;
        }
        let length = self.length(initial & 0x1f)?;
        let end = self.at.checked_add(length)?;
        let text = std::str::from_utf8(self.bytes.get(self.at..end)?).ok()?;
        self.at = end;
        Some(text)
    }

    fn skip_text(&mut self) -> Option<()> {
        self.text().map(|_| ())
    }

    fn bytes(&mut self) -> Option<&'a [u8]> {
        let initial = self.byte()?;
        if initial >> 5 != 2 {
            return None;
        }
        let length = self.length(initial & 0x1f)?;
        let end = self.at.checked_add(length)?;
        let bytes = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(bytes)
    }

    fn at_end(&self) -> bool {
        self.at == self.bytes.len()
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// The PRF input, as the protocol constant.
pub fn prf_input() -> [u8; 32] {
    PRF_INPUT
}

/// The transports hint string the server stores (`usb`, `nfc`, `internal`, …),
/// space separated and de-duplicated. Only the hints the client saw.
pub fn transports_value(transports: &[String]) -> String {
    let mut seen: Vec<&str> = Vec::new();
    for transport in transports {
        let value = transport.as_str();
        if !value.is_empty()
            && value.bytes().all(|b| b.is_ascii_lowercase() || b == b' ')
            && !seen.contains(&value)
        {
            seen.push(value);
        }
    }
    seen.join(" ")
}

/// Whether text is canonical unpadded base64url, for the fields the wire
/// carries (credential IDs and the like).
pub fn is_base64url(text: &str) -> bool {
    unb64(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_handle_is_sixteen_bytes() {
        let handle = user_handle("a_0123456789abcdef0123456789abcdef");
        assert_eq!(handle.len(), 16);
        assert_eq!(
            handle,
            [
                0x2e, 0xaa, 0x5e, 0xcd, 0xf9, 0xae, 0x9d, 0x60, 0xd7, 0x96, 0x7d, 0xa5, 0xe2, 0x7d,
                0xa7, 0x88
            ]
        );
    }

    #[test]
    fn attestation_object_round_trips() {
        let auth_data = vec![7u8; 120];
        let object = attestation_object_none(&auth_data);
        assert_eq!(attestation_auth_data(&object), Some(auth_data.as_slice()));
        assert!(attestation_auth_data(&object[..object.len() - 1]).is_none());
    }

    #[test]
    fn client_data_has_the_three_fields_the_server_checks() {
        let data = client_data_json("webauthn.create", &[1u8; 32], "https://kanso.example");
        let value: serde_json::Value = serde_json::from_slice(&data).unwrap();
        assert_eq!(value["type"], "webauthn.create");
        assert_eq!(value["challenge"], b64(&[1u8; 32]));
        assert_eq!(value["origin"], "https://kanso.example");
        assert_eq!(value["crossOrigin"], false);
    }

    #[test]
    fn transports_are_filtered_and_deduplicated() {
        assert_eq!(
            transports_value(&[
                "usb".into(),
                "nfc".into(),
                "usb".into(),
                "USB".into(),
                "".into()
            ]),
            "usb nfc"
        );
    }
}
