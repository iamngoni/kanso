//! Meeting-note creation and event-to-note resolution.

use std::collections::HashMap;

use kanso_core::entities::EntityKind;
use kanso_core::properties::{self, Value};
use kanso_core::roles::Role;
use kanso_core::{DocType, now_ms};
use kanso_store::{Origin, Put, Result, Store};

use super::model::{Event, MeetingLink};

#[cfg(test)]
fn unique_person<'a>(rows: impl Iterator<Item = &'a kanso_store::EntityRow>) -> Option<&'a str> {
    let mut people = rows.filter(|row| row.kind == EntityKind::Person);
    let title = people.next()?.title.as_str();
    people.next().is_none().then_some(title)
}

fn attendee_page(store: &Store, attendee: &super::model::Attendee) -> Result<Option<String>> {
    if let Some(id) = attendee.page.as_deref()
        && store
            .entity(id)?
            .is_some_and(|row| row.kind == EntityKind::Person)
    {
        return Ok(Some(id.to_owned()));
    }
    let named = store.entities_named(attendee.name.trim())?;
    let mut named_people = named.iter().filter(|row| row.kind == EntityKind::Person);
    if let Some(person) = named_people.next()
        && named_people.next().is_none()
    {
        return Ok(Some(person.id.clone()));
    }
    let email = attendee.email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(None);
    }
    let mut matched = None;
    for row in store.entities()? {
        if row.kind != EntityKind::Person {
            continue;
        }
        let Some(note) = store.note(&row.id)? else {
            continue;
        };
        let props = properties::parse(&note.body);
        let email_matches = ["email", "emails"]
            .into_iter()
            .filter_map(|key| properties::get(&props, key))
            .any(|value| match value {
                Value::Text(text) => text.trim().eq_ignore_ascii_case(&email),
                Value::List(values) => values.iter().any(|candidate| match candidate {
                    Value::Text(text) => text.trim().eq_ignore_ascii_case(&email),
                    _ => false,
                }),
                _ => false,
            });
        if email_matches {
            if matched.is_some() {
                return Ok(None);
            }
            matched = Some(row.id);
        }
    }
    Ok(matched)
}

/// Creates a Journal note linked to this event instance.
pub fn create_note(
    store: &Store,
    event: &Event,
    local_offset_ms: i64,
    context: &str,
) -> Result<String> {
    let offset_seconds = (local_offset_ms / 1000).clamp(-86_400, 86_400) as i32;
    let date = chrono::DateTime::from_timestamp_millis(event.start)
        .map(|at| {
            let offset = chrono::FixedOffset::east_opt(offset_seconds)
                .unwrap_or(chrono::FixedOffset::east_opt(0).expect("UTC offset is valid"));
            at.with_timezone(&offset)
                .format("%Y-%m-%dT%H:%M:%S%:z")
                .to_string()
        })
        .unwrap_or_default();
    let mut body = String::new();
    body = properties::set_property(&body, "event", &Value::Text(event.uid.clone()));
    body = properties::set_property(&body, "when", &Value::Text(date));
    if !event.attendees.is_empty() {
        body.push_str("# Attendees\n\n");
        for attendee in &event.attendees {
            let name = attendee.name.trim();
            if name.is_empty() {
                continue;
            }
            let title = attendee_page(store, attendee)?
                .and_then(|id| store.entity(&id).ok().flatten())
                .map(|row| row.title)
                .unwrap_or_else(|| name.to_owned());
            body.push_str("- [[");
            body.push_str(&title);
            body.push_str("]]\n");
        }
        body.push('\n');
    }
    if !context.trim().is_empty() {
        body.push_str(context.trim());
        body.push_str("\n\n");
    }
    body.push_str("\n# Notes\n\n## Actions\n- [ ] \n");
    let title = if event.title.trim().is_empty() {
        "Meeting"
    } else {
        event.title.trim()
    };
    let book_id = store.role_book(Role::Journal, Origin::Ui)?;
    let result = store.put(
        DocType::Note,
        serde_json::json!({
            "title": title,
            "body": body,
            "bookId": book_id,
            "createdAt": now_ms(),
        }),
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(result.id)
}

/// Links an existing readable note to this specific event occurrence by
/// writing the same portable front-matter properties as a new meeting note.
pub fn link_note(store: &Store, note_id: &str, event: &Event, local_offset_ms: i64) -> Result<()> {
    let Some(mut note) = store.note(note_id)? else {
        return Err(kanso_store::StoreError::Invalid(
            "That note is unavailable.".into(),
        ));
    };
    let when = chrono::DateTime::from_timestamp_millis(event.start)
        .map(|at| {
            let offset_seconds = (local_offset_ms / 1000).clamp(-86_400, 86_400) as i32;
            let offset = chrono::FixedOffset::east_opt(offset_seconds)
                .unwrap_or(chrono::FixedOffset::east_opt(0).expect("UTC offset is valid"));
            at.with_timezone(&offset)
                .format("%Y-%m-%dT%H:%M:%S%:z")
                .to_string()
        })
        .unwrap_or_default();
    note.body = properties::set_property(&note.body, "event", &Value::Text(event.uid.clone()));
    note.body = properties::set_property(&note.body, "when", &Value::Text(when));
    store.put(
        DocType::Note,
        serde_json::to_value(note).map_err(|error| {
            kanso_store::StoreError::Invalid(format!("Couldn't encode that note: {error}"))
        })?,
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(())
}

/// Resolves each event to its own note, a series note, or the previous
/// instance's note, in that order.
pub fn links(
    store: &Store,
    events: &[Event],
    local_offset_ms: i64,
) -> Result<HashMap<String, MeetingLink>> {
    let notes = store.meeting_notes()?;
    let mut result = HashMap::new();
    for event in events {
        let instance = notes
            .iter()
            .find(|note| note.uid == event.uid && note.instance_start == Some(event.start));
        let series = notes
            .iter()
            .find(|note| note.uid == event.uid && note.instance_start.is_none());
        let previous = notes.iter().find(|note| {
            note.uid == event.uid && note.instance_start.is_some_and(|start| start < event.start)
        });
        let (note, which) = match (instance, series, previous) {
            (Some(note), _, _) => (note, "this"),
            (None, Some(note), _) => (note, "series"),
            (None, None, Some(note)) => (note, "last"),
            (None, None, None) => continue,
        };
        let day = if which == "last" {
            notes
                .iter()
                .find(|candidate| candidate.note_id == note.note_id)
                .and_then(|candidate| candidate.instance_start)
                .and_then(|start| {
                    chrono::DateTime::from_timestamp_millis(start.saturating_add(local_offset_ms))
                })
                .map(|at| at.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        result.insert(
            format!("{}@{}", event.uid, event.start),
            MeetingLink {
                note_id: note.note_id.clone(),
                which: which.to_owned(),
                day,
                locked: false,
            },
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::properties;
    use kanso_store::{EntityRow, Store};
    use proptest::prelude::*;

    #[test]
    fn a_new_meeting_note_has_instance_properties_in_front_matter() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.sqlite")).unwrap();
        let event = Event {
            uid: "meeting-1@example.test".into(),
            start: 1_793_943_000_000,
            end: 1_793_944_800_000,
            title: "Design review".into(),
            ..Event::default()
        };
        let id = create_note(&store, &event, 7_200_000, "09:30–10:00 · Rudo Moyo").unwrap();
        let note = store.note(&id).unwrap().unwrap();
        let props = properties::parse(&note.body);
        assert_eq!(
            properties::get(&props, "event").map(Value::text).as_deref(),
            Some("meeting-1@example.test")
        );
        assert_eq!(
            properties::get(&props, "when").map(Value::text).as_deref(),
            Some("2026-11-06T07:30:00+02:00")
        );
        assert_eq!(note.title, "Design review");
        assert!(note.body.contains("# Notes"));
        assert!(note.body.contains("Rudo Moyo"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            cases: 256,
            ..ProptestConfig::default()
        })]

        #[test]
        fn attendee_names_resolve_only_to_one_person_page(
            rows in prop::collection::vec(("[A-Z][a-z]{2,10}", any::<bool>()), 0..20)
        ) {
            let entities: Vec<EntityRow> = rows.into_iter().enumerate().map(|(index, (title, person))| {
                EntityRow {
                    id: format!("note:{index}"),
                    title,
                    kind: if person { EntityKind::Person } else { EntityKind::Project },
                    names: Vec::new(),
                    book_id: String::new(),
                    updated_at: 0,
                }
            }).collect();
            let people: Vec<&EntityRow> = entities
                .iter()
                .filter(|row| row.kind == EntityKind::Person)
                .collect();
            let found = unique_person(entities.iter());
            if people.len() == 1 {
                prop_assert_eq!(found, Some(people[0].title.as_str()));
            } else {
                prop_assert_eq!(found, None);
            }
        }
    }

    #[test]
    fn a_meeting_note_links_an_unambiguous_person_attendee() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.sqlite")).unwrap();
        let body = properties::set_property("", "kanso", &Value::Text("person".into()));
        let body = properties::set_property(
            &body,
            "aliases",
            &Value::List(vec![Value::Text("Rudo".into())]),
        );
        let book_id = store.role_book(Role::Journal, Origin::Ui).unwrap();
        store
            .put(
                DocType::Note,
                serde_json::json!({"title":"Rudo Chikore", "body":body, "bookId":book_id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let event = Event {
            uid: "meeting-2@example.test".into(),
            start: 1_799_043_000_000,
            end: 1_799_044_800_000,
            title: "Design review".into(),
            attendees: vec![super::super::model::Attendee {
                name: "Rudo".into(),
                email: "rudo@example.test".into(),
                page: None,
            }],
            ..Event::default()
        };
        let id = create_note(&store, &event, 0, "10:00–10:30 · Rudo").unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert!(note.body.contains("- [[Rudo Chikore]]\n"));
    }

    #[test]
    fn a_meeting_note_resolves_an_attendee_by_email() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.sqlite")).unwrap();
        let body = properties::set_property("", "kanso", &Value::Text("person".into()));
        let body =
            properties::set_property(&body, "email", &Value::Text("rudo@example.test".into()));
        let book_id = store.role_book(Role::Journal, Origin::Ui).unwrap();
        store
            .put(
                DocType::Note,
                serde_json::json!({"title":"Rudo Chikore", "body":body, "bookId":book_id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let event = Event {
            uid: "meeting-email@example.test".into(),
            start: 1_799_043_000_000,
            end: 1_799_044_800_000,
            title: "Design review".into(),
            attendees: vec![super::super::model::Attendee {
                name: "Unknown attendee".into(),
                email: "RUDO@example.test".into(),
                page: None,
            }],
            ..Event::default()
        };
        let id = create_note(&store, &event, 0, "10:00–10:30").unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert!(note.body.contains("- [[Rudo Chikore]]\n"));
    }

    #[test]
    fn linking_a_note_adds_the_event_properties_without_replacing_its_content() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.sqlite")).unwrap();
        let book_id = store.role_book(Role::Journal, Origin::Ui).unwrap();
        let created = store
            .put(
                DocType::Note,
                serde_json::json!({"title":"Existing notes", "body":"# Notes\n\nKeep this text.\n", "bookId":book_id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let event = Event {
            uid: "series-3@example.test".into(),
            start: 1_799_043_000_000,
            ..Event::default()
        };
        link_note(&store, &created.id, &event, 0).unwrap();
        let note = store.note(&created.id).unwrap().unwrap();
        let props = properties::parse(&note.body);
        assert_eq!(note.title, "Existing notes");
        assert!(note.body.contains("Keep this text."));
        assert_eq!(
            properties::get(&props, "event").map(Value::text).as_deref(),
            Some("series-3@example.test")
        );
        assert_eq!(store.meeting_notes().unwrap()[0].note_id, created.id);
    }
}
