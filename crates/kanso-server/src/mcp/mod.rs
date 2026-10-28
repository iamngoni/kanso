//! Kanso's native MCP endpoint at `/mcp`: the tools of `@inkdropapp/mcp-server`
//! 1.5.0 and its note resource template
//! ([`NOTE_LINK_PREFIX`](kanso_core::links::NOTE_LINK_PREFIX)`{noteId}`).
//!
//! The transport is MCP's streamable HTTP in its simplest conforming form:
//! each POST carries a JSON-RPC message (or a batch) and requests get a JSON
//! answer; notifications get `202 Accepted`. The server keeps no sessions and
//! offers no event stream, so GET and DELETE answer 405.
//!
//! An agent's token ([`Caller::Agent`]) sees a narrower server: only the read
//! tools its scope offers are listed or run, every note they would show is
//! checked first ([`Visibility`](crate::scope::Visibility)), and `resources/read`
//! follows the same rule. Everything else answers that the agent has
//! read-only access. A scope the person let suggest edits also gets
//! `patch-note`, `update-note` and `create-note`, which stage a change in the
//! review queue (`suggest.rs`) and write nothing.

mod schema;
mod suggest;
mod tools;

use axum::body::Bytes;
use axum::extract::State;
use axum::extract::rejection::BytesRejection;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde_json::{Value, json};

use crate::AppState;
use crate::error::ApiError;
use crate::scope::Caller;

/// What an agent is told for a tool it isn't given.
pub(crate) const READ_ONLY: &str = "This agent has read-only access to your notes.";

/// Protocol versions Kanso speaks, newest first (as the Node server's SDK).
pub(crate) const PROTOCOL_VERSIONS: [&str; 5] = [
    "2025-11-25",
    "2025-06-18",
    "2025-03-26",
    "2024-11-05",
    "2024-10-07",
];

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;
const SERVER_ERROR: i64 = -32000;

/// A JSON-RPC error answer.
pub(crate) struct RpcError {
    code: i64,
    message: String,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        RpcError {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS, message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR, message)
    }
}

fn error_message(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn error_response(status: StatusCode, code: i64, message: &str) -> Response {
    (status, Json(error_message(Value::Null, code, message))).into_response()
}

/// Whether an `Origin` header names this machine. Browsers send one; local
/// MCP clients usually don't. Guards against DNS rebinding.
fn is_local_origin(origin: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    let Some((_, rest)) = origin.split_once("://") else {
        return false;
    };
    let host = if let Some(v6) = rest.strip_prefix('[') {
        v6.split(']').next().unwrap_or_default()
    } else {
        rest.split([':', '/']).next().unwrap_or_default()
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// `POST /mcp`.
pub(crate) async fn handle(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    if let Some(origin) = headers.get(header::ORIGIN)
        && !is_local_origin(origin)
    {
        return error_response(
            StatusCode::FORBIDDEN,
            SERVER_ERROR,
            "Forbidden: requests from this origin are not allowed.",
        );
    }
    if let Some(version) = headers.get("mcp-protocol-version")
        && !version
            .to_str()
            .is_ok_and(|v| PROTOCOL_VERSIONS.contains(&v))
    {
        return error_response(
            StatusCode::BAD_REQUEST,
            SERVER_ERROR,
            "Bad Request: Unsupported protocol version",
        );
    }
    let bytes = match body {
        Ok(bytes) => bytes,
        Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return ApiError::too_large().into_response();
        }
        Err(rejection) => {
            return error_response(StatusCode::BAD_REQUEST, PARSE_ERROR, &rejection.body_text());
        }
    };
    let Ok(message) = serde_json::from_slice::<Value>(&bytes) else {
        return error_response(StatusCode::BAD_REQUEST, PARSE_ERROR, "Parse error");
    };
    match tokio::task::spawn_blocking(move || dispatch(&state, &caller, message)).await {
        Ok(Some(answer)) => Json(answer).into_response(),
        Ok(None) => StatusCode::ACCEPTED.into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            INTERNAL_ERROR,
            "Internal error",
        ),
    }
}

/// `GET` and `DELETE /mcp`: no event stream, no sessions.
pub(crate) async fn no_stream() -> Response {
    let mut response = error_response(
        StatusCode::METHOD_NOT_ALLOWED,
        SERVER_ERROR,
        "Method not allowed: this server answers POST requests only.",
    );
    response
        .headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    response
}

/// Answers one message or a batch; `None` when nothing needs an answer.
pub(crate) fn dispatch(state: &AppState, caller: &Caller, message: Value) -> Option<Value> {
    match message {
        Value::Array(batch) if batch.is_empty() => Some(error_message(
            Value::Null,
            INVALID_REQUEST,
            "Invalid Request",
        )),
        Value::Array(batch) => {
            let answers: Vec<Value> = batch
                .into_iter()
                .filter_map(|message| dispatch_one(state, caller, message))
                .collect();
            (!answers.is_empty()).then_some(Value::Array(answers))
        }
        message => dispatch_one(state, caller, message),
    }
}

fn dispatch_one(state: &AppState, caller: &Caller, message: Value) -> Option<Value> {
    let Value::Object(message) = message else {
        return Some(error_message(
            Value::Null,
            INVALID_REQUEST,
            "Invalid Request",
        ));
    };
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        // A response to a server request (Kanso sends none) needs no answer.
        if message.contains_key("result") || message.contains_key("error") {
            return None;
        }
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        return Some(error_message(id, INVALID_REQUEST, "Invalid Request"));
    };
    // Notifications (`notifications/initialized`, `…/cancelled`) need no answer.
    let id = message.get("id").cloned()?;
    if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(error_message(id, INVALID_REQUEST, "Invalid Request"));
    }
    Some(match call(state, caller, method, message.get("params")) {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(err) => error_message(id, err.code, &err.message),
    })
}

fn call(
    state: &AppState,
    caller: &Caller,
    method: &str,
    params: Option<&Value>,
) -> Result<Value, RpcError> {
    let param = |key: &str| params.and_then(|p| p.get(key));
    match method {
        "initialize" => {
            let requested = param("protocolVersion").and_then(Value::as_str);
            let version = requested
                .filter(|v| PROTOCOL_VERSIONS.contains(v))
                .unwrap_or(PROTOCOL_VERSIONS[0]);
            Ok(json!({
                "protocolVersion": version,
                "capabilities": {
                    "resources": { "listChanged": false },
                    "tools": { "listChanged": false },
                },
                "serverInfo": { "name": "Kanso", "version": env!("CARGO_PKG_VERSION") },
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => match caller {
            Caller::Full => Ok(json!({ "tools": schema::definitions() })),
            Caller::Agent { scope, .. } => {
                let offered: Vec<&Value> = schema::definitions()
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|tool| {
                        tool.get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| scope.allows_tool(name))
                    })
                    .collect();
                Ok(json!({ "tools": offered }))
            }
        },
        "tools/call" => {
            let name = param("name").and_then(Value::as_str).ok_or_else(|| {
                RpcError::invalid_params("Invalid params: `name` must be a string.")
            })?;
            match caller {
                Caller::Full => Ok(tools::call(state, None, name, param("arguments"))),
                Caller::Agent { token, scope } => {
                    if !scope.allows_tool(name) {
                        return Ok(tools::error_result(READ_ONLY));
                    }
                    if !state.tokens.charge(*token) {
                        return Ok(tools::error_result(
                            "This question has used as many note lookups as it may. Answer from what you have read.",
                        ));
                    }
                    let Some(visible) = crate::scope::Visibility::new(&state.store, scope) else {
                        return Ok(tools::error_result("Couldn't read the notes."));
                    };
                    let answer = if crate::scope::SUGGEST_TOOLS.contains(&name) {
                        // Staged for review; the store is only read.
                        suggest::call(state, *token, scope, &visible, name, param("arguments"))
                    } else {
                        tools::call(state, Some(&visible), name, param("arguments"))
                    };
                    state.tokens.record(*token, &visible.shown());
                    Ok(answer)
                }
            }
        }
        "resources/list" => Ok(json!({ "resources": [] })),
        "resources/templates/list" => Ok(json!({
            "resourceTemplates": [{
                "name": "note",
                "uriTemplate": format!("{}{{noteId}}", kanso_core::links::NOTE_LINK_PREFIX),
                "description": "A note data",
                "mimeType": "application/json",
            }]
        })),
        "resources/read" => {
            let uri = param("uri").and_then(Value::as_str).ok_or_else(|| {
                RpcError::invalid_params("Invalid params: `uri` must be a string.")
            })?;
            match caller {
                Caller::Full => tools::read_resource(state, None, uri),
                Caller::Agent { token, scope } => {
                    let visible = crate::scope::Visibility::new(&state.store, scope);
                    let Some(visible) = visible else {
                        return Err(RpcError::invalid_params(format!(
                            "MCP error -32602: Resource {uri} not found"
                        )));
                    };
                    let answer = tools::read_resource(state, Some(&visible), uri);
                    state.tokens.record(*token, &visible.shown());
                    answer
                }
            }
        }
        _ => Err(RpcError::new(METHOD_NOT_FOUND, "Method not found")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_local_origins_pass() {
        for ok in [
            "http://localhost:6274",
            "http://127.0.0.1",
            "https://[::1]:8080",
            "http://LOCALHOST",
        ] {
            assert!(is_local_origin(&HeaderValue::from_static(ok)), "{ok}");
        }
        for bad in [
            "http://evil.com",
            "null",
            "http://localhost.evil.com",
            "http://127.0.0.1.evil.com",
        ] {
            assert!(!is_local_origin(&HeaderValue::from_static(bad)), "{bad}");
        }
    }
}
