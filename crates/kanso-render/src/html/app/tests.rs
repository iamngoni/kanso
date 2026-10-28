//! The page in the app: addresses, headings, task boxes, query views, due
//! chips, copy links, and that no page carries anything active.

use proptest::prelude::*;

use super::*;
use crate::html::{PageOptions, body_with, page_with};
use crate::{Failure, Palette};

fn none(_: &str) -> Option<(String, Vec<u8>)> {
    None
}

/// A page's body in the app with these task slots.
fn in_app(markdown: &str, mode: &AppMode<'_>) -> String {
    let options = PageOptions {
        app: Some(mode),
        ..PageOptions::default()
    };
    body_with(markdown, &Palette::LIGHT, &none, None, &options)
}

/// The tasks of a note as the page numbers its boxes by.
fn slots(markdown: &str) -> Vec<TaskSlot> {
    kanso_core::tasks::task_items(markdown)
        .iter()
        .map(|item| TaskSlot {
            line: item.line,
            checked: item.checked,
        })
        .collect()
}

fn in_app_tasks(markdown: &str) -> String {
    let slots = slots(markdown);
    in_app(
        markdown,
        &AppMode {
            tasks: &slots,
            ..AppMode::default()
        },
    )
}

/// The real tags of an HTML page: its `<…>` pieces, which can be told from
/// text because text escapes `<`. Quotes are honored.
fn tags(html: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find('<') {
        let Some(end) = tag_end(rest, at) else { break };
        found.push(rest[at..end].to_owned());
        rest = &rest[end..];
    }
    found
}

/// Every `href`, `src` and `xlink:href` value in the tags of `html`.
fn addresses(html: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for tag in tags(html) {
        let inside = tag.trim_start_matches('<').trim_end_matches('>');
        let name_len = inside
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(inside.len());
        let (attrs, _) = attributes(&inside[name_len..]);
        for attr in attrs {
            let lower = attr.name.to_ascii_lowercase();
            if matches!(lower.as_str(), "href" | "src" | "xlink:href") {
                found.push((lower, unescape(&attr.value.unwrap_or_default())));
            }
        }
    }
    found
}

/// What a page in the app must never hold: no script, frame, form, `<meta>`
/// but the policy, event handler or address that isn't allowed.
fn assert_inert(html: &str) {
    let head_end = html.find("</head>").map_or(0, |at| at + 7);
    for tag in tags(&html[head_end..]) {
        let lower = tag.to_ascii_lowercase();
        for banned in [
            "<script", "<iframe", "<object", "<embed", "<meta", "<link", "<base", "<form",
        ] {
            assert!(!lower.starts_with(banned), "{tag}");
        }
        let (attrs, _) = attributes(
            lower[1..lower.len() - 1]
                .split_once(char::is_whitespace)
                .map_or("", |(_, rest)| rest),
        );
        for attr in attrs {
            assert!(!attr.name.starts_with("on"), "{tag}");
            assert!(!BANNED_ATTRIBUTES.contains(&attr.name.as_str()), "{tag}");
        }
    }
    for (name, url) in addresses(&html[head_end..]) {
        if name == "src" {
            assert!(is_allowed_source(&url), "src {url}");
        } else {
            assert!(
                is_allowed_link(&url) || is_allowed_source(&url),
                "{name} {url}"
            );
        }
    }
    assert!(html.contains("default-src 'none'"));
}

#[test]
fn links_into_the_app_keep_their_addresses_and_the_rest_lose_theirs() {
    let md = "[n](inkdrop://note/abc) [h](inkdrop://note/abc#Head%20x) [f](inkdrop://file:xyz) \
              [new](kanso-new:Plan%20B) [t](kanso-tag:work%2Fa) [w](https://example.com/a?b=1&c=2) \
              [m](mailto:a@b.co) [js](javascript:alert(1)) [file](file:///etc/passwd) \
              [data](data:text/html,hi) [v](vbscript:x) [x](&#106;avascript:alert(1)) [s](#top)";
    let html = in_app(md, &AppMode::default());
    for kept in [
        r#"<a href="inkdrop://note/abc">n</a>"#,
        r#"<a href="inkdrop://note/abc#Head%20x">h</a>"#,
        r#"<a href="inkdrop://file:xyz">f</a>"#,
        r#"<a href="kanso-new:Plan%20B">new</a>"#,
        r#"<a href="kanso-tag:work%2Fa">t</a>"#,
        r#"<a href="https://example.com/a?b=1&amp;c=2">w</a>"#,
        r#"<a href="mailto:a@b.co">m</a>"#,
    ] {
        assert!(html.contains(kept), "{kept}\n{html}");
    }
    for dropped in ["js", "file", "data", "v", "x"] {
        assert!(
            html.contains(&format!("<a>{dropped}</a>")),
            "{dropped}\n{html}"
        );
    }
    // `#top` names no heading here.
    assert!(html.contains("<a>s</a>"));
    // Without the mode, the same note keeps none of the app's addresses.
    let plain = body_with(md, &Palette::LIGHT, &none, None, &PageOptions::default());
    assert!(!plain.contains("inkdrop://") && !plain.contains("kanso-"));
}

#[test]
fn publishing_output_is_not_changed_by_the_mode_existing() {
    let md = "# Title\n\n- [ ] a\n- [x] b\n\n```rust\nfn main() {}\n```\n\n```query\nfrom: x\n```\n\n[w](https://x.org) [n](inkdrop://note/abc)\n";
    let page = page_with("T", md, &Palette::LIGHT, &none, &PageOptions::default());
    assert!(page.contains("<h1>Title</h1>"));
    assert!(page.contains(r#"<input type="checkbox" disabled="" />"#));
    assert!(page.contains(r#"<pre><code class="language-rust">"#));
    assert!(page.contains(r#"<pre><code class="language-query">"#));
    assert!(page.contains("<a>n</a>"));
    for app_only in [
        "kanso-task",
        "kanso-copy",
        "kanso-box",
        "kanso-due",
        "id=\"title\"",
    ] {
        assert!(!page.contains(app_only), "{app_only}");
    }
    let again = page_with("T", md, &Palette::LIGHT, &none, &PageOptions::default());
    assert_eq!(page, again);
}

proptest! {
    /// Without the mode, a page has none of the app's markup, whatever the
    /// note holds.
    #[test]
    fn prop_published_pages_have_no_app_markup(md in "[ -~\n]{0,200}") {
        let page = page_with("T", &md, &Palette::DARK, &none, &PageOptions::default());
        prop_assert!(!page.contains("kanso-task:"));
        prop_assert!(!page.contains("kanso-copy:"));
        prop_assert!(!page.contains("class=\"kanso-box\""));
        prop_assert!(!page.contains("<h1 id="));
    }
}

#[test]
fn headings_have_stable_ids_and_same_page_links_find_them() {
    let md = "# My Title\n\n## Café au lait!\n\n## my   title\n\n[a](#my-title) [b](#Café%20au%20lait!) [c](#nowhere)\n";
    let html = in_app(md, &AppMode::default());
    assert!(
        html.contains(r#"<h1 id="my-title">My Title</h1>"#),
        "{html}"
    );
    assert!(html.contains(r#"<h2 id="café-au-lait">"#));
    assert!(html.contains(r#"<h2 id="my-title-1">my   title</h2>"#));
    assert!(html.contains(r##"<a href="#my-title">a</a>"##));
    assert!(html.contains(r##"<a href="#café-au-lait">b</a>"##));
    assert!(html.contains("<a>c</a>"));
    let ids = headings(md);
    assert_eq!(
        ids.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
        ["my-title", "café-au-lait", "my-title-1"]
    );
}

proptest! {
    /// Headings that `[[Title#Heading]]` treats as one (case and spacing
    /// aside) get one slug, every page's ids are their own, and a slug is
    /// safe to put in an address.
    #[test]
    fn prop_slugs_follow_the_wiki_link_rule(words in prop::collection::vec("[A-Za-z0-9 ]{1,6}", 1..5), pad in " {0,3}") {
        let text = words.join(" ");
        let shouted = format!("{pad}{}{pad}", text.to_uppercase().replace(' ', "  "));
        prop_assert_eq!(heading_slug(&text), heading_slug(&shouted));
        prop_assert!(!heading_slug(&text).is_empty());
        prop_assert!(!heading_slug(&text).contains(char::is_whitespace));
    }

    #[test]
    fn prop_heading_ids_are_unique(titles in prop::collection::vec("[a-c ]{0,3}", 0..12)) {
        let md: String = titles.iter().map(|t| format!("# x{t}\n\n")).collect();
        let found = headings(&md);
        let mut seen = std::collections::HashSet::new();
        for h in &found {
            prop_assert!(seen.insert(h.id.clone()), "{:?}", found);
        }
        let html = in_app(&md, &AppMode::default());
        for h in &found {
            let tag = format!(r#"id="{}""#, h.id);
            prop_assert!(html.contains(&tag));
        }
    }
}

#[test]
fn task_boxes_are_links_numbered_by_the_task_list() {
    let md = "- [ ] one\n  - [x] two\n- [ ] three\n\n```\n- [ ] not a task\n```\n\n> - [x] quoted\n\n1. [ ] numbered\n";
    let html = in_app_tasks(md);
    let items = kanso_core::tasks::task_items(md);
    assert_eq!(items.len(), 5);
    for (n, item) in items.iter().enumerate() {
        let class = if item.checked {
            "kanso-task kanso-task-done"
        } else {
            "kanso-task"
        };
        let link = format!(r#"<a class="{class}" href="kanso-task:{n}""#);
        assert!(html.contains(&link), "{link}\n{html}");
    }
    assert!(!html.contains("kanso-task:5"));
    assert!(!html.contains("<input"));
    // The stylesheet keeps a tap target of at least 44 points.
    let options = PageOptions {
        app: Some(&AppMode::default()),
        ..PageOptions::default()
    };
    let css = crate::html::stylesheet(&Palette::LIGHT, &options);
    assert!(css.contains("padding:15px"), "15 + 15 + 15 is 45 points");
}

/// The boxes of `markdown` as the preview reads it (front matter is a block of
/// its own there), numbered by the note's task list.
fn boxes(markdown: &str) -> Vec<TaskBox> {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = true;
    let root = crate::markdown_safe::to_mdast(markdown, &options).unwrap();
    task_boxes(&root, markdown, &slots(markdown))
}

/// The line `toggle_task(markdown, slot)` changes, if it changes one.
fn toggled_line(markdown: &str, slot: usize) -> Option<usize> {
    let after = kanso_core::tasks::toggle_task(markdown, slot)?;
    let mut changed = markdown
        .split('\n')
        .zip(after.split('\n'))
        .enumerate()
        .filter(|(_, (before, after))| before != after)
        .map(|(line, _)| line);
    let line = changed.next()?;
    changed.next().is_none().then_some(line)
}

#[test]
fn task_boxes_know_their_place_in_the_task_list() {
    let md =
        "- [ ] one\n  - [x] two\n- [ ] three\n\n```\n- [ ] not a task\n```\n\n1. [ ] numbered\n";
    let found = boxes(md);
    assert_eq!(
        found
            .iter()
            .map(|b| (md[b.offset..].lines().next().unwrap(), b.slot))
            .collect::<Vec<_>>(),
        [
            ("- [ ] one", Some(0)),
            ("- [x] two", Some(1)),
            ("- [ ] three", Some(2)),
            ("1. [ ] numbered", Some(3)),
        ]
    );
    assert_eq!(
        found.iter().map(|b| b.checked).collect::<Vec<_>>(),
        [false, true, false, false]
    );
}

#[test]
fn task_boxes_skip_front_matter_and_indented_code() {
    // The line scan counts both; the tree draws neither as a task, so the
    // real task is the third in the list.
    let md = "---\ntags:\n  - [ ] yaml\n---\n\npara\n\n    - [ ] code\n\n- [ ] real\n";
    let found = boxes(md);
    assert_eq!(found.len(), 1);
    assert_eq!(&md[found[0].offset..], "- [ ] real\n");
    assert_eq!(found[0].slot, Some(2));
    assert_eq!(toggled_line(md, 2), Some(found[0].line as usize));
}

#[test]
fn task_boxes_in_crlf_notes_keep_their_place() {
    let md = "- [ ] a\r\n- [x] b\r\n\r\n```\r\n- [ ] code\r\n```\r\n\r\n- [ ] c\r\n";
    let found = boxes(md);
    assert_eq!(
        found.iter().map(|b| b.slot).collect::<Vec<_>>(),
        [Some(0), Some(1), Some(2)]
    );
    for b in &found {
        assert_eq!(toggled_line(md, b.slot.unwrap()), Some(b.line as usize));
    }
}

#[test]
fn task_boxes_with_a_lone_carriage_return_have_no_place() {
    let found = boxes("a\r- [ ] x\r- [ ] y\n");
    assert!(!found.is_empty());
    assert!(found.iter().all(|b| b.slot.is_none()));
}

#[test]
fn task_boxes_the_line_scan_does_not_see_have_no_place() {
    // The inner box starts mid-line, where the task list doesn't look, so
    // the next line's task is the list's first.
    let found = boxes("- - [ ] x\n- [ ] y\n");
    assert_eq!(
        found.iter().map(|b| b.slot).collect::<Vec<_>>(),
        [None, Some(0)]
    );
}

#[test]
fn tasks_sharing_a_line_are_not_told_apart() {
    // The task list has one slot to a line, so two tasks of the tree on one
    // line can't each be given it.
    let task = |offset, line| TaskNode {
        offset,
        line,
        checked: false,
    };
    let slots = [TaskSlot {
        line: 0,
        checked: false,
    }];
    assert_eq!(match_slots(&[task(0, 0), task(6, 0)], &slots), [None, None]);
    assert_eq!(match_slots(&[task(0, 0)], &slots), [Some(0)]);
}

proptest! {
    /// Agreement: whatever a note is made of, a box that has a place is the
    /// task `toggle_task` changes at that place, and on its own line.
    #[test]
    fn prop_a_box_with_a_place_is_the_task_that_toggles(
        lines in prop::collection::vec(
            prop_oneof![
                Just("- [ ] open"), Just("- [x] done"), Just("  - [ ] nested"),
                Just("1. [X] numbered"), Just("> - [ ] quoted"), Just("```"), Just("~~~"),
                Just("    - [ ] indented"), Just("- not a task"), Just("---"), Just(""),
                Just("text"),
            ],
            0..14,
        ),
        eol in prop_oneof![Just("\n"), Just("\r\n")],
    ) {
        let md: String = lines.iter().map(|line| format!("{line}{eol}")).collect();
        for b in boxes(&md) {
            if let Some(slot) = b.slot {
                prop_assert_eq!(toggled_line(&md, slot), Some(b.line as usize), "{:?}", md);
            }
        }
    }
}

#[test]
fn boxes_the_task_list_does_not_name_stay_plain() {
    // No slots: nothing to number by.
    let html = in_app("- [ ] a\n", &AppMode::default());
    assert!(html.contains("<input"));
    assert!(!html.contains("kanso-task:"));
    // Indented code that the line scan counts but Markdown doesn't draw.
    let md = "para\n\n    - [ ] code\n\n- [ ] real\n";
    let html = in_app_tasks(md);
    assert_eq!(html.matches("kanso-task:").count(), 1);
    assert!(
        html.contains("kanso-task:1"),
        "the real task is the second of the list\n{html}"
    );
    // A lone carriage return makes the two scans count lines differently.
    let html = in_app_tasks("a\r- [ ] x\r- [ ] y\n");
    assert!(!html.contains("kanso-task:"));
}

/// A line of a generated note.
fn note_line() -> impl Strategy<Value = String> {
    let word = "[a-z]{1,6}";
    prop_oneof![
        word.prop_map(|w| format!("{w} {w}\n")),
        (0usize..2, prop::bool::ANY, word).prop_map(|(depth, done, w)| format!(
            "{}- [{}] {w}\n",
            "  ".repeat(depth),
            if done { 'x' } else { ' ' }
        )),
        (prop::bool::ANY, word)
            .prop_map(|(done, w)| format!("> - [{}] {w}\n", if done { 'x' } else { ' ' })),
        (prop::bool::ANY, word)
            .prop_map(|(done, w)| format!("1. [{}] {w}\n", if done { 'x' } else { ' ' })),
        word.prop_map(|w| format!("```\n- [ ] {w}\n```\n")),
        word.prop_map(|w| format!("~~~\n- [x] {w}\n~~~\n")),
        word.prop_map(|w| format!("`- [ ] {w}`\n")),
        word.prop_map(|w| format!("# {w}\n")),
        Just("\n".to_owned()),
    ]
}

proptest! {
    /// Each box link `n` is task `n` of the list: its place, its ticked
    /// state and its words, whatever surrounds it; links are in order.
    #[test]
    fn prop_box_links_are_the_task_list(lines in prop::collection::vec(note_line(), 0..24)) {
        let md: String = lines.concat();
        let items = kanso_core::tasks::task_items(&md);
        let html = in_app_tasks(&md);
        let mut last = None;
        let mut linked = 0;
        let mut rest = html.as_str();
        while let Some(at) = rest.find("<a class=\"kanso-task") {
            let tag = &rest[at..];
            let done = tag.starts_with("<a class=\"kanso-task kanso-task-done\"");
            let n: usize = tag
                .split("href=\"kanso-task:").nth(1).unwrap()
                .split('"').next().unwrap().parse().unwrap();
            prop_assert!(last.is_none_or(|l| n > l), "in order");
            last = Some(n);
            prop_assert!(n < items.len());
            prop_assert_eq!(items[n].checked, done);
            let after = tag.split("</a>").nth(1).unwrap();
            let text = &items[n].text;
            prop_assert!(after.starts_with(&format!(" {text}")), "{} / {}", text, after);
            linked += 1;
            rest = &tag[1..];
        }
        // Every task the scan counts is a box unless Markdown draws it as
        // code (which this generator never writes), so none is missing.
        prop_assert_eq!(linked, items.len(), "{}\n{}", md, html);
        prop_assert!(!html.contains("<input"));
    }
}

#[test]
fn query_fences_are_tables_of_links() {
    struct Fixed;
    impl QueryViews for Fixed {
        fn run(&self, source: &str) -> QueryResult {
            match source.trim() {
                "ok" => QueryResult::Table(QueryTable {
                    columns: vec!["Title".into(), "Due".into()],
                    rows: vec![QueryRow {
                        id: "note:a1".into(),
                        title: "A <b> & c".into(),
                        cells: vec!["Sunday".into()],
                    }],
                }),
                "empty" => QueryResult::Table(QueryTable {
                    columns: vec!["Title".into()],
                    rows: vec![],
                }),
                _ => QueryResult::Invalid {
                    message: "Unknown field.".into(),
                },
            }
        }
    }
    let mode = AppMode {
        query: Some(&Fixed),
        ..AppMode::default()
    };
    let html = in_app(
        "```query\nok\n```\n\n```query\nempty\n```\n\n```query\nbad <x>\n```\n",
        &mode,
    );
    assert!(html.contains(r#"<th>Title</th><th>Due</th>"#), "{html}");
    assert!(html.contains(
        r#"<td><a href="inkdrop://note/a1">A &lt;b&gt; &amp; c</a></td><td>Sunday</td>"#
    ));
    assert!(html.contains("No notes match."));
    assert!(html.contains(r#"<span class="kanso-query-badge">Query</span>"#));
    assert!(html.contains("bad &lt;x&gt;"));
    assert!(html.contains("Unknown field."));
    assert!(!html.contains("<b>"));
    // Without the hook, the fence is the code it is.
    let plain = in_app("```query\nok\n```\n", &AppMode::default());
    assert!(plain.contains("kanso-code") && !plain.contains("<table"));
}

proptest! {
    /// Whatever a query returns is text in the page: no markup from it.
    #[test]
    fn prop_query_results_are_escaped(title in "\\PC{0,20}", cell in "\\PC{0,20}", message in "\\PC{0,20}") {
        let table = query_html("src", &QueryResult::Table(QueryTable {
            columns: vec![title.clone()],
            rows: vec![QueryRow { id: format!("note:{title}"), title: title.clone(), cells: vec![cell] }],
        }));
        let bad = query_html(&title, &QueryResult::Invalid { message });
        for html in [table, bad] {
            let page = format!("<main>{html}</main>");
            prop_assert_eq!(sanitize(&page), page.clone());
            for tag in tags(&page) {
                let name = tag.trim_start_matches(['<', '/']).split([' ', '>']).next().unwrap().to_owned();
                prop_assert!(["main","div","table","thead","tbody","tr","th","td","a","span","pre","code","p"].contains(&name.as_str()), "{tag}");
            }
        }
    }
}

#[test]
fn due_links_become_chips() {
    let chip = |url: &str| {
        (url == "kanso-due:2026-10-26").then(|| DueChip {
            label: "Today".into(),
            title: "Due Monday".into(),
            tone: DueTone::Today,
        })
    };
    let mode = AppMode {
        due: Some(&chip),
        ..AppMode::default()
    };
    let html = in_app(
        "[@2026-10-26](kanso-due:2026-10-26) [@x](kanso-due:2026-10-27)",
        &mode,
    );
    assert!(
        html.contains(r#"<a class="kanso-due kanso-due-today" href="kanso-due:2026-10-26" title="Due Monday">Today</a>"#),
        "{html}"
    );
    assert!(html.contains(r#"<a href="kanso-due:2026-10-27">@x</a>"#));
}

#[test]
fn code_blocks_get_copy_links_and_drawings_do_not() {
    let md = "```rust\nlet a = 1;\n```\n\n```mermaid\ngraph TD\n  A --> B\n```\n\n- item\n\n  ```sh\n  ls\n  ```\n\n> ```\n> quoted\n> ```\n\n    indented\n\n```math\nx^2\n```\n";
    let blocks = code_blocks(md);
    assert_eq!(
        blocks,
        [
            "let a = 1;",
            "graph TD\n  A --> B",
            "ls",
            "quoted",
            "indented",
            "x^2"
        ]
    );
    let html = in_app(md, &AppMode::default());
    let linked: Vec<usize> = html
        .split("href=\"kanso-copy:")
        .skip(1)
        .map(|s| s.split('"').next().unwrap().parse().unwrap())
        .collect();
    assert_eq!(linked, [0, 2, 3, 4], "{html}");
}

proptest! {
    /// The n-th copy link is the n-th code block of the note, in lists,
    /// quotes and fences of either kind; links are in order, once each.
    #[test]
    fn prop_copy_links_are_the_code_blocks(
        blocks in prop::collection::vec((0u8..4, "[a-z]{1,6}( [a-z]{1,6}){0,2}"), 0..8)
    ) {
        let md: String = blocks.iter().map(|(kind, code)| match kind {
            0 => format!("```\n{code}\n```\n\n"),
            1 => format!("- item\n\n  ~~~rust\n  {code}\n  ~~~\n\n"),
            2 => format!("> ```\n> {code}\n> ```\n\n"),
            _ => format!("para\n\n    {code}\n\n"),
        }).collect();
        let found = code_blocks(&md);
        prop_assert_eq!(found.len(), blocks.len());
        for (found, (_, code)) in found.iter().zip(&blocks) {
            prop_assert_eq!(found, code);
        }
        let html = in_app(&md, &AppMode::default());
        for n in 0..blocks.len() {
            let link = format!("href=\"kanso-copy:{n}\"");
            prop_assert_eq!(html.matches(&link).count(), 1);
        }
        let past = format!("kanso-copy:{}", blocks.len());
        prop_assert!(!html.contains(&past));
        // The default page has no copy links.
        let published = body_with(&md, &Palette::LIGHT, &none, None, &PageOptions::default());
        prop_assert!(!published.contains("kanso-copy"));
    }
}

#[test]
fn a_drawing_that_fails_shows_its_code_and_why() {
    let html = in_app(
        "```mermaid\nnot a diagram at all\n```\n\n```excalidraw\n{\"elements\":[]}\n```\n",
        &AppMode::default(),
    );
    let why = Failure::Unsupported(String::new()).badge();
    assert!(
        html.contains(&format!("class=\"kanso-query-badge\">{why}</span>"))
            || html.contains("Couldn&#39;t draw this")
            || html.contains("diagrams aren&#39;t drawn yet"),
        "{html}"
    );
    assert!(html.contains("kanso-empty-drawing"));
    assert!(html.contains("href=\"kanso-copy:0\""));
    // A drawing attached and gone.
    let html = in_app("[a.excalidraw](inkdrop://file:gone)\n", &AppMode::default());
    assert!(html.contains("Couldn&#39;t draw this"), "{html}");
}

#[test]
fn script_attempts_are_not_active() {
    let md = "<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n\n[a](javascript:alert(1))\n\n<svg onload=alert(1)><script>x</script></svg>\n\n<iframe src=\"https://evil.example\"></iframe>\n\n<meta http-equiv=\"refresh\" content=\"0;url=https://evil.example\">\n\n<style>body{background:url(https://evil.example/a.png)}</style>\n\n<a href=\"javascript:alert(1)\" onclick=\"x()\">h</a>\n\n![i](https://evil.example/t.png) ![j](javascript:alert(1)) ![k](data:text/html;base64,AAAA)\n\n[css](data:text/css,body{})\n\n`<script>`\n\n```html\n<script>alert(1)</script>\n```\n";
    let page = page_with(
        "T",
        md,
        &Palette::LIGHT,
        &none,
        &PageOptions {
            app: Some(&AppMode::default()),
            ..PageOptions::default()
        },
    );
    assert_inert(&page);
    let body = &page[page.find("<main>").unwrap()..];
    // Raw HTML is text, never markup.
    assert!(!body.contains("<script") && !body.contains("<iframe") && !body.contains("<img"));
    assert!(!body.contains("evil.example/t.png"));
    assert!(body.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    // The policy stays.
    assert!(
        page.contains(r#"content="default-src 'none'; img-src data:; style-src 'unsafe-inline'""#)
    );
}

#[test]
fn the_scrubber_removes_what_could_act() {
    let dirty = concat!(
        "<p onclick=\"x()\" class=\"a\">t</p>",
        "<script>alert(1)</script><IFRAME src=\"x\">in</IFRAME>",
        "<a href=\"JaVaScRiPt:x\">j</a><a href=\" javascript:x\">k</a><a href=\"&#106;avascript:x\">l</a>",
        "<a href=\"HTTPS://ok.example\">ok</a>",
        "<img src=\"https://x.example/a.png\" alt=\"a b\"><img src=\"data:image/png;base64,AA\" alt=\"c\">",
        "<svg viewBox=\"0 0 1 1\"><use xlink:href=\"#a\"/><use href=\"https://x.example/s.svg#a\"/>",
        "<style>.a{fill:url(#g)} .b{background:url(https://x.example/i.png)}</style>",
        "<rect style=\"fill:url(#g)\"/><rect style=\"fill:url(https://x.example/i.png)\"/>",
        "<animate attributeName=\"href\" to=\"javascript:x\"/><foreignObject><div>x</div></foreignObject></svg>",
        "<form action=\"https://x.example\"><button>b</button></form><!-- c -->",
    );
    let clean = sanitize(dirty);
    assert!(clean.contains("<p class=\"a\">t</p>"), "{clean}");
    assert!(clean.contains("<a>j</a>") && clean.contains("<a>k</a>") && clean.contains("<a>l</a>"));
    assert!(clean.contains("href=\"HTTPS://ok.example\""));
    assert!(clean.contains("<span class=\"kanso-missing\">a b</span>"));
    assert!(clean.contains("src=\"data:image/png;base64,AA\""));
    assert!(
        clean.contains("viewBox=\"0 0 1 1\""),
        "attribute case is kept: {clean}"
    );
    assert!(clean.contains("xlink:href=\"#a\""));
    assert!(clean.contains("<style></style>"));
    assert!(clean.contains("<rect />") && clean.contains("<rect style=\"fill:url(#g)\" />"));
    for gone in [
        "script",
        "iframe",
        "x.example",
        "javascript",
        "animate",
        "foreignObject",
        "<form",
        "<button",
        "<!--",
    ] {
        assert!(
            !clean.to_lowercase().contains(&gone.to_lowercase()),
            "{gone}: {clean}"
        );
    }
}

proptest! {
    /// Whatever Markdown and HTML a note holds, the page in the app carries
    /// no address but the allowed ones, no handler, no active tag.
    #[test]
    fn prop_no_other_scheme_survives(
        pieces in prop::collection::vec(
            prop_oneof![
                "[a-z ]{0,8}",
                "\\[[a-z]{1,4}\\]\\((javascript|JavaScript|file|data|vbscript|ftp|tel|blob|about|http|https|mailto|inkdrop|kanso-tag|kanso-new|kanso-copy|kanso-task|kanso-due|x-evil):[ -~]{0,12}\\)",
                "!\\[[a-z]{0,3}\\]\\([ -~]{0,16}\\)",
                "<(a|img|svg|script|iframe|meta|link|p|b)( [a-z]{1,8}(=\"?[ -~&&[^\"]]{0,12}\"?)?){0,3}/?>",
                "&#[0-9]{2,3};",
                "\\[[a-z]{1,4}\\]: [ -~]{0,16}\n",
                "<(https?|javascript):[a-z./:]{0,12}>",
                "[#>`~*_\\-\\[\\]()!\\\\|\n]{1,4}",
            ],
            0..14,
        )
    ) {
        let md = pieces.concat();
        let slots = slots(&md);
        let mode = AppMode { tasks: &slots, ..AppMode::default() };
        let page = page_with("T", &md, &Palette::LIGHT, &none, &PageOptions { app: Some(&mode), ..PageOptions::default() });
        assert_inert(&page);
    }

    /// Sanitizing is idempotent, never panics, and leaves clean text alone.
    #[test]
    fn prop_sanitize_is_idempotent(html in "[ -~\n]{0,160}") {
        let once = sanitize(&html);
        prop_assert_eq!(sanitize(&once), once);
    }

    #[test]
    fn prop_links_are_allowed_only_by_scheme(scheme in "[a-zA-Z][a-zA-Z0-9+.-]{0,10}", rest in "[ -~]{0,16}") {
        let url = format!("{scheme}:{rest}");
        if is_allowed_link(&url) {
            let lower = scheme.to_ascii_lowercase();
            prop_assert!(
                ["http", "https", "mailto", "kanso-new", "kanso-tag", "kanso-due", "kanso-task", "kanso-copy"].contains(&lower.as_str())
                    || (lower == "inkdrop" && (rest.starts_with("//note/") || rest.starts_with("//file:")))
            );
        }
        let spaced = format!(" {url}");
        prop_assert!(!is_allowed_link(&spaced));
    }
}

#[test]
fn the_app_draws_the_title_and_the_margins_itself() {
    let mode = AppMode::default();
    let options = PageOptions {
        app: Some(&mode),
        ..PageOptions::default()
    };
    let html = crate::html::page_with("Title", "Some text.", &Palette::LIGHT, &none, &options);
    assert!(html.contains("header{display:none}"));
    assert!(html.contains("body{max-width:none;margin:0;padding:0 0 1rem}"));
    // A page for anything else keeps them.
    let page = crate::html::page("Title", "Some text.", &Palette::LIGHT, &none);
    assert!(!page.contains("header{display:none}"));
}
