//! Pieces the chat-like panels share: Ask and an agent session (issue #375).

use gpui_kit::component::h_flex;
use gpui_kit::*;

use crate::theme::Colors;
use crate::zoom::px;

/// Carries what a worker sends into the view `V` until `step` says it is
/// over (it returns true), or until the channel closes, when `closed` runs
/// so the view can settle. Ends when the view is gone.
pub fn watch<V: 'static, T: 'static>(
    rx: async_channel::Receiver<T>,
    cx: &mut Context<V>,
    mut step: impl FnMut(&mut V, T, &mut Context<V>) -> bool + 'static,
    closed: impl FnOnce(&mut V, &mut Context<V>) + 'static,
) -> Task<()> {
    cx.spawn(async move |this, cx| {
        while let Ok(update) = rx.recv().await {
            let finished = this
                .update(cx, |this, cx| step(this, update, cx))
                .unwrap_or(true);
            if finished {
                break;
            }
        }
        this.update(cx, |this, cx| closed(this, cx)).ok();
    })
}

/// A row that names a note and opens it on click.
pub fn note_row(id: (&'static str, u64), title: &str, colors: &Colors) -> Stateful<Div> {
    let title = if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.to_owned()
    };
    h_flex()
        .id(ElementId::NamedInteger(id.0.into(), id.1))
        .w_full()
        .px(px(6.))
        .py(px(4.))
        .rounded(px(5.))
        .cursor_pointer()
        .hover(|el| el.bg(colors.row_hover))
        .text_size(px(12.5))
        .text_color(colors.link)
        .child(div().truncate().child(title))
}
