//! Settings › General › Discovery: the switches for
//! the features that find notes by their words (`related.rs`, `ocr.rs`,
//! `pdf/index.rs`, `ink_ocr.rs`),
//! and where each stands.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::Button;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, v_flex};
use gpui_kit::*;

use crate::ink_ocr::{self, InkText};
use crate::ocr::{self, ImageText};
use crate::pdf::index::{self as pdf_index, PdfText};
use crate::pdf::ocr::{self as pdf_scans, ScannedPages};
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

    let pdf_row = if pdf_index::supported() {
        let reading = cx
            .try_global::<PdfText>()
            .map(|s| s.status.clone())
            .unwrap_or_default();
        let detail = match reading {
            pdf_index::Status::Indexing { left } if discovery.pdf_text && left > 0 => format!(
                "Reading PDFs… {} to go.",
                crate::text_stats::format_count(left)
            ),
            pdf_index::Status::Waiting { left } if discovery.pdf_text => format!(
                "Kanso's PDF reader isn't working right now. It will try again soon; {} to go.",
                crate::text_stats::format_count(left)
            ),
            _ => format!(
                "Reads the words in PDFs you attach, on {device}, so search finds the notes they're in. Turning this off forgets them."
            ),
        };
        Row::new(Lucide::FileText, "Search text in PDFs")
            .detail(detail)
            .trailing(
                div().debug_selector(|| "pdf-text".into()).child(
                    Switch::new("pdf-text")
                        .checked(discovery.pdf_text)
                        .on_click(|on, _, cx| pdf_index::set_enabled(*on, cx)),
                ),
            )
    } else {
        Row::new(Lucide::FileText, "Search text in PDFs")
            .detail("Kanso couldn't find its PDF reader, so the words in PDFs can't be searched.")
    };
    rows.push(pdf_row);
    rows.push(scans_row(&discovery, device, cx));
    rows.push(
        Row::new(Lucide::Quote, "Quote highlights into the note")
            .detail(
                "Highlighting text in a PDF also adds the quote, with a link back to its page, under Highlights in the note. Undo takes it out in one step.",
            )
            .trailing(
                div().debug_selector(|| "pdf-quote".into()).child(
                    Switch::new("pdf-quote")
                        .checked(discovery.pdf_quote)
                        .on_click(|on, _, cx| {
                            let on = *on;
                            AppState::update_config(cx, |config| config.discovery.pdf_quote = on);
                        }),
                ),
            ),
    );
    rows.push(ink_row(&discovery, device, cx));
    vec![section("Discovery", rows, colors)]
}

/// The scanned-pages row: where reading them stands and a pause, where this
/// device can read them; where it can't, why, and that words a Mac read are
/// used here.
// no-prop-test: layout and a button; `pdf/ocr.rs` tests the words it says.
fn scans_row(
    discovery: &crate::config::DiscoverySettings,
    device: &str,
    cx: &Context<SettingsView>,
) -> Row {
    let title = "Search scanned PDF pages";
    if !pdf_scans::supported() {
        return Row::new(Lucide::ScanText, title).detail(
            "Kanso reads scanned PDF pages on macOS. Words read there sync with the PDF, and search, selecting and highlighting use them here too.",
        );
    }
    let state = cx.try_global::<ScannedPages>();
    let status = state.map(|s| s.status.clone()).unwrap_or_default();
    let paused = state.is_some_and(ScannedPages::is_paused);
    let on = discovery.pdf_text && discovery.image_text;
    let detail = if !on {
        "Turn on Search text in PDFs and Search text in images to read scanned pages.".to_owned()
    } else {
        pdf_scans::words(&status).unwrap_or_else(|| {
            format!(
                "Reads the words on scanned PDF pages, on {device}, in the background. You can select, copy and highlight them, and search finds them. The words sync with the PDF."
            )
        })
    };
    let mut row = Row::new(Lucide::ScanText, title).detail(detail);
    let active = matches!(
        status,
        pdf_scans::Status::Reading { .. }
            | pdf_scans::Status::Paused { .. }
            | pdf_scans::Status::Waiting { .. }
    );
    if on && (active || paused) {
        let (label, id) = if paused {
            ("Resume", "pdf-scans-resume")
        } else {
            ("Pause", "pdf-scans-pause")
        };
        row = row.trailing(
            Button::new(id)
                .outline()
                .small()
                .label(label)
                .on_click(move |_, _, cx| pdf_scans::set_paused(!paused, cx)),
        );
    }
    row
}

/// The handwriting row: a switch where this device can read ink, the same
/// words as the images row elsewhere.
// no-prop-test: layout and a switch; the settings snapshots show it.
fn ink_row(
    discovery: &crate::config::DiscoverySettings,
    device: &str,
    cx: &Context<SettingsView>,
) -> Row {
    if !ink_ocr::supported() {
        return Row::new(Lucide::PenTool, "Search handwriting").detail(
            "Kanso reads handwriting on canvases and sketches on macOS. Words read there sync with the canvas, and search finds them here too.",
        );
    }
    let reading = cx
        .try_global::<InkText>()
        .map(|s| s.status.clone())
        .unwrap_or_default();
    let detail = match reading {
        ocr::Status::Reading { left } if discovery.ink_text && left > 0 => format!(
            "Reading handwriting… {} to go.",
            crate::text_stats::format_count(left)
        ),
        _ => format!(
            "Reads what you write or draw on canvases and sketches, on {device}, so search finds them. The words stay inside the canvas, sealed with it if you lock it."
        ),
    };
    Row::new(Lucide::PenTool, "Search handwriting")
        .detail(detail)
        .trailing(
            div().debug_selector(|| "ink-text".into()).child(
                Switch::new("ink-text")
                    .checked(discovery.ink_text)
                    .on_click(|on, _, cx| ink_ocr::set_enabled(*on, cx)),
            ),
        )
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
