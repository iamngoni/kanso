//! What accounts say, in plain words: errors, the steps of a running task,
//! storage and devices, the binding question, and the recovery-key file
//! (docs/ACCOUNTS.md › Plain-language errors, › Account page, › Library
//! binding). Pure: no window, clock, Keychain or network. A server's own
//! `message` never shows; neither does a transport error's text.

use std::path::Path;

use chrono::{DateTime, Datelike as _, Timelike as _};
use kanso_sync::{
    AccountError, Choice, DeviceInfo, EndpointError, Password, PasswordError, Plan, QuotaKind,
    RecoveryKey, RecoveryKeyError, SignOutReason, SignupClosedReason, SignupState, Step, Usage,
    UsernameError, check_new_username, normalize_username,
};

/// The button's words while a task runs.
pub fn step_label(step: Step) -> &'static str {
    match step {
        Step::StoppingSync => "Stopping sync…",
        Step::DerivingKeys => "Securing your keys…",
        Step::Contacting => "Contacting the server…",
        Step::Saving => "Saving…",
    }
}

/// What the person was doing when an error came back, which decides what a
/// wrong password or key means.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorContext {
    SignUp,
    SignIn,
    /// Signing in with the recovery key.
    Recovery,
    /// Changing the password, a new recovery key or deleting the account:
    /// the current password is checked.
    CurrentPassword,
    /// Checking a server for "Use a different server…".
    Server,
    /// Anything else signed in (devices, usage).
    Account,
}

pub const CANT_REACH: &str = "Can't reach the server. Your notes are safe on this device.";
pub const NOT_A_KANSO_SERVER: &str = "That server doesn't run this version of Kanso sync.";
pub const KEYS_WRONG: &str =
    "Something's wrong with the server's copy of your keys. Nothing was changed on this device.";
pub const PASSWORDS_DIFFER: &str = "The passwords don't match.";
const SERVER_TROUBLE: &str = "The server couldn't do that. Try again in a moment.";
/// The approver found the request's key or name doesn't match what it
/// committed to (docs/ACCOUNTS.md › The protocol, step 5).
pub const APPROVAL_CHANGED: &str = "This request changed after it was made. Don't approve.";

/// The words for a failed account task. Every variant has words, and none
/// repeats text that came from the server or the network.
pub fn error_words(err: &AccountError, context: ErrorContext) -> String {
    let words: &str = match err {
        AccountError::InvalidCredentials => match context {
            ErrorContext::Recovery => "That recovery key doesn't match this account.",
            ErrorContext::CurrentPassword => "That isn't your current password.",
            _ => "That username and password don't match.",
        },
        AccountError::UsernameTaken => "That username is taken. Try another.",
        AccountError::InvalidUsername => username_words(UsernameError::Invalid),
        AccountError::ReservedUsername => username_words(UsernameError::Reserved),
        AccountError::Username(err) => username_words(*err),
        AccountError::Password(err) => match context {
            // No account has such a password, so it can't be the right one.
            ErrorContext::SignIn => "That username and password don't match.",
            ErrorContext::CurrentPassword => "That isn't your current password.",
            _ => password_words(*err),
        },
        AccountError::RecoveryKey(err) => recovery_key_words(*err),
        AccountError::SignupClosed { reason } => signup_closed_words(*reason),
        AccountError::RateLimited { retry_after } => return rate_limited_words(*retry_after),
        AccountError::AccountDisabled => {
            "This account is turned off on the server. Ask the server's owner."
        }
        AccountError::KeyMismatch | AccountError::UnsafeKdf => KEYS_WRONG,
        AccountError::Transport(_) => CANT_REACH,
        AccountError::ServerOutdated | AccountError::NotFound => NOT_A_KANSO_SERVER,
        AccountError::Endpoint(err) => endpoint_words(err),
        AccountError::UsernameMismatch => "That isn't this account's username.",
        AccountError::Unauthorized { reason } => {
            return format!("{} Sign in again.", signed_out_sentence(*reason));
        }
        AccountError::Misconfigured => "That server isn't set up correctly. Ask its owner.",
        AccountError::ForbiddenOrigin | AccountError::UnsupportedMediaType => NOT_A_KANSO_SERVER,
        AccountError::UnsupportedKdf | AccountError::WeakKdf => {
            "That server asks for settings this Kanso doesn't use. Update Kanso, or ask the server's owner."
        }
        AccountError::QuotaExceeded { .. } => {
            "Your account is full. Delete something or ask the server's owner for more space."
        }
        AccountError::Keychain(_) => SECRET_STORE_TROUBLE,
        AccountError::UnsavedRecoveryKey { username } => {
            return format!(
                "Save the new recovery key for {username} first. Kanso shows it on the Account page."
            );
        }
        AccountError::SelfApproval => "A device can't approve itself.",
        AccountError::ApprovalUsed => "That request was already answered. Ask again.",
        AccountError::ApprovalExpired => "That approval request expired. Ask again.",
        AccountError::ApprovalChanged => APPROVAL_CHANGED,
        AccountError::BadRequest
        | AccountError::ChecksumMismatch
        | AccountError::MethodNotAllowed
        | AccountError::AccountIdTaken
        | AccountError::TooLarge
        | AccountError::Internal
        | AccountError::Protocol(_) => SERVER_TROUBLE,
    };
    words.to_owned()
}

/// "Too many attempts. Try again in N minutes.", rounded up to a minute.
pub fn rate_limited_words(retry_after_secs: u64) -> String {
    match retry_after_secs.div_ceil(60).max(1) {
        1 => "Too many attempts. Try again in a minute.".to_owned(),
        minutes => format!("Too many attempts. Try again in {minutes} minutes."),
    }
}

pub fn username_words(err: UsernameError) -> &'static str {
    match err {
        UsernameError::Invalid => {
            "Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit."
        }
        UsernameError::Reserved => "That username is reserved. Try another.",
    }
}

pub fn password_words(err: PasswordError) -> &'static str {
    match err {
        PasswordError::TooShort => "Use at least 12 characters. A few random words work well.",
        PasswordError::TooLong => "Use at most 256 characters.",
        PasswordError::SameAsUsername => "Choose a password that isn't your username.",
    }
}

pub fn recovery_key_words(err: RecoveryKeyError) -> &'static str {
    match err {
        RecoveryKeyError::Checksum => "That recovery key has a typo. Check it and try again.",
        RecoveryKeyError::BadCharacter => {
            "A recovery key only has the letters and digits shown on it. Check it and try again."
        }
        RecoveryKeyError::WrongLength => {
            "A recovery key has 28 letters and digits, in groups of four."
        }
    }
}

pub fn endpoint_words(err: &EndpointError) -> &'static str {
    match err {
        EndpointError::Invalid => "Enter the server's address, such as https://sync.example.com.",
        EndpointError::Insecure => {
            "Use an https:// address. Plain http:// only reaches this device (127.0.0.1 or localhost)."
        }
        EndpointError::Extra => "Use just the server's address, without a name, password or query.",
    }
}

pub fn signup_closed_words(reason: SignupClosedReason) -> &'static str {
    match reason {
        SignupClosedReason::Closed => "This server isn't taking new accounts.",
        SignupClosedReason::Full => "This server is full.",
        SignupClosedReason::Busy => {
            "This server has taken enough new accounts today. Try tomorrow."
        }
    }
}

/// Why this Mac was signed out, as the first sentence of what the Account
/// page and the sidebar's tooltip say.
pub fn signed_out_sentence(reason: SignOutReason) -> &'static str {
    match reason {
        SignOutReason::PasswordChanged => "Your password was changed on another device.",
        SignOutReason::Revoked => "This device was signed out from another device.",
        SignOutReason::Recovered => "Your account was recovered with its recovery key.",
        SignOutReason::Evicted => "Signed out to make room for another device.",
        SignOutReason::Expired => "Signed out after 180 days without syncing.",
        SignOutReason::SignedOut | SignOutReason::Rekeyed | SignOutReason::Unknown => {
            "This device was signed out."
        }
    }
}

/// What a full account has no room for, in the Account page's help.
pub fn quota_words(kind: QuotaKind) -> &'static str {
    match kind {
        QuotaKind::Docs => "number of items",
        QuotaKind::DocBytes => "notes and notebooks",
        QuotaKind::Files => "number of attachments",
        QuotaKind::FileBytes => "attachments",
        QuotaKind::Server => "the server",
    }
}

// MARK: Forms

/// A new password and its confirmation, for `username`.
pub fn check_new_password(
    password: &str,
    confirm: &str,
    username: &str,
) -> Result<Password, String> {
    let checked =
        Password::new(password, username).map_err(|err| password_words(err).to_owned())?;
    if password != confirm {
        return Err(PASSWORDS_DIFFER.to_owned());
    }
    Ok(checked)
}

/// The Create account form: a username nobody may be refused offline, then
/// the password and its confirmation. Returns the normalized username.
pub fn check_sign_up(
    username: &str,
    password: &str,
    confirm: &str,
) -> Result<(String, Password), String> {
    let name = check_new_username(username).map_err(|err| username_words(err).to_owned())?;
    let password = check_new_password(password, confirm, &name)?;
    Ok((name, password))
}

/// The Sign in form. A password no account can have is answered like a
/// wrong one.
pub fn check_sign_in(username: &str, password: &str) -> Result<(String, Password), String> {
    let name = normalize_username(username).map_err(|err| username_words(err).to_owned())?;
    let password = Password::new(password, &name)
        .map_err(|err| error_words(&AccountError::Password(err), ErrorContext::SignIn))?;
    Ok((name, password))
}

/// The recovery form: the username (signed out), the key, and the new
/// password twice.
pub fn check_recovery(
    username: &str,
    key: &str,
    password: &str,
    confirm: &str,
) -> Result<(String, RecoveryKey, Password), String> {
    let name = normalize_username(username).map_err(|err| username_words(err).to_owned())?;
    let key = RecoveryKey::parse(key).map_err(|err| recovery_key_words(err).to_owned())?;
    let password = check_new_password(password, confirm, &name)?;
    Ok((name, key, password))
}

/// The current password, for the tasks that check it. Anything no account
/// can have is answered like a wrong one.
pub fn check_current_password(password: &str, username: &str) -> Result<Password, String> {
    Password::new(password, username)
        .map_err(|err| error_words(&AccountError::Password(err), ErrorContext::CurrentPassword))
}

// MARK: Servers

/// The server as the forms show it: the default named as such, or the
/// address given.
pub fn server_label(server: Option<&str>) -> String {
    match server {
        Some(server) if !crate::config::is_default_server(server) => {
            kanso_sync::check_endpoint(server).unwrap_or_else(|_| server.trim().to_owned())
        }
        _ => format!("{} (default)", host_of(kanso_sync::DEFAULT_SERVER)),
    }
}

/// An address without its scheme, for a row's value.
pub fn host_of(server: &str) -> String {
    let server = server.trim();
    server
        .split_once("://")
        .map_or(server, |(_, rest)| rest)
        .trim_end_matches('/')
        .to_owned()
}

/// Under the Create account form: what happens to the notes here, and that
/// the password can't be reset, or that the server takes no accounts.
pub fn sign_up_line(notes: usize, signup: Option<SignupState>) -> String {
    match signup {
        Some(SignupState::Full) => return "This server isn't taking new accounts.".to_owned(),
        // Closed also covers a name the owner reserved (`signups only`), which
        // health doesn't advertise, so the form still submits; the server decides.
        Some(SignupState::Closed) => {
            return "This server isn't open to new accounts. If its owner reserved your username, you can still create it.".to_owned();
        }
        _ => {}
    }
    let uploaded = match notes {
        0 => String::new(),
        1 => "Your note on this device will be encrypted and uploaded. ".to_owned(),
        n => format!(
            "Your {} notes on this device will be encrypted and uploaded. ",
            format_count(n as u64)
        ),
    };
    format!("{uploaded}Kanso can't reset your password; you'll get a recovery key instead.")
}

// MARK: Storage and devices

/// `1234567` as "1,234,567".
pub fn format_count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (ix, c) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A size in binary units, as the quotas are set: "512 bytes", "3.1 MB",
/// "100 MB", "1.5 GB". One decimal below 10, none above.
pub fn format_bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if n < 1024 {
        return match n {
            1 => "1 byte".to_owned(),
            n => format!("{n} bytes"),
        };
    }
    let mut value = n as f64 / 1024.;
    let mut unit = 0;
    while value >= 1024. && unit < UNITS.len() - 1 {
        value /= 1024.;
        unit += 1;
    }
    // Rounding can carry into the next unit ("1024 KB" is "1 MB").
    if value.round() >= 1024. && unit < UNITS.len() - 1 {
        value /= 1024.;
        unit += 1;
    }
    if value < 9.95 {
        let text = format!("{value:.1}");
        let text = text.strip_suffix(".0").unwrap_or(&text);
        format!("{text} {}", UNITS[unit])
    } else {
        format!("{:.0} {}", value, UNITS[unit])
    }
}

/// "850 items · 3.1 MB of 100 MB · attachments 14 MB of 500 MB (15 of
/// 10,000)": every kind of item counts.
pub fn storage_line(usage: &Usage) -> String {
    let items = match usage.docs {
        1 => "1 item".to_owned(),
        n => format!("{} items", format_count(n)),
    };
    format!(
        "{items} · {} of {} · attachments {} of {} ({} of {})",
        format_bytes(usage.doc_bytes),
        format_bytes(usage.quota.doc_bytes),
        format_bytes(usage.file_bytes),
        format_bytes(usage.quota.file_bytes),
        format_count(usage.files),
        format_count(usage.quota.files),
    )
}

/// A device's name as the list shows it: its own, or "A Mac" until it has
/// one.
pub fn device_title(device: &DeviceInfo) -> String {
    device
        .name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "A device".to_owned())
}

/// The devices as the list shows them: this Mac first, then the most
/// recently active.
pub fn sort_devices(devices: &mut [DeviceInfo]) {
    devices.sort_by(|a, b| {
        b.current
            .cmp(&a.current)
            .then(b.last_seen_at.cmp(&a.last_seen_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

/// The confirmation for signing out another device.
pub fn revoke_question(name: &str) -> (String, String) {
    (
        format!("Sign out {name}?"),
        "It stops syncing until someone signs in on it again. Its notes stay on it.".to_owned(),
    )
}

/// What the app says after a sign-in signed another device out.
pub fn evicted_notice(device: &DeviceInfo) -> String {
    format!(
        "Signed out “{}” to make room: an account can be signed in on 10 devices at a time.",
        device_title(device)
    )
}

/// "Sign out of …?"; `unsaved_key` when a new recovery key is still on
/// screen, which signing out deletes.
/// After signing another device out.
pub fn revoked_notice(name: &str) -> String {
    format!("Signed out {name}.")
}

pub fn sign_out_question(username: &str, unsaved_key: bool) -> (String, String) {
    let body = "Notes stay on this device. Sync stops until you sign in again.";
    (
        format!("Sign out of {username}?"),
        match unsaved_key {
            true => format!("{UNSAVED_KEY_WARNING} {body}"),
            false => body.to_owned(),
        },
    )
}

/// When the platform's secret store, where sign-ins are kept, fails.
#[cfg(target_os = "macos")]
const SECRET_STORE_TROUBLE: &str =
    "Kanso couldn't use the Keychain. Unlock it, or allow Kanso when macOS asks, then try again.";
#[cfg(target_os = "windows")]
const SECRET_STORE_TROUBLE: &str =
    "Kanso couldn't use the Windows Credential Manager. Try again in a moment.";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const SECRET_STORE_TROUBLE: &str = "Kanso couldn't use the Secret Service keyring. Make sure GNOME Keyring or KWallet is running and unlocked, then try again.";

/// After a sign-out the secret store wouldn't fully carry out.
#[cfg(target_os = "macos")]
pub const SIGNED_OUT_KEYCHAIN: &str = "Signed out. Kanso couldn't remove this Mac's sign-in from the Keychain, but it won't use it again.";
#[cfg(target_os = "windows")]
pub const SIGNED_OUT_KEYCHAIN: &str = "Signed out. Kanso couldn't remove this PC's sign-in from the Credential Manager, but it won't use it again.";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const SIGNED_OUT_KEYCHAIN: &str = "Signed out. Kanso couldn't remove this device's sign-in from the Secret Service keyring, but it won't use it again.";

pub const UNSAVED_KEY_WARNING: &str =
    "Your new recovery key hasn't been saved yet; signing out deletes it.";

pub fn delete_account_words(username: &str) -> String {
    format!(
        "This deletes {username}'s encrypted notes, attachments and devices from the server. Notes on this device stay here, and other devices keep their copies. The server's backups keep a copy for up to 30 days. This can't be undone."
    )
}

pub const PASSWORD_CHANGED: &str = "Password changed. Your other devices will ask for it.";
/// Over the Change password form.
pub const CHANGE_PASSWORD_TEXT: &str = "Your other devices sign out and ask for the new password. Your recovery key changes too: the one you saved stops working, and Kanso shows the new one to save.";
pub const REMERGED: &str =
    "The server's copy was restored from a backup; this device sent its changes again.";

// MARK: Time

/// When a paused sync resumes, from local times in milliseconds, to follow
/// "Sync resumes": "at 14:30", "tomorrow at 00:00", or "on May 29 at 08:15".
pub fn resume_time(until_local_ms: i64, now_local_ms: i64) -> String {
    let (Some(until), Some(now)) = (
        DateTime::from_timestamp_millis(until_local_ms),
        DateTime::from_timestamp_millis(now_local_ms),
    ) else {
        return "later".to_owned();
    };
    let time = format!("{:02}:{:02}", until.hour(), until.minute());
    let days = until.date_naive().num_days_from_ce() - now.date_naive().num_days_from_ce();
    match days {
        ..=0 => format!("at {time}"),
        1 => format!("tomorrow at {time}"),
        _ => format!("on {} at {time}", until.format("%b %-d")),
    }
}

// MARK: The binding question

/// One of the binding question's answers, with what it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub choice: Choice,
    pub label: String,
    pub description: String,
}

/// The question a sign-in or recovery asks before binding a library that
/// has notes (docs/ACCOUNTS.md › Library binding).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub title: String,
    pub body: String,
    /// Add, then Replace; Cancel is always there too.
    pub answers: Vec<Answer>,
}

/// The question for `plan` when binding into `account`'s account, or
/// `None` when the plan asks nothing after authenticating.
pub fn binding_question(plan: &Plan, account: &str) -> Option<Question> {
    let Plan::Ask {
        from,
        notes,
        unsynced,
    } = plan
    else {
        return None;
    };
    let count = match notes {
        1 => "1 note on this device".to_owned(),
        n => format!("{} notes on this device", format_count(*n as u64)),
    };
    let never = match unsynced {
        0 => String::new(),
        n => format!(
            " ({} with changes that were never synced)",
            format_count(*n as u64)
        ),
    };
    let body = match from {
        Some(other) => format!("{count}{never} came from {other}."),
        None if *notes == 1 => format!("{count}{never} isn't in {account}'s account."),
        None => format!("{count}{never} aren't in {account}'s account."),
    };
    Some(Question {
        title: "This device already has notes".to_owned(),
        body,
        answers: vec![
            Answer {
                choice: Choice::Add,
                label: format!("Add them to {account}"),
                description: format!(
                    "They're encrypted and uploaded. A note deleted in {account}'s account but still on this device comes back, and a note changed on both sides keeps both versions."
                ),
            },
            Answer {
                choice: Choice::Replace,
                label: format!("Replace them with {account}'s notes"),
                description: format!(
                    "Kanso saves a backup of this device's library first, then downloads {account}'s notes."
                ),
            },
        ],
    })
}

/// Before a sign-up whose notes came from another account: the question
/// and its button.
pub fn upload_question(from: &str) -> (String, String, &'static str) {
    (
        "Upload to the new account?".to_owned(),
        format!("This device's notes came from {from}. Upload them to the new account?"),
        "Upload to new account",
    )
}

// MARK: The recovery key

pub const RECOVERY_KEY_TEXT: &str = "If you forget your password, this key and your username are the only way back into your account. Kanso and the server's owner can't reset it. Anyone with it can take over your account and read your notes, so keep it somewhere safe, like a password manager or on paper. Each time it's used, Kanso gives you a new one.";
/// When a new key shows while the Account page doesn't.
pub const KEY_READY: &str = "Save your new recovery key. It's on the Account page.";
/// Over a key "New recovery key" made, when the answer was lost (and over
/// any unconfirmed key found at launch), until Kanso has asked the server.
pub const KEY_UNCONFIRMED: &str = "Kanso couldn't tell yet whether the server took this key. Save it, and keep your old recovery key too until Kanso confirms which one works.";
/// Over a sign-up's key, when the answer was lost.
pub const KEY_UNCONFIRMED_SIGN_UP: &str = "Kanso couldn't tell yet whether your account was created. Save this key anyway: if it was, it's the account's recovery key. Then sign in with your password.";
/// Over a recovery's or a password change's key, when the answer was lost.
pub const KEY_UNCONFIRMED_PASSWORD: &str = "Kanso couldn't tell yet whether your password was changed. Save this key, and keep your old recovery key too until Kanso confirms which one works.";
/// Over an unconfirmed key once the server said it holds it.
pub const KEY_TAKEN: &str =
    "The server took this key: it's your recovery key now, and the old one no longer works.";
/// Over a sign-up's unconfirmed key once the server said it holds it.
pub const KEY_TAKEN_SIGN_UP: &str =
    "Your account was created, and this is its recovery key. Sign in with your password.";
/// Over a recovery's or a password change's unconfirmed key once the server
/// said it holds it.
pub const KEY_TAKEN_PASSWORD: &str =
    "Your password was changed, and this is your recovery key now: the old one no longer works.";
/// When the server said it doesn't hold an unconfirmed key (it goes).
pub const KEY_NOT_TAKEN: &str =
    "The server didn't take the new recovery key, so nothing changed: your old one still works.";
/// When the server said it doesn't hold a sign-up's unconfirmed key.
pub const KEY_NOT_TAKEN_SIGN_UP: &str =
    "Your account wasn't created after all. You can create it again.";
/// When the server said it doesn't hold a recovery's or a password change's
/// unconfirmed key.
pub const KEY_NOT_TAKEN_PASSWORD: &str =
    "Your password wasn't changed after all, so your old recovery key still works.";

/// What Kanso says once it asked the server about a key it couldn't tell
/// the server took (review-verify 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyChecked {
    /// Over the key: the server holds it.
    pub taken: &'static str,
    /// In a notification: the server doesn't hold it, and the key goes.
    pub not_taken: &'static str,
}

/// The words for once the server is asked, when `note` (over a key) says
/// Kanso couldn't tell whether the server took it; `None` for any other
/// note, whose key needs no asking.
pub fn key_checked(note: &str) -> Option<KeyChecked> {
    let (taken, not_taken) = match note {
        KEY_UNCONFIRMED => (KEY_TAKEN, KEY_NOT_TAKEN),
        KEY_UNCONFIRMED_SIGN_UP => (KEY_TAKEN_SIGN_UP, KEY_NOT_TAKEN_SIGN_UP),
        KEY_UNCONFIRMED_PASSWORD => (KEY_TAKEN_PASSWORD, KEY_NOT_TAKEN_PASSWORD),
        _ => return None,
    };
    Some(KeyChecked { taken, not_taken })
}
/// Over the key a password change set.
pub const KEY_AFTER_PASSWORD: &str =
    "Your password changed, and so did your recovery key: the one you saved no longer works.";
/// The account exists, but this Mac isn't signed in to it.
pub const CREATED_NOT_SIGNED_IN: &str = "Your account was created, but this device couldn't finish signing in. Sign in with your password.";
/// The password changed, but this Mac isn't signed in with it.
pub const CHANGED_NOT_SIGNED_IN: &str = "Your password was changed, but this device couldn't finish signing in. Sign in with the new password.";
pub const RECOVERY_KEY_FILE_NAME: &str = "Kanso recovery key.txt";
pub const COPIED_KEY: &str = "Copied. It leaves the clipboard in a minute.";
pub const ICLOUD_WARNING: &str = "This folder syncs to iCloud. Save the key there anyway?";

/// The day a key was saved, from a local time in milliseconds:
/// "2026-05-28".
pub fn file_date(local_ms: i64) -> String {
    DateTime::from_timestamp_millis(local_ms)
        .map(|at| at.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// The saved file: the key, the account it opens and where, the date, and
/// the warning.
pub fn recovery_file(key: &str, username: &str, server: &str, date: &str) -> String {
    format!(
        "Kanso recovery key\n\n{key}\n\nUsername: {username}\nServer: {server}\nSaved: {date}\n\n{RECOVERY_KEY_TEXT}\n"
    )
}

/// Whether `path` is inside iCloud Drive's folder (`~/Library/Mobile
/// Documents`), which syncs to Apple's servers.
pub fn is_icloud_path(path: &Path, home: &Path) -> bool {
    !home.as_os_str().is_empty() && path.starts_with(home.join("Library").join("Mobile Documents"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_sync::{Quota, SignOutReason};
    use proptest::prelude::*;

    fn every_error(text: String) -> Vec<AccountError> {
        let mut errors = vec![
            AccountError::BadRequest,
            AccountError::InvalidUsername,
            AccountError::ReservedUsername,
            AccountError::UnsupportedKdf,
            AccountError::WeakKdf,
            AccountError::UsernameMismatch,
            AccountError::ChecksumMismatch,
            AccountError::InvalidCredentials,
            AccountError::ForbiddenOrigin,
            AccountError::AccountDisabled,
            AccountError::NotFound,
            AccountError::MethodNotAllowed,
            AccountError::UsernameTaken,
            AccountError::AccountIdTaken,
            AccountError::SelfApproval,
            AccountError::ApprovalUsed,
            AccountError::ApprovalExpired,
            AccountError::ApprovalChanged,
            AccountError::TooLarge,
            AccountError::QuotaExceeded {
                quota: QuotaKind::Docs,
                used: 1,
                limit: 2,
            },
            AccountError::UnsupportedMediaType,
            AccountError::RateLimited { retry_after: 61 },
            AccountError::Internal,
            AccountError::Misconfigured,
            AccountError::Transport(text.clone()),
            AccountError::Protocol(text.clone()),
            AccountError::ServerOutdated,
            AccountError::UnsafeKdf,
            AccountError::KeyMismatch,
            AccountError::Keychain(kanso_sync::SecretError(text)),
            // The username is this Mac's own, from the Keychain.
            AccountError::UnsavedRecoveryKey {
                username: "bob".into(),
            },
            AccountError::Endpoint(EndpointError::Invalid),
            AccountError::Endpoint(EndpointError::Insecure),
            AccountError::Endpoint(EndpointError::Extra),
            AccountError::Password(PasswordError::TooShort),
            AccountError::Password(PasswordError::TooLong),
            AccountError::Password(PasswordError::SameAsUsername),
            AccountError::Username(UsernameError::Invalid),
            AccountError::Username(UsernameError::Reserved),
            AccountError::RecoveryKey(RecoveryKeyError::BadCharacter),
            AccountError::RecoveryKey(RecoveryKeyError::WrongLength),
            AccountError::RecoveryKey(RecoveryKeyError::Checksum),
        ];
        for reason in [
            SignupClosedReason::Closed,
            SignupClosedReason::Full,
            SignupClosedReason::Busy,
        ] {
            errors.push(AccountError::SignupClosed { reason });
        }
        for reason in SignOutReason::ALL {
            errors.push(AccountError::Unauthorized { reason });
        }
        errors
    }

    const CONTEXTS: [ErrorContext; 6] = [
        ErrorContext::SignUp,
        ErrorContext::SignIn,
        ErrorContext::Recovery,
        ErrorContext::CurrentPassword,
        ErrorContext::Server,
        ErrorContext::Account,
    ];

    #[test]
    fn error_examples() {
        let words = |err: AccountError, context| error_words(&err, context);
        assert_eq!(
            words(AccountError::InvalidCredentials, ErrorContext::SignIn),
            "That username and password don't match."
        );
        assert_eq!(
            words(AccountError::InvalidCredentials, ErrorContext::Recovery),
            "That recovery key doesn't match this account."
        );
        assert_eq!(
            words(
                AccountError::InvalidCredentials,
                ErrorContext::CurrentPassword
            ),
            "That isn't your current password."
        );
        assert_eq!(
            words(
                AccountError::RecoveryKey(RecoveryKeyError::Checksum),
                ErrorContext::Recovery
            ),
            "That recovery key has a typo. Check it and try again."
        );
        assert_eq!(
            words(AccountError::UsernameTaken, ErrorContext::SignUp),
            "That username is taken. Try another."
        );
        assert_eq!(
            words(
                AccountError::SignupClosed {
                    reason: SignupClosedReason::Busy
                },
                ErrorContext::SignUp
            ),
            "This server has taken enough new accounts today. Try tomorrow."
        );
        assert_eq!(
            rate_limited_words(1),
            "Too many attempts. Try again in a minute."
        );
        assert_eq!(
            rate_limited_words(60),
            "Too many attempts. Try again in a minute."
        );
        assert_eq!(
            rate_limited_words(61),
            "Too many attempts. Try again in 2 minutes."
        );
        assert_eq!(
            words(AccountError::Transport("dns".into()), ErrorContext::SignIn),
            CANT_REACH
        );
        assert_eq!(
            words(AccountError::ServerOutdated, ErrorContext::Server),
            NOT_A_KANSO_SERVER
        );
        assert_eq!(
            words(AccountError::KeyMismatch, ErrorContext::SignIn),
            KEYS_WRONG
        );
        assert_eq!(
            words(
                AccountError::Unauthorized {
                    reason: SignOutReason::PasswordChanged
                },
                ErrorContext::CurrentPassword
            ),
            "Your password was changed on another device. Sign in again."
        );
        // A short password at sign-in can't be anyone's: a plain mismatch.
        assert_eq!(
            words(
                AccountError::Password(PasswordError::TooShort),
                ErrorContext::SignIn
            ),
            "That username and password don't match."
        );
        assert_eq!(
            words(
                AccountError::Password(PasswordError::TooShort),
                ErrorContext::SignUp
            ),
            "Use at least 12 characters. A few random words work well."
        );
    }

    #[test]
    fn form_examples() {
        assert!(check_sign_up("Ngoni", "correct horse battery", "correct horse battery").is_ok());
        assert_eq!(
            check_sign_up("ngoni", "correct horse battery", "correct horse").err(),
            Some(PASSWORDS_DIFFER.to_owned())
        );
        assert_eq!(
            check_sign_up("admin", "correct horse battery", "correct horse battery").err(),
            Some("That username is reserved. Try another.".to_owned())
        );
        assert_eq!(
            check_sign_up("a", "correct horse battery", "correct horse battery").err(),
            Some(username_words(UsernameError::Invalid).to_owned())
        );
        assert_eq!(
            check_sign_up("ngoni", "short", "short").err(),
            Some(password_words(PasswordError::TooShort).to_owned())
        );
        assert_eq!(
            check_sign_up("ngonimangudya", "NGONIMANGUDYA", "NGONIMANGUDYA").err(),
            Some("Choose a password that isn't your username.".to_owned())
        );
        let (name, _) = check_sign_in(" Ngoni ", "correct horse battery").unwrap();
        assert_eq!(name, "ngoni");
        assert_eq!(
            check_sign_in("ngoni", "short").err(),
            Some("That username and password don't match.".to_owned())
        );
        let key = RecoveryKey::from_bytes([7; 16]).display();
        assert!(
            check_recovery(
                "ngoni",
                &key,
                "correct horse battery",
                "correct horse battery"
            )
            .is_ok()
        );
        let typo = format!("{}X", &key[..key.len() - 1]);
        let typo = if typo == key {
            format!("{}Y", &key[..key.len() - 1])
        } else {
            typo
        };
        assert_eq!(
            check_recovery(
                "ngoni",
                &typo,
                "correct horse battery",
                "correct horse battery"
            )
            .err(),
            Some(recovery_key_words(RecoveryKeyError::Checksum).to_owned())
        );
    }

    #[test]
    fn storage_examples() {
        let usage = Usage {
            docs: 850,
            doc_bytes: 3_250_586,
            files: 15,
            file_bytes: 14 * 1024 * 1024,
            writes_today: 12,
            quota: Quota {
                docs: 50_000,
                doc_bytes: 104_857_600,
                files: 10_000,
                file_bytes: 524_288_000,
                writes_per_day: 20_000,
            },
        };
        assert_eq!(
            storage_line(&usage),
            "850 items · 3.1 MB of 100 MB · attachments 14 MB of 500 MB (15 of 10,000)"
        );
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(1), "1 byte");
        assert_eq!(format_bytes(1023), "1023 bytes");
        assert_eq!(format_bytes(1024), "1 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024 - 1), "1 MB");
        assert_eq!(format_bytes(6_000_000_000), "5.6 GB");
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1000), "1,000");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }

    #[test]
    fn revoked_examples() {
        assert_eq!(revoked_notice("Laptop"), "Signed out Laptop.");
    }

    #[test]
    fn unsaved_key_examples() {
        assert_eq!(
            error_words(
                &AccountError::UnsavedRecoveryKey {
                    username: "bob".into()
                },
                ErrorContext::SignUp
            ),
            "Save the new recovery key for bob first. Kanso shows it on the Account page."
        );
        assert_eq!(
            sign_out_question("ngoni", false),
            (
                "Sign out of ngoni?".to_owned(),
                "Notes stay on this device. Sync stops until you sign in again.".to_owned()
            )
        );
        assert_eq!(
            sign_out_question("ngoni", true).1,
            "Your new recovery key hasn't been saved yet; signing out deletes it. Notes stay on this device. Sync stops until you sign in again."
        );
    }

    #[test]
    fn question_examples() {
        let plan = Plan::Ask {
            from: None,
            notes: 12,
            unsynced: 3,
        };
        let question = binding_question(&plan, "ngoni").unwrap();
        assert_eq!(question.title, "This device already has notes");
        assert_eq!(
            question.body,
            "12 notes on this device (3 with changes that were never synced) aren't in ngoni's account."
        );
        assert_eq!(question.answers[0].label, "Add them to ngoni");
        assert_eq!(question.answers[1].label, "Replace them with ngoni's notes");
        let from_alex = Plan::Ask {
            from: Some("alex".into()),
            notes: 1,
            unsynced: 0,
        };
        assert_eq!(
            binding_question(&from_alex, "ngoni").unwrap().body,
            "1 note on this device came from alex."
        );
        assert_eq!(binding_question(&Plan::Fresh, "ngoni"), None);
        assert_eq!(binding_question(&Plan::Resume, "ngoni"), None);
    }

    #[test]
    fn small_examples() {
        assert_eq!(server_label(None), "sync.kansoapp.app (default)");
        // The same server under its first name reads as the default too.
        assert_eq!(
            server_label(Some("https://kanso.antonlabs.cc/")),
            "sync.kansoapp.app (default)"
        );
        assert_eq!(
            server_label(Some("https://sync.example.dev/")),
            "https://sync.example.dev"
        );
        assert_eq!(host_of("https://sync.example.dev/"), "sync.example.dev");
        assert_eq!(
            sign_up_line(42, Some(SignupState::Open)),
            "Your 42 notes on this device will be encrypted and uploaded. Kanso can't reset your password; you'll get a recovery key instead."
        );
        assert_eq!(
            sign_up_line(0, None),
            "Kanso can't reset your password; you'll get a recovery key instead."
        );
        assert_eq!(
            sign_up_line(42, Some(SignupState::Full)),
            "This server isn't taking new accounts."
        );
        let day = 86_400_000;
        let now = 1_779_961_500_000; // 2026-05-28 09:45
        assert_eq!(resume_time(now + 3_600_000, now), "at 10:45");
        assert_eq!(resume_time(now - now % day + day, now), "tomorrow at 00:00");
        assert_eq!(resume_time(now + 2 * day, now), "on May 30 at 09:45");
        assert_eq!(file_date(now), "2026-05-28");
        let home = Path::new("/Users/me");
        assert!(is_icloud_path(
            Path::new("/Users/me/Library/Mobile Documents/com~apple~CloudDocs/key.txt"),
            home
        ));
        assert!(!is_icloud_path(
            Path::new("/Users/me/Documents/key.txt"),
            home
        ));
        let file = recovery_file("0K7M-QX9P", "ngoni", "https://example.dev", "28 May 2026");
        assert!(file.contains("0K7M-QX9P") && file.contains("Username: ngoni"));
        assert!(file.contains("Anyone with it can take over your account"));
    }

    fn gen_usage() -> impl Strategy<Value = Usage> {
        (
            (
                0u64..2_000_000,
                0u64..10_000_000_000,
                0u64..20_000,
                0u64..60_000_000_000,
            ),
            (
                1u64..2_000_000,
                1u64..10_000_000_000,
                1u64..20_000,
                1u64..60_000_000_000,
            ),
        )
            .prop_map(
                |((docs, doc_bytes, files, file_bytes), (qd, qdb, qf, qfb))| Usage {
                    docs,
                    doc_bytes,
                    files,
                    file_bytes,
                    writes_today: 0,
                    quota: Quota {
                        docs: qd,
                        doc_bytes: qdb,
                        files: qf,
                        file_bytes: qfb,
                        writes_per_day: 1,
                    },
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Every account error has words in every context: a sentence of
        /// its own, never the text that came with it.
        #[test]
        fn prop_every_error_has_words(text in "zq[a-z0-9 :./_-]{3,30}") {
            for err in every_error(text.clone()) {
                for context in CONTEXTS {
                    let words = error_words(&err, context);
                    prop_assert!(words.ends_with('.'), "{:?}: {}", err, words);
                    prop_assert!(words.chars().next().is_some_and(char::is_uppercase), "{}", words);
                    prop_assert!(!words.contains("zq"), "{:?}: {}", err, words);
                }
            }
        }

        /// A rate limit names whole minutes, rounded up, and never "0".
        #[test]
        fn prop_rate_limits_round_up(secs in 0u64..100_000) {
            let words = rate_limited_words(secs);
            let minutes = secs.div_ceil(60).max(1);
            if minutes == 1 {
                prop_assert!(words.ends_with("in a minute."));
            } else {
                let expected = format!("in {minutes} minutes.");
                prop_assert!(words.ends_with(&expected), "{}", words);
            }
        }

        /// Counts read back as the same number.
        #[test]
        fn prop_counts_read_back(n: u64) {
            let shown = format_count(n);
            prop_assert_eq!(shown.replace(',', "").parse::<u64>().unwrap(), n);
            // Groups of three after the first.
            for group in shown.split(',').skip(1) {
                prop_assert_eq!(group.len(), 3);
            }
        }

        /// Sizes read back within the rounding of their one or two
        /// significant digits, in the largest unit that keeps a whole one.
        #[test]
        fn prop_sizes_read_back(n in 0u64..(1u64 << 48)) {
            let shown = format_bytes(n);
            let (number, unit) = shown.split_once(' ').unwrap();
            let number: f64 = number.parse().unwrap();
            let scale = match unit {
                "byte" | "bytes" => 1.,
                "KB" => 1024.,
                "MB" => 1024f64.powi(2),
                "GB" => 1024f64.powi(3),
                "TB" => 1024f64.powi(4),
                other => panic!("unit {other}"),
            };
            let back = number * scale;
            let n = n as f64;
            prop_assert!((back - n).abs() <= n * 0.051 + 0.5, "{} for {}", shown, n);
            prop_assert!(number >= 1. || n == 0., "{}", shown);
            prop_assert!(number < 1024., "{}", shown);
        }

        /// The storage line says every figure it was given.
        #[test]
        fn prop_storage_line_says_everything(usage in gen_usage()) {
            let line = storage_line(&usage);
            prop_assert!(line.starts_with(&format_count(usage.docs)), "{}", line);
            for figure in [
                format_bytes(usage.doc_bytes),
                format_bytes(usage.quota.doc_bytes),
                format_bytes(usage.file_bytes),
                format_bytes(usage.quota.file_bytes),
                format!("({} of {})", format_count(usage.files), format_count(usage.quota.files)),
            ] {
                prop_assert!(line.contains(&figure), "{} lacks {}", line, figure);
            }
        }

        /// The new-password check refuses exactly what Password refuses, and
        /// then a confirmation that differs.
        #[test]
        fn prop_new_password_needs_rules_and_confirmation(
            password in "[a-zA-Z ]{0,20}",
            confirm in "[a-zA-Z ]{0,20}",
            same: bool,
            username in "[a-z]{3,10}",
        ) {
            let confirm = if same { password.clone() } else { confirm };
            let result = check_new_password(&password, &confirm, &username);
            match Password::new(&password, &username) {
                Err(err) => prop_assert_eq!(result.err(), Some(password_words(err).to_owned())),
                Ok(_) if password != confirm => prop_assert_eq!(result.err(), Some(PASSWORDS_DIFFER.to_owned())),
                Ok(_) => prop_assert!(result.is_ok()),
            }
        }

        /// Any recovery key round-trips through the recovery form, however
        /// it is spaced and cased.
        #[test]
        fn prop_recovery_form_takes_any_written_key(bytes: [u8; 16], lower: bool, spaced: bool) {
            let shown = RecoveryKey::from_bytes(bytes).display();
            let typed = if lower { shown.to_lowercase() } else { shown.clone() };
            let typed = if spaced { typed.replace('-', " ") } else { typed };
            let (_, key, _) = check_recovery("ngoni", &typed, "correct horse battery", "correct horse battery").unwrap();
            prop_assert_eq!(key.display(), shown);
        }

        /// The binding question names the account, counts the notes, and
        /// offers Add (which warns about deleted notes) before Replace
        /// (which promises a backup).
        #[test]
        fn prop_binding_question_says_what_happens(
            notes in 1usize..5000,
            unsynced in 0usize..5000,
            from in prop::option::of("[a-z]{3,8}"),
            account in "[a-z]{3,8}",
        ) {
            let plan = Plan::Ask { from: from.clone(), notes, unsynced };
            let question = binding_question(&plan, &account).unwrap();
            prop_assert!(question.body.starts_with(&format_count(notes as u64)), "{}", question.body);
            prop_assert_eq!(question.body.contains("never synced"), unsynced > 0);
            match &from {
                Some(other) => {
                    let came = format!("came from {other}.");
                    prop_assert!(question.body.ends_with(&came), "{}", question.body);
                }
                None => {
                    let theirs = format!("{account}'s account.");
                    prop_assert!(question.body.ends_with(&theirs), "{}", question.body);
                }
            }
            let choices: Vec<Choice> = question.answers.iter().map(|a| a.choice).collect();
            prop_assert_eq!(choices, vec![Choice::Add, Choice::Replace]);
            prop_assert!(question.answers[0].description.contains("comes back"));
            prop_assert!(question.answers[1].description.contains("backup"));
            for answer in &question.answers {
                prop_assert!(answer.label.contains(&account));
            }
        }

        /// The device list: this Mac first, then the most recently active,
        /// and nothing lost or added.
        #[test]
        fn prop_devices_sort_this_mac_first(seen in prop::collection::vec((0i64..1_000_000, any::<bool>()), 0..12)) {
            let mut devices: Vec<DeviceInfo> = seen
                .iter()
                .enumerate()
                .map(|(ix, (at, current))| DeviceInfo {
                    id: format!("d_{ix:032x}"),
                    name: None,
                    created_at: 0,
                    last_seen_at: *at,
                    current: *current,
                })
                .collect();
            let mut ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
            sort_devices(&mut devices);
            for pair in devices.windows(2) {
                prop_assert!(pair[0].current >= pair[1].current);
                if pair[0].current == pair[1].current {
                    prop_assert!(pair[0].last_seen_at >= pair[1].last_seen_at);
                }
            }
            let mut sorted: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
            ids.sort();
            sorted.sort();
            prop_assert_eq!(ids, sorted);
        }

        /// A pause ends today, tomorrow or on a named day, at the minute it
        /// ends; a time already past reads as a time today.
        #[test]
        fn prop_resume_times_name_the_minute(now in 0i64..4_000_000_000_000, ahead in -100_000_000i64..1_000_000_000) {
            let until = now + ahead;
            let words = resume_time(until, now);
            let at = DateTime::from_timestamp_millis(until).unwrap();
            let expected = format!("{:02}:{:02}", at.hour(), at.minute());
            prop_assert!(words.ends_with(&expected), "{}", words);
            let days = at.date_naive().num_days_from_ce()
                - DateTime::from_timestamp_millis(now).unwrap().date_naive().num_days_from_ce();
            prop_assert_eq!(words.starts_with("tomorrow"), days == 1);
            prop_assert_eq!(words.starts_with("on "), days > 1);
            prop_assert_eq!(words.len() == 8, days <= 0);
        }

        /// Round-trip: a file's date reads back as the day it was saved.
        #[test]
        fn prop_file_dates_read_back(at in 0i64..4_000_000_000_000) {
            let date = file_date(at);
            let day = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            prop_assert_eq!(day, DateTime::from_timestamp_millis(at).unwrap().date_naive());
        }

        /// Only paths inside iCloud Drive's folder are taken for iCloud.
        #[test]
        fn prop_icloud_paths_are_inside_mobile_documents(rest in "(/[a-zA-Z ~.]{1,8}){0,3}", inside: bool) {
            let home = Path::new("/Users/me");
            let path = if inside {
                format!("/Users/me/Library/Mobile Documents{rest}")
            } else {
                format!("/Users/me/Documents{rest}")
            };
            prop_assert_eq!(is_icloud_path(Path::new(&path), home), inside);
        }
    }
}
