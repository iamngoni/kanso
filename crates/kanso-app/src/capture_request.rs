//! The `capture` message of the local link (`url_callback.rs`): a note
//! `kanso capture` asks the running app to add, as one line of JSON,
//! `{"kind":"capture","title":…,"body":…,"book":…}`.
//!
//! This is the whole of what the link lets a program do with notes: add one.
//! There is no message that reads, searches or lists, so a program that
//! finds the endpoint file learns nothing from Kanso. Anything with another
//! `kind`, another field, or a size over the limits below is refused with a
//! plain message that says what to do.

use serde::Deserialize;
use serde_json::json;

/// The `kind` of a capture message. No other kind is known.
pub const KIND: &str = "capture";

/// The longest title a capture may carry, in bytes.
pub const MAX_TITLE: usize = 1024;

/// The longest body a capture may carry, in bytes. Half of what a note
/// holds, so the escapes JSON adds never push a note that fits over the
/// line limit.
pub const MAX_BODY: usize = 512 * 1024;

/// The longest notebook name or ID a capture may carry, in bytes.
pub const MAX_BOOK: usize = 1024;

/// The longest message line: every byte of the three fields escaped at its
/// longest (`\u00XX`, six bytes), plus the keys.
pub const MAX_LINE: usize = (MAX_TITLE + MAX_BODY + MAX_BOOK) * 6 + 256;

/// A note `kanso capture` asks the running app to add.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureRequest {
    pub title: String,
    pub body: String,
    /// The notebook's name or ID; the Inbox when `None`.
    pub book: Option<String>,
}

/// What the line holds, with nothing else allowed.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    #[allow(dead_code)]
    kind: String,
    title: String,
    body: String,
    #[serde(default)]
    book: Option<String>,
}

impl CaptureRequest {
    /// Whether the request is within the limits and has a title.
    pub fn check(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err(
                "There's nothing to add: the note has no title. Give `kanso capture` some text."
                    .into(),
            );
        }
        if self.title.len() > MAX_TITLE {
            return Err(format!(
                "That title is longer than {MAX_TITLE} bytes. Use a shorter first line."
            ));
        }
        if self.body.len() > MAX_BODY {
            return Err(format!(
                "That note is more than {} KB, more than Kanso takes from outside. Split it into smaller notes, or quit Kanso and capture again.",
                MAX_BODY / 1024
            ));
        }
        if self.book.as_ref().is_some_and(|book| book.len() > MAX_BOOK) {
            return Err(format!(
                "That notebook name is longer than {MAX_BOOK} bytes. Use the notebook's ID or a shorter name."
            ));
        }
        Ok(())
    }

    /// The message line, without its line break. `Err` for a request over
    /// the limits.
    pub fn to_line(&self) -> Result<String, String> {
        self.check()?;
        Ok(json!({
            "kind": KIND,
            "title": self.title,
            "body": self.body,
            "book": self.book,
        })
        .to_string())
    }

    /// The request a message line holds. `Err` says why it was refused, in
    /// words that go back to whoever sent it: another kind (the link only
    /// adds notes), a field that isn't known, bad JSON, or a size over a
    /// limit.
    pub fn from_line(line: &str) -> Result<CaptureRequest, String> {
        if line.len() > MAX_LINE {
            return Err("That message is too large for Kanso. Send a smaller note.".into());
        }
        let value: serde_json::Value = serde_json::from_str(line).map_err(|_| {
            "Kanso couldn't read that message. `kanso capture` sends it; update Kanso if you see this from it."
                .to_owned()
        })?;
        if value.get("kind").and_then(|kind| kind.as_str()) != Some(KIND) {
            return Err(
                "Kanso's local link only adds notes. Use `kanso capture` to add one; to read notes, create an API key in Settings › Local API."
                    .into(),
            );
        }
        let wire: Wire = serde_json::from_value(value).map_err(|_| {
            "Kanso couldn't read that note. A capture has a title, a body and, optionally, a book."
                .to_owned()
        })?;
        let request = CaptureRequest {
            title: wire.title,
            body: wire.body,
            book: wire.book.filter(|book| !book.trim().is_empty()),
        };
        request.check()?;
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn text() -> impl Strategy<Value = String> {
        // Everything a note can hold, controls and non-ASCII included.
        prop::collection::vec(any::<char>(), 0..200).prop_map(|c| c.into_iter().collect())
    }

    proptest! {
        /// Title, body and book come back exactly as they went.
        #[test]
        fn prop_a_capture_round_trips(
            title in text().prop_filter("a title", |t| !t.trim().is_empty()),
            body in text(),
            book in prop::option::of(text().prop_filter("a book", |b| !b.trim().is_empty())),
        ) {
            let request = CaptureRequest { title, body, book };
            let line = request.to_line().unwrap();
            prop_assert!(!line.contains('\n'));
            prop_assert_eq!(CaptureRequest::from_line(&line), Ok(request));
        }

        /// Any kind but `capture` is refused, whatever else is in the
        /// message.
        #[test]
        fn prop_no_other_kind_is_taken(kind in "[a-z-]{0,16}", title in text(), query in text()) {
            prop_assume!(kind != KIND);
            let line = json!({"kind": kind, "title": title, "body": "", "query": query}).to_string();
            prop_assert!(CaptureRequest::from_line(&line).is_err());
            let line = json!({"kind": kind}).to_string();
            prop_assert!(CaptureRequest::from_line(&line).is_err());
        }

        /// Arbitrary bytes that aren't a capture are refused, never taken.
        #[test]
        fn prop_garbage_is_refused(line in "\\PC{0,300}") {
            if let Ok(request) = CaptureRequest::from_line(&line) {
                // Only a real capture gets through, and it is within limits.
                prop_assert!(request.check().is_ok());
            }
        }

        /// Fields beyond the three are refused.
        #[test]
        fn prop_unknown_fields_are_refused(field in "[a-z]{1,12}") {
            prop_assume!(!["kind", "title", "body", "book"].contains(&field.as_str()));
            let line = json!({"kind": KIND, "title": "t", "body": "", field: "x"}).to_string();
            prop_assert!(CaptureRequest::from_line(&line).is_err());
        }
    }

    #[test]
    fn oversized_and_malformed_messages_are_refused() {
        let ok = |title: &str, body: &str, book: Option<&str>| CaptureRequest {
            title: title.into(),
            body: body.into(),
            book: book.map(str::to_owned),
        };
        assert!(ok("t", &"x".repeat(MAX_BODY), None).to_line().is_ok());
        assert!(ok("t", &"x".repeat(MAX_BODY + 1), None).to_line().is_err());
        assert!(ok(&"t".repeat(MAX_TITLE + 1), "", None).to_line().is_err());
        assert!(
            ok("t", "", Some(&"b".repeat(MAX_BOOK + 1)))
                .to_line()
                .is_err()
        );
        assert!(ok("  ", "body", None).to_line().is_err());
        // The same limits hold on the receiving side, whatever the sender
        // checked.
        let big = json!({"kind": KIND, "title": "t", "body": "x".repeat(MAX_BODY + 1)});
        assert!(CaptureRequest::from_line(&big.to_string()).is_err());
        let long_line = " ".repeat(MAX_LINE + 1);
        assert!(CaptureRequest::from_line(&long_line).is_err());
        for bad in [
            "",
            "{",
            "[]",
            "null",
            "\"capture\"",
            r#"{"kind":"capture"}"#,
            r#"{"kind":"capture","title":5,"body":""}"#,
            r#"{"kind":"capture","title":"t","body":"","book":5}"#,
            r#"{"kind":"capture","title":"t"}"#,
            r#"{"title":"t","body":""}"#,
        ] {
            assert!(CaptureRequest::from_line(bad).is_err(), "{bad}");
        }
        // Every refusal says what to do.
        let said = CaptureRequest::from_line("{").unwrap_err();
        assert!(said.contains("kanso capture"), "{said}");
    }
}
