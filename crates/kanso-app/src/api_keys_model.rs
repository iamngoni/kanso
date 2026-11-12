//! Settings › Local API › API keys (issue #243, step 7): the values the
//! page and its sheets are made of, and the words they say. No drawing and
//! no state here, so each rule can be tested on its own.

use std::collections::BTreeSet;

use kanso_server::{KeyAccess, NewKey};

use crate::grouping::edited_label;
use crate::mcp_setup::Agent;

/// How long a refused request stays on the page.
pub const REFUSED_SHOWN_FOR_MS: i64 = 24 * 60 * 60 * 1000;

/// The title of the warning Settings shows when the local API listens on an
/// address other devices can reach.
pub const LAN_WARNING_TITLE: &str = "Keys travel unencrypted";

/// The warning's words, from the design: binding to a LAN address sends keys
/// unencrypted. They say how to fix it, too.
pub const LAN_WARNING: &str = "Binding to a LAN address sends keys unencrypted, so anyone on the network can read them. Set the address to 127.0.0.1 to keep the API on this device.";

/// Whether `bind_address` lets other devices reach the API: anything that
/// isn't a loopback address. An address that doesn't parse counts as one
/// that does, so a mistake errs towards the warning.
pub fn is_lan_address(bind_address: &str) -> bool {
    !matches!(bind_address.trim().parse::<std::net::IpAddr>(), Ok(ip) if ip.is_loopback())
}

/// What a key is for: the "For" choice of the New key sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Client {
    Agent(Agent),
    /// A script or another app, which reaches the REST API.
    Other,
}

/// The `client` a key made for a script or another app is saved with.
pub const OTHER_ID: &str = "other";

impl Client {
    /// Every choice, in the order the sheet lists them.
    pub fn all() -> impl Iterator<Item = Client> {
        Agent::ALL
            .into_iter()
            .map(Client::Agent)
            .chain([Client::Other])
    }

    pub fn name(self) -> &'static str {
        match self {
            Client::Agent(agent) => agent.name(),
            Client::Other => "Script or other app",
        }
    }

    /// What is saved as the key's `client`.
    pub fn id(self) -> &'static str {
        match self {
            Client::Agent(agent) => agent.id(),
            Client::Other => OTHER_ID,
        }
    }

    /// The choice a saved `client` names. A stranger reads as a script or
    /// another app, which is what it is: something Kanso has no setup for.
    pub fn from_id(id: &str) -> Client {
        Agent::from_id(id).map_or(Client::Other, Client::Agent)
    }

    /// The name the sheet starts with: "Codex on this Mac".
    pub fn default_name(self) -> String {
        match self {
            Client::Agent(agent) => format!("{} on this Mac", agent.name()),
            Client::Other => "Script on this Mac".to_owned(),
        }
    }

    /// The toast after Copy.
    pub fn copied_message(self) -> String {
        match self {
            Client::Agent(agent) => format!("Setup for {} copied.", agent.name()),
            Client::Other => "Setup copied.".to_owned(),
        }
    }
}

/// What the access choice says, with the line under it when it has one.
pub fn access_choice(access: KeyAccess) -> (&'static str, Option<&'static str>) {
    match access {
        KeyAccess::Read => ("Read notes", None),
        KeyAccess::Suggest => (
            "Read and suggest edits",
            Some("You review each edit before it's saved."),
        ),
        KeyAccess::Write => ("Read and write", Some("Changes are saved right away.")),
        KeyAccess::Capture => (
            "Add and tick only",
            Some(
                "Can read, add notes and tasks, and tick tasks. It can't change or delete anything.",
            ),
        ),
    }
}

/// The access as the list says it.
pub fn access_short(access: KeyAccess) -> &'static str {
    match access {
        KeyAccess::Read => "Read notes",
        KeyAccess::Suggest => "Suggest edits",
        KeyAccess::Write => "Read and write",
        KeyAccess::Capture => "Add and tick only",
    }
}

/// The notebooks a key may read, in words: "All notebooks", or their names.
/// `name_of` gives a notebook's name, or `None` when it is gone. A key
/// limited to notebooks that are all gone reads nothing, and says so.
pub fn books_summary(books: Option<&[String]>, name_of: impl Fn(&str) -> Option<String>) -> String {
    let Some(books) = books else {
        return "All notebooks".to_owned();
    };
    let names: Vec<String> = books.iter().filter_map(|id| name_of(id)).collect();
    match names.len() {
        0 => "No notebooks".to_owned(),
        1..=3 => names.join(", "),
        n => format!("{}, {} and {} more", names[0], names[1], n - 2),
    }
}

/// "Used 2 min ago" or "Never used". The times are as [`edited_label`]
/// takes them.
pub fn used_label(last_used: Option<i64>, now: i64, local: impl Fn(i64) -> i64) -> String {
    match last_used {
        None => "Never used".to_owned(),
        Some(at) => format!("Used {}", edited_label(at, now, local(at), local(now))),
    }
}

/// "Created May 25".
pub fn created_label(created: i64, now: i64, local: impl Fn(i64) -> i64) -> String {
    format!(
        "Created {}",
        edited_label(created, now, local(created), local(now))
    )
}

/// The time of day of a local time in ms, as "14:05".
pub fn clock_time(local_ms: i64) -> String {
    let minutes = local_ms.div_euclid(60_000).rem_euclid(24 * 60);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// The notice for a refused request, while it is under a day old. It says
/// nothing about the program that asked, because that could be faked.
pub fn refused_notice(
    last_refused: Option<i64>,
    now: i64,
    local: impl Fn(i64) -> i64,
) -> Option<String> {
    let at = last_refused?;
    let age = now - at;
    // A clock that moved back reads as recent, not as never.
    (age < REFUSED_SHOWN_FOR_MS).then(|| {
        format!(
            "Something tried to reach Kanso without a valid key at {}.",
            clock_time(local(at))
        )
    })
}

/// What the New key sheet asks to make a key. `None` while it can't be made:
/// no name, or limited to notebooks but none chosen.
pub fn new_key(
    client: Client,
    name: &str,
    access: KeyAccess,
    all_books: bool,
    ticked: &BTreeSet<String>,
) -> Option<NewKey> {
    let name = name.trim();
    if name.is_empty() || (!all_books && ticked.is_empty()) {
        return None;
    }
    Some(NewKey {
        name: name.to_owned(),
        client: client.id().to_owned(),
        access,
        books: (!all_books).then(|| ticked.iter().cloned().collect()),
    })
}

/// The question before a revoke.
pub fn revoke_question(name: &str) -> String {
    format!("Revoke the key for “{name}”?")
}

/// What a revoke does, said before it is asked for.
pub const REVOKE_DETAIL: &str = "It stops working right away.";

/// The group's own line.
pub const GROUP_DETAIL: &str = "Every agent, script and app that reaches Kanso needs its own key. Kanso refuses anything without one, even a program that can start Kanso.";

/// Under the key, once it is made.
pub const SHOWN_ONCE: &str = "Kanso shows this key only once. Copy it now.";

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn names(id: &str) -> Option<String> {
        match id {
            "w" => Some("Work".into()),
            "i" => Some("Ideas".into()),
            "p" => Some("Personal".into()),
            "j" => Some("Journal".into()),
            _ => None,
        }
    }

    #[test]
    fn default_names_follow_the_choice() {
        assert_eq!(
            Client::Agent(Agent::Codex).default_name(),
            "Codex on this Mac"
        );
        assert_eq!(Client::Other.default_name(), "Script on this Mac");
    }

    #[test]
    fn books_in_words() {
        let ids = |ids: &[&str]| ids.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(books_summary(None, names), "All notebooks");
        assert_eq!(books_summary(Some(&ids(&["w", "i"])), names), "Work, Ideas");
        assert_eq!(
            books_summary(Some(&ids(&["w", "i", "p", "j"])), names),
            "Work, Ideas and 2 more"
        );
        assert_eq!(books_summary(Some(&ids(&["gone"])), names), "No notebooks");
        assert_eq!(books_summary(Some(&[]), names), "No notebooks");
    }

    #[test]
    fn use_and_creation_in_words() {
        let utc = |ms| ms;
        let now = 1_779_954_300_000;
        assert_eq!(used_label(None, now, utc), "Never used");
        assert_eq!(used_label(Some(now - 120_000), now, utc), "Used 2 min ago");
        assert_eq!(
            created_label(now - 3 * 3_600_000, now, utc),
            "Created 3 h ago"
        );
    }

    #[test]
    fn the_clock_reads_as_hours_and_minutes() {
        assert_eq!(clock_time(0), "00:00");
        assert_eq!(clock_time((14 * 60 + 5) * 60_000), "14:05");
        assert_eq!(clock_time(-60_000), "23:59");
    }

    #[test]
    fn the_refused_notice_names_the_time_for_a_day() {
        let day = 24 * 60 * 60 * 1000;
        let at = (14 * 60 + 5) * 60_000 + 7 * day;
        let utc = |ms| ms;
        assert_eq!(refused_notice(None, at, utc), None);
        assert_eq!(
            refused_notice(Some(at), at + 60_000, utc).as_deref(),
            Some("Something tried to reach Kanso without a valid key at 14:05.")
        );
        assert_eq!(refused_notice(Some(at), at + day, utc), None);
    }

    #[test]
    fn a_key_needs_a_name_and_notebooks_when_limited() {
        let none = BTreeSet::new();
        let one: BTreeSet<String> = ["w".to_owned()].into();
        let codex = Client::Agent(Agent::Codex);
        assert!(new_key(codex, "  ", KeyAccess::Read, true, &none).is_none());
        assert!(new_key(codex, "Codex", KeyAccess::Read, false, &none).is_none());
        let key = new_key(codex, " Codex ", KeyAccess::Write, false, &one).unwrap();
        assert_eq!(key.name, "Codex");
        assert_eq!(key.client, "codex");
        assert_eq!(key.books, Some(vec!["w".to_owned()]));
        assert_eq!(
            new_key(codex, "Codex", KeyAccess::Read, true, &one)
                .unwrap()
                .books,
            None
        );
    }

    #[test]
    fn lan_address_examples() {
        for lan in [
            "0.0.0.0",
            "192.168.1.5",
            "10.0.0.2",
            "::",
            "fe80::1",
            "nonsense",
            "localhost",
            "my-mac.local",
            "",
        ] {
            assert!(is_lan_address(lan), "{lan}");
        }
        for local in ["127.0.0.1", "127.9.9.9", "::1", " 127.0.0.1 "] {
            assert!(!is_lan_address(local), "{local}");
        }
    }

    proptest! {
        /// Every 127.x.y.z address is quiet and every other IPv4 address
        /// warns.
        #[test]
        fn prop_only_loopback_is_quiet(a in any::<u8>(), b in any::<u8>(), c in any::<u8>(), d in any::<u8>()) {
            let addr = format!("{a}.{b}.{c}.{d}");
            prop_assert_eq!(is_lan_address(&addr), a != 127);
        }

        /// Whatever the time, the notice shows only inside the day after the
        /// refusal, and says the minute it happened.
        #[test]
        fn prop_the_notice_lasts_a_day(
            at in 0i64..4_000_000_000_000,
            age in 0i64..(3 * REFUSED_SHOWN_FOR_MS),
            offset in -12i64 * 3600..14 * 3600,
        ) {
            let local = move |ms: i64| ms + offset * 1000;
            let notice = refused_notice(Some(at), at + age, local);
            prop_assert_eq!(notice.is_some(), age < REFUSED_SHOWN_FOR_MS);
            if let Some(text) = notice {
                prop_assert!(text.contains(&clock_time(local(at))));
            }
        }

        /// A limited key never reads as every notebook, and a blank name
        /// never makes a key.
        #[test]
        fn prop_only_all_says_all(ids in prop::collection::vec("[a-z]{1,4}", 0..6), name in ".{0,12}") {
            prop_assert_ne!(books_summary(Some(&ids), |_| Some("x".into())), "All notebooks");
            let ticked: BTreeSet<String> = ids.iter().cloned().collect();
            let made = new_key(Client::Other, &name, KeyAccess::Read, false, &ticked);
            prop_assert_eq!(made.is_some(), !name.trim().is_empty() && !ticked.is_empty());
        }

        /// A saved client reads back as the choice it was made from.
        #[test]
        fn prop_clients_round_trip(ix in 0usize..10) {
            let client = Client::all().nth(ix).unwrap();
            prop_assert_eq!(Client::from_id(client.id()), client);
        }
    }
}
