//! In-app feedback on the phones (issue #232): Settings › Feedback.
//!
//! The rules and the flow are the desktop's (`kanso_engine::feedback` and
//! `feedback_flow`): a report is saved to the library's outbox first and
//! then sent, retried when the phone is back online and at app resume, and
//! never contains a note. This module is the seam: it builds the app info
//! from what the shell says, picks who is sending (the signed-in device, or
//! the install's anonymous key), and turns the outcomes into values the
//! shell words in its own language.

use std::sync::Arc;

use kanso_engine::feedback::{
    self, AppInfo, Chip, Draft, DraftError, Kind, Platform, ReportView, Screen, Status, SyncMode,
    clean_build, clean_device, clean_locale, clean_os,
};
use kanso_engine::feedback_flow::{self, Picture, Sent, Wait};
use kanso_sync::feedback::{Client, HttpTransport, Reporter, Transport};
use kanso_sync::{Machine, SecretStore};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::feedback_model::{
    FeedbackChipKind, FeedbackContext, FeedbackInput, FeedbackPreview, FeedbackProblem,
    FeedbackReport, FeedbackSent,
};
use crate::library::{Inner, Library};

/// The `meta` key holding the releases My feedback has shown: the ids of
/// the released reports the person has looked at, comma-separated.
const SEEN_KEY: &str = "mobile.feedback.seen";

/// Kanso's own server, whichever server the library syncs with.
const KANSO_ORIGIN: &str = "https://sync.kansoapp.app";

/// The feedback handle of one library.
#[derive(Clone)]
pub struct MobileFeedback {
    pub(crate) inner: Arc<Inner>,
    transport: Arc<dyn Transport>,
}

impl Library {
    /// Send feedback and My feedback.
    pub fn feedback(&self) -> MobileFeedback {
        MobileFeedback {
            inner: self.inner.clone(),
            transport: Arc::new(HttpTransport::new()),
        }
    }

    /// The same over `transport`, for tests.
    pub fn feedback_over(&self, transport: Arc<dyn Transport>) -> MobileFeedback {
        MobileFeedback {
            inner: self.inner.clone(),
            transport,
        }
    }
}

fn problem_of(err: &DraftError) -> FeedbackProblem {
    match err {
        DraftError::TooShort => FeedbackProblem::TooShort,
        DraftError::TooLong => FeedbackProblem::TooLong,
        DraftError::BadCharacter => FeedbackProblem::BadCharacter,
        DraftError::BadCredit => FeedbackProblem::BadCredit,
        DraftError::BadScreenshot => FeedbackProblem::BadScreenshot,
        DraftError::BadId | DraftError::BadApp(_) => FeedbackProblem::Other,
    }
}

fn screen_of(word: &str) -> Screen {
    Screen::ALL
        .into_iter()
        .find(|screen| screen.wire() == word)
        .unwrap_or(Screen::Other)
}

/// The version without a build suffix: `0.9.4+57` is `0.9.4`. Falls back to
/// `0.0.0` when it is not three numbers.
pub(crate) fn core_version(version: &str) -> String {
    let core = version.split(['+', '-']).next().unwrap_or("");
    if feedback::is_version(core) {
        core.to_owned()
    } else {
        "0.0.0".to_owned()
    }
}

/// The app info the shell's context makes. The crash types are the type
/// names of waiting crash reports (`crash: <type>`); the others are
/// cleaned to the Worker's patterns.
pub(crate) fn app_info(context: &FeedbackContext, sync: SyncMode) -> AppInfo {
    let crashes = context
        .crash_types
        .iter()
        .map(|name| format!("crash: {name}"));
    AppInfo {
        platform: Platform::current(),
        version: core_version(&context.version),
        build: clean_build(&context.build),
        patch: None,
        os: clean_os(&context.os),
        device: clean_device(&context.device),
        locale: clean_locale(&context.locale),
        screen: screen_of(&context.screen),
        sync,
        errors: feedback::scrub_log_lines(crashes.collect::<Vec<_>>().iter().map(String::as_str)),
    }
}

fn draft_of(input: &FeedbackInput) -> Result<Draft, DraftError> {
    let kind = Kind::from_wire(&input.kind).unwrap_or(Kind::Bug);
    let screenshot = match &input.screenshot {
        // Declared as it will be sent: without the photo's EXIF (GPS) and text.
        Some(bytes) => Some(feedback::declare_screenshot(&feedback::strip_metadata(
            bytes,
        ))?),
        None => None,
    };
    Ok(Draft {
        kind,
        text: input.text.clone(),
        credit_name: input.credit_name.clone(),
        screenshot,
    })
}

fn chip_kind(chip: &Chip) -> (FeedbackChipKind, Option<u32>, Option<String>) {
    match chip {
        Chip::WaitingToSend => (FeedbackChipKind::WaitingToSend, None, None),
        Chip::Received => (FeedbackChipKind::Received, None, None),
        Chip::FiledAs(n) => (FeedbackChipKind::FiledAs, Some(*n), None),
        Chip::Building => (FeedbackChipKind::Building, None, None),
        Chip::FixProposed => (FeedbackChipKind::FixProposed, None, None),
        Chip::Merged => (FeedbackChipKind::Merged, None, None),
        Chip::ReleasedIn(release) => (
            FeedbackChipKind::ReleasedIn,
            None,
            Some(feedback::release_name(release).to_owned()),
        ),
        Chip::NotFiled => (FeedbackChipKind::NotFiled, None, None),
        Chip::SameAs(n) => (FeedbackChipKind::SameAs, Some(*n), None),
        Chip::Closed => (FeedbackChipKind::Closed, None, None),
        Chip::Deleting => (FeedbackChipKind::Deleting, None, None),
    }
}

/// A report as My feedback shows it. `seen` holds the ids already looked at.
pub(crate) fn report_of(view: &ReportView, seen: &[String]) -> FeedbackReport {
    let chip = feedback::chip(view);
    let (kind, number, release) = chip_kind(&chip);
    FeedbackReport {
        id: view.id.clone(),
        created_ms: view.created_ms,
        kind: view.kind.wire().to_owned(),
        preview: view.preview.clone(),
        chip: kind,
        number,
        release,
        pr: view.pr,
        issue_url: feedback::issue_url(feedback::REPO, view),
        deleting: view.deleting,
        fresh: view.status == Status::Released && !seen.contains(&view.id),
        on_server: view.status != Status::Waiting,
    }
}

impl MobileFeedback {
    fn client(&self) -> Client<Arc<dyn Transport>> {
        Client::new(self.transport.clone())
    }

    fn sync_mode(&self) -> SyncMode {
        match self.inner_session() {
            None => SyncMode::None,
            Some(origin) if origin == KANSO_ORIGIN => SyncMode::Kanso,
            Some(_) => SyncMode::SelfHosted,
        }
    }

    fn inner_session(&self) -> Option<String> {
        self.sync_handle()
            .load_session()
            .ok()
            .flatten()
            .map(|session| session.origin.clone())
    }

    fn sync_handle(&self) -> crate::SyncHandle {
        crate::SyncHandle {
            inner: self.inner.clone(),
        }
    }

    /// Who sends a new report: the signed-in device when the session is
    /// Kanso's own server's, else the install's anonymous key. A self-hosted
    /// server's token and device proof never go to Kanso's Worker.
    fn reporter(&self) -> Result<Reporter, KansoError> {
        if let Some(session) = self.sync_handle().load_session()? {
            let machine = Machine {
                hardware_uuid: None,
                computer_name: String::new(),
            };
            let device = kanso_sync::device_identity(&self.inner.store, &machine)?;
            if let Some(reporter) = Reporter::from_session(&session, &device) {
                return Ok(reporter);
            }
        }
        let secrets: &dyn SecretStore = self.inner.secrets.as_ref();
        Reporter::anonymous(secrets).map_err(KansoError::from)
    }

    /// Every reporter this phone has: [`Self::reporter`], and the anonymous
    /// key too when signed in and one was made, so reports sent before
    /// signing in can still be listed, uploaded and deleted.
    fn reporters(&self) -> Result<Vec<Reporter>, KansoError> {
        let first = self.reporter()?;
        let secrets: &dyn SecretStore = self.inner.secrets.as_ref();
        let anonymous = first
            .is_account()
            .then(|| Reporter::existing_anonymous(secrets))
            .flatten();
        Ok(std::iter::once(first).chain(anonymous).collect())
    }

    fn data_dir(&self) -> std::path::PathBuf {
        self.inner
            .path
            .parent()
            .map_or_else(|| ".".into(), std::path::Path::to_path_buf)
    }

    fn seen(&self) -> Vec<String> {
        self.inner
            .store
            .meta(SEEN_KEY)
            .ok()
            .flatten()
            .map(|text| {
                text.split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The exact JSON "Show what's sent" displays, or why there is none yet.
    pub fn preview(&self, input: FeedbackInput) -> Result<FeedbackPreview, KansoError> {
        guard(|| {
            let app = input
                .include_app
                .then(|| app_info(&input.context, self.sync_mode()));
            let shown = draft_of(&input)
                .and_then(|draft| feedback_flow::preview_payload(&draft, app.as_ref()));
            Ok(match shown {
                Ok(json) => FeedbackPreview {
                    json,
                    problem: None,
                },
                Err(err) => FeedbackPreview {
                    json: String::new(),
                    problem: Some(problem_of(&err)),
                },
            })
        })
    }

    /// Saves the report, then tries to send it. Blocking: the shell calls it
    /// off the UI thread (every bridge call is).
    pub fn send(&self, input: FeedbackInput) -> Result<FeedbackSent, KansoError> {
        guard(|| {
            let draft = match draft_of(&input) {
                Ok(draft) => draft,
                Err(err) => {
                    return Ok(FeedbackSent::Invalid {
                        problem: problem_of(&err),
                    });
                }
            };
            let app = input
                .include_app
                .then(|| app_info(&input.context, self.sync_mode()));
            let reporter = self.reporter()?;
            let mut entropy = [0u8; 16];
            getrandom::fill(&mut entropy).map_err(|err| KansoError::Internal {
                message: err.to_string(),
            })?;
            let picture = input.screenshot.as_deref().map(|bytes| Picture { bytes });
            let sent = feedback_flow::send(
                &self.inner.store,
                &self.client(),
                &reporter,
                &self.data_dir(),
                entropy,
                draft,
                picture,
                app.as_ref(),
                kanso_core::now_ms(),
            );
            Ok(match sent {
                Ok((_, Sent::Done)) => FeedbackSent::Sent,
                Ok((_, Sent::Queued(Wait::Paused))) => FeedbackSent::Paused,
                Ok((_, Sent::Queued(Wait::RateLimited { .. }))) => FeedbackSent::RateLimited,
                Ok((_, Sent::Queued(_))) => FeedbackSent::Queued,
                Ok((_, Sent::Rejected { field })) => FeedbackSent::Rejected { field },
                Err(feedback_flow::Error::Draft(err)) => FeedbackSent::Invalid {
                    problem: problem_of(&err),
                },
                Err(feedback_flow::Error::Store(err)) => return Err(err.into()),
                Err(feedback_flow::Error::Io(err)) => {
                    return Err(KansoError::Internal {
                        message: err.to_string(),
                    });
                }
            })
        })
    }

    /// Sends what is waiting: at app resume and when connectivity returns.
    pub fn flush(&self) -> Result<(), KansoError> {
        guard(|| {
            let reporters = self.reporters()?;
            feedback_flow::flush_as(
                &self.inner.store,
                &self.client(),
                &reporters,
                kanso_core::now_ms(),
            )
            .map(|_| ())
            .map_err(flow_error)
        })
    }

    /// Makes every waiting report due again: connectivity returned.
    pub fn retry_now(&self) -> Result<(), KansoError> {
        guard(|| feedback_flow::store_retry_now(&self.inner.store).map_err(flow_error))
    }

    /// The reports, newest first. With `refresh`, the server is asked first
    /// (when My feedback opens and at launch, at most every 6 hours); offline
    /// or refused, what the phone last knew is shown.
    pub fn reports(&self, refresh: bool) -> Result<Vec<FeedbackReport>, KansoError> {
        guard(|| {
            if refresh {
                let reporters = self.reporters()?;
                let now = kanso_core::now_ms();
                // What is waiting goes first, so a fresh report is listed as sent.
                feedback_flow::flush_as(&self.inner.store, &self.client(), &reporters, now).ok();
                feedback_flow::refresh_as(&self.inner.store, &self.client(), &reporters, now)
                    .map_err(flow_error)?;
            }
            let seen = self.seen();
            Ok(feedback_flow::views(&self.inner.store)
                .map_err(flow_error)?
                .iter()
                .map(|view| report_of(view, &seen))
                .collect())
        })
    }

    /// Whether a launch refresh is due (never done, or 6 hours ago).
    pub fn refresh_due(&self) -> bool {
        feedback_flow::refresh_due(&self.inner.store, kanso_core::now_ms())
    }

    /// Deletes a report: gone at once if the server never had it, else
    /// "Deleting…" until the server confirms.
    pub fn delete(&self, id: String) -> Result<(), KansoError> {
        guard(|| {
            if feedback_flow::delete(&self.inner.store, &id).map_err(flow_error)? {
                self.flush()?;
            }
            Ok(())
        })
    }

    /// The released reports have been looked at: their rows stop being bold
    /// and the Settings row stops saying "Fixed in…".
    pub fn mark_seen(&self) -> Result<(), KansoError> {
        guard(|| {
            let mut seen = self.seen();
            for view in feedback_flow::views(&self.inner.store).map_err(flow_error)? {
                if view.status == Status::Released && !seen.contains(&view.id) {
                    seen.push(view.id);
                }
            }
            self.inner.store.set_meta(SEEN_KEY, &seen.join(","))?;
            Ok(())
        })
    }
}

fn flow_error(err: feedback_flow::Error) -> KansoError {
    match err {
        feedback_flow::Error::Store(err) => err.into(),
        feedback_flow::Error::Io(err) => KansoError::Internal {
            message: err.to_string(),
        },
        feedback_flow::Error::Draft(_) => KansoError::Internal {
            message: "the report could not be built".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn versions_lose_their_build() {
        assert_eq!(core_version("0.9.4+57"), "0.9.4");
        assert_eq!(core_version("0.9.4-dev.abc"), "0.9.4");
        assert_eq!(core_version("garbage"), "0.0.0");
    }

    #[test]
    fn crash_types_become_error_lines() {
        let context = FeedbackContext {
            version: "0.9.4".into(),
            build: "57".into(),
            os: "18.1".into(),
            device: "iPhone15,2".into(),
            locale: "en-GB".into(),
            screen: "settings".into(),
            crash_types: vec!["FormatException".into(), "has a space".into()],
        };
        let info = app_info(&context, SyncMode::None);
        assert_eq!(info.errors, vec!["crash: FormatException".to_owned()]);
        assert_eq!(info.screen, Screen::Settings);
        assert!(feedback::validate_app(&info).is_ok());
    }

    #[test]
    fn unknown_screens_are_other() {
        assert_eq!(screen_of("note:abc123"), Screen::Other);
        assert_eq!(screen_of("note-list"), Screen::NoteList);
    }

    proptest! {
        /// Whatever the shell says, the app info passes the Worker's patterns
        /// and never carries the free text through.
        #[test]
        fn prop_app_info_is_always_valid(
            version in ".{0,20}", build in ".{0,20}", os in ".{0,50}", device in ".{0,90}",
            locale in ".{0,20}", screen in ".{0,30}",
            crashes in proptest::collection::vec(".{0,40}", 0..30),
        ) {
            let context = FeedbackContext { version, build, os, device, locale, screen, crash_types: crashes };
            let info = app_info(&context, SyncMode::Kanso);
            prop_assert!(feedback::validate_app(&info).is_ok());
            prop_assert!(info.errors.len() <= feedback::MAX_ERRORS);
        }
    }
}
