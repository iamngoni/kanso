//! The manual matrix of issue #46 step 8, against the agents installed on
//! this computer. Ignored by default: each run uses a little of the
//! person's subscription. Run one agent at a time:
//!
//! ```sh
//! KANSO_REAL_AGENT=opencode cargo test -p kanso-ai --test real_agents -- --ignored --test-threads 1 --nocapture
//! ```
//!
//! `KANSO_REAL_AGENT` is `claudeAgent`, `codex`, `geminiCli` or `opencode`.

#![cfg(feature = "acp")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use kanso_ai::agent::{self, AgentCheck, AgentKind, AgentSettings};
use kanso_ai::{Event, Prompt};

fn kind() -> AgentKind {
    match std::env::var("KANSO_REAL_AGENT").as_deref() {
        Ok("claudeAgent") => AgentKind::ClaudeAgent,
        Ok("codex") => AgentKind::Codex,
        Ok("geminiCli") => AgentKind::GeminiCli,
        Ok("opencode") => AgentKind::OpenCode,
        other => panic!("set KANSO_REAL_AGENT, not {other:?}"),
    }
}

fn settings(kind: AgentKind) -> AgentSettings {
    AgentSettings {
        kind,
        command: None,
        args: Vec::new(),
        env: Vec::new(),
        workspace: std::env::temp_dir().join("kanso-real-agent-matrix"),
    }
}

fn ask(settings: &AgentSettings, question: &str) -> Box<dyn Iterator<Item = Event> + Send> {
    let provider = agent::agent_client(settings);
    let provider = provider.lock().unwrap();
    provider
        .stream(&Prompt {
            system: "You are a helpful assistant. Don't use tools.".to_owned(),
            user: question.to_owned(),
        })
        .expect("the question starts")
}

/// Ready (or the sign-in it needs), a full answer, Stop mid-answer keeping
/// what arrived, the agent still answering after Stop, a crash mid-answer
/// reported as an error rather than a hang, and the next question starting
/// it again.
#[test]
#[ignore = "uses a real agent and its subscription"]
fn real_agent_matrix() {
    let kind = kind();
    let settings = settings(kind);
    let label = settings.label();

    // Ready, or signed out.
    let started = Instant::now();
    match agent::check(&settings) {
        Ok(AgentCheck::Ready {
            name,
            version,
            said,
        }) => eprintln!(
            "[{label}] ready: {name} {version:?} said {said:?} in {:?}",
            started.elapsed()
        ),
        Ok(AgentCheck::SignIn { methods }) => {
            eprintln!("[{label}] signed out; sign-in methods: {methods:?}");
            agent::shutdown();
            return;
        }
        Err(err) => panic!("[{label}] check failed: {err}"),
    }

    // A whole answer.
    let mut said = String::new();
    for event in ask(&settings, "Reply with exactly: kanso matrix ok") {
        match event {
            Event::Text(text) => said.push_str(&text),
            Event::Error(err) => panic!("[{label}] answer failed: {err}"),
            Event::Done => break,
            Event::Status(line) => eprintln!("[{label}] status: {line}"),
            Event::Thought(_) | Event::Tool { .. } => {}
        }
    }
    eprintln!("[{label}] answered: {said:?}");
    assert!(!said.trim().is_empty());

    // Stop mid-answer: dropping the stream cancels; what arrived stays.
    let mut partial = String::new();
    let stopped_at = {
        let mut events = ask(&settings, "Count from 1 to 400, one number per line.");
        loop {
            match events.next() {
                Some(Event::Text(text)) => {
                    partial.push_str(&text);
                    if partial.len() > 20 {
                        break Instant::now();
                    }
                }
                Some(Event::Error(err)) => panic!("[{label}] long answer failed: {err}"),
                Some(Event::Done) | None => break Instant::now(),
                Some(Event::Status(_) | Event::Thought(_) | Event::Tool { .. }) => {}
            }
        }
    };
    eprintln!(
        "[{label}] stopped after {:?} chars; cancel took {:?}",
        partial.len(),
        stopped_at.elapsed()
    );
    assert!(!partial.is_empty());

    // Still answers after Stop.
    let after: String = ask(&settings, "Reply with exactly: after stop")
        .filter_map(|event| match event {
            Event::Text(text) => Some(text),
            Event::Error(err) => panic!("[{label}] after Stop failed: {err}"),
            _ => None,
        })
        .collect();
    eprintln!("[{label}] after Stop: {after:?}");
    assert!(!after.trim().is_empty());

    // Crash mid-answer: kill the agent's process while it answers.
    let killed = Arc::new(AtomicBool::new(false));
    let mut events = ask(&settings, "Count from 1 to 400, one number per line.");
    let mut outcome = None;
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline {
        match events.next() {
            Some(Event::Text(_)) if !killed.swap(true, Ordering::Relaxed) => {
                let pattern = match kind {
                    AgentKind::OpenCode => "opencode acp",
                    AgentKind::GeminiCli => "gemini --acp",
                    AgentKind::ClaudeAgent => "claude-agent-acp",
                    AgentKind::Codex => "codex-acp",
                    AgentKind::Custom => unreachable!(),
                };
                kill_ours(pattern);
            }
            Some(Event::Error(err)) => {
                outcome = Some(format!("error: {err}"));
                break;
            }
            Some(Event::Done) => {
                outcome = Some("done before the kill landed".to_owned());
                break;
            }
            None => {
                outcome = Some("ended without an error".to_owned());
                break;
            }
            Some(_) => {}
        }
    }
    eprintln!("[{label}] crash mid-answer: {outcome:?}");
    assert!(outcome.is_some(), "a crash must not hang the answer");

    // The next question starts it again.
    let again: String = ask(&settings, "Reply with exactly: restarted")
        .filter_map(|event| match event {
            Event::Text(text) => Some(text),
            Event::Error(err) => panic!("[{label}] restart failed: {err}"),
            _ => None,
        })
        .collect();
    eprintln!("[{label}] after a crash: {again:?}");
    assert!(!again.trim().is_empty());
    agent::shutdown();
}

/// Kills the agent Kanso started, and no other copy the person runs: only a
/// process matching `pattern` whose working folder is the matrix's own.
fn kill_ours(pattern: &str) {
    let found = std::process::Command::new("pgrep")
        .args(["-f", pattern])
        .output()
        .expect("pgrep runs");
    for pid in String::from_utf8_lossy(&found.stdout).split_whitespace() {
        let cwd = std::process::Command::new("lsof")
            .args(["-a", "-p", pid, "-d", "cwd", "-Fn"])
            .output()
            .expect("lsof runs");
        if String::from_utf8_lossy(&cwd.stdout).contains("kanso-real-agent-matrix") {
            let _ = std::process::Command::new("kill")
                .args(["-9", pid])
                .status();
        }
    }
}

/// A known agent that isn't installed says so instead of failing oddly.
#[test]
#[ignore = "needs PATH without the agent"]
fn a_missing_agent_says_it_is_not_installed() {
    let mut settings = settings(AgentKind::Custom);
    settings.command = Some("/nonexistent/kanso-no-such-agent".to_owned());
    let err = agent::check(&settings).expect_err("it can't start");
    eprintln!("not installed: {err}");
    agent::shutdown();
}
