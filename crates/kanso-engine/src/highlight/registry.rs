//! The grammars the highlighter parses with, and their queries, each
//! compiled the first time it is used (a query takes tens of milliseconds
//! for the large grammars).
//!
//! This is the table `LanguageRegistry` is for the desktop's library:
//! the same grammar crates, the same queries, the Markdown ones patched
//! (see [`super::queries`]).

use std::sync::OnceLock;

use tree_sitter::{Language, Query};

use super::Token;
use super::queries;

/// A grammar with the queries that color it.
pub(super) struct Grammar {
    /// The name [`super::grammar`] gives, or `markdown_inline`.
    pub name: &'static str,
    pub language: Language,
    highlights: String,
    injections: String,
    locals: String,
    highlight_query: OnceLock<Option<Highlights>>,
    main_query: OnceLock<Option<Highlights>>,
    injection_query: OnceLock<Option<Injections>>,
}

/// A compiled highlight query with the token of each capture.
pub(super) struct Highlights {
    pub query: Query,
    /// `tokens[capture index]`: `None` for a capture no theme key colors.
    pub tokens: Vec<Option<Token>>,
}

/// A compiled injection query with its two special captures.
pub(super) struct Injections {
    pub query: Query,
    pub content: Option<u32>,
    pub language: Option<u32>,
}

impl Grammar {
    fn new(
        name: &'static str,
        language: Language,
        highlights: impl Into<String>,
        injections: impl Into<String>,
        locals: impl Into<String>,
    ) -> Self {
        Grammar {
            name,
            language,
            highlights: highlights.into(),
            injections: injections.into(),
            locals: locals.into(),
            highlight_query: OnceLock::new(),
            main_query: OnceLock::new(),
            injection_query: OnceLock::new(),
        }
    }

    /// The highlight query; `None` when it doesn't compile for the grammar
    /// (a test checks every one does).
    pub fn highlights(&self) -> Option<&Highlights> {
        self.highlight_query
            .get_or_init(|| {
                let query = Query::new(&self.language, &self.highlights).ok()?;
                let tokens = query
                    .capture_names()
                    .iter()
                    .map(|name| Token::from_capture(name))
                    .collect();
                Some(Highlights { query, tokens })
            })
            .as_ref()
    }

    /// The query the component library uses for the language itself: its
    /// locals query, then its highlights, as one query. Locals captures keep
    /// their place in the capture order, which is what makes a declaration
    /// keep the name of the capture already over it.
    pub fn main_highlights(&self) -> Option<&Highlights> {
        if self.locals.is_empty() {
            return self.highlights();
        }
        self.main_query
            .get_or_init(|| {
                let mut source = String::with_capacity(self.locals.len() + self.highlights.len());
                source.push_str(&self.locals);
                source.push_str(&self.highlights);
                let query = Query::new(&self.language, &source).ok()?;
                let tokens = query
                    .capture_names()
                    .iter()
                    .map(|name| Token::from_capture(name))
                    .collect();
                Some(Highlights { query, tokens })
            })
            .as_ref()
    }

    /// The injection query, for the grammars that have one.
    pub fn injections(&self) -> Option<&Injections> {
        if self.injections.is_empty() {
            return None;
        }
        self.injection_query
            .get_or_init(|| {
                let query = Query::new(&self.language, &self.injections).ok()?;
                let index = |wanted: &str| {
                    query
                        .capture_names()
                        .iter()
                        .position(|name| *name == wanted)
                        .map(|i| i as u32)
                };
                Some(Injections {
                    content: index("injection.content"),
                    language: index("injection.language"),
                    query,
                })
            })
            .as_ref()
    }
}

/// The grammar named `name` (one of [`super::GRAMMARS`]' own names, or
/// `markdown_inline`), when it is compiled in.
pub(super) fn get(name: &str) -> Option<&'static Grammar> {
    all().iter().find(|grammar| grammar.name == name)
}

/// Every grammar compiled in.
pub(super) fn all() -> &'static [Grammar] {
    static ALL: OnceLock<Vec<Grammar>> = OnceLock::new();
    ALL.get_or_init(build)
}

fn build() -> Vec<Grammar> {
    let markdown = queries::in_use();
    let all = vec![
        Grammar::new(
            "markdown",
            tree_sitter_md::LANGUAGE.into(),
            markdown.highlights.as_str(),
            markdown.injections.as_str(),
            "",
        ),
        Grammar::new(
            "markdown_inline",
            tree_sitter_md::INLINE_LANGUAGE.into(),
            markdown.inline_highlights.as_str(),
            "",
            "",
        ),
    ];
    #[cfg(feature = "code-grammars")]
    let all = {
        let mut all = all;
        all.extend(code_grammars());
        all
    };
    all
}

#[cfg(feature = "code-grammars")]
fn code_grammars() -> Vec<Grammar> {
    // Where the library uses a grammar crate's own query, so does this; the
    // others are its own, copied into `queries/`. Swift and C# ship their
    // queries in their crates, which the library compiles in without them
    // (the desktop registers them in `code_lang::ensure_registered`).
    fn plain(name: &'static str, language: impl Into<Language>, highlights: &str) -> Grammar {
        Grammar::new(name, language.into(), highlights, "", "")
    }

    fn with_locals(
        name: &'static str,
        language: impl Into<Language>,
        highlights: &str,
        locals: &str,
    ) -> Grammar {
        Grammar::new(name, language.into(), highlights, "", locals)
    }
    vec![
        plain(
            "bash",
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY,
        ),
        plain("c", tree_sitter_c::LANGUAGE, tree_sitter_c::HIGHLIGHT_QUERY),
        plain(
            "cpp",
            tree_sitter_cpp::LANGUAGE,
            tree_sitter_cpp::HIGHLIGHT_QUERY,
        ),
        plain(
            "csharp",
            tree_sitter_c_sharp::LANGUAGE,
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
        ),
        plain(
            "css",
            tree_sitter_css::LANGUAGE,
            tree_sitter_css::HIGHLIGHTS_QUERY,
        ),
        plain(
            "diff",
            tree_sitter_diff::LANGUAGE,
            tree_sitter_diff::HIGHLIGHTS_QUERY,
        ),
        plain(
            "go",
            tree_sitter_go::LANGUAGE,
            include_str!("queries/go.scm"),
        ),
        plain(
            "html",
            tree_sitter_html::LANGUAGE,
            include_str!("queries/html.scm"),
        ),
        plain(
            "java",
            tree_sitter_java::LANGUAGE,
            tree_sitter_java::HIGHLIGHTS_QUERY,
        ),
        with_locals(
            "javascript",
            tree_sitter_javascript::LANGUAGE,
            include_str!("queries/javascript.scm"),
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        plain(
            "json",
            tree_sitter_json::LANGUAGE,
            include_str!("queries/json.scm"),
        ),
        plain(
            "kotlin",
            tree_sitter_kotlin_sg::LANGUAGE,
            include_str!("queries/kotlin.scm"),
        ),
        with_locals(
            "lua",
            tree_sitter_lua::LANGUAGE,
            include_str!("queries/lua.scm"),
            tree_sitter_lua::LOCALS_QUERY,
        ),
        plain(
            "php",
            tree_sitter_php::LANGUAGE_PHP,
            tree_sitter_php::HIGHLIGHTS_QUERY,
        ),
        plain(
            "python",
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY,
        ),
        with_locals(
            "ruby",
            tree_sitter_ruby::LANGUAGE,
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            tree_sitter_ruby::LOCALS_QUERY,
        ),
        plain(
            "rust",
            tree_sitter_rust::LANGUAGE,
            include_str!("queries/rust.scm"),
        ),
        plain(
            "sql",
            tree_sitter_sequel::LANGUAGE,
            tree_sitter_sequel::HIGHLIGHTS_QUERY,
        ),
        plain(
            "swift",
            tree_sitter_swift::LANGUAGE,
            tree_sitter_swift::HIGHLIGHTS_QUERY,
        ),
        plain(
            "toml",
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
        ),
        with_locals(
            "tsx",
            tree_sitter_typescript::LANGUAGE_TSX,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            tree_sitter_typescript::LOCALS_QUERY,
        ),
        with_locals(
            "typescript",
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            include_str!("queries/typescript.scm"),
            tree_sitter_typescript::LOCALS_QUERY,
        ),
        plain(
            "yaml",
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
        ),
    ]
}
