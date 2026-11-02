//! The AI parts of the brief and the review on the phone: finding more open
//! loops (proposals checked against the notes, kept only when ticked) and
//! the summaries. The model is a script that keeps what it was sent.

mod common;

use common::model::*;
use common::*;
use kanso_mobile::{AI_OFF, AiRun, KansoError, OpenLoopKind};

const SENTENCE: &str = "Sandra will bring the keys on Friday";

// no-prop-test: one scripted model reply, checked step by step.
#[test]
fn found_loops_are_proposed_checked_and_kept_only_when_chosen() {
    let reply = format!(
        "[{{\"note\": 1, \"line\": 0, \"text\": \"{SENTENCE}\", \"kind\": \"waiting\"}}, \
          {{\"note\": 1, \"line\": 0, \"text\": \"A sentence the note never said\", \"kind\": \"mine\"}}]"
    );
    let model = Scripted::new(&reply);
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Keys", &format!("{SENTENCE}.\n"));

    let scan = ai.find_open_loops(&AiRun::new()).unwrap();
    assert_eq!(scan.proposals.len(), 1, "{scan:?}");
    assert_eq!(scan.proposals[0].note_id, id);
    assert_eq!(scan.proposals[0].kind, OpenLoopKind::Waiting);
    assert_eq!(
        scan.dropped_text.as_deref(),
        Some("1 suggestion no longer matches its note")
    );
    // Nothing is saved until the person chooses.
    assert_eq!(phone.library.open_loops().unwrap().total, 0);

    assert_eq!(
        phone
            .library
            .approve_ai_loops(scan.proposals.clone())
            .unwrap(),
        1
    );
    let board = phone.library.open_loops().unwrap();
    assert_eq!(board.total, 1);
    assert!(board.items[0].from_ai);
    // Keeping the same one again adds nothing.
    assert_eq!(phone.library.approve_ai_loops(scan.proposals).unwrap(), 0);
}

// no-prop-test: AI off refuses before anything is sent.
#[test]
fn nothing_is_sent_while_ai_is_off() {
    let model = Scripted::new("[]");
    let phone = lone_phone();
    let ai = phone.library.ai_with_provider(maker_of(model.clone()));
    note(&phone.library, "Keys", &format!("{SENTENCE}.\n"));
    for result in [
        ai.find_open_loops(&AiRun::new()).map(|_| ()),
        ai.summarise_brief(&AiRun::new()).map(|_| ()),
        ai.summarise_review(None, &AiRun::new()).map(|_| ()),
    ] {
        assert_eq!(result.unwrap_err(), KansoError::invalid(AI_OFF));
    }
    assert!(model.everything_sent().is_empty());
}

// no-prop-test: one scripted summary with a citation the brief never held.
#[test]
fn summaries_warn_about_notes_that_were_not_given() {
    let model = Scripted::new("Start with [[Keys]]. Then [[Somewhere Else]].");
    let (phone, ai) = ai_phone(&model);
    note(&phone.library, "Keys", "I'll collect the keys today.\n");

    let brief = ai.summarise_brief(&AiRun::new()).unwrap();
    assert!(brief.text.contains("[[Keys]]"));
    assert_eq!(
        brief.warning.as_deref(),
        Some("This summary mentions notes that weren't in the brief.")
    );
    let review = ai.summarise_review(None, &AiRun::new()).unwrap();
    assert!(review.warning.is_some_and(|w| w.contains("review")));
    assert!(
        ai.summarise_review(Some("nope".into()), &AiRun::new())
            .is_err()
    );
}
