//! Google Fonts for the desktop app: which family names are acceptable, how
//! the font file is found in Google's stylesheet, how it is fetched, and how
//! it is kept in the data folder so it works offline.
//!
//! This module owns the rules and the files, not the registering of a font
//! with the window system (the app does that with the bytes it is handed) and
//! not any drawing. Nothing is fetched except `fonts.googleapis.com/css` and
//! the one `*.gstatic.com` file that stylesheet names, over HTTPS.
//!
//! The phones do the same in Dart (`google_font_source.dart`); both ask for
//! the regular weight only and both send no browser user agent, because for
//! an unknown client Google Fonts answers with a plain TrueType file instead
//! of the WOFF2 its browser answers use, which the window system cannot load.
//!
//! Values are parsed once at the edge: a [`Family`] is a name that is safe in
//! a URL and a file name, a [`FontUrl`] is an HTTPS address on Google's font
//! host, and a [`FontFile`] is bytes that start like a TrueType or OpenType
//! font. The cache and the fetch only accept those types.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

/// The most characters a family name may have, as on the phones.
const MAX_FAMILY_CHARS: usize = 80;
/// The most a stylesheet may be before it is refused: a family's is a few
/// hundred bytes.
pub const MAX_STYLESHEET_BYTES: u64 = 256 * 1024;
/// The most a font file may be before it is refused. The largest CJK
/// families are around 20 MB.
pub const MAX_FONT_BYTES: u64 = 64 * 1024 * 1024;
/// Where Google's stylesheets are.
pub const STYLESHEET_URL: &str = "https://fonts.googleapis.com/css";
/// The folder under the data folder the fonts are kept in.
const FOLDER: &str = "fonts";
/// The extension of a kept font.
const EXTENSION: &str = "ttf";

// MARK: Family

/// Why a name is not a Google Fonts family name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FamilyError {
    #[error("Type the name of a Google Font.")]
    Empty,
    #[error("That name is too long for a font family.")]
    TooLong,
    #[error("Font names use letters, numbers, spaces, hyphens and underscores.")]
    BadCharacter,
}

/// A Google Fonts family name, such as "Open Sans": 1 to 80 letters, digits,
/// hyphens, underscores and single spaces, with no space at either end. Such
/// a name is safe in a query string and in a file name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Family(String);

impl Family {
    /// Reads a family name as someone typed it: the ends are trimmed and a
    /// run of spaces becomes one.
    pub fn parse(raw: &str) -> Result<Family, FamilyError> {
        let mut name = String::with_capacity(raw.len());
        for c in raw.trim().chars() {
            match c {
                ' ' if name.ends_with(' ') => {}
                ' ' | '-' | '_' => name.push(c),
                c if c.is_ascii_alphanumeric() => name.push(c),
                _ => return Err(FamilyError::BadCharacter),
            }
        }
        match name.chars().count() {
            0 => Err(FamilyError::Empty),
            count if count > MAX_FAMILY_CHARS => Err(FamilyError::TooLong),
            _ => Ok(Family(name)),
        }
    }

    /// The name, as the window system knows the family.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `other` is this family with letters in another case: Google
    /// takes "open sans" and calls it "Open Sans".
    pub fn same_as(&self, other: &Family) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// This name with the first letter of each word in capitals ("open sans"
    /// is "Open Sans"), or `None` when it already is. Google Fonts' stylesheet
    /// answers only to a family's own capitals, so this is the second try
    /// for a name typed in lower case.
    pub fn capitalized(&self) -> Option<Family> {
        let mut at_word_start = true;
        let name: String = self
            .0
            .chars()
            .map(|c| {
                let c = if at_word_start {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                at_word_start = c == ' ';
                c
            })
            .collect();
        (name != self.0).then_some(Family(name))
    }

    /// The name in a query string: spaces as `+`.
    fn query_name(&self) -> String {
        self.0.replace(' ', "+")
    }

    /// The file this family is kept in. `+` stands for a space; it can't
    /// be in a family name, so the name is read back from the file's.
    fn file_name(&self) -> String {
        format!("{}.{EXTENSION}", self.query_name())
    }

    /// The family a kept file's name stands for, when the name is one
    /// [`Self::file_name`] could have made.
    fn from_file_name(name: &OsStr) -> Option<Family> {
        let path = Path::new(name);
        if path.extension() != Some(OsStr::new(EXTENSION)) {
            return None;
        }
        let family = Family::parse(&path.file_stem()?.to_str()?.replace('+', " ")).ok()?;
        (family.file_name().as_str() == name.to_str()?).then_some(family)
    }
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The address of `family`'s stylesheet, for the regular weight alone.
pub fn stylesheet_url(family: &Family) -> String {
    format!(
        "{STYLESHEET_URL}?family={}:wght@400&display=swap",
        family.query_name()
    )
}

// MARK: Stylesheet

/// An HTTPS address on Google's font host (`gstatic.com`): the only place a
/// font is downloaded from, however the stylesheet is worded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontUrl(String);

impl FontUrl {
    /// Reads an address, refusing anything but `https://` on `gstatic.com`
    /// or a subdomain of it, with no user, port or odd characters.
    pub fn parse(raw: &str) -> Option<FontUrl> {
        let rest = raw.strip_prefix("https://")?;
        let (host, path) = rest.split_once('/')?;
        let host_ok = host
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-');
        let on_google = host == "gstatic.com" || host.ends_with(".gstatic.com");
        let path_ok = path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~/%+=&?".contains(&b));
        (host_ok && on_google && path_ok).then(|| FontUrl(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The font a stylesheet describes: Google's own spelling of the family,
/// and where its file is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub family: Family,
    pub url: FontUrl,
}

/// Why a stylesheet gave no font.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Google Fonts sent no font file for that family.")]
pub struct NoFace;

/// The first `@font-face` of `css` that names a family and an HTTPS font
/// file on Google's host. The family is the stylesheet's, which is how the
/// font file itself calls it (a typed "open sans" is "Open Sans" there).
pub fn parse_stylesheet(css: &str) -> Result<Face, NoFace> {
    css.split("@font-face")
        .skip(1)
        .filter_map(|block| {
            // The rule's body: between its braces.
            let (_, rest) = block.split_once('{')?;
            let body = rest.split_once('}').map_or(rest, |(body, _)| body);
            let family = unquote(declaration(body, "font-family")?);
            let url = first_url(declaration(body, "src")?)?;
            Some(Face {
                family: Family::parse(family).ok()?,
                url: FontUrl::parse(url)?,
            })
        })
        .next()
        .ok_or(NoFace)
}

/// The value of the declaration `name` in a rule's body.
fn declaration<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    body.split(';').find_map(|part| {
        let (key, value) = part.split_once(':')?;
        key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

/// `value` without one pair of quotes around it.
fn unquote(value: &str) -> &str {
    let value = value.trim();
    ['\'', '"']
        .into_iter()
        .find_map(|q| value.strip_prefix(q)?.strip_suffix(q))
        .unwrap_or(value)
        .trim()
}

/// The address in the first `url(…)` of a `src` value, unquoted.
fn first_url(src: &str) -> Option<&str> {
    let start = src.to_ascii_lowercase().find("url(")? + "url(".len();
    let (inside, _) = src[start..].split_once(')')?;
    Some(unquote(inside))
}

// MARK: Font files

/// Bytes that start like a font the window system can load: TrueType,
/// OpenType (CFF), or a collection. Google also serves WOFF, WOFF2 and EOT
/// to browsers; those are not accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFile(Vec<u8>);

/// What a font file starts with.
const SIGNATURES: [&[u8; 4]; 5] = [b"\x00\x01\x00\x00", b"OTTO", b"true", b"typ1", b"ttcf"];

/// The smallest file that could hold an sfnt's header and a table.
const MIN_FONT_BYTES: usize = 12;

/// The bytes weren't a TrueType or OpenType font.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Google Fonts sent a file Kanso can't use.")]
pub struct NotAFont;

impl FontFile {
    pub fn parse(bytes: Vec<u8>) -> Result<FontFile, NotAFont> {
        let known = bytes.len() >= MIN_FONT_BYTES
            && SIGNATURES
                .iter()
                .any(|signature| bytes.starts_with(signature.as_slice()));
        if known {
            Ok(FontFile(bytes))
        } else {
            Err(NotAFont)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

// MARK: Fetching

/// A step of a download, for the words under the Download button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Asking Google Fonts which file the family is.
    Stylesheet,
    /// Receiving the file: the bytes so far, and the total when the server
    /// said how long it is.
    Font { received: u64, total: Option<u64> },
}

impl Stage {
    /// "Downloading Lora… 120 KB of 340 KB".
    pub fn label(&self, family: &Family) -> String {
        match self {
            Stage::Stylesheet => format!("Looking up {family}…"),
            Stage::Font {
                received,
                total: Some(total),
            } if *total >= *received && *total > 0 => format!(
                "Downloading {family}… {} of {}",
                crate::account::words::format_bytes(*received),
                crate::account::words::format_bytes(*total)
            ),
            Stage::Font { received, .. } => format!(
                "Downloading {family}… {}",
                crate::account::words::format_bytes(*received)
            ),
        }
    }
}

/// Why Google Fonts couldn't be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FetchError {
    #[error("Couldn't reach Google Fonts. Fonts you have already downloaded still work.")]
    Offline,
    #[error("Google Fonts has no family with that name.")]
    UnknownFamily,
    #[error("Google Fonts answered {0}.")]
    Status(u16),
    #[error("That font file is too large to download.")]
    TooLarge,
    #[error("Google Fonts' answer couldn't be read.")]
    Unreadable,
}

/// Where the stylesheet and the font file come from: Google Fonts over
/// HTTPS in the app ([`HttpSource`]), memory in tests.
pub trait Source: Send + Sync {
    /// The stylesheet of the regular weight of `family`.
    fn stylesheet(&self, family: &Family) -> Result<String, FetchError>;

    /// The file at `url`, reporting the bytes as they arrive.
    fn font(&self, url: &FontUrl, progress: &mut dyn FnMut(Stage)) -> Result<Vec<u8>, FetchError>;
}

/// Google Fonts over HTTPS: only HTTPS (every hop of a redirect too), no
/// cookies or credentials, and a plain `Kanso` user agent. Not a browser's:
/// that is what makes Google Fonts answer with TrueType (see the module).
/// It trusts what Kanso trusts everywhere else (`kanso_tls`).
#[cfg(feature = "google-fonts")]
pub struct HttpSource {
    agent: ureq::Agent,
}

#[cfg(feature = "google-fonts")]
impl Default for HttpSource {
    fn default() -> Self {
        use std::time::Duration;

        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(3)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(15)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .user_agent("Kanso")
            .tls_config(kanso_tls::ureq_tls())
            .build()
            .into();
        HttpSource { agent }
    }
}

#[cfg(feature = "google-fonts")]
impl HttpSource {
    // no-prop-test: asks the network, which has no oracle offline; the
    // answers it maps are read by the pure functions above.
    fn get(&self, url: &str) -> Result<ureq::http::Response<ureq::Body>, FetchError> {
        // Before the handshake: an Android app that hasn't handed over its
        // Context gets an error, not a panic inside the verifier.
        kanso_tls::ready_for(url).map_err(|_| FetchError::Offline)?;
        let response = self
            .agent
            .get(url)
            .call()
            .map_err(|_| FetchError::Offline)?;
        match response.status().as_u16() {
            200 => Ok(response),
            // Google Fonts answers 400 for a family it doesn't have.
            400 | 404 => Err(FetchError::UnknownFamily),
            code => Err(FetchError::Status(code)),
        }
    }
}

#[cfg(feature = "google-fonts")]
impl Source for HttpSource {
    // no-prop-test: asks the network; see `get`.
    fn stylesheet(&self, family: &Family) -> Result<String, FetchError> {
        let mut response = self.get(&stylesheet_url(family))?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_STYLESHEET_BYTES)
            .read_to_string()
            .map_err(|_| FetchError::Unreadable)
    }

    // no-prop-test: asks the network; see `get`.
    fn font(&self, url: &FontUrl, progress: &mut dyn FnMut(Stage)) -> Result<Vec<u8>, FetchError> {
        use std::io::Read as _;

        let response = self.get(url.as_str())?;
        let total = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok()?.parse::<u64>().ok());
        if total.is_some_and(|total| total > MAX_FONT_BYTES) {
            return Err(FetchError::TooLarge);
        }
        let mut reader = response.into_body().into_reader().take(MAX_FONT_BYTES + 1);
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 16 * 1024];
        loop {
            let read = reader.read(&mut chunk).map_err(|_| FetchError::Offline)?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..read]);
            progress(Stage::Font {
                received: bytes.len() as u64,
                total,
            });
        }
        if bytes.len() as u64 > MAX_FONT_BYTES {
            return Err(FetchError::TooLarge);
        }
        Ok(bytes)
    }
}

// MARK: Cache

/// A font kept in the data folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cached {
    pub family: Family,
    pub file: FontFile,
}

/// What reading the whole cache found: the fonts, and a line for each file
/// that couldn't be read.
#[derive(Debug, Default)]
pub struct Scan {
    pub found: Vec<Cached>,
    pub failed: Vec<String>,
}

/// The downloaded fonts, kept as `fonts/<Family+Name>.ttf` in the data
/// folder. A file that isn't a font is deleted when it is found, so one bad
/// file doesn't fail every launch the same way.
#[derive(Clone, Debug)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    /// The cache in the data folder `data_dir`.
    pub fn new(data_dir: &Path) -> Cache {
        Cache {
            dir: data_dir.join(FOLDER),
        }
    }

    /// The kept font for `family` (in any case), if there is a good one.
    pub fn find(&self, family: &Family) -> io::Result<Option<Cached>> {
        for (name, file_name) in self.entries()? {
            if name.same_as(family)
                && let Some(cached) = self.read(&name, &file_name)?
            {
                return Ok(Some(cached));
            }
        }
        Ok(None)
    }

    /// Every good kept font, by family. The rest of the folder is left
    /// alone.
    pub fn scan(&self) -> Scan {
        let mut scan = Scan::default();
        let entries = match self.entries() {
            Ok(entries) => entries,
            Err(err) => {
                scan.failed.push(format!("{}: {err}", self.dir.display()));
                return scan;
            }
        };
        for (family, file_name) in entries {
            match self.read(&family, &file_name) {
                Ok(Some(cached)) => scan.found.push(cached),
                Ok(None) => {}
                Err(err) => scan.failed.push(format!("{file_name:?}: {err}")),
            }
        }
        scan.found.sort_by(|a, b| a.family.cmp(&b.family));
        scan
    }

    /// Keeps `file` as `family`'s font. It is written beside its place and
    /// moved there, so a quit part way leaves no half font.
    pub fn store(&self, family: &Family, file: &FontFile) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let to = self.dir.join(family.file_name());
        let part = to.with_extension("tmp");
        let mut out = fs::File::create(&part)?;
        out.write_all(file.as_bytes())?;
        out.sync_all()?;
        fs::rename(&part, &to).inspect_err(|_| {
            fs::remove_file(&part).ok();
        })
    }

    /// Deletes `family`'s kept font, if it has one.
    pub fn remove(&self, family: &Family) -> io::Result<()> {
        match fs::remove_file(self.dir.join(family.file_name())) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        }
    }

    /// The files whose names are a family's. A folder that isn't there yet
    /// has none.
    fn entries(&self) -> io::Result<Vec<(Family, std::ffi::OsString)>> {
        let read = match fs::read_dir(&self.dir) {
            Ok(read) => read,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let mut entries = Vec::new();
        for entry in read {
            let file_name = entry?.file_name();
            if let Some(family) = Family::from_file_name(&file_name) {
                entries.push((family, file_name));
            }
        }
        Ok(entries)
    }

    /// The font in the file `file_name`, or `None` (after deleting it) when
    /// it isn't one.
    fn read(&self, family: &Family, file_name: &OsStr) -> io::Result<Option<Cached>> {
        let path = self.dir.join(file_name);
        match FontFile::parse(fs::read(&path)?) {
            Ok(file) => Ok(Some(Cached {
                family: family.clone(),
                file,
            })),
            Err(NotAFont) => {
                fs::remove_file(&path)?;
                Ok(None)
            }
        }
    }
}

// MARK: Loading

/// Where a loaded font came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Cache,
    Download,
}

/// A family ready to register with the window system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loaded {
    /// The family under Google's spelling, as the font file calls it.
    pub family: Family,
    pub file: FontFile,
    pub origin: Origin,
}

/// Why a family couldn't be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    NoFace(#[from] NoFace),
    #[error(transparent)]
    NotAFont(#[from] NotAFont),
    #[error("Couldn't keep the font on this computer: {0}")]
    Cache(#[from] io::Error),
}

impl LoadError {
    /// Whether Google Fonts couldn't be reached at all, which the words
    /// answer with the fonts that still work.
    pub fn is_offline(&self) -> bool {
        matches!(self, LoadError::Fetch(FetchError::Offline))
    }
}

/// The stylesheet of `family` as typed, or when Google Fonts doesn't know it
/// by that name, as [`Family::capitalized`].
fn stylesheet_of(source: &dyn Source, family: &Family) -> Result<String, FetchError> {
    match (source.stylesheet(family), family.capitalized()) {
        (Err(FetchError::UnknownFamily), Some(capitalized)) => source.stylesheet(&capitalized),
        (answer, _) => answer,
    }
}

/// `family` from the cache, or else downloaded and kept: what the Download
/// button does. A cached font needs no network, so it loads offline.
pub fn load(
    source: &dyn Source,
    cache: &Cache,
    family: &Family,
    progress: &mut dyn FnMut(Stage),
) -> Result<Loaded, LoadError> {
    if let Some(Cached { family, file }) = cache.find(family)? {
        return Ok(Loaded {
            family,
            file,
            origin: Origin::Cache,
        });
    }
    progress(Stage::Stylesheet);
    let face = parse_stylesheet(&stylesheet_of(source, family)?)?;
    let file = FontFile::parse(source.font(&face.url, progress)?)?;
    cache.store(&face.family, &file)?;
    Ok(Loaded {
        family: face.family,
        file,
        origin: Origin::Download,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// A family name, in the shapes Google has them: words, digits, a hyphen.
    fn family_text() -> impl Strategy<Value = String> {
        "[A-Za-z0-9][A-Za-z0-9_-]{0,9}( [A-Za-z0-9][A-Za-z0-9_-]{0,9}){0,3}"
    }

    fn family(text: &str) -> Family {
        Family::parse(text).expect("a family in the strategy's grammar")
    }

    /// Bytes that start like a TrueType font.
    fn font_bytes(tail: &[u8]) -> Vec<u8> {
        let mut bytes = b"\x00\x01\x00\x00".to_vec();
        bytes.extend_from_slice(&[0; 8]);
        bytes.extend_from_slice(tail);
        bytes
    }

    fn css(family: &str, url: &str) -> String {
        format!(
            "@font-face {{\n  font-family: '{family}';\n  font-style: normal;\n  font-weight: 400;\n  src: url({url}) format('truetype');\n}}\n"
        )
    }

    const LORA_URL: &str =
        "https://fonts.gstatic.com/s/lora/v37/0QI6MX1D_JOuGQbT0gvTJPa787weuxJBkqg.ttf";

    /// A source answering from memory, counting its questions.
    struct Memory {
        css: Result<String, FetchError>,
        font: Result<Vec<u8>, FetchError>,
        asked: Mutex<(u32, u32)>,
    }

    impl Memory {
        fn answering(css: String, font: Vec<u8>) -> Memory {
            Memory {
                css: Ok(css),
                font: Ok(font),
                asked: Mutex::new((0, 0)),
            }
        }

        fn offline() -> Memory {
            Memory {
                css: Err(FetchError::Offline),
                font: Err(FetchError::Offline),
                asked: Mutex::new((0, 0)),
            }
        }

        fn asked(&self) -> (u32, u32) {
            *self.asked.lock().expect("the counter isn't poisoned")
        }
    }

    impl Source for Memory {
        fn stylesheet(&self, _: &Family) -> Result<String, FetchError> {
            self.asked.lock().expect("the counter isn't poisoned").0 += 1;
            self.css.clone()
        }

        fn font(
            &self,
            _: &FontUrl,
            progress: &mut dyn FnMut(Stage),
        ) -> Result<Vec<u8>, FetchError> {
            self.asked.lock().expect("the counter isn't poisoned").1 += 1;
            if let Ok(bytes) = &self.font {
                progress(Stage::Font {
                    received: bytes.len() as u64,
                    total: Some(bytes.len() as u64),
                });
            }
            self.font.clone()
        }
    }

    #[test]
    fn a_family_is_read_as_typed() {
        assert_eq!(family("  Open   Sans ").as_str(), "Open Sans");
        assert_eq!(Family::parse(""), Err(FamilyError::Empty));
        assert_eq!(Family::parse("   "), Err(FamilyError::Empty));
        assert_eq!(Family::parse("Open/Sans"), Err(FamilyError::BadCharacter));
        assert_eq!(Family::parse("Open\tSans"), Err(FamilyError::BadCharacter));
        assert_eq!(Family::parse("Üppig"), Err(FamilyError::BadCharacter));
        assert_eq!(Family::parse(&"a".repeat(80)).map(|f| f.0.len()), Ok(80));
        assert_eq!(Family::parse(&"a".repeat(81)), Err(FamilyError::TooLong));
    }

    #[test]
    fn a_lower_case_name_is_tried_with_capitals() {
        assert_eq!(
            family("open sans").capitalized().map(|f| f.0),
            Some("Open Sans".to_owned())
        );
        assert_eq!(family("Open Sans").capitalized(), None);
        assert_eq!(
            family("ibm plex-mono").capitalized().map(|f| f.0),
            Some("Ibm Plex-mono".to_owned())
        );

        // Google Fonts only knows "Open Sans": the second try finds it.
        struct Capitals(Mutex<Vec<String>>);
        impl Source for Capitals {
            fn stylesheet(&self, family: &Family) -> Result<String, FetchError> {
                self.0
                    .lock()
                    .expect("not poisoned")
                    .push(family.to_string());
                if family.as_str() == "Open Sans" {
                    Ok(css("Open Sans", LORA_URL))
                } else {
                    Err(FetchError::UnknownFamily)
                }
            }
            fn font(&self, _: &FontUrl, _: &mut dyn FnMut(Stage)) -> Result<Vec<u8>, FetchError> {
                Ok(font_bytes(b"open"))
            }
        }
        let dir = tempfile::tempdir().expect("a folder");
        let source = Capitals(Mutex::new(Vec::new()));
        let loaded = load(
            &source,
            &Cache::new(dir.path()),
            &family("open sans"),
            &mut |_| {},
        )
        .expect("the capitalized name");
        assert_eq!(loaded.family.as_str(), "Open Sans");
        assert_eq!(
            *source.0.lock().expect("not poisoned"),
            ["open sans", "Open Sans"]
        );

        // A name that is right as typed is asked once; one nobody has, twice.
        let source = Capitals(Mutex::new(Vec::new()));
        load(
            &source,
            &Cache::new(dir.path()),
            &family("nope"),
            &mut |_| {},
        )
        .expect_err("unknown");
        assert_eq!(*source.0.lock().expect("not poisoned"), ["nope", "Nope"]);
    }

    #[test]
    fn the_stylesheet_asks_for_the_regular_weight() {
        assert_eq!(
            stylesheet_url(&family("Open Sans")),
            "https://fonts.googleapis.com/css?family=Open+Sans:wght@400&display=swap"
        );
    }

    #[test]
    fn googles_stylesheet_gives_the_family_and_the_file() {
        let face = parse_stylesheet(&css("Lora", LORA_URL)).expect("a face");
        assert_eq!(face.family.as_str(), "Lora");
        assert_eq!(face.url.as_str(), LORA_URL);
        assert_eq!(parse_stylesheet("/* nothing */"), Err(NoFace));
        assert_eq!(parse_stylesheet(""), Err(NoFace));
    }

    #[test]
    fn a_stylesheet_off_googles_host_gives_nothing() {
        for url in [
            "https://evil.example/a.ttf",
            "https://evilgstatic.com/a.ttf",
            "https://fonts.gstatic.com@evil.example/a.ttf",
            "https://fonts.gstatic.com:8443/a.ttf",
            "http://fonts.gstatic.com/a.ttf",
            "https://fonts.gstatic.com",
            "data:font/ttf;base64,AAAA",
        ] {
            assert_eq!(parse_stylesheet(&css("Lora", url)), Err(NoFace), "{url}");
        }
    }

    #[test]
    fn the_first_usable_face_wins() {
        let css = format!("{}{}", css("Bad Family!", LORA_URL), css("Lora", LORA_URL));
        assert_eq!(
            parse_stylesheet(&css).expect("a face").family.as_str(),
            "Lora"
        );
    }

    #[test]
    fn only_truetype_and_opentype_are_fonts() {
        assert!(FontFile::parse(font_bytes(b"")).is_ok());
        assert!(FontFile::parse(b"OTTO\0\0\0\0\0\0\0\0".to_vec()).is_ok());
        assert!(FontFile::parse(b"ttcf\0\0\0\0\0\0\0\0".to_vec()).is_ok());
        assert_eq!(
            FontFile::parse(b"wOF2\0\0\0\0\0\0\0\0".to_vec()),
            Err(NotAFont)
        );
        assert_eq!(
            FontFile::parse(b"wOFF\0\0\0\0\0\0\0\0".to_vec()),
            Err(NotAFont)
        );
        assert_eq!(FontFile::parse(b"OTTO".to_vec()), Err(NotAFont));
        assert_eq!(FontFile::parse(Vec::new()), Err(NotAFont));
    }

    #[test]
    fn the_stages_are_said_in_words() {
        let lora = family("Lora");
        assert_eq!(Stage::Stylesheet.label(&lora), "Looking up Lora…");
        assert_eq!(
            Stage::Font {
                received: 2048,
                total: Some(4096)
            }
            .label(&lora),
            "Downloading Lora… 2 KB of 4 KB"
        );
        assert_eq!(
            Stage::Font {
                received: 2048,
                total: None
            }
            .label(&lora),
            "Downloading Lora… 2 KB"
        );
        // A total smaller than what arrived (a server that lied) is left out.
        assert_eq!(
            Stage::Font {
                received: 2048,
                total: Some(10)
            }
            .label(&lora),
            "Downloading Lora… 2 KB"
        );
    }

    #[test]
    fn a_download_is_kept_and_then_loads_offline() {
        let dir = tempfile::tempdir().expect("a folder");
        let cache = Cache::new(dir.path());
        let source = Memory::answering(css("Lora", LORA_URL), font_bytes(b"lora"));
        let mut stages = Vec::new();
        let loaded =
            load(&source, &cache, &family("lora"), &mut |s| stages.push(s)).expect("a download");
        assert_eq!(loaded.family.as_str(), "Lora");
        assert_eq!(loaded.origin, Origin::Download);
        assert_eq!(stages.first(), Some(&Stage::Stylesheet));
        assert!(matches!(stages.last(), Some(Stage::Font { .. })));
        assert!(dir.path().join("fonts/Lora.ttf").is_file());
        assert!(!dir.path().join("fonts/Lora.tmp").exists());

        // Offline, in another case: the cache answers.
        let offline = Memory::offline();
        let again = load(&offline, &cache, &family("LORA"), &mut |_| {}).expect("the cache");
        assert_eq!(again.origin, Origin::Cache);
        assert_eq!(again.family.as_str(), "Lora");
        assert_eq!(again.file, loaded.file);
        assert_eq!(offline.asked(), (0, 0));
    }

    #[test]
    fn offline_without_a_cache_says_so() {
        let dir = tempfile::tempdir().expect("a folder");
        let error = load(
            &Memory::offline(),
            &Cache::new(dir.path()),
            &family("Lora"),
            &mut |_| {},
        )
        .expect_err("no network, no cache");
        assert!(error.is_offline());
        assert!(error.to_string().contains("already downloaded"));
        assert!(!dir.path().join("fonts").exists());
    }

    #[test]
    fn a_web_format_is_not_kept() {
        let dir = tempfile::tempdir().expect("a folder");
        let cache = Cache::new(dir.path());
        let source = Memory::answering(css("Lora", LORA_URL), b"wOF2\0\0\0\0\0\0\0\0".to_vec());
        let error = load(&source, &cache, &family("Lora"), &mut |_| {}).expect_err("woff2");
        assert!(matches!(error, LoadError::NotAFont(NotAFont)));
        assert!(cache.scan().found.is_empty());
    }

    #[test]
    fn a_stylesheet_without_a_font_is_an_error() {
        let dir = tempfile::tempdir().expect("a folder");
        let source = Memory::answering("body {}".to_owned(), font_bytes(b""));
        let error = load(
            &source,
            &Cache::new(dir.path()),
            &family("Lora"),
            &mut |_| {},
        )
        .expect_err("no face");
        assert!(matches!(error, LoadError::NoFace(NoFace)));
        assert!(!error.is_offline());
        assert_eq!(source.asked(), (1, 0));
    }

    #[test]
    fn an_unknown_family_is_not_offline() {
        let dir = tempfile::tempdir().expect("a folder");
        let source = Memory {
            css: Err(FetchError::UnknownFamily),
            font: Err(FetchError::UnknownFamily),
            asked: Mutex::new((0, 0)),
        };
        let error = load(
            &source,
            &Cache::new(dir.path()),
            &family("Nope"),
            &mut |_| {},
        )
        .expect_err("unknown");
        assert!(!error.is_offline());
        assert_eq!(
            error.to_string(),
            "Google Fonts has no family with that name."
        );
    }

    #[test]
    fn a_bad_cached_file_is_deleted_and_fetched_again() {
        let dir = tempfile::tempdir().expect("a folder");
        let fonts = dir.path().join("fonts");
        fs::create_dir_all(&fonts).expect("the folder");
        fs::write(fonts.join("Lora.ttf"), b"wOF2 not a font file").expect("a bad file");
        let cache = Cache::new(dir.path());
        assert_eq!(cache.find(&family("Lora")).expect("a read"), None);
        assert!(!fonts.join("Lora.ttf").exists());

        fs::write(fonts.join("Lora.ttf"), b"wOF2 not a font file").expect("a bad file");
        let source = Memory::answering(css("Lora", LORA_URL), font_bytes(b"new"));
        let loaded = load(&source, &cache, &family("Lora"), &mut |_| {}).expect("a download");
        assert_eq!(loaded.origin, Origin::Download);
        assert_eq!(
            fs::read(fonts.join("Lora.ttf")).expect("the new file"),
            font_bytes(b"new")
        );
    }

    #[test]
    fn the_scan_lists_good_fonts_and_leaves_other_files() {
        let dir = tempfile::tempdir().expect("a folder");
        let cache = Cache::new(dir.path());
        for name in ["Roboto Mono", "Lora"] {
            cache
                .store(
                    &family(name),
                    &FontFile::parse(font_bytes(name.as_bytes())).expect("font"),
                )
                .expect("stored");
        }
        let fonts = dir.path().join("fonts");
        fs::write(fonts.join("notes.txt"), "mine").expect("a stray file");
        fs::write(fonts.join("Lora.tmp"), "half").expect("a part file");
        fs::write(fonts.join("Open+Sans.ttf"), b"junk").expect("a bad font");
        let scan = cache.scan();
        let names: Vec<_> = scan.found.iter().map(|c| c.family.as_str()).collect();
        assert_eq!(names, ["Lora", "Roboto Mono"]);
        assert!(scan.failed.is_empty());
        assert!(fonts.join("notes.txt").exists());
        assert!(fonts.join("Lora.tmp").exists());
        assert!(!fonts.join("Open+Sans.ttf").exists());
    }

    #[test]
    fn a_missing_folder_has_no_fonts_and_removing_is_idempotent() {
        let dir = tempfile::tempdir().expect("a folder");
        let cache = Cache::new(dir.path());
        assert!(cache.scan().found.is_empty());
        assert_eq!(cache.find(&family("Lora")).expect("a read"), None);
        cache.remove(&family("Lora")).expect("nothing to remove");
        cache
            .store(
                &family("Lora"),
                &FontFile::parse(font_bytes(b"")).expect("font"),
            )
            .expect("stored");
        cache.remove(&family("Lora")).expect("removed");
        assert_eq!(cache.find(&family("Lora")).expect("a read"), None);
    }

    #[test]
    fn a_cache_that_cannot_be_read_is_reported() {
        let dir = tempfile::tempdir().expect("a folder");
        // A file where the folder should be.
        fs::write(dir.path().join("fonts"), "not a folder").expect("a file");
        let cache = Cache::new(dir.path());
        assert_eq!(cache.scan().failed.len(), 1);
        assert!(cache.find(&family("Lora")).is_err());
        let source = Memory::answering(css("Lora", LORA_URL), font_bytes(b""));
        assert!(matches!(
            load(&source, &cache, &family("Lora"), &mut |_| {}),
            Err(LoadError::Cache(_))
        ));
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Whatever is typed, a parsed family is within the grammar: no
        /// space at an end or doubled, only name characters, 1 to 80 long.
        #[test]
        fn prop_a_parsed_family_is_in_the_grammar(raw in "\\PC{0,100}") {
            if let Ok(family) = Family::parse(&raw) {
                let name = family.as_str();
                prop_assert!(!name.is_empty() && name.chars().count() <= 80);
                prop_assert!(!name.starts_with(' ') && !name.ends_with(' ') && !name.contains("  "));
                prop_assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || " -_".contains(c)));
            }
        }

        /// Capitalizing changes only letters' case, is a family already,
        /// and leaves nothing to capitalize the second time.
        #[test]
        fn prop_capitalizing_changes_only_case(text in family_text()) {
            let typed = family(&text);
            match typed.capitalized() {
                Some(capitals) => {
                    prop_assert!(typed.same_as(&capitals));
                    prop_assert_ne!(&typed, &capitals);
                    prop_assert_eq!(capitals.capitalized(), None);
                    prop_assert_eq!(Family::parse(capitals.as_str()), Ok(capitals));
                }
                None => prop_assert!(typed.0.split(' ').all(|word| !word.starts_with(|c: char| c.is_ascii_lowercase()))),
            }
        }

        /// Parsing is idempotent, and extra spaces never change the family.
        #[test]
        fn prop_parsing_normalizes_once(text in family_text(), pad in "[ ]{0,3}") {
            let once = family(&text);
            prop_assert_eq!(Family::parse(once.as_str()), Ok(once.clone()));
            let padded = format!("{pad}{}{pad}", text.replace(' ', "   "));
            prop_assert_eq!(Family::parse(&padded), Ok(once));
        }

        /// A family's file name is read back as the family, and `same_as`
        /// ignores case alone.
        #[test]
        fn prop_a_file_name_round_trips(text in family_text()) {
            let family = family(&text);
            let name = family.file_name();
            prop_assert!(!name.contains(' ') && name.ends_with(".ttf"));
            prop_assert_eq!(Family::from_file_name(OsStr::new(&name)), Some(family.clone()));
            let shouted = Family::parse(&text.to_ascii_uppercase()).expect("same grammar");
            prop_assert!(family.same_as(&shouted));
        }

        /// Names that no family makes aren't read as one.
        #[test]
        fn prop_foreign_file_names_are_not_families(text in family_text()) {
            for name in [format!("{text}.tmp"), format!("{text}.TTF"), format!("{text}.otf"), text.clone()] {
                prop_assert_eq!(Family::from_file_name(OsStr::new(&name)), None);
            }
            // Spelled with a space or a doubled plus: not the name `file_name` makes.
            prop_assert_eq!(Family::from_file_name(OsStr::new(&format!("{text}.ttf"))).is_some(), !text.contains(' '));
            prop_assert_eq!(Family::from_file_name(OsStr::new(&format!("{}++x.ttf", text.replace(' ', "+")))), None);
        }

        /// The stylesheet's address names the family once, with no space.
        #[test]
        fn prop_the_stylesheet_url_names_the_family(text in family_text()) {
            let family = family(&text);
            let url = stylesheet_url(&family);
            prop_assert!(url.is_ascii() && !url.contains(' '));
            let query = url.strip_prefix("https://fonts.googleapis.com/css?family=").expect("the host");
            let name = query.strip_suffix(":wght@400&display=swap").expect("the weight");
            prop_assert_eq!(name.replace('+', " "), family.as_str());
        }

        /// A stylesheet in any of Google's spellings gives back its family
        /// and address.
        #[test]
        fn prop_a_stylesheet_round_trips(
            text in family_text(),
            path in "[A-Za-z0-9_-]{1,12}(/[A-Za-z0-9_.-]{1,12}){0,3}",
            quote in prop::sample::select(vec!["'", "\"", ""]),
            url_quote in prop::sample::select(vec!["'", "\"", ""]),
            gap in "[ \\n\\t]{0,3}",
            subdomain in prop::sample::select(vec!["fonts.", "", "x-1."]),
        ) {
            let url = format!("https://{subdomain}gstatic.com/{path}");
            let css = format!(
                "/* c */@font-face{gap}{{{gap}font-family:{gap}{quote}{text}{quote}{gap};{gap}SRC:{gap}url({url_quote}{url}{url_quote}){gap}format('truetype');{gap}}}"
            );
            let face = parse_stylesheet(&css).expect("a face");
            prop_assert_eq!(face.family.as_str(), text.as_str());
            prop_assert_eq!(face.url.as_str(), url.as_str());
        }

        /// Whatever a stylesheet says, a face that comes out is an HTTPS
        /// address on Google's host, and nothing panics.
        #[test]
        fn prop_a_face_is_always_on_googles_host(css in "\\PC{0,200}", host in "[a-z.:@/-]{0,20}") {
            let attempts = [
                css.clone(),
                format!("@font-face {{ font-family: Lora; src: url(https://{host}/a.ttf) }} {css}"),
                format!("{css} @font-face {{ font-family: 'Lora'; src: url('https://{host}') }}"),
            ];
            for attempt in attempts {
                if let Ok(face) = parse_stylesheet(&attempt) {
                    let rest = face.url.as_str().strip_prefix("https://").expect("https");
                    let host = rest.split('/').next().expect("a host");
                    prop_assert!(host == "gstatic.com" || host.ends_with(".gstatic.com"));
                    prop_assert!(!host.contains(['@', ':']));
                }
            }
        }

        /// A file is a font exactly when it has a known signature and room
        /// for a header.
        #[test]
        fn prop_a_font_has_a_signature(
            head in prop::collection::vec(any::<u8>(), 0..8),
            tail in prop::collection::vec(any::<u8>(), 0..32),
            signature in prop::sample::select(SIGNATURES.to_vec()),
        ) {
            let mut real = signature.to_vec();
            real.extend_from_slice(&[0; 8]);
            real.extend_from_slice(&tail);
            prop_assert_eq!(FontFile::parse(real.clone()).map(FontFile::into_bytes), Ok(real));

            let mut other = head.clone();
            other.extend_from_slice(&tail);
            let known = other.len() >= MIN_FONT_BYTES
                && SIGNATURES.iter().any(|s| other.starts_with(s.as_slice()));
            prop_assert_eq!(FontFile::parse(other).is_ok(), known);

            // Too short, even with a signature.
            let mut short = signature.to_vec();
            short.truncate(MIN_FONT_BYTES - 1);
            prop_assert_eq!(FontFile::parse(short), Err(NotAFont));
        }

        /// A kept font is the font that was stored, found in any case, for
        /// every family; a second store replaces the first.
        #[test]
        fn prop_the_cache_returns_what_was_stored(
            text in family_text(),
            first in prop::collection::vec(any::<u8>(), 0..64),
            second in prop::collection::vec(any::<u8>(), 0..64),
        ) {
            let dir = tempfile::tempdir().expect("a folder");
            let cache = Cache::new(dir.path());
            let family = family(&text);
            let file = |tail: &[u8]| FontFile::parse(font_bytes(tail)).expect("font");
            cache.store(&family, &file(&first)).expect("stored");
            cache.store(&family, &file(&second)).expect("replaced");
            let shouted = Family::parse(&text.to_ascii_lowercase()).expect("same grammar");
            let found = cache.find(&shouted).expect("a read").expect("a font");
            prop_assert_eq!(found.family, family);
            prop_assert_eq!(found.file, file(&second));
            prop_assert_eq!(cache.scan().found.len(), 1);
        }

        /// Loading goes to the network exactly once for a family that is
        /// not kept, and never for one that is.
        #[test]
        fn prop_load_asks_the_network_only_on_a_miss(text in family_text(), tail in prop::collection::vec(any::<u8>(), 0..32)) {
            let dir = tempfile::tempdir().expect("a folder");
            let cache = Cache::new(dir.path());
            let wanted = family(&text);
            let source = Memory::answering(css(wanted.as_str(), LORA_URL), font_bytes(&tail));
            let first = load(&source, &cache, &wanted, &mut |_| {}).expect("a download");
            prop_assert_eq!(first.origin, Origin::Download);
            prop_assert_eq!(source.asked(), (1, 1));
            let second = load(&source, &cache, &wanted, &mut |_| {}).expect("the cache");
            prop_assert_eq!(second.origin, Origin::Cache);
            prop_assert_eq!(source.asked(), (1, 1));
            prop_assert_eq!(second.file, first.file);
        }
    }
}
