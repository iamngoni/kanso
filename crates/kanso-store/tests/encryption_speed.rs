//! The speed of an encrypted library against a plain one (issue #243 step 9).
//! Slow, so it only runs on request:
//!
//! `cargo test -p kanso-store --release --test encryption_speed -- --ignored --nocapture`
//!
//! `KANSO_SPEED_MB` sets the library's size on disk (default 150).
//!
//! The run fails when encryption costs more than the budget in `within`.
//! CI runs it on master (`.github/workflows/ci.yml`, job `encryption-speed`).

use std::path::Path;
use std::time::{Duration, Instant};

use kanso_core::{DocType, Query};
use kanso_store::{LibraryKey, Origin, Put, SearchParams, Sort, Store};
use serde_json::json;

fn size_mb(db: &Path) -> f64 {
    ["", "-wal"]
        .iter()
        .map(|suffix| {
            let mut name = db.as_os_str().to_owned();
            name.push(suffix);
            std::fs::metadata(name).map(|m| m.len()).unwrap_or(0)
        })
        .sum::<u64>() as f64
        / 1_048_576.0
}

fn vocabulary() -> Vec<String> {
    let mut rng = fastrand::Rng::with_seed(7);
    (0..4000)
        .map(|_| {
            let len = rng.usize(3..10);
            (0..len).map(|_| rng.lowercase()).collect::<String>()
        })
        .collect()
}

fn body(rng: &mut fastrand::Rng, words: &[String], n: usize) -> String {
    let mut text = String::with_capacity(n * 7);
    for i in 0..n {
        if i > 0 {
            text.push(if i % 14 == 0 { '\n' } else { ' ' });
        }
        text.push_str(&words[rng.usize(..words.len())]);
    }
    text
}

fn median(mut runs: Vec<Duration>) -> Duration {
    runs.sort();
    runs[runs.len() / 2]
}

fn search(store: &Store, term: &str) -> usize {
    store
        .search(&SearchParams {
            query: Query::parse(term),
            sort: Sort::Title,
            descending: false,
            limit: Some(50),
            ..SearchParams::default()
        })
        .unwrap()
        .len()
}

/// How much slower than plain an encrypted step may be: five times, plus
/// 50 ms of slack for steps too short to time. Measured on 150 MB: opens and
/// searches run 0.9x to 2x, a save and a backup (which ciphers every page)
/// about 3.6x. The budget catches a real regression (no page cache, a key
/// derived for every statement) without failing on a noisy runner.
const SLOWER_BY: f64 = 5.0;
const SLACK_MS: f64 = 50.0;

fn within(plain_ms: f64, keyed_ms: f64) -> bool {
    keyed_ms <= plain_ms * SLOWER_BY + SLACK_MS
}

struct Numbers {
    build: Duration,
    size_mb: f64,
    open: Duration,
    word_search: Duration,
    fragment_search: Duration,
    put: Duration,
    backup: Duration,
}

fn measure(label: &str, key: Option<&LibraryKey>, target_mb: f64) -> Numbers {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kanso.db");
    let words = vocabulary();
    let mut rng = fastrand::Rng::with_seed(11);

    let started = Instant::now();
    let store = Store::open_with_key(&db, key).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Big"}),
            Put::Upsert,
            Origin::Import,
        )
        .unwrap()
        .id;
    let mut count = 0;
    while size_mb(&db) < target_mb {
        for _ in 0..200 {
            let title = body(&mut rng, &words, 5);
            let text = body(&mut rng, &words, 700);
            store
                .put(
                    DocType::Note,
                    json!({"bookId": book, "title": title, "body": text}),
                    Put::Upsert,
                    Origin::Import,
                )
                .unwrap();
            count += 1;
        }
    }
    drop(store);
    let build = started.elapsed();

    let opened = Instant::now();
    let store = Store::open_with_key(&db, key).unwrap();
    let open = opened.elapsed();
    let size = size_mb(&db);

    let probe_word = words[100].clone();
    let probe_fragment = words[200][1..4.min(words[200].len())].to_owned();
    let word_search = median(
        (0..9)
            .map(|_| {
                let t = Instant::now();
                search(&store, &probe_word);
                t.elapsed()
            })
            .collect(),
    );
    let fragment_search = median(
        (0..9)
            .map(|_| {
                let t = Instant::now();
                search(&store, &probe_fragment);
                t.elapsed()
            })
            .collect(),
    );
    let put = {
        let t = Instant::now();
        for _ in 0..100 {
            store
                .put(
                    DocType::Note,
                    json!({"bookId": book, "title": "x", "body": body(&mut rng, &words, 200)}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        t.elapsed() / 100
    };
    let backup = {
        let t = Instant::now();
        store.backup_to(&dir.path().join("copy.db")).unwrap();
        t.elapsed()
    };
    eprintln!("{label}: {count} notes");
    Numbers {
        build,
        size_mb: size,
        open,
        word_search,
        fragment_search,
        put,
        backup,
    }
}

#[test]
#[ignore = "builds a library of KANSO_SPEED_MB (150) on disk; run with --release"]
fn a_150_mb_library_is_about_as_quick_encrypted() {
    let target: f64 = std::env::var("KANSO_SPEED_MB")
        .ok()
        .and_then(|mb| mb.parse().ok())
        .unwrap_or(150.0);
    let plain = measure("plain", None, target);
    let key = LibraryKey::generate();
    let keyed = measure("encrypted", Some(&key), target);
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let row = |name: &str, plain: f64, keyed: f64, unit: &str| {
        eprintln!(
            "{name:<22}{plain:>12.1}{keyed:>12.1}{:>9.2}x  {unit}",
            keyed / plain
        );
    };
    eprintln!("{:<22}{:>12}{:>12}", "", "plain", "encrypted");
    row("size on disk", plain.size_mb, keyed.size_mb, "MB");
    row(
        "build",
        plain.build.as_secs_f64(),
        keyed.build.as_secs_f64(),
        "s",
    );
    row("open (cold)", ms(plain.open), ms(keyed.open), "ms");
    row(
        "word search",
        ms(plain.word_search),
        ms(keyed.word_search),
        "ms",
    );
    row(
        "fragment search",
        ms(plain.fragment_search),
        ms(keyed.fragment_search),
        "ms",
    );
    row("one save", ms(plain.put), ms(keyed.put), "ms");
    row("backup", ms(plain.backup), ms(keyed.backup), "ms");
    let over: Vec<&str> = [
        ("open (cold)", plain.open, keyed.open),
        ("word search", plain.word_search, keyed.word_search),
        (
            "fragment search",
            plain.fragment_search,
            keyed.fragment_search,
        ),
        ("one save", plain.put, keyed.put),
        ("backup", plain.backup, keyed.backup),
    ]
    .into_iter()
    .filter(|(_, plain, keyed)| !within(ms(*plain), ms(*keyed)))
    .map(|(name, ..)| name)
    .collect();
    assert!(
        over.is_empty(),
        "encryption is more than {SLOWER_BY}x slower (plus {SLACK_MS} ms) for: {}. \
         Check that the page cache and the key setup are not repeated per statement.",
        over.join(", ")
    );
}

mod budget {
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngSeed};

    use super::{SLACK_MS, SLOWER_BY, within};

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: None,
            ..Config::default()
        })]

        /// Equal times, or a faster encrypted step, are always within budget.
        #[test]
        fn prop_no_slower_is_always_within(plain in 0.0f64..1e6, gain in 0.0f64..1e6) {
            prop_assert!(within(plain, (plain - gain).max(0.0)));
            prop_assert!(within(plain, plain));
        }

        /// A step over the budget is refused; one at the edge is not.
        #[test]
        fn prop_the_budget_is_the_stated_ratio_plus_slack(plain in 0.0f64..1e6, extra in 1.0f64..1e6) {
            let edge = plain * SLOWER_BY + SLACK_MS;
            prop_assert!(!within(plain, edge + extra));
            prop_assert!(within(plain, edge));
        }

        /// A slower plain step never makes the same encrypted time fail.
        #[test]
        fn prop_slower_plain_loosens_the_budget(
            plain in 0.0f64..1e6, more in 0.0f64..1e6, keyed in 0.0f64..1e7,
        ) {
            prop_assert!(!within(plain, keyed) || within(plain + more, keyed));
        }
    }
}
