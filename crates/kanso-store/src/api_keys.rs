//! The API keys table (issue #243): the keys that open the local API.
//!
//! The table is local and never synced. It holds the SHA-256 of each key and
//! nothing that could rebuild one.

use rusqlite::{Row, params};

use crate::api_keys_model::{ApiKeyRow, KeyAccess, LiveKey, NewApiKey};
use crate::{Result, Store, StoreError};

const COLUMNS: &str = "id, name, client, access, books, created_at, last_used_at, revoked_at";

fn read(row: &Row) -> rusqlite::Result<ApiKeyRow> {
    let access: String = row.get(3)?;
    let books: Option<String> = row.get(4)?;
    Ok(ApiKeyRow {
        id: row.get(0)?,
        name: row.get(1)?,
        client: row.get(2)?,
        // The table's CHECK keeps this to the three; a stranger reads as the
        // least access, never the most.
        access: KeyAccess::parse(&access).unwrap_or(KeyAccess::Read),
        // A list that doesn't parse reads as no notebooks, never as all.
        books: books.map(|json| serde_json::from_str(&json).unwrap_or_default()),
        created_at: row.get(5)?,
        last_used_at: row.get(6)?,
        revoked_at: row.get(7)?,
    })
}

impl Store {
    /// Saves a key.
    pub fn api_key_insert(&self, new: &NewApiKey) -> Result<()> {
        let books = new.books.as_ref().map(serde_json::to_string).transpose()?;
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO api_keys (id, name, client, token_hash, access, books, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    new.id,
                    new.name,
                    new.client,
                    new.token_hash.as_slice(),
                    new.access.as_str(),
                    books,
                    new.created_at
                ],
            )
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StoreError::Invalid("that API key already exists".into())
                }
                other => other.into(),
            })?;
            Ok(())
        })
    }

    /// Every key, revoked ones included, newest first.
    pub fn api_keys_all(&self) -> Result<Vec<ApiKeyRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {COLUMNS} FROM api_keys ORDER BY created_at DESC, id"
            ))?;
            let rows = stmt.query_map([], read)?.collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
    }

    /// The keys not revoked, with their hashes.
    pub fn api_keys_live(&self) -> Result<Vec<LiveKey>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {COLUMNS}, token_hash FROM api_keys WHERE revoked_at IS NULL \
                 ORDER BY created_at, id"
            ))?;
            let rows = stmt
                .query_map([], |row| {
                    let hash: Vec<u8> = row.get(8)?;
                    Ok((read(row)?, hash))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            // A hash of the wrong length can't match anything; leave it out.
            Ok(rows
                .into_iter()
                .filter_map(|(row, hash)| {
                    Some(LiveKey {
                        row,
                        token_hash: hash.try_into().ok()?,
                    })
                })
                .collect())
        })
    }

    /// Revokes key `id` at `now_ms`. `false` when there is no such live key.
    pub fn api_key_revoke(&self, id: &str, now_ms: i64) -> Result<bool> {
        self.write(|tx, _| {
            let changed = tx.execute(
                "UPDATE api_keys SET revoked_at = ?2 WHERE id = ?1 AND revoked_at IS NULL",
                params![id, now_ms],
            )?;
            Ok(changed > 0)
        })
    }

    /// Records that key `id` was used at `now_ms`. A revoked key is left as
    /// it was.
    pub fn api_key_touch(&self, id: &str, now_ms: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE api_keys SET last_used_at = ?2 WHERE id = ?1 AND revoked_at IS NULL",
                params![id, now_ms],
            )?;
            Ok(())
        })
    }
}
