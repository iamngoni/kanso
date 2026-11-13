//! The pickers in the inspector's Properties section: a dropdown for a
//! select or a person, a calendar for a date, a calendar with a time for a
//! date and time, and a zone dropdown beside it for a date and time with a
//! zone.
//!
//! Each row that has a picker keeps one control across draws. The control
//! shows the value the row had when it was last drawn (`synced`); a pick
//! writes through [`Inspector::write_picked`], which checks the row still
//! holds that value before writing, so a pick never lands on another
//! property when the note's front matter changed under it (a half-typed
//! block, a line added above).

use chrono::NaiveTime;
use gpui_kit::component::date_picker::{DatePicker, DatePickerEvent, DatePickerState, DateTime};
use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::time_field::{HourCycle, TimePrecision};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{IndexPath, Sizable as _, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::properties::Value;
use kanso_core::zoned;

use crate::inspector::Inspector;
use crate::property_pick::{self as pick, Pick};
use crate::zoom::px;

pub(crate) type Dropdown = SelectState<SearchableVec<SharedString>>;
pub(crate) type DropdownEvent = SelectEvent<SearchableVec<SharedString>>;

/// The time a new date-and-time value starts on when only a day is picked.
const DEFAULT_TIME: NaiveTime = match NaiveTime::from_hms_opt(9, 0, 0) {
    Some(time) => time,
    None => NaiveTime::MIN,
};

/// Rows above which a dropdown has a search box.
const SEARCH_ABOVE: usize = 8;

/// The control a row is edited with.
enum Control {
    Dropdown {
        state: Entity<Dropdown>,
        /// The rows it was made with: a different list means a new control.
        rows: Vec<String>,
    },
    Date {
        state: Entity<DatePickerState>,
    },
    Zoned {
        date: Entity<DatePickerState>,
        zone: Entity<Dropdown>,
    },
}

/// A row's picker, and what it last showed.
pub(super) struct RowPicker {
    /// The property the control was made for.
    key: String,
    pick: Pick,
    /// The value the row held when the control was last brought up to date:
    /// the original a pick is checked against.
    synced: Value,
    control: Control,
    _subscriptions: Vec<Subscription>,
}

/// The zone the device is in, when the tz database names it.
fn device_zone() -> Option<String> {
    iana_time_zone::get_timezone()
        .ok()
        .filter(|name| zoned::zone(name).is_some())
}

impl Inspector {
    /// The usernames a person property can take: the open shared note's
    /// people. Empty outside a shared note.
    pub(super) fn property_members(&self, cx: &App) -> Vec<String> {
        #[cfg(test)]
        if let Some(members) = &self.members_for_test {
            return members.clone();
        }
        self.editor
            .read(cx)
            .shared_comments()
            .map(|(_, _, members)| members.to_vec())
            .unwrap_or_default()
    }

    /// The picker for row `ix`, made or brought up to date with `value`.
    pub(super) fn picker_view(
        &mut self,
        ix: usize,
        key: &str,
        value: &Value,
        pick: Pick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // A dropdown made for other rows (the notebook's options changed, or
        // the value is one they never listed) is made again.
        let stale = self.pickers.get(&ix).is_none_or(|row| {
            row.key != key
                || row.pick != pick
                || match (&row.control, &pick) {
                    (
                        Control::Dropdown { rows, .. },
                        Pick::Select(options) | Pick::Person(options),
                    ) => *rows != pick::choices(options, value),
                    _ => false,
                }
        });
        if stale {
            let row = self.make_picker(ix, key, value, pick, window, cx);
            self.pickers.insert(ix, row);
        } else if let Some(row) = self.pickers.get_mut(&ix)
            && row.synced != *value
        {
            row.synced = value.clone();
            show(&row.control, value, window, cx);
        }
        let Some(row) = self.pickers.get(&ix) else {
            return div().into_any_element();
        };
        let selector = format!("property-{key}");
        let body = match &row.control {
            Control::Dropdown { state, .. } => Select::new(state)
                .small()
                .cleanable(true)
                .placeholder("Choose")
                .search_placeholder("Search")
                .into_any_element(),
            Control::Date { state } => DatePicker::new(state)
                .small()
                .cleanable(true)
                .placeholder("Pick a date")
                .into_any_element(),
            Control::Zoned { date, zone } => v_flex()
                .gap(px(3.))
                .child(
                    DatePicker::new(date)
                        .small()
                        .cleanable(true)
                        .placeholder("Pick a date and time"),
                )
                .child(
                    div()
                        .debug_selector({
                            let selector = format!("{selector}-zone");
                            move || selector.clone()
                        })
                        .child(
                            Select::new(zone)
                                .small()
                                .placeholder("Time zone")
                                .search_placeholder("Search time zones"),
                        ),
                )
                .into_any_element(),
        };
        // A zoned time reads as the person would say it, beside the field.
        let label = pick::zoned_label(&row.synced);
        div()
            .id(ElementId::NamedInteger("property-picker".into(), ix as u64))
            .debug_selector(move || selector.clone())
            .flex_1()
            .min_w_0()
            .py(px(2.))
            .when_some(label, |el, label| {
                el.tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
            })
            .child(body)
            .into_any_element()
    }

    fn make_picker(
        &mut self,
        ix: usize,
        key: &str,
        value: &Value,
        pick: Pick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> RowPicker {
        let mut subscriptions = Vec::new();
        let control = match &pick {
            Pick::Select(options) | Pick::Person(options) => {
                let rows = pick::choices(options, value);
                let state = dropdown(&rows, pick::position(&rows, value), window, cx);
                subscriptions.push(cx.subscribe_in(
                    &state,
                    window,
                    move |this: &mut Inspector, _, event: &DropdownEvent, window, cx| {
                        let DropdownEvent::Confirm(row) = event;
                        let value = pick::chosen(row.as_deref());
                        this.write_picked(ix, value, window, cx);
                    },
                ));
                Control::Dropdown { state, rows }
            }
            Pick::Date | Pick::DateTime | Pick::DateTimeZone => {
                let with_time = pick != Pick::Date;
                let state = cx.new(|cx| {
                    let state = DatePickerState::new(window, cx).date_format(if with_time {
                        "%-d %b %Y, %H:%M"
                    } else {
                        "%a %-d %b %Y"
                    });
                    if with_time {
                        state
                            .time_precision(TimePrecision::Minute)
                            .hour_cycle(HourCycle::H23)
                            .default_time(DEFAULT_TIME)
                    } else {
                        state
                    }
                });
                if pick == Pick::DateTimeZone {
                    let zone_name = pick::zone_of(value).or_else(device_zone);
                    let (rows, at) = pick::zone_rows(zone_name.as_deref());
                    let zone = dropdown(&rows, at, window, cx);
                    subscriptions.push(cx.subscribe_in(
                        &state,
                        window,
                        move |this: &mut Inspector, _, event: &DatePickerEvent, window, cx| {
                            let DatePickerEvent::Change(picked) = event;
                            let zone = this.zone_picked(ix, cx);
                            let value = match (picked.start(), zone) {
                                (Some(at), Some(zone)) => pick::zoned_value(at, &zone),
                                // A time with no zone to put it in is kept
                                // as a plain date and time: the zone dropdown
                                // finishes it.
                                (Some(at), None) => pick::datetime_value(at),
                                (None, _) => Value::Text(String::new()),
                            };
                            this.write_picked(ix, value, window, cx);
                        },
                    ));
                    subscriptions.push(cx.subscribe_in(
                        &zone,
                        window,
                        move |this: &mut Inspector, _, event: &DropdownEvent, window, cx| {
                            let DropdownEvent::Confirm(Some(zone)) = event else {
                                return;
                            };
                            this.write_zone(ix, zone, window, cx);
                        },
                    ));
                    show_date(&state, value, window, cx);
                    let row = Control::Zoned { date: state, zone };
                    return RowPicker {
                        key: key.to_owned(),
                        pick,
                        synced: value.clone(),
                        control: row,
                        _subscriptions: subscriptions,
                    };
                }
                subscriptions.push(cx.subscribe_in(
                    &state,
                    window,
                    move |this: &mut Inspector, _, event: &DatePickerEvent, window, cx| {
                        let DatePickerEvent::Change(picked) = event;
                        let value = match (picked.start(), with_time) {
                            (Some(at), true) => pick::datetime_value(at),
                            (Some(at), false) => pick::date_value(at),
                            (None, _) => Value::Text(String::new()),
                        };
                        this.write_picked(ix, value, window, cx);
                    },
                ));
                show_date(&state, value, window, cx);
                Control::Date { state }
            }
        };
        RowPicker {
            key: key.to_owned(),
            pick,
            synced: value.clone(),
            control,
            _subscriptions: subscriptions,
        }
    }

    /// The zone the row's dropdown is on.
    fn zone_picked(&self, ix: usize, cx: &App) -> Option<String> {
        let Control::Zoned { zone, .. } = &self.pickers.get(&ix)?.control else {
            return None;
        };
        zone.read(cx).selected_value().map(|zone| zone.to_string())
    }

    /// Writes a zone picked for row `ix`: the value's wall-clock time stays.
    /// With no time yet, the zone waits until one is picked.
    fn write_zone(&mut self, ix: usize, zone: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(synced) = self.pickers.get(&ix).map(|row| row.synced.clone()) else {
            return;
        };
        // A date and time that has no zone yet gets this one.
        let value = match &synced {
            Value::DateTime(_) => pick::naive_of(&synced).map(|at| pick::zoned_value(at, zone)),
            other => pick::with_zone(other, zone),
        };
        if let Some(value) = value {
            self.write_picked(ix, value, window, cx);
        }
    }

    /// Writes what a picker picked into row `ix`, if the row still holds
    /// what the picker showed.
    fn write_picked(
        &mut self,
        ix: usize,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((key, original)) = self
            .pickers
            .get(&ix)
            .map(|row| (row.key.clone(), row.synced.clone()))
        else {
            return;
        };
        self.commit_value(ix, &key, &original, value, window, cx);
    }

    /// The date picker a row has (a date, a date and time, or a date and
    /// time with a zone), for the tests.
    #[cfg(test)]
    pub(crate) fn date_picker_for_test(&self, ix: usize) -> Option<Entity<DatePickerState>> {
        match &self.pickers.get(&ix)?.control {
            Control::Date { state } | Control::Zoned { date: state, .. } => Some(state.clone()),
            Control::Dropdown { .. } => None,
        }
    }

    /// The dropdown a row has (a select or a person), for the tests.
    #[cfg(test)]
    pub(crate) fn dropdown_for_test(&self, ix: usize) -> Option<Entity<Dropdown>> {
        match &self.pickers.get(&ix)?.control {
            Control::Dropdown { state, .. } => Some(state.clone()),
            _ => None,
        }
    }

    /// The zone dropdown a row has, for the tests.
    #[cfg(test)]
    pub(crate) fn zone_picker_for_test(&self, ix: usize) -> Option<Entity<Dropdown>> {
        match &self.pickers.get(&ix)?.control {
            Control::Zoned { zone, .. } => Some(zone.clone()),
            _ => None,
        }
    }

    /// Gives a person property these people to pick from.
    #[cfg(test)]
    pub(crate) fn set_members_for_test(&mut self, members: Vec<String>) {
        self.members_for_test = Some(members);
    }
}

/// A dropdown over `rows`, on row `at`.
fn dropdown(
    rows: &[String],
    at: Option<usize>,
    window: &mut Window,
    cx: &mut Context<Inspector>,
) -> Entity<Dropdown> {
    let items = SearchableVec::new(
        rows.iter()
            .map(|row| SharedString::from(row.clone()))
            .collect::<Vec<_>>(),
    );
    let searchable = rows.len() > SEARCH_ABOVE;
    cx.new(|cx| SelectState::new(items, at.map(IndexPath::new), window, cx).searchable(searchable))
}

/// Brings a control up to date with the row's value.
fn show(control: &Control, value: &Value, window: &mut Window, cx: &mut Context<Inspector>) {
    match control {
        Control::Dropdown { state, rows } => {
            let at = pick::position(rows, value);
            state.update(cx, |state, cx| {
                state.set_selected_index(at.map(IndexPath::new), window, cx)
            });
        }
        Control::Date { state } => show_date(state, value, window, cx),
        Control::Zoned { date, zone } => {
            show_date(date, value, window, cx);
            if let Some(name) = pick::zone_of(value) {
                let (_, at) = pick::zone_rows(Some(&name));
                zone.update(cx, |state, cx| {
                    state.set_selected_index(at.map(IndexPath::new), window, cx)
                });
            }
        }
    }
}

/// Puts a date picker on `value`'s day and time, or on nothing.
fn show_date(
    state: &Entity<DatePickerState>,
    value: &Value,
    window: &mut Window,
    cx: &mut Context<Inspector>,
) {
    let wanted = DateTime::Single(pick::naive_of(value));
    state.update(cx, |state, cx| {
        if state.date_time() != wanted {
            state.set_date_time(wanted, window, cx);
        }
    });
}
