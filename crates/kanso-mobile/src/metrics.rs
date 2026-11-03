//! The anonymous usage count (docs/METRICS.md) on the phones: the notice
//! shown once, the features counted where they happen, and the batches sent
//! at launch, hourly while the app is open and when it goes to the
//! background.
//!
//! The rules live in `kanso-metrics`, as they do on the desktop. This module
//! keeps the ledger in the library's own settings (`Store::meta`), reads the
//! account and sync state from the library, and never sends before the
//! notice was seen. Development builds and tests send nothing.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use kanso_core::DocType;
use kanso_import::library::SourceKind;
use kanso_metrics::ranges::{AttachmentsMb, Channel, LocaleLang, NotebooksRange, Theme, ThemeMode};
use kanso_metrics::values::ActiveExtras;
use kanso_metrics::{
    Active, Day, Device, Form, Id, ImportSource, Ledger, Outcome, Platform, SyncMode,
};

pub(crate) use kanso_metrics::Feature;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::{Inner, Library};

/// The notice's acknowledgement: the version of the notice that was seen.
/// Version 2 is the hourly count with its longer list; an older `1` shows the
/// notice again once.
const NOTICE_SEEN: &str = "mobile.metrics.notice_seen";
/// The version of the notice this build shows.
pub(crate) const NOTICE_VERSION: u32 = 2;
/// The languages the app ships (`mobile/lib/l10n/arb/app_<code>.arb`).
pub(crate) const SHIPPED_LANGUAGES: &[&str] = &["en"];
/// The shared client's day's bookkeeping, as JSON.
const LEDGER: &str = "mobile.metrics.ledger";

/// The anonymous usage count of one library.
#[derive(Clone)]
pub struct MobileMetrics {
    pub(crate) inner: Arc<Inner>,
}

impl Inner {
    /// The metrics handle for this library, shared by the feature calls that
    /// sit deeper than the `Library` facade.
    pub(crate) fn metrics(self: &Arc<Self>) -> MobileMetrics {
        MobileMetrics {
            inner: self.clone(),
        }
    }
}

impl Library {
    /// The anonymous usage count: the notice's state, feature counts, and
    /// the batch `send_due` sends.
    pub fn metrics(&self) -> MobileMetrics {
        self.inner.metrics()
    }
}

impl MobileMetrics {
    /// Whether the notice has been acknowledged.
    pub fn notice_seen(&self) -> Result<bool, KansoError> {
        guard(|| self.notice_seen_now())
    }

    /// Keeps that the notice was acknowledged. Counting starts after this.
    pub fn mark_notice_seen(&self) -> Result<(), KansoError> {
        guard(|| {
            self.inner
                .store
                .set_meta(NOTICE_SEEN, &NOTICE_VERSION.to_string())?;
            Ok(())
        })
    }

    /// Counts `feature` once for today, if this build counts and the notice
    /// was seen. The name is one of the closed feature list; an unknown name
    /// is refused rather than counted as something else.
    pub fn record(&self, feature: &str) -> Result<(), KansoError> {
        guard(|| {
            let feature = feature_named(feature)
                .ok_or_else(|| KansoError::invalid("That isn't a feature Kanso counts."))?;
            self.record_if(enabled(), feature)
        })
    }

    /// Notes that the app was launched, once per process: one `app_opened`
    /// waits for the next send. Dart calls it once the notice was seen.
    pub fn launched(&self) -> Result<(), KansoError> {
        guard(|| self.launched_if(enabled()))
    }

    /// Notes that a foreground stretch of `secs` seconds ended (the app went
    /// to the background): one `session` waits for the next send.
    pub fn end_session(&self, secs: u64) -> Result<(), KansoError> {
        guard(|| self.end_session_if(enabled(), secs))
    }

    /// Sends what is due: `app_active` once a day, `app_opened`, ended
    /// sessions and feature counts. Does nothing unless the notice was seen
    /// and this is a release build. `tablet` is the device's form,
    /// `os_version` its system version (or `unknown`), `ai` whether AI is on,
    /// `locale` the UI's language code, `theme_mode` one of `light`, `dark`,
    /// `system` (else left out) and `theme` the theme ID in use (a built-in
    /// one by its ID, anything else is `custom`; empty leaves it out). Says
    /// whether the Worker accepted a batch.
    pub fn send_due(
        &self,
        tablet: bool,
        os_version: &str,
        ai: bool,
        locale: &str,
        theme_mode: &str,
        theme: &str,
    ) -> Result<bool, KansoError> {
        guard(|| self.send_due_if(enabled(), tablet, os_version, ai, locale, theme_mode, theme))
    }

    /// Counts a feature from product code. Metrics are best effort: a write
    /// failure must never turn a successful note, lock, share or import into
    /// an error.
    pub(crate) fn count(&self, feature: Feature) {
        self.count_n(feature, 1);
    }

    /// Counts `n` uses of a feature at once (a change to many notes counts
    /// each note). Buffered in memory and written to the ledger in a batch.
    pub(crate) fn count_n(&self, feature: Feature, n: u32) {
        let _ = guard(|| self.count_n_if(enabled(), feature, n));
    }

    /// Counts that the note `id` was edited, once per note until the next
    /// send, however much it is typed in. The ID is held in memory only and
    /// never sent.
    pub(crate) fn count_note_edited(&self, id: &str) {
        if !enabled() || !edited().insert_new(id) {
            return;
        }
        self.count(Feature::NoteEdited);
    }

    /// Counts a failed sync; it is reported as a range with `app_active`.
    pub(crate) fn count_sync_error(&self) {
        let _ = guard(|| {
            if !enabled() || !self.notice_seen_now()? {
                return Ok(());
            }
            let _gate = crate::boundary::held(&self.inner.metrics_gate);
            let mut ledger = self.read_ledger()?;
            if ledger.record_sync_error() {
                self.write_ledger(&ledger)?;
            }
            Ok(())
        });
    }

    fn launched_if(&self, enabled: bool) -> Result<(), KansoError> {
        if !enabled || !self.notice_seen_now()? || LAUNCH_COUNTED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let _gate = crate::boundary::held(&self.inner.metrics_gate);
        let mut ledger = self.read_ledger()?;
        if ledger.launch() {
            self.write_ledger(&ledger)?;
        }
        Ok(())
    }

    fn end_session_if(&self, enabled: bool, secs: u64) -> Result<(), KansoError> {
        if !enabled || !self.notice_seen_now()? {
            return Ok(());
        }
        self.flush_buffer()?;
        let _gate = crate::boundary::held(&self.inner.metrics_gate);
        let mut ledger = self.read_ledger()?;
        ledger.end_session(Day::today(), secs);
        self.write_ledger(&ledger)
    }

    fn record_if(&self, enabled: bool, feature: Feature) -> Result<(), KansoError> {
        self.count_n_if(enabled, feature, 1)
    }

    /// Adds `n` uses to the in-memory buffer; the buffer is written to the
    /// ledger at most every 30 seconds, and on every send and every end of
    /// a session, so a hot action isn't a SQLite write.
    fn count_n_if(&self, enabled: bool, feature: Feature, n: u32) -> Result<(), KansoError> {
        if !enabled || n == 0 || !self.notice_seen_now()? {
            return Ok(());
        }
        let due = {
            let mut buffer = crate::boundary::held(&self.inner.metrics_buffer);
            buffer.add(Day::today(), feature, n);
            buffer.flush_due()
        };
        if due {
            self.flush_buffer()?;
        }
        Ok(())
    }

    /// Writes the buffered counts into the ledger in one read and one write.
    /// If the write fails they go back in the buffer.
    pub(crate) fn flush_buffer(&self) -> Result<(), KansoError> {
        let pending = crate::boundary::held(&self.inner.metrics_buffer).take();
        if pending.is_empty() {
            return Ok(());
        }
        let written = (|| {
            let _gate = crate::boundary::held(&self.inner.metrics_gate);
            let mut ledger = self.read_ledger()?;
            for (day, feature, n) in &pending {
                ledger.record_n(*day, *feature, *n);
            }
            self.write_ledger(&ledger)
        })();
        if written.is_err() {
            crate::boundary::held(&self.inner.metrics_buffer).put_back(pending);
        }
        written
    }

    #[allow(clippy::too_many_arguments)]
    fn send_due_if(
        &self,
        enabled: bool,
        tablet: bool,
        os_version: &str,
        ai: bool,
        locale: &str,
        theme_mode: &str,
        theme: &str,
    ) -> Result<bool, KansoError> {
        if !enabled || !self.notice_seen_now()? {
            return Ok(false);
        }
        let Some(platform) = Platform::current() else {
            return Ok(false);
        };
        self.flush_buffer()?;
        let today = Day::today();
        let form = if tablet { Form::Tablet } else { Form::Phone };
        let device = Device::new(platform, form, env!("CARGO_PKG_VERSION"));
        let counts = self.inner.store.counts()?;
        let count_of = |doc_type| {
            u64::try_from(counts.get(&doc_type).copied().unwrap_or(0)).unwrap_or(u64::MAX)
        };
        let extras = ActiveExtras {
            locale_lang: Some(LocaleLang::new(locale, SHIPPED_LANGUAGES)),
            theme_mode: ThemeMode::ALL
                .iter()
                .copied()
                .find(|m| m.as_str() == theme_mode),
            theme: (!theme.is_empty()).then(|| Theme::new(theme, is_builtin_theme(theme))),
            notebooks: Some(NotebooksRange::from_count(count_of(DocType::Book))),
            attachments_mb: self
                .inner
                .store
                .attachments_bytes()
                .ok()
                .map(AttachmentsMb::from_bytes),
            channel: Some(Channel::Stable),
            ..ActiveExtras::default()
        };
        let active = Active::new(os_version, self.sync_mode()?, ai, count_of(DocType::Note))
            .with_extras(extras);
        // Read the ledger and choose the batch under the gate, then release
        // it while the request is on the wire so a feature used meanwhile is
        // not overwritten by this send's older copy.
        let batch = {
            let _gate = crate::boundary::held(&self.inner.metrics_gate);
            let mut ledger = self.read_ledger()?;
            let id = self.id(&mut ledger);
            let Some(batch) = ledger.due(today, &id, &device, &active) else {
                // The install ID may have just been made; keep it for next time.
                self.write_ledger(&ledger)?;
                return Ok(false);
            };
            // Keep a newly made install ID even if the send then fails.
            self.write_ledger(&ledger)?;
            batch
        };
        // A new batch window: a note edited from here counts again.
        edited().clear();
        let outcome = kanso_metrics::send(&batch, enabled);
        let _gate = crate::boundary::held(&self.inner.metrics_gate);
        let mut ledger = self.read_ledger()?;
        ledger.settle(&batch, outcome);
        self.write_ledger(&ledger)?;
        Ok(outcome == Outcome::Accepted)
    }

    fn notice_seen_now(&self) -> Result<bool, KansoError> {
        Ok(notice_version_seen(
            self.inner.store.meta(NOTICE_SEEN)?.as_deref(),
        ))
    }

    fn read_ledger(&self) -> Result<Ledger, KansoError> {
        let Some(text) = self.inner.store.meta(LEDGER)? else {
            return Ok(Ledger::default());
        };
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    fn write_ledger(&self, ledger: &Ledger) -> Result<(), KansoError> {
        let text =
            serde_json::to_string(ledger).map_err(|err| KansoError::storage(err.to_string()))?;
        self.inner.store.set_meta(LEDGER, &text)?;
        Ok(())
    }

    /// The account's ID while signed in and holding its data key, else this
    /// install's. A new install ID is kept in `ledger` for the caller to
    /// save.
    fn id(&self, ledger: &mut Ledger) -> Id {
        let account = crate::syncing::SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()
        .ok()
        .flatten()
        .and_then(|session| Id::parse(&session.metrics_id()));
        if let Some(id) = account {
            return id;
        }
        if let Some(id) = ledger.existing_install_id() {
            return id.clone();
        }
        ledger.install_id()
    }

    /// `off` unless signed in with sync on; `kanso` on Kanso's own server,
    /// else `self_hosted`.
    fn sync_mode(&self) -> Result<SyncMode, KansoError> {
        let settings = crate::sync_words::read_settings(&self.inner.store)?;
        if !settings.enabled {
            return Ok(SyncMode::Off);
        }
        let session = crate::syncing::SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()?;
        let Some(session) = session else {
            return Ok(SyncMode::Off);
        };
        Ok(if kanso_sync::is_owner_server(&session.origin) {
            SyncMode::Kanso
        } else {
            SyncMode::SelfHosted
        })
    }
}

/// Whether this executable may count: a release build, never a test or a
/// development build. The Dart side also only starts the notice in the
/// production environment, so a `flutter run --release` of the development
/// entry point still counts nothing (it never sees the notice).
fn enabled() -> bool {
    !cfg!(debug_assertions) && !cfg!(test)
}

/// Whether the stored acknowledgement is of this build's notice or a later
/// one. The first notice stored `1`; anything unreadable is not seen.
pub(crate) fn notice_version_seen(stored: Option<&str>) -> bool {
    stored
        .and_then(|v| v.trim().parse::<u32>().ok())
        .is_some_and(|v| v >= NOTICE_VERSION)
}

/// Whether the app already counted this launch (once per process).
static LAUNCH_COUNTED: AtomicBool = AtomicBool::new(false);

/// The notes edited since the last send, by ID. In memory only.
struct Edited(Mutex<HashSet<String>>);

impl Edited {
    /// True the first time `id` is seen in this window.
    fn insert_new(&self, id: &str) -> bool {
        crate::boundary::held(&self.0).insert(id.to_owned())
    }

    fn clear(&self) {
        crate::boundary::held(&self.0).clear();
    }
}

fn edited() -> &'static Edited {
    static EDITED: std::sync::OnceLock<Edited> = std::sync::OnceLock::new();
    EDITED.get_or_init(|| Edited(Mutex::new(HashSet::new())))
}

/// Whether `id` is one of the themes compiled into the app.
fn is_builtin_theme(id: &str) -> bool {
    kanso_engine::theme::builtin::BUILTIN
        .iter()
        .any(|(file, _)| kanso_engine::theme::builtin::id_of(file) == id)
}

/// The closed feature names the bridge and the app's own calls use.
fn feature_named(name: &str) -> Option<Feature> {
    Some(match name {
        "note_created" => Feature::NoteCreated,
        "voice_input" => Feature::VoiceInput,
        "publish" => Feature::Publish,
        "lock" => Feature::Lock,
        "ai" => Feature::Ai,
        "quick_capture" => Feature::QuickCapture,
        "shared_note" => Feature::SharedNote,
        "export" => Feature::Export,
        // Counted once per search, when a result is opened.
        "search" => Feature::Search,
        // Counted by the Flutter shell, where the action happens.
        "graph_opened" => Feature::GraphOpened,
        "theme_changed" => Feature::ThemeChanged,
        "wikilink_followed" => Feature::WikilinkFollowed,
        "focus_mode" => Feature::FocusMode,
        // The app asked the system for a review of it.
        "review_prompt" => Feature::ReviewPrompt,
        _ => return None,
    })
}

/// The metrics `source` an import kind maps to. The kinds the published list
/// doesn't name are `other`.
pub(crate) fn import_source(kind: SourceKind) -> ImportSource {
    match kind {
        SourceKind::Markdown => ImportSource::Markdown,
        SourceKind::Obsidian => ImportSource::Obsidian,
        SourceKind::Logseq => ImportSource::Logseq,
        SourceKind::Notion => ImportSource::Notion,
        SourceKind::Evernote => ImportSource::Evernote,
        SourceKind::Bear => ImportSource::Bear,
        SourceKind::Joplin => ImportSource::Joplin,
        SourceKind::StandardNotes => ImportSource::StandardNotes,
        SourceKind::Simplenote => ImportSource::Simplenote,
        SourceKind::UpNote => ImportSource::Upnote,
        SourceKind::AppleNotes => ImportSource::AppleNotes,
        SourceKind::AppleNotesFiles
        | SourceKind::AppleJournal
        | SourceKind::GoogleKeep
        | SourceKind::Html
        | SourceKind::Roam
        | SourceKind::TextBundle
        | SourceKind::Tomboy => ImportSource::Other,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_sync::{MemorySecrets, MemoryServer};
    use proptest::prelude::*;

    use super::*;

    fn library() -> (tempfile::TempDir, Library) {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        (dir, library)
    }

    #[test]
    fn names_are_the_closed_feature_list() {
        for (name, feature) in [
            ("note_created", Feature::NoteCreated),
            ("voice_input", Feature::VoiceInput),
            ("publish", Feature::Publish),
            ("lock", Feature::Lock),
            ("ai", Feature::Ai),
            ("quick_capture", Feature::QuickCapture),
            ("shared_note", Feature::SharedNote),
            ("export", Feature::Export),
            ("graph_opened", Feature::GraphOpened),
            ("theme_changed", Feature::ThemeChanged),
            ("wikilink_followed", Feature::WikilinkFollowed),
            ("focus_mode", Feature::FocusMode),
            ("review_prompt", Feature::ReviewPrompt),
        ] {
            assert_eq!(feature_named(name), Some(feature));
        }
        for name in [
            "import",
            "",
            "telemetry",
            "Note_Created",
            "plugin_command",
            "sticky_opened",
        ] {
            assert_eq!(feature_named(name), None, "{name}");
        }
    }

    #[test]
    fn import_sources_are_closed() {
        assert_eq!(import_source(SourceKind::Obsidian), ImportSource::Obsidian);
        assert_eq!(
            import_source(SourceKind::AppleNotes),
            ImportSource::AppleNotes
        );
        assert_eq!(import_source(SourceKind::Html), ImportSource::Other);
        assert_eq!(import_source(SourceKind::Tomboy), ImportSource::Other);
    }

    #[test]
    fn notice_and_ledger_are_kept_in_the_library() {
        let (_dir, library) = library();
        let metrics = library.metrics();
        assert!(!metrics.notice_seen().unwrap());
        metrics.mark_notice_seen().unwrap();
        assert!(metrics.notice_seen().unwrap());
        assert!(!metrics.record_if(enabled(), Feature::Lock).is_err());
        metrics.record_if(true, Feature::Lock).unwrap();
        let ledger = metrics.read_ledger().unwrap();
        assert!(
            ledger.existing_install_id().is_none(),
            "no ID made before a send"
        );
        let legacy = library.metrics();
        assert!(legacy.notice_seen().unwrap(), "the same library keeps it");
    }

    #[test]
    fn the_notice_shows_again_after_a_new_version() {
        let (_dir, library) = library();
        let metrics = library.metrics();
        // The first notice stored `1`.
        library.inner.store.set_meta(NOTICE_SEEN, "1").unwrap();
        assert!(!metrics.notice_seen().unwrap());
        metrics.mark_notice_seen().unwrap();
        assert_eq!(
            library.inner.store.meta(NOTICE_SEEN).unwrap().as_deref(),
            Some("2")
        );
        assert!(metrics.notice_seen().unwrap());
    }

    #[test]
    fn launches_and_sessions_wait_in_the_ledger() {
        let (_dir, library) = library();
        let metrics = library.metrics();
        metrics.mark_notice_seen().unwrap();
        // Not counted before the notice and never in a test build, but the
        // internal paths with the gate passed in are.
        metrics.launched_if(true).unwrap();
        metrics.end_session_if(true, 400).unwrap();
        let ledger = metrics.read_ledger().unwrap();
        let id = kanso_metrics::values::install_id();
        let device = Device::new(Platform::Ios, Form::Phone, "0.9.0");
        let batch = ledger
            .due(
                Day::today(),
                &id,
                &device,
                &Active::new("18.1", SyncMode::Off, false, 0),
            )
            .unwrap();
        let json = String::from_utf8(batch.to_json()).unwrap();
        assert!(json.contains("app_opened") && json.contains(r#""duration":"5-15m""#));
    }

    #[test]
    fn shipped_languages_are_the_arb_files() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mobile/lib/l10n/arb");
        let mut found: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| {
                let name = e.ok()?.file_name().into_string().ok()?;
                let code = name.strip_prefix("app_")?.strip_suffix(".arb")?.to_owned();
                code.split('_').next().map(str::to_owned)
            })
            .collect();
        found.sort();
        found.dedup();
        let mut shipped: Vec<String> = SHIPPED_LANGUAGES.iter().map(|s| (*s).to_owned()).collect();
        shipped.sort();
        assert_eq!(found, shipped);
    }

    #[test]
    fn built_in_themes_are_told_from_custom_ones() {
        assert!(is_builtin_theme("kanso-dark"));
        assert!(!is_builtin_theme("my-own-theme"));
        assert!(!is_builtin_theme(""));
    }

    #[test]
    fn counts_are_buffered_and_written_together() {
        let (_dir, library) = library();
        let metrics = library.metrics();
        metrics.mark_notice_seen().unwrap();
        metrics.count_n_if(true, Feature::NoteMoved, 3).unwrap();
        metrics.count_n_if(true, Feature::NoteMoved, 2).unwrap();
        metrics.record_if(true, Feature::Search).unwrap();
        // Nothing is in the ledger yet: no SQLite write per action.
        assert_eq!(metrics.read_ledger().unwrap(), Ledger::default());
        metrics.flush_buffer().unwrap();
        let ledger = metrics.read_ledger().unwrap();
        let id = kanso_metrics::install_id();
        let device = Device::new(Platform::Ios, Form::Phone, "0.9.0");
        let active = Active::new("17.0", SyncMode::Off, false, 0);
        let batch = ledger.due(Day::today(), &id, &device, &active).unwrap();
        let counts: Vec<_> = batch
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                kanso_metrics::EventKind::FeatureUsed(f, n) => Some((f.name(), *n)),
                _ => None,
            })
            .collect();
        assert_eq!(counts, [("note_moved", 5), ("search", 1)]);
        // A flush with nothing waiting writes nothing.
        metrics.flush_buffer().unwrap();
    }

    #[test]
    fn a_damaged_ledger_costs_a_count_not_the_library() {
        let (_dir, library) = library();
        library.inner.store.set_meta(LEDGER, "{not json").unwrap();
        let metrics = library.metrics();
        assert_eq!(metrics.read_ledger().unwrap(), Ledger::default());
        metrics.write_ledger(&Ledger::default()).unwrap();
        assert_eq!(metrics.read_ledger().unwrap(), Ledger::default());
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(256) })]

        /// The notice counts as seen exactly when the stored version is this
        /// build's or later; nothing else (text, empty, negative) is.
        #[test]
        fn prop_notice_version(stored in prop::option::of("\\PC{0,6}"), n in 0u32..10) {
            prop_assert_eq!(notice_version_seen(Some(&n.to_string())), n >= NOTICE_VERSION);
            if let Some(text) = stored
                && text.trim().parse::<u32>().is_err()
            {
                prop_assert!(!notice_version_seen(Some(&text)));
            }
            prop_assert!(!notice_version_seen(None));
        }
    }
}
