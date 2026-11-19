//! Property tests for sync over sealed accounts on a `MemoryServer`
//! (docs/ACCOUNTS.md › Tests). CI pins the seed with `PROPTEST_RNG_SEED`.
//!
//! - prop_stores_converge_and_no_edit_is_lost (model-based): three Macs of
//!   one account under random interleavings of local writes and syncs. Also
//!   checks M2 over these multi-Mac histories: every revision the server
//!   accepts for a document has a higher generation than the one it
//!   replaces, and an honest history never produces a stale document.
//! - prop_syncing_a_synced_store_changes_nothing (idempotence).
//! - prop_a_hostile_server_never_changes_local_data_wrongly: a server that
//!   flips bits, swaps sealed values between documents and accounts, forges
//!   tombstones, relabels revisions, replays old versions, rewinds, changes
//!   the epoch, answers `current: null`, sends unknown formats and another
//!   key ID, omits documents from a re-read, and answers pushes with older
//!   versions; up to three of these in one sync. Whatever the mix, the
//!   replay floor never drops and content only moves to versions above it.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_store::{Change, Origin, Store, StoreError};
use kanso_sync::wire::{WireChange, WireDoc, WirePushResult};
use kanso_sync::{MemoryServer, Route, SyncError, SyncReport, meta_keys};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

const WORDS: [&str; 6] = [
    "alpha",
    "Beta",
    "über",
    "x%y",
    "- [ ] task",
    "kanso://note/x",
];
const NAMES: [&str; 4] = ["Inbox", "Work", "Ideas", "Archive"];

fn generation(rev: &str) -> u64 {
    rev.parse::<kanso_core::Rev>().unwrap().generation
}

fn hash(rev: &str) -> String {
    rev.parse::<kanso_core::Rev>().unwrap().hash
}

/// One step of a scenario. `s` picks a Mac; `pick` picks one of that Mac's
/// live documents of the kind (modulo their count), so steps stay
/// meaningful as the scenario shrinks.
#[derive(Clone, Debug)]
enum Op {
    CreateBook {
        s: usize,
    },
    RenameBook {
        s: usize,
        pick: usize,
        name: usize,
    },
    DeleteBook {
        s: usize,
        pick: usize,
    },
    CreateTag {
        s: usize,
    },
    DeleteTag {
        s: usize,
        pick: usize,
    },
    CreateNote {
        s: usize,
        book: usize,
        tag: Option<usize>,
        word: usize,
    },
    /// `bump: false` keeps `updatedAt`, so concurrent edits tie.
    EditNote {
        s: usize,
        pick: usize,
        word: usize,
        bump: bool,
    },
    TagNote {
        s: usize,
        pick: usize,
        tag: usize,
    },
    DeleteNote {
        s: usize,
        pick: usize,
    },
    CreateFile {
        s: usize,
        bytes: Vec<u8>,
    },
    RenameFile {
        s: usize,
        pick: usize,
        name: usize,
    },
    ReplaceFile {
        s: usize,
        pick: usize,
        bytes: Vec<u8>,
    },
    DeleteFile {
        s: usize,
        pick: usize,
    },
    Sync {
        s: usize,
    },
}

fn gen_op(stores: usize) -> impl Strategy<Value = Op> {
    let s = 0..stores;
    let pick = 0..6usize;
    // A tiny alphabet so two devices sometimes produce identical bytes.
    let bytes = prop::collection::vec(0u8..3, 0..5);
    prop_oneof![
        2 => s.clone().prop_map(|s| Op::CreateBook { s }),
        1 => (s.clone(), pick.clone(), 0..NAMES.len()).prop_map(|(s, pick, name)| Op::RenameBook { s, pick, name }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteBook { s, pick }),
        1 => s.clone().prop_map(|s| Op::CreateTag { s }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteTag { s, pick }),
        4 => (s.clone(), pick.clone(), prop::option::of(pick.clone()), 0..WORDS.len())
            .prop_map(|(s, book, tag, word)| Op::CreateNote { s, book, tag, word }),
        5 => (s.clone(), pick.clone(), 0..WORDS.len(), any::<bool>())
            .prop_map(|(s, pick, word, bump)| Op::EditNote { s, pick, word, bump }),
        1 => (s.clone(), pick.clone(), pick.clone()).prop_map(|(s, pick, tag)| Op::TagNote { s, pick, tag }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteNote { s, pick }),
        2 => (s.clone(), bytes.clone()).prop_map(|(s, bytes)| Op::CreateFile { s, bytes }),
        1 => (s.clone(), pick.clone(), 0..NAMES.len()).prop_map(|(s, pick, name)| Op::RenameFile { s, pick, name }),
        1 => (s.clone(), pick.clone(), bytes).prop_map(|(s, pick, bytes)| Op::ReplaceFile { s, pick, bytes }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteFile { s, pick }),
        6 => s.prop_map(|s| Op::Sync { s }),
    ]
}

/// A local write a Mac made: which Mac, document, revision, tombstone.
type Version = (usize, String, String, bool);
/// A document version: id, revision, tombstone.
type Seen = (String, String, bool);

struct World {
    server: MemoryServer,
    macs: Vec<Mac>,
    feeds: Vec<async_channel::Receiver<Change>>,
    next_id: usize,
    clock: i64,
    local: Vec<Version>,
    /// Per Mac, every version the server handed it.
    seen: Vec<Vec<Seen>>,
    /// The latest server revision of each document, to check M2.
    accepted: BTreeMap<String, String>,
}

fn live_ids(store: &Store, prefix: &str) -> Vec<String> {
    all_docs(store)
        .into_iter()
        .filter(|(id, (_, deleted))| !deleted && id.starts_with(prefix))
        .map(|(id, _)| id)
        .collect()
}

fn choose(ids: &[String], pick: usize) -> Option<String> {
    (!ids.is_empty()).then(|| ids[pick % ids.len()].clone())
}

fn id_or_trash(id: Option<String>) -> String {
    id.unwrap_or_else(|| "trash".to_owned())
}

impl World {
    fn new(stores: usize) -> World {
        let server = MemoryServer::new();
        let mut macs = vec![Mac::new(sign_up(&server, "alice", 1))];
        for n in 1..stores {
            macs.push(Mac::new(sign_in(&server, "alice", n as u8 + 1)));
        }
        let feeds = macs.iter().map(|m| m.store.subscribe()).collect();
        World {
            server,
            macs,
            feeds,
            next_id: 0,
            clock: 0,
            local: Vec::new(),
            seen: vec![Vec::new(); stores],
            accepted: BTreeMap::new(),
        }
    }

    /// A world whose Macs already share a few documents, so random edits on
    /// different Macs often hit the same document.
    fn seeded(stores: usize) -> World {
        let mut world = World::new(stores);
        let first = &world.macs[0].store;
        put(
            first,
            DocType::Book,
            json!({"_id": "book:base1", "name": "Inbox", "createdAt": 1, "updatedAt": 1}),
        )
        .unwrap();
        put(first, DocType::Book, json!({"_id": "book:base2", "name": "Work", "parentBookId": "book:base1", "createdAt": 1, "updatedAt": 1})).unwrap();
        put(
            first,
            DocType::Tag,
            json!({"_id": "tag:base1", "name": "red", "createdAt": 1, "updatedAt": 1}),
        )
        .unwrap();
        for (i, book) in ["book:base1", "book:base2", "book:base1"]
            .iter()
            .enumerate()
        {
            put(
                first,
                DocType::Note,
                json!({"_id": format!("note:base{i}"), "bookId": book, "tags": ["tag:base1"], "title": "base",
                       "body": "- [x] seeded", "createdAt": 1, "updatedAt": 1}),
            )
            .unwrap();
        }
        put(
            first,
            DocType::File,
            file_body("file:base1", "base.png", &[1, 2, 3]),
        )
        .unwrap();
        world.run(&Op::Sync { s: 0 });
        for s in 0..stores {
            world.run(&Op::Sync { s });
        }
        world
    }

    fn fresh(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }

    fn run(&mut self, op: &Op) {
        self.clock += 1;
        let now = 1_000_000 + self.clock;
        match self.write(op, now) {
            Ok(()) | Err(StoreError::Invalid(_) | StoreError::Conflict | StoreError::NotFound) => {}
            Err(err) => panic!("{op:?}: {err}"),
        }
        for (s, feed) in self.feeds.iter().enumerate() {
            while let Ok(change) = feed.try_recv() {
                if change.origin != Origin::Sync {
                    self.local.push((s, change.id, change.rev, change.deleted));
                }
            }
        }
    }

    /// Syncs Mac `s` and checks what an honest sync must never report.
    fn sync(&mut self, s: usize) -> SyncReport {
        let view = self.server.plain_view(&self.macs[s].session);
        let before = view.docs();
        let cursor = self.macs[s].cursor();
        let report = self.macs[s].sync(&self.server);
        assert!(report.invalid.is_empty(), "invalid: {:?}", report.invalid);
        assert!(report.too_large.is_empty());
        assert_eq!(report.files_missing, 0, "a live file without bytes");
        assert!(
            report.stale.is_empty(),
            "M2: an honest history is never stale: {:?}",
            report.stale
        );
        assert!(report.skipped.is_empty() && report.repaired.is_empty());
        assert!(!report.too_new && !report.remerged);
        assert_eq!((report.quota, report.paused_until), (None, None));
        for doc in &before {
            if doc.seq > cursor || report.conflicted.contains(&doc.id) {
                self.seen[s].push((doc.id.clone(), doc.rev.clone(), doc.deleted));
            }
        }
        // M2: each accepted revision has a higher generation than the one
        // it replaced.
        for doc in view.docs() {
            if let Some(old) = self.accepted.get(&doc.id)
                && *old != doc.rev
            {
                assert!(
                    generation(&doc.rev) > generation(old),
                    "{} went from {old} to {}",
                    doc.id,
                    doc.rev
                );
            }
            self.accepted.insert(doc.id, doc.rev);
        }
        report
    }

    fn write(&mut self, op: &Op, now: i64) -> Result<(), StoreError> {
        let id = |world: &World, s: usize, prefix: &str, pick: usize| {
            choose(&live_ids(&world.macs[s].store, prefix), pick)
        };
        match op.clone() {
            Op::CreateBook { s } => {
                let id = self.fresh("book:b");
                let name = NAMES[self.next_id % NAMES.len()];
                put(
                    &self.macs[s].store,
                    DocType::Book,
                    json!({"_id": id, "name": name, "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::RenameBook { s, pick, name } => {
                if let Some(id) = id(self, s, "book:", pick) {
                    merge(
                        &self.macs[s].store,
                        DocType::Book,
                        json!({"_id": id, "name": NAMES[name], "updatedAt": now}),
                    )?;
                }
            }
            Op::DeleteBook { s, pick }
            | Op::DeleteTag { s, pick }
            | Op::DeleteNote { s, pick }
            | Op::DeleteFile { s, pick } => {
                let prefix = match op {
                    Op::DeleteBook { .. } => "book:",
                    Op::DeleteTag { .. } => "tag:",
                    Op::DeleteNote { .. } => "note:",
                    _ => "file:",
                };
                if let Some(id) = id(self, s, prefix, pick) {
                    self.macs[s].store.delete(&id, Origin::Api)?;
                }
            }
            Op::CreateTag { s } => {
                let id = self.fresh("tag:t");
                let name = format!("tag{}", self.next_id);
                put(
                    &self.macs[s].store,
                    DocType::Tag,
                    json!({"_id": id, "name": name, "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::CreateNote { s, book, tag, word } => {
                let id = self.fresh("note:n");
                let store = &self.macs[s].store;
                let book = id_or_trash(choose(&live_ids(store, "book:"), book));
                let tags: Vec<String> = tag
                    .and_then(|t| choose(&live_ids(store, "tag:"), t))
                    .into_iter()
                    .collect();
                put(
                    store,
                    DocType::Note,
                    json!({"_id": id, "bookId": book, "tags": tags, "title": WORDS[word],
                           "body": format!("{} {now}", WORDS[word]), "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::EditNote {
                s,
                pick,
                word,
                bump,
            } => {
                if let Some(id) = id(self, s, "note:", pick) {
                    let store = &self.macs[s].store;
                    let mut doc = store.get_api(&id, None, false)?;
                    doc["body"] = format!("{} {now}", WORDS[word]).into();
                    if bump {
                        doc["updatedAt"] = now.into();
                    }
                    put(store, DocType::Note, doc)?;
                }
            }
            Op::TagNote { s, pick, tag } => {
                let note = id(self, s, "note:", pick);
                let tag = id(self, s, "tag:", tag);
                if let (Some(note), Some(tag)) = (note, tag) {
                    merge(
                        &self.macs[s].store,
                        DocType::Note,
                        json!({"_id": note, "tags": [tag]}),
                    )?;
                }
            }
            Op::CreateFile { s, bytes } => {
                let id = self.fresh("file:f");
                put(
                    &self.macs[s].store,
                    DocType::File,
                    file_body(&id, "f.png", &bytes),
                )?;
            }
            Op::RenameFile { s, pick, name } => {
                if let Some(id) = id(self, s, "file:", pick) {
                    merge(
                        &self.macs[s].store,
                        DocType::File,
                        json!({"_id": id, "name": format!("{}.png", NAMES[name])}),
                    )?;
                }
            }
            Op::ReplaceFile { s, pick, bytes } => {
                if let Some(id) = id(self, s, "file:", pick) {
                    let store = &self.macs[s].store;
                    let mut doc = store.get_api(&id, None, false)?;
                    doc["_attachments"] = file_body(&id, "f.png", &bytes)["_attachments"].clone();
                    put(store, DocType::File, doc)?;
                }
            }
            Op::Sync { s } => {
                self.sync(s);
            }
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(config(128))]

    #[test]
    fn prop_stores_converge_and_no_edit_is_lost(ops in prop::collection::vec(gen_op(3), 0..40)) {
        let mut world = World::seeded(3);
        for op in &ops {
            world.run(op);
        }
        // Final round: every Mac syncs twice.
        for _ in 0..2 {
            for s in 0..world.macs.len() {
                world.run(&Op::Sync { s });
            }
        }
        let session = &world.macs[0].session;
        let server = server_docs(&world.server, session);
        let view = world.server.plain_view(session);
        prop_assert_eq!(view.unreadable(), 0);

        for (s, mac) in world.macs.iter().enumerate() {
            prop_assert_eq!(pending(&mac.store), 0, "Mac {} has pending pushes", s);
            prop_assert_eq!(&all_docs(&mac.store), &server, "Mac {} differs from the server", s);
        }
        let server_fields = server_live(&world.server, session);
        for mac in &world.macs {
            prop_assert_eq!(&live_docs(&mac.store), &server_fields);
        }
        // Same attachment bytes everywhere, and on the server, sealed.
        for id in server_fields.keys().filter(|id| id.starts_with("file:")) {
            let first = world.macs[0].store.attachment(id).unwrap();
            prop_assert!(first.is_some(), "{} has no bytes", id);
            let (content_type, blob) = first.unwrap();
            for mac in &world.macs[1..] {
                let other = mac.store.attachment(id).unwrap().unwrap();
                prop_assert_eq!(&other.0, &content_type);
                prop_assert_eq!(&other.1.data, &blob.data);
            }
            prop_assert_eq!(view.file(id), Some(blob.data.clone()));
        }
        for (id, (_, deleted)) in &server {
            if *deleted && id.starts_with("file:") {
                prop_assert!(view.file(id).is_none(), "{} kept its bytes", id);
            }
        }
        // No edit is lost: every version a Mac wrote or was handed by the
        // server (including the losing side of each conflict it resolved)
        // is still its current version or in its revision history. A
        // rebased revision (M2) keeps the content hash.
        for (s, mac) in world.macs.iter().enumerate() {
            let wrote = world.local.iter().filter(|v| v.0 == s).map(|(_, id, rev, deleted)| (id, rev, *deleted));
            let received = world.seen[s].iter().map(|(id, rev, deleted)| (id, rev, *deleted));
            for (id, rev, deleted) in wrote.chain(received) {
                if deleted {
                    continue;
                }
                let current = mac.store.get(id).unwrap().map(|d| hash(&d.rev));
                let kept = mac.store.revisions(id).unwrap().iter().any(|r| hash(&r.rev) == hash(rev));
                prop_assert!(current.as_deref() == Some(hash(rev).as_str()) || kept, "Mac {} lost {} {}", s, id, rev);
            }
        }
        // Sync never invents content: each final revision's hash was written
        // by a Mac.
        let written: BTreeSet<(&str, String)> =
            world.local.iter().map(|(_, id, rev, _)| (id.as_str(), hash(rev))).collect();
        for (id, (rev, _)) in &server {
            prop_assert!(written.contains(&(id.as_str(), hash(rev))), "{} {} came from nowhere", id, rev);
        }
        // A document only one Mac ever wrote ends at that Mac's last
        // version: with no concurrent edit there is nothing to lose to.
        let mut writers: BTreeMap<&str, (BTreeSet<usize>, &str)> = BTreeMap::new();
        for (s, id, rev, _) in &world.local {
            let entry = writers.entry(id.as_str()).or_insert_with(|| (BTreeSet::new(), rev.as_str()));
            entry.0.insert(*s);
            entry.1 = rev.as_str();
        }
        for (id, (macs_that_wrote, last)) in &writers {
            if macs_that_wrote.len() == 1 {
                prop_assert_eq!(server[*id].0.as_str(), *last, "single-writer {} was reverted", id);
            }
        }
        // And one more sync on a converged Mac does nothing.
        let pushes = world.server.stats().count(Route::Push);
        for mac in &world.macs {
            prop_assert_eq!(mac.sync(&world.server), SyncReport::default());
        }
        prop_assert_eq!(world.server.stats().count(Route::Push), pushes);
        prop_assert_eq!(server_docs(&world.server, session), server);
    }

    #[test]
    fn prop_syncing_a_synced_store_changes_nothing(ops in prop::collection::vec(gen_op(1), 0..20)) {
        let mut world = World::new(1);
        for op in &ops {
            world.run(op);
        }
        let mac = &world.macs[0];
        mac.sync(&world.server);
        // The next pull moves the cursor past the Mac's own pushes.
        mac.sync(&world.server);
        prop_assert_eq!(pending(&mac.store), 0);
        let stats = world.server.stats();
        let before = (snapshot(&mac.store), server_docs(&world.server, &mac.session));
        prop_assert_eq!(mac.sync(&world.server), SyncReport::default());
        let after = (snapshot(&mac.store), server_docs(&world.server, &mac.session));
        prop_assert_eq!(&after, &before);
        let now = world.server.stats();
        for route in [Route::Push, Route::PutFile, Route::GetFile] {
            prop_assert_eq!(now.count(route), stats.count(route), "{:?}", route);
        }
    }
}

// ---------------------------------------------------------------------------
// A hostile server
// ---------------------------------------------------------------------------

/// What the hostile server does to one of the victim's syncs.
#[derive(Clone, Debug)]
enum Attack {
    None,
    FlipBit {
        pick: usize,
        byte: usize,
        bit: u8,
    },
    SwapSealed {
        a: usize,
        b: usize,
    },
    OtherAccount {
        pick: usize,
    },
    ForgeTombstone {
        pick: usize,
    },
    Relabel {
        pick: usize,
    },
    Replay {
        pick: usize,
    },
    Rewind,
    Epoch,
    Lost {
        pick: usize,
    },
    NewFormat {
        pick: usize,
        format: u8,
    },
    WrongKey,
    /// While the victim re-reads the feed from 0 (a re-merge or full
    /// resync), leaves one document out, as a restored copy that lacks it.
    OmitInRemerge {
        pick: usize,
    },
    /// Answers a push with a conflict whose `current` is an older version
    /// of that document the account once held (`behind` the real one).
    ConflictReplay {
        pick: usize,
    },
}

fn gen_attack() -> impl Strategy<Value = Attack> {
    let pick = 0..8usize;
    prop_oneof![
        2 => Just(Attack::None),
        2 => (pick.clone(), any::<usize>(), 0u8..8).prop_map(|(pick, byte, bit)| Attack::FlipBit { pick, byte, bit }),
        1 => (pick.clone(), pick.clone()).prop_map(|(a, b)| Attack::SwapSealed { a, b }),
        1 => pick.clone().prop_map(|pick| Attack::OtherAccount { pick }),
        1 => pick.clone().prop_map(|pick| Attack::ForgeTombstone { pick }),
        1 => pick.clone().prop_map(|pick| Attack::Relabel { pick }),
        2 => pick.clone().prop_map(|pick| Attack::Replay { pick }),
        1 => Just(Attack::Rewind),
        1 => Just(Attack::Epoch),
        1 => pick.clone().prop_map(|pick| Attack::Lost { pick }),
        1 => (pick.clone(), 2u8..=255).prop_map(|(pick, format)| Attack::NewFormat { pick, format }),
        1 => Just(Attack::WrongKey),
        1 => pick.clone().prop_map(|pick| Attack::OmitInRemerge { pick }),
        2 => pick.clone().prop_map(|pick| Attack::ConflictReplay { pick }),
    ]
}

/// One to three attacks on the same sync. Page attacks apply in order to
/// every page; answer attacks (`Lost`, `ConflictReplay`) take the push
/// answers in turn, one each. Besides random mixes, the sequences the
/// review confirmed come up often: an answer lost, then an older version as
/// the next answer's `current`; and a new epoch whose re-read omits a
/// document, then an older version of it as a push answer's `current`.
fn gen_attacks() -> impl Strategy<Value = Vec<Attack>> {
    let pick = 0..8usize;
    prop_oneof![
        4 => gen_attack().prop_map(|attack| vec![attack]),
        3 => prop::collection::vec(gen_attack(), 2..=3),
        1 => (pick.clone(), pick.clone()).prop_map(|(p, q)| vec![
            Attack::Lost { pick: p },
            Attack::ConflictReplay { pick: q },
        ]),
        1 => (pick.clone(), pick.clone()).prop_map(|(p, q)| vec![
            Attack::Epoch,
            Attack::OmitInRemerge { pick: p },
            Attack::ConflictReplay { pick: q },
        ]),
    ]
}

/// Whether an attack rewrites a push answer rather than a page.
fn on_answer(attack: &Attack) -> bool {
    matches!(attack, Attack::Lost { .. } | Attack::ConflictReplay { .. })
}

/// Per document: (rev, deleted, replay floor), for the floor checks.
fn floors(store: &Store) -> BTreeMap<String, (String, bool, u64)> {
    store
        .sync_states()
        .unwrap()
        .into_iter()
        .map(|s| (s.id, (s.rev, s.deleted, s.synced_gen)))
        .collect()
}

/// Writes on the victim (A) and an honest Mac (B) of the same account, B's
/// syncs, and A's syncs through the hostile server.
#[derive(Clone, Debug)]
enum HostileOp {
    Write {
        mac: usize,
        pick: usize,
        word: usize,
    },
    Delete {
        mac: usize,
        pick: usize,
    },
    SyncB,
    SyncA(Vec<Attack>),
}

fn gen_hostile_op() -> impl Strategy<Value = HostileOp> {
    prop_oneof![
        4 => (0..2usize, 0..4usize, 0..WORDS.len()).prop_map(|(mac, pick, word)| HostileOp::Write { mac, pick, word }),
        1 => (0..2usize, 0..4usize).prop_map(|(mac, pick)| HostileOp::Delete { mac, pick }),
        2 => Just(HostileOp::SyncB),
        4 => gen_attacks().prop_map(HostileOp::SyncA),
    ]
}

/// Creates or edits `note:p<pick>` (in the trash, so no notebook is needed).
fn write_note(store: &Store, pick: usize, text: &str, now: i64) {
    let id = format!("note:p{pick}");
    let result = match store.get_api(&id, None, false) {
        Ok(mut doc) => {
            doc["body"] = text.into();
            doc["updatedAt"] = now.into();
            put(store, DocType::Note, doc)
        }
        Err(_) => put(
            store,
            DocType::Note,
            json!({"_id": id, "bookId": "trash", "title": "p", "body": text, "createdAt": now, "updatedAt": now}),
        ),
    };
    match result {
        Ok(_) | Err(StoreError::Invalid(_) | StoreError::Conflict | StoreError::NotFound) => {}
        Err(err) => panic!("{err}"),
    }
}

/// The victim's state the invariants compare: local ID -> (rev, synced_rev).
fn revs(store: &Store) -> BTreeMap<String, (String, Option<String>)> {
    store
        .sync_docs()
        .unwrap()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.base_rev)))
        .collect()
}

fn reseal(sealed: &str, f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut raw = B64.decode(sealed).unwrap_or_default();
    if raw.is_empty() {
        return sealed.to_owned();
    }
    f(&mut raw);
    B64.encode(raw)
}

proptest! {
    #![proptest_config(config(96))]

    /// Invariants: the victim's content for an ID is always some client's
    /// write for that same ID (never another account's, never a forgery);
    /// its replay floor never drops, and its content only changes to a
    /// version above the floor it had before the sync, under any mix of
    /// attacks (review-sync: a lost answer or an omitted document, then an
    /// older version, brought deleted notes back). Alone, replays of
    /// versions older than the synced one are `Stale`; rewinds and epoch
    /// changes re-merge; a newer format never overwrites and stops the pull
    /// before it; another key ID writes nothing in any mix; and after the
    /// attacks, honest syncs converge with nothing pending lost.
    #[test]
    fn prop_a_hostile_server_never_changes_local_data_wrongly(
        ops in prop::collection::vec(gen_hostile_op(), 1..30)
    ) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        let b = Mac::new(sign_in(&server, "alice", 2));
        // Another account with notes under the same local IDs.
        let z = Mac::new(sign_up(&server, "zed", 3));
        for pick in 0..4 {
            write_note(&z.store, pick, &format!("zed's secret {pick}"), 1);
        }
        z.store.delete("note:p3", Origin::Api).unwrap();
        z.sync(&server);
        let zed_rows = server.plain_view(&z.session).wire_docs();

        let view = server.plain_view(&a.session);
        let feeds = [a.store.subscribe(), b.store.subscribe()];
        // Every content hash a client of the account wrote, per local ID.
        let mut written: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        // The victim's local writes of live versions: (id, rev). A deletion
        // can lose to an edit (an edit beats a deletion), so tombstones
        // aren't counted.
        let mut a_wrote: Vec<(String, String)> = Vec::new();
        // Every row the account ever held, with its local ID and revision.
        let mut history: Vec<(WireChange, String, String)> = Vec::new();
        let mut clock = 0i64;
        let mut rounds = 0u64;

        let record = |history: &mut Vec<(WireChange, String, String)>| {
            let plain = view.docs();
            for wire in view.wire_docs() {
                if let Some(doc) = plain.iter().find(|d| d.seq == wire.seq)
                    && !history.iter().any(|(w, ..)| w.id == wire.id && w.rev == wire.rev)
                {
                    history.push((wire, doc.id.clone(), doc.rev.clone()));
                }
            }
        };

        for op in &ops {
            clock += 1;
            match op {
                HostileOp::Write { mac, pick, word } => {
                    let store = if *mac == 0 { &a.store } else { &b.store };
                    write_note(store, *pick, &format!("{} by {mac} at {clock}", WORDS[*word]), 1_000 + clock);
                }
                HostileOp::Delete { mac, pick } => {
                    let store = if *mac == 0 { &a.store } else { &b.store };
                    let _ = store.delete(&format!("note:p{pick}"), Origin::Api);
                }
                HostileOp::SyncB => {
                    b.sync(&server);
                }
                HostileOp::SyncA(attacks) => {
                    rounds += 1;
                    let before = snapshot(&a.store);
                    let revs_before = revs(&a.store);
                    let floors_before = floors(&a.store);
                    let pending_before: BTreeSet<String> =
                        a.store.pending_push(usize::MAX).unwrap().into_iter().map(|d| d.id).collect();
                    let saved_epoch = a.store.meta(meta_keys::EPOCH).unwrap();
                    let cursor = a.cursor();
                    // What an attack aimed at, for the checks below.
                    let target: Mutex<Option<(String, String, i64)>> = Mutex::new(None);
                    let mut changes_calls = 0;
                    let mut since = 0;
                    let mut answers = attacks.iter().filter(|a| on_answer(a));
                    let fake_epoch = format!("{rounds:016x}");
                    let tap = Tap::new(&server, |mut event| {
                        if let Event::Changes { since: s } = event {
                            since = s;
                            changes_calls += 1;
                        }
                        let answer = match event {
                            Event::Pushed(_) => answers.next(),
                            _ => None,
                        };
                        let pages = attacks.iter().filter(|a| !on_answer(a));
                        for attack in pages.chain(answer) {
                            match (attack, &mut event) {
                                (Attack::FlipBit { pick, byte, bit }, Event::Page(page)) if !page.docs.is_empty() => {
                                    let n = page.docs.len();
                                    let doc = &mut page.docs[pick % n];
                                    doc.sealed = reseal(&doc.sealed, |raw| {
                                        let i = byte % raw.len();
                                        raw[i] ^= 1 << bit;
                                    });
                                }
                                (Attack::SwapSealed { a, b }, Event::Page(page)) if page.docs.len() > 1 => {
                                    let n = page.docs.len();
                                    let (i, j) = (a % n, b % n);
                                    if i != j {
                                        let sealed = page.docs[i].sealed.clone();
                                        page.docs[i].sealed = std::mem::replace(&mut page.docs[j].sealed, sealed);
                                    }
                                }
                                (Attack::OtherAccount { pick }, Event::Page(page)) if !page.docs.is_empty() => {
                                    let n = page.docs.len();
                                    page.docs[pick % n].sealed = zed_rows[pick % zed_rows.len()].sealed.clone();
                                }
                                (Attack::ForgeTombstone { pick }, Event::Page(page)) if !page.docs.is_empty() => {
                                    let n = page.docs.len();
                                    page.docs[pick % n].deleted = true;
                                }
                                (Attack::Relabel { pick }, Event::Page(page)) if !page.docs.is_empty() => {
                                    let n = page.docs.len();
                                    let other = page.docs[(pick + 1) % n].rev.clone();
                                    let doc = &mut page.docs[pick % n];
                                    doc.rev = if other != doc.rev { other } else { "0".repeat(32) };
                                }
                                (Attack::Replay { pick }, Event::Page(page)) if !history.is_empty() => {
                                    let (wire, id, rev) = &history[pick % history.len()];
                                    let mut replay = wire.clone();
                                    replay.seq = page.head_seq.max(page.last_seq) + 1;
                                    page.head_seq = replay.seq;
                                    page.last_seq = replay.seq;
                                    page.more = false;
                                    *target.lock().unwrap() = Some((id.clone(), rev.clone(), replay.seq));
                                    page.docs.push(replay);
                                }
                                (Attack::Rewind, Event::Page(page)) => {
                                    if changes_calls == 1 && cursor > 0 {
                                        page.docs.clear();
                                        page.last_seq = cursor;
                                        page.head_seq = cursor - 1;
                                        page.more = false;
                                    }
                                }
                                (Attack::Epoch, Event::Page(page)) => page.epoch = fake_epoch.clone(),
                                (Attack::Lost { pick }, Event::Pushed(results)) if !results.is_empty() => {
                                    let n = results.len();
                                    let id = results[pick % n].id().to_owned();
                                    results[pick % n] = WirePushResult::Lost { id };
                                }
                                (Attack::NewFormat { pick, format }, Event::Page(page)) if !page.docs.is_empty() => {
                                    let n = page.docs.len();
                                    let doc = &mut page.docs[pick % n];
                                    doc.sealed = reseal(&doc.sealed, |raw| raw[0] = *format);
                                    if let Some((_, id, rev)) = history.iter().find(|(w, ..)| w.id == doc.id) {
                                        let mut target = target.lock().unwrap();
                                        if target.is_none() {
                                            *target = Some((id.clone(), rev.clone(), doc.seq));
                                        }
                                    }
                                }
                                (Attack::WrongKey, Event::Page(page)) => page.key_id = "00000000000000ff".into(),
                                (Attack::OmitInRemerge { pick }, Event::Page(page))
                                    if since == 0 && cursor > 0 && !page.docs.is_empty() =>
                                {
                                    let n = page.docs.len();
                                    page.docs.remove(pick % n);
                                    page.last_seq = page.docs.last().map_or(since, |d| d.seq);
                                    page.more = page.more && !page.docs.is_empty();
                                }
                                (Attack::ConflictReplay { pick }, Event::Pushed(results)) if !results.is_empty() => {
                                    let n = results.len();
                                    let id = results[pick % n].id().to_owned();
                                    let older: Vec<&WireChange> =
                                        history.iter().filter(|(w, ..)| w.id == id).map(|(w, ..)| w).collect();
                                    if !older.is_empty() {
                                        let old = older[pick % older.len()];
                                        results[pick % n] = WirePushResult::Conflict {
                                            id: id.clone(),
                                            current: WireDoc {
                                                id,
                                                rev: old.rev.clone(),
                                                deleted: old.deleted,
                                                sealed: old.sealed.clone(),
                                            },
                                        };
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(())
                    });
                    let result = a.try_sync(&tap);
                    drop(tap);

                    // Whatever the attacks, the replay floor never drops, and
                    // a document's content only changes to a version above
                    // the floor it had: a replay never reverts. The one
                    // documented exception is a live version this Mac never
                    // held over its own deletion (an edit beats a deletion);
                    // that is `RemoteWon`, flagged as a conflict.
                    let floors_after = floors(&a.store);
                    for (id, (rev, deleted, floor)) in &floors_before {
                        let Some((after_rev, after_deleted, after_floor)) = floors_after.get(id) else {
                            prop_assert!(false, "{} vanished", id);
                            continue;
                        };
                        prop_assert!(after_floor >= floor, "{}'s floor fell from {} to {}", id, floor, after_floor);
                        if hash(after_rev) != hash(rev) || after_deleted != deleted {
                            let resurrected = *deleted
                                && !after_deleted
                                && matches!(&result, Ok(report) if report.conflicted.iter().any(|c| c == id));
                            prop_assert!(
                                resurrected || generation(after_rev) > *floor,
                                "{} went from {} to {}, not above its floor {}", id, rev, after_rev, floor
                            );
                        }
                    }

                    let wrong_key = attacks.iter().any(|a| matches!(a, Attack::WrongKey));
                    if wrong_key {
                        prop_assert!(matches!(result, Err(SyncError::WrongKey)), "{:?}", result);
                        prop_assert_eq!(&snapshot(&a.store), &before, "a wrong key ID wrote something");
                    } else if let [attack] = attacks.as_slice() {
                        match attack {
                            Attack::Rewind if cursor > 0 => {
                                let report = result.expect("sync");
                                prop_assert!(report.remerged, "a rewind must re-merge");
                            }
                            Attack::Epoch if saved_epoch.is_some() => {
                                let report = result.expect("sync");
                                prop_assert!(report.remerged, "a new epoch must re-merge");
                            }
                            Attack::Replay { .. } => {
                                let report = result.expect("sync");
                                // A replay of a version older than the one this
                                // Mac synced never reverts it: it is stale.
                                if let Some((id, replayed, _)) = target.lock().unwrap().clone()
                                    && let Some((_, Some(synced))) = revs_before.get(&id)
                                    && replayed != *synced
                                    && generation(&replayed) <= generation(synced)
                                {
                                    prop_assert!(report.stale.contains(&id), "{} replayed at {} over {}: {:?}", id, replayed, synced, report);
                                }
                            }
                            Attack::NewFormat { .. } => {
                                if let Ok(report) = &result
                                    && let Some((id, _, seq)) = target.lock().unwrap().clone()
                                    && seq > cursor
                                {
                                    prop_assert!(report.too_new);
                                    prop_assert!(a.cursor() < seq, "the pull went past a newer format");
                                    if !pending_before.contains(&id) {
                                        let after = revs(&a.store);
                                        prop_assert_eq!(after.get(&id), revs_before.get(&id), "a newer format overwrote {}", id);
                                    }
                                }
                            }
                            _ => {
                                // Undecryptable rows are skipped or repaired, a
                                // lost answer pushed again; none of it fails.
                                if let Err(err) = &result {
                                    prop_assert!(false, "{:?}: {}", attack, err);
                                }
                            }
                        }
                    } else if let Err(err) = &result {
                        // Combined, the attacks may only make a page the
                        // client rejects as malformed.
                        prop_assert!(
                            matches!(err, SyncError::Api(kanso_sync::ApiError::Protocol(_))),
                            "{:?}: {}", attacks, err
                        );
                    }
                }
            }
            for (m, feed) in feeds.iter().enumerate() {
                while let Ok(change) = feed.try_recv() {
                    if change.origin != Origin::Sync {
                        written.entry(change.id.clone()).or_default().insert(hash(&change.rev));
                        if m == 0 && !change.deleted {
                            a_wrote.push((change.id, change.rev));
                        }
                    }
                }
            }
            // The victim's content for an ID is always a client's write for it.
            for doc in a.store.sync_docs().unwrap() {
                prop_assert!(
                    written.get(&doc.id).is_some_and(|w| w.contains(&hash(&doc.rev))),
                    "{} holds {}, which no client wrote for it", doc.id, doc.rev
                );
            }
            record(&mut history);
        }

        // Honest syncs converge, with nothing pending lost. A hostile replay
        // may have moved the victim's cursor past the head; a full resync
        // (`kanso-sync sync --full`) recovers.
        a.store.set_meta(meta_keys::LAST_SEQ, "0").unwrap();
        for _ in 0..3 {
            a.sync(&server);
            b.sync(&server);
        }
        prop_assert_eq!(pending(&a.store), 0);
        prop_assert_eq!(pending(&b.store), 0);
        let on_server = server_docs(&server, &a.session);
        prop_assert_eq!(&all_docs(&a.store), &on_server);
        prop_assert_eq!(&all_docs(&b.store), &on_server);
        prop_assert_eq!(live_docs(&a.store), live_docs(&b.store));
        for (id, rev) in &a_wrote {
            let current = a.store.get(id).unwrap().map(|d| hash(&d.rev));
            let kept = a.store.revisions(id).unwrap().iter().any(|r| hash(&r.rev) == hash(rev));
            prop_assert!(current.as_deref() == Some(hash(rev).as_str()) || kept, "A lost {} {}", id, rev);
        }
        // Nothing of the other account ever reached this one.
        prop_assert_eq!(server.plain_view(&z.session).docs().len(), 4);
    }
}
