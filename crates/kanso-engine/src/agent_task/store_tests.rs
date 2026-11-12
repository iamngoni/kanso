//! Agent sessions on disk: what is saved is what is loaded, files that can't
//! be used are skipped, no secret is written, and the size cap trims the
//! oldest thoughts and tool calls first.

use std::collections::BTreeSet;
use std::fs;
use std::sync::Arc;

use kanso_server::{
    AgentScope, ChangeSet, Counts, MergeTarget, ProposedOp, TaskId, TaskLimits, TaskOps,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::Value;

use super::model::Phase;
use super::session::{
    Item, PermissionAnswer, RoundState, SavedSession, SessionKey, ToolKind, ToolStatus,
};
use super::store::{
    MAX_BYTES, TRIMMED_NOTICE, VERSION, count, delete, delete_all, load_all, save, trimmed,
};
use super::{ReadScope, TaskSpec};

fn is_thought(item: &Item) -> bool {
    matches!(item, Item::Thought { .. })
}

fn is_tool(item: &Item) -> bool {
    matches!(item, Item::Tool { .. })
}

fn text() -> impl Strategy<Value = String> {
    "\\PC{0,24}"
}

fn kind() -> impl Strategy<Value = ToolKind> {
    prop::sample::select(vec![
        ToolKind::Read,
        ToolKind::Edit,
        ToolKind::Search,
        ToolKind::Execute,
        ToolKind::Other,
    ])
}

fn item() -> impl Strategy<Value = Item> {
    prop_oneof![
        (text(), any::<i64>()).prop_map(|(text, at_ms)| Item::User { text, at_ms }),
        (text(), any::<i64>()).prop_map(|(text, at_ms)| Item::Agent { text, at_ms }),
        (text(), any::<u64>()).prop_map(|(text, ms)| Item::Thought { text, ms }),
        (text(), text(), kind()).prop_map(|(id, title, kind)| Item::Tool {
            id,
            title,
            kind,
            status: ToolStatus::Completed,
        }),
        (text(), kind()).prop_map(|(title, kind)| Item::Permission {
            title,
            kind,
            answer: PermissionAnswer::AllowedForSession,
        }),
        (any::<u32>(), 0usize..9).prop_map(|(round, edits)| Item::Proposed {
            round,
            counts: Counts {
                edits,
                ..Counts::default()
            },
            state: RoundState::ToReview,
        }),
        (text(), any::<i64>()).prop_map(|(text, at_ms)| Item::Notice { text, at_ms }),
    ]
}

fn op() -> impl Strategy<Value = ProposedOp> {
    prop_oneof![
        (text(), text(), text()).prop_map(|(temp, title, body)| ProposedOp::Create {
            temp,
            book_id: "book:a".into(),
            title,
            body,
            sources: vec!["note:1".into()],
        }),
        (text(), text()).prop_map(|(old_body, new_body)| ProposedOp::Edit {
            note_id: "note:2".into(),
            base_rev: "1-a".into(),
            old_title: "Old".into(),
            old_body,
            new_title: "New".into(),
            new_body,
        }),
        text().prop_map(|to_book| ProposedOp::Move {
            note_id: "note:3".into(),
            base_rev: "1-b".into(),
            from_book: "book:a".into(),
            to_book,
        }),
        text().prop_map(|body| ProposedOp::Merge {
            into: MergeTarget::New {
                temp: "note:new-1".into(),
                book_id: "book:a".into(),
            },
            sources: vec![
                ("note:4".into(), "1-c".into()),
                ("note:5".into(), "1-d".into())
            ],
            title: "Merged".into(),
            body,
        }),
    ]
}

fn scope() -> Arc<AgentScope> {
    Arc::new(AgentScope::task(
        None,
        None,
        "Claude Agent",
        TaskId(3),
        TaskOps::default(),
        TaskLimits {
            lifetime: std::time::Duration::from_secs(60),
            calls: 10,
            max_notes: 10,
        },
    ))
}

fn pending(ops: Vec<ProposedOp>) -> ChangeSet {
    let mut set = ChangeSet::new(TaskId(3), "goal", "Claude Agent", scope(), BTreeSet::new());
    set.summary = "all done".into();
    set.done = true;
    set.ops = ops;
    set
}

fn session(key: &str, items: Vec<Item>, pending: Option<ChangeSet>) -> SavedSession {
    let mut spec = TaskSpec::new(
        "Tidy the trips",
        ReadScope::Notes(vec!["n1".into()]),
        "book:a",
    );
    spec.start_note = Some("n1".into());
    SavedSession {
        version: VERSION,
        key: SessionKey(key.to_owned()),
        title: "Tidy the trips".into(),
        agent: "Claude Agent".into(),
        agent_key: "claude-agent".into(),
        acp_session_id: Some("acp-1".into()),
        spec,
        created_ms: 10,
        last_activity_ms: 20,
        phase: Phase::Ready,
        round: 2,
        items,
        pending,
        continued_from: Some(SessionKey("from".into())),
        continued_in: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: RngSeed::Fixed(0x0073_6573_7331),
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// What is saved is what is loaded, the unreviewed proposal included.
    #[test]
    fn a_saved_session_loads_back_the_same(
        items in prop::collection::vec(item(), 0..12),
        ops in prop::collection::vec(op(), 0..5),
        round in 1u32..9,
        phase in 0u8..3,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let set = (!ops.is_empty()).then(|| {
            // Ops from earlier rounds keep their label across the file.
            let mut set = pending(ops);
            set.begin_round(1);
            set.begin_round(round);
            set
        });
        let mut saved = session("01ARZ3NDEKTSV4RRFFQ69G5FAV", items, set);
        saved.round = round;
        saved.phase = match phase {
            0 => Phase::Idle,
            1 => Phase::Ready,
            _ => Phase::Failed { message: "boom".into() },
        };
        save(dir.path(), &saved).unwrap();
        let loaded = load_all(dir.path());
        prop_assert_eq!(loaded.len(), 1);
        prop_assert_eq!(
            serde_json::to_value(&loaded[0]).unwrap(),
            serde_json::to_value(&saved).unwrap()
        );
        prop_assert_eq!(&loaded[0].items, &saved.items);
        prop_assert_eq!(&loaded[0].phase, &saved.phase);
        prop_assert_eq!(
            loaded[0].pending.as_ref().map(|set| (set.round, set.rounds())),
            saved.pending.as_ref().map(|set| (set.round, set.rounds()))
        );
        prop_assert_eq!(&loaded[0].spec, &saved.spec);
        prop_assert_eq!(
            loaded[0].pending.as_ref().map(|set| set.ops.clone()),
            saved.pending.as_ref().map(|set| set.ops.clone())
        );
    }
}

#[test]
fn a_proposal_read_back_sees_nothing_until_given_a_scope() {
    let dir = tempfile::tempdir().unwrap();
    let saved = session("A1", vec![], Some(pending(vec![])));
    save(dir.path(), &saved).unwrap();
    let set = load_all(dir.path()).remove(0).pending.unwrap();
    assert_eq!(set.scope.notebooks, Some(Default::default()));
    let set = set.with_scope(TaskId(9), scope());
    assert_eq!(set.task, TaskId(9));
    assert!(set.scope.notebooks.is_none());
}

#[test]
fn files_that_cannot_be_used_are_skipped_not_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let good = session("GOOD1", vec![], None);
    save(dir.path(), &good).unwrap();
    fs::write(dir.path().join("CORRUPT.json"), b"{ not json").unwrap();
    fs::write(dir.path().join("EMPTY.json"), b"").unwrap();
    let mut old = serde_json::to_value(session("OLD1", vec![], None)).unwrap();
    old["version"] = 0.into();
    fs::write(dir.path().join("OLD1.json"), old.to_string()).unwrap();
    let mut newer = serde_json::to_value(session("NEW1", vec![], None)).unwrap();
    newer["version"] = 2.into();
    fs::write(dir.path().join("NEW1.json"), newer.to_string()).unwrap();
    let mut no_version = serde_json::to_value(session("NOVER", vec![], None)).unwrap();
    no_version.as_object_mut().unwrap().remove("version");
    fs::write(dir.path().join("NOVER.json"), no_version.to_string()).unwrap();
    let other = serde_json::to_value(session("ELSE", vec![], None)).unwrap();
    fs::write(dir.path().join("MISNAMED.json"), other.to_string()).unwrap();
    fs::write(dir.path().join("notes.txt"), b"hi").unwrap();

    let loaded = load_all(dir.path());
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].key, good.key);
    assert!(dir.path().join("CORRUPT.json").exists());
    assert!(dir.path().join("OLD1.json").exists());
    assert!(load_all(&dir.path().join("missing")).is_empty());
}

#[test]
fn a_session_key_cannot_name_a_path() {
    let dir = tempfile::tempdir().unwrap();
    let bad = session("a/b", vec![], None);
    assert!(save(dir.path(), &bad).is_err());
    assert!(delete(dir.path(), &bad.key).is_ok());
    assert_eq!(count(dir.path()), 0);
}

#[test]
fn no_secret_reaches_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let saved = session("SECRET1", vec![], Some(pending(vec![])));
    save(dir.path(), &saved).unwrap();
    let text = fs::read_to_string(dir.path().join("SECRET1.json")).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    fn walk(value: &Value, keys: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, inner) in map {
                    keys.insert(key.to_lowercase());
                    walk(inner, keys);
                }
            }
            Value::Array(list) => list.iter().for_each(|inner| walk(inner, keys)),
            _ => {}
        }
    }
    let mut keys = BTreeSet::new();
    walk(&value, &mut keys);
    for key in &keys {
        for word in ["token", "secret", "api_key", "apikey", "password", "scope"] {
            assert!(!key.contains(word), "a field called {key}");
        }
    }
}

fn thought(n: usize) -> Item {
    Item::Thought {
        text: format!("{n}:{}", "t".repeat(40_000)),
        ms: 1,
    }
}

fn tool(n: usize) -> Item {
    Item::Tool {
        id: format!("t{n}"),
        title: "x".repeat(40_000),
        kind: ToolKind::Read,
        status: ToolStatus::Completed,
    }
}

fn user(text: &str) -> Item {
    Item::User {
        text: text.into(),
        at_ms: 1,
    }
}

#[test]
fn a_session_over_the_cap_loses_old_thoughts_before_tools() {
    let dir = tempfile::tempdir().unwrap();
    let mut items = vec![user("goal")];
    items.extend((0..30).map(thought)); // about 1.2 MB
    items.extend((0..30).map(tool)); // about 1.2 MB
    items.push(Item::Agent {
        text: "the answer".into(),
        at_ms: 2,
    });
    let saved = session("BIG1", items, Some(pending(vec![])));
    save(dir.path(), &saved).unwrap();
    let size = fs::metadata(dir.path().join("BIG1.json")).unwrap().len() as usize;
    assert!(size <= MAX_BYTES, "{size}");
    let loaded = load_all(dir.path()).remove(0);
    assert!(matches!(loaded.items[0], Item::User { .. }));
    assert!(matches!(&loaded.items[1], Item::Notice { text, .. } if text == TRIMMED_NOTICE));
    assert!(
        loaded
            .items
            .iter()
            .any(|item| matches!(item, Item::Agent { text, .. } if text == "the answer"))
    );
    assert!(loaded.pending.is_some());
    // Dropping thoughts was enough, so every tool call is still there, and
    // the thoughts that stay are the newest.
    let thoughts = loaded.items.iter().filter(|i| is_thought(i)).count();
    assert!(thoughts < 30);
    assert_eq!(loaded.items.iter().filter(|i| is_tool(i)).count(), 30);
    assert!(matches!(
        loaded.items.iter().rev().find(|i| is_thought(i)),
        Some(Item::Thought { text, .. }) if text.starts_with("29:")
    ));
}

#[test]
fn tools_go_when_thoughts_are_not_enough_and_a_small_session_is_untouched() {
    let mut items = vec![user("goal")];
    items.extend((0..3).map(thought));
    items.extend((0..60).map(tool)); // about 2.4 MB
    let big = session("BIG2", items, None);
    let once = trimmed(&big);
    assert!(serde_json::to_vec(&once).unwrap().len() <= MAX_BYTES);
    assert_eq!(once.items.iter().filter(|i| is_thought(i)).count(), 0);
    assert!(once.items.iter().filter(|i| is_tool(i)).count() < 60);
    // The newest tool call is the one kept.
    assert!(matches!(
        once.items.iter().rev().find(|i| is_tool(i)),
        Some(Item::Tool { id, .. }) if id == "t59"
    ));
    // Noticed once, however often it is trimmed.
    let twice = trimmed(&once);
    let notices = twice
        .items
        .iter()
        .filter(|i| matches!(i, Item::Notice { text, .. } if text == TRIMMED_NOTICE))
        .count();
    assert_eq!(notices, 1);

    let small = session("SMALL", vec![user("hi"), tool(1)], None);
    assert_eq!(trimmed(&small).items, small.items);
}

#[test]
fn delete_and_delete_all_remove_files() {
    let dir = tempfile::tempdir().unwrap();
    for key in ["A1", "B2", "C3"] {
        save(dir.path(), &session(key, vec![], None)).unwrap();
    }
    assert_eq!(count(dir.path()), 3);
    delete(dir.path(), &SessionKey("B2".into())).unwrap();
    delete(dir.path(), &SessionKey("B2".into())).unwrap();
    assert_eq!(count(dir.path()), 2);
    assert_eq!(delete_all(dir.path()), 2);
    assert_eq!(count(dir.path()), 0);
}

#[test]
fn saving_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), &session("A1", vec![], None)).unwrap();
    save(dir.path(), &session("A1", vec![user("again")], None)).unwrap();
    let names: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["A1.json".to_owned()]);
}
