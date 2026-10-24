//! Provider sign-in and the account's ways in (docs/ACCOUNTS.md › Sign in
//! with Apple, Google and GitHub, › Client: account flows). The Workflow's
//! own routes are covered end to end against the Worker in
//! `cloud/test/providers.test.ts`; here the client's pieces are checked
//! against a stub [`Api`], so no provider is ever contacted and no token
//! exists.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use kanso_sync::api::{
    Api, Backend, Health, HealthAuth, HealthPasskeys, LoginDevice, SessionResponse,
};
use kanso_sync::provider::{
    GoogleFinishRequest, GoogleStartRequest, IdentityIntent, IdentityStartRequest, LinkOutcome,
    MethodsReply, NewAccount, ProofWire, Provider, ProviderPollRequest, ProviderSession,
    ProviderSignIn, ProviderStart, ProviderStartRequest, ProviderStep, SocialSignupRequest,
    UnlinkRequest,
};
use kanso_sync::wire::{ApiError, SessionToken};
use kanso_sync::{
    AccountError, DeviceIdentity, KdfParams, Loopback, MemorySecrets, Session, SignupState, Step,
};

const SERVER: &str = "http://127.0.0.1:8787";

/// What the stub answers next, and what it was asked.
#[derive(Default)]
struct Seen {
    starts: Vec<String>,
    start_devices: Vec<LoginDevice>,
    polls: Vec<(String, String)>,
    finishes: Vec<(String, String, String)>,
    social: Vec<SocialSignupRequest>,
    unlinks: Vec<(String, String)>,
    identity_starts: Vec<(String, String)>,
}

struct Stub {
    seen: Mutex<Seen>,
    step: Mutex<ProviderStep>,
    link: Mutex<LinkOutcome>,
    reject_social_once: Mutex<bool>,
    auth: Option<HealthAuth>,
}

impl Stub {
    fn new() -> Arc<Stub> {
        Arc::new(Stub {
            seen: Mutex::new(Seen::default()),
            step: Mutex::new(ProviderStep::Session(provider_session("alice"))),
            link: Mutex::new(LinkOutcome::Linked {
                provider: Provider::Github,
                created_at: 1_800_000_000_000,
            }),
            reject_social_once: Mutex::new(false),
            auth: Some(HealthAuth {
                password: true,
                passkeys: Some(HealthPasskeys {
                    rp_id: "sync.kanso.test".into(),
                    origins: vec!["https://sync.kanso.test".into()],
                    security_key: true,
                }),
                providers: vec![Provider::Apple, Provider::Google, Provider::Github],
            }),
        })
    }
}

fn provider_session(username: &str) -> ProviderSession {
    ProviderSession {
        account_id: "a_0123456789abcdef0123456789abcdef".into(),
        username: username.into(),
        token: SessionToken::new(format!("kanso_{}", "A".repeat(43))),
        key_id: "0123456789abcdef".into(),
        epoch: "0000000000000000".into(),
        has_password: false,
        passkey_wrapped_key: None,
        evicted_device: None,
    }
}

struct StubApi {
    stub: Arc<Stub>,
    auth: Option<HealthAuth>,
}

impl Api for StubApi {
    fn health(&self) -> Result<Health, ApiError> {
        Ok(Health {
            ok: true,
            protocol: 2,
            signup: SignupState::Open,
            auth: self.auth.clone(),
        })
    }

    fn provider_start(
        &self,
        provider: &Provider,
        request: &ProviderStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        self.stub
            .seen
            .lock()
            .unwrap()
            .starts
            .push(provider.to_string());
        self.stub
            .seen
            .lock()
            .unwrap()
            .start_devices
            .push(request.device.clone());
        Ok(ProviderStart::Apple(kanso_sync::AppleStart {
            flow_id: "A".repeat(43),
            auth_url: format!(
                "https://appleid.apple.com/auth/authorize?device={}",
                request.device.id
            ),
            expires_at: 1_800_000_600_000,
        }))
    }

    fn google_start(&self, request: &GoogleStartRequest) -> Result<ProviderStart, ApiError> {
        self.stub.seen.lock().unwrap().starts.push("google".into());
        Ok(ProviderStart::Google(kanso_sync::GoogleStart {
            flow_id: "B".repeat(43),
            auth_url: format!("https://accounts.google.com/o/oauth2/v2/auth?{request:?}"),
            nonce: "n".repeat(43),
            state: "s".repeat(43),
            expires_at: 1_800_000_600_000,
        }))
    }

    fn provider_poll(
        &self,
        provider: &Provider,
        request: &ProviderPollRequest,
    ) -> Result<ProviderStep, ApiError> {
        self.stub
            .seen
            .lock()
            .unwrap()
            .polls
            .push((provider.to_string(), request.flow_id.clone()));
        Ok(self.stub.step.lock().unwrap().clone())
    }

    fn google_finish(&self, request: &GoogleFinishRequest) -> Result<ProviderStep, ApiError> {
        self.stub.seen.lock().unwrap().finishes.push((
            request.flow_id.clone(),
            request.code.clone(),
            request.code_verifier.clone(),
        ));
        Ok(self.stub.step.lock().unwrap().clone())
    }

    fn social_signup(&self, request: &SocialSignupRequest) -> Result<ProviderSession, ApiError> {
        self.stub.seen.lock().unwrap().social.push(request.clone());
        let reject = std::mem::take(&mut *self.stub.reject_social_once.lock().unwrap());
        if reject {
            return Err(ApiError::AccountIdTaken);
        }
        Ok(ProviderSession {
            account_id: request.account_id.clone(),
            username: request.username.clone(),
            token: SessionToken::new(format!("kanso_{}", "A".repeat(43))),
            key_id: request.key_id.clone(),
            epoch: "0000000000000000".into(),
            has_password: false,
            passkey_wrapped_key: None,
            evicted_device: None,
        })
    }

    fn sign_in_methods(&self, _token: &SessionToken) -> Result<MethodsReply, ApiError> {
        Ok(MethodsReply {
            username: "alice".into(),
            has_password: false,
            identities: vec![kanso_sync::provider::IdentityRow {
                provider: Provider::Github,
                created_at: 1_800_000_000_000,
            }],
            passkeys: vec![kanso_sync::provider::PasskeyRow {
                credential_id: "AQID".into(),
                created_at: 1_800_000_000_000,
                last_used_at: None,
                prf: true,
                transports: vec!["internal".into()],
            }],
            recovery: true,
        })
    }

    fn identity_start(
        &self,
        _token: &SessionToken,
        provider: &Provider,
        request: &IdentityStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        self.stub
            .seen
            .lock()
            .unwrap()
            .identity_starts
            .push((provider.to_string(), format!("{:?}", request.intent)));
        Ok(ProviderStart::Github(kanso_sync::GithubStart {
            flow_id: "C".repeat(43),
            user_code: "ABCD-EFGH".into(),
            verification_uri: "https://github.com/login/device".into(),
            interval: 5,
            expires_in: 600,
        }))
    }

    fn identity_poll(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &ProviderPollRequest,
    ) -> Result<LinkOutcome, ApiError> {
        Ok(self.stub.link.lock().unwrap().clone())
    }

    fn identity_finish_google(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &GoogleFinishRequest,
    ) -> Result<LinkOutcome, ApiError> {
        Ok(self.stub.link.lock().unwrap().clone())
    }

    fn unlink_identity(
        &self,
        _token: &SessionToken,
        provider: &Provider,
        request: &UnlinkRequest,
    ) -> Result<(), ApiError> {
        self.stub
            .seen
            .lock()
            .unwrap()
            .unlinks
            .push((provider.to_string(), format!("{:?}", request.proof)));
        Ok(())
    }

    fn prelogin(
        &self,
        _request: &kanso_sync::api::PreloginRequest,
    ) -> Result<kanso_sync::api::PreloginResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn signup(
        &self,
        _request: &kanso_sync::api::SignupRequest,
    ) -> Result<SessionResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn login(
        &self,
        _request: &kanso_sync::api::LoginRequest,
    ) -> Result<kanso_sync::api::LoginResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn recover_start(
        &self,
        _request: &kanso_sync::api::RecoverStartRequest,
    ) -> Result<kanso_sync::api::RecoverStartResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn recover_finish(
        &self,
        _request: &kanso_sync::api::RecoverFinishRequest,
    ) -> Result<SessionResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn logout(&self, _token: &SessionToken) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }

    fn account(&self, _token: &SessionToken) -> Result<kanso_sync::api::AccountResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn change_password(
        &self,
        _token: &SessionToken,
        _request: &kanso_sync::api::PasswordRequest,
    ) -> Result<kanso_sync::api::PasswordResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn replace_recovery_key(
        &self,
        _token: &SessionToken,
        _request: &kanso_sync::api::RecoveryKeyRequest,
    ) -> Result<kanso_sync::api::RecoveryKeyResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn delete_account(
        &self,
        _token: &SessionToken,
        _request: &kanso_sync::api::DeleteRequest,
    ) -> Result<kanso_sync::api::DeleteResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn set_device_name(
        &self,
        _token: &SessionToken,
        _request: &kanso_sync::api::DeviceNameRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }

    fn devices(&self, _token: &SessionToken) -> Result<kanso_sync::api::DevicesResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn revoke_device(&self, _token: &SessionToken, _device_id: &str) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }

    fn push(
        &self,
        _token: &SessionToken,
        _request: &kanso_sync::wire::PushRequest,
    ) -> Result<kanso_sync::wire::PushResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn changes(
        &self,
        _token: &SessionToken,
        _since: Option<i64>,
        _limit: Option<usize>,
    ) -> Result<kanso_sync::wire::WirePage, ApiError> {
        Err(ApiError::NotFound)
    }

    fn put_file(
        &self,
        _token: &SessionToken,
        _wire_id: &str,
        _sealed: &[u8],
    ) -> Result<kanso_sync::wire::PutFileResponse, ApiError> {
        Err(ApiError::NotFound)
    }

    fn get_file(&self, _token: &SessionToken, _wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        Err(ApiError::NotFound)
    }
}

struct StubBackend(Arc<Stub>);

impl Backend for StubBackend {
    fn api(&self, _server: &str) -> Box<dyn Api + '_> {
        Box::new(StubApi {
            stub: self.0.clone(),
            auth: self.0.auth.clone(),
        })
    }

    fn kdf_floor(&self) -> KdfParams {
        KdfParams::TEST
    }

    fn kdf_default(&self) -> KdfParams {
        KdfParams::TEST
    }

    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

fn device() -> LoginDevice {
    LoginDevice {
        id: "d_0123456789abcdef0123456789abcdef".into(),
        proof: "A".repeat(43),
    }
}

fn identity() -> DeviceIdentity {
    DeviceIdentity::from_parts("l_0123456789abcdef0123456789abcdef", "Alex's Mac", [7; 32])
}

/// A session from a memory sign-up, so `methods` has a real token.
fn a_session() -> Session {
    let server = kanso_sync::MemoryServer::new();
    let secrets = MemorySecrets::new();
    let password = kanso_sync::Password::new("correct horse battery staple", "alice").unwrap();
    kanso_sync::sign_up(
        &server,
        &secrets,
        SERVER,
        "alice",
        &password,
        &identity(),
        &|_| {},
    )
    .unwrap()
    .session
}

#[test]
fn a_start_is_the_providers_shape_and_google_needs_the_loopback() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let apple =
        kanso_sync::start_provider(&backend, SERVER, &Provider::Apple, &device(), None, None)
            .unwrap();
    assert!(matches!(apple, ProviderStart::Apple(_)));
    assert_eq!(apple.flow_id(), "A".repeat(43).as_str());

    let loopback = Loopback::parse("http://127.0.0.1:49152").unwrap();
    let google = kanso_sync::start_provider(
        &backend,
        SERVER,
        &Provider::Google,
        &device(),
        Some(&loopback),
        Some("challenge"),
    )
    .unwrap();
    assert!(matches!(google, ProviderStart::Google(_)));
    assert!(matches!(
        kanso_sync::start_provider(&backend, SERVER, &Provider::Google, &device(), None, None),
        Err(AccountError::BadRequest)
    ));
}

#[test]
fn a_poll_returns_a_session_or_a_ticket_and_google_finishes_with_the_verifier() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let session = kanso_sync::sign_in_with_provider(
        &backend,
        SERVER,
        &Provider::Apple,
        &"A".repeat(43),
        &device(),
        &|_| {},
    )
    .unwrap();
    assert!(matches!(session, ProviderSignIn::Session(_)));

    *stub.step.lock().unwrap() = ProviderStep::NewAccount(NewAccount {
        ticket: "T".into(),
        suggested_username: "quiet-otter-4821".into(),
    });
    assert_eq!(
        kanso_sync::sign_in_with_provider(
            &backend,
            SERVER,
            &Provider::Apple,
            &"A".repeat(43),
            &device(),
            &|_| {}
        )
        .unwrap(),
        ProviderSignIn::NewAccount(NewAccount {
            ticket: "T".into(),
            suggested_username: "quiet-otter-4821".into()
        })
    );

    *stub.step.lock().unwrap() = ProviderStep::Pending { slow_down: false };
    assert!(matches!(
        kanso_sync::poll_provider(&backend, SERVER, &Provider::Github, "F", &device()).unwrap(),
        ProviderStep::Pending { .. }
    ));
    assert!(matches!(
        kanso_sync::sign_in_with_provider(
            &backend,
            SERVER,
            &Provider::Github,
            "F",
            &device(),
            &|_| {}
        ),
        Err(AccountError::Protocol(_))
    ));
    assert!(matches!(
        kanso_sync::sign_in_with_provider(
            &backend,
            SERVER,
            &Provider::Google,
            "G",
            &device(),
            &|_| {}
        ),
        Err(AccountError::Protocol(_))
    ));

    *stub.step.lock().unwrap() = ProviderStep::Session(provider_session("alice"));
    let finished =
        kanso_sync::finish_google(&backend, SERVER, "G", "code", "verifier", &device()).unwrap();
    assert!(matches!(finished, ProviderStep::Session(_)));
    let seen = stub.seen.lock().unwrap();
    assert_eq!(seen.finishes.len(), 1);
    assert_eq!(seen.finishes[0].1, "code");
    assert_eq!(seen.finishes[0].2, "verifier");
}

#[test]
fn a_social_sign_up_saves_the_key_before_the_request_and_verifies_the_answer() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let secrets = MemorySecrets::new();
    let identity = identity();
    let steps = Mutex::new(Vec::new());
    let signed = kanso_sync::sign_up_with_provider(
        &backend,
        &secrets,
        SERVER,
        &Provider::Github,
        &"T".repeat(43),
        "quiet-otter-4821",
        &identity,
        &device(),
        &|step| steps.lock().unwrap().push(step),
    )
    .unwrap();
    assert_eq!(signed.session.username, "quiet-otter-4821");
    assert!(!signed.session.device_id.is_empty());
    let pending = kanso_sync::keychain::load_any_recovery_key(&secrets, &identity.library_id)
        .unwrap()
        .expect("the recovery key is saved");
    assert!(!pending.unconfirmed);
    assert_eq!(
        pending.recovery_key.display(),
        signed.recovery_key.display()
    );
    let steps = steps.lock().unwrap();
    assert!(steps.contains(&Step::Contacting) && steps.contains(&Step::DerivingKeys));
    assert_eq!(stub.seen.lock().unwrap().social.len(), 1);
}

#[test]
fn provider_sign_up_keeps_the_exact_device_from_provider_start() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let secrets = MemorySecrets::new();
    let identity = identity();
    let started =
        kanso_sync::start_provider(&backend, SERVER, &Provider::Apple, &device(), None, None)
            .unwrap();
    let flow = match started {
        ProviderStart::Apple(started) => started,
        _ => panic!("stub returns an Apple flow"),
    };
    let login_device = stub.seen.lock().unwrap().start_devices[0].clone();
    *stub.reject_social_once.lock().unwrap() = true;

    kanso_sync::sign_up_with_provider(
        &backend,
        &secrets,
        SERVER,
        &Provider::Apple,
        &flow.flow_id,
        "quiet-otter-4821",
        &identity,
        &login_device,
        &|_| {},
    )
    .unwrap();

    let seen = stub.seen.lock().unwrap();
    assert!(
        seen.social.len() == 2,
        "account id collision must retry once"
    );
    for request in &seen.social {
        assert!(
            request.ticket.as_str() == flow.flow_id.as_str(),
            "signup must keep the provider ticket"
        );
        assert!(
            request.device.id.as_str() == login_device.id.as_str(),
            "signup device id must match provider start"
        );
        assert!(
            request.device.proof.as_str() == login_device.proof.as_str(),
            "signup device proof must match provider start"
        );
    }
    assert!(
        seen.social[0].account_id != seen.social[1].account_id,
        "account id collision must choose a new account id"
    );
    assert!(
        seen.social[0].device.sealed_name != seen.social[1].device.sealed_name,
        "retry must reseal the device name for its new account keys"
    );
}

#[test]
fn a_reserved_or_invalid_name_never_reaches_the_server() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let secrets = MemorySecrets::new();
    for name in ["admin", "no", "Owner"] {
        let result = kanso_sync::sign_up_with_provider(
            &backend,
            &secrets,
            SERVER,
            &Provider::Github,
            &"T".repeat(43),
            name,
            &identity(),
            &device(),
            &|_| {},
        );
        assert!(matches!(
            result,
            Err(kanso_sync::RotationError {
                error: AccountError::Username(_),
                ..
            })
        ));
    }
    assert!(stub.seen.lock().unwrap().social.is_empty());
}

#[test]
fn methods_combine_the_advertisement_and_the_accounts_rows() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let session = a_session();
    let methods = kanso_sync::methods(&backend, &session).unwrap();
    assert_eq!(methods.linked.len(), 1);
    assert_eq!(methods.linked[0].provider, Provider::Github);
    assert_eq!(methods.passkeys.len(), 1);
    assert_eq!(methods.passkeys[0].credential_id, vec![1, 2, 3]);
    assert_eq!(methods.rp_id.as_deref(), Some("sync.kanso.test"));
    assert_eq!(methods.unlinked(), vec![Provider::Apple, Provider::Google]);
    assert!(!methods.last_method(&Provider::Github));
}

#[test]
fn a_link_and_unlink_use_the_proofs_the_server_expects() {
    let stub = Stub::new();
    let backend = StubBackend(stub.clone());
    let session = a_session();
    assert!(matches!(
        kanso_sync::finish_identity(&backend, &session, &Provider::Github, "F", &device()).unwrap(),
        LinkOutcome::Linked { .. }
    ));
    assert!(matches!(
        kanso_sync::start_identity(
            &backend,
            &session,
            &Provider::Github,
            IdentityIntent::Proof,
            None,
            None,
            None,
            &device()
        )
        .unwrap(),
        ProviderStart::Github(_)
    ));
    kanso_sync::unlink_identity(
        &backend,
        &session,
        &Provider::Github,
        ProofWire::provider("F".repeat(43)),
        None,
    )
    .unwrap();
    let seen = stub.seen.lock().unwrap();
    assert_eq!(seen.identity_starts[0].1, "Proof");
    assert!(seen.unlinks[0].1.contains("Provider"));
}

#[test]
fn an_older_server_advertises_nothing() {
    let stub = Stub::new();
    let backend = StubBackend(stub);
    // A Backend whose health omits auth is the same as an old server.
    struct Old(Arc<Stub>);
    impl Backend for Old {
        fn api(&self, _server: &str) -> Box<dyn Api + '_> {
            Box::new(StubApi {
                stub: self.0.clone(),
                auth: None,
            })
        }
        fn kdf_floor(&self) -> KdfParams {
            KdfParams::TEST
        }
        fn kdf_default(&self) -> KdfParams {
            KdfParams::TEST
        }
    }
    let auth = kanso_sync::server_auth(&Old(backend.0.clone()), SERVER).unwrap();
    assert_eq!(auth, kanso_sync::ServerAuth::default());
}
