//! The notes under embargo, for the reminder that says when each lifts.

use kanso_core::embargo;
use kanso_core::model::LOCK_FIELD;
use kanso_core::reminders::{ACK_FIELD, read_acks};

use crate::embargo_model::EmbargoNote;
use crate::{Result, Store};

impl Store {
    /// The live notes outside the trash with an embargo Kanso can read,
    /// lifted or not, each with its reminder acks. A locked note's text is
    /// sealed, so it has none. Only notes whose text names the property are
    /// read.
    pub fn embargo_notes(&self) -> Result<Vec<EmbargoNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, d.title, d.book_id, json_extract(d.json, '$.body'),
                        json_extract(d.json, '$.{ACK_FIELD}')
                 FROM docs d
                 WHERE d.type = 'note' AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                   AND json_extract(d.json, '$.{LOCK_FIELD}') IS NULL
                   AND instr(lower(d.json), 'embargo') > 0
                 ORDER BY d.id"
            ))?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    row.get::<_, Option<String>>(4)?,
                ))
            })?;
            let mut out = Vec::new();
            for row in rows {
                let (note_id, note_title, book_id, body, acks) = row?;
                let Some(found) = embargo::of(&body) else {
                    continue;
                };
                let mut fields = serde_json::Map::new();
                if let Some(acks) = acks.and_then(|a| serde_json::from_str(&a).ok()) {
                    fields.insert(ACK_FIELD.into(), acks);
                }
                out.push(EmbargoNote {
                    note_id,
                    note_title,
                    book_id,
                    embargo: found,
                    acks: read_acks(&fields),
                });
            }
            Ok(out)
        })
    }
}
