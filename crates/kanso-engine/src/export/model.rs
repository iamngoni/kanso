//! What an export reports and the type a page is set in: plain data, with
//! the logic in the modules beside it.

use std::collections::BTreeMap;

use crate::theme::file::TokenStyle;

/// What a Markdown export wrote.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ExportReport {
    pub notes: usize,
    pub attachments: usize,
    /// Locked notes left out: their text is only sealed, and an export is
    /// plain text.
    pub locked: usize,
}

/// The styles a page's code blocks are drawn in: a syntax theme's, fitted to
/// the code background (`Tokens::code`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeColors {
    /// Whether the colors are made for a dark background.
    pub dark: bool,
    /// The style of each token, by its theme key (`Token::key`).
    pub syntax: BTreeMap<&'static str, TokenStyle>,
}

/// The faces, size and code colors an exported page sets text with: the
/// same ones the preview uses, so a page and the preview look alike.
#[derive(Clone, Debug, PartialEq)]
pub struct PageStyle {
    /// The body face, when the settings or the theme name one.
    pub font: Option<String>,
    /// The monospaced face, when it isn't the system's.
    pub mono: Option<String>,
    /// The body text size, in points.
    pub size: f32,
    /// The colors the preview's code blocks are drawn in; without them code
    /// stays plain.
    pub code: Option<CodeColors>,
}
