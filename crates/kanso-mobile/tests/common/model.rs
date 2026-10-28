//! A model that answers from a script, with no network, and keeps every byte
//! it was sent. Used by the AI tests through [`ai_phone`].
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent_runtime::{HttpClient, HttpRequest, HttpResponse, HttpStreamResponse};
use async_trait::async_trait;
use futures_util::stream;
use kanso_ai::{ClientSettings, SharedProvider};
use kanso_mobile::{AiHandle, AiSettingsInput, Library, ProviderMaker};
use serde_json::json;

use super::{Phone, lone_phone};

/// How the scripted model's streamed answer ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    /// With the end marker.
    Complete,
    /// Without it.
    CutOff,
    /// After its first words, it goes quiet and never finishes.
    Hangs,
}

/// A model: it may ask for tool calls first, then streams `answer`.
pub struct Scripted {
    pub answer: String,
    /// `(tool, JSON arguments)` asked for in the model's first turn.
    pub tool_calls: Vec<(String, String)>,
    pub ending: Ending,
    /// Every request body sent to it, as text.
    pub sent: Mutex<Vec<String>>,
    pub requests: AtomicUsize,
}

impl Scripted {
    pub fn new(answer: &str) -> Arc<Scripted> {
        Scripted::with(answer, Vec::new(), Ending::Complete)
    }

    pub fn with(answer: &str, tool_calls: Vec<(&str, String)>, ending: Ending) -> Arc<Scripted> {
        Arc::new(Scripted {
            answer: answer.to_owned(),
            tool_calls: tool_calls
                .into_iter()
                .map(|(name, args)| (name.to_owned(), args))
                .collect(),
            ending,
            sent: Mutex::new(Vec::new()),
            requests: AtomicUsize::new(0),
        })
    }

    /// Everything it was sent, joined.
    pub fn everything_sent(&self) -> String {
        self.sent.lock().unwrap().join("\n")
    }

    pub fn request_count(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }

    fn record(&self, request: &HttpRequest) {
        self.requests.fetch_add(1, Ordering::SeqCst);
        self.sent
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(&request.body).into_owned());
    }

    fn sse(&self) -> Vec<Vec<u8>> {
        let (first, second) = self.answer.split_at(self.answer.len() / 2);
        let mut chunks = vec![
            format!(
                "data: {}\n\n",
                json!({"choices":[{"index":0,"delta":{"content": first}}]})
            ),
            format!(
                "data: {}\n\n",
                json!({"choices":[{"index":0,"delta":{"content": second}}]})
            ),
        ];
        if self.ending == Ending::Complete {
            chunks.push(format!(
                "data: {}\n\n",
                json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]})
            ));
            chunks.push("data: [DONE]\n\n".to_owned());
        }
        chunks.into_iter().map(String::into_bytes).collect()
    }
}

#[async_trait]
impl HttpClient for Scripted {
    /// The model's deciding turn: tool calls if it has any, else its answer.
    async fn send(&self, request: HttpRequest) -> anyhow::Result<HttpResponse> {
        self.record(&request);
        let message = if self.tool_calls.is_empty() {
            json!({"content": self.answer})
        } else {
            let calls: Vec<_> = self
                .tool_calls
                .iter()
                .enumerate()
                .map(|(at, (name, args))| {
                    json!({"id": format!("call_{at}"), "type": "function",
                           "function": {"name": name, "arguments": args}})
                })
                .collect();
            json!({"content": null, "tool_calls": calls})
        };
        Ok(HttpResponse {
            status: 200,
            body: json!({"choices": [{"message": message}]})
                .to_string()
                .into_bytes(),
        })
    }

    async fn send_streaming(&self, request: HttpRequest) -> anyhow::Result<HttpStreamResponse> {
        self.record(&request);
        let chunks: Vec<anyhow::Result<Vec<u8>>> = self.sse().into_iter().map(Ok).collect();
        if self.ending == Ending::Hangs {
            let first = futures_util::stream::iter(chunks);
            let rest = futures_util::stream::once(async {
                std::future::pending::<()>().await;
                Ok(Vec::new())
            });
            return Ok(HttpStreamResponse {
                status: 200,
                body: Box::pin(futures_util::StreamExt::chain(first, rest)),
            });
        }
        Ok(HttpStreamResponse {
            status: 200,
            body: Box::pin(stream::iter(chunks)),
        })
    }
}

/// A maker whose providers are `model`.
pub fn maker_of(model: Arc<Scripted>) -> ProviderMaker {
    Arc::new(move |settings: &ClientSettings| {
        let provider = kanso_ai::client_with(settings, model.clone())?;
        let shared: SharedProvider = Arc::new(Mutex::new(provider));
        Ok(shared)
    })
}

/// The settings the tests turn AI on with: the custom provider, which needs
/// no key.
pub fn custom_settings(enabled: bool) -> AiSettingsInput {
    AiSettingsInput {
        enabled,
        provider: "custom".into(),
        base_url: "http://127.0.0.1:9/v1".into(),
        model: "test-model".into(),
        region: String::new(),
    }
}

/// A library with AI on, answering with `model`.
pub fn ai_on(library: &Library, model: &Arc<Scripted>) -> AiHandle {
    let ai = library.ai_with_provider(maker_of(model.clone()));
    ai.set_settings(custom_settings(true)).unwrap();
    ai
}

/// A new library with AI on, answering with `model`.
pub fn ai_phone(model: &Arc<Scripted>) -> (Phone, AiHandle) {
    let phone = lone_phone();
    let ai = ai_on(&phone.library, model);
    (phone, ai)
}

/// A sink and what it hears. The receiver gets every value.
pub fn listen<T: Send + 'static>() -> (
    Box<dyn FnMut(T) -> bool + Send + 'static>,
    std::sync::mpsc::Receiver<T>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    (Box::new(move |value| tx.send(value).is_ok()), rx)
}

/// Everything heard until `last` says a value is the end (it is included).
pub fn until<T>(rx: &std::sync::mpsc::Receiver<T>, last: impl Fn(&T) -> bool) -> Vec<T> {
    let mut heard = Vec::new();
    loop {
        let value = rx
            .recv_timeout(Duration::from_secs(20))
            .expect("the stream ended without its last event");
        let end = last(&value);
        heard.push(value);
        if end {
            return heard;
        }
    }
}
