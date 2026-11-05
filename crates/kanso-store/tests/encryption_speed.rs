//! The speed of an encrypted library against a plain one (issue #243 step 9).
//! Slow, so it only runs on request:
//!
//! `cargo test -p kanso-store --release --test encryption_speed -- --ignored --nocapture`
//!
//! `KANSO_SPEED_MB` sets the library's size on disk (default 150).

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
}
