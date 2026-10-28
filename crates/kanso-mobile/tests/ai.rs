//! The AI features at the facade: settings and the key's place, Ask your
//! notes (nothing matches, citations, tools, stop, a cut-off stream, a
//! conversation that restarts), the inline assistant and the summary, and
//! what they refuse. The model is a script that keeps what it was sent.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_mobile::{
    AI_OFF, AiRun, AskEvent, AssistEvent, CITATION_PREFIX, KEPT_OUT, KansoError, NOTHING_MATCHES,
};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

fn is_end(event: &AskEvent) -> bool {
    matches!(
        event,
        AskEvent::Done { .. }
            | AskEvent::NothingMatches { .. }
            | AskEvent::CutOff { .. }
            | AskEvent::Failed { .. }
            | AskEvent::Stopped
    )
}

fn is_assist_end(event: &AssistEvent) -> bool {
    matches!(
        event,
        AssistEvent::Done { .. }
            | AssistEvent::CutOff { .. }
            | AssistEvent::Failed { .. }
            | AssistEvent::Stopped
    )
}

fn invalid(message: &str) -> KansoError {
    KansoError::invalid(message)
}

/// Asks `question` and returns everything heard.
fn ask(
    ai: &kanso_mobile::AiHandle,
    conversation: &kanso_mobile::AskConversation,
    q: &str,
) -> Vec<AskEvent> {
    let _ = ai;
    let (sink, rx) = listen();
    conversation.ask(q.to_owned(), sink).unwrap();
    until(&rx, is_end)
}

// no-prop-test: settings and the key's place, each checked once.
#[test]
fn settings_round_trip_and_the_key_lives_only_in_the_secret_store() {
    let phone = lone_phone();
    let ai = phone.library.ai();
    let first = ai.settings().unwrap();
    assert!(!first.enabled, "AI is off until it is turned on");
    assert_eq!(first.provider, "local");
    assert!(!first.has_key);
    assert!(!first.is_remote);

    let saved = ai
        .set_settings(kanso_mobile::AiSettingsInput {
            enabled: true,
            provider: "openai".into(),
            base_url: String::new(),
            model: String::new(),
            region: "ignored".into(),
        })
        .unwrap();
    assert!(saved.enabled);
    assert_eq!(saved.base_url, "https://api.openai.com/v1");
    assert_eq!(saved.model, "gpt-4o-mini");
    assert_eq!(saved.region, "", "the region is Bedrock's alone");
    assert_eq!(saved.host, "api.openai.com");
    assert!(saved.is_remote);
    assert_eq!(ai.settings().unwrap(), saved);

    ai.set_key("openai".into(), "  sk-TOPSECRET-123  ".into())
        .unwrap();
    assert!(ai.settings().unwrap().has_key);
    let file = std::fs::read_to_string(phone.dir.path().join("ai.json")).unwrap();
    assert!(!file.contains("TOPSECRET"), "{file}");
    assert!(!format!("{:?}", ai.settings().unwrap()).contains("TOPSECRET"));
    use kanso_mobile::Secrets;
    assert_eq!(
        phone
            .secrets
            .get("ai-key:openai")
            .unwrap()
            .map(|key| key.to_string()),
        Some("sk-TOPSECRET-123".to_owned())
    );
    ai.set_key("openai".into(), String::new()).unwrap();
    assert!(!ai.settings().unwrap().has_key);
    assert!(phone.secrets.get("ai-key:openai").unwrap().is_none());

    assert!(ai.set_key("nope".into(), "k".into()).is_err());
    // A file from a build that knows other things still reads.
    std::fs::write(
        phone.dir.path().join("ai.json"),
        "{\"enabled\":true,\"x\":1}",
    )
    .unwrap();
    assert!(ai.settings().unwrap().enabled);
}

// no-prop-test: the fixed list of providers and actions.
#[test]
fn the_providers_and_actions_are_the_desktops_words() {
    let phone = lone_phone();
    let ai = phone.library.ai();
    let providers = ai.providers();
    assert_eq!(providers.len(), 13);
    let local = &providers[0];
    assert_eq!(local.key, "local");
    assert!(local.description.contains("LM Studio"), "{local:?}");
    assert!(!local.needs_key);
    assert!(
        providers
            .iter()
            .find(|p| p.key == "openai")
            .unwrap()
            .needs_key
    );
    let labels: Vec<_> = ai.actions().into_iter().map(|a| a.label).collect();
    assert_eq!(
        labels,
        [
            "Improve Writing",
            "Make Shorter",
            "Fix Spelling and Grammar",
            "Continue Writing",
            "Summarize Selection"
        ]
    );
    let line = ai.privacy_line("this phone".into()).unwrap();
    assert!(
        line.contains("from this phone to the model at localhost:11434"),
        "{line}"
    );
    assert!(!line.contains("Mac"));
}

// no-prop-test: AI off is one state; each feature refuses in it.
#[test]
fn nothing_is_asked_while_ai_is_off() {
    let model = Scripted::new("never");
    let phone = lone_phone();
    let id = note(&phone.library, "Pricing", "The plan costs 40 dollars.");
    let ai = phone.library.ai_with_provider(maker_of(model.clone()));
    let (sink, _rx) = listen();
    assert_eq!(
        ai.conversation().ask("pricing?".into(), sink),
        Err(invalid(AI_OFF))
    );
    let (sink, _rx) = listen();
    assert_eq!(
        ai.assist(
            &id,
            "improve",
            String::new(),
            "text".into(),
            &AiRun::new(),
            sink
        )
        .err(),
        Some(invalid(AI_OFF))
    );
    let (sink, _rx) = listen();
    assert_eq!(
        ai.summarize(&id, &AiRun::new(), sink).err(),
        Some(invalid(AI_OFF))
    );
    assert_eq!(model.request_count(), 0);
}

// no-prop-test: the empty case is one fixed question.
#[test]
fn a_question_nothing_matches_calls_no_model() {
    let model = Scripted::new("never");
    let (phone, ai) = ai_phone(&model);
    note(&phone.library, "Pricing", "The plan costs 40 dollars.");
    let events = ask(&ai, &ai.conversation(), "quantum chromodynamics lattice");
    assert_eq!(
        events.last(),
        Some(&AskEvent::NothingMatches {
            message: NOTHING_MATCHES.into()
        })
    );
    assert_eq!(model.request_count(), 0);
}

// no-prop-test: a fixed note and a fixed answer.
#[test]
fn a_citation_becomes_a_link_that_names_the_note() {
    let model = Scripted::new(
        "A yearly plan, see [[Pricing]] and ![x](https://evil.test/p.png) <b>hi</b>.",
    );
    let (phone, ai) = ai_phone(&model);
    let id = note(
        &phone.library,
        "Pricing",
        "We decided the yearly plan costs 40 dollars.",
    );
    let events = ask(
        &ai,
        &ai.conversation(),
        "What did we decide about the yearly plan pricing?",
    );
    assert_eq!(events.first(), Some(&AskEvent::Searching));
    assert!(events.iter().any(|e| matches!(e, AskEvent::Answer { .. })));
    let Some(AskEvent::Done { markdown, notes }) = events.last() else {
        panic!("{events:?}");
    };
    assert!(
        markdown.contains(&format!("[Pricing]({CITATION_PREFIX}{id})")),
        "{markdown}"
    );
    assert!(!markdown.contains("evil.test"), "{markdown}");
    assert!(!markdown.contains("<b>"), "{markdown}");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, id);
    assert_eq!(notes[0].title, "Pricing");
    // The note the answer names is the note the link opens.
    assert_eq!(phone.library.note(&id).unwrap().title, "Pricing");
}

// no-prop-test: one scripted tool session.
#[test]
fn a_model_can_search_and_read_through_its_tools_and_those_notes_are_listed() {
    let phone = lone_phone();
    let id = note(&phone.library, "Garden", "Tomatoes need staking in July.");
    let model = Scripted::with(
        "Stake them, per [[Garden]].",
        vec![
            ("search_notes", json!({"query": "tomatoes"}).to_string()),
            ("read_note", json!({"id": id}).to_string()),
        ],
        Ending::Complete,
    );
    let ai = ai_on(&phone.library, &model);
    note(
        &phone.library,
        "Pricing",
        "The yearly plan costs 40 dollars.",
    );
    let events = ask(&ai, &ai.conversation(), "yearly plan pricing");
    let Some(AskEvent::Done { notes, .. }) = events.last() else {
        panic!("{events:?}");
    };
    assert!(notes.iter().any(|n| n.id == id), "{notes:?}");
    assert!(notes.iter().any(|n| n.title == "Pricing"), "{notes:?}");
    assert!(model.everything_sent().contains("Tomatoes need staking"));
}

// no-prop-test: stopping is one event ordering.
#[test]
fn stop_cancels_the_answer_and_ends_the_stream() {
    let model = Scripted::with(
        "Words that never finish",
        vec![("search_notes", json!({"query": "pricing"}).to_string())],
        Ending::Hangs,
    );
    let (phone, ai) = ai_phone(&model);
    note(
        &phone.library,
        "Pricing",
        "The yearly plan costs 40 dollars.",
    );
    let conversation = ai.conversation();
    let (sink, rx) = listen();
    conversation
        .ask("yearly plan pricing".into(), sink)
        .unwrap();
    let mut heard = Vec::new();
    // The tool ran (the second list of notes), and the answer then hangs.
    while heard
        .iter()
        .filter(|e| matches!(e, AskEvent::Used { .. }))
        .count()
        < 2
    {
        heard.push(
            rx.recv_timeout(std::time::Duration::from_secs(20))
                .unwrap_or_else(|_| panic!("heard only {heard:?}")),
        );
    }
    conversation.stop();
    heard.extend(until(&rx, is_end));
    assert_eq!(heard.last(), Some(&AskEvent::Stopped));
    assert_eq!(conversation.turn_count(), 0, "a stopped answer is not kept");
}

// no-prop-test: a stream with no end marker, once.
#[test]
fn a_stream_that_stops_short_is_cut_off() {
    let model = Scripted::with(
        "Half an ans",
        vec![("search_notes", json!({"query": "pricing"}).to_string())],
        Ending::CutOff,
    );
    let (phone, ai) = ai_phone(&model);
    note(
        &phone.library,
        "Pricing",
        "The yearly plan costs 40 dollars.",
    );
    let conversation = ai.conversation();
    let events = ask(&ai, &conversation, "yearly plan pricing");
    assert!(
        matches!(events.last(), Some(AskEvent::CutOff { .. })),
        "{events:?}"
    );
    assert_eq!(conversation.turn_count(), 0);
}

// no-prop-test: two asks and a settings change.
#[test]
fn a_follow_up_carries_the_conversation_and_another_model_starts_over() {
    let model = Scripted::new("The plan costs 40 dollars, see [[Pricing]].");
    let (phone, ai) = ai_phone(&model);
    note(
        &phone.library,
        "Pricing",
        "The yearly plan costs 40 dollars.",
    );
    let conversation = ai.conversation();
    ask(&ai, &conversation, "yearly plan pricing");
    assert_eq!(conversation.turn_count(), 1);
    ask(&ai, &conversation, "and the yearly plan pricing again");
    assert_eq!(conversation.turn_count(), 2);
    assert!(
        model
            .everything_sent()
            .matches("yearly plan pricing")
            .count()
            >= 2
    );

    let before = model.request_count();
    let mut other = custom_settings(true);
    other.model = "another-model".into();
    ai.set_settings(other).unwrap();
    ask(&ai, &conversation, "yearly plan pricing once more");
    assert_eq!(
        conversation.turn_count(),
        1,
        "the conversation started over"
    );
    let last = model.sent.lock().unwrap()[before..].join("\n");
    assert!(
        !last.contains("The plan costs 40 dollars, see"),
        "the earlier answer was resent"
    );
    conversation.clear();
    assert_eq!(conversation.turn_count(), 0);
}

// no-prop-test: a note locked after it was cited.
#[test]
fn an_exchange_whose_note_gets_locked_is_forgotten() {
    let model = Scripted::new("The plan costs 40 dollars, see [[Pricing]].");
    let (phone, ai) = ai_phone(&model);
    let id = note(
        &phone.library,
        "Pricing",
        "The yearly plan costs 40 dollars.",
    );
    let conversation = ai.conversation();
    ask(&ai, &conversation, "yearly plan pricing");
    assert_eq!(conversation.turn_count(), 1);
    phone
        .library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    // The next question (about something else) drops the exchange first.
    note(&phone.library, "Garden", "Tomatoes need staking in July.");
    ask(&ai, &conversation, "tomatoes staking");
    assert_eq!(conversation.turn_count(), 1);
    let sent = model.sent.lock().unwrap().last().cloned().unwrap();
    assert!(!sent.contains("The plan costs 40 dollars, see"), "{sent}");
}

// no-prop-test: each way a note is kept out, once.
#[test]
fn the_assistant_and_the_summary_refuse_a_note_kept_out_of_ai() {
    let model = Scripted::new("never");
    let (phone, ai) = ai_phone(&model);
    let locks = phone.library.locks();
    let sealed = note(&phone.library, "Sealed", "SEALED-BODY text");
    locks.lock_note(&sealed, LOCK_PASSWORD.into()).unwrap();
    // The lock is open for this session; the note is still kept out.
    assert!(locks.is_unlocked().unwrap());
    assert!(locks.is_note_unlocked(&sealed).unwrap());
    let trashed = note(&phone.library, "Trashed", "TRASHED-BODY text");
    phone.library.trash(&trashed).unwrap();
    let excluded = note(&phone.library, "Excluded", "EXCLUDED-BODY text");
    let store = Store::open(phone.path()).unwrap();
    let tag_id = tag(&store, "AI-Exclude");
    store
        .put(
            DocType::Note,
            json!({"_id": excluded, "tags": [tag_id]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    for id in [&sealed, &trashed, &excluded] {
        assert!(!ai.note_allowed(id).unwrap(), "{id}");
        let (sink, _rx) = listen();
        assert_eq!(
            ai.assist(
                id,
                "improve",
                String::new(),
                "text".into(),
                &AiRun::new(),
                sink
            )
            .err(),
            Some(invalid(KEPT_OUT))
        );
        let (sink, _rx) = listen();
        assert_eq!(
            ai.summarize(id, &AiRun::new(), sink).err(),
            Some(invalid(KEPT_OUT))
        );
    }
    assert_eq!(model.request_count(), 0);
    assert_eq!(KEPT_OUT, "This note is kept out of AI.");
}

// no-prop-test: one fixed rewrite and one fixed summary.
#[test]
fn the_assistant_rewrites_a_selection_and_the_summary_is_made_safe() {
    let model = Scripted::new("```\nWe ship on Friday.\n```");
    let (phone, ai) = ai_phone(&model);
    let id = note(
        &phone.library,
        "Plan",
        "Earlier words.\n\nWe ship on friday.",
    );
    let (sink, rx) = listen();
    let run = AiRun::new();
    ai.assist(
        &id,
        "improve",
        "Earlier words.\n\n".into(),
        "We ship on friday.".into(),
        &run,
        sink,
    )
    .unwrap();
    let heard = until(&rx, is_assist_end);
    assert_eq!(
        heard.last(),
        Some(&AssistEvent::Done {
            text: "We ship on Friday.".into()
        })
    );
    run.stop();
    assert!(model.everything_sent().contains("We ship on friday."));
    // Nothing selected: only Continue goes on, from the paragraph before.
    let (sink, _rx) = listen();
    assert!(
        ai.assist(&id, "improve", "x".into(), "  ".into(), &AiRun::new(), sink)
            .is_err()
    );
    let (sink, rx) = listen();
    ai.assist(
        &id,
        "continue",
        "Intro.\n\nLast paragraph here.".into(),
        String::new(),
        &AiRun::new(),
        sink,
    )
    .unwrap();
    until(&rx, is_assist_end);
    assert!(model.everything_sent().contains("Last paragraph here."));

    let model =
        Scripted::new("- point <img src=\"http://evil.test/x.png\"> ![i](http://evil.test/y.png)");
    let ai = ai_on(&phone.library, &model);
    let (sink, rx) = listen();
    ai.summarize(&id, &AiRun::new(), sink).unwrap();
    let Some(AssistEvent::Done { text }) = until(&rx, is_assist_end).pop() else {
        panic!("no summary");
    };
    assert!(text.contains("point"));
    assert!(!text.contains("<img"), "{text}");
    assert!(!text.contains("evil.test/y.png"), "{text}");
    assert!(model.everything_sent().contains("We ship on friday."));
}

/// A server on an ephemeral port that answers one streamed request.
fn loopback(answer: &'static str) -> String {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(1) {
            let mut stream = stream.unwrap();
            let mut seen = Vec::new();
            let mut buf = [0u8; 4096];
            while !seen.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                seen.extend_from_slice(&buf[..n]);
            }
            let head = String::from_utf8_lossy(&seen).to_lowercase();
            let length: usize = head
                .split("content-length: ")
                .nth(1)
                .and_then(|rest| rest.split("\r\n").next())
                .and_then(|n| n.trim().parse().ok())
                .unwrap_or(0);
            let have = seen.len() - seen.windows(4).position(|w| w == b"\r\n\r\n").unwrap() - 4;
            let mut left = length.saturating_sub(have);
            while left > 0 {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                left = left.saturating_sub(n);
            }
            let body = format!(
                "data: {}\n\ndata: [DONE]\n\n",
                json!({"choices":[{"index":0,"delta":{"content": answer}}]})
            );
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\ncontent-length: {}\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    format!("http://127.0.0.1:{port}/v1")
}

// no-prop-test: one probe against a loopback server.
#[test]
fn the_test_button_asks_the_server_the_settings_name() {
    let phone = lone_phone();
    let ai = phone.library.ai();
    let mut settings = custom_settings(true);
    settings.base_url = loopback("OK");
    ai.set_settings(settings).unwrap();
    assert_eq!(ai.test().unwrap(), "OK");
    // A provider that needs a key says so, in plain words.
    ai.set_settings(kanso_mobile::AiSettingsInput {
        provider: "openai".into(),
        ..custom_settings(true)
    })
    .unwrap();
    let Err(KansoError::Invalid { message }) = ai.test() else {
        panic!("a missing key must be refused");
    };
    assert!(message.contains("API key"), "{message}");
}
