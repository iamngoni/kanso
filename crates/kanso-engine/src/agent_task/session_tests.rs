//! The summary a continued session starts from: capped in messages and in
//! bytes, the most recent kept, nothing cut in the middle of a character.

use kanso_server::Counts;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

use super::session::{
    Item, RoundState, SUMMARY_BYTES, SUMMARY_MESSAGES, ToolKind, ToolStatus, summary,
};

fn user(text: &str) -> Item {
    Item::User {
        text: text.to_owned(),
        at_ms: 0,
    }
}

fn agent(text: &str) -> Item {
    Item::Agent {
        text: text.to_owned(),
        at_ms: 0,
    }
}

fn item() -> impl Strategy<Value = Item> {
    prop_oneof![
        "\\PC{0,40}".prop_map(|text| user(&text)),
        "\\PC{0,40}".prop_map(|text| agent(&text)),
        "\\PC{0,20}".prop_map(|ms| Item::Thought { text: ms, ms: 1 }),
        "\\PC{0,20}".prop_map(|title| Item::Tool {
            id: "t".into(),
            title,
            kind: ToolKind::Read,
            status: ToolStatus::Completed,
        }),
        (0usize..4).prop_map(|edits| Item::Proposed {
            round: 1,
            counts: Counts {
                creates: 0,
                edits,
                moves: 0,
                merges: 0,
            },
            state: RoundState::Applied,
        }),
        "\\PC{0,20}".prop_map(|text| Item::Notice { text, at_ms: 0 }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: RngSeed::Fixed(0x0073_756d_6d61),
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// However long the old session was, the summary is within both caps.
    #[test]
    fn a_summary_is_capped_in_messages_and_bytes(
        items in prop::collection::vec(item(), 0..80),
        big in "\\PC{0,200}",
        copies in 0usize..300,
    ) {
        let mut items = items;
        // One very long message in the middle.
        items.insert(items.len() / 2, agent(&big.repeat(copies)));
        let text = summary(&items);
        prop_assert!(text.len() <= SUMMARY_BYTES);
        let said = text
            .lines()
            .filter(|line| line.starts_with("You: ") || line.starts_with("Agent: "))
            .count();
        // (The messages here are one line each.)
        prop_assert!(said <= SUMMARY_MESSAGES);
    }

    /// The most recent message is always in it, whole when it fits.
    #[test]
    fn a_summary_ends_with_the_latest_message(
        items in prop::collection::vec(item(), 0..30),
        last in "[a-z ]{1,60}",
    ) {
        let mut items = items;
        items.push(agent(&last));
        let text = summary(&items);
        prop_assert!(text.ends_with(&format!("Agent: {}", last.trim())), "{text}");
    }
}

#[test]
fn only_the_last_twenty_messages_go_in_oldest_first_with_tools_as_one_liners() {
    let mut items = Vec::new();
    for n in 0..30 {
        items.push(user(&format!("question {n}")));
        items.push(Item::Tool {
            id: format!("t{n}"),
            title: format!("Read note {n}"),
            kind: ToolKind::Read,
            status: if n == 29 {
                ToolStatus::Failed
            } else {
                ToolStatus::Completed
            },
        });
        items.push(agent(&format!("answer {n}")));
    }
    items.push(Item::Thought {
        text: "secret thought".into(),
        ms: 1,
    });
    let text = summary(&items);
    let messages = text
        .lines()
        .filter(|line| line.starts_with("You: ") || line.starts_with("Agent: "))
        .count();
    assert_eq!(messages, SUMMARY_MESSAGES);
    // Ten questions and ten answers: 20..30 are the last ten of each.
    assert!(text.starts_with("You: question 20\n"), "{text}");
    assert!(!text.contains("question 19"));
    assert!(text.contains("Tool: Read note 25\n"));
    assert!(text.ends_with("Agent: answer 29"));
    assert!(text.contains("Tool: Read note 29 (failed)"));
    assert!(!text.contains("secret thought"));
}

#[test]
fn a_message_over_the_cap_is_cut_on_a_character_boundary() {
    let text = summary(&[user(&"é".repeat(SUMMARY_BYTES))]);
    assert!(text.len() <= SUMMARY_BYTES);
    assert!(text.ends_with('…'));
    assert!(text.starts_with("You: é"));
    assert_eq!(summary(&[]), "");
}
