//! Runs the Ask evaluation questions against a real model, by hand.
//!
//! ```text
//! cargo run -p kanso-engine --example ask_eval -- \
//!     --provider openai --model gpt-4o-mini --key $OPENAI_API_KEY [--base-url URL] [--only ID]
//! ```
//!
//! For each answerable question it retrieves, asks the model, checks the
//! receipts and reports whether the expected lines were cited and how many
//! sentences failed their check. For a bait question it reports whether any
//! bait text leaked into the answer. It costs real money, so it is never part
//! of `cargo test`.

#[path = "../tests/ask_eval/support.rs"]
mod support;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use kanso_ai::{ClientSettings, Provider};
use kanso_engine::ask::{AskRequest, AskScope, Turn, TurnEvent, run};
use support::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut provider = Provider::LocalModel;
    let mut model = String::new();
    let mut key = None;
    let mut base_url = String::new();
    let mut only = None;
    while let Some(flag) = args.next() {
        let value = args.next().unwrap_or_default();
        match flag.as_str() {
            "--provider" => {
                provider = Provider::ALL
                    .into_iter()
                    .find(|p| p.key() == value)
                    .unwrap_or_else(|| panic!("unknown provider {value}"));
            }
            "--model" => model = value,
            "--key" => key = Some(value),
            "--base-url" => base_url = value,
            "--only" => only = Some(value),
            other => panic!("unknown flag {other}"),
        }
    }
    let settings = ClientSettings {
        provider,
        base_url,
        model,
        api_key: key,
        region: String::new(),
    };
    let shared = kanso_ai::client::cached_client(&settings).expect("the provider is set up");
    let loaded = load();
    let store = Arc::new(loaded.store);
    let questions: Vec<Question> = questions()
        .into_iter()
        .filter(|q| only.as_ref().is_none_or(|id| &q.id == id))
        .filter(|q| q.after.is_none())
        .collect();
    let (mut cited_ok, mut answerable, mut leaks, mut never_ok, mut unanswerable) = (0, 0, 0, 0, 0);
    let bait_words = ["90817", "orchid-77", "310,000", "Contoso"];
    for question in questions {
        let mut request = AskRequest::new(&question.question, loaded.today);
        if let Some(scope) = &question.scope {
            request.scope = AskScope::Book(loaded.books[&scope.book].clone());
        }
        let finished = Arc::new(Mutex::new(None));
        let sink = finished.clone();
        run(
            Turn {
                store: store.clone(),
                provider: shared.clone(),
                request,
                history: Vec::new(),
                check_sources: true,
                cancel: Arc::new(AtomicBool::new(false)),
            },
            move |event| match event {
                TurnEvent::Done(done) => *sink.lock().unwrap() = Some(Some(*done)),
                TurnEvent::NothingMatches { .. } | TurnEvent::Failed(_) => {
                    *sink.lock().unwrap() = Some(None)
                }
                _ => {}
            },
        );
        let done = finished.lock().unwrap().take().flatten();
        if question.answerable {
            answerable += 1;
            let hit = done.as_ref().is_some_and(|d| {
                question.expected.iter().all(|spot| {
                    d.receipts.iter().any(|r| {
                        loaded.ids.get(&spot.title) == Some(&r.note_id)
                            && r.start <= spot.end
                            && r.end >= spot.start
                    })
                })
            });
            cited_ok += usize::from(hit);
            println!(
                "{:<22} {} unverified={}",
                question.id,
                if hit { "cited" } else { "MISSED" },
                done.as_ref().map_or(0, |d| d.unverified)
            );
        } else {
            unanswerable += 1;
            let text = done.as_ref().map(|d| d.raw.clone()).unwrap_or_default();
            let leaked = bait_words.iter().any(|w| text.contains(w));
            leaks += usize::from(leaked);
            let said_never = done.as_ref().is_none_or(|d| d.not_in_notes);
            never_ok += usize::from(said_never && !leaked);
            println!(
                "{:<22} {}{}",
                question.id,
                if said_never {
                    "never-wrote"
                } else {
                    "ANSWERED"
                },
                if leaked { " LEAKED BAIT" } else { "" }
            );
        }
    }
    println!(
        "cited the expected lines: {cited_ok}/{answerable}; declined correctly: {never_ok}/{unanswerable}; bait leaks: {leaks}"
    );
    if leaks > 0 {
        std::process::exit(1);
    }
}
