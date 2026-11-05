//! Update notices in the desktop app (issue #243, "Update notices"): what
//! the banner, the How to fix sheet, What's new and the palette show.
//!
//! The list, the rules and the dismissal state are `kanso_core::notices`;
//! this module holds the list the app runs with, builds the context a notice
//! is checked against, and writes what the person did to `config.json`. The
//! drawing is in `notice_views`.

use gpui_kit::*;
use kanso_core::notices::{
    self, BannerPick, NOTICES, Notice, NoticeAction, NoticeContext, NoticePage, Platform,
};
use semver::Version;

use crate::services::Services;
use crate::settings::{
    OpenAccountSettings, OpenApiSettings, OpenGeneralSettings, OpenUpdateSettings,
};
use crate::workspace::AppState;

/// Store meta key: the version that ran before the last update.
pub const META_PREVIOUS: &str = "update.previous_version";

/// Why the library opened plain this run (the system's key store refused its
/// key, or there wasn't room to encrypt it), set by the app as it starts. Its
/// presence is what the "Your library isn't encrypted yet" notice looks at.
pub struct PlainLibrary(pub String);

impl Global for PlainLibrary {}

/// The notices this run knows, and the version that ran before the last
/// update. Tests and snapshots put their own list in.
pub struct NoticeBoard {
    pub list: &'static [Notice],
    pub previous: Option<Version>,
}

impl Global for NoticeBoard {}

/// Sets the board up for the real app.
pub fn init(previous: Option<Version>, cx: &mut App) {
    cx.set_global(NoticeBoard {
        list: NOTICES,
        previous,
    });
}

/// What a notice is checked against. `login` is whether `config.json` held
/// a local API login before this update; `library_key_saved` whether the
/// library key has been saved.
pub fn build_context(
    current: Version,
    previous: Option<Version>,
    login: bool,
    signed_in_to_sync: bool,
    library_key_saved: bool,
    library_plain: bool,
) -> NoticeContext {
    NoticeContext {
        current,
        previous,
        had_login: login,
        signed_in_to_sync,
        library_key_saved,
        library_plain,
    }
}

/// The context now. "A login existed" is the fact `config.json` recorded
/// when it dropped the login; "the library key was saved" is what the person
/// chose on the library key's sheet, kept in `config.json` too.
fn context(cx: &App) -> NoticeContext {
    let board = cx.try_global::<NoticeBoard>();
    let signed_in = cx
        .try_global::<Services>()
        .is_some_and(|services| services.signed_in() || services.has_locked_session());
    let login = cx
        .try_global::<AppState>()
        .is_some_and(|state| state.config.had_login);
    let key_saved = crate::library_key_sheet::key_saved(cx);
    build_context(
        crate::updates::current_version(),
        board.and_then(|board| board.previous.clone()),
        login,
        signed_in,
        key_saved,
        // Set once, as the library opens: the secret store refused its key or
        // there wasn't room to encrypt it.
        cx.has_global::<PlainLibrary>(),
    )
}

/// Why the library is plain, for the notice's sheet: what Kanso found when
/// it tried. `None` for a notice that has no such detail.
pub fn detail(id: &str, cx: &App) -> Option<String> {
    (id == notices::LIBRARY_PLAIN_ID)
        .then(|| cx.try_global::<PlainLibrary>().map(|plain| plain.0.clone()))
        .flatten()
}

fn list(cx: &App) -> &'static [Notice] {
    cx.try_global::<NoticeBoard>()
        .map_or(NOTICES, |board| board.list)
}

fn state(cx: &App) -> kanso_core::notices::NoticeState {
    cx.try_global::<AppState>()
        .map(|state| state.config.notices.clone())
        .unwrap_or_default()
}

/// The notice for the banner and how many wait, or nothing.
pub fn banner(cx: &App) -> Option<BannerPick<'static>> {
    notices::banner(list(cx), &context(cx), Platform::Desktop, &state(cx))
}

/// Every notice for the palette's list: those that apply, minus the ones
/// marked fixed.
pub fn listed(cx: &App) -> Vec<&'static Notice> {
    notices::listed(list(cx), &context(cx), Platform::Desktop, &state(cx))
}

/// The notices What's new lists under "Needs your attention".
pub fn for_whats_new(cx: &App) -> Vec<&'static Notice> {
    notices::for_whats_new(list(cx), &context(cx), Platform::Desktop)
}

/// The notice with this ID, if this version has it.
pub fn find(id: &str, cx: &App) -> Option<&'static Notice> {
    list(cx).iter().find(|notice| notice.id == id)
}

/// Hides the banner for `id` on this computer for good.
pub fn dismiss(id: &str, cx: &mut App) {
    let id = id.to_owned();
    AppState::update_config(cx, move |config| {
        config.notices.dismiss(&id);
    });
    cx.refresh_windows();
}

/// Hides the banner and the palette entry for `id`.
pub fn mark_fixed(id: &str, cx: &mut App) {
    let id = id.to_owned();
    AppState::update_config(cx, move |config| {
        config.notices.mark_fixed(&id);
    });
    cx.refresh_windows();
}

/// The words on a notice's button.
pub fn action_label(action: NoticeAction) -> &'static str {
    match action {
        NoticeAction::OpenSettings(_) => "Open Settings",
        NoticeAction::OpenUrl(_) => "Learn more",
    }
}

/// Does what a notice's button says.
// no-prop-test: wiring; `settings_action` maps every page, tested.
pub fn run_action(action: NoticeAction, window: &mut Window, cx: &mut App) {
    match action {
        NoticeAction::OpenSettings(page) => window.dispatch_action(settings_action(page), cx),
        NoticeAction::OpenUrl(url) => {
            if crate::preview::is_openable(url) {
                cx.open_url(url);
            }
        }
    }
}

/// The action that opens a page of Settings.
pub fn settings_action(page: NoticePage) -> Box<dyn Action> {
    match page {
        NoticePage::General => Box::new(OpenGeneralSettings),
        NoticePage::Account => Box::new(OpenAccountSettings),
        NoticePage::LocalApi => Box::new(OpenApiSettings),
        NoticePage::Updates => Box::new(OpenUpdateSettings),
    }
}

#[cfg(test)]
mod tests {
    use super::settings_action;
    use kanso_core::notices::NoticePage;
    #[test]
    fn every_page_has_a_settings_action() {
        let names: std::collections::HashSet<_> = [
            NoticePage::General,
            NoticePage::Account,
            NoticePage::LocalApi,
            NoticePage::Updates,
        ]
        .into_iter()
        .map(|page| settings_action(page).name())
        .collect();
        assert_eq!(names.len(), 4);
    }
}
