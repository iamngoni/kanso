//! The Cloudflare Worker over HTTPS (blocking; the app has no async runtime):
//! [`HttpBackend`], every v2 route with `max_redirects(0)` and the response
//! caps of docs/ACCOUNTS.md › Conventions.

use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use ureq::Agent;
use zeroize::Zeroizing;

use crate::account::Session;
use crate::api::{
    AccountPasskeysFinishRequest, AccountPasskeysFinishResponse, AccountPasskeysStartRequest,
    AccountPasskeysStartResponse, AccountResponse, AccountSignInMethodsResponse, Api, Backend,
    DeleteRequest, DeleteResponse, DeviceNameRequest, DevicesResponse, Health, LoginRequest,
    LoginResponse, PasskeyFinishRequest, PasskeyProofRequest, PasskeyProofStartRequest,
    PasskeyProofStartResponse, PasskeySessionResponse, PasskeySignupFinishRequest,
    PasskeySignupStartRequest, PasskeySignupStartResponse, PasskeyStartRequest,
    PasskeyStartResponse, PasswordRequest, PasswordResponse, PreloginRequest, PreloginResponse,
    PushRegistration, RecoverFinishRequest, RecoverStartRequest, RecoverStartResponse,
    RecoveryKeyRequest, RecoveryKeyResponse, Route, SessionResponse, SignInPasswordRequest,
    SignupRequest,
};
use crate::api::{
    ApprovalApproveRequest, ApprovalChallengeRequest, ApprovalCreateRequest, ApprovalCreated,
    ApprovalList, ApprovalNameRequest, ApprovalPoll,
};
use crate::binding::DeviceIdentity;
use crate::crypto::{KdfParams, sha256_hex};
use crate::events::{EventError, EventSource, EventStream};
use crate::provider::{
    GoogleFinishRequest, GoogleStartRequest, IdentityStartRequest, LinkOutcome, MethodsReply,
    Provider, ProviderPollRequest, ProviderSession, ProviderStart, ProviderStartRequest,
    ProviderStep, SocialSignupRequest, UnlinkRequest,
};
use crate::remote::encode_segment;
use crate::wire::{
    ApiError, MAX_AUTH_RESPONSE, MAX_FILE_RESPONSE, MAX_SYNC_RESPONSE, PushRequest, PushResponse,
    PutFileResponse, SessionToken, WirePage,
};

/// The app's [`Backend`]: every v2 route over blocking `ureq`, with no
/// redirects (a redirect could re-send a proof) and the response caps of
/// docs/ACCOUNTS.md › Conventions. It never sends an `Origin` header. Its
/// KDF floor and default are the production constants.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpBackend;

impl HttpBackend {
    pub fn new() -> HttpBackend {
        HttpBackend
    }
}

impl Backend for HttpBackend {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        Box::new(HttpApi::new(server))
    }

    fn kdf_floor(&self) -> KdfParams {
        KdfParams::FLOOR
    }

    fn kdf_default(&self) -> KdfParams {
        KdfParams::DEFAULT
    }

    fn events(
        &self,
        session: &Session,
        device: &DeviceIdentity,
        path: &str,
    ) -> Result<Box<dyn EventSource>, EventError> {
        // The proof the session was created with: a passkey or provider
        // sign-in's flow device, else this library's own.
        let login = session.login_device(device);
        let stream = EventStream::connect(
            &session.server,
            path,
            &session.authorization(),
            &login.id,
            &login.proof,
        )?;
        Ok(Box::new(stream))
    }
}

/// How long `route` may take, connecting included (A7). A logout never
/// keeps a sign-out waiting; the account routes, which a person waits on
/// behind a spinner, give up after 30 s; pushes and files carry up to the
/// size caps and keep 300 s.
fn timeout(route: Route) -> Duration {
    match route {
        Route::Logout => Duration::from_secs(5),
        Route::Push | Route::Changes | Route::PutFile | Route::GetFile => Duration::from_secs(300),
        Route::Health
        | Route::Prelogin
        | Route::Signup
        | Route::Login
        | Route::PasskeyStart
        | Route::PasskeyFinish
        | Route::PasskeySignupStart
        | Route::PasskeySignupFinish
        | Route::AccountSignInMethods
        | Route::PasskeyProofStart
        | Route::AccountPasskeysStart
        | Route::AccountPasskeysFinish
        | Route::AccountPasskeyRemove
        | Route::SignInPassword
        | Route::RecoverStart
        | Route::RecoverFinish
        | Route::Account
        | Route::Password
        | Route::RecoveryKey
        | Route::Delete
        | Route::SetDevice
        | Route::Devices
        | Route::RevokeDevice
        | Route::ProviderStart
        | Route::ProviderPoll
        | Route::GoogleStart
        | Route::GoogleFinish
        | Route::SocialSignup
        | Route::SignInMethods
        | Route::IdentityStart
        | Route::IdentityUnlink
        | Route::ApprovalCreate
        | Route::ApprovalList
        | Route::ApprovalPoll
        | Route::ApprovalChallenge
        | Route::ApprovalName
        | Route::ApprovalApprove
        | Route::ApprovalCancel
        | Route::PutPush
        | Route::DeletePush => Duration::from_secs(30),
    }
}

/// The v2 routes of one server.
struct HttpApi {
    base: String,
    agent: Agent,
}

/// A request body.
enum Payload<'a> {
    None,
    /// Zeroized once sent: it may hold auth keys and proofs.
    Json(Zeroizing<Vec<u8>>),
    /// Sealed file bytes, sent with their SHA-256 in `x-kanso-sha256`.
    File(&'a [u8]),
}

impl HttpApi {
    fn new(server: &str) -> HttpApi {
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_global(Some(Duration::from_secs(300)))
            .user_agent(concat!("kanso-sync/", env!("CARGO_PKG_VERSION")))
            .tls_config(kanso_tls::ureq_tls())
            .build()
            .into();
        HttpApi {
            base: server.trim().trim_end_matches('/').to_owned(),
            agent,
        }
    }

    /// Sends one request and reads the answer up to `cap` bytes: the status
    /// and body of a 2xx answer, or the route's [`ApiError`].
    fn send(
        &self,
        route: Route,
        path: &str,
        token: Option<&SessionToken>,
        payload: Payload,
        cap: u64,
    ) -> Result<(u16, Vec<u8>), ApiError> {
        let (method, _) = route.http();
        let url = format!("{}{path}", self.base);
        // Before the handshake: an Android app that hasn't handed over its
        // Context gets an error, not a panic inside the verifier.
        kanso_tls::ready_for(&url).map_err(|err| ApiError::Transport(err.to_string()))?;
        let auth = token.map(|t| Zeroizing::new(format!("Bearer {}", t.expose())));
        let timeout = Some(timeout(route));
        let response = match method {
            "GET" | "DELETE" => {
                let mut request = if method == "GET" {
                    self.agent.get(&url)
                } else {
                    self.agent.delete(&url)
                };
                if let Some(auth) = &auth {
                    request = request.header("authorization", auth.as_str());
                }
                request.config().timeout_global(timeout).build().call()
            }
            _ => {
                let mut request = if method == "POST" {
                    self.agent.post(&url)
                } else {
                    self.agent.put(&url)
                };
                if let Some(auth) = &auth {
                    request = request.header("authorization", auth.as_str());
                }
                let request = request.config().timeout_global(timeout).build();
                match payload {
                    Payload::None => request
                        .header("content-type", "application/json")
                        .send(&b"{}"[..]),
                    Payload::Json(body) => request
                        .header("content-type", "application/json")
                        .send(&body[..]),
                    Payload::File(bytes) => request
                        .header("content-type", "application/octet-stream")
                        .header("x-kanso-sha256", sha256_hex(bytes))
                        .send(bytes),
                }
            }
        };
        let mut response = response.map_err(transport_error)?;
        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            return Err(ApiError::Protocol(format!(
                "the server answered {status} (a redirect), which the client never follows"
            )));
        }
        let ok = (200..300).contains(&status);
        let limit = if ok { cap } else { MAX_AUTH_RESPONSE };
        let body = response
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(|err| match err {
                ureq::Error::BodyExceedsLimit(_) => {
                    ApiError::Protocol(format!("the answer is over the {limit}-byte cap"))
                }
                err => ApiError::Transport(err.to_string()),
            })?;
        if ok {
            Ok((status, body))
        } else {
            Err(ApiError::from_response(status, &body))
        }
    }

    /// A JSON request whose 2xx answer is JSON.
    fn json<T: DeserializeOwned>(
        &self,
        route: Route,
        path: &str,
        token: Option<&SessionToken>,
        body: Option<&impl Serialize>,
        cap: u64,
    ) -> Result<T, ApiError> {
        let payload = match body {
            Some(body) => Payload::Json(Zeroizing::new(
                serde_json::to_vec(body).map_err(|e| ApiError::Protocol(e.to_string()))?,
            )),
            None => Payload::None,
        };
        // An answer may carry a session token.
        let bytes = Zeroizing::new(self.send(route, path, token, payload, cap)?.1);
        serde_json::from_slice(&bytes)
            .map_err(|e| ApiError::Protocol(format!("{}: {e}", route.http().1)))
    }

    /// A request whose answer is 204 (or any 2xx; its body is ignored).
    fn empty(
        &self,
        route: Route,
        path: &str,
        token: &SessionToken,
        body: Option<&impl Serialize>,
    ) -> Result<(), ApiError> {
        let payload = match body {
            Some(body) => Payload::Json(Zeroizing::new(
                serde_json::to_vec(body).map_err(|e| ApiError::Protocol(e.to_string()))?,
            )),
            None => Payload::None,
        };
        self.send(route, path, Some(token), payload, MAX_AUTH_RESPONSE)
            .map(|_| ())
    }
}

/// A request that got no answer. Failures before anything was sent (the
/// name didn't resolve, the connection was refused or never opened) are
/// [`ApiError::unsent`]: the server can't have acted on them. Anything later
/// (a reset, a timeout waiting for the answer, TLS trouble) may have reached
/// it.
fn transport_error(err: ureq::Error) -> ApiError {
    use std::io::ErrorKind;
    use ureq::Timeout;
    let unsent = match &err {
        ureq::Error::HostNotFound | ureq::Error::ConnectionFailed | ureq::Error::BadUri(_) => true,
        ureq::Error::Timeout(Timeout::Resolve | Timeout::Connect) => true,
        ureq::Error::Io(io) => matches!(
            io.kind(),
            ErrorKind::ConnectionRefused
                | ErrorKind::HostUnreachable
                | ErrorKind::NetworkUnreachable
                | ErrorKind::AddrNotAvailable
                | ErrorKind::NotConnected
        ),
        _ => false,
    };
    if unsent {
        ApiError::unsent(err)
    } else {
        ApiError::Transport(err.to_string())
    }
}

/// No request body.
const NO_BODY: Option<&()> = None;

impl Api for HttpApi {
    fn health(&self) -> Result<Health, ApiError> {
        self.json(
            Route::Health,
            "/v2/health",
            None,
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn prelogin(&self, request: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        self.json(
            Route::Prelogin,
            "/v2/auth/prelogin",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn signup(&self, request: &SignupRequest) -> Result<SessionResponse, ApiError> {
        self.json(
            Route::Signup,
            "/v2/auth/signup",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn login(&self, request: &LoginRequest) -> Result<LoginResponse, ApiError> {
        self.json(
            Route::Login,
            "/v2/auth/login",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn passkey_start(
        &self,
        request: &PasskeyStartRequest,
    ) -> Result<PasskeyStartResponse, ApiError> {
        self.json(
            Route::PasskeyStart,
            "/v2/auth/passkey/start",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }
    fn passkey_finish(
        &self,
        request: &PasskeyFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.json(
            Route::PasskeyFinish,
            "/v2/auth/passkey/finish",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }
    fn passkey_signup_start(
        &self,
        request: &PasskeySignupStartRequest,
    ) -> Result<PasskeySignupStartResponse, ApiError> {
        self.json(
            Route::PasskeySignupStart,
            "/v2/auth/passkey/signup/start",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }
    fn passkey_signup_finish(
        &self,
        request: &PasskeySignupFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.json(
            Route::PasskeySignupFinish,
            "/v2/auth/passkey/signup/finish",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn account_sign_in_methods(
        &self,
        token: &SessionToken,
    ) -> Result<AccountSignInMethodsResponse, ApiError> {
        self.json(
            Route::AccountSignInMethods,
            "/v2/account/sign-in",
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn passkey_proof_start(
        &self,
        token: &SessionToken,
        request: &PasskeyProofStartRequest,
    ) -> Result<PasskeyProofStartResponse, ApiError> {
        self.json(
            Route::PasskeyProofStart,
            "/v2/account/sign-in/proof/start",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn account_passkeys_start(
        &self,
        token: &SessionToken,
        request: &AccountPasskeysStartRequest,
    ) -> Result<AccountPasskeysStartResponse, ApiError> {
        self.json(
            Route::AccountPasskeysStart,
            "/v2/account/sign-in/passkeys/start",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn account_passkeys_finish(
        &self,
        token: &SessionToken,
        request: &AccountPasskeysFinishRequest,
    ) -> Result<AccountPasskeysFinishResponse, ApiError> {
        self.json(
            Route::AccountPasskeysFinish,
            "/v2/account/sign-in/passkeys/finish",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn account_passkey_remove(
        &self,
        token: &SessionToken,
        credential_id: &str,
        request: &PasskeyProofRequest,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/v2/account/sign-in/passkeys/{}/remove",
            encode_segment(credential_id)
        );
        self.empty(Route::AccountPasskeyRemove, &path, token, Some(request))
    }

    fn set_first_password(
        &self,
        token: &SessionToken,
        request: &SignInPasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.json(
            Route::SignInPassword,
            "/v2/account/sign-in/password",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn recover_start(
        &self,
        request: &RecoverStartRequest,
    ) -> Result<RecoverStartResponse, ApiError> {
        self.json(
            Route::RecoverStart,
            "/v2/auth/recover/start",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn recover_finish(&self, request: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
        self.json(
            Route::RecoverFinish,
            "/v2/auth/recover/finish",
            None,
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn logout(&self, token: &SessionToken) -> Result<(), ApiError> {
        self.empty(Route::Logout, "/v2/auth/logout", token, NO_BODY)
    }

    fn account(&self, token: &SessionToken) -> Result<AccountResponse, ApiError> {
        self.json(
            Route::Account,
            "/v2/account",
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn change_password(
        &self,
        token: &SessionToken,
        request: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.json(
            Route::Password,
            "/v2/account/password",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn replace_recovery_key(
        &self,
        token: &SessionToken,
        request: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.json(
            Route::RecoveryKey,
            "/v2/account/recovery-key",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn delete_account(
        &self,
        token: &SessionToken,
        request: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError> {
        self.json(
            Route::Delete,
            "/v2/account/delete",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn set_device_name(
        &self,
        token: &SessionToken,
        request: &DeviceNameRequest,
    ) -> Result<(), ApiError> {
        self.empty(Route::SetDevice, "/v2/account/device", token, Some(request))
    }

    fn devices(&self, token: &SessionToken) -> Result<DevicesResponse, ApiError> {
        self.json(
            Route::Devices,
            "/v2/account/devices",
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn revoke_device(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        let path = format!("/v2/account/devices/{}", encode_segment(device_id));
        self.send(
            Route::RevokeDevice,
            &path,
            Some(token),
            Payload::None,
            MAX_AUTH_RESPONSE,
        )
        .map(|_| ())
    }

    fn put_push(
        &self,
        token: &SessionToken,
        device_id: &str,
        registration: &PushRegistration,
    ) -> Result<(), ApiError> {
        self.empty(
            Route::PutPush,
            &push_path(device_id),
            token,
            Some(registration),
        )
    }

    fn delete_push(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        self.empty(Route::DeletePush, &push_path(device_id), token, NO_BODY)
    }

    fn push(&self, token: &SessionToken, request: &PushRequest) -> Result<PushResponse, ApiError> {
        self.json(
            Route::Push,
            "/v2/push",
            Some(token),
            Some(request),
            MAX_SYNC_RESPONSE,
        )
    }

    fn changes(
        &self,
        token: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<WirePage, ApiError> {
        let mut query = Vec::new();
        if let Some(since) = since {
            query.push(format!("since={since}"));
        }
        if let Some(limit) = limit {
            query.push(format!("limit={limit}"));
        }
        let path = if query.is_empty() {
            "/v2/changes".to_owned()
        } else {
            format!("/v2/changes?{}", query.join("&"))
        };
        self.json(
            Route::Changes,
            &path,
            Some(token),
            NO_BODY,
            MAX_SYNC_RESPONSE,
        )
    }

    fn put_file(
        &self,
        token: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
    ) -> Result<PutFileResponse, ApiError> {
        let path = format!("/v2/files/{}", encode_segment(wire_id));
        let (_, body) = self.send(
            Route::PutFile,
            &path,
            Some(token),
            Payload::File(sealed),
            MAX_AUTH_RESPONSE,
        )?;
        serde_json::from_slice(&body).map_err(|e| ApiError::Protocol(format!("/v2/files: {e}")))
    }

    fn get_file(&self, token: &SessionToken, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        let path = format!("/v2/files/{}", encode_segment(wire_id));
        match self.send(
            Route::GetFile,
            &path,
            Some(token),
            Payload::None,
            MAX_FILE_RESPONSE,
        ) {
            Ok((_, bytes)) => Ok(Some(bytes)),
            Err(ApiError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn provider_start(
        &self,
        provider: &Provider,
        request: &ProviderStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        let path = format!("/v2/auth/{}/start", provider.as_str());
        let bytes = Zeroizing::new(
            self.send(
                Route::ProviderStart,
                &path,
                None,
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        ProviderStart::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn google_start(&self, request: &GoogleStartRequest) -> Result<ProviderStart, ApiError> {
        let bytes = Zeroizing::new(
            self.send(
                Route::GoogleStart,
                "/v2/auth/google/start",
                None,
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        ProviderStart::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn provider_poll(
        &self,
        provider: &Provider,
        request: &ProviderPollRequest,
    ) -> Result<ProviderStep, ApiError> {
        let path = format!("/v2/auth/{}/poll", provider.as_str());
        let bytes = Zeroizing::new(
            self.send(
                Route::ProviderPoll,
                &path,
                None,
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        ProviderStep::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn google_finish(&self, request: &GoogleFinishRequest) -> Result<ProviderStep, ApiError> {
        let bytes = Zeroizing::new(
            self.send(
                Route::GoogleFinish,
                "/v2/auth/google/finish",
                None,
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        ProviderStep::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn social_signup(&self, request: &SocialSignupRequest) -> Result<ProviderSession, ApiError> {
        let bytes = Zeroizing::new(
            self.send(
                Route::SocialSignup,
                "/v2/auth/social/signup",
                None,
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        match ProviderStep::from_json(&bytes).map_err(ApiError::Protocol)? {
            ProviderStep::Session(session) => Ok(session),
            _ => Err(ApiError::Protocol("a sign-up without a session".into())),
        }
    }

    fn sign_in_methods(&self, token: &SessionToken) -> Result<MethodsReply, ApiError> {
        self.json(
            Route::SignInMethods,
            "/v2/account/sign-in",
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn create_approval(
        &self,
        token: &SessionToken,
        request: &ApprovalCreateRequest,
    ) -> Result<ApprovalCreated, ApiError> {
        self.json(
            Route::ApprovalCreate,
            "/v2/account/sign-in/approvals",
            Some(token),
            Some(request),
            MAX_AUTH_RESPONSE,
        )
    }

    fn approvals(&self, token: &SessionToken) -> Result<ApprovalList, ApiError> {
        self.json(
            Route::ApprovalList,
            "/v2/account/sign-in/approvals",
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn approval(&self, token: &SessionToken, id: &str) -> Result<ApprovalPoll, ApiError> {
        self.json(
            Route::ApprovalPoll,
            &approval_path(id, ""),
            Some(token),
            NO_BODY,
            MAX_AUTH_RESPONSE,
        )
    }

    fn challenge_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalChallengeRequest,
    ) -> Result<(), ApiError> {
        self.empty(
            Route::ApprovalChallenge,
            &approval_path(id, "/challenge"),
            token,
            Some(request),
        )
    }

    fn name_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalNameRequest,
    ) -> Result<(), ApiError> {
        self.empty(
            Route::ApprovalName,
            &approval_path(id, "/name"),
            token,
            Some(request),
        )
    }

    fn approve_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalApproveRequest,
    ) -> Result<(), ApiError> {
        self.empty(
            Route::ApprovalApprove,
            &approval_path(id, "/approve"),
            token,
            Some(request),
        )
    }

    fn cancel_approval(&self, token: &SessionToken, id: &str) -> Result<(), ApiError> {
        self.send(
            Route::ApprovalCancel,
            &approval_path(id, ""),
            Some(token),
            Payload::None,
            MAX_AUTH_RESPONSE,
        )
        .map(|_| ())
    }

    fn identity_start(
        &self,
        token: &SessionToken,
        provider: &Provider,
        request: &IdentityStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        let path = format!("/v2/account/sign-in/identities/{}/start", provider.as_str());
        let bytes = Zeroizing::new(
            self.send(
                Route::IdentityStart,
                &path,
                Some(token),
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        ProviderStart::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn identity_poll(
        &self,
        token: &SessionToken,
        provider: &Provider,
        request: &ProviderPollRequest,
    ) -> Result<LinkOutcome, ApiError> {
        let path = format!("/v2/auth/{}/poll", provider.as_str());
        let bytes = Zeroizing::new(
            self.send(
                Route::ProviderPoll,
                &path,
                Some(token),
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        LinkOutcome::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn identity_finish_google(
        &self,
        token: &SessionToken,
        _provider: &Provider,
        request: &GoogleFinishRequest,
    ) -> Result<LinkOutcome, ApiError> {
        let path = "/v2/auth/google/finish";
        let bytes = Zeroizing::new(
            self.send(
                Route::GoogleFinish,
                path,
                Some(token),
                json_payload(request)?,
                MAX_AUTH_RESPONSE,
            )?
            .1,
        );
        LinkOutcome::from_json(&bytes).map_err(ApiError::Protocol)
    }

    fn unlink_identity(
        &self,
        token: &SessionToken,
        provider: &Provider,
        request: &UnlinkRequest,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/v2/account/sign-in/identities/{}/unlink",
            provider.as_str()
        );
        self.empty(Route::IdentityUnlink, &path, token, Some(request))
    }
}

/// A JSON body for a route that parses its answer itself.
fn json_payload(body: &impl Serialize) -> Result<Payload<'static>, ApiError> {
    Ok(Payload::Json(Zeroizing::new(
        serde_json::to_vec(body).map_err(|e| ApiError::Protocol(e.to_string()))?,
    )))
}

/// `/v2/account/devices/<device_id>/push`, the ID percent-encoded.
fn push_path(device_id: &str) -> String {
    format!("/v2/account/devices/{}/push", encode_segment(device_id))
}

/// `/v2/account/sign-in/approvals/<id><rest>`, the ID percent-encoded.
fn approval_path(id: &str, rest: &str) -> String {
    format!("/v2/account/sign-in/approvals/{}{rest}", encode_segment(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every route has its limit: 5 s for a logout, 300 s for what carries
    /// notes and files, 30 s for the rest.
    #[test]
    fn each_route_has_its_timeout() {
        for route in Route::ALL
            .into_iter()
            .chain(Route::PASSKEYS)
            .chain(Route::PROVIDERS)
        {
            let secs = timeout(route).as_secs();
            let expected = match route {
                Route::Logout => 5,
                Route::Push | Route::Changes | Route::PutFile | Route::GetFile => 300,
                _ => 30,
            };
            assert_eq!(secs, expected, "{route:?}");
        }
    }
}
