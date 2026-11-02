//! What a model costs, by hand: a small table of list prices in US dollars
//! per million tokens, so Settings can say what one question costs at most.
//! A model that isn't in it shows no estimate. Updated by hand; prices change.

use crate::Provider;
use crate::budget::Budget;

/// One model's list price, dollars per million tokens.
struct Price {
    provider: Provider,
    /// The model ID, or the start of one (`gpt-4o-mini-2024-07-18`).
    model: &'static str,
    input: f64,
    output: f64,
}

/// The table. Longer, more specific IDs come before the shorter IDs they
/// start with.
const PRICES: &[Price] = &[
    Price {
        provider: Provider::OpenAi,
        model: "gpt-4o-mini",
        input: 0.15,
        output: 0.60,
    },
    Price {
        provider: Provider::OpenAi,
        model: "gpt-4o",
        input: 2.50,
        output: 10.00,
    },
    Price {
        provider: Provider::OpenAi,
        model: "gpt-4.1-nano",
        input: 0.10,
        output: 0.40,
    },
    Price {
        provider: Provider::OpenAi,
        model: "gpt-4.1-mini",
        input: 0.40,
        output: 1.60,
    },
    Price {
        provider: Provider::OpenAi,
        model: "gpt-4.1",
        input: 2.00,
        output: 8.00,
    },
    Price {
        provider: Provider::Anthropic,
        model: "claude-3-5-haiku",
        input: 0.80,
        output: 4.00,
    },
    Price {
        provider: Provider::Anthropic,
        model: "claude-3-5-sonnet",
        input: 3.00,
        output: 15.00,
    },
    Price {
        provider: Provider::Anthropic,
        model: "claude-3-7-sonnet",
        input: 3.00,
        output: 15.00,
    },
    Price {
        provider: Provider::Gemini,
        model: "gemini-1.5-flash",
        input: 0.075,
        output: 0.30,
    },
    Price {
        provider: Provider::Gemini,
        model: "gemini-2.0-flash",
        input: 0.10,
        output: 0.40,
    },
    Price {
        provider: Provider::DeepSeek,
        model: "deepseek-chat",
        input: 0.27,
        output: 1.10,
    },
    Price {
        provider: Provider::Mistral,
        model: "mistral-small-latest",
        input: 0.20,
        output: 0.60,
    },
    Price {
        provider: Provider::Groq,
        model: "llama-3.3-70b-versatile",
        input: 0.59,
        output: 0.79,
    },
];

/// The most one question can cost in US dollars with `model` at `provider`,
/// or `None` when the price isn't known: `(input chars / 4) × input price +
/// output tokens × output price`, for the whole of `budget`.
pub fn most_per_question(provider: Provider, model: &str, budget: &Budget) -> Option<f64> {
    let model = model.trim().to_ascii_lowercase();
    let price = PRICES
        .iter()
        .find(|price| price.provider == provider && model.starts_with(price.model))?;
    let input_tokens = budget.most_chars_sent() as f64 / 4.;
    Some(
        input_tokens * price.input / 1_000_000.
            + f64::from(budget.output_tokens) * price.output / 1_000_000.,
    )
}

/// "At most about $0.01 per question with gpt-4o-mini.", rounding the cost
/// up to a whole cent so it is never understated.
pub fn cost_line(provider: Provider, model: &str, budget: &Budget) -> Option<String> {
    let cost = most_per_question(provider, model, budget)?;
    let cents = (cost * 100.).ceil().max(1.);
    Some(format!(
        "At most about ${}.{:02} per question with {}.",
        (cents as u64) / 100,
        (cents as u64) % 100,
        model.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::BudgetSize;
    use proptest::prelude::*;

    #[test]
    fn a_known_model_has_a_cost_line_and_an_unknown_one_has_none() {
        let standard = Budget::default();
        assert_eq!(
            cost_line(Provider::OpenAi, "gpt-4o-mini", &standard).as_deref(),
            Some("At most about $0.01 per question with gpt-4o-mini.")
        );
        assert_eq!(cost_line(Provider::OpenAi, "gpt-9", &standard), None);
        assert_eq!(cost_line(Provider::LocalModel, "llama3.2", &standard), None);
    }

    #[test]
    fn a_dated_model_id_matches_its_family() {
        let cost = most_per_question(
            Provider::OpenAi,
            "gpt-4o-mini-2024-07-18",
            &Budget::default(),
        )
        .unwrap();
        let plain = most_per_question(Provider::OpenAi, "gpt-4o-mini", &Budget::default()).unwrap();
        assert_eq!(cost, plain);
    }

    proptest! {
        /// A bigger budget never costs less, and a price is never negative.
        #[test]
        fn prop_cost_grows_with_the_budget(ix in 0usize..PRICES.len()) {
            let price = &PRICES[ix];
            let costs: Vec<f64> = BudgetSize::ALL
                .iter()
                .map(|size| most_per_question(price.provider, price.model, &Budget::of(*size)).unwrap())
                .collect();
            prop_assert!(costs.iter().all(|c| *c > 0.));
            prop_assert!(costs.windows(2).all(|w| w[0] < w[1]));
        }
    }
}
