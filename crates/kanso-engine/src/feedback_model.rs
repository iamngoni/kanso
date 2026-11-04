//! The data a feedback report is made of (issue #232, docs/FEEDBACK-CLIENT.md).
//!
//! These are plain values. Nothing here reads a note: a report is built from
//! what the person typed and from [`AppInfo`], whose fields are all enums or
//! short patterned tokens.

/// What the person says the report is. Only a hint: triage may override it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Bug,
    Idea,
    Question,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Bug, Kind::Idea, Kind::Question];

    /// The word the wire uses.
    pub fn wire(self) -> &'static str {
        match self {
            Kind::Bug => "bug",
            Kind::Idea => "idea",
            Kind::Question => "question",
        }
    }

    pub fn from_wire(word: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.wire() == word)
    }
}

/// The platform a report comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub fn wire(self) -> &'static str {
        match self {
            Platform::Macos => "macos",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::Ios => "ios",
            Platform::Android => "android",
        }
    }

    /// The platform this build runs on, from the compile target.
    pub fn current() -> Platform {
        if cfg!(target_os = "ios") {
            Platform::Ios
        } else if cfg!(target_os = "android") {
            Platform::Android
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Macos
        } else {
            Platform::Linux
        }
    }
}

/// Where in the app the person was: a fixed list, never a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    Editor,
    NoteList,
    Settings,
    Capture,
    Ask,
    Board,
    Graph,
    Calendar,
    Sync,
    Account,
    Other,
}

impl Screen {
    pub const ALL: [Screen; 11] = [
        Screen::Editor,
        Screen::NoteList,
        Screen::Settings,
        Screen::Capture,
        Screen::Ask,
        Screen::Board,
        Screen::Graph,
        Screen::Calendar,
        Screen::Sync,
        Screen::Account,
        Screen::Other,
    ];

    pub fn wire(self) -> &'static str {
        match self {
            Screen::Editor => "editor",
            Screen::NoteList => "note-list",
            Screen::Settings => "settings",
            Screen::Capture => "capture",
            Screen::Ask => "ask",
            Screen::Board => "board",
            Screen::Graph => "graph",
            Screen::Calendar => "calendar",
            Screen::Sync => "sync",
            Screen::Account => "account",
            Screen::Other => "other",
        }
    }
}

/// Whether and where the library syncs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyncMode {
    None,
    Kanso,
    SelfHosted,
}

impl SyncMode {
    pub const ALL: [SyncMode; 3] = [SyncMode::None, SyncMode::Kanso, SyncMode::SelfHosted];

    pub fn wire(self) -> &'static str {
        match self {
            SyncMode::None => "none",
            SyncMode::Kanso => "kanso",
            SyncMode::SelfHosted => "self-hosted",
        }
    }
}

/// The app info a report may carry. Every field is an enum or a token that
/// [`crate::feedback`] cleans to the Worker's pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppInfo {
    pub platform: Platform,
    /// `major.minor.patch`.
    pub version: String,
    /// The build number, digits only.
    pub build: String,
    /// The release's patch counter, when there is one.
    pub patch: Option<u32>,
    pub os: String,
    pub device: String,
    pub locale: String,
    pub screen: Screen,
    pub sync: SyncMode,
    /// Lines already through `scrub_log_line`.
    pub errors: Vec<String>,
}

/// A screenshot as the report declares it. The bytes go up separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenshotDecl {
    /// `image/png` or `image/jpeg`.
    pub mime: &'static str,
    pub bytes: u64,
    /// 64 lowercase hex characters.
    pub sha256: String,
}

/// What the person filled in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub kind: Kind,
    pub text: String,
    /// `Some` when "Credit me by name" is on.
    pub credit_name: Option<String>,
    pub screenshot: Option<ScreenshotDecl>,
}

/// Why a draft can't be sent yet. Each maps to a line of copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftError {
    /// Fewer than 10 characters.
    TooShort,
    /// More than 4,000 characters.
    TooLong,
    /// A control character other than a line feed.
    BadCharacter,
    /// The credit name is blank, over 39 characters or has a forbidden one.
    BadCredit,
    /// The image is over 5 MB, empty, or not a PNG or a JPEG.
    BadScreenshot,
    /// The id is not `fb_` and 26 base32 characters.
    BadId,
    /// An app info field is off-pattern (its JSON path).
    BadApp(&'static str),
}

/// Where a report stands, from the server's list answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// On the device, not yet accepted.
    Waiting,
    Received,
    Noise,
    Duplicate,
    Filed,
    Building,
    PrOpen,
    Merged,
    Released,
    Closed,
}

impl Status {
    pub const ALL: [Status; 10] = [
        Status::Waiting,
        Status::Received,
        Status::Noise,
        Status::Duplicate,
        Status::Filed,
        Status::Building,
        Status::PrOpen,
        Status::Merged,
        Status::Released,
        Status::Closed,
    ];

    pub fn wire(self) -> &'static str {
        match self {
            Status::Waiting => "waiting",
            Status::Received => "received",
            Status::Noise => "noise",
            Status::Duplicate => "duplicate",
            Status::Filed => "filed",
            Status::Building => "building",
            Status::PrOpen => "pr_open",
            Status::Merged => "merged",
            Status::Released => "released",
            Status::Closed => "closed",
        }
    }

    /// A status the server sent; `None` for one this app doesn't know.
    pub fn from_wire(word: &str) -> Option<Status> {
        Status::ALL.into_iter().find(|status| status.wire() == word)
    }
}

/// One report as My feedback shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportView {
    pub id: String,
    pub created_ms: i64,
    pub kind: Kind,
    /// The first line of the text.
    pub preview: String,
    pub status: Status,
    pub issue: Option<u32>,
    pub duplicate_of: Option<u32>,
    pub pr: Option<u32>,
    /// `0.9.2`, without a leading `v`.
    pub release: Option<String>,
    /// A delete is queued and not yet through.
    pub deleting: bool,
}

/// What My feedback's chip says, as a stable key plus its arguments, so each
/// app words it in its own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chip {
    WaitingToSend,
    Received,
    FiledAs(u32),
    Building,
    FixProposed,
    Merged,
    ReleasedIn(String),
    NotFiled,
    SameAs(u32),
    Closed,
    Deleting,
}
