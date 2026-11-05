//! What a failed AI request means, in words a person can act on.
//!
//! Every AI feature on the desktop and the phone reads its error through
//! [`classify`], so a missing key, a missing agent or a busy model is worded
//! once. The raw text is never the message; each [`Trouble`] names the real
//! cause and says where to fix it. The phone mirrors the same rules in
//! `ai_trouble_text.dart`; `tests/trouble_corpus.json` is read by both test
//! suites so the two cannot drift.

/// What went wrong, as far as the error's text tells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trouble {
    /// The provider needs a key and none is saved.
    MissingKey,
    /// A key is saved and the provider refused it.
    KeyRejected,
    /// Something between Kanso and the provider turned the request away: a
    /// proxy that wants its own sign-in, a gateway's or firewall's page, or a
    /// region the provider doesn't serve. Not the key.
    Blocked,
    /// No model name is set.
    MissingModel,
    /// The server's address is empty.
    MissingUrl,
    /// No agent is chosen, or a custom agent has no command.
    NoAgent,
    /// The chosen agent isn't on this computer.
    AgentNotInstalled,
    /// The chosen agent needs a newer Node.
    AgentNeedsNode,
    /// The model can't use tools, and the task needs them.
    NoTools,
    /// The provider is limiting requests.
    Busy,
    /// The request ran out of time.
    Slow,
    /// The named model isn't there.
    ModelNotFound,
    /// The provider's server failed (5xx).
    ServerDown,
    /// The server couldn't be reached; the local app that should be
    /// running, when the error tells.
    Unreachable(Option<LocalApp>),
    /// AI isn't set up at all.
    NotSetUp,
    /// Anything else.
    Other,
}

/// The app on this computer or network that serves a local model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalApp {
    Ollama,
    LmStudio,
    /// A Local model whose app the error doesn't name. agent-runtime labels
    /// every Local model error "ollama", LM Studio's included, so that label
    /// alone names neither.
    OllamaOrLmStudio,
}

impl LocalApp {
    /// The app's name, as the person knows it.
    pub fn name(self) -> &'static str {
        match self {
            LocalApp::Ollama => "Ollama",
            LocalApp::LmStudio => "LM Studio",
            LocalApp::OllamaOrLmStudio => "Ollama or LM Studio",
        }
    }
}

/// The words for a [`Trouble`]: a short title and what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Words {
    pub title: &'static str,
    pub detail: String,
}

impl Trouble {
    /// A stable name, for the tests both platforms share.
    pub fn key(self) -> &'static str {
        match self {
            Trouble::MissingKey => "missing_key",
            Trouble::KeyRejected => "key_rejected",
            Trouble::Blocked => "blocked",
            Trouble::MissingModel => "missing_model",
            Trouble::MissingUrl => "missing_url",
            Trouble::NoAgent => "no_agent",
            Trouble::AgentNotInstalled => "agent_not_installed",
            Trouble::AgentNeedsNode => "agent_needs_node",
            Trouble::NoTools => "no_tools",
            Trouble::Busy => "busy",
            Trouble::Slow => "slow",
            Trouble::ModelNotFound => "model_not_found",
            Trouble::ServerDown => "server_down",
            Trouble::Unreachable(None) => "unreachable",
            Trouble::Unreachable(Some(LocalApp::Ollama)) => "unreachable_ollama",
            Trouble::Unreachable(Some(LocalApp::LmStudio)) => "unreachable_lm_studio",
            Trouble::Unreachable(Some(LocalApp::OllamaOrLmStudio)) => "unreachable_local",
            Trouble::NotSetUp => "not_set_up",
            Trouble::Other => "other",
        }
    }

    /// The title and the fix.
    pub fn words(self) -> Words {
        let (title, detail): (&'static str, String) = match self {
            Trouble::MissingKey => (
                "No API key is saved",
                "Add one in Settings › AI: choose your provider, then paste the key into API key."
                    .to_owned(),
            ),
            Trouble::KeyRejected => (
                "Your model turned the key down",
                "The API key was refused. Check it in Settings › AI, then try again.".to_owned(),
            ),
            Trouble::Blocked => (
                "Something blocked the request",
                "A proxy, firewall or region limit turned it away before it reached your model, so it isn't your API key. Check your network and the address in Settings › AI, then try again."
                    .to_owned(),
            ),
            Trouble::MissingModel => (
                "No model is chosen",
                "Pick a model in Settings › AI, then try again.".to_owned(),
            ),
            Trouble::MissingUrl => (
                "No server address is set",
                "Enter the server's address in Settings › AI, then try again.".to_owned(),
            ),
            Trouble::NoAgent => (
                "No agent is set up",
                "Choose an agent in Settings › AI, or type the command of your own. Then try again."
                    .to_owned(),
            ),
            Trouble::AgentNotInstalled => (
                "Your agent isn't installed",
                "Install it, or choose another agent in Settings › AI and use Find to check again."
                    .to_owned(),
            ),
            Trouble::AgentNeedsNode => (
                "Your agent needs a newer Node",
                "Install the Node version it asks for, then use Find again in Settings › AI."
                    .to_owned(),
            ),
            Trouble::NoTools => (
                "This model can't use tools",
                "Pick a model that supports tools in Settings › AI.".to_owned(),
            ),
            Trouble::Busy => (
                "Your model is busy",
                "It's limiting requests right now. Wait a moment, then try again.".to_owned(),
            ),
            Trouble::Slow => (
                "Your model took too long",
                "It didn't answer in time. Try again, or pick a smaller model in Settings › AI."
                    .to_owned(),
            ),
            Trouble::ModelNotFound => (
                "Your model wasn't found",
                "The model named in Settings › AI isn't available. Choose another, then try again."
                    .to_owned(),
            ),
            Trouble::ServerDown => (
                "Your model's server had a problem",
                "That's on its side. Wait a moment, then try again.".to_owned(),
            ),
            Trouble::Unreachable(Some(app)) => (
                "Couldn't reach your model",
                format!("Is {} running? Start it, then try again.", app.name()),
            ),
            Trouble::Unreachable(None) => (
                "Couldn't reach your model",
                "Check your connection and the address in Settings › AI, then try again."
                    .to_owned(),
            ),
            Trouble::NotSetUp => (
                "AI isn't set up yet",
                "Choose a model in Settings › AI to use this.".to_owned(),
            ),
            Trouble::Other => (
                "Your model ran into a problem",
                "Try again. If it keeps happening, check Settings › AI.".to_owned(),
            ),
        };
        Words { title, detail }
    }
}

/// Whether `text` has HTTP status `code` as a number of its own: "status
/// 401", "HTTP 401:", "(401)", not the 401 inside "14010" or "a401b".
fn has_status_code(text: &str, code: &str) -> bool {
    text.match_indices(code).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + code.len()..].chars().next();
        let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        edge(before) && edge(after)
    })
}

/// Whether `text` has a 5xx status as a number of its own.
fn has_server_status(text: &str) -> bool {
    (500..600).any(|code| has_status_code(text, &code.to_string()))
}

/// Whether `text` names `port` as a port of its own (":1234/", ":1234)"),
/// not the start of ":12345".
fn has_port(text: &str, port: &str) -> bool {
    let needle = format!(":{port}");
    text.match_indices(&needle).any(|(at, _)| {
        text[at + needle.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_digit())
    })
}

/// How agent-runtime starts a Local model error. It says "ollama" for
/// every local server, so it is no sign of which app is serving.
const LOCAL_LABELS: &[&str] = &[
    "ollama transport error",
    "ollama reported an error",
    "ollama call returned",
];

/// Which local app an unreachable-server error points at, if any: a name in
/// the text, else the app's own default port, else "Ollama or LM Studio"
/// when only the runtime's Local model label is there.
fn local_app(lower: &str) -> Option<LocalApp> {
    let named = LOCAL_LABELS
        .iter()
        .fold(lower.to_owned(), |text, label| text.replace(label, ""));
    if named.contains("lm studio") || named.contains("lmstudio") {
        Some(LocalApp::LmStudio)
    } else if named.contains("ollama") || has_port(&named, "11434") {
        Some(LocalApp::Ollama)
    } else if has_port(&named, "1234") {
        Some(LocalApp::LmStudio)
    } else if named.len() != lower.len() {
        Some(LocalApp::OllamaOrLmStudio)
    } else {
        None
    }
}

/// What `raw` (an [`AiError`](crate::AiError)'s text, or a provider's own)
/// means.
pub fn classify(raw: &str) -> Trouble {
    let lower = raw.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| lower.contains(w));
    let code = |codes: &[&str]| codes.iter().any(|c| has_status_code(&lower, c));
    if has(&[
        "add an api key",
        "no api key",
        "missing api key",
        "api key is missing",
        "api key not set",
    ]) {
        Trouble::MissingKey
    } else if has(&["set a model in settings"]) {
        Trouble::MissingModel
    } else if has(&["set the server's address"]) {
        Trouble::MissingUrl
    } else if has(&["set the agent's command", "no agent"]) {
        Trouble::NoAgent
    } else if has(&["can't use tools"]) {
        Trouble::NoTools
    } else if has(&["needs node"]) {
        Trouble::AgentNeedsNode
    } else if has(&[
        "isn't installed",
        "on this computer. settings",
        "old claude code adapter",
    ]) {
        Trouble::AgentNotInstalled
    } else if code(&["407"])
        || has(&[
            "proxy authentication",
            "unsupported_country",
            "region, or territory",
            "location is not supported",
        ])
        || ((lower.contains("<html") || lower.contains("<!doctype"))
            && (code(&["401", "403"]) || has(&["unauthori", "forbidden", "authorization"])))
    {
        // A proxy's or gateway's own page, not the provider's API: provider
        // APIs answer in JSON. Checked before the key, which it isn't.
        Trouble::Blocked
    } else if has(&["access to the model"]) {
        // Bedrock's 403 for a model the account hasn't been given.
        Trouble::ModelNotFound
    } else if code(&["401", "403"])
        || has(&[
            "unauthori",
            "api key",
            "invalid key",
            "forbidden",
            "authentication",
        ])
    {
        Trouble::KeyRejected
    } else if code(&["429"]) || has(&["rate limit", "quota", "too many requests"]) {
        Trouble::Busy
    } else if has(&["timed out", "timeout", "didn't answer in time"]) {
        Trouble::Slow
    } else if code(&["404"])
        || has(&[
            "not found",
            "no such model",
            "unknown model",
            "model_not_found",
        ])
    {
        Trouble::ModelNotFound
    } else if has_server_status(&lower) || has(&["overloaded", "bad gateway"]) {
        Trouble::ServerDown
    } else if has(&[
        "transport",
        "connect",
        "connection",
        "refused",
        "unreachable",
        "dns",
        "network",
        "failed to call",
        "error sending request",
        "can't reach",
    ]) {
        Trouble::Unreachable(local_app(&lower))
    } else if has(&[
        "not set up",
        "not configured",
        "no model",
        "no provider",
        "ai is off",
    ]) {
        Trouble::NotSetUp
    } else {
        Trouble::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AiError, NO_TOOLS};
    use proptest::prelude::*;

    fn kind_of(err: &AiError) -> Trouble {
        classify(&err.to_string())
    }

    #[test]
    fn missing_key_says_the_key_is_missing_and_where_to_add_one() {
        let t = kind_of(&AiError::MissingKey);
        assert_eq!(t, Trouble::MissingKey);
        let w = t.words();
        assert!(w.title.contains("No API key"), "{}", w.title);
        assert!(w.detail.contains("Settings › AI"), "{}", w.detail);
        assert!(w.detail.contains("API key"), "{}", w.detail);
    }

    #[test]
    fn missing_model_names_the_missing_model() {
        assert_eq!(kind_of(&AiError::MissingModel), Trouble::MissingModel);
    }

    #[test]
    fn missing_url_names_the_missing_address() {
        assert_eq!(kind_of(&AiError::MissingUrl), Trouble::MissingUrl);
    }

    #[test]
    fn a_status_reads_by_its_number() {
        for (status, want) in [
            (401, Trouble::KeyRejected),
            (403, Trouble::KeyRejected),
            (404, Trouble::ModelNotFound),
            (429, Trouble::Busy),
            (500, Trouble::ServerDown),
            (503, Trouble::ServerDown),
            (400, Trouble::Other),
        ] {
            let err = AiError::Status {
                status,
                message: "nope".to_owned(),
            };
            assert_eq!(kind_of(&err), want, "{status}");
        }
    }

    #[test]
    fn a_status_message_that_mentions_a_key_is_not_a_missing_key() {
        let err = AiError::Status {
            status: 401,
            message: "Incorrect API key provided".to_owned(),
        };
        assert_eq!(kind_of(&err), Trouble::KeyRejected);
    }

    #[test]
    fn a_provider_error_reads_by_its_words() {
        assert_eq!(
            kind_of(&AiError::Provider("rate limit exceeded".to_owned())),
            Trouble::Busy
        );
        assert_eq!(
            kind_of(&AiError::Provider("something odd".to_owned())),
            Trouble::Other
        );
    }

    #[test]
    fn transport_names_a_local_app_when_the_error_does() {
        assert_eq!(
            kind_of(&AiError::Transport("Ollama refused".to_owned())),
            Trouble::Unreachable(Some(LocalApp::Ollama))
        );
        assert_eq!(
            kind_of(&AiError::Transport("LM Studio refused".to_owned())),
            Trouble::Unreachable(Some(LocalApp::LmStudio))
        );
        assert_eq!(
            kind_of(&AiError::Transport("ollama transport error".to_owned())),
            Trouble::Unreachable(Some(LocalApp::OllamaOrLmStudio))
        );
        assert_eq!(
            kind_of(&AiError::Transport("dns".to_owned())),
            Trouble::Unreachable(None)
        );
    }

    #[test]
    fn a_timeout_is_slow() {
        assert_eq!(kind_of(&AiError::Timeout("30s".to_owned())), Trouble::Slow);
    }

    #[test]
    fn agent_errors_name_the_agent_problem() {
        let sentence = |s: &str| kind_of(&AiError::Agent(s.to_owned()));
        assert_eq!(sentence(NO_TOOLS), Trouble::NoTools);
        assert_eq!(
            sentence("Set the agent's command in Settings › AI."),
            Trouble::NoAgent
        );
        assert_eq!(
            sentence("Claude Code isn't installed. Settings › AI has the command."),
            Trouble::AgentNotInstalled
        );
        assert_eq!(
            sentence(
                "Kanso can't find foo on this computer. Settings › AI has the agent's command."
            ),
            Trouble::AgentNotInstalled
        );
        assert_eq!(
            sentence("Only the old Claude Code adapter (x) is installed, which is deprecated."),
            Trouble::AgentNotInstalled
        );
        assert_eq!(
            sentence(
                "Codex needs Node 22 or newer. Install Node, then use Find again in Settings › AI."
            ),
            Trouble::AgentNeedsNode
        );
        assert_eq!(sentence("The agent isn't running."), Trouble::Other);
    }

    #[cfg(feature = "acp")]
    #[test]
    fn the_real_custom_agent_errors_read_right() {
        use crate::{AgentKind, AgentSettings};
        let mut settings = AgentSettings {
            kind: AgentKind::Custom,
            command: None,
            args: Vec::new(),
            env: Vec::new(),
            workspace: std::env::temp_dir(),
        };
        let err = settings.spec().expect_err("no command");
        assert_eq!(kind_of(&err), Trouble::NoAgent);
        settings.command = Some("kanso-no-such-agent-binary".to_owned());
        let err = settings.spec().expect_err("not found");
        assert_eq!(kind_of(&err), Trouble::AgentNotInstalled);
    }

    #[test]
    fn a_proxy_or_gateway_refusal_is_not_the_key() {
        for raw in [
            "The model's server answered 407: Proxy Authentication Required",
            "The model's server answered 403: <html><head><title>403 Forbidden</title>",
            "The model's server answered 401: <!DOCTYPE html><title>401 Authorization Required",
            "The model's server answered 403: Country, region, or territory not supported",
        ] {
            assert_eq!(classify(raw), Trouble::Blocked, "{raw}");
            let words = classify(raw).words();
            assert!(!words.title.contains("key"), "{}", words.title);
        }
        // The provider's own JSON refusal is still the key.
        assert_eq!(
            classify("The model's server answered 401: {\"error\":{\"code\":\"invalid_api_key\"}}"),
            Trouble::KeyRejected
        );
    }

    #[test]
    fn the_local_model_label_names_no_app() {
        assert_eq!(
            local_app("ollama transport error: failed to call chat completions"),
            Some(LocalApp::OllamaOrLmStudio)
        );
        assert_eq!(
            local_app("ollama transport error: (http://localhost:1234/v1)"),
            Some(LocalApp::LmStudio)
        );
        assert_eq!(
            local_app("ollama transport error: (http://localhost:11434/v1)"),
            Some(LocalApp::Ollama)
        );
        assert_eq!(
            local_app("custom transport error: (http://h:12345/v1)"),
            None
        );
        assert_eq!(
            Trouble::Unreachable(Some(LocalApp::OllamaOrLmStudio))
                .words()
                .detail,
            "Is Ollama or LM Studio running? Start it, then try again."
        );
    }

    #[test]
    fn off_and_unset_read_as_not_set_up() {
        assert_eq!(classify("AI is off"), Trouble::NotSetUp);
        assert_eq!(classify("no provider chosen"), Trouble::NotSetUp);
    }

    #[test]
    fn status_codes_match_as_numbers_not_substrings() {
        assert_eq!(classify("port 14010 closed"), Trouble::Other);
        assert_eq!(classify("id a401b"), Trouble::Other);
        assert_eq!(classify("took 4290 ms"), Trouble::Other);
        assert_eq!(classify("HTTP 401: nope"), Trouble::KeyRejected);
    }

    const ALL: &[Trouble] = &[
        Trouble::MissingKey,
        Trouble::KeyRejected,
        Trouble::Blocked,
        Trouble::MissingModel,
        Trouble::MissingUrl,
        Trouble::NoAgent,
        Trouble::AgentNotInstalled,
        Trouble::AgentNeedsNode,
        Trouble::NoTools,
        Trouble::Busy,
        Trouble::Slow,
        Trouble::ModelNotFound,
        Trouble::ServerDown,
        Trouble::Unreachable(None),
        Trouble::Unreachable(Some(LocalApp::Ollama)),
        Trouble::Unreachable(Some(LocalApp::LmStudio)),
        Trouble::Unreachable(Some(LocalApp::OllamaOrLmStudio)),
        Trouble::NotSetUp,
        Trouble::Other,
    ];

    #[test]
    fn every_kind_has_a_title_a_fix_and_a_key_of_its_own() {
        let mut titles = std::collections::HashSet::new();
        let mut keys = std::collections::HashSet::new();
        for t in ALL {
            let w = t.words();
            assert!(!w.title.is_empty() && !w.detail.is_empty(), "{t:?}");
            titles.insert(w.title);
            keys.insert(t.key());
        }
        assert_eq!(keys.len(), ALL.len());
        // The four "unreachable" kinds share one title.
        assert_eq!(titles.len(), ALL.len() - 3);
    }

    #[test]
    fn the_corpus_reads_as_written() {
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../tests/trouble_corpus.json")).unwrap();
        for case in corpus.as_array().unwrap() {
            let raw = case["raw"].as_str().unwrap();
            let want = case["kind"].as_str().unwrap();
            assert_eq!(classify(raw).key(), want, "{raw}");
        }
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            rng_seed: proptest::test_runner::RngSeed::Fixed(38),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Whatever the error says, the person gets a title and a fix.
        #[test]
        fn no_message_is_empty(raw in ".{0,160}") {
            let w = classify(&raw).words();
            prop_assert!(!w.title.trim().is_empty());
            prop_assert!(!w.detail.trim().is_empty());
        }

        /// Every variant, whatever its payload, gets words.
        #[test]
        fn every_variant_has_words(msg in ".{0,80}", status in 0u16..700) {
            let errs = [
                AiError::MissingKey,
                AiError::MissingModel,
                AiError::MissingUrl,
                AiError::Status { status, message: msg.clone() },
                AiError::Provider(msg.clone()),
                AiError::Transport(msg.clone()),
                AiError::Timeout(msg.clone()),
                AiError::Agent(msg),
            ];
            for err in errs {
                let w = kind_of(&err).words();
                prop_assert!(!w.title.is_empty() && !w.detail.is_empty());
            }
        }
    }
}
