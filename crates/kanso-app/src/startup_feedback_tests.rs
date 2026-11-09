use std::sync::{Arc, Mutex};

use kanso_engine::feedback::{AppInfo, Platform, Screen, SyncMode};
use kanso_sync::MemorySecrets;
use kanso_sync::feedback::{Request, Response, Transport};

use super::{report_text, scrub_paths, send};

struct Fake {
    answer: Result<Response, String>,
    seen: Mutex<Vec<Request>>,
}

impl Transport for Fake {
    fn send(&self, request: &Request) -> Result<Response, String> {
        self.seen.lock().unwrap().push(request.clone());
        self.answer.clone()
    }
}

fn fake(answer: Result<Response, String>) -> Arc<Fake> {
    Arc::new(Fake {
        answer,
        seen: Mutex::new(Vec::new()),
    })
}

fn app() -> AppInfo {
    AppInfo {
        platform: Platform::Macos,
        version: "0.10.0".into(),
        build: "57".into(),
        patch: None,
        os: "15.1".into(),
        device: "Mac14,9".into(),
        locale: "en-GB".into(),
        screen: Screen::Other,
        sync: SyncMode::None,
        errors: vec![],
    }
}

const DETAILS: &str = "Kanso 0.10.0 on windows\nProblem: Kanso can't open your library\nData folder: C:\\Users\\ana\\AppData\\Roaming\\Kanso\nDetails: table note_loops has no column named note_updated_at";

#[test]
fn home_folders_lose_the_name() {
    assert_eq!(
        scrub_paths("in C:\\Users\\ana\\AppData\\Kanso ok"),
        "in ~\\AppData\\Kanso ok"
    );
    assert_eq!(
        scrub_paths("/Users/ana/Library/x and /home/bob/.kanso"),
        "~/Library/x and ~/.kanso"
    );
    assert_eq!(scrub_paths("nothing here"), "nothing here");
}

#[test]
fn the_text_has_the_details_and_the_persons_words_without_the_name() {
    let text = report_text(DETAILS, "  it broke after updating ");
    assert!(text.contains("note_updated_at"));
    assert!(text.contains("it broke after updating"));
    assert!(!text.contains("ana"));
    assert!(text.contains("~\\AppData\\Roaming\\Kanso"));
}

#[test]
fn a_report_reaches_the_worker_in_its_schema() {
    let t = fake(Ok(Response {
        status: 201,
        body: br#"{"id":"fb_x","status":"received"}"#.to_vec(),
    }));
    send(
        t.clone(),
        &MemorySecrets::default(),
        DETAILS,
        "hi there friend",
        &app(),
    )
    .unwrap();
    let seen = t.seen.lock().unwrap();
    let request = &seen[0];
    assert_eq!(
        (request.method, request.path.as_str()),
        ("POST", "/v1/feedback")
    );
    assert!(
        request
            .headers
            .iter()
            .any(|(k, v)| *k == "x-kanso-feedback-key" && v.len() == 43)
    );
    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
    assert!(
        body["id"]
            .as_str()
            .is_some_and(kanso_engine::feedback::is_id)
    );
    assert_eq!(body["kind"], "bug");
    assert_eq!(body["app"]["version"], "0.10.0");
    assert_eq!(body["app"]["platform"], "macos");
    assert!(body["credit"].is_null());
    let text = body["text"].as_str().unwrap();
    assert!(text.contains("note_updated_at") && !text.contains("ana"));
}

#[test]
fn a_failure_says_so_and_a_retry_sends_again() {
    let down = fake(Err("offline".into()));
    let secrets = MemorySecrets::default();
    let err = send(down.clone(), &secrets, DETAILS, "", &app()).unwrap_err();
    assert!(err.contains("Couldn't reach Kanso"));
    let up = fake(Ok(Response {
        status: 201,
        body: br#"{"id":"fb_x","status":"received"}"#.to_vec(),
    }));
    send(up, &secrets, DETAILS, "", &app()).unwrap();
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig {
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x6665_6564),
        ..proptest::prelude::ProptestConfig::with_cases(256)
    })]

    /// Invariant: a home folder's name never survives, on any system's path.
    #[test]
    fn prop_the_user_name_never_survives(
        name in "[a-z]{6,12}",
        tail in "[A-Z]{1,8}",
        style in 0usize..3,
    ) {
        let path = match style {
            0 => format!("C:\\Users\\{name}\\{tail}"),
            1 => format!("/Users/{name}/{tail}"),
            _ => format!("/home/{name}/{tail}"),
        };
        let out = scrub_paths(&format!("x {path} y"));
        proptest::prop_assert!(!out.contains(&name));
        proptest::prop_assert!(out.contains(&tail));
    }
}
