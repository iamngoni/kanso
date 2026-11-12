//! Update notices on the phone (issue #243): the card at the top of the
//! notes list when a change in an update affects this phone.
//!
//! The notices, the rules and what Dismiss and Mark as fixed do are
//! `kanso_core::notices`, compiled in and never fetched. What a person put
//! away is kept in the library's local meta, where the desktop keeps it in
//! `config.json`.

use kanso_core::notices::{
    self, NOTICES, Notice, NoticeAction, NoticeContext, NoticePage, NoticeState, Platform,
    previous_version,
};
use kanso_store::Store;
use semver::Version;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::notices_model::{PhoneNotice, PhoneNoticeActionKind, PhoneNotices};

/// What was put away, as JSON.
const STATE_META: &str = "notices.state";
/// The version that ran last, and the one before the last update.
const LAST_RUN_META: &str = "mobile.last_run_version";
const PREVIOUS_META: &str = "update.previous_version";

impl Library {
    /// The card for the notes list. `signed_in_to_sync` is whether this
    /// phone has an account (the shell knows; the notice may ask).
    pub fn notices(&self, signed_in_to_sync: bool) -> Result<PhoneNotices, KansoError> {
        guard(|| {
            let current = Version::parse(&crate::diagnostics::version())
                .map_err(|err| KansoError::storage(err.to_string()))?;
            card(&self.inner.store, NOTICES, &current, signed_in_to_sync)
        })
    }

    /// Hides the card for `id` on this phone. Doing it twice is the same as
    /// once.
    pub fn dismiss_notice(&self, id: &str) -> Result<(), KansoError> {
        guard(|| change(&self.inner.store, |state| state.dismiss(id)))
    }

    /// Hides the card for `id` because the person fixed what it said.
    pub fn mark_notice_fixed(&self, id: &str) -> Result<(), KansoError> {
        guard(|| change(&self.inner.store, |state| state.mark_fixed(id)))
    }
}

/// What the phone has put away. A damaged entry reads as nothing put away.
fn read_state(store: &Store) -> Result<NoticeState, KansoError> {
    Ok(store
        .meta(STATE_META)?
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

fn change(store: &Store, apply: impl FnOnce(&mut NoticeState) -> bool) -> Result<(), KansoError> {
    let mut state = read_state(store)?;
    if apply(&mut state) {
        let json =
            serde_json::to_string(&state).map_err(|err| KansoError::storage(err.to_string()))?;
        store.set_meta(STATE_META, &json)?;
    }
    Ok(())
}

/// Notes the run in the library and says which version came before the last
/// update.
fn context(
    store: &Store,
    current: &Version,
    signed_in_to_sync: bool,
) -> Result<NoticeContext, KansoError> {
    let last_run = store.meta(LAST_RUN_META)?;
    let remembered = store.meta(PREVIOUS_META)?;
    let previous = previous_version(remembered.as_deref(), last_run.as_deref(), current);
    if let Some(previous) = &previous {
        let text = previous.to_string();
        if remembered.as_deref() != Some(text.as_str()) {
            store.set_meta(PREVIOUS_META, &text)?;
        }
    }
    let now = current.to_string();
    if last_run.as_deref() != Some(now.as_str()) {
        store.set_meta(LAST_RUN_META, &now)?;
    }
    Ok(NoticeContext {
        current: current.clone(),
        previous,
        // The phones have no local API login, and their library isn't
        // re-encrypted: nothing for a notice to ask there.
        had_login: false,
        signed_in_to_sync,
        library_key_saved: true,
        // The phones' library stays on stock SQLite; the OS protects it.
        library_plain: false,
    })
}

/// The card for `list`. Split out so a test can give its own list.
fn card(
    store: &Store,
    list: &[Notice],
    current: &Version,
    signed_in_to_sync: bool,
) -> Result<PhoneNotices, KansoError> {
    let ctx = context(store, current, signed_in_to_sync)?;
    let state = read_state(store)?;
    Ok(
        match notices::banner(list, &ctx, Platform::Phones, &state) {
            None => PhoneNotices::default(),
            Some(pick) => PhoneNotices {
                card: Some(view(pick.notice)),
                total: u32::try_from(pick.total).unwrap_or(u32::MAX),
                count_label: notices::count_label(pick.total).unwrap_or_default(),
            },
        },
    )
}

fn view(notice: &Notice) -> PhoneNotice {
    let (action_kind, action_label, action_target) = match notice.action {
        None => (PhoneNoticeActionKind::None, "", ""),
        Some(NoticeAction::OpenSettings(page)) => (
            PhoneNoticeActionKind::OpenSettings,
            "Open Settings",
            match page {
                NoticePage::General => "general",
                NoticePage::Account => "account",
                NoticePage::LocalApi => "local-api",
                NoticePage::Updates => "updates",
            },
        ),
        Some(NoticeAction::OpenUrl(url)) => (PhoneNoticeActionKind::OpenUrl, "Learn more", url),
    };
    PhoneNotice {
        id: notice.id.to_owned(),
        title: notice.title.to_owned(),
        what_changed: notice.what_changed.to_owned(),
        steps: notice.steps.iter().map(|s| (*s).to_owned()).collect(),
        action_kind,
        action_label: action_label.to_owned(),
        action_target: action_target.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::notices::Platforms;
    use proptest::prelude::*;

    fn here(ctx: &NoticeContext) -> bool {
        ctx.signed_in_to_sync
    }

    fn list() -> Vec<Notice> {
        vec![
            Notice {
                id: "fixture-a-0.10",
                since: Version::new(0, 10, 0),
                title: "A",
                what_changed: "Something changed.",
                steps: &["Do this."],
                action: Some(NoticeAction::OpenSettings(NoticePage::Account)),
                applies: here,
                platforms: Platforms::Phones,
            },
            Notice {
                id: "fixture-b-0.10",
                since: Version::new(0, 10, 0),
                title: "B",
                what_changed: "Something else.",
                steps: &["Do that."],
                action: None,
                applies: |_| true,
                platforms: Platforms::Both,
            },
            Notice {
                id: "fixture-desktop-0.10",
                since: Version::new(0, 10, 0),
                title: "Desktop only",
                what_changed: "Not for phones.",
                steps: &["Nothing."],
                action: None,
                applies: |_| true,
                platforms: Platforms::Desktop,
            },
        ]
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn the_card_shows_the_newest_notice_for_phones() {
        let (_dir, store) = store();
        let now = Version::new(0, 10, 0);
        let shown = card(&store, &list(), &now, true).unwrap();
        assert_eq!(shown.card.as_ref().unwrap().id, "fixture-a-0.10");
        assert_eq!(shown.total, 2);
        assert_eq!(shown.count_label, "1 of 2");
        let card_ = shown.card.unwrap();
        assert_eq!(card_.action_kind, PhoneNoticeActionKind::OpenSettings);
        assert_eq!(card_.action_target, "account");
        // Signed out, A doesn't apply.
        let shown = card(&store, &list(), &now, false).unwrap();
        assert_eq!(shown.card.unwrap().id, "fixture-b-0.10");
        assert_eq!(shown.count_label, "");
        // Before its version, nothing.
        let shown = card(&store, &list(), &Version::new(0, 9, 4), true).unwrap();
        assert!(shown.card.is_none() && shown.total == 0);
        // The shipped list has one thing for phones: update every device
        // for shared notes, for someone signed in to sync, from the release
        // that has it. The API keys notice is for the desktop's local API,
        // which phones don't have.
        for signed_in in [true, false] {
            for version in [&now, &Version::new(0, 10, 3), &Version::new(1, 0, 0)] {
                let shown = card(&store, NOTICES, version, signed_in).unwrap();
                if signed_in && *version >= Version::new(0, 10, 8) {
                    assert_eq!(shown.card.unwrap().id, "shared-properties-0.10");
                    assert_eq!(shown.total, 1);
                } else {
                    assert!(shown.card.is_none() && shown.total == 0);
                }
            }
        }
    }

    #[test]
    fn dismissing_persists_in_the_library() {
        let (dir, store) = store();
        let now = Version::new(0, 10, 0);
        change(&store, |s| s.dismiss("fixture-a-0.10")).unwrap();
        drop(store);
        let store = Store::open(dir.path().join("library.db")).unwrap();
        let shown = card(&store, &list(), &now, true).unwrap();
        assert_eq!(shown.card.unwrap().id, "fixture-b-0.10");
        change(&store, |s| s.mark_fixed("fixture-b-0.10")).unwrap();
        assert!(card(&store, &list(), &now, true).unwrap().card.is_none());
    }

    #[test]
    fn a_damaged_entry_puts_nothing_away() {
        let (_dir, store) = store();
        store.set_meta(STATE_META, "{not json").unwrap();
        assert_eq!(read_state(&store).unwrap(), NoticeState::default());
        let now = Version::new(0, 10, 0);
        assert!(card(&store, &list(), &now, true).unwrap().card.is_some());
    }

    #[test]
    fn the_version_before_an_update_is_kept() {
        let (_dir, store) = store();
        store.set_meta(LAST_RUN_META, "0.9.4").unwrap();
        let now = Version::new(0, 10, 0);
        let ctx = context(&store, &now, false).unwrap();
        assert_eq!(ctx.previous, Some(Version::new(0, 9, 4)));
        // The next launch of the same version still knows it.
        let ctx = context(&store, &now, false).unwrap();
        assert_eq!(ctx.previous, Some(Version::new(0, 9, 4)));
    }

    proptest! {
        /// Dismissing any number of times, in any order with fixing, leaves
        /// the same card and never shows a put-away notice.
        #[test]
        fn prop_put_away_notices_stay_away(dismiss in proptest::collection::vec(0usize..3, 0..6), fix in proptest::collection::vec(0usize..3, 0..3)) {
            let (_dir, store) = store();
            let list = list();
            let now = Version::new(0, 10, 0);
            for &ix in &dismiss {
                change(&store, |s| s.dismiss(list[ix].id)).unwrap();
            }
            for &ix in &fix {
                change(&store, |s| s.mark_fixed(list[ix].id)).unwrap();
            }
            let first = card(&store, &list, &now, true).unwrap();
            let second = card(&store, &list, &now, true).unwrap();
            prop_assert_eq!(&first, &second);
            if let Some(shown) = &first.card {
                prop_assert!(!dismiss.iter().any(|&ix| list[ix].id == shown.id));
                prop_assert!(!fix.iter().any(|&ix| list[ix].id == shown.id));
            }
        }
    }
}
