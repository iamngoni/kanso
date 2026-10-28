//! The other device in the phone's end-to-end flows (#48 step 8): a command
//! line that signs up, approves, asks to be approved, shares and edits over
//! `HttpBackend` against a local Worker, so a script can stand a second
//! device up beside the phone. `scripts/mobile-sync-e2e.sh` runs it.
//!
//! ```text
//! e2e_peer sign-up         --server URL --username U --password P [--notes N] [--dir D]
//! e2e_peer approve-next    --server URL --username U --password P [--timeout-secs S] [--dir D]
//! e2e_peer request-approval --server URL --username U --password P [--timeout-secs S] [--dir D]
//! e2e_peer share           --server URL --username U --password P --to V [--role editor]
//!                          [--title T] [--body B] [--dir D]
//! e2e_peer edit            --server URL --username U --password P --share S --append TEXT [--dir D]
//! e2e_peer wait-text       --server URL --username U --password P --share S --contains TEXT
//!                          [--timeout-secs S] [--dir D]
//! e2e_peer accept          --server URL --username U --password P [--share S] [--timeout-secs S] [--dir D]
//! ```
//!
//! Every event is one JSON line on stdout (`{"event": "...", ...}`); what a
//! person would read goes to stderr. The exit status is 0 when the command
//! did what it was asked and 1 otherwise (the last line is then
//! `{"event":"error","message":"..."}`).
//!
//! The peer keeps its library in `--dir`, a temporary folder when it isn't
//! given (removed on exit). With the same `--dir` a later command is the
//! same device with the same library, so `share` and `edit` and `wait-text`
//! talk about one note.
//!
//! It only ever talks to a server on this computer or the Android emulator's
//! name for it ([`check_server`]): never to production. The recovery key it
//! prints and the passwords it takes are test data on a throwaway Worker.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use kanso_collab::http::HttpCollab;
use kanso_collab::{Account, Engine, Role};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::events::{self, EventError, EventSource, Signal};
use kanso_sync::{
    AccountError, ApprovalKeys, Backend, Choice, DeviceIdentity, HttpBackend, LockedSession,
    Password, PinnedRequest, RequestState, Session, SessionToken, SyncOptions, account_events_path,
    bind, challenge_request, complete_request, finish_approval, finish_unlock, name_device,
    open_request, pending_approvals, poll_approval, request_approval,
};
use serde_json::{Value, json};

/// A command's failure: the message the `error` event carries.
type Failure = String;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse(&args).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            emit(json!({"event": "error", "message": message}));
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// What was asked for
// ---------------------------------------------------------------------------

/// The hosts the peer may talk to: this computer, and the Android emulator's
/// name for it. Never a deployed server.
const ALLOWED_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "10.0.2.2"];

/// The port the desktop app's local server uses; the peer never goes near it.
const DESKTOP_PORT: u16 = 19840;

/// How long a command waits for the other side before it gives up.
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Whether `server` is a URL the peer may talk to: `http` or `https`, on this
/// computer or the emulator's name for it, with no credentials in it and not
/// the desktop app's port. Returns the URL without a trailing slash.
fn check_server(server: &str) -> Result<String, Failure> {
    let refused = |why: &str| {
        Err(format!(
            "refusing server {server:?}: {why}; the peer only talks to 127.0.0.1, localhost or 10.0.2.2"
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
    if !ALLOWED_HOSTS.iter().any(|h| host.eq_ignore_ascii_case(h)) {
        return refused("not a local host");
    }
    if port == Some(DESKTOP_PORT) {
        return refused("that port is the desktop app's");
    }
    Ok(server.trim_end_matches('/').to_owned())
}

/// What the person asked the peer to do.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Command {
    SignUp {
        notes: u32,
    },
    ApproveNext,
    RequestApproval,
    Share {
        to: String,
        role: Role,
        title: String,
        body: String,
    },
    Edit {
        share: String,
        append: String,
    },
    WaitText {
        share: String,
        contains: String,
    },
    Accept {
        share: Option<String>,
    },
}

/// A parsed command line.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Request {
    command: Command,
    server: String,
    username: String,
    password: String,
    timeout: Duration,
    dir: Option<PathBuf>,
}

/// The options of one command line: `--name value` pairs, each name once.
struct Options(BTreeMap<String, String>);

impl Options {
    fn read(args: &[String], allowed: &[&str]) -> Result<Options, Failure> {
        let mut found = BTreeMap::new();
        let mut args = args.iter();
        while let Some(name) = args.next() {
            let Some(key) = name.strip_prefix("--") else {
                return Err(format!("unexpected argument {name:?}"));
            };
            if !allowed.contains(&key) {
                return Err(format!("unknown option --{key}"));
            }
            let Some(value) = args.next() else {
                return Err(format!("--{key} needs a value"));
            };
            if found.insert(key.to_owned(), value.clone()).is_some() {
                return Err(format!("--{key} given twice"));
            }
        }
        Ok(Options(found))
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn need(&self, key: &str) -> Result<String, Failure> {
        self.get(key)
            .map(str::to_owned)
            .ok_or_else(|| format!("--{key} is required"))
    }

    fn number(&self, key: &str, default: u64) -> Result<u64, Failure> {
        match self.get(key) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| format!("--{key} must be a whole number, not {text:?}")),
        }
    }
}

/// A role as people say it: the engine's names, or the common ones.
fn parse_role(text: &str) -> Result<Role, Failure> {
    match text.to_ascii_lowercase().as_str() {
        "edit" | "editor" => Ok(Role::Edit),
        "comment" | "commenter" => Ok(Role::Comment),
        "view" | "viewer" => Ok(Role::View),
        other => Err(format!(
            "--role must be editor, commenter or viewer, not {other:?}"
        )),
    }
}

/// The command line, checked: `args` without the program's name.
fn parse(args: &[String]) -> Result<Request, Failure> {
    let Some((name, rest)) = args.split_first() else {
        return Err("a subcommand is required: sign-up, approve-next, request-approval, share, edit, wait-text or accept".into());
    };
    const COMMON: [&str; 5] = ["server", "username", "password", "timeout-secs", "dir"];
    let extra: &[&str] = match name.as_str() {
        "sign-up" => &["notes"],
        "approve-next" | "request-approval" => &[],
        "share" => &["to", "role", "title", "body"],
        "edit" => &["share", "append"],
        "wait-text" => &["share", "contains"],
        "accept" => &["share"],
        other => return Err(format!("unknown subcommand {other:?}")),
    };
    let allowed: Vec<&str> = COMMON.iter().chain(extra).copied().collect();
    let options = Options::read(rest, &allowed)?;
    let command = match name.as_str() {
        "sign-up" => {
            let notes = options.number("notes", 3)?;
            if notes > 200 {
                return Err("--notes is at most 200".into());
            }
            Command::SignUp {
                notes: notes as u32,
            }
        }
        "approve-next" => Command::ApproveNext,
        "request-approval" => Command::RequestApproval,
        "share" => Command::Share {
            to: options.need("to")?,
            role: parse_role(options.get("role").unwrap_or("editor"))?,
            title: options
                .get("title")
                .unwrap_or("Shared from the peer")
                .to_owned(),
            body: options
                .get("body")
                .unwrap_or("Shared note body.")
                .to_owned(),
        },
        "edit" => Command::Edit {
            share: options.need("share")?,
            append: options.need("append")?,
        },
        "wait-text" => Command::WaitText {
            share: options.need("share")?,
            contains: options.need("contains")?,
        },
        _ => Command::Accept {
            share: options.get("share").map(str::to_owned),
        },
    };
    let username = options.need("username")?;
    let password = options.need("password")?;
    if username.trim().is_empty() || password.is_empty() {
        return Err("--username and --password must not be empty".into());
    }
    Ok(Request {
        command,
        server: check_server(&options.need("server")?)?,
        username,
        password,
        timeout: Duration::from_secs(options.number("timeout-secs", DEFAULT_TIMEOUT_SECS)?),
        dir: options.get("dir").map(PathBuf::from),
    })
}

/// The text of the `i`th note `sign-up` writes for `username`: the canary the
/// phone's flows look for after joining.
fn canary(username: &str, i: u32) -> String {
    format!("canary-{username}-{i}")
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// One event: a JSON line on stdout, flushed so a script following the peer
/// sees it at once.
fn emit(event: Value) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{event}");
    let _ = out.flush();
}

fn say(message: &str) {
    eprintln!("[e2e_peer] {message}");
}

// ---------------------------------------------------------------------------
// The library and the device
// ---------------------------------------------------------------------------

/// A library of the peer's own: a folder with the store, and the device's
/// identity beside it so a later command is the same device.
struct Library {
    store: Arc<Store>,
    device: DeviceIdentity,
    dir: PathBuf,
    /// Held so a temporary folder lives as long as the library.
    _temp: Option<tempfile::TempDir>,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the system has randomness");
    bytes
}

impl Library {
    fn open(dir: Option<&Path>, name: &str) -> Result<Library, Failure> {
        let (dir, temp) = match dir {
            Some(dir) => (dir.to_owned(), None),
            None => {
                let temp = tempfile::Builder::new()
                    .prefix("kanso-e2e-peer.")
                    .tempdir()
                    .map_err(|err| format!("couldn't make a temporary folder: {err}"))?;
                (temp.path().to_owned(), Some(temp))
            }
        };
        std::fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;
        let store = Store::open(dir.join("kanso.db")).map_err(|err| format!("library: {err}"))?;
        let identity_file = dir.join("device.json");
        let device = match std::fs::read_to_string(&identity_file) {
            Ok(text) => {
                let saved: Value = serde_json::from_str(&text)
                    .map_err(|err| format!("{}: {err}", identity_file.display()))?;
                let library_id = saved["library_id"].as_str().unwrap_or_default();
                let secret = saved["secret"]
                    .as_str()
                    .and_then(unhex)
                    .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
                    .ok_or_else(|| format!("{} is not a device file", identity_file.display()))?;
                DeviceIdentity::from_parts(library_id, name, secret)
            }
            Err(_) => {
                let library_id = format!("l_{}", hex(&random::<16>()));
                let secret = random::<32>();
                let saved = json!({"library_id": library_id, "secret": hex(&secret)});
                std::fs::write(&identity_file, saved.to_string())
                    .map_err(|err| format!("{}: {err}", identity_file.display()))?;
                DeviceIdentity::from_parts(&library_id, name, secret)
            }
        };
        Ok(Library {
            store: Arc::new(store),
            device,
            dir,
            _temp: temp,
        })
    }

    fn backups(&self) -> PathBuf {
        self.dir.join("backups")
    }

    /// Binds the library to the account (adding what is in it) and syncs.
    fn bind_and_sync(&self, session: &Session) -> Result<kanso_sync::SyncReport, Failure> {
        bind(
            &self.store,
            session,
            Choice::Add,
            &self.backups(),
            kanso_core::now_ms(),
        )
        .map_err(|err| format!("couldn't bind the library: {err}"))?;
        self.sync(session)
    }

    fn sync(&self, session: &Session) -> Result<kanso_sync::SyncReport, Failure> {
        kanso_sync::sync(&self.store, &HttpBackend, session, &SyncOptions::default())
            .map_err(|err| format!("sync failed: {err}"))
    }

    /// The live notes in the library.
    fn note_count(&self) -> usize {
        self.store
            .changes_since(0, None, false)
            .map(|docs| {
                docs.into_iter()
                    .filter(|d| !d.deleted && d.id.starts_with("note:"))
                    .count()
            })
            .unwrap_or(0)
    }
}

/// The auth front filter's window (20 requests a minute per address): a 429
/// with a retry time inside it is waited out, as the sync tests do.
const BINDING_SECS: u64 = 60;

/// Runs an account call, waiting out the auth front filter's 429.
fn patiently<T>(call: impl Fn() -> Result<T, AccountError>) -> Result<T, AccountError> {
    for _ in 0..3 {
        match call() {
            Err(AccountError::RateLimited { retry_after }) if retry_after <= BINDING_SECS => {
                say(&format!("rate limited; waiting {retry_after} s"));
                std::thread::sleep(Duration::from_secs(retry_after + 1));
            }
            other => return other,
        }
    }
    call()
}

fn password_of(request: &Request) -> Result<Password, Failure> {
    Password::new(&request.password, &request.username)
        .map_err(|err| format!("that password won't do: {err}"))
}

/// Signs in with the password as this device: an unlocked session.
fn sign_in(request: &Request, device: &DeviceIdentity) -> Result<Session, Failure> {
    let password = password_of(request)?;
    patiently(|| {
        kanso_sync::sign_in(
            &HttpBackend,
            &request.server,
            &request.username,
            &password,
            device,
            &|_| {},
        )
    })
    .map(|signed_in| signed_in.session)
    .map_err(|err| format!("sign-in failed: {err}"))
}

fn engine(library: &Library, session: &Session) -> Engine {
    Engine::new(
        library.store.clone(),
        Arc::new(HttpCollab),
        Account::of(session),
    )
}

// ---------------------------------------------------------------------------
// The commands
// ---------------------------------------------------------------------------

fn run(request: Request) -> Result<(), Failure> {
    let name = match request.command {
        Command::ApproveNext => "E2E approver",
        Command::RequestApproval => "E2E new device",
        _ => "E2E peer",
    };
    let library = Library::open(request.dir.as_deref(), name)?;
    match request.command.clone() {
        Command::SignUp { notes } => sign_up(&request, &library, notes),
        Command::ApproveNext => approve_next(&request, &library),
        Command::RequestApproval => request_approval_flow(&request, &library),
        Command::Share {
            to,
            role,
            title,
            body,
        } => share(&request, &library, &to, role, &title, &body),
        Command::Edit { share, append } => edit(&request, &library, &share, &append),
        Command::WaitText { share, contains } => wait_text(&request, &library, &share, &contains),
        Command::Accept { share } => accept(&request, &library, share.as_deref()),
    }
}

/// Makes the account and puts `notes` notes with known canary text in it.
fn sign_up(request: &Request, library: &Library, notes: u32) -> Result<(), Failure> {
    let password = password_of(request)?;
    let signed_up = patiently(|| {
        kanso_sync::sign_up(
            &HttpBackend,
            &kanso_sync::MemorySecrets::new(),
            &request.server,
            &request.username,
            &password,
            &library.device,
            &|_| {},
        )
        .map_err(|err| err.error)
    })
    .map_err(|err| format!("sign-up failed: {err}"))?;
    let session = signed_up.session;
    emit(json!({"event": "signed_up", "username": session.username}));
    let book = library
        .store
        .put(
            DocType::Book,
            json!({"name": "E2E"}),
            Put::Upsert,
            Origin::Api,
        )
        .map_err(|err| format!("book: {err}"))?
        .id;
    for i in 0..notes {
        library
            .store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": format!("Peer note {i}"), "body": canary(&request.username, i)}),
                Put::Upsert,
                Origin::Api,
            )
            .map_err(|err| format!("note: {err}"))?;
    }
    let report = library.bind_and_sync(&session)?;
    say(&format!("pushed {} documents", report.pushed));
    // Publishes the account's sharing identity, so others can share with it.
    engine(library, &session)
        .identity()
        .map_err(|err| format!("sharing identity: {err}"))?;
    emit(json!({
        "event": "ready",
        "username": session.username,
        "notes": notes,
        "recovery_key": signed_up.recovery_key.display(),
    }));
    Ok(())
}

/// A device that is signed in and unlocked answers the next approval
/// request: it hears of it on the account event socket, challenges, waits
/// for the name, checks the commitment, shows the code and approves.
fn approve_next(request: &Request, library: &Library) -> Result<(), Failure> {
    let session = sign_in(request, &library.device)?;
    let backend = HttpBackend;
    let path = account_events_path(&backend, &request.server)
        .map_err(|err| format!("health: {err}"))?
        .unwrap_or_else(|| events::ACCOUNT_EVENTS_PATH.to_owned());
    let stop = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel::<Signal>();
    std::thread::scope(|scope| {
        let (session_ref, device, path, stop_ref) =
            (&session, &library.device, path.as_str(), &stop);
        let events_thread = scope.spawn(move || {
            let mut connect = || -> Result<Box<dyn EventSource>, EventError> {
                HttpBackend.events(session_ref, device, path)
            };
            let mut on = |signal: Signal| {
                tx.send(signal).ok();
            };
            let sleep = |wait: Duration| events::sleep_unless_stopped(stop_ref, wait);
            events::run(&mut connect, stop_ref, &mut on, &sleep);
        });
        let outcome = answer_next_request(&backend, &session, &rx, request.timeout);
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = events_thread.join();
        outcome
    })
}

/// The approver's side once the socket runs: every signal re-reads the list.
fn answer_next_request(
    backend: &dyn Backend,
    session: &Session,
    signals: &mpsc::Receiver<Signal>,
    timeout: Duration,
) -> Result<(), Failure> {
    let deadline = Instant::now() + timeout;
    // The request this device took, the request as it first looked, and the
    // key it answered with.
    let mut taken: Option<(String, PinnedRequest, ApprovalKeys)> = None;
    let mut waiting = true;
    emit(json!({"event": "listening"}));
    while waiting || taken.is_some() {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("no approval request arrived in time".into());
        }
        match signals.recv_timeout(left.min(Duration::from_secs(5))) {
            Ok(Signal::Ended(err)) => return Err(format!("the event socket ended: {err:?}")),
            Ok(Signal::Connected | Signal::Event(_)) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("the event loop stopped".into());
            }
        }
        let pending = pending_approvals(backend, session)
            .map_err(|err| format!("listing approval requests failed: {err}"))?;
        match taken.take() {
            None => {
                // The oldest request nobody answered.
                let Some(row) = pending
                    .iter()
                    .find(|row| row.epk.is_none() && row.approver_device_id.is_none())
                else {
                    continue;
                };
                let pinned = PinnedRequest::of(row);
                let keys = ApprovalKeys::generate();
                challenge_request(backend, session, row, &keys)
                    .map_err(|err| format!("challenge failed: {err}"))?;
                emit(json!({"event": "challenged", "request": row.id}));
                taken = Some((row.id.clone(), pinned, keys));
            }
            Some((id, pinned, keys)) => {
                let Some(row) = pending.iter().find(|row| row.id == id) else {
                    // Gone: the new device cancelled, or it expired.
                    return Err("the request was cancelled or expired".into());
                };
                match open_request(session, row, &pinned, &keys) {
                    Ok(None) => taken = Some((id, pinned, keys)),
                    Ok(Some(opened)) => {
                        emit(
                            json!({"event": "code", "code": opened.approval.code(), "device": opened.name}),
                        );
                        finish_approval(backend, session, row, &opened.approval)
                            .map_err(|err| format!("approve failed: {err}"))?;
                        emit(json!({"event": "approved", "request": id}));
                        waiting = false;
                    }
                    Err(err) => {
                        // A commitment that doesn't match: never approve.
                        return Err(format!("not approving: {err}"));
                    }
                }
            }
        }
    }
    Ok(())
}

/// A device that signs in with the password, then demotes itself to a locked
/// session as a provider or passkey sign-in without PRF would leave it, asks
/// to be approved and joins when it is.
fn request_approval_flow(request: &Request, library: &Library) -> Result<(), Failure> {
    let backend = HttpBackend;
    let session = sign_in(request, &library.device)?;
    let token = SessionToken::new(
        session
            .authorization()
            .trim_start_matches("Bearer ")
            .to_owned(),
    );
    let mut locked = LockedSession::new(
        session.server.clone(),
        session.origin.clone(),
        session.account_id.clone(),
        session.username.clone(),
        session.device_id.clone(),
        token,
        session.key_id(),
        "0000000000000000".to_owned(),
        None,
        None,
    )
    .with_login_device(kanso_sync::api::LoginDevice::for_identity(
        &library.device,
        &session.origin,
        &session.account_id,
    ));
    locked.set_library_id(&library.device.library_id);
    drop(session);

    let keys = ApprovalKeys::generate();
    let asked = request_approval(&backend, &locked, &keys)
        .map_err(|err| format!("asking for approval failed: {err}"))?;
    emit(json!({"event": "requested", "request": asked.id, "other_devices": asked.other_devices}));
    let deadline = Instant::now() + request.timeout;
    let wait = |what: &str| -> Result<RequestState, Failure> {
        loop {
            if Instant::now() >= deadline {
                return Err(format!("timed out {what}"));
            }
            match poll_approval(&backend, &locked, &asked) {
                Ok(RequestState::Waiting) => std::thread::sleep(Duration::from_millis(500)),
                Ok(state) => return Ok(state),
                Err(err) => return Err(format!("polling failed {what}: {err}")),
            }
        }
    };
    let RequestState::Challenged { approver_epk, .. } = wait("waiting for a device to answer")?
    else {
        return Err("the request ended before a device answered".into());
    };
    let approval = complete_request(&locked, &asked, &keys, &approver_epk)
        .map_err(|err| format!("the approver's key doesn't match: {err}"))?;
    name_device(&backend, &locked, &asked, &approval, &library.device)
        .map_err(|err| format!("naming the device failed: {err}"))?;
    emit(json!({"event": "code", "code": approval.code(), "device": library.device.name}));
    let loop_state = loop {
        match wait("waiting to be approved")? {
            RequestState::Challenged { .. } => std::thread::sleep(Duration::from_millis(500)),
            state => break state,
        }
    };
    let RequestState::Approved(approved) = loop_state else {
        return Err("the request was cancelled or expired".into());
    };
    let unlocked = finish_unlock(&backend, &locked, &approval, &approved, &library.device)
        .map_err(|err| format!("unlocking failed: {err}"))?;
    let session = unlocked.session;
    library.bind_and_sync(&session)?;
    emit(json!({"event": "joined", "notes": library.note_count()}));
    Ok(())
}

/// Makes a note and shares it with `to`.
fn share(
    request: &Request,
    library: &Library,
    to: &str,
    role: Role,
    title: &str,
    body: &str,
) -> Result<(), Failure> {
    let session = sign_in(request, &library.device)?;
    let engine = engine(library, &session);
    let book = library
        .store
        .put(
            DocType::Book,
            json!({"name": "Shared"}),
            Put::Upsert,
            Origin::Ui,
        )
        .map_err(|err| format!("book: {err}"))?
        .id;
    let note = library
        .store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": title, "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .map_err(|err| format!("note: {err}"))?
        .id;
    engine
        .person(to)
        .map_err(|err| format!("looking up {to:?} failed: {err}"))?;
    let share_id = engine
        .share(&note, &[(to.to_owned(), role)])
        .map_err(|err| format!("sharing failed: {err}"))?;
    emit(json!({"event": "shared", "share_id": share_id, "note_id": note}));
    Ok(())
}

/// The shared note's ID in the library, after the account's shares are
/// brought up to date.
fn shared_note_id(engine: &Engine, library: &Library, share_id: &str) -> Result<String, Failure> {
    engine
        .refresh()
        .map_err(|err| format!("refreshing the shares failed: {err}"))?;
    library
        .store
        .shared_note(share_id)
        .map_err(|err| format!("library: {err}"))?
        .map(|note| note.note_id)
        .ok_or_else(|| format!("this library has no shared note {share_id:?}"))
}

fn note_body(library: &Library, note_id: &str) -> Result<String, Failure> {
    library
        .store
        .note(note_id)
        .map_err(|err| format!("library: {err}"))?
        .map(|note| note.body)
        .ok_or_else(|| format!("no note {note_id:?}"))
}

/// Appends text to a shared note as a person typing would, and syncs it.
fn edit(request: &Request, library: &Library, share_id: &str, append: &str) -> Result<(), Failure> {
    let session = sign_in(request, &library.device)?;
    let engine = engine(library, &session);
    let note_id = shared_note_id(&engine, library, share_id)?;
    engine
        .sync_note(share_id)
        .map_err(|err| format!("syncing the note failed: {err}"))?;
    let body = format!("{}{append}", note_body(library, &note_id)?);
    library
        .store
        .put(
            DocType::Note,
            json!({"_id": note_id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .map_err(|err| format!("editing failed: {err}"))?;
    engine
        .sync_note(share_id)
        .map_err(|err| format!("syncing the note failed: {err}"))?;
    emit(json!({"event": "edited", "share_id": share_id, "length": body.len()}));
    Ok(())
}

/// Waits until the shared note's text holds `contains`: the other side's
/// edit has arrived.
fn wait_text(
    request: &Request,
    library: &Library,
    share_id: &str,
    contains: &str,
) -> Result<(), Failure> {
    let session = sign_in(request, &library.device)?;
    let engine = engine(library, &session);
    let note_id = shared_note_id(&engine, library, share_id)?;
    emit(json!({"event": "waiting", "share_id": share_id}));
    let deadline = Instant::now() + request.timeout;
    loop {
        engine
            .sync_note(share_id)
            .map_err(|err| format!("syncing the note failed: {err}"))?;
        let body = note_body(library, &note_id)?;
        if body.contains(contains) {
            emit(json!({"event": "text", "share_id": share_id, "contains": true}));
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("the note never came to contain {contains:?}"));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Waits for an invitation (to `share`, or any) and accepts it.
fn accept(request: &Request, library: &Library, share: Option<&str>) -> Result<(), Failure> {
    let session = sign_in(request, &library.device)?;
    let engine = engine(library, &session);
    let deadline = Instant::now() + request.timeout;
    loop {
        engine
            .refresh()
            .map_err(|err| format!("refreshing the shares failed: {err}"))?;
        let invitation = engine
            .invitations()
            .into_iter()
            .find(|i| share.is_none_or(|wanted| i.share_id == wanted));
        if let Some(invitation) = invitation {
            engine
                .accept(&invitation.share_id)
                .map_err(|err| format!("accepting failed: {err}"))?;
            let note = library
                .store
                .shared_note(&invitation.share_id)
                .ok()
                .flatten()
                .map(|n| n.note_id);
            emit(
                json!({"event": "accepted", "share_id": invitation.share_id, "note_id": note, "from": invitation.owner.username}),
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("no invitation arrived in time".into());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    // no-prop-test: a table of the exact hosts the brief names and the ones it forbids
    #[test]
    fn local_servers_are_allowed_and_everything_else_is_refused() {
        for ok in [
            "http://127.0.0.1:8787",
            "http://localhost:8787/",
            "https://127.0.0.1",
            "http://10.0.2.2:5000",
            "HTTP://LOCALHOST:1",
        ] {
            assert!(check_server(ok).is_ok(), "{ok}");
        }
        for bad in [
            "https://sync.kansoapp.app",
            "https://example.com",
            "http://127.0.0.1.evil.com:80",
            "http://evil.com/127.0.0.1",
            "http://127.0.0.1@evil.com",
            "http://localhost:8787@evil.com",
            "http://127.0.0.1:19840",
            "http://127.0.0.1:notaport",
            "http://127.0.0.1:0",
            "ftp://127.0.0.1",
            "127.0.0.1:8787",
            "",
            "http://",
            "http://[::1]:8787",
            "http://0.0.0.0:8787",
        ] {
            assert!(check_server(bad).is_err(), "{bad}");
        }
        assert_eq!(
            check_server("http://127.0.0.1:8787/").unwrap(),
            "http://127.0.0.1:8787"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        // A host that is not one of the three is refused whatever scheme,
        // port, path or credentials surround it.
        #[test]
        fn a_foreign_host_is_always_refused(
            host in "[a-z0-9.-]{1,24}",
            https in any::<bool>(),
            port in proptest::option::of(1u16..),
            path in "(/[a-z0-9]{0,8}){0,3}",
        ) {
            prop_assume!(!ALLOWED_HOSTS.iter().any(|h| host.eq_ignore_ascii_case(h)));
            let scheme = if https { "https" } else { "http" };
            let port = port.map(|p| format!(":{p}")).unwrap_or_default();
            let url = format!("{scheme}://{host}{port}{path}");
            prop_assert!(check_server(&url).is_err(), "{}", url);
        }

        // An allowed host is accepted with any port but the desktop's, and
        // the answer has no trailing slash.
        #[test]
        fn an_allowed_host_is_accepted_on_any_other_port(
            index in 0usize..3,
            https in any::<bool>(),
            port in proptest::option::of(1u16..),
            slash in any::<bool>(),
        ) {
            prop_assume!(port != Some(DESKTOP_PORT));
            let scheme = if https { "https" } else { "http" };
            let port_text = port.map(|p| format!(":{p}")).unwrap_or_default();
            let url = format!("{scheme}://{}{port_text}{}", ALLOWED_HOSTS[index], if slash { "/" } else { "" });
            let checked = check_server(&url).expect("allowed");
            prop_assert!(!checked.ends_with('/'));
            prop_assert!(url.starts_with(&checked));
        }

        // Credentials in front of the host never get through, even an allowed one.
        #[test]
        fn credentials_are_always_refused(
            user in "[a-z0-9.:-]{0,12}",
            index in 0usize..3,
            other in "[a-z]{1,10}\\.[a-z]{2,3}",
        ) {
            let allowed_first = format!("http://{}@{other}", ALLOWED_HOSTS[index]);
            prop_assert!(check_server(&allowed_first).is_err());
            let allowed_last = format!("http://{user}@{}", ALLOWED_HOSTS[index]);
            prop_assert!(check_server(&allowed_last).is_err());
        }

        // The command line never panics, whatever it holds.
        #[test]
        fn parsing_never_panics(words in proptest::collection::vec(".{0,12}", 0..8)) {
            let _ = parse(&words);
        }
    }

    // no-prop-test: the exact shape of each subcommand's line
    #[test]
    fn the_subcommands_parse() {
        let base = "--server http://127.0.0.1:9 --username alice --password pw";
        let sign_up = parse(&args(&format!("sign-up {base} --notes 5"))).unwrap();
        assert_eq!(sign_up.command, Command::SignUp { notes: 5 });
        assert_eq!(sign_up.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        assert_eq!(
            parse(&args(&format!("sign-up {base}"))).unwrap().command,
            Command::SignUp { notes: 3 }
        );
        assert_eq!(
            parse(&args(&format!("approve-next {base} --timeout-secs 7")))
                .unwrap()
                .timeout,
            Duration::from_secs(7)
        );
        assert_eq!(
            parse(&args(&format!("request-approval {base}")))
                .unwrap()
                .command,
            Command::RequestApproval
        );
        assert_eq!(
            parse(&args(&format!(
                "share {base} --to bob --role viewer --title T --body B"
            )))
            .unwrap()
            .command,
            Command::Share {
                to: "bob".into(),
                role: Role::View,
                title: "T".into(),
                body: "B".into()
            }
        );
        assert_eq!(
            parse(&args(&format!("edit {base} --share S --append x")))
                .unwrap()
                .command,
            Command::Edit {
                share: "S".into(),
                append: "x".into()
            }
        );
        assert_eq!(
            parse(&args(&format!("wait-text {base} --share S --contains x")))
                .unwrap()
                .command,
            Command::WaitText {
                share: "S".into(),
                contains: "x".into()
            }
        );
        assert_eq!(
            parse(&args(&format!("accept {base}"))).unwrap().command,
            Command::Accept { share: None }
        );
    }

    // no-prop-test: each mistake is one fixed line
    #[test]
    fn mistakes_in_the_command_line_are_named() {
        let base = "--server http://127.0.0.1:9 --username alice --password pw";
        let err = |line: String| parse(&args(&line)).unwrap_err();
        assert!(parse(&[]).unwrap_err().contains("subcommand"));
        assert!(err("fly".into()).contains("unknown subcommand"));
        assert!(err("sign-up --username a --password p".into()).contains("--server is required"));
        assert!(err(format!("sign-up {base} --notes many")).contains("whole number"));
        assert!(err(format!("sign-up {base} --notes 201")).contains("at most 200"));
        assert!(err(format!("sign-up {base} --username twice")).contains("twice"));
        assert!(err(format!("sign-up {base} --bogus 1")).contains("unknown option"));
        assert!(err(format!("sign-up {base} --notes")).contains("needs a value"));
        assert!(err(format!("sign-up {base} stray")).contains("unexpected argument"));
        assert!(err(format!("share {base}")).contains("--to is required"));
        assert!(err(format!("share {base} --to b --role owner")).contains("--role"));
        assert!(err(format!("edit {base} --share S")).contains("--append is required"));
        assert!(
            err("sign-up --server https://sync.kansoapp.app --username a --password p".into())
                .contains("refusing server")
        );
        // `notes` belongs to sign-up only.
        assert!(err(format!("approve-next {base} --notes 2")).contains("unknown option"));
    }

    proptest! {
        // The canary is a function of the account and the index, and
        // different notes never share one.
        #[test]
        fn canaries_are_distinct_per_note(user in "[a-z0-9]{1,12}", a in 0u32..200, b in 0u32..200) {
            prop_assume!(a != b);
            prop_assert_ne!(canary(&user, a), canary(&user, b));
            prop_assert!(canary(&user, a).contains(&user));
        }

        // Hex round-trips, which the device file relies on.
        #[test]
        fn hex_round_trips(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
            prop_assert_eq!(unhex(&hex(&bytes)), Some(bytes));
        }
    }
}
