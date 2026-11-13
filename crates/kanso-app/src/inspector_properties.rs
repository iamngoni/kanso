//! A note's properties at the top of the inspector: each key and value,
//! edited in place, and a row to add one.
//!
//! A value goes into the note's front matter through
//! [`kanso_core::properties::set_property`], and the editor takes the new
//! text as one undoable edit. A locked note that isn't open shows nothing.

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Icon, Sizable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::properties::{self, Value};
use kanso_core::schema::PropertyDef;

use crate::inspector::Inspector;
use crate::locked::LockView;
use crate::property_pick::{self, Pick};
use crate::theme::{Colors, section_label};
use crate::zoom::px;

use super::PAD_X;

/// The value being edited in place, by the row it is in.
pub(super) struct PropertyEdit {
    index: usize,
    /// The property the field was opened on: if the text changes under it,
    /// the index may name another property, and the edit is dropped.
    key: String,
    value: Value,
    input: Entity<InputState>,
    _event: Subscription,
}

/// A property being added: its key and its value.
pub(super) struct PropertyAdd {
    key: Entity<InputState>,
    value: Entity<InputState>,
    _events: [Subscription; 2],
}

impl Inspector {
    /// The open note's text, if the section should show: a note is open,
    /// and a locked one is open with this session's key.
    fn property_source(&self, cx: &App) -> Option<SharedString> {
        let editor = self.editor.read(cx);
        if editor.note_id().is_none() || matches!(editor.lock_view(), LockView::Closed(_)) {
            return None;
        }
        Some(editor.body_text(cx))
    }

    /// What the open note's notebook says its properties are.
    fn property_defs(&self, cx: &App) -> Vec<PropertyDef> {
        let Some(id) = self.editor.read(cx).note_id().map(str::to_owned) else {
            return Vec::new();
        };
        let book = match self.store.note(&id) {
            Ok(Some(note)) => note.book_id,
            _ => return Vec::new(),
        };
        self.store.notebook_properties(&book).unwrap_or_default()
    }

    /// The Properties section: key and value rows, the one being edited as
    /// a field, and Add property.
    pub(super) fn render_properties(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let body = self.property_source(cx)?;
        let colors = Colors::get(cx);
        let properties = properties::parse(&body);
        let defs = self.property_defs(cx);
        let members = self.property_members(cx);
        let section = section_label(div(), colors)
            .px(px(8.))
            .pt(px(10.))
            .pb(px(6.))
            .child("Properties");
        let mut list = v_flex()
            .id("properties")
            .debug_selector(|| "inspector-properties".into())
            .w_full()
            .flex_none()
            .px(px(PAD_X))
            .pt(px(2.))
            .pb(px(8.))
            .border_b_1()
            .border_color(colors.divider)
            .child(section);
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = std::collections::HashSet::new();
        for (key, _) in &properties {
            if !seen.insert(key.to_lowercase()) {
                duplicates.insert(key.to_lowercase());
            }
        }
        let mut with_picker = std::collections::HashSet::new();
        for (ix, (key, value)) in properties.iter().enumerate() {
            let duplicate = duplicates.contains(&key.to_lowercase());
            let def = defs.iter().find(|def| def.key.eq_ignore_ascii_case(key));
            let picker = property_pick::pick_for(def, value, &members);
            if picker.is_some() {
                with_picker.insert(ix);
            }
            list = list
                .child(self.property_row(ix, key, value, picker, duplicate, colors, window, cx));
        }
        self.pickers.retain(|ix, _| with_picker.contains(ix));
        list = list.child(self.add_property_row(properties.len(), colors, cx));
        Some(list.into_any_element())
    }

    #[allow(clippy::too_many_arguments)]
    fn property_row(
        &mut self,
        ix: usize,
        key: &str,
        value: &Value,
        picker: Option<Pick>,
        duplicate: bool,
        colors: &Colors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editing = self
            .property_edit
            .as_ref()
            .is_some_and(|edit| edit.index == ix);
        let key_view = div()
            .flex_none()
            .w(relative(0.42))
            .min_w_0()
            .truncate()
            .text_size(px(12.5))
            .text_color(colors.text_secondary)
            .child(key.to_owned());
        let value_view = if editing {
            let input = self
                .property_edit
                .as_ref()
                .map(|edit| edit.input.clone())
                .expect("editing has an input");
            div()
                .flex_1()
                .min_w_0()
                .child(Input::new(&input).small())
                .into_any_element()
        } else if let Some(picker) = picker {
            self.picker_view(ix, key, value, picker, window, cx)
        } else if value.is_raw() {
            // A value Kanso doesn't understand stays as written, and is
            // edited in the note itself.
            div()
                .id(ElementId::NamedInteger("property-raw".into(), ix as u64))
                .debug_selector({
                    let key = key.to_owned();
                    move || format!("property-{key}")
                })
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.5))
                .text_color(colors.text_muted)
                .child(crate::query_view_ui::cell_text(value))
                .tooltip(|window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new("Edit this value in the note.")
                        .build(window, cx)
                })
                .into_any_element()
        } else if let Value::Bool(on) = value {
            let key = key.to_owned();
            let debug_key = key.clone();
            let on = *on;
            let switch = Switch::new(ElementId::NamedInteger("property-switch".into(), ix as u64))
                .checked(on)
                .on_click(cx.listener(move |this, on: &bool, window, cx| {
                    let body = this.editor.read(cx).body_text(cx).to_string();
                    let next = properties::set_property(&body, &key, &Value::Bool(*on));
                    this.editor
                        .update(cx, |editor, cx| editor.set_body(next, window, cx));
                    cx.notify();
                }));
            div()
                .id(ElementId::NamedInteger("property-bool".into(), ix as u64))
                .debug_selector(move || format!("property-{debug_key}"))
                .flex_1()
                .min_w_0()
                .child(switch)
                .into_any_element()
        } else {
            let key = key.to_owned();
            let current = value.clone();
            let shown = if value.is_empty() {
                "Empty".to_owned()
            } else {
                crate::query_view_ui::cell_text(value)
            };
            let muted = value.is_empty();
            div()
                .id(ElementId::NamedInteger("property-value".into(), ix as u64))
                .debug_selector({
                    let key = key.clone();
                    move || format!("property-{key}")
                })
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.5))
                .when(muted, |el| el.text_color(colors.text_muted))
                .when(!muted, |el| el.text_color(colors.text))
                .cursor_pointer()
                .hover(|el| el.text_color(colors.link))
                .child(shown)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.edit_property(ix, &key, &current, window, cx);
                }))
                .into_any_element()
        };
        h_flex()
            .id(ElementId::NamedInteger("property-row".into(), ix as u64))
            .w_full()
            .min_w_0()
            .min_h(px(26.))
            .gap(px(8.))
            .rounded(px(5.))
            .hover(|el| el.bg(colors.row_hover))
            .child(key_view)
            .when(duplicate, |el| {
                el.child(
                    div()
                        .id(ElementId::NamedInteger(
                            "property-duplicate".into(),
                            ix as u64,
                        ))
                        .flex_none()
                        .child(
                            Icon::new(gpui_kit::assets::IconName::TriangleAlert)
                                .size(px(11.))
                                .text_color(colors.warning),
                        )
                        .tooltip(|window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(
                                "This key is written more than once. The last one is used.",
                            )
                            .build(window, cx)
                        }),
                )
            })
            .child(value_view)
            .into_any_element()
    }

    fn add_property_row(
        &mut self,
        ix: usize,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(add) = &self.property_add {
            return h_flex()
                .id("add-property")
                .w_full()
                .min_w_0()
                .gap(px(6.))
                .py(px(2.))
                .child(
                    div()
                        .flex_none()
                        .w(relative(0.42))
                        .min_w_0()
                        .child(Input::new(&add.key).small()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&add.value).small()),
                )
                .into_any_element();
        }
        h_flex()
            .id(ElementId::NamedInteger("add-property".into(), ix as u64))
            .debug_selector(|| "add-property".into())
            .w_full()
            .h(px(26.))
            .gap(px(6.))
            .pl(px(2.))
            .rounded(px(5.))
            .cursor_pointer()
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .hover(|el| el.bg(colors.row_hover).text_color(colors.text_secondary))
            .child(Icon::new(gpui_kit::assets::IconName::Plus).size(px(12.)))
            .child("Add property")
            .on_click(cx.listener(|this, _, window, cx| this.add_property(window, cx)))
            .into_any_element()
    }

    /// Starts editing `key`'s value, with `like` keeping its type.
    fn edit_property(
        &mut self,
        index: usize,
        key: &str,
        like: &Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Value");
            state.set_value(like.text(), window, cx);
            state
        });
        input.update(cx, |state, cx| state.focus(window, cx));
        let event = cx.subscribe_in(
            &input,
            window,
            move |this: &mut Inspector, input, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let text = input.read(cx).value().to_string();
                    let Some((index, key, value)) = this
                        .property_edit
                        .as_ref()
                        .map(|edit| (edit.index, edit.key.clone(), edit.value.clone()))
                    else {
                        return;
                    };
                    this.commit_property(index, &key, &value, &text, window, cx);
                }
            },
        );
        self.property_edit = Some(PropertyEdit {
            index,
            key: key.to_owned(),
            value: like.clone(),
            input,
            _event: event,
        });
        cx.notify();
    }

    /// Starts adding a property: a key and a value, Enter in either to save.
    fn add_property(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = cx.new(|cx| InputState::new(window, cx).placeholder("Property"));
        let value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        key.update(cx, |state, cx| state.focus(window, cx));
        let key_event = cx.subscribe_in(
            &key,
            window,
            |this: &mut Inspector, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.commit_add(window, cx);
                }
            },
        );
        let value_event = cx.subscribe_in(
            &value,
            window,
            |this: &mut Inspector, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.commit_add(window, cx);
                }
            },
        );
        self.property_add = Some(PropertyAdd {
            key,
            value,
            _events: [key_event, value_event],
        });
        cx.notify();
    }

    /// Saves `text` as the value of the row at `index`, keeping its type,
    /// and closes the field. Text that hasn't changed writes nothing. If the
    /// note changed under the field so the row no longer names `key` with
    /// `original`, the edit is dropped rather than written to another
    /// property.
    fn commit_property(
        &mut self,
        index: usize,
        key: &str,
        original: &Value,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.editor.read(cx).body_text(cx).to_string();
        let Some((found, like)) = properties::parse(&body).into_iter().nth(index) else {
            self.property_edit = None;
            cx.notify();
            return;
        };
        self.property_edit = None;
        if !found.eq_ignore_ascii_case(key) || like != *original {
            window.push_notification(
                Notification::info("That property changed while you were editing it."),
                cx,
            );
            cx.notify();
            return;
        }
        if text.trim() == like.text().trim() {
            cx.notify();
            return;
        }
        let value = properties::typed(text, Some(&like));
        let next = properties::set_property_at(&body, index, &value);
        if next != body {
            self.editor
                .update(cx, |editor, cx| editor.set_body(next, window, cx));
        }
        cx.notify();
    }

    /// Saves a value a picker picked for the row at `index`, under the same
    /// check as a typed one: the row must still name `key` and hold
    /// `original`, or the pick is dropped rather than written to another
    /// property. A value the row already holds writes nothing.
    pub(super) fn commit_value(
        &mut self,
        index: usize,
        key: &str,
        original: &Value,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.editor.read(cx).body_text(cx).to_string();
        let Some((found, like)) = properties::parse(&body).into_iter().nth(index) else {
            cx.notify();
            return;
        };
        if !found.eq_ignore_ascii_case(key) || like != *original {
            window.push_notification(
                Notification::info("That property changed while you were editing it."),
                cx,
            );
            cx.notify();
            return;
        }
        let next = properties::set_property_at(&body, index, &value);
        if next != body {
            self.editor
                .update(cx, |editor, cx| editor.set_body(next, window, cx));
        }
        cx.notify();
    }

    /// Saves the property being added.
    fn commit_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(add) = &self.property_add else {
            return;
        };
        let key = add.key.read(cx).value().trim().to_owned();
        let text = add.value.read(cx).value().to_string();
        self.property_add = None;
        if key.is_empty() {
            cx.notify();
            return;
        }
        let body = self.editor.read(cx).body_text(cx).to_string();
        let like = properties::parse(&body)
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&key))
            .map(|(_, value)| value);
        if like.as_ref().is_some_and(Value::is_raw) {
            // A raw key is kept as written; writing a typed value into it
            // would replace YAML Kanso doesn't understand.
            window.push_notification(
                Notification::info(format!("“{key}” is kept as written. Edit it in the note.")),
                cx,
            );
            cx.notify();
            return;
        }
        let value = properties::typed(&text, like.as_ref());
        let next = properties::set_property(&body, &key, &value);
        if next != body {
            self.editor
                .update(cx, |editor, cx| editor.set_body(next, window, cx));
        }
        cx.notify();
    }

    /// The properties as last drawn, for the tests.
    #[cfg(test)]
    pub(crate) fn properties_for_test(&self, cx: &App) -> Vec<(String, Value)> {
        self.property_source(cx)
            .map(|body| properties::parse(&body))
            .unwrap_or_default()
    }

    /// Edits a value as the section does, from the keyboard.
    #[cfg(test)]
    pub(crate) fn set_property_for_test(
        &mut self,
        key: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let parsed = properties::parse(&self.editor.read(cx).body_text(cx));
        let index = parsed
            .iter()
            .rposition(|(name, _)| name.eq_ignore_ascii_case(key));
        if let Some(index) = index {
            let (found, like) = &parsed[index];
            let found = found.clone();
            let like = like.clone();
            self.commit_property(index, &found, &like, text, window, cx);
        }
    }

    /// Edits the value at a row, as a click on that row and Enter do.
    #[cfg(test)]
    pub(crate) fn set_property_at_for_test(
        &mut self,
        index: usize,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let parsed = properties::parse(&self.editor.read(cx).body_text(cx));
        if let Some((key, like)) = parsed.get(index) {
            let key = key.clone();
            let like = like.clone();
            self.commit_property(index, &key, &like, text, window, cx);
        }
    }

    /// Opens the field at a row, as a click on it does.
    #[cfg(test)]
    pub(crate) fn open_property_for_test(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let parsed = properties::parse(&self.editor.read(cx).body_text(cx));
        if let Some((key, like)) = parsed.get(index) {
            let key = key.clone();
            let like = like.clone();
            self.edit_property(index, &key, &like, window, cx);
        }
    }

    /// Saves what was typed into the open field.
    #[cfg(test)]
    pub(crate) fn commit_open_property_for_test(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((index, key, value)) = self
            .property_edit
            .as_ref()
            .map(|edit| (edit.index, edit.key.clone(), edit.value.clone()))
        else {
            return;
        };
        self.commit_property(index, &key, &value, text, window, cx);
    }

    /// Adds a property as the section does, from the keyboard.
    #[cfg(test)]
    pub(crate) fn add_property_for_test(
        &mut self,
        key: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.add_property(window, cx);
        if let Some(add) = &self.property_add {
            add.key
                .update(cx, |state, cx| state.set_value(key, window, cx));
            add.value
                .update(cx, |state, cx| state.set_value(text, window, cx));
        }
        self.commit_add(window, cx);
    }
}
