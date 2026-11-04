//! What Dart sees of the library: thin wrappers over `kanso-mobile`, one
//! file per area. The bridge generator reads this folder
//! (`mobile/flutter_rust_bridge.yaml`) and writes the Dart under
//! `mobile/lib/core/bridge`.
//!
//! No decision is made here. A wrapper calls the facade and, for a stream,
//! hands the facade's callback a stream sink. `types` mirrors the facade's
//! data types field by field, so the compiler checks that they still match.

pub mod account;
pub mod account_settings;
pub mod agent_tasks;
pub mod ai;
pub mod approvals;
pub mod board;
pub mod brief;
pub mod canvas;
pub mod canvas_types;
pub mod cards;
pub mod cards_types;
pub mod diagnostics;
pub mod discovery;
pub mod draft;
pub mod draft_types;
pub mod editor;
pub mod feedback;
pub mod gallery;
pub mod import;
pub mod journal;
pub mod library;
pub mod linking;
pub mod linking_types;
pub mod locks;
pub mod metrics;
pub mod notebook_home;
pub mod notebook_home_types;
pub mod overview;
pub mod overview_types;
pub mod people;
pub mod people_types;
pub mod publish;
pub mod recording;
pub mod recording_types;
pub mod reminders;
pub mod renderer;
pub mod shared;
pub mod shortcuts;
pub mod sync;
pub mod themes;
pub mod triage;
pub mod triage_types;
pub mod types;
pub mod voice;
pub mod voice_capture;
pub mod voice_capture_types;
pub mod word_goal;
