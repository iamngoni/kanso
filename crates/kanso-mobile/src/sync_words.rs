//! What sync says on the phone, in the desktop's words, and the settings
//! that decide when it runs.
//!
//! The status line is `kanso_engine::sync::words`, the code Settings ›
//! Account calls: [`SyncHandle::status_line`] turns a [`SyncStatus`] back
//! into the engine's own states and asks it. Times are the shell's to
//! format (it owns the locale): it passes the words for "when the last sync
//! was" and "when a pause ends" in.
//!
//! The settings are kept in the library's meta, like the device's name: sync
//! automatically (on by default) and how often (every 5 minutes by default),
//! from the desktop's choices.

use kanso_engine::sync::words::{SYNC_INTERVALS, interval_label, sync_every, sync_status_line};
use kanso_engine::sync::{AccountState, SyncState, Tone};
use kanso_sync::{QuotaKind, SignOutReason, Step, SyncProblem};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::model::{SyncPhase, SyncProblemKind, SyncStatus};
use crate::sync_model::{
    AccountStep, QuotaLimit, SignOutReasonKind, SyncAccount, SyncIntervalOption, SyncSettingsRow,
    SyncStatusLine, SyncTone,
};
use crate::syncing::SyncHandle;

/// Where "sync automatically" is kept.
const ENABLED: &str = "mobile.sync_enabled";
/// Where "how often", in seconds, is kept.
const INTERVAL_SECS: &str = "mobile.sync_interval_secs";

impl SyncHandle {
    /// The status line for `status`, in the words Settings › Account uses on
    /// the desktop: a tone, a label ("Synced just now") and a sentence that
    /// says what it means or what to do. `settings` are the sync settings
    /// ([`SyncHandle::settings`]). `when` is the shell's words for
    /// `status.last_synced_ms` ("just now", "3 min ago", "yesterday") to
    /// follow "Synced", and `resume` its words for `status.resume_at_ms`
    /// ("at 3:00 PM") to follow "Sync resumes".
    pub fn status_line(
        status: SyncStatus,
        settings: SyncSettingsRow,
        when: String,
        resume: String,
    ) -> SyncStatusLine {
        let line = sync_status_line(
            &state_of(&status),
            &account_of(&status),
            settings.enabled,
            u64::from(settings.interval_secs),
            |_| when,
            |_| resume,
            status.conflicted_title.as_deref(),
        );
        SyncStatusLine {
            tone: tone(line.tone),
            label: line.label,
            detail: line.detail,
        }
    }

    /// The choices for how often sync runs, with the desktop's words.
    pub fn interval_options() -> Vec<SyncIntervalOption> {
        SYNC_INTERVALS
            .iter()
            .map(|&minutes| SyncIntervalOption {
                minutes: u32::try_from(minutes).unwrap_or(u32::MAX),
                label: interval_label(minutes),
            })
            .collect()
    }

    /// "1 minute", "15 minutes", "1 hour", "4 hours".
    pub fn interval_label(minutes: u32) -> String {
        interval_label(u64::from(minutes))
    }

    /// "every minute", "every 15 minutes", "every hour".
    pub fn interval_phrase(minutes: u32) -> String {
        sync_every(u64::from(minutes))
    }

    /// Whether Kanso syncs on its own, and how often.
    pub fn settings(&self) -> Result<SyncSettingsRow, KansoError> {
        guard(|| self.stored_settings())
    }

    /// Sets whether Kanso syncs on its own and how often. `interval_secs`
    /// must be one of [`SyncHandle::interval_options`] in seconds. The
    /// schedule follows at once; Sync now works whatever these say, and a
    /// sync that is running finishes.
    pub fn set_settings(
        &self,
        enabled: bool,
        interval_secs: u32,
    ) -> Result<SyncSettingsRow, KansoError> {
        guard(|| {
            let row = SyncSettingsRow {
                enabled,
                interval_secs,
            };
            check_settings(&row)?;
            let store = &self.inner.store;
            store.set_meta(ENABLED, if enabled { "1" } else { "0" })?;
            store.set_meta(INTERVAL_SECS, &interval_secs.to_string())?;
            self.inner.auto.apply_settings(row);
            Ok(row)
        })
    }

    pub(crate) fn stored_settings(&self) -> Result<SyncSettingsRow, KansoError> {
        read_settings(&self.inner.store)
    }
}

/// The settings kept in `store`; what is missing or unreadable is the
/// default.
pub(crate) fn read_settings(store: &kanso_store::Store) -> Result<SyncSettingsRow, KansoError> {
    let default = SyncSettingsRow::default();
    let enabled = store.meta(ENABLED)?.map_or(default.enabled, |v| v != "0");
    let row = SyncSettingsRow {
        enabled,
        interval_secs: store
            .meta(INTERVAL_SECS)?
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.interval_secs),
    };
    Ok(if check_settings(&row).is_ok() {
        row
    } else {
        SyncSettingsRow {
            interval_secs: default.interval_secs,
            ..row
        }
    })
}

/// Refuses an interval the desktop doesn't offer.
pub(crate) fn check_settings(row: &SyncSettingsRow) -> Result<(), KansoError> {
    let offered = SYNC_INTERVALS
        .iter()
        .any(|minutes| minutes * 60 == u64::from(row.interval_secs));
    if offered {
        Ok(())
    } else {
        Err(KansoError::invalid(
            "Choose how often to sync from the list.",
        ))
    }
}

fn tone(tone: Tone) -> SyncTone {
    match tone {
        Tone::Good => SyncTone::Good,
        Tone::Busy => SyncTone::Busy,
        Tone::Quiet => SyncTone::Quiet,
        Tone::Warning => SyncTone::Warning,
    }
}

// ------------------------------------------------- status <-> the engine's

/// The engine's sync state a status says.
pub(crate) fn state_of(status: &SyncStatus) -> SyncState {
    let problem = problem_of(status);
    match status.phase {
        SyncPhase::SignedOut | SyncPhase::Idle => SyncState::Off,
        SyncPhase::Syncing => SyncState::Syncing,
        SyncPhase::Synced => SyncState::Synced {
            at_ms: status.last_synced_ms.unwrap_or(0),
            conflicts: status.conflicts as usize,
            conflicted: Vec::new(),
            problem,
        },
        SyncPhase::Failed => SyncState::Failed(problem.unwrap_or(SyncProblem::Other)),
    }
}

/// The engine's account state a status says.
pub(crate) fn account_of(status: &SyncStatus) -> AccountState {
    let username = || status.username.clone().unwrap_or_default();
    match status.account {
        SyncAccount::SignedOut => AccountState::SignedOut,
        SyncAccount::SignedIn => AccountState::SignedIn {
            username: username(),
        },
        SyncAccount::Locked => AccountState::SignedInLocked {
            username: username(),
        },
        SyncAccount::SessionEnded => AccountState::SessionEnded {
            username: username(),
            reason: status
                .signed_out_reason
                .map_or(SignOutReason::Unknown, reason_of),
        },
        SyncAccount::Busy(step) => AccountState::Busy(step_of(step)),
    }
}

/// The problem a status carries, rebuilt from its kind and the details the
/// kind needs.
pub(crate) fn problem_of(status: &SyncStatus) -> Option<SyncProblem> {
    let until = status.resume_at_ms.unwrap_or(0);
    Some(match status.problem? {
        SyncProblemKind::SignedOut => SyncProblem::SignedOut(
            status
                .signed_out_reason
                .map_or(SignOutReason::Unknown, reason_of),
        ),
        SyncProblemKind::AccountDisabled => SyncProblem::AccountDisabled,
        SyncProblemKind::Paused => SyncProblem::Paused { until },
        SyncProblemKind::ServerBusy => SyncProblem::ServerBusy { until },
        SyncProblemKind::BackOff => SyncProblem::BackOff { until },
        SyncProblemKind::AccountFull => {
            SyncProblem::QuotaExceeded(status.quota.map_or(QuotaKind::Docs, quota_of))
        }
        SyncProblemKind::ServerFull => SyncProblem::ServerFull,
        SyncProblemKind::SomeItemsSkipped => SyncProblem::Skipped {
            count: status.skipped_count as usize,
        },
        SyncProblemKind::UpdateKanso => SyncProblem::UpdateKanso,
        SyncProblemKind::KeysDontMatch => SyncProblem::WrongKey,
        SyncProblemKind::ServerNeedsUpdate => SyncProblem::ServerOutdated,
        SyncProblemKind::Offline => SyncProblem::Offline,
        SyncProblemKind::Other => SyncProblem::Other,
    })
}

/// A problem's details for the status, besides its kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProblemDetails {
    pub(crate) resume_at_ms: Option<i64>,
    pub(crate) skipped_count: u32,
    pub(crate) quota: Option<QuotaLimit>,
    pub(crate) signed_out_reason: Option<SignOutReasonKind>,
}

pub(crate) fn details_of(problem: &SyncProblem) -> ProblemDetails {
    let mut details = ProblemDetails::default();
    match problem {
        SyncProblem::Paused { until }
        | SyncProblem::ServerBusy { until }
        | SyncProblem::BackOff { until } => details.resume_at_ms = Some(*until),
        SyncProblem::Skipped { count } => {
            details.skipped_count = u32::try_from(*count).unwrap_or(u32::MAX);
        }
        SyncProblem::QuotaExceeded(kind) => details.quota = Some(quota_limit(*kind)),
        SyncProblem::SignedOut(reason) => details.signed_out_reason = Some(reason_kind(*reason)),
        _ => {}
    }
    details
}

pub(crate) fn reason_kind(reason: SignOutReason) -> SignOutReasonKind {
    match reason {
        SignOutReason::SignedOut => SignOutReasonKind::SignedOut,
        SignOutReason::Revoked => SignOutReasonKind::Revoked,
        SignOutReason::PasswordChanged => SignOutReasonKind::PasswordChanged,
        SignOutReason::Recovered => SignOutReasonKind::Recovered,
        SignOutReason::Evicted => SignOutReasonKind::Evicted,
        SignOutReason::Expired => SignOutReasonKind::Expired,
        SignOutReason::Rekeyed => SignOutReasonKind::Rekeyed,
        SignOutReason::Unknown => SignOutReasonKind::Unknown,
    }
}

fn reason_of(kind: SignOutReasonKind) -> SignOutReason {
    match kind {
        SignOutReasonKind::SignedOut => SignOutReason::SignedOut,
        SignOutReasonKind::Revoked => SignOutReason::Revoked,
        SignOutReasonKind::PasswordChanged => SignOutReason::PasswordChanged,
        SignOutReasonKind::Recovered => SignOutReason::Recovered,
        SignOutReasonKind::Evicted => SignOutReason::Evicted,
        SignOutReasonKind::Expired => SignOutReason::Expired,
        SignOutReasonKind::Rekeyed => SignOutReason::Rekeyed,
        SignOutReasonKind::Unknown => SignOutReason::Unknown,
    }
}

fn quota_limit(kind: QuotaKind) -> QuotaLimit {
    match kind {
        QuotaKind::Docs => QuotaLimit::Docs,
        QuotaKind::DocBytes => QuotaLimit::DocBytes,
        QuotaKind::Files => QuotaLimit::Files,
        QuotaKind::FileBytes => QuotaLimit::FileBytes,
        QuotaKind::Server => QuotaLimit::Server,
    }
}

fn quota_of(limit: QuotaLimit) -> QuotaKind {
    match limit {
        QuotaLimit::Docs => QuotaKind::Docs,
        QuotaLimit::DocBytes => QuotaKind::DocBytes,
        QuotaLimit::Files => QuotaKind::Files,
        QuotaLimit::FileBytes => QuotaKind::FileBytes,
        QuotaLimit::Server => QuotaKind::Server,
    }
}

fn step_of(step: AccountStep) -> Step {
    match step {
        AccountStep::StoppingSync => Step::StoppingSync,
        AccountStep::DerivingKeys => Step::DerivingKeys,
        AccountStep::Contacting => Step::Contacting,
        AccountStep::Saving => Step::Saving,
    }
}

#[cfg(test)]
mod tests {
    use kanso_engine::strategies;
    use kanso_engine::sync::words::{problem_help, problem_label, sync_line};
    use proptest::prelude::*;

    use super::*;
    use crate::syncing::{problem_kind, status_of};

    fn quiet(s: &str) -> impl FnOnce(i64) -> String {
        let s = s.to_owned();
        move |_| s
    }

    /// A status for a library signed in as "ada", in `state`.
    fn status_in(state: &SyncState) -> SyncStatus {
        let mut status = status_of(Some(crate::syncing::test_session()), state);
        status.account = SyncAccount::SignedIn;
        status
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Every problem the sync can end with survives the status: what
        /// the status carries builds the same problem back, so the phone's
        /// words are the engine's words for it.
        #[test]
        fn prop_a_problem_round_trips_through_the_status(problem in strategies::problem()) {
            let state = SyncState::Failed(problem.clone());
            let status = status_in(&state);
            prop_assert_eq!(problem_of(&status), Some(problem.clone()));
            prop_assert_eq!(state_of(&status), state);
            prop_assert_eq!(status.problem, Some(problem_kind(&problem)));
        }

        /// The phone's line is the desktop's line: for any state, any
        /// account and any settings it says what `sync_status_line` says
        /// over the same state, with the times the shell gave.
        #[test]
        fn prop_the_status_line_is_the_engines(
            state in strategies::sync_state(),
            account in strategies::account(),
            enabled: bool,
            minutes in prop::sample::select(SYNC_INTERVALS.to_vec()),
        ) {
            let mut status = status_in(&state);
            // The details of a state ride along with its problem.
            if let SyncState::Synced { conflicts, .. } = &state {
                prop_assert_eq!(status.conflicts as usize, *conflicts);
            }
            status.account = match &account {
                AccountState::SignedOut => SyncAccount::SignedOut,
                AccountState::SignedIn { .. } => SyncAccount::SignedIn,
                AccountState::SignedInLocked { .. } => SyncAccount::Locked,
                AccountState::SessionEnded { reason, .. } => {
                    status.signed_out_reason = Some(reason_kind(*reason));
                    SyncAccount::SessionEnded
                }
                AccountState::Busy(step) => SyncAccount::Busy(match step {
                    Step::StoppingSync => AccountStep::StoppingSync,
                    Step::DerivingKeys => AccountStep::DerivingKeys,
                    Step::Contacting => AccountStep::Contacting,
                    Step::Saving => AccountStep::Saving,
                }),
            };
            let settings = SyncSettingsRow { enabled, interval_secs: (minutes * 60) as u32 };
            let line = SyncHandle::status_line(status.clone(), settings, "WHEN".into(), "RESUME".into());
            let expected = sync_status_line(
                &state_of(&status),
                &account_of(&status),
                enabled,
                minutes * 60,
                quiet("WHEN"),
                quiet("RESUME"),
                None,
            );
            prop_assert_eq!(line.label, expected.label);
            prop_assert_eq!(line.detail, expected.detail);
            prop_assert_eq!(line.tone, tone(expected.tone));
        }

        /// A pause names when it ends with the shell's words, and nothing
        /// else does.
        #[test]
        fn prop_only_a_pause_names_the_resume_time(problem in strategies::problem()) {
            let status = status_in(&SyncState::Failed(problem.clone()));
            let settings = SyncSettingsRow::default();
            let line = SyncHandle::status_line(status, settings, "ZQWHEN".into(), "ZQRESUME".into());
            prop_assert_eq!(
                line.detail.contains("ZQRESUME"),
                matches!(problem, SyncProblem::Paused { .. } | SyncProblem::ServerBusy { .. })
            );
            prop_assert!(!line.detail.contains("ZQWHEN"));
            prop_assert_eq!(line.label, problem_label(&problem).to_owned());
        }

        /// Only the desktop's intervals are accepted, and what is accepted
        /// is kept as given.
        #[test]
        fn prop_settings_take_only_the_desktops_intervals(secs in 0u32..20_000, enabled: bool) {
            let row = SyncSettingsRow { enabled, interval_secs: secs };
            let offered = SYNC_INTERVALS.iter().any(|m| m * 60 == u64::from(secs));
            prop_assert_eq!(check_settings(&row).is_ok(), offered);
        }
    }

    // no-prop-test: the words of the examples in the brief, each once; the
    // engine's tests hold the table of every problem.
    #[test]
    fn the_phone_says_what_the_desktop_says() {
        let synced = SyncState::Synced {
            at_ms: 10,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: None,
        };
        let line = SyncHandle::status_line(
            status_in(&synced),
            SyncSettingsRow::default(),
            "just now".into(),
            String::new(),
        );
        assert_eq!(
            (line.tone, line.label.as_str(), line.detail.as_str()),
            (
                SyncTone::Good,
                "Synced just now",
                "Kanso syncs every 5 minutes and soon after you edit."
            )
        );
        let paused = status_in(&SyncState::Failed(SyncProblem::Paused { until: 5 }));
        let line = SyncHandle::status_line(
            paused,
            SyncSettingsRow::default(),
            String::new(),
            "at 3:00 PM".into(),
        );
        assert_eq!(
            line.detail,
            problem_help(&SyncProblem::Paused { until: 5 }, |_| "at 3:00 PM".into())
        );
        assert_eq!(
            sync_line(&SyncState::Off, &AccountState::SignedOut, |_| String::new()).1,
            "Set up sync"
        );
        let signed_out = SyncHandle::status_line(
            SyncStatus::default(),
            SyncSettingsRow::default(),
            String::new(),
            String::new(),
        );
        assert_eq!(signed_out.label, "Sync is off");
        assert_eq!(signed_out.tone, SyncTone::Quiet);
    }

    // no-prop-test: the desktop's six choices and their words, in order.
    #[test]
    fn the_intervals_are_the_desktops() {
        let options = SyncHandle::interval_options();
        assert_eq!(
            options
                .iter()
                .map(|o| (o.minutes, o.label.as_str()))
                .collect::<Vec<_>>(),
            [
                (1, "1 minute"),
                (5, "5 minutes"),
                (15, "15 minutes"),
                (30, "30 minutes"),
                (60, "1 hour"),
                (240, "4 hours"),
            ]
        );
        assert_eq!(SyncHandle::interval_phrase(15), "every 15 minutes");
        assert_eq!(SyncHandle::interval_label(60), "1 hour");
        assert_eq!(SyncSettingsRow::default().interval_secs, 300);
        assert!(SyncSettingsRow::default().enabled);
    }
}
