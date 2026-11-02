//! The words and choices behind Settings › Plugins that need no window:
//! the status line, a grant in plain sentences, the notebooks a person
//! picks, and a log line for the network and the plugin's own output.

use std::collections::{BTreeSet, HashSet};

use kanso_plugins::model::{Grant, Manifest};
use kanso_plugins::net::Egress;

use crate::settings::StatusLine;
use crate::sidebar::Tone;

/// How many network requests the page lists for one plugin.
pub const EGRESS_SHOWN: usize = 20;
/// How many log lines the page lists for one plugin.
pub const LOG_SHOWN: usize = 40;

/// The page's status: safe mode, or how many plugins are installed.
pub fn status(installed: usize, safe_mode: bool) -> StatusLine {
    if safe_mode {
        return StatusLine {
            tone: Tone::Warning,
            label: "Safe mode: every plugin is off for this session".to_owned(),
            detail: "Switch one on to use it until you quit Kanso. Nothing is removed.".to_owned(),
        };
    }
    let (tone, label) = match installed {
        0 => (Tone::Quiet, "No plugins installed".to_owned()),
        1 => (Tone::Good, "1 plugin installed".to_owned()),
        n => (Tone::Good, format!("{n} plugins installed")),
    };
    StatusLine {
        tone,
        label,
        detail: "A plugin only does what you allow below, and only for the notebooks you choose."
            .to_owned(),
    }
}

/// "12 B", "1.5 KB", "3.0 MB".
pub fn bytes_text(n: u64) -> String {
    const KB: f64 = 1024.0;
    let n_f = n as f64;
    if n < 1024 {
        format!("{n} B")
    } else if n_f < KB * KB {
        format!("{:.1} KB", n_f / KB)
    } else {
        format!("{:.1} MB", n_f / (KB * KB))
    }
}

/// One network request in a line: what was asked, how it went, how much
/// moved, and when (`when` is the time in the person's words).
pub fn egress_line(entry: &Egress, when: &str) -> String {
    let asked = format!("{} {}{}", entry.method, entry.host, entry.url_path);
    let outcome = match (&entry.refused, entry.status) {
        (Some(why), _) => format!("refused: {why}"),
        (None, Some(status)) => status.to_string(),
        (None, None) => "no answer".to_owned(),
    };
    format!(
        "{asked} · {outcome} · {} sent, {} received · {when}",
        bytes_text(entry.sent),
        bytes_text(entry.received)
    )
}

/// The newest requests first, at most [`EGRESS_SHOWN`]. The input may
/// come in either order.
pub fn recent_egress(mut entries: Vec<Egress>) -> Vec<Egress> {
    entries.sort_by_key(|e| std::cmp::Reverse(e.at_ms));
    entries.truncate(EGRESS_SHOWN);
    entries
}

/// The last [`LOG_SHOWN`] lines of a plugin's log, as `level: text`.
pub fn log_tail(lines: &[(String, String)]) -> Vec<String> {
    let start = lines.len().saturating_sub(LOG_SHOWN);
    lines[start..]
        .iter()
        .map(|(level, text)| format!("{level}: {text}"))
        .collect()
}

/// What a grant lets a plugin do, one plain line each, in a fixed order.
/// `name_of` gives a notebook's name from its ID, or `None` once it is
/// gone.
pub fn grant_lines(grant: &Grant, name_of: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    use kanso_plugins::model::EditorPermission as E;
    let mut lines = Vec::new();
    if grant.commands {
        lines.push("Add commands to the palette".to_owned());
    }
    for editor in &grant.editor {
        lines.push(
            match editor {
                E::NoteRead => "Read the note you have open",
                E::SelectionRead => "Read the text you select",
                E::SelectionReplace => "Change the text you select",
            }
            .to_owned(),
        );
    }
    let notes = &grant.notes;
    if notes.read_all {
        lines.push("Read every notebook that isn't locked".to_owned());
    } else if !notes.read_books.is_empty() {
        lines.push(format!(
            "Read notes in {}",
            book_names(&notes.read_books, name_of)
        ));
    }
    if !notes.read_tags.is_empty() {
        let tags: Vec<&str> = notes.read_tags.iter().map(String::as_str).collect();
        lines.push(format!("Read notes tagged {}", tags.join(", ")));
    }
    if !notes.write_books.is_empty() {
        lines.push(format!(
            "Change notes in {}",
            book_names(&notes.write_books, name_of)
        ));
    }
    if notes.create {
        lines.push("Make new notes".to_owned());
    }
    for fence in &grant.fences {
        lines.push(format!("Draw {fence} blocks in your notes"));
    }
    if grant.panel {
        lines.push("Show a panel in the sidebar".to_owned());
    }
    for secret in &grant.secrets {
        lines.push(format!("Use the token you save for it as {secret}"));
    }
    if grant
        .clipboard
        .contains(&kanso_plugins::model::ClipboardPermission::HistoryRead)
    {
        lines.push(
            "Read the text you copy in any app while Kanso is open; Kanso keeps the last 15 in memory".to_owned(),
        );
    }
    if grant
        .clipboard
        .contains(&kanso_plugins::model::ClipboardPermission::Write)
    {
        lines.push("Put text on the clipboard".to_owned());
    }
    for host in &grant.network {
        lines.push(format!("Connect to {host}"));
    }
    if lines.is_empty() {
        lines.push("Nothing: it has no permissions".to_owned());
    }
    lines
}

fn book_names(ids: &BTreeSet<String>, name_of: &dyn Fn(&str) -> Option<String>) -> String {
    let mut names: Vec<String> = ids
        .iter()
        .map(|id| name_of(id).unwrap_or_else(|| "a notebook that's gone".to_owned()))
        .collect();
    names.sort_by_key(|n| n.to_lowercase());
    names.join(", ")
}

/// What the notebook picker saves into a grant: the notebooks to read and
/// to change, or every notebook to read. Locked notebooks are dropped,
/// changing needs reading, and a request the manifest didn't make is
/// ignored. Tags, fences, tokens and the rest stay as they were. Pure.
pub fn with_books(
    grant: &Grant,
    manifest: &Manifest,
    read: &BTreeSet<String>,
    write: &BTreeSet<String>,
    read_all: bool,
    locked: &HashSet<String>,
) -> Grant {
    let mut next = grant.clone();
    next.notes.read_all = read_all;
    next.notes.read_books = read
        .iter()
        .filter(|id| !locked.contains(*id))
        .cloned()
        .collect();
    next.notes.write_books = write
        .iter()
        .filter(|id| !locked.contains(*id))
        .cloned()
        .collect();
    if !read_all {
        let readable = next.notes.read_books.clone();
        next.notes.write_books.retain(|id| readable.contains(id));
    }
    kanso_plugins::grant::narrow_to(&next, manifest)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use kanso_plugins::grant::check;
    use kanso_plugins::model::{
        Grant, Manifest, NotesGrant, NotesRequest, Permissions, Publisher, ReadRequest,
    };
    use kanso_plugins::net::Egress;
    use proptest::prelude::*;

    use super::{
        EGRESS_SHOWN, LOG_SHOWN, bytes_text, egress_line, grant_lines, log_tail, recent_egress,
        status, with_books,
    };

    fn manifest(read: bool, write: bool) -> Manifest {
        Manifest {
            schema: 1,
            api_version: 1,
            id: "com.example.demo".into(),
            name: "Demo".into(),
            version: "1.0.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "Example".into(),
                key: format!("ed25519:{}", "0".repeat(64)),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions: Permissions {
                notes: NotesRequest {
                    read: read.then(ReadRequest::default),
                    write: if write { vec!["Work".into()] } else { vec![] },
                    create: false,
                },
                ..Permissions::default()
            },
        }
    }

    fn empty_grant() -> Grant {
        Grant {
            commands: false,
            editor: BTreeSet::new(),
            notes: NotesGrant::default(),
            fences: BTreeSet::new(),
            panel: false,
            secrets: BTreeSet::new(),
            network: BTreeSet::new(),
            clipboard: Default::default(),
            ask_before_changing: true,
        }
    }

    fn egress(at_ms: i64) -> Egress {
        Egress {
            at_ms,
            method: "GET".into(),
            host: "api.example.com".into(),
            url_path: "/v1/items".into(),
            status: Some(200),
            sent: 0,
            received: 1536,
            refused: None,
        }
    }

    #[test]
    fn the_status_names_safe_mode_or_the_count() {
        assert!(
            status(3, true)
                .label
                .starts_with("Safe mode: every plugin is off")
        );
        assert_eq!(status(0, false).label, "No plugins installed");
        assert_eq!(status(1, false).label, "1 plugin installed");
        assert_eq!(status(2, false).label, "2 plugins installed");
    }

    #[test]
    fn bytes_are_short() {
        assert_eq!(bytes_text(0), "0 B");
        assert_eq!(bytes_text(1536), "1.5 KB");
        assert_eq!(bytes_text(3 * 1024 * 1024), "3.0 MB");
    }

    #[test]
    fn a_request_reads_in_one_line() {
        assert_eq!(
            egress_line(&egress(1), "2 min ago"),
            "GET api.example.com/v1/items · 200 · 0 B sent, 1.5 KB received · 2 min ago"
        );
        let mut refused = egress(1);
        refused.refused = Some("host not allowed".into());
        refused.status = None;
        assert!(egress_line(&refused, "now").contains("refused: host not allowed"));
    }

    #[test]
    fn a_grant_reads_in_plain_sentences() {
        let mut grant = empty_grant();
        assert_eq!(
            grant_lines(&grant, &|_| None),
            ["Nothing: it has no permissions"]
        );
        grant.commands = true;
        grant.notes.read_books = ["book:w".to_owned()].into();
        grant.notes.write_books = ["book:gone".to_owned()].into();
        grant.network = ["api.example.com".to_owned()].into();
        let lines = grant_lines(&grant, &|id| (id == "book:w").then(|| "Work".to_owned()));
        assert_eq!(
            lines,
            [
                "Add commands to the palette",
                "Read notes in Work",
                "Change notes in a notebook that's gone",
                "Connect to api.example.com",
            ]
        );
    }

    #[test]
    fn the_picker_never_widens_past_the_manifest_or_into_locked_books() {
        let read: BTreeSet<String> = ["book:a".to_owned(), "book:lock".to_owned()].into();
        let write: BTreeSet<String> = ["book:a".to_owned(), "book:other".to_owned()].into();
        let locked: HashSet<String> = ["book:lock".to_owned()].into();
        let granted = with_books(
            &empty_grant(),
            &manifest(true, true),
            &read,
            &write,
            false,
            &locked,
        );
        assert_eq!(granted.notes.read_books, ["book:a".to_owned()].into());
        assert_eq!(granted.notes.write_books, ["book:a".to_owned()].into());
        let none = with_books(
            &empty_grant(),
            &manifest(false, false),
            &read,
            &write,
            true,
            &locked,
        );
        assert!(!none.notes.reads() && !none.notes.writes());
    }

    #[test]
    fn newest_requests_come_first_and_are_capped() {
        let entries: Vec<Egress> = (0..30).map(egress).collect();
        let shown = recent_egress(entries);
        assert_eq!(shown.len(), EGRESS_SHOWN);
        assert_eq!(shown[0].at_ms, 29);
    }

    #[test]
    fn the_log_tail_keeps_the_last_lines() {
        let lines: Vec<(String, String)> = (0..100)
            .map(|i| ("info".to_owned(), format!("line {i}")))
            .collect();
        let tail = log_tail(&lines);
        assert_eq!(tail.len(), LOG_SHOWN);
        assert_eq!(tail.last().unwrap(), "info: line 99");
    }

    proptest! {
        /// Invariant: whatever is picked, the saved grant passes the same
        /// check the host runs: within the manifest, no locked notebook,
        /// and no change without reading.
        #[test]
        fn prop_a_picked_grant_always_passes_the_check(
            read in prop::collection::btree_set("book:[a-d]", 0..4),
            write in prop::collection::btree_set("book:[a-d]", 0..4),
            lock in prop::collection::hash_set("book:[a-d]", 0..3),
            read_all in any::<bool>(),
            asks_read in any::<bool>(),
            asks_write in any::<bool>(),
        ) {
            let m = manifest(asks_read, asks_write);
            let grant = with_books(&empty_grant(), &m, &read, &write, read_all, &lock);
            prop_assert_eq!(check(&grant, &m, &lock), Ok(()));
        }

        /// Invariant: the request list is never longer than the cap and is
        /// sorted newest first.
        #[test]
        fn prop_requests_are_newest_first(times in prop::collection::vec(0i64..1_000, 0..60)) {
            let shown = recent_egress(times.iter().map(|t| egress(*t)).collect());
            prop_assert!(shown.len() <= EGRESS_SHOWN);
            prop_assert!(shown.windows(2).all(|w| w[0].at_ms >= w[1].at_ms));
        }

        /// Invariant: the log tail has at most the cap and ends with the
        /// last line.
        #[test]
        fn prop_the_log_tail_ends_at_the_last_line(n in 1usize..120) {
            let lines: Vec<(String, String)> =
                (0..n).map(|i| ("info".to_owned(), i.to_string())).collect();
            let tail = log_tail(&lines);
            prop_assert!(tail.len() <= LOG_SHOWN);
            prop_assert_eq!(tail.last().cloned(), Some(format!("info: {}", n - 1)));
        }

        /// Invariant: byte text always ends in a unit.
        #[test]
        fn prop_bytes_have_a_unit(n in any::<u64>()) {
            let text = bytes_text(n);
            prop_assert!(text.ends_with(" B") || text.ends_with(" KB") || text.ends_with(" MB"));
        }
    }
}
