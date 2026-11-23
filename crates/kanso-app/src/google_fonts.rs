//! Google Fonts in the running app: the downloaded fonts are handed to the
//! window system under their family name, at launch from the cache (so they
//! work offline) and again whenever the Download button fetches one.
//!
//! What a family name may be, how Google Fonts is asked and how the files
//! are kept are `kanso_engine::google_fonts`'s; the field and the button are
//! `settings_google_fonts`. This module owns the one thing both share: a
//! family is registered with the text system at most once, and only counts
//! as registered when the system lists it under the name the settings store.

use std::borrow::Cow;
use std::collections::BTreeSet;

use gpui_kit::{App, Global, SharedString};
use kanso_engine::google_fonts::{Cache, Family, FontFile, HttpSource, load};

use crate::workspace::AppState;

/// The families already given to the text system. Adding one twice would
/// keep two copies of it.
#[derive(Default)]
struct Registered(BTreeSet<Family>);

impl Global for Registered {}

/// Why a font couldn't be registered.
#[derive(Debug, PartialEq, Eq)]
pub enum RegisterError {
    /// The text system refused the file.
    Rejected(String),
    /// The system took the file but lists no family by this name.
    NotListed(Family),
}

impl std::fmt::Display for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterError::Rejected(why) => write!(f, "Kanso couldn't load that font file: {why}"),
            RegisterError::NotListed(family) => write!(
                f,
                "The font file doesn't call itself {family}, so Kanso can't use it by that name."
            ),
        }
    }
}

impl std::error::Error for RegisterError {}

/// The downloaded fonts' folder, in this library's data folder.
pub fn cache(cx: &App) -> Cache {
    Cache::new(&cx.global::<AppState>().data_dir)
}

/// Whether the system's family `names` include `family`, spelled exactly:
/// the settings store the name the font file gives, and a font that the
/// system lists under another name would draw as the fallback.
pub fn is_listed(names: &[SharedString], family: &Family) -> bool {
    names.iter().any(|name| name.as_ref() == family.as_str())
}

/// Gives `file` to the text system as `family`, then reads the installed
/// families again. A family already given is left as it is.
// no-prop-test: the text system is the platform's; `is_listed` has the
// properties, and the test platform's text system registers nothing.
pub fn register(family: &Family, file: &FontFile, cx: &mut App) -> Result<(), RegisterError> {
    if cx.default_global::<Registered>().0.contains(family) {
        return Ok(());
    }
    cx.text_system()
        .add_fonts(vec![Cow::Owned(file.as_bytes().to_vec())])
        .map_err(|err| RegisterError::Rejected(err.to_string()))?;
    cx.default_global::<Registered>().0.insert(family.clone());
    let names = crate::fonts::refresh(cx);
    if is_listed(&names, family) {
        Ok(())
    } else {
        Err(RegisterError::NotListed(family.clone()))
    }
}

/// Registers every font kept in the data folder, before the first window is
/// drawn, then downloads again any the settings list but the folder lacks.
/// None of it needs the network when the cache is whole.
// no-prop-test: reads the data folder and the platform; the cache and the
// load are tested in `kanso_engine::google_fonts`.
pub fn init(cx: &mut App) {
    let scan = cache(cx).scan();
    for failure in &scan.failed {
        eprintln!("kanso: couldn't read the downloaded fonts: {failure}");
    }
    for cached in &scan.found {
        if let Err(err) = register(&cached.family, &cached.file, cx) {
            eprintln!("kanso: Google Font {}: {err}", cached.family);
        }
    }
    restore_missing(cx);
}

/// The families the settings list that the text system doesn't have, to
/// download. A name that isn't a family is left out.
fn missing(cx: &App) -> Vec<Family> {
    let registered = cx
        .try_global::<Registered>()
        .map(|registered| registered.0.clone())
        .unwrap_or_default();
    cx.global::<AppState>()
        .config
        .appearance
        .google_fonts
        .iter()
        .filter_map(|name| Family::parse(name).ok())
        .filter(|family| !registered.iter().any(|known| known.same_as(family)))
        .collect()
}

/// Downloads, in the background, the fonts the settings list but the cache
/// lacks (it was cleared, or the library was copied here). Offline it does
/// nothing but say so; those fonts draw as the default until a download
/// works.
// no-prop-test: starts the network; `kanso_engine::google_fonts::load`
// is tested with a source in memory.
fn restore_missing(cx: &mut App) {
    let families = missing(cx);
    if families.is_empty() {
        return;
    }
    let cache = cache(cx);
    cx.spawn(async move |cx| {
        for family in families {
            let cache = cache.clone();
            let wanted = family.clone();
            let loaded = cx
                .background_executor()
                .spawn(async move { load(&HttpSource::default(), &cache, &wanted, &mut |_| {}) })
                .await;
            cx.update(|cx| match loaded {
                Ok(loaded) => {
                    if let Err(err) = register(&loaded.family, &loaded.file, cx) {
                        eprintln!("kanso: Google Font {family}: {err}");
                    }
                    cx.refresh_windows();
                }
                Err(err) => eprintln!("kanso: couldn't restore Google Font {family}: {err}"),
            });
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::with_cases(128)
        })]

        /// A family is listed only under its exact spelling.
        #[test]
        fn prop_a_family_is_listed_by_its_exact_name(
            names in prop::collection::vec("[A-Za-z][A-Za-z ]{0,9}", 0..6),
            wanted in "[A-Za-z][A-Za-z ]{0,9}",
        ) {
            let Ok(family) = Family::parse(&wanted) else { return Ok(()) };
            let listed: Vec<SharedString> = names.iter().cloned().map(SharedString::from).collect();
            prop_assert_eq!(
                is_listed(&listed, &family),
                names.iter().any(|name| name == family.as_str())
            );
            let mut with = listed.clone();
            with.push(SharedString::from(family.as_str().to_owned()));
            prop_assert!(is_listed(&with, &family));
        }
    }
}
