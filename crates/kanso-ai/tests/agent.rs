//! An agent on this computer answering through `ChatProvider`, against the
//! fake ACP agent in process.

#![cfg(feature = "acp")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use kanso_acp::fake::{self, Script};
use kanso_acp::{AcpError, AgentProcess, Limits};
use kanso_ai::agent::{self, AgentCheck, AgentKind, AgentSettings, Starter};
use kanso_ai::{Event, Prompt, ToolRegistry};

/// The app keeps one agent at a time, so these tests take turns.
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn settings(name: &str) -> AgentSettings {
    let workspace = std::env::temp_dir().join(format!("kanso-ai-agent-test-{name}"));
    AgentSettings {
        kind: AgentKind::Custom,
        // Any existing file passes the custom command check; the starter
        // below connects the fake instead of running it.
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: Vec::new(),
        env: Vec::new(),
        workspace,
    }
}

fn limits() -> Limits {
    Limits {
        cancel_grace: Duration::from_millis(500),
        ..Limits::default()
    }
}

/// Starts the fake with `script`, counting starts.
fn starter(script: Script, starts: Arc<AtomicUsize>) -> Starter {
    Arc::new(move |spec, limits| {
        starts.fetch_add(1, Ordering::Relaxed);
        let (from_agent, to_agent) =
            fake::in_process(script.clone()).map_err(|err| AcpError::Spawn(err.to_string()))?;
        AgentProcess::connect(
            &spec.label,
            from_agent,
            to_agent,
            limits,
            kanso_acp::reject_all(),
        )
    })
}

fn failing(starts: Arc<AtomicUsize>) -> Starter {
    Arc::new(move |_, _| {
        starts.fetch_add(1, Ordering::Relaxed);
        Err(AcpError::Spawn("no such file".to_owned()))
    })
}

fn ask(provider: &kanso_ai::SharedProvider, question: &str) -> Vec<Event> {
    let provider = provider.lock().unwrap();
    let prompt = Prompt {
        system: "Reply.".to_owned(),
        user: question.to_owned(),
    };
    provider
        .ask(
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            Arc::new(AtomicBool::new(false)),
            &kanso_ai::Budget::default(),
        )
        .unwrap()
        .collect()
}

fn text(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_agent_answers_and_stays_warm_for_the_next_question() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("warm");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::Answer("From your notes.".into()), starts.clone()),
    );
    let first = ask(&provider, "one");
    assert_eq!(
        first.first(),
        Some(&Event::Status(format!("Starting {}…", settings.label())))
    );
    assert_eq!(text(&first), "From your notes.");
    assert_eq!(first.last(), Some(&Event::Done));
    let second = ask(&provider, "two");
    assert!(!second.iter().any(|event| matches!(event, Event::Status(_))));
    assert_eq!(text(&second), "From your notes.");
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn stop_ends_the_answer_and_the_agent_answers_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("stop");
    let provider =
        agent::agent_client_with(&settings, limits(), starter(Script::Hang, starts.clone()));
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let provider = provider.lock().unwrap();
        let prompt = Prompt {
            system: "Reply.".into(),
            user: "x".into(),
        };
        let mut events = provider
            .ask(
                &prompt,
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                cancel.clone(),
                &kanso_ai::Budget::default(),
            )
            .unwrap();
        assert!(matches!(events.next(), Some(Event::Status(_))));
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(events.next(), None);
    }
    // The fake stopped on cancel, so the same process takes the next
    // question, which is stopped too.
    let cancel = Arc::new(AtomicBool::new(false));
    let stopper = {
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            cancel.store(true, Ordering::Relaxed);
        })
    };
    let events: Vec<Event> = {
        let provider = provider.lock().unwrap();
        let prompt = Prompt {
            system: "Reply.".into(),
            user: "again".into(),
        };
        provider
            .ask(
                &prompt,
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                cancel,
                &kanso_ai::Budget::default(),
            )
            .unwrap()
            .collect()
    };
    stopper.join().unwrap();
    assert!(
        events.iter().all(|event| !matches!(event, Event::Error(_))),
        "{events:?}"
    );
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn a_crash_is_reported_and_the_next_question_starts_it_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("crash");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        // The in-process fake can't exit, so a crash is a bad line instead.
        starter(Script::BadJson, starts.clone()),
    );
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("couldn't read"), "{message}");
    ask(&provider, "y");
    assert_eq!(starts.load(Ordering::Relaxed), 2);
}

#[test]
fn three_failed_starts_stop_the_tries_until_find_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("fails");
    let provider = agent::agent_client_with(&settings, limits(), failing(starts.clone()));
    for _ in 0..3 {
        let events = ask(&provider, "x");
        let Some(Event::Error(message)) = events.last() else {
            panic!("no error: {events:?}");
        };
        assert!(message.contains("didn't start"), "{message}");
    }
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("Find again"), "{message}");
    assert_eq!(starts.load(Ordering::Relaxed), 3);
}

#[test]
fn test_reports_the_agent_and_its_answer() {
    let _serial = serial();
    let settings = settings("check");
    let check = agent::check_with(
        &settings,
        limits(),
        starter(Script::Answer("OK".into()), Arc::new(AtomicUsize::new(0))),
    )
    .unwrap();
    assert_eq!(
        check,
        AgentCheck::Ready {
            name: "Fake Agent".into(),
            version: Some("1.2.3".into()),
            said: "OK".into(),
        }
    );
}

#[test]
fn test_asks_for_a_sign_in_and_signing_in_clears_it() {
    let _serial = serial();
    let settings = settings("auth");
    let starts = Arc::new(AtomicUsize::new(0));
    let start = starter(Script::Auth, starts.clone());
    let check = agent::check_with(&settings, limits(), start.clone()).unwrap();
    assert_eq!(
        check,
        AgentCheck::SignIn {
            methods: vec![("fake-login".into(), "Log in".into())]
        }
    );
    agent::sign_in_with(&settings, limits(), start.clone(), "fake-login").unwrap();
    let check = agent::check_with(&settings, limits(), start).unwrap();
    assert!(matches!(check, AgentCheck::Ready { .. }), "{check:?}");
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn a_question_needing_a_sign_in_says_how() {
    let _serial = serial();
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        ..settings("auth-ask")
    };
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::Auth, Arc::new(AtomicUsize::new(0))),
    );
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("needs you to sign in"), "{message}");
    assert!(message.contains("Settings › AI"), "{message}");
}

/// The manual check against a real agent installed on this computer:
/// `KANSO_REAL_AGENT=opencode cargo test -p kanso-ai --test agent real -- --ignored`
/// (claudeAgent, codex, geminiCli or opencode). It asks "Reply with OK." and
/// uses a little of that agent's subscription or quota.
#[test]
#[ignore = "starts a real agent; run on demand with KANSO_REAL_AGENT set"]
fn real_agent_answers_the_test_question() {
    let _serial = serial();
    let kind = match std::env::var("KANSO_REAL_AGENT").as_deref() {
        Ok("claudeAgent") => AgentKind::ClaudeAgent,
        Ok("codex") => AgentKind::Codex,
        Ok("geminiCli") => AgentKind::GeminiCli,
        Ok("opencode") => AgentKind::OpenCode,
        other => panic!("set KANSO_REAL_AGENT, not {other:?}"),
    };
    let settings = AgentSettings {
        kind,
        ..settings("real")
    };
    let check = agent::check(&settings);
    eprintln!("{check:?}");
    agent::shutdown();
    assert!(check.is_ok(), "{check:?}");
}

/// A notes grant with a known token, as the app would hand one out.
struct FakeLease;

impl agent::NotesLease for FakeLease {
    fn is_live(&self) -> bool {
        true
    }
    fn renew(&self) -> bool {
        true
    }
    fn begin_turn(&self) {}
    fn reads(&self) -> Vec<String> {
        Vec::new()
    }
}

fn grant_with(token: String) -> agent::NotesAccess {
    Arc::new(move || {
        Some(agent::NotesGrant {
            url: "http://127.0.0.1:9/mcp".to_owned(),
            token: token.clone(),
            relay: None,
            lease: Box::new(FakeLease),
            suggest: false,
        })
    })
}

/// Asks one question in a conversation and waits for its events to end.
fn converse(conversation: &Arc<agent::Conversation>, user: &str) -> Vec<Event> {
    let prompt = Prompt {
        system: "Reply.".to_owned(),
        user: user.to_owned(),
    };
    conversation
        .ask(
            &prompt,
            &[],
            Box::new(|_| {}),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .collect()
}

/// The report once no turn holds the agent any more (a turn lets go just
/// after its last event).
fn settled_report(settings: &AgentSettings) -> String {
    for _ in 0..200 {
        let report = agent::diagnostics(settings, "9.9.9");
        if !report.contains("busy answering") {
            return report;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("still busy");
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(12))]

    /// Whatever the question, the answer, the notes and the token say, the
    /// report holds none of it, nor the home folder.
    #[test]
    fn prop_diagnostics_never_hold_content_tokens_or_the_home_folder(
        question in "[a-z]{14}",
        answer in "[a-z]{14}",
        title in "[a-z]{14}",
        body in "[a-z]{14}",
        token in "[a-z0-9]{24}",
        case in 0u32..1_000_000,
    ) {
        let _serial = serial();
        let (question, answer, title, body) = (
            format!("qq{question}"),
            format!("aa{answer}"),
            format!("tt{title}"),
            format!("bb{body}"),
        );
        let token = format!("kmcp-{token}");
        let settings = settings(&format!("diag-{case}"));
        let conversation = agent::conversation_with(
            &settings,
            limits(),
            starter(Script::Answer(answer.clone()), Arc::new(AtomicUsize::new(0))),
            Some(grant_with(token.clone())),
        );
        let user = format!("<note title=\"{title}\">{body}</note>\nQuestion: {question}");
        let events = converse(&conversation, &user);
        proptest::prop_assert_eq!(text(&events), answer.clone());
        let report = settled_report(&settings);
        for secret in [&question, &answer, &title, &body, &token] {
            proptest::prop_assert!(!report.contains(secret.as_str()), "{secret} in:\n{report}");
        }
        if home().len() > 1 {
            proptest::prop_assert!(!report.contains(&home()), "home in:\n{report}");
        }
        proptest::prop_assert!(report.contains("Process: running"), "{report}");
        proptest::prop_assert!(report.contains("Agent says: Fake Agent 1.2.3"), "{report}");
        proptest::prop_assert!(report.contains("session/prompt"), "{report}");
    }
}

#[test]
fn a_report_before_any_process_has_run_says_so_and_names_the_computer() {
    let _serial = serial();
    let settings = settings("diag-none");
    let report = agent::diagnostics(&settings, "9.9.9");
    assert!(report.starts_with("Kanso 9.9.9 on "), "{report}");
    assert!(report.contains(std::env::consts::ARCH), "{report}");
    assert!(report.contains("(Custom)"), "{report}");
    assert!(report.contains("Detection: found at "), "{report}");
    assert!(report.contains("Process: none has run"), "{report}");
    if home().len() > 1 {
        assert!(!report.contains(&home()), "{report}");
    }
    let missing = AgentSettings {
        command: Some("/no/such/agent".to_owned()),
        ..settings
    };
    let report = agent::diagnostics(&missing, "9.9.9");
    assert!(report.contains("Detection: not installed"), "{report}");
}

#[test]
fn a_report_after_a_crash_keeps_the_dead_process_and_its_log() {
    let _serial = serial();
    let settings = settings("diag-crash");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::BadJson, Arc::new(AtomicUsize::new(0))),
    );
    ask(&provider, "x");
    let mut report = settled_report(&settings);
    for _ in 0..200 {
        if report.contains("not running") {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
        report = settled_report(&settings);
    }
    eprintln!("{report}");
    assert!(report.contains("Process: not running"), "{report}");
    assert!(report.contains("Agent says: Fake Agent 1.2.3"), "{report}");
    assert!(report.contains("ended: "), "{report}");
    assert!(
        report.contains("Failed starts in the last minute: 0; starts allowed"),
        "{report}"
    );
}

#[test]
fn a_report_while_a_question_is_running_does_not_wait() {
    let _serial = serial();
    let settings = settings("diag-busy");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::Hang, Arc::new(AtomicUsize::new(0))),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let provider = provider.lock().unwrap();
    let prompt = Prompt {
        system: "Reply.".into(),
        user: "x".into(),
    };
    let mut events = provider
        .ask(
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            cancel.clone(),
            &kanso_ai::Budget::default(),
        )
        .unwrap();
    assert!(matches!(events.next(), Some(Event::Status(_))));
    let began = std::time::Instant::now();
    let report = agent::diagnostics(&settings, "9.9.9");
    assert!(began.elapsed() < Duration::from_secs(1));
    assert!(
        report.contains("Process: busy answering a question"),
        "{report}"
    );
    assert!(report.contains("Detection: found at "), "{report}");
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(events.next(), None);
}

#[test]
fn a_report_after_three_failed_starts_says_starts_are_blocked() {
    let _serial = serial();
    let settings = settings("diag-blocked");
    let provider =
        agent::agent_client_with(&settings, limits(), failing(Arc::new(AtomicUsize::new(0))));
    for _ in 0..3 {
        ask(&provider, "x");
    }
    let report = settled_report(&settings);
    assert!(
        report.contains("Failed starts in the last minute: 3; starts blocked until Find again"),
        "{report}"
    );
}
