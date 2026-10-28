//! Locked notes without a window: the lock keys a session holds, what a
//! note shows of its lock, and sealing, opening and re-keying notes with
//! them (`kanso_sync::lock` has the cryptography).
//!
//! Nothing is unlocked at the start. Typing the lock password once opens
//! every note locked with it until the session is locked again, or
//! [`AUTO_LOCK`] after it was typed. Opening locked notes doesn't keep them
//! open longer; saving one does, so they don't close while you write. The
//! store only ever gets a note sealed.
//!
//! [`Session`] takes the time as a parameter, never reading a clock of its
//! own, so a test (or a phone that locks when it goes to the background)
//! decides what "now" is. Its keys are handed out as `Arc` clones: a key a
//! caller holds outlives the session's lock, as it does on the desktop.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kanso_core::Note;
use kanso_core::model::{KANSO_FILES_FIELD, note_files_from_extra, store_note_files};
use kanso_store::Store;
use kanso_sync::lock::{Envelope, FileKey, LOCK_KDF, LockError, LockKey, LockPassword, SealId};
use kanso_sync::{KdfParams, Machine};

mod notebook;

pub use notebook::{book_subtree, seal_notebook, take_lock_off};

/// How long the notes stay unlocked after the lock password (or Touch ID)
/// opened them, or a locked note was last saved.
pub const AUTO_LOCK: Duration = Duration::from_secs(10 * 60);

/// The lock keys typed this session. The first is the one new locks use;
/// the others open notes locked on another device, with another salt.
#[derive(Default)]
pub struct Session {
    keys: Vec<Arc<LockKey>>,
    /// When a key was added or a locked note saved: [`AUTO_LOCK`] counts
    /// from here.
    last_used: Option<Instant>,
    /// Argon2 parameters for new keys; tests use cheap ones.
    params: Option<KdfParams>,
    /// File keys held only for the current lock generation, keyed by seal ID.
    file_keys: HashMap<[u8; 16], FileKey>,
}

impl Session {
    /// The key new locks use, if a password was typed.
    pub fn current(&self) -> Option<Arc<LockKey>> {
        self.keys.first().cloned()
    }

    /// Whether any locked notes are open now.
    pub fn is_unlocked(&self) -> bool {
        !self.keys.is_empty()
    }

    /// The key that opens `envelope`, if this session holds it.
    pub fn fitting(&self, envelope: &Envelope) -> Option<Arc<LockKey>> {
        self.keys.iter().find(|key| key.fits(envelope)).cloned()
    }

    /// Every key this session holds, the one new locks use first.
    pub fn keys(&self) -> &[Arc<LockKey>] {
        &self.keys
    }

    /// When a key was last added or a locked note saved.
    pub fn last_used(&self) -> Option<Instant> {
        self.last_used
    }

    /// Keeps `key` for the session (a key with the same salt and parameters
    /// is kept once) and starts [`AUTO_LOCK`] over.
    pub fn add(&mut self, key: LockKey, now: Instant) {
        if !self.keys.iter().any(|kept| kept.same_kdf(&key)) {
            self.keys.push(Arc::new(key));
        }
        self.last_used = Some(now);
    }

    /// Takes the keys this session holds, with `key` alone in their place
    /// when given (a new password). `None` locks the notes now.
    pub fn replace(&mut self, key: Option<LockKey>, now: Instant) {
        self.keys = key.into_iter().map(Arc::new).collect();
        self.file_keys.clear();
        self.last_used = Some(now);
    }

    /// Remembers a decrypted file key for one seal generation.
    pub fn remember_file_key(&mut self, seal: &SealId, key: FileKey) {
        self.file_keys.insert(*seal.as_bytes(), key);
    }

    /// Returns a cached key for this seal generation.
    pub fn file_key(&self, seal: &SealId) -> Option<&FileKey> {
        self.file_keys.get(seal.as_bytes())
    }

    /// Forgets every decrypted file key held by this session.
    pub fn forget_file_keys(&mut self) {
        self.file_keys.clear();
    }

    /// Keeps the notes open for another [`AUTO_LOCK`]: a locked note was
    /// saved. Opening one doesn't, so reading them doesn't keep them open.
    pub fn touch(&mut self, now: Instant) {
        if self.is_unlocked() {
            self.last_used = Some(now);
        }
    }

    /// Whether [`AUTO_LOCK`] has passed at `now` since the keys were added
    /// or a locked note was saved.
    pub fn idle(&self, now: Instant) -> bool {
        self.is_unlocked()
            && self
                .last_used
                .is_none_or(|used| now.saturating_duration_since(used) >= AUTO_LOCK)
    }

    /// Argon2 parameters for new keys.
    pub fn params(&self) -> KdfParams {
        self.params.unwrap_or(LOCK_KDF)
    }

    /// Other parameters for new keys: cheap ones, so tests derive in
    /// milliseconds.
    pub fn set_params(&mut self, params: KdfParams) {
        self.params = Some(params);
    }

    /// The key that seals a note going into the locked notebook of `target`
    /// (its own when held), else the one new locks use.
    fn sealing_key(&self, target: Option<&Target>) -> Option<Arc<LockKey>> {
        target
            .and_then(|target| target.envelope.as_ref().ok())
            .and_then(|envelope| self.fitting(envelope))
            .or_else(|| self.current())
    }

    /// Puts a locked note's text in its `body` when a key this session
    /// holds opens it, and says what a view should show. A note already
    /// revealed is revealed again to the same text. `store` is where a note
    /// in a locked notebook is sealed once a key is held; without one, such
    /// a note shows plain.
    pub fn reveal(&self, note: &mut Note, store: Option<&Store>) -> LockView {
        let envelope = match Envelope::from_extra(&note.extra) {
            None => return self.reveal_unsealed(note, store),
            Some(Ok(envelope)) => envelope,
            Some(Err(err)) => {
                note.body.clear();
                return LockView::Closed(Some(sentence(&err)));
            }
        };
        let Some(key) = self.fitting(&envelope) else {
            note.body.clear();
            return LockView::Closed(None);
        };
        match key.open(&note.id, &envelope) {
            Ok(body) => {
                note.body = body.to_string();
                LockView::Open
            }
            Err(err) => {
                note.body.clear();
                LockView::Closed(Some(sentence(&err)))
            }
        }
    }

    /// A note that isn't sealed: plain, unless it is in a locked notebook.
    /// There it is sealed as soon as a key is held, and shows closed until
    /// then.
    fn reveal_unsealed(&self, note: &mut Note, store: Option<&Store>) -> LockView {
        let Some(store) = store else {
            return LockView::Plain;
        };
        if !store.note_in_locked_book(&note.id).unwrap_or(false) {
            return LockView::Plain;
        }
        let mut sealing = note.clone();
        let sealed = self
            .seal_for_notebook(&mut sealing, store)
            .and_then(|stored| serde_json::to_value(&stored).map_err(|err| err.to_string()))
            .and_then(|value| store.lock_note(value).map_err(|err| err.to_string()));
        match sealed {
            Ok(result) => {
                sealing.rev = Some(result.rev);
                *note = sealing;
                LockView::Open
            }
            Err(_) => {
                note.body.clear();
                LockView::Closed(None)
            }
        }
    }

    /// Seals an open locked note's `body` into its `kansoLock` and empties
    /// the body, as the store takes it. The note keeps its salt when this
    /// session holds its key; otherwise it moves to the current one. Saving
    /// keeps the notes open for another [`AUTO_LOCK`].
    pub fn seal(&mut self, note: &mut Note, now: Instant) -> Result<(), String> {
        let fitting = match Envelope::from_extra(&note.extra) {
            Some(Ok(envelope)) => self.fitting(&envelope),
            _ => None,
        };
        let key = fitting
            .or_else(|| self.current())
            .ok_or_else(|| "The notes were locked again before this one was saved.".to_owned())?;
        let envelope = key.seal(&note.id, &note.body);
        Envelope::store(Some(&envelope), &mut note.extra);
        note.body.clear();
        self.touch(now);
        Ok(())
    }

    /// Seals `note`, which isn't sealed, for the locked notebook it is in or
    /// is going into, keeping its text in `body` for the editor; the note
    /// returned is what the store then takes. Fails when no key is held.
    pub fn seal_for_notebook(&self, note: &mut Note, store: &Store) -> Result<Note, String> {
        let target = Target::of_book(&note.book_id, store);
        let key = self
            .sealing_key(target.as_ref())
            .ok_or_else(|| "Unlock the notebook first: its notes are locked.".to_owned())?;
        let envelope = key.seal(&note.id, &note.body);
        Envelope::store(Some(&envelope), &mut note.extra);
        let mut stored = note.clone();
        stored.body.clear();
        Ok(stored)
    }

    /// Seals the notes in locked notebooks that aren't sealed yet (made
    /// there empty, or put there by another device or an older Kanso), now
    /// that a key is held. One that changed meanwhile waits for the next
    /// time.
    pub fn seal_unsealed(&self, store: &Store) {
        let Ok(notes) = store.unsealed_in_locked_books() else {
            return;
        };
        for mut note in notes {
            if let Ok(stored) = self.seal_for_notebook(&mut note, store)
                && let Ok(value) = serde_json::to_value(&stored)
                && let Err(err) = store.lock_note(value)
            {
                eprintln!("kanso: locked notebooks: couldn't seal a note: {err}");
            }
        }
    }
}

/// What a view shows of a note.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LockView {
    /// Not locked.
    #[default]
    Plain,
    /// Locked, and open with a key this session holds: `body` is its text.
    Open,
    /// Locked and not open, with what is wrong when a password won't help.
    Closed(Option<String>),
}

impl LockView {
    pub fn is_closed(&self) -> bool {
        matches!(self, LockView::Closed(_))
    }
}

/// The lock password typed, or the sentence that says what is wrong with it.
pub fn password(text: &str) -> Result<LockPassword, String> {
    LockPassword::new(text).map_err(|err| format!("For the lock password, {err}."))
}

/// What a password is tried on: a note's lock, or a locked notebook's, with
/// the ID it is bound to.
#[derive(Clone, Debug)]
pub struct Target {
    pub id: String,
    pub envelope: Result<Envelope, LockError>,
}

impl Target {
    /// A note's own lock, or, for a note in a locked notebook that isn't
    /// sealed yet, the notebook's.
    pub fn of_note(note: &Note, store: &Store) -> Option<Target> {
        match Envelope::from_extra(&note.extra) {
            Some(envelope) => Some(Target {
                id: note.id.clone(),
                envelope,
            }),
            None => Target::of_book(&note.book_id, store),
        }
    }

    /// The lock of notebook `book_id`, or of the locked notebook it is in.
    pub fn of_book(book_id: &str, store: &Store) -> Option<Target> {
        let books = store.books().ok()?;
        let mut cursor = Some(book_id.to_owned());
        // A cycle a sync could make ends the walk.
        for _ in 0..=books.len() {
            let book = books.iter().find(|b| Some(&b.id) == cursor.as_ref())?;
            if let Some(envelope) = Envelope::from_extra(&book.extra) {
                return Some(Target {
                    id: book.id.clone(),
                    envelope,
                });
            }
            cursor = book.parent_book_id.clone();
        }
        None
    }

    /// Whether a key `session` holds opens it.
    pub fn is_open(&self, session: &Session) -> bool {
        self.envelope
            .as_ref()
            .ok()
            .is_some_and(|envelope| session.fitting(envelope).is_some())
    }
}

/// The key `password` gives for `target`, once it is seen to open it.
pub fn key_for(target: &Target, password: &LockPassword) -> Result<LockKey, String> {
    let envelope = target.envelope.as_ref().map_err(sentence)?;
    let key = LockKey::for_envelope(password, envelope).map_err(|err| sentence(&err))?;
    key.open(&target.id, envelope)
        .map_err(|err| sentence(&err))?;
    Ok(key)
}

/// A lock error as a sentence.
pub fn sentence(err: &LockError) -> String {
    let words = err.to_string();
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => format!("{}{}.", first.to_uppercase(), chars.as_str()),
        None => words,
    }
}

/// Some lock already made here or elsewhere, to try the lock password on:
/// a locked note's, or a locked notebook's. `None` when nothing is locked.
pub fn any_lock(store: &Store) -> Option<Target> {
    let note = store
        .locked_notes()
        .ok()
        .and_then(|notes| notes.into_iter().next());
    if let Some(note) = note {
        return Target::of_note(&note, store);
    }
    let books = store.books().ok()?;
    books.iter().find_map(|book| {
        Envelope::from_extra(&book.extra).map(|envelope| Target {
            id: book.id.clone(),
            envelope,
        })
    })
}

/// The library's ID, which names its Keychain (or Credential Manager) items.
pub fn library_id(store: &Store, machine: &Machine) -> Result<String, String> {
    kanso_sync::device_identity(store, machine)
        .map(|device| device.library_id)
        .map_err(|err| err.to_string())
}

/// A note's title as lists show it.
pub fn display_title(note: &Note) -> String {
    if note.title.trim().is_empty() {
        "Untitled".into()
    } else {
        note.title.clone()
    }
}

/// Seals every locked note `old` opens again under a new key from `new`,
/// and every locked notebook's lock. Returns the new key and how many notes
/// and notebooks `old` didn't open.
pub fn rekey(
    store: &Store,
    old: &LockPassword,
    new: &LockPassword,
    params: KdfParams,
) -> Result<(LockKey, usize), String> {
    let notes = store.locked_notes().map_err(|err| err.to_string())?;
    let mut old_keys: Vec<LockKey> = Vec::new();
    let mut opened = Vec::new();
    let mut skipped = 0;
    for note in notes {
        let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) else {
            skipped += 1;
            continue;
        };
        if !old_keys.iter().any(|key| key.fits(&envelope)) {
            match LockKey::for_envelope(old, &envelope) {
                Ok(key) => old_keys.push(key),
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            }
        }
        let key = old_keys
            .iter()
            .find(|key| key.fits(&envelope))
            .expect("a key for this salt was just made");
        match key.open(&note.id, &envelope) {
            Ok(body) => opened.push((note, body)),
            Err(_) => skipped += 1,
        }
    }
    // A locked notebook's own lock, which tells the password, moves too.
    let mut books = Vec::new();
    for book in store.books().map_err(|err| err.to_string())? {
        let Some(found) = Envelope::from_extra(&book.extra) else {
            continue;
        };
        let Ok(envelope) = found else {
            skipped += 1;
            continue;
        };
        if !old_keys.iter().any(|key| key.fits(&envelope)) {
            match LockKey::for_envelope(old, &envelope) {
                Ok(key) => old_keys.push(key),
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            }
        }
        let key = old_keys
            .iter()
            .find(|key| key.fits(&envelope))
            .expect("a key for this salt was just made");
        match key.open(&book.id, &envelope) {
            Ok(_) => books.push(book),
            Err(_) => skipped += 1,
        }
    }
    if opened.is_empty() && books.is_empty() && skipped > 0 {
        return Err(sentence(&LockError::WrongPassword));
    }
    let key = LockKey::create(new, params).map_err(|err| sentence(&err))?;
    for (mut note, body) in opened {
        let old_envelope = Envelope::from_extra(&note.extra).and_then(Result::ok);
        let envelope = key.seal(&note.id, &body);
        if note.extra.contains_key(KANSO_FILES_FIELD) {
            let Some(old_envelope) = old_envelope else {
                skipped += 1;
                continue;
            };
            let Some(mut files) = note_files_from_extra(&note.extra) else {
                skipped += 1;
                continue;
            };
            let Some(old_key) = old_keys.iter().find(|old_key| old_key.fits(&old_envelope)) else {
                skipped += 1;
                continue;
            };
            let mut valid = true;
            for file in &mut files {
                let Some(seal) = kanso_sync::lock::SealId::decode(&file.seal) else {
                    valid = false;
                    break;
                };
                let Ok(wrapped) = STANDARD.decode(&file.key) else {
                    valid = false;
                    break;
                };
                let Ok(file_key) = kanso_sync::lock::unwrap_file_key(
                    old_key,
                    &old_envelope,
                    &note.id,
                    &file.file,
                    &seal,
                    &wrapped,
                ) else {
                    valid = false;
                    break;
                };
                file.key = STANDARD.encode(kanso_sync::lock::wrap_file_key(
                    &key, &envelope, &note.id, &file.file, &seal, &file_key,
                ));
            }
            if !valid {
                skipped += 1;
                continue;
            }
            store_note_files(Some(&files), &mut note.extra);
        }
        Envelope::store(Some(&envelope), &mut note.extra);
        note.body.clear();
        let value = serde_json::to_value(&note).map_err(|err| err.to_string())?;
        // A note changed meanwhile keeps its old seal; it opens with the old
        // password until changed again.
        if store.lock_note(value).is_err() {
            skipped += 1;
        }
    }
    for mut book in books {
        Envelope::store(Some(&key.seal(&book.id, "")), &mut book.extra);
        let value = serde_json::to_value(&book).map_err(|err| err.to_string())?;
        if store
            .put(
                kanso_core::DocType::Book,
                value,
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .is_err()
        {
            skipped += 1;
        }
    }
    Ok((key, skipped))
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use kanso_sync::KdfParams;
    use kanso_sync::lock::{Envelope, LockError, LockKey, LockPassword};
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn lock_password(text: &str) -> LockPassword {
        LockPassword::new(text).unwrap()
    }

    /// Three keys, each from its own password, made once: deriving costs
    /// the same however cheap the parameters.
    static KEYS: LazyLock<[LockKey; 3]> = LazyLock::new(|| {
        ["first password", "second password", "third password"]
            .map(|text| LockKey::create(&lock_password(text), KdfParams::TEST).unwrap())
    });

    /// A copy of key `i`, as a session takes ownership of one.
    fn copy_of(i: usize) -> LockKey {
        LockKey::from_secret(&KEYS[i].to_secret()).unwrap()
    }

    #[test]
    fn file_key_cache_is_generation_scoped_and_clearable() {
        let mut session = Session::default();
        let seal = [7; 16];
        let file_key = FileKey::from_bytes([9; 32]);
        let seal = SealId::from_bytes(seal);
        session.remember_file_key(&seal, file_key);
        assert_eq!(session.file_key(&seal).unwrap().as_bytes(), &[9; 32]);
        session.forget_file_keys();
        assert!(session.file_key(&seal).is_none());
        session.remember_file_key(&seal, FileKey::from_bytes([9; 32]));
        session.replace(None, Instant::now());
        assert!(session.file_key(&seal).is_none());
    }

    fn note(id: &str, body: &str) -> Note {
        serde_json::from_value(json!({"_id": id, "bookId": "b", "title": "T", "body": body}))
            .unwrap()
    }

    fn locked_note(store: &Store, key: &LockKey, body: &str) -> String {
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": ""}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let mut note = store.note(&id).unwrap().unwrap();
        Envelope::store(Some(&key.seal(&id, body)), &mut note.extra);
        store
            .lock_note(serde_json::to_value(&note).unwrap())
            .unwrap();
        id
    }

    fn open(store: &Store, key: &LockKey, id: &str) -> Result<String, LockError> {
        let note = store.note(id).unwrap().unwrap();
        let envelope = Envelope::from_extra(&note.extra).unwrap().unwrap();
        key.open(id, &envelope).map(|body| body.to_string())
    }

    #[test]
    fn rekeying_moves_every_note_the_old_password_opens_to_the_new_one() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let old = LockKey::create(&lock_password("old password"), KdfParams::TEST).unwrap();
        let other = LockKey::create(&lock_password("another one"), KdfParams::TEST).unwrap();
        let a = locked_note(&store, &old, "first");
        let b = locked_note(&store, &old, "second");
        let c = locked_note(&store, &other, "third");

        let (new, skipped) = rekey(
            &store,
            &lock_password("old password"),
            &lock_password("new password"),
            KdfParams::TEST,
        )
        .unwrap();
        assert_eq!(skipped, 1);
        assert_eq!(open(&store, &new, &a).unwrap(), "first");
        assert_eq!(open(&store, &new, &b).unwrap(), "second");
        assert!(open(&store, &old, &a).is_err());
        // The note the old password didn't open keeps its own.
        assert_eq!(open(&store, &other, &c).unwrap(), "third");
        // Nothing sealed under the old password is left in history.
        assert!(store.revisions(&a).unwrap().is_empty());
    }

    #[test]
    fn rekeying_moves_a_locked_notebooks_lock_to_the_new_password() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let old = LockKey::create(&lock_password("old password"), KdfParams::TEST).unwrap();
        let a = locked_note(&store, &old, "first");
        let book = store.note(&a).unwrap().unwrap().book_id;
        seal_notebook(&store, &book, &old).unwrap();

        let (new, skipped) = rekey(
            &store,
            &lock_password("old password"),
            &lock_password("new password"),
            KdfParams::TEST,
        )
        .unwrap();
        assert_eq!(skipped, 0);
        let lock = store.books().unwrap()[0].extra["kansoLock"].clone();
        let envelope: Envelope = serde_json::from_value(lock).unwrap();
        assert!(new.fits(&envelope));
        assert_eq!(new.open(&book, &envelope).unwrap().as_str(), "");
        assert_eq!(open(&store, &new, &a).unwrap(), "first");
    }

    #[test]
    fn rekeying_with_a_wrong_password_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let old = LockKey::create(&lock_password("old password"), KdfParams::TEST).unwrap();
        let a = locked_note(&store, &old, "first");
        let err = rekey(
            &store,
            &lock_password("not the one"),
            &lock_password("new password"),
            KdfParams::TEST,
        )
        .unwrap_err();
        assert_eq!(err, "That isn't the lock password.");
        assert_eq!(open(&store, &old, &a).unwrap(), "first");
    }

    #[test]
    fn sealing_without_a_key_fails_and_leaves_the_text_alone() {
        let mut session = Session::default();
        let mut note = note("n1", "my secret");
        let err = session.seal(&mut note, Instant::now()).unwrap_err();
        assert_eq!(
            err,
            "The notes were locked again before this one was saved."
        );
        assert_eq!(note.body, "my secret");
        assert!(!note.is_locked());
    }

    #[test]
    fn a_note_sealed_for_one_id_does_not_open_as_another() {
        let mut session = Session::default();
        let now = Instant::now();
        session.add(copy_of(0), now);
        let mut note = note("n1", "my secret");
        session.seal(&mut note, now).unwrap();
        let mut moved = note.clone();
        moved.id = "n2".into();
        assert_eq!(
            session.reveal(&mut moved, None),
            LockView::Closed(Some("That isn't the lock password.".into()))
        );
        assert_eq!(moved.body, "");
    }

    #[test]
    fn the_lock_password_is_checked_in_plain_words() {
        assert_eq!(
            password("short").unwrap_err(),
            format!("For the lock password, {}.", LockError::TooShort)
        );
        assert!(password("long enough password").is_ok());
    }

    #[derive(Clone, Debug)]
    enum Op {
        Add(usize),
        Replace(Option<usize>),
        Touch,
        Tick(u64),
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        let op = prop_oneof![
            (0usize..3).prop_map(Op::Add),
            prop::option::of(0usize..3).prop_map(Op::Replace),
            Just(Op::Touch),
            // Ticks straddle AUTO_LOCK (600 seconds).
            (0u64..400).prop_map(Op::Tick),
        ];
        prop::collection::vec(op, 0..40)
    }

    proptest! {
        /// Model: the keys a session holds, which one new locks use, whether
        /// notes are open and when they close follow a list of key numbers
        /// and a clock the test moves. Locking now leaves nothing open.
        #[test]
        fn prop_no_key_survives_lock(ops in ops()) {
            let envelopes: Vec<Envelope> = (0..3).map(|i| KEYS[i].seal("n", "x")).collect();
            let base = Instant::now();
            let mut now = base;
            let mut session = Session::default();
            let mut keys: Vec<usize> = Vec::new();
            let mut last_used: Option<Instant> = None;
            for op in ops {
                match op {
                    Op::Add(i) => {
                        session.add(copy_of(i), now);
                        if !keys.contains(&i) {
                            keys.push(i);
                        }
                        last_used = Some(now);
                    }
                    Op::Replace(i) => {
                        session.replace(i.map(copy_of), now);
                        keys = i.into_iter().collect();
                        last_used = Some(now);
                        if i.is_none() {
                            prop_assert!(session.current().is_none());
                            prop_assert!(session.keys().is_empty());
                            prop_assert!(envelopes.iter().all(|e| session.fitting(e).is_none()));
                            prop_assert!(!session.idle(now + AUTO_LOCK * 2));
                        }
                    }
                    Op::Touch => {
                        session.touch(now);
                        if !keys.is_empty() {
                            last_used = Some(now);
                        }
                    }
                    Op::Tick(secs) => now += Duration::from_secs(secs),
                }
                prop_assert_eq!(session.is_unlocked(), !keys.is_empty());
                prop_assert_eq!(session.keys().len(), keys.len());
                prop_assert_eq!(
                    session.current().is_some_and(|key| keys.first().is_some_and(|&i| key.same_kdf(&KEYS[i]))),
                    !keys.is_empty()
                );
                for (i, envelope) in envelopes.iter().enumerate() {
                    prop_assert_eq!(session.fitting(envelope).is_some(), keys.contains(&i));
                }
                let model_idle = !keys.is_empty()
                    && last_used.is_some_and(|used| now.duration_since(used) >= AUTO_LOCK);
                prop_assert_eq!(session.idle(now), model_idle);
                prop_assert_eq!(session.last_used(), last_used);
            }
        }

        /// Invariant: a locked note shows its text only through a key the
        /// session holds that fits its seal. Otherwise it is closed and its
        /// body is empty, whatever stale text it carried in.
        #[test]
        fn prop_a_locked_note_is_never_exposed(
            body in "\\PC{1,40}",
            sealed_with in 0usize..4,
            held in prop::collection::vec(any::<bool>(), 3),
        ) {
            let mut sealer = Session::default();
            let now = Instant::now();
            let mut note = note("n1", &body);
            match sealed_with {
                0 | 1 => {
                    sealer.add(copy_of(sealed_with), now);
                    sealer.seal(&mut note, now).unwrap();
                }
                2 => {
                    sealer.add(copy_of(2), now);
                    sealer.seal(&mut note, now).unwrap();
                    // Not an envelope at all.
                    note.extra.insert(kanso_sync::lock::LOCK_KEY.to_owned(), json!({"v": 1}));
                }
                _ => {
                    sealer.add(copy_of(0), now);
                    sealer.seal(&mut note, now).unwrap();
                    // An envelope whose sealed text was damaged.
                    let mut envelope = Envelope::from_extra(&note.extra).unwrap().unwrap();
                    envelope.data = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into();
                    Envelope::store(Some(&envelope), &mut note.extra);
                }
            }
            prop_assert!(note.body.is_empty());
            let mut reader = Session::default();
            for (i, held) in held.iter().enumerate() {
                if *held {
                    reader.add(copy_of(i), now);
                }
            }
            note.body = "stale".into();
            let view = reader.reveal(&mut note, None);
            let opens = sealed_with < 2 && held[sealed_with];
            match view {
                LockView::Open => {
                    prop_assert!(opens);
                    prop_assert_eq!(note.body, body);
                }
                LockView::Closed(_) => {
                    prop_assert!(!opens);
                    prop_assert!(note.body.is_empty());
                }
                LockView::Plain => prop_assert!(false, "a locked note is never plain"),
            }
        }

        /// Round trip: whatever the text, sealing an open note empties its
        /// body, and revealing it gives the text back; saving keeps the
        /// notes open for another AUTO_LOCK.
        #[test]
        fn prop_seal_then_reveal_round_trips(
            body in "\\PC{0,200}",
            key in 0usize..3,
            later in 0u64..1200,
        ) {
            let base = Instant::now();
            let mut session = Session::default();
            session.add(copy_of(key), base);
            let mut note = note("n1", &body);
            let saved = base + Duration::from_secs(later);
            session.seal(&mut note, saved).unwrap();
            prop_assert!(note.body.is_empty());
            prop_assert!(note.is_locked());
            prop_assert_eq!(session.last_used(), Some(saved));
            prop_assert_eq!(session.reveal(&mut note, None), LockView::Open);
            prop_assert_eq!(&note.body, &body);
            // Revealing again gives the same text.
            prop_assert_eq!(session.reveal(&mut note, None), LockView::Open);
            prop_assert_eq!(&note.body, &body);
        }
    }
}
