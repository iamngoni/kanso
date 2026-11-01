//! Clipboard history: the last texts copied in any app, while Kanso runs,
//! for the clipboard sticky (`clipboard_sticky.rs`) to copy again with a
//! click. Off until turned on (Settings › General, or the sticky's own
//! Turn on), and kept in memory only.
//!
//! What was copied can be anything: a password, a key, a card number. So
//! the history:
//!
//! - is never written anywhere: not the store, so not search, sync, the
//!   local API or MCP; not the config, nor a log. Quitting forgets it, so
//!   does turning it off. On purpose, no IPC message, API route, MCP tool
//!   or command reads it.
//! - skips a copy the app it came from marked secret (a password manager's
//!   concealed or KDE password hint, `clipboard_watch.rs`), one of Kanso's
//!   own secrets (the recovery key), and text that looks like a key, a
//!   token or a card number ([`screen`]).
//! - forgets each copy after a while (an hour by default), and shows them
//!   masked until hovered.
//!
//! The copies come from a watcher on a thread of its own
//! (`clipboard_watch.rs`); a sticky opened later shows what it already
//! saw.

use std::collections::VecDeque;

use gpui_kit::*;

use crate::clipboard_watch::{self, Copied, Support, Watcher};
use crate::clock::Clock;
use crate::workspace::AppState;

/// How many copies the history keeps: the newest.
pub const CAPACITY: usize = 15;
/// The longest copy kept, in bytes: a snippet, not a document.
pub const MAX_CLIP: usize = 16 * 1024;
/// How often old copies are forgotten.
const SWEEP: std::time::Duration = std::time::Duration::from_secs(60);

/// One copied text, and when (ms since the epoch).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub text: String,
    pub copied_ms: i64,
}

/// The copies kept, newest first, and whether collecting is paused.
#[derive(Debug, Default)]
pub struct History {
    pub items: VecDeque<Clip>,
    pub paused: bool,
}

impl History {
    /// Keeps `text`, copied at `now`, as the newest. A copy already kept
    /// moves to the front; blank and oversized ones are dropped. Returns
    /// whether the history changed.
    pub fn push(&mut self, text: String, now: i64) -> bool {
        if text.trim().is_empty() || text.len() > MAX_CLIP {
            return false;
        }
        if let Some(ix) = self.items.iter().position(|clip| clip.text == text) {
            self.items.remove(ix);
        }
        self.items.push_front(Clip {
            text,
            copied_ms: now,
        });
        self.items.truncate(CAPACITY);
        true
    }

    /// Forgets the copies older than `max_age_ms` at `now`. Returns whether
    /// any were.
    pub fn expire(&mut self, now: i64, max_age_ms: i64) -> bool {
        let before = self.items.len();
        self.items.retain(|clip| now - clip.copied_ms < max_age_ms);
        self.items.len() != before
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Whether a copy may be kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Keep,
    /// Looks like a key, a token, a password or a card number.
    Secret,
}

/// Whether `text` looks secret: a known kind of key or token, a card
/// number, or a lone word random enough to be a generated password. A
/// guess, so the history is masked and forgets too. Pure.
pub fn screen(text: &str) -> Verdict {
    let text = text.trim();
    let secret = text.contains("-----BEGIN ") && text.contains("KEY")
        || text.split_whitespace().any(is_token)
        || is_card_number(text)
        || is_random_word(text);
    if secret {
        Verdict::Secret
    } else {
        Verdict::Keep
    }
}

/// A word that is a known kind of key or token.
fn is_token(word: &str) -> bool {
    let word = word.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | ';' | '`'));
    // A value assigned in a line copied from a config: `KEY=ghp_…`.
    let word = word.rsplit_once('=').map_or(word, |(_, value)| value);
    let rest_after = |prefix: &str| {
        word.strip_prefix(prefix)
            .map(|rest| rest.len())
            .unwrap_or(0)
    };
    // AWS access key IDs.
    if let Some(rest) = word
        .strip_prefix("AKIA")
        .or_else(|| word.strip_prefix("ASIA"))
        && rest.len() == 16
        && rest
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return true;
    }
    // GitHub, OpenAI and Anthropic style, Stripe, Slack.
    let prefixed = [
        ("ghp_", 30),
        ("gho_", 30),
        ("ghs_", 30),
        ("ghu_", 30),
        ("ghr_", 30),
        ("github_pat_", 30),
        ("glpat-", 20),
        ("sk-", 20),
        ("sk_live_", 16),
        ("rk_live_", 16),
        ("xoxb-", 20),
        ("xoxp-", 20),
        ("xoxa-", 20),
        ("AIza", 30),
    ];
    if prefixed
        .iter()
        .any(|(prefix, min)| rest_after(prefix) >= *min)
    {
        return true;
    }
    // A JSON Web Token: three base64url parts, the first a JSON header.
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() == 3
        && word.starts_with("eyJ")
        && parts.iter().all(|part| {
            part.len() >= 8
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    {
        return true;
    }
    false
}

/// 13 to 19 digits, spaced or dashed or not, that pass the Luhn check.
fn is_card_number(text: &str) -> bool {
    if !text
        .chars()
        .all(|c| c.is_ascii_digit() || c == ' ' || c == '-')
    {
        return false;
    }
    let digits: Vec<u32> = text.chars().filter_map(|c| c.to_digit(10)).collect();
    (13..=19).contains(&digits.len()) && luhn(&digits)
}

/// The Luhn checksum cards end with.
pub fn luhn(digits: &[u32]) -> bool {
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                d
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// A lone word of 20 or more characters, with at least three of lower
/// case, upper case, digits and symbols, and random: what a password
/// generator makes. Paths and links aren't (their `/` and `.` repeat, and
/// they score lower).
fn is_random_word(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 20 || chars.len() > 256 || chars.iter().any(|c| c.is_whitespace()) {
        return false;
    }
    if text.contains("://") || text.starts_with('/') || text.starts_with('~') || is_uuid(text) {
        return false;
    }
    let kinds = [
        chars.iter().any(char::is_ascii_lowercase),
        chars.iter().any(char::is_ascii_uppercase),
        chars.iter().any(char::is_ascii_digit),
        chars.iter().any(|c| !c.is_ascii_alphanumeric()),
    ]
    .into_iter()
    .filter(|&kind| kind)
    .count();
    kinds >= 3 && entropy(&chars) >= 3.5
}

/// An ID such as `0b9f…-…`: copied often, and an ID rather than a key.
fn is_uuid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.iter().map(|p| p.len()).eq([8, 4, 4, 4, 12])
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Shannon entropy of `chars`, in bits per character.
fn entropy(chars: &[char]) -> f64 {
    let mut counts = std::collections::HashMap::new();
    for c in chars {
        *counts.entry(c).or_insert(0usize) += 1;
    }
    let n = chars.len() as f64;
    counts
        .values()
        .map(|&count| {
            let p = count as f64 / n;
            -p * p.log2()
        })
        .sum()
}

/// How a copy shows in the sticky while masked: its first line, a long
/// word's ends with dots between, a short one only dots.
pub fn mask(text: &str) -> String {
    let line = first_line(text);
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= 8 {
        return "••••".to_owned();
    }
    let ends = if chars.len() >= 16 { 4 } else { 2 };
    let head: String = chars[..ends].iter().collect();
    let tail: String = chars[chars.len() - ends..].iter().collect();
    format!("{head}••••{tail}")
}

/// The first line of `text` with something on it, for a row.
pub fn first_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
}

/// The history, the watcher filling it, and what this session allows.
pub struct ClipboardHistory {
    pub history: Entity<History>,
    pub support: Support,
    watcher: Option<Watcher>,
    copies: async_channel::Sender<Copied>,
}

impl Global for ClipboardHistory {}

/// Sets up the history, empty, and starts watching when it's turned on.
pub fn init(cx: &mut App) {
    let (copies, received) = async_channel::unbounded();
    let history = cx.new(|_| History::default());
    cx.set_global(ClipboardHistory {
        history,
        support: clipboard_watch::support(cx),
        watcher: None,
        copies,
    });
    if enabled(cx) {
        start(cx);
    }
    serve(received, cx);
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(SWEEP).await;
            cx.update(sweep);
        }
    })
    .detach();
}

/// Whether the history is turned on.
pub fn enabled(cx: &App) -> bool {
    cx.global::<AppState>().config.clipboard.enabled
}

/// Turns the history on, watching from now, or off, forgetting it all.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.clipboard.enabled = on);
    if on {
        start(cx);
    } else {
        cx.global_mut::<ClipboardHistory>().watcher = None;
        let history = cx.global::<ClipboardHistory>().history.clone();
        history.update(cx, |history, cx| {
            history.clear();
            history.paused = false;
            cx.notify();
        });
    }
}

fn start(cx: &mut App) {
    let state = cx.global::<ClipboardHistory>();
    if state.watcher.is_some() || state.support != Support::Watching {
        return;
    }
    let watcher = clipboard_watch::start(state.copies.clone());
    cx.global_mut::<ClipboardHistory>().watcher = watcher;
}

/// Keeps each copy the watcher sees that may be kept.
fn serve(received: async_channel::Receiver<Copied>, cx: &mut App) {
    cx.spawn(async move |cx| {
        while let Ok(copied) = received.recv().await {
            cx.update(|cx| keep(copied, cx));
        }
    })
    .detach();
}

/// Keeps `copied` unless the history is off or paused, or the copy is
/// secret.
pub fn keep(copied: Copied, cx: &mut App) {
    if !enabled(cx)
        || copied.sensitive
        || screen(&copied.text) == Verdict::Secret
        || crate::clipboard::is_own_secret(&copied.text)
    {
        return;
    }
    let Some(state) = cx.try_global::<ClipboardHistory>() else {
        return;
    };
    let history = state.history.clone();
    let now = Clock::now_ms(cx);
    history.update(cx, |history, cx| {
        if !history.paused && history.push(copied.text, now) {
            cx.notify();
        }
    });
}

/// Forgets the copies older than the setting, and has the rows' ages
/// redrawn.
fn sweep(cx: &mut App) {
    let Some(state) = cx.try_global::<ClipboardHistory>() else {
        return;
    };
    let history = state.history.clone();
    let max_age = i64::from(cx.global::<AppState>().config.clipboard.expire_minutes) * 60_000;
    let now = Clock::now_ms(cx);
    history.update(cx, |history, cx| {
        history.expire(now, max_age);
        // Redrawn either way: the rows say how long ago.
        cx.notify();
    });
}

#[cfg(test)]
mod tests {
    use super::{CAPACITY, History, MAX_CLIP, Verdict, luhn, mask, screen};
    use proptest::prelude::*;

    #[test]
    fn keeps_the_newest_fifteen_once_each() {
        let mut history = History::default();
        for i in 0..20 {
            assert!(history.push(format!("copy {i}"), i));
        }
        assert_eq!(history.items.len(), CAPACITY);
        assert_eq!(history.items[0].text, "copy 19");
        assert_eq!(history.items[CAPACITY - 1].text, "copy 5");
        // Copied again: to the front, not twice.
        assert!(history.push("copy 10".into(), 30));
        assert_eq!(history.items.len(), CAPACITY);
        assert_eq!(history.items[0].text, "copy 10");
        assert_eq!(
            history.items.iter().filter(|c| c.text == "copy 10").count(),
            1
        );
    }

    #[test]
    fn drops_blank_and_oversized_copies() {
        let mut history = History::default();
        assert!(!history.push("   \n\t".into(), 0));
        assert!(!history.push("x".repeat(MAX_CLIP + 1), 0));
        assert!(history.items.is_empty());
    }

    #[test]
    fn forgets_old_copies() {
        let mut history = History::default();
        history.push("old".into(), 0);
        history.push("new".into(), 50_000);
        assert!(history.expire(70_000, 60_000));
        let left: Vec<_> = history.items.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(left, ["new"]);
        assert!(!history.expire(70_000, 60_000));
    }

    #[test]
    fn screens_out_keys_tokens_and_cards() {
        let secrets = [
            "AKIAIOSFODNN7EXAMPLE",
            "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
            "export GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz0123456789",
            "github_pat_11ABCDEFG0abcdefghijklmnopqrstuvwxyz",
            "sk-proj-abcdefghijklmnopqrstuvwxyz012345",
            "sk_live_abcdefghijklmnop1234",
            "xoxb-123456789012-1234567890123-abcdefghij",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
            "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAA\n-----END OPENSSH PRIVATE KEY-----",
            "4111 1111 1111 1111",
            "5500-0000-0000-0004",
            "x7#Kq9!mZ2@pL4$vN8&wR3",
            "Tr0ub4dor&3xQ9zLmP2kW7vB",
        ];
        for secret in secrets {
            assert_eq!(screen(secret), Verdict::Secret, "{secret}");
        }
    }

    #[test]
    fn keeps_ordinary_text() {
        let ordinary = [
            "docker ps -a",
            "Remember to buy milk",
            "https://github.com/iamngoni/kanso/pull/173",
            "/home/user/Documents/github/kanso/crates",
            "kanso_app::clipboard_history::screen",
            "supercalifragilisticexpialidocious",
            "cargo test -p kanso-app --locked",
            "4111 1111 1111 1112",
            "12345",
            "sk-short",
            "let x = 42;",
            "2026-11-01T10:30:00Z",
            "0b9f6c2e-3d4a-4f1b-9a7e-5c8d2e1f0a3b",
            "ed3fa62e4b1c9d8f7a6e5d4c3b2a1f0e9d8c7b6a",
        ];
        for text in ordinary {
            assert_eq!(screen(text), Verdict::Keep, "{text}");
        }
    }

    #[test]
    fn masks_all_but_the_ends() {
        assert_eq!(mask("hi"), "••••");
        assert_eq!(mask("docker ps -a"), "do••••-a");
        assert_eq!(mask("ssh deploy@example.com\nmore"), "ssh ••••.com");
        assert_eq!(mask("\n\n  short"), "••••");
    }

    /// The Luhn check digit of `digits`.
    fn check_digit(digits: &[u32]) -> u32 {
        (0..10)
            .find(|d| {
                let mut all = digits.to_vec();
                all.push(*d);
                luhn(&all)
            })
            .unwrap()
    }

    proptest! {
        /// Invariant: any 13 to 19 digit number with a right check digit
        /// is screened out, and the same number with a wrong one is kept.
        #[test]
        fn prop_card_numbers_are_screened_by_their_check_digit(
            digits in proptest::collection::vec(0u32..10, 12..19),
            off in 1u32..10,
        ) {
            let good = check_digit(&digits);
            let text = |last: u32| {
                digits.iter().chain([last].iter()).map(|d| d.to_string()).collect::<String>()
            };
            prop_assert_eq!(screen(&text(good)), Verdict::Secret);
            prop_assert_eq!(screen(&text((good + off) % 10)), Verdict::Keep);
        }

        /// Invariant: the history never holds more than its capacity, nor
        /// the same text twice.
        #[test]
        fn prop_the_history_stays_small_and_unique(
            copies in proptest::collection::vec("[a-c]{1,2}", 0..60),
        ) {
            let mut history = History::default();
            for (i, text) in copies.into_iter().enumerate() {
                history.push(text, i as i64);
            }
            prop_assert!(history.items.len() <= CAPACITY);
            let mut texts: Vec<_> = history.items.iter().map(|c| c.text.clone()).collect();
            texts.sort();
            texts.dedup();
            prop_assert_eq!(texts.len(), history.items.len());
        }
    }
}
