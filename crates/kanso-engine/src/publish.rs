//! Publishing a note or a notebook at a link anyone can open, sealed on this
//! device so the sync server keeps only a copy it can't read
//! (docs/ACCOUNTS.md › Published notes). The desktop's Publish sheet and the
//! mobile app's call the same code: what the sheet shows for an account and
//! a note, the wording of what went wrong, and the publish itself.
//!
//! The page is the note's web page export ([`crate::export::page`]), images
//! inside it, in the preview's theme when that is light and in Kanso's light
//! theme otherwise. `kanso_sync::share` seals and uploads it; the link and
//! its key are kept in the note (`kansoPublish`), so the account's other
//! devices can update or unpublish it. [`site`] does the same for a
//! notebook.

use chrono::{DateTime, Datelike as _, Timelike as _};
use kanso_core::DocType;
use kanso_render::Palette;
use kanso_store::{Origin, Put, Store};
use kanso_sync::share::{self, Expiry, KansoPublish, Page, ShareApi, ShareError};
use kanso_sync::{ApiError, Session};

use crate::export::{PageStyle, page_without_queries};

mod model;
pub mod site;

pub use model::{Busy, Phase, Source};

/// Why a note can't be published, as the notice says: `locked` when the note
/// is locked or in a locked notebook, `in_locked_notebook` when it is the
/// notebook's lock. Their text would go to the web in the clear, so they
/// stay where they are.
pub fn refusal(locked: bool, in_locked_notebook: bool) -> Option<&'static str> {
    match (locked, in_locked_notebook) {
        (false, false) => None,
        (_, true) => Some(
            "Notes in a locked notebook can't be published. Move the note out and remove its lock first.",
        ),
        (true, false) => Some("Locked notes can't be published. Remove the lock first."),
    }
}

/// What the sheet shows for an account state and a note.
pub fn phase(signed_in: bool, publish: Option<&KansoPublish>) -> Phase {
    match (signed_in, publish) {
        (false, _) => Phase::SignedOut,
        (true, None) => Phase::Unpublished,
        (true, Some(_)) => Phase::Published,
    }
}

/// The expiry choice closest to how long a link was given.
pub fn expiry_choice(publish: &KansoPublish) -> Expiry {
    let Some(at) = publish.expires_at else {
        return Expiry::Never;
    };
    let given = at - publish.published_at;
    Expiry::ALL[1..]
        .iter()
        .copied()
        .min_by_key(|e| (e.at(0).unwrap_or(0) - given).abs())
        .unwrap_or(Expiry::Never)
}

/// Whether "Update published version" has something to do: the note
/// changed, another expiry was picked, or the link expired.
pub fn needs_update(
    publish: &KansoPublish,
    title: &str,
    body: &str,
    expiry: Expiry,
    now_ms: i64,
) -> bool {
    publish.is_outdated(title, body)
        || expiry != expiry_choice(publish)
        || publish.is_expired(now_ms)
}

/// "28 May at 09:41", from a local time in ms.
pub fn when_label(local_ms: i64) -> String {
    DateTime::from_timestamp_millis(local_ms).map_or_else(String::new, |at| {
        format!(
            "{} {} at {:02}:{:02}",
            at.day(),
            month(at.month()),
            at.hour(),
            at.minute()
        )
    })
}

fn month(m: u32) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][(m as usize).clamp(1, 12) - 1]
}

/// What an error means to the person publishing.
pub fn error_text(err: &ShareError) -> String {
    match err {
        ShareError::TooLarge(_) => {
            "This note is too large to publish: with its images it is over 10 MB.".into()
        }
        ShareError::Undecryptable => "Kanso couldn't seal the note. Try again.".into(),
        ShareError::Api(api) => match api {
            ApiError::Transport(_) => {
                "Couldn't reach your sync server. Check your connection and try again.".into()
            }
            ApiError::Unauthorized { .. } | ApiError::AccountDisabled => {
                "Your account signed this device out. Sign in again to publish.".into()
            }
            ApiError::QuotaExceeded { .. } => {
                "Your account has no room left for this. Unpublish another note, or remove some attachments.".into()
            }
            ApiError::RateLimited { .. } => {
                "Your sync server is busy. Try again in a minute.".into()
            }
            ApiError::NotFound | ApiError::MethodNotAllowed => {
                "Your sync server doesn't publish notes yet.".into()
            }
            other => format!("Couldn't publish the note ({other})."),
        },
    }
}

/// Publishes `source` as the page it makes in `palette` and `style`, or
/// updates its link in place when it has one (`source.publish`): the page is
/// sealed and uploaded with `session`, expiring at `expires_at` (see
/// [`Expiry::at`]). Returns what the note should now keep
/// ([`store_publish`]). Blocks while math and diagrams are drawn and the
/// page uploads.
// A page needs its palette, style, expiry and clock; all are callers' choices.
#[allow(clippy::too_many_arguments)]
pub fn publish_note(
    api: &dyn ShareApi,
    session: &Session,
    store: &Store,
    source: &Source,
    palette: &Palette,
    style: &PageStyle,
    expires_at: Option<i64>,
    now_ms: i64,
) -> Result<KansoPublish, ShareError> {
    let html = page_without_queries(store, &source.title, &source.body, palette, style);
    let page = Page {
        html: html.as_bytes(),
        title: &source.title,
        body: &source.body,
        rev: &source.rev,
    };
    share::publish(
        api,
        session,
        source.publish.as_ref(),
        &page,
        expires_at,
        now_ms,
    )
}

/// Keeps `publish` in the note `id` (or, with `None`, takes it out).
pub fn store_publish(
    store: &Store,
    id: &str,
    publish: Option<&KansoPublish>,
) -> anyhow::Result<()> {
    let Some(mut note) = store.note(id)? else {
        return Ok(());
    };
    KansoPublish::store(publish, &mut note.extra);
    store.put(
        DocType::Note,
        serde_json::to_value(&note)?,
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ApiError, Expiry, KansoPublish, Phase, ShareError, Source, error_text, expiry_choice,
        needs_update, phase, publish_note, refusal, share, store_publish, when_label,
    };
    use kanso_core::DocType;
    use kanso_render::Palette;
    use kanso_store::{Origin, Put, Store};
    use kanso_sync::share::{MemoryShares, PUBLISH_KEY, open_share};
    use kanso_sync::{DeviceIdentity, MemorySecrets, MemoryServer, Password};
    use proptest::prelude::*;
    use serde_json::{Map, json};

    fn published(at: i64, expires: Option<i64>, title: &str, body: &str) -> KansoPublish {
        KansoPublish {
            share_id: share::new_share_id(),
            key: share::ShareKey::generate().to_text(),
            published_at: at,
            published_rev: "1-a".into(),
            expires_at: expires,
            content_hash: Some(share::content_hash(title, body)),
            short_url: None,
            other: Map::new(),
        }
    }

    #[test]
    fn examples() {
        let p = published(0, None, "T", "B");
        assert_eq!(phase(false, Some(&p)), Phase::SignedOut);
        assert_eq!(phase(true, None), Phase::Unpublished);
        assert_eq!(phase(true, Some(&p)), Phase::Published);
        assert!(!needs_update(&p, "T", "B", Expiry::Never, 5));
        assert!(needs_update(&p, "T", "B!", Expiry::Never, 5));
        assert!(needs_update(&p, "T", "B", Expiry::Week, 5));
        let week = published(0, Expiry::Week.at(0), "T", "B");
        assert_eq!(expiry_choice(&week), Expiry::Week);
        assert!(needs_update(
            &week,
            "T",
            "B",
            Expiry::Week,
            Expiry::Week.at(0).unwrap()
        ));
        assert_eq!(when_label(1_779_954_300_000 + 7_200_000), "28 May at 09:45");
        assert!(
            error_text(&ShareError::Api(ApiError::Transport("x".into()))).contains("connection")
        );
        assert!(error_text(&ShareError::TooLarge(1)).contains("10 MB"));
    }

    #[test]
    fn locked_notes_are_refused_in_the_desktops_words() {
        assert_eq!(refusal(false, false), None);
        assert_eq!(
            refusal(true, false),
            Some("Locked notes can't be published. Remove the lock first.")
        );
        assert_eq!(
            refusal(true, true),
            Some(
                "Notes in a locked notebook can't be published. Move the note out and remove its lock first."
            )
        );
        assert_eq!(refusal(false, true), refusal(true, true));
    }

    fn signed_in() -> kanso_sync::Session {
        let server = MemoryServer::new();
        let username = "alice";
        kanso_sync::sign_up(
            &server,
            &MemorySecrets::new(),
            kanso_sync::DEFAULT_SERVER,
            username,
            &Password::new("correct horse battery", username).unwrap(),
            &DeviceIdentity::from_parts(&format!("l_{:032x}", 1), "Mac", [1; 32]),
            &|_| {},
        )
        .unwrap()
        .session
    }

    // no-prop-test: an end-to-end publish against the in-memory server; the
    // properties are the pure functions' here and kanso-sync's share tests.
    #[test]
    fn a_note_publishes_updates_and_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "Plan", "body": "The secret plan.", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let shares = MemoryShares::new();
        let session = signed_in();
        let mut source = Source {
            id: id.clone(),
            title: "Plan".into(),
            body: "The secret plan.".into(),
            rev: "1-a".into(),
            publish: None,
        };
        let style = Default::default();
        let first = publish_note(
            &shares,
            &session,
            &store,
            &source,
            &Palette::LIGHT,
            &style,
            None,
            10,
        )
        .unwrap();
        store_publish(&store, &id, Some(&first)).unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(KansoPublish::from_extra(&note.extra), Some(first.clone()));
        let stored = shares.get(&first.share_id).unwrap();
        let page =
            open_share(&first.share_key().unwrap(), &first.share_id, &stored.sealed).unwrap();
        let page = String::from_utf8(page).unwrap();
        assert!(page.contains("<title>Plan</title>") && page.contains("The secret plan."));
        assert!(!page.contains("<script"));
        assert!(!String::from_utf8_lossy(&stored.sealed).contains("secret plan"));
        assert!(!first.is_outdated("Plan", "The secret plan."));

        // Edited: the same link, new contents.
        source.body = "The revised plan.".into();
        source.publish = Some(first.clone());
        let second = publish_note(
            &shares,
            &session,
            &store,
            &source,
            &Palette::LIGHT,
            &style,
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            (&second.share_id, &second.key),
            (&first.share_id, &first.key)
        );
        assert!(!second.is_outdated("Plan", "The revised plan."));

        // A failed upload says why and keeps nothing.
        shares.go_offline(1);
        let err = publish_note(
            &shares,
            &session,
            &store,
            &source,
            &Palette::LIGHT,
            &style,
            None,
            30,
        )
        .unwrap_err();
        assert!(error_text(&err).contains("connection"));

        store_publish(&store, &id, None).unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert!(!note.extra.contains_key(PUBLISH_KEY));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round trip: a link published for any of the choices reads back
        /// as that choice, wherever in time it was published.
        #[test]
        fn prop_expiry_choice_round_trips(ix in 0usize..4, at in 0i64..4_000_000_000_000) {
            let expiry = Expiry::ALL[ix];
            let p = published(at, expiry.at(at), "t", "b");
            prop_assert_eq!(expiry_choice(&p), expiry);
        }

        /// A fresh publish never needs an update until the note, the choice
        /// or the clock moves past it.
        #[test]
        fn prop_fresh_publishes_are_up_to_date(ix in 0usize..4, title in "\\PC{0,20}", body in "\\PC{0,60}", at in 0i64..4_000_000_000_000) {
            let expiry = Expiry::ALL[ix];
            let p = published(at, expiry.at(at), &title, &body);
            prop_assert!(!needs_update(&p, &title, &body, expiry, at));
            let edited = format!("{body}.");
            prop_assert!(needs_update(&p, &title, &edited, expiry, at));
        }
    }
}
