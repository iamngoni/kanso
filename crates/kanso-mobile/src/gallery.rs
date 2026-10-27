//! The theme gallery: the signed index, installing and removing themes.
//!
//! The index is fetched, verified and installed from exactly as the desktop's
//! (`kanso-app/src/settings_gallery.rs`): the Ed25519 signature over the
//! index's exact bytes against the themes keys compiled into the gallery
//! crate, then the serial floor (no rollback, no rewrite under one serial),
//! the catalog's entry rules, and for each file its pinned size and SHA-256.
//! A theme that fails any check is never written. Everything lives in
//! `gallery/` beside the library's database, laid out as the desktop lays out
//! its own `gallery/` folder: `themes/<id>.json`, `icon-themes/<id>/`,
//! `state.json`, the last good `index.json` with its `.sig`, and `meta.json`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use kanso_engine::theme::builtin::BUILTIN;
use kanso_engine::theme::file::ThemeFile;
use kanso_gallery::state::Meta;
use kanso_gallery::{
    Action, Appearance, Index, IndexError, Installed, Kind, PlanState, State, Theme, is_valid_name,
};
use semver::Version;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::gallery_model::{GalleryColors, GalleryItem, GalleryKind};
use crate::gallery_net::{CheckError, Source, cached, fetch_and_check};
use crate::gallery_store;
use crate::library::Inner;
use crate::model::{BuiltinTheme, ThemeTokens};
use crate::theme_tokens;

/// One catalog check or install at a time: they share the cached index, the
/// floor in `meta.json` and `state.json`.
static CATALOG: Mutex<()> = Mutex::new(());

/// The theme gallery: the signed index, installing and removing themes.
#[derive(Clone)]
pub struct Gallery {
    pub(crate) inner: Arc<Inner>,
}

impl Gallery {
    /// The gallery's themes, verified. With `refresh` the signed index is
    /// fetched first, and a revocation in it is applied to the installed
    /// themes (a withdrawn theme is removed, or disabled if it was edited).
    /// Without it, the last good index is read from disk and verified again;
    /// with none on disk, it is fetched.
    ///
    /// A fetch the network fails (offline, rate limited, the server down) is
    /// answered from the last good index when there is one. An index that
    /// fails a check (a bad signature, a rollback, a rewrite under one
    /// serial, an entry Kanso can't use, a catalog from a newer Kanso) is an
    /// error, and nothing of it is kept.
    pub fn index(&self, refresh: bool) -> Result<Vec<GalleryItem>, KansoError> {
        guard(|| {
            let _checking = held(&CATALOG);
            let index = self.catalog(refresh)?;
            Ok(self.items(Some(&index)))
        })
    }

    /// Installs the theme `id` from the verified index (fetching the index
    /// if none is on disk), after its file matches the size and SHA-256 the
    /// index pins and, for a color theme, reads well and does not take a
    /// built-in's name. A theme that is already installed is updated, unless
    /// its copy was edited. Returns the list as [`Gallery::index`] does.
    pub fn install(&self, id: String) -> Result<Vec<GalleryItem>, KansoError> {
        guard(|| {
            let _installing = held(&CATALOG);
            let index = self.catalog(false)?;
            let source = self.source();
            let theme = index.theme(&id).ok_or(KansoError::NotFound)?.clone();
            let current = current_version();
            if !theme.runs_on(&current) {
                return Err(KansoError::invalid(format!(
                    "{} needs a newer Kanso.",
                    theme.name
                )));
            }
            let payload = theme.payload().ok_or(KansoError::NotFound)?.clone();
            let cap = match theme.kind {
                Kind::Color => kanso_gallery::MAX_COLOR_BYTES,
                Kind::Icons => kanso_gallery::MAX_BUNDLE_BYTES,
            };
            let bytes = kanso_update::fetch::get_bytes(
                source.transport.as_ref(),
                &source.policy,
                &kanso_gallery::file_url(&payload.path),
                cap,
            )
            .map_err(|err| {
                KansoError::network(format!("Couldn't download {}: {err}", theme.name))
            })?;
            kanso_gallery::check_pin(&payload, &bytes, cap).map_err(|err| {
                KansoError::invalid(format!("{} didn't match the catalog: {err}.", theme.name))
            })?;
            let root = self.root();
            let installed = match theme.kind {
                Kind::Color => {
                    gallery_store::install_color(&root, &theme, &bytes, &reserved_names(), false)
                }
                Kind::Icons => gallery_store::install_icons(
                    &root,
                    &theme,
                    &bytes,
                    &HashSet::new(),
                    &HashSet::new(),
                    false,
                ),
            };
            installed.map_err(|message| {
                KansoError::invalid(format!("Couldn't install {}: {message}", theme.name))
            })?;
            Ok(self.items(Some(&index)))
        })
    }

    /// Removes the installed theme `id` and its record.
    /// [`KansoError::NotFound`] for one that isn't installed.
    pub fn uninstall(&self, id: String) -> Result<Vec<GalleryItem>, KansoError> {
        guard(|| {
            let _removing = held(&CATALOG);
            let root = self.root();
            let kind = gallery_store::reconcile(&root)
                .record(&id)
                .map(|record| record.kind)
                .ok_or(KansoError::NotFound)?;
            gallery_store::remove(&root, kind, &id).map_err(KansoError::storage)?;
            let index = self.cached_index();
            Ok(self.items(index.as_ref()))
        })
    }

    /// The installed color themes, as [`crate::Renderer::builtin_themes`]
    /// lists the built-in ones, each fitted through the same code. A theme
    /// the catalog withdrew after it was edited is not in the list; neither
    /// is one whose file no longer reads.
    pub fn installed_themes(&self) -> Result<Vec<BuiltinTheme>, KansoError> {
        guard(|| {
            let root = self.root();
            let state = gallery_store::reconcile(&root);
            Ok(state
                .installed
                .iter()
                .filter(|record| record.kind == Kind::Color)
                .filter(|record| !state.disabled.contains(&record.id))
                .filter_map(|record| installed_row(&root, &record.id))
                .collect())
        })
    }

    /// The installed color theme `id`, fitted as the built-in ones are.
    /// [`KansoError::NotFound`] for one that isn't installed (or is
    /// disabled).
    pub fn installed_theme_tokens(&self, id: String) -> Result<ThemeTokens, KansoError> {
        guard(|| {
            let root = self.root();
            let text = installed_text(&root, &id)?;
            theme_tokens::fit(&text)
        })
    }

    /// Reads and trusts `transport` and `keys` instead of the real server and
    /// the keys compiled in, so a test serves a signed index from memory.
    #[cfg(any(test, feature = "test-support"))]
    pub fn use_source_for_test(
        &self,
        transport: Arc<dyn kanso_update::Transport>,
        keys: Vec<kanso_update::VerifyingKey>,
    ) {
        *held(&self.inner.gallery_source) = Some(Source {
            transport,
            keys,
            policy: kanso_gallery::GalleryPolicy::GALLERY,
            index_url: kanso_gallery::INDEX_URL.to_owned(),
        });
    }

    fn source(&self) -> Source {
        held(&self.inner.gallery_source)
            .clone()
            .unwrap_or_else(Source::production)
    }

    fn root(&self) -> PathBuf {
        gallery_store::root_of(&self.inner.path)
    }

    /// The last good index, verified again against the keys and the floor.
    fn cached_index(&self) -> Option<Index> {
        let root = self.root();
        let floor = Meta::load(&root.join("meta.json")).floor();
        cached(&root, &self.source(), &floor, &current_version()).map(|(_, _, index)| index)
    }

    /// The verified index: the cached one, or a fetched one.
    fn catalog(&self, refresh: bool) -> Result<Index, KansoError> {
        let source = self.source();
        if source.keys.is_empty() {
            return Err(KansoError::invalid(
                "The theme gallery isn't set up in this build.",
            ));
        }
        let root = self.root();
        let meta_path = root.join("meta.json");
        let mut meta = Meta::load(&meta_path);
        let current = current_version();
        if !refresh && let Some((_, _, index)) = cached(&root, &source, &meta.floor(), &current) {
            return Ok(index);
        }
        let now = now_ms();
        match fetch_and_check(
            &source,
            meta.index_etag.as_deref(),
            &root,
            &meta.floor(),
            &current,
        ) {
            Ok(checked) => {
                // Each file is written atomically before the floor moves. A
                // crash between the two leaves a pair that doesn't verify;
                // the next start then refetches instead of trusting a mixed
                // pair.
                kanso_gallery::state::write_atomic(&root.join("index.json"), &checked.bytes)
                    .and_then(|()| {
                        kanso_gallery::state::write_atomic(
                            &root.join("index.json.sig"),
                            &checked.signature,
                        )
                    })
                    .map_err(|err| {
                        KansoError::storage(format!("Couldn't store the theme catalog: {err}"))
                    })?;
                meta.accept(
                    checked.index.serial,
                    kanso_gallery::sha256_hex(&checked.bytes),
                    checked.etag,
                );
                meta.last_checked_ms = Some(now);
                meta.last_error = None;
                meta.save(&meta_path).ok();
                let plan =
                    kanso_gallery::plan(&checked.index, &gallery_store::installed(&root), &current);
                apply_revocations(&plan.actions, &root);
                Ok(checked.index)
            }
            Err(err) => {
                meta.last_error = Some(err.to_string());
                meta.last_checked_ms = Some(now);
                meta.save(&meta_path).ok();
                if err.is_offline()
                    && let Some((_, _, index)) = cached(&root, &source, &meta.floor(), &current)
                {
                    return Ok(index);
                }
                Err(check_error(err))
            }
        }
    }

    /// The catalog's themes (or, with no catalog, only the installed ones) as
    /// the page lists them.
    fn items(&self, index: Option<&Index>) -> Vec<GalleryItem> {
        let root = self.root();
        let records = gallery_store::installed(&root);
        let state = State::load(&root.join("state.json"));
        let current = current_version();
        let generated_ms = index
            .and_then(|index| rfc3339_ms(&index.generated_at))
            .unwrap_or(0);
        let mut items = Vec::new();
        if let Some(index) = index {
            for theme in &index.themes {
                let record = records.iter().find(|record| record.id == theme.id);
                let plan_state =
                    record.map(|record| kanso_gallery::plan::state_for(record, theme, &current));
                items.push(item_of(
                    theme,
                    plan_state,
                    record,
                    &state,
                    &current,
                    generated_ms,
                ));
            }
        }
        for record in &records {
            if index.is_some_and(|index| index.theme(&record.id).is_some()) {
                continue;
            }
            items.push(GalleryItem {
                id: record.id.clone(),
                name: record.name.clone(),
                kind: kind_of(record.kind),
                author: String::new(),
                official: false,
                version: record.version.to_string(),
                installs: 0,
                updated_ms: rfc3339_ms(&record.installed_at).unwrap_or(0),
                source: None,
                dark: false,
                headline: None,
                installed: !record.is_missing(),
                update_available: false,
                runs_here: true,
                description: String::new(),
                license: String::new(),
                family: record.family.clone(),
                edited: !record.is_missing() && !record.is_unedited(),
                disabled: state.disabled.contains(&record.id),
            });
        }
        items
    }
}

fn item_of(
    theme: &Theme,
    plan_state: Option<PlanState>,
    record: Option<&Installed>,
    state: &State,
    current: &Version,
    generated_ms: i64,
) -> GalleryItem {
    GalleryItem {
        id: theme.id.clone(),
        name: theme.name.clone(),
        kind: kind_of(theme.kind),
        author: theme.author.clone(),
        official: theme.source.starts_with("https://github.com/iamngoni/"),
        version: theme.version.to_string(),
        installs: 0,
        updated_ms: generated_ms,
        source: Some(theme.source.clone()),
        dark: theme.kind == Kind::Color && theme.appearance == Some(Appearance::Dark),
        headline: theme.colors.as_ref().and_then(headline_of),
        installed: record.is_some() && plan_state != Some(PlanState::Missing),
        update_available: plan_state == Some(PlanState::Update),
        runs_here: theme.runs_on(current),
        description: kanso_gallery::clean_text(&theme.description),
        license: theme.license.clone(),
        family: theme.family.clone(),
        edited: plan_state == Some(PlanState::Edited),
        disabled: state.disabled.contains(&theme.id),
    }
}

fn kind_of(kind: Kind) -> GalleryKind {
    match kind {
        Kind::Color => GalleryKind::ColorTheme,
        Kind::Icons => GalleryKind::IconTheme,
    }
}

fn headline_of(colors: &kanso_gallery::HeadlineColors) -> Option<GalleryColors> {
    Some(GalleryColors {
        editor: hex_rgb(&colors.editor)?,
        text: hex_rgb(&colors.text)?,
        accent: hex_rgb(&colors.accent)?,
        sidebar: hex_rgb(&colors.sidebar)?,
        list: hex_rgb(&colors.list)?,
    })
}

/// `#RGB`, `#RRGGBB` or `#RRGGBBAA` (the alpha dropped) as `0xRRGGBB`.
pub(crate) fn hex_rgb(text: &str) -> Option<u32> {
    let digits = text.strip_prefix('#')?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        3 => {
            let value = u32::from_str_radix(digits, 16).ok()?;
            let channel = |shift: u32| {
                let nibble = (value >> shift) & 0xF;
                nibble << 4 | nibble
            };
            Some(channel(8) << 16 | channel(4) << 8 | channel(0))
        }
        6 => u32::from_str_radix(digits, 16).ok(),
        8 => u32::from_str_radix(&digits[..6], 16).ok(),
        _ => None,
    }
}

/// A catalog that failed to be fetched or checked, as the sentence the page
/// shows.
fn check_error(err: CheckError) -> KansoError {
    match err {
        CheckError::Index(IndexError::NewerSchema) => {
            KansoError::invalid("The theme gallery needs a newer Kanso.")
        }
        CheckError::Index(err) => KansoError::invalid(format!(
            "The theme gallery's catalog was refused and not used: {err}."
        )),
        err if err.is_offline() => {
            KansoError::network("Kanso couldn't reach the theme gallery. It tries again later.")
        }
        err => KansoError::network(format!("The theme gallery couldn't be checked: {err}.")),
    }
}

/// Applies the revocations of a verified plan: a withdrawn theme is removed,
/// or, if the person had edited it, kept on disk and disabled. Updates are
/// not applied here; the page offers them and `install` does them.
fn apply_revocations(actions: &[Action], root: &Path) {
    for action in actions {
        match action {
            Action::Remove { id, kind, reason } => match gallery_store::remove(root, *kind, id) {
                Ok(()) => eprintln!("kanso: removed revoked gallery theme {id}: {reason}"),
                Err(err) => eprintln!("kanso: removing revoked {id} failed: {err}"),
            },
            Action::Disable { id, reason, .. } => {
                if gallery_store::disable(root, id) {
                    eprintln!("kanso: disabled revoked, edited gallery theme {id}: {reason}");
                }
            }
            Action::Update(_) => {}
        }
    }
}

/// The text of the installed, enabled color theme `id`.
fn installed_text(root: &Path, id: &str) -> Result<String, KansoError> {
    if !is_valid_name(id) {
        return Err(KansoError::NotFound);
    }
    let state = gallery_store::reconcile(root);
    let known = state
        .record(id)
        .is_some_and(|record| record.kind == Kind::Color);
    if !known || state.disabled.iter().any(|other| other == id) {
        return Err(KansoError::NotFound);
    }
    std::fs::read_to_string(root.join("themes").join(format!("{id}.json")))
        .map_err(|_| KansoError::NotFound)
}

fn installed_row(root: &Path, id: &str) -> Option<BuiltinTheme> {
    let text = installed_text(root, id).ok()?;
    let file = ThemeFile::parse(&text).ok()?;
    let (tokens, _) = theme_tokens::fit_file(&file);
    Some(BuiltinTheme {
        id: id.to_owned(),
        name: file.name.clone(),
        author: file.author.clone(),
        license: file.license.clone(),
        source: file.source.clone(),
        dark: tokens.dark,
        tokens,
    })
}

/// The lowercased names of the built-in themes, which a gallery theme must
/// not take.
fn reserved_names() -> HashSet<String> {
    static NAMES: OnceLock<HashSet<String>> = OnceLock::new();
    NAMES
        .get_or_init(|| {
            BUILTIN
                .iter()
                .filter_map(|(_, text)| ThemeFile::parse(text).ok())
                .map(|file| file.name.to_lowercase())
                .collect()
        })
        .clone()
}

fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the crate's version is semver")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}

/// An RFC 3339 time (`2026-10-23T12:00:00Z`, with or without a fraction of a
/// second, `Z` or an offset) as milliseconds since the epoch. `None` for
/// anything else.
pub(crate) fn rfc3339_ms(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() < 20 || !text.is_ascii() {
        return None;
    }
    let number = |from: usize, to: usize| -> Option<i64> {
        let digits = text.get(from..to)?;
        digits
            .bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| digits.parse().ok())?
    };
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || !matches!(bytes[10], b'T' | b't' | b' ')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let mut at = 19;
    let mut millis = 0;
    if bytes[at] == b'.' {
        let start = at + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return None;
        }
        let fraction = &text[start..end];
        let first3: String = fraction.chars().chain("000".chars()).take(3).collect();
        millis = first3.parse::<i64>().ok()?;
        at = end;
    }
    let offset_minutes = match bytes.get(at)? {
        b'Z' | b'z' if at + 1 == bytes.len() => 0,
        sign @ (b'+' | b'-') if at + 6 == bytes.len() && bytes[at + 3] == b':' => {
            let minutes = number(at + 1, at + 3)? * 60 + number(at + 4, at + 6)?;
            if *sign == b'-' { -minutes } else { minutes }
        }
        _ => return None,
    };
    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour * 3_600 + minute * 60 + second - offset_minutes * 60;
    Some(seconds * 1_000 + millis)
}

/// Days from 1970-01-01 to a civil date (proleptic Gregorian).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    // no-prop-test: known instants, computed apart from this code.
    #[test]
    fn rfc3339_times_read() {
        assert_eq!(rfc3339_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(rfc3339_ms("2000-03-01T00:00:00Z"), Some(951_868_800_000));
        assert_eq!(
            rfc3339_ms("2026-10-23T12:00:00.250+02:00"),
            Some(1_792_749_600_250)
        );
        assert_eq!(rfc3339_ms("2026-10-23"), None);
        assert_eq!(rfc3339_ms("2026-13-23T12:00:00Z"), None);
        assert_eq!(rfc3339_ms("2026-10-23T12:00:00"), None);
    }

    proptest! {
        /// One more day is one more 86 400 000 ms; an offset of `h` hours
        /// east is `h` hours earlier; no text makes it panic.
        #[test]
        fn prop_rfc3339_steps_are_exact(
            year in 1971i64..2100,
            month in 1i64..=12,
            day in 1i64..=27,
            hour in 0i64..24,
            offset in 0i64..14,
            noise in ".{0,40}",
        ) {
            let at = |day: i64, zone: &str| {
                rfc3339_ms(&format!("{year:04}-{month:02}-{day:02}T{hour:02}:00:00{zone}"))
            };
            let today = at(day, "Z").unwrap();
            prop_assert_eq!(at(day + 1, "Z").unwrap() - today, 86_400_000);
            prop_assert_eq!(
                today - at(day, &format!("+{offset:02}:00")).unwrap(),
                offset * 3_600_000
            );
            let _ = rfc3339_ms(&noise);
        }

        /// A headline color is the same three channels however it is spelled.
        #[test]
        fn prop_hex_colors_read(r in 0u32..256, g in 0u32..256, b in 0u32..256, a in 0u32..256) {
            let rgb = r << 16 | g << 8 | b;
            prop_assert_eq!(hex_rgb(&format!("#{rgb:06x}")), Some(rgb));
            prop_assert_eq!(hex_rgb(&format!("#{rgb:06X}{a:02x}")), Some(rgb));
            let short = (r & 0xF) << 8 | (g & 0xF) << 4 | (b & 0xF);
            let expanded = ((r & 0xF) * 17) << 16 | ((g & 0xF) * 17) << 8 | ((b & 0xF) * 17);
            prop_assert_eq!(hex_rgb(&format!("#{short:03x}")), Some(expanded));
            prop_assert_eq!(hex_rgb(&format!("{rgb:06x}")), None);
        }
    }
}
