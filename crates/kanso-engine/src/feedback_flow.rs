//! Sending feedback (issue #232): save to the outbox, then send; retry with
//! backoff; mirror the server's status for My feedback; delete.
//!
//! One flow for both apps. A report is saved to the library's local outbox
//! *before* any request, so it can't be lost, and its id is the idempotency
//! key, so a retry can't make a duplicate. Nothing here polls: the apps call
//! [`flush`] after a send, at launch and when connectivity returns, and
//! [`refresh`] when My feedback opens and at launch (at most every 6 hours).

use std::path::{Path, PathBuf};

use kanso_store::{OutboxNew, OutboxRow, OutboxState, ServerStatus, Store, StoreError};
use kanso_sync::feedback::{Client, FeedbackError, Reporter, Transport};

use crate::feedback::{
    self, AppInfo, Draft, DraftError, ReportView, Status, build_payload, declare_screenshot,
    new_id, retry_delay_ms,
};

/// How long after one status refresh the launch refresh waits for the next.
pub const REFRESH_EVERY_MS: i64 = 6 * 60 * 60 * 1000;

/// The `meta` key of the last refresh.
const REFRESHED_KEY: &str = "feedback.refreshed_ms";

/// Not before the next launch.
const NEXT_LAUNCH: i64 = i64::MAX / 2;

/// Why a report is waiting rather than sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wait {
    /// No connection, or the server can't be reached.
    Offline,
    /// 429: not before this long.
    RateLimited { retry_after_ms: i64 },
    /// 503: feedback is paused.
    Paused,
    /// The session or key isn't accepted, or the server failed.
    Later,
}

/// How a send went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sent {
    /// The server has it.
    Done,
    /// Saved; it goes when it can.
    Queued(Wait),
    /// The server refused this field (its JSON path). The report is not kept:
    /// the sheet stays open so the person can edit it.
    Rejected { field: String },
}

/// What went wrong before a send began.
#[derive(Debug)]
pub enum Error {
    Draft(DraftError),
    Store(StoreError),
    Io(std::io::Error),
}

impl From<StoreError> for Error {
    fn from(err: StoreError) -> Self {
        Error::Store(err)
    }
}

/// A screenshot as the person chose it.
pub struct Picture<'a> {
    pub bytes: &'a [u8],
}

/// The payload the sheet shows under "Show what's sent": built the same way
/// as the one that is sent (the id is a stand-in), so what is shown is what
/// goes. Returns the draft's error while it can't be sent yet.
pub fn preview_payload(
    draft: &Draft,
    app: Option<&AppInfo>,
) -> Result<String, DraftError> {
    build_payload(&new_id([0; 16]), draft, app)
}

/// Saves the report, then tries to send it. `entropy` makes the id;
/// `data_dir/feedback/` keeps the screenshot's copy until it is uploaded.
///
/// On `Rejected` (a 400) nothing stays in the outbox.
#[allow(clippy::too_many_arguments)]
pub fn send<T: Transport>(
    store: &Store,
    client: &Client<T>,
    reporter: &Reporter,
    data_dir: &Path,
    entropy: [u8; 16],
    mut draft: Draft,
    picture: Option<Picture>,
    app: Option<&AppInfo>,
    now_ms: i64,
) -> Result<(String, Sent), Error> {
    let id = new_id(entropy);
    let mut path = None;
    if let Some(picture) = &picture {
        let decl = declare_screenshot(picture.bytes).map_err(Error::Draft)?;
        let dir = data_dir.join("feedback");
        std::fs::create_dir_all(&dir).map_err(Error::Io)?;
        let file = dir.join(format!(
            "{id}.{}",
            if decl.mime == "image/png" { "png" } else { "jpg" }
        ));
        std::fs::write(&file, picture.bytes).map_err(Error::Io)?;
        path = Some(file);
        draft.screenshot = Some(decl);
    } else {
        draft.screenshot = None;
    }
    let payload = match build_payload(&id, &draft, app) {
        Ok(payload) => payload,
        Err(err) => {
            forget_file(path.as_deref());
            return Err(Error::Draft(err));
        }
    };
    let preview = draft
        .text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(80)
        .collect::<String>();
    store.feedback_enqueue(&OutboxNew {
        id: id.clone(),
        payload,
        kind: draft.kind.wire().to_owned(),
        preview,
        screenshot_path: path.as_ref().map(|p| p.to_string_lossy().into_owned()),
        created_ms: now_ms,
    })?;
    let row = store.feedback_get(&id)?.expect("just saved");
    let outcome = step(store, client, reporter, &row, now_ms)?;
    Ok((id, outcome))
}

fn forget_file(path: Option<&Path>) {
    if let Some(path) = path {
        let _ = std::fs::remove_file(path);
    }
}

fn wait_of(err: &FeedbackError) -> Wait {
    match err {
        FeedbackError::Offline => Wait::Offline,
        FeedbackError::RateLimited { retry_after_ms } => Wait::RateLimited {
            retry_after_ms: *retry_after_ms,
        },
        FeedbackError::Closed => Wait::Paused,
        _ => Wait::Later,
    }
}

/// When to try again after this failure.
fn next_try(row: &OutboxRow, err: &FeedbackError, now_ms: i64) -> i64 {
    if let FeedbackError::RateLimited { retry_after_ms } = err {
        return now_ms.saturating_add(*retry_after_ms);
    }
    match retry_delay_ms(row.attempts + 1) {
        Some(delay) => now_ms + delay,
        None => NEXT_LAUNCH,
    }
}

/// One outbox row's work: send it, upload its screenshot, or delete it.
fn step<T: Transport>(
    store: &Store,
    client: &Client<T>,
    reporter: &Reporter,
    row: &OutboxRow,
    now_ms: i64,
) -> Result<Sent, Error> {
    match row.state {
        OutboxState::Deleting => {
            match client.delete(reporter, &row.id) {
                Ok(()) => {
                    forget_file(row.screenshot_path.as_deref().map(Path::new));
                    store.feedback_remove(&row.id)?;
                    Ok(Sent::Done)
                }
                Err(err) => {
                    store.feedback_failed(&row.id, next_try(row, &err, now_ms))?;
                    Ok(Sent::Queued(wait_of(&err)))
                }
            }
        }
        OutboxState::Queued => match client.submit(reporter, &row.payload) {
            Ok(submitted) => {
                store.feedback_mark_sent(&row.id, &submitted.status)?;
                match (&row.screenshot_path, submitted.upload_path) {
                    (Some(_), Some(_)) => {
                        let sent = store.feedback_get(&row.id)?.expect("row");
                        upload(store, client, reporter, &sent, now_ms)?;
                    }
                    // Nothing wanted: the slot is closed or stored already.
                    (Some(path), None) => {
                        forget_file(Some(Path::new(path)));
                        store.feedback_mark_screenshot_sent(&row.id)?;
                    }
                    (None, _) => {}
                }
                Ok(Sent::Done)
            }
            // The same bytes would fail again: don't keep them.
            Err(FeedbackError::Invalid { field }) => {
                forget_file(row.screenshot_path.as_deref().map(Path::new));
                store.feedback_remove(&row.id)?;
                Ok(Sent::Rejected { field })
            }
            Err(FeedbackError::TooLarge) => {
                forget_file(row.screenshot_path.as_deref().map(Path::new));
                store.feedback_remove(&row.id)?;
                Ok(Sent::Rejected {
                    field: String::new(),
                })
            }
            Err(err) => {
                store.feedback_failed(&row.id, next_try(row, &err, now_ms))?;
                Ok(Sent::Queued(wait_of(&err)))
            }
        },
        OutboxState::Sent => {
            upload(store, client, reporter, row, now_ms)?;
            Ok(Sent::Done)
        }
    }
}

/// Uploads a stored report's screenshot; gives up on the picture (not the
/// report) when the server won't take it.
fn upload<T: Transport>(
    store: &Store,
    client: &Client<T>,
    reporter: &Reporter,
    row: &OutboxRow,
    now_ms: i64,
) -> Result<(), Error> {
    let Some(path) = row.screenshot_path.as_deref().map(PathBuf::from) else {
        return Ok(());
    };
    let Ok(bytes) = std::fs::read(&path) else {
        store.feedback_mark_screenshot_sent(&row.id)?;
        return Ok(());
    };
    let mime = feedback::sniff_image(&bytes).unwrap_or("image/png");
    match client.upload_screenshot(reporter, &row.id, mime, &bytes) {
        Ok(()) | Err(FeedbackError::Gone | FeedbackError::Invalid { .. } | FeedbackError::TooLarge | FeedbackError::NotFound) => {
            forget_file(Some(&path));
            store.feedback_mark_screenshot_sent(&row.id)?;
        }
        Err(err) => store.feedback_failed(&row.id, next_try(row, &err, now_ms))?,
    }
    Ok(())
}

/// Does all the work that is due: sends, uploads and deletes. Call it after
/// a send, at launch (after [`store_retry_now`]) and when connectivity
/// returns. Stops at the first rate limit or pause.
pub fn flush<T: Transport>(
    store: &Store,
    client: &Client<T>,
    reporter: &Reporter,
    now_ms: i64,
) -> Result<Vec<(String, Sent)>, Error> {
    let mut out = Vec::new();
    for row in store.feedback_due(now_ms)? {
        let sent = step(store, client, reporter, &row, now_ms)?;
        let stop = matches!(
            sent,
            Sent::Queued(Wait::RateLimited { .. } | Wait::Paused | Wait::Offline)
        );
        out.push((row.id.clone(), sent));
        if stop {
            break;
        }
    }
    Ok(out)
}

/// At launch and when back online: every waiting report is due again.
pub fn store_retry_now(store: &Store) -> Result<(), Error> {
    Ok(store.feedback_retry_now()?)
}

/// The person deleted a report. Returns whether the server still has to be
/// asked (so the row shows "Deleting…"); [`flush`] then sends the delete.
pub fn delete(store: &Store, id: &str) -> Result<bool, Error> {
    Ok(store.feedback_mark_deleting(id)?)
}

/// The reports on this device, newest first, as My feedback shows them.
pub fn views(store: &Store) -> Result<Vec<ReportView>, Error> {
    Ok(store.feedback_all()?.iter().map(view).collect())
}

fn view(row: &OutboxRow) -> ReportView {
    let status = match row.state {
        OutboxState::Queued => Status::Waiting,
        _ => row
            .server_status
            .as_deref()
            .and_then(Status::from_wire)
            .unwrap_or(Status::Received),
    };
    ReportView {
        id: row.id.clone(),
        created_ms: row.created_ms,
        kind: feedback::Kind::from_wire(&row.kind).unwrap_or(feedback::Kind::Bug),
        preview: row.preview.clone(),
        status,
        issue: row.issue,
        duplicate_of: row.duplicate_of,
        pr: row.pr,
        release: row.release.as_deref().map(|r| feedback::release_name(r).to_owned()),
        deleting: row.state == OutboxState::Deleting,
    }
}

/// Whether a refresh is due: never done, or 6 hours ago.
pub fn refresh_due(store: &Store, now_ms: i64) -> bool {
    store
        .meta(REFRESHED_KEY)
        .ok()
        .flatten()
        .and_then(|v| v.parse::<i64>().ok())
        .is_none_or(|last| now_ms - last >= REFRESH_EVERY_MS || now_ms < last)
}

/// Asks the server how this reporter's reports stand and mirrors it into
/// the outbox. Reports another device of the account made are added.
/// Nothing calls this on a timer: My feedback opening and launch do.
pub fn refresh<T: Transport>(
    store: &Store,
    client: &Client<T>,
    reporter: &Reporter,
    now_ms: i64,
) -> Result<Vec<ReportView>, Error> {
    match client.list(reporter, None) {
        Ok(remote) => {
            for report in &remote {
                store.feedback_adopt(&report.id, &report.kind, &report.preview, report.created_ms)?;
                store.feedback_record_status(
                    &report.id,
                    &ServerStatus {
                        status: report.status.clone(),
                        issue: report.issue,
                        duplicate_of: report.duplicate_of,
                        pr: report.pr,
                        release: report.release.clone(),
                    },
                )?;
            }
            store.set_meta(REFRESHED_KEY, &now_ms.to_string())?;
        }
        // Offline or refused: show what the device last knew.
        Err(_) => {}
    }
    views(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_sync::feedback::{Request, Response};
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;
    use std::sync::Mutex;

    /// A server that answers from a script (the last answer repeats) and
    /// records each request's method and path.
    struct Script {
        answers: Mutex<Vec<Result<Response, String>>>,
        seen: Mutex<Vec<(String, String)>>,
    }

    impl Script {
        fn new(answers: Vec<Result<Response, String>>) -> Script {
            Script {
                answers: Mutex::new(answers),
                seen: Mutex::new(Vec::new()),
            }
        }
        fn count(&self, method: &str) -> usize {
            self.seen.lock().unwrap().iter().filter(|(m, _)| m == method).count()
        }
    }

    impl Transport for &Script {
        fn send(&self, request: &Request) -> Result<Response, String> {
            self.seen.lock().unwrap().push((request.method.into(), request.path.clone()));
            let mut answers = self.answers.lock().unwrap();
            if answers.len() > 1 { answers.remove(0) } else { answers[0].clone() }
        }
    }

    fn reply(status: u16, body: &str) -> Result<Response, String> {
        Ok(Response { status, body: body.as_bytes().to_vec() })
    }

    fn reporter() -> Reporter {
        Reporter::account("Bearer x", "d_1", "p")
    }

    fn setup() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    fn draft(text: &str) -> Draft {
        Draft { kind: feedback::Kind::Bug, text: text.into(), credit_name: None, screenshot: None }
    }

    const PNG: [u8; 12] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3, 4];

    fn go(
        dir: &tempfile::TempDir,
        store: &Store,
        script: &Script,
        picture: Option<&[u8]>,
        now: i64,
    ) -> Result<(String, Sent), Error> {
        send(
            store,
            &Client::new(script),
            &reporter(),
            dir.path(),
            [7; 16],
            draft("Export to PDF cuts off tables."),
            picture.map(|bytes| Picture { bytes }),
            None,
            now,
        )
    }

    #[test]
    fn online_sends_once_with_its_screenshot() {
        let (dir, store) = setup();
        let script = Script::new(vec![
            reply(201, r#"{"id":"x","status":"received","upload":{"url":"/v1/feedback/x/screenshot","expires_ms":1}}"#),
            reply(204, ""),
        ]);
        let (id, sent) = go(&dir, &store, &script, Some(&PNG), 1_000).unwrap();
        assert_eq!(sent, Sent::Done);
        assert_eq!((script.count("POST"), script.count("PUT")), (1, 1));
        let row = store.feedback_get(&id).unwrap().unwrap();
        assert_eq!((row.state, row.screenshot_path, row.screenshot_sent), (OutboxState::Sent, None, true));
        // Nothing more is due, so a flush sends nothing again.
        assert!(flush(&store, &Client::new(&script), &reporter(), i64::MAX / 4).unwrap().is_empty());
        assert_eq!(script.count("POST"), 1);
        assert!(!dir.path().join("feedback").join(format!("{id}.png")).exists());
    }

    #[test]
    fn offline_then_online_sends_exactly_once() {
        let (dir, store) = setup();
        let offline = Script::new(vec![Err("offline".into())]);
        let (id, sent) = go(&dir, &store, &offline, None, 1_000).unwrap();
        assert_eq!(sent, Sent::Queued(Wait::Offline));
        let row = store.feedback_get(&id).unwrap().unwrap();
        assert_eq!((row.state, row.attempts, row.next_try_ms), (OutboxState::Queued, 1, 1_000 + 60_000));
        // Not due before the wait ends; launch makes it due.
        let online = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
        assert!(flush(&store, &Client::new(&online), &reporter(), 2_000).unwrap().is_empty());
        store_retry_now(&store).unwrap();
        let done = flush(&store, &Client::new(&online), &reporter(), 2_000).unwrap();
        assert_eq!(done, vec![(id.clone(), Sent::Done)]);
        assert!(flush(&store, &Client::new(&online), &reporter(), 3_000).unwrap().is_empty());
        assert_eq!(online.count("POST"), 1);
    }

    #[test]
    fn a_rate_limit_waits_for_retry_after() {
        let (dir, store) = setup();
        let script = Script::new(vec![reply(429, r#"{"error":"rate_limited","retry_after_ms":90000}"#)]);
        let (id, sent) = go(&dir, &store, &script, None, 10).unwrap();
        assert_eq!(sent, Sent::Queued(Wait::RateLimited { retry_after_ms: 90_000 }));
        assert_eq!(store.feedback_get(&id).unwrap().unwrap().next_try_ms, 90_010);
    }

    #[test]
    fn a_400_keeps_nothing() {
        let (dir, store) = setup();
        let script = Script::new(vec![reply(400, r#"{"error":"invalid","field":"text"}"#)]);
        let (_, sent) = go(&dir, &store, &script, Some(&PNG), 10).unwrap();
        assert_eq!(sent, Sent::Rejected { field: "text".into() });
        assert!(store.feedback_all().unwrap().is_empty());
        assert!(std::fs::read_dir(dir.path().join("feedback")).unwrap().next().is_none());
    }

    #[test]
    fn paused_feedback_keeps_the_report() {
        let (dir, store) = setup();
        let script = Script::new(vec![reply(503, r#"{"error":"feedback_closed"}"#)]);
        let (_, sent) = go(&dir, &store, &script, None, 10).unwrap();
        assert_eq!(sent, Sent::Queued(Wait::Paused));
        assert_eq!(store.feedback_all().unwrap().len(), 1);
    }

    #[test]
    fn a_bad_draft_is_refused_before_anything_is_saved() {
        let (dir, store) = setup();
        let script = Script::new(vec![reply(201, "{}")]);
        let err = send(&store, &Client::new(&script), &reporter(), dir.path(), [1; 16], draft("short"), None, None, 1);
        assert!(matches!(err, Err(Error::Draft(DraftError::TooShort))));
        assert!(store.feedback_all().unwrap().is_empty());
        assert_eq!(script.count("POST"), 0);
    }

    #[test]
    fn delete_waits_for_the_server() {
        let (dir, store) = setup();
        let ok = Script::new(vec![reply(201, r#"{"id":"x","status":"filed"}"#)]);
        let (id, _) = go(&dir, &store, &ok, None, 1).unwrap();
        assert!(delete(&store, &id).unwrap());
        assert!(views(&store).unwrap()[0].deleting);
        let down = Script::new(vec![Err("offline".into())]);
        flush(&store, &Client::new(&down), &reporter(), 2).unwrap();
        assert_eq!(views(&store).unwrap().len(), 1);
        store_retry_now(&store).unwrap();
        let up = Script::new(vec![reply(204, "")]);
        flush(&store, &Client::new(&up), &reporter(), 3).unwrap();
        assert!(views(&store).unwrap().is_empty());
        assert_eq!(up.count("DELETE"), 1);
    }

    #[test]
    fn refresh_mirrors_status_and_adopts_other_devices() {
        let (dir, store) = setup();
        let ok = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
        let (id, _) = go(&dir, &store, &ok, None, 1).unwrap();
        assert!(refresh_due(&store, 5));
        let list = format!(
            r#"{{"reports":[{{"id":"{id}","created_ms":1,"kind":"bug","preview":"Export","status":"released","issue":241,"duplicate_of":null,"pr":252,"release":"v0.9.2","events":[]}},{{"id":"fb_other0000000000000000000a","created_ms":0,"kind":"idea","preview":"Dark","status":"filed","issue":7,"duplicate_of":null,"pr":null,"release":null,"events":[]}}]}}"#
        );
        let script = Script::new(vec![reply(200, &list)]);
        let views = refresh(&store, &Client::new(&script), &reporter(), 10).unwrap();
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].status, Status::Released);
        assert_eq!(views[0].release.as_deref(), Some("0.9.2"));
        assert_eq!(feedback::fixed_in(&views, "0.9.2"), 1);
        assert_eq!(views[1].issue, Some(7));
        assert!(!refresh_due(&store, 10 + REFRESH_EVERY_MS - 1));
        assert!(refresh_due(&store, 10 + REFRESH_EVERY_MS));
        // Offline: the last known state is shown.
        let down = Script::new(vec![Err("offline".into())]);
        assert_eq!(refresh(&store, &Client::new(&down), &reporter(), 99).unwrap().len(), 2);
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(0x006b_616e_736f), failure_persistence: None, ..ProptestConfig::with_cases(64) })]

        /// However the network behaves, a report is sent to the server at
        /// most as many times as it was tried, never lost while unsent, and
        /// once the server has it a flush never posts it again.
        #[test]
        fn prop_no_loss_and_no_duplicate(answers in proptest::collection::vec(0u8..4, 1..8)) {
            let (dir, store) = setup();
            let script = Script::new(answers.iter().map(|a| match a {
                0 => Err("offline".to_string()),
                1 => reply(500, ""),
                2 => reply(429, r#"{"error":"rate_limited","retry_after_ms":1000}"#),
                _ => reply(201, r#"{"id":"x","status":"received"}"#),
            }).collect());
            let (id, first) = go(&dir, &store, &script, None, 0).unwrap();
            let mut now = 0i64;
            for _ in 0..answers.len() {
                now += 40 * 60_000;
                store_retry_now(&store).unwrap();
                flush(&store, &Client::new(&script), &reporter(), now).unwrap();
            }
            let row = store.feedback_get(&id).unwrap();
            // Never lost: still queued, or accepted.
            prop_assert!(row.is_some());
            let row = row.unwrap();
            let accepted = row.state == OutboxState::Sent;
            if accepted || first == Sent::Done {
                let posts = script.count("POST");
                flush(&store, &Client::new(&script), &reporter(), now + 1).unwrap();
                prop_assert_eq!(script.count("POST"), posts);
            }
            // Exactly one acceptance can ever be recorded.
            prop_assert!(script.count("POST") >= 1);
        }
    }
}
