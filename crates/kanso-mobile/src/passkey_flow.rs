//! The account half of a passkey sign-in or sign-up on the phone
//! (docs/ACCOUNTS.md › Passkeys, › Account flows › Sign in with a passkey,
//! › Sign up with a passkey): the desktop's `passkey_accounts`, without the
//! seam the desktop's tests use.
//!
//! The ceremony is the platform's sheet, through
//! [`crate::passkey_bridge::NativeCeremony`]. Everything around it (the
//! server's challenge, the assertion sent back, the PRF unlock, a new
//! account's keys) is `kanso-sync`'s. Between the start and the ceremony the
//! relying party is checked against the phone's rule, so a sheet is never
//! shown for a relying party this app may not use.

use kanso_engine::account::Failure;
use kanso_engine::account::words::{self, ErrorContext};
use kanso_sync::passkey::{PasskeyCeremony, PasskeyCeremonyError};
use kanso_sync::{
    AccountError, Backend, DeviceIdentity, PasskeySignIn, RotationError, SecretStore, SignedUp,
    Step,
};

use crate::errors::KansoError;

/// Which relying parties a ceremony may be run for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RpRule {
    /// The phones: only the relying party in the app's entitlement (iOS
    /// Associated Domains, Android's `assetlinks.json`), and only from the
    /// owner's server. Another server advertising it could relay the owner's
    /// challenges and have the app sign them.
    Entitlement,
    /// A computer, which runs the facade's tests: the server's host or a
    /// parent of it.
    ServerHost,
}

impl RpRule {
    /// The rule of the platform this is built for.
    pub const fn platform() -> RpRule {
        if cfg!(any(target_os = "ios", target_os = "android")) {
            RpRule::Entitlement
        } else {
            RpRule::ServerHost
        }
    }

    pub fn allows(self, rp_id: &str, server: &str) -> bool {
        match self {
            RpRule::Entitlement => {
                rp_id == kanso_sync::OFFICIAL_RP_ID && kanso_sync::is_owner_server(server)
            }
            RpRule::ServerHost => kanso_sync::passkey_rp_allowed(rp_id, server),
        }
    }

    /// Whether the relying party the server named, and the origin it was
    /// started with, are ones this ceremony may run for.
    pub fn allows_ceremony(self, rp_id: &str, origin: &str, server: &str) -> bool {
        self.allows(rp_id, server) && kanso_sync::passkey_origin_allowed(origin, rp_id)
    }
}

/// What a sign-in or sign-up needs besides its ceremony.
pub(crate) struct Context<'a> {
    pub backend: &'a dyn Backend,
    pub secrets: &'a dyn SecretStore,
    pub server: &'a str,
    pub device: &'a DeviceIdentity,
    pub rule: RpRule,
    pub progress: &'a dyn Fn(Step),
}

/// Why a passkey sign-in or sign-up didn't finish.
#[derive(Debug)]
pub(crate) enum PasskeyError {
    Account(AccountError),
    /// A sign-up's request failed; its recovery key may need showing.
    Rotation(RotationError),
    Ceremony(PasskeyCeremonyError),
    /// The server's relying party isn't one this app may use for it.
    WrongRelyingParty,
}

impl From<AccountError> for PasskeyError {
    fn from(error: AccountError) -> Self {
        PasskeyError::Account(error)
    }
}

impl From<RotationError> for PasskeyError {
    fn from(error: RotationError) -> Self {
        PasskeyError::Rotation(error)
    }
}

/// How a passkey task ended without an answer.
pub(crate) enum Ended {
    /// The sheet was dismissed: nothing changed, and nothing is an error.
    Cancelled,
    /// The task's own failure, to settle as any account failure.
    Failed(Failure),
    /// The shell's error.
    Error(KansoError),
}

/// What a passkey task's error is to the shell, in the desktop's words.
pub(crate) fn ended(error: PasskeyError, context: ErrorContext) -> Ended {
    match error {
        PasskeyError::Account(err) => Ended::Failed(Failure::account(err, context)),
        PasskeyError::Rotation(err) => Ended::Failed(Failure::rotation(
            err,
            ErrorContext::SignUp,
            words::KEY_UNCONFIRMED_PASSKEY_SIGN_UP,
        )),
        PasskeyError::Ceremony(PasskeyCeremonyError::Cancelled) => Ended::Cancelled,
        PasskeyError::Ceremony(PasskeyCeremonyError::Unavailable) => {
            Ended::Error(KansoError::invalid(words::PASSKEY_UNAVAILABLE))
        }
        PasskeyError::Ceremony(PasskeyCeremonyError::Failed) => {
            Ended::Error(KansoError::invalid(words::PASSKEY_FAILED))
        }
        PasskeyError::WrongRelyingParty => {
            Ended::Error(KansoError::invalid(words::PASSKEY_WRONG_SERVER))
        }
    }
}

/// Signs in with a passkey. No username is given: the platform's picker
/// lists the credentials. The flow is bound to a new ephemeral device,
/// because before the assertion the library doesn't know which account it
/// is for.
pub(crate) fn sign_in(
    cx: &Context<'_>,
    ceremony: &dyn PasskeyCeremony,
) -> Result<PasskeySignIn, PasskeyError> {
    (cx.progress)(Step::Contacting);
    let login_device = kanso_sync::api::LoginDevice::ephemeral();
    let start = kanso_sync::start_passkey_sign_in(cx.backend, cx.server, None, &login_device)?;
    if !cx.rule.allows_ceremony(
        &start.challenge.rp_id,
        &start.challenge.origin,
        &start.server,
    ) {
        return Err(PasskeyError::WrongRelyingParty);
    }
    let assertion = ceremony
        .assert(&start.challenge)
        .map_err(PasskeyError::Ceremony)?;
    Ok(kanso_sync::sign_in_with_passkey(
        cx.backend,
        cx.secrets,
        &start,
        cx.device,
        assertion,
        cx.progress,
    )?)
}

/// Creates an account whose first way in is a new passkey: the account, and
/// whether the credential can unlock a new device (it answered PRF).
pub(crate) fn sign_up(
    cx: &Context<'_>,
    username: &str,
    ceremony: &dyn PasskeyCeremony,
) -> Result<(SignedUp, bool), PasskeyError> {
    (cx.progress)(Step::Contacting);
    let start = kanso_sync::start_passkey_sign_up(cx.backend, cx.server, username, cx.device)?;
    if !cx.rule.allows_ceremony(
        &start.challenge.rp_id,
        &start.challenge.origin,
        &start.server,
    ) {
        return Err(PasskeyError::WrongRelyingParty);
    }
    let registration = ceremony
        .register(&start.challenge)
        .map_err(PasskeyError::Ceremony)?;
    let can_unlock = registration.prf().is_some();
    let signed = kanso_sync::sign_up_with_passkey(
        cx.backend,
        cx.secrets,
        &start,
        cx.device,
        registration,
        cx.progress,
    )?;
    Ok((signed, can_unlock))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const OWNER: &str = "https://sync.kansoapp.app";

    proptest! {
        /// The phones' rule lets exactly one relying party through, and only
        /// from the owner's server, whatever else a server advertises.
        #[test]
        fn prop_the_entitlement_rule_allows_only_the_owners_relying_party(
            rp in "[a-z]{1,10}(\\.[a-z]{1,10}){1,2}",
            host in "[a-z]{1,10}\\.[a-z]{2,6}",
        ) {
            let server = format!("https://{host}");
            prop_assert!(!RpRule::Entitlement.allows(&rp, &server) || (
                rp == kanso_sync::OFFICIAL_RP_ID && kanso_sync::is_owner_server(&server)
            ));
            prop_assert!(!RpRule::Entitlement.allows(kanso_sync::OFFICIAL_RP_ID, &server)
                || kanso_sync::is_owner_server(&server));
        }
    }

    // no-prop-test: a table of who may ask for which relying party.
    #[test]
    fn the_phones_rule_is_the_owners_relying_party_from_the_owners_server() {
        let phone = RpRule::Entitlement;
        assert!(phone.allows(kanso_sync::OFFICIAL_RP_ID, OWNER));
        assert!(!phone.allows(kanso_sync::OFFICIAL_RP_ID, "https://evil.example"));
        assert!(!phone.allows("evil.example", "https://evil.example"));
        assert!(
            !phone.allows("kansoapp.app", OWNER),
            "a parent is not the entitlement"
        );
        // The computers' rule, which the tests run: the server's host or a
        // parent of it, and the owner's relying party only from the owner.
        let host = RpRule::ServerHost;
        assert!(host.allows("memory.kanso.test", "https://memory.kanso.test"));
        assert!(host.allows("kanso.test", "https://memory.kanso.test"));
        assert!(!host.allows("other.test", "https://memory.kanso.test"));
        assert!(!host.allows(kanso_sync::OFFICIAL_RP_ID, "https://evil.example"));
        // The origin must be on the relying party.
        assert!(phone.allows_ceremony(kanso_sync::OFFICIAL_RP_ID, OWNER, OWNER));
        assert!(!phone.allows_ceremony(kanso_sync::OFFICIAL_RP_ID, "https://evil.example", OWNER));
    }

    // no-prop-test: the words of each way a passkey task can end.
    #[test]
    fn each_ending_has_the_desktops_words() {
        assert!(matches!(
            ended(
                PasskeyError::Ceremony(PasskeyCeremonyError::Cancelled),
                ErrorContext::Passkey
            ),
            Ended::Cancelled
        ));
        for (error, words) in [
            (
                PasskeyCeremonyError::Unavailable,
                words::PASSKEY_UNAVAILABLE,
            ),
            (PasskeyCeremonyError::Failed, words::PASSKEY_FAILED),
        ] {
            let Ended::Error(KansoError::Invalid { message }) =
                ended(PasskeyError::Ceremony(error), ErrorContext::Passkey)
            else {
                panic!("an invalid error");
            };
            assert_eq!(message, words);
        }
        let Ended::Error(KansoError::Invalid { message }) =
            ended(PasskeyError::WrongRelyingParty, ErrorContext::Passkey)
        else {
            panic!("an invalid error");
        };
        assert_eq!(message, words::PASSKEY_WRONG_SERVER);
        assert!(matches!(
            ended(
                PasskeyError::Account(AccountError::InvalidCredentials),
                ErrorContext::Passkey
            ),
            Ended::Failed(_)
        ));
    }
}
