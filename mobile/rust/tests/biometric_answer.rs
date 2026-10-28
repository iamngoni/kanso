//! The C entry the native lock plugin answers a biometric check through:
//! an unknown ticket, and an enrolment and an unlock through the exported
//! symbol, in a process with no Dart.

use std::sync::Arc;

use kanso_mobile::{Library, MemorySecrets, NewNote};
use kanso_mobile_bridge::biometric::kanso_biometric_answer;
use kanso_sync::MemoryServer;

const B: [u8; 32] = [3; 32];

fn answer(ticket: u64, status: i32, secret: Option<&[u8]>) -> i32 {
    let (pointer, len) = match secret {
        Some(bytes) => (bytes.as_ptr(), bytes.len()),
        None => (std::ptr::null(), 0),
    };
    // SAFETY: null, or `len` bytes that outlive the call.
    unsafe { kanso_biometric_answer(ticket, status, pointer, len) }
}

// no-prop-test: the entry's fixed outcomes through the C types.
#[test]
fn an_enrolment_and_an_unlock_go_through_the_c_entry() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_for_test(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    let locks = library.locks();

    // No such ticket.
    assert_eq!(answer(12345, 0, Some(&B)), 0);
    assert_eq!(answer(0, 0, None), 0);

    let id = library
        .create(NewNote {
            notebook_id: None,
            title: "Plan".into(),
            body: "the secret plan".into(),
            ..Default::default()
        })
        .unwrap()
        .id;
    locks.lock_note(&id, "open sesame".into()).unwrap();

    // A secret of the wrong size, or none, is a failed check: nothing kept.
    let ticket = locks.begin_biometric_enrollment().unwrap();
    assert_eq!(answer(ticket, 0, Some(&B[..16])), 1);
    assert!(!locks.biometrics_on().unwrap());
    let ticket = locks.begin_biometric_enrollment().unwrap();
    assert_eq!(answer(ticket, 0, None), 1);
    assert!(!locks.biometrics_on().unwrap());

    // The enrolment, once.
    let ticket = locks.begin_biometric_enrollment().unwrap();
    assert_eq!(answer(ticket, 0, Some(&B)), 1);
    assert_eq!(answer(ticket, 0, Some(&B)), 0);
    assert!(locks.biometrics_on().unwrap());

    // The unlock, with a wrong secret and then the right one.
    locks.lock().unwrap();
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(answer(ticket, 0, Some(&[4; 32])), -1);
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(locks.begin_biometric_unlock().unwrap(), None);

    locks.unlock("open sesame".into()).unwrap();
    let ticket = locks.begin_biometric_enrollment().unwrap();
    assert_eq!(answer(ticket, 0, Some(&B)), 1);
    locks.lock().unwrap();
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(answer(ticket, 0, Some(&B)), 1);
    assert!(locks.is_unlocked().unwrap());
    assert_eq!(
        library.note(&id).unwrap().body.as_deref(),
        Some("the secret plan")
    );

    // Invalidated: the keys are forgotten.
    locks.lock().unwrap();
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(answer(ticket, 2, None), 1);
    assert!(!locks.biometrics_on().unwrap());
}
