//! Tokens for agents: short-lived bearer tokens that open `/mcp` alone, read
//! only, and only the notes in their scope.
//!
//! A token is 256 bits from the OS random source, in base64url. Only its
//! SHA-256 is kept, and a token given is compared with every one kept in
//! constant time. One token per agent session, at most [`MAX_LIVE`] at once;
//! each expires [`LIFETIME`] after it was minted or [`IDLE`] after its last
//! request, whichever is sooner, and may be renewed once. Revoking one is
//! immediate. Nothing here is written anywhere: the tokens live in memory
//! and end with the app.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;

use crate::changeset::{ChangeSets, TaskId};
use crate::review::ReviewQueue;
use crate::scope::{AgentScope, Caller, TaskLimits};

/// How long a token lasts after it is minted (or renewed).
pub const LIFETIME: Duration = Duration::from_secs(15 * 60);
/// How long a token lasts after its last request.
pub const IDLE: Duration = Duration::from_secs(5 * 60);
/// The most tokens live at once; minting another revokes the oldest.
pub const MAX_LIVE: usize = 4;
/// The most tool calls one question may make with a token.
pub const MAX_CALLS_PER_TURN: usize = 40;

/// Which token, without the token itself: what the app keeps to renew,
/// revoke, or ask what the agent read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenId(u64);

impl TokenId {
    /// The ID a suggestion from API key `key` is kept under in the review
    /// queue. The top bit is set, so it never equals a session token's ID,
    /// which count up from 1.
    pub(crate) fn for_key(key: &crate::key_model::KeyId) -> TokenId {
        let digest = Sha256::digest(key.as_str().as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        TokenId(u64::from_be_bytes(bytes) | (1 << 63))
    }
}

/// A token just minted. The text is what the agent sends; it is never
/// printed.
pub struct Minted {
    pub id: TokenId,
    pub token: String,
}

impl fmt::Debug for Minted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Minted")
            .field("id", &self.id)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// Lines of one note an agent was shown, numbered as the excerpts of its
/// prompt are, so a receipt `[n:Lx]` can be checked against them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShownNote {
    pub number: usize,
    pub id: String,
    pub title: String,
    /// The day the note last changed (`2026-10-14`).
    pub updated: Option<String>,
    /// 0-based line numbers and their text as shown, in order, each once.
    pub lines: Vec<(u32, String)>,
}

struct Entry {
    id: TokenId,
    hash: [u8; 32],
    minted: Instant,
    last_used: Instant,
    renewed: bool,
    scope: Arc<AgentScope>,
    /// Every note shown since the question began, in order, once each.
    reads: Vec<String>,
    /// Tool calls since the question began.
    calls: usize,
    /// The lines shown since the question began, numbered.
    shown: Vec<ShownNote>,
    /// Between turns of an agent session: live, but every call is refused.
    paused: bool,
}

impl Entry {
    fn live_at(&self, now: Instant) -> bool {
        // A paused token waits for the next turn of its session; the app
        // ends it (idle close, End Session) or the store evicts it.
        if self.paused {
            return true;
        }
        // A task's token lives for the task's time limit and is never idle:
        // an agent may think a long while between calls.
        let (lifetime, idle) = match &self.scope.task_limits {
            Some(limits) => (limits.lifetime, limits.lifetime),
            None => (LIFETIME, IDLE),
        };
        now.saturating_duration_since(self.minted) < lifetime
            && now.saturating_duration_since(self.last_used) < idle
    }
}

type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

struct Inner {
    entries: Vec<Entry>,
    next_id: u64,
    /// Tool calls and notes shown per task, over every token the task had
    /// (an agent session that starts again gets a new token, not a new
    /// budget).
    task_calls: HashMap<TaskId, usize>,
    task_reads: HashMap<TaskId, Vec<String>>,
}

/// The agents' tokens. Cloning shares the same store: the app keeps one and
/// hands it to whichever server answers the agents.
#[derive(Clone)]
pub struct TokenStore {
    inner: Arc<Mutex<Inner>>,
    clock: Clock,
    /// The edits agents suggested. Kept beside the tokens so one handle
    /// reaches both, but not tied to them: a token ending doesn't drop a
    /// suggestion.
    reviews: ReviewQueue,
    /// The change sets of agent tasks, beside the tokens for the same
    /// reason.
    changesets: ChangeSets,
}

impl Default for TokenStore {
    fn default() -> Self {
        TokenStore::new()
    }
}

impl fmt::Debug for TokenStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenStore")
            .field("live", &self.live_count())
            .finish()
    }
}

fn hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

impl TokenStore {
    pub fn new() -> TokenStore {
        TokenStore::with_clock(Arc::new(Instant::now))
    }

    /// A store reading the time from `clock`, for tests.
    pub fn with_clock(clock: Arc<dyn Fn() -> Instant + Send + Sync>) -> TokenStore {
        TokenStore {
            reviews: ReviewQueue::with_clock(clock.clone()),
            changesets: ChangeSets::default(),
            inner: Arc::new(Mutex::new(Inner {
                entries: Vec::new(),
                next_id: 1,
                task_calls: HashMap::new(),
                task_reads: HashMap::new(),
            })),
            clock,
        }
    }

    /// The review queue for the edits agents suggest.
    pub fn reviews(&self) -> &ReviewQueue {
        &self.reviews
    }

    /// The change sets of the agent tasks.
    pub fn changesets(&self) -> &ChangeSets {
        &self.changesets
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn prune(inner: &mut Inner, now: Instant) {
        inner.entries.retain(|entry| entry.live_at(now));
    }

    /// A new token for `scope`. With [`MAX_LIVE`] already live, the one used
    /// longest ago is revoked first.
    pub fn mint(&self, scope: AgentScope) -> Minted {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the system's random source failed");
        let token = URL_SAFE_NO_PAD.encode(bytes);
        let now = (self.clock)();
        let mut inner = self.lock();
        Self::prune(&mut inner, now);
        while inner.entries.len() >= MAX_LIVE {
            // The oldest idle (paused) session's token goes first: nobody is
            // waiting on it. Then the one used longest ago.
            let oldest = inner
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| (!entry.paused, entry.last_used))
                .map(|(at, _)| at)
                .unwrap_or(0);
            inner.entries.remove(oldest);
        }
        let id = TokenId(inner.next_id);
        inner.next_id += 1;
        inner.entries.push(Entry {
            id,
            hash: hash(&token),
            minted: now,
            last_used: now,
            renewed: false,
            scope: Arc::new(scope),
            reads: Vec::new(),
            calls: 0,
            shown: Vec::new(),
            paused: false,
        });
        Minted { id, token }
    }

    /// The caller a bearer token stands for, when it is live; its use is
    /// counted. Every kept token is compared, each in constant time.
    pub fn verify(&self, token: &str) -> Option<Caller> {
        let given = hash(token.trim());
        let now = (self.clock)();
        let mut inner = self.lock();
        Self::prune(&mut inner, now);
        let mut found = None;
        for (at, entry) in inner.entries.iter().enumerate() {
            if bool::from(entry.hash.ct_eq(&given)) {
                found = Some(at);
            }
        }
        let entry = &mut inner.entries[found?];
        entry.last_used = now;
        Some(Caller::Agent {
            token: entry.id,
            scope: entry.scope.clone(),
        })
    }

    /// Between turns, token `id` refuses every call but stays live, so the
    /// agent's session (and the bearer it was started with) stays valid.
    pub fn pause(&self, id: TokenId) {
        if let Some(entry) = self.lock().entries.iter_mut().find(|entry| entry.id == id) {
            entry.paused = true;
        }
    }

    /// Pauses every token of task `task` (a session that started again has
    /// more than one over its life).
    pub fn pause_task(&self, task: TaskId) {
        for entry in &mut self.lock().entries {
            if entry.scope.task == Some(task) {
                entry.paused = true;
            }
        }
    }

    /// Whether token `id` is paused.
    pub fn is_paused(&self, id: TokenId) -> bool {
        self.lock()
            .entries
            .iter()
            .any(|entry| entry.id == id && entry.paused)
    }

    /// A new turn begins on paused token `id`: it accepts calls again with
    /// `limits` as its own lifetime and call budget, and its clock, renewal
    /// and call counter start over. `false` when the token is gone.
    pub fn resume(&self, id: TokenId, limits: TaskLimits) -> bool {
        let now = (self.clock)();
        let mut inner = self.lock();
        let Some(at) = inner.entries.iter().position(|entry| entry.id == id) else {
            return false;
        };
        Self::resume_entry(&mut inner.entries[at], limits, now);
        if let Some(task) = inner.entries[at].scope.task {
            inner.task_calls.remove(&task);
        }
        true
    }

    /// [`TokenStore::resume`] for every token of task `task`. `false` when
    /// it has none left.
    pub fn resume_task(&self, task: TaskId, limits: TaskLimits) -> bool {
        let now = (self.clock)();
        let mut inner = self.lock();
        let mut any = false;
        for entry in &mut inner.entries {
            if entry.scope.task == Some(task) {
                Self::resume_entry(entry, limits, now);
                any = true;
            }
        }
        inner.task_calls.remove(&task);
        any
    }

    fn resume_entry(entry: &mut Entry, limits: TaskLimits, now: Instant) {
        let mut scope = (*entry.scope).clone();
        scope.task_limits = Some(limits);
        entry.scope = Arc::new(scope);
        entry.paused = false;
        entry.minted = now;
        entry.last_used = now;
        entry.renewed = false;
        entry.calls = 0;
    }

    /// Whether token `id` would still be accepted.
    pub fn is_live(&self, id: TokenId) -> bool {
        let now = (self.clock)();
        self.lock()
            .entries
            .iter()
            .any(|entry| entry.id == id && entry.live_at(now))
    }

    /// Gives a live token a fresh lifetime, once. `false` when it has been
    /// renewed already, or is gone.
    pub fn renew(&self, id: TokenId) -> bool {
        let now = (self.clock)();
        let mut inner = self.lock();
        Self::prune(&mut inner, now);
        match inner.entries.iter_mut().find(|entry| entry.id == id) {
            Some(entry) if !entry.renewed => {
                entry.renewed = true;
                entry.minted = now;
                entry.last_used = now;
                true
            }
            _ => false,
        }
    }

    /// A new question begins on token `id`: its tool-call budget and its
    /// reads start over.
    pub fn begin_turn(&self, id: TokenId) {
        if let Some(entry) = self.lock().entries.iter_mut().find(|entry| entry.id == id) {
            entry.calls = 0;
            entry.reads.clear();
            entry.shown.clear();
        }
    }

    /// Starts token `id`'s numbering from the excerpts the question's prompt
    /// already numbered. Replaces anything shown before.
    pub fn seed_shown(&self, id: TokenId, notes: Vec<ShownNote>) {
        if let Some(entry) = self.lock().entries.iter_mut().find(|entry| entry.id == id) {
            entry.shown = notes;
        }
    }

    /// Everything token `id` was shown this question, numbered. Empty once
    /// the token is revoked.
    pub fn shown_notes(&self, id: TokenId) -> Vec<ShownNote> {
        self.lock()
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.shown.clone())
            .unwrap_or_default()
    }

    /// Notes that `lines` of note `note` were shown to token `id`, and
    /// returns the number they carry: the note's existing number when it was
    /// shown before (the lines are merged into it), else the next one.
    pub(crate) fn show_lines(
        &self,
        id: TokenId,
        note: &str,
        title: &str,
        updated: Option<String>,
        lines: Vec<(u32, String)>,
    ) -> usize {
        let mut inner = self.lock();
        let Some(entry) = inner.entries.iter_mut().find(|entry| entry.id == id) else {
            return 0;
        };
        if let Some(seen) = entry.shown.iter_mut().find(|seen| seen.id == note) {
            for (n, text) in lines {
                match seen.lines.binary_search_by_key(&n, |(line, _)| *line) {
                    Ok(_) => {}
                    Err(at) => seen.lines.insert(at, (n, text)),
                }
            }
            return seen.number;
        }
        let number = entry
            .shown
            .iter()
            .map(|seen| seen.number)
            .max()
            .unwrap_or(0)
            + 1;
        let mut lines = lines;
        lines.sort_by_key(|(n, _)| *n);
        lines.dedup_by_key(|(n, _)| *n);
        entry.shown.push(ShownNote {
            number,
            id: note.to_owned(),
            title: title.to_owned(),
            updated,
            lines,
        });
        number
    }

    /// Counts one tool call on `id`; `false` once the question has made
    /// [`MAX_CALLS_PER_TURN`].
    pub(crate) fn charge(&self, id: TokenId) -> bool {
        let mut inner = self.lock();
        let Some(entry) = inner.entries.iter_mut().find(|entry| entry.id == id) else {
            return false;
        };
        if entry.paused {
            return false;
        }
        entry.calls += 1;
        let max = entry
            .scope
            .task_limits
            .map_or(MAX_CALLS_PER_TURN, |limits| limits.calls);
        let (own, task) = (entry.calls, entry.scope.task);
        match task {
            Some(task) => {
                let calls = inner.task_calls.entry(task).or_default();
                *calls += 1;
                *calls <= max
            }
            None => own <= max,
        }
    }

    /// [`TokenStore::charge`], for tests in other crates.
    #[doc(hidden)]
    pub fn charge_for_test(&self, id: TokenId) -> bool {
        self.charge(id)
    }

    /// How many tool calls task `task` has made, over all its tokens.
    pub fn task_calls(&self, task: TaskId) -> usize {
        self.lock().task_calls.get(&task).copied().unwrap_or(0)
    }

    /// The notes task `task` has been shown, over all its tokens, in order.
    pub fn task_reads(&self, task: TaskId) -> Vec<String> {
        self.lock()
            .task_reads
            .get(&task)
            .cloned()
            .unwrap_or_default()
    }

    /// Forgets what task `task` counted.
    pub fn forget_task(&self, task: TaskId) {
        let mut inner = self.lock();
        inner.task_calls.remove(&task);
        inner.task_reads.remove(&task);
    }

    /// How many tool calls token `id` has made this question or task.
    pub fn calls(&self, id: TokenId) -> usize {
        self.lock()
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .map_or(0, |entry| entry.calls)
    }

    /// Notes an answer on `id` showed, for the person to see.
    pub(crate) fn record(&self, id: TokenId, notes: &[String]) {
        let mut inner = self.lock();
        let Some(entry) = inner.entries.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        for note in notes {
            if !entry.reads.contains(note) {
                entry.reads.push(note.clone());
            }
        }
        if let Some(task) = entry.scope.task {
            let all = inner.task_reads.entry(task).or_default();
            for note in notes {
                if !all.contains(note) {
                    all.push(note.clone());
                }
            }
        }
    }

    /// Every note token `id` has been shown this question, in order: what
    /// the agent searched or read. Empty once the token is revoked.
    pub fn reads(&self, id: TokenId) -> Vec<String> {
        self.lock()
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.reads.clone())
            .unwrap_or_default()
    }

    /// Ends token `id` now.
    pub fn revoke(&self, id: TokenId) {
        self.lock().entries.retain(|entry| entry.id != id);
    }

    /// Ends every token of task `task`.
    pub fn revoke_task(&self, task: TaskId) {
        self.lock()
            .entries
            .retain(|entry| entry.scope.task != Some(task));
    }

    /// Ends every token: AI was turned off, the agent changed, or Kanso is
    /// quitting.
    pub fn revoke_all(&self) {
        self.lock().entries.clear();
    }

    /// How many tokens are live.
    pub fn live_count(&self) -> usize {
        let now = (self.clock)();
        self.lock()
            .entries
            .iter()
            .filter(|entry| entry.live_at(now))
            .count()
    }

    /// The ids of the live tokens, oldest first, for tests.
    pub fn live(&self) -> HashSet<TokenId> {
        let now = (self.clock)();
        self.lock()
            .entries
            .iter()
            .filter(|entry| entry.live_at(now))
            .map(|entry| entry.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn clock() -> (Arc<Mutex<Instant>>, TokenStore) {
        let now = Arc::new(Mutex::new(Instant::now()));
        let read = now.clone();
        let store = TokenStore::with_clock(Arc::new(move || *read.lock().unwrap()));
        (now, store)
    }

    fn advance(now: &Mutex<Instant>, by: Duration) {
        let mut now = now.lock().unwrap();
        *now += by;
    }

    #[test]
    fn a_token_is_256_bits_of_base64url_and_never_printed() {
        let store = TokenStore::new();
        let minted = store.mint(AgentScope::ask(None));
        assert_eq!(minted.token.len(), 43);
        assert!(
            minted
                .token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!format!("{minted:?}").contains(&minted.token));
        assert!(!format!("{store:?}").contains(&minted.token));
        assert!(store.verify(&minted.token).is_some());
        assert!(store.verify(&minted.token[1..]).is_none());
        assert!(store.verify("").is_none());
    }

    #[test]
    fn tokens_expire_after_their_lifetime_or_idling() {
        let (now, store) = clock();
        let minted = store.mint(AgentScope::ask(None));
        advance(&now, IDLE - Duration::from_secs(1));
        assert!(store.verify(&minted.token).is_some());
        advance(&now, IDLE - Duration::from_secs(1));
        assert!(store.verify(&minted.token).is_some());
        // In use all along, it still ends at its lifetime.
        for _ in 0..3 {
            advance(&now, Duration::from_secs(4 * 60));
            store.verify(&minted.token);
        }
        assert!(store.verify(&minted.token).is_none());
        assert!(!store.is_live(minted.id));

        let idle = store.mint(AgentScope::ask(None));
        advance(&now, IDLE);
        assert!(store.verify(&idle.token).is_none());
    }

    #[test]
    fn a_token_renews_once_and_revokes_at_once() {
        let (now, store) = clock();
        let minted = store.mint(AgentScope::ask(None));
        advance(&now, Duration::from_secs(4 * 60));
        assert!(store.renew(minted.id));
        assert!(!store.renew(minted.id));
        store.revoke(minted.id);
        assert!(store.verify(&minted.token).is_none());
        assert!(!store.renew(minted.id));
    }

    #[test]
    fn a_question_has_a_budget_of_tool_calls() {
        let store = TokenStore::new();
        let minted = store.mint(AgentScope::ask(None));
        for _ in 0..MAX_CALLS_PER_TURN {
            assert!(store.charge(minted.id));
        }
        assert!(!store.charge(minted.id));
        store.begin_turn(minted.id);
        assert!(store.charge(minted.id));
    }

    fn task_limits(calls: usize) -> TaskLimits {
        TaskLimits {
            lifetime: Duration::from_secs(120),
            calls,
            max_notes: 10,
        }
    }

    fn task_token(store: &TokenStore, calls: usize) -> (TaskId, Minted) {
        let id = store.changesets().next_id();
        let scope = AgentScope::task(
            None,
            None,
            "Agent",
            id,
            crate::scope::TaskOps::default(),
            task_limits(calls),
        );
        (id, store.mint(scope))
    }

    #[test]
    fn a_paused_token_stays_live_past_its_lifetime_and_refuses_calls() {
        let (now, store) = clock();
        let (_, minted) = task_token(&store, 3);
        assert!(store.charge(minted.id));
        store.pause(minted.id);
        assert!(store.is_paused(minted.id));
        assert!(!store.charge(minted.id), "a paused token is not charged");
        advance(&now, Duration::from_secs(3600));
        assert!(store.is_live(minted.id), "an idle session's token waits");
        assert!(store.verify(&minted.token).is_some());
    }

    #[test]
    fn resume_starts_the_turns_budget_clock_and_renewal_over() {
        let (now, store) = clock();
        let (task, minted) = task_token(&store, 2);
        assert!(store.charge(minted.id));
        assert!(store.charge(minted.id));
        assert!(!store.charge(minted.id), "the turn's calls are used up");
        assert!(store.renew(minted.id));
        store.pause(minted.id);
        advance(&now, Duration::from_secs(1000));
        assert!(store.resume(minted.id, task_limits(2)));
        assert!(!store.is_paused(minted.id));
        assert_eq!(store.calls(minted.id), 0);
        assert_eq!(store.task_calls(task), 0);
        assert!(store.charge(minted.id));
        assert!(store.charge(minted.id));
        assert!(!store.charge(minted.id));
        // Its lifetime started over, and it may be renewed once more.
        store.pause(minted.id);
        assert!(store.resume(minted.id, task_limits(2)));
        advance(&now, Duration::from_secs(100));
        assert!(store.is_live(minted.id));
        assert!(store.renew(minted.id));
        // A gone token doesn't resume.
        store.revoke(minted.id);
        assert!(!store.resume(minted.id, task_limits(2)));
        assert!(!store.resume_task(task, task_limits(2)));
    }

    #[test]
    fn a_full_store_evicts_the_oldest_paused_token_first() {
        let (now, store) = clock();
        let mut minted = Vec::new();
        for _ in 0..MAX_LIVE {
            minted.push(store.mint(AgentScope::ask(None)));
            advance(&now, Duration::from_secs(1));
        }
        // The first is the longest unused, the third is paused.
        store.pause(minted[2].id);
        let fresh = store.mint(AgentScope::ask(None));
        let live = store.live();
        assert!(!live.contains(&minted[2].id), "the idle session's token");
        assert!(live.contains(&minted[0].id));
        assert!(live.contains(&fresh.id));
        assert_eq!(store.live_count(), MAX_LIVE);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Whatever happens between turns, a paused token is never charged
        /// and resumes with the full budget it was given.
        #[test]
        fn prop_a_resumed_token_has_the_full_budget(
            before in 0usize..8,
            calls in 1usize..8,
            waited in 0u64..5000,
        ) {
            let (now, store) = clock();
            let (task, minted) = task_token(&store, calls);
            for _ in 0..before {
                store.charge(minted.id);
            }
            store.pause(minted.id);
            advance(&now, Duration::from_secs(waited));
            prop_assert!(!store.charge(minted.id));
            prop_assert!(store.resume(minted.id, task_limits(calls)));
            prop_assert_eq!(store.task_calls(task), 0);
            for _ in 0..calls {
                prop_assert!(store.charge(minted.id));
            }
            prop_assert!(!store.charge(minted.id));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// However tokens are minted, used, renewed and revoked, no more than
        /// four are ever live, a revoked token is never accepted again, and a
        /// token is accepted only as itself.
        #[test]
        fn prop_tokens_stay_bounded_and_revocation_is_final(
            ops in prop::collection::vec((0u8..5, 0usize..8, 0u64..400), 1..60),
        ) {
            let (now, store) = clock();
            let mut minted: Vec<Minted> = Vec::new();
            let mut revoked: HashSet<TokenId> = HashSet::new();
            for (op, pick, secs) in ops {
                match op {
                    0 => minted.push(store.mint(AgentScope::ask(None))),
                    1 => if let Some(m) = minted.get(pick % minted.len().max(1))
                        && let Some(Caller::Agent { token, .. }) = store.verify(&m.token)
                    {
                        prop_assert_eq!(token, m.id);
                    },
                    2 => if let Some(m) = minted.get(pick % minted.len().max(1)) {
                        store.revoke(m.id);
                        revoked.insert(m.id);
                    },
                    3 => if let Some(m) = minted.get(pick % minted.len().max(1)) {
                        store.renew(m.id);
                    },
                    _ => advance(&now, Duration::from_secs(secs)),
                }
                prop_assert!(store.live_count() <= MAX_LIVE);
                for m in &minted {
                    if revoked.contains(&m.id) {
                        prop_assert!(store.verify(&m.token).is_none());
                        prop_assert!(!store.renew(m.id));
                    }
                }
            }
        }
    }
}
