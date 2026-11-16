//! The values of an agent's problem report (issue #447): what an agent
//! says went wrong, and the text the person is shown before anything is
//! sent.

use unicode_normalization::UnicodeNormalization;

/// The shortest summary, in characters (the feedback sheet's own limit).
pub const MIN_SUMMARY: usize = 10;
/// The longest report text, in characters (the feedback sheet's own limit).
pub const MAX_TEXT: usize = 4000;
/// What every prefilled report starts with, so triage can tell who wrote it.
pub const REPORTED_BY: &str = "Reported by an AI agent:";

/// What kind of report it is; the same three the feedback sheet offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReportKind {
    Bug,
    Idea,
    Question,
}

impl ReportKind {
    /// The kind named `name`, as the tool's `kind` argument spells it.
    pub fn parse(name: &str) -> Option<ReportKind> {
        match name {
            "bug" => Some(ReportKind::Bug),
            "idea" => Some(ReportKind::Idea),
            "question" => Some(ReportKind::Question),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ReportKind::Bug => "bug",
            ReportKind::Idea => "idea",
            ReportKind::Question => "question",
        }
    }
}

/// Why a report was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportError {
    TooShort,
    TooLong,
    BadCharacter,
}

impl ReportError {
    pub fn message(self) -> &'static str {
        match self {
            ReportError::TooShort => "The summary must be at least 10 characters.",
            ReportError::TooLong => {
                "The report is longer than 4,000 characters with its tool and workaround. Shorten it."
            }
            ReportError::BadCharacter => {
                "The report may not contain control characters other than line feeds."
            }
        }
    }
}

/// One problem an agent reported. Never sent as is: the person sees
/// [`AgentReport::text`] in the feedback sheet and presses Send.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AgentReport {
    pub kind: ReportKind,
    pub summary: String,
    /// The MCP tool involved, when the agent named one.
    pub tool: Option<String>,
    /// How the agent worked around the problem, when it did.
    pub workaround: Option<String>,
}

/// Text as the feedback sheet would send it: NFC, and no control
/// characters but line feeds (a CRLF becomes a line feed).
pub fn clean(input: &str) -> Result<String, ReportError> {
    let text: String = input.replace("\r\n", "\n").nfc().collect();
    if text.chars().any(|c| c != '\n' && c.is_control()) {
        return Err(ReportError::BadCharacter);
    }
    Ok(text.trim().to_owned())
}

impl AgentReport {
    /// A report from the tool's arguments, cleaned, or why it can't be one.
    /// Its [`text`](AgentReport::text) always fits the feedback sheet.
    pub fn new(
        kind: ReportKind,
        summary: &str,
        tool: Option<&str>,
        workaround: Option<&str>,
    ) -> Result<AgentReport, ReportError> {
        fn clean_optional(value: Option<&str>) -> Result<Option<String>, ReportError> {
            let Some(value) = value else {
                return Ok(None);
            };

            let value = clean(value)?;
            let value = (!value.is_empty()).then_some(value);
            Ok(value)
        }

        let summary = clean(summary)?;
        if summary.chars().count() < MIN_SUMMARY {
            return Err(ReportError::TooShort);
        }

        let tool = clean_optional(tool)?;
        let workaround = clean_optional(workaround)?;
        let report = AgentReport {
            kind,
            summary,
            tool,
            workaround,
        };
        let report_text_length = report.text().chars().count();
        if report_text_length > MAX_TEXT {
            return Err(ReportError::TooLong);
        }
        Ok(report)
    }

    /// The text the feedback sheet is filled with.
    pub fn text(&self) -> String {
        let mut text = format!("{REPORTED_BY}\n{}", self.summary);
        if let Some(tool) = &self.tool {
            text.push_str("\n\nTool: ");
            text.push_str(tool);
        }
        if let Some(workaround) = &self.workaround {
            text.push_str("\n\nWorkaround: ");
            text.push_str(workaround);
        }
        text
    }
}
