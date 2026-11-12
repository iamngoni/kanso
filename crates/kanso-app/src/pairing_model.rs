//! The pairing sheet and Settings › Local API › Paired apps (issue #390,
//! step 13): the values they are made of, and the words they say. No
//! drawing and no state, so each rule can be tested on its own.

use kanso_server::{KeyAccess, PendingPair};
use kanso_store::ApiKeyRow;

use crate::api_keys_model::OTHER_ID;
use crate::grouping::edited_label;
use crate::mcp_setup::Agent;

/// The longest name the sheet shows. Pairing takes up to 60 characters; a
/// longer one is cut so it can't push the code off the sheet.
const NAME_SHOWN: usize = 40;

/// What the key the sheet makes can do.
pub const CAN: &str =
    "It will be able to: search and read notes \u{b7} add notes and tasks \u{b7} tick tasks.";

/// What it can't.
pub const CANNOT: &str = "It will not be able to change or delete existing text.";

/// Under the notebooks.
pub const LOCKED: &str = "Locked notes are never shown to a key.";

/// Said when a second app asked while this request waited: one of them
/// may not be what the person just started.
pub const CONTESTED: &str =
    "Two apps asked for a key. Deny unless you just started this from Raycast.";

/// The title of the Paired apps group.
pub const GROUP_TITLE: &str = "Paired apps";

/// What a revoke does, said before it is asked for.
pub const DISCONNECT_DETAIL: &str =
    "It stops working right away. Run a Kanso command in Raycast to connect again.";

/// The name as the sheet and list show it: as given, trimmed and cut.
pub fn shown_name(name: &str) -> String {
    let name = name.trim();
    if name.chars().count() <= NAME_SHOWN {
        return name.to_owned();
    }
    let cut: String = name.chars().take(NAME_SHOWN - 1).collect();
    format!("{}\u{2026}", cut.trim_end())
}

/// Whether Deny, not Allow, is the sheet's main button: when more than one
/// app asked, a person who did nothing must not allow by habit.
pub fn deny_is_default(pending: &PendingPair) -> bool {
    pending.contested
}

/// The sheet's title: "Raycast wants to connect".
pub fn title(pending: &PendingPair) -> String {
    format!("{} wants to connect", shown_name(&pending.name))
}

/// "An app on this Mac calling itself “Raycast” asks for a key."
pub fn intro(pending: &PendingPair) -> String {
    format!(
        "An app on this Mac calling itself \u{201c}{}\u{201d} asks for a key.",
        shown_name(&pending.name)
    )
}

/// The code, in two groups to compare: "482 913".
pub fn code_label(code: &str) -> String {
    match code.len() {
        6 if code.is_ascii() => format!("{} {}", &code[..3], &code[3..]),
        _ => code.to_owned(),
    }
}

/// "Check that Raycast shows the same code."
pub fn compare(pending: &PendingPair) -> String {
    format!(
        "Check that {} shows the same code.",
        shown_name(&pending.name)
    )
}

/// The notebooks it may read, in words, given the name of each by id.
pub fn notebooks_line(pending: &PendingPair, name_of: impl Fn(&str) -> Option<String>) -> String {
    format!(
        "Notebooks: {}",
        crate::api_keys_model::books_summary(pending.books.as_deref(), name_of)
    )
}

/// Whether a key was made by pairing: it can only add and tick, and its
/// client is one only a pairing request names. Settings' New key sheet
/// offers each agent, and "other", so those are never paired.
pub fn is_paired(row: &ApiKeyRow) -> bool {
    row.access == KeyAccess::Capture
        && row.client != OTHER_ID
        && Agent::from_id(&row.client).is_none()
}

/// "Added 12 Nov" or "Added 3 h ago".
pub fn added_label(created: i64, now: i64, local: impl Fn(i64) -> i64) -> String {
    format!(
        "Added {}",
        edited_label(created, now, local(created), local(now))
    )
}

/// "Last used 2 min ago", or "Never used".
pub fn last_used_label(last_used: Option<i64>, now: i64, local: impl Fn(i64) -> i64) -> String {
    match last_used {
        None => "Never used".to_owned(),
        Some(at) => format!("Last used {}", edited_label(at, now, local(at), local(now))),
    }
}

/// The row's line: "Raycast · Added 12 Nov · Last used 2 min ago".
pub fn row_line(row: &ApiKeyRow, now: i64, local: impl Fn(i64) -> i64 + Copy) -> String {
    format!(
        "{} \u{b7} {} \u{b7} {}",
        shown_name(&row.name),
        added_label(row.created_at, now, local),
        last_used_label(row.last_used_at, now, local),
    )
}

/// The question before a revoke: "Disconnect Raycast?"
pub fn disconnect_question(name: &str) -> String {
    format!("Disconnect {}?", shown_name(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn pending(name: &str, books: Option<Vec<String>>) -> PendingPair {
        PendingPair {
            request: "r".into(),
            name: name.into(),
            client: "raycast".into(),
            code: "482913".into(),
            books,
            expires_in: 120,
            contested: false,
        }
    }

    fn row(client: &str, access: KeyAccess) -> ApiKeyRow {
        ApiKeyRow {
            id: "i".into(),
            name: "Raycast".into(),
            client: client.into(),
            access,
            books: None,
            created_at: 0,
            last_used_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn the_sheet_says_what_the_design_says() {
        let p = pending("Raycast", None);
        assert_eq!(title(&p), "Raycast wants to connect");
        assert_eq!(
            intro(&p),
            "An app on this Mac calling itself \u{201c}Raycast\u{201d} asks for a key."
        );
        assert_eq!(code_label(&p.code), "482 913");
        assert_eq!(compare(&p), "Check that Raycast shows the same code.");
        assert_eq!(notebooks_line(&p, |_| None), "Notebooks: All notebooks");
    }

    #[test]
    fn a_contested_request_makes_deny_the_default() {
        let mut p = pending("Raycast", None);
        assert!(!deny_is_default(&p));
        p.contested = true;
        assert!(deny_is_default(&p));
    }

    #[test]
    fn only_a_pairing_client_with_capture_access_is_paired() {
        assert!(is_paired(&row("raycast", KeyAccess::Capture)));
        assert!(!is_paired(&row("raycast", KeyAccess::Write)));
        assert!(!is_paired(&row("other", KeyAccess::Capture)));
        assert!(!is_paired(&row("codex", KeyAccess::Capture)));
    }

    #[test]
    fn the_row_reads_name_added_and_last_used() {
        let utc = |ms| ms;
        let now = 1_779_954_300_000;
        let mut r = row("raycast", KeyAccess::Capture);
        r.created_at = now - 3 * 3_600_000;
        assert_eq!(
            row_line(&r, now, utc),
            "Raycast \u{b7} Added 3 h ago \u{b7} Never used"
        );
        r.last_used_at = Some(now - 120_000);
        assert_eq!(
            row_line(&r, now, utc),
            "Raycast \u{b7} Added 3 h ago \u{b7} Last used 2 min ago"
        );
        assert_eq!(disconnect_question("Raycast"), "Disconnect Raycast?");
    }

    proptest! {
        /// A name, however long or strange, is never shown longer than the
        /// limit, and a short one is shown as it is.
        #[test]
        fn prop_a_name_is_cut_to_the_limit(name in "\\PC{0,200}") {
            let shown = shown_name(&name);
            prop_assert!(shown.chars().count() <= NAME_SHOWN);
            if name.trim().chars().count() <= NAME_SHOWN {
                prop_assert_eq!(shown, name.trim());
            }
        }

        /// Six digits split into two groups of three, and nothing else is
        /// changed.
        #[test]
        fn prop_the_code_keeps_its_digits(code in "[0-9]{6}") {
            let label = code_label(&code);
            prop_assert_eq!(label.replace(' ', ""), code);
            prop_assert_eq!(label.len(), 7);
        }
    }
}
