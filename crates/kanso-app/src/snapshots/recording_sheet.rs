//! Snapshot states for the Record sheet's system audio choices (issue #38,
//! step 7): the permission row waiting to be asked, after the system said
//! no, and allowed with the app picker beside it. The machine is fixed, so
//! the shots don't depend on the apps that happen to be open.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_audio::system_audio::Availability;
use kanso_audio::{CaptureApp, ScreenAccess};
use kanso_engine::recording_flow::{SystemSource, WhatToRecord};
use kanso_store::Store;

use crate::record_sheet::{Fixed, Machine};
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up,
/// and the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-record-system-asking", asking, Some(WIDTH)),
    ("light-record-system-asking", asking, Some(WIDTH)),
    ("dark-record-system-denied", denied, Some(WIDTH)),
    ("light-record-system-denied", denied, Some(WIDTH)),
    ("dark-record-system-allowed", allowed, Some(WIDTH)),
    ("light-record-system-allowed", allowed, Some(WIDTH)),
];

const WIDTH: f32 = 480.;

fn app(id: &str, name: &str) -> CaptureApp {
    CaptureApp {
        bundle_id: id.into(),
        name: name.into(),
    }
}

/// What the picker lists, in the order the app gives it: meeting apps first.
fn apps() -> Vec<CaptureApp> {
    kanso_audio::picker_order(vec![
        app("com.apple.Notes", "Notes"),
        app("com.google.Chrome", "Google Chrome"),
        app("us.zoom.xos", "zoom.us"),
        app("com.tinyspeck.slackmacgap", "Slack"),
        app("com.apple.Music", "Music"),
    ])
}

fn zoom() -> SystemSource {
    SystemSource::App {
        bundle_id: "us.zoom.xos".into(),
        name: "zoom.us".into(),
    }
}

fn open(cx: &mut HeadlessAppContext, window: AnyWindowHandle, fixed: Fixed) {
    cx.update_window(window, |_, window, cx| {
        crate::record_sheet::open_with(None, Some(fixed), window, cx);
    })
    .unwrap();
}

/// System audio just turned on, before macOS has been asked.
fn asking(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(
        cx,
        window,
        Fixed {
            machine: Some(Machine {
                apps: apps(),
                access: ScreenAccess::NotAllowed,
                asked: false,
            }),
            what: WhatToRecord::MicrophoneAndSystem,
            system: Some(zoom()),
            backend: Some((Availability::Available, Vec::new())),
        },
    );
}

/// macOS said no: the row says where to turn it on and that the microphone
/// records either way.
fn denied(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(
        cx,
        window,
        Fixed {
            machine: Some(Machine {
                apps: apps(),
                access: ScreenAccess::NotAllowed,
                asked: true,
            }),
            what: WhatToRecord::MicrophoneAndSystem,
            system: Some(zoom()),
            backend: Some((Availability::Available, Vec::new())),
        },
    );
}

/// Allowed, with zoom.us picked.
fn allowed(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(
        cx,
        window,
        Fixed {
            machine: Some(Machine {
                apps: apps(),
                access: ScreenAccess::Allowed,
                asked: true,
            }),
            what: WhatToRecord::MicrophoneAndSystem,
            system: Some(zoom()),
            backend: Some((Availability::Available, Vec::new())),
        },
    );
}
