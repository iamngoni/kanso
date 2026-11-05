//! API keys (issue #243): minting, hashing, the live-key cache, and reading
//! a key out of an `Authorization` header.
//!
//! A key is `kcl_` and 256 bits from the OS random source in base64url.
//! Only its SHA-256 is saved, in the library's `api_keys` table; the key is
//! returned once, from [`KeyStore::mint`]. The live keys are kept in memory
//! so a request makes no SQL query; a given key is compared with every one
//! kept, each in constant time, and no early exit tells which one matched.
//!
//! Revoking writes the table and drops the key from the cache under the same
//! lock that [`KeyStore::verify`] reads under, so no request is accepted on
//! a key once its revoke has returned.

use std::collections::HashSet;
use std::fmt;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_store::{ApiKeyRow, KeyAccess, LiveKey, NewApiKey, Store};
use sha2::{Digest as _, Sha256};
use subtle::{Choice, ConstantTimeEq as _};

use crate::auth::{bearer, decode_basic};
#[cfg(test)]
use crate::key_model::KEY_LEN;
use crate::lease::{DRAIN, KeyGate, Lease};
use crate::key_model::{KEY_PREFIX, KeyId, MintedKey, NewKey, is_key_shaped};
use crate::scope::{AgentScope, Caller, READ_TOOLS, SUGGEST_TOOLS};

/// `last_used_at` is written at most this often per key.
const TOUCH_EVERY_MS: i64 = 60_000;

type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

/// The SHA-256 of a key.
fn hash(key: &str) -> [u8; 32] {
    Sha256::digest(key.as_bytes()).into()
}

/// A UUID v7: 48 bits of milliseconds, then random bits.
fn uuid_v7(now_ms: i64) -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    let ms = (now_ms.max(0) as u64).to_be_bytes();
    bytes[..6].copy_from_slice(&ms[2..]);
    bytes[6] = 0x70 | (bytes[6] & 0x0f);
    bytes[8] = 0x80 | (bytes[8] & 0x3f);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// What a request of this key may read, and whom its suggestions are from.
fn scope_for(row: &ApiKeyRow) -> AgentScope {
    let books = row
        .books
        .as_ref()
        .map(|books| books.iter().cloned().collect::<HashSet<_>>());
    let mut scope = AgentScope::with_tools(books, &READ_TOOLS);
    if row.access != KeyAccess::Read {
        scope
            .tools
            .extend(SUGGEST_TOOLS.iter().map(|tool| (*tool).to_owned()));
        scope.suggests_as = Some(row.name.trim().to_owned());
    }
    scope
}

struct Cached {
    hash: [u8; 32],
    id: KeyId,
    access: KeyAccess,
    scope: Arc<AgentScope>,
    /// When `last_used_at` was last written, in ms.
    written: AtomicI64,
    /// What the requests signed in by this key hold.
    gate: Arc<KeyGate>,
}

impl Cached {
    fn new(key: LiveKey) -> Cached {
        Cached {
            hash: key.token_hash,
            scope: Arc::new(scope_for(&key.row)),
            access: key.row.access,
            written: AtomicI64::new(key.row.last_used_at.unwrap_or(0)),
            id: KeyId(key.row.id),
            gate: KeyGate::new(),
        }
    }
}

/// The API keys of one library. Cloning shares the same cache.
#[derive(Clone)]
pub struct KeyStore {
    store: Arc<Store>,
    live: Arc<RwLock<Vec<Cached>>>,
    /// When a request was last refused for want of a valid key, in ms; 0
    /// before any was. Shared by every clone, kept in memory only.
    refused: Arc<AtomicI64>,
    /// Counts every change to the keys, by any handle: lets a page showing
    /// them tell that they moved.
    changes: Arc<AtomicU64>,
    clock: Clock,
}

impl fmt::Debug for KeyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyStore")
            .field("live", &self.read().len())
            .finish()
    }
}

impl KeyStore {
    /// The keys saved in `store`, loaded.
    pub fn open(store: Arc<Store>) -> kanso_store::Result<KeyStore> {
        KeyStore::with_clock(store, Arc::new(now_ms))
    }

    /// [`KeyStore::open`] reading the time (ms since the epoch) from
    /// `clock`, for tests.
    pub fn with_clock(store: Arc<Store>, clock: Clock) -> kanso_store::Result<KeyStore> {
        let keys = KeyStore {
            store,
            live: Arc::new(RwLock::new(Vec::new())),
            refused: Arc::new(AtomicI64::new(0)),
            changes: Arc::new(AtomicU64::new(0)),
            clock,
        };
        keys.reload()?;
        Ok(keys)
    }

    // A poisoned lock still holds a consistent list: every writer replaces
    // or removes whole entries.
    fn read(&self) -> RwLockReadGuard<'_, Vec<Cached>> {
        self.live.read().unwrap_or_else(|p| p.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, Vec<Cached>> {
        self.live.write().unwrap_or_else(|p| p.into_inner())
    }

    /// Reads the live keys from the table again, for when something other
    /// than this handle changed it. On an error the cache is emptied, so a
    /// key the table no longer vouches for is never accepted. A key that is
    /// gone after the reload is cut off from the requests it signed in.
    pub fn reload(&self) -> kanso_store::Result<()> {
        let mut live = self.write();
        let loaded = self.store.api_keys_live();
        let mut old = std::mem::take(&mut *live);
        let result = match loaded {
            Ok(keys) => {
                *live = keys
                    .into_iter()
                    .map(|key| {
                        let mut entry = Cached::new(key);
                        // The same key keeps its gate: its requests go on.
                        if let Some(at) = old
                            .iter()
                            .position(|o| o.id == entry.id && o.hash == entry.hash)
                        {
                            entry.gate = old.swap_remove(at).gate;
                        }
                        entry
                    })
                    .collect();
                Ok(())
            }
            Err(e) => Err(e),
        };
        for gone in &old {
            gone.gate.close();
        }
        self.changes.fetch_add(1, Ordering::Relaxed);
        result
    }

    /// How many times the keys have changed since this handle was opened.
    pub fn changes(&self) -> u64 {
        self.changes.load(Ordering::Relaxed)
    }

    /// Makes a key. The key comes back in the result and nowhere else: only
    /// its hash is saved.
    pub fn mint(&self, new: NewKey) -> kanso_store::Result<MintedKey> {
        let name = new.name.trim();
        if name.is_empty() {
            return Err(kanso_store::StoreError::Invalid(
                "a key needs a name".into(),
            ));
        }
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the system's random source failed");
        let key = format!("{KEY_PREFIX}{}", URL_SAFE_NO_PAD.encode(bytes));
        let now = (self.clock)();
        let row = NewApiKey {
            id: uuid_v7(now),
            name: name.to_owned(),
            client: new.client.trim().to_owned(),
            token_hash: hash(&key),
            access: new.access,
            books: new.books,
            created_at: now,
        };
        let mut live = self.write();
        self.store.api_key_insert(&row)?;
        let id = KeyId(row.id.clone());
        live.push(Cached::new(LiveKey {
            row: ApiKeyRow {
                id: row.id,
                name: row.name,
                client: row.client,
                access: row.access,
                books: row.books,
                created_at: now,
                last_used_at: None,
                revoked_at: None,
            },
            token_hash: row.token_hash,
        }));
        self.changes.fetch_add(1, Ordering::Relaxed);
        Ok(MintedKey { id, key })
    }

    /// The caller a key stands for, when it is live. Every kept key is
    /// compared in constant time. A use is saved as `last_used_at`, at most
    /// once a minute per key; a failed save never refuses the request.
    pub fn verify(&self, key: &str) -> Option<Caller> {
        self.check(key, false).map(|(caller, _)| caller)
    }

    /// [`KeyStore::verify`], and a [`Lease`] the request holds until it is
    /// answered: a revoke waits for it and cuts the request short.
    pub(crate) fn verify_leased(&self, key: &str) -> Option<(Caller, Lease)> {
        let (caller, lease) = self.check(key, true)?;
        lease.map(|lease| (caller, lease))
    }

    fn check(&self, key: &str, lease: bool) -> Option<(Caller, Option<Lease>)> {
        if !is_key_shaped(key) {
            return None;
        }
        let given = hash(key);
        let (caller, written, id, lease) = {
            let live = self.read();
            let mut found: Option<&Cached> = None;
            for entry in live.iter() {
                let same: Choice = entry.hash.ct_eq(&given);
                if bool::from(same) {
                    found = Some(entry);
                }
            }
            let entry = found?;
            // Taken under the read lock a revoke needs to write: a key is
            // either still here to lease, or already refused.
            let lease = if lease {
                Some(entry.gate.lease()?)
            } else {
                None
            };
            (
                Caller::Key {
                    id: entry.id.clone(),
                    access: entry.access,
                    scope: entry.scope.clone(),
                },
                entry.written.load(Ordering::Relaxed),
                entry.id.clone(),
                lease,
            )
        };
        let now = (self.clock)();
        if now.saturating_sub(written) >= TOUCH_EVERY_MS {
            // The revoke lock is not held: a touch never revives a key,
            // because the update skips revoked rows.
            if self.store.api_key_touch(id.as_str(), now).is_ok()
                && let Some(entry) = self.read().iter().find(|e| e.id == id)
            {
                entry.written.store(now, Ordering::Relaxed);
            }
        }
        Some((caller, lease))
    }

    /// Revokes key `id`. No request is accepted on it from now on, a request
    /// it already signed in is cut short, and this returns only once none of
    /// them is still working in the library (at most [`DRAIN`] if the
    /// library is stuck), so nothing the key started finishes after it.
    /// Nothing brings the key back. `false` when there is no such live key.
    pub fn revoke(&self, id: &KeyId) -> kanso_store::Result<bool> {
        let mut live = self.write();
        // Out of the cache first: if the table write fails, the key is
        // still refused here, and `reload` finds it live again only if the
        // table says so.
        let gates: Vec<Arc<KeyGate>> = live
            .iter()
            .filter(|entry| entry.id == *id)
            .map(|entry| entry.gate.clone())
            .collect();
        live.retain(|entry| entry.id != *id);
        for gate in &gates {
            gate.close();
        }
        let revoked = self.store.api_key_revoke(id.as_str(), (self.clock)());
        drop(live);
        self.changes.fetch_add(1, Ordering::Relaxed);
        for gate in &gates {
            gate.drain(DRAIN);
        }
        revoked
    }

    /// Notes that a request was just refused for want of a valid key. Only
    /// the time is kept: nothing about the caller, which could be faked.
    pub fn note_refused(&self) {
        self.refused.store((self.clock)().max(1), Ordering::Relaxed);
    }

    /// When a request was last refused for want of a valid key, in ms since
    /// the epoch, since this handle (or any clone) was opened.
    pub fn last_refused(&self) -> Option<i64> {
        match self.refused.load(Ordering::Relaxed) {
            0 => None,
            at => Some(at),
        }
    }

    /// Every key, revoked ones included, newest first, for Settings.
    pub fn list(&self) -> kanso_store::Result<Vec<ApiKeyRow>> {
        self.store.api_keys_all()
    }
}

/// The key an `Authorization` header carries, when it carries one in the
/// shape of a key: `Bearer kcl_…`, or `Basic` with any username and the key
/// as the password (for tools that only take a login). `None` for anything
/// else, a session token included.
pub fn key_from_header(header: &[u8]) -> Option<String> {
    if let Some(token) = bearer(header) {
        return is_key_shaped(token).then(|| token.to_owned());
    }
    let decoded = decode_basic(header)?;
    let text = std::str::from_utf8(&decoded).ok()?;
    let (_username, password) = text.split_once(':')?;
    is_key_shaped(password).then(|| password.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Mutex;

    fn basic_header(username: &str, password: &str) -> String {
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"))
        )
    }

    fn fixture() -> (tempfile::TempDir, Arc<Store>, Arc<Mutex<i64>>, KeyStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let now = Arc::new(Mutex::new(1_000_000i64));
        let read = now.clone();
        let keys =
            KeyStore::with_clock(store.clone(), Arc::new(move || *read.lock().unwrap())).unwrap();
        (dir, store, now, keys)
    }

    fn new_key(access: KeyAccess) -> NewKey {
        NewKey {
            name: "Codex on this Mac".into(),
            client: "codex".into(),
            access,
            books: None,
        }
    }

    #[test]
    fn a_key_is_47_characters_and_only_its_hash_is_saved() {
        let (dir, store, _, keys) = fixture();
        let minted = keys.mint(new_key(KeyAccess::Read)).unwrap();
        assert_eq!(minted.key.len(), KEY_LEN);
        assert!(minted.key.starts_with("kcl_"));
        assert!(!format!("{minted:?}").contains(&minted.key));
        assert!(!format!("{keys:?}").contains(&minted.key));
        let live = store.api_keys_live().unwrap();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].token_hash, hash(&minted.key));
        assert!(!format!("{:?}", live[0]).contains(&minted.key));
        // The key is nowhere in the library's files.
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let bytes = std::fs::read(entry.unwrap().path()).unwrap();
            let needle = minted.key.as_bytes();
            assert!(!bytes.windows(needle.len()).any(|w| w == needle));
        }
        assert!(matches!(
            keys.verify(&minted.key),
            Some(Caller::Key {
                access: KeyAccess::Read,
                ..
            })
        ));
    }

    #[test]
    fn ids_are_uuid_v7() {
        let id = uuid_v7(0x0123_4567_89ab);
        assert_eq!(id.len(), 36);
        assert!(id.starts_with("01234567-89ab-7"));
        assert!(matches!(id.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
    }

    #[test]
    fn access_decides_the_scope() {
        let (_d, _s, _n, keys) = fixture();
        let read = keys.mint(new_key(KeyAccess::Read)).unwrap();
        let write = keys.mint(new_key(KeyAccess::Write)).unwrap();
        let Some(Caller::Key { scope, access, .. }) = keys.verify(&read.key) else {
            panic!("live key refused");
        };
        assert_eq!(access, KeyAccess::Read);
        assert!(scope.allows_tool("search-notes"));
        assert!(!scope.allows_tool("update-note"));
        assert!(scope.suggests_as.is_none());
        assert!(!scope.allow_attachments);
        let Some(Caller::Key { scope, access, .. }) = keys.verify(&write.key) else {
            panic!("live key refused");
        };
        assert_eq!(access, KeyAccess::Write);
        assert!(scope.allows_tool("update-note"));
        assert_eq!(scope.suggests_as.as_deref(), Some("Codex on this Mac"));
    }

    #[test]
    fn a_blank_name_makes_no_key() {
        let (_d, store, _n, keys) = fixture();
        let mut new = new_key(KeyAccess::Read);
        new.name = "  ".into();
        assert!(keys.mint(new).is_err());
        assert!(store.api_keys_all().unwrap().is_empty());
    }

    #[test]
    fn revoking_stops_a_key_at_once_and_after_a_restart() {
        let (_d, store, _n, keys) = fixture();
        let minted = keys.mint(new_key(KeyAccess::Read)).unwrap();
        assert!(keys.verify(&minted.key).is_some());
        assert!(keys.revoke(&minted.id).unwrap());
        assert!(keys.verify(&minted.key).is_none());
        assert!(!keys.revoke(&minted.id).unwrap());
        let again = KeyStore::open(store.clone()).unwrap();
        assert!(again.verify(&minted.key).is_none());
        let rows = keys.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].revoked_at.is_some());
    }

    // no-prop-test: a fixed sequence; the clock decides the value.
    #[test]
    fn a_refusal_is_noted_for_every_clone_with_the_time_only() {
        let (_d, _s, now, keys) = fixture();
        let other = keys.clone();
        assert_eq!(keys.last_refused(), None);
        *now.lock().unwrap() = 5_000_000;
        other.note_refused();
        assert_eq!(keys.last_refused(), Some(5_000_000));
        *now.lock().unwrap() = 6_000_000;
        keys.note_refused();
        assert_eq!(other.last_refused(), Some(6_000_000));
    }

    #[test]
    fn a_clone_shares_the_cache_and_reload_picks_up_other_changes() {
        let (_d, store, _n, keys) = fixture();
        let other = keys.clone();
        let minted = keys.mint(new_key(KeyAccess::Read)).unwrap();
        assert!(other.verify(&minted.key).is_some());
        keys.revoke(&minted.id).unwrap();
        assert!(other.verify(&minted.key).is_none());
        // Revoked behind the cache's back: refused after a reload.
        let second = keys.mint(new_key(KeyAccess::Read)).unwrap();
        store.api_key_revoke(second.id.as_str(), 5).unwrap();
        keys.reload().unwrap();
        assert!(keys.verify(&second.key).is_none());
    }

    #[test]
    fn last_used_is_written_at_most_once_a_minute() {
        let (_d, _s, now, keys) = fixture();
        let minted = keys.mint(new_key(KeyAccess::Read)).unwrap();
        let used = |keys: &KeyStore| keys.list().unwrap()[0].last_used_at;
        assert_eq!(used(&keys), None);
        keys.verify(&minted.key);
        assert_eq!(used(&keys), Some(1_000_000));
        *now.lock().unwrap() += 30_000;
        keys.verify(&minted.key);
        assert_eq!(used(&keys), Some(1_000_000));
        *now.lock().unwrap() += 30_000;
        keys.verify(&minted.key);
        assert_eq!(used(&keys), Some(1_060_000));
    }

    #[test]
    fn a_revoked_key_is_not_marked_used() {
        let (_d, store, _n, keys) = fixture();
        let minted = keys.mint(new_key(KeyAccess::Read)).unwrap();
        keys.revoke(&minted.id).unwrap();
        store.api_key_touch(minted.id.as_str(), 9).unwrap();
        assert_eq!(keys.list().unwrap()[0].last_used_at, None);
    }

    #[test]
    fn headers_carry_a_key_as_bearer_or_a_basic_password() {
        let key = format!("kcl_{}", "A".repeat(43));
        assert_eq!(
            key_from_header(format!("Bearer {key}").as_bytes()).as_deref(),
            Some(key.as_str())
        );
        assert_eq!(
            key_from_header(format!("bearer  {key} ").as_bytes()).as_deref(),
            Some(key.as_str())
        );
        for user in ["", "anyone", "me@example.com"] {
            let header = basic_header(user, &key);
            assert_eq!(
                key_from_header(header.as_bytes()).as_deref(),
                Some(key.as_str())
            );
        }
        let unpadded = basic_header("u", &key).trim_end_matches('=').to_owned();
        assert_eq!(
            key_from_header(unpadded.as_bytes()).as_deref(),
            Some(key.as_str())
        );
        for bad in [
            "Bearer abc".to_owned(),
            "Bearer ".to_owned(),
            format!("Bearer {}x", key),
            format!("Bearer {}", &key[1..]),
            basic_header("kcl_x", "password"),
            basic_header(&key, "nope"),
            format!("Basic {key}"),
            "Basic !!!".to_owned(),
            String::new(),
        ] {
            assert!(key_from_header(bad.as_bytes()).is_none(), "{bad}");
        }
        // A session token is not a key.
        assert!(
            key_from_header(b"Bearer ZXhhbXBsZS1zZXNzaW9uLXRva2VuLTQzLWNoYXJzLWxvbmc").is_none()
        );
    }

    fn arbitrary_key() -> impl Strategy<Value = String> {
        proptest::string::string_regex("kcl_[A-Za-z0-9_-]{43}").unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// No key a caller makes up verifies, whether or not it has the
        /// shape of one, however many real keys are live.
        #[test]
        fn prop_random_keys_never_verify(
            guess in arbitrary_key(),
            noise in ".{0,80}",
            live in 0usize..4,
        ) {
            let (_d, _s, _n, keys) = fixture();
            for _ in 0..live {
                keys.mint(new_key(KeyAccess::Read)).unwrap();
            }
            prop_assert!(keys.verify(&guess).is_none());
            prop_assert!(keys.verify(&noise).is_none());
        }

        /// A revoked key never verifies again, whatever else is minted,
        /// revoked or reloaded afterwards; the others keep working.
        #[test]
        fn prop_revoked_keys_never_verify(
            ops in prop::collection::vec((0u8..4, 0usize..6), 1..24),
        ) {
            let (_d, store, _n, keys) = fixture();
            let mut minted: Vec<MintedKey> = Vec::new();
            let mut revoked: HashSet<usize> = HashSet::new();
            for (op, pick) in ops {
                match op {
                    0 => minted.push(keys.mint(new_key(KeyAccess::Read)).unwrap()),
                    1 if !minted.is_empty() => {
                        let at = pick % minted.len();
                        keys.revoke(&minted[at].id).unwrap();
                        revoked.insert(at);
                    }
                    2 => keys.reload().unwrap(),
                    _ => {
                        // A restart: a fresh cache from the table.
                        let again = KeyStore::open(store.clone()).unwrap();
                        for (at, m) in minted.iter().enumerate() {
                            prop_assert_eq!(again.verify(&m.key).is_some(), !revoked.contains(&at));
                        }
                    }
                }
                for (at, m) in minted.iter().enumerate() {
                    prop_assert_eq!(keys.verify(&m.key).is_some(), !revoked.contains(&at));
                }
            }
        }

        /// The cache never outlives a revoke: from any number of handles
        /// sharing it, and in a handle that was loaded before the revoke.
        #[test]
        fn prop_the_cache_never_outlives_a_revoke(handles in 1usize..4, keys_made in 1usize..4) {
            let (_d, store, _n, keys) = fixture();
            let clones: Vec<KeyStore> = (0..handles).map(|_| keys.clone()).collect();
            let minted: Vec<MintedKey> =
                (0..keys_made).map(|_| keys.mint(new_key(KeyAccess::Write)).unwrap()).collect();
            for m in &minted {
                for h in &clones {
                    prop_assert!(h.verify(&m.key).is_some());
                }
            }
            for m in &minted {
                clones[0].revoke(&m.id).unwrap();
                for h in &clones {
                    prop_assert!(h.verify(&m.key).is_none());
                }
            }
            prop_assert!(store.api_keys_live().unwrap().is_empty());
        }

        /// Keys round-trip through both header forms, for any username
        /// that can be one, and the result is the key.
        #[test]
        fn prop_parsing_round_trips(key in arbitrary_key(), user in "[^:]{0,20}") {
            prop_assert_eq!(
                key_from_header(format!("Bearer {key}").as_bytes()),
                Some(key.clone())
            );
            prop_assert_eq!(
                key_from_header(basic_header(&user, &key).as_bytes()),
                Some(key)
            );
        }

        /// Parsing never panics on any bytes, and what it returns always
        /// has the shape of a key.
        #[test]
        fn prop_parsing_only_ever_returns_key_shaped_text(
            bytes in prop::collection::vec(any::<u8>(), 0..120),
        ) {
            if let Some(key) = key_from_header(&bytes) {
                prop_assert!(is_key_shaped(&key));
            }
        }
    }
}
