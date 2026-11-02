//! What the person allows a plugin, which is never more than its manifest
//! asks for. The install sheet starts from [`initial`]; Settings narrows a
//! grant one permission at a time; nothing a plugin does can widen it.

use std::collections::{BTreeSet, HashSet};

use crate::model::{Grant, Manifest, NotesGrant, Surface};

/// Why a grant can't stand.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GrantError {
    #[error("it grants {0}, which the plugin doesn't ask for")]
    NotAsked(String),
    #[error("a locked notebook can't be given to a plugin")]
    LockedBook,
    #[error("a plugin can only change notes in notebooks it may read")]
    WriteWithoutRead,
}

/// What the install sheet offers before the person changes anything: what
/// the manifest asks for, except the notebooks and tags, which the person
/// picks, and every notebook, which is a switch of its own and starts off.
/// Asking before changing notes starts on. Pure.
pub fn initial(manifest: &Manifest) -> Grant {
    let p = &manifest.permissions;
    Grant {
        commands: p.commands,
        editor: p.editor.clone(),
        notes: NotesGrant {
            create: p.notes.create,
            ..NotesGrant::default()
        },
        fences: p.fences.iter().map(|f| f.language.clone()).collect(),
        panel: p.ui.contains(&Surface::Panel),
        secrets: p.secrets.clone(),
        network: p.network.clone(),
        ask_before_changing: true,
    }
}

/// Holds `grant` to `manifest`: everything granted is asked for, no locked
/// notebook is given, and every notebook it may change it may also read.
/// Pure.
pub fn check(
    grant: &Grant,
    manifest: &Manifest,
    locked: &HashSet<String>,
) -> Result<(), GrantError> {
    let p = &manifest.permissions;
    let not_asked = |what: &str| Err(GrantError::NotAsked(what.to_owned()));
    if grant.commands && !p.commands {
        return not_asked("commands");
    }
    if let Some(extra) = grant.editor.difference(&p.editor).next() {
        return not_asked(&format!("{extra:?}"));
    }
    let notes = &grant.notes;
    match &p.notes.read {
        None if notes.reads() => return not_asked("reading notes"),
        Some(read) if notes.read_all && !read.all => return not_asked("every notebook"),
        _ => {}
    }
    if notes.writes() && p.notes.write.is_empty() {
        return not_asked("changing notes");
    }
    if notes.create && !p.notes.create {
        return not_asked("making notes");
    }
    let asked_fences: BTreeSet<&str> = p.fences.iter().map(|f| f.language.as_str()).collect();
    if let Some(extra) = grant
        .fences
        .iter()
        .find(|f| !asked_fences.contains(f.as_str()))
    {
        return not_asked(&format!("{extra} blocks"));
    }
    if grant.panel && !p.ui.contains(&Surface::Panel) {
        return not_asked("a panel");
    }
    if let Some(extra) = grant.secrets.difference(&p.secrets).next() {
        return not_asked(&format!("the token {extra}"));
    }
    if let Some(extra) = grant.network.difference(&p.network).next() {
        return not_asked(extra);
    }
    if notes
        .read_books
        .iter()
        .chain(&notes.write_books)
        .any(|book| locked.contains(book))
    {
        return Err(GrantError::LockedBook);
    }
    if !notes.read_all
        && notes
            .write_books
            .iter()
            .any(|book| !notes.read_books.contains(book))
    {
        return Err(GrantError::WriteWithoutRead);
    }
    Ok(())
}

/// `grant` cut down to what `manifest` asks for: after an update that asks
/// for less, or when a manifest changed under developer mode. Never adds.
/// Pure.
pub fn narrow_to(grant: &Grant, manifest: &Manifest) -> Grant {
    let p = &manifest.permissions;
    let asked_fences: BTreeSet<String> = p.fences.iter().map(|f| f.language.clone()).collect();
    let reads = p.notes.read.as_ref();
    let mut notes = grant.notes.clone();
    if reads.is_none() {
        notes.read_books.clear();
        notes.read_tags.clear();
        notes.read_all = false;
    }
    if !reads.is_some_and(|r| r.all) {
        notes.read_all = false;
    }
    if p.notes.write.is_empty() || !notes.reads() {
        notes.write_books.clear();
    }
    if !notes.read_all {
        let readable = notes.read_books.clone();
        notes.write_books.retain(|book| readable.contains(book));
    }
    notes.create = notes.create && p.notes.create;
    Grant {
        commands: grant.commands && p.commands,
        editor: grant.editor.intersection(&p.editor).copied().collect(),
        notes,
        fences: grant.fences.intersection(&asked_fences).cloned().collect(),
        panel: grant.panel && p.ui.contains(&Surface::Panel),
        secrets: grant.secrets.intersection(&p.secrets).cloned().collect(),
        network: grant.network.intersection(&p.network).cloned().collect(),
        ask_before_changing: grant.ask_before_changing,
    }
}

/// Whether `narrower` allows nothing `wider` doesn't. Pure.
pub fn is_within(narrower: &Grant, wider: &Grant) -> bool {
    let (n, w) = (&narrower.notes, &wider.notes);
    (!narrower.commands || wider.commands)
        && narrower.editor.is_subset(&wider.editor)
        && (!n.read_all || w.read_all)
        && (w.read_all || n.read_books.is_subset(&w.read_books))
        && n.read_tags.is_subset(&w.read_tags)
        && n.write_books.is_subset(&w.write_books)
        && (!n.create || w.create)
        && narrower.fences.is_subset(&wider.fences)
        && (!narrower.panel || wider.panel)
        && narrower.secrets.is_subset(&wider.secrets)
        && narrower.network.is_subset(&wider.network)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        EditorPermission, Fence, NotesRequest, Permissions, Publisher, ReadRequest,
    };
    use proptest::prelude::*;

    fn manifest(permissions: Permissions) -> Manifest {
        Manifest {
            schema: 1,
            api_version: 1,
            id: "com.example.wordcount".into(),
            name: "Word count".into(),
            version: "1.0.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "Sam".into(),
                key: format!("ed25519:{}", "ab".repeat(32)),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions,
        }
    }

    fn permissions() -> impl Strategy<Value = Permissions> {
        (
            any::<bool>(),
            prop::collection::btree_set(
                prop::sample::select(vec![
                    EditorPermission::NoteRead,
                    EditorPermission::SelectionRead,
                    EditorPermission::SelectionReplace,
                ]),
                0..3,
            ),
            prop::option::of(any::<bool>()),
            any::<bool>(),
            any::<bool>(),
            prop::collection::btree_set("[a-c]", 0..3),
            any::<bool>(),
            prop::collection::btree_set("[a-c]", 0..3),
            prop::collection::btree_set("[a-c]\\.example", 0..3),
        )
            .prop_map(
                |(commands, editor, read, write, create, fences, panel, secrets, network)| {
                    Permissions {
                        commands,
                        editor,
                        notes: NotesRequest {
                            read: read.map(|all| ReadRequest {
                                all,
                                ..ReadRequest::default()
                            }),
                            write: if write { vec!["book:x".into()] } else { vec![] },
                            create,
                        },
                        fences: fences
                            .into_iter()
                            .map(|language| Fence { language })
                            .collect(),
                        ui: if panel {
                            [Surface::Panel].into()
                        } else {
                            BTreeSet::new()
                        },
                        secrets,
                        network,
                    }
                },
            )
    }

    fn grant() -> impl Strategy<Value = Grant> {
        (
            any::<bool>(),
            prop::collection::btree_set(
                prop::sample::select(vec![
                    EditorPermission::NoteRead,
                    EditorPermission::SelectionRead,
                    EditorPermission::SelectionReplace,
                ]),
                0..3,
            ),
            prop::collection::btree_set("book:[a-d]", 0..3),
            prop::collection::btree_set("tag:[a-d]", 0..2),
            any::<bool>(),
            prop::collection::btree_set("book:[a-d]", 0..3),
            any::<bool>(),
            prop::collection::btree_set("[a-d]", 0..3),
            (
                any::<bool>(),
                prop::collection::btree_set("[a-d]", 0..3),
                prop::collection::btree_set("[a-d]\\.example", 0..3),
            ),
        )
            .prop_map(
                |(
                    commands,
                    editor,
                    read_books,
                    read_tags,
                    read_all,
                    write_books,
                    create,
                    fences,
                    (panel, secrets, network),
                )| Grant {
                    commands,
                    editor,
                    notes: NotesGrant {
                        read_books,
                        read_tags,
                        read_all,
                        write_books,
                        create,
                    },
                    fences,
                    panel,
                    secrets,
                    network,
                    ask_before_changing: true,
                },
            )
    }

    proptest! {
        /// Invariant: narrowing a grant to a manifest always gives one that
        /// passes the check and is within the grant it came from.
        #[test]
        fn prop_narrowed_grants_pass(p in permissions(), g in grant()) {
            let m = manifest(p);
            let narrowed = narrow_to(&g, &m);
            prop_assert_eq!(check(&narrowed, &m, &HashSet::new()), Ok(()));
            prop_assert!(is_within(&narrowed, &g));
            // And narrowing again changes nothing.
            prop_assert_eq!(narrow_to(&narrowed, &m), narrowed);
        }

        /// Invariant: the initial grant passes the check and gives no
        /// notebook, tag or "every notebook" until the person picks.
        #[test]
        fn prop_initial_grants_read_nothing(p in permissions()) {
            let m = manifest(p);
            let g = initial(&m);
            prop_assert_eq!(check(&g, &m, &HashSet::new()), Ok(()));
            prop_assert!(!g.notes.reads() && !g.notes.writes());
            prop_assert!(g.ask_before_changing);
        }

        /// Invariant: a grant that passes never names a locked notebook.
        #[test]
        fn prop_locked_books_never_pass(p in permissions(), g in grant(), locked in prop::collection::hash_set("book:[a-d]", 0..3)) {
            let m = manifest(p);
            if check(&g, &m, &locked).is_ok() {
                prop_assert!(g.notes.read_books.iter().chain(&g.notes.write_books).all(|b| !locked.contains(b)));
            }
        }
    }

    #[test]
    fn a_grant_beyond_the_manifest_is_refused() {
        let m = manifest(Permissions {
            commands: true,
            ..Permissions::default()
        });
        let mut g = initial(&m);
        g.network.insert("evil.example".into());
        assert!(matches!(
            check(&g, &m, &HashSet::new()),
            Err(GrantError::NotAsked(_))
        ));
        let mut g = initial(&m);
        g.notes.read_books.insert("book:a".into());
        assert!(check(&g, &m, &HashSet::new()).is_err());
    }

    #[test]
    fn writing_needs_reading() {
        let m = manifest(Permissions {
            notes: NotesRequest {
                read: Some(ReadRequest::default()),
                write: vec!["book:w".into()],
                create: false,
            },
            ..Permissions::default()
        });
        let mut g = initial(&m);
        g.notes.write_books.insert("book:w".into());
        assert_eq!(
            check(&g, &m, &HashSet::new()),
            Err(GrantError::WriteWithoutRead)
        );
        g.notes.read_books.insert("book:w".into());
        assert_eq!(check(&g, &m, &HashSet::new()), Ok(()));
    }
}
