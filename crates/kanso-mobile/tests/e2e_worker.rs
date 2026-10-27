//! The phone's sync against the real Worker (#48 step 8), through the facade
//! exactly as the app calls it: `Library::open_with` with `HttpBackend`, the
//! account, approvals, shared notes and sync handles, and real key
//! derivation settings. `scripts/mobile-sync-e2e.sh` starts `wrangler dev`
//! on 127.0.0.1 and runs these with `KANSO_E2E_ENDPOINT` set:
//!
//! ```text
//! cargo test -p kanso-mobile --features test-support --test e2e_worker -- --ignored --test-threads=1
//! ```
//!
//! Every test makes accounts of its own with names no other run uses, and
//! none of them talks to anything but a server on this computer.
//!
//! A provider sign-in needs a real provider, so the phone that has to be
//! approved or recover is made locked with `AccountHandle::lock_for_test`:
//! a password sign-in, kept as the locked item a provider sign-in would
//! save, which is how `crates/kanso-sync/tests/e2e.rs` does it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use kanso_mobile::background::{self, BackgroundOutcome};
use kanso_mobile::{
    AccountState, ApprovalPhase, KansoError, Library, ListQuery, MemorySecrets, NewNote,
    PromptState, Secrets, ShareRole,
};
use kanso_sync::{Backend, HttpBackend};

/// The test password of every account here: throwaway data on a throwaway Worker.
const PASSWORD: &str = "correct horse battery e2e";

/// The port the desktop app's local server uses; never touched.
const DESKTOP_PORT: u16 = 19840;

/// Whether `server` is a URL these tests may talk to: `http` or `https` on
/// this computer, with no credentials in it and not the desktop app's port.
/// Returns the URL without a trailing slash.
fn check_endpoint(server: &str) -> Result<String, String> {
    let refused = |why: &str| {
        Err(format!(
            "refusing endpoint {server:?}: {why}; these tests only talk to 127.0.0.1 or localhost"
        ))
    };
    let Some((scheme, rest)) = server.split_once("://") else {
        return refused("not an http(s) URL");
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return refused("not an http(s) URL");
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') || authority.contains('\\') {
        return refused("it has credentials in it");
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => match port.parse::<u16>() {
            Ok(port) if port != 0 => (host, Some(port)),
            _ => return refused("its port is not a number"),
        },
        None => (authority, None),
    };
    if !["127.0.0.1", "localhost"]
        .iter()
        .any(|h| host.eq_ignore_ascii_case(h))
    {
        return refused("not a local host");
    }
    if port == Some(DESKTOP_PORT) {
        return refused("that port is the desktop app's");
    }
    Ok(server.trim_end_matches('/').to_owned())
}

/// The Worker these tests run against, from `KANSO_E2E_ENDPOINT`.
fn endpoint() -> String {
    let given = std::env::var("KANSO_E2E_ENDPOINT")
        .expect("set KANSO_E2E_ENDPOINT (scripts/mobile-sync-e2e.sh does)");
    check_endpoint(&given).unwrap_or_else(|why| panic!("{why}"))
}

// no-prop-test: a table of addresses the guard must refuse or take.
#[test]
fn the_guard_takes_local_addresses_and_refuses_the_rest() {
    for good in [
        "http://127.0.0.1:8787",
        "http://localhost:8787/",
        "https://LOCALHOST",
    ] {
        assert!(check_endpoint(good).is_ok(), "{good}");
    }
    for bad in [
        "https://sync.kansoapp.app",
        "http://127.0.0.1.evil.example:80",
        "http://user@127.0.0.1:8787",
        "http://127.0.0.1:19840",
        "ftp://127.0.0.1:8787",
        "127.0.0.1:8787",
        "http://10.0.2.2:8787",
    ] {
        assert!(check_endpoint(bad).is_err(), "{bad}");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A name no other run or call uses.
fn unique(prefix: &str) -> String {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let n = COUNT.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}{:x}{n}", kanso_core::now_ms())
}

/// How long a call waits out the Worker's front filter (20 requests a minute
/// per address) when it answers 429.
const FILTER_WAIT: Duration = Duration::from_secs(61);

/// Runs a facade call, waiting out a rate limit (its words start with "Too
/// many attempts"; the call hadn't run). Anything else comes back as it is.
fn patiently<T>(what: &str, call: impl Fn() -> Result<T, KansoError>) -> Result<T, KansoError> {
    for _ in 0..4 {
        match call() {
            Err(err) if err.to_string().starts_with("Too many attempts") => {
                eprintln!("{what}: rate limited ({err}); waiting");
                std::thread::sleep(FILTER_WAIT);
            }
            other => return other,
        }
    }
    call()
}

/// Polls `check` until it is true, or panics after `within`.
fn wait_until(what: &str, within: Duration, mut check: impl FnMut() -> bool) {
    let start = Instant::now();
    loop {
        if check() {
            return;
        }
        assert!(start.elapsed() < within, "{what}: not within {within:?}");
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// One device: a library in a folder of its own, with the secret store it
/// was opened with (the background entry is given the same one).
struct Device {
    library: Library,
    secrets: Arc<MemorySecrets>,
    dir: tempfile::TempDir,
    server: String,
}

impl Device {
    fn new(server: &str) -> Device {
        let dir = tempfile::tempdir().expect("a folder");
        let secrets = Arc::new(MemorySecrets::new());
        let library = open(&db_path(dir.path()), &secrets);
        Device {
            library,
            secrets,
            dir,
            server: server.to_owned(),
        }
    }

    fn path(&self) -> PathBuf {
        db_path(self.dir.path())
    }

    /// Signs up with the test password; returns the recovery key.
    fn sign_up(&self, username: &str, name: &str) -> String {
        let signed = patiently("sign up", || {
            self.library.account().sign_up_password(
                self.server.clone(),
                username.to_owned(),
                PASSWORD.to_owned(),
                name.to_owned(),
            )
        })
        .expect("sign up");
        assert!(matches!(signed.state, AccountState::SignedIn { .. }));
        assert!(!signed.recovery_key.key.is_empty());
        signed.recovery_key.key
    }

    fn sign_in(&self, username: &str, name: &str) {
        let state = patiently("sign in", || {
            self.library.account().sign_in_password(
                self.server.clone(),
                username.to_owned(),
                PASSWORD.to_owned(),
                name.to_owned(),
            )
        })
        .expect("sign in");
        assert!(matches!(state, AccountState::SignedIn { .. }), "{state:?}");
    }

    /// Writes a note and returns its ID.
    fn write(&self, title: &str, body: &str) -> String {
        self.library
            .create(NewNote {
                notebook_id: None,
                title: title.to_owned(),
                body: body.to_owned(),
                place: None,
                preferred_notebook_id: None,
                tags: Vec::new(),
                pinned: false,
            })
            .expect("create")
            .id
    }

    fn sync(&self) {
        patiently("sync", || self.library.sync().sync_now()).expect("sync");
    }

    /// Every note's text, from the library.
    fn texts(&self) -> Vec<String> {
        texts_of(&self.library)
    }
}

fn db_path(dir: &Path) -> PathBuf {
    dir.join("kanso.db")
}

fn open(path: &Path, secrets: &Arc<MemorySecrets>) -> Library {
    let secrets: Arc<dyn Secrets> = secrets.clone();
    Library::open_with(path, secrets, Arc::new(HttpBackend::new())).expect("open the library")
}

fn texts_of(library: &Library) -> Vec<String> {
    library
        .list(ListQuery::default())
        .expect("list")
        .rows
        .iter()
        .filter_map(|row| library.note(&row.id).ok()?.body)
        .collect()
}

/// A device with `count` notes of canary text, synced; returns the canaries.
fn with_notes(device: &Device, count: u32) -> Vec<String> {
    let run = unique("canary");
    let canaries: Vec<String> = (0..count).map(|i| format!("{run}-{i}")).collect();
    for (i, canary) in canaries.iter().enumerate() {
        device.write(&format!("Note {i}"), &format!("Body holding {canary}"));
    }
    device.sync();
    canaries
}

fn assert_holds(texts: &[String], canaries: &[String]) {
    for canary in canaries {
        assert!(
            texts.iter().any(|text| text.contains(canary.as_str())),
            "no note holds {canary}; the library has {texts:?}"
        );
    }
}

/// What a locked phone shows for the account, signed in with its password.
fn lock(device: &Device, username: &str, name: &str) {
    let state = patiently("lock", || {
        device.library.account().lock_for_test(
            device.server.clone(),
            username.to_owned(),
            PASSWORD.to_owned(),
            name.to_owned(),
        )
    })
    .expect("lock the phone");
    assert!(matches!(state, AccountState::Locked(_)), "{state:?}");
}

// ---------------------------------------------------------------------------
// 1. A new phone joins by device approval and decrypts
// ---------------------------------------------------------------------------

#[test]
#[ignore = "needs the local Worker (scripts/mobile-sync-e2e.sh)"]
fn a_new_phone_joins_by_device_approval_and_decrypts() {
    let server = endpoint();
    let username = unique("e2eapprove");
    let a = Device::new(&server);
    a.sign_up(&username, "Device A");
    let canaries = with_notes(&a, 3);

    // A listens on the live account event socket, as the app does.
    let (prompts, seen) = mpsc::channel();
    a.library
        .approvals()
        .watch(move |list| prompts.send(list).is_ok());

    let phone = Device::new(&server);
    lock(&phone, &username, "New phone");
    let (waits, waited) = mpsc::channel();
    phone
        .library
        .account()
        .request_approval("New phone".to_owned(), move |wait| waits.send(wait).is_ok());

    let (mut approver_code, mut phone_code) = (None, None);
    let mut approved = false;
    let mut done = false;
    let start = Instant::now();
    while !done {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "no approval within 120 s (approver code {approver_code:?}, phone code {phone_code:?})"
        );
        while let Ok(list) = seen.try_recv() {
            if approved {
                continue;
            }
            if let Some(ready) = list.iter().find(|p| p.state == PromptState::Ready) {
                assert_eq!(ready.device_name.as_deref(), Some("New phone"));
                approver_code = ready.code.clone();
                a.library
                    .approvals()
                    .approve(ready.id.clone())
                    .expect("approve");
                approved = true;
            }
        }
        while let Ok(wait) = waited.try_recv() {
            if wait.code.is_some() {
                phone_code = wait.code.clone();
            }
            match wait.phase {
                ApprovalPhase::Approved => done = true,
                ApprovalPhase::Failed { message } => panic!("the approval failed: {message}"),
                ApprovalPhase::Cancelled | ApprovalPhase::Expired | ApprovalPhase::NoDevices => {
                    panic!("the approval ended: {:?}", wait.phase)
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(approver_code.is_some(), "the approver never saw a code");
    assert_eq!(approver_code, phone_code, "both screens show one code");

    assert!(matches!(
        phone.library.account().state().expect("state"),
        AccountState::SignedIn { .. }
    ));
    phone.sync();
    assert_holds(&phone.texts(), &canaries);
}

// ---------------------------------------------------------------------------
// 2. ...and by recovery key
// ---------------------------------------------------------------------------

#[test]
#[ignore = "needs the local Worker (scripts/mobile-sync-e2e.sh)"]
fn a_new_phone_joins_by_recovery_key_and_decrypts() {
    let server = endpoint();
    let username = unique("e2erecover");
    let a = Device::new(&server);
    let recovery_key = a.sign_up(&username, "Device A");
    let canaries = with_notes(&a, 3);

    let phone = Device::new(&server);
    lock(&phone, &username, "New phone");
    let new_password = format!("{PASSWORD} new");
    let recovered = patiently("recover", || {
        phone.library.account().recover(
            server.clone(),
            username.clone(),
            recovery_key.clone(),
            new_password.clone(),
            "New phone".to_owned(),
        )
    })
    .expect("recover with the recovery key");
    assert!(
        matches!(recovered.state, AccountState::SignedIn { .. }),
        "{:?}",
        recovered.state
    );
    assert_ne!(
        recovered.recovery_key.key, recovery_key,
        "recovery makes a new key"
    );
    phone.sync();
    assert_holds(&phone.texts(), &canaries);

    // The key that was used is gone: a signed-out phone can't use it again.
    let other = Device::new(&server);
    let again = other.library.account().recover(
        server.clone(),
        username.clone(),
        recovery_key,
        format!("{PASSWORD} again"),
        "Another phone".to_owned(),
    );
    assert!(again.is_err(), "the old recovery key still works");
    assert!(matches!(
        other.library.account().state().expect("state"),
        AccountState::SignedOut
    ));
}

// ---------------------------------------------------------------------------
// 3. Background sync completes within the OS budget
// ---------------------------------------------------------------------------

#[test]
#[ignore = "needs the local Worker (scripts/mobile-sync-e2e.sh)"]
fn background_sync_completes_within_the_os_budget() {
    let server = endpoint();
    let username = unique("e2ebg");
    let a = Device::new(&server);
    a.sign_up(&username, "Device A");

    let phone = Device::new(&server);
    phone.sign_in(&username, "Phone");
    phone.sync();

    let canaries = with_notes(&a, 1);
    // The app isn't open: the library is closed, as when the system wakes the job.
    let (path, secrets) = (phone.path(), phone.secrets.clone());
    let Device {
        library,
        dir,
        server: _,
        secrets: _,
    } = phone;
    drop(library);

    let backend = || -> Arc<dyn Backend> { Arc::new(HttpBackend::new()) };
    let secrets_for = |secrets: &Arc<MemorySecrets>| -> Arc<dyn Secrets> { secrets.clone() };
    let ios_budget = Duration::from_secs(25);

    let start = Instant::now();
    let first = background::run(&path, ios_budget, backend(), secrets_for(&secrets));
    let took = start.elapsed();
    eprintln!("the background sync took {took:?}: {first:?}");
    assert_eq!(first, BackgroundOutcome::Done { changed: true });
    assert!(took < Duration::from_secs(15), "took {took:?} of 25 s");

    let second = background::run(&path, ios_budget, backend(), secrets_for(&secrets));
    assert_eq!(second, BackgroundOutcome::Done { changed: false });

    // A budget that is gone at once never hangs or panics.
    a.write("Another", "written while the phone is away");
    a.sync();
    let start = Instant::now();
    let spent = background::run(
        &path,
        Duration::from_millis(1),
        backend(),
        secrets_for(&secrets),
    );
    assert!(
        matches!(
            spent,
            BackgroundOutcome::OutOfTime | BackgroundOutcome::Done { .. }
        ),
        "{spent:?}"
    );
    assert!(start.elapsed() < Duration::from_secs(20), "it hung");
    let last = background::run(&path, ios_budget, backend(), secrets_for(&secrets));
    assert!(matches!(last, BackgroundOutcome::Done { .. }), "{last:?}");

    // What it pulled is in the library.
    let reopened = open(&path, &secrets);
    assert_holds(&texts_of(&reopened), &canaries);
    drop(dir);
}

// ---------------------------------------------------------------------------
// 4. Shared notes work in the foreground
// ---------------------------------------------------------------------------

/// Appends `addition` to a note's text.
fn append(library: &Library, note_id: &str, addition: &str) {
    let view = library.note(note_id).expect("note");
    let body = format!("{}{addition}", view.body.expect("open note"));
    library.save(note_id, &view.rev, body).expect("save");
}

fn body_of(library: &Library, note_id: &str) -> String {
    library
        .note(note_id)
        .ok()
        .and_then(|view| view.body)
        .unwrap_or_default()
}

#[test]
#[ignore = "needs the local Worker (scripts/mobile-sync-e2e.sh)"]
fn shared_notes_work_in_the_foreground() {
    let server = endpoint();
    let (owner, friend) = (unique("e2eown"), unique("e2efri"));
    let a = Device::new(&server);
    a.sign_up(&owner, "Device A");
    let b = Device::new(&server);
    b.sign_up(&friend, "Device B");
    let a_note = a.write("Shared", "start of the shared note");
    a.sync();
    // B's sign-up publishes its sharing identity by itself, off the sign-in
    // thread: an owner can share with B without B syncing first.
    let mut shared = None;
    wait_until(
        "B's sharing identity",
        Duration::from_secs(30),
        || match patiently("share", || {
            a.library
                .shared()
                .share(a_note.clone(), vec![(friend.clone(), ShareRole::Editor)])
        }) {
            Ok(info) => {
                shared = Some(info);
                true
            }
            Err(err) if err.to_string().contains("can share yet") => false,
            Err(err) => panic!("share: {err}"),
        },
    );
    let info = shared.expect("share");
    assert_eq!(info.my_role, ShareRole::Owner);

    let mut invitation = None;
    wait_until("the invitation", Duration::from_secs(60), || {
        invitation = b
            .library
            .shared()
            .invitations()
            .expect("invitations")
            .into_iter()
            .next();
        invitation.is_some()
    });
    let invitation = invitation.expect("an invitation");
    assert_eq!(invitation.from, owner);
    assert_eq!(invitation.role, ShareRole::Editor);
    let b_note = b
        .library
        .shared()
        .accept(invitation.share_id)
        .expect("accept");
    assert!(body_of(&b.library, &b_note).contains("start of the shared note"));

    a.library
        .shared()
        .open_live(a_note.clone())
        .expect("A live");
    b.library
        .shared()
        .open_live(b_note.clone())
        .expect("B live");

    // No manual sync from here on: the rooms carry the edits.
    let from_b = format!(" b-wrote-{}", unique("x"));
    append(&b.library, &b_note, &from_b);
    wait_until("B's edit reaches A", Duration::from_secs(60), || {
        body_of(&a.library, &a_note).contains(from_b.trim())
    });
    let from_a = format!(" a-wrote-{}", unique("y"));
    append(&a.library, &a_note, &from_a);
    wait_until("A's edit reaches B", Duration::from_secs(60), || {
        body_of(&b.library, &b_note).contains(from_a.trim())
    });
    assert!(body_of(&a.library, &a_note).contains(from_b.trim()));
    assert!(body_of(&b.library, &b_note).contains(from_a.trim()));
}

// ---------------------------------------------------------------------------
// 5. Sign-up shows a recovery key and syncs
// ---------------------------------------------------------------------------

#[test]
#[ignore = "needs the local Worker (scripts/mobile-sync-e2e.sh)"]
fn sign_up_shows_a_recovery_key_and_the_foreground_syncs_by_itself() {
    let server = endpoint();
    let username = unique("e2esignup");
    let phone = Device::new(&server);
    let key = phone.sign_up(&username, "Phone");
    let pending = phone
        .library
        .account()
        .pending_recovery_key()
        .expect("pending key")
        .expect("the key waits to be confirmed saved");
    assert_eq!(pending.key, key);
    phone.library.account().recovery_key_saved().expect("saved");
    assert!(
        phone
            .library
            .account()
            .pending_recovery_key()
            .expect("pending key")
            .is_none()
    );

    // In front, with the schedule's delays shortened, an edit syncs by itself.
    phone
        .library
        .sync()
        .set_schedule_for_test(Duration::from_millis(300), Duration::from_secs(2));
    phone.library.sync().set_foreground(true);
    let canary = unique("autosync");
    phone.write("Typed in front", &format!("text with {canary}"));

    let other = Device::new(&server);
    other.sign_in(&username, "Second device");
    wait_until(
        "the edit reaches the server by itself",
        Duration::from_secs(90),
        || {
            other.sync();
            other.texts().iter().any(|text| text.contains(&canary))
        },
    );
    phone.library.sync().set_foreground(false);
}
