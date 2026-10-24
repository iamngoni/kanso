//! [`MemoryServer`], the in-memory test server: the Worker's v2 rules for
//! accounts, sessions, sealed sync, quotas and limits (frozen by step 1).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::account::{Session, SignupState};
use crate::api::{
    AccountResponse, Api, Backend, DeleteRequest, DeleteResponse, DeviceNameRequest, DeviceRow,
    DevicesResponse, Health, LoginRequest, LoginResponse, NewDevice, PasswordRequest,
    PasswordResponse, PreloginRequest, PreloginResponse, RecoverFinishRequest, RecoverStartRequest,
    RecoverStartResponse, RecoveryKeyRequest, RecoveryKeyResponse, Route, SessionResponse,
    SignupRequest, WireDevice, WireKdf, WireQuota, WireUsage,
};
use crate::crypto::{KdfParams, b64, hex, hkdf32, hmac, random, sha256_hex, unb64};
use crate::sealed::PlainView;
use crate::username::{UsernameError, check_new_username, normalize_username};
use crate::wire::{
    ApiError, DEFAULT_CHANGES_LIMIT, DOC_ROW_COST, FILE_ROW_COST, MAX_ACTIVE_DEVICES,
    MAX_CHANGES_LIMIT as MAX_WIRE_CHANGES_LIMIT, MAX_PUSH_BODY, MAX_PUSH_DOCS, MAX_SEALED_FILE,
    MAX_SEALED_LEN, MIN_SEALED_FILE, MIN_SEALED_LEN, PAGE_SEALED_BUDGET, PROTOCOL_VERSION,
    PushRequest, PushResponse, PutFileResponse, QuotaKind, SESSION_IDLE_MS, SessionToken,
    SignOutReason, SignupClosedReason, WireChange, WireDoc, WirePage, WirePushDoc, WirePushResult,
    is_account_id, is_base64url, is_device_id, is_hex16, is_sealed_name, is_token, is_wire_file_id,
    is_wire_id, is_wire_rev,
};

// ---------------------------------------------------------------------------
// MemoryServer: the v2 test server (docs/ACCOUNTS.md › Modules, › Build order)
// ---------------------------------------------------------------------------

/// The `settings` rows `cloud/migrations/0002_accounts.sql` seeds, which
/// [`MemoryServer::new`] starts from, except that its sign-ups are open.
pub const DEFAULT_SETTINGS: [(&str, &str); 13] = [
    ("signup", "closed"),
    ("max_accounts", "50"),
    ("signups_per_day", "5"),
    ("quota_docs", "50000"),
    ("quota_doc_bytes", "104857600"),
    ("quota_files", "10000"),
    ("quota_file_bytes", "524288000"),
    ("writes_per_day", "20000"),
    ("upload_bytes_per_day", "1073741824"),
    ("global_doc_bytes", "6000000000"),
    ("global_file_bytes", "50000000000"),
    ("global_writes_per_day", "300000"),
    ("global_upload_bytes_per_day", "5000000000"),
];

/// Whether D1's `CHECK` on `settings` accepts this row.
pub fn is_valid_setting(name: &str, value: &str) -> bool {
    if name == "signup" {
        matches!(value, "open" | "closed")
            || (value.starts_with("only:") && (8..=37).contains(&value.len()))
    } else {
        (1..=15).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_digit())
    }
}

/// The sign-up switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signup {
    Open,
    Closed,
    /// Only this (normalized) username may sign up.
    Only(String),
}

/// Default per-account quotas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
}

impl Limits {
    /// The migration's defaults.
    pub const DEFAULT: Limits = Limits {
        docs: 50_000,
        doc_bytes: 104_857_600,
        files: 10_000,
        file_bytes: 524_288_000,
    };
}

/// One account's quota overrides; `None` uses the default (D1's NULL).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuotaOverride {
    pub docs: Option<u64>,
    pub doc_bytes: Option<u64>,
    pub files: Option<u64>,
    pub file_bytes: Option<u64>,
    /// `quota_writes_per_day`: the account's own `writes-account` budget.
    pub writes_per_day: Option<u64>,
}

/// A session row as the server holds it (names stay sealed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerDevice {
    pub id: String,
    pub sealed_name: Option<String>,
    pub created_at: i64,
    pub last_seen_at: i64,
    /// `None` while active.
    pub revoked: Option<SignOutReason>,
}

/// Requests answered, per route, and documents pushed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerStats {
    pub requests: BTreeMap<Route, usize>,
    pub pushed_docs: usize,
}

impl ServerStats {
    pub fn count(&self, route: Route) -> usize {
        self.requests.get(&route).copied().unwrap_or(0)
    }
}

const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 86_400_000;
const QUARTER_HOUR_MS: i64 = 900_000;
/// How many revoked session rows an account keeps.
const MAX_REVOKED_ROWS: usize = 90;
/// The subject of every IP rule: one client, like the Worker without
/// `CF-Connecting-IP`.
const CLIENT_IP: &str = "unknown";

/// The D1 rate-limit rules (docs/ACCOUNTS.md › Rate limits).
#[derive(Clone, Copy)]
enum Rule {
    SignupIp,
    LoginUser,
    LoginIp,
    RecoverIp,
    PasswordAccount,
    WritesAccount,
    UploadsAccount,
}

impl Rule {
    fn name(self) -> &'static str {
        match self {
            Rule::SignupIp => "signup-ip",
            Rule::LoginUser => "login-user",
            Rule::LoginIp => "login-ip",
            Rule::RecoverIp => "recover-ip",
            Rule::PasswordAccount => "password-account",
            Rule::WritesAccount => "writes-account",
            Rule::UploadsAccount => "uploads-account",
        }
    }

    fn window(self) -> i64 {
        match self {
            Rule::SignupIp | Rule::RecoverIp => HOUR_MS,
            Rule::LoginUser | Rule::LoginIp | Rule::PasswordAccount => QUARTER_HOUR_MS,
            Rule::WritesAccount | Rule::UploadsAccount => DAY_MS,
        }
    }
}

/// An `accounts` row.
#[derive(Clone, Debug)]
struct Account {
    id: String,
    username: String,
    kdf: KdfParams,
    salt: String,
    auth_hash: String,
    wrapped_key: String,
    recovery_hash: String,
    recovery_wrapped_key: String,
    key_id: String,
    epoch: String,
    last_seq: i64,
    docs: u64,
    doc_bytes: u64,
    files: u64,
    file_bytes: u64,
    quota: QuotaOverride,
    created_at: i64,
    disabled_at: Option<i64>,
}

/// A `sessions` row.
#[derive(Clone, Debug)]
struct SessionRow {
    account_id: String,
    device_id: String,
    token_hash: String,
    proof_hash: String,
    sealed_name: Option<String>,
    created_at: i64,
    last_seen_at: i64,
    revoked_at: Option<i64>,
    revoked: Option<SignOutReason>,
}

/// An `account_docs` row.
#[derive(Clone, Debug)]
struct DocRow {
    rev: String,
    seq: i64,
    deleted: bool,
    sealed: String,
}

/// A `daily` row: a server-wide count for one UTC day.
#[derive(Clone, Copy, Debug, Default)]
struct Daily {
    day: i64,
    count: u64,
}

impl Daily {
    /// The count for the day of `now` (a later day starts from zero).
    fn today(self, now: i64) -> u64 {
        if self.day >= now.div_euclid(DAY_MS) {
            self.count
        } else {
            0
        }
    }

    fn add(&mut self, now: i64, amount: u64) {
        let day = now.div_euclid(DAY_MS);
        self.count = if self.day >= day {
            self.count + amount
        } else {
            amount
        };
        self.day = self.day.max(day);
    }
}

/// The session a request authenticated with.
struct Caller {
    account_id: String,
    device_id: String,
    token_hash: String,
}

/// A checked `device` object.
struct CheckedDevice {
    id: String,
    proof_hash: String,
    sealed_name: Option<String>,
}

type DocKey = (String, String);

struct ServerState {
    settings: BTreeMap<String, String>,
    /// `None`: the wall clock.
    now: Option<i64>,
    drops: BTreeMap<Route, usize>,
    /// Requests per route still to time out before they are carried out.
    losses: BTreeMap<Route, usize>,
    stats: ServerStats,
    floor: KdfParams,
    default: KdfParams,
    /// Stands in for `AUTH_PEPPER`: fake prelogin answers derive from it.
    pepper: [u8; 32],
    accounts: BTreeMap<String, Account>,
    sessions: Vec<SessionRow>,
    docs: BTreeMap<DocKey, DocRow>,
    files: BTreeMap<DocKey, Vec<u8>>,
    signups: Daily,
    writes: Daily,
    uploads: Daily,
    limits: BTreeMap<String, (i64, u64)>,
}

fn wall_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// Whole seconds until `ms` from now have passed, rounded up.
fn seconds_until(ms: i64) -> u64 {
    u64::try_from(ms.max(0)).unwrap_or(0).div_ceil(1000)
}

fn rate_limited(limit: &str, ms: i64) -> ApiError {
    ApiError::RateLimited {
        retry_after: seconds_until(ms),
        limit: limit.to_owned(),
    }
}

/// The milliseconds from `now` to the next UTC midnight.
fn until_midnight(now: i64) -> i64 {
    (now.div_euclid(DAY_MS) + 1) * DAY_MS - now
}

/// 32 bytes of canonical base64url (43 characters), as the Worker's `proof32`.
fn key32(text: &str) -> Option<Vec<u8>> {
    if text.len() != 43 {
        return None;
    }
    unb64(text).filter(|bytes| bytes.len() == 32)
}

fn is_salt(salt: &str) -> bool {
    salt.len() == 22 && is_base64url(salt)
}

fn is_wrap(wrap: &str) -> bool {
    wrap.len() == 98 && is_base64url(wrap)
}

fn signed_out(reason: SignOutReason) -> ApiError {
    ApiError::Unauthorized { reason }
}

fn new_token() -> (SessionToken, String) {
    let token = format!("kanso_{}", b64(&random::<32>()));
    let hash = sha256_hex(token.as_bytes());
    (SessionToken::new(token), hash)
}

impl ServerState {
    fn new(floor: KdfParams, default: KdfParams) -> ServerState {
        let mut settings: BTreeMap<String, String> = DEFAULT_SETTINGS
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        settings.insert("signup".into(), "open".into());
        ServerState {
            settings,
            now: None,
            drops: BTreeMap::new(),
            losses: BTreeMap::new(),
            stats: ServerStats::default(),
            floor,
            default,
            pepper: random::<32>(),
            accounts: BTreeMap::new(),
            sessions: Vec::new(),
            docs: BTreeMap::new(),
            files: BTreeMap::new(),
            signups: Daily::default(),
            writes: Daily::default(),
            uploads: Daily::default(),
            limits: BTreeMap::new(),
        }
    }

    fn clock(&self) -> i64 {
        self.now.unwrap_or_else(wall_ms)
    }

    /// A numeric setting; a missing or unreadable one is 0, as the
    /// triggers' `coalesce(…, 0)`.
    fn setting(&self, name: &str) -> u64 {
        self.settings
            .get(name)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0)
    }

    fn account_by_name(&self, username: &str) -> Option<&Account> {
        self.accounts.values().find(|a| a.username == username)
    }

    fn account_id_of(&self, username: &str) -> String {
        let username = normalize_username(username).unwrap_or_else(|_| username.to_owned());
        self.account_by_name(&username)
            .unwrap_or_else(|| panic!("MemoryServer has no account {username}"))
            .id
            .clone()
    }

    /// The counting upsert, then the comparison (H1): a refused request
    /// still counts.
    fn count(
        &mut self,
        rule: Rule,
        key: &str,
        now: i64,
        amount: u64,
        max: u64,
    ) -> Result<(), ApiError> {
        let window = rule.window();
        let start = now.div_euclid(window) * window;
        let row = self.limits.entry(key.to_owned()).or_insert((start, 0));
        if row.0 == start {
            row.1 += amount;
        } else {
            *row = (start, amount);
        }
        if row.1 > max {
            return Err(rate_limited(rule.name(), start + window - now));
        }
        Ok(())
    }

    fn quota_docs(&self, a: &Account) -> u64 {
        a.quota.docs.unwrap_or_else(|| self.setting("quota_docs"))
    }

    fn quota_doc_bytes(&self, a: &Account) -> u64 {
        a.quota
            .doc_bytes
            .unwrap_or_else(|| self.setting("quota_doc_bytes"))
    }

    fn quota_files(&self, a: &Account) -> u64 {
        a.quota.files.unwrap_or_else(|| self.setting("quota_files"))
    }

    fn quota_file_bytes(&self, a: &Account) -> u64 {
        a.quota
            .file_bytes
            .unwrap_or_else(|| self.setting("quota_file_bytes"))
    }

    fn quota_writes_per_day(&self, a: &Account) -> u64 {
        a.quota
            .writes_per_day
            .unwrap_or_else(|| self.setting("writes_per_day"))
    }

    fn check_kdf(&self, kdf: &WireKdf) -> Result<KdfParams, ApiError> {
        if kdf.alg != "argon2id" || kdf.version != 19 {
            return Err(ApiError::UnsupportedKdf);
        }
        let params = KdfParams {
            m_kib: kdf.m_kib,
            t: kdf.t,
            p: kdf.p,
        };
        let ceiling = KdfParams::CEILING;
        let floor = self.floor;
        if params.m_kib < floor.m_kib
            || params.t < floor.t
            || params.p < floor.p.max(1)
            || params.m_kib > ceiling.m_kib
            || params.t > ceiling.t
            || params.p > ceiling.p
        {
            return Err(ApiError::WeakKdf);
        }
        Ok(params)
    }

    // -- sessions ----------------------------------------------------------

    /// Authenticates a session route (docs/ACCOUNTS.md › Sessions in the
    /// Worker).
    fn authenticate(
        &mut self,
        token: &SessionToken,
        now: i64,
        allow_disabled: bool,
    ) -> Result<Caller, ApiError> {
        if !is_token(token.expose()) {
            return Err(signed_out(SignOutReason::Unknown));
        }
        let token_hash = sha256_hex(token.expose().as_bytes());
        let Some(row) = self
            .sessions
            .iter_mut()
            .find(|s| s.token_hash == token_hash)
        else {
            return Err(signed_out(SignOutReason::Unknown));
        };
        if let Some(reason) = row.revoked {
            return Err(signed_out(reason));
        }
        if now - row.last_seen_at > SESSION_IDLE_MS {
            row.revoked_at = Some(now);
            row.revoked = Some(SignOutReason::Expired);
            return Err(signed_out(SignOutReason::Expired));
        }
        let disabled = self
            .accounts
            .get(&row.account_id)
            .is_none_or(|a| a.disabled_at.is_some());
        if disabled && !allow_disabled {
            return Err(ApiError::AccountDisabled);
        }
        if now - row.last_seen_at > HOUR_MS {
            row.last_seen_at = now;
        }
        Ok(Caller {
            account_id: row.account_id.clone(),
            device_id: row.device_id.clone(),
            token_hash,
        })
    }

    fn revoke(row: &mut SessionRow, now: i64, reason: SignOutReason) {
        row.revoked_at = Some(now);
        row.revoked = Some(reason);
    }

    /// Makes `device` the account's active session: evicts the least
    /// recently seen other session when ten are active, upserts this one
    /// (keeping an earlier sealed name), and keeps 90 revoked rows.
    fn start_session(
        &mut self,
        account_id: &str,
        device: &CheckedDevice,
        now: i64,
    ) -> (SessionToken, Option<WireDevice>) {
        let others: Vec<usize> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.account_id == account_id && s.revoked.is_none() && s.device_id != device.id
            })
            .map(|(i, _)| i)
            .collect();
        let mut evicted = None;
        if others.len() >= MAX_ACTIVE_DEVICES
            && let Some(&oldest) = others.iter().min_by(|&&a, &&b| {
                let (a, b) = (&self.sessions[a], &self.sessions[b]);
                (a.last_seen_at, a.created_at, &a.device_id).cmp(&(
                    b.last_seen_at,
                    b.created_at,
                    &b.device_id,
                ))
            })
        {
            let row = &mut self.sessions[oldest];
            Self::revoke(row, now, SignOutReason::Evicted);
            evicted = Some(WireDevice {
                id: row.device_id.clone(),
                sealed_name: row.sealed_name.clone(),
            });
        }
        let (token, token_hash) = new_token();
        match self
            .sessions
            .iter_mut()
            .find(|s| s.account_id == account_id && s.device_id == device.id)
        {
            Some(row) => {
                row.token_hash = token_hash;
                row.proof_hash = device.proof_hash.clone();
                if device.sealed_name.is_some() {
                    row.sealed_name = device.sealed_name.clone();
                }
                row.created_at = now;
                row.last_seen_at = now;
                row.revoked_at = None;
                row.revoked = None;
            }
            None => self.sessions.push(SessionRow {
                account_id: account_id.to_owned(),
                device_id: device.id.clone(),
                token_hash,
                proof_hash: device.proof_hash.clone(),
                sealed_name: device.sealed_name.clone(),
                created_at: now,
                last_seen_at: now,
                revoked_at: None,
                revoked: None,
            }),
        }
        let mut revoked: Vec<(i64, String)> = self
            .sessions
            .iter()
            .filter(|s| s.account_id == account_id && s.revoked.is_some())
            .map(|s| (s.revoked_at.unwrap_or(0), s.device_id.clone()))
            .collect();
        if revoked.len() > MAX_REVOKED_ROWS {
            revoked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
            let gone: Vec<String> = revoked
                .split_off(MAX_REVOKED_ROWS)
                .into_iter()
                .map(|(_, id)| id)
                .collect();
            self.sessions.retain(|s| {
                !(s.account_id == account_id && s.revoked.is_some() && gone.contains(&s.device_id))
            });
        }
        (token, evicted)
    }

    fn check_device(device: &NewDevice) -> Result<CheckedDevice, ApiError> {
        if !is_device_id(&device.id) {
            return Err(ApiError::BadRequest);
        }
        let proof = key32(&device.proof).ok_or(ApiError::BadRequest)?;
        if let Some(name) = &device.sealed_name
            && !is_sealed_name(name)
        {
            return Err(ApiError::BadRequest);
        }
        Ok(CheckedDevice {
            id: device.id.clone(),
            proof_hash: sha256_hex(&proof),
            sealed_name: device.sealed_name.clone(),
        })
    }

    // -- routes ------------------------------------------------------------

    fn health(&self) -> Health {
        let signup = if self.settings.get("signup").map(String::as_str) == Some("open") {
            if self.accounts.len() as u64 >= self.setting("max_accounts") {
                SignupState::Full
            } else {
                SignupState::Open
            }
        } else {
            SignupState::Closed
        };
        Health {
            ok: true,
            protocol: PROTOCOL_VERSION,
            signup,
            auth: None,
        }
    }

    fn prelogin(&self, request: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        let username =
            normalize_username(&request.username).map_err(|_| ApiError::InvalidUsername)?;
        if let Some(a) = self.account_by_name(&username) {
            return Ok(PreloginResponse {
                kdf: a.kdf.into(),
                salt: a.salt.clone(),
                account_id: a.id.clone(),
            });
        }
        let fake = hkdf32(&self.pepper, None, b"kanso/v1/fake");
        let salt = hmac(&fake[..], &[b"salt", &[0], username.as_bytes()]);
        let account = hmac(&fake[..], &[b"account", &[0], username.as_bytes()]);
        Ok(PreloginResponse {
            kdf: self.default.into(),
            salt: b64(&salt[..16]),
            account_id: format!("a_{}", hex(&account[..16])),
        })
    }

    fn signup(&mut self, r: &SignupRequest, now: i64) -> Result<SessionResponse, ApiError> {
        self.count(
            Rule::SignupIp,
            &format!("signup-ip:{CLIENT_IP}"),
            now,
            1,
            10,
        )?;
        let username = match check_new_username(&r.username) {
            Ok(username) => username,
            Err(UsernameError::Reserved) => return Err(ApiError::ReservedUsername),
            Err(_) => return Err(ApiError::InvalidUsername),
        };
        if !is_account_id(&r.account_id) {
            return Err(ApiError::BadRequest);
        }
        let kdf = self.check_kdf(&r.kdf)?;
        if !is_salt(&r.salt) {
            return Err(ApiError::BadRequest);
        }
        let auth = key32(&r.auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key) || !is_hex16(&r.key_id) {
            return Err(ApiError::BadRequest);
        }
        let device = Self::check_device(&r.device)?;
        let auth_hash = sha256_hex(&auth);

        // A retried sign-up whose answer was lost (security re-check 7): the
        // same request again. The auth key alone would make sign-up a
        // password oracle outside the sign-in limits.
        if let Some(existing) = self.account_by_name(&username)
            && existing.auth_hash == auth_hash
            && existing.recovery_hash == sha256_hex(&recovery)
            && existing.id == r.account_id
            && existing.kdf == kdf
            && existing.salt == r.salt
            && existing.wrapped_key == r.wrapped_key
            && existing.recovery_wrapped_key == r.recovery_wrapped_key
            && existing.key_id == r.key_id
        {
            if existing.disabled_at.is_some() {
                return Err(ApiError::AccountDisabled);
            }
            let (account_id, epoch) = (existing.id.clone(), existing.epoch.clone());
            let (token, _) = self.start_session(&account_id, &device, now);
            return Ok(SessionResponse {
                account_id,
                username,
                token,
                epoch,
            });
        }

        // The sign-up triggers, before any UNIQUE check.
        let switch = self
            .settings
            .get("signup")
            .cloned()
            .unwrap_or_else(|| "closed".into());
        if switch != "open" && switch != format!("only:{username}") {
            return Err(ApiError::SignupClosed {
                reason: SignupClosedReason::Closed,
            });
        }
        if self.accounts.len() as u64 >= self.setting("max_accounts") {
            return Err(ApiError::SignupClosed {
                reason: SignupClosedReason::Full,
            });
        }
        if self.signups.today(now) >= self.setting("signups_per_day") {
            return Err(ApiError::SignupClosed {
                reason: SignupClosedReason::Busy,
            });
        }
        if self.accounts.contains_key(&r.account_id) {
            return Err(ApiError::AccountIdTaken);
        }
        if self.account_by_name(&username).is_some() {
            return Err(ApiError::UsernameTaken);
        }
        let epoch = hex(&random::<8>());
        self.accounts.insert(
            r.account_id.clone(),
            Account {
                id: r.account_id.clone(),
                username: username.clone(),
                kdf,
                salt: r.salt.clone(),
                auth_hash,
                wrapped_key: r.wrapped_key.clone(),
                recovery_hash: sha256_hex(&recovery),
                recovery_wrapped_key: r.recovery_wrapped_key.clone(),
                key_id: r.key_id.clone(),
                epoch: epoch.clone(),
                last_seq: 0,
                docs: 0,
                doc_bytes: 0,
                files: 0,
                file_bytes: 0,
                quota: QuotaOverride::default(),
                created_at: now,
                disabled_at: None,
            },
        );
        self.signups.add(now, 1);
        let (token, _) = self.start_session(&r.account_id, &device, now);
        Ok(SessionResponse {
            account_id: r.account_id.clone(),
            username,
            token,
            epoch,
        })
    }

    fn login(&mut self, r: &LoginRequest, now: i64) -> Result<LoginResponse, ApiError> {
        self.count(Rule::LoginIp, &format!("login-ip:{CLIENT_IP}"), now, 1, 30)?;
        let username = normalize_username(&r.username).map_err(|_| ApiError::InvalidUsername)?;
        let auth = key32(&r.auth_key).ok_or(ApiError::BadRequest)?;
        let device = Self::check_device(&NewDevice {
            id: r.device.id.clone(),
            sealed_name: None,
            proof: r.device.proof.clone(),
        })?;
        let account = self.account_by_name(&username).cloned();
        let known = account.as_ref().is_some_and(|a| {
            self.sessions.iter().any(|s| {
                s.account_id == a.id
                    && s.device_id == device.id
                    && s.revoked_at.is_none()
                    && s.proof_hash == device.proof_hash
            })
        });
        if !known {
            let key = account.as_ref().map_or_else(
                || "login-user:unknown".to_owned(),
                |a| format!("login-user:{}", a.id),
            );
            self.count(Rule::LoginUser, &key, now, 1, 10)?;
        }
        let Some(account) = account.filter(|a| a.auth_hash == sha256_hex(&auth)) else {
            return Err(ApiError::InvalidCredentials);
        };
        if account.disabled_at.is_some() {
            return Err(ApiError::AccountDisabled);
        }
        let (token, evicted_device) = self.start_session(&account.id, &device, now);
        Ok(LoginResponse {
            account_id: account.id,
            username: account.username,
            token,
            wrapped_key: account.wrapped_key,
            key_id: account.key_id,
            epoch: account.epoch,
            evicted_device,
        })
    }

    fn recover_start(
        &mut self,
        r: &RecoverStartRequest,
        now: i64,
    ) -> Result<RecoverStartResponse, ApiError> {
        self.count(
            Rule::RecoverIp,
            &format!("recover-ip:{CLIENT_IP}"),
            now,
            1,
            10,
        )?;
        let username = normalize_username(&r.username).map_err(|_| ApiError::InvalidUsername)?;
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        let Some(a) = self
            .account_by_name(&username)
            .filter(|a| a.recovery_hash == sha256_hex(&recovery))
        else {
            return Err(ApiError::InvalidCredentials);
        };
        if a.disabled_at.is_some() {
            return Err(ApiError::AccountDisabled);
        }
        Ok(RecoverStartResponse {
            account_id: a.id.clone(),
            recovery_wrapped_key: a.recovery_wrapped_key.clone(),
            key_id: a.key_id.clone(),
        })
    }

    fn recover_finish(
        &mut self,
        r: &RecoverFinishRequest,
        now: i64,
    ) -> Result<SessionResponse, ApiError> {
        self.count(
            Rule::RecoverIp,
            &format!("recover-ip:{CLIENT_IP}"),
            now,
            1,
            10,
        )?;
        let username = normalize_username(&r.username).map_err(|_| ApiError::InvalidUsername)?;
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        let kdf = self.check_kdf(&r.kdf)?;
        if !is_salt(&r.salt) {
            return Err(ApiError::BadRequest);
        }
        let auth = key32(&r.auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let recovery_new = key32(&r.recovery_auth_key_new).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key_new) {
            return Err(ApiError::BadRequest);
        }
        let device = Self::check_device(&r.device)?;
        let auth_hash = sha256_hex(&auth);
        let recovery_hash_new = sha256_hex(&recovery_new);

        let account = self.account_by_name(&username).cloned();
        if let Some(a) = account
            .as_ref()
            .filter(|a| a.recovery_hash == sha256_hex(&recovery))
        {
            if a.disabled_at.is_some() {
                return Err(ApiError::AccountDisabled);
            }
            for row in self
                .sessions
                .iter_mut()
                .filter(|s| s.account_id == a.id && s.revoked.is_none())
            {
                Self::revoke(row, now, SignOutReason::Recovered);
            }
            let (token, _) = self.start_session(&a.id, &device, now);
            let stored = self.accounts.get_mut(&a.id).expect("the account just read");
            stored.kdf = kdf;
            stored.salt = r.salt.clone();
            stored.auth_hash = auth_hash;
            stored.wrapped_key = r.wrapped_key.clone();
            stored.recovery_hash = recovery_hash_new;
            stored.recovery_wrapped_key = r.recovery_wrapped_key_new.clone();
            return Ok(SessionResponse {
                account_id: a.id.clone(),
                username: a.username.clone(),
                token,
                epoch: a.epoch.clone(),
            });
        }
        // A retry after a lost answer: the new keys are already in place.
        if let Some(a) =
            account.filter(|a| a.recovery_hash == recovery_hash_new && a.auth_hash == auth_hash)
        {
            if a.disabled_at.is_some() {
                return Err(ApiError::AccountDisabled);
            }
            let (token, _) = self.start_session(&a.id, &device, now);
            return Ok(SessionResponse {
                account_id: a.id,
                username: a.username,
                token,
                epoch: a.epoch,
            });
        }
        Err(ApiError::InvalidCredentials)
    }

    fn logout(&mut self, token: &SessionToken, now: i64) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, true)?;
        self.revoke_self(&caller, now);
        Ok(())
    }

    fn revoke_self(&mut self, caller: &Caller, now: i64) {
        if let Some(row) = self
            .sessions
            .iter_mut()
            .find(|s| s.token_hash == caller.token_hash && s.revoked.is_none())
        {
            Self::revoke(row, now, SignOutReason::SignedOut);
        }
    }

    fn account(&mut self, token: &SessionToken, now: i64) -> Result<AccountResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let a = &self.accounts[&caller.account_id];
        let day = now.div_euclid(DAY_MS) * DAY_MS;
        let writes_today = self
            .limits
            .get(&format!("writes-account:{}", a.id))
            .filter(|(start, _)| *start == day)
            .map_or(0, |(_, count)| *count);
        Ok(AccountResponse {
            account_id: a.id.clone(),
            username: a.username.clone(),
            created_at: a.created_at,
            usage: WireUsage {
                docs: a.docs,
                doc_bytes: a.doc_bytes,
                files: a.files,
                file_bytes: a.file_bytes,
                writes_today,
            },
            quota: WireQuota {
                docs: self.quota_docs(a),
                doc_bytes: self.quota_doc_bytes(a),
                files: self.quota_files(a),
                file_bytes: self.quota_file_bytes(a),
                writes_per_day: self.quota_writes_per_day(a),
            },
        })
    }

    /// Counts `password-account` and checks the current key's shape.
    fn current_password(
        &mut self,
        caller: &Caller,
        key: &str,
        now: i64,
    ) -> Result<String, ApiError> {
        self.count(
            Rule::PasswordAccount,
            &format!("password-account:{}", caller.account_id),
            now,
            1,
            5,
        )?;
        let current = key32(key).ok_or(ApiError::BadRequest)?;
        Ok(sha256_hex(&current))
    }

    fn change_password(
        &mut self,
        token: &SessionToken,
        r: &PasswordRequest,
        now: i64,
    ) -> Result<PasswordResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let current = self.current_password(&caller, &r.current_auth_key, now)?;
        let kdf = self.check_kdf(&r.kdf)?;
        if !is_salt(&r.salt) {
            return Err(ApiError::BadRequest);
        }
        let auth = key32(&r.auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let auth_hash = sha256_hex(&auth);
        let recovery_hash = sha256_hex(&recovery);
        let account = &self.accounts[&caller.account_id];
        if account.auth_hash == current {
            let mut signed_out_devices = 0;
            for row in self.sessions.iter_mut().filter(|s| {
                s.account_id == caller.account_id
                    && s.revoked.is_none()
                    && s.token_hash != caller.token_hash
            }) {
                Self::revoke(row, now, SignOutReason::PasswordChanged);
                signed_out_devices += 1;
            }
            let stored = self
                .accounts
                .get_mut(&caller.account_id)
                .expect("authenticated");
            stored.kdf = kdf;
            stored.salt = r.salt.clone();
            stored.auth_hash = auth_hash;
            stored.wrapped_key = r.wrapped_key.clone();
            stored.recovery_hash = recovery_hash;
            stored.recovery_wrapped_key = r.recovery_wrapped_key.clone();
            return Ok(PasswordResponse { signed_out_devices });
        }
        // A retry whose answer was lost: everything it sets is set already.
        if account.auth_hash == auth_hash
            && account.wrapped_key == r.wrapped_key
            && account.recovery_hash == recovery_hash
            && account.recovery_wrapped_key == r.recovery_wrapped_key
        {
            return Ok(PasswordResponse {
                signed_out_devices: 0,
            });
        }
        Err(ApiError::InvalidCredentials)
    }

    fn replace_recovery_key(
        &mut self,
        token: &SessionToken,
        r: &RecoveryKeyRequest,
        now: i64,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let current = self.current_password(&caller, &r.current_auth_key, now)?;
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let stored = self
            .accounts
            .get_mut(&caller.account_id)
            .expect("authenticated");
        if stored.auth_hash != current {
            return Err(ApiError::InvalidCredentials);
        }
        stored.recovery_hash = sha256_hex(&recovery);
        stored.recovery_wrapped_key = r.recovery_wrapped_key.clone();
        Ok(RecoveryKeyResponse {})
    }

    fn delete_account(
        &mut self,
        token: &SessionToken,
        r: &DeleteRequest,
        now: i64,
    ) -> Result<DeleteResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let current = self.current_password(&caller, &r.auth_key, now)?;
        let account = &self.accounts[&caller.account_id];
        if account.auth_hash != current {
            return Err(ApiError::InvalidCredentials);
        }
        if normalize_username(&r.username).ok().as_deref() != Some(account.username.as_str()) {
            return Err(ApiError::UsernameMismatch);
        }
        let id = caller.account_id;
        self.docs.retain(|(account, _), _| *account != id);
        self.files.retain(|(account, _), _| *account != id);
        self.sessions.retain(|s| s.account_id != id);
        for rule in [
            "login-user",
            "password-account",
            "writes-account",
            "uploads-account",
        ] {
            self.limits.remove(&format!("{rule}:{id}"));
        }
        self.accounts.remove(&id);
        Ok(DeleteResponse { deleted: true })
    }

    fn set_device_name(
        &mut self,
        token: &SessionToken,
        r: &DeviceNameRequest,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_sealed_name(&r.sealed_name) {
            return Err(ApiError::BadRequest);
        }
        if let Some(row) = self
            .sessions
            .iter_mut()
            .find(|s| s.token_hash == caller.token_hash && s.revoked.is_none())
        {
            row.sealed_name = Some(r.sealed_name.clone());
        }
        Ok(())
    }

    fn devices(&mut self, token: &SessionToken, now: i64) -> Result<DevicesResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let mut rows: Vec<&SessionRow> = self
            .sessions
            .iter()
            .filter(|s| s.account_id == caller.account_id && s.revoked.is_none())
            .collect();
        rows.sort_by(|a, b| {
            b.last_seen_at
                .cmp(&a.last_seen_at)
                .then_with(|| b.created_at.cmp(&a.created_at))
                .then_with(|| a.device_id.cmp(&b.device_id))
        });
        Ok(DevicesResponse {
            devices: rows
                .into_iter()
                .map(|s| DeviceRow {
                    id: s.device_id.clone(),
                    sealed_name: s.sealed_name.clone(),
                    created_at: s.created_at,
                    last_seen_at: s.last_seen_at,
                    current: s.device_id == caller.device_id,
                })
                .collect(),
        })
    }

    fn revoke_device(
        &mut self,
        token: &SessionToken,
        device_id: &str,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_device_id(device_id) {
            return Err(ApiError::BadRequest);
        }
        if device_id == caller.device_id {
            self.revoke_self(&caller, now);
            return Ok(());
        }
        // A row already signed out, for any reason, is marked `revoked`
        // too, keeping its time; 404 only when the account has no row.
        let mut found = false;
        for row in self
            .sessions
            .iter_mut()
            .filter(|s| s.account_id == caller.account_id && s.device_id == device_id)
        {
            row.revoked_at = Some(row.revoked_at.unwrap_or(now));
            row.revoked = Some(SignOutReason::Revoked);
            found = true;
        }
        if found {
            Ok(())
        } else {
            Err(ApiError::NotFound)
        }
    }

    /// 413 for a refused batch: the account's usage now (the batch rolled
    /// back) and the quota or cap.
    fn quota_exceeded(&self, account_id: &str, quota: QuotaKind) -> ApiError {
        let a = &self.accounts[account_id];
        let (used, limit) = match quota {
            QuotaKind::Docs => (a.docs, self.quota_docs(a)),
            QuotaKind::DocBytes => (a.doc_bytes, self.quota_doc_bytes(a)),
            QuotaKind::Files => (a.files, self.quota_files(a)),
            QuotaKind::FileBytes => (a.file_bytes, self.quota_file_bytes(a)),
            QuotaKind::Server => unreachable!("the caller picks the family"),
        };
        ApiError::QuotaExceeded { quota, used, limit }
    }

    fn server_exceeded(&self, files: bool) -> ApiError {
        let (used, limit) = if files {
            (
                self.accounts.values().map(|a| a.file_bytes).sum(),
                self.setting("global_file_bytes"),
            )
        } else {
            (
                self.accounts.values().map(|a| a.doc_bytes).sum(),
                self.setting("global_doc_bytes"),
            )
        };
        ApiError::QuotaExceeded {
            quota: QuotaKind::Server,
            used,
            limit,
        }
    }

    /// The push body's shape and sizes, checked before anything is stored.
    fn check_push(r: &PushRequest) -> Result<(), ApiError> {
        let body = serde_json::to_vec(r).map_err(|e| ApiError::Protocol(e.to_string()))?;
        if body.len() > MAX_PUSH_BODY || r.docs.len() > MAX_PUSH_DOCS {
            return Err(ApiError::TooLarge);
        }
        for doc in &r.docs {
            if !is_wire_id(&doc.id) || !is_wire_rev(&doc.rev) || doc.sealed.len() < MIN_SEALED_LEN {
                return Err(ApiError::BadRequest);
            }
            if doc.sealed.len() > MAX_SEALED_LEN {
                return Err(ApiError::TooLarge);
            }
            if doc.base_rev.as_deref().is_some_and(|b| !is_wire_rev(b)) {
                return Err(ApiError::BadRequest);
            }
        }
        Ok(())
    }

    /// Whether the account's own count today under `key` (this request
    /// included) is above its fair share of the server-wide daily cap
    /// `cap` (cap / max_accounts). Fails closed like the trigger: no row for
    /// today, or `max_accounts` 0, counts as above.
    fn above_fair_share(&self, key: &str, now: i64, cap: &str) -> bool {
        let max_accounts = self.setting("max_accounts");
        let today = now.div_euclid(DAY_MS) * DAY_MS;
        match self.limits.get(key) {
            Some(&(start, count)) if max_accounts > 0 && start == today => {
                count > self.setting(cap) / max_accounts
            }
            _ => true,
        }
    }

    fn push(
        &mut self,
        token: &SessionToken,
        r: &PushRequest,
        now: i64,
    ) -> Result<PushResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if let Err(refused) = Self::check_push(r) {
            // A refused body still cost a read and a parse: one write.
            let id = &caller.account_id;
            self.count(
                Rule::WritesAccount,
                &format!("writes-account:{id}"),
                now,
                1,
                self.quota_writes_per_day(&self.accounts[id]),
            )?;
            return Err(refused);
        }
        if r.docs.is_empty() {
            return Ok(PushResponse { results: vec![] });
        }
        let id = caller.account_id;
        self.count(
            Rule::WritesAccount,
            &format!("writes-account:{id}"),
            now,
            r.docs.len() as u64,
            self.quota_writes_per_day(&self.accounts[&id]),
        )?;
        self.stats.pushed_docs += r.docs.len();

        // One batch: on any refusal everything it wrote is undone.
        let saved_account = self.accounts[&id].clone();
        let saved_writes = self.writes;
        let mut undo: Vec<(DocKey, Option<DocRow>)> = Vec::new();
        let mut results = Vec::with_capacity(r.docs.len());
        let mut refusal = None;
        for doc in &r.docs {
            let key = (id.clone(), doc.id.clone());
            let existing = self.docs.get(&key).cloned();
            let accept = match (&doc.base_rev, &existing) {
                (None, None) => true,
                (Some(base), Some(current)) => current.rev == *base,
                _ => false,
            };
            if !accept {
                results.push(match existing {
                    Some(current) => WirePushResult::Conflict {
                        id: doc.id.clone(),
                        current: WireDoc {
                            id: doc.id.clone(),
                            rev: current.rev,
                            deleted: current.deleted,
                            sealed: current.sealed,
                        },
                    },
                    None => WirePushResult::Lost { id: doc.id.clone() },
                });
                continue;
            }
            // BUMP and its triggers.
            let a = &self.accounts[&id];
            let (old_docs, old_bytes) = (a.docs as i64, a.doc_bytes as i64);
            let new_docs = old_docs + i64::from(!doc.deleted)
                - existing.as_ref().map_or(0, |e| i64::from(!e.deleted));
            let new_bytes = old_bytes + (doc.sealed.len() + DOC_ROW_COST as usize) as i64
                - existing
                    .as_ref()
                    .map_or(0, |e| (e.sealed.len() + DOC_ROW_COST as usize) as i64);
            let over_docs = new_docs > old_docs && new_docs > self.quota_docs(a) as i64;
            let grows = new_bytes > old_bytes && !over_docs;
            let over_bytes = grows && new_bytes > self.quota_doc_bytes(a) as i64;
            let all: i64 = self.accounts.values().map(|a| a.doc_bytes as i64).sum();
            let over_server = grows
                && !over_bytes
                && all - old_bytes + new_bytes > self.setting("global_doc_bytes") as i64;
            if over_docs || over_bytes || over_server {
                refusal = Some(if over_docs {
                    QuotaKind::Docs
                } else if over_bytes {
                    QuotaKind::DocBytes
                } else {
                    QuotaKind::Server
                });
                break;
            }
            // WRITE: the global daily cap, then the sealed alphabet CHECK.
            if self.writes.today(now) >= self.setting("global_writes_per_day")
                && self.above_fair_share(
                    &format!("writes-account:{id}"),
                    now,
                    "global_writes_per_day",
                )
            {
                self.undo_push(&id, saved_account, saved_writes, undo);
                return Err(rate_limited("writes-global", until_midnight(now)));
            }
            if !is_base64url(&doc.sealed) {
                self.undo_push(&id, saved_account, saved_writes, undo);
                return Err(ApiError::BadRequest);
            }
            let a = self.accounts.get_mut(&id).expect("authenticated");
            a.last_seq += 1;
            a.docs = new_docs as u64;
            a.doc_bytes = new_bytes as u64;
            let seq = a.last_seq;
            self.writes.add(now, 1);
            undo.push((key.clone(), existing));
            self.docs.insert(
                key,
                DocRow {
                    rev: doc.rev.clone(),
                    seq,
                    deleted: doc.deleted,
                    sealed: doc.sealed.clone(),
                },
            );
            results.push(WirePushResult::Ok {
                id: doc.id.clone(),
                seq,
            });
        }
        if let Some(quota) = refusal {
            self.undo_push(&id, saved_account, saved_writes, undo);
            return Err(match quota {
                QuotaKind::Server => self.server_exceeded(false),
                quota => self.quota_exceeded(&id, quota),
            });
        }

        // Conflicts share the response's budget of sealed text.
        let mut budget = PAGE_SEALED_BUDGET;
        for result in &mut results {
            if let WirePushResult::Conflict { id, current } = result {
                if current.sealed.len() > budget {
                    *result = WirePushResult::Behind { id: id.clone() };
                } else {
                    budget -= current.sealed.len();
                }
            }
        }

        // Accepted file tombstones drop their bytes and charges.
        let mut last: BTreeMap<&str, bool> = BTreeMap::new();
        for (doc, result) in r.docs.iter().zip(&results) {
            if matches!(result, WirePushResult::Ok { .. }) && doc.id.starts_with("file:") {
                last.insert(&doc.id, doc.deleted);
            }
        }
        for (file, deleted) in last {
            if deleted && let Some(bytes) = self.files.remove(&(id.clone(), file.to_owned())) {
                let a = self.accounts.get_mut(&id).expect("authenticated");
                a.files -= 1;
                a.file_bytes -= bytes.len() as u64 + FILE_ROW_COST;
            }
        }
        Ok(PushResponse { results })
    }

    fn undo_push(
        &mut self,
        id: &str,
        account: Account,
        writes: Daily,
        undo: Vec<(DocKey, Option<DocRow>)>,
    ) {
        for (key, old) in undo.into_iter().rev() {
            match old {
                Some(row) => {
                    self.docs.insert(key, row);
                }
                None => {
                    self.docs.remove(&key);
                }
            }
        }
        self.accounts.insert(id.to_owned(), account);
        self.writes = writes;
    }

    fn changes(
        &mut self,
        token: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
        now: i64,
    ) -> Result<WirePage, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let since = since.unwrap_or(0);
        let limit = limit.unwrap_or(DEFAULT_CHANGES_LIMIT);
        if since < 0 || !(1..=MAX_WIRE_CHANGES_LIMIT).contains(&limit) {
            return Err(ApiError::BadRequest);
        }
        let mut rows: Vec<(&String, &DocRow)> = self
            .docs
            .iter()
            .filter(|((account, _), row)| *account == caller.account_id && row.seq > since)
            .map(|((_, id), row)| (id, row))
            .collect();
        rows.sort_by_key(|(_, row)| row.seq);
        rows.truncate(limit);
        let mut cut = 0;
        let mut total = 0;
        while cut < rows.len()
            && (cut == 0 || total + rows[cut].1.sealed.len() <= PAGE_SEALED_BUDGET)
        {
            total += rows[cut].1.sealed.len();
            cut += 1;
        }
        let more = cut < rows.len() || rows.len() == limit;
        let docs: Vec<WireChange> = rows[..cut]
            .iter()
            .map(|(id, row)| WireChange {
                id: (*id).clone(),
                rev: row.rev.clone(),
                deleted: row.deleted,
                sealed: row.sealed.clone(),
                seq: row.seq,
            })
            .collect();
        let a = &self.accounts[&caller.account_id];
        Ok(WirePage {
            last_seq: docs.last().map_or(since, |d| d.seq),
            docs,
            more,
            head_seq: a.last_seq,
            epoch: a.epoch.clone(),
            key_id: a.key_id.clone(),
        })
    }

    fn put_file(
        &mut self,
        token: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
        now: i64,
    ) -> Result<PutFileResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_wire_file_id(wire_id) {
            return Err(ApiError::BadRequest);
        }
        let id = caller.account_id;
        self.count(
            Rule::WritesAccount,
            &format!("writes-account:{id}"),
            now,
            1,
            self.quota_writes_per_day(&self.accounts[&id]),
        )?;
        // The Worker counts Content-Length before it reads the body; a body
        // over the largest file is refused anyway, so it counts one more.
        self.count(
            Rule::UploadsAccount,
            &format!("uploads-account:{id}"),
            now,
            sealed.len().min(MAX_SEALED_FILE + 1) as u64,
            self.setting("upload_bytes_per_day"),
        )?;
        if sealed.len() > MAX_SEALED_FILE {
            return Err(ApiError::TooLarge);
        }
        if sealed.len() < MIN_SEALED_FILE {
            return Err(ApiError::BadRequest);
        }
        let size = sealed.len() as i64;
        let key = (id.clone(), wire_id.to_owned());
        let old = self.files.get(&key).map(|bytes| bytes.len() as i64);
        let a = &self.accounts[&id];
        let (old_files, old_bytes) = (a.files as i64, a.file_bytes as i64);
        let new_files = old_files + i64::from(old.is_none());
        let new_bytes =
            old_bytes + size + FILE_ROW_COST as i64 - old.map_or(0, |o| o + FILE_ROW_COST as i64);
        let over_files = new_files > old_files && new_files > self.quota_files(a) as i64;
        let grows = new_bytes > old_bytes && !over_files;
        let over_bytes = grows && new_bytes > self.quota_file_bytes(a) as i64;
        let all: i64 = self.accounts.values().map(|a| a.file_bytes as i64).sum();
        let over_server = grows
            && !over_bytes
            && all - old_bytes + new_bytes > self.setting("global_file_bytes") as i64;
        if over_files {
            return Err(self.quota_exceeded(&id, QuotaKind::Files));
        }
        if over_bytes {
            return Err(self.quota_exceeded(&id, QuotaKind::FileBytes));
        }
        if over_server {
            return Err(self.server_exceeded(true));
        }
        if self.uploads.today(now) as i64
            > self.setting("global_upload_bytes_per_day") as i64 - size
            && self.above_fair_share(
                &format!("uploads-account:{id}"),
                now,
                "global_upload_bytes_per_day",
            )
        {
            return Err(rate_limited("uploads-global", until_midnight(now)));
        }
        let a = self.accounts.get_mut(&id).expect("authenticated");
        a.files = new_files as u64;
        a.file_bytes = new_bytes as u64;
        self.uploads.add(now, size as u64);
        self.files.insert(key, sealed.to_vec());
        Ok(PutFileResponse {
            ok: true,
            size: size as u64,
        })
    }

    fn get_file(
        &mut self,
        token: &SessionToken,
        wire_id: &str,
        now: i64,
    ) -> Result<Option<Vec<u8>>, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_wire_file_id(wire_id) {
            return Err(ApiError::BadRequest);
        }
        Ok(self
            .files
            .get(&(caller.account_id, wire_id.to_owned()))
            .cloned())
    }
}

/// The test server: implements [`Backend`] with the Worker's sync, auth,
/// session, quota and limit rules (the migration's triggers included), in
/// memory, and runs the shared fixtures. Its auth hashes are SHA-256, as in
/// the Worker; its KDF floor and default are [`KdfParams::TEST`]. Tests and
/// snapshots only; `main.rs` never constructs it.
pub struct MemoryServer {
    /// Shared with the [`PlainView`]s it hands out.
    state: Arc<Mutex<ServerState>>,
}

impl Default for MemoryServer {
    fn default() -> Self {
        MemoryServer::new()
    }
}

impl MemoryServer {
    /// An empty server with the migration's settings, except that sign-ups
    /// are open, on the wall clock.
    pub fn new() -> MemoryServer {
        MemoryServer::with_kdf(KdfParams::TEST, KdfParams::TEST)
    }

    /// Like [`MemoryServer::new`], with another KDF floor and default. The
    /// conformance runner uses the production pair, which the shared
    /// fixtures assume (`weak_kdf` cases, fake prelogin answers); nothing
    /// derives keys against it.
    pub fn with_kdf(floor: KdfParams, default: KdfParams) -> MemoryServer {
        MemoryServer {
            state: Arc::new(Mutex::new(ServerState::new(floor, default))),
        }
    }

    fn server(&self) -> std::sync::MutexGuard<'_, ServerState> {
        self.state.lock().expect("memory server poisoned")
    }

    /// Pins the server's clock (ms). Fixture steps set it to `start + t`.
    pub fn set_now(&self, now_ms: i64) {
        self.server().now = Some(now_ms);
    }

    /// Writes one `settings` row, as `UPDATE settings SET value = ? WHERE
    /// name = ?` would. Panics when D1's `CHECK` would refuse it.
    pub fn set_setting(&self, name: &str, value: &str) {
        assert!(
            is_valid_setting(name, value),
            "D1 refuses settings row {name} = {value}"
        );
        self.server()
            .settings
            .insert(name.to_owned(), value.to_owned());
    }

    pub fn set_signup(&self, signup: Signup) {
        let value = match signup {
            Signup::Open => "open".to_owned(),
            Signup::Closed => "closed".to_owned(),
            Signup::Only(name) => format!("only:{name}"),
        };
        self.set_setting("signup", &value);
    }

    pub fn set_max_accounts(&self, n: u64) {
        self.set_setting("max_accounts", &n.to_string());
    }

    pub fn set_default_quota(&self, limits: Limits) {
        self.set_setting("quota_docs", &limits.docs.to_string());
        self.set_setting("quota_doc_bytes", &limits.doc_bytes.to_string());
        self.set_setting("quota_files", &limits.files.to_string());
        self.set_setting("quota_file_bytes", &limits.file_bytes.to_string());
    }

    pub fn set_writes_per_day(&self, n: u64) {
        self.set_setting("writes_per_day", &n.to_string());
    }

    pub fn set_upload_bytes_per_day(&self, n: u64) {
        self.set_setting("upload_bytes_per_day", &n.to_string());
    }

    /// Sets an account's quota overrides (all five columns). Panics for an
    /// unknown username, as every admin control here does.
    pub fn set_quota(&self, username: &str, quota: QuotaOverride) {
        let mut state = self.server();
        let id = state.account_id_of(username);
        state.accounts.get_mut(&id).expect("found").quota = quota;
    }

    /// An account's quota overrides (additive: the fixtures' `admin quota`
    /// step changes only the columns it names).
    pub fn quota(&self, username: &str) -> QuotaOverride {
        let state = self.server();
        let id = state.account_id_of(username);
        state.accounts[&id].quota
    }

    /// The account ID of a username (additive, for tests).
    pub fn account_id(&self, username: &str) -> Option<String> {
        let state = self.server();
        let username = normalize_username(username).ok()?;
        state.account_by_name(&username).map(|a| a.id.clone())
    }

    /// Sets `disabled_at`, like `admin disable`.
    pub fn disable(&self, username: &str) {
        let mut state = self.server();
        let now = state.clock();
        let id = state.account_id_of(username);
        state.accounts.get_mut(&id).expect("found").disabled_at = Some(now);
    }

    /// Clears `disabled_at`.
    pub fn enable(&self, username: &str) {
        let mut state = self.server();
        let id = state.account_id_of(username);
        state.accounts.get_mut(&id).expect("found").disabled_at = None;
    }

    /// Every session row of the account, active and revoked.
    pub fn devices(&self, username: &str) -> Vec<ServerDevice> {
        let state = self.server();
        let id = state.account_id_of(username);
        state
            .sessions
            .iter()
            .filter(|s| s.account_id == id)
            .map(|s| ServerDevice {
                id: s.device_id.clone(),
                sealed_name: s.sealed_name.clone(),
                created_at: s.created_at,
                last_seen_at: s.last_seen_at,
                revoked: s.revoked,
            })
            .collect()
    }

    /// Revokes every active session of the account with `reason`.
    pub fn revoke_all(&self, username: &str, reason: SignOutReason) {
        let mut state = self.server();
        let now = state.clock();
        let id = state.account_id_of(username);
        for row in state
            .sessions
            .iter_mut()
            .filter(|s| s.account_id == id && s.revoked.is_none())
        {
            ServerState::revoke(row, now, reason);
        }
    }

    /// The cron's expiry: revokes (`expired`) active sessions unseen for at
    /// least `unseen_for`.
    pub fn expire_sessions(&self, unseen_for: Duration) {
        let mut state = self.server();
        let now = state.clock();
        let unseen = i64::try_from(unseen_for.as_millis()).unwrap_or(i64::MAX);
        for row in state
            .sessions
            .iter_mut()
            .filter(|s| s.revoked.is_none() && now - s.last_seen_at >= unseen)
        {
            ServerState::revoke(row, now, SignOutReason::Expired);
        }
    }

    /// A new random epoch for the account, like `admin restored` (M1).
    pub fn set_epoch(&self, username: &str) {
        let mut state = self.server();
        let id = state.account_id_of(username);
        state.accounts.get_mut(&id).expect("found").epoch = hex(&random::<8>());
    }

    /// Rewinds the account as a restore would: writes after `to_seq` are
    /// gone (documents last written after it disappear) and `last_seq`
    /// becomes `to_seq`; the epoch stays, so clients see `head_seq` drop.
    pub fn rewind(&self, username: &str, to_seq: i64) {
        let mut state = self.server();
        let id = state.account_id_of(username);
        state
            .docs
            .retain(|(account, _), row| *account != id || row.seq <= to_seq);
        let (docs, doc_bytes) = state
            .docs
            .iter()
            .filter(|((account, _), _)| *account == id)
            .fold((0, 0), |(docs, bytes), (_, row)| {
                (
                    docs + u64::from(!row.deleted),
                    bytes + row.sealed.len() as u64 + DOC_ROW_COST,
                )
            });
        let a = state.accounts.get_mut(&id).expect("found");
        a.last_seq = to_seq.min(a.last_seq);
        a.docs = docs;
        a.doc_bytes = doc_bytes;
    }

    /// A copy of one account's documents, files and counters, for
    /// [`MemoryServer::restore`].
    pub fn snapshot(&self, username: &str) -> AccountSnapshot {
        let state = self.server();
        let id = state.account_id_of(username);
        let account = &state.accounts[&id];
        AccountSnapshot {
            docs: state
                .docs
                .iter()
                .filter(|((account, _), _)| *account == id)
                .map(|((_, wire_id), row)| (wire_id.clone(), row.clone()))
                .collect(),
            files: state
                .files
                .iter()
                .filter(|((account, _), _)| *account == id)
                .map(|((_, wire_id), bytes)| (wire_id.clone(), bytes.clone()))
                .collect(),
            last_seq: account.last_seq,
            counters: [
                account.docs,
                account.doc_bytes,
                account.files,
                account.file_bytes,
            ],
        }
    }

    /// Puts the account's documents, files, counters and `last_seq` back as
    /// `snapshot` holds them and gives it a new epoch: a D1 Time Travel
    /// restore followed by `admin restored` (M1). Sessions and credentials
    /// stay.
    pub fn restore(&self, username: &str, snapshot: &AccountSnapshot) {
        let mut state = self.server();
        let id = state.account_id_of(username);
        state.docs.retain(|(account, _), _| *account != id);
        state.files.retain(|(account, _), _| *account != id);
        for (wire_id, row) in &snapshot.docs {
            state
                .docs
                .insert((id.clone(), wire_id.clone()), row.clone());
        }
        for (wire_id, bytes) in &snapshot.files {
            state
                .files
                .insert((id.clone(), wire_id.clone()), bytes.clone());
        }
        let a = state.accounts.get_mut(&id).expect("found");
        a.last_seq = snapshot.last_seq;
        [a.docs, a.doc_bytes, a.files, a.file_bytes] = snapshot.counters;
        a.epoch = hex(&random::<8>());
    }

    /// The next `n` requests to `route` are applied but answer with a
    /// transport error, as when the connection drops after the server
    /// committed.
    pub fn drop_next_responses(&self, route: Route, n: usize) {
        self.server().drops.insert(route, n);
    }

    /// The next `n` requests to `route` time out before the server carries
    /// them out: a transport error after sending, as for a lost answer, but
    /// nothing changed (review-verify 6).
    pub fn lose_next_requests(&self, route: Route, n: usize) {
        self.server().losses.insert(route, n);
    }

    pub fn stats(&self) -> ServerStats {
        self.server().stats.clone()
    }

    /// The account of `session`, decrypted with its keys.
    pub fn plain_view(&self, session: &Session) -> PlainView {
        PlainView::new(
            AccountHandle {
                state: Arc::clone(&self.state),
                account_id: session.account_id.clone(),
                token: session.token().clone(),
            },
            session,
        )
    }

    /// Runs one request: counts it, applies it, then answers with a
    /// transport error instead if a drop is pending for its route.
    fn call<T>(
        &self,
        route: Route,
        handle: impl FnOnce(&mut ServerState, i64) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        let mut state = self.server();
        if let Some(n) = state.losses.get_mut(&route)
            && *n > 0
        {
            *n -= 1;
            return Err(ApiError::Transport("timed out (simulated)".into()));
        }
        let now = state.clock();
        *state.stats.requests.entry(route).or_default() += 1;
        let result = handle(&mut state, now);
        if let Some(n) = state.drops.get_mut(&route)
            && *n > 0
        {
            *n -= 1;
            return Err(ApiError::Transport("connection reset (simulated)".into()));
        }
        result
    }
}

/// One account's rows on a [`MemoryServer`], from
/// [`MemoryServer::snapshot`].
#[derive(Clone, Debug)]
pub struct AccountSnapshot {
    docs: Vec<(String, DocRow)>,
    files: Vec<(String, Vec<u8>)>,
    last_seq: i64,
    /// `docs`, `doc_bytes`, `files`, `file_bytes`.
    counters: [u64; 4],
}

/// One account of a [`MemoryServer`], for [`PlainView`]: reads its rows
/// directly and pushes through the push route with the session's token,
/// without counting the request in [`MemoryServer::stats`] or taking a
/// pending dropped response.
pub(crate) struct AccountHandle {
    state: Arc<Mutex<ServerState>>,
    account_id: String,
    token: SessionToken,
}

impl AccountHandle {
    fn server(&self) -> std::sync::MutexGuard<'_, ServerState> {
        self.state.lock().expect("memory server poisoned")
    }

    /// The account's documents in `seq` order, tombstones included.
    pub(crate) fn docs(&self) -> Vec<WireChange> {
        let state = self.server();
        let mut docs: Vec<WireChange> = state
            .docs
            .iter()
            .filter(|((account, _), _)| *account == self.account_id)
            .map(|((_, id), row)| WireChange {
                id: id.clone(),
                rev: row.rev.clone(),
                deleted: row.deleted,
                sealed: row.sealed.clone(),
                seq: row.seq,
            })
            .collect();
        docs.sort_by_key(|d| d.seq);
        docs
    }

    /// The sealed bytes stored for `wire_id`.
    pub(crate) fn file(&self, wire_id: &str) -> Option<Vec<u8>> {
        self.server()
            .files
            .get(&(self.account_id.clone(), wire_id.to_owned()))
            .cloned()
    }

    /// `PUT /v2/files/<wire_id>` with the session's token.
    pub(crate) fn put_file(&self, wire_id: &str, sealed: &[u8]) -> Result<(), ApiError> {
        let mut state = self.server();
        let now = state.clock();
        state
            .put_file(&self.token, wire_id, sealed, now)
            .map(|_| ())
    }

    /// `POST /v2/push` with the session's token.
    pub(crate) fn push(&self, docs: &[WirePushDoc]) -> Result<Vec<WirePushResult>, ApiError> {
        let mut state = self.server();
        let now = state.clock();
        let request = PushRequest {
            docs: docs.to_vec(),
        };
        Ok(state.push(&self.token, &request, now)?.results)
    }
}

/// [`MemoryServer`]'s routes; every server URL reaches the same server.
struct MemoryApi<'a> {
    server: &'a MemoryServer,
}

impl Api for MemoryApi<'_> {
    fn health(&self) -> Result<Health, ApiError> {
        self.server.call(Route::Health, |s, _| Ok(s.health()))
    }

    fn prelogin(&self, request: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        self.server
            .call(Route::Prelogin, |s, _| s.prelogin(request))
    }

    fn signup(&self, request: &SignupRequest) -> Result<SessionResponse, ApiError> {
        self.server
            .call(Route::Signup, |s, now| s.signup(request, now))
    }

    fn login(&self, request: &LoginRequest) -> Result<LoginResponse, ApiError> {
        self.server
            .call(Route::Login, |s, now| s.login(request, now))
    }

    fn recover_start(
        &self,
        request: &RecoverStartRequest,
    ) -> Result<RecoverStartResponse, ApiError> {
        self.server
            .call(Route::RecoverStart, |s, now| s.recover_start(request, now))
    }

    fn recover_finish(&self, request: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
        self.server.call(Route::RecoverFinish, |s, now| {
            s.recover_finish(request, now)
        })
    }

    fn logout(&self, token: &SessionToken) -> Result<(), ApiError> {
        self.server
            .call(Route::Logout, |s, now| s.logout(token, now))
    }

    fn account(&self, token: &SessionToken) -> Result<AccountResponse, ApiError> {
        self.server
            .call(Route::Account, |s, now| s.account(token, now))
    }

    fn change_password(
        &self,
        token: &SessionToken,
        request: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.server.call(Route::Password, |s, now| {
            s.change_password(token, request, now)
        })
    }

    fn replace_recovery_key(
        &self,
        token: &SessionToken,
        request: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.server.call(Route::RecoveryKey, |s, now| {
            s.replace_recovery_key(token, request, now)
        })
    }

    fn delete_account(
        &self,
        token: &SessionToken,
        request: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError> {
        self.server.call(Route::Delete, |s, now| {
            s.delete_account(token, request, now)
        })
    }

    fn set_device_name(
        &self,
        token: &SessionToken,
        request: &DeviceNameRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::SetDevice, |s, now| {
            s.set_device_name(token, request, now)
        })
    }

    fn devices(&self, token: &SessionToken) -> Result<DevicesResponse, ApiError> {
        self.server
            .call(Route::Devices, |s, now| s.devices(token, now))
    }

    fn revoke_device(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        self.server.call(Route::RevokeDevice, |s, now| {
            s.revoke_device(token, device_id, now)
        })
    }

    fn push(&self, token: &SessionToken, request: &PushRequest) -> Result<PushResponse, ApiError> {
        self.server
            .call(Route::Push, |s, now| s.push(token, request, now))
    }

    fn changes(
        &self,
        token: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<WirePage, ApiError> {
        self.server
            .call(Route::Changes, |s, now| s.changes(token, since, limit, now))
    }

    fn put_file(
        &self,
        token: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
    ) -> Result<PutFileResponse, ApiError> {
        self.server.call(Route::PutFile, |s, now| {
            s.put_file(token, wire_id, sealed, now)
        })
    }

    fn get_file(&self, token: &SessionToken, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.server
            .call(Route::GetFile, |s, now| s.get_file(token, wire_id, now))
    }
}

impl Backend for MemoryServer {
    fn api(&self, _server: &str) -> Box<dyn Api + '_> {
        Box::new(MemoryApi { server: self })
    }

    fn kdf_floor(&self) -> KdfParams {
        self.server().floor
    }

    fn kdf_default(&self) -> KdfParams {
        self.server().default
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;
    use proptest::prelude::*;

    /// `DEFAULT_SETTINGS` is exactly what the migration seeds.
    #[test]
    fn default_settings_mirror_the_migration() {
        let sql = include_str!("../../../cloud/migrations/0002_accounts.sql");
        let start = sql.find("INSERT INTO settings").unwrap();
        let end = start + sql[start..].find(';').unwrap();
        let seeded: Vec<(String, String)> = sql[start..end]
            .lines()
            .filter_map(|line| {
                let line = line.trim().trim_end_matches(',');
                let inner = line.strip_prefix("('")?.strip_suffix("')")?;
                let (name, value) = inner.split_once("', '")?;
                Some((name.to_owned(), value.to_owned()))
            })
            .collect();
        let expected: Vec<(String, String)> = DEFAULT_SETTINGS
            .iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect();
        assert_eq!(seeded, expected);
        let limits = Limits::DEFAULT;
        for (name, value) in [
            ("quota_docs", limits.docs),
            ("quota_doc_bytes", limits.doc_bytes),
            ("quota_files", limits.files),
            ("quota_file_bytes", limits.file_bytes),
        ] {
            assert!(
                DEFAULT_SETTINGS.contains(&(name, value.to_string().as_str())),
                "{name}"
            );
        }
    }

    #[test]
    fn a_new_server_takes_sign_ups() {
        let server = MemoryServer::new();
        assert_eq!(server.server().settings["signup"], "open");
        server.set_signup(Signup::Only("alice".into()));
        assert_eq!(server.server().settings["signup"], "only:alice");
        assert_eq!(server.kdf_floor(), KdfParams::TEST);
        assert_eq!(server.stats(), ServerStats::default());
    }

    #[test]
    #[should_panic(expected = "D1 refuses")]
    fn settings_d1_refuses_are_refused() {
        MemoryServer::new().set_setting("max_accounts", "12a");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: the settings check agrees with D1's CHECK, spelled out
        /// with explicit cases for each branch.
        #[test]
        fn prop_settings_check_matches_d1(value in "(only:)?[a-z0-9]{0,40}") {
            let signup_ok = value == "open" || value == "closed"
                || (value.starts_with("only:") && value.len() >= 8 && value.len() <= 37);
            prop_assert_eq!(is_valid_setting("signup", &value), signup_ok);
            let numeric_ok = !value.is_empty() && value.len() <= 15 && value.chars().all(|c| c.is_ascii_digit());
            prop_assert_eq!(is_valid_setting("max_accounts", &value), numeric_ok);
        }
    }
}
