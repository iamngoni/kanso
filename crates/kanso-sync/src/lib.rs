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
pub mod approval;
pub mod approvals;
pub mod binding;
pub mod crypto;
mod engine;
pub mod events;
mod http;
pub mod keychain;
pub mod lock;
mod memory;
pub mod methods;
pub mod passkey;
pub mod provider;
mod remote;
mod sealed;
pub mod security_key;
pub mod share;
pub mod site;
#[cfg(any(test, feature = "test-support"))]
mod test_authenticator;
pub mod username;
mod verify;
pub mod webauthn;
pub mod wire;
pub mod ws;

pub use account::{
    AccountError, DEFAULT_SERVER, DeviceInfo, PasswordChanged, Quota, Recovered, RotationError,
    Session, SignedIn, SignedUp, SignupState, Step, Usage, change_password, delete_account,
    devices, discard_locked_session, discard_session, finish_google, finish_identity,
    finish_identity_google, flush_pending_logouts, methods, methods_locked, new_recovery_key,
    poll_provider, recover, recovery_key_held, refresh_locked_methods, register_push,
    revoke_device, server_auth, server_info, sign_in, sign_in_with_provider, sign_out,
    sign_out_locked, sign_up, sign_up_with_provider, start_identity, start_provider,
    unlink_identity, unlock_with_password, unregister_push, usage,
};
pub use api::{Api, Backend, PushEnvironment, PushPlatform, PushRegistration, Route};
pub use approval::{Approval, ApprovalKeys};
pub use approvals::{
    ApprovalRequest, ApprovalResume, Approved, LockedSession, OpenedRequest, PendingApproval,
    PinnedRequest, RequestState, account_events_path, cancel_request, cancel_saved_approval,
    challenge_request, complete_request, complete_saved_request, decline_request, finish_approval,
    finish_unlock, name_device, name_device_resumable, new_approval_request_id, open_request,
    pending_approvals, poll_approval, poll_saved_approval, request_approval,
    request_approval_resumable, resume_approval, saved_locked_pending, unlock_with_local_key,
};
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
pub use events::{AccountEvent, EventError, EventSource, Signal};
pub use http::HttpBackend;
pub use kanso_store::SyncDoc;
#[cfg(all(
    feature = "system-keychain",
    any(target_os = "macos", target_os = "windows", target_os = "linux")
))]
pub use keychain::Keychain;
pub use keychain::{MemorySecrets, SecretError, SecretStore};
pub use memory::{
    AccountSnapshot, DEFAULT_SETTINGS, Limits, MEMORY_PASSKEY_ORIGIN, MEMORY_PASSKEY_RP_ID,
    MEMORY_PASSKEY_SERVER, MemoryServer, QuotaOverride, ServerDevice, ServerStats, Signup,
    is_valid_setting,
};
pub use methods::{
    PasskeyProofStart, PasskeyRegistrationStart, PasskeySignIn, PasskeySignInStart,
    PasskeySignUpStart, PasskeyUnlock, Proof, add_passkey, delete_account_with_proof,
    new_recovery_key_with_proof, passkey_unlock, password_proof, remove_passkey, set_password,
    sign_in_with_passkey, sign_up_with_passkey, start_add_passkey, start_passkey_proof,
    start_passkey_sign_in, start_passkey_sign_up,
};
pub use passkey::{
    PasskeyAssertion, PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError,
    PasskeyRegistration, PasskeyRegistrationChallenge,
};
pub use provider::{
    AccountMethods, AppleCallback, AppleStart, GithubStart, GoogleStart, IdentityIntent,
    LinkOutcome, LinkedProvider, Loopback, NewAccount, PasskeyInfo, PasskeyMethods, ProofWire,
    Provider, ProviderSession, ProviderSignIn, ProviderStart, ProviderStep, ServerAuth,
    apple_callback_flow,
};
pub use remote::{
    PushResult, RemoteDoc, RemoteError, is_doc_id, is_file_id, is_md5_hex, is_rev, md5_hex,
};
pub use sealed::PlainView;
#[cfg(any(test, feature = "test-support"))]
pub use test_authenticator::TestAuthenticator;
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
    send_sync::<LockedSession>();
    send_sync::<ApprovalKeys>();
    send_sync::<Approval>();
    send_sync::<dyn Backend>();
    send_sync::<HttpBackend>();
    send_sync::<MemoryServer>();
    send_sync::<dyn SecretStore>();
    #[cfg(all(
        feature = "system-keychain",
        any(target_os = "macos", target_os = "windows", target_os = "linux")
    ))]
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

/// The relying party the official app's Associated Domains entitlement
/// names (`webcredentials:sync.kansoapp.app`), which the owner's server
/// advertises.
pub const OFFICIAL_RP_ID: &str = "sync.kansoapp.app";

/// A relying-party ID WebAuthn accepts: a lowercase DNS name with a dot, or
/// `localhost` for development. No scheme, port, path or IP literal. Pure.
pub fn valid_rp_id(rp_id: &str) -> bool {
    if rp_id == "localhost" {
        return true;
    }
    rp_id.len() <= 253
        && rp_id.contains('.')
        && rp_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
        && !rp_id.split('.').any(str::is_empty)
        && !rp_id
            .split('.')
            .all(|label| label.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `rp_id` is the host of `server` or a parent of it. Pure.
pub fn rp_id_fits_server(rp_id: &str, server: &str) -> bool {
    let Some(host) = url::Url::parse(server.trim())
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
    else {
        return false;
    };
    valid_rp_id(rp_id) && (host == rp_id || host.ends_with(&format!(".{rp_id}")))
}

/// Whether this client runs a passkey ceremony for `rp_id` when `server`
/// advertises it. [`OFFICIAL_RP_ID`] only from the owner's server: the
/// official Mac app's entitlement lets it sign for that relying party, so
/// another server advertising it could relay the owner's challenges and
/// have them signed. Any other relying party must be the server's host or a
/// parent of it. Pure.
pub fn passkey_rp_allowed(rp_id: &str, server: &str) -> bool {
    if rp_id == OFFICIAL_RP_ID {
        is_owner_server(server)
    } else {
        rp_id_fits_server(rp_id, server)
    }
}

/// Whether `origin` may go in the client data for `rp_id`: scheme, host and
/// port only, `https://` on the relying party or under it, or
/// `http://localhost` for the `localhost` relying party. Pure.
pub fn passkey_origin_allowed(origin: &str, rp_id: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let secure = match url.scheme() {
        "https" => true,
        "http" => rp_id == "localhost" && host == "localhost",
        _ => false,
    };
    secure
        && url.origin().ascii_serialization() == origin
        && valid_rp_id(rp_id)
        && (host == rp_id || host.ends_with(&format!(".{rp_id}")))
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

        /// Refinement: the relying-party rule. The official relying party
        /// only from a server that is the owner's; any other only from a
        /// server on it or under it; a label-boundary lookalike never.
        #[test]
        fn prop_passkey_rp_allowed(
            labels in prop::collection::vec("[a-z][a-z0-9]{0,6}", 1..3),
            sub in "[a-z]{1,6}",
            beside in "[a-z]{1,6}",
        ) {
            let rp_id = format!("{}.example", labels.join("."));
            let under = format!("https://{sub}.{rp_id}");
            let itself = format!("https://{rp_id}:8443/v2");
            let lookalike = format!("https://{beside}{rp_id}");
            prop_assert!(super::passkey_rp_allowed(&rp_id, &under));
            prop_assert!(super::passkey_rp_allowed(&rp_id, &itself));
            prop_assert!(!super::passkey_rp_allowed(&rp_id, &lookalike));
            prop_assert!(!super::passkey_rp_allowed(super::OFFICIAL_RP_ID, &under));
            let official_under = format!("https://{sub}.{}", super::OFFICIAL_RP_ID);
            prop_assert!(!super::passkey_rp_allowed(super::OFFICIAL_RP_ID, &official_under));
            prop_assert!(super::passkey_origin_allowed(&under, &rp_id));
            prop_assert!(!super::passkey_origin_allowed(&lookalike, &rp_id));
            let http_under = under.replacen("https", "http", 1);
            prop_assert!(!super::passkey_origin_allowed(&http_under, &rp_id));
            let with_path = format!("{under}/x");
            prop_assert!(!super::passkey_origin_allowed(&with_path, &rp_id));
        }
    }

    #[test]
    fn the_relying_party_rule() {
        use super::{OFFICIAL_RP_ID, passkey_origin_allowed, passkey_rp_allowed, valid_rp_id};
        assert!(passkey_rp_allowed(OFFICIAL_RP_ID, OWNER_ORIGIN));
        assert!(passkey_rp_allowed(
            OFFICIAL_RP_ID,
            "https://sync.kansoapp.app/"
        ));
        assert!(!passkey_rp_allowed(OFFICIAL_RP_ID, "https://evil.example"));
        assert!(!passkey_rp_allowed(
            OFFICIAL_RP_ID,
            "https://sync.kansoapp.app.evil.example"
        ));
        assert!(!passkey_rp_allowed("example.com", "https://ample.com"));
        assert!(!passkey_rp_allowed("ample.com", "https://example.com"));
        assert!(passkey_rp_allowed("localhost", "http://localhost:8787"));
        assert!(!passkey_rp_allowed("localhost", "http://127.0.0.1:8787"));
        assert!(!passkey_rp_allowed("127.0.0.1", "http://127.0.0.1:8787"));
        assert!(!passkey_rp_allowed("com", "https://example.com"));
        assert!(!valid_rp_id("Sync.Example.com"));
        assert!(!valid_rp_id("sync.example.com."));
        assert!(passkey_origin_allowed(
            "https://sync.kansoapp.app",
            OFFICIAL_RP_ID
        ));
        assert!(passkey_origin_allowed("http://localhost:8787", "localhost"));
        assert!(!passkey_origin_allowed(
            "http://sync.kansoapp.app",
            OFFICIAL_RP_ID
        ));
        assert!(!passkey_origin_allowed(
            "https://evil.example",
            OFFICIAL_RP_ID
        ));
        assert!(!passkey_origin_allowed(
            "https://user@sync.kansoapp.app",
            OFFICIAL_RP_ID
        ));
        assert!(!passkey_origin_allowed(
            "https://sync.kansoapp.app/",
            OFFICIAL_RP_ID
        ));
    }
}
