//! Board settings: the dialog behind the "…" on a board grouped by a
//! property. It renames, reorders, adds and removes the board's lanes
//! (`kanso_engine::board_settings`) and says whether the lane of notes with
//! no value shows while empty. Renaming or removing a lane rewrites the
//! property on the notes in it, so removing one that has notes asks where
//! they go first: "3 notes are in Shooting. Move them to: [Editing ▾]".

use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::board::property_board;
use kanso_engine::board_settings::{
    LaneEdit, SettingsUndo, apply_lane_edit, edit_words, notes_in_lane, removal_prompt,
};
use kanso_store::{Origin, Store, StoreError};

use crate::clock::Clock;
use crate::theme::{Colors, section_label};
use crate::zoom::px;

const DIALOG_WIDTH: f32 = 440.;

/// Told of each written edit: what undoes it, and the words for its notice.
type ChangeHandler = Rc<dyn Fn(SettingsUndo, String, &mut Window, &mut App)>;

/// A lane asked to be removed, while its notes wait for a place to go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Removal {
    pub lane: String,
    pub notes: usize,
    /// The lane they go to, or `None` for "no value".
    pub target: Option<String>,
}

pub struct BoardSettings {
    store: Arc<Store>,
    book_id: String,
    /// The property the board groups by.
    key: String,
    lanes: Vec<String>,
    show_unset: bool,
    /// A field per lane, holding its name.
    names: Vec<Entity<InputState>>,
    add: Entity<InputState>,
    removal: Option<Removal>,
    error: Option<String>,
    on_change: ChangeHandler,
    _subscriptions: Vec<Subscription>,
}

impl BoardSettings {
    fn new(
        store: Arc<Store>,
        book_id: String,
        on_change: ChangeHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let add = cx.new(|cx| InputState::new(window, cx).placeholder("Add a lane"));
        let added = cx.subscribe_in(
            &add,
            window,
            |this: &mut Self, input, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let text = input.read(cx).value().to_string();
                    this.add_lane(&text, window, cx);
                }
            },
        );
        let mut dialog = BoardSettings {
            store,
            book_id,
            key: String::new(),
            lanes: Vec::new(),
            show_unset: true,
            names: Vec::new(),
            add,
            removal: None,
            error: None,
            on_change,
            _subscriptions: vec![added],
        };
        dialog.refresh(window, cx);
        dialog
    }

    /// Reads the board again and makes a name field for each lane.
    pub(crate) fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(Some(group)) = property_board(&self.store, &self.book_id) else {
            self.lanes.clear();
            self.names.clear();
            return;
        };
        self.key = group.key;
        self.show_unset = group.show_unset;
        self.lanes = group.values;
        self.names.clear();
        self._subscriptions.truncate(1);
        for (ix, lane) in self.lanes.iter().enumerate() {
            let name = lane.clone();
            let input = cx.new(|cx| InputState::new(window, cx).default_value(name));
            let sub = cx.subscribe_in(
                &input,
                window,
                move |this: &mut Self, input, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        let text = input.read(cx).value().to_string();
                        this.rename(ix, &text, window, cx);
                    }
                },
            );
            self._subscriptions.push(sub);
            self.names.push(input);
        }
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn lanes(&self) -> &[String] {
        &self.lanes
    }

    #[cfg(test)]
    pub(crate) fn removal(&self) -> Option<&Removal> {
        self.removal.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Writes `edit`, tells the owner, and reads the board again.
    fn edit(&mut self, edit: LaneEdit, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.error = None;
        let now = Clock::now_ms(cx);
        match apply_lane_edit(&self.store, &self.book_id, &edit, now) {
            Ok(done) => {
                let words = edit_words(&edit, done.rewritten, &self.key);
                let on_change = self.on_change.clone();
                on_change(done.undo, words, window, cx);
                self.removal = None;
                self.refresh(window, cx);
                true
            }
            Err(err) => {
                self.error = Some(match err {
                    StoreError::Invalid(message) => message,
                    other => format!("Couldn't change the board: {other}"),
                });
                self.refresh(window, cx);
                false
            }
        }
    }

    /// Gives lane `ix` the name `text`, unless it has it already.
    pub(crate) fn rename(
        &mut self,
        ix: usize,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(from) = self.lanes.get(ix).cloned() else {
            return;
        };
        if text.trim() == from {
            return;
        }
        self.edit(
            LaneEdit::Rename {
                from,
                to: text.to_owned(),
            },
            window,
            cx,
        );
    }

    /// Moves lane `ix` one place up (`-1`) or down (`1`).
    pub(crate) fn move_lane(
        &mut self,
        ix: usize,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(to) = ix
            .checked_add_signed(delta)
            .filter(|&to| to < self.lanes.len())
        else {
            return;
        };
        self.edit(LaneEdit::Move { from: ix, to }, window, cx);
    }

    /// Adds a lane named `text` at the end.
    pub(crate) fn add_lane(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if text.trim().is_empty() {
            return;
        }
        if self.edit(LaneEdit::Add(text.to_owned()), window, cx) {
            self.add
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
    }

    /// Asks to remove lane `ix`: at once when it has no notes, else with
    /// the question of where its notes go.
    pub(crate) fn request_remove(
        &mut self,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(lane) = self.lanes.get(ix).cloned() else {
            return;
        };
        // A board needs one lane.
        if self.lanes.len() == 1 {
            return;
        }
        let group = match property_board(&self.store, &self.book_id) {
            Ok(Some(group)) => group,
            _ => return,
        };
        let notes = notes_in_lane(&self.store, &self.book_id, &group, &lane)
            .map(|ids| ids.len())
            .unwrap_or(0);
        if notes == 0 {
            self.edit(
                LaneEdit::Remove {
                    lane,
                    move_to: None,
                },
                window,
                cx,
            );
            return;
        }
        // The notes go to the next lane, or the one before the last.
        let target = self
            .lanes
            .get(ix + 1)
            .or_else(|| ix.checked_sub(1).and_then(|before| self.lanes.get(before)))
            .cloned();
        self.removal = Some(Removal {
            lane,
            notes,
            target,
        });
        cx.notify();
    }

    /// Chooses where the notes of the lane being removed go.
    pub(crate) fn set_target(&mut self, target: Option<String>, cx: &mut Context<Self>) {
        if let Some(removal) = &mut self.removal {
            removal.target = target;
            cx.notify();
        }
    }

    pub(crate) fn cancel_removal(&mut self, cx: &mut Context<Self>) {
        self.removal = None;
        cx.notify();
    }

    /// Removes the lane asked about, moving its notes where chosen.
    pub(crate) fn confirm_removal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(removal) = self.removal.clone() {
            self.edit(
                LaneEdit::Remove {
                    lane: removal.lane,
                    move_to: removal.target,
                },
                window,
                cx,
            );
        }
    }

    /// Shows or hides the lane of notes with no value while it is empty.
    pub(crate) fn set_show_unset(
        &mut self,
        show: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(Some(mut spec)) = self.store.notebook_board(&self.book_id) else {
            return;
        };
        spec.show_unset = show;
        match self
            .store
            .set_notebook_board(&self.book_id, Some(&spec), Origin::Ui)
        {
            Ok(_) => self.error = None,
            Err(err) => self.error = Some(format!("Couldn't change the board: {err}")),
        }
        self.refresh(window, cx);
    }

    fn lane_row(&self, ix: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let last = self.lanes.len() - 1;
        let only = self.lanes.len() == 1;
        let view = cx.entity();
        let (up, down, remove) = (view.clone(), view.clone(), view);
        h_flex()
            .gap(px(4.))
            .items_center()
            .child(div().flex_1().child(Input::new(&self.names[ix]).small()))
            .child(
                Button::new(("lane-up", ix))
                    .ghost()
                    .xsmall()
                    .icon(Lucide::ChevronUp)
                    .tooltip("Move up")
                    .disabled(ix == 0)
                    .on_click(move |_, window, cx| {
                        up.update(cx, |this, cx| this.move_lane(ix, -1, window, cx))
                    }),
            )
            .child(
                Button::new(("lane-down", ix))
                    .ghost()
                    .xsmall()
                    .icon(Lucide::ChevronDown)
                    .tooltip("Move down")
                    .disabled(ix == last)
                    .on_click(move |_, window, cx| {
                        down.update(cx, |this, cx| this.move_lane(ix, 1, window, cx))
                    }),
            )
            .child(
                Button::new(("lane-remove", ix))
                    .ghost()
                    .xsmall()
                    .icon(Lucide::Trash)
                    .tooltip("Remove lane")
                    .disabled(only)
                    .debug_selector(move || format!("lane-remove-{ix}"))
                    .on_click(move |_, window, cx| {
                        remove.update(cx, |this, cx| this.request_remove(ix, window, cx))
                    }),
            )
    }

    /// The question of where a removed lane's notes go.
    fn removal_panel(&self, removal: &Removal, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let others: Vec<String> = self
            .lanes
            .iter()
            .filter(|lane| **lane != removal.lane)
            .cloned()
            .collect();
        let none = format!("No {}", self.key);
        let shown = removal.target.clone().unwrap_or_else(|| none.clone());
        let chosen = removal.target.clone();
        let menu_view = view.clone();
        let (cancel, confirm) = (view.clone(), view);
        v_flex()
            .gap(px(8.))
            .p(px(10.))
            .rounded(px(8.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.list)
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .flex_wrap()
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(colors.text)
                            .child(removal_prompt(&removal.lane, removal.notes)),
                    )
                    .child(
                        Button::new("lane-move-to")
                            .outline()
                            .small()
                            .label(shown)
                            .dropdown_caret(true)
                            .debug_selector(|| "lane-move-to".into())
                            .dropdown_menu(move |menu, _, _| {
                                let menu = others.iter().fold(menu, |menu, lane| {
                                    let (view, target) = (menu_view.clone(), lane.clone());
                                    menu.item(
                                        PopupMenuItem::new(lane.clone())
                                            .checked(chosen.as_deref() == Some(lane.as_str()))
                                            .on_click(move |_, _, cx| {
                                                let target = target.clone();
                                                view.update(cx, |this, cx| {
                                                    this.set_target(Some(target), cx)
                                                })
                                            }),
                                    )
                                });
                                let view = menu_view.clone();
                                menu.item(
                                    PopupMenuItem::new(none.clone())
                                        .checked(chosen.is_none())
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| this.set_target(None, cx))
                                        }),
                                )
                            }),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .justify_end()
                    .child(
                        Button::new("lane-remove-cancel")
                            .small()
                            .label("Cancel")
                            .on_click(move |_, _, cx| {
                                cancel.update(cx, |this, cx| this.cancel_removal(cx))
                            }),
                    )
                    .child(
                        Button::new("lane-remove-confirm")
                            .small()
                            .primary()
                            .label("Remove lane")
                            .debug_selector(|| "lane-remove-confirm".into())
                            .on_click(move |_, window, cx| {
                                confirm.update(cx, |this, cx| this.confirm_removal(window, cx))
                            }),
                    ),
            )
    }
}

impl Render for BoardSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let rows: Vec<_> = (0..self.lanes.len())
            .map(|ix| self.lane_row(ix, cx))
            .collect();
        let removal = self.removal.clone();
        v_flex()
            .gap(px(10.))
            .child(section_label(div(), colors).child("Lanes"))
            .child(v_flex().gap(px(4.)).children(rows))
            .child(Input::new(&self.add).small())
            .children(removal.map(|removal| self.removal_panel(&removal, cx)))
            .child(
                Checkbox::new("board-show-unset")
                    .label(format!("Show cards without a {}", self.key))
                    .checked(self.show_unset)
                    .on_click(move |checked: &bool, window, cx| {
                        let on = *checked;
                        view.update(cx, |this, cx| this.set_show_unset(on, window, cx))
                    }),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.warning)
                        .child(error),
                )
            })
    }
}

/// Opens the dialog for notebook `book_id`'s board. `on_change` hears of
/// each edit with what undoes it. Returns the dialog's body.
pub fn open_dialog(
    store: Arc<Store>,
    book_id: String,
    window: &mut Window,
    cx: &mut App,
    on_change: impl Fn(SettingsUndo, String, &mut Window, &mut App) + 'static,
) -> Entity<BoardSettings> {
    let on_change: ChangeHandler = Rc::new(on_change);
    let dialog = cx.new(|cx| BoardSettings::new(store, book_id, on_change, window, cx));
    let view = dialog.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Board settings")
            .width(px(DIALOG_WIDTH))
            .child(view.clone())
            .footer(
                h_flex().w_full().justify_end().child(
                    Button::new("board-settings-done")
                        .small()
                        .primary()
                        .label("Done")
                        .debug_selector(|| "board-settings-done".into())
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                ),
            )
    });
    dialog
}
