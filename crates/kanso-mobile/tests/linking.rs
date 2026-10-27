//! Links and getting around: the outline, backlinks, unlinked mentions, Link
//! on a mention, Open Note and the command palette, and the rule that keeps
//! a locked note out of all of them until the lock session is open.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::links::NOTE_LINK_PREFIX;
use kanso_engine::lock::seal_notebook;
use kanso_mobile::{
    KansoError, LinkRef, NoteChange, PaletteContext, PaletteSection, PaletteTarget,
};
use kanso_store::Store;
use kanso_sync::lock::{LockKey, LockPassword};
use kanso_sync::{KdfParams, MemoryServer};
use proptest::prelude::*;
use serde_json::json;

fn ids(links: &[LinkRef]) -> Vec<String> {
    let mut ids: Vec<String> = links.iter().map(|l| l.id.clone()).collect();
    ids.sort();
    ids
}

/// The notes a list of sections names, by ID.
fn listed_notes(sections: &[PaletteSection]) -> Vec<String> {
    let mut ids: Vec<String> = sections
        .iter()
        .flat_map(|s| &s.entries)
        .filter_map(|e| match &e.target {
            PaletteTarget::Note { id } => Some(id.clone()),
            _ => None,
        })
        .collect();
    ids.sort();
    ids
}

/// The notes whose headings a list of sections names, by ID.
fn listed_headings(sections: &[PaletteSection]) -> Vec<String> {
    let mut ids: Vec<String> = sections
        .iter()
        .flat_map(|s| &s.entries)
        .filter_map(|e| match &e.target {
            PaletteTarget::Heading { note_id, .. } => Some(note_id.clone()),
            _ => None,
        })
        .collect();
    ids.sort();
    ids
}

fn sorted(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids
}

// no-prop-test: a fixed text and the exact outline it makes.
#[test]
fn the_outline_is_parsed_from_the_text_and_the_caret_finds_its_heading() {
    let phone = lone_phone();
    let linking = phone.library.linking();
    let outline = linking.outline("intro\n# One\ntext\n```\n# not a heading\n```\n## Two\n");
    let seen: Vec<(u8, &str, u32)> = outline
        .iter()
        .map(|h| (h.level, h.text.as_str(), h.line))
        .collect();
    assert_eq!(seen, [(1, "One", 1), (2, "Two", 6)]);
    assert_eq!(linking.current_heading(&outline, 0), None);
    assert_eq!(linking.current_heading(&outline, 1), Some(0));
    assert_eq!(linking.current_heading(&outline, 5), Some(0));
    assert_eq!(linking.current_heading(&outline, 6), Some(1));
    assert_eq!(linking.current_heading(&outline, 99), Some(1));
    assert!(linking.outline("").is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// The heading under the caret is on or above the caret's line, and no
    /// later heading is.
    #[test]
    fn prop_the_current_heading_is_the_last_one_at_or_above_the_caret(
        body in "(#{1,4} [a-z]{1,5}\n|[a-z ]{0,8}\n){0,14}",
        caret in 0u32..20,
    ) {
        let phone = lone_phone();
        let linking = phone.library.linking();
        let outline = linking.outline(&body);
        match linking.current_heading(&outline, caret) {
            Some(ix) => {
                let ix = ix as usize;
                prop_assert!(outline[ix].line <= caret);
                prop_assert!(outline[ix + 1..].iter().all(|h| h.line > caret));
            }
            None => prop_assert!(outline.iter().all(|h| h.line > caret)),
        }
    }
}

// no-prop-test: one fixed library, the lists they make and the order.
#[test]
fn backlinks_and_mentions_come_with_the_line_they_are_on() {
    let phone = lone_phone();
    let lib = &phone.library;
    let target = note(lib, "Hubword", "");
    let by_title = note(lib, "By title", "first\nsee [[Hubword]] now");
    let by_id = note(
        lib,
        "By id",
        &format!(
            "[the hub]({NOTE_LINK_PREFIX}{})",
            target.trim_start_matches("note:")
        ),
    );
    let mention = note(lib, "Mention", "nothing\nI wrote hubword in words.");
    let linking = lib.linking();
    let back = linking.backlinks(&target).unwrap();
    assert_eq!(ids(&back), sorted(vec![by_title.clone(), by_id.clone()]));
    let title_row = back.iter().find(|l| l.id == by_title).unwrap();
    // The context is the line as words, so a wiki-link shows the text it
    // links to and not its brackets: it is the store's own context line.
    assert_eq!(
        (title_row.line, title_row.context.as_str()),
        (1, "see Hubword now")
    );
    let mentions = linking.unlinked_mentions(&target).unwrap();
    assert_eq!(ids(&mentions), vec![mention.clone()]);
    assert_eq!(mentions[0].line, 1);
    assert!(mentions[0].context.contains("hubword"));
    // A mention that links is not unlinked.
    assert!(!ids(&mentions).contains(&by_title));
}

// no-prop-test: Link on a fixed mention, in the three states it can be in.
#[test]
fn linking_a_mention_is_one_write_and_a_second_does_nothing() {
    let phone = lone_phone();
    let lib = &phone.library;
    let target = note(lib, "Hubword", "");
    let mention = note(lib, "Mention", "keep\nI wrote Hubword here.\nkeep too");
    let linking = lib.linking();
    let before = lib.note(&mention).unwrap();
    let saved = linking.link_mention(&target, &mention, 1).unwrap().unwrap();
    assert_eq!(saved.id, mention);
    assert_ne!(saved.rev, before.rev);
    let after = lib.note(&mention).unwrap();
    assert_eq!(
        after.body.as_deref(),
        Some(
            format!(
                "keep\nI wrote [Hubword]({NOTE_LINK_PREFIX}{}) here.\nkeep too",
                target.trim_start_matches("note:")
            )
            .as_str()
        )
    );
    // It now links, so it moves from the mentions to the backlinks.
    assert!(linking.unlinked_mentions(&target).unwrap().is_empty());
    assert_eq!(
        ids(&linking.backlinks(&target).unwrap()),
        vec![mention.clone()]
    );
    // Nothing left to link on that line, or on a line that isn't there.
    assert_eq!(linking.link_mention(&target, &mention, 1).unwrap(), None);
    assert_eq!(linking.link_mention(&target, &mention, 40).unwrap(), None);
    assert_eq!(lib.note(&mention).unwrap().rev, after.rev);
}

// no-prop-test: a locked mention in its three states; the property over
// many libraries is below.
#[test]
fn a_locked_note_is_linked_only_once_it_is_open_and_stays_sealed() {
    let phone = lone_phone();
    let lib = &phone.library;
    let target = note(lib, "Hubword", "");
    let mention = note(lib, "Mention", "I wrote hubword here.");
    let locks = lib.locks();
    locks.lock_note(&mention, LOCK_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    let linking = lib.linking();
    assert_eq!(
        linking.link_mention(&target, &mention, 0),
        Err(KansoError::Locked)
    );
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert!(
        linking
            .link_mention(&target, &mention, 0)
            .unwrap()
            .is_some()
    );
    assert!(
        lib.note(&mention)
            .unwrap()
            .body
            .unwrap()
            .contains(NOTE_LINK_PREFIX)
    );
    let store = Store::open(phone.path()).unwrap();
    let stored = store.note(&mention).unwrap().unwrap();
    assert!(stored.is_locked());
    assert_eq!(stored.body, "");
}

// no-prop-test: Open Note on a fixed library: its sections, the trash and
// the open note.
#[test]
fn open_note_lists_recent_notes_then_matches_and_never_the_trash() {
    let phone = lone_phone();
    let lib = &phone.library;
    let old = note(lib, "Gardening", "# Compost\nsoil");
    let trashed = note(lib, "Gardening in the trash", "# Compost\n");
    let recent = note(lib, "Garden plan", "# Beds\n");
    lib.change(vec![trashed.clone()], NoteChange::Trash)
        .unwrap();
    let linking = lib.linking();

    let recents = linking.quick_open("", Some(recent.clone())).unwrap();
    assert_eq!(recents.len(), 1);
    assert_eq!(recents[0].label, "Recently edited");
    assert_eq!(
        listed_notes(&recents),
        sorted(vec![old.clone(), recent.clone()])
    );
    let checked: Vec<&str> = recents[0]
        .entries
        .iter()
        .filter(|e| e.checked)
        .map(|e| e.label.as_str())
        .collect();
    assert_eq!(checked, ["Garden plan"]);

    let found = linking.quick_open("garden", None).unwrap();
    assert_eq!(
        listed_notes(&found),
        sorted(vec![old.clone(), recent.clone()])
    );
    let found = linking.quick_open("compost", None).unwrap();
    assert_eq!(listed_headings(&found), vec![old.clone()]);
    assert!(listed_notes(&found).contains(&old));
    assert!(!listed_notes(&found).contains(&trashed));
}

// no-prop-test: the palette on a fixed context: handlers gate commands,
// and a command's needs come from the library's own state.
#[test]
fn the_palette_lists_what_has_a_handler_and_what_the_open_note_allows() {
    let phone = lone_phone();
    let lib = &phone.library;
    let open = note(lib, "Open", "text");
    let linking = lib.linking();
    let context = |handlers: &[&str], note: Option<String>| PaletteContext {
        open_note_id: note,
        handlers: handlers.iter().map(|h| (*h).to_owned()).collect(),
        ..PaletteContext::default()
    };
    let commands = |sections: &[PaletteSection]| -> Vec<String> {
        sections
            .iter()
            .flat_map(|s| &s.entries)
            .filter_map(|e| match &e.target {
                PaletteTarget::Command { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect()
    };
    let none = linking.palette("", context(&[], None)).unwrap();
    assert!(commands(&none).is_empty());
    // Places are listed whatever has a handler.
    assert!(none.iter().any(|s| s.label == "Go to"));
    let some = linking
        .palette("", context(&["new-note", "pin", "trash", "sync"], None))
        .unwrap();
    assert_eq!(commands(&some), ["new-note"]);
    let with_note = linking
        .palette(
            "",
            context(&["new-note", "pin", "trash", "sync"], Some(open.clone())),
        )
        .unwrap();
    assert_eq!(commands(&with_note), ["new-note", "pin", "trash"]);
    lib.change(vec![open.clone()], NoteChange::TogglePin)
        .unwrap();
    let pinned = linking
        .palette("unpin", context(&["pin"], Some(open.clone())))
        .unwrap();
    let pin = pinned
        .iter()
        .flat_map(|s| &s.entries)
        .find(|e| matches!(&e.target, PaletteTarget::Command { id, .. } if id == "pin"))
        .unwrap();
    assert_eq!(
        (pin.label.as_str(), pin.icon.as_str()),
        ("Unpin note", "pin-off")
    );
    // A note in the trash can't be pinned.
    lib.change(vec![open.clone()], NoteChange::Trash).unwrap();
    let trashed = linking.palette("", context(&["pin"], Some(open))).unwrap();
    assert!(commands(&trashed).is_empty());
}

/// How a note is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kept {
    Plain,
    /// Sealed by locking the note itself.
    Sealed,
    /// In a locked notebook.
    Vault,
}

fn kept() -> impl Strategy<Value = Kept> {
    prop_oneof![Just(Kept::Plain), Just(Kept::Sealed), Just(Kept::Vault)]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// For any mix of plain, locked and locked-notebook notes that link to,
    /// or mention, a target: while the session is closed none of the locked
    /// ones is in the backlinks, the mentions, Open Note (by title or by
    /// heading) or the palette; once it is open all of them are.
    #[test]
    fn prop_a_locked_note_is_out_until_the_session_is_open(
        linkers in proptest::collection::vec(kept(), 1..4),
        mentioners in proptest::collection::vec(kept(), 1..4),
    ) {
        let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST).unwrap();
        let mut target = String::new();
        let mut linker_ids = Vec::new();
        let mut mentioner_ids = Vec::new();
        let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
            let vault = book(store, "Vault");
            let make = |title: String, body: String, how: Kept| {
                let mut fields = json!({"title": title, "body": body});
                if how == Kept::Vault {
                    fields["bookId"] = json!(vault);
                }
                raw_note(store, fields)
            };
            target = make("Hubword".into(), String::new(), Kept::Plain);
            for (i, how) in linkers.iter().enumerate() {
                linker_ids.push(make(
                    format!("Linker {i}"),
                    format!("# Zeta linker {i}\nsee [[Hubword]] here"),
                    *how,
                ));
            }
            for (i, how) in mentioners.iter().enumerate() {
                mentioner_ids.push(make(
                    format!("Mentioner {i}"),
                    format!("# Zeta mentioner {i}\nsays hubword in words"),
                    *how,
                ));
            }
            seal_notebook(store, &vault, &key).unwrap();
        });
        let lib = &phone.library;
        let locks = lib.locks();
        let all_linkers = sorted(linker_ids.clone());
        let all_mentioners = sorted(mentioner_ids.clone());
        let mut open_linkers = Vec::new();
        let mut open_mentioners = Vec::new();
        for (ids, kinds, open) in [
            (&linker_ids, &linkers, &mut open_linkers),
            (&mentioner_ids, &mentioners, &mut open_mentioners),
        ] {
            for (id, how) in ids.iter().zip(kinds) {
                match how {
                    Kept::Sealed => { locks.lock_note(id, LOCK_PASSWORD.into()).unwrap(); }
                    Kept::Plain => open.push(id.clone()),
                    Kept::Vault => {}
                }
            }
        }
        let (open_linkers, open_mentioners) = (sorted(open_linkers), sorted(open_mentioners));
        locks.lock().unwrap();
        prop_assert!(!locks.is_unlocked().unwrap());
        let linking = lib.linking();
        let context = PaletteContext { handlers: vec!["new-note".into()], ..PaletteContext::default() };

        // Closed: only the plain ones.
        prop_assert_eq!(ids(&linking.backlinks(&target).unwrap()), open_linkers.clone());
        prop_assert_eq!(ids(&linking.unlinked_mentions(&target).unwrap()), open_mentioners.clone());
        let by_title = linking.quick_open("linker", None).unwrap();
        prop_assert_eq!(listed_notes(&by_title), open_linkers.clone());
        let by_heading = linking.quick_open("zeta linker", None).unwrap();
        prop_assert_eq!(listed_headings(&by_heading), open_linkers.clone());
        let recent = linking.quick_open("", None).unwrap();
        let mut plain: Vec<String> = open_linkers.iter().chain(&open_mentioners).cloned().collect();
        plain.push(target.clone());
        prop_assert_eq!(listed_notes(&recent), sorted(plain));
        let palette = linking.palette("mentioner", context.clone()).unwrap();
        prop_assert_eq!(listed_notes(&palette), open_mentioners.clone());
        // Neither the title nor the words show anywhere in the text of
        // what came back.
        let everything = format!(
            "{:?}{:?}{:?}{:?}",
            linking.backlinks(&target).unwrap(),
            linking.unlinked_mentions(&target).unwrap(),
            by_heading,
            palette
        );
        for id in all_linkers.iter().filter(|id| !open_linkers.contains(id)) {
            prop_assert!(!everything.contains(id.as_str()));
        }

        // Open: all of them.
        prop_assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
        let back = linking.backlinks(&target).unwrap();
        prop_assert_eq!(ids(&back), all_linkers.clone());
        for link in &back {
            prop_assert_eq!(link.line, 1);
            // Plain words, as the store's own backlink context is.
            prop_assert_eq!(link.context.as_str(), "see Hubword here");
        }
        let mentions = linking.unlinked_mentions(&target).unwrap();
        prop_assert_eq!(ids(&mentions), all_mentioners.clone());
        for mention in &mentions {
            prop_assert_eq!(mention.line, 1);
            prop_assert!(mention.context.contains("hubword"));
        }
        prop_assert_eq!(listed_notes(&linking.quick_open("linker", None).unwrap()), all_linkers.clone());
        prop_assert_eq!(
            listed_headings(&linking.quick_open("zeta linker", None).unwrap()),
            all_linkers.clone()
        );
        prop_assert_eq!(
            listed_notes(&linking.palette("mentioner", context).unwrap()),
            all_mentioners.clone()
        );

        // Closed again: out again.
        locks.lock().unwrap();
        prop_assert_eq!(ids(&linking.backlinks(&target).unwrap()), open_linkers);
    }

    /// A locked note's own backlinks and mentions are empty while it is
    /// closed, whoever links to it.
    #[test]
    fn prop_the_links_of_a_closed_locked_note_are_not_listed(how in prop_oneof![Just(Kept::Sealed), Just(Kept::Vault)]) {
        let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST).unwrap();
        let mut target = String::new();
        let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
            let vault = book(store, "Vault");
            let mut fields = json!({"title": "Secret target", "body": ""});
            if how == Kept::Vault {
                fields["bookId"] = json!(vault);
            }
            target = raw_note(store, fields);
            raw_note(store, json!({"title": "Open linker", "body": "see [[Secret target]]\nSecret target again"}));
            seal_notebook(store, &vault, &key).unwrap();
        });
        let lib = &phone.library;
        let locks = lib.locks();
        if how == Kept::Sealed {
            locks.lock_note(&target, LOCK_PASSWORD.into()).unwrap();
        }
        locks.lock().unwrap();
        let linking = lib.linking();
        prop_assert!(linking.backlinks(&target).unwrap().is_empty());
        prop_assert!(linking.unlinked_mentions(&target).unwrap().is_empty());
        prop_assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
        prop_assert_eq!(linking.backlinks(&target).unwrap().len(), 1);
    }
}
