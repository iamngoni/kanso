//! Agent tasks: tools that stage a change set and write nothing, a scope the
//! agent cannot get out of, and one transaction to apply or undo it.

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;
use std::time::Duration;

use kanso_core::DocType;
use kanso_server::agent_tools;
use kanso_server::{
    AgentScope, ChangeSet, Selection, TaskId, TaskLimits, TaskOps, TokenId, TokenStore,
    undo_changes,
};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

struct Task {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    tokens: TokenStore,
    token: TokenId,
    scope: Arc<AgentScope>,
    id: TaskId,
}

fn limits() -> TaskLimits {
    TaskLimits {
        lifetime: Duration::from_secs(1200),
        calls: 200,
    }
}

fn task(
    store_setup: impl FnOnce(&Store) -> (Option<HashSet<String>>, Option<HashSet<String>>),
    ops: TaskOps,
) -> Task {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let (books, notes) = store_setup(&store);
    let tokens = TokenStore::new();
    let id = tokens.changesets().next_id();
    let scope = AgentScope::task(books, notes, "Claude Agent", id, ops, limits());
    let minted = tokens.mint(scope.clone());
    let scope = Arc::new(scope);
    tokens.changesets().open(ChangeSet::new(
        id,
        "Merge my trip notes",
        "Claude Agent",
        scope.clone(),
        BTreeSet::new(),
    ));
    Task {
        _dir: dir,
        store,
        tokens,
        token: minted.id,
        scope,
        id,
    }
}

impl Task {
    fn call(&self, name: &str, args: Value) -> Value {
        agent_tools::call(
            &self.store,
            &self.tokens,
            self.token,
            &self.scope,
            name,
            Some(&args),
        )
    }

    fn set(&self) -> ChangeSet {
        self.tokens.changesets().get(self.id).unwrap()
    }
}

fn text(answer: &Value) -> String {
    answer["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_owned()
}

fn is_error(answer: &Value) -> bool {
    answer["isError"] == json!(true)
}

fn book(store: &Store, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({ "name": name }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, book: &str, title: &str, body: &str) -> (String, String) {
    let w = store
        .put(
            DocType::Note,
            json!({ "bookId": book, "title": title, "body": body }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (w.id, w.rev)
}

fn lock(store: &Store, id: &str) {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
    store.lock_note(note).unwrap();
}

struct Library {
    trips: String,
    archive: String,
    notes: Vec<(String, String)>,
}

fn library(store: &Store) -> Library {
    let trips = book(store, "Trips");
    let archive = book(store, "Archive");
    let notes = (0..4)
        .map(|n| note(store, &trips, &format!("Trip {n}"), &format!("day {n}")))
        .collect();
    Library {
        trips,
        archive,
        notes,
    }
}

fn whole_library(store: &Store) -> (Library, Option<HashSet<String>>) {
    let lib = library(store);
    (lib, None)
}

fn in_trips() -> Task {
    let mut lib = None;
    let t = task(
        |s| {
            let l = library(s);
            let books = HashSet::from([l.trips.clone()]);
            lib = Some(l);
            (Some(books), None)
        },
        TaskOps::default(),
    );
    let _ = lib;
    t
}

fn notes_of(task: &Task) -> Vec<(String, String)> {
    let trips = task
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Trips")
        .unwrap()
        .id;
    let mut notes: Vec<_> = task
        .store
        .notes_in_books(&[trips])
        .unwrap()
        .into_iter()
        .map(|n| (n.id, n.rev.unwrap()))
        .collect();
    notes.sort();
    notes
}

fn book_named(task: &Task, name: &str) -> String {
    task.store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == name)
        .unwrap()
        .id
}

#[test]
fn a_merge_trashes_its_sources_and_undo_restores_them() {
    let t = in_trips();
    let notes = notes_of(&t);
    let trips = book_named(&t, "Trips");
    let answer = t.call(
        "kanso-propose-merge",
        json!({
            "sources": notes[..3].iter().map(|(id, rev)| json!({"_id": id, "_rev": rev})).collect::<Vec<_>>(),
            "bookId": trips,
            "title": "Lisbon plan",
            "body": "All of it. [[Trip 0]]",
        }),
    );
    assert!(!is_error(&answer), "{}", text(&answer));
    assert!(text(&answer).contains("Nothing is written"));
    // Staging wrote nothing.
    for (id, _) in &notes {
        assert_eq!(t.store.note(id).unwrap().unwrap().book_id, trips);
    }
    let set = t.set();
    let applied = set
        .apply(&t.store, &Selection::all(set.ops.len()), 50)
        .unwrap();
    assert_eq!(applied.created.len(), 1);
    for (id, _) in &notes[..3] {
        assert_eq!(t.store.note(id).unwrap().unwrap().book_id, "trash");
    }
    assert_eq!(t.store.note(&notes[3].0).unwrap().unwrap().book_id, trips);
    let merged = t.store.note(&applied.created[0]).unwrap().unwrap();
    assert_eq!(merged.title, "Lisbon plan");

    let report = undo_changes(&t.store, &applied.batch, 60).unwrap();
    assert!(report.skipped.is_empty());
    for (id, _) in &notes {
        assert_eq!(t.store.note(id).unwrap().unwrap().book_id, trips);
    }
    assert!(
        t.store
            .get(&applied.created[0])
            .unwrap()
            .is_none_or(|d| d.deleted)
    );
}

#[test]
fn an_op_outside_the_scope_is_not_found() {
    let t = task(
        |s| {
            let (lib, _) = whole_library(s);
            let other = book(s, "Private");
            let (secret, _) = note(s, &other, "Secret", "s");
            let _ = (secret, lib.archive.clone());
            (Some(HashSet::from([lib.trips.clone()])), None)
        },
        TaskOps::default(),
    );
    let private = book_named(&t, "Private");
    let secret = t
        .store
        .notes_in_books(&[private.clone()])
        .unwrap()
        .remove(0);
    let answer = t.call(
        "update-note",
        json!({"_id": secret.id, "_rev": secret.rev.clone().unwrap(), "body": "x"}),
    );
    assert!(is_error(&answer));
    let answer = t.call(
        "kanso-propose-move",
        json!({"_id": notes_of(&t)[0].0, "_rev": notes_of(&t)[0].1, "bookId": private}),
    );
    assert!(
        is_error(&answer),
        "a notebook outside the scope is not found"
    );
    assert!(t.set().ops.is_empty());
}

#[test]
fn notes_chosen_by_id_are_the_only_ones_visible() {
    let mut chosen = None;
    let t = task(
        |s| {
            let lib = library(s);
            chosen = Some(lib.notes[0].0.clone());
            (None, Some(HashSet::from([lib.notes[0].0.clone()])))
        },
        TaskOps::default(),
    );
    let all = notes_of(&t);
    let ok = t.call("read-note", json!({"noteId": chosen.unwrap()}));
    assert!(!is_error(&ok), "{}", text(&ok));
    let other = all.iter().find(|(id, _)| id != &chosen_id(&ok)).unwrap();
    let denied = t.call("read-note", json!({"noteId": other.0}));
    assert!(is_error(&denied));
}

fn chosen_id(answer: &Value) -> String {
    let body: Value = serde_json::from_str(&text(answer)).unwrap_or(Value::Null);
    body["_id"].as_str().unwrap_or("").to_owned()
}

#[test]
fn a_task_may_only_propose_what_the_person_allowed() {
    let t = task(
        |s| {
            let lib = library(s);
            (Some(HashSet::from([lib.trips.clone()])), None)
        },
        TaskOps {
            moves: false,
            merge: false,
            ..TaskOps::default()
        },
    );
    let names: Vec<String> = agent_tools::definitions_for(&t.scope)
        .iter()
        .map(|d| d["name"].as_str().unwrap().to_owned())
        .collect();
    assert!(!names.contains(&"kanso-propose-move".to_owned()));
    assert!(!names.contains(&"kanso-propose-merge".to_owned()));
    assert!(names.contains(&"kanso-task-done".to_owned()));
    let n = notes_of(&t);
    let answer = t.call(
        "kanso-propose-move",
        json!({"_id": n[0].0, "_rev": n[0].1, "bookId": book_named(&t, "Archive")}),
    );
    assert!(is_error(&answer));
}

#[test]
fn a_later_edit_of_the_same_note_replaces_the_earlier_one_and_temp_ids_work() {
    let t = in_trips();
    let n = notes_of(&t);
    let trips = book_named(&t, "Trips");
    t.call(
        "update-note",
        json!({"_id": n[0].0, "_rev": n[0].1, "body": "first"}),
    );
    t.call(
        "patch-note",
        json!({"_id": n[0].0, "_rev": n[0].1, "old_string": "first", "new_string": "second"}),
    );
    let made = t.call(
        "create-note",
        json!({"bookId": trips, "title": "Plan", "body": "draft"}),
    );
    assert!(text(&made).contains("note:new-1"), "{}", text(&made));
    let edited = t.call(
        "update-note",
        json!({"_id": "note:new-1", "_rev": "-", "body": "final"}),
    );
    assert!(!is_error(&edited), "{}", text(&edited));
    let set = t.set();
    assert_eq!(set.ops.len(), 2);
    let listed = t.call("kanso-list-proposed", json!({}));
    assert!(text(&listed).contains("Plan"));
    let done = t.call("kanso-task-done", json!({"summary": "Did two things"}));
    assert!(!is_error(&done));
    let set = t.set();
    assert!(set.done);
    assert_eq!(set.summary, "Did two things");
    let applied = set.apply(&t.store, &Selection::all(2), 9).unwrap();
    assert_eq!(t.store.note(&n[0].0).unwrap().unwrap().body, "second");
    assert_eq!(
        t.store.note(&applied.created[0]).unwrap().unwrap().body,
        "final"
    );
}

#[test]
fn a_changed_note_is_a_conflict_unless_applied_anyway() {
    let t = in_trips();
    let n = notes_of(&t);
    t.call(
        "update-note",
        json!({"_id": n[0].0, "_rev": n[0].1, "body": "agent"}),
    );
    t.store
        .put(
            DocType::Note,
            json!({"_id": n[0].0, "body": "person"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let set = t.set();
    assert!(matches!(
        set.state_of(&t.store, &set.ops[0]),
        kanso_server::OpState::Changed { .. }
    ));
    assert_eq!(
        set.apply(&t.store, &Selection::all(1), 9),
        Err(kanso_server::ApplyFailure::Conflict)
    );
    assert_eq!(t.store.note(&n[0].0).unwrap().unwrap().body, "person");
    let mut sel = Selection::all(1);
    sel.apply_anyway.insert(0);
    set.apply(&t.store, &sel, 9).unwrap();
    assert_eq!(t.store.note(&n[0].0).unwrap().unwrap().body, "agent");
}

#[test]
fn the_task_tools_are_not_listed_to_the_full_api() {
    let names: Vec<String> = kanso_server::agent_tools::definitions_for(&AgentScope::ask(None))
        .iter()
        .map(|d| d["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["read-note", "search-notes"]);
}

#[test]
fn a_task_token_gets_its_own_call_budget() {
    let t = task(
        |s| {
            let lib = library(s);
            (Some(HashSet::from([lib.trips])), None)
        },
        TaskOps::default(),
    );
    for _ in 0..60 {
        let a = t.call("list-notebooks", json!({}));
        assert!(!is_error(&a), "{}", text(&a));
    }
}

#[derive(Clone, Debug)]
enum Call {
    Create(String, String),
    Update(usize, String),
    Patch(usize, String),
    Move(usize),
    Merge(Vec<usize>, String),
    Temp(String),
    Done,
    List,
}

fn call_strategy() -> impl Strategy<Value = Call> {
    prop_oneof![
        ("[a-z]{1,6}", "[a-z ]{0,20}").prop_map(|(t, b)| Call::Create(t, b)),
        (0usize..5, "[a-z ]{0,20}").prop_map(|(n, b)| Call::Update(n, b)),
        (0usize..5, "[a-z]{1,3}").prop_map(|(n, b)| Call::Patch(n, b)),
        (0usize..5).prop_map(Call::Move),
        (prop::collection::vec(0usize..5, 0..5), "[a-z]{1,6}").prop_map(|(s, t)| Call::Merge(s, t)),
        "[a-z ]{0,10}".prop_map(Call::Temp),
        Just(Call::Done),
        Just(Call::List),
    ]
}

fn run(t: &Task, notes: &[(String, String)], trips: &str, archive: &str, c: &Call) {
    let pick = |n: usize| &notes[n % notes.len()];
    match c {
        Call::Create(title, body) => {
            t.call(
                "create-note",
                json!({"bookId": trips, "title": title, "body": body}),
            );
        }
        Call::Update(n, body) => {
            let (id, rev) = pick(*n);
            t.call("update-note", json!({"_id": id, "_rev": rev, "body": body}));
        }
        Call::Patch(n, new) => {
            let (id, rev) = pick(*n);
            t.call(
                "patch-note",
                json!({"_id": id, "_rev": rev, "old_string": "day", "new_string": new}),
            );
        }
        Call::Move(n) => {
            let (id, rev) = pick(*n);
            t.call(
                "kanso-propose-move",
                json!({"_id": id, "_rev": rev, "bookId": archive}),
            );
        }
        Call::Merge(sources, title) => {
            let refs: Vec<Value> = sources
                .iter()
                .map(|n| {
                    let (id, rev) = pick(*n);
                    json!({"_id": id, "_rev": rev})
                })
                .collect();
            t.call(
                "kanso-propose-merge",
                json!({"sources": refs, "bookId": archive, "title": title, "body": "merged"}),
            );
        }
        Call::Temp(body) => {
            t.call(
                "update-note",
                json!({"_id": "note:new-1", "_rev": "-", "body": body}),
            );
        }
        Call::Done => {
            t.call("kanso-task-done", json!({"summary": "done"}));
        }
        Call::List => {
            t.call("kanso-list-proposed", json!({}));
        }
    }
}

fn fixture() -> (Task, Vec<(String, String)>, String, String) {
    let t = task(
        |s| {
            let lib = library(s);
            let _ = lib.archive;
            (None, None)
        },
        TaskOps::default(),
    );
    let mut notes = t
        .store
        .notes_in_books(&[book_named(&t, "Trips")])
        .unwrap()
        .into_iter()
        .map(|n| (n.id, n.rev.unwrap()))
        .collect::<Vec<_>>();
    let (secret, secret_rev) = note(&t.store, &book_named(&t, "Archive"), "Plain", "day x");
    notes.push((secret, secret_rev));
    let trips = book_named(&t, "Trips");
    let archive = book_named(&t, "Archive");
    (t, notes, trips, archive)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Invariant: whatever tool calls an agent makes, the store is
    /// unchanged: not one write, not one new document.
    #[test]
    fn prop_staging_never_writes_the_store(calls in prop::collection::vec(call_strategy(), 1..25)) {
        let (t, notes, trips, archive) = fixture();
        let seq = t.store.last_seq().unwrap();
        let counts = t.store.counts().unwrap();
        for c in &calls {
            run(&t, &notes, &trips, &archive, c);
        }
        prop_assert_eq!(t.store.last_seq().unwrap(), seq);
        prop_assert_eq!(t.store.counts().unwrap(), counts);
        for (id, rev) in &notes {
            prop_assert_eq!(&t.store.get(id).unwrap().unwrap().rev, rev);
        }
    }

    /// Invariant: a locked note, and a note tagged #ai-exclude, is never
    /// readable by a task, never a target of an op, and never written by
    /// Apply even when a note was locked after the agent proposed.
    #[test]
    fn prop_no_task_reads_or_changes_a_locked_note(calls in prop::collection::vec(call_strategy(), 1..25), lock_after in any::<bool>()) {
        let (t, notes, trips, archive) = fixture();
        let locked = notes[0].0.clone();
        if !lock_after {
            lock(&t.store, &locked);
        }
        for c in &calls {
            run(&t, &notes, &trips, &archive, c);
        }
        let read = t.call("read-note", json!({"noteId": locked}));
        if !lock_after {
            prop_assert!(is_error(&read));
        }
        if lock_after {
            lock(&t.store, &locked);
        }
        let set = t.set();
        let before = t.store.get(&locked).unwrap().unwrap().rev;
        let _ = set.apply(&t.store, &Selection::all(set.ops.len()), 7);
        let after = t.store.get(&locked).unwrap().unwrap();
        prop_assert_eq!(&after.rev, &before);
        prop_assert!(after.fields.contains_key("kansoLock"));
        if !lock_after {
            for op in &set.ops {
                prop_assert!(!op.touched().iter().any(|(id, _)| *id == locked));
            }
        }
    }

    /// Invariant: lowering a set to a batch leaves no temporary ID: every
    /// edit names a real note, and every new note or new merge target
    /// becomes exactly one create.
    #[test]
    fn prop_to_batch_resolves_every_temp_id(calls in prop::collection::vec(call_strategy(), 1..25), mask in prop::collection::vec(any::<bool>(), 25)) {
        let (t, notes, trips, archive) = fixture();
        for c in &calls {
            run(&t, &notes, &trips, &archive, c);
        }
        let set = t.set();
        let ticked: BTreeSet<usize> = (0..set.ops.len()).filter(|i| mask[*i % mask.len()]).collect();
        let (edits, creates) = set.lower(&ticked, &BTreeSet::new(), &|_| None);
        for e in &edits {
            prop_assert!(!ChangeSet::is_temp(&e.id));
        }
        let expected = ticked.iter().filter(|i| matches!(
            &set.ops[**i],
            kanso_server::ProposedOp::Create { .. }
                | kanso_server::ProposedOp::Merge { into: kanso_server::MergeTarget::New { .. }, .. }
        )).count();
        prop_assert_eq!(creates.len(), expected);
        // Each note is edited once.
        let ids: HashSet<&String> = edits.iter().map(|e| &e.id).collect();
        prop_assert_eq!(ids.len(), edits.len());
    }

    /// Invariant: only the ticked ops reach the store.
    #[test]
    fn prop_only_ticked_ops_are_applied(calls in prop::collection::vec(call_strategy(), 1..20), mask in prop::collection::vec(any::<bool>(), 20)) {
        let (t, notes, trips, archive) = fixture();
        for c in &calls {
            run(&t, &notes, &trips, &archive, c);
        }
        let set = t.set();
        let ticked: BTreeSet<usize> = (0..set.ops.len()).filter(|i| mask[*i % mask.len()]).collect();
        let (edits, creates) = set.lower(&ticked, &BTreeSet::new(), &|_| None);
        let touched: HashSet<String> = ticked.iter()
            .flat_map(|i| set.ops[*i].touched().into_iter().map(|(id, _)| id.to_owned()))
            .collect();
        for e in &edits {
            prop_assert!(touched.contains(&e.id));
        }
        let created_before = t.store.counts().unwrap().get(&DocType::Note).copied().unwrap_or(0);
        let result = set.apply(&t.store, &Selection { ticked: ticked.clone(), apply_anyway: BTreeSet::new() }, 5);
        let created_after = t.store.counts().unwrap().get(&DocType::Note).copied().unwrap_or(0);
        if result.is_ok() {
            prop_assert_eq!(created_after - created_before, creates.len());
        } else {
            prop_assert_eq!(created_after, created_before);
        }
        // Notes no ticked op touched are as they were.
        for (id, rev) in &notes {
            if !touched.contains(id) {
                prop_assert_eq!(&t.store.get(id).unwrap().unwrap().rev, rev);
            }
        }
    }
}
