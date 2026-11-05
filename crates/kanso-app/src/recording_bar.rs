//! The strip at the top of every Kanso window while a recording runs, or
//! after a crash or a quit left one unfinished (issue #38, step 4):
//!
//! ```text
//! ● 00:42:17   ▁▃▅▂▆▁  Microphone      [Pause] [Stop] [Discard]
//! ```
//!
//! The red dot pulses, the clock is audio time with pause excluded, and the
//! meter is the recording's level. The strip takes no space when there is
//! nothing to say.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex};
use gpui_kit::*;

use crate::recording;
use crate::recording_model::{BarState, CrashBanner, Notice, PendingTranscript};
use crate::theme::Colors;
use crate::zoom::px;

/// The strip, or nothing.
// no-prop-test: drawing; the snapshots show each state.
pub fn strip(cx: &mut App) -> Option<AnyElement> {
    let notice = recording::notice(cx)?;
    let colors = Colors::get(cx);
    let row = h_flex()
        .id("recording-strip")
        .debug_selector(|| "recording-strip".into())
        .flex_none()
        .w_full()
        .h(px(32.))
        .pl(px(if cfg!(target_os = "macos") { 84. } else { 12. }))
        .pr(px(12.))
        .gap(px(10.))
        .items_center()
        .bg(colors.sidebar)
        .border_b_1()
        .border_color(colors.divider)
        .text_size(px(12.5))
        .text_color(colors.text_secondary);
    Some(match notice {
        Notice::Recording(state) => recording_bar(row, &state, colors).into_any_element(),
        Notice::Crash(banner) => crash_banner(row, &banner, colors).into_any_element(),
        Notice::Pending(pending) => pending_banner(row, &pending, colors).into_any_element(),
    })
}

fn recording_bar(row: Stateful<Div>, state: &BarState, colors: &Colors) -> impl IntoElement {
    let dot = if state.dot_lit {
        Hsla::from(colors.danger)
    } else {
        Hsla::from(colors.danger).opacity(0.35)
    };
    let paused = state.paused;
    row.child(
        div()
            .debug_selector(|| "recording-dot".into())
            .size(px(9.))
            .rounded_full()
            .bg(dot),
    )
    .child(
        div()
            .debug_selector(|| "recording-clock".into())
            .min_w(px(58.))
            .text_color(colors.text)
            .child(kanso_engine::recording_flow::clock(state.seconds)),
    )
    .child(meter(state.level, colors))
    .child(div().flex_1().child(if paused {
        format!("{} · Paused", state.label)
    } else {
        state.label.clone()
    }))
    .child(
        Button::new("recording-pause")
            .debug_selector(|| "recording-pause".into())
            .ghost()
            .xsmall()
            .icon(
                Icon::new(if paused { Lucide::Play } else { Lucide::Pause })
                    .text_color(colors.text_secondary),
            )
            .label(if paused { "Resume" } else { "Pause" })
            .on_click(move |_, _, cx| {
                if paused {
                    recording::resume(cx)
                } else {
                    recording::pause(cx)
                }
            }),
    )
    .child(
        Button::new("recording-stop")
            .debug_selector(|| "recording-stop".into())
            .ghost()
            .xsmall()
            .icon(Icon::new(Lucide::Square).text_color(colors.danger))
            .label("Stop")
            .tooltip("Stop and keep the recording")
            .on_click(|_, _, cx| recording::stop(cx)),
    )
    .child(
        Button::new("recording-discard")
            .debug_selector(|| "recording-discard".into())
            .ghost()
            .xsmall()
            .icon(Icon::new(Lucide::Trash).text_color(colors.text_secondary))
            .label("Discard")
            .tooltip("Delete the recording")
            .on_click(|_, window, cx| recording::discard(window, cx)),
    )
}

/// The level, in the editor's voice-pill style.
fn meter(level: f32, colors: &Colors) -> impl IntoElement {
    let level = level.clamp(0., 1.).sqrt();
    h_flex().h(px(14.)).gap(px(2.)).items_center().children(
        [0.4f32, 0.7, 1., 0.8, 0.55, 0.9, 0.6].map(|weight| {
            div()
                .w(px(2.5))
                .h(px(3. + 11. * level * weight))
                .rounded_full()
                .bg(colors.danger)
        }),
    )
}

fn crash_banner(row: Stateful<Div>, banner: &CrashBanner, colors: &Colors) -> impl IntoElement {
    let (recover_id, discard_id) = (banner.id.clone(), banner.id.clone());
    row.child(
        Icon::new(Lucide::TriangleAlert)
            .size(px(14.))
            .text_color(colors.danger),
    )
    .child(
        div()
            .debug_selector(|| "recovery-text".into())
            .flex_1()
            .text_color(colors.text)
            .child(banner.text.clone()),
    )
    .child(
        Button::new("recording-recover")
            .debug_selector(|| "recording-recover".into())
            .small()
            .primary()
            .label("Recover")
            .on_click(move |_, _, cx| recording::recover(&recover_id, cx)),
    )
    .child(
        Button::new("recording-discard-found")
            .debug_selector(|| "recording-discard-found".into())
            .small()
            .label("Discard")
            .on_click(move |_, window, cx| recording::discard_found(&discard_id, window, cx)),
    )
}

fn pending_banner(
    row: Stateful<Div>,
    _pending: &PendingTranscript,
    colors: &Colors,
) -> impl IntoElement {
    row.child(
        Icon::new(Lucide::AudioLines)
            .size(px(14.))
            .text_color(colors.text_secondary),
    )
    .child(
        div()
            .flex_1()
            .text_color(colors.text)
            .child("A recording was still being transcribed when Kanso quit."),
    )
    .child(
        Button::new("recording-finish-transcribing")
            .debug_selector(|| "recording-finish-transcribing".into())
            .small()
            .primary()
            .label("Finish transcribing")
            .on_click(|_, window, cx| recording::finish_transcribing(window, cx)),
    )
    .child(
        Button::new("recording-dismiss-pending")
            .debug_selector(|| "recording-dismiss-pending".into())
            .small()
            .label("Dismiss")
            .on_click(|_, _, cx| recording::clear_pending(cx)),
    )
}
