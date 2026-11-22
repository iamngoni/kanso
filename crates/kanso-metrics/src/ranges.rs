//! The closed lists and ranges a batch may carry beyond the first ones
//! (docs/METRICS.md › Events). Plain value types: no logic beyond turning a
//! count or a measure into its range. Declaration order is the order of the
//! ranges, so a larger input never lands in an earlier one.

use serde::{Serialize, Serializer};

/// A closed list: the variants, their order, their text and how they are
/// written.
macro_rules! closed_list {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// Every value, in order.
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// The text the Worker checks for.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
    };
}

closed_list! {
    /// How many times a feature was used since the last batch.
    CountRange {
        One => "1",
        TwoToFive => "2-5",
        SixToTwenty => "6-20",
        TwentyOnePlus => "21+",
    }
}

impl CountRange {
    /// Nothing counted is still reported as `1`: an event means it happened.
    pub const fn from_count(count: u64) -> CountRange {
        match count {
            0..=1 => CountRange::One,
            2..=5 => CountRange::TwoToFive,
            6..=20 => CountRange::SixToTwenty,
            _ => CountRange::TwentyOnePlus,
        }
    }
}

closed_list! {
    /// How long the app was in use, from opening to quitting or going to the
    /// background.
    SessionLength {
        UnderMinute => "<1m",
        OneToFiveMinutes => "1-5m",
        FiveToFifteenMinutes => "5-15m",
        FifteenToSixtyMinutes => "15-60m",
        OneToThreeHours => "1-3h",
        ThreeHoursPlus => "3h+",
    }
}

impl SessionLength {
    pub const fn from_secs(secs: u64) -> SessionLength {
        match secs {
            0..=59 => SessionLength::UnderMinute,
            60..=299 => SessionLength::OneToFiveMinutes,
            300..=899 => SessionLength::FiveToFifteenMinutes,
            900..=3_599 => SessionLength::FifteenToSixtyMinutes,
            3_600..=10_799 => SessionLength::OneToThreeHours,
            _ => SessionLength::ThreeHoursPlus,
        }
    }
}

closed_list! {
    /// Installed plugins.
    PluginsRange {
        None => "0",
        OneToTwo => "1-2",
        ThreeToFive => "3-5",
        SixPlus => "6+",
    }
}

impl PluginsRange {
    pub const fn from_count(count: u64) -> PluginsRange {
        match count {
            0 => PluginsRange::None,
            1..=2 => PluginsRange::OneToTwo,
            3..=5 => PluginsRange::ThreeToFive,
            _ => PluginsRange::SixPlus,
        }
    }
}

closed_list! {
    /// Notebooks in the library.
    NotebooksRange {
        None => "0",
        OneToFour => "1-4",
        FiveToNineteen => "5-19",
        TwentyPlus => "20+",
    }
}

impl NotebooksRange {
    pub const fn from_count(count: u64) -> NotebooksRange {
        match count {
            0 => NotebooksRange::None,
            1..=4 => NotebooksRange::OneToFour,
            5..=19 => NotebooksRange::FiveToNineteen,
            _ => NotebooksRange::TwentyPlus,
        }
    }
}

closed_list! {
    /// What the attachments take, in megabytes (1 MB is 1,000,000 bytes). A
    /// range includes its lower end and not its upper one.
    AttachmentsMb {
        None => "0",
        UnderTen => "<10",
        TenToHundred => "10-100",
        HundredToThousand => "100-1000",
        ThousandPlus => "1000+",
    }
}

impl AttachmentsMb {
    pub const fn from_bytes(bytes: u64) -> AttachmentsMb {
        if bytes == 0 {
            return AttachmentsMb::None;
        }
        match bytes / 1_000_000 {
            0..=9 => AttachmentsMb::UnderTen,
            10..=99 => AttachmentsMb::TenToHundred,
            100..=999 => AttachmentsMb::HundredToThousand,
            _ => AttachmentsMb::ThousandPlus,
        }
    }
}

closed_list! {
    /// Sync errors since the last batch.
    SyncErrors {
        None => "0",
        OneToFive => "1-5",
        SixPlus => "6+",
    }
}

impl SyncErrors {
    pub const fn from_count(count: u64) -> SyncErrors {
        match count {
            0 => SyncErrors::None,
            1..=5 => SyncErrors::OneToFive,
            _ => SyncErrors::SixPlus,
        }
    }
}

closed_list! {
    /// Whether the interface is light, dark, or follows the system.
    ThemeMode {
        Light => "light",
        Dark => "dark",
        System => "system",
    }
}

closed_list! {
    /// The default editor mode.
    EditorMode {
        Edit => "edit",
        Live => "live",
        Preview => "preview",
        Split => "split",
    }
}

closed_list! {
    /// Which release line the app was built from.
    Channel {
        Stable => "stable",
        Beta => "beta",
        Dev => "dev",
        Nightly => "nightly",
    }
}

/// The value standing for anything not on a list.
pub const OTHER: &str = "other";
/// The `theme` of any theme that isn't one of Kanso's own.
pub const CUSTOM: &str = "custom";

/// `locale_lang`: an ISO 639-1 code of a language the app ships, else
/// `other`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocaleLang(String);

impl LocaleLang {
    /// `code` (the language part of a locale, any case) when it is one of
    /// `shipped`, else `other`.
    pub fn new(code: &str, shipped: &[&str]) -> LocaleLang {
        let lang = code
            .split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let known = lang.len() == 2
            && lang.bytes().all(|b| b.is_ascii_lowercase())
            && shipped.iter().any(|s| s.eq_ignore_ascii_case(&lang));
        LocaleLang(if known { lang } else { OTHER.to_owned() })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `theme`: the ID of one of Kanso's own themes (lowercase letters, digits
/// and dashes, up to 32) or `custom`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Theme(String);

impl Theme {
    /// The ID of a theme `name` when `builtin`, else `custom`: the name of a
    /// theme someone made or downloaded is never kept. A built-in theme's
    /// name becomes its ID by lowercasing and joining words with dashes.
    pub fn new(name: &str, builtin: bool) -> Theme {
        if !builtin {
            return Theme(CUSTOM.to_owned());
        }
        let mut id = String::new();
        for c in name.chars() {
            if c.is_ascii_alphanumeric() {
                id.push(c.to_ascii_lowercase());
            } else if !id.is_empty() && !id.ends_with('-') {
                id.push('-');
            }
        }
        let id = id.trim_end_matches('-');
        if (1..=32).contains(&id.len()) {
            Theme(id.to_owned())
        } else {
            Theme(CUSTOM.to_owned())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! text_serialize {
    ($($name:ident),+) => {$(
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
    )+};
}
text_serialize!(LocaleLang, Theme);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn edges() {
        assert_eq!(CountRange::from_count(0).as_str(), "1");
        assert_eq!(CountRange::from_count(5).as_str(), "2-5");
        assert_eq!(CountRange::from_count(6).as_str(), "6-20");
        assert_eq!(CountRange::from_count(21).as_str(), "21+");
        assert_eq!(SessionLength::from_secs(59).as_str(), "<1m");
        assert_eq!(SessionLength::from_secs(60).as_str(), "1-5m");
        assert_eq!(SessionLength::from_secs(10_800).as_str(), "3h+");
        assert_eq!(AttachmentsMb::from_bytes(0).as_str(), "0");
        assert_eq!(AttachmentsMb::from_bytes(1).as_str(), "<10");
        assert_eq!(AttachmentsMb::from_bytes(10_000_000).as_str(), "10-100");
        assert_eq!(AttachmentsMb::from_bytes(1_000_000_000).as_str(), "1000+");
        assert_eq!(LocaleLang::new("pt-BR", &["en", "pt"]).as_str(), "pt");
        assert_eq!(LocaleLang::new("zz", &["en"]).as_str(), "other");
        assert_eq!(Theme::new("Kanso Dark", true).as_str(), "kanso-dark");
        assert_eq!(Theme::new("My secret theme", false).as_str(), "custom");
    }

    /// A monotonic bucketing: a larger input never lands in an earlier range.
    fn monotonic<T: Ord>(f: impl Fn(u64) -> T, a: u64, b: u64) -> bool {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        f(lo) <= f(hi)
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        #[test]
        fn prop_ranges_are_monotonic(a in any::<u64>(), b in any::<u64>()) {
            prop_assert!(monotonic(CountRange::from_count, a, b));
            prop_assert!(monotonic(SessionLength::from_secs, a, b));
            prop_assert!(monotonic(PluginsRange::from_count, a, b));
            prop_assert!(monotonic(NotebooksRange::from_count, a, b));
            prop_assert!(monotonic(AttachmentsMb::from_bytes, a, b));
            prop_assert!(monotonic(SyncErrors::from_count, a, b));
        }

        /// A theme is a short slug or `custom`, a language two letters or
        /// `other`, whatever was typed.
        #[test]
        fn prop_text_values_fit(text in "\\PC{0,60}", builtin in any::<bool>()) {
            let theme = Theme::new(&text, builtin);
            let t = theme.as_str();
            prop_assert!(t == CUSTOM || ((1..=32).contains(&t.len())
                && t.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !t.starts_with('-') && !t.ends_with('-')));
            let lang = LocaleLang::new(&text, &["en", "fr"]);
            prop_assert!(["en", "fr", OTHER].contains(&lang.as_str()));
        }
    }
}
