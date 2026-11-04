//! `kanso mcp`: Kanso's MCP server over standard input and output, for AI
//! agents that start a program rather than connect to a URL (Claude
//! Desktop, say). Each JSON-RPC message read from a line of input goes to
//! the running app's `/mcp` with the agent's API key, and each answer is
//! written back on a line of its own. It never opens the library itself: an
//! agent keeps this process running, and holding the library would stop
//! Kanso from opening.
//!
//! The key comes from `KANSO_MCP_TOKEN` (never from the arguments), and a
//! relay without one refuses: it answers `initialize` with an error that
//! says how to get a key, and exits. The address is `KANSO_MCP_URL` when
//! set (only plain HTTP to this computer, so the key can't be sent
//! elsewhere), otherwise the one in `config.json`, which holds no secrets.
//! An agent Kanso itself starts gets both variables, with a short-lived
//! token in place of a key.

use std::io::{BufRead as _, Write as _};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, Result};
use serde_json::{Value, json};

use crate::config::ServerSettings;

const USAGE: &str = "\
Usage: kanso mcp

Serves Kanso's MCP tools over standard input and output, for AI agents
that start a program (Claude Desktop, Codex). It talks to the running app,
so Kanso must be open with Settings › Local API on, and it needs an API
key in KANSO_MCP_TOKEN: in Kanso, open Settings › Local API › API keys and
choose New key….
";

/// The JSON-RPC error code for a request the app couldn't answer.
const UNAVAILABLE: i64 = -32000;
/// The JSON-RPC error code for a relay that has no key, or whose key Kanso
/// doesn't recognise.
const NEEDS_KEY: i64 = -32001;
/// What the relay exits with when it has no key.
const NO_KEY_EXIT: i32 = 2;

/// What an agent is told when it starts the relay without a key.
const NO_KEY: &str = "Kanso needs a key for this agent. In Kanso, open Settings › Local API › API keys, choose New key…, and paste the setup it gives you into this agent's settings.";
/// What an agent is told when Kanso answers 401 to its key.
const UNKNOWN_KEY: &str = "Kanso doesn't recognise this key. It may have been revoked. Create a new one in Kanso › Settings › Local API › API keys.";

/// The variables that put the relay in an agent's token mode.
pub const URL_VAR: &str = "KANSO_MCP_URL";
pub const TOKEN_VAR: &str = "KANSO_MCP_TOKEN";

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let key = std::env::var(TOKEN_VAR)
        .ok()
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty());
    let Some(key) = key else {
        return refuse_without_key();
    };
    let relay = match std::env::var(URL_VAR).ok() {
        Some(url) => relay_to(url, &key)?,
        None => Relay::new(&crate::load_config(data_dir)?.server, &key),
    };
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.context("reading standard input")?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = relay.send(&line) {
            writeln!(stdout, "{answer}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// Answers the agent's first request (its `initialize`) with [`NO_KEY`],
/// then exits with status 2.
fn refuse_without_key() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let lines = stdin.lock().lines().map_while(std::result::Result::ok);
    if let Some(answer) = no_key_answer(lines) {
        writeln!(stdout, "{answer}")?;
        stdout.flush()?;
    }
    eprintln!("kanso mcp: {NO_KEY}");
    std::process::exit(NO_KEY_EXIT);
}

/// The answer to the first request among `lines`: error [`NEEDS_KEY`] with
/// the request's own ID. `None` when there is no request.
fn no_key_answer(lines: impl IntoIterator<Item = String>) -> Option<String> {
    lines
        .into_iter()
        .find_map(|line| request_id(&line))
        .map(|id| error_with(id, NEEDS_KEY, NO_KEY))
}

/// The relay to `url` with `key`, only to plain HTTP on this computer, so
/// the key can't be sent elsewhere.
fn relay_to(url: String, key: &str) -> Result<Relay> {
    if !is_loopback_url(&url) {
        anyhow::bail!(
            "{URL_VAR} must be Kanso's own address on this computer: plain http:// to 127.0.0.1, [::1] or localhost. Unset it to use the address in Kanso's settings."
        );
    }
    Ok(Relay {
        endpoint: url,
        authorization: format!("Bearer {key}"),
        enabled: true,
        agent: http_agent(),
    })
}

/// Whether `url` is plain HTTP to this computer: `http://127.0.0.1:…`,
/// `http://[::1]:…` or `http://localhost:…`.
fn is_loopback_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return false;
    }
    let host = match authority.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or_default(),
        None => authority.split(':').next().unwrap_or_default(),
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(120)))
        .build()
        .into()
}

/// The running app's MCP endpoint and how to sign in to it.
struct Relay {
    endpoint: String,
    authorization: String,
    /// Whether the local API is on (always, for an agent's token).
    enabled: bool,
    agent: ureq::Agent,
}

impl Relay {
    fn new(server: &ServerSettings, key: &str) -> Relay {
        Relay {
            endpoint: crate::settings::mcp_endpoint(&server.bind_address, server.port),
            authorization: format!("Bearer {key}"),
            enabled: server.enabled,
            agent: http_agent(),
        }
    }

    /// Passes one message on, and gives back the line to answer with:
    /// the app's answer, an error for a request it couldn't answer, or
    /// nothing for a notification.
    fn send(&self, message: &str) -> Option<String> {
        let id = request_id(message);
        if !self.enabled {
            return id.map(|id| {
                error(
                    id,
                    "Kanso's local API is off. Turn it on in Kanso's Settings › Local API.",
                )
            });
        }
        let response = self
            .agent
            .post(&self.endpoint)
            .header("authorization", &self.authorization)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .send(message.as_bytes());
        let mut response = match response {
            Ok(response) => response,
            Err(_) => {
                return id.map(|id| error(id, "Kanso isn't running. Open Kanso, then try again."));
            }
        };
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        if status == 401 {
            return id.map(|id| {
                if self.authorization.starts_with("Bearer kcl_") {
                    error_with(id, NEEDS_KEY, UNKNOWN_KEY)
                } else {
                    // An agent's token expired or was revoked: the question
                    // goes on with what it has read.
                    error(id, "Kanso's note access for this question has ended.")
                }
            });
        }
        answer(id, status, &body)
    }
}

/// The ID of a JSON-RPC request; `None` for a notification (or a line that
/// isn't JSON-RPC, which the app answers itself).
fn request_id(message: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(message).ok()?;
    value.get("id").filter(|id| !id.is_null()).cloned()
}

/// The line to write for the app's answer to a request with `id`: its body
/// on one line, or an error saying what went wrong; nothing when there is
/// nothing to answer.
fn answer(id: Option<Value>, status: u16, body: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value.to_string());
    }
    let id = id?;
    Some(match status {
        202 | 204 => return None,
        401 => error_with(id, NEEDS_KEY, UNKNOWN_KEY),
        _ => error(id, &format!("Kanso answered with HTTP {status}.")),
    })
}

fn error(id: Value, message: &str) -> String {
    error_with(id, UNAVAILABLE, message)
}

fn error_with(id: Value, code: i64, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // no-prop-test: a fixed table of addresses, each accepted or refused.
    #[test]
    fn a_key_goes_only_to_this_computer() {
        for ok in [
            "http://127.0.0.1:4242/mcp",
            "http://localhost:1/mcp",
            "http://[::1]:9/mcp",
        ] {
            assert!(is_loopback_url(ok), "{ok}");
            let relay = relay_to(ok.into(), "kcl_tok").unwrap();
            assert_eq!(relay.authorization, "Bearer kcl_tok");
        }
        for bad in [
            "https://127.0.0.1/mcp",
            "http://example.com/mcp",
            "http://127.0.0.1.example.com/mcp",
            "http://user@127.0.0.1/mcp",
            "http://10.0.0.2/mcp",
            "127.0.0.1:4242",
        ] {
            assert!(!is_loopback_url(bad), "{bad}");
            let err = relay_to(bad.into(), "kcl_tok").err().expect("refused");
            assert!(err.to_string().contains("Unset it"), "{err}");
        }
    }

    // no-prop-test: the messages are fixed by the design; the property
    // below covers the IDs.
    #[test]
    fn the_messages_are_the_designs() {
        assert_eq!(
            NO_KEY,
            "Kanso needs a key for this agent. In Kanso, open Settings › Local API › API keys, choose New key…, and paste the setup it gives you into this agent's settings."
        );
        assert_eq!(
            UNKNOWN_KEY,
            "Kanso doesn't recognise this key. It may have been revoked. Create a new one in Kanso › Settings › Local API › API keys."
        );
        assert_eq!((NEEDS_KEY, NO_KEY_EXIT), (-32001, 2));
        let init = r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}"#;
        let refused: Value =
            serde_json::from_str(&no_key_answer([init.to_owned()]).unwrap()).unwrap();
        assert_eq!(refused["id"], 0);
        assert_eq!(refused["error"]["code"], -32001);
        assert_eq!(refused["error"]["message"], NO_KEY);
        // Notifications and blank lines come first without an answer.
        let lines = [
            String::new(),
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#.to_owned(),
            init.to_owned(),
        ];
        assert_eq!(
            no_key_answer(lines).unwrap(),
            no_key_answer([init.to_owned()]).unwrap()
        );
        assert_eq!(no_key_answer([]), None);
        let unknown: Value =
            serde_json::from_str(&answer(Some(json!("a")), 401, "").unwrap()).unwrap();
        assert_eq!(unknown["error"]["code"], -32001);
        assert_eq!(unknown["error"]["message"], UNKNOWN_KEY);
    }

    #[test]
    fn answer_examples() {
        let ok = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        assert_eq!(answer(Some(json!(1)), 200, ok).as_deref(), Some(ok));
        // A pretty-printed answer still goes back on one line.
        let pretty = "{\n  \"jsonrpc\": \"2.0\",\n  \"id\": 1,\n  \"result\": {}\n}";
        assert_eq!(answer(Some(json!(1)), 200, pretty).as_deref(), Some(ok));
        assert_eq!(answer(None, 202, ""), None);
        assert_eq!(answer(Some(json!(1)), 202, ""), None);
        let refused: Value =
            serde_json::from_str(&answer(Some(json!("a")), 401, "").unwrap()).unwrap();
        assert_eq!(refused["id"], "a");
        assert_eq!(refused["error"]["code"], NEEDS_KEY);
        assert_eq!(
            request_id(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
            None
        );
        assert_eq!(
            request_id(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#),
            Some(json!(7))
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

        /// Whatever the app answers, what goes back is one line of JSON,
        /// and a request whose answer was lost still gets an error with
        /// its own ID.
        #[test]
        fn prop_answers_are_one_line_of_json(
            id in prop_oneof![any::<i64>().prop_map(|n| json!(n)), "[a-z0-9-]{1,12}".prop_map(|s| json!(s))],
            status in prop_oneof![Just(200u16), Just(400), Just(401), Just(500), Just(502)],
            body in prop_oneof![Just(String::new()), "[ -~]{0,40}", Just("{\n \"x\": 1\n}".to_owned())],
        ) {
            let line = answer(Some(id.clone()), status, &body).expect("a request is answered");
            prop_assert!(!line.contains('\n'));
            let value: Value = serde_json::from_str(&line).unwrap();
            if serde_json::from_str::<Value>(&body).is_err() {
                prop_assert_eq!(&value["id"], &id);
                let code = if status == 401 { NEEDS_KEY } else { UNAVAILABLE };
                prop_assert_eq!(&value["error"]["code"], &json!(code));
            }
        }

        /// A relay with no key answers the first request, whatever else
        /// came before it, with its own ID and the fix-it message.
        #[test]
        fn prop_no_key_answers_the_first_request(
            id in prop_oneof![any::<i64>().prop_map(|n| json!(n)), "[a-z0-9-]{1,12}".prop_map(|s| json!(s))],
            noise in prop::collection::vec("[ -~]{0,20}", 0..4),
        ) {
            let mut lines: Vec<String> = noise
                .into_iter()
                .filter(|line| request_id(line).is_none())
                .collect();
            lines.push(json!({"jsonrpc": "2.0", "id": id, "method": "initialize"}).to_string());
            let line = no_key_answer(lines).expect("a request is answered");
            prop_assert!(!line.contains('\n'));
            let value: Value = serde_json::from_str(&line).unwrap();
            prop_assert_eq!(&value["id"], &id);
            prop_assert_eq!(&value["error"]["code"], &json!(-32001));
            prop_assert_eq!(&value["error"]["message"], &json!(NO_KEY));
        }
    }
}
