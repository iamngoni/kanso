//! A scriptable ACP agent for tests. It speaks the same newline-delimited
//! JSON-RPC as a real agent, over stdio (`kanso-acp-fake`) or in process
//! over pipes ([`in_process`]), and can stream, hang, crash, ask
//! permission, request the file system or a terminal, or misbehave. With
//! [`Script::Notes`] it uses the MCP server it was given in `session/new`,
//! over HTTP or by starting the stdio command, as a real agent would.

use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

use serde_json::{Value, json};

/// What the fake agent does when prompted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Script {
    /// Streams `answer` in three pieces, then ends the turn.
    Answer(String),
    /// Streams back the prompt it was sent.
    Echo,
    /// Streams back the `initialize` params it got.
    EchoInit,
    /// Streams back the names of its environment and its working folder.
    Env,
    /// Never answers; stops when cancelled.
    Hang,
    /// Never answers and ignores cancel.
    IgnoreCancel,
    /// Sends one piece, then exits with code 3.
    Crash,
    /// Asks permission for a tool, then answers with what it was told.
    Permission,
    /// Says "before ", then asks permission `count` times in a row, then
    /// answers "permissions=" and the option each got, or "cancelled".
    Permissions(usize),
    /// Asks Kanso to read a file and open a terminal, then answers with the
    /// error codes it got.
    Fs,
    /// Sends one line of 9 MB.
    Huge,
    /// Sends a line that isn't JSON.
    BadJson,
    /// Answers `session/new` with "authentication required" until
    /// `authenticate` is called.
    Auth,
    /// Answers `initialize` with protocol version 2.
    V2,
    /// Starts a child process that sleeps, and answers with its pid.
    SpawnChild,
    /// Waits `ms` before answering "late".
    Slow(u64),
    /// Sends `count` pieces of text as fast as it can.
    Flood(usize),
    /// Searches the MCP server from `session/new` for the question's last
    /// word, reads the first note found, and cites it: "<text> [[Title]]
    /// (via http|stdio, turn N)". With no server: "no notes server".
    Notes,
    /// Answers HTTP MCP is not supported; otherwise as [`Script::Notes`].
    NotesStdio,
    /// Thinks, starts a tool call, reports it completed in an update that
    /// has no title, then answers "done".
    Tools,
    /// An agent task: searches the MCP server for the task's last word,
    /// proposes merging the notes found into one new note in the first
    /// one's notebook (`kanso-propose-merge`), calls `kanso-task-done` and
    /// answers "Done. <what the server said>".
    Task,
    /// Answers each prompt with a report of where it is: "session=<id>
    /// turn=<n> saw=<text>". It offers `session/load`: a session it doesn't
    /// know yet, but whose id is of its own kind, is picked up after the two
    /// messages of its history are replayed, and its turns go on from 2.
    Report,
    /// As [`Script::Report`], but without `loadSession`.
    ReportNoLoad,
    /// As [`Script::Report`], but it no longer has the conversation:
    /// `session/load` fails with "Session not found".
    ReportGone,
}

impl Script {
    /// The script named by the fake binary's arguments.
    pub fn from_args(args: &[String]) -> Script {
        let arg = |ix: usize| args.get(ix).cloned().unwrap_or_default();
        match arg(0).as_str() {
            "echo" => Script::Echo,
            "echo-init" => Script::EchoInit,
            "env" => Script::Env,
            "hang" => Script::Hang,
            "ignore-cancel" => Script::IgnoreCancel,
            "crash" => Script::Crash,
            "permission" => Script::Permission,
            "permissions" => Script::Permissions(arg(1).parse().unwrap_or(1)),
            "fs" => Script::Fs,
            "huge" => Script::Huge,
            "bad-json" => Script::BadJson,
            "auth" => Script::Auth,
            "v2" => Script::V2,
            "spawn-child" => Script::SpawnChild,
            "slow" => Script::Slow(arg(1).parse().unwrap_or(1000)),
            "flood" => Script::Flood(arg(1).parse().unwrap_or(1000)),
            "notes" => Script::Notes,
            "notes-stdio" => Script::NotesStdio,
            "task" => Script::Task,
            "tools" => Script::Tools,
            "report" => Script::Report,
            "report-no-load" => Script::ReportNoLoad,
            "report-gone" => Script::ReportGone,
            "sleep" => {
                std::thread::sleep(Duration::from_secs(600));
                Script::Hang
            }
            _ => Script::Answer(if arg(1).is_empty() {
                "OK".to_owned()
            } else {
                arg(1)
            }),
        }
    }
}

struct Fake<W: Write> {
    out: W,
    script: Script,
    init: Value,
    signed_in: bool,
    /// The id of a prompt still open (hang scripts).
    open_prompt: Option<Value>,
    next_id: u64,
    children: Vec<std::process::Child>,
    /// Each session's MCP servers and how many prompts it has had.
    sessions: Vec<(String, Value, usize)>,
}

/// One JSON-RPC request to an MCP server entry from `session/new`; the
/// answer, or why there is none.
fn mcp_call(server: &Value, method: &str, params: Value) -> Result<Value, String> {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let pairs = |key: &str| -> Vec<(String, String)> {
        server
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|pair| {
                Some((
                    pair.get("name")?.as_str()?.to_owned(),
                    pair.get("value")?.as_str()?.to_owned(),
                ))
            })
            .collect()
    };
    if server.get("type").and_then(Value::as_str) == Some("http") {
        let url = server
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let rest = url.strip_prefix("http://").ok_or("not an http url")?;
        let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
        let mut stream = std::net::TcpStream::connect(host).map_err(|err| err.to_string())?;
        let body = message.to_string();
        let mut request = format!(
            "POST /{path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, value) in pairs("headers") {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str("\r\n");
        request.push_str(&body);
        stream
            .write_all(request.as_bytes())
            .map_err(|err| err.to_string())?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|err| err.to_string())?;
        let status = response.split(' ').nth(1).unwrap_or_default().to_owned();
        if status != "200" {
            return Err(format!("HTTP {status}"));
        }
        let (_, body) = response.split_once("\r\n\r\n").ok_or("no body")?;
        return serde_json::from_str(body).map_err(|err| err.to_string());
    }
    let command = server
        .get("command")
        .and_then(Value::as_str)
        .ok_or("no command")?;
    let args: Vec<String> = server
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|arg| arg.as_str().map(str::to_owned))
        .collect();
    let mut child = std::process::Command::new(command)
        .args(&args)
        .envs(pairs("env"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    writeln!(stdin, "{message}").map_err(|err| err.to_string())?;
    drop(stdin);
    let mut line = String::new();
    BufReader::new(child.stdout.take().ok_or("no stdout")?)
        .read_line(&mut line)
        .map_err(|err| err.to_string())?;
    child.wait().ok();
    serde_json::from_str(&line).map_err(|err| err.to_string())
}

/// What [`Script::Task`] does for `text`, using `server`.
fn do_task(server: &Value, text: &str) -> String {
    let keyword = text
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .trim_matches(|c: char| !c.is_alphanumeric());
    let found = match mcp_call(
        server,
        "tools/call",
        json!({ "name": "search-notes", "arguments": { "keyword": keyword } }),
    ) {
        Ok(found) => found,
        Err(why) => return format!("the notes server failed: {why}"),
    };
    let notes: Vec<Value> = serde_json::from_str(&tool_text(&found)).unwrap_or_default();
    let sources: Vec<Value> = notes
        .iter()
        .filter_map(|note| Some(json!({ "_id": note.get("_id")?, "_rev": note.get("_rev")? })))
        .collect();
    let book = notes
        .first()
        .and_then(|note| note.get("bookId"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let merged = mcp_call(
        server,
        "tools/call",
        json!({ "name": "kanso-propose-merge", "arguments": {
            "sources": sources, "bookId": book, "title": "Merged", "body": "All of it.",
        } }),
    )
    .unwrap_or_default();
    let said = tool_text(&merged);
    let done = mcp_call(
        server,
        "tools/call",
        json!({ "name": "kanso-task-done", "arguments": { "summary": "Merged them." } }),
    )
    .unwrap_or_default();
    let _ = tool_text(&done);
    format!("Done. {said}")
}

/// The text of a tool result.
fn tool_text(answer: &Value) -> String {
    answer
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// What [`Script::Notes`] answers for `question`, using `server`.
fn answer_from_notes(server: &Value, question: &str, turn: usize) -> String {
    let via = if server.get("type").and_then(Value::as_str) == Some("http") {
        "http"
    } else {
        "stdio"
    };
    let keyword = question
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .trim_matches(|c: char| !c.is_alphanumeric());
    let found = match mcp_call(
        server,
        "tools/call",
        json!({ "name": "search-notes", "arguments": { "keyword": keyword } }),
    ) {
        Ok(found) => found,
        Err(why) => return format!("the notes server failed: {why} (via {via}, turn {turn})"),
    };
    let notes: Vec<Value> = serde_json::from_str(&tool_text(&found)).unwrap_or_default();
    let Some(id) = notes
        .first()
        .and_then(|note| note.get("_id"))
        .and_then(Value::as_str)
    else {
        return format!("nothing found for {keyword} (via {via}, turn {turn})");
    };
    let read = mcp_call(
        server,
        "tools/call",
        json!({ "name": "read-note", "arguments": { "noteId": id } }),
    )
    .unwrap_or_default();
    let note: Value = serde_json::from_str(&tool_text(&read)).unwrap_or_default();
    let title = note
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    format!("{} [[{title}]] (via {via}, turn {turn})", body.trim())
}

impl<W: Write> Fake<W> {
    fn send(&mut self, message: Value) {
        let mut line = serde_json::to_vec(&message).unwrap_or_default();
        line.push(b'\n');
        self.out.write_all(&line).ok();
        self.out.flush().ok();
    }

    fn reply(&mut self, id: &Value, result: Value) {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    fn error(&mut self, id: &Value, code: i64, message: &str) {
        self.send(json!({
            "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message },
        }));
    }

    fn chunk(&mut self, session: &str, text: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": session,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": { "type": "text", "text": text },
                },
            },
        }));
    }

    fn answer(&mut self, id: &Value, session: &str, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let third = chars.len().div_ceil(3).max(1);
        for piece in chars.chunks(third) {
            let piece: String = piece.iter().collect();
            self.chunk(session, &piece);
        }
        self.reply(id, json!({ "stopReason": "end_turn" }));
    }

    /// Sends a request and reads lines until its response, answering
    /// nothing else in between.
    fn ask<R: BufRead>(&mut self, input: &mut R, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id + 900;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line).unwrap_or(0) == 0 {
                return Value::Null;
            }
            if let Ok(message) = serde_json::from_str::<Value>(&line)
                && message.get("id").and_then(Value::as_u64) == Some(id)
                && message.get("method").is_none()
            {
                return message;
            }
        }
    }

    fn prompt<R: BufRead>(&mut self, input: &mut R, id: Value, params: &Value) {
        let session = params
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let text: String = params
            .get("prompt")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect();
        let turn = match self.sessions.iter_mut().find(|(id, _, _)| *id == session) {
            Some((_, _, turns)) => {
                *turns += 1;
                *turns
            }
            None => 1,
        };
        match self.script.clone() {
            Script::Notes | Script::NotesStdio => {
                let question = text
                    .lines()
                    .rev()
                    .find_map(|line| line.strip_prefix("Question: "))
                    .unwrap_or_default()
                    .to_owned();
                let server = self
                    .sessions
                    .iter()
                    .find(|(id, _, _)| *id == session)
                    .and_then(|(_, servers, _)| servers.get(0).cloned());
                let answer = match server {
                    Some(server) => answer_from_notes(&server, &question, turn),
                    None => format!("no notes server (turn {turn})"),
                };
                self.answer(&id, &session, &answer);
            }
            Script::Task => {
                let server = self
                    .sessions
                    .iter()
                    .find(|(id, _, _)| *id == session)
                    .and_then(|(_, servers, _)| servers.get(0).cloned());
                let answer = match server {
                    Some(server) => do_task(&server, &text),
                    None => "no notes server".to_owned(),
                };
                self.answer(&id, &session, &answer);
            }
            Script::Answer(answer) => self.answer(&id, &session, &answer),
            Script::Tools => {
                let update = |this: &mut Self, update: Value| {
                    this.send(json!({
                        "jsonrpc": "2.0",
                        "method": "session/update",
                        "params": { "sessionId": session, "update": update },
                    }));
                };
                update(
                    self,
                    json!({
                        "sessionUpdate": "agent_thought_chunk",
                        "content": { "type": "text", "text": "hmm" },
                    }),
                );
                update(
                    self,
                    json!({
                        "sessionUpdate": "tool_call",
                        "toolCallId": "c1",
                        "title": "Read the plan",
                        "kind": "read",
                        "status": "pending",
                    }),
                );
                update(
                    self,
                    json!({
                        "sessionUpdate": "tool_call_update",
                        "toolCallId": "c1",
                        "status": "completed",
                    }),
                );
                self.answer(&id, &session, "done");
            }
            Script::Echo => self.answer(&id, &session, &text),
            Script::EchoInit => {
                let init = self.init.to_string();
                self.answer(&id, &session, &init);
            }
            Script::Env => {
                let mut names: Vec<String> = std::env::vars().map(|(name, _)| name).collect();
                names.sort();
                let cwd = std::env::current_dir().unwrap_or_default();
                let text = format!("cwd={} env={}", cwd.display(), names.join(","));
                self.answer(&id, &session, &text);
            }
            Script::Hang | Script::IgnoreCancel => self.open_prompt = Some(id),
            Script::Crash => {
                self.chunk(&session, "partial ");
                std::process::exit(3);
            }
            Script::Permission => {
                let response = self.ask(
                    input,
                    "session/request_permission",
                    json!({
                        "sessionId": session,
                        "toolCall": { "toolCallId": "t1", "title": "Read ~/.ssh/id_ed25519", "kind": "read" },
                        "options": [
                            { "optionId": "yes", "name": "Allow", "kind": "allow_once" },
                            { "optionId": "always", "name": "Always", "kind": "allow_always" },
                            { "optionId": "no", "name": "Reject", "kind": "reject_once" },
                        ],
                    }),
                );
                let outcome = response
                    .pointer("/result/outcome/optionId")
                    .and_then(Value::as_str)
                    .unwrap_or("cancelled")
                    .to_owned();
                self.answer(&id, &session, &format!("permission={outcome}"));
            }
            Script::Permissions(count) => {
                self.chunk(&session, "before ");
                let mut outcomes = Vec::new();
                for ix in 0..count {
                    let response = self.ask(
                        input,
                        "session/request_permission",
                        json!({
                            "sessionId": session,
                            "toolCall": { "toolCallId": format!("t{ix}"), "title": format!("Tool {ix}"), "kind": "execute" },
                            "options": [
                                { "optionId": "always", "name": "Always", "kind": "allow_always" },
                                { "optionId": "yes", "name": "Allow", "kind": "allow_once" },
                                { "optionId": "never", "name": "Never", "kind": "reject_always" },
                                { "optionId": "no", "name": "Reject", "kind": "reject_once" },
                            ],
                        }),
                    );
                    outcomes.push(
                        response
                            .pointer("/result/outcome/optionId")
                            .and_then(Value::as_str)
                            .unwrap_or("cancelled")
                            .to_owned(),
                    );
                }
                self.answer(
                    &id,
                    &session,
                    &format!("permissions={}", outcomes.join(",")),
                );
            }
            Script::Fs => {
                let fs = self.ask(
                    input,
                    "fs/read_text_file",
                    json!({ "sessionId": session, "path": "/etc/hosts" }),
                );
                let terminal = self.ask(
                    input,
                    "terminal/create",
                    json!({ "sessionId": session, "command": "id" }),
                );
                let elicit = self.ask(input, "elicitation/create", json!({ "sessionId": session }));
                let code = |message: &Value| {
                    message
                        .pointer("/error/code")
                        .and_then(Value::as_i64)
                        .map_or("ok".to_owned(), |code| code.to_string())
                };
                let text = format!(
                    "fs={} terminal={} elicitation={}",
                    code(&fs),
                    code(&terminal),
                    code(&elicit)
                );
                self.answer(&id, &session, &text);
            }
            Script::Huge => {
                let huge = "x".repeat(9 * 1024 * 1024);
                self.chunk(&session, &huge);
                self.reply(&id, json!({ "stopReason": "end_turn" }));
            }
            Script::BadJson => {
                self.out.write_all(b"this is not json\n").ok();
                self.out.flush().ok();
            }
            Script::SpawnChild => {
                let me = std::env::current_exe().unwrap_or_default();
                match std::process::Command::new(me).arg("sleep").spawn() {
                    Ok(child) => {
                        let pid = child.id().to_string();
                        self.children.push(child);
                        self.answer(&id, &session, &pid);
                    }
                    Err(err) => self.answer(&id, &session, &format!("no child: {err}")),
                }
            }
            Script::Slow(ms) => {
                std::thread::sleep(Duration::from_millis(ms));
                self.answer(&id, &session, "late");
            }
            Script::Flood(count) => {
                for _ in 0..count {
                    self.chunk(&session, "word ");
                }
                self.reply(&id, json!({ "stopReason": "end_turn" }));
            }
            Script::Auth | Script::V2 => self.answer(&id, &session, "OK"),
            Script::Report | Script::ReportNoLoad | Script::ReportGone => {
                let report = format!("session={session} turn={turn} saw={text}");
                self.answer(&id, &session, &report);
            }
        }
    }

    /// `session/load`: replays the history, then answers like `session/new`.
    fn load(&mut self, id: &Value, params: &Value) {
        let session = params
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if self.script != Script::Report || !session.starts_with("fake-session-") {
            self.error(id, crate::INVALID_PARAMS, "Session not found");
            return;
        }
        let servers = params.get("mcpServers").cloned().unwrap_or(json!([]));
        // Its history is one turn old.
        self.sessions.retain(|(have, _, _)| *have != session);
        self.sessions.push((session.clone(), servers, 1));
        for (kind, text) in [
            ("user_message_chunk", "old question"),
            ("agent_message_chunk", "old answer"),
        ] {
            self.send(json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": session,
                    "update": { "sessionUpdate": kind, "content": { "type": "text", "text": text } },
                },
            }));
        }
        self.reply(
            id,
            json!({
                "modes": {
                    "currentModeId": "code",
                    "availableModes": [
                        { "id": "code", "name": "Code" },
                        { "id": "read-only", "name": "Read only" },
                    ],
                },
            }),
        );
    }

    fn handle<R: BufRead>(&mut self, input: &mut R, message: Value) {
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let Some(id) = id else {
            if method == "session/cancel"
                && self.script != Script::IgnoreCancel
                && let Some(open) = self.open_prompt.take()
            {
                self.reply(&open, json!({ "stopReason": "cancelled" }));
            }
            return;
        };
        match method {
            "initialize" => {
                self.init = params;
                let version = if self.script == Script::V2 { 2 } else { 1 };
                self.reply(
                    &id,
                    json!({
                        "protocolVersion": version,
                        "agentCapabilities": {
                            "mcpCapabilities": { "http": self.script != Script::NotesStdio },
                            "sessionCapabilities": { "close": {} },
                            "loadSession": matches!(self.script, Script::Report | Script::ReportGone),
                        },
                        "agentInfo": { "name": "fake-agent", "title": "Fake Agent", "version": "1.2.3" },
                        "authMethods": [{ "id": "fake-login", "name": "Log in", "description": "Opens a browser" }],
                    }),
                );
            }
            "authenticate" => {
                self.signed_in = true;
                self.reply(&id, json!({}));
            }
            "session/new" => {
                if self.script == Script::Auth && !self.signed_in {
                    self.error(&id, crate::AUTH_REQUIRED, "Authentication required");
                    return;
                }
                let session = format!("fake-session-{}", self.sessions.len() + 1);
                let servers = params.get("mcpServers").cloned().unwrap_or(json!([]));
                self.sessions.push((session.clone(), servers, 0));
                self.reply(
                    &id,
                    json!({
                        "sessionId": session,
                        "modes": {
                            "currentModeId": "code",
                            "availableModes": [
                                { "id": "code", "name": "Code" },
                                { "id": "ask", "name": "Ask" },
                                { "id": "read-only", "name": "Read only" },
                            ],
                        },
                    }),
                );
            }
            "session/load" => self.load(&id, &params),
            "session/set_mode" | "session/close" => self.reply(&id, json!({})),
            "session/prompt" => self.prompt(input, id, &params),
            _ => self.error(&id, crate::METHOD_NOT_FOUND, "Unknown method"),
        }
    }
}

/// Runs the fake agent on `input` and `output` until `input` closes.
pub fn serve(input: impl Read, output: impl Write, script: Script) {
    let mut input = BufReader::new(input);
    let mut fake = Fake {
        out: output,
        script,
        init: Value::Null,
        signed_in: false,
        open_prompt: None,
        next_id: 0,
        children: Vec::new(),
        sessions: Vec::new(),
    };
    let mut line = String::new();
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                if let Ok(message) = serde_json::from_str::<Value>(&line) {
                    fake.handle(&mut input, message);
                }
            }
        }
    }
    for mut child in fake.children.drain(..) {
        child.kill().ok();
        child.wait().ok();
    }
}

/// A fake agent on a thread, connected by pipes: what to read its messages
/// from, and what to write Kanso's to.
pub fn in_process(script: Script) -> std::io::Result<(std::io::PipeReader, std::io::PipeWriter)> {
    let (from_agent, agent_out) = std::io::pipe()?;
    let (agent_in, to_agent) = std::io::pipe()?;
    std::thread::Builder::new()
        .name("kanso-acp-fake".into())
        .spawn(move || serve(agent_in, agent_out, script))?;
    Ok((from_agent, to_agent))
}
