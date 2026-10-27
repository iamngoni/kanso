//! Where a passkey ceremony's answer comes back: from the native plugin
//! straight to Rust (`kanso_passkey_answer` in `mobile/rust`), never through
//! Dart, so the PRF output never reaches the shell.
//!
//! A ceremony the account flows need runs in two halves. [`NativeCeremony`]
//! is Rust's: it makes a ticket, hands the shell a [`PasskeyRequest`] (the
//! server's options, nothing secret) and waits. The shell shows the
//! platform's sheet through the native plugin, and the plugin calls
//! [`answer`] with the ticket and what the platform returned. If the shell
//! can't run the sheet it says so with `AccountHandle::passkey_failed`.
//!
//! Tickets live in one registry for the process: the native entry has no
//! library to name. A ticket takes one answer.
//!
//! # The JSON
//!
//! Everything binary is base64url without padding. `prf_input` is Kanso's
//! fixed PRF input ([`kanso_sync::PRF_INPUT`]); the platform hashes it again
//! itself, and it is `null` when no PRF output is wanted.
//!
//! - Register request: `{"rp_id","challenge","user_id","username",
//!   "display_name","exclude_credentials":[…],"prf_input"}`.
//! - Assert request: `{"rp_id","challenge","allow_credentials":[…],
//!   "prf_input"}`.
//! - Register answer: `{"credential_id","attestation_object",
//!   "client_data_json","transports":[…],"prf"}`.
//! - Assert answer: `{"credential_id","authenticator_data",
//!   "client_data_json","signature","user_handle","prf"}`.
//! - Failure: `{"error":"cancelled"|"unavailable"|"failed"}`.
//!
//! The answer's `prf` is read without a copy of its own and decoded into the
//! zeroizing value of `kanso-sync`; the caller zeroes the buffer it passed.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_sync::passkey::{
    PasskeyAssertion, PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError,
    PasskeyRegistration, PasskeyRegistrationChallenge,
};
use serde::Deserialize;
use zeroize::Zeroize;

use crate::account_model::{PasskeyFailure, PasskeyRequest, PasskeyRequestKind};
use crate::boundary::held;
use crate::stream::Listeners;

/// How long a ceremony waits for the platform's sheet.
pub const CEREMONY_LIMIT: Duration = Duration::from_secs(5 * 60);

/// How often a waiting ceremony looks at its cancel flag.
const SLICE: Duration = Duration::from_millis(50);

/// What the platform said, parsed.
enum Outcome {
    Registration(PasskeyRegistration),
    Assertion(PasskeyAssertion),
    Failed(PasskeyCeremonyError),
}

/// A ceremony waiting for its answer.
struct Slot {
    kind: PasskeyRequestKind,
    outcome: Mutex<Option<Outcome>>,
    ready: Condvar,
}

impl Slot {
    /// Takes `outcome` unless the ticket already has an answer.
    fn offer(&self, outcome: Outcome) -> bool {
        let mut slot = held(&self.outcome);
        if slot.is_some() {
            return false;
        }
        *slot = Some(outcome);
        self.ready.notify_all();
        true
    }
}

/// The tickets waiting for an answer, by number.
static WAITING: LazyLock<Mutex<HashMap<u64, Arc<Slot>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static NEXT_TICKET: AtomicU64 = AtomicU64::new(1);

/// Takes the ticket out of the registry when its ceremony is over, however
/// it ended, so a late answer finds nobody.
struct Ticket(u64);

impl Drop for Ticket {
    fn drop(&mut self) {
        held(&WAITING).remove(&self.0);
    }
}

/// Hands the native plugin's `json` answer to the ceremony waiting on
/// `ticket`. False for a ticket nobody waits on, or one that has its answer
/// already. An answer that can't be read still finishes its ticket: the
/// ceremony then fails.
pub fn answer(ticket: u64, json: &str) -> bool {
    let Some(slot) = held(&WAITING).get(&ticket).cloned() else {
        return false;
    };
    slot.offer(parse_answer(slot.kind, json))
}

/// The shell couldn't run the sheet for `ticket`. False for a ticket nobody
/// waits on.
pub(crate) fn fail(ticket: u64, failure: PasskeyFailure) -> bool {
    let Some(slot) = held(&WAITING).get(&ticket).cloned() else {
        return false;
    };
    slot.offer(Outcome::Failed(match failure {
        PasskeyFailure::Cancelled => PasskeyCeremonyError::Cancelled,
        PasskeyFailure::Unavailable => PasskeyCeremonyError::Unavailable,
        PasskeyFailure::Failed => PasskeyCeremonyError::Failed,
    }))
}

// ---------------------------------------------------------------- requests

/// `kanso-sync`'s ceremony through the shell's sheet. `sinks` are the
/// shell's `passkey_requests` streams; `cancel` ends the wait.
pub(crate) struct NativeCeremony<'a> {
    pub sinks: &'a Listeners<PasskeyRequest>,
    pub cancel: &'a AtomicBool,
    pub limit: Duration,
}

impl<'a> NativeCeremony<'a> {
    pub fn new(sinks: &'a Listeners<PasskeyRequest>, cancel: &'a AtomicBool) -> Self {
        NativeCeremony {
            sinks,
            cancel,
            limit: CEREMONY_LIMIT,
        }
    }

    /// Asks the shell to run `options_json` as a `kind` ceremony and waits
    /// for what the platform answers.
    fn run(
        &self,
        kind: PasskeyRequestKind,
        options_json: String,
    ) -> Result<Outcome, PasskeyCeremonyError> {
        // Nobody to show the sheet: don't wait for one.
        if self.sinks.is_empty() {
            return Err(PasskeyCeremonyError::Unavailable);
        }
        let ticket = NEXT_TICKET.fetch_add(1, Ordering::Relaxed);
        let slot = Arc::new(Slot {
            kind,
            outcome: Mutex::new(None),
            ready: Condvar::new(),
        });
        held(&WAITING).insert(ticket, slot.clone());
        let _registered = Ticket(ticket);
        self.sinks.send(PasskeyRequest {
            ticket,
            kind,
            options_json,
        });
        // Every listener said it is gone.
        if self.sinks.is_empty() {
            return Err(PasskeyCeremonyError::Unavailable);
        }
        let end = Instant::now() + self.limit;
        let mut outcome = held(&slot.outcome);
        loop {
            if let Some(done) = outcome.take() {
                return Ok(done);
            }
            if self.cancel.load(Ordering::Acquire) {
                return Err(PasskeyCeremonyError::Cancelled);
            }
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(PasskeyCeremonyError::Failed);
            }
            outcome = slot
                .ready
                .wait_timeout(outcome, left.min(SLICE))
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
    }
}

impl PasskeyCeremony for NativeCeremony<'_> {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
        match self.run(PasskeyRequestKind::Register, register_options(challenge))? {
            Outcome::Registration(registration) => Ok(registration),
            Outcome::Failed(error) => Err(error),
            Outcome::Assertion(_) => Err(PasskeyCeremonyError::Failed),
        }
    }

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
        match self.run(PasskeyRequestKind::Assert, assert_options(challenge))? {
            Outcome::Assertion(assertion) => Ok(assertion),
            Outcome::Failed(error) => Err(error),
            Outcome::Registration(_) => Err(PasskeyCeremonyError::Failed),
        }
    }
}

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn prf_input(wanted: bool) -> Option<String> {
    wanted.then(|| b64(&kanso_sync::PRF_INPUT))
}

/// The registration request the shell hands the platform. Nothing in it is
/// secret.
pub(crate) fn register_options(challenge: &PasskeyRegistrationChallenge) -> String {
    serde_json::json!({
        "rp_id": challenge.rp_id,
        "challenge": b64(&challenge.challenge),
        "user_id": b64(&challenge.user_handle),
        "username": challenge.username,
        "display_name": challenge.display_name,
        "exclude_credentials": challenge
            .exclude_credentials
            .iter()
            .map(|id| b64(id))
            .collect::<Vec<_>>(),
        "prf_input": prf_input(challenge.request_prf),
    })
    .to_string()
}

/// The assertion request the shell hands the platform.
pub(crate) fn assert_options(challenge: &PasskeyAssertionChallenge) -> String {
    serde_json::json!({
        "rp_id": challenge.rp_id,
        "challenge": b64(&challenge.challenge),
        "allow_credentials": challenge
            .allow_credentials
            .iter()
            .map(|id| b64(id))
            .collect::<Vec<_>>(),
        "prf_input": prf_input(challenge.request_prf),
    })
    .to_string()
}

// ----------------------------------------------------------------- answers

/// Every field either answer can carry; which are needed depends on the
/// ticket's kind. `prf` borrows from the text it came from.
#[derive(Deserialize)]
struct Wire<'a> {
    error: Option<String>,
    credential_id: Option<String>,
    attestation_object: Option<String>,
    authenticator_data: Option<String>,
    client_data_json: Option<String>,
    signature: Option<String>,
    user_handle: Option<String>,
    #[serde(default)]
    transports: Vec<String>,
    #[serde(borrow, default)]
    prf: Option<Cow<'a, str>>,
}

fn decode(text: Option<&String>) -> Result<Vec<u8>, PasskeyCeremonyError> {
    let text = text.ok_or(PasskeyCeremonyError::Failed)?;
    URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|_| PasskeyCeremonyError::Failed)
}

/// The 32-byte PRF output, decoded without a stray copy: into a buffer that
/// is wiped before this returns.
fn decode_prf(text: Option<&Cow<'_, str>>) -> Result<Option<[u8; 32]>, PasskeyCeremonyError> {
    let Some(text) = text else {
        return Ok(None);
    };
    let mut buffer = [0u8; 48];
    let written = URL_SAFE_NO_PAD
        .decode_slice(text.as_bytes(), &mut buffer)
        .map_err(|_| PasskeyCeremonyError::Failed);
    let prf = match written {
        Ok(32) => {
            let mut prf = [0u8; 32];
            prf.copy_from_slice(&buffer[..32]);
            Ok(Some(prf))
        }
        Ok(_) | Err(_) => Err(PasskeyCeremonyError::Failed),
    };
    buffer.zeroize();
    prf
}

/// What the platform's `json` says for a `kind` ceremony.
fn parse_answer(kind: PasskeyRequestKind, json: &str) -> Outcome {
    match read_answer(kind, json) {
        Ok(outcome) => outcome,
        Err(error) => Outcome::Failed(error),
    }
}

fn read_answer(kind: PasskeyRequestKind, json: &str) -> Result<Outcome, PasskeyCeremonyError> {
    let wire: Wire<'_> = serde_json::from_str(json).map_err(|_| PasskeyCeremonyError::Failed)?;
    if let Some(error) = &wire.error {
        return Ok(Outcome::Failed(match error.as_str() {
            "cancelled" => PasskeyCeremonyError::Cancelled,
            "unavailable" => PasskeyCeremonyError::Unavailable,
            _ => PasskeyCeremonyError::Failed,
        }));
    }
    let credential_id = decode(wire.credential_id.as_ref())?;
    if credential_id.is_empty() {
        return Err(PasskeyCeremonyError::Failed);
    }
    let client_data_json = decode(wire.client_data_json.as_ref())?;
    let prf = decode_prf(wire.prf.as_ref())?;
    match kind {
        PasskeyRequestKind::Register => Ok(Outcome::Registration(PasskeyRegistration {
            credential_id,
            attestation_object: decode(wire.attestation_object.as_ref())?,
            client_data_json,
            transports: wire.transports,
            prf,
        })),
        PasskeyRequestKind::Assert => Ok(Outcome::Assertion(PasskeyAssertion {
            credential_id,
            authenticator_data: decode(wire.authenticator_data.as_ref())?,
            client_data_json,
            signature: decode(wire.signature.as_ref())?,
            user_handle: wire
                .user_handle
                .as_ref()
                .map(|handle| decode(Some(handle)))
                .transpose()?,
            prf,
        })),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use kanso_sync::TestAuthenticator;
    use proptest::prelude::*;

    use super::*;

    fn registration_challenge(request_prf: bool) -> PasskeyRegistrationChallenge {
        PasskeyRegistrationChallenge {
            rp_id: kanso_sync::MEMORY_PASSKEY_RP_ID.into(),
            origin: kanso_sync::MEMORY_PASSKEY_ORIGIN.into(),
            challenge: vec![7; 32],
            user_handle: vec![1; 16],
            username: "ada".into(),
            display_name: "ada".into(),
            exclude_credentials: vec![vec![9; 8]],
            request_prf,
        }
    }

    /// A ticket with nobody waiting on it, for the registry's rules.
    fn open_ticket(kind: PasskeyRequestKind) -> (u64, Ticket, Arc<Slot>) {
        let ticket = NEXT_TICKET.fetch_add(1, Ordering::Relaxed);
        let slot = Arc::new(Slot {
            kind,
            outcome: Mutex::new(None),
            ready: Condvar::new(),
        });
        held(&WAITING).insert(ticket, slot.clone());
        (ticket, Ticket(ticket), slot)
    }

    fn registration_json(credential: &[u8], prf: Option<&[u8; 32]>) -> String {
        serde_json::json!({
            "credential_id": b64(credential),
            "attestation_object": b64(b"attestation"),
            "client_data_json": b64(b"{\"type\":\"webauthn.create\"}"),
            "transports": ["internal", "hybrid"],
            "prf": prf.map(|prf| b64(prf)),
        })
        .to_string()
    }

    fn assertion_json(credential: &[u8], handle: Option<&[u8]>, prf: Option<&[u8; 32]>) -> String {
        serde_json::json!({
            "credential_id": b64(credential),
            "authenticator_data": b64(b"auth data"),
            "client_data_json": b64(b"{\"type\":\"webauthn.get\"}"),
            "signature": b64(b"signature"),
            "user_handle": handle.map(b64),
            "prf": prf.map(|prf| b64(prf)),
        })
        .to_string()
    }

    proptest! {
        /// A registration answer parses back to what the platform made, for
        /// any bytes, with or without a PRF output.
        #[test]
        fn prop_a_registration_answer_round_trips(
            credential in proptest::collection::vec(any::<u8>(), 1..64),
            prf in proptest::option::of(any::<[u8; 32]>()),
        ) {
            let Outcome::Registration(parsed) = parse_answer(
                PasskeyRequestKind::Register,
                &registration_json(&credential, prf.as_ref()),
            ) else {
                return Err(TestCaseError::fail("a registration"));
            };
            prop_assert_eq!(&parsed.credential_id, &credential);
            prop_assert_eq!(parsed.attestation_object.as_slice(), b"attestation".as_slice());
            prop_assert_eq!(&parsed.transports, &["internal".to_owned(), "hybrid".to_owned()]);
            prop_assert_eq!(parsed.prf, prf);
        }

        /// An assertion answer parses back, with the user handle if the
        /// platform returned one.
        #[test]
        fn prop_an_assertion_answer_round_trips(
            credential in proptest::collection::vec(any::<u8>(), 1..64),
            handle in proptest::option::of(proptest::collection::vec(any::<u8>(), 0..32)),
            prf in proptest::option::of(any::<[u8; 32]>()),
        ) {
            let Outcome::Assertion(parsed) = parse_answer(
                PasskeyRequestKind::Assert,
                &assertion_json(&credential, handle.as_deref(), prf.as_ref()),
            ) else {
                return Err(TestCaseError::fail("an assertion"));
            };
            prop_assert_eq!(&parsed.credential_id, &credential);
            prop_assert_eq!(&parsed.user_handle, &handle);
            prop_assert_eq!(parsed.signature.as_slice(), b"signature".as_slice());
            prop_assert_eq!(parsed.prf, prf);
        }

        /// A PRF output that isn't exactly 32 bytes, or isn't base64url,
        /// fails the ceremony; it is never padded or cut.
        #[test]
        fn prop_a_prf_of_the_wrong_size_fails_the_ceremony(
            prf in proptest::collection::vec(any::<u8>(), 0..64)
                .prop_filter("not 32", |prf| prf.len() != 32),
        ) {
            let json = serde_json::json!({
                "credential_id": b64(b"id"),
                "attestation_object": b64(b"a"),
                "client_data_json": b64(b"c"),
                "prf": b64(&prf),
            })
            .to_string();
            prop_assert!(matches!(
                parse_answer(PasskeyRequestKind::Register, &json),
                Outcome::Failed(PasskeyCeremonyError::Failed)
            ));
        }

        /// Whatever text arrives, parsing never panics and never yields an
        /// answer of the other kind.
        #[test]
        fn prop_garbage_is_a_failed_ceremony(text in "\\PC{0,200}") {
            let outcome = parse_answer(PasskeyRequestKind::Assert, &text);
            prop_assert!(!matches!(outcome, Outcome::Registration(_)));
        }

        /// An answer reaches only its own ticket, and a second answer for it
        /// is refused whatever it says.
        #[test]
        fn prop_an_answer_reaches_only_its_ticket_and_only_once(
            first in proptest::collection::vec(any::<u8>(), 1..16),
            second in proptest::collection::vec(any::<u8>(), 1..16),
        ) {
            let (mine, _mine_guard, mine_slot) = open_ticket(PasskeyRequestKind::Register);
            let (other, _other_guard, other_slot) = open_ticket(PasskeyRequestKind::Register);
            prop_assert!(answer(mine, &registration_json(&first, None)));
            prop_assert!(!answer(mine, &registration_json(&second, None)));
            prop_assert!(held(&other_slot.outcome).is_none(), "the other ticket heard nothing");
            let Some(Outcome::Registration(got)) = held(&mine_slot.outcome).take() else {
                return Err(TestCaseError::fail("its answer"));
            };
            prop_assert_eq!(&got.credential_id, &first);
            prop_assert!(answer(other, &registration_json(&second, None)));
        }
    }

    // no-prop-test: the shapes of the failures a plugin can name.
    #[test]
    fn a_named_failure_finishes_the_ticket_with_that_failure() {
        for (name, expected) in [
            ("cancelled", PasskeyCeremonyError::Cancelled),
            ("unavailable", PasskeyCeremonyError::Unavailable),
            ("failed", PasskeyCeremonyError::Failed),
            ("something else", PasskeyCeremonyError::Failed),
        ] {
            let json = serde_json::json!({ "error": name }).to_string();
            let Outcome::Failed(got) = parse_answer(PasskeyRequestKind::Assert, &json) else {
                panic!("a failure");
            };
            assert_eq!(got, expected, "{name}");
        }
    }

    // no-prop-test: unknown, finished and cleared tickets.
    #[test]
    fn a_ticket_nobody_waits_on_takes_no_answer() {
        assert!(!answer(u64::MAX, "{}"));
        assert!(!fail(u64::MAX, PasskeyFailure::Cancelled));
        let (ticket, guard, _slot) = open_ticket(PasskeyRequestKind::Assert);
        assert!(fail(ticket, PasskeyFailure::Unavailable));
        assert!(
            !fail(ticket, PasskeyFailure::Cancelled),
            "a second one is refused"
        );
        drop(guard);
        assert!(!answer(ticket, "{}"), "a cleared ticket is unknown");
    }

    // no-prop-test: the request shapes are a fixed contract with the native
    // plugin; this reads them back field by field.
    #[test]
    fn the_requests_carry_the_options_the_plugin_needs_and_nothing_secret() {
        let json: serde_json::Value =
            serde_json::from_str(&register_options(&registration_challenge(true))).unwrap();
        assert_eq!(json["rp_id"], kanso_sync::MEMORY_PASSKEY_RP_ID);
        assert_eq!(json["challenge"], b64(&[7; 32]));
        assert_eq!(json["user_id"], b64(&[1; 16]));
        assert_eq!(json["username"], "ada");
        assert_eq!(json["display_name"], "ada");
        assert_eq!(
            json["exclude_credentials"],
            serde_json::json!([b64(&[9; 8])])
        );
        assert_eq!(json["prf_input"], b64(&kanso_sync::PRF_INPUT));

        let json: serde_json::Value =
            serde_json::from_str(&register_options(&registration_challenge(false))).unwrap();
        assert!(json["prf_input"].is_null());

        let json: serde_json::Value =
            serde_json::from_str(&assert_options(&PasskeyAssertionChallenge {
                rp_id: "rp.test".into(),
                origin: "https://rp.test".into(),
                challenge: vec![5; 32],
                allow_credentials: vec![vec![3; 4], vec![4; 4]],
                request_prf: true,
            }))
            .unwrap();
        assert_eq!(json["allow_credentials"].as_array().unwrap().len(), 2);
        assert_eq!(json["prf_input"], b64(&kanso_sync::PRF_INPUT));
        assert!(json.get("origin").is_none());
    }

    // no-prop-test: a ceremony with nobody to show the sheet, one the shell
    // answers, one nobody answers, and one cancelled.
    #[test]
    fn a_ceremony_waits_for_its_answer_and_gives_up_cleanly() {
        let cancel = AtomicBool::new(false);
        let sinks = Listeners::<PasskeyRequest>::default();
        let ceremony = NativeCeremony::new(&sinks, &cancel);
        assert_eq!(
            ceremony
                .register(&registration_challenge(true))
                .unwrap_err(),
            PasskeyCeremonyError::Unavailable,
            "no listener: no wait"
        );

        // A listener that plays the platform.
        let key = Arc::new(TestAuthenticator::new(4));
        let plays = key.clone();
        sinks.add(move |request: PasskeyRequest| {
            assert_eq!(request.kind, PasskeyRequestKind::Register);
            let ticket = request.ticket;
            let challenge = registration_challenge(true);
            let plays = plays.clone();
            std::thread::spawn(move || {
                let made = plays.register(&challenge).unwrap();
                let json = registration_json(&made.credential_id, made.prf());
                assert!(answer(ticket, &json));
            });
            true
        });
        let made = ceremony.register(&registration_challenge(true)).unwrap();
        assert_eq!(made.credential_id, key.credential_id());
        assert_eq!(made.prf(), Some(&key.prf_output()));

        // The sheet never answers: the wait ends at its limit, and the
        // ticket is gone.
        let quiet = Listeners::<PasskeyRequest>::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        quiet.add(move |request: PasskeyRequest| {
            log.lock().unwrap().push(request.ticket);
            true
        });
        let mut slow = NativeCeremony::new(&quiet, &cancel);
        slow.limit = Duration::from_millis(120);
        assert_eq!(
            slow.register(&registration_challenge(true)).unwrap_err(),
            PasskeyCeremonyError::Failed
        );
        let ticket = seen.lock().unwrap()[0];
        assert!(
            !answer(ticket, &registration_json(b"late", None)),
            "too late"
        );

        // Cancelled from outside.
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(80));
            flag.store(true, Ordering::Release);
        });
        let patient = NativeCeremony::new(&quiet, &stop);
        assert_eq!(
            patient.register(&registration_challenge(true)).unwrap_err(),
            PasskeyCeremonyError::Cancelled
        );
        canceller.join().unwrap();
    }

    // no-prop-test: every listener leaving is the same as none.
    #[test]
    fn listeners_that_all_leave_make_the_ceremony_unavailable() {
        let cancel = AtomicBool::new(false);
        let sinks = Listeners::<PasskeyRequest>::default();
        sinks.add(|_| false);
        let ceremony = NativeCeremony::new(&sinks, &cancel);
        assert_eq!(
            ceremony
                .register(&registration_challenge(true))
                .unwrap_err(),
            PasskeyCeremonyError::Unavailable
        );
    }
}
