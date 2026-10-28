//! AI for Kanso: choosing where a question goes, streaming the answer, and
//! the prompt it is asked with.
//!
//! The provider layer is [`agent_runtime`]'s `Llm`: every named provider and
//! any OpenAI-compatible endpoint, with streaming, tool calling and typed
//! timeouts. Kanso keeps a small synchronous surface here (an [`Event`]
//! iterator) so the GPUI app never awaits, and owns one small tokio runtime on
//! a background thread for the async calls. An agent on this computer
//! (`agent.rs`, over `kanso-acp`, with the `acp` feature the phones leave
//! off) answers through the same [`ChatProvider`].
//! Nothing in this crate reads or writes notes.

#[cfg(feature = "acp")]
pub mod agent;
pub mod client;
pub mod features;
pub mod prompt;

#[cfg(feature = "acp")]
pub use agent::{AgentCheck, AgentKind, AgentSettings};
pub use client::{
    AiError, CUT_OFF, ChatProvider, ClientSettings, Event, SharedProvider, Turn, answer,
    cached_client, client, client_with, client_with_timeouts, probe, redact_provider_error,
};
pub use features::InlineAction;
pub use prompt::{DEFAULT_BUDGET, Excerpt, Prompt, build as build_prompt};

/// The tool types a question can use. Re-exported so the app does not depend
/// on agent-runtime directly.
pub use agent_runtime::{
    JsonTool, ToolCall, ToolDefinition, ToolOutput, ToolRegistry, ToolSessionOutcome,
};

use serde::{Deserialize, Serialize};

/// Where questions go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    /// Ollama, or another model on this device or network, through an
    /// OpenAI-compatible server.
    #[default]
    LocalModel,
    OpenAi,
    Anthropic,
    Gemini,
    OpenRouter,
    Groq,
    DeepSeek,
    Xai,
    Mistral,
    Cohere,
    Bedrock,
    Kimi,
    /// Any other OpenAI-compatible server, at a URL the person types.
    Custom,
}

impl Provider {
    pub const ALL: [Provider; 13] = [
        Provider::LocalModel,
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::Gemini,
        Provider::OpenRouter,
        Provider::Groq,
        Provider::DeepSeek,
        Provider::Xai,
        Provider::Mistral,
        Provider::Cohere,
        Provider::Bedrock,
        Provider::Kimi,
        Provider::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Provider::LocalModel => "Local model",
            Provider::OpenAi => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Google Gemini",
            Provider::OpenRouter => "OpenRouter",
            Provider::Groq => "Groq",
            Provider::DeepSeek => "DeepSeek",
            Provider::Xai => "xAI",
            Provider::Mistral => "Mistral",
            Provider::Cohere => "Cohere",
            Provider::Bedrock => "AWS Bedrock",
            Provider::Kimi => "Kimi (Moonshot)",
            Provider::Custom => "Custom URL",
        }
    }

    /// The name under `Provider` in a config file.
    pub fn key(self) -> &'static str {
        match self {
            Provider::LocalModel => "local",
            Provider::OpenAi => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Gemini => "gemini",
            Provider::OpenRouter => "openrouter",
            Provider::Groq => "groq",
            Provider::DeepSeek => "deepseek",
            Provider::Xai => "xai",
            Provider::Mistral => "mistral",
            Provider::Cohere => "cohere",
            Provider::Bedrock => "bedrock",
            Provider::Kimi => "kimi",
            Provider::Custom => "custom",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Provider::LocalModel => "http://localhost:11434/v1",
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Anthropic => "https://api.anthropic.com",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::Groq => "https://api.groq.com/openai/v1",
            Provider::DeepSeek => "https://api.deepseek.com/v1",
            Provider::Xai => "https://api.x.ai/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::Cohere => "https://api.cohere.com/v2",
            // Bedrock's endpoint follows the configured AWS region.
            Provider::Bedrock => "",
            Provider::Kimi => "https://api.moonshot.cn/v1",
            Provider::Custom => "http://localhost:1234/v1",
        }
    }

    pub fn default_model(self) -> &'static str {
        // Names the provider documents today, or the agent-runtime default
        // tier where the vendor keeps a moving alias.
        match self {
            Provider::LocalModel => "llama3.2",
            Provider::OpenAi => "gpt-4o-mini",
            Provider::Anthropic => "claude-haiku-5-5",
            Provider::Gemini => "gemini-1.5-flash",
            Provider::OpenRouter => "openai/gpt-4o-mini",
            Provider::Groq => "llama-3.3-70b-versatile",
            Provider::DeepSeek => "deepseek-chat",
            Provider::Xai => "grok-2-latest",
            Provider::Mistral => "mistral-small-latest",
            Provider::Cohere => "command-r-plus",
            Provider::Bedrock => "anthropic.claude-3-5-haiku-20241022-v1:0",
            Provider::Kimi => "moonshot-v1-8k",
            Provider::Custom => "",
        }
    }

    /// The AWS region Bedrock starts in; ignored by other providers.
    pub fn default_region(self) -> &'static str {
        if self == Provider::Bedrock {
            "us-east-1"
        } else {
            ""
        }
    }

    /// Whether a key is required before asking. Cloud providers need one; a
    /// local server and a custom endpoint may or may not, so their key is
    /// optional.
    pub fn needs_key(self) -> bool {
        !matches!(self, Provider::LocalModel | Provider::Custom)
    }

    /// Whether the provider takes a key at all (cloud ones require it,
    /// local/custom have an optional field).
    pub fn accepts_key(self) -> bool {
        true
    }

    /// The agent-runtime provider this maps to.
    pub fn agent_kind(self) -> agent_runtime::AgentProviderKind {
        use agent_runtime::AgentProviderKind;
        match self {
            Provider::LocalModel => AgentProviderKind::Ollama,
            Provider::OpenAi => AgentProviderKind::OpenAi,
            Provider::Anthropic => AgentProviderKind::Anthropic,
            Provider::Gemini => AgentProviderKind::Gemini,
            Provider::OpenRouter => AgentProviderKind::OpenRouter,
            Provider::Groq => AgentProviderKind::Groq,
            Provider::DeepSeek => AgentProviderKind::DeepSeek,
            Provider::Xai => AgentProviderKind::Xai,
            Provider::Mistral => AgentProviderKind::Mistral,
            Provider::Cohere => AgentProviderKind::Cohere,
            Provider::Bedrock => AgentProviderKind::Bedrock,
            Provider::Kimi => AgentProviderKind::Kimi,
            Provider::Custom => AgentProviderKind::Custom("custom".to_owned()),
        }
    }

    /// The server host requests go to, for the line that says where they go.
    pub fn host(self, base_url: &str) -> String {
        let rest = base_url
            .trim()
            .split_once("://")
            .map_or(base_url.trim(), |(_, rest)| rest);
        let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        if host.is_empty() {
            // Bedrock derives its address from the region; every other
            // provider has a default URL.
            let fallback = if self == Provider::Bedrock {
                "bedrock-runtime.us-east-1.amazonaws.com"
            } else {
                self.default_base_url()
            };
            fallback
                .trim()
                .split_once("://")
                .map_or(fallback, |(_, rest)| rest)
                .split('/')
                .next()
                .unwrap_or("")
                .to_owned()
        } else {
            host.to_owned()
        }
    }

    /// Whether requests leave this Mac for the address: true unless the host
    /// is a loopback address.
    pub fn is_remote(self, base_url: &str) -> bool {
        let authority = self.host(base_url);
        // `[::1]:1234` is loopback; strip the brackets before the colons.
        let host = if let Some(inner) = authority.strip_prefix('[')
            && let Some((inside, _)) = inner.split_once(']')
        {
            inside.to_owned()
        } else {
            authority.split(':').next().unwrap_or(&authority).to_owned()
        };
        let host = host.to_ascii_lowercase();
        !matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")
    }
}

#[cfg(test)]
mod tests {
    use super::Provider;

    #[test]
    fn providers_name_their_defaults() {
        assert_eq!(Provider::Anthropic.default_model(), "claude-haiku-5-5");
        assert!(Provider::OpenAi.needs_key());
        assert!(Provider::Gemini.needs_key());
        assert!(Provider::Bedrock.needs_key());
        assert!(!Provider::LocalModel.needs_key());
        assert!(!Provider::Custom.needs_key());
        assert!(Provider::Custom.accepts_key());
        assert_eq!(
            Provider::OpenAi.host("https://api.openai.com/v1"),
            "api.openai.com"
        );
        assert_eq!(
            Provider::LocalModel.host("http://localhost:11434/v1"),
            "localhost:11434"
        );
        assert_eq!(Provider::OpenAi.host(""), "api.openai.com");
        assert_eq!(
            Provider::Bedrock.host(""),
            "bedrock-runtime.us-east-1.amazonaws.com"
        );
    }

    #[test]
    fn only_loopback_stays_on_this_mac() {
        assert!(!Provider::LocalModel.is_remote("http://localhost:11434/v1"));
        assert!(!Provider::Custom.is_remote("http://127.0.0.1:1234/v1"));
        assert!(!Provider::Custom.is_remote("http://[::1]:1234/v1"));
        assert!(Provider::Custom.is_remote("http://192.168.1.9:1234/v1"));
        assert!(Provider::OpenAi.is_remote("https://api.openai.com/v1"));
    }

    #[test]
    fn every_provider_has_a_label_and_an_agent_kind() {
        let mut seen = std::collections::HashSet::new();
        for provider in Provider::ALL {
            assert!(!provider.label().is_empty());
            assert!(seen.insert(provider.key()), "{:?} repeated", provider);
            let kind = provider.agent_kind();
            let expected = match provider {
                Provider::LocalModel => "ollama",
                other => other.key(),
            };
            assert_eq!(kind.as_str(), expected);
        }
    }
}
