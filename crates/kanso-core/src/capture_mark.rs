//! What a voice capture records about itself in the note's `kansoCapture`
//! field: `{v: 1, kind: "voice", duration_ms, engine, tidy}`.
//!
//! The field syncs with the note. Its `tidy` state only moves forward
//! (none, then pending, then done), so two devices that disagree about it
//! agree on the later one ([`merge_capture`]).

use serde_json::{Map, Value, json};

/// The note field holding the capture's record.
pub const CAPTURE_FIELD: &str = "kansoCapture";

/// Where a capture's tidy-up stands. Ordered: a state never goes back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TidyState {
    /// Not tidied, and not asked to be.
    None,
    /// To be tidied the next time the app can.
    Pending,
    /// Tidied.
    Done,
}

impl TidyState {
    /// The word the field stores.
    pub const fn word(self) -> &'static str {
        match self {
            TidyState::None => "none",
            TidyState::Pending => "pending",
            TidyState::Done => "done",
        }
    }

    /// The state a stored word names; anything unknown reads as `None`.
    pub fn from_word(word: &str) -> TidyState {
        match word {
            "pending" => TidyState::Pending,
            "done" => TidyState::Done,
            _ => TidyState::None,
        }
    }
}

/// The recogniser that made a capture's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Engine {
    /// The desktop's Parakeet model, on this computer.
    Parakeet,
    /// The phone's recogniser, on the device.
    SystemOnDevice,
    /// The phone's recogniser, which sent the audio to a server.
    SystemServer,
}

impl Engine {
    /// The word the field stores.
    pub const fn word(self) -> &'static str {
        match self {
            Engine::Parakeet => "parakeet",
            Engine::SystemOnDevice => "system-on-device",
            Engine::SystemServer => "system-server",
        }
    }

    /// The engine a stored word names.
    pub fn from_word(word: &str) -> Option<Engine> {
        [
            Engine::Parakeet,
            Engine::SystemOnDevice,
            Engine::SystemServer,
        ]
        .into_iter()
        .find(|engine| engine.word() == word)
    }

    /// Whether the audio stayed on the device.
    pub const fn on_device(self) -> bool {
        !matches!(self, Engine::SystemServer)
    }
}

/// The field's value for a new capture.
pub fn capture_value(duration_ms: u64, engine: Engine, tidy: TidyState) -> Value {
    json!({
        "v": 1,
        "kind": "voice",
        "duration_ms": duration_ms,
        "engine": engine.word(),
        "tidy": tidy.word(),
    })
}

/// The tidy state a note's fields record, if it is a voice capture.
pub fn read_tidy(fields: &Map<String, Value>) -> Option<TidyState> {
    let mark = fields.get(CAPTURE_FIELD)?.as_object()?;
    (mark.get("kind").and_then(Value::as_str) == Some("voice"))
        .then(|| TidyState::from_word(mark.get("tidy").and_then(Value::as_str).unwrap_or("none")))
}

/// The duration a note's capture record gives, in ms.
pub fn read_duration_ms(fields: &Map<String, Value>) -> Option<u64> {
    fields
        .get(CAPTURE_FIELD)?
        .as_object()?
        .get("duration_ms")?
        .as_u64()
}

/// `mark` with its tidy state set to `tidy`. A value that isn't an object
/// is left as it is.
pub fn with_tidy(mark: &Value, tidy: TidyState) -> Value {
    let mut mark = mark.clone();
    if let Some(object) = mark.as_object_mut() {
        object.insert("tidy".into(), json!(tidy.word()));
    }
    mark
}

/// The field two devices agree on: `mine`, with the later of the two tidy
/// states. `None` when that changes nothing.
pub fn merge_capture(mine: &Value, theirs: &Value) -> Option<Value> {
    let state = |value: &Value| {
        TidyState::from_word(value.get("tidy").and_then(Value::as_str).unwrap_or("none"))
    };
    let (a, b) = (state(mine), state(theirs));
    (b > a && mine.is_object()).then(|| with_tidy(mine, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    fn state() -> impl Strategy<Value = TidyState> {
        prop_oneof![
            Just(TidyState::None),
            Just(TidyState::Pending),
            Just(TidyState::Done)
        ]
    }

    proptest! {
        #![proptest_config(Config {
            cases: 128,
            rng_seed: RngSeed::Fixed(0x0063_6170_7475),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        #[test]
        fn merging_keeps_the_later_state_whichever_side_is_ours(a in state(), b in state()) {
            let one = capture_value(5, Engine::Parakeet, a);
            let two = capture_value(5, Engine::Parakeet, b);
            let left = merge_capture(&one, &two).unwrap_or(one.clone());
            let right = merge_capture(&two, &one).unwrap_or(two.clone());
            prop_assert_eq!(left, right);
        }

        #[test]
        fn a_state_never_goes_back(a in state(), b in state()) {
            let one = capture_value(5, Engine::SystemOnDevice, a);
            let two = capture_value(5, Engine::SystemOnDevice, b);
            let merged = merge_capture(&one, &two).unwrap_or(one);
            prop_assert!(TidyState::from_word(merged["tidy"].as_str().unwrap()) >= a);
        }
    }

    // no-prop-test: three fixed words read back.
    #[test]
    fn words_round_trip() {
        for engine in [
            Engine::Parakeet,
            Engine::SystemOnDevice,
            Engine::SystemServer,
        ] {
            assert_eq!(Engine::from_word(engine.word()), Some(engine));
        }
        assert_eq!(TidyState::from_word("junk"), TidyState::None);
        assert!(!Engine::SystemServer.on_device());
    }
}
