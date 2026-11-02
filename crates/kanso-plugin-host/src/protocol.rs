//! The messages between the app and a plugin's host: newline-delimited
//! JSON over the host's standard input and output.
//!
//! One request is in flight at a time and the plugin's API is synchronous.
//! The app sends [`ToHost::Start`] once, then one [`ToHost::Invoke`],
//! [`ToHost::Render`] or [`ToHost::Press`] at a time. While the plugin runs
//! it may ask the app for things with [`FromHost::Host`], each answered by a
//! [`ToHost::Reply`] with the same `id`, and it ends with
//! [`FromHost::Result`]:
//!
//! ```text
//! {"id":7,"op":"invoke","command":"count-words","args":{}}
//! {"id":7,"op":"host","call":"notes.list","args":{"bookId":"book:work","limit":50}}
//! {"id":7,"op":"reply","ok":true,"value":[{"id":"note:abc","title":"Draft"}]}
//! {"id":7,"op":"result","ok":true,"value":{"text":"Words: 412"}}
//! ```
//!
//! Every line is capped: [`MAX_FROM_HOST`] for what the plugin sends,
//! [`MAX_TO_HOST`] for what the app sends (its code comes in `start`). A
//! longer line is an error, read no further than the cap.

use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The largest line a plugin may send the app: 1 MB.
pub const MAX_FROM_HOST: usize = 1 << 20;
/// The largest line the app sends a host: the 2 MB entry script, escaped,
/// with room to spare.
pub const MAX_TO_HOST: usize = 6 << 20;

/// Every call a plugin may make, by name. A `host` message with any other
/// name is refused before it reaches the app's router.
pub const CALLS: [&str; 15] = [
    "notes.list",
    "notes.get",
    "notes.create",
    "notes.update",
    "notes.pluginData.get",
    "notes.pluginData.set",
    "attachments.get",
    "editor.noteId",
    "editor.selection",
    "editor.replaceSelection",
    "ui.panel",
    "settings.get",
    "settings.set",
    "secrets.get",
    "fetch",
];

/// Whether `name` is one of [`CALLS`]. Pure.
pub fn is_call(name: &str) -> bool {
    CALLS.contains(&name)
}

/// The plugin, as the host is told about it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// The CPU time one request may use, in milliseconds. The host stops the
/// script when it is spent; the app kills the process at its own, longer,
/// wall deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Budgets {
    pub command_ms: u64,
    pub render_ms: u64,
    pub startup_ms: u64,
    /// Bytes the engine may hold.
    pub memory: usize,
    /// Bytes of native stack the engine may use.
    pub stack: usize,
}

impl Budgets {
    /// The design's limits: 2 s a command, 3 s a render, 1 s to start,
    /// 64 MiB and a 1 MiB stack.
    pub const DEFAULT: Budgets = Budgets {
        command_ms: 2_000,
        render_ms: 3_000,
        startup_ms: 1_000,
        memory: 64 << 20,
        stack: 1 << 20,
    };
}

/// When a command is offered in the palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum When {
    #[default]
    Always,
    /// Only while a note is open.
    NoteOpen,
    /// Only while text is selected in the open note.
    Selection,
}

/// A command the plugin registered with `kanso.commands.register`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    /// Unique within the plugin: `[a-z0-9-]{1,64}`.
    pub id: String,
    /// The palette label, at most 80 characters.
    pub title: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub when: When,
}

/// What the plugin set up while its script ran at start.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registered {
    #[serde(default)]
    pub commands: Vec<CommandSpec>,
    /// Fenced languages it gave a renderer for.
    #[serde(default)]
    pub fences: Vec<String>,
    /// Button IDs it gave a press callback for.
    #[serde(default)]
    pub presses: Vec<String>,
}

/// The colors a fence is drawn for, as `#rrggbb`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub dark: bool,
    pub background: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub border: String,
    pub surface: String,
}

/// Why a call or request failed. `code` is one of the `CODE_` constants.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub code: String,
    pub message: String,
}

/// Nothing by that ID the plugin may see. A locked, trashed or
/// out-of-scope note is "not found" too, never "forbidden".
pub const CODE_NOT_FOUND: &str = "not_found";
/// The note changed since the plugin read it (409).
pub const CODE_CONFLICT: &str = "conflict";
/// The plugin wasn't granted this.
pub const CODE_DENIED: &str = "denied";
/// The arguments are wrong.
pub const CODE_INVALID: &str = "invalid";
/// A cap or rate limit was reached.
pub const CODE_LIMIT: &str = "limit";
/// The request ran out of time.
pub const CODE_TIMEOUT: &str = "timeout";
/// The script threw, or the engine couldn't run it.
pub const CODE_THROWN: &str = "thrown";
/// The app couldn't do it just now.
pub const CODE_UNAVAILABLE: &str = "unavailable";

impl Failure {
    pub fn new(code: &str, message: impl Into<String>) -> Failure {
        Failure {
            code: code.to_owned(),
            message: message.into(),
        }
    }

    pub fn not_found() -> Failure {
        Failure::new(CODE_NOT_FOUND, "not found")
    }

    pub fn denied(what: &str) -> Failure {
        Failure::new(CODE_DENIED, format!("this plugin wasn't allowed {what}"))
    }
}

/// From the app to the host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum ToHost {
    /// Runs the plugin's entry script once; the host answers with
    /// [`FromHost::Started`].
    Start {
        plugin: PluginInfo,
        code: String,
        budgets: Budgets,
    },
    /// Runs a registered command.
    Invoke {
        id: u64,
        command: String,
        #[serde(default)]
        args: Value,
    },
    /// Draws a fenced block with the language's renderer.
    Render {
        id: u64,
        language: String,
        source: String,
        theme: Theme,
    },
    /// A button in the plugin's panel was pressed, with the panel's inputs.
    Press {
        id: u64,
        button: String,
        #[serde(default)]
        values: Map<String, Value>,
    },
    /// The answer to the host's [`FromHost::Host`] with this `id`.
    Reply {
        id: u64,
        ok: bool,
        /// The answer; `null` with a failure.
        #[serde(default)]
        value: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<Failure>,
    },
    /// Ends the process.
    Stop,
}

/// From the host to the app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum FromHost {
    /// The entry script ran: what it registered, or why it failed.
    Started {
        ok: bool,
        #[serde(default)]
        registered: Registered,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<Failure>,
    },
    /// A call to the app, on behalf of request `id`.
    Host {
        id: u64,
        call: String,
        #[serde(default)]
        args: Value,
    },
    /// Request `id` finished.
    Result {
        id: u64,
        ok: bool,
        /// The answer; `null` with a failure.
        #[serde(default)]
        value: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<Failure>,
    },
    /// `kanso.log(level, text)`: kept in the plugin's log, never answered.
    Log { level: String, text: String },
}

/// Why a line couldn't be read.
#[derive(Debug)]
pub enum FrameError {
    /// The line is longer than the cap; it was read no further.
    TooLong,
    /// The line isn't one message.
    Malformed(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::TooLong => f.write_str("the message is too long"),
            FrameError::Malformed(err) => write!(f, "the message isn't valid: {err}"),
            FrameError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<std::io::Error> for FrameError {
    fn from(err: std::io::Error) -> Self {
        FrameError::Io(err)
    }
}

/// Reads one line of at most `max` bytes (without its newline), or `None`
/// at the end of input. Blank lines are skipped. A longer line is
/// [`FrameError::TooLong`], and nothing past the cap is buffered.
pub fn read_line(reader: &mut impl BufRead, max: usize) -> Result<Option<Vec<u8>>, FrameError> {
    loop {
        let mut line = Vec::new();
        loop {
            let available = match reader.fill_buf() {
                Ok(buf) => buf,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err.into()),
            };
            if available.is_empty() {
                if line.is_empty() {
                    return Ok(None);
                }
                break;
            }
            match available.iter().position(|&b| b == b'\n') {
                Some(end) => {
                    if line.len() + end > max {
                        return Err(FrameError::TooLong);
                    }
                    line.extend_from_slice(&available[..end]);
                    reader.consume(end + 1);
                    break;
                }
                None => {
                    let n = available.len();
                    if line.len() + n > max {
                        return Err(FrameError::TooLong);
                    }
                    line.extend_from_slice(available);
                    reader.consume(n);
                }
            }
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        return Ok(Some(line));
    }
}

/// Reads one message of at most `max` bytes, or `None` at the end of input.
pub fn read_message<T: serde::de::DeserializeOwned>(
    reader: &mut impl BufRead,
    max: usize,
) -> Result<Option<T>, FrameError> {
    match read_line(reader, max)? {
        Some(line) => serde_json::from_slice(&line)
            .map(Some)
            .map_err(FrameError::Malformed),
        None => Ok(None),
    }
}

/// Writes `message` as one line and flushes. A message longer than `max`
/// is refused with [`FrameError::TooLong`] and nothing is written.
pub fn write_message<T: Serialize>(
    writer: &mut impl Write,
    message: &T,
    max: usize,
) -> Result<(), FrameError> {
    let line = serde_json::to_vec(message).map_err(FrameError::Malformed)?;
    if line.len() > max {
        return Err(FrameError::TooLong);
    }
    writer.write_all(&line)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn any_value() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(Value::from),
            "\\PC{0,24}".prop_map(Value::String),
        ];
        leaf.prop_recursive(3, 24, 4, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                prop::collection::btree_map("[a-z]{1,6}", inner, 0..4)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    fn any_from_host() -> impl Strategy<Value = FromHost> {
        prop_oneof![
            (any::<u64>(), "[a-z.]{1,20}", any_value())
                .prop_map(|(id, call, args)| FromHost::Host { id, call, args }),
            (any::<u64>(), any_value()).prop_map(|(id, value)| FromHost::Result {
                id,
                ok: true,
                value,
                error: None,
            }),
            ("[a-z]{1,8}", "\\PC{0,40}").prop_map(|(level, text)| FromHost::Log { level, text }),
        ]
    }

    proptest! {
        /// Invariant: every message comes back as itself, in order, however
        /// many are on the pipe, and the end of input is `None`.
        #[test]
        fn prop_messages_round_trip(messages in prop::collection::vec(any_from_host(), 0..8)) {
            let mut pipe = Vec::new();
            for message in &messages {
                write_message(&mut pipe, message, MAX_FROM_HOST).unwrap();
            }
            let mut reader = std::io::Cursor::new(pipe);
            for message in &messages {
                let back: FromHost = read_message(&mut reader, MAX_FROM_HOST).unwrap().unwrap();
                prop_assert_eq!(&back, message);
            }
            prop_assert!(read_message::<FromHost>(&mut reader, MAX_FROM_HOST).unwrap().is_none());
        }

        /// Invariant: a line is read whole when it fits the cap and refused
        /// when it doesn't, whatever the reader's buffer size.
        #[test]
        fn prop_the_cap_holds(len in 0usize..300, max in 1usize..200, chunk in 1usize..64) {
            let mut pipe = vec![b'x'; len];
            pipe.push(b'\n');
            let mut reader = std::io::BufReader::with_capacity(chunk, std::io::Cursor::new(pipe));
            match read_line(&mut reader, max) {
                Ok(Some(line)) => prop_assert!(len > 0 && len <= max && line.len() == len),
                Ok(None) => prop_assert_eq!(len, 0),
                Err(FrameError::TooLong) => prop_assert!(len > max),
                Err(other) => prop_assert!(false, "unexpected {other}"),
            }
        }

        /// Invariant: random bytes never panic the reader; each line is a
        /// message, an error, or the end.
        #[test]
        fn prop_garbage_is_an_error_not_a_panic(bytes in prop::collection::vec(any::<u8>(), 0..400)) {
            let mut reader = std::io::Cursor::new(bytes);
            for _ in 0..50 {
                match read_message::<ToHost>(&mut reader, 128) {
                    Ok(None) => break,
                    Ok(Some(_)) | Err(FrameError::Malformed(_)) | Err(FrameError::TooLong) => {}
                    Err(FrameError::Io(err)) => prop_assert!(false, "{err}"),
                }
                if matches!(read_line(&mut reader, 128), Ok(None)) {
                    break;
                }
            }
        }
    }

    #[test]
    fn the_design_example_parses() {
        let lines = concat!(
            r#"{"id":7,"op":"invoke","command":"count-words","args":{}}"#,
            "\n",
            r#"{"id":7,"op":"reply","ok":true,"value":[{"id":"note:abc","title":"Draft"}]}"#,
            "\n"
        );
        let mut reader = std::io::Cursor::new(lines);
        let invoke: ToHost = read_message(&mut reader, MAX_TO_HOST).unwrap().unwrap();
        assert!(
            matches!(invoke, ToHost::Invoke { id: 7, ref command, .. } if command == "count-words")
        );
        let reply: ToHost = read_message(&mut reader, MAX_TO_HOST).unwrap().unwrap();
        assert!(matches!(
            reply,
            ToHost::Reply {
                id: 7,
                ok: true,
                ..
            }
        ));
    }

    #[test]
    fn a_message_over_the_cap_is_not_written() {
        let mut pipe = Vec::new();
        let big = FromHost::Log {
            level: "info".into(),
            text: "x".repeat(64),
        };
        assert!(matches!(
            write_message(&mut pipe, &big, 32),
            Err(FrameError::TooLong)
        ));
        assert!(pipe.is_empty());
    }

    #[test]
    fn every_call_name_is_known() {
        assert!(is_call("notes.get"));
        assert!(!is_call("notes.delete"));
        assert!(!is_call("fs.read"));
    }
}
