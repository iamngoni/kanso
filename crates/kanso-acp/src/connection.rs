//! One running agent: requests out with ids that are never reused, a reader
//! thread routing whatever comes back, and prompt turns as blocking
//! iterators.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::framing::{FrameError, LineFramer};
use crate::process::{self, AgentSpec, ChildGuard, StderrRing};
use crate::{
    AUTH_REQUIRED, AcpError, AcpEvent, Limits, METHOD_NOT_FOUND, PROTOCOL_VERSION, Stage,
    StopReason,
};

/// What an agent asked permission for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionRequest {
    /// The agent's name, as the person sees it.
    pub agent: String,
    /// The tool call's title, as the agent gave it.
    pub title: String,
    /// The kind of tool (read, edit, execute, …), if given.
    pub kind: Option<String>,
}

/// How a pending permission request ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Allow,
    Reject,
    /// Nobody answered within [`Limits::permission`].
    TimedOut,
    /// The turn was cancelled, or the session or process ended, while it
    /// waited.
    Cancelled,
}

/// The way to answer one [`PermissionRequest`], from any thread, once.
/// Dropping it unanswered rejects the request. Kanso never answers "always",
/// so there is no way to.
pub struct PermissionReply {
    tx: Option<Sender<Decision>>,
    settled: Arc<AtomicBool>,
}

impl PermissionReply {
    /// Lets the agent use the tool this once.
    pub fn allow_once(mut self) {
        self.send(Decision::Allow);
    }

    /// Tells the agent no.
    pub fn reject(mut self) {
        self.send(Decision::Reject);
    }

    /// The request is already over: answered, timed out, or cancelled. A
    /// sheet showing it has nothing left to ask.
    pub fn is_settled(&self) -> bool {
        self.settled.load(Ordering::Relaxed)
    }

    fn send(&mut self, decision: Decision) {
        if let Some(tx) = self.tx.take() {
            tx.send(decision).ok();
        }
    }
}

impl Drop for PermissionReply {
    fn drop(&mut self) {
        self.send(Decision::Reject);
    }
}

/// Decides a permission request. It is called on the connection's reader
/// thread, so it must hand the request on and return at once; the answer
/// comes later through the [`PermissionReply`], from any thread. With no
/// answer within [`Limits::permission`] the request is rejected, and a
/// cancelled turn or an ended connection settles it as cancelled.
pub type PermissionPolicy = Arc<dyn Fn(PermissionRequest, PermissionReply) + Send + Sync>;

/// The policy when no person can be asked: every request is rejected.
pub fn reject_all() -> PermissionPolicy {
    Arc::new(|_, reply| reply.reject())
}

/// The most permission requests waiting at once; more are rejected unasked.
const MAX_PENDING_PERMISSIONS: usize = 8;

/// A permission request waiting for its decision.
struct PendingPermission {
    session: Option<String>,
    tx: Sender<Decision>,
}

/// One way an agent can sign the person in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthMethod {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// What an agent said about itself in `initialize`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentInfo {
    pub name: String,
    pub title: Option<String>,
    pub version: Option<String>,
    pub auth_methods: Vec<AuthMethod>,
    /// It can reach an MCP server over HTTP.
    pub mcp_http: bool,
    /// It offers `session/close`.
    pub close_session: bool,
}

/// An MCP server a session may reach: Kanso's read-only notes server, over
/// HTTP when the agent can, or through a program it starts. Header and
/// variable values hold a token, so they are never printed.
#[derive(Clone, PartialEq, Eq)]
pub enum McpServer {
    Http {
        name: String,
        url: String,
        headers: Vec<(String, String)>,
    },
    Stdio {
        name: String,
        command: PathBuf,
        args: Vec<String>,
        env: Vec<(String, String)>,
    },
}

impl McpServer {
    /// The entry in `session/new`'s `mcpServers`.
    pub fn to_json(&self) -> Value {
        let pairs = |pairs: &[(String, String)]| {
            pairs
                .iter()
                .map(|(name, value)| json!({ "name": name, "value": value }))
                .collect::<Vec<_>>()
        };
        match self {
            McpServer::Http { name, url, headers } => json!({
                "type": "http", "name": name, "url": url, "headers": pairs(headers),
            }),
            McpServer::Stdio {
                name,
                command,
                args,
                env,
            } => json!({
                "name": name, "command": command, "args": args, "env": pairs(env),
            }),
        }
    }
}

impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = |pairs: &[(String, String)]| {
            pairs
                .iter()
                .map(|(name, _)| format!("{name}=<redacted>"))
                .collect::<Vec<_>>()
        };
        match self {
            McpServer::Http { name, url, headers } => f
                .debug_struct("Http")
                .field("name", name)
                .field("url", url)
                .field("headers", &names(headers))
                .finish(),
            McpServer::Stdio {
                name,
                command,
                args,
                env,
            } => f
                .debug_struct("Stdio")
                .field("name", name)
                .field("command", command)
                .field("args", args)
                .field("env", &names(env))
                .finish(),
        }
    }
}

/// A mode a session can run in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionMode {
    pub id: String,
    pub name: String,
}

/// Who is waiting for the response to a request.
enum Waiter {
    Reply(Sender<Result<Value, AcpError>>),
    /// A `session/prompt`: its response ends that session's turn.
    Prompt(String),
    /// Nobody: a best-effort request such as `session/close`.
    Discard,
}

/// The most diagnostic lines kept.
const DIAGNOSTICS: usize = 200;

struct Shared {
    label: String,
    limits: Limits,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    next_id: AtomicU64,
    waiters: Mutex<HashMap<u64, Waiter>>,
    sessions: Mutex<HashMap<String, SyncSender<AcpEvent>>>,
    ended: Mutex<Option<AcpError>>,
    /// Method names, timings, error codes and exit status: never a payload.
    diagnostics: Mutex<VecDeque<String>>,
    permission: PermissionPolicy,
    pending_permissions: Mutex<HashMap<u64, PendingPermission>>,
    /// Sessions whose turn was cancelled: a request they make is cancelled
    /// without asking.
    cancelling: Mutex<HashSet<String>>,
    unknown_responses: AtomicU64,
    child: Mutex<Option<ChildGuard>>,
    stderr: Arc<Mutex<StderrRing>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Shared {
    fn note(&self, line: impl Into<String>) {
        let mut lines = lock(&self.diagnostics);
        if lines.len() == DIAGNOSTICS {
            lines.pop_front();
        }
        lines.push_back(line.into());
    }

    fn ended(&self) -> Option<AcpError> {
        lock(&self.ended).clone()
    }

    fn ended_reason(&self) -> AcpError {
        self.ended().unwrap_or(AcpError::Closed)
    }

    fn take_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_line(&self, message: &Value) -> Result<(), AcpError> {
        if let Some(reason) = self.ended() {
            return Err(reason);
        }
        let mut line =
            serde_json::to_vec(message).map_err(|err| AcpError::Protocol(err.to_string()))?;
        line.push(b'\n');
        let written = {
            let mut writer = lock(&self.writer);
            match writer.as_mut() {
                Some(writer) => writer.write_all(&line).and_then(|()| writer.flush()),
                None => Err(std::io::ErrorKind::BrokenPipe.into()),
            }
        };
        if written.is_err() {
            let code = self.exit_code();
            self.end(AcpError::Exited(code));
            return Err(self.ended_reason());
        }
        Ok(())
    }

    fn request(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
        stage: Stage,
    ) -> Result<Value, AcpError> {
        let id = self.take_id();
        let (tx, rx) = mpsc::channel();
        lock(&self.waiters).insert(id, Waiter::Reply(tx));
        let started = Instant::now();
        if let Err(err) = self.send_line(&json!({
            "jsonrpc": "2.0", "id": id, "method": method, "params": params,
        })) {
            lock(&self.waiters).remove(&id);
            return Err(err);
        }
        match rx.recv_timeout(timeout) {
            Ok(result) => {
                let outcome = match &result {
                    Ok(_) => "ok".to_owned(),
                    Err(AcpError::Agent { code, .. }) => format!("error {code}"),
                    Err(AcpError::AuthRequired(_)) => format!("error {AUTH_REQUIRED}"),
                    Err(_) => "failed".to_owned(),
                };
                self.note(format!(
                    "{method} {outcome} in {} ms",
                    started.elapsed().as_millis()
                ));
                result
            }
            Err(RecvTimeoutError::Timeout) => {
                lock(&self.waiters).remove(&id);
                self.note(format!("{method} timed out"));
                Err(AcpError::Timeout(stage))
            }
            Err(RecvTimeoutError::Disconnected) => Err(self.ended_reason()),
        }
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), AcpError> {
        self.send_line(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    fn respond(&self, id: Value, result: Value) {
        self.send_line(&json!({ "jsonrpc": "2.0", "id": id, "result": result }))
            .ok();
    }

    fn respond_error(&self, id: Value, code: i64, message: &str) {
        self.send_line(&json!({
            "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message },
        }))
        .ok();
    }

    /// Ends the connection once, for `reason`: closes the agent's stdin, fails
    /// every pending request, and tells every open turn.
    fn end(&self, reason: AcpError) {
        {
            let mut ended = lock(&self.ended);
            if ended.is_some() {
                return;
            }
            *ended = Some(reason.clone());
        }
        self.note(format!("ended: {}", ended_label(&reason)));
        lock(&self.writer).take();
        self.cancel_permissions(None);
        let waiters: Vec<Waiter> = lock(&self.waiters)
            .drain()
            .map(|(_, waiter)| waiter)
            .collect();
        for waiter in waiters {
            if let Waiter::Reply(tx) = waiter {
                tx.send(Err(reason.clone())).ok();
            }
        }
        let sessions: Vec<SyncSender<AcpEvent>> =
            lock(&self.sessions).drain().map(|(_, tx)| tx).collect();
        for tx in sessions {
            tx.try_send(AcpEvent::Error(reason.clone())).ok();
        }
    }

    fn kill(&self) {
        if let Some(child) = lock(&self.child).as_mut() {
            child.kill();
        }
    }

    fn exit_code(&self) -> Option<i32> {
        lock(&self.child)
            .as_mut()
            .and_then(|child| child.exit_code(Duration::from_millis(500)))
    }

    /// Hands `event` to the turn of session `id`. Answer text and the end of
    /// a turn wait for room, up to [`Limits::stuck`]; thinking, tool lines
    /// and costs are dropped when the turn is behind.
    fn deliver(&self, id: &str, event: AcpEvent) {
        let Some(tx) = lock(&self.sessions).get(id).cloned() else {
            return;
        };
        let must = matches!(
            event,
            AcpEvent::Text(_) | AcpEvent::Done(_) | AcpEvent::Error(_) | AcpEvent::Notice(_)
        );
        if !must {
            tx.try_send(event).ok();
            return;
        }
        let started = Instant::now();
        let mut event = event;
        loop {
            match tx.try_send(event) {
                Ok(()) | Err(TrySendError::Disconnected(_)) => return,
                Err(TrySendError::Full(back)) => {
                    if started.elapsed() >= self.limits.stuck {
                        self.end(AcpError::Stuck);
                        self.kill();
                        return;
                    }
                    event = back;
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }

    /// Handles one line from the agent. Returns false once the connection is
    /// over.
    fn handle_line(self: &Arc<Self>, line: &str) -> bool {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            self.end(AcpError::Protocol("a message that isn't JSON".to_owned()));
            self.kill();
            return false;
        };
        match message {
            Value::Array(batch) => batch.iter().for_each(|message| self.handle(message)),
            message => self.handle(&message),
        }
        self.ended().is_none()
    }

    fn handle(self: &Arc<Self>, message: &Value) {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").filter(|id| !id.is_null()).cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        match (method, id) {
            (Some(method), Some(id)) => self.agent_request(method, id, &params),
            (Some(method), None) => self.notification(method, &params),
            (None, Some(id)) => self.response(&id, message),
            (None, None) => self.note("a message without a method or id"),
        }
    }

    fn response(&self, id: &Value, message: &Value) {
        let waiter = id.as_u64().and_then(|id| lock(&self.waiters).remove(&id));
        let result = match message.get("error") {
            Some(error) => {
                let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("no reason given")
                    .to_owned();
                Err(if code == AUTH_REQUIRED {
                    AcpError::AuthRequired(message)
                } else {
                    AcpError::Agent { code, message }
                })
            }
            None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
        };
        match waiter {
            Some(Waiter::Reply(tx)) => {
                tx.send(result).ok();
            }
            Some(Waiter::Prompt(session)) => {
                let event = match result {
                    Ok(result) => AcpEvent::Done(StopReason::parse(
                        result
                            .get("stopReason")
                            .and_then(Value::as_str)
                            .unwrap_or("end_turn"),
                    )),
                    Err(err) => AcpEvent::Error(err),
                };
                self.deliver(&session, event);
            }
            Some(Waiter::Discard) => {}
            None => {
                self.unknown_responses.fetch_add(1, Ordering::Relaxed);
                self.note("a response to no request");
            }
        }
    }

    fn notification(&self, method: &str, params: &Value) {
        if method != "session/update" {
            self.note(format!("notification {}", short(method)));
            return;
        }
        let Some(session) = params.get("sessionId").and_then(Value::as_str) else {
            return;
        };
        let update = params.get("update").cloned().unwrap_or(Value::Null);
        let kind = update
            .get("sessionUpdate")
            .and_then(Value::as_str)
            .unwrap_or("");
        let event = match kind {
            "agent_message_chunk" => content_text(&update).map(AcpEvent::Text),
            "agent_thought_chunk" => content_text(&update).map(AcpEvent::Thought),
            "tool_call" | "tool_call_update" => {
                self.note(format!(
                    "{kind} {}",
                    update.get("kind").and_then(Value::as_str).unwrap_or("")
                ));
                update
                    .get("title")
                    .and_then(Value::as_str)
                    .filter(|title| !title.trim().is_empty())
                    .map(|title| AcpEvent::Tool(title.to_owned()))
            }
            "usage_update" => update.get("cost").and_then(|cost| {
                Some(AcpEvent::Cost {
                    amount: cost.get("amount")?.as_f64()?,
                    currency: cost.get("currency")?.as_str()?.to_owned(),
                })
            }),
            "plan" => {
                self.note("plan");
                None
            }
            _ => None,
        };
        if let Some(event) = event {
            self.deliver(session, event);
        }
    }

    /// A request from the agent. Only permission requests are answered; every
    /// other method (`fs/*`, `terminal/*`, elicitation, anything new) is
    /// refused and does nothing.
    fn agent_request(self: &Arc<Self>, method: &str, id: Value, params: &Value) {
        if method != "session/request_permission" {
            self.note(format!("refused {}", short(method)));
            self.respond_error(id, METHOD_NOT_FOUND, "Kanso doesn't offer this.");
            return;
        }
        let tool = params.get("toolCall").cloned().unwrap_or(Value::Null);
        let request = PermissionRequest {
            agent: self.label.clone(),
            title: tool
                .get("title")
                .and_then(Value::as_str)
                .filter(|title| !title.trim().is_empty())
                .unwrap_or("one of its tools")
                .to_owned(),
            kind: tool.get("kind").and_then(Value::as_str).map(str::to_owned),
        };
        let asked = AskedPermission {
            id,
            session: params
                .get("sessionId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            options: params.get("options").cloned().unwrap_or(Value::Null),
            request,
        };
        let (tx, rx) = mpsc::channel();
        let key = self.take_id();
        {
            let mut pending = lock(&self.pending_permissions);
            if pending.len() >= MAX_PENDING_PERMISSIONS {
                drop(pending);
                self.answer_permission(&asked, Decision::Reject);
                return;
            }
            pending.insert(
                key,
                PendingPermission {
                    session: asked.session.clone(),
                    tx: tx.clone(),
                },
            );
        }
        // A cancel or an end may have come in just before the request was
        // listed, so look again now that it is.
        let over = self.ended().is_some()
            || asked
                .session
                .as_ref()
                .is_some_and(|session| lock(&self.cancelling).contains(session));
        if over {
            lock(&self.pending_permissions).remove(&key);
            self.answer_permission(&asked, Decision::Cancelled);
            return;
        }
        let settled = Arc::new(AtomicBool::new(false));
        // The waiter owns the decision; it answers the agent, so the reader
        // goes on reading while a person decides.
        let waiting = Arc::new(Mutex::new(Some((rx, asked.clone()))));
        let shared = self.clone();
        let flag = settled.clone();
        let slot = waiting.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-acp-permission".into())
            .spawn(move || {
                let Some((rx, asked)) = lock(&slot).take() else {
                    return;
                };
                let decision = match rx.recv_timeout(shared.limits.permission) {
                    Ok(decision) => decision,
                    Err(RecvTimeoutError::Timeout) => Decision::TimedOut,
                    Err(RecvTimeoutError::Disconnected) => Decision::Reject,
                };
                lock(&shared.pending_permissions).remove(&key);
                flag.store(true, Ordering::Relaxed);
                shared.answer_permission(&asked, decision);
            });
        if spawned.is_err() {
            lock(&self.pending_permissions).remove(&key);
            settled.store(true, Ordering::Relaxed);
            if lock(&waiting).take().is_some() {
                self.answer_permission(&asked, Decision::Reject);
            }
            return;
        }
        (self.permission)(
            asked.request,
            PermissionReply {
                tx: Some(tx),
                settled,
            },
        );
    }

    /// Settles every waiting permission request of `session`, or of all
    /// sessions, as cancelled.
    fn cancel_permissions(&self, session: Option<&str>) {
        let pending = lock(&self.pending_permissions);
        for waiting in pending.values() {
            if session.is_none() || waiting.session.as_deref() == session {
                waiting.tx.send(Decision::Cancelled).ok();
            }
        }
    }

    /// Answers the agent's permission request, once: allow once, reject
    /// once, or cancelled. Never an "always" option.
    fn answer_permission(&self, asked: &AskedPermission, decision: Decision) {
        let wanted = match decision {
            Decision::Allow => Some("allow_once"),
            Decision::Reject | Decision::TimedOut => Some("reject_once"),
            Decision::Cancelled => None,
        };
        let option = wanted.and_then(|wanted| {
            asked
                .options
                .as_array()
                .into_iter()
                .flatten()
                .find(|option| option.get("kind").and_then(Value::as_str) == Some(wanted))
                .and_then(|option| option.get("optionId").and_then(Value::as_str))
        });
        let allowed = decision == Decision::Allow && option.is_some();
        self.note(format!(
            "permission {} {}",
            asked.request.kind.as_deref().unwrap_or("tool"),
            match decision {
                Decision::Allow if allowed => "allowed once",
                Decision::TimedOut => "timed out",
                Decision::Cancelled => "cancelled",
                _ => "rejected",
            }
        ));
        let outcome = match option {
            Some(option) => json!({ "outcome": "selected", "optionId": option }),
            None => json!({ "outcome": "cancelled" }),
        };
        self.respond(asked.id.clone(), json!({ "outcome": outcome }));
        if allowed || decision == Decision::Cancelled {
            return;
        }
        if let Some(session) = &asked.session {
            let why = if decision == Decision::TimedOut {
                "No one answered in time, so Kanso said no."
            } else {
                "Kanso said no."
            };
            self.deliver(
                session,
                AcpEvent::Notice(format!(
                    "{} asked to use {}. {why}",
                    self.label, asked.request.title
                )),
            );
        }
    }
}

/// A permission request as the agent sent it.
#[derive(Clone)]
struct AskedPermission {
    id: Value,
    session: Option<String>,
    options: Value,
    request: PermissionRequest,
}

/// A method name for diagnostics: short, and only what a method name has.
fn short(method: &str) -> String {
    method
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
        .take(64)
        .collect()
}

fn ended_label(reason: &AcpError) -> String {
    match reason {
        AcpError::Exited(code) => format!("exited ({code:?})"),
        AcpError::Agent { code, .. } => format!("agent error {code}"),
        AcpError::Protocol(_) => "protocol error".to_owned(),
        other => format!("{other:?}"),
    }
}

/// The text in an update's `content`: one block, or several.
fn content_text(update: &Value) -> Option<String> {
    let content = update.get("content")?;
    let blocks: Vec<&Value> = match content {
        Value::Array(blocks) => blocks.iter().collect(),
        block => vec![block],
    };
    let text: String = blocks
        .into_iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect();
    (!text.is_empty()).then_some(text)
}

fn read_loop(shared: Arc<Shared>, mut reader: Box<dyn Read + Send>) {
    let mut framer = LineFramer::new(shared.limits.max_line);
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => {
                let reason = match framer.finish() {
                    Ok(()) => AcpError::Exited(shared.exit_code()),
                    Err(_) => AcpError::Protocol("the last message was cut off".to_owned()),
                };
                shared.end(reason);
                return;
            }
            Ok(read) => match framer.push(&buf[..read]) {
                Ok(lines) => {
                    for line in lines {
                        if !shared.handle_line(&line) {
                            return;
                        }
                    }
                }
                Err(FrameError::TooLarge | FrameError::Partial) => {
                    shared.end(AcpError::TooLarge);
                    shared.kill();
                    return;
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => {
                shared.end(AcpError::Exited(shared.exit_code()));
                return;
            }
        }
    }
}

/// A running agent, initialized and ready for sessions. Dropping it closes
/// the connection and kills the agent's process tree.
pub struct AgentProcess {
    shared: Arc<Shared>,
    info: AgentInfo,
}

impl AgentProcess {
    /// Starts `spec` and initializes it.
    pub fn start(
        spec: &AgentSpec,
        limits: Limits,
        permission: PermissionPolicy,
    ) -> Result<AgentProcess, AcpError> {
        let (child, stdin, stdout, stderr) = process::spawn(spec)?;
        let ring = Arc::new(Mutex::new(StderrRing::new(limits.stderr)));
        process::drain_stderr(stderr, ring.clone());
        let timeout = if spec.is_npx() {
            limits.initialize_cold
        } else {
            limits.initialize
        };
        Self::open(
            spec.label.clone(),
            Box::new(stdout),
            Box::new(stdin),
            Some(child),
            ring,
            limits,
            permission,
            timeout,
        )
    }

    /// Initializes an agent already connected by `reader` and `writer`: an
    /// in-process agent in tests, or any other transport.
    pub fn connect(
        label: &str,
        reader: impl Read + Send + 'static,
        writer: impl Write + Send + 'static,
        limits: Limits,
        permission: PermissionPolicy,
    ) -> Result<AgentProcess, AcpError> {
        let ring = Arc::new(Mutex::new(StderrRing::new(limits.stderr)));
        Self::open(
            label.to_owned(),
            Box::new(reader),
            Box::new(writer),
            None,
            ring,
            limits,
            permission,
            limits.initialize,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open(
        label: String,
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
        child: Option<ChildGuard>,
        stderr: Arc<Mutex<StderrRing>>,
        limits: Limits,
        permission: PermissionPolicy,
        timeout: Duration,
    ) -> Result<AgentProcess, AcpError> {
        let shared = Arc::new(Shared {
            label,
            limits,
            writer: Mutex::new(Some(writer)),
            next_id: AtomicU64::new(1),
            waiters: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            ended: Mutex::new(None),
            diagnostics: Mutex::new(VecDeque::new()),
            permission,
            pending_permissions: Mutex::new(HashMap::new()),
            cancelling: Mutex::new(HashSet::new()),
            unknown_responses: AtomicU64::new(0),
            child: Mutex::new(child),
            stderr,
        });
        let reading = shared.clone();
        std::thread::Builder::new()
            .name("kanso-acp-read".into())
            .spawn(move || read_loop(reading, reader))
            .map_err(|err| AcpError::Spawn(err.to_string()))?;
        let mut process = AgentProcess {
            shared,
            info: AgentInfo::default(),
        };
        let result = process.shared.request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "clientCapabilities": {},
                "clientInfo": {
                    "name": "kanso",
                    "title": "Kanso",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            }),
            timeout,
            Stage::Initialize,
        )?;
        let version = result
            .get("protocolVersion")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if version != PROTOCOL_VERSION {
            return Err(AcpError::ProtocolVersion(version));
        }
        process.info = agent_info(&result);
        Ok(process)
    }

    pub fn info(&self) -> &AgentInfo {
        &self.info
    }

    pub fn label(&self) -> &str {
        &self.shared.label
    }

    /// Asks the agent to sign the person in with `method`. The agent may open
    /// a browser; this waits up to the answer limit.
    pub fn authenticate(&self, method: &str) -> Result<(), AcpError> {
        self.shared
            .request(
                "authenticate",
                json!({ "methodId": method }),
                self.shared.limits.answer,
                Stage::Request,
            )
            .map(|_| ())
    }

    /// Opens a session working in `cwd`, in the most restrictive mode the
    /// agent offers. No MCP servers are passed.
    pub fn new_session(&self, cwd: &Path) -> Result<Session, AcpError> {
        self.new_session_with(cwd, &[])
    }

    /// [`new_session`](Self::new_session) that may reach `mcp`. An HTTP
    /// entry is only for an agent that said it can reach one
    /// ([`AgentInfo::mcp_http`]); the caller picks.
    pub fn new_session_with(&self, cwd: &Path, mcp: &[McpServer]) -> Result<Session, AcpError> {
        let servers: Vec<Value> = mcp.iter().map(McpServer::to_json).collect();
        let result = self.shared.request(
            "session/new",
            json!({ "cwd": cwd, "mcpServers": servers }),
            self.shared.limits.session_new,
            Stage::SessionNew,
        )?;
        let id = result
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| AcpError::Protocol("a session without an id".to_owned()))?
            .to_owned();
        let modes: Vec<SessionMode> = result
            .pointer("/modes/availableModes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|mode| {
                Some(SessionMode {
                    id: mode.get("id")?.as_str()?.to_owned(),
                    name: mode
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                })
            })
            .collect();
        let current = result
            .pointer("/modes/currentModeId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let mut session = Session {
            shared: self.shared.clone(),
            id,
            modes,
            mode: current,
            close: self.info.close_session,
        };
        if let Some(safest) = safest_mode(&session.modes, session.mode.as_deref()) {
            match self.shared.request(
                "session/set_mode",
                json!({ "sessionId": session.id, "modeId": safest }),
                self.shared.limits.session_new,
                Stage::SessionNew,
            ) {
                Ok(_) => session.mode = Some(safest),
                Err(_) => self.shared.note("kept the default mode"),
            }
        }
        Ok(session)
    }

    /// Whether the agent is still running and connected.
    pub fn is_alive(&self) -> bool {
        self.shared.ended().is_none()
    }

    /// Why the connection ended, if it has.
    pub fn ended(&self) -> Option<AcpError> {
        self.shared.ended()
    }

    /// The last protocol events: method names, timings, error codes and exit
    /// status, with no note text, answers or tokens. Safe to paste.
    pub fn diagnostics(&self) -> String {
        let mut text: String = lock(&self.shared.diagnostics)
            .iter()
            .map(|line| format!("{line}\n"))
            .collect();
        let unknown = self.shared.unknown_responses.load(Ordering::Relaxed);
        if unknown > 0 {
            text.push_str(&format!("responses to no request: {unknown}\n"));
        }
        text
    }

    /// The end of what the agent wrote to stderr, with tokens taken out.
    pub fn stderr_tail(&self) -> String {
        lock(&self.shared.stderr).text().to_owned()
    }

    /// Closes the connection and kills the agent.
    pub fn close(&self) {
        self.shared.end(AcpError::Closed);
        self.shared.kill();
    }
}

impl Drop for AgentProcess {
    fn drop(&mut self) {
        self.close();
    }
}

fn agent_info(result: &Value) -> AgentInfo {
    let agent = result.get("agentInfo").cloned().unwrap_or(Value::Null);
    let text = |value: &Value, key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|text| !text.trim().is_empty())
    };
    AgentInfo {
        name: text(&agent, "name").unwrap_or_else(|| "Agent".to_owned()),
        title: text(&agent, "title"),
        version: text(&agent, "version"),
        auth_methods: result
            .get("authMethods")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|method| {
                Some(AuthMethod {
                    id: text(method, "id")?,
                    name: text(method, "name").unwrap_or_default(),
                    description: text(method, "description"),
                })
            })
            .collect(),
        mcp_http: result
            .pointer("/agentCapabilities/mcpCapabilities/http")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        close_session: result
            .pointer("/agentCapabilities/sessionCapabilities/close")
            .is_some_and(|close| !close.is_null()),
    }
}

/// How much a mode lets the agent do, from its id and name: lower is safer.
fn mode_rank(mode: &SessionMode) -> u8 {
    let text = format!("{} {}", mode.id, mode.name).to_ascii_lowercase();
    let words: Vec<&str> = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let has = |word: &str| words.contains(&word);
    if text.contains("read-only")
        || text.contains("readonly")
        || text.contains("read only")
        || text.contains("read_only")
    {
        0
    } else if has("plan") {
        1
    } else if has("ask") {
        2
    } else if has("default") {
        3
    } else {
        4
    }
}

/// The mode to switch to: the most restrictive one offered, if it is a safe
/// one (read-only, plan or ask) and safer than `current`. Ids are read from
/// the agent, never guessed.
pub(crate) fn safest_mode(modes: &[SessionMode], current: Option<&str>) -> Option<String> {
    let best = modes.iter().min_by_key(|mode| mode_rank(mode))?;
    let current_rank = current
        .and_then(|id| modes.iter().find(|mode| mode.id == id))
        .map_or(u8::MAX, mode_rank);
    (mode_rank(best) <= 2 && mode_rank(best) < current_rank && Some(best.id.as_str()) != current)
        .then(|| best.id.clone())
}

/// One conversation with an agent: one per question, or one per Ask panel
/// conversation so a follow-up keeps what came before.
pub struct Session {
    shared: Arc<Shared>,
    id: String,
    modes: Vec<SessionMode>,
    mode: Option<String>,
    close: bool,
}

impl Session {
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The mode the session runs in, if the agent has modes.
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }

    pub fn modes(&self) -> &[SessionMode] {
        &self.modes
    }

    /// Sends `text` as one text block and returns the turn as it arrives.
    /// Setting `cancel` asks the agent to stop.
    pub fn prompt(&self, text: &str, cancel: Option<Arc<AtomicBool>>) -> Result<Turn, AcpError> {
        self.prompt_within(text, cancel, self.shared.limits.answer)
    }

    /// [`Session::prompt`] with `answer` as the whole turn's time limit
    /// instead of [`Limits::answer`]: an agent task runs longer than a
    /// question.
    pub fn prompt_within(
        &self,
        text: &str,
        cancel: Option<Arc<AtomicBool>>,
        answer: std::time::Duration,
    ) -> Result<Turn, AcpError> {
        let (tx, rx) = mpsc::sync_channel(512);
        lock(&self.shared.cancelling).remove(&self.id);
        lock(&self.shared.sessions).insert(self.id.clone(), tx);
        let id = self.shared.take_id();
        lock(&self.shared.waiters).insert(id, Waiter::Prompt(self.id.clone()));
        let sent = self.shared.send_line(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "session/prompt",
            "params": {
                "sessionId": self.id,
                "prompt": [{ "type": "text", "text": text }],
            },
        }));
        if let Err(err) = sent {
            lock(&self.shared.waiters).remove(&id);
            lock(&self.shared.sessions).remove(&self.id);
            return Err(err);
        }
        self.shared.note("session/prompt sent");
        let now = Instant::now();
        Ok(Turn {
            shared: self.shared.clone(),
            session: self.id.clone(),
            rx,
            cancel,
            first_text_by: Some(now + self.shared.limits.first_text),
            deadline: now + answer,
            cancelling: None,
            timed_out: false,
            finished: false,
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        lock(&self.shared.sessions).remove(&self.id);
        if self.close && self.shared.ended().is_none() {
            let id = self.shared.take_id();
            lock(&self.shared.waiters).insert(id, Waiter::Discard);
            self.shared
                .send_line(&json!({
                    "jsonrpc": "2.0", "id": id, "method": "session/close",
                    "params": { "sessionId": self.id },
                }))
                .ok();
        }
    }
}

/// One prompt's answer, as it arrives. Ends with [`AcpEvent::Done`] or
/// [`AcpEvent::Error`]. Dropping it before the end cancels the turn, waits
/// up to the cancel grace, and kills the agent if it didn't stop.
pub struct Turn {
    shared: Arc<Shared>,
    session: String,
    rx: Receiver<AcpEvent>,
    cancel: Option<Arc<AtomicBool>>,
    first_text_by: Option<Instant>,
    deadline: Instant,
    cancelling: Option<Instant>,
    timed_out: bool,
    finished: bool,
}

impl Turn {
    /// Asks the agent to stop. What arrived stays; the turn ends with the
    /// agent's stop reason, or an error after the grace.
    pub fn cancel(&mut self) {
        if self.finished || self.cancelling.is_some() {
            return;
        }
        self.cancelling = Some(Instant::now());
        self.shared.note("session/cancel sent");
        lock(&self.shared.cancelling).insert(self.session.clone());
        self.shared.cancel_permissions(Some(&self.session));
        self.shared
            .notify("session/cancel", json!({ "sessionId": self.session }))
            .ok();
    }
}

impl Iterator for Turn {
    type Item = AcpEvent;

    fn next(&mut self) -> Option<AcpEvent> {
        loop {
            if self.finished {
                return None;
            }
            if self
                .cancel
                .as_ref()
                .is_some_and(|cancel| cancel.load(Ordering::Relaxed))
            {
                self.cancel();
            }
            if let Some(since) = self.cancelling
                && since.elapsed() >= self.shared.limits.cancel_grace
            {
                self.finished = true;
                self.shared.note("cancel ignored; killed");
                self.shared.end(AcpError::CancelIgnored);
                self.shared.kill();
                return Some(AcpEvent::Error(AcpError::CancelIgnored));
            }
            if self.cancelling.is_none() && Instant::now() >= self.deadline {
                self.timed_out = true;
                self.cancel();
                continue;
            }
            match self.rx.recv_timeout(Duration::from_millis(25)) {
                Ok(event) => {
                    match &event {
                        AcpEvent::Text(_) => self.first_text_by = None,
                        AcpEvent::Done(_) | AcpEvent::Error(_) => self.finished = true,
                        _ => {}
                    }
                    if self.timed_out && matches!(event, AcpEvent::Done(_)) {
                        return Some(AcpEvent::Error(AcpError::Timeout(Stage::Answer)));
                    }
                    return Some(event);
                }
                Err(RecvTimeoutError::Timeout) => {
                    if self
                        .first_text_by
                        .is_some_and(|by| Instant::now() >= by && self.cancelling.is_none())
                    {
                        self.first_text_by = None;
                        return Some(AcpEvent::StillWorking);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.finished = true;
                    return Some(AcpEvent::Error(self.shared.ended_reason()));
                }
            }
        }
    }
}

impl Drop for Turn {
    fn drop(&mut self) {
        if !self.finished {
            self.cancel();
            while self.next().is_some() {}
        }
        lock(&self.shared.sessions).remove(&self.session);
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionMode, mode_rank, safest_mode};
    use proptest::prelude::*;

    fn mode(id: &str, name: &str) -> SessionMode {
        SessionMode {
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn the_most_restrictive_mode_is_picked() {
        let claude = [
            mode("default", "Default"),
            mode("acceptEdits", "Accept Edits"),
            mode("plan", "Plan Mode"),
            mode("bypassPermissions", "Bypass Permissions"),
        ];
        assert_eq!(
            safest_mode(&claude, Some("default")),
            Some("plan".to_owned())
        );
        let codex = [
            mode("read-only", "Read Only"),
            mode("auto", "Auto"),
            mode("full-access", "Full Access"),
        ];
        assert_eq!(
            safest_mode(&codex, Some("auto")),
            Some("read-only".to_owned())
        );
        assert_eq!(safest_mode(&codex, Some("read-only")), None);
        let unsafe_only = [mode("code", "Code"), mode("yolo", "YOLO")];
        assert_eq!(safest_mode(&unsafe_only, Some("code")), None);
        assert_eq!(safest_mode(&[], None), None);
    }

    fn gen_mode() -> impl Strategy<Value = SessionMode> {
        (
            prop::sample::select(vec![
                "read-only",
                "readonly",
                "plan",
                "ask",
                "default",
                "code",
                "auto",
                "full-access",
                "bypassPermissions",
                "acceptEdits",
                "x",
            ]),
            "[A-Za-z ]{0,10}",
        )
            .prop_map(|(id, name)| mode(id, &name))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A switch only ever goes to a mode the agent offered, that is a
        /// safe kind, and is safer than the one it had.
        #[test]
        fn prop_a_mode_switch_never_widens_what_the_agent_may_do(
            modes in prop::collection::vec(gen_mode(), 0..6),
            current in prop::option::of(0usize..6),
        ) {
            let current = current.and_then(|ix| modes.get(ix)).map(|mode| mode.id.clone());
            if let Some(picked) = safest_mode(&modes, current.as_deref()) {
                let picked = modes.iter().find(|mode| mode.id == picked).unwrap();
                prop_assert!(mode_rank(picked) <= 2);
                if let Some(current) = current.as_deref().and_then(|id| modes.iter().find(|mode| mode.id == id)) {
                    prop_assert!(mode_rank(picked) < mode_rank(current));
                }
                prop_assert!(modes.iter().all(|mode| mode_rank(mode) >= mode_rank(picked)));
            }
        }
    }
}
