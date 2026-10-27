//! The sync schedule while the app is in front, against an in-memory server,
//! with delays shortened so no test waits for the real twenty seconds. The
//! rules themselves (at most one sync at a time, none behind or signed out)
//! are property-tested in `schedule.rs`.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::*;
use kanso_mobile::{Library, SyncPhase};
use kanso_sync::MemoryServer;

const AFTER_EDIT: Duration = Duration::from_millis(300);
const HOUR: Duration = Duration::from_secs(3600);

/// Counts the syncs that start on a library.
fn count_syncs(library: &Library) -> Arc<AtomicUsize> {
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    library.sync().status_stream(move |status| {
        if status.phase == SyncPhase::Syncing {
            seen.fetch_add(1, Ordering::SeqCst);
        }
        true
    });
    count
}

fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        if check() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("never happened: {what}");
}

fn settle() {
    std::thread::sleep(AFTER_EDIT * 4);
}

fn signed_in_phone() -> (Arc<MemoryServer>, Phone) {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    (server, phone)
}

// no-prop-test: one walk through the front, an edit, a burst, the back and
// the front again; the generated orders are in `schedule.rs`.
#[test]
fn the_front_syncs_now_an_edit_syncs_once_later_and_the_back_stops_it() {
    let (server, phone) = signed_in_phone();
    let sync = phone.library.sync();
    sync.set_schedule_for_test(AFTER_EDIT, HOUR);
    let count = count_syncs(&phone.library);

    // Nothing runs until the app is in front.
    settle();
    assert_eq!(count.load(Ordering::SeqCst), 0);
    sync.set_foreground(true);
    eventually("the front syncs at once", || {
        count.load(Ordering::SeqCst) == 1
    });
    settle();
    assert_eq!(count.load(Ordering::SeqCst), 1, "and only once");

    // One edit: one sync after the delay, which another phone then sees.
    let id = note(&phone.library, "Plan", "Alpha");
    assert_eq!(count.load(Ordering::SeqCst), 1, "not before the delay");
    eventually("the edit syncs", || count.load(Ordering::SeqCst) == 2);
    settle();
    assert_eq!(count.load(Ordering::SeqCst), 2);
    let other = common::phone(&server);
    sign_in(&other, "Other");
    other.library.sync().sync_now().unwrap();
    assert_eq!(other.library.note(&id).unwrap().title, "Plan");

    // A burst of edits is one sync, after the last.
    for n in 0..3 {
        let rev = phone.library.note(&id).unwrap().rev;
        phone.library.save(&id, &rev, format!("Alpha {n}")).unwrap();
        std::thread::sleep(AFTER_EDIT / 3);
    }
    eventually("the burst syncs", || count.load(Ordering::SeqCst) == 3);
    settle();
    assert_eq!(count.load(Ordering::SeqCst), 3, "one for the burst");

    // What a sync wrote here is not an edit: it asks for no sync of its own.
    let theirs = note(&other.library, "Theirs", "from the other phone");
    other.library.sync().sync_now().unwrap();
    let before = count.load(Ordering::SeqCst);
    phone.library.sync().sync_now().unwrap();
    assert_eq!(phone.library.note(&theirs).unwrap().title, "Theirs");
    settle();
    assert_eq!(
        count.load(Ordering::SeqCst),
        before + 1,
        "only the sync that was asked for"
    );

    // Behind, an edit syncs nothing.
    sync.set_foreground(false);
    let before = count.load(Ordering::SeqCst);
    note(&phone.library, "Later", "written behind");
    settle();
    assert_eq!(count.load(Ordering::SeqCst), before);

    // In front again, it syncs at once, and the edit goes.
    sync.set_foreground(true);
    eventually("the front syncs again", || {
        count.load(Ordering::SeqCst) == before + 1
    });
    other.library.sync().sync_now().unwrap();
    eventually("what was written behind reached the server", || {
        other.library.sync().sync_now().ok();
        other
            .library
            .list(Default::default())
            .map(|l| l.rows.iter().any(|r| r.title == "Later"))
            .unwrap_or(false)
    });
}

// no-prop-test: the interval over a second of real time.
#[test]
fn the_interval_repeats_in_front_and_stops_behind() {
    let (_server, phone) = signed_in_phone();
    let sync = phone.library.sync();
    sync.set_schedule_for_test(HOUR, Duration::from_millis(250));
    let count = count_syncs(&phone.library);
    sync.set_foreground(true);
    eventually("three syncs", || count.load(Ordering::SeqCst) >= 3);
    sync.set_foreground(false);
    // One may have been on its way; after it nothing starts.
    std::thread::sleep(Duration::from_millis(400));
    let stopped = count.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(count.load(Ordering::SeqCst), stopped);
}

// no-prop-test: signed out, then signing in while in front.
#[test]
fn signed_out_nothing_runs_and_signing_in_starts_the_schedule() {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    let phone = phone(&server);
    let sync = phone.library.sync();
    sync.set_schedule_for_test(AFTER_EDIT, Duration::from_millis(400));
    let count = count_syncs(&phone.library);
    sync.set_foreground(true);
    settle();
    assert_eq!(count.load(Ordering::SeqCst), 0);
    assert_eq!(sync.status().unwrap().phase, SyncPhase::SignedOut);
    assert!(phone.secrets.items().is_empty());

    sign_in(&phone, "Phone");
    eventually("signing in starts the schedule", || {
        count.load(Ordering::SeqCst) >= 1
    });
}

// no-prop-test: the schedule ends with its library.
#[test]
fn the_schedule_ends_when_the_library_closes() {
    let (_server, phone) = signed_in_phone();
    let path = phone.path();
    let sync = phone.library.sync();
    sync.set_schedule_for_test(AFTER_EDIT, Duration::from_millis(200));
    sync.set_foreground(true);
    std::thread::sleep(AFTER_EDIT);
    drop(sync);
    let Phone { dir, library, .. } = phone;
    drop(library);
    eventually("the library is closed, nothing keeps it open", || {
        Library::existing(&path).is_none()
    });
    drop(dir);
}
