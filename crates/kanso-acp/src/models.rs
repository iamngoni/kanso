//! The models an agent says a session can use, read from the answer to
//! `session/new` or `session/load`, and how to ask for another one.
//!
//! ACP has two shapes for this, and an agent may send either or both:
//!
//! * `configOptions`: a select option whose `category` is `"model"`, set
//!   with `session/set_config_option` (`configId`, `value`). The current
//!   protocol; its options are a flat list, or groups of them.
//! * `models`: `availableModels` (`modelId`, `name`, `description`) and
//!   `currentModelId`, set with `session/set_model`. The older, unstable
//!   shape that claude-code-acp and codex-acp still send.
//!
//! When both are there, `configOptions` wins. Everything here is read from
//! the agent's own message and never guessed; a model id is only ever one
//! the agent listed.

use serde_json::Value;

/// One model an agent offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelChoice {
    /// What the agent calls it in requests; this is what Kanso keeps.
    pub id: String,
    /// What the person sees. The id when the agent gave no name.
    pub name: String,
    pub description: Option<String>,
}

/// Which request switches the model of a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSwitch {
    /// `session/set_config_option` on the option `config_id`.
    ConfigOption { config_id: String },
    /// `session/set_model`.
    SetModel,
}

/// The models one agent offered in one session.
///
/// `choices` is never empty and has no id twice: an agent that listed no
/// model has no catalog at all ([`parse_models`] returns `None`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelCatalog {
    choices: Vec<ModelChoice>,
    current: Option<String>,
    switch: ModelSwitch,
}

impl ModelCatalog {
    /// The catalog, or `None` when `choices` is empty. Repeated ids keep
    /// their first entry.
    pub fn new(
        choices: Vec<ModelChoice>,
        current: Option<String>,
        switch: ModelSwitch,
    ) -> Option<ModelCatalog> {
        let mut unique: Vec<ModelChoice> = Vec::with_capacity(choices.len());
        for choice in choices {
            if !unique.iter().any(|have| have.id == choice.id) {
                unique.push(choice);
            }
        }
        (!unique.is_empty()).then_some(ModelCatalog {
            choices: unique,
            current,
            switch,
        })
    }

    pub fn choices(&self) -> &[ModelChoice] {
        &self.choices
    }

    /// The id of the model the session runs now, if the agent said.
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn switch(&self) -> &ModelSwitch {
        &self.switch
    }

    /// The model offered under `id`.
    pub fn find(&self, id: &str) -> Option<&ModelChoice> {
        self.choices.iter().find(|choice| choice.id == id)
    }

    /// Whether the agent offers `id`.
    pub fn offers(&self, id: &str) -> bool {
        self.find(id).is_some()
    }

    /// This catalog after the session switched to `id`.
    pub(crate) fn with_current(mut self, id: &str) -> ModelCatalog {
        self.current = Some(id.to_owned());
        self
    }
}

/// The models in a `session/new` or `session/load` result, or in the answer
/// to `session/set_config_option`; `None` when the agent lists none.
pub fn parse_models(result: &Value) -> Option<ModelCatalog> {
    from_config_options(result).or_else(|| from_models(result))
}

/// A string field, trimmed; empty is absent.
fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// The `configOptions` entry with category `model`, as a catalog.
fn from_config_options(result: &Value) -> Option<ModelCatalog> {
    let option = result
        .get("configOptions")?
        .as_array()?
        .iter()
        .find(|option| option.get("category").and_then(Value::as_str) == Some("model"))?;
    let config_id = text(option, "id")?;
    let mut choices = Vec::new();
    for entry in option.get("options")?.as_array()? {
        // A group holds its own options; a plain entry is one itself.
        match entry.get("options").and_then(Value::as_array) {
            Some(group) => choices.extend(group.iter().filter_map(config_choice)),
            None => choices.extend(config_choice(entry)),
        }
    }
    ModelCatalog::new(
        choices,
        text(option, "currentValue"),
        ModelSwitch::ConfigOption { config_id },
    )
}

fn config_choice(entry: &Value) -> Option<ModelChoice> {
    let id = text(entry, "value")?;
    Some(ModelChoice {
        name: text(entry, "name").unwrap_or_else(|| id.clone()),
        description: text(entry, "description"),
        id,
    })
}

/// The older `models` block, as a catalog.
fn from_models(result: &Value) -> Option<ModelCatalog> {
    let models = result.get("models")?;
    let choices = models
        .get("availableModels")?
        .as_array()?
        .iter()
        .filter_map(|entry| {
            let id = text(entry, "modelId")?;
            Some(ModelChoice {
                name: text(entry, "name").unwrap_or_else(|| id.clone()),
                description: text(entry, "description"),
                id,
            })
        })
        .collect();
    ModelCatalog::new(
        choices,
        text(models, "currentModelId"),
        ModelSwitch::SetModel,
    )
}

#[cfg(test)]
mod tests {
    use super::{ModelCatalog, ModelChoice, ModelSwitch, parse_models};
    use proptest::prelude::*;
    use serde_json::{Value, json};

    fn choice(id: &str) -> ModelChoice {
        ModelChoice {
            id: id.to_owned(),
            name: id.to_owned(),
            description: None,
        }
    }

    #[test]
    fn the_older_models_block_is_read() {
        let result = json!({
            "sessionId": "s",
            "models": {
                "currentModelId": "default",
                "availableModels": [
                    { "modelId": "default", "name": "Default (recommended)", "description": "Opus 4.7" },
                    { "modelId": "sonnet", "name": "Sonnet" },
                ],
            },
        });
        let catalog = parse_models(&result).unwrap();
        assert_eq!(catalog.current(), Some("default"));
        assert_eq!(catalog.switch(), &ModelSwitch::SetModel);
        assert_eq!(catalog.choices().len(), 2);
        assert_eq!(
            catalog.find("default").unwrap().name,
            "Default (recommended)"
        );
        assert_eq!(
            catalog.find("default").unwrap().description.as_deref(),
            Some("Opus 4.7")
        );
        assert_eq!(catalog.find("sonnet").unwrap().description, None);
    }

    #[test]
    fn a_model_config_option_is_read_flat_or_in_groups() {
        let flat = json!({
            "configOptions": [
                { "id": "mode", "category": "mode", "type": "select", "currentValue": "ask",
                  "options": [{ "value": "ask", "name": "Ask" }] },
                { "id": "model", "category": "model", "type": "select", "currentValue": "m1",
                  "options": [
                      { "value": "m1", "name": "Model 1", "description": "Fast" },
                      { "value": "m2", "name": "Model 2" },
                  ] },
            ],
        });
        let catalog = parse_models(&flat).unwrap();
        assert_eq!(
            catalog.switch(),
            &ModelSwitch::ConfigOption {
                config_id: "model".to_owned()
            }
        );
        assert_eq!(catalog.current(), Some("m1"));
        let ids: Vec<&str> = catalog.choices().iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["m1", "m2"]);

        let grouped = json!({
            "configOptions": [{
                "id": "m", "category": "model", "type": "select", "currentValue": "b",
                "options": [
                    { "group": "g1", "name": "Group 1", "options": [{ "value": "a", "name": "A" }] },
                    { "group": "g2", "name": "Group 2", "options": [{ "value": "b", "name": "B" }] },
                ],
            }],
        });
        let catalog = parse_models(&grouped).unwrap();
        let ids: Vec<&str> = catalog.choices().iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
    }

    #[test]
    fn config_options_win_when_an_agent_sends_both() {
        let both = json!({
            "models": { "currentModelId": "old", "availableModels": [{ "modelId": "old", "name": "Old" }] },
            "configOptions": [{
                "id": "model", "category": "model", "type": "select", "currentValue": "new",
                "options": [{ "value": "new", "name": "New" }],
            }],
        });
        let catalog = parse_models(&both).unwrap();
        assert!(catalog.offers("new"));
        assert!(!catalog.offers("old"));
        // An empty config option leaves the older block to answer.
        let empty = json!({
            "models": { "currentModelId": "old", "availableModels": [{ "modelId": "old", "name": "Old" }] },
            "configOptions": [{ "id": "model", "category": "model", "options": [] }],
        });
        assert_eq!(
            parse_models(&empty).unwrap().switch(),
            &ModelSwitch::SetModel
        );
    }

    #[test]
    fn an_agent_that_lists_no_model_has_no_catalog() {
        for result in [
            json!({}),
            json!({ "sessionId": "s", "modes": { "availableModes": [] } }),
            json!({ "models": { "availableModels": [] } }),
            json!({ "models": { "availableModels": [{ "name": "no id" }, { "modelId": "  " }] } }),
            json!({ "models": "nope" }),
            json!({ "configOptions": "nope" }),
            json!({ "configOptions": [{ "id": "mode", "category": "mode", "options": [{ "value": "x" }] }] }),
            json!({ "configOptions": [{ "category": "model", "options": [{ "value": "x" }] }] }),
            Value::Null,
        ] {
            assert_eq!(parse_models(&result), None, "{result}");
        }
    }

    #[test]
    fn a_repeated_id_keeps_its_first_entry() {
        let catalog = ModelCatalog::new(
            vec![
                ModelChoice {
                    name: "First".into(),
                    ..choice("x")
                },
                ModelChoice {
                    name: "Second".into(),
                    ..choice("x")
                },
                choice("y"),
            ],
            None,
            ModelSwitch::SetModel,
        )
        .unwrap();
        assert_eq!(catalog.choices().len(), 2);
        assert_eq!(catalog.find("x").unwrap().name, "First");
        assert!(ModelCatalog::new(Vec::new(), None, ModelSwitch::SetModel).is_none());
    }

    /// An agent's model entries in the older shape, from ids and names.
    fn legacy(entries: &[(String, String)], current: &str) -> Value {
        json!({
            "models": {
                "currentModelId": current,
                "availableModels": entries
                    .iter()
                    .map(|(id, name)| json!({ "modelId": id, "name": name }))
                    .collect::<Vec<_>>(),
            },
        })
    }

    /// The same entries in the newer shape, flat or in one group.
    fn options(entries: &[(String, String)], current: &str, grouped: bool) -> Value {
        let flat: Vec<Value> = entries
            .iter()
            .map(|(id, name)| json!({ "value": id, "name": name }))
            .collect();
        let options = if grouped {
            json!([{ "group": "g", "name": "G", "options": flat }])
        } else {
            json!(flat)
        };
        json!({ "configOptions": [{
            "id": "model", "category": "model", "type": "select",
            "currentValue": current, "options": options,
        }] })
    }

    fn gen_entries() -> impl Strategy<Value = Vec<(String, String)>> {
        prop::collection::vec(("[a-z0-9.-]{1,12}", "[A-Za-z0-9 ]{1,16}"), 1..8)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: whichever shape an agent uses, flat, grouped or the older
        /// block, the same list gives the same choices in the same order and
        /// the same current model; only how to switch differs.
        #[test]
        fn prop_every_shape_of_one_list_reads_the_same(
            entries in gen_entries(),
            pick in any::<prop::sample::Index>(),
        ) {
            let current = entries[pick.index(entries.len())].0.clone();
            let shapes = [
                legacy(&entries, &current),
                options(&entries, &current, false),
                options(&entries, &current, true),
            ];
            let read: Vec<ModelCatalog> = shapes
                .iter()
                .map(|shape| parse_models(shape).expect("a list of models"))
                .collect();
            for catalog in &read {
                prop_assert_eq!(catalog.choices(), read[0].choices());
                prop_assert_eq!(catalog.current(), Some(current.as_str()));
            }
            prop_assert_eq!(read[0].switch(), &ModelSwitch::SetModel);
            // Bound first: `prop_assert!` stringifies its expression into a
            // format string, and the pattern's braces would be read as one.
            let grouped = matches!(read[1].switch(), ModelSwitch::ConfigOption { .. });
            prop_assert!(grouped);
        }

        /// Refinement: a catalog has at least one model, no id twice, and
        /// every id it offers is one the agent sent, whatever the agent sent.
        #[test]
        fn prop_a_catalog_only_offers_what_the_agent_listed(
            entries in prop::collection::vec(("[a-c]{0,2}", "[a-z ]{0,6}"), 0..10),
        ) {
            let result = legacy(&entries, "none");
            match parse_models(&result) {
                None => prop_assert!(entries.iter().all(|(id, _)| id.trim().is_empty())),
                Some(catalog) => {
                    prop_assert!(!catalog.choices().is_empty());
                    let mut seen = std::collections::HashSet::new();
                    for choice in catalog.choices() {
                        prop_assert!(seen.insert(choice.id.clone()), "{} twice", choice.id);
                        prop_assert!(entries.iter().any(|(id, _)| id.trim() == choice.id));
                        prop_assert!(!choice.name.is_empty());
                    }
                }
            }
        }

        /// Whatever JSON arrives, parsing never panics.
        #[test]
        fn prop_arbitrary_json_never_panics(
            key in "[a-zA-Z]{0,8}",
            text in "[ -~]{0,12}",
            number in any::<i64>(),
        ) {
            let result = json!({
                key: [{ "category": text, "options": [number, { "value": text }] }],
                "models": { "availableModels": [number, text] },
            });
            let _ = parse_models(&result);
        }
    }
}
