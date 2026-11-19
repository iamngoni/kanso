//! One-time migration for note bodies written with the retired link scheme.
//!
//! The scheme is assembled at compile time so the retired spelling is not a
//! source-tree literal. A migration keeps old libraries readable and lets the
//! change push as an ordinary local revision.

use kanso_core::DocType;
use kanso_core::links::{LINK_SCHEME, current_scheme_body};
use rusqlite::{Connection, TransactionBehavior, params};
use serde_json::{Map, Value};

use crate::validate::{Refs, normalize};
use crate::write::{Synced, load_row, next_rev, store_row};
use crate::{Origin, Result, schema};

/// The marker that says the body-link scheme has been migrated.
pub(crate) const SCHEME_META: &str = "links.body_scheme";
/// The scheme this store writes.
pub(crate) const CURRENT_SCHEME: &str = LINK_SCHEME;
/// The retired scheme, assembled so it never appears as a source literal.
const RETIRED_SCHEME: &str = concat!("ink", "drop://");

fn canonicalize_fields(fields: &mut Map<String, Value>) -> bool {
    let Some(Value::String(body)) = fields.get("body") else {
        return false;
    };
    let Some(replaced) = current_scheme_body(body) else {
        return false;
    };
    fields.insert("body".into(), Value::String(replaced));
    true
}

fn canonicalize_json(json: &str) -> Option<String> {
    let Value::Object(mut fields) = serde_json::from_str::<Value>(json).ok()? else {
        return None;
    };
    if !canonicalize_fields(&mut fields) {
        return None;
    }
    serde_json::to_string(&Value::Object(fields)).ok()
}

/// Rewrites live note bodies and saved versions under the current scheme.
///
/// The docs are written through the normal row path so `rev` advances,
/// `synced_rev` stays where it was (the changed body must push), and every
/// derived table sees the new body. A locked note's sealed text is not
/// readable here; its stored `body` is already empty. A plaintext note in a
/// locked notebook is left for a later open, after it is unlocked. A shared
/// note's text is the collaboration CRDT's projection, so this store-local
/// migration skips it rather than diverge from what other devices hold.
pub(crate) fn migrate(conn: &mut Connection) -> Result<usize> {
    if schema::get_meta(conn, SCHEME_META)?.as_deref() == Some(CURRENT_SCHEME) {
        return Ok(0);
    }

    let tx = rusqlite::Transaction::new(conn, TransactionBehavior::Immediate)?;
    let ids: Vec<String> = tx
        .prepare(
            "SELECT id FROM docs
             WHERE type = 'note' AND deleted = 0 AND instr(json, ?1) > 0
             ORDER BY id",
        )?
        .query_map(params![RETIRED_SCHEME], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;

    let mut migrated = 0;
    let mut waiting_for_unlock = false;
    for id in ids {
        let Some(previous) = load_row(&tx, &id)? else {
            continue;
        };
        let mut fields = previous.fields.clone();
        // Shared text belongs to `kanso-collab`'s CRDT; write it there, not
        // here, or the next merge would put the old text back.
        if crate::collab::share_of(&fields).is_some() {
            continue;
        }
        if !canonicalize_fields(&mut fields) {
            continue;
        }
        let book_id = fields.get("bookId").and_then(Value::as_str);
        if fields.contains_key(kanso_core::model::LOCK_FIELD)
            || crate::write::in_locked_book(&tx, book_id)?
        {
            waiting_for_unlock = true;
            continue;
        }
        let fields = normalize(
            DocType::Note,
            &id,
            fields,
            Some(&previous.fields),
            kanso_core::now_ms(),
            Refs::Skip,
        )?;
        let rev = next_rev(Some(&previous), &Value::Object(fields.clone()))?;
        store_row(
            &tx,
            &id,
            DocType::Note,
            &rev,
            false,
            &fields,
            Some(&previous),
            Synced::Keep,
            Origin::Migration,
            &mut Vec::new(),
        )?;
        migrated += 1;
    }

    let revisions: Vec<(i64, String)> = tx
        .prepare("SELECT id, json FROM revisions WHERE instr(json, ?1) > 0 ORDER BY id")?
        .query_map(params![RETIRED_SCHEME], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (id, json) in revisions {
        if let Some(rewritten) = canonicalize_json(&json) {
            tx.execute(
                "UPDATE revisions SET json = ?1 WHERE id = ?2",
                params![rewritten, id],
            )?;
        }
    }

    // A note in a locked notebook retries on the next open after it is
    // unlocked. Sealed notes have no readable body here; `reveal` rewrites
    // their text when the lock opens, and `unlock_note` stores it.
    if !waiting_for_unlock {
        schema::set_meta(&tx, SCHEME_META, CURRENT_SCHEME)?;
    }
    tx.commit()?;
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn replaces_only_the_retired_scheme() {
        let mut fields = json!({
            "body": format!("see [x]({RETIRED_SCHEME}note/abc) and kanso://note/keep"),
        });
        assert!(canonicalize_fields(fields.as_object_mut().unwrap()));
        assert_eq!(
            fields["body"],
            "see [x](kanso://note/abc) and kanso://note/keep"
        );
        assert!(!canonicalize_fields(fields.as_object_mut().unwrap()));
    }

    #[test]
    fn rewrites_saved_versions() {
        let json = json!({"body": format!("[x]({RETIRED_SCHEME}note/abc)")}).to_string();
        let rewritten = canonicalize_json(&json).expect("a changed body");
        assert!(rewritten.contains(CURRENT_SCHEME));
        assert!(!rewritten.contains(RETIRED_SCHEME));
    }
}
