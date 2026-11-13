//! Shared notebooks as the app holds them (issue #390 › Shared studio
//! notebooks): what the last sync said about each notebook joined here, the
//! Share notebook sheet's steps, and the words and choices the sheet, the
//! Shared sidebar section and the note list show. The rules about who may
//! do what are the engine's ([`kanso_collab::engine::book_roster`]); this
//! only asks it. The views are in `book_share_sheet.rs` and
//! `shared_books.rs`.

use kanso_collab::engine::book_roster::may_change_role;
use kanso_collab::{BookMemberView, BookView, MemberStanding, Person, Role};

/// The roles anyone can be given in a notebook, lowest first.
pub const BOOK_ROLES: [Role; 4] = [Role::View, Role::Comment, Role::Edit, Role::Manager];

/// A shared notebook joined on this device, as the last sync left it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookRow {
    pub view: BookView,
    /// Notes with changes this device hasn't sent.
    pub waiting: usize,
}

/// What one background sync of shared notebooks brought.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookNews {
    /// The notebooks joined here; `None` when they couldn't be read (the
    /// ones shown stay).
    pub rows: Option<Vec<BookRow>>,
    /// Invitations as the server has them; `None` offline (the ones shown
    /// stay).
    pub invitations: Option<Vec<kanso_collab::BookSummary>>,
    pub edits: Vec<kanso_collab::NoteEdit>,
    /// What to tell the person about notebooks that ended here.
    pub ended: Vec<String>,
    /// Titles of notes this account deleted that someone's edit brought
    /// back.
    pub brought_back: Vec<String>,
    /// A notebook's sync got no answer from the server.
    pub offline: bool,
}

/// What the Share notebook sheet shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookStep {
    /// Sharing needs an account.
    SignedOut,
    /// The notebook can't be shared: why.
    NotShareable(String),
    /// The people, and the invite field.
    Main,
    /// The first share with someone: their fingerprint to compare.
    Verify { person: Person, role: Role },
    /// Someone's keys changed since they were pinned.
    KeyChanged {
        pinned: Option<Person>,
        offered: Person,
        role: Role,
    },
    /// "Remove Dan from "Studio"?"
    ConfirmRemove { account: String, username: String },
    /// "Leave "Studio"?" or "Stop sharing "Studio"?"
    ConfirmEnd,
}

/// A step the sheet is running, which its busy line names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookBusy {
    Loading,
    LookingUp,
    /// Sharing the notebook for the first time: notes up so far, of all.
    Sharing {
        done: usize,
        total: usize,
    },
    Inviting,
    /// A role going up: the same key, a new wrap.
    Raising,
    /// A role going down or someone removed: the notebook gets a new key.
    Rotating,
    Leaving,
    Stopping,
}

/// What the busy line says while `busy` runs, in the notebook `name`. Pure.
pub fn busy_words(busy: BookBusy, name: &str) -> String {
    match busy {
        BookBusy::Loading => "Reading who is in the notebook…".into(),
        BookBusy::LookingUp => "Looking them up…".into(),
        BookBusy::Sharing { done, total } => format!("Sharing {name}… {done} of {total}"),
        BookBusy::Inviting => "Inviting…".into(),
        BookBusy::Raising => "Changing their role…".into(),
        BookBusy::Rotating => "Changing the notebook's key…".into(),
        BookBusy::Leaving => "Leaving the notebook…".into(),
        BookBusy::Stopping => "Stopping sharing…".into(),
    }
}

/// The roles someone of `mine` can invite people at: every role below
/// Owner for the owner, below Manager for a manager, none otherwise. Pure.
pub fn invite_roles(mine: Option<Role>) -> Vec<Role> {
    match mine {
        // A notebook not shared yet is this account's to share.
        None | Some(Role::Owner) => BOOK_ROLES.to_vec(),
        Some(Role::Manager) => BOOK_ROLES
            .iter()
            .copied()
            .filter(|r| *r < Role::Manager)
            .collect(),
        Some(_) => Vec::new(),
    }
}

/// What this account can do to one member's row: the roles it can give
/// them (their own included, checked) and whether it can remove them. Pure.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemberActions {
    pub roles: Vec<Role>,
    pub remove: bool,
}

impl MemberActions {
    pub fn any(&self) -> bool {
        self.remove || self.roles.len() > 1
    }
}

/// The member menu for `member`, seen by an account of role `mine`. The
/// rank rules are the engine's [`may_change_role`]. Pure.
pub fn member_actions(mine: Option<Role>, member: &BookMemberView) -> MemberActions {
    let Some(mine) = mine.filter(|r| r.may_manage()) else {
        return MemberActions::default();
    };
    if member.me || member.role == Role::Owner {
        return MemberActions::default();
    }
    let roles: Vec<Role> = BOOK_ROLES
        .iter()
        .copied()
        .filter(|to| *to == member.role || may_change_role(mine, member.role, *to))
        .collect();
    // Removing someone is the same rank rule as changing their role.
    let remove = member.role < Role::Manager || mine == Role::Owner;
    MemberActions { roles, remove }
}

/// The words under a member's name. Pure.
pub fn member_detail(member: &BookMemberView) -> Vec<&'static str> {
    let mut out = Vec::new();
    match member.standing {
        MemberStanding::Invited => out.push("Invited, hasn't accepted yet"),
        MemberStanding::Left => out.push("Left or declined"),
        MemberStanding::Member | MemberStanding::Unknown => {}
    }
    if member.me {
        out.push("You");
    }
    out
}

/// The line under a shared notebook: how its sync stands. Pure.
pub fn status_words(offline: bool, syncing: bool, waiting: usize) -> String {
    let changes = |n: usize| {
        if n == 1 {
            "1 change waiting".to_owned()
        } else {
            format!("{n} changes waiting")
        }
    };
    match (offline, syncing, waiting) {
        (true, _, 0) => "Offline".into(),
        (true, _, n) => format!("Offline · {}", changes(n)),
        (false, true, _) => "Syncing…".into(),
        (false, false, 0) => "Up to date".into(),
        (false, false, n) => changes(n),
    }
}

/// How a shared notebook's sync stands, for its glyph. Pure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookTone {
    UpToDate,
    Waiting,
    Offline,
}

pub fn book_tone(offline: bool, waiting: usize) -> BookTone {
    match (offline, waiting) {
        (true, _) => BookTone::Offline,
        (false, 0) => BookTone::UpToDate,
        (false, _) => BookTone::Waiting,
    }
}

/// "Edited by Dan · 2 min ago", from the edit's member-list name and how
/// long ago (as `ago` words it). Pure.
pub fn edited_by(name: &str, ago: &str) -> String {
    format!("Edited by {name} · {ago}")
}

/// The initials in a member's avatar: the first letter of the username,
/// upper-cased. Pure.
pub fn initial(username: &str) -> String {
    username
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".into())
}

/// The avatars a Shared row shows: the members other than this account,
/// at most `most`, and how many more there are. Pure.
pub fn avatars(members: &[BookMemberView], most: usize) -> (Vec<&BookMemberView>, usize) {
    let others: Vec<&BookMemberView> = members.iter().filter(|m| !m.me).collect();
    let more = others.len().saturating_sub(most);
    (others.into_iter().take(most).collect(), more)
}

/// What an invitation in the Shared section says. `status` is the
/// server's: `invited`, or `member` for a notebook another device of this
/// account joined. Pure.
pub fn invitation_words(owner: &str, role: Role, status: &str) -> String {
    if status == "invited" {
        format!("{owner} invited you to a notebook · {}", role.label())
    } else {
        format!("A notebook {owner} shared with you · {}", role.label())
    }
}

/// What the sheet first shows. Pure.
pub fn first_step(signed_in: bool, unshareable: Option<String>) -> BookStep {
    match (signed_in, unshareable) {
        (false, _) => BookStep::SignedOut,
        (true, Some(reason)) => BookStep::NotShareable(reason),
        (true, None) => BookStep::Main,
    }
}

/// The note on locked notes before a notebook is shared, when it has any.
/// Pure.
pub fn locked_words(locked: usize, name: &str) -> Option<String> {
    match locked {
        0 => None,
        1 => Some(format!(
            "Locked notes can't be in a shared notebook. 1 locked note will stay in a private \"{name} (locked)\" notebook."
        )),
        n => Some(format!(
            "Locked notes can't be in a shared notebook. {n} locked notes will stay in a private \"{name} (locked)\" notebook."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn member(role: Role, me: bool, standing: MemberStanding) -> BookMemberView {
        BookMemberView {
            account: "a_2".into(),
            username: "dan".into(),
            role,
            standing,
            me,
        }
    }

    fn any_role() -> impl Strategy<Value = Role> {
        prop::sample::select(vec![
            Role::View,
            Role::Comment,
            Role::Edit,
            Role::Manager,
            Role::Owner,
        ])
    }

    #[test]
    fn status_reads_as_the_design_says() {
        assert_eq!(status_words(true, false, 4), "Offline · 4 changes waiting");
        assert_eq!(status_words(true, false, 1), "Offline · 1 change waiting");
        assert_eq!(status_words(false, false, 0), "Up to date");
        assert_eq!(status_words(false, true, 3), "Syncing…");
        assert_eq!(
            busy_words(
                BookBusy::Sharing {
                    done: 120,
                    total: 340
                },
                "Studio"
            ),
            "Sharing Studio… 120 of 340"
        );
        assert_eq!(edited_by("Dan", "2 min ago"), "Edited by Dan · 2 min ago");
    }

    #[test]
    fn menus_follow_the_rank() {
        let editor = member(Role::Edit, false, MemberStanding::Member);
        let owner_sees = member_actions(Some(Role::Owner), &editor);
        assert_eq!(owner_sees.roles, BOOK_ROLES.to_vec());
        assert!(owner_sees.remove);
        let manager_sees = member_actions(Some(Role::Manager), &editor);
        assert_eq!(
            manager_sees.roles,
            vec![Role::View, Role::Comment, Role::Edit]
        );
        let other_manager = member(Role::Manager, false, MemberStanding::Member);
        assert!(!member_actions(Some(Role::Manager), &other_manager).any());
        assert!(!member_actions(Some(Role::Edit), &editor).any());
        assert_eq!(initial("dan"), "D");
        assert_eq!(initial("_x"), "X");
        assert_eq!(initial(""), "?");
    }

    proptest! {
        /// A member's menu never offers what the chain would refuse: every
        /// role in it but their own passes `may_change_role`, nobody touches
        /// the owner or themselves, and only managers and the owner see one.
        #[test]
        fn prop_menus_offer_only_what_the_chain_takes(
            mine in prop::option::of(any_role()),
            role in any_role(),
            me in any::<bool>(),
        ) {
            let m = member(role, me, MemberStanding::Member);
            let actions = member_actions(mine, &m);
            if me || role == Role::Owner || !mine.is_some_and(|r| r.may_manage()) {
                prop_assert!(!actions.any());
                prop_assert!(actions.roles.is_empty());
            } else {
                let mine = mine.unwrap();
                prop_assert!(actions.roles.contains(&role) || role == Role::Owner);
                for to in &actions.roles {
                    prop_assert!(*to == role || may_change_role(mine, role, *to));
                }
                prop_assert_eq!(actions.remove, role < Role::Manager || mine == Role::Owner);
            }
        }

        /// Invite roles are a prefix of the role list, never Owner, and
        /// Manager only for the owner (or a notebook not shared yet).
        #[test]
        fn prop_invite_roles(mine in prop::option::of(any_role())) {
            let roles = invite_roles(mine);
            prop_assert!(!roles.contains(&Role::Owner));
            prop_assert_eq!(roles.contains(&Role::Manager), matches!(mine, None | Some(Role::Owner)));
            prop_assert_eq!(&BOOK_ROLES[..roles.len()], &roles[..]);
            prop_assert_eq!(roles.is_empty(), mine.is_some_and(|r| !r.may_manage()));
        }

        /// The status always names an offline notebook as offline, counts
        /// what waits with the right plural, and says "Up to date" only
        /// when online with nothing waiting.
        #[test]
        fn prop_status_words(offline: bool, syncing: bool, waiting in 0usize..500) {
            let words = status_words(offline, syncing, waiting);
            prop_assert_eq!(words.starts_with("Offline"), offline);
            prop_assert_eq!(words == "Up to date", !offline && !syncing && waiting == 0);
            if offline && waiting > 0 {
                let plural = if waiting == 1 { "change" } else { "changes" };
                let expected = format!("{waiting} {plural} waiting");
                prop_assert!(words.ends_with(&expected));
            }
            let tone = book_tone(offline, waiting);
            prop_assert_eq!(tone == BookTone::Offline, offline);
            prop_assert_eq!(tone == BookTone::UpToDate, !offline && waiting == 0);
        }

        /// Avatars leave this account out, show at most `most`, and count
        /// the rest: shown + more = everyone else.
        #[test]
        fn prop_avatars(mes in prop::collection::vec(any::<bool>(), 0..30), most in 0usize..6) {
            let members: Vec<BookMemberView> = mes
                .iter()
                .map(|me| member(Role::Edit, *me, MemberStanding::Unknown))
                .collect();
            let (shown, more) = avatars(&members, most);
            let others = mes.iter().filter(|me| !**me).count();
            prop_assert!(shown.len() <= most);
            prop_assert!(shown.iter().all(|m| !m.me));
            prop_assert_eq!(shown.len() + more, others);
        }

        /// The first step: signed out wins, then a reason, then the people.
        #[test]
        fn prop_first_step(signed_in: bool, reason in prop::option::of("[a-z ]{1,20}")) {
            match first_step(signed_in, reason.clone()) {
                BookStep::SignedOut => prop_assert!(!signed_in),
                BookStep::NotShareable(shown) => {
                    prop_assert!(signed_in);
                    prop_assert_eq!(Some(shown), reason);
                }
                BookStep::Main => prop_assert!(signed_in && reason.is_none()),
                _ => prop_assert!(false),
            }
        }

        /// The locked-notes note names how many, in the right number, and
        /// the private notebook they go to; nothing when there are none.
        #[test]
        fn prop_locked_words(locked in 0usize..1000, name in "[A-Za-z]{1,12}") {
            match locked_words(locked, &name) {
                None => prop_assert_eq!(locked, 0),
                Some(words) => {
                    let expected = format!("\"{name} (locked)\"");
                    prop_assert!(words.contains(&expected));
                    let count = format!(" {locked} locked note");
                    prop_assert!(words.contains(&count));
                    prop_assert_eq!(words.contains("notes will"), locked > 1);
                }
            }
        }
    }
}
