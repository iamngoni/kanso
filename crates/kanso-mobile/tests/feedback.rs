//! In-app feedback on the phone (issue #232), through the facade: a report
//! is saved before it is sent, an offline one waits and goes once, a refused
//! one is not kept, My feedback mirrors the server, and nothing of a note is
//! in what leaves the phone.

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use kanso_mobile::{
    FeedbackChipKind, FeedbackContext, FeedbackCredit, FeedbackGithubUser, FeedbackInput,
    FeedbackProblem, FeedbackSent, FeedbackUsers,
};
use kanso_sync::feedback::{Request, Response, Transport};

/// A server that answers from a script (the last answer repeats) and keeps
/// every request.
struct Script {
    answers: Mutex<Vec<Result<Response, String>>>,
    seen: Mutex<Vec<Request>>,
}

impl Script {
    fn new(answers: Vec<Result<Response, String>>) -> Arc<Script> {
        Arc::new(Script {
            answers: Mutex::new(answers),
            seen: Mutex::new(Vec::new()),
        })
    }

    fn count(&self, method: &str) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method == method)
            .count()
    }
}

impl Transport for Script {
    fn send(&self, request: &Request) -> Result<Response, String> {
        self.seen.lock().unwrap().push(request.clone());
        let mut answers = self.answers.lock().unwrap();
        if answers.len() > 1 {
            answers.remove(0)
        } else {
            answers[0].clone()
        }
    }
}

fn reply(status: u16, body: &str) -> Result<Response, String> {
    Ok(Response {
        status,
        body: body.as_bytes().to_vec(),
    })
}

fn input(text: &str) -> FeedbackInput {
    FeedbackInput {
        kind: "bug".into(),
        text: text.into(),
        credit: None,
        include_app: true,
        screenshot: None,
        context: FeedbackContext {
            version: "0.9.4".into(),
            build: "57".into(),
            os: "18.1".into(),
            device: "iPhone15,2".into(),
            locale: "en-GB".into(),
            screen: "settings".into(),
            crash_types: vec![],
        },
    }
}

// no-prop-test: the facade end to end; the engine's flow and the payload
// have their properties in kanso-engine.
#[test]
fn an_offline_report_waits_and_goes_once() {
    let phone = lone_phone();
    let offline = Script::new(vec![Err("offline".into())]);
    let sent = phone
        .library
        .feedback_over(offline.clone())
        .send(input("Export to PDF cuts off wide tables."))
        .unwrap();
    assert_eq!(sent, FeedbackSent::Queued);
    let waiting = phone.library.feedback_over(offline).reports(false).unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].chip, FeedbackChipKind::WaitingToSend);
    assert!(!waiting[0].on_server);

    let online = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
    let feedback = phone.library.feedback_over(online.clone());
    feedback.retry_now().unwrap();
    feedback.flush().unwrap();
    feedback.flush().unwrap();
    assert_eq!(online.count("POST"), 1);
    let reports = feedback.reports(false).unwrap();
    assert_eq!(reports[0].chip, FeedbackChipKind::Received);
    assert!(reports[0].on_server);
}

// no-prop-test: see above.
#[test]
fn a_short_report_is_invalid_and_nothing_is_saved() {
    let phone = lone_phone();
    let server = Script::new(vec![reply(201, "{}")]);
    let feedback = phone.library.feedback_over(server.clone());
    assert_eq!(
        feedback.send(input("short")).unwrap(),
        FeedbackSent::Invalid {
            problem: FeedbackProblem::TooShort
        }
    );
    assert!(feedback.reports(false).unwrap().is_empty());
    assert_eq!(server.count("POST"), 0);
}

// no-prop-test: see above.
#[test]
fn the_servers_refusals_and_pauses_are_told() {
    let phone = lone_phone();
    let refused = Script::new(vec![reply(
        400,
        r#"{"error":"invalid","field":"app.version"}"#,
    )]);
    let feedback = phone.library.feedback_over(refused);
    assert_eq!(
        feedback
            .send(input("Export to PDF cuts off wide tables."))
            .unwrap(),
        FeedbackSent::Rejected {
            field: "app.version".into()
        }
    );
    assert!(feedback.reports(false).unwrap().is_empty());

    let paused = Script::new(vec![reply(503, r#"{"error":"feedback_closed"}"#)]);
    let feedback = phone.library.feedback_over(paused);
    assert_eq!(
        feedback
            .send(input("Export to PDF cuts off wide tables."))
            .unwrap(),
        FeedbackSent::Paused
    );
    let limited = Script::new(vec![reply(
        429,
        r#"{"error":"rate_limited","retry_after_ms":86400000}"#,
    )]);
    let feedback = phone.library.feedback_over(limited);
    assert_eq!(
        feedback
            .send(input("Another report about tables."))
            .unwrap(),
        FeedbackSent::RateLimited
    );
    assert_eq!(feedback.reports(false).unwrap().len(), 2);
}

// no-prop-test: see above.
#[test]
fn my_feedback_mirrors_the_server_and_marks_what_was_seen() {
    let phone = lone_phone();
    let accept = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
    let id = {
        let feedback = phone.library.feedback_over(accept);
        feedback
            .send(input("Export to PDF cuts off wide tables."))
            .unwrap();
        feedback.reports(false).unwrap()[0].id.clone()
    };
    let list = format!(
        r#"{{"reports":[{{"id":"{id}","created_ms":1,"kind":"bug","preview":"Export to PDF","status":"released","issue":241,"duplicate_of":null,"pr":252,"release":"v0.9.2","events":[]}}]}}"#
    );
    let server = Script::new(vec![reply(200, &list)]);
    let feedback = phone.library.feedback_over(server);
    assert!(feedback.refresh_due());
    let reports = feedback.reports(true).unwrap();
    assert_eq!(reports[0].chip, FeedbackChipKind::ReleasedIn);
    assert_eq!(reports[0].release.as_deref(), Some("0.9.2"));
    assert_eq!(
        reports[0].issue_url.as_deref(),
        Some("https://github.com/iamngoni/kanso/issues/241")
    );
    assert!(reports[0].fresh);
    assert!(!feedback.refresh_due());
    feedback.mark_seen().unwrap();
    assert!(!feedback.reports(false).unwrap()[0].fresh);
}

// no-prop-test: see above.
#[test]
fn deleting_waits_for_the_server_and_then_goes() {
    let phone = lone_phone();
    let accept = Script::new(vec![reply(201, r#"{"id":"x","status":"filed"}"#)]);
    let feedback = phone.library.feedback_over(accept);
    feedback
        .send(input("Export to PDF cuts off wide tables."))
        .unwrap();
    let id = feedback.reports(false).unwrap()[0].id.clone();

    let down = Script::new(vec![Err("offline".into())]);
    let offline = phone.library.feedback_over(down);
    offline.delete(id.clone()).unwrap();
    let rows = offline.reports(false).unwrap();
    assert_eq!(rows[0].chip, FeedbackChipKind::Deleting);

    let up = Script::new(vec![reply(204, "")]);
    let online = phone.library.feedback_over(up.clone());
    online.retry_now().unwrap();
    online.flush().unwrap();
    assert!(online.reports(false).unwrap().is_empty());
    assert_eq!(up.count("DELETE"), 1);
}

// no-prop-test: see above.
#[test]
fn nothing_of_a_note_leaves_the_phone() {
    let phone = lone_phone();
    let secret = "zebra crossing quartz harbour lantern";
    note(&phone.library, "Private plan", secret);
    let server = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
    let feedback = phone.library.feedback_over(server.clone());
    feedback
        .send(input("Export to PDF cuts off wide tables."))
        .unwrap();
    for request in server.seen.lock().unwrap().iter() {
        let body = String::from_utf8_lossy(&request.body);
        assert!(!body.contains("zebra") && !body.contains("Private plan"));
    }
}

// no-prop-test: the facade end to end; the credit rules have their properties in kanso-engine.
#[test]
fn a_github_credit_is_sent_tagged_and_a_bad_login_is_invalid() {
    let phone = lone_phone();
    let server = Script::new(vec![reply(201, r#"{"id":"x","status":"received"}"#)]);
    let feedback = phone.library.feedback_over(server.clone());
    let mut report = input("Export to PDF cuts off wide tables.");
    report.credit = Some(FeedbackCredit::Github {
        login: "octocat".into(),
    });
    assert_eq!(feedback.send(report).unwrap(), FeedbackSent::Sent);
    let seen = server.seen.lock().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&seen[0].body).unwrap();
    assert_eq!(
        body["credit"],
        serde_json::json!({"kind": "github", "login": "octocat"})
    );
    drop(seen);

    for login in ["", "@octocat", "a--b"] {
        let mut bad = input("Export to PDF cuts off wide tables.");
        bad.credit = Some(FeedbackCredit::Github {
            login: login.into(),
        });
        assert_eq!(
            feedback.send(bad).unwrap(),
            FeedbackSent::Invalid {
                problem: FeedbackProblem::BadGithubUser
            }
        );
    }
    let mut mention = input("Export to PDF cuts off wide tables.");
    mention.credit = Some(FeedbackCredit::Name {
        name: "@octocat".into(),
    });
    assert_eq!(
        feedback.send(mention).unwrap(),
        FeedbackSent::Invalid {
            problem: FeedbackProblem::BadCredit
        }
    );
    assert_eq!(server.count("POST"), 1);
}

// no-prop-test: see above.
#[test]
fn the_user_search_goes_through_the_worker_and_fails_softly() {
    let phone = lone_phone();
    let server = Script::new(vec![reply(
        200,
        r#"{"users":[{"login":"octocat","avatar_url":"https://avatars.githubusercontent.com/u/583231?v=4"}]}"#,
    )]);
    let feedback = phone.library.feedback_over(server.clone());
    assert_eq!(
        feedback.search_users("@octo".into()).unwrap(),
        FeedbackUsers::Found {
            users: vec![FeedbackGithubUser {
                login: "octocat".into(),
                avatar_url: "https://avatars.githubusercontent.com/u/583231?v=4".into()
            }]
        }
    );
    assert_eq!(
        server.seen.lock().unwrap()[0].path,
        "/v1/feedback/users?q=octo"
    );
    // Too short: nothing is asked.
    assert_eq!(
        feedback.search_users("o".into()).unwrap(),
        FeedbackUsers::Found { users: vec![] }
    );
    assert_eq!(server.seen.lock().unwrap().len(), 1);
    for down in [reply(503, "{}"), reply(429, "{}"), Err("offline".into())] {
        let feedback = phone.library.feedback_over(Script::new(vec![down]));
        assert_eq!(
            feedback.search_users("octo".into()).unwrap(),
            FeedbackUsers::Unavailable
        );
    }
}
