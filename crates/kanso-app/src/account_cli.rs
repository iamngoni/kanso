//! `kanso account`: signs a library in to a Kanso account without a window,
//! so `kanso serve` on a headless server can sync it, and shows its status,
//! devices and sign-out (docs/ACCOUNTS.md › Command line).
//!
//! It drives the engine's window-free [`Flow`], the same code the app signs
//! in with. Password sign-in is the main way in; GitHub sign-in works too,
//! approved from another signed-in device with a matching code. Google,
//! Apple, passkeys and security keys need a browser or an authenticator on
//! this computer, and are refused. It never creates or recovers an account.

use std::io::{BufRead as _, IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use anyhow::Result;
use kanso_engine::Secrets;
use kanso_engine::account::provider::{Browser, BrowserError, Listener};
use kanso_engine::account::{
    Flow, Next, NextUnlock, ProviderTaskResult, UnlockedCandidate, WaitEnd, WaitUpdate,
    after_auth, after_unlock, words,
};
use kanso_store::{Store, lock_data_dir};
use kanso_sync::{
    Backend, Choice, DeviceIdentity, GithubStart, GoogleStart, Machine, Password,
    Provider, Session, meta_keys,
};
use kanso_sync::api::LoginDevice;
use zeroize::Zeroizing;

use crate::config::{AccountRef, AppConfig, SecretStoreKind};

pub const USAGE: &str = "\
Usage: kanso account <command> [--data-dir <dir>]

Commands:
  sign-in      Sign this library in to your Kanso account
               [--username <name>] [--password-stdin] [--server <url>]
               [--provider github] [--device-name <name>]
               [--secrets file|system] [--add | --replace]
  status       Show the account, this device and the last sync; works
               while `kanso serve` runs
  devices      List the account's devices
  revoke <id>  Sign another device out (an ID from `devices`, or the
               start of one)
  sign-out     Sign this library out; its notes stay here
  help         Show this message

Sign-in, devices, revoke and sign-out need the data folder to themselves:
stop `kanso serve` (or quit Kanso) first. The password is asked for on the
terminal; scripts pass it on the first line of standard input with
--password-stdin. Google, Apple, passkeys and security keys need a browser
on this computer: sign in with a password or with --provider github.";

/// Exit status of a refusal: wrong password, server unreachable, a
/// question unanswered.
const REFUSED: i32 = 1;
/// Exit status of a bad command line.
const BAD_COMMAND: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    SignIn(SignInArgs),
    Status,
    SignOut,
    Devices,
    Revoke { device_id: String },
}

/// How `sign-in` was asked to sign in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Password,
    Github,
    /// A method that needs a browser or an authenticator on this computer.
    Refused(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignInArgs {
    pub server: Option<String>,
    pub username: Option<String>,
    pub password_stdin: bool,
    pub method: Method,
    pub device_name: Option<String>,
    pub secrets: Option<SecretStoreKind>,
    /// `--add` or `--replace`: the answer to "this computer already has
    /// notes" without a terminal.
    pub binding: Option<Choice>,
}

/// Why a command line isn't one.
#[derive(Debug, PartialEq, Eq)]
pub struct BadCommand(pub String);

/// The command `args` (after `account`) names. Pure.
pub fn parse(args: &[String]) -> Result<Command, BadCommand> {
    let bad = |text: String| Err(BadCommand(text));
    let Some((command, rest)) = args.split_first() else {
        return Ok(Command::Help);
    };
    let no_more = |command: Command| match rest.first() {
        Some(extra) => bad(format!("`{extra}` isn't something `{}` takes", args[0])),
        None => Ok(command),
    };
    match command.as_str() {
        "help" | "-h" | "--help" => no_more(Command::Help),
        "status" => no_more(Command::Status),
        "devices" => no_more(Command::Devices),
        "sign-out" => no_more(Command::SignOut),
        "revoke" => match rest {
            [id] if !id.starts_with('-') && !id.is_empty() => Ok(Command::Revoke {
                device_id: id.clone(),
            }),
            [] => bad("revoke needs a device ID; `kanso account devices` lists them".into()),
            _ => bad("revoke takes one device ID".into()),
        },
        "sign-in" => parse_sign_in(rest).map(Command::SignIn),
        other => bad(format!("unknown command `{other}`")),
    }
}

fn parse_sign_in(args: &[String]) -> Result<SignInArgs, BadCommand> {
    let mut parsed = SignInArgs {
        server: None,
        username: None,
        password_stdin: false,
        method: Method::Password,
        device_name: None,
        secrets: None,
        binding: None,
    };
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .filter(|value| !value.starts_with("--"))
                .cloned()
                .ok_or_else(|| BadCommand(format!("{name} needs a value")))
        };
        match arg.as_str() {
            "--server" => parsed.server = Some(value("--server")?),
            "--username" => parsed.username = Some(value("--username")?),
            "--device-name" => parsed.device_name = Some(value("--device-name")?),
            "--password-stdin" => parsed.password_stdin = true,
            "--provider" => {
                parsed.method = match value("--provider")?.to_ascii_lowercase().as_str() {
                    "github" => Method::Github,
                    "password" => Method::Password,
                    "google" => Method::Refused("Google"),
                    "apple" => Method::Refused("Apple"),
                    "passkey" => Method::Refused("a passkey"),
                    "security-key" => Method::Refused("a security key"),
                    other => {
                        return Err(BadCommand(format!(
                            "unknown provider `{other}`; use github, or a password"
                        )));
                    }
                }
            }
            "--secrets" => {
                let kind = value("--secrets")?;
                parsed.secrets = Some(SecretStoreKind::parse(&kind).ok_or_else(|| {
                    BadCommand(format!("--secrets is `file` or `system`, not `{kind}`"))
                })?);
            }
            "--add" | "--replace" => {
                if parsed.binding.is_some() {
                    return Err(BadCommand("pass --add or --replace, not both".into()));
                }
                parsed.binding = Some(if arg == "--add" {
                    Choice::Add
                } else {
                    Choice::Replace
                });
            }
            other => return Err(BadCommand(format!("sign-in doesn't take `{other}`"))),
        }
    }
    if parsed.password_stdin && parsed.method == Method::Github {
        return Err(BadCommand(
            "--password-stdin is for password sign-in, not --provider github".into(),
        ));
    }
    Ok(parsed)
}

/// The command line again, as `parse` reads it: for the round-trip test.
#[cfg(test)]
fn unparse(command: &Command) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    match command {
        Command::Help => out.push("help".into()),
        Command::Status => out.push("status".into()),
        Command::Devices => out.push("devices".into()),
        Command::SignOut => out.push("sign-out".into()),
        Command::Revoke { device_id } => {
            out.push("revoke".into());
            out.push(device_id.clone());
        }
        Command::SignIn(args) => {
            out.push("sign-in".into());
            let mut pair = |flag: &str, value: &Option<String>| {
                if let Some(value) = value {
                    out.push(flag.into());
                    out.push(value.clone());
                }
            };
            pair("--server", &args.server);
            pair("--username", &args.username);
            pair("--device-name", &args.device_name);
            pair(
                "--secrets",
                &args.secrets.map(|kind| kind.as_str().to_owned()),
            );
            let provider = match args.method {
                Method::Password => None,
                Method::Github => Some("github"),
                Method::Refused("Google") => Some("google"),
                Method::Refused("Apple") => Some("apple"),
                Method::Refused("a passkey") => Some("passkey"),
                Method::Refused(_) => Some("security-key"),
            };
            pair("--provider", &provider.map(str::to_owned));
            if args.password_stdin {
                out.push("--password-stdin".into());
            }
            match args.binding {
                Some(Choice::Add) => out.push("--add".into()),
                Some(Choice::Replace) => out.push("--replace".into()),
                _ => {}
            }
        }
    }
    out
}

/// The terminal, as the commands see it: tests pass a scripted one.
pub trait Console {
    /// Whether someone is there to answer a question.
    fn is_tty(&self) -> bool;
    fn prompt(&mut self, line: &str) -> Result<String>;
    /// Asks without showing what is typed.
    fn prompt_secret(&mut self, line: &str) -> Result<Zeroizing<String>>;
    /// The first line of standard input (`--password-stdin`).
    fn stdin_line(&mut self) -> Result<Zeroizing<String>>;
    fn say(&mut self, line: &str);
}

/// The real terminal.
struct Terminal;

// no-prop-test: a thin wrapper over the terminal and `rpassword`.
impl Console for Terminal {
    fn is_tty(&self) -> bool {
        std::io::stdin().is_terminal()
    }

    fn prompt(&mut self, line: &str) -> Result<String> {
        print!("{line}");
        std::io::stdout().flush().ok();
        let mut answer = String::new();
        std::io::stdin().lock().read_line(&mut answer)?;
        Ok(answer.trim_end_matches(['\r', '\n']).to_owned())
    }

    fn prompt_secret(&mut self, line: &str) -> Result<Zeroizing<String>> {
        Ok(Zeroizing::new(rpassword::prompt_password(line)?))
    }

    fn stdin_line(&mut self) -> Result<Zeroizing<String>> {
        let mut line = Zeroizing::new(String::new());
        std::io::stdin().lock().read_line(&mut line)?;
        let trimmed = line.trim_end_matches(['\r', '\n']).len();
        line.truncate(trimmed);
        Ok(line)
    }

    fn say(&mut self, line: &str) {
        println!("{line}");
    }
}

/// A refusal: the words to print after `kanso account: `.
#[derive(Debug)]
pub struct Refusal(pub String);

impl<E: std::fmt::Display> From<E> for Refusal {
    fn from(err: E) -> Refusal {
        Refusal(err.to_string())
    }
}

fn refuse<T>(words: impl Into<String>) -> Result<T, Refusal> {
    Err(Refusal(words.into()))
}

/// What the commands reach the world through: the real ones in `run_cli`,
/// `MemoryServer` and `MemorySecrets` in tests.
pub struct World {
    pub data_dir: PathBuf,
    pub backend: Arc<dyn Backend>,
    /// Opens the secret store of a kind.
    pub open_secrets: Box<dyn Fn(SecretStoreKind) -> Result<Arc<dyn Secrets>>>,
    /// Whether the system's store answers a probe (Linux's default).
    pub system_works: Box<dyn Fn() -> bool>,
    pub machine: Machine,
    pub now_ms: Box<dyn Fn() -> i64>,
    /// How the approval wait pauses between polls.
    pub pause: Box<dyn Fn(std::time::Duration) + Send + Sync>,
}

/// `kanso account …`. Prints its own errors and exits with 1 for a refusal
/// and 2 for a bad command line.
pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    let command = match parse(rest) {
        Ok(command) => command,
        Err(BadCommand(words)) => {
            eprintln!("kanso account: {words}\n\n{USAGE}");
            std::process::exit(BAD_COMMAND);
        }
    };
    let data_dir_owned = data_dir.to_path_buf();
    let world = World {
        data_dir: data_dir.to_path_buf(),
        backend: Arc::new(kanso_sync::HttpBackend::new()),
        open_secrets: Box::new(move |kind| {
            let secrets: Arc<dyn Secrets> = crate::secrets::open_kind(kind, &data_dir_owned)?;
            Ok(secrets)
        }),
        system_works: Box::new(|| crate::secrets::store_works(&crate::flavour::CURRENT.keychain())),
        machine: Machine::this_mac(),
        now_ms: Box::new(kanso_core::now_ms),
        pause: Box::new(std::thread::sleep),
    };
    match run(&world, command, &mut Terminal) {
        Ok(()) => Ok(()),
        Err(Refusal(words)) => {
            eprintln!("kanso account: {words}");
            std::process::exit(REFUSED);
        }
    }
}

/// Runs `command` in `world`, talking through `console`.
pub fn run(world: &World, command: Command, console: &mut dyn Console) -> Result<(), Refusal> {
    match command {
        Command::Help => {
            console.say(USAGE);
            Ok(())
        }
        Command::SignIn(args) => sign_in(world, args, console),
        Command::Status => status(world, console),
        Command::SignOut => sign_out(world, console),
        Command::Devices => devices(world, console),
        Command::Revoke { device_id } => revoke(world, &device_id, console),
    }
}

/// Takes the data-folder lock, or says who holds it.
fn lock(data_dir: &Path) -> Result<kanso_store::DataDirLock, Refusal> {
    std::fs::create_dir_all(data_dir)?;
    match lock_data_dir(data_dir)? {
        Some(lock) => Ok(lock),
        None => refuse(format!(
            "Kanso is running with {}; stop it first (systemctl stop kanso, or quit the app).",
            data_dir.display()
        )),
    }
}

/// The config, brought up to date and with API credentials, saved when that
/// changed it.
fn config(data_dir: &Path) -> Result<AppConfig, Refusal> {
    crate::load_config(data_dir).map_err(|err| Refusal(format!("{err:#}")))
}

fn save_config(data_dir: &Path, config: &AppConfig) -> Result<(), Refusal> {
    config
        .save(&crate::paths::config_path(data_dir))
        .map_err(|err| Refusal(format!("saving the config: {err:#}")))
}

/// The engine's seams for this library.
fn flow(world: &World, store: Arc<Store>, secrets: Arc<dyn Secrets>) -> Flow {
    Flow {
        backend: world.backend.clone(),
        secrets,
        store,
        machine: Some(world.machine.clone()),
        backups: crate::paths::backups_dir(&world.data_dir),
        now_ms: (world.now_ms)(),
    }
}

/// The signed-in session the config, the binding and the secret store agree
/// on, with the library's ID.
fn current_session(flow: &Flow, config: &AppConfig) -> Result<Option<Session>, Refusal> {
    let Some(account) = &config.sync.account else {
        return Ok(None);
    };
    let Some(restored) = flow.restore(Some((&account.origin, &account.id))) else {
        return refuse("couldn't read this library's device identity");
    };
    match restored.loaded {
        Ok(session) => Ok(session),
        Err(err) => refuse(format!("reading the session: {err}")),
    }
}

/// `sign-in`.
fn sign_in(world: &World, args: SignInArgs, console: &mut dyn Console) -> Result<(), Refusal> {
    if let Method::Refused(what) = args.method {
        return refuse(format!(
            "this sign-in needs a browser on this computer ({what}). Use a password or --provider github."
        ));
    }
    let _lock = lock(&world.data_dir)?;
    let mut config = config(&world.data_dir)?;
    let server = args
        .server
        .clone()
        .or_else(|| config.sync.server.clone())
        .unwrap_or_else(|| kanso_sync::DEFAULT_SERVER.to_owned());
    let server = kanso_sync::check_endpoint(&server)
        .map_err(|_| Refusal(format!("{server} isn't a server address")))?;
    let kind = match args.secrets {
        Some(kind) => kind,
        None => crate::secrets::default_kind(&config, &world.system_works),
    };
    let secrets = (world.open_secrets)(kind).map_err(|err| Refusal(format!("{err:#}")))?;
    if kind == SecretStoreKind::System
        && args.secrets.is_none()
        && cfg!(target_os = "linux")
        && !crate::secrets::store_works(secrets.as_ref())
    {
        return refuse(
            "the Secret Service isn't available here. Pass --secrets file to keep the session in an owner-only file, or set one up.",
        );
    }
    let store = Arc::new(Store::open(crate::paths::db_path(&world.data_dir))?);
    let mut machine = world.machine.clone();
    if let Some(name) = &args.device_name {
        machine.computer_name = name.clone();
    }
    let flow = Flow {
        machine: Some(machine),
        ..flow(world, store, secrets)
    };
    if let Some(session) = current_session(&flow, &config)? {
        return refuse(format!(
            "this library is signed in as {} already; run `kanso account sign-out` first.",
            session.username
        ));
    }
    console.say(&format!("Kanso account on {server}"));
    let (session, library_id, backup, evicted) = match args.method {
        Method::Github => sign_in_github(&flow, &server, &args, &*world.pause, console)?,
        _ => sign_in_password(&flow, &server, &args, console)?,
    };
    // The config last: the session and the binding are saved already, so
    // the config never names an account the library can't open.
    config.sync.account = Some(AccountRef {
        origin: session.origin.clone(),
        id: session.account_id.clone(),
        username: session.username.clone(),
    });
    config.sync.enabled = true;
    config.sync.server =
        Some(session.server.clone()).filter(|s| !kanso_engine::account::is_default_server(s));
    config.sync.secret_store = kind;
    save_config(&world.data_dir, &config)?;
    flow.flush_logouts(&library_id);
    if let Some(evicted) = evicted {
        console.say(&format!(
            "Signing in here signed out {}.",
            evicted.name.as_deref().unwrap_or("another device")
        ));
    }
    if let Some(backup) = backup {
        console.say(&format!(
            "This computer's notes were backed up to {} first.",
            backup.display()
        ));
    }
    let device = flow
        .device()
        .map(|device| device.name)
        .unwrap_or_default();
    console.say(&format!(
        "Signed in as {}. This computer is \"{device}\" in your devices.",
        session.username
    ));
    if kind == SecretStoreKind::File {
        console.say(&format!(
            "Secrets are in {} (owner-only).",
            crate::secrets::secrets_path(&world.data_dir).display()
        ));
    }
    console.say("Start `kanso serve` to sync.");
    Ok(())
}

/// A finished sign-in: the session, the library's ID, a Replace's backup,
/// and the device the sign-in signed out to make room.
type SignedIn = (
    Session,
    String,
    Option<PathBuf>,
    Option<kanso_sync::DeviceInfo>,
);

fn sign_in_password(
    flow: &Flow,
    server: &str,
    args: &SignInArgs,
    console: &mut dyn Console,
) -> Result<SignedIn, Refusal> {
    let username = match &args.username {
        Some(username) => username.clone(),
        None if console.is_tty() => console.prompt("Username: ")?.trim().to_owned(),
        None => return refuse("pass --username, or run this on a terminal"),
    };
    if username.is_empty() {
        return refuse("a username is needed");
    }
    let typed = if args.password_stdin {
        console.stdin_line()?
    } else if console.is_tty() {
        console.prompt_secret("Password: ")?
    } else {
        return refuse("no terminal to ask for the password; pass --password-stdin");
    };
    let Some(password) = Password::for_sign_in(&typed) else {
        return refuse("a password is needed");
    };
    let authenticated = {
        let server = server.to_owned();
        let username = username.clone();
        let said = std::cell::RefCell::new(&mut *console);
        flow.authenticate(
            Box::new(move |flow, device, step| {
                flow.sign_in_password(&server, &username, &password, device, step)
            }),
            &|step| said.borrow_mut().say(words::step_label(step)),
        )
    };
    let authenticated = authenticated.map_err(|failure| Refusal(failure.words))?;
    let choice = binding_choice(
        after_auth(
            kanso_sync::Flow::SignIn,
            &authenticated.library,
            &authenticated.session.username,
            console.is_tty() || args.binding.is_some(),
        ),
        args.binding,
        console,
    );
    let choice = match choice {
        Ok(choice) => choice,
        Err(refusal) => {
            flow.abandon(&authenticated.session);
            return Err(refusal);
        }
    };
    console.say(words::step_label(kanso_sync::Step::Saving));
    let (session, library_id, backup) = flow
        .bind_and_save(
            authenticated.session,
            authenticated.library_id,
            choice,
            None,
        )
        .map_err(|failure| Refusal(failure.words))?;
    Ok((session, library_id, backup, authenticated.evicted))
}

/// The answer to the binding question: asked on the terminal, taken from
/// `--add` or `--replace`, or refused.
fn binding_choice(
    next: Next,
    flag: Option<Choice>,
    console: &mut dyn Console,
) -> Result<Option<Choice>, Refusal> {
    match next {
        Next::Bind(choice) => Ok(choice),
        Next::Abandon => refuse("this computer already has notes; pass --add or --replace."),
        Next::Ask(question) => ask_binding(&question, flag, console),
    }
}

fn ask_binding(
    question: &words::Question,
    flag: Option<Choice>,
    console: &mut dyn Console,
) -> Result<Option<Choice>, Refusal> {
    if let Some(choice) = flag {
        return Ok(Some(choice));
    }
    if !console.is_tty() {
        return refuse("this computer already has notes; pass --add or --replace.");
    }
    console.say(&question.title.replace("device", "computer"));
    console.say(&question.body.replace("device", "computer"));
    for answer in &question.answers {
        let key = match answer.choice {
            Choice::Add => "a",
            Choice::Replace => "r",
            Choice::Cancel => "c",
        };
        console.say(&format!(
            "  [{key}] {}: {}",
            answer.label, answer.description
        ));
    }
    console.say("  [c] Cancel");
    loop {
        match console.prompt("Choice: ")?.trim().to_ascii_lowercase().as_str() {
            "a" => return Ok(Some(Choice::Add)),
            "r" => return Ok(Some(Choice::Replace)),
            "c" | "" => return refuse("cancelled; nothing changed."),
            _ => console.say("Type a, r or c."),
        }
    }
}

/// GitHub's device flow needs no browser here: the page and the code are
/// printed, for any device. Google's loopback redirect does, so it fails.
struct Headless;

impl Browser for Headless {
    fn listen(&self) -> Result<Box<dyn Listener>, BrowserError> {
        Err(BrowserError::Failed)
    }

    fn open_google(&self, _: &GoogleStart) -> Result<(), BrowserError> {
        Err(BrowserError::Failed)
    }

    fn open_github(&self, _: &GithubStart) -> Result<(), BrowserError> {
        // The caller prints the page and the code.
        Ok(())
    }
}

/// GitHub, then this computer's approval by another signed-in device (or
/// the account's password).
fn sign_in_github(
    flow: &Flow,
    server: &str,
    args: &SignInArgs,
    pause: &dyn Fn(std::time::Duration),
    console: &mut dyn Console,
) -> Result<SignedIn, Refusal> {
    let provider = Provider::Github;
    let login_device = LoginDevice::ephemeral();
    let started = flow
        .start_provider(&Headless, server, &provider, &login_device, &|_| {})
        .map_err(|failure| Refusal(failure.words))?;
    match &started.start {
        kanso_sync::ProviderStart::Github(github) => console.say(&format!(
            "Open {} on any device and enter {}.",
            github.verification_uri, github.user_code
        )),
        _ => return refuse("the server didn't start a GitHub sign-in"),
    }
    console.say("Waiting for GitHub…");
    let cancel = Arc::new(AtomicBool::new(false));
    let finished = flow
        .finish_provider(
            &Headless,
            server,
            &provider,
            &login_device,
            started,
            None,
            cancel,
            &|_| {},
        )
        .map_err(|failure| Refusal(failure.words))?;
    let (locked, device) = match finished {
        ProviderTaskResult::Locked(locked, device) => (*locked, device),
        ProviderTaskResult::Cancelled => return refuse("GitHub sign-in was cancelled."),
        ProviderTaskResult::NewAccount(_) => {
            return refuse(
                "there's no Kanso account for that GitHub login. Create it in Kanso on a computer first.",
            );
        }
    };
    console.say(&format!(
        "Signed in to {}, but this computer doesn't have your notes' key yet.",
        locked.username
    ));
    let candidate = match unlock(flow, &locked, &device, args, pause, console) {
        Ok(candidate) => candidate,
        Err(refusal) => {
            flow.discard_locked(&locked);
            return Err(refusal);
        }
    };
    let next = after_unlock(
        &match flow.library(&candidate.session.origin, &candidate.session.account_id) {
            Ok(library) => library,
            Err(failure) => {
                flow.discard_locked(&locked);
                return refuse(failure.words);
            }
        },
        &candidate.session.username,
        console.is_tty() || args.binding.is_some(),
    );
    let choice = match next {
        NextUnlock::Promote(choice) => Ok(choice),
        NextUnlock::Ask(question) => ask_binding(&question, args.binding, console),
        NextUnlock::Park => refuse("this computer already has notes; pass --add or --replace."),
        NextUnlock::Reject => refuse("this sign-in can't be used for this library."),
    };
    let choice = match choice {
        Ok(choice) => choice,
        Err(refusal) => {
            flow.discard_locked(&locked);
            return Err(refusal);
        }
    };
    console.say(words::step_label(kanso_sync::Step::Saving));
    let UnlockedCandidate {
        session,
        library_id,
        evicted,
        same_token_as_locked,
        ..
    } = candidate;
    let (session, library_id) = flow
        .promote(
            session,
            library_id,
            same_token_as_locked,
            choice,
            Some(&locked),
        )
        .map_err(|failure| Refusal(failure.words))?;
    let backup = (choice == Some(Choice::Replace))
        .then(|| newest_backup(&crate::paths::backups_dir(&flow_data_dir(flow))))
        .flatten();
    Ok((session, library_id, backup, evicted))
}

/// The data folder a flow's backups folder sits in.
fn flow_data_dir(flow: &Flow) -> PathBuf {
    flow.backups
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// The newest backup in `dir`, for the line after a Replace.
fn newest_backup(dir: &Path) -> Option<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths.pop()
}

/// The data key for a locked GitHub sign-in: the account's password, when
/// it has one and it's typed, else another device's approval.
fn unlock(
    flow: &Flow,
    locked: &kanso_sync::LockedSession,
    device: &DeviceIdentity,
    args: &SignInArgs,
    pause: &dyn Fn(std::time::Duration),
    console: &mut dyn Console,
) -> Result<UnlockedCandidate, Refusal> {
    if locked.has_password && console.is_tty() && !args.password_stdin {
        let typed = console.prompt_secret(
            "This account has a password. Type it to unlock this computer, or press Enter to wait for approval: ",
        )?;
        if let Some(password) = Password::for_sign_in(&typed) {
            return flow
                .unlock_with_password(locked, &password, &|_| {})
                .map_err(|failure| Refusal(failure.words));
        }
    }
    console.say("Approve this computer in Kanso on a device that's already signed in.");
    console.say("Waiting for another device…");
    let waited = {
        let said = std::cell::RefCell::new(&mut *console);
        flow.wait_for_approval(
            locked,
            device,
            None,
            &|update| {
                if let WaitUpdate::Code(code) = update {
                    said.borrow_mut().say(&format!(
                        "It should show {}. Only approve there if it does.",
                        spaced_code(&code)
                    ));
                }
            },
            pause,
        )
    };
    match waited {
        Ok(candidate) => {
            console.say("Approved.");
            Ok(candidate)
        }
        Err(WaitEnd::NoDevices) => refuse(
            "no other device is signed in to approve this one. Sign in with a password, or on a computer first.",
        ),
        Err(WaitEnd::Cancelled) => refuse("the approval was cancelled."),
        Err(WaitEnd::Expired) => refuse("the approval request expired; sign in again."),
        Err(WaitEnd::Failed(failure)) => refuse(failure.words),
    }
}

/// A six-digit code as two groups of three, the way the approving device
/// shows it. Pure.
pub fn spaced_code(code: &str) -> String {
    let digits: Vec<char> = code.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() == 6 {
        let (a, b) = digits.split_at(3);
        format!("{} {}", a.iter().collect::<String>(), b.iter().collect::<String>())
    } else {
        code.to_owned()
    }
}

/// `status`: reads without the lock, so it works while `kanso serve` runs.
fn status(world: &World, console: &mut dyn Console) -> Result<(), Refusal> {
    let config = AppConfig::load(&crate::paths::config_path(&world.data_dir))
        .map_err(|err| Refusal(format!("{err:#}")))?;
    let db = crate::paths::db_path(&world.data_dir);
    let store = if db.exists() {
        Some(Store::open_read_only(&db)?)
    } else {
        None
    };
    let meta = |key: &str| {
        store
            .as_ref()
            .and_then(|store| store.meta(key).ok().flatten())
    };
    let bound = meta(meta_keys::ACCOUNT_ID);
    let Some(account) = &config.sync.account else {
        console.say(if bound.is_some() {
            "Signed out"
        } else {
            "Not signed in"
        });
        return Ok(());
    };
    let library_id = meta(meta_keys::LIBRARY_ID);
    let has_session = match &library_id {
        Some(library_id) => (world.open_secrets)(config.sync.secret_store)
            .ok()
            .and_then(|secrets| {
                secrets
                    .get(&kanso_sync::keychain::account_item(library_id))
                    .ok()
            })
            .flatten()
            .is_some(),
        None => false,
    };
    if !has_session || bound.as_deref() != Some(account.id.as_str()) {
        console.say(&format!("Signed out (was {})", account.username));
        return Ok(());
    }
    let device_id = match (&library_id, meta(meta_keys::DEVICE_SECRET)) {
        (Some(library_id), Some(secret)) => parse_hex32(&secret)
            .map(|secret| DeviceIdentity::from_parts(library_id, "", secret).device_id(&account.id)),
        _ => None,
    };
    let server = meta(meta_keys::SERVER).unwrap_or_else(|| account.origin.clone());
    console.say(&format!(
        "Account   {} ({})",
        account.username,
        short_id(&account.id)
    ));
    console.say(&format!("Server    {server}"));
    console.say(&format!(
        "Device    {} ({})",
        world.machine.computer_name.trim(),
        device_id.as_deref().map(short_id).unwrap_or("-".into())
    ));
    console.say(&match config.sync.secret_store {
        SecretStoreKind::File => format!(
            "Secrets   file, {}",
            crate::secrets::secrets_path(&world.data_dir).display()
        ),
        SecretStoreKind::System => "Secrets   system".to_owned(),
    });
    console.say(&if config.sync.enabled {
        format!(
            "Sync      on, {}",
            kanso_engine::sync::words::sync_every(
                kanso_engine::sync::interval(config.sync.interval_secs).as_secs() / 60
            )
            .to_lowercase()
        )
    } else {
        "Sync      off".to_owned()
    });
    let last = meta(kanso_engine::sync::driver::LAST_SYNC_MS)
        .and_then(|ms| ms.parse::<i64>().ok())
        .zip(
            meta(kanso_engine::sync::driver::LAST_RESULT)
                .and_then(|json| serde_json::from_str::<kanso_engine::sync::driver::LastResult>(&json).ok()),
        );
    console.say(&match last {
        Some((at, result)) => format!(
            "Last sync {}: {}",
            ago((world.now_ms)(), at),
            last_result_words(&result)
        ),
        None => "Last sync never, from this command line".to_owned(),
    });
    Ok(())
}

/// What a last sync did, in a line.
fn last_result_words(result: &kanso_engine::sync::driver::LastResult) -> String {
    let mut line = format!(
        "{} pulled, {} pushed, {} pending",
        result.pulled, result.pushed, result.pending
    );
    if let Some(problem) = &result.problem {
        line.push_str(&format!(" ({problem})"));
    }
    line
}

/// An ID shortened for a table: its first 6 characters and an ellipsis.
fn short_id(id: &str) -> String {
    match id.char_indices().nth(6) {
        Some((at, _)) => format!("{}…", &id[..at]),
        None => id.to_owned(),
    }
}

fn parse_hex32(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(out)
}

/// How long ago `then_ms` was, at `now_ms`, in a few words. Pure.
pub fn ago(now_ms: i64, then_ms: i64) -> String {
    let secs = (now_ms - then_ms).max(0) / 1000;
    match secs {
        0..60 => "just now".to_owned(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86_400 => match secs / 3600 {
            1 => "1 hour ago".to_owned(),
            hours => format!("{hours} hours ago"),
        },
        86_400..172_800 => "yesterday".to_owned(),
        _ => format!("{} days ago", secs / 86_400),
    }
}

/// The signed-in session for a command that uses it, with its flow, under
/// the lock.
fn signed_in(world: &World) -> Result<(Flow, AppConfig, Session), Refusal> {
    let config = config(&world.data_dir)?;
    let secrets = (world.open_secrets)(config.sync.secret_store)
        .map_err(|err| Refusal(format!("{err:#}")))?;
    let store = Arc::new(Store::open(crate::paths::db_path(&world.data_dir))?);
    let flow = flow(world, store, secrets);
    match current_session(&flow, &config)? {
        Some(session) => Ok((flow, config, session)),
        None => refuse("this library isn't signed in; run `kanso account sign-in`."),
    }
}

/// `sign-out`.
fn sign_out(world: &World, console: &mut dyn Console) -> Result<(), Refusal> {
    let _lock = lock(&world.data_dir)?;
    let (flow, mut config, session) = signed_in(world)?;
    if let Err(err) = flow.sign_out(&session) {
        // The secret items are gone and the logout is queued whatever the
        // server said; only a store that refused is worth saying.
        if matches!(err, kanso_sync::AccountError::Keychain(_)) {
            return refuse(format!("{err}"));
        }
    }
    config.sync.account = None;
    save_config(&world.data_dir, &config)?;
    console.say(
        "Signed out. Your notes stay on this computer; nothing syncs until you sign in again.",
    );
    Ok(())
}

/// `devices`.
fn devices(world: &World, console: &mut dyn Console) -> Result<(), Refusal> {
    let _lock = lock(&world.data_dir)?;
    let (flow, _, session) = signed_in(world)?;
    let devices = kanso_sync::devices(flow.backend.as_ref(), &session)
        .map_err(|err| Refusal(words::error_words(&err, words::ErrorContext::Account)))?;
    let now = (world.now_ms)();
    for line in device_lines(&devices, now) {
        console.say(&line);
    }
    Ok(())
}

/// The devices table: ID, name, this computer, last seen. Pure.
pub fn device_lines(devices: &[kanso_sync::DeviceInfo], now_ms: i64) -> Vec<String> {
    let width = devices
        .iter()
        .map(|d| device_label(d).chars().count())
        .max()
        .unwrap_or(0);
    devices
        .iter()
        .map(|device| {
            let name = device_label(device);
            let pad = " ".repeat(width - name.chars().count());
            let here = if device.current {
                "this computer"
            } else {
                "             "
            };
            format!(
                "{}  {name}{pad}  {here}  seen {}",
                short_id(&device.id),
                ago(now_ms, device.last_seen_at)
            )
        })
        .collect()
}

fn device_label(device: &kanso_sync::DeviceInfo) -> &str {
    device.name.as_deref().unwrap_or("Unnamed device")
}

/// The one device `wanted` names: by its ID, or a unique start of one (an
/// ellipsis from the table is ignored). Pure.
pub fn pick_device<'a>(
    devices: &'a [kanso_sync::DeviceInfo],
    wanted: &str,
) -> Result<&'a kanso_sync::DeviceInfo, String> {
    let wanted = wanted.trim().trim_end_matches('…');
    if let Some(device) = devices.iter().find(|d| d.id == wanted) {
        return Ok(device);
    }
    let matching: Vec<_> = devices
        .iter()
        .filter(|d| !wanted.is_empty() && d.id.starts_with(wanted))
        .collect();
    match matching.as_slice() {
        [device] => Ok(device),
        [] => Err(format!("no device's ID starts with {wanted}")),
        _ => Err(format!(
            "several devices' IDs start with {wanted}; type more of it"
        )),
    }
}

/// `revoke <id>`.
fn revoke(world: &World, wanted: &str, console: &mut dyn Console) -> Result<(), Refusal> {
    let _lock = lock(&world.data_dir)?;
    let (flow, _, session) = signed_in(world)?;
    let devices = kanso_sync::devices(flow.backend.as_ref(), &session)
        .map_err(|err| Refusal(words::error_words(&err, words::ErrorContext::Account)))?;
    let device = pick_device(&devices, wanted).map_err(Refusal)?;
    if device.current {
        return refuse("that's this computer; use `kanso account sign-out`.");
    }
    flow.revoke_device(&session, &device.id)
        .map_err(|failure| Refusal(failure.words))?;
    console.say(&format!("Signed {} out.", device_label(device)));
    Ok(())
}

#[cfg(test)]
#[path = "account_cli_tests.rs"]
mod tests;
