//! What the install and update sheets say, generated from the manifest and
//! the grant, never written by the author.

use crate::model::{ClipboardPermission, EditorPermission, Grant, Manifest, Permissions, Surface};

/// Each thing `permissions` asks for, one plain line apiece, in a fixed
/// order: what the directory lists and what an update's diff compares.
/// Pure.
pub fn permission_lines(permissions: &Permissions) -> Vec<String> {
    let mut lines = Vec::new();
    if permissions.commands {
        lines.push("add commands to the palette".to_owned());
    }
    for editor in &permissions.editor {
        lines.push(
            match editor {
                EditorPermission::NoteRead => "read the note you have open",
                EditorPermission::SelectionRead => "read the text you select",
                EditorPermission::SelectionReplace => "change the text you select",
            }
            .to_owned(),
        );
    }
    if let Some(read) = &permissions.notes.read {
        lines.push(if read.all {
            "read every notebook that isn't locked".to_owned()
        } else {
            "read notes in the notebooks you choose".to_owned()
        });
    }
    if !permissions.notes.write.is_empty() {
        lines.push("change notes in the notebooks you choose".to_owned());
    }
    if permissions.notes.create {
        lines.push("make new notes".to_owned());
    }
    for fence in &permissions.fences {
        lines.push(format!("draw {} blocks in your notes", fence.language));
    }
    for surface in &permissions.ui {
        match surface {
            Surface::Panel => lines.push("show a panel in the sidebar".to_owned()),
        }
    }
    for secret in &permissions.secrets {
        lines.push(format!("use the token you save for it as {secret}"));
    }
    for clipboard in &permissions.clipboard {
        lines.push(clipboard_line(*clipboard).to_owned());
    }
    for host in &permissions.network {
        lines.push(format!("connect to {host}"));
    }
    lines
}

/// What a clipboard permission lets a plugin do, in the sheet's words.
/// Pure.
fn clipboard_line(permission: ClipboardPermission) -> &'static str {
    match permission {
        ClipboardPermission::HistoryRead => {
            "read the text you copy in any app while Kanso is open (Kanso keeps the last 15 in memory, leaving out passwords and what looks like a key)"
        }
        ClipboardPermission::Write => "put text on the clipboard",
    }
}

/// How a plugin's code is vouched for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signed {
    /// The directory signed it, and the author did too.
    Directory,
    /// An unpacked folder in developer mode.
    Developer,
}

/// The install sheet's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sheet {
    pub title: String,
    /// Publisher, ID and version on one line.
    pub byline: String,
    pub signed_by: String,
    /// What the grant lets it do.
    pub can: Vec<String>,
    /// What it can never do, whatever it asks.
    pub cannot: Vec<String>,
    /// Set when what it reads can leave this Mac.
    pub warning: Option<String>,
}

/// "A", "A and B", "A, B and C". Pure.
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// The install sheet for `manifest` as `grant` allows it. `name_of` turns a
/// notebook or tag ID into its name; an ID it doesn't know is shown as is.
/// What it can do comes from the grant, never the manifest. Pure.
pub fn sheet(
    manifest: &Manifest,
    grant: &Grant,
    signed: Signed,
    name_of: &dyn Fn(&str) -> Option<String>,
) -> Sheet {
    let name = |id: &String| name_of(id).unwrap_or_else(|| id.clone());
    let notes = &grant.notes;
    let books: Vec<String> = notes.read_books.iter().map(name).collect();
    let tags: Vec<String> = notes
        .read_tags
        .iter()
        .map(|t| format!("#{}", name(t)))
        .collect();
    let write_books: Vec<String> = notes.write_books.iter().map(name).collect();
    let hosts: Vec<String> = grant.network.iter().cloned().collect();

    let note_read = grant.editor.contains(&EditorPermission::NoteRead);
    let selection_read = grant.editor.contains(&EditorPermission::SelectionRead);

    let mut can = Vec::new();
    if grant.commands {
        can.push("add commands to the palette".to_owned());
    }
    match (note_read, selection_read) {
        (true, true) => can.push("read the note you have open, and the text you select".to_owned()),
        (true, false) => can.push("read the note you have open".to_owned()),
        (false, true) => can.push("read the text you select".to_owned()),
        (false, false) => {}
    }
    if grant.editor.contains(&EditorPermission::SelectionReplace) {
        can.push("change the text you select".to_owned());
    }
    if notes.read_all {
        can.push("read every notebook that isn't locked".to_owned());
    }
    if !books.is_empty() {
        can.push(format!("read notes in {}", join_and(&books)));
    }
    if !tags.is_empty() {
        can.push(format!("read notes tagged {}", join_and(&tags)));
    }
    if notes.writes() {
        can.push(format!("change notes in {}", join_and(&write_books)));
        if notes.create {
            can.push(format!("make new notes in {}", join_and(&write_books)));
        }
    }
    for language in &grant.fences {
        can.push(format!("draw {language} blocks in your notes"));
    }
    if grant.panel {
        can.push("show a panel in the sidebar".to_owned());
    }
    for secret in &grant.secrets {
        can.push(format!("use the token you save for it as {secret}"));
    }
    for clipboard in &grant.clipboard {
        can.push(clipboard_line(*clipboard).to_owned());
    }

    let mut warning = None;
    if !hosts.is_empty() {
        let reads = notes.reads() || note_read || selection_read;
        if reads {
            let mut sources = Vec::new();
            if notes.read_all {
                sources.push("every notebook".to_owned());
            }
            sources.extend(books.iter().cloned());
            if !tags.is_empty() {
                sources.push(format!("notes tagged {}", join_and(&tags)));
            }
            match (note_read, selection_read) {
                (true, true) => {
                    sources.push("the note you have open and the text you select".to_owned())
                }
                (true, false) => sources.push("the note you have open".to_owned()),
                (false, true) => sources.push("the text you select".to_owned()),
                (false, false) => {}
            }
            can.push(format!(
                "send what it reads from {} to {}",
                join_and(&sources),
                join_and(&hosts)
            ));
            warning = Some("Anything it reads can leave this Mac.".to_owned());
        } else {
            can.push(format!("connect to {}", join_and(&hosts)));
        }
    }

    let mut cannot = Vec::new();
    if !notes.read_all {
        cannot.push("read other notes or notebooks".to_owned());
    }
    cannot.push(
        "read locked notes, your lock password, your keys, your account or your sync code"
            .to_owned(),
    );
    if !grant.clipboard.contains(&ClipboardPermission::HistoryRead) {
        cannot.push("read what you copy in other apps".to_owned());
    }
    if hosts.is_empty() {
        cannot.push("use the network".to_owned());
    }
    cannot.push("read files on your computer".to_owned());
    cannot.push("delete notes, notebooks or tags".to_owned());

    let signed_by = match signed {
        Signed::Directory => format!(
            "signed by the author ({}) and kansoapp.app",
            fingerprint(&manifest.publisher.key)
        ),
        Signed::Developer => "not signed: developer mode".to_owned(),
    };
    Sheet {
        title: manifest.name.clone(),
        byline: format!(
            "{} · {} · {}",
            manifest.publisher.name, manifest.id, manifest.version
        ),
        signed_by,
        can,
        cannot,
        warning,
    }
}

/// The key's first four bytes as `9F2A 71C4 …`. Pure.
fn fingerprint(key: &str) -> String {
    let hex = key.strip_prefix("ed25519:").unwrap_or(key);
    let head: String = hex.chars().take(8).collect::<String>().to_uppercase();
    let (a, b) = head.split_at(head.len().min(4));
    format!("{a} {b} …")
}

/// What an update adds and what it drops, as sheet lines, each in the
/// order [`permission_lines`] gives. Pure.
pub fn diff(old: &Permissions, new: &Permissions) -> (Vec<String>, Vec<String>) {
    let (old, new) = (permission_lines(old), permission_lines(new));
    let added = new.iter().filter(|l| !old.contains(l)).cloned().collect();
    let removed = old.iter().filter(|l| !new.contains(l)).cloned().collect();
    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Fence, NotesGrant, NotesRequest, Publisher};
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn manifest() -> Manifest {
        Manifest {
            schema: 1,
            api_version: 1,
            id: "com.samrivera.wordcount".into(),
            name: "Word count".into(),
            version: "1.4.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "Sam Rivera".into(),
                key: "ed25519:9f2a71c4d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68"
                    .into(),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions: Permissions::default(),
        }
    }

    fn grant() -> Grant {
        Grant {
            commands: false,
            editor: BTreeSet::new(),
            notes: NotesGrant::default(),
            fences: BTreeSet::new(),
            panel: false,
            secrets: BTreeSet::new(),
            network: BTreeSet::new(),
            clipboard: BTreeSet::new(),
            ask_before_changing: true,
        }
    }

    fn names(id: &str) -> Option<String> {
        Some(
            match id {
                "w" => "Work",
                "c" => "Clients",
                "i" => "Inbox",
                "x" => "x",
                "y" => "y",
                _ => return None,
            }
            .to_owned(),
        )
    }

    fn set<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn the_word_count_sheet_matches_the_design() {
        let mut g = grant();
        g.commands = true;
        g.editor = BTreeSet::from([
            EditorPermission::NoteRead,
            EditorPermission::SelectionRead,
            EditorPermission::SelectionReplace,
        ]);
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        assert_eq!(s.title, "Word count");
        assert_eq!(s.byline, "Sam Rivera · com.samrivera.wordcount · 1.4.0");
        assert_eq!(
            s.signed_by,
            "signed by the author (9F2A 71C4 …) and kansoapp.app"
        );
        assert_eq!(
            s.can,
            [
                "add commands to the palette",
                "read the note you have open, and the text you select",
                "change the text you select",
            ]
        );
        assert_eq!(
            s.cannot,
            [
                "read other notes or notebooks",
                "read locked notes, your lock password, your keys, your account or your sync code",
                "read what you copy in other apps",
                "use the network",
                "read files on your computer",
                "delete notes, notebooks or tags",
            ]
        );
        assert_eq!(s.warning, None);
    }

    #[test]
    fn developer_mode_says_it_is_unsigned() {
        let s = sheet(&manifest(), &grant(), Signed::Developer, &names);
        assert_eq!(s.signed_by, "not signed: developer mode");
    }

    #[test]
    fn reading_and_sending_warns() {
        let mut g = grant();
        g.notes.read_books = set(["w", "c", "i"]);
        g.network = set(["api.clip.example"]);
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        assert!(
            s.can
                .contains(&"read notes in Clients, Inbox and Work".to_owned())
        );
        assert!(s.can.contains(
            &"send what it reads from Clients, Inbox and Work to api.clip.example".to_owned()
        ));
        assert!(!s.can.iter().any(|l| l.starts_with("connect to")));
        assert_eq!(
            s.warning.as_deref(),
            Some("Anything it reads can leave this Mac.")
        );
        assert!(!s.cannot.contains(&"use the network".to_owned()));
    }

    #[test]
    fn network_alone_connects_without_a_warning() {
        let mut g = grant();
        g.network = set(["a.example", "b.example"]);
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        assert!(
            s.can
                .contains(&"connect to a.example and b.example".to_owned())
        );
        assert_eq!(s.warning, None);
    }

    #[test]
    fn other_lines_read_plainly() {
        let mut g = grant();
        g.editor = BTreeSet::from([EditorPermission::NoteRead]);
        g.network = set(["a.example"]);
        g.notes.read_tags = set(["x", "y"]);
        g.notes.write_books = set(["w"]);
        g.notes.create = true;
        g.fences = set(["chart"]);
        g.panel = true;
        g.secrets = set(["TOKEN"]);
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        for want in [
            "read the note you have open",
            "read notes tagged #x and #y",
            "change notes in Work",
            "make new notes in Work",
            "draw chart blocks in your notes",
            "show a panel in the sidebar",
            "use the token you save for it as TOKEN",
            "send what it reads from notes tagged #x and #y and the note you have open to a.example",
        ] {
            assert!(s.can.contains(&want.to_owned()), "{want}: {:?}", s.can);
        }
        let mut g = grant();
        g.notes.read_all = true;
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        assert!(
            s.can
                .contains(&"read every notebook that isn't locked".to_owned())
        );
        assert!(
            !s.cannot
                .contains(&"read other notes or notebooks".to_owned())
        );
    }

    #[test]
    fn unknown_ids_show_as_themselves() {
        let mut g = grant();
        g.notes.read_books = set(["zzz"]);
        let s = sheet(&manifest(), &g, Signed::Directory, &names);
        assert!(s.can.contains(&"read notes in zzz".to_owned()));
    }

    #[test]
    fn diff_lists_additions_and_removals() {
        let a = Permissions {
            commands: true,
            network: set(["a.example"]),
            ..Default::default()
        };
        let b = Permissions {
            commands: true,
            notes: NotesRequest {
                create: true,
                ..Default::default()
            },
            fences: vec![Fence {
                language: "chart".into(),
            }],
            ..Default::default()
        };
        let (added, removed) = diff(&a, &b);
        assert_eq!(added, ["make new notes", "draw chart blocks in your notes"]);
        assert_eq!(removed, ["connect to a.example"]);
    }

    fn arb_grant() -> impl Strategy<Value = Grant> {
        let ids = proptest::collection::btree_set(
            prop_oneof![Just("w"), Just("c"), Just("i"), Just("q")].prop_map(String::from),
            0..4,
        );
        (
            any::<bool>(),
            proptest::collection::btree_set(
                prop_oneof![
                    Just(EditorPermission::NoteRead),
                    Just(EditorPermission::SelectionRead),
                    Just(EditorPermission::SelectionReplace)
                ],
                0..3,
            ),
            (ids.clone(), ids.clone(), any::<bool>(), ids, any::<bool>()),
            proptest::collection::btree_set("[0-9]{1,6}", 0..3),
            any::<bool>(),
            proptest::collection::btree_set("[A-Z]{1,6}", 0..3),
            proptest::collection::btree_set("[a-z]{1,6}\\.example", 0..3),
            proptest::collection::btree_set(
                proptest::sample::select(vec![
                    ClipboardPermission::HistoryRead,
                    ClipboardPermission::Write,
                ]),
                0..3,
            ),
        )
            .prop_map(
                |(
                    commands,
                    editor,
                    (rb, rt, ra, wb, create),
                    fences,
                    panel,
                    secrets,
                    network,
                    clipboard,
                )| {
                    Grant {
                        commands,
                        editor,
                        notes: NotesGrant {
                            read_books: rb,
                            read_tags: rt,
                            read_all: ra,
                            write_books: wb,
                            create,
                        },
                        fences,
                        panel,
                        secrets,
                        network,
                        clipboard,
                        ask_before_changing: true,
                    }
                },
            )
    }

    fn arb_permissions() -> impl Strategy<Value = Permissions> {
        (
            any::<bool>(),
            proptest::collection::btree_set("[a-z]{1,4}\\.example", 0..4),
            any::<bool>(),
        )
            .prop_map(|(commands, network, create)| Permissions {
                commands,
                network,
                notes: NotesRequest {
                    create,
                    ..Default::default()
                },
                ..Default::default()
            })
    }

    proptest! {
        /// Invariant: reading plus network always warns and never leaves a
        /// bare "connect to" line.
        #[test]
        fn prop_reads_with_network_warn(g in arb_grant()) {
            let s = sheet(&manifest(), &g, Signed::Directory, &names);
            let reads = g.notes.reads() || g.editor.contains(&EditorPermission::NoteRead) || g.editor.contains(&EditorPermission::SelectionRead);
            if reads && !g.network.is_empty() {
                prop_assert!(s.warning.is_some());
                prop_assert!(!s.can.iter().any(|l| l.starts_with("connect to")));
                prop_assert!(s.can.iter().any(|l| l.starts_with("send what it reads from")));
            }
            if g.network.is_empty() {
                prop_assert!(s.warning.is_none());
                prop_assert!(s.cannot.contains(&"use the network".to_owned()));
            }
        }

        /// Invariant: the cannot list always names locked notes and files.
        #[test]
        fn prop_cannot_always_has_the_locked_line(g in arb_grant()) {
            let s = sheet(&manifest(), &g, Signed::Developer, &names);
            prop_assert!(s.cannot.iter().any(|l| l.starts_with("read locked notes")));
            prop_assert!(s.cannot.contains(&"read files on your computer".to_owned()));
            prop_assert!(s.cannot.contains(&"delete notes, notebooks or tags".to_owned()));
        }

        /// Invariant: the can list says nothing the grant didn't give.
        #[test]
        fn prop_can_only_lists_what_is_granted(g in arb_grant()) {
            let s = sheet(&manifest(), &g, Signed::Directory, &names);
            let has = |needle: &str| s.can.iter().any(|l| l.contains(needle));
            if !g.commands { prop_assert!(!has("palette")); }
            if !g.editor.contains(&EditorPermission::SelectionReplace) { prop_assert!(!has("change the text")); }
            if !g.editor.contains(&EditorPermission::NoteRead) { prop_assert!(!has("note you have open")); }
            if !g.editor.contains(&EditorPermission::SelectionRead) { prop_assert!(!has("read the text you select") && !has("and the text you select")); }
            if !g.notes.read_all { prop_assert!(!has("every notebook")); }
            if g.notes.read_tags.is_empty() { prop_assert!(!has("tagged")); }
            if !g.notes.writes() { prop_assert!(!has("change notes") && !has("make new notes")); }
            if !g.notes.create { prop_assert!(!has("make new notes")); }
            if g.fences.is_empty() { prop_assert!(!has("draw")); }
            if !g.panel { prop_assert!(!has("panel")); }
            if g.secrets.is_empty() { prop_assert!(!has("token")); }
            // The clipboard only as granted, and its absence said out loud.
            let reads_clipboard = g.clipboard.contains(&ClipboardPermission::HistoryRead);
            prop_assert_eq!(has("text you copy"), reads_clipboard);
            prop_assert_eq!(
                s.cannot.contains(&"read what you copy in other apps".to_owned()),
                !reads_clipboard
            );
            prop_assert_eq!(
                has("put text on the clipboard"),
                g.clipboard.contains(&ClipboardPermission::Write)
            );
            if g.network.is_empty() { prop_assert!(!has("connect to") && !has("send what")); }
            if !g.notes.reads() && g.editor.is_empty() && !g.network.is_empty() {
                prop_assert!(!has("read notes"));
            }
        }

        /// Invariant: nothing is both added and removed, and no change is empty.
        #[test]
        fn prop_diff_is_disjoint_and_empty_for_equal(a in arb_permissions(), b in arb_permissions()) {
            let (added, removed) = diff(&a, &b);
            prop_assert!(added.iter().all(|l| !removed.contains(l)));
            let (added, removed) = diff(&a, &a);
            prop_assert!(added.is_empty() && removed.is_empty());
            let (back_added, back_removed) = diff(&b, &a);
            let (added, removed) = diff(&a, &b);
            prop_assert_eq!(added, back_removed);
            prop_assert_eq!(removed, back_added);
        }
    }
}
