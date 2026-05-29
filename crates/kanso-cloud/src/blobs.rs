//! Attachment blob storage — content-addressed, per-user.
//!
//! Attachment *content* travels out of band from the event log: the client
//! uploads bytes keyed by their SHA-256, and references them from sync payloads
//! by hash. [`MemoryBlobStore`] is the dev/test backend; object storage
//! (S3/R2/GCS) slots in behind [`BlobStore`] for production.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sha2::{Digest, Sha256};

#[async_trait]
pub trait BlobStore: Send + Sync {
    async fn put(&self, user_id: &str, hash: &str, bytes: Vec<u8>);
    async fn get(&self, user_id: &str, hash: &str) -> Option<Vec<u8>>;
    async fn exists(&self, user_id: &str, hash: &str) -> bool;
}

/// In-memory [`BlobStore`], keyed by `(user_id, hash)` so accounts are isolated.
#[derive(Default)]
pub struct MemoryBlobStore {
    blobs: Mutex<HashMap<(String, String), Vec<u8>>>,
}

#[async_trait]
impl BlobStore for MemoryBlobStore {
    async fn put(&self, user_id: &str, hash: &str, bytes: Vec<u8>) {
        self.blobs
            .lock()
            .expect("blob mutex poisoned")
            .insert((user_id.to_string(), hash.to_string()), bytes);
    }

    async fn get(&self, user_id: &str, hash: &str) -> Option<Vec<u8>> {
        self.blobs
            .lock()
            .expect("blob mutex poisoned")
            .get(&(user_id.to_string(), hash.to_string()))
            .cloned()
    }

    async fn exists(&self, user_id: &str, hash: &str) -> bool {
        self.blobs
            .lock()
            .expect("blob mutex poisoned")
            .contains_key(&(user_id.to_string(), hash.to_string()))
    }
}

/// Hex-encoded SHA-256 of `bytes` — the content address.
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
