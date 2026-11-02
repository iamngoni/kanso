//! The quick capture shortcut, which works from any app: ⌃⌥Space unless
//! changed in Settings › General. The `global-hotkey` crate registers it
//! with Carbon's RegisterEventHotKey on macOS (no Accessibility permission
//! needed), RegisterHotKey on Windows and an X11 key grab on Linux.
//! Wayland has no way for an app to take a shortcut from every other app,
//! so there Settings says so and `kanso capture` is the way in.
//!
//! The shortcut is kept as GPUI writes a keystroke (`ctrl-alt-space`):
//! modifiers `ctrl`, `alt`, `shift` and `cmd` (⌘, or the Windows key), then
//! one key.

use std::sync::atomic::{AtomicU32, Ordering};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::{App, Global, Keystroke};

use crate::workspace::AppState;

/// The modifiers a shortcut may use, in the order they are written and
/// shown (macOS's order: ⌃⌥⇧⌘).
const MODIFIERS: [(&str, Modifiers, &str, &str); 4] = [
    ("ctrl", Modifiers::CONTROL, "⌃", "Ctrl"),
    ("alt", Modifiers::ALT, "⌥", "Alt"),
    ("shift", Modifiers::SHIFT, "⇧", "Shift"),
    ("cmd", Modifiers::META, "⌘", "Win"),
];

/// The keys a shortcut may end with, as written and as `global-hotkey`
/// names them, and how they are shown.
fn key_code(key: &str) -> Option<(Code, String)> {
    let letter = |c: char| -> Option<Code> {
        Some(match c {
            'a' => Code::KeyA,
            'b' => Code::KeyB,
            'c' => Code::KeyC,
            'd' => Code::KeyD,
            'e' => Code::KeyE,
            'f' => Code::KeyF,
            'g' => Code::KeyG,
            'h' => Code::KeyH,
            'i' => Code::KeyI,
            'j' => Code::KeyJ,
            'k' => Code::KeyK,
            'l' => Code::KeyL,
            'm' => Code::KeyM,
            'n' => Code::KeyN,
            'o' => Code::KeyO,
            'p' => Code::KeyP,
            'q' => Code::KeyQ,
            'r' => Code::KeyR,
            's' => Code::KeyS,
            't' => Code::KeyT,
            'u' => Code::KeyU,
            'v' => Code::KeyV,
            'w' => Code::KeyW,
            'x' => Code::KeyX,
            'y' => Code::KeyY,
            'z' => Code::KeyZ,
            '0' => Code::Digit0,
            '1' => Code::Digit1,
            '2' => Code::Digit2,
            '3' => Code::Digit3,
            '4' => Code::Digit4,
            '5' => Code::Digit5,
            '6' => Code::Digit6,
            '7' => Code::Digit7,
            '8' => Code::Digit8,
            '9' => Code::Digit9,
            _ => return None,
        })
    };
    let mut chars = key.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return letter(c).map(|code| (code, c.to_ascii_uppercase().to_string()));
    }
    let (code, shown) = match key {
        "space" => (Code::Space, "Space"),
        "enter" => (Code::Enter, "Return"),
        "tab" => (Code::Tab, "Tab"),
        "up" => (Code::ArrowUp, "↑"),
        "down" => (Code::ArrowDown, "↓"),
        "left" => (Code::ArrowLeft, "←"),
        "right" => (Code::ArrowRight, "→"),
        "f1" => (Code::F1, "F1"),
        "f2" => (Code::F2, "F2"),
        "f3" => (Code::F3, "F3"),
        "f4" => (Code::F4, "F4"),
        "f5" => (Code::F5, "F5"),
        "f6" => (Code::F6, "F6"),
        "f7" => (Code::F7, "F7"),
        "f8" => (Code::F8, "F8"),
        "f9" => (Code::F9, "F9"),
        "f10" => (Code::F10, "F10"),
        "f11" => (Code::F11, "F11"),
        "f12" => (Code::F12, "F12"),
        _ => return None,
    };
    Some((code, shown.to_owned()))
}

/// A shortcut read from its written form: its modifiers and its key, each
/// written at most once, modifiers first. A shortcut needs at least one
/// modifier other than Shift (except a function key), so it can't take a
/// key every app needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: String,
}

impl Shortcut {
    pub fn parse(text: &str) -> Option<Shortcut> {
        let text = text.trim().to_ascii_lowercase();
        let mut parts: Vec<&str> = text.split('-').collect();
        // `ctrl--`: no such key here.
        let key = parts.pop().filter(|k| !k.is_empty())?;
        key_code(key)?;
        let mut modifiers = Modifiers::empty();
        for part in parts {
            let alias = match part {
                "control" => "ctrl",
                "option" => "alt",
                "command" | "super" | "win" | "meta" | "platform" => "cmd",
                other => other,
            };
            let (_, flag, ..) = MODIFIERS.iter().find(|(name, ..)| *name == alias)?;
            if modifiers.contains(*flag) {
                return None;
            }
            modifiers |= *flag;
        }
        let function_key = key.starts_with('f') && key.len() > 1;
        if !function_key && (modifiers - Modifiers::SHIFT).is_empty() {
            return None;
        }
        Some(Shortcut {
            modifiers,
            key: key.to_owned(),
        })
    }

    /// From a key pressed in Settings, when it makes a shortcut.
    pub fn from_keystroke(keystroke: &Keystroke) -> Option<Shortcut> {
        let mut text = String::new();
        let m = &keystroke.modifiers;
        for (on, name) in [
            (m.control, "ctrl"),
            (m.alt, "alt"),
            (m.shift, "shift"),
            (m.platform, "cmd"),
        ] {
            if on {
                text.push_str(name);
                text.push('-');
            }
        }
        text.push_str(&keystroke.key);
        Shortcut::parse(&text)
    }

    /// The written form: `ctrl-alt-space`.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for (name, flag, ..) in MODIFIERS {
            if self.modifiers.contains(flag) {
                out.push_str(name);
                out.push('-');
            }
        }
        out.push_str(&self.key);
        out
    }

    /// How the shortcut is shown: `⌃⌥Space` on macOS, `Ctrl+Alt+Space`
    /// elsewhere.
    pub fn label(&self) -> String {
        let key = key_code(&self.key).map_or_else(|| self.key.clone(), |(_, shown)| shown);
        if cfg!(target_os = "macos") {
            let mut out = String::new();
            for (_, flag, glyph, _) in MODIFIERS {
                if self.modifiers.contains(flag) {
                    out.push_str(glyph);
                }
            }
            out + &key
        } else {
            let mut parts: Vec<String> = MODIFIERS
                .iter()
                .filter(|(_, flag, ..)| self.modifiers.contains(*flag))
                .map(|(.., word)| (*word).to_owned())
                .collect();
            parts.push(key);
            parts.join("+")
        }
    }

    fn hotkey(&self) -> Option<HotKey> {
        let (code, _) = key_code(&self.key)?;
        Some(HotKey::new(Some(self.modifiers), code))
    }
}

/// Why the shortcut isn't working.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unavailable {
    /// A Wayland session: no app can take a shortcut from every other one.
    Wayland,
    /// The system turned the shortcut down, most often because another app
    /// has it.
    Taken(String),
    /// The shortcut in the settings can't be read.
    Invalid,
}

/// Whether the shortcut works now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    /// Turned off in Settings, or not set up (tests, `kanso serve`).
    #[default]
    Off,
    On(Shortcut),
    Unavailable(Unavailable),
}

/// The registration, kept for the app's life.
#[derive(Default)]
pub struct Hotkeys {
    manager: Option<GlobalHotKeyManager>,
    registered: Option<HotKey>,
    voice_registered: Option<HotKey>,
    pub status: Status,
    /// The second shortcut, which opens the capture window already
    /// listening. `Off` when none is set.
    pub voice_status: Status,
}

/// The ID of the registered voice shortcut, 0 for none, for the event
/// handler (which runs on the system's thread) to tell the two apart.
static VOICE_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);

impl Global for Hotkeys {}

/// Whether this is a Wayland session, where shortcuts can't be global.
/// Under XWayland an X11 grab only sees keys while an X11 window has the
/// keyboard, so a Wayland session counts as Wayland even with `DISPLAY`.
pub fn is_wayland() -> bool {
    cfg!(target_os = "linux")
        && (std::env::var("XDG_SESSION_TYPE").is_ok_and(|kind| kind == "wayland")
            || (std::env::var_os("WAYLAND_DISPLAY").is_some()
                && std::env::var_os("DISPLAY").is_none()))
}

/// Starts listening for the shortcut and registers it as Settings has it.
/// Each press opens the quick capture window.
pub fn init(cx: &mut App) {
    let (tx, rx) = async_channel::unbounded::<bool>();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state() == HotKeyState::Pressed {
            let voice = VOICE_HOTKEY_ID.load(Ordering::Relaxed);
            tx.try_send(voice != 0 && event.id() == voice).ok();
        }
    }));
    cx.spawn(async move |cx| {
        while let Ok(voice) = rx.recv().await {
            cx.update(|cx| {
                if voice {
                    crate::capture::open_listening(cx)
                } else {
                    crate::capture::open(cx)
                }
            });
        }
    })
    .detach();
    cx.set_global(Hotkeys::default());
    apply(cx);
}

/// Registers the shortcut Settings has now, in place of the one before.
/// Does nothing until [`init`] ran.
pub fn apply(cx: &mut App) {
    if !cx.has_global::<Hotkeys>() {
        return;
    }
    let settings = cx.global::<AppState>().config.capture.clone();
    let hotkeys = cx.global_mut::<Hotkeys>();
    unregister_all(hotkeys);
    hotkeys.status = if !settings.enabled {
        Status::Off
    } else if is_wayland() {
        Status::Unavailable(Unavailable::Wayland)
    } else {
        match Shortcut::parse(&settings.shortcut) {
            None => Status::Unavailable(Unavailable::Invalid),
            Some(shortcut) => register(hotkeys, shortcut),
        }
    };
    if let Status::Unavailable(why) = &hotkeys.status {
        eprintln!("kanso: the quick capture shortcut isn't available: {why:?}");
    }
    hotkeys.voice_status = voice_status(hotkeys, &settings);
    if let Status::Unavailable(why) = &hotkeys.voice_status {
        eprintln!("kanso: the voice capture shortcut isn't available: {why:?}");
    }
}

/// Registers the voice capture shortcut when one is set, on, and not the
/// same as the quick note one.
fn voice_status(hotkeys: &mut Hotkeys, settings: &crate::config::CaptureSettings) -> Status {
    if !settings.enabled || settings.voice_shortcut.trim().is_empty() {
        return Status::Off;
    }
    if is_wayland() {
        return Status::Unavailable(Unavailable::Wayland);
    }
    let Some(shortcut) = Shortcut::parse(&settings.voice_shortcut) else {
        return Status::Unavailable(Unavailable::Invalid);
    };
    if Shortcut::parse(&settings.shortcut).as_ref() == Some(&shortcut) {
        return Status::Unavailable(Unavailable::Taken(
            "It is the Quick note shortcut".to_owned(),
        ));
    }
    let status = register_with(hotkeys, shortcut, true);
    if matches!(status, Status::On(_)) {
        if let Some(hotkey) = hotkeys.voice_registered {
            VOICE_HOTKEY_ID.store(hotkey.id(), Ordering::Relaxed);
        }
    }
    status
}

fn unregister_all(hotkeys: &mut Hotkeys) {
    VOICE_HOTKEY_ID.store(0, Ordering::Relaxed);
    if let Some(manager) = &hotkeys.manager {
        if let Some(old) = hotkeys.registered.take() {
            manager.unregister(old).ok();
        }
        if let Some(old) = hotkeys.voice_registered.take() {
            manager.unregister(old).ok();
        }
    }
}

/// Lets go of the shortcut until the next [`apply`], so Settings can take
/// it as the new one when it is pressed.
pub fn suspend(cx: &mut App) {
    if !cx.has_global::<Hotkeys>() {
        return;
    }
    unregister_all(cx.global_mut::<Hotkeys>());
}

fn register(hotkeys: &mut Hotkeys, shortcut: Shortcut) -> Status {
    register_with(hotkeys, shortcut, false)
}

fn register_with(hotkeys: &mut Hotkeys, shortcut: Shortcut, voice: bool) -> Status {
    let Some(hotkey) = shortcut.hotkey() else {
        return Status::Unavailable(Unavailable::Invalid);
    };
    if hotkeys.manager.is_none() {
        match GlobalHotKeyManager::new() {
            Ok(manager) => hotkeys.manager = Some(manager),
            Err(err) => return Status::Unavailable(Unavailable::Taken(err.to_string())),
        }
    }
    let Some(manager) = &hotkeys.manager else {
        return Status::Unavailable(Unavailable::Invalid);
    };
    match manager.register(hotkey) {
        Ok(()) => {
            if voice {
                hotkeys.voice_registered = Some(hotkey);
            } else {
                hotkeys.registered = Some(hotkey);
            }
            Status::On(shortcut)
        }
        Err(err) => Status::Unavailable(Unavailable::Taken(err.to_string())),
    }
}

/// The shortcut's status, for Settings.
pub fn status(cx: &App) -> Status {
    if let Some(hotkeys) = cx.try_global::<Hotkeys>() {
        return hotkeys.status.clone();
    }
    // Not listening yet (or in tests): as the settings have it.
    let settings = cx
        .try_global::<AppState>()
        .map(|state| state.config.capture.clone())
        .unwrap_or_default();
    match Shortcut::parse(&settings.shortcut) {
        Some(shortcut) if settings.enabled => Status::On(shortcut),
        _ => Status::Off,
    }
}

/// The voice capture shortcut's status, for Settings.
pub fn voice_status_now(cx: &App) -> Status {
    if let Some(hotkeys) = cx.try_global::<Hotkeys>() {
        return hotkeys.voice_status.clone();
    }
    let settings = cx
        .try_global::<AppState>()
        .map(|state| state.config.capture.clone())
        .unwrap_or_default();
    match Shortcut::parse(&settings.voice_shortcut) {
        Some(shortcut) if settings.enabled => Status::On(shortcut),
        _ => Status::Off,
    }
}

/// What Settings says about a shortcut that isn't working. Pure.
pub fn unavailable_note(why: &Unavailable, label: &str) -> String {
    match why {
        Unavailable::Wayland => "Wayland doesn't let apps take a shortcut from every other app. Bind `kanso capture` to a shortcut in your desktop's keyboard settings instead; it saves what you type, or what you pipe to it, to the Inbox.".to_owned(),
        Unavailable::Taken(_) => format!("{label} couldn't be set up; another app may be using it. Choose another shortcut, or use `kanso capture` from a terminal."),
        Unavailable::Invalid => "The shortcut in the settings can't be used. Choose another one.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn examples() {
        let default = Shortcut::parse(crate::config::DEFAULT_CAPTURE_SHORTCUT).unwrap();
        assert_eq!(default.modifiers, Modifiers::CONTROL | Modifiers::ALT);
        assert_eq!(default.key, "space");
        if cfg!(target_os = "macos") {
            assert_eq!(default.label(), "⌃⌥Space");
        } else {
            assert_eq!(default.label(), "Ctrl+Alt+Space");
        }
        assert_eq!(
            Shortcut::parse("Control-Option-N").unwrap().text(),
            "ctrl-alt-n"
        );
        assert_eq!(
            Shortcut::parse("shift-cmd-k").unwrap().text(),
            "shift-cmd-k"
        );
        assert_eq!(Shortcut::parse("f5").unwrap().text(), "f5");
        assert!(Shortcut::parse("space").is_none(), "no modifier");
        assert!(Shortcut::parse("shift-a").is_none(), "Shift alone");
        assert!(Shortcut::parse("ctrl-ctrl-a").is_none());
        assert!(Shortcut::parse("ctrl-hyper-a").is_none());
        assert!(Shortcut::parse("ctrl-").is_none());
        assert!(Shortcut::parse("ctrl-escape").is_none());
        let keystroke = Keystroke::parse("ctrl-alt-space").unwrap();
        assert_eq!(Shortcut::from_keystroke(&keystroke), Some(default));
        assert!(unavailable_note(&Unavailable::Wayland, "x").contains("kanso capture"));
    }

    fn key() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-z0-9]",
            prop::sample::select(vec!["space", "enter", "tab", "up", "f1", "f12"])
                .prop_map(str::to_owned),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round trip: a shortcut's written form reads back as the same
        /// shortcut, whatever order and case the modifiers were typed in.
        #[test]
        fn prop_shortcuts_round_trip(
            mods in prop::sample::subsequence(vec!["ctrl", "alt", "shift", "cmd"], 1..=4),
            key in key(),
            upper in any::<bool>(),
            reversed in any::<bool>(),
        ) {
            let mut mods = mods;
            if reversed {
                mods.reverse();
            }
            let mut text = mods.join("-");
            text.push('-');
            text.push_str(&key);
            if upper {
                text = text.to_uppercase();
            }
            let only_shift = mods == ["shift"];
            match Shortcut::parse(&text) {
                Some(shortcut) => {
                    prop_assert!(!only_shift || key.starts_with('f') && key.len() > 1);
                    prop_assert_eq!(Shortcut::parse(&shortcut.text()), Some(shortcut.clone()));
                    prop_assert!(shortcut.hotkey().is_some());
                    prop_assert!(!shortcut.label().is_empty());
                }
                None => prop_assert!(only_shift),
            }
        }

        /// Invariant: what can't be a shortcut never parses, and nothing
        /// panics.
        #[test]
        fn prop_parse_never_panics(text in "\\PC{0,20}") {
            if let Some(shortcut) = Shortcut::parse(&text) {
                prop_assert_eq!(Shortcut::parse(&shortcut.text()), Some(shortcut));
            }
        }
    }
}
