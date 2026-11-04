//! Today's agenda: one list, ordered by time, that merges calendar events,
//! timed reminders, tasks due today and tasks that are overdue.
//!
//! Pure: it takes rows already built and puts them in order, folds the
//! events that are over into one line, and caps what is left.
//!
//! The order is total and stable. Overdue first, the oldest first. Then
//! everything with a time, by time, an event ahead of a reminder at the
//! same minute. Then tasks due today without a time. The cap never drops an
//! overdue row silently: what it cuts is counted in `more`.

use std::cmp::Ordering;

use super::density::AGENDA_CAP;
use super::model::{Agenda, AgendaKind, AgendaRow};

fn rank(row: &AgendaRow) -> u8 {
    match row.kind {
        AgendaKind::Overdue => 0,
        AgendaKind::Event | AgendaKind::Reminder => 1,
        AgendaKind::Due => {
            if row.at_min.is_some() {
                1
            } else {
                2
            }
        }
    }
}

/// The agenda's order: a total order over rows, with the row's key and
/// title as the last tie-breakers so equal rows never swap places.
pub fn order(a: &AgendaRow, b: &AgendaRow) -> Ordering {
    let (ra, rb) = (rank(a), rank(b));
    ra.cmp(&rb)
        .then_with(|| match ra {
            // The oldest first.
            0 => b.days_late.cmp(&a.days_late),
            1 => a.at_min.cmp(&b.at_min).then_with(|| {
                let event = |r: &AgendaRow| u8::from(r.kind != AgendaKind::Event);
                event(a).cmp(&event(b))
            }),
            _ => Ordering::Equal,
        })
        .then_with(|| a.title.cmp(&b.title))
        .then_with(|| a.key.cmp(&b.key))
        .then_with(|| a.note_id.cmp(&b.note_id))
        .then_with(|| a.line.cmp(&b.line))
        .then_with(|| b.now.cmp(&a.now))
        .then_with(|| a.free.cmp(&b.free))
        .then_with(|| a.context.cmp(&b.context))
        .then_with(|| a.due.cmp(&b.due))
        .then_with(|| a.end_min.cmp(&b.end_min))
        .then_with(|| a.is_loop.cmp(&b.is_loop))
}

/// An event that has finished at `now_min`.
fn is_over(row: &AgendaRow, now_min: u32) -> bool {
    row.kind == AgendaKind::Event && !row.now && row.end_min.is_some_and(|end| end <= now_min)
}

/// Builds the agenda from `rows` at `now_min` (minutes into the local
/// day): events that are over fold into `earlier`; the rest are ordered
/// and capped at [`AGENDA_CAP`].
pub fn build(rows: Vec<AgendaRow>, now_min: u32) -> Agenda {
    build_capped(rows, now_min, AGENDA_CAP)
}

pub fn build_capped(rows: Vec<AgendaRow>, now_min: u32, cap: usize) -> Agenda {
    let (mut earlier, mut ahead): (Vec<AgendaRow>, Vec<AgendaRow>) =
        rows.into_iter().partition(|r| is_over(r, now_min));
    earlier.sort_by(order);
    ahead.sort_by(order);
    let task_total = ahead.iter().filter(|r| r.is_task()).count() as u32;
    let mut more = 0u32;
    let mut more_tasks = 0u32;
    if ahead.len() > cap {
        for row in ahead.drain(cap..) {
            more += 1;
            more_tasks += u32::from(row.is_task());
        }
    }
    Agenda {
        earlier,
        rows: ahead,
        more,
        more_tasks,
        task_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn row(kind: AgendaKind, title: &str, at: Option<u32>, late: u32) -> AgendaRow {
        AgendaRow {
            kind,
            key: format!("{title}-{at:?}-{late}"),
            title: title.into(),
            context: String::new(),
            at_min: at,
            end_min: at.map(|a| a + 30),
            days_late: late,
            now: false,
            free: false,
            note_id: String::new(),
            line: 0,
            is_loop: false,
            raw: String::new(),
            due: String::new(),
            meeting: None,
            color: String::new(),
            event: None,
        }
    }

    fn arb_row() -> impl Strategy<Value = AgendaRow> {
        (
            prop_oneof![
                Just(AgendaKind::Overdue),
                Just(AgendaKind::Event),
                Just(AgendaKind::Reminder),
                Just(AgendaKind::Due)
            ],
            "[a-d]{1,3}",
            prop::option::of(0u32..1400),
            0u32..30,
            any::<bool>(),
        )
            .prop_map(|(kind, title, at, late, now)| {
                let at = match kind {
                    AgendaKind::Event | AgendaKind::Reminder => Some(at.unwrap_or(600)),
                    AgendaKind::Overdue => None,
                    AgendaKind::Due => at,
                };
                let late = if kind == AgendaKind::Overdue {
                    late.max(1)
                } else {
                    0
                };
                let mut r = row(kind, &title, at, late);
                r.now = now && kind == AgendaKind::Event;
                r
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// The order is total: sorting any permutation gives the same list.
        #[test]
        fn the_order_is_total_and_stable(mut rows in prop::collection::vec(arb_row(), 0..24), seed in any::<u64>()) {
            let mut a = rows.clone();
            a.sort_by(order);
            // A shuffle by a fixed rotation and reversal.
            let n = rows.len().max(1);
            rows.rotate_left((seed as usize) % n);
            rows.reverse();
            rows.sort_by(order);
            prop_assert_eq!(a, rows);
        }

        /// Nothing is lost: what is shown, folded and counted adds up, and
        /// an overdue row is never cut while a later one is shown.
        #[test]
        fn the_cap_and_the_fold_drop_nothing_silently(rows in prop::collection::vec(arb_row(), 0..30), now in 0u32..1440) {
            let total = rows.len();
            let overdue = rows.iter().filter(|r| r.kind == AgendaKind::Overdue).count();
            let agenda = build(rows, now);
            prop_assert!(agenda.rows.len() <= AGENDA_CAP);
            prop_assert_eq!(agenda.rows.len() + agenda.more as usize + agenda.earlier.len(), total);
            let shown_overdue = agenda.rows.iter().filter(|r| r.kind == AgendaKind::Overdue).count();
            if overdue > 0 {
                prop_assert_eq!(agenda.rows[0].kind, AgendaKind::Overdue);
            }
            // Overdue rows come first, so a cut overdue row means every
            // shown row is overdue.
            if shown_overdue < overdue {
                prop_assert_eq!(shown_overdue, agenda.rows.len());
            }
            prop_assert!(agenda.more_tasks <= agenda.more);
            // Only events fold into the earlier line.
            prop_assert!(agenda.earlier.iter().all(|r| r.kind == AgendaKind::Event));
        }
    }

    #[test]
    fn an_event_leads_a_reminder_at_the_same_minute() {
        let rows = vec![
            row(AgendaKind::Reminder, "Call", Some(660), 0),
            row(AgendaKind::Event, "Standup", Some(660), 0),
            row(AgendaKind::Due, "Untimed", None, 0),
            row(AgendaKind::Overdue, "Late", None, 2),
            row(AgendaKind::Overdue, "Later", None, 5),
        ];
        let agenda = build(rows, 600);
        let titles: Vec<&str> = agenda.rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["Later", "Late", "Standup", "Call", "Untimed"]);
    }

    #[test]
    fn past_events_fold_and_the_rest_is_capped_with_a_count() {
        let mut rows = vec![row(AgendaKind::Event, "Standup", Some(540), 0)];
        for i in 0..10 {
            rows.push(row(AgendaKind::Due, &format!("t{i}"), None, 0));
        }
        let agenda = build(rows, 600);
        assert_eq!(agenda.earlier.len(), 1);
        assert_eq!(agenda.rows.len(), 8);
        assert_eq!(
            (agenda.more, agenda.more_tasks, agenda.task_total),
            (2, 2, 10)
        );
    }
}
