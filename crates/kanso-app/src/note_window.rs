//! A note in a window of its own: its header band, title, text and footer,
//! with no sidebar, list or inspector. "Open in New Window" in the note's
//! menu, the palette, ⌘⇧O and an ⌥-click on its row in the list open one;
//! a note already in a window of its own brings that window forward.
//!
//! The window edits the same note as the main window, live: what is typed
//! in one shows in the other at once, and only the one typed in saves
//! (`editor_peers.rs`). Closing it saves the note and leaves the app
//! running; quitting closes it with the rest. A note moved to the trash
//! shows the trash's banner here, and one deleted for good closes its
//! window.
//!
//! The windows open at launch where they were left
//! (`WindowLayout::note_windows`), moved onto a display if that one has
//! gone. What the note window can't do itself (the palette, a new note, the
//! inspector) it does in the main window.

use std::sync::Arc;

use gpui_kit::base::GlobalState;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{ActiveTheme as _, WindowExt as _};
use gpui_kit::*;
use kanso_store::Store;

use crate::config::{NoteWindowFrame, ViewMode};
use crate::editor::{EditorEvent, NoteEditor};
use crate::layout::{BandAction, BandDrag};
use crate::library::{Library, LibraryEvent};
use crate::note_menu::NoteCommand;
use crate::theme::Colors;
use crate::workspace::{
    AppState, CopyKansoLink, CopyNoteLink, Dictate, ExportNote, FocusEditor, FormatTable,
    NewDrawing, NewNote, OpenCommandPalette, OpenNotePicker, PastePlainText, SaveNote,
    SearchAllNotes, ShowBacklinks, ShowEditor, ShowHistory, ShowPreview, ShowSplit, TogglePinned,
    TrashNote, Workspace,
};

actions!(kanso, [OpenInNewWindow, CloseNoteWindow]);

/// A new note window's size, in points: about a readable column of text
/// beside its margins.
pub const WINDOW_SIZE: (f32, f32) = (780., 820.);
/// The smallest a note window gets: the header band's controls in a row,
/// and a few lines of text.
pub const MIN_SIZE: (f32, f32) = (480., 360.);
/// How far each new note window sits from the window it opened from, so it
/// doesn't cover that window exactly.
const CASCADE: f32 = 28.;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // ⌘⇧N is taken by New Notebook; O for open.
        KeyBinding::new("secondary-shift-o", OpenInNewWindow, None),
        KeyBinding::new("secondary-w", CloseNoteWindow, Some("NoteWindow")),
    ]);
}

/// A window frame as `(x, y, width, height)` in points.
pub type Frame = (f32, f32, f32, f32);

/// Where a window with `frame` goes so that it is on screen: on the display
/// it overlaps most (the first display when it overlaps none), at least
/// `min` large unless the display is smaller, and no larger than the
/// display, moved in as little as it takes. A frame already on a display
/// stays as it is. With no displays known it only grows to `min`. Pure.
pub fn clamp_frame(frame: Frame, displays: &[Frame], min: (f32, f32)) -> Frame {
    let (x, y, w, h) = frame;
    let (w, h) = (w.max(min.0), h.max(min.1));
    let overlap = |d: &Frame| {
        let across = (x + w).min(d.0 + d.2) - x.max(d.0);
        let down = (y + h).min(d.1 + d.3) - y.max(d.1);
        across.max(0.) * down.max(0.)
    };
    let Some(display) = displays
        .iter()
        .copied()
        .enumerate()
        .max_by(|(ia, a), (ib, b)| {
            overlap(a)
                .total_cmp(&overlap(b))
                // The earlier display wins a tie.
                .then(ib.cmp(ia))
        })
        .map(|(_, d)| d)
    else {
        return (x, y, w, h);
    };
    let (dx, dy, dw, dh) = display;
    let (w, h) = (w.min(dw), h.min(dh));
    // Off every display: centred on the first.
    let (x, y) = if overlap(&display) <= 0. {
        (dx + (dw - w) / 2., dy + (dh - h) / 2.)
    } else {
        (x, y)
    };
    // Not `clamp`: rounding can leave the far bound a hair below the near.
    (x.min(dx + dw - w).max(dx), y.min(dy + dh - h).max(dy), w, h)
}

/// The frame a note window opened from a window at `from` gets: the same
/// height, its own width, down and to the right of it. Pure.
pub fn cascade(from: Frame) -> Frame {
    (
        from.0 + CASCADE,
        from.1 + CASCADE,
        WINDOW_SIZE.0,
        from.3.max(MIN_SIZE.1),
    )
}

fn frame_of(bounds: Bounds<Pixels>) -> Frame {
    (
        f32::from(bounds.origin.x),
        f32::from(bounds.origin.y),
        f32::from(bounds.size.width),
        f32::from(bounds.size.height),
    )
}

fn bounds_of(frame: Frame) -> Bounds<Pixels> {
    Bounds::new(
        point(px(frame.0), px(frame.1)),
        size(px(frame.2), px(frame.3)),
    )
}

/// The displays' frames, the main display first.
fn display_frames(cx: &App) -> Vec<Frame> {
    let mut displays = cx.displays();
    if let Some(primary) = cx.primary_display() {
        displays.sort_by_key(|d| d.id() != primary.id());
    }
    displays.iter().map(|d| frame_of(d.bounds())).collect()
}

/// Every note window open now, with its note.
pub fn note_windows(cx: &mut App) -> Vec<(AnyWindowHandle, Entity<NoteWindow>)> {
    let mut found = Vec::new();
    for handle in cx.windows() {
        let view = handle
            .update(cx, |root, _, cx| {
                let root = root.downcast::<gpui_kit::base::Root>().ok()?;
                root.read(cx).view().clone().downcast::<NoteWindow>().ok()
            })
            .ok()
            .flatten();
        if let Some(view) = view {
            found.push((handle, view));
        }
    }
    found
}

/// Opens note `id` in a window of its own, or brings forward the one it is
/// already in. `from` is the frame of the window it was asked for in, to
/// open beside it. Not done from inside a window's update: the windows
/// can't be looked through then.
pub fn open_note_window(id: &str, from: Option<Frame>, cx: &mut App) {
    if let Some((handle, _)) = note_windows(cx)
        .into_iter()
        .find(|(_, view)| view.read(cx).note_id() == id)
    {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .ok();
        return;
    }
    let store = cx.global::<AppState>().store.clone();
    if !matches!(store.note(id), Ok(Some(_))) {
        return;
    }
    let saved = cx
        .global::<AppState>()
        .config
        .layout
        .note_windows
        .iter()
        .find(|f| f.note_id == id)
        .map(|f| (f.x, f.y, f.width, f.height));
    let frame = saved.or(from.map(cascade));
    open_window(id.to_owned(), frame, store, cx);
}

/// Opens the note windows open when Kanso last quit, where they were. A
/// note deleted since is forgotten.
pub fn restore(cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    let frames = cx.global::<AppState>().config.layout.note_windows.clone();
    let (live, gone): (Vec<_>, Vec<_>) = frames
        .into_iter()
        .partition(|f| matches!(store.note(&f.note_id), Ok(Some(_))));
    if !gone.is_empty() {
        AppState::update_config(cx, |config| {
            config
                .layout
                .note_windows
                .retain(|f| !gone.iter().any(|g| g.note_id == f.note_id));
        });
    }
    for frame in live {
        let at = (frame.x, frame.y, frame.width, frame.height);
        open_window(frame.note_id, Some(at), store.clone(), cx);
    }
}

fn open_window(id: String, frame: Option<Frame>, store: Arc<Store>, cx: &mut App) {
    let bounds = match frame {
        Some(frame) => bounds_of(clamp_frame(frame, &display_frames(cx), MIN_SIZE)),
        None => Bounds::centered(None, size(px(WINDOW_SIZE.0), px(WINDOW_SIZE.1)), cx),
    };
    #[cfg(target_os = "macos")]
    let titlebar = TitlebarOptions {
        title: Some(crate::flavour::CURRENT.name.into()),
        appears_transparent: true,
        traffic_light_position: Some(point(
            px(crate::layout::TRAFFIC_LIGHTS.0),
            px(crate::layout::TRAFFIC_LIGHTS.1),
        )),
    };
    // Off macOS the system's title bar stays: Kanso's own menus are in the
    // main window.
    #[cfg(not(target_os = "macos"))]
    let titlebar = TitlebarOptions {
        title: Some(crate::flavour::CURRENT.name.into()),
        appears_transparent: false,
        traffic_light_position: None,
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(titlebar),
        app_owns_titlebar_drag: cfg!(target_os = "macos"),
        window_min_size: Some(size(px(MIN_SIZE.0), px(MIN_SIZE.1))),
        #[cfg(target_os = "linux")]
        app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
        ..Default::default()
    };
    let library = crate::workspace::main_workspace(cx)
        .map(|(_, workspace)| workspace.read(cx).library.clone())
        .unwrap_or_else(|| cx.new(|cx| Library::new(store, cx)));
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        crate::themes::apply(Some(window), cx);
        cx.new(|cx| NoteWindow::new(id, library, window, cx))
    });
    match opened {
        Ok((handle, view)) => {
            handle
                .update(cx, |_, window, cx| {
                    view.update(cx, |view, cx| view.remember_frame(window, cx));
                    let view = view.clone();
                    // Closed by the user: saved, and not opened again.
                    window.on_window_should_close(cx, move |_, cx| {
                        view.update(cx, |view, cx| {
                            view.editor.update(cx, |editor, cx| editor.flush(cx));
                            view.forget_frame(cx);
                        });
                        true
                    });
                })
                .ok();
        }
        Err(err) => eprintln!("kanso: couldn't open a note window: {err:#}"),
    }
}

/// Does `f` in the main window, brought forward; opens one when there is
/// none. Not from inside that window's update.
pub fn in_main_window(
    cx: &mut App,
    f: impl FnOnce(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
) {
    cx.defer(move |cx| {
        if crate::workspace::main_workspace(cx).is_none() {
            crate::open_main_window(cx);
        }
        let Some((handle, workspace)) = crate::workspace::main_workspace(cx) else {
            return;
        };
        handle
            .update(cx, |_, window, cx| {
                window.activate_window();
                workspace.update(cx, |workspace, cx| f(workspace, window, cx));
            })
            .ok();
    });
}

pub struct NoteWindow {
    note_id: String,
    pub(crate) editor: Entity<NoteEditor>,
    /// The window's own focus, so its shortcuts work when nothing inside
    /// has the keyboard.
    focus: FocusHandle,
    band_drag: BandDrag,
    /// The window title as last set.
    title: String,
    /// The note went: the window is closing.
    closing: bool,
    /// Draws stand-ins for the traffic lights in headless snapshots.
    #[cfg(test)]
    pub(crate) placeholder_traffic_lights: bool,
    _subscriptions: Vec<Subscription>,
    _links: Task<()>,
}

impl NoteWindow {
    pub fn new(
        note_id: String,
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.global::<AppState>();
        let store = state.store.clone();
        let appearance = state.config.appearance.clone();
        let (links_tx, links_rx) = async_channel::unbounded::<String>();
        let editor = cx.new(|cx| {
            let mut editor =
                NoteEditor::new(store, library.clone(), links_tx, &appearance, window, cx);
            editor.set_standalone(cx);
            editor.open(Some(&note_id), window, cx);
            editor.refresh_wiki(cx);
            editor
        });
        let focus = cx.focus_handle();
        let subscriptions = vec![
            cx.subscribe_in(&library, window, |this, _, event, window, cx| {
                if let LibraryEvent::Changed(change) = event {
                    if change.id != this.note_id {
                        if change.id.starts_with("note:") {
                            this.editor.update(cx, |editor, cx| {
                                editor.refresh_backlinks(cx);
                                editor.refresh_wiki(cx);
                            });
                        }
                        return;
                    }
                    let change = change.clone();
                    this.editor.update(cx, |editor, cx| {
                        editor.on_store_change(&change, window, cx);
                        editor.refresh_backlinks(cx);
                    });
                }
            }),
            cx.subscribe_in(&editor, window, |this, _, event, window, cx| {
                this.on_editor_event(event, window, cx);
            }),
            // The note went (deleted for good, with nothing unsaved): the
            // window goes with it.
            cx.observe_in(&editor, window, |this, editor, window, cx| {
                if editor.read(cx).note_id().is_none() && !editor.read(cx).asks_about_a_change() {
                    this.close_for_good(window, cx);
                }
            }),
            cx.observe_window_bounds(window, |this, window, cx| this.remember_frame(window, cx)),
            cx.observe_window_appearance(window, |_, window, cx| {
                if cx
                    .global::<AppState>()
                    .config
                    .appearance
                    .themes
                    .follow_system
                {
                    crate::themes::apply(Some(window), cx);
                }
            }),
            cx.on_focus_lost(window, |this, window, cx| window.focus(&this.focus, cx)),
        ];
        // Links in the preview open in the main window: this one shows only
        // its note.
        let links = cx.spawn_in(window, async move |_, cx| {
            while let Ok(link) = links_rx.recv().await {
                if cx
                    .update(|_, cx| {
                        in_main_window(cx, move |workspace, window, cx| {
                            workspace.follow_link(link, window, cx)
                        })
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        window.focus(&focus, cx);
        editor.update(cx, |editor, cx| editor.focus_body(window, cx));
        NoteWindow {
            note_id,
            editor,
            focus,
            band_drag: BandDrag::default(),
            title: String::new(),
            closing: false,
            #[cfg(test)]
            placeholder_traffic_lights: false,
            _subscriptions: subscriptions,
            _links: links,
        }
    }

    pub fn note_id(&self) -> &str {
        &self.note_id
    }

    fn on_editor_event(
        &mut self,
        event: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.note_id.clone();
        match event {
            EditorEvent::Notice(message) => {
                window.push_notification(Notification::error(message.clone()), cx);
            }
            EditorEvent::ShowHistory => self.show_history(window, cx),
            EditorEvent::Command(command) => self.run_command(*command, window, cx),
            // The palette lives in the main window; it picks for this note
            // there.
            EditorEvent::Pick(picker) => {
                let picker = *picker;
                self.editor.update(cx, |editor, cx| editor.flush(cx));
                in_main_window(cx, move |workspace, window, cx| {
                    workspace.pick_for_note(&id, picker, window, cx)
                });
            }
            EditorEvent::ShowBacklinks => self.show_backlinks(cx),
            EditorEvent::ToggleInspector => {}
            // This window's note is its own; nothing else shows it.
            EditorEvent::Closed(_) => {}
            EditorEvent::Renamed {
                id,
                old,
                new,
                links,
                notes,
            } => {
                let (id, old, new, links, notes) =
                    (id.clone(), old.clone(), new.clone(), *links, *notes);
                in_main_window(cx, move |workspace, window, cx| {
                    workspace.offer_relink(&id, &old, &new, links, notes, window, cx)
                });
            }
        }
    }

    /// A choice from the note's menu. What needs the main window (a copy
    /// to show in the list, a folder to export to) happens there.
    fn run_command(&mut self, command: NoteCommand, window: &mut Window, cx: &mut Context<Self>) {
        match command {
            NoteCommand::TogglePinned => self.editor.update(cx, |e, cx| e.toggle_pinned(cx)),
            NoteCommand::SetStatus(status) => {
                self.editor.update(cx, |e, cx| e.set_status(status, cx))
            }
            NoteCommand::Trash => self.editor.update(cx, |e, cx| e.trash(cx)),
            NoteCommand::ShowHistory => self.show_history(window, cx),
            NoteCommand::Publish => {
                crate::publish::open_publish(self.editor.clone(), window, cx);
            }
            NoteCommand::Share => {
                crate::share_sheet::open_share(self.editor.clone(), window, cx);
            }
            NoteCommand::Lock => {
                crate::locked::lock_open_note(self.editor.clone(), window, cx);
            }
            NoteCommand::RemoveLock => {
                crate::locked::remove_lock(self.editor.clone(), window, cx);
            }
            NoteCommand::OpenInNewWindow => window.activate_window(),
            NoteCommand::OpenInNewTab => {
                let id = self.note_id.clone();
                in_main_window(cx, move |workspace, window, cx| {
                    workspace.open_note_in_new_tab(&id, window, cx)
                });
            }
            NoteCommand::CopyLink | NoteCommand::CopyKansoLink => {
                self.editor.update(cx, |editor, cx| editor.flush(cx));
                let store = cx.global::<AppState>().store.clone();
                let kanso = matches!(command, NoteCommand::CopyKansoLink);
                crate::note_link::copy_link(&store, &self.note_id, kanso, window, cx);
            }
            NoteCommand::Duplicate | NoteCommand::Export => {
                self.editor.update(cx, |editor, cx| editor.flush(cx));
                let id = self.note_id.clone();
                in_main_window(cx, move |workspace, window, cx| {
                    workspace.run_note_command(&id, command, window, cx)
                });
            }
        }
    }

    fn show_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        crate::history::open_history(self.editor.clone(), window, cx);
    }

    /// The note's backlinks, in the main window's inspector.
    fn show_backlinks(&mut self, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let id = self.note_id.clone();
        in_main_window(cx, move |workspace, window, cx| {
            workspace.show_backlinks_of(&id, window, cx)
        });
    }

    fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_view_mode(mode, cx));
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    /// Keeps where the window is, to open it there next time.
    fn remember_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing || window.is_fullscreen() {
            return;
        }
        let (x, y, width, height) = frame_of(window.bounds());
        let frame = NoteWindowFrame {
            note_id: self.note_id.clone(),
            x,
            y,
            width,
            height,
        };
        let frames = &cx.global::<AppState>().config.layout.note_windows;
        if frames.contains(&frame) {
            return;
        }
        AppState::update_config_soon(cx, move |config| {
            let frames = &mut config.layout.note_windows;
            match frames.iter_mut().find(|f| f.note_id == frame.note_id) {
                Some(kept) => *kept = frame,
                None => frames.push(frame),
            }
        });
    }

    /// The window won't open again at the next launch.
    fn forget_frame(&mut self, cx: &mut Context<Self>) {
        self.closing = true;
        let id = self.note_id.clone();
        AppState::update_config(cx, |config| {
            config.layout.note_windows.retain(|f| f.note_id != id)
        });
    }

    /// Closes the window for a note that is gone.
    fn close_for_good(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }
        self.forget_frame(cx);
        // Not from inside this view's own update.
        window.defer(cx, |window, _| window.remove_window());
    }

    #[cfg(test)]
    fn render_placeholder_traffic_lights(&self) -> Option<impl IntoElement> {
        use crate::layout::TRAFFIC_LIGHTS;
        self.placeholder_traffic_lights.then(|| {
            div()
                .flex()
                .absolute()
                .left(px(TRAFFIC_LIGHTS.0))
                .top(px(TRAFFIC_LIGHTS.1))
                .gap(px(8.))
                .children(
                    [0xFF5F57, 0xFEBC2E, 0x28C840]
                        .map(|color| div().size(px(12.)).rounded_full().bg(gpui_kit::rgb(color))),
                )
        })
    }

    #[cfg(not(test))]
    fn render_placeholder_traffic_lights(&self) -> Option<Div> {
        None
    }
}

impl Render for NoteWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The window list and the Dock name the window after its note.
        let title = self
            .editor
            .read(cx)
            .current_note(cx)
            .map(|note| note.title)
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_owned());
        if title != self.title {
            window.set_window_title(&title);
            self.title = title;
        }
        let colors = Colors::get(cx);
        let editor = self.editor.clone();
        div()
            .size_full()
            .relative()
            .key_context("NoteWindow")
            .track_focus(&self.focus)
            .bg(colors.editor)
            .text_color(cx.theme().foreground)
            .capture_any_mouse_down(|_, _, cx| {
                if cx.has_global::<GlobalState>() {
                    GlobalState::reset_text_selection_suppression(cx);
                }
            })
            // Presses on the empty part of the top band move the window, as
            // in the main window.
            .on_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let claimed = window.default_prevented()
                    || (cx.has_global::<GlobalState>()
                        && GlobalState::is_text_selection_suppressed(cx));
                if event.button == MouseButton::Left && cfg!(target_os = "macos") {
                    let action = this
                        .band_drag
                        .down(event.position, event.click_count, claimed);
                    on_band(action, window);
                }
                window.prevent_default();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, _| {
                if this.band_drag.is_armed() {
                    let pressed = event.pressed_button == Some(MouseButton::Left);
                    on_band(this.band_drag.moved(event.position, pressed), window);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, _| this.band_drag.up()),
            )
            .on_action(cx.listener(|this, _: &CloseNoteWindow, window, cx| {
                // As the close button does: saved, and forgotten.
                this.editor.update(cx, |editor, cx| editor.flush(cx));
                this.forget_frame(cx);
                window.defer(cx, |window, _| window.remove_window());
            }))
            .on_action(cx.listener(|this, _: &SaveNote, _, cx| {
                this.editor.update(cx, |editor, cx| editor.flush(cx));
            }))
            .on_action(cx.listener(|this, _: &FocusEditor, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }))
            .on_action(cx.listener(|this, _: &ShowEditor, _, cx| {
                this.set_view_mode(ViewMode::Edit, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowSplit, _, cx| {
                this.set_view_mode(ViewMode::Split, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowPreview, _, cx| {
                this.set_view_mode(ViewMode::Preview, cx)
            }))
            .on_action(cx.listener(|this, _: &TogglePinned, _, cx| {
                this.editor.update(cx, |editor, cx| editor.toggle_pinned(cx));
            }))
            .on_action(cx.listener(|this, _: &TrashNote, _, cx| {
                this.editor.update(cx, |editor, cx| editor.trash(cx));
            }))
            .on_action(cx.listener(|this, _: &ShowHistory, window, cx| {
                this.show_history(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ShowBacklinks, _, cx| {
                this.show_backlinks(cx);
            }))
            .on_action(cx.listener(|this, _: &PastePlainText, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.paste_plain(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FormatTable, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.format_table(window, cx));
            }))
            .on_action(cx.listener(|this, _: &NewDrawing, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.insert_drawing(window, cx));
            }))
            .on_action(|_: &OpenInNewWindow, window, _| window.activate_window())
            .on_action(cx.listener(|this, _: &ExportNote, window, cx| {
                this.run_command(NoteCommand::Export, window, cx);
            }))
            .on_action(cx.listener(|this, _: &CopyNoteLink, window, cx| {
                this.run_command(NoteCommand::CopyLink, window, cx);
            }))
            .on_action(cx.listener(|this, _: &CopyKansoLink, window, cx| {
                this.run_command(NoteCommand::CopyKansoLink, window, cx);
            }))
            .on_action(cx.listener(|this, _: &Dictate, window, cx| {
                crate::voice::press(this.editor.clone(), window, cx);
            }))
            .on_action(cx.listener(
                |this, _: &crate::transcribe::TranscribeAudio, window, cx| {
                    let found = this.editor.read(cx).audio_source_on_caret_line(cx);
                    match found {
                        Some(source) => crate::transcribe::start(
                            this.editor.clone(), source, window, cx,
                        ),
                        None => window.push_notification(
                            Notification::info(
                                "Put the cursor on the line with a recording to transcribe it, or drop a recording into the note.",
                            ),
                            cx,
                        ),
                    }
                },
            ))
            // What only the main window has, it does there.
            .on_action(|_: &NewNote, _, cx| forward(NewNote, cx))
            .on_action(|_: &OpenCommandPalette, _, cx| forward(OpenCommandPalette, cx))
            .on_action(|_: &OpenNotePicker, _, cx| forward(OpenNotePicker, cx))
            .on_action(|_: &SearchAllNotes, _, cx| forward(SearchAllNotes, cx))
            .child(editor)
            .children(self.render_placeholder_traffic_lights())
    }
}

/// Dispatches `action` in the main window, brought forward.
fn forward(action: impl Action, cx: &mut App) {
    let action = action.boxed_clone();
    in_main_window(cx, move |workspace, window, cx| {
        let focus = workspace.focus.clone();
        focus.dispatch_action(action.as_ref(), window, cx);
    });
}

fn on_band(action: BandAction, window: &mut Window) {
    match action {
        BandAction::None => {}
        BandAction::StartMove => window.start_window_move(),
        BandAction::TitlebarDoubleClick => window.titlebar_double_click(),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{Frame, MIN_SIZE, WINDOW_SIZE, cascade, clamp_frame};

    const LAPTOP: Frame = (0., 0., 1512., 945.);
    const BESIDE: Frame = (1512., -200., 2560., 1440.);

    #[test]
    fn frames_land_on_a_display() {
        // On screen: unchanged.
        let on = (100., 80., 700., 600.);
        assert_eq!(clamp_frame(on, &[LAPTOP], MIN_SIZE), on);
        // The second display was unplugged: centred on the first.
        let gone = (2000., 100., 700., 600.);
        assert_eq!(
            clamp_frame(gone, &[LAPTOP], MIN_SIZE),
            (406., 172.5, 700., 600.)
        );
        // Still there: kept.
        assert_eq!(clamp_frame(gone, &[LAPTOP, BESIDE], MIN_SIZE), gone);
        // Hanging off the right edge: moved in.
        assert_eq!(
            clamp_frame((1300., 100., 700., 600.), &[LAPTOP], MIN_SIZE),
            (812., 100., 700., 600.)
        );
        // Too small: grown to the least size.
        assert_eq!(
            clamp_frame((10., 10., 100., 50.), &[LAPTOP], MIN_SIZE),
            (10., 10., MIN_SIZE.0, MIN_SIZE.1)
        );
        // Larger than the display: as large as it.
        assert_eq!(
            clamp_frame((0., 0., 4000., 3000.), &[LAPTOP], MIN_SIZE),
            LAPTOP
        );
        // No displays known (headless): only the least size.
        assert_eq!(
            clamp_frame((5., 6., 10., 10.), &[], MIN_SIZE),
            (5., 6., MIN_SIZE.0, MIN_SIZE.1)
        );
    }

    #[test]
    fn a_new_window_cascades() {
        assert_eq!(
            cascade((100., 50., 1440., 900.)),
            (128., 78., WINDOW_SIZE.0, 900.)
        );
    }

    fn gen_display() -> impl Strategy<Value = Frame> {
        (
            -3000.0f32..3000.,
            -2000.0f32..2000.,
            600.0f32..3000.,
            400.0f32..2000.,
        )
    }

    fn inside(frame: Frame, display: Frame) -> bool {
        let eps = 1e-2;
        frame.0 >= display.0 - eps
            && frame.1 >= display.1 - eps
            && frame.0 + frame.2 <= display.0 + display.2 + eps
            && frame.1 + frame.3 <= display.1 + display.3 + eps
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Wherever a window was left, it opens whole on one of the
        /// displays, at least the least size, and opening it again puts
        /// it in the same place.
        #[test]
        fn a_restored_frame_is_whole_on_a_display(
            frame in (-8000.0f32..8000., -8000.0f32..8000., 0.0f32..5000., 0.0f32..5000.),
            displays in prop::collection::vec(gen_display(), 1..4),
        ) {
            let placed = clamp_frame(frame, &displays, MIN_SIZE);
            prop_assert!(displays.iter().any(|d| inside(placed, *d)), "{placed:?}");
            prop_assert!(placed.2 >= MIN_SIZE.0 && placed.3 >= MIN_SIZE.1);
            prop_assert_eq!(clamp_frame(placed, &displays, MIN_SIZE), placed);
        }

        /// A window already whole on a display, at least the least size,
        /// stays exactly where it is.
        #[test]
        fn a_frame_on_screen_stays(
            display in gen_display(),
            share in (0.0f32..1., 0.0f32..1., 0.0f32..1., 0.0f32..1.),
        ) {
            let w = MIN_SIZE.0 + share.2 * (display.2 - MIN_SIZE.0);
            let h = MIN_SIZE.1 + share.3 * (display.3 - MIN_SIZE.1);
            let frame = (
                display.0 + share.0 * (display.2 - w),
                display.1 + share.1 * (display.3 - h),
                w,
                h,
            );
            prop_assert_eq!(clamp_frame(frame, &[display], MIN_SIZE), frame);
        }
    }
}
