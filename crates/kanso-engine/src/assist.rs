//! What the inline assistant and the note summary share between the desktop
//! and the phones: which notes they may work on, and what they say when one
//! is out of reach.
//!
//! A feature sends its own prompt, the selection or the one note it works
//! on, and nothing more: no tools, no search. The prompts are in
//! `kanso_ai::features`; this is the rule about which notes they may read.

use kanso_store::Store;

use crate::ask::{self, Candidate};

/// What a feature says when the note it would work on is out of reach of AI.
pub const KEPT_OUT: &str = "This note is kept out of AI.";

/// The plain line about where a question goes: the server it reaches, and
/// what is never sent. `device` is what the sentence calls the device the
/// questions leave ("this Mac", "this phone"). Settings and the Ask panel
/// show the same line.
pub fn requests_line(provider: kanso_ai::Provider, base_url: &str, device: &str) -> String {
    let host = provider.host(base_url);
    let to = if provider.needs_key() {
        host
    } else {
        format!("the model at {host}")
    };
    format!(
        "Questions and the note excerpts they use go straight from {device} to {to}. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
    )
}

/// Whether note `id` may be sent to AI: it is not locked, not in the trash
/// and not tagged `#ai-exclude`. The same decision Ask makes.
pub fn eligible(store: &Store, id: &str) -> bool {
    ask::note_is_eligible(store, id)
}

/// Note `id` when it may be sent, or `None` when it is kept out (or does not
/// exist, or the check failed: it fails closed).
pub fn note_for_ai(store: &Store, id: &str) -> Option<Candidate> {
    ask::eligible_note(store, id).ok().flatten()
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use serde_json::json;

    use super::*;

    // no-prop-test: two fixed sentences, one per kind of provider.
    #[test]
    fn the_requests_line_names_the_device_and_the_host() {
        use kanso_ai::Provider;
        assert_eq!(
            requests_line(Provider::OpenAi, "https://api.openai.com/v1", "this Mac"),
            "Questions and the note excerpts they use go straight from this Mac to api.openai.com. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
        );
        let local = requests_line(
            Provider::LocalModel,
            "http://10.0.0.2:11434/v1",
            "this phone",
        );
        assert!(
            local.contains("from this phone to the model at 10.0.0.2:11434."),
            "{local}"
        );
    }

    // no-prop-test: the exclusions have their store-level property in
    // `ask`; this checks the one door the features use.
    #[test]
    fn a_feature_reads_only_what_ask_may_read() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let put = |title: &str, extra: serde_json::Value| {
            let mut fields = json!({"title": title, "body": "Body text.", "bookId": book});
            fields
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            store
                .put(DocType::Note, fields, Put::Upsert, Origin::Ui)
                .unwrap()
                .id
        };
        let plain = put("Plain", json!({}));
        let sealed = put("Sealed", json!({}));
        store
            .put(
                DocType::Note,
                json!({"_id": sealed, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let excluded = put("Excluded", json!({"tags": [tag]}));
        let trashed = put("Trashed", json!({}));
        store.delete(&trashed, Origin::Ui).unwrap();

        assert_eq!(note_for_ai(&store, &plain).unwrap().title, "Plain");
        assert!(eligible(&store, &plain));
        for id in [&sealed, &excluded, &trashed, &"note:missing".to_owned()] {
            assert!(note_for_ai(&store, id).is_none(), "{id}");
            assert!(!eligible(&store, id), "{id}");
        }
    }
}
