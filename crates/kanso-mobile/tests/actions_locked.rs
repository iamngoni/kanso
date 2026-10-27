//! Every new action against locked notes and locked notebooks: what it
//! leaves out, what stays sealed, and what needs the lock open.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::model::LOCK_FIELD;
use kanso_mobile::{KansoError, Library, ListPlace, ListQuery, LockState, NewNote, NoteChange};
use kanso_store::Store;
use kanso_sync::lock::{LockKey, LockPassword};
use kanso_sync::{KdfParams, MemoryServer};
use proptest::prelude::*;
use serde_json::json;

/// The note as the store keeps it, read through a second handle.
fn stored(phone: &Phone, id: &str) -> kanso_core::Note {
    Store::open(phone.path())
        .unwrap()
        .note(id)
        .unwrap()
        .unwrap()
}

fn status(name: &str) -> NoteChange {
    NoteChange::SetStatus {
        status: name.to_owned(),
    }
}

fn ids_in(library: &Library, place: ListPlace) -> Vec<String> {
    library
        .list(ListQuery {
            place,
            ..ListQuery::default()
        })
        .unwrap()
        .rows
        .into_iter()
        .map(|row| row.id)
        .collect()
}

/// A library with a plain note and a locked one (the session is open).
fn plain_and_locked() -> (Phone, String, String) {
    let phone = lone_phone();
    let plain = note(&phone.library, "Plain", "plain text");
    let locked = note(&phone.library, "Secret", "SECRET-TEXT");
    phone
        .library
        .locks()
        .lock_note(&locked, LOCK_PASSWORD.into())
        .unwrap();
    (phone, plain, locked)
}

/// A library whose notebook "Vault" was locked before it was opened, with
/// two notes in it, and a notebook "Zed" after it by name.
fn vault() -> (Phone, String, String, [String; 2]) {
    let mut ids = [String::new(), String::new()];
    let (mut vault, mut zed) = (String::new(), String::new());
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        vault = book(store, "Vault");
        zed = book(store, "Zed");
        for (i, id) in ids.iter_mut().enumerate() {
            *id = raw_note(
                store,
                json!({"title": format!("Vaulted {i}"), "body": format!("VAULT-TEXT-{i}"), "bookId": vault}),
            );
        }
        let key =
            LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST).unwrap();
        assert_eq!(
            kanso_engine::lock::seal_notebook(store, &vault, &key),
            Ok(2)
        );
    });
    (phone, vault, zed, ids)
}

// no-prop-test: fixed notes, open and closed; the mask is a property below.
#[test]
fn status_and_tags_leave_a_locked_note_out() {
    let (phone, plain, secret) = plain_and_locked();
    let library = &phone.library;
    let tag = library.create_tag("work".into()).unwrap().id;
    let both = vec![plain.clone(), secret.clone()];
    for open in [true, false] {
        if !open {
            library.locks().lock().unwrap();
        }
        let status_to = if open { "active" } else { "dropped" };
        let done = library.change(both.clone(), status(status_to)).unwrap();
        assert_eq!(
            (done.count, done.message.as_str()),
            (
                1,
                &*format!("Set 1 note to {}.", if open { "Active" } else { "Dropped" })
            )
        );
        assert_eq!(stored(&phone, &plain).status.as_str(), status_to);
        assert_eq!(stored(&phone, &secret).status.as_str(), "none");
        let add = NoteChange::AddTag {
            tag_id: tag.clone(),
        };
        // The plain note takes the tag; the sealed one keeps none, open or
        // closed, because its tags are its content.
        assert_eq!(library.change(both.clone(), add).unwrap().count, 1);
        assert!(stored(&phone, &secret).tags.is_empty());
        let remove = NoteChange::RemoveTag {
            tag_id: tag.clone(),
        };
        assert_eq!(library.change(both.clone(), remove).unwrap().count, 1);
    }
    // Only a locked note: nothing to write, say or undo.
    let none = library.change(vec![secret], status("active")).unwrap();
    assert_eq!(
        (none.count, none.message.as_str(), none.undo),
        (0, "", None)
    );
}

// no-prop-test: the notes of a locked notebook, fixed.
#[test]
fn notes_in_a_locked_notebook_take_no_status_or_tags() {
    let (phone, _, _, ids) = vault();
    let library = &phone.library;
    let tag = library.create_tag("work".into()).unwrap().id;
    for open in [false, true] {
        if open {
            assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
        }
        assert_eq!(
            library
                .change(ids.to_vec(), status("active"))
                .unwrap()
                .count,
            0
        );
        assert_eq!(
            library
                .change(
                    ids.to_vec(),
                    NoteChange::AddTag {
                        tag_id: tag.clone()
                    }
                )
                .unwrap()
                .count,
            0
        );
    }
    assert_eq!(stored(&phone, &ids[0]).status.as_str(), "none");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// A status or tag change never writes a locked note, whichever notes
    /// are locked and whether the lock is open: its stored status and tags
    /// stay, and the count is the plain notes that changed.
    #[test]
    fn prop_status_and_tags_never_reach_a_locked_note(
        mask in proptest::collection::vec(any::<bool>(), 4),
        kind in 0usize..3,
        open in any::<bool>(),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let tag = library.create_tag("work".into()).unwrap().id;
        let ids: Vec<String> = (0..4).map(|i| note(library, &format!("n{i}"), "x")).collect();
        for (id, locked) in ids.iter().zip(&mask) {
            if *locked {
                library.locks().lock_note(id, LOCK_PASSWORD.into()).unwrap();
            }
        }
        if !open {
            library.locks().lock().unwrap();
        }
        let before: Vec<_> = ids.iter().map(|id| stored(&phone, id)).collect();
        let change = match kind {
            0 => status("completed"),
            1 => NoteChange::AddTag { tag_id: tag.clone() },
            _ => NoteChange::RemoveTag { tag_id: tag.clone() },
        };
        let written = library.change(ids.clone(), change).unwrap();
        let plain = mask.iter().filter(|locked| !**locked).count() as u32;
        // A removal has nothing to take off a note without the tag.
        prop_assert_eq!(written.count, if kind == 2 { 0 } else { plain });
        for ((id, locked), old) in ids.iter().zip(&mask).zip(&before) {
            let now = stored(&phone, id);
            if *locked {
                prop_assert_eq!((&now.status, &now.tags, &now.rev), (&old.status, &old.tags, &old.rev));
            }
        }
    }
}

// no-prop-test: a fixed locked note through trash and restore.
#[test]
fn a_restored_locked_note_stays_locked() {
    let (phone, plain, secret) = plain_and_locked();
    let library = &phone.library;
    let alpha = library.create_notebook("Aaa".into(), None).unwrap().id;
    library.locks().lock().unwrap();
    library
        .change(vec![plain.clone(), secret.clone()], NoteChange::Trash)
        .unwrap();
    let back = library
        .change(vec![plain.clone(), secret.clone()], NoteChange::Restore)
        .unwrap();
    assert_eq!(back.count, 2);
    let view = library.note(&secret).unwrap();
    assert_eq!(view.notebook_id, alpha);
    assert!(view.lock.is_closed() && view.body.is_none());
    assert!(stored(&phone, &secret).is_locked());
    assert_eq!(stored(&phone, &secret).body, "");
    // Undo puts it back in the trash, still sealed.
    assert_eq!(library.undo(back.undo.unwrap()), Ok(2));
    assert_eq!(library.note(&secret).unwrap().notebook_id, "trash");
    assert!(stored(&phone, &secret).is_locked());
    library
        .change(vec![secret.clone()], NoteChange::Restore)
        .unwrap();
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        library.note(&secret).unwrap().body.as_deref(),
        Some("SECRET-TEXT")
    );
}

// no-prop-test: restoring into a locked notebook, which only sealed notes
// may enter in a batch.
#[test]
fn restore_into_a_locked_notebook_takes_sealed_notes_only() {
    let (phone, vault, zed, ids) = vault();
    let library = &phone.library;
    // Vault sorts first, so it is where Restore goes.
    assert_eq!(library.notebooks().unwrap()[0].id, vault);
    let plain = library
        .create(NewNote {
            notebook_id: Some(zed.clone()),
            title: "Plain".into(),
            body: "p".into(),
            ..NewNote::default()
        })
        .unwrap()
        .id;
    library
        .change(vec![ids[0].clone(), plain.clone()], NoteChange::Trash)
        .unwrap();
    // An unsealed note must go in one at a time, with the lock open.
    assert_eq!(
        library
            .change(vec![ids[0].clone(), plain.clone()], NoteChange::Restore)
            .map(|done| done.count),
        Err(KansoError::Locked)
    );
    assert_eq!(library.note(&ids[0]).unwrap().notebook_id, "trash");
    assert_eq!(
        library
            .change(vec![ids[0].clone()], NoteChange::Restore)
            .unwrap()
            .count,
        1
    );
    assert_eq!(library.note(&ids[0]).unwrap().notebook_id, vault);
    assert!(stored(&phone, &ids[0]).is_locked());
}

// no-prop-test: locked notes, open and closed, through both deletions.
#[test]
fn locked_notes_can_be_deleted_for_good() {
    let (phone, plain, secret) = plain_and_locked();
    let library = &phone.library;
    let second = note(library, "Second secret", "S2");
    library
        .locks()
        .lock_note(&second, LOCK_PASSWORD.into())
        .unwrap();
    library.locks().lock().unwrap();
    library
        .change(
            vec![plain, secret.clone(), second.clone()],
            NoteChange::Trash,
        )
        .unwrap();
    // Not in the trash: refused, and nothing is deleted.
    assert!(matches!(
        library.change(
            vec![secret.clone(), "note:x".into()],
            NoteChange::DeleteForever
        ),
        Err(KansoError::NotFound)
    ));
    let gone = library
        .change(vec![secret.clone()], NoteChange::DeleteForever)
        .unwrap();
    assert_eq!((gone.count, gone.undo), (1, None));
    assert_eq!(library.note(&secret), Err(KansoError::NotFound));
    assert!(
        Store::open(phone.path())
            .unwrap()
            .note(&secret)
            .unwrap()
            .is_none()
    );
    assert_eq!(library.empty_trash(), Ok(2));
    assert_eq!(library.note(&second), Err(KansoError::NotFound));
    assert_eq!(library.overview().unwrap().trash, 0);
}

// no-prop-test: a locked note outside the trash is refused like any note.
#[test]
fn a_locked_note_outside_the_trash_is_not_deleted() {
    let (phone, _, secret) = plain_and_locked();
    let library = &phone.library;
    assert!(matches!(
        library.change(vec![secret.clone()], NoteChange::DeleteForever),
        Err(KansoError::Invalid { .. })
    ));
    assert!(library.note(&secret).is_ok());
}

// no-prop-test: a locked notebook's notes go through the trash, deleted.
#[test]
fn a_locked_notebooks_notes_can_be_deleted_for_good() {
    let (phone, _, _, ids) = vault();
    let library = &phone.library;
    library.change(ids.to_vec(), NoteChange::Trash).unwrap();
    assert_eq!(library.empty_trash(), Ok(2));
    assert_eq!(library.note(&ids[1]), Err(KansoError::NotFound));
}

// no-prop-test: fixed notes; copies of any text are in the library test.
#[test]
fn duplicating_a_locked_note_needs_the_lock_open_and_seals_the_copy() {
    let (phone, _, secret) = plain_and_locked();
    let library = &phone.library;
    let before = library.overview().unwrap().all;
    library.locks().lock().unwrap();
    assert_eq!(library.duplicate(&secret), Err(KansoError::Locked));
    assert_eq!(library.overview().unwrap().all, before, "no copy was left");
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    let copy = library.duplicate(&secret).unwrap();
    let view = library.note(&copy.id).unwrap();
    assert_eq!(view.title, "Secret (copy)");
    assert_eq!(view.lock, LockState::Open);
    assert_eq!(view.body.as_deref(), Some("SECRET-TEXT"));
    let (original, copied) = (stored(&phone, &secret), stored(&phone, &copy.id));
    assert!(copied.is_locked());
    assert_eq!(copied.body, "", "the stored copy holds no text");
    assert_ne!(
        copied.extra.get(LOCK_FIELD),
        original.extra.get(LOCK_FIELD),
        "sealed for its own ID"
    );
    assert_eq!(
        (copied.book_id.as_str(), &copied.tags),
        (original.book_id.as_str(), &original.tags)
    );
    library.locks().lock().unwrap();
    let closed = library.note(&copy.id).unwrap();
    assert!(closed.body.is_none() && closed.lock.is_closed());
    assert!(
        ids_in(library, ListPlace::All).contains(&copy.id),
        "its title lists, closed"
    );
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        library.note(&copy.id).unwrap().body.as_deref(),
        Some("SECRET-TEXT")
    );
}

// no-prop-test: a note in a locked notebook, fixed.
#[test]
fn duplicating_a_note_in_a_locked_notebook_stays_in_it_sealed() {
    let (phone, vault, _, ids) = vault();
    let library = &phone.library;
    assert_eq!(library.duplicate(&ids[0]), Err(KansoError::Locked));
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    let copy = library.duplicate(&ids[0]).unwrap();
    let view = library.note(&copy.id).unwrap();
    assert_eq!(
        (view.notebook_id.as_str(), view.title.as_str()),
        (vault.as_str(), "Vaulted 0 (copy)")
    );
    assert_eq!(view.body.as_deref(), Some("VAULT-TEXT-0"));
    let copied = stored(&phone, &copy.id);
    assert!(copied.is_locked());
    assert_eq!(copied.body, "");
    library.locks().lock().unwrap();
    assert!(library.note(&copy.id).unwrap().body.is_none());
}

// no-prop-test: the title is all a link needs; fixed notes.
#[test]
fn a_locked_notes_link_works_off_its_title() {
    let (phone, _, secret) = plain_and_locked();
    let library = &phone.library;
    for open in [true, false] {
        if !open {
            library.locks().lock().unwrap();
        }
        let link = library.note_link(&secret, false).unwrap();
        assert_eq!(
            (link.text.as_str(), link.message.as_str()),
            ("[[Secret]]", "Link copied.")
        );
        let by_id = library.note_link(&secret, true).unwrap();
        assert!(
            by_id.text.contains(secret.trim_start_matches("note:"))
                && !by_id.text.contains("SECRET-TEXT")
        );
    }
    drop(phone);
    // And a note in a locked notebook, closed.
    let (phone, _, _, ids) = vault();
    let link = phone.library.note_link(&ids[0], false).unwrap();
    assert_eq!(link.text, "[[Vaulted 0]]");
}

// no-prop-test: a note with a status and tags before and after it is locked.
#[test]
fn a_locked_note_is_never_in_a_status_or_tag_path_place() {
    let phone = lone_phone();
    let library = &phone.library;
    let nested = library.create_tag("work/acme".into()).unwrap().id;
    let a = note(library, "A", "a");
    let b = note(library, "B", "b");
    for id in [&a, &b] {
        library.change(vec![id.clone()], status("active")).unwrap();
        library
            .change(
                vec![id.clone()],
                NoteChange::AddTag {
                    tag_id: nested.clone(),
                },
            )
            .unwrap();
    }
    let places = || {
        [
            ListPlace::Status {
                status: "active".into(),
            },
            ListPlace::TagPath {
                path: "work".into(),
            },
            ListPlace::Tag { id: nested.clone() },
        ]
    };
    for place in places() {
        assert_eq!(ids_in(library, place).len(), 2);
    }
    library.locks().lock_note(&b, LOCK_PASSWORD.into()).unwrap();
    for open in [true, false] {
        if !open {
            library.locks().lock().unwrap();
        }
        for place in places() {
            assert_eq!(ids_in(library, place.clone()), vec![a.clone()], "{place:?}");
        }
    }
    // The vault's notes have no status or tags to list.
    let (vaulted, ..) = vault();
    for place in [
        ListPlace::Status {
            status: "none".into(),
        },
        ListPlace::TagPath {
            path: "work".into(),
        },
    ] {
        assert!(ids_in(&vaulted.library, place).is_empty());
    }
}

// no-prop-test: a fixed locked notebook, renamed and decorated.
#[test]
fn a_locked_notebook_stays_locked_when_renamed_and_recolored() {
    let (phone, vault, _, ids) = vault();
    let library = &phone.library;
    library.rename_notebook(&vault, "Safe".into()).unwrap();
    library
        .set_notebook_color(&vault, Some("#F00".into()))
        .unwrap();
    library
        .set_notebook_icon(&vault, Some("lock".into()))
        .unwrap();
    library.set_notebook_color(&vault, None).unwrap();
    let row = library
        .notebooks()
        .unwrap()
        .into_iter()
        .find(|row| row.id == vault)
        .unwrap();
    assert!(row.locked);
    assert_eq!(row.name, "Safe");
    assert_eq!(
        (row.icon.as_deref(), row.color.as_deref()),
        (Some("lock"), None)
    );
    let books = Store::open(phone.path()).unwrap().books().unwrap();
    assert!(books.iter().find(|b| b.id == vault).unwrap().is_locked());
    for id in &ids {
        assert!(library.note(id).unwrap().lock.is_closed());
    }
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        library.note(&ids[1]).unwrap().body.as_deref(),
        Some("VAULT-TEXT-1")
    );
}

// no-prop-test: deleting a locked notebook, fixed.
#[test]
fn deleting_a_locked_notebook_trashes_its_notes_still_sealed() {
    let (phone, vault, _, ids) = vault();
    let library = &phone.library;
    library.delete_notebook(&vault).unwrap();
    assert!(library.notebooks().unwrap().iter().all(|b| b.id != vault));
    for id in &ids {
        let view = library.note(id).unwrap();
        assert_eq!(view.notebook_id, "trash");
        assert!(view.lock.is_closed() && view.body.is_none());
        assert!(stored(&phone, id).is_locked());
    }
}
