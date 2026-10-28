//! The guarantee the whole facade is built to keep: while a note's lock is
//! not open, nothing the library returns holds its text.
//!
//! A library of notes, each with a marker in its text, its front matter, its
//! tasks and an inline tag, some locked. A random sequence of calls (lists, reads, search,
//! properties, query views, previews, saves, creating, trashing, locking and
//! unlocking, the auto-lock) runs against it. After every call, the marker of
//! every note that was closed when the call was made is looked for in the debug
//! text of what the call returned, and must not be found. The streams'
//! payloads, which only ever carry IDs and states, are checked at the end for
//! any marker at all. A model of which notes are closed (locked, and the
//! session not open) decides what "closed" means; it never asks the library.
//!
//! Locking by hand, by the auto-lock or by the app going to the background,
//! and opening and closing with biometrics, are among the calls: after any
//! lock no key survives, and every locked note reads closed through every
//! read path.
//!
//! Notes open for editing are in the sequence too: a session on a closed note
//! is refused, every call of a session on an open one is checked like the
//! rest (snapshots, runs, completions, links, properties), and when the lock
//! closes the sessions on locked notes are saved sealed first, then return
//! `Locked` and send `Closed` and nothing after it. What was typed in them
//! comes back when the notes are opened again; the store holds none of it in
//! the clear.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::*;
use kanso_engine::lock::AUTO_LOCK;
use kanso_mobile::biometrics;
use kanso_mobile::{
    EditorSession, KansoError, ListPlace, ListQuery, LockEvent, NoteChange, NoteRow, PaletteDto,
    Range16, Renderer, SearchScope, SortKey, TextChange,
};
use proptest::prelude::*;

const NOTES: usize = 4;

/// The text note `i` is made of, every part carrying its marker.
fn marker(i: usize) -> String {
    format!("SECRET-MARKER-{i:04}")
}

/// The inline tag note `i` carries.
fn tag_name(i: usize) -> String {
    format!("TAGMARK{i:04}")
}

fn body_of(i: usize) -> String {
    let m = marker(i);
    format!(
        "---\nowner: {m}\nlevel: {m}\n---\nThe plan {m} is here. #{tag}\n- [ ] do {m}\n- [x] done {m}\n```rust\nlet code = \"{m}\";\n```\n",
        tag = tag_name(i)
    )
}

/// Whether note `i` is locked and the session isn't open: its text must not
/// come back.
fn is_closed(locked: &[bool], open: bool, i: usize) -> bool {
    locked[i] && !open
}

/// Every row that is a locked note carries its title and its lock only.
fn check_rows(rows: &[NoteRow], ids: &[String], locked: &[bool]) -> Result<(), TestCaseError> {
    for row in rows {
        if let Some(i) = ids.iter().position(|id| *id == row.id) {
            prop_assert_eq!(row.locked, locked[i], "{:?}", row);
            if row.locked {
                prop_assert!(row.snippet.is_empty(), "{:?}", row);
                prop_assert!(row.tags.is_empty(), "{:?}", row);
                prop_assert_eq!(row.status.as_str(), "none", "{:?}", row);
                prop_assert_eq!((row.tasks_done, row.tasks_total), (0, 0), "{:?}", row);
            }
        }
    }
    Ok(())
}

fn palette() -> PaletteDto {
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{},"preview":{}}"##;
    Renderer::theme_tokens(theme.into()).unwrap().palette
}

/// The secret the platform keeps, and one that isn't it.
const SECRET_B: [u8; 32] = [9; 32];
const OTHER_B: [u8; 32] = [8; 32];

/// What no key leaves open: every note locked reads closed by every path.
fn sweep(
    library: &kanso_mobile::Library,
    ids: &[String],
    locked: &[bool],
) -> Result<(), TestCaseError> {
    for (i, id) in ids.iter().enumerate() {
        if !locked[i] {
            continue;
        }
        let view = library.note(id).unwrap();
        prop_assert_eq!(&view.body, &None);
        prop_assert!(view.lock.is_closed());
        prop_assert_eq!(
            library.save(id, &view.rev, "x".into()),
            Err(KansoError::Locked)
        );
        prop_assert!(!library.locks().is_note_unlocked(id).unwrap());
        let text = format!(
            "{:?}",
            (
                library.list(ListQuery::default()).unwrap().rows,
                library.search(&marker(i), SearchScope::All, 0).unwrap(),
                library.renderer().preview_html(id, palette()),
            )
        );
        prop_assert!(!text.contains(&marker(i)), "{text}");
    }
    Ok(())
}

/// One call of the sequence.
#[derive(Clone, Debug)]
enum Op {
    ListAll,
    ListTrash,
    ReadNote(usize),
    Search(usize),
    SearchWord,
    Properties(usize),
    QueryView,
    Preview(usize),
    Save(usize),
    Create,
    Trash(usize),
    LockNote(usize),
    LockSession,
    UnlockRight,
    UnlockWrong,
    IsNoteUnlocked(usize),
    Idle,
    /// The app goes to the background.
    Background,
    /// Biometrics are turned on.
    BioEnroll,
    /// Biometrics are asked to open the notes, with the right secret or not.
    BioUnlock(bool),
    BioOff,
    /// A place and narrowing of the list: which place, status narrowing,
    /// pinned only, which sort.
    List(usize, bool, bool, usize),
    Change(usize, usize),
    SearchIn(usize, usize),
    CreateNotebook,
    SaveWithTag(usize),
    EditOpen(usize),
    EditCalls(usize),
    EditType(usize),
    TypeThenLock(usize),
    /// The page of a note that links every note by wiki-link and by
    /// address and lists them in a query view.
    PreviewHub,
    /// What a tapped address in the hub's page is, for note `i`: with a
    /// heading, so the anchor looks into the note's text.
    PreviewLink(usize),
    /// The first code block of note `i`.
    CodeBlock(usize),
    /// Ticks the first task of note `i`, as the page last drew it.
    Toggle(usize),
    /// A note made from a wiki-link in note `i`.
    CreateLinked(usize),
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    let index = 0..NOTES;
    proptest::collection::vec(
        prop_oneof![
            2 => Just(Op::ListAll),
            1 => Just(Op::ListTrash),
            3 => index.clone().prop_map(Op::ReadNote),
            2 => index.clone().prop_map(Op::Search),
            1 => Just(Op::SearchWord),
            2 => index.clone().prop_map(Op::Properties),
            1 => Just(Op::QueryView),
            2 => index.clone().prop_map(Op::Preview),
            2 => index.clone().prop_map(Op::Save),
            1 => Just(Op::Create),
            1 => index.clone().prop_map(Op::Trash),
            2 => index.clone().prop_map(Op::LockNote),
            2 => Just(Op::LockSession),
            2 => Just(Op::UnlockRight),
            1 => Just(Op::UnlockWrong),
            1 => index.clone().prop_map(Op::IsNoteUnlocked),
            1 => Just(Op::Idle),
            2 => Just(Op::Background),
            2 => Just(Op::BioEnroll),
            2 => any::<bool>().prop_map(Op::BioUnlock),
            1 => Just(Op::BioOff),
            4 => (0..6usize, any::<bool>(), any::<bool>(), 0..3usize)
                .prop_map(|(p, s, o, k)| Op::List(p, s, o, k)),
            3 => (index.clone(), 0..3usize).prop_map(|(i, k)| Op::Change(i, k)),
            3 => (0..4usize, 0..5usize).prop_map(|(s, w)| Op::SearchIn(s, w)),
            1 => Just(Op::CreateNotebook),
            1 => index.clone().prop_map(Op::SaveWithTag),
            3 => index.clone().prop_map(Op::EditOpen),
            3 => index.clone().prop_map(Op::EditCalls),
            2 => index.clone().prop_map(Op::EditType),
            2 => index.clone().prop_map(Op::TypeThenLock),
            2 => Just(Op::PreviewHub),
            2 => index.clone().prop_map(Op::PreviewLink),
            2 => index.clone().prop_map(Op::CodeBlock),
            2 => index.clone().prop_map(Op::Toggle),
            1 => index.clone().prop_map(Op::CreateLinked),
        ],
        1..24,
    )
}

/// The events of a session, as debug text, in the order they came.
type Log = Arc<Mutex<Vec<String>>>;

fn listen(session: &EditorSession) -> Log {
    let log: Log = Arc::default();
    let sink = log.clone();
    session.events(move |event| {
        sink.lock().unwrap().push(format!("{event:?}"));
        true
    });
    log
}

fn caret(at: u32) -> Range16 {
    Range16 { start: at, end: at }
}

/// Types `text` at the start of the session's text. `Ok(false)` when the
/// text changed under it meanwhile.
fn type_first(session: &EditorSession, text: &str) -> Result<bool, KansoError> {
    let version = session.snapshot()?.version;
    match session.apply(TextChange {
        base_version: version,
        start: 0,
        end: 0,
        text: text.to_owned(),
        selection: caret(text.encode_utf16().count() as u32),
        composing: None,
    }) {
        Ok(_) => Ok(true),
        Err(KansoError::Invalid { .. }) => Ok(false),
        Err(err) => Err(err),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn prop_a_closed_note_never_comes_back(
        initially_locked in proptest::collection::vec(any::<bool>(), NOTES),
        ops in ops(),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let locks = library.locks();
        let ids: Vec<String> = (0..NOTES)
            .map(|i| note(library, &format!("Note {i}"), &body_of(i)))
            .collect();

        // The streams, collected as debug text.
        let heard = Arc::new(Mutex::new(Vec::<String>::new()));
        let log = heard.clone();
        library.changes(move |ids| { log.lock().unwrap().push(format!("{ids:?}")); true });
        let log = heard.clone();
        locks.lock_events(move |event: LockEvent| { log.lock().unwrap().push(format!("{event:?}")); true });
        let log = heard.clone();
        library.sync().status_stream(move |status| { log.lock().unwrap().push(format!("{status:?}")); true });

        // The model: which notes are locked, and whether the session is open.
        let mut locked = [false; NOTES];
        let mut open = false;
        let mut any_lock = false;
        for (i, lock) in initially_locked.iter().enumerate() {
            if *lock {
                locks.lock_note(&ids[i], LOCK_PASSWORD.into()).unwrap();
                locked[i] = true;
                any_lock = true;
                open = true;
            }
        }
        let mut trashed = [false; NOTES];
        let mut biometrics_on = false;
        // The sessions open for editing, which of them still work, and the
        // events each one sent.
        let mut sessions: Vec<Option<EditorSession>> = (0..NOTES).map(|_| None).collect();
        let mut alive = [false; NOTES];
        let mut logs: Vec<(usize, Log)> = Vec::new();
        let hub = note(
            library,
            "Hub",
            &format!(
                "{}\n```query\nfrom: SECRET\ncolumns: owner, level\n```\n",
                (0..NOTES)
                    .map(|i| format!("[[Note {i}#Plan]] [there](inkdrop://note/{}#Plan)", ids[i].trim_start_matches("note:")))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        );

        for op in ops {
            // What is closed as this call starts, and what it returns, as text.
            let closed_now: Vec<usize> =
                (0..NOTES).filter(|i| is_closed(&locked, open, *i)).collect();
            let mut returned: Vec<String> = Vec::new();
            match &op {
                Op::ListAll | Op::ListTrash => {
                    let trash = matches!(op, Op::ListTrash);
                    let place = if trash { ListPlace::Trash } else { ListPlace::All };
                    let rows = library.list(ListQuery { place, ..ListQuery::default() }).unwrap().rows;
                    check_rows(&rows, &ids, &locked)?;
                    returned.push(format!("{rows:?}"));
                }
                Op::ReadNote(i) => {
                    let view = library.note(&ids[*i]).unwrap();
                    if is_closed(&locked, open, *i) {
                        prop_assert_eq!(&view.body, &None);
                        prop_assert!(view.lock.is_closed());
                        prop_assert!(view.tags.is_empty());
                        prop_assert_eq!(view.status.as_str(), "none");
                    } else {
                        // The other side: what may be read is read.
                        prop_assert!(view.body.as_deref().is_some_and(|b| b.contains(&marker(*i))));
                    }
                    returned.push(format!("{view:?}"));
                }
                Op::Search(i) => {
                    let hits = library.search(&marker(*i), SearchScope::All, 0).unwrap();
                    if !locked[*i] && !trashed[*i] {
                        prop_assert!(hits.iter().any(|h| h.row.id == ids[*i]), "a plain note is found");
                    }
                    returned.push(format!("{hits:?}"));
                }
                Op::SearchWord => {
                    returned.push(format!("{:?}", library.search("SECRET MARKER plan", SearchScope::All, 0).unwrap()));
                }
                Op::Properties(i) => {
                    let props = library.properties(&ids[*i]).unwrap();
                    prop_assert_eq!(props.is_empty(), is_closed(&locked, open, *i));
                    returned.push(format!("{props:?}"));
                }
                Op::QueryView => {
                    let table = library
                        .query_view("from: SECRET\ncolumns: owner, level", None)
                        .unwrap();
                    // Locked notes are never listed, open or not.
                    for row in &table.rows {
                        // (The hub holds the word in its query; it isn't one of the notes.)
                        let Some(at) = ids.iter().position(|id| *id == row.id) else {
                            continue;
                        };
                        prop_assert!(!locked[at]);
                    }
                    returned.push(format!("{table:?}"));
                }
                Op::Preview(i) => {
                    match library.renderer().preview_html(&ids[*i], palette()) {
                        Ok(page) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            prop_assert!(page.contains(&marker(*i)));
                            returned.push(page);
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::Save(i) => {
                    let rev = library.note(&ids[*i]).unwrap().rev;
                    let edited = format!("{}\nedited {}", body_of(*i), marker(*i));
                    match library.save(&ids[*i], &rev, edited) {
                        Ok(saved) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            if locked[*i] { open = true; }
                            returned.push(format!("{saved:?}"));
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::List(place, status, pinned_only, sort) => {
                    let notebooks = library.notebooks().unwrap();
                    let tags = library.tags().unwrap();
                    let places: Vec<ListPlace> = match place {
                        0 => vec![ListPlace::All],
                        1 => vec![ListPlace::Pinned],
                        2 => vec![ListPlace::Recent],
                        3 => notebooks.iter().map(|b| ListPlace::Notebook { id: b.id.clone() }).collect(),
                        4 => tags.iter().map(|t| ListPlace::Tag { id: t.id.clone() }).collect(),
                        _ => vec![ListPlace::Trash],
                    };
                    for place in places {
                        let by_tag = matches!(place, ListPlace::Tag { .. });
                        let list = library
                            .list(ListQuery {
                                place,
                                status_only: status.then(|| "none".to_owned()),
                                pinned_only: *pinned_only,
                                sort: [SortKey::Updated, SortKey::Created, SortKey::Title][*sort],
                                descending: true,
                            })
                            .unwrap();
                        check_rows(&list.rows, &ids, &locked)?;
                        if by_tag || *status {
                            // Made of what a note holds: a locked note is
                            // not in it.
                            for row in &list.rows {
                                if let Some(i) = ids.iter().position(|id| *id == row.id) {
                                    prop_assert!(!locked[i], "a locked note in a tag or status list: {:?}", row);
                                }
                            }
                        }
                        returned.push(format!("{list:?}"));
                    }
                    returned.push(format!("{:?}", library.overview().unwrap()));
                }
                Op::Change(i, kind) => {
                    let notebooks = library.notebooks().unwrap();
                    let change = match kind {
                        0 => NoteChange::TogglePin,
                        1 => NoteChange::Trash,
                        _ => NoteChange::Move { notebook_id: notebooks.last().unwrap().id.clone() },
                    };
                    // Pinning, trashing and moving between unlocked
                    // notebooks are allowed on a locked note, open or not.
                    let written = library.change(vec![ids[*i].clone()], change).unwrap().count;
                    if *kind == 1 {
                        trashed[*i] = true;
                    }
                    returned.push(format!("{written}"));
                }
                Op::SearchIn(scope, word) => {
                    let notebooks = library.notebooks().unwrap();
                    let scope = match scope {
                        0 => SearchScope::All,
                        1 => SearchScope::Titles,
                        2 => SearchScope::Tags,
                        _ => SearchScope::Notebook { id: notebooks.first().unwrap().id.clone() },
                    };
                    let word = ["SECRET", "TAGMARK0000", "Note", "status:none", "tag:TAGMARK0001"][*word];
                    let by_content = matches!(scope, SearchScope::Tags)
                        || word.starts_with("status:")
                        || word.starts_with("tag:");
                    let hits = library.search(word, scope.clone(), 0).unwrap();
                    let rows: Vec<NoteRow> = hits.iter().map(|h| h.row.clone()).collect();
                    check_rows(&rows, &ids, &locked)?;
                    if by_content {
                        for row in &rows {
                            if let Some(i) = ids.iter().position(|id| *id == row.id) {
                                prop_assert!(!locked[i], "a locked note found by its tags or status: {:?}", row);
                            }
                        }
                    }
                    if scope == SearchScope::Tags && word == "TAGMARK0000" {
                        // The other side: a plain note is found by its tag.
                        if !locked[0] && !trashed[0] {
                            prop_assert!(rows.iter().any(|r| r.id == ids[0]), "not found by its tag");
                        }
                    }
                    returned.push(format!("{hits:?}"));
                }
                Op::CreateNotebook => {
                    let made = library.create_notebook("Garden".into(), None).unwrap();
                    returned.push(format!("{made:?}"));
                }
                Op::SaveWithTag(i) => {
                    let rev = library.note(&ids[*i]).unwrap().rev;
                    let tag = format!("extra{i}");
                    let edited = format!("{} #{tag}", body_of(*i));
                    match library.save(&ids[*i], &rev, edited) {
                        Ok(saved) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            if locked[*i] {
                                open = true;
                            } else {
                                // A plain note takes the tag its text names.
                                let names: Vec<String> =
                                    library.tags().unwrap().into_iter().map(|t| t.name).collect();
                                prop_assert!(names.iter().any(|n| n.eq_ignore_ascii_case(&tag)), "{names:?}");
                                let view = library.note(&ids[*i]).unwrap();
                                prop_assert!(!view.tags.is_empty());
                            }
                            returned.push(format!("{saved:?}"));
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::EditOpen(i) => {
                    match library.edit(&ids[*i]) {
                        Ok(session) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            let snapshot = session.snapshot().unwrap();
                            returned.push(format!("{snapshot:?}"));
                            let log = listen(&session);
                            logs.push((*i, log.clone()));
                            sessions[*i] = Some(session);
                            alive[*i] = true;
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::EditCalls(i) => {
                    if let Some(session) = &sessions[*i] {
                        let shown = [
                            session.snapshot().map(|s| format!("{s:?}")),
                            session.styled_runs(0, 400, "kanso-light".into()).map(|r| format!("{r:?}")),
                            session.completion().map(|c| {
                                let text = format!("{c:?}");
                                // Titles only: nothing of any note's text.
                                assert!(!text.contains("SECRET-MARKER"), "{text}");
                                text
                            }),
                            session.link_at(0).map(|l| format!("{l:?}")),
                            session.select(caret(0), false).map(|c| format!("{c:?}")),
                            session.undo().map(|o| format!("{o:?}")),
                            session.set_property(None, "added".into(), "1".into()).map(|p| format!("{p:?}")),
                        ];
                        for result in shown {
                            match result {
                                Ok(text) => {
                                    prop_assert!(alive[*i], "a call of an ended session worked: {text}");
                                    returned.push(text);
                                }
                                Err(err) => {
                                    prop_assert_eq!(err, KansoError::Locked);
                                    prop_assert!(!alive[*i]);
                                }
                            }
                        }
                    }
                }
                Op::EditType(i) => {
                    if let Some(session) = &sessions[*i] {
                        // `[[N` and a caret after it: titles to complete.
                        match type_first(session, "[[N") {
                            Ok(_) => prop_assert!(alive[*i]),
                            Err(err) => {
                                prop_assert_eq!(err, KansoError::Locked);
                                prop_assert!(!alive[*i]);
                            }
                        }
                    }
                }
                Op::TypeThenLock(i) => {
                    // Typing in an open locked note, then the lock closing:
                    // the typing is saved, sealed, first.
                    if locked[*i] && open {
                        let session = library.edit(&ids[*i]).unwrap();
                        let log = listen(&session);
                        logs.push((*i, log.clone()));
                        let typed = format!("TYPED-{i} ");
                        prop_assert!(type_first(&session, &typed).unwrap());
                        locks.lock().unwrap();
                        for j in 0..NOTES {
                            alive[j] &= !locked[j];
                        }
                        // The session is over, and said so.
                        prop_assert_eq!(session.snapshot().map(|_| ()), Err(KansoError::Locked));
                        prop_assert_eq!(session.flush().map(|_| ()), Err(KansoError::Locked));
                        prop_assert_eq!(type_first(&session, "more"), Err(KansoError::Locked));
                        prop_assert!(log.lock().unwrap().last().is_some_and(|e| e == "Closed"));
                        // The store has the typing, and not in the clear.
                        let raw = library.store().note(&ids[*i]).unwrap().unwrap();
                        prop_assert!(raw.is_locked() && raw.body.is_empty());
                        // Open again, it is there.
                        prop_assert!(locks.unlock(LOCK_PASSWORD.into()).unwrap());
                        open = true;
                        let view = library.note(&ids[*i]).unwrap();
                        prop_assert!(view.body.as_deref().is_some_and(|b| b.contains(&typed)), "{:?}", view.body);
                    }
                }
                Op::PreviewHub => {
                    let page = library.renderer().preview_html(&hub, palette()).unwrap();
                    // The other side: a locked note is linked, whatever its lock.
                    for id in &ids {
                        prop_assert!(page.contains(id.trim_start_matches("note:")), "{id} not linked");
                    }
                    returned.push(page);
                }
                Op::PreviewLink(i) => {
                    let url = format!("inkdrop://note/{}#Plan", ids[*i].trim_start_matches("note:"));
                    let target = library.renderer().link_target(&hub, &url).unwrap();
                    match &target {
                        kanso_mobile::PreviewLink::Note { id, heading, anchor } => {
                            prop_assert_eq!(id, &ids[*i]);
                            prop_assert_eq!(heading.as_deref(), Some("Plan"));
                            // An anchor is only known from text that can be read.
                            if is_closed(&locked, open, *i) {
                                prop_assert_eq!(anchor, &None);
                            }
                        }
                        // Trashed notes are not targets.
                        kanso_mobile::PreviewLink::Nothing => prop_assert!(trashed[*i]),
                        other => prop_assert!(false, "{other:?}"),
                    }
                    returned.push(format!("{target:?}"));
                }
                Op::CodeBlock(i) => match library.renderer().code_block(&ids[*i], 0) {
                    Ok(code) => {
                        prop_assert!(!is_closed(&locked, open, *i));
                        prop_assert!(code.contains(&marker(*i)), "{code}");
                        returned.push(code);
                    }
                    Err(err) => {
                        prop_assert_eq!(err, KansoError::Locked);
                        prop_assert!(is_closed(&locked, open, *i));
                    }
                },
                Op::Toggle(i) => {
                    let rev = library.note(&ids[*i]).unwrap().rev;
                    match library.toggle_task_at(&ids[*i], 0, &rev) {
                        Ok(ticked) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            if locked[*i] { open = true; }
                            returned.push(format!("{ticked:?}"));
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::CreateLinked(i) => {
                    let made = library.create_linked_note("Brand new", &ids[*i]).unwrap();
                    returned.push(format!("{made:?}"));
                }
                Op::Create => {
                    let saved = library
                        .create(kanso_mobile::NewNote {
                            notebook_id: None,
                            title: "Fresh".into(),
                            body: "fresh note".into(),
                            ..Default::default()
                        })
                        .unwrap();
                    returned.push(format!("{saved:?}"));
                }
                Op::Trash(i) => {
                    library.trash(&ids[*i]).unwrap();
                    trashed[*i] = true;
                }
                Op::LockNote(i) => {
                    match locks.lock_note(&ids[*i], LOCK_PASSWORD.into()) {
                        Ok(saved) => {
                            prop_assert!(!locked[*i]);
                            locked[*i] = true;
                            any_lock = true;
                            open = true;
                            returned.push(format!("{saved:?}"));
                        }
                        // Already locked.
                        Err(KansoError::Invalid { .. }) => prop_assert!(locked[*i]),
                        Err(other) => prop_assert!(false, "unexpected {other:?}"),
                    }
                }
                Op::LockSession => {
                    locks.lock().unwrap();
                    open = false;
                    for i in 0..NOTES {
                        // Sessions on locked notes end with the lock.
                        alive[i] &= !locked[i];
                    }
                    sweep(library, &ids, &locked)?;
                }
                Op::UnlockRight => match locks.unlock(LOCK_PASSWORD.into()) {
                    Ok(true) => { prop_assert!(any_lock); open = true; }
                    Ok(false) => prop_assert!(false, "the right password was refused"),
                    Err(_) => prop_assert!(!any_lock),
                },
                Op::UnlockWrong => {
                    let answer = locks.unlock("not the right one".into());
                    prop_assert!(matches!(answer, Ok(false) | Err(_)), "{answer:?}");
                }
                Op::IsNoteUnlocked(i) => {
                    prop_assert_eq!(locks.is_note_unlocked(&ids[*i]).unwrap(), !is_closed(&locked, open, *i));
                }
                Op::Idle => {
                    let later = Instant::now() + AUTO_LOCK + Duration::from_secs(1);
                    prop_assert_eq!(locks.lock_if_idle(later), open);
                    open = false;
                    for i in 0..NOTES {
                        alive[i] &= !locked[i];
                    }
                    sweep(library, &ids, &locked)?;
                }
                Op::Background => {
                    locks.lock_for_background().unwrap();
                    open = false;
                    prop_assert!(!locks.is_unlocked().unwrap());
                    for i in 0..NOTES {
                        alive[i] &= !locked[i];
                    }
                    sweep(library, &ids, &locked)?;
                }
                Op::BioEnroll => match locks.begin_biometric_enrollment() {
                    Ok(ticket) => {
                        prop_assert!(open);
                        prop_assert_eq!(
                            biometrics::deliver(ticket, 0, Some(&SECRET_B)),
                            1
                        );
                        biometrics_on = true;
                    }
                    Err(err) => {
                        prop_assert_eq!(err, KansoError::Locked);
                        prop_assert!(!open);
                    }
                },
                Op::BioUnlock(right) => {
                    let ticket = locks.begin_biometric_unlock().unwrap();
                    prop_assert_eq!(ticket.is_some(), biometrics_on);
                    if let Some(ticket) = ticket {
                        let b = if *right { SECRET_B } else { OTHER_B };
                        let answer = biometrics::deliver(ticket, 0, Some(&b));
                        if *right {
                            prop_assert_eq!(answer, 1);
                            open = true;
                        } else {
                            prop_assert_eq!(answer, -1);
                            biometrics_on = false;
                            prop_assert_eq!(locks.begin_biometric_unlock().unwrap(), None);
                        }
                    }
                }
                Op::BioOff => {
                    locks.turn_off_biometrics().unwrap();
                    biometrics_on = false;
                }
            }
            prop_assert_eq!(locks.is_unlocked().unwrap(), open, "the session after {:?}", op);

            // The guarantee: nothing this call returned holds the marker of a
            // note that was closed when it was asked.
            for i in &closed_now {
                let m = marker(*i);
                for text in &returned {
                    prop_assert!(!text.contains(&m), "{m} leaked from {op:?}: {text}");
                }
            }
        }

        // A session says it is over once, and nothing of the note after.
        for (i, log) in &logs {
            let events = log.lock().unwrap();
            if let Some(closed) = events.iter().position(|e| e == "Closed") {
                prop_assert_eq!(closed + 1, events.len(), "events after Closed for {}: {:?}", i, events);
            }
            prop_assert!(events.iter().all(|e| !e.contains("TYPED-") || e.starts_with("Reloaded")));
        }

        // Streams only ever carry IDs and states: no marker at all.
        // (Give the change thread a moment to deliver what it was sent.)
        std::thread::sleep(Duration::from_millis(30));
        for text in heard.lock().unwrap().iter() {
            prop_assert!(!text.contains("SECRET-MARKER"), "a stream carried text: {text}");
        }
    }
}
