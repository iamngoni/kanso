//! The format-1 constructions (docs/ACCOUNTS.md › Constructions) through
//! the public API: round trips, and every tamper, wrong key, AAD change and
//! too-new format byte rejected. The golden vectors are in crypto.rs's unit
//! tests (`tests/fixtures/crypto-v1.json`). CI pins the seed with
//! `PROPTEST_RNG_SEED`.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_sync::SyncDoc;
use kanso_sync::crypto::{
    ContentKeys, CryptoError, DataKey, KdfParams, Opened, Password, RecoveryKey, WrapPurpose,
    key_id, password_keys, recovery_keys, unwrap, wrap,
};
use kanso_sync::wire::{WireDoc, is_sealed_name, is_wire_id, is_wire_rev};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};
use unicode_normalization::UnicodeNormalization as _;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

const ACCOUNT: &str = "a_0123456789abcdef0123456789abcdef";
const OTHER_ACCOUNT: &str = "a_fedcba9876543210fedcba9876543210";
const ORIGIN: &str = "https://kanso.example";

fn decode(text: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD.decode(text).unwrap()
}

fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn flipped(sealed: &str, bit: usize) -> String {
    let mut bytes = decode(sealed);
    let at = bit % (bytes.len() * 8);
    bytes[at / 8] ^= 1 << (at % 8);
    encode(&bytes)
}

fn with_format(sealed: &str, format: u8) -> String {
    let mut bytes = decode(sealed);
    bytes[0] = format;
    encode(&bytes)
}

fn opened<T>(opened: &Opened<T>) -> bool {
    matches!(opened, Opened::Opened(_))
}

fn kind() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("note"), Just("book"), Just("tag"), Just("file")]
}

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        "\\PC{0,24}".prop_map(Value::String),
    ];
    leaf.prop_recursive(2, 12, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::btree_map("[a-zA-Z_]{1,8}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

prop_compose! {
    fn sync_doc()(
        kind in kind(),
        suffix in "[A-Za-z0-9_-]{4,22}",
        generation in 1u64..1000,
        hash in "[0-9a-f]{32}",
        deleted in any::<bool>(),
        fields in prop::collection::btree_map("[a-zA-Z]{1,10}", json_value(), 0..6),
    ) -> SyncDoc {
        SyncDoc {
            id: format!("{kind}:{suffix}"),
            rev: format!("{generation}-{hash}"),
            deleted,
            fields: if deleted { Map::new() } else { fields.into_iter().collect() },
            base_rev: None,
        }
    }
}

fn data_key() -> impl Strategy<Value = DataKey> {
    any::<[u8; 32]>().prop_map(DataKey::from_bytes)
}

proptest! {
    #![proptest_config(config(128))]

    /// Round trip: a sealed document opens to itself, with a well-formed
    /// wire ID and revision of the same kind, and sealed text the server
    /// accepts.
    #[test]
    fn prop_documents_round_trip(key in data_key(), doc in sync_doc()) {
        let keys = ContentKeys::derive(&key);
        let wire = keys.seal_doc(ACCOUNT, &doc);
        prop_assert!(is_wire_id(&wire.id));
        prop_assert!(is_wire_rev(&wire.rev));
        prop_assert_eq!(wire.id.split(':').next(), doc.id.split(':').next());
        prop_assert_eq!(wire.deleted, doc.deleted);
        prop_assert!(wire.sealed.len() >= 55);
        prop_assert_eq!(keys.open_doc(ACCOUNT, &wire), Opened::Opened(doc.clone()));
        // The opaque names are deterministic; the sealed text is not.
        let again = keys.seal_doc(ACCOUNT, &doc);
        prop_assert_eq!(&again.id, &wire.id);
        prop_assert_eq!(&again.rev, &wire.rev);
        prop_assert_ne!(&again.sealed, &wire.sealed);
    }

    /// Invariant: no single bit flip anywhere in a sealed document opens; a
    /// flip that raises the format byte reads as too new.
    #[test]
    fn prop_every_bit_flip_in_a_document_is_rejected(key in data_key(), doc in sync_doc(), bit in any::<usize>()) {
        let keys = ContentKeys::derive(&key);
        let wire = keys.seal_doc(ACCOUNT, &doc);
        let tampered = WireDoc { sealed: flipped(&wire.sealed, bit), ..wire };
        let format = decode(&tampered.sealed)[0];
        let result = keys.open_doc(ACCOUNT, &tampered);
        prop_assert!(!opened(&result));
        if format > 1 {
            prop_assert_eq!(result, Opened::TooNew);
        } else {
            prop_assert_eq!(result, Opened::Undecryptable);
        }
    }

    /// Invariant: a document opens only under its own data key, account,
    /// wire ID, wire revision and deleted flag.
    #[test]
    fn prop_documents_are_bound_to_their_context(key in data_key(), other in data_key(), doc in sync_doc(), second in sync_doc()) {
        prop_assume!(key != other);
        let keys = ContentKeys::derive(&key);
        let wire = keys.seal_doc(ACCOUNT, &doc);
        prop_assert_eq!(ContentKeys::derive(&other).open_doc(ACCOUNT, &wire), Opened::Undecryptable);
        prop_assert_eq!(keys.open_doc(OTHER_ACCOUNT, &wire), Opened::Undecryptable);
        let flipped_deleted = WireDoc { deleted: !wire.deleted, ..wire.clone() };
        prop_assert_eq!(keys.open_doc(ACCOUNT, &flipped_deleted), Opened::Undecryptable);
        // Swapping in another document's wire ID or revision.
        let other_wire = keys.seal_doc(ACCOUNT, &second);
        if other_wire.id != wire.id {
            let swapped = WireDoc { id: other_wire.id.clone(), ..wire.clone() };
            prop_assert_eq!(keys.open_doc(ACCOUNT, &swapped), Opened::Undecryptable);
        }
        if other_wire.rev != wire.rev {
            let swapped = WireDoc { rev: other_wire.rev.clone(), ..wire.clone() };
            prop_assert_eq!(keys.open_doc(ACCOUNT, &swapped), Opened::Undecryptable);
        }
        // The same opaque ID under another kind.
        let (kind, rest) = wire.id.split_once(':').unwrap();
        let relabelled = if kind == "note" { "book" } else { "note" };
        let rekinded = WireDoc { id: format!("{relabelled}:{rest}"), ..wire.clone() };
        prop_assert_eq!(keys.open_doc(ACCOUNT, &rekinded), Opened::Undecryptable);
        // Sealed text swapped between two documents.
        let moved = WireDoc { sealed: other_wire.sealed.clone(), ..wire.clone() };
        if other_wire.id != wire.id || other_wire.rev != wire.rev || other_wire.deleted != wire.deleted {
            prop_assert_eq!(keys.open_doc(ACCOUNT, &moved), Opened::Undecryptable);
        }
    }

    /// Invariant: any format byte above 1 is too new, for documents, files
    /// and device names alike; 0 is undecryptable.
    #[test]
    fn prop_newer_formats_are_too_new(key in data_key(), doc in sync_doc(), format in 2u8..=255) {
        let keys = ContentKeys::derive(&key);
        let wire = keys.seal_doc(ACCOUNT, &doc);
        let newer = WireDoc { sealed: with_format(&wire.sealed, format), ..wire.clone() };
        prop_assert_eq!(keys.open_doc(ACCOUNT, &newer), Opened::TooNew);
        let zero = WireDoc { sealed: with_format(&wire.sealed, 0), ..wire };
        prop_assert_eq!(keys.open_doc(ACCOUNT, &zero), Opened::Undecryptable);
        let file_id = keys.wire_id("file:abc");
        let mut file = keys.seal_file(ACCOUNT, &file_id, b"bytes");
        file[0] = format;
        prop_assert_eq!(keys.open_file(ACCOUNT, &file_id, &file), Opened::TooNew);
        let name = keys.seal_name(ACCOUNT, "d_00", "Mac");
        prop_assert_eq!(keys.open_name(ACCOUNT, "d_00", &with_format(&name, format)), Opened::TooNew);
    }

    /// Round trip and binding for files: the bytes open only under their own
    /// key, account and wire ID, and no bit flip opens.
    #[test]
    fn prop_files_round_trip_and_reject_tampering(
        key in data_key(),
        other in data_key(),
        bytes in prop::collection::vec(any::<u8>(), 0..300),
        bit in any::<usize>(),
    ) {
        prop_assume!(key != other);
        let keys = ContentKeys::derive(&key);
        let wire_id = keys.wire_id("file:Gold3n");
        let sealed = keys.seal_file(ACCOUNT, &wire_id, &bytes);
        prop_assert_eq!(sealed.len(), bytes.len() + 41);
        prop_assert_eq!(keys.open_file(ACCOUNT, &wire_id, &sealed), Opened::Opened(bytes.clone()));
        prop_assert!(!opened(&ContentKeys::derive(&other).open_file(ACCOUNT, &wire_id, &sealed)));
        prop_assert!(!opened(&keys.open_file(OTHER_ACCOUNT, &wire_id, &sealed)));
        prop_assert!(!opened(&keys.open_file(ACCOUNT, &keys.wire_id("file:Other"), &sealed)));
        let mut tampered = sealed.clone();
        let at = bit % (tampered.len() * 8);
        tampered[at / 8] ^= 1 << (at % 8);
        prop_assert!(!opened(&keys.open_file(ACCOUNT, &wire_id, &tampered)));
        prop_assert!(!opened(&keys.open_file(ACCOUNT, &wire_id, &sealed[..sealed.len() - 1])));
    }

    /// Round trip and binding for device names: within the server's 55–400
    /// characters, and bound to the account and device ID.
    #[test]
    fn prop_device_names_round_trip(key in data_key(), name in "\\PC{1,64}", bit in any::<usize>()) {
        let keys = ContentKeys::derive(&key);
        let device = "d_0123456789abcdef0123456789abcdef";
        let sealed = keys.seal_name(ACCOUNT, device, &name);
        prop_assert!(is_sealed_name(&sealed), "{} characters", sealed.len());
        let expected = kanso_sync::device_name(&name);
        prop_assert_eq!(keys.open_name(ACCOUNT, device, &sealed), Opened::Opened(expected));
        prop_assert!(!opened(&keys.open_name(OTHER_ACCOUNT, device, &sealed)));
        prop_assert!(!opened(&keys.open_name(ACCOUNT, "d_fedcba9876543210fedcba9876543210", &sealed)));
        prop_assert!(!opened(&keys.open_name(ACCOUNT, device, &flipped(&sealed, bit))));
    }

    /// Round trip and binding for wraps: a wrap opens only with its KEK,
    /// purpose and account; every bit flip and truncation is a key mismatch.
    #[test]
    fn prop_wraps_open_only_in_their_context(
        key in data_key(),
        secret in any::<[u8; 16]>(),
        other_secret in any::<[u8; 16]>(),
        bit in any::<usize>(),
    ) {
        prop_assume!(secret != other_secret);
        let kek = recovery_keys(&RecoveryKey::from_bytes(secret), ORIGIN, "alice").kek;
        let other = recovery_keys(&RecoveryKey::from_bytes(other_secret), ORIGIN, "alice").kek;
        let wrapped = wrap(&kek, WrapPurpose::Recovery, ACCOUNT, &key);
        prop_assert_eq!(wrapped.len(), 98);
        prop_assert_eq!(unwrap(&kek, WrapPurpose::Recovery, ACCOUNT, &wrapped), Ok(key.clone()));
        prop_assert_eq!(unwrap(&other, WrapPurpose::Recovery, ACCOUNT, &wrapped), Err(CryptoError::KeyMismatch));
        prop_assert_eq!(unwrap(&kek, WrapPurpose::Password, ACCOUNT, &wrapped), Err(CryptoError::KeyMismatch));
        prop_assert_eq!(unwrap(&kek, WrapPurpose::Recovery, OTHER_ACCOUNT, &wrapped), Err(CryptoError::KeyMismatch));
        prop_assert_eq!(
            unwrap(&kek, WrapPurpose::Recovery, ACCOUNT, &flipped(&wrapped, bit)),
            Err(CryptoError::KeyMismatch)
        );
        prop_assert_eq!(unwrap(&kek, WrapPurpose::Recovery, ACCOUNT, &wrapped[..97]), Err(CryptoError::KeyMismatch));
        // A fresh wrap of the same key differs but opens to the same key.
        let again = wrap(&kek, WrapPurpose::Recovery, ACCOUNT, &key);
        prop_assert_ne!(&again, &wrapped);
        prop_assert_eq!(unwrap(&kek, WrapPurpose::Recovery, ACCOUNT, &again), Ok(key));
    }

    /// Metamorphic: rotating the recovery key rewraps the same data key; the
    /// old recovery KEK can't open the new wrap, and the key ID is unchanged.
    #[test]
    fn prop_recovery_rotation_keeps_the_data_key(key in data_key(), old in any::<[u8; 16]>(), new in any::<[u8; 16]>()) {
        prop_assume!(old != new);
        let old = recovery_keys(&RecoveryKey::from_bytes(old), ORIGIN, "alice");
        let new = recovery_keys(&RecoveryKey::from_bytes(new), ORIGIN, "alice");
        prop_assert_ne!(old.auth_key.to_base64url(), new.auth_key.to_base64url());
        let wrapped = wrap(&new.kek, WrapPurpose::Recovery, ACCOUNT, &key);
        prop_assert_eq!(unwrap(&old.kek, WrapPurpose::Recovery, ACCOUNT, &wrapped), Err(CryptoError::KeyMismatch));
        let opened = unwrap(&new.kek, WrapPurpose::Recovery, ACCOUNT, &wrapped).unwrap();
        prop_assert_eq!(key_id(&opened), key_id(&key));
    }

    /// Invariant: the key ID is 16 hex digits and tells data keys apart.
    #[test]
    fn prop_key_ids_identify_data_keys(a in data_key(), b in data_key()) {
        let id = key_id(&a);
        prop_assert_eq!(id.len(), 16);
        prop_assert!(id.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        prop_assert_eq!(key_id(&a) == key_id(&b), a == b);
    }
}

proptest! {
    #![proptest_config(config(24))]

    /// Metamorphic: NFC and NFD spellings of a password derive the same
    /// keys; changing the origin or the username in the salt input changes
    /// the auth key.
    #[test]
    fn prop_password_keys_bind_origin_and_username(
        text in "[a-zé\u{e9}\u{301}ñÅ0-9 ]{12,30}",
        salt in any::<[u8; 16]>(),
    ) {
        let nfc: String = text.nfc().collect();
        let nfd: String = text.nfd().collect();
        let (Ok(a), Ok(b)) = (Password::new(&nfc, "alice"), Password::new(&nfd, "alice")) else {
            return Ok(());
        };
        let derive = |password: &Password, origin: &str, username: &str| {
            password_keys(password, origin, username, &salt, KdfParams::TEST).unwrap().auth_key.to_base64url()
        };
        let base = derive(&a, ORIGIN, "alice");
        prop_assert_eq!(&base, &derive(&b, ORIGIN, "alice"));
        prop_assert_ne!(&base, &derive(&a, "https://kanso.example:444", "alice"));
        prop_assert_ne!(&base, &derive(&a, ORIGIN, "alice2"));
    }
}

/// Rewrapping under a new password keeps the data key; the old KEK can't
/// open the new wrap.
#[test]
fn rewrap_keeps_the_data_key() {
    let key = DataKey::generate();
    let derive = |text: &str, salt: [u8; 16]| {
        let password = Password::new(text, "alice").unwrap();
        password_keys(&password, ORIGIN, "alice", &salt, KdfParams::TEST).unwrap()
    };
    let old = derive("correct horse battery", [1; 16]);
    let new = derive("another long passphrase", [2; 16]);
    let first = wrap(&old.kek, WrapPurpose::Password, ACCOUNT, &key);
    let second = wrap(&new.kek, WrapPurpose::Password, ACCOUNT, &key);
    assert_eq!(
        unwrap(&new.kek, WrapPurpose::Password, ACCOUNT, &second),
        Ok(key.clone())
    );
    assert_eq!(
        unwrap(&old.kek, WrapPurpose::Password, ACCOUNT, &second),
        Err(CryptoError::KeyMismatch)
    );
    assert_eq!(
        unwrap(&old.kek, WrapPurpose::Password, ACCOUNT, &first),
        Ok(key)
    );
    // Same password, new salt: a different auth key.
    let same = derive("correct horse battery", [3; 16]);
    assert_ne!(same.auth_key.to_base64url(), old.auth_key.to_base64url());
    // Secrets never print.
    assert!(format!("{:?}", old.kek).contains("redacted"));
    assert!(format!("{:?}", old.auth_key).contains("redacted"));
    assert!(format!("{:?}", ContentKeys::derive(&DataKey::generate())).contains("redacted"));
}

/// A tombstone carrying fields doesn't open, and plaintext of the wrong
/// shape can't be produced without the key, so a server can only forge a
/// tombstone by flipping the deleted flag, which the AAD refuses.
#[test]
fn forged_tombstones_are_refused() {
    let keys = ContentKeys::derive(&DataKey::generate());
    let doc = SyncDoc {
        id: "note:abc".into(),
        rev: "2-0123456789abcdef0123456789abcdef".into(),
        deleted: false,
        fields: json!({ "body": "hello" }).as_object().unwrap().clone(),
        base_rev: None,
    };
    let wire = keys.seal_doc(ACCOUNT, &doc);
    let forged = WireDoc {
        deleted: true,
        ..wire
    };
    assert_eq!(keys.open_doc(ACCOUNT, &forged), Opened::Undecryptable);
    // Malformed sealed text.
    let garbage = WireDoc {
        sealed: "!!".into(),
        ..forged.clone()
    };
    assert_eq!(keys.open_doc(ACCOUNT, &garbage), Opened::Undecryptable);
    let short = WireDoc {
        sealed: encode(&[1, 2, 3]),
        ..forged
    };
    assert_eq!(keys.open_doc(ACCOUNT, &short), Opened::Undecryptable);
}

// --------------------------------------------------------------- passkeys

use kanso_sync::PRF_INPUT;
use kanso_sync::PasskeyAssertion;
use kanso_sync::passkey::{
    client_data_json, open_passkey_wrapped_key, passkey_wrapped_key, prf_input, user_handle,
};
use kanso_sync::security_key::ctap_salt;

const CREDENTIAL: &[u8] = &[7u8; 32];
const OTHER_CREDENTIAL: &[u8] = &[8u8; 32];

#[test]
fn the_prf_input_and_ctap_salt_are_pinned() {
    assert_eq!(
        PRF_INPUT,
        [
            0x98, 0x43, 0x20, 0xbf, 0xab, 0xbd, 0x34, 0x26, 0x97, 0xaf, 0x83, 0x64, 0x7d, 0xc7,
            0x74, 0xe3, 0x07, 0xf9, 0x8e, 0x62, 0xec, 0xf2, 0x7d, 0xd5, 0xd8, 0x4f, 0x8f, 0x00,
            0x6d, 0x86, 0xfd, 0x59
        ]
    );
    assert_eq!(prf_input(), PRF_INPUT);
    // SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT): the authData the
    // authenticator hashes the PRF input with.
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
fn the_user_handle_is_sixteen_opaque_bytes() {
    assert_eq!(user_handle(ACCOUNT).len(), 16);
    assert_eq!(user_handle(ACCOUNT), user_handle(ACCOUNT));
    assert_ne!(user_handle(ACCOUNT), user_handle(OTHER_ACCOUNT));
}

#[test]
fn a_passkey_wrap_round_trips_and_is_bound_to_its_context() {
    let key = DataKey::from_bytes([9u8; 32]);
    let prf = [3u8; 32];
    let wrapped = passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &key);
    assert_eq!(wrapped.len(), 98);
    assert_eq!(
        open_passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &wrapped).unwrap(),
        key
    );
    // A wrap moved to another credential, account or origin derives another
    // KEK and fails to open. Another PRF output fails too.
    for wrong in [
        open_passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, OTHER_CREDENTIAL, &wrapped),
        open_passkey_wrapped_key(&prf, ORIGIN, OTHER_ACCOUNT, CREDENTIAL, &wrapped),
        open_passkey_wrapped_key(&prf, "https://other.example", ACCOUNT, CREDENTIAL, &wrapped),
        open_passkey_wrapped_key(&[4u8; 32], ORIGIN, ACCOUNT, CREDENTIAL, &wrapped),
    ] {
        assert_eq!(wrong, Err(CryptoError::KeyMismatch));
    }
    // The same bytes under another purpose don't open either.
    assert_eq!(
        unwrap(
            &kanso_sync::crypto::passkey_kek(&prf, ORIGIN, ACCOUNT, &encode(CREDENTIAL)),
            WrapPurpose::Password,
            ACCOUNT,
            &wrapped
        ),
        Err(CryptoError::KeyMismatch)
    );
}

proptest! {
    #![proptest_config(config(128))]

    /// Invariant: a wrap opens only with the same PRF output, origin, account
    /// and credential, and every bit flip in it is refused. A credential
    /// without a PRF output has no wrap to move.
    #[test]
    fn prop_passkey_wraps_open_only_in_their_context(
        prf in any::<[u8; 32]>(),
        other_prf in any::<[u8; 32]>(),
        key in data_key(),
        other_key in data_key(),
        bit in any::<usize>(),
    ) {
        prop_assume!(prf != other_prf);
        let wrapped = passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &key);
        prop_assert_eq!(
            open_passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &wrapped),
            Ok(key.clone())
        );
        prop_assert_eq!(
            open_passkey_wrapped_key(&other_prf, ORIGIN, ACCOUNT, CREDENTIAL, &wrapped),
            Err(CryptoError::KeyMismatch)
        );
        let moved = passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &other_key);
        if key != other_key {
            prop_assert_ne!(&moved, &wrapped);
        }
        prop_assert_eq!(
            open_passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, OTHER_CREDENTIAL, &wrapped),
            Err(CryptoError::KeyMismatch)
        );
        prop_assert_eq!(
            open_passkey_wrapped_key(&prf, ORIGIN, ACCOUNT, CREDENTIAL, &flipped(&wrapped, bit)),
            Err(CryptoError::KeyMismatch)
        );
    }
}

#[test]
fn passkey_shapes_never_print_the_prf_output() {
    let registration = kanso_sync::PasskeyRegistration {
        credential_id: CREDENTIAL.to_vec(),
        attestation_object: vec![1, 2, 3],
        client_data_json: b"{}".to_vec(),
        transports: vec!["internal".into()],
        prf: Some([0xab; 32]),
    };
    let assertion = PasskeyAssertion {
        credential_id: CREDENTIAL.to_vec(),
        authenticator_data: vec![1, 2, 3],
        client_data_json: b"{}".to_vec(),
        signature: vec![4, 5, 6],
        user_handle: None,
        prf: Some([0xcd; 32]),
    };
    for debug in [format!("{registration:?}"), format!("{assertion:?}")] {
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("cdcd"));
        assert!(!debug.contains("abab"));
    }
    let data = client_data_json("webauthn.get", &[1u8; 32], ORIGIN);
    assert!(!String::from_utf8_lossy(&data).contains("prf"));
}
