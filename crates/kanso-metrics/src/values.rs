//! Everything a batch may carry (docs/METRICS.md › Events): closed lists,
//! two checked patterns, the ID and the UTC day. A value that doesn't fit is
//! `unknown` before it leaves this module.

use std::fmt;

use serde::{Deserialize, Serialize};

/// What a free-form value becomes when it doesn't match its pattern.
pub const UNKNOWN: &str = "unknown";

/// The system Kanso runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Macos,
    Windows,
    Linux,
    Ios,
    Android,
}

impl Platform {
    pub const ALL: [Platform; 5] = [
        Platform::Macos,
        Platform::Windows,
        Platform::Linux,
        Platform::Ios,
        Platform::Android,
    ];

    /// The system this was built for, if it is one of the five.
    pub const fn current() -> Option<Platform> {
        if cfg!(target_os = "macos") {
            Some(Platform::Macos)
        } else if cfg!(target_os = "windows") {
            Some(Platform::Windows)
        } else if cfg!(target_os = "linux") {
            Some(Platform::Linux)
        } else if cfg!(target_os = "ios") {
            Some(Platform::Ios)
        } else if cfg!(target_os = "android") {
            Some(Platform::Android)
        } else {
            None
        }
    }

    pub const fn is_desktop(self) -> bool {
        matches!(self, Platform::Macos | Platform::Windows | Platform::Linux)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Platform::Macos => "macos",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::Ios => "ios",
            Platform::Android => "android",
        }
    }
}

/// The kind of device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    Desktop,
    Phone,
    Tablet,
}

impl Form {
    pub const fn as_str(self) -> &'static str {
        match self {
            Form::Desktop => "desktop",
            Form::Phone => "phone",
            Form::Tablet => "tablet",
        }
    }
}

/// Where the library syncs: nowhere, Kanso's own server, or another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Off,
    Kanso,
    SelfHosted,
}

impl SyncMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            SyncMode::Off => "off",
            SyncMode::Kanso => "kanso",
            SyncMode::SelfHosted => "self_hosted",
        }
    }
}

/// How many notes the library holds, as a range only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotesRange {
    #[serde(rename = "0")]
    None,
    #[serde(rename = "1-9")]
    Ones,
    #[serde(rename = "10-99")]
    Tens,
    #[serde(rename = "100-999")]
    Hundreds,
    #[serde(rename = "1000+")]
    Thousands,
}

impl NotesRange {
    pub const fn from_count(notes: u64) -> NotesRange {
        match notes {
            0 => NotesRange::None,
            1..=9 => NotesRange::Ones,
            10..=99 => NotesRange::Tens,
            100..=999 => NotesRange::Hundreds,
            _ => NotesRange::Thousands,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            NotesRange::None => "0",
            NotesRange::Ones => "1-9",
            NotesRange::Tens => "10-99",
            NotesRange::Hundreds => "100-999",
            NotesRange::Thousands => "1000+",
        }
    }
}

/// Where an import came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    Markdown,
    Obsidian,
    Logseq,
    Notion,
    Evernote,
    Bear,
    Joplin,
    StandardNotes,
    Simplenote,
    Upnote,
    AppleNotes,
    Pouch,
    Other,
}

impl ImportSource {
    pub const ALL: [ImportSource; 13] = [
        ImportSource::Markdown,
        ImportSource::Obsidian,
        ImportSource::Logseq,
        ImportSource::Notion,
        ImportSource::Evernote,
        ImportSource::Bear,
        ImportSource::Joplin,
        ImportSource::StandardNotes,
        ImportSource::Simplenote,
        ImportSource::Upnote,
        ImportSource::AppleNotes,
        ImportSource::Pouch,
        ImportSource::Other,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            ImportSource::Markdown => "markdown",
            ImportSource::Obsidian => "obsidian",
            ImportSource::Logseq => "logseq",
            ImportSource::Notion => "notion",
            ImportSource::Evernote => "evernote",
            ImportSource::Bear => "bear",
            ImportSource::Joplin => "joplin",
            ImportSource::StandardNotes => "standard_notes",
            ImportSource::Simplenote => "simplenote",
            ImportSource::Upnote => "upnote",
            ImportSource::AppleNotes => "apple_notes",
            ImportSource::Pouch => "pouch",
            ImportSource::Other => "other",
        }
    }

    fn parse(name: &str) -> Option<ImportSource> {
        ImportSource::ALL.into_iter().find(|s| s.as_str() == name)
    }
}

/// A feature that was used; an import says where from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feature {
    NoteCreated,
    VoiceInput,
    Publish,
    Import(ImportSource),
    Lock,
    Ai,
    QuickCapture,
    SharedNote,
    Export,
}

impl Feature {
    /// Every feature, with each import source.
    pub fn all() -> impl Iterator<Item = Feature> {
        [
            Feature::NoteCreated,
            Feature::VoiceInput,
            Feature::Publish,
            Feature::Lock,
            Feature::Ai,
            Feature::QuickCapture,
            Feature::SharedNote,
            Feature::Export,
        ]
        .into_iter()
        .chain(ImportSource::ALL.into_iter().map(Feature::Import))
    }

    /// The `feature` prop. One import a day is counted, whatever its source.
    pub const fn name(self) -> &'static str {
        match self {
            Feature::NoteCreated => "note_created",
            Feature::VoiceInput => "voice_input",
            Feature::Publish => "publish",
            Feature::Import(_) => "import",
            Feature::Lock => "lock",
            Feature::Ai => "ai",
            Feature::QuickCapture => "quick_capture",
            Feature::SharedNote => "shared_note",
            Feature::Export => "export",
        }
    }

    /// The `source` prop: only, and always, with an import.
    pub const fn source(self) -> Option<ImportSource> {
        match self {
            Feature::Import(source) => Some(source),
            _ => None,
        }
    }

    /// How the ledger keeps it: the name, and `import:<source>`.
    pub(crate) fn key(self) -> String {
        match self {
            Feature::Import(source) => format!("import:{}", source.as_str()),
            other => other.name().to_owned(),
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<Feature> {
        if let Some(source) = key.strip_prefix("import:") {
            return ImportSource::parse(source).map(Feature::Import);
        }
        Feature::all().find(|f| f.source().is_none() && f.name() == key)
    }
}

/// Who a batch counts: `a_` (an account, from its data key) or `i_` (this
/// install) and 32 lowercase hex digits.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Id(String);

impl Id {
    pub fn parse(id: &str) -> Option<Id> {
        let (prefix, hex) = (id.get(..2)?, id.get(2..)?);
        let hex_ok = hex.len() == 32
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        (matches!(prefix, "a_" | "i_") && hex_ok).then(|| Id(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Id, D::Error> {
        let id = String::deserialize(deserializer)?;
        Id::parse(&id).ok_or_else(|| serde::de::Error::custom("not a metrics ID"))
    }
}

/// A new install ID: `i_` and 16 random bytes as hex.
pub fn install_id() -> Id {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    Id(format!("i_{}", hex(&bytes)))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A UTC day, as days since 1970-01-01, written `YYYY-MM-DD`. Kept within
/// the years 1970 to 9999, so it always has four digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day(i64);

/// 9999-12-31.
const LAST_DAY: i64 = 2_932_896;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

impl Day {
    /// The UTC day of a time in milliseconds since 1970.
    pub fn from_unix_ms(ms: i64) -> Day {
        Day(ms.div_euclid(DAY_MS).clamp(0, LAST_DAY))
    }

    /// Today by this device's clock.
    pub fn today() -> Day {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
        Day::from_unix_ms(ms)
    }

    /// The day after, for tests and callers that step through days.
    pub fn next(self) -> Day {
        Day((self.0 + 1).min(LAST_DAY))
    }

    /// The day before, for tests and callers that step through days.
    pub fn previous(self) -> Day {
        Day((self.0 - 1).max(0))
    }

    /// `YYYY-MM-DD`, an existing date between 1970 and 9999.
    pub fn parse(text: &str) -> Option<Day> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let num = |range: std::ops::Range<usize>| -> Option<i64> {
            let part = text.get(range)?;
            part.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| part.parse().ok())?
        };
        let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
        let day = Day(days_from_civil(y, m, d));
        (day.ymd() == (y, m, d) && (0..=LAST_DAY).contains(&day.0)).then_some(day)
    }

    fn ymd(self) -> (i64, i64, i64) {
        civil_from_days(self.0)
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = self.ymd();
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

impl Serialize for Day {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Day {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Day, D::Error> {
        let text = String::deserialize(deserializer)?;
        Day::parse(&text).ok_or_else(|| serde::de::Error::custom("not a day"))
    }
}

/// Howard Hinnant's `days_from_civil`: the proleptic Gregorian calendar.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        if m <= 2 {
            yoe + era * 400 + 1
        } else {
            yoe + era * 400
        },
        m,
        d,
    )
}

/// `app_version`: three numbers of up to four digits. A pre-release or build
/// suffix (`-beta.1`, `+abc`) is left off; anything else is `unknown`, which
/// the Worker refuses, so a malformed version is never counted as another.
pub fn app_version(version: &str) -> String {
    let core = version.split(['-', '+']).next().unwrap_or_default();
    let parts: Vec<&str> = core.split('.').collect();
    let fits = parts.len() == 3
        && parts
            .iter()
            .all(|p| (1..=4).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit()));
    if fits { core } else { UNKNOWN }.to_owned()
}

/// `os_version`: up to 32 letters, digits, dots, underscores and dashes, else
/// `unknown`.
pub fn os_version(version: &str) -> String {
    let fits = (1..=32).contains(&version.len())
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
    if fits { version } else { UNKNOWN }.to_owned()
}

/// What every event says about the device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    platform: Platform,
    form: Form,
    app_version: String,
}

impl Device {
    /// The version is checked ([`app_version`]); a computer is always a
    /// desktop.
    pub fn new(platform: Platform, form: Form, app_version: &str) -> Device {
        Device {
            platform,
            form: if platform.is_desktop() {
                Form::Desktop
            } else {
                form
            },
            app_version: self::app_version(app_version),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn form(&self) -> Form {
        self.form
    }

    pub fn app_version(&self) -> &str {
        &self.app_version
    }
}

/// What `app_active` adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Active {
    os_version: String,
    sync: SyncMode,
    ai: bool,
    notes: NotesRange,
}

impl Active {
    /// The OS version is checked ([`os_version`]); notes are counted as a
    /// range only.
    pub fn new(os_version: &str, sync: SyncMode, ai: bool, notes: u64) -> Active {
        Active {
            os_version: self::os_version(os_version),
            sync,
            ai,
            notes: NotesRange::from_count(notes),
        }
    }

    pub fn os_version(&self) -> &str {
        &self.os_version
    }

    pub fn sync(&self) -> SyncMode {
        self.sync
    }

    pub fn ai(&self) -> bool {
        self.ai
    }

    pub fn notes(&self) -> NotesRange {
        self.notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert_eq!(Day::from_unix_ms(0).to_string(), "1970-01-01");
        // 2026-10-30T23:59:59.999Z and the next millisecond.
        assert_eq!(
            Day::from_unix_ms(1_793_404_799_999).to_string(),
            "2026-10-30"
        );
        assert_eq!(
            Day::from_unix_ms(1_793_404_800_000).to_string(),
            "2026-10-31"
        );
        assert_eq!(Day::parse("2024-02-29").unwrap().to_string(), "2024-02-29");
        assert_eq!(Day::parse("2026-02-29"), None);
        assert_eq!(Day::parse("2026-1-30"), None);
        assert_eq!(Day::parse("+026-01-30"), None);
        assert_eq!(Day::from_unix_ms(-5), Day::from_unix_ms(0));

        assert_eq!(app_version("0.8.4"), "0.8.4");
        assert_eq!(app_version("1.10.0-beta.2"), "1.10.0");
        assert_eq!(app_version("1.2"), "unknown");
        assert_eq!(app_version("12345.0.0"), "unknown");
        assert_eq!(app_version("1.2.x"), "unknown");
        assert_eq!(os_version("15.1"), "15.1");
        assert_eq!(os_version("10.0.26100"), "10.0.26100");
        assert_eq!(os_version("Ubuntu 24.04"), "unknown");
        assert_eq!(os_version(""), "unknown");
        assert_eq!(os_version(&"1".repeat(33)), "unknown");

        assert_eq!(NotesRange::from_count(0).as_str(), "0");
        assert_eq!(NotesRange::from_count(9).as_str(), "1-9");
        assert_eq!(NotesRange::from_count(10).as_str(), "10-99");
        assert_eq!(NotesRange::from_count(999).as_str(), "100-999");
        assert_eq!(NotesRange::from_count(1000).as_str(), "1000+");

        assert_eq!(
            Device::new(Platform::Linux, Form::Phone, "1.0.0").form(),
            Form::Desktop
        );
        assert_eq!(
            Device::new(Platform::Ios, Form::Tablet, "1.0.0").form(),
            Form::Tablet
        );

        assert!(Id::parse("a_3f9c0d4e5b6a79881726354433221100").is_some());
        assert!(Id::parse("i_3F9C0D4E5B6A79881726354433221100").is_none());
        assert!(Id::parse("d_3f9c0d4e5b6a79881726354433221100").is_none());
        assert!(Id::parse("a_3f9c").is_none());
        assert!(Id::parse("").is_none());
        let id = install_id();
        assert!(Id::parse(id.as_str()).is_some() && id.as_str().starts_with("i_"));
        assert_ne!(install_id(), id);

        assert_eq!(Feature::Import(ImportSource::AppleNotes).name(), "import");
        assert_eq!(
            Feature::from_key("import:standard_notes"),
            Some(Feature::Import(ImportSource::StandardNotes))
        );
        assert_eq!(Feature::from_key("import"), None);
        assert_eq!(Feature::from_key("telemetry"), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Days read back as they were written, keep their order, and every
        /// one is a valid `YYYY-MM-DD`.
        #[test]
        fn prop_days_round_trip(a in 0i64..LAST_DAY + 10, b in 0i64..LAST_DAY + 10) {
            let (da, db) = (Day::from_unix_ms(a * DAY_MS), Day::from_unix_ms(b * DAY_MS));
            let text = da.to_string();
            prop_assert_eq!(text.len(), 10);
            prop_assert_eq!(Day::parse(&text), Some(da));
            prop_assert_eq!(da.cmp(&db), da.to_string().cmp(&db.to_string()));
        }

        /// A version is either its own `x.y.z` core or `unknown`, and the
        /// OS version is either itself or `unknown`: nothing typed else gets
        /// through.
        #[test]
        fn prop_free_values_fit_their_patterns(text in "\\PC{0,40}") {
            let app = app_version(&text);
            prop_assert!(app == UNKNOWN || text.starts_with(&app));
            if app != UNKNOWN {
                let parts: Vec<&str> = app.split('.').collect();
                prop_assert_eq!(parts.len(), 3);
            }
            let os = os_version(&text);
            prop_assert!(os == UNKNOWN || os == text);
        }

        /// Every feature survives the ledger's key.
        #[test]
        fn prop_feature_keys_round_trip(ix in 0usize..21) {
            let feature = Feature::all().nth(ix).unwrap();
            prop_assert_eq!(Feature::from_key(&feature.key()), Some(feature));
        }
    }
}
