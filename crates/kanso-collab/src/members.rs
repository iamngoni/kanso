//! The owner-signed member list (docs/ACCOUNTS.md › Collaboration › The
//! member list): who is in a shared note for one epoch, with their roles and
//! keys, and the commitment to that epoch's note key. Clients check every
//! list they are given, and drop writes from anyone a list doesn't allow.

use crate::codec::{FORMAT_V1, b64, unb64, unb64_array};
use crate::identity::{Identity, verify};
use crate::model::{Member, MemberList, Role};

/// At most this many people in a note, its owner included.
pub const MAX_MEMBERS: usize = 20;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListError {
    #[error("the member list isn't well-formed")]
    Malformed,
    #[error("the member list isn't signed by the note's owner")]
    BadSignature,
    #[error("the member list is for another note")]
    WrongShare,
    #[error("the member list is older than one already seen")]
    Stale,
}

fn signed_message(list_bytes: &[u8]) -> Vec<u8> {
    let mut message = b"kanso-members".to_vec();
    message.push(FORMAT_V1);
    message.extend_from_slice(list_bytes);
    message
}

/// A list's shape rules: members sorted by account and unique, exactly one
/// owner who is the list's `owner`, at most [`MAX_MEMBERS`], well-formed keys.
pub fn is_well_formed(list: &MemberList) -> bool {
    let sorted = list
        .members
        .windows(2)
        .all(|pair| pair[0].account < pair[1].account);
    let owners: Vec<&Member> = list
        .members
        .iter()
        .filter(|m| m.role == Role::Owner)
        .collect();
    list.v == 1
        && list.epoch >= 1
        && (list.epoch == 1) == list.from_seq.is_none()
        && sorted
        && !list.members.is_empty()
        && list.members.len() <= MAX_MEMBERS
        && owners.len() == 1
        && owners[0].account == list.owner
        // Managers exist in shared notebooks only (crate::book_members).
        && list.members.iter().all(|m| m.role != Role::Manager)
        && list.key_commit.len() == 64
        && list.members.iter().all(|m| {
            unb64_array::<32>(&m.sign_pub).is_some() && unb64_array::<32>(&m.box_pub).is_some()
        })
}

/// The list's bytes and the owner's signature, as base64url.
pub fn sign(owner: &Identity, list: &MemberList) -> (String, String) {
    let bytes = serde_json::to_vec(list).expect("a member list serializes");
    let signature = owner.sign(&signed_message(&bytes));
    (b64(&bytes), b64(&signature))
}

/// Opens a list received as base64url bytes and signature: well-formed, for
/// `share_id`, signed by `owner_sign_pub`, and no older than `min_epoch`.
pub fn open(
    list: &str,
    list_sig: &str,
    share_id: &str,
    owner_sign_pub: &[u8; 32],
    min_epoch: u32,
) -> Result<MemberList, ListError> {
    let bytes = unb64(list).ok_or(ListError::Malformed)?;
    let signature = unb64_array::<64>(list_sig).ok_or(ListError::Malformed)?;
    if !verify(owner_sign_pub, &signed_message(&bytes), &signature) {
        return Err(ListError::BadSignature);
    }
    let parsed: MemberList = serde_json::from_slice(&bytes).map_err(|_| ListError::Malformed)?;
    if !is_well_formed(&parsed) {
        return Err(ListError::Malformed);
    }
    if parsed.share != share_id {
        return Err(ListError::WrongShare);
    }
    // The owner's own entry must carry the key the list is signed with.
    let owner = parsed
        .members
        .iter()
        .find(|m| m.role == Role::Owner)
        .ok_or(ListError::Malformed)?;
    if unb64_array::<32>(&owner.sign_pub) != Some(*owner_sign_pub) {
        return Err(ListError::BadSignature);
    }
    if parsed.epoch < min_epoch {
        return Err(ListError::Stale);
    }
    Ok(parsed)
}

impl MemberList {
    pub fn member(&self, account: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.account == account)
    }

    pub fn role_of(&self, account: &str) -> Option<Role> {
        self.member(account).map(|m| m.role)
    }

    /// Sorts members by account, as lists must be.
    pub fn sort(&mut self) {
        self.members.sort_by(|a, b| a.account.cmp(&b.account));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::NoteKey;
    use proptest::prelude::*;

    pub(crate) fn member(identity: &Identity, account: &str, role: Role) -> Member {
        Member {
            account: account.into(),
            username: format!("u{account}"),
            role,
            sign_pub: b64(&identity.sign_pub()),
            box_pub: b64(&identity.box_pub()),
        }
    }

    fn list_of(owner: &Identity, others: &[(String, Role)], epoch: u32) -> MemberList {
        let mut members = vec![member(owner, "a_0", Role::Owner)];
        for (account, role) in others {
            members.push(member(&Identity::generate(), account, *role));
        }
        let mut list = MemberList {
            v: 1,
            share: "s".into(),
            epoch,
            owner: "a_0".into(),
            key_commit: NoteKey::generate().commit("s", epoch),
            at: 1,
            from_seq: (epoch > 1).then_some(0),
            kind: None,
            members,
        };
        list.sort();
        list
    }

    fn others() -> impl Strategy<Value = Vec<(String, Role)>> {
        prop::collection::btree_map(
            "a_[1-9][0-9]{0,2}",
            prop::sample::select(vec![Role::View, Role::Comment, Role::Edit]),
            0..6,
        )
        .prop_map(|m| m.into_iter().collect())
    }

    proptest! {
        #![proptest_config(crate::tests::config(32))]

        /// Round trip: a list the owner signed opens, with the same members.
        #[test]
        fn prop_signed_lists_open(others in others(), epoch in 1u32..20) {
            let owner = Identity::generate();
            let list = list_of(&owner, &others, epoch);
            let (bytes, sig) = sign(&owner, &list);
            prop_assert_eq!(open(&bytes, &sig, "s", &owner.sign_pub(), epoch), Ok(list.clone()));
            prop_assert_eq!(open(&bytes, &sig, "s", &owner.sign_pub(), epoch + 1), Err(ListError::Stale));
            prop_assert_eq!(open(&bytes, &sig, "t", &owner.sign_pub(), 0), Err(ListError::WrongShare));
        }

        /// Nobody but the owner can make a list members accept: a list signed
        /// by anyone else, or changed after signing, is refused.
        #[test]
        fn prop_forged_lists_are_refused(others in others(), byte in any::<prop::sample::Index>()) {
            let owner = Identity::generate();
            let forger = Identity::generate();
            let list = list_of(&owner, &others, 1);
            let (bytes, _) = sign(&owner, &list);
            let (_, forged_sig) = sign(&forger, &list);
            prop_assert!(open(&bytes, &forged_sig, "s", &owner.sign_pub(), 0).is_err());
            let (bytes2, sig2) = sign(&owner, &list);
            let mut raw = unb64(&bytes2).unwrap();
            let i = byte.index(raw.len());
            raw[i] ^= 0x20;
            prop_assert!(open(&b64(&raw), &sig2, "s", &owner.sign_pub(), 0).is_err());
            // A list the forger signs naming themself owner fails against the pinned owner key.
            let mut stolen = list.clone();
            stolen.members.retain(|m| m.role != Role::Owner);
            stolen.members.push(member(&forger, "a_0", Role::Owner));
            stolen.sort();
            let (sb, ss) = sign(&forger, &stolen);
            prop_assert!(open(&sb, &ss, "s", &owner.sign_pub(), 0).is_err());
        }
    }

    /// Managers exist in shared notebooks only: a note's list that names one
    /// is malformed.
    #[test]
    fn note_lists_have_no_managers() {
        let owner = Identity::generate();
        let mut list = list_of(&owner, &[("a_1".into(), Role::Edit)], 1);
        assert!(is_well_formed(&list));
        list.members[1].role = Role::Manager;
        assert!(!is_well_formed(&list));
    }
}
