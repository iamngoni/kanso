//! How soon the preview beside the source shows what is typed, measured on
//! the real views with real text shaping and layout. Run with
//!
//! ```text
//! cargo test -p kanso-app latency_bench -- --ignored --nocapture
//! ```
//!
//! `KANSO_BENCH_INSPECTOR=1` measures with the outline open beside the
//! note.
//!
//! `remote_merge_bench` measures someone else's edit to an open shared
//! note merging into the editor (`cargo test -p kanso-app
//! remote_merge_bench -- --ignored --nocapture`).
//!
//! Keystrokes arrive on a simulated clock (one every [`typing_interval`],
//! frames every [`FRAME`]), so the delays the editor chooses are measured
//! exactly; the work each frame does (parsing, layout, painting) is timed on
//! the wall clock.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::base::input::{EditorState, Position};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, EntityInputHandler as _, HeadlessAppContext,
    WindowBounds, WindowOptions, px, size,
};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::{AppConfig, ThemePreference, ThemeSettings, ViewMode};
use crate::services::Services;
use crate::workspace::{AppState, Workspace, apply_theme};

/// A fast typist: ten characters a second (`KANSO_BENCH_INTERVAL_MS`
/// changes it).
fn typing_interval() -> Duration {
    Duration::from_millis(
        std::env::var("KANSO_BENCH_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100),
    )
}
/// A 60 Hz display.
const FRAME: Duration = Duration::from_millis(16);
const KEYSTROKES: usize = 40;

/// One section of a realistic note: headings, prose with inline markup,
/// lists, tasks, a quote, code, a table and an image.
fn section(n: usize) -> String {
    format!(
        "## Section {n}: sync and storage\n\n\
         Documents live in D1 and attachments in R2. Last writer wins by `updatedAt`; \
         the loser stays in history, so **nothing is lost** and *every* revision can be \
         restored. See the [protocol overview](https://example.com/{n}) for details.\n\n\
         - First point about section {n}, with `inline code`\n\
         - Second point\n  - A nested point under it\n  - Another nested point\n\
         - Third point, which runs on long enough to wrap onto a second line in the preview pane\n\n\
         1. Step one\n2. Step two\n3. Step three\n\n\
         - [x] Done task {n}\n- [ ] Open task {n}\n\n\
         > A quote that spans\n> two lines, with **bold** text.\n\n\
         ```rust\nfn section_{n}(store: &Store) -> Result<()> {{\n    let note = store.note(\"note:{n}\")?;\n    println!(\"{{note:?}}\");\n    Ok(())\n}}\n```\n\n\
         | Permission | Default | Notes |\n|---|---|---|\n| Read | Allowed | {n} |\n| Write | Ask | — |\n| Delete | Ask | — |\n\n\
         ![diagram {n}](kanso://file:missing{n})\n\n"
    )
}

/// A realistic note of about `bytes` bytes.
pub(crate) fn realistic_note(bytes: usize) -> String {
    let mut text = String::from("# A long note\n\nAn introduction paragraph.\n\n");
    let mut n = 1;
    while text.len() < bytes {
        text.push_str(&section(n));
        n += 1;
    }
    text
}

struct Bench {
    cx: HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    _dir: tempfile::TempDir,
}

impl Bench {
    fn open(body: &str) -> Bench {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"title": "Long", "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let mut cx = crate::snapshots::context(None);
        let (window, workspace) = open_split(&mut cx, store, dir.path(), &note);
        let mut bench = Bench {
            cx,
            window,
            workspace,
            _dir: dir,
        };
        for _ in 0..4 {
            bench.cx.advance_clock(Duration::from_millis(500));
            bench.cx.run_until_parked();
            bench.draw(true);
        }
        bench
    }

    fn draw(&mut self, refresh: bool) {
        self.cx
            .update_window(self.window, |_, window, cx| {
                if refresh {
                    window.refresh();
                }
                window.draw(cx).clear(cx);
            })
            .unwrap();
    }

    fn body(&mut self) -> Entity<EditorState> {
        let workspace = self.workspace.clone();
        self.cx
            .update(|cx| workspace.read(cx).editor.read(cx).body_state())
    }

    /// The text the preview shows, as of its last layout.
    fn shown(&mut self) -> String {
        let workspace = self.workspace.clone();
        self.cx.update(|cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.read(cx).preview_shown_for_test(cx)
        })
    }

    fn text(&mut self) -> String {
        let body = self.body();
        self.cx.update(|cx| body.read(cx).text().to_string())
    }

    fn type_char(&mut self, c: &str) {
        let body = self.body();
        self.cx
            .update_window(self.window, |_, window, cx| {
                body.update(cx, |state, cx| {
                    state.replace_text_in_range(None, c, window, cx)
                })
            })
            .unwrap();
    }

    /// Types `c` as a key, through the window: the writing aids (pairing)
    /// see it first, as in the app.
    fn press(&mut self, c: &str) {
        let keystroke = gpui_kit::Keystroke::parse(c).unwrap();
        self.cx
            .update_window(self.window, |_, window, cx| {
                window.dispatch_keystroke(keystroke, cx);
            })
            .unwrap();
    }

    fn set_mode(&mut self, mode: ViewMode) {
        let workspace = self.workspace.clone();
        self.cx.update(|cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.set_view_mode(mode, cx));
        });
        for _ in 0..3 {
            self.cx.advance_clock(Duration::from_millis(500));
            self.cx.run_until_parked();
            self.draw(true);
        }
    }

    fn place_cursor(&mut self, line: u32) {
        let body = self.body();
        self.cx
            .update_window(self.window, |_, window, cx| {
                body.update(cx, |state, cx| {
                    state.focus(window, cx);
                    state.set_cursor_position(Position::new(line, 0), window, cx);
                })
            })
            .unwrap();
        self.cx.run_until_parked();
        self.draw(false);
    }
}

fn open_split(
    cx: &mut HeadlessAppContext,
    store: Arc<Store>,
    data_dir: &Path,
    note: &str,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let mut config = AppConfig::default();
    config.appearance.theme = ThemePreference::Light;
    config.appearance.themes = ThemeSettings::from_preference(ThemePreference::Light);
    config.appearance.view_mode = ViewMode::Split;
    config.last_note = Some(note.to_owned());
    cx.update(|cx| {
        cx.set_global(AppState {
            store,
            data_dir: data_dir.to_path_buf(),
            config,
        });
        cx.set_global(Services::default());
        apply_theme(None, cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Default::default(),
                    size: size(px(1600.), px(1000.)),
                })),
                focus: false,
                show: false,
                ..Default::default()
            },
            cx,
            |window, cx| {
                apply_theme(Some(window), cx);
                cx.new(|cx| Workspace::new(window, cx))
            },
        )
    })
    .unwrap()
}

/// Milliseconds spent splitting notes so far; in the app that runs on a
/// background thread.
fn split_ms() -> f64 {
    crate::preview_chunks::PREPARE_NANOS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.
}

fn percentile(values: &mut [f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.;
    }
    values.sort_by(f64::total_cmp);
    values[((values.len() - 1) as f64 * p).round() as usize]
}

/// Types [`KEYSTROKES`] characters at `line` of a note of about `bytes`, and
/// reports what the preview did. With `keys`, it types keys through the
/// window instead, making "(x) " over and over: with pairing on (`true`) by
/// typing "(", "x", ")" (stepped over) and a space; with it off, by typing
/// the closer and moving back into the brackets, as one would by hand.
fn run(bytes: usize, place: &str, keys: Option<bool>) {
    let by_keys = keys.is_some();
    let note = realistic_note(bytes);
    let lines = note.lines().count() as u32;
    let line = match place {
        "end" => lines,
        _ => lines / 2,
    };
    let mut bench = Bench::open(&note);
    // With the outline beside the note (`KANSO_BENCH_INSPECTOR`), as many
    // writers keep it.
    if std::env::var_os("KANSO_BENCH_INSPECTOR").is_some() {
        let workspace = bench.workspace.clone();
        bench.cx.update(|cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(crate::config::InspectorTab::Outline), cx)
            })
        });
        bench.draw(true);
    }
    if keys == Some(false) {
        let workspace = bench.workspace.clone();
        bench.cx.update(|cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| {
                editor.set_settings(
                    crate::config::EditorSettings {
                        auto_pair: false,
                        ..Default::default()
                    },
                    cx,
                )
            });
        });
    }
    // With Vim keys on (`KANSO_BENCH_VIM`), typing in Insert mode.
    let vim = std::env::var_os("KANSO_BENCH_VIM").is_some();
    if vim {
        let workspace = bench.workspace.clone();
        bench.cx.update(|cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| {
                editor.set_settings(
                    crate::config::EditorSettings {
                        auto_pair: keys != Some(false),
                        vim: true,
                        ..Default::default()
                    },
                    cx,
                )
            });
        });
    }
    bench.place_cursor(line);
    if vim {
        bench.press("i");
        bench.cx.run_until_parked();
        bench.draw(false);
    }
    if std::env::var_os("KANSO_BENCH_REDRAW").is_some() {
        for mode in [
            ViewMode::Edit,
            ViewMode::Live,
            ViewMode::Preview,
            ViewMode::Split,
        ] {
            bench.set_mode(mode);
            let mut took = Vec::new();
            for _ in 0..4 {
                let start = Instant::now();
                bench.draw(true);
                took.push(ms(start.elapsed()));
            }
            println!("    full redraw in {mode:?}: {took:.1?} ms");
        }
    }
    // One preview update on its own: the note with three characters typed
    // where the cursor is, handed straight to the preview. Each frame after
    // it runs everything pending (the parse, which runs on a background
    // thread in the app but on this one here) and draws.
    {
        let at = if place == "end" {
            note.len()
        } else {
            note[..note.len() / 2].rfind('\n').unwrap()
        };
        let mut edited = note.clone();
        edited.insert_str(at, "zzz");
        let parse = {
            let start = Instant::now();
            markdown::to_mdast(&edited, &markdown::ParseOptions::gfm()).unwrap();
            ms(start.elapsed())
        };
        let mut frames = Vec::new();
        let mut off_thread = 0.;
        for text in [edited, note.clone()] {
            let workspace = bench.workspace.clone();
            bench.cx.update(|cx| {
                let editor = workspace.read(cx).editor.clone();
                editor.update(cx, |e, cx| e.set_preview_for_test(text, cx));
            });
            frames.clear();
            off_thread = 0.;
            for _ in 0..12 {
                let start = Instant::now();
                let split_before = split_ms();
                bench.cx.run_until_parked();
                bench.draw(false);
                let split = split_ms() - split_before;
                off_thread += split;
                frames.push(ms(start.elapsed()) - split);
                bench.cx.advance_clock(FRAME);
            }
        }
        let total: f64 = frames.iter().sum();
        let worst = frames.iter().copied().fold(0., f64::max);
        println!(
            "    one update: main thread worst frame {worst:.1} ms, 12 frames {total:.1} ms \
             (idle frame ~{:.1} ms); off the main thread {off_thread:.1} ms; \
             a whole-note Markdown parse alone takes {parse:.1} ms",
            frames[frames.len() - 1]
        );
    }
    // For a profiler: `refresh` redraws everything, `type` types.
    match std::env::var("KANSO_BENCH_SPIN").as_deref() {
        Ok("refresh") => {
            println!("spinning pid {}", std::process::id());
            for _ in 0..300 {
                bench.draw(true);
            }
            return;
        }
        Ok("keys") => {
            println!("spinning pid {}", std::process::id());
            const KEYS: [&str; 4] = ["(", "x", ")", "space"];
            for i in 0..1500 {
                bench.press(KEYS[i % 4]);
                bench.cx.run_until_parked();
                bench.draw(false);
                bench.cx.advance_clock(FRAME);
            }
            return;
        }
        Ok("type") => {
            println!("spinning pid {}", std::process::id());
            for _ in 0..1500 {
                bench.type_char("x");
                bench.cx.run_until_parked();
                bench.draw(false);
                bench.cx.advance_clock(FRAME);
            }
            return;
        }
        _ => {}
    }

    let frames_per_key = (typing_interval().as_millis() / FRAME.as_millis()) as usize;
    let mut key_frames = Vec::new();
    let mut preview_frames = Vec::new();
    let mut other_frames = Vec::new();
    // When each keystroke was typed (simulated), and the text it made.
    let mut typed: Vec<(Duration, String)> = Vec::new();
    let mut latencies = Vec::new();
    let mut updates = Vec::new();
    let mut shown = bench.shown();
    let mut now = Duration::ZERO;
    let mut caught_up = 0;
    let mut split_total = 0.;
    let total_frames = KEYSTROKES * frames_per_key + 2000 / FRAME.as_millis() as usize;
    for frame in 0..total_frames {
        let keystroke = frame % frames_per_key == 0 && typed.len() < KEYSTROKES;
        let start = Instant::now();
        let split_before = split_ms();
        if keystroke && by_keys {
            // The same text either way: pairing closes the bracket and
            // steps over it; without it the keys do.
            let keys: &[&str] = if keys == Some(true) {
                &["(", "x", ")", "space"]
            } else {
                &["(", ")", "left", "x", "right", "space"]
            };
            bench.press(keys[typed.len() % keys.len()]);
        } else if keystroke {
            bench.type_char("x");
        }
        let typed_at = Instant::now();
        bench.cx.run_until_parked();
        let parked_at = Instant::now();
        bench.draw(false);
        let off_thread = split_ms() - split_before;
        let took = ms(start.elapsed()) - off_thread;
        split_total += off_thread;
        if std::env::var("KANSO_BENCH_TRACE")
            .ok()
            .and_then(|t| t.parse::<f64>().ok())
            .is_some_and(|t| took > t)
        {
            println!(
                "    frame {frame:>3} t={:>5}ms key={keystroke} type {:.2} parked {:.2} draw {:.2}",
                now.as_millis(),
                ms(typed_at - start),
                ms(parked_at - typed_at),
                ms(parked_at.elapsed()),
            );
        }
        // Reading the texts is not part of the frame.
        if keystroke {
            typed.push((now, bench.text()));
        }
        let now_shown = bench.shown();
        let changed = now_shown != shown;
        if keystroke {
            key_frames.push(took);
        } else if changed {
            preview_frames.push(took);
        } else {
            other_frames.push(took);
        }
        if changed && std::env::var_os("KANSO_BENCH_TRACE").is_some() {
            println!(
                "    preview text changed at frame {frame} t={}ms",
                now.as_millis()
            );
        }
        if changed {
            shown = now_shown;
            updates.push(now);
            // Every keystroke whose text, or a later one, now shows.
            while caught_up < typed.len() {
                let newest = typed.iter().rposition(|(_, text)| *text == shown);
                match newest {
                    Some(ix) if ix >= caught_up => {
                        for (at, _) in &typed[caught_up..=ix] {
                            latencies.push(ms(now - *at));
                        }
                        caught_up = ix + 1;
                    }
                    _ => break,
                }
            }
        }
        bench.cx.advance_clock(FRAME);
        now += FRAME;
    }
    if let Some(dir) = std::env::var_os("KANSO_SNAPSHOT_DIR") {
        let image = bench
            .cx
            .capture_screenshot(bench.window)
            .expect("Metal rendering");
        let path = Path::new(&dir).join(format!("bench-{}k-{place}.png", bytes / 1024));
        image.save(&path).unwrap();
        println!("wrote {}", path.display());
    }
    let final_text = bench.text();
    assert_eq!(bench.shown(), final_text, "the preview caught up");
    let typing_end = typed.last().map(|(at, _)| *at).unwrap_or_default();
    let updates_while_typing = updates.iter().filter(|at| **at <= typing_end).count();
    let mut gaps: Vec<f64> = updates
        .windows(2)
        .filter(|w| w[1] <= typing_end + typing_interval())
        .map(|w| ms(w[1] - w[0]))
        .collect();
    let first = latencies.first().copied().unwrap_or(f64::NAN);
    println!(
        "{:>4} KB at {:<6}{}: preview updates while typing {:>2}/{KEYSTROKES}; \
         keystroke→preview (simulated) first {first:.0} ms, p50 {:.0} ms, max {:.0} ms; \
         max gap {:.0} ms",
        note.len() / 1024,
        place,
        match keys {
            Some(true) => " (keys, pairing on)",
            Some(false) => " (keys, pairing off)",
            None => "",
        },
        updates_while_typing,
        percentile(&mut latencies.clone(), 0.5),
        percentile(&mut latencies, 1.0),
        percentile(&mut gaps, 1.0),
    );
    println!("              splitting off the main thread: {split_total:.0} ms in all");
    println!(
        "              main-thread frame cost (wall): keystroke p50 {:.2} / p95 {:.2} / max {:.2} ms; \
         preview update p50 {:.2} / max {:.2} ms (n={}); idle p50 {:.2} ms",
        percentile(&mut key_frames.clone(), 0.5),
        percentile(&mut key_frames.clone(), 0.95),
        percentile(&mut key_frames, 1.0),
        percentile(&mut preview_frames.clone(), 0.5),
        percentile(&mut preview_frames.clone(), 1.0),
        preview_frames.len(),
        percentile(&mut other_frames, 0.5),
    );
}

/// Another member's edits to a shared note of about `bytes` open here, in
/// Split: each round the owner writes a sentence in the middle and both
/// sides sync with the in-memory server (timed apart, as the network
/// would be), then the editor takes the merge and draws a frame.
fn run_remote(bytes: usize) {
    use kanso_collab::Role;
    use kanso_collab::memory::MemoryCollab;

    use crate::collab_tests::{body_of, device, shared_body};

    const ROUNDS: usize = 20;
    let server = MemoryCollab::new();
    let (owner, member) = (device(&server, 1), device(&server, 2));
    let (share, note) = shared_body(&owner, &member, Role::Edit, &realistic_note(bytes));
    let dir = tempfile::tempdir().unwrap();
    let mut cx = crate::snapshots::context(None);
    cx.update(|cx| crate::collab::use_engine(member.engine.clone(), cx));
    let (window, workspace) = open_split(&mut cx, member.store.clone(), dir.path(), &note);
    let mut bench = Bench {
        cx,
        window,
        workspace,
        _dir: dir,
    };
    for _ in 0..4 {
        bench.cx.advance_clock(Duration::from_millis(500));
        bench.cx.run_until_parked();
        bench.draw(true);
    }
    let owner_note = owner.store.note_with_share(&share).unwrap().unwrap();
    let (mut network, mut merges, mut frames) = (Vec::new(), Vec::new(), Vec::new());
    for round in 0..ROUNDS {
        let mut body = owner.store.note(&owner_note).unwrap().unwrap().body;
        let middle = crate::collab_text::floor_boundary(&body, body.len() / 2);
        body.insert_str(middle, &format!("A remote sentence, number {round}. "));
        owner
            .store
            .put(
                DocType::Note,
                json!({"_id": owner_note, "body": body}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let start = Instant::now();
        owner.engine.sync_note(&share).unwrap();
        member.engine.sync_note(&share).unwrap();
        network.push(ms(start.elapsed()));
        let editor = bench
            .cx
            .update(|cx| bench.workspace.read(cx).editor.clone());
        let start = Instant::now();
        bench
            .cx
            .update_window(bench.window, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.collab_sync_for_test(window, cx))
            })
            .unwrap();
        merges.push(ms(start.elapsed()));
        let start = Instant::now();
        bench.draw(false);
        frames.push(ms(start.elapsed()));
        bench.cx.advance_clock(FRAME);
        bench.cx.run_until_parked();
    }
    assert_eq!(
        bench.text(),
        body_of(&owner, &share),
        "every edit merged in"
    );
    println!(
        "remote merge, {} KB: sync p50 {:.2} ms; merge into the editor p50 {:.2} / p95 {:.2} / max {:.2} ms; \
         next frame p50 {:.2} / max {:.2} ms (n={ROUNDS})",
        bytes / 1024,
        percentile(&mut network, 0.5),
        percentile(&mut merges.clone(), 0.5),
        percentile(&mut merges.clone(), 0.95),
        percentile(&mut merges, 1.0),
        percentile(&mut frames.clone(), 0.5),
        percentile(&mut frames, 1.0),
    );
}

#[test]
#[ignore = "a benchmark: run with --ignored --nocapture"]
fn remote_merge_bench() {
    let sizes = std::env::var("KANSO_BENCH_KB").map_or(vec![20, 100], |v| {
        v.split(',').map(|n| n.parse().unwrap()).collect()
    });
    for kb in sizes {
        run_remote(kb * 1024);
    }
}

#[test]
#[ignore = "a benchmark: run with --ignored --nocapture"]
fn latency_bench() {
    let sizes = std::env::var("KANSO_BENCH_KB").map_or(vec![20, 100], |v| {
        v.split(',').map(|n| n.parse().unwrap()).collect()
    });
    for bytes in sizes.into_iter().map(|kb: usize| kb * 1024) {
        for place in ["middle", "end"] {
            run(bytes, place, None);
        }
        run(bytes, "middle", Some(false));
        run(bytes, "middle", Some(true));
    }
}
