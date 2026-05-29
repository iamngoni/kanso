//! Kanso Cloud library — routes, handlers, stores, and auth.
//!
//! Shared by the `kanso-cloud` binary and integration tests. Sync is ordered,
//! per-user, origin-aware event replication; auth is JWT (HS256). Push/pull
//! derive the user and device from the verified token, never the request body.

pub mod accounts;
pub mod auth;
pub mod blobs;
pub mod dto;
pub mod store;

use std::sync::Arc;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{HttpRequest, HttpResponse, http::header::AUTHORIZATION, web};
use kanso_types::{AuthResponse, LoginRequest, PullResponse, PushRequest, PushResponse, RegisterRequest};

use crate::accounts::AccountStore;
use crate::auth::{Claims, JwtKeys};
use crate::blobs::{BlobStore, content_hash};
use crate::dto::PullQuery;
use crate::store::EventStore;

/// 30-day token lifetime — these are long-lived device sessions.
const TOKEN_TTL_SECS: i64 = 60 * 60 * 24 * 30;

type StoreData = web::Data<Arc<dyn EventStore>>;
type AccountData = web::Data<Arc<dyn AccountStore>>;
type JwtData = web::Data<JwtKeys>;
type BlobData = web::Data<Arc<dyn BlobStore>>;

/// Extract and verify the bearer token, returning its claims or a 401.
fn require_auth(req: &HttpRequest, keys: &JwtKeys) -> Result<Claims, HttpResponse> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    match token {
        Some(token) => keys.verify(token).map_err(|_| HttpResponse::Unauthorized().finish()),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn register(
    accounts: AccountData,
    keys: JwtData,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let user_id = match accounts.register(&req.email, &req.password).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::Conflict().json(serde_json::json!({ "error": e.to_string() })),
    };
    issue_session(&accounts, &keys, &user_id).await
}

async fn login(accounts: AccountData, keys: JwtData, body: web::Json<LoginRequest>) -> HttpResponse {
    let req = body.into_inner();
    let user_id = match accounts.login(&req.email, &req.password).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "invalid credentials" })),
    };
    issue_session(&accounts, &keys, &user_id).await
}

/// Register a fresh device for the user and mint a token for it.
async fn issue_session(accounts: &AccountData, keys: &JwtData, user_id: &str) -> HttpResponse {
    let device_id = match accounts.register_device(user_id, "device").await {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };
    match keys.issue(user_id, &device_id, TOKEN_TTL_SECS) {
        Ok(token) => HttpResponse::Ok().json(AuthResponse {
            token,
            user_id: user_id.to_string(),
            device_id,
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Reissue a fresh token for the same user+device (sliding session). Requires a
/// currently-valid bearer token.
async fn refresh(req: HttpRequest, keys: JwtData) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match keys.issue(&claims.sub, &claims.device_id, TOKEN_TTL_SECS) {
        Ok(token) => HttpResponse::Ok().json(AuthResponse {
            token,
            user_id: claims.sub,
            device_id: claims.device_id,
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn push(
    req: HttpRequest,
    store: StoreData,
    keys: JwtData,
    body: web::Json<PushRequest>,
) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let request = body.into_inner();
    // Scope by the authenticated user + device, not the (spoofable) body fields.
    let (accepted_ids, server_high_water) =
        store.append(&claims.sub, &claims.device_id, request.events).await;
    HttpResponse::Ok().json(PushResponse { accepted_ids, server_high_water })
}

async fn pull(
    req: HttpRequest,
    store: StoreData,
    keys: JwtData,
    query: web::Query<PullQuery>,
) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let limit = query.limit.unwrap_or(500).clamp(1, 5_000) as usize;
    let changes = store.since(&claims.sub, &claims.device_id, query.since, limit).await;
    let server_high_water = store.high_water(&claims.sub).await;
    HttpResponse::Ok().json(PullResponse { changes, server_high_water })
}

async fn put_blob(
    req: HttpRequest,
    blobs: BlobData,
    keys: JwtData,
    path: web::Path<String>,
    body: web::Bytes,
) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let hash = path.into_inner();
    let actual = content_hash(&body);
    if actual != hash {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "content hash mismatch", "expected": hash, "actual": actual }));
    }
    let size = body.len();
    blobs.put(&claims.sub, &hash, body.to_vec()).await;
    HttpResponse::Ok().json(serde_json::json!({ "hash": hash, "size": size }))
}

async fn get_blob(
    req: HttpRequest,
    blobs: BlobData,
    keys: JwtData,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match blobs.get(&claims.sub, &path.into_inner()).await {
        Some(bytes) => HttpResponse::Ok().content_type("application/octet-stream").body(bytes),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn blob_exists(
    req: HttpRequest,
    blobs: BlobData,
    keys: JwtData,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &keys) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let exists = blobs.exists(&claims.sub, &path.into_inner()).await;
    HttpResponse::Ok().json(serde_json::json!({ "exists": exists }))
}

/// Register all routes. The caller supplies `web::Data` for the event store,
/// account store (`Arc<dyn AccountStore>`), [`JwtKeys`], and the blob store
/// (`Arc<dyn BlobStore>`).
pub fn routes(cfg: &mut web::ServiceConfig) {
    // Throttle the unauthenticated auth surface to blunt credential stuffing.
    // Token bucket: burst of 10, refilling one request every 2s, keyed by peer
    // IP. (Per-worker limiter — approximate under multiple workers; a shared
    // store is the production upgrade.)
    let auth_governor = GovernorConfigBuilder::default()
        .seconds_per_request(2)
        .burst_size(10)
        .finish()
        .expect("valid governor config");

    cfg.route("/health", web::get().to(health))
        .service(
            web::scope("/v1/auth")
                .wrap(Governor::new(&auth_governor))
                .route("/register", web::post().to(register))
                .route("/login", web::post().to(login))
                .route("/refresh", web::post().to(refresh)),
        )
        .route("/v1/sync/push", web::post().to(push))
        .route("/v1/sync/pull", web::get().to(pull))
        .route("/v1/blobs/{hash}", web::put().to(put_blob))
        .route("/v1/blobs/{hash}", web::get().to(get_blob))
        .route("/v1/blobs/{hash}/exists", web::get().to(blob_exists));
}
