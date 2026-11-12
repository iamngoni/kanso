//! Markdown and text files the operating system asks Kanso to open: Finder's
//! Open With and double-click, a file manager on Windows and Linux, and
//! `kanso file.md` in a terminal.
//!
//! They arrive three ways (`file://` URLs from macOS, arguments of the
//! process, a path forwarded by a second process, see [`crate::url_callback`])
//! and end the same: the paths that are existing note files are queued until
//! the main window exists, then imported the way File › Open… imports them
//! ([`crate::library_import::open_paths_in`]), and the last note is shown.

use std::path::{Path, PathBuf};

use gpui_kit::App;
use kanso_import::library::resolve::is_note_path;

use crate::workspace::main_workspace;

/// The file paths among `urls`. Anything that isn't a `file://` URL for a
/// path on this computer is left out; names are percent-decoded. Pure.
pub fn paths_from_urls(urls: &[String]) -> Vec<PathBuf> {
    urls.iter()
        .filter_map(|text| {
            let url = url::Url::parse(text).ok()?;
            if url.scheme() != "file" {
                return None;
            }
            url.to_file_path().ok()
        })
        .collect()
}

/// Whether `path` is an existing file named like a note (`.md`, `.markdown`,
/// `.txt`…).
pub fn is_note_file(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| is_note_path(&name.to_string_lossy()))
        && path.is_file()
}

/// The paths that are existing note files, in order.
pub fn note_files(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|path| is_note_file(path))
        .collect()
}

/// Paths waiting for the main window. Each is opened once, however many
/// times the system asks before then.
#[derive(Default, Debug)]
pub struct Queue {
    waiting: Vec<PathBuf>,
}

impl Queue {
    pub fn push(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            if !self.waiting.contains(&path) {
                self.waiting.push(path);
            }
        }
    }

    /// Everything waiting once the window is `ready` to show it; nothing
    /// before, and nothing is lost.
    pub fn take(&mut self, ready: bool) -> Vec<PathBuf> {
        if ready {
            std::mem::take(&mut self.waiting)
        } else {
            Vec::new()
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }
}

/// Where files to open are sent from anywhere: the platform's callback, the
/// command line, a forwarding process.
#[derive(Clone)]
pub struct Opener(async_channel::Sender<Vec<PathBuf>>);

pub fn channel() -> (Opener, async_channel::Receiver<Vec<PathBuf>>) {
    let (sender, receiver) = async_channel::unbounded();
    (Opener(sender), receiver)
}

impl Opener {
    /// Files the system opened, as URLs.
    pub fn urls(&self, urls: &[String]) {
        self.paths(paths_from_urls(urls));
    }

    /// Queues the existing note files among `paths`; the rest are ignored.
    pub fn paths(&self, paths: Vec<PathBuf>) {
        let paths = note_files(paths);
        if !paths.is_empty() {
            self.0.try_send(paths).ok();
        }
    }
}

/// Opens what arrives on `receiver` in the main window, opening it with
/// `open_main_window` first if it was closed, and brings it forward. Call
/// after the first window is open; what arrived at launch waits in the
/// channel.
pub fn serve(
    receiver: async_channel::Receiver<Vec<PathBuf>>,
    open_main_window: fn(&mut App),
    cx: &mut App,
) {
    cx.spawn(async move |cx| {
        let mut queue = Queue::default();
        while let Ok(paths) = receiver.recv().await {
            queue.push(paths);
            while let Ok(more) = receiver.try_recv() {
                queue.push(more);
            }
            cx.update(|cx| show_queued(&mut queue, open_main_window, cx));
        }
    })
    .detach();
}

fn show_queued(queue: &mut Queue, open_main_window: fn(&mut App), cx: &mut App) {
    let mut found = main_workspace(cx);
    if found.is_none() {
        open_main_window(cx);
        found = main_workspace(cx);
    }
    let paths = queue.take(found.is_some());
    let Some((window, workspace)) = found else {
        return;
    };
    window
        .update(cx, |_, window, cx| {
            window.activate_window();
            workspace.update(cx, |workspace, cx| {
                workspace.open_documents(paths, window, cx)
            });
        })
        .ok();
    cx.activate(true);
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn temp_path(parts: &[&str]) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.extend(parts);
        path
    }

    #[test]
    fn urls_become_paths_and_other_urls_are_ignored() {
        let path = temp_path(&["my notes", "caf\u{e9} \u{65e5}\u{672c} #1.md"]);
        let url = url::Url::from_file_path(&path).unwrap().to_string();
        assert!(url.contains("%20"));
        let urls = vec![
            "https://example.com/a.md".to_owned(),
            url,
            "kanso://auth/apple".to_owned(),
            "not a url".to_owned(),
        ];
        assert_eq!(paths_from_urls(&urls), vec![path]);
    }

    #[test]
    fn only_existing_note_files_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, "# hi").unwrap();
            path
        };
        let md = write("a.md");
        let txt = write("b.txt");
        let upper = write("C.MARKDOWN");
        let png = write("d.png");
        let none = write("README");
        let folder = dir.path().join("e.md");
        std::fs::create_dir(&folder).unwrap();
        let missing = dir.path().join("missing.md");
        let kept = note_files([
            md.clone(),
            png,
            missing,
            folder,
            txt.clone(),
            none,
            upper.clone(),
        ]);
        assert_eq!(kept, vec![md, txt, upper]);
    }

    /// Files that arrive at launch, before the window exists, wait; they are
    /// opened once the window does, each once, in order.
    #[test]
    fn files_queued_before_the_window_open_once_it_does() {
        let mut queue = Queue::default();
        let a = PathBuf::from("/notes/a.md");
        let b = PathBuf::from("/notes/b.md");
        queue.push([a.clone()]);
        assert!(queue.take(false).is_empty(), "no window yet");
        queue.push([b.clone(), a.clone()]);
        assert!(queue.take(false).is_empty());
        assert_eq!(queue.take(true), vec![a.clone(), b]);
        assert!(queue.take(true).is_empty(), "not opened twice");
        assert!(queue.is_empty());
        // Asked again later: opened again.
        queue.push([a.clone()]);
        assert_eq!(queue.take(true), vec![a]);
    }

    /// What reaches the channel at launch is still there when the task that
    /// serves it starts.
    #[test]
    fn the_opener_keeps_launch_files_until_served() {
        let dir = tempfile::tempdir().unwrap();
        let note = dir.path().join("launch.md");
        std::fs::write(&note, "x").unwrap();
        let (opener, receiver) = channel();
        opener.urls(&[
            url::Url::from_file_path(&note).unwrap().to_string(),
            "https://example.com/x.md".to_owned(),
        ]);
        opener.paths(vec![dir.path().join("missing.md")]);
        assert_eq!(receiver.try_recv().unwrap(), vec![note]);
        assert!(receiver.try_recv().is_err());
    }

    proptest! {
        // `.` and `..` are not file names: a URL resolves them away, so a folder
        // called `..` was never a path that could round-trip.
        #[test]
        fn prop_file_url_round_trips_to_the_same_path(
            folder in "[a-zA-Z0-9 _%#?&+=;,.~\u{e9}\u{fc}\u{65e5}\u{672c}\u{1f600}-]{1,16}".prop_filter("not a dot segment", |s| s != "." && s != ".."),
            name in "[a-zA-Z0-9 _%#?&+=;,.~\u{e9}\u{fc}\u{65e5}\u{672c}\u{1f600}-]{1,16}",
        ) {
            let path = temp_path(&[&folder, &format!("{name}.md")]);
            let url = url::Url::from_file_path(&path).unwrap().to_string();
            prop_assert_eq!(paths_from_urls(&[url]), vec![path]);
        }
    }

    proptest! {
        #[test]
        fn prop_urls_that_are_not_files_are_ignored(
            scheme in "(https?|ftp|kanso|mailto|data|x-[a-z]{1,6})",
            rest in "[a-zA-Z0-9/._%-]{0,24}",
        ) {
            let hier = format!("{}://{}", scheme, rest);
            prop_assert!(paths_from_urls(&[hier]).is_empty());
            let opaque = format!("{}:{}", scheme, rest);
            prop_assert!(paths_from_urls(&[opaque]).is_empty());
        }
    }

    proptest! {
        #[test]
        fn prop_files_are_kept_only_with_a_note_extension(
            stem in "[a-z0-9]{1,8}",
            ext in "(md|MD|markdown|txt|TXT|png|pdf|rs|)",
            exists in any::<bool>(),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let name = if ext.is_empty() { stem } else { format!("{stem}.{ext}") };
            let path = dir.path().join(&name);
            if exists {
                std::fs::write(&path, "x").unwrap();
            }
            prop_assert_eq!(
                note_files([path.clone()]) == vec![path],
                exists && is_note_path(&name)
            );
        }
    }

    proptest! {
        /// Whatever arrives, before or after the window, each path is
        /// opened and none is lost; before the window nothing is opened.
        #[test]
        fn prop_a_queue_opens_each_path_once(
            batches in proptest::collection::vec(
                proptest::collection::vec(0u8..6, 0..5), 0..6),
            ready_after in 0usize..8,
        ) {
            let path = |n: &u8| PathBuf::from(format!("/n/{n}.md"));
            let mut queue = Queue::default();
            let mut opened = Vec::new();
            for (index, batch) in batches.iter().enumerate() {
                queue.push(batch.iter().map(path));
                let taken = queue.take(index >= ready_after);
                if index < ready_after {
                    prop_assert!(taken.is_empty());
                }
                opened.extend(taken);
            }
            opened.extend(queue.take(true));
            let wanted: std::collections::HashSet<PathBuf> =
                batches.iter().flatten().map(path).collect();
            let seen: std::collections::HashSet<&PathBuf> = opened.iter().collect();
            prop_assert!(wanted.iter().all(|p| seen.contains(p)));
            prop_assert_eq!(seen.len(), wanted.len());
            if ready_after >= batches.len() {
                prop_assert_eq!(opened.len(), wanted.len(), "exactly once each");
            }
        }
    }

    /// The macOS bundle declares what Finder's Open With needs.
    #[test]
    fn the_bundle_script_declares_markdown_and_text_documents() {
        let script = include_str!("../../../scripts/bundle-macos.sh");
        for needed in [
            "<key>CFBundleDocumentTypes</key>",
            "<key>UTImportedTypeDeclarations</key>",
            "<string>net.daringfireball.markdown</string>",
            "<string>public.markdown</string>",
            "<string>public.plain-text</string>",
            "<string>Editor</string>",
            "<string>Alternate</string>",
            "<string>mkdn</string>",
        ] {
            assert!(script.contains(needed), "{needed}");
        }
        assert!(!script.contains("<string>Owner</string>"));
    }

    #[test]
    fn linux_and_windows_register_the_files() {
        let desktop = include_str!("../../../assets/linux/kanso.desktop");
        assert!(desktop.contains("text/markdown;text/plain;"));
        assert!(desktop.lines().any(|line| line == "Exec=kanso %U"));
        let installer = include_str!("../../../packaging/windows/kanso.iss");
        assert!(installer.contains(r"Software\Classes\Applications\kanso.exe\shell\open\command"));
        assert!(installer.contains(".markdown"));
    }

    /// What `kanso a.md b.txt` (or a file manager's `file://` URLs) means:
    /// open these, and nothing else. Anything else on the line is not it.
    #[test]
    fn existing_note_files_as_arguments_are_files_to_open() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.md");
        let b = dir.path().join("b b.txt");
        let pdf = dir.path().join("c.pdf");
        for path in [&a, &b, &pdf] {
            std::fs::write(path, "x").unwrap();
        }
        let text = |path: &Path| path.to_str().unwrap().to_owned();
        let parsed = crate::parse_args_from([text(&a), text(&b)]).unwrap();
        assert_eq!(parsed.files, vec![a.clone(), b.clone()]);
        assert!(parsed.command.is_none() && parsed.rest.is_empty());
        // A file manager's URLs.
        let url = url::Url::from_file_path(&b).unwrap().to_string();
        let parsed = crate::parse_args_from([url]).unwrap();
        assert_eq!(parsed.files, vec![b.clone()]);
        // Not every argument a note file: no files, the usual rules.
        for args in [
            vec![text(&a), text(&pdf)],
            vec![text(&a), "--after-update".to_owned()],
            vec![text(&dir.path().join("missing.md"))],
            vec!["serve".to_owned(), text(&a)],
        ] {
            let parsed = crate::parse_args_from(args.clone()).unwrap();
            assert!(parsed.files.is_empty(), "{args:?}");
        }
        assert!(crate::parse_args_from(Vec::new()).unwrap().files.is_empty());
    }

    /// A relative path is made absolute, so it means the same to the Kanso
    /// it is forwarded to.
    #[test]
    fn relative_arguments_become_absolute() {
        let name = "relative-argument-test.md";
        let path = std::env::current_dir().unwrap().join(name);
        std::fs::write(&path, "x").unwrap();
        let parsed = crate::parse_args_from([name.to_owned()]).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(parsed.files, vec![path]);
    }

    proptest! {
        #[test]
        fn prop_arguments_are_files_only_when_all_are_note_files(
            names in proptest::collection::vec(
                ("[a-z0-9]{1,6}", "(md|txt|markdown|png|)", any::<bool>()), 0..5),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let mut args = Vec::new();
            let mut all_notes = !names.is_empty();
            for (index, (stem, ext, exists)) in names.iter().enumerate() {
                let name = if ext.is_empty() {
                    format!("{index}{stem}")
                } else {
                    format!("{index}{stem}.{ext}")
                };
                let path = dir.path().join(&name);
                if *exists {
                    std::fs::write(&path, "x").unwrap();
                }
                all_notes &= *exists && is_note_path(&name);
                args.push(path.to_str().unwrap().to_owned());
            }
            let parsed = crate::parse_args_from(args.clone());
            if all_notes {
                let files = parsed.unwrap().files;
                prop_assert_eq!(files.len(), args.len());
                prop_assert!(files.iter().all(|f| f.is_absolute()));
            } else if let Ok(parsed) = parsed {
                prop_assert!(parsed.files.is_empty());
            }
        }
    }
}
