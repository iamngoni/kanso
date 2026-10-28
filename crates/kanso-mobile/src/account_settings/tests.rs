use kanso_sync::LinkedProvider;
use proptest::prelude::*;

use super::*;

fn providers() -> impl Strategy<Value = Provider> {
    prop_oneof![
        Just(Provider::Apple),
        Just(Provider::Google),
        Just(Provider::Github),
        "[a-z][a-z0-9_]{1,8}".prop_filter_map("a name no variant has", |name| {
            matches!(Provider::parse(&name), Some(Provider::Other(_)))
                .then(|| Provider::Other(name))
        }),
    ]
}

fn devices() -> impl Strategy<Value = Vec<DeviceInfo>> {
    proptest::collection::vec(
        (
            "d_[0-9a-f]{8}",
            proptest::option::of("[ -~]{0,12}"),
            -5i64..2_000_000,
            any::<bool>(),
        ),
        0..12,
    )
    .prop_map(|rows| {
        rows.into_iter()
            .enumerate()
            .map(|(i, (id, name, last_seen_at, current))| DeviceInfo {
                id: format!("{id}{i}"),
                name,
                created_at: 0,
                last_seen_at,
                current,
            })
            .collect()
    })
}

fn passkeys() -> impl Strategy<Value = Vec<PasskeyInfo>> {
    proptest::collection::vec(
        (
            proptest::collection::vec(any::<u8>(), 1..40),
            0i64..1_000_000,
            proptest::option::of(0i64..1_000_000),
            any::<bool>(),
            proptest::option::of(prop_oneof![
                Just(kanso_sync::provider::ICLOUD_KEYCHAIN_AAGUIDS[0].to_owned()),
                Just("00000000000000000000000000000000".to_owned()),
            ]),
        ),
        0..8,
    )
    .prop_map(|rows| {
        rows.into_iter()
            .enumerate()
            .map(
                |(i, (random, created_at, last_used_at, prf, aaguid))| PasskeyInfo {
                    // The index in front keeps the IDs apart.
                    credential_id: [vec![0, u8::try_from(i).unwrap()], random].concat(),
                    created_at,
                    last_used_at,
                    prf,
                    transports: Vec::new(),
                    aaguid,
                },
            )
            .collect()
    })
}

proptest! {
    /// The device list keeps every device once, this phone first, then the
    /// most recently seen; every device has a name to show, and a time only
    /// when it was seen.
    #[test]
    fn prop_device_rows_sort_this_phone_first_then_by_last_seen(devices in devices()) {
        let rows = device_rows(devices.clone());
        let mut ids: Vec<&str> = devices.iter().map(|device| device.id.as_str()).collect();
        let mut shown: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        ids.sort_unstable();
        shown.sort_unstable();
        prop_assert_eq!(ids, shown);
        for pair in rows.windows(2) {
            prop_assert!(pair[0].this_device >= pair[1].this_device);
            if pair[0].this_device == pair[1].this_device {
                prop_assert!(pair[0].last_seen_ms.unwrap_or(0) >= pair[1].last_seen_ms.unwrap_or(0));
            }
        }
        for row in &rows {
            prop_assert!(!row.title.trim().is_empty());
            let source = devices.iter().find(|device| device.id == row.id).unwrap();
            prop_assert_eq!(row.this_device, source.current);
            prop_assert_eq!(row.last_seen_ms.is_some(), source.last_seen_at > 0);
        }
    }

    /// Each provider the app has a button for is listed once, linked when
    /// the account linked it; a linked one the server stopped offering
    /// stays listed; names the app doesn't know are never listed.
    #[test]
    fn prop_provider_rows_list_each_known_provider_once(
        available in proptest::collection::vec(providers(), 0..5),
        linked in proptest::collection::vec(providers(), 0..5),
    ) {
        let methods = AccountMethods {
            has_password: true,
            rp_id: None,
            available: available.clone(),
            linked: linked
                .iter()
                .map(|provider| LinkedProvider { provider: provider.clone(), created_at: 1 })
                .collect(),
            passkeys: Vec::new(),
        };
        let rows = provider_rows(&methods);
        for (i, row) in rows.iter().enumerate() {
            prop_assert!(rows[i + 1..].iter().all(|other| other.provider != row.provider));
        }
        let known = |provider: &Provider| kind_of(provider).is_some();
        for provider in available.iter().chain(&linked).filter(|provider| known(provider)) {
            let kind = kind_of(provider).unwrap();
            let row = rows.iter().find(|row| row.provider == kind);
            prop_assert!(row.is_some());
            prop_assert_eq!(row.unwrap().linked, linked.contains(provider));
        }
        prop_assert_eq!(
            rows.len(),
            available
                .iter()
                .chain(&linked)
                .filter_map(kind_of)
                .map(|kind| format!("{kind:?}"))
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
    }

    /// A passkey's row names it by its credential ID, which decodes back to
    /// the bytes, and the rows are oldest first.
    #[test]
    fn prop_passkey_rows_name_each_by_its_credential(passkeys in passkeys()) {
        let rows = passkey_rows(&passkeys);
        prop_assert_eq!(rows.len(), passkeys.len());
        for row in &rows {
            let bytes = URL_SAFE_NO_PAD.decode(&row.id).unwrap();
            let source = passkeys.iter().find(|passkey| passkey.credential_id == bytes).unwrap();
            prop_assert_eq!(row.can_unlock, source.prf);
            prop_assert_eq!(row.added_ms, source.created_at);
            prop_assert_eq!(row.last_used_ms, source.last_used_at);
            prop_assert!(row.label.starts_with("Passkey"));
        }
        for pair in rows.windows(2) {
            prop_assert!(pair[0].added_ms <= pair[1].added_ms);
        }
    }
}
