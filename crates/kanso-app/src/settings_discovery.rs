//! Settings › General › Discovery: the switches for
//! the features that find notes by their words (`related.rs`, `ocr.rs`),
//! and where each stands.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::Button;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, v_flex};
use gpui_kit::*;

use crate::ocr::{self, ImageText};
use crate::related::{self, Related, Status};
use crate::settings::{Row, SettingsView, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// What the device is called in the settings' words.
pub fn this_device() -> &'static str {
    "this device"
}

/// "12.4 MB" and the words related notes' status is told in: the same
/// ones the phones use.
pub use kanso_engine::discovery::related::{megabytes, related_words};

/// The Discovery section, for the General page.
// no-prop-test: layout and switches; `related_words` says what it shows,
// and the settings snapshots show each state.
pub fn sections(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let discovery = cx.global::<AppState>().config.discovery;
    let status = cx
        .try_global::<Related>()
        .map(|r| r.status.clone())
        .unwrap_or_default();
    let device = this_device();
    let mut related_rows = vec![
        Row::new(Lucide::Sparkles, "Related notes")
            .detail(format!(
                "Lists notes about the same things as the open one, beside it. Turning this on downloads a small language model (about 30\u{a0}MB) once. It runs on {device}: nothing about your notes is sent anywhere."
            ))
            .trailing(
                div().debug_selector(|| "related-notes".into()).child(
                    Switch::new("related-notes")
                        .checked(discovery.related_notes)
                        .on_click(|on, _, cx| related::set_enabled(*on, cx)),
                ),
            ),
    ];
    if discovery.related_notes
        && let Some((title, detail)) = related_words(&status)
    {
        let icon = match status {
            Status::Downloading { .. } => Lucide::Download,
            Status::Failed(_) => Lucide::TriangleAlert,
            Status::Ready { .. } => Lucide::Check,
            _ => Lucide::LoaderCircle,
        };
        let mut row = Row::new(icon, title).detail(detail);
        if let Status::Downloading { done, total } = status {
            row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
        }
        if let Status::Indexing { done, total } = status {
            row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
        }
        if matches!(status, Status::Failed(_)) {
            row = row.trailing(
                Button::new("related-retry")
                    .outline()
                    .small()
                    .label("Try Again")
                    .on_click(|_, _, cx| related::start(cx)),
            );
        }
        related_rows.push(row);
    }
    let mut rows = related_rows;

    let image_row = if ocr::supported() {
        let reading = cx
            .try_global::<ImageText>()
            .map(|s| s.status.clone())
            .unwrap_or_default();
        let detail = match reading {
            ocr::Status::Reading { left } if discovery.image_text && left > 0 => format!(
                "Reading images… {} to go.",
                crate::text_stats::format_count(left)
            ),
            _ => format!(
                "Reads the words in pictures you attach, on {device}, so search finds the notes they're in. The words sync with the picture."
            ),
        };
        Row::new(Lucide::ScanText, "Search text in images")
            .detail(detail)
            .trailing(
                div().debug_selector(|| "image-text".into()).child(
                    Switch::new("image-text")
                        .checked(discovery.image_text)
                        .on_click(|on, _, cx| ocr::set_enabled(*on, cx)),
                ),
            )
    } else {
        Row::new(Lucide::ScanText, "Search text in images").detail(
            "Kanso reads the words in pictures on macOS. Words read there sync with the picture, and search finds them here too.",
        )
    };
    rows.push(image_row);
    vec![section("Discovery", rows, colors)]
}

/// A short bar `share` (0 to 1) of the way full.
// no-prop-test: drawing; the settings snapshots show it.
pub(crate) fn meter(share: f32, colors: &Colors) -> impl IntoElement {
    v_flex().w(px(96.)).child(
        div()
            .w_full()
            .h(px(4.))
            .rounded_full()
            .bg(colors.fill)
            .child(
                div()
                    .h_full()
                    .w(px(96. * share.clamp(0., 1.)))
                    .rounded_full()
                    .bg(colors.accent),
            ),
    )
}
