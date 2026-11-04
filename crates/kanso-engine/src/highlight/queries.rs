//! How Kanso reads the Markdown queries of the component library the
//! desktop editor draws with: both apps patch them the same way.
//!
//! The library parses the inline Markdown of a note (emphasis, code spans,
//! links) as one combined document, putting back the text between the
//! paragraphs it keeps. A fenced code block between two such paragraphs then
//! reads as an inline code span, so its opening fence is drawn in the code
//! color, and a stray backtick in one list item pairs up with one in the
//! next. Kanso parses each paragraph on its own instead, as Markdown does.
//!
//! The library's queries also leave some markers uncaptured, so they took
//! the text color while every other marker receded: table pipes and
//! delimiter rows, task boxes, and the brackets around links and images.
//! Link reference definitions used capture names no theme styles. Kanso adds
//! the missing captures.
//!
//! Themes can color quotes, rules, task boxes, front matter and HTML blocks
//! apart from other markers, so each gets a capture of its own. Highlight
//! names are a fixed list, so they borrow names no bundled code grammar uses
//! ([`crate::theme::file::CAPTURES`] maps them to the theme's keys).
//!
//! The functions take the library's own query text, because the desktop
//! patches the queries it registered and the engine patches its copies
//! (`highlight/queries/`); a test in `kanso-app` checks the two agree.

/// Block markers the library's Markdown query leaves in the text color.
/// Node names are tree-sitter-md 0.5.3's; a misspelled one fails the whole
/// query, which the tests would catch as missing colors.
pub const BLOCK_MARKERS: &str = r#"
(pipe_table_header "|" @punctuation.delimiter)
(pipe_table_row "|" @punctuation.delimiter)
(pipe_table_delimiter_row "|" @punctuation.delimiter)
(pipe_table_delimiter_cell) @punctuation.delimiter
[(task_list_marker_checked) (task_list_marker_unchecked)] @enum
(link_reference_definition ":" @punctuation.delimiter)
(link_label ["[" "]"] @punctuation.delimiter)
(thematic_break) @variant
(block_quote) @primary
(minus_metadata) @preproc
(html_block) @tag.doctype
"#;

/// The brackets around links and images, which the inline query leaves in
/// the text color between the link text and its muted destination.
pub const INLINE_MARKERS: &str = r#"
(inline_link ["[" "]" "(" ")"] @punctuation.delimiter)
(image ["!" "[" "]" "(" ")"] @punctuation.delimiter)
(full_reference_link ["[" "]"] @punctuation.delimiter)
(collapsed_reference_link ["[" "]"] @punctuation.delimiter)
(shortcut_link ["[" "]"] @punctuation.delimiter)
"#;

/// The Markdown highlight query, with link destinations and references named
/// the way the inline query names a link's, rules left out of the list
/// markers' group (they get a capture of their own), and the markers added.
// no-prop-test: fixed edits to the library's queries; the tests check what
// they highlight.
pub fn markdown_highlights(base: &str) -> String {
    let highlights = base
        .replace("@text.uri", "@link_uri")
        .replace("@text.reference", "@link_text")
        .replace("  (thematic_break)\n", "");
    format!("{highlights}\n{BLOCK_MARKERS}")
}

/// The Markdown injection query without the library's `injection.combined`
/// directive, so each paragraph's inline text is parsed on its own, and with
/// its YAML injection for a rule, a block and a rule anchored to siblings
/// that touch: unanchored, any block after a leading rule and before another
/// was read as YAML.
// no-prop-test: fixed edit to the library's query; the tests check what it
// highlights.
pub fn markdown_injections(base: &str) -> String {
    base.replace("(#set! injection.combined)", "").replace(
        "(thematic_break) (_) @injection.content (thematic_break)",
        "(thematic_break) . (_) @injection.content . (thematic_break)",
    )
}

/// The inline Markdown highlight query with the link brackets added.
// no-prop-test: fixed edit to the library's query; the tests check what it
// highlights.
pub fn markdown_inline_highlights(base: &str) -> String {
    format!("{base}\n{INLINE_MARKERS}")
}

/// The Markdown queries the engine highlights with.
#[derive(Debug)]
pub struct InUse {
    pub highlights: String,
    pub injections: String,
    pub inline_highlights: String,
}

/// The library's Markdown queries (copied into `highlight/queries/`) with the
/// patches above made: what the engine's highlighter reads, and what the
/// desktop's registry must hold after its own patching.
pub fn in_use() -> &'static InUse {
    static IN_USE: std::sync::OnceLock<InUse> = std::sync::OnceLock::new();
    IN_USE.get_or_init(|| InUse {
        highlights: markdown_highlights(include_str!("queries/markdown.scm")),
        injections: markdown_injections(include_str!("queries/markdown_injections.scm")),
        inline_highlights: markdown_inline_highlights(include_str!("queries/markdown_inline.scm")),
    })
}
