//! Update notices: what to tell someone when an update breaks something
//! they rely on, with the steps to fix it (issue #243, "Update notices").
//!
//! Notices are compiled into the app and never fetched, so nothing on a
//! network can put words in front of someone. The desktop and the phones
//! (through `kanso-mobile`) share this list, its wording and these rules; the
//! shapes are in [`crate::notice_model`].
//!
//! A notice is *shown* for a context when its platform matches, the running
//! version has reached its `since`, and its `applies` function says the
//! library is affected. What a person did with it (Dismiss, Mark as fixed)
//! is a [`NoticeState`], which each app keeps where it keeps its settings.

use std::cmp::Reverse;

pub use crate::notice_model::{
    BannerPick, Notice, NoticeAction, NoticeContext, NoticePage, NoticeState, Platform, Platforms,
};

/// Every notice this version knows about. A change that breaks something
/// adds one here; it is never reused or removed while a version that shows it
/// is supported.
pub const NOTICES: &[Notice] = &[API_KEYS, LIBRARY_PLAIN];

/// The ID of the API keys notice. Never reused.
pub const API_KEYS_ID: &str = "api-keys-0.10";

/// The local API's username and password are retired: every agent, script
/// and app needs its own key (issue #243). Only a library that had a login
/// is affected; the phones have no local API, so they never show it.
const API_KEYS: Notice = Notice {
    id: API_KEYS_ID,
    since: semver::Version::new(0, 10, 0),
    title: "Your AI agents and scripts need new keys",
    what_changed: "Kanso no longer accepts the local API's username and password, so a program that can start Kanso can't read your notes without your say-so. Agents and scripts you connected before won't reach your notes until you give each one a key.",
    steps: &[
        "Open Settings \u{203a} Local API \u{203a} API keys.",
        "Choose New key\u{2026} for each agent or script, and pick what it may do.",
        "In an MCP client (Claude Code, Claude Desktop, Codex, Cursor, VS Code, Gemini CLI, Windsurf, Zed or OpenCode), paste the setup Kanso gives you in place of the old Kanso entry. Then restart the client.",
        "In a REST tool or script, send the key as \u{201c}Authorization: Bearer <key>\u{201d} instead of the username and password. A tool that only takes a username and password can use any username, with the key as the password.",
    ],
    action: Some(NoticeAction::OpenSettings(NoticePage::LocalApi)),
    applies: had_login,
    platforms: Platforms::Desktop,
};

fn had_login(ctx: &NoticeContext) -> bool {
    ctx.had_login
}

/// The ID of the notice for a library that could not be encrypted. Never
/// reused.
pub const LIBRARY_PLAIN_ID: &str = "library-plain-0.10";

/// Kanso encrypts the library on disk (issue #243), but when it can't (the
/// system's key store refuses the key, or the disk is too small) the library
/// stays a plain file that any program running as you can read. Nothing is
/// lost, and Kanso tries again at every launch; this says so, and how to
/// help it. It shows only while the library is plain. The phones keep stock
/// SQLite and never show it.
const LIBRARY_PLAIN: Notice = Notice {
    id: LIBRARY_PLAIN_ID,
    since: semver::Version::new(0, 10, 0),
    title: "Your library isn't encrypted yet",
    what_changed: "Kanso encrypts your library on this computer, but couldn't this time, so it is still a plain file that any program running as you can read. Your notes are safe and nothing is lost. Kanso tries again each time it opens, and this goes away when it works.",
    steps: &[
        "Check that this computer's keychain is unlocked and lets Kanso keep an item. On Linux, run a Secret Service such as GNOME Keyring or KeePassXC.",
        "On a computer with no keychain, keep the library key in an owner-only file instead. If this library isn't signed in to sync, quit Kanso and set \"secretStore\": \"file\" in the \"sync\" section of config.json in the Kanso folder; if it is, sign in again with `kanso account sign-in --secrets file`. Then set KANSO_SECRETS_KEY_FILE to a key file kept away from the Kanso folder.",
        "Make sure the disk that holds the Kanso folder has free space about the size of your library.",
        "Quit Kanso and open it again.",
    ],
    action: None,
    applies: library_plain,
    platforms: Platforms::Desktop,
};

fn library_plain(ctx: &NoticeContext) -> bool {
    ctx.library_plain
}

impl Platforms {
    /// Whether an app on `platform` shows it.
    pub fn includes(self, platform: Platform) -> bool {
        matches!(
            (self, platform),
            (Platforms::Both, _)
                | (Platforms::Desktop, Platform::Desktop)
                | (Platforms::Phones, Platform::Phones)
        )
    }
}

impl NoticeContext {
    /// A context for `current` with nothing known about the library: no
    /// earlier version, no login, not signed in, key not saved.
    pub fn new(current: semver::Version) -> NoticeContext {
        NoticeContext {
            current,
            previous: None,
            had_login: false,
            signed_in_to_sync: false,
            library_key_saved: false,
            library_plain: false,
        }
    }
}

impl Notice {
    /// Whether this notice is for `ctx` on `platform`. Pure: the same
    /// context always gives the same answer.
    pub fn applies_to(&self, ctx: &NoticeContext, platform: Platform) -> bool {
        self.platforms.includes(platform) && ctx.current >= self.since && (self.applies)(ctx)
    }

    /// Whether `since` is in the release line (major and minor) of
    /// `version`, so What's new for that release lists it.
    pub fn is_from_release(&self, version: &semver::Version) -> bool {
        self.since.major == version.major && self.since.minor == version.minor
    }
}

impl NoticeState {
    pub fn is_dismissed(&self, id: &str) -> bool {
        self.dismissed.iter().any(|d| d == id)
    }

    pub fn is_fixed(&self, id: &str) -> bool {
        self.fixed.iter().any(|f| f == id)
    }

    /// Hides the banner for `id`. Says whether anything changed: doing it
    /// twice is the same as once.
    pub fn dismiss(&mut self, id: &str) -> bool {
        if self.is_dismissed(id) {
            return false;
        }
        self.dismissed.push(id.to_owned());
        true
    }

    /// Hides the banner and the palette entry for `id`. Says whether
    /// anything changed.
    pub fn mark_fixed(&mut self, id: &str) -> bool {
        if self.is_fixed(id) {
            return false;
        }
        self.fixed.push(id.to_owned());
        true
    }

    /// Whether the banner may show `id`.
    pub fn banner_allowed(&self, id: &str) -> bool {
        !self.is_dismissed(id) && !self.is_fixed(id)
    }
}

/// `list` newest first (by `since`), notices from the same version in the
/// order of their IDs. The same notices give the same order whatever order
/// they are listed in.
pub fn ordered(list: &[Notice]) -> Vec<&Notice> {
    let mut notices: Vec<&Notice> = list.iter().collect();
    notices.sort_by(|a, b| {
        Reverse(&a.since)
            .cmp(&Reverse(&b.since))
            .then_with(|| a.id.cmp(b.id))
    });
    notices
}

/// Every notice that applies to `ctx` on `platform`, in [`ordered`] order.
/// Dismissed and fixed ones included: What's new keeps them.
pub fn applying<'a>(
    list: &'a [Notice],
    ctx: &NoticeContext,
    platform: Platform,
) -> Vec<&'a Notice> {
    ordered(list)
        .into_iter()
        .filter(|notice| notice.applies_to(ctx, platform))
        .collect()
}

/// The notice for the banner (the newest that applies and hasn't been
/// dismissed or marked fixed) and how many are waiting.
pub fn banner<'a>(
    list: &'a [Notice],
    ctx: &NoticeContext,
    platform: Platform,
    state: &NoticeState,
) -> Option<BannerPick<'a>> {
    let waiting: Vec<&Notice> = applying(list, ctx, platform)
        .into_iter()
        .filter(|notice| state.banner_allowed(notice.id))
        .collect();
    Some(BannerPick {
        notice: waiting.first()?,
        total: waiting.len(),
    })
}

/// The notices the command palette lists: all that apply, except those
/// marked fixed. Dismissed ones stay.
pub fn listed<'a>(
    list: &'a [Notice],
    ctx: &NoticeContext,
    platform: Platform,
    state: &NoticeState,
) -> Vec<&'a Notice> {
    applying(list, ctx, platform)
        .into_iter()
        .filter(|notice| !state.is_fixed(notice.id))
        .collect()
}

/// The notices What's new lists under "Needs your attention": those from the
/// release running that apply, dismissed and fixed ones included, so the
/// steps are still there later.
pub fn for_whats_new<'a>(
    list: &'a [Notice],
    ctx: &NoticeContext,
    platform: Platform,
) -> Vec<&'a Notice> {
    applying(list, ctx, platform)
        .into_iter()
        .filter(|notice| notice.is_from_release(&ctx.current))
        .collect()
}

/// The version to remember as the one before the last update: the version
/// that ran last when the running one is newer (an update just happened),
/// otherwise the one remembered then. A notice that looks at the old version
/// keeps seeing it on every launch until the next update.
pub fn previous_version(
    remembered: Option<&str>,
    last_run: Option<&str>,
    current: &semver::Version,
) -> Option<semver::Version> {
    let parse = |text: Option<&str>| text.and_then(|t| semver::Version::parse(t).ok());
    match parse(last_run) {
        Some(last) if last < *current => Some(last),
        _ => parse(remembered),
    }
}

/// The words for the banner's "1 of 2", or nothing for a single notice.
pub fn count_label(total: usize) -> Option<String> {
    (total > 1).then(|| format!("1 of {total}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use semver::Version;

    fn affected(ctx: &NoticeContext) -> bool {
        ctx.had_login && !ctx.signed_in_to_sync
    }

    fn unsaved(ctx: &NoticeContext) -> bool {
        !ctx.signed_in_to_sync && !ctx.library_key_saved
    }

    /// The notices the tests use. None of them is shipped.
    fn fixture() -> Vec<Notice> {
        vec![
            Notice {
                id: "fixture-keys-0.10",
                since: Version::new(0, 10, 0),
                title: "Your scripts need new keys",
                what_changed: "Programs that used the old login need a key.",
                steps: &["Open Settings.", "Make a key.", "Restart the program."],
                action: Some(NoticeAction::OpenSettings(NoticePage::LocalApi)),
                applies: affected,
                platforms: Platforms::Desktop,
            },
            Notice {
                id: "fixture-library-0.10",
                since: Version::new(0, 10, 0),
                title: "Your library is now encrypted",
                what_changed: "Backups open elsewhere only with your key.",
                steps: &["Open Settings.", "Write the key down."],
                action: Some(NoticeAction::OpenSettings(NoticePage::General)),
                applies: unsaved,
                platforms: Platforms::Both,
            },
            Notice {
                id: "fixture-phones-0.11",
                since: Version::new(0, 11, 0),
                title: "Re-open the app once",
                what_changed: "The phone's index was rebuilt.",
                steps: &["Open the app."],
                action: None,
                applies: |_| true,
                platforms: Platforms::Phones,
            },
        ]
    }

    fn version() -> impl Strategy<Value = Version> {
        (0u64..2, 0u64..14, 0u64..4).prop_map(|(a, b, c)| Version::new(a, b, c))
    }

    fn context() -> impl Strategy<Value = NoticeContext> {
        (
            version(),
            proptest::option::of(version()),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(|(current, previous, had_login, signed_in, saved, plain)| {
                NoticeContext {
                    current,
                    previous,
                    had_login,
                    signed_in_to_sync: signed_in,
                    library_key_saved: saved,
                    library_plain: plain,
                }
            })
    }

    fn platform() -> impl Strategy<Value = Platform> {
        prop_oneof![Just(Platform::Desktop), Just(Platform::Phones)]
    }

    fn ids(notices: &[&Notice]) -> Vec<&'static str> {
        notices.iter().map(|n| n.id).collect()
    }

    #[test]
    fn the_shipped_list_is_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for notice in NOTICES {
            assert!(seen.insert(notice.id), "{} is listed twice", notice.id);
            assert!(!notice.title.is_empty(), "{}", notice.id);
            assert!(!notice.what_changed.is_empty(), "{}", notice.id);
            assert!(!notice.steps.is_empty(), "{} has no steps", notice.id);
        }
    }

    #[test]
    fn the_banner_shows_the_newest_notice_with_a_count() {
        let list = fixture();
        let mut ctx = NoticeContext::new(Version::new(0, 11, 0));
        ctx.had_login = true;
        let state = NoticeState::default();
        let desktop = banner(&list, &ctx, Platform::Desktop, &state).unwrap();
        assert_eq!(desktop.notice.id, "fixture-keys-0.10");
        assert_eq!(desktop.total, 2);
        assert_eq!(count_label(desktop.total).as_deref(), Some("1 of 2"));
        let phones = banner(&list, &ctx, Platform::Phones, &state).unwrap();
        assert_eq!(phones.notice.id, "fixture-phones-0.11");
        assert_eq!(count_label(1), None);
    }

    #[test]
    fn dismissing_moves_the_banner_on_but_keeps_the_notice_listed() {
        let list = fixture();
        let mut ctx = NoticeContext::new(Version::new(0, 10, 1));
        ctx.had_login = true;
        let mut state = NoticeState::default();
        state.dismiss("fixture-keys-0.10");
        let pick = banner(&list, &ctx, Platform::Desktop, &state).unwrap();
        assert_eq!(pick.notice.id, "fixture-library-0.10");
        assert_eq!(pick.total, 1);
        assert_eq!(
            ids(&listed(&list, &ctx, Platform::Desktop, &state)),
            ["fixture-keys-0.10", "fixture-library-0.10"]
        );
        state.mark_fixed("fixture-keys-0.10");
        assert_eq!(
            ids(&listed(&list, &ctx, Platform::Desktop, &state)),
            ["fixture-library-0.10"]
        );
        // What's new keeps both.
        assert_eq!(
            ids(&for_whats_new(&list, &ctx, Platform::Desktop)),
            ["fixture-keys-0.10", "fixture-library-0.10"]
        );
    }

    #[test]
    fn a_notice_waits_for_its_version() {
        let list = fixture();
        let ctx = NoticeContext::new(Version::new(0, 9, 4));
        assert!(applying(&list, &ctx, Platform::Phones).is_empty());
    }

    #[test]
    fn the_previous_version_is_remembered_across_launches() {
        let now = Version::new(0, 10, 0);
        // The launch after an update.
        assert_eq!(
            previous_version(None, Some("0.9.4"), &now),
            Some(Version::new(0, 9, 4))
        );
        // The next launch: the last run is this version, so the remembered
        // one stands.
        assert_eq!(
            previous_version(Some("0.9.4"), Some("0.10.0"), &now),
            Some(Version::new(0, 9, 4))
        );
        // A first launch knows nothing, and neither does a damaged entry.
        assert_eq!(previous_version(None, None, &now), None);
        assert_eq!(previous_version(Some("junk"), Some("junk"), &now), None);
    }

    #[test]
    fn state_reads_files_from_before_it_existed() {
        let state: NoticeState = serde_json::from_str("{}").unwrap();
        assert_eq!(state, NoticeState::default());
        let state: NoticeState =
            serde_json::from_str(r#"{"dismissed":["a"],"fixed":["b"],"later":1}"#).unwrap();
        assert!(state.is_dismissed("a") && state.is_fixed("b"));
    }

    fn api_keys() -> &'static Notice {
        NOTICES
            .iter()
            .find(|notice| notice.id == API_KEYS_ID)
            .expect("the API keys notice ships")
    }

    fn library_plain() -> &'static Notice {
        NOTICES
            .iter()
            .find(|notice| notice.id == LIBRARY_PLAIN_ID)
            .expect("the plain-library notice ships")
    }

    #[test]
    fn the_plain_library_notice_says_why_and_what_to_do() {
        let notice = library_plain();
        assert_eq!(notice.platforms, Platforms::Desktop);
        assert!(notice.what_changed.contains("nothing is lost"));
        assert!(notice.what_changed.contains("tries again"));
        assert!(notice.steps.iter().any(|step| step.contains("keychain")));
        assert!(notice.steps.iter().any(|step| step.contains("free space")));
        assert!(
            notice
                .steps
                .iter()
                .any(|step| step.contains("KANSO_SECRETS_KEY_FILE"))
        );
        // A library not signed in to sync has a way to the file store too:
        // signing in is not the only one.
        assert!(
            notice
                .steps
                .iter()
                .any(|step| step.contains("\"secretStore\": \"file\"")
                    && step.contains("isn't signed in to sync"))
        );
    }

    #[test]
    fn the_plain_library_notice_follows_the_library_not_the_update() {
        let mut ctx = NoticeContext::new(Version::new(0, 10, 0));
        assert!(!library_plain().applies_to(&ctx, Platform::Desktop));
        ctx.library_plain = true;
        assert!(library_plain().applies_to(&ctx, Platform::Desktop));
        // Encrypted later, it is gone with no one dismissing it.
        ctx.library_plain = false;
        assert!(!library_plain().applies_to(&ctx, Platform::Desktop));
    }

    #[test]
    fn the_api_keys_notice_says_where_to_go_and_how_to_update_each_tool() {
        let notice = api_keys();
        assert_eq!(
            notice.action,
            Some(NoticeAction::OpenSettings(NoticePage::LocalApi))
        );
        let steps = notice.steps.join("\n");
        assert!(steps.contains("API keys"), "{steps}");
        assert!(steps.contains("MCP client"), "{steps}");
        assert!(steps.contains("REST"), "{steps}");
        assert!(steps.contains("Authorization: Bearer"), "{steps}");
    }

    proptest! {
        /// The API keys notice is for a library that had a login, on the
        /// desktop, once the release has it. The phones have no local API,
        /// so they never show it.
        #[test]
        fn prop_the_api_keys_notice_is_for_libraries_that_had_a_login(
            ctx in context(),
            platform in platform(),
        ) {
            let expected = ctx.had_login
                && platform == Platform::Desktop
                && ctx.current >= Version::new(0, 10, 0);
            prop_assert_eq!(api_keys().applies_to(&ctx, platform), expected);
        }

        /// The plain-library notice is for a library that is plain, on the
        /// desktop, once the release has it; the phones never show it.
        #[test]
        fn prop_the_plain_library_notice_is_for_libraries_that_are_plain(
            ctx in context(),
            platform in platform(),
        ) {
            let expected = ctx.library_plain
                && platform == Platform::Desktop
                && ctx.current >= Version::new(0, 10, 0);
            prop_assert_eq!(library_plain().applies_to(&ctx, platform), expected);
        }

        /// `applies` reads only the context: asking again, or asking a copy,
        /// or asking in between other contexts, gives the same answer.
        #[test]
        fn prop_applies_is_pure(ctx in context(), other in context(), platform in platform()) {
            let list = fixture();
            for notice in list.iter().chain(NOTICES) {
                let first = notice.applies_to(&ctx, platform);
                let _ = notice.applies_to(&other, platform);
                prop_assert_eq!(first, notice.applies_to(&ctx.clone(), platform));
                prop_assert_eq!((notice.applies)(&ctx), (notice.applies)(&ctx.clone()));
            }
        }

        #[test]
        fn prop_nothing_shows_before_its_version_or_on_the_other_platform(
            ctx in context(),
            platform in platform(),
        ) {
            let list = fixture();
            for notice in applying(&list, &ctx, platform) {
                prop_assert!(ctx.current >= notice.since);
                prop_assert!(notice.platforms.includes(platform));
                prop_assert!((notice.applies)(&ctx));
            }
        }

        /// Dismissing persists through the file and is idempotent: once or
        /// five times is the same state, and other notices are untouched.
        #[test]
        fn prop_dismissal_persists_and_is_idempotent(
            first in proptest::collection::vec("[a-z0-9.-]{1,12}", 0..6),
            id in "[a-z0-9.-]{1,12}",
            times in 1usize..5,
        ) {
            let mut state = NoticeState::default();
            for other in &first {
                state.dismiss(other);
            }
            let before = state.clone();
            let mut once = state.clone();
            once.dismiss(&id);
            for _ in 0..times {
                state.dismiss(&id);
            }
            prop_assert_eq!(&state, &once);
            prop_assert!(state.is_dismissed(&id));
            for other in &before.dismissed {
                prop_assert!(state.is_dismissed(other));
            }
            // A second dismissal says nothing changed.
            prop_assert!(!state.dismiss(&id));
            // Through config.json and back.
            let json = serde_json::to_string(&state).unwrap();
            let back: NoticeState = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&back, &state);
            prop_assert!(back.is_dismissed(&id));
            let mut again = back.clone();
            again.dismiss(&id);
            prop_assert_eq!(again, back);
            // No duplicates.
            let mut sorted = state.dismissed.clone();
            sorted.sort();
            sorted.dedup();
            prop_assert_eq!(sorted.len(), state.dismissed.len());
        }

        #[test]
        fn prop_fixing_is_idempotent_and_leaves_dismissals_alone(
            id in "[a-z0-9.-]{1,12}",
            other in "[a-z0-9.-]{1,12}",
        ) {
            let mut state = NoticeState::default();
            state.dismiss(&other);
            let mut once = state.clone();
            once.mark_fixed(&id);
            state.mark_fixed(&id);
            state.mark_fixed(&id);
            prop_assert_eq!(&state, &once);
            prop_assert!(state.is_fixed(&id));
            prop_assert!(state.is_dismissed(&other));
            prop_assert!(!state.banner_allowed(&id));
        }

        /// Whatever the last run and the remembered version were, the
        /// answer is never newer than the version running, and asking again
        /// at the next launch gives the same answer.
        #[test]
        fn prop_previous_is_never_newer_than_current(
            remembered in proptest::option::of(version()),
            last in proptest::option::of(version()),
            current in version(),
        ) {
            let text = |v: &Option<Version>| v.as_ref().map(ToString::to_string);
            let answer = previous_version(
                text(&remembered).as_deref(),
                text(&last).as_deref(),
                &current,
            );
            if let Some(answer) = &answer {
                let from_update = last.as_ref().is_some_and(|l| l < &current && l == answer);
                prop_assert!(from_update || Some(answer) == remembered.as_ref());
            }
            let again = previous_version(
                answer.as_ref().map(ToString::to_string).as_deref(),
                Some(&current.to_string()),
                &current,
            );
            prop_assert_eq!(again, answer);
        }

        /// The order doesn't depend on how the list was written down, and
        /// ordering twice changes nothing.
        #[test]
        fn prop_ordering_is_stable(seed in any::<u64>()) {
            let list = fixture();
            let expected = ids(&ordered(&list));
            let mut shuffled = list.clone();
            fastrand::seed(seed);
            fastrand::shuffle(&mut shuffled);
            prop_assert_eq!(ids(&ordered(&shuffled)), expected.clone());
            let again: Vec<Notice> = ordered(&shuffled).into_iter().cloned().collect();
            prop_assert_eq!(ids(&ordered(&again)), expected);
        }

        #[test]
        fn prop_the_banner_never_shows_a_hidden_notice(
            ctx in context(),
            platform in platform(),
            dismissed in proptest::collection::vec(0usize..3, 0..3),
            fixed in proptest::collection::vec(0usize..3, 0..3),
        ) {
            let list = fixture();
            let mut state = NoticeState::default();
            for ix in dismissed {
                state.dismiss(list[ix].id);
            }
            for ix in fixed {
                state.mark_fixed(list[ix].id);
            }
            if let Some(pick) = banner(&list, &ctx, platform, &state) {
                prop_assert!(state.banner_allowed(pick.notice.id));
                prop_assert!(pick.notice.applies_to(&ctx, platform));
                prop_assert!(pick.total >= 1);
            }
            for notice in listed(&list, &ctx, platform, &state) {
                prop_assert!(!state.is_fixed(notice.id));
            }
        }
    }
}
