//! What one question may cost at most: characters sent, tool calls and
//! output. Three sizes, chosen in Settings, each a ceiling the engine
//! enforces and Settings can state in words and, for a known price, dollars.

/// The three sizes a person chooses between.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BudgetSize {
    /// 12k characters of notes; for small local models.
    Small,
    /// 24k characters, what Ask has always sent.
    #[default]
    Standard,
    /// 60k characters.
    Large,
}

impl BudgetSize {
    /// Every size, smallest first.
    pub const ALL: [BudgetSize; 3] = [BudgetSize::Small, BudgetSize::Standard, BudgetSize::Large];

    /// The size's name as Settings shows it.
    pub fn label(self) -> &'static str {
        match self {
            BudgetSize::Small => "Small",
            BudgetSize::Standard => "Standard",
            BudgetSize::Large => "Large",
        }
    }

    /// The name saved in the settings file.
    pub fn key(self) -> &'static str {
        match self {
            BudgetSize::Small => "small",
            BudgetSize::Standard => "standard",
            BudgetSize::Large => "large",
        }
    }

    /// The size saved under `key`.
    pub fn from_key(key: &str) -> Option<BudgetSize> {
        BudgetSize::ALL.into_iter().find(|size| size.key() == key)
    }
}

/// Tokens a rewrite of a follow-up may write.
pub const REWRITE_OUTPUT_TOKENS: u32 = 300;

/// Characters of the follow-up question and its framing in a rewrite.
pub const REWRITE_QUESTION_CHARS: usize = 2_000;

/// The ceilings for one question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub size: BudgetSize,
    /// Characters of the first prompt: instructions, excerpts, question.
    pub excerpt_chars: usize,
    /// Characters every tool call may hand the model, all together.
    pub tool_chars: usize,
    /// Tool calls.
    pub tool_calls: usize,
    /// Characters of earlier exchanges resent.
    pub history_chars: usize,
    /// Tokens the model may write.
    pub output_tokens: u32,
}

impl Default for Budget {
    fn default() -> Self {
        Budget::of(BudgetSize::Standard)
    }
}

impl Budget {
    /// The ceilings of `size`.
    pub const fn of(size: BudgetSize) -> Budget {
        match size {
            BudgetSize::Small => Budget {
                size,
                excerpt_chars: 12_000,
                tool_chars: 12_000,
                tool_calls: 4,
                history_chars: 6_000,
                output_tokens: 1_024,
            },
            BudgetSize::Standard => Budget {
                size,
                excerpt_chars: 24_000,
                tool_chars: 24_000,
                tool_calls: 6,
                history_chars: 12_000,
                output_tokens: 2_048,
            },
            BudgetSize::Large => Budget {
                size,
                excerpt_chars: 60_000,
                tool_chars: 48_000,
                tool_calls: 10,
                history_chars: 24_000,
                output_tokens: 4_096,
            },
        }
    }

    /// An upper bound on the characters one question can send to the model,
    /// over every request it can make:
    ///
    /// - each round of the answer (`tool_calls + 1` of them) resends the
    ///   first prompt and the history, and, in the worst case, every tool
    ///   result already handed over (all of it from the first call);
    /// - one repair, which resends the excerpts, the lines the tools read
    ///   and the answer being repaired;
    /// - one rewrite of a follow-up, which sends the history and the
    ///   question.
    pub fn most_chars_sent(&self) -> usize {
        let base = self.excerpt_chars + self.history_chars;
        let rounds = self.tool_calls + 1;
        let answer = rounds * base + self.tool_calls * self.tool_chars;
        let repair = self.excerpt_chars + self.tool_chars + self.output_tokens as usize * 4;
        let rewrite = self.history_chars + REWRITE_QUESTION_CHARS;
        answer + repair + rewrite
    }

    /// An upper bound on the tokens the model may write for one question:
    /// every round, one repair and one rewrite.
    pub fn most_tokens_written(&self) -> u32 {
        (self.tool_calls as u32 + 2) * self.output_tokens + REWRITE_OUTPUT_TOKENS
    }

    /// About how many words of notes one question may send (a word is
    /// about eight characters with its space).
    pub fn words_of_notes(&self) -> usize {
        self.excerpt_chars / 8
    }

    /// "About 3,000 words of your notes per question."
    pub fn words_line(&self) -> String {
        format!(
            "About {} words of your notes per question.",
            group_thousands(self.words_of_notes())
        )
    }
}

/// `n` with a comma between thousands.
pub fn group_thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (ix, ch) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_sizes_are_the_ones_the_settings_name() {
        let small = Budget::of(BudgetSize::Small);
        let standard = Budget::default();
        let large = Budget::of(BudgetSize::Large);
        assert_eq!(standard.excerpt_chars, crate::DEFAULT_BUDGET);
        assert_eq!(standard.output_tokens, 2048);
        assert_eq!(standard.tool_calls, crate::client::MAX_TOOL_CALLS);
        assert_eq!((small.excerpt_chars, large.excerpt_chars), (12_000, 60_000));
        assert_eq!(
            small.words_line(),
            "About 1,500 words of your notes per question."
        );
        assert_eq!(
            standard.words_line(),
            "About 3,000 words of your notes per question."
        );
        assert_eq!(
            large.words_line(),
            "About 7,500 words of your notes per question."
        );
    }

    proptest! {
        /// Every size is bigger than the one before it, and its key reads back.
        #[test]
        fn prop_sizes_grow_and_keys_round_trip(ix in 0usize..3) {
            let size = BudgetSize::ALL[ix];
            prop_assert_eq!(BudgetSize::from_key(size.key()), Some(size));
            if ix > 0 {
                let before = Budget::of(BudgetSize::ALL[ix - 1]);
                let now = Budget::of(size);
                prop_assert!(now.excerpt_chars > before.excerpt_chars);
                prop_assert!(now.tool_calls > before.tool_calls);
                prop_assert!(now.most_chars_sent() > before.most_chars_sent());
            }
        }

        #[test]
        fn prop_thousands_group_by_three(n in 0usize..10_000_000) {
            let text = group_thousands(n);
            prop_assert_eq!(text.replace(',', "").parse::<usize>().unwrap(), n);
            for group in text.split(',').skip(1) {
                prop_assert_eq!(group.len(), 3);
            }
        }
    }
}
