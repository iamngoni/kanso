//! [`MemoryServer`], the in-memory test server: the Worker's v2 rules for
//! accounts, sessions, sealed sync, quotas and limits (frozen by step 1).

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::account::{Session, SignupState};
use crate::api::{
    AccountPasskeyResponse, AccountPasskeysFinishRequest, AccountPasskeysFinishResponse,
    AccountPasskeysStartRequest, AccountPasskeysStartResponse, AccountSignInMethodsResponse,
    LinkedIdentityResponse, PasskeyProofRequest, PasskeyProofStartRequest,
    PasskeyProofStartResponse, SignInPasswordRequest,
};
use crate::api::{
    AccountResponse, Api, ApprovalApproveRequest, ApprovalChallengeRequest, ApprovalCreateRequest,
    ApprovalCreated, ApprovalList, ApprovalNameRequest, ApprovalPoll, ApprovalRow, Backend,
    DeleteRequest, DeleteResponse, DeviceNameRequest, DeviceRow, DevicesResponse, Health,
    HealthAuth, HealthEvents, HealthPasskeys, LoginRequest, LoginResponse, NewDevice,
    PasskeyAllowedCredential, PasskeyFinishRequest, PasskeySelection, PasskeySessionKind,
    PasskeySessionResponse, PasskeySignupFinishRequest, PasskeySignupStartRequest,
    PasskeySignupStartResponse, PasskeyStartRequest, PasskeyStartResponse, PasskeyUser,
    PasswordRequest, PasswordResponse, PreloginRequest, PreloginResponse, PushRegistration,
    RecoverFinishRequest, RecoverStartRequest, RecoverStartResponse, RecoveryKeyRequest,
    RecoveryKeyResponse, Route, SessionResponse, SignupRequest, WireDevice, WireKdf, WireQuota,
    WireUsage,
};
use crate::crypto::{KdfParams, b64, hex, hkdf32, hmac, random, sha256_hex, unb64};
use crate::provider::ProofWire;
use crate::provider::{
    AppleStart, GithubStart, GoogleFinishRequest, GoogleStart, GoogleStartRequest, IdentityIntent,
    IdentityRow, IdentityStartRequest, LinkOutcome, MethodsReply, Provider, ProviderPollRequest,
    ProviderSession, ProviderStart, ProviderStartRequest, ProviderStep, SocialSignupRequest,
    UnlinkRequest,
};
use crate::sealed::PlainView;
use crate::username::{UsernameError, check_new_username, normalize_username};
use crate::webauthn::{Expected, verify_assertion, verify_registration};
use crate::wire::{
    ApiError, DEFAULT_CHANGES_LIMIT, DOC_ROW_COST, FILE_ROW_COST, MAX_ACTIVE_DEVICES,
    MAX_CHANGES_LIMIT as MAX_WIRE_CHANGES_LIMIT, MAX_PUSH_BODY, MAX_PUSH_DOCS, MAX_SEALED_FILE,
    MAX_SEALED_LEN, MIN_SEALED_FILE, MIN_SEALED_LEN, PAGE_SEALED_BUDGET, PROTOCOL_VERSION,
    PushRequest, PushResponse, PutFileResponse, QuotaKind, SESSION_IDLE_MS, SessionToken,
    SignOutReason, SignupClosedReason, WireChange, WireDoc, WirePage, WirePushDoc, WirePushResult,
    is_account_id, is_approval_id, is_base64url, is_device_id, is_hex16, is_sealed_name, is_token,
    is_wire_file_id, is_wire_id, is_wire_rev,
};
use sha2::{Digest as _, Sha256};

/// Memory-only relying party; it is for client tests and is not a live RP.
pub const MEMORY_PASSKEY_RP_ID: &str = "memory.kanso.test";
/// The one client-data origin the memory relying party accepts.
pub const MEMORY_PASSKEY_ORIGIN: &str = "https://memory.kanso.test";

/// A server URL whose host is [`MEMORY_PASSKEY_RP_ID`], so the client's
/// relying-party rule ([`crate::passkey_rp_allowed`]) accepts
/// `MemoryServer`'s default relying party from it.
pub const MEMORY_PASSKEY_SERVER: &str = "https://memory.kanso.test";
const PASSKEY_FLOW_TTL_MS: i64 = 5 * 60_000;

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
/// A device approval lives five minutes (docs/ACCOUNTS.md › The protocol).
const APPROVAL_TTL_MS: i64 = 300_000;
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
    ApprovalCreate,
    ApprovalPoll,
    ApprovalRespond,
    ProofAccount,
    /// Per account, counted after a device's own `password-account`.
    PasswordTotal,
    /// Per account, counted after a device's own `proof-account`.
    ProofTotal,
    PasskeyChallenge,
    PasskeyManage,
    ProviderPoll,
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
            Rule::ApprovalCreate => "approval-create",
            Rule::ApprovalPoll => "approval-poll",
            Rule::ApprovalRespond => "approval-respond",
            Rule::ProofAccount => "proof-account",
            Rule::PasswordTotal => "password-total",
            Rule::ProofTotal => "proof-total",
            Rule::PasskeyChallenge => "passkey-challenge",
            Rule::PasskeyManage => "passkey-manage",
            Rule::ProviderPoll => "provider-poll",
        }
    }

    fn window(self) -> i64 {
        match self {
            Rule::SignupIp | Rule::RecoverIp => HOUR_MS,
            Rule::LoginUser | Rule::LoginIp | Rule::PasswordAccount | Rule::PasswordTotal => {
                QUARTER_HOUR_MS
            }
            Rule::WritesAccount | Rule::UploadsAccount => DAY_MS,
            Rule::ApprovalCreate | Rule::ApprovalPoll | Rule::ApprovalRespond => HOUR_MS,
            Rule::ProofAccount | Rule::ProofTotal | Rule::PasskeyManage => HOUR_MS,
            Rule::PasskeyChallenge | Rule::ProviderPoll => QUARTER_HOUR_MS,
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
    has_password: bool,
}

#[derive(Clone, Debug)]
struct PasskeyRow {
    credential_id: String,
    account_id: String,
    /// The COSE public key the registration attested.
    public_key: Vec<u8>,
    alg: i32,
    sign_count: u32,
    transports: String,
    wrapped_key: Option<String>,
    created_at: i64,
    last_used_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PasskeyFlowKind {
    Signin,
    Signup,
    /// `sign-in/proof/start`: a session's fresh proof.
    Proof,
    /// `sign-in/passkeys/start`: a signed-in account adds a credential.
    Register,
}

#[derive(Clone, Debug)]
struct PasskeyFlow {
    kind: PasskeyFlowKind,
    account_id: Option<String>,
    username: Option<String>,
    device_id: String,
    device_proof_hash: String,
    challenge: String,
    expires_at: i64,
    consumed_at: Option<i64>,
    /// A proof flow started for one credential takes only that one.
    credential: Option<String>,
}

/// Scripted provider behavior for deterministic client tests.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ProviderTestOutcome {
    Session { account_id: String },
    NewAccount { suggested_username: String },
    Pending,
    SlowDown,
    ProviderDenied,
    ProviderUnavailable,
    FlowExpired,
    IdentityTaken,
}

#[derive(Clone, Debug)]
struct ProviderFlow {
    provider: Provider,
    device_id: String,
    proof_hash: String,
    expires_at: i64,
    transitions: VecDeque<ProviderTestOutcome>,
    current: ProviderTestOutcome,
    subject: String,
    account_id: Option<String>,
    intent: Option<IdentityIntent>,
    code_challenge: Option<String>,
    consumed: bool,
    /// A completed proof flow was spent on a sensitive change.
    proof_used: bool,
    /// Its starting session ended, or its account was disabled (migration
    /// 0010): it is used up, a finished proof included.
    ended: bool,
    /// The device flow's state when the fake GitHub runs it
    /// ([`MemoryServer::set_github_device_flow`]).
    github: Option<GithubFlow>,
}

/// One GitHub device flow against the fake GitHub.
#[derive(Clone, Debug)]
struct GithubFlow {
    /// The GitHub user ID, once a token poll was authorized.
    subject: Option<String>,
    next_poll_at: i64,
    interval: u64,
}

/// What the fake GitHub answers (the fixtures' `github` object).
#[derive(Clone, Debug)]
struct GithubFake {
    subject: String,
    user_code: String,
    interval: u64,
    /// The token endpoint's next answers; empty is `authorization_pending`.
    answers: VecDeque<GithubAnswer>,
}

#[derive(Clone, Copy, Debug)]
enum GithubAnswer {
    Authorized,
    Denied,
    SlowDown,
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
    /// The device's push token; dropped with the session.
    push: Option<PushRegistration>,
}

/// A `device_approvals` row.
#[derive(Clone, Debug)]
struct ApprovalRowState {
    id: String,
    account_id: String,
    device_id: String,
    commit_hash: String,
    epk: Option<String>,
    approver_device_id: Option<String>,
    approver_epk: Option<String>,
    sealed_name: Option<String>,
    wrapped_key: Option<String>,
    created_at: i64,
    expires_at: i64,
    completed_at: Option<i64>,
    cancelled_at: Option<i64>,
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
    passkeys: Vec<PasskeyRow>,
    passkey_flows: BTreeMap<String, PasskeyFlow>,
    providers: Vec<Provider>,
    provider_outcomes: BTreeMap<String, VecDeque<ProviderTestOutcome>>,
    provider_flows: BTreeMap<String, ProviderFlow>,
    /// `PASSKEY_RP_ID` and `PASSKEY_ORIGINS` as health advertises them;
    /// `None` is a server without passkeys.
    passkey_rp: Option<(String, Vec<String>)>,
    github: Option<GithubFake>,
    identities: Vec<(String, Provider, String, i64)>,
    sessions: Vec<SessionRow>,
    approvals: Vec<ApprovalRowState>,
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

/// `SHA-256("kanso/v1/approve-commit" || 0x01 || epk)` as 43 base64url
/// characters, as the Worker checks a revealed key.
fn approval_commit(epk: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"kanso/v1/approve-commit");
    hasher.update([1u8]);
    hasher.update(epk);
    b64(&hasher.finalize())
}

fn is_salt(salt: &str) -> bool {
    salt.len() == 22 && is_base64url(salt)
}

fn is_wrap(wrap: &str) -> bool {
    wrap.len() == 98 && is_base64url(wrap)
}

fn passkey_bytes(value: &str, min: usize, max: usize) -> Option<Vec<u8>> {
    if !is_base64url(value) {
        return None;
    }
    let bytes = unb64(value)?;
    (bytes.len() >= min && bytes.len() <= max && b64(&bytes) == value).then_some(bytes)
}

/// The Worker's sign-count rule as its claim applies it: a stored 0 never
/// counts, and otherwise the new count must grow (a 0 after a count is
/// refused, as the claim's `sign_count < ?` is).
fn counts_up(stored: u32, next: u32) -> bool {
    stored == 0 || next > stored
}

/// A flow ID: 32 bytes of canonical base64url.
fn is_flow_id(flow: &str) -> bool {
    flow.len() == 43 && is_base64url(flow) && unb64(flow).is_some_and(|b| b.len() == 32)
}

/// Transport hints as the column keeps them: de-duplicated, space separated.
fn transports_value(transports: &[String]) -> String {
    let mut unique: Vec<&String> = Vec::new();
    for t in transports {
        if !unique.contains(&t) {
            unique.push(t);
        }
    }
    unique
        .into_iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Malformed transport hints (400).
fn bad_transports(transports: &[String]) -> bool {
    transports.len() > 32
        || transports.iter().any(|t| {
            t.is_empty()
                || t.len() > 32
                || !t
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !t.as_bytes()[0].is_ascii_lowercase()
        })
}

/// Which provider proofs a change takes (proofs.ts `ProofRules.provider`):
/// none, any linked provider (with the recovery key), or only this one.
#[derive(Clone, Copy)]
enum ProviderRule<'a> {
    None,
    Any,
    Only(&'a Provider),
}

/// How a change checks its proof.
#[derive(Clone, Copy)]
struct ProofRules<'a> {
    provider: ProviderRule<'a>,
    /// Count `proof-account` for a passkey or provider proof.
    count_proof: bool,
    /// The route counted `password-account` already.
    password_counted: bool,
}

impl ProofRules<'_> {
    /// A password or passkey proof: adding or removing a way in.
    const STRONG: ProofRules<'static> = ProofRules {
        provider: ProviderRule::None,
        count_proof: false,
        password_counted: false,
    };
}

/// One fake-GitHub poll's answer: a sign-in step, or a link or proof outcome.
enum GithubPolled {
    Step(ProviderStep),
    Link(LinkOutcome),
}

/// What a checked proof uses up once its change goes through.
enum Spend {
    Nothing,
    Passkey {
        flow: String,
        credential: String,
        count: u32,
    },
    Provider(String),
}

/// The Worker's `passkeyAdvertisement`: a usable RP ID and a non-empty JSON
/// array of origins, each scheme, host and port only, `https://` (or
/// `http://localhost`), on the RP ID or under it.
fn passkey_advertisement(
    rp_id: Option<&str>,
    origins_json: Option<&str>,
) -> Option<(String, Vec<String>)> {
    let rp_id = rp_id.map(str::trim).filter(|r| !r.is_empty())?;
    let usable = rp_id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        && !rp_id.contains("..")
        && !rp_id.starts_with(['.', '-'])
        && !rp_id.ends_with('.')
        && rp_id.parse::<std::net::Ipv4Addr>().is_err();
    if !usable {
        return None;
    }
    let raw = origins_json.map(str::trim).filter(|r| !r.is_empty())?;
    let values: Vec<String> = serde_json::from_str(raw).ok()?;
    if values.is_empty() {
        return None;
    }
    for value in &values {
        let url = url::Url::parse(value).ok()?;
        if url.origin().ascii_serialization() != *value {
            return None;
        }
        let host = url.host_str()?;
        let localhost = url.scheme() == "http" && host == "localhost";
        if url.scheme() != "https" && !localhost {
            return None;
        }
        if host != rp_id && !host.ends_with(&format!(".{rp_id}")) {
            return None;
        }
    }
    Some((rp_id.to_owned(), values))
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
            passkeys: Vec::new(),
            passkey_flows: BTreeMap::new(),
            providers: Vec::new(),
            provider_outcomes: BTreeMap::new(),
            provider_flows: BTreeMap::new(),
            passkey_rp: Some((
                MEMORY_PASSKEY_RP_ID.to_owned(),
                vec![MEMORY_PASSKEY_ORIGIN.to_owned()],
            )),
            github: None,
            identities: Vec::new(),
            sessions: Vec::new(),
            approvals: Vec::new(),
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

    /// The Worker's `countSession`: `password-account` (5 per 15 minutes)
    /// or `proof-account` (20 an hour) per account and device (the device
    /// ID's first 64 bits), then, only once that passes, the account's total
    /// (`password-total` 15, `proof-total` 60), so one session can't use up
    /// another device's allowance.
    fn count_session(&mut self, rule: Rule, caller: &Caller, now: i64) -> Result<(), ApiError> {
        let (max, total, total_max) = match rule {
            Rule::PasswordAccount => (5, Rule::PasswordTotal, 15),
            Rule::ProofAccount => (20, Rule::ProofTotal, 60),
            _ => unreachable!("only the per-session proof rules"),
        };
        let device = caller.device_id.get(2..18).unwrap_or(&caller.device_id);
        let own = format!("{}:{}:{device}", rule.name(), caller.account_id);
        self.count(rule, &own, now, 1, max)?;
        let total_key = format!("{}:{}", total.name(), caller.account_id);
        self.count(total, &total_key, now, 1, total_max)
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
            row.push = None;
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
        row.push = None;
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
                push: None,
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

    fn provider_start(
        &mut self,
        provider: &Provider,
        device: &crate::api::LoginDevice,
        now: i64,
        intent: Option<IdentityIntent>,
        account_id: Option<String>,
        code_challenge: Option<String>,
    ) -> Result<ProviderStart, ApiError> {
        if !matches!(
            provider,
            Provider::Apple | Provider::Google | Provider::Github
        ) {
            return Err(ApiError::UnsupportedMethod);
        }
        if !self.providers.contains(provider) {
            return Err(ApiError::ProviderDisabled);
        }
        if !is_device_id(&device.id) || key32(&device.proof).is_none() {
            return Err(ApiError::BadRequest);
        }
        let flow_id = b64(&random::<32>());
        let github = (*provider == Provider::Github)
            .then(|| self.github.clone())
            .flatten();
        let mut outcomes = self
            .provider_outcomes
            .remove(provider.as_str())
            .unwrap_or_default();
        let current = outcomes.pop_front().unwrap_or(ProviderTestOutcome::Pending);
        self.provider_flows.insert(
            flow_id.clone(),
            ProviderFlow {
                provider: provider.clone(),
                device_id: device.id.clone(),
                proof_hash: sha256_hex(&unb64(&device.proof).ok_or(ApiError::BadRequest)?),
                expires_at: now + 600_000,
                transitions: outcomes,
                current,
                subject: format!("memory:{flow_id}"),
                account_id,
                intent,
                code_challenge,
                consumed: false,
                proof_used: false,
                ended: false,
                github: github.as_ref().map(|fake| GithubFlow {
                    subject: None,
                    next_poll_at: now + fake.interval as i64 * 1000,
                    interval: fake.interval,
                }),
            },
        );
        Ok(match provider {
            Provider::Apple => ProviderStart::Apple(AppleStart {
                flow_id,
                auth_url: "https://appleid.apple.com/auth/authorize?client_id=memory".into(),
                expires_at: now + 600_000,
            }),
            Provider::Google => ProviderStart::Google(GoogleStart {
                flow_id,
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth?client_id=memory".into(),
                nonce: b64(&random::<32>()),
                state: b64(&random::<32>()),
                expires_at: now + 600_000,
            }),
            Provider::Github => match github {
                Some(fake) => ProviderStart::Github(GithubStart {
                    flow_id,
                    user_code: fake.user_code,
                    verification_uri: "https://github.com/login/device".into(),
                    interval: fake.interval,
                    expires_in: 600,
                }),
                None => ProviderStart::Github(GithubStart {
                    flow_id,
                    user_code: "MEMORY-CODE".into(),
                    verification_uri: "https://github.com/login/device".into(),
                    interval: 5,
                    expires_in: 600,
                }),
            },
            Provider::Other(_) => unreachable!(),
        })
    }

    // -- the fake GitHub's device flow (MemoryServer::set_github_device_flow)
    //
    // As the Worker's githubPoll and complete(): one poll per interval, the
    // token endpoint's queued answers, then what the subject does by the
    // flow's intent.

    fn github_flow(&self, flow_id: &str) -> bool {
        self.provider_flows
            .get(flow_id)
            .is_some_and(|f| f.github.is_some())
    }

    fn github_poll(
        &mut self,
        flow_id: &str,
        provider: &Provider,
        device: &crate::api::LoginDevice,
        now: i64,
    ) -> Result<GithubPolled, ApiError> {
        self.count(
            Rule::ProviderPoll,
            &format!("provider-poll:{flow_id}"),
            now,
            1,
            300,
        )?;
        let checked = Self::check_device(&NewDevice {
            id: device.id.clone(),
            sealed_name: None,
            proof: device.proof.clone(),
        })?;
        let f = self
            .provider_flows
            .get(flow_id)
            .filter(|f| f.expires_at > now)
            .cloned()
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if f.provider != *provider
            || f.device_id != checked.id
            || f.proof_hash != checked.proof_hash
        {
            return Err(ApiError::BadRequest);
        }
        let github = f.github.clone().expect("a GitHub device flow");
        if github.subject.is_some() {
            return self.github_complete(flow_id, &checked, now);
        }
        if f.consumed {
            // The Worker's endedFlow: used up before GitHub answered. A link
            // or proof whose starting session has since ended (migration
            // 0010 used it up) is 400 flow_expired; otherwise GitHub refused.
            let ended = f.intent.is_some()
                && !f.account_id.as_deref().is_some_and(|account| {
                    self.starting_session_live(account, &checked.id, &checked.proof_hash)
                });
            return Err(if ended {
                ApiError::FlowExpired { gone: false }
            } else {
                ApiError::ProviderDenied
            });
        }
        if !self.providers.contains(&Provider::Github) {
            return Err(ApiError::ProviderDisabled);
        }
        if now < github.next_poll_at {
            let wait = ((github.next_poll_at - now + 999) / 1000).max(1);
            return Err(ApiError::RateLimited {
                retry_after: wait as u64,
                limit: "provider-poll".into(),
            });
        }
        let (answer, subject) = match self.github.as_mut() {
            Some(fake) => (fake.answers.pop_front(), fake.subject.clone()),
            None => (None, String::new()),
        };
        let flow = self
            .provider_flows
            .get_mut(flow_id)
            .expect("flow was checked");
        let state = flow.github.as_mut().expect("a GitHub device flow");
        state.next_poll_at = now + state.interval as i64 * 1000;
        match answer {
            None => Ok(GithubPolled::Step(ProviderStep::Pending {
                slow_down: false,
            })),
            Some(GithubAnswer::SlowDown) => {
                // GitHub adds five seconds each time.
                state.interval += 5;
                state.next_poll_at = state.next_poll_at.max(now + state.interval as i64 * 1000);
                Ok(GithubPolled::Step(ProviderStep::Pending {
                    slow_down: true,
                }))
            }
            Some(GithubAnswer::Denied) => {
                flow.consumed = true;
                Err(ApiError::ProviderDenied)
            }
            Some(GithubAnswer::Authorized) => {
                state.subject = Some(subject);
                self.github_complete(flow_id, &checked, now)
            }
        }
    }

    /// The Worker's migration 0010 triggers: a session's sensitive flows end
    /// with it. Unconsumed passkey proof and registration flows, and link
    /// and proof provider flows, are used up once no live session of their
    /// account is on their device (a revocation, a sign-out, an eviction),
    /// and every one of an account's once it is disabled. Sign-in and
    /// sign-up flows belong to no session and are left alone. Run after
    /// every request and after [`MemoryServer::disable`].
    fn end_flows_of_ended_sessions(&mut self, now: i64) {
        let live = |state: &ServerState, account_id: &str, device_id: &str| {
            state
                .accounts
                .get(account_id)
                .is_some_and(|a| a.disabled_at.is_none())
                && state.sessions.iter().any(|s| {
                    s.account_id == account_id && s.device_id == device_id && s.revoked_at.is_none()
                })
        };
        let ended: Vec<String> = self
            .passkey_flows
            .iter()
            .filter(|(_, f)| {
                matches!(f.kind, PasskeyFlowKind::Proof | PasskeyFlowKind::Register)
                    && f.consumed_at.is_none()
                    && f.account_id
                        .as_deref()
                        .is_some_and(|account| !live(self, account, &f.device_id))
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ended {
            self.passkey_flows.get_mut(&id).expect("listed").consumed_at = Some(now);
        }
        let ended: Vec<String> = self
            .provider_flows
            .iter()
            .filter(|(_, f)| {
                f.intent.is_some()
                    && !f.ended
                    && f.account_id
                        .as_deref()
                        .is_some_and(|account| !live(self, account, &f.device_id))
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ended {
            let flow = self.provider_flows.get_mut(&id).expect("listed");
            flow.consumed = true;
            flow.ended = true;
        }
    }

    /// Whether the session that started a link or proof flow is still
    /// live, as the Worker's link completion requires: a session of the
    /// account on the flow's device with this poll's device proof, not
    /// revoked, and the account enabled.
    fn starting_session_live(&self, account_id: &str, device_id: &str, proof_hash: &str) -> bool {
        self.accounts
            .get(account_id)
            .is_some_and(|a| a.disabled_at.is_none())
            && self.sessions.iter().any(|s| {
                s.account_id == account_id
                    && s.device_id == device_id
                    && s.proof_hash == proof_hash
                    && s.revoked_at.is_none()
            })
    }

    /// What a verified GitHub subject does, by the flow's intent.
    fn github_complete(
        &mut self,
        flow_id: &str,
        device: &CheckedDevice,
        now: i64,
    ) -> Result<GithubPolled, ApiError> {
        let f = self.provider_flows[flow_id].clone();
        let subject = f
            .github
            .as_ref()
            .and_then(|g| g.subject.clone())
            .expect("a verified subject");
        let provider = f.provider.clone();
        let owner = self
            .identities
            .iter()
            .find(|(_, p, s, _)| *p == provider && *s == subject)
            .map(|(account, _, _, created_at)| (account.clone(), *created_at));
        match f.intent {
            None => {
                self.count(Rule::LoginIp, &format!("login-ip:{CLIENT_IP}"), now, 1, 30)?;
                let Some((account_id, _)) = owner else {
                    if f.consumed {
                        return Err(ApiError::FlowExpired { gone: false });
                    }
                    // Every unknown identity shares one login-user row.
                    self.count(Rule::LoginUser, "login-user:unknown", now, 1, 10)?;
                    return Ok(GithubPolled::Step(ProviderStep::NewAccount(
                        crate::provider::NewAccount {
                            ticket: flow_id.to_owned(),
                            suggested_username: "githubuser".into(),
                        },
                    )));
                };
                let known = self.sessions.iter().any(|r| {
                    r.account_id == account_id
                        && r.device_id == device.id
                        && r.revoked.is_none()
                        && r.proof_hash == device.proof_hash
                });
                if !known {
                    self.count(
                        Rule::LoginUser,
                        &format!("login-user:{account_id}"),
                        now,
                        1,
                        10,
                    )?;
                }
                let account = self.accounts[&account_id].clone();
                if account.disabled_at.is_some() {
                    return Err(ApiError::AccountDisabled);
                }
                if f.consumed {
                    return Err(ApiError::FlowExpired { gone: false });
                }
                self.provider_flows
                    .get_mut(flow_id)
                    .expect("flow was checked")
                    .consumed = true;
                let checked = CheckedDevice {
                    id: device.id.clone(),
                    proof_hash: device.proof_hash.clone(),
                    sealed_name: None,
                };
                let (token, evicted_device) = self.start_session(&account_id, &checked, now);
                Ok(GithubPolled::Step(ProviderStep::Session(ProviderSession {
                    account_id,
                    username: account.username,
                    token,
                    key_id: account.key_id,
                    epoch: account.epoch,
                    has_password: account.has_password,
                    passkey_wrapped_key: None,
                    evicted_device,
                })))
            }
            Some(IdentityIntent::Link) => {
                let account_id = f.account_id.clone().ok_or(ApiError::BadRequest)?;
                if f.consumed {
                    return Err(ApiError::FlowExpired { gone: false });
                }
                self.provider_flows
                    .get_mut(flow_id)
                    .expect("flow was checked")
                    .consumed = true;
                // It adds a way in: only while the session that started it lives.
                if !self.starting_session_live(&account_id, &device.id, &device.proof_hash) {
                    return Err(ApiError::FlowExpired { gone: false });
                }
                let has_provider = self
                    .identities
                    .iter()
                    .any(|(a, p, _, _)| *a == account_id && *p == provider);
                match owner {
                    None if !has_provider && self.accounts.contains_key(&account_id) => {
                        self.identities
                            .push((account_id, provider.clone(), subject, now));
                        Ok(GithubPolled::Link(LinkOutcome::Linked {
                            provider,
                            created_at: now,
                        }))
                    }
                    Some((owner, created_at)) if owner == account_id => {
                        Ok(GithubPolled::Link(LinkOutcome::Linked {
                            provider,
                            created_at,
                        }))
                    }
                    // Another account's, or this one has another identity of
                    // the provider; never says which.
                    _ => Err(ApiError::IdentityTaken),
                }
            }
            // The flow stays unused: the change that carries it uses it.
            Some(IdentityIntent::Proof) => {
                let account_id = f.account_id.clone().unwrap_or_default();
                if owner.is_some_and(|(owner, _)| owner == account_id) {
                    Ok(GithubPolled::Link(LinkOutcome::Proof { provider }))
                } else {
                    Err(ApiError::InvalidCredentials)
                }
            }
        }
    }

    fn provider_result(
        &mut self,
        flow_id: &str,
        provider: &Provider,
        device: &crate::api::LoginDevice,
        now: i64,
    ) -> Result<ProviderStep, ApiError> {
        if self.github_flow(flow_id) {
            return match self.github_poll(flow_id, provider, device, now)? {
                GithubPolled::Step(step) => Ok(step),
                GithubPolled::Link(_) => Err(ApiError::BadRequest),
            };
        }
        let flow = self
            .provider_flows
            .get_mut(flow_id)
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if flow.consumed || flow.expires_at <= now {
            return Err(ApiError::FlowExpired { gone: false });
        }
        if &flow.provider != provider
            || flow.device_id != device.id
            || flow.proof_hash != sha256_hex(&unb64(&device.proof).ok_or(ApiError::BadRequest)?)
        {
            return Err(ApiError::BadRequest);
        }
        if matches!(
            flow.current,
            ProviderTestOutcome::Pending | ProviderTestOutcome::SlowDown
        ) && let Some(next) = flow.transitions.pop_front()
        {
            flow.current = next;
        }
        let outcome = flow.current.clone();
        let subject = flow.subject.clone();
        match outcome {
            ProviderTestOutcome::Pending => Ok(ProviderStep::Pending { slow_down: false }),
            ProviderTestOutcome::SlowDown => Ok(ProviderStep::Pending { slow_down: true }),
            ProviderTestOutcome::ProviderDenied => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::ProviderDenied)
            }
            ProviderTestOutcome::ProviderUnavailable => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::ProviderUnavailable)
            }
            ProviderTestOutcome::FlowExpired => {
                self.provider_flows.remove(flow_id);
                Err(ApiError::FlowExpired { gone: true })
            }
            ProviderTestOutcome::IdentityTaken => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::IdentityTaken)
            }
            ProviderTestOutcome::NewAccount { suggested_username } => {
                if self.provider_flows[flow_id].account_id.is_some() {
                    return Err(ApiError::BadRequest);
                }
                Ok(ProviderStep::NewAccount(crate::provider::NewAccount {
                    ticket: flow_id.into(),
                    suggested_username,
                }))
            }
            ProviderTestOutcome::Session { account_id } => {
                let account = self
                    .accounts
                    .get(&account_id)
                    .cloned()
                    .ok_or(ApiError::NotFound)?;
                if account.disabled_at.is_some() {
                    return Err(ApiError::AccountDisabled);
                }
                let checked = Self::check_device(&NewDevice {
                    id: device.id.clone(),
                    sealed_name: None,
                    proof: device.proof.clone(),
                })?;
                self.count(Rule::LoginIp, &format!("login-ip:{CLIENT_IP}"), now, 1, 30)?;
                if !self.sessions.iter().any(|r| {
                    r.account_id == account_id
                        && r.device_id == checked.id
                        && r.revoked.is_none()
                        && r.proof_hash == checked.proof_hash
                }) {
                    self.count(
                        Rule::LoginUser,
                        &format!("login-user:{account_id}"),
                        now,
                        1,
                        10,
                    )?;
                }
                let (token, evicted_device) = self.start_session(&account_id, &checked, now);
                if !self
                    .identities
                    .iter()
                    .any(|(id, p, s, _)| id == &account_id && p == provider && s == &subject)
                {
                    self.identities
                        .push((account_id.clone(), provider.clone(), subject, now));
                }
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Ok(ProviderStep::Session(ProviderSession {
                    account_id,
                    username: account.username,
                    token,
                    key_id: account.key_id,
                    epoch: account.epoch,
                    has_password: account.has_password,
                    passkey_wrapped_key: None,
                    evicted_device,
                }))
            }
        }
    }

    fn identity_result(
        &mut self,
        flow_id: &str,
        provider: &Provider,
        device: &crate::api::LoginDevice,
        now: i64,
    ) -> Result<LinkOutcome, ApiError> {
        if self.github_flow(flow_id) {
            return match self.github_poll(flow_id, provider, device, now)? {
                GithubPolled::Link(outcome) => Ok(outcome),
                GithubPolled::Step(_) => Err(ApiError::BadRequest),
            };
        }
        let f = self
            .provider_flows
            .get_mut(flow_id)
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if f.consumed || f.expires_at <= now {
            return Err(ApiError::FlowExpired { gone: false });
        }
        if &f.provider != provider
            || f.device_id != device.id
            || f.proof_hash != sha256_hex(&unb64(&device.proof).ok_or(ApiError::BadRequest)?)
        {
            return Err(ApiError::BadRequest);
        }
        if matches!(
            f.current,
            ProviderTestOutcome::Pending | ProviderTestOutcome::SlowDown
        ) && let Some(next) = f.transitions.pop_front()
        {
            f.current = next
        }
        let result = f.current.clone();
        let account = f.account_id.clone().ok_or(ApiError::BadRequest)?;
        let subject = f.subject.clone();
        let intent = f.intent.ok_or(ApiError::BadRequest)?;
        match result {
            ProviderTestOutcome::Pending | ProviderTestOutcome::SlowDown => {
                Err(ApiError::RateLimited {
                    retry_after: 1,
                    limit: "provider-poll".into(),
                })
            }
            ProviderTestOutcome::ProviderDenied => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::ProviderDenied)
            }
            ProviderTestOutcome::ProviderUnavailable => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::ProviderUnavailable)
            }
            ProviderTestOutcome::FlowExpired => {
                self.provider_flows.remove(flow_id);
                Err(ApiError::FlowExpired { gone: true })
            }
            ProviderTestOutcome::IdentityTaken => {
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                Err(ApiError::IdentityTaken)
            }
            ProviderTestOutcome::NewAccount { .. } => Err(ApiError::BadRequest),
            ProviderTestOutcome::Session { account_id } => {
                if account_id != account {
                    return Err(ApiError::BadRequest);
                }
                if self
                    .identities
                    .iter()
                    .any(|(id, p, s, _)| p == provider && s == &subject && id != &account)
                {
                    return Err(ApiError::IdentityTaken);
                }
                self.provider_flows.get_mut(flow_id).unwrap().consumed = true;
                let proof_hash = sha256_hex(&unb64(&device.proof).ok_or(ApiError::BadRequest)?);
                if intent == IdentityIntent::Link
                    && !self.starting_session_live(&account, &device.id, &proof_hash)
                {
                    return Err(ApiError::FlowExpired { gone: false });
                }
                match intent {
                    IdentityIntent::Proof => Ok(LinkOutcome::Proof {
                        provider: provider.clone(),
                    }),
                    IdentityIntent::Link => {
                        if self
                            .identities
                            .iter()
                            .any(|(id, p, s, _)| id == &account && p == provider && s == &subject)
                        {
                            return Err(ApiError::IdentityTaken);
                        }
                        self.identities
                            .push((account, provider.clone(), subject, now));
                        Ok(LinkOutcome::Linked {
                            provider: provider.clone(),
                            created_at: now,
                        })
                    }
                }
            }
        }
    }

    fn social_signup(
        &mut self,
        r: &SocialSignupRequest,
        now: i64,
    ) -> Result<ProviderSession, ApiError> {
        self.count(
            Rule::SignupIp,
            &format!("signup-ip:{CLIENT_IP}"),
            now,
            1,
            10,
        )?;
        let username = match check_new_username(&r.username) {
            Ok(name) => name,
            Err(UsernameError::Reserved) => return Err(ApiError::ReservedUsername),
            Err(_) => return Err(ApiError::InvalidUsername),
        };
        if !is_account_id(&r.account_id)
            || !is_hex16(&r.key_id)
            || key32(&r.recovery_auth_key).is_none()
            || !is_wrap(&r.recovery_wrapped_key)
        {
            return Err(ApiError::BadRequest);
        }
        let device = Self::check_device(&r.device)?;
        let flow = self
            .provider_flows
            .get(&r.ticket)
            .cloned()
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if flow.consumed || flow.expires_at <= now {
            return Err(ApiError::FlowExpired { gone: false });
        }
        if flow.device_id != device.id || flow.proof_hash != device.proof_hash {
            return Err(ApiError::BadRequest);
        }
        // A fake-GitHub ticket is a verified sign-in flow; a scripted one says
        // new_account.
        let subject = match &flow.github {
            Some(github) => match (&github.subject, flow.intent) {
                (Some(subject), None) => subject.clone(),
                _ => return Err(ApiError::FlowExpired { gone: false }),
            },
            None if matches!(flow.current, ProviderTestOutcome::NewAccount { .. }) => {
                flow.subject.clone()
            }
            None => return Err(ApiError::BadRequest),
        };
        let switch = self
            .settings
            .get("signup")
            .map(String::as_str)
            .unwrap_or("closed");
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
        if self
            .identities
            .iter()
            .any(|(_, p, s, _)| p == &flow.provider && *s == subject)
        {
            return Err(ApiError::IdentityTaken);
        }
        let account_id = r.account_id.clone();
        let epoch = hex(&random::<8>());
        let account = Account {
            id: account_id.clone(),
            username: username.clone(),
            kdf: self.default,
            salt: b64(&random::<16>()),
            auth_hash: sha256_hex(&random::<32>()),
            wrapped_key: b64(&random::<73>()),
            recovery_hash: sha256_hex(&unb64(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?),
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
            has_password: false,
        };
        self.accounts.insert(account_id.clone(), account);
        self.identities
            .push((account_id.clone(), flow.provider, subject, now));
        self.signups.add(now, 1);
        let (token, evicted_device) = self.start_session(&account_id, &device, now);
        self.provider_flows.get_mut(&r.ticket).unwrap().consumed = true;
        Ok(ProviderSession {
            account_id,
            username,
            token,
            key_id: r.key_id.clone(),
            epoch,
            has_password: false,
            passkey_wrapped_key: None,
            evicted_device,
        })
    }

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
            auth: Some(HealthAuth {
                password: true,
                passkeys: self
                    .passkey_rp
                    .clone()
                    .map(|(rp_id, origins)| HealthPasskeys {
                        rp_id,
                        origins,
                        security_key: true,
                    }),
                providers: self.providers.clone(),
            }),
            events: Some(HealthEvents {
                account: Some(crate::events::ACCOUNT_EVENTS_PATH.to_owned()),
            }),
        }
    }

    /// What a ceremony on `flow` must match.
    fn expected<'a>(rp_id: &'a str, origins: &'a [String], challenge: &'a str) -> Expected<'a> {
        Expected {
            rp_id,
            origins,
            challenge,
        }
    }

    /// The relying party, or 400 `unsupported_method` without passkeys.
    fn relying_party(&self) -> Result<(String, Vec<String>), ApiError> {
        self.passkey_rp.clone().ok_or(ApiError::UnsupportedMethod)
    }

    // The memory relying party checks ceremonies as the Worker does
    // (`webauthn.rs`): client data, RP ID hash, UP and UV, the attested key
    // and every assertion's signature and sign count.
    fn passkey_start(
        &mut self,
        r: &PasskeyStartRequest,
        now: i64,
    ) -> Result<PasskeyStartResponse, ApiError> {
        let (rp_id, _) = self.relying_party()?;
        self.count(
            Rule::PasskeyChallenge,
            &format!("passkey-challenge:{CLIENT_IP}"),
            now,
            1,
            30,
        )?;
        let device = Self::check_device(&NewDevice {
            id: r.device.id.clone(),
            sealed_name: None,
            proof: r.device.proof.clone(),
        })?;
        let username = r
            .username
            .as_deref()
            .map(normalize_username)
            .transpose()
            .map_err(|_| ApiError::InvalidUsername)?;
        let account_id = username
            .as_deref()
            .and_then(|name| self.account_by_name(name).map(|a| a.id.clone()));
        let challenge = b64(&random::<32>());
        let flow_id = b64(&random::<32>());
        self.passkey_flows.insert(
            flow_id.clone(),
            PasskeyFlow {
                kind: PasskeyFlowKind::Signin,
                account_id: account_id.clone(),
                username,
                device_id: device.id,
                device_proof_hash: device.proof_hash,
                challenge: challenge.clone(),
                expires_at: now + PASSKEY_FLOW_TTL_MS,
                consumed_at: None,
                credential: None,
            },
        );
        let allow_credentials = account_id
            .as_deref()
            .map(|id| {
                let mut rows: Vec<_> = self
                    .passkeys
                    .iter()
                    .filter(|p| p.account_id == id)
                    .collect();
                rows.sort_by(|a, b| {
                    (a.created_at, &a.credential_id).cmp(&(b.created_at, &b.credential_id))
                });
                rows.into_iter()
                    .map(|p| PasskeyAllowedCredential {
                        id: p.credential_id.clone(),
                        transports: p
                            .transports
                            .split_ascii_whitespace()
                            .map(str::to_owned)
                            .collect(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(PasskeyStartResponse {
            flow_id,
            challenge,
            rp_id,
            allow_credentials,
            user_verification: "required".into(),
            timeout: 300_000,
        })
    }

    fn passkey_finish(
        &mut self,
        r: &PasskeyFinishRequest,
        now: i64,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.count(Rule::LoginIp, &format!("login-ip:{CLIENT_IP}"), now, 1, 30)?;
        let device = Self::check_device(&NewDevice {
            id: r.device.id.clone(),
            sealed_name: None,
            proof: r.device.proof.clone(),
        })?;
        let credential_id =
            passkey_bytes(&r.credential_id, 16, 1023).ok_or(ApiError::BadRequest)?;
        let (Some(authenticator_data), Some(client_data_json), Some(signature)) = (
            passkey_bytes(&r.authenticator_data, 37, 8192),
            passkey_bytes(&r.client_data_json, 1, 8192),
            passkey_bytes(&r.signature, 1, 1024),
        ) else {
            return Err(ApiError::BadRequest);
        };
        if !is_flow_id(&r.flow_id) {
            return Err(ApiError::BadRequest);
        }
        let flow = self
            .passkey_flows
            .get(&r.flow_id)
            .cloned()
            .filter(|f| f.expires_at > now)
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if flow.kind != PasskeyFlowKind::Signin
            || flow.device_id != device.id
            || flow.device_proof_hash != device.proof_hash
        {
            return Err(ApiError::BadRequest);
        }
        let credential_id = b64(&credential_id);
        let index = self
            .passkeys
            .iter()
            .position(|p| p.credential_id == credential_id)
            .ok_or(ApiError::InvalidCredentials)?;
        let account_id = self.passkeys[index].account_id.clone();
        if flow.account_id.as_ref().is_some_and(|id| id != &account_id) {
            return Err(ApiError::InvalidCredentials);
        }
        let known = self.sessions.iter().any(|s| {
            s.account_id == account_id
                && s.device_id == device.id
                && s.revoked_at.is_none()
                && s.proof_hash == device.proof_hash
        });
        if !known {
            self.count(
                Rule::LoginUser,
                &format!("login-user:{account_id}"),
                now,
                1,
                10,
            )?;
        }
        // A consumed sign-in flow is never replayed.
        if flow.consumed_at.is_some() {
            return Err(ApiError::FlowExpired { gone: false });
        }
        let (rp_id, origins) = self.relying_party()?;
        let row = &self.passkeys[index];
        let count = verify_assertion(
            &authenticator_data,
            &client_data_json,
            &signature,
            &row.public_key,
            row.alg,
            &Self::expected(&rp_id, &origins, &flow.challenge),
        )
        .ok_or(ApiError::InvalidCredentials)?;
        if !counts_up(row.sign_count, count) {
            // A possible clone: the challenge is used up, as a proof's is.
            self.passkey_flows
                .get_mut(&r.flow_id)
                .expect("flow was checked")
                .consumed_at = Some(now);
            return Err(ApiError::InvalidCredentials);
        }
        // The account is checked after the signature.
        let account = self
            .accounts
            .get(&account_id)
            .ok_or(ApiError::InvalidCredentials)?;
        if account.disabled_at.is_some() {
            return Err(ApiError::AccountDisabled);
        }
        let account_info = (
            account.id.clone(),
            account.username.clone(),
            account.key_id.clone(),
            account.epoch.clone(),
            account.has_password,
        );
        let wrapped_key = self.passkeys[index].wrapped_key.clone();
        self.passkey_flows
            .get_mut(&r.flow_id)
            .expect("flow was checked")
            .consumed_at = Some(now);
        self.passkeys[index].sign_count = count;
        self.passkeys[index].last_used_at = Some(now);
        let (token, evicted_device) = self.start_session(&account_id, &device, now);
        Ok(PasskeySessionResponse {
            kind: PasskeySessionKind::Session,
            account_id: account_info.0,
            username: account_info.1,
            token,
            key_id: account_info.2,
            epoch: account_info.3,
            has_password: account_info.4,
            passkey_wrapped_key: wrapped_key,
            evicted_device,
        })
    }

    fn passkey_signup_start(
        &mut self,
        r: &PasskeySignupStartRequest,
        now: i64,
    ) -> Result<PasskeySignupStartResponse, ApiError> {
        let (rp_id, _) = self.relying_party()?;
        // signup-ip only: the route text, not the rate table's stale
        // "and sign-up start" for passkey-challenge.
        self.count(
            Rule::SignupIp,
            &format!("signup-ip:{CLIENT_IP}"),
            now,
            1,
            10,
        )?;
        let username = match check_new_username(&r.username) {
            Ok(n) => n,
            Err(UsernameError::Reserved) => return Err(ApiError::ReservedUsername),
            Err(_) => return Err(ApiError::InvalidUsername),
        };
        if !is_account_id(&r.account_id) {
            return Err(ApiError::BadRequest);
        }
        let device = Self::check_device(&r.device)?;
        if self.account_by_name(&username).is_some() {
            return Err(ApiError::UsernameTaken);
        }
        if self.accounts.contains_key(&r.account_id) {
            return Err(ApiError::AccountIdTaken);
        }
        self.check_signup_allowed(&username, now)?;
        let challenge = b64(&random::<32>());
        let flow_id = b64(&random::<32>());
        self.passkey_flows.insert(
            flow_id.clone(),
            PasskeyFlow {
                kind: PasskeyFlowKind::Signup,
                account_id: Some(r.account_id.clone()),
                username: Some(username.clone()),
                device_id: device.id,
                device_proof_hash: device.proof_hash,
                challenge: challenge.clone(),
                expires_at: now + PASSKEY_FLOW_TTL_MS,
                consumed_at: None,
                credential: None,
            },
        );
        Ok(PasskeySignupStartResponse {
            flow_id,
            challenge,
            rp_id,
            user: PasskeyUser {
                id: b64(&crate::passkey::user_handle(&r.account_id)),
                name: username.clone(),
                display_name: username,
            },
            pub_key_cred_params: vec![-7, -8, -257],
            authenticator_selection: PasskeySelection {
                resident_key: "required".into(),
                user_verification: "required".into(),
            },
            exclude_credentials: vec![],
        })
    }

    fn check_signup_allowed(&self, username: &str, now: i64) -> Result<(), ApiError> {
        let switch = self
            .settings
            .get("signup")
            .map(String::as_str)
            .unwrap_or("closed");
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
        Ok(())
    }

    fn passkey_signup_finish(
        &mut self,
        r: &PasskeySignupFinishRequest,
        now: i64,
    ) -> Result<PasskeySessionResponse, ApiError> {
        let device = Self::check_device(&r.device)?;
        if !is_flow_id(&r.flow_id) {
            return Err(ApiError::BadRequest);
        }
        let flow = self
            .passkey_flows
            .get(&r.flow_id)
            .cloned()
            .filter(|f| f.expires_at > now)
            .ok_or(ApiError::FlowExpired { gone: false })?;
        if flow.kind != PasskeyFlowKind::Signup
            || flow.device_id != device.id
            || flow.device_proof_hash != device.proof_hash
        {
            return Err(ApiError::BadRequest);
        }
        let username = match check_new_username(&r.username) {
            Ok(n) => n,
            Err(UsernameError::Reserved) => return Err(ApiError::ReservedUsername),
            Err(_) => return Err(ApiError::InvalidUsername),
        };
        if flow.username.as_deref() != Some(&username)
            || flow.account_id.as_deref() != Some(&r.account_id)
        {
            return Err(ApiError::BadRequest);
        }
        let credential_bytes =
            passkey_bytes(&r.credential_id, 16, 1023).ok_or(ApiError::BadRequest)?;
        let (Some(attestation_object), Some(client_data_json)) = (
            passkey_bytes(&r.attestation_object, 1, 16384),
            passkey_bytes(&r.client_data_json, 1, 8192),
        ) else {
            return Err(ApiError::BadRequest);
        };
        let credential_id = b64(&credential_bytes);
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key)
            || !is_hex16(&r.key_id)
            || r.passkey_wrapped_key
                .as_deref()
                .is_some_and(|w| !is_wrap(w))
        {
            return Err(ApiError::BadRequest);
        }
        if r.transports.len() > 32
            || r.transports.iter().any(|t| {
                t.is_empty()
                    || t.len() > 32
                    || !t
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    || !t.as_bytes()[0].is_ascii_lowercase()
            })
        {
            return Err(ApiError::BadRequest);
        }
        if flow.consumed_at.is_some() {
            let existing = self
                .accounts
                .get(&r.account_id)
                .filter(|a| a.username == username)
                .ok_or(ApiError::FlowExpired { gone: false })?;
            if existing.key_id != r.key_id
                || existing.recovery_wrapped_key != r.recovery_wrapped_key
                || existing.recovery_hash != sha256_hex(&recovery)
                || !self
                    .passkeys
                    .iter()
                    .any(|p| p.account_id == r.account_id && p.credential_id == credential_id)
            {
                return Err(ApiError::FlowExpired { gone: false });
            }
            let (account_id, username, key_id, epoch, has_password) = (
                existing.id.clone(),
                existing.username.clone(),
                existing.key_id.clone(),
                existing.epoch.clone(),
                existing.has_password,
            );
            let (token, evicted_device) = self.start_session(&account_id, &device, now);
            return Ok(PasskeySessionResponse {
                kind: PasskeySessionKind::Session,
                account_id,
                username,
                token,
                key_id,
                epoch,
                has_password,
                passkey_wrapped_key: None,
                evicted_device,
            });
        }
        let (rp_id, origins) = self.relying_party()?;
        let registration = verify_registration(
            &credential_bytes,
            &attestation_object,
            &client_data_json,
            &Self::expected(&rp_id, &origins, &flow.challenge),
        )
        .ok_or(ApiError::InvalidCredentials)?;
        if self
            .passkeys
            .iter()
            .any(|p| p.credential_id == credential_id)
        {
            return Err(ApiError::CredentialExists);
        }
        if self.account_by_name(&username).is_some() {
            return Err(ApiError::UsernameTaken);
        }
        if self.accounts.contains_key(&r.account_id) {
            return Err(ApiError::AccountIdTaken);
        }
        self.check_signup_allowed(&username, now)?;
        let epoch = hex(&random::<8>());
        let salt = b64(&random::<16>());
        self.accounts.insert(
            r.account_id.clone(),
            Account {
                id: r.account_id.clone(),
                username: username.clone(),
                kdf: self.default,
                salt,
                auth_hash: sha256_hex(&random::<32>()),
                wrapped_key: b64(&random::<73>()),
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
                has_password: false,
            },
        );
        let transports = transports_value(&r.transports);
        self.passkeys.push(PasskeyRow {
            credential_id,
            account_id: r.account_id.clone(),
            public_key: registration.public_key,
            alg: registration.alg,
            sign_count: registration.sign_count,
            transports,
            wrapped_key: r.passkey_wrapped_key.clone(),
            created_at: now,
            last_used_at: Some(now),
        });
        self.signups.add(now, 1);
        self.passkey_flows
            .get_mut(&r.flow_id)
            .expect("flow was checked")
            .consumed_at = Some(now);
        let (token, evicted_device) = self.start_session(&r.account_id, &device, now);
        Ok(PasskeySessionResponse {
            kind: PasskeySessionKind::Session,
            account_id: r.account_id.clone(),
            username,
            token,
            key_id: r.key_id.clone(),
            epoch,
            has_password: false,
            // Fresh sign-up already holds the data key locally; the Worker does
            // not echo the wrap on this answer.
            passkey_wrapped_key: None,
            evicted_device,
        })
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
                has_password: true,
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
            stored.has_password = true;
            // Recovery is the break-glass path: every passkey and its wrap go.
            self.passkeys.retain(|p| p.account_id != a.id);
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
        self.count_session(Rule::PasswordAccount, caller, now)?;
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
        if !self.accounts[&caller.account_id].has_password {
            return Err(ApiError::NoPassword);
        }
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
        let spend = self.proof_or_password(
            &caller,
            r.proof.as_ref(),
            r.current_auth_key.as_deref(),
            r.current_recovery_auth_key.as_deref(),
            now,
        )?;
        let recovery = key32(&r.recovery_auth_key).ok_or(ApiError::BadRequest)?;
        if !is_wrap(&r.recovery_wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        self.spend(spend, now);
        let stored = self
            .accounts
            .get_mut(&caller.account_id)
            .expect("authenticated");
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
        let spend = self.proof_or_password(
            &caller,
            r.proof.as_ref(),
            r.auth_key.as_deref(),
            r.current_recovery_auth_key.as_deref(),
            now,
        )?;
        // After the proof, which a mismatch leaves unused.
        let account = &self.accounts[&caller.account_id];
        if normalize_username(&r.username).ok().as_deref() != Some(account.username.as_str()) {
            return Err(ApiError::UsernameMismatch);
        }
        self.spend(spend, now);
        let id = caller.account_id;
        self.docs.retain(|(account, _), _| *account != id);
        self.files.retain(|(account, _), _| *account != id);
        self.sessions.retain(|s| s.account_id != id);
        self.approvals.retain(|a| a.account_id != id);
        self.identities.retain(|(account, _, _, _)| *account != id);
        self.passkeys.retain(|p| p.account_id != id);
        self.passkey_flows
            .retain(|_, f| f.account_id.as_deref() != Some(id.as_str()));
        self.provider_flows
            .retain(|_, f| f.account_id.as_deref() != Some(id.as_str()));
        for rule in [
            "login-user",
            "password-account",
            "proof-account",
            "passkey-manage",
            "writes-account",
            "uploads-account",
            "approval-create",
            "approval-poll",
            "approval-respond",
        ] {
            self.limits.remove(&format!("{rule}:{id}"));
        }
        // The per-device proof rows and their totals.
        self.limits.retain(|key, _| {
            !["password-account", "proof-account"]
                .iter()
                .any(|rule| key.starts_with(&format!("{rule}:{id}:")))
                && key != &format!("password-total:{id}")
                && key != &format!("proof-total:{id}")
        });
        self.accounts.remove(&id);
        Ok(DeleteResponse { deleted: true })
    }

    // -- proofs and the account's ways in ---------------------------------
    //
    // As the Worker's proofs.ts: a proof is checked first and spent only with
    // the change it authorizes ([`ServerState::spend`]), except a passkey
    // proof whose assertion failed, which uses its challenge up. No proof or
    // a kind the change doesn't take is 401 `reauth_required`; an unknown,
    // used, expired or another session's flow is 400 `flow_expired`; a wrong
    // password or assertion is 401 `invalid_credentials`.

    /// The session's device proof hash.
    fn session_proof_hash(&self, caller: &Caller) -> Option<String> {
        self.sessions
            .iter()
            .find(|s| s.token_hash == caller.token_hash)
            .map(|s| s.proof_hash.clone())
    }

    /// Whether a provider proof flow has finished at the provider, and the
    /// identity it proved is this account's.
    fn provider_proof_linked(&self, account_id: &str, f: &ProviderFlow) -> Option<bool> {
        match &f.github {
            Some(github) => {
                let subject = github.subject.as_ref()?;
                Some(
                    self.identities
                        .iter()
                        .any(|(a, p, s, _)| a == account_id && *p == f.provider && s == subject),
                )
            }
            None => f.consumed.then(|| {
                self.identities
                    .iter()
                    .any(|(a, p, _, _)| a == account_id && *p == f.provider)
            }),
        }
    }

    fn check_proof(
        &mut self,
        caller: &Caller,
        proof: Option<&ProofWire>,
        rules: ProofRules,
        now: i64,
    ) -> Result<Spend, ApiError> {
        match proof {
            None | Some(ProofWire::Unknown) => Err(ApiError::ReauthRequired),
            Some(ProofWire::Password { auth_key }) => {
                if !rules.password_counted {
                    self.count_session(Rule::PasswordAccount, caller, now)?;
                }
                let key = key32(auth_key).ok_or(ApiError::BadRequest)?;
                let account = &self.accounts[&caller.account_id];
                if account.auth_hash != sha256_hex(&key) || !account.has_password {
                    return Err(ApiError::InvalidCredentials);
                }
                Ok(Spend::Nothing)
            }
            Some(ProofWire::Passkey { flow, assertion }) => {
                let (rp_id, origins) = self.relying_party()?;
                if !is_flow_id(flow) {
                    return Err(ApiError::ReauthRequired);
                }
                let (Some(credential), Some(data), Some(client_data), Some(signature)) = (
                    passkey_bytes(&assertion.credential_id, 16, 1023),
                    passkey_bytes(&assertion.authenticator_data, 37, 8192),
                    passkey_bytes(&assertion.client_data_json, 1, 8192),
                    passkey_bytes(&assertion.signature, 1, 1024),
                ) else {
                    return Err(ApiError::BadRequest);
                };
                if rules.count_proof {
                    self.count_session(Rule::ProofAccount, caller, now)?;
                }
                let proof_hash = self.session_proof_hash(caller);
                let f = self
                    .passkey_flows
                    .get(flow)
                    .filter(|f| {
                        f.kind == PasskeyFlowKind::Proof
                            && f.account_id.as_deref() == Some(caller.account_id.as_str())
                            && f.consumed_at.is_none()
                            && f.expires_at > now
                            && f.device_id == caller.device_id
                            && Some(&f.device_proof_hash) == proof_hash.as_ref()
                    })
                    .cloned()
                    .ok_or(ApiError::FlowExpired { gone: false })?;
                let credential = b64(&credential);
                let verified = f
                    .credential
                    .as_ref()
                    .is_none_or(|only| *only == credential)
                    .then(|| {
                        self.passkeys.iter().find(|p| {
                            p.credential_id == credential && p.account_id == caller.account_id
                        })
                    })
                    .flatten()
                    .and_then(|row| {
                        verify_assertion(
                            &data,
                            &client_data,
                            &signature,
                            &row.public_key,
                            row.alg,
                            &Self::expected(&rp_id, &origins, &f.challenge),
                        )
                        .filter(|count| counts_up(row.sign_count, *count))
                    });
                let Some(count) = verified else {
                    // One try per challenge.
                    self.passkey_flows
                        .get_mut(flow)
                        .expect("flow was checked")
                        .consumed_at = Some(now);
                    return Err(ApiError::InvalidCredentials);
                };
                Ok(Spend::Passkey {
                    flow: flow.clone(),
                    credential,
                    count,
                })
            }
            Some(ProofWire::Provider { flow }) => {
                if matches!(rules.provider, ProviderRule::None) || !is_flow_id(flow) {
                    return Err(ApiError::ReauthRequired);
                }
                if rules.count_proof {
                    self.count_session(Rule::ProofAccount, caller, now)?;
                }
                // Bound to this session's device and its proof, as a passkey
                // proof's flow is; another session's is unknown.
                let proof_hash = self.session_proof_hash(caller);
                let f = self
                    .provider_flows
                    .get(flow)
                    .filter(|f| {
                        f.account_id.as_deref() == Some(caller.account_id.as_str())
                            && !f.proof_used
                            && !f.ended
                            && f.expires_at > now
                            && f.intent == Some(IdentityIntent::Proof)
                            && f.device_id == caller.device_id
                            && Some(&f.proof_hash) == proof_hash.as_ref()
                    })
                    .cloned()
                    .ok_or(ApiError::FlowExpired { gone: false })?;
                // A provider proof acts only on its own provider, unless the
                // change takes any.
                if let ProviderRule::Only(p) = rules.provider
                    && f.provider != *p
                {
                    return Err(ApiError::ReauthRequired);
                }
                // Not finished at the provider, or not this account's identity.
                if self.provider_proof_linked(&caller.account_id, &f) != Some(true) {
                    return Err(ApiError::ReauthRequired);
                }
                Ok(Spend::Provider(flow.clone()))
            }
        }
    }

    /// Uses a checked proof, with the change it authorized.
    fn spend(&mut self, spend: Spend, now: i64) {
        match spend {
            Spend::Nothing => {}
            Spend::Passkey {
                flow,
                credential,
                count,
            } => {
                if let Some(f) = self.passkey_flows.get_mut(&flow) {
                    f.consumed_at = Some(now);
                }
                if let Some(row) = self
                    .passkeys
                    .iter_mut()
                    .find(|p| p.credential_id == credential)
                {
                    row.sign_count = count;
                    row.last_used_at = Some(now);
                }
            }
            Spend::Provider(flow) => {
                if let Some(f) = self.provider_flows.get_mut(&flow) {
                    f.proof_used = true;
                }
            }
        }
    }

    /// The body's recovery key, when it has one: its hash when it is the
    /// account's, 400 malformed, 401 `invalid_credentials` wrong.
    fn recovery_proof(
        &self,
        caller: &Caller,
        given: Option<&str>,
    ) -> Result<Option<String>, ApiError> {
        let Some(given) = given else {
            return Ok(None);
        };
        let key = key32(given).ok_or(ApiError::BadRequest)?;
        let hash = sha256_hex(&key);
        if self.accounts[&caller.account_id].recovery_hash != hash {
            return Err(ApiError::InvalidCredentials);
        }
        Ok(Some(hash))
    }

    /// A proof for a change that also takes the recovery key: a provider
    /// proof needs `current_recovery_auth_key` (401 `reauth_required`
    /// without it), and a given one is always checked.
    fn proof_with_recovery(
        &mut self,
        caller: &Caller,
        proof: Option<&ProofWire>,
        recovery: Option<&str>,
        password_counted: bool,
        count_proof: bool,
        now: i64,
    ) -> Result<Spend, ApiError> {
        let recovery = self.recovery_proof(caller, recovery)?;
        if matches!(proof, Some(ProofWire::Provider { .. })) && recovery.is_none() {
            return Err(ApiError::ReauthRequired);
        }
        self.check_proof(
            caller,
            proof,
            ProofRules {
                provider: ProviderRule::Any,
                count_proof,
                password_counted,
            },
            now,
        )
    }

    /// The recovery-key and delete routes: a proof, or for a password
    /// account its auth key in the older field (counting `password-account`).
    fn proof_or_password(
        &mut self,
        caller: &Caller,
        proof: Option<&ProofWire>,
        auth_key: Option<&str>,
        recovery: Option<&str>,
        now: i64,
    ) -> Result<Spend, ApiError> {
        if proof.is_some() {
            return self.proof_with_recovery(caller, proof, recovery, false, true, now);
        }
        let current = self.current_password(caller, auth_key.ok_or(ApiError::BadRequest)?, now)?;
        if self.accounts[&caller.account_id].auth_hash != current {
            return Err(ApiError::InvalidCredentials);
        }
        Ok(Spend::Nothing)
    }

    /// The account's credentials, in the order added.
    fn account_passkeys(&self, account_id: &str) -> Vec<&PasskeyRow> {
        let mut rows: Vec<&PasskeyRow> = self
            .passkeys
            .iter()
            .filter(|p| p.account_id == account_id)
            .collect();
        rows.sort_by(|a, b| {
            (a.created_at, &a.credential_id).cmp(&(b.created_at, &b.credential_id))
        });
        rows
    }

    fn allowed(rows: &[&PasskeyRow]) -> Vec<PasskeyAllowedCredential> {
        rows.iter()
            .map(|p| PasskeyAllowedCredential {
                id: p.credential_id.clone(),
                transports: p
                    .transports
                    .split_ascii_whitespace()
                    .map(str::to_owned)
                    .collect(),
            })
            .collect()
    }

    fn passkey_proof_start(
        &mut self,
        token: &SessionToken,
        r: &PasskeyProofStartRequest,
        now: i64,
    ) -> Result<PasskeyProofStartResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let (rp_id, _) = self.relying_party()?;
        self.count_session(Rule::ProofAccount, &caller, now)?;
        let only = match &r.credential_id {
            Some(id) => Some(b64(
                &passkey_bytes(id, 16, 1023).ok_or(ApiError::BadRequest)?
            )),
            None => None,
        };
        let rows: Vec<&PasskeyRow> = self
            .account_passkeys(&caller.account_id)
            .into_iter()
            .filter(|p| only.as_ref().is_none_or(|id| *id == p.credential_id))
            .collect();
        if only.is_some() && rows.is_empty() {
            return Err(ApiError::NotFound);
        }
        let allow_credentials = Self::allowed(&rows);
        let proof_hash = self.session_proof_hash(&caller).unwrap_or_default();
        let challenge = b64(&random::<32>());
        let flow_id = b64(&random::<32>());
        self.passkey_flows.insert(
            flow_id.clone(),
            PasskeyFlow {
                kind: PasskeyFlowKind::Proof,
                account_id: Some(caller.account_id),
                username: None,
                device_id: caller.device_id,
                device_proof_hash: proof_hash,
                challenge: challenge.clone(),
                expires_at: now + PASSKEY_FLOW_TTL_MS,
                consumed_at: None,
                credential: only,
            },
        );
        Ok(PasskeyProofStartResponse {
            flow_id,
            challenge,
            rp_id,
            allow_credentials,
            user_verification: "required".into(),
            timeout: PASSKEY_FLOW_TTL_MS as u64,
        })
    }

    fn account_passkeys_start(
        &mut self,
        token: &SessionToken,
        r: &AccountPasskeysStartRequest,
        now: i64,
    ) -> Result<AccountPasskeysStartResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let (rp_id, _) = self.relying_party()?;
        self.count(
            Rule::PasskeyManage,
            &format!("passkey-manage:{}", caller.account_id),
            now,
            1,
            20,
        )?;
        // A provider proof can't add a way in.
        let spend = self.check_proof(&caller, Some(&r.proof), ProofRules::STRONG, now)?;
        self.spend(spend, now);
        let username = self.accounts[&caller.account_id].username.clone();
        let exclude_credentials = Self::allowed(&self.account_passkeys(&caller.account_id));
        let proof_hash = self.session_proof_hash(&caller).unwrap_or_default();
        let challenge = b64(&random::<32>());
        let flow_id = b64(&random::<32>());
        self.passkey_flows.insert(
            flow_id.clone(),
            PasskeyFlow {
                kind: PasskeyFlowKind::Register,
                account_id: Some(caller.account_id.clone()),
                username: Some(username.clone()),
                device_id: caller.device_id.clone(),
                device_proof_hash: proof_hash,
                challenge: challenge.clone(),
                expires_at: now + PASSKEY_FLOW_TTL_MS,
                consumed_at: None,
                credential: None,
            },
        );
        Ok(AccountPasskeysStartResponse {
            flow_id,
            challenge,
            rp_id,
            user: Some(PasskeyUser {
                id: b64(&crate::passkey::user_handle(&caller.account_id)),
                name: username.clone(),
                display_name: username,
            }),
            pub_key_cred_params: vec![-7, -8, -257],
            authenticator_selection: PasskeySelection {
                resident_key: "required".into(),
                user_verification: "required".into(),
            },
            exclude_credentials,
        })
    }

    fn account_passkeys_finish(
        &mut self,
        token: &SessionToken,
        r: &AccountPasskeysFinishRequest,
        now: i64,
    ) -> Result<AccountPasskeysFinishResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let (rp_id, origins) = self.relying_party()?;
        self.count(
            Rule::PasskeyManage,
            &format!("passkey-manage:{}", caller.account_id),
            now,
            1,
            20,
        )?;
        if !is_flow_id(&r.flow_id) {
            return Err(ApiError::BadRequest);
        }
        let credential_bytes =
            passkey_bytes(&r.credential_id, 16, 1023).ok_or(ApiError::BadRequest)?;
        let (Some(attestation_object), Some(client_data_json)) = (
            passkey_bytes(&r.attestation_object, 1, 16384),
            passkey_bytes(&r.client_data_json, 1, 8192),
        ) else {
            return Err(ApiError::BadRequest);
        };
        if bad_transports(&r.transports)
            || r.passkey_wrapped_key
                .as_deref()
                .is_some_and(|w| !is_wrap(w))
        {
            return Err(ApiError::BadRequest);
        }
        let proof_hash = self.session_proof_hash(&caller);
        // Another session's flow is answered as an unknown one.
        let flow = self
            .passkey_flows
            .get(&r.flow_id)
            .filter(|f| {
                f.kind == PasskeyFlowKind::Register
                    && f.account_id.as_deref() == Some(caller.account_id.as_str())
                    && f.device_id == caller.device_id
                    && Some(&f.device_proof_hash) == proof_hash.as_ref()
                    && f.expires_at > now
            })
            .cloned()
            .ok_or(ApiError::FlowExpired { gone: false })?;
        let credential_id = b64(&credential_bytes);
        if let Some(consumed_at) = flow.consumed_at {
            // A lost answer: the credential this flow stored, when it is this one.
            return self
                .passkeys
                .iter()
                .find(|p| {
                    p.credential_id == credential_id
                        && p.account_id == caller.account_id
                        && p.created_at == consumed_at
                })
                .map(|p| AccountPasskeysFinishResponse {
                    credential_id: p.credential_id.clone(),
                    prf: p.wrapped_key.is_some(),
                    created_at: p.created_at,
                })
                .ok_or(ApiError::FlowExpired { gone: false });
        }
        let registration = verify_registration(
            &credential_bytes,
            &attestation_object,
            &client_data_json,
            &Self::expected(&rp_id, &origins, &flow.challenge),
        )
        .ok_or(ApiError::InvalidCredentials)?;
        // The flow is used even when the credential is refused.
        self.passkey_flows
            .get_mut(&r.flow_id)
            .expect("flow was checked")
            .consumed_at = Some(now);
        if self
            .passkeys
            .iter()
            .any(|p| p.credential_id == credential_id)
        {
            return Err(ApiError::CredentialExists);
        }
        self.passkeys.push(PasskeyRow {
            credential_id: credential_id.clone(),
            account_id: caller.account_id.clone(),
            public_key: registration.public_key,
            alg: registration.alg,
            sign_count: registration.sign_count,
            transports: transports_value(&r.transports),
            wrapped_key: r.passkey_wrapped_key.clone(),
            created_at: now,
            last_used_at: None,
        });
        Ok(AccountPasskeysFinishResponse {
            credential_id,
            prf: r.passkey_wrapped_key.is_some(),
            created_at: now,
        })
    }

    /// Whether `account_id` would have no password, identity or passkey
    /// left without `credential` (a passkey) or `provider`.
    fn last_method(
        &self,
        account_id: &str,
        credential: Option<&str>,
        provider: Option<&Provider>,
    ) -> bool {
        let account = &self.accounts[account_id];
        // Only the ways in this server offers count, as the Worker's
        // offeredMethods: an identity of a provider it doesn't serve, or a
        // passkey where it offers none, can't sign in.
        let passkeys = self
            .passkeys
            .iter()
            .filter(|p| {
                self.passkey_rp.is_some()
                    && p.account_id == account_id
                    && Some(p.credential_id.as_str()) != credential
            })
            .count();
        let identities = self
            .identities
            .iter()
            .filter(|(id, p, _, _)| {
                id == account_id && Some(p) != provider && self.providers.contains(p)
            })
            .count();
        !account.has_password && passkeys == 0 && identities == 0
    }

    fn account_passkey_remove(
        &mut self,
        token: &SessionToken,
        credential_id: &str,
        r: &PasskeyProofRequest,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        self.count(
            Rule::PasskeyManage,
            &format!("passkey-manage:{}", caller.account_id),
            now,
            1,
            20,
        )?;
        let credential = b64(&passkey_bytes(credential_id, 16, 1023).ok_or(ApiError::BadRequest)?);
        let spend = self.check_proof(&caller, Some(&r.proof), ProofRules::STRONG, now)?;
        let recovery = self.recovery_proof(&caller, r.recovery_auth_key.as_deref())?;
        let index = self
            .passkeys
            .iter()
            .position(|p| p.credential_id == credential && p.account_id == caller.account_id)
            .ok_or(ApiError::NotFound)?;
        // The proof is used even when the removal is refused.
        self.spend(spend, now);
        if recovery.is_none() && self.last_method(&caller.account_id, Some(&credential), None) {
            return Err(ApiError::LastMethod);
        }
        self.passkeys.remove(index);
        Ok(())
    }

    fn set_first_password(
        &mut self,
        token: &SessionToken,
        r: &SignInPasswordRequest,
        now: i64,
    ) -> Result<PasswordResponse, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        self.count_session(Rule::PasswordAccount, &caller, now)?;
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
        if account.has_password {
            // A retry whose answer was lost: everything it sets is set.
            if account.auth_hash == auth_hash
                && account.wrapped_key == r.wrapped_key
                && account.recovery_hash == recovery_hash
                && account.recovery_wrapped_key == r.recovery_wrapped_key
            {
                return Ok(PasswordResponse {
                    signed_out_devices: 0,
                });
            }
            return Err(ApiError::AlreadyHasPassword);
        }
        let spend = self.proof_with_recovery(
            &caller,
            Some(&r.proof),
            r.current_recovery_auth_key.as_deref(),
            true,
            false,
            now,
        )?;
        self.spend(spend, now);
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
        stored.has_password = true;
        Ok(PasswordResponse { signed_out_devices })
    }

    fn sign_in_methods(
        &mut self,
        token: &SessionToken,
        now: i64,
    ) -> Result<MethodsReply, ApiError> {
        let c = self.authenticate(token, now, false)?;
        let a = self.accounts.get(&c.account_id).ok_or(ApiError::NotFound)?;
        let mut identities: Vec<IdentityRow> = self
            .identities
            .iter()
            .filter(|(id, _, _, _)| id == &c.account_id)
            .map(|(_, provider, _, created_at)| IdentityRow {
                provider: provider.clone(),
                created_at: *created_at,
            })
            .collect();
        identities.sort_by(|x, y| {
            (x.created_at, x.provider.as_str()).cmp(&(y.created_at, y.provider.as_str()))
        });
        Ok(MethodsReply {
            username: a.username.clone(),
            has_password: a.has_password,
            identities,
            passkeys: self
                .account_passkeys(&c.account_id)
                .into_iter()
                .map(|p| crate::provider::PasskeyRow {
                    credential_id: p.credential_id.clone(),
                    created_at: p.created_at,
                    // Read to the hour, but not before it was added, as the
                    // Worker answers it.
                    last_used_at: p
                        .last_used_at
                        .map(|t| (t - t.rem_euclid(HOUR_MS)).max(p.created_at)),
                    prf: p.wrapped_key.is_some(),
                    transports: p
                        .transports
                        .split_ascii_whitespace()
                        .map(str::to_owned)
                        .collect(),
                    aaguid: None,
                })
                .collect(),
            recovery: true,
        })
    }

    /// The approval's state, as the Worker's `approvalStatus`: cancelled,
    /// expired, approved, challenged or waiting.
    fn approval_status(row: &ApprovalRowState, now: i64) -> &'static str {
        if row.cancelled_at.is_some() {
            "cancelled"
        } else if row.expires_at <= now {
            "expired"
        } else if row.completed_at.is_some() {
            "approved"
        } else if row.approver_epk.is_some() {
            "challenged"
        } else {
            "waiting"
        }
    }

    fn create_approval(
        &mut self,
        token: &SessionToken,
        r: &ApprovalCreateRequest,
        now: i64,
    ) -> Result<ApprovalCreated, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_approval_id(&r.request_id) || key32(&r.commit).is_none() {
            return Err(ApiError::BadRequest);
        }
        if let Some(existing) = self.approvals.iter().find(|row| row.id == r.request_id) {
            if existing.account_id != caller.account_id
                || existing.device_id != caller.device_id
                || existing.commit_hash != r.commit
            {
                return Err(ApiError::ApprovalRequestIdConflict);
            }
            let other_devices = self
                .sessions
                .iter()
                .filter(|session| {
                    session.account_id == caller.account_id
                        && session.revoked_at.is_none()
                        && session.device_id != caller.device_id
                })
                .count();
            return Ok(ApprovalCreated {
                id: existing.id.clone(),
                expires_at: existing.expires_at,
                other_devices: other_devices as u64,
            });
        }
        self.count(
            Rule::ApprovalCreate,
            &format!("approval-create:{}", caller.account_id),
            now,
            1,
            10,
        )?;
        let account = caller.account_id.clone();
        let mut pending: Vec<usize> = self
            .approvals
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                a.account_id == account
                    && a.cancelled_at.is_none()
                    && a.completed_at.is_none()
                    && a.expires_at > now
            })
            .map(|(ix, _)| ix)
            .collect();
        pending.sort_by(|&a, &b| {
            self.approvals[a]
                .created_at
                .cmp(&self.approvals[b].created_at)
                .then_with(|| self.approvals[a].id.cmp(&self.approvals[b].id))
        });
        // Three may be pending; inserting a fourth cancels the oldest.
        for &ix in pending.iter().take(pending.len().saturating_sub(2)) {
            self.approvals[ix].cancelled_at = Some(now);
        }
        let id = r.request_id.clone();
        let expires_at = now + APPROVAL_TTL_MS;
        self.approvals.push(ApprovalRowState {
            id: id.clone(),
            account_id: account.clone(),
            device_id: caller.device_id.clone(),
            commit_hash: r.commit.clone(),
            epk: None,
            approver_device_id: None,
            approver_epk: None,
            sealed_name: None,
            wrapped_key: None,
            created_at: now,
            expires_at,
            completed_at: None,
            cancelled_at: None,
        });
        let other_devices = self
            .sessions
            .iter()
            .filter(|s| {
                s.account_id == account && s.revoked_at.is_none() && s.device_id != caller.device_id
            })
            .count();
        Ok(ApprovalCreated {
            id,
            expires_at,
            other_devices: other_devices as u64,
        })
    }

    fn list_approvals(&mut self, token: &SessionToken, now: i64) -> Result<ApprovalList, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let mut rows: Vec<&ApprovalRowState> = self
            .approvals
            .iter()
            .filter(|a| {
                a.account_id == caller.account_id
                    && a.device_id != caller.device_id
                    && a.cancelled_at.is_none()
                    && a.completed_at.is_none()
                    && a.expires_at > now
            })
            .collect();
        rows.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(ApprovalList {
            approvals: rows
                .into_iter()
                .map(|row| ApprovalRow {
                    id: row.id.clone(),
                    device_id: row.device_id.clone(),
                    commit: row.commit_hash.clone(),
                    epk: row.epk.clone(),
                    created_at: row.created_at,
                    expires_at: row.expires_at,
                    approver_epk: row.approver_epk.clone(),
                    approver_device_id: row.approver_device_id.clone(),
                    sealed_name: row.sealed_name.clone(),
                    status: Self::approval_status(row, now).to_owned(),
                })
                .collect(),
        })
    }

    fn poll_approval(
        &mut self,
        token: &SessionToken,
        id: &str,
        now: i64,
    ) -> Result<ApprovalPoll, ApiError> {
        let caller = self.authenticate(token, now, false)?;
        self.count(
            Rule::ApprovalPoll,
            &format!("approval-poll:{}", caller.account_id),
            now,
            1,
            900,
        )?;
        if !is_approval_id(id) {
            return Err(ApiError::BadRequest);
        }
        let row = self
            .approvals
            .iter()
            .find(|a| a.id == id && a.account_id == caller.account_id)
            .filter(|a| a.device_id == caller.device_id)
            .ok_or(ApiError::NotFound)?;
        let status = Self::approval_status(row, now);
        let mut answer = ApprovalPoll {
            status: status.to_owned(),
            expires_at: Some(row.expires_at),
            approver_epk: None,
            approver_device_id: None,
            sealed_name: None,
            wrapped_key: None,
        };
        if matches!(status, "challenged" | "approved") {
            answer.approver_epk = row.approver_epk.clone();
            answer.approver_device_id = row.approver_device_id.clone();
            answer.sealed_name = row.sealed_name.clone();
            answer.wrapped_key = row.wrapped_key.clone();
        }
        Ok(answer)
    }

    fn challenge_approval(
        &mut self,
        token: &SessionToken,
        id: &str,
        r: &ApprovalChallengeRequest,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_approval_id(id) {
            return Err(ApiError::BadRequest);
        }
        let ix = self.respond_row(&caller.account_id, id, now)?;
        if key32(&r.epk).is_none() {
            return Err(ApiError::BadRequest);
        }
        let row = &mut self.approvals[ix];
        if row.device_id == caller.device_id {
            return Err(ApiError::SelfApproval);
        }
        if row.expires_at <= now {
            return Err(ApiError::ApprovalExpired);
        }
        match &row.approver_epk {
            None => {
                row.approver_device_id = Some(caller.device_id);
                row.approver_epk = Some(r.epk.clone());
                Ok(())
            }
            Some(epk)
                if row.approver_device_id.as_deref() == Some(caller.device_id.as_str())
                    && epk == &r.epk =>
            {
                Ok(())
            }
            Some(_) => Err(ApiError::ApprovalUsed),
        }
    }

    fn name_approval(
        &mut self,
        token: &SessionToken,
        id: &str,
        r: &ApprovalNameRequest,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_approval_id(id) {
            return Err(ApiError::BadRequest);
        }
        let ix = self.respond_row(&caller.account_id, id, now)?;
        let epk = key32(&r.epk).ok_or(ApiError::BadRequest)?;
        if !is_sealed_name(&r.sealed_name) {
            return Err(ApiError::BadRequest);
        }
        let row = &mut self.approvals[ix];
        if row.cancelled_at.is_some() {
            return Err(ApiError::NotFound);
        }
        if row.device_id != caller.device_id {
            return Err(ApiError::NotFound);
        }
        if row.expires_at <= now {
            return Err(ApiError::ApprovalExpired);
        }
        if row.approver_epk.is_none() {
            return Err(ApiError::BadRequest);
        }
        if approval_commit(&epk) != row.commit_hash {
            return Err(ApiError::BadRequest);
        }
        match &row.epk {
            None => {
                row.epk = Some(r.epk.clone());
                row.sealed_name = Some(r.sealed_name.clone());
                Ok(())
            }
            Some(stored)
                if stored == &r.epk && row.sealed_name.as_deref() == Some(&r.sealed_name) =>
            {
                Ok(())
            }
            Some(_) => Err(ApiError::ApprovalUsed),
        }
    }

    fn approve_approval(
        &mut self,
        token: &SessionToken,
        id: &str,
        r: &ApprovalApproveRequest,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_approval_id(id) {
            return Err(ApiError::BadRequest);
        }
        let ix = self.respond_row(&caller.account_id, id, now)?;
        if !is_wrap(&r.wrapped_key) {
            return Err(ApiError::BadRequest);
        }
        let row = &mut self.approvals[ix];
        if row.cancelled_at.is_some() {
            return Err(ApiError::NotFound);
        }
        if row.device_id == caller.device_id {
            return Err(ApiError::SelfApproval);
        }
        if row.expires_at <= now {
            return Err(ApiError::ApprovalExpired);
        }
        if row.approver_device_id.as_deref() != Some(caller.device_id.as_str()) {
            return Err(ApiError::ApprovalUsed);
        }
        if row.epk.is_none() {
            return Err(ApiError::BadRequest);
        }
        match &row.wrapped_key {
            None => {
                row.wrapped_key = Some(r.wrapped_key.clone());
                row.completed_at = Some(now);
                Ok(())
            }
            Some(stored) if stored == &r.wrapped_key => Ok(()),
            Some(_) => Err(ApiError::ApprovalUsed),
        }
    }

    /// The account's live (not cancelled) request `id` for an answer, then
    /// the `approval-respond` count, as the Worker: an answer to a request
    /// the account doesn't have spends nothing.
    fn respond_row(&mut self, account_id: &str, id: &str, now: i64) -> Result<usize, ApiError> {
        let ix = self
            .approvals
            .iter()
            .position(|a| a.id == id && a.account_id == account_id && a.cancelled_at.is_none())
            .ok_or(ApiError::NotFound)?;
        self.count(
            Rule::ApprovalRespond,
            &format!("approval-respond:{account_id}"),
            now,
            1,
            30,
        )?;
        Ok(ix)
    }

    fn cancel_approval(
        &mut self,
        token: &SessionToken,
        id: &str,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        if !is_approval_id(id) {
            return Err(ApiError::BadRequest);
        }
        let Some(ix) = self
            .approvals
            .iter()
            .position(|a| a.id == id && a.account_id == caller.account_id)
        else {
            return Err(ApiError::NotFound);
        };
        let row = &mut self.approvals[ix];
        // Only the requester or the device that challenged, as the Worker.
        let participant = row.device_id == caller.device_id
            || row.approver_device_id.as_deref() == Some(caller.device_id.as_str());
        if !participant {
            return Err(ApiError::NotFound);
        }
        if row.cancelled_at.is_some() {
            return Ok(());
        }
        if row.completed_at.is_some() {
            if row.device_id != caller.device_id {
                return Err(ApiError::ApprovalUsed);
            }
            self.approvals.remove(ix);
            return Ok(());
        }
        row.cancelled_at = Some(now);
        Ok(())
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
            row.push = None;
            found = true;
        }
        if found {
            Ok(())
        } else {
            Err(ApiError::NotFound)
        }
    }

    /// The caller's own active session row; another device's id is a 404, so
    /// a session can't probe or change its siblings' push settings.
    fn own_session(
        &mut self,
        caller: &Caller,
        device_id: &str,
    ) -> Result<&mut SessionRow, ApiError> {
        if !is_device_id(device_id) {
            return Err(ApiError::BadRequest);
        }
        if device_id != caller.device_id {
            return Err(ApiError::NotFound);
        }
        self.sessions
            .iter_mut()
            .find(|s| s.token_hash == caller.token_hash && s.revoked.is_none())
            .ok_or(ApiError::NotFound)
    }

    fn put_push(
        &mut self,
        token: &SessionToken,
        device_id: &str,
        registration: &PushRegistration,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        let row = self.own_session(&caller, device_id)?;
        if !registration.is_valid() {
            return Err(ApiError::BadRequest);
        }
        row.push = Some(registration.clone());
        Ok(())
    }

    fn delete_push(
        &mut self,
        token: &SessionToken,
        device_id: &str,
        now: i64,
    ) -> Result<(), ApiError> {
        let caller = self.authenticate(token, now, false)?;
        self.own_session(&caller, device_id)?.push = None;
        Ok(())
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

    /// Configures provider routes and the health advertisement.
    /// The relying party, as the Worker reads `PASSKEY_RP_ID` and
    /// `PASSKEY_ORIGINS` (a JSON array): `None` for either, an unusable RP ID
    /// or any origin that isn't `https://` (or `http://localhost`) under it
    /// leaves the server without passkeys, as health then says.
    pub fn set_passkey_env(&self, rp_id: Option<&str>, origins_json: Option<&str>) {
        self.server().passkey_rp = passkey_advertisement(rp_id, origins_json);
    }

    /// Runs GitHub's device flow against a fake GitHub, as the shared
    /// fixtures do: device/code answers `user_code` and `interval`, each
    /// token poll takes the next [`MemoryServer::push_github_answer`] (none
    /// is `authorization_pending`), and an authorized one is the GitHub user
    /// `subject`. The Worker's rules then apply: the poll interval, sign-in
    /// or a new-account ticket by subject, linking, and proofs.
    pub fn set_github_device_flow(&self, subject: &str, user_code: &str, interval: u64) {
        self.server().github = Some(GithubFake {
            subject: subject.to_owned(),
            user_code: user_code.to_owned(),
            interval,
            answers: VecDeque::new(),
        });
    }

    /// Queues the fake GitHub's next token answer: `authorized`, `denied` or
    /// `slow_down`.
    pub fn push_github_answer(&self, answer: &str) {
        let answer = match answer {
            "authorized" => GithubAnswer::Authorized,
            "denied" => GithubAnswer::Denied,
            "slow_down" => GithubAnswer::SlowDown,
            other => panic!("unknown GitHub answer {other}"),
        };
        self.server()
            .github
            .as_mut()
            .expect("set_github_device_flow first")
            .answers
            .push_back(answer);
    }

    pub fn set_providers(&self, providers: Vec<Provider>) {
        self.server().providers = providers;
    }

    /// Scripts the first result and subsequent polling transitions for the next flow.
    /// Scripts `pending`, `slow_down`, `provider_denied`,
    /// `provider_unavailable`, `flow_expired`, `identity_taken`,
    /// `session:<account_id>` or `new_account:<username>` results.
    pub fn set_provider_outcomes<I, S>(&self, provider: Provider, outcomes: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.server().provider_outcomes.insert(
            provider.as_str().into(),
            outcomes
                .into_iter()
                .map(|value| {
                    let value = value.as_ref();
                    match value {
                        "pending" => ProviderTestOutcome::Pending,
                        "slow_down" => ProviderTestOutcome::SlowDown,
                        "provider_denied" => ProviderTestOutcome::ProviderDenied,
                        "provider_unavailable" => ProviderTestOutcome::ProviderUnavailable,
                        "flow_expired" => ProviderTestOutcome::FlowExpired,
                        "identity_taken" => ProviderTestOutcome::IdentityTaken,
                        _ if value.starts_with("session:") => ProviderTestOutcome::Session {
                            account_id: value[8..].to_owned(),
                        },
                        _ if value.starts_with("new_account:") => ProviderTestOutcome::NewAccount {
                            suggested_username: value[12..].to_owned(),
                        },
                        _ => panic!("unknown provider test outcome: {value}"),
                    }
                })
                .collect(),
        );
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
        state.end_flows_of_ended_sessions(now);
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

    /// The push token the server holds for one of the account's devices;
    /// `None` when it has none, or the session has ended.
    pub fn push_registration(&self, username: &str, device_id: &str) -> Option<PushRegistration> {
        let state = self.server();
        let id = state.account_id_of(username);
        state
            .sessions
            .iter()
            .find(|s| s.account_id == id && s.device_id == device_id)
            .and_then(|s| s.push.clone())
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
        state.end_flows_of_ended_sessions(now);
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

    fn passkey_start(
        &self,
        request: &PasskeyStartRequest,
    ) -> Result<PasskeyStartResponse, ApiError> {
        self.server
            .call(Route::PasskeyStart, |s, now| s.passkey_start(request, now))
    }

    fn passkey_finish(
        &self,
        request: &PasskeyFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.server.call(Route::PasskeyFinish, |s, now| {
            s.passkey_finish(request, now)
        })
    }

    fn passkey_signup_start(
        &self,
        request: &PasskeySignupStartRequest,
    ) -> Result<PasskeySignupStartResponse, ApiError> {
        self.server.call(Route::PasskeySignupStart, |s, now| {
            s.passkey_signup_start(request, now)
        })
    }

    fn passkey_signup_finish(
        &self,
        request: &PasskeySignupFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.server.call(Route::PasskeySignupFinish, |s, now| {
            s.passkey_signup_finish(request, now)
        })
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

    fn put_push(
        &self,
        token: &SessionToken,
        device_id: &str,
        registration: &PushRegistration,
    ) -> Result<(), ApiError> {
        self.server.call(Route::PutPush, |s, now| {
            s.put_push(token, device_id, registration, now)
        })
    }

    fn delete_push(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        self.server.call(Route::DeletePush, |s, now| {
            s.delete_push(token, device_id, now)
        })
    }

    fn create_approval(
        &self,
        token: &SessionToken,
        request: &ApprovalCreateRequest,
    ) -> Result<ApprovalCreated, ApiError> {
        self.server.call(Route::ApprovalCreate, |s, now| {
            s.create_approval(token, request, now)
        })
    }

    fn approvals(&self, token: &SessionToken) -> Result<ApprovalList, ApiError> {
        self.server
            .call(Route::ApprovalList, |s, now| s.list_approvals(token, now))
    }

    fn approval(&self, token: &SessionToken, id: &str) -> Result<ApprovalPoll, ApiError> {
        self.server.call(Route::ApprovalPoll, |s, now| {
            s.poll_approval(token, id, now)
        })
    }

    fn challenge_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalChallengeRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::ApprovalChallenge, |s, now| {
            s.challenge_approval(token, id, request, now)
        })
    }

    fn name_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalNameRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::ApprovalName, |s, now| {
            s.name_approval(token, id, request, now)
        })
    }

    fn approve_approval(
        &self,
        token: &SessionToken,
        id: &str,
        request: &ApprovalApproveRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::ApprovalApprove, |s, now| {
            s.approve_approval(token, id, request, now)
        })
    }

    fn cancel_approval(&self, token: &SessionToken, id: &str) -> Result<(), ApiError> {
        self.server.call(Route::ApprovalCancel, |s, now| {
            s.cancel_approval(token, id, now)
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

    fn provider_start(
        &self,
        p: &Provider,
        r: &ProviderStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        self.server.call(Route::ProviderStart, |s, n| {
            s.provider_start(p, &r.device, n, None, None, None)
        })
    }
    fn google_start(&self, r: &GoogleStartRequest) -> Result<ProviderStart, ApiError> {
        self.server.call(Route::GoogleStart, |s, n| {
            if crate::provider::Loopback::parse(&r.redirect_uri).is_none()
                || r.code_challenge.len() != 43
                || !is_base64url(&r.code_challenge)
            {
                return Err(ApiError::BadRequest);
            }
            s.provider_start(
                &Provider::Google,
                &r.device,
                n,
                None,
                None,
                Some(r.code_challenge.clone()),
            )
        })
    }
    fn provider_poll(
        &self,
        p: &Provider,
        r: &ProviderPollRequest,
    ) -> Result<ProviderStep, ApiError> {
        self.server.call(Route::ProviderPoll, |s, n| {
            s.provider_result(&r.flow_id, p, &r.device, n)
        })
    }
    fn google_finish(&self, r: &GoogleFinishRequest) -> Result<ProviderStep, ApiError> {
        self.server.call(Route::GoogleFinish, |s, n| {
            if r.code.is_empty()
                || !(43..=128).contains(&r.code_verifier.len())
                || !is_base64url(&r.code_verifier)
            {
                return Err(ApiError::BadRequest);
            }
            let f = s
                .provider_flows
                .get(&r.flow_id)
                .ok_or(ApiError::FlowExpired { gone: false })?;
            if f.code_challenge.as_deref()
                != Some(b64(&Sha256::digest(r.code_verifier.as_bytes())).as_str())
            {
                return Err(ApiError::BadRequest);
            }
            s.provider_result(&r.flow_id, &Provider::Google, &r.device, n)
        })
    }
    fn social_signup(&self, r: &SocialSignupRequest) -> Result<ProviderSession, ApiError> {
        self.server
            .call(Route::SocialSignup, |s, n| s.social_signup(r, n))
    }

    fn sign_in_methods(&self, t: &SessionToken) -> Result<MethodsReply, ApiError> {
        self.server
            .call(Route::SignInMethods, |s, n| s.sign_in_methods(t, n))
    }
    fn account_sign_in_methods(
        &self,
        t: &SessionToken,
    ) -> Result<AccountSignInMethodsResponse, ApiError> {
        self.server.call(Route::AccountSignInMethods, |s, n| {
            let reply = s.sign_in_methods(t, n)?;
            Ok(AccountSignInMethodsResponse {
                username: reply.username,
                has_password: reply.has_password,
                identities: reply
                    .identities
                    .into_iter()
                    .map(|row| LinkedIdentityResponse {
                        provider: row.provider.as_str().to_owned(),
                        created_at: row.created_at,
                    })
                    .collect(),
                passkeys: reply
                    .passkeys
                    .into_iter()
                    .map(|row| AccountPasskeyResponse {
                        credential_id: row.credential_id,
                        created_at: row.created_at,
                        last_used_at: row.last_used_at,
                        prf: row.prf,
                        transports: row.transports,
                        aaguid: None,
                    })
                    .collect(),
                recovery: reply.recovery,
            })
        })
    }
    fn passkey_proof_start(
        &self,
        t: &SessionToken,
        r: &PasskeyProofStartRequest,
    ) -> Result<PasskeyProofStartResponse, ApiError> {
        self.server.call(Route::PasskeyProofStart, |s, n| {
            s.passkey_proof_start(t, r, n)
        })
    }
    fn account_passkeys_start(
        &self,
        t: &SessionToken,
        r: &AccountPasskeysStartRequest,
    ) -> Result<AccountPasskeysStartResponse, ApiError> {
        self.server.call(Route::AccountPasskeysStart, |s, n| {
            s.account_passkeys_start(t, r, n)
        })
    }
    fn account_passkeys_finish(
        &self,
        t: &SessionToken,
        r: &AccountPasskeysFinishRequest,
    ) -> Result<AccountPasskeysFinishResponse, ApiError> {
        self.server.call(Route::AccountPasskeysFinish, |s, n| {
            s.account_passkeys_finish(t, r, n)
        })
    }
    fn account_passkey_remove(
        &self,
        t: &SessionToken,
        credential_id: &str,
        r: &PasskeyProofRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::AccountPasskeyRemove, |s, n| {
            s.account_passkey_remove(t, credential_id, r, n)
        })
    }
    fn set_first_password(
        &self,
        t: &SessionToken,
        r: &SignInPasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.server
            .call(Route::SignInPassword, |s, n| s.set_first_password(t, r, n))
    }
    fn identity_start(
        &self,
        t: &SessionToken,
        p: &Provider,
        r: &IdentityStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        self.server.call(Route::IdentityStart, |s, n| {
            let c = s.authenticate(t, n, false)?;
            if !matches!(p, Provider::Apple | Provider::Google | Provider::Github) {
                return Err(ApiError::UnsupportedMethod);
            }
            if !s.providers.contains(p) {
                return Err(ApiError::ProviderDisabled);
            }
            s.count_session(Rule::ProofAccount, &c, n)?;
            let row = s
                .sessions
                .iter()
                .find(|x| x.token_hash == c.token_hash)
                .ok_or(ApiError::NotFound)?;
            if row.device_id != r.device.id
                || row.proof_hash
                    != sha256_hex(&unb64(&r.device.proof).ok_or(ApiError::BadRequest)?)
            {
                return Err(ApiError::BadRequest);
            }
            // A link adds a way in: a password or passkey proof, used with
            // the flow it starts.
            let spend = match r.intent {
                IdentityIntent::Link => {
                    s.check_proof(&c, r.proof.as_ref(), ProofRules::STRONG, n)?
                }
                IdentityIntent::Proof => Spend::Nothing,
            };
            let challenge = if *p == Provider::Google {
                let uri = r
                    .redirect_uri
                    .as_deref()
                    .and_then(crate::provider::Loopback::parse)
                    .ok_or(ApiError::BadRequest)?;
                let _ = uri;
                if r.code_challenge
                    .as_deref()
                    .is_none_or(|x| x.len() != 43 || !is_base64url(x))
                {
                    return Err(ApiError::BadRequest);
                }
                r.code_challenge.clone()
            } else {
                None
            };
            let started = s.provider_start(
                p,
                &r.device,
                n,
                Some(r.intent),
                Some(c.account_id),
                challenge,
            )?;
            s.spend(spend, n);
            Ok(started)
        })
    }
    fn identity_poll(
        &self,
        t: &SessionToken,
        p: &Provider,
        r: &ProviderPollRequest,
    ) -> Result<LinkOutcome, ApiError> {
        self.server.call(Route::ProviderPoll, |s, n| {
            // The Worker's poll takes no session: the flow's device binds it.
            if !s.github_flow(&r.flow_id) {
                let c = s.authenticate(t, n, false)?;
                let f = s
                    .provider_flows
                    .get(&r.flow_id)
                    .ok_or(ApiError::FlowExpired { gone: false })?;
                if f.account_id.as_deref() != Some(&c.account_id) {
                    return Err(ApiError::NotFound);
                }
            }
            s.identity_result(&r.flow_id, p, &r.device, n)
        })
    }
    fn identity_finish_google(
        &self,
        t: &SessionToken,
        p: &Provider,
        r: &GoogleFinishRequest,
    ) -> Result<LinkOutcome, ApiError> {
        self.server.call(Route::GoogleFinish, |s, n| {
            if p != &Provider::Google
                || r.code.is_empty()
                || !(43..=128).contains(&r.code_verifier.len())
                || !is_base64url(&r.code_verifier)
            {
                return Err(ApiError::BadRequest);
            }
            let c = s.authenticate(t, n, false)?;
            let f = s
                .provider_flows
                .get(&r.flow_id)
                .ok_or(ApiError::FlowExpired { gone: false })?;
            if f.code_challenge.as_deref()
                != Some(b64(&Sha256::digest(r.code_verifier.as_bytes())).as_str())
            {
                return Err(ApiError::BadRequest);
            }
            if f.account_id.as_deref() != Some(&c.account_id) {
                return Err(ApiError::NotFound);
            }
            s.identity_result(&r.flow_id, p, &r.device, n)
        })
    }
    fn unlink_identity(
        &self,
        t: &SessionToken,
        p: &Provider,
        r: &UnlinkRequest,
    ) -> Result<(), ApiError> {
        self.server.call(Route::IdentityUnlink, |s, n| {
            let c = s.authenticate(t, n, false)?;
            let rules = ProofRules {
                provider: ProviderRule::Only(p),
                count_proof: false,
                password_counted: false,
            };
            let spend = s.check_proof(&c, Some(&r.proof), rules, n)?;
            let recovery = s.recovery_proof(&c, r.recovery_auth_key.as_deref())?;
            let i = s
                .identities
                .iter()
                .position(|(id, provider, _, _)| id == &c.account_id && provider == p)
                .ok_or(ApiError::NotFound)?;
            // The proof is used even when the unlink is refused.
            s.spend(spend, n);
            if recovery.is_none() && s.last_method(&c.account_id, None, Some(p)) {
                return Err(ApiError::LastMethod);
            }
            s.identities.remove(i);
            Ok(())
        })
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
    use crate::TestAuthenticator;
    use crate::api::LoginDevice;
    use crate::passkey::PasskeyCeremony as _;
    use proptest::prelude::*;

    fn device(id: &str) -> NewDevice {
        NewDevice {
            id: id.into(),
            sealed_name: None,
            proof: b64(&[9; 32]),
        }
    }

    /// The Worker's countSession: each device has its own `password-account`
    /// (5) and `proof-account` (20); the account's total (15, 60) is three
    /// devices' worth and is counted only once a device's own rule passes.
    #[test]
    fn proof_limits_count_per_device_then_per_account() {
        for (rule, own, total, total_name) in [
            (Rule::PasswordAccount, 5, 15, "password-total"),
            (Rule::ProofAccount, 20, 60, "proof-total"),
        ] {
            let mut state = ServerState::new(KdfParams::TEST, KdfParams::TEST);
            let caller = |n: u8| Caller {
                account_id: "a_1".into(),
                // Keyed by the ID's first 64 bits, as the Worker's.
                device_id: format!("d_{n:02x}{}", "0".repeat(30)),
                token_hash: String::new(),
            };
            // One device using up its own leaves the others theirs, and a
            // refused try doesn't reach the total.
            for _ in 0..own {
                state.count_session(rule, &caller(1), 0).unwrap();
            }
            for _ in 0..3 {
                assert!(matches!(
                    state.count_session(rule, &caller(1), 0),
                    Err(ApiError::RateLimited { ref limit, .. }) if limit == rule.name()
                ));
            }
            for _ in 0..own {
                state.count_session(rule, &caller(2), 0).unwrap();
                state.count_session(rule, &caller(3), 0).unwrap();
            }
            assert_eq!(3 * own, total);
            // A fourth device has its own allowance but the account's total is spent.
            assert!(matches!(
                state.count_session(rule, &caller(4), 0),
                Err(ApiError::RateLimited { ref limit, .. }) if limit == total_name
            ));
        }
    }

    fn signup_flow(
        api: &dyn Api,
        name: &str,
        id: &str,
        device: &NewDevice,
    ) -> PasskeySignupStartResponse {
        api.passkey_signup_start(&PasskeySignupStartRequest {
            username: name.into(),
            account_id: id.into(),
            device: device.clone(),
        })
        .unwrap()
    }

    /// A real registration over `flow`'s challenge by `auth`.
    fn signup_finish(
        flow: &PasskeySignupStartResponse,
        name: &str,
        id: &str,
        device: NewDevice,
        auth: &TestAuthenticator,
    ) -> PasskeySignupFinishRequest {
        let registration = auth
            .register(&crate::passkey::PasskeyRegistrationChallenge {
                rp_id: MEMORY_PASSKEY_RP_ID.into(),
                origin: MEMORY_PASSKEY_ORIGIN.into(),
                challenge: unb64(&flow.challenge).unwrap(),
                user_handle: crate::passkey::user_handle(id),
                username: name.into(),
                display_name: name.into(),
                exclude_credentials: vec![],
                request_prf: true,
            })
            .unwrap();
        PasskeySignupFinishRequest {
            flow_id: flow.flow_id.clone(),
            username: name.into(),
            account_id: id.into(),
            credential_id: b64(&registration.credential_id),
            attestation_object: b64(&registration.attestation_object),
            client_data_json: b64(&registration.client_data_json),
            transports: registration.transports.clone(),
            passkey_wrapped_key: Some(b64(&[7; 73])),
            recovery_auth_key: b64(&[3; 32]),
            recovery_wrapped_key: b64(&[4; 73]),
            key_id: "0011223344556677".into(),
            device,
        }
    }

    /// `auth`'s assertion over a sign-in flow, as `device`.
    fn signin_finish(
        start: &PasskeyStartResponse,
        auth: &TestAuthenticator,
        device: &NewDevice,
    ) -> PasskeyFinishRequest {
        let assertion = auth.assertion(
            MEMORY_PASSKEY_RP_ID,
            MEMORY_PASSKEY_ORIGIN,
            &unb64(&start.challenge).unwrap(),
            false,
        );
        PasskeyFinishRequest {
            flow_id: start.flow_id.clone(),
            credential_id: b64(&assertion.credential_id),
            authenticator_data: b64(&assertion.authenticator_data),
            client_data_json: b64(&assertion.client_data_json),
            signature: b64(&assertion.signature),
            user_handle: None,
            device: LoginDevice {
                id: device.id.clone(),
                proof: device.proof.clone(),
            },
        }
    }

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
    fn passkey_signup_creates_passwordless_account_and_session() {
        let server = MemoryServer::new();
        server.set_now(1_700_000_000_000);
        let api = server.api("");
        let d = device("d_00000000000000000000000000000001");
        let id = "a_00000000000000000000000000000001";
        let flow = signup_flow(api.as_ref(), "alice", id, &d);
        let auth = TestAuthenticator::new(5);
        let finish = signup_finish(&flow, "alice", id, d, &auth);
        let session = api.passkey_signup_finish(&finish).unwrap();
        assert!(!session.has_password);
        assert_eq!(session.passkey_wrapped_key, None);
        assert_eq!(api.account(&session.token).unwrap().username, "alice");
        // A lost answer is retried with the same request: the Worker answers a
        // session again rather than `flow_expired`, and sends no passkey wrap.
        let retried = api.passkey_signup_finish(&finish).unwrap();
        assert_eq!(retried.account_id, session.account_id);
        assert_eq!(retried.passkey_wrapped_key, None);
        assert!(api.account(&retried.token).is_ok());
    }

    /// The Worker's offeredMethods: for the last-method rule, only the
    /// providers this server offers count, and passkeys only when it offers
    /// them; a way in it doesn't offer can't sign in.
    #[test]
    fn the_last_method_rule_counts_only_offered_ways_in() {
        let server = MemoryServer::new();
        server.set_now(1_700_000_000_000);
        server.set_providers(vec![Provider::Github]);
        let api = server.api("");
        let d = device("d_00000000000000000000000000000001");
        let id = "a_00000000000000000000000000000001";
        let flow = signup_flow(api.as_ref(), "alice", id, &d);
        let auth = TestAuthenticator::new(5);
        api.passkey_signup_finish(&signup_finish(&flow, "alice", id, d, &auth))
            .unwrap();
        let credential = b64(auth.credential_id());
        let mut state = server.server();
        state
            .identities
            .push((id.into(), Provider::Github, "4242".into(), 0));
        // Both offered: neither is the last.
        assert!(!state.last_method(id, Some(&credential), None));
        assert!(!state.last_method(id, None, Some(&Provider::Github)));
        // GitHub no longer offered: the passkey is the last way in.
        state.providers.clear();
        assert!(state.last_method(id, Some(&credential), None));
        // Passkeys no longer offered: GitHub is the last.
        state.providers.push(Provider::Github);
        state.passkey_rp = None;
        assert!(state.last_method(id, None, Some(&Provider::Github)));
    }

    fn pdevice(n: u8) -> LoginDevice {
        LoginDevice {
            id: format!("d_{:032x}", n),
            proof: b64(&[n; 32]),
        }
    }
    fn pnew(ticket: String, name: &str, id: &str, d: &LoginDevice) -> SocialSignupRequest {
        SocialSignupRequest {
            ticket,
            username: name.into(),
            account_id: id.into(),
            recovery_auth_key: b64(&[8; 32]),
            recovery_wrapped_key: b64(&[9; 73]),
            key_id: "0011223344556677".into(),
            device: NewDevice {
                id: d.id.clone(),
                sealed_name: None,
                proof: d.proof.clone(),
            },
        }
    }

    #[test]
    fn provider_health_and_new_account_session() {
        let s = MemoryServer::new();
        s.set_now(1_700_000_000_000);
        let a = s.api("");
        assert!(a.health().unwrap().auth.unwrap().providers.is_empty());
        s.set_providers(vec![Provider::Apple]);
        s.set_provider_outcomes(Provider::Apple, ["new_account:apple-user"]);
        let d = pdevice(41);
        let start = a
            .provider_start(
                &Provider::Apple,
                &ProviderStartRequest { device: d.clone() },
            )
            .unwrap();
        let ProviderStep::NewAccount(ticket) = a
            .provider_poll(
                &Provider::Apple,
                &ProviderPollRequest {
                    flow_id: start.flow_id().into(),
                    s: None,
                    device: d.clone(),
                },
            )
            .unwrap()
        else {
            panic!("expected new account")
        };
        let session = a
            .social_signup(&pnew(
                ticket.ticket,
                "apple-user",
                "a_00000000000000000000000000000041",
                &d,
            ))
            .unwrap();
        assert!(!session.has_password);
        assert_eq!(session.passkey_wrapped_key, None);
        assert!(a.account(&session.token).is_ok());
        assert_eq!(s.devices("apple-user").len(), 1);
        let methods = a.sign_in_methods(&session.token).unwrap();
        assert!(!methods.has_password);
        assert_eq!(methods.identities.len(), 1);
    }

    #[test]
    fn provider_poll_slow_down_then_ticket() {
        let s = MemoryServer::new();
        let a = s.api("");
        s.set_providers(vec![Provider::Github]);
        s.set_provider_outcomes(
            Provider::Github,
            ["pending", "slow_down", "new_account:gh-user"],
        );
        let d = pdevice(42);
        let start = a
            .provider_start(
                &Provider::Github,
                &ProviderStartRequest { device: d.clone() },
            )
            .unwrap();
        let poll = || {
            a.provider_poll(
                &Provider::Github,
                &ProviderPollRequest {
                    flow_id: start.flow_id().into(),
                    s: None,
                    device: d.clone(),
                },
            )
            .unwrap()
        };
        assert_eq!(poll(), ProviderStep::Pending { slow_down: true });
        assert!(matches!(poll(), ProviderStep::NewAccount(_)));
    }

    #[test]
    fn provider_poll_rejects_wrong_device_and_expired_flow() {
        let s = MemoryServer::new();
        s.set_now(1_700_000_000_000);
        let a = s.api("");
        s.set_providers(vec![Provider::Apple]);
        let d = pdevice(43);
        let start = a
            .provider_start(
                &Provider::Apple,
                &ProviderStartRequest { device: d.clone() },
            )
            .unwrap();
        assert_eq!(
            a.provider_poll(
                &Provider::Apple,
                &ProviderPollRequest {
                    flow_id: start.flow_id().into(),
                    s: None,
                    device: pdevice(44)
                }
            )
            .unwrap_err(),
            ApiError::BadRequest
        );
        s.set_now(1_700_000_600_001);
        assert_eq!(
            a.provider_poll(
                &Provider::Apple,
                &ProviderPollRequest {
                    flow_id: start.flow_id().into(),
                    s: None,
                    device: d
                }
            )
            .unwrap_err(),
            ApiError::FlowExpired { gone: false }
        );
    }

    #[test]
    fn provider_existing_account_signin_creates_a_session_and_evicts_oldest() {
        let s = MemoryServer::new();
        s.set_now(1_700_000_000_000);
        let a = s.api("");
        s.set_providers(vec![Provider::Apple]);
        let initial = pdevice(45);
        s.set_provider_outcomes(Provider::Apple, ["new_account:provider-user"]);
        let st = a
            .provider_start(
                &Provider::Apple,
                &ProviderStartRequest {
                    device: initial.clone(),
                },
            )
            .unwrap();
        let ProviderStep::NewAccount(ticket) = a
            .provider_poll(
                &Provider::Apple,
                &ProviderPollRequest {
                    flow_id: st.flow_id().into(),
                    s: None,
                    device: initial.clone(),
                },
            )
            .unwrap()
        else {
            panic!()
        };
        let created = a
            .social_signup(&pnew(
                ticket.ticket,
                "provider-user",
                "a_00000000000000000000000000000045",
                &initial,
            ))
            .unwrap();
        let mut last = None;
        for n in 46..=55 {
            s.set_provider_outcomes(Provider::Apple, [format!("session:{}", created.account_id)]);
            let d = pdevice(n);
            let st = a
                .provider_start(
                    &Provider::Apple,
                    &ProviderStartRequest { device: d.clone() },
                )
                .unwrap();
            let ProviderStep::Session(ss) = a
                .provider_poll(
                    &Provider::Apple,
                    &ProviderPollRequest {
                        flow_id: st.flow_id().into(),
                        s: None,
                        device: d,
                    },
                )
                .unwrap()
            else {
                panic!()
            };
            assert!(!ss.has_password);
            assert_eq!(ss.passkey_wrapped_key, None);
            last = Some(ss)
        }
        assert_eq!(
            last.unwrap().evicted_device.unwrap().id,
            "d_0000000000000000000000000000002d"
        );
        assert_eq!(
            s.devices("provider-user")
                .iter()
                .filter(|d| d.revoked.is_none())
                .count(),
            10
        );
    }

    #[test]
    fn passkey_signup_rejects_a_flow_from_another_device() {
        let server = MemoryServer::new();
        server.set_now(1_700_000_000_000);
        let api = server.api("");
        let first = device("d_00000000000000000000000000000001");
        let second = device("d_00000000000000000000000000000002");
        let id = "a_00000000000000000000000000000001";
        let flow = signup_flow(api.as_ref(), "alice", id, &first);
        let finish = signup_finish(&flow, "alice", id, second, &TestAuthenticator::new(5));
        assert_eq!(
            api.passkey_signup_finish(&finish),
            Err(ApiError::BadRequest)
        );
    }

    #[test]
    fn passkey_signin_lists_only_typed_account_credentials_and_consumes_flow() {
        let server = MemoryServer::new();
        server.set_now(1_700_000_000_000);
        let api = server.api("");
        let d = device("d_00000000000000000000000000000002");
        let id = "a_00000000000000000000000000000002";
        let flow = signup_flow(api.as_ref(), "alice", id, &d);
        let auth = TestAuthenticator::new(6);
        let credential = b64(auth.credential_id());
        let finish = signup_finish(&flow, "alice", id, d.clone(), &auth);
        api.passkey_signup_finish(&finish).unwrap();
        let start_for = |username: &str| {
            api.passkey_start(&PasskeyStartRequest {
                username: Some(username.into()),
                device: LoginDevice {
                    id: d.id.clone(),
                    proof: d.proof.clone(),
                },
            })
            .unwrap()
        };
        let start = start_for(" Alice ");
        assert_eq!(
            start.allow_credentials,
            vec![PasskeyAllowedCredential {
                id: credential.clone(),
                transports: vec!["internal".into()]
            }]
        );
        assert!(start_for("nobody").allow_credentials.is_empty());
        // A signature over another challenge, or by another key, is refused
        // and leaves the flow usable.
        let mut forged = signin_finish(&start, &auth, &d);
        forged.signature = signin_finish(&start_for("alice"), &auth, &d).signature;
        assert_eq!(
            api.passkey_finish(&forged),
            Err(ApiError::InvalidCredentials)
        );
        let assertion = signin_finish(&start, &auth, &d);
        let session = api.passkey_finish(&assertion).unwrap();
        assert!(!session.has_password);
        assert_eq!(session.passkey_wrapped_key, finish.passkey_wrapped_key);
        assert_eq!(
            api.passkey_finish(&assertion),
            Err(ApiError::FlowExpired { gone: false })
        );
        // A count that doesn't grow is a possible clone.
        auth.set_sign_count(0);
        assert_eq!(
            api.passkey_finish(&signin_finish(&start_for("alice"), &auth, &d)),
            Err(ApiError::InvalidCredentials)
        );
    }

    #[test]
    fn passkey_finish_rejects_unknown_credentials_and_mismatched_devices() {
        let server = MemoryServer::new();
        server.set_now(1_700_000_000_000);
        let api = server.api("");
        let d = device("d_00000000000000000000000000000003");
        let start = || {
            api.passkey_start(&PasskeyStartRequest {
                username: None,
                device: LoginDevice {
                    id: d.id.clone(),
                    proof: d.proof.clone(),
                },
            })
            .unwrap()
        };
        let req = signin_finish(&start(), &TestAuthenticator::new(1), &d);
        assert_eq!(api.passkey_finish(&req), Err(ApiError::InvalidCredentials));
        let mut mismatch = signin_finish(&start(), &TestAuthenticator::new(1), &d);
        mismatch.device.proof = b64(&[8; 32]);
        assert_eq!(api.passkey_finish(&mismatch), Err(ApiError::BadRequest));
        let mut malformed = mismatch;
        malformed.credential_id = "not canonical!".into();
        assert_eq!(api.passkey_finish(&malformed), Err(ApiError::BadRequest));
    }

    #[test]
    fn health_advertises_memory_passkey_rp() {
        let health = MemoryServer::new().api("").health().unwrap();
        let auth = health.auth.unwrap();
        let rp = auth.passkeys.unwrap();
        assert_eq!(rp.rp_id, MEMORY_PASSKEY_RP_ID);
        assert_eq!(rp.origins, vec!["https://memory.kanso.test"]);
        assert!(rp.security_key);
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
