//! Account and device store.
//!
//! [`MemoryAccountStore`] is the in-process implementation used for dev and
//! tests. A Postgres-backed store slots in behind [`AccountStore`] later, the
//! same way [`crate::store::PostgresStore`] does for events.

use std::collections::HashMap;
use std::sync::Mutex;

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("email already registered")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("password hashing failed: {0}")]
    Hash(String),
    #[error("account backend error: {0}")]
    Backend(String),
}

#[async_trait]
pub trait AccountStore: Send + Sync {
    /// Register a new account, returning its user id.
    async fn register(&self, email: &str, password: &str) -> Result<String, AccountError>;
    /// Verify credentials, returning the user id.
    async fn login(&self, email: &str, password: &str) -> Result<String, AccountError>;
    /// Register a device for a user, returning the device id.
    async fn register_device(&self, user_id: &str, name: &str) -> Result<String, AccountError>;
}

fn hash_password(password: &str) -> Result<String, AccountError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AccountError::Hash(e.to_string()))
}

fn verify_password(password: &str, phc: &str) -> bool {
    PasswordHash::new(phc)
        .ok()
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

#[derive(Default)]
struct Inner {
    /// email -> (user_id, password hash (PHC string))
    by_email: HashMap<String, (String, String)>,
    /// user_id -> device ids
    devices: HashMap<String, Vec<String>>,
}

#[derive(Default)]
pub struct MemoryAccountStore {
    inner: Mutex<Inner>,
}

#[async_trait]
impl AccountStore for MemoryAccountStore {
    async fn register(&self, email: &str, password: &str) -> Result<String, AccountError> {
        let phc = hash_password(password)?;
        let mut inner = self.inner.lock().expect("account mutex poisoned");
        if inner.by_email.contains_key(email) {
            return Err(AccountError::EmailTaken);
        }
        let user_id = format!("user:{}", Uuid::now_v7());
        inner
            .by_email
            .insert(email.to_string(), (user_id.clone(), phc));
        Ok(user_id)
    }

    async fn login(&self, email: &str, password: &str) -> Result<String, AccountError> {
        let inner = self.inner.lock().expect("account mutex poisoned");
        let (user_id, phc) = inner
            .by_email
            .get(email)
            .ok_or(AccountError::InvalidCredentials)?;
        if verify_password(password, phc) {
            Ok(user_id.clone())
        } else {
            Err(AccountError::InvalidCredentials)
        }
    }

    async fn register_device(&self, user_id: &str, _name: &str) -> Result<String, AccountError> {
        let device_id = format!("device:{}", Uuid::now_v7());
        self.inner
            .lock()
            .expect("account mutex poisoned")
            .devices
            .entry(user_id.to_string())
            .or_default()
            .push(device_id.clone());
        Ok(device_id)
    }
}

// ─── PostgresAccountStore ─────────────────────────────────────────────────────

/// Postgres-backed [`AccountStore`]. Durable; migrations run on startup.
pub struct PostgresAccountStore {
    pool: sqlx::PgPool,
}

impl PostgresAccountStore {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl AccountStore for PostgresAccountStore {
    async fn register(&self, email: &str, password: &str) -> Result<String, AccountError> {
        let phc = hash_password(password)?;
        let user_id = format!("user:{}", uuid::Uuid::now_v7());
        let result =
            sqlx::query("INSERT INTO users (user_id, email, password_hash) VALUES ($1, $2, $3)")
                .bind(&user_id)
                .bind(email)
                .bind(&phc)
                .execute(&self.pool)
                .await;

        match result {
            Ok(_) => Ok(user_id),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(AccountError::EmailTaken)
            }
            Err(e) => Err(AccountError::Backend(e.to_string())),
        }
    }

    async fn login(&self, email: &str, password: &str) -> Result<String, AccountError> {
        use sqlx::Row;
        let row = sqlx::query("SELECT user_id, password_hash FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccountError::Backend(e.to_string()))?;

        let row = row.ok_or(AccountError::InvalidCredentials)?;
        let user_id: String = row
            .try_get("user_id")
            .map_err(|e| AccountError::Backend(e.to_string()))?;
        let phc: String = row
            .try_get("password_hash")
            .map_err(|e| AccountError::Backend(e.to_string()))?;

        if verify_password(password, &phc) {
            Ok(user_id)
        } else {
            Err(AccountError::InvalidCredentials)
        }
    }

    async fn register_device(&self, user_id: &str, name: &str) -> Result<String, AccountError> {
        let device_id = format!("device:{}", uuid::Uuid::now_v7());
        sqlx::query("INSERT INTO devices (device_id, user_id, name) VALUES ($1, $2, $3)")
            .bind(&device_id)
            .bind(user_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| AccountError::Backend(e.to_string()))?;

        Ok(device_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_login_and_reject() {
        let store = MemoryAccountStore::default();
        let uid = store.register("a@example.com", "hunter2").await.unwrap();

        // Duplicate email rejected.
        assert!(store.register("a@example.com", "other").await.is_err());

        // Correct password logs in to the same user.
        assert_eq!(store.login("a@example.com", "hunter2").await.unwrap(), uid);

        // Wrong password rejected.
        assert!(store.login("a@example.com", "nope").await.is_err());

        // Device registration yields distinct ids.
        let d1 = store.register_device(&uid, "mac").await.unwrap();
        let d2 = store.register_device(&uid, "phone").await.unwrap();
        assert_ne!(d1, d2);
    }
}
