//! `kanso serve`'s sync start against `MemoryServer` and `MemorySecrets`.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{
    Choice, DeviceIdentity, MemorySecrets, MemoryServer, Password, Session, SyncProblem,
};
use proptest::prelude::*;

use super::*;
use crate::config::AccountRef;

const PASSWORD: &str = "correct horse battery";

proptest! {
    #[test]
    fn prop_serve_related_notes_override_round_trips(on: bool, verbose in any::<bool>()) {
        let related = if on { "on" } else { "off" };
        let mut args = vec!["--related-notes".to_owned(), related.to_owned()];
        if verbose {
            args.push("--verbose".to_owned());
        }
        prop_assert_eq!(
            ServeOptions::parse(&args).unwrap(),
            ServeOptions { verbose, related_notes: Some(on) }
        );
    }
}

#[test]
fn serve_related_notes_rejects_missing_and_unknown_values() {
    assert!(ServeOptions::parse(&["--related-notes".into()]).is_err());
    assert!(ServeOptions::parse(&["--related-notes".into(), "maybe".into()]).is_err());
}

struct Lab {
    dir: tempfile::TempDir,
    server: Arc<MemoryServer>,
    secrets: Arc<MemorySecrets>,
    store: Arc<Store>,
    said: Arc<Mutex<Vec<String>>>,
    /// The account's other device.
    laptop: Session,
}

fn machine() -> Machine {
    Machine {
        hardware_uuid: None,
        computer_name: "vps-01".into(),
    }
}

impl Lab {
    fn new() -> Lab {
        let server = Arc::new(MemoryServer::new());
        let device = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Laptop", [9; 32]);
        let laptop = kanso_sync::sign_up(
            server.as_ref(),
            &MemorySecrets::new(),
            kanso_sync::DEFAULT_SERVER,
            "alice",
            &Password::new(PASSWORD, "alice").unwrap(),
            &device,
            &|_| {},
        )
        .unwrap()
        .session;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(crate::paths::db_path(dir.path())).unwrap());
        Lab {
            dir,
            server,
            secrets: Arc::new(MemorySecrets::new()),
            store,
            said: Arc::default(),
            laptop,
        }
    }

    fn flow(&self) -> Flow {
        Flow {
            backend: self.server.clone(),
            secrets: self.secrets.clone(),
            store: self.store.clone(),
            machine: Some(machine()),
            backups: crate::paths::backups_dir(self.dir.path()),
            now_ms: 0,
        }
    }

    /// Signs this library in, as `kanso account sign-in` does, and returns
    /// its session.
    fn sign_in(&self) -> Session {
        let flow = self.flow();
        let device = flow.device().unwrap();
        let session = kanso_sync::sign_in(
            self.server.as_ref(),
            kanso_sync::DEFAULT_SERVER,
            "alice",
            &Password::for_sign_in(PASSWORD).unwrap(),
            &device,
            &|_| {},
        )
        .unwrap()
        .session;
        flow.save_and_bind(&device.library_id, &session, Some(Choice::Add))
            .unwrap();
        let mut config = self.config();
        config.sync.account = Some(AccountRef {
            origin: session.origin.clone(),
            id: session.account_id.clone(),
            username: session.username.clone(),
        });
        config.sync.enabled = true;
        config
            .save(&crate::paths::config_path(self.dir.path()))
            .unwrap();
        session
    }

    fn config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self.dir.path())).unwrap()
    }

    fn start(&self) -> Option<Driver> {
        let said = self.said.clone();
        start_sync(SyncStart {
            config: &self.config(),
            data_dir: self.dir.path(),
            store: self.store.clone(),
            secrets: self.secrets.clone(),
            backend: self.server.clone(),
            machine: machine(),
            shared_notes: false,
            verbose: false,
            say: Arc::new(move |line| said.lock().unwrap().push(line)),
        })
    }

    fn said(&self) -> String {
        self.said.lock().unwrap().join("\n")
    }
}

/// Waits up to 10 s for `done`.
fn eventually(mut done: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn not_signed_in_says_so_once_and_runs_no_driver() {
    let lab = Lab::new();
    assert!(lab.start().is_none());
    assert_eq!(lab.said(), NOT_SIGNED_IN);
}

#[test]
fn signed_in_it_syncs_what_the_api_wrote() {
    let lab = Lab::new();
    let session = lab.sign_in();
    let book = lab
        .store
        .put(
            DocType::Book,
            serde_json::json!({"name": "Agents"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    lab.store
        .put(
            DocType::Note,
            serde_json::json!({"title": "from an agent", "bookId": book.id}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let driver = lab.start().expect("the driver runs");
    assert!(
        eventually(|| !lab.server.plain_view(&session).docs().is_empty()),
        "{}",
        lab.said()
    );
    driver.stop();
    let said = lab.said();
    assert!(said.contains("signed in as alice"), "{said}");
    assert!(said.contains("synced, 0 pulled, 2 pushed."), "{said}");
    // Nothing secret is ever said.
    for item in lab.secrets.items() {
        let raw = lab.secrets.get(&item).unwrap().unwrap();
        assert!(!said.contains(raw.as_str()));
    }
}

#[test]
fn sync_turned_off_starts_nothing() {
    let lab = Lab::new();
    lab.sign_in();
    let mut config = lab.config();
    config.sync.enabled = false;
    config
        .save(&crate::paths::config_path(lab.dir.path()))
        .unwrap();
    assert!(lab.start().is_none());
    assert!(lab.said().contains("sync is off"), "{}", lab.said());
}

#[test]
fn an_ended_session_is_forgotten_and_the_config_signed_out() {
    let lab = Lab::new();
    let session = lab.sign_in();
    kanso_sync::revoke_device(lab.server.as_ref(), &lab.laptop, &session.device_id).unwrap();
    let driver = lab.start().expect("the driver runs");
    assert!(
        eventually(|| lab.config().sync.account.is_none()),
        "{}",
        lab.said()
    );
    assert!(eventually(|| driver.finished()));
    driver.stop();
    let said = lab.said();
    assert_eq!(said.matches("sync stopped").count(), 1, "{said}");
    assert!(said.contains("The library stays and the API keeps working"));
    assert!(
        !lab.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
}

#[test]
fn only_loopback_is_quiet() {
    assert_eq!(exposed_warning("127.0.0.1"), None);
    assert_eq!(exposed_warning("::1"), None);
    assert_eq!(exposed_warning(" 127.0.0.2 "), None);
    // Every other address warns, a name or one that doesn't parse too, in
    // the design's words.
    for exposed in [
        "0.0.0.0",
        "::",
        "192.168.1.4",
        "fe80::1",
        "localhost",
        "my-mac.local",
        "nonsense",
        "",
    ] {
        let line = exposed_warning(exposed).unwrap_or_else(|| panic!("{exposed}"));
        assert!(
            line.contains("reachable from other computers over plain HTTP"),
            "{line}"
        );
    }
}

#[test]
fn serve_names_a_library_with_no_working_key() {
    let row = |revoked_at| kanso_store::ApiKeyRow {
        id: "k".into(),
        name: "n".into(),
        client: "other".into(),
        access: kanso_store::KeyAccess::Read,
        books: None,
        created_at: 0,
        last_used_at: None,
        revoked_at,
    };
    let notice = no_keys_notice(&[]).unwrap();
    assert!(notice.contains("kanso api-key create"), "{notice}");
    assert!(no_keys_notice(&[row(Some(1))]).is_some());
    assert_eq!(no_keys_notice(&[row(Some(1)), row(None)]), None);
}

fn gen_report() -> impl Strategy<Value = SyncReport> {
    (0usize..3, 0usize..3, 0usize..3, 0usize..3).prop_map(
        |(pulled, pushed, files_up, files_down)| SyncReport {
            pulled,
            pushed,
            files_up,
            files_down,
            ..SyncReport::default()
        },
    )
}

proptest! {
    /// A sync that moved nothing is quiet unless verbose; one that moved
    /// something says its counts on one line.
    #[test]
    fn prop_quiet_syncs_say_nothing(report in gen_report(), verbose: bool) {
        let lines = event_lines(&DriverEvent::Synced(report.clone()), verbose);
        let moved = report.pulled + report.pushed + report.files_up + report.files_down > 0;
        prop_assert_eq!(lines.len(), usize::from(moved || verbose));
        if let Some(line) = lines.first() {
            let expected = format!("{} pulled, {} pushed.", report.pulled, report.pushed);
            prop_assert!(line.ends_with(&expected));
        }
        prop_assert!(event_lines(&DriverEvent::Started, verbose).is_empty());
        prop_assert!(event_lines(&DriverEvent::Stopped, verbose).is_empty());
        prop_assert!(event_lines(&DriverEvent::SyncStarted, verbose).is_empty());
        prop_assert!(event_lines(&DriverEvent::Cancelled, verbose).is_empty());
        prop_assert_eq!(
            event_lines(&DriverEvent::HookFailed("x".into()), verbose).len(),
            usize::from(verbose)
        );
    }
}

#[test]
fn problems_read_as_one_line() {
    let line = problem_line(&SyncProblem::Offline);
    assert!(line.starts_with("kanso serve: offline: "), "{line}");
    assert!(!line.contains('\n'));
}
