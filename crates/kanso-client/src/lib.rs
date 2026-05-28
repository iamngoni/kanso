//! HTTP sync transport — the client side of Kanso Cloud sync.
//!
//! Implements the engine's [`SyncTransport`] over HTTP against the cloud's
//! `/v1/sync/push` and `/v1/sync/pull` endpoints. The native apps construct one
//! of these and hand it to [`kanso_engine::Engine::sync`]; the engine itself
//! never touches the network.

use async_trait::async_trait;
use kanso_engine::SyncTransport;
use kanso_types::{
    AuthResponse, LoginRequest, OutboxEvent, PullResponse, PushRequest, PushResponse, RegisterRequest,
    RemoteChange,
};
use uuid::Uuid;

/// Register a new account; returns a session token + ids for a fresh device.
pub async fn register(base_url: &str, email: &str, password: &str) -> Result<AuthResponse, String> {
    let base = base_url.trim_end_matches('/');
    let body = RegisterRequest { email: email.to_string(), password: password.to_string() };
    let response = reqwest::Client::new()
        .post(format!("{base}/v1/auth/register"))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("register failed: HTTP {}", response.status()));
    }
    response.json().await.map_err(|e| e.to_string())
}

/// Log in to an existing account; returns a token for a new device session.
pub async fn login(base_url: &str, email: &str, password: &str) -> Result<AuthResponse, String> {
    let base = base_url.trim_end_matches('/');
    let body = LoginRequest { email: email.to_string(), password: password.to_string() };
    let response = reqwest::Client::new()
        .post(format!("{base}/v1/auth/login"))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("login failed: HTTP {}", response.status()));
    }
    response.json().await.map_err(|e| e.to_string())
}

/// Talks to a Kanso Cloud server over HTTP. Cheap to clone-construct; holds a
/// pooled `reqwest::Client`.
pub struct HttpSyncTransport {
    base_url: String,
    client: reqwest::Client,
    token: Option<String>,
}

impl HttpSyncTransport {
    /// `base_url` is the server root, e.g. `https://cloud.kanso.app`. `token` is
    /// the optional bearer token (must match the server's `KANSO_CLOUD_TOKEN`).
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            token,
        }
    }

    fn with_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }
}

#[async_trait]
impl SyncTransport for HttpSyncTransport {
    async fn push(
        &self,
        device_id: &str,
        last_known_server_seq: i64,
        events: Vec<OutboxEvent>,
    ) -> Result<(Vec<Uuid>, i64), String> {
        let request = PushRequest {
            device_id: device_id.to_string(),
            last_known_server_seq,
            events,
        };
        let response = self
            .with_auth(self.client.post(format!("{}/v1/sync/push", self.base_url)).json(&request))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("push failed: HTTP {}", response.status()));
        }
        let body: PushResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok((body.accepted_ids, body.server_high_water))
    }

    async fn pull(
        &self,
        device_id: &str,
        since_server_seq: i64,
        limit: i64,
    ) -> Result<Vec<RemoteChange>, String> {
        // Build the query string directly (avoids pulling reqwest's url-encoding
        // feature). Device ids are `device:<uuid>` — only the colon needs escaping.
        let url = format!(
            "{}/v1/sync/pull?device_id={}&since={}&limit={}",
            self.base_url,
            device_id.replace(':', "%3A"),
            since_server_seq,
            limit,
        );
        let response = self
            .with_auth(self.client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("pull failed: HTTP {}", response.status()));
        }
        let body: PullResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(body.changes)
    }
}
