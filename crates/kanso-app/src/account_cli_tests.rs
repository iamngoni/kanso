//! `kanso account` against `MemoryServer` and `MemorySecrets`, with a
//! scripted terminal.

use std::collections::VecDeque;
use std::sync::Arc;

use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{DeviceIdentity, MemorySecrets, MemoryServer, Password};
use proptest::prelude::*;
use zeroize::Zeroizing;

use super::*;

const PASSWORD: &str = "correct horse battery";

/// A terminal that answers from a script and keeps what was said.
struct Script {
    tty: bool,
    answers: VecDeque<String>,
    said: Vec<String>,
}

impl Script {
    fn new(tty: bool, answers: &[&str]) -> Script {
        Script {
            tty,
            answers: answers.iter().map(|a| a.to_string()).collect(),
            said: Vec::new(),
        }
    }

    fn transcript(&self) -> String {
        self.said.join("\n")
    }
}

impl Console for Script {
    fn is_tty(&self) -> bool {
        self.tty
    }

    fn prompt(&mut self, line: &str) -> anyhow::Result<String> {
        self.said.push(line.to_owned());
        self.answers
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("no answer scripted for {line}"))
    }

    fn prompt_secret(&mut self, line: &str) -> anyhow::Result<Zeroizing<String>> {
        self.prompt(line).map(Zeroizing::new)
    }

    fn stdin_line(&mut self) -> anyhow::Result<Zeroizing<String>> {
        self.answers
            .pop_front()
            .map(Zeroizing::new)
            .ok_or_else(|| anyhow::anyhow!("nothing on standard input"))
    }

    fn say(&mut self, line: &str) {
        self.said.push(line.to_owned());
    }
}

/// An account `alice` on `server`, made from another computer.
fn account(server: &MemoryServer) {
    let device = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Laptop", [9; 32]);
    let password = Password::new(PASSWORD, "alice").unwrap();
    kanso_sync::sign_up(
        server,
        &MemorySecrets::new(),
        kanso_sync::DEFAULT_SERVER,
        "alice",
        &password,
        &device,
        &|_| {},
    )
    .unwrap();
}

struct Lab {
    dir: tempfile::TempDir,
    server: Arc<MemoryServer>,
    secrets: Arc<MemorySecrets>,
}

impl Lab {
    fn new() -> Lab {
        let server = Arc::new(MemoryServer::new());
        account(&server);
        Lab {
            dir: tempfile::tempdir().unwrap(),
            server,
            secrets: Arc::new(MemorySecrets::new()),
        }
    }

    fn world(&self) -> World {
        let secrets = self.secrets.clone();
        World {
            data_dir: self.dir.path().to_path_buf(),
            backend: self.server.clone(),
            open_secrets: Box::new(move |_| {
                let secrets: Arc<dyn Secrets> = secrets.clone();
                Ok(secrets)
            }),
            system_works: Box::new(|| false),
            machine: Machine {
                hardware_uuid: None,
                computer_name: "vps-01".into(),
            },
            now_ms: Box::new(|| 1_000_000),
            pause: Box::new(|_| {}),
        }
    }

    fn run(&self, args: &[&str], console: &mut Script) -> Result<(), Refusal> {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        run(&self.world(), parse(&args).unwrap(), console)
    }

    fn config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self.dir.path())).unwrap()
    }

    /// A note written before sign-in, as an agent would.
    fn note(&self) {
        let store = Store::open(crate::paths::db_path(self.dir.path())).unwrap();
        store
            .put(
                DocType::Note,
                serde_json::json!({"title": "from an agent"}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap();
    }
}

#[test]
fn a_fresh_library_signs_in_with_the_password_on_stdin() {
    let lab = Lab::new();
    let mut console = Script::new(false, &[PASSWORD]);
    lab.run(
        &["sign-in", "--username", "alice", "--password-stdin"],
        &mut console,
    )
    .unwrap();
    let config = lab.config();
    let account = config.sync.account.expect("signed in");
    assert_eq!(account.username, "alice");
    assert!(config.sync.enabled);
    // No system store answered here, so the file is the default.
    assert_eq!(config.sync.secret_store, SecretStoreKind::File);
    assert!(
        lab.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    let said = console.transcript();
    assert!(said.contains("Signed in as alice."), "{said}");
    assert!(said.contains("\"vps-01\""), "{said}");
    assert!(!said.contains(PASSWORD));
    // The session's secrets never reach the terminal.
    for value in lab.secrets.items() {
        let raw = lab.secrets.get(&value).unwrap().unwrap();
        assert!(!said.contains(raw.as_str()));
    }

    // Signed in already: a second sign-in is refused and changes nothing.
    let mut again = Script::new(false, &[PASSWORD]);
    let refused = lab
        .run(
            &["sign-in", "--username", "alice", "--password-stdin"],
            &mut again,
        )
        .unwrap_err();
    assert!(refused.0.contains("signed in as alice already"), "{}", refused.0);

    // Status, devices, revoke, sign-out.
    let mut status = Script::new(false, &[]);
    lab.run(&["status"], &mut status).unwrap();
    let said = status.transcript();
    assert!(said.contains("Account   alice"), "{said}");
    assert!(said.contains("Secrets   file"), "{said}");
    assert!(said.contains("Last sync never"), "{said}");

    let mut listed = Script::new(false, &[]);
    lab.run(&["devices"], &mut listed).unwrap();
    assert_eq!(listed.said.len(), 2, "{}", listed.transcript());
    assert!(listed.transcript().contains("this computer"));
    assert!(listed.transcript().contains("Laptop"));

    let laptop = listed
        .said
        .iter()
        .find(|line| line.contains("Laptop"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let mut revoked = Script::new(false, &[]);
    lab.run(&["revoke", &laptop], &mut revoked).unwrap();
    assert_eq!(revoked.transcript(), "Signed Laptop out.");

    let mut out = Script::new(false, &[]);
    lab.run(&["sign-out"], &mut out).unwrap();
    assert!(lab.config().sync.account.is_none());
    assert!(
        !lab.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    let mut status = Script::new(false, &[]);
    lab.run(&["status"], &mut status).unwrap();
    assert_eq!(status.transcript(), "Signed out");
}

#[test]
fn a_wrong_password_saves_nothing() {
    let lab = Lab::new();
    let mut console = Script::new(false, &["not the password"]);
    let refused = lab
        .run(
            &["sign-in", "--username", "alice", "--password-stdin"],
            &mut console,
        )
        .unwrap_err();
    assert!(!refused.0.is_empty());
    assert!(lab.config().sync.account.is_none());
    assert!(lab.secrets.items().is_empty());
}

#[test]
fn without_a_terminal_the_password_needs_stdin() {
    let lab = Lab::new();
    let mut console = Script::new(false, &[]);
    let refused = lab
        .run(&["sign-in", "--username", "alice"], &mut console)
        .unwrap_err();
    assert!(refused.0.contains("--password-stdin"), "{}", refused.0);
}

#[test]
fn a_terminal_is_asked_for_the_username_and_password() {
    let lab = Lab::new();
    let mut console = Script::new(true, &["alice", PASSWORD]);
    lab.run(&["sign-in"], &mut console).unwrap();
    assert!(lab.config().sync.account.is_some());
    assert!(console.said.contains(&"Password: ".to_owned()));
}

#[test]
fn a_library_with_notes_needs_an_answer() {
    let lab = Lab::new();
    lab.note();
    // No terminal and no flag: the new session is logged out, nothing kept.
    let mut console = Script::new(false, &[PASSWORD]);
    let refused = lab
        .run(
            &["sign-in", "--username", "alice", "--password-stdin"],
            &mut console,
        )
        .unwrap_err();
    assert!(refused.0.contains("--add or --replace"), "{}", refused.0);
    assert!(lab.config().sync.account.is_none());
    assert!(
        !lab.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    // With --add it binds.
    let mut console = Script::new(false, &[PASSWORD]);
    lab.run(
        &[
            "sign-in",
            "--username",
            "alice",
            "--password-stdin",
            "--add",
        ],
        &mut console,
    )
    .unwrap();
    assert!(lab.config().sync.account.is_some());
}

#[test]
fn a_terminal_answers_the_notes_question() {
    let lab = Lab::new();
    lab.note();
    let mut console = Script::new(true, &["alice", PASSWORD, "x", "a"]);
    lab.run(&["sign-in"], &mut console).unwrap();
    let said = console.transcript();
    assert!(said.contains("This computer already has notes"), "{said}");
    assert!(said.contains("Type a, r or c."), "{said}");
    assert!(lab.config().sync.account.is_some());
}

#[test]
fn a_running_kanso_holds_the_library() {
    let lab = Lab::new();
    let _held = kanso_store::lock_data_dir(lab.dir.path()).unwrap().unwrap();
    let mut console = Script::new(false, &[PASSWORD]);
    let refused = lab
        .run(
            &["sign-in", "--username", "alice", "--password-stdin"],
            &mut console,
        )
        .unwrap_err();
    assert!(refused.0.contains("stop it first"), "{}", refused.0);
    // Status works all the same.
    let mut status = Script::new(false, &[]);
    lab.run(&["status"], &mut status).unwrap();
    assert_eq!(status.transcript(), "Not signed in");
}

#[test]
fn browser_sign_ins_are_refused_up_front() {
    let lab = Lab::new();
    for provider in ["google", "apple", "passkey", "security-key"] {
        let mut console = Script::new(true, &[]);
        let refused = lab
            .run(&["sign-in", "--provider", provider], &mut console)
            .unwrap_err();
        assert!(refused.0.contains("needs a browser"), "{}", refused.0);
        assert!(console.said.is_empty());
    }
}

#[test]
fn commands_that_need_a_session_say_so() {
    let lab = Lab::new();
    for command in ["devices", "sign-out"] {
        let mut console = Script::new(false, &[]);
        let refused = lab.run(&[command], &mut console).unwrap_err();
        assert!(refused.0.contains("isn't signed in"), "{}", refused.0);
    }
}

#[test]
fn bad_command_lines() {
    let args = |s: &str| -> Vec<String> { s.split_whitespace().map(str::to_owned).collect() };
    assert_eq!(parse(&[]), Ok(Command::Help));
    assert!(parse(&args("sign-in --add --replace")).is_err());
    assert!(parse(&args("sign-in --frobnicate")).is_err());
    assert!(parse(&args("sign-in --username")).is_err());
    assert!(parse(&args("sign-in --secrets cloud")).is_err());
    assert!(parse(&args("sign-in --provider github --password-stdin")).is_err());
    assert!(parse(&args("sign-in --provider myspace")).is_err());
    assert!(parse(&args("revoke")).is_err());
    assert!(parse(&args("revoke a b")).is_err());
    assert!(parse(&args("status now")).is_err());
    assert!(parse(&args("launch")).is_err());
    assert_eq!(
        parse(&args("revoke d_91d2…")),
        Ok(Command::Revoke {
            device_id: "d_91d2…".into()
        })
    );
}

#[test]
fn small_words() {
    assert_eq!(spaced_code("482913"), "482 913");
    assert_eq!(spaced_code("12"), "12");
    assert_eq!(ago(100_000, 99_000), "just now");
    assert_eq!(ago(10 * 60_000, 0), "10 min ago");
    assert_eq!(ago(3_600_000, 0), "1 hour ago");
    assert_eq!(ago(86_400_000 + 1, 0), "yesterday");
    assert_eq!(ago(0, 5_000), "just now");
    assert_eq!(short_id("d_7a1e0000"), "d_7a1e…");
    assert_eq!(short_id("d_7a"), "d_7a");
}

fn gen_args() -> impl Strategy<Value = Command> {
    let word = "[a-z][a-z0-9.:/_-]{0,12}";
    let sign_in = (
        prop::option::of(word),
        prop::option::of(word),
        any::<bool>(),
        prop::sample::select(vec![
            Method::Password,
            Method::Github,
            Method::Refused("Google"),
            Method::Refused("Apple"),
            Method::Refused("a passkey"),
            Method::Refused("a security key"),
        ]),
        prop::option::of(word),
        prop::option::of(prop::sample::select(vec![
            SecretStoreKind::System,
            SecretStoreKind::File,
        ])),
        prop::option::of(prop::sample::select(vec![Choice::Add, Choice::Replace])),
    )
        .prop_map(
            |(server, username, stdin, method, device_name, secrets, binding)| {
                Command::SignIn(SignInArgs {
                    server,
                    username,
                    password_stdin: stdin && method != Method::Github,
                    method,
                    device_name,
                    secrets,
                    binding,
                })
            },
        );
    prop_oneof![
        Just(Command::Help),
        Just(Command::Status),
        Just(Command::Devices),
        Just(Command::SignOut),
        "d_[0-9a-f]{1,32}".prop_map(|device_id| Command::Revoke { device_id }),
        sign_in,
    ]
}

fn gen_device() -> impl Strategy<Value = kanso_sync::DeviceInfo> {
    (
        "d_[0-9a-f]{4}",
        prop::option::of("[A-Za-z ]{1,12}"),
        any::<bool>(),
    )
        .prop_map(|(id, name, current)| kanso_sync::DeviceInfo {
            id,
            name,
            created_at: 0,
            last_seen_at: 0,
            current,
        })
}

proptest! {
    /// Every command line `parse` accepts reads back the same.
    #[test]
    fn prop_commands_round_trip(command in gen_args()) {
        prop_assert_eq!(parse(&unparse(&command)), Ok(command));
    }

    /// Unknown flags are never accepted.
    #[test]
    fn prop_unknown_flags_are_refused(flag in "--[a-z]{3,10}") {
        prop_assume!(![
            "--server", "--username", "--password-stdin", "--provider",
            "--device-name", "--secrets", "--add", "--replace",
        ].contains(&flag.as_str()));
        prop_assert!(parse(&["sign-in".to_owned(), flag]).is_err());
    }

    /// A device is picked by its whole ID, or by a start of it only one
    /// device has; the table lists every device on its own line.
    #[test]
    fn prop_devices_are_picked_by_a_unique_start(
        devices in prop::collection::vec(gen_device(), 1..6),
        pick in any::<prop::sample::Index>(),
        cut in 1usize..7,
    ) {
        let wanted = pick.get(&devices);
        let exact = pick_device(&devices, &wanted.id).unwrap();
        prop_assert_eq!(&exact.id, &wanted.id);
        let start: String = wanted.id.chars().take(cut).collect();
        let sharing = devices.iter().filter(|d| d.id.starts_with(&start)).count();
        let exact_match = devices.iter().any(|d| d.id == start);
        match pick_device(&devices, &format!("{start}…")) {
            Ok(device) => prop_assert!(exact_match || (sharing == 1 && device.id.starts_with(&start))),
            Err(_) => prop_assert!(!exact_match && sharing != 1),
        }
        prop_assert_eq!(device_lines(&devices, 0).len(), devices.len());
    }
}
