//! Offline conflicts in a shared notebook (issue #390 › slice 40): three
//! devices of three members go offline, each edits the same notebook, and
//! they come back in every order; one is killed part way through a sync, or
//! loses the server's answers, and starts again. Whatever the order, all
//! three end with the same notes and text, and nothing anyone wrote is lost
//! except by the documented rules (docs: issue #390 › Shared notebooks:
//! data):
//!
//! - text merges by character, so every insertion stays, whole;
//! - properties merge one key at a time; two values for the same key end as
//!   one of them, and the version that lost is in the history of the device
//!   that had it;
//! - an edit to a note someone deleted, which the deleter hadn't seen,
//!   brings it back for everyone;
//! - a member removed, or a notebook stopped, keeps what that device had
//!   (its unsent edits included) in a private copy, and nothing more of it
//!   reaches the others.
//!
//! Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use std::collections::BTreeMap;

use kanso_core::id::DocType;
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use serde_json::json;

use super::book_tests::{notebook, shared, sync, texts};
use super::tests::{Device, device};
use crate::memory::MemoryCollab;
use crate::model::Role;

const BRIEF: &str =
    "---\nstage: Idea\ndue: 2026-01-01\n---\nIntro paragraph.\n\nSecond paragraph.\n";

/// Three members' devices on one shared notebook: A owns it, B and C edit.
struct Studio {
    server: MemoryCollab,
    devices: Vec<Device>,
    share: String,
}

fn studio() -> Studio {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let b = device(&server, 2, 1);
    let c = device(&server, 3, 1);
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[
            ("Brief".to_owned(), BRIEF.to_owned()),
            ("Shots".to_owned(), "one\n".to_owned()),
            ("Old".to_owned(), "keep me\n".to_owned()),
        ],
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], &book);
    sync(&[&a, &b, &c], &share);
    let studio = Studio {
        server,
        devices: vec![a, b, c],
        share,
    };
    for d in 0..3 {
        assert_eq!(studio.texts(d).len(), 3);
    }
    studio
}

/// The (account, device) numbers of the studio's device `d`.
fn numbers(d: usize) -> (u8, u8) {
    (d as u8 + 1, 1)
}

impl Studio {
    fn d(&self, d: usize) -> &Device {
        &self.devices[d]
    }

    fn book(&self, d: usize) -> String {
        self.d(d)
            .store
            .shared_book(&self.share)
            .unwrap()
            .unwrap()
            .book_id
    }

    fn texts(&self, d: usize) -> BTreeMap<String, String> {
        texts(self.d(d), &self.book(d))
    }

    /// The id of the note called `title` in the notebook on device `d`.
    fn id(&self, d: usize, title: &str) -> Option<String> {
        self.d(d)
            .store
            .notes_in_books(&[self.book(d)])
            .unwrap()
            .into_iter()
            .find(|n| n.title == title)
            .map(|n| n.id)
    }

    /// Device `d` edits note `title` as a person would, in the library.
    fn edit(&self, d: usize, title: &str, change: impl FnOnce(&str) -> String) {
        let id = self.id(d, title).unwrap_or_else(|| panic!("no {title}"));
        let now = self.d(d).store.note(&id).unwrap().unwrap().body;
        self.d(d)
            .store
            .put(
                DocType::Note,
                json!({"_id": id, "body": change(&now)}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
    }

    /// Device `d` deletes note `title` for good (the deletion is synced).
    fn delete(&self, d: usize, title: &str) {
        let id = self.id(d, title).unwrap();
        self.d(d).store.delete(&id, Origin::Ui).unwrap();
    }

    /// Device `d` moves note `title` to its trash.
    fn trash(&self, d: usize, title: &str) {
        let id = self.id(d, title).unwrap();
        self.d(d)
            .store
            .put(
                DocType::Note,
                json!({"_id": id, "bookId": kanso_core::TRASH_BOOK_ID}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
    }

    fn sync_one(&self, d: usize) {
        self.d(d).engine.sync_book(&self.share).unwrap();
    }

    /// Everyone online again, a few rounds: the notes and text every device
    /// has, the same on all three.
    fn converge(&self) -> BTreeMap<String, String> {
        let all: Vec<&Device> = self.devices.iter().collect();
        sync(&all, &self.share);
        let first = self.texts(0);
        for d in 1..3 {
            assert_eq!(self.texts(d), first, "device {d} differs from device 0");
        }
        first
    }

    /// Kills device `d` and starts it again on the same library.
    fn restart(&mut self, d: usize) {
        let (n, k) = numbers(d);
        let old = self.devices.remove(d);
        let fresh = old.restarted(&self.server, n, k);
        self.devices.insert(d, fresh);
    }

    /// Whether device `d` has a version of note `title` (now, or in its
    /// history) whose text contains `needle`.
    fn has_version(&self, d: usize, id: &str, needle: &str) -> bool {
        let store = &self.d(d).store;
        let now = store
            .note(id)
            .unwrap()
            .is_some_and(|n| n.body.contains(needle));
        now || store.revisions(id).unwrap().iter().any(|r| {
            r.fields
                .get("body")
                .and_then(|b| b.as_str())
                .is_some_and(|b| b.contains(needle))
        })
    }

    /// The notes in device `d`'s trash that came from the notebook, by title.
    fn trashed(&self, d: usize) -> BTreeMap<String, String> {
        texts(self.d(d), kanso_core::TRASH_BOOK_ID)
    }
}

/// Every order three devices can come back in.
fn orders() -> Vec<[usize; 3]> {
    vec![
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
}

/// The edits of the text cases, offline on all three.
fn text_edits(s: &Studio) {
    // Same paragraph, both ends of it, and the same word replaced twice.
    s.edit(0, "Brief", |t| {
        t.replace("Intro paragraph.", "Intro paragraph, from A.")
    });
    s.edit(1, "Brief", |t| {
        t.replace("Intro paragraph", "B says: Intro paragraph")
    });
    // Another paragraph, and a new one.
    s.edit(2, "Brief", |t| {
        t.replace(
            "Second paragraph.\n",
            "Second paragraph (C).\n\nThird, by C.\n",
        )
    });
    // The same spot: the end of the note.
    s.edit(0, "Shots", |t| format!("{t}two A\n"));
    s.edit(1, "Shots", |t| format!("{t}two B\n"));
    s.edit(2, "Shots", |t| t.replace("one", "ONE"));
}

#[test]
fn text_edited_offline_on_three_devices_lands_whole_in_every_order() {
    for order in orders() {
        let s = studio();
        text_edits(&s);
        for d in order {
            s.sync_one(d);
        }
        let all = s.converge();
        let brief = &all["Brief"];
        for piece in [
            "B says: ",
            ", from A.",
            "Second paragraph (C).",
            "\n\nThird, by C.\n",
        ] {
            assert!(
                brief.contains(piece),
                "{order:?}: {piece:?} lost in {brief:?}"
            );
        }
        assert!(
            brief.starts_with("---\nstage: Idea\ndue: 2026-01-01\n---\n"),
            "{brief:?}"
        );
        let shots = &all["Shots"];
        assert!(shots.starts_with("ONE\n"), "{order:?}: {shots:?}");
        // Two lines at one spot: both, each whole, once.
        for line in ["two A\n", "two B\n"] {
            assert_eq!(shots.matches(line).count(), 1, "{order:?}: {shots:?}");
        }
    }
}

#[test]
fn one_word_replaced_on_two_devices_keeps_both_replacements() {
    for order in orders() {
        let s = studio();
        s.edit(0, "Brief", |t| t.replace("Intro", "Lead"));
        s.edit(1, "Brief", |t| t.replace("Intro", "Help"));
        for d in order {
            s.sync_one(d);
        }
        let brief = s.converge()["Brief"].clone();
        // By character: the word both removed is gone once, and what each
        // typed in its place is there.
        assert!(!brief.contains("Intro"), "{brief:?}");
        assert!(
            brief.contains("Lead") && brief.contains("Help"),
            "{brief:?}"
        );
    }
}

#[test]
fn the_same_card_moved_on_two_devices_ends_on_one_stage_and_history_keeps_the_other() {
    for order in orders() {
        let s = studio();
        s.edit(0, "Brief", |t| t.replace("stage: Idea", "stage: Shooting"));
        s.edit(1, "Brief", |t| t.replace("stage: Idea", "stage: Review"));
        // Other keys meanwhile: one changed, one added.
        s.edit(2, "Brief", |t| {
            t.replace("due: 2026-01-01", "due: 2026-03-03\nowner: dan")
        });
        for d in order {
            s.sync_one(d);
        }
        let brief = s.converge()["Brief"].clone();
        assert_eq!(brief.matches("stage: ").count(), 1, "{brief:?}");
        assert!(brief.contains("due: 2026-03-03\n") && brief.contains("owner: dan\n"));
        assert!(brief.ends_with("---\nIntro paragraph.\n\nSecond paragraph.\n"));
        let (winner, loser) = if brief.contains("stage: Shooting\n") {
            ("Shooting", (1, "Review"))
        } else {
            assert!(brief.contains("stage: Review\n"), "{brief:?}");
            ("Review", (0, "Shooting"))
        };
        let _ = winner;
        // The move that lost is in the history of the device that made it.
        let (who, value) = loser;
        let id = s.id(who, "Brief").unwrap();
        assert!(
            s.has_version(who, &id, &format!("stage: {value}\n")),
            "{order:?}: device {who} lost {value} without a version"
        );
    }
}

#[test]
fn an_edit_brings_back_a_note_deleted_or_trashed_on_another_device_in_every_order() {
    for order in orders() {
        let s = studio();
        // A deletes "Old" for good, C puts "Shots" in its trash; B edits
        // both, never having seen either go. A also deletes "Brief", which
        // C only changes a property of.
        s.delete(0, "Old");
        s.trash(2, "Shots");
        s.edit(1, "Old", |t| format!("{t}edited by B\n"));
        s.edit(1, "Shots", |t| format!("{t}two B\n"));
        s.delete(0, "Brief");
        s.edit(2, "Brief", |t| t.replace("stage: Idea", "stage: Shooting"));
        for d in order {
            s.sync_one(d);
        }
        let all = s.converge();
        assert_eq!(
            all.get("Old").map(String::as_str),
            Some("keep me\nedited by B\n"),
            "{order:?}: {all:?}"
        );
        assert_eq!(
            all.get("Shots").map(String::as_str),
            Some("one\ntwo B\n"),
            "{order:?}: {all:?}"
        );
        assert!(
            all.get("Brief")
                .is_some_and(|b| b.contains("stage: Shooting\n")),
            "{order:?}: {all:?}"
        );
    }
}

/// The person who deleted a note an edit brought back is told, by its
/// title, whatever the order; nobody else is.
#[test]
fn whoever_deleted_a_note_an_edit_brought_back_is_told_in_every_order() {
    for order in orders() {
        let s = studio();
        s.delete(0, "Old");
        s.trash(2, "Shots");
        s.edit(1, "Old", |t| format!("{t}edited by B\n"));
        s.edit(1, "Shots", |t| format!("{t}two B\n"));
        let mut told: [Vec<String>; 3] = Default::default();
        let mut sync_one = |d: usize| {
            let sync = s.d(d).engine.sync_book(&s.share).unwrap();
            told[d].extend(sync.brought_back);
        };
        for d in order {
            sync_one(d);
        }
        for _ in 0..3 {
            for d in 0..3 {
                sync_one(d);
            }
        }
        assert_eq!(told[0], ["Old"], "{order:?}");
        assert!(told[1].is_empty(), "{order:?}: {:?}", told[1]);
        assert_eq!(told[2], ["Shots"], "{order:?}");
        assert_eq!(
            crate::book_copy::brought_back_notice("Old"),
            "\"Old\" was edited after you deleted it, so it's back."
        );
    }
}

#[test]
fn a_deletion_nobody_answers_with_an_edit_holds_and_lands_in_every_trash() {
    for order in orders() {
        let s = studio();
        s.delete(0, "Old");
        s.trash(1, "Shots");
        s.edit(2, "Brief", |t| format!("{t}more\n"));
        for d in order {
            s.sync_one(d);
        }
        let all = s.converge();
        assert!(
            !all.contains_key("Old") && !all.contains_key("Shots"),
            "{all:?}"
        );
        // Nothing is lost: each note is still in a trash where it can be
        // restored from.
        assert_eq!(
            s.trashed(1).get("Old").map(String::as_str),
            Some("keep me\n")
        );
        assert_eq!(
            s.trashed(2).get("Old").map(String::as_str),
            Some("keep me\n")
        );
        assert_eq!(s.trashed(0).get("Shots").map(String::as_str), Some("one\n"));
        assert_eq!(s.trashed(2).get("Shots").map(String::as_str), Some("one\n"));
    }
}

#[test]
fn a_member_removed_while_another_is_offline_cuts_the_one_and_not_the_other() {
    let s = studio();
    let b_account = s.d(1).engine.caller().account_id.clone();
    // Offline: B and C both write; A removes B (a new key).
    s.edit(1, "Brief", |t| format!("{t}from B, before the removal\n"));
    s.edit(2, "Brief", |t| format!("{t}from C, offline\n"));
    s.edit(2, "Brief", |t| t.replace("stage: Idea", "stage: Editing"));
    s.d(0)
        .engine
        .remove_book_member(&s.share, &b_account)
        .unwrap();
    s.sync_one(0);
    // B comes back: the notebook ended for B, whose unsent edit stays in
    // the private copy and reaches nobody.
    let b_end = s.d(1).engine.sync_book(&s.share).unwrap();
    let copy = b_end.ended.expect("B's notebook ended");
    let b_texts = texts(s.d(1), &copy.book_id);
    assert!(b_texts["Brief"].contains("from B, before the removal\n"));
    // C, an epoch behind with edits made under the old key, lands them.
    s.sync_one(2);
    s.sync_one(0);
    s.sync_one(2);
    let (a, c) = (s.texts(0), s.texts(2));
    assert_eq!(a, c);
    assert!(a["Brief"].contains("from C, offline\n") && a["Brief"].contains("stage: Editing\n"));
    assert!(!a["Brief"].contains("from B"), "{a:?}");
    assert!(!texts(s.d(1), &copy.book_id)["Brief"].contains("from C"));
}

#[test]
fn a_member_leaving_while_another_is_offline_loses_nothing_of_the_others() {
    let s = studio();
    s.edit(2, "Shots", |t| format!("{t}from C, offline\n"));
    // B's last edit goes with the leave; B keeps a private copy.
    s.edit(1, "Old", |t| format!("{t}from B, leaving\n"));
    let left = s.d(1).engine.leave_book(&s.share).unwrap();
    assert!(texts(s.d(1), &left.book_id)["Old"].contains("from B, leaving\n"));
    s.sync_one(2);
    s.sync_one(0);
    s.sync_one(2);
    let (a, c) = (s.texts(0), s.texts(2));
    assert_eq!(a, c);
    assert!(a["Shots"].contains("from C, offline\n"));
    assert!(a["Old"].contains("from B, leaving\n"));
}

#[test]
fn a_notebook_stopped_while_a_member_is_offline_leaves_their_edits_in_their_copy() {
    let s = studio();
    s.edit(2, "Shots", |t| format!("{t}from C, offline\n"));
    s.edit(0, "Old", |t| format!("{t}from A, last\n"));
    // B is online and has what A wrote last.
    s.sync_one(0);
    s.sync_one(1);
    let stopped = s.d(0).engine.stop_book(&s.share).unwrap();
    assert!(texts(s.d(0), &stopped.book_id)["Old"].contains("from A, last\n"));
    let b = s
        .d(1)
        .engine
        .sync_book(&s.share)
        .unwrap()
        .ended
        .expect("ended for B");
    assert!(texts(s.d(1), &b.book_id)["Old"].contains("from A, last\n"));
    // C was offline: its copy has its own edit, which reached nobody (the
    // notebook ends as each device has it: the documented rule).
    let c = s
        .d(2)
        .engine
        .sync_book(&s.share)
        .unwrap()
        .ended
        .expect("ended for C");
    let c_texts = texts(s.d(2), &c.book_id);
    assert_eq!(c_texts["Shots"], "one\nfrom C, offline\n");
    assert!(!texts(s.d(0), &stopped.book_id)["Shots"].contains("from C"));
}

/// Every kind of offline edit at once, on all three.
fn everything(s: &Studio) {
    text_edits(s);
    s.edit(0, "Brief", |t| t.replace("stage: Idea", "stage: Shooting"));
    s.edit(1, "Brief", |t| t.replace("stage: Idea", "stage: Review"));
    s.delete(0, "Old");
    s.edit(2, "Old", |t| format!("{t}edited by C\n"));
}

fn check_everything(s: &Studio, all: &BTreeMap<String, String>, what: &str) {
    let brief = &all["Brief"];
    for piece in [
        "B says: ",
        ", from A.",
        "Second paragraph (C).",
        "Third, by C.",
    ] {
        assert!(brief.contains(piece), "{what}: {piece:?} lost in {brief:?}");
    }
    assert_eq!(brief.matches("stage: ").count(), 1, "{what}: {brief:?}");
    let shots = &all["Shots"];
    for line in ["ONE\n", "two A\n", "two B\n"] {
        assert_eq!(shots.matches(line).count(), 1, "{what}: {shots:?}");
    }
    assert_eq!(
        all.get("Old").map(String::as_str),
        Some("keep me\nedited by C\n"),
        "{what}"
    );
    assert_eq!(all.len(), 3, "{what}: {all:?}");
    let _ = s;
}

#[test]
fn a_device_killed_at_any_point_of_its_sync_starts_again_and_everything_lands() {
    for killed in 0..3 {
        let mut k = 0;
        loop {
            let mut s = studio();
            everything(&s);
            // The others first, so the killed sync has the most to do.
            for d in (0..3).filter(|d| *d != killed) {
                s.sync_one(d);
            }
            s.server.go_offline_after(k, 10_000);
            let finished = s.d(killed).engine.sync_book(&s.share).is_ok();
            s.server.go_offline_after(0, 0);
            s.restart(killed);
            let all = s.converge();
            check_everything(
                &s,
                &all,
                &format!("device {killed} killed after {k} requests"),
            );
            if finished {
                break;
            }
            k += 1;
        }
        assert!(k > 3, "the sync of device {killed} made only {k} requests");
    }
}

#[test]
fn answers_lost_after_the_server_took_a_push_send_nothing_twice() {
    for lost in 1..12 {
        let mut s = studio();
        everything(&s);
        s.sync_one(0);
        s.sync_one(2);
        s.server.lose_book_replies(lost);
        // Each try stores one entry and never hears so; then the app dies.
        for _ in 0..lost {
            let _ = s.d(1).engine.sync_book(&s.share);
        }
        s.server.lose_book_replies(0);
        s.restart(1);
        let all = s.converge();
        check_everything(&s, &all, &format!("{lost} answers lost"));
    }
}

// ------------------------------------------------------------ properties

#[derive(Clone, Debug)]
enum Step {
    /// Adds a line at the end of a note.
    Append(usize, usize),
    /// Types a word at the start of the note's first paragraph.
    Prefix(usize, usize),
    /// Sets the note's stage.
    Stage(usize, usize),
    /// Sets another property of the note.
    Due(usize, usize),
    /// Deletes the note for good.
    Delete(usize, usize),
    /// Moves the note to the trash.
    Trash(usize, usize),
    /// The device comes online and syncs.
    Sync(usize),
    /// The device syncs, is killed after so many requests, and starts again.
    Kill(usize, usize),
}

fn step() -> impl Strategy<Value = Step> {
    let (d, n) = (0usize..3, 0usize..3);
    prop_oneof![
        4 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Append(d, n)),
        2 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Prefix(d, n)),
        3 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Stage(d, n)),
        1 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Due(d, n)),
        1 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Delete(d, n)),
        1 => (d.clone(), n.clone()).prop_map(|(d, n)| Step::Trash(d, n)),
        4 => d.clone().prop_map(Step::Sync),
        1 => (d, 0usize..25).prop_map(|(d, k)| Step::Kill(d, k)),
    ]
}

/// A word written by a person: unique to the step, so it can be looked for.
struct Wrote {
    at: usize,
    by: usize,
    note: String,
    word: String,
}

/// A note a device deleted (or put in its trash) at a step.
struct Deleted {
    at: usize,
    by: usize,
    note: String,
}

fn stage_of(text: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix("stage: "))
        .map(str::to_owned)
}

/// Runs a schedule; what was written, and who deleted which note after
/// writing to it.
fn run(s: &mut Studio, steps: &[Step]) -> (Vec<Wrote>, Vec<Deleted>) {
    let mut wrote = Vec::new();
    let mut deleted = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let word = format!("w{i}x");
        let title = |n: &usize| ["Brief", "Shots", "Old"][*n].to_owned();
        match step {
            Step::Append(d, n) | Step::Prefix(d, n) | Step::Stage(d, n) | Step::Due(d, n)
                if s.id(*d, &title(n)).is_none() => {}
            Step::Append(d, n) => {
                s.edit(*d, &title(n), |t| format!("{t}{word}\n"));
                wrote.push(Wrote {
                    at: i,
                    by: *d,
                    note: title(n),
                    word: format!("{word}\n"),
                });
            }
            Step::Prefix(d, n) => {
                s.edit(*d, &title(n), |t| {
                    let at = kanso_core::properties::split(t).map_or(0, |f| f.body_start);
                    format!("{}{word} {}", &t[..at], &t[at..])
                });
                wrote.push(Wrote {
                    at: i,
                    by: *d,
                    note: title(n),
                    word: format!("{word} "),
                });
            }
            Step::Stage(d, n) => s.edit(*d, &title(n), |t| {
                if stage_of(t).is_some() {
                    t.lines()
                        .map(|l| {
                            if l.starts_with("stage: ") {
                                format!("stage: {word}\n")
                            } else {
                                format!("{l}\n")
                            }
                        })
                        .collect()
                } else {
                    format!("---\nstage: {word}\n---\n{t}")
                }
            }),
            Step::Due(d, n) => s.edit(*d, &title(n), |t| {
                if t.contains("due: ") {
                    t.lines()
                        .map(|l| {
                            if l.starts_with("due: ") {
                                format!("due: {word}\n")
                            } else {
                                format!("{l}\n")
                            }
                        })
                        .collect()
                } else {
                    t.to_owned()
                }
            }),
            Step::Delete(d, n) | Step::Trash(d, n) if s.id(*d, &title(n)).is_none() => {}
            Step::Delete(d, n) => {
                s.delete(*d, &title(n));
                deleted.push(Deleted {
                    at: i,
                    by: *d,
                    note: title(n),
                });
            }
            Step::Trash(d, n) => {
                s.trash(*d, &title(n));
                deleted.push(Deleted {
                    at: i,
                    by: *d,
                    note: title(n),
                });
            }
            Step::Sync(d) => {
                // Whatever a sync replaces of a property is kept as a version.
                let before: Vec<(String, String, Option<String>)> = s
                    .d(*d)
                    .store
                    .notes_in_books(&[s.book(*d)])
                    .unwrap()
                    .into_iter()
                    .map(|n| (n.id, n.title, stage_of(&n.body)))
                    .collect();
                s.sync_one(*d);
                for (id, title, stage) in before {
                    let Some(stage) = stage else { continue };
                    let now = s.d(*d).store.note(&id).unwrap().map(|n| n.body);
                    if now
                        .as_deref()
                        .and_then(stage_of)
                        .is_some_and(|s| s != stage)
                    {
                        assert!(
                            s.has_version(*d, &id, &format!("stage: {stage}\n")),
                            "device {d}: stage {stage} of {title} replaced without a version"
                        );
                    }
                }
            }
            Step::Kill(d, k) => {
                s.server.go_offline_after(*k, 10_000);
                let _ = s.d(*d).engine.sync_book(&s.share);
                s.server.go_offline_after(0, 0);
                s.restart(*d);
            }
        }
    }
    (wrote, deleted)
}

proptest! {
    #![proptest_config(crate::tests::config(32))]

    /// Three devices that edit offline, delete, sync and are killed in any
    /// order end with the same notes and text. Every word written is in the
    /// notebook, unless its writer deleted the note after writing it (their
    /// own choice), or its note was deleted and nobody's edit answered: then
    /// it is in a trash.
    #[test]
    fn prop_three_devices_offline_converge(steps in prop::collection::vec(step(), 1..24)) {
        let mut s = studio();
        let (wrote, deleted) = run(&mut s, &steps);
        let all = s.converge();
        for text in all.values() {
            prop_assert!(text.matches("stage: ").count() <= 1, "{text:?}");
        }
        for w in &wrote {
            let own = deleted.iter().any(|x| x.by == w.by && x.note == w.note && x.at > w.at);
            match all.get(&w.note) {
                Some(text) => prop_assert!(
                    own || text.contains(&w.word),
                    "{:?} by device {} lost from {}: {:?}", w.word, w.by, w.note, text
                ),
                None => {
                    let in_trash = (0..3).any(|d| {
                        s.trashed(d).get(&w.note).is_some_and(|t| t.contains(&w.word))
                    });
                    prop_assert!(
                        in_trash || own,
                        "{:?} by device {} lost with {}", w.word, w.by, w.note
                    );
                }
            }
        }
    }
}
