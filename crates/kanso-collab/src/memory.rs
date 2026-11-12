//! [`CollabApi`] in memory, for tests: the Worker's rules for shared notes
//! (docs/ACCOUNTS.md › Collaboration › The server) without a server, rooms
//! and live connections included. Like the Worker it reads envelope headers
//! and member lists, and nothing else.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::json;

use crate::api::Caller;
use crate::api::{
    BookCreated, BookInfo, BookLogPage, CollabApi, CollabError, Created, EpochList, EpochWrap,
    JoinRequest, Joined, LinkInfo, LiveEvent, LiveLink, LogEntry, LogPage, MemberState, NewBook,
    NewBookList, NewEpoch, NewLink, NewShare, NoticeIn, NoticeOut, Result, ShareInfo,
    SnapshotEntry,
};
use crate::codec::{b64, hex, is_id, sha256, unb64};
use crate::envelope::{max_size, parse_header};
use crate::identity::Published;
use crate::members::{MAX_MEMBERS, is_well_formed};
use crate::model::{Kind, MemberList, PublicIdentity, Role};

mod books;

pub use books::MAX_BOOK_NOTES;

pub const MAX_SINCE_SNAPSHOT: u64 = 10_000;
pub const MAX_BYTES_SINCE_SNAPSHOT: u64 = 16 * 1024 * 1024;
pub const MAX_OWNED_SHARES: usize = 200;
pub const MAX_LINKS: usize = 10;
pub const MAX_REQUESTS: usize = 50;
pub const MAX_NOTICES: usize = 500;
pub const MAX_LINK_MS: i64 = 30 * 86_400_000;

#[derive(Clone, Debug)]
struct Membership {
    role: Role,
    /// `member` or `invited`.
    status: &'static str,
}

#[derive(Debug, Default)]
struct Room {
    head: u64,
    /// `(seq, kind, envelope)`.
    log: Vec<(u64, Kind, Vec<u8>)>,
    snapshot: Option<(u64, Vec<u8>)>,
    since_snapshot: u64,
    bytes_since_snapshot: u64,
    sockets: Vec<Socket>,
}

#[derive(Debug)]
struct Socket {
    id: u64,
    account: String,
    tx: Sender<LiveEvent>,
}

#[derive(Debug)]
struct Share {
    owner: String,
    epoch: u32,
    list: String,
    list_sig: String,
    /// Every epoch's list and signature.
    lists: BTreeMap<u32, (String, String)>,
    members: BTreeMap<String, Membership>,
    wraps: BTreeMap<(String, u32), String>,
    /// link ID → (proof hash, role, expires at).
    links: BTreeMap<String, (String, Role, i64)>,
    /// account → (link ID, asked at).
    requests: BTreeMap<String, (String, i64)>,
    room: Room,
}

#[derive(Debug, Default)]
struct State {
    /// account → (username, identity).
    identities: HashMap<String, (String, Published)>,
    shares: BTreeMap<String, Share>,
    books: BTreeMap<String, books::Book>,
    retired: HashSet<String>,
    /// (recipient, notice).
    notices: Vec<(String, NoticeOut)>,
    next_notice: u64,
    next_socket: u64,
    offline: usize,
    /// Requests that succeed before `offline` begins.
    grace: usize,
    now: i64,
}

/// The Worker's shared-note routes in memory.
#[derive(Clone, Debug, Default)]
pub struct MemoryCollab {
    state: Arc<Mutex<State>>,
}

fn refused(status: u16, code: &str) -> CollabError {
    CollabError::api(status, code, json!({ "error": code }))
}

fn not_found() -> CollabError {
    refused(404, "not_found")
}

/// A cap, as the Worker answers it (its `used` and `limit` aren't kept here).
fn quota(kind: &str) -> CollabError {
    CollabError::api(
        413,
        "quota_exceeded",
        json!({ "error": "quota_exceeded", "quota": kind }),
    )
}

fn bad() -> CollabError {
    refused(400, "bad_request")
}

impl MemoryCollab {
    pub fn new() -> MemoryCollab {
        let server = MemoryCollab::default();
        server.set_now(kanso_core::now_ms());
        server
    }

    fn lock(&self) -> Result<MutexGuard<'_, State>> {
        let mut state = self.state.lock().expect("memory server poisoned");
        if state.grace > 0 {
            state.grace -= 1;
        } else if state.offline > 0 {
            state.offline -= 1;
            return Err(CollabError::Transport("offline (test)".into()));
        }
        Ok(state)
    }

    /// Deletes an account as the Worker does: its identity (looking it up
    /// finds nobody) and its memberships go; the signed lists that name it
    /// stay until their owners start the next epoch.
    pub fn delete_account(&self, account_id: &str) {
        let mut state = self.state.lock().expect("poisoned");
        state.identities.remove(account_id);
        for share in state.shares.values_mut() {
            share.members.remove(account_id);
            share.wraps.retain(|(account, _), _| account != account_id);
        }
    }

    /// The next `n` requests fail as if offline.
    pub fn go_offline(&self, n: usize) {
        self.go_offline_after(0, n);
    }

    /// The next `ok` requests succeed, then `n` fail as if offline.
    pub fn go_offline_after(&self, ok: usize, n: usize) {
        let mut state = self.state.lock().expect("poisoned");
        state.grace = ok;
        state.offline = n;
    }

    /// The server's clock, for link expiry (Unix ms).
    pub fn set_now(&self, now: i64) {
        self.state.lock().expect("poisoned").now = now;
    }

    /// Every byte the server holds for shared notes: identities, lists,
    /// wraps, logs, snapshots and notifications. Tests look for plaintext in
    /// it.
    pub fn stored_bytes(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().expect("poisoned");
        let mut out = Vec::new();
        for (_, published) in state.identities.values() {
            out.push(serde_json::to_vec(published).unwrap_or_default());
        }
        for share in state.shares.values() {
            out.push(unb64(&share.list).unwrap_or_default());
            out.extend(share.wraps.values().map(|w| unb64(w).unwrap_or_default()));
            out.extend(share.room.log.iter().map(|(_, _, e)| e.clone()));
            if let Some((_, snapshot)) = &share.room.snapshot {
                out.push(snapshot.clone());
            }
        }
        for (_, notice) in &state.notices {
            out.push(unb64(&notice.sealed).unwrap_or_default());
        }
        out
    }

    /// A share's log head and how many entries its room holds (tests).
    pub fn room(&self, share_id: &str) -> Option<(u64, usize)> {
        let state = self.state.lock().expect("poisoned");
        let share = state.shares.get(share_id)?;
        Some((share.room.head, share.room.log.len()))
    }

    /// Whether a share has a snapshot past its first (tests).
    pub fn snapshot_base(&self, share_id: &str) -> Option<u64> {
        let state = self.state.lock().expect("poisoned");
        state
            .shares
            .get(share_id)?
            .room
            .snapshot
            .as_ref()
            .map(|(b, _)| *b)
    }
}

fn parse_list(list: &str) -> Option<MemberList> {
    let parsed: MemberList = serde_json::from_slice(&unb64(list)?).ok()?;
    is_well_formed(&parsed).then_some(parsed)
}

impl State {
    fn share(&mut self, id: &str) -> Result<&mut Share> {
        self.shares.get_mut(id).ok_or_else(not_found)
    }

    fn has_identity(&self, account: &str) -> bool {
        self.identities.contains_key(account)
    }

    /// The caller's role in a share it is a member of (not merely invited).
    fn role(&self, id: &str, caller: &Caller) -> Result<Role> {
        let share = self.shares.get(id).ok_or_else(not_found)?;
        match share.members.get(&caller.account_id) {
            Some(m) if m.status == "member" => Ok(m.role),
            _ => Err(not_found()),
        }
    }

    /// Checks a list and its wraps against the share: the list's epoch,
    /// share and owner, every member known with an identity, a wrap for
    /// each.
    fn check_epoch(
        &self,
        id: &str,
        caller: &Caller,
        list: &str,
        wraps: &[crate::api::AccountWrap],
        epoch: u32,
    ) -> Result<MemberList> {
        let parsed = parse_list(list).ok_or_else(bad)?;
        if parsed.share != id || parsed.epoch != epoch || parsed.owner != caller.account_id {
            return Err(bad());
        }
        if parsed.members.len() > MAX_MEMBERS {
            return Err(quota("members"));
        }
        for member in &parsed.members {
            // A listed member's keys and username are their identity's.
            match self.identities.get(&member.account) {
                None => return Err(bad()),
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
            let current = wraps
                .iter()
                .filter(|w| w.account == member.account && w.epoch.is_none_or(|e| e == epoch))
                .count();
            if current != 1 {
                return Err(bad());
            }
        }
        let earlier_ok = wraps.iter().all(|w| {
            w.epoch.is_none_or(|e| e >= 1 && e <= epoch)
                && parsed.members.iter().any(|m| m.account == w.account)
        });
        if !earlier_ok || wraps.len() > (20 * epoch as usize).min(400) {
            return Err(bad());
        }
        Ok(parsed)
    }

    fn broadcast(room: &mut Room, event: &LiveEvent, except: Option<u64>) {
        room.sockets
            .retain(|s| Some(s.id) == except || s.tx.send(event.clone()).is_ok());
    }

    fn close(room: &mut Room, accounts: Option<&HashSet<String>>, reason: &str) {
        room.sockets.retain(|s| {
            let closing = accounts.is_none_or(|a| a.contains(&s.account));
            if closing {
                let _ = s.tx.send(LiveEvent::Closed {
                    reason: reason.into(),
                });
            }
            !closing
        });
    }

    /// Appends an update or comment for `caller`; the stored sequence.
    fn append(&mut self, id: &str, caller: &Caller, envelope: &[u8]) -> Result<u64> {
        let role = self.role(id, caller)?;
        let share = self.share(id)?;
        let (header, _) = parse_header(envelope).ok_or_else(bad)?;
        let allowed = match header.kind {
            Kind::Update => role.may_edit(),
            Kind::Comment => role.may_comment(),
            Kind::Snapshot => return Err(bad()),
            Kind::Presence => return Err(bad()),
        };
        if header.author_account != caller.account_id || header.author_device != caller.device_id {
            return Err(refused(403, "forbidden_author"));
        }
        if !allowed {
            return Err(refused(403, "forbidden_role"));
        }
        if envelope.len() > max_size(header.kind) {
            return Err(refused(413, "too_large"));
        }
        if header.epoch != share.epoch {
            return Err(CollabError::api(
                409,
                "epoch_changed",
                json!({ "error": "epoch_changed", "epoch": share.epoch }),
            ));
        }
        let room = &mut share.room;
        if room.since_snapshot >= MAX_SINCE_SNAPSHOT
            || room.bytes_since_snapshot >= MAX_BYTES_SINCE_SNAPSHOT
        {
            return Err(refused(409, "compact_first"));
        }
        room.head += 1;
        let seq = room.head;
        room.log.push((seq, header.kind, envelope.to_vec()));
        if header.kind == Kind::Update {
            room.since_snapshot += 1;
            room.bytes_since_snapshot += envelope.len() as u64;
        }
        State::broadcast(
            room,
            &LiveEvent::Stored {
                seq,
                envelope: envelope.to_vec(),
            },
            None,
        );
        Ok(seq)
    }
}

impl CollabApi for MemoryCollab {
    fn put_identity(&self, caller: &Caller, published: &Published) -> Result<()> {
        let mut state = self.lock()?;
        if !crate::identity::is_published_shape(published) {
            return Err(bad());
        }
        match state.identities.get(&caller.account_id) {
            Some((_, existing)) if existing == published => Ok(()),
            Some(_) => Err(refused(409, "identity_exists")),
            None => {
                state.identities.insert(
                    caller.account_id.clone(),
                    (caller.username.clone(), published.clone()),
                );
                Ok(())
            }
        }
    }

    fn get_identity(&self, caller: &Caller) -> Result<Option<Published>> {
        let state = self.lock()?;
        Ok(state
            .identities
            .get(&caller.account_id)
            .map(|(_, p)| p.clone()))
    }

    fn lookup(&self, _caller: &Caller, username: &str) -> Result<Option<PublicIdentity>> {
        let state = self.lock()?;
        let wanted = username.trim().to_ascii_lowercase();
        Ok(state
            .identities
            .iter()
            .find(|(_, (name, _))| *name == wanted)
            .map(|(account, (name, p))| PublicIdentity {
                account_id: account.clone(),
                username: name.clone(),
                sign_pub: p.sign_pub.clone(),
                box_pub: p.box_pub.clone(),
                binding: p.binding.clone(),
            }))
    }

    fn create_share(&self, caller: &Caller, share: &NewShare) -> Result<Created> {
        let mut state = self.lock()?;
        if !state.has_identity(&caller.account_id) || !is_id(&share.share_id) {
            return Err(bad());
        }
        if state.shares.contains_key(&share.share_id) || state.retired.contains(&share.share_id) {
            return Err(refused(409, "share_id_taken"));
        }
        let owned = state
            .shares
            .values()
            .filter(|s| s.owner == caller.account_id)
            .count();
        if owned >= MAX_OWNED_SHARES {
            return Err(refused(413, "quota_exceeded"));
        }
        let list = state.check_epoch(&share.share_id, caller, &share.list, &share.wraps, 1)?;
        let snapshot = unb64(&share.snapshot).ok_or_else(bad)?;
        let (header, _) = parse_header(&snapshot).ok_or_else(bad)?;
        if header.kind != Kind::Snapshot
            || header.epoch != 1
            || header.author_account != caller.account_id
            || snapshot.len() > max_size(Kind::Snapshot)
        {
            return Err(bad());
        }
        let members = list
            .members
            .iter()
            .map(|m| {
                let status = if m.account == caller.account_id {
                    "member"
                } else {
                    "invited"
                };
                (
                    m.account.clone(),
                    Membership {
                        role: m.role,
                        status,
                    },
                )
            })
            .collect();
        let wraps = share
            .wraps
            .iter()
            .map(|w| ((w.account.clone(), w.epoch.unwrap_or(1)), w.wrapped.clone()))
            .collect();
        state.shares.insert(
            share.share_id.clone(),
            Share {
                owner: caller.account_id.clone(),
                epoch: 1,
                list: share.list.clone(),
                list_sig: share.list_sig.clone(),
                lists: BTreeMap::from([(1, (share.list.clone(), share.list_sig.clone()))]),
                members,
                wraps,
                links: BTreeMap::new(),
                requests: BTreeMap::new(),
                room: Room {
                    snapshot: Some((0, snapshot)),
                    ..Room::default()
                },
            },
        );
        Ok(Created {
            share_id: share.share_id.clone(),
            epoch: 1,
            head: 0,
        })
    }

    fn shares(&self, caller: &Caller) -> Result<Vec<ShareInfo>> {
        let state = self.lock()?;
        let mut out = Vec::new();
        for (id, share) in &state.shares {
            let Some(me) = share.members.get(&caller.account_id) else {
                continue;
            };
            let owner_username = state
                .identities
                .get(&share.owner)
                .map(|(n, _)| n.clone())
                .unwrap_or_default();
            let owner = share.owner == caller.account_id;
            out.push(ShareInfo {
                share_id: id.clone(),
                owner: share.owner.clone(),
                owner_username,
                role: me.role,
                status: me.status.into(),
                epoch: share.epoch,
                list: share.list.clone(),
                list_sig: share.list_sig.clone(),
                wraps: share
                    .wraps
                    .iter()
                    .filter(|((account, _), _)| *account == caller.account_id)
                    .map(|((_, epoch), wrapped)| EpochWrap {
                        epoch: *epoch,
                        wrapped: wrapped.clone(),
                    })
                    .collect(),
                lists: share
                    .lists
                    .iter()
                    .filter(|(epoch, _)| {
                        **epoch == share.epoch
                            || share
                                .wraps
                                .contains_key(&(caller.account_id.clone(), **epoch))
                    })
                    .map(|(epoch, (list, list_sig))| EpochList {
                        epoch: *epoch,
                        list: list.clone(),
                        list_sig: list_sig.clone(),
                    })
                    .collect(),
                head: share.room.head,
                members: if owner {
                    share
                        .members
                        .iter()
                        .map(|(account, m)| MemberState {
                            account: account.clone(),
                            status: m.status.into(),
                        })
                        .collect()
                } else {
                    Vec::new()
                },
                links: if owner {
                    share
                        .links
                        .iter()
                        .filter(|(_, (_, _, expires))| *expires > state.now)
                        .map(|(link_id, (_, role, expires_at))| LinkInfo {
                            link_id: link_id.clone(),
                            role: *role,
                            created_at: 0,
                            expires_at: *expires_at,
                        })
                        .collect()
                } else {
                    Vec::new()
                },
                requests: if owner {
                    share
                        .requests
                        .iter()
                        .filter_map(|(account, (link_id, at))| {
                            let (username, p) = state.identities.get(account)?;
                            Some(JoinRequest {
                                link_id: link_id.clone(),
                                account: account.clone(),
                                username: username.clone(),
                                role: share.links.get(link_id).map_or(Role::View, |l| l.1),
                                sign_pub: p.sign_pub.clone(),
                                box_pub: p.box_pub.clone(),
                                binding: p.binding.clone(),
                                at: *at,
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                },
            });
        }
        Ok(out)
    }

    fn accept(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let share = state.share(share_id)?;
        match share.members.get_mut(&caller.account_id) {
            Some(m) => {
                m.status = "member";
                Ok(())
            }
            None => Err(not_found()),
        }
    }

    fn leave(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let share = state.share(share_id)?;
        if share.owner == caller.account_id {
            return Err(refused(403, "forbidden_role"));
        }
        if share.members.remove(&caller.account_id).is_none() {
            return Err(not_found());
        }
        share
            .wraps
            .retain(|(account, _), _| *account != caller.account_id);
        let gone = HashSet::from([caller.account_id.clone()]);
        State::close(&mut share.room, Some(&gone), "removed");
        Ok(())
    }

    fn new_epoch(&self, caller: &Caller, share_id: &str, epoch: &NewEpoch) -> Result<u32> {
        let mut state = self.lock()?;
        let current = {
            let share = state.share(share_id)?;
            if share.owner != caller.account_id {
                return Err(not_found());
            }
            share.epoch
        };
        if epoch.epoch != current + 1 {
            return Err(CollabError::api(
                409,
                "epoch_changed",
                json!({ "error": "epoch_changed", "epoch": current }),
            ));
        }
        let list = state.check_epoch(share_id, caller, &epoch.list, &epoch.wraps, epoch.epoch)?;
        let share = state.share(share_id)?;
        // The epoch starts from the log's head as it is now.
        if list.from_seq != Some(share.room.head) {
            return Err(CollabError::api(
                409,
                "head_moved",
                json!({ "error": "head_moved", "head": share.room.head }),
            ));
        }
        let listed: HashSet<String> = list.members.iter().map(|m| m.account.clone()).collect();
        let removed: HashSet<String> = share
            .members
            .keys()
            .filter(|a| !listed.contains(*a))
            .cloned()
            .collect();
        share.members.retain(|a, _| listed.contains(a));
        share.wraps.retain(|(a, _), _| listed.contains(a));
        for member in &list.members {
            let asked = share.requests.remove(&member.account).is_some();
            let entry = share
                .members
                .entry(member.account.clone())
                .or_insert(Membership {
                    role: member.role,
                    status: if asked { "member" } else { "invited" },
                });
            entry.role = member.role;
        }
        for wrap in &epoch.wraps {
            share.wraps.insert(
                (wrap.account.clone(), wrap.epoch.unwrap_or(epoch.epoch)),
                wrap.wrapped.clone(),
            );
        }
        share.epoch = epoch.epoch;
        share.list = epoch.list.clone();
        share.list_sig = epoch.list_sig.clone();
        share
            .lists
            .insert(epoch.epoch, (epoch.list.clone(), epoch.list_sig.clone()));
        State::close(&mut share.room, Some(&removed), "removed");
        State::broadcast(&mut share.room, &LiveEvent::Epoch(epoch.epoch), None);
        Ok(epoch.epoch)
    }

    fn stop(&self, caller: &Caller, share_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let share = state.share(share_id)?;
        if share.owner != caller.account_id {
            return Err(not_found());
        }
        State::close(&mut share.room, None, "stopped");
        state.shares.remove(share_id);
        state.retired.insert(share_id.to_owned());
        state.notices.retain(|(_, n)| n.share_id != share_id);
        Ok(())
    }

    fn log(&self, caller: &Caller, share_id: &str, since: u64, limit: usize) -> Result<LogPage> {
        let mut state = self.lock()?;
        state.role(share_id, caller)?;
        let room = &state.share(share_id)?.room;
        let base = room.snapshot.as_ref().map_or(0, |(b, _)| *b);
        let snapshot = room
            .snapshot
            .as_ref()
            .filter(|(b, _)| since < *b || (since == 0 && *b == 0))
            .map(|(b, e)| SnapshotEntry {
                seq: *b,
                base_seq: *b,
                envelope: b64(e),
            });
        let after = if snapshot.is_some() { base } else { since };
        let limit = limit.clamp(1, 1000);
        let candidates: Vec<&(u64, Kind, Vec<u8>)> = room
            .log
            .iter()
            .filter(|(seq, kind, _)| *seq > since && (*kind == Kind::Comment || *seq > after))
            .collect();
        let more = candidates.len() > limit;
        let entries = candidates
            .into_iter()
            .take(limit)
            .map(|(seq, _, e)| LogEntry {
                seq: *seq,
                envelope: b64(e),
            })
            .collect();
        Ok(LogPage {
            head: room.head,
            snapshot,
            entries,
            more,
        })
    }

    fn append(&self, caller: &Caller, share_id: &str, envelope: &[u8]) -> Result<u64> {
        let mut state = self.lock()?;
        state.append(share_id, caller, envelope)
    }

    fn snapshot(
        &self,
        caller: &Caller,
        share_id: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64> {
        let mut state = self.lock()?;
        let role = state.role(share_id, caller)?;
        let share = state.share(share_id)?;
        let (header, _) = parse_header(envelope).ok_or_else(bad)?;
        if header.kind != Kind::Snapshot || header.author_account != caller.account_id {
            return Err(bad());
        }
        if !role.may_edit() {
            return Err(refused(403, "forbidden_role"));
        }
        if envelope.len() > max_size(Kind::Snapshot) {
            return Err(refused(413, "too_large"));
        }
        if header.epoch != share.epoch {
            return Err(CollabError::api(
                409,
                "epoch_changed",
                json!({ "error": "epoch_changed", "epoch": share.epoch }),
            ));
        }
        let room = &mut share.room;
        if base_seq != room.head {
            return Err(CollabError::api(
                409,
                "head_moved",
                json!({ "error": "head_moved", "head": room.head }),
            ));
        }
        room.log
            .retain(|(seq, kind, _)| *kind == Kind::Comment || *seq > base_seq);
        room.snapshot = Some((base_seq, envelope.to_vec()));
        room.since_snapshot = 0;
        room.bytes_since_snapshot = 0;
        Ok(base_seq)
    }

    fn create_link(&self, caller: &Caller, share_id: &str, link: &NewLink) -> Result<()> {
        let mut state = self.lock()?;
        let now = state.now;
        let share = state.share(share_id)?;
        if share.owner != caller.account_id {
            return Err(not_found());
        }
        if !is_id(&link.link_id)
            || link.role == Role::Owner
            || link.proof_hash.len() != 64
            || link.expires_at <= now
            || link.expires_at > now + MAX_LINK_MS
        {
            return Err(bad());
        }
        share.links.retain(|_, (_, _, expires)| *expires > now);
        if share.links.len() >= MAX_LINKS {
            return Err(quota("links"));
        }
        share.links.insert(
            link.link_id.clone(),
            (link.proof_hash.clone(), link.role, link.expires_at),
        );
        Ok(())
    }

    fn delete_link(&self, caller: &Caller, share_id: &str, link_id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let share = state.share(share_id)?;
        if share.owner != caller.account_id || share.links.remove(link_id).is_none() {
            return Err(not_found());
        }
        Ok(())
    }

    fn join(&self, caller: &Caller, link_id: &str, proof: &str) -> Result<Joined> {
        let mut state = self.lock()?;
        if !state.has_identity(&caller.account_id) {
            return Err(bad());
        }
        let now = state.now;
        let proof_hash = unb64(proof).map(|p| hex(&sha256(&[&p])));
        let owner_names: HashMap<String, String> = state
            .identities
            .iter()
            .map(|(a, (n, _))| (a.clone(), n.clone()))
            .collect();
        for (id, share) in state.shares.iter_mut() {
            let Some((hash, role, expires)) = share.links.get(link_id).cloned() else {
                continue;
            };
            if expires <= now || Some(&hash) != proof_hash.as_ref() {
                return Err(not_found());
            }
            if !share.members.contains_key(&caller.account_id) {
                if share.requests.len() >= MAX_REQUESTS {
                    return Err(quota("requests"));
                }
                share
                    .requests
                    .insert(caller.account_id.clone(), (link_id.to_owned(), now));
            }
            return Ok(Joined {
                share_id: id.clone(),
                owner_username: owner_names.get(&share.owner).cloned().unwrap_or_default(),
                role,
            });
        }
        Err(not_found())
    }

    fn decline_request(&self, caller: &Caller, share_id: &str, account: &str) -> Result<()> {
        let mut state = self.lock()?;
        let share = state.share(share_id)?;
        if share.owner != caller.account_id || share.requests.remove(account).is_none() {
            return Err(not_found());
        }
        Ok(())
    }

    fn notify(&self, caller: &Caller, notice: &NoticeIn) -> Result<()> {
        let mut state = self.lock()?;
        let role = state.role(&notice.share_id, caller)?;
        if !role.may_comment() {
            return Err(refused(403, "forbidden_role"));
        }
        let share = state.share(&notice.share_id)?;
        if !share.members.contains_key(&notice.recipient) {
            return Err(bad());
        }
        let count = state
            .notices
            .iter()
            .filter(|(r, _)| *r == notice.recipient)
            .count();
        if count >= MAX_NOTICES {
            return Err(refused(429, "rate_limited"));
        }
        state.next_notice += 1;
        let id = format!("n_{:032x}", state.next_notice);
        let at = state.now;
        state.notices.push((
            notice.recipient.clone(),
            NoticeOut {
                id,
                share_id: notice.share_id.clone(),
                from: caller.account_id.clone(),
                sealed: notice.sealed.clone(),
                at,
            },
        ));
        Ok(())
    }

    fn notifications(&self, caller: &Caller) -> Result<Vec<NoticeOut>> {
        let state = self.lock()?;
        Ok(state
            .notices
            .iter()
            .filter(|(r, _)| *r == caller.account_id)
            .map(|(_, n)| n.clone())
            .collect())
    }

    fn delete_notification(&self, caller: &Caller, id: &str) -> Result<()> {
        let mut state = self.lock()?;
        let before = state.notices.len();
        state
            .notices
            .retain(|(r, n)| !(*r == caller.account_id && n.id == id));
        if state.notices.len() == before {
            return Err(not_found());
        }
        Ok(())
    }

    fn live(&self, caller: &Caller, share_id: &str) -> Result<Box<dyn LiveLink>> {
        let mut state = self.lock()?;
        state.role(share_id, caller)?;
        state.next_socket += 1;
        let id = state.next_socket;
        let (tx, rx) = channel();
        let share = state.share(share_id)?;
        let _ = tx.send(LiveEvent::Hello {
            head: share.room.head,
            epoch: share.epoch,
        });
        share.room.sockets.push(Socket {
            id,
            account: caller.account_id.clone(),
            tx,
        });
        Ok(Box::new(MemoryLink {
            server: self.clone(),
            caller: caller.clone(),
            share_id: share_id.to_owned(),
            socket: id,
            rx,
        }))
    }

    fn create_book(&self, caller: &Caller, book: &NewBook) -> Result<BookCreated> {
        self.create_book_impl(caller, book)
    }

    fn book(&self, caller: &Caller, share_id: &str, since_rev: u32) -> Result<BookInfo> {
        self.book_impl(caller, share_id, since_rev)
    }

    fn accept_book(&self, caller: &Caller, share_id: &str) -> Result<()> {
        self.accept_book_impl(caller, share_id)
    }

    fn new_book_list(&self, caller: &Caller, share_id: &str, list: &NewBookList) -> Result<()> {
        self.new_book_list_impl(caller, share_id, list)
    }

    fn book_log(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        since: u64,
        limit: usize,
    ) -> Result<BookLogPage> {
        self.book_log_impl(caller, share_id, stream, since, limit)
    }

    fn book_append(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        envelope: &[u8],
    ) -> Result<u64> {
        self.book_append_impl(caller, share_id, stream, envelope)
    }

    fn book_snapshot(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64> {
        self.book_snapshot_impl(caller, share_id, stream, base_seq, envelope)
    }
}

/// A live connection to a [`MemoryCollab`] room.
struct MemoryLink {
    server: MemoryCollab,
    caller: Caller,
    share_id: String,
    socket: u64,
    rx: Receiver<LiveEvent>,
}

impl MemoryLink {
    fn own_socket(&self, state: &mut State) -> Option<Sender<LiveEvent>> {
        let share = state.shares.get(&self.share_id)?;
        share
            .room
            .sockets
            .iter()
            .find(|s| s.id == self.socket)
            .map(|s| s.tx.clone())
    }
}

impl LiveLink for MemoryLink {
    fn send(&mut self, envelope: &[u8]) -> Result<()> {
        let mut state = self.server.state.lock().expect("poisoned");
        let Some(mine) = self.own_socket(&mut state) else {
            return Err(CollabError::Transport("the room closed".into()));
        };
        let refusal = |code: &str| {
            let _ = mine.send(LiveEvent::Error { code: code.into() });
        };
        match parse_header(envelope) {
            Some((header, _)) if header.kind == Kind::Presence => {
                let member = state.role(&self.share_id, &self.caller).is_ok();
                if !member
                    || header.author_account != self.caller.account_id
                    || envelope.len() > max_size(Kind::Presence)
                {
                    refusal("forbidden_role");
                    return Ok(());
                }
                if let Some(share) = state.shares.get_mut(&self.share_id) {
                    State::broadcast(
                        &mut share.room,
                        &LiveEvent::Presence {
                            envelope: envelope.to_vec(),
                        },
                        Some(self.socket),
                    );
                }
                Ok(())
            }
            Some(_) => {
                if let Err(err) = state.append(&self.share_id, &self.caller, envelope) {
                    refusal(err.code().unwrap_or("bad_request"));
                }
                Ok(())
            }
            None => {
                refusal("bad_request");
                Ok(())
            }
        }
    }

    fn recv(&mut self, wait: Duration) -> Result<Option<LiveEvent>> {
        match self.rx.recv_timeout(wait) {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => {
                Err(CollabError::Transport("the room closed".into()))
            }
        }
    }
}

impl Drop for MemoryLink {
    fn drop(&mut self) {
        if let Ok(mut state) = self.server.state.lock()
            && let Some(share) = state.shares.get_mut(&self.share_id)
        {
            share.room.sockets.retain(|s| s.id != self.socket);
        }
    }
}
