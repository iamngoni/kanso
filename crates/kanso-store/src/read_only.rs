//! [`ReadOnlyStore`]: reading a library's `meta` while another process (the
//! app, or `kanso serve`) owns it and holds the data-directory lock.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags};

use crate::{Result, Store, schema};

/// A read-only view of a library's database: no migration, no writer, no
/// lock. For reading `meta` while another process owns the library.
pub struct ReadOnlyStore {
    conn: Mutex<Connection>,
}

impl Store {
    /// Opens the database at `path` read-only. A missing file is an error;
    /// nothing is created or migrated, and nothing can be written through it.
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<ReadOnlyStore> {
        Store::open_read_only_with_key(path, None)
    }

    /// [`Store::open_read_only`] for a library encrypted with `key`.
    pub fn open_read_only_with_key(
        path: impl AsRef<Path>,
        key: Option<&crate::LibraryKey>,
    ) -> Result<ReadOnlyStore> {
        if crate::encryption_unfinished(path.as_ref()) {
            return Err(crate::StoreError::Converting);
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        crate::library_key::unlock(&conn, key)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "query_only", "ON")?;
        Ok(ReadOnlyStore {
            conn: Mutex::new(conn),
        })
    }
}

impl ReadOnlyStore {
    /// Reads a value from the `meta` table, as [`Store::meta`] does.
    pub fn meta(&self, key: &str) -> Result<Option<String>> {
        schema::get_meta(&self.conn.lock().expect("connection poisoned"), key)
    }

    /// How many documents have changes the server hasn't accepted, as
    /// [`Store::pending_count`] counts them.
    pub fn pending_count(&self) -> Result<usize> {
        // The same query as `Store::pending_count` (sync.rs).
        let count: i64 = self.conn.lock().expect("connection poisoned").query_row(
            "SELECT count(*) FROM docs WHERE synced_rev IS NULL OR synced_rev != rev",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Runs `sql` on the connection, for showing that writes fail.
    #[cfg(test)]
    fn execute(&self, sql: &str) -> Result<usize> {
        Ok(self
            .conn
            .lock()
            .expect("connection poisoned")
            .execute(sql, [])?)
    }
}

#[cfg(test)]
mod tests {
    // no-prop-test: a thin wrapper over a read-only SQLite connection and
    // `schema::get_meta`; the cases below are the whole behaviour.
    use super::*;
    use crate::{Origin, Put, StoreError};
    use kanso_core::DocType;

    #[test]
    fn read_only_open_works_beside_a_writer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanso.db");
        let store = Store::open(&path).unwrap();
        store.set_meta("library_id", "l_1").unwrap();
        let view = Store::open_read_only(&path).unwrap();
        assert_eq!(view.meta("library_id").unwrap().as_deref(), Some("l_1"));
        assert_eq!(view.meta("missing").unwrap(), None);

        // It sees the writer's later commits, and counts pending changes
        // as the writer does.
        store.set_meta("library_id", "l_2").unwrap();
        store
            .put(
                DocType::Note,
                serde_json::json!({"_id": "note:ro1", "bookId": "trash", "body": "v1", "updatedAt": 100}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap();
        assert_eq!(view.meta("library_id").unwrap().as_deref(), Some("l_2"));
        assert_eq!(
            view.pending_count().unwrap(),
            store.pending_count().unwrap()
        );
        assert_eq!(view.pending_count().unwrap(), 1);

        // Writing through it fails, and changes nothing.
        assert!(
            view.execute("INSERT INTO meta (key, value) VALUES ('x', 'y')")
                .is_err()
        );
        assert!(view.execute("DELETE FROM meta").is_err());
        assert_eq!(store.meta("x").unwrap(), None);
        assert_eq!(store.meta("library_id").unwrap().as_deref(), Some("l_2"));

        // The writer keeps writing while it is open.
        store.set_meta("library_id", "l_3").unwrap();
        assert_eq!(view.meta("library_id").unwrap().as_deref(), Some("l_3"));
    }

    #[test]
    fn read_only_open_works_with_no_writer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanso.db");
        Store::open(&path).unwrap().set_meta("k", "v").unwrap();
        let view = Store::open_read_only(&path).unwrap();
        assert_eq!(view.meta("k").unwrap().as_deref(), Some("v"));
    }

    #[test]
    fn a_missing_database_is_an_error_and_is_not_created() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanso.db");
        assert!(matches!(
            Store::open_read_only(&path),
            Err(StoreError::Sqlite(_))
        ));
        assert!(!path.exists());
    }
}
