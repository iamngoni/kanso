//! A transcript as a session view draws it (issue #375): the items in order,
//! with runs of tool calls folded into one line. A model file: values and the
//! words for them, nothing that draws.

use super::session::{Item, PermissionAnswer, RoundState, ToolKind, ToolStatus};

/// Tool calls in a row, this many or more, fold into one line.
pub const FOLD_AT: usize = 3;

/// One tool call, as a row shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolLine {
    pub id: String,
    pub title: String,
    pub kind: ToolKind,
    pub status: ToolStatus,
}

/// What a proposal card offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Offer {
    /// Review, before anything is applied.
    Review,
    /// Applied; Undo, when it can still be taken back.
    Undo,
    /// Nothing to do: the card only says what happened.
    Nothing,
}

/// One thing in the view, top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// The person's goal or follow-up.
    You {
        text: String,
    },
    /// The agent's words, as Markdown.
    Agent {
        text: String,
    },
    /// Collapsed `Thought for 12s`; opens to the text.
    Thought {
        label: String,
        text: String,
    },
    Tool(ToolLine),
    /// Three or more tool calls in a row: `Used 6 tools`.
    Tools {
        label: String,
        tools: Vec<ToolLine>,
    },
    /// What the person answered when the agent asked for a tool.
    Permission {
        text: String,
    },
    /// A round's proposal.
    Proposal {
        round: u32,
        label: String,
        state: RoundState,
        offer: Offer,
    },
    /// Muted: `Stopped`.
    Notice {
        text: String,
    },
}

/// `Thought for 12s`, never less than a second.
pub fn thought_label(ms: u64) -> String {
    format!("Thought for {}s", (ms / 1000).max(1))
}

/// `Used 6 tools`.
pub fn tools_label(count: usize) -> String {
    format!("Used {count} tools")
}

/// What the transcript records of a permission answer.
pub fn permission_text(answer: PermissionAnswer, title: &str) -> String {
    let lead = match answer {
        PermissionAnswer::AllowedOnce => "Allowed once",
        PermissionAnswer::AllowedForSession => "Allowed for this session",
        PermissionAnswer::AllowedAlways => "Always allowed",
        PermissionAnswer::Rejected => "Rejected",
    };
    format!("{lead}: {title}")
}

/// A round's card: `Round 2 · 3 new notes, 1 edit`, or just the counts for
/// the first round.
pub fn proposal_label(round: u32, state: RoundState, changes: usize, counts: &str) -> String {
    let what = match state {
        RoundState::ToReview => counts.to_owned(),
        RoundState::Applied => format!(
            "Applied {changes} change{}",
            if changes == 1 { "" } else { "s" }
        ),
        RoundState::Undone => "Undone".to_owned(),
        RoundState::Discarded => "Discarded".to_owned(),
        RoundState::Lost => "No longer available".to_owned(),
    };
    if round > 1 {
        format!("Round {round} · {what}")
    } else {
        what
    }
}

/// The items as blocks. `undoable_round` is the round whose Undo is still on
/// offer, if any.
pub fn fold(items: &[Item], undoable_round: Option<u32>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut run: Vec<ToolLine> = Vec::new();
    let flush = |run: &mut Vec<ToolLine>, blocks: &mut Vec<Block>| {
        if run.len() >= FOLD_AT {
            blocks.push(Block::Tools {
                label: tools_label(run.len()),
                tools: std::mem::take(run),
            });
        } else {
            blocks.extend(run.drain(..).map(Block::Tool));
        }
    };
    for item in items {
        let block = match item {
            Item::Tool {
                id,
                title,
                kind,
                status,
            } => {
                run.push(ToolLine {
                    id: id.clone(),
                    title: title.clone(),
                    kind: *kind,
                    status: *status,
                });
                continue;
            }
            Item::User { text, .. } => Block::You { text: text.clone() },
            Item::Agent { text, .. } => Block::Agent { text: text.clone() },
            Item::Thought { text, ms } => Block::Thought {
                label: thought_label(*ms),
                text: text.clone(),
            },
            Item::Permission { title, answer, .. } => Block::Permission {
                text: permission_text(*answer, title),
            },
            Item::Proposed {
                round,
                counts,
                state,
            } => Block::Proposal {
                round: *round,
                label: proposal_label(*round, *state, counts.total(), &counts.describe()),
                state: *state,
                offer: match state {
                    RoundState::ToReview => Offer::Review,
                    RoundState::Applied if undoable_round == Some(*round) => Offer::Undo,
                    _ => Offer::Nothing,
                },
            },
            Item::Notice { text, .. } => Block::Notice { text: text.clone() },
        };
        flush(&mut run, &mut blocks);
        blocks.push(block);
    }
    flush(&mut run, &mut blocks);
    blocks
}

impl Block {
    /// Whether any tool in the block is still going.
    pub fn working(&self) -> bool {
        let going =
            |tool: &ToolLine| matches!(tool.status, ToolStatus::Pending | ToolStatus::InProgress);
        match self {
            Block::Tool(tool) => going(tool),
            Block::Tools { tools, .. } => tools.iter().any(going),
            _ => false,
        }
    }

    /// Whether any tool in the block failed.
    pub fn failed(&self) -> bool {
        match self {
            Block::Tool(tool) => tool.status == ToolStatus::Failed,
            Block::Tools { tools, .. } => tools.iter().any(|t| t.status == ToolStatus::Failed),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_server::Counts;

    fn tool(id: &str, status: ToolStatus) -> Item {
        Item::Tool {
            id: id.to_owned(),
            title: format!("Read {id}"),
            kind: ToolKind::Read,
            status,
        }
    }

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

    fn shape(blocks: &[Block]) -> Vec<&'static str> {
        blocks
            .iter()
            .map(|block| match block {
                Block::You { .. } => "you",
                Block::Agent { .. } => "agent",
                Block::Thought { .. } => "thought",
                Block::Tool(_) => "tool",
                Block::Tools { .. } => "tools",
                Block::Permission { .. } => "permission",
                Block::Proposal { .. } => "proposal",
                Block::Notice { .. } => "notice",
            })
            .collect()
    }

    #[test]
    fn the_view_keeps_the_order_of_the_conversation() {
        let items = vec![
            user("Tidy my trips"),
            Item::Thought {
                text: "hm".into(),
                ms: 12_400,
            },
            tool("a", ToolStatus::Completed),
            agent("Done."),
            Item::Permission {
                title: "Read /tmp/x".into(),
                kind: ToolKind::Read,
                answer: PermissionAnswer::AllowedOnce,
            },
            Item::Notice {
                text: "Stopped".into(),
                at_ms: 0,
            },
        ];
        let blocks = fold(&items, None);
        assert_eq!(
            shape(&blocks),
            ["you", "thought", "tool", "agent", "permission", "notice"]
        );
        assert_eq!(
            blocks[1],
            Block::Thought {
                label: "Thought for 12s".into(),
                text: "hm".into()
            }
        );
        assert_eq!(
            blocks[4],
            Block::Permission {
                text: "Allowed once: Read /tmp/x".into()
            }
        );
    }

    #[test]
    fn three_tools_in_a_row_fold_and_two_do_not() {
        let two = [
            tool("a", ToolStatus::Completed),
            tool("b", ToolStatus::Completed),
        ];
        assert_eq!(shape(&fold(&two, None)), ["tool", "tool"]);
        let mut six: Vec<Item> = (0..6)
            .map(|n| tool(&n.to_string(), ToolStatus::Completed))
            .collect();
        let blocks = fold(&six, None);
        assert_eq!(shape(&blocks), ["tools"]);
        assert!(matches!(&blocks[0], Block::Tools { label, tools }
            if label == "Used 6 tools" && tools.len() == 6));
        // Words in between break the run.
        six.insert(3, agent("x"));
        assert_eq!(shape(&fold(&six, None)), ["tools", "agent", "tools"]);
    }

    #[test]
    fn a_folded_run_shows_a_tool_still_going_or_failed() {
        let items = [
            tool("a", ToolStatus::Completed),
            tool("b", ToolStatus::Failed),
            tool("c", ToolStatus::InProgress),
        ];
        let blocks = fold(&items, None);
        assert!(blocks[0].working() && blocks[0].failed());
    }

    #[test]
    fn a_proposal_card_offers_what_its_round_allows() {
        let counts = Counts {
            creates: 3,
            edits: 1,
            moves: 0,
            merges: 0,
        };
        let card = |round, state| Item::Proposed {
            round,
            counts,
            state,
        };
        let blocks = fold(
            &[
                card(1, RoundState::Applied),
                card(2, RoundState::ToReview),
                card(3, RoundState::Applied),
                card(4, RoundState::Discarded),
            ],
            Some(3),
        );
        let seen: Vec<(String, Offer)> = blocks
            .iter()
            .map(|b| match b {
                Block::Proposal { label, offer, .. } => (label.clone(), offer.clone()),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("Applied 4 changes".to_owned(), Offer::Nothing),
                ("Round 2 · 3 new notes, 1 edit".to_owned(), Offer::Review),
                ("Round 3 · Applied 4 changes".to_owned(), Offer::Undo),
                ("Round 4 · Discarded".to_owned(), Offer::Nothing),
            ]
        );
    }

    #[test]
    fn permission_answers_read_as_the_issue_says() {
        let said = |answer| permission_text(answer, "Read a");
        assert_eq!(
            said(PermissionAnswer::AllowedForSession),
            "Allowed for this session: Read a"
        );
        assert_eq!(
            said(PermissionAnswer::AllowedAlways),
            "Always allowed: Read a"
        );
        assert_eq!(said(PermissionAnswer::Rejected), "Rejected: Read a");
    }
}
