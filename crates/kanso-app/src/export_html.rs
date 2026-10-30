//! Exporting the open note as a web page or a PDF, in the preview's theme.
//!
//! The page is `kanso_engine::export::page`: one self-contained file, with
//! math, diagrams and drawings inline as SVG and attached images as data
//! URIs, and no scripts. On macOS a PDF is printed from that page by
//! WebKit, offscreen, in pages; elsewhere the page opens in the browser, to
//! print to PDF from there.

#[cfg(not(target_os = "macos"))]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_engine::export::{file_name, page, print_palette};
use kanso_store::Store;

fn documents() -> PathBuf {
    let home = crate::paths::home();
    let documents = home.join("Documents");
    if documents.is_dir() { documents } else { home }
}

fn tell(window: &mut Window, cx: &mut App, notification: Notification) {
    window.push_notification(notification, cx);
}

/// Asks where to save the page, then writes it there and shows it.
pub fn export_html(
    store: Arc<Store>,
    title: String,
    body: String,
    self_id: Option<String>,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = crate::preview_rich::palette(cx);
    let style = crate::export_style::for_palette(cx, &palette);
    let picked = cx.prompt_for_new_path(&documents(), Some(&file_name(&title, "html")));
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(path))) = picked.await else {
                return;
            };
            let written = cx
                .background_executor()
                .spawn({
                    let path = path.clone();
                    let self_id = self_id.clone();
                    let style = style.clone();
                    async move {
                        std::fs::write(
                            &path,
                            page(&store, &title, &body, &palette, self_id.as_deref(), &style),
                        )
                    }
                })
                .await;
            cx.update(|window, cx| match written {
                Ok(()) => {
                    crate::metrics::record(cx, crate::metrics::Feature::Export);
                    cx.reveal_path(&path);
                }
                Err(err) => tell(
                    window,
                    cx,
                    Notification::error(format!("Couldn't save the page: {err}")),
                ),
            })
            .ok();
        })
        .detach();
}

/// Makes a PDF of the page: on macOS saved where asked, by WebKit; elsewhere
/// the page opens in the browser, to print to PDF.
///
/// Paper is printed on white, so a dark preview theme exports in Kanso's
/// light colors: printing leaves out backgrounds, and light text would be
/// lost on the page.
pub fn export_pdf(
    store: Arc<Store>,
    title: String,
    body: String,
    self_id: Option<String>,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = print_palette(crate::preview_rich::palette(cx));
    let style = crate::export_style::for_palette(cx, &palette);
    #[cfg(target_os = "macos")]
    {
        let picked = cx.prompt_for_new_path(&documents(), Some(&file_name(&title, "pdf")));
        window
            .spawn(cx, async move |cx| {
                let Ok(Ok(Some(path))) = picked.await else {
                    return;
                };
                let html =
                    cx.background_executor()
                        .spawn({
                            let self_id = self_id.clone();
                            let style = style.clone();
                            async move {
                                page(&store, &title, &body, &palette, self_id.as_deref(), &style)
                            }
                        })
                        .await;
                let result = mac_pdf(&html, &path, cx).await;
                cx.update(|window, cx| match result {
                    Ok(()) => {
                        crate::metrics::record(cx, crate::metrics::Feature::Export);
                        cx.reveal_path(&path);
                    }
                    Err(err) => tell(
                        window,
                        cx,
                        Notification::error(format!("Couldn't make the PDF: {err}")),
                    ),
                })
                .ok();
            })
            .detach();
    }
    #[cfg(not(target_os = "macos"))]
    {
        window
            .spawn(cx, async move |cx| {
                let written = cx
                    .background_executor()
                    .spawn(async move {
                        let dir = std::env::temp_dir().join("kanso-export");
                        std::fs::create_dir_all(&dir)?;
                        let path = dir.join(file_name(&title, "html"));
                        std::fs::write(
                            &path,
                            page(&store, &title, &body, &palette, self_id.as_deref(), &style),
                        )?;
                        Ok::<_, std::io::Error>(path)
                    })
                    .await;
                cx.update(|window, cx| match written {
                    Ok(path) => {
                        crate::metrics::record(cx, crate::metrics::Feature::Export);
                        open_in_browser(&path, cx);
                        tell(
                            window,
                            cx,
                            Notification::info(
                                "The note opened in your browser. Print it and choose Save as PDF.",
                            ),
                        );
                    }
                    Err(err) => tell(
                        window,
                        cx,
                        Notification::error(format!("Couldn't make the page: {err}")),
                    ),
                })
                .ok();
            })
            .detach();
    }
}

/// How often, and how many times, to look in on WebKit: every 50 ms, for up
/// to a minute.
#[cfg(target_os = "macos")]
const POLL: std::time::Duration = std::time::Duration::from_millis(50);
#[cfg(target_os = "macos")]
const POLLS: usize = 1200;

/// Prints `html` to a paged PDF at `path`, on the user's default paper:
/// WebKit's print operation, run offscreen with no panels. If printing
/// fails, the PDF is made by createPDF instead, as one long page, so there
/// is still a PDF.
#[cfg(target_os = "macos")]
async fn mac_pdf(
    html: &str,
    path: &std::path::Path,
    cx: &mut AsyncWindowContext,
) -> Result<(), String> {
    let closed = |_| "the window closed".to_string();
    let page = cx.update(|_, _| mac::Page::load(html)).map_err(closed)??;
    let mut loaded = false;
    for _ in 0..POLLS {
        cx.background_executor().timer(POLL).await;
        if !cx.update(|_, _| page.is_loading()).unwrap_or(false) {
            loaded = true;
            break;
        }
    }
    if !loaded {
        return Err("the page took too long to lay out".into());
    }
    // In pages, by printing.
    std::fs::remove_file(path).ok();
    let (done, finished) = async_channel::bounded(1);
    let printed = match cx.update(|_, _| page.print(path, done)).map_err(closed)? {
        Ok(()) => {
            let mut success = false;
            for _ in 0..POLLS {
                if let Ok(answer) = finished.try_recv() {
                    success = answer;
                    break;
                }
                cx.background_executor().timer(POLL).await;
            }
            success && std::fs::metadata(path).is_ok_and(|m| m.len() > 0)
        }
        Err(err) => {
            eprintln!("kanso: couldn't print the PDF: {err}");
            false
        }
    };
    if printed {
        return Ok(());
    }
    // One long page, rather than none.
    eprintln!("kanso: printing the PDF failed; making it as one page");
    let (done, finished) = async_channel::bounded(1);
    cx.update(|_, _| page.create_pdf(path.to_path_buf(), done))
        .map_err(closed)?;
    finished
        .recv()
        .await
        .unwrap_or_else(|_| Err("WebKit stopped".into()))
}

#[cfg(not(target_os = "macos"))]
fn open_in_browser(path: &Path, cx: &mut App) {
    cx.open_with_system(path);
}

#[cfg(target_os = "macos")]
mod mac;
