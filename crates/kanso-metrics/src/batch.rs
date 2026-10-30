//! The batch as the Worker reads it (docs/METRICS.md › The batch): version
//! 1, one ID, 1 to 32 events, each with exactly its props.

use serde::{Serialize, Serializer};

use crate::values::{Active, Day, Device, Feature, Form, Id, ImportSource, Platform};

/// The most events a batch carries.
pub const MAX_EVENTS: usize = 32;
/// The most bytes a batch's body may be.
pub const MAX_BYTES: usize = 16 * 1024;

/// What happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    AppActive(Active),
    FeatureUsed(Feature),
}

/// One event, on the UTC day it happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub day: Day,
    pub kind: EventKind,
}

/// What one request sends: who is counted, the device every event shares,
/// and the events. Made by [`crate::Ledger::due`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub id: Id,
    pub device: Device,
    pub events: Vec<Event>,
}

impl Batch {
    /// The request's body.
    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("a batch is plain JSON")
    }
}

#[derive(Serialize)]
struct Wire<'a> {
    v: u8,
    id: &'a str,
    events: Vec<WireEvent<'a>>,
}

#[derive(Serialize)]
struct WireEvent<'a> {
    event: &'static str,
    day: Day,
    props: WireProps<'a>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum WireProps<'a> {
    Active {
        platform: Platform,
        form: Form,
        app_version: &'a str,
        os_version: &'a str,
        sync: crate::values::SyncMode,
        ai: bool,
        notes: crate::values::NotesRange,
    },
    Feature {
        platform: Platform,
        form: Form,
        app_version: &'a str,
        feature: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<ImportSource>,
    },
}

impl Serialize for Batch {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let device = &self.device;
        let events = self
            .events
            .iter()
            .map(|event| {
                let (name, props) = match &event.kind {
                    EventKind::AppActive(active) => (
                        "app_active",
                        WireProps::Active {
                            platform: device.platform(),
                            form: device.form(),
                            app_version: device.app_version(),
                            os_version: active.os_version(),
                            sync: active.sync(),
                            ai: active.ai(),
                            notes: active.notes(),
                        },
                    ),
                    EventKind::FeatureUsed(feature) => (
                        "feature_used",
                        WireProps::Feature {
                            platform: device.platform(),
                            form: device.form(),
                            app_version: device.app_version(),
                            feature: feature.name(),
                            source: feature.source(),
                        },
                    ),
                };
                WireEvent {
                    event: name,
                    day: event.day,
                    props,
                }
            })
            .collect();
        Wire {
            v: 1,
            id: self.id.as_str(),
            events,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::SyncMode;

    /// The page's own example, byte for byte apart from spacing.
    #[test]
    fn writes_the_spec_example() {
        let day = Day::parse("2026-10-30").unwrap();
        let batch = Batch {
            id: Id::parse("a_3f9c0d4e5b6a79881726354433221100").unwrap(),
            device: Device::new(Platform::Macos, Form::Desktop, "0.9.0"),
            events: vec![
                Event {
                    day,
                    kind: EventKind::AppActive(Active::new("15.1", SyncMode::Kanso, false, 500)),
                },
                Event {
                    day,
                    kind: EventKind::FeatureUsed(Feature::Import(ImportSource::Obsidian)),
                },
            ],
        };
        let expected = serde_json::json!({
            "v": 1,
            "id": "a_3f9c0d4e5b6a79881726354433221100",
            "events": [
                {
                    "event": "app_active",
                    "day": "2026-10-30",
                    "props": {
                        "platform": "macos",
                        "form": "desktop",
                        "app_version": "0.9.0",
                        "os_version": "15.1",
                        "sync": "kanso",
                        "ai": false,
                        "notes": "100-999"
                    }
                },
                {
                    "event": "feature_used",
                    "day": "2026-10-30",
                    "props": { "platform": "macos", "form": "desktop", "app_version": "0.9.0", "feature": "import", "source": "obsidian" }
                }
            ]
        });
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&batch.to_json()).unwrap(),
            expected
        );
        // The props keep the page's order.
        let text = String::from_utf8(batch.to_json()).unwrap();
        assert!(text.contains(r#""props":{"platform":"macos","form":"desktop","app_version":"0.9.0","os_version":"15.1","sync":"kanso","ai":false,"notes":"100-999"}"#));
        // A feature without a source has no `source` at all.
        let lock = Batch {
            events: vec![Event {
                day,
                kind: EventKind::FeatureUsed(Feature::Lock),
            }],
            ..batch
        };
        assert!(
            !String::from_utf8(lock.to_json())
                .unwrap()
                .contains("source")
        );
    }
}
