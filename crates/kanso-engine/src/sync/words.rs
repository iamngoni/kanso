//! What sync says, in plain words: the sidebar's sync line, a problem's
//! label and what to do about it, the Account page's status and the choices
//! for how often sync runs (docs/ACCOUNTS.md › Problems). Pure: the desktop
//! and the phone say the same things, and times are named by the caller's
//! `when` and `resume` words.

use kanso_sync::{QuotaKind, SyncProblem};

use super::SyncState;
use super::model::{AccountState, StatusLine, Tone};
use crate::account::words::{quota_words, signed_out_sentence, step_label};

/// What a sync problem says in the sidebar (docs/ACCOUNTS.md › Problems):
/// a few fixed words for each; what to do is in Settings › Account and the
/// tooltip.
pub fn problem_label(problem: &SyncProblem) -> &'static str {
    match problem {
        SyncProblem::SignedOut(_) => "Signed out",
        SyncProblem::AccountDisabled => "Account turned off",
        SyncProblem::Paused { .. } => "Sync paused",
        SyncProblem::ServerBusy { .. } => "Server busy",
        SyncProblem::BackOff { .. } => "Syncing shortly",
        SyncProblem::QuotaExceeded(_) => "Account full",
        SyncProblem::ServerFull => "Server full",
        SyncProblem::Skipped { .. } => "Some items skipped",
        SyncProblem::UpdateKanso => "Update Kanso",
        SyncProblem::WrongKey => "Keys don't match",
        SyncProblem::ServerOutdated => "Server needs an update",
        SyncProblem::Offline => "Offline",
        SyncProblem::Other => "Couldn't sync",
    }
}

/// The sync line while signed out: quiet, and a click away from signing in.
pub const SIGN_IN_TO_SYNC: &str = "Set up sync";
/// A provider session has no data key yet; it cannot sync until the user
/// unlocks it and chooses how this library should be bound.
pub const WAITING_FOR_APPROVAL: &str = "Waiting for approval";

/// The sidebar's sync line: "Set up sync" while signed out, the step of
/// a running account task, "Synced just now", "Synced yesterday",
/// "Syncing…", "Sync off", or a problem's label. `when` says when an
/// instant was, as the editor's header says when a note was edited, so the
/// line and the list name a day alike.
pub fn sync_line(
    state: &SyncState,
    account: &AccountState,
    when: impl FnOnce(i64) -> String,
) -> (Tone, String) {
    match account {
        AccountState::SignedOut => (Tone::Quiet, SIGN_IN_TO_SYNC.into()),
        AccountState::SignedInLocked { .. } => (Tone::Quiet, WAITING_FOR_APPROVAL.into()),
        AccountState::SessionEnded { reason, .. } => (
            Tone::Warning,
            problem_label(&SyncProblem::SignedOut(*reason)).into(),
        ),
        AccountState::Busy(step) => (Tone::Busy, step_label(*step).into()),
        AccountState::SignedIn { .. } => match state {
            SyncState::Off => (Tone::Quiet, "Sync off".into()),
            SyncState::Syncing => (Tone::Busy, "Syncing…".into()),
            SyncState::Failed(problem)
            | SyncState::Synced {
                problem: Some(problem),
                ..
            } => (Tone::Warning, problem_label(problem).into()),
            SyncState::Synced {
                conflicts: 0,
                at_ms,
                ..
            } => (Tone::Good, format!("Synced {}", when(*at_ms))),
            SyncState::Synced { .. } => (Tone::Warning, "Conflict detected".into()),
        },
    }
}

/// What to do about a sync problem, as Settings › Account and the sidebar's
/// tooltip say it (docs/ACCOUNTS.md › Problems). `resume` names when a
/// pause ends, to follow "Sync resumes".
pub fn problem_help(problem: &SyncProblem, resume: impl FnOnce(i64) -> String) -> String {
    match problem {
        SyncProblem::SignedOut(reason) => format!(
            "{} Sign in again to keep syncing.",
            signed_out_sentence(*reason)
        ),
        SyncProblem::AccountDisabled => {
            "This account is turned off on the server. Ask the server's owner.".to_owned()
        }
        SyncProblem::Paused { until } => format!(
            "This account made a lot of changes today. Sync resumes {}.",
            resume(*until)
        ),
        SyncProblem::ServerBusy { until } => format!(
            "The server has taken all the changes it can for today. Sync resumes {}.",
            resume(*until)
        ),
        SyncProblem::BackOff { .. } => {
            "This device sent the server a lot of requests at once. Sync resumes in a moment."
                .to_owned()
        }
        SyncProblem::QuotaExceeded(QuotaKind::Server) | SyncProblem::ServerFull => {
            "The server is out of space. Ask its owner.".to_owned()
        }
        SyncProblem::QuotaExceeded(kind) => format!(
            "Your account is full ({}). Delete something or ask the server's owner for more space.",
            quota_words(*kind)
        ),
        SyncProblem::Skipped { count: 1 } => {
            "1 item on the server couldn't be read. It was skipped; your copy here is unchanged."
                .to_owned()
        }
        SyncProblem::Skipped { count } => format!(
            "{count} items on the server couldn't be read. They were skipped; your copies here are unchanged."
        ),
        SyncProblem::UpdateKanso => {
            "Another device uses a newer Kanso. Update this one to keep syncing.".to_owned()
        }
        SyncProblem::WrongKey => "The server's account keys don't match this device's. Sync is stopped; your notes are safe here. Sign out and in again.".to_owned(),
        SyncProblem::ServerOutdated => {
            "That server doesn't run this version of Kanso sync.".to_owned()
        }
        SyncProblem::Offline => {
            "Can't reach the server. Your notes are safe on this device.".to_owned()
        }
        SyncProblem::Other => "Something went wrong. Try Sync now.".to_owned(),
    }
}

/// What the Account page's status says while signed out.
pub const NOT_SIGNED_IN: &str = "Sync is off";
pub const SIGNED_OUT_DETAIL: &str = "Your notes are only on this device. With a free account they sync to all your devices and are backed up, locked so only you can read them.";

/// The Account page's status: the sidebar's words ("Synced just now",
/// "Syncing…", a problem's label) and what they mean here, with `when`
/// naming the last sync's time as the sidebar does (see [`sync_line`]) and
/// `resume` the end of a pause (see [`problem_help`]). Signed out, it reads
/// "Sync is off". A conflict names the note it was in, by
/// `conflicted_title` when that is known. `enabled` and `interval_secs` are
/// the sync settings: whether sync runs on its own, and how often.
pub fn sync_status_line(
    state: &SyncState,
    account: &AccountState,
    enabled: bool,
    interval_secs: u64,
    when: impl FnOnce(i64) -> String,
    resume: impl FnOnce(i64) -> String,
    conflicted_title: Option<&str>,
) -> StatusLine {
    let (tone, label) = sync_line(state, account, when);
    let every = sync_every((interval_secs / 60).max(1));
    let (label, detail) = match account {
        AccountState::SignedOut => (NOT_SIGNED_IN.to_owned(), SIGNED_OUT_DETAIL.to_owned()),
        AccountState::SignedInLocked { .. } => (
            label,
            "Open Kanso on a device that's already signed in and approve this one.".to_owned(),
        ),
        AccountState::SessionEnded { reason, .. } => (
            label,
            problem_help(&SyncProblem::SignedOut(*reason), resume),
        ),
        AccountState::Busy(_) => (
            label,
            "This takes a few seconds. Your notes stay on this device.".to_owned(),
        ),
        AccountState::SignedIn { .. } => {
            let detail = match state {
                SyncState::Failed(problem)
                | SyncState::Synced {
                    problem: Some(problem),
                    ..
                } => problem_help(problem, resume),
                SyncState::Syncing => {
                    "Sending and fetching changes. Writing carries on as usual.".to_owned()
                }
                SyncState::Off | SyncState::Synced { conflicts: 0, .. } if enabled => {
                    format!("Kanso syncs {every} and soon after you edit.")
                }
                SyncState::Off => {
                    "Turn on Sync automatically, or sync once with Sync now.".to_owned()
                }
                SyncState::Synced { conflicts: 0, .. } => {
                    "Automatic sync is off; Sync now syncs again.".to_owned()
                }
                SyncState::Synced { conflicts: 1, .. } => match conflicted_title {
                    Some(title) => format!(
                        "“{title}” changed on two devices at once. Both versions are kept: the other one is in its history."
                    ),
                    None => "A note changed on two devices at once. Both versions are kept: the other one is in the note's history.".to_owned(),
                },
                SyncState::Synced { conflicts, .. } => match conflicted_title {
                    Some(title) => format!(
                        "“{title}” and {} other {} changed on two devices at once. Both versions of each are kept: the others are in their history.",
                        conflicts - 1,
                        if *conflicts == 2 { "note" } else { "notes" },
                    ),
                    None => format!(
                        "{conflicts} notes changed on two devices at once. Both versions of each are kept: the others are in their history."
                    ),
                },
            };
            (label, detail)
        }
    };
    StatusLine {
        tone,
        label,
        detail,
    }
}

/// The choices for how often sync runs, in minutes.
pub const SYNC_INTERVALS: [u64; 6] = [1, 5, 15, 30, 60, 240];

/// "1 minute", "15 minutes", "1 hour", "4 hours".
pub fn interval_label(minutes: u64) -> String {
    match minutes {
        1 => "1 minute".to_owned(),
        60 => "1 hour".to_owned(),
        m if m > 0 && m % 60 == 0 => format!("{} hours", m / 60),
        m => format!("{m} minutes"),
    }
}

/// "every minute", "every 15 minutes", "every hour".
pub fn sync_every(minutes: u64) -> String {
    match minutes {
        1 => "every minute".to_owned(),
        60 => "every hour".to_owned(),
        m => format!("every {}", interval_label(m)),
    }
}

#[cfg(test)]
mod tests {
    use kanso_sync::{QuotaKind, SignOutReason, SyncProblem};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;
    use crate::strategies;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// A time as the page names it, for the examples.
    fn at(ms: i64) -> String {
        format!("at {ms}")
    }

    fn signed_in() -> AccountState {
        AccountState::SignedIn {
            username: "ngoni".into(),
        }
    }

    // no-prop-test: a table of the problems' words, one row each.
    /// Every problem in the table has its help, as the table words it.
    #[test]
    fn every_sync_problem_has_help() {
        let cases = [
            (
                SyncProblem::SignedOut(SignOutReason::Revoked),
                "This device was signed out from another device. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Recovered),
                "Your account was recovered with its recovery key. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Evicted),
                "Signed out to make room for another device. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Expired),
                "Signed out after 180 days without syncing. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Unknown),
                "This device was signed out. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::AccountDisabled,
                "This account is turned off on the server. Ask the server's owner.",
            ),
            (
                SyncProblem::Paused { until: 1 },
                "This account made a lot of changes today. Sync resumes at 1.",
            ),
            (
                SyncProblem::QuotaExceeded(QuotaKind::DocBytes),
                "Your account is full (notes and notebooks). Delete something or ask the server's owner for more space.",
            ),
            (
                SyncProblem::QuotaExceeded(QuotaKind::Files),
                "Your account is full (number of attachments). Delete something or ask the server's owner for more space.",
            ),
            (
                SyncProblem::ServerFull,
                "The server is out of space. Ask its owner.",
            ),
            (
                SyncProblem::Skipped { count: 3 },
                "3 items on the server couldn't be read. They were skipped; your copies here are unchanged.",
            ),
            (
                SyncProblem::UpdateKanso,
                "Another device uses a newer Kanso. Update this one to keep syncing.",
            ),
            (
                SyncProblem::WrongKey,
                "The server's account keys don't match this device's. Sync is stopped; your notes are safe here. Sign out and in again.",
            ),
            (
                SyncProblem::ServerOutdated,
                "That server doesn't run this version of Kanso sync.",
            ),
            (
                SyncProblem::Offline,
                "Can't reach the server. Your notes are safe on this device.",
            ),
            (SyncProblem::Other, "Something went wrong. Try Sync now."),
        ];
        for (problem, help) in cases {
            assert_eq!(problem_help(&problem, at), help);
            let line = sync_status_line(
                &SyncState::Failed(problem),
                &signed_in(),
                true,
                300,
                at,
                at,
                None,
            );
            assert_eq!(line.detail, help);
        }
    }

    // no-prop-test: a table of the interval choices and their words.
    #[test]
    fn interval_examples() {
        assert_eq!(
            SYNC_INTERVALS.map(interval_label),
            [
                "1 minute",
                "5 minutes",
                "15 minutes",
                "30 minutes",
                "1 hour",
                "4 hours"
            ]
        );
        assert_eq!(sync_every(1), "every minute");
        assert_eq!(sync_every(15), "every 15 minutes");
        assert_eq!(sync_every(60), "every hour");
    }

    // no-prop-test: a handful of fixed states and the words they get.
    #[test]
    fn status_line_examples() {
        let synced = |conflicts| SyncState::Synced {
            at_ms: 5,
            conflicts,
            conflicted: Vec::new(),
            problem: None,
        };
        let line = |state: &SyncState, enabled: bool, title: Option<&str>| {
            sync_status_line(state, &signed_in(), enabled, 300, at, at, title)
        };
        let done = line(&synced(0), true, None);
        assert_eq!(
            (done.tone, done.label.as_str()),
            (Tone::Good, "Synced at 5")
        );
        assert_eq!(
            done.detail,
            "Kanso syncs every 5 minutes and soon after you edit."
        );
        assert_eq!(
            line(&synced(0), false, None).detail,
            "Automatic sync is off; Sync now syncs again."
        );
        assert_eq!(
            line(&SyncState::Off, false, None).detail,
            "Turn on Sync automatically, or sync once with Sync now."
        );
        assert_eq!(
            line(&synced(1), true, Some("Plans")).detail,
            "“Plans” changed on two devices at once. Both versions are kept: the other one is in its history."
        );
        assert_eq!(
            line(&synced(2), true, Some("Plans")).detail,
            "“Plans” and 1 other note changed on two devices at once. Both versions of each are kept: the others are in their history."
        );
        assert_eq!(
            line(&synced(3), true, None).detail,
            "3 notes changed on two devices at once. Both versions of each are kept: the others are in their history."
        );
        let out = sync_status_line(
            &SyncState::Off,
            &AccountState::SignedOut,
            true,
            300,
            at,
            at,
            None,
        );
        assert_eq!(
            (out.tone, out.label.as_str(), out.detail.as_str()),
            (Tone::Quiet, NOT_SIGNED_IN, SIGNED_OUT_DETAIL)
        );
    }

    proptest! {
        #![proptest_config(config())]

        /// Every problem's help is a sentence of its own, and a pause (the
        /// account's or the server's) names when it ends.
        #[test]
        fn prop_problem_help_says_what_to_do(problem in strategies::problem()) {
            let help = problem_help(&problem, |at| format!("at zq{at}"));
            prop_assert!(help.ends_with('.'), "{}", help);
            prop_assert_eq!(
                help.contains("zq"),
                matches!(problem, SyncProblem::Paused { .. } | SyncProblem::ServerBusy { .. }),
                "{}",
                help
            );
            if let SyncProblem::SignedOut(_) = problem {
                prop_assert!(help.ends_with("Sign in again to keep syncing."), "{}", help);
            }
        }

        /// The status says what the sidebar says, with the same color,
        /// except that signed out it reads "Sync is off"; and it promises
        /// syncing on its own only when it will.
        #[test]
        fn prop_sync_status_agrees_with_the_sidebar(
            state in strategies::sync_state(),
            account in strategies::account(),
            enabled: bool,
            minutes in 1u64..2000,
            ago in 0i64..100_000_000,
        ) {
            let now = match &state { SyncState::Synced { at_ms, .. } => at_ms + ago, _ => ago };
            let line = sync_status_line(&state, &account, enabled, minutes * 60, |_| at(now), at, None);
            let (tone, label) = sync_line(&state, &account, |_| at(now));
            prop_assert_eq!(line.tone, tone);
            if account == AccountState::SignedOut {
                prop_assert_eq!(line.label.as_str(), NOT_SIGNED_IN);
                prop_assert_eq!(line.detail.as_str(), SIGNED_OUT_DETAIL);
            } else {
                prop_assert_eq!(line.label, label);
            }
            prop_assert!(!line.detail.is_empty());
            let promises = line.detail.contains("Kanso syncs");
            let signed_in = matches!(account, AccountState::SignedIn { .. });
            prop_assert!(!promises || (enabled && signed_in), "{}", line.detail);
            let settled = matches!(
                state,
                SyncState::Off | SyncState::Synced { conflicts: 0, problem: None, .. }
            );
            if settled && enabled && signed_in {
                prop_assert!(promises && line.detail.contains(&sync_every(minutes)), "{}", line.detail);
            }
            // A problem's help shows exactly while signed in with it.
            if signed_in
                && let SyncState::Failed(problem) | SyncState::Synced { problem: Some(problem), .. } = &state
            {
                prop_assert_eq!(line.detail, problem_help(problem, at));
            }
        }

        /// Round-trip: an interval's label reads back as the same number of
        /// minutes, singular exactly for one.
        #[test]
        fn prop_interval_labels_read_back(minutes in 1u64..100_000) {
            let label = interval_label(minutes);
            let (count, unit) = label.split_once(' ').expect("a count and a unit");
            let count: u64 = count.parse().expect("a number");
            let back = match unit {
                "minute" | "minutes" => count,
                "hour" | "hours" => count * 60,
                other => panic!("unit {other}"),
            };
            prop_assert_eq!(back, minutes);
            prop_assert_eq!(unit.ends_with('s'), count != 1);
        }

        /// The problems table: every problem has its own label, a failed
        /// sync and a sync that finished with the problem read alike, with a
        /// warning, and signed out reads "Set up sync" whatever the last
        /// state was.
        #[test]
        fn prop_sync_problems_read_from_the_table(problem in strategies::problem(), conflicts in 0usize..3, when in 0i64..1_000_000) {
            let account = signed_in();
            let label = problem_label(&problem);
            let failed = sync_line(&SyncState::Failed(problem.clone()), &account, at);
            prop_assert_eq!(failed.clone(), (Tone::Warning, label.to_owned()));
            let finished = SyncState::Synced { at_ms: when, conflicts, conflicted: Vec::new(), problem: Some(problem.clone()) };
            prop_assert_eq!(sync_line(&finished, &account, at), failed);
            for state in [SyncState::Failed(problem), finished] {
                prop_assert_eq!(
                    sync_line(&state, &AccountState::SignedOut, at),
                    (Tone::Quiet, SIGN_IN_TO_SYNC.to_owned())
                );
            }
        }
    }
}
