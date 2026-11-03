//! The blocks are the page in the app: the same headings, task boxes, copy
//! numbers and links, and each construct as the page shows it.

use super::*;
use crate::html::app::{DueChip, QueryRow as ViewRow, QueryTable, QueryViews};
use crate::html::{CodeSpan, PageOptions, page_with};

fn none(_: &str) -> Option<(String, Vec<u8>)> {
    None
}

fn slots(markdown: &str) -> Vec<app::TaskSlot> {
    kanso_core::tasks::task_items(markdown)
        .iter()
        .map(|item| app::TaskSlot {
            line: item.line,
            checked: item.checked,
        })
        .collect()
}

fn plain_blocks(markdown: &str) -> Vec<Block> {
    let slots = slots(markdown);
    let mode = AppMode {
        tasks: &slots,
        ..AppMode::default()
    };
    blocks(markdown, &none, &Palette::LIGHT, &mode, None)
}

struct Fixed;

impl QueryViews for Fixed {
    fn run(&self, source: &str) -> QueryResult {
        match source.trim() {
            "ok" => QueryResult::Table(QueryTable {
                columns: vec!["Title".into(), "Due".into()],
                rows: vec![
                    ViewRow {
                        id: "note:a1".into(),
                        title: "A".into(),
                        cells: vec!["Sunday".into()],
                    },
                    ViewRow {
                        id: "note:b2".into(),
                        title: "  ".into(),
                        cells: vec![String::new()],
                    },
                ],
            }),
            _ => QueryResult::Invalid {
                message: "Unknown field.".into(),
            },
        }
    }
}

fn chip(url: &str) -> Option<DueChip> {
    (url == "kanso-due:2026-10-26").then(|| DueChip {
        label: "Today".into(),
        title: "Due Monday".into(),
        tone: DueTone::Today,
    })
}

/// A 2×1 PNG's worth of bytes, attached as `file:img1`.
fn attachments(id: &str) -> Option<(String, Vec<u8>)> {
    match id {
        "file:img1" => Some(("image/png".into(), vec![1, 2, 3])),
        "file:svg" => Some(("text/html".into(), vec![1])),
        _ => None,
    }
}

const FIXTURE: &str = "# Title

## Same

## Same

Text with [a note](inkdrop://note/abc), [new](kanso-new:Missing%20One), [#work](kanso-tag:work), due [@2026-10-26](kanso-due:2026-10-26) and [later](kanso-due:2026-10-27), [web](<https://example.com/a b>) [u](https://example.com/é) [mail](mailto:a@b.c) [bad](javascript:alert(1)) [frag](#same-1) [words](#Title) [lost](#nowhere) [[Left over]] [file](inkdrop://file:doc) and www.example.org.

- [ ] first task
- [x] done task
  - nested item
    1. deep
    2. [ ] deep task
- plain

```
- [ ] fenced fake task
```

```rust
fn main() {}
```

    indented code

Inline $x^2$ and $5 and $10.

$$
e = mc^2
$$

$$y$$

```math
a+b
```

```mermaid
graph TD
  A --> B
```

```mermaid
not a diagram at all
```

```excalidraw
{\"elements\":[]}
```

```query
ok
```

```query
bad
```

| a | b |
|:--|--:|
| [cell](https://x.org) | *2* |

> quoted [q](https://q.org)
>
> ```sh
> ls
> ```

![pic|200](inkdrop://file:img1) ![web](https://example.com/x.png) ![gone](inkdrop://file:nope)

<div>raw <b>html</b></div>

<img src=\"inkdrop://file:img1\" alt=\"tag\" width=\"50\">

[ref link][r] and footnote[^1].

[r]: https://ref.example/x
[^1]: The note, [see](#title).
";

/// `&amp;` and friends as characters.
fn unescape(text: &str) -> String {
    app::unescape(text)
}

/// The values after `attr="` in `html`, unescaped.
fn values_after(html: &str, prefix: &str) -> Vec<String> {
    html.split(prefix)
        .skip(1)
        .map(|rest| unescape(rest.split('"').next().unwrap_or_default()))
        .collect()
}

#[derive(Debug, Default, PartialEq)]
struct Addresses {
    headings: Vec<String>,
    tasks: Vec<u32>,
    copies: Vec<u32>,
    links: Vec<String>,
}

fn page_addresses(html: &str) -> Addresses {
    let main = html.split("<main>").nth(1).unwrap_or_default();
    let headings = main
        .split("<h")
        .skip(1)
        .filter(|rest| rest.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .filter_map(|rest| rest[1..].strip_prefix(" id=\""))
        .filter_map(|rest| {
            let (id, after) = rest.split_once('"')?;
            after.starts_with('>').then(|| unescape(id))
        })
        .collect();
    // Links only: drawings' `<use href="#…">` are not.
    let mut links: Vec<String> = main
        .split("<a ")
        .skip(1)
        .filter_map(|tag| tag.split('>').next())
        .flat_map(|tag| values_after(tag, "href=\""))
        .collect();
    let tasks = links
        .iter()
        .filter_map(|url| url.strip_prefix("kanso-task:")?.parse().ok())
        .collect();
    let copies = links
        .iter()
        .filter_map(|url| url.strip_prefix("kanso-copy:")?.parse().ok())
        .collect();
    links.retain(|url| !url.starts_with("kanso-task:") && !url.starts_with("kanso-copy:"));
    Addresses {
        headings,
        tasks,
        copies,
        links,
    }
}

fn block_addresses(blocks: &[Block]) -> Addresses {
    fn inlines(found: &mut Addresses, list: &[Inline]) {
        for inline in list {
            match inline {
                Inline::Link {
                    url,
                    inlines: inner,
                } => {
                    found.links.push(url.clone());
                    inlines(found, inner);
                }
                Inline::Due { url, .. } => found.links.push(url.clone()),
                _ => {}
            }
        }
    }
    fn walk(found: &mut Addresses, blocks: &[Block]) {
        for block in blocks {
            match block {
                Block::Heading { id, inlines: i, .. } => {
                    found.headings.push(id.clone());
                    inlines(found, i);
                }
                Block::Paragraph { inlines: i } => inlines(found, i),
                Block::List { items, .. } => {
                    for item in items {
                        if let Some(index) = item.task.and_then(|task| task.index) {
                            found.tasks.push(index);
                        }
                        walk(found, &item.blocks);
                    }
                }
                Block::Quote { blocks } => walk(found, blocks),
                Block::Code { copy, .. } => found.copies.push(*copy),
                Block::Failed { copy, .. } => found.copies.extend(*copy),
                Block::Table { head, rows, .. } => {
                    for cell in head.iter().chain(rows.iter().flatten()) {
                        inlines(found, cell);
                    }
                }
                Block::Query { rows, .. } => {
                    found.links.extend(rows.iter().map(|row| row.url.clone()));
                }
                _ => {}
            }
        }
    }
    let mut found = Addresses::default();
    walk(&mut found, blocks);
    found
}

#[test]
fn the_blocks_carry_the_pages_addresses() {
    let slots = slots(FIXTURE);
    let mode = AppMode {
        tasks: &slots,
        query: Some(&Fixed),
        due: Some(&chip),
    };
    let page = page_with(
        "Note",
        FIXTURE,
        &Palette::LIGHT,
        &attachments,
        &PageOptions {
            app: Some(&mode),
            ..PageOptions::default()
        },
    );
    let drawn = blocks(FIXTURE, &attachments, &Palette::LIGHT, &mode, None);
    let mut from_page = page_addresses(&page);
    let mut from_blocks = block_addresses(&drawn);
    // Sanity: the fixture exercises each kind of address.
    assert_eq!(from_page.headings, ["title", "same", "same-1"], "{page}");
    assert_eq!(from_page.tasks.len(), 3, "{page}");
    assert!(from_page.copies.len() >= 5, "{page}");
    assert!(from_page.links.contains(&"#same-1".to_owned()), "{page}");
    assert!(from_page.links.contains(&"#title".to_owned()), "{page}");
    assert!(
        from_page
            .links
            .contains(&"https://example.com/a%20b".to_owned())
    );
    assert!(from_page.links.contains(&"inkdrop://note/a1".to_owned()));
    assert!(from_page.links.contains(&"kanso-due:2026-10-26".to_owned()));
    assert!(
        from_page
            .links
            .contains(&"https://ref.example/x".to_owned())
    );
    assert!(!from_page.links.iter().any(|url| url.contains("javascript")));
    // Order may differ where the page moves a footnote; the sets match.
    for found in [&mut from_page, &mut from_blocks] {
        found.links.sort();
        found.tasks.sort();
        found.copies.sort();
    }
    assert_eq!(from_blocks, from_page, "{page}\n{drawn:#?}");
}

#[test]
fn headings_get_the_pages_ids() {
    let drawn = plain_blocks("# A\n\n## A\n\n> # A b!\n\n# \n");
    let ids: Vec<&str> = drawn
        .iter()
        .flat_map(|block| match block {
            Block::Heading { id, .. } => vec![id.as_str()],
            Block::Quote { blocks } => blocks
                .iter()
                .filter_map(|b| match b {
                    Block::Heading { id, .. } => Some(id.as_str()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        })
        .collect();
    let expected: Vec<String> = html::headings("# A\n\n## A\n\n> # A b!\n\n# \n")
        .into_iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(ids, expected);
    assert_eq!(ids, ["a", "a-1", "a-b", "section"]);
}

fn first_list(drawn: &[Block]) -> &[ListItem] {
    drawn
        .iter()
        .find_map(|block| match block {
            Block::List { items, .. } => Some(items.as_slice()),
            _ => None,
        })
        .expect("a list")
}

#[test]
fn task_boxes_carry_their_slot() {
    let drawn = plain_blocks("```\n- [ ] fake\n```\n\n- [ ] a\n- [x] b\n- c\n");
    let tasks: Vec<Option<TaskBox>> = first_list(&drawn).iter().map(|item| item.task).collect();
    assert_eq!(
        tasks,
        [
            Some(TaskBox {
                checked: false,
                index: Some(0)
            }),
            Some(TaskBox {
                checked: true,
                index: Some(1)
            }),
            None
        ]
    );
}

#[test]
fn task_boxes_in_a_note_with_a_lone_carriage_return_are_not_linked() {
    let md = "a\rb\n\n- [ ] a\n";
    let drawn = plain_blocks(md);
    assert_eq!(
        first_list(&drawn)[0].task,
        Some(TaskBox {
            checked: false,
            index: None
        })
    );
}

#[test]
fn tight_and_loose_lists_and_their_numbers() {
    let drawn = plain_blocks("3. a\n4. b\n\n---\n\n- a\n\n- b\n");
    assert!(matches!(
        drawn[0],
        Block::List {
            ordered: true,
            start: 3,
            tight: true,
            ..
        }
    ));
    assert_eq!(drawn[1], Block::Rule);
    assert!(matches!(
        drawn[2],
        Block::List {
            ordered: false,
            tight: false,
            ..
        }
    ));
}

fn paragraph(drawn: &[Block]) -> &[Inline] {
    drawn
        .iter()
        .find_map(|block| match block {
            Block::Paragraph { inlines } => Some(inlines.as_slice()),
            _ => None,
        })
        .expect("a paragraph")
}

fn plain_text(text: &str) -> Inline {
    super::text(text, Style::default(), false)
}

#[test]
fn links_the_page_drops_are_their_words() {
    let drawn = plain_blocks("[a](javascript:x) [b](#nowhere) [c](ftp://x) [d]()");
    let links: Vec<&Inline> = paragraph(&drawn)
        .iter()
        .filter(|inline| matches!(inline, Inline::Link { .. }))
        .collect();
    assert!(links.is_empty(), "{drawn:#?}");
    assert!(paragraph(&drawn).contains(&plain_text("a")));
}

#[test]
fn styles_flatten_onto_text() {
    let drawn = plain_blocks("***a*** ~~b~~ `c`  \nd");
    let inlines = paragraph(&drawn);
    assert_eq!(
        inlines[0],
        Inline::Text {
            text: "a".into(),
            bold: true,
            italic: true,
            strike: false,
            code: false
        }
    );
    assert!(inlines.contains(&Inline::Text {
        text: "b".into(),
        bold: false,
        italic: false,
        strike: true,
        code: false
    }));
    assert!(inlines.contains(&Inline::Text {
        text: "c".into(),
        bold: false,
        italic: false,
        strike: false,
        code: true
    }));
    assert!(inlines.contains(&Inline::Break));
}

#[test]
fn due_links_are_chips_where_the_mode_makes_one() {
    let mode = AppMode {
        due: Some(&chip),
        ..AppMode::default()
    };
    let drawn = blocks(
        "[@a](kanso-due:2026-10-26) [@b](kanso-due:2026-10-27)",
        &none,
        &Palette::LIGHT,
        &mode,
        None,
    );
    let inlines = paragraph(&drawn);
    assert_eq!(
        inlines[0],
        Inline::Due {
            url: "kanso-due:2026-10-26".into(),
            label: "Today".into(),
            title: "Due Monday".into(),
            tone: DueTone::Today
        }
    );
    assert_eq!(
        inlines[2],
        Inline::Link {
            url: "kanso-due:2026-10-27".into(),
            inlines: vec![plain_text("@b")]
        }
    );
}

#[test]
fn dollar_amounts_stay_text_and_math_is_drawn() {
    let drawn = plain_blocks("It costs $5 and $10.");
    assert!(
        !paragraph(&drawn)
            .iter()
            .any(|inline| matches!(inline, Inline::Math { .. }))
    );
    let text: String = paragraph(&drawn)
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "It costs $5 and $10.");
    let drawn = plain_blocks("Area $x^2$ here.\n\n$$y$$\n");
    assert!(
        paragraph(&drawn)
            .iter()
            .any(|inline| matches!(inline, Inline::Math { .. }))
    );
    assert!(matches!(
        drawn[1],
        Block::Figure {
            kind: FigureKind::Math,
            ..
        }
    ));
}

#[test]
fn raw_html_is_text_and_attached_img_tags_are_images() {
    let md = "<div>hi <b>x</b></div>\n\n<img src=\"inkdrop://file:img1\" alt=\"cat\" width=\"40\">\n\nan <b>inline</b> tag\n";
    let drawn = blocks(md, &attachments, &Palette::LIGHT, &AppMode::default(), None);
    assert_eq!(
        drawn[0],
        Block::Html {
            text: "<div>hi <b>x</b></div>".into()
        }
    );
    assert_eq!(
        drawn[1],
        Block::Paragraph {
            inlines: vec![Inline::Image {
                content_type: "image/png".into(),
                bytes: vec![1, 2, 3],
                alt: "cat".into(),
                width: Some(40)
            }]
        }
    );
    let words: String = paragraph(&drawn[2..])
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(words, "an <b>inline</b> tag");
}

#[test]
fn images_are_attached_bytes_or_their_words() {
    let md = "![cat|120](inkdrop://file:img1) ![gone](inkdrop://file:nope) ![web|50](https://x.org/a.png) ![bad](inkdrop://file:svg)";
    let drawn = blocks(md, &attachments, &Palette::LIGHT, &AppMode::default(), None);
    let inlines = paragraph(&drawn);
    assert_eq!(
        inlines[0],
        Inline::Image {
            content_type: "image/png".into(),
            bytes: vec![1, 2, 3],
            alt: "cat".into(),
            width: Some(120)
        }
    );
    assert!(inlines.contains(&Inline::Missing {
        text: "gone".into()
    }));
    assert!(inlines.contains(&Inline::Missing { text: "web".into() }));
    assert!(inlines.contains(&Inline::Missing { text: "bad".into() }));
}

#[test]
fn wiki_links_left_over_show_their_label() {
    let drawn = plain_blocks("See [[Plans#Q3|the plan]] and [[A & B]] now `[[code]]`");
    let inlines = paragraph(&drawn);
    assert!(inlines.contains(&Inline::Missing {
        text: "the plan".into()
    }));
    assert!(inlines.contains(&Inline::Missing {
        text: "A & B".into()
    }));
    assert!(inlines.contains(&super::text("[[code]]", Style::default(), true)));
}

#[test]
fn tables_keep_their_alignment_and_cells() {
    let drawn = plain_blocks("| a | b | c |\n|:--|:-:|--:|\n| 1 | 2 | 3 |\n");
    let Block::Table { aligns, head, rows } = &drawn[0] else {
        panic!("{drawn:#?}")
    };
    assert_eq!(aligns, &[Align::Left, Align::Center, Align::Right]);
    assert_eq!(head[0], vec![plain_text("a")]);
    assert_eq!(rows[0][2], vec![plain_text("3")]);
}

#[test]
fn query_fences_are_views() {
    let mode = AppMode {
        query: Some(&Fixed),
        ..AppMode::default()
    };
    let drawn = blocks(
        "```query\nok\n```\n\n```query\nbad\n```\n",
        &none,
        &Palette::LIGHT,
        &mode,
        None,
    );
    assert_eq!(
        drawn[0],
        Block::Query {
            columns: vec!["Title".into(), "Due".into()],
            rows: vec![
                QueryRow {
                    url: "inkdrop://note/a1".into(),
                    title: "A".into(),
                    cells: vec!["Sunday".into()]
                },
                QueryRow {
                    url: "inkdrop://note/b2".into(),
                    title: "Untitled".into(),
                    cells: vec![String::new()]
                }
            ]
        }
    );
    assert_eq!(
        drawn[1],
        Block::QueryInvalid {
            code: "bad".into(),
            message: "Unknown field.".into()
        }
    );
    // Without views, a query fence is code.
    assert!(matches!(
        plain_blocks("```query\nok\n```\n")[0],
        Block::Code { .. }
    ));
}

#[test]
fn drawings_that_fail_show_their_code_and_why() {
    let drawn = plain_blocks(
        "```rust\nx\n```\n\n```mermaid\nnot a diagram at all\n```\n\n```excalidraw\n{\"elements\":[]}\n```\n\n[a.excalidraw](inkdrop://file:gone)\n",
    );
    let Block::Failed {
        badge, code, copy, ..
    } = &drawn[1]
    else {
        panic!("{drawn:#?}")
    };
    assert!(!badge.is_empty());
    assert_eq!(code, "not a diagram at all");
    assert_eq!(*copy, Some(1));
    assert_eq!(drawn[2], Block::EmptyDrawing);
    assert_eq!(
        drawn[3],
        Block::Failed {
            badge: "Couldn't draw this".into(),
            detail: "The drawing's file is missing.".into(),
            language: None,
            code: "[a.excalidraw](inkdrop://file:gone)".into(),
            copy: None
        }
    );
}

/// Colors a few byte ranges, overlapping and off character boundaries.
struct Patchy;

impl Highlighter for Patchy {
    fn highlight(&self, _: &str, code: &str) -> Option<Vec<CodeSpan>> {
        let span = |start, end, color| CodeSpan {
            start,
            end,
            color: Some(color),
            bold: color == 2,
            italic: false,
        };
        Some(vec![
            span(2, 4, 1),
            span(3, 7, 2),
            span(9, 10, 3),
            span(code.len() + 4, code.len() + 9, 4),
        ])
    }
}

#[test]
fn code_runs_cover_the_whole_code_in_order() {
    let code = "ab😀cdefghij";
    let drawn = blocks(
        &format!("```rust\n{code}\n```\n"),
        &none,
        &Palette::LIGHT,
        &AppMode::default(),
        Some(&Patchy),
    );
    let Block::Code {
        language,
        runs,
        copy,
    } = &drawn[0]
    else {
        panic!("{drawn:#?}")
    };
    assert_eq!(language.as_deref(), Some("rust"));
    assert_eq!(*copy, 0);
    let joined: String = runs.iter().map(|run| run.text.as_str()).collect();
    assert_eq!(joined, code);
    assert!(runs.iter().all(|run| !run.text.is_empty()));
    assert_eq!(runs[0].color, None);
    assert!(runs.iter().any(|run| run.color == Some(2) && run.bold));
    // Uncolored code is one run.
    let drawn = plain_blocks("```\nplain\n```\n");
    assert!(
        matches!(&drawn[0], Block::Code { language: None, runs, .. } if runs == &[CodeRun { text: "plain".into(), color: None, bold: false, italic: false }])
    );
}

#[test]
fn urls_are_written_as_the_page_writes_them() {
    assert_eq!(
        page_url("https://a b/é?q=1#x"),
        "https://a%20b/%C3%A9?q=1#x"
    );
    assert_eq!(page_url("a%20b%zz%"), "a%20b%zz%25");
    assert_eq!(page_url("kanso-tag:work"), "kanso-tag:work");
    assert_eq!(
        page_url("https://x.org/a b?x=1&y=ü"),
        "https://x.org/a%20b?x=1&y=%C3%BC"
    );
}

#[test]
fn urls_with_spaces_ampersands_and_accents_match_the_page() {
    let md = "[a](<https://x.org/a b?x=1&y=ü>) [b](https://x.org/?q=a&amp;b=é) [c](#Caf%C3%A9)\n\n# Café\n";
    let mode = AppMode::default();
    let options = PageOptions {
        app: Some(&mode),
        ..PageOptions::default()
    };
    let page = page_with("N", md, &Palette::LIGHT, &none, &options);
    let from_page = page_addresses(&page);
    let from_blocks = block_addresses(&plain_blocks(md));
    assert!(
        from_page
            .links
            .contains(&"https://x.org/a%20b?x=1&y=%C3%BC".to_owned()),
        "{page}"
    );
    assert_eq!(from_blocks.links, from_page.links, "{page}");
}

#[test]
fn ragged_table_rows_have_a_cell_for_each_column() {
    let drawn = plain_blocks("| a | b |\n|---|---|\n| 1 |\n| 1 | 2 | 3 |\n");
    let Block::Table { aligns, head, rows } = &drawn[0] else {
        panic!("{drawn:#?}")
    };
    assert_eq!(aligns.len(), 2);
    assert_eq!(head.len(), 2);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec![vec![plain_text("1")], vec![]]);
    assert_eq!(rows[1], vec![vec![plain_text("1")], vec![plain_text("2")]]);
}

/// How many quotes deep `blocks` go.
fn quote_depth(blocks: &[Block]) -> usize {
    let mut depth = 0;
    let mut level = blocks;
    while let Some(inner) = level.iter().find_map(|block| match block {
        Block::Quote { blocks } => Some(blocks),
        _ => None,
    }) {
        depth += 1;
        level = inner;
    }
    depth
}

#[test]
fn deep_quotes_are_flattened_not_overflowed() {
    let md = format!("{}deep\n", ">".repeat(2000));
    let drawn = plain_blocks(&md);
    assert_eq!(quote_depth(&drawn), MAX_DEPTH);
    let mut level = drawn.as_slice();
    for _ in 0..MAX_DEPTH {
        let Some(Block::Quote { blocks }) = level.first() else {
            panic!("{level:#?}")
        };
        level = blocks;
    }
    assert_eq!(
        level,
        [Block::Paragraph {
            inlines: vec![plain_text("deep")]
        }]
    );
}

#[test]
fn deep_lists_are_flattened_not_overflowed() {
    // A list in a list, 2000 deep, on one line.
    let md = format!("{}deep\n", "- ".repeat(2000));
    let drawn = plain_blocks(&md);
    let mut depth = 0;
    let mut level = drawn.as_slice();
    while let Some(Block::List { items, .. }) = level.first() {
        depth += 1;
        level = &items[0].blocks;
    }
    assert_eq!(depth, MAX_DEPTH);
    assert_eq!(
        level,
        [Block::Paragraph {
            inlines: vec![plain_text("deep")]
        }]
    );
}

#[test]
fn a_lone_carriage_return_is_a_space() {
    let drawn = plain_blocks("a\rb\n");
    assert_eq!(paragraph(&drawn), [plain_text("a b")]);
}

#[test]
fn reference_labels_match_as_markdown_matches_them() {
    // A no-break space is not collapsed: the labels differ.
    let drawn = plain_blocks("[x][a\u{a0}b]\n\n[a b]: https://x.org\n");
    assert!(
        !block_addresses(&drawn)
            .links
            .contains(&"https://x.org".to_owned()),
        "{drawn:#?}"
    );
    let drawn = plain_blocks("[x][A  b]\n\n[a b]: https://x.org\n");
    assert_eq!(block_addresses(&drawn).links, ["https://x.org"]);
}

#[test]
fn footnotes_follow_the_note() {
    let drawn = plain_blocks("A[^n] b.\n\n[^n]: Said.\n");
    assert!(paragraph(&drawn).contains(&plain_text("[1]")));
    assert!(
        matches!(drawn.last(), Some(Block::List { ordered: true, items, .. }) if items.len() == 1)
    );
}

fn sourced(markdown: &str) -> Vec<(Block, Option<std::ops::Range<usize>>)> {
    let slots = slots(markdown);
    let mode = AppMode {
        tasks: &slots,
        ..AppMode::default()
    };
    blocks_with_sources(markdown, &none, &Palette::LIGHT, &mode, None)
}

#[test]
fn each_top_level_block_says_where_it_is_written() {
    let md = "# Head\n\n- a\n  - b\n\n```rust\nfn x() {}\n```\n\n[r]: https://example.com\n\n| a |\n| - |\n| 1 |\n\nCite[^1].\n\n[^1]: note\n";
    let drawn = sourced(md);
    let written: Vec<Option<&str>> = drawn
        .iter()
        .map(|(_, source)| source.clone().map(|range| &md[range]))
        .collect();
    assert_eq!(
        written,
        vec![
            Some("# Head"),
            // A list's span runs to the line after it.
            Some("- a\n  - b\n"),
            Some("```rust\nfn x() {}\n```"),
            Some("| a |\n| - |\n| 1 |"),
            Some("Cite[^1]."),
            None,
            None,
        ]
    );
    assert!(matches!(drawn[0].0, Block::Heading { level: 1, .. }));
    assert!(matches!(drawn[2].0, Block::Code { .. }));
    assert!(matches!(drawn[3].0, Block::Table { .. }));
    // The footnotes after the note: a rule and their list.
    assert!(matches!(drawn[5].0, Block::Rule));
    assert!(matches!(drawn[6].0, Block::List { ordered: true, .. }));
}

mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Whatever the highlighter says, the runs are the code, in order.
        #[test]
        fn prop_code_runs_are_the_code(
            code in "\\PC{0,24}",
            spans in prop::collection::vec((0usize..40, 0usize..40, any::<bool>()), 0..6),
        ) {
            let spans: Vec<CodeSpan> = spans
                .into_iter()
                .map(|(start, end, bold)| CodeSpan { start, end, color: Some(7), bold, italic: false })
                .collect();
            let runs = code_runs(&code, &spans);
            let joined: String = runs.iter().map(|run| run.text.as_str()).collect();
            prop_assert_eq!(joined, code);
        }

        /// The sourced blocks are the blocks, and their sources are in
        /// order, apart and inside the text.
        #[test]
        fn prop_sources_are_ordered_and_inside(
            lines in prop::collection::vec(0u8..8, 0..14),
        ) {
            let md: String = lines.iter().map(|kind| match kind {
                0 => "- [ ] a\n",
                1 => "# h\n",
                2 => "\n```\ncode\n```\n\n",
                3 => "\n> q\n> r\n\n",
                4 => "| a |\n| - |\n",
                5 => "\n$$\nx\n$$\n\n",
                6 => "[^n]: f\n\nsee[^n]\n",
                _ => "\npara é🙂\n\n",
            }).collect();
            let slots = slots(&md);
            let mode = AppMode { tasks: &slots, ..AppMode::default() };
            let drawn = blocks(&md, &none, &Palette::LIGHT, &mode, None);
            let with = sourced(&md);
            prop_assert_eq!(
                with.iter().map(|(block, _)| block.clone()).collect::<Vec<_>>(),
                drawn
            );
            let mut last_end = 0;
            let mut last: Option<std::ops::Range<usize>> = None;
            for (_, source) in &with {
                let Some(source) = source else { continue };
                prop_assert!(source.start <= source.end && source.end <= md.len());
                prop_assert!(md.is_char_boundary(source.start) && md.is_char_boundary(source.end));
                // Blocks from one node share its source; others follow it.
                if last.as_ref() != Some(source) {
                    prop_assert!(source.start >= last_end, "{:?} after {}", source, last_end);
                    last_end = source.end;
                    last = Some(source.clone());
                }
            }
        }

        /// Task indices and copy numbers match the page's for notes of
        /// tasks and code in lists, quotes and fences.
        #[test]
        fn prop_tasks_and_copies_match_the_page(
            lines in prop::collection::vec(0u8..6, 0..12),
        ) {
            let md: String = lines.iter().map(|kind| match kind {
                0 => "- [ ] a\n",
                1 => "- [x] b\n  - [ ] c\n",
                2 => "\n```\n- [ ] fake\n```\n\n",
                3 => "\n> - [ ] q\n\n",
                4 => "\n    code\n\n",
                _ => "\npara\n\n",
            }).collect();
            let slots = slots(&md);
            let mode = AppMode { tasks: &slots, ..AppMode::default() };
            let page = page_with("N", &md, &Palette::LIGHT, &none, &PageOptions { app: Some(&mode), ..PageOptions::default() });
            let drawn = blocks(&md, &none, &Palette::LIGHT, &mode, None);
            let from_page = page_addresses(&page);
            let from_blocks = block_addresses(&drawn);
            prop_assert_eq!(from_blocks.tasks, from_page.tasks);
            prop_assert_eq!(from_blocks.copies, from_page.copies);
        }
    }
}

#[test]
fn html_comments_are_hidden_and_other_html_is_not() {
    let md = "<!-- kanso:transcript -->\n[00:01](kanso-audio://file:a?t=1) Hi\n<!-- /kanso:transcript -->\n\na <!-- note --> b\n\n<!-- x --> <div>y</div>\n";
    let drawn = plain_blocks(md);
    let shown = format!("{drawn:?}");
    assert!(!shown.contains("kanso:transcript"), "{shown}");
    assert!(!shown.contains("note"), "{shown}");
    assert!(
        shown.contains("00:01") && shown.contains("<div>y</div>"),
        "{shown}"
    );
    assert!(only_comments("<!-- a -->\n<!-- b -->"));
    assert!(!only_comments("<!-- a"));
    assert!(!only_comments(""));
}
