//! Kanso Cloud — the first-party sync service.
//!
//! Ordered event replication per user: clients `push` their outbox, the server
//! assigns an authoritative monotonic sequence, and clients `pull` everything
//! after their last-seen sequence. Attachment blobs are content-addressed and
//! exchanged out of band via presigned URLs (not yet wired here).

mod store;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web};
use kanso_types::{PullResponse, PushRequest, PushResponse};
use serde::Deserialize;

use crate::store::{EventStore, MemoryStore};

type Store = web::Data<Arc<dyn EventStore>>;

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn push(store: Store, body: web::Json<PushRequest>) -> HttpResponse {
    let req = body.into_inner();
    let (accepted_ids, server_high_water) = store.append(req.events);
    log::info!(
        "push device={} accepted={} high_water={}",
        req.device_id,
        accepted_ids.len(),
        server_high_water
    );
    HttpResponse::Ok().json(PushResponse { accepted_ids, server_high_water })
}

#[derive(Debug, Deserialize)]
struct PullQuery {
    #[serde(default)]
    since: i64,
    limit: Option<i64>,
}

async fn pull(store: Store, query: web::Query<PullQuery>) -> HttpResponse {
    let limit = query.limit.unwrap_or(500).clamp(1, 5_000) as usize;
    let changes = store.since(query.since, limit);
    HttpResponse::Ok().json(PullResponse {
        changes,
        server_high_water: store.high_water(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let store: Arc<dyn EventStore> = Arc::new(MemoryStore::default());
    let data: web::Data<Arc<dyn EventStore>> = web::Data::new(store);

    let bind = std::env::var("KANSO_CLOUD_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    log::info!("kanso-cloud listening on http://{bind}");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .wrap(actix_web::middleware::Logger::default())
            .route("/health", web::get().to(health))
            .route("/v1/sync/push", web::post().to(push))
            .route("/v1/sync/pull", web::get().to(pull))
    })
    .bind(&bind)?
    .run()
    .await
}
