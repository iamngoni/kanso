//! Where the time of a fresh highlight goes, on the host: a release-mode
//! harness, not a test of behavior. Run it with
//!
//! ```sh
//! cargo test -p kanso-mobile --release --test highlight_latency -- --ignored --nocapture
//! ```
//!
//! It opens sessions on generated notes of 5,000, 20,000 and 50,000 words
//! (with one broken table three quarters of the way down, the syntax error
//! the Dart fixture has), types at the end and in the middle, and times
//! `styled_runs` for the whole text and for a window of 8,192 units around
//! the caret, cold and warm.

mod common;

use std::time::{Duration, Instant};

use common::*;
use kanso_engine::highlight::Highlighter;
use kanso_mobile::{EditorSession, Range16, TextChange};

const THEME: &str = "kanso-light";
const WINDOW: u32 = 8192;
const KEYSTROKES: usize = 24;

fn micros(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

fn stats(mut times: Vec<f64>) -> String {
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |p: f64| times[((times.len() as f64 - 1.0) * p) as usize];
    format!(
        "p50 {:>9.0} us  p95 {:>9.0} us  max {:>9.0} us",
        at(0.5),
        at(0.95),
        times[times.len() - 1]
    )
}

struct Open {
    session: EditorSession,
    version: u64,
    units: usize,
}

fn open(phone: &Phone, text: &str) -> Open {
    let id = note(&phone.library, "Title", text);
    let session = phone.library.edit(&id).unwrap();
    let snapshot = session.snapshot().unwrap();
    Open {
        session,
        version: snapshot.version,
        units: snapshot.text.encode_utf16().count(),
    }
}

impl Open {
    fn type_at(&mut self, at: usize) {
        let out = self
            .session
            .apply(TextChange {
                base_version: self.version,
                start: at as u32,
                end: at as u32,
                text: "x".into(),
                selection: Range16 {
                    start: at as u32 + 1,
                    end: at as u32 + 1,
                },
                composing: None,
            })
            .unwrap();
        self.version = out.version;
        self.units += 1;
    }

    fn window(&self, caret: usize) -> (u32, u32) {
        let start = caret.saturating_sub(WINDOW as usize / 2) as u32;
        (start, (start + WINDOW).min(self.units as u32))
    }
}

/// The times of `KEYSTROKES` keystrokes at `at`, each followed by a request
/// for `range` (None: the whole text).
fn typing(open: &mut Open, at: usize, windowed: bool) -> (Vec<f64>, usize) {
    let mut times = Vec::new();
    let mut runs = 0;
    for i in 0..KEYSTROKES {
        open.type_at(at + i);
        let (start, end) = if windowed {
            open.window(at + i)
        } else {
            (0, open.units as u32)
        };
        let began = Instant::now();
        let batch = open.session.styled_runs(start, end, THEME.into()).unwrap();
        times.push(micros(began.elapsed()));
        runs = batch.runs.len();
    }
    (times, runs)
}

#[test]
#[ignore = "a measurement, run in release mode"]
fn where_a_fresh_highlight_goes() {
    for words in [5_000usize, 20_000, 50_000] {
        let text = long_note_with_error(words);
        let units = text.encode_utf16().count();
        println!(
            "\n=== {words} words: {} bytes, {units} units ===",
            text.len()
        );
        let phone = lone_phone();
        let positions = [
            ("end (after the error)", units),
            ("90% (after the error)", units * 9 / 10),
            ("40% (before the error)", units * 4 / 10),
        ];
        for windowed in [false, true] {
            let kind = if windowed {
                "window 8192"
            } else {
                "whole text"
            };
            for (label, at) in positions {
                // Cold: a fresh session, the first request of its life.
                let mut cold = open(&phone, &text);
                let (start, end) = if windowed {
                    cold.window(at)
                } else {
                    (0, cold.units as u32)
                };
                let began = Instant::now();
                let batch = cold.session.styled_runs(start, end, THEME.into()).unwrap();
                let first = micros(began.elapsed());
                // Warm the whole note once, as scrolling through it would.
                cold.session
                    .styled_runs(0, cold.units as u32, THEME.into())
                    .unwrap();
                let caret = at.min(cold.units);
                let (times, runs) = typing(&mut cold, caret, windowed);
                println!(
                    "{kind:<12} {label:<24} cold first {first:>9.0} us ({} runs) | after a keystroke {} ({runs} runs)",
                    batch.runs.len(),
                    stats(times)
                );
            }
        }
        // The highlighter alone, on the whole text, with a keystroke between.
        let mut highlighter = Highlighter::new();
        let began = Instant::now();
        let spans = highlighter.spans(&text, 0..text.len());
        println!(
            "Highlighter alone: cold whole text {:>9.0} us ({} spans, {} blocks parsed)",
            micros(began.elapsed()),
            spans.len(),
            highlighter.blocks_parsed()
        );
        let mut edited = text.clone();
        let mut times = Vec::new();
        for i in 0..KEYSTROKES {
            let at = edited.len() * 9 / 10;
            let at = (0..=at)
                .rev()
                .find(|&b| edited.is_char_boundary(b))
                .unwrap();
            edited.insert(at + i % 3, 'x');
            let began = Instant::now();
            highlighter.spans(&edited, 0..edited.len());
            times.push(micros(began.elapsed()));
        }
        println!(
            "Highlighter alone: keystroke at 90%, whole text {}",
            stats(times)
        );
    }
}
