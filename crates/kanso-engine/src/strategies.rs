//! Generators for the account's and sync's states, shared by the tests of
//! the views that show them (the desktop app's, and the mobile app's). Only
//! built for tests and with the `test-support` feature.

use kanso_sync::{QuotaKind, SignOutReason, Step, SyncProblem};
use proptest::prelude::*;

use crate::sync::{AccountState, SyncState};

pub fn problem() -> impl Strategy<Value = SyncProblem> {
    prop_oneof![
        prop::sample::select(SignOutReason::ALL.to_vec()).prop_map(SyncProblem::SignedOut),
        Just(SyncProblem::AccountDisabled),
        (0i64..1_000_000).prop_map(|until| SyncProblem::Paused { until }),
        (0i64..1_000_000).prop_map(|until| SyncProblem::ServerBusy { until }),
        (0i64..1_000_000).prop_map(|until| SyncProblem::BackOff { until }),
        prop::sample::select(QuotaKind::ALL.to_vec()).prop_map(SyncProblem::QuotaExceeded),
        Just(SyncProblem::ServerFull),
        (1usize..100).prop_map(|count| SyncProblem::Skipped { count }),
        Just(SyncProblem::UpdateKanso),
        Just(SyncProblem::WrongKey),
        Just(SyncProblem::ServerOutdated),
        Just(SyncProblem::Offline),
        Just(SyncProblem::Other),
    ]
}

pub fn step() -> impl Strategy<Value = Step> {
    prop::sample::select(vec![
        Step::StoppingSync,
        Step::DerivingKeys,
        Step::Contacting,
        Step::Saving,
    ])
}

pub fn account() -> impl Strategy<Value = AccountState> {
    prop_oneof![
        Just(AccountState::SignedOut),
        "[a-z]{3,12}".prop_map(|username| AccountState::SignedIn { username }),
        "[a-z]{3,12}".prop_map(|username| AccountState::SignedInLocked { username }),
        (
            "[a-z]{3,12}",
            prop::sample::select(SignOutReason::ALL.to_vec())
        )
            .prop_map(|(username, reason)| AccountState::SessionEnded { username, reason }),
        step().prop_map(AccountState::Busy),
    ]
}

pub fn sync_state() -> impl Strategy<Value = SyncState> {
    prop_oneof![
        Just(SyncState::Off),
        Just(SyncState::Syncing),
        (0i64..1_000_000_000, 0usize..4, prop::option::of(problem())).prop_map(
            |(at_ms, conflicts, problem)| SyncState::Synced {
                at_ms,
                conflicts,
                conflicted: Vec::new(),
                problem,
            }
        ),
        problem().prop_map(SyncState::Failed),
    ]
}

/// Markdown notes for the highlighter's tests (the engine's and the desktop
/// app's parity test).
#[cfg(feature = "highlight")]
pub mod markdown {
    use proptest::prelude::*;

    /// The grammars a fence may name, by their own names.
    pub const LANGUAGES: [&str; 24] = [
        "bash",
        "c",
        "cpp",
        "csharp",
        "css",
        "diff",
        "go",
        "html",
        "java",
        "javascript",
        "json",
        "kotlin",
        "lua",
        "markdown",
        "php",
        "python",
        "ruby",
        "rust",
        "sql",
        "swift",
        "toml",
        "tsx",
        "typescript",
        "yaml",
    ];

    /// Other spellings and names no grammar has.
    const OTHER_INFO: [&str; 10] = [
        "zsh",
        "rust,ignore",
        "JS",
        "c++",
        "yml",
        "{.python}",
        "text",
        "dart",
        "py3 {1,3}",
        "svg",
    ];

    /// What a generated note may hold.
    #[derive(Clone, Copy)]
    struct Domain {
        /// Fences name grammars by other spellings, and by none.
        aliases: bool,
        /// Code blocks hold complete, valid code, unindented: what any two
        /// tree-sitter parses of it agree on. Broken code is parsed
        /// differently depending on where in a text it sits.
        valid: bool,
    }

    fn word() -> impl Strategy<Value = String> {
        prop_oneof![
            8 => "[a-z]{1,7}",
            1 => Just("é".to_owned()),
            1 => Just("日本語".to_owned()),
            1 => Just("🙂".to_owned()),
            1 => "[0-9]{1,3}",
        ]
    }

    /// Inline text: words with the constructs inline Markdown has.
    fn inline() -> impl Strategy<Value = String> {
        let piece = prop_oneof![
            10 => word(),
            2 => word().prop_map(|w| format!("*{w}*")),
            2 => word().prop_map(|w| format!("**{w}**")),
            1 => word().prop_map(|w| format!("_{w}_")),
            2 => word().prop_map(|w| format!("`{w}`")),
            2 => (word(), word()).prop_map(|(a, b)| format!("[{a}]({b})")),
            1 => (word(), word()).prop_map(|(a, b)| format!("![{a}]({b})")),
            1 => word().prop_map(|w| format!("[{w}][r]")),
            1 => word().prop_map(|w| format!("[{w}]")),
            1 => Just("<https://a.b>".to_owned()),
            1 => Just("\\*".to_owned()),
            1 => Just("~~x~~".to_owned()),
            1 => Just("`".to_owned()),
            1 => Just("*".to_owned()),
            1 => Just("[[wiki]]".to_owned()),
            1 => Just("#tag".to_owned()),
        ];
        prop::collection::vec(piece, 1..6).prop_map(|pieces| pieces.join(" "))
    }

    /// Complete, valid code in each language, as written in a note.
    pub fn valid_code(language: &str) -> &'static [&'static str] {
        match language {
            "rust" => &[
                "fn main() {\n    let x = \"a\";\n}\n",
                "// comment\nstruct A { b: u32 }\n",
            ],
            "python" => &["def f(x):\n    return x + 1\n", "# comment\nx = 'a'\n"],
            "javascript" => &["const a = `b`;\nfunction f() { return 1; }\n"],
            "typescript" => &["let a: number = 1;\ninterface A { b: string }\n"],
            "tsx" => &["const a = <div>{b}</div>;\n"],
            "json" => &["{\"a\": [1, true, null]}\n"],
            "bash" => &["echo \"$A\" | grep -v x\nif [ -f a ]; then ls; fi\n"],
            "html" => &["<div class=\"a\">b</div>\n", "<!-- c -->\n"],
            "yaml" => &["a: 1\nb:\n  - c\n"],
            "toml" => &["[a]\nb = \"c\"\n"],
            "markdown" => &["# h\n\n*a* `b`\n"],
            "sql" => &["select a from b where c = 1;\n"],
            "c" => &["int main() { return 0; }\n"],
            "cpp" => &["class A { int b; };\n"],
            "go" => &["package main\n\nfunc main() { x := 1 }\n"],
            "java" => &["class A { int b = 1; }\n"],
            "css" => &["a { color: red; }\n"],
            "diff" => &["@@ -1 +1 @@\n-a\n+b\n"],
            "kotlin" => &["fun main() { val a = 1 }\n"],
            "lua" => &["local a = function() end\n"],
            "php" => &["<?php\nfunction f() { return 1; }\n"],
            "ruby" => &["def f\n  1\nend\n"],
            "swift" => &["struct A { let b = \"x\" }\n"],
            "csharp" => &["public class A { string B = \"x\"; }\n"],
            _ => &["a b c\n"],
        }
    }

    fn code_line(language: &str) -> BoxedStrategy<String> {
        let lines: &[&str] = match language {
            "rust" => &[
                "fn main() { let x = \"a\"; }",
                "// comment",
                "struct A { b: u32 }",
                "",
            ],
            "python" => &["def f(x):", "    return 'a' + x", "# c", ""],
            "javascript" | "typescript" | "tsx" => {
                &["const a = `b`;", "function f() { return 1 }", "// c", ""]
            }
            "json" => &["{\"a\": [1, true, null]}", ""],
            "bash" => &["echo \"$A\" | grep -v x", "# c", "if [ -f a ]; then ls; fi"],
            "html" => &["<div class=\"a\">b</div>", "<!-- c -->", ""],
            "yaml" => &["a: 1", "b:", "  - c", "# d"],
            "toml" => &["[a]", "b = \"c\"", ""],
            "markdown" => &["# h", "*a* `b`", "- [ ] c", ""],
            "sql" => &["select a from b where c = 1;", "-- d"],
            _ => &["a b c", "int x = 1;", "// c", "\"s\"", ""],
        };
        proptest::sample::select(lines.to_vec())
            .prop_map(str::to_owned)
            .boxed()
    }

    fn info(domain: Domain) -> BoxedStrategy<String> {
        let canonical = proptest::sample::select(LANGUAGES.to_vec()).prop_map(str::to_owned);
        if domain.aliases {
            prop_oneof![
                6 => canonical,
                2 => proptest::sample::select(OTHER_INFO.to_vec()).prop_map(str::to_owned),
                2 => Just(String::new()),
            ]
            .boxed()
        } else {
            prop_oneof![
                6 => canonical,
                1 => Just("text".to_owned()),
                2 => Just(String::new()),
            ]
            .boxed()
        }
    }

    /// A fenced code block, closed or not, perhaps indented as in a list
    /// item, perhaps with a blank line inside.
    fn fence(domain: Domain) -> impl Strategy<Value = String> {
        (
            info(domain),
            any::<bool>(),
            if domain.valid { 0usize..1 } else { 0usize..3 },
            prop::bool::weighted(if domain.valid { 1.0 } else { 0.85 }),
            3usize..5,
            1usize..4,
            prop::bool::weighted(0.3),
            any::<prop::sample::Index>(),
        )
            .prop_flat_map(
                move |(info, tilde, indent, closed, len, count, gap, pick)| {
                    let language = info
                        .split(|c: char| !c.is_alphanumeric())
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    let snippet = {
                        let snippets = valid_code(&language);
                        snippets[pick.index(snippets.len())]
                    };
                    let lines = if domain.valid {
                        Just(snippet.lines().map(str::to_owned).collect::<Vec<_>>()).boxed()
                    } else {
                        prop::collection::vec(code_line(&language), count).boxed()
                    };
                    lines.prop_map(move |mut lines| {
                        if gap && !domain.valid {
                            lines.insert(lines.len() / 2, String::new());
                        }
                        let mark = if tilde { "~" } else { "`" }.repeat(len);
                        let pad = " ".repeat(indent);
                        let mut out = format!("{pad}{mark}{info}\n");
                        for line in lines {
                            out.push_str(&format!("{pad}{line}\n"));
                        }
                        if closed {
                            out.push_str(&format!("{pad}{mark}\n"));
                        }
                        out
                    })
                },
            )
    }

    fn list(domain: Domain, with_fence: bool) -> impl Strategy<Value = String> {
        let marker = prop_oneof![
            Just("- ".to_owned()),
            Just("* ".to_owned()),
            Just("1. ".to_owned()),
            Just("- [ ] ".to_owned()),
            Just("- [x] ".to_owned()),
        ];
        let item = (marker, inline(), 0usize..3, prop::bool::weighted(0.25));
        (
            prop::collection::vec(item, 1..5),
            prop::bool::weighted(0.4),
            prop::option::of(fence(domain)),
        )
            .prop_map(move |(items, loose, nested_fence)| {
                let nested_fence = nested_fence.filter(|_| with_fence && !domain.valid);
                let gap = if loose { "\n\n" } else { "\n" };
                let mut out = String::new();
                for (marker, text, depth, continued) in items {
                    out.push_str(&"  ".repeat(depth));
                    out.push_str(&format!("{marker}{text}{gap}"));
                    if continued {
                        out.push_str(&format!("  more {text}\n\n"));
                    }
                }
                if let Some(fence) = nested_fence {
                    out.push('\n');
                    for line in fence.lines() {
                        out.push_str(&format!("  {line}\n"));
                    }
                }
                out
            })
    }

    fn table() -> impl Strategy<Value = String> {
        (
            prop::collection::vec(word(), 2..4),
            prop::collection::vec(prop::collection::vec(inline(), 2..4), 1..3),
        )
            .prop_map(|(head, rows)| {
                let n = head.len();
                let mut out = format!("| {} |\n|{}\n", head.join(" | "), ":--|".repeat(n));
                for row in rows {
                    let cells: Vec<_> = row.iter().take(n).cloned().collect();
                    out.push_str(&format!("| {} |\n", cells.join(" | ")));
                }
                out
            })
    }

    fn html() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("<!-- a\n\nb -->\n".to_owned()),
            Just("<!-- one line -->\n".to_owned()),
            Just("<script>\nvar a = 1;\n\nvar b = 2;\n</script>\n".to_owned()),
            Just("<pre>\na\n\nb\n</pre>\n".to_owned()),
            Just("<div>\nhello *x*\n</div>\n".to_owned()),
            Just("<div>\n```\n</div>\n".to_owned()),
            Just("<style>a{}\n\nb{}</style>\n".to_owned()),
            Just("<?php echo 1; ?>\n".to_owned()),
            Just("<!DOCTYPE html>\n".to_owned()),
            Just("<span>inline</span> text\n".to_owned()),
        ]
    }

    fn block(domain: Domain) -> impl Strategy<Value = String> {
        prop_oneof![
            6 => prop::collection::vec(inline(), 1..4).prop_map(|l| format!("{}\n", l.join("\n"))),
            3 => (1usize..7, inline()).prop_map(|(n, t)| format!("{} {t}\n", "#".repeat(n))),
            1 => (inline(), any::<bool>()).prop_map(|(t, two)| format!("{t}\n{}\n", if two { "---" } else { "===" })),
            3 => list(domain, true),
            2 => prop::collection::vec(inline(), 1..3)
                .prop_map(|l| l.iter().map(|l| format!("> {l}\n")).collect::<String>()),
            1 => table(),
            4 => fence(domain),
            2 => html(),
            1 => proptest::sample::select(vec!["---\n", "***\n", "___\n", "- - -\n"]).prop_map(str::to_owned),
            1 => inline().prop_map(|t| format!("    {t}\n")),
            1 => Just("[r]: https://e.x \"t\"\n".to_owned()),
            1 => Just("\\# escaped\n".to_owned()),
        ]
    }

    /// A note without the constructs that swallow blank lines (fences, HTML
    /// blocks, front matter) and without a rule to start it: its blocks are
    /// separated by blank lines and nothing else.
    pub fn calm_notes() -> impl Strategy<Value = String> {
        let domain = Domain {
            aliases: false,
            valid: false,
        };
        let block = prop_oneof![
            6 => prop::collection::vec(inline(), 1..4).prop_map(|l| format!("{}\n", l.join("\n"))),
            3 => (1usize..7, inline()).prop_map(|(n, t)| format!("{} {t}\n", "#".repeat(n))),
            3 => list(domain, false),
            2 => table(),
        ];
        prop::collection::vec((block, prop_oneof![Just("\n"), Just("\n\n")]), 1..10).prop_map(
            |blocks| {
                let mut out = String::from("A first paragraph of the note.\n\n");
                for (block, separator) in blocks {
                    out.push_str(&block);
                    out.push_str(separator);
                }
                out
            },
        )
    }

    fn notes_in(domain: Domain) -> impl Strategy<Value = String> {
        let separator = prop_oneof![
            8 => Just("\n"),
            1 => Just(""),
            1 => Just("\n\n"),
        ];
        (
            prop::option::weighted(
                0.15,
                prop_oneof![
                    Just("---\ntitle: a\n\ntags: [b]\n---\n".to_owned()),
                    Just("+++\na = 1\n+++\n".to_owned()),
                    Just("---\na: 1\n---\n".to_owned()),
                ],
            ),
            prop::collection::vec((block(domain), separator), 0..8),
        )
            .prop_map(|(front, blocks)| {
                let mut out = String::new();
                if let Some(front) = front {
                    out.push_str(&front);
                    out.push('\n');
                }
                for (block, separator) in blocks {
                    out.push_str(&block);
                    out.push_str(separator);
                    out.push('\n');
                }
                out
            })
    }

    /// A Markdown note: blocks separated by blank lines, now and then by
    /// none, with front matter at the top of some. With `aliases` false, the
    /// fences name their grammars by the grammars' own names.
    pub fn notes(aliases: bool) -> impl Strategy<Value = String> {
        notes_in(Domain {
            aliases,
            valid: false,
        })
    }

    /// Notes the desktop's highlighter and the engine's agree on (see
    /// `markdown_source` in `kanso-app`): fences name grammars by their own
    /// names and hold valid code, and few things in the note are broken.
    pub fn agreeing_notes() -> impl Strategy<Value = String> {
        notes_in(Domain {
            aliases: false,
            valid: true,
        })
    }
}
