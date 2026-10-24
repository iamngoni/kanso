//! The Linux security-key path's pure half (docs/ACCOUNTS.md › Passkeys ›
//! Where the ceremony runs › Linux).
//!
//! Linux has no platform passkey UI: the app is the CTAP2 client and uses
//! [libfido2](https://developers.yubico.com/libfido2/) for USB HID and NFC.
//! This module only maps what libfido2 returns to the same WebAuthn shapes the
//! macOS and Windows ceremonies produce. Enumeration and every libfido2 call
//! stay behind the app's platform module, so the server needs no Linux route
//! and this logic is testable without hardware.
//!
//! CTAP2's `hmac-secret` extension returns the same credential-scoped secret
//! the WebAuthn PRF extension does. The app computes the same salt WebAuthn
//! would (`SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`) and passes it as
//! `salt1`; the authenticator's `HMAC-SHA256(credential_secret, salt)` is
//! `prf_out` exactly, used without another hash.

use sha2::{Digest as _, Sha256};

use crate::crypto::PRF_INPUT;
use crate::passkey::{PasskeyAssertion, PasskeyRegistration, attestation_object_none};

/// The label WebAuthn hashes into the salt before the authenticator sees it.
pub const CTAP_PRF_LABEL: &[u8] = b"WebAuthn PRF";

/// `ctap_salt = SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`: the same salt
/// WebAuthn's PRF extension computes, passed to `fido_assert_set_hmac_salt`.
pub fn ctap_salt() -> [u8; 32] {
    let mut input = Vec::with_capacity(CTAP_PRF_LABEL.len() + 1 + PRF_INPUT.len());
    input.extend_from_slice(CTAP_PRF_LABEL);
    input.push(0x00);
    input.extend_from_slice(&PRF_INPUT);
    Sha256::digest(&input).into()
}

/// What `fido_dev_make_cred` gave the app. `auth_data` is libfido2's
/// `fido_cred_authdata_ptr`, already in WebAuthn's authenticator-data layout.
#[derive(Clone, PartialEq, Eq)]
pub struct SecurityKeyRegistration {
    pub credential_id: Vec<u8>,
    pub auth_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub transports: Vec<String>,
    /// The `hmac-secret` assertion that follows registration for a key that
    /// advertises it. libfido2's make-credential binding does not expose
    /// `hmac-secret-mc`, so the app runs one immediate local assertion.
    pub prf: Option<[u8; 32]>,
}

/// What `fido_dev_get_assert` gave the app.
#[derive(Clone, PartialEq, Eq)]
pub struct SecurityKeyAssertion {
    pub credential_id: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
    pub prf: Option<[u8; 32]>,
}

impl SecurityKeyRegistration {
    /// The standard registration message the Worker checks: attestation
    /// `"none"` and the authenticator data libfido2 produced.
    pub fn to_passkey(&self) -> PasskeyRegistration {
        PasskeyRegistration {
            credential_id: self.credential_id.clone(),
            attestation_object: attestation_object_none(&self.auth_data),
            client_data_json: self.client_data_json.clone(),
            transports: self.transports.clone(),
            prf: self.prf,
        }
    }
}

impl SecurityKeyAssertion {
    /// The standard assertion message the Worker checks.
    pub fn to_passkey(&self) -> PasskeyAssertion {
        PasskeyAssertion {
            credential_id: self.credential_id.clone(),
            authenticator_data: self.authenticator_data.clone(),
            client_data_json: self.client_data_json.clone(),
            signature: self.signature.clone(),
            user_handle: self.user_handle.clone(),
            prf: self.prf,
        }
    }
}

/// Whether the credential in `auth_data` is the one the key said it made.
/// The app checks this locally before trusting an immediate `hmac-secret`
/// assertion over the new credential.
pub fn auth_data_has_credential_id(auth_data: &[u8], credential_id: &[u8]) -> bool {
    crate::passkey::attested_credential_id(auth_data) == Some(credential_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctap_salt_matches_the_webauthn_construction() {
        // `SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`, pinned.
        assert_eq!(
            ctap_salt(),
            [
                0x2b, 0x3e, 0xd9, 0x02, 0xb2, 0x5c, 0x43, 0xe8, 0x4d, 0xd6, 0x47, 0xaa, 0x61, 0x18,
                0xa3, 0xc5, 0x30, 0x70, 0xaa, 0xaf, 0x23, 0x04, 0x54, 0x25, 0xf8, 0xa4, 0x1f, 0x1b,
                0xa0, 0xc0, 0x4b, 0x40
            ]
        );
    }

    #[test]
    fn a_registration_becomes_the_same_webauthn_shape() {
        let mut auth_data = vec![0u8; 37];
        auth_data[32] = 0x41;
        let credential = vec![9u8; 32];
        auth_data.extend_from_slice(&[0u8; 16]);
        auth_data.extend_from_slice(&(credential.len() as u16).to_be_bytes());
        auth_data.extend_from_slice(&credential);
        auth_data.extend_from_slice(&[0xa0]); // an empty COSE key map placeholder

        let registration = SecurityKeyRegistration {
            credential_id: credential.clone(),
            auth_data: auth_data.clone(),
            client_data_json: b"{}".to_vec(),
            transports: vec!["usb".into()],
            prf: Some([3u8; 32]),
        };
        let passkey = registration.to_passkey();
        assert_eq!(passkey.credential_id, credential);
        assert_eq!(passkey.prf, Some([3u8; 32]));
        assert_eq!(
            crate::passkey::attestation_auth_data(&passkey.attestation_object),
            Some(auth_data.as_slice())
        );
        assert!(auth_data_has_credential_id(
            &auth_data,
            &passkey.credential_id
        ));
        assert!(!auth_data_has_credential_id(&auth_data, &[8u8; 32]));
    }
}
