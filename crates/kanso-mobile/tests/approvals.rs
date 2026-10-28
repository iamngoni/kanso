//! This phone approves other devices of the account: the request shows, the
//! two screens show the same code, and approving gives the new device the
//! data key and, with it, the phone's notes. Against an in-memory server,
//! with the asking device driven by `kanso_sync`'s calls for it.

mod common;
mod second_device;

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::*;
use kanso_mobile::{ApprovalPrompt, KansoError, Library, PromptState};
use kanso_sync::events::Frame;
use kanso_sync::{AccountEvent, MemoryServer, RequestState};
use second_device::*;

fn server_with_account() -> Arc<MemoryServer> {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    server
}

/// A signed-in phone, the approver.
fn approver(server: &Arc<MemoryServer>) -> Phone {
    let phone = phone(server);
    sign_in(&phone, "Phone A");
    phone
}

fn only(prompts: Vec<ApprovalPrompt>) -> ApprovalPrompt {
    assert_eq!(prompts.len(), 1, "{prompts:?}");
    prompts.into_iter().next().unwrap()
}

// no-prop-test: one device approved end to end, step by step.
#[test]
fn this_phone_approves_a_new_device_end_to_end() {
    let server = server_with_account();
    let a = approver(&server);
    let id = note(&a.library, "Groceries", "milk and eggs");
    a.library.sync().sync_now().unwrap();
    let mut b = new_device(&server, "Phone B");

    assert!(a.library.approvals().refresh().unwrap().is_empty());
    b.ask();

    // The phone answers at once with a key of its own; the device has not
    // named itself yet, so there is no name and no code.
    let waiting = only(a.library.approvals().refresh().unwrap());
    assert_eq!(waiting.state, PromptState::Challenged);
    assert_eq!(
        (waiting.device_name.as_deref(), waiting.code.as_deref()),
        (None, None)
    );
    assert!(waiting.expires_at_ms > waiting.asked_at_ms);
    // Refreshing again doesn't answer again, and doesn't change the state.
    assert_eq!(only(a.library.approvals().refresh().unwrap()), waiting);
    // Approving before the device is named is refused.
    assert!(matches!(
        a.library.approvals().approve(waiting.id.clone()),
        Err(KansoError::Invalid { .. })
    ));

    // The device sees the phone's answer, works out its code and names itself.
    let shown_on_b = b.name_to_approver().expect("the phone answered");
    let ready = only(a.library.approvals().refresh().unwrap());
    assert_eq!(ready.state, PromptState::Ready);
    assert_eq!(ready.device_name.as_deref(), Some("Phone B"));
    assert_eq!(ready.code.as_deref(), Some(shown_on_b.as_str()));
    assert_eq!(ready.id, waiting.id);

    assert!(a.library.approvals().approve(ready.id).unwrap().is_empty());
    assert!(a.library.approvals().refresh().unwrap().is_empty());

    // The new device opens the data key, binds, and syncs the phone's notes.
    b.unlock();
    let report = b.phone.library.sync().sync_now().unwrap();
    assert!(report.pulled > 0, "{report:?}");
    assert_eq!(b.phone.library.note(&id).unwrap().title, "Groceries");
    assert_eq!(
        b.phone.library.note(&id).unwrap().body.as_deref(),
        Some("milk and eggs")
    );
}

// no-prop-test: decline, then the same device asks again.
#[test]
fn declining_cancels_the_request_and_the_device_can_ask_again() {
    let server = server_with_account();
    let a = approver(&server);
    let mut b = new_device(&server, "Phone B");
    b.ask();
    let prompt = only(a.library.approvals().refresh().unwrap());

    assert!(
        a.library
            .approvals()
            .decline(prompt.id.clone())
            .unwrap()
            .is_empty()
    );
    assert_eq!(b.state(), RequestState::Cancelled);
    assert!(a.library.approvals().refresh().unwrap().is_empty());
    // Declining what is gone counts as declined.
    assert!(a.library.approvals().decline(prompt.id).unwrap().is_empty());

    b.ask();
    assert_eq!(
        only(a.library.approvals().refresh().unwrap()).state,
        PromptState::Challenged
    );
}

// no-prop-test: the server's clock past the request's end.
#[test]
fn an_expired_request_disappears() {
    let server = server_with_account();
    let a = approver(&server);
    let mut b = new_device(&server, "Phone B");
    let now = kanso_core::now_ms();
    server.set_now(now);
    b.ask();
    let prompt = only(a.library.approvals().refresh().unwrap());
    assert!(prompt.expires_at_ms > now);

    server.set_now(prompt.expires_at_ms + 1);
    assert!(a.library.approvals().refresh().unwrap().is_empty());
    assert_eq!(b.state(), RequestState::Expired);
    assert_eq!(
        a.library.approvals().approve(prompt.id),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the app closed between the phone's answer and the device's
// name; a request the phone has lost its key for can't be approved.
#[test]
fn a_request_whose_key_was_lost_reads_as_changed_and_is_never_approved() {
    let server = server_with_account();
    let a = approver(&server);
    let mut b = new_device(&server, "Phone B");
    b.ask();
    only(a.library.approvals().refresh().unwrap());

    // The app is closed and opened again: the phone's key for the request
    // lived in memory only.
    let Phone {
        dir,
        library,
        secrets,
    } = a;
    drop(library);
    let reopened =
        Library::open_for_test(&dir.path().join("library.db"), secrets, server.clone()).unwrap();

    b.name_to_approver().expect("the first answer stands");
    let prompt = only(reopened.approvals().refresh().unwrap());
    assert_eq!(prompt.state, PromptState::Changed);
    assert_eq!(
        (prompt.device_name.clone(), prompt.code.clone()),
        (None, None)
    );
    let refused = reopened.approvals().approve(prompt.id.clone());
    assert!(
        matches!(refused, Err(KansoError::Invalid { .. })),
        "{refused:?}"
    );
    // The device never got a key.
    assert!(!matches!(b.state(), RequestState::Approved(_)));

    // Declining it lets the device ask again.
    assert!(reopened.approvals().decline(prompt.id).unwrap().is_empty());
    assert_eq!(b.state(), RequestState::Cancelled);
    b.ask();
    assert_eq!(
        only(reopened.approvals().refresh().unwrap()).state,
        PromptState::Challenged
    );
}

// no-prop-test: two phones see one request; the first to answer owns it.
#[test]
fn a_request_another_phone_answered_is_not_shown_here() {
    let server = server_with_account();
    let first = approver(&server);
    let second = phone(&server);
    sign_in(&second, "Phone C");
    let mut b = new_device(&server, "Phone B");
    b.ask();

    assert_eq!(
        only(first.library.approvals().refresh().unwrap()).state,
        PromptState::Challenged
    );
    assert!(second.library.approvals().refresh().unwrap().is_empty());
    let code = b.name_to_approver().unwrap();
    let ready = only(first.library.approvals().refresh().unwrap());
    assert_eq!(ready.code.as_deref(), Some(code.as_str()));
    assert!(second.library.approvals().refresh().unwrap().is_empty());
    assert_eq!(
        second.library.approvals().approve(ready.id.clone()),
        Err(KansoError::NotFound)
    );
    first.library.approvals().approve(ready.id).unwrap();
}

// no-prop-test: the calls with nothing to approve, and a phone not signed in.
#[test]
fn only_a_signed_in_phone_approves() {
    let server = server_with_account();
    let signed_out = phone(&server);
    assert_eq!(
        signed_out.library.approvals().refresh(),
        Err(KansoError::SignedOut)
    );
    assert_eq!(
        signed_out.library.approvals().approve("x".into()),
        Err(KansoError::SignedOut)
    );
    assert_eq!(
        signed_out.library.approvals().decline("x".into()),
        Err(KansoError::SignedOut)
    );

    let a = approver(&server);
    assert_eq!(
        a.library.approvals().approve("not a request".into()),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: a locked note on the phone while a device is approved.
#[test]
fn nothing_of_a_locked_note_reaches_the_prompts() {
    let server = server_with_account();
    let a = approver(&server);
    let id = note(&a.library, "SECRET-TITLE-17", "SECRET-BODY-17");
    a.library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    a.library.locks().lock().unwrap();
    a.library.sync().sync_now().unwrap();
    let mut b = new_device(&server, "Phone B");
    b.ask();
    let mut shown = format!("{:?}", a.library.approvals().refresh().unwrap());
    b.name_to_approver().unwrap();
    shown += &format!("{:?}", a.library.approvals().refresh().unwrap());
    assert!(!shown.contains("SECRET"), "{shown}");
}

/// The lists a sink was sent.
fn lists() -> (
    Receiver<Vec<ApprovalPrompt>>,
    impl FnMut(Vec<ApprovalPrompt>) -> bool + Send + 'static,
) {
    let (tx, rx) = mpsc::channel();
    (rx, move |prompts| tx.send(prompts).is_ok())
}

/// The next list the sink is sent, within a few seconds.
fn next(rx: &Receiver<Vec<ApprovalPrompt>>) -> Vec<ApprovalPrompt> {
    rx.recv_timeout(Duration::from_secs(10))
        .expect("the sink should have been sent a list")
}

/// Waits until `condition` holds, for a few seconds.
fn until(what: &str, mut condition: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

// no-prop-test: a socket driven by hand: connect, events, a drop, a stop.
#[test]
fn watching_follows_the_event_socket_and_stops_with_its_sinks() {
    let server = server_with_account();
    let evented = Evented::new(&server);
    let a = phone_on(evented.clone());
    sign_in(&a, "Phone A");
    let mut b = new_device(&server, "Phone B");

    let (rx, sink) = lists();
    a.library.approvals().watch(sink);
    // The list at the start, and a socket opened.
    assert!(next(&rx).is_empty());
    until("the socket", || evented.connects() == 1);
    // The connect reads it again.
    assert!(next(&rx).is_empty());

    // An event for a request: the phone reads the list and answers.
    let request = b.ask();
    assert!(evented.send(Frame::Event(AccountEvent::ApprovalCreated(
        request.id.clone()
    ))));
    assert_eq!(only(next(&rx)).state, PromptState::Challenged);

    b.name_to_approver().unwrap();
    assert!(evented.send(Frame::Event(AccountEvent::ApprovalNamed(
        request.id.clone()
    ))));
    let ready = only(next(&rx));
    assert_eq!(ready.state, PromptState::Ready);

    // A second sink hears the lists too, and approving tells both.
    let (second_rx, second) = lists();
    a.library.approvals().watch(second);
    assert_eq!(only(next(&second_rx)).state, PromptState::Ready);
    assert_eq!(only(next(&rx)).state, PromptState::Ready);
    assert!(a.library.approvals().approve(ready.id).unwrap().is_empty());
    assert!(next(&rx).is_empty());
    assert!(next(&second_rx).is_empty());
    assert_eq!(evented.connects(), 1, "one socket serves every sink");

    // The network goes away: the thread reconnects and reads the list again.
    evented.drop_sockets();
    until("a second socket", || evented.connects() == 2);
    assert!(next(&rx).is_empty());
    assert!(next(&second_rx).is_empty());

    // Both sinks leave. The thread lets go of the socket by itself.
    drop((rx, second_rx));
    assert!(evented.send(Frame::Event(AccountEvent::ApprovalCancelled(request.id))));
    until("the socket to be let go", || !evented.open());
    // Watching again starts a thread again.
    let (rx, sink) = lists();
    a.library.approvals().watch(sink);
    assert!(next(&rx).is_empty());
    until("the third socket", || evented.connects() == 3);
}

// no-prop-test: lists sent from other threads while the thread reads.
#[test]
fn lists_sent_from_other_threads_do_not_end_the_watch() {
    let server = server_with_account();
    let evented = Evented::new(&server);
    let a = phone_on(evented.clone());
    sign_in(&a, "Phone A");
    let (rx, sink) = lists();
    a.library.approvals().watch(sink);
    until("the socket", || evented.connects() == 1);

    let threads: Vec<_> = (0..4)
        .map(|_| {
            let approvals = a.library.approvals();
            std::thread::spawn(move || {
                for _ in 0..40 {
                    approvals.refresh().unwrap();
                    std::thread::sleep(Duration::from_millis(5));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    // Still one socket, still held, and an event still gets its list.
    let heard = rx.try_iter().count();
    assert!(heard >= 160, "{heard}");
    assert_eq!(evented.connects(), 1);
    let mut b = new_device(&server, "Phone B");
    let request = b.ask();
    assert!(evented.send(Frame::Event(AccountEvent::ApprovalCreated(request.id))));
    assert_eq!(only(next(&rx)).state, PromptState::Challenged);
}

// no-prop-test: closing the library ends the thread and its socket.
#[test]
fn closing_the_library_ends_the_event_socket() {
    let server = server_with_account();
    let evented = Evented::new(&server);
    let a = phone_on(evented.clone());
    sign_in(&a, "Phone A");
    let seen = Arc::new(Mutex::new(0_usize));
    let counted = seen.clone();
    a.library.approvals().watch(move |_| {
        *counted.lock().unwrap() += 1;
        true
    });
    until("the socket", || {
        evented.connects() == 1 && *seen.lock().unwrap() >= 2
    });

    let Phone {
        dir,
        library,
        secrets,
    } = a;
    drop((library, secrets));
    until("the socket to be let go", || !evented.open());
    drop(dir);
}

// no-prop-test: signing out ends the thread, and the sink hears there is
// nothing to approve.
#[test]
fn signing_out_ends_the_event_socket_and_empties_the_list() {
    let server = server_with_account();
    let evented = Evented::new(&server);
    let a = phone_on(evented.clone());
    sign_in(&a, "Phone A");
    let mut b = new_device(&server, "Phone B");
    b.ask();

    let (rx, sink) = lists();
    a.library.approvals().watch(sink);
    assert_eq!(only(next(&rx)).state, PromptState::Challenged);
    until("the socket", || evented.connects() == 1);
    // The connect reads the list again.
    assert_eq!(only(next(&rx)).state, PromptState::Challenged);

    a.library.account_settings().sign_out().unwrap();
    assert!(next(&rx).is_empty());
    until("the socket to be let go", || !evented.open());
    // Nothing to approve now, and nothing kept.
    assert_eq!(a.library.approvals().refresh(), Err(KansoError::SignedOut));
}
