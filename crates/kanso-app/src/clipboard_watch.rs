//! Watching the clipboard for clipboard history (`clipboard_history.rs`),
//! on a thread of its own, while a plugin holding `clipboard.history.read`
//! is on.
//!
//! - X11 (and Kanso run under XWayland): XFixes says when the clipboard
//!   changes hands, and the new owner is asked for its text. A copy that
//!   carries KDE's password hint (`x-kde-passwordManagerHint: secret`, as
//!   KeePassXC sets it) is marked sensitive.
//! - macOS: the pasteboard's change count, every half second. A copy marked
//!   concealed, transient or generated (nspasteboard.org) is sensitive.
//! - Windows: the clipboard's sequence number, every half second. A copy
//!   kept from monitors or the clipboard history (as password managers do)
//!   is sensitive.
//! - Wayland lets only the focused window read the clipboard: nothing is
//!   watched there; it needs Kanso run under XWayland.
//!
//! What was copied before turning it on isn't read, except what is on the
//! clipboard at that moment.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::time::Duration;

use gpui_kit::App;

/// One copy seen. `sensitive`: the app it came from marked it secret, so
/// it isn't kept; such copies come without their text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copied {
    pub text: String,
    pub sensitive: bool,
}

/// Whether this session lets Kanso watch the clipboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Support {
    Watching,
    /// Wayland: only with Kanso run under XWayland.
    NeedsXWayland,
    Unsupported,
}

/// Whether the clipboard can be watched here.
pub fn support(cx: &App) -> Support {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        Support::Watching
    } else if cfg!(target_os = "linux") {
        if cx.compositor_name() == "X11" {
            Support::Watching
        } else {
            Support::NeedsXWayland
        }
    } else {
        Support::Unsupported
    }
}

/// The watching thread; dropped, it stops.
pub struct Watcher {
    stop: Arc<AtomicBool>,
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// A watcher with no thread, for tests that hand copies in themselves.
#[cfg(test)]
pub fn idle_for_test() -> Watcher {
    Watcher {
        stop: Arc::new(AtomicBool::new(false)),
    }
}

/// Starts watching, sending each copy to `copies`. `None` when the thread
/// couldn't start.
pub fn start(copies: async_channel::Sender<Copied>) -> Option<Watcher> {
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    std::thread::Builder::new()
        .name("kanso-clipboard".into())
        .spawn(move || {
            if let Err(err) = watch(&stopped, &copies) {
                eprintln!("kanso: stopped watching the clipboard: {err:#}");
            }
        })
        .map_err(|err| eprintln!("kanso: couldn't watch the clipboard: {err}"))
        .ok()?;
    Some(Watcher { stop })
}

/// Whether KDE's password hint says the copy is a secret. Pure.
pub fn is_secret_hint(hint: &[u8]) -> bool {
    String::from_utf8_lossy(hint).trim() == "secret"
}

/// Hashes of the last copies marked secret. An app that copied a secret
/// can hand the clipboard on unmarked: on GNOME, Mutter keeps a copy alive
/// once its app quits, without the password hint. The same text again is
/// as secret. Only the hashes are kept.
#[derive(Default)]
struct Secrets(std::collections::VecDeque<u64>);

impl Secrets {
    const KEPT: usize = 8;

    fn remember(&mut self, text: &str) {
        self.0.push_front(crate::clipboard::fingerprint(text));
        self.0.truncate(Self::KEPT);
    }

    fn contains(&self, text: &str) -> bool {
        self.0.contains(&crate::clipboard::fingerprint(text))
    }
}

/// What a copy read (whether marked secret, and its text) is handed on as:
/// a secret's text never is. `None` when there is nothing to hand on.
fn screened(read: (bool, Option<String>), secrets: &mut Secrets) -> Option<Copied> {
    let secret = Copied {
        text: String::new(),
        sensitive: true,
    };
    match read {
        (true, text) => {
            if let Some(text) = text {
                secrets.remember(&text);
            }
            Some(secret)
        }
        (false, Some(text)) if secrets.contains(&text) => Some(secret),
        (false, Some(text)) => Some(Copied {
            text,
            sensitive: false,
        }),
        (false, None) => None,
    }
}

/// Hands on a copy read; false once nobody listens.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn send(
    copies: &async_channel::Sender<Copied>,
    read: (bool, Option<String>),
    secrets: &mut Secrets,
) -> bool {
    match screened(read, secrets) {
        Some(copied) => copies.send_blocking(copied).is_ok(),
        None => true,
    }
}

/// The polling of macOS and Windows: reads the clipboard whenever its count
/// changes. A clipboard another app holds open is read on the next turn.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn watch(stop: &AtomicBool, copies: &async_channel::Sender<Copied>) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    use crate::clipboard::mac::{change_count as count, read_copy};
    #[cfg(target_os = "windows")]
    use crate::clipboard::win::{read_copy, sequence};
    #[cfg(target_os = "windows")]
    let count = || Some(sequence());

    const POLL: Duration = Duration::from_millis(500);
    // None: what is on the clipboard now is read first.
    let mut seen = None;
    let mut secrets = Secrets::default();
    while !stop.load(Ordering::Relaxed) {
        let now = count();
        if now.is_some() && now != seen {
            if let Some(read) = read_copy() {
                seen = now;
                if !send(copies, read, &mut secrets) {
                    break;
                }
            }
        }
        std::thread::sleep(POLL);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn watch(stop: &AtomicBool, copies: &async_channel::Sender<Copied>) -> anyhow::Result<()> {
    x11::watch(stop, copies)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn watch(_: &AtomicBool, _: &async_channel::Sender<Copied>) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
mod x11 {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use anyhow::{Context as _, Result};
    use x11rb::connection::Connection as _;
    use x11rb::protocol::Event;
    use x11rb::protocol::xfixes::{ConnectionExt as _, SelectionEventMask};
    use x11rb::protocol::xproto::{
        Atom, AtomEnum, ConnectionExt as _, CreateWindowAux, Window, WindowClass,
    };
    use x11rb::rust_connection::RustConnection;

    use super::{Copied, Secrets, is_secret_hint, send};
    use crate::clipboard_history::MAX_CLIP;

    /// How long an owner gets to answer before its copy is skipped.
    const ANSWER: Duration = Duration::from_secs(1);
    /// How often the stop flag is looked at while nothing happens.
    const IDLE: Duration = Duration::from_millis(100);

    struct Atoms {
        clipboard: Atom,
        targets: Atom,
        utf8: Atom,
        hint: Atom,
        incr: Atom,
        /// Where owners put what they're asked for, on Kanso's window.
        into: Atom,
    }

    pub fn watch(stop: &AtomicBool, copies: &async_channel::Sender<Copied>) -> Result<()> {
        let (x11, screen) = RustConnection::connect(None).context("connecting to X11")?;
        let root = x11.setup().roots[screen].root;
        let window = x11.generate_id()?;
        x11.create_window(
            0,
            window,
            root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_ONLY,
            0,
            &CreateWindowAux::new(),
        )?;
        let intern =
            |name: &[u8]| -> Result<Atom> { Ok(x11.intern_atom(false, name)?.reply()?.atom) };
        let atoms = Atoms {
            clipboard: intern(b"CLIPBOARD")?,
            targets: intern(b"TARGETS")?,
            utf8: intern(b"UTF8_STRING")?,
            hint: intern(b"x-kde-passwordManagerHint")?,
            incr: intern(b"INCR")?,
            into: intern(b"KANSO_CLIPBOARD")?,
        };
        x11.xfixes_query_version(5, 0)?
            .reply()
            .context("X11 has no XFixes")?;
        x11.xfixes_select_selection_input(
            window,
            atoms.clipboard,
            SelectionEventMask::SET_SELECTION_OWNER,
        )?;
        x11.flush()?;
        // What is on the clipboard now, then each change.
        let mut changed = true;
        let mut secrets = Secrets::default();
        while !stop.load(Ordering::Relaxed) {
            if changed {
                changed = false;
                let owner = x11.get_selection_owner(atoms.clipboard)?.reply()?.owner;
                if owner != x11rb::NONE {
                    let read = read(&x11, window, &atoms, &mut changed)?;
                    if read.is_some_and(|read| !send(copies, read, &mut secrets)) {
                        break;
                    }
                }
                continue;
            }
            match x11.poll_for_event()? {
                Some(Event::XfixesSelectionNotify(_)) => changed = true,
                Some(_) => {}
                None => std::thread::sleep(IDLE),
            }
        }
        x11.destroy_window(window)?;
        x11.flush()?;
        Ok(())
    }

    /// Asks the owner for the copy: its targets, the password hint if it
    /// has one, else its text. `None` when it didn't answer, or the copy is
    /// too big. A change of hands meanwhile sets `changed`.
    fn read(
        x11: &RustConnection,
        window: Window,
        atoms: &Atoms,
        changed: &mut bool,
    ) -> Result<Option<(bool, Option<String>)>> {
        let Some(targets) = convert(x11, window, atoms, atoms.targets, changed)? else {
            return Ok(None);
        };
        let targets: Vec<Atom> = targets
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_ne_bytes(*b))
            .collect();
        // A secret's text is read too, only for its hash (`Secrets`).
        let secret = targets.contains(&atoms.hint)
            && convert(x11, window, atoms, atoms.hint, changed)?
                .as_deref()
                .is_some_and(is_secret_hint);
        let target = if targets.contains(&atoms.utf8) {
            atoms.utf8
        } else if targets.contains(&AtomEnum::STRING.into()) {
            AtomEnum::STRING.into()
        } else {
            // Not text: an image or files.
            return Ok(Some((secret, None)));
        };
        let text = convert(x11, window, atoms, target, changed)?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        Ok(Some((secret, text)))
    }

    /// Asks the clipboard's owner for `target` and waits for its answer.
    fn convert(
        x11: &RustConnection,
        window: Window,
        atoms: &Atoms,
        target: Atom,
        changed: &mut bool,
    ) -> Result<Option<Vec<u8>>> {
        x11.convert_selection(
            window,
            atoms.clipboard,
            target,
            atoms.into,
            x11rb::CURRENT_TIME,
        )?;
        x11.flush()?;
        let deadline = Instant::now() + ANSWER;
        let property = loop {
            match x11.poll_for_event()? {
                Some(Event::SelectionNotify(event)) if event.requestor == window => {
                    break event.property;
                }
                Some(Event::XfixesSelectionNotify(_)) => *changed = true,
                Some(_) => {}
                None if Instant::now() >= deadline => return Ok(None),
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        };
        if property == x11rb::NONE {
            return Ok(None);
        }
        // A little over the longest copy kept, in 32-bit units.
        let longest = (MAX_CLIP as u32 + 1024) / 4;
        let reply = x11
            .get_property(true, window, atoms.into, AtomEnum::ANY, 0, longest)?
            .reply()?;
        if reply.type_ == atoms.incr || reply.bytes_after > 0 {
            // Too big to keep; what is left is deleted.
            x11.delete_property(window, atoms.into)?;
            x11.flush()?;
            return Ok(None);
        }
        Ok(Some(reply.value))
    }
}

#[cfg(test)]
mod tests {
    use super::{Copied, Secrets, is_secret_hint, screened};

    /// A secret handed on unmarked, as Mutter does once its app quits, is
    /// still secret; other copies aren't.
    #[test]
    fn a_secret_stays_secret_once_unmarked() {
        let mut secrets = Secrets::default();
        let secret = Some(Copied {
            text: String::new(),
            sensitive: true,
        });
        let read = |sensitive: bool, text: &str| (sensitive, Some(text.to_owned()));
        assert_eq!(screened(read(true, "hunter2"), &mut secrets), secret);
        assert_eq!(screened(read(false, "hunter2"), &mut secrets), secret);
        assert_eq!(
            screened(read(false, "docker ps"), &mut secrets),
            Some(Copied {
                text: "docker ps".into(),
                sensitive: false,
            })
        );
        assert_eq!(screened((false, None), &mut secrets), None);
        assert_eq!(screened((true, None), &mut secrets), secret);
    }

    #[test]
    fn only_the_secret_hint_is_secret() {
        assert!(is_secret_hint(b"secret"));
        assert!(is_secret_hint(b"secret\n"));
        assert!(!is_secret_hint(b"public"));
        assert!(!is_secret_hint(b""));
    }
}
