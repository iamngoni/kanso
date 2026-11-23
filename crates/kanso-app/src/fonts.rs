//! The fonts someone can pick for the editor and the preview (Settings ›
//! Editor): every family installed on this computer, and every Google Font
//! downloaded to it, read once and again whenever one is added.

use std::sync::Arc;

use gpui_kit::{App, Global, SharedString};

/// The installed font families, sorted, as the system lists them.
struct Installed(Arc<[SharedString]>);

impl Global for Installed {}

/// Every font family installed here, sorted; read from the system the first
/// time and kept.
// no-prop-test: asks the system; `family` has the properties.
pub fn installed(cx: &mut App) -> Arc<[SharedString]> {
    if let Some(Installed(names)) = cx.try_global::<Installed>() {
        return names.clone();
    }
    refresh(cx)
}

/// Reads the installed families again, after fonts were added to the text
/// system (a downloaded Google Font, `crate::google_fonts`).
// no-prop-test: asks the system; `family` has the properties.
pub fn refresh(cx: &mut App) -> Arc<[SharedString]> {
    let names: Arc<[SharedString]> = cx
        .text_system()
        .all_font_names()
        .into_iter()
        .filter(|name| !name.starts_with('.'))
        .map(SharedString::from)
        .collect();
    cx.set_global(Installed(names.clone()));
    names
}

/// The family to draw with: `choice` when it is installed, else `default`
/// (a font someone picked and later removed falls back). Pure.
pub fn family(
    choice: Option<&str>,
    installed: &[SharedString],
    default: SharedString,
) -> SharedString {
    choice
        .filter(|name| installed.iter().any(|i| i.as_ref() == *name))
        .map(|name| SharedString::from(name.to_owned()))
        .unwrap_or(default)
}

/// A font picker's first entry: Kanso's own font.
pub const DEFAULT: &str = "Default";

/// A font picker's entries: [`DEFAULT`], then every installed family.
pub fn choices(installed: &[SharedString]) -> Vec<SharedString> {
    std::iter::once(SharedString::from(DEFAULT))
        .chain(installed.iter().cloned())
        .collect()
}

/// The setting an entry picks: `None` for [`DEFAULT`], else its family.
pub fn picked(entry: &str) -> Option<String> {
    (entry != DEFAULT).then(|| entry.to_owned())
}

/// Where `choice` is among `choices`: [`DEFAULT`]'s place for none, or
/// for a font no longer installed.
pub fn position(choice: Option<&str>, choices: &[SharedString]) -> usize {
    choice
        .and_then(|name| choices.iter().skip(1).position(|c| c.as_ref() == name))
        .map_or(0, |ix| ix + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    #[test]
    fn a_picked_font_draws_while_it_is_installed() {
        let installed: Vec<SharedString> = vec!["Georgia".into(), "Menlo".into()];
        let default = SharedString::from("System");
        assert_eq!(
            family(Some("Georgia"), &installed, default.clone()),
            "Georgia"
        );
        assert_eq!(
            family(Some("Gone Sans"), &installed, default.clone()),
            "System"
        );
        assert_eq!(family(None, &installed, default.clone()), "System");
        assert_eq!(family(Some("georgia"), &installed, default), "System");
    }

    #[test]
    fn the_picker_lists_default_first() {
        let installed: Vec<SharedString> = vec!["Georgia".into(), "Menlo".into()];
        let list = choices(&installed);
        assert_eq!(list[0], DEFAULT);
        assert_eq!(position(Some("Menlo"), &list), 2);
        assert_eq!(position(Some("Gone"), &list), 0);
        assert_eq!(position(None, &list), 0);
        assert_eq!(picked(DEFAULT), None);
        assert_eq!(picked("Menlo").as_deref(), Some("Menlo"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Picking an entry and showing the picker again marks that entry:
        /// the setting round-trips through the list.
        #[test]
        fn prop_a_pick_round_trips(
            installed in prop::collection::btree_set("[A-Za-z][A-Za-z ]{0,9}", 0..8),
            ix in any::<prop::sample::Index>(),
        ) {
            let installed: Vec<SharedString> = installed
                .into_iter()
                .filter(|name| name != DEFAULT)
                .map(SharedString::from)
                .collect();
            let list = choices(&installed);
            let at = ix.index(list.len());
            let setting = picked(&list[at]);
            prop_assert_eq!(position(setting.as_deref(), &list), at);
            prop_assert_eq!(setting.is_none(), at == 0);
        }

        /// The family drawn is the choice exactly when it is installed, and
        /// the default otherwise; never anything else.
        #[test]
        fn prop_the_choice_or_the_default(
            installed in prop::collection::vec("[A-Za-z ]{1,10}", 0..8),
            choice in prop::option::of("[A-Za-z ]{1,10}"),
        ) {
            let installed: Vec<SharedString> = installed.into_iter().map(SharedString::from).collect();
            let default = SharedString::from("Default Face");
            let drawn = family(choice.as_deref(), &installed, default.clone());
            let available = choice
                .as_deref()
                .is_some_and(|c| installed.iter().any(|i| i.as_ref() == c));
            if available {
                prop_assert_eq!(drawn.as_ref(), choice.as_deref().unwrap());
            } else {
                prop_assert_eq!(drawn, default);
            }
        }
    }
}
