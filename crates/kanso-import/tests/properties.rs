//! Property tests for the PouchDB library importer. CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use common::{Fixture, Revision, attachment_stub, binary_string};
use kanso_core::model::SUPPORTED_IMAGE_TYPES;
use kanso_core::tasks::count_tasks;
use kanso_core::{DocType, TagColor};
use kanso_import::pouch::{decode_binary_string, encode_binary_string, escape_blob, unescape_blob};
use kanso_import::{ImportReport, import};
use kanso_store::{Attachment, Store};
use proptest::prelude::*;
use proptest::sample::Index;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Generators: a random PouchDB library with revisions, deletions and attachments.

#[derive(Clone, Debug)]
struct GenRev {
    fields: Map<String, Value>,
    deleted: bool,
    hash: u128,
    /// Attachment bytes of a file revision.
    bytes: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct GenDoc {
    id: String,
    /// `None` for `activity:` documents.
    doc_type: Option<DocType>,
    revisions: Vec<GenRev>,
    winner: usize,
}

impl GenDoc {
    fn rev_string(&self, index: usize) -> String {
        format!("{}-{:032x}", index + 1, self.revisions[index].hash)
    }

    fn winning(&self) -> &GenRev {
        &self.revisions[self.winner]
    }

    fn winning_rev(&self) -> String {
        self.rev_string(self.winner)
    }

    fn is_live(&self) -> bool {
        !self.winning().deleted
    }

    fn fixture_revisions(&self) -> Vec<Revision> {
        (0..self.revisions.len())
            .map(|i| {
                let r = &self.revisions[i];
                if r.deleted {
                    Revision::deleted(&self.rev_string(i))
                } else {
                    Revision {
                        rev: self.rev_string(i),
                        fields: r.fields.clone(),
                        deleted: false,
                    }
                }
            })
            .collect()
    }

    /// The fields the store should hold: the winning revision's, minus the
    /// attachment stub and the derived `count` that Kanso recomputes on read.
    fn expected_fields(&self) -> Map<String, Value> {
        let mut fields = self.winning().fields.clone();
        fields.remove("_attachments");
        if matches!(self.doc_type, Some(DocType::Book | DocType::Tag)) {
            fields.remove("count");
        }
        fields
    }

    fn digest(&self) -> Option<String> {
        self.winning()
            .bytes
            .as_ref()
            .map(|b| Attachment::new(b.clone()).digest())
    }
}

#[derive(Clone, Debug)]
struct Corpus {
    docs: Vec<GenDoc>,
}

impl Corpus {
    fn live(&self, doc_type: Option<DocType>) -> impl Iterator<Item = &GenDoc> {
        self.docs
            .iter()
            .filter(move |d| d.doc_type == doc_type && d.is_live())
    }

    fn write(&self, path: &Path) -> Fixture {
        let fixture = Fixture::create(path);
        fixture.put_doc(
            "_design/index_notes",
            &[Revision::live("1-aa", json!({"views": {}}))],
            0,
        );
        fixture.put_doc("config", &[Revision::live("3-bb", json!({"core": {}}))], 0);
        for doc in &self.docs {
            fixture.put_doc(&doc.id, &doc.fixture_revisions(), doc.winner);
            for revision in &doc.revisions {
                if let Some(bytes) = &revision.bytes {
                    fixture.put_attachment(bytes);
                }
            }
        }
        fixture
    }
}

fn gen_name() -> impl Strategy<Value = String> {
    "[A-Za-z0-9ü]([A-Za-z0-9 _.ü-]{0,20}[A-Za-z0-9])?"
}

fn gen_time() -> impl Strategy<Value = i64> {
    0i64..4_000_000_000_000
}

fn gen_text(max: usize) -> impl Strategy<Value = String> {
    proptest::string::string_regex(&format!(".{{0,{max}}}")).unwrap()
}

fn suffix(id: &str) -> String {
    id.split_once(':').unwrap().1.to_owned()
}

fn gen_body(note_ids: Vec<String>, file_ids: Vec<String>) -> impl Strategy<Value = String> {
    let mut fragments: Vec<BoxedStrategy<String>> = vec![
        gen_text(30).boxed(),
        Just("\n".to_owned()).boxed(),
        Just("- [ ] task\n".to_owned()).boxed(),
        Just("* [x] done\n".to_owned()).boxed(),
        Just("```\n- [ ] in code\n```\n".to_owned()).boxed(),
        Just("[gone](kanso://note/missing-1)".to_owned()).boxed(),
    ];
    if !note_ids.is_empty() {
        fragments.push(
            prop::sample::select(note_ids)
                .prop_map(|id| format!("[n](kanso://note/{})", suffix(&id)))
                .boxed(),
        );
    }
    if !file_ids.is_empty() {
        fragments.push(
            prop::sample::select(file_ids)
                .prop_map(|id| format!("![f](kanso://file:{})", suffix(&id)))
                .boxed(),
        );
    }
    prop::collection::vec(prop::strategy::Union::new(fragments), 0..8)
        .prop_map(|parts| parts.concat())
}

type Fields = (Map<String, Value>, Option<Vec<u8>>);

fn gen_book_fields(parents: Vec<String>) -> BoxedStrategy<Fields> {
    let parent = if parents.is_empty() {
        Just(None).boxed()
    } else {
        prop::option::of(prop::sample::select(parents)).boxed()
    };
    (
        gen_name(),
        parent,
        prop::option::of(0i64..100),
        gen_time(),
        gen_time(),
        0u32..50,
    )
        .prop_map(|(name, parent, order, created, updated, count)| {
            let mut f = common::object(json!({
                "name": name, "parentBookId": parent, "createdAt": created,
                "updatedAt": updated, "count": count,
            }));
            if let Some(order) = order {
                f.insert("order".into(), order.into());
            }
            (f, None)
        })
        .boxed()
}

fn gen_tag_fields() -> BoxedStrategy<Fields> {
    (
        gen_name(),
        prop::sample::select(TagColor::ALL.to_vec()),
        gen_time(),
        gen_time(),
        0u32..50,
    )
        .prop_map(|(name, color, created, updated, count)| {
            let f = common::object(json!({
                "name": name, "color": color, "createdAt": created,
                "updatedAt": updated, "count": count,
            }));
            (f, None)
        })
        .boxed()
}

fn gen_file_fields(note_ids: Vec<String>) -> BoxedStrategy<Fields> {
    let public_in = if note_ids.is_empty() {
        Just(Vec::new()).boxed()
    } else {
        prop::sample::subsequence(note_ids.clone(), 0..=note_ids.len().min(2)).boxed()
    };
    (
        "[a-z0-9_-]{1,12}\\.(png|jpg|gif)",
        prop::sample::select(SUPPORTED_IMAGE_TYPES.to_vec()),
        prop::collection::vec(any::<u8>(), 0..300),
        public_in,
        any::<bool>(),
        gen_time(),
    )
        .prop_map(|(name, content_type, bytes, public_in, md5, created)| {
            let mut f = common::object(json!({
                "name": name, "contentType": content_type, "contentLength": bytes.len(),
                "publicIn": public_in, "createdAt": created,
                "_attachments": attachment_stub(&bytes, content_type),
            }));
            if md5 {
                f.insert(
                    "md5digest".into(),
                    Attachment::new(bytes.clone()).md5_hex().into(),
                );
            }
            (f, Some(bytes))
        })
        .boxed()
}

fn gen_note_fields(
    book_ids: Vec<String>,
    tag_ids: Vec<String>,
    note_ids: Vec<String>,
    file_ids: Vec<String>,
) -> BoxedStrategy<Fields> {
    let mut books = book_ids;
    books.push("trash".to_owned());
    let tags = if tag_ids.is_empty() {
        Just(Vec::new()).boxed()
    } else {
        prop::sample::subsequence(tag_ids.clone(), 0..=tag_ids.len()).boxed()
    };
    (
        (
            gen_text(40),
            gen_body(note_ids, file_ids),
            prop::sample::select(books),
            tags,
        ),
        (
            prop::sample::select(vec!["none", "active", "onHold", "completed", "dropped"]),
            prop::sample::select(vec!["private", "public"]),
            any::<bool>(),
        ),
        (gen_time(), gen_time(), gen_time()),
        prop::option::of("note:[a-zA-Z0-9]{8}"),
    )
        .prop_map(
            |(
                (title, body, book, tags),
                (status, share, pinned),
                (created, updated, stamp),
                template,
            )| {
                let tasks = count_tasks(&body);
                let mut f = common::object(json!({
                    "doctype": "markdown", "bookId": book, "tags": tags,
                    "updatedAt": updated, "createdAt": created, "status": status,
                    "share": share, "pinned": pinned, "timestamp": stamp,
                    "title": title, "body": body,
                    "numOfTasks": tasks.total, "numOfCheckedTasks": tasks.checked,
                }));
                if let Some(template) = template {
                    f.insert("sourceTemplateId".into(), template.into());
                }
                (f, None)
            },
        )
        .boxed()
}

fn gen_activity_fields(note_ids: Vec<String>) -> BoxedStrategy<Fields> {
    let keys = if note_ids.is_empty() {
        Just("note:none".to_owned()).boxed()
    } else {
        prop::sample::select(note_ids).boxed()
    };
    (
        prop::collection::btree_map(
            keys,
            prop::collection::vec(("[a-z]{1,8}", gen_time()), 0..3),
            0..3,
        ),
        gen_time(),
    )
        .prop_map(|(items, created)| {
            let items: BTreeMap<String, Value> = items
                .into_iter()
                .map(|(note, actions)| {
                    let actions: Vec<Value> = actions
                        .into_iter()
                        .map(|(action, at)| json!({"action": action, "at": at}))
                        .collect();
                    (note, Value::Array(actions))
                })
                .collect();
            (
                common::object(json!({"items": items, "createdAt": created, "updatedAt": created})),
                None,
            )
        })
        .boxed()
}

/// A document with 1–3 revisions, any of which may be the winner and any of
/// which may be a deletion.
fn gen_doc(
    id: String,
    doc_type: Option<DocType>,
    fields: BoxedStrategy<Fields>,
) -> BoxedStrategy<GenDoc> {
    (
        prop::collection::vec((fields, prop::bool::weighted(0.2), any::<u128>()), 1..=3),
        any::<Index>(),
    )
        .prop_map(move |(revisions, winner)| GenDoc {
            id: id.clone(),
            doc_type,
            winner: winner.index(revisions.len()),
            revisions: revisions
                .into_iter()
                .map(|((fields, bytes), deleted, hash)| GenRev {
                    fields,
                    deleted,
                    hash,
                    bytes,
                })
                .collect(),
        })
        .boxed()
}

fn gen_corpus(files: std::ops::Range<usize>) -> impl Strategy<Value = Corpus> {
    (1..4usize, 0..4usize, files, 0..7usize, 0..3usize).prop_flat_map(|(nb, nt, nf, nn, na)| {
        let book_ids: Vec<String> = (0..nb)
            .map(|i| {
                if i == 0 {
                    "book:inbox".to_owned()
                } else {
                    format!("book:b{i}-x_Y")
                }
            })
            .collect();
        let tag_ids: Vec<String> = (0..nt).map(|i| format!("tag:t{i}")).collect();
        let file_ids: Vec<String> = (0..nf).map(|i| format!("file:f{i}_Z-")).collect();
        let note_ids: Vec<String> = (0..nn).map(|i| format!("note:n{i}-Ab_c")).collect();

        let mut docs: Vec<BoxedStrategy<GenDoc>> = Vec::new();
        for (i, id) in book_ids.iter().enumerate() {
            docs.push(gen_doc(
                id.clone(),
                Some(DocType::Book),
                gen_book_fields(book_ids[..i].to_vec()),
            ));
        }
        for id in &tag_ids {
            docs.push(gen_doc(id.clone(), Some(DocType::Tag), gen_tag_fields()));
        }
        for id in &file_ids {
            docs.push(gen_doc(
                id.clone(),
                Some(DocType::File),
                gen_file_fields(note_ids.clone()),
            ));
        }
        for id in &note_ids {
            docs.push(gen_doc(
                id.clone(),
                Some(DocType::Note),
                gen_note_fields(
                    book_ids.clone(),
                    tag_ids.clone(),
                    note_ids.clone(),
                    file_ids.clone(),
                ),
            ));
        }
        for i in 0..na {
            docs.push(gen_doc(
                format!("activity:a{i}"),
                None,
                gen_activity_fields(note_ids.clone()),
            ));
        }
        docs.prop_map(|docs| Corpus { docs })
    })
}

// ---------------------------------------------------------------------------
// Helpers.

struct Run {
    _dir: tempfile::TempDir,
    source: std::path::PathBuf,
    fixture: Fixture,
    store: Store,
}

fn setup(corpus: &Corpus) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("0123456789abcdef0123456789abcdef.sqlite");
    let fixture = corpus.write(&source);
    let store = Store::open(dir.path().join("data/kanso.db")).unwrap();
    Run {
        _dir: dir,
        source,
        fixture,
        store,
    }
}

fn run_import(run: &Run) -> ImportReport {
    import(&run.source, &run.store, |_| {}).unwrap()
}

fn archive(store: &Store) -> HashMap<String, Value> {
    let conn = rusqlite::Connection::open(store.path()).unwrap();
    let mut stmt = conn.prepare("SELECT id, json FROM archive").unwrap();
    stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })
    .unwrap()
    .map(|row| {
        let (id, json) = row.unwrap();
        (id, serde_json::from_str(&json).unwrap())
    })
    .collect()
}

fn blobs(store: &Store) -> BTreeMap<String, Vec<u8>> {
    let conn = rusqlite::Connection::open(store.path()).unwrap();
    let mut stmt = conn.prepare("SELECT file_id, data FROM blobs").unwrap();
    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn mismatch_ids(report: &ImportReport) -> BTreeSet<String> {
    report
        .attachment_mismatches
        .iter()
        .map(|line| line.split_once(": ").unwrap().0.to_owned())
        .collect()
}

#[derive(Clone, Debug)]
enum Corruption {
    /// Changes one character's byte value.
    Flip(Index),
    /// Appends a byte, changing length and digest.
    Append(u8),
    /// Inserts a character above U+00FF, which no binary string may hold.
    Wide(Index),
}

fn gen_corruption() -> impl Strategy<Value = Corruption> {
    prop_oneof![
        any::<Index>().prop_map(Corruption::Flip),
        any::<u8>().prop_map(Corruption::Append),
        any::<Index>().prop_map(Corruption::Wide),
    ]
}

fn corrupt(body: &str, corruption: &Corruption) -> String {
    let mut chars: Vec<char> = body.chars().collect();
    match corruption {
        Corruption::Flip(at) if !chars.is_empty() => {
            let i = at.index(chars.len());
            chars[i] = char::from_u32(u32::from(chars[i]) ^ 0x5a).unwrap();
        }
        Corruption::Flip(_) => chars.push('x'),
        Corruption::Append(byte) => chars.push(char::from(*byte)),
        Corruption::Wide(at) => {
            let i = at.index(chars.len() + 1);
            chars.insert(i, '\u{100}');
        }
    }
    chars.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Properties.

proptest! {
    #![proptest_config(config(256))]

    /// Round-trip: encoding bytes as a PouchDB binary string (optionally
    /// escaped as websql does) and decoding gives the same bytes; the encoder
    /// agrees with an independent oracle.
    #[test]
    fn round_trip_binary_string(bytes in prop::collection::vec(any::<u8>(), 0..1024), escaped in any::<bool>()) {
        let text = encode_binary_string(&bytes);
        prop_assert_eq!(&text, &binary_string(&bytes));
        let stored = if escaped { escape_blob(&text) } else { text.clone() };
        let has_nul = stored.contains(char::from(0));
        prop_assert!(!(escaped && has_nul), "escaped bodies hold no NUL");
        let read = if escaped { unescape_blob(&stored) } else { stored };
        prop_assert_eq!(decode_binary_string(&read).unwrap(), bytes);
    }

    /// Refinement: decoding succeeds only for strings of code points up to
    /// U+00FF and otherwise names the first wider character.
    #[test]
    fn refinement_binary_string_rejects_wide_characters(
        prefix in prop::collection::vec(any::<u8>(), 0..64),
        wide in (0x100u32..=0x10FFFF).prop_filter_map("scalar value", char::from_u32),
        rest in ".{0,16}",
    ) {
        let text = format!("{}{wide}{rest}", binary_string(&prefix));
        let err = decode_binary_string(&text).unwrap_err();
        prop_assert_eq!(err.index, prefix.len());
        prop_assert_eq!(err.code_point, u32::from(wide));
    }

    /// Round-trip: every live note, notebook, tag and file comes out of the
    /// store with the winning revision's fields and rev, files with identical
    /// bytes, and activity docs are archived verbatim.
    #[test]
    fn round_trip_import(corpus in gen_corpus(0..4)) {
        let run = setup(&corpus);
        let report = run_import(&run);
        prop_assert!(report.is_clean(), "{report}");

        for doc in corpus.docs.iter().filter(|d| d.doc_type.is_some() && d.is_live()) {
            let stored = run.store.get(&doc.id).unwrap();
            prop_assert!(stored.is_some(), "{} missing", doc.id);
            let stored = stored.unwrap();
            prop_assert_eq!(&stored.rev, &doc.winning_rev());
            prop_assert_eq!(&stored.fields, &doc.expected_fields(), "{}", doc.id);
            if let Some(bytes) = &doc.winning().bytes {
                let (_, attachment) = run.store.attachment(&doc.id).unwrap().unwrap();
                prop_assert_eq!(&attachment.data, bytes);
            }
        }
        let archived = archive(&run.store);
        for doc in corpus.live(None) {
            let mut expected = Map::new();
            expected.insert("_id".into(), doc.id.clone().into());
            expected.insert("_rev".into(), doc.winning_rev().into());
            expected.extend(doc.winning().fields.clone());
            prop_assert_eq!(archived.get(&doc.id), Some(&Value::Object(expected)));
        }

        prop_assert_eq!(report.notes, corpus.live(Some(DocType::Note)).count());
        prop_assert_eq!(
            report.trashed_notes,
            corpus.live(Some(DocType::Note)).filter(|n| n.winning().fields["bookId"] == "trash").count()
        );
        prop_assert_eq!(report.books, corpus.live(Some(DocType::Book)).count());
        prop_assert_eq!(report.tags, corpus.live(Some(DocType::Tag)).count());
        prop_assert_eq!(report.files, corpus.live(Some(DocType::File)).count());
        prop_assert_eq!(report.archived, corpus.live(None).count());
        prop_assert_eq!(report.skipped_design_and_config, 2);
        prop_assert_eq!(report.unchanged, 0);
        drop(run.fixture);
    }

    /// Idempotence: a second import writes nothing, reports every document
    /// as unchanged and leaves the store byte-for-byte the same.
    #[test]
    fn idempotence_second_import_changes_nothing(corpus in gen_corpus(0..4)) {
        let run = setup(&corpus);
        let first = run_import(&run);
        let docs = run.store.changes_since(0, None, false).unwrap();
        let archived = archive(&run.store);
        let stored_blobs = blobs(&run.store);

        let second = run_import(&run);
        prop_assert_eq!(second.written(), 0, "{}", second);
        prop_assert_eq!(second.replaced, 0);
        prop_assert_eq!(second.unchanged, first.written() + first.unchanged);
        prop_assert_eq!(&second.source, &first.source);
        prop_assert_eq!(run.store.changes_since(0, None, false).unwrap(), docs);
        prop_assert_eq!(archive(&run.store), archived);
        prop_assert_eq!(blobs(&run.store), stored_blobs);
    }

    /// Refinement: a file whose stored bytes don't match its digest is
    /// reported in `attachment_mismatches` and never imported; everything else
    /// still is.
    #[test]
    fn refinement_corrupt_attachment_is_reported_not_imported(
        corpus in gen_corpus(1..4),
        corruptions in prop::collection::vec((any::<Index>(), gen_corruption()), 1..3),
    ) {
        let live_files: Vec<&GenDoc> = corpus.live(Some(DocType::File)).collect();
        prop_assume!(!live_files.is_empty());
        let run = setup(&corpus);
        let mut corrupted = BTreeSet::new();
        for (pick, corruption) in &corruptions {
            let digest = live_files[pick.index(live_files.len())].digest().unwrap();
            // One corruption per body: a second flip could undo the first.
            if corrupted.insert(digest.clone()) {
                let body = run.fixture.attachment_body(&digest).unwrap();
                run.fixture.put_attachment_body(&digest, &corrupt(&body, corruption), false);
            }
        }
        let expected: BTreeSet<String> = live_files
            .iter()
            .filter(|f| corrupted.contains(&f.digest().unwrap()))
            .map(|f| f.id.clone())
            .collect();

        let report = run_import(&run);
        prop_assert_eq!(mismatch_ids(&report), expected.clone());
        for file in &live_files {
            let stored = run.store.get(&file.id).unwrap();
            if expected.contains(&file.id) {
                prop_assert!(stored.is_none(), "{} was imported", file.id);
                prop_assert!(run.store.attachment(&file.id).unwrap().is_none());
            } else {
                let (_, attachment) = run.store.attachment(&file.id).unwrap().unwrap();
                prop_assert_eq!(Some(&attachment.data), file.winning().bytes.as_ref());
            }
        }
        prop_assert_eq!(report.files, live_files.len() - expected.len());
        prop_assert_eq!(report.notes, corpus.live(Some(DocType::Note)).count());
    }

    /// Invariant: deleted winners and losing revisions never reach the store;
    /// every stored document is at its winning revision with no history.
    #[test]
    fn invariant_deleted_and_losing_revisions_never_appear(corpus in gen_corpus(0..4)) {
        let run = setup(&corpus);
        let report = run_import(&run);
        let archived = archive(&run.store);

        for doc in &corpus.docs {
            match doc.doc_type {
                Some(_) => {
                    let stored = run.store.get(&doc.id).unwrap();
                    if doc.is_live() {
                        prop_assert_eq!(stored.map(|d| d.rev), Some(doc.winning_rev()));
                    } else {
                        prop_assert!(stored.is_none(), "deleted {} was imported", doc.id);
                    }
                    prop_assert!(run.store.revisions(&doc.id).unwrap().is_empty());
                }
                None => {
                    let rev = archived.get(&doc.id).map(|json| json["_rev"].clone());
                    if doc.is_live() {
                        prop_assert_eq!(rev, Some(Value::from(doc.winning_rev())));
                    } else {
                        prop_assert_eq!(rev, None);
                    }
                }
            }
        }
        let counts = run.store.counts().unwrap();
        for doc_type in DocType::ALL {
            prop_assert_eq!(
                counts.get(&doc_type).copied().unwrap_or(0),
                corpus.live(Some(doc_type)).count()
            );
        }
        prop_assert_eq!(
            report.skipped_deleted,
            corpus.docs.iter().filter(|d| !d.is_live()).count()
        );
    }
}

proptest! {
    #![proptest_config(config(256))]

    /// Round-trip: Electron safeStorage encryption decrypts back with the
    /// same Keychain secret.
    #[test]
    fn round_trip_safe_storage(plaintext in prop::collection::vec(any::<u8>(), 0..200), secret in "[ -~]{1,40}") {
        let data = kanso_import::safe_storage::encrypt(&plaintext, &secret);
        prop_assert!(data.starts_with(b"v10"));
        prop_assert_eq!(kanso_import::safe_storage::decrypt(&data, &secret).unwrap(), plaintext);
    }

    /// Invariant: decrypting with a different secret never yields the
    /// plaintext.
    #[test]
    fn refinement_safe_storage_wrong_secret_fails(plaintext in "[a-z]{1,30}:[ -~]{1,30}", secret in "[a-z]{8}", other in "[A-Z]{8}") {
        let data = kanso_import::safe_storage::encrypt(plaintext.as_bytes(), &secret);
        let decrypted = kanso_import::safe_storage::decrypt(&data, &other);
        prop_assert!(decrypted.map_or(true, |bytes| bytes != plaintext.as_bytes()));
    }
}
