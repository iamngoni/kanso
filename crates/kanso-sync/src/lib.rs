//! Kanso's sync client: accounts with end-to-end encryption
//! (docs/ACCOUNTS.md).
//!
//! A library binds to one account on one server ([`binding`]). The
//! [`account`] functions sign up, sign in, recover and manage it; the
//! [`keychain`] keeps the session and data key. [`sync()`] pulls the
//! account's sealed change feed and pushes local changes through `Sealed`,
//! which seals documents and files with the account's content keys
//! ([`crypto`]) and maps local IDs and revisions to opaque wire ones, so the
//! server never sees a note, a title, an ID or a revision. [`Backend`] is the
//! server: [`HttpBackend`] (the Cloudflare Worker's `/v2` routes) or
//! [`MemoryServer`] (tests). [`verify()`] and [`estimate()`] back the
//! `kanso-sync` command line. [`share`] publishes a note as a web page
//! anyone with its link can read and the server can't.

pub mod account;
pub mod api;
pub mod binding;
pub mod crypto;
mod engine;
mod http;
pub mod keychain;
pub mod lock;
mod memory;
pub mod passkey;
mod remote;
mod sealed;
pub mod security_key;
pub mod share;
pub mod site;
pub mod username;
mod verify;
pub mod wire;

pub use account::{
    AccountError, DEFAULT_SERVER, DeviceInfo, PasswordChanged, Quota, Recovered, RotationError,
    Session, SignedIn, SignedUp, SignupState, Step, Usage, change_password, delete_account,
    devices, discard_session, flush_pending_logouts, new_recovery_key, recover, recovery_key_held,
    revoke_device, server_info, sign_in, sign_out, sign_up, usage,
};
pub use api::{Api, Backend, Route};
pub use binding::{
    BindError, Choice, DeviceIdentity, Flow, LocalLibrary, Machine, Plan, bind, device_identity,
    device_name, local_library, meta_keys, plan, unbind,
};
pub use crypto::{
    CollabRoots, KdfParams, PRF_INPUT, Password, PasswordError, RecoveryKey, RecoveryKeyError,
    WrapPurpose, passkey_kek,
};
pub use engine::{
    Schedule, SyncError, SyncOptions, SyncProblem, SyncReport, batches, problem, report_problem,
    sync,
};
pub use http::HttpBackend;
pub use kanso_store::SyncDoc;
pub use keychain::{Keychain, MemorySecrets, SecretError, SecretStore};
pub use memory::{
    AccountSnapshot, DEFAULT_SETTINGS, Limits, MemoryServer, QuotaOverride, ServerDevice,
    ServerStats, Signup, is_valid_setting,
};
pub use passkey::{PasskeyAssertion, PasskeyInfo, PasskeyRegistration};
pub use remote::{
    PushResult, RemoteDoc, RemoteError, is_doc_id, is_file_id, is_md5_hex, is_rev, md5_hex,
};
pub use sealed::PlainView;
pub use username::{UsernameError, check_new_username, is_reserved_username, normalize_username};
pub use verify::{
    Estimate, LegacyManifest, LegacyReport, LegacyRow, VerifyReport, estimate, manifest_sha256,
    verify,
};
pub use wire::{ApiError, QuotaKind, SessionToken, SignOutReason, SignupClosedReason};

// The app shares these across threads (docs/ACCOUNTS.md › Services).
const _: fn() = || {
    fn send_sync<T: Send + Sync + ?Sized>() {}
    send_sync::<Session>();
    send_sync::<dyn Backend>();
    send_sync::<HttpBackend>();
    send_sync::<MemoryServer>();
    send_sync::<dyn SecretStore>();
    send_sync::<Keychain>();
    send_sync::<MemorySecrets>();
    send_sync::<SyncOptions>();
    send_sync::<DeviceIdentity>();
    send_sync::<dyn share::ShareApi>();
    send_sync::<share::HttpShares>();
    send_sync::<share::MemoryShares>();
};

/// Why a Worker URL was refused.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EndpointError {
    #[error("that isn't a URL")]
    Invalid,
    #[error(
        "use an https:// URL (http:// only reaches this device: 127.0.0.1, localhost or [::1])"
    )]
    Insecure,
    #[error("the URL can't contain a user name, password, query or fragment")]
    Extra,
}

/// The owner's server under its first name. Every account on it was made
/// there, and the key derivations bind it, so its later names
/// ([`OWNER_ALIASES`]) derive as this one.
pub const OWNER_ORIGIN: &str = "https://kanso.antonlabs.cc";

/// The owner's server's later names, which derive as [`OWNER_ORIGIN`].
const OWNER_ALIASES: &[&str] = &["https://sync.kansoapp.app"];

/// Where the owner's server shows published notes.
pub const OWNER_SHARE_SERVER: &str = "https://share.kansoapp.app";

/// The origin of a server URL, as the key derivations bind it:
/// `url::Url::parse(server)?.origin().ascii_serialization()`: lowercase,
/// punycode, default port omitted, path dropped; one of the owner's
/// server's later names gives its first, [`OWNER_ORIGIN`]. The server never
/// supplies it. Pure.
pub fn origin(server: &str) -> Result<String, EndpointError> {
    let url = url::Url::parse(server.trim()).map_err(|_| EndpointError::Invalid)?;
    match url.origin() {
        url::Origin::Tuple(..) => {
            let origin = url.origin().ascii_serialization();
            if OWNER_ALIASES.contains(&origin.as_str()) {
                Ok(OWNER_ORIGIN.to_owned())
            } else {
                Ok(origin)
            }
        }
        url::Origin::Opaque(_) => Err(EndpointError::Invalid),
    }
}

/// Whether `server` is the owner's, under any of its names. Pure.
pub fn is_owner_server(server: &str) -> bool {
    origin(server).as_deref() == Ok(OWNER_ORIGIN)
}

/// Checks a Worker URL and returns it normalized, without a trailing slash.
/// Only https is accepted, apart from plain http to this Mac (for `wrangler
/// dev`), so proofs and session tokens never cross the network unencrypted. A host
/// ending in a dot is refused: it would give the same server a second origin
/// (docs/ACCOUNTS.md › Conventions).
pub fn check_endpoint(input: &str) -> Result<String, EndpointError> {
    let url = url::Url::parse(input.trim()).map_err(|_| EndpointError::Invalid)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(EndpointError::Extra);
    }
    let loopback = match url.host() {
        Some(url::Host::Domain(domain)) if domain.ends_with('.') => {
            return Err(EndpointError::Invalid);
        }
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        None => return Err(EndpointError::Invalid),
    };
    match url.scheme() {
        "https" => {}
        "http" if loopback => {}
        _ => return Err(EndpointError::Insecure),
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

#[cfg(test)]
mod endpoint_tests {
    use super::{EndpointError, OWNER_ORIGIN, check_endpoint, is_owner_server, origin};
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert_eq!(
            check_endpoint(" https://kanso-sync.imngonii.workers.dev/ ").as_deref(),
            Ok("https://kanso-sync.imngonii.workers.dev")
        );
        assert_eq!(
            check_endpoint("http://127.0.0.1:8787").as_deref(),
            Ok("http://127.0.0.1:8787")
        );
        assert_eq!(
            check_endpoint("http://localhost:8787/").as_deref(),
            Ok("http://localhost:8787")
        );
        assert_eq!(
            check_endpoint("http://[::1]:8787").as_deref(),
            Ok("http://[::1]:8787")
        );
        assert_eq!(
            check_endpoint("http://127.0.0.1.evil.example"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("http://localhost.evil.example"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("http://example.com"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("ftp://example.com"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("https://user:pw@example.com"),
            Err(EndpointError::Extra)
        );
        // A trailing dot names the same host under a second origin.
        assert_eq!(
            check_endpoint("https://kanso-sync.imngonii.workers.dev./"),
            Err(EndpointError::Invalid)
        );
        assert_eq!(
            check_endpoint("http://localhost.:8787"),
            Err(EndpointError::Invalid)
        );
        assert_eq!(
            check_endpoint("https://example.com/?a=1"),
            Err(EndpointError::Extra)
        );
        assert_eq!(check_endpoint("https://"), Err(EndpointError::Invalid));
        assert_eq!(check_endpoint("example.com"), Err(EndpointError::Invalid));
        assert_eq!(check_endpoint("https://"), Err(EndpointError::Invalid));
    }

    #[test]
    fn origin_examples() {
        assert_eq!(
            origin("https://Kanso-Sync.imngonii.workers.dev/v2/").as_deref(),
            Ok("https://kanso-sync.imngonii.workers.dev")
        );
        assert_eq!(
            origin("https://example.com:443").as_deref(),
            Ok("https://example.com")
        );
        assert_eq!(
            origin("http://127.0.0.1:8787").as_deref(),
            Ok("http://127.0.0.1:8787")
        );
        assert_eq!(
            origin("https://bücher.example").as_deref(),
            Ok("https://xn--bcher-kva.example")
        );
        assert_eq!(origin("not a url"), Err(EndpointError::Invalid));
        // The owner's server derives as its first name under every name.
        for name in [
            "https://sync.kansoapp.app",
            "https://Sync.KansoApp.app/v2/",
            OWNER_ORIGIN,
        ] {
            assert_eq!(origin(name).as_deref(), Ok(OWNER_ORIGIN), "{name}");
            assert!(is_owner_server(name), "{name}");
        }
        assert!(is_owner_server(crate::DEFAULT_SERVER));
        assert!(!is_owner_server("https://share.kansoapp.app"));
        assert!(!is_owner_server("https://kansoapp.app"));
    }

    fn gen_url() -> impl Strategy<Value = String> {
        let scheme = prop_oneof![Just("http"), Just("https"), Just("HTTP"), Just("ftp")];
        let host = prop_oneof![
            Just("127.0.0.1".to_owned()),
            Just("localhost".to_owned()),
            Just("[::1]".to_owned()),
            "[a-z]{1,8}\\.[a-z]{2,3}",
            "(127\\.0\\.0\\.1|localhost)\\.[a-z]{1,8}\\.[a-z]{2,3}",
            "(127\\.0\\.0\\.1|localhost)@[a-z]{1,8}\\.[a-z]{2,3}",
        ];
        let port = prop::option::of(1u16..);
        let path = "(/[a-z]{0,4}){0,2}";
        (scheme, host, port, path).prop_map(|(scheme, host, port, path)| {
            let port = port.map(|p| format!(":{p}")).unwrap_or_default();
            format!("{scheme}://{host}{port}{path}")
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Refinement: an accepted URL is https, or http to a loopback host,
        /// and carries nothing that could hide where it really goes.
        #[test]
        fn prop_accepted_urls_are_https_or_this_mac(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                let parsed = url::Url::parse(&url).unwrap();
                let loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
                prop_assert!(parsed.scheme() == "https" || (parsed.scheme() == "http" && loopback), "{}", url);
                prop_assert!(parsed.username().is_empty() && parsed.password().is_none());
                prop_assert!(!url.ends_with('/'));
            }
        }

        /// Metamorphic: plain http is refused unless the host is this Mac,
        /// and switching to https makes any well-formed host acceptable.
        #[test]
        fn prop_http_only_reaches_this_mac(input in gen_url()) {
            let parsed = url::Url::parse(&input).unwrap();
            if parsed.scheme() == "http" && parsed.username().is_empty() {
                let loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
                prop_assert_eq!(check_endpoint(&input).is_ok(), loopback);
                let secure = input.replacen("http://", "https://", 1).replacen("HTTP://", "https://", 1);
                prop_assert!(check_endpoint(&secure).is_ok(), "{}", secure);
            }
        }

        /// Invariant: an accepted URL's origin is its scheme and host (and a
        /// non-default port) only, and the URL starts with it.
        #[test]
        fn prop_origins_drop_the_path(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                let o = origin(&url).unwrap();
                prop_assert!(url.starts_with(&o), "{} {}", url, o);
                prop_assert!(!o[o.find("://").unwrap() + 3..].contains('/'));
                prop_assert_eq!(origin(&o), Ok(o.clone()));
            }
        }

        /// Idempotence: checking an accepted URL again returns it unchanged.
        #[test]
        fn prop_check_endpoint_is_idempotent(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                prop_assert_eq!(check_endpoint(&url), Ok(url.clone()));
            }
        }
    }
}
