//! The one request (docs/METRICS.md › Sending): the batch as JSON to Kanso's
//! own server, whichever server the library syncs with, with nothing else
//! (no session, cookie or device ID). Blocking: call it off the UI thread.

use std::time::Duration;

use ureq::Agent;

use crate::batch::{Batch, MAX_BYTES};

/// Where every batch goes.
pub const ENDPOINT: &str = "https://sync.kansoapp.app/v1/metrics";

/// A count isn't worth waiting for.
const TIMEOUT: Duration = Duration::from_secs(15);

/// How a send went, and so what the ledger does with the batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 204: its events are done for their day.
    Accepted,
    /// 400 or 413: not a valid batch. It would fail again, so it is dropped.
    Rejected,
    /// 429, any other answer or none: kept for a later try the same day.
    Retry,
    /// Not sent: this build never sends.
    Off,
}

impl Outcome {
    /// What an answer's status means.
    pub fn from_status(status: u16) -> Outcome {
        match status {
            204 => Outcome::Accepted,
            400 | 413 => Outcome::Rejected,
            _ => Outcome::Retry,
        }
    }
}

/// Sends `batch` when `enabled` (a release build of the release flavour,
/// never a test; the caller decides, and only after the notice was seen).
// no-prop-test: the network; `post` is tested against a local server and
// `Outcome::from_status` below.
pub fn send(batch: &Batch, enabled: bool) -> Outcome {
    if !enabled {
        return Outcome::Off;
    }
    post(ENDPOINT, batch)
}

/// POSTs `batch` to `url`.
fn post(url: &str, batch: &Batch) -> Outcome {
    let body = batch.to_json();
    if body.len() > MAX_BYTES {
        return Outcome::Rejected;
    }
    // An Android app that hasn't handed over its Context gets a later try,
    // not a panic in the handshake.
    if kanso_tls::ready_for(url).is_err() {
        return Outcome::Retry;
    }
    let agent: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(TIMEOUT))
        .user_agent("Kanso")
        .tls_config(kanso_tls::ureq_tls())
        .build()
        .into();
    match agent
        .post(url)
        .header("content-type", "application/json")
        .send(&body[..])
    {
        Ok(response) => Outcome::from_status(response.status().as_u16()),
        Err(_) => Outcome::Retry,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Active, Device, Form, Ledger, Platform, SyncMode};
    use proptest::prelude::*;

    fn batch() -> Batch {
        let mut ledger = Ledger::default();
        let id = ledger.install_id();
        ledger
            .due(
                crate::Day::today(),
                &id,
                &Device::new(Platform::Linux, Form::Desktop, "0.9.0"),
                &Active::new("6.1", SyncMode::Off, true, 0),
            )
            .unwrap()
    }

    /// One request to a local server answering `status`: the body and
    /// header names it received, and what the send made of the answer.
    fn exchange(status: u16) -> (String, Vec<String>, Outcome) {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let url = format!(
            "http://{}/v1/metrics",
            server.server_addr().to_ip().unwrap()
        );
        let batch = batch();
        let sender = std::thread::spawn(move || post(&url, &batch));
        let mut request = server.recv().unwrap();
        assert_eq!(request.method(), &tiny_http::Method::Post);
        assert_eq!(request.url(), "/v1/metrics");
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).unwrap();
        let headers = request
            .headers()
            .iter()
            .map(|h| h.field.as_str().as_str().to_ascii_lowercase())
            .collect();
        request.respond(tiny_http::Response::empty(status)).unwrap();
        (body, headers, sender.join().unwrap())
    }

    #[test]
    fn sends_the_batch_and_reads_the_answer() {
        let (body, headers, outcome) = exchange(204);
        assert_eq!(outcome, Outcome::Accepted);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["v"], 1);
        assert_eq!(json["events"][0]["event"], "app_active");
        assert!(headers.contains(&"content-type".to_owned()));
        for header in ["cookie", "authorization"] {
            assert!(!headers.contains(&header.to_owned()), "{header}");
        }
        assert_eq!(exchange(400).2, Outcome::Rejected);
        assert_eq!(exchange(429).2, Outcome::Retry);
        assert_eq!(exchange(500).2, Outcome::Retry);
        // Nobody listening: kept for later.
        let closed = {
            let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
            server.server_addr().to_ip().unwrap()
        };
        assert_eq!(
            post(&format!("http://{closed}/v1/metrics"), &batch()),
            Outcome::Retry
        );
    }

    #[test]
    fn statuses() {
        assert_eq!(Outcome::from_status(204), Outcome::Accepted);
        assert_eq!(Outcome::from_status(400), Outcome::Rejected);
        assert_eq!(Outcome::from_status(413), Outcome::Rejected);
        assert_eq!(Outcome::from_status(429), Outcome::Retry);
        assert_eq!(Outcome::from_status(200), Outcome::Retry);
        // Off sends nothing, whatever the batch.
        assert_eq!(send(&batch(), false), Outcome::Off);
    }

    proptest! {
        /// Only 204 counts as sent and only 400 and 413 drop the batch.
        #[test]
        fn prop_only_204_is_sent(status in 100u16..600) {
            let outcome = Outcome::from_status(status);
            prop_assert_eq!(outcome == Outcome::Accepted, status == 204);
            prop_assert_eq!(outcome == Outcome::Rejected, status == 400 || status == 413);
        }
    }
}
