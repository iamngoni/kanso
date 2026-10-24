//! The Linux security-key path (docs/ACCOUNTS.md › Passkeys › Where the
//! ceremony runs › Linux).
//!
//! Linux has no platform passkey UI. The app is the CTAP2 client through
//! [libfido2](https://developers.yubico.com/libfido2/) 1.12 or newer for USB HID and
//! NFC security keys. This crate keeps the pure half here — the CTAP2
//! `hmac-secret` to WebAuthn PRF mapping, the standard `clientDataJSON` and
//! `attestationObject` the Worker checks, and the returned shapes — and puts
//! every libfido2 call behind the `linux` module.
//!
//! `hmac-secret` returns the same credential-scoped secret WebAuthn's PRF
//! extension does. The app passes
//! `ctap_salt = SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)` as `salt1`; the
//! authenticator's `HMAC-SHA256(credential_secret, ctap_salt)` is `prf_out`
//! exactly, used without another hash. A key without `hmac-secret` registers
//! without a wrap: it signs in, but a new device needs approval or recovery.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest as _, Sha256};

/// The fixed Kanso PRF input, the same 32-byte constant every passkey uses.
pub const PRF_INPUT: [u8; 32] = [
    0x98, 0x43, 0x20, 0xbf, 0xab, 0xbd, 0x34, 0x26, 0x97, 0xaf, 0x83, 0x64, 0x7d, 0xc7, 0x74, 0xe3,
    0x07, 0xf9, 0x8e, 0x62, 0xec, 0xf2, 0x7d, 0xd5, 0xd8, 0x4f, 0x8f, 0x00, 0x6d, 0x86, 0xfd, 0x59,
];

/// The label WebAuthn hashes into the salt before the authenticator sees it.
pub const CTAP_PRF_LABEL: &[u8] = b"WebAuthn PRF";

/// `SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`, passed to
/// `fido_assert_set_hmac_salt` as `salt1`. The authenticator's answer is
/// `prf_out`; it is not hashed again.
pub fn ctap_salt() -> [u8; 32] {
    let mut input = Vec::with_capacity(CTAP_PRF_LABEL.len() + 1 + PRF_INPUT.len());
    input.extend_from_slice(CTAP_PRF_LABEL);
    input.push(0x00);
    input.extend_from_slice(&PRF_INPUT);
    Sha256::digest(&input).into()
}

/// A security key libfido2 enumerated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityKeyDevice {
    /// The libfido2 path (`/dev/hidrawN` on Linux).
    pub path: String,
    /// The USB vendor ID as four lowercase hex digits.
    pub vendor: String,
    /// The authenticator's product string, when libfido2 reports one.
    pub product: String,
    /// Whether this key advertises `hmac-secret` capabilities. libfido2
    /// reports capability flags; the app still checks the ceremony's answer.
    pub hmac_secret: bool,
}

/// What a registration ceremony needs, in CTAP2 terms.
#[derive(Clone, Debug)]
pub struct RegistrationRequest {
    pub rp_id: String,
    pub origin: String,
    pub challenge: Vec<u8>,
    pub user_handle: Vec<u8>,
    pub username: String,
    pub display_name: String,
    /// Enable `hmac-secret` on the credential so a following assertion can
    /// return the PRF output. The make-credential binding exposes no
    /// `hmac-secret-mc` output.
    pub hmac_secret: bool,
    pub pin: Option<zeroize::Zeroizing<String>>,
}

/// What an assertion ceremony needs.
#[derive(Clone, Debug)]
pub struct AssertionRequest {
    pub rp_id: String,
    pub origin: String,
    pub challenge: Vec<u8>,
    /// Empty asks the key to pick a discoverable credential.
    pub allow_credentials: Vec<Vec<u8>>,
    /// Ask for `hmac-secret` (the PRF output).
    pub hmac_secret: bool,
    pub pin: Option<zeroize::Zeroizing<String>>,
}

/// The raw authenticator data libfido2 produced, before the app wraps it in
/// the WebAuthn registration message.
#[derive(Clone, PartialEq, Eq)]
pub struct Registration {
    pub credential_id: Vec<u8>,
    pub auth_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub transports: Vec<String>,
}

/// The raw assertion libfido2 produced.
#[derive(Clone, PartialEq, Eq)]
pub struct Assertion {
    pub credential_id: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
    pub hmac_secret: Option<[u8; 32]>,
}

impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registration")
            .field(
                "credential_id",
                &URL_SAFE_NO_PAD.encode(&self.credential_id),
            )
            .field("transports", &self.transports)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Assertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assertion")
            .field(
                "credential_id",
                &URL_SAFE_NO_PAD.encode(&self.credential_id),
            )
            .field("prf", &self.hmac_secret.map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

/// Why a security-key ceremony didn't finish.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FidoError {
    /// This build has no libfido2 (not Linux), or no key matched.
    #[error("no compatible FIDO2 security key")]
    NoDevice,
    /// The key asked for a PIN this call didn't carry.
    #[error("the security key needs its PIN")]
    PinRequired,
    /// The key or libfido2 refused the ceremony.
    #[error("the security key refused: {0}")]
    Refused(String),
    /// The answer was missing a field the WebAuthn message needs.
    #[error("the security key returned an incomplete answer")]
    Incomplete,
    /// The request couldn't be made (an empty RP ID, a long name).
    #[error("the security-key request is not valid")]
    Invalid,
}

/// Enumerates attached USB HID and NFC security keys. Empty where libfido2
/// isn't available; the app then shows no security-key button.
pub fn enumerate() -> Vec<SecurityKeyDevice> {
    platform::enumerate()
}

/// Registers a credential on `device`, with `rk=true` and user verification
/// required. libfido2 carries the WebAuthn ceremony's RP ID, user handle and
/// client data hash.
pub fn register(
    device: &SecurityKeyDevice,
    request: &RegistrationRequest,
) -> Result<Registration, FidoError> {
    validate(request.rp_id.is_empty(), &request.username)?;
    platform::register(device, request)
}

/// Asserts with `device`. The returned `hmac_secret`, when requested and the
/// key supports it, is `prf_out`.
pub fn assert(
    device: &SecurityKeyDevice,
    request: &AssertionRequest,
) -> Result<Assertion, FidoError> {
    validate(request.rp_id.is_empty(), "")?;
    platform::assert(device, request)
}

fn validate(empty_rp: bool, username: &str) -> Result<(), FidoError> {
    if empty_rp || username.contains('\0') {
        return Err(FidoError::Invalid);
    }
    Ok(())
}

// ------------------------------------------------------------- WebAuthn shapes

/// The standard `clientDataJSON` the app sends with a CTAP2 ceremony. The
/// Worker checks the type, challenge, origin and that cross-origin isn't true.
pub fn client_data_json(kind: &str, challenge: &[u8], origin: &str) -> Vec<u8> {
    format!(
        r#"{{"type":"{}","challenge":"{}","origin":"{}","crossOrigin":false}}"#,
        json_escape(kind),
        URL_SAFE_NO_PAD.encode(challenge),
        json_escape(origin)
    )
    .into_bytes()
}

/// The attestation object for attestation `"none"`:
/// `{"fmt":"none","attStmt":{},"authData":<bytes>}` as canonical CBOR.
pub fn attestation_object_none(auth_data: &[u8]) -> Vec<u8> {
    let mut out = vec![0xa3];
    cbor_text(&mut out, "fmt");
    cbor_text(&mut out, "none");
    cbor_text(&mut out, "attStmt");
    out.push(0xa0);
    cbor_text(&mut out, "authData");
    cbor_bytes(&mut out, auth_data);
    out
}

/// The credential ID inside authenticator data with the AT flag set.
pub fn attested_credential_id(auth_data: &[u8]) -> Option<&[u8]> {
    if auth_data.len() < 55 || auth_data[32] & 0x40 == 0 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([auth_data[53], auth_data[54]]));
    auth_data.get(55..55 + length)
}

/// The transports a security key reports, as the server stores them. USB HID
/// is the required transport; NFC appears where libfido2 and the host expose
/// a reader.
pub fn transports(usb: bool, nfc: bool) -> Vec<String> {
    let mut out = Vec::new();
    if usb {
        out.push("usb".to_owned());
    }
    if nfc {
        out.push("nfc".to_owned());
    }
    out
}

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

// -------------------------------------------------------------------- Linux

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::ffi::{CStr, CString, c_char, c_int, c_void};

    type FidoDevInfo = c_void;
    type FidoDev = c_void;
    type FidoCred = c_void;
    type FidoAssert = c_void;
    type FidoCborInfo = c_void;

    #[link(name = "fido2")]
    unsafe extern "C" {
        fn fido_dev_info_new(n: usize) -> *mut FidoDevInfo;
        fn fido_dev_info_manifest(list: *mut FidoDevInfo, max: usize, out: *mut usize) -> c_int;
        fn fido_dev_info_free(list: *mut *mut FidoDevInfo, n: usize);
        fn fido_dev_info_ptr(list: *const FidoDevInfo, i: usize) -> *const FidoDevInfo;
        fn fido_dev_info_path(info: *const FidoDevInfo) -> *const c_char;
        fn fido_dev_info_vendor(info: *const FidoDevInfo) -> i16;
        fn fido_dev_info_product_string(info: *const FidoDevInfo) -> *const c_char;

        fn fido_dev_new() -> *mut FidoDev;
        fn fido_dev_free(dev: *mut *mut FidoDev);
        fn fido_dev_open(dev: *mut FidoDev, path: *const c_char) -> c_int;
        fn fido_dev_close(dev: *mut FidoDev) -> c_int;

        fn fido_cred_new() -> *mut FidoCred;
        fn fido_cred_free(cred: *mut *mut FidoCred);
        fn fido_cred_set_type(cred: *mut FidoCred, alg: c_int) -> c_int;
        fn fido_cred_set_rp(cred: *mut FidoCred, id: *const c_char, name: *const c_char) -> c_int;
        fn fido_cred_set_user(
            cred: *mut FidoCred,
            id: *const u8,
            len: usize,
            name: *const c_char,
            display_name: *const c_char,
            icon: *const c_char,
        ) -> c_int;
        fn fido_cred_set_clientdata_hash(cred: *mut FidoCred, hash: *const u8, len: usize)
        -> c_int;
        fn fido_cred_set_rk(cred: *mut FidoCred, value: c_int) -> c_int;
        fn fido_cred_set_uv(cred: *mut FidoCred, value: c_int) -> c_int;
        fn fido_cred_set_extensions(cred: *mut FidoCred, flags: c_int) -> c_int;
        fn fido_cred_set_fmt(cred: *mut FidoCred, fmt: *const c_char) -> c_int;
        #[cfg(test)]
        fn fido_cred_authdata_ptr(cred: *const FidoCred) -> *const u8;
        #[cfg(test)]
        fn fido_cred_authdata_len(cred: *const FidoCred) -> usize;
        fn fido_cred_authdata_raw_ptr(cred: *const FidoCred) -> *const u8;
        fn fido_cred_authdata_raw_len(cred: *const FidoCred) -> usize;
        #[cfg(test)]
        fn fido_cred_set_authdata_raw(cred: *mut FidoCred, ptr: *const u8, len: usize) -> c_int;
        fn fido_cred_id_ptr(cred: *const FidoCred) -> *const u8;
        fn fido_cred_id_len(cred: *const FidoCred) -> usize;
        fn fido_dev_make_cred(dev: *mut FidoDev, cred: *mut FidoCred, pin: *const c_char) -> c_int;

        fn fido_assert_new() -> *mut FidoAssert;
        fn fido_assert_free(assert: *mut *mut FidoAssert);
        fn fido_assert_set_rp(assert: *mut FidoAssert, id: *const c_char) -> c_int;
        fn fido_assert_set_clientdata_hash(
            assert: *mut FidoAssert,
            hash: *const u8,
            len: usize,
        ) -> c_int;
        fn fido_assert_allow_cred(assert: *mut FidoAssert, id: *const u8, len: usize) -> c_int;
        fn fido_assert_set_extensions(assert: *mut FidoAssert, flags: c_int) -> c_int;
        fn fido_assert_set_hmac_salt(assert: *mut FidoAssert, salt: *const u8, len: usize)
        -> c_int;
        fn fido_assert_set_uv(assert: *mut FidoAssert, value: c_int) -> c_int;
        fn fido_assert_set_up(assert: *mut FidoAssert, value: c_int) -> c_int;
        fn fido_assert_count(assert: *const FidoAssert) -> usize;
        fn fido_assert_authdata_ptr(assert: *const FidoAssert, i: usize) -> *const u8;
        fn fido_assert_authdata_len(assert: *const FidoAssert, i: usize) -> usize;
        #[cfg(test)]
        fn fido_assert_set_count(assert: *mut FidoAssert, n: usize) -> c_int;
        #[cfg(test)]
        fn fido_assert_set_authdata_raw(
            assert: *mut FidoAssert,
            i: usize,
            ptr: *const u8,
            len: usize,
        ) -> c_int;
        fn fido_assert_sig_ptr(assert: *const FidoAssert, i: usize) -> *const u8;
        fn fido_assert_sig_len(assert: *const FidoAssert, i: usize) -> usize;
        fn fido_assert_id_ptr(assert: *const FidoAssert, i: usize) -> *const u8;
        fn fido_assert_id_len(assert: *const FidoAssert, i: usize) -> usize;
        fn fido_assert_user_id_ptr(assert: *const FidoAssert, i: usize) -> *const u8;
        fn fido_assert_user_id_len(assert: *const FidoAssert, i: usize) -> usize;
        fn fido_assert_hmac_secret_ptr(assert: *const FidoAssert, i: usize) -> *const u8;
        fn fido_assert_hmac_secret_len(assert: *const FidoAssert, i: usize) -> usize;
        fn fido_dev_get_assert(
            dev: *mut FidoDev,
            assert: *mut FidoAssert,
            pin: *const c_char,
        ) -> c_int;

        fn fido_cbor_info_new() -> *mut FidoCborInfo;
        fn fido_cbor_info_free(info: *mut *mut FidoCborInfo);
        fn fido_dev_get_cbor_info(dev: *mut FidoDev, info: *mut FidoCborInfo) -> c_int;
        fn fido_cbor_info_extensions_ptr(info: *const FidoCborInfo) -> *mut *mut c_char;
        fn fido_cbor_info_extensions_len(info: *const FidoCborInfo) -> usize;

        fn fido_strerr(code: c_int) -> *const c_char;
    }

    const FIDO_OPT_TRUE: c_int = 1;
    // libfido2 <fido/param.h>: FIDO_EXT_HMAC_SECRET.
    const FIDO_EXT_HMAC_SECRET: c_int = 0x01;
    // libfido2 <fido/err.h>: FIDO_ERR_UNSUPPORTED_EXTENSION.
    const FIDO_ERR_UNSUPPORTED_EXTENSION: c_int = 0x16;
    const COSE_ES256: c_int = -7;
    const MAX_DEVICES: usize = 8;

    fn text(ptr: *const c_char) -> String {
        if ptr.is_null() {
            return String::new();
        }
        // SAFETY: libfido2 returns a NUL-terminated string valid for the call.
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }

    fn error(code: c_int) -> FidoError {
        // FIDO_ERR_PIN_REQUIRED is 0x36 (54) and FIDO_ERR_PIN_INVALID 0x31.
        if code == 0x36 || code == 0x31 {
            return FidoError::PinRequired;
        }
        let message = text(unsafe { fido_strerr(code) });
        if message.is_empty() {
            FidoError::Refused(format!("error {code}"))
        } else {
            FidoError::Refused(message)
        }
    }

    struct Device(*mut FidoDev);
    impl Drop for Device {
        fn drop(&mut self) {
            unsafe {
                fido_dev_close(self.0);
                let mut pointer = self.0;
                fido_dev_free(&mut pointer);
            }
        }
    }

    struct Cred(*mut FidoCred);
    impl Drop for Cred {
        fn drop(&mut self) {
            unsafe {
                let mut pointer = self.0;
                fido_cred_free(&mut pointer);
            }
        }
    }

    struct Assert(*mut FidoAssert);
    impl Drop for Assert {
        fn drop(&mut self) {
            unsafe {
                let mut pointer = self.0;
                fido_assert_free(&mut pointer);
            }
        }
    }

    struct CborInfo(*mut FidoCborInfo);
    impl Drop for CborInfo {
        fn drop(&mut self) {
            unsafe {
                let mut pointer = self.0;
                fido_cbor_info_free(&mut pointer);
            }
        }
    }

    /// Whether the opened key advertises the CTAP2 `hmac-secret` extension.
    /// The extension list is authoritative for this key; a key that omits it
    /// can still register and sign in, just without a PRF wrap.
    fn supports_hmac_secret(device: &Device) -> bool {
        // SAFETY: the CBOR-info object is owned by `CborInfo`; the returned
        // string array remains valid until that object is freed.
        unsafe {
            let raw = fido_cbor_info_new();
            if raw.is_null() {
                return false;
            }
            let info = CborInfo(raw);
            if fido_dev_get_cbor_info(device.0, info.0) != 0 {
                return false;
            }
            let extensions = fido_cbor_info_extensions_ptr(info.0);
            if extensions.is_null() {
                return false;
            }
            let count = fido_cbor_info_extensions_len(info.0);
            (0..count).any(|index| {
                let extension = *extensions.add(index);
                !extension.is_null() && CStr::from_ptr(extension).to_bytes() == b"hmac-secret"
            })
        }
    }

    pub(super) fn enumerate() -> Vec<SecurityKeyDevice> {
        let mut out = Vec::new();
        // SAFETY: libfido2 allocates and frees its own device-info list.
        unsafe {
            let list = fido_dev_info_new(MAX_DEVICES);
            if list.is_null() {
                return out;
            }
            let mut found = 0usize;
            if fido_dev_info_manifest(list, MAX_DEVICES, &mut found) == 0 {
                for i in 0..found {
                    let info = fido_dev_info_ptr(list, i);
                    if info.is_null() {
                        continue;
                    }
                    let path = text(fido_dev_info_path(info));
                    if path.is_empty() {
                        continue;
                    }
                    let hmac_secret = open_path(&path)
                        .map(|device| supports_hmac_secret(&device))
                        .unwrap_or(false);
                    out.push(SecurityKeyDevice {
                        path,
                        vendor: format!("{:04x}", fido_dev_info_vendor(info)),
                        product: text(fido_dev_info_product_string(info)),
                        hmac_secret,
                    });
                }
            }
            let mut pointer = list;
            fido_dev_info_free(&mut pointer, MAX_DEVICES);
        }
        out
    }

    fn open_path(path: &str) -> Result<Device, FidoError> {
        let path = CString::new(path).map_err(|_| FidoError::Invalid)?;
        // SAFETY: libfido2 owns the device until `Device` drops it.
        unsafe {
            let raw = fido_dev_new();
            if raw.is_null() {
                return Err(FidoError::NoDevice);
            }
            let dev = Device(raw);
            let code = fido_dev_open(dev.0, path.as_ptr());
            if code != 0 {
                return Err(error(code));
            }
            Ok(dev)
        }
    }

    fn open(device: &SecurityKeyDevice) -> Result<Device, FidoError> {
        open_path(&device.path)
    }

    pub(super) fn register(
        device: &SecurityKeyDevice,
        request: &RegistrationRequest,
    ) -> Result<Registration, FidoError> {
        let dev = open(device)?;
        let rp = CString::new(request.rp_id.clone()).map_err(|_| FidoError::Invalid)?;
        let name = CString::new(request.username.clone()).map_err(|_| FidoError::Invalid)?;
        let display = CString::new(request.display_name.clone()).map_err(|_| FidoError::Invalid)?;
        let client_data =
            super::client_data_json("webauthn.create", &request.challenge, &request.origin);
        let hash = Sha256::digest(&client_data);
        let pin = request
            .pin
            .as_deref()
            .map(|pin| CString::new(pin.as_bytes()).map_err(|_| FidoError::Invalid))
            .transpose()?;
        let use_hmac_secret = request.hmac_secret && supports_hmac_secret(&dev);
        // SAFETY: every pointer is valid for the call; libfido2 owns the credential.
        unsafe {
            let raw = fido_cred_new();
            if raw.is_null() {
                return Err(FidoError::Refused(
                    "could not allocate a credential".to_owned(),
                ));
            }
            let cred = Cred(raw);
            let mut code = fido_cred_set_type(cred.0, COSE_ES256);
            if code == 0 {
                code = fido_cred_set_rp(cred.0, rp.as_ptr(), rp.as_ptr());
            }
            if code == 0 {
                code = fido_cred_set_user(
                    cred.0,
                    request.user_handle.as_ptr(),
                    request.user_handle.len(),
                    name.as_ptr(),
                    display.as_ptr(),
                    std::ptr::null(),
                );
            }
            if code == 0 {
                code = fido_cred_set_clientdata_hash(cred.0, hash.as_ptr(), hash.len());
            }
            if code == 0 {
                code = fido_cred_set_rk(cred.0, FIDO_OPT_TRUE);
            }
            if code == 0 {
                code = fido_cred_set_uv(cred.0, FIDO_OPT_TRUE);
            }
            if code == 0 && use_hmac_secret {
                code = fido_cred_set_extensions(cred.0, FIDO_EXT_HMAC_SECRET);
            }
            if code == 0 {
                code = fido_cred_set_fmt(cred.0, c"none".as_ptr());
            }
            if code == 0 {
                code = fido_dev_make_cred(
                    dev.0,
                    cred.0,
                    pin.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
                );
            }
            if code != 0 {
                return Err(error(code));
            }
            // WebAuthn uses the raw authenticator data. `*_authdata_ptr`
            // returns the CBOR byte string that wraps these bytes in the
            // CTAP2 response, so it must not be put in `attestationObject`.
            let auth_data = registration_auth_data(cred.0);
            let credential_id = slice(fido_cred_id_ptr(cred.0), fido_cred_id_len(cred.0));
            if auth_data.is_empty() || credential_id.is_empty() {
                return Err(FidoError::Incomplete);
            }
            Ok(Registration {
                credential_id,
                auth_data,
                client_data_json: client_data,
                transports: super::transports(true, false),
            })
        }
    }

    pub(super) fn assert(
        device: &SecurityKeyDevice,
        request: &AssertionRequest,
    ) -> Result<Assertion, FidoError> {
        let dev = open(device)?;
        let rp = CString::new(request.rp_id.clone()).map_err(|_| FidoError::Invalid)?;
        let client_data =
            super::client_data_json("webauthn.get", &request.challenge, &request.origin);
        let hash = Sha256::digest(&client_data);
        let salt = super::ctap_salt();
        let pin = request
            .pin
            .as_deref()
            .map(|pin| CString::new(pin.as_bytes()).map_err(|_| FidoError::Invalid))
            .transpose()?;
        let use_hmac_secret = request.hmac_secret && supports_hmac_secret(&dev);
        // SAFETY: every pointer is valid for the call; libfido2 owns the assertion.
        unsafe {
            let raw = fido_assert_new();
            if raw.is_null() {
                return Err(FidoError::Refused(
                    "could not allocate an assertion".to_owned(),
                ));
            }
            let assertion = Assert(raw);
            let mut code = fido_assert_set_rp(assertion.0, rp.as_ptr());
            if code == 0 {
                code = fido_assert_set_clientdata_hash(assertion.0, hash.as_ptr(), hash.len());
            }
            for credential in &request.allow_credentials {
                if code != 0 {
                    break;
                }
                code = fido_assert_allow_cred(assertion.0, credential.as_ptr(), credential.len());
            }
            if code == 0 {
                code = fido_assert_set_uv(assertion.0, FIDO_OPT_TRUE);
            }
            if code == 0 {
                code = fido_assert_set_up(assertion.0, FIDO_OPT_TRUE);
            }
            if code == 0 && use_hmac_secret {
                code = fido_assert_set_extensions(assertion.0, FIDO_EXT_HMAC_SECRET);
            }
            if code == 0 && use_hmac_secret {
                code = fido_assert_set_hmac_salt(assertion.0, salt.as_ptr(), salt.len());
            }
            if code == 0 {
                code = fido_dev_get_assert(
                    dev.0,
                    assertion.0,
                    pin.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
                );
            }
            // A key can advertise hmac-secret but still hold an older
            // credential that was created before the extension was enabled.
            // That credential must remain usable for sign-in, so retry once
            // without the extension and accept a session without a PRF.
            if code == FIDO_ERR_UNSUPPORTED_EXTENSION && use_hmac_secret {
                code = fido_assert_set_extensions(assertion.0, 0);
                if code == 0 {
                    code = fido_dev_get_assert(
                        dev.0,
                        assertion.0,
                        pin.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
                    );
                }
            }
            if code != 0 {
                return Err(error(code));
            }
            if fido_assert_count(assertion.0) == 0 {
                return Err(FidoError::NoDevice);
            }
            // The signature covers the raw authenticator data, not the CBOR
            // byte-string wrapper around it in the CTAP2 response.
            let authenticator_data = assertion_auth_data(assertion.0);
            let signature = slice(
                fido_assert_sig_ptr(assertion.0, 0),
                fido_assert_sig_len(assertion.0, 0),
            );
            let credential_id = slice(
                fido_assert_id_ptr(assertion.0, 0),
                fido_assert_id_len(assertion.0, 0),
            );
            let user = slice(
                fido_assert_user_id_ptr(assertion.0, 0),
                fido_assert_user_id_len(assertion.0, 0),
            );
            let secret = slice_opt(
                fido_assert_hmac_secret_ptr(assertion.0, 0),
                fido_assert_hmac_secret_len(assertion.0, 0),
            );
            if authenticator_data.is_empty() || signature.is_empty() || credential_id.is_empty() {
                return Err(FidoError::Incomplete);
            }
            let hmac_secret = match secret {
                Some(bytes) if bytes.len() == 32 => {
                    let mut out = [0u8; 32];
                    out.copy_from_slice(&bytes);
                    Some(out)
                }
                _ => None,
            };
            Ok(Assertion {
                credential_id,
                authenticator_data,
                client_data_json: client_data,
                signature,
                user_handle: (!user.is_empty()).then_some(user),
                hmac_secret,
            })
        }
    }

    unsafe fn slice(ptr: *const u8, len: usize) -> Vec<u8> {
        unsafe { slice_opt(ptr, len) }.unwrap_or_default()
    }

    pub(super) unsafe fn registration_auth_data(cred: *const FidoCred) -> Vec<u8> {
        unsafe {
            slice(
                fido_cred_authdata_raw_ptr(cred),
                fido_cred_authdata_raw_len(cred),
            )
        }
    }

    pub(super) unsafe fn assertion_auth_data(assert: *const FidoAssert) -> Vec<u8> {
        unsafe {
            // libfido2 1.12 has no `fido_assert_authdata_raw_*` getter. Its
            // getter returns the CBOR byte string that wraps the raw bytes.
            let encoded = slice(
                fido_assert_authdata_ptr(assert, 0),
                fido_assert_authdata_len(assert, 0),
            );
            cbor_byte_string(&encoded)
                .map(ToOwned::to_owned)
                .unwrap_or_default()
        }
    }

    /// Decodes one definite-length CBOR byte string. libfido2 wraps assertion
    /// authenticator data with `cbor_encode_blob`, so the encoded value has no
    /// trailing bytes and is never indefinite-length.
    pub(super) fn cbor_byte_string(encoded: &[u8]) -> Option<&[u8]> {
        let (length, header): (usize, usize) = match *encoded.first()? {
            major if major & 0xe0 != 0x40 => return None,
            major @ 0x40..=0x57 => (usize::from(major & 0x1f), 1),
            0x58 => (usize::from(*encoded.get(1)?), 2),
            0x59 => (
                usize::from(u16::from_be_bytes([*encoded.get(1)?, *encoded.get(2)?])),
                3,
            ),
            0x5a => (
                u32::from_be_bytes([
                    *encoded.get(1)?,
                    *encoded.get(2)?,
                    *encoded.get(3)?,
                    *encoded.get(4)?,
                ]) as usize,
                5,
            ),
            // libfido2's encoder never emits 64-bit lengths, and an
            // indefinite-length string would have no single payload to read.
            _ => return None,
        };
        let end = header.checked_add(length)?;
        (end == encoded.len()).then(|| &encoded[header..end])
    }

    unsafe fn slice_opt(ptr: *const u8, len: usize) -> Option<Vec<u8>> {
        if ptr.is_null() || len == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(ptr, len).to_vec() })
        }
    }

    #[cfg(test)]
    pub(super) fn raw_authdata_regression() {
        let registration = test_registration_auth_data();
        // SAFETY: the test owns both objects and frees them via their RAII
        // wrappers. The bytes stay alive for the duration of the calls.
        unsafe {
            let cred = Cred(fido_cred_new());
            assert!(!cred.0.is_null());
            assert_eq!(fido_cred_set_type(cred.0, COSE_ES256), 0);
            assert_eq!(
                fido_cred_set_authdata_raw(cred.0, registration.as_ptr(), registration.len()),
                0
            );
            assert_eq!(registration_auth_data(cred.0), registration);
            let wrapped = slice(
                fido_cred_authdata_ptr(cred.0),
                fido_cred_authdata_len(cred.0),
            );
            assert_ne!(
                wrapped, registration,
                "registration authData was CBOR-wrapped"
            );

            let raw_assertion = test_assertion_auth_data();
            let assertion = Assert(fido_assert_new());
            assert!(!assertion.0.is_null());
            assert_eq!(fido_assert_set_count(assertion.0, 1), 0);
            assert_eq!(
                fido_assert_set_authdata_raw(
                    assertion.0,
                    0,
                    raw_assertion.as_ptr(),
                    raw_assertion.len(),
                ),
                0
            );
            assert_eq!(assertion_auth_data(assertion.0), raw_assertion);
            let wrapped = slice(
                fido_assert_authdata_ptr(assertion.0, 0),
                fido_assert_authdata_len(assertion.0, 0),
            );
            assert_ne!(
                wrapped, raw_assertion,
                "assertion authData was CBOR-wrapped"
            );
            assert_eq!(cbor_byte_string(&wrapped), Some(raw_assertion.as_slice()));
        }
    }

    #[cfg(test)]
    fn test_registration_auth_data() -> Vec<u8> {
        let mut out = vec![0u8; 32];
        out.push(0x41); // UP | AT
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&[0u8; 16]); // AAGUID
        out.extend_from_slice(&4u16.to_be_bytes());
        out.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]); // credential ID
        // COSE_Key for ES256: {1: 2, 3: -7, -1: 1, -2: x, -3: y}.
        out.extend_from_slice(&[0xa5, 0x01, 0x02, 0x03, 0x26, 0x20, 0x01]);
        for (key, coordinate) in [(0x21, [0x55u8; 32]), (0x22, [0x66u8; 32])] {
            out.push(key);
            out.extend_from_slice(&[0x58, 0x20]);
            out.extend_from_slice(&coordinate);
        }
        out
    }

    #[cfg(test)]
    fn test_assertion_auth_data() -> Vec<u8> {
        let mut out = vec![0u8; 32];
        out.push(0x01); // UP
        out.extend_from_slice(&0u32.to_be_bytes());
        out
    }

    #[cfg(test)]
    pub(super) fn cbor_byte_string_regression() {
        for length in [0usize, 23, 24, 255, 256, 65_536] {
            let payload = vec![0xa5; length];
            let mut encoded = Vec::new();
            cbor_bytes(&mut encoded, &payload);
            assert_eq!(cbor_byte_string(&encoded), Some(payload.as_slice()));
        }
        assert_eq!(cbor_byte_string(&[]), None);
        assert_eq!(cbor_byte_string(&[0x40]), Some([].as_slice()));
        assert_eq!(cbor_byte_string(&[0x58, 0x02, 0xaa]), None);
        assert_eq!(cbor_byte_string(&[0x58, 0x01, 0xaa, 0xbb]), None);
        assert_eq!(cbor_byte_string(&[0x5f, 0x40, 0xff]), None);
        assert_eq!(cbor_byte_string(&[0x81, 0x00]), None);
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::*;

    pub(super) fn enumerate() -> Vec<SecurityKeyDevice> {
        Vec::new()
    }

    pub(super) fn register(
        _device: &SecurityKeyDevice,
        _request: &RegistrationRequest,
    ) -> Result<Registration, FidoError> {
        Err(FidoError::NoDevice)
    }

    pub(super) fn assert(
        _device: &SecurityKeyDevice,
        _request: &AssertionRequest,
    ) -> Result<Assertion, FidoError> {
        Err(FidoError::NoDevice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctap_salt_is_the_webauthn_hash_of_the_prf_input() {
        let expected: [u8; 32] =
            Sha256::digest([CTAP_PRF_LABEL, &[0x00], PRF_INPUT.as_slice()].concat()).into();
        assert_eq!(ctap_salt(), expected);
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
    fn attestation_object_round_trips_the_auth_data() {
        let auth_data = vec![7u8; 100];
        let object = attestation_object_none(&auth_data);
        assert_eq!(object[0], 0xa3);
        assert!(
            object
                .windows(auth_data.len())
                .any(|window| window == auth_data.as_slice())
        );
    }

    // -------------------------------------------------- software authenticator

    use hmac::{Hmac, KeyInit, Mac};
    use ring::rand::SystemRandom;
    use ring::signature::{ED25519, Ed25519KeyPair, KeyPair as _, UnparsedPublicKey};

    /// A software authenticator: the fields a libfido2 key returns, without
    /// hardware. It registers an Ed25519 credential, signs assertions with it
    /// and answers `hmac-secret` with HMAC-SHA256(credential secret, salt).
    struct FakeKey {
        pair: Ed25519KeyPair,
        credential_id: Vec<u8>,
        secret: [u8; 32],
        rp_id: String,
        origin: String,
        sign_count: u32,
    }

    impl FakeKey {
        fn new(rp_id: &str, origin: &str) -> FakeKey {
            let rng = SystemRandom::new();
            let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("key generation");
            let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("pkcs8");
            FakeKey {
                pair,
                credential_id: vec![0x5a; 32],
                secret: [0x11; 32],
                rp_id: rp_id.to_owned(),
                origin: origin.to_owned(),
                sign_count: 1,
            }
        }

        fn registration(&self, challenge: &[u8], _user_handle: &[u8]) -> Registration {
            let client_data = client_data_json("webauthn.create", challenge, &self.origin);
            let mut auth_data = Vec::new();
            auth_data.extend_from_slice(&Sha256::digest(self.rp_id.as_bytes()));
            auth_data.push(0x45); // UP | UV | AT
            auth_data.extend_from_slice(&0u32.to_be_bytes());
            auth_data.extend_from_slice(&[0u8; 16]); // AAGUID
            auth_data.extend_from_slice(&(self.credential_id.len() as u16).to_be_bytes());
            auth_data.extend_from_slice(&self.credential_id);
            // COSE EdDSA: {1: 1, 3: -8, -1: 6, -2: public key}
            let mut cose = vec![0xa4];
            cose.extend_from_slice(&[0x01, 0x01, 0x03, 0x27, 0x20, 0x06]);
            cose.push(0x21);
            cbor_bytes(&mut cose, self.pair.public_key().as_ref());
            auth_data.extend_from_slice(&cose);
            Registration {
                credential_id: self.credential_id.clone(),
                auth_data,
                client_data_json: client_data,
                transports: transports(true, false),
            }
        }

        fn assertion(&self, challenge: &[u8], hmac: bool) -> Assertion {
            let client_data = client_data_json("webauthn.get", challenge, &self.origin);
            let mut auth_data = Vec::new();
            auth_data.extend_from_slice(&Sha256::digest(self.rp_id.as_bytes()));
            auth_data.push(0x05); // UP | UV
            auth_data.extend_from_slice(&self.sign_count.to_be_bytes());
            let mut signed = auth_data.clone();
            signed.extend_from_slice(&Sha256::digest(&client_data));
            let signature = self.pair.sign(&signed).as_ref().to_vec();
            let hmac_secret = hmac.then(|| {
                let mut mac =
                    <Hmac<Sha256> as KeyInit>::new_from_slice(&self.secret).expect("hmac key");
                mac.update(&ctap_salt());
                let out = mac.finalize().into_bytes();
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&out);
                bytes
            });
            Assertion {
                credential_id: self.credential_id.clone(),
                authenticator_data: auth_data,
                client_data_json: client_data,
                signature,
                user_handle: Some(vec![0x22; 16]),
                hmac_secret,
            }
        }
    }

    #[test]
    fn a_fake_key_registers_and_asserts_with_hmac_secret() {
        let key = FakeKey::new("sync.kansoapp.app", "https://sync.kansoapp.app");
        let challenge = [0x7u8; 32];
        let registration = key.registration(&challenge, &[0x33; 16]);
        assert_eq!(
            attested_credential_id(&registration.auth_data),
            Some(key.credential_id.as_slice())
        );
        let object = attestation_object_none(&registration.auth_data);
        assert!(
            object
                .windows(registration.auth_data.len())
                .any(|w| w == registration.auth_data)
        );
        let client = String::from_utf8(registration.client_data_json.clone()).unwrap();
        assert!(client.contains("\"type\":\"webauthn.create\""));
        assert!(client.contains(&format!(
            "\"challenge\":\"{}\"",
            URL_SAFE_NO_PAD.encode(challenge)
        )));

        let assertion = key.assertion(&challenge, true);
        assert_eq!(assertion.hmac_secret, {
            let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&key.secret).unwrap();
            mac.update(&ctap_salt());
            let mut out = [0u8; 32];
            out.copy_from_slice(&mac.finalize().into_bytes());
            Some(out)
        });
        // The signature is over authenticatorData || SHA-256(clientDataJSON)
        // and verifies with the credential's public key.
        let mut signed = assertion.authenticator_data.clone();
        signed.extend_from_slice(&Sha256::digest(&assertion.client_data_json));
        UnparsedPublicKey::new(&ED25519, key.pair.public_key().as_ref())
            .verify(&signed, &assertion.signature)
            .expect("the assertion verifies");
        // Another RP's assertion doesn't verify against this key's message.
        let mut other = key.assertion(&[0x8u8; 32], false);
        other.authenticator_data[0] ^= 1;
        assert!(
            UnparsedPublicKey::new(&ED25519, key.pair.public_key().as_ref())
                .verify(&other.authenticator_data, &other.signature)
                .is_err()
        );
    }

    #[test]
    fn a_key_without_hmac_secret_signs_in_but_returns_no_prf() {
        let key = FakeKey::new("sync.kansoapp.app", "https://sync.kansoapp.app");
        let assertion = key.assertion(&[1u8; 32], false);
        assert_eq!(assertion.hmac_secret, None);
        assert!(!assertion.signature.is_empty());
    }

    #[test]
    fn the_credential_id_is_read_from_attested_data() {
        let mut auth_data = vec![0u8; 37];
        auth_data[32] = 0x41;
        let credential = vec![9u8; 32];
        auth_data.extend_from_slice(&[0u8; 16]);
        auth_data.extend_from_slice(&(credential.len() as u16).to_be_bytes());
        auth_data.extend_from_slice(&credential);
        auth_data.push(0xa0);
        assert_eq!(
            attested_credential_id(&auth_data),
            Some(credential.as_slice())
        );
        auth_data[32] = 0x01;
        assert_eq!(attested_credential_id(&auth_data), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn libfido2_authdata_is_read_raw() {
        platform::raw_authdata_regression();
        platform::cbor_byte_string_regression();
    }
}
