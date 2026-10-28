//! What the account calls hand the shell: where the account stands, what a
//! server offers, the steps of a provider or passkey sign-in, a device
//! approval's wait, the account's devices and ways in, and the requests
//! this phone may approve.
//!
//! Plain data: no key, token or password wrap is here. A recovery key is,
//! because the person must write it down.

/// A sign-in provider the Worker runs (docs/ACCOUNTS.md › Sign in with
/// Apple, Google and GitHub).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Apple,
    Google,
    Github,
}

/// The ways in a server offers that this phone can use.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerMethods {
    /// Whether the server takes new accounts.
    pub signups_open: bool,
    pub password: bool,
    /// Platform passkeys: the server advertises them for a relying party
    /// this app is entitled to.
    pub passkeys: bool,
    pub providers: Vec<ProviderKind>,
}

/// Where this phone's account stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AccountState {
    #[default]
    SignedOut,
    SignedIn {
        username: String,
        server: String,
    },
    /// Signed in, but this phone has no data key yet (a provider, or a
    /// passkey that can't unlock): the password, another device or the
    /// recovery key opens it.
    Locked(LockedInfo),
    /// A sign-in found notes on this phone that aren't this account's:
    /// ask, then answer with `AccountHandle::answer_binding`.
    Question(BindingQuestion),
}

/// A signed-in account this phone can't open yet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockedInfo {
    pub username: String,
    pub server: String,
    /// Whether the account has a password, so it can unlock this phone.
    pub has_password: bool,
    /// A device approval this phone asked for earlier, still to resume.
    pub approval_pending: bool,
}

/// The question a sign-in asks before it binds this phone's library.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingQuestion {
    pub title: String,
    pub body: String,
    /// Add, then Replace; Cancel is always offered too.
    pub answers: Vec<BindingAnswer>,
}

/// An answer to a [`BindingQuestion`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingAnswer {
    /// Add this phone's notes to the account.
    Add,
    /// Back this phone's notes up, then replace them with the account's.
    Replace,
}

/// A recovery key to show once, grouped as the desktop shows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecoveryKeyText {
    pub key: String,
}

/// An account made, or a password set with the recovery key: where the
/// account stands and the new recovery key to write down.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignedUp {
    pub state: AccountState,
    pub recovery_key: RecoveryKeyText,
}

/// A step of a provider sign-in, for the shell to act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderEvent {
    /// Open this page in the system's sign-in session. `callback_scheme`
    /// is `kanso` for Apple, whose page ends at a `kanso://` address;
    /// `None` for Google (Kanso's loopback takes the code) and GitHub (a
    /// device code).
    OpenPage {
        url: String,
        callback_scheme: Option<String>,
    },
    /// GitHub: show this code beside the page.
    ShowCode {
        user_code: String,
        verification_uri: String,
    },
    /// The flow has what it needs: close the page if it is still open.
    ClosePage,
    /// No account has this identity: ask for a username, then
    /// `AccountHandle::finish_provider_sign_up`.
    NeedsUsername {
        suggested: String,
    },
    Finished(AccountState),
    Failed {
        message: String,
    },
    Cancelled,
}

/// Which passkey ceremony the native sheet runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasskeyRequestKind {
    Register,
    Assert,
}

/// A passkey ceremony for the native plugin to run. Its answer goes from
/// the plugin to Rust directly (`kanso_passkey_answer`), never through
/// Dart, so the PRF output never reaches the shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasskeyRequest {
    pub ticket: u64,
    pub kind: PasskeyRequestKind,
    /// The server's options, base64url, with nothing secret in them.
    pub options_json: String,
}

/// Why the shell couldn't run a passkey ceremony.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasskeyFailure {
    Cancelled,
    Unavailable,
    Failed,
}

/// Where a device approval this phone asked for stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApprovalWait {
    pub phase: ApprovalPhase,
    /// The six digits both screens compare, "123 456", once a device has
    /// answered.
    pub code: Option<String>,
    /// How many of the account's other devices could answer.
    pub other_devices: u32,
    /// Offer the recovery key instead: two minutes went by, or no device
    /// can answer.
    pub offer_recovery: bool,
}

/// The steps of a device approval, as the waiting screen shows them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ApprovalPhase {
    #[default]
    Requesting,
    Waiting,
    /// A device answered: compare the code.
    Compare,
    Unlocking,
    Approved,
    NoDevices,
    Cancelled,
    Expired,
    Failed {
        message: String,
    },
}

/// One of the account's devices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceRow {
    pub id: String,
    /// Its name, as the desktop's device list shows it.
    pub title: String,
    pub this_device: bool,
    pub last_seen_ms: Option<i64>,
}

/// Whether a provider is linked to the account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkedProviderRow {
    pub provider: ProviderKind,
    pub linked: bool,
}

/// One of the account's passkeys.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PasskeyRow {
    pub id: String,
    pub label: String,
    pub added_ms: i64,
    pub last_used_ms: Option<i64>,
    /// Whether it unlocks a new device by itself (it has a PRF wrap).
    pub can_unlock: bool,
}

/// The signed-in account, for Settings › Account.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountDetails {
    pub username: String,
    pub server: String,
    pub devices: Vec<DeviceRow>,
    /// "1.2 MB of 1 GB", when the server says.
    pub storage_line: Option<String>,
    pub has_password: bool,
    pub providers: Vec<LinkedProviderRow>,
    pub passkeys: Vec<PasskeyRow>,
    /// A recovery key was shown and not yet confirmed saved.
    pub recovery_key_unsaved: bool,
}

/// Where a request from another device stands, on this phone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptState {
    /// Just seen; this phone is answering with its key.
    #[default]
    New,
    /// Answered; waiting for the device to name itself.
    Challenged,
    /// Named: its name and the code can be shown, and it can be approved.
    Ready,
    /// The device's key isn't the one it committed to. Never approve.
    Changed,
}

/// A request from another device to join the account.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApprovalPrompt {
    pub id: String,
    pub device_name: Option<String>,
    pub asked_at_ms: i64,
    pub expires_at_ms: i64,
    pub code: Option<String>,
    pub state: PromptState,
}
