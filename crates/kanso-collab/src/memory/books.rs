//! The Worker's shared-notebook routes in memory (`cloud/src/book.ts`,
//! `cloud/src/book_room.ts`): the chain of signed lists, the wraps and
//! history links, and the logs, whose sequence numbers share one count for
//! the whole notebook. Like the Worker it reads envelope headers and lists
//! and nothing else; the clients check the chain's signatures.

use std::collections::BTreeMap;

use serde_json::json;

use super::*;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

use crate::api::{
    BookCreated, BookEpochLink, BookInfo, BookListEntry, BookLiveEvent, BookLiveLink, BookLogPage,
    BookMemberStatus, BookSummary, MANIFEST_STREAM, NewBook, NewBookList,
};
use crate::book_members::{MAX_BOOK_MEMBERS, is_well_formed as list_well_formed, link_hash};
use crate::book_members_model::BookMemberList;

/// A shared notebook may hold this many notes.
pub const MAX_BOOK_NOTES: usize = 2_000;
const PAGE_LISTS: usize = 200;

/// The server's wraps (account, epoch, wrap) and history links (epoch, link).
pub type BookKeying = (Vec<(String, u32, String)>, Vec<(u32, String)>);

/// Something a test runs just before the server takes a notebook's next
/// list, as another client writing at that moment would.
#[derive(Default)]
pub(super) struct Hook(Option<Box<dyn FnOnce() + Send>>);

impl std::fmt::Debug for Hook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Hook")
    }
}

#[derive(Debug, Default)]
struct Stream {
    head: u64,
    /// `(seq, kind, envelope)`.
    log: Vec<(u64, Kind, Vec<u8>)>,
    /// `(base_seq, envelope)`.
    snapshot: Option<(u64, Vec<u8>)>,
}

#[derive(Debug)]
pub(super) struct Book {
    owner: String,
    epoch: u32,
    head_rev: u32,
    head_hash: String,
    /// `(epoch, signer, list, list_sig)` by position: rev - 1.
    lists: Vec<(u32, String, String, String)>,
    /// account → (role, status): `member`, `invited` or `left`.
    members: BTreeMap<String, (Role, &'static str)>,
    wraps: BTreeMap<(String, u32), String>,
    history: BTreeMap<u32, String>,
    /// The log's head: every stream's sequences share this count.
    head: u64,
    streams: BTreeMap<String, Stream>,
    /// Open live connections.
    sockets: Vec<BookSocket>,
}

#[derive(Debug)]
struct BookSocket {
    id: u64,
    account: String,
    /// The note streams opened on this socket (the manifest always is).
    open: HashSet<String>,
    tx: Sender<BookLiveEvent>,
}

impl Book {
    /// Tells every socket that has `stream` open (the manifest: all of
    /// them), except `except`.
    fn tell(&mut self, stream: &str, event: &BookLiveEvent, except: Option<u64>) {
        self.sockets.retain(|s| {
            let hears = stream == MANIFEST_STREAM || s.open.contains(stream);
            if !hears || Some(s.id) == except {
                return true;
            }
            s.tx.send(event.clone()).is_ok()
        });
    }
}

fn parse(list: &str) -> Option<(BookMemberList, Vec<u8>)> {
    let bytes = unb64(list)?;
    let parsed: BookMemberList = serde_json::from_slice(&bytes).ok()?;
    list_well_formed(&parsed).then_some((parsed, bytes))
}

fn conflict(book: &Book) -> CollabError {
    CollabError::api(
        409,
        "rev_conflict",
        json!({ "error": "rev_conflict", "rev": book.head_rev, "hash": book.head_hash }),
    )
}

impl State {
    fn store_book_entry(book: &mut Book, stream: &str, kind: Kind, envelope: &[u8]) -> u64 {
        book.head += 1;
        let seq = book.head;
        let entry = book.streams.entry(stream.to_owned()).or_default();
        entry.head = seq;
        entry.log.push((seq, kind, envelope.to_vec()));
        book.tell(
            stream,
            &BookLiveEvent::Stored {
                stream: stream.to_owned(),
                seq,
                envelope: envelope.to_vec(),
            },
            None,
        );
        seq
    }

    fn book(&mut self, id: &str) -> Result<&mut Book> {
        self.books.get_mut(id).ok_or_else(not_found)
    }

    /// The caller's role in a notebook, with `status` (`member` only unless
    /// `read`, which also takes an invitation).
    fn book_role(&self, id: &str, caller: &Caller, read: bool) -> Result<(Role, &'static str)> {
        let book = self.books.get(id).ok_or_else(not_found)?;
        match book.members.get(&caller.account_id) {
            Some((role, status)) if *status == "member" || (read && *status == "invited") => {
                Ok((*role, status))
            }
            _ => Err(not_found()),
        }
    }

    /// The wraps a list needs, as account → count, checked against `needed`.
    fn check_book_wraps(
        wraps: &[crate::api::AccountWrap],
        needed: &[&str],
    ) -> std::result::Result<(), CollabError> {
        let ok = wraps.len() == needed.len()
            && needed
                .iter()
                .all(|a| wraps.iter().filter(|w| w.account == *a).count() == 1);
        if ok { Ok(()) } else { Err(bad()) }
    }

    /// A listed member's keys and username must be their account's own.
    /// `stayers` are those the list before also named: one of them whose
    /// account is gone is `member_gone` (cut them), any other account
    /// without keys `unknown_member`.
    fn check_book_members(&self, list: &BookMemberList, stayers: &[String]) -> Result<()> {
        if self.lax_book_members {
            return Ok(());
        }
        for member in &list.members {
            match self.identities.get(&member.account) {
                None => {
                    let code = if stayers.contains(&member.account) {
                        "member_gone"
                    } else {
                        "unknown_member"
                    };
                    return Err(CollabError::api(
                        409,
                        code,
                        json!({ "error": code, "account": member.account }),
                    ));
                }
                Some((username, published))
                    if *username != member.username
                        || published.sign_pub != member.sign_pub
                        || published.box_pub != member.box_pub =>
                {
                    return Err(CollabError::api(
                        409,
                        "member_keys_mismatch",
                        json!({ "error": "member_keys_mismatch", "account": member.account }),
                    ));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }
}

impl MemoryCollab {
    /// A notebook's log head and how many streams it has (tests).
    pub fn book_room(&self, share_id: &str) -> Option<(u64, usize)> {
        let state = self.state.lock().expect("poisoned");
        let book = state.books.get(share_id)?;
        Some((book.head, book.streams.len()))
    }

    /// Every byte the server holds for notebooks: lists, wraps, history
    /// links, logs and snapshots. Tests look for plaintext in it.
    pub fn book_bytes(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().expect("poisoned");
        let mut out = Vec::new();
        for book in state.books.values() {
            out.extend(book.lists.iter().map(|l| unb64(&l.2).unwrap_or_default()));
            out.extend(book.wraps.values().map(|w| unb64(w).unwrap_or_default()));
            out.extend(book.history.values().map(|w| unb64(w).unwrap_or_default()));
            for stream in book.streams.values() {
                out.extend(stream.log.iter().map(|(_, _, e)| e.clone()));
                if let Some((_, snapshot)) = &stream.snapshot {
                    out.push(snapshot.clone());
                }
            }
        }
        out
    }

    /// Every log entry and snapshot the server holds for a notebook, as
    /// (stream, snapshot's base sequence, envelope) (tests).
    pub fn book_entries(&self, share_id: &str) -> Vec<(String, Option<u64>, Vec<u8>)> {
        let state = self.state.lock().expect("poisoned");
        let mut out = Vec::new();
        if let Some(book) = state.books.get(share_id) {
            for (name, stream) in &book.streams {
                for (_, _, envelope) in &stream.log {
                    out.push((name.clone(), None, envelope.clone()));
                }
                if let Some((base, envelope)) = &stream.snapshot {
                    out.push((name.clone(), Some(*base), envelope.clone()));
                }
            }
        }
        out
    }

    /// The server's wraps and history links for a notebook (tests):
    /// (account, epoch, wrap) and (epoch, link).
    pub fn book_keying(&self, share_id: &str) -> BookKeying {
        let state = self.state.lock().expect("poisoned");
        let Some(book) = state.books.get(share_id) else {
            return (Vec::new(), Vec::new());
        };
        (
            book.wraps
                .iter()
                .map(|((account, epoch), wrap)| (account.clone(), *epoch, wrap.clone()))
                .collect(),
            book.history.iter().map(|(e, l)| (*e, l.clone())).collect(),
        )
    }

    /// Runs `hook` once, just before the server takes the next list of any
    /// notebook (tests).
    pub fn before_book_list(&self, hook: impl FnOnce() + Send + 'static) {
        self.state.lock().expect("poisoned").book_list_hook = Hook(Some(Box::new(hook)));
    }

    /// Takes every history link away, as a server that lost them would
    /// (tests); [`MemoryCollab::put_book_history`] gives them back.
    pub fn take_book_history(&self, share_id: &str) -> BTreeMap<u32, String> {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        std::mem::take(&mut book.history)
    }

    pub fn put_book_history(&self, share_id: &str, links: BTreeMap<u32, String>) {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        book.history = links;
    }

    /// Takes an account's published identity away, as if the account were
    /// gone (tests); [`MemoryCollab::put_identity`] gives it back.
    pub fn take_identity(&self, account: &str) -> Option<(String, crate::identity::Published)> {
        self.state
            .lock()
            .expect("poisoned")
            .identities
            .remove(account)
    }

    pub fn put_identity(&self, account: &str, identity: (String, crate::identity::Published)) {
        self.state
            .lock()
            .expect("poisoned")
            .identities
            .insert(account.to_owned(), identity);
    }

    /// Stops the server checking that a list's members' keys are their
    /// accounts' own (tests).
    pub fn skip_book_member_checks(&self) {
        self.state.lock().expect("poisoned").lax_book_members = true;
    }

    /// Makes the server name `account` as the notebook's owner in what it
    /// answers (tests).
    pub fn set_book_owner(&self, share_id: &str, account: &str) {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        book.owner = account.to_owned();
    }

    /// Makes the server claim `by` more log entries than it has, as a
    /// server lying about its head would (tests).
    pub fn inflate_book_head(&self, share_id: &str, by: u64) {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        book.head += by;
    }

    /// Replaces a stored list, as a server serving another history would
    /// (tests): the chain a client checked before no longer matches.
    pub fn replace_book_list(&self, share_id: &str, rev: u32, list: &str, sig: &str) {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        let at = rev as usize - 1;
        let hash = parse(list)
            .map(|(_, bytes)| link_hash(&bytes))
            .expect("a list");
        book.lists[at].2 = list.to_owned();
        book.lists[at].3 = sig.to_owned();
        if rev == book.head_rev {
            book.head_hash = hash;
        }
    }

    /// Drops every list after `rev`, as a server rolled back to an older
    /// state would (tests).
    pub fn truncate_book_chain(&self, share_id: &str, rev: u32) {
        let mut state = self.state.lock().expect("poisoned");
        let book = state.books.get_mut(share_id).expect("a notebook");
        book.lists.truncate(rev as usize);
        book.head_rev = rev;
        let (_, bytes) = parse(&book.lists[rev as usize - 1].2).expect("a list");
        book.head_hash = link_hash(&bytes);
        book.epoch = book.lists[rev as usize - 1].0;
    }

    pub(super) fn create_book_impl(&self, caller: &Caller, new: &NewBook) -> Result<BookCreated> {
        let mut state = self.lock()?;
        if !state.has_identity(&caller.account_id) || !is_id(&new.share_id) {
            return Err(bad());
        }
        let (list, bytes) = parse(&new.list).ok_or_else(bad)?;
        if list.share != new.share_id
            || list.rev != 1
            || list.epoch != 1
            || list.owner != caller.account_id
            || list.signer != caller.account_id
        {
            return Err(bad());
        }
        if list.members.len() > MAX_BOOK_MEMBERS {
            return Err(quota("members"));
        }
        let hash = link_hash(&bytes);
        // The same notebook sent again (a lost answer) is answered again.
        if let Some(existing) = state.books.get(&new.share_id) {
            let same = existing.owner == caller.account_id
                && existing.lists.first().is_some_and(|l| l.3 == new.list_sig)
                && existing.head_hash.len() == 64
                && existing.lists[0].2 == new.list;
            return if same {
                Ok(BookCreated {
                    share_id: new.share_id.clone(),
                    epoch: 1,
                    rev: 1,
                })
            } else {
                Err(refused(409, "share_id_taken"))
            };
        }
        if state.shares.contains_key(&new.share_id) || state.retired.contains(&new.share_id) {
            return Err(refused(409, "share_id_taken"));
        }
        let owned = state
            .books
            .values()
            .filter(|b| b.owner == caller.account_id)
            .count();
        if owned >= 50 {
            return Err(quota("books"));
        }
        state.check_book_members(&list, &[])?;
        let accounts: Vec<&str> = list.members.iter().map(|m| m.account.as_str()).collect();
        State::check_book_wraps(&new.wraps, &accounts)?;
        let members = list
            .members
            .iter()
            .map(|m| {
                let status = if m.account == caller.account_id {
                    "member"
                } else {
                    "invited"
                };
                (m.account.clone(), (m.role, status))
            })
            .collect();
        let wraps = new
            .wraps
            .iter()
            .map(|w| ((w.account.clone(), 1), w.wrapped.clone()))
            .collect();
        state.books.insert(
            new.share_id.clone(),
            Book {
                owner: caller.account_id.clone(),
                epoch: 1,
                head_rev: 1,
                head_hash: hash,
                lists: vec![(
                    1,
                    caller.account_id.clone(),
                    new.list.clone(),
                    new.list_sig.clone(),
                )],
                members,
                wraps,
                history: BTreeMap::new(),
                head: 0,
                sockets: Vec::new(),
                streams: BTreeMap::new(),
            },
        );
        Ok(BookCreated {
            share_id: new.share_id.clone(),
            epoch: 1,
            rev: 1,
        })
    }

    pub(super) fn book_impl(
        &self,
        caller: &Caller,
        share_id: &str,
        since_rev: u32,
    ) -> Result<BookInfo> {
        let state = self.lock()?;
        let (role, status) = state.book_role(share_id, caller, true)?;
        let book = &state.books[share_id];
        let since = since_rev.max(1) as usize;
        let rows: Vec<&(u32, String, String, String)> = book
            .lists
            .iter()
            .skip(since - 1)
            .take(PAGE_LISTS + 1)
            .collect();
        let lists = rows
            .iter()
            .take(PAGE_LISTS)
            .enumerate()
            .map(|(i, (epoch, signer, list, sig))| BookListEntry {
                rev: (since + i) as u32,
                epoch: *epoch,
                signer: signer.clone(),
                list: list.clone(),
                list_sig: sig.clone(),
            })
            .collect();
        Ok(BookInfo {
            share_id: share_id.to_owned(),
            owner: book.owner.clone(),
            role,
            status: status.to_owned(),
            epoch: book.epoch,
            rev: book.head_rev,
            hash: book.head_hash.clone(),
            lists,
            more: rows.len() > PAGE_LISTS,
            wraps: book
                .wraps
                .iter()
                .filter(|((account, _), _)| *account == caller.account_id)
                .map(|((_, epoch), wrapped)| EpochWrap {
                    epoch: *epoch,
                    wrapped: wrapped.clone(),
                })
                .collect(),
            epochs: book
                .history
                .iter()
                .map(|(epoch, link)| BookEpochLink {
                    epoch: *epoch,
                    history_link: link.clone(),
                })
                .collect(),
            members: if status == "member" && role.may_manage() {
                book.members
                    .iter()
                    .map(|(account, (role, status))| BookMemberStatus {
                        account: account.clone(),
                        role: *role,
                        status: (*status).to_owned(),
                    })
                    .collect()
            } else {
                Vec::new()
            },
        })
    }

    pub(super) fn leave_book_impl(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        state.book_role(share_id, caller, true)?;
        let book = state.book(share_id)?;
        if book.owner == caller.account_id {
            return Err(refused(403, "forbidden_role"));
        }
        // The signed list still names them until a manager cuts them: they
        // stay as left, without wraps.
        book.wraps
            .retain(|(account, _), _| *account != caller.account_id);
        if let Some(entry) = book.members.get_mut(&caller.account_id) {
            entry.1 = "left";
        }
        Ok(())
    }

    pub(super) fn stop_book_impl(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let owned = state
            .books
            .get(share_id)
            .is_some_and(|book| book.owner == caller.account_id);
        if !owned {
            return Err(not_found());
        }
        if let Some(book) = state.books.remove(share_id) {
            for socket in &book.sockets {
                let _ = socket.tx.send(BookLiveEvent::Closed {
                    reason: "stopped".into(),
                });
            }
        }
        state.retired.insert(share_id.to_owned());
        Ok(())
    }

    pub(super) fn accept_book_impl(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        state.book_role(share_id, caller, true)?;
        let book = state.book(share_id)?;
        if let Some(entry) = book.members.get_mut(&caller.account_id) {
            entry.1 = "member";
        }
        Ok(())
    }

    pub(super) fn new_book_list_impl(
        &self,
        caller: &Caller,
        share_id: &str,
        new: &NewBookList,
    ) -> Result<()> {
        let hook = self.state.lock().expect("poisoned").book_list_hook.0.take();
        if let Some(hook) = hook {
            hook();
        }
        let mut state = self.lock()?;
        let (role, _) = state.book_role(share_id, caller, false)?;
        if !role.may_manage() {
            return Err(refused(403, "forbidden_role"));
        }
        let book = state.books.get(share_id).ok_or_else(not_found)?;
        if new.rev != book.head_rev + 1 {
            return Err(conflict(book));
        }
        let (list, bytes) = parse(&new.list).ok_or_else(bad)?;
        if list.share != share_id || list.rev != new.rev || list.owner != book.owner {
            return Err(bad());
        }
        if list.prev.as_deref() != Some(book.head_hash.as_str()) {
            return Err(conflict(book));
        }
        if list.signer != caller.account_id {
            return Err(refused(403, "forbidden_author"));
        }
        let (prev_list, _) = parse(&book.lists[book.head_rev as usize - 1].2).ok_or_else(bad)?;
        let new_epoch = list.epoch != prev_list.epoch;
        if new_epoch {
            if list.epoch != prev_list.epoch + 1
                || list.key_commit == prev_list.key_commit
                || list.from_seq < prev_list.from_seq
                || list.from_seq.is_none()
            {
                return Err(bad());
            }
            // The room checks that the list's position is its real head.
            if list.from_seq != Some(book.head) {
                return Err(CollabError::api(
                    409,
                    "head_moved",
                    json!({ "error": "head_moved", "head": book.head }),
                ));
            }
            if new.history_link.is_none() {
                return Err(bad());
            }
        } else if list.key_commit != prev_list.key_commit
            || list.from_seq != prev_list.from_seq
            || new.history_link.is_some()
        {
            return Err(bad());
        }
        if list.members.len() > MAX_BOOK_MEMBERS {
            return Err(quota("members"));
        }
        let stayers: Vec<String> = prev_list
            .members
            .iter()
            .map(|m| m.account.clone())
            .collect();
        state.check_book_members(&list, &stayers)?;
        let needed: Vec<&str> = list
            .members
            .iter()
            .filter(|m| {
                new_epoch
                    || prev_list
                        .member(&m.account)
                        .is_none_or(|old| old.role != m.role)
            })
            .map(|m| m.account.as_str())
            .collect();
        State::check_book_wraps(&new.wraps, &needed)?;
        let hash = link_hash(&bytes);
        let book = state.book(share_id)?;
        for account in book.members.keys().cloned().collect::<Vec<_>>() {
            if list.member(&account).is_none() {
                book.members.remove(&account);
                book.wraps.retain(|(a, _), _| *a != account);
            }
        }
        for member in &list.members {
            let entry = book
                .members
                .entry(member.account.clone())
                .or_insert((member.role, "invited"));
            entry.0 = member.role;
        }
        for wrap in &new.wraps {
            if book
                .members
                .get(&wrap.account)
                .is_some_and(|m| m.1 != "left")
            {
                book.wraps
                    .insert((wrap.account.clone(), list.epoch), wrap.wrapped.clone());
            }
        }
        if let Some(link) = &new.history_link {
            book.history.insert(list.epoch, link.clone());
        }
        let epoch_moved = book.epoch != list.epoch;
        book.sockets.retain(|s| {
            if list.member(&s.account).is_none() {
                let _ = s.tx.send(BookLiveEvent::Closed {
                    reason: "removed".into(),
                });
                return false;
            }
            !epoch_moved || s.tx.send(BookLiveEvent::Epoch(list.epoch)).is_ok()
        });
        book.epoch = list.epoch;
        book.head_rev = new.rev;
        book.head_hash = hash;
        book.lists.push((
            list.epoch,
            caller.account_id.clone(),
            new.list.clone(),
            new.list_sig.clone(),
        ));
        Ok(())
    }

    pub(super) fn book_log_impl(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        since: u64,
        limit: usize,
    ) -> Result<BookLogPage> {
        let state = self.lock()?;
        state.book_role(share_id, caller, false)?;
        if !is_id(stream) {
            return Err(bad());
        }
        let book = &state.books[share_id];
        let empty = Stream::default();
        let stream = book.streams.get(stream).unwrap_or(&empty);
        let snapshot = stream
            .snapshot
            .as_ref()
            .filter(|(base, _)| since == 0 || since < *base)
            .map(|(base, envelope)| SnapshotEntry {
                seq: *base,
                base_seq: *base,
                envelope: b64(envelope),
            });
        let limit = limit.clamp(1, 1000);
        let candidates: Vec<&(u64, Kind, Vec<u8>)> = stream
            .log
            .iter()
            .filter(|(seq, _, _)| *seq > since)
            .collect();
        let more = candidates.len() > limit;
        Ok(BookLogPage {
            head: book.head,
            stream_head: stream.head,
            snapshot,
            entries: candidates
                .into_iter()
                .take(limit)
                .map(|(seq, _, e)| LogEntry {
                    seq: *seq,
                    envelope: b64(e),
                })
                .collect(),
            more,
        })
    }

    /// What a write to a stream must pass: the caller a member with a role
    /// that may write the kind, the envelope its own and of the notebook's
    /// epoch, and the size.
    fn admit_book_write(
        state: &State,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        envelope: &[u8],
        snapshot: bool,
    ) -> Result<(Kind, bool)> {
        let (role, _) = state.book_role(share_id, caller, false)?;
        if !is_id(stream) {
            return Err(bad());
        }
        let (header, _) = parse_header(envelope).ok_or_else(bad)?;
        let manifest = stream == MANIFEST_STREAM;
        let allowed = match header.kind {
            Kind::Update => !snapshot && role.may_edit(),
            Kind::Snapshot => snapshot && role.may_edit(),
            Kind::Comment => !snapshot && !manifest && role.may_comment(),
            Kind::Presence => return Err(bad()),
        };
        if (header.kind == Kind::Snapshot) != snapshot {
            return Err(bad());
        }
        if header.author_account != caller.account_id || header.author_device != caller.device_id {
            return Err(refused(403, "forbidden_author"));
        }
        if !allowed {
            return Err(refused(403, "forbidden_role"));
        }
        if envelope.len() > max_size(header.kind) {
            return Err(refused(413, "too_large"));
        }
        let book = &state.books[share_id];
        if header.epoch != book.epoch {
            return Err(CollabError::api(
                409,
                "epoch_changed",
                json!({ "error": "epoch_changed", "epoch": book.epoch }),
            ));
        }
        if !manifest
            && !book.streams.contains_key(stream)
            && book.streams.len() - usize::from(book.streams.contains_key(MANIFEST_STREAM))
                >= MAX_BOOK_NOTES
        {
            return Err(quota("notes"));
        }
        Ok((header.kind, manifest))
    }

    pub(super) fn book_append_impl(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        envelope: &[u8],
    ) -> Result<u64> {
        let mut state = self.lock()?;
        MemoryCollab::admit_book_write(&state, caller, share_id, stream, envelope, false)?;
        let (kind, _) = parse_header(envelope)
            .map(|(h, _)| (h.kind, ()))
            .ok_or_else(bad)?;
        let seq = State::store_book_entry(state.book(share_id)?, stream, kind, envelope);
        if state.lost_replies > 0 {
            state.lost_replies -= 1;
            return Err(CollabError::Transport("reply lost (test)".into()));
        }
        Ok(seq)
    }

    /// The caller's notebooks (`GET /v2/collab/books`).
    pub(super) fn books_list(&self, caller: &Caller) -> Result<Vec<BookSummary>> {
        let state = self.lock()?;
        let mut out: Vec<BookSummary> = state
            .books
            .iter()
            .filter_map(|(id, book)| {
                let (role, status) = book.members.get(&caller.account_id)?;
                (*status != "left").then(|| BookSummary {
                    share_id: id.clone(),
                    owner: book.owner.clone(),
                    owner_username: state
                        .identities
                        .get(&book.owner)
                        .map(|(name, _)| name.clone())
                        .unwrap_or_default(),
                    role: *role,
                    status: (*status).to_owned(),
                    epoch: book.epoch,
                    rev: book.head_rev,
                })
            })
            .collect();
        out.sort_by(|a, b| a.share_id.cmp(&b.share_id));
        Ok(out)
    }

    pub(super) fn book_live_impl(
        &self,
        caller: &Caller,
        share_id: &str,
    ) -> Result<Box<dyn BookLiveLink>> {
        let mut state = self.lock()?;
        state.book_role(share_id, caller, false)?;
        state.next_socket += 1;
        let id = state.next_socket;
        let (tx, rx) = channel();
        let book = state.book(share_id)?;
        let _ = tx.send(BookLiveEvent::Hello {
            head: book.head,
            epoch: book.epoch,
        });
        book.sockets.push(BookSocket {
            id,
            account: caller.account_id.clone(),
            open: HashSet::new(),
            tx,
        });
        Ok(Box::new(MemoryBookLink {
            server: self.clone(),
            caller: caller.clone(),
            share_id: share_id.to_owned(),
            socket: id,
            rx,
        }))
    }

    pub(super) fn book_snapshot_impl(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64> {
        let mut state = self.lock()?;
        MemoryCollab::admit_book_write(&state, caller, share_id, stream, envelope, true)?;
        let book = state.book(share_id)?;
        let head = book.streams.get(stream).map_or(0, |s| s.head);
        if base_seq != head {
            return Err(CollabError::api(
                409,
                "head_moved",
                json!({ "error": "head_moved", "head": head }),
            ));
        }
        let entry = book.streams.entry(stream.to_owned()).or_default();
        entry
            .log
            .retain(|(seq, kind, _)| *kind == Kind::Comment || *seq > base_seq);
        entry.snapshot = Some((base_seq, envelope.to_vec()));
        Ok(base_seq)
    }
}

/// A live connection to a [`MemoryCollab`] notebook room.
struct MemoryBookLink {
    server: MemoryCollab,
    caller: Caller,
    share_id: String,
    socket: u64,
    rx: Receiver<BookLiveEvent>,
}

impl MemoryBookLink {
    fn refuse(&self, state: &mut State, code: &str) {
        if let Some(book) = state.books.get_mut(&self.share_id)
            && let Some(mine) = book.sockets.iter().find(|s| s.id == self.socket)
        {
            let _ = mine.tx.send(BookLiveEvent::Error { code: code.into() });
        }
    }
}

impl BookLiveLink for MemoryBookLink {
    fn open(&mut self, stream: &str) -> Result<()> {
        let mut state = self.server.state.lock().expect("poisoned");
        let book = state
            .books
            .get_mut(&self.share_id)
            .ok_or_else(|| CollabError::Transport("the room closed".into()))?;
        let head = book.streams.get(stream).map_or(0, |s| s.head);
        let Some(mine) = book.sockets.iter_mut().find(|s| s.id == self.socket) else {
            return Err(CollabError::Transport("the room closed".into()));
        };
        mine.open.insert(stream.to_owned());
        let _ = mine.tx.send(BookLiveEvent::Opened {
            stream: stream.to_owned(),
            head,
        });
        Ok(())
    }

    fn close(&mut self, stream: &str) -> Result<()> {
        let mut state = self.server.state.lock().expect("poisoned");
        if let Some(book) = state.books.get_mut(&self.share_id)
            && let Some(mine) = book.sockets.iter_mut().find(|s| s.id == self.socket)
        {
            mine.open.remove(stream);
        }
        Ok(())
    }

    fn send(&mut self, stream: &str, envelope: &[u8]) -> Result<()> {
        let mut state = self.server.state.lock().expect("poisoned");
        let open = state
            .books
            .get(&self.share_id)
            .and_then(|b| b.sockets.iter().find(|s| s.id == self.socket))
            .map(|s| stream == MANIFEST_STREAM || s.open.contains(stream));
        let Some(open) = open else {
            return Err(CollabError::Transport("the room closed".into()));
        };
        let Some((header, _)) = parse_header(envelope) else {
            self.refuse(&mut state, "bad_request");
            return Ok(());
        };
        if !open {
            self.refuse(&mut state, "not_open");
            return Ok(());
        }
        if header.kind == Kind::Presence {
            let member = state.book_role(&self.share_id, &self.caller, false).is_ok();
            if !member
                || header.author_account != self.caller.account_id
                || envelope.len() > max_size(Kind::Presence)
            {
                self.refuse(&mut state, "forbidden_role");
                return Ok(());
            }
            if let Some(book) = state.books.get_mut(&self.share_id) {
                book.tell(
                    stream,
                    &BookLiveEvent::Presence {
                        stream: stream.to_owned(),
                        envelope: envelope.to_vec(),
                    },
                    Some(self.socket),
                );
            }
            return Ok(());
        }
        match MemoryCollab::admit_book_write(
            &state,
            &self.caller,
            &self.share_id,
            stream,
            envelope,
            false,
        ) {
            Ok((kind, _)) => {
                if let Some(book) = state.books.get_mut(&self.share_id) {
                    State::store_book_entry(book, stream, kind, envelope);
                }
            }
            Err(err) => {
                let code = err.code().unwrap_or("bad_request").to_owned();
                self.refuse(&mut state, &code);
            }
        }
        Ok(())
    }

    fn recv(&mut self, wait: Duration) -> Result<Option<BookLiveEvent>> {
        match self.rx.recv_timeout(wait) {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => {
                Err(CollabError::Transport("the room closed".into()))
            }
        }
    }
}

impl Drop for MemoryBookLink {
    fn drop(&mut self) {
        if let Ok(mut state) = self.server.state.lock()
            && let Some(book) = state.books.get_mut(&self.share_id)
        {
            book.sockets.retain(|s| s.id != self.socket);
        }
    }
}
