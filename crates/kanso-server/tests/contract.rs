//! Kanso's native MCP contract test: every tool is called over a real HTTP
//! socket, in the order a client uses them. Each call must succeed (or fail)
//! where expected, the results must have the documented shape, and the store
//! must be left in the expected state.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::PNG;
use kanso_server::{ServerConfig, ServerHandle};
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
            .header("Authorization", common::auth())
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
    /// Whether the tools may read and write local files. The Node server
    /// does so itself; Kanso's `/mcp` never does for an API key.
    local_paths: bool,
    /// Steps whose results differ because of that, left out of the
    /// step-by-step comparison.
    local_path_steps: Vec<usize>,
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
            local_paths: true,
            local_path_steps: Vec::new(),
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

    /// The `pixel.png` file, created over REST.
    fn create_file_over_rest(&mut self) -> String {
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("{}/files", self.base_url))
            .header("Authorization", common::auth())
            .json(&json!({
                "name": "pixel.png",
                "contentType": "image/png",
                "_attachments": { "index": { "content_type": "image/png", "data": STANDARD.encode(PNG) } },
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        let id = response["id"].as_str().unwrap().to_owned();
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
            .header("Authorization", common::auth())
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
                    "body": format!("Links to [first](kanso://note/{}) and repeat repeat", bare(&n1)) }),
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
            json!({ "keyword": format!("kanso://note/{}", bare(&n1)) }),
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

        let f1 = if self.local_paths {
            self.create("create-file", json!({ "filePath": png_path }))
        } else {
            // A key is never given local file paths: the path is refused,
            // and the file is made over REST instead.
            self.local_path_steps.push(self.steps.len());
            self.call("create-file", json!({ "filePath": png_path }), true);
            self.create_file_over_rest()
        };
        self.create(
            "create-file",
            json!({ "data": STANDARD.encode(PNG), "contentType": "image/png", "name": "inline.png" }),
        );
        self.call("create-file", json!({}), true);
        self.ok("read-file", json!({ "fileId": f1 }));
        self.ok("read-file", json!({ "fileId": f1, "includeImage": false }));
        let out = self.out_path.clone();
        if !self.local_paths {
            self.local_path_steps.push(self.steps.len());
        }
        self.call(
            "read-file",
            json!({ "fileId": bare(&f1), "outputPath": out }),
            !self.local_paths,
        );

        let answer = self.mcp.request(
            "resources/read",
            json!({ "uri": format!("kanso://note/{}", bare(&n1)) }),
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
    if out.exists() {
        assert_eq!(
            std::fs::read(out).unwrap(),
            PNG,
            "{name}: read-file outputPath"
        );
    }
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
        },
        common::keys_for(&store),
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
fn native_mcp_calls_every_tool_and_leaves_the_expected_store() {
    let files = tempfile::tempdir().unwrap();
    let png_path = files.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();

    let server = start_server();
    let client = NativeMcp {
        client: reqwest::blocking::Client::new(),
        url: format!("{}/mcp", server.url),
        next_id: 0,
    };
    let mut run = Run::new(
        "native /mcp",
        Box::new(client),
        server.url.clone(),
        files.path().join("native-out.png"),
    );
    run.local_paths = false;
    run.sequence(&png_path);
    assert!(
        !run.out_path.exists(),
        "a key's read-file wrote a local file"
    );
    let names: Vec<&str> = run
        .tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names[..TOOL_NAMES.len()], TOOL_NAMES);
    assert!(
        names[TOOL_NAMES.len()..]
            .iter()
            .all(|n| n.starts_with("kanso-"))
    );
    eprintln!("native /mcp:\n{}", summary(&run));
    verify_store(run.name, &server.store, &run.ids, &run.out_path);
    let failures = run.unexpected();
    server.handle.stop().unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
