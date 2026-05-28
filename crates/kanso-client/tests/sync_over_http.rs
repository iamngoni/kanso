//! End-to-end auth + sync over real HTTP: a live Kanso Cloud server (in-memory
//! stores) mounted via `actix-test`. Proves multi-device convergence on one
//! account, isolation between accounts, and that auth is required.

use std::sync::Arc;

use actix_web::{App, web};
use kanso_client::HttpSyncTransport;
use kanso_cloud::accounts::{AccountStore, MemoryAccountStore};
use kanso_cloud::auth::JwtKeys;
use kanso_cloud::routes;
use kanso_cloud::store::{EventStore, MemoryStore};
use kanso_engine::Engine;

fn test_server() -> actix_test::TestServer {
    let events: Arc<dyn EventStore> = Arc::new(MemoryStore::default());
    let accounts: Arc<dyn AccountStore> = Arc::new(MemoryAccountStore::default());
    let keys = JwtKeys::new(b"integration-test-secret");

    let events_data = web::Data::new(events);
    let accounts_data = web::Data::new(accounts);
    let jwt_data = web::Data::new(keys);

    actix_test::start(move || {
        App::new()
            .app_data(events_data.clone())
            .app_data(accounts_data.clone())
            .app_data(jwt_data.clone())
            .configure(routes)
    })
}

#[actix_web::test]
async fn two_devices_on_one_account_converge() {
    let srv = test_server();
    let base = srv.url("");

    // Same account, two device sessions.
    let dev_a = kanso_client::register(&base, "a@example.com", "hunter2").await.unwrap();
    let dev_b = kanso_client::login(&base, "a@example.com", "hunter2").await.unwrap();
    assert_eq!(dev_a.user_id, dev_b.user_id);
    assert_ne!(dev_a.device_id, dev_b.device_id);

    let a = Engine::open_in_memory().await.unwrap();
    let b = Engine::open_in_memory().await.unwrap();
    let ta = HttpSyncTransport::new(base.clone(), Some(dev_a.token));
    let tb = HttpSyncTransport::new(base, Some(dev_b.token));

    let nb = a.create_notebook("Shared", None).await.unwrap();
    let note = a.create_note(&nb.id, "Hello", "from device A").await.unwrap();
    a.sync(&dev_a.device_id, &ta).await.unwrap();

    let report = b.sync(&dev_b.device_id, &tb).await.unwrap();
    assert!(report.applied >= 2);
    assert_eq!(
        b.get_note(&note.id).await.unwrap().unwrap().body_markdown,
        "from device A"
    );
}

#[actix_web::test]
async fn accounts_are_isolated() {
    let srv = test_server();
    let base = srv.url("");

    let user1 = kanso_client::register(&base, "u1@example.com", "pw").await.unwrap();
    let user2 = kanso_client::register(&base, "u2@example.com", "pw").await.unwrap();
    assert_ne!(user1.user_id, user2.user_id);

    // User 1 creates and syncs private content.
    let a = Engine::open_in_memory().await.unwrap();
    let t1 = HttpSyncTransport::new(base.clone(), Some(user1.token));
    let nb = a.create_notebook("Private", None).await.unwrap();
    a.create_note(&nb.id, "secret", "user 1 only").await.unwrap();
    a.sync(&user1.device_id, &t1).await.unwrap();

    // User 2 syncs and sees nothing.
    let c = Engine::open_in_memory().await.unwrap();
    let t2 = HttpSyncTransport::new(base, Some(user2.token));
    let report = c.sync(&user2.device_id, &t2).await.unwrap();
    assert_eq!(report.applied, 0);
    assert_eq!(c.list_notebooks().await.unwrap().len(), 0);
}

#[actix_web::test]
async fn missing_token_is_rejected() {
    let srv = test_server();
    let base = srv.url("");

    let a = Engine::open_in_memory().await.unwrap();
    let nb = a.create_notebook("x", None).await.unwrap();
    a.create_note(&nb.id, "t", "b").await.unwrap();

    let no_auth = HttpSyncTransport::new(base, None);
    assert!(a.sync("device:anon", &no_auth).await.is_err());
}
