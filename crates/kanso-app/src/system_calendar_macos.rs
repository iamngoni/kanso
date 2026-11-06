//! Read-only EventKit access for Overview. Event data stays on this Mac.

use std::sync::Mutex;

use block2::RcBlock;
use gpui_kit::*;
use kanso_engine::overview::{Attendee, Event};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_event_kit::{
    EKAuthorizationStatus, EKEntityType, EKEvent, EKEventAvailability, EKEventStore,
    EKParticipantStatus,
};
use objc2_foundation::{NSDate, NSString};

/// One system calendar and the local Overview switch for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CalendarChoice {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

fn event_store() -> Retained<EKEventStore> {
    // SAFETY: EventKit's event store has a public initializer.
    unsafe { EKEventStore::new() }
}

pub(crate) fn has_access() -> bool {
    // SAFETY: `Event` is the documented entity type for calendar events.
    unsafe {
        EKEventStore::authorizationStatusForEntityType(EKEntityType::Event)
            == EKAuthorizationStatus::FullAccess
    }
}

/// Ask macOS for read access after the Settings page has explained why.
pub(crate) fn request_access(done: impl FnOnce(bool) + Send + 'static) {
    let store = event_store();
    let done = Mutex::new(Some(done));
    let handler = RcBlock::new(
        move |granted: Bool, _error: *mut objc2_foundation::NSError| {
            let callback = done.lock().ok().and_then(|mut slot| slot.take());
            if let Some(callback) = callback {
                callback(granted.as_bool());
            }
        },
    );
    // SAFETY: EventKit copies the completion block before returning.
    unsafe { store.requestFullAccessToEventsWithCompletion(RcBlock::as_ptr(&handler)) };
}

fn disabled_key(id: &str) -> Retained<NSString> {
    NSString::from_str(&format!("overview.calendar.disabled.{id}"))
}

fn is_enabled(id: &str) -> bool {
    let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
    !defaults.boolForKey(&disabled_key(id))
}

/// Store only whether the person disabled a calendar; new system calendars
/// stay enabled by default, and identifiers remain local to this Mac.
pub(crate) fn set_enabled(id: &str, enabled: bool) {
    let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
    defaults.setBool_forKey(!enabled, &disabled_key(id));
}

/// The calendars macOS currently exposes, when permission has been granted.
pub(crate) fn calendars() -> Vec<CalendarChoice> {
    if !has_access() {
        return Vec::new();
    }
    let store = event_store();
    // SAFETY: EventKit returns the calendars that support event records.
    let calendars = unsafe { store.calendarsForEntityType(EKEntityType::Event) };
    calendars
        .iter()
        .map(|calendar| {
            // SAFETY: these are immutable getters on a live EventKit calendar.
            let (id, name) = unsafe {
                (
                    calendar.calendarIdentifier().to_string(),
                    calendar.title().to_string(),
                )
            };
            CalendarChoice {
                enabled: is_enabled(&id),
                id,
                name,
            }
        })
        .collect()
}

/// Draw Settings › Calendar using the app's normal Settings rows.
pub(crate) fn page(cx: &mut Context<crate::settings::SettingsView>) -> Vec<AnyElement> {
    use crate::zoom::px;
    use gpui_kit::assets::IconName as Lucide;
    use gpui_kit::component::Sizable as _;
    use gpui_kit::component::button::Button;
    use gpui_kit::component::switch::Switch;
    use gpui_kit::*;

    let colors = crate::theme::Colors::get(cx);
    if !has_access() {
        let row = crate::settings::Row::new(Lucide::CalendarDays, "System calendar")
            .detail(
                "Calendar events stay on this Mac. Kanso reads calendars and never changes them.",
            )
            .trailing(
                Button::new("calendar-allow")
                    .small()
                    .label("Allow access")
                    .on_click({
                        let view = cx.entity().downgrade();
                        move |_, _, cx| {
                            let (tx, rx) = async_channel::bounded(1);
                            request_access(move |granted| {
                                tx.try_send(granted).ok();
                            });
                            let view = view.clone();
                            cx.spawn(async move |cx| {
                                let _ = rx.recv().await;
                                cx.update(|cx| {
                                    view.update(cx, |_, cx| cx.notify()).ok();
                                });
                            })
                            .detach();
                        }
                    }),
            );
        return vec![
            crate::settings::section("Calendar", vec![row], colors),
            div()
                .mt(px(10.))
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child("Allow access to show system-calendar events in Overview. You can change permission in System Settings › Privacy & Security › Calendars.")
                .into_any_element(),
        ];
    }

    let rows = calendars()
        .into_iter()
        .map(|calendar| {
            let id = calendar.id.clone();
            crate::settings::Row::new(Lucide::CalendarDays, calendar.name).trailing(
                Switch::new(SharedString::from(format!("calendar-{}", id)))
                    .checked(calendar.enabled)
                    .on_click(move |enabled, _, cx| {
                        set_enabled(&id, *enabled);
                        cx.refresh_windows();
                    }),
            )
        })
        .collect();
    vec![
        crate::settings::section("System calendars", rows, colors),
        div()
            .mt(px(10.))
            .text_size(px(13.))
            .text_color(colors.text_secondary)
            .child("Google and Microsoft calendars appear here when they are already added to macOS. Events stay on this Mac; Kanso never changes them.")
            .into_any_element(),
    ]
}

/// EventKit occurrences in `[start_ms, end_ms)`, from enabled calendars only.
pub(crate) fn events(start_ms: i64, end_ms: i64) -> Vec<Event> {
    if !has_access() || start_ms >= end_ms {
        return Vec::new();
    }
    let store = event_store();
    // SAFETY: the dates are finite timestamps and EventKit owns the query.
    let (start, end) = (
        NSDate::dateWithTimeIntervalSince1970(start_ms as f64 / 1000.0),
        NSDate::dateWithTimeIntervalSince1970(end_ms as f64 / 1000.0),
    );
    // SAFETY: the predicate is created by the same EventKit store.
    let predicate =
        unsafe { store.predicateForEventsWithStartDate_endDate_calendars(&start, &end, None) };
    // SAFETY: the predicate came from this store and the result is read-only.
    let rows = unsafe { store.eventsMatchingPredicate(&predicate) };
    rows.iter()
        .filter_map(|row| event(&row))
        .filter(|event| is_enabled(&event.calendar_id))
        .collect()
}

fn event(row: &EKEvent) -> Option<Event> {
    // SAFETY: these getters read a live event owned by EventKit.
    let (
        uid,
        calendar,
        calendar_id,
        start,
        end,
        title,
        location,
        conference_url,
        attendees,
        all_day,
        free,
        declined,
    ) = unsafe {
        let calendar = row.calendar()?;
        let attendee_data = row.attendees().map(|people| {
            let declined = people.iter().any(|person| {
                person.isCurrentUser()
                    && person.participantStatus() == EKParticipantStatus::Declined
            });
            let attendees = people
                .iter()
                .filter_map(|person| {
                    let name = person.name()?.to_string();
                    let address = person.URL().absoluteString()?.to_string();
                    (!name.trim().is_empty()).then_some(Attendee {
                        name,
                        email: address
                            .strip_prefix("mailto:")
                            .unwrap_or_default()
                            .to_owned(),
                        page: None,
                    })
                })
                .collect::<Vec<_>>();
            (attendees, declined)
        });
        let (attendees, declined) = attendee_data.unwrap_or_default();
        (
            row.calendarItemExternalIdentifier()
                .or_else(|| row.eventIdentifier())?
                .to_string(),
            calendar.title().to_string(),
            calendar.calendarIdentifier().to_string(),
            row.startDate().timeIntervalSince1970(),
            row.endDate().timeIntervalSince1970(),
            row.title().to_string(),
            row.location()
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.URL()
                .and_then(|value| value.absoluteString())
                .map(|value| value.to_string())
                .unwrap_or_default(),
            attendees,
            row.isAllDay(),
            row.availability() == EKEventAvailability::Free,
            declined,
        )
    };
    Some(Event {
        uid,
        start: (start * 1000.0) as i64,
        end: (end * 1000.0) as i64,
        title,
        location,
        conference_url,
        attendees,
        calendar,
        calendar_id,
        color: String::new(),
        all_day,
        free,
        // EventKit does not expose the signed-in attendee's response through
        // the portable event object, so do not infer a decline from others.
        declined,
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    #[test]
    fn disabled_calendar_keys_are_namespaced() {
        assert_eq!(
            super::disabled_key("cal-123").to_string(),
            "overview.calendar.disabled.cal-123"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            cases: 128,
            ..ProptestConfig::default()
        })]

        #[test]
        fn calendar_switch_keys_do_not_collide(id in "[a-zA-Z0-9_-]{1,80}") {
            prop_assert!(super::disabled_key(&id).to_string().starts_with("overview.calendar.disabled."));
        }
    }
}
