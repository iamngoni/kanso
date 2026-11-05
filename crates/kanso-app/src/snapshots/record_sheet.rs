//! Snapshot states for the Record sheet's system audio choices: the output
//! picker on a Windows-like machine with three outputs, and the choices
//! disabled on a Linux machine whose sound server has no monitor source.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_audio::system_audio::{Availability, SystemAudioDevice, Unavailable};
use kanso_engine::recording_flow::WhatToRecord;
use kanso_store::Store;

use crate::workspace::Workspace;

fn open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    system: Availability,
    outputs: Vec<SystemAudioDevice>,
    what: WhatToRecord,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::record_sheet::open_for_snapshot(system, outputs, what, window, cx);
    })
    .unwrap();
}

fn output(id: &str, name: &str, is_default: bool) -> SystemAudioDevice {
    SystemAudioDevice {
        id: id.into(),
        name: name.into(),
        is_default,
    }
}

/// Microphone + System Audio chosen, with three outputs to pick from.
pub(super) fn output_picker(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open(
        cx,
        window,
        Availability::Available,
        vec![
            output(
                "wasapi:{0.0.0.1}.speakers",
                "Speakers (Realtek Audio)",
                true,
            ),
            output("wasapi:{0.0.0.2}.headset", "Headset (Jabra Evolve2)", false),
            output("wasapi:{0.0.0.3}.monitor", "DELL U2723QE (HDMI)", false),
        ],
        WhatToRecord::MicrophoneAndSystem,
    );
}

/// No monitor source: both system audio choices are disabled.
pub(super) fn no_monitor(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open(
        cx,
        window,
        Availability::Unavailable(Unavailable::NoMonitorSource),
        Vec::new(),
        WhatToRecord::Microphone,
    );
}
