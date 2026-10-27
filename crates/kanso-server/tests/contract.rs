//! Kanso's contract test: the real `@inkdropapp/mcp-server` 1.5.0 runs against
//! Kanso's HTTP API over a real socket and calls every tool; the same tool
//! sequence then runs against the native `/mcp` endpoint. Both must succeed
//! (or fail) where expected, leave the store in the expected state, and
//! produce identical results once IDs, revisions, timestamps and output
//! paths are masked.
//!
//! Skipped, with a message, when `node` is not available. Set `KANSO_NODE`
//! to pick a node binary and `KANSO_MCP_SERVER_JS` to point at the
//! server's `dist/index.js`; otherwise the npx cache is tried, then
//! `npx -y @inkdropapp/mcp-server@1.5.0`.

mod common;

/// The npm package of the reference MCP server, at the version Kanso's
/// tools match.
const MCP_PACKAGE: &str = "@inkdropapp/mcp-server";
const MCP_PACKAGE_VERSION: &str = "1.5.0";

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::{PASS, PNG, USER};
use kanso_server::{ServerConfig, ServerHandle, basic_header};
use kanso_store::Store;
use serde_json::{Value, json};

const TOOL_NAMES: [&str; 18] = [
    "read-note",
    "read-backlinks",
    "search-notes",
    "list-notes",
    "list-templates",
    "create-note",
    "update-note",
    "patch-note",
    "list-tags",
    "read-tag",
    "create-tag",
    "update-tag",
    "list-notebooks",
    "read-book",
    "create-book",
    "update-book",
    "create-file",
    "read-file",
];

/// An MCP client: one JSON-RPC request in, its answer out.
trait Mcp {
    fn request(&mut self, method: &str, params: Value) -> Value;
    fn notify(&mut self, method: &str);
}

// ---------------------------------------------------------------------------
// The Node server over stdio

struct NodeMcp {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
    next_id: u64,
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    })
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// The command that starts the Node MCP server, or why it can't run.
fn node_server_command() -> Result<Command, String> {
    let node = std::env::var_os("KANSO_NODE")
        .map(PathBuf::from)
        .or_else(|| find_in_path("node"))
        .or_else(|| {
            home()
                .map(|h| h.join(".local/bin/node"))
                .filter(|p| p.is_file())
        })
        .ok_or("node is not installed")?;
    let works = Command::new(&node)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !works {
        return Err(format!("{} does not run", node.display()));
    }
    let script = std::env::var_os("KANSO_MCP_SERVER_JS")
        .map(PathBuf::from)
        .or_else(|| {
            home().map(|h| {
                h.join(".npm/_npx/bf42cecc764f1052/node_modules")
                    .join(MCP_PACKAGE)
                    .join("dist/index.js")
            })
        })
        .filter(|p| p.is_file());
    if let Some(script) = script {
        let mut command = Command::new(node);
        command.arg(script);
        return Ok(command);
    }
    let npx = node
        .parent()
        .map(|dir| dir.join("npx"))
        .filter(|p| p.is_file())
        .or_else(|| find_in_path("npx"))
        .ok_or("the MCP server is not cached and npx is not installed")?;
    let mut command = Command::new(npx);
    command.args(["-y", &format!("{MCP_PACKAGE}@{MCP_PACKAGE_VERSION}")]);
    Ok(command)
}

impl NodeMcp {
    fn spawn(mut command: Command, url: &str) -> NodeMcp {
        // Test credentials only; these override anything in the environment.
        // The variable names are the ones the Node server reads.
        let mut child = command
            .env("INKDROP_LOCAL_SERVER_URL", url)
            .env("INKDROP_LOCAL_USERNAME", USER)
            .env("INKDROP_LOCAL_PASSWORD", PASS)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start the Node MCP server");
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        NodeMcp {
            child,
            stdin,
            lines,
            next_id: 0,
        }
    }

    fn send(&mut self, message: &Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }
}

impl Mcp for NodeMcp {
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(120))
                .unwrap_or_else(|_| panic!("no answer from the Node MCP server to {method}"));
            let Ok(message) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if message["id"] == json!(id) {
                return message;
            }
        }
    }

    fn notify(&mut self, method: &str) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method }));
    }
}

impl Drop for NodeMcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// The native endpoint over HTTP

struct NativeMcp {
    client: reqwest::blocking::Client,
    url: String,
    next_id: u64,
}

impl NativeMcp {
    fn post(&self, message: &Value) -> reqwest::blocking::Response {
        self.client
            .post(&self.url)
            .header("Authorization", basic_header(USER, PASS))
            .header("Accept", "application/json, text/event-stream")
            .json(message)
            .send()
            .unwrap()
    }
}

impl Mcp for NativeMcp {
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let response = self.post(&json!({
            "jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params,
        }));
        assert_eq!(response.status().as_u16(), 200, "{method}");
        response.json().unwrap()
    }

    fn notify(&mut self, method: &str) {
        let response = self.post(&json!({ "jsonrpc": "2.0", "method": method }));
        assert_eq!(response.status().as_u16(), 202, "{method}");
    }
}

// ---------------------------------------------------------------------------
// The tool sequence

struct Step {
    label: String,
    /// The JSON-RPC result, or `{"rpcError": …}`.
    result: Value,
    expect_error: bool,
}

impl Step {
    fn is_error(&self) -> bool {
        self.result.get("rpcError").is_some() || self.result["isError"] == json!(true)
    }
}

struct Run {
    name: &'static str,
    mcp: Box<dyn Mcp>,
    base_url: String,
    steps: Vec<Step>,
    /// IDs of the documents the run created, in creation order.
    ids: Vec<String>,
    out_path: PathBuf,
    tools: Value,
    templates: Value,
}

fn text(result: &Value, index: usize) -> &str {
    result["content"][index]["text"]
        .as_str()
        .unwrap_or_default()
}

fn doc(result: &Value) -> Value {
    serde_json::from_str(text(result, 0)).unwrap_or(Value::Null)
}

fn bare(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, rest)| rest)
}

impl Run {
    fn new(name: &'static str, mcp: Box<dyn Mcp>, base_url: String, out_path: PathBuf) -> Run {
        Run {
            name,
            mcp,
            base_url,
            steps: Vec::new(),
            ids: Vec::new(),
            out_path,
            tools: Value::Null,
            templates: Value::Null,
        }
    }

    fn record(&mut self, label: String, answer: Value, expect_error: bool) -> Value {
        let result = match answer.get("result") {
            Some(result) => result.clone(),
            None => json!({ "rpcError": answer["error"] }),
        };
        self.steps.push(Step {
            label,
            result: result.clone(),
            expect_error,
        });
        result
    }

    fn call(&mut self, name: &str, arguments: Value, expect_error: bool) -> Value {
        // Distinct timestamps keep sort orders deterministic across runs.
        std::thread::sleep(Duration::from_millis(5));
        let label = format!("{name} {arguments}");
        let answer = self.mcp.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        self.record(label, answer, expect_error)
    }

    fn ok(&mut self, name: &str, arguments: Value) -> Value {
        self.call(name, arguments, false)
    }

    fn create(&mut self, name: &str, arguments: Value) -> String {
        let result = self.ok(name, arguments);
        let id = doc(&result)["id"]
            .as_str()
            .unwrap_or("<missing>")
            .to_owned();
        self.ids.push(id.clone());
        id
    }

    /// A note created over REST, for fields the tools can't set.
    fn create_over_rest(&mut self, note: Value) -> String {
        // As in `call`: a note made in the same millisecond as the one
        // before would sort either way.
        std::thread::sleep(Duration::from_millis(5));
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("{}/notes", self.base_url))
            .header("Authorization", basic_header(USER, PASS))
            .json(&note)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let id = response["id"].as_str().unwrap().to_owned();
        self.ids.push(id.clone());
        id
    }

    fn sequence(&mut self, png_path: &Path) {
        let init = self.mcp.request(
            "initialize",
            json!({ "protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": { "name": "kanso-contract", "version": "1" } }),
        );
        assert_eq!(
            init["result"]["protocolVersion"], "2025-06-18",
            "{}: {init}",
            self.name
        );
        self.mcp.notify("notifications/initialized");
        self.tools = self.mcp.request("tools/list", json!({}))["result"]["tools"].clone();
        self.templates = self.mcp.request("resources/templates/list", json!({}))["result"].clone();

        let b1 = self.create("create-book", json!({ "name": "Contract Book" }));
        let b2 = self.create(
            "create-book",
            json!({ "name": "Child Book", "parentBookId": b1 }),
        );
        let t1 = self.create(
            "create-tag",
            json!({ "name": "contract-tag", "color": "blue" }),
        );
        let t2 = self.create("create-tag", json!({ "name": "second" }));
        let n1 = self.create(
            "create-note",
            json!({ "bookId": b1, "title": "First note", "status": "active", "tags": [t1],
                    "body": "Hello world\n\n- [ ] task one\n- [x] task two\n" }),
        );
        let created_n2 = self.ok(
            "create-note",
            json!({ "bookId": b2, "title": "Second note", "tags": [t1, t2],
                    "body": format!("Links to [first](inkdrop://note/{}) and repeat repeat", bare(&n1)) }),
        );
        let n2 = doc(&created_n2)["id"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let n2_rev = doc(&created_n2)["rev"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        self.ids.push(n2.clone());
        let n3 = self.create(
            "create-note",
            json!({ "bookId": b1, "title": "Long note",
                    "body": format!("{}😀 tail {}", "x".repeat(199), "y".repeat(50)) }),
        );
        let n4 = self.create_over_rest(json!({
            "bookId": b1, "title": "From template", "body": "", "sourceTemplateId": n3,
        }));

        self.ok("list-notebooks", json!({}));
        self.ok("read-book", json!({ "bookId": b1 }));
        self.ok("read-book", json!({ "bookId": bare(&b2) }));
        self.ok("update-book", json!({ "_id": b2, "name": "Renamed Child" }));
        self.ok("update-book", json!({ "_id": b2, "parentBookId": null }));

        self.ok("list-tags", json!({}));
        self.ok("read-tag", json!({ "tagId": t1 }));
        self.ok("update-tag", json!({ "_id": t2, "color": "red" }));
        self.call(
            "update-tag",
            json!({ "_id": t2, "_rev": "1-00000000000000000000000000000000", "name": "x" }),
            true,
        );

        self.ok("list-notes", json!({}));
        self.ok("list-notes", json!({ "bookId": b1 }));
        self.ok(
            "list-notes",
            json!({ "tagIds": [t1], "sort": "title", "descending": false }),
        );
        self.ok("list-notes", json!({ "keyword": "hello", "limit": 1 }));
        self.ok("search-notes", json!({ "keyword": "hello" }));
        self.ok(
            "search-notes",
            json!({ "keyword": "title:\"Second note\"" }),
        );
        self.ok(
            "search-notes",
            json!({ "keyword": "tag:contract-tag -book:\"Contract Book\"" }),
        );
        self.ok(
            "search-notes",
            json!({ "keyword": format!("inkdrop://note/{}", bare(&n1)) }),
        );
        self.ok("search-notes", json!({ "keyword": "status:active" }));
        self.ok("search-notes", json!({ "keyword": "tail" }));

        let read = self.ok("read-note", json!({ "noteId": n1 }));
        let first_rev = doc(&read)["_rev"].as_str().unwrap_or_default().to_owned();
        self.ok("read-note", json!({ "noteId": bare(&n1) }));
        self.call("read-note", json!({ "noteId": "note:doesnotexist" }), true);
        self.ok("read-note", json!({ "noteId": n4 }));
        self.ok(
            "read-note",
            json!({ "noteId": n4, "includeTemplate": true }),
        );

        let updated = self.ok(
            "update-note",
            json!({ "_id": n1, "_rev": first_rev, "title": "First note (edited)", "status": "completed" }),
        );
        let rev = doc(&updated)["rev"].as_str().unwrap_or_default().to_owned();
        self.call(
            "update-note",
            json!({ "_id": n1, "_rev": first_rev, "title": "stale" }),
            true,
        );
        self.ok("read-note", json!({ "noteId": n1 }));
        let patched = self.ok(
            "patch-note",
            json!({ "_id": n1, "_rev": rev, "old_string": "Hello world", "new_string": "Hello Kanso" }),
        );
        let rev = doc(&patched)["rev"].as_str().unwrap_or_default().to_owned();
        self.call(
            "patch-note",
            json!({ "_id": n1, "_rev": rev, "old_string": "absent text", "new_string": "x" }),
            true,
        );
        self.call(
            "patch-note",
            json!({ "_id": n2, "_rev": n2_rev, "old_string": "repeat", "new_string": "x" }),
            true,
        );

        self.ok("read-backlinks", json!({ "noteId": n1 }));
        self.ok("read-backlinks", json!({ "noteId": n3 }));

        let f1 = self.create("create-file", json!({ "filePath": png_path }));
        self.create(
            "create-file",
            json!({ "data": STANDARD.encode(PNG), "contentType": "image/png", "name": "inline.png" }),
        );
        self.call("create-file", json!({}), true);
        self.ok("read-file", json!({ "fileId": f1 }));
        self.ok("read-file", json!({ "fileId": f1, "includeImage": false }));
        let out = self.out_path.clone();
        self.ok(
            "read-file",
            json!({ "fileId": bare(&f1), "outputPath": out }),
        );

        let answer = self.mcp.request(
            "resources/read",
            json!({ "uri": format!("inkdrop://note/{}", bare(&n1)) }),
        );
        self.record("resources/read note".into(), answer, false);

        self.call(
            "create-note",
            json!({ "bookId": "x", "title": "t", "body": "b" }),
            true,
        );
        self.call("nope", json!({}), true);
        self.ok("list-notebooks", json!({}));
    }

    /// Every step that failed when it should have succeeded, or vice versa.
    fn unexpected(&self) -> Vec<String> {
        self.steps
            .iter()
            .filter(|step| step.is_error() != step.expect_error)
            .map(|step| format!("{}: {} -> {}", self.name, step.label, step.result))
            .collect()
    }

    /// Results with IDs, revisions, timestamps and the output path masked.
    fn normalized(&self) -> Vec<String> {
        let rev = regex_lite::Regex::new(r"\b\d+-[0-9a-f]{32}\b").unwrap();
        let stamp = regex_lite::Regex::new(r#""(createdAt|updatedAt|timestamp)": ?-?\d+"#).unwrap();
        // Known, deliberate difference: when the 200-character cut splits a
        // surrogate pair, Node keeps the lone half (`\ud83d`, which strict
        // JSON parsers reject) and Kanso drops it.
        let lone_surrogate = regex_lite::Regex::new(r#"\\\\u[dD][89abAB][0-9a-fA-F]{2}""#).unwrap();
        let out = self.out_path.display().to_string();
        self.steps
            .iter()
            .map(|step| {
                // Key order in the JSON-RPC result is not significant; text
                // blocks are compared exactly.
                let mut s = canonical(&step.result);
                s = s.replace(&out, "<out>");
                for (i, id) in self.ids.iter().enumerate() {
                    s = s.replace(id.as_str(), &format!("<id{i}>"));
                }
                for (i, id) in self.ids.iter().enumerate() {
                    s = s.replace(bare(id), &format!("<id{i}>"));
                }
                s = rev.replace_all(&s, "<rev>").into_owned();
                // Inside JSON text the quotes are escaped.
                let s = s.replace("\\\"", "\"");
                let s = lone_surrogate.replace_all(&s, "\"");
                stamp
                    .replace_all(&s, |caps: &regex_lite::Captures| {
                        format!("\"{}\": <ts>", &caps[1])
                    })
                    .into_owned()
            })
            .collect()
    }
}

/// JSON text with object keys sorted, whatever order `serde_json` keeps.
fn canonical(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by_key(|(key, _)| *key);
            let fields: Vec<String> = entries
                .into_iter()
                .map(|(key, value)| format!("{}:{}", Value::from(key.as_str()), canonical(value)))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

/// The store state the sequence must leave behind.
fn verify_store(name: &str, store: &Store, ids: &[String], out: &Path) {
    let get = |i: usize| {
        store
            .get_api(&ids[i], None, true)
            .unwrap_or_else(|e| panic!("{name}: {} missing: {e}", ids[i]))
    };
    let (b2, t2, n1, n2, n4, f1, f2) = (get(1), get(3), get(4), get(5), get(7), get(8), get(9));
    assert_eq!(b2["name"], "Renamed Child", "{name}");
    assert_eq!(b2["parentBookId"], Value::Null, "{name}");
    assert_eq!(
        (t2["name"].as_str(), t2["color"].as_str()),
        (Some("second"), Some("red")),
        "{name}"
    );
    assert_eq!(n1["title"], "First note (edited)", "{name}");
    assert_eq!(n1["status"], "completed", "{name}");
    assert!(
        n1["body"].as_str().unwrap().starts_with("Hello Kanso\n"),
        "{name}: {n1}"
    );
    assert!(
        n2["body"].as_str().unwrap().ends_with("repeat repeat"),
        "{name}: patch must not apply"
    );
    assert_eq!(n4["sourceTemplateId"], ids[6], "{name}");
    for file in [&f1, &f2] {
        assert_eq!(
            file["_attachments"]["index"]["data"],
            STANDARD.encode(PNG),
            "{name}"
        );
        assert_eq!(file["contentType"], "image/png", "{name}");
    }
    assert_eq!(f1["name"], "pixel.png", "{name}");
    assert_eq!(f2["name"], "inline.png", "{name}");
    assert_eq!(
        std::fs::read(out).unwrap(),
        PNG,
        "{name}: read-file outputPath"
    );
    let counts = store.counts().unwrap();
    for (doc_type, n) in [
        (kanso_core::DocType::Note, 4),
        (kanso_core::DocType::Book, 2),
        (kanso_core::DocType::Tag, 2),
        (kanso_core::DocType::File, 2),
    ] {
        assert_eq!(
            counts.get(&doc_type).copied().unwrap_or(0),
            n,
            "{name}: {doc_type}"
        );
    }
}

struct Server {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    handle: ServerHandle,
    url: String,
}

fn start_server() -> Server {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    // Port 0: never the API's default 19840.
    let handle = kanso_server::spawn(
        store.clone(),
        ServerConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            username: USER.into(),
            password: PASS.into(),
        },
    );
    let addr = handle.local_addr().expect("server bound");
    Server {
        _dir: dir,
        store,
        handle,
        url: format!("http://{addr}"),
    }
}

fn summary(run: &Run) -> String {
    let mut per_tool: Vec<(String, usize, usize)> = TOOL_NAMES
        .iter()
        .map(|name| (name.to_string(), 0, 0))
        .collect();
    for step in &run.steps {
        if let Some(entry) = per_tool
            .iter_mut()
            .find(|(name, _, _)| step.label.starts_with(&format!("{name} ")))
        {
            entry.1 += 1;
            if step.is_error() == step.expect_error {
                entry.2 += 1;
            }
        }
    }
    per_tool
        .iter()
        .map(|(name, calls, passed)| format!("  {name}: {passed}/{calls} as expected"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn node_mcp_server_and_native_mcp_agree() {
    let files = tempfile::tempdir().unwrap();
    let png_path = files.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();

    // The native endpoint always runs.
    let native_server = start_server();
    let native_client = NativeMcp {
        client: reqwest::blocking::Client::new(),
        url: format!("{}/mcp", native_server.url),
        next_id: 0,
    };
    let mut native = Run::new(
        "native /mcp",
        Box::new(native_client),
        native_server.url.clone(),
        files.path().join("native-out.png"),
    );
    native.sequence(&png_path);
    let names: Vec<&str> = native
        .tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    // The 17 first, then Kanso's own.
    assert_eq!(names[..TOOL_NAMES.len()], TOOL_NAMES);
    assert!(
        names[TOOL_NAMES.len()..]
            .iter()
            .all(|n| n.starts_with("kanso-"))
    );
    let mut failures = native.unexpected();
    eprintln!("native /mcp:\n{}", summary(&native));
    verify_store(
        native.name,
        &native_server.store,
        &native.ids,
        &native.out_path,
    );

    let command = match node_server_command() {
        Ok(command) => command,
        Err(reason) => {
            eprintln!("skipping the Node half of the contract test: {reason}");
            assert!(failures.is_empty(), "{}", failures.join("\n"));
            return;
        }
    };
    let node_server = start_server();
    let mut node = Run::new(
        MCP_PACKAGE,
        Box::new(NodeMcp::spawn(command, &node_server.url)),
        node_server.url.clone(),
        files.path().join("node-out.png"),
    );
    node.sequence(&png_path);
    eprintln!(
        "{MCP_PACKAGE} against Kanso's HTTP API:\n{}",
        summary(&node)
    );
    let names: Vec<&str> = node
        .tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, TOOL_NAMES);
    failures.extend(node.unexpected());
    verify_store(node.name, &node_server.store, &node.ids, &node.out_path);

    // Same tool definitions and resource template.
    for (n, k) in node
        .tools
        .as_array()
        .unwrap()
        .iter()
        .zip(native.tools.as_array().unwrap())
    {
        for key in ["name", "description", "inputSchema"] {
            if n[key] != k[key] {
                failures.push(format!("tools/list {} differs in {key}", n["name"]));
            }
        }
    }
    if node.templates != native.templates {
        failures.push(format!(
            "resources/templates/list differs: {} vs {}",
            node.templates, native.templates
        ));
    }

    // Same results, step by step.
    assert_eq!(node.steps.len(), native.steps.len());
    for ((step, a), b) in node
        .steps
        .iter()
        .zip(node.normalized())
        .zip(native.normalized())
    {
        if a != b {
            failures.push(format!(
                "{} differs:\n  node:   {a}\n  native: {b}",
                step.label
            ));
        }
    }
    eprintln!(
        "compared {} results between the Node server and /mcp",
        node.steps.len()
    );
    drop(node);
    node_server.handle.stop().unwrap();
    native_server.handle.stop().unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
