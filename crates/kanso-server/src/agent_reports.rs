//! The agents' problem reports (issue #447): `kanso-report-issue` puts a
//! report here, and nothing more. Nothing leaves the computer from here.
//! The app shows "An agent reported a problem" and opens the feedback sheet
//! filled in with the report; the person may edit it and presses Send.
//!
//! The queue lives in memory, like the [`ReviewQueue`](crate::ReviewQueue):
//! at most [`MAX_PENDING`] reports wait, and a report the same as one
//! already waiting is dropped, so an agent stuck in a loop can't fill it.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{Value, json};

use crate::agent_report_model::{AgentReport, ReportKind};
use crate::mcp::schema;

/// The most reports waiting at once.
pub const MAX_PENDING: usize = 20;

/// What happened to a report put in the queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Queued {
    /// It waits for the person.
    Added,
    /// The same report already waits.
    Duplicate,
    /// [`MAX_PENDING`] already wait.
    Full,
}

#[derive(Default)]
struct Inner {
    pending: Vec<AgentReport>,
    subscribers: Vec<async_channel::Sender<AgentReport>>,
}

/// The reports waiting for the person. Cloning shares the same queue; it
/// rides with the agents' [`TokenStore`](crate::TokenStore).
#[derive(Clone, Default)]
pub struct AgentReports {
    inner: Arc<Mutex<Inner>>,
}

impl fmt::Debug for AgentReports {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentReports")
            .field("pending", &self.len())
            .finish()
    }
}

impl AgentReports {
    pub fn new() -> AgentReports {
        AgentReports::default()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Tells the receiver of every report added. The channel is unbounded
    /// and a dropped receiver is forgotten.
    pub fn subscribe(&self) -> async_channel::Receiver<AgentReport> {
        let (sender, receiver) = async_channel::unbounded();
        self.lock().subscribers.push(sender);
        receiver
    }

    /// Puts `report` in the queue, unless it is full or the same report
    /// already waits.
    pub fn push(&self, report: AgentReport) -> Queued {
        let mut inner = self.lock();
        if inner.pending.contains(&report) {
            return Queued::Duplicate;
        }
        if inner.pending.len() >= MAX_PENDING {
            return Queued::Full;
        }
        inner.pending.push(report.clone());

        // Queue first, then notify while still holding the lock. A receiver
        // that opens the sheet can therefore always remove an already queued
        // report, and concurrent reports reach subscribers in queue order.
        let subscribers = &mut inner.subscribers;
        subscribers.retain(|subscriber| subscriber.try_send(report.clone()).is_ok());
        Queued::Added
    }

    /// How many reports wait.
    pub fn len(&self) -> usize {
        self.lock().pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The waiting reports, oldest first, without taking them.
    pub fn list(&self) -> Vec<AgentReport> {
        self.lock().pending.clone()
    }

    /// Takes report `report` out of the queue, once the person has seen it.
    pub fn remove(&self, report: &AgentReport) -> bool {
        let mut inner = self.lock();
        let before = inner.pending.len();
        inner.pending.retain(|waiting| waiting != report);
        inner.pending.len() != before
    }

    /// Takes every waiting report, oldest first.
    pub fn drain(&self) -> Vec<AgentReport> {
        std::mem::take(&mut self.lock().pending)
    }
}

/// Answers `kanso-report-issue`: validates the arguments, queues the report
/// and says what happens next. Reads no notes and sends nothing.
pub(crate) fn call(reports: &AgentReports, arguments: Option<&Value>) -> Value {
    use crate::mcp::tools::error_result;
    const NAME: &str = "kanso-report-issue";
    let Some(tool) = schema::tool(NAME) else {
        return error_result(format!("MCP error -32602: Tool {NAME} not found"));
    };
    let args = match tool.validate(arguments) {
        Ok(args) => args,
        Err(issues) => {
            return error_result(format!(
                "MCP error -32602: Input validation error: Invalid arguments for tool {NAME}: {issues}"
            ));
        }
    };
    let text = |key: &str| args.get(key).and_then(Value::as_str);
    let kind_name = text("kind");
    let kind = kind_name
        .and_then(ReportKind::parse)
        .unwrap_or(ReportKind::Bug);
    let summary = text("summary").unwrap_or_default();
    let tool_name = text("tool");
    let workaround = text("workaround");
    let report = match AgentReport::new(kind, summary, tool_name, workaround) {
        Ok(report) => report,
        Err(err) => return error_result(err.message()),
    };
    let message = match reports.push(report) {
        Queued::Added => {
            "Thanks. The report waits in Kanso for the person to review; nothing is sent until they press Send."
        }
        Queued::Duplicate => "The same report is already waiting for the person to review.",
        Queued::Full => {
            return error_result(
                "Too many reports are waiting for the person to review. Don't report this again; carry on.",
            );
        }
    };
    json!({ "content": [{ "type": "text", "text": message }] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_report_model::{MAX_TEXT, REPORTED_BY, clean};
    use proptest::prelude::*;

    fn report(summary: &str) -> AgentReport {
        AgentReport::new(ReportKind::Bug, summary, None, None).unwrap()
    }

    #[test]
    fn duplicates_are_dropped_and_the_cap_holds() {
        let reports = AgentReports::new();
        assert_eq!(
            reports.push(report("search-notes timed out")),
            Queued::Added
        );
        assert_eq!(
            reports.push(report("search-notes timed out")),
            Queued::Duplicate
        );
        for i in 1..MAX_PENDING {
            assert_eq!(
                reports.push(report(&format!("problem number {i}"))),
                Queued::Added
            );
        }
        assert_eq!(reports.push(report("one problem too many")), Queued::Full);
        assert_eq!(reports.len(), MAX_PENDING);
        assert_eq!(reports.drain().len(), MAX_PENDING);
        assert!(reports.is_empty());
    }

    #[test]
    fn subscribers_hear_of_added_reports_only() {
        let reports = AgentReports::new();
        let events = reports.subscribe();
        reports.push(report("read-note returned nothing"));
        reports.push(report("read-note returned nothing"));
        assert_eq!(
            events.try_recv().unwrap().summary,
            "read-note returned nothing"
        );
        assert!(events.try_recv().is_err());
        let first = reports.list().remove(0);
        assert!(reports.remove(&first));
        assert!(!reports.remove(&first));
    }

    #[test]
    fn the_text_starts_with_who_wrote_it() {
        let report = AgentReport::new(
            ReportKind::Question,
            "  list-notes ignored the bookId  ",
            Some("list-notes"),
            Some("filtered by hand"),
        )
        .unwrap();
        assert_eq!(
            report.text(),
            format!(
                "{REPORTED_BY}\nlist-notes ignored the bookId\n\nTool: list-notes\n\nWorkaround: filtered by hand"
            )
        );
    }

    #[test]
    fn short_long_and_control_text_is_refused() {
        let new = |s: &str| AgentReport::new(ReportKind::Bug, s, None, None);
        assert_eq!(new("too short"), Err(ReportError::TooShort));
        assert_eq!(new(&"x".repeat(MAX_TEXT)), Err(ReportError::TooLong));
        assert_eq!(
            new("a bell \u{7} rang here"),
            Err(ReportError::BadCharacter)
        );
    }

    use crate::agent_report_model::ReportError;

    fn summary() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-zA-Z0-9 .,\n]{0,60}",
            "[\\PC\n\r\t]{0,40}",
            ".{0,40}",
            "[e\u{301}\u{e9}\u{1F600} ]{5,30}",
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        // Invariant + idempotence: an accepted report's text is already as
        // clean as the feedback sheet's rules make it, and fits the sheet.
        #[test]
        fn prop_accepted_report_text_is_clean_and_fits(
            summary in summary(),
            tool in proptest::option::of("[a-z-]{0,20}"),
            workaround in proptest::option::of(summary()),
        ) {
            if let Ok(report) = AgentReport::new(
                ReportKind::Bug, &summary, tool.as_deref(), workaround.as_deref()
            ) {
                let text = report.text();
                prop_assert_eq!(clean(&text), Ok(text.clone()));
                prop_assert_eq!(clean(&report.summary), Ok(report.summary.clone()));
                let chars = text.chars().count();
                prop_assert!(report.summary.chars().count() >= 10);
                prop_assert!(chars <= MAX_TEXT);
                prop_assert!(text.starts_with(REPORTED_BY));
            }
        }

        // Model-based: any run of pushes leaves the queue holding the
        // distinct reports, in order, never more than the cap.
        #[test]
        fn prop_queue_matches_a_model(picks in proptest::collection::vec(0usize..40, 0..80)) {
            let reports = AgentReports::new();
            let mut model: Vec<AgentReport> = Vec::new();
            for pick in picks {
                let next = report(&format!("problem number {pick}"));
                let expected = if model.contains(&next) {
                    Queued::Duplicate
                } else if model.len() >= MAX_PENDING {
                    Queued::Full
                } else {
                    model.push(next.clone());
                    Queued::Added
                };
                prop_assert_eq!(reports.push(next), expected);
                prop_assert!(reports.len() <= MAX_PENDING);
            }
            prop_assert_eq!(reports.list(), model);
        }
    }
}
