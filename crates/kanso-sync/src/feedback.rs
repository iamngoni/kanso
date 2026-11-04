//! The feedback client (issue #232): `POST /v1/feedback`, the screenshot
//! `PUT`, `GET /v1/feedback` and `DELETE /v1/feedback/<id>`.
//!
//! These routes are always on Kanso's own Worker, even for a library that
//! syncs with a self-hosted server (self-hosted servers don't serve them,
//! so they are not in [`crate::Route::ALL`]). A report is sent as a signed-in
//! device (bearer token, device ID and proof) or, with no session, with the
//! install's anonymous key.
//!
//! [`Client`] is over a [`Transport`], so the retry and outbox logic that
//! drives it (kanso-engine's `feedback_flow`) is tested against a fake;
//! [`HttpTransport`] is the real one, over `ureq` and rustls, blocking.
//! Nothing here reads a note.

use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::Value;
use zeroize::Zeroizing;

use crate::binding::DeviceIdentity;
use crate::keychain::{SecretError, SecretStore};

/// Where every feedback request goes.
pub const ENDPOINT: &str = "https://sync.kansoapp.app";

/// Whether a session belongs to Kanso's own server. Only such a session may
/// sign a feedback request: a self-hosted server's token and device proof
/// must never be sent to Kanso's Worker.
pub fn is_kanso_origin(origin: &str) -> bool {
    origin.trim_end_matches('/') == ENDPOINT
}

/// The outbox `sender` of an anonymous report.
pub const ANONYMOUS_SENDER: &str = "anonymous";

/// The Keychain item holding an install's anonymous key (43 base64url
/// characters): the only way to list or delete reports made without an
/// account. The Worker stores only a keyed hash of it.
pub const ANONYMOUS_KEY_ITEM: &str = "feedback-key";

/// The most a list answer may be (100 reports with their chains).
const MAX_LIST_BYTES: u64 = 2 * 1024 * 1024;

/// Who is sending.
#[derive(Clone)]
pub enum Reporter {
    /// A signed-in device: `Authorization: Bearer …` and the device's ID and proof.
    Account {
        account_id: String,
        authorization: Zeroizing<String>,
        device_id: String,
        device_proof: Zeroizing<String>,
    },
    /// No account: the install's key.
    Anonymous { key: Zeroizing<String> },
}

impl Reporter {
    /// A signed-in reporter from the session and this library's device
    /// identity, or `None` when the session is another server's (a
    /// self-hosted library): its credentials never go to Kanso's Worker.
    pub fn from_session(session: &crate::Session, device: &DeviceIdentity) -> Option<Reporter> {
        if !is_kanso_origin(&session.origin) {
            return None;
        }
        let login = session.login_device(device);
        Some(Reporter::Account {
            account_id: session.account_id.clone(),
            authorization: session.authorization(),
            device_id: login.id.clone(),
            device_proof: Zeroizing::new(login.proof.clone()),
        })
    }

    /// A signed-in reporter from its parts (the account and the header values).
    pub fn account(
        account_id: &str,
        authorization: &str,
        device_id: &str,
        device_proof: &str,
    ) -> Reporter {
        Reporter::Account {
            account_id: account_id.to_owned(),
            authorization: Zeroizing::new(authorization.to_owned()),
            device_id: device_id.to_owned(),
            device_proof: Zeroizing::new(device_proof.to_owned()),
        }
    }

    /// An anonymous reporter, with the install's key made on first use and
    /// kept in the secret store.
    pub fn anonymous(secrets: &dyn SecretStore) -> Result<Reporter, SecretError> {
        if let Some(key) = secrets.get(ANONYMOUS_KEY_ITEM)?
            && valid_key(&key)
        {
            return Ok(Reporter::Anonymous { key });
        }
        let mut bytes = Zeroizing::new([0u8; 32]);
        getrandom::fill(&mut bytes[..]).map_err(|err| SecretError(err.to_string()))?;
        let key = Zeroizing::new(URL_SAFE_NO_PAD.encode(&bytes[..]));
        secrets.set(ANONYMOUS_KEY_ITEM, &key)?;
        Ok(Reporter::Anonymous { key })
    }

    /// The install's anonymous reporter if its key was ever made; never
    /// makes one. A signed-in device uses it to reach the reports it sent
    /// before signing in.
    pub fn existing_anonymous(secrets: &dyn SecretStore) -> Option<Reporter> {
        let key = secrets.get(ANONYMOUS_KEY_ITEM).ok()??;
        valid_key(&key).then_some(Reporter::Anonymous { key })
    }

    /// Who this is, as the outbox records it: `anonymous` or
    /// `account:<id>`. The server only lets the reporter that sent a report
    /// list, upload to or delete it, so the outbox keeps this per report.
    pub fn sender(&self) -> String {
        match self {
            Reporter::Account { account_id, .. } => format!("account:{account_id}"),
            Reporter::Anonymous { .. } => ANONYMOUS_SENDER.to_owned(),
        }
    }

    /// The headers that say who is sending.
    fn headers(&self) -> Vec<(&'static str, String)> {
        match self {
            Reporter::Account {
                authorization,
                device_id,
                device_proof,
                ..
            } => vec![
                ("authorization", authorization.to_string()),
                ("x-kanso-device", device_id.clone()),
                ("x-kanso-device-proof", device_proof.to_string()),
            ],
            Reporter::Anonymous { key } => vec![("x-kanso-feedback-key", key.to_string())],
        }
    }

    /// Whether this reporter has an account (so a release push can reach it).
    pub fn is_account(&self) -> bool {
        matches!(self, Reporter::Account { .. })
    }
}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reporter::Account { .. } => f.write_str("Reporter::Account(..)"),
            Reporter::Anonymous { .. } => f.write_str("Reporter::Anonymous(..)"),
        }
    }
}

fn valid_key(key: &str) -> bool {
    key.len() == 43
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// One request.
#[derive(Clone, Debug)]
pub struct Request {
    pub method: &'static str,
    pub path: String,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

/// One answer: any status.
#[derive(Clone, Debug)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Something that sends a request, or says it couldn't be reached.
pub trait Transport: Send + Sync {
    fn send(&self, request: &Request) -> Result<Response, String>;
}

impl<T: Transport + ?Sized> Transport for std::sync::Arc<T> {
    fn send(&self, request: &Request) -> Result<Response, String> {
        (**self).send(request)
    }
}

/// How a feedback call went wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackError {
    /// No answer (offline, DNS, TLS, timeout): try again later.
    Offline,
    /// 400: this field is off-pattern. It will fail again unless edited.
    Invalid { field: String },
    /// 401 or 403: the session or key isn't accepted.
    Unauthorized,
    /// 404: the server doesn't have it.
    NotFound,
    /// 409 on a report: the id belongs to another reporter.
    Conflict,
    /// 410: the screenshot slot has closed.
    Gone,
    /// 413 or 415.
    TooLarge,
    /// 429: not before `retry_after_ms`.
    RateLimited { retry_after_ms: i64 },
    /// 503 `feedback_closed`.
    Closed,
    /// Any other answer: try again later.
    Server(u16),
}

/// The server's answer to a stored report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submitted {
    pub id: String,
    pub status: String,
    /// True when the id was already stored (200).
    pub repeat: bool,
    /// Where the screenshot goes, when the server wants one.
    pub upload_path: Option<String>,
}

/// One entry of the server's list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteReport {
    pub id: String,
    pub created_ms: i64,
    pub kind: String,
    pub preview: String,
    pub status: String,
    pub issue: Option<u32>,
    pub duplicate_of: Option<u32>,
    pub pr: Option<u32>,
    pub release: Option<String>,
}

/// The feedback API over a [`Transport`].
pub struct Client<T: Transport> {
    transport: T,
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T) -> Client<T> {
        Client { transport }
    }

    fn call(
        &self,
        reporter: &Reporter,
        method: &'static str,
        path: String,
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> Result<Response, FeedbackError> {
        let mut headers = reporter.headers();
        if let Some(content_type) = content_type {
            headers.push(("content-type", content_type.to_owned()));
        }
        let response = self
            .transport
            .send(&Request {
                method,
                path,
                headers,
                body,
            })
            .map_err(|_| FeedbackError::Offline)?;
        if (200..300).contains(&response.status) {
            Ok(response)
        } else {
            Err(error_of(&response))
        }
    }

    /// `POST /v1/feedback` with the payload exactly as the person saw it.
    pub fn submit(&self, reporter: &Reporter, payload: &str) -> Result<Submitted, FeedbackError> {
        let response = self.call(
            reporter,
            "POST",
            "/v1/feedback".into(),
            Some("application/json"),
            payload.as_bytes().to_vec(),
        )?;
        let value: Value = serde_json::from_slice(&response.body)
            .map_err(|_| FeedbackError::Server(response.status))?;
        Ok(Submitted {
            id: str_of(&value, "id").unwrap_or_default(),
            status: str_of(&value, "status").unwrap_or_else(|| "received".into()),
            repeat: response.status == 200,
            upload_path: value
                .get("upload")
                .and_then(|u| u.get("url"))
                .and_then(Value::as_str)
                .filter(|p| p.starts_with("/v1/feedback/"))
                .map(str::to_owned),
        })
    }

    /// `PUT /v1/feedback/<id>/screenshot`. 204 is done; so is 409 (already stored).
    pub fn upload_screenshot(
        &self,
        reporter: &Reporter,
        id: &str,
        mime: &str,
        bytes: &[u8],
    ) -> Result<(), FeedbackError> {
        match self.call(
            reporter,
            "PUT",
            format!("/v1/feedback/{id}/screenshot"),
            Some(mime),
            bytes.to_vec(),
        ) {
            Ok(_) | Err(FeedbackError::Conflict) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// `GET /v1/feedback[?released=X.Y.Z]`, newest first.
    pub fn list(
        &self,
        reporter: &Reporter,
        released: Option<&str>,
    ) -> Result<Vec<RemoteReport>, FeedbackError> {
        let path = match released {
            Some(version) => format!("/v1/feedback?released={version}"),
            None => "/v1/feedback".to_owned(),
        };
        let response = self.call(reporter, "GET", path, None, Vec::new())?;
        if response.body.len() as u64 > MAX_LIST_BYTES {
            return Err(FeedbackError::Server(response.status));
        }
        let value: Value = serde_json::from_slice(&response.body)
            .map_err(|_| FeedbackError::Server(response.status))?;
        let reports = value
            .get("reports")
            .and_then(Value::as_array)
            .ok_or(FeedbackError::Server(response.status))?;
        Ok(reports.iter().filter_map(remote_report).collect())
    }

    /// `DELETE /v1/feedback/<id>`. A 404 means it is already gone, which is done.
    pub fn delete(&self, reporter: &Reporter, id: &str) -> Result<(), FeedbackError> {
        match self.call(
            reporter,
            "DELETE",
            format!("/v1/feedback/{id}"),
            None,
            Vec::new(),
        ) {
            Ok(_) | Err(FeedbackError::NotFound) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

fn str_of(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number_of(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
}

fn remote_report(value: &Value) -> Option<RemoteReport> {
    Some(RemoteReport {
        id: str_of(value, "id")?,
        created_ms: value.get("created_ms")?.as_i64()?,
        kind: str_of(value, "kind")?,
        preview: str_of(value, "preview").unwrap_or_default(),
        status: str_of(value, "status")?,
        issue: number_of(value, "issue"),
        duplicate_of: number_of(value, "duplicate_of"),
        pr: number_of(value, "pr"),
        release: str_of(value, "release"),
    })
}

/// What a non-2xx answer means. The body is read for its `field`,
/// `retry_after` (seconds, as `http.ts` `rateLimited` sends it) and
/// `error`; nothing else of it is kept.
pub fn error_of(response: &Response) -> FeedbackError {
    let body: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
    match response.status {
        400 => FeedbackError::Invalid {
            field: str_of(&body, "field").unwrap_or_default(),
        },
        401 | 403 => FeedbackError::Unauthorized,
        404 => FeedbackError::NotFound,
        409 => FeedbackError::Conflict,
        410 => FeedbackError::Gone,
        413 | 415 => FeedbackError::TooLarge,
        429 => FeedbackError::RateLimited {
            retry_after_ms: body
                .get("retry_after")
                .and_then(Value::as_i64)
                .map(|seconds| seconds.saturating_mul(1000))
                .or_else(|| body.get("retry_after_ms").and_then(Value::as_i64))
                .unwrap_or(60_000)
                .max(0),
        },
        503 if str_of(&body, "error").as_deref() == Some("feedback_closed") => {
            FeedbackError::Closed
        }
        status => FeedbackError::Server(status),
    }
}

/// The real transport: blocking `ureq` over rustls, no redirects, nothing
/// but the request's own headers (no cookie, no Origin).
pub struct HttpTransport {
    base: String,
    agent: ureq::Agent,
}

impl HttpTransport {
    /// Talks to Kanso's Worker.
    pub fn new() -> HttpTransport {
        HttpTransport::to(ENDPOINT)
    }

    /// Talks to Kanso's Worker, giving up after `seconds` (a notice that
    /// can't wait for a slow network).
    pub fn quick(seconds: u64) -> HttpTransport {
        HttpTransport::build(ENDPOINT, Duration::from_secs(seconds))
    }

    /// Talks to `base` (tests use a local server).
    pub fn to(base: &str) -> HttpTransport {
        HttpTransport::build(base, Duration::from_secs(60))
    }

    fn build(base: &str, total: Duration) -> HttpTransport {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(Duration::from_secs(15).min(total)))
            .timeout_global(Some(total))
            .user_agent(concat!("kanso-sync/", env!("CARGO_PKG_VERSION")))
            .tls_config(kanso_tls::ureq_tls())
            .build()
            .into();
        HttpTransport {
            base: base.trim_end_matches('/').to_owned(),
            agent,
        }
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        HttpTransport::new()
    }
}

impl Transport for HttpTransport {
    // no-prop-test: the network; `error_of` and the client are tested over a fake.
    fn send(&self, request: &Request) -> Result<Response, String> {
        let url = format!("{}{}", self.base, request.path);
        // An Android app that hasn't handed over its Context gets an error, not a panic.
        kanso_tls::ready_for(&url).map_err(|err| err.to_string())?;
        let mut response = match request.method {
            "GET" => {
                let mut builder = self.agent.get(&url);
                for (name, value) in &request.headers {
                    builder = builder.header(*name, value.as_str());
                }
                builder.call()
            }
            "DELETE" => {
                let mut builder = self.agent.delete(&url);
                for (name, value) in &request.headers {
                    builder = builder.header(*name, value.as_str());
                }
                builder.call()
            }
            "PUT" => {
                let mut builder = self.agent.put(&url);
                for (name, value) in &request.headers {
                    builder = builder.header(*name, value.as_str());
                }
                builder.send(&request.body[..])
            }
            _ => {
                let mut builder = self.agent.post(&url);
                for (name, value) in &request.headers {
                    builder = builder.header(*name, value.as_str());
                }
                builder.send(&request.body[..])
            }
        }
        .map_err(|err| err.to_string())?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_LIST_BYTES)
            .read_to_vec()
            .map_err(|err| err.to_string())?;
        Ok(Response { status, body })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keychain::MemorySecrets;
    use std::sync::Mutex;

    /// Answers from a script and records what was asked.
    struct Fake {
        answers: Mutex<Vec<Result<Response, String>>>,
        seen: Mutex<Vec<Request>>,
    }

    impl Fake {
        fn new(answers: Vec<Result<Response, String>>) -> Fake {
            Fake {
                answers: Mutex::new(answers),
                seen: Mutex::new(Vec::new()),
            }
        }
    }

    impl Transport for &Fake {
        fn send(&self, request: &Request) -> Result<Response, String> {
            self.seen.lock().unwrap().push(request.clone());
            self.answers.lock().unwrap().remove(0)
        }
    }

    fn ok(status: u16, body: &str) -> Result<Response, String> {
        Ok(Response {
            status,
            body: body.as_bytes().to_vec(),
        })
    }

    fn account() -> Reporter {
        Reporter::Account {
            account_id: "a_1".into(),
            authorization: Zeroizing::new("Bearer kanso_x".into()),
            device_id: "d_1".into(),
            device_proof: Zeroizing::new("p".into()),
        }
    }

    #[test]
    fn submit_sends_the_payload_with_the_device_headers() {
        let fake = Fake::new(vec![ok(
            201,
            r#"{"id":"fb_x","status":"received","upload":{"url":"/v1/feedback/fb_x/screenshot","expires_ms":1}}"#,
        )]);
        let sent = Client::new(&fake).submit(&account(), "{\"v\":1}").unwrap();
        assert_eq!(
            sent.upload_path.as_deref(),
            Some("/v1/feedback/fb_x/screenshot")
        );
        assert!(!sent.repeat);
        let seen = fake.seen.lock().unwrap();
        assert_eq!(
            (seen[0].method, seen[0].path.as_str()),
            ("POST", "/v1/feedback")
        );
        assert_eq!(seen[0].body, b"{\"v\":1}");
        let names: Vec<&str> = seen[0].headers.iter().map(|(n, _)| *n).collect();
        assert!(
            names.contains(&"authorization")
                && names.contains(&"x-kanso-device")
                && names.contains(&"x-kanso-device-proof")
        );
    }

    #[test]
    fn a_repeat_is_not_an_error() {
        let fake = Fake::new(vec![ok(200, r#"{"id":"fb_x","status":"filed"}"#)]);
        let sent = Client::new(&fake).submit(&account(), "{}").unwrap();
        assert!(sent.repeat);
        assert_eq!(sent.upload_path, None);
    }

    #[test]
    fn answers_map_to_errors() {
        let cases = [
            (
                400,
                r#"{"error":"invalid","field":"app.version"}"#,
                FeedbackError::Invalid {
                    field: "app.version".into(),
                },
            ),
            (401, "{}", FeedbackError::Unauthorized),
            (403, "{}", FeedbackError::Unauthorized),
            (409, "{}", FeedbackError::Conflict),
            (413, "{}", FeedbackError::TooLarge),
            // The Worker's shape: `retry_after` in seconds (http.ts `rateLimited`).
            (
                429,
                r#"{"error":"rate_limited","message":"Too many requests.","limit":"feedback-ip","retry_after":86399}"#,
                FeedbackError::RateLimited {
                    retry_after_ms: 86_399_000,
                },
            ),
            (
                429,
                r#"{"error":"rate_limited","retry_after_ms":9000}"#,
                FeedbackError::RateLimited {
                    retry_after_ms: 9000,
                },
            ),
            (503, r#"{"error":"feedback_closed"}"#, FeedbackError::Closed),
            (503, "{}", FeedbackError::Server(503)),
            (500, "oops", FeedbackError::Server(500)),
        ];
        for (status, body, want) in cases {
            let fake = Fake::new(vec![ok(status, body)]);
            assert_eq!(Client::new(&fake).submit(&account(), "{}"), Err(want));
        }
        let fake = Fake::new(vec![Err("dns".into())]);
        assert_eq!(
            Client::new(&fake).submit(&account(), "{}"),
            Err(FeedbackError::Offline)
        );
    }

    #[test]
    fn delete_and_screenshot_treat_gone_and_stored_as_done() {
        let fake = Fake::new(vec![ok(404, "{}"), ok(409, "{}"), ok(410, "{}")]);
        let client = Client::new(&fake);
        assert_eq!(client.delete(&account(), "fb_x"), Ok(()));
        assert_eq!(
            client.upload_screenshot(&account(), "fb_x", "image/png", b"x"),
            Ok(())
        );
        assert_eq!(
            client.upload_screenshot(&account(), "fb_x", "image/png", b"x"),
            Err(FeedbackError::Gone)
        );
    }

    #[test]
    fn list_reads_reports_and_skips_malformed_ones() {
        let fake = Fake::new(vec![ok(
            200,
            r#"{"reports":[{"id":"fb_a","created_ms":5,"kind":"bug","preview":"Export","status":"released","issue":241,"duplicate_of":null,"pr":252,"release":"v0.9.2","events":[]},{"nope":1}]}"#,
        )]);
        let reports = Client::new(&fake).list(&account(), Some("0.9.2")).unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(
            (
                reports[0].issue,
                reports[0].pr,
                reports[0].release.as_deref()
            ),
            (Some(241), Some(252), Some("v0.9.2"))
        );
        assert_eq!(
            fake.seen.lock().unwrap()[0].path,
            "/v1/feedback?released=0.9.2"
        );
    }

    #[test]
    fn the_anonymous_key_is_made_once() {
        let secrets = MemorySecrets::new();
        let (Reporter::Anonymous { key: a }, Reporter::Anonymous { key: b }) = (
            Reporter::anonymous(&secrets).unwrap(),
            Reporter::anonymous(&secrets).unwrap(),
        ) else {
            panic!("anonymous")
        };
        assert_eq!(a.len(), 43);
        assert_eq!(a, b);
        assert!(valid_key(&a));
        assert_eq!(
            Reporter::existing_anonymous(&secrets).map(|r| r.sender()),
            Some("anonymous".to_owned())
        );
    }

    #[test]
    fn no_anonymous_key_is_made_by_looking() {
        let secrets = MemorySecrets::new();
        assert!(Reporter::existing_anonymous(&secrets).is_none());
        assert!(secrets.get(ANONYMOUS_KEY_ITEM).unwrap().is_none());
    }

    #[test]
    fn only_kanso_sessions_sign_feedback() {
        assert!(is_kanso_origin("https://sync.kansoapp.app"));
        assert!(is_kanso_origin("https://sync.kansoapp.app/"));
        assert!(!is_kanso_origin("https://notes.example.com"));
        assert!(!is_kanso_origin("https://sync.kansoapp.app.evil.example"));
        assert!(!is_kanso_origin("http://sync.kansoapp.app"));
        assert_eq!(account().sender(), "account:a_1");
    }
}
