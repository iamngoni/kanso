//! kanso-cloud — the sync service binary.
//!
//! Env:
//! - `DATABASE_URL`     → use Postgres for events (else in-memory)
//! - `KANSO_JWT_SECRET` → HS256 signing secret (required in production)
//! - `KANSO_CLOUD_BIND` → listen address (default 127.0.0.1:8787)

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use kanso_cloud::accounts::{AccountStore, MemoryAccountStore, PostgresAccountStore};
use kanso_cloud::auth::JwtKeys;
use kanso_cloud::routes;
use kanso_cloud::store::{EventStore, MemoryStore, PostgresStore};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let (event_store, account_store): (Arc<dyn EventStore>, Arc<dyn AccountStore>) =
        match std::env::var("DATABASE_URL") {
            Ok(url) => {
                log::info!("DATABASE_URL set — connecting to Postgres");
                let pg_events = match PostgresStore::connect(&url).await {
                    Ok(pg) => pg,
                    Err(e) => {
                        log::error!("failed to connect event store to Postgres: {e}");
                        std::process::exit(1);
                    }
                };
                let pg_accounts = match PostgresAccountStore::connect(&url).await {
                    Ok(pg) => pg,
                    Err(e) => {
                        log::error!("failed to connect account store to Postgres: {e}");
                        std::process::exit(1);
                    }
                };
                (Arc::new(pg_events), Arc::new(pg_accounts))
            }
            Err(_) => {
                log::info!("DATABASE_URL not set — using in-memory stores (non-durable)");
                (
                    Arc::new(MemoryStore::default()),
                    Arc::new(MemoryAccountStore::default()),
                )
            }
        };

    let secret = match std::env::var("KANSO_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            log::warn!(
                "KANSO_JWT_SECRET not set — using an ephemeral dev secret (tokens reset on restart)"
            );
            "dev-only-insecure-secret".to_string()
        }
    };
    let keys = JwtKeys::new(secret.as_bytes());

    // Blob storage is in-memory for now; object storage (S3/R2) slots in here.
    let blob_store: std::sync::Arc<dyn kanso_cloud::blobs::BlobStore> =
        std::sync::Arc::new(kanso_cloud::blobs::MemoryBlobStore::default());

    let event_data = web::Data::new(event_store);
    let account_data = web::Data::new(account_store);
    let jwt_data = web::Data::new(keys);
    let blob_data = web::Data::new(blob_store);

    let bind = std::env::var("KANSO_CLOUD_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    log::info!("kanso-cloud listening on http://{bind}");

    HttpServer::new(move || {
        App::new()
            .app_data(event_data.clone())
            .app_data(account_data.clone())
            .app_data(jwt_data.clone())
            .app_data(blob_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(Logger::default())
            .configure(routes)
    })
    .bind(&bind)?
    .run()
    .await
}
