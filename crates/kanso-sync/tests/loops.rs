//! Open loops over a `MemoryServer` account: `kansoLoops` is last-writer-wins
//! like the rest of a note, but two Macs that close different loops in the
//! same note keep both.

mod common;

use common::*;
use kanso_core::loops::{loop_items, loop_key};
use kanso_store::{LoopClose, Origin};
use kanso_sync::MemoryServer;

const BODY: &str = "Waiting on Tendai.\n\nI'll call Rudo.\n";

fn close(mac: &Mac, id: &str, text_start: &str, now: i64) {
    let body = body(&mac.store, id).unwrap();
    let item = loop_items(&body)
        .into_iter()
        .find(|i| i.text.starts_with(text_start))
        .unwrap();
    mac.store
        .close_loop(
            id,
            &loop_key(id, &item.text),
            item.line,
            LoopClose::Done {
                mark_in_note: false,
            },
            now,
            Origin::Ui,
        )
        .unwrap();
}

fn closed(mac: &Mac, id: &str) -> usize {
    let doc = mac.store.get_api(id, None, false).unwrap();
    doc["kansoLoops"]["closed"]
        .as_object()
        .map_or(0, |m| m.len())
}

#[test]
fn loops_closed_on_two_macs_in_the_same_note_both_stay_closed() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b", "B");
    let id = "note:loops".to_owned();
    note(&a.store, &id, "book:b", BODY, 1_000);
    a.sync(&server);
    b.sync(&server);

    close(&a, &id, "Waiting", 1_000);
    close(&b, &id, "I'll call", 2_000);
    a.sync(&server);
    b.sync(&server);
    a.sync(&server);
    b.sync(&server);

    for mac in [&a, &b] {
        assert_eq!(closed(mac, &id), 2);
    }
    assert_eq!(pending(&a.store), 0);
    assert_eq!(pending(&b.store), 0);
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
}
